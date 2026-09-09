#!/usr/bin/env python3
"""Execute the closed H3 reader-parity corpus through a read-only backend.

The backend is deliberately injected.  PostgreSQL qualification supplies one
adapter whose nine methods call the production readers (or projection-only
SQL) after its own deterministic fixture is installed.  This module owns no
formula, join, fixture write, compatibility read, or database lifecycle.
"""

from __future__ import annotations

import hashlib
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol
from uuid import UUID

from tools.verify_h3_reader_cutover_v1 import (
    RESULT_DOMAIN,
    H3ReaderCutoverRefusal,
    _canonical_atom,
    _canonical_json,
    verify_reader_parity_vectors,
)


class ReaderParityBackendV1(Protocol):
    """The sole read-only integration seam for the nine governed operations."""

    def execute_reader_case(
        self, operation: str, inputs: dict[str, Any]
    ) -> dict[str, list[dict[str, Any]]]:
        """Return named, ordered rows from production readers or projection SQL."""


@dataclass(frozen=True, slots=True)
class H3ReaderParityFinding:
    """One bounded case-scoped mismatch from the live executor."""

    case_id: str
    code: str
    detail: str


def _decode_atom(atom: dict[str, Any]) -> Any:
    canonical = _canonical_atom(atom)
    atom_type = canonical["type"]
    value = canonical["value"]
    if atom_type in {"h3_cell_id", "nullable_h3_cell_id"}:
        return None if value is None else int(value)
    if atom_type == "f64_bits":
        return struct.unpack(">d", bytes.fromhex(value))[0]
    if atom_type == "uuid":
        return UUID(value)
    return value


def _encode_actual_atom(expected: dict[str, Any], actual: Any) -> dict[str, Any]:
    atom_type = expected["type"]
    value: Any
    if atom_type == "bool":
        value = actual
    elif atom_type == "i64":
        value = actual
    elif atom_type in {"h3_cell_id", "nullable_h3_cell_id"}:
        if actual is None and atom_type == "nullable_h3_cell_id":
            value = None
        elif isinstance(actual, bool) or not isinstance(actual, int):
            raise H3ReaderCutoverRefusal("vector_atom", atom_type)
        else:
            value = str(actual)
    elif atom_type == "f64_bits":
        if not isinstance(actual, float):
            raise H3ReaderCutoverRefusal("vector_atom", "f64_bits")
        value = struct.pack(">d", actual).hex()
    elif atom_type == "uuid":
        value = str(actual) if isinstance(actual, UUID) else actual
    elif atom_type in {"text", "nullable_text"}:
        value = actual
    else:
        raise H3ReaderCutoverRefusal("vector_atom", f"type={atom_type!r}")
    return _canonical_atom({"type": atom_type, "value": value})


def _normalize_actual_sets(
    expected: dict[str, list[dict[str, dict[str, Any]]]], actual: object
) -> dict[str, list[dict[str, dict[str, Any]]]]:
    if not isinstance(actual, dict) or set(actual) != set(expected):
        raise H3ReaderCutoverRefusal("result_shape", "named result sets")
    normalized: dict[str, list[dict[str, dict[str, Any]]]] = {}
    for set_name, expected_rows in expected.items():
        actual_rows = actual[set_name]
        if not isinstance(actual_rows, list) or len(actual_rows) != len(expected_rows):
            raise H3ReaderCutoverRefusal("result_shape", f"{set_name}: row count")
        normalized_rows: list[dict[str, dict[str, Any]]] = []
        for index, (expected_row, actual_row) in enumerate(
            zip(expected_rows, actual_rows, strict=True)
        ):
            if not isinstance(actual_row, dict) or set(actual_row) != set(expected_row):
                raise H3ReaderCutoverRefusal("result_shape", f"{set_name}[{index}]: columns")
            normalized_rows.append(
                {
                    column: _encode_actual_atom(expected_atom, actual_row[column])
                    for column, expected_atom in expected_row.items()
                }
            )
        normalized[set_name] = normalized_rows
    return normalized


def execute_h3_reader_parity_v1(
    contract: dict[str, Any],
    vectors: list[dict[str, Any]],
    root: Path,
    backend: ReaderParityBackendV1,
) -> list[H3ReaderParityFinding]:
    """Execute every verified case once and return bounded semantic findings.

    Integration call signature::

        execute_h3_reader_parity_v1(contract, vectors, repository_root, backend)

    ``backend.execute_reader_case(operation, decoded_inputs)`` must be read-only
    and return all named row sets in database order.  Exact row order is part of
    the digest; the executor never sorts or otherwise repairs backend output.
    """
    verify_reader_parity_vectors(contract, vectors, root)
    findings: list[H3ReaderParityFinding] = []
    for vector in vectors:
        case_id = vector["id"]
        inputs = {name: _decode_atom(atom) for name, atom in vector["inputs"].items()}
        try:
            actual = backend.execute_reader_case(vector["operation"], inputs)
            normalized = _normalize_actual_sets(vector["expected"]["sets"], actual)
            digest = hashlib.sha256(RESULT_DOMAIN + _canonical_json(normalized)).hexdigest()
            expected_digest = vector["expected"]["sha256"]
            if digest != expected_digest:
                findings.append(
                    H3ReaderParityFinding(
                        case_id=case_id,
                        code="result_digest",
                        detail=f"expected={expected_digest} actual={digest}",
                    )
                )
        except H3ReaderCutoverRefusal as error:
            findings.append(
                H3ReaderParityFinding(case_id=case_id, code=error.code, detail=error.detail)
            )
        except Exception as error:  # noqa: BLE001 - adapter failures become bounded evidence
            findings.append(
                H3ReaderParityFinding(
                    case_id=case_id,
                    code="backend_refusal",
                    detail=f"{type(error).__name__}: {error}",
                )
            )
    return findings


__all__ = [
    "H3ReaderParityFinding",
    "ReaderParityBackendV1",
    "execute_h3_reader_parity_v1",
]
