# Hex/Scale County-Keying + ScaleAdjunction Binding (Task #39) — Mini-Plan

> **For agentic workers:** execute via superpowers:subagent-driven-development, one implementer
> per task on branch `feature/queue-2026-07-19` in the `babylon-queue` worktree (owner directive
> 2026-07-19: one branch, one worktree, no per-track ceremony). Steps are condensed; each task's
> dispatch brief expands them with the TDD cycle.

**Goal:** Key USScenario territories by county (the res-3 H3 inversion bug), bind
`ScaleAdjunction` into the engine with real SubstrateSystem dynamics, add the CZ/MSA scale rungs
with real data, and ratify the corrected spatial ladder as a constitutional amendment.

**Owner rulings in force (2026-07-19):** W1 — USScenario ONLY; Wayne untouched (ticket #57).
W2 — constitutional amendment wanted; CZ data verified ABSENT from the drive (Chetty CZ-keyed
outcomes exist, no county→CZ crosswalk; `WID_data_CZ.csv` is Czechia) → acquire USDA ERS
crosswalk; MSA included (data already in reference DB: `dim_metro_area`/`bridge_county_metro`).
Queue-wide autonomy: self-merge green, ceremonies pre-authorized with drift tables.

**Investigation corrections adopted (2026-07-19 brief):** the 5 canonical qa:regression
baselines use opaque `T00N` territories and never touch USScenario → **expected ZERO baseline
drift**; the LODES-vs-H3 sovereignty finding is STALE (ADR080 fixed it) → excluded;
`Territory.id`'s pattern (`^(T[0-9]{3,}|[0-9a-f]{15})$`) forbids FIPS-in-id → county-keying =
**one territory per county, `county_fips` populated, id opaque** (the `WorldStateBridge`
pattern, `engine/headless_runner/bridge.py:789-815`); `resolve_county_identity`
(`domain/economics/tick/graph_bridge.py:41-74`) stays the sole county accessor.

## The lattice correction (⚠ owner must see this at ratification)

The recorded disposition ladder `county < CZ < [MSA] < state < national` cannot be a literal
nesting chain: **ERS commuting zones and OMB MSAs both cross state lines** (e.g. the NYC and
Memphis CZs/MSAs span multiple states). The amendment therefore codifies a **lattice**, not a
chain:

- `hex (res-7, immutable substrate) < county` — the base spatial atom chain.
- Above `county`, three PARALLEL county-aggregations, each its own `ScaleAdjunction`
  (allocate ⊣ aggregate): **CZ** (total partition — every county has exactly one CZ),
  **MSA** (partial — non-metro counties have none), **state** (total partition).
- Only `state < nation` nests further. CZ and MSA do NOT aggregate into state or into each
  other.

This is exactly what `ScaleAdjunction` already models (independent `mapping` dicts per rung) —
no new machinery, just honest geometry.

## Amendment text (draft — BD ratification required before the PR merges)

Replace Amendment K's spatial-lattice clause (`CONSTITUTION.md`, "Level lattices (spatial
hex < county < state < nation; …)") with:

> **Spatial scale lattice (revised by Amendment U):** the immutable substrate is the res-7 H3
> hex; the base administrative atom is the **county** (`county_fips`), the sole spatial key the
> economy reads (`resolve_county_identity`). Above the county sit three parallel materialist
> aggregations, each an allocate ⊣ aggregate adjunction: the **commuting zone** (USDA ERS 1990
> delineation — the daily reproduction-of-labor-power geography, from journey-to-work flows),
> the **metropolitan statistical area** (OMB delineation via `dim_metro_area` — the
> concentrated labor/housing-market geography; partial: non-metro counties carry no MSA), and
> the **state** (the juridical-repressive geography). Only the state aggregates further, into
> the **nation**. CZ and MSA cross state lines by construction and never nest into the state
> rung. Territory graph nodes are county-grain overlays on the substrate; hex is never a graph
> node (INV-010 discipline extends: the substrate is stateful only as `hex_`-prefixed territory
> attributes or service-held grids).
>
> Aleksandrov traces: hex→county = `bridge_county_h3`; county→CZ = ERS journey-to-work
> delineation (`bridge_county_cz`); county→MSA = `bridge_county_metro`; county→state = FIPS
> prefix; state→nation = constant.

