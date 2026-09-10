"""Capture qualified Michigan commodity and selected physical facts beside defines.

The supplied qualification chooses relationships; the supplied capacity groups
choose shared constraints. This packager checks their source identity and path
continuity without choosing routes, capacity groups, or experiment targets.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import json
import os
import re
import sys
import tempfile
import tomllib
from collections.abc import Sequence
from dataclasses import asdict
from pathlib import Path
from typing import Any

from tools.michigan_road_network_v1 import (
    MAX_U64,
    ROUTING_PROFILE_VERSION,
    RoadNetworkError,
    SourceIdentity,
    VehicleProfile,
    _length_mm,
)
from tools.qualify_michigan_road_terminals import TerminalQualificationError, _graph_records, _hash

MAX_CONTENT_BYTES = 64 * 1024 * 1024  # Rust MAX_MICHIGAN_CAPTURED_CONTENT_BYTES_V2.
MAX_MATRIX_BYTES = 512 * 1024 * 1024
OUTPUT_NAMES = (
    "statewide-qualification.json.gz",
    "statewide-physical.json.gz",
    "statewide-sources.json",
)
QUALIFICATION_FIELDS = {
    "schema",
    "evidence_class",
    "qualified",
    "owners",
    "processes",
    "orders",
    "retail_final_demands",
    "diagnostics",
    "defines_sha256",
    "roster_sha256",
    "paths_sha256",
}
TERMINAL_FIELDS = {
    "county_geoid",
    "county_name",
    "anchor_lon_e7",
    "anchor_lat_e7",
    "atlas_grid_x",
    "atlas_grid_y",
    "node_id",
    "node_lon_e7",
    "node_lat_e7",
    "attachment_distance_mm",
    "status",
    "evidence_class",
}
EDGE_FIELDS = {
    "id",
    "way_id",
    "from_node",
    "to_node",
    "distance_mm",
    "shape_e7",
    "tags",
    "way_version",
    "way_timestamp",
}
PIN_FIELDS = {"atlas_sha256", "atlas_pin_sha256", "graph_sha256", "defines_sha256"}
POLICY_FIELDS = {
    "terminal_evidence_class",
    "anchor",
    "attachment_limit_mm",
    "attachment_distance",
    "usable_node",
    "physical_path_distance",
    "projection",
}


class CaptureError(ValueError):
    """A source mismatch or invalid selected path prevents source publication."""


def _canonical(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"), allow_nan=False)
        + "\n"
    ).encode("ascii")


def _object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CaptureError(f"duplicate JSON field: {key}")
        result[key] = value
    return result


def _invalid_constant(value: str) -> None:
    raise CaptureError(f"nonfinite JSON number: {value}")


def _json(
    path: Path,
    *,
    compressed: bool = False,
    bound: int = MAX_CONTENT_BYTES,
    expected_sha256: str | None = None,
) -> Any:
    with path.open("rb") as source:
        raw = source.read(bound + 1)
    if len(raw) > bound:
        raise CaptureError(f"source exceeds {bound} bytes: {path}")
    if expected_sha256 is not None and hashlib.sha256(raw).hexdigest() != expected_sha256:
        raise CaptureError(f"source SHA-256 differs from qualification: {path}")
    payload = raw
    if compressed:
        with gzip.GzipFile(fileobj=io.BytesIO(raw), mode="rb") as source:
            payload = source.read(bound + 1)
        if len(payload) > bound:
            raise CaptureError(f"decompressed source exceeds {bound} bytes: {path}")
    return json.loads(payload, object_pairs_hook=_object, parse_constant=_invalid_constant)


def _fields(value: Any, expected: set[str], label: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != expected:
        raise CaptureError(f"invalid {label} fields")
    return value


def _uint(value: Any, label: str, *, positive: bool = True, maximum: int = MAX_U64) -> int:
    if type(value) is not int or not (1 if positive else 0) <= value <= maximum:
        raise CaptureError(f"invalid unsigned {label}")
    return value


def _text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise CaptureError(f"invalid {label}")
    return value


def _coordinate(value: Any, bound: int) -> int:
    if type(value) is not int or not -bound <= value <= bound:
        raise CaptureError("invalid physical WGS84 E7 coordinate")
    return value


def _county(value: Any) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"26[0-9]{3}", value):
        raise CaptureError("invalid Michigan county identity")
    return value


def _validate_matrix(
    matrix: Any, owners: list[Any], attachment_limit: int
) -> tuple[dict[str, Any], dict[tuple[str, str], Any]]:
    _fields(
        matrix,
        {"schema", "source_pins", "policy", "terminals", "paths", "diagnostics"},
        "county matrix",
    )
    if matrix["schema"] != "MichiganCountyPathMatrixV1" or not isinstance(
        matrix["diagnostics"], list
    ):
        raise CaptureError("invalid county matrix schema")
    pins = _fields(matrix["source_pins"], PIN_FIELDS, "terminal source pins")
    if any(
        not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value)
        for value in pins.values()
    ):
        raise CaptureError("invalid terminal source SHA-256")
    policy = _fields(matrix["policy"], POLICY_FIELDS, "terminal policy")
    if (
        policy["terminal_evidence_class"] != "Designed"
        or policy["attachment_limit_mm"] != attachment_limit * 1000
    ):
        raise CaptureError("terminal policy differs from authored attachment limit")
    for key in POLICY_FIELDS - {"attachment_limit_mm"}:
        _text(policy[key], f"terminal policy {key}")
    terminals = {}
    for row in matrix["terminals"]:
        _fields(row, TERMINAL_FIELDS, "terminal")
        county = _county(row["county_geoid"])
        if (
            county in terminals
            or row["status"] != "attached"
            or row["evidence_class"] != "Designed"
        ):
            raise CaptureError("duplicate or disconnected county terminal")
        _text(row["county_name"], "county name")
        _uint(row["node_id"], "terminal node ID", maximum=2**63 - 1)
        for prefix in ("anchor", "node"):
            _coordinate(row[f"{prefix}_lon_e7"], 1_800_000_000)
            _coordinate(row[f"{prefix}_lat_e7"], 900_000_000)
        for axis in ("x", "y"):
            _uint(
                row[f"atlas_grid_{axis}"],
                "atlas grid coordinate",
                positive=False,
                maximum=2**63 - 1,
            )
        if _uint(row["attachment_distance_mm"], "attachment distance") > attachment_limit * 1000:
            raise CaptureError("county terminal exceeds authored attachment limit")
        terminals[county] = row
    if (
        not terminals
        or len(terminals) > 83
        or set(terminals) != {_county(owner["county_geoid"]) for owner in owners}
    ):
        raise CaptureError("terminals must cover exactly the qualified owner counties")
    paths = {}
    for row in matrix["paths"]:
        _fields(row, {"source_county_geoid", "destination_county_geoid", "path"}, "county path")
        identity = (_county(row["source_county_geoid"]), _county(row["destination_county_geoid"]))
        if identity in paths:
            raise CaptureError("duplicate directed county path")
        path = row["path"]
        if path is not None:
            _fields(path, {"distance_mm", "edge_ids"}, "physical county path")
            _uint(path["distance_mm"], "county path distance", positive=False)
            if not isinstance(path["edge_ids"], list) or any(
                not isinstance(key, str) or not key for key in path["edge_ids"]
            ):
                raise CaptureError("invalid ordered county path edges")
        paths[identity] = path
    if set(paths) != {(source, destination) for source in terminals for destination in terminals}:
        raise CaptureError("county matrix must cover the exact directed terminal product")
    return terminals, paths


def _selected_orders(qualification: Any, paths: dict[tuple[str, str], Any]) -> set[str]:
    selected = set()
    seen = set()
    owners = {(row["county_geoid"], row["sector_code"]) for row in qualification["owners"]}
    if len(owners) != len(qualification["owners"]):
        raise CaptureError("duplicate qualified owner")
    for order in qualification["orders"]:
        _uint(order["distance_mm"], "qualified order distance", positive=False)
        if not isinstance(order["edge_ids"], list) or any(
            not isinstance(key, str) or not key for key in order["edge_ids"]
        ):
            raise CaptureError("invalid qualified ordered physical edge identities")
        source = (order["supplier_county_geoid"], order["supplier_sector_code"])
        destination = (order["buyer_county_geoid"], order["buyer_sector_code"])
        identity = (*source, *destination, order["good"], order["unit"])
        if source not in owners or destination not in owners or identity in seen:
            raise CaptureError("invalid or duplicate qualified order identity")
        seen.add(identity)
        _uint(order["units"], "finite order units")
        if type(order["local"]) is not bool or order["local"] != (source[0] == destination[0]):
            raise CaptureError("local order disagrees with county identity")
        expected = {"distance_mm": order["distance_mm"], "edge_ids": order["edge_ids"]}
        if order["local"]:
            if expected != {"distance_mm": 0, "edge_ids": []}:
                raise CaptureError("local transfer cannot have a physical road path")
        else:
            if not order["edge_ids"] or paths[(source[0], destination[0])] != expected:
                raise CaptureError("qualified order differs from its connected pinned county path")
            _uint(order["distance_mm"], "physical order distance")
            selected.update(order["edge_ids"])
    return selected


def _validate_edge(row: Any) -> None:
    _fields(row, EDGE_FIELDS, "physical edge")
    match = re.fullmatch(
        r"w([1-9][0-9]*):(0|[1-9][0-9]*):(0|[1-9][0-9]*):[fr]", _text(row["id"], "edge identity")
    )
    if match is None or int(match[1]) != _uint(row["way_id"], "way ID", maximum=2**63 - 1):
        raise CaptureError("physical edge identity disagrees with original way")
    for key in ("from_node", "to_node"):
        _uint(row[key], "edge endpoint", maximum=2**63 - 1)
    _uint(row["way_version"], "way version")
    if (
        not isinstance(row["way_timestamp"], str)
        or not isinstance(row["tags"], dict)
        or any(not isinstance(k, str) or not isinstance(v, str) for k, v in row["tags"].items())
    ):
        raise CaptureError("invalid retained physical way facts")
    shape = row["shape_e7"]
    if (
        not isinstance(shape, list)
        or len(shape) < 2
        or len(shape) != int(match[3]) - int(match[2]) + 1
    ):
        raise CaptureError("physical edge shape disagrees with original way positions")
    for point in shape:
        if not isinstance(point, list) or len(point) != 2:
            raise CaptureError("invalid physical edge shape point")
        _coordinate(point[0], 1_800_000_000)
        _coordinate(point[1], 900_000_000)
    if _uint(row["distance_mm"], "edge distance") != _length_mm(
        tuple(tuple(point) for point in shape)
    ):
        raise CaptureError("physical edge distance disagrees with original geometry")


def _selected_graph(
    graph: Path, selected: set[str], terminals: dict[str, Any], defines: Any
) -> tuple[dict[str, Any], list[Any]]:
    metadata = {}
    edges: dict[str, Any] = {}
    nodes: dict[int, Any] = {}
    needed_nodes = {row["node_id"] for row in terminals.values()}
    last_edge, last_node = "", 0
    selected_bytes = 0
    for section, row in _graph_records(graph):
        if section == "edges[]":
            key = _text(row["id"], "physical edge identity")
            if key <= last_edge:
                raise CaptureError("physical graph edge identities must be unique and canonical")
            last_edge = key
            if key in selected:
                _validate_edge(row)
                selected_bytes += len(_canonical(row))
                if selected_bytes > MAX_CONTENT_BYTES:
                    raise CaptureError("selected geometry exceeds Rust captured-content bound")
                edges[key] = row
                needed_nodes.update((row["from_node"], row["to_node"]))
        elif section == "nodes":
            if "edges" not in metadata:
                raise CaptureError("canonical graph edges must precede nodes for bounded capture")
            metadata[section] = row
        elif section == "nodes[]":
            ident = _uint(row["id"], "graph node ID", maximum=2**63 - 1)
            if ident <= last_node:
                raise CaptureError("physical graph node identities must be unique and canonical")
            last_node = ident
            if ident in needed_nodes:
                nodes[ident] = [
                    _coordinate(row["lon_e7"], 1_800_000_000),
                    _coordinate(row["lat_e7"], 900_000_000),
                ]
        elif not section.endswith("[]"):
            metadata[section] = row
    if (
        set(metadata)
        != {
            "schema_version",
            "routing_profile_version",
            "source",
            "profile",
            "license",
            "edges",
            "nodes",
            "turn_rules",
            "diagnostics",
        }
        or type(metadata["schema_version"]) is not int
        or metadata["schema_version"] != 1
    ):
        raise CaptureError("unsupported physical graph schema")
    if metadata["routing_profile_version"] != ROUTING_PROFILE_VERSION:
        raise CaptureError("unsupported physical routing profile version")
    if set(edges) != selected or set(nodes) != needed_nodes:
        raise CaptureError("selected physical edge or original endpoint node is missing")
    source = SourceIdentity(**metadata["source"])
    transport = defines["transport"]
    profile = VehicleProfile(**metadata["profile"])
    expected_profile = VehicleProfile(
        transport["TRUCK_GROSS_WEIGHT_KG"],
        transport["TRUCK_HEIGHT_MM"],
        transport["TRUCK_WIDTH_MM"],
        transport["TRUCK_LENGTH_MM"],
        transport["DEFAULT_MAXHEIGHT_MM"],
    )
    if (
        profile != expected_profile
        or transport["EVIDENCE_CLASS"] != "Designed"
        or source.buffer_degrees_e7 != transport["EXTRACTION_BUFFER_DEGREES_E7"]
    ):
        raise CaptureError("physical routing profile or extraction buffer differs from defines")
    for edge in edges.values():
        if (
            edge["shape_e7"][0] != nodes[edge["from_node"]]
            or edge["shape_e7"][-1] != nodes[edge["to_node"]]
        ):
            raise CaptureError("physical geometry endpoint disagrees with original node identity")
    for terminal in terminals.values():
        if nodes[terminal["node_id"]] != [terminal["node_lon_e7"], terminal["node_lat_e7"]]:
            raise CaptureError("county terminal coordinates disagree with original node identity")
    return {
        "source": {
            **asdict(source),
            "routing_profile_version": metadata["routing_profile_version"],
        },
        "profile": metadata["profile"],
    }, list(edges.values())


def _validate_continuity(orders: list[Any], edges: list[Any], terminals: dict[str, Any]) -> None:
    by_id = {edge["id"]: edge for edge in edges}
    for order in orders:
        if order["local"]:
            continue
        previous = terminals[order["supplier_county_geoid"]]["node_id"]
        distance = 0
        for key in order["edge_ids"]:
            edge = by_id[key]
            if previous != edge["from_node"]:
                raise CaptureError("selected physical path has a disconnected directed edge")
            previous = edge["to_node"]
            distance += edge["distance_mm"]
            _uint(distance, "summed physical distance")
        if (
            previous != terminals[order["buyer_county_geoid"]]["node_id"]
            or distance != order["distance_mm"]
        ):
            raise CaptureError(
                "selected path endpoint or summed distance disagrees with qualification"
            )


def _capacity_groups(document: Any, selected: set[str]) -> list[Any]:
    if not isinstance(document, list):
        raise CaptureError("capacity groups must be an explicit JSON list")
    keys, covered, groups = set(), set(), []
    for group in document:
        _fields(group, {"key", "label", "edge_keys"}, "capacity group")
        key = _text(group["key"], "capacity group key")
        _text(group["label"], "capacity group label")
        edges = group["edge_keys"]
        if (
            key in keys
            or not isinstance(edges, list)
            or not edges
            or any(not isinstance(edge, str) for edge in edges)
        ):
            raise CaptureError("duplicate capacity group or invalid explicit edge membership")
        if len(set(edges)) != len(edges) or not set(edges) <= selected:
            raise CaptureError("capacity group has duplicate or unused physical edges")
        keys.add(key)
        covered.update(edges)
        groups.append({**group, "edge_keys": sorted(edges)})
    if covered != selected:
        raise CaptureError("supplied capacity groups do not cover every selected physical edge")
    return sorted(groups, key=lambda group: group["key"])


def _gzip(payload: bytes) -> bytes:
    if len(payload) > MAX_CONTENT_BYTES:
        raise CaptureError("capture exceeds Rust decompressed-content bound")
    output = io.BytesIO()
    with gzip.GzipFile(
        fileobj=output, mode="wb", filename="", mtime=0, compresslevel=9
    ) as compressed:
        compressed.write(payload)
    result = output.getvalue()
    if len(result) > MAX_CONTENT_BYTES:
        raise CaptureError("capture exceeds Rust compressed-content bound")
    return result


def _publish(directory: Path, outputs: dict[str, bytes]) -> None:
    for name, payload in outputs.items():
        destination = directory / name
        if destination.is_symlink() or (
            destination.exists()
            and (destination.stat().st_size != len(payload) or destination.read_bytes() != payload)
        ):
            raise CaptureError(f"refusing unrelated output overwrite: {destination}")
    staged: list[tuple[Path, Path]] = []
    created: list[Path] = []
    try:
        for name, payload in outputs.items():
            destination = directory / name
            if destination.exists():
                continue
            with tempfile.NamedTemporaryFile(
                dir=directory, prefix=f".{name}.", delete=False
            ) as stream:
                temporary = Path(stream.name)
                staged.append((temporary, destination))
                stream.write(payload)
                stream.flush()
                os.fsync(stream.fileno())
        # The manifest is last. Exclusive links cannot replace another writer's file.
        for temporary, destination in staged:
            os.link(temporary, destination)
            created.append(destination)
    except OSError:
        for destination in created:
            destination.unlink()
        raise
    finally:
        for temporary, _ in staged:
            temporary.unlink(missing_ok=True)


def capture_sources(
    *,
    defines: Path,
    qualification: Path,
    matrix: Path,
    graph: Path,
    roster: Path,
    capacity_groups: Path,
) -> dict[str, str]:
    directory = defines.resolve().parent
    if {
        path.resolve() for path in (defines, qualification, matrix, graph, roster, capacity_groups)
    } & {directory / name for name in OUTPUT_NAMES}:
        raise CaptureError("capture output overlaps an input authority")
    defines_bytes = defines.read_bytes()
    definitions = tomllib.loads(defines_bytes.decode("utf-8"))
    if (
        definitions["SCHEMA_VERSION"] != 3
        or definitions["statewide"]["EVIDENCE_CLASS"] != "Designed"
    ):
        raise CaptureError("capture requires schema V3 Designed statewide definitions")
    limit = _uint(
        definitions["statewide"]["TERMINAL_ATTACHMENT_LIMIT_METERS"],
        "terminal attachment limit",
        maximum=MAX_U64 // 1000,
    )
    qualified = _fields(_json(qualification), QUALIFICATION_FIELDS, "commodity qualification")
    if (
        qualified["schema"] != "MichiganCommodityCircuitV1"
        or qualified["evidence_class"] != "Designed"
        or qualified["qualified"] is not True
        or qualified["diagnostics"] != []
    ):
        raise CaptureError("commodity circuit is disconnected or not qualified")
    for name in ("owners", "processes", "orders", "retail_final_demands"):
        if not isinstance(qualified[name], list):
            raise CaptureError(f"invalid qualified {name}")
    defines_hash = hashlib.sha256(defines_bytes).hexdigest()
    for field, actual in (("defines_sha256", defines_hash), ("roster_sha256", _hash(roster))):
        if qualified[field] != actual:
            raise CaptureError(f"qualification {field} differs from supplied authority")
    matrix_document = _json(
        matrix, compressed=True, bound=MAX_MATRIX_BYTES, expected_sha256=qualified["paths_sha256"]
    )
    terminals, paths = _validate_matrix(matrix_document, qualified["owners"], limit)
    if matrix_document["source_pins"]["defines_sha256"] != defines_hash:
        raise CaptureError("county matrix defines SHA-256 differs from current defines")
    graph_stat = graph.stat()
    graph_hash = _hash(graph)
    if matrix_document["source_pins"]["graph_sha256"] != graph_hash:
        raise CaptureError("county matrix graph SHA-256 differs from supplied graph")
    selected = _selected_orders(qualified, paths)
    groups = _capacity_groups(_json(capacity_groups), selected)
    metadata, edges = _selected_graph(graph, selected, terminals, definitions)
    final_stat = graph.stat()
    if (final_stat.st_ino, final_stat.st_size, final_stat.st_mtime_ns, final_stat.st_ctime_ns) != (
        graph_stat.st_ino,
        graph_stat.st_size,
        graph_stat.st_mtime_ns,
        graph_stat.st_ctime_ns,
    ):
        raise CaptureError("physical graph changed during capture")
    _validate_continuity(qualified["orders"], edges, terminals)
    physical = {
        "source": {**metadata["source"], "graph_sha256": graph_hash},
        "profile": metadata["profile"],
        "terminal_source_pins": matrix_document["source_pins"],
        "terminal_policy": matrix_document["policy"],
        "terminal_attachment_limit_meters": limit,
        "terminals": matrix_document["terminals"],
        "edges": edges,
        "capacity_groups": groups,
    }
    qualification_bytes, physical_bytes = _gzip(_canonical(qualified)), _gzip(_canonical(physical))
    manifest = {
        "schema": "MichiganStatewideSourcesV1",
        "defines_sha256": defines_hash,
        "qualification_sha256": hashlib.sha256(qualification_bytes).hexdigest(),
        "physical_network_sha256": hashlib.sha256(physical_bytes).hexdigest(),
    }
    _publish(
        directory,
        {
            OUTPUT_NAMES[0]: qualification_bytes,
            OUTPUT_NAMES[1]: physical_bytes,
            OUTPUT_NAMES[2]: _canonical(manifest),
        },
    )
    return manifest


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("defines", "qualification", "matrix", "graph", "roster", "capacity-groups"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        print(json.dumps(capture_sources(**vars(args)), sort_keys=True))
        return 0
    except (
        CaptureError,
        TerminalQualificationError,
        RoadNetworkError,
        OSError,
        EOFError,
        KeyError,
        TypeError,
        ValueError,
    ) as error:
        print(f"statewide source capture refused: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
