"""Preservation contracts for the ADR076 data artifacts (rulings R1-R5).

Once a table's DB copy is dropped (the catalog->manifest demotion handoff),
these tests are what pins its content: row counts + known rows read from the
ARTIFACT, never the DB. The in-repo CSV tier always runs; the dist-tier
parquet contracts skip where the files are absent (they ship as ci-data
release assets, generated locally by ``tools/make_data_artifacts.py``).

The synthetic double-generation test proves the generator's byte-stability
property on every run, everywhere — the real-DB proof (two full runs,
identical manifest) was recorded 2026-07-17.
"""

from __future__ import annotations

import csv
import hashlib
import sqlite3
import sys
from pathlib import Path
from types import SimpleNamespace

import pytest
import yaml

_REPO_ROOT = Path(__file__).resolve().parents[3]
_CSV_HOME = _REPO_ROOT / "src" / "babylon" / "data" / "reference"
_DIST_HOME = _REPO_ROOT / "dist" / "data-artifacts"
_MANIFEST = _REPO_ROOT / "data-artifacts.yaml"

TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402
from make_data_artifacts import (  # type: ignore[import-not-found]  # noqa: E402
    ArtifactError,
    _arrow_type,
    _rewrite_manifest_preserving_blocks,
    _sha256,
    _table_layout,
    _write_csv,
    _write_manifest,
    export_table_parquet,
    generate,
    update_product_block,
    update_schema_block,
)


