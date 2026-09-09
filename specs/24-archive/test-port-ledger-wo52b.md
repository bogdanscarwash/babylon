# WO-52b — spec-061 live test-estate ledger extension

**Scope:** WO-52 (`specs/24-archive/test-port-ledger.md`) closed the *disabled*
`tests/unit/web/test_engine_bridge.py` suite (63 classes) and the `src/frontend/e2e/*`
specs. It did **not** cover a third, still-live category: the ~19 test files under
`tests/integration/` (root + `web/`) and `tests/unit/persistence/` whose docstrings
carry the original `spec-061` ("Real Backend Wire-Up") task/FR tags — these files
still run and gate the web CI leg today. Per the v1.0.0 master plan
(`ai/_inbox/PROGRAM_v1_0_0_playable_archive.md`, decision #10): *"Port ~11 spec-061
tests to projection/engine contracts + retarget 2 probes; observatory stays as
local diagnostics until Grafana metropole."* This ledger closes that gap —
"observatory" there names `web/observatory` (spec-096/099), which this WO does
**not** touch (untouched: `web/observatory/`, `tests/unit/observatory/`,
`tests/integration/observatory/`).

**Authority:** `ai/_inbox/PROGRAM_v1_0_0_playable_archive.md` decision #10; T1.2 keel
scope (`CLAUDE.md` "Architecture spine" train table). Format follows the WO-52
precedent (`specs/24-archive/test-port-ledger.md`).

**Method:** every live `spec-061`-tagged test file read in full; for each, checked
whether the modern `babylon.projection`/`babylon.persistence`/`babylon.tui` layer
already pins the same behavior (cite, no new code — **RE-GUARDED**), whether the
behavior is pure engine/persistence logic with zero web dependency (relocate +
reframe — **PORTED**), whether it needs a genuinely new pin at the modern layer
(write it — **PORTED**), whether it is web-transport-only with no Archive/TUI
analogue (**STAYS-WEB** — still gates the web leg, deliberately out of porting
scope, not retired), or whether the underlying game feature the test probes
doesn't exist yet at the projection layer (**CARRIED**, cites the owning future
WO). **Nothing was deleted** — every original file is untouched and keeps running.

## Disposition counts

| Disposition | Files | Meaning |
|---|---|---|
| PORTED | 5 | New/relocated test landed at the projection/engine/persistence layer |
| RE-GUARDED | 4 | Same behavior already pinned elsewhere; citation only, no new code |
| CARRIED | 3 | Real gap; deferred to a named future WO (no fabricated coverage) |
| RETARGETED | 1 (2 classes) | Existing probe re-scoped in place (SC-003 + SC-006) |
| STAYS-WEB | 6 | Django/HTTP-transport-only; no Archive/TUI analogue exists or is needed |
| **Total** | **19** | |

## Ledger

