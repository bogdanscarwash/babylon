"""Verify the active PER-311 PostgreSQL persistence V2 authority contract."""

from __future__ import annotations

import hashlib
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = Path("contracts/rust_persistence_cutover_v2.yaml")
MAX_CONTRACT_BYTES = 32_768

EXPECTED_TOP_LEVEL = {
    "meta",
    "authority",
    "activation",
    "tick_identity",
    "envelope",
    "events",
    "choice_receipts",
}
EXPECTED_AUTHORITY = {
    "canonical_crate": "babylon-persistence",
    "composition_module": "rust/crates/babylon-persistence/src/runtime.rs",
    "composition_type": "DurableReplayRuntimeV2",
    "prepared_tick_type": "PreparedCommittedTickV2",
    "identified_tick_input": "babylon_tick::replay_session::IdentifiedTickReportV2",
    "prepared_tick_constructor": "prepare_committed_tick_v2",
    "activation_function": "activate_rust_persistence_v2",
    "activation_report": "ActivationReportV2",
    "activation_error": "RustPersistenceActivationErrorV2",
    "runtime_error": "RustPersistenceRuntimeErrorV2",
    "committed_receipt": "CommittedTickReceiptV2",
    "retained_inner_v1": [
        "CampaignFoundationV1",
        "FoundationContentBundleV1",
        "hydrate_campaign_foundation_v1",
        "TickContentHashV1",
        "OrderedPracticeActionBatchV1",
    ],
}
EXPECTED_FAMILIES = [
    {"name": "graph", "tag_u8": 16},
    {"name": "state", "tag_u8": 17},
    {"name": "event", "tag_u8": 18},
    {"name": "choice_receipt", "tag_u8": 24},
    {"name": "checkpoint", "tag_u8": 22},
    {"name": "archive_dirty_receipt", "tag_u8": 23},
]
EXPECTED_RUST_FAMILIES = [
    "Graph",
    "State",
    "Event",
    "ChoiceReceipt",
    "Checkpoint",
    "ArchiveDirtyReceipt",
]
PREDECESSOR_EPOCH_9_MIGRATION = (
    "rust/crates/babylon-persistence/migrations/0009_rust_persistence_activation.sql"
)
EXPECTED_PREDECESSOR_EPOCH_9_LAW = {
    "transaction_isolation": "read_committed",
    "relation_lock": "access_exclusive",
    "census_snapshot": "fresh_after_lock_wait",
    "lock_lifetime": "through_census_disposition_drop_authority_row_and_commit",
}


@dataclass(frozen=True, order=True)
class Finding:
    code: str
    path: str
    detail: str


