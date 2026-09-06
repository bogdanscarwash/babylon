#!/usr/bin/env python3
"""Independently verify the bounded ArchiveWorkerV1 contract corpus."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

import yaml

SOURCE_PATH = "rust/crates/babylon-persistence/src/archive_worker.rs"
MAX_I64 = (1 << 63) - 1
MAX_RECEIPTS_PER_SWEEP = 256
MAX_SCAN_PER_SWEEP = 4096
MAX_PAGE_COUNT = 256
MAX_REMAINING_COUNT = 65_535
MAX_CONTRACT_BYTES = 131_072
MAX_VECTOR_ROWS = 32
MAX_VECTOR_LINE_BYTES = 16_384
MAX_VECTOR_OBJECT_FIELDS = 64
PLANS = ["Consume", "Stage"]
DISPOSITIONS = ["Applied", "AlreadyConsumed", "Paged"]
ERROR_VARIANTS_USED = ["InvalidVerifiedTick", "ReceiptMismatch", "StoredPageMismatch"]
REQUIRED_VECTOR_KINDS = {"watermark", "match", "plan", "sweep", "identity"}
REQUIRED_ROW_IDS = {
    "watermark": {
        "watermark-empty-state",
        "watermark-all-consumed",
        "watermark-gap-pending",
        "watermark-pending-first",
    },
    "match": {"match-exact-ok", "match-tick-mismatch", "match-hash-mismatch"},
    "plan": {
        "plan-empty-consumes",
        "plan-nonempty-materializes",
        "plan-multi-page-materializes",
        "plan-paged-materializes-without-consuming",
        "plan-empty-budget-exhausted-still-materializes",
    },
    "sweep": {
        "sweep-all-quiet",
        "sweep-mixed-order",
        "sweep-stop-on-first-error",
        "sweep-error-first",
        "sweep-foundation-drain-one",
        "sweep-foundation-drain-two",
        "sweep-foundation-drain-three",
        "sweep-foundation-drain-four",
    },
    "identity": {"identity-sql-and-bound"},
}
PENDING_SQL_REQUIRED_CLAUSES = [
    "JOIN babylon_state.tick_commit",
    "LEFT JOIN babylon_meta.archive_receipt_consumption_v1",
    "c.campaign_id IS NULL",
    "d.resolve_tick > $3::bigint",
    "ORDER BY d.resolve_tick ASC",
    "LIMIT $2",
    "d.campaign_id = $1::uuid",
]
PENDING_SQL_FORBIDDEN_CLAUSES = ["FOR UPDATE", "MAX(", "OFFSET"]
WATERMARK_SQL_REQUIRED_CLAUSES = [
    "MIN(d.resolve_tick)",
    "COALESCE((SELECT MAX(d.resolve_tick)",
    ", 0)",
    "archive_receipt_consumption_v1",
]
WATERMARK_SQL_FORBIDDEN_CLAUSES = ["FOR UPDATE"]
COMPILED_META = {
    "contract": "ArchiveWorkerV1",
    "version": 1,
    "issue": "PER-22",
    "digest": "SHA-256 diagnostic; exact bytes govern retry equality",
}
COMPILED_CONSTANTS = {
    "source_path": SOURCE_PATH,
    "pending_receipts_sql_sha256": "e475c0c9c8148e60102a592f22c6749f686704a7a6911cd545257c59e50be13e",
    "watermark_sql_sha256": "a93cb9e0d2ff34e2ef58d6d4e5475ad44a313bb052e3d100df71e82521afe204",
    "sweep_max_receipts": MAX_RECEIPTS_PER_SWEEP,
    "sweep_max_scan": MAX_SCAN_PER_SWEEP,
    "receipt_plans": PLANS,
    "receipt_dispositions": DISPOSITIONS,
    "error_variants_used": ERROR_VARIANTS_USED,
}
COMPILED_BOUNDS = {
    "contract_bytes": MAX_CONTRACT_BYTES,
    "vector_rows": MAX_VECTOR_ROWS,
    "vector_line_bytes": MAX_VECTOR_LINE_BYTES,
    "vector_object_fields": MAX_VECTOR_OBJECT_FIELDS,
}
COMPILED_LAYOUTS = {
    "watermark_input_v1": {
        "fields": ["first_pending_tick_nullable_u64", "max_receipt_tick_u64"],
        "derivation": "first_pending present -> first_pending - 1; absent -> max_receipt_tick",
    },
    "batch_ref_v1": {
        "fields": [
            "resolve_tick_u64",
            "tick_content_hash_exact_32_bytes_lower_hex",
            "page_count_u64",
        ],
        "resolve_tick_domain": "1..=i64::MAX",
        "page_count_domain": "0..=256",
    },
    "paged_batch_ref_v1": {
        "fields": [
            "resolve_tick_u64",
            "tick_content_hash_exact_32_bytes_lower_hex",
            "page_count_u64",
            "remaining_u64",
        ],
        "resolve_tick_domain": "1..=i64::MAX",
        "page_count_domain": "0..=256",
        "remaining_domain": "0..=65535",
    },
    "receipt_ref_v1": {
        "fields": ["resolve_tick_u64", "tick_content_hash_exact_32_bytes_lower_hex"],
        "resolve_tick_domain": "1..=i64::MAX",
    },
    "match_input_v1": {"fields": ["batch_ref_v1", "receipt_ref_v1"]},
    "sweep_step_v1": {
        "one_of": [
            "batch: paged_batch_ref_v1",
            "error: SemanticArchiveErrorV1 variant name",
        ]
    },
    "sweep_output_v1": {
        "one_of": [
            "expected: ordered receipt plans",
            "expected_error: SemanticArchiveErrorV1 variant name",
        ]
    },
}


class ArchiveWorkerContractRefusal(ValueError):
    """One typed independent-verifier refusal."""

    def __init__(self, code: str, detail: str) -> None:
        self.code = code
        self.detail = detail
        super().__init__(f"{code}: {detail}")


def _bounded_file_bytes(path: Path, maximum: int, code: str) -> bytes:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise ArchiveWorkerContractRefusal("file_read", str(path)) from error
    if size > maximum:
        raise ArchiveWorkerContractRefusal(code, str(size))
    try:
        return path.read_bytes()
    except OSError as error:
        raise ArchiveWorkerContractRefusal("file_read", str(path)) from error


def load_contract(path: Path) -> dict[str, Any]:
    """Load one bounded YAML mapping."""
    raw = _bounded_file_bytes(path, MAX_CONTRACT_BYTES, "schema_too_large")
    try:
        loaded = yaml.safe_load(raw)
    except yaml.YAMLError as error:
        raise ArchiveWorkerContractRefusal("invalid_schema", str(path)) from error
    if not isinstance(loaded, dict):
        raise ArchiveWorkerContractRefusal("invalid_schema", "root mapping")
    return loaded


def load_vectors(path: Path) -> list[dict[str, Any]]:
    """Load bounded JSONL rows without an unbounded whole-file read."""
    maximum = MAX_VECTOR_ROWS * (MAX_VECTOR_LINE_BYTES + 1)
    raw = _bounded_file_bytes(path, maximum, "vectors_too_large")
    lines = raw.splitlines()
    if len(lines) > MAX_VECTOR_ROWS:
        raise ArchiveWorkerContractRefusal("too_many_rows", str(len(lines)))
    rows: list[dict[str, Any]] = []
    for index in range(MAX_VECTOR_ROWS):
        if index >= len(lines):
            break
        line = lines[index]
        if not line or len(line) > MAX_VECTOR_LINE_BYTES:
            raise ArchiveWorkerContractRefusal("invalid_line_length", str(index + 1))
        try:
            row = json.loads(line, object_pairs_hook=_unique_json_object)
        except (json.JSONDecodeError, UnicodeDecodeError) as error:
            raise ArchiveWorkerContractRefusal("invalid_json", str(index + 1)) from error
        if not isinstance(row, dict):
            raise ArchiveWorkerContractRefusal("vector_row_shape", str(index + 1))
        rows.append(row)
    return rows


def _unique_json_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    if len(pairs) > MAX_VECTOR_OBJECT_FIELDS:
        raise ArchiveWorkerContractRefusal("json_object_fields", str(len(pairs)))
    result: dict[str, Any] = {}
    for index in range(MAX_VECTOR_OBJECT_FIELDS):
        if index >= len(pairs):
            break
        key, value = pairs[index]
        if key in result:
            raise ArchiveWorkerContractRefusal("duplicate_json_key", key)
        result[key] = value
    return result


def _verify_compiled_contract(contract: dict[str, Any]) -> None:
    if contract.get("meta") != COMPILED_META:
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "meta")
    if contract.get("constants") != COMPILED_CONSTANTS:
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "constants")
    if contract.get("bounds") != COMPILED_BOUNDS:
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "bounds")
    if contract.get("layouts") != COMPILED_LAYOUTS:
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "layouts")
    if contract.get("production_decoder") != "prohibited":
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "production_decoder")
    required = contract.get("vector_kinds", {}).get("required")
    if not isinstance(required, list) or set(required) != REQUIRED_VECTOR_KINDS:
        raise ArchiveWorkerContractRefusal("compiled_contract_drift", "vector_kinds")


def _validated_rows(vectors: list[dict[str, Any]]) -> list[dict[str, Any]]:
    if len(vectors) > MAX_VECTOR_ROWS:
        raise ArchiveWorkerContractRefusal("too_many_rows", str(len(vectors)))
    rows = vectors[:MAX_VECTOR_ROWS]
    seen_ids: set[str] = set()
    for index in range(MAX_VECTOR_ROWS):
        if index >= len(rows):
            break
        row = rows[index]
        row_id = row.get("id")
        if (
            set(row) != {"id", "kind", "data"}
            or not isinstance(row_id, str)
            or not row_id
            or not isinstance(row.get("kind"), str)
            or not isinstance(row.get("data"), dict)
        ):
            raise ArchiveWorkerContractRefusal("vector_row_shape", str(index + 1))
        if row_id in seen_ids:
            raise ArchiveWorkerContractRefusal("duplicate_vector_id", row_id)
        seen_ids.add(row_id)
        if row["kind"] not in REQUIRED_VECTOR_KINDS:
            raise ArchiveWorkerContractRefusal("unknown_vector_kind", row["kind"])
    for kind, required_ids in REQUIRED_ROW_IDS.items():
        actual = {row["id"] for row in rows if row["kind"] == kind}
        if actual != required_ids:
            raise ArchiveWorkerContractRefusal("vector_id_drift", repr(sorted(actual)))
    return rows


def _tick(value: object, field: str, *, allow_zero: bool = False) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ArchiveWorkerContractRefusal("invalid_tick", field)
    if value < 0 or (value == 0 and not allow_zero) or value > MAX_I64:
        raise ArchiveWorkerContractRefusal("invalid_tick", field)
    return value


_LOWER_HEX = frozenset("0123456789abcdef")


def _digest32(value: object, field: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(char not in _LOWER_HEX for char in value)
    ):
        raise ArchiveWorkerContractRefusal("invalid_digest", field)
    if len(bytes.fromhex(value)) != 32:
        raise ArchiveWorkerContractRefusal("invalid_digest", field)
    return value


def _page_count(value: object, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ArchiveWorkerContractRefusal("invalid_page_count", field)
    if value < 0 or value > MAX_PAGE_COUNT:
        raise ArchiveWorkerContractRefusal("invalid_page_count", field)
    return value


def _remaining_count(value: object, field: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ArchiveWorkerContractRefusal("invalid_remaining_count", field)
    if value < 0 or value > MAX_REMAINING_COUNT:
        raise ArchiveWorkerContractRefusal("invalid_remaining_count", field)
    return value


def _batch_ref(value: object, field: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != {
        "resolve_tick",
        "tick_content_hash_hex",
        "page_count",
    }:
        raise ArchiveWorkerContractRefusal("invalid_batch_ref", field)
    return {
        "resolve_tick": _tick(value.get("resolve_tick"), f"{field}.resolve_tick"),
        "tick_content_hash_hex": _digest32(
            value.get("tick_content_hash_hex"), f"{field}.tick_content_hash_hex"
        ),
        "page_count": _page_count(value.get("page_count"), f"{field}.page_count"),
    }


def _paged_batch_ref(value: object, field: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != {
        "resolve_tick",
        "tick_content_hash_hex",
        "page_count",
        "remaining",
    }:
        raise ArchiveWorkerContractRefusal("invalid_batch_ref", field)
    return {
        "resolve_tick": _tick(value.get("resolve_tick"), f"{field}.resolve_tick"),
        "tick_content_hash_hex": _digest32(
            value.get("tick_content_hash_hex"), f"{field}.tick_content_hash_hex"
        ),
        "page_count": _page_count(value.get("page_count"), f"{field}.page_count"),
        "remaining": _remaining_count(value.get("remaining"), f"{field}.remaining"),
    }


def _receipt_ref(value: object, field: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != {"resolve_tick", "tick_content_hash_hex"}:
        raise ArchiveWorkerContractRefusal("invalid_receipt_ref", field)
    return {
        "resolve_tick": _tick(value.get("resolve_tick"), f"{field}.resolve_tick"),
        "tick_content_hash_hex": _digest32(
            value.get("tick_content_hash_hex"), f"{field}.tick_content_hash_hex"
        ),
    }


def derive_watermark(first_pending_tick: int | None, max_receipt_tick: int) -> int:
    """Recompute the contiguous persisted consumption watermark."""
    if first_pending_tick is not None:
        return first_pending_tick - 1
    return max_receipt_tick


def classify_receipt(_page_count: int, remaining: int) -> str:
    """Settle a fully evaluated receipt; preserve every undrained page."""
    return "Consume" if remaining == 0 else "Stage"


def match_batch_receipt(batch: dict[str, Any], receipt: dict[str, Any]) -> str | None:
    """Recompute the batch-identity refusal; None means the exact match."""
    if (
        batch["resolve_tick"] != receipt["resolve_tick"]
        or batch["tick_content_hash_hex"] != receipt["tick_content_hash_hex"]
    ):
        return "ReceiptMismatch"
    return None


def classify_sweep(steps: list[dict[str, Any]]) -> tuple[list[str], str | None]:
    """Recompute the sweep plan sequence, stopping at the first error."""
    plans: list[str] = []
    for step in steps:
        if "error" in step:
            return plans, step["error"]
        plans.append(classify_receipt(step["batch"]["page_count"], step["batch"]["remaining"]))
    return plans, None


def _rust_const_str(source: str, name: str) -> str:
    """Extract one Rust string literal, resolving continuation escapes exactly."""
    match = re.search(
        rf"pub const {name}: &str = \"(.*?)\";",
        source,
        flags=re.DOTALL,
    )
    if match is None:
        raise ArchiveWorkerContractRefusal("source_drift", name)
    literal = match.group(1)
    output = bytearray()
    index = 0
    while index < len(literal):
        char = literal[index]
        if char != "\\":
            output.extend(char.encode("utf-8"))
            index += 1
            continue
        if index + 1 >= len(literal):
            raise ArchiveWorkerContractRefusal("source_drift", name)
        escaped = literal[index + 1]
        if escaped == "\n":
            index += 2
            while index < len(literal) and literal[index] in " \t":
                index += 1
        elif escaped == "n":
            output.append(10)
            index += 2
        elif escaped == "t":
            output.append(9)
            index += 2
        elif escaped == '"':
            output.append(34)
            index += 2
        elif escaped == "\\":
            output.append(92)
            index += 2
        else:
            raise ArchiveWorkerContractRefusal("source_drift", f"{name} escape {escaped!r}")
    return bytes(output).decode("utf-8")


def _rust_const_int(source: str, name: str) -> int:
    match = re.search(rf"pub const {name}: i64 = (\d+);", source)
    if match is None:
        raise ArchiveWorkerContractRefusal("source_drift", name)
    return int(match.group(1))


def _source_sql(root: Path) -> tuple[str, str, int, int]:
    source = _bounded_file_bytes(root / SOURCE_PATH, MAX_CONTRACT_BYTES, "source_too_large").decode(
        "utf-8"
    )
    pending_sql = _rust_const_str(source, "ARCHIVE_PENDING_RECEIPTS_SQL_V1")
    watermark_sql = _rust_const_str(source, "ARCHIVE_SWEEP_WATERMARK_SQL_V1")
    max_receipts = _rust_const_int(source, "ARCHIVE_SWEEP_MAX_RECEIPTS_V1")
    max_scan = _rust_const_int(source, "ARCHIVE_SWEEP_MAX_SCAN_V1")
    return pending_sql, watermark_sql, max_receipts, max_scan


def _require_clauses(sql: str, clauses: list[str], code: str) -> None:
    for clause in clauses:
        if clause not in sql:
            raise ArchiveWorkerContractRefusal(code, clause)


def _forbid_clauses(sql: str, clauses: list[str], code: str) -> None:
    for clause in clauses:
        if clause in sql:
            raise ArchiveWorkerContractRefusal(code, clause)


def _verify_watermark(row: dict[str, Any]) -> str | None:
    data = row["data"]
    first_pending_raw = data.get("first_pending_tick")
    if first_pending_raw is not None:
        first_pending: int | None = _tick(first_pending_raw, "first_pending_tick")
    else:
        first_pending = None
    max_receipt = _tick(data.get("max_receipt_tick"), "max_receipt_tick", allow_zero=True)
    expected = _tick(data.get("expected"), "expected", allow_zero=True)
    if derive_watermark(first_pending, max_receipt) != expected:
        return f"{row['id']}: watermark derivation mismatch"
    return None


def _verify_match(row: dict[str, Any]) -> str | None:
    data = row["data"]
    batch = _batch_ref(data.get("batch"), "data.batch")
    receipt = _receipt_ref(data.get("receipt"), "data.receipt")
    error = match_batch_receipt(batch, receipt)
    if error is None:
        if data.get("expected") != "ok":
            return f"{row['id']}: exact match must report ok"
        return None
    if data.get("expected_error") != error:
        return f"{row['id']}: batch-identity refusal mismatch"
    return None


def _verify_plan(row: dict[str, Any]) -> str | None:
    data = row["data"]
    batch = _paged_batch_ref(data.get("batch"), "data.batch")
    if classify_receipt(batch["page_count"], batch["remaining"]) != data.get("expected"):
        return f"{row['id']}: receipt plan mismatch"
    return None


def _verify_sweep(row: dict[str, Any]) -> str | None:
    data = row["data"]
    steps_value = data.get("steps")
    if not isinstance(steps_value, list) or len(steps_value) > MAX_VECTOR_ROWS:
        raise ArchiveWorkerContractRefusal("invalid_sweep_steps", "data.steps")
    steps: list[dict[str, Any]] = []
    for index, step in enumerate(steps_value):
        path = f"data.steps[{index}]"
        if not isinstance(step, dict):
            raise ArchiveWorkerContractRefusal("invalid_sweep_steps", path)
        if "error" in step:
            error = step["error"]
            if set(step) != {"error"} or error not in ERROR_VARIANTS_USED:
                raise ArchiveWorkerContractRefusal("invalid_sweep_steps", path)
            steps.append({"error": error})
        else:
            if set(step) != {"batch"}:
                raise ArchiveWorkerContractRefusal("invalid_sweep_steps", path)
            steps.append({"batch": _paged_batch_ref(step["batch"], f"{path}.batch")})
    plans, error = classify_sweep(steps)
    if error is not None:
        if data.get("expected_error") != error:
            return f"{row['id']}: sweep stop-on-first-error mismatch"
        return None
    if data.get("expected") != plans:
        return f"{row['id']}: sweep plan ordering mismatch"
    return None


def _verify_identity(row: dict[str, Any], root: Path, contract: dict[str, Any]) -> str | None:
    data = row["data"]
    if data.get("source_path") != SOURCE_PATH:
        return f"{row['id']}: pinned source path drift"
    constants = contract.get("constants", {})
    pending_sql, watermark_sql, max_receipts, max_scan = _source_sql(root)
    if max_receipts != MAX_RECEIPTS_PER_SWEEP:
        return f"{row['id']}: sweep bound drift in pinned source"
    if max_scan != MAX_SCAN_PER_SWEEP or max_scan < max_receipts:
        return f"{row['id']}: sweep scan bound drift in pinned source"
    pending_sha256 = hashlib.sha256(pending_sql.encode("utf-8")).hexdigest()
    watermark_sha256 = hashlib.sha256(watermark_sql.encode("utf-8")).hexdigest()
    if pending_sha256 != constants.get("pending_receipts_sql_sha256"):
        return f"{row['id']}: pending-receipts SQL SHA-256 drift from contract constant"
    if watermark_sha256 != constants.get("watermark_sql_sha256"):
        return f"{row['id']}: watermark SQL SHA-256 drift from contract constant"
    if pending_sha256 != data.get("pending_receipts_sql_sha256_hex"):
        return f"{row['id']}: pending-receipts SQL SHA-256 mismatch"
    if watermark_sha256 != data.get("watermark_sql_sha256_hex"):
        return f"{row['id']}: watermark SQL SHA-256 mismatch"
    if data.get("max_receipts_per_sweep") != max_receipts:
        return f"{row['id']}: sweep bound mismatch"
    if data.get("max_scan_per_sweep") != max_scan:
        return f"{row['id']}: sweep scan bound mismatch"
    if (
        constants.get("sweep_max_receipts") != max_receipts
        or constants.get("sweep_max_scan") != max_scan
    ):
        return f"{row['id']}: sweep bound drift from contract constants"
    if data.get("plans") != PLANS or constants.get("receipt_plans") != PLANS:
        return f"{row['id']}: receipt plan taxonomy drift"
    if (
        data.get("dispositions") != DISPOSITIONS
        or constants.get("receipt_dispositions") != DISPOSITIONS
    ):
        return f"{row['id']}: disposition taxonomy drift"
    if data.get("error_variants") != ERROR_VARIANTS_USED:
        return f"{row['id']}: error-variant taxonomy drift"
    _require_clauses(pending_sql, PENDING_SQL_REQUIRED_CLAUSES, "pending_sql_drift")
    _forbid_clauses(pending_sql, PENDING_SQL_FORBIDDEN_CLAUSES, "pending_sql_drift")
    _require_clauses(watermark_sql, WATERMARK_SQL_REQUIRED_CLAUSES, "watermark_sql_drift")
    _forbid_clauses(watermark_sql, WATERMARK_SQL_FORBIDDEN_CLAUSES, "watermark_sql_drift")
    return None


def verify_all(contract: dict[str, Any], vectors: list[dict[str, Any]], root: Path) -> list[str]:
    """Verify all bounded rows and return exact row-scoped mismatches."""
    _verify_compiled_contract(contract)
    rows = _validated_rows(vectors)
    errors: list[str] = []
    for row in rows:
        kind = row["kind"]
        error: str | None = None
        if kind == "watermark":
            error = _verify_watermark(row)
        elif kind == "match":
            error = _verify_match(row)
        elif kind == "plan":
            error = _verify_plan(row)
        elif kind == "sweep":
            error = _verify_sweep(row)
        elif kind == "identity":
            error = _verify_identity(row, root, contract)
        if error is not None:
            errors.append(error)
    return errors


def main() -> int:
    """Verify repository contract paths or explicit alternatives."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--schema",
        type=Path,
        default=Path("contracts/archive_worker_v1.yaml"),
    )
    parser.add_argument(
        "--vectors",
        type=Path,
        default=Path("contracts/archive_worker_v1_vectors.jsonl"),
    )
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    try:
        errors = verify_all(load_contract(arguments.schema), load_vectors(arguments.vectors), root)
    except ArchiveWorkerContractRefusal as refusal:
        print(refusal)
        return 1
    for error in errors:
        print(error)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
