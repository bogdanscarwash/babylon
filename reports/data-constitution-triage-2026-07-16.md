# Babylon Data Constitution — Final Triage Matrix

## 1. Executive counts

**Tables (99):** KEEP 47 · FILL 8 · ARTIFACT_IZE 6 · INVESTIGATE 15 · **AMPUTATE 23** (all *proposed — owner ruling required*)
**Views (10):** KEEP 6 · FILL 3 · INVESTIGATE 1

Adversarial pass overturned **11** census AMPUTATE calls (`refuted=true`): →KEEP ×4 (`fact_census_employment`, `fact_hpms_road_segment`, `dim_employment_area`, `dim_poverty_category`); →FILL ×5 (`fact_census_income_sources`, `fact_census_poverty`, `dim_import_source`, `fact_mineral_employment`, `dim_geographic_hierarchy`); →INVESTIGATE ×2 (`dim_commodity`, `dim_employment_status`). Reviewer soft-downgrades (`refuted=false`, corrected disposition adopted): `fact_census_education`→INV, `fact_census_worker_class`→INV, `fact_energy_annual`→ARTIFACT, `bridge_lodes_block`→ARTIFACT, `staging_arcgis_feature`→ARTIFACT.

## 2. Per-disposition tables

### KEEP (47) — real consumer, healthy data
| table | rows | consumer / lineage |
|---|---|---|
| fact_census_housing | 1.35M | SQLiteCensusHousingSource renter_share (adapters.py:634) / loader-deleted |
| fact_census_income | 7.2M | bracket-ratio + county_aggregation + hex_hydrator; FULL exception / loader-deleted |
| fact_census_employment | 22.5K | **override:** Phase-5.1 gamma_III brief (feat-gamma-atus-adapter.md) planned consumer |
| fact_census_rent | 45K | sqlite_hydrator._copy_rent:508 / loader-deleted |
| dim_income_bracket | 17 | SQLiteCensusIncomeSource bracket_order (adapters.py:865) |
| fact_qcew_annual | 14.67M | 5+ src (hydrators, adapters, PG preflight) / loader live (writer.py) |
| fact_qcew_county_rollup | 240K | MELT+throughput national employment / loader live |
| fact_bls_unemployment_decomposition | 51K | unemployment_source tick pipeline (U-3 only) / loader stale-import |
| dim_industry | 2.66K | every QCEW/BEA adapter; FK anchor / loader dead-import (data survives) |
| fact_bea_county_gdp | 2.0M | hex_hydrator GDP (627) + PG init / trove loader |
| fact_bea_final_demand_annual | 2.04K | Leontief-rent→ImperialRent / loader live (bea_final_demand.py) |
| fact_bea_io_coefficient | 162.9K | inter_industry + production_chain_rent; all 3 types / loader live |
| fact_bea_national_industry | 1.07K | BEAShareLookupService + PG init hydration / loader live |
| bridge_naics_bea | 462 | II.11 concordance bridge / loader live (load_bea_io US3) |
| dim_bea_industry | 107 | tensor_hierarchy + shaikh_bands + hydrator / loader live |
| dim_bea_io_table_type | 3 | inter_industry table_type slice / writer live |
| fact_fred_national | 1.4K | SQLiteCPISource per-tick deflator + 3 MELT sites / loader-deleted |
| fact_fred_wealth_levels | 720 | TestWealthDistribution integration / loader-deleted |
| fact_fred_wealth_shares | 480 | un-orphaned 2026-07-16, test_fred_wealth_shares / loader-deleted |
| dim_fred_series | 41 | joined by every fred_* consumer / loader-deleted |
| dim_wealth_class | 4 | 2 tests + analyze_wealth_distribution / loader-deleted |
| dim_asset_category | 3 | FK for wealth facts + analyze tool / loader-deleted |
| fact_trade_monthly | 45K | sqlite_hydrator._copy_ricci_unequal:365 / loader-deleted |
| fact_bilateral_trade_annual | 120 | Φ-attribution + MELT alpha; fails loud / schema-only (spec-100) |
| fact_hickel_erdi_annual | 58 | MELT gamma + Φ bootstrap + periphery labor / loader live |
| dim_country | 273 | Φ bloc crosswalk + gamma + hydration / partial loader |
| fact_faf_commodity_flow | 2.49M | sqlite_hydrator._copy_faf_freight (REQUIRED; ENGINE_FAILURE if absent) |
| fact_hpms_road_segment | 0 | **override:** RATIFIED Program 11 (Const II.13/Amend O); loader recoverable @4ce7c96a^ |
| fact_lodes_commuter_flow | 2.65M | SQLiteLODESCommuterFlowSource + real Wayne/Oakland tests (adapter dormant in factory) |
| fact_broadband_coverage | 3.2K | hex_hydrator internet attrs / loader live (fcc) |
| fact_coercive_infrastructure | 3.87K | hex_hydrator SUM(facility_count) / loader live (hifld) |
| fact_county_exposure_by_external | 384K | county_exposure.py + 3 tests / loader live (data:exposure) |
| dim_coercive_type | 15 | FK parent of coercive_infrastructure |
| dim_housing_tenure | 3 | SQLiteCensusHousingSource tenure filter / loader live |
| dim_county | 3.28K | pervasive (hydration/persistence/economics) / loader dead-import |
| dim_county_geometry | 3.22K | hex_hydrator area + tiger_ingestion / loader live |
| dim_state | 52 | FK for ~10 tables + tiger / loader dead-import |
| dim_metro_area | 1.12K | MSA zoom-tier tests (no runtime SELECT yet) / loader dead-import |
| bridge_county_h3 | 48.8K | query_hex_claims in hydration / loader live |
| bridge_county_metro | 3.26K | paired w/ dim_metro_area (test-only) / loader dead-import |
| dim_employment_area | 1.6K | **override:** ratified `full` policy + FK; real BLS data (KEEP not AMPUTATE) |
| ingest_checkpoint | 47K | live qcew/writer.py resume mechanism + tests |
| dim_data_source | 21 | FK provenance backbone + integration test |
| dim_ownership | 7 | throughput+MELT own_code filter + coverage sentinel |
| dim_poverty_category | 60 | **override:** owner-RATIFIED 2026-07-16 National-Oppression program consumer |
| dim_race | 10 | throughput adapter race_code='T' filter |
| dim_time | 485 | universal join key, dozens of consumers + 7 views |

