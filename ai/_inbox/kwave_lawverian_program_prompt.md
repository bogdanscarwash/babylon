# Claude Code Prompt — Program: Long Waves as Emergent Dialectics

You are working in the Babylon repo as a senior implementation agent. This program builds the **material generators of long-wave (Kondratiev-scale) dynamics — never the wave itself** — as adjunction instances in the existing Lawverian contradiction layer, plus an offline correspondence harness that tests for emergent quasi-periodicity on long headless runs.

Governing ruling (theoretical, binding): **Mandel's asymmetry.** The downswing is endogenous — this program builds its generators. The upswing is **not** automatic and may only occur through adjudicated event-layer outcomes that already exist (fascist bifurcation +1, war devaluation, re-division of the periphery). Building any endogenous "spring" mechanism is forbidden. Kondratiev periodicity is an **output** (a spectral property of the registry's slow modes) and an **initial condition** (the 2020s hydrated as K-winter) — never an input.

---

## 0. Read first, in order

1. `CONSTITUTION.md` — the dialectic primitive `D = (A, Ā, w, T, σ)`; III.8 Aleksandrov Test; III.10 Earn-Its-Keep; III.11 Loud Failure / honest nulls; VIII.11 tension-as-accumulator anti-pattern; the Amendment S escalation trigger (new primitive beside/above the dialectic).
2. `ai/decisions/ADR051_lawverian_dialectics_refactor.yaml` + its contract `project/06-lawverian-dialectics.md` — the executable layer: `GaloisConnection`, `AdjointCylinder`, `LevelLattice` + Aufhebung, `OppositionRegistry` (gap / balance ∈ [−1,1] / rate; principal = `gap*(1 + rate_weight*|rate|)`), regime classifier (reproduction/crisis/sublation), RUPTURE = gap above threshold **and** rising.
3. `ai/decisions/ADR070_emergent_class_partition.yaml` — `PoleSample`/`PoleReading`/`read_poles()`; the precedent for extending the registry **without** firing Amendment S.
4. `ai/decisions/ADR071_program20_means_of_production.yaml` — **verify what capital-stock state actually landed.** This program builds on Program 20, never parallel to it.
5. `ADR020` (parameter tuning methodology), `ADR031` (tick-keyed temporal tables), `ADR041` (headless runner), `ADR046`/`ADR045` (BEA/QCEW ingest), `ADR047` (per-tick read cache), `ADR038` (invariant test bundle).
6. Theory sources to **mine, not treat as landed reality**: `ai/brainstorms/tensor/capital_volume_ii_integration.md` (turnover time, fixed capital, depreciation funds as latent money capital, replacement waves), `capital_volume_iii_integration.md` (credit cycle phases, devaluation clearing), `ai/brainstorms/tensor/transformation_methodology_memo.md` (divergence-tracking, TSSI, Φ = W_actual − V_reproduction), `ai/epochs/epoch3/epoch2-trpf.yaml` (OCC, six counteracting factors).
7. The dialectics package itself (`src/babylon/domain/dialectics/` — **verify the actual path**; ADR051 predates the Program 14 layering move), the current opposition instance catalog, `src/babylon/data/defines.yaml`, the engine system registry / `run_tick` order, and the `qa:regression` harness + baselines.

Then write a **Verified Engine Reality** section in the ADR: what exists versus what the brainstorms imagine. Do not build against imagined state. Confirm tick semantics (1 tick = 1 week is the working assumption from the history-sweep; confirm against `SimulationConfig` before deriving any coefficient).

---

## 1. The Lawverian requirement (structural, not decorative)

Long-wave generators do **not** get their own subsystem vocabulary. Each is a new **opposition instance in the existing `OppositionRegistry`** — same measured-adjunction-defect shape, same fresh-per-tick discipline (VIII.11: recomputed, never accumulated), same honest-null rules (III.11: UNPOSITIONED, never a fabricated 0.0). The wave, if it exists, is the **slow mode of the contradiction layer's own time series** — gap/balance/rate of these instances over thousands of ticks. That is the tie-back: K-dynamics are not beside the dialectic; they are what the dialectic does at long timescale.

This follows the ADR070 precedent (instances/coarse-grainings of the ratified primitive), so Amendment S should not fire. **Argue this explicitly in the ADR, and STOP + escalate if any part of the design would fire it.**

Per III.10 (Earn-Its-Keep), every instance ships with a LAW, a PREDICTION, and a COMPUTATION — the tables below are mandatory ADR content.

### Instance 1 — VINTAGE: use-value durability ⊣ value durability (fixed capital)

