# Test Estate Audit & Sharpening Plan — 2026-07-16

Owner directive: benchmark everything, everything green, tests match the codebase,
sane CI (dev = fast/loud, main = heavy hitters), ghost data explicitly declared and
centrally recorded, meaningful AI-readable logs, no long-winded legs in the dev PR gate.

Method: 10-leg single-flight benchmark (all suites, wall-times + JUnit/JSON artifacts) +
27-agent read-only audit fan-out (every tests/ slice, frontend, CI, IaC, reverse coverage,
ghost data) + Context7 doc-grounding (pytest/xdist/cov/hypothesis, vitest/playwright) +
claude-mem history mining (rulings that bind this refactor). Raw findings:
workflow `wf_8b91461c-f33` journal; benchmark logs in `reports/test-results/`.

## 1. Benchmark (local, dev box; CI-parity commands)

| leg | command | wall | verdict |
|---|---|---|---|
| unit-ci | mise run test:unit-ci | 277s | 10,080 pass / 17 skip / 4 xfail / **1 ERROR** |
| rest-ci | mise run test:rest-ci | 5688s† | 1,212 pass / **34 FAIL / 47 ERROR** / 62 skip / 4 xfail — worker crash + loadscope deadlock at 99%, SIGINT-salvaged (see incident section); †includes ~40 min audit-agent CPU contention; slow-unit sub-leg skipped by errexit |
| db-bootstrap | mise run db:bootstrap | 2s | green |
| web-pg | pytest tests/integration/web (PG :5433) | 244s | green |
| int-pg | mise run test:int-pg | 88s | 14 pass / **4 FAIL** — all one family: archival ON CONFLICT (migration 0027/0028) |
| refdb | pytest -m requires_reference_db | 477s | 191 pass / 10 xfail — green vs pinned ci-data subset |
| doctest | mise run test:doctest | 2s | green (47 collected) |
| qa-regression | mise run qa:regression | 4s | green — all 5 scenarios byte-identical (verified in log, not just exit code) |
| tick-budget | mise run qa:tick-budget | 232s | green — "All systems within budget" (FIRST EVER run of the spec-104 gate in a pipeline context) |
| frontend | npm run check && build | 62s | green (pre-existing >500 kB chunk-size warning at build) |
| ai-tests | pytest tests/unit/ai | 6s | green — **217 pass in 3.3s**, decisively cheap enough for the dev fast lane (Phase B5 confirmed) |

Raw logs: `$CLAUDE_JOB_DIR/tmp/bench-*.log`; JUnit artifacts in `reports/test-results/`.

The unit-ci ERROR is `tests/unit/persistence/test_per_tick_transaction_atomicity.py::
test_idempotent_retry_after_crash` — an `integration`-marked Postgres test **living under
tests/unit**, collected by the path-based unit shard, racing xdist workers on shared-DB
migration DDL (UniqueViolation on CREATE INDEX). Root cause class, not a one-off: see §3.1.

## 2. Verdict in one paragraph

The estate is large (~860 test files) and in far better health than a suite this size has
any right to be: the audit found **zero dead imports** across every slice (every
`from babylon.X` resolves), mock/marker discipline is mostly followed, integration tests
are real black-box tests, and the frontend suite is exemplary. The debt clusters into a
small number of *systemic* problems: (1) tests physically located in the wrong tier so CI
runs them in the wrong lane or **never runs them at all** (the biggest single class — a
whole set of real acceptance suites is permanently dark); (2) ~80 files carrying vestigial
`import networkx as nx` used only for now-false type annotations (Amendment L residue,
invisible because mypy excludes tests/); (3) a cluster of tautological/vacuous "property"
tests that can never fail (false confidence, worse than no test); (4) copy-pasted fast-lane
job definitions drifting between ci.yml and main.yml; (5) fixture duplication +
a fixture graveyard; (6) stale RED-phase framing on long-green tests.

## 3. Findings → actions (phased; each phase = one or more conventional commits)

### Phase A — kill dead weight (mechanical, safe)
- **A1 NetworkX annotation sweep** (~80 files): drop `import networkx as nx` where used
  only for wrong `nx.DiGraph`/`nx.Graph` annotations on `BabylonGraph`/`BabylonUGraph`
  objects; fix annotations. KEEP legitimate uses: the documented nx-compat surface
  (Constitution II.12) and the differential oracle in
  tests/unit/organizations/test_connectivity_instance.py. networkx is NOT a declared
  dependency (transitive via xgi/torch) — this is a live fresh-install breakage risk.
- **A2 Delete dead tests** (each verified dead, targets removed contracts):
  - tests/scenario/ (entire tree — two empty __init__.py, spec-070 scaffolding never built)
  - tests/unit/tools/test_parameter_analysis.py::{TestRunTrace,TestWriteCsv,
    TestExtractSweepSummary,TestRunSweep::test_run_sweep_result_has_entity_final_states}
    (assert a 3-tuple run_trace contract retired by spec-064)
  - tests/integration/tools/test_parameter_analysis.py::TestIntegration::{
    test_trace_captures_wealth_changes,test_trace_includes_phase_4_1b_metrics,
    test_json_export_captures_dag_structure} (same retired contract)
  - tests/fixtures/test_data/sample_embeddings.py::TestChromaDBCrud (ChromaDB API,
    nonexistent fixture, never collected) + tests/fixtures/test_data/test_sample_embeddings.py
    (asserts its own constants)
  - tests/contract/reference/ (+bea/) — empty spec-068 scaffolding
  - tests/unit/engine/test_endgame_detector.py::TestEndgamePriority (xfail names its own
    live replacement: tests/unit/balkanization/test_endgame_priority_order.py)
