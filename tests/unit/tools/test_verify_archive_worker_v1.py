"""Independent behavioral checks for ArchiveWorkerV1."""

from __future__ import annotations

import copy
import json
from pathlib import Path

import pytest
from tools.verify_archive_worker_v1 import (
    ArchiveWorkerContractRefusal,
    classify_receipt,
    classify_sweep,
    derive_watermark,
    load_contract,
    load_vectors,
    main,
    match_batch_receipt,
    verify_all,
)

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = ROOT / "contracts" / "archive_worker_v1.yaml"
VECTORS = ROOT / "contracts" / "archive_worker_v1_vectors.jsonl"


def test_quiet_receipt_settles_but_an_undrained_receipt_stages() -> None:
    assert classify_receipt(0, 0) == "Consume"
    assert classify_receipt(0, 4) == "Stage"
    assert classify_receipt(256, 1) == "Stage"
    assert classify_receipt(60, 0) == "Consume"


def test_shared_contract_verifies_independently() -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)

    assert contract["meta"]["contract"] == "ArchiveWorkerV1"
    assert verify_all(contract, vectors, ROOT) == []


def test_every_vector_kind_is_present() -> None:
    vectors = load_vectors(VECTORS)

    assert {row["kind"] for row in vectors} == {"watermark", "match", "plan", "sweep", "identity"}
    assert len(vectors) == 21
    assert [row["id"] for row in vectors if row["kind"] == "watermark"] == [
        "watermark-empty-state",
        "watermark-all-consumed",
        "watermark-gap-pending",
        "watermark-pending-first",
    ]
    multi_page = next(row for row in vectors if row["id"] == "plan-multi-page-materializes")
    assert multi_page["data"]["batch"]["page_count"] == 2
    assert multi_page["data"]["expected"] == "Consume"


def test_pure_derivations_match_the_pinned_semantics() -> None:
    assert derive_watermark(None, 0) == 0
    assert derive_watermark(None, 5) == 5
    assert derive_watermark(2, 3) == 1
    assert derive_watermark(1, 3) == 0
    assert classify_receipt(0, 0) == "Consume"
    assert classify_receipt(1, 0) == "Consume"
    assert classify_receipt(256, 316) == "Stage"
    assert classify_receipt(0, 4) == "Stage"
    plans, error = classify_sweep(
        [{"batch": {"page_count": 0, "remaining": 0}}, {"batch": {"page_count": 1, "remaining": 0}}]
    )
    assert plans == ["Consume", "Consume"]
    assert error is None
    plans, error = classify_sweep(
        [{"batch": {"page_count": 1, "remaining": 0}}, {"error": "ReceiptMismatch"}]
    )
    assert plans == ["Consume"]
    assert error == "ReceiptMismatch"


def test_watermark_mutation_refuses_stale_expected_value() -> None:
    contract = load_contract(SCHEMA)
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "watermark-gap-pending")
    row["data"]["expected"] = 2

    errors = verify_all(contract, vectors, ROOT)

    assert any("watermark-gap-pending" in error for error in errors)


def test_match_semantic_mutation_moves_the_identity_refusal() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "match-tick-mismatch")
    batch = row["data"]["batch"]
    receipt = row["data"]["receipt"]
    assert (
        match_batch_receipt(
            {
                "resolve_tick": batch["resolve_tick"],
                "tick_content_hash_hex": batch["tick_content_hash_hex"],
            },
            {
                "resolve_tick": receipt["resolve_tick"],
                "tick_content_hash_hex": receipt["tick_content_hash_hex"],
            },
        )
        == "ReceiptMismatch"
    )
    row["data"]["expected"] = "ok"
    row["data"].pop("expected_error")

    errors = verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert any("match-tick-mismatch" in error for error in errors)


def test_sweep_stop_on_first_error_pins_the_later_steps_unconsulted() -> None:
    contract = load_contract(SCHEMA)
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "sweep-stop-on-first-error")
    row["data"]["steps"].append({"error": "StoredPageMismatch"})

    assert verify_all(contract, vectors, ROOT) == []


