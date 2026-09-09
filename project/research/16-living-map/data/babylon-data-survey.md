# babylon-data Survey — What's Mappable for the Living Map

**Surveyed:** 2026-07-11 · **Location:** `/media/user/data/babylon-data/` (read-only source trove)
**Scope:** identify what can ground the frontend's political map, lenses, and gameplay surfaces for
the "living map" build (Program 16). Cross-checked against the repo's `data-catalog.yaml` (III.4
Data Source Traceability) and the runtime reference DB the engine actually reads.

## 0. The single most important finding

**Most of what matters is already ETL'd into `data/sqlite/marxist-data-3NF.sqlite`** (symlinked
from `/media/user/data/babylon-data/sqlite/marxist-data-3NF.sqlite`, 3.4 GB, 97 tables, star
schema, county-grain, 1997–2024). This is not a raw-data survey finding so much as a build-order
finding: for most fact layers the raw CSV/XLSX/geodatabase sources below are the *pre-ETL inputs*,
and the reference DB is the *already-normalized, county-keyed, FIPS/H3-bridged* product a map
pipeline should read from directly rather than re-deriving. The DB even carries a
`geometry_wkt` column and an H3 res-5/res-7 county bridge already built. Raw sources are documented
below for the ~4 layers NOT yet loaded (HPMS roads, minerals) and for lenses that need finer-than-
county grain than the star schema carries (raw LODES block-level, FCC provider-level).

Table inventory (`/media/user/data/babylon-data/sqlite/data_dictionary.md`): 35 dimensions
(~20,000 rows), 36 facts (~53,000,000 rows), 5 bridges (~42,000 rows), 10 views. Time coverage
1997–2024.

## 1. Top-level dataset inventory

