"""Compile a pinned OSM road extract into the sole directed freight routing graph.

This module has no PBF dependency. Source records, compilation, serialization and
the reference router are also used by small topology fixtures. Geometry crossings
never create connectivity. Costs are physical WGS84 distance, not freight capacity.
"""

from __future__ import annotations

import hashlib
import heapq
import json
import math
import os
import re
import tempfile
from collections import Counter, defaultdict
from collections.abc import Iterable, Iterator
from dataclasses import asdict, dataclass
from decimal import Decimal, InvalidOperation, localcontext
from pathlib import Path
from typing import Literal, Protocol

from pyproj import Geod

Tags = tuple[tuple[str, str], ...]
MAX_U64 = (1 << 64) - 1
ROUTING_PROFILE_VERSION = "michigan-freight-routing-v1"
MAX_RESTRICTION_PATHS = 256
MAX_RESTRICTION_EDGES = 512
ROAD_CLASSES = frozenset(
    {
        "motorway",
        "motorway_link",
        "trunk",
        "trunk_link",
        "primary",
        "primary_link",
        "secondary",
        "secondary_link",
        "tertiary",
        "tertiary_link",
    }
)
ACCESS_KEYS = ("access", "vehicle", "motor_vehicle", "hgv")
ACCESS_ALLOWED = frozenset({"yes", "permissive", "designated", "official"})
NO_RESTRICTIONS = frozenset(
    {"no_left_turn", "no_right_turn", "no_straight_on", "no_u_turn", "no_entry", "no_exit"}
)
ONLY_RESTRICTIONS = frozenset(
    {"only_left_turn", "only_right_turn", "only_straight_on", "only_u_turn"}
)
GEOD = Geod(ellps="WGS84")


class RoadNetworkError(ValueError):
    """A source or network cannot establish a closed, reproducible road graph."""


@dataclass(frozen=True, slots=True)
class Node:
    id: int
    lon_e7: int
    lat_e7: int
    tags: Tags = ()
    version: int = 1
    timestamp: str = ""


@dataclass(frozen=True, slots=True)
class Way:
    id: int
    node_ids: tuple[int, ...]
    tags: Tags
    version: int = 1
    timestamp: str = ""


@dataclass(frozen=True, slots=True)
class RestrictionMember:
    kind: str
    ref: int
    role: str


@dataclass(frozen=True, slots=True)
class Restriction:
    id: int
    members: tuple[RestrictionMember, ...]
    tags: Tags
    version: int = 1
    timestamp: str = ""


@dataclass(frozen=True, slots=True)
class VehicleProfile:
    gross_weight_kg: int
    height_mm: int
    width_mm: int
    length_mm: int
    default_maxheight_mm: int
    axle_load_kg: int | None = None
    evidence_class: str = "Designed"

    def __post_init__(self) -> None:
        values = (
            self.gross_weight_kg,
            self.height_mm,
            self.width_mm,
            self.length_mm,
            self.default_maxheight_mm,
        )
        if any(type(value) is not int or value <= 0 for value in values):
            raise RoadNetworkError("vehicle dimensions and gross weight must be positive integers")
        if self.axle_load_kg is not None and (
            type(self.axle_load_kg) is not int or self.axle_load_kg <= 0
        ):
            raise RoadNetworkError("axle load must be absent or a positive integer")
        if self.evidence_class != "Designed":
            raise RoadNetworkError("the routing vehicle profile must be explicitly Designed")


@dataclass(frozen=True, slots=True)
class SourceIdentity:
    pbf_sha256: str
    pbf_bytes: int
    pbf_url: str
    replication_timestamp: str
    footprint_sha256: str
    buffer_degrees_e7: int
    extraction_version: str
    distance_version: str

    def __post_init__(self) -> None:
        for field in (self.pbf_sha256, self.footprint_sha256):
            if not re.fullmatch(r"[0-9a-f]{64}", field):
                raise RoadNetworkError("source identity requires lowercase SHA-256 hashes")
        if self.pbf_bytes <= 0 or self.buffer_degrees_e7 < 0:
            raise RoadNetworkError("invalid source size or footprint buffer")


@dataclass(frozen=True, slots=True)
class Edge:
    id: str
    way_id: int
    from_node: int
    to_node: int
    distance_mm: int
    shape_e7: tuple[tuple[int, int], ...]
    tags: Tags
    way_version: int
    way_timestamp: str


