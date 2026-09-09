"""Commodity eligibility contracts independent of the local acquisition directory."""

from __future__ import annotations

import copy
import csv
import gzip
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parents[3]
ARTIFACT = ROOT / "src/babylon/data/reference/economy/michigan_commodity_roster_mi_2024.json.gz"
sys.path.insert(0, str(ROOT / "tools"))

import make_michigan_commodity_roster as builder  # type: ignore[import-not-found]  # noqa: E402
import make_qcew_county_economics_artifacts as county  # type: ignore[import-not-found]  # noqa: E402
import verify_michigan_commodity_roster_v1 as verifier  # type: ignore[import-not-found]  # noqa: E402


def source_row(code: str, *, geoid: str = "26001", **changes: str) -> dict[str, str]:
    row = dict.fromkeys(county.EXPECTED_SOURCE_COLUMNS, "0")
    row.update(
        area_fips=geoid,
        own_code="5",
        industry_code=code,
        agglvl_code="74" if code in builder.COMMODITY_SECTORS else str(72 + len(code)),
        size_code="0",
        year="2024",
        qtr="A",
        disclosure_code="",
        industry_title=f"NAICS {code} Fixture industry",
        annual_avg_estabs_count="7",
        annual_avg_emplvl="81",
        total_annual_wages="10954108416",
        annual_avg_wkly_wage="2100",
    )
    row.update(changes)
    return row


def select(rows: list[dict[str, str]]) -> builder.CountySelection:
    return builder.canonicalize_county("26001", county.EXPECTED_SOURCE_COLUMNS, rows)


def test_checked_roster_admits_397_owners_and_preserves_upstream_alternatives() -> None:
    document = json.loads(gzip.decompress(ARTIFACT.read_bytes()))
    actors = document["actors"]
    assert len(actors) == 397
    assert Counter(actor["sector_code"] for actor in actors) == {
        "11": 83,
        "21": 65,
        "31-33": 83,
        "42": 83,
        "44-45": 83,
    }
    assert len({actor["county_geoid"] for actor in actors}) == 83
    assert document["excluded_cohorts"] == [
        {"county_geoid": geoid, "sector_code": "21", "reason": "support-only"}
        for geoid in ("26011", "26019", "26039", "26089", "26119")
    ]
    assert Counter(actor["primary_family"] for actor in actors if actor["role"] == "producer") == {
        "grain": 51,
        "animal_products": 9,
        "logs": 23,
        "hydrocarbon_feedstock": 10,
        "metal_ore": 2,
        "mineral_feedstock": 53,
        "prepared_food": 9,
        "packaged_beverages": 4,
        "wood_products": 10,
        "industrial_chemicals": 1,
        "metal_parts": 38,
        "machinery": 16,
        "electrical_goods": 1,
        "household_wares": 4,
    }
    assert sum("paper_packaging" in actor["eligible_families"] for actor in actors) == 39
    assert sum("metal_stock" in actor["eligible_families"] for actor in actors) == 54
    assert all(actor["primary_family"] is None for actor in actors if actor["role"] != "producer")


def test_primary_ties_use_exact_naics_and_support_does_not_outvote_production() -> None:
    result = select(
        [
            source_row("11"),
            source_row("111"),
            source_row("113"),
            source_row("115", annual_avg_estabs_count="1000"),
        ]
    )
    (actor,) = result.actors
    assert actor.primary_family == "grain"
    assert actor.eligible_families == ("grain", "logs")
    assert [row.industry_code for row in result.industries] == ["111", "113", "115"]


@pytest.mark.parametrize(("sector", "support"), [("11", "115"), ("21", "213"), ("42", "425")])
def test_support_only_cells_stay_context_instead_of_becoming_commodity_actors(
    sector: str, support: str
) -> None:
    result = select([source_row(sector), source_row(support)])
    assert result.actors == ()
    assert result.excluded_cohorts == (builder.ExcludedCohort("26001", sector, "support-only"),)


def test_mining_chooses_parent_then_child_without_adding_nested_establishments() -> None:
    result = select(
        [
            source_row("21"),
            source_row("211", annual_avg_estabs_count="7"),
            source_row("212", annual_avg_estabs_count="8"),
            source_row("2122", annual_avg_estabs_count="3"),
            source_row("2123", annual_avg_estabs_count="4"),
        ]
    )
    (actor,) = result.actors
    assert actor.primary_family == "mineral_feedstock"
    assert actor.eligible_families == ("hydrocarbon_feedstock", "metal_ore", "mineral_feedstock")


@pytest.mark.parametrize(("sector", "code"), [("11", "114"), ("31-33", "335")])
def test_unmapped_primary_refuses_instead_of_silently_selecting_a_smaller_industry(
    sector: str, code: str
) -> None:
    with pytest.raises(county.QcewBuildError, match="source_unmapped_primary"):
        select([source_row(sector), source_row(code)])


def test_suppression_is_absence_while_disclosed_zero_remains_zero() -> None:
    result = select(
        [
            source_row("31-33"),
            source_row(
                "332",
                disclosure_code="N",
                annual_avg_emplvl="0",
                total_annual_wages="0",
                annual_avg_wkly_wage="0",
            ),
            source_row("331", annual_avg_estabs_count="2", annual_avg_emplvl="0"),
        ]
    )
    metal, fabrication = result.industries
    assert fabrication.annual_avg_estabs_count == 7
    assert (
        fabrication.annual_avg_emplvl,
        fabrication.total_annual_wages,
        fabrication.annual_avg_wkly_wage,
    ) == (None, None, None)
    assert metal.annual_avg_emplvl == 0
    assert metal.total_annual_wages == 10_954_108_416
    assert result.actors[0].primary_family == "metal_parts"