| Spec-061 file | Class(es) | Disposition | Destination / rationale |
|---|---|---|---|
| `tests/integration/test_tick_immutability.py` | `TestTickImmutability` | **PORTED** | `tests/integration/archive/test_session_persistence_contracts.py::TestTickImmutability` — zero web import in the original (100% `PostgresRuntime`); relocated + reframed, same assertions, dropped `spec-061` scenario tags |
| `tests/integration/test_persist_tick_atomic.py` | `TestPersistFullTickAtomic` | **PORTED** | `tests/integration/archive/test_session_persistence_contracts.py::TestPersistFullTickAtomic` — same reasoning; zero web import in the original |
| `tests/integration/test_multi_session_distinct.py` | `TestTwoSessionsIsolated` | **PORTED** | `tests/integration/archive/test_session_persistence_contracts.py::TestTwoSessionsIsolated` — 3 of 4 sub-tests were already pure `PostgresRuntime`; the 4th (`test_distinct_rng_seeds_persisted`) read the seed back through the web-private `web.game.engine_bridge._fetch_session_rng_seed_from_pool` helper, ported to `PostgresRuntime.get_session(session_id)["rng_seed"]` — the runtime's own public API, and the one the Archive headless runner (`engine/headless_runner/runner.py`) actually constructs |
| `tests/integration/test_event_serialization.py` | `TestSeveritySchema` | **PORTED** | `tests/unit/tui/test_chronicle_salience.py::TestPortedPerTypeSeverityPins` — the 15 named event-type→tier examples (6 critical / 6 warning / 3 informational) ported onto `classify_event_salience`; the tier-count aggregate (14/20/13) was already pinned by `test_ported_tier_counts_match_the_legacy_bridge` (prior WO-48 work) but a count-only check cannot catch two types silently swapping tiers, so the named pins add real regression coverage the count check does not. **Deliberately NOT ported:** the legacy `test_unknown_events_default_to_informational` assertion — `chronicle_salience.py` intentionally changed this default to `"warning"` + `unclassified=True` (Constitution III.11, documented in the module's own docstring and already pinned by `TestUnclassifiedSurfacesLoud`); porting the old assertion verbatim would contradict a deliberate, already-ratified improvement, not a gap. `TestSerializedEventShape` (id/severity/title/body envelope) and `_humanize_event_type` stay **STAYS-WEB** — they pin the web JSON envelope shape, which the TUI (no HTTP layer) never produces; the chronicle's own `ChronicleEvent`/rendering shape is separately contract-tested in `tests/unit/tui/test_chronicle.py` |
| `tests/integration/test_action_determinism.py` | `TestSimulationConfigAcceptsRngSeed` | **RE-GUARDED** | Byte-identical coverage already exists: `tests/unit/models/test_config.py::test_default_rng_seed` / `test_custom_rng_seed`. No new code — this class was already fully redundant before this WO |
| ″ | `TestFetchSessionRngSeedFromPool`, `TestResolveTickThreadsRngSeed` | **STAYS-WEB** + genuine finding | These pin that the *web bridge* threads `session.rng_seed` into `SimulationConfig(rng_seed=N)` (spec-061 FR-024) — real behavior, but web-bridge-private (`_fetch_session_rng_seed_from_pool`). **Finding surfaced, not fixed here (out of this WO's surgical scope):** the Archive's own headless runner does **not** do the analogous threading — `engine/headless_runner/runner.py`'s `ServiceContainer.create(defines=defines, **economics_overrides)` call (line ~1267) never passes `config=SimulationConfig(rng_seed=config.random_seed)`, so every headless run's `SimulationEngine` sees the hardcoded `SimulationConfig()` default (`rng_seed=0`), regardless of `SimulationRunConfig.random_seed`. This is architecturally different from "missing" in one respect — `Organization.rng_seed` (per-entity, `models/entities/organization.py`) and `ConservationAuditor(rng_seed=config.random_seed)` (hash-stamping, `persistence/conservation_audit.py`) already consume the run seed for their own purposes — but `SimulationConfig.rng_seed` itself is genuinely dormant on the headless path. Recorded here for the T1.2 "assumptions-ledger surface" deliverable rather than silently patched; a future WO threading `config.random_seed` into `ServiceContainer.create` (or proving no System actually reads `SimulationConfig.rng_seed` today, making the dormancy moot) should close it |
| `tests/integration/test_territory_edge_serialization.py` | `TestTerritorySerializationFR013` | **RE-GUARDED** | The honest-`None`-when-unattributed discipline for `consciousness` is already pinned at the modern layer by `tests/unit/projection/test_county.py::test_unattributed_county_has_no_consciousness` — `CountyView.consciousness` is a *better-grounded* quantity (population-weighted aggregate over real `SocialClass` ideology) than the old bridge's raw `Territory.consciousness` attribute (which no system ever wrote), so the old test's "honest None" spirit is preserved and improved, not just duplicated. `solidarity`/`dominant_community`/territory-level `wealth` passthrough have no `CountyView` field yet — **CARRIED** to the future economy-dossier WO (T3 in the master plan), which already owns the closely-related LOUD gaps `TestEconomyDashboardFundamentalTheorem`/`TestEconomyDashboardChipContract` in the WO-52 ledger |
| ″ | `TestEdgeSerializationFR014` | **CARRIED** | No `EdgeView`/edge dossier exists at the projection layer — per the WO-52 ledger's own precedent (`TestGetEdgeHistory` → RETIRED, "edges surface on entity pages"), edge fields (`rate_of_profit`/`rent_burden`/`age_ticks`) are read inline by whichever dossier consumes the edge, not as a standalone view. No fabricated port; genuinely nothing to cite yet |
| `tests/integration/test_inspector_endpoints.py` | `TestInspectOrg` | **RE-GUARDED** | "Unknown id → all-None dossier" already pinned by `tests/unit/projection/test_organization.py::TestHonestAbsence::test_unknown_to_both_graph_and_world_yields_all_none` |
| ″ | `TestInspectNode`, `TestInspectCommunity`, `TestInspectEdge`, `TestInspectHex` | **STAYS-WEB** | Generic node envelope (no generic "node" projection — the Archive projects per-type views, by design); community inspection differs by design (`CommunityView`'s `community_id` is a fixed 14-member enum, "unrecognized string is a caller error, never an absence" — a different contract, not a gap); edge/hex inspection follow the WO-52 ledger's own precedent (`TestGetEdgeHistory`, `TestHexFeaturePropertiesHabitability`, `TestHexStateRowStateFips` — all RETIRED there for the same reason: no dedicated view, fields surface inline or via `v_hex_state_asof`'s fixed column set) |
| `tests/integration/test_communities_endpoint.py` | `TestCommunitiesEnvelope` | **CARRIED** | Tests a permanent stub (`get_communities_dashboard` always returns `{"communities": []}` pending the XGI-membership query — `ADR039#follow_up_specs`). The real community projection (`CommunityView`, `tests/unit/projection/test_community.py`) already exists and is honest about the same underlying absence (`CommunitySystem.step` is a structural no-op per `sentinels/seam/registry.py`'s `STRUCTURALLY_IMPOSSIBLE` marker — no scenario populates `community_memberships` yet). Nothing to port: the stub and the real projection are both honestly empty for the same root cause: a future community-membership WO would light up both simultaneously |
| `tests/integration/test_timeseries_endpoint.py` | all classes | **CARRIED** | The 6-metric timeseries reshape (+ Program-23 scissors series + veil gating + Task-19 crisis series) has no projection-layer equivalent yet. This is exactly the master plan's spine E "ONE genuine build": `v_*_trend` DeclaredViews over `tick_summary` (T5 narrator train, not T1.2). Porting this now would either fabricate a nonexistent view or duplicate the web reshape math with no Archive consumer — premature. The Veil-of-Money gating *mechanism* itself (not this specific payload) is separately, already covered: `tests/unit/projection/test_veil_gating.py` (WO-41 PORTED, per the WO-52 ledger's `TestEconomyDashboardVeil` row) |
| `tests/integration/test_action_lifecycle.py` | `TestSubmitActionPersists`, `TestResolveTickProcessesActions` | **RE-GUARDED** | Mock-choreography tests of the *web bridge's own* call wiring (submit→`persistence.submit_turn`, resolve→`persistence.persist_tick`) — implementation-coupled scaffolding, not a behavioral contract (CLAUDE.md "Tests as Behavioral Contracts": mock-choreography tests "die with the code"). The real submit→queue→fold→adjudicate behavior is already ported at the projection layer: `tests/unit/projection/verbs/test_submit.py` (WO-39 PORTED, `TestEngineBridgeActions`/`TestActionInjection` rows in the WO-52 ledger) + `tests/integration/archive/test_verb_resolution.py` (WO-39 REWRITTEN) |
| `tests/integration/test_org_serialization.py` | `TestOodaPhaseDerivation`, `TestShortNameDerivation` | **CARRIED** | Pure-function tests (`_derive_ooda_phase` argmax-tiebreak, `_derive_short_name` truncation) — real logic, but `OrganizationView` (`projection/view_models.py`) has no `ooda`/`short_name` field, and no vault render surfaces a derived OODA-phase string or truncated display name for organizations today. Not a fabricated gap: the OODA *profile* itself is a real, wired `Organization` model field (readable via other means); only these two specific *derived display strings* have no projection consumer yet |
| ″ | `TestSerializedOrgShape` | **CARRIED** | `player_controlled`/`vanguard`/`legitimacy`/`opacity`-default/`hyperedge_memberships` — same reasoning; no `OrganizationView` field for any of these today (`legitimacy`/`opacity` exist on other views under different semantics, not this one's 0.5-default-on-absence contract). Future organization-dossier WO scope, not fabricated here |
| `tests/integration/web/test_full_persistence.py` | `TestFullTickPersistence` | **RE-GUARDED** | The underlying claim (tick 0 create + every resolve fills `tick_summary`/`org_snapshot`/`edge_snapshot` via the shared `PostgresRuntime.persist_full_tick`) is already exercised end-to-end on the Archive side by `tests/integration/archive/test_county_e2e.py` (WO-7, real engine tick loop + real persistence) plus this WO's own `test_session_persistence_contracts.py::TestPersistFullTickAtomic` (the same seven-table write, pinned directly) |
| ″ | `TestWebPathMigrations` | **STAYS-WEB** | Genuinely Django/web-history-specific: heals a live *web* Postgres DB created before migrations 0033/0034 existed. The Archive's headless runner applies the same DDL via its own `_apply_migrations(pool)` call (`runner.py`) against a freshly-created DB every run — there is no equivalent "pre-existing stale schema" history to heal on that path |
| `tests/integration/test_rate_criteria.py` | `TestSC003ActionToResultRate` | **RETARGETED** | Already drove `PostgresRuntime` directly with no web import; retargeted in place (scenario tag `spec-061-t130-sc003` → `archive-t12-rate-sc003`, docstring reframed) — no assertion changed |
| ″ | `TestSC006PgVectorQueryRate` | **RETARGETED** | Already drove `PgVectorStore` directly with no web import; retargeted in place (collection/source tags reframed) — no assertion changed |
| ″ | `TestSC007EngineBridgeBootRate` | **STAYS-WEB** | Drives Django's `GameConfig._initialize_engine_with_retry` boot-retry loop directly — no Archive analogue (the headless runner has its own, separately tested, boot sequence; see `tests/unit/engine/headless_runner/test_runner_engine_invocation.py`) |
| `tests/integration/test_engine_bridge_boot.py` | `TestEngineBridgeBootRetry` | **STAYS-WEB** | Django `AppConfig.ready()` retry-then-`sys.exit` semantics — no Archive process boots this way |
| `tests/integration/test_health_detail.py` | `TestPublicHealthEndpoint`, `TestHealthDetailObscurity`, `TestHealthDetailDiagnostic` | **STAYS-WEB** | `/health/` + `/health/detail/` are Django REST endpoints for an orchestrator liveness probe; the Archive is a local single-player TUI process, not an orchestrated service — no analogue needed |
| `tests/integration/test_mid_session_503.py` | `TestEngineAvailabilityMiddleware` | **STAYS-WEB** | Django middleware translating a lost DB connection into HTTP 503 — no HTTP layer in the Archive to translate for |
| `tests/integration/test_purged_session_404.py` | `TestPurgedSession404` | **STAYS-WEB** | Django URL-routing / 404 semantics for a purged `game_session` row — web-transport-specific |
| `tests/integration/test_seed_initial_game_command.py` | `TestSeedInitialGameBridgeContract` | **STAYS-WEB** | A Django management command (`seed_initial_game`); the Archive's campaign creation path is the (separately specced, T4-future) campaign menu / `PostgresRuntime.create_session`, not a Django command |
| `tests/unit/persistence/test_pgvector_store.py` | all | **already correctly tiered** | Already lives under `tests/unit/persistence/`, imports only `babylon.persistence.pgvector_store` — never was a web test; the `spec 061`/`spec-061-us1` tags are historical FR-numbering only. No action needed |

## Divergences recorded

- **Headless-runner RNG-seed dormancy** (see `TestResolveTickThreadsRngSeed` row
  above). Genuine, previously-undocumented finding: `SimulationConfig.rng_seed` is
  wired end-to-end on the web path (spec-061 FR-024) but dormant on the Archive's
  headless-runner path. Not fixed by this WO (surgical scope: test ports, not
  engine wiring) — surfaced for the T1.2 keel's assumptions-ledger and a future
  owner-scoped WO.
- **Deliberate unknown-severity-default divergence** (see `TestSeveritySchema` row
  above). The legacy web bridge defaults an unrecognized `EventType` to
  `"informational"` (silent); `babylon.tui.chronicle_salience.classify_event_salience`
  deliberately defaults to `"warning"` + `unclassified=True` instead (Constitution
  III.11, "Loud Failure") — already ratified and already pinned by
  `TestUnclassifiedSurfacesLoud`; this ledger does not "port" the old default, since
  doing so would re-introduce the exact silent-degrade bug the newer code fixed.

## What this WO did NOT do

- Did not touch `web/observatory/`, `tests/unit/observatory/`, or
  `tests/integration/observatory/` — per decision #10, the Observatory stays as a
  local diagnostic tool until a future Grafana metropole; out of scope here.
- Did not delete or modify the behavior of any of the 19 original spec-061 files
  beyond the two in-place retargets (`test_rate_criteria.py`'s SC-003/SC-006, tags
  and docstrings only — every assertion is byte-identical to before).
- Did not build the three CARRIED gaps' missing projection features (economy
  wealth/edge dossier, `v_*_trend` timeseries views, organization OODA/vanguard
  display fields) — that is out of this test-port unit's surgical scope; each is
  cited to its owning future WO rather than fabricated.