- **A2-endgame — outcome-doctrine prune (OWNER RULING 2026-07-16, in-session)**: "we don't
  want to test for specific endgame outcomes — the game is over after a century and the
  endgame is Emergent" (extends the 2026-07-14 emergent-endgames direction). Tests may use an
  outcome as a fixture *vehicle* to trigger machinery; the asserted *subject* must be mechanics
  (wiring, emission, termination, determinism), never the 5-outcome adjudication doctrine.
  Applied:
  - DELETED tests/unit/balkanization/test_endgame_priority_order.py (whole file — FR-033
    cascade contract; supersedes the earlier plan to treat it as TestEndgamePriority's
    replacement coverage)
  - DELETED tests/unit/balkanization/test_endgame_revolutionary_victory_augmented.py
    (spec-070 augmented-RV predicate pinning)
  - DELETED test_endgame_detector.py::{TestRevolutionaryVictoryDetection,
    TestEcologicalCollapseDetection,TestFascistConsolidationDetection,TestEndgamePriority}
    (kept: enum/protocol/initial-state/event-emission/lifecycle — observer plumbing)
  - DELETED tests/unit/web/test_endgame_priority.py::TestEndgameDetectorFR033Priority
    (kept TestBridgeEndgameCoverage — bridge must surface whatever the detector fires;
    Program 17 seam coverage, outcome-agnostic in subject)
  - COLLAPSED tests/scenarios/test_endgame_flow.py::TestSimulationTermination 3→1: kept a
    single detector-fires⇒run-stops mechanism test; deleted the RV (already xfail-stale) and
    ecological variants
  - KEPT tests/integration/test_endgame_detection_round_trip.py — tests the
    --endgame-detector *plugin machinery* with a synthetic detector (exit at tick N /
    run-to-completion without one); that IS the fixed-horizon direction, not doctrine
  - Consequence, recorded honestly: EndgameDetector's 5 _check_* predicates now have NO
    direct unit coverage. Deliberate — the predicates are doctrine slated for demolition by
    the emergent-endgames ADR (memory 2026-07-14); their tests would entrench exactly what
    the ADR unwinds. The detector's *plumbing* (protocol, latch-once, event emission,
    lifecycle, bridge surfacing) remains covered until the unwind.
  - tests/unit/protocols/test_mypy_protocols.py (zero test functions; its mypy-run
    verification is unreachable — tests/ excluded from mypy)
  - ADR062 rewrite-test check applied: none of these is the sole record of a contract.
- **A3 Stale RED-phase / stale-docstring sweep**: test_metabolism.py, test_multiverse_factory.py,
  adapters/test_aggregation_mixin.py (drop dead try/except+skipif scaffold),
  contract test_systembase_inheritance.py / test_faction_contract.py / test_decision_contract.py /
  test_audit_report_contract.py, test_headless_runner.py stale skip ("endgame not implemented"
  — it shipped), ~6 vestigial hasattr-only RED relics in economics, stale NetworkX-semantics
  comment in test_runtime_db.py.

### Phase B — right test, right tier (the core of "sane CI")
- **B1 Hermetic unit shard**: relocate Postgres-touching tests out of tests/unit
  (test_per_tick_transaction_atomicity.py, test_trace_view_columns{,_v2}.py,
  test_migration_apply_healing.py, test_pgvector_store.py round-trip class, postgres
  session/turn/trace-spatial files as marker-verified) → tests/integration/persistence/;
  add `requires_postgres` markers; add `not integration and not requires_postgres` to
  test:unit / test:unit-ci -m filters as belt-and-braces. Relocation (not filter alone)
  because rest-ci ignores tests/unit — filtered-out tests would run NOWHERE.
  This also fixes the benchmark's unit-ci ERROR (xdist DDL race on shared PG).
- **B2 requires_reference_db backfill** (suites that NEVER run in any CI job today —
  refdata-tests fetches the DB but selects by this marker):
  tensors/test_empirical_validation.py (17 tests), economics/test_data_layer_e2e.py,
  economics/test_detroit_wiring.py, test_qcew_live_reconciliation.py,
  test_reference_data_window_policy.py, test_tensor_hydration.py (~17 tests incl. the
  accounting-identity contract), unit/persistence/test_county_aggregation.py (3 classes).
- **B3 Known-red refdb test**: tick/test_imperial_rent_real_wiring.py → xfail(strict=True)
  per reports/heavy-tier-triage-2026-07-15.md item 4 (IMPORT_USE rows absent until ci-data-v3).
- **B4 BABYLON_SLOW_TESTS is a dead switch** (never set anywhere): convert its 9+ gated
  acceptance tests (SC-005/006/012 suites, wallclock smokes) to pytest.mark.slow so they
  land in the already-wired rest-ci slow leg. Delete the bespoke env-var gate.
- **B5 tests/unit/ai is 100% deterministic but excluded from every blocking CI leg**
  (~5,000 lines, 11 files, all MockLLM/mocked-network): drop the conftest blanket
  `ai`-marker hook, remove `--ignore=tests/unit/ai` from test:unit/test:unit-ci/test:all/
  test:cov, keep the `ai` marker for genuinely non-deterministic evals (currently none in
  tree). Restores real regression coverage to the merge gate.
- **B6 Wall-clock tests out of the fast lane**: test_fracture_operation_o1.py timing tests
  → pytest.mark.benchmark; tests/test_simplex_invariants.py (orphan at tests/ root, pure
  fast Hypothesis math) → tests/property/.
