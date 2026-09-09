"""Contracts for the Michigan Dynamic-Hex Foundation V1 fixture."""

from __future__ import annotations

import hashlib
import inspect
import math
import struct
from argparse import Namespace
from dataclasses import replace
from pathlib import Path
from typing import Any

import h3
import pyarrow as pa  # type: ignore[import-untyped]
import pyarrow.parquet as pq  # type: ignore[import-untyped]
import pytest
import tools.build_michigan_dynamic_hex_foundation_v1 as foundation_builder
from tools.build_michigan_dynamic_hex_foundation_v1 import (
    FOUNDATION_DOMAIN,
    FOUNDATION_LAYOUT,
    MAX_FIXTURE_PART_BYTES,
    R8_CHILD_PARENT_DOMAIN,
    REFERENCE_BUNDLE_DOMAIN,
    MichiganDynamicHexFoundationRefusal,
    _compose_michigan_dynamic_hex_foundation_v1,
    _r8_child_parent_rows,
    build_michigan_dynamic_hex_foundation_v1,
    split_fixture_parts,
)
from tools.verify_h3_reference_release import (
    ArtifactSpec,
    PinnedArtifact,
    VerificationError,
    _p27_schema,
    _read_p27_dynamic_hex_archive,
)

SOURCE_DOMAIN = b"babylon.h3.reference-source.v1\0"
BASE_REFERENCE_COHORT_SHA256 = "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
H3_TEXT = ("872664800ffffff", "872664801ffffff", "872664802ffffff")
VALUE_COLUMNS = (
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
SESSIONS = (
    "5ef6b154-0fe8-44a1-ac12-58d40e69a75c",
    "ccf3082d-f94d-41d8-9b2a-27d488585507",
    "7f6599f0-2c56-4b91-a955-1bb50e0d8987",
)


def _schema() -> pa.Schema:
    return pa.schema(
        [
            pa.field("session_id", pa.string()),
            pa.field("tick", pa.int64()),
            pa.field("h3_index", pa.string()),
            pa.field("county_fips", pa.string()),
            pa.field("state_fips", pa.string()),
            pa.field("region_id", pa.string()),
            *(pa.field(name, pa.float64()) for name in VALUE_COLUMNS),
        ]
    )


def _rows() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for row_index, cell in enumerate(H3_TEXT):
        row: dict[str, Any] = {
            "session_id": "",
            "tick": 0,
            "h3_index": cell,
            "county_fips": None,
            "state_fips": None,
            "region_id": None,
        }
        row.update(
            {
                name: float(row_index * len(VALUE_COLUMNS) + column_index + 1)
                for column_index, name in enumerate(VALUE_COLUMNS)
            }
        )
        row["internet_access_pct"] = 0.25 + row_index * 0.25
        row["surveillance_coupling"] = 0.75 - row_index * 0.25
        rows.append(row)
    return rows


def _write_archive(
    path: Path,
    session_id: str,
    order: tuple[int, ...],
    *,
    mutate: tuple[int, str, Any] | None = None,
    omit_column: str | None = None,
) -> None:
    rows = _rows()
    for row in rows:
        row["session_id"] = session_id
    if mutate is not None:
        row_index, field, value = mutate
        rows[row_index][field] = value
    schema = _schema()
    if omit_column is not None:
        schema = pa.schema(field for field in schema if field.name != omit_column)
        for row in rows:
            row.pop(omit_column)
    table = pa.Table.from_pylist([rows[index] for index in order], schema=schema)
    pq.write_table(table, path, row_group_size=len(rows))


def _load(path: Path, session_id: str):  # type: ignore[no-untyped-def]
    payload = path.read_bytes()
    artifact = PinnedArtifact(
        ArtifactSpec(
            "synthetic-p27",
            path,
            len(payload),
            hashlib.sha256(payload).hexdigest(),
            len(H3_TEXT),
            _p27_schema(),
            session_id,
        ),
        payload,
    )
    return _read_p27_dynamic_hex_archive(
        artifact,
        frozenset(h3.str_to_int(cell) for cell in H3_TEXT),
    )


def _source_digest(cells: tuple[int, ...]) -> str:
    framed = bytearray(SOURCE_DOMAIN)
    framed.extend(len(cells).to_bytes(8, "big"))
    for cell in cells:
        framed.extend(cell.to_bytes(8, "big"))
    return hashlib.sha256(framed).hexdigest()


def _checked_archives(tmp_path: Path):  # type: ignore[no-untyped-def]
    orders = ((2, 0, 1), (1, 2, 0), (0, 1, 2))
    checked = []
    for index, (session, order) in enumerate(zip(SESSIONS, orders, strict=True)):
        path = tmp_path / f"seed-{index}.parquet"
        _write_archive(path, session, order)
        checked.append(_load(path, session))
    return tuple(checked)


def _expected_fixture() -> bytes:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    rows = {h3.str_to_int(row["h3_index"]): row for row in _rows()}
    r8_rows = tuple(
        sorted(
            (
                h3.str_to_int(child),
                parent,
            )
            for parent in cells
            for child in h3.cell_to_children(h3.int_to_str(parent), 8)
        )
    )
    r8_section = bytearray(R8_CHILD_PARENT_DOMAIN)
    r8_section.extend(len(r8_rows).to_bytes(8, "big"))
    for child, parent in r8_rows:
        r8_section.extend(child.to_bytes(8, "big"))
        r8_section.extend(parent.to_bytes(8, "big"))
    r8_section_digest = hashlib.sha256(r8_section).digest()
    reference_bundle = bytearray(REFERENCE_BUNDLE_DOMAIN)
    reference_bundle.extend(bytes.fromhex(BASE_REFERENCE_COHORT_SHA256))
    reference_bundle.extend(r8_section_digest)
    reference_bundle_digest = hashlib.sha256(reference_bundle).digest()

    expected = bytearray(FOUNDATION_DOMAIN)
    expected.extend(FOUNDATION_LAYOUT.to_bytes(4, "big"))
    expected.extend(bytes.fromhex(_source_digest(cells)))
    expected.extend(bytes.fromhex(BASE_REFERENCE_COHORT_SHA256))
    expected.extend(r8_section_digest)
    expected.extend(reference_bundle_digest)
    expected.extend(len(cells).to_bytes(8, "big"))
    for cell in cells:
        expected.extend(cell.to_bytes(8, "big"))
        for field in VALUE_COLUMNS:
            expected.extend(struct.pack(">d", rows[cell][field]))
    expected.extend(r8_section)
    return bytes(expected)


def test_three_way_consensus_ignores_parquet_order_and_emits_exact_layout(
    tmp_path: Path,
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))

    fixture = _compose_michigan_dynamic_hex_foundation_v1(
        _checked_archives(tmp_path),
        expected_row_count=len(cells),
        source_r7_digest=_source_digest(cells),
        base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
    )

    assert FOUNDATION_DOMAIN == b"babylon.michigan-dynamic-hex-foundation.v1\0"
    assert FOUNDATION_LAYOUT == 1
    assert fixture == _expected_fixture()


