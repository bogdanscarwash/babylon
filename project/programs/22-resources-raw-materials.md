# Program 22 — Resources & Raw Materials

**Status: CHARTERED** — owner-approved 2026-07-16 (ADR075 `rulings_resolved` ruling 1c, the
minerals amendment to the Data Constitution triage). Wave 1 (data) executes with the Program 21
fill/amputation slate; Wave 2 (engine coupling) is owner-gated and baseline-moving.

**One sentence:** give the complex economy a real resources and raw-materials system by loading
the staged USGS Mineral Commodity Summaries data at three honest altitudes — ~85 minerals
national, 42 SCTG freight-flow classes on the Program 11 transport substrate, per-hex substrate
stocks — with import-reliance × criticality becoming the material basis of imperial dependency
(Φ) in Wave 2.

## Origin

Owner ruling (2026-07-16, verbatim intent): "I want to keep the minerals and the commodity flows
… this complex economy does need a resources and raw materials system … I just want it to be a
realistic and detailed game." Resolution principle, recorded in ADR075: **full resolution where
data exists, honest class-aggregation where it doesn't, never fabricated precision.** This
program supersedes the census's AMPUTATE proposals A15–A18 for the minerals/commodity subtree —
the tables were empty not because the want was fake but because the loader was deleted (spec-037
loader retirement) before it ever ran against the staged trove data.

## The three altitudes

| # | Resolution | Data | Status (verified 2026-07-17) |
|---|-----------|------|------------------------------|
| 1 | ~85 USGS MCS minerals, **national** annual | `raw_mats/` salient CSVs + T1–T5/Fig1–13 aggregates | dims loaded (85 commodities, 593 metrics); `fact_commodity_observation` 3,788 rows (2020–2023); all four target fact/dim tables **empty** |
| 2 | 42 SCTG commodity classes, **FAF zone→zone** flows | `fact_faf_commodity_flow` 2,494,901 rows; `dim_sctg_commodity` 42 rows | loaded AND consumed (`persistence/sqlite_hydrator.py` Determinism-Bundle hydration; `domain/economics/tensor_hierarchy/geographic_flow.py` — implemented+tested, not yet engine-wired) |
| 3 | per-hex substrate stocks | `raw_material_stock` / `energy_stock` / `biocapacity_stock` graph attrs | **slot live, dynamics stubbed**: `SubstrateSystem` @2.5 (`engine/systems/substrate.py:33-46`) seeds stocks 0.0 and passes them through unchanged — "concrete dynamics land with the downstream physical-substrate spec." The extraction/regeneration *rift* (ΔB = R − E·η) runs separately on territories in `MetabolismSystem` (`engine/systems/metabolism.py:33`) |

Altitude 1 is measured data; Altitude 2 is measured data on the Program 11 (Transport Substrate,
spec-108, Constitution II.13) rails; Altitude 3 is the engine seam the first two eventually
calibrate. County-level mineral precision does not exist in MCS and is **not fabricated** —
county resolution comes only from FAF disaggregation, explicitly labeled derived.

## Data inventory (verified paths)

Staging area: `/media/user/data/babylon-data/raw_mats/` (CI/tests never touch this drive —
owner ruling 2026-07-14; data ships as deterministic artifacts).

- `minerals/` — 185 files: **85 per-commodity pairs** `mcs2025-<slug>_salient.csv` +
  `mcs2025-<slug>_meta.xml`, plus the aggregate releases:
  - `MCS2025_T1_Mineral_Industry_Trends.csv`, `MCS2025_T2_Mineral_Economic_Trends.csv`
  - `MCS2025_T3_State_Value_Rank.csv` (→ `fact_state_minerals`)
  - `MCS2025_T4_Critical_Minerals_End_Use.csv`, `MCS2025_T5_Critical_Minerals_Salient.csv`
    (→ authoritative `is_critical` re-derivation)
  - `MCS2025_Fig2_Net_Import_Reliance.csv`, `MCS2025_Fig3_Major_Import_Sources.csv`
    (→ `dim_import_source` + NIR)
  - Fig1/4/10–13 (economy share, value by type, price growth, consumption change, scrap)