### FILL (8) — real/planned consumer, data absent or stub
| table | rows | reason |
|---|---|---|
| fact_census_income_sources | 45K | **override:** Wave-6/Prog-098 census wiring target (epochs-gap-audit:690); 2/13 already wired |
| fact_census_institutional_ownership | 6.57K | all-zero scaffold; test asserts existence + xfail; owner-queue #59 |
| fact_census_poverty | 26.5M | **override:** owner-RATIFIED National-Oppression program (seed doc, ai/state.yaml:87) |
| dim_import_source | 0 | **override:** real deleted loader (f1f7c917) + extant CSV (43 rows) MCS2025_Fig3 |
| fact_mineral_employment | 0 | **override:** deleted loader (etl.py:2139) + staged MCS2025_T1 CSV |
| fact_state_minerals | 0 | hex_hydrator branches on emptiness; spec-065/ADR042 contracted source |
| dim_cfs_area | 132 | DefaultGeographicFlowSource reads it; rows are on-demand stubs (state_id 100% null) |
| dim_geographic_hierarchy | 6.47K | **override:** deleted CFSLoader consumer recoverable @4ce7c96a^; real weight data |

### ARTIFACT_IZE (6) — preserve as artifact, drop from live schema (destructive)
| table | rows | target |
|---|---|---|
| bridge_county_bea_ea | 83 | → CSV (src/babylon/data/reference/, sibling precedent bea_us_labor_share.csv) |
| dim_bea_economic_area | 8 | → CSV (pairs w/ above) |
| fact_ricci_unequal_exchange | 29 | CSV already canonical (babylon_ricci_final.csv); drop/demote DB copy |
| fact_energy_annual | 525 | → parquet + data-catalog; superseded by fact_state_minerals per spec-065 (+dim_energy_series/table lockstep) |
| bridge_lodes_block | 1.15M | → parquet (109MB block crosswalk; design-intent hex-disagg unwired) |
| staging_arcgis_feature | 5.97K | → parquet (per-facility HIFLD provenance behind aggregate) |