@dataclass(frozen=True, slots=True)
class TurnRule:
    relation_id: int
    kind: Literal["no", "only"]
    prefix_edge_ids: tuple[str, ...]
    next_edge_ids: tuple[str, ...]


@dataclass(frozen=True, slots=True, order=True)
class Diagnostic:
    code: str
    osm_type: str
    osm_id: int
    detail: str


@dataclass(frozen=True, slots=True)
class RoadGraph:
    source: SourceIdentity
    profile: VehicleProfile
    nodes: tuple[Node, ...]
    edges: tuple[Edge, ...]
    turn_rules: tuple[TurnRule, ...]
    diagnostics: tuple[Diagnostic, ...]

    @property
    def edge_by_id(self) -> dict[str, Edge]:
        return {edge.id: edge for edge in self.edges}


@dataclass(frozen=True, slots=True)
class Route:
    distance_mm: int
    edge_ids: tuple[str, ...]


def _tag_map(tags: Tags) -> dict[str, str]:
    result = dict(tags)
    if len(result) != len(tags):
        raise RoadNetworkError("duplicate OSM tag key")
    return result


def _access(tags: dict[str, str], direction: str | None = None) -> bool:
    decision = "yes"
    for key in ACCESS_KEYS:
        if key in tags:
            decision = tags[key]
        if direction is not None and f"{key}:{direction}" in tags:
            decision = tags[f"{key}:{direction}"]
    return decision in ACCESS_ALLOWED


def _limit(value: str, *, mass: bool) -> Decimal | None:
    """OSM implicit units are tonnes for mass and metres for dimensions."""
    if not mass and (
        feet_inches := re.fullmatch(r"\s*([0-9]+)\s*'\s*([0-9]+(?:\.[0-9]+)?)\s*\"\s*", value)
    ):
        inches = Decimal(feet_inches[2])
        if inches >= 12:
            return None
        with localcontext() as context:
            context.prec = len(feet_inches[1]) + len(feet_inches[2]) + 4
            return Decimal(feet_inches[1]) * Decimal("304.8") + inches * Decimal("25.4")
    match = re.fullmatch(r"\s*(\d+(?:\.\d+)?)\s*(t|st|kg|lbs|m|cm|mm|ft)?\s*", value)
    if match is None:
        return None
    factors: dict[str, int | Decimal] = (
        {"t": 1000, "st": Decimal("907.18474"), "kg": 1, "lbs": Decimal("0.45359237")}
        if mass
        else {"m": 1000, "cm": 10, "mm": 1, "ft": Decimal("304.8")}
    )
    unit = match[2] or ("t" if mass else "m")
    if unit not in factors:
        return None
    try:
        with localcontext() as context:
            context.prec = len(match[1]) + len(str(factors[unit])) + 1
            return Decimal(match[1]) * factors[unit]
    except InvalidOperation:
        return None


def _limits_allow(tags: dict[str, str], profile: VehicleProfile) -> bool:
    values = {
        "maxweight": profile.gross_weight_kg,
        "maxaxleload": profile.axle_load_kg,
        "maxheight": profile.height_mm,
        "maxheight:physical": profile.height_mm,
        "maxwidth": profile.width_mm,
        "maxwidth:physical": profile.width_mm,
        "maxlength": profile.length_mm,
    }
    for key, required in values.items():
        if key not in tags:
            continue
        limit = (
            Decimal(profile.default_maxheight_mm)
            if key == "maxheight" and tags[key] == "default"
            else _limit(tags[key], mass=key in {"maxweight", "maxaxleload"})
        )
        if required is None or limit is None or limit < required:
            return False
    return True


def _explicit_unknown(tags: dict[str, str]) -> bool:
    prefixes = (
        *ACCESS_KEYS,
        "oneway",
        "maxweight",
        "maxaxleload",
        "maxheight",
        "maxwidth",
        "maxlength",
    )
    for key in tags:
        if key.startswith(prefixes) and key.endswith(":conditional"):
            return True
        if key.startswith(
            ("maxweight:", "maxaxleload:", "maxheight:", "maxwidth:", "maxlength:")
        ) and key not in {"maxheight:physical", "maxwidth:physical"}:
            return True
    return False