def test_source_row_permutations_preserve_selection_and_semantic_bytes() -> None:
    rows = [source_row(code) for code in ("11", "111", "113", "42", "423", "424", "425")]
    first = select(rows)
    assert first == select(list(reversed(rows)))
    assert first == select(rows[3:] + rows[:3])
    assert builder.canonical_json(first) == builder.canonical_json(select(list(reversed(rows))))


@pytest.mark.parametrize(
    ("changes", "code"),
    [
        ({"area_fips": "26003"}, "source_geoid"),
        ({"year": "2023"}, "source_row_identity"),
        ({"disclosure_code": "X"}, "source_disclosure"),
        ({"disclosure_code": "N"}, "source_suppressed_value"),
        ({"annual_avg_estabs_count": "1.5"}, "source_value"),
        ({"annual_avg_emplvl": "-1"}, "source_value"),
        ({"total_annual_wages": str(2**63)}, "source_value"),
    ],
)
def test_malformed_selected_source_refuses(changes: dict[str, str], code: str) -> None:
    with pytest.raises(county.QcewBuildError, match=code):
        select([source_row("11"), source_row("111", **changes)])


def test_duplicate_detail_refuses_even_when_the_duplicate_has_zero_establishments() -> None:
    with pytest.raises(county.QcewBuildError, match="source_duplicate_industry"):
        select(
            [source_row("11"), source_row("111"), source_row("111", annual_avg_estabs_count="0")]
        )


@pytest.fixture
def sources(tmp_path: Path) -> tuple[Path, Path]:
    source_dir = tmp_path / "sources"
    source_dir.mkdir()
    entries = []
    for geoid in county.MICHIGAN_COUNTY_GEOIDS:
        name = f"2024.annual {geoid} Fixture {geoid} County, Michigan.csv"
        path = source_dir / name
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=county.EXPECTED_SOURCE_COLUMNS)
            writer.writeheader()
            writer.writerows([source_row(code, geoid=geoid) for code in ("11", "111", "113")])
        entries.append({"file": name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps({"contract": "QcewCountyEconomicsV1", "version": 1, "entries": entries})
    )
    return source_dir, manifest


def test_rebuild_is_byte_deterministic_and_binds_each_county_source(
    tmp_path: Path, sources: tuple[Path, Path]
) -> None:
    source_dir, manifest = sources
    first, second = tmp_path / "first.json.gz", tmp_path / "second.json.gz"
    stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=first)
    assert builder.build(source_dir=source_dir, source_manifest=manifest, out_path=second) == stats
    assert first.read_bytes() == second.read_bytes()
    document = json.loads(gzip.decompress(first.read_bytes()))
    assert len(document["actors"]) == 83
    assert document["sources"] == [
        {"county_geoid": geoid, **entry}
        for geoid, entry in zip(
            county.MICHIGAN_COUNTY_GEOIDS, county.load_source_manifest(manifest), strict=True
        )
    ]
    assert document["source_manifest_sha256"] == hashlib.sha256(manifest.read_bytes()).hexdigest()


def test_hash_refusal_precedes_csv_parsing_and_preserves_existing_output(
    tmp_path: Path, sources: tuple[Path, Path]
) -> None:
    source_dir, manifest = sources
    path = source_dir / county.load_source_manifest(manifest)[0]["file"]
    path.write_bytes(b"malformed and unpinned")
    output = tmp_path / "roster.json.gz"
    output.write_bytes(b"preserve previous artifact")
    with pytest.raises(county.QcewBuildError, match="source_sha256"):
        builder.build(source_dir=source_dir, source_manifest=manifest, out_path=output)
    assert output.read_bytes() == b"preserve previous artifact"


def test_checked_contract_verifies_without_raw_sources_or_writes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def refuse(*args: Any, **kwargs: Any) -> None:
        pytest.fail("ordinary verification must not read raw acquisition or write")

    monkeypatch.setattr(builder, "build", refuse)
    monkeypatch.setattr(county, "verify_source_file", refuse)
    monkeypatch.setattr(Path, "write_bytes", refuse)
    monkeypatch.setattr(Path, "write_text", refuse)
    assert verifier.main(["--repo-root", str(ROOT)]) == 0


@pytest.mark.parametrize(
    "mutation", ["suppressed_zero", "unknown_family", "duplicate_actor", "source_pin"]
)
def test_verifier_refuses_semantic_corruption_independently_of_artifact_hash(mutation: str) -> None:
    document = copy.deepcopy(json.loads(gzip.decompress(ARTIFACT.read_bytes())))
    if mutation == "suppressed_zero":
        row = next(row for row in document["industries"] if row["disclosure_code"] == "N")
        row["annual_avg_emplvl"] = 0
    elif mutation == "unknown_family":
        document["actors"][0]["primary_family"] = "invented_observed_output"
    elif mutation == "duplicate_actor":
        document["actors"].append(document["actors"][0])
    else:
        document["sources"][0]["sha256"] = "0" * 64
    with pytest.raises(county.QcewBuildError):
        verifier.verify_document(document, ROOT)
