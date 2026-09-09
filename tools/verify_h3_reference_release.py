#!/usr/bin/env python3
"""Prove the pinned PER-62 H3 identity cohort before PostgreSQL installation."""

from __future__ import annotations

import argparse
import hashlib
import math
import os
import stat
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import h3
import pyarrow as pa  # type: ignore[import-untyped]
import pyarrow.parquet as pq  # type: ignore[import-untyped]

SOURCE_DOMAIN: Final = b"babylon.h3.reference-source.v1\0"
MAX_PARQUET_ARTIFACTS: Final = 5
MAX_SOURCE_ROWS: Final = 48_764
MAX_P27_ARCHIVES: Final = 3
SOURCE_ROWS: Final = 48_764
R5_ROWS: Final = 3_192
R7_ROWS: Final = 45_572
SOURCE_FIXTURE_BYTES: Final = 390_151
SOURCE_DIGEST: Final = "a4685e6ad882930e7064cb225ee649155fb74e52ef8b7d7550691a70a6087f5a"
R5_DIGEST: Final = "83c093393bdf7a0e30ace8e208f3bcaa366fb7c6350abf7ff55d446322dcca87"
R7_DIGEST: Final = "7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b"
P27_SESSION_IDS: Final = (
    "5ef6b154-0fe8-44a1-ac12-58d40e69a75c",
    "ccf3082d-f94d-41d8-9b2a-27d488585507",
    "7f6599f0-2c56-4b91-a955-1bb50e0d8987",
)


class VerificationError(Exception):
    """The pinned release or P27 identity evidence did not match its contract."""


@dataclass(frozen=True)
class ArtifactSpec:
    """Exact bytes and decoded shape for one governed Parquet artifact."""

    name: str
    path: Path
    byte_count: int
    sha256: str
    row_count: int
    schema: pa.Schema
    session_id: str | None = None


@dataclass(frozen=True)
class PinnedArtifact:
    """Hash-proved immutable bytes that may now be decoded."""

    spec: ArtifactSpec
    payload: bytes


@dataclass(frozen=True)
class CanonicalCells:
    """One bounded H3 column in both source order and canonical order."""

    raw_rows: tuple[int, ...]
    resolutions: tuple[int, ...]
    ordered: tuple[tuple[int, int], ...]
    identity_set: frozenset[int]


@dataclass(frozen=True)
class CheckedP27DynamicHexRow:
    """One checked tick-zero H3 row with exact binary64 source bits."""

    cell_id: int
    value_bits: tuple[int, ...]


@dataclass(frozen=True)
class CheckedP27DynamicHexArchive:
    """One byte-, schema-, session-, and identity-proved P27 archive."""

    session_id: str
    rows: tuple[CheckedP27DynamicHexRow, ...]


@dataclass(frozen=True)
class CheckedH3ReferenceRelease:
    """All decoded facts from the single pinned H3 release authority."""

    bridge: CanonicalCells
    land_mask: CanonicalCells
    p27_archives: tuple[CheckedP27DynamicHexArchive, ...]
    source_fixture: bytes
    r5: tuple[int, ...]
    r7: tuple[int, ...]


def _bridge_schema() -> pa.Schema:
    return pa.schema(
        [
            pa.field("h3_index", pa.string()),
            pa.field("county_id", pa.int64()),
            pa.field("resolution", pa.int64()),
            pa.field("coverage_pct", pa.float64()),
        ]
    )


def _land_mask_schema() -> pa.Schema:
    return pa.schema(
        [
            pa.field("h3_index", pa.string(), nullable=False),
            pa.field("county_fips", pa.string(), nullable=False),
            pa.field("land_fraction", pa.float64(), nullable=False),
        ]
    )


def _p27_schema() -> pa.Schema:
    fields = [
        pa.field("session_id", pa.string()),
        pa.field("tick", pa.int64()),
        pa.field("h3_index", pa.string()),
        pa.field("county_fips", pa.string()),
        pa.field("state_fips", pa.string()),
        pa.field("region_id", pa.string()),
    ]
    fields.extend(
        pa.field(name, pa.float64())
        for name in (
            "c",
            "v",
            "s",
            "k",
            "biocapacity_stock",
            "energy_stock",
            "raw_material_stock",
            "internet_access_pct",
            "surveillance_coupling",
        )
    )
    return pa.schema(fields)


