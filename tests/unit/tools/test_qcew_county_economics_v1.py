"""Independent contract tests for PER-319 QCEW county economics."""

from __future__ import annotations

import copy
import csv
import gzip
import hashlib
import io
import json
import re
import shutil
import sys
from pathlib import Path
from typing import Any

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[3]
TOOLS = ROOT / "tools"
sys.path.insert(0, str(TOOLS))

import make_qcew_county_economics_artifacts as builder  # type: ignore[import-not-found]  # noqa: E402
import verify_qcew_county_economics_v1 as verifier  # type: ignore[import-not-found]  # noqa: E402
from verify_qcew_county_economics_v1 import (  # type: ignore[import-not-found]  # noqa: E402
    QcewCountyEconomicsRefusal,
    load_contract,
    verify_artifact_manifest,
    verify_artifacts,
    verify_contract,
    verify_source_manifest,
)

CONTRACT = ROOT / "contracts" / "qcew_county_economics_v1.yaml"
MANIFEST = ROOT / "data-artifacts.yaml"
SOURCE_MANIFEST = ROOT / "tools" / "qcew_county_economics_v1_source_manifest.json"
ARTIFACT = (
    ROOT
    / "src"
    / "babylon"
    / "data"
    / "reference"
    / "economy"
    / "qcew_county_economics_mi_2024.csv.gz"
)

WAYNE_ROW = ["26163", "36727", "725504", "55436615328", "1469"]
PROJECTION_FIXTURES = ROOT / "tests" / "fixtures" / "qcew_economics"
CHANGED_SOURCE_VALUES = {
    "annual_avg_estabs_count": "40001",
    "annual_avg_emplvl": "812345",
    "total_annual_wages": "99999999999",
    "annual_avg_wkly_wage": "2049",
}


def test_consumer_mapping_keeps_source_units_and_observed_baseline() -> None:
    mapping = load_contract(CONTRACT)["consumer_mapping"]
    assert mapping["evidence_class"] == "Observed"
    assert mapping["conversion"] == "exact-nonnegative-i64-no-rounding"
    assert mapping["time_policy"] == "fixed-2024-observed-baseline"
    assert mapping["mechanics_consumers"] == []
    fields = mapping["fields"]
    assert [field["source_statistic"] for field in fields] == list(verifier.COLUMNS[1:])
    assert [field["unit"] for field in fields] == [
        "establishments",
        "jobs",
        "USD",
        "USD-per-employee-per-week",
    ]
    assert all(field["target_field"] == f"territory/{field['grant_key']}" for field in fields)
    verify_contract(load_contract(CONTRACT))


@pytest.mark.parametrize(
    ("key", "value"),
    [
        ("conversion", "annual-payroll-divided-by-weekly-wage"),
        ("evidence_class", "Designed"),
        ("time_policy", "advance-vintage-each-campaign-week"),
        ("mechanics_consumers", ["production-output"]),
    ],
)
def test_consumer_mapping_refuses_changed_meaning(key: str, value: object) -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    contract["consumer_mapping"][key] = value
    with pytest.raises(QcewCountyEconomicsRefusal, match="consumer_mapping"):
        verify_contract(contract)