- **B7 Postgres-tier CI home** (main tier): extend postgres-integration job to also run
  `tests/integration/{balkanization,persistence,observatory,engine/headless_runner}
  -m requires_postgres` — today those suites' real assertions have never executed in CI.
- **B8 Misplaced/mismarked**: test_falsification_criteria.py + test_inspect_qcew_audit.py
  (pure unit-shaped, zero markers, in tests/integration) — mark properly;
  test_map_api.py drops wrong `unit` marker (real Django DB I/O);
  economics test_static_economy_flow.py class that needs neither PG nor Django → dev tier.

### Phase C — make the false-confidence tests honest (behavioral-contract repairs)
Tautological/vacuous tests that can never fail (worse than no test — they report
coverage that does not exist):
- property/invariants/test_population.py (`assert x == x`) — rewire to real
  LifecycleSystem/VitalitySystem tick per its own TODO, or delete (its integration sibling
  keeps the surface). DECISION: delete + note; engine wiring is a feature task.
- property/invariants/test_aggregate_equalities_property.py — delete (integration sibling
  test_aggregate_equalities.py covers the real surface).
- property/invariants/test_proportional_scaling.py — docstring falsely claims to exercise
  scale_c_v_preserving_s_over_v (dead helper, zero callers): wire it for real.
- property/invariants/test_wealth_heat_bounds.py Predicate C self-fills gaps before
  checking — snapshot before fallback so omissions can actually fail.
- property/systems/test_metamorphic.py — implement the promised zero-output/idempotence
  checks or rename; drop type-guaranteed wealth>=0 tautologies.
- property/invariants/test_consequence_after_actions.py AS1 — worldstate_strategy never
  generates organization nodes so the sweep is vacuous: add org-bearing strategy.
- property/invariants/test_frozen_discipline.py Predicate C — actually call
  assert_no_in_place_mutation under pytest.raises.
- unit/topology/test_topology_monitor.py tautology (`not all(x) or all(x)`) → real assertion.
- unit/bifurcation/test_analysis.py (asserts membership in the full enum) and
  test_axis.py (never asserts the value its name promises) → real assertions.
- unit/reference/qcew/test_singlefile_classify.py bare generator-expression assert → real.
- unit/persistence/test_hex_hydrator_perf.py "parallel" tests never cross the
  _PARALLEL_POLYFILL_THRESHOLD=8 — monkeypatch threshold or 8+ county fixtures.
- unit/engine/test_runner.py concurrent-serialization test (asserts only a private attr
  exists; JS `.then()` on a coroutine = dead branch) → rewrite with asyncio.gather or delete.
- contract/engine/test_systembase_inheritance.py hardcodes 22 systems; engine has 28 —
  derive from _DEFAULT_SYSTEMS so the contract self-updates.
- integration/test_endgame_detection_round_trip.py "at_tick_3" actually runs 300 ticks —
  add FireAtTick3 fixture class; mark slow.
- integration/balkanization/test_audit_round_trip.py unconditional skip citing a fixture
  that exists — wire the sketched pg_pool body (FR-049).
- unit/economics/test_factory_shims.py resolves a pre-Program-14 path; xfail masks
  file-not-found as a LOC rationale — fix path, re-evaluate xfail.
- economics/test_coefficient_lookup_policy.py (private-method bypass) and
  test_derived_tensors.py (tautological assertion) — restore real coverage.
- integration/test_visibility_integration.py blanket skipif(True) un-deadened.

### Phase D — pytest/vitest config modernization (Context7-grounded)
- `--color=yes` → drop (auto default honors NO_COLOR; explicit yes overrides the repo's
  own agent-ergonomics env).
- Add `filterwarnings`: `error::DeprecationWarning:babylon.*` (+ targeted third-party
  ignores) — Loud Failure for first-party deprecations.
- Consider `strict_parametrization_ids` (pytest 9) — add; `strict_xfail` — evaluate against
  existing xfail(strict=False) population before enabling (explicit strict= wins, but
  unmarked xfails flip; run the suite once before committing).
- xdist: test:unit-ci gets `--dist loadscope` to match rest-ci (scope-bound fixtures stay
  on one worker; also mitigates shared-resource races).
- Hypothesis: tests/property/conftest.py already registers dev/ci/nightly profiles that
  NOTHING loads — wire HYPOTHESIS_PROFILE env into mise tasks/CI (dev=fast local,
  ci=deadline None on heavy lane).
- pytest-randomly: standardize `-p no:randomly` on PG/state-touching tasks (test:pg,
  test:int) like test:int-pg; capture --randomly-seed into reports/ artifacts elsewhere.
- vitest.config.ts: `restoreMocks: true`; GitHub Actions annotation reporter when
  GITHUB_ACTIONS=true. Coverage thresholds: docs claim 80/75/80, config has none — OWNER
  CALL: implement or fix the doc (verifiability principle).

### Phase E — CI workflow repairs (dev fast / main heavy / nightly deep)
- main.yml fast-gate is missing check:seams (added to ci.yml 2026-07-12, never propagated —
  both jobs share the branch-protection check name). Fix now; then factor the fast lane
  into a reusable workflow (workflow_call) so ci.yml/main.yml cannot drift again.
- docs.yml + dependabot-automerge.yml: add timeout-minutes (last two workflows violating
  the house Loud-Failure rule).
- Wire the built-but-never-run gates: check:coverage (static, cheap → fast lane),
  qa:tick-budget (→ nightly, advisory first; the ONLY engine-perf gate, currently unwired).