| Dataset | Path | Size | Contents | Map layer / lens | Gameplay surface |
|---|---|---|---|---|---|
| **tiger** | `tiger/` | 222M | TIGER/Line county + AIANNH shapefiles | **Base cartography** — county polygons | Territory node boundaries, starting "colonial" county map |
| **natural-earth** | `natural-earth/` | 811M | Packaged Natural Earth vector (SQLite, 186 tables, 10m/50m/110m) | World context, coastline, admin-1 states, roads/rail (backdrop), urban areas, populated places | Non-US context frame, zoom-dependent generalization |
| **sqlite** (reference DB) | `sqlite/marxist-data-3NF.sqlite` | 5.7G on disk (3.4G reported in dict) | ETL'd star schema: 97 tables, county-grain facts 1997–2024 | **Primary data source for nearly every lens below** | Feeds `dim_county_geometry`, class composition, exploitation, repression |
| **lodes** | `lodes/od/` | 7.9G | Census LEHD LODES origin-destination commuter flow, block-level, 2010–2021, all 50 states + DC | Animated commuter-flow arcs (block-level, if needed finer than the pre-aggregated county fact) | Labor mobility, ReserveArmy/atomization inputs |
| **qcew** | `qcew/` | 8.3G | BLS QCEW annual singlefile, county×industry×ownership, 2010–2024 | Employment/wage choropleth by industry | Production system, ImperialRent wage gap |
| **employment_industry** | `employment_industry/2024.annual.by_area/` | 2.2G | Same QCEW data, one CSV per county (3,000+ files), 2024 only, human-readable titles | Same as QCEW, easier per-county lookup | Inspection-panel drill-down (Victoria-3 style "explain this number") |
| **dot** | `dot/` | 30G | HPMS road segments (CSV+GeoJSON, full geometry), NTAD geodatabases: aviation facilities, intermodal freight (air-to-truck, marine ro-ro, pipeline terminals, rail TOFC/COFC), military bases, North American rail network lines | Transport substrate — road/rail/air/pipeline corridors | Program 11 Transport Substrate corridor mesh; `fact_hpms_road_segment` exists in schema but is currently **empty (0 rows)** — raw HPMS CSV is the load source |
| **freight** | `freight/faf/` | 1.3G | FHWA Freight Analysis Framework 5 — state-to-state + county-level commodity flows by mode/commodity 2018–2024, mode-split factors (truck/rail/water/pipeline) | Freight corridor lens, commodity flow arcs | Supply-chain / unequal-exchange visualization; already loaded as `fact_faf_commodity_flow` (2.49M rows) |
| **fcc** | `fcc/downloads/2025-06-30/` | 2.0G | FCC Broadband Data Collection, per-state (53 dirs incl. territories) fixed + mobile broadband summary by geography (place-level) | Telecom infrastructure choropleth | Atomization/Solidarity substrate (connectivity as organizing capacity); already loaded county-level as `fact_broadband_coverage` (3,221 rows) |
| **mobility-atlas** | `mobility-atlas/` | 351M | Opportunity Insights county + commuting-zone intergenerational mobility (kfr = kid family-income-rank outcomes by race/gender/cohort), county covariates | Class mobility / opportunity choropleth | Survival Calculus context, class stratification texture |
| **piketty** | `piketty/` | 5.8G | World Inequality Database (WID) — per-country CSVs, wealth/income shares/percentiles | Global unequal-exchange comparisons (non-US, mostly national scale) | Imperial Rent Φ cross-country grounding, not directly county-mappable |
| **raw_mats** | `raw_mats/` | 16M | USGS Mineral Commodity Summaries 2025 — per-commodity (~90 files) salient statistics + metadata | Strategic materials (non-geographic at this grain; national production/reserves) | Metabolic Rift / raw-materials throughput; `fact_mineral_production` exists but is **empty (0 rows)** |
| **census** | `census/` | 18M | ACS code lists (PDF), CBSA delineation file, principal cities list | Metro-area definitions, ACS variable reference | Bridges county → metro for `bridge_county_metro` |
| **energy** | `energy/` | 13M | EIA Monthly Energy Review tables (national, by fuel type) | National energy mix (not county-grain) | Metabolic Rift ΔB inputs; loaded as `fact_energy_annual` (525 rows) |
| **imperial_rent** | `imperial_rent/country.xlsx` | 1.8M | Country-level imperial rent working table | National/bilateral Φ | Fundamental Theorem grounding |
| **bea** | `bea/` | 163M | BEA county GDP, IO make-use tables, loaders (`.py` files — this is actually source code, not raw data) | County GDP choropleth | `fact_bea_county_gdp` (1.99M rows) already loaded |
| **fixed-assets, gross-output, input-output, intermediate-output, value-added, gdp-by-industry, productivity** | various | 149M–47M each | BEA/BLS national-industry tables (KLEMS, IO Make-Use, Total Requirements, hours/productivity) | **National-industry, not county-geographic** — feeds the production-function math, not the map | Production system coefficients |
| **atus** | `atus/` | 68K | American Time Use Survey — reproductive-labor seed data + loader code | Not geographic | `fact_atus_reproductive_labor` |
| **bls** | `bls/laus/` | 12K | BLS LAUS county unemployment — **file is a stale "Access Denied" HTML, not real data** (failed download) | N/A — needs re-fetch if county unemployment lens wanted beyond QCEW | Flag for remediation, not usable as-is |
| **concordance** | `concordance/` | 56K | BEA-industry ↔ NAICS crosswalk (xlsx) | N/A (industry taxonomy plumbing) | `bridge_naics_bea` |
| **diagnostics** | `diagnostics/` | 484K | Stale pre-load QA CSVs (all-zero row counts) from an earlier ETL run | None — historical/stale, do not trust current-state claims in these files | — |
| **reference** | `reference/` | 292K | Python loader/schema/view source code (not data) | N/A | Defines the 3NF schema itself |
| **external, gross-output, intermediate-output, pce, gdp-by-industry** | small dirs | <4M | Misc BEA/PCE xlsx, mostly empty or single small files | Minor | — |
| loose CSVs at root | `babylon_hickel_final.csv`, `babylon_ricci_final.csv`, `Babylon Simulation_ Long Format...csv`, `Global South Value Drain...csv` | <15K each | Hickel/Ricci unequal-exchange long-format datasets, pre-built for Babylon | National/bilateral unequal-exchange time series (1960–2017, 1995–2007) | Spectrum of Unequal Exchange (Program 10) grounding, not county-mappable |

## 2. Cartography deep-dive (load-bearing for the Lane-Carto build)

### 2.1 TIGER — `tiger/`

Two shapefile sets, both **2024/2025 vintage, national coverage, GCS NAD83 (unprojected
lat/lon)**:

- **`tiger/county/tl_2024_us_county.{shp,shx,dbf,prj,cpg}`** (+ `.zip` of the same, + ISO XML
  metadata). **This is the file the county→TopoJSON pipeline should consume.**
  - Vintage: `tl_2024` (2024 TIGER/Line).
  - Record count: **3,235 counties/county-equivalents** — confirmed by direct DBF header read
    (`numrec=3235`).
  - Projection: `.prj` reads `GEOGCS["GCS_North_American_1983", DATUM["D_North_American_1983",
    SPHEROID["GRS_1980",...]]]` — **geographic (unprojected), NAD83**, degrees. Needs a projection
    step (e.g., Albers Equal Area CONUS, or a web-Mercator/D3 projection with AK/HI insets) before
    it's screen-ready; do not draw it raw in lat/lon and expect proportional Alaska/US ratios.
  - **State/territory coverage — verified by reading all 3,235 DBF records' `STATEFP` field**: 56
    distinct STATEFP codes present — `01`–`56` for the 50 states + DC (`11`), plus `02` (AK), `15`
    (HI), `72` (PR), and `60/66/69/78` (American Samoa, Guam, N. Mariana Islands, US Virgin
    Islands). **National coverage confirmed, AK/HI/PR/territories all present** — this is the full
    TIGER national county file, not a CONUS-only subset. AK/HI/PR will need cartographic
    repositioning (standard inset treatment) since the raw geometry is at true lat/lon — Alaska
    especially will dominate any naive bounding-box fit.
  - Schema (`DBF` fields, confirmed by direct header read): `STATEFP, COUNTYFP, COUNTYNS, GEOID,
    GEOIDFQ, NAME, NAMELSAD, LSAD, CLASSFP, MTFCC, CSAFP, CBSAFP, METDIVFP, FUNCSTAT, ALAND,
    AWATER, INTPTLAT, INTPTLON`. `GEOID` (5-digit FIPS = STATEFP+COUNTYFP) is the join key to
    `dim_county.fips` in the reference DB.
  - No tract-level or state-level TIGER files are present in this trove — county is the finest
    TIGER grain available locally. (Block-level exists implicitly via LODES geocodes and the H3
    bridge, but not as TIGER boundary geometry.)

- **`tiger/aiannh/tl_2025_us_aiannh.{shp,...}`** — American Indian/Alaska Native/Native Hawaiian
  Areas, 2025 vintage, 867 records, same NAD83 geographic projection. Useful as an overlay layer
  (tribal land / sovereignty boundaries) distinct from county polygons — could ground a
  "Sovereignty" system overlay lens (the engine already has a Sovereignty system in the
  Consequences phase).

**No GDAL/`ogr2ogr`/`ogrinfo` is installed in this environment** — verified with `which`. Any
shapefile→GeoJSON/TopoJSON conversion pipeline will need `pyshp`/`shapely`/`geopandas` (pure
Python) or a GDAL install added to the toolchain; note this as a build dependency for Lane-Carto,
not assume `ogr2ogr` is available.

### 2.2 Natural Earth — `natural-earth/packages/natural_earth_vector.sqlite`

Single **packaged SQLite/SpatiaLite database, Natural Earth v5.1.2 (2022-05-13 per CHANGELOG)**,
811M, containing **186 tables** across all three standard scales:

- **10m** (1:10,000,000, most detailed): `ne_10m_admin_0_countries` (258), `ne_10m_admin_0_map_units`,
  `ne_10m_admin_1_states_provinces` (4,596 — includes all US states as separate polygons),
  `ne_10m_admin_2_counties` (3,224 — US/Canada county-equivalent, could serve as a Natural-Earth-only
  fallback/cross-check against TIGER but TIGER is the authoritative source given it has 3,235 vs
  3,224 units and native FIPS fields), `ne_10m_coastline` (4,133 segments), `ne_10m_land`,
  `ne_10m_lakes` (1,355), `ne_10m_lakes_north_america`, `ne_10m_rivers_lake_centerlines` (1,473),
  `ne_10m_rivers_north_america` (4,897), `ne_10m_roads_north_america` (49,183), `ne_10m_railroads`
  + `_north_america` (1,127), `ne_10m_populated_places` (7,342) + `_simple`, `ne_10m_urban_areas`
  (11,878 polygons) + `_landscan`, `ne_10m_time_zones` (120), `ne_10m_ocean`,
  `ne_10m_geography_regions_polys/points` (physical/cultural region labels), `ne_10m_playas`,
  `ne_10m_parks_and_protected_lands_*`, `ne_10m_airports`, `ne_10m_ports`.
- **50m** (1:50,000,000, medium): `ne_50m_admin_1_states_provinces` (294), plus parallel land/lake/
  river/road/urban/coastline tables at this generalization level.
  - **110m** (1:110,000,000, coarsest, whole-world-on-one-screen): `ne_110m_admin_0_countries`
    (177), `ne_110m_admin_1_states_provinces`, `ne_110m_land`, `ne_110m_coastline`, etc.
- Projection: confirmed via `spatial_ref_sys` + `geometry_columns` — **all layers are EPSG:4326
  (WGS 84), geometry type GEOMETRY, WKB encoding**. Same unprojected-degrees situation as TIGER;
  needs the same projection step.