def _artifact_specs(bridge: Path, land_mask: Path, p27_root: Path) -> tuple[ArtifactSpec, ...]:
    p27 = _p27_schema()
    return (
        ArtifactSpec(
            "bridge_county_h3.parquet",
            bridge,
            158_431,
            "e60d93a43d6c66e84f1e53ecaf633af5911bd5b48b0ef0ad6a012f6d9f5b13a9",
            SOURCE_ROWS,
            _bridge_schema(),
        ),
        ArtifactSpec(
            "h3_res7_land_mask.parquet",
            land_mask,
            295_194,
            "4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194",
            R7_ROWS,
            _land_mask_schema(),
        ),
        ArtifactSpec(
            "p27-seed-0",
            p27_root / f"{P27_SESSION_IDS[0]}/dynamic_hex_state.parquet",
            247_073,
            "116e6f57fc4c8c0d907e0c50c07afb02b9c481e484a17f4c8c3213a7167e0764",
            R7_ROWS,
            p27,
            P27_SESSION_IDS[0],
        ),
        ArtifactSpec(
            "p27-seed-1",
            p27_root / f"{P27_SESSION_IDS[1]}/dynamic_hex_state.parquet",
            246_983,
            "3a11623705824e3d906caeee65fe2bc588a9017aa6f4aeee27b646437942a746",
            R7_ROWS,
            p27,
            P27_SESSION_IDS[1],
        ),
        ArtifactSpec(
            "p27-seed-2",
            p27_root / f"{P27_SESSION_IDS[2]}/dynamic_hex_state.parquet",
            247_061,
            "6029aa7dbd618643e4502c99ccd71be1b5cd032298229c74def432ce5d68cd48",
            R7_ROWS,
            p27,
            P27_SESSION_IDS[2],
        ),
    )


def _verified_bytes(path: Path, expected_size: int, expected_sha: str) -> bytes:
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    except OSError as error:
        raise VerificationError(
            f"cannot open regular non-symlink artifact {path}: {error}"
        ) from error
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise VerificationError(f"artifact must be a regular file: {path}")
        if metadata.st_size != expected_size:
            raise VerificationError(
                f"artifact byte count drift for {path}: {metadata.st_size} != {expected_size}"
            )
        with os.fdopen(descriptor, "rb", closefd=True) as source:
            descriptor = -1
            payload = source.read(expected_size + 1)
    except OSError as error:
        raise VerificationError(f"cannot read {path}: {error}") from error
    finally:
        if descriptor >= 0:
            os.close(descriptor)
    if len(payload) != expected_size:
        raise VerificationError(f"artifact changed while being read: {path}")
    actual_sha = hashlib.sha256(payload).hexdigest()
    if actual_sha != expected_sha:
        raise VerificationError(f"artifact sha256 drift for {path}: {actual_sha} != {expected_sha}")
    return payload


def _verify_all_parquet_bytes(
    specs: tuple[ArtifactSpec, ...],
) -> tuple[PinnedArtifact, ...]:
    if len(specs) != MAX_PARQUET_ARTIFACTS:
        raise VerificationError(f"expected {MAX_PARQUET_ARTIFACTS} pinned Parquet files")
    verified: list[PinnedArtifact] = []
    for spec in specs[:MAX_PARQUET_ARTIFACTS]:
        verified.append(
            PinnedArtifact(spec, _verified_bytes(spec.path, spec.byte_count, spec.sha256))
        )
    return tuple(verified)


def _open_pinned_parquet(artifact: PinnedArtifact) -> pq.ParquetFile:
    spec = artifact.spec
    try:
        parquet = pq.ParquetFile(pa.BufferReader(artifact.payload))
    except (OSError, pa.ArrowException) as error:
        raise VerificationError(f"cannot decode pinned {spec.name}: {error}") from error
    if parquet.num_row_groups != 1:
        raise VerificationError(f"{spec.name} must contain exactly one row group")
    if parquet.metadata.num_rows != spec.row_count:
        raise VerificationError(
            f"{spec.name} row count drift: {parquet.metadata.num_rows} != {spec.row_count}"
        )
    if not parquet.schema_arrow.equals(spec.schema, check_metadata=True):
        raise VerificationError(f"{spec.name} schema drift: {parquet.schema_arrow}")
    return parquet


def _read_columns(artifact: PinnedArtifact, columns: list[str]) -> pa.Table:
    parquet = _open_pinned_parquet(artifact)
    try:
        return parquet.read_row_group(0, columns=columns)
    except (OSError, pa.ArrowException) as error:
        raise VerificationError(
            f"cannot read columns from pinned {artifact.spec.name}: {error}"
        ) from error


