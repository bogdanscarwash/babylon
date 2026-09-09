#!/usr/bin/env python3
"""Independently verify Michigan Dynamic-Hex Foundation V1 bytes and authority."""

from __future__ import annotations

import argparse
import hashlib
import math
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import h3
import yaml

DEFAULT_CONTRACT = Path("contracts/michigan_dynamic_hex_foundation_v1.yaml")


@dataclass(frozen=True, slots=True)
class FoundationFinding:
    """One stable contract refusal."""

    code: str
    detail: str


def _finding(code: str, detail: str) -> FoundationFinding:
    return FoundationFinding(code, detail)


def _mapping(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{field} must be a mapping")
    return value


def _integer(value: Any, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"{field} must be a nonnegative integer")
    return value


def _text(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field} must be nonempty text")
    return value


def _digest(value: Any, field: str) -> bytes:
    text = _text(value, field)
    if len(text) != 64 or text.lower() != text:
        raise ValueError(f"{field} must be lowercase SHA-256")
    try:
        decoded = bytes.fromhex(text)
    except ValueError as error:
        raise ValueError(f"{field} must be lowercase SHA-256") from error
    if len(decoded) != 32:
        raise ValueError(f"{field} must be lowercase SHA-256")
    return decoded


def _domain(value: Any, field: str) -> bytes:
    encoded = _text(value, field).encode("utf-8")
    if not encoded.endswith(b"\0"):
        raise ValueError(f"{field} must end in NUL")
    return encoded


def _load_contract(path: Path) -> dict[str, Any]:
    loaded = yaml.safe_load(path.read_text(encoding="utf-8"))
    return _mapping(loaded, "contract")


def _fixture_bytes(contract: dict[str, Any], root: Path) -> tuple[bytes, tuple[int, ...]]:
    artifact = _mapping(contract.get("artifact"), "artifact")
    raw_parts = artifact.get("fixture_parts")
    if not isinstance(raw_parts, list) or not raw_parts:
        raise ValueError("artifact.fixture_parts must be a nonempty list")
    parts = tuple(
        (root / _text(item, "artifact.fixture_parts item")).read_bytes() for item in raw_parts
    )
    return b"".join(parts), tuple(map(len, parts))


def _u64(bytes_: bytes) -> int:
    return int.from_bytes(bytes_, "big", signed=False)


def verify_foundation_artifact(
    contract: dict[str, Any], artifact_bytes: bytes, part_sizes: tuple[int, ...]
) -> tuple[FoundationFinding, ...]:
    """Verify supplied bytes without importing the builder or Rust constants."""

    findings: list[FoundationFinding] = []
    try:
        wire = _mapping(contract.get("wire"), "wire")
        dynamic = _mapping(contract.get("dynamic_r7"), "dynamic_r7")
        reference = _mapping(contract.get("reference_bundle"), "reference_bundle")
        r8_contract = _mapping(reference.get("r8_child_parent"), "reference_bundle.r8_child_parent")
        artifact = _mapping(contract.get("artifact"), "artifact")
        foundation_domain = _domain(
            wire.get("foundation_domain_utf8"), "wire.foundation_domain_utf8"
        )
        source_domain = _domain(
            wire.get("source_identity_domain_utf8"), "wire.source_identity_domain_utf8"
        )
        r8_domain = _domain(wire.get("r8_section_domain_utf8"), "wire.r8_section_domain_utf8")
        bundle_domain = _domain(
            wire.get("reference_bundle_domain_utf8"), "wire.reference_bundle_domain_utf8"
        )
        layout = _integer(wire.get("layout"), "wire.layout")
        dynamic_count = _integer(dynamic.get("row_count"), "dynamic_r7.row_count")
        dynamic_row_bytes = _integer(dynamic.get("row_bytes"), "dynamic_r7.row_bytes")
        expected_source_digest = _digest(dynamic.get("source_digest"), "dynamic_r7.source_digest")
        expected_base_digest = _digest(
            reference.get("base_cohort_digest"), "reference_bundle.base_cohort_digest"
        )
        r8_count = _integer(
            r8_contract.get("row_count"), "reference_bundle.r8_child_parent.row_count"
        )
        r8_row_bytes = _integer(
            r8_contract.get("row_bytes"), "reference_bundle.r8_child_parent.row_bytes"
        )
        expected_r8_digest = _digest(
            r8_contract.get("section_digest"), "reference_bundle.r8_child_parent.section_digest"
        )
        expected_bundle_digest = _digest(
            reference.get("composite_digest"), "reference_bundle.composite_digest"
        )
        expected_bytes = _integer(artifact.get("byte_count"), "artifact.byte_count")
        expected_artifact_digest = _digest(artifact.get("sha256"), "artifact.sha256")
        max_part_bytes = _integer(artifact.get("max_part_bytes"), "artifact.max_part_bytes")
        expected_part_sizes_raw = artifact.get("part_sizes")
        if not isinstance(expected_part_sizes_raw, list):
            raise ValueError("artifact.part_sizes must be a list")
        expected_part_sizes = tuple(
            _integer(size, "artifact.part_sizes item") for size in expected_part_sizes_raw
        )
        audited = _mapping(contract.get("audited_identities"), "audited_identities")
        audited_r7_raw = audited.get("r7_cells")
        if not isinstance(audited_r7_raw, list):
            raise ValueError("audited_identities.r7_cells must be a list")
        audited_r7_values = tuple(
            h3.str_to_int(_text(cell, "audited R7 cell")) for cell in audited_r7_raw
        )
        audited_pair = (
            h3.str_to_int(_text(audited.get("r8_child"), "audited r8_child")),
            h3.str_to_int(_text(audited.get("r8_parent"), "audited r8_parent")),
        )
    except (KeyError, TypeError, ValueError, OverflowError, h3.H3BaseException) as error:
        return (_finding("invalid_contract", str(error)),)

    if layout != 1 or dynamic_row_bytes != 80 or r8_row_bytes != 16:
        findings.append(_finding("invalid_layout", "layout and row widths must equal V1"))
    if part_sizes != expected_part_sizes or any(size > max_part_bytes for size in part_sizes):
        findings.append(_finding("fixture_parts", "fixture part sizes or ceiling differ"))
    if len(artifact_bytes) != expected_bytes:
        findings.append(
            _finding("artifact_bytes", f"expected {expected_bytes}, got {len(artifact_bytes)}")
        )
        return tuple(findings)
    if hashlib.sha256(artifact_bytes).digest() != expected_artifact_digest:
        findings.append(_finding("artifact_digest", "whole artifact SHA-256 differs"))

    offset = 0

    def take(count: int) -> bytes:
        nonlocal offset
        end = offset + count
        if end > len(artifact_bytes):
            raise ValueError("artifact is truncated")
        value = artifact_bytes[offset:end]
        offset = end
        return value

    try:
        if take(len(foundation_domain)) != foundation_domain:
            findings.append(_finding("foundation_domain", "foundation domain differs"))
        if int.from_bytes(take(4), "big") != layout:
            findings.append(_finding("foundation_layout", "foundation layout differs"))
        encoded_source_digest = take(32)
        encoded_base_digest = take(32)
        encoded_r8_digest = take(32)
        encoded_bundle_digest = take(32)
        encoded_dynamic_count = _u64(take(8))
        if encoded_dynamic_count != dynamic_count:
            findings.append(_finding("dynamic_count", "dynamic R7 count differs"))
            return tuple(findings)

        dynamic_rows: list[tuple[int, tuple[int, ...]]] = []
        for _ in range(dynamic_count):
            cell = _u64(take(8))
            value_bits = tuple(_u64(take(8)) for _ in range(9))
            dynamic_rows.append((cell, value_bits))

        r8_start = offset
        if take(len(r8_domain)) != r8_domain:
            findings.append(_finding("r8_domain", "R8 section domain differs"))
        encoded_r8_count = _u64(take(8))
        if encoded_r8_count != r8_count:
            findings.append(_finding("r8_count", "R8 child-parent count differs"))
            return tuple(findings)
        r8_rows = [(_u64(take(8)), _u64(take(8))) for _ in range(r8_count)]
        if offset != len(artifact_bytes):
            findings.append(_finding("trailing_bytes", "artifact has bytes after R8 section"))
        r8_section = artifact_bytes[r8_start:offset]
    except ValueError as error:
        findings.append(_finding("truncated", str(error)))
        return tuple(findings)

    cells = tuple(cell for cell, _ in dynamic_rows)
    if tuple(sorted(cells)) != cells or len(set(cells)) != len(cells):
        findings.append(
            _finding("dynamic_order", "dynamic R7 identities are not strict numeric order")
        )
    for cell, values in dynamic_rows:
        try:
            cell_text = h3.int_to_str(cell)
            valid_r7 = h3.is_valid_cell(cell_text) and h3.get_resolution(cell_text) == 7
        except (h3.H3BaseException, OverflowError, ValueError):
            valid_r7 = False
        if not valid_r7:
            findings.append(_finding("dynamic_identity", f"{cell:#x} is not canonical R7"))
            break
        decoded = tuple(struct.unpack(">d", bits.to_bytes(8, "big"))[0] for bits in values)
        if any(not math.isfinite(value) for value in decoded) or any(
            bits == 0x8000_0000_0000_0000 for bits in values
        ):
            findings.append(
                _finding("dynamic_value", "dynamic values must be finite and not negative zero")
            )
            break
        if any(value < 0.0 for value in decoded[:7]) or any(
            not 0.0 <= value <= 1.0 for value in decoded[7:]
        ):
            findings.append(
                _finding("dynamic_value_domain", "dynamic values exceed their V1 domains")
            )
            break

    source_frame = bytearray(source_domain)
    source_frame.extend(len(cells).to_bytes(8, "big"))
    for cell in cells:
        source_frame.extend(cell.to_bytes(8, "big"))
    actual_source_digest = hashlib.sha256(source_frame).digest()
    if (
        actual_source_digest != expected_source_digest
        or encoded_source_digest != expected_source_digest
    ):
        findings.append(_finding("source_digest", "R7 source digest chain differs"))
    if encoded_base_digest != expected_base_digest:
        findings.append(_finding("base_cohort_digest", "base cohort digest differs"))

    children = tuple(child for child, _ in r8_rows)
    if tuple(sorted(children)) != children or len(set(children)) != len(children):
        findings.append(_finding("r8_order", "R8 children are not strict numeric order"))
    parent_set = frozenset(cells)
    for child, parent in r8_rows:
        try:
            child_text = h3.int_to_str(child)
            parent_text = h3.int_to_str(parent)
            valid_parent = (
                h3.is_valid_cell(child_text)
                and h3.get_resolution(child_text) == 8
                and h3.is_valid_cell(parent_text)
                and h3.get_resolution(parent_text) == 7
                and parent in parent_set
                and h3.str_to_int(h3.cell_to_parent(child_text, 7)) == parent
            )
        except (h3.H3BaseException, OverflowError, ValueError):
            valid_parent = False
        if not valid_parent:
            findings.append(
                _finding("r8_parent", f"invalid R8 child-parent pair {child:#x}/{parent:#x}")
            )
            break

    try:
        expected_r8_rows = tuple(
            sorted(
                (h3.str_to_int(child), parent)
                for parent in cells
                for child in h3.cell_to_children(h3.int_to_str(parent), 8)
            )
        )
    except (h3.H3BaseException, OverflowError, ValueError):
        findings.append(
            _finding("r8_coverage", "R8 section is not the exact complete immediate-child set")
        )
    else:
        if tuple(r8_rows) != expected_r8_rows:
            findings.append(
                _finding("r8_coverage", "R8 section is not the exact complete immediate-child set")
            )

    actual_r8_digest = hashlib.sha256(r8_section).digest()
    if actual_r8_digest != expected_r8_digest or encoded_r8_digest != expected_r8_digest:
        findings.append(_finding("r8_digest", "R8 section digest chain differs"))
    bundle_frame = bytearray(bundle_domain)
    bundle_frame.extend(expected_base_digest)
    bundle_frame.extend(expected_r8_digest)
    actual_bundle_digest = hashlib.sha256(bundle_frame).digest()
    if (
        actual_bundle_digest != expected_bundle_digest
        or encoded_bundle_digest != expected_bundle_digest
    ):
        findings.append(
            _finding("reference_bundle_digest", "composite reference digest chain differs")
        )

    if any(cell not in parent_set for cell in audited_r7_values):
        findings.append(_finding("audited_r7", "audited R7 identities are absent"))
    if audited_pair not in frozenset(r8_rows):
        findings.append(_finding("audited_r8", "audited R8 child-parent pair is absent"))
    return tuple(findings)


def verify_michigan_dynamic_hex_foundation_v1(
    contract_path: Path = DEFAULT_CONTRACT,
) -> tuple[FoundationFinding, ...]:
    """Load the canonical manifest and fixture parts, then verify independently."""

    try:
        contract = _load_contract(contract_path)
        root = contract_path.resolve().parent.parent
        artifact, part_sizes = _fixture_bytes(contract, root)
    except (OSError, TypeError, ValueError, yaml.YAMLError) as error:
        return (_finding("load_error", str(error)),)
    return verify_foundation_artifact(contract, artifact, part_sizes)


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("contract", nargs="?", type=Path, default=DEFAULT_CONTRACT)
    return parser.parse_args()


def main() -> int:
    findings = verify_michigan_dynamic_hex_foundation_v1(_parse_args().contract)
    if findings:
        for finding in findings:
            print(f"{finding.code}: {finding.detail}")
        return 1
    print("MICHIGAN_DYNAMIC_HEX_FOUNDATION_V1_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
