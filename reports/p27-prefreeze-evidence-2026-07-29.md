# Program 27 Phase 0 — Pre-Freeze Evidence: phi_hour Regression + Numeric-Site Traces

Two Task-13-flagged pre-freeze items (the numeric-site → column-family trace and
the Michigan stochastic-family verification), plus the live-regression forensics
that emerged when the scheduled N=32 reference ensemble was attempted. All
raw artifacts: `/media/user/data/babylon-data/p27-ensemble-2026-07-29/`
(manifest, per-seed console logs, `phi_probe.py`, full probe output).

---

## PART 1 — The phi_hour crash is a LIVE REGRESSION on dev, not a latent edge case

### Timeline of evidence

1. **Task 13 pilot** (2026-07-29): seed 3001 crashed ~5 min into a
   `michigan-canada --ticks 520` run with
   `ValidationError: EconomicConditions.phi_hour = -9.8384306576833e-07 < 0`.
   Classified then as seed-dependent (the seed-2010 baseline on file completes
   520 ticks).
2. **N=32 reference ensemble** launched per the Task 13 schedule
   (serial single-flight, seeds 0–31). Seeds 0 and 1 both crashed with the
   **byte-identical value at the identical wall time (294 s)** — tick-deterministic,
   not seed-dependent. Ensemble killed after 3-for-3 crashes (3001, 0, 1).
3. **Decisive experiment**: seed **2010** — the baseline's own seed — re-run on
   current dev (`b0d1a619`). **CRASHED, same value, same timing**
   (`seed_2010_regression_check/`). The checked-in `michigan-e2e.json` baseline
   (last regenerated **2026-07-09**, commit `f528e7d3`) predates the break.
   Every seed crashes; the flagship 520-tick scenario is broken on dev **now**.

### Why every gate stayed green

The crash fires at tick ~52 — the year 2010 → 2011 rollover, when the
imperial-rent pipeline pulls year-2011 data. No CI leg runs a michigan-scope
simulation past tick 52: qa:regression's 11 scenarios and the vault gates are
short or synthetic-scope, `detroit-tri-county-5t`/`storage-budget-5t` are
5 ticks, `nightly.yml` has no michigan e2e leg (verified by inspection), and
`mise run sim:e2e-michigan` is manual-only. A tick-52 michigan-scope crash is
structurally invisible to the current gate estate. **That is itself a Phase-0
coverage finding**: the freeze-tag gate set cannot currently see the flagship
scenario's health.

### Data-level reproduction (no simulation needed)

`phi_probe.py` (preserved beside the artifacts) wires the Spec-057 Leontief
services exactly as `create_leontief_rent_services` does and runs the
decompose → calculate → allocate chain standalone against the reference DB,
in ~30 seconds:

| Year | Counties allocated | Negative counties | Industry-level phi_vector negatives |
| --- | ---: | ---: | --- |
| 2010 | 3,220 | 40 | `113FF` −1,211 · `211` −172,529 · `331` −65,221 |
| 2011 | 3,220 | 50 | `113FF` −2,998 · `211` −232,108 · `331` −109,767 |
| 2012 | 3,220 | 33 | `113FF` −1,606 · `211` −184,210 · `327` −336 · `331` −77,188 |

Year 2011 negatives include **26131 (Ontonagon County, MI) = −9.8384306576833e-07
— the exact crash value** — and 26135 (Oscoda County, MI). Year 2010 has **no
negative Michigan county**, which is why ticks 0–51 (year 2010) pass and the
crash detonates exactly at the rollover.

**The negatives are structural, not float noise.** The industry-level rent
vector carries large negative rents for extraction industries — Oil & Gas
(`211`) at −232k, Primary Metals (`331`) at −110k, Forestry/Fishing (`113FF`)
at −3k (phi-vector units, year 2011). The per-county values are small only
because the allocator divides by county employment-hours. Whether negative
imperial rent for extraction industries is (a) economically meaningful under
MLM-TW unequal-exchange theory (a core industry paying *above* periphery labor
content), (b) a data artifact, or (c) a formula defect is an **ideologically
load-bearing question reserved for the Director** (it is the same territory as
the P26 σ-attribution ⚠ review flag).

### Crash mechanism (three steps, all verified by reading the code)

1. **Origin** — `industry_to_county_allocator.py:248`:
   `phi_hour = county_rent / (total_emp × HOURS_PER_YEAR)`; `county_rent`
   accumulates `phi_vector[i] × employment_share` including the structural
   industry negatives above.