### INVESTIGATE (15) — conflicting evidence, owner call
| table | rows | tension |
|---|---|---|
| fact_census_education | 80.5K | dead babylon_data loader (import-rot) = family-scoped repair-vs-drop decision |
| fact_census_hours | 16K | ORPHANED_DATA_INVENTORY D1 cites _FredProductivityAdapter but it reads FRED not this; aggregate_hours 100% NULL |
| fact_census_worker_class | 900K | dead view_class_composition but maps to core Marxian classes; revivable |
| dim_energy_series | 20 | lockstep w/ fact_energy_annual (→ARTIFACT_IZE) |
| dim_energy_table | 14 | lockstep w/ fact_energy_annual (→ARTIFACT_IZE) |
| dim_commodity | 85 | **override:** owner-ratified Prog-09 USGS-minerals stretch (deferred/gated); is_critical loader bug |
| bridge_cfs_county | 0 | docs call it MUST-level Φ path but ImperialRentSystem has no CFS wiring; blocks FAF MI-scoping |
| dim_sctg_commodity | 42 | FK of KEEP fact_faf (can't drop); 100% placeholder names — wire enrichment or accept scaffold |
| fact_eviction_lab_filing | 6.57K | all-zero stub + broken loader vs data-shortages.md wants Eviction Lab for Dispossession/TENANCY |
| fact_foreclosure_rate | 6.57K | all-zero stub vs weak dispossession-finance want (no catalog entry) |
| dim_rent_burden | 11 | healthy live-loader data, zero consumer; dormant view_rent_crisis; HUD-burden named want |
| dim_employment_status | 8 | **override:** Phase-5.1 gamma brief (owner-ruled, unexecuted) plans FactCensusEmployment join |
| dim_gender | 6 | FK of fact_atus (105 rows, Dept-III named source); dual code-scheme artifact |
| dim_occupation | 74 | labor_type 100% NULL — half-built Marxian classification (finish vs drop) |
| dim_worker_class | 22 | marxian_class 18/22 populated, view written, zero consumer — class-composition wiring candidate |

### AMPUTATE (23) — proposed, owner ruling required (see §4)

## 3. Pathology honor roll

**All-zero / placeholder-stub:** `fact_bls_productivity` (5.32K rows, hardcoded-0 loader stub, loader.py:70-81) · `fact_census_institutional_ownership` (6.57K SUM=0, Feature-021) · `fact_eviction_lab_filing` (6.57K, hardcoded 0s) · `fact_foreclosure_rate` (6.57K, hardcoded 0s) · `fact_census_hours` (aggregate_hours 100% NULL) · `dim_occupation.labor_type` (100% NULL → view_labor_type empty) · `dim_cfs_area`/`dim_sctg_commodity` (100% "{code}" stub names, state_id 100% null).

**Empty-under-consumed-view:** `fact_productivity_annual` (0 rows, **never had a loader**, yet `view_imperial_rent` + `view_surplus_value` both SELECT it → §5.1).

**Vestigial `__pre_086`:** `fact_qcew_annual__pre_086` (15.1M rows / ~522MB dead swap-backup; `--drop-backup` tool exists, never run).

**Orphaned big tables (0 real consumers):** `fact_census_poverty` 26.5M (→FILL, ratified consumer) · `fact_census_occupation` 8.1M (AMPUTATE) · `fact_employment_industry_annual` 1.49M (AMPUTATE, 60% zero) · `bridge_lodes_block` 1.15M (→ARTIFACT) · `fact_census_commute` 945K · `fact_census_worker_class` 900K (→INV) · `fact_census_median_income` 314K.

**Broken view:** `view_rent_crisis` — JOIN omits time_id/race_id → 200K+ Cartesian rows/county, `LIMIT 1` times out >180s (twice).

## 4. Owner ruling list (23 AMPUTATE + 6 ARTIFACT_IZE = destructive)

| # | table | rows | recommendation |
|---|---|---|---|
| A1 | fact_qcew_annual__pre_086 | 15.1M | **Run `babylon_data qcew --drop-backup`** (reclaims ~522MB); keep swap mechanism |
| A2 | fact_census_commute | 945K | Drop schema (clean orphan) |
| A3 | fact_census_gini | 45K | Drop (healthy but zero consumer; epoch Gini concept not wired) |
| A4 | fact_census_median_income | 314K | Drop — but 1 consumer of dormant `view_rent_crisis` revives it + rent_burden |
| A5 | fact_census_occupation | 8.1M | Drop (view_labor_type dead; labor_type unclassified) |
| A6 | fact_census_rent_burden | 450K | Drop — pairs with A4 in `view_rent_crisis` (low-effort FILL alt) |
| A7 | fact_qcew_metro_annual | 78K | Drop; re-derivable from same singlefile CSVs |
| A8 | fact_qcew_state_annual | 0 | Drop (empty, dead loader) |
| A9 | fact_employment_industry_annual | 1.49M | Drop **or** FILL if national/MSA grain wanted (dead loader, 60% zero) |
| A10 | fact_bls_productivity | 5.32K | Drop schema; future work extends fact_productivity_annual (⚠ also empty — §5.1) |
| A11 | fact_productivity_annual | 0 | **CONTRADICTION (§5.1)** — table=AMPUTATE, its 2 views=FILL. Escalate |
| A12 | fact_fred_industry_unemployment | 720 | Drop; superseded by BLS-LAUS decomposition |
| A13 | fact_fred_state_unemployment | 765 | Drop; superseded by BLS-LAUS decomposition |
| A14 | fact_hickel_drain | 0 | Drop; superseded by fact_hickel_erdi_annual (TablePolicy comment is false) |
| A15 | fact_mineral_production | 0 | Drop **or** FILL (raw MCS staged; repeatedly named deferred stretch) |
| A16 | fact_commodity_observation | 3.79K | Drop (USGS commodity subtree, view_critical_materials dead) |
| A17 | dim_commodity_metric | 593 | Drop (subtree w/ A16); `dim_commodity`→INVESTIGATE (ratified stretch) |
| A18 | fact_commodity_flow | 0 | Drop (county-disagg never built) |
| A19 | fact_atus_reproductive_labor | 105 | Drop; superseded by data/atus/seed_data.yaml |
| A20 | dim_atus_activity_category | 26 | Drop (subtree w/ A19) |
| A21 | dim_education_level | 26 | Drop (education subgraph dead) |
| A22 | dim_sector | 0 | Drop; redundant w/ dim_industry.class_composition |
| A23 | dim_commute_mode | 22 | Drop (distinct from LODES; no program) |
| R1 | bridge_county_bea_ea + dim_bea_economic_area | 83+8 | → checked-in CSV artifacts |
| R2 | fact_ricci_unequal_exchange | 29 | CSV already canonical; drop/demote DB mirror |
| R3 | fact_energy_annual (+series/table) | 525 | → parquet + catalog; superseded by fact_state_minerals |
| R4 | bridge_lodes_block | 1.15M | → parquet (future hex-disagg) |
| R5 | staging_arcgis_feature | 5.97K | → parquet (facility-grain provenance) |

## 5. Contradictions / uncertainties needing human eyes

1. **`fact_productivity_annual` — table vs views split.** Table reviewers → AMPUTATE (0 rows, *no loader ever existed* in full git history, Program 10 §203 explicitly "not required for slice 1"). Views reviewer → FILL (ORM model exists schema.py:1415; raw `labor-productivity-detailed-industries.xlsx` staged in trove; formula sign-verified vs Fundamental Theorem). **Decide: build a real loader (FILL both views) or drop table + both views.** Note `fact_bls_productivity` (A10) is the wrong table (different schema, all-zero) — not a substitute.
2. **Views KEEP over AMPUTATE/INVESTIGATE bases** — resolve view+base as a unit: `view_critical_materials`(KEEP) ← `fact_commodity_observation`(A16)+`dim_commodity_metric`(A17); `view_class_composition`(KEEP) ← `fact_census_worker_class`(INV); `view_energy_consumption`(KEEP) ← `fact_energy_annual`(R3). Dropping a base orphans the view.
3. **`view_rent_crisis` broken** (missing-time_id combinatorial join, unrunnable) over KEEP/AMPUTATE/AMPUTATE bases → fix predicate (add time_id; rule on race_id) or drop.
4. **`dim_energy_series`/`dim_energy_table`** left INVESTIGATE but census says lockstep w/ `fact_energy_annual` → should follow to ARTIFACT_IZE.
5. **Data-Coverage sentinel bug** (not a disposition): `sentinels/coverage/registry.py:107-116` names `fact_lodes_od` — a table that does not exist — so the LODES coverage check is vacuous.
6. **Stale test precondition:** `test_imperial_rent_real_wiring.py:23` docstring declares IMPORT_USE=0 "KNOWN-RED"; DB now has 31,688 IMPORT_USE rows — cheap re-run to confirm green.
7. **Doc-drift (fix regardless of disposition):** `data_dictionary.md` says "0 rows / Awaiting ETL" for many populated tables (qcew_annual 14.67M, employment, education 80.5K, energy 525, dim_country 273, trade 45K…); `make_reference_subset.py` `_UNREFERENCED_REASON` cites deleted `tools/full_database_audit.py` (commit 0e26fe7e) across ~20 skip entries.