def _directions(
    way: Way, profile: VehicleProfile, diagnostics: list[Diagnostic]
) -> tuple[bool, bool]:
    tags = _tag_map(way.tags)
    if tags.get("highway") not in ROAD_CLASSES or tags.get("route") == "ferry":
        return False, False
    if _explicit_unknown(tags) or not _limits_allow(tags, profile):
        diagnostics.append(
            Diagnostic(
                "vehicle_restriction_excluded",
                "way",
                way.id,
                "unsupported, conditional, unspecified vehicle parameter, or exceeded size/load restriction",
            )
        )
        return False, False
    implied = (
        "yes" if tags.get("junction") == "roundabout" or tags.get("highway") == "motorway" else "no"
    )
    oneway = tags.get(
        "oneway:hgv",
        tags.get("oneway:motor_vehicle", tags.get("oneway:vehicle", tags.get("oneway", implied))),
    )
    if oneway not in {"yes", "1", "true", "no", "0", "false", "-1"}:
        diagnostics.append(Diagnostic("unknown_direction_excluded", "way", way.id, oneway))
        return False, False
    forward = oneway != "-1" and _access(tags, "forward")
    reverse = oneway in {"no", "0", "false", "-1"} and _access(tags, "backward")
    if not forward and not reverse:
        diagnostics.append(
            Diagnostic(
                "access_excluded",
                "way",
                way.id,
                "no through-HGV direction is explicitly permitted by the profile",
            )
        )
    return forward, reverse


def _node_blocked(node: Node, profile: VehicleProfile) -> bool:
    tags = _tag_map(node.tags)
    if _explicit_unknown(tags) or not _access(tags) or not _limits_allow(tags, profile):
        return True
    if tags.get("barrier") == "lift_gate" and tags.get("access") in ACCESS_ALLOWED:
        return False
    # A barrier without a positive vehicle permission is not assumed traversable.
    return tags.get("barrier", "no") not in {
        "no",
        "entrance",
        "toll_booth",
        "border_control",
    } and not any(tags.get(key) in ACCESS_ALLOWED for key in ACCESS_KEYS[1:])


def _unique[T: (Node, Way, Restriction)](rows: Iterable[T], kind: str) -> dict[int, T]:
    output: dict[int, T] = {}
    for row in rows:
        if type(row.id) is not int or row.id <= 0 or row.id > (1 << 63) - 1:
            raise RoadNetworkError(f"invalid {kind} identity {row.id}")
        if row.id in output:
            raise RoadNetworkError(f"duplicate {kind} identity {row.id}")
        _tag_map(row.tags)
        output[row.id] = row
    return output


def _length_mm(shape: tuple[tuple[int, int], ...]) -> int:
    total = 0
    for (lon1, lat1), (lon2, lat2) in zip(shape, shape[1:], strict=False):
        _, _, metres = GEOD.inv(lon1 / 1e7, lat1 / 1e7, lon2 / 1e7, lat2 / 1e7)
        if not math.isfinite(metres) or metres < 0:
            raise RoadNetworkError("non-finite or negative WGS84 segment distance")
        total += math.floor(metres * 1000 + 0.5)
    if total <= 0 or total > MAX_U64:
        raise RoadNetworkError("road segment distance must be positive and fit u64")
    return total


def _make_edges(
    ways: dict[int, Way],
    nodes: dict[int, Node],
    directions: dict[int, tuple[bool, bool]],
    split_nodes: set[int],
    blocked_nodes: set[int],
    diagnostics: list[Diagnostic],
) -> list[Edge]:
    edges = []
    for way_id in sorted(ways):
        way = ways[way_id]
        forward, reverse = directions[way_id]
        if not forward and not reverse:
            continue
        tags = tuple(sorted(way.tags))
        start = 0
        for end in range(1, len(way.node_ids)):
            if way.node_ids[end] not in split_nodes and end != len(way.node_ids) - 1:
                continue
            ids = way.node_ids[start : end + 1]
            if not blocked_nodes.intersection(ids):
                shape = tuple((nodes[ident].lon_e7, nodes[ident].lat_e7) for ident in ids)
                try:
                    distance = _length_mm(shape)
                except RoadNetworkError as error:
                    diagnostics.append(
                        Diagnostic(
                            "invalid_distance_excluded",
                            "way",
                            way.id,
                            f"positions {start}:{end}: {error}",
                        )
                    )
                    start = end
                    continue
                for suffix, allowed, geometry in (
                    ("f", forward, shape),
                    ("r", reverse, tuple(reversed(shape))),
                ):
                    if allowed:
                        edges.append(
                            Edge(
                                f"w{way.id}:{start}:{end}:{suffix}",
                                way.id,
                                ids[0] if suffix == "f" else ids[-1],
                                ids[-1] if suffix == "f" else ids[0],
                                distance,
                                geometry,
                                tags,
                                way.version,
                                way.timestamp,
                            )
                        )
            start = end
    return edges