2. **Validation bypass** — `imperial_rent.py:300`:
   `county.model_copy(update={"phi_hour": allocation[fips]})`. Pydantic v2
   `model_copy` **skips validation**, so `CountyEconomicState.phi_hour`'s own
   `ge=0` never fires; the spec-057 §R5 "two-layer axiom enforcement" is
   bypassed at exactly this line. (Scar class: `frozen model +
   model_copy(update=...)` is the repo-standard mutation idiom — **every**
   bounded-scalar constraint is unenforced on the mutation path. The Rust
   kernel scalars close this class structurally; flagged as a candidate
   v3.0.0 rider.)
3. **Detonation** — `tick/system/__init__.py:2398`: the year-rollover
   `EconomicConditions(...)` synthesis is a real validating constructor; at
   tick 52 it validates Ontonagon's poisoned value and aborts the run.

### Regression window and prescribed bisect

- Window: **2026-07-09 → 2026-07-29** (baseline regeneration → confirmed
  crash). The synthesis loop text predates the 2026-07-11 domain refactor;
  import shares come from BEA USE/IMPORT_USE tables (NOT the
  `fact_bilateral_trade_annual` table P26 re-cut 120→540 rows). Prime
  suspects: P26's σ-composition Φ-attribution changes (merged 2026-07-28,
  already ⚠-flagged for Director review) and the Vol I/II economics merges
  (2026-07-21).
- **Bisect method**: `phi_probe.py` reproduces the poisoned allocation from
  reference data in ~30 s with **no simulation** — `git checkout <commit> &&
  python phi_probe.py` over the window dates the break in minutes. (Caveat:
  the reference DB is a build product; commits before the pinned-DB era
  (2026-07-20, `5f1f23fe`) need `mise run data:build-db` under the dataBuild
  shell to reproduce their DB.)

### Disposition (Director)

- **(a) Authorize the allocator clamp** (`max(0.0, phi_hour)` +
  `PhiHourOutlierEvent` kept loud beyond a declared floor) as a second
  Phase-0 authorized fix. One line; restores §R5; drifts the vault
  `imperial_rent_phi` column → requires a baseline/vault ceremony.
- **(b) Accept ENGINE_FAILURE as an outcome band** — now effectively dead:
  with *every* seed crashing there is no reference distribution to build.
- **(c) Rule on the negative industry rents themselves** — the deeper
  question underneath (a): whether the σ-attribution/Leontief math *should*
  produce negative Φ for extraction industries, or the data/formula needs
  correction. (a) unblocks the ensemble either way; (c) decides whether the
  clamped values are theoretically honest.

The **N=32 ensemble is blocked** on this disposition. The freeze tag
(Task 17) should not be cut while the flagship scenario crashes: the frozen
Python reference would pin a broken engine.

---

## PART 2 — Numeric-site → column-family traces and Michigan family verification

The following is the full read-only investigation (file:line-cited) covering
the two items Task 13 explicitly deferred to pre-freeze work: which of the six
audited numeric sites actually feed baseline columns (Item A), and whether the
Michigan/tri-county stochastic families share the canon scenarios' writers
(Item B). Headline: **only the `production_chain_rent.py:144` `np.linalg.inv`
site is live** — the exact site behind Part 1's regression — feeding
`tick_phi_hour` → the vault `imperial_rent_phi` dense column and the LEGISLATE
fiscal calc; four sites are confirmed dead (one a confirmed III.10 retirement
case), one is gated to the interactive session only. Michigan envelope
statistics **must** be built from `county_terminal_snapshot` (fully populated),
never `terminal_state.mean_*` (hardcoded `None` — a code gap in
`runner.py:791-844`, not a data hole).

---


Read-only investigation. All claims below are grounded in direct file reads
(file:line citations), not pattern-matching. Where a claim about "no caller
exists" is made, it is backed by an `rg` search over `src/babylon/` excluding
`tests/`.

---

## ITEM A — numeric-site → column-family forward trace

### Site 1 — `np.linalg.inv`, `inter_industry.py:253` (`DefaultLeontiefComputer.compute_inverse`)

**DEAD END — built but never wired into any production path.**

