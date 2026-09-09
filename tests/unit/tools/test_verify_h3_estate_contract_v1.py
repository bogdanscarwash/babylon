"""Independent checks for the PER-275 H3 estate and artifact contract."""

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
    discover_current_view_census,
    discover_persistent_table_census,
    discover_runtime_consumer_census,
    load_contract,
    main,
    verified_artifact_bytes,
    verify_artifact_manifest,
    verify_contract,
    verify_h3_vectors,
    verify_source_inventory,
)

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "contracts" / "h3_estate_contract_v1.yaml"
MANIFEST = ROOT / "data-artifacts.yaml"
VECTORS = (
    ROOT / "rust" / "crates" / "babylon-kernel" / "tests" / "fixtures" / "h3_cell_id_vectors_v1.txt"
)


def test_checked_in_contract_and_current_sources_verify() -> None:
    contract = load_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "H3EstateContractV1",
        "version": 1,
        "issue": "PER-275",
        "parent": "PER-21",
    }
    assert verify_contract(contract, ROOT) == []


def test_contract_closes_the_full_estate_and_hard_gaps() -> None:
    contract = load_contract(CONTRACT)

    assert len(contract["estate"]["persistent_tables"]) == 15
    assert len(contract["estate"]["current_views"]) == 10
    assert {row["name"] for row in contract["estate"]["temporary_shapes"]} == {
        "_hex_spatial_map_tmp",
        "_hex_state_tmp",
    }
    assert contract["estate"]["unused_domain"]["name"] == "h3index"
    assert len(contract["estate"]["runtime_consumer_census"]) == 31
    assert {row["kind"] for row in contract["hard_gaps"]} == {
        "census_place_identity",
        "census_place_geometry",
        "county_place_h3_overlap",
    }


def test_retired_legacy_runtime_read_write_triples_are_absent() -> None:
    contract = load_contract(CONTRACT)
    consumers = {
        (row["path"], row["relation"], row["access"])
        for row in contract["estate"]["runtime_consumer_census"]
    }
    legacy = "src/babylon/persistence/postgres_runtime/_legacy.py"
    retired = {
        (legacy, "hex_activity", "read"),
        (legacy, "hex_activity", "write"),
        (legacy, "hex_cell", "read"),
        (legacy, "hex_latest", "write"),
        (legacy, "hex_map", "read"),
        (legacy, "hex_state", "read"),
        (legacy, "hex_state", "write"),
        (legacy, "hex_terrain_state", "write"),
        (legacy, "infrastructure_link_state", "read"),
        (legacy, "infrastructure_link_state", "write"),
    }

    assert consumers.isdisjoint(retired)
    assert contract["vectors"]["path"] == (
        "rust/crates/babylon-kernel/tests/fixtures/h3_cell_id_vectors_v1.txt"
    )


def test_catalog_census_detects_catalog_addition_and_removal(tmp_path: Path) -> None:
    persistence = tmp_path / "src" / "babylon" / "persistence"
    migrations = persistence / "migrations"
    migrations.mkdir(parents=True)
    (persistence / "postgres_schema.py").write_text("", encoding="utf-8")
    migration = migrations / "9999_contract_probe.sql"
    migration.write_text(
        """CREATE TABLE contract_probe (
    h3_index TEXT NOT NULL
);
CREATE VIEW v_contract_probe AS SELECT h3_index FROM contract_probe;
""",
        encoding="utf-8",
    )
    contract = load_contract(CONTRACT)

    assert discover_persistent_table_census(tmp_path) == {
        "contract_probe": {
            "identity_fields": {"h3_index": "TEXT"},
            "tagged_discriminators": {},
        }
    }
    assert discover_current_view_census(contract, tmp_path) == {"v_contract_probe"}

    migration.unlink()
    assert discover_persistent_table_census(tmp_path) == {}
    assert discover_current_view_census(contract, tmp_path) == set()