def _restriction_value(row: Restriction) -> tuple[str | None, str | None]:
    tags = _tag_map(row.tags)
    if tags.get("type") not in {
        "restriction",
        "restriction:hgv",
        "restriction:motor_vehicle",
        "restriction:vehicle",
    }:
        return None, None
    if set(tags.get("except", "").split(";")) & {"hgv", "motor_vehicle", "vehicle"}:
        return None, None
    relevant = (
        "restriction",
        "restriction:vehicle",
        "restriction:motor_vehicle",
        "restriction:hgv",
    )
    if any(f"{key}:conditional" in tags for key in relevant):
        return None, "conditional freight turn restriction is not evaluated"
    value = None
    for key in relevant:
        if key in tags:
            value = tags[key]
    if value is None and any(key.startswith("restriction:") for key in tags):
        return None, None  # A restriction applying only to another transport mode.
    if value not in NO_RESTRICTIONS | ONLY_RESTRICTIONS:
        return None, f"unsupported freight restriction value {value!r}"
    return value, None


def _members(
    row: Restriction,
) -> tuple[tuple[int, ...], tuple[RestrictionMember, ...], tuple[int, ...]]:
    sources = tuple(
        member.ref for member in row.members if member.role == "from" and member.kind == "w"
    )
    targets = tuple(
        member.ref for member in row.members if member.role == "to" and member.kind == "w"
    )
    via = tuple(member for member in row.members if member.role == "via")
    if (
        not sources
        or not targets
        or not via
        or any(member.role not in {"from", "to", "via", "location_hint"} for member in row.members)
    ):
        raise RoadNetworkError("missing or unsupported restriction members")
    if not (len(via) == 1 and via[0].kind == "n") and any(member.kind != "w" for member in via):
        raise RoadNetworkError("mixed node/way restriction via members")
    if any(member.kind != "w" for member in row.members if member.role in {"from", "to"}):
        raise RoadNetworkError("restriction from/to must be ways")
    return sources, via, targets


def _via_paths(
    start: int, way_ids: tuple[int, ...], outgoing: dict[int, list[Edge]]
) -> Iterator[tuple[Edge, ...]]:
    """Enumerate directed simple paths through the ordered via ways, bounded strictly."""
    stack: list[tuple[int, int, tuple[Edge, ...], frozenset[str]]] = [(start, 0, (), frozenset())]
    expanded = 0
    while stack:
        node, index, path, used = stack.pop()
        expanded += 1
        if expanded > MAX_RESTRICTION_PATHS * MAX_RESTRICTION_EDGES:
            raise RoadNetworkError("restriction path expansion limit")
        if len(path) > MAX_RESTRICTION_EDGES:
            raise RoadNetworkError("restriction edge sequence limit")
        for edge in outgoing.get(node, ()):
            if edge.way_id != way_ids[index] or edge.id in used:
                continue
            if path and path[-1].way_id == edge.way_id and path[-1].id[-1] != edge.id[-1]:
                continue
            extended = (*path, edge)
            if index == len(way_ids) - 1:
                yield extended
            else:
                stack.append((edge.to_node, index + 1, extended, used | {edge.id}))
            stack.append((edge.to_node, index, extended, used | {edge.id}))


def _paths(
    row: Restriction, value: str, by_way: dict[int, list[Edge]], outgoing: dict[int, list[Edge]]
) -> list[tuple[str, ...]]:
    sources, via, targets = _members(row)
    if (len(sources) != 1 and value != "no_entry") or (len(targets) != 1 and value != "no_exit"):
        raise RoadNetworkError("unsupported multiple from/to restriction members")
    paths: set[tuple[str, ...]] = set()
    for source in sources:
        for incoming in by_way.get(source, ()):
            if via[0].kind == "n":
                middles: Iterable[tuple[Edge, ...]] = [()] if incoming.to_node == via[0].ref else []
            else:
                middles = _via_paths(
                    incoming.to_node, tuple(member.ref for member in via), outgoing
                )
            for middle in middles:
                end = middle[-1].to_node if middle else incoming.to_node
                for target in outgoing.get(end, ()):
                    if target.way_id not in targets:
                        continue
                    if not middle and incoming.way_id == target.way_id:
                        reversed_geometry = target.shape_e7 == tuple(reversed(incoming.shape_e7))
                        if (value.endswith("u_turn")) != reversed_geometry:
                            continue
                    paths.add((incoming.id, *(edge.id for edge in middle), target.id))
                    if len(paths) > MAX_RESTRICTION_PATHS:
                        raise RoadNetworkError("restriction path count limit")
    return sorted(paths)


