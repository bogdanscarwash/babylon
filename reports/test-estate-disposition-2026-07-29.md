# Test-estate disposition tallies — Program 27 Phase 0 (Task 8)

**Date:** 2026-07-29
**Tool:** `tools/test_estate_disposition.py` (run: `python3 tools/test_estate_disposition.py`)
**Scope:** every `*.py` file under `tests/` (excluding `__pycache__`) — **1,329 files**, matching
the file count the plan cites for the estate.

## Doctrine this classification implements

The 2026-07-09 stratification (`project/assessments/TEST_SUITE_REWRITE_AUDIT-2026-07-09.md`),
ratified same-day as Amendment Q (Constitution v2.9.0, III.12 + VIII.13) and program 13
(behavioral contracts), and restated by ADR063 (Program 14): **"tests are REHOMED, never
rewritten … scaffolding dies only with its code."** That audit measured the estate at
6 layers (`unit/`, `integration/`, `contract/`, `property/`, `scenarios/`, `benchmark/`) and
found 74% of test LOC is Python-implementation-coupled scaffolding that *should* die with the
Python engine — not a defect, "the ratio working as intended."

Program 27 Phase 0 collapses that into the three tiers the plan names, which map onto the
2026-07-09 verdict as follows:

| Program 27 tier | 2026-07-09 language | What it means for the Rust port |
| --- | --- | --- |
| **transcribe** | "survives byte-for-byte" / "ports in spirit" | Durable behavioral contract — baselines, property/Hypothesis laws, contract-boundary tests, scenario/emergence tests, and integration tests shaped like a DB-schema or HTTP-contract boundary. Gets re-expressed (proptest/quickcheck, replay harness, contract test) against the Rust engine. |
| **retire-with-code** | "dies, and should" / "dies happily" | Python-implementation-coupled scaffolding: unit tests mirroring one `src/babylon` module 1:1, benchmarks, and test support code (factories/fixtures/mocks/install/scripts/`_helpers`). Retires the moment its src module is ported — no rewrite effort budgeted. |
| **re-derive-as-property-law** | the property layer's own charter, generalized | Tests that pin a **cross-cutting LAW** (conservation, determinism, ordering, enum/hash closure, round-trip identity) rather than one module's behavior. These should become abstract law statements stated once against the Rust engine, not hand-translated line by line — most already live in `tests/property/`, but the same law shape recurs inside `tests/unit/` and `tests/integration/` and is called out below wherever it does. |

## Classification method

Path + mechanical marker only, no per-file reading (1,329 files is out of budget for that; the
tool is the audit). Rule precedence, first match wins:

1. `tests/baselines/**` → **transcribe** (byte-for-byte artifacts per the audit's "Three strata").
2. File greps positive for `hypothesis` or `@given` (56 files, any directory) → **transcribe**.
3. `tests/contract/**`, `tests/property/**`, `tests/scenarios/**` → **transcribe** ("ports in
   spirit" / "ports mechanically" per the audit).
4. `tests/benchmark/**` → **retire-with-code** ("dies happily").
5. Filename matches a cross-cutting-law marker (`determinis|conservation|invariant|closure|
   ordering|thread_cap|constants_sync|round_trip|numeraire`) → **re-derive-as-property-law**.
6. `tests/integration/**` AND filename matches a contract-shape marker (`endpoint|schema|
   postgres|db_|api|serialization|bridge|contract|atomicity|atomic|two_phase|commit`) →
   **transcribe** (the audit's "DB-schema- and HTTP-contract-shaped parts" of `integration/`).
7. Remaining `tests/unit/**` / `tests/integration/**` files → **retire-with-code** (mirror one
   src module; this is the majority case per the audit's 74%-disposable measurement).
8. Everything else — `factories/`, `fixtures/`, `mocks/`, `install/`, `scripts/`, `_helpers/`,
   root `conftest.py`/`__init__.py` — → **retire-with-code** (test support/scaffolding, not
   itself a behavioral contract).

Rules 2, 5, and 6 are the cases where the **top-level directory alone did not decide the
tier** — a marker inside the path or file content did. Those 114 files are the judgment-call
list (below), each with its one-line reason. Every other file (1,215 of 1,329) was decided by
directory alone.

## Overall tally

| tier | count | example paths |
| --- | --- | --- |
| transcribe | 128 | `tests/conftest.py`; `tests/contract/engine/test_systembase_inheritance.py`; `tests/contract/qcew/__init__.py` |
| retire-with-code | 1,168 | `tests/__init__.py`; `tests/_helpers/invariants/melt_consistency.py`; `tests/_helpers/invariants/metamorphic.py` |
| re-derive-as-property-law | 33 | `tests/_helpers/invariants/h3_round_trip.py`; `tests/integration/balkanization/test_audit_round_trip.py`; `tests/integration/economics/test_h3_round_trip.py` |
| **total** | **1,329** | |

128 + 1,168 + 33 = 1,329 — the tiers exhaust the estate (tool asserts this on every run).

The proportions echo the 2026-07-09 audit closely: transcribe+re-derive (161 files, 12.1%)
vs. retire-with-code (1,168 files, 87.9%) — slightly more disposable than the audit's 74%-LOC
figure, which is expected since file count and LOC diverge (many single-assertion unit files
inflate the file count on the retire side) and since 2026-07-09 to 2026-07-29 saw the estate
grow mostly in `tests/unit/` and `tests/integration/` (Programs 15–26).

## Per-directory breakdown

| directory | transcribe | retire-with-code | re-derive-as-property-law | total |
| --- | ---: | ---: | ---: | ---: |
| `(root)` | 2 | 4 | 0 | 6 |
| `_helpers/` | 0 | 11 | 1 | 12 |
| `benchmark/` | 0 | 3 | 0 | 3 |
| `contract/` | 23 | 0 | 0 | 23 |
| `factories/` | 0 | 2 | 0 | 2 |
| `fixtures/` | 0 | 4 | 0 | 4 |
| `integration/` | 26 | 163 | 14 | 203 |
| `mocks/` | 0 | 2 | 0 | 2 |
| `property/` | 64 | 0 | 0 | 64 |
| `scenarios/` | 6 | 0 | 0 | 6 |
| `scripts/` | 0 | 2 | 0 | 2 |
| `unit/` | 7 | 977 | 18 | 1,002 |
| **total** | **128** | **1,168** | **33** | **1,329** |

`contract/`, `property/`, and `scenarios/` land 100% in **transcribe** — consistent with the
2026-07-09 verdict that these are "the best code in the suite by this metric," the assertion
being the spec rather than the harness. `unit/` is 97.5% **retire-with-code** by file count,
confirming the audit's "the bulk of the 8,968 unit funcs... are scaffolding for THIS
materialization." `integration/` is genuinely mixed (26 transcribe / 163 retire / 14 re-derive
out of 203) — the audit's own word for that layer ("mixed").

## Judgment calls (114 files)

These are every file where directory-alone classification (rules 1/3/4/7/8) did **not**
apply — a marker in the file's content (hypothesis usage) or its filename (a law-shape word,
or a DB/HTTP-contract shape word inside `integration/`) decided the tier instead. Full list,
one line each with its assigned tier and reason (grouped by reason for readability; the raw
tool output has the literal per-file table):

### Hypothesis/`@given` usage → transcribe (72 files)

All of `tests/property/**` restated for completeness (64 files: `dialectics/test_composition_
laws.py`, `test_cylinder_laws.py`, `test_galois_laws.py`, `test_intervention_laws.py`,
`test_value_form_invariance.py`, `test_wealth_asymmetry_invariance.py`;
`invariants/test_alpha_smoothing.py`, `test_capital_recurrence.py`, `test_circulation_v.py`,
`test_community_membership_lint.py`, `test_consequence_after_actions.py`,
`test_edge_mode_trajectory.py`, `test_frozen_discipline.py`, `test_h3_hierarchical.py`,
`test_invariant_harness.py`, `test_material_base_ordering.py`,
`test_monetary_anchor_absence.py`, `test_no_db_io_during_tick.py`,
`test_numeraire_invariance.py`, `test_probability_bounds.py`,
`test_proportional_scaling.py`, `test_round_trip_identity.py`,
`test_simplex_pipeline.py`, `test_tick_persistence_monotonic.py`,
`test_value_conservation.py`, `test_wealth_heat_bounds.py`;
`strategies/alpha_coefficient.py`, `capital_stock.py`, `dpd_state.py`,
`edge_mode_evidence.py`, `hex_grid.py`, `multi_tick_sequence.py`, `od_matrix.py`,
`primitives.py`, `probability_field.py`, `test_strategies.py`, `worldstate.py`;
`systems/test_metamorphic.py`; `test_crisis_machinery_weekly_cadence.py`,
`test_geometric_depreciation_inverse.py`, `test_hex_to_county_conservation.py`,
`test_per_stage_conservation.py`; `circulation/test_v_conservation.py`; `conftest.py`) —
these all live in `property/` already so directory alone (rule 3) would have caught them too;
listed here because the tool's marker fired first, not because the tier is in doubt.

Files **outside** `property/` where the hypothesis marker is the *only* reason for transcribe
(the real judgment calls — directory alone would have said retire-with-code): `tests/conftest.
py`, `tests/scenarios/conftest.py`, `tests/scenarios/test_carceral_equilibrium.py` (scenarios/
would also catch it via rule 3), `tests/test_simplex_invariants.py`, `tests/integration/
tensors/test_empirical_validation.py`, `tests/unit/dialectics/test_connectivity_instance.py`,
`tests/unit/dialectics/test_scale.py`, `tests/unit/dialectics/test_value_form.py`, `tests/
unit/engine/test_distribution_split.py`, `tests/unit/engine/test_graph_iteration_order.py`,
`tests/unit/formulas/test_survival_calculus_properties.py`, `tests/unit/reference/bea/
test_accounting_identity_hypothesis.py`.

**Reason:** each of these files states a law via Hypothesis strategies rather than pinning one
call's return value — the Hypothesis `@given`/strategy machinery already IS the property-law
shape Program 27 wants ported, regardless of which directory it happens to sit in.

### Cross-cutting-law filename marker → re-derive-as-property-law (18 files)

`tests/_helpers/invariants/h3_round_trip.py`, `tests/integration/balkanization/
test_audit_round_trip.py`, `test_determinism_replay.py`, `test_seed_coverage_invariant.py`,
`tests/integration/economics/test_h3_round_trip.py`, `tests/integration/engine/
headless_runner/test_shock_determinism.py`, `tests/integration/mvp/test_determinism.py`,
`tests/integration/test_action_determinism.py`, `test_audit_log_round_trip.py`,
`test_baseline_determinism.py`, `test_canada_required_invariant.py`,
`test_circulation_determinism.py`, `test_conservation_audit_strict.py`,
`test_endgame_detection_round_trip.py`, `test_invariant_suite_under_bridge.py`, `tests/unit/
config/test_constants_sync.py`, `test_wealth_distribution_invariants.py`, `tests/unit/
economics/substrate/test_conservation.py`, `test_alpha_weekly_invariant.py`, `tests/unit/
engine/test_determinism_ab.py`, `test_invariants.py`, `test_substrate_system_ordering.py`,
`tests/unit/models/test_world_state_round_trip_spec066.py`, `tests/unit/persistence/
test_circulation_v_conservation_evaluator.py`, `test_conservation_auditor.py`,
`test_fips_mapping_invariant.py`, `test_phi_week_conservation.py`, `tests/unit/reference/
qcew/test_imputation_determinism.py`, `tests/unit/sentinels/test_conservation.py`,
`test_determinism.py`, `tests/unit/test_blas_thread_cap.py`, `tests/unit/tools/
test_regression_construction_cadence_determinism.py`, `tests/unit/topology/
test_graph_algorithms_determinism.py`.

**Reason:** filename pins a cross-cutting LAW (conservation / determinism / round-trip
identity / ordering / enum or thread closure) that must hold across the *whole* engine, not
one module's behavior — a Rust port should state each of these once as an abstract law
(proptest-shaped), not translate the Python assertion line by line.

### `integration/` DB-schema/HTTP-contract shape → transcribe (24 files)

`tests/integration/archive/test_session_persistence_contracts.py`, `balkanization/
test_postgres_persistence.py`, `economics/test_serialization_roundtrip.py`, `engine/
headless_runner/test_bridge_uses_cache.py`, `test_persist_tick_no_db_increment.py`,
`persistence/test_datetime_event_contract.py`, `test_domain_contracts.py`,
`test_per_tick_transaction_atomicity.py`, `test_schema_stamp.py`,
`test_atomicity_inheritance.py`, `test_bridge_income_circuit.py`,
`test_communities_endpoint.py`, `test_db_initialization_queries.py`,
`test_engine_bridge.py`, `test_event_serialization.py`,
`test_lawverian_contradiction_bridge.py`, `test_org_serialization.py`,
`test_persist_tick_atomic.py`, `test_persistence_monotonic_postgres.py`,
`test_postgres_integration.py`, `test_territory_edge_serialization.py`,
`test_tick_commit.py`, `test_timeseries_endpoint.py`, `test_two_phase_initialization.py`,
`test_value_form_bridged.py`.

**Reason:** these are the "DB-schema- and HTTP-contract-shaped parts" of `integration/` the
2026-07-09 audit called out as surviving in spirit — the assertion is "does this row/endpoint/
transaction boundary hold," not "does this Python function return X," so it transcribes to a
Rust-side contract test against the same Postgres schema / HTTP surface.

## What this leaves undecided (honest gaps, not resolved here)

- The `integration/` "mixed" verdict (163 retire-with-code of 203) is the coarsest cut in this
  pass — some of those 163 may deserve `transcribe` on closer reading (e.g. a file that
  exercises one domain module through the full engine stack rather than a boundary). Task 8's
  budget was a mechanical tally, not a per-file read of 203 integration tests; a finer
  integration-layer pass is future work, not blocking Phase 0.
- `tests/unit/**` files matching a law marker only in **content** (not filename) are not
  caught by this tool (rule 5 is filename-only, to keep the run under a minute without an
  `rg` pass per marker). The hypothesis-content check (rule 2) is the one exception — it
  already reads content because that check was cheap (one `rg -l` for the whole tree). A
  content-level law-marker sweep would very likely move a handful more `unit/` files from
  retire-with-code into re-derive-as-property-law; flagged for the Task 11 coverage-floor
  pass, which reads `tests/unit/engine/laws/` gaps individually anyway.
- No file was read in full to verify its classification — this is a path/marker census per
  the task's own scope, not a line-by-line audit. Any single row here can be wrong; the
  purpose is the tier *tallies*, which is what Task 10 (porting contract table) and Task 11
  (coverage-floor backfill) consume next.

## Reproduction

```bash
python3 tools/test_estate_disposition.py
```

Stdlib-only, no venv required. Deterministic given the current `tests/` tree (the only
non-static input is the `rg -l "hypothesis|@given"` grep for rule 2).