def test_catalog_census_applies_later_migration_changes_in_order(tmp_path: Path) -> None:
    persistence = tmp_path / "src" / "babylon" / "persistence"
    migrations = persistence / "migrations"
    migrations.mkdir(parents=True)
    (persistence / "postgres_schema.py").write_text("", encoding="utf-8")
    (migrations / "0001_contract_probe.sql").write_text(
        """CREATE TABLE contract_probe (
    h3_index TEXT NOT NULL
);
CREATE VIEW v_contract_probe AS SELECT h3_index FROM contract_probe;
""",
        encoding="utf-8",
    )
    (migrations / "0002_commented_out_change.sql").write_text(
        """-- DROP TABLE contract_probe;
-- DROP VIEW v_contract_probe;
-- ALTER TABLE contract_probe ALTER COLUMN h3_index TYPE BIGINT;
""",
        encoding="utf-8",
    )
    contract = load_contract(CONTRACT)
    assert discover_persistent_table_census(tmp_path) == {
        "contract_probe": {
            "identity_fields": {"h3_index": "TEXT"},
            "tagged_discriminators": {},
        }
    }
    assert discover_current_view_census(contract, tmp_path) == {"v_contract_probe"}

    later = migrations / "0003_contract_probe_change.sql"
    later.write_text(
        """ALTER TABLE contract_probe ALTER COLUMN h3_index TYPE VARCHAR(16);
DROP VIEW v_contract_probe;
""",
        encoding="utf-8",
    )
    assert discover_persistent_table_census(tmp_path) == {
        "contract_probe": {
            "identity_fields": {"h3_index": "VARCHAR(16)"},
            "tagged_discriminators": {},
        }
    }
    assert discover_current_view_census(contract, tmp_path) == set()

    later.write_text("DROP TABLE contract_probe;\n", encoding="utf-8")
    assert discover_persistent_table_census(tmp_path) == {}


def test_historical_table_census_preserves_tagged_destination_discriminator() -> None:
    contract = load_contract(CONTRACT)
    table = next(
        row
        for row in contract["estate"]["persistent_tables"]
        if row["name"] == "immutable_reference_lodes_od_matrix"
    )

    assert table["tag_field"] == "workplace_dest_kind"
    assert table["tag_legacy_type"] == "TEXT"
    assert table["tag_allowed_values"] == ["external", "hex"]


def test_table_census_applies_later_tag_constraint_changes(tmp_path: Path) -> None:
    persistence = tmp_path / "src" / "babylon" / "persistence"
    migrations = persistence / "migrations"
    migrations.mkdir(parents=True)
    (persistence / "postgres_schema.py").write_text("", encoding="utf-8")
    (migrations / "0001_tagged_destination.sql").write_text(
        """CREATE TABLE contract_probe (
    home_hex TEXT NOT NULL,
    workplace_dest TEXT NOT NULL,
    workplace_dest_kind TEXT NOT NULL
        CHECK (workplace_dest_kind IN ('hex', 'external'))
);
""",
        encoding="utf-8",
    )
    (migrations / "0002_widen_tag.sql").write_text(
        """ALTER TABLE contract_probe
DROP CONSTRAINT contract_probe_workplace_dest_kind_check;
ALTER TABLE contract_probe
ADD CONSTRAINT contract_probe_workplace_dest_kind_check
CHECK (workplace_dest_kind IN ('hex', 'external', 'unknown'));
""",
        encoding="utf-8",
    )

    census = discover_persistent_table_census(tmp_path)

    assert census["contract_probe"]["tagged_discriminators"] == {
        "workplace_dest_kind": {
            "legacy_type": "TEXT",
            "allowed_values": ["external", "hex", "unknown"],
        }
    }


def test_terminal_runtime_consumer_census_excludes_retired_archive_cli() -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    contract["estate"]["partition"]["default_child"] = "dynamic_hex_state_default"

    consumers = discover_runtime_consumer_census(contract, ROOT)

    assert consumers == []
    assert not (ROOT / "tools/archive_sessions.py").exists()


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


def test_source_inventory_refuses_a_missing_persistent_table() -> None:
    contract = load_contract(CONTRACT)
    contract = copy.deepcopy(contract)
    contract["estate"]["persistent_tables"].pop()

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_source_inventory(contract, ROOT)

    assert exc_info.value.code == "persistent_table_census"


def test_source_inventory_refuses_a_missing_current_view() -> None:
    contract = load_contract(CONTRACT)
    contract = copy.deepcopy(contract)
    contract["estate"]["current_views"].pop()

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_source_inventory(contract, ROOT)

    assert exc_info.value.code == "view_census"


def test_source_inventory_refuses_runtime_consumer_drift() -> None:
    contract = load_contract(CONTRACT)
    contract = copy.deepcopy(contract)
    contract["estate"]["runtime_consumer_census"].pop()

    with pytest.raises(H3EstateContractRefusal) as exc_info:
        verify_source_inventory(contract, ROOT)

    assert exc_info.value.code == "runtime_consumer_census"


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