def _compile_restrictions(
    restrictions: dict[int, Restriction],
    edges: list[Edge],
    way_ids: set[int],
    diagnostics: list[Diagnostic],
) -> tuple[list[Edge], list[TurnRule]]:
    by_way: dict[int, list[Edge]] = defaultdict(list)
    outgoing: dict[int, list[Edge]] = defaultdict(list)
    for edge in edges:
        by_way[edge.way_id].append(edge)
        outgoing[edge.from_node].append(edge)
    rules: list[TurnRule] = []
    excluded: set[int] = set()
    for ident in sorted(restrictions):
        row = restrictions[ident]
        value, refusal = _restriction_value(row)
        if value is None and refusal is None:
            continue
        try:
            if refusal:
                raise RoadNetworkError(refusal)
            sources, via, targets = _members(row)
            needed_ways = (
                *sources,
                *(member.ref for member in via if member.kind == "w"),
                *targets,
            )
            if any(way_id not in way_ids for way_id in needed_ways):
                raise RoadNetworkError("restriction way member outside complete extract")
            assert value is not None
            paths = _paths(row, value, by_way, outgoing)
            if not paths:
                raise RoadNetworkError("restriction has no matching permitted directed movement")
            next_edges: dict[tuple[str, ...], set[str]] = defaultdict(set)
            for path in paths:
                if value in NO_RESTRICTIONS:
                    next_edges[path[:-1]].add(path[-1])
                else:
                    for length in range(1, len(path)):
                        next_edges[path[:length]].add(path[length])
            for prefix, following in sorted(next_edges.items()):
                rules.append(
                    TurnRule(
                        ident,
                        "no" if value in NO_RESTRICTIONS else "only",
                        prefix,
                        tuple(sorted(following)),
                    )
                )
        except RoadNetworkError as error:
            excluded_sources = {
                member.ref for member in row.members if member.kind == "w" and member.role == "from"
            }
            # Malformed missing-from relations still cannot open their touched roads.
            if not excluded_sources:
                excluded_sources = {member.ref for member in row.members if member.kind == "w"}
            excluded.update(excluded_sources)
            diagnostics.append(
                Diagnostic(
                    "unsupported_restriction",
                    "relation",
                    ident,
                    f"{error}; excluded from ways {','.join(map(str, sorted(excluded_sources)))}",
                )
            )
    del by_way, outgoing
    retained = [edge for edge in edges if edge.way_id not in excluded]
    retained_ids = {edge.id for edge in retained}
    surviving = []
    for rule in rules:
        if not set(rule.prefix_edge_ids) <= retained_ids:
            continue
        retained_following = tuple(ident for ident in rule.next_edge_ids if ident in retained_ids)
        if retained_following or rule.kind == "only":
            surviving.append(
                TurnRule(rule.relation_id, rule.kind, rule.prefix_edge_ids, retained_following)
            )
    return retained, surviving


