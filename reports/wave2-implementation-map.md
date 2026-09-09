# Wave 2 — Light the Economic Substrate: Implementation Map + Owner Rulings

**Date:** 2026-07-14 · **Branch:** `feature/epochs-wave1-spine` · **Source:** 4-agent read-only
recon (Gap-1 attrs / EconomyDashboard / map lenses / survival duel), owner rulings same evening.
Backlog parent: `reports/epochs-vision-gap-audit.md` § Wave 2.

## Owner rulings (2026-07-14, binding for this wave)

1. **`throughput_position` gets wired for real.** It is fabricated today — hardcoded `1.0` at
   seed (`domain/economics/tick/initializer.py:170-177`), carried forward forever because
   `_bridge_economics_overrides` (engine_bridge.py:4722) never constructs a
   `throughput_calculator`, and `_carry_tick_dynamics_flows` (:4803) never carries
   `tick_throughput_position`/`tick_supply_chain_depth` across the round-trip. Both gaps are
   engineering, not data (BEA/QCEW adapters read the same reference DB Φ already uses). Fix both,
   then ship the lens honestly. Shipping the lens without this = III.11 silent no-op that even
   the liveness probe cannot catch (1.0 probes as live).
2. **All 16 structurally-dead Gap-1 attrs register as `STRUCTURALLY_IMPOSSIBLE`** (Group C: 7
   circulation attrs gated on unwired `turnover_profile_source`, system/__init__.py:1050;
   Group D: 9 financial-distribution attrs gated on unwired `interest_calculator`, :1248), each
   row naming its exact unwired service. The observatory tells the truth; wiring those services
   is a named future item, not silent debt.
3. **Survival duel chart history = a real `class_snapshot` table**, mirroring the org/territory
   snapshot pattern (`persist_full_tick` + `query_*_snapshot_history` + `/history/` route), not
   client-side accumulation. Rupture markers come from `UPRISING` events filtered to
   `data.trigger == "revolutionary_pressure"` — the honest crossing signal (struggling roles
   only, agitation-gated; the raw crossing is NOT evented for other classes and must not be
   fabricated).
4. **County-zoom `imperial_rent` stays a SUM, documented as a ruling.** Φ is an extensive flow —
   the county total is the meaningful number, unlike the intensive rate lenses
   (pop-weighted mean) and categorical lenses (pop-weighted mode). The registry row's notes must
   record this so it reads as a decision, not an accident. All NEW numeric lenses use
   population-weighted mean; categorical use population-weighted mode with deterministic
   tie-break.

