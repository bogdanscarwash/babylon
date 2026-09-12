"""Independent checks for the pinned H3 reference predecessor and artifacts."""

from __future__ import annotations

import copy
import hashlib
from pathlib import Path

import pytest
from tools.verify_h3_estate_contract_v1 import (
    H3EstateContractRefusal,
    canonical_contract_digest,
    checked_count,
    checked_land_fraction,
    load_contract,
    main,
    verified_artifact_bytes,
    verify_artifact_manifest,
    verify_contract,
    verify_h3_vectors,
)

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "contracts" / "h3_estate_contract_v1.yaml"
MANIFEST = ROOT / "data-artifacts.yaml"
VECTORS = (
    ROOT / "rust" / "crates" / "babylon-kernel" / "tests" / "fixtures" / "h3_cell_id_vectors_v1.txt"
)


def test_checked_in_reference_predecessor_and_artifacts_verify() -> None:
    contract = load_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "H3EstateContractV1",
        "version": 1,
        "issue": "PER-275",
        "parent": "PER-21",
    }
    assert verify_contract(contract, ROOT) == []


def test_reference_predecessor_refuses_changed_evidence_even_with_valid_shape() -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    contract["artifacts"][0]["evidence_class"] = "changed-evidence"

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_contract(contract, ROOT)

    assert exc_info.value.code == "historical_contract_digest"


def test_contract_loader_refuses_duplicate_mapping_keys(tmp_path: Path) -> None:
    path = tmp_path / "duplicate.yaml"
    path.write_text("meta: first\nmeta: second\n", encoding="utf-8")

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        load_contract(path)

    assert exc_info.value.code == "invalid_contract"


@pytest.mark.parametrize(
    "extra_gap",
    [
        None,
        {
            "kind": "census_place_identity",
            "status": "blocking",
            "required_authority": "duplicate contradiction",
        },
    ],
)
def test_contract_refuses_malformed_or_duplicate_hard_gaps(extra_gap: object) -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    contract["hard_gaps"].append(extra_gap)

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_contract(contract, ROOT)

    assert exc_info.value.code == "contract_shape"


def test_artifact_ledger_matches_the_versioned_manifest() -> None:
    contract = load_contract(CONTRACT)

    verify_artifact_manifest(contract, MANIFEST)


def test_artifact_manifest_drift_refuses() -> None:
    contract = load_contract(CONTRACT)
    contract = copy.deepcopy(contract)
    population = next(row for row in contract["artifacts"] if row["name"] == "h3_res7_population")
    population["rows"] += 1

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_artifact_manifest(contract, MANIFEST)

    assert exc_info.value.code == "artifact_manifest_drift"


def test_artifact_bytes_are_hash_proved_before_decode(tmp_path: Path) -> None:
    path = tmp_path / "artifact.parquet"
    payload = b"not parquet, but independently pinned"
    path.write_bytes(payload)

    assert (
        verified_artifact_bytes(path, len(payload), hashlib.sha256(payload).hexdigest()) == payload
    )

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verified_artifact_bytes(path, len(payload), "0" * 64)

    assert exc_info.value.code == "artifact_bytes"


def test_python_executes_the_shared_rust_sql_vector_bytes() -> None:
    contract = load_contract(CONTRACT)

    receipt = verify_h3_vectors(contract, VECTORS)

    assert receipt == {
        "valid": 208,
        "pentagons": 192,
        "invalid_raw": 6,
        "invalid_sql": 1,
        "invalid_text": 6,
        "invalid_ancestor": 2,
    }


@pytest.mark.parametrize("value", [-1.0, 1.5, float("inf"), float("nan")])
def test_count_contract_refuses_negative_fractional_or_nonfinite_values(value: float) -> None:
    with pytest.raises(H3EstateContractRefusal) as exc_info:
        checked_count(value)

    assert exc_info.value.code == "invalid_count"


def test_count_contract_preserves_u64_and_refuses_unsafe_float() -> None:
    assert checked_count((1 << 64) - 1) == (1 << 64) - 1

    for value in (1 << 64, float((1 << 53) + 2)):
        with pytest.raises(H3EstateContractRefusal) as exc_info:
            checked_count(value)

        assert exc_info.value.code == "invalid_count"


@pytest.mark.parametrize("value", [-0.000001, 1.000001, float("inf"), float("nan")])
def test_land_fraction_refuses_out_of_range_or_nonfinite_values(value: float) -> None:
    with pytest.raises(H3EstateContractRefusal) as exc_info:
        checked_land_fraction(value, scale=6)

    assert exc_info.value.code == "invalid_land_fraction"


def test_land_fraction_refuses_more_than_six_decimal_places() -> None:
    with pytest.raises(H3EstateContractRefusal) as exc_info:
        checked_land_fraction(0.1234567, scale=6)

    assert exc_info.value.code == "land_fraction_scale"


def test_handoff_digest_ignores_yaml_mapping_order_but_not_semantics() -> None:
    contract = load_contract(CONTRACT)
    reordered = {key: contract[key] for key in reversed(contract)}

    assert canonical_contract_digest(reordered) == canonical_contract_digest(contract)

    changed = copy.deepcopy(contract)
    changed["migration_handoff"]["post_per20_epoch"]["expected"] = 6
    assert canonical_contract_digest(changed) != canonical_contract_digest(contract)


def test_cli_verifies_contract_without_downloading_artifacts(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    monkeypatch.setattr(
        "sys.argv",
        [
            "verify_h3_estate_contract_v1.py",
            "--contract",
            str(CONTRACT),
            "--repo-root",
            str(ROOT),
        ],
    )

    assert main() == 0
    output = capsys.readouterr().out
    assert "H3EstateContractV1 verified" in output
    assert "artifact bytes: not requested" in output