def build_graph(
    nodes: Iterable[Node],
    ways: Iterable[Way],
    restrictions: Iterable[Restriction],
    *,
    profile: VehicleProfile,
    source: SourceIdentity,
) -> RoadGraph:
    node_rows = _unique(nodes, "node")
    way_rows = _unique(ways, "way")
    restriction_rows = _unique(restrictions, "relation")
    diagnostics: list[Diagnostic] = []
    for node in node_rows.values():
        if (
            type(node.lon_e7) is not int
            or type(node.lat_e7) is not int
            or not (
                -1800000000 <= node.lon_e7 <= 1800000000 and -900000000 <= node.lat_e7 <= 900000000
            )
        ):
            raise RoadNetworkError(f"invalid E7 coordinate for node {node.id}")
    directions = {ident: _directions(way, profile, diagnostics) for ident, way in way_rows.items()}
    occurrences: Counter[int] = Counter()
    split_nodes: set[int] = set()
    for ident, way in way_rows.items():
        if len(way.node_ids) < 2:
            raise RoadNetworkError(f"way {ident} has fewer than two nodes")
        for node_id in way.node_ids:
            if node_id not in node_rows:
                raise RoadNetworkError(f"way {ident} references missing node {node_id}")
        if any(directions[ident]):
            occurrences.update(way.node_ids)
            split_nodes.update((way.node_ids[0], way.node_ids[-1]))
    split_nodes.update(ident for ident, count in occurrences.items() if count > 1)
    split_nodes.update(
        member.ref
        for row in restriction_rows.values()
        for member in row.members
        if member.kind == "n" and member.role == "via"
    )
    blocked_nodes = {ident for ident in occurrences if _node_blocked(node_rows[ident], profile)}
    for ident in sorted(blocked_nodes):
        diagnostics.append(
            Diagnostic(
                "barrier_or_access_excluded",
                "node",
                ident,
                "incident segments excluded by through-HGV profile",
            )
        )
    split_nodes.update(blocked_nodes)
    del occurrences
    edges = _make_edges(way_rows, node_rows, directions, split_nodes, blocked_nodes, diagnostics)
    del directions, split_nodes, blocked_nodes
    way_ids = set(way_rows)
    del way_rows
    # Geometry owns interior coordinates; only endpoint nodes need source metadata.
    endpoints = {ident for edge in edges for ident in (edge.from_node, edge.to_node)}
    node_rows = {ident: node_rows[ident] for ident in endpoints}
    del endpoints
    edges, rules = _compile_restrictions(restriction_rows, edges, way_ids, diagnostics)
    del restriction_rows, way_ids
    endpoints = {ident for edge in edges for ident in (edge.from_node, edge.to_node)}
    return RoadGraph(
        source,
        profile,
        tuple(node_rows[ident] for ident in sorted(endpoints)),
        tuple(sorted(edges, key=lambda edge: edge.id)),
        tuple(
            sorted(
                rules,
                key=lambda rule: (
                    rule.relation_id,
                    rule.kind,
                    rule.prefix_edge_ids,
                    rule.next_edge_ids,
                ),
            )
        ),
        tuple(sorted(diagnostics)),
    )


@dataclass(frozen=True, slots=True, eq=False)
class _Path:
    """Shared predecessors avoid copying a full road route into every queue entry."""

    parent: _Path | None
    edge_id: str | None

    def edge_ids(self) -> tuple[str, ...]:
        output = []
        current: _Path | None = self
        while current is not None and current.edge_id is not None:
            output.append(current.edge_id)
            current = current.parent
        return tuple(reversed(output))

    def __lt__(self, other: _Path) -> bool:
        return self.edge_ids() < other.edge_ids()


class RoutingNodeView(Protocol):
    @property
    def id(self) -> int: ...


class RoutingEdgeView(Protocol):
    @property
    def id(self) -> str: ...

    @property
    def from_node(self) -> int: ...

    @property
    def to_node(self) -> int: ...

    @property
    def distance_mm(self) -> int: ...


class RoutingGraphView(Protocol):
    """The same router can release drawing-only metadata after source validation."""

    @property
    def nodes(self) -> Iterable[RoutingNodeView]: ...

    @property
    def edges(self) -> Iterable[RoutingEdgeView]: ...

    @property
    def turn_rules(self) -> Iterable[TurnRule]: ...