@pytest.mark.parametrize("field", VALUE_COLUMNS)
def test_consensus_refuses_one_seed_bit_drift(tmp_path: Path, field: str) -> None:
    checked = list(_checked_archives(tmp_path))
    drift = tmp_path / "drift.parquet"
    drift_value = 0.125 if field in VALUE_COLUMNS[7:] else 99.0
    _write_archive(drift, SESSIONS[2], (0, 1, 2), mutate=(1, field, drift_value))
    checked[2] = _load(drift, SESSIONS[2])
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="consensus"):
        _compose_michigan_dynamic_hex_foundation_v1(
            tuple(checked),
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


@pytest.mark.parametrize(
    ("mutate", "omit_column", "message"),
    [
        ((0, "session_id", "00000000-0000-0000-0000-000000000000"), None, "session"),
        ((0, "tick", 1), None, "tick"),
        (None, "surveillance_coupling", "schema"),
        ((0, "county_fips", "26001"), None, "geography"),
        ((0, "state_fips", "26"), None, "geography"),
        ((0, "region_id", "michigan"), None, "geography"),
        ((0, "h3_index", H3_TEXT[1]), None, "duplicate"),
        ((0, "h3_index", "not-an-h3-cell"), None, "H3"),
        ((0, "c", math.inf), None, "finite"),
        ((0, "c", -0.0), None, "negative zero"),
    ],
)
def test_checked_source_refuses_invalid_archive_authority(
    tmp_path: Path,
    mutate: tuple[int, str, Any] | None,
    omit_column: str | None,
    message: str,
) -> None:
    path = tmp_path / "invalid.parquet"
    _write_archive(path, SESSIONS[0], (0, 1, 2), mutate=mutate, omit_column=omit_column)

    with pytest.raises(VerificationError, match=message):
        _load(path, SESSIONS[0])


@pytest.mark.parametrize("field", VALUE_COLUMNS[:7])
def test_checked_source_refuses_negative_stock_and_value_lanes(
    tmp_path: Path,
    field: str,
) -> None:
    path = tmp_path / f"negative-{field}.parquet"
    _write_archive(path, SESSIONS[0], (0, 1, 2), mutate=(0, field, -1.0))

    with pytest.raises(VerificationError, match="nonnegative"):
        _load(path, SESSIONS[0])


@pytest.mark.parametrize("field", VALUE_COLUMNS[7:])
@pytest.mark.parametrize("value", [-0.25, 1.25])
def test_checked_source_refuses_values_outside_unit_interval(
    tmp_path: Path,
    field: str,
    value: float,
) -> None:
    path = tmp_path / f"unit-{field}-{value}.parquet"
    _write_archive(path, SESSIONS[0], (0, 1, 2), mutate=(0, field, value))

    with pytest.raises(VerificationError, match="unit interval"):
        _load(path, SESSIONS[0])


def test_foundation_refuses_r7_digest_mismatch(tmp_path: Path) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="source_r7_digest"):
        _compose_michigan_dynamic_hex_foundation_v1(
            _checked_archives(tmp_path),
            expected_row_count=len(cells),
            source_r7_digest="00" * 32,
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


@pytest.mark.parametrize("lane", range(7))
def test_private_composer_defends_nonnegative_lanes(
    tmp_path: Path,
    lane: int,
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    archives = list(_checked_archives(tmp_path))
    rows = list(archives[0].rows)
    bits = list(rows[0].value_bits)
    bits[lane] = struct.unpack(">Q", struct.pack(">d", -1.0))[0]
    rows[0] = replace(rows[0], value_bits=tuple(bits))
    archives[0] = replace(archives[0], rows=tuple(rows))

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="nonnegative"):
        _compose_michigan_dynamic_hex_foundation_v1(
            tuple(archives),
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


@pytest.mark.parametrize("lane", range(7, 9))
@pytest.mark.parametrize("value", [-0.25, 1.25])
def test_private_composer_defends_unit_interval_lanes(
    tmp_path: Path,
    lane: int,
    value: float,
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    archives = list(_checked_archives(tmp_path))
    rows = list(archives[0].rows)
    bits = list(rows[0].value_bits)
    bits[lane] = struct.unpack(">Q", struct.pack(">d", value))[0]
    rows[0] = replace(rows[0], value_bits=tuple(bits))
    archives[0] = replace(archives[0], rows=tuple(rows))

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="unit interval"):
        _compose_michigan_dynamic_hex_foundation_v1(
            tuple(archives),
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


@pytest.mark.parametrize("archive_count", [2, 4])
def test_foundation_requires_exactly_three_checked_archives(
    tmp_path: Path,
    archive_count: int,
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    checked = _checked_archives(tmp_path)
    archives = checked[:archive_count]
    if archive_count == 4:
        archives = (*checked, checked[0])

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="archive count"):
        _compose_michigan_dynamic_hex_foundation_v1(
            archives,
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


@pytest.mark.parametrize(
    "sessions",
    [
        (SESSIONS[0], SESSIONS[0], SESSIONS[2]),
        (SESSIONS[0], SESSIONS[1], "00000000-0000-0000-0000-000000000000"),
    ],
)
def test_foundation_requires_the_three_unique_governed_sessions(
    tmp_path: Path,
    sessions: tuple[str, str, str],
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    archives = tuple(
        replace(archive, session_id=session)
        for archive, session in zip(_checked_archives(tmp_path), sessions, strict=True)
    )

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="session"):
        _compose_michigan_dynamic_hex_foundation_v1(
            archives,
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        )


def test_foundation_refuses_base_reference_cohort_digest_mismatch(tmp_path: Path) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="base_reference_cohort_digest"):
        _compose_michigan_dynamic_hex_foundation_v1(
            _checked_archives(tmp_path),
            expected_row_count=len(cells),
            source_r7_digest=_source_digest(cells),
            base_reference_cohort_digest="00" * 32,
        )


def test_r8_section_is_the_complete_ordered_immediate_child_set() -> None:
    parents = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))

    rows = _r8_child_parent_rows(parents)

    assert len(rows) == 21
    assert tuple(child for child, _ in rows) == tuple(sorted(child for child, _ in rows))
    assert len({child for child, _ in rows}) == len(rows)
    assert (h3.str_to_int("8826648001fffff"), parents[0]) in rows
    for child, parent in rows:
        assert h3.get_resolution(h3.int_to_str(child)) == 8
        assert h3.str_to_int(h3.cell_to_parent(h3.int_to_str(child), 7)) == parent
    assert all(sum(row_parent == parent for _, row_parent in rows) == 7 for parent in parents)


def test_public_builder_owns_all_foundation_authority(tmp_path: Path) -> None:
    assert tuple(inspect.signature(build_michigan_dynamic_hex_foundation_v1).parameters) == (
        "archives",
    )
    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="row count"):
        build_michigan_dynamic_hex_foundation_v1(_checked_archives(tmp_path))