def test_sweep_plans_mutation_refuses_stale_ordering() -> None:
    contract = load_contract(SCHEMA)
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "sweep-mixed-order")
    row["data"]["expected"] = ["Stage", "Consume", "Consume"]

    errors = verify_all(contract, vectors, ROOT)

    assert any("sweep-mixed-order" in error for error in errors)


def test_identity_pins_the_checked_in_source_bytes() -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    identity = next(row for row in vectors if row["id"] == "identity-sql-and-bound")

    assert (
        identity["data"]["pending_receipts_sql_sha256_hex"]
        == contract["constants"]["pending_receipts_sql_sha256"]
    )
    assert (
        identity["data"]["watermark_sql_sha256_hex"]
        == contract["constants"]["watermark_sql_sha256"]
    )
    assert identity["data"]["max_receipts_per_sweep"] == 256
    assert identity["data"]["max_scan_per_sweep"] == 4096
    assert contract["constants"]["sweep_max_scan"] == 4096
    assert verify_all(contract, vectors, ROOT) == []


def test_identity_scan_bound_mutation_refuses() -> None:
    contract = load_contract(SCHEMA)
    vectors = copy.deepcopy(load_vectors(VECTORS))
    identity = next(row for row in vectors if row["id"] == "identity-sql-and-bound")
    identity["data"]["max_scan_per_sweep"] = 8192

    errors = verify_all(contract, vectors, ROOT)

    assert any("sweep scan bound mismatch" in error for error in errors)


def test_identity_forbidding_the_keyset_cursor_clause_refuses(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    monkeypatch.setattr(
        "tools.verify_archive_worker_v1.PENDING_SQL_FORBIDDEN_CLAUSES",
        ["d.resolve_tick > $3::bigint"],
    )

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "pending_sql_drift"
    assert exc_info.value.detail == "d.resolve_tick > $3::bigint"


def test_identity_source_drift_refuses(monkeypatch: pytest.MonkeyPatch) -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    monkeypatch.setattr("tools.verify_archive_worker_v1.SOURCE_PATH", "rust/crates/missing.rs")

    errors = verify_all(contract, vectors, ROOT)

    assert any("pinned source path drift" in error for error in errors)


def test_identity_sql_clause_removal_refuses(monkeypatch: pytest.MonkeyPatch) -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    monkeypatch.setattr(
        "tools.verify_archive_worker_v1.PENDING_SQL_REQUIRED_CLAUSES",
        ["ORDER BY d.resolve_tick DESC"],
    )

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "pending_sql_drift"


def test_compiled_contract_drift_refuses() -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    contract["constants"]["sweep_max_receipts"] = 255

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "compiled_contract_drift"


def test_missing_vector_kind_refuses() -> None:
    contract = load_contract(SCHEMA)
    vectors = [row for row in load_vectors(VECTORS) if row["kind"] != "sweep"]

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "vector_id_drift"


def test_duplicate_vector_id_refuses_before_indexing() -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    duplicate = copy.deepcopy(vectors[1])
    duplicate["id"] = vectors[0]["id"]
    vectors.append(duplicate)

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "duplicate_vector_id"


def test_cli_prints_typed_refusal_without_traceback(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    vectors[0].pop("id")
    vectors_path = tmp_path / "malformed.jsonl"
    vectors_path.write_text(
        "\n".join(json.dumps(row, separators=(",", ":")) for row in vectors),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "sys.argv",
        [
            "verify_archive_worker_v1.py",
            "--schema",
            str(SCHEMA),
            "--vectors",
            str(vectors_path),
        ],
    )

    assert main() == 1
    assert capsys.readouterr().out == "vector_row_shape: 1\n"


def test_vector_loader_refuses_row_and_line_overflow(tmp_path: Path) -> None:
    row = '{"id":"x","kind":"x","data":{}}'
    rows_path = tmp_path / "rows.jsonl"
    rows_path.write_text("\n".join([row] * 33), encoding="utf-8")
    with pytest.raises(ArchiveWorkerContractRefusal, match="too_many_rows"):
        load_vectors(rows_path)

    line_path = tmp_path / "line.jsonl"
    line_path.write_text("x" * 16_385, encoding="utf-8")
    with pytest.raises(ArchiveWorkerContractRefusal, match="invalid_line_length"):
        load_vectors(line_path)


def test_vector_loader_refuses_duplicate_json_keys(tmp_path: Path) -> None:
    vectors_path = tmp_path / "duplicate-key.jsonl"
    vectors_path.write_text(
        '{"id":"a","id":"b","kind":"x","data":{}}',
        encoding="utf-8",
    )

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        load_vectors(vectors_path)

    assert exc_info.value.code == "duplicate_json_key"


def test_compiled_layout_drift_refuses() -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    fields = contract["layouts"]["batch_ref_v1"]["fields"]
    fields[0], fields[1] = fields[1], fields[0]

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "compiled_contract_drift"
    assert exc_info.value.detail == "layouts"


def test_match_receipt_tick_zero_refuses_with_the_typed_code() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "match-exact-ok")
    row["data"]["receipt"]["resolve_tick"] = 0

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_tick"
    assert "data.receipt.resolve_tick" in exc_info.value.detail


def test_match_tick_past_bigint_refuses_like_the_rust_receipt_constructor() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "match-exact-ok")
    row["data"]["receipt"]["resolve_tick"] = (1 << 63) + 1

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_tick"


