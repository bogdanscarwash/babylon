"""County-sector source and artifact contracts without the local raw-data trove."""

from __future__ import annotations

import copy
import csv
import gzip
import hashlib
import io
import json
import shutil
import sys
from pathlib import Path
from typing import Any

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))

import make_qcew_county_economics_artifacts as county  # type: ignore[import-not-found]  # noqa: E402
import make_qcew_county_sector_artifacts as builder  # type: ignore[import-not-found]  # noqa: E402
import verify_qcew_county_sectors_v1 as verifier  # type: ignore[import-not-found]  # noqa: E402

CONTRACT = ROOT / "contracts/qcew_county_sectors_v1.yaml"


def source_row(geoid: str = "26001", sector: str = "31-33", **changes: str) -> dict[str, str]:
    row = dict.fromkeys(county.EXPECTED_SOURCE_COLUMNS, "0")
    row.update(
        area_fips=geoid,
        own_code="5",
        industry_code=sector,
        agglvl_code="74",
        size_code="0",
        year="2024",
        qtr="A",
        disclosure_code="",
        industry_title=f"{sector} Fixture sector",
        annual_avg_estabs_count="7",
        annual_avg_emplvl="81",
        total_annual_wages="10954108416",
        annual_avg_wkly_wage="2100",
    )
    row.update(changes)
    return row


def canonicalize(rows: list[dict[str, str]]) -> tuple[builder.SectorRow, ...]:
    return builder.canonicalize_sector_rows("26001", county.EXPECTED_SOURCE_COLUMNS, rows)


def test_suppression_preserves_establishments_but_never_manufactures_zero_jobs() -> None:
    row = source_row(
        disclosure_code="N",
        annual_avg_emplvl="0",
        total_annual_wages="0",
        annual_avg_wkly_wage="0",
    )
    (result,) = canonicalize([row])
    assert result.annual_avg_estabs_count == 7
    assert result.annual_avg_emplvl is None
    assert result.total_annual_wages is None
    assert result.annual_avg_wkly_wage is None
    assert result.disclosure_code == "N"
    (public_zero,) = canonicalize([source_row(annual_avg_emplvl="0")])
    assert public_zero.annual_avg_emplvl == 0


def test_composite_codes_and_unclassified_are_preserved_without_rollup() -> None:
    rows = [source_row(sector=code) for code in ("99", "48-49", "44-45", "31-33")]
    results = canonicalize(rows)
    assert [row.sector_code for row in results] == ["31-33", "44-45", "48-49", "99"]
    assert [row.sector_disposition for row in results] == ["classified"] * 3 + ["unclassified"]
    assert all(row.total_annual_wages == 10_954_108_416 for row in results)
    assert all(row.sector_title.endswith("Fixture sector") for row in results)


def test_selection_excludes_other_ownership_levels_and_zero_establishments() -> None:
    rows = [
        source_row(),
        source_row(sector="11", annual_avg_estabs_count="0"),
        source_row(own_code="0"),
        source_row(agglvl_code="70"),
        source_row(sector="331", agglvl_code="75"),
    ]
    assert [row.sector_code for row in canonicalize(rows)] == ["31-33"]


@pytest.mark.parametrize(
    ("changes", "code"),
    [
        ({"area_fips": "26003"}, "source_geoid"),
        ({"year": "2023"}, "source_row_identity"),
        ({"qtr": "1"}, "source_row_identity"),
        ({"size_code": "1"}, "source_row_identity"),
        ({"industry_code": "31"}, "source_sector"),
        ({"disclosure_code": "X"}, "source_disclosure"),
        ({"disclosure_code": "N"}, "source_suppressed_value"),
        ({"industry_title": ""}, "source_title"),
        ({"industry_title": "hidden\nline"}, "source_title"),
        ({"total_annual_wages": str(2**63)}, "source_value"),
        ({"annual_avg_emplvl": "-1"}, "source_value"),
        ({"annual_avg_emplvl": "1.5"}, "source_value"),
        ({"annual_avg_emplvl": "NaN"}, "source_value"),
        ({"annual_avg_emplvl": "１２"}, "source_value"),
    ],
)
def test_selected_source_faults_refuse(changes: dict[str, str], code: str) -> None:
    with pytest.raises(county.QcewBuildError, match=code):
        canonicalize([source_row(**changes)])


def test_duplicate_sector_and_malformed_header_refuse() -> None:
    with pytest.raises(county.QcewBuildError, match="source_duplicate_sector"):
        canonicalize([source_row(), source_row()])
    with pytest.raises(county.QcewBuildError, match="source_schema"):
        builder.canonicalize_sector_rows("26001", ["area_fips"], [source_row()])