- playwright job: drive the whole testDir and let the config's project routing decide
  (two backend-free specs — event-popup, inspection-stack — are silently unwired today).
- B7's postgres-integration extension (above).
- tests/baselines/michigan-e2e.json is a Git-LFS pointer no workflow fetches — consumer
  only checks exists() → future confusing JSONDecodeError. Add lfs:true to the checkout
  that needs it or make the consumer detect pointer files loudly.
- Artifact retention: cap high-frequency uploads (unit/rest results) to ~14 days.
- Dead-duplicate mise tasks: qa:patterns vs qa:patterns-strict duplication, shadow
  qa:security vs security:pip-audit — consolidate.

### Phase F — shared-infra consolidation
- tests/README.md is actively misleading (documents ChromaDB fixtures, tests/unit/data/,
  tests/unit/ui/ — none exist). Rewrite from what conftest actually exports.
- QCEW fixture machinery duplicated ~150 lines between integration/tensors/conftest.py and
  integration/economics/conftest.py (self-acknowledged) → one shared module.
- tests/integration/web: 6 files redefine a `bridge` fixture shadowing the shared one.
- Detroit tri-county FIPS literals triplicated (constants.py DetroitMetro is dead while
  6 files re-declare) → import from tests/constants.py everywhere.
- tests/contract: apply the `contract` marker to all 18 files (5/18 today); dedupe
  _make_defines into the existing conftest.
- Fixture graveyard: remove ~24 never-consumed fixtures (mock_llm_provider et al.) after
  a use-check; delete tests/mocks/metrics_collector.py if unconsumed.
- Mock hygiene: one specced _make_mock_persistence factory in tests/unit/web conftest
  (replaces ~5 duplicated unspecced helpers; RuntimePersistence spec=).
- Declared-mocks registry (owner goal: "intentional mocks centrally recorded"):
  new `docs/reference/declared-synthetic-data.rst` + a static sentinel check listing every
  sanctioned synthetic/fallback in production paths (StubEngineBridge DEBUG-gate,
  employment-100k graceful-degradation default + economics_fallbacks tally, seed scenario
  fixtures) — audit confirmed these are all guarded/instrumented; the registry makes the
  set closed and reviewable. (Ghost sweep found NO unguarded fake data in production paths;
  the old "employment 100k honesty gap" note is stale — it self-documents + instruments.)

### Phase G — owner-triage list (NO action without your ruling; Respect Existing Code)
Tested-but-unwired or zero-reference production code the audit surfaced (report only):
- EventTemplateSystem: full SystemBase subclass + 600-line mutation-killing test suite;
  absent from _DEFAULT_SYSTEMS — register it or mark experimental.
- TopologyMonitor: 42 test instantiations, zero production callers.
- compute_ollivier_ricci, DistributionSplit, scale_c_v_preserving_s_over_v (dead helper),
  field_registry (5 test files, no production wiring).
