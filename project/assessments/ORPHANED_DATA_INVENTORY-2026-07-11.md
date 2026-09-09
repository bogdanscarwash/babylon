# Orphaned Data Inventory — 2026-07-11

**Status:** Factual inventory only. No code changes made. This is the seed document for
Phase-7 data-reconciliation (see `ai/state.yaml` task tracker, item "Phase 6-8: Data
reconciliation").

**Scope:** loaded-but-unconsumed reference-DB tables and registered-but-unread DI adapters
in `src/babylon/domain/economics/`, discovered during the tools-optimization sweep. Every
row count and code citation below was re-verified against the live reference DB and the
worktree source tree on 2026-07-11 (not carried over unverified from the prior sweep).

**DB reached at:** `/media/user/data/babylon-data/sqlite/marxist-data-3NF.sqlite` (the
in-repo `data/sqlite/` symlink does not exist in this worktree — worktrees don't inherit
untracked/symlinked paths from the main checkout — so counts were pulled directly from the
canonical trove location per `data_location.md`).

---

## Summary Table

| Dataset | Rows (verified) | Status | Wiring target | Need verdict |
|---|---|---|---|---|
| `fact_fred_wealth_levels` | 720 | Loaded, zero runtime consumers | Feature-043 endogenous class-shares (`substrate/transitions.py`) as calibration anchor | Owner-gated (D1) |
| `fact_fred_wealth_shares` | 480 | Loaded, zero runtime consumers | Same as above | Owner-gated (D1) |
| `fact_census_hours` | 16,062 | Loaded, zero runtime consumers | Variable-hours (HOANBS) via `_FredProductivityAdapter` | Owner-gated (D1) |
| `_FredProductivityAdapter` / `productivity_data_source` | N/A (DI adapter, not a table) | Registered in DI, zero tick readers | Same as above | Owner-gated (D1) |
| `imperial_rent/country.xlsx` | 8,973 rows / ~253 distinct country-year entities (see discrepancy note) | Unloaded (not in reference DB) | Optional per-country ERDI | Default SKIP — national Hickel path already live |
| `fact_bilateral_trade_annual` | 120 | Loaded, in use (bloc-level, feeds gamma `alpha`) | Already wired — baseline for the country.xlsx decision above | N/A — reference point, not orphaned |
| OES wage distribution | — | Absent from trove | Cannot wire | Cannot wire (no source data) |
| BEA RPP | — | Absent from trove | Cannot wire | Cannot wire (no source data) |
| BLS CEX | — | Absent from trove; correctly stubbed via `NoDataSentinel` | N/A | Already handled correctly (III.11 exemplar) |
| Gamma per-country MVP seam (`MVP_ERDI_VALUES`, `DefaultGammaImportCalculator`, `MVP_IMPORT_SHARES`) | N/A | Orphaned dead code (live path bypasses it) | Superseded by `SQLiteGammaHydrationSource` | Delete candidate (separate from this inventory's scope — flagged only) |
| `estimate_la_share` (`melt/wealth_proxy.py`) | N/A | Deprecated, emits `DeprecationWarning` at call time | Superseded by `check_equity_threshold` + `evaluate_class_shares` | Delete candidate once callers migrate |
| `babylon_data/reference/schema.py` | N/A | Stale frozen fork (343-line diff vs. canonical) | N/A | Reconcile or delete fork |

---

## Per-Item Detail

### 1. `fact_fred_wealth_levels` (720 rows) + `fact_fred_wealth_shares` (480 rows)

**Verified counts** (2026-07-11, direct query against
`/media/user/data/babylon-data/sqlite/marxist-data-3NF.sqlite`):

```
fact_fred_wealth_levels:  720
fact_fred_wealth_shares:  480
```

Both match the prior sweep's reported figures exactly.

**Status:** Loaded into the reference DB (present in `src/babylon/reference/schema.py`), but
`rg -l "fact_fred_wealth" src -g "*.py"` returns only `schema.py` itself — no
adapter, hydrator, or system in `src/babylon/domain/economics/` or `src/babylon/engine/`
reads either table.

**Wiring target:** Feature-043 already implements *endogenous* class-share derivation —
`evaluate_class_shares()` and `check_equity_threshold()` in
`src/babylon/domain/economics/substrate/transitions.py` (lines 148 and 195 respectively).
These derive Labor Aristocracy / Proletariat / Lumpenproletariat / Bourgeoisie shares
directly from `HexTenureComposition` (owner-occupied vs. rental vs. commercial vs.
vacant land), per the FR-005 equity-ratio test (`s / (c + v + s)` vs. a threshold). This is
the *runtime mechanism* — it needs no FRED data to function.

FRED wealth levels/shares should NOT become a second runtime mechanism competing with
Feature 043. The correct role is a **calibration anchor**: FRED's national wealth-share
time series (by percentile bracket) can validate that Feature-043's endogenous,
tenure-driven class shares track real-world wealth concentration over time, and can inform
defines-level tuning of the equity threshold in `GameDefines` — not become a second
per-tick data source read inside the engine.

### 2. `fact_census_hours` (16,062 rows) + `_FredProductivityAdapter` / `productivity_data_source`

**Verified count:**

```
fact_census_hours: 16,062
```

Matches the prior sweep exactly.

**Status:** `fact_census_hours` — same pattern as item 1: present in `schema.py` only, zero
consumers anywhere else in `src/`.

The adapter side is more subtle and was independently verified:
- `ProductivityDataSource` Protocol is defined at
  `src/babylon/domain/economics/working_day/data_sources.py:13`.
- `_FredProductivityAdapter` is implemented at
  `src/babylon/domain/economics/factory.py:619` and instantiated/registered into the
  service container at `factory.py:679` (`"productivity_data_source":
  _FredProductivityAdapter()`).
- The field is threaded through the DI plumbing: declared in
  `src/babylon/kernel/services.py:47` (Protocol) and
  `src/babylon/engine/services.py:163,239,329` (concrete `ServiceContainer`).
- **But** `rg "\.productivity_data_source" src -g "*.py"` returns **zero matches** — nothing
  in any System ever reads the attribute off the service container. The adapter is fully
  wired into DI and never called.

**Wiring target:** the adapter's docstring states its purpose plainly — "derives
WorkingDayState from OPHNFB + HOANBS" (Output Per Hour Nonfarm Business, and Hours of All
Persons Nonfarm Business), i.e. variable-hours modeling. This is the correct wiring target
for both `fact_census_hours` and the adapter: a System (not yet written, or not yet reading
this field) that derives a per-tick or per-year working-day length from productivity data,
rather than treating hours as a flat constant.

**Important boundary:** `HOURS_PER_YEAR = 2080` (documented at
`src/babylon/domain/economics/tensor_hierarchy/leontief_rent/industry_to_county_allocator.py:28`
as "project standard, see CLAUDE.md") is a **unit-conversion constant**, not a competing
model of working hours. It should remain in place for its unit-conversion role even after
variable-hours wiring lands — the two are not mutually exclusive; variable-hours would
modulate around/replace the *behavioral* hours assumption, while 2080 continues to serve
wherever a units conversion (not a class-position or wage calculation) needs an annual-hours
constant.

### 3. `imperial_rent/country.xlsx` — discrepancy flagged

**Prior sweep's claim:** "~140 countries."

**Re-verification result (2026-07-11):** the file is `8,973` rows × `29` columns
(`year`, `CTY_CODE`, `CTYNAME`, 12 monthly import columns, 12 monthly export columns, plus
totals). Grouping by `CTYNAME` yields **259 distinct name values** across **41** distinct
years (1985–2025). Of those 259, at least 6 are non-country aggregates (`European Union`,
`World, Not Seasonally Adjusted`, `World, Seasonally Adjusted`, two `NAFTA with ... (Consump)`
rows, `Advanced Technology Products`), leaving roughly **~253** country/territory-level
entities — not ~140.

**This does not match the prior sweep's figure.** Per the project's verifiability standard
(never let an unverified or now-contradicted claim stand), this discrepancy is flagged for
human resolution rather than silently corrected or silently repeated. Possible explanations
not yet checked: the prior sweep may have counted only a single year's worth of rows, may
have applied a stricter "recognized sovereign state" filter (ISO-3166 country list, dropping
territories/dependencies/historical entities), or may have referenced a different file
revision. Phase-7 should re-derive the number with an explicit filter definition before
using it to scope wiring effort.

**Status:** UNLOADED — no `fact_imperial_rent_country` or equivalent table exists in the
reference DB; only the bloc-level `fact_bilateral_trade_annual` (120 rows, confirmed present,
schema: `time_id, country_id, imports_usd_millions, exports_usd_millions,
total_trade_usd_millions`, PK `(time_id, country_id)`) is loaded and consumed today (it feeds
the gamma `alpha` calculation per `melt/gamma_hydration.py`'s docstring).

**Need verdict:** default **SKIP**. Building a full per-country ERDI/unequal-exchange model
from `country.xlsx` would duplicate work that Hickel's `fact_hickel_erdi_annual` (58 rows,
1980–2016, `scale_type='Intensive'`) already does at the national level, and the national
Hickel path is already live (`SQLiteGammaHydrationSource`,
`src/babylon/domain/economics/melt/gamma_hydration.py:132`). Per-country resolution is a
possible future enhancement, not a current gap — flag as optional, not blocking.

### 4. Absent from trove — cannot wire

- **OES (Occupational Employment Statistics) wage distribution:** not present anywhere
  under `/media/user/data/babylon-data/`. No table, no source file. Cannot be wired without
  first sourcing the data.
- **BEA RPP (Regional Price Parities):** same — absent from trove.
- **BLS CEX (Consumer Expenditure Survey):** absent from trove, but this is the *correct*
  state — the codebase already handles this via `NoDataSentinel` (the documented
  III.11 Loud Failure exemplar: a falsy sentinel object carrying `fips`/`year`/`reason`
  fields rather than silently defaulting). No action needed here; this is cited as a
  positive contrast to the FRED/Census orphans above, which are loaded-but-silent rather
  than absent-and-loud.

### 5. Dead seams (deletion candidates, separate from the load-bearing orphans above)

These were found during the same sweep but are a different category — code to *remove*,
not data to *wire*:

- **Gamma per-country MVP seam:** `MVP_ERDI_VALUES` (`gamma/types.py:237`),
  `MVP_IMPORT_SHARES` (`gamma_import.py:39`), and `DefaultGammaImportCalculator`
  (`gamma_import.py:99`, registered into DI at `protocol_kit.py:258`) form a hardcoded
  per-country ERDI table that predates the real data path. The live path —
  `SQLiteGammaHydrationSource` (`melt/gamma_hydration.py:132`) — reads
  `fact_hickel_erdi_annual` (58 rows, confirmed present) directly and is what
  `DefaultBasketVisibilityCalculator` actually falls back through. The MVP seam is still
  registered in DI (`protocol_kit.py:228,258`) and re-exported (`gamma/__init__.py`), so it
  is reachable, just not exercised by any documented runtime path — confirm before deleting
  that nothing else constructs `DefaultGammaImportCalculator` directly.
- **`estimate_la_share`** (`melt/wealth_proxy.py:293`): explicitly marked
  `.. deprecated::` in its own docstring, emits a `DeprecationWarning` at call time
  (`wealth_proxy.py:320-326`), and its docstring names its own replacement
  (`check_equity_threshold` + `evaluate_class_shares`, the same Feature-043 functions
  identified as the wiring target in item 1 above). Confirmed still called internally at
  `wealth_proxy.py:486` (self-reference) — check external callers before removal.

### 6. GOTCHA — two diverged reference schemas

**Verified 2026-07-11:** `diff` between `/media/user/data/babylon-data/reference/schema.py`
(2,415 lines) and the canonical `src/babylon/reference/schema.py` (2,731 lines) in this
worktree produces a **343-line diff**. Confirmed differences include:
- an import-path difference (`from babylon_data.reference.database import NormalizedBase`
  vs. `from babylon.reference.database import NormalizedBase`) — these are genuinely
  different packages, not a copy-paste artifact,
- the canonical schema has classes the stale fork lacks entirely (e.g.
  `DimBEAEconomicArea`, BEA Economic Areas / 2004 redefinition — wall-to-wall functional
  regions, absent from the `babylon_data` fork).

**Implication for Phase-7:** any tooling that imports `babylon_data.reference.schema`
instead of `babylon.reference.schema` is working against a schema that is missing tables
the canonical target already has (and, structurally, may be missing whatever backs the
orphaned-data items above too, if new tables were added to the canonical schema without a
corresponding sync). Treat `src/babylon/reference/schema.py` as the only trustworthy
source of table definitions going forward; the `babylon_data` fork should be reconciled or
retired, not read from.

---

## Phase-7 Wiring-Order Recommendation

Ordered by dependency and by "cheapest verified win first":

1. **Variable-hours wiring (`fact_census_hours` + `_FredProductivityAdapter`).** The DI
   plumbing already exists end-to-end (Protocol, adapter, registration) — only a System-side
   *read* is missing. This is the lowest-effort wire: no new adapter, no new DI wiring, just
   a consumer. Do this first as the proof-of-pattern for closing the other two gaps.
2. **FRED wealth as calibration anchor for Feature-043.** Requires deciding the calibration
   mechanism (a defines-tuning script? a validation-only report? a `GameDefines` default
   derived once from FRED and then frozen?) before touching runtime code — this is a design
   decision, not a wiring bug, and should be scoped as its own small spec rather than bolted
   onto the variable-hours work.
3. **Resolve the `country.xlsx` "~140 vs ~253" discrepancy** before making the SKIP verdict
   final — a five-minute filter-definition exercise, but it should happen before Phase-7
   closes this item, since the current recommendation (SKIP) was reached independently of
   the exact count and shouldn't quietly rest on a wrong number in the historical record.
4. **Reconcile or delete the `babylon_data/reference/schema.py` fork.** Blocking risk: any
   future data-loading tool written against the stale fork will silently miss whatever
   tables were added to canon after the fork diverged. Cheap to fix (delete the fork, point
   remaining code at `babylon.reference.schema`), but do it before Phase-7 adds any new
   schema-touching tooling.
5. **Delete the gamma MVP dead seam + `estimate_la_share`.** Out of scope for *this*
   inventory's charter (orphaned *data*, not dead *code*), but flagged here because both
   were found in the same sweep and both block on nothing — safe to schedule as
   quick-follow cleanup once the D1/D2 wiring above lands and callers are confirmed clear.

---

## Touch Points Referenced

- `src/babylon/domain/economics/factory.py` — DI registration site (`_FredProductivityAdapter`
  at line 619, service-container entry at line 679).
- `src/babylon/domain/economics/substrate/transitions.py` — `evaluate_class_shares()` (line
  148) and `check_equity_threshold()` (line 195), the Feature-043 runtime mechanism.
- `src/babylon/domain/economics/working_day/data_sources.py` — `ProductivityDataSource`
  Protocol (line 13).
- `src/babylon/domain/economics/melt/gamma_hydration.py` — `SQLiteGammaHydrationSource`
  (line 132), the live Hickel-ERDI path.
- `src/babylon/domain/economics/melt/wealth_proxy.py` — deprecated `estimate_la_share`
  (line 293).
- `src/babylon/domain/economics/gamma/gamma_import.py` /
  `src/babylon/domain/economics/gamma/types.py` — the orphaned MVP seam
  (`DefaultGammaImportCalculator`, `MVP_ERDI_VALUES`, `MVP_IMPORT_SHARES`).
- `src/babylon/reference/schema.py` vs. `/media/user/data/babylon-data/reference/schema.py`
  — the diverged-schema gotcha.
