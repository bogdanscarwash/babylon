#!/usr/bin/env python3
"""Reference SQLite CI-subset generator (Program 14/15 Phase 6, owner item 40).

The reference database (``data/sqlite/marxist-data-3NF.sqlite``) is 5.7 GB —
too large to check into git or fetch cheaply on every CI run. This tool
builds a much smaller, byte-reproducible subset for CI (target: comfortably
under the 2 GB GitHub release-asset ceiling) by applying a reviewable,
module-level table policy (:data:`TABLE`) to every table in the source:

* ``"full"`` — copy every row (small dimension/bridge tables, tiny national
  fact tables, and three tables real tests assert *national* coverage over —
  the "BLOCKED-FULL" trio: ``dim_county_geometry``, ``fact_bea_county_gdp``,
  ``fact_qcew_county_rollup``).
* ``"michigan"`` — copy only rows tied to a Michigan county (state FIPS
  ``26``), via one or more ``county_id``-typed FK columns resolved against
  ``dim_county.fips``. May also carry a reviewable REGRESSION-GUARD
  CARVE-OUT (``extra_fips``) — a fixed list of non-Michigan county FIPS
  codes a specific test depends on (e.g. ``fact_qcew_annual``'s spec-098
  0-vs-None guard row) that would otherwise be dropped by the state filter.
* ``"skip"`` — drop the table entirely, documented with a reason (either it
  is read by nothing in ``src/`` or CI-relevant tests, or a sibling recon
  confirmed its real content is never read in any test).

Every ``fact_``/``dim_``/``bridge_`` table in the source MUST have a policy
entry. An unclassified table matching those prefixes is a hard, loud error
(Constitution III.11) — the trove growing a new table must force policy
review, not silently ship the table full, subsetted, or dropped by accident.

Source DB is opened genuinely read-only (SQLite ``mode=ro`` URI) — this
tool never writes to it, and the OS/driver enforces that even if the code
did not.

Usage::

    uv run python tools/make_reference_subset.py \\
        --output /path/to/reference-subset.sqlite \\
        --manifest /path/to/manifest.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import sys
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Literal

#: Repo root, resolved relative to this file (mirrors tools/run_pip_audit.py).
REPO_ROOT: Path = Path(__file__).resolve().parent.parent

#: Default source database — the read-only, never-written-to reference DB.
DEFAULT_SOURCE: Path = REPO_ROOT / "data" / "sqlite" / "marxist-data-3NF.sqlite"

#: Michigan's two-digit state FIPS prefix.
MICHIGAN_FIPS_PREFIX: str = "26"

#: sha256 streaming chunk size and a fixed upper bound on chunk count (Power-
#: of-10 rule 2 — no unbounded loop). 8192 * 1 MiB = 8 GiB, well above the
#: ~2 GiB release-asset ceiling this tool targets; exceeding it is a loud
#: failure, not silent truncation.
_HASH_CHUNK_BYTES: int = 1_048_576
_MAX_HASH_CHUNKS: int = 8192

Scope = Literal["full", "michigan", "skip"]


@dataclass(frozen=True)
class TablePolicy:
    """One table's classification in the reviewable :data:`TABLE` policy dict.

    :param scope: ``"full"`` (copy all rows), ``"michigan"`` (copy rows tied
        to a Michigan county via ``county_columns``), or ``"skip"`` (table
        dropped entirely).
    :param reason: Human-readable justification — never blank, so the policy
        dict stays reviewable data rather than buried logic.
    :param county_columns: For ``"michigan"`` scope, the FK column name(s) on
        this table that reference ``dim_county.county_id`` (OR-combined —
        multiple columns keep a row if *any* side is Michigan, e.g. a
        commuter-flow pair). Must be empty for ``"full"``/``"skip"``.
    :param extra_fips: For ``"michigan"`` scope, additional 5-digit county
        FIPS codes to preserve regardless of state — a reviewable
        REGRESSION-GUARD CARVE-OUT for rows outside Michigan that a specific
        test depends on (e.g. ``"01011"``, Bullock County AL, spec-098's
        confirmed-zero-employment guard on ``fact_qcew_annual``). OR-combined
        into the same Michigan subquery, so it composes with multi-column
        ``county_columns`` the same way the state filter does. Must be empty
        for ``"full"``/``"skip"``.
    """

    scope: Scope
    reason: str
    county_columns: tuple[str, ...] = field(default=())
    extra_fips: tuple[str, ...] = field(default=())

    def __post_init__(self) -> None:
        if not self.reason.strip():
            raise ValueError("TablePolicy.reason must not be blank")
        if self.scope == "michigan" and not self.county_columns:
            raise ValueError(
                "TablePolicy(scope='michigan') requires at least one entry in county_columns"
            )
        if self.scope != "michigan" and self.county_columns:
            raise ValueError(
                f"TablePolicy(scope={self.scope!r}) must not set county_columns "
                "(only 'michigan' scope filters by county)"
            )
        if self.scope != "michigan" and self.extra_fips:
            raise ValueError(
                f"TablePolicy(scope={self.scope!r}) must not set extra_fips "
                "(only 'michigan' scope supports a regression-guard carve-out)"
            )
        for fips in self.extra_fips:
            if len(fips) != 5 or not fips.isdigit():
                raise ValueError(f"extra_fips entries must be 5-digit FIPS codes, got {fips!r}")


# ---------------------------------------------------------------------------
# Shared reason strings (DRY — reused across structurally-identical entries).
# ---------------------------------------------------------------------------

_DIM_BRIDGE_REASON = (
    "Dimension/bridge table — small, kept FULL per ruled policy (no "
    "per-row Michigan benefit; needed for joins across every scope)."
)

_UNREFERENCED_REASON = (
    "Present only as an ORM declaration in src/babylon/reference/schema.py; "
    "no src/ module or test reads it (only tools/full_database_audit.py, a "
    "standalone audit script never exercised in CI) — SKIP per the "
    "'not referenced by src/ or a CI-relevant test -> SKIP' rule."
)

# ---------------------------------------------------------------------------
# The policy. Every fact_/dim_/bridge_ table in the source DB must appear
# here (see find_unknown_tables / generate_subset's loud-failure check).
# ---------------------------------------------------------------------------

TABLE: dict[str, TablePolicy] = {
    # -- dim_* — ALL full (small; see _DIM_BRIDGE_REASON), a few with a more
    #    specific reason where a real test depends on their exact content. --
    "dim_asset_category": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_bea_industry": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_bea_io_table_type": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_cfs_area": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_coercive_type": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_commodity": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_commodity_metric": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_country": TablePolicy(
        "full",
        "263 rows; needed for the bloc-id {1,7,9,12} filter in "
        "postgres_initialization.py/gamma_hydration.py and dim_country.cty_code "
        "joins in fact_trade_monthly reads.",
    ),
    "dim_county": TablePolicy(
        "full",
        "3,285 rows (123 KB); the join target for every Michigan filter in "
        "this policy — must always be complete, and is tiny regardless.",
    ),
    "dim_county_geometry": TablePolicy(
        "full",
        "BLOCKED-FULL: tests/integration/test_michigan_reference_data.py::"
        "test_us_wide_geometry_coverage asserts COUNT(*) >= 3000 WHERE "
        "geometry_wkt IS NOT NULL — a Michigan-only cut (83 rows) fails "
        "outright. Ship complete (184.1 MB, largest table in the subset).",
    ),
    "dim_data_source": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_education_level": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_employment_area": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_employment_status": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_fred_series": TablePolicy(
        "full", "41 rows; joined by fact_fred_national reads in sqlite_hydrator.py."
    ),
    "dim_gender": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_geographic_hierarchy": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_housing_tenure": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_import_source": TablePolicy("full", "0 rows in source — trivially full."),
    "dim_income_bracket": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_industry": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_metro_area": TablePolicy(
        "full",
        "Small; test_michigan_reference_data.py::test_detroit_msa_exists and "
        "test_michigan_has_sufficient_msas depend on its content.",
    ),
    "dim_occupation": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_ownership": TablePolicy(
        "full", "7 rows; needed to resolve own_code='0' Total rows everywhere."
    ),
    "dim_poverty_category": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_race": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_rent_burden": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_sctg_commodity": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_state": TablePolicy(
        "full", "52 rows; resolves michigan_state_id in Michigan-scoped tests."
    ),
    "dim_time": TablePolicy("full", "485 rows; the year/annual dimension every fact table joins."),
    "dim_wealth_class": TablePolicy("full", _DIM_BRIDGE_REASON),
    "dim_worker_class": TablePolicy("full", _DIM_BRIDGE_REASON),
    # -- bridge_* — full except bridge_lodes_block (deviation, see reason). --
    "bridge_cfs_county": TablePolicy(
        "full", "3,148 rows since PER-31 (2026-09-04); still small — trivially full."
    ),
    "bridge_county_h3": TablePolicy(
        "full",
        "OWNER'S CALL (documented per task): already 93.6% Michigan "
        "(45,655/48,764 rows) — Michigan-only would save ~90 KB on a 1.46 MB "
        "table. Kept full for simplicity; the near-zero size delta isn't "
        "worth a bespoke filter path, and it keeps Keweenaw (26083, the "
        "test's >=500-cell check) and any adjacency-relevant non-MI cells "
        "intact for test_michigan_reference_data.py.",
    ),
    "bridge_county_metro": TablePolicy(
        "full", "Small; feeds the Detroit-MSA test alongside dim_metro_area."
    ),
    "bridge_naics_bea": TablePolicy("full", _DIM_BRIDGE_REASON),
    # -- fact_* — Michigan-scoped (county-linked, actually read). --
    # (fact_census_income is the one exception below: FULL, not Michigan —
    # see its own policy for why.)
    "fact_qcew_annual": TablePolicy(
        "michigan",
        "County-scoped QCEW employment/wages; read by "
        "reference_data_cache.py, county_aggregation.py, hex_hydrator.py, "
        "throughput/adapters.py (single-county queries). REGRESSION-GUARD "
        "CARVE-OUT: extra_fips=('01011',) preserves Bullock County, AL — "
        "spec-098's confirmed-zero-employment guard row (NAICS 21, 2024), "
        "which a plain Michigan cut would drop. Guarded directly by "
        "tests/integration/economics/throughput/test_adapters.py::"
        "test_get_county_naics_employment_real_zero_is_not_none and "
        "::test_get_county_employment_by_naics_includes_confirmed_zero_sector.",
        county_columns=("county_id",),
        extra_fips=("01011",),
    ),
    "fact_census_income": TablePolicy(
        "full",
        "Wave 6 C3: read by SQLiteCensusIncomeSource (bracket ratio), which "
        "queries ANY county nationwide (Constitution Amendment R canonical "
        "scale), not just Michigan — a 'michigan' scope would silently starve "
        "get_county_bracket_ratio for every non-MI county. Also read by "
        "reference_data_cache.py, county_aggregation.py, hex_hydrator.py. "
        "7.2M rows — the largest census table; kept full anyway per the "
        "nationwide-adapter requirement.",
    ),
    "fact_county_exposure_by_external": TablePolicy(
        "michigan",
        "Per-county exposure-to-bloc weights; read by "
        "domain/economics/county_exposure.py, filtered per-county in Python.",
        county_columns=("county_id",),
    ),
    "fact_census_rent": TablePolicy(
        "michigan",
        "County-scoped median rent; read by sqlite_hydrator.py (fact_census_rent join dim_county).",
        county_columns=("county_id",),
    ),
    "fact_coercive_infrastructure": TablePolicy(
        "michigan",
        "County-scoped carceral facility counts; read by hex_hydrator.py.",
        county_columns=("county_id",),
    ),
    "fact_broadband_coverage": TablePolicy(
        "michigan",
        "County-scoped broadband coverage; read by hex_hydrator.py and "
        "consumed via domain/geography/internet.py.",
        county_columns=("county_id",),
    ),
    "fact_census_institutional_ownership": TablePolicy(
        "michigan",
        "Found via verification grep beyond the original recon inventory: "
        "read directly by tests/integration/test_db_initialization_queries.py "
        "(Michigan-data assertion joining county_id -> dim_county.state_id). "
        "168 of 6,570 rows are Michigan. WARNING (2026-07-15, owner-queue "
        "item 59): every numeric column is 0 in ALL rows, coverage 2010-2011 "
        "only — a Feature-021 loader placeholder. Do NOT wire an engine "
        "consumer to this table until real ownership data lands.",
        county_columns=("county_id",),
    ),
    "fact_lodes_commuter_flow": TablePolicy(
        "michigan",
        "Found via verification grep beyond the original recon inventory: "
        "county-to-county commuter OD, read by "
        "throughput/adapters_lodes.py (SQLiteLODESCommuterFlowSource, "
        "single-FIPS queries) and tested directly by "
        "test_throughput_validation.py + test_db_initialization_queries.py. "
        "Filtered on home_county_id OR work_county_id so inbound/outbound "
        "cross-border commuting into Michigan survives the cut "
        "(69,419 of 2,645,347 rows).",
        county_columns=("home_county_id", "work_county_id"),
    ),
    "fact_asm_manufacturing_annual": TablePolicy(
        "skip",
        "PER-30 staged data (2026-09-05): ASM 2022-benchmark national x NAICS "
        "manufacturing rows (2,592; inventories by stage, shipments, cost of "
        "materials, hours) — staged for the Gate 4 productive circuit, not "
        "yet read by any system or CI-relevant test. " + _UNREFERENCED_REASON,
    ),
    # -- fact_* — BLOCKED-FULL (national test assertions). --
    "fact_bea_county_gdp": TablePolicy(
        "full",
        "BLOCKED-FULL: tests/integration/economics/throughput/test_adapters.py"
        "::test_get_all_counties_returns_dict asserts len(get_all_counties())"
        " > 3000 (reads every county); "
        "tests/integration/economics/test_melt_adapters.py::"
        "test_get_gdp_returns_real_total sums this table nationally as the "
        "MELT fallback (the primary fact_bea_national_industry line_number=1 "
        "row is empty in this DB). A Michigan-only cut fails both. Ship "
        "complete (45.8 MB).",
    ),
    "fact_qcew_county_rollup": TablePolicy(
        "full",
        "BLOCKED-FULL: tests/integration/economics/test_melt_adapters.py::"
        "test_get_national_employment_returns_real_total sums "
        "SUM(employment) nationally (own_code='0') as the QCEW total; a "
        "Michigan-only cut returns ~2.7% of the real total and fails the "
        "[100M, 200M] sanity bound. Ship complete (6.27 MB).",
    ),
    # -- fact_* — FULL, national and tiny (actually read by src/). --
    "fact_bea_national_industry": TablePolicy(
        "full",
        "National fallback-source table for melt/adapters.py "
        "SQLiteBEANationalGDPSource; tiny (1,065 rows, 41 KB).",
    ),
    "fact_fred_national": TablePolicy(
        "full",
        "National macro series read by sqlite_hydrator.py (joined with "
        "dim_fred_series); tiny (1,395 rows, 33 KB).",
    ),
    "fact_hickel_erdi_annual": TablePolicy(
        "full",
        "ERDI/tribute bootstrap path (postgres_initialization.py, "
        "gamma_hydration.py, scale_type='Intensive'); load-bearing for 20+ "
        "integration tests (test_phi_attribution, "
        "test_circulation_determinism, test_two_phase_initialization, ...); "
        "tiny (58 rows, 12 KB).",
    ),
    "fact_bilateral_trade_annual": TablePolicy(
        "full",
        "Bloc-filtered (_DISJOINT_BLOC_IDS {1,7,9,12}) trade table feeding "
        "_bootstrap_external_nodes; load-bearing for the same 20+ "
        "integration tests as fact_hickel_erdi_annual; tiny (120 rows, 12 KB).",
    ),
    "fact_ricci_unequal_exchange_gvc": TablePolicy(
        "full",
        "P26 U5b (ADR165 D2/D3): Ricci (2019) GVC unequal-exchange transfer "
        "series grounding the sigma-gradient pipeline's world_stats; "
        "national/region-aggregate, not county-mappable; trivially full "
        "(51 rows, ~5 KB).",
    ),
    "fact_bea_io_coefficient": TablePolicy(
        "full",
        "National Leontief IO coefficients read by sqlite_hydrator.py; no "
        "direct real-DB test hit found, but tiny (131,239 rows, 4.7 MB) — "
        "kept full, low cost.",
    ),
    "fact_bea_final_demand_annual": TablePolicy(
        "full",
        "National BEA final-demand table read by "
        "domain/economics/melt/gamma_hydration.py; tiny (2,044 rows, 37 KB).",
    ),
    "fact_trade_monthly": TablePolicy(
        "full",
        "National monthly trade series (joined dim_country) read by "
        "sqlite_hydrator.py; tiny (44,808 rows, 1.34 MB).",
    ),
    "fact_fred_wealth_levels": TablePolicy(
        "full",
        "Found via verification grep beyond the original recon inventory: "
        "national wealth-levels table read directly by "
        "tests/integration/test_db_initialization_queries.py "
        "(hydration-year checks); tiny (720 rows, 20 KB).",
    ),
    # -- fact_* — explicit SKIP (recon-confirmed zero real-DB test coverage). --
    "fact_faf_commodity_flow": TablePolicy(
        "full",
        "Read by sqlite_hydrator.py — which IS the headless runner's "
        "hydration path: the retired strict 5-tick Python bundle died "
        "ENGINE_FAILURE 'no such table: fact_faf_commodity_flow' without it "
        "(proven on the ci-data-v1 proving run, 2026-07-11). The original "
        "skip reasoning ('zero real-DB test coverage') missed that the "
        "Determinism Bundle CI job exercises the real engine. Its county "
        "bridge (bridge_cfs_county) was empty when this policy was written, "
        "so FAF-zone rows could not be Michigan-scoped — FULL copy "
        "(~97 MiB, 2.49M rows).",
    ),
    # -- fact_* — default SKIP: not referenced by any src/ module or test. --
    "fact_bls_unemployment_decomposition": TablePolicy(
        "full",
        "Wave 6 D8: read by SQLiteBLSUnemploymentSource "
        "(domain/economics/throughput/adapters.py) — per-county BLS LAUS "
        "U-3 wired into the tick pipeline's unemployment_rate via "
        "services.unemployment_source (web bridge + headless runner); "
        "tiny (51,404 rows, ~1 MB).",
    ),
    "fact_census_education": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_census_employment": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_census_hours": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_census_housing": TablePolicy(
        "full",
        "Wave 6 C2: read by SQLiteCensusHousingSource "
        "(domain/economics/throughput/adapters.py) — per-county ACS housing "
        "tenure (renter share) wired into the tick pipeline's renter_share "
        "via services.housing_source (web bridge); 1.35M rows.",
    ),
    "fact_census_income_sources": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_census_median_income": TablePolicy(
        "michigan",
        "ADR075 ruling-1 FILL (2026-07-17): housing pair reviving the repaired "
        "view_rent_crisis; refdb contracts in "
        "tests/unit/reference/test_marxian_views.py pin Wayne County slices. "
        "MI slice keeps the subset small (314K rows national).",
        county_columns=("county_id",),
    ),
    "fact_census_poverty": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_census_rent_burden": TablePolicy(
        "michigan",
        "ADR075 ruling-1 FILL (2026-07-17): housing pair reviving the repaired "
        "view_rent_crisis (burden brackets aggregate to cost-burdened counts); "
        "refdb contracts in tests/unit/reference/test_marxian_views.py. "
        "MI slice keeps the subset small (450K rows national).",
        county_columns=("county_id",),
    ),
    "fact_census_worker_class": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_commodity_flow": TablePolicy(
        "skip",
        "Distinct from fact_faf_commodity_flow (which IS read, but only by "
        "production ingestion + MagicMock-only tests — see that entry). " + _UNREFERENCED_REASON,
    ),
    "fact_commodity_observation": TablePolicy(
        "full",
        "Program 22 Wave 1 (2026-07-17): 4,735 EAV rows (85 commodities x 593 "
        "metrics, 2020-2024) via python -m babylon_data.materials; base of the "
        "KEEP view_critical_materials; tiny — ship complete.",
    ),
    "fact_eviction_lab_filing": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_foreclosure_rate": TablePolicy("skip", _UNREFERENCED_REASON),
    "fact_fred_wealth_shares": TablePolicy(
        "full",
        "Un-orphaned 2026-07-16: Fed DFA net-worth shares (SCF-benchmarked) "
        "are the redundant-source corroboration for the WID wealth-"
        "distribution invariants — read by tests/unit/reference/"
        "test_fred_wealth_shares.py (requires_reference_db lane); tiny "
        "(240 net-worth rows + siblings, 2010Q1-2024Q4).",
    ),
    "fact_hpms_road_segment": TablePolicy(
        "skip",
        "Transport Substrate (Program 11) staged data — not yet wired into "
        "any system as of this writing. " + _UNREFERENCED_REASON,
    ),
    "fact_qwi_county_flow": TablePolicy(
        "skip",
        "PER-32 staged data (2026-09-05): LEHD QWI Michigan 2000Q3-2021Q4 "
        "labor flows — staged for the Gate 4 circuit and COVID benchmark, "
        "not yet read by any system or CI-relevant test. " + _UNREFERENCED_REASON,
    ),
    "fact_mineral_employment": TablePolicy(
        "full",
        "Program 22 Wave 1 (2026-07-17): 25 rows (5 years x 5 sectors, T1 "
        "trends) — mining-sector variable capital; trivially full.",
    ),
    "fact_mineral_production": TablePolicy(
        "full",
        "Program 22 Wave 1 (2026-07-17): 15 rows (5 years x metals/industrial/"
        "coal, T1 trends); trivially full.",
    ),
    "fact_productivity_annual": TablePolicy(
        "full",
        "ADR075 ruling-1 FILL (2026-07-17): 17,336 (industry, year) rows via "
        "tools/load_productivity_annual.py (BLS detailed industries) — the base "
        "of view_surplus_value and view_imperial_rent; national coverage is the "
        "point (Fundamental Theorem legs), and it is tiny.",
    ),
    "fact_state_minerals": TablePolicy(
        "full",
        "Program 22 Wave 1 (2026-07-17): 50 rows (2024 state mineral value/"
        "rank/principal commodities, T3) — territorial extraction geography; "
        "trivially full.",
    ),
}


# ---------------------------------------------------------------------------
# Schema introspection.
# ---------------------------------------------------------------------------


def get_source_table_names(conn: sqlite3.Connection) -> list[str]:
    """Return every base-table name in a database (views excluded).

    :param conn: An open connection to the database to inspect.
    :returns: Sorted table names from ``sqlite_master`` where ``type='table'``.
    """
    rows = conn.execute(
        "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name"
    ).fetchall()
    return [str(row[0]) for row in rows]


def get_table_ddl(conn: sqlite3.Connection, table_name: str) -> str:
    """Fetch the verbatim ``CREATE TABLE`` statement for a table.

    :param conn: Open connection to the database that owns the table.
    :param table_name: Table name.
    :returns: The DDL string exactly as SQLite recorded it.
    :raises LookupError: If the table has no ``sqlite_master`` entry.
    """
    row = conn.execute(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?",
        (table_name,),
    ).fetchone()
    if row is None or row[0] is None:
        raise LookupError(f"no CREATE TABLE statement found for {table_name!r}")
    return str(row[0])


def get_index_ddls(conn: sqlite3.Connection, table_name: str) -> list[str]:
    """Fetch named-index ``CREATE INDEX`` statements for a table.

    Auto-indexes implicitly created by ``UNIQUE``/``PRIMARY KEY`` constraints
    have a ``NULL`` ``sql`` column and are recreated automatically when the
    table DDL itself is replayed — only explicitly named indexes need
    copying here.

    :param conn: Open connection to the database that owns the table.
    :param table_name: Table name.
    :returns: DDL strings in ``sqlite_master`` name order (``[]`` if none).
    """
    rows = conn.execute(
        "SELECT sql FROM sqlite_master WHERE type = 'index' AND tbl_name = ? "
        "AND sql IS NOT NULL ORDER BY name",
        (table_name,),
    ).fetchall()
    return [str(row[0]) for row in rows]


# ---------------------------------------------------------------------------
# DDL rewriting — schema-qualify captured DDL to target the attached "dest".
# ---------------------------------------------------------------------------

_CREATE_TABLE_HEADER_RE = re.compile(
    r"(?P<pre>CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?)"
    r'(?P<quote>"?)(?P<name>\w+)(?P=quote)',
    re.IGNORECASE,
)

_CREATE_INDEX_HEADER_RE = re.compile(
    r"(?P<pre>CREATE\s+(?:UNIQUE\s+)?INDEX\s+(?:IF\s+NOT\s+EXISTS\s+)?)"
    r'(?P<quote>"?)(?P<name>\w+)(?P=quote)',
    re.IGNORECASE,
)


def qualify_table_ddl_for_dest(ddl: str, table_name: str) -> str:
    """Rewrite a captured ``CREATE TABLE`` statement to target ``dest``.

    :param ddl: The verbatim DDL captured from the source's ``sqlite_master``.
    :param table_name: The table name the DDL is expected to declare.
    :returns: The DDL with its table name schema-qualified as
        ``dest.<table_name>`` (quoting, if any, preserved on the name itself).
    :raises ValueError: If no ``CREATE TABLE`` header is found, or the
        declared name does not match ``table_name``.
    """
    stripped = ddl.strip()
    match = _CREATE_TABLE_HEADER_RE.match(stripped)
    if match is None:
        raise ValueError(f"not a CREATE TABLE statement: {stripped[:80]!r}")
    if match.group("name") != table_name:
        raise ValueError(
            f"DDL declares {match.group('name')!r}, which does not match "
            f"the expected table name {table_name!r}"
        )
    pre, quote, name = match.group("pre", "quote", "name")
    return f"{pre}dest.{quote}{name}{quote}{stripped[match.end() :]}"


def qualify_index_ddl_for_dest(ddl: str) -> str:
    """Rewrite a captured ``CREATE INDEX`` statement to target ``dest``.

    The indexed table name (``ON <table>``) needs no rewriting — SQLite
    requires an index and its table to live in the same schema, which is
    guaranteed here because the table was already created in ``dest``.

    :param ddl: The verbatim DDL captured from the source's ``sqlite_master``.
    :returns: The DDL with its index name schema-qualified as ``dest.<name>``.
    :raises ValueError: If no ``CREATE INDEX`` header is found.
    """
    stripped = ddl.strip()
    match = _CREATE_INDEX_HEADER_RE.match(stripped)
    if match is None:
        raise ValueError(f"not a CREATE INDEX statement: {stripped[:80]!r}")
    pre, quote, name = match.group("pre", "quote", "name")
    return f"{pre}dest.{quote}{name}{quote}{stripped[match.end() :]}"


# ---------------------------------------------------------------------------
# Michigan WHERE-clause builder.
# ---------------------------------------------------------------------------


def build_michigan_where_clause(
    county_columns: tuple[str, ...], extra_fips: tuple[str, ...] = ()
) -> str:
    """Build the SQL boolean expression selecting Michigan-linked rows.

    Each listed column is OR-combined — a row survives if *any* of its
    county-FK columns resolves to a Michigan county. This preserves
    cross-border relationships (e.g. a LODES commuter-flow row where only
    one side of a home/work county pair is in Michigan).

    :param county_columns: One or more ``county_id``-typed FK column names,
        resolved against ``main.dim_county.fips`` (explicitly schema-
        qualified so the subquery is unambiguous regardless of table-copy
        order against the attached ``dest`` database).
    :param extra_fips: Additional 5-digit county FIPS codes to preserve
        regardless of state (see :attr:`TablePolicy.extra_fips`) — OR-combined
        into the same subquery used to resolve Michigan ``county_id``s, so a
        listed county survives the cut on every listed column. Empty by
        default, reproducing the pre-carve-out clause exactly.
    :returns: A SQL boolean expression (no leading ``WHERE`` keyword).
    :raises ValueError: If ``county_columns`` is empty.
    """
    if not county_columns:
        raise ValueError("build_michigan_where_clause needs at least one county column")
    fips_predicate = f"fips LIKE '{MICHIGAN_FIPS_PREFIX}%'"
    if extra_fips:
        quoted_fips = ", ".join(f"'{fips}'" for fips in extra_fips)
        fips_predicate = f"({fips_predicate} OR fips IN ({quoted_fips}))"
    # noqa justification: MICHIGAN_FIPS_PREFIX and extra_fips both come from
    # the trusted, reviewed module-level TABLE policy dict, not external input.
    subquery = f"SELECT county_id FROM main.dim_county WHERE {fips_predicate}"  # noqa: S608
    return " OR ".join(f"{column} IN ({subquery})" for column in county_columns)


# ---------------------------------------------------------------------------
# Table copy.
# ---------------------------------------------------------------------------


def copy_table(conn: sqlite3.Connection, table_name: str, policy: TablePolicy) -> tuple[int, int]:
    """Copy one table from the attached source (``main``) into ``dest``.

    Recreates the table's DDL and named indexes in ``dest``, then inserts
    either every row (``"full"``) or the Michigan-filtered subset
    (``"michigan"``).

    :param conn: Connection with ``main`` = source and ``dest`` = the
        attached output database.
    :param table_name: Table to copy.
    :param policy: The table's resolved policy; must not be ``"skip"``.
    :returns: ``(kept_rows, source_rows)``.
    :raises ValueError: If called with a ``"skip"``-scoped policy.
    """
    if policy.scope == "skip":
        raise ValueError(f"copy_table called on skip-scoped table {table_name!r}")

    table_ddl = qualify_table_ddl_for_dest(get_table_ddl(conn, table_name), table_name)
    conn.execute(table_ddl)
    for index_ddl in get_index_ddls(conn, table_name):
        conn.execute(qualify_index_ddl_for_dest(index_ddl))

    where_clause = ""
    if policy.scope == "michigan":
        where_clause = (
            f" WHERE {build_michigan_where_clause(policy.county_columns, policy.extra_fips)}"
        )
    # noqa justification (this call + the two below): table_name is a
    # sqlite_master-enumerated identifier, checked against TABLE — not external input.
    conn.execute(
        f"INSERT INTO dest.{table_name} SELECT * FROM main.{table_name}{where_clause}"  # noqa: S608
    )

    kept_row = conn.execute(f"SELECT COUNT(*) FROM dest.{table_name}").fetchone()  # noqa: S608
    source_row = conn.execute(f"SELECT COUNT(*) FROM main.{table_name}").fetchone()  # noqa: S608
    return int(kept_row[0]), int(source_row[0])


# ---------------------------------------------------------------------------
# Manifest + sha256.
# ---------------------------------------------------------------------------


def build_manifest(
    *,
    source_path: Path,
    output_path: Path,
    generated_at: str,
    table_results: dict[str, dict[str, object]],
) -> dict[str, object]:
    """Assemble the manifest dict written alongside the generated subset.

    :param source_path: Source database path (recorded for provenance).
    :param output_path: Generated subset database path.
    :param generated_at: ISO-8601 generation timestamp.
    :param table_results: Per-table ``{scope, reason, kept_rows, source_rows}``.
    :returns: The manifest dict (JSON-serializable).
    """
    return {
        "generated_at": generated_at,
        "source": str(source_path),
        "output": str(output_path),
        "tables": table_results,
    }


def compute_sha256(path: Path) -> str:
    """Compute the SHA-256 hex digest of a file, streaming in fixed chunks.

    :param path: File to hash.
    :returns: Lowercase hex digest.
    :raises RuntimeError: If the file exceeds the fixed chunk-count bound
        (Power-of-10 rule 2 — no unbounded loop; a file this large is itself
        a sign something went wrong upstream, not a reason to loop forever).
    """
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for _ in range(_MAX_HASH_CHUNKS):
            chunk = handle.read(_HASH_CHUNK_BYTES)
            if not chunk:
                break
            digest.update(chunk)
        else:
            raise RuntimeError(
                f"{path} exceeds the maximum hashable size "
                f"({_MAX_HASH_CHUNKS} chunks of {_HASH_CHUNK_BYTES} bytes)"
            )
    return digest.hexdigest()


# ---------------------------------------------------------------------------
# Orchestration.
# ---------------------------------------------------------------------------


def find_unknown_tables(table_names: list[str], policy: dict[str, TablePolicy]) -> list[str]:
    """Find fact_/dim_/bridge_ tables present in the source but unclassified.

    Loud failure (Constitution III.11): a trove update that adds a new
    fact_/dim_/bridge_ table must force policy review, not silently ship the
    table full, subsetted, or dropped.

    :param table_names: Table names present in the source database.
    :param policy: The reviewable :data:`TABLE` policy dict.
    :returns: Sorted names matching a classified prefix but missing a policy
        entry (``[]`` if none).
    """
    from babylon.sentinels.coverage.catalog import GOVERNED_PREFIXES

    prefixes = tuple(p for p in GOVERNED_PREFIXES if p != "view_")
    return sorted(name for name in table_names if name.startswith(prefixes) and name not in policy)


def _open_source_readonly(source_path: Path) -> sqlite3.Connection:
    """Open the source database as a genuinely read-only connection.

    :param source_path: Path to the source SQLite file.
    :returns: A connection opened with ``mode=ro`` — writes raise
        ``sqlite3.OperationalError`` at the driver level, on top of the code
        discipline of never issuing one.
    """
    uri = f"{source_path.resolve().as_uri()}?mode=ro"
    return sqlite3.connect(uri, uri=True)


def _copy_all_tables(
    conn: sqlite3.Connection, policy: dict[str, TablePolicy]
) -> dict[str, dict[str, object]]:
    """Copy every non-skip table and tally every policy-covered table.

    Tables declared in ``policy`` but absent from the source are silently
    skipped here (not an error) — this lets small synthetic test fixtures
    exercise a subset of a larger shared policy dict.

    :param conn: Connection with ``main`` = source, ``dest`` = attached output.
    :param policy: The table policy dict.
    :returns: Per-table manifest entries keyed by table name.
    """
    source_tables = set(get_source_table_names(conn))
    results: dict[str, dict[str, object]] = {}
    for table_name in sorted(policy):
        if table_name not in source_tables:
            continue
        table_policy = policy[table_name]
        if table_policy.scope == "skip":
            # noqa justification: table_name comes from sorted(policy), a trusted dict.
            source_rows = conn.execute(
                f"SELECT COUNT(*) FROM main.{table_name}"  # noqa: S608
            ).fetchone()[0]
            kept_rows = 0
        else:
            kept_rows, source_rows = copy_table(conn, table_name, table_policy)
        results[table_name] = {
            "scope": table_policy.scope,
            "reason": table_policy.reason,
            "kept_rows": kept_rows,
            "source_rows": int(source_rows),
        }
    return results


def _vacuum_output(output_path: Path) -> None:
    """Compact the generated subset file with a standalone ``VACUUM`` pass.

    Also forces ``journal_mode=DELETE``: under WAL, the first plain (non
    read-only) connection checkpoint-settles the file on close and silently
    changes its bytes — which would break the sha256 the fetch composite
    verifies. A DELETE-mode artifact is byte-inert for readers.

    :param output_path: The subset database file to vacuum in place.
    """
    conn = sqlite3.connect(output_path)
    try:
        conn.execute("PRAGMA journal_mode=DELETE")
        conn.execute("VACUUM")
        conn.commit()
    finally:
        conn.close()


def generate_subset(
    source_path: Path,
    output_path: Path,
    manifest_path: Path,
    *,
    policy: dict[str, TablePolicy] = TABLE,
) -> dict[str, object]:
    """Build the Michigan-scoped reference-DB CI subset and its manifest.

    :param source_path: Read-only source SQLite database.
    :param output_path: Destination path for the generated subset (created
        fresh — any existing file at this path is replaced).
    :param manifest_path: Where to write the JSON manifest.
    :param policy: Table policy dict (defaults to the production :data:`TABLE`).
    :returns: The manifest dict that was written.
    :raises FileNotFoundError: If ``source_path`` does not exist.
    :raises ValueError: If ``output_path`` resolves to ``source_path``, or if
        unclassified fact_/dim_/bridge_ tables are found in the source
        (Constitution III.11 Loud Failure).
    """
    if not source_path.is_file():
        raise FileNotFoundError(f"source database not found: {source_path}")
    if output_path.resolve() == source_path.resolve():
        raise ValueError("output_path must not be the same file as source_path")

    if output_path.exists():
        output_path.unlink()

    source_conn = _open_source_readonly(source_path)
    try:
        unknown = find_unknown_tables(get_source_table_names(source_conn), policy)
        if unknown:
            raise ValueError(
                "unclassified reference tables found (policy review required "
                "in tools/make_reference_subset.py TABLE): " + ", ".join(unknown)
            )
        source_conn.execute("ATTACH DATABASE ? AS dest", (str(output_path),))
        table_results = _copy_all_tables(source_conn, policy)
        source_conn.commit()
    finally:
        source_conn.close()

    _vacuum_output(output_path)

    manifest = build_manifest(
        source_path=source_path,
        output_path=output_path,
        generated_at=datetime.now(UTC).isoformat(),
        table_results=table_results,
    )
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")

    digest = compute_sha256(output_path)
    sidecar = output_path.with_name(output_path.name + ".sha256")
    sidecar.write_text(f"{digest}  {output_path.name}\n")

    return manifest


# ---------------------------------------------------------------------------
# CLI.
# ---------------------------------------------------------------------------


def build_arg_parser() -> argparse.ArgumentParser:
    """Build the CLI parser for this generator.

    :returns: A configured :class:`argparse.ArgumentParser`.
    """
    parser = argparse.ArgumentParser(description="Build the Michigan-scoped reference-DB CI subset")
    parser.add_argument(
        "--source",
        type=Path,
        default=DEFAULT_SOURCE,
        help=f"Path to the source reference SQLite DB (default: {DEFAULT_SOURCE})",
    )
    parser.add_argument(
        "--output", type=Path, required=True, help="Path to write the generated subset"
    )
    parser.add_argument(
        "--manifest", type=Path, required=True, help="Path to write the JSON manifest"
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    """Run the subset generator from the command line.

    :param argv: Command-line arguments (defaults to ``sys.argv[1:]``).
    :returns: Process exit code — 2 for input/policy errors, else 0.
    """
    args = build_arg_parser().parse_args(argv)
    try:
        manifest = generate_subset(args.source, args.output, args.manifest)
    except (FileNotFoundError, ValueError) as exc:
        print(f"make_reference_subset error: {exc}", file=sys.stderr)
        return 2

    tables = manifest["tables"]
    assert isinstance(tables, dict)  # noqa: S101 — manifest is our own output
    total_kept = sum(int(entry["kept_rows"]) for entry in tables.values())
    print(f"reference subset written: {args.output} ({total_kept:,} rows kept)")
    print(f"manifest written: {args.manifest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
