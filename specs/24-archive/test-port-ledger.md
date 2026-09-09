# Program 24 — The Archive: test-port ledger

**Scope:** every behavioral assertion the legacy web client's test estate makes,
mapped to the contract test that carries it after the cutover. Seeded at P1
(county scope); **full closure is a P4 cutover-gate criterion** (charter
criterion 1). Format follows the Program 12 precedent
(`specs/112-cutover/test-port-ledger.md`).

**Authority:** `project/programs/24-the-archive.md` (P1 item 1, P4 gate).
**Method:** dispositions per row — PORTED (test moved, asserts the same
behavior), REWRITTEN (behavior re-asserted against the projection contract),
RE-GUARDED (behavior now pinned by a registry/view contract test), CARRIED
(deferred to a named P2 lane), RETIRED (legacy-only behavior, dies with the
web client).

## Disposition counts (P1 seed + Lane E through WO-42 + Wave-1 WO-35)

| Disposition | Rows |
|---|---|
| PORTED | 9 |
| REWRITTEN | 11 |
| RE-GUARDED | 2 |
| CARRIED (P2) | 1 |
| CARRIED (P3) | 1 |
| RETIRED | 1 |

## engine_bridge full-closure counts (WO-52 — all 63 disabled classes)

All 63 `tests/unit/web/test_engine_bridge.py` classes are now dispositioned:
17 already carried by the Ledger rows above (rows 35–47), 46 added in the
`engine_bridge disposition` section below. `GAP` is the honest disposition for
engine/projection behavior that no new test covers and no in-flight WO owns —
these are the cutover blockers enumerated in the LOUD list.

| Disposition | Classes |
|---|---|
| PORTED (already in rows 35–47) | 17 |
| PORTED (new) | 10 |
| RE-GUARDED (new) | 3 |
| CARRIED — P3 (new) | 8 |
| RETIRED (new) | 21 |
| REWRITTEN (new — T3 U2/U3/U4 closure) | 4 |
| **Total** | **63** |

## Ledger