Delegated rulings (mine, recorded here): `agitation` registers as `DECLARED_CONDITIONAL`
("non-zero once IdeologySystem processes a falling-wage/rent/Φ/g33 crisis tick"; it is
legitimately 0.0 at tick 0 in every scenario — never fabricate warmth). `territory_type` ships
knowing only CORE/PERIPHERY appear in shipped scenarios; the Necropolitical Triad
(RESERVATION/PENAL_COLONY/CONCENTRATION_CAMP) renders only when a scenario seeds one — flagged as
content for the nationwide scenario (Amendment R execution, task #49). Do not conflate
`stub_bridge.py`'s legacy `"territory_type": "URBAN|SUBURBAN|PERIURBAN"` vocabulary with the real
`TerritoryType` enum.

## Recon deltas vs the audit (what changed)

- 6 of the 8 "phantom" economy fields are already live on this branch (spec-109 A4 + item 30);
  only `profit_rate`/`occ` remain hardcoded `None` in `get_economy_dashboard` (:2135-2136). Fix =
  a ~10-line `_mean_territory_attr(graph, key)` averaging non-null per-territory
  `tick_profit_rate`/`tick_occ` (write-site graph_bridge.py:122-123, year-boundary).
- `panels.economy` is not merely unread — it is **never fetched**: `setMounted(true)` has zero
  production call sites, so the per-tick fan-out (worldSlice.ts:32-51) skips it. Building the
  panel closes fetch + render at once.
- Of 26 unregistered Gap-1 `tick_*` attrs (all written by `write_tick_state_to_graph`,
  graph_bridge.py:102-195, year-boundary only): **Group A (5) genuinely live** —
  `tick_crisis_phase`/`tick_crisis_duration`/`tick_bifurcation_score`/`tick_wage_compression`
  (self-contained crisis detector) + `tick_capital_stock` (real when session has FIPS);
  **Group B (5) frozen constants** — `tick_throughput_position` (1.0) /
  `tick_supply_chain_depth` (2.0) / `tick_unemployment_rate` (0.05) / `tick_median_wage` (21.0) /
  `tick_class_distribution` (seed shares; transition_engine unwired); **Groups C+D (16) dead**
  (ruling 2). Group A registers TERRITORY/`DECLARED_CONDITIONAL` (year-boundary) and gets
  serialized in `_serialize_territory`; Group B likewise but with the frozen-constant caveat in
  notes (throughput leaves Group B via ruling 1).
- P(S|A)/P(S|R) (`SurvivalSystem.step` → `update_node(p_acquiescence=, p_revolution=)`,
  survival.py:143) reach no live payload: `_serialize_entity` (engine_bridge.py:5901) is dead
  code with zero call sites; `_social_class_inspector_fields` drops both. 2-line exposure +
  `InspectorNodeResponse` fields.
- Dead-code flags for later triage: `compute_disproportionality()` (circulation/reproduction.py:160,
  zero callers); `services.credit_cycle_detector` never read — graph hardcodes literal
  `"expansion"` (graph_bridge.py:67); orphaned `economic_summary` table (postgres_schema.py:707,
  writer+reader implemented, zero callers ever).

## Execution rounds (single-writer on engine_bridge.py — backend items sequence)

> **STATUS 2026-07-14: ALL THREE ROUNDS LANDED** on `feature/epochs-wave1-spine` —
> R1 `c090d48c`/`31f4b42b`, R2 `b92de750`/`d50f915d`, R3 `e284a65e`/`ef29cb4e`
> (backend/frontend per round; R3 frontend also rewired rupture markers to the
> uncapped server-filtered `ruptures` array — single fetch, defense-in-depth
> client re-filter). Gates at close: 909/909 vitest + tsc/eslint/prettier,
> scoped python suites green, seam sentinel exit 0, qa:regression 5/5
> byte-identical. Liveness partition honest at "1 of 7 live"
> (territory_type MUST_BE_LIVE added in R2).

- **Round 1** — ∥ two agents:
  - *Backend-1 (W2.1+W2.2b):* Gap-1 registration (24 rows: A 5 live + B 3 frozen + C 7 + D 9
    impossible; the 2 throughput attrs register in Round 2 with the real wiring), Group A+B
    emission in `_serialize_territory`, `profit_rate`/`occ` via `_mean_territory_attr`,
    `county_flow` noted for TS typing, imperial_rent SUM ruling into its registry notes.
    Follow-up noted: the dashboard's national `profit_rate`/`occ` is an unweighted mean of
    county rates — correct enough for one-county wayne, but the true national figure is
    Σs/Σ(K+v); revisit when the nationwide scenario (task #49) makes per-county s/K/v worth
    emitting.
  - *Frontend-1 (W2.2a):* EconomyDashboard panel component + mount (AppShell chrome line,
    FloatingPanel), `makeEconomyDashboardPayload` fixture, standard 4-test suite; wealth
    trajectory via existing `get_game_timeseries` `wealth` array (Recharts/TimeseriesChart
    template); crisis timeline from `crisis_phase_transition` journal events.
- **Round 2** — ∥ two agents:
  - *Backend-2 (W2.3b+W2.4):* throughput wiring (ruling 1) + three lens backends
    (`map_contract` entries, `_agitation_index_by_territory` pop-weighted via
    `_tenancy_members_by_territory`, `territory_type` pop-weighted-mode via the
    `dominant_class_pop` pattern, hex props + aggregation + `_hex_state_row` + inspector-hex
    parity + MAP registry rows). Aggregations per ruling 4.
  - *Frontend-2 (W2.3a):* lens registry entries + `TERRITORY_TYPE_COLOR/LABELS` palette +
    dedicated categorical lens kind + `HexMapFeatureProperties`/`LensTerritory` fields + both
    fill engines + legend + registry/mapLensLayers/regionFill tests + e2e lens-id assertions.
- **Round 3** — ∥ two agents:
  - *Backend-3 (W2.5b):* `class_snapshot` table + `_class_snapshot_rows` + `persist_full_tick`
    wiring + `query_class_snapshot_history` + `get_class_history` + `/node/:id/history/` route +
    p_acq/p_rev inspector exposure.
  - *Frontend-3 (W2.5a):* two-series `DuelSparkline` (extends bbl/Sparkline pattern), "Survival
    Calculus" InspectionSection on the class card, `inspectorNodeHistory` endpoints row + types,
    UPRISING/revolutionary_pressure crossing markers.

Every agent: Sonnet, TDD red-first, scoped test runs only, no commits; my review + sequential
commits between rounds; `mise run check:seams`/`check:coverage` exit 0 after every backend batch;
qa:regression must stay 5/5 (all work is bridge/presentation-side — a byte moving means STOP).