**Recommended use**: 110m/50m for the zoomed-out "world stage" frame (so the player sees the US
inside a plausible globe, not floating in void), 10m for the strategic zoom level's non-US context
(borders, oceans, major rivers/rail as texture), and **TIGER county (not `ne_10m_admin_2_counties`)
for the actual playable US county mesh** since it's the authoritative, higher-record-count,
FIPS-native source the rest of the game's data already keys on. Natural Earth's `ne_10m_roads_
north_america` / `ne_10m_railroads_north_america` are plausible lightweight backdrop-texture
alternatives to the much heavier DOT/NTAD geodatabases if a decorative (non-interactive) transport
texture layer is wanted without the 30G DOT payload.

### 2.3 The reference-DB shortcut: `dim_county_geometry` + `bridge_county_h3`

The star-schema DB already carries derived cartographic products, **verified by direct query**:

- **`dim_county_geometry`** (3,222 rows): `county_id, centroid_lat, centroid_lon, area_sq_km,
  geometry_wkt`. Sample WKT for one county is **46,814 characters** — this is full-resolution
  (TIGER-derived, un-simplified) polygon WKT, not pre-simplified for web rendering. Useable
  directly for server-side geometry but **will need a simplification pass (e.g., Douglas-Peucker
  via `shapely`/`topojson`/`mapshaper`) before shipping to the browser** — do not serve this WKT
  raw to the frontend.
- **`bridge_county_h3`** (48,764 rows): `h3_index, county_id, resolution, coverage_pct`. Confirmed
  **two resolutions present: H3 res-5 and res-7** (`SELECT DISTINCT resolution` → `[5, 7]`) — the
  res-7 figure matches Constitution II.13's "res-8 corridor mesh" language only approximately
  (constitution says res-8; this bridge is res-7/res-5) — **flag this discrepancy for the Transport
  Substrate program owner**, it may mean either the constitution's res-8 hasn't been materialized
  yet or an amendment/correction is needed.
- **`dim_county.h3_res4`** column exists in schema but is **NULL in the sampled rows** — not
  populated.

This means a "hex tiles are deep-zoom, county/state polygons are the base cartography" pipeline
already has its two endpoints half-built in the DB: TIGER/reference-DB WKT for the county mesh, and
an H3 bridge table for the hex deep-zoom layer — the missing piece is the simplification +
TopoJSON-export step, not new data acquisition.

## 3. Recommended map lenses grounded in real, on-disk data

1. **Class Composition** — `fact_census_worker_class` × `dim_worker_class` (900,900 rows). Worker
   class dimension is **pre-coded with MLM-TW-relevant labels**: `proletariat` (private wage/salary,
   employee-of-private-company), `petty_bourgeois` (self-employed incorporated/unincorporated),
   `state_worker` (local/state/federal government), `unpaid_labor`. Choropleth by county, split by
   gender (`dim_gender`) and race (`dim_race`) available. This is the single most on-theme, most
   ready-to-use lens in the trove.

2. **Commuter Flow / Labor Mobility Arcs** — `fact_lodes_commuter_flow` (2,645,347 rows,
   pre-aggregated to county×county×year, split by age bracket and earnings tier) for an
   immediately-usable animated-arc layer; raw block-level LODES OD files (`lodes/od/`, 7.9G, all 50
   states + DC, 2010–2021, `w_geocode/h_geocode/S000/SA0x/SE0x/SI0x` job counts) available if
   finer-than-county granularity is wanted for a deep-zoom arc lens.

3. **Employment & Wages (QCEW)** — `fact_qcew_annual`/`fact_qcew_county_rollup` (240,488 rows) for
   a wage/employment choropleth by NAICS industry, county, year (2010–2024 present both as raw
   annual singlefiles and by-county CSVs). Directly feeds ImperialRent (`W_c` wage side) and
   Production system framing.

4. **Freight & Unequal Exchange Corridors** — `fact_faf_commodity_flow` (2,494,901 rows,
   CFS-area×CFS-area×commodity×mode×year, 2018–2024) for animated freight-corridor arcs by
   commodity class (`dim_sctg_commodity`) and mode; raw FAF5 county-level + mode-split factor CSVs
   available in `freight/faf/` for finer work. Directly grounds Spectrum-of-Unequal-Exchange /
   supply-chain visualization.

5. **Coercive Infrastructure (Repression)** — `fact_coercive_infrastructure` (3,867 rows) ×
   `dim_coercive_type` (15 types: federal/state/local/private/tribal prisons, all 6 armed-service
   branches + National Guard + joint installations). Directly grounds the Survival Calculus
   `P(S|R)` repression term and the "carceral"/"military" facility overlay a Paradox-style map lens
   would want (icon layer, not choropleth).