- 39 src/babylon modules with zero test references (engine/actions/*, optimization
  internals, persistence/{serialization,sqlite_hydrator,...}, bea/ingest/*, ...) and
  ~23 modules with zero internal imports — full lists in the workflow journal.
- web: log_handler.py sanitize_for_log (security-relevant, 15+ call sites, zero tests),
  tick_resolver.py (live endpoint, zero direct tests) — candidates for NEW tests.
- Hickel CSV + GDP XLSX tests read the babylon-data drive directly (violates the
  2026-07-14 no-drive ruling) — needs the parquet-artifact migration, not just markers.
- vitest coverage thresholds (implement vs un-document).
- Frontend coverage gaps: EndStateScreen outcome branches, endpoints.ts path builder,
  Doctrine Tree fixture sync guard.

### Rulings honored throughout (from history mining)
mutation = local-only never CI; max-verbosity AI-readable CI logs preserved everywhere;
coverage gate stays narrow (engine/systems>=80) — no widening, no heavy-shard cov until
the xdist OOM is solved; reference-DB tests stay SQLite (ADR037 rejected testcontainers
unification); skips need a disposition class; bounded-poll (never longer sleeps) for
flaky fixes; ADR062 rewrite-test before any deletion; promote.sh treats "advisory"
substring as non-blocking — naming preserved.

## Live-run findings — benchmark incident, 2026-07-16 (owner-approved for incorporation)

The rest-ci benchmark leg **deadlocked at 99%** and had to be salvaged by SIGINT.
Owner ruling (14:3x): everything discovered here folds into the refactor + CI.

### The incident chain (all evidence in `$CLAUDE_JOB_DIR/tmp/bench-rest-ci.log`)

1. **Worker hard-crash:** xdist gw0 died ("node down: Not properly terminated") during
   *setup* of `tests/integration/test_health_detail.py::TestHealthDetailDiagnostic::
   test_staff_returns_diagnostic` (the pytest-django `staff_user(db)` fixture, ~13:44).
   No faulthandler dump surfaced (pytest enables it by default and it prints on
   SIGSEGV/SIGABRT) → points to **SIGKILL** (earlyoom / kernel OOM); kernel + journal
   logs need sudo, so signal identity is UNVERIFIED. The view under test only echoes
   config pins (no model load, no bridge boot) — the test body is innocent-looking.
   → **Action (triage):** reproduce the file in isolation post-benchmark, watch RSS.
2. **loadscope stranding = scheduler deadlock:** xdist restarted the worker and charged
   the crash to the running test, but the ~13 tests still queued on gw0's scope queue
   were NEVER redistributed. Controller + 4 workers sat in `futex_wait` at 0 CPU for
   5+ min at "99%". Upstream docs confirm crash handling covers only the *crashing
   item* (restart + fail); a crashed node's remaining scope queue has no recovery
   path under `--dist loadscope`.
3. **Salvage pattern that worked:** `kill -INT <coordinator>` produced a full clean
   summary (1212 passed / 34 failed / 47 errors / 62 skipped / 4 xfailed, 1:34:46)
   plus a complete junit.xml. A hosted runner hitting this wedge would instead burn
   the whole 35–45 min `timeout-minutes` and die with NO summary artifact.

### CI hardening actions (append to Phase E)

- **E-9 — `--max-worker-restart=2` on every xdist invocation** (.mise.toml test:unit-ci,
  test:rest-ci, test:unit, …): bounds crash-loops; repeated worker death becomes a loud
  session error instead of infinite respawn. (Does NOT cure the loadscope stranding.)
- **E-10 — self-salvaging leg timeouts:** wrap heavy pytest legs at the mise-task level
  in `timeout --signal=INT <cap>` (cap ≈ job timeout − 3 min) so a wedge interrupts
  pytest cleanly → summary + junit still land as artifacts, replicating the manual
  salvage. Job-level `timeout-minutes` stays as the outer axe.
- **E-11 — loadscope exposure decision:** rest-ci keeps `--dist loadscope` for scope-
  fixture economy, but with E-10 as the wedge guard. If the crash test's root cause is
  memory (the known "xdist OOM" family — note workers hit 1.3 GB RSS *without*
  coverage), revisit worker count / dist mode with the durations data.
- **E-12 — wall-time honesty:** local rest-ci took 1:34:46 (with ~40 min of audit-agent
  CPU contention early on) vs nightly's 35-min `timeout-minutes` — the margin on hosted
  runners must be re-measured from a clean run before trusting the green checkmarks.

### New reds banked for triage (task #3), beyond the unit-ci atomicity race

- `tests/integration/test_health_detail.py` — worker-killer (root cause open, above).
- `tests/integration/test_events_capture.py::test_engine_emitted_event_visible_in_summary` FAILED.
- `tests/integration/test_ci_gate_baseline_compare.py::test_fresh_run_matches_baseline` FAILED.
- Full bucket of 34 F + 47 E to classify from junit.xml once all legs land.
- int-pg leg (exit 1, 88s): 4 failed / 14 passed — ALL four in
  `tests/integration/test_archival_integration.py` (Export/Purge/ArchivedQuery) with the
  same `psycopg.errors.InvalidColumnReference: no unique/exclusion constraint matching
  ON CONFLICT` — the known migration-0027(h3_index)/0028(composite-PK) family from the
  2026-07-07 recon; one root cause, one fix.
- ~13 stranded never-started tests (reconstruct via `--collect-only` diff vs log).
- The slow-marked-unit sub-leg of rest-ci was skipped by the leg's `errexit` after the
  salvage exit — benchmark it separately before re-tiering slow tests (Phase B4/B6).

### DDL-family triage — the resolution arc (2026-07-16, proof runs 1–3)

Layered root causes, each fix exposing the next:

1. **Catalog race** (`UniqueViolation pg_class_relname_nsp_index` + applier-vs-applier
   deadlocks): `IF NOT EXISTS` DDL is idempotent but NOT concurrency-safe. FIX =
   `schema_apply_lock` session advisory lock (`postgres_schema.py`), taken by all four
   appliers. Proof run 1: family eliminated.
2. **Stale `ON CONFLICT (h3_index)`** (InvalidColumnReference): migration 0028 made
   hex_spatial_map's PK composite `(session_id, h3_index)`; three test writers (observatory
   conftest seed/unseed, session_partitioning, archival_integration) still wrote the bare
   key — and the un-scoped unseed DELETEd sibling sessions' rows (the suspected
   TerminalAggregateResolutionError trigger). FIX = session-scoped inserts/deletes.
   Proof run 2: family eliminated.
3. **Applier-vs-reader deadlocks** (12× `DeadlockDetected`, surfaced BY fix 1):
   `0030_views_current.sql` re-executes view DDL on every apply by design; its
   AccessExclusiveLock on the views deadlocks against sibling workers' tests reading them.
   Serializing appliers made them queue politely and then still DDL into live traffic.
   FIX = **digest-stamped apply-once** (`ensure_ddl_applied`): sha256 over the exact DDL
   set; already-applied sets fast-path via pure SELECT (zero DDL, zero locks); first apply
   runs while sibling appliers are parked on the advisory lock, not while tests run. A
   failed/killed apply leaves no stamp → the healing retry re-applies exactly when it
   should. Contract pinned in `tests/integration/persistence/test_schema_stamp.py`;
   `test_migration_idempotency.py` drops the stamp between passes so its load-bearing
   second pass still genuinely re-executes (it would otherwise have been silently defanged
   — the exact Phase-C false-confidence pattern).
4. **Series-shape failures** (4×, `test_endpoints_data.py::TestSeries` — missing ticks in
   national/county/state series): CONFIRMED COLLATERAL — gone in proof run 3 once the
   applier deadlocks stopped poisoning the module fixtures.
5. **Partition-lifecycle deadlocks** (proof run 3: 71P/4F/1E in 7:26 — stamp fast-path
   also cut wall-clock 4× vs proof 2's 30 min): the remaining family was
   `partitioning.py` — BOTH `ensure_session_partitions` and `drop_session_partitions`
   ran all 9 families in ONE pool transaction, accumulating parent-table
   AccessExclusiveLocks against concurrent multi-table readers (archival purge tests +
   observatory reads). Real production hazard, not test flake: a live dashboard reading
   during a purge deadlocks the same way. FIX = one `conn.transaction()` per family —
   the loop holds at most one parent lock at a time, so it can WAIT behind a reader but
   can no longer form a lock cycle. (Deliberately NOT `conn.autocommit = True`:
   psycopg_pool's `_reset_connection` never restores a leaked autocommit flag, so
   mutating it on a pooled conn bleeds into unrelated checkouts — verified against the
   installed pool source.) Collateral in the same run: observatory 503 + one setup
   deadlock (same DB moment), plus one test-double break —
   `test_migrations_dir_resolution_is_cwd_independent`'s fake conn spoke the old
   one-arg-loop protocol; updated to `ensure_ddl_applied`'s execute/fetchone protocol
   with the assertion filtering to migration-header texts. Proof run 4 = the green gate
   for the whole DDL-family commit.

### Phase C execution notes (17-agent fan-out, 2026-07-16)

All 17 audit premises confirmed on inspection (two with path drift: the "dead helper"
lives in `tests/_helpers/invariants/`, not src/; visibility file under
`integration/economics/`). Latent findings surfaced by agents, banked for follow-up:

- `tests/integration/economics/test_aggregate_equalities.py` proportional-prices arm
  computes labor aggregates from its own profit/price ÷ hardcoded τ=65.0 — self-admittedly
  "trivially passes today". Same family as the deleted property tautology; needs a real
  MELT source. (Phase C follow-up.)
- `test_topology_monitor.py`: two further false-confidence spots adjacent to the fixed
  tautology (resilience-interval checks). (Phase C follow-up.)
- **Production finding** (not touched — test-only sweep):
  `ImmutableReferenceLookup.get()`'s FR-041 clamp-to-earliest branch is dead code from the
  public API — `_tick_to_year_and_week` derives `simulated_year` from the same base year,
  so no valid tick can reach below-range. Owner-triage list.

### Phase D validation gate + fallout fixes (2026-07-16, commits `95b10fa3` + `194b3315`)

Full `mise run check` under the new config: all non-test legs clean; test:unit
**12 failed / 10,316 passed / 1 xfailed in 6:27** (`-n 4 --dist loadscope`). Both
red families were new-config discoveries, fixed same-day:

1. **10 melt tests** — `filterwarnings = error::DeprecationWarning:babylon.*`
   caught production `get_class_distribution_estimate` internally calling the
   deprecated `estimate_la_share`. Because the warning uses `stacklevel=2`, the
   filter is surgically precise: it errors ONLY on first-party internal use
   (warning attributed to the babylon caller module); tests exercising the
   deprecated API directly get a plain visible warning. Fix: computation
   extracted to `_estimate_la_share_unwarned`; public method still warns
   external adopters toward Feature 043. **Keep this pattern**: deprecated
   public APIs delegate to an unwarned private impl for internal callers.
2. **2 healing-stub tests** — the stamped applier's protocol (params-accepting
   `execute`, fetchone-able results, lock/stamp bookkeeping statements) broke
   the one-arg `execute(sql)` stub. Stub re-modeled; apply-count assertion now
   filters to `-- 0NNN`-headed migration texts.

Formerly-red-in-benchmark `test_per_tick_transaction_atomicity.py` passed
under 4-way xdist in this gate — the advisory-lock family fix holds.

## Empirical-invariant program (owner direction, 2026-07-16 — "divine the data")

Owner supplied the spirit: the WID/Piketty trove (`/media/user/data/babylon-data/piketty`,
full World Inequality Database dump) expresses a Pareto wealth-ownership law to
test against. **Verified directly** (US, `shwealj992` = net personal wealth,
equal-split adults; p90–99 derived as p90p100 − p99p100):

| year | top 1% | p90–99 | p50–90 | bottom 50% |
|------|--------|--------|--------|------------|
| 1913 | 46.6% | 32.7% | 18.7% | 1.9% |
| 1978 | 21.8% | 42.1% | 35.3% | 0.8% |
| 2010 | 34.6% | 38.4% | 28.8% | −1.8% |
| 2024 | 34.8% | 34.8% | 29.5% | 1.0% |

- **Modern era (≈2010+)**: owner's ~⅓ / ⅓ / ⅓ / ~0 quartile split holds within
  ~±5pp (2023–24 nearly exact). Right calibration anchor for game start.
- **Full-record law (1913–2024)**: bottom-50% wealth share NEVER left
  [−1.8%, +3.7%] — through the New Deal and Great Compression. The Fundamental
  Theorem in empirical form: reformism does not redistribute wealth.

**Owner ruling — conditionality**: these are laws of the *capitalist mode of
production*, not universal laws. Tests must NOT fail when the distribution law
breaks during an in-game communist revolution — that breakage is the signal.
Encoding: (a) *initialization contracts* are unconditional (seeded distribution
must match WID at calibration); (b) *runtime invariants* are conditional —
assert the band holds WHILE no rupture has occurred, and assert the
**contrapositive**: if the distribution leaves the capitalist band (e.g.
bottom-50 share > ~5%), a rupture/revolution event MUST exist in history.
Mechanic-level implication, never outcome adjudication (consistent with the
emergent-endgames testing corollary).

Divination workflow (`wf_11736c1d-2ba`, 4 read-only sonnet sweeps: WID US macro,
WID world structure, trove census, repo attach-points) findings + proposal to
follow in this section.

### Divination results (wf_11736c1d-2ba, 4 sweeps, all verified-computed unless noted)

**Tier 1 — SHIPPED as tests this session:**
1. `tests/unit/config/test_wealth_distribution_invariants.py` — pins
   `class_dynamics.equilibrium_w1..w4` (the FRED-DFA-fitted wealth-share
   equilibrium ALREADY in GameDefines, defines.yaml:414-417) inside cross-source
   WID⋃DFA bands; plus the two full-record structural laws (bottom-50 ≤ 0.04
   historical ceiling; top-decile ≥ 0.60 supermajority floor, 205-year record).
   CI-safe: constants pinned with reproduction commands, no drive access.
2. `tests/unit/reference/test_fred_wealth_shares.py` (`requires_reference_db`
   lane) — un-orphans `fact_fred_wealth_shares` (240 obs, 2010Q1–2024Q4,
   SCF-benchmarked, independent of WID tax data): coverage, bands, strictly-
   positive bottom-50, per-quarter partition-of-100%, bracket→babylon_class
   mapping pin. Redundant-source verification per Constitution VIII.13.

**Tier 2 — strong candidates:** *(owner ruling 2026-07-16 approved β, US
aggregate labor share, and the imperial-rent multiple — ALL THREE SHIPPED in
`11a8307f` as derived in-repo series artifacts + 13 contract tests, extraction
cross-verified against the sweep's independent computations; the rest remain
proposed. Also found en route: `fact_productivity_annual` is EMPTY (0 rows)
while `view_surplus_value` reads from it, and `fact_bls_productivity` holds
5,320 all-zero placeholder rows — two more schema-only vestiges for the
owner-triage list.)*
- **β (wealth/income ratio)** ✅APPROVED: [2.0, 7.0] full 1800–2024 record (225
  continuous years), [4, 6.5] modern. Attach: wealth-accumulation calibration bound.
- **α + labor-share = 1** accounting identity (max residual 0.57pp, 1970–2024);
  **α ∈ [0.20, 0.31]** rising. Attach: defines cross-validators.
- **Wealth concentration > income concentration at every percentile cut, every
  year** — 351 comparisons, 0 violations, min margin 9.5pp. A structural law
  relating two distributions the engine could assert whenever both axes exist.
- **r > g** (54/54 years, 1971–2024) — use 10yr-CAGR-smoothed g (single-year g
  near-tied 2021, gap 0.02pp). Attach: tick-level property with smoothed g.
- ✅APPROVED — **US aggregate labor share ∈ [0.53, 0.60] every year 1997–2024** (BEA
  supply-use: Compensation/Value-Added) — W_c < V_c economy-wide on the whole
  record = the Fundamental Theorem's empirical anchor. NOT encoded anywhere;
  natural home = BEA loader data-fidelity test (refdb/artifact lane).
- ✅APPROVED — **Imperial-rent multiple**: US:SSA per-adult income 14–33× for 115 years,
  no convergence; **labor-aristocracy anchor**: US bottom-50% AVERAGE income =
  3.4–5.0× the SSA MEAN for 75 years (stdev 0.45 — tightest band found).
  Attach: core:periphery seeding contracts + runtime band absent rupture.
- **Global top1/bottom50 income-share ratio ∈ [2.28, 2.90]** (1980–2024, PPP).
- Exact accounting identities for loader round-trip tests: LODES SA=SE=SI=S000
  (0 mismatches, WY-2019 verified); QCEW own_code 1+2+3+5 = 0 to $0 (county ±1
  employment disclosure noise); Census trade monthly→annual exact over all
  8,973 rows; BEA supply=use ≈ 2.5e-8 relative (existing tests corroborated).

**Runtime conditional invariants (blocked on production work):** the live tick
engine carries POPULATION shares only — the wealth-share axis does not exist in
`ClassDistribution`. The contrapositive law ("distribution leaves the
capitalist band ⟹ rupture event exists") cannot attach until it does.

**Production findings → Phase G owner-triage list:**
1. **Population-vs-wealth conflation + defines violation**: the tuple
   (0.01, 0.09, 0.40, 0.35, 0.15) is hardcoded in FOUR duplicated call sites
   (tick/initializer.py:32-36, tick/graph_bridge.py:243-247,
   tick/system/__init__.py:416-420 + 732-736), none reading GameDefines.
   Proposed: a `wealth_distribution` defines category (WID/DFA-cited) as the
   single source; wire the 4 sites; add the wealth-share axis. Moves
   qa:regression baselines → needs deliberate regen + its own PR.
2. **`formulas/class_dynamics.py` is orphaned** — fully tested (30+ units),
   GameDefines-wired, zero engine consumers. Either wire into the tick loop
   (it's the natural carrier of the wealth axis) or mark its status honestly.
3. ~~Ricci/Hickel silent no-op (III.11)~~ **CORRECTED + RESOLVED 2026-07-16**:
   the sweep agent's claim was WRONG — verification showed the hydrator was
   already rerouted to POPULATED sources (`_copy_hickel_drain` reads
   `fact_hickel_erdi_annual`; `_copy_ricci_unequal` reads `fact_trade_monthly`,
   44,808 rows). The REAL gap: Ricci's actual transfer estimates sat unloaded
   while `fact_ricci_unequal_exchange` was schema-only. RESOLVED in `532b2307`:
   both CSVs ship in-repo (`src/babylon/data/reference/`, owner-directed),
   `tools/ingest/ricci_unequal.py` landed 29 TOTAL rows (+10 RICnn dim_country
   region rows with tiers), 14 new contract tests (Hickel cumulative-sum
   identity exact; Ricci extraction-direction law CORE⟺INFLOW zero violations;
   2007 Non-OECD GVC>TOTAL anomaly PINNED — owner: Bear Stearns crisis onset).
   Remaining vestige for triage: `fact_hickel_drain` schema (per-resource
   drain) has no source data anywhere in the trove.
4. **`fact_census_gini` orphaned** (45,045 rows; national avg county Gini
   rose monotonically [0.4333, 0.4488] 2010–2023) — candidate income-inequality
   regression anchor once a consumer exists.
5. **WID methodology trap (recorded for any future WID loader)**: single-country
   `mnninc`/`anninc` values are REAL reference-year LCU; dividing by same-year
   nominal FX (`xlcusx`) overstates periphery income up to 17× (Nigeria 2000).
   Use WID's pre-aggregated USD regional files (XF-MER, WO-MER…) or PPP.
6. Rejected as invariant: gender income ratio (monotonic trend, not a band);
   TIGER county count 3,235 (coverage anchor only, needs authoritative
   cross-check).

### Remaining-reds battery results (2026-07-16, post-DDL-fix)

Solo `test_health_detail.py`: **6 passed, 0.77s, child peak RSS 158MB** — the
benchmark "worker crash" was pure contention collateral, NOT OOM. -n4 battery:
**29 passed / 10 failed / 4 skipped in 21:09**. Newly GREEN (were red at
benchmark): conservation_audit_strict (patched_advance **extra fix),
hex_spatial_map_isolation (k.n → WITH ORDINALITY fix), us3_tool_smokes,
external_node_flows, engine_bridge substrate_distinguishability,
output_dir_overwrite, marx c_v identities (partial). Still red, families:

1. `test_baseline_determinism.py::test_sc006_recent_regens_epsilon_deterministic`
   — 33% relative error / 86,320 diff cells: NOT epsilon noise; regen artifacts
   on disk likely span code eras (or real non-determinism — triage next).
2. `test_ci_gate_baseline_compare.py::test_fresh_run_matches_baseline` — fresh
   run vs stored baseline mismatch (possibly stale baseline vs Wave-6 changes).
3. Aggregation family (5): cross_scale ×2, hex_to_county_conservation,
   hex_hydration invariants, marx value_added identity — survived the DDL fix,
   so NOT stomp collateral; shared root suspected in hex/county view reads.
4. `test_events_capture.py::test_engine_emitted_event_visible_in_summary`.
5. `test_headless_runner.py::test_conservation_violation_does_not_abort` +
   `test_sigint_partial_artifacts` (exit −2 vs 130).

### Close-out arc (2026-07-16 evening)

**All 10 remaining reds GREEN** (commit `79a9da4a`; re-verification battery
19+2 passed / 3 honest skips in 2:51, was 10F in 21:09). Every fix was a
stale test or test bug — zero production defects: spec-088 spatial-map
seeding (3 raw-loop migration fixtures also replaced with the stamped
applier, ~230s setup each), spec-068 GDP-proxy rewrite for the Marx
identity, SIGINT import-race readiness wait, `**extra` shim forwarding
(events_capture — same drift as conservation_audit_strict), SC-006
input-hash pairing (the 33% "error" was cross-era artifact comparison),
ci_gate like-for-like baseline (michigan-e2e.json became 83-county/520-tick
under the same filename → guaranteed 3-vs-83 mismatch; now gates on
detroit-tri-county-5t.json and PASSES against dev HEAD), LFS pointer guard
(+ `git lfs pull` materialized the 36MB baseline locally), audit
info-severity rows are successes not violations, runner subprocess timeouts
180→600s (5-tick runs take 160–300s under 4-way contention).

**B1 relocation executed**: `test_per_tick_transaction_atomicity`,
`test_trace_view_columns{,_v2}`, `test_migration_apply_healing` →
`tests/integration/persistence/`; the healing applier moved from
tests/unit/persistence/conftest.py (no fixtures — pure helpers) to
`tests/integration/persistence/migration_healing.py`. Remaining
psycopg-importing unit files are mock-based with skip-guarded live extras
and stay. `test_county_aggregation`'s 3 reference-DB classes marked
`requires_reference_db` (refdata lane).

**ci-data-v5 subset cut + released** (owner-delegated `gh release create`;
precedent ci-data-v1..v4): fact_fred_wealth_shares skip→FULL,
fact_ricci_unequal_exchange now populated (29 rows + RICnn dims); 14.1M rows,
sha256 `2e7920e2…`; fetch-reference-db composite defaults bumped. Without
this, the new refdb sync tests would red the refdata CI legs against v4.

### Accepted residue (recorded, deliberately not done this session)

**Polish-tier refactor items** (correctness unaffected; candidates for a
follow-up sweep): QCEW fixture dedup, web-bridge fixture dedup, Detroit FIPS
constants extraction, contract-marker backfill (5/18 done), fixture
graveyard sweep, mock-hygiene pass, strict_xfail enablement (pending the
XPASS census of the ~10 refdb-lane xfails — enabling before the census
could hard-fail XPASSing tests unseen).

**Owner-triage list (production; NO action without ruling)**: wealth-share
axis program (4 hardcoded population-share call sites + orphaned
formulas/class_dynamics.py — gateway to runtime contrapositive invariants);
fact_hickel_drain schema (per-resource drain — no source data exists
anywhere in the trove); fact_productivity_annual EMPTY while
view_surplus_value reads from it; fact_bls_productivity 5,320 all-zero
placeholder rows; fact_census_gini orphaned (45,045 rows, rising-Gini
anchor); FR-041 dead clamp branch; employment-100k placeholder ("Fix C");
py3.13 nightly runner-shutdown flake (pre-existing, owner stop-fixing-CI
ruling).
