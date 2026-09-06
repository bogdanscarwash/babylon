"""Independent behavioral checks for ArchiveFoundationGrantsV1."""

from __future__ import annotations

from pathlib import Path

import pytest
from tools.verify_archive_foundation_grants_v1 import (
    COMPILED_CONSTANTS,
    FIXTURE_PARTS,
    FoundationGrantsRefusal,
    build_grant_rows,
    compute_semantic_sha256,
    load_concept_ids,
    load_contract,
    load_spatial_subjects,
    main,
    verify_all,
)

ROOT = Path(__file__).resolve().parents[3]
SCHEMA = ROOT / "contracts" / "archive_foundation_grants_v1.yaml"
GLOSSARY_FIXTURE = ROOT / "contracts" / "fixtures" / "glossary_concepts_v1.jsonl"


def _subjects() -> tuple[list[tuple[str, str]], list[str], list[str]]:
    counties, places = load_spatial_subjects(ROOT)
    concept_ids = load_concept_ids(GLOSSARY_FIXTURE)
    return counties, places, concept_ids


def test_shared_contract_verifies_independently() -> None:
    contract = load_contract(SCHEMA)
    counties, places, concept_ids = _subjects()

    assert contract["meta"]["contract"] == "ArchiveFoundationGrantsV1"
    assert verify_all(contract, counties, places, concept_ids) == []


def test_grant_row_census_covers_exactly_the_public_reference_subjects() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)

    assert len(counties) == 3285
    assert len(places) == 745
    assert len(concept_ids) == 8
    assert len(rows) == COMPILED_CONSTANTS["expected_grant_rows"] == 2832
    kinds = [row[0] for row in rows]
    assert kinds.count("county") == 7 * 83
    assert kinds.count("place") == 3 * 745
    assert kinds.count("concept") == 2 * 8


def test_rows_sort_to_the_canonical_order() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)

    assert rows == sorted(rows)
    assert rows[0][0] == "concept"
    assert rows[-1][0] == "place"


def test_semantic_digest_matches_the_contract_and_rust_pin() -> None:
    contract = load_contract(SCHEMA)
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)

    assert compute_semantic_sha256(rows) == contract["constants"]["semantic_sha256"]
    assert (
        compute_semantic_sha256(rows)
        == "9bb3cdda37dc3dee59242fcfe08f6c3f5bac01ff433988383d459fa0df9dfc65"
    )


def test_residual_and_non_michigan_counties_seed_nothing() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)
    county_subjects = {row[1] for row in rows if row[0] == "county"}

    assert "26999" not in county_subjects
    assert all(geoid.startswith("26") for geoid, _ in counties if geoid in county_subjects)
    assert len(county_subjects) == 83


def test_earned_magnitude_keys_never_seed() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)

    for forbidden in ["median-wage", "phi-hour", "class-composition", "employment"]:
        assert all(row[2] != forbidden for row in rows)


def test_all_counties_receive_exact_public_qcew_baseline_grants() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)
    baseline = [row for row in rows if row[2].startswith("qcew-")]

    assert len(baseline) == 4 * 83
    for kind, geoid, key, source_id, locator in baseline:
        assert kind == "county"
        assert key in {
            "qcew-establishments",
            "qcew-employment",
            "qcew-total-annual-wages",
            "qcew-average-weekly-wage",
        }
        assert source_id == "qcew-county-economics-v1"
        assert locator == (
            f"qcew_county_economics_mi_2024.csv.gz#county_geoid={geoid}"
            "&sha256=116affb2998c6c0259d5bf14840f99f835d7e0733aa0b4f4c60a257b2723cd16"
        )


def test_place_containment_rows_carry_the_overlap_citation() -> None:
    counties, places, concept_ids = _subjects()
    rows = build_grant_rows(counties, places, concept_ids)

    for kind, _subject_id, grant_key, source_id, locator in rows:
        if kind == "place" and grant_key == "containment":
            assert source_id == "county-place-h3-overlap-v1"
            assert locator.startswith(
                "census_county_place_h3_land_overlap_mi_2023.parquet#place_geoid="
            )
        if kind == "place" and grant_key != "containment":
            assert source_id == "census-place-authority-v1"


def test_spatial_fixture_digest_drift_refuses(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(
        "tools.verify_archive_foundation_grants_v1.COMPILED_CONSTANTS",
        {**COMPILED_CONSTANTS, "spatial_fixture_digest": "00" * 32},
    )

    with pytest.raises(FoundationGrantsRefusal) as exc_info:
        load_spatial_subjects(ROOT)

    assert exc_info.value.code == "spatial_fixture_digest"


def test_glossary_fixture_digest_drift_refuses(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(
        "tools.verify_archive_foundation_grants_v1.COMPILED_CONSTANTS",
        {**COMPILED_CONSTANTS, "glossary_fixture_sha256": "00" * 32},
    )

    with pytest.raises(FoundationGrantsRefusal) as exc_info:
        load_concept_ids(GLOSSARY_FIXTURE)

    assert exc_info.value.code == "glossary_fixture_digest"


def test_compiled_contract_drift_refuses() -> None:
    contract = load_contract(SCHEMA)
    contract["constants"]["grant_tick"] = 1
    counties, places, concept_ids = _subjects()

    with pytest.raises(FoundationGrantsRefusal) as exc_info:
        verify_all(contract, counties, places, concept_ids)

    assert exc_info.value.code == "compiled_contract_drift"


def test_missing_michigan_county_refuses_in_the_census() -> None:
    contract = load_contract(SCHEMA)
    counties, places, concept_ids = _subjects()
    counties = [row for row in counties if row[0] != "26001"]

    errors = verify_all(contract, counties, places, concept_ids)

    assert any("county census" in error for error in errors)


def test_malformed_spatial_fixture_refuses(tmp_path: Path) -> None:
    part_path = tmp_path / "spatial_reference_products_v1.part-00.bin"
    part_path.write_bytes(b"not the fixture")
    other_parts = [ROOT / part for part in FIXTURE_PARTS[1:]]
    monkeypatch_parts = [part_path, *other_parts]

    import tools.verify_archive_foundation_grants_v1 as verifier

    original = verifier.FIXTURE_PARTS
    verifier.FIXTURE_PARTS = [str(path) for path in monkeypatch_parts]
    try:
        with pytest.raises(FoundationGrantsRefusal) as exc_info:
            verifier.load_spatial_subjects(tmp_path)
    finally:
        verifier.FIXTURE_PARTS = original

    assert exc_info.value.code == "spatial_fixture_digest"


def test_cli_verifies_the_repository_corpus(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr("sys.argv", ["verify_archive_foundation_grants_v1.py"])

    assert main() == 0


def test_cli_prints_typed_refusal_without_traceback(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    schema_path = tmp_path / "drift.yaml"
    contract = load_contract(SCHEMA)
    contract["constants"]["expected_places"] = 1
    import yaml

    schema_path.write_text(yaml.safe_dump(contract), encoding="utf-8")
    monkeypatch.setattr(
        "tools.verify_archive_foundation_grants_v1.COMPILED_CONSTANTS",
        {**COMPILED_CONSTANTS, "expected_places": 1},
    )
    monkeypatch.setattr(
        "sys.argv",
        ["verify_archive_foundation_grants_v1.py", "--schema", str(schema_path)],
    )

    assert main() == 1
    assert "place census" in capsys.readouterr().out


def test_fixture_parts_are_pinned_relative_paths() -> None:
    for part in FIXTURE_PARTS:
        candidate = ROOT / part
        assert candidate.is_file(), part
        assert len(candidate.read_bytes()) > 0
