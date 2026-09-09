#!/usr/bin/env python3
"""Build the checked Michigan Dynamic-Hex Foundation V1 fixture."""

from __future__ import annotations

import argparse
import hashlib
import math
import os
import struct
import sys
import tempfile
from collections.abc import Sequence
from pathlib import Path
from typing import Final

import h3
from tools.verify_h3_reference_release import (
    P27_SESSION_IDS,
    R7_DIGEST,
    R7_ROWS,
    SOURCE_DOMAIN,
    CheckedP27DynamicHexArchive,
    VerificationError,
    load_checked_p27_dynamic_hex_sources,
)

FOUNDATION_DOMAIN: Final = b"babylon.michigan-dynamic-hex-foundation.v1\0"
FOUNDATION_LAYOUT: Final = 1
R8_CHILD_PARENT_DOMAIN: Final = b"babylon.h3.reference-r8-child-parent.v1\0"
REFERENCE_BUNDLE_DOMAIN: Final = b"babylon.h3.reference-bundle-composite.v1\0"
BASE_REFERENCE_COHORT_DIGEST: Final = (
    "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
)
R8_CHILD_ROWS: Final = 319_004
R8_SECTION_DIGEST: Final = "b5ebf405140f6f79ddbc44fa1005b195bed0bc28e0eacf2d8e1697cd9c839491"
REFERENCE_BUNDLE_DIGEST: Final = "84bbffa9b2388aa168c065e710a61313fbd46522d2022b628f0919ecffec9831"
GOVERNED_ARTIFACT_SHA256: Final = "81ee8f8abbee6727655d52c6d56a6f2967af9dfdf01da53dd593da8339d650a4"
EXPECTED_ARCHIVES: Final = 3
EXPECTED_FIXTURE_PARTS: Final = 9
MAX_FIXTURE_PART_BYTES: Final = 1_000_000
VALUE_LANES: Final = 9


class MichiganDynamicHexFoundationRefusal(RuntimeError):
    """A checked source or requested artifact violates the closed foundation law."""


def _digest_bytes(value: str, field: str) -> bytes:
    if len(value) != 64 or value.lower() != value:
        raise MichiganDynamicHexFoundationRefusal(f"{field} must be lowercase SHA-256")
    try:
        decoded = bytes.fromhex(value)
    except ValueError as error:
        raise MichiganDynamicHexFoundationRefusal(f"{field} must be lowercase SHA-256") from error
    if len(decoded) != 32:
        raise MichiganDynamicHexFoundationRefusal(f"{field} must be lowercase SHA-256")
    return decoded


def _source_digest(cells: Sequence[int]) -> str:
    framed = bytearray(SOURCE_DOMAIN)
    framed.extend(len(cells).to_bytes(8, byteorder="big", signed=False))
    for cell in cells:
        framed.extend(cell.to_bytes(8, byteorder="big", signed=False))
    return hashlib.sha256(framed).hexdigest()


def _r8_child_parent_rows(parents: Sequence[int]) -> tuple[tuple[int, int], ...]:
    """Derive the complete canonical immediate-R8 child section from R7 authority."""

    pairs: list[tuple[int, int]] = []
    for parent in parents:
        parent_text = h3.int_to_str(parent)
        if h3.get_resolution(parent_text) != 7:
            raise MichiganDynamicHexFoundationRefusal(
                f"R8 section parent {parent_text} is not resolution 7"
            )
        children = tuple(
            sorted(h3.str_to_int(child) for child in h3.cell_to_children(parent_text, 8))
        )
        if len(children) != 7:
            raise MichiganDynamicHexFoundationRefusal(
                f"R7 parent {parent_text} must have exactly seven immediate R8 children"
            )
        for child in children:
            child_text = h3.int_to_str(child)
            if h3.get_resolution(child_text) != 8:
                raise MichiganDynamicHexFoundationRefusal(
                    f"derived child {child_text} is not resolution 8"
                )
            actual_parent = h3.str_to_int(h3.cell_to_parent(child_text, 7))
            if actual_parent != parent:
                raise MichiganDynamicHexFoundationRefusal(
                    f"derived child {child_text} does not map to its R7 parent"
                )
            pairs.append((child, parent))
    pairs.sort()
    for left, right in zip(pairs, pairs[1:], strict=False):
        if left[0] >= right[0]:
            raise MichiganDynamicHexFoundationRefusal(
                "R8 child identities must be strictly numeric-H3 ordered and unique"
            )
    return tuple(pairs)


def _r8_child_parent_section(rows: Sequence[tuple[int, int]]) -> bytes:
    output = bytearray(R8_CHILD_PARENT_DOMAIN)
    output.extend(len(rows).to_bytes(8, byteorder="big", signed=False))
    for child, parent in rows:
        output.extend(child.to_bytes(8, byteorder="big", signed=False))
        output.extend(parent.to_bytes(8, byteorder="big", signed=False))
    return bytes(output)


