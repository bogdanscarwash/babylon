"""Qualify Designed county terminals and paths against one pinned physical graph.

The existing atlas supplies Michigan's interior-land display anchors. Reusing one
as a county aggregate terminal is a Designed choice, not a factory coordinate.
Attachment distances are reported separately and never become invented roads.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import math
import os
import re
import resource
import struct
import sys
import tempfile
import tomllib
from collections import defaultdict
from collections.abc import Callable, Iterator, Mapping, Sequence
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, TextIO

import pyproj
from pyproj import Transformer
from tools.michigan_road_network_v1 import (
    GEOD,
    MAX_U64,
    ROUTING_PROFILE_VERSION,
    Node,
    RoadGraph,
    RoadNetworkError,
    RoadRouter,
    Route,
    SourceIdentity,
    TurnRule,
    VehicleProfile,
)

SCHEMA = "MichiganCountyPathMatrixV1"
ROOT = Path(__file__).resolve().parents[1]
BUCKET_E7 = 1_000_000
ATLAS_HEADER = struct.Struct("<8sII32sdddIIII32s8x")
COUNTY_RECORD = struct.Struct("<5sxIHH4H2H2x")


class TerminalQualificationError(ValueError):
    """Source identity, attachment, or resource bounds prevent qualification."""


@dataclass(frozen=True, slots=True)
class CountyAnchor:
    county_geoid: str
    county_name: str
    lon_e7: int
    lat_e7: int
    atlas_grid_x: int
    atlas_grid_y: int


@dataclass(frozen=True, slots=True)
class CountyTerminal:
    county_geoid: str
    county_name: str
    anchor_lon_e7: int
    anchor_lat_e7: int
    atlas_grid_x: int
    atlas_grid_y: int
    node_id: int | None
    node_lon_e7: int | None
    node_lat_e7: int | None
    attachment_distance_mm: int | None
    status: str
    evidence_class: str = "Designed"


@dataclass(frozen=True, slots=True)
class RoutingNode:
    id: int
    lon_e7: int
    lat_e7: int


@dataclass(frozen=True, slots=True)
class RoutingArc:
    id: str
    from_node: int
    to_node: int
    distance_mm: int


@dataclass(frozen=True, slots=True)
class RoutingView:
    nodes: tuple[RoutingNode, ...]
    edges: tuple[RoutingArc, ...]
    turn_rules: tuple[TurnRule, ...]
    source: SourceIdentity
    profile: VehicleProfile


def _hash(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def _uint(value: object, label: str, *, positive: bool = True) -> int:
    if type(value) is not int or not (int(positive) <= value <= MAX_U64):
        raise TerminalQualificationError(
            f"{label} must be a {'positive' if positive else 'nonnegative'} u64"
        )
    return value


def _coordinate(value: object, limit: int) -> int:
    if type(value) is not int or not -limit <= value <= limit:
        raise TerminalQualificationError("invalid integer E7 coordinate")
    return value


def _e7(value: float) -> int:
    if not math.isfinite(value):
        raise TerminalQualificationError("non-finite atlas inverse projection")
    return int(math.copysign(math.floor(abs(value) * 1e7 + 0.5), value))


def read_atlas_anchors(
    atlas: Path, pin_file: Path
) -> tuple[tuple[CountyAnchor, ...], dict[str, str]]:
    raw = atlas.read_bytes()
    pin_bytes = pin_file.read_bytes()
    pin = json.loads(pin_bytes)
    atlas_sha = hashlib.sha256(raw).hexdigest()
    if atlas_sha != pin.get("atlas_sha256"):
        raise TerminalQualificationError("atlas SHA-256 differs from the checked land evidence")
    if len(raw) < ATLAS_HEADER.size:
        raise TerminalQualificationError("truncated county atlas header")
    magic, version, flags, digest, ox, oy, scale, counties, rings, vertices, nnz, _ = (
        ATLAS_HEADER.unpack_from(raw)
    )
    if (
        magic != b"BABCTY\0\x01"
        or version != 1
        or flags != 0
        or hashlib.sha256(raw[48:]).digest() != digest
    ):
        raise TerminalQualificationError("county atlas header or content hash mismatch")
    if not all(math.isfinite(value) for value in (ox, oy, scale)) or scale <= 0:
        raise TerminalQualificationError("invalid county atlas grid transform")
    name_offset = (
        128
        + counties * COUNTY_RECORD.size
        + rings * 12
        + vertices * 4
        + (counties + 1) * 4
        + nnz * 4
    )
    if name_offset + 4 > len(raw):
        raise TerminalQualificationError("county atlas counts exceed its bytes")
    (name_length,) = struct.unpack_from("<I", raw, name_offset)
    if name_offset + 4 + name_length != len(raw):
        raise TerminalQualificationError("county atlas name section length mismatch")
    names = raw[name_offset + 4 :].decode("utf-8").splitlines()
    if len(names) != counties:
        raise TerminalQualificationError("county atlas names do not match county rows")
    inverse = Transformer.from_crs(5070, 4326, always_xy=True)
    anchors = []
    previous = ""
    for index in range(counties):
        row = COUNTY_RECORD.unpack_from(raw, 128 + index * COUNTY_RECORD.size)
        fips = row[0].decode("ascii")
        if not re.fullmatch(r"\d{5}", fips) or fips <= previous:
            raise TerminalQualificationError("county atlas identities are not sorted and unique")
        previous = fips
        if not fips.startswith("26"):
            continue
        gx, gy = row[-2:]
        if not (row[4] <= gx <= row[6] and row[5] <= gy <= row[7]):
            raise TerminalQualificationError(f"atlas anchor lies outside county {fips} bounds")
        lon, lat = inverse.transform(ox + gx * scale, oy + gy * scale)
        anchors.append(CountyAnchor(fips, names[index], _e7(lon), _e7(lat), gx, gy))
    if len(anchors) != 83:
        raise TerminalQualificationError(f"expected 83 Michigan atlas anchors, got {len(anchors)}")
    return tuple(anchors), {
        "atlas_sha256": atlas_sha,
        "atlas_pin_sha256": hashlib.sha256(pin_bytes).hexdigest(),
    }


class _JsonStream:
    """Read one canonical graph row at a time with the standard JSON decoder."""

    def __init__(self, source: TextIO) -> None:
        self.source = source
        self.buffer = ""
        self.decoder = json.JSONDecoder()
        self.eof = False

    def _read(self) -> None:
        part = self.source.read(65536)
        self.eof = not part
        self.buffer += part
        if len(self.buffer) > 4 * 1024 * 1024:
            raise TerminalQualificationError("graph JSON row exceeds 4 MiB")

    def peek(self) -> str:
        self.buffer = self.buffer.lstrip()
        while not self.buffer and not self.eof:
            self._read()
            self.buffer = self.buffer.lstrip()
        return self.buffer[:1]

    def take(self, expected: str) -> None:
        if self.peek() != expected:
            raise TerminalQualificationError(f"graph JSON expected {expected!r}")
        self.buffer = self.buffer[1:]

    def value(self) -> Any:
        self.peek()
        while True:
            try:
                value, end = self.decoder.raw_decode(self.buffer)
                # A scalar number at the buffer boundary might be incomplete.
                if end == len(self.buffer) and not self.eof:
                    self._read()
                    continue
                self.buffer = self.buffer[end:]
                return value
            except json.JSONDecodeError as error:
                if self.eof:
                    raise TerminalQualificationError(f"invalid graph JSON: {error.msg}") from error
                self._read()


def _graph_records(path: Path) -> Iterator[tuple[str, Any]]:
    with path.open("r", encoding="ascii") as source:
        reader = _JsonStream(source)
        reader.take("{")
        seen = set()
        while reader.peek() != "}":
            key = reader.value()
            if not isinstance(key, str) or key in seen:
                raise TerminalQualificationError("duplicate or invalid graph section")
            seen.add(key)
            reader.take(":")
            if key in {"nodes", "edges", "turn_rules", "diagnostics"}:
                reader.take("[")
                yield key, []
                while reader.peek() != "]":
                    yield f"{key}[]", reader.value()
                    if reader.peek() != "]":
                        reader.take(",")
                reader.take("]")
            else:
                yield key, reader.value()
            if reader.peek() != "}":
                reader.take(",")
        reader.take("}")
        if reader.peek():
            raise TerminalQualificationError("trailing graph JSON bytes")


def _resident_bytes() -> int:
    # Qualification runs on the Linux reference-data host; do not invent a value.
    return int(Path("/proc/self/statm").read_text().split()[1]) * os.sysconf("SC_PAGE_SIZE")


class _MemoryGuard:
    def __init__(self, budget: int) -> None:
        self.budget = _uint(budget, "memory budget")
        self.start_rss = _resident_bytes()
        self.start_peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024

    def check(self) -> None:
        growth = max(
            _resident_bytes() - self.start_rss,
            resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024 - self.start_peak,
        )
        if growth > self.budget:
            raise TerminalQualificationError(
                f"routing view exceeded {self.budget} byte memory-growth budget"
            )


def _fields(row: Any, expected: set[str], label: str) -> dict[str, Any]:
    if not isinstance(row, dict) or set(row) != expected:
        raise TerminalQualificationError(f"unexpected {label} fields")
    return row


def _validate_turns(rules: Sequence[TurnRule], edges: Sequence[RoutingArc]) -> None:
    needed = {ident for rule in rules for ident in (*rule.prefix_edge_ids, *rule.next_edge_ids)}
    lookup = {edge.id: edge for edge in edges if edge.id in needed}
    if set(lookup) != needed:
        raise TerminalQualificationError("turn rule references an absent directed edge")
    for rule in rules:
        prefix = [lookup[ident] for ident in rule.prefix_edge_ids]
        if any(
            left.to_node != right.from_node for left, right in zip(prefix, prefix[1:], strict=False)
        ):
            raise TerminalQualificationError("turn prefix is not a contiguous directed path")
        if any(lookup[ident].from_node != prefix[-1].to_node for ident in rule.next_edge_ids):
            raise TerminalQualificationError("turn continuation is not incident to the prefix")


def load_routing_view(
    path: Path,
    expected_sha256: str,
    *,
    memory_budget_bytes: int,
    progress: Callable[[dict[str, Any]], None] | None = None,
) -> RoutingView:
    if not re.fullmatch(r"[0-9a-f]{64}", expected_sha256) or _hash(path) != expected_sha256:
        raise TerminalQualificationError("road graph SHA-256 differs from the expected source")
    guard = _MemoryGuard(memory_budget_bytes)
    nodes: list[RoutingNode] = []
    edges: list[RoutingArc] = []
    rules: list[TurnRule] = []
    metadata = {}
    previous_node, previous_edge = 0, ""
    count = 0
    for section, value in _graph_records(path):
        count += 1
        if count % 8192 == 0:
            guard.check()
        if section == "nodes[]":
            row = _fields(value, {"id", "lon_e7", "lat_e7", "tags", "version", "timestamp"}, "node")
            ident = _uint(row["id"], "node ID")
            if ident <= previous_node:
                raise TerminalQualificationError("graph nodes are not sorted and unique")
            previous_node = ident
            nodes.append(
                RoutingNode(
                    ident,
                    _coordinate(row["lon_e7"], 1800000000),
                    _coordinate(row["lat_e7"], 900000000),
                )
            )
        elif section == "edges[]":
            row = _fields(
                value,
                {
                    "id",
                    "way_id",
                    "from_node",
                    "to_node",
                    "distance_mm",
                    "shape_e7",
                    "tags",
                    "way_version",
                    "way_timestamp",
                },
                "edge",
            )
            ident = row["id"]
            if (
                not isinstance(ident, str)
                or not re.fullmatch(r"w[1-9]\d*:\d+:\d+:[fr]", ident)
                or ident <= previous_edge
            ):
                raise TerminalQualificationError("graph edges are not valid, sorted and unique")
            previous_edge = ident
            edges.append(
                RoutingArc(
                    ident,
                    _uint(row["from_node"], "from node"),
                    _uint(row["to_node"], "to node"),
                    _uint(row["distance_mm"], "edge distance"),
                )
            )
        elif section == "turn_rules[]":
            row = _fields(
                value, {"relation_id", "kind", "prefix_edge_ids", "next_edge_ids"}, "turn rule"
            )
            if (
                row["kind"] not in {"no", "only"}
                or not isinstance(row["prefix_edge_ids"], list)
                or not row["prefix_edge_ids"]
                or not isinstance(row["next_edge_ids"], list)
                or any(
                    not isinstance(ident, str)
                    for ident in (*row["prefix_edge_ids"], *row["next_edge_ids"])
                )
            ):
                raise TerminalQualificationError("invalid directed turn rule")
            rules.append(
                TurnRule(
                    _uint(row["relation_id"], "restriction ID"),
                    row["kind"],
                    tuple(row["prefix_edge_ids"]),
                    tuple(row["next_edge_ids"]),
                )
            )
        elif section == "diagnostics[]":
            _fields(value, {"code", "osm_type", "osm_id", "detail"}, "diagnostic")
        else:
            metadata[section] = value
    expected_sections = {
        "schema_version",
        "routing_profile_version",
        "source",
        "profile",
        "license",
        "nodes",
        "edges",
        "turn_rules",
        "diagnostics",
    }
    if (
        set(metadata) != expected_sections
        or type(metadata["schema_version"]) is not int
        or metadata["schema_version"] != 1
    ):
        raise TerminalQualificationError("unsupported road graph schema")
    if metadata["routing_profile_version"] != ROUTING_PROFILE_VERSION:
        raise TerminalQualificationError("unsupported road routing profile version")
    ids = {node.id for node in nodes}
    if any(edge.from_node not in ids or edge.to_node not in ids for edge in edges):
        raise TerminalQualificationError("directed edge references an absent node")
    _validate_turns(rules, edges)
    view = RoutingView(
        tuple(nodes),
        tuple(edges),
        tuple(rules),
        SourceIdentity(**metadata["source"]),
        VehicleProfile(**metadata["profile"]),
    )
    guard.check()
    if progress is not None:
        progress(
            {
                "stage": "routing_view_loaded",
                "graph_bytes": path.stat().st_size,
                "nodes": len(nodes),
                "edges": len(edges),
                "turn_rules": len(rules),
                "rss_bytes": _resident_bytes(),
            }
        )
    return view


def attach_terminals(
    anchors: Sequence[CountyAnchor], graph: RoadGraph | RoutingView, *, attachment_limit_mm: int
) -> tuple[CountyTerminal, ...]:
    _uint(attachment_limit_mm, "attachment limit")
    if len({row.county_geoid for row in anchors}) != len(anchors):
        raise TerminalQualificationError("duplicate county anchor identity")
    incoming = {edge.to_node for edge in graph.edges}
    usable = incoming & {edge.from_node for edge in graph.edges}
    buckets: dict[tuple[int, int], list[Node | RoutingNode]] = defaultdict(list)
    for node in graph.nodes:
        if node.id in usable:
            buckets[(node.lon_e7 // BUCKET_E7, node.lat_e7 // BUCKET_E7)].append(node)
    output = []
    for anchor in sorted(anchors, key=lambda row: row.county_geoid):
        if not re.fullmatch(r"26\d{3}", anchor.county_geoid):
            raise TerminalQualificationError("terminal county must have a Michigan GEOID")
        _coordinate(anchor.lon_e7, 1800000000)
        _coordinate(anchor.lat_e7, 900000000)
        lat, lon = anchor.lat_e7 / 1e7, anchor.lon_e7 / 1e7
        # This deliberately broad index window only rejects faraway candidates;
        # WGS84 geodesic distance below makes the actual admission and ordering.
        dlat = attachment_limit_mm / 1000 / 110000
        dlon = dlat / max(0.001, math.cos(math.radians(abs(lat) + dlat)))
        best: tuple[int, int, Node | RoutingNode] | None = None
        for x in range(
            math.floor((lon - dlon) * 1e7 / BUCKET_E7),
            math.floor((lon + dlon) * 1e7 / BUCKET_E7) + 1,
        ):
            for y in range(
                math.floor((lat - dlat) * 1e7 / BUCKET_E7),
                math.floor((lat + dlat) * 1e7 / BUCKET_E7) + 1,
            ):
                for node in buckets.get((x, y), ()):
                    _, _, metres = GEOD.inv(lon, lat, node.lon_e7 / 1e7, node.lat_e7 / 1e7)
                    if not math.isfinite(metres) or metres < 0:
                        raise TerminalQualificationError("invalid WGS84 attachment distance")
                    distance = max(1, math.floor(metres * 1000 + 0.5))
                    if distance <= attachment_limit_mm and (
                        best is None or (distance, node.id) < best[:2]
                    ):
                        best = (distance, node.id, node)
        selected = best[2] if best is not None else None
        output.append(
            CountyTerminal(
                anchor.county_geoid,
                anchor.county_name,
                anchor.lon_e7,
                anchor.lat_e7,
                anchor.atlas_grid_x,
                anchor.atlas_grid_y,
                selected.id if selected is not None else None,
                selected.lon_e7 if selected is not None else None,
                selected.lat_e7 if selected is not None else None,
                best[0] if best is not None else None,
                "attached" if best is not None else "no_usable_node_within_limit",
            )
        )
    return tuple(output)


def qualify_paths(
    graph: RoadGraph | RoutingView,
    terminals: Sequence[CountyTerminal],
    *,
    progress: Callable[[dict[str, Any]], None] | None = None,
) -> dict[tuple[str, str], Route | None]:
    if len({row.county_geoid for row in terminals}) != len(terminals):
        raise TerminalQualificationError("duplicate terminal identity")
    ordered = sorted(terminals, key=lambda row: row.county_geoid)
    router = RoadRouter(graph)
    targets = {row.node_id for row in ordered if row.node_id is not None}
    output = {}
    for origin in ordered:
        found = router.shortest_paths(origin.node_id, targets) if origin.node_id is not None else {}
        for destination in ordered:
            output[(origin.county_geoid, destination.county_geoid)] = (
                found.get(destination.node_id) if destination.node_id is not None else None
            )
        if progress is not None:
            progress(
                {
                    "stage": "county_paths",
                    "origin_county_geoid": origin.county_geoid,
                    "reachable_destinations": sum(value is not None for value in found.values()),
                    "rss_bytes": _resident_bytes(),
                }
            )
    return output


def matrix_document(
    terminals: Sequence[CountyTerminal],
    paths: Mapping[tuple[str, str], Route | None],
    source_pins: Mapping[str, str],
    *,
    attachment_limit_mm: int,
) -> dict[str, Any]:
    ordered = sorted(terminals, key=lambda row: row.county_geoid)
    expected = {(left.county_geoid, right.county_geoid) for left in ordered for right in ordered}
    if set(paths) != expected:
        raise TerminalQualificationError(
            "county path matrix does not cover the exact terminal product"
        )
    diagnostics = [
        {"code": row.status, "county_geoid": row.county_geoid}
        for row in ordered
        if row.status != "attached"
    ]
    diagnostics.extend(
        {
            "code": "no_directed_road_path",
            "source_county_geoid": source,
            "destination_county_geoid": destination,
        }
        for (source, destination), value in sorted(paths.items())
        if value is None
    )
    return {
        "schema": SCHEMA,
        "source_pins": dict(source_pins),
        "policy": {
            "terminal_evidence_class": "Designed",
            "anchor": "existing Michigan interior-land county atlas anchor",
            "attachment_limit_mm": attachment_limit_mm,
            "attachment_distance": "WGS84 geodesic, half-up millimetres, minimum one millimetre",
            "usable_node": "has an incoming and an outgoing physical road edge",
            "physical_path_distance": "sum of directed road edge distances; excludes Designed attachment distance",
            "projection": f"EPSG5070-to-EPSG4326;pyproj={pyproj.__version__};PROJ={pyproj.proj_version_str}",
        },
        "terminals": [asdict(row) for row in ordered],
        "paths": [
            {
                "source_county_geoid": source,
                "destination_county_geoid": destination,
                "path": asdict(value) if value is not None else None,
            }
            for (source, destination), value in sorted(paths.items())
        ],
        "diagnostics": diagnostics,
    }


def write_matrix(document: Mapping[str, Any], destination: Path) -> str:
    payload = (
        json.dumps(
            document, sort_keys=True, ensure_ascii=True, allow_nan=False, separators=(",", ":")
        ).encode("ascii")
        + b"\n"
    )
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=destination.parent, prefix=f".{destination.name}.", delete=False
        ) as output:
            temporary = output.name
            with gzip.GzipFile(
                fileobj=output, mode="wb", filename="", mtime=0, compresslevel=9
            ) as compressed:
                compressed.write(payload)
            output.flush()
            os.fsync(output.fileno())
        digest = _hash(Path(temporary))
        os.replace(temporary, destination)
        temporary = None
        return digest
    finally:
        if temporary is not None:
            Path(temporary).unlink(missing_ok=True)


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--graph", type=Path, required=True)
    parser.add_argument("--graph-sha256", required=True)
    parser.add_argument("--atlas", type=Path, default=ROOT / "assets/map/county_atlas.bin")
    parser.add_argument(
        "--atlas-pin",
        type=Path,
        default=ROOT / "rust/crates/babylon-client/tests/fixtures/michigan_atlas_land_probes.json",
    )
    parser.add_argument(
        "--defines", type=Path, default=ROOT / "content/scenarios/michigan/defines.toml"
    )
    parser.add_argument("--memory-budget-mib", type=int, default=4096)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.output.resolve() in {
            path.resolve() for path in (args.graph, args.atlas, args.atlas_pin, args.defines)
        }:
            raise TerminalQualificationError("output overlaps a pinned input")
        budget = _uint(args.memory_budget_mib, "memory budget MiB") * 1024 * 1024
        guard = _MemoryGuard(budget)

        def progress(report: dict[str, Any]) -> None:
            guard.check()
            print(json.dumps(report, sort_keys=True), flush=True)

        defines_bytes = args.defines.read_bytes()
        defines = tomllib.loads(defines_bytes.decode("utf-8"))
        if defines["statewide"]["EVIDENCE_CLASS"] != "Designed":
            raise TerminalQualificationError("county terminal policy must be explicitly Designed")
        limit = (
            _uint(
                defines["statewide"]["TERMINAL_ATTACHMENT_LIMIT_METERS"], "attachment limit metres"
            )
            * 1000
        )
        anchors, pins = read_atlas_anchors(args.atlas, args.atlas_pin)
        pins.update(
            {
                "graph_sha256": args.graph_sha256,
                "defines_sha256": hashlib.sha256(defines_bytes).hexdigest(),
            }
        )
        view = load_routing_view(
            args.graph, args.graph_sha256, memory_budget_bytes=budget, progress=progress
        )
        transport = defines["transport"]
        if view.profile != VehicleProfile(
            transport["TRUCK_GROSS_WEIGHT_KG"],
            transport["TRUCK_HEIGHT_MM"],
            transport["TRUCK_WIDTH_MM"],
            transport["TRUCK_LENGTH_MM"],
            transport["DEFAULT_MAXHEIGHT_MM"],
        ):
            raise TerminalQualificationError(
                "graph vehicle profile differs from the authored transport profile"
            )
        terminals = attach_terminals(anchors, view, attachment_limit_mm=limit)
        progress(
            {
                "stage": "matrix_preflight",
                "counties": len(terminals),
                "attached_terminals": sum(row.node_id is not None for row in terminals),
                "graph_nodes": len(view.nodes),
                "graph_edges": len(view.edges),
                "turn_rules": len(view.turn_rules),
                "rss_bytes": _resident_bytes(),
                "memory_growth_budget_bytes": budget,
            }
        )
        paths = qualify_paths(view, terminals, progress=progress)
        document = matrix_document(terminals, paths, pins, attachment_limit_mm=limit)
        digest = write_matrix(document, args.output)
        print(
            json.dumps(
                {
                    "stage": "published",
                    "sha256": digest,
                    "terminals": len(terminals),
                    "paths": len(paths),
                    "diagnostics": len(document["diagnostics"]),
                },
                sort_keys=True,
            )
        )
        return 0
    except (
        TerminalQualificationError,
        RoadNetworkError,
        OSError,
        ValueError,
        TypeError,
        KeyError,
    ) as error:
        print(f"county road qualification refused: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