Version bump `CONSTITUTION.md` 2.10.0 → 2.11.0; ADR (`ai/decisions/ADR0NN_scale_lattice.yaml` +
index) records W2, the lattice correction, and the data provenance.

## Tasks

### T1 — CZ crosswalk acquisition (data, in-repo tier)
Download the USDA ERS **1990 commuting-zone county crosswalk** (the vintage matching the
drive's Chetty CZ outcome tables) on the dev box; archive the raw file to
`/media/user/data/babylon-data/external/` (drive = raw upstream archive); derive
`src/babylon/data/reference/bridge_county_cz.csv` (columns: `county_fips` 5-char, `cz_id`,
`cz_name`; PK-sorted; every US county exactly once — loud validation). Register it the
established way: `data-catalog.yaml` categories source row (`USDA_ERS_CZ` already exists as a
source entry) + `data-artifacts.yaml` entry via `tools/make_data_artifacts.py` (mode=`register`,
csv, in-repo home — the `babylon_ricci_final` precedent). Tests: coverage (≥3000 counties, all
5-digit, unique), Michigan spot-checks. CI never touches the drive — the committed CSV is the
canonical artifact.

### T2 — Amendment + ADR (docs; merges only with owner ratification comment)
Apply the amendment text above to `CONSTITUTION.md` (+ version bump) and write the ADR.
Present the text to the owner (AskUserQuestion or PR comment) BEFORE the queue PR merges —
the ratification is the one non-autonomous gate in this plan.

### T3 — Lattice construction (`domain/dialectics/instances/levels.py`)
Extend `spatial_lattice_for_counties` (levels.py:223-263) to build the two new adjunctions from
real data: `cz_adjunction` (from `bridge_county_cz.csv`) and `msa_adjunction` (from
`bridge_county_metro` via `get_reference_session`; partial mapping — counties absent from the
bridge are simply absent from the mapping, and `allocate`/`aggregate` operate on the covered
subset; document that MSA sums are partial covers, never grossed up). Keep the existing
county→state→nation chain. Property tests: shares sum to 1 per parent; CZ mapping total over
the county universe; MSA mapping partial-but-consistent; determinism (sorted construction).

### T4 — USScenario county-keying (the core change)
`engine/scenarios/_legacy.py::_create_us_territories` (:680-712): replace
`h3.polygon_to_cells(polygon, 3)` hex minting with per-county territories from the reference DB
(`dim_county` via `get_reference_session`): id `T0001…` over FIPS-sorted counties (the
WorldStateBridge idiom), `county_fips` set, `h3_index=None`, real per-county seeds for the
fields the scenario currently derives per-hex (population/production anchors — pull from the
same reference tables the headless runner's per-county path uses; the implementer mirrors
`_build_per_county_territories`, never invents constants). Update dependent seeding in the same
scenario (org `territory_ids`, TENANCY/ADJACENCY edge construction) to the county grain.
ADJACENCY: derive from county adjacency if a reference source exists; if none exists, emit NO
adjacency edges and record the gap loudly in the scenario docstring + report (III.11 — no
fabricated adjacency). Grep-pass the 13 test files the investigation flagged for H3-format
assertions; fix fixtures to the county grain. Expected: qa:regression 5/5 UNCHANGED (verify);
web `"default"` sessions now create county territories.

### T5 — Balkanization seed at county grain (`web/game/engine_bridge.py::_seed_balkanization_layer`)
The h3-parent aggregation path (:8699-8712) only fires for territories with `h3_index` — county
territories skip it entirely today (silent influence-layer no-op for USScenario). Add the
county path: aggregate the res-7 influence seed to counties via `bridge_county_h3`
(`BridgeCountyH3`, reference DB) keyed on `county_fips`. Wayne's hex path stays untouched.
Test: county-keyed scenario gets a non-empty influence layer from the same seed fixture.

### T6 — SubstrateSystem real dynamics + engine binding of ScaleAdjunction
`engine/systems/substrate.py` (@2.5) stops querying the never-stamped `NodeType.HEX` and gains
real, data-grounded dynamics on county territories:
- Init (first tick a territory with `county_fips` is seen): seed `raw_material_stock` /
  `energy_stock` / `biocapacity_stock` from reference data (P22 minerals + energy tables —
  implementer scouts exact tables; loud NoDataSentinel-style skip with reason when a county has
  no row, never a fabricated default).
- Per-tick: depletion from the territory's production draw + regeneration, via NEW GameDefines
  coefficients (a `SubstrateDefines` category; regenerate defines.yaml) and the existing
  metabolic-rift formula family (`ΔB = R − E·η`).
- Binding: the system holds the T3 lattice and publishes aggregate substrate state at
  CZ/MSA/state/nation grain into `persistent_data["substrate.<rung>"]` (sorted, deterministic)
  — the first engine consumer of `ScaleAdjunction`.
- `UNSTAMPED_QUERY_ALLOWLIST` shrinks by `"hex"` if this removes the last community…hex query
  set member (check `Vol2CirculationStep`/`territory_diagnostics` — if their hex queries
  remain, the entry stays with corrected citation, the #40 lesson).
Canonical-5 baselines: their scenarios have no `county_fips` → system no-ops there with a
documented reason → still expect 5/5 byte-identical. If consumption wiring (Metabolism reading
the new stocks) moves ANY baseline, that lands as the pre-authorized declared ceremony with a
drift table.

### T7 — CZ becomes real in the web layer
`web/game/engine_bridge.py` `group_key_map` (:2719-2725) gains `"cz": "cz_id"` backed by a
county→CZ lookup (from the T1 CSV, cached); territory payloads at county grain carry `cz_id`.
The FramingSelector's CZ framing stops silently falling back to county. Test: `/map/?zoom=cz`
returns CZ-keyed aggregation.

### T8 — Sentinels for the discovered classes (mutation-validated, per standing rule)
- Vocabulary rule (f) — **wrong-rung keying**: AST check that every `Territory(id=…)`
  construction site passes either a `T`-prefixed opaque label or a value derived from an H3
  cell variable, and that no site passes a bare FIPS-shaped literal to `id=` or an H3 cell to
  `county_fips=` (the res-3 inversion class, both directions). Exemption governance as usual.
- Coverage `DataRequirement` rows per lattice rung (hex→county, county→CZ, county→MSA) naming
  the backing table/CSV — a rung whose concordance goes missing fails loud instead of
  silently degrading (the CZ-silent-fallback class).

### T9 — Close-out
Consolidated gates in `babylon-queue`: scoped suites per task along the way, then `mise run
check` + `mise run qa:regression` (5/5 expected; ceremony if T6 consumption moved values) +
`mise run check:sentinels`. Docs: scenario reference updates, `ai/state.yaml`, memory
(`hex-community-lawverian-disposition` gets the lattice correction). The queue PR carries the
amendment ratification thread.

## Sequencing & risk

T1 → T2 → T3 are independent of T4 (parallel-safe as separate sequential dispatches). T4 is the
blast-radius task — its implementer gets the investigation's consumer map verbatim. T5–T7
depend on T4. T8 lands last-but-one so its gates see the final shapes. Biggest honest risks:
(a) per-county seeding data for USScenario (T4) may surface reference-data gaps → loud skips,
never invented values; (b) ADJACENCY at county grain may lack a source → shipped absent + a
data-acquisition note, not fabricated; (c) T6 consumption wiring is the one place drift is
plausible → ceremony path stands ready.