def _read_csv(name: str) -> list[dict[str, str]]:
    with (_CSV_HOME / name).open(encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


class TestManifest:
    def test_manifest_hashes_match_in_repo_artifacts(self) -> None:
        manifest = yaml.safe_load(_MANIFEST.read_text())
        checked = 0
        for entry in manifest["artifacts"]:
            path = _REPO_ROOT / entry["home"]
            if not str(entry["home"]).startswith("src/"):
                continue
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            assert digest == entry["sha256"], f"{entry['name']} drifted from its manifest hash"
            checked += 1
        assert checked == 29
        # the four registered canonical CSVs (R1 pair post-demotion, ricci,
        # county->CZ) plus the 13 Vol II Unit U2 hand-registered LODES
        # entries (1 tri-county crosswalk + 12 OD-matrix years, generator
        # tools/make_lodes_tri_county_artifact.py — see
        # tests/unit/tools/test_lodes_artifact_manifest_entries.py for their
        # dedicated content-pinning tripwire) plus the P25 U6/ADR132
        # mit_countypres_rep_share election artifact (generator
        # tools/make_election_lab_artifact.py — tripwire:
        # tests/unit/tools/test_election_lab_artifact_manifest.py) plus the
        # P26 U3 faf_bloc_trade_tons freight artifact (generator
        # tools/make_faf_bloc_tons_artifact.py — tripwire:
        # tests/unit/tools/test_faf_artifact_manifest_entry.py) plus the
        # #334 Phase 0 national-incidence trio (county_fips_vintage_crosswalk,
        # national_incidence_county_pole, national_reproduction_floor —
        # generators tools/make_fips_vintage_crosswalk.py /
        # tools/make_national_incidence_artifact.py — tripwire:
        # tests/unit/tools/test_national_incidence_manifest_entries.py) plus
        # the two PER-276 Census place authority artifacts (generator
        # tools/make_census_place_artifacts.py — tripwire:
        # tests/unit/tools/test_census_place_authority_v1.py) plus the two
        # PER-277 county/place/H3 land-overlap artifacts (generator
        # tools/make_county_place_h3_overlap_artifacts.py — tripwire:
        # tests/unit/tools/test_county_place_h3_overlap_v1.py) plus the
        # PER-319 QCEW county-economics artifact (generator
        # tools/make_qcew_county_economics_artifacts.py — tripwire:
        # tests/unit/tools/test_qcew_county_economics_v1.py), the QCEW
        # county-sector artifact (test_qcew_county_sectors_v1.py), and the
        # observed Detroit-Windsor rail source (test_ntad_border_rail_v1.py).

    def test_manifest_carries_all_registered_artifacts(self) -> None:
        # Post-cutover the manifest is FULL-COVERAGE: the registered
        # canonical artifacts plus one generate-mode parquet per governed
        # table (Task 10 Step 2, 2026-07-20) — so the law is containment of
        # the curated set plus catalog-complete coverage, not set equality.
        manifest = yaml.safe_load(_MANIFEST.read_text())
        names = {entry["name"] for entry in manifest["artifacts"]}
        curated = {
            "bridge_county_bea_ea",
            "dim_bea_economic_area",
            "babylon_ricci_final",
            "bridge_county_cz",
            "fact_energy_annual",
            "dim_energy_series",
            "dim_energy_table",
            "bridge_lodes_block",
            "staging_arcgis_feature",
            "census_place_identity_mi_2023",
            "census_place_geometry_mi_2023",
            "census_county_h3_land_overlap_mi_2023",
            "census_county_place_h3_land_overlap_mi_2023",
            "qcew_county_economics_mi_2024",
            "qcew_county_sectors_mi_2024",
            "ntad_detroit_windsor_rail_v1",
        }
        assert curated <= names
        from babylon.sentinels.coverage.catalog import load_catalog_tables

        governed_tables = {t.name for t in load_catalog_tables() if t.kind == "table"}
        missing = governed_tables - names
        assert missing == set(), f"governed tables with no manifest source: {sorted(missing)}"


class TestR1BeaCsvPreservation:
    def test_bridge_county_bea_ea(self) -> None:
        rows = _read_csv("bridge_county_bea_ea.csv")
        assert len(rows) == 83
        assert rows[0] == {"county_id": "1232", "bea_ea_id": "8"}

    def test_dim_bea_economic_area(self) -> None:
        rows = _read_csv("dim_bea_economic_area.csv")
        assert len(rows) == 8
        assert rows[0]["ea_code"] == "CHI"
        assert rows[0]["ea_name"] == "Chicago-Naperville (cross-border)"
        assert {r["ea_code"] for r in rows} == {
            "CHI",
            "DET",
            "GRR",
            "KAL",
            "LAN",
            "MQT",
            "SAG",
            "TVC",
        }

    def test_cross_border_ea_semantics_survive_the_demotion(self) -> None:
        # Migrated from tests/integration/test_michigan_reference_data.py's
        # TestBEAEconomicAreas when the DB copies dropped (ADR076 R1). The
        # county_id surrogates are pinned with their identities: 1242 =
        # Berrien MI (26021), 1313 = Wayne MI (26163), 1283 = Marquette MI
        # (26103) — resolved against dim_county on 2026-07-17.
        bridge = {
            row["county_id"]: row["bea_ea_id"] for row in _read_csv("bridge_county_bea_ea.csv")
        }
        code_by_ea = {
            row["bea_ea_id"]: row["ea_code"] for row in _read_csv("dim_bea_economic_area.csv")
        }
        assert code_by_ea[bridge["1242"]] == "CHI"  # Berrien belongs to cross-border Chicago
        assert code_by_ea[bridge["1313"]] == "DET"  # Wayne is the Detroit EA node
        assert code_by_ea[bridge["1283"]] == "MQT"  # Marquette anchors the Upper Peninsula EA


class TestR2RicciRegistration:
    def test_canonical_csv_is_untouched_and_registered(self) -> None:
        # register-mode must never rewrite the pre-existing canonical CSV —
        # its content contract lives in test_unequal_exchange_artifacts.py.
        rows = _read_csv("babylon_ricci_final.csv")
        assert len(rows) == 51
        assert rows[0]["region_name"] == "North America"


@pytest.mark.skipif(
    not (_DIST_HOME / "bridge_lodes_block.parquet").is_file(),
    reason="dist-tier parquet artifacts absent (generate locally or fetch ci-data assets)",
)
class TestParquetPreservation:
    def test_bridge_lodes_block_spike(self) -> None:
        import pyarrow.parquet as pq

        table = pq.read_table(_DIST_HOME / "bridge_lodes_block.parquet")
        assert table.num_rows == 1_150_562
        assert table.column_names[0] == "block_geoid"
        first = table.slice(0, 1).to_pylist()[0]
        assert first["block_geoid"] == "011010001001000"
        assert first["state_fips"] == "01"
        assert first["cbsa_code"] == "33860"

    def test_energy_family(self) -> None:
        import pyarrow.parquet as pq

        fact = pq.read_table(_DIST_HOME / "fact_energy_annual.parquet")
        assert fact.num_rows == 525
        first = fact.slice(0, 1).to_pylist()[0]
        assert first["series_id"] == 1
        assert first["time_id"] == 1
        assert first["value"] == pytest.approx(69869.338)
        assert pq.read_table(_DIST_HOME / "dim_energy_series.parquet").num_rows == 20
        assert pq.read_table(_DIST_HOME / "dim_energy_table.parquet").num_rows == 14

    def test_staging_arcgis_feature(self) -> None:
        import pyarrow.parquet as pq

        table = pq.read_table(_DIST_HOME / "staging_arcgis_feature.parquet")
        assert table.num_rows == 5_974


class TestGeneratorDeterminism:
    """The ADR076 byte-stability property, proven on synthetic input."""

    @pytest.fixture()
    def source_db(self, tmp_path: Path) -> Path:
        db = tmp_path / "source.sqlite"
        conn = sqlite3.connect(db)
        conn.executescript(
            """
            CREATE TABLE fact_sample (
                id INTEGER NOT NULL,
                label VARCHAR(20),
                value NUMERIC(10, 2),
                PRIMARY KEY (id)
            );
            INSERT INTO fact_sample VALUES (2, 'b', 2.5), (1, 'a', 1.25), (3, NULL, NULL);
            """
        )
        conn.commit()
        conn.close()
        return db

    def test_double_generation_is_byte_identical(self, source_db: Path, tmp_path: Path) -> None:
        # The parquet leg exercises the production writer, export_table_parquet
        # (the generate() path's only parquet-writing call) — not a second,
        # production-unused writer that the fast gate wouldn't otherwise guard.
        conn = sqlite3.connect(f"file:{source_db}?mode=ro", uri=True)
        try:
            columns, _pk, _schema = _table_layout(conn, "fact_sample")
            rows = conn.execute("SELECT id, label, value FROM fact_sample ORDER BY id").fetchall()
            hashes: dict[str, set[str]] = {"csv": set(), "parquet": set()}
            for run in (1, 2):
                csv_out = tmp_path / f"run{run}.csv"
                pq_out = tmp_path / f"run{run}.parquet"
                _write_csv(csv_out, columns, rows)
                export_table_parquet(conn, "fact_sample", pq_out)
                hashes["csv"].add(_sha256(csv_out))
                hashes["parquet"].add(_sha256(pq_out))
        finally:
            conn.close()
        assert len(hashes["csv"]) == 1
        assert len(hashes["parquet"]) == 1

    def test_unmapped_decltype_is_loud(self) -> None:
        with pytest.raises(ArtifactError, match="BLOB"):
            _arrow_type("BLOB")

    def test_missing_source_table_is_loud(self, source_db: Path) -> None:
        conn = sqlite3.connect(source_db)
        try:
            with pytest.raises(ArtifactError, match="fact_missing"):
                _table_layout(conn, "fact_missing")
        finally:
            conn.close()

    def test_generate_mode_requires_its_spec_table(
        self, source_db: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        # A generate-mode spec must hard-fail against a DB lacking its table —
        # the loud-policy discipline inherited from make_reference_subset.
        # Synthetic spec: every CURATED spec is register-mode post-demotion
        # (all nine adopt canonical artifacts), so the law is pinned here
        # directly; full-coverage enumerated specs are the live generate path.
        import make_data_artifacts as mda

        spec = mda.ArtifactSpec(
            name="fact_missing",
            format="parquet",
            source_table="fact_missing",
            home="dist/data-artifacts/fact_missing.parquet",
            material_relation="test: names a table the DB does not have.",
        )
        monkeypatch.setattr(mda, "ARTIFACTS", (spec,))
        with pytest.raises(ArtifactError, match="fact_missing"):
            generate(source_db)


class TestEnumerateFullCoverageSpecs:
    """Auto-enumeration of the full-DB-coverage artifact registry (parquet
    pipeline plan Task 2): one spec per governed table not already curated in
    ``ARTIFACTS``, with ``material_relation`` inherited from the table's
    ``data-catalog.yaml`` row via :func:`make_data_artifacts._catalog_by_name`.
    """

    def test_enumerate_full_coverage_specs(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        conn = sqlite3.connect(tmp_path / "m.sqlite")
        conn.execute("CREATE TABLE fact_new (id INTEGER PRIMARY KEY, v REAL)")
        conn.execute("CREATE TABLE fact_energy_annual (id INTEGER PRIMARY KEY)")  # curated already
        conn.commit()
        fake_catalog = {"fact_new": SimpleNamespace(material_relation="test relation")}
        monkeypatch.setattr(make_data_artifacts, "_catalog_by_name", lambda: fake_catalog)
        specs = make_data_artifacts.enumerate_full_coverage_specs(conn)
        assert [s.name for s in specs] == ["fact_new"]  # curated table skipped
        assert specs[0].home == "dist/data-artifacts/fact_new.parquet"
        assert specs[0].material_relation == "test relation"
        assert specs[0].mode == "generate"
        assert specs[0].format == "parquet"

    def test_enumerate_full_coverage_missing_catalog_row_is_loud(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        conn = sqlite3.connect(tmp_path / "m.sqlite")
        conn.execute("CREATE TABLE fact_orphan (id INTEGER PRIMARY KEY)")
        conn.commit()
        monkeypatch.setattr(make_data_artifacts, "_catalog_by_name", dict)
        with pytest.raises(KeyError, match="fact_orphan"):
            make_data_artifacts.enumerate_full_coverage_specs(conn)

    def test_enumerate_full_coverage_specs_is_name_sorted(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        # FOLD-IN 3 (Task 2 review): >=2 enumerated tables come back in
        # name-sorted order. governed_db_tables() already ORDER BYs the
        # table name in SQL; this pins that the enumeration wrapper doesn't
        # lose the ordering on its way through.
        conn = sqlite3.connect(tmp_path / "m3.sqlite")
        conn.execute("CREATE TABLE fact_zzz_last (id INTEGER PRIMARY KEY)")
        conn.execute("CREATE TABLE fact_aaa_first (id INTEGER PRIMARY KEY)")
        conn.execute("CREATE TABLE fact_mmm_mid (id INTEGER PRIMARY KEY)")
        conn.commit()
        fake_catalog = {
            "fact_zzz_last": SimpleNamespace(material_relation="z"),
            "fact_aaa_first": SimpleNamespace(material_relation="a"),
            "fact_mmm_mid": SimpleNamespace(material_relation="m"),
        }
        monkeypatch.setattr(make_data_artifacts, "_catalog_by_name", lambda: fake_catalog)
        specs = make_data_artifacts.enumerate_full_coverage_specs(conn)
        names = [s.name for s in specs]
        assert len(names) >= 2
        assert names == sorted(names)
        assert names == ["fact_aaa_first", "fact_mmm_mid", "fact_zzz_last"]


class TestManifestV2Writer:
    """Manifest v2 (parquet pipeline Task 3): the ``schema``/``product``
    blocks, the single ``_write_manifest`` writer, and its second caller
    ``update_product_block``.
    """

    @pytest.fixture()
    def real_entries(self) -> list[dict[str, object]]:
        """The actual committed v1 manifest's artifact entries — reused so
        the round-trip tests prove something about real content, not just
        synthetic fixtures, and so at least one entry exercises a
        multi-line-wrapped ``material_relation`` (the FOLD-IN 1 fix).

        Filtered to entries this writer actually manages
        (``generator: tools/make_data_artifacts.py``): ``_write_manifest``
        unconditionally stamps that literal on every entry it writes (it has
        no per-entry ``generator`` concept), so round-tripping the Vol II
        Unit U2 hand-maintained LODES entries (``generator:
        tools/make_lodes_tri_county_artifact.py``) through it would rewrite
        their generator field out from under them — a real fact about the
        writer, not a bug this suite is chartered to pin. See
        ``tests/unit/tools/test_lodes_artifact_manifest_entries.py`` for the
        LODES entries' own dedicated content contract.
        """
        manifest = yaml.safe_load(_MANIFEST.read_text())
        return [
            entry
            for entry in manifest["artifacts"]
            if entry["generator"] == "tools/make_data_artifacts.py"
        ]

    @pytest.fixture()
    def schema_block(self) -> dict[str, object]:
        return {
            "file": "dist/data-artifacts/schema.sql",
            "sha256": "a" * 64,
            "tables": 76,
            "views": 8,
            "indexes": 100,
        }

    @pytest.fixture()
    def product_block(self) -> dict[str, object]:
        return {
            "name": "marxist-data-3NF.sqlite",
            "sha256": "b" * 64,
            "page_size": 4096,
            "application_id": 1112359244,
            "user_version": 1,
            "sqlite_version": "3.46.1",
        }

    def test_write_manifest_v2_round_trip(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
        product_block: dict[str, object],
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(
            real_entries, schema_entry=schema_block, product_entry=product_block, path=out
        )
        parsed = yaml.safe_load(out.read_text())
        assert parsed["version"] == "2.0.0"
        # fixed block order: version, schema, product, artifacts
        assert list(parsed.keys()) == ["version", "schema", "product", "artifacts"]
        assert parsed["schema"] == schema_block
        assert parsed["product"] == product_block
        assert parsed["artifacts"] == real_entries

    def test_write_manifest_v2_omits_absent_blocks(
        self, tmp_path: Path, real_entries: list[dict[str, object]]
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, path=out)
        parsed = yaml.safe_load(out.read_text())
        assert parsed["version"] == "2.0.0"
        assert "schema" not in parsed
        assert "product" not in parsed
        assert parsed["artifacts"] == real_entries

    def test_update_product_block_adds_product_preserving_artifacts_section(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        product_block: dict[str, object],
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, path=out)  # no product block yet
        before_text = out.read_text()
        before_artifacts_section = before_text.split("\nartifacts:\n", 1)[1]

        update_product_block(out, product_block)

        after_text = out.read_text()
        after_artifacts_section = after_text.split("\nartifacts:\n", 1)[1]
        assert after_artifacts_section == before_artifacts_section

        parsed = yaml.safe_load(after_text)
        assert parsed["product"] == product_block
        assert "schema" not in parsed  # wasn't present before; stays absent

    def test_update_product_block_preserves_existing_schema_block(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
        product_block: dict[str, object],
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, schema_entry=schema_block, path=out)

        update_product_block(out, product_block)

        parsed = yaml.safe_load(out.read_text())
        assert parsed["schema"] == schema_block
        assert parsed["product"] == product_block

    def test_rewrite_idempotence_is_byte_identical(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        product_block: dict[str, object],
    ) -> None:
        # FOLD-IN 1 consequence: write -> update_product_block (unchanged
        # product) -> bytes identical. No pre-commit hook involved, so this
        # must hold on the raw writer output alone.
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, product_entry=product_block, path=out)
        first_bytes = out.read_bytes()

        update_product_block(out, product_block)
        second_bytes = out.read_bytes()

        assert first_bytes == second_bytes
        # direct proof of the _wrap trailing-space fix: no line in the file
        # ends with a trailing space (would previously only be true after
        # the trailing-whitespace pre-commit hook ran).
        text = first_bytes.decode("utf-8")
        assert all(not line.endswith(" ") for line in text.splitlines())

    def test_write_manifest_rejects_duplicate_names(self, tmp_path: Path) -> None:
        # FOLD-IN 2 (Task 2 review): not reachable via the real ARTIFACTS
        # today, but a future builder assumes name-uniqueness — guard it.
        dup_entries: list[dict[str, object]] = [
            {
                "name": "x",
                "format": "csv",
                "source_table": "x",
                "mode": "generate",
                "rows": 1,
                "sha256": "a" * 64,
                "home": "src/babylon/data/reference/x.csv",
                "material_relation": "r1",
            },
            {
                "name": "x",
                "format": "csv",
                "source_table": "x2",
                "mode": "generate",
                "rows": 1,
                "sha256": "b" * 64,
                "home": "src/babylon/data/reference/x2.csv",
                "material_relation": "r2",
            },
        ]
        with pytest.raises(ArtifactError, match="x"):
            _write_manifest(dup_entries, path=tmp_path / "data-artifacts.yaml")

    @staticmethod
    def _schema_block_lines(text: str) -> list[str]:
        """The exact lines of the manifest's ``schema:`` block (its key plus
        every indented line directly under it) — used to pin the block's
        bytes untouched by an unrelated rewrite, stricter than parsed-YAML
        equality."""
        lines = text.splitlines()
        start = lines.index("schema:")
        end = start + 1
        while end < len(lines) and lines[end].startswith("  "):
            end += 1
        return lines[start:end]

    def test_update_product_block_preserves_schema_block_bytes(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
        product_block: dict[str, object],
    ) -> None:
        # FOLD-IN (Task 3 review, Minor): the existing preservation test
        # (test_update_product_block_preserves_existing_schema_block) only
        # checks parsed-YAML equality; this pins the schema section's exact
        # bytes are untouched by a product-only rewrite.
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, schema_entry=schema_block, path=out)
        before = self._schema_block_lines(out.read_text())

        update_product_block(out, product_block)

        after = self._schema_block_lines(out.read_text())
        assert after == before

    def test_update_schema_block_adds_schema_preserving_artifacts_section(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
    ) -> None:
        # update_schema_block is the mirror image of update_product_block
        # (Task 4's counterpart) — same artifacts-preservation contract.
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, path=out)  # no schema block yet
        before_text = out.read_text()
        before_artifacts_section = before_text.split("\nartifacts:\n", 1)[1]

        update_schema_block(out, schema_block)

        after_text = out.read_text()
        after_artifacts_section = after_text.split("\nartifacts:\n", 1)[1]
        assert after_artifacts_section == before_artifacts_section

        parsed = yaml.safe_load(after_text)
        assert parsed["schema"] == schema_block
        assert "product" not in parsed  # wasn't present before; stays absent

    def test_update_schema_block_preserves_existing_product_block(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
        product_block: dict[str, object],
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(real_entries, product_entry=product_block, path=out)

        update_schema_block(out, schema_block)

        parsed = yaml.safe_load(out.read_text())
        assert parsed["schema"] == schema_block
        assert parsed["product"] == product_block

    def test_update_schema_block_missing_manifest_is_loud(self, tmp_path: Path) -> None:
        with pytest.raises(ArtifactError, match="manifest missing"):
            update_schema_block(tmp_path / "nope.yaml", {"file": "x"})

    def test_update_product_block_missing_manifest_is_loud(self, tmp_path: Path) -> None:
        with pytest.raises(ArtifactError, match="manifest missing"):
            update_product_block(tmp_path / "nope.yaml", {"name": "x"})


class TestManifestRewriteAtomicity:
    """Task 4's BINDING atomicity fix: ``main()``'s manifest rewrite must
    never silently drop an existing ``schema``/``product`` block once one
    exists (the Task 3 review finding — a plain ``_write_manifest(entries)``
    would still pass the coverage sentinel afterward, since a schema-less
    manifest is valid, making the drop invisible). Exercises
    ``_rewrite_manifest_preserving_blocks`` — the actual seam ``main()``
    calls — not a hand-rolled substitute.
    """

    @pytest.fixture()
    def real_entries(self) -> list[dict[str, object]]:
        # Filtered like TestManifestV2Writer's fixture of the same name (see
        # its docstring): entries the writer round-trips faithfully, not the
        # Vol II Unit U2 hand-maintained LODES entries whose generator field
        # this writer doesn't preserve.
        manifest = yaml.safe_load(_MANIFEST.read_text())
        return [
            entry
            for entry in manifest["artifacts"]
            if entry["generator"] == "tools/make_data_artifacts.py"
        ]

    @pytest.fixture()
    def schema_block(self) -> dict[str, object]:
        return {
            "file": "dist/data-artifacts/schema.sql",
            "sha256": "a" * 64,
            "tables": 76,
            "views": 8,
            "indexes": 100,
        }

    @pytest.fixture()
    def product_block(self) -> dict[str, object]:
        return {
            "name": "marxist-data-3NF.sqlite",
            "sha256": "b" * 64,
            "page_size": 4096,
            "application_id": 1112359244,
            "user_version": 1,
            "sqlite_version": "3.46.1",
        }

    def test_regeneration_preserves_both_blocks_byte_for_byte(
        self,
        tmp_path: Path,
        real_entries: list[dict[str, object]],
        schema_block: dict[str, object],
        product_block: dict[str, object],
    ) -> None:
        out = tmp_path / "data-artifacts.yaml"
        _write_manifest(
            real_entries, schema_entry=schema_block, product_entry=product_block, path=out
        )

        new_entries = real_entries[:-1]  # a "regeneration" that dropped one entry
        _rewrite_manifest_preserving_blocks(new_entries, manifest_path=out)

        parsed = yaml.safe_load(out.read_text())
        assert parsed["schema"] == schema_block
        assert parsed["product"] == product_block
        assert parsed["artifacts"] == new_entries

        # BYTE-FOR-BYTE (FOLD-IN, Task 4 review): the name promises more than
        # parsed-YAML equality — pin that the atomicity-preserving rewrite
        # produces EXACTLY the same bytes a direct one-shot _write_manifest
        # call would, given the same entries + blocks.
        direct = tmp_path / "direct.yaml"
        _write_manifest(
            new_entries, schema_entry=schema_block, product_entry=product_block, path=direct
        )
        assert out.read_bytes() == direct.read_bytes()

    def test_first_ever_run_with_no_prior_manifest_omits_blocks(
        self, tmp_path: Path, real_entries: list[dict[str, object]]
    ) -> None:
        # No manifest yet is NOT an error here (unlike update_product_block/
        # update_schema_block) — generate() supplies its own fresh entries.
        out = tmp_path / "data-artifacts.yaml"
        assert not out.exists()
        _rewrite_manifest_preserving_blocks(real_entries, manifest_path=out)
        parsed = yaml.safe_load(out.read_text())
        assert "schema" not in parsed
        assert "product" not in parsed
        assert parsed["artifacts"] == real_entries