- `DefaultLeontiefComputer.compute_inverse()` (`src/babylon/domain/economics/tensor_hierarchy/inter_industry.py:229-264`, `np.linalg.inv` at line 253) computes a generic Leontief inverse `L = (I-A)^-1` from a raw `InterIndustryFlow`.
- Searched every production caller: `rg -n "DefaultLeontiefComputer\(\)" src/ tests/` → only 4 hits, all in `tests/integration/economics/test_tensor_hierarchy.py` and `tests/unit/economics/tensor_hierarchy/test_inter_industry.py`. Zero hits in `src/babylon/engine/`, `src/babylon/domain/economics/factory.py`, or `web/`.
- `factory.py::create_leontief_rent_services` (the one place that instantiates `DefaultInterIndustryFlowSource`, line 213) only calls `flow_source.get_industry_codes()` (factory.py:214) — it never calls `DefaultLeontiefComputer().compute_inverse(...)`.
- Verdict: this specific `np.linalg.inv` call site has **zero production reachability**. It carries no cutover tolerance risk because nothing downstream ever executes it outside test fixtures.

### Site 2 — `np.linalg.inv`, `production_chain_rent.py:144` (`ProductionChainDecomposer.decompose`)

**LIVE — feeds `tick_phi_hour` territory column, thence the golden-vault `imperial_rent_phi` dense column and the LEGISLATE fiscal calc.**

Forward chain (each hop verified by reading the file):

1. `ProductionChainDecomposer.decompose()` — `src/babylon/domain/economics/tensor_hierarchy/production_chain_rent.py:117-152`, `np.linalg.inv` at **line 144** — computes `L_d = (I - A_d)^-1`.
2. Wired via `ProductionChainCalculatorBundle` in `domain/economics/factory.py:154-250` (`create_leontief_rent_services`), consumed by both `web/game/engine_bridge.py:7846` and `src/babylon/engine/headless_runner/runner.py:1089`.
3. `babylon.domain.economics.tick.system.imperial_rent.compute()` (`src/babylon/domain/economics/tick/system/imperial_rent.py:45-150`) calls `services.production_chain_calculator.decomposer.decompose(flow, shares)` at line 127, then `.calculator.calculate(...)` at line 128, then `services.industry_county_allocator.allocate(...)` at line 138 → per-county `phi_hour`.
4. `_apply_allocation()` (imperial_rent.py:288-303) writes `phi_hour` onto `CountyEconomicState` via `model_copy`.
5. `src/babylon/domain/economics/tick/graph_bridge.py:178` — `graph.update_node(node_id, ..., tick_phi_hour=county.phi_hour, ...)` — **the graph node attribute write** (territory node, condition (a) satisfied).
6. Downstream consumers of `tick_phi_hour`:
   - `src/babylon/domain/economics/tick/derived_rates.py:108` — `Phi_aggregate = sum(phi_hour * employment * ANNUAL_HOURS_PER_WORKER)`.
   - `src/babylon/engine/systems/policy.py:260` — `phi_raw = attrs.get("tick_phi_hour")`, feeding `Φ_inflow` for the LEGISLATE fiscal resolver.
   - `src/babylon/projection/county.py:183` and `src/babylon/projection/state.py:354` — `imperial_rent_phi=attrs.get("tick_phi_hour")` / `_sum_territory_attribute(territories, "tick_phi_hour")` — the **golden-vault dense `imperial_rent_phi` column**, per-county and state-level.
   - Asserted live by `src/babylon/sentinels/seam/checks.py:206` and `registry.py:98,1895` (`tick_phi_hour` graph-write sentinel).

Verdict: **imperial_rent_phi (county + state dense golden columns) inherits LAPACK cross-language tolerance risk from this site.** Not gated — this path runs in every canonical scenario that wires `create_leontief_rent_services` (both headless_runner and the interactive session).

### Site 3 — `np.linalg.eig`, `class_transition.py:74` (`DefaultClassTransitionComputer.stationary_distribution`)

**DEAD END — gated behind an undelivered constitutional amendment (US5), and has zero callers regardless.**

- `DefaultClassTransitionSource` (`src/babylon/domain/economics/tensor_hierarchy/class_transition.py:176-210`) is a **permanent stub**: `get_transition_matrix()` and `get_stationary_distribution()` both unconditionally `return NoDataSentinel(..., _STUB_REASON)` where `_STUB_REASON = "PSID data source pending constitutional amendment (US5 deferred loader)"` (line 34).
- `rg -n "DefaultClassTransitionComputer|stationary_distribution" src/babylon --type py -g '!tests*'` finds **zero callers** anywhere outside the module's own class body, `tensor_hierarchy/__init__.py`'s re-export, and `protocols.py`'s docstring. No `factory.py` wiring, no engine system import.
- Verdict: even if the PSID loader existed, nothing calls `stationary_distribution()`. Double-dead: (1) no real data source ever supplies a non-stub `ClassTransitionMatrix`, and (2) no production code calls the computer at all. Gate: constitutional amendment US5 (never ratified).

