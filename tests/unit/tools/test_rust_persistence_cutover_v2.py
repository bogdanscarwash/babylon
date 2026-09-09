"""Focused laws for the active PER-311 persistence V2 contract."""

from __future__ import annotations

import hashlib
import tomllib
from copy import deepcopy
from pathlib import Path
from typing import Any

import yaml
from tools.verify_rust_persistence_cutover_v2 import (
    CONTRACT_PATH,
    EXPECTED_PREDECESSOR_EPOCH_9_LAW,
    PREDECESSOR_EPOCH_9_MIGRATION,
    Finding,
    _validate_contract,
    _validate_epoch_nine_migration_policy,
    _validate_epoch_nine_runtime_policy,
    _validate_runtime_authority_binding,
    verify,
)

ROOT = Path(__file__).resolve().parents[3]
V1_CONTRACT_SHA256 = "df438ddae7af7023e7a7b210712a51e09ca702ef1f47458891ced1e5ae28655f"
V1_VECTORS_SHA256 = "eb7e50f887e39a30d48e085b2d9b001bb3abd823089d7bd6df7c7a066e68ff94"


def _contract() -> dict[str, Any]:
    payload = yaml.safe_load((ROOT / CONTRACT_PATH).read_text(encoding="utf-8"))
    assert isinstance(payload, dict)
    return payload


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def test_checked_in_v2_contract_is_green() -> None:
    assert verify(ROOT) == []


def test_contract_refuses_family_reordering() -> None:
    contract = deepcopy(_contract())
    family_order = contract["envelope"]["family_order"]
    family_order[2], family_order[3] = family_order[3], family_order[2]
    findings: set[Finding] = set()

    _validate_contract(contract, ROOT, findings)

    assert findings == {
        Finding(
            "envelope_family_order",
            str(CONTRACT_PATH),
            "six-family order differs",
        )
    }


def test_v1_predecessor_artifacts_remain_exact_and_offline_only() -> None:
    meta = _contract()["meta"]
    assert meta["predecessor"] == {
        "path": "contracts/rust_persistence_cutover_v1.yaml",
        "sha256": V1_CONTRACT_SHA256,
    }
    assert meta["predecessor_vectors"] == {
        "path": "contracts/rust_persistence_cutover_v1_vectors.jsonl",
        "sha256": V1_VECTORS_SHA256,
        "disposition": "offline_historical_verification_only",
    }
    assert _sha256(ROOT / meta["predecessor"]["path"]) == V1_CONTRACT_SHA256
    assert _sha256(ROOT / meta["predecessor_vectors"]["path"]) == V1_VECTORS_SHA256


def test_active_contract_binds_epoch_nine_lock_and_fresh_snapshot_law() -> None:
    predecessor = _contract()["activation"]["predecessor_epoch_9"]

    assert predecessor["migration"]["path"] == PREDECESSOR_EPOCH_9_MIGRATION
    assert _sha256(ROOT / PREDECESSOR_EPOCH_9_MIGRATION) == predecessor["migration"]["sha256"]
    assert {
        key: value for key, value in predecessor.items() if key != "migration"
    } == EXPECTED_PREDECESSOR_EPOCH_9_LAW


def test_contract_refuses_epoch_nine_snapshot_policy_drift() -> None:
    contract = deepcopy(_contract())
    contract["activation"]["predecessor_epoch_9"]["transaction_isolation"] = "serializable"
    findings: set[Finding] = set()

    _validate_contract(contract, ROOT, findings)

    assert findings == {
        Finding(
            "predecessor_epoch9_law",
            str(CONTRACT_PATH),
            "lock, snapshot, or lifetime law differs",
        )
    }


def test_active_mise_persistence_gate_names_v2() -> None:
    mise = tomllib.loads((ROOT / ".mise.toml").read_text(encoding="utf-8"))
    gate = mise["tasks"]["check:rust-persistence-cutover"]

    assert gate == {
        "description": (
            "PER-311 active PostgreSQL 17 persistence V2 authority, envelope, and receipt contract"
        ),
        "run": "uv run --frozen python tools/verify_rust_persistence_cutover_v2.py",
    }


def test_runtime_authority_ledger_binds_active_contract_through_one_recipe() -> None:
    runtime_path = "rust/crates/babylon-persistence/src/runtime.rs"
    runtime = (ROOT / runtime_path).read_text(encoding="utf-8")
    production = runtime.split("#[cfg(test)]\nmod live_tests", maxsplit=1)[0]
    findings: set[Finding] = set()

    _validate_runtime_authority_binding(production, runtime_path, findings)

    assert findings == set()
    assert production.count("v2_authority_contract_digests(&migrations)") == 3
    assert "sha256_of(ACTIVE_V2_CUTOVER_CONTRACT)" in production
    assert "let contract_sha256 = *migrations[0].checksum().as_bytes();" not in production


def test_runtime_authority_verifier_refuses_a_divergent_call_site() -> None:
    runtime_path = "rust/crates/babylon-persistence/src/runtime.rs"
    runtime = (ROOT / runtime_path).read_text(encoding="utf-8")
    production = runtime.split("#[cfg(test)]\nmod live_tests", maxsplit=1)[0]
    divergent = production.replace(
        "v2_authority_contract_digests(&migrations)",
        "([0_u8; 32], [0_u8; 32])",
        1,
    )
    findings: set[Finding] = set()

    _validate_runtime_authority_binding(divergent, runtime_path, findings)

    assert findings == {
        Finding(
            "authority_digest_recipe",
            runtime_path,
            "expected 3 centralized call sites; got 2",
        )
    }


def test_runtime_verifier_refuses_epoch_nine_snapshot_policy_drift() -> None:
    runtime_path = "rust/crates/babylon-persistence/src/runtime.rs"
    runtime = (ROOT / runtime_path).read_text(encoding="utf-8")
    production = runtime.split("#[cfg(test)]\nmod live_tests", maxsplit=1)[0]
    divergent = production.replace("if schema_epoch == 9", "if schema_epoch == 8", 1)
    findings: set[Finding] = set()

    _validate_epoch_nine_runtime_policy(divergent, runtime_path, findings)

    assert findings == {
        Finding(
            "epoch9_snapshot_policy",
            runtime_path,
            "epoch 9 is not the exact READ COMMITTED predecessor",
        )
    }


def test_migration_verifier_refuses_missing_opaque_lock() -> None:
    migration = (ROOT / PREDECESSOR_EPOCH_9_MIGRATION).read_text(encoding="utf-8")
    divergent = migration.replace(
        "LOCK TABLE\n    babylon_state.tick_graph_row,\n    babylon_state.tick_state_row,",
        "LOCK TABLE\n    babylon_state.tick_state_row,",
        1,
    )
    findings: set[Finding] = set()

    _validate_epoch_nine_migration_policy(
        divergent,
        PREDECESSOR_EPOCH_9_MIGRATION,
        findings,
    )

    assert findings == {
        Finding(
            "epoch9_lock_census_order",
            PREDECESSOR_EPOCH_9_MIGRATION,
            "each destructive target group must lock before census and drop",
        )
    }