def _gzip_csv(path: Path, header: list[str], rows: list[list[Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with (
        path.open("wb") as raw,
        gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as binary,
        io.TextIOWrapper(binary, encoding="utf-8", newline="") as text,
    ):
        writer = csv.writer(text, lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


def _read_artifact_rows(path: Path = ARTIFACT) -> tuple[list[str], list[list[str]]]:
    with gzip.GzipFile(fileobj=io.BytesIO(path.read_bytes()), mode="rb") as compressed:
        text = io.TextIOWrapper(compressed, encoding="utf-8", newline="")
        reader = csv.reader(text)
        return list(next(reader)), list(reader)


def _fixture_county_rows(
    geoid: str = "26001",
    *,
    disclosure_code: str = "",
    annual_avg_emplvl: str = "1494",
) -> list[dict[str, str]]:
    row = dict.fromkeys(builder.EXPECTED_SOURCE_COLUMNS, "0")
    row.update(
        {
            "area_fips": geoid,
            "own_code": "0",
            "industry_code": "10",
            "agglvl_code": "70",
            "size_code": "0",
            "year": "2024",
            "qtr": "A",
            "disclosure_code": disclosure_code,
            "area_title": "Alcona County, Michigan",
            "own_title": "Total Covered",
            "industry_title": "10 Total, all industries",
            "agglvl_title": "County, Total Covered",
            "size_title": "All establishment sizes",
            "annual_avg_estabs_count": "214",
            "annual_avg_emplvl": annual_avg_emplvl,
            "total_annual_wages": "62042985",
            "annual_avg_wkly_wage": "798",
        }
    )
    return [row]


@pytest.fixture
def county_sources(tmp_path: Path) -> tuple[Path, Path]:
    """Generate one tiny public-schema source per county, with pinned bytes."""
    source_dir = tmp_path / "sources"
    source_dir.mkdir()
    entries = []
    for index, geoid in enumerate(builder.MICHIGAN_COUNTY_GEOIDS):
        name = f"2024.annual {geoid} Fixture {geoid} County, Michigan.csv"
        source = source_dir / name
        with source.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=builder.EXPECTED_SOURCE_COLUMNS)
            writer.writeheader()
            writer.writerows(_fixture_county_rows(geoid, annual_avg_emplvl=str(index + 1)))
        entries.append({"file": name, "sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
    manifest = tmp_path / "source-manifest.json"
    manifest.write_text(
        json.dumps({"contract": "QcewCountyEconomicsV1", "version": 1, "entries": entries}),
        encoding="utf-8",
    )
    return source_dir, manifest


def test_artifact_verification_never_rebuilds_or_writes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    contract = load_contract(CONTRACT)
    artifact = tmp_path / contract["artifact"]["path"]
    artifact.parent.mkdir(parents=True)
    shutil.copyfile(ARTIFACT, artifact)
    original = artifact.read_bytes()

    def refuse_build(**kwargs: Any) -> None:
        pytest.fail("ordinary verification must not read acquisition sources or rebuild")

    def refuse_write(*args: Any, **kwargs: Any) -> None:
        pytest.fail("ordinary verification must not write or delete files")

    monkeypatch.setattr(builder, "build", refuse_build)
    monkeypatch.setattr(Path, "write_bytes", refuse_write)
    monkeypatch.setattr(Path, "write_text", refuse_write)
    monkeypatch.setattr(Path, "unlink", refuse_write)

    assert len(verify_artifacts(contract, tmp_path)) == 83
    assert artifact.read_bytes() == original
    assert [path for path in tmp_path.rglob("*") if path.is_file()] == [artifact]


def test_cli_verifies_artifact_only_checkout(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    for path in (CONTRACT, MANIFEST, SOURCE_MANIFEST, ARTIFACT):
        destination = tmp_path / path.relative_to(ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)

    def refuse_build(**kwargs: Any) -> None:
        pytest.fail("ordinary CLI must not rebuild")

    monkeypatch.setattr(builder, "build", refuse_build)
    monkeypatch.chdir(tmp_path.parent)
    assert verifier.main(["--repo-root", str(tmp_path)]) == 0
    assert "83 county rows" in capsys.readouterr().out


@pytest.mark.parametrize("refusal", [None, "source_sha256", "artifact_regeneration"])
def test_explicit_acquisition_verification_cleans_temporary_output(
    tmp_path: Path,
    county_sources: tuple[Path, Path],
    monkeypatch: pytest.MonkeyPatch,
    refusal: str | None,
) -> None:
    source_dir, manifest = county_sources
    contract = copy.deepcopy(load_contract(CONTRACT))
    root = tmp_path / "checkout"
    source_manifest = root / verifier.EXPECTED_SOURCE_MANIFEST
    source_manifest.parent.mkdir(parents=True)
    shutil.copyfile(manifest, source_manifest)
    contract["source"]["manifest_sha256"] = hashlib.sha256(manifest.read_bytes()).hexdigest()
    artifact = root / contract["artifact"]["path"]
    stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=artifact)
    contract["artifact"].update(sha256=stats.sha256, semantic_sha256=stats.semantic_sha256)
    original = artifact.read_bytes()
    if refusal == "source_sha256":
        entry = builder.load_source_manifest(manifest)[0]
        (source_dir / entry["file"]).write_bytes(b"changed source")
    elif refusal == "artifact_regeneration":
        contract["artifact"]["sha256"] = "0" * 64

    output_paths = []
    original_build = builder.build

    def record_build(**kwargs: Any) -> builder.ArtifactStats:
        output_paths.append(kwargs["out_path"])
        assert not kwargs["out_path"].is_relative_to(root)
        return original_build(**kwargs)

    monkeypatch.setattr(builder, "build", record_build)
    if refusal is None:
        verifier.verify_acquisition(contract, root, source_dir)
        verifier.verify_acquisition(contract, root, source_dir)
        assert len(set(output_paths)) == 2
    else:
        with pytest.raises(QcewCountyEconomicsRefusal, match=refusal):
            verifier.verify_acquisition(contract, root, source_dir)

    assert output_paths
    assert all(not path.parent.exists() for path in output_paths)
    assert artifact.read_bytes() == original


@pytest.mark.parametrize(
    ("mutation", "refusal"),
    [
        ("duplicate", "source_manifest_order"),
        ("reverse", "source_manifest_order"),
        ("other_county", "source_manifest_geoids"),
        ("nested_path", "source_manifest_file"),
    ],
)
def test_independent_manifest_verifier_requires_canonical_county_coverage(
    tmp_path: Path, mutation: str, refusal: str
) -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    document = json.loads(SOURCE_MANIFEST.read_text(encoding="utf-8"))
    entries = document["entries"]
    if mutation == "duplicate":
        entries[1] = entries[0]
    elif mutation == "reverse":
        entries.reverse()
    elif mutation == "other_county":
        entries[0]["file"] = entries[0]["file"].replace("26001", "26002")
    else:
        entries[0]["file"] = entries[0]["file"].replace("Alcona", "nested/Alcona")
    manifest = tmp_path / "manifest.json"
    manifest.write_text(json.dumps(document), encoding="utf-8")
    contract["source"]["manifest_sha256"] = hashlib.sha256(manifest.read_bytes()).hexdigest()

    with pytest.raises(QcewCountyEconomicsRefusal, match=refusal):
        verify_source_manifest(contract, manifest)


def test_source_manifest_declares_exactly_the_83_michigan_county_files() -> None:
    entries = builder.load_source_manifest(SOURCE_MANIFEST)

    assert len(entries) == 83
    geoids = []
    for entry in entries:
        match = builder.COUNTY_FILE_RE.fullmatch(entry["file"])
        assert match is not None, entry["file"]
        geoids.append(match.group(1))
        assert len(entry["sha256"]) == 64
        assert re.fullmatch(r"[0-9a-f]{64}", entry["sha256"])
    assert geoids == sorted(builder.MICHIGAN_COUNTY_GEOIDS)
    assert geoids == sorted(geoids)


def test_checked_in_contract_artifact_and_manifest_verify() -> None:
    contract = load_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "QcewCountyEconomicsV1",
        "version": 1,
        "issue": "PER-319",
        "parent": "PER-10",
    }
    verify_contract(contract)
    verify_source_manifest(contract, SOURCE_MANIFEST)
    rows = verify_artifacts(contract, ROOT)
    verify_artifact_manifest(contract, MANIFEST)
    assert len(rows) == contract["artifact"]["rows"] == 83


def test_builder_output_is_byte_identical_across_runs(
    tmp_path: Path, county_sources: tuple[Path, Path]
) -> None:
    source_dir, manifest = county_sources
    first = tmp_path / "first.csv.gz"
    second = tmp_path / "second.csv.gz"

    first_stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=first)
    second_stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=second)

    assert first.read_bytes() == second.read_bytes()
    assert first_stats == second_stats
    assert first_stats.rows == 83


def test_changed_source_reaches_the_shared_rust_foundation_fixture(
    tmp_path: Path, county_sources: tuple[Path, Path]
) -> None:
    """Designed test inputs cross the real Python builder and Rust consumer boundary."""
    source_dir, manifest = county_sources
    baseline = tmp_path / "baseline.csv.gz"
    builder.build(source_dir=source_dir, source_manifest=manifest, out_path=baseline)
    assert (
        gzip.decompress(baseline.read_bytes())
        == (PROJECTION_FIXTURES / "baseline.csv").read_bytes()
    )

    document = json.loads(manifest.read_text())
    entry = next(row for row in document["entries"] if "26163" in row["file"])
    source = source_dir / entry["file"]
    with source.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    rows[0].update(CHANGED_SOURCE_VALUES)
    with source.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=builder.EXPECTED_SOURCE_COLUMNS)
        writer.writeheader()
        writer.writerows(rows)

    changed = tmp_path / "changed.csv.gz"
    with pytest.raises(builder.QcewBuildError, match="source_sha256"):
        builder.build(source_dir=source_dir, source_manifest=manifest, out_path=changed)
    assert not changed.exists(), "changed acquisition bytes must not silently bypass their pin"
    entry["sha256"] = hashlib.sha256(source.read_bytes()).hexdigest()
    manifest.write_text(json.dumps(document))
    stats = builder.build(source_dir=source_dir, source_manifest=manifest, out_path=changed)
    assert (
        gzip.decompress(changed.read_bytes()) == (PROJECTION_FIXTURES / "changed.csv").read_bytes()
    )

    before_header, before = _read_artifact_rows(baseline)
    after_header, after = _read_artifact_rows(changed)
    assert before_header == after_header == list(builder.COLUMNS)
    assert len(before) == len(after) == 83
    differences = [(old, new) for old, new in zip(before, after, strict=True) if old != new]
    assert len(differences) == 1
    old, new = differences[0]
    assert old[0] == new[0] == "26163"
    assert new[1:] == [CHANGED_SOURCE_VALUES[column] for column in builder.COLUMNS[1:]]

    # Ordinary verification reads the generated artifact only, using its explicitly
    # changed test pin. The repository's observed artifact and pins remain untouched.
    contract = copy.deepcopy(load_contract(CONTRACT))
    contract["artifact"].update(
        path=changed.name, sha256=stats.sha256, semantic_sha256=stats.semantic_sha256
    )
    assert verify_artifacts(contract, tmp_path) == after


@pytest.mark.parametrize("column", tuple(CHANGED_SOURCE_VALUES))
def test_each_source_statistic_changes_only_its_canonical_column(column: str) -> None:
    rows = _fixture_county_rows()
    before = builder.canonicalize_county_row("26001", builder.EXPECTED_SOURCE_COLUMNS, rows)
    rows[0][column] = CHANGED_SOURCE_VALUES[column]
    after = builder.canonicalize_county_row("26001", builder.EXPECTED_SOURCE_COLUMNS, rows)
    expected = before.copy()
    expected[builder.COLUMNS.index(column)] = CHANGED_SOURCE_VALUES[column]
    assert after == expected


def test_committed_artifact_schema_and_ordering() -> None:
    header, rows = _read_artifact_rows()

    assert header == list(builder.COLUMNS)
    assert len(rows) == 83
    geoids = [row[0] for row in rows]
    assert geoids == sorted(geoids)
    assert len(set(geoids)) == 83
    assert geoids == list(builder.MICHIGAN_COUNTY_GEOIDS)
    for row in rows:
        assert len(row) == 5
        assert re.fullmatch(r"26[0-1][0-9]{2}", row[0])
        assert all(re.fullmatch(r"[0-9]+", value) for value in row[1:])


def test_wayne_county_known_values_match_bls_public_record() -> None:
    _, rows = _read_artifact_rows()

    assert WAYNE_ROW in rows


def test_gzip_writer_is_byte_identical_across_output_paths(tmp_path: Path) -> None:
    rows = [WAYNE_ROW]
    first = tmp_path / "first.csv.gz"
    second = tmp_path / "second.csv.gz"

    first_stats = builder._write_gzip_csv(first, builder.COLUMNS, rows)
    second_stats = builder._write_gzip_csv(second, builder.COLUMNS, rows)

    assert first.read_bytes() == second.read_bytes()
    assert first_stats == second_stats


def test_canonicalizer_refuses_source_schema_drift() -> None:
    rows = _fixture_county_rows()
    fieldnames = [column for column in builder.EXPECTED_SOURCE_COLUMNS if column != "qtr"]

    with pytest.raises(builder.QcewBuildError, match="source_schema"):
        builder.canonicalize_county_row("26001", fieldnames, rows)


def test_canonicalizer_refuses_missing_or_duplicate_county_total() -> None:
    with pytest.raises(builder.QcewBuildError, match="source_selection"):
        builder.canonicalize_county_row("26001", builder.EXPECTED_SOURCE_COLUMNS, [])
    with pytest.raises(builder.QcewBuildError, match="source_selection"):
        builder.canonicalize_county_row(
            "26001",
            builder.EXPECTED_SOURCE_COLUMNS,
            _fixture_county_rows() * 2,
        )


@pytest.mark.parametrize(
    ("field", "value", "code"),
    [
        ("area_fips", "26003", "source_geoid"),
        ("disclosure_code", "N", "source_disclosure"),
        ("size_code", "1", "source_row_identity"),
        ("qtr", "1", "source_row_identity"),
        ("year", "2023", "source_row_identity"),
        ("industry_code", "1024", "source_row_identity"),
        ("annual_avg_emplvl", "1,494", "source_value"),
        ("annual_avg_emplvl", "", "source_value"),
        ("annual_avg_wkly_wage", "nan", "source_value"),
    ],
)
def test_canonicalizer_refuses_noncanonical_source_values(
    field: str, value: str, code: str
) -> None:
    rows = _fixture_county_rows()
    rows[0][field] = value

    with pytest.raises(builder.QcewBuildError, match=code):
        builder.canonicalize_county_row("26001", builder.EXPECTED_SOURCE_COLUMNS, rows)


def test_builder_refuses_source_manifest_digest_mismatch(
    tmp_path: Path, county_sources: tuple[Path, Path]
) -> None:
    source_dir, source_manifest = county_sources
    document = json.loads(source_manifest.read_text(encoding="utf-8"))
    document["entries"][0]["sha256"] = "0" * 64
    manifest = tmp_path / "manifest.json"
    manifest.write_text(json.dumps(document), encoding="utf-8")

    with pytest.raises(builder.QcewBuildError, match="source_sha256"):
        builder.build(
            source_dir=source_dir, out_path=tmp_path / "out.csv.gz", source_manifest=manifest
        )


def test_contract_refuses_missing_substantive_value_classification() -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    del contract["classifications"]["semantic_digests"]

    with pytest.raises(QcewCountyEconomicsRefusal, match="contract_shape"):
        verify_contract(contract)


def test_contract_loader_refuses_duplicate_keys(tmp_path: Path) -> None:
    duplicate = tmp_path / "duplicate.yaml"
    duplicate.write_text("meta: first\nmeta: second\n", encoding="utf-8")

    with pytest.raises(QcewCountyEconomicsRefusal, match="invalid_contract"):
        load_contract(duplicate)


def test_artifact_verifier_refuses_unsorted_rows(tmp_path: Path) -> None:
    contract = copy.deepcopy(load_contract(CONTRACT))
    artifact_path = tmp_path / str(contract["artifact"]["path"])
    rows = [
        ["26163", "36727", "725504", "55436615328", "1469"],
        ["26001", "214", "1494", "62042985", "798"],
    ]
    _gzip_csv(artifact_path, list(contract["artifact"]["columns"]), rows)
    contract["artifact"]["path"] = str(artifact_path.relative_to(tmp_path))
    contract["artifact"]["rows"] = 2
    contract["artifact"]["sha256"] = hashlib.sha256(artifact_path.read_bytes()).hexdigest()

    with pytest.raises(QcewCountyEconomicsRefusal, match="artifact_order"):
        verify_artifacts(contract, tmp_path)


def test_artifact_manifest_tripwire_is_not_managed_by_sqlite_generator() -> None:
    manifest = yaml.safe_load(MANIFEST.read_text(encoding="utf-8"))
    by_name = {row["name"]: row for row in manifest["artifacts"]}
    managed = {spec.name for spec in builder.make_data_artifacts_specs()}

    assert builder.ARTIFACT_NAME in by_name
    assert builder.ARTIFACT_NAME not in managed


def test_contract_json_is_finite_and_canonicalizable() -> None:
    contract = load_contract(CONTRACT)

    encoded = json.dumps(contract, sort_keys=True, allow_nan=False).encode("utf-8")
    assert encoded