### Site 4 — `scipy.optimize.linprog`, `curvature.py:225` (`_wasserstein_1`, called from `compute_ollivier_ricci`)

**DEAD END — confirmed unreachable from any system, strengthening the audit's "likely-(c) retirement" flag into a certainty.**

- `compute_ollivier_ricci()` (`src/babylon/formulas/curvature.py:32-92`) is exported from `src/babylon/formulas/__init__.py:78,247`.
- `rg -n "compute_ollivier_ricci" src/babylon --type py` → the **only** hits are the definition and the `formulas/__init__.py` import/`__all__` entry. Zero callers in `engine/systems/*.py` or `domain/dialectics/*` (checked `ContradictionFieldSystem`/`FieldDerivativeSystem`, rows 29/30 of the porting table — neither imports curvature.py).
- The one plausible consumer, `persist_contradiction_fields(tick, fields, curvatures, session_id=...)` (`src/babylon/persistence/protocols.py:332`, impl at `postgres_runtime/_legacy.py:746`, writing the `edge_curvature` Postgres table, `postgres_schema.py:332-337`), has exactly one production caller: `WorldStateBridge._persist_opposition_fields` (`src/babylon/engine/headless_runner/bridge.py:582-631`) — and it calls `self._runtime.persist_contradiction_fields(tick, fields, [], session_id=...)` at **line 631** with a **hardcoded empty list literal** for the `curvatures` parameter.
- Verdict: the numeric-closure audit's own report (`reports/numeric-closure-audit-2026-07-29.md` row 4) flagged this "UNVERIFIED — needs an explicit call-site trace before Director sign-off." That trace is now done: **zero production callers**, and the one DB sink this curvature would feed is permanently starved by an empty-list literal. This is stronger evidence for (c) retirement under III.10 than the audit had — not just "unreachable from run_tick," but genuinely orphaned code with no live consumer anywhere.

### Site 5 — `scipy.sparse`, `domain/economics/substrate/circulation.py:27` (`DefaultHexCirculationComputer`)

**DEAD END — a distinct, unwired sibling of the module that actually does this job (site 6). The audit's row-5 reasoning did not check callers and assumed liveness.**