@pytest.fixture
def sector_sources(tmp_path: Path) -> tuple[Path, Path]:
    """All 83 tiny CSVs are generated and pinned here, independent of raw acquisition."""
    source_dir = tmp_path / "sources"
    source_dir.mkdir()
    entries = []
    for index, geoid in enumerate(county.MICHIGAN_COUNTY_GEOIDS):
        rows = [source_row(geoid, "31-33", annual_avg_emplvl=str(index))]
        rows.append(source_row(geoid, "44-45"))
        if index != 0:
            rows.append(
                source_row(
                    geoid,
                    "99",
                    disclosure_code="N",
                    annual_avg_emplvl="0",
                    total_annual_wages="0",
                    annual_avg_wkly_wage="0",
                )
            )
        name = f"2024.annual {geoid} Fixture {geoid} County, Michigan.csv"
        path = source_dir / name
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=county.EXPECTED_SOURCE_COLUMNS)
            writer.writeheader()
            writer.writerows(rows)
        entries.append({"file": name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
    manifest = tmp_path / "source-manifest.json"
    manifest.write_text(
        json.dumps({"contract": "QcewCountyEconomicsV1", "version": 1, "entries": entries}),
        encoding="utf-8",
    )
    return source_dir, manifest


def test_source_build_is_deterministic_and_keeps_absence_and_lineage(
    tmp_path: Path, sector_sources: tuple[Path, Path]
) -> None:
    source_dir, manifest = sector_sources
    first, second = tmp_path / "first.csv.gz", tmp_path / "second.csv.gz"
    stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=first)
    assert builder.build(source_dir=source_dir, source_manifest=manifest, out_path=second) == stats
    assert first.read_bytes() == second.read_bytes()
    assert stats.rows == 248
    assert stats.suppressed_rows == 82
    assert stats.unclassified_rows == 82
    assert stats.absent_cells == 1412
    with gzip.open(first, "rt", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    assert not any(row["county_geoid"] == "26001" and row["sector_code"] == "99" for row in rows)
    assert rows[0]["annual_avg_emplvl"] == "0"
    assert all(row["annual_avg_emplvl"] == "" for row in rows if row["disclosure_code"] == "N")
    pins = {entry["file"]: entry["sha256"] for entry in county.load_source_manifest(manifest)}
    assert all(pins[row["source_file"]] == row["source_sha256"] for row in rows)


def test_source_order_twins_have_identical_typed_rows_and_semantic_identity() -> None:
    rows = [source_row(sector=code) for code in ("31-33", "11", "99")]
    assert canonicalize(rows) == canonicalize(list(reversed(rows)))
    assert builder.semantic_sha256(canonicalize(rows)) == builder.semantic_sha256(
        canonicalize(list(reversed(rows)))
    )


def test_hash_refusal_precedes_parsing_and_leaves_existing_output_untouched(
    tmp_path: Path, sector_sources: tuple[Path, Path]
) -> None:
    source_dir, manifest = sector_sources
    source = source_dir / county.load_source_manifest(manifest)[0]["file"]
    source.write_bytes(b"malformed and unpinned")
    output = tmp_path / "artifact.csv.gz"
    output.write_bytes(b"preserve this previous artifact")
    with pytest.raises(county.QcewBuildError, match="source_sha256"):
        builder.build(source_dir=source_dir, source_manifest=manifest, out_path=output)
    assert output.read_bytes() == b"preserve this previous artifact"


def test_checked_artifact_contract_and_independent_row_census() -> None:
    contract = verifier.load_contract(CONTRACT)
    verifier.verify_contract(contract)
    rows = verifier.verify_artifact(contract, ROOT)
    assert len(rows) == 1603
    assert len({row.county_geoid for row in rows}) == 83
    assert sum(row.disclosure_code == "N" for row in rows) == 416
    assert sum(row.sector_disposition == "unclassified" for row in rows) == 81
    assert sum(row.annual_avg_estabs_count for row in rows) == 235170
    assert max(row.total_annual_wages or 0 for row in rows) == 10954108416
    verifier.verify_artifact_manifest(contract, ROOT / "data-artifacts.yaml")


def test_ordinary_verifier_never_reads_raw_sources_or_writes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    contract = verifier.load_contract(CONTRACT)
    for relative in (
        "contracts/qcew_county_sectors_v1.yaml",
        "data-artifacts.yaml",
        contract["source"]["manifest"],
        contract["artifact"]["path"],
    ):
        path = tmp_path / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / relative, path)

    def refuse(*args: Any, **kwargs: Any) -> None:
        pytest.fail("ordinary verification must not build, read raw CSVs, or write")

    monkeypatch.setattr(builder, "build", refuse)
    monkeypatch.setattr(county, "verify_source_file", refuse)
    monkeypatch.setattr(Path, "write_bytes", refuse)
    monkeypatch.setattr(Path, "write_text", refuse)
    monkeypatch.chdir(tmp_path.parent)
    assert verifier.main(["--repo-root", str(tmp_path)]) == 0


def test_explicit_acquisition_rebuild_uses_cleaned_temporary_output(
    tmp_path: Path, sector_sources: tuple[Path, Path], monkeypatch: pytest.MonkeyPatch
) -> None:
    source_dir, manifest = sector_sources
    contract = copy.deepcopy(verifier.load_contract(CONTRACT))
    root = tmp_path / "checkout"
    pinned_manifest = root / contract["source"]["manifest"]
    pinned_manifest.parent.mkdir(parents=True)
    shutil.copyfile(manifest, pinned_manifest)
    contract["source"]["manifest_sha256"] = hashlib.sha256(manifest.read_bytes()).hexdigest()
    output = root / contract["artifact"]["path"]
    stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=output)
    contract["artifact"].update(sha256=stats.sha256, semantic_sha256=stats.semantic_sha256)
    original = output.read_bytes()
    calls = []
    original_build = builder.build

    def record(**kwargs: Any) -> builder.ArtifactStats:
        calls.append(kwargs["out_path"])
        assert not kwargs["out_path"].is_relative_to(root)
        return original_build(**kwargs)

    monkeypatch.setattr(builder, "build", record)
    verifier.verify_acquisition(contract, root, source_dir)
    contract["artifact"]["sha256"] = "0" * 64
    with pytest.raises(county.QcewBuildError, match="artifact_regeneration"):
        verifier.verify_acquisition(contract, root, source_dir)
    assert len(calls) == 2
    assert all(not path.parent.exists() for path in calls)
    assert output.read_bytes() == original