def _mapping(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{field} must be a mapping")
    return value


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _check_artifact(
    root: Path,
    row: dict[str, Any],
    label: str,
    findings: set[Finding],
) -> None:
    relative = row.get("path")
    expected = row.get("sha256")
    if not isinstance(relative, str) or not isinstance(expected, str):
        findings.add(Finding("invalid_contract", str(CONTRACT_PATH), f"{label} identity"))
        return
    path = root / relative
    if not path.is_file():
        findings.add(Finding("missing_artifact", relative, label))
        return
    actual = _sha256(path)
    if actual != expected:
        findings.add(Finding("artifact_digest", relative, f"expected {expected}; got {actual}"))


def _require_text(
    source: str,
    required: list[str],
    forbidden: list[str],
    path: str,
    findings: set[Finding],
) -> None:
    for token in required:
        if token not in source:
            findings.add(Finding("missing_v2_surface", path, token))
    for token in forbidden:
        if token in source:
            findings.add(Finding("live_v1_surface", path, token))


def _validate_runtime_authority_binding(
    source: str,
    path: str,
    findings: set[Finding],
) -> None:
    for token, detail in [
        (
            'include_bytes!("../../../../contracts/rust_persistence_cutover_v2.yaml")',
            "active V2 contract bytes not embedded",
        ),
        (
            "sha256_of(ACTIVE_V2_CUTOVER_CONTRACT)",
            "contract_sha256 does not hash the active V2 contract",
        ),
        (
            "*migrations[1].checksum().as_bytes()",
            "reader_contract_sha256 does not bind migration 0011",
        ),
    ]:
        if token not in source:
            findings.add(Finding("authority_digest_binding", path, detail))
    recipe_calls = source.count("v2_authority_contract_digests(&migrations)")
    if recipe_calls != 3:
        findings.add(
            Finding(
                "authority_digest_recipe",
                path,
                f"expected 3 centralized call sites; got {recipe_calls}",
            )
        )
    if "let contract_sha256 = *migrations[0].checksum().as_bytes();" in source:
        findings.add(
            Finding(
                "migration_as_contract",
                path,
                "migration 0010 cannot stand in for the active V2 contract",
            )
        )


def _validate_epoch_nine_runtime_policy(
    source: str,
    path: str,
    findings: set[Finding],
) -> None:
    for token, detail in [
        (
            "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
            "epoch 9 lacks a fresh per-command snapshot policy",
        ),
        (
            "fn predecessor_activation_transaction_settings_v2(schema_epoch: u16)",
            "predecessor isolation policy is not explicit",
        ),
        (
            "if schema_epoch == 9",
            "epoch 9 is not the exact READ COMMITTED predecessor",
        ),
        (
            "predecessor_activation_transaction_settings_v2(\n            authority_row.schema_epoch,",
            "predecessor execution does not select isolation by schema epoch",
        ),
    ]:
        if token not in source:
            findings.add(Finding("epoch9_snapshot_policy", path, detail))


def _validate_epoch_nine_migration_policy(
    source: str,
    path: str,
    findings: set[Finding],
) -> None:
    public_lock = source.find("'LOCK TABLE %s IN ACCESS EXCLUSIVE MODE'")
    public_count = source.find("'SELECT pg_catalog.count(*) FROM %s'")
    public_drop = source.find("DROP TABLE IF EXISTS\n    public.action_result")
    opaque_lock = source.find(
        "LOCK TABLE\n    babylon_state.tick_graph_row,\n    babylon_state.tick_state_row,"
    )
    opaque_count = source.find("(SELECT pg_catalog.count(*) FROM babylon_state.tick_graph_row)")
    opaque_drop = source.find("DROP TABLE babylon_state.tick_archive_dirty_receipt_row")
    if not (
        0 <= public_lock < public_count < public_drop
        and 0 <= opaque_lock < opaque_count < opaque_drop
    ):
        findings.add(
            Finding(
                "epoch9_lock_census_order",
                path,
                "each destructive target group must lock before census and drop",
            )
        )


def _validate_contract(contract: dict[str, Any], root: Path, findings: set[Finding]) -> None:
    if set(contract) != EXPECTED_TOP_LEVEL:
        findings.add(
            Finding(
                "contract_shape",
                str(CONTRACT_PATH),
                f"expected top-level {sorted(EXPECTED_TOP_LEVEL)}; got {sorted(contract)}",
            )
        )
        return

    meta = _mapping(contract["meta"], "meta")
    if meta.get("contract") != "RustPersistenceCutoverV2" or meta.get("version") != 2:
        findings.add(Finding("contract_identity", str(CONTRACT_PATH), "expected V2"))
    if meta.get("issue") != "PER-311":
        findings.add(Finding("contract_issue", str(CONTRACT_PATH), "expected PER-311"))
    predecessor = _mapping(meta.get("predecessor"), "meta.predecessor")
    predecessor_vectors = _mapping(meta.get("predecessor_vectors"), "meta.predecessor_vectors")
    _check_artifact(root, predecessor, "frozen V1 predecessor", findings)
    _check_artifact(root, predecessor_vectors, "frozen V1 vectors", findings)
    if predecessor_vectors.get("disposition") != "offline_historical_verification_only":
        findings.add(
            Finding("predecessor_disposition", str(CONTRACT_PATH), "V1 vectors not offline")
        )

    authority = _mapping(contract["authority"], "authority")
    if authority != EXPECTED_AUTHORITY:
        findings.add(Finding("authority_surface", str(CONTRACT_PATH), "V2 API differs"))

    activation = _mapping(contract["activation"], "activation")
    expected_activation_scalars = {
        "required_postgresql_major": 17,
        "predecessor_epoch": 9,
        "preparation_epoch": 10,
        "active_epoch": 11,
        "reader_writer": "v2_only",
        "historical_v1_decoder": "prohibited",
    }
    for field, expected in expected_activation_scalars.items():
        if activation.get(field) != expected:
            findings.add(
                Finding("activation_law", str(CONTRACT_PATH), f"{field} must be {expected!r}")
            )
    predecessor_epoch_9 = _mapping(
        activation.get("predecessor_epoch_9"), "activation.predecessor_epoch_9"
    )
    predecessor_migration = _mapping(
        predecessor_epoch_9.get("migration"),
        "activation.predecessor_epoch_9.migration",
    )
    _check_artifact(root, predecessor_migration, "epoch 9 predecessor activation", findings)
    if predecessor_migration.get("path") != PREDECESSOR_EPOCH_9_MIGRATION:
        findings.add(
            Finding(
                "predecessor_epoch9_migration",
                str(CONTRACT_PATH),
                "epoch 9 migration path differs",
            )
        )
    predecessor_law = {
        key: value for key, value in predecessor_epoch_9.items() if key != "migration"
    }
    if predecessor_law != EXPECTED_PREDECESSOR_EPOCH_9_LAW:
        findings.add(
            Finding(
                "predecessor_epoch9_law",
                str(CONTRACT_PATH),
                "lock, snapshot, or lifetime law differs",
            )
        )
    migrations = activation.get("migrations")
    if not isinstance(migrations, list) or len(migrations) != 2:
        findings.add(Finding("migration_set", str(CONTRACT_PATH), "expected epochs 10 and 11"))
    else:
        for index, row in enumerate(migrations):
            _check_artifact(
                root, _mapping(row, f"migration {index}"), f"migration {index}", findings
            )
    inventory = _mapping(activation.get("pre_activation_inventory"), "inventory")
    if inventory != {
        "read_only_before_mutation": True,
        "relation_targets": 93,
        "nonzero_incompatible_data": "typed_refusal",
    }:
        findings.add(Finding("activation_inventory", str(CONTRACT_PATH), "inventory differs"))

    tick_identity = _mapping(contract["tick_identity"], "tick_identity")
    if tick_identity != {
        "payload_type": "TickPayloadV2",
        "outer_hash": "TickContentHashV1",
        "outer_hash_codec_changed": False,
        "graph_hash_scope": "graph_only",
    }:
        findings.add(Finding("tick_identity", str(CONTRACT_PATH), "identity law differs"))

    envelope = _mapping(contract["envelope"], "envelope")
    if envelope.get("type") != "CommittedTickEnvelopeV2" or envelope.get("layout") != 2:
        findings.add(Finding("envelope_identity", str(CONTRACT_PATH), "expected V2 layout 2"))
    if envelope.get("family_order") != EXPECTED_FAMILIES:
        findings.add(
            Finding("envelope_family_order", str(CONTRACT_PATH), "six-family order differs")
        )
    for field, expected in {
        "exact_retry_identity": "complete_envelope_bytes",
        "marker_last": True,
        "post_commit_acknowledgement": "infallible_after_preflight",
        "concurrent_identical_retry": "reconcile_exact_envelope_after_campaign_lock",
    }.items():
        if envelope.get(field) != expected:
            findings.add(Finding("envelope_law", str(CONTRACT_PATH), f"{field} differs"))

    events = _mapping(contract["events"], "events")
    if events != {
        "row_type": "successful_event_v2",
        "relation": "babylon_state.tick_event_v2",
        "emitting_rule": "required",
        "choice_receipt_reference": "optional_engine_derived",
        "authored_probability_payload": "prohibited",
    }:
        findings.add(Finding("event_v2", str(CONTRACT_PATH), "event law differs"))

    receipts = _mapping(contract["choice_receipts"], "choice_receipts")
    if (
        receipts.get("type") != "ChoiceReceiptV1"
        or receipts.get("ticket_columns") != "NUMERIC(20,0)"
    ):
        findings.add(Finding("choice_receipt", str(CONTRACT_PATH), "receipt identity differs"))
    if receipts.get("family_position") != "between_event_and_checkpoint":
        findings.add(Finding("choice_receipt_order", str(CONTRACT_PATH), "family position differs"))
    if receipts.get("no_op_outcome_receipted") is not True:
        findings.add(Finding("choice_receipt_no_op", str(CONTRACT_PATH), "no-op evidence missing"))


def verify(root: Path = ROOT) -> list[Finding]:
    findings: set[Finding] = set()
    contract_file = root / CONTRACT_PATH
    if not contract_file.is_file():
        return [Finding("missing_contract", str(CONTRACT_PATH), "active V2 contract absent")]
    payload = contract_file.read_bytes()
    if len(payload) > MAX_CONTRACT_BYTES:
        return [
            Finding(
                "contract_bound",
                str(CONTRACT_PATH),
                f"{len(payload)} exceeds {MAX_CONTRACT_BYTES}",
            )
        ]
    try:
        contract = _mapping(yaml.safe_load(payload), "contract")
        _validate_contract(contract, root, findings)
    except (OSError, UnicodeDecodeError, ValueError, yaml.YAMLError) as error:
        findings.add(Finding("invalid_contract", str(CONTRACT_PATH), str(error)))
        return sorted(findings)

    runtime_path = "rust/crates/babylon-persistence/src/runtime.rs"
    runtime = (root / runtime_path).read_text(encoding="utf-8")
    _require_text(
        runtime,
        [
            "pub struct DurableReplayRuntimeV2",
            "pub struct PreparedCommittedTickV2",
            "pub struct ActivationReportV2",
            "pub enum RustPersistenceActivationErrorV2",
            "pub enum RustPersistenceRuntimeErrorV2",
            "pub struct CommittedTickReceiptV2",
            "pub fn activate_rust_persistence_v2",
            "pub fn prepare_committed_tick_v2",
            "IdentifiedTickReportV2",
            "commit_prepared_and_publish",
            "server_version_num",
        ],
        [
            "pub struct DurableReplayRuntimeV1",
            "pub struct PreparedCommittedTickV1",
            "pub fn activate_rust_persistence_v1",
            "pub fn prepare_committed_tick_v1",
        ],
        runtime_path,
        findings,
    )
    production_runtime = runtime.split("#[cfg(test)]\nmod live_tests", maxsplit=1)[0]
    _validate_runtime_authority_binding(production_runtime, runtime_path, findings)
    _validate_epoch_nine_runtime_policy(production_runtime, runtime_path, findings)
    if runtime.count("marker_matches_envelope_v2(") < 3:
        findings.add(
            Finding("retry_reconciliation", runtime_path, "missing under-lock exact recheck")
        )

    envelope_path = "rust/crates/babylon-persistence/src/committed_tick_envelope.rs"
    envelope_source = (root / envelope_path).read_text(encoding="utf-8")
    order_match = re.search(
        r"ALL_COMMITTED_TICK_ROW_FAMILIES_V2[^=]*= \[(.*?)\];",
        envelope_source,
        re.DOTALL,
    )
    rust_order = (
        []
        if order_match is None
        else re.findall(r"CommittedTickRowFamilyV2::([A-Za-z]+)", order_match.group(1))
    )
    if rust_order != EXPECTED_RUST_FAMILIES:
        findings.add(Finding("rust_family_order", envelope_path, repr(rust_order)))
    for family, tag in zip(
        EXPECTED_RUST_FAMILIES, [0x10, 0x11, 0x12, 0x18, 0x16, 0x17], strict=True
    ):
        if f"Self::{family} => 0x{tag:02x}" not in envelope_source:
            findings.add(Finding("rust_family_tag", envelope_path, family))

    replay_path = "rust/crates/babylon-tick/src/replay_identity.rs"
    replay_source = (root / replay_path).read_text(encoding="utf-8")
    _require_text(
        replay_source,
        ["pub struct TickPayloadV2"],
        ["pub struct TickPayloadV1"],
        replay_path,
        findings,
    )

    migration9_path = PREDECESSOR_EPOCH_9_MIGRATION
    migration9 = (root / migration9_path).read_text(encoding="utf-8")
    _validate_epoch_nine_migration_policy(migration9, migration9_path, findings)

    mise_path = ".mise.toml"
    mise = (root / mise_path).read_text(encoding="utf-8")
    if "python tools/verify_rust_persistence_cutover_v2.py" not in mise:
        findings.add(Finding("inactive_v2_gate", mise_path, "V2 verifier not active"))

    architecture_path = "ai/architecture.yaml"
    architecture = (root / architecture_path).read_text(encoding="utf-8")
    if "composition_root: babylon_persistence::runtime::DurableReplayRuntimeV2" not in architecture:
        findings.add(Finding("stale_architecture", architecture_path, "V2 runtime not named"))

    return sorted(findings)


def main() -> int:
    findings = verify()
    for finding in findings:
        print(f"{finding.code}: {finding.path}: {finding.detail}")
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