| Old assertion (source) | Destination contract test | Disposition | Landed |
|---|---|---|---|
| `src/frontend/e2e/real-loop.spec.ts` (tick→page loop) | `tests/integration/archive/test_county_e2e.py` | REWRITTEN | WO-7 |
| `src/frontend/e2e/inspection-stack.spec.ts` (inspector reads) | `tests/unit/projection/test_county.py` (`project_county` contract) | REWRITTEN | WO-3 ✓ |
| `src/frontend/e2e/briefing-map-smoke.spec.ts` + `map-lens-cycling.spec.ts` | `tests/unit/projection/test_registry.py` (`v_county_value_aggregate` declared contract) | RE-GUARDED | WO-2 ✓ |
| `src/frontend/e2e/veil.spec.ts` | `tests/unit/projection/fog/` (veil gate carried with the fog relocate) | PORTED | WO-1 ✓ |
| `tests/unit/web/test_engine_bridge.py::TestGetEconomy` / `TestImperialRentGapByRegion` | `tests/unit/projection/test_county.py` (per-county Φ field: `tick_phi_hour` producer ruling) | REWRITTEN | WO-3 ✓ |
| `::TestSerializeTerritoryGraphThreading` / `TestStateToSnapshot` | `tests/unit/projection/test_view_models.py` (`CountyView` hydration) | REWRITTEN | WO-2 ✓ |
| `::TestHexStateProjection` | `tests/unit/projection/test_registry.py` (`v_hex_state_asof` declared contract) | RE-GUARDED | WO-2 ✓ |
| `::TestDeriveIntelLedger*` | `tests/unit/projection/fog/test_ledger.py` | PORTED | WO-1 ✓ |
| `::TestOrgCountByTerritory` / `TestMeanTerritoryAttr` / `TestHeatDeltaByTerritory` | `tests/unit/projection/test_county.py` (aggregation helpers contract) | REWRITTEN | WO-3 ✓ |
| `::TestEndgameDetection` (national axes) | national dossier lane — the axis is national-only, not county | CARRIED (P2 Lane P) | — |
| `::TestExpectedDeltas` (spec-116 preview == resolution) | `tests/unit/projection/verbs/test_preview.py` (`TestResolverParity` — doctrine threading pinned per-resolver; heuristic verbs pinned) | PORTED | WO-38 ✓ |
| `::get_verb_eligibility` plate assertions (eligibility predicates + (reason, remedy) copy + affordability ride-along) | `tests/unit/projection/verbs/test_plate.py` (`build_verb_plate` contract; copy table relocated to `projection/verbs/copy.py`, web shim `is`-single-sourced) | PORTED | WO-38 ✓ |
| `src/frontend/e2e/verb-submit.spec.ts` (plate render: 9 verbs, ineligible shows reason) | `tests/unit/projection/verbs/test_plate.py` + WO-26 verb-plate snapshot | REWRITTEN | WO-38 ✓ (widget golden lands WO-26) |
| `src/frontend/e2e/end-turn-flow.spec.ts` (submit → resolve) | `tests/integration/archive/test_verb_resolution.py` (submit → fold → OODASystem adjudicates; rejection never reaches the engine) | REWRITTEN | WO-39 ✓ |
| `::TestEngineBridgeActions` / `TestActionInjection` (submit affordability + injection shape) | `tests/unit/projection/verbs/test_submit.py` (attack mode-specific labor gates; JSON-null params coercion; org-absent fallthrough pinned) | PORTED | WO-39 ✓ |
| bridge endgame-recognition block (century horizon, UNRESOLVED fallback, lock window) | `tests/unit/projection/test_endgame.py` (`endgame_status` pure fold; detector stays engine-side) | PORTED | WO-39 ✓ |
| `::TestHexFeaturePropertiesVeilGate` / `TestDerivedEconomyVeilGate` + `test_veil.py` (doctrine veil tiers) | `tests/unit/projection/test_veil_gating.py` + `tests/unit/web/test_veil.py` (green through the `is`-identity shim over `babylon.projection.veil`) | PORTED | WO-41 ✓ |
| `test_vision_gate.py` (desert/mud/water class gate) | `tests/unit/projection/test_veil_gating.py::TestClassVisionPort` (pure `apply_class_vision`; bridge wrapper delegates, its 12 behavioral tests stay green) | PORTED | WO-41 ✓ |
| fog+class-vision two-gates hazard (`fog/filter.py` header, unresolved at P1) | `tests/unit/projection/test_veil_gating.py::TestGateComposition` (`apply_political_gates`: vision-then-fog, restriction-map composition — RESOLVED, not carried) | REWRITTEN | WO-41 ✓ |
| `tests/unit/web/test_narration_record.py` (durable narrator persistence; idempotent `update_or_create` re-generation; degraded record visible) | `tests/unit/projection/vault/test_narrator_cache.py` (vault-page durability survives a fresh cache instance; degraded entry retried and superseded in place; degraded page renders as `{absence}`) | REWRITTEN | WO-42 ✓ |
| `tests/unit/web/test_narrator.py::TestProviderSwap` + `NarrativeService` behaviors (non-blocking schedule; degraded-loud marker; III.6 model pinning) | `tests/unit/projection/vault/test_narrator_cache.py` (`TestSideProcess` fire-and-forget never raises; `TestDegradedLoud`; `TestModelPinSurvivesDeprecation` — deprecated pin's block byte-identical after a pin switch) | PORTED | WO-42 ✓ |
| `tests/unit/web/test_narrator.py` (DeterministicNarrator Wire-feed templates, euphemism sync, bespoke class/org templates) | `tests/unit/projection/vault/test_narrator_cache.py::TestNarratorOffFullyInformative` — the deterministic baked page IS the fallback surface (R4); the Wire-feed template estate itself dies with the web client | REWRITTEN | WO-42 ✓ (template specifics retire at P4) |
| `src/frontend/e2e/lobby-briefing.spec.ts` (codename regex, 5 pattern rows, win badge, "100 years" horizon copy — NOT in ledger before this WO) | `tests/unit/projection/test_briefing.py` + `tests/unit/projection/vault/test_render_briefing.py` (`project_briefing`/`render_briefing` contract: `operation_codename` + `journal_objectives` ports) | REWRITTEN | WO-35 ✓ |
| `src/frontend/e2e/lobby-briefing.spec.ts` (lobby row codename/tick/status metadata; archive → ABANDONED; arm-then-confirm delete) | campaign menu over the `babylon_meta` catalog | CARRIED (P3 WO-49) | — |
| `src/frontend/e2e/auth.spec.ts` (Django login) | none — Django auth dies with the web client | RETIRED | — |
| `src/frontend/e2e/first-session.spec.ts` (lobby→briefing→verb-grid→campaign-submit→resolve-tick legs; spec-116 acceptance gate 6 — absent from this ledger before the WO-52 sentinel build) | every engine/projection behavior this trunk walks already has a row/class above: lobby codenames = row 72 (CARRIED P3 WO-49); briefing content + `journal_objectives` axes = row 71 (REWRITTEN WO-35); verb-grid eligibility = `TestVerbEligibilityAgreesWithTargetsRealWayneCounty` (PORTED WO-38) + `TestDefixturedVerbTargets` (RETIRED); campaign preview/submit = rows 61–63 (REWRITTEN WO-38/39, PORTED WO-39); event-dedup rendering is web-UI-only (`src/frontend/src/lib/__tests__/eventDedup.test.ts`, outside this ledger's python/e2e scope) | RETIRED | — |
| `src/frontend/e2e/first-session.spec.ts` (epilogue leg, lines 397–482: rigged-horizon UNRESOLVED epilogue body + terminal-state immutability) | `tests/unit/projection/vault/test_epilogues.py` (its own docstring cites this spec file's lines 434–439 verbatim as the `UNRESOLVED` body's source) + `tests/unit/projection/test_endgame.py` (`endgame_status` pure fold) | PORTED | WO-34 ✓ |
| `src/frontend/e2e/event-popup.spec.ts` (toast rail: urgent-event popup, two-toast-lifetime contract, dismiss→"Missed" tray — absent from this ledger before the WO-52 sentinel build) | web-only React toast UI (`EventToasts.tsx`/`eventsSlice.ts`); classification/dedup already unit-tested in `src/frontend/src/lib/__tests__/eventClassifier.test.ts` + `eventDedup.test.ts` (frontend-only, outside this ledger's python/e2e scope); successor salience/autopause surfacing is the Chronicle (WO-27/48, already CARRIED via the `TestAlertsDashboard`/`TestSpineWhitelistSeverityAndTitles` rows above) | RETIRED | — |

## Deviations recorded

- **No forced bridge delegation in WO-3.** The work-order draft named
  `web/game/engine_bridge.py::get_county_import_exposure` as a thin-shim
  delegation site; inspection shows it serves spec-103 *import-exposure
  provenance* — a different quantity than the `CountyView` dossier, so a
  delegation there would be cosmetic, not a real seam. The genuine Django
  shim landed with the fog relocate (WO-1: `web/game/fog/*` re-export shims,
  `is`-identity guarded by `tests/unit/web/test_fog_shim.py`). Bridge
  *inspection* reads are REWRITTEN rows above; the bridge keeps serving the
  legacy client untouched until P4 deletes it.
- The disabled `test_engine_bridge.py` suite (module-level skip, owner ruling
  2026-07-20) remains in-tree as the P4 closure worklist: at cutover, every
  one of its 63 test classes must appear in this ledger with a disposition.
  **CLOSED by WO-52** — see the `engine_bridge disposition` section below.
- **Row-35 economy mapping is narrower than its wording (WO-52 finding).** Rows 35
  map `TestGetEconomy`/`TestImperialRentGapByRegion` onto `test_county.py`'s
  per-county `imperial_rent_phi` (`tick_phi_hour`). That is the Leontief-tensor Φ
  read straight off a territory attribute — a **different quantity** from the
  legacy classes' graph-wide `value_produced`/`rent_extracted`/`exploitation_rate`
  aggregation over `social_class` wealth + EXPLOITATION/WAGES edges and the
  population-weighted per-capita Wc−Vc breakdown. The per-county Φ reading is
  genuinely REWRITTEN; the **graph-aggregate economy quantities are NOT projected
  anywhere** and surface in the LOUD gap list below (`TestEconomyDashboardFundamentalTheorem`,
  `TestEconomyDashboardChipContract`). Existing rows are left intact per the
  no-delete rule; this note records the scope gap.
- **`first-session.spec.ts` and `event-popup.spec.ts` had no ledger row (WO-52
  closure-sentinel finding).** Building the mechanical sentinel test
  (`tests/unit/archive/test_ledger_closed.py`) enumerates every
  `src/frontend/e2e/*.spec.ts` filename and found these two absent from every
  prior WO-52 pass. Neither hid an unowned behavior: `first-session.spec.ts`'s
  legs are each already covered by rows/classes landed under other WOs (its
  epilogue leg is even cited BY NAME in `test_epilogues.py`'s own docstring,
  WO-34), and `event-popup.spec.ts` is purely legacy React toast-rail UI with
  its own frontend unit tests. Both appended as rows above rather than
  weakening the sentinel's containment check.

## engine_bridge disposition (WO-52)

Full closure of the 63 disabled `tests/unit/web/test_engine_bridge.py` classes.
Method: for each class, the **behavior** it pins (not its mock choreography) was
traced to a landed new test, an in-flight P3/P4 work order, a standing gate, or
judged legacy-web-only. Branch scanned: `feature/archive-p2-p4 @ 5474c44e`
(P2 Waves 1/1b/2 + Lane E WO-38…44 landed; P3 WO-46…50 + P4 WO-51…53 in flight).

### Already carried by the Ledger rows above (17)

| Class | Ledger row | Disposition |
|---|---|---|
| `TestGetEconomy` | 35 | REWRITTEN (WO-3) — per-county Φ only; see Deviations note |
| `TestImperialRentGapByRegion` | 35 | REWRITTEN (WO-3) — per-county Φ only; see Deviations note |
| `TestSerializeTerritoryGraphThreading` | 36 | REWRITTEN (WO-2) |
| `TestStateToSnapshot` | 36 | REWRITTEN (WO-2) |
| `TestHexStateProjection` | 37 | RE-GUARDED (WO-2) |
| `TestDeriveIntelLedger` | 38 | PORTED (WO-1) |
| `TestDeriveIntelLedgerWithoutDjangoDb` | 38 | PORTED (WO-1) |
| `TestOrgCountByTerritory` | 39 | REWRITTEN (WO-3) |
| `TestHeatDeltaByTerritory` | 39 | REWRITTEN (WO-3) |
| `TestMeanTerritoryAttr` | 39 | REWRITTEN (WO-3) |
| `TestEndgameDetection` | 40 | CARRIED (P2 Lane P) — recognition fold additionally PORTED via WO-39 `test_endgame.py` |
| `TestExpectedDeltas` | 41 | PORTED (WO-38) — preview parity; per-target rows die with the picker |
| `TestVerbEligibility` | 42 | PORTED (WO-38) `verbs/test_plate.py` |
| `TestEngineBridgeActions` | 45 | PORTED (WO-39) |
| `TestActionInjection` | 45 | PORTED (WO-39) |
| `TestHexFeaturePropertiesVeilGate` | 47 | PORTED (WO-41) |
| `TestDerivedEconomyVeilGate` | 47 | PORTED (WO-41) |

### New dispositions (46)

| Class | Behavior pinned | Destination / rationale | Disposition |
|---|---|---|---|
| `TestActionResultPersistence` | engine's per-action results (success/ci/details) written back after resolve | engine adjudication PORTED `tests/integration/archive/test_verb_resolution.py`; action_result round-trip in `projection/fog/test_investigate_wiring.py::TestStashAndDerive` | PORTED (WO-39/40) |
| `TestResolveTickConsumesEngineResults` | `turn_resolution` is authoritative for per-action success (never blind `success=True`); loud failure on missing result | engine-authoritative resolution PORTED `test_verb_resolution.py`; loud-failure/persist discipline end-to-end in the Pilot | PORTED (WO-39) → Pilot WO-50 |
| `TestInvestigateFieldSnapshot` | successful INVESTIGATE freezes TRUE post-tick field values; absence→None | `projection/fog/test_investigate_wiring.py::TestInvestigateFieldSnapshot` | PORTED (WO-40) |
| `TestResolveTickPersistsInvestigateSnapshot` | intel snapshot stashed onto the `action_result` row; bridge-compatible detail keys | `projection/fog/test_investigate_wiring.py::TestStashAndDerive` | PORTED (WO-40) |
| `TestResolveTickNarrativeServiceHook` | narrative generation scheduled fire-and-forget, never blocks the tick | `projection/vault/test_narrator_cache.py::TestSideProcess` | PORTED (WO-42) |
| `TestWireFeed` | class/org-scoped events narrate the scenario's REAL names, never the canonical map | actor-naming grounding PORTED `tui/test_chronicle.py::TestActorResolution` (`resolve_actor`); WireFeed envelope + DeterministicNarrator retire (Chronicle plate WO-27 + narrator cache WO-42) | PORTED (WO-27) |
| `TestEducateTargetsResolveViaTenancy` | social_class targets resolve via TENANCY edges, never a `territory_ids` field | `verbs/plate.py::_tenancy_members_by_territory`, `verbs/test_plate.py`/`test_preview.py` fixtures rely on TENANCY only | PORTED (WO-38) |
| `TestVerbEligibilityAgreesWithTargetsRealWayneCounty` | eligibility agrees verb-by-verb with the target lists on the real graph | `build_verb_plate` unifies eligibility with the target-existence predicates (one source of truth → the disagreement bug class is structurally eliminated), `verbs/test_plate.py` | PORTED (WO-38) |
| `TestBuildTickSummarySeriesAggregates` | county-deduped, population-weighted tick_* aggregates (no N-fold inflation, extensive-sum-once-per-county) | `projection/national.py` pop-weighted county rollup + `test_national.py::test_unattributed_territory_does_not_skew_the_rollup` (same intensive-aggregation-variance discipline) | PORTED (WO-17) |
| `TestEconomyDashboardVeil` | value-axis fields masked below doctrine Veil Tier 1, real at Tier 1+ | veil mechanism PORTED `projection/veil.py` + `test_veil_gating.py`; **no economy payload consumes it yet** (tied to the economy-dashboard GAP below) | PORTED (WO-41) |
| `TestDefixturedQueryCorrectness` | queries read correct node TYPES/attrs/enums (social_class≠community; no `territory_ids` on social_class; real declared fields) | `mise run check:vocabulary` (3-rule vocabulary sentinel) is the exact standing guard for this bug class; extraction_intensity/edge_mode economy reads fold into the economy GAP | RE-GUARDED |
| `TestCausalHeartbeatWiring` | per-session observer emits a TICK_PULSE frame each tick | frame emission RE-GUARDED `tests/unit/engine/observers/test_causal_chain.py::TestFrameCaptureApi`; per-session cache + NarrationRecord persistence = web Voice-heartbeat → Archive analogue is the WO-42 narrator cache | RE-GUARDED |
| `TestCausalHeartbeatPersistence` | pulse beat persisted as a deterministic NarrationRecord | same — observer RE-GUARDED `test_causal_chain.py`; the "week's ledger" beat is web-narration → WO-42 | RE-GUARDED |
| `TestEngineBridgeCreateGame` | create_game mints a session, seeds tick-0 state, rejects unknown scenario before creating a session | campaign creation is the campaign menu (session mint + reuse `GameSession` path); tick-0 seed is WO-44 | CARRIED (P3 WO-49) |
| `TestPatternShiftSeverity` | `pattern_shift` classified `warning`; severity keys are plain strings | severity classifier explicitly deferred (chronicle.py docstring) | CARRIED (P3 WO-48) |
| `TestTickEventPersistence` | tick_event rows built with event_type/severity/source_id; skip when empty | severity-tagged row construction = WO-48; the `persist_tick_events` DB primitive is RE-GUARDED `tests/unit/persistence/test_postgres_runtime.py::TestPersistTickEvents` | CARRIED (P3 WO-48) |
| `TestJournalDashboard` | persisted tick_event history read back (type/severity/tick/body/data), degrade-to-empty | the journal IS the Chronicle browsable stream (WO-27) + severity (WO-48) | CARRIED (P3 WO-48) |
| `TestAlertsDashboard` | latest tick's events filtered to critical/warning | chronicle salience/autopause | CARRIED (P3 WO-48) |
| `TestSpineWhitelistSeverityAndTitles` | severity tiers + humanized titles for 14 event types | WO-48 owns severity classification + the loud-unclassified fix | CARRIED (P3 WO-48) |
| `TestReactionaryVerbSeverityAndAnchoring` | pogrom/lockout/vigilantism `warning` + anchored to target's territory | severity → WO-48; event→territory anchoring is chronicle enrichment (TENANCY-inversion primitive alive in `plate.py`) — borderline, see LOUD | CARRIED (P3 WO-48) |
| `TestSerializeEventUprisingTerritoryAnchoring` | UPRISING `data.node_id`→territory via TENANCY inversion; honest None absent | chronicle event enrichment; primitive alive (`plate._tenancy_members_by_territory`) — WO-48 scope doesn't explicitly name it (borderline, see LOUD) | CARRIED (P3 WO-48) |
| `TestScenarioBootstrap` | `resolve_scenario` aliases (default→us, detroit→wayne_county); every catalog key seedable | web-API alias table (`game.api`); durable scenario-builds-tick-0 is engine-owned (`tests/unit/engine/scenarios/test_scenario_registry.py`, `test_scenario_initialization.py`); Archive selection is WO-49 | RETIRED |
| `TestEngineBridgeHydrate` | hydrate_state loads graph + reconstructs WorldState; bootstraps when unseeded | web-bridge wrapper over `persistence.hydrate_graph`; `WorldState.from_graph` engine-tested; Archive reads via the headless runner | RETIRED |
| `TestEngineBridgeResolveTick` | resolve_tick hydrate→step→persist order; events JSON-safe | web tick orchestration; the tick loop is the headless runner (`tests/unit/engine/headless_runner/`); event serialization is chronicle's | RETIRED |
| `TestEngineBridgeSnapshot` | Spec-052 snapshot envelope (per-type lists, no top-level entities/economy, derived block) | web `GameSnapshotSerializer` shape; replaced by per-kind `ProjectionRecord` view-models (the `observe()` seam); derived.economy veil-gating separately PORTED (row 47) | RETIRED |
| `TestSessionScopedDefines` | resolve_tick reads defines from the session row, not the global blob | web per-session `game_defines_json` plumbing; durable resolution is runner-owned (`tests/unit/engine/headless_runner/test_defines_resolution.py`); per-campaign runner invocation inherently avoids cross-session leakage | RETIRED |
| `TestDefixturedVerbTargets` | 5 verb-target LIST methods: real data, iterate all territories, TENANCY, dedup, no fixture literals, warsaw_ghetto from real p_acquiescence | web target-picker lists — the TUI targets via page-navigation + verb plate (R4/S6). TENANCY PORTED in `plate`; node-shape RE-GUARDED by `check:vocabulary`; p_acquiescence/consciousness on `SocialClassView` (WO-23) | RETIRED |
| `TestInvestigateTargetsDemocked` | get_investigate_targets: observe_capability/targeted_scans/counter_intelligence de-mock | web investigate target-picker; INVESTIGATE's durable intel wiring PORTED WO-40; observe_capability/counter_intelligence rich payload has no TUI analogue | RETIRED |
| `TestMobilizeTargetsIncludeSeededBusinesses` | QCEW-seeded Business NPCs are MOBILIZE targets with real name/type | web mobilize target-picker; business seeding engine-tested (`create_us_scenario`/`build_seeded_businesses`, `test_us_scenario_county_grain.py`); mobilize eligibility over business/civil_society PORTED in `plate.py` | RETIRED |
| `TestClassSnapshotRows` | project class dicts onto class_snapshot columns (survival calculus fields) | web class_snapshot persistence-table projection; the survival-calculus fields are projected onto `SocialClassView` (WO-23, `test_social_class.py`) | RETIRED |
| `TestGetClassHistory` | class_snapshot history sparkline + UPRISING/rupture markers | web class-history dashboard; the Archive's per-tick history is the baked vault page's git log (WO-44/51); ruptures via the Chronicle (WO-27/48) | RETIRED |
| `TestGetEdgeHistory` | edge-weight sparkline (weight←value_flow, honest null, Decimal→float, edge_id parse) | web edge-history dashboard; the query is RE-GUARDED `test_postgres_runtime.py::TestQueryEdgeSnapshotHistory`; edges surface on entity pages | RETIRED |
| `TestGetFieldStateStubParity` | StubEngineBridge returns a well-formed empty field-state payload | web-transport stub; no TUI HTTP layer to stub | RETIRED |
| `TestGetFieldStateAPIView` | Django `field_state/` view returns the standard envelope | web Django REST envelope; no TUI HTTP layer | RETIRED |
| `TestGetMapHistory` | map-lens scrubber: metric taxonomy (unknown/not_replayable), window cap, veil-0 masking | web map-lens time-scrubber; per-tick history is the baked vault git log (WO-44/51); map room WO-33 renders current-tick | RETIRED |
| `TestHexFeaturePropertiesHabitability` | per-hex habitability read from an `attributes` JSONB blob, honest None | web per-hex map-lens; habitability stays a live engine attr (MetabolismSystem, read by endgame/balkanization); `v_hex_state_asof` (WO-33) has a fixed column set (no attributes blob) | RETIRED |
| `TestHexStateRowStateFips` | state_fips = county_fips[:2], absent when no county_fips | web `_hex_state_row` helper; the derivation lives in persistence (`hex_hydrator.py`, `territory_diagnostics.py`) feeding `v_hex_state_asof` | RETIRED |
| `TestPersistSnapshotsGraphWiring` | `_persist_snapshots_safe` threads graph= so territory rate columns aren't NULL | web territory_snapshot persistence plumbing; the tick_* rate reads are PORTED into `CountyView` (WO-2/3, `test_county.py`) which the Archive tick-baker bakes | RETIRED |
| `TestBuildTickSummaryMarketAxis` | state.market → tick_summary (price_log/fictitious_log/market_corrections) | web tick_summary columns; the price⟷value scissors axis surfaces per-territory as `price_divergence` in `CountyView` (row 36) and is veil-gated in `projection/veil.py` (WO-41) | RETIRED |
| `TestBridgeEconomicsOverridesWiresCirculationAndFinancialServices` | `_bridge_economics_overrides` wires FRED circulation/financial services | web-bridge-local DUPLICATE of the headless-runner wiring (`domain/economics/factory.py`); Archive runs the engine via the runner; durable behavior covered by `tests/unit/economics/test_create_financial_services.py` + `tests/integration/test_circulation_one_tick.py` | RETIRED |
| `TestBridgeEconomicsOverridesWiresVol1ReserveArmyServices` | `_bridge_economics_overrides` wires Vol I reserve-army services | same — web duplicate of the runner's `create_vol1_services`; durable behavior covered by `tests/integration/test_volume_i_integration.py` | RETIRED |
| `TestGroupCDDocstringsHonest` | web/game/engine_bridge.py docstrings say "CORRECTED 2026-07-18", not "both gating services are unwired" | docstring-accuracy meta-test on web-bridge source that dies at cutover; pins no runtime behavior | RETIRED |
| `TestEconomyDashboardFundamentalTheorem` | graph-wide Wc−Vc imperial-rent gap + per-region population-weighted per-capita breakdown | the T3 spine-C economy dossier reads the SAME verdict the engine already adjudicates (`opposition_states["wage"].balance`, never a parallel Φ) + per-class Φ readings off the `fundamental_theorem` graph stash — `babylon.projection.economy.project_economy`, `tests/unit/projection/test_economy.py::TestEconomyDashboardFundamentalTheorem` | REWRITTEN (T3 U2) |
| `TestEconomyDashboardChipContract` | economy dashboard emits an exact key set of aggregate quantities | the chip key-SET itself was web-shape and retires; the underlying quantities (Volume III surplus split s=p+i+r+t + the metabolic matter-book) are now projected, extensive RATIO-OF-SUMS — `babylon.projection.economy.project_economy`, `tests/unit/projection/test_economy.py::TestEconomyDashboardChipContract` | REWRITTEN (T3 U2) |
| `TestGetFieldState` | dialectical field-stack projection: contradiction_fields + field_derivatives (laplacian/df_dt) honest-omitted, id-sorted, TENANCY-anchored edges, principal_field/dialectical_regime | the T3 U3 field-state dossier ports `EngineBridge.get_field_state`'s exact read logic — `babylon.projection.field_state.project_field_state`, `tests/unit/projection/test_field_state.py::TestGetFieldStateNodes`/`TestGetFieldStateEdges`/`TestGetFieldStatePrincipalFieldAndRegime` | REWRITTEN (T3 U3) |
| `TestBalkanizationMapFields` | balkanization block: faction enumeration + per-territory contested/dominant_faction from INFLUENCES reads | the T3 U4 balkanization dossier adds `FactionView` + `territory_influence` (outgoing INFLUENCES edges, edge weight/channel + target `county_fips`) mirroring `project_sovereign`'s recipe — `babylon.projection.faction.project_faction`, `tests/unit/projection/test_faction.py`; no engine scenario seeds `NodeType.FACTION` today (only the legacy web bridge's `_seed_balkanization_layer`) — the honest all-absent dossier is correct on a real campaign; porting the seed is deferred to the RED_OGV repair program (`ai/wiring-doctrine.md` gap ledger, OPEN row) | REWRITTEN (T3 U4) |

### LOUD — coverage gaps the main loop must close before cutover

Engine/projection behavior that (a) no landed test covers, (b) no in-flight WO
clearly owns. These block the WO-52 cutover gate:

1. **CLOSED (T3 U2).** ~~`TestEconomyDashboardFundamentalTheorem` → graph-wide
   Wc−Vc imperial-rent gap + per-region population-weighted per-capita
   breakdown.~~ Closed by `babylon.projection.economy.project_economy` — the
   verdict reads `opposition_states["wage"].balance` verbatim (never a
   parallel Φ) plus the `fundamental_theorem` graph stash's per-class Φ
   readings. Row 35's `tick_phi_hour` remains a distinct quantity (per-county
   Leontief Φ, not the Fundamental Theorem) — the reconciling note that scope
   boundary still stands. See row 191 above.
2. **CLOSED (T3 U2).** ~~`TestEconomyDashboardChipContract` → economy
   aggregate quantities.~~ The chip key-SET contract itself was web-shape and
   retires with the web client; the underlying aggregates are now the
   economy dossier's Volume III surplus split (`s = p + i + r + t`, extensive
   ratio-of-sums) + metabolic matter-book (`overshoot_ratio`,
   `biocapacity_ceiling`) — `wealth_by_class_role`/`county_flow` specifically
   have no successor (no `FactionView`/per-role rollup exists; not this
   unit's scope). See row 192 above.
3. **CLOSED (T3 U3).** ~~`TestGetFieldState` → dialectical field-stack /
   "Weather Layer" projection.~~ Closed by
   `babylon.projection.field_state.project_field_state` — a direct port of
   `EngineBridge.get_field_state`'s read logic (`contradiction_fields`,
   `field_derivatives` laplacian/df_dt only, `fascist_alignment`,
   TENANCY-anchored `field_gradients`, graph-level `principal_field`/
   `dialectical_regime`) into a pure projection read-model, singleton page
   `field_state/USA.md` (`ArchiveTickBaker`/`IncrementalArchiveTickBaker`
   dispatch). See row 194 above.
