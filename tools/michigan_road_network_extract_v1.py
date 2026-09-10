"""Stream a pinned PBF through a bounded-memory, disk-backed road extraction.

Run with ``python -m tools.michigan_road_network_extract_v1``. The North America
file is never loaded into memory. Graph compilation subsequently loads only the
regional road records. A byte-pinned footprint is an extraction extent, not a
legal administrative boundary or economic geography extension.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib
import importlib.metadata
import json
import math
import os
import re
import sqlite3
import subprocess
import sys
import tempfile
from collections.abc import Callable, Iterator, Sequence
from dataclasses import asdict, replace
from decimal import Decimal
from pathlib import Path
from types import ModuleType
from typing import Any

import pyproj
import shapely  # type: ignore[import-untyped]
from shapely.geometry import shape  # type: ignore[import-untyped]
from shapely.ops import unary_union  # type: ignore[import-untyped]
from tools.michigan_road_network_v1 import (
    ROAD_CLASSES,
    Node,
    Restriction,
    RestrictionMember,
    RoadNetworkError,
    SourceIdentity,
    VehicleProfile,
    Way,
    build_graph,
    write_graph,
)

ADAPTER_VERSION = "michigan-road-extract-v2-regional-freight"
BATCH_SIZE = 8192
QUERY_SIZE = 900


def require_osmium() -> ModuleType:
    try:
        return importlib.import_module("osmium")
    except ModuleNotFoundError as error:
        raise RoadNetworkError(
            "PBF extraction requires the pinned project 'osmium' dependency; install the locked project environment"
        ) from error


class IdBloom:
    """Fixed allocation, deterministic no-false-negative ID prefilter; SQLite judges."""

    def __init__(self, byte_count: int) -> None:
        if type(byte_count) is not int or byte_count <= 0:
            raise RoadNetworkError("Bloom filter byte count must be positive")
        self.bits = bytearray(byte_count)
        self.modulus = byte_count * 8

    def _positions(self, ident: int) -> tuple[int, int]:
        mixed = ident * 0x9E3779B185EBCA87 & ((1 << 64) - 1)
        mixed ^= mixed >> 33
        return mixed % self.modulus, (mixed ^ (ident * 0xC2B2AE3D27D4EB4F)) % self.modulus

    def add(self, ident: int) -> None:
        for position in self._positions(ident):
            self.bits[position >> 3] |= 1 << (position & 7)

    def __contains__(self, ident: int) -> bool:
        return all(
            self.bits[position >> 3] & (1 << (position & 7)) for position in self._positions(ident)
        )


class Footprint:
    def __init__(self, path: Path, buffer_degrees_e7: int) -> None:
        payload = path.read_bytes()
        self.sha256 = hashlib.sha256(payload).hexdigest()
        if type(buffer_degrees_e7) is not int or not 0 <= buffer_degrees_e7 <= 1_000_000:
            raise RoadNetworkError(
                "footprint buffer must be integer E7 degrees between 0 and 0.1 degree"
            )
        try:
            data = json.loads(payload)
            if data["type"] == "FeatureCollection":
                geometries = [shape(feature["geometry"]) for feature in data["features"]]
            elif data["type"] == "Feature":
                geometries = [shape(data["geometry"])]
            else:
                geometries = [shape(data)]
        except (KeyError, TypeError, ValueError) as error:
            raise RoadNetworkError(f"invalid footprint GeoJSON: {error}") from error
        if not geometries or any(
            geometry.is_empty
            or not geometry.is_valid
            or geometry.geom_type not in {"Polygon", "MultiPolygon"}
            for geometry in geometries
        ):
            raise RoadNetworkError("footprint must contain valid, nonempty polygons")
        geometry = unary_union(geometries)
        if not (
            -180 <= geometry.bounds[0] < geometry.bounds[2] <= 180
            and -90 <= geometry.bounds[1] < geometry.bounds[3] <= 90
        ):
            raise RoadNetworkError("footprint coordinates must be WGS84 longitude/latitude")
        geometry = geometry.buffer(buffer_degrees_e7 / 1e7) if buffer_degrees_e7 else geometry
        self.buffer_degrees_e7 = buffer_degrees_e7
        self.bounds = geometry.bounds
        self.geometry = geometry
        shapely.prepare(self.geometry)

    def covered_nodes(self, nodes: Sequence[Node]) -> Iterator[Node]:
        """Point intersection equals polygon coverage, including both ring boundaries."""
        if not nodes:
            return
        selected = shapely.intersects_xy(
            self.geometry,
            [node.lon_e7 / 1e7 for node in nodes],
            [node.lat_e7 / 1e7 for node in nodes],
        )
        for node, covered in zip(nodes, selected, strict=True):
            if covered:
                yield node


class RoadSpool:
    """Task-owned scratch database. Its implementation bytes are not the artifact."""

    def __init__(self, path: Path, bloom_bytes: int = 64 * 1024 * 1024) -> None:
        if path.exists():
            raise RoadNetworkError(f"refusing to overwrite existing road spool: {path}")
        self.database = sqlite3.connect(path)
        self.database.executescript("""
            PRAGMA journal_mode=WAL;
            PRAGMA busy_timeout=60000;
            PRAGMA cache_size=-65536;
            PRAGMA temp_store=FILE;
            CREATE TABLE nodes(id INTEGER PRIMARY KEY, lon_e7 INTEGER NOT NULL, lat_e7 INTEGER NOT NULL, tags TEXT NOT NULL, version INTEGER NOT NULL, timestamp TEXT NOT NULL);
            CREATE TABLE ways(id INTEGER PRIMARY KEY, nodes TEXT NOT NULL, tags TEXT NOT NULL, version INTEGER NOT NULL, timestamp TEXT NOT NULL);
            CREATE TABLE restrictions(id INTEGER PRIMARY KEY, members TEXT NOT NULL, tags TEXT NOT NULL, version INTEGER NOT NULL, timestamp TEXT NOT NULL);
            CREATE TABLE needed_nodes(id INTEGER PRIMARY KEY);
        """)
        self.footprint_ids = IdBloom(bloom_bytes)
        self.way_ids = IdBloom(max(1024, bloom_bytes // 8))
        self.node_batch: list[tuple[object, ...]] = []
        self.pending = 0

    def close(self) -> None:
        self.database.close()

    def add_footprint_node(self, row: Node) -> None:
        self.footprint_ids.add(row.id)
        self.node_batch.append(
            (row.id, row.lon_e7, row.lat_e7, json.dumps(row.tags), row.version, row.timestamp)
        )
        if len(self.node_batch) >= BATCH_SIZE:
            self.flush_nodes()

    def flush_nodes(self) -> None:
        self.database.executemany("INSERT INTO nodes VALUES(?,?,?,?,?,?)", self.node_batch)
        self.node_batch.clear()
        self.database.commit()

    def _any_present(self, table: str, ids: Sequence[int]) -> bool:
        if table not in {"nodes", "ways"}:
            raise RoadNetworkError("invalid spool lookup table")
        for start in range(0, len(ids), QUERY_SIZE):
            chunk = ids[start : start + QUERY_SIZE]
            placeholders = ",".join("?" for _ in chunk)
            if self.database.execute(
                f"SELECT 1 FROM {table} WHERE id IN ({placeholders}) LIMIT 1", chunk
            ).fetchone():
                return True
        return False

    def add_touching_way(self, row: Way) -> None:
        if dict(row.tags).get("highway") not in ROAD_CLASSES:
            return
        candidates = [ident for ident in row.node_ids if ident in self.footprint_ids]
        if not candidates or not self._any_present("nodes", candidates):
            return
        self.database.execute(
            "INSERT INTO ways VALUES(?,?,?,?,?)",
            (row.id, json.dumps(row.node_ids), json.dumps(row.tags), row.version, row.timestamp),
        )
        self.database.executemany(
            "INSERT OR IGNORE INTO needed_nodes VALUES(?)", ((ident,) for ident in row.node_ids)
        )
        self.way_ids.add(row.id)
        self._batch_commit()

    def add_touching_restriction(self, row: Restriction) -> None:
        if not dict(row.tags).get("type", "").startswith("restriction"):
            return
        ids = [
            member.ref
            for member in row.members
            if member.kind == "w" and member.ref in self.way_ids
        ]
        if not ids or not self._any_present("ways", ids):
            return
        self.database.execute(
            "INSERT INTO restrictions VALUES(?,?,?,?,?)",
            (
                row.id,
                json.dumps([asdict(member) for member in row.members]),
                json.dumps(row.tags),
                row.version,
                row.timestamp,
            ),
        )
        self._batch_commit()

    def _batch_commit(self) -> None:
        self.pending += 1
        if self.pending >= BATCH_SIZE:
            self.database.commit()
            self.pending = 0

    def missing_filter(self) -> IdBloom | None:
        self.database.commit()
        self.database.execute("DELETE FROM nodes WHERE id NOT IN (SELECT id FROM needed_nodes)")
        self.database.execute("DELETE FROM needed_nodes WHERE id IN (SELECT id FROM nodes)")
        self.database.commit()
        if self.database.execute("SELECT 1 FROM needed_nodes LIMIT 1").fetchone() is None:
            return None
        result = IdBloom(8 * 1024 * 1024)
        for (ident,) in self.database.execute("SELECT id FROM needed_nodes"):
            result.add(ident)
        return result

    def add_missing_node(self, row: Node) -> None:
        if not self.database.execute("SELECT 1 FROM needed_nodes WHERE id=?", (row.id,)).fetchone():
            return
        self.database.execute(
            "INSERT INTO nodes VALUES(?,?,?,?,?,?)",
            (row.id, row.lon_e7, row.lat_e7, json.dumps(row.tags), row.version, row.timestamp),
        )
        self.database.execute("DELETE FROM needed_nodes WHERE id=?", (row.id,))
        self._batch_commit()

    def finish(self) -> None:
        self.database.commit()
        missing = self.database.execute(
            "SELECT id FROM needed_nodes ORDER BY id LIMIT 1"
        ).fetchone()
        if missing is not None:
            raise RoadNetworkError(f"PBF is missing required road node {missing[0]}")

    def nodes(self) -> Iterator[Node]:
        for ident, lon, lat, tags, version, timestamp in self.database.execute(
            "SELECT * FROM nodes ORDER BY id"
        ):
            yield Node(
                ident, lon, lat, tuple(tuple(pair) for pair in json.loads(tags)), version, timestamp
            )

    def ways(self) -> Iterator[Way]:
        for ident, nodes, tags, version, timestamp in self.database.execute(
            "SELECT * FROM ways ORDER BY id"
        ):
            yield Way(
                ident,
                tuple(json.loads(nodes)),
                tuple(tuple(pair) for pair in json.loads(tags)),
                version,
                timestamp,
            )

    def restrictions(self) -> Iterator[Restriction]:
        for ident, members, tags, version, timestamp in self.database.execute(
            "SELECT * FROM restrictions ORDER BY id"
        ):
            yield Restriction(
                ident,
                tuple(RestrictionMember(**member) for member in json.loads(members)),
                tuple(tuple(pair) for pair in json.loads(tags)),
                version,
                timestamp,
            )


def _node(row: Any) -> Node:
    if not row.location.valid():
        raise RoadNetworkError(f"OSM node {row.id} has no valid location")
    return Node(
        row.id,
        row.location.x,
        row.location.y,
        tuple(sorted((tag.k, tag.v) for tag in row.tags)),
        row.version,
        str(row.timestamp),
    )


def _way(row: Any) -> Way:
    return Way(
        row.id,
        tuple(node.ref for node in row.nodes),
        tuple(sorted((tag.k, tag.v) for tag in row.tags)),
        row.version,
        str(row.timestamp),
    )


def _restriction(row: Any) -> Restriction:
    return Restriction(
        row.id,
        tuple(RestrictionMember(member.type, member.ref, member.role) for member in row.members),
        tuple(sorted((tag.k, tag.v) for tag in row.tags)),
        row.version,
        str(row.timestamp),
    )


def extract_pbf(
    pbf: Path, footprint: Footprint, spool: RoadSpool, *, progress: Callable[[str], None] = print
) -> None:
    osmium = require_osmium()

    class FootprintNodes(osmium.SimpleHandler):  # type: ignore[name-defined,misc]
        def __init__(self) -> None:
            super().__init__()
            self.pending: list[Node] = []

        def flush(self) -> None:
            for node in footprint.covered_nodes(self.pending):
                spool.add_footprint_node(node)
            self.pending.clear()

        def node(self, row: Any) -> None:
            if not row.location.valid():
                raise RoadNetworkError(f"OSM node {row.id} has no valid location")
            # Pyosmium rows expire after the callback; retain bounded immutable facts.
            self.pending.append(_node(row))
            if len(self.pending) == BATCH_SIZE:
                self.flush()

    class Roads(osmium.SimpleHandler):  # type: ignore[name-defined,misc]
        def way(self, row: Any) -> None:
            if row.tags.get("highway") in ROAD_CLASSES:
                spool.add_touching_way(_way(row))

        def relation(self, row: Any) -> None:
            if row.tags.get("type", "").startswith("restriction"):
                spool.add_touching_restriction(_restriction(row))

    class MissingNodes(osmium.SimpleHandler):  # type: ignore[name-defined,misc]
        def node(self, row: Any) -> None:
            if missing is not None and row.id in missing:
                spool.add_missing_node(_node(row))

    progress("road extraction: stream footprint nodes to SQLite")
    footprint_nodes = FootprintNodes()
    footprint_nodes.apply_file(str(pbf))
    footprint_nodes.flush()
    spool.flush_nodes()
    progress("road extraction: stream complete touching road ways and restrictions")
    Roads().apply_file(str(pbf))
    missing = spool.missing_filter()
    if missing is not None:
        progress("road extraction: complete outside-boundary node references")
        MissingNodes().apply_file(str(pbf))
    spool.finish()


NATIVE_VERSION = "osmium version 1.18.0"
PREPARATION_SCHEMA = "MichiganRoadNativePreprocessingV1"
ROAD_FILTER = "w/highway=" + ",".join(sorted(ROAD_CLASSES))
RESTRICTION_FILTER = "r/type=restriction*"


def native_bounds_e7(footprint: Footprint) -> tuple[int, int, int, int]:
    """Strictly enclose the already-buffered footprint, including boundary nodes."""
    west, south, east, north = footprint.bounds
    bounds = (
        math.floor(west * 10_000_000) - 1,
        math.floor(south * 10_000_000) - 1,
        math.ceil(east * 10_000_000) + 1,
        math.ceil(north * 10_000_000) + 1,
    )
    if not (-1_800_000_000 <= bounds[0] < bounds[2] <= 1_800_000_000) or not (
        -900_000_000 <= bounds[1] < bounds[3] <= 900_000_000
    ):
        raise RoadNetworkError("native bounding box must fit within WGS84 longitude/latitude")
    return bounds


def native_commands(
    binary: Path, original: Path, regional: Path, prepared: Path, footprint: Footprint
) -> tuple[tuple[str, ...], tuple[str, ...]]:
    bbox = ",".join(
        format(Decimal(value) / 10_000_000, ".7f") for value in native_bounds_e7(footprint)
    )
    common = ("--output-format=pbf", "--output-header=osmosis_replication_timestamp!", "--fsync")
    return (
        (
            str(binary.resolve()),
            "extract",
            f"--bbox={bbox}",
            "--strategy=complete_ways",
            *common,
            f"--output={regional.resolve()}",
            str(original.resolve()),
            "--verbose",
        ),
        (
            str(binary.resolve()),
            "tags-filter",
            *common,
            f"--output={prepared.resolve()}",
            str(regional.resolve()),
            ROAD_FILTER,
            RESTRICTION_FILTER,
            "--verbose",
        ),
    )


def _file_pin(path: Path) -> dict[str, Any]:
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"path": str(path.resolve()), "sha256": digest, "bytes": path.stat().st_size}


def _replication_timestamp(pbf: Path) -> str:
    osmium = require_osmium()
    reader = osmium.io.Reader(str(pbf), osmium.osm.osm_entity_bits.NOTHING)
    try:
        return str(reader.header().get("osmosis_replication_timestamp"))
    finally:
        reader.close()


def _validate_native_commands(
    commands: Sequence[Sequence[str]],
    binary: Path,
    original: Path,
    regional: Path,
    prepared: Path,
    footprint: Footprint,
    working_directory: Path,
) -> None:
    """Validate the allowlisted semantics while retaining exact executed argv bytes."""
    if len(commands) != 2:
        raise RoadNetworkError("native preparation requires exactly extract and tags-filter stages")
    expected = native_commands(binary, original, regional, prepared, footprint)
    for actual, wanted in zip(commands, expected, strict=True):
        if not all(isinstance(value, str) for value in actual) or len(actual) < 3:
            raise RoadNetworkError("invalid native command argv")
        if (working_directory / actual[0]).resolve() != binary.resolve() or actual[1] != wanted[1]:
            raise RoadNetworkError("native stage binary or command differs from pinned policy")
        # argparse accepts equivalent --key=value and --key value forms. No unknown
        # flags are allowed: in particular no clipping strategy or tag/reference removal.
        parser = argparse.ArgumentParser(add_help=False, allow_abbrev=False, exit_on_error=False)
        for name in ("output-format", "output-header", "output", "bbox", "strategy"):
            parser.add_argument(f"--{name}")
        parser.add_argument("--fsync", action="store_true")
        parser.add_argument("--verbose", action="store_true")
        parser.add_argument("inputs", nargs="*")
        try:
            parsed, unknown = parser.parse_known_args(actual[2:])
        except argparse.ArgumentError as error:
            raise RoadNetworkError(f"invalid native command: {error}") from error
        if (
            unknown
            or not parsed.fsync
            or parsed.output_format != "pbf"
            or parsed.output_header != "osmosis_replication_timestamp!"
        ):
            raise RoadNetworkError("native stage flags differ from pinned policy")
        inputs = (
            [str((working_directory / parsed.inputs[0]).resolve()), *parsed.inputs[1:]]
            if parsed.inputs
            else []
        )
        if actual[1] == "extract":
            if (
                parsed.strategy != "complete_ways"
                or parsed.bbox != wanted[2].split("=", 1)[1]
                or inputs != [str(original.resolve())]
            ):
                raise RoadNetworkError(
                    "native extract bounds, strategy or input differ from pinned policy"
                )
            output = regional
        else:
            if (
                parsed.strategy is not None
                or parsed.bbox is not None
                or inputs != [str(regional.resolve()), ROAD_FILTER, RESTRICTION_FILTER]
            ):
                raise RoadNetworkError(
                    "native tags-filter input or expressions differ from pinned policy"
                )
            output = prepared
        if (
            parsed.output is None
            or (working_directory / parsed.output).resolve() != output.resolve()
        ):
            raise RoadNetworkError("native stage output differs from pinned artifact path")


def _native_identity(binary: Path, expected_sha256: str) -> dict[str, Any]:
    pinned = _file_pin(binary)
    if pinned["sha256"] != expected_sha256:
        raise RoadNetworkError("native osmium binary SHA-256 differs from supplied pin")
    try:
        version = subprocess.run(
            [str(binary.resolve()), "--version"], check=True, capture_output=True, text=True
        ).stdout
    except subprocess.CalledProcessError as error:
        raise RoadNetworkError(f"native osmium version command failed: {error}") from error
    if version.splitlines()[0:1] != [NATIVE_VERSION]:
        raise RoadNetworkError("native preprocessing requires osmium 1.18.0")
    return {"path": pinned["path"], "sha256": pinned["sha256"], "version": version}


def _canonical_preparation(data: Any) -> bytes:
    return (
        json.dumps(data, sort_keys=True, ensure_ascii=True, separators=(",", ":"), allow_nan=False)
        + "\n"
    ).encode("ascii")


def record_native_preparation(
    *,
    source: SourceIdentity,
    footprint: Footprint,
    original: Path,
    regional: Path,
    prepared: Path,
    binary: Path,
    binary_sha256: str,
    commands: Sequence[Sequence[str]],
    manifest: Path,
    working_directory: Path | None = None,
) -> str:
    """Record completed stages using an original SourceIdentity already verified by the caller.

    This explicit continuation API does not rehash the original continent-sized PBF.
    It hashes both intermediate files and validates their retained source timestamp.
    The caller supplies the actual successful argv; this is an operator attestation,
    not a reconstruction or proof of an unobserved subprocess invocation.
    """
    cwd = (working_directory or Path.cwd()).resolve()
    if manifest.exists():
        raise RoadNetworkError("refusing to overwrite an existing native preparation manifest")
    if len({path.resolve() for path in (original, regional, prepared, manifest)}) != 4:
        raise RoadNetworkError("native preparation paths must be distinct")
    if (
        source.footprint_sha256 != footprint.sha256
        or source.buffer_degrees_e7 != footprint.buffer_degrees_e7
    ):
        raise RoadNetworkError("native preparation footprint differs from verified source")
    if original.stat().st_size != source.pbf_bytes:
        raise RoadNetworkError("original PBF byte count differs from verified source")
    _validate_native_commands(commands, binary, original, regional, prepared, footprint, cwd)
    native = _native_identity(binary, binary_sha256)
    for path in (original, regional, prepared):
        if _replication_timestamp(path) != source.replication_timestamp:
            raise RoadNetworkError("native PBF replication timestamp differs from verified source")
    data = {
        "schema": PREPARATION_SCHEMA,
        "source": asdict(source),
        "original_path": str(original.resolve()),
        "working_directory": str(cwd),
        "bounds_e7": native_bounds_e7(footprint),
        "native": native,
        "commands": [list(command) for command in commands],
        "regional": _file_pin(regional),
        "prepared": _file_pin(prepared),
    }
    payload = _canonical_preparation(data)
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=manifest.parent, prefix=f".{manifest.name}.", delete=False
        ) as stream:
            temporary = Path(stream.name)
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, manifest)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return hashlib.sha256(payload).hexdigest()


def prepare_native_pbf(
    *,
    source: SourceIdentity,
    footprint: Footprint,
    original: Path,
    regional: Path,
    prepared: Path,
    binary: Path,
    binary_sha256: str,
    manifest: Path,
) -> str:
    """Run the two native passes, then attest successful outputs for precise extraction."""
    for path in (regional, prepared, manifest):
        if path.exists():
            raise RoadNetworkError(f"refusing existing native preparation output: {path}")
    _native_identity(binary, binary_sha256)
    commands = native_commands(binary, original, regional, prepared, footprint)
    for command in commands:
        try:
            subprocess.run(command, check=True)
        except subprocess.CalledProcessError as error:
            raise RoadNetworkError(
                f"native {command[1]} failed with status {error.returncode}; partial outputs retained"
            ) from error
    return record_native_preparation(
        source=source,
        footprint=footprint,
        original=original,
        regional=regional,
        prepared=prepared,
        binary=binary,
        binary_sha256=binary_sha256,
        commands=commands,
        manifest=manifest,
    )


def verified_preparation_source(
    *,
    original: Path,
    prepared: Path,
    manifest: Path,
    source_manifest: Path,
    footprint: Footprint,
) -> SourceIdentity:
    """Verify the prepared input; retain the original source without reading it again."""
    payload = manifest.read_bytes()
    data = json.loads(payload)
    expected_keys = {
        "schema",
        "source",
        "original_path",
        "working_directory",
        "bounds_e7",
        "native",
        "commands",
        "regional",
        "prepared",
    }
    if (
        not isinstance(data, dict)
        or set(data) != expected_keys
        or data["schema"] != PREPARATION_SCHEMA
        or _canonical_preparation(data) != payload
    ):
        raise RoadNetworkError("invalid canonical native preparation manifest")
    source = _source_identity(source_manifest, footprint, footprint.buffer_degrees_e7)
    if (
        data["source"] != asdict(source)
        or data["original_path"] != str(original.resolve())
        or data["bounds_e7"] != list(native_bounds_e7(footprint))
    ):
        raise RoadNetworkError("native preparation source or footprint differs from current pins")
    try:
        native = data["native"]
        if (
            set(native) != {"path", "sha256", "version"}
            or not re.fullmatch(r"[0-9a-f]{64}", native["sha256"])
            or native["version"].splitlines()[0:1] != [NATIVE_VERSION]
        ):
            raise RoadNetworkError("invalid native binary identity")
        for name in ("regional", "prepared"):
            pin = data[name]
            if (
                set(pin) != {"path", "sha256", "bytes"}
                or not re.fullmatch(r"[0-9a-f]{64}", pin["sha256"])
                or type(pin["bytes"]) is not int
                or pin["bytes"] <= 0
            ):
                raise RoadNetworkError("invalid native intermediate artifact pin")
        _validate_native_commands(
            data["commands"],
            Path(native["path"]),
            original,
            Path(data["regional"]["path"]),
            prepared,
            footprint,
            Path(data["working_directory"]),
        )
    except (KeyError, TypeError, AttributeError) as error:
        raise RoadNetworkError(f"invalid native preparation fields: {error}") from error
    if data["prepared"] != _file_pin(prepared):
        raise RoadNetworkError(
            "prepared PBF path, SHA-256 or byte count differs from preparation manifest"
        )
    if _replication_timestamp(prepared) != source.replication_timestamp:
        raise RoadNetworkError("prepared PBF replication timestamp differs from pinned source")
    return replace(
        source,
        extraction_version=f"{source.extraction_version};native-preparation-sha256={hashlib.sha256(payload).hexdigest()}",
    )


def _source_identity(
    manifest: Path, footprint: Footprint, buffer_degrees_e7: int
) -> SourceIdentity:
    data = json.loads(manifest.read_bytes())
    expected = {"pbf_sha256", "pbf_bytes", "pbf_url", "replication_timestamp", "footprint_sha256"}
    if set(data) != expected:
        raise RoadNetworkError(f"source manifest must contain exactly {sorted(expected)}")
    if footprint.sha256 != data["footprint_sha256"]:
        raise RoadNetworkError("footprint SHA-256 differs from pinned source manifest")
    versions = f"{ADAPTER_VERSION};osmium={importlib.metadata.version('osmium')};shapely={shapely.__version__};GEOS={shapely.geos_version_string}"
    return SourceIdentity(
        **data,
        buffer_degrees_e7=buffer_degrees_e7,
        extraction_version=versions,
        distance_version=f"WGS84-segment-mm-half-up;pyproj={pyproj.__version__};PROJ={pyproj.proj_version_str}",
    )


def _verified_source(
    pbf: Path, manifest: Path, footprint: Footprint, buffer_degrees_e7: int
) -> SourceIdentity:
    source = _source_identity(manifest, footprint, buffer_degrees_e7)
    if pbf.stat().st_size != source.pbf_bytes:
        raise RoadNetworkError("PBF byte count differs from pinned source manifest")
    if _file_pin(pbf)["sha256"] != source.pbf_sha256:
        raise RoadNetworkError("PBF SHA-256 differs from pinned source manifest")
    if _replication_timestamp(pbf) != source.replication_timestamp:
        raise RoadNetworkError("PBF replication timestamp differs from pinned source manifest")
    return source


def _extraction_input(
    args: argparse.Namespace, footprint: Footprint
) -> tuple[Path, SourceIdentity]:
    native_options = (args.native_osmium, args.native_osmium_sha256, args.regional_pbf)
    if args.prepared_pbf is None and args.preparation_manifest is None:
        if any(value is not None for value in native_options):
            raise RoadNetworkError(
                "native preprocessing requires prepared PBF and preparation manifest paths"
            )
        return args.pbf, _verified_source(
            args.pbf, args.source_manifest, footprint, args.buffer_degrees_e7
        )
    if args.prepared_pbf is None or args.preparation_manifest is None:
        raise RoadNetworkError("prepared PBF and preparation manifest must be supplied together")
    if any(value is not None for value in native_options):
        if any(value is None for value in native_options):
            raise RoadNetworkError(
                "native preprocessing requires binary, binary SHA-256 and regional PBF path"
            )
        source = _verified_source(args.pbf, args.source_manifest, footprint, args.buffer_degrees_e7)
        prepare_native_pbf(
            source=source,
            footprint=footprint,
            original=args.pbf,
            regional=args.regional_pbf,
            prepared=args.prepared_pbf,
            binary=args.native_osmium,
            binary_sha256=args.native_osmium_sha256,
            manifest=args.preparation_manifest,
        )
    return args.prepared_pbf, verified_preparation_source(
        original=args.pbf,
        prepared=args.prepared_pbf,
        manifest=args.preparation_manifest,
        source_manifest=args.source_manifest,
        footprint=footprint,
    )


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pbf", type=Path, required=True)
    parser.add_argument("--source-manifest", type=Path, required=True)
    parser.add_argument("--footprint", type=Path, required=True)
    parser.add_argument("--buffer-degrees-e7", type=int, default=0)
    parser.add_argument("--vehicle-profile", type=Path, required=True)
    parser.add_argument("--spool", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--bloom-mib", type=int, default=64)
    parser.add_argument(
        "--prepared-pbf",
        type=Path,
        help="Native-filtered input, verified against the preparation manifest",
    )
    parser.add_argument(
        "--preparation-manifest", type=Path, help="Canonical native-stage provenance sidecar"
    )
    parser.add_argument(
        "--native-osmium",
        type=Path,
        help="Run native preprocessing with this pinned osmium 1.18.0 binary",
    )
    parser.add_argument(
        "--native-osmium-sha256", help="Required binary SHA-256 when running native preprocessing"
    )
    parser.add_argument("--regional-pbf", type=Path, help="New complete-ways intermediate output")
    args = parser.parse_args(argv)
    spool = None
    try:
        require_osmium()
        if not 1 <= args.bloom_mib <= 1024:
            raise RoadNetworkError("Bloom allocation must be between 1 and 1024 MiB")
        footprint = Footprint(args.footprint, args.buffer_degrees_e7)
        input_pbf, source = _extraction_input(args, footprint)
        profile = VehicleProfile(**json.loads(args.vehicle_profile.read_bytes()))
        spool = RoadSpool(args.spool, args.bloom_mib * 1024 * 1024)
        extract_pbf(input_pbf, footprint, spool)
        print("road extraction: compile the regional directed graph")
        graph = build_graph(
            spool.nodes(), spool.ways(), spool.restrictions(), profile=profile, source=source
        )
        digest = write_graph(graph, args.output)
        print(
            json.dumps(
                {
                    "sha256": digest,
                    "nodes": len(graph.nodes),
                    "edges": len(graph.edges),
                    "turn_rules": len(graph.turn_rules),
                    "diagnostics": len(graph.diagnostics),
                },
                sort_keys=True,
            )
        )
        return 0
    except (RoadNetworkError, OSError, TypeError, ValueError, sqlite3.Error) as error:
        print(f"road extraction refused: {error}", file=sys.stderr)
        return 2
    finally:
        if spool is not None:
            spool.close()


if __name__ == "__main__":
    raise SystemExit(main())