def _composite_reference_bundle_digest(
    *,
    base_cohort_digest: str,
    r8_section_digest: str,
) -> str:
    framed = bytearray(REFERENCE_BUNDLE_DOMAIN)
    framed.extend(_digest_bytes(base_cohort_digest, "base_reference_cohort_digest"))
    framed.extend(_digest_bytes(r8_section_digest, "r8_child_parent_digest"))
    return hashlib.sha256(framed).hexdigest()


def _ordered_rows(
    archive: CheckedP27DynamicHexArchive,
) -> tuple[tuple[int, tuple[int, ...]], ...]:
    ordered = tuple(sorted((row.cell_id, row.value_bits) for row in archive.rows))
    previous: int | None = None
    for cell, value_bits in ordered:
        if previous is not None and cell <= previous:
            raise MichiganDynamicHexFoundationRefusal(
                f"archive {archive.session_id} contains duplicate or unordered H3 identity"
            )
        previous = cell
        if len(value_bits) != VALUE_LANES:
            raise MichiganDynamicHexFoundationRefusal(
                f"archive {archive.session_id} row has {len(value_bits)} value lanes"
            )
        for bits in value_bits:
            value = struct.unpack(">d", bits.to_bytes(8, "big"))[0]
            if not math.isfinite(value):
                raise MichiganDynamicHexFoundationRefusal(
                    f"archive {archive.session_id} contains a non-finite value"
                )
            if bits == 0x8000_0000_0000_0000:
                raise MichiganDynamicHexFoundationRefusal(
                    f"archive {archive.session_id} contains negative zero"
                )
        for lane, bits in enumerate(value_bits[:7]):
            if struct.unpack(">d", bits.to_bytes(8, "big"))[0] < 0.0:
                raise MichiganDynamicHexFoundationRefusal(
                    f"archive {archive.session_id} lane {lane} must be nonnegative"
                )
        for lane, bits in enumerate(value_bits[7:], start=7):
            value = struct.unpack(">d", bits.to_bytes(8, "big"))[0]
            if not 0.0 <= value <= 1.0:
                raise MichiganDynamicHexFoundationRefusal(
                    f"archive {archive.session_id} lane {lane} must be in the unit interval"
                )
    return ordered


def _compose_michigan_dynamic_hex_foundation_v1(
    archives: Sequence[CheckedP27DynamicHexArchive],
    *,
    expected_row_count: int,
    source_r7_digest: str,
    base_reference_cohort_digest: str,
    expected_r8_row_count: int | None = None,
    expected_r8_section_digest: str | None = None,
    expected_reference_bundle_digest: str | None = None,
) -> bytes:
    """Compose checked sources for bounded synthetic and governed tests."""

    if len(archives) != EXPECTED_ARCHIVES:
        raise MichiganDynamicHexFoundationRefusal(
            f"archive count must be exactly {EXPECTED_ARCHIVES}"
        )
    actual_sessions = frozenset(archive.session_id for archive in archives)
    if len(actual_sessions) != EXPECTED_ARCHIVES or actual_sessions != frozenset(P27_SESSION_IDS):
        raise MichiganDynamicHexFoundationRefusal(
            "archive session identities must equal the three governed P27 sessions"
        )
    ordered = tuple(_ordered_rows(archive) for archive in archives)
    authority = ordered[0]
    if len(authority) != expected_row_count:
        raise MichiganDynamicHexFoundationRefusal(f"row count must be exactly {expected_row_count}")
    for archive_index, candidate in enumerate(ordered[1:], start=1):
        if candidate != authority:
            raise MichiganDynamicHexFoundationRefusal(
                f"three-way P27 consensus differs in archive {archive_index}"
            )

    cells = tuple(cell for cell, _ in authority)
    actual_source_digest = _source_digest(cells)
    if source_r7_digest != actual_source_digest:
        raise MichiganDynamicHexFoundationRefusal(
            f"source_r7_digest mismatch: {source_r7_digest} != {actual_source_digest}"
        )
    if base_reference_cohort_digest != BASE_REFERENCE_COHORT_DIGEST:
        raise MichiganDynamicHexFoundationRefusal(
            "base_reference_cohort_digest does not match the governed reference cohort"
        )

    r8_rows = _r8_child_parent_rows(cells)
    if expected_r8_row_count is not None and len(r8_rows) != expected_r8_row_count:
        raise MichiganDynamicHexFoundationRefusal(
            f"R8 child row count must be exactly {expected_r8_row_count}"
        )
    r8_section = _r8_child_parent_section(r8_rows)
    r8_section_digest = hashlib.sha256(r8_section).hexdigest()
    if expected_r8_section_digest is not None and r8_section_digest != expected_r8_section_digest:
        raise MichiganDynamicHexFoundationRefusal(
            "R8 child-parent section digest does not match the governed cohort"
        )
    reference_bundle_digest = _composite_reference_bundle_digest(
        base_cohort_digest=base_reference_cohort_digest,
        r8_section_digest=r8_section_digest,
    )
    if (
        expected_reference_bundle_digest is not None
        and reference_bundle_digest != expected_reference_bundle_digest
    ):
        raise MichiganDynamicHexFoundationRefusal(
            "composite reference_bundle_digest does not match the governed cohort"
        )

    output = bytearray(FOUNDATION_DOMAIN)
    output.extend(FOUNDATION_LAYOUT.to_bytes(4, byteorder="big", signed=False))
    output.extend(_digest_bytes(source_r7_digest, "source_r7_digest"))
    output.extend(_digest_bytes(base_reference_cohort_digest, "base_reference_cohort_digest"))
    output.extend(_digest_bytes(r8_section_digest, "r8_child_parent_digest"))
    output.extend(_digest_bytes(reference_bundle_digest, "reference_bundle_digest"))
    output.extend(len(authority).to_bytes(8, byteorder="big", signed=False))
    for cell, value_bits in authority:
        output.extend(cell.to_bytes(8, byteorder="big", signed=False))
        for bits in value_bits:
            output.extend(bits.to_bytes(8, byteorder="big", signed=False))
    output.extend(r8_section)
    return bytes(output)


