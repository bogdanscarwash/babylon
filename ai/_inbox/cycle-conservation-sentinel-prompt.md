# Claude Code Prompt — Generalized Cycle Layer + Conservation Sentinel

You are implementing a new observational construct in **Babylon**: the generalized
Cycle (the abstraction unifying Marx's circuits — C-M-C, M-C-M′, M-C{LP,MP}…P…C′-M′,
M-M′, the metabolic loop, and the D-P-D′ lifecycle circuit) and the **Conservation
sentinel** that is its earn-its-keep law. This is the Economic Conservation /
Accounting invariant candidate already ranked as earned in
`ai/_inbox/archive/abstractions.md` (candidate #3), landing in the sentinel-family
pattern established by Program 17 (ADR068) and the partition sentinel (ADR070).

Work autonomously except for **one owner checkpoint at the end of Phase 0**.
Surface decisions; never guess silently. If any step appears to require a new
kernel primitive or a constitutional violation, STOP and report — do not improvise
an amendment.

---

## 0. Read first (mandatory, before any code)

Read, in order:

1. `CONSTITUTION.md` — especially III.10 (Earn-Its-Keep), III.11 (Loud Failure),
   VIII.11 (Tension-as-Accumulator anti-pattern), VIII.12 (wiring/liveness),
   the Aleksandrov Test, and the amendment rules.
2. `ai/decisions/ADR051_lawverian_dialectics_refactor.yaml` — the dialectics core
   this extends (OppositionRegistry, adjunction defects, frames deferral).
3. `ai/decisions/ADR068_program17_wave1_sentinels.yaml` — sentinel family
   structure, layer-0.5 placement, shared_tick, CLI conventions, owner rulings
   (no nightly CI for sentinels; advisory posture).
4. `ai/decisions/ADR070_emergent_class_partition.yaml` — the landing pattern to
   copy: advisory-only sentinel, single-SoT registry, probe harness, shadow-safe,
   baseline-neutral, honest-None.
5. `src/babylon/dialectics/` — actual module layout and APIs (core, instances,
   levels, regime). Verify names; the ADRs are the map, the tree is the territory.
6. The sentinels package and `tools/sentinel_check.py`, `tools/partition_probe.py`
   — match their structure, naming, and report conventions exactly.
7. `src/babylon/data/defines.yaml` — where every new tolerance/coefficient goes.
8. The `GraphInputs` snapshot type and how ContradictionSystem @18 consumes it.
9. `git log`/`git diff` on the latent branch `053-conservation-invariants` —
   decide salvage vs. supersede; record the disposition either way.
10. `project/research/melt-tvt/` — the Marx aggregate equalities (Σ price = Σ value,
    Σ profit = Σ surplus) that become this sentinel's cross-frame correspondences.
11. The import-linter contract (Program 14 layering) — determines legal homes for
    new modules.

File paths above are cited from the ADR record, not a live tree. **Verify every
path before designing against it; where reality differs, reality wins** and you
record the delta in the audit.

---

## 1. Theory capsule — what you are building

A **Cycle** is a closed walk in the category of value-forms. Objects are forms
{M, C, P, LP, D, B}; generating morphisms are metamorphoses (sale, purchase,
production, wage payment, tribute, extraction, care), each the transformation of
a dialectic instance. A Cycle = (ordered steps returning to start, basepoint,
valuation frame). Marx's three figures of the circuit (Capital II) are rotations
of one loop; C-M-C and M-C-M are the two composites of the single sale/purchase
opposition (the value-form adjunction ADR051 already instantiates). The
OppositionRegistry measures adjunction defects at pole pairs (dimension 0); the
Cycle measures the same failure-to-close around a walk (dimension 1).

**Frames** (enum, extensible): `MONEY`, `LABOR_VALUE`, `BIOMASS`.

**Per-edge decomposition**, per frame f:

    ΔV_f(e) = creation_f(e) + transfer_f(e)

where transfer is signed net (in − out) and creation is licensed per
(MorphismClass, Frame) pair. Default license table — everything zero except:

| MorphismClass | Frame        | Creation licensed |
|---------------|--------------|-------------------|
| PRODUCTION    | MONEY        | yes (realized surplus) |
| PRODUCTION    | LABOR_VALUE  | yes (living labor)     |
| ECOLOGICAL    | BIOMASS      | yes (regeneration R; extraction is signed negative) |
| all others    | any          | no — pure transfer     |

**The laws (declared invariants of the sentinel):**

- **L1 — No value from circulation.** For every edge with an unlicensed
  (class, frame) pair, creation_f(e) = 0. Sale, purchase, finance, tribute,
  rent redistribute; they never create. Unequal exchange is transfer, not
  creation — Φ is L1's canonical consequence.
- **L2 — Cycle accounting.** Δ_f(cycle) = Σ_e ΔV_f(e), and the residual
  |Δ_f − Σ creation − Σ transfer| ≤ ε_f. This subsumes value-in = value-out +
  surplus (M-C-M′), and ΔB = R − E·η is exactly L2 in the BIOMASS frame.
- **L3 — Rotation invariance.** Δ_f is invariant under basepoint rotation
  within a frame (Capital II's "same circuit, three standpoints" as a property
  test; guards future refactors).
- **L4 — Global transfer antisymmetry.** Σ over all instrumented edges of
  transfer_f = 0 within ε (double-entry). Named special case: every Φ credited
  to a Core node is debited to a Periphery node.
- **Diagnostics (not violations):** sign classification per cycle —
  accumulating (Δ > tol), reproducing (|Δ| ≤ tol), depleting (Δ < −tol) —
  plus coverage fraction (share of edges instrumented) and honest-absence list.

**Canonical instances** (registry candidates; Phase 0 decides which are
instrumentable):

| Cycle | Frame(s) | Expected reading |
|-------|----------|------------------|
| C-M-C per social_class (reproduction circuit) | MONEY | Δ ≈ pure transfer; persistent positive transfer into a Core class's reproduction circuit is W_c > V_c made per-cycle — labor-aristocracy visibility |
| M-C{LP,MP}…P…C′-M′ per capital unit (granularity per audit) | MONEY, LABOR_VALUE | Δ = surplus, sourced only on PRODUCTION edges |
| M-M′ finance/interest | MONEY | creation = 0; Δ entirely claims on other cycles' creation |
| Φ trade circuit per trade relation | MONEY | pure transfer; L4 antisymmetry |
| Global metabolic loop | BIOMASS | L2 = ΔB = R − E·η; overshoot = MONEY-frame Δ>0 coupled to BIOMASS-frame Δ<0 |
| D-P-D′ generational lifecycle | (future) | ADR future-work table only — see Phase 2 |

**Cross-frame correspondences (MELT):** Σ prices = Σ values and Σ profit =
Σ surplus are computed and reported but tagged ADVISORY/PROXY — the
value→price transformation is explicitly unimplemented (ADR051 deferred
`observe(frame=…)`); do not implement it here, and say so inline in the report.

---

## 2. Hard constraints (constitutional, non-negotiable)

1. **Derived, not primitive.** A Cycle is a closed walk of dialectical
   transitions plus a frame — built entirely from existing primitives. No
   amendment is needed and none may be smuggled in. If you find yourself
   needing a new kernel primitive, STOP and report.
2. **III.10 Earn-Its-Keep.** The construct ships WITH its law: no Cycle code
   lands without the sentinel that checks it.
3. **III.11 Loud Failure / honest absence.** Never fabricate a quantity the
   engine didn't produce. An uninstrumented edge reads as absent (None),
   propagates to coverage < 1, and is listed — never defaulted to 0.0.
4. **VIII.11 Fresh-per-tick.** Every reading is recomputed from that tick's
   flows. No cross-tick accumulators inside the sentinel; cumulative series
   are derived downstream by analysis tooling, never stored as state.
5. **VIII.12 No phantoms.** The registry contains only cycles with ≥ 1
   instrumented edge. Everything else is documented as future work in the ADR,
   not registered as dead code.
6. **Aleksandrov Test.** Every registered step carries a `material_relation`
   string naming the concrete relation it traces to (wage payment, sale of
   labor-power, tribute, extraction…). No ungrounded arrows.
7. **Determinism.** Sorted iteration everywhere (registry keys, edge
   traversal); no RNG, no wall clock; numeric discipline matched to the repo's
   existing convention for value quantities — find how Program 17 resolved the
   `float += Decimal` seam and follow that convention exactly.
8. **Coefficients in defines.yaml, never hardcoded.** Add a `conservation:`
   block: `epsilon_money`, `epsilon_labor_value`, `epsilon_biomass`,
   `classification_tolerance`. Conservative defaults; the owner tunes.
   **Never widen an ε to silence a residual** — a residual is a finding.
9. **Advisory posture.** The sentinel gates nothing. No nightly CI plumbing
   (standing owner ruling: sentinels are local/on-demand).
10. **TDD** (red → green → refactor) with Amendment-Q-style behavioral
    contracts: property laws + goldens pin what the layer does, not how it's
    built.

---

## 3. Phase 0 — Instrumentation audit (ends in the owner checkpoint)

Produce `reports/cycle-instrumentation-audit.md`:

1. Inventory every per-edge / per-node / per-tick value-flow quantity the
   engine actually emits that could instrument a candidate cycle edge: wage
   flows, surplus/exploitation quantities, Φ / imperial_rent, tribute/rent
   flows, metabolic R, E, η, B, consumption flows. Cite the emitting system
   and the attr/table for each.
2. For each candidate cycle in the table above: list its edges, the quantity
   (if any) instrumenting each edge, its frame, and a disposition —
   `instrumented` / `computed-but-unexposed` / `not-computed`.
3. Apply the ≥1-live-edge rule to propose the wave-1 registry set, and pick
   the capital-circuit granularity (per-org vs per-county) from what is
   actually emitted, not from what would be nice.
4. Record the `053-conservation-invariants` branch disposition
   (salvage: which commits; or supersede: why).
5. Decide implementation posture: **prefer probe-only (zero engine mutation)**.
   Only if required quantities are computed-but-unexposed do you propose
   engine-side shadow writes, following ADR070's proven-baseline-neutral
   pattern (computed-fields registered same-commit, honest-None,
   qa:regression byte-identical).
6. Propose branch name (default `feature/cycle-conservation-sentinel`; use the
   repo's spec-number scheme if one is assigned) and the next free ADR number.

**CHECKPOINT: present the audit + proposed registry + posture to the owner and
wait for approval before Phase 1.** This is the only pause.

---

## 4. Phase 1 — Cycle core (TDD)

Home: `src/babylon/dialectics/cycles/` (verify against the import-linter
contract; dialectics sits in the domain layer and must not import persistence
or engine — if layering forbids your plan, stop and report, don't bend the
contract).

Frozen Pydantic models, constrained types per house style:

- `Frame` enum; `MorphismClass` enum
  (PRODUCTION, CIRCULATION, FINANCE, TRIBUTE, ECOLOGICAL, REPRODUCTION).
- `CycleStep`: stable endpoint IDs, edge type, morphism class,
  `material_relation: str`.
- `CycleSpec`: id, ordered steps (closed-walk validated), basepoint index,
  frames.
- `EdgeReading`: per frame — delta, creation, transfer, `instrumented: bool`;
  absent values are None, not 0.0.
- `CycleReading`: per frame — yield, creation_sum, transfer_sum, residual,
  coverage, classification, violations.

Pure functions: `read_cycle(spec, snapshot, frame) -> CycleReading`; the
creation-license table lives beside the registry as data, not code branching.

Tests first, minimum set:

- Decomposition arithmetic and residual computation.
- L1 adversarial: a circulation-class edge carrying nonzero creation is
  flagged, never absorbed.
- L3 rotation invariance as a property law (match how existing property tests
  are written — deterministic settings).
- Honest-absence propagation: a None edge lowers coverage and appears in the
  absence list; it never fabricates a residual.
- Classification thresholds read from defines, not literals.

## 5. Phase 2 — Registry (single source of truth)

Mirror the partition precedent: one registry module that is the sole
vocabulary owner. If the audit chose probe-only, keep the registry entirely
outside the engine's import graph and say so in the ADR; if shadow writes were
approved, the engine imports the registry exactly the way it imports the
partition cell vocabulary — drift impossible by construction.

Canonical builders for the audit-approved set only. Deterministic
construction: sorted node-ID iteration; cycle IDs derived from content, stable
across runs. D-P-D′ goes in the ADR's future-work table with the lifecycle
quantities it awaits — it does not enter the code registry (no phantoms). If
the audit surprisingly finds lifecycle instrumentation, that's a checkpoint
escalation, not a unilateral add.

## 6. Phase 3 — Sentinel + harness

- `tools/cycle_probe.py`: persistent-graph harness modeled directly on
  `partition_probe.py` — run a scenario, evaluate the registry per tick from
  the live snapshot, emit readings.
- `tools/sentinel_check.py` grows a `conservation` subcommand (match sibling
  naming and `--check` semantics); add the matching `mise` task.
- Report format consistent with the sentinel family: per cycle, per frame —
  Δ, creation, transfer, residual vs ε, coverage, classification; then L1/L4
  violations; then the cross-frame MELT correspondences under an explicit
  ADVISORY/PROXY banner carrying the transformation-problem caveat.

## 7. Phase 4 — Verification

- Full suite green; mypy/ruff/hygiene clean.
- `qa:regression` 5/5 byte-identical, baselines UNMODIFIED. Probe-only should
  make this trivial; if any engine-side write exists, prove neutrality the
  ADR070 way (stash-and-compare or equivalent).
- Run the probe on the canonical michigan and wayne_county scenarios.
- Write a crown-finding section reporting exactly what the laws say about
  real runs. A violated law on real data is a FINDING — report it; do not
  tune ε, patch the registry, or massage quantities to make it pass.

## 8. Phase 5 — Records

- ADR at the next free number, in the yaml house style, copying ADR070's
  section structure (context / decision / consequences / deferred /
  verification). Include: the 053 branch disposition, the future-work cycle
  table (D-P-D′ and any not-computed edges), and every path-delta found in
  Phase 0.
- Session note in `ai/state.yaml` per repo convention; update the sentinel
  family count wherever it is documented.
- Conventional commits; branch from `dev`; never commit to `main`/`dev`.

---

## 9. Out of scope — do not implement

- OppositionRegistry circuit-defect coupling / regime-classifier input from
  cycle yields (a follow-on program with its own earn-its-keep case).
- The value→price transformation (ADR051 explicitly deferred it; cross-frame
  stays proxy).
- Automatic cycle detection/enumeration over the graph (registry-only by
  design: determinism + exponential cycle space).
- Making cycles causal — engine scheduling and the 28-system order are
  untouched.
- Any frontend lens (punch-list territory).
- Nightly CI plumbing for the sentinel (standing owner ruling).

## 10. Acceptance checklist

- [ ] Phase-0 audit written; owner checkpoint passed; 053 disposition recorded.
- [ ] Cycle core: frozen models, pure read functions, license table as data.
- [ ] L1–L4 implemented as declared invariants; diagnostics separated from
      violations.
- [ ] Registry: single SoT, deterministic IDs, ≥1-live-edge membership only.
- [ ] Probe + `sentinel_check conservation` + mise task, matching family
      conventions.
- [ ] `conservation:` block in defines.yaml; zero hardcoded tolerances.
- [ ] Honest absence throughout; no fabricated zeros; coverage reported.
- [ ] Fresh-per-tick readings; no cross-tick accumulators.
- [ ] Suite green; qa:regression 5/5 byte-identical; baselines unmodified.
- [ ] Crown findings reported honestly from michigan + wayne_county.
- [ ] ADR + state note landed; conventional commits on a feature branch.