def _canonical_cells(values: list[object], expected_rows: int) -> CanonicalCells:
    if expected_rows > MAX_SOURCE_ROWS or len(values) != expected_rows:
        raise VerificationError(f"H3 column row count drift: {len(values)} != {expected_rows}")
    raw_rows: list[int] = []
    resolutions: list[int] = []
    for index in range(expected_rows):
        value = values[index]
        if not isinstance(value, str) or not h3.is_valid_cell(value):
            raise VerificationError(f"invalid H3 cell at row {index}: {value!r}")
        raw = h3.str_to_int(value)
        if h3.int_to_str(raw) != value:
            raise VerificationError(f"noncanonical H3 text at row {index}: {value!r}")
        raw_rows.append(raw)
        resolutions.append(h3.get_resolution(value))
    identities = frozenset(raw_rows)
    if len(identities) != expected_rows:
        raise VerificationError("H3 source contains a duplicate identity")
    ordered = tuple(sorted(zip(resolutions, raw_rows, strict=True)))
    return CanonicalCells(tuple(raw_rows), tuple(resolutions), ordered, identities)


def _read_bridge(artifact: PinnedArtifact) -> CanonicalCells:
    table = _read_columns(artifact, ["h3_index", "resolution"])
    cells = _canonical_cells(table["h3_index"].to_pylist(), SOURCE_ROWS)
    declared = table["resolution"].to_pylist()
    for index in range(SOURCE_ROWS):
        if declared[index] != cells.resolutions[index]:
            raise VerificationError(f"bridge resolution drift at row {index}")
        if cells.resolutions[index] not in (5, 7):
            raise VerificationError(f"bridge contains unsupported resolution at row {index}")
    return cells


def _read_r7(artifact: PinnedArtifact, *, verify_session: bool) -> CanonicalCells:
    spec = artifact.spec
    columns = ["h3_index", "session_id", "tick"] if verify_session else ["h3_index"]
    table = _read_columns(artifact, columns)
    cells = _canonical_cells(table["h3_index"].to_pylist(), R7_ROWS)
    for index in range(R7_ROWS):
        if cells.resolutions[index] != 7:
            raise VerificationError(f"{spec.name} contains non-r7 identity at row {index}")
        if verify_session and table["tick"][index].as_py() != 0:
            raise VerificationError(f"{spec.name} contains a nonzero tick at row {index}")
        if verify_session and table["session_id"][index].as_py() != spec.session_id:
            raise VerificationError(f"{spec.name} contains an unexpected session at row {index}")
    return cells


def _read_p27_dynamic_hex_archive(
    artifact: PinnedArtifact,
    expected_h3_cells: frozenset[int],
) -> CheckedP27DynamicHexArchive:
    """Decode one already hash-proved P27 archive through the canonical schema."""

    spec = artifact.spec
    if spec.session_id is None:
        raise VerificationError(f"{spec.name} lacks its pinned session identity")
    columns = [
        "session_id",
        "tick",
        "h3_index",
        "county_fips",
        "state_fips",
        "region_id",
        "c",
        "v",
        "s",
        "k",
        "biocapacity_stock",
        "energy_stock",
        "raw_material_stock",
        "internet_access_pct",
        "surveillance_coupling",
    ]
    table = _read_columns(artifact, columns)
    rows: list[CheckedP27DynamicHexRow] = []
    identities: set[int] = set()
    for index in range(spec.row_count):
        session_id = table["session_id"][index].as_py()
        tick = table["tick"][index].as_py()
        if session_id != spec.session_id:
            raise VerificationError(f"{spec.name} session drift at row {index}")
        if tick != 0:
            raise VerificationError(f"{spec.name} tick drift at row {index}")
        if any(
            table[field][index].as_py() is not None
            for field in ("county_fips", "state_fips", "region_id")
        ):
            raise VerificationError(f"{spec.name} geography must be null at row {index}")

        text = table["h3_index"][index].as_py()
        if not isinstance(text, str) or not h3.is_valid_cell(text):
            raise VerificationError(f"{spec.name} invalid H3 cell at row {index}")
        cell_id = h3.str_to_int(text)
        if h3.int_to_str(cell_id) != text or h3.get_resolution(text) != 7:
            raise VerificationError(f"{spec.name} noncanonical R7 H3 cell at row {index}")
        if cell_id in identities:
            raise VerificationError(f"{spec.name} duplicate H3 cell at row {index}")
        identities.add(cell_id)

        value_bits: list[int] = []
        for field in columns[6:]:
            value = table[field][index].as_py()
            if not isinstance(value, float) or not math.isfinite(value):
                raise VerificationError(f"{spec.name} {field} must be finite at row {index}")
            bits = struct.unpack(">Q", struct.pack(">d", value))[0]
            if bits == 0x8000_0000_0000_0000:
                raise VerificationError(f"{spec.name} {field} negative zero at row {index}")
            if field in columns[6:13] and value < 0.0:
                raise VerificationError(f"{spec.name} {field} must be nonnegative at row {index}")
            if field in columns[13:] and not 0.0 <= value <= 1.0:
                raise VerificationError(
                    f"{spec.name} {field} must be in the unit interval at row {index}"
                )
            value_bits.append(bits)
        rows.append(CheckedP27DynamicHexRow(cell_id, tuple(value_bits)))

    if frozenset(identities) != expected_h3_cells:
        raise VerificationError(f"{spec.name} H3 membership differs from the R7 authority")
    return CheckedP27DynamicHexArchive(spec.session_id, tuple(rows))