def build_michigan_dynamic_hex_foundation_v1(
    archives: Sequence[CheckedP27DynamicHexArchive],
) -> bytes:
    """Build only the exact governed 45,572-row Michigan foundation."""

    fixture = _compose_michigan_dynamic_hex_foundation_v1(
        archives,
        expected_row_count=R7_ROWS,
        source_r7_digest=R7_DIGEST,
        base_reference_cohort_digest=BASE_REFERENCE_COHORT_DIGEST,
        expected_r8_row_count=R8_CHILD_ROWS,
        expected_r8_section_digest=R8_SECTION_DIGEST,
        expected_reference_bundle_digest=REFERENCE_BUNDLE_DIGEST,
    )
    actual_sha256 = hashlib.sha256(fixture).hexdigest()
    if actual_sha256 != GOVERNED_ARTIFACT_SHA256:
        raise MichiganDynamicHexFoundationRefusal(
            "artifact SHA-256 does not match the governed Michigan foundation: "
            f"{actual_sha256} != {GOVERNED_ARTIFACT_SHA256}"
        )
    return fixture


def split_fixture_parts(payload: bytes) -> tuple[bytes, ...]:
    """Split one canonical artifact into bounded plain-Git fixture parts."""

    if not payload:
        raise MichiganDynamicHexFoundationRefusal("foundation artifact must not be empty")
    return tuple(
        payload[offset : offset + MAX_FIXTURE_PART_BYTES]
        for offset in range(0, len(payload), MAX_FIXTURE_PART_BYTES)
    )


def _write_fixture_parts_atomic(output_prefix: Path, parts: tuple[bytes, ...]) -> tuple[Path, ...]:
    if len(parts) != EXPECTED_FIXTURE_PARTS:
        raise MichiganDynamicHexFoundationRefusal(
            f"fixture part count must be exactly {EXPECTED_FIXTURE_PARTS}"
        )
    output_prefix.parent.mkdir(parents=True, exist_ok=True)
    outputs = tuple(
        output_prefix.with_name(f"{output_prefix.name}.part-{index:02}.bin")
        for index in range(EXPECTED_FIXTURE_PARTS)
    )
    staged: list[tuple[Path, Path]] = []
    try:
        for output, part in zip(outputs, parts, strict=True):
            descriptor, temporary_name = tempfile.mkstemp(
                prefix=f".{output.name}.",
                dir=output.parent,
            )
            temporary = Path(temporary_name)
            try:
                with os.fdopen(descriptor, "wb", closefd=True) as destination:
                    destination.write(part)
                    destination.flush()
                    os.fsync(destination.fileno())
            except BaseException:
                temporary.unlink(missing_ok=True)
                raise
            staged.append((temporary, output))
        for temporary, output in staged:
            os.replace(temporary, output)
        directory = os.open(output_prefix.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        for temporary, _ in staged:
            temporary.unlink(missing_ok=True)
    return outputs


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bridge", type=Path, required=True)
    parser.add_argument("--land-mask", type=Path, required=True)
    parser.add_argument("--p27-archive-root", type=Path, required=True)
    parser.add_argument("--source-fixture", type=Path, required=True)
    parser.add_argument(
        "--output-prefix",
        type=Path,
        default=Path(
            "rust/crates/babylon-persistence/src/fixtures/michigan_dynamic_hex_foundation_v1"
        ),
    )
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    try:
        archives = load_checked_p27_dynamic_hex_sources(
            bridge=args.bridge,
            land_mask=args.land_mask,
            p27_archive_root=args.p27_archive_root,
            source_fixture=args.source_fixture,
        )
        fixture = build_michigan_dynamic_hex_foundation_v1(archives)
        parts = split_fixture_parts(fixture)
        outputs = _write_fixture_parts_atomic(args.output_prefix, parts)
    except (MichiganDynamicHexFoundationRefusal, VerificationError, OSError) as error:
        print(f"build_michigan_dynamic_hex_foundation_v1: {error}", file=sys.stderr)
        return 2
    print(
        "MICHIGAN_DYNAMIC_HEX_FOUNDATION_V1 "
        f"rows={len(archives[0].rows)} bytes={len(fixture)} "
        f"sha256={hashlib.sha256(fixture).hexdigest()} "
        f"part_bytes={','.join(str(len(part)) for part in parts)} "
        f"outputs={','.join(str(output) for output in outputs)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