- `world/MCS2025_World_Data.csv` (+ `.xml`) — production/capacity/reserves by
  COMMODITY × COUNTRY × TYPE, 2023 + 2024-est (→ import-source country dependency mapping)
- `commodities/` — earlier unzip of the same salient pairs (minus 3 aggregate files);
  `minerals/` is the fuller copy and the canonical input
- Salient CSV shape: `DataSource,Commodity,Year,<per-commodity stat columns>` — columns VARY
  per commodity (aluminum carries 15 incl. `Employment_num`, `NIR_pct`); this wide-varying
  shape is exactly why the schema pairs an EAV fact (`fact_commodity_observation`) with a
  metric dimension (`dim_commodity_metric`)

**Recovered loader** (deleted spec-037, recoverable at `4ce7c96a^`):
`src/babylon/data/materials/{__init__,loader_3nf,parser,schema}.py`. `MaterialsLoader`
(loader_3nf.py) populated `DimCommodity`/`DimCommodityMetric`/`DimTime`/`FactCommodityObservation`
from `data/raw_mats` via `parser.py` (`discover_commodity_files`, `parse_commodity_csv`,
`get_metric_category`); it carried a hardcoded 13-entry `CRITICAL_MATERIALS` dict with Marxian
interpretations. `schema.py` holds the *legacy* census-schema classes (`StateMineral`,
`MineralTrend`, `ImportSource`) that map to the aggregate T/Fig files the 3NF loader never
handled. Revival target namespace: `babylon_data` (the external trove repo is the loader home
since the QCEW-writer migration).

## Target tables (data-catalog.yaml state at charter)

| Table | Disposition | Rows | Wave-1 source |
|---|---|---|---|
| `dim_commodity` | keep | 85 | loaded; **re-derive `is_critical` from T4/T5** — only 2/85 flagged today because the old name-keyed dict missed (e.g. `reare` vs the actual commodity naming) |
| `dim_commodity_metric` | keep | 593 | loaded (from `_meta.xml` / column discovery) |
| `fact_commodity_observation` | keep | 3,788 (2020–2023) | refresh from salient CSVs; close the 2024 gap (salient files carry 2024 rows) |
| `dim_import_source` | fill | 0 | `Fig3_Major_Import_Sources.csv` + `world/MCS2025_World_Data.csv` |
| `fact_mineral_production` | fill | 0 | `T1_Mineral_Industry_Trends.csv` (+T2) |
| `fact_mineral_employment` | fill | 0 | `Employment_num` salient columns (+T1) |
| `fact_state_minerals` | fill | 0 | `T3_State_Value_Rank.csv` |
| `fact_faf_commodity_flow` | keep | 2,494,901 | already loaded+consumed (Altitude 2) |
| `fact_commodity_flow` | fill | 0 | **DERIVED** county disagg of FAF zone flows, employment-weighted, labeled derived-not-measured (catalog note, owner 2026-07-16) |
| `view_critical_materials` | — | view | revives automatically once observations/criticality flow (joins obs × commodity(is_critical, marxian_interpretation) × metric × time) |

Every fill flips its catalog row to `keep` in the same commit (the catalog sentinel's
KEEP-emptiness law then *enforces* the table never silently empties again).

## Waves

**Wave 1 — data (rides the Program 21 fill slate, no engine change):** revive the materials
loader under `babylon_data`, extend it to the aggregate T/Fig/world files, fill the four empty
tables, re-derive criticality, refresh observations through 2024, flip catalog rows, cut the
next ci-data release. DoD: catalog sentinel green with the new keeps; `view_critical_materials`
returns rows; qa:regression 5/5 untouched (reference-DB fills are invisible to the tick).