- **Pole A:** physical persistence of means of production (service life remaining).
- **Pole Ā:** value persistence (economic life under **moral depreciation** — obsolescence driven by technique change).
- **gap:** normalized defect between economic and physical remaining life. Marx's moral depreciation *is* a measured adjunction failure: the value-form of the machine dies ahead of (or behind) its body.
- **T:** replacement/scrapping — a devaluation event that consumes the depreciation fund and instantiates a new vintage. Aufhebung reading: the old vintage is negated-and-preserved (fund → new capacity); consider whether this should emit through the existing `LEVEL_TRANSITION` pipeline or a sibling event — escalate the choice.
- **Wave physics:** replacement echo. Synchronized vintages bunch reinvestment; per-industry service lives come from BEA Fixed Assets (defines, `NEEDS-CALIBRATION`).
- **Earn-its-keep:** LAW — value conservation through depreciation (Σ transferred = initial value; remaining value never negative). PREDICTION — investment autocorrelation peaks near the vintage service-life lag. COMPUTATION — per-tick capital-stock age spectrum.

### Instance 2 — CREDIT: fictitious claim ⊣ realized surplus

- **Pole A:** claims on future surplus (aggregate debt service due this tick).
- **Pole Ā:** surplus actually produced (s from the economy).
- **gap:** bounded defect — use the house style already used by the value-form instance's Φ proxy `(w−v)/(w+v)`; verify and match.
- **T:** crisis/devaluation clearing, firing on the ADR051 RUPTURE shape — gap above threshold **and** rising (condition AND level; do not invent a new trigger form). Devaluation writes down claims **and** destroys capital value (feeds VINTAGE) — this is the Vol III "clearing ground" mechanism.
- **Earn-its-keep:** LAW — phase ordering (expansion → overextension → crisis → recovery; no teleports). PREDICTION — crisis incidence correlates with VINTAGE replacement bunching (depreciation-fund hoards are the credit system's material base — Vol II memo). COMPUTATION — the per-tick claims/surplus ratio series.
- **Storage:** if aggregate claims require new persisted state, that is a II.11 storage-owner ruling — **escalate, don't improvise.**

### Instance 3 — TECHNIQUE-RENT: monopoly technique ⊣ generalized technique

- **Pole A:** innovation surplus-profit — price above value while a technique is scarce.
- **Pole Ā:** socially generalized technique — value after diffusion.
- **gap:** intra-core price-value divergence magnitude. This **extends the value-form instance family**; it must not duplicate it. Critical distinction: imperial divergences (γ < 1, unequal exchange) **persist by design** — imperial rent is not competed away; only intra-core technique rents decay.
- **T:** diffusion — dissipative, monotone decay absent new innovation events. Innovation events (state `RESEARCH`/`DEVELOP` verbs or an exogenous catalog — verify what exists) mint new divergence.
- **Earn-its-keep:** LAW — monotone diffusion decay absent innovation. PREDICTION — profit-rate equalization fails across γ < 1 boundaries while succeeding intra-core over the diffusion timescale. COMPUTATION — divergence-magnitude series.
- The transformation operator remains unimplemented and **out of scope** (transformation memo ruling stands: track divergences, don't derive prices from values).

### Composite prediction (the program's own earn-its-keep)

On long runs with all three instances gated in, slow-mode spectra of registry series (gaps, balances, r, Φ) show multi-decade quasi-periodicity **only** in runs where restoration events occur; downswing-only runs (restoration suppressed) show secular decline without recovery. This is Mandel-vs-Kondratiev as a falsifiable structural difference: **the engine has no automatic spring.** If this prediction fails in either direction, that is a reportable finding, not a tuning target.

---

## 2. Phase plan

These phases are the project's standard SHADOW → GATING lifecycle, not scope cuts. The full program above is the plan; phases sequence its promotion.

### Phase 0 — Survey + ADR (no code)

- Verified Engine Reality: dialectics module path + registry API; what ADR071/Program 20 landed for capital stock and where it lives on the graph; bifurcation-consequence hooks relevant to restoration; tick length; current instance catalog; defines schema; whether an innovation-event source exists.
- Author the ADR (next free number — verify against `ai/decisions/`) containing: the three instances with earn-its-keep tables, the Amendment-S non-firing argument, an Aleksandrov trace for **every** proposed coefficient, phase gates, and any storage rulings needed.
- **STOP conditions (escalate to the BD, do not proceed):** any new node/edge/morphism type; any new persisted table without an owner ruling; any missing restoration hook you'd be tempted to build here; any Amendment-S ambiguity.

### Phase 1 — Shadow substrate (byte-identical)

- Frozen Pydantic state models (constrained types — `Probability`, `Currency`, `Intensity`; no raw dicts) for vintage/credit/rent state, riding the entities Phase 0 ruled on.
- Register the three instances; measure fresh per tick; write readings as honest-null shadow attrs + seam rows. Zero adjudication — nothing downstream reads them.
- Gate: `qa:regression` 5/5 byte-identical (a `defines_hash`-only regeneration is acceptable and must be documented as such).
- Tests: the LAW rows as unit laws; registry integration; determinism (same seed → identical readings); UNPOSITIONED honesty (entities with no basis on an axis get **no** reading).

### Phase 2 — Gated promotion (baselines move; one instance at a time)

- **VINTAGE → economy:** depreciation flows into c; replacement events consume funds and reset vintages. Golden trace: a hand-computed multi-vintage scenario, numbers pinned.
- **CREDIT → economy:** gap gates investment capacity; crisis T triggers claim write-downs plus a capital-devaluation shock into VINTAGE and the profit rate. Golden trace: a pinned expansion → overextension → crisis sequence.
- **RENT → economy:** divergence feeds the price fields; innovation mints, diffusion decays. Golden trace: a single-innovation lifecycle, birth to full diffusion.
- Each promotion: R-PROOF divergence doc + regenerated baselines + property laws added to the invariant bundle (ADR038 style). All coefficients in `defines.yaml` with sources; seed values marked `NEEDS-CALIBRATION` and tuned only per ADR020 methodology.

### Phase 3 — Correspondence harness (offline tool, not engine code)

- Lives in `tools/` (or the house analysis location): reads tick-keyed series from Postgres for long headless runs (ADR041; multi-thousand ticks, multiple seeds) and computes Welch PSDs + wavelet scalograms for r, Φ, instance gaps/balances, and investment.
- Runs the composite prediction: matched seed-pairs with restoration events enabled vs suppressed. Reports emergence or an honest null — III.11 applies to research claims too.
- Determinism check: identical spectra per seed across reruns.
- Deliverable: the tool, a committed report template, and a findings doc under `project/research/`.
- **Prohibited:** any coefficient change justified by the spectrum. If someone tunes a service life to sharpen a 50-year peak, the wave has been imposed through the back door and the program has failed its own mandate.

---

## 3. Initial conditions — hydrating the K-winter

- Capital-stock age structure from BEA Fixed Assets where the ingest pipelines exist (ADR045/046 family); credit gap seeded near overextension; technique-rent field compressed (late-diffusion phase). The 2020s data **is** the winter — stagnant QCEW wages against financialized composition are the phase variables; no phase clock is stored anywhere.
- Respect ADR051's recorded open item: hydration heterogeneity (per-county wealth from QCEW) belongs to the hydration/economics workstream — coordinate with it, don't fork it.
- Anything authored rather than data-derived goes in defines with an Aleksandrov note explaining the material relation it traces to.

---

## 4. Hard constraints (checklist)

- Determinism and tick-purity: seeded RNG only; identical hash per seed remains the invariant. Non-determinism is a bug, full stop.
- III.10: earn-its-keep tables are mandatory ADR content; a construct with no law/prediction/computation does not ship.
- III.11: honest nulls everywhere, including the harness report.
- VIII.11: every reading is a fresh measured gap; nothing accumulates.
- Layering (Program 14, import-linter enforced): instances in `domain/dialectics`, math in `formulas`, wiring in `engine`, `intelligence` observes only.
- No hardcoded coefficients — `GameDefines`/`defines.yaml` is the single moddable source of truth.
- Frozen Pydantic models, constrained types, never raw dicts.
- TDD (red → green → refactor) + Amendment Q behavioral contracts: goldens, property laws, byte-identical regression gates.
- Conventional commits; branch from `dev` (house naming, e.g. `feature/kwave-dialectics`); never commit to `main`/`dev` directly.
- Mermaid for any diagram.
- No MVP scoping: the phases above are promotion gates on the full program, not cuts of it.

## 5. Deliverables

1. ADR (next free number) + program/spec doc.
2. Three registered opposition instances + state models + defines entries.
3. Engine wiring per promotion (Phase 2), with R-PROOF docs and regenerated baselines.
4. Test suites: instance laws, golden traces, property laws in the invariant bundle, regression gates.
5. Correspondence harness + report template + findings doc.
6. Hydration coordination notes (what the K-winter seeding needs from the ingest workstream).

Escalate to the BD at every STOP condition and at every choice the ADR flags as a ruling. When in doubt between building and asking, ask.