6. **Housing Precarity (Dispossession)** — `fact_eviction_lab_filing` + `fact_foreclosure_rate`
   (6,570 rows each, county×year) for a Dispossession-system choropleth (filing rate, execution/
   completion rate). Directly names the engine's Dispossession system.

7. **Inequality (Gini)** — `fact_census_gini` (45,045 rows, county×race×year) for a wealth/income
   concentration choropleth, race-splittable.

8. **Telecom / Organizing-Capacity Infrastructure** — `fact_broadband_coverage` (3,221 rows: pct
   25/3, pct 100/20, pct 1000/100 Mbps coverage, provider count) as an Atomization/Solidarity
   substrate texture layer; raw FCC BDC per-state place-level files (`fcc/downloads/2025-06-30/`,
   53 states/territories, fixed + mobile summaries) available for finer grain than county.

9. **Transport Substrate (Program 11)** — raw HPMS road-segment CSV/GeoJSON
   (`dot/HPMS_Spatial_All_Sections_-_2024.csv`, full `LINESTRING` geometry, AADT traffic counts,
   surface/lane/speed attributes) is the **load source for `fact_hpms_road_segment`, currently 0
   rows** — this table exists in the schema but was never populated; loading it is a prerequisite
   for a road-network corridor lens grounded in the reference DB rather than the raw CSV. NTAD
   geodatabases (rail network lines 87M, aviation facilities 8.9M, military bases 7.7M, intermodal
   freight terminals) are additional layers not yet represented in the star schema at all.

10. **Class Mobility / Opportunity** — `mobility-atlas/Table_1_county_trends_estimates.csv` +
    `Table_8_county_covariates.csv` (Opportunity Insights, county-grain, race/gender-split
    intergenerational income-rank outcomes) — not yet loaded into the reference DB but immediately
    joinable via FIPS; would ground a "life chances" lens distinct from raw income/Gini.

## 4. Repo `data/` check

No standalone `data/` directory exists at the repo root of this worktree
(`/home/user/projects/game/babylon/worktrees/living-map/`) — `find`/`ls` confirm this. The
project's data-facing artifacts here are `data-catalog.yaml` (323 lines, the III.4 traceability
catalog — a runtime/fixture manifest with `id/agency/dataset/vintage/granularity/cadence/class`
per source, e.g. `QCEW`, `BEA_GDP`, `BEA_TiVA`) and `ai/database-spec.yaml`. The actual runtime
reference DB (`data/sqlite/marxist-data-3NF.sqlite`) is a symlink into `/media/user/data/
babylon-data/sqlite/` per `MEMORY.md`, consistent with what was surveyed above — nothing staged in
this worktree diverges from or duplicates the trove.

## 5. Caveats / data-quality flags found during the survey

- `bls/laus/la.data.64.County` is **not real data** — it's a saved "Access Denied" HTML page (a
  failed scrape). If county-level unemployment beyond what QCEW offers is wanted, this file needs
  re-fetching, not reuse.
- `diagnostics/*.csv` (e.g., `03_geographic_coverage.csv`, `01_all_tables_inventory.csv`) report
  **all-zero row counts** for tables that are demonstrably populated (verified by direct SQLite
  query in this survey — e.g. `fact_lodes_commuter_flow` shows 0 in the diagnostic CSV but 2.65M by
  direct count). These diagnostics are **stale, pre-ETL snapshots** — do not trust them for
  current-state claims; query the live DB directly.
- `fact_hpms_road_segment` and `fact_mineral_production` are defined in the schema but **currently
  empty (0 rows)** — the raw source data exists on disk (`dot/HPMS_Spatial_All_Sections_-_2024.csv`,
  `raw_mats/commodities/*_salient.csv`) but has not been loaded.
- `freight/faf/county/01_Alabama.zip` (and presumably its sibling per-state files) **fails to open
  with `unzip`** ("End-of-central-directory signature not found") — either corrupt or an
  incomplete/interrupted download; re-download before relying on FAF county-level files if the
  state-level `FAF5.7.1_State_2018-2024.csv` (which opens fine) isn't sufficient.
- `bridge_county_h3` resolutions are **5 and 7**, not res-8 as Constitution II.13 names for the
  Transport Substrate corridor mesh — flagged above in §2.3, worth reconciling with the Program 11
  owner before Lane-Carto assumes res-8 is available.
- No GDAL toolchain (`ogrinfo`/`ogr2ogr`/`gdalinfo`) is installed in this environment; the
  cartography pipeline will need a pure-Python geometry stack (`shapely`, `pyshp`, `geopandas`, or
  a JS-side `mapshaper`/`topojson` step) or GDAL added as a dependency.