def _selected(ordered: tuple[tuple[int, int], ...], resolution: int) -> tuple[int, ...]:
    selected: list[int] = []
    for cell_resolution, raw in ordered[:MAX_SOURCE_ROWS]:
        if cell_resolution == resolution:
            selected.append(raw)
    return tuple(selected)


def _framed_source(raw_cells: tuple[int, ...]) -> bytes:
    if len(raw_cells) > MAX_SOURCE_ROWS:
        raise VerificationError("canonical source frame exceeds its fixed row ceiling")
    payload = bytearray(SOURCE_DOMAIN)
    payload.extend(len(raw_cells).to_bytes(8, byteorder="big", signed=False))
    for raw in raw_cells[:MAX_SOURCE_ROWS]:
        payload.extend(raw.to_bytes(8, byteorder="big", signed=False))
    return bytes(payload)


def _digest(raw_cells: tuple[int, ...]) -> str:
    return hashlib.sha256(_framed_source(raw_cells)).hexdigest()


def _load_checked_release(
    bridge_path: Path,
    land_mask_path: Path,
    p27_archive_root: Path,
    source_fixture_path: Path,
) -> CheckedH3ReferenceRelease:
    specs = _artifact_specs(bridge_path, land_mask_path, p27_archive_root)
    artifacts = _verify_all_parquet_bytes(specs)
    fixture = _verified_bytes(source_fixture_path, SOURCE_FIXTURE_BYTES, SOURCE_DIGEST)

    bridge = _read_bridge(artifacts[0])
    land_mask = _read_r7(artifacts[1], verify_session=False)
    p27_archives: list[CheckedP27DynamicHexArchive] = []
    for artifact in artifacts[2 : 2 + MAX_P27_ARCHIVES]:
        p27_archives.append(_read_p27_dynamic_hex_archive(artifact, land_mask.identity_set))

    r5 = _selected(bridge.ordered, 5)
    r7 = _selected(bridge.ordered, 7)
    if (len(r5), len(r7)) != (R5_ROWS, R7_ROWS):
        raise VerificationError(f"bridge resolution counts drift: r5={len(r5)} r7={len(r7)}")
    actual_digests = (_digest(tuple(raw for _, raw in bridge.ordered)), _digest(r5), _digest(r7))
    if actual_digests != (SOURCE_DIGEST, R5_DIGEST, R7_DIGEST):
        raise VerificationError(f"canonical identity digest drift: {actual_digests}")
    if fixture != _framed_source(tuple(raw for _, raw in bridge.ordered)):
        raise VerificationError("checked-in source fixture differs from pinned bridge identities")
    if bridge.identity_set.intersection(land_mask.identity_set) != land_mask.identity_set:
        raise VerificationError("land mask contains an identity absent from the bridge")
    if frozenset(r7) != land_mask.identity_set:
        raise VerificationError("bridge r7 set differs from the Phase 0D land mask")
    return CheckedH3ReferenceRelease(
        bridge,
        land_mask,
        tuple(p27_archives),
        fixture,
        r5,
        r7,
    )


def load_checked_p27_dynamic_hex_sources(
    *,
    bridge: Path,
    land_mask: Path,
    p27_archive_root: Path,
    source_fixture: Path,
) -> tuple[CheckedP27DynamicHexArchive, ...]:
    """Return all three P27 tick-zero sources after the full pinned release proof."""

    return _load_checked_release(
        bridge,
        land_mask,
        p27_archive_root,
        source_fixture,
    ).p27_archives


def verify(args: argparse.Namespace) -> str:
    _load_checked_release(
        args.bridge,
        args.land_mask,
        args.p27_archive_root,
        args.source_fixture,
    )
    return (
        "PER62_H3_EQUIVALENCE parquet_artifacts=5 source=48764 r5=3192 r7=45572 "
        f"source_digest={SOURCE_DIGEST} r5_digest={R5_DIGEST} r7_digest={R7_DIGEST} "
        "p27_archives=3 fixture_match=yes"
    )


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--land-mask", type=Path, required=True)
    parser.add_argument("--p27-archive-root", type=Path, required=True)
    parser.add_argument("--source-fixture", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    try:
        print(verify(_parse_args()))
    except VerificationError as error:
        print(f"verify_h3_reference_release: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