def test_registry_tripwire_requires_exact_artifact_entry(tmp_path: Path) -> None:
    contract = verifier.load_contract(CONTRACT)
    document = yaml.safe_load((ROOT / "data-artifacts.yaml").read_text())
    document["artifacts"] = [
        entry for entry in document["artifacts"] if entry["name"] != builder.ARTIFACT_NAME
    ]
    path = tmp_path / "data-artifacts.yaml"
    path.write_text(yaml.safe_dump(document))
    with pytest.raises(county.QcewBuildError, match="artifact_manifest_pin"):
        verifier.verify_artifact_manifest(contract, path)


@pytest.mark.parametrize(
    ("mutation", "refusal"),
    [
        ("suppressed_zero", "artifact_disclosure"),
        ("duplicate", "artifact_order"),
        ("reverse", "artifact_order"),
        ("wrong_source", "artifact_provenance"),
        ("missing", "artifact_census"),
        ("overflow", "artifact_value"),
        ("changed_value", "artifact_semantic_sha256"),
        ("misclassified", "artifact_disposition"),
    ],
)
def test_artifact_reader_refuses_tampering(tmp_path: Path, mutation: str, refusal: str) -> None:
    contract = verifier.load_contract(CONTRACT)
    artifact = ROOT / contract["artifact"]["path"]
    with gzip.open(artifact, "rt", encoding="utf-8", newline="") as handle:
        rows = list(csv.reader(handle))
    if mutation == "suppressed_zero":
        next(row for row in rows[1:] if row[4] == "N")[6] = "0"
    elif mutation == "duplicate":
        rows[2] = rows[1]
    elif mutation == "reverse":
        rows[1:] = reversed(rows[1:])
    elif mutation == "wrong_source":
        rows[1][10] = "0" * 64
    elif mutation == "missing":
        rows.pop()
    elif mutation == "overflow":
        rows[1][5] = str(2**63)
    elif mutation == "changed_value":
        rows[1][5] = str(int(rows[1][5]) + 1)
    else:
        next(row for row in rows[1:] if row[1] == "99")[3] = "classified"
    text = io.StringIO(newline="")
    csv.writer(text, lineterminator="\n").writerows(rows)
    path = tmp_path / contract["artifact"]["path"]
    path.parent.mkdir(parents=True)
    raw = gzip.compress(text.getvalue().encode(), mtime=0)
    path.write_bytes(raw)
    source_manifest = tmp_path / contract["source"]["manifest"]
    source_manifest.parent.mkdir(parents=True)
    shutil.copyfile(ROOT / contract["source"]["manifest"], source_manifest)
    contract["artifact"]["sha256"] = hashlib.sha256(raw).hexdigest()
    with pytest.raises(county.QcewBuildError, match=refusal):
        verifier.verify_artifact(contract, tmp_path)


@pytest.mark.parametrize("section", ["source", "semantics", "authority"])
def test_contract_refuses_a_different_observation_boundary(section: str) -> None:
    contract = verifier.load_contract(CONTRACT)
    if section == "source":
        contract[section]["own_code"] = "0"
    elif section == "semantics":
        contract[section]["suppression"] = "fill-with-zero"
    else:
        contract[section]["mechanics_consumers"] = ["employment-outcomes"]
    with pytest.raises(county.QcewBuildError, match=f"contract_{section}"):
        verifier.verify_contract(contract)


def test_contract_version_cannot_be_a_yaml_boolean() -> None:
    contract = verifier.load_contract(CONTRACT)
    contract["meta"]["version"] = True
    with pytest.raises(county.QcewBuildError, match="contract_meta"):
        verifier.verify_contract(contract)