def test_watermark_first_pending_tick_zero_refuses() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "watermark-all-consumed")
    row["data"]["first_pending_tick"] = 0

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_tick"


def test_plan_page_count_past_the_batch_bound_refuses() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "plan-nonempty-materializes")
    row["data"]["batch"]["page_count"] = 257

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_page_count"


def test_digest32_rejects_whitespace_padding() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "match-hash-mismatch")
    digest = row["data"]["batch"]["tick_content_hash_hex"]
    row["data"]["batch"]["tick_content_hash_hex"] = digest[:2] + "  " + digest[4:]

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_digest"


def test_sweep_step_with_unknown_error_variant_refuses() -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "sweep-all-quiet")
    row["data"]["steps"].append({"error": "UnknownSubject"})

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_sweep_steps"


def test_cli_derives_repo_root_independently_of_vectors_location(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    vectors_path = tmp_path / "elsewhere" / "copied.jsonl"
    vectors_path.parent.mkdir(parents=True)
    vectors_path.write_text(VECTORS.read_text(encoding="utf-8"), encoding="utf-8")
    monkeypatch.setattr(
        "sys.argv",
        [
            "verify_archive_worker_v1.py",
            "--schema",
            str(SCHEMA),
            "--vectors",
            str(vectors_path),
        ],
    )

    assert main() == 0


@pytest.mark.parametrize("expected", [True, False])
def test_watermark_boolean_expected_refuses_with_the_typed_code(expected: bool) -> None:
    vectors = copy.deepcopy(load_vectors(VECTORS))
    row = next(item for item in vectors if item["id"] == "watermark-all-consumed")
    row["data"]["expected"] = expected

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(load_contract(SCHEMA), vectors, ROOT)

    assert exc_info.value.code == "invalid_tick"


def test_unknown_vector_kind_refuses_before_indexing() -> None:
    contract = load_contract(SCHEMA)
    vectors = copy.deepcopy(load_vectors(VECTORS))
    vectors[0]["kind"] = "mystery"

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "unknown_vector_kind"
    assert exc_info.value.detail == "mystery"


def test_oversize_source_read_uses_the_size_specific_refusal(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    contract = load_contract(SCHEMA)
    vectors = load_vectors(VECTORS)
    monkeypatch.setattr("tools.verify_archive_worker_v1.MAX_CONTRACT_BYTES", 1)

    with pytest.raises(ArchiveWorkerContractRefusal) as exc_info:
        verify_all(contract, vectors, ROOT)

    assert exc_info.value.code == "source_too_large"