**Wave 2 — engine coupling (owner-gated, baseline-moving, its own spec):**
1. **Imperial dependency → Φ:** import-reliance (NIR) × criticality as the material basis of
   imperial dependency feeding the imperial-rent circuit — the Third-Worldist reading of the
   critical-minerals panic (Congo cobalt, Chinese rare-earth processing) becomes a measured
   coefficient, not flavor text.
2. **SCTG flows on the transport substrate:** wire `geographic_flow.py`'s tensor hierarchy
   (implemented, tested, unwired) into the engine on Program 11 rails.
3. **Per-hex stock dynamics:** replace SubstrateSystem's pass-through with depletion/regeneration
   coefficients calibrated from Altitudes 1–2.
   All three move tick hashes ⇒ regenerate + declare baselines per the R-PROOF convention;
   sequenced after Wave 1, gated like the wealth-axis Phase 2.

## Open questions

- ~~`fact_commodity_observation` stops at 2023 while salient CSVs carry 2024 rows~~ RESOLVED
  2026-07-17: it was the loader-year config — `LoaderConfig.materials_years` defaulted to
  `range(2015, 2024)`; the CLI passes `range(2015, 2025)`.
- The `commodity_observation` catalog row name vs the actual table `fact_commodity_observation`
  — verified the catalog uses the correct `fact_` name; the shorthand appears only in prose.
- Legacy `materials/schema.py` targets old census-schema table names (`state_minerals`,
  `mineral_trends`, `import_sources`) — Wave 1 rewrites these against the 3NF names rather than
  reviving the legacy classes verbatim.
- Whether `MCS 2025 Data Release Revision File 2025-03-13.txt` contains corrections that
  supersede the staged CSVs — read before loading.

## Next-actions checklist

- [x] Charter (this document) + ADR075 ruling 1c recorded. Catalog rows carry the owner notes.
- [x] Wave 1 (2026-07-17): loader revived under `babylon_data.materials` (imports re-pointed off
      the dead `babylon.data.*`/`babylon.utils.*` namespaces; internal commit removed — the
      `__main__` CLI owns the transaction, dry-run = one rolled-back transaction). New
      `aggregates.py` writers + `python -m babylon_data.materials` CLI. Filled:
      `dim_import_source` 43 / `fact_mineral_production` 15 / `fact_mineral_employment` 25 /
      `fact_state_minerals` 50; observations refreshed 3,788 → 4,735 (2024 gap closed);
      `is_critical` re-derived from T4: 2 → **30** flagged rows (48 official minerals folded onto
      family rows; Cesium + Rubidium have no salient chapter — reported unrepresented);
      `view_critical_materials` alive (1,895 rows). Contracts:
      `tests/unit/reference/test_minerals_tables.py` + `materials/test_materials_cli.py`.
- [ ] Wave 1 residual: `fact_commodity_flow` derived county disagg — **BLOCKED 2026-07-17**:
      requires `bridge_cfs_county` (0 rows, `investigate`, no loader ever) and real
      `dim_cfs_area` definitions (stub rows, 100% null state_id); the FAF zone→county
      equivalency file is not staged in the trove. Unblock = stage the FAF5 zone-equivalency
      (BTS), fill `dim_cfs_area` + `bridge_cfs_county`, then design the disagg scope
      (SCTG classes × geography — a naive national county-pair expansion of 2.49M zone flows
      is combinatorially explosive; needs a scoping ruling).
- [x] Wave 1 close (2026-07-17): **ci-data-v6 released** (subset 1.198GB / 14.19M rows kept,
      + the four ADR076 parquet artifacts, tiger unchanged); fetch-reference-db re-pinned;
      catalog sentinel green both tiers against BOTH the full DB (83/83) and the v6 subset;
      qa:regression 5/5 byte-identical after every surgery batch.
- [ ] Wave 2 spec (owner-gated): Φ coupling, geographic_flow wiring, substrate dynamics.