class RoadRouter:
    """Reusable exact-distance router over the compiled artifact's turn authority.

    Equal costs choose the lexical directed edge sequence. State retains the
    longest history suffix that prefixes any restriction, including via-way rules.
    One multi-target search per county constructs the county path matrix.
    """

    def __init__(self, graph: RoutingGraphView) -> None:
        self.node_ids = {node.id for node in graph.nodes}
        self.outgoing: dict[int, list[RoutingEdgeView]] = defaultdict(list)
        for edge in graph.edges:
            if not 0 < edge.distance_mm <= MAX_U64:
                raise RoadNetworkError("routing requires positive u64 edge distances")
            self.outgoing[edge.from_node].append(edge)
        self.prefixes = {
            rule.prefix_edge_ids[:length]
            for rule in graph.turn_rules
            for length in range(1, len(rule.prefix_edge_ids) + 1)
        }
        self.lengths = sorted({len(prefix) for prefix in self.prefixes}, reverse=True)
        self.rules_by_last: dict[str, list[TurnRule]] = defaultdict(list)
        for rule in graph.turn_rules:
            self.rules_by_last[rule.prefix_edge_ids[-1]].append(rule)

    def _advance(self, state: tuple[str, ...], edge_id: str) -> tuple[str, ...] | None:
        applicable = self.rules_by_last.get(state[-1], ()) if state else ()
        if any(
            state[-len(rule.prefix_edge_ids) :] == rule.prefix_edge_ids
            and ((edge_id in rule.next_edge_ids) == (rule.kind == "no"))
            for rule in applicable
        ):
            return None
        extended = (*state, edge_id)
        return next(
            (
                extended[-length:]
                for length in self.lengths
                if length <= len(extended) and extended[-length:] in self.prefixes
            ),
            (),
        )

    def shortest_path(self, start: int, destination: int) -> Route | None:
        return self.shortest_paths(start, (destination,))[destination]

    def shortest_paths(self, start: int, destinations: Iterable[int]) -> dict[int, Route | None]:
        result: dict[int, Route | None] = dict.fromkeys(sorted(set(destinations)))
        remaining = set(result) & self.node_ids
        if start not in self.node_ids or not remaining:
            return result
        empty_path = _Path(None, None)
        queue: list[tuple[int, _Path, int, tuple[str, ...]]] = [(0, empty_path, start, ())]
        best: dict[tuple[int, tuple[str, ...]], tuple[int, _Path]] = {(start, ()): (0, empty_path)}
        while queue:
            distance, path, node, state = heapq.heappop(queue)
            if best.get((node, state)) != (distance, path):
                continue
            if node in remaining:
                result[node] = Route(distance, path.edge_ids())
                remaining.remove(node)
                if not remaining:
                    break
            for edge in self.outgoing.get(node, ()):
                next_state = self._advance(state, edge.id)
                if next_state is None:
                    continue
                new_distance = distance + edge.distance_mm
                if new_distance > MAX_U64:
                    raise RoadNetworkError("route distance overflows u64")
                key = (edge.to_node, next_state)
                previous = best.get(key)
                if previous is not None and new_distance > previous[0]:
                    continue
                candidate = (new_distance, _Path(path, edge.id))
                if previous is None or candidate < previous:
                    best[key] = candidate
                    heapq.heappush(queue, (*candidate, edge.to_node, next_state))
        return result


def _json_bytes(value: object) -> bytes:
    return json.dumps(
        value, ensure_ascii=True, allow_nan=False, sort_keys=True, separators=(",", ":")
    ).encode("ascii")


def _rows(rows: Iterable[object]) -> Iterator[bytes]:
    yield b"["
    first = True
    for row in rows:
        if not first:
            yield b","
        first = False
        value = asdict(row)  # type: ignore[call-overload]
        if "tags" in value:
            value["tags"] = dict(value["tags"])
        yield _json_bytes(value)
    yield b"]"


def canonical_chunks(graph: RoadGraph) -> Iterator[bytes]:
    """Canonical sorted-key JSON, streamed without a second full graph allocation."""
    yield b'{"diagnostics":'
    yield from _rows(graph.diagnostics)
    yield b',"edges":'
    yield from _rows(graph.edges)
    yield b',"license":"ODbL-1.0; OpenStreetMap contributors","nodes":'
    yield from _rows(graph.nodes)
    yield b',"profile":' + _json_bytes(asdict(graph.profile))
    yield b',"routing_profile_version":' + _json_bytes(ROUTING_PROFILE_VERSION)
    yield b',"schema_version":1,"source":' + _json_bytes(asdict(graph.source))
    yield b',"turn_rules":'
    yield from _rows(graph.turn_rules)
    yield b"}\n"


def canonical_bytes(graph: RoadGraph) -> bytes:
    return b"".join(canonical_chunks(graph))


def write_graph(graph: RoadGraph, destination: Path) -> str:
    """Publish complete canonical bytes atomically; an error preserves the old file."""
    digest = hashlib.sha256()
    temporary: str | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=destination.parent, prefix=f".{destination.name}.", delete=False
        ) as output:
            temporary = output.name
            for chunk in canonical_chunks(graph):
                output.write(chunk)
                digest.update(chunk)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, destination)
        temporary = None
    finally:
        if temporary is not None:
            Path(temporary).unlink(missing_ok=True)
    return digest.hexdigest()