def test_public_builder_refuses_consensus_legal_value_mutation_on_artifact_sha(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    cells = tuple(sorted(h3.str_to_int(cell) for cell in H3_TEXT))
    r8_rows = _r8_child_parent_rows(cells)
    r8_digest = hashlib.sha256(foundation_builder._r8_child_parent_section(r8_rows)).hexdigest()
    reference_digest = foundation_builder._composite_reference_bundle_digest(
        base_cohort_digest=BASE_REFERENCE_COHORT_SHA256,
        r8_section_digest=r8_digest,
    )
    monkeypatch.setattr(foundation_builder, "R7_ROWS", len(cells))
    monkeypatch.setattr(foundation_builder, "R7_DIGEST", _source_digest(cells))
    monkeypatch.setattr(foundation_builder, "R8_CHILD_ROWS", len(r8_rows))
    monkeypatch.setattr(foundation_builder, "R8_SECTION_DIGEST", r8_digest)
    monkeypatch.setattr(foundation_builder, "REFERENCE_BUNDLE_DIGEST", reference_digest)
    archives = []
    for archive in _checked_archives(tmp_path):
        rows = list(archive.rows)
        row_index = next(index for index, row in enumerate(rows) if row.cell_id == cells[0])
        bits = list(rows[row_index].value_bits)
        bits[0] = struct.unpack(">Q", struct.pack(">d", 42.0))[0]
        rows[row_index] = replace(rows[row_index], value_bits=tuple(bits))
        archives.append(replace(archive, rows=tuple(rows)))

    with pytest.raises(MichiganDynamicHexFoundationRefusal, match="artifact SHA-256"):
        build_michigan_dynamic_hex_foundation_v1(tuple(archives))


def test_fixture_parts_obey_the_plain_git_one_megabyte_ceiling() -> None:
    payload = b"x" * (2 * MAX_FIXTURE_PART_BYTES + 1)

    parts = split_fixture_parts(payload)

    assert MAX_FIXTURE_PART_BYTES == 1_000_000
    assert tuple(map(len, parts)) == (1_000_000, 1_000_000, 1)
    assert b"".join(parts) == payload


def test_cli_loads_sources_only_through_the_full_checked_release(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    bridge = tmp_path / "bridge.parquet"
    land_mask = tmp_path / "land-mask.parquet"
    p27_root = tmp_path / "p27"
    source_fixture = tmp_path / "source.bin"
    output_prefix = tmp_path / "foundation"
    archives = _checked_archives(tmp_path)
    observed: dict[str, object] = {}

    monkeypatch.setattr(
        foundation_builder,
        "_parse_args",
        lambda: Namespace(
            bridge=bridge,
            land_mask=land_mask,
            p27_archive_root=p27_root,
            source_fixture=source_fixture,
            output_prefix=output_prefix,
        ),
    )

    def checked_loader(**kwargs: Path):  # type: ignore[no-untyped-def]
        observed["loader"] = kwargs
        return archives

    def checked_builder(actual_archives) -> bytes:  # type: ignore[no-untyped-def]
        observed["builder"] = actual_archives
        return b"x" * (8 * MAX_FIXTURE_PART_BYTES + 1)

    def checked_writer(prefix: Path, parts: tuple[bytes, ...]):
        observed["writer"] = (prefix, tuple(map(len, parts)))
        return tuple(prefix.with_name(f"{prefix.name}.part-{index:02}.bin") for index in range(9))

    monkeypatch.setattr(
        foundation_builder,
        "load_checked_p27_dynamic_hex_sources",
        checked_loader,
    )
    monkeypatch.setattr(
        foundation_builder,
        "build_michigan_dynamic_hex_foundation_v1",
        checked_builder,
    )
    monkeypatch.setattr(
        foundation_builder,
        "_write_fixture_parts_atomic",
        checked_writer,
    )

    assert foundation_builder.main() == 0
    assert observed["loader"] == {
        "bridge": bridge,
        "land_mask": land_mask,
        "p27_archive_root": p27_root,
        "source_fixture": source_fixture,
    }
    assert observed["builder"] == archives
    assert observed["writer"] == (
        output_prefix,
        (1_000_000,) * 8 + (1,),
    )
