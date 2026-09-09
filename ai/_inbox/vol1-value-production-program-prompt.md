# PROGRAM PROMPT — Volume I: The Value Production Engine

**Authored:** 2026-07-20 by the queue controller session, at `feature/queue-2026-07-19` @
`4c812d96`, from a dedicated read-only reconnaissance sweep. **Every file:line anchor below was
verified on that tree — re-verify at program start; lines drift.** Companion prompt:
`ai/_inbox/vol2-circulation-engine-program-prompt.md` (Vol II). §10 defines the protocol for
running the two programs IN PARALLEL as one workflow (owner directive 2026-07-20).

**Mission (the Vol III program's own words, adapted):** unify the TWO disconnected Volume I
estates — spec-021's dormant graph-node systems (reserve army, dispossession) and the live but
web-only county-tick `create_vol1_services` path — into one honest value-production layer;
compute the Fundamental Theorem FOR REAL (the named `W_c vs V_c` comparison is dead code today
while all its ingredients are live); wire the fully-built working-day lever into consciousness;
ground the accumulation loop (organic composition → mechanization displacement → reserve army);
bind the missing Vol I oppositions; and ship a sentinel for every error class this investigation
discovered. Wiring, patching, monitoring — the same rigor pass Volumes III and II get.

Run this as a full program: brainstorm → spec (owner review) → plan (U-chain, per-unit review) →
subagent-driven execution → one declared ceremony. Mirror the Vol III conventions (§7).

---

## 1. Sequencing preconditions (HARD GATES)

1. The 2026-07-19 queue PR (`feature/queue-2026-07-19`: #39 county lattice + #42 consciousness
   wiring) has merged. Vol I builds on Amendment U's county-keyed territories and the #42
   ideology.py changes (§9).
2. The qa:regression modernization branch (`refactor/qa-regression-modernization`) has merged —
   Vol I moves county financial/production channels, exactly the coverage class the old gate was
   blind to. Neither volume develops under the blind gate.
3. The Vol III program has merged — **satisfied** (PR #216 → dev `f59d5852`, ADR089).
4. Parquet cutover (#46) is a **soft** gate for Vol I (unlike Vol II, where it is hard): no
   drive-only data blocks the core program (§3). It hardens only if the spec revives spec-021's
   county-level BLS/Eviction-Lab/foreclosure ambition — that data would be new drive/API scope
   and MUST enter as a parquet-pipeline citizen (CI-no-drive standing rule).

ADR numbering: ADR091 (Amendment U) and ADR092 (county seed artifact) exist on the queue branch (renumbered post-merge; dev's ADR090 = qa modernization);
check `ai/decisions/index.yaml` for the true next free number at authoring time (≥ 092).

## 2. The estate — what exists, and exactly how it is disconnected (verified)

### 2a. spec-021's own fate (the prior program, half-landed)
`specs/021-capital-volume-i/` (identical in both trees) specced Reserve Army (US1),
Dispossession (US2), Working Day (US3), cross-system integration (US4), and 5 federal data
loaders (US5). Its own 2026-07-08 audit annotations in `tasks.md:19-191` are honest and
re-verified true today: type layer + calculators shipped; **US5 loaders: zero built** (all five
package dirs absent; superseded by the babylon-data 086/097/098 territory per `tasks.md:143`);
US4 integration ⅓ done (wage_pressure→median_wage wired; dispossession→class-transitions NOT;
working-day→consciousness NOT). Subsumption of Labor and Concentration/Centralization were
explicitly deferred at `spec.md:220-221` — keep them deferred (follow-up program).

### 2b. Graph-node path — DORMANT BY DOCUMENTED DESIGN
- `ReserveArmySystem` (`engine/systems/reserve_army.py:67-72`, System #5) reads `reserve_ratio`
  off territory nodes, early-continues `<= 0.0`. `DispossessionEventSystem`
  (`engine/systems/dispossession_events.py:69-74`, System #10) reads
  `foreclosure_rate`/`eviction_rate`/`displacement_rate`, early-continues if all `<= 0.0`.
- The field docstring admits it (`models/entities/territory.py:207-211`): *"Zero defaults keep
  both systems inert unless a scenario seeds them."* **No scenario ever seeds them** — zero hits
  for any of the four attrs across `engine/scenarios/`, and `DispossessionEventSystem` has zero
  production producers of ANY of its three gating inputs repo-wide. Textbook **gated dormancy**
  (the exact error class Vol II's prompt §6.1 specs a sentinel for — build it ONCE, §6/§10).
- Partial rescue, reserve army only: `MarketScissors._swell_reserve_army`
  (`engine/systems/market_scissors.py:439-461`, invoked `:384-385`) bumps `reserve_ratio` during
  a price/value crisis — a real emergent activation path. Dispossession has no equivalent;
  it is unconditionally dead, and the seam registry documents its own deadness
  (`sentinels/seam/registry.py:420,1168` — the `dispossession_intensity` rows).

### 2c. County-tick path (`create_vol1_services`) — LIVE IN WEB ONLY
- `factory.py:672-794` builds three adapters: `_FredReserveArmyAdapter` (UNRATE+NROU),
  `_FredProductivityAdapter` (OPHNFB+HOANBS), `_FredDispossessionAdapter` (hardcoded 2007-2020,
  UNRATE-proxy 2021+ — an honesty gap to fix or loudly document). National-level series applied
  to every county — not county-grain.
- Genuinely consumed: `_apply_wage_pressure` (`domain/economics/tick/system/__init__.py:1191-1204`)
  compresses `median_wage` via the reserve ratio — live. `_simulate_transitions` (`:2010-2062`)
  overrides foreclosure/bankruptcy/eviction defaults for `DefaultClassTransitionEngine` — but its
  own `services.transition_engine is not None` gate is **never satisfied by any production
  caller** (a second gated-dormancy instance). `services.productivity_data_source` has **zero
  readers anywhere** (self-documented at `engine_bridge.py:8102-8104`).
- **Web/headless parity gap, mirrors Vol II §2d exactly:** `web/game/engine_bridge.py:8113-8123`
  wires `create_vol1_services`; the canonical `headless_runner/runner.py:911-1049`
  (`_build_economics_overrides`) never calls it — the canonical run computes less Vol I
  economics than a web session. (A third, mostly-superseded wiring site:
  `engine/simulation/_legacy.py:279-317`.) Seam registry books it as `DECLARED_CONDITIONAL`
  (`registry.py:561-580,725-745`). Close the gap (no-compromise default) or get an explicit
  owner ruling that the asymmetry is deliberate. **§10: this parity fix is ONE shared unit with
  Vol II — whichever lane reaches it first lands BOTH `create_vol1_services` and
  `create_circulation_services` in the runner.**

### 2d. V_c — the Fundamental Theorem's comparison is dead code; its ingredients are live
- LIVE: `ValueTensor4x3` (`domain/economics/tensor.py:214-241,404,522` — organic composition
  c/v, exploitation rate s/v, Fortunati variant), QCEW-hydrated. Consumed in the tick:
  `production.py:161-169` (`tensor.total_v` → `effective_labor_power` → produced_value → wealth)
  and the Vol III interest ceiling (`tick/system/__init__.py:1852-1903` reads
  `tensor.profit_rate`/`total_s`; docstring `:1547-1556` records why s/v was deliberately not
  used as the profit rate there).
- DEAD: the named theorem formulas. `formulas/fundamental_theorem.py:16-59`
  (`calculate_labor_aristocracy_ratio`, `is_labor_aristocracy`, `calculate_consciousness_drift`)
  are formula-registry-registered (`formula_registry.py:92-93`) but have **zero call sites**
  outside registration/tests. The Lawverian successor `phi_class(w_c, v_c)`
  (`domain/dialectics/instances/value_form.py:191-213`, literally `(W_c − V_c)/V_c`) — **also
  zero callers.** Nothing a player or narrator can point to as "the Fundamental Theorem,
  computed."
- ALIVE but reference-only: `view_imperial_rent` / `view_surplus_value`
  (`data-catalog.yaml:1396-1409,1423-1437` over `fact_productivity_annual`, 17,336 BLS rows,
  1988-2024) literally compute `Φ = W_c − V_c` and the rate of exploitation from real data —
  consumed only by `tests/unit/reference/test_marxian_views.py` and guarded by
  `sentinels/coverage/db_probe.py`. Historical/calibration truth, never read at simulation
  runtime. This is the program's empirical-validation surface (Constitution III.12 redundant
  verification): sim-produced Φ should be calibratable against these views.

### 2e. Working day — fully built, fully disconnected (not even gated)
`WorkingDayDefines` (`config/defines/economy_labor.py:120-`, assembled at
`_assembler.py:167,320`) + `DefaultWorkingDayClassifier`
(`domain/economics/working_day/classifier.py:14`, ABSOLUTE_DOMINANT/RELATIVE_DOMINANT/MIXED +
`compute_visibility_modifier`): **zero callers outside its own package.** `ConsciousnessSystem`'s
`exploitation_visibility` (`engine/systems/ideology.py:357` via `compute_exploitation_visibility`)
is computed from `wage_change`/`repression_faced`, never from `visibility_modifier` — spec-021
T063's finding, still true. Ch. 10's lever (the length/intensity of the working day shaping how
visible exploitation is) exists as code and touches nothing.

### 2f. Wages — the ONE live W_c-vs-V_c instance; REUSE, don't rebuild
`economic.py:494-507` stamps `w_paid`/`v_produced`/`value_flow` on WAGES edges (Phase D4);
the `wage` opposition (`catalog.py:443`, docstring `:19-24`: *"the imperial bribe (Fundamental
Theorem `W_c > V_c`)"*) consumes those pairs and is genuinely live in the tick — verified against
a 30-tick bridged probe (2026-07-02). `ideology.py:240` independently sums the same edges for the
bifurcation mechanic. **The wage opposition is existing Fundamental-Theorem infrastructure; U2
builds on it rather than creating a parallel Φ computation.**

### 2g. Accumulation — the theory's causal chain is unimplemented
Ch. 25's law (rising organic composition → mechanization displaces workers → reserve army grows)
exists at both ends and not in the middle: `organic_composition` is real and QCEW-populated
(`tensor.py:214-225`) but **no Vol I System reads it**; `ReserveArmyDynamics
.mechanization_displacement`/`firm_failure_displacement` (`domain/economics/reserve_army/types.py:50-`)
have models but **no production writer**. `ReserveArmySystem` reads a scalar someone else must
seed (§2b). The accumulation loop is the program's core new mechanics work.

### 2h. Deliberately out of scope (record, don't build)
- Subsumption of Labor; Concentration/Centralization of Capital (`spec.md:220-221` deferral —
  O*NET/Census BDS/SEC data; follow-up program).
- The `domain/economics/substrate/` exploitation-rate/OCC family (Feature 026) is dormant with
  the whole hex substrate (no production code stamps `hex` nodes,
  `sentinels/vocabulary/registry.py:90`) — do NOT route Vol I mechanics through `substrate/*`;
  county-tick and graph-node paths only. Cross-reference task #57 (Wayne hex-substrate
  migration) before touching anything substrate-adjacent.
- International trade (Ch. 22 National Differences of Wages notwithstanding) — deferred behind
  BOTH volumes per `project/research/trade-after-capital-refactors.md`.

## 3. Data path (deterministic-artifact rule applies)
- QCEW (`fact_qcew_annual` + rollup, `data-catalog.yaml:1290-1316`) is the live wage/employment
  substrate feeding `ValueTensor4x3` — already resident, no new ingestion for core scope.
- FRED national series (UNRATE, NROU, OPHNFB, HOANBS) feed the three §2c adapters — already
  catalogued. The `_FredDispossessionAdapter` post-2020 UNRATE-proxy is the honesty gap: either
  acquire real post-2020 foreclosure/eviction data (parquet-pipeline citizen; hard-gates on #46)
  or make the proxy loud (NoDataSentinel/derivation note), never silent.
- `fact_productivity_annual` + the two Marxian views (§2d) = calibration truth for U2.
- **No drive-only data blocks the core program** — the contrast with Vol II's LODES gate is
  deliberate and is what makes the parallel start (§10) safe for Vol I immediately after gates
  1-2.

## 4. Dialectics binding surface
`catalog.py` today: **ten** bound oppositions (`capital_labor:431, wage:443, tenancy:456,
atomization:466, imperial:476, price_value:490, surplus_distribution:510, debt_spiral:526,
credit:541, financial:557`), `GraphInputs` at 12 fields (`:60-181`), `_DEFAULT_COUPLINGS`
(`:582-611`) reserving exactly TWO dead slots — **both Vol II's** (`circulation→realization`,
`reproduction→disproportionality`). **Vol I has NO reserved slots**: it must both bind new
oppositions AND author brand-new `Coupling(...)` entries — strictly more design surface than
Vol II, and the #1 collision risk with a concurrent Vol II (§10 resolves this with the contract
commit).

Candidate new oppositions (spec phase owns the final cut, Aleksandrov-grounded each):
`value_usevalue` (Ch. 1), `labor_laborpower` (Ch. 6 — the wage-form mystification),
`absolute_relative_surplus` (Chs. 10/12/15 — working-day vs mechanization as the two surplus
strategies; natural consumer of §2e's classifier). Mechanics to mirror: shadow-first
`BoundOpposition(..., shadow=True)` with the `price_value` ADR077→ADR078 promotion precedent;
`GraphInputs` fields as `float | None = None` following the `market_balance` pattern
(`contradiction.py:284-290`); measures from ratios with natural zero points; every coefficient a
define.

## 5. Defines
Vol I is the one volume whose coefficients ALREADY have top-level categories:
`ReserveArmyDefines`/`DispossessionDefines`/`WorkingDayDefines`
(`config/defines/economy_labor.py:12,55,120`, assembled `_assembler.py:165-167,318-320`).
Recommendation (spec phase confirms): **extend those three in place** for their mechanics; put
genuinely NEW Vol I mechanics (theorem computation, accumulation-loop coefficients, opposition
scales) in a new `capital_vol1.py` → `CapitalVolumeIDefines` for parity with the
`capital_vol3.py` convention. House rules: frozen BaseModel, every field docstring names its
exact consumer `file:function`, `gt=0.0` for live divisors, NO field lands unread. Regenerate
`defines.yaml`; `test_constants_sync.py` guards.

## 6. Sentinel units (standing rule: every discovered class ships a gate, mutation-validated)
1. **Gated dormancy** — §2b/§2c found three live instances (`ReserveArmySystem` seeds,
   `DispossessionEventSystem` rates, `transition_engine` gate). This is the SAME class Vol II's
   prompt §6.1 specs (`vol2_step`, LODES kwargs). **Build the registry-of-runtime-gates sentinel
   ONCE across both programs** (§10 assigns the builder; the other program contributes its
   instances as registry rows).
2. **Dead-formula registration** — a formula registered in `formula_registry.py` (hot-swappable
   surface, implied API) with zero production invocations (`labor_aristocracy_ratio`,
   `consciousness_drift`; `phi_class`). The `inert` family sees the registration as a satisfied
   reference. Gate: every registry-listed formula names its production call site or carries a
   cited exemption.
3. On lighting the graph-node path (U3), retire/replace the `dispossession_intensity`
   dead-observability seam rows (`registry.py:420,1168`) — the exemption must not outlive the
   defect. Same discipline as Vol II §6.3.
4. Reuse, don't rebuild: the defines-passthrough sentinel (#42 fix wave) and the vocabulary
   attribute-shape rules already gate the classes Vol I's new node attrs/formula calls would
   otherwise reintroduce.

## 7. Program shape (mirror the Vol III plan conventions)
Model: `docs/superpowers/plans/2026-07-18-vol3-money-scissors.md`. Adopt: U-numbered units with
per-unit spec-compliance review; a **binding interface contract block** at the top of the plan
pinning exact names (opposition keys, GraphInputs fields, node attrs, defines category, ADR
numbers verified free) BEFORE any code — in the parallel mode this block is SHARED with Vol II
(§10); house-rules block (TDD red phase, determinism, zero inline coefficients, layering,
honest absence); plan-splice amendments; ONE declared ceremony at the end.

Suggested unit skeleton (spec phase owns the final cut):
- **U1 Activation audit** — re-verify this §2 on the then-current dev; owner design questions:
  (a) graph-node systems: seed the gates from real data, or DERIVE them (reserve_ratio from the
  accumulation loop, dispossession rates from the county-tick adapters) — recommend derive
  (Aleksandrov: a seeded scalar with no producer is a fabricated input); (b) defines split (§5);
  (c) parity ruling if not already closed by Vol II's lane.
- **U2 The Fundamental Theorem, computed** — one named, cited computation of Φ per class/county
  (reuse the `wage` opposition's `(w_paid, v_produced)` feed, §2f; retire or wire the dead
  `fundamental_theorem.py` formulas — no parallel Φ). Calibration check against
  `view_imperial_rent` (§2d) as a redundant-verification test. Narrator/observability surface so
  the theorem is *visible* (Ch. 25 cited in the ADR, per the ADR089 inline-citation convention).
- **U3 The accumulation loop** — organic composition (live tensor) → mechanization displacement →
  `ReserveArmyDynamics` flows → `reserve_ratio` producer; `DispossessionEventSystem` fed real
  rates (county-tick adapters or successor); dispossession → class transitions (satisfy the
  `transition_engine` gate); the MarketScissors crisis-swell (§2b) becomes one input among
  several, not the only producer. Chs. 25-27 cited.
- **U4 The working day** — `visibility_modifier` consumed by ConsciousnessSystem's
  exploitation-visibility path (coordinate with #42's ideology.py changes, §9); classifier fed
  from real hours/productivity (OPHNFB/HOANBS via `productivity_data_source` — its first
  reader). Ch. 10 cited.
- **U5 Parity** — headless runner wires `create_vol1_services` (SHARED unit with Vol II, §10).
- **U6 Oppositions** — shadow-bind the §4 candidates; GraphInputs fields; author the new
  couplings AT the contract-commit-reserved slots.
- **U7 Sentinels** — §6, red/green/red mutation-validated, in the umbrella + CLI.
- **U8 Defines sweep** — §5; every field read.
- **U9 Monitoring** — seam-registry rows for every new emission; retire §6.3's dead rows;
  observability veil-gated where value-axis.
- **Ceremony** — single baseline pass + drift table + ADR. Drift determined EMPIRICALLY;
  an all-zero table with stated reason is valid (Unit-6 doctrine-ceremony precedent) but
  unlikely here — U2/U3 touch live wealth channels; expect a real value-drift ceremony on the
  modernized gate (§1.2).

## 8. Verification battery
Per-unit scoped tests; spec-021's existing calculator suites become activation-truth tests once
gates light; property tests for conservation of the new flows (displacement flows sum to reserve
army delta); calibration test vs `view_imperial_rent`/`view_surplus_value`; `mise run
qa:regression` byte-identical until the declared ceremony; `mise run check` + `check:sentinels`
green; determinism (BLAS=1 pin, no wall-clock).

## 9. Known cross-program touchpoints
- **Vol II** — §10 protocol governs every shared room.
- **#42 consciousness wiring** (queue branch) — U4 touches the same
  `ideology.py`/`compute_exploitation_visibility` region #42-A/B just rewired; branch only after
  the queue PR merges (gate §1.1) and re-verify §2e line anchors.
- **#57 Wayne hex-substrate migration** — §2h; agree substrate boundaries before U3.
- **Trade program** (`project/research/trade-after-capital-refactors.md`) — kickoff proposal
  only after BOTH volumes merge; Ch. 22 material lands there, not here.
- **Empirical invariants program** (WID/DFA Pareto wealth law) — U2/U3 move wealth channels;
  its conditionality checks are downstream consumers.

## 10. PARALLEL EXECUTION PROTOCOL — Vol I ∥ Vol II as one workflow (owner directive 2026-07-20)

The owner runs both volumes concurrently as one giant workflow. The recon's shared-rooms sweep
found exactly where that is safe and where it needs structure:

**10.1 Branch model.** Both programs branch from the SAME post-gate dev tip (after §1 gates 1-2).
One branch per volume (`refactor/vol1-value-production`, `refactor/vol2-circulation`), one
worktree per branch, ONE controller session orchestrating both lanes. Mutating subagents stay
serialized PER WORKTREE (two lanes may mutate concurrently — different worktrees — but never two
mutators in one worktree). Vol I may start immediately; Vol II's U2+ holds until parquet cutover
(its hard gate) — staggered starts inside one workflow are expected.

**10.2 The contract commit (FIRST, before either lane writes code).** The #1 collision risk is
`catalog.py`: both programs append to the same `GraphInputs` dataclass and the same
`_DEFAULT_COUPLINGS` tuple, and Vol I has no reserved slots. Resolution: a single small PR to
dev, authored jointly at kickoff, that (a) reserves BOTH programs' exact opposition keys,
`GraphInputs` field names, and `Coupling(...)` entries (Vol I's authored fresh; Vol II's two
already exist) as documented dead slots in the established convention (*"out of scope for X,
NOT faked"* comment style, `catalog.py:585-586`); (b) records the shared binding-interface
contract block (both plans reference it instead of each pinning their own); (c) assigns the
shared units: who builds the gated-dormancy sentinel (§6.1 here ≡ §6.1 there), who lands the
headless-runner parity fix for BOTH service families (§2c here ≡ §2d/U5 there). Both branches
then fork AFTER the contract commit — every later catalog/coupling edit is append-at-reserved-slot,
mechanically conflict-free.

**10.3 Room partition (from the verified conflict map).**

| Room | Disposition |
|---|---|
| `catalog.py` (GraphInputs, couplings, bindings) | Contract commit reserves slots; then append-only per lane |
| `domain/economics/tick/system/__init__.py` (2223L) | Method-level ownership: Vol I owns `_apply_wage_pressure`/`_simulate_transitions`/working-day integration; Vol II owns the reproduction/circulation stub region (`:1308-1374`) and circulation methods. NO cross-edit; second-to-merge rebases (file drifts ~14 lines/day — expect it) |
| `headless_runner/runner.py` parity | ONE lane lands both `create_vol1_services` + `create_circulation_services` (contract commit assigns) |
| Gated-dormancy sentinel | Built ONCE (contract commit assigns); other lane contributes registry rows |
| Defines | Independent files/categories (`capital_vol1.py` + `economy_labor.py` extensions vs `capital_vol2.py`) — no conflict |
| Seam registry | Different row ranges, append-heavy — trivial rebase only |
| `factory.py` | Different named functions; only `__all__` may need a trivial rebase |
| `engine_bridge.py` | Vol I: no change needed (already wired). Vol II only |

**10.4 Machine constraints (dev box, standing).** The workflow NEVER fans out heavy test runs:
`qa:regression`, full `test:unit`, and baseline generation are single-flight ACROSS BOTH lanes
(one at a time, controller-scheduled). Scoped `mise run test:q -- <path>` per lane is fine.
Parallel read-only recon/review agents are unrestricted.

**10.5 Ceremonies and merge order.** Each program keeps its OWN declared ceremony and its own
honest PR (drift attributed per volume — never one blended table). First-ready merges first;
the second rebases onto the merged tip and re-runs its full battery (§8) there before its own
ceremony. If both are ready in the same window, Vol I merges first (production before
circulation is the material-causality order, and Vol II's realization/consumption reads benefit
from rebasing on the theorem/accumulation channels).

**10.6 Reviews.** Per-unit adversarial review per lane as usual (Sonnet implement, Opus review —
never Fable subagents). The final whole-branch review of the SECOND program to merge explicitly
checks the contract: every reserved slot consumed or honestly still-dead, no divergent
conventions between the two lanes' opposition bindings.

---

## Primary source
`sources/Capital-Volume-I.pdf` — **materialized in the queue worktree 2026-07-20** (`git lfs
pull --include sources/Capital-Volume-I.pdf`; 650 pages; the file is a 132-byte LFS pointer in
any fresh checkout — pull before grounding). Chapter numbering verified (English ed.): Part 1
Ch. 1 Commodities; Part 2 Ch. 6 Buying and Selling of Labour-Power; Part 3 Chs. 7-11 (Ch. 9 Rate
of Surplus-Value, Ch. 10 The Working Day); Part 4 Chs. 12-15 (Relative Surplus-Value, Machinery);
Part 5 Chs. 16-18; Part 6 Chs. 19-22 (Wages; Ch. 22 National Differences); Part 7 Chs. 23-25
(Ch. 25 The General Law of Capitalist Accumulation); Part 8 Chs. 26-33 (Primitive Accumulation).
Citation convention: inline by Part/chapter in the ADR, per `ADR089_vol3_money_scissors.yaml:57-60`.