4. **CLOSED (T3 U4).** ~~`TestBalkanizationMapFields` → faction enumeration +
   contested-territory derivation.~~ Closed by
   `babylon.projection.faction.project_faction` + `render_faction.py` —
   `territory_influence` derives from outgoing INFLUENCES edges (mirroring
   `project_sovereign._claimed_county_fips`'s query pattern), sorted
   influence-level-descending/territory-id-ascending to match
   `GraphProtocol.query_faction_influence_by_territory`'s own ordering.
   Honest-empty verified: no engine scenario seeds `NodeType.FACTION` (only
   the legacy web bridge's `_seed_balkanization_layer` does), so a real
   headless campaign's graph has zero faction nodes and `project_faction`
   correctly returns a valid all-absent dossier for it — porting the seed
   into engine scenarios is a physics change EXPLICITLY DEFERRED to the
   RED_OGV repair program, tracked as an OPEN row in `ai/wiring-doctrine.md`'s
   gap ledger (ADR125). See row 195 above.

Secondary / borderline (surface for a ruling, not hard cutover blockers):
- **Event→territory anchoring** (`TestSerializeEventUprisingTerritoryAnchoring`,
  anchoring half of `TestReactionaryVerbSeverityAndAnchoring`) — CARRIED to WO-48,
  but WO-48's charter (salience/dedup/autopause) does not explicitly name event
  enrichment; confirm WO-48 absorbs it or open a sub-task. The TENANCY-inversion
  primitive is already alive in `verbs/plate.py`.
- **Per-hex `habitability`** (`TestHexFeaturePropertiesHabitability`) — RETIRED as a
  web map-lens, but the ecological-overshoot signal is not surfaced in any TUI hex
  projection (`v_hex_state_asof` has no habitability column). Confirm this is an
  intentional map-lens omission, not a dropped signal.

---

## WO-50 — Pilot e2e (first-session trunk spine → the Archive seams)

The cutover gate #2 acceptance evidence: the legacy web trunk e2e
`src/frontend/e2e/first-session.spec.ts` (483 lines) ported behavior-not-selectors
onto the Archive stack (engine + projection + TUI pure logic; the web client is
never imported). One test per hard-asserted sub-behavior in
`tests/integration/archive/test_pilot_first_action.py`.

| first-session.spec.ts leg (lines) | Archive test | Disposition |
|---|---|---|
| lobby shows generated codenames, no unnamed rows (100–116) | `test_lobby_every_catalog_row_carries_a_derived_codename` | REWRITTEN (real `BabylonMetaStore`/`CampaignMenu` over `pg_pool`; row codename == `operation_codename(campaign_id)`) |
| briefing: five patterns, win condition, fixed century horizon (118–158) | `test_briefing_five_patterns_win_condition_and_century_horizon` | REWRITTEN (`project_briefing`/`render_briefing`; "100 years" + "Century" copy; honest 0.0 progress on a fresh campaign) |
| Campaign: preview (prob + cost) before submit, submit succeeds (223–259) | `test_verb_preview_precedes_submit_then_the_engine_adjudicates` | PORTED (from `test_verb_resolution.py`; preview_verb → submit_verb → OODASystem adjudicates; EDUCATE is the affordable eligible verb in the OODA-minimal fixture the web's Campaign maps to) |
| (verb write-side rejection half of the plate contract) | `test_unaffordable_verb_is_refused_before_the_engine` | PORTED (from `test_verb_resolution.py`) |
| forced first crisis autopauses + acknowledged live (261–304) | `test_forced_endgame_crisis_autopauses_amber_then_ack_clears` | REWRITTEN (`classify_event_salience`/`compute_autopause_state`/`render_autopause_indicator` + pilot-loop Step gate + ack-clears) |
| no two consecutive identical event cards; volume floors (319–350) | `test_event_dedup_and_volume_floors_over_real_tick_events` | REWRITTEN (real engine tick → `dedupe_consecutive`/`apply_volume_floors`; test-local engine→ChronicleEvent adapter — see gap 1) |
| endgame_progress axes render honestly, none pinned 1.00 (352–362) | `test_objective_progress_after_two_real_ticks_never_pinned` | REWRITTEN (two real ticks → real `EndgameDetector.axis_progress()` → `journal_objectives`; none == 1.0) |
| epilogue: real horizon termination → UNRESOLVED, exact copy (365–439) | `test_rigged_horizon_crosses_into_the_unresolved_epilogue` | REWRITTEN (rigged `campaign_horizon_years=1`/`weeks_per_year=1`; one real tick; `endgame_status` crosses; body byte-equal to `EPILOGUES["unresolved"]`) |
| terminal-state: further resolve succeeds, persisted epilogue immutable (441–482) | `test_terminal_epilogue_is_stable_across_a_further_tick` | REWRITTEN (further real tick resolves — no engine refusal gate; outcome/epilogue stable; see gap 4) |

### WO-50 honest gaps (load-bearing — surfaced to the BD, never faked)

1. **No shipped engine→Chronicle feed.** `babylon.tui.chronicle` is fixture-fed by
   construction; the dedup leg reshapes REAL bus events into `ChronicleEvent` via a
   test-local adapter (`_chronicle_events_from_bus`). Events real, dedup/floor logic
   real; the production adapter is a **future WO**.
2. **Narrative-cap floor vacuous in-process.** The minimal in-process scenario emits
   only warning-tier events (`lifecycle_transition` + `organizational_action`), so the
   informational-tier per-tick cap holds but caps nothing. The ORGANIZATIONAL_ACTION
   aggregation floor IS exercised live. A future WO wiring the headless wayne run into
   the Chronicle feed exercises the informational cap against real informational events.
3. **`endgame_reached` is critical but NOT the sole critical tier.** The web re-tiered
   critical to endgame-only (FR-116-2); the ported `EVENT_SEVERITY` keeps the full
   spec-061 taxonomy (14 critical types). endgame_reached is critical + drives autopause,
   but not uniquely so in the Archive — a divergence to rule on.
4. **Pure endgame fold is memoryless.** `endgame_status` recomputes each tick; the web's
   "first ENDGAME_REACHED row is authoritative" immutability (`ORDER BY tick ASC LIMIT 1`)
   is a persistence guarantee absent from the pure projection. The terminal-state leg's
   stability holds because the material state never crosses into a pattern; a future WO
   must persist the first endgame event to lock the outcome against a LATER pattern.
5. **No autopause-acknowledgement state machine.** `chronicle_salience` omits the
   once-per-key ack layer (WO-46 `babylon_meta`); the crisis leg models ack as the pilot
   dropping the acknowledged critical event from the surfaced slice.