- `DefaultHexCirculationComputer.build_od_matrix()` / `.circulate_wages()` (`src/babylon/domain/economics/substrate/circulation.py:40-215`) is the module the spec named. Its own docstring (lines 11-14) is the source of the "~1e-9 accumulation" language the audit report quoted.
- `rg -n "DefaultHexCirculationComputer|circulate_wages|build_od_matrix" src/babylon --type py -g '!tests*'` → only the class body, `substrate/protocols.py` docstring, and `substrate/__init__.py` re-export. **Zero callers.**
- Verified this is NOT the same code the live Vol II circulation pipeline uses: `engine/systems/substrate.py` (`SubstrateSystem`) imports only `formulas.metabolic_rift` — no `substrate.circulation` import. The live per-tick commuter-flow redistribution is a **separate module**, `engine/systems/vol2_circulation.py` + `domain/economics/lodes_commute_matrix.py` (site 6), using a different type (`LODESYearMatrix`, not this module's `sparse.csr_matrix` OD builder), a different data source (LODES-loader-backed, not `CommuterFlowSource`), and different graph-write semantics (county `Territory.v`, not `HexEconomicState.variable_capital`).
- Verdict: this exact spec-named site is dead code. The live, tick-reachable analogue is site 6 below (which is itself gated — see there).

### Site 6 — `scipy.sparse`, `lodes_commute_matrix.py:42` (`LODESYearMatrix`/`LODESCommuteMatrixLoader`)

**LIVE production write path exists (`Territory.v` graph attribute), but it is GATED OFF in every byte-identical baseline/CI harness — only reachable via the interactive play-session path.**

Forward chain:

1. `LODESYearMatrix` (`src/babylon/domain/economics/lodes_commute_matrix.py:58-100`, `sp.issparse`/CSR validation) is built by `LODESCommuteMatrixLoader.load_year()`.
2. `Vol2CirculationStep.step()` (`src/babylon/engine/systems/vol2_circulation.py:146-356`) reads it (`self._od_loader.load_year(simulated_year)`, line 187), does the sparse mat-vec `year_matrix.matrix.T @ normalized` (line 235), and **writes** `protocol.update_node(fips_to_node[fips], v=v_post_val)` at **line 344** — a genuine county `Territory` graph-node attribute write (condition (a)).
3. This step is invoked from `ImperialRentSystem._invoke_vol2_circulation_if_wired()` (`src/babylon/engine/systems/economic.py:158-199`), called unconditionally from `economic.py:86` — **but** it silently no-ops (line 179-180) unless FOUR `context.persistent_data` keys are all present: `vol2_step`, `boundary_flow_register`, `session_id`, `simulated_year`.
4. The gate registry documents the supplier explicitly (`src/babylon/sentinels/seam_algebra/registry.py:415-459`, `GatedInput(name="vol2_circulation_vol2_step", ..., supplier_files=("src/babylon/game/session.py",))`): **only `GameSession.advance_tick`** (`src/babylon/game/session.py:1451`, the interactive `babylon play` / TUI-client path) stamps `context["vol2_step"]`. The comment states explicitly: *"the headless runner remains unwired by design, tracked in ai/wiring-doctrine.md."*
5. Verified the canonical/CI harnesses never touch this: `tools/regression_test.py:75` imports `from babylon.engine.simulation_engine import step` and calls it directly — no `GameSession`, no bridge. `tools/vault_regression.py:128-130` uses `babylon.engine.headless_runner.runner.run` — also not `GameSession`. Neither path ever populates `context["vol2_step"]`.

Verdict: **for all 11 canonical qa:regression scenarios (in-memory, byte-identical gate) and both `qa:vault-regression` legs (headless_runner-backed), this numeric site is dormant** — `_invoke_vol2_circulation_if_wired` early-returns every tick. It only executes during an interactive `babylon play` session, which carries no byte-identical baseline obligation. Gate: `context.get("vol2_step") is None` (economic.py:179), supplied only by `game/session.py`.

### Item A summary table

| # | Site | Live in any baseline/CI gate? | Feeds | Risk |
|---|---|---|---|---|
| 1 | `inter_industry.py:253` (`DefaultLeontiefComputer`) | No — zero production callers | nothing | none |
| 2 | `production_chain_rent.py:144` | **Yes**, unconditional | `tick_phi_hour` → `imperial_rent_phi` dense golden column (county+state), `Φ_inflow` fiscal calc | **HIGH — real cutover risk** |
| 3 | `class_transition.py:74` | No — stub source (US5) + zero callers | nothing | none |
| 4 | `curvature.py:225` (linprog) | No — zero callers, DB sink starved by literal `[]` | nothing | none (retirement candidate, confirmed) |
| 5 | `substrate/circulation.py:27` (`DefaultHexCirculationComputer`) | No — zero callers (distinct dead sibling of #6) | nothing | none |
| 6 | `lodes_commute_matrix.py:42` | Only via `GameSession.advance_tick` (interactive play) — **gated OFF in every qa:regression / qa:vault-regression harness** | `Territory.v` (only when wired) | none for the freeze-gated baselines; real risk only if/when the interactive session ever gets a byte-identical obligation |

**Only site 2 carries live cutover tolerance risk against the current freeze-tag baseline set.** Sites 1, 3, 4, 5 are genuinely dead. Site 6 has a live mechanism but it is provably unreachable from every scenario/CI/vault harness that currently defines "byte-identical."

---

## ITEM B — stochastic-family classification verification (Michigan / tri-county)

### Writers, per family

**`p_acquiescence` / `p_revolution`**

- Primary writer: `SurvivalSystem.step()` — `src/babylon/engine/systems/survival.py:84-165`. Iterates `graph.query_nodes()` skipping only `node_type == "territory"` (line 118) — **no scope/scenario branch**. Computes `calculate_acquiescence_probability`/`calculate_revolution_probability` from the formula registry (deterministic sigmoids) and writes `graph.update_node(node.id, p_acquiescence=p_acq, p_revolution=p_rev)` at **line 165**. **No RNG** (confirmed: no `resolve_rng`/`rng.` calls anywhere in the file).
- Overridden during a rupture event by `StruggleSystem.step()` — `src/babylon/engine/systems/struggle.py:278-343+`. Same generic node loop (skips only territory, line 316), filters to `PERIPHERY_PROLETARIAT`/`LUMPENPROLETARIAT` roles. **Does** consume the tick RNG: `rng = resolve_rng(services, tick)` at **line 299**, then `spark_occurred = rng.random() < spark_probability` at **line 343**; a full uprising sets `graph.update_node(p_w_id, p_revolution=1.0, ideology=new_ideology)` (line 608) and a fascist-boost path sets a new `p_acquiescence` (line 658). `resolve_rng` (`src/babylon/kernel/system_base.py:35`) derives a `random.Random` from the tick-seeded stream — deterministic given the run seed, not process entropy.
- **No scope-gated branch found in either file.** Both loops are generic over `graph.query_nodes()`, so whatever entities the scenario seeds (canon SocialClass nodes, or Michigan/tri-county's county-scoped SocialClass nodes) go through identical code.
- **Verdict: CONFIRMED SAME MECHANISM.** The RNG-taint classification (StruggleSystem can override these two fields via a seeded roll) applies identically to canon scenarios and to Michigan/tri-county, contingent only on the scenario seeding entities with a struggling role — which both do.

**`ideology_r` / `ideology_l` / `ideology_f`**

- These are **not** direct per-node graph attributes. The per-node substrate is `class_consciousness` / `national_identity`, written by the ideology/consciousness step — `src/babylon/engine/systems/ideology.py:115-424` — via `route_agitation_to_ternary(...)` (deterministic, Shannon-entropy-style ternary router, **no RNG** — confirmed no `resolve_rng`/`rng.` in the file) and `graph.update_node(node.id, ideology={"class_consciousness": new_class, "national_identity": new_nation, "agitation": new_agitation}, ...)` at **line 418-424**. Same generic per-node loop, no scope branch.
- The `(r, l, f)` simplex is derived from `(class_consciousness, national_identity)` by `_ideology_to_ternary()` — `src/babylon/projection/aggregation.py:50-98` — a pure deterministic formula (`r = cc*(1-ni)`, `f = ni*(1-cc)`, `l = 1-r-f`). **No RNG.**
- For county-scoped/Postgres-backed runs (Michigan-e2e, and the Wayne/tri-county "runner-backed detroit leg" the `qa:vault-regression` dev task description names), this per-node value is **additionally rolled up** by `aggregate_consciousness_for_county(world, county_fips)` (`projection/aggregation.py:154+`) — a population-weighted mean over every entity sharing that `county_fips` — called from `WorldStateBridge._derive_subsystem_rows_for_county()` (`src/babylon/engine/headless_runner/bridge.py:950-983`, county aggregation at line 970), which persists the county-level `ideology_r/l/f` into the `dynamic_consciousness_state` Postgres table.
- **The canon in-memory qa:regression scenarios never execute this aggregation layer at all** — `tools/regression_test.py` calls `babylon.engine.simulation_engine.step()` directly (no `WorldStateBridge`, no Postgres), so there is no SQL column and no `aggregate_consciousness_for_county` call in that path; ideology_r/l/f as a *persisted column* simply doesn't exist for those runs.
- **Verdict: CONFIRMED SAME per-node writer** (ideology.py's ternary router, no RNG) feeds both scopes identically. **The aggregation/persistence layer is genuinely DIFFERENT** — not a different physics mechanism, but an extra population-weighted rollup (`aggregate_consciousness_for_county`) that only exists on the headless_runner+Postgres bridge path (Michigan-e2e, and the dev-only Wayne/detroit Postgres leg), and is entirely absent from the in-memory byte-identical qa:regression scenarios. Task 13's "same mechanism by attribute-name correspondence" assumption is right about the underlying physics writer, but incomplete about the aggregation step — worth stating explicitly since it's a real asymmetry, even though it doesn't change the RNG-taint classification (the aggregation itself is deterministic).

### Why `mean_p_acquiescence` / `mean_p_revolution` / `mean_ideology_*` / `total_population` are null in `tests/baselines/michigan-e2e.json`'s `terminal_state`

Root cause located precisely: `_query_terminal_aggregates()` — `src/babylon/engine/headless_runner/runner.py:791-844`.

The function's SQL query (lines 809-817) only computes:
```sql
SELECT COUNT(*) FILTER (WHERE v > 0), SUM(v), SUM(c), SUM(s), SUM(k),
       COUNT(*) FILTER (WHERE ideology_r IS NOT NULL),
       COUNT(DISTINCT entity_id) FILTER (WHERE entity_id IS NOT NULL),
       COUNT(*) FILTER (WHERE entity_id IS NULL)
FROM view_runtime_trace_emission WHERE session_id = %s AND tick = %s
```
It never selects `AVG(p_acquiescence)`, `AVG(p_revolution)`, `AVG(ideology_r/l/f)`, or `SUM(population)`. The return dict (lines 830-844) hardcodes:
```python
"total_population": None,
...
"mean_p_acquiescence": None,
"mean_p_revolution": None,
"mean_ideology_r": None,
"mean_ideology_l": None,
"mean_ideology_f": None,
```
These are **literal Python `None` constants, unconditional** — not a data-availability fallback. This is confirmed **not** a data problem: the sibling function `_county_terminal_snapshot()` (runner.py:847-924) queries the same `view_runtime_trace_emission` view for `p_acquiescence, p_revolution, ideology_r, ideology_l, ideology_f, population` **per entity_id**, and the actual baseline file has these fully populated for all 83 Michigan counties with zero nulls (verified programmatically — `python3` null-count sweep over `county_terminal_snapshot` returned 0 nulls for every one of `entity_id, v, c, s, k, p_acquiescence, p_revolution, ideology_r, ideology_l, ideology_f, population, delta_k_vs_initial` across all 83 rows). Example row (FIPS 26001):
```json
{"entity_id": "26001", "v": 850573.87, "c": 2104243.42, "s": 2652560.75, "k": 546488999.9999939,
 "p_acquiescence": 1.0, "p_revolution": 0.31034510344827587,
 "ideology_r": 0.05, "ideology_l": 0.5, "ideology_f": 0.45,
 "population": 14888, "delta_k_vs_initial": 0.0}
```

**Conclusion (matches the task's suspicion): the ensemble's envelope families for Michigan/tri-county scope MUST be built from `county_terminal_snapshot`'s per-county values (client-side population-weighted rollup), never from `terminal_state.mean_*`, which is a permanently-null placeholder in this code path — not a scope-specific data hole.** `run_metadata.scope_name` confirms this baseline is `"michigan-canada"`, 83 county FIPS + 1 external node (`"canada"`), seed 2010, 520 ticks completed.

### Per-family classification (final)

| Family | Writer | RNG? | Michigan/tri-county vs canon | Classification |
|---|---|---|---|---|
| `p_acquiescence`, `p_revolution` (base) | `SurvivalSystem.step()` (survival.py:165) | No | Identical generic node loop, no scope branch | CONFIRMED SAME MECHANISM |
| `p_acquiescence`, `p_revolution` (rupture override) | `StruggleSystem.step()` (struggle.py:299,343,608,658) | **Yes** — `resolve_rng(services, tick)` | Identical generic node loop, no scope branch | CONFIRMED SAME MECHANISM (RNG-tainted equally in both) |
| `ideology_r/l/f` (per-node substrate: `class_consciousness`/`national_identity`) | `ideology.py` ternary router (ideology.py:394-424) | No | Identical generic node loop, no scope branch | CONFIRMED SAME MECHANISM |
| `ideology_r/l/f` (persisted/aggregated form) | `aggregate_consciousness_for_county` (aggregation.py:154) via `WorldStateBridge` (bridge.py:970) | No (deterministic weighted mean) | **Extra aggregation/persistence layer that ONLY exists on the headless_runner+Postgres bridge path** (Michigan-e2e, dev-only Wayne/detroit Postgres leg) — absent entirely from in-memory qa:regression scenarios | DIFFERENT LAYER, same underlying deterministic writer (worth flagging, doesn't change RNG-taint status) |
| `terminal_state.mean_*` / `total_population` | `_query_terminal_aggregates()` (runner.py:791-844) | N/A | Hardcoded `None` for both scopes — not scope-specific | NOT WRITTEN (code gap, not a data gap) |

### What IS populated per county in `county_terminal_snapshot`

Full field list, all non-null across all 83 Michigan counties in the baseline: `entity_id` (FIPS), `v`, `c`, `s`, `k` (Vol I/II/III circuit stocks), `p_acquiescence`, `p_revolution`, `ideology_r`, `ideology_l`, `ideology_f`, `population`, `delta_k_vs_initial`. This is the correct source for any envelope/ensemble statistic over the RNG-tainted families at Michigan scope.
