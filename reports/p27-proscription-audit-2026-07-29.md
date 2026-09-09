# Proscription Audit — Imposed Functional Forms & Question-Begging (Director directive 2026-07-29)

**Directive (verbatim):** "we shouldn't be enforcing a sigmoid on any mechanics,
the sigmoid should be the result of P(revolution) and P(acquiescence). we should
review our tests additionally and thoroughly ensure we weren't being proscriptive
trying to beg the question or tune our simulation to a certain outcome. remember:
organic emergence as a result of the algebraic lawverian operations."

**Method:** four parallel read-only sweep lanes (formula surface, test estate,
defines/coefficients, goldens/scenarios), an Opus theory-synthesis lane, then an
**adversarial verification pass** — every non-minor finding attacked by an
independent verifier instructed to refute it. 29 agents; findings below are
evidence for the Director's theory review, **not** decisions; no code was changed.

**Verdict statistics:** 24 non-minor findings raised → **21 REFUTED** by the
adversarial pass (the estate is substantially cleaner than a naive sweep
suggests; the refuted list is Appendix B, the 50 checked-and-clean areas are
Appendix C) → **3 CONFIRMED** (below). 11 minors (Appendix A).

The three confirmed findings are precisely the directive's named concern:

---

## Confirmed findings (survived adversarial refutation)

### src/babylon/formulas/survival_calculus.py:43 — CORE-LINE

**Class:** imposed-form

**Finding:** P(S|A) is a stipulated logistic curve 1/(1+exp(-k(wealth-subsistence))) rather than an emergent result of any accumulation.

**Evidence:** calculate_acquiescence_probability hardwires the response curve the Director says should EMERGE. What it short-circuits: for a class with a within-class wealth distribution, the probability of surviving by acquiescence IS the measure of members whose wealth clears subsistence — a sigmoid-shaped aggregate would emerge organically from integrating any unimodal wealth distribution against the subsistence threshold (and its steepness would BE the class's wealth variance, connecting to the intensive-aggregation-variance-error memory). Instead the curve is imposed on the class MEAN with a free 'steepness_k' knob (SurvivalDefines.steepness_k=10.0, described literally as 'Game design: sigmoid sharpness'). Mitigating seam: it is injected via formula_registry.py:106 as hot-swappable 'acquiescence_probability', consumed by SurvivalSystem (engine/systems/survival.py:154) and re-used consistently by counterfactual_hope_gain (formulas/politics.py:68) for electoral hope — so one seam replaces it everywhere. Note: the imposed form is enshrined in the project CLAUDE.md 'Mathematical Core' (P(S|A) = Sigmoid(Wealth − Subsistence)), so fixing it touches the documented theory line — Director-level, as the directive anticipates.

**Adversarial verdict (survived):** The finding survives adversarial attack on every refutation avenue. (1) Boundary-contract defense fails: the FormulaRegistry seam (formula_registry.py:106) covers only SurvivalSystem; formulas/politics.py:19 imports calculate_acquiescence_probability directly at module level and allegiance.py:109 passes the raw steepness_k define, so the electoral hope path is hard-bound to the logistic — the finding's 'one seam replaces it everywhere' mitigation is actually OVERSTATED, strengthening the finding. (2) Material-derivation defense fails: no written Aleksandrov chain (III.8) exists for the logistic or for k=10.0 — SurvivalDefines.steepness_k is described as 'Game design: sigmoid sharpness in acquiescence probability' naming no material process, and docs/reference/formulas.rst:180-207 states the form without derivation; the materialist substitute (measure of members clearing subsistence, steepness = within-class dispersion) is realizable in-model since the per-class inequality attribute already drives Grinding Attrition mortality, yet k is one global knob unwired to it. (3) Ratified-core-line defense fails against controlling authority: the Director's 2026-07-29 theory-line ruling (memory: direction-no-imposed-sigmoids-organic-emergence) — the ruling that chartered this audit — explicitly names 'the hardcoded Sigmoid(Wealth − Subsistence) in the survival calculus' as the paradigm proscriptive imposed form, and Constitution I.3 already codifies the emergence discipline ('Stable r MUST emerge from interaction, not be assumed'). (4) Numeric-guard defense fails: line 42 is the overflow clamp; cited line 43 is the logistic itself. (5) Fixture-vehicle defense fails: this is live production math (survival.py:154 every tick; electoral hope field). All checkable factual claims in the finding verified exact (line numbers, default 10.0, description text, consumers); its single error (seam coverage) understates the problem. Severity 'core-line' is correct — disposition is Director-level per the ruling's own clause that changing any live formula is theory-line work.

### src/babylon/models/entities/precarity_state.py:91 — SIGNIFICANT

**Class:** imposed-form

**Finding:** precarity_index = 1 − sigmoid(real_wage − subsistence) with hardcoded unit steepness, duplicating the survival sigmoid outside the formula registry.

**Evidence:** sigmoid_value = 1.0/(1.0+math.exp(-diff)) in a computed_field — no define parameterizes it, and it does not route through the hot-swappable 'acquiescence_probability' registry seam, so a future emergent replacement of the survival curve would leave this twin behind (DRY violation doubling as an audit hazard). Same short-circuit as survival_calculus: precarity is the below-subsistence measure of the class's wage distribution.

**Adversarial verdict (survived):** The finding survives every refutation avenue and is STRENGTHENED. Verified: precarity_state.py:91 computes 1/(1+exp(-diff)) inline with implicit k=1, while the canonical seam is fully parameterized — formulas/survival_calculus.py::calculate_acquiescence_probability(wealth, subsistence, steepness_k), registered as 'acquiescence_probability' (engine/formula_registry.py:106), driven by define survival.steepness_k=10.0 (data/defines.yaml:164) — and the site bypasses all three. Refutation attempts and failures: (1) 'Materially-derived form with written grounding' — the sigmoid FORM is the ratified Survival Calculus complement, but the finding targets the hardcoded scale, which has NO written grounding and contradicts the project's own tracked convention (defines.yaml:649 cites 'matches SurvivalDefines.steepness_k=10.0' as codebase precedent). At gap=-0.2 the twin yields 0.5498 vs 0.8808 for the canonical complement — a divergent curve, not a restatement. (2) 'Numeric guard' — inverts: the twin DROPPED the canonical ±500 exponent clamp; Currency is unbounded above, and PrecarityState(nominal_wage=0.0, subsistence_threshold=800.0).precarity_index raises OverflowError (demonstrated live). (3) 'Fixture vehicle / dormant code' — fails: PrecarityState is a src/ public-API model with zero production consumers BUT is a chartered P0 wiring target (reports/epochs-vision-gap-audit.md:301 'fully implemented, zero consumers... Wire into every wage StatChip'; reports/aidocs-vs-code-audit-2026-05-16.md:627,841 charter a PrecaritySystem feeding precarity_index into ConsciousnessSystem drift). Under the wiring doctrine (ADR109) the 'twin left behind' hazard is concrete: a registry hot-swap or emergent replacement of the survival curve would never touch this site, and tests/constants.py:511 plus the 65-test unit suite pin the k=1 curve as a behavioral contract, entrenching the divergence. (4) 'Boundary contract' — no persistence/serialization path consumes it. The question-begging leg also holds: 1-sigmoid(w-s) equals the below-subsistence mass of the class wage distribution only under an imposed Logistic(mean=w, scale=1) assumption asserted nowhere. New aggravators found: numeric divergence from canon (0.55 vs 0.88), missing overflow clamp (live OverflowError on valid Currency inputs), and chartered-P0-wiring status making the audit hazard concrete rather than hypothetical.

### tests/scenarios/test_endgame_flow.py:233 — SIGNIFICANT

**Class:** outcome-assertion

**Finding:** Asserts `outcome == GameOutcome.FASCIST_CONSOLIDATION` (also line 427) even though the class docstring explicitly declares the fascist fixture is 'only the cheapest vehicle' and 'Babylon does not test for specific endgame outcomes.'

**Evidence:** The docstring (lines 199-208) states the subject under test is the detector-to-Simulation wiring, and lines 273/518 show the compliant form (`outcome != GameOutcome.IN_PROGRESS`). But lines 233 and 427 pin the specific political outcome equality, making FASCIST_CONSOLIDATION load-bearing: if the engineered false-consciousness fixture ever recognized a different pattern first, the wiring test would fail on outcome identity, not on wiring. Letter-of-the-ruling violation (outcome as asserted subject) with an easy fix: assert `outcome == endgame_detector.recognized_pattern` and `!= IN_PROGRESS`.

**Adversarial verdict (survived):** The finding survives adversarial attack. Verified against tests/scenarios/test_endgame_flow.py: lines 233 and 427 assert `outcome == GameOutcome.FASCIST_CONSOLIDATION` while the class docstring (199-208), the module header (16-21), and the per-test docstring (251-255) all declare the fascist fixture a mere vehicle and the subject to be detector-to-Simulation wiring. The ratified emergent-endgames ruling (owner 2026-07-16, verbatim) permits outcomes as fixture vehicles but forbids them as asserted subjects — 'never which outcome fires under which conditions' — and these two lines assert exactly that. Every refutation route fails: (1) not a boundary contract — the facade returns `detector.recognized_pattern or IN_PROGRESS` (_legacy.py:1067-1073), so the minimal wiring assertion is pass-through identity plus `!= IN_PROGRESS`, the form the same file already uses at 273/518; the enum pin adds only proscribed political knowledge; (2) not a determinism pin — that form exists value-free at lines 458-459; (3) not grounded — the file's own docstrings contradict the assertions, marking them as residue the ruling-driven sharpening refactor missed; (4) not risk-free — the fixture docstring (101-110) records the fixture-outcome coupling already broke once (spec-116 pacing ceremony, 0.75→0.9 gate), and a future retune firing a different axis first would fail the test on outcome identity, not wiring. The proposed fix (`outcome == endgame_detector.recognized_pattern` and `!= IN_PROGRESS`) strictly dominates: stronger on wiring, silent on politics. Finding stands as a letter-of-ruling violation with an easy compliant rewrite.

---

# Theory-Line Synthesis — The Sigmoid Imposition and the Emergence Path

**Lane:** theory-synthesis (reserved-line-adjacent — everything below is **proposal and evidence, never decision**).
**Directive under review (Director, 2026-07-29):** *"we shouldn't be enforcing a sigmoid on any mechanics, the sigmoid should be the result of P(revolution) and P(acquiescence) … organic emergence as a result of the algebraic lawverian operations."*

**Verification note.** Everything marked ✅ below I read directly this session (`src/babylon/formulas/survival_calculus.py`, `engine/systems/survival.py`, `formulas/vitality.py`, `formulas/lifecycle.py`, `models/entities/social_class.py`, `domain/bifurcation/consciousness.py`, `domain/economics/reserve_army/calculator.py`, `formulas/market.py`, `NORTH_STAR.md`, `docs/reference/bsl-language.rst`, the P27 design spec, and a read-only query against `data/sqlite/marxist-data-3NF.sqlite`). Items marked ⟨L*n⟩ are carried from sweep lane *n* and not independently re-read; I have not downgraded any lane finding, but the Director should treat ⟨L⟩-only claims as one-source.

---

## 1. THE SIGMOID-IMPOSITION MAP

Ranked by how load-bearing the imposed curve is on the game's central question (does crisis route to fascism or revolution). "Short-circuits" names the operation the curve stands in for.

### Tier 1 — core-line: the curve decides the thing the game is about

**1.1 `P(S|A)` — the constitutional sigmoid.** ✅
`src/babylon/formulas/survival_calculus.py:41-43` — `1/(1+exp(-k(w − s)))`, applied by `engine/systems/survival.py:154` to **per-capita class-mean wealth**.

*Short-circuits:* the measure of class members whose wealth clears subsistence. That measure is a counting operation over a population; the sigmoid replaces it with a designer curve on the mean, with the population's internal structure discarded and its role handed to a free knob (`steepness_k = 10.0`, described verbatim as *"Game design: sigmoid sharpness"*, `config/defines/survival.py:18`). This is the exact failure mode the intensive-aggregation-variance-error memory records, one level down: the variance that got averaged away comes back as a tuning parameter.

*Blast radius* (all one seam — `formula_registry.py:106` registers it hot-swappable, and `SurvivalSystem` consumes via `services.formulas.get`, verified ✅): SurvivalSystem @15; `calculate_crossover_threshold` (which inverts it analytically); `formulas/politics.py:68` `counterfactual_hope_gain`, hence the whole hope-field → Allegiance @17.42 → Electoral @17.45 chain; OODA action previews; plus one **off-registry twin** at `models/entities/precarity_state.py:91` (hardcoded unit steepness, would survive a registry replacement — an audit hazard, not just DRY) ⟨L1⟩.

*Governance weight:* this form is written into the project `CLAUDE.md` "Mathematical Core" and into `docs/reference/` as the Survival Calculus. Changing it touches the documented theory line, which is why it is correctly a Director item rather than an engineering one.

**1.2 `consciousness_sigmoid` — the bifurcation weighting.** ✅
`src/babylon/domain/bifurcation/consciousness.py:66`; consumed by `bifurcation/bridges.py:135` as `bridge_potential = infrastructure × sigmoid(collective_identity)`.

This is the most serious finding in the sweep, and it is not really a shape problem — it is an **admission in the defines**. `config/defines/consciousness.py:459` sets `consciousness_sigmoid_midpoint = 0.4` with the description *"Behavior-tuned: CI value at sigmoid inflection. **Below-center so breakage cliff catches assimilated communities (CI<0.4)**"*, and the module docstring states the mechanism's purpose is that *"assimilationist solidarity classifies as fragile/fascist."* The steepness (`10.0`) is justified only as *"Codebase precedent: matches SurvivalDefines.steepness_k"* — an undocumented value from 1.1 metastasizing into a justification — and `consciousness_filter_threshold = 0.2` is then *"Derived"* by evaluating the tuned curve at CI=0.27.

*Short-circuits:* the fragility of assimilated solidarity under crisis. That fragility should be an *observed outcome* of crisis events actually breaking low-CI SOLIDARITY edges in the dynamics. Pre-multiplying edge weights by a cliff whose inflection was chosen to catch the intended communities means the analysis **predicts what the weighting encoded**. Companion: `_CRISIS_FRAGILE_THRESHOLD = 0.3` is hardcoded inline at `consciousness.py:142` (not a define) and directly stamps the `crisis_fragile` label ⟨L1⟩.

**1.3 The wealth-distribution spring — an outcome as attractor.** ✅
`src/babylon/formulas/class_dynamics.py:228` — `d²W/dt² = β·(dW/dt) − ω²(W − W*)`, with `W*` = the observed 2015-2025 US distribution (`equilibrium_w1..w4 = 0.305/0.382/0.294/0.02`, `config/defines/economy_class.py:43-75`).

Not a transcendental, but the same species of imposition and arguably the purest instance of begging the question in the estate: the module docstring celebrates *"Key Finding: wealth distribution is remarkably stable"* while five of six inter-class extraction alphas default to `0.0000`. The stability is the spring, not the exploitation. This directly inverts the standing empirical-invariants ruling (invariants belong in **contrapositive runtime checks**, not enforced attractors) — as written, the Pareto invariant check can never fail, so it can never evidence the theory. *Mitigating:* `WealthDistributionSystem @21.5` is Program-21 Phase-1 shadow; Phase-2 feedback is owner-gated ⟨L1,L3⟩.

**1.4 Reserve-army wage pressure.** ✅
`src/babylon/domain/economics/reserve_army/calculator.py:52-65` — a baseline-renormalized sigmoid of `reserve_ratio` (`k=20`, `r0=0.08`, ceiling 0.5). Consumed live twice: `engine/systems/reserve_army.py:94` (`median_wage *= (1 − pressure)`) and the TickDynamics Vol-I layer ⟨L1⟩.

*Short-circuits:* labour-market confrontation. The same package already models the accumulation loop (`reserve_army/accumulation.py`, Vol I ch. 25) — downward wage movement could emerge from employed/reserve replacement flows and organizational strength. Instead the modeller stipulates *where* wage pressure switches on (8% reserve ratio) and *how sharply*. Since **falling `W_c` is the trigger of the whole bifurcation loop** (Fundamental Theorem: revolution impossible while `W_c > V_c`), this curve's shape parameterizes when the game's core loop fires at all. `sigmoid_k`/`sigmoid_r0` descriptions name only the consumer, never a derivation (`config/defines/economy_labor.py:62`).

### Tier 2 — significant: mechanically live, upstream of contradiction dynamics

| # | Site | Imposed form | What it short-circuits |
|---|---|---|---|
| 2.1 | `formulas/market.py:107` ✅ | `tanh(log_ratio / scale)` as the **canonical** `price_value` opposition Balance; `scale=0.5` labelled *"Engineering"* | A Lawverian opposition balance computable directly from the poles: `(p−v)/(p+v)`, the pattern other oppositions already use. `tanh` stipulates where the opposition **stops responding**. Feeds `ContradictionSystem` @ `systems/contradiction.py:427`. |
| 2.2 | `formulas/market.py:87` ✅ | Linear damped-driven harmonic oscillator in log space | Gravitation of price to value operates in Marx through inter-sector capital mobility equalizing profit rates — which `domain/economics/substrate/equalization.py` **already models**. Oscillation/damping would emerge from capital chasing sectoral deviations. Has a partial written derivation (Vol III ch. 10) — the defensible end of the spectrum, but the underdamping is a coefficient-guaranteed property of the chosen form, not a result. |
| 2.3 | `formulas/reactionary.py:89` ⟨L1,L3⟩ | `sigmoid(chauvinism − discipline)` with **hardcoded** unit steepness and zero midpoint — not routed through defines at all | An organization is a population of members; defection under crisis is the *fraction* whose accumulated chauvinism exceeds the discipline the org can bring to bear. Both a directive violation and a defines-routing gap (the module docstring claims all defaults trace to `ReactionaryDefines`; this one doesn't). |
| 2.4 | `formulas/sustained_exploitation.py:198` ⟨L1⟩ | Gaussian bump `exp(−(b−peak)²/2σ²)` for chauvinist agitation of bribed strata | Best-documented curve in the estate (Emmanuel, MIM, Amin cited; non-monotonicity pinned by a sentinel), but peak/falloff are self-declared `PROVISIONAL`. "The *marginal* labour aristocracy is the most reactionary" is a claim about **precarity of the bribe** — it would emerge as the measure of the stratum within threat-distance of losing its bribe under the current Φ trend, not as a bell curve over the mean. |
| 2.5 | `survival_calculus.py:90` ✅ | `crossover = s − ln(1/p_rev − 1)/k` | Inherits 1.1 exactly: the wealth at which revolution becomes rational exists only as the **inverse image of the designer curve**. Under an emergent `P(S|A)` this becomes a quantile equality on a real distribution. |
| 2.6 | `engine/field_registry.py:194` ⟨L1⟩ | `10·(1 − e^(−raw/10))` imperial-rent normalization, `/10` hardcoded, comment admits *"maps to reasonable field values"* | Knee position silently decides how much additional rent still moves the ContradictionField → FieldDerivative → CollapseTransition chain. |
| 2.7 | `models/entities/precarity_state.py:91` ⟨L1⟩ | `1 − sigmoid(wage − subsistence)`, hardcoded, off-registry | The off-registry twin of 1.1. Would survive a clean replacement of the survival curve. |

### Tier 3 — minor / confined

- `ooda/action_effects.py:95` ⟨L1⟩ — the Shannon-entropy contestation measure, **explicitly fenced as a read-only diagnostic** in `consciousness_routing.py:439-448`, leaks into mechanics: EDUCATE gets a multiplier bonus above a tuned threshold. The entropy functional isn't the problem; a declared diagnostic quietly gating a mechanic is.
- `engine/scenarios/_legacy.py:628` ⟨L1⟩ — Gaussian metro kernel (σ=2.0° inline) and a hardcoded 50k-influence CORE/PERIPHERY cutoff. Initial-condition synthesis is legitimate, but the CORE/PERIPHERY partition is theoretically load-bearing in MLM-TW and is baked in by an arbitrary threshold rather than derived from extraction relations.
- `domain/institution/balance.py:83` ⟨L1⟩ — `min(1, 1 − max_weight + 0.1)`; the inline `+0.1` guarantees nonzero contestation under total hegemony, undeclared.

### What the map shows in aggregate

Five of the six transcendental families in the codebase are **imposed response curves on class-mean scalars**. The exceptions verified clean: `anchor.py:89` (coordinate change on an empirical FRED ratio, with `NoDataSentinel` guards), `contradiction.py:455` (exact inverse of a log-ratio), geodesy `sqrt/sin/cos` on the immutable substrate, `monte_carlo.py` offline statistics, and `layout.py` presentation-only trigonometry. **Fourteen of seventeen `formulas/` modules contain zero transcendental call sites at all** — linear/ratio/clamp algebra only ⟨L1⟩. The imposition is concentrated, not diffuse. That is good news for the review: this is a small number of specific sites, not a pervasive stylistic habit.

---

## 2. EMERGENCE SKETCH — the survival calculus without an imposed curve

### 2.1 The reframing

Today: **`P(S|A) = σ(k·(w̄ − s))`** — a stipulated response of a class *scalar*.

Proposed: **`P(S|A)ᵢ = μᵢ({m ∈ class i : wₘ ≥ sᵢ}) / Nᵢ = 1 − Fᵢ(sᵢ)`**

— the normalized measure of class members whose individual wealth clears their subsistence requirement, where `Fᵢ` is the class's within-class wealth CDF. This is a **definition, not a curve**: a pushforward of the individual threshold-crossing indicator along the class's population measure. In the C/G/P vocabulary of NORTH_STAR §3 it is a **G-family coarse-graining** (population → class) of a **thresholded opposition** (accumulated wealth ⊣ subsistence requirement) — both constructor families already ratified, no new formalism minted.

### 2.2 Why the sigmoid comes back as a *result*

The S-curve is not lost; it is *derived*, and its steepness stops being free.

- **Lognormal within-class wealth** (the standard empirical form): `P(S|A) = Φ((μ − ln s)/σ)` — a Gaussian CDF in log-wealth. It is sigmoid-shaped, bounded in [0,1] by construction, monotone in mean wealth, and crosses 0.5 exactly when median wealth meets subsistence. Its **steepness is `1/σ`** — the inverse of within-class dispersion. And σ is pinned by the Gini the class already carries: `G = 2Φ(σ/√2) − 1`, so `σ = √2·Φ⁻¹((G+1)/2)`. **`steepness_k` stops being "Game design: sigmoid sharpness" and becomes a measured property of the class's internal inequality.** A more unequal class has a *flatter* survival response — which is itself a substantive theoretical claim the model would now make rather than assume.
- **Pareto tail** (the form the repo already knows — `formulas/lifecycle.py:143` `compute_pareto_gini`, `G = 1/(2α−1)` ✅): survival fraction above a floor is `(w_min/s)^α` — a power law, **not** a logistic. That the two candidate distributions give different aggregate shapes is the point: the shape becomes an *empirically decidable question* instead of a stipulation.
- **Empirical CDF** — the strongest option, and it is already in the build product. Verified ✅ against the read-only reference DB: `fact_census_income` holds **7,207,200 rows** of per-county × 16-bracket × year × race ACS B19001 household counts, with `dim_income_bracket` carrying the real bracket ladder (`Less than $10,000` … `$200,000 or more`). That table **is** `F`, measured, per county. Today it is consumed only as a top-2/bottom-2 band *ratio proxy* (`domain/economics/throughput/adapters.py:793`). Read as a CDF instead, `P(S|A)` is a linear interpolation over a real Lorenz curve — **zero transcendentals, zero free shape parameters, and a per-county rather than per-archetype answer.**

### 2.3 The state variables are already declared

This is what makes the sketch concrete rather than aspirational. `SocialClass` already carries (verified ✅, `models/entities/social_class.py`):

- `wealth: Currency` (:308) and `population: int` (:406) — the mean;
- **`inequality: Gini` (:411)** — *"Intra-class Gini coefficient"*, the dispersion parameter;
- `s_bio: Currency` (:386) and `s_class: Currency` (:391) — subsistence split into biological and social-reproduction components;
- `county_fips` (:426) — the attribution key into the ACS distribution.

`(w̄, G, N)` is exactly the parameter triple a two-parameter CDF needs. **The emergent form requires no new primitives and no new fields.**

**But — a data-seam finding the sweep lanes did not surface.** `inequality` is read as `required=True` by exactly one system (`engine/systems/vitality.py:229` ✅) and is **seeded by nothing** in `src/babylon/engine/scenarios/` or `src/babylon/data/game/` (verified: zero assignment hits ✅). It therefore defaults to `0.0` across the canon scenarios, which makes Grinding Attrition's `threshold = 1.0 + inequality` collapse to `1.0` and the heterogeneity channel **materially dead in every gated run**. Two consequences worth the Director's attention: (a) the imposed sigmoid is currently doing the work a live distribution would do, which is *why* it needs a free steepness knob; (b) the emergent form's key input already exists in the schema and in the reference data but is un-hydrated — this is a wiring gap (a W-C dataflow motion under ADR109), not a mathematical obstacle.

### 2.4 The precedent is already in the codebase — in the wrong system

`formulas/vitality.py:38-47` ✅ (Grinding Attrition):

```
coverage_ratio = wealth_per_capita / subsistence_needs
threshold      = 1.0 + inequality
if coverage_ratio >= threshold: return 0.0
attrition_rate = (threshold − coverage_ratio) · (base_factor + inequality)
```

This is a **piecewise-linear approximation of exactly the measure §2.1 proposes** — "what fraction of the class falls below the threshold, given mean coverage and dispersion." No transcendental. The docstring even states the emergent reading: *"with high inequality you need almost 2× subsistence to prevent deaths."*

So the codebase already contains, in one system, the derivation shape the survival calculus lacks. The structural observation this invites: **mortality and acquiescence are two readings of one below-subsistence measure at two different thresholds** — Vitality reads the biological level-set (`s_bio`), Survival reads the social-reproduction level-set (`s_bio + s_class`). The model already splits the two thresholds. Unifying them removes a curve *and* a duplication, and yields a claim with real theoretical content: the gap between the two level-sets is the population living above biological death and below social reproduction — precariat as a measured region of the distribution rather than a labelled archetype.

### 2.5 The `P(S|R)` side

Current: `min(1.0, organization / (repression + ε))` ✅. Partly emergent already — `_calculate_solidarity_multiplier` (`systems/survival.py:29-61`) sums incoming SOLIDARITY edge strengths, a genuine C-family graph composition. Three problems remain:

1. `repression_faced` is a per-class scalar with **no dispersion** — repression is modelled as uniform incidence, so the same heterogeneity that gives `P(S|A)` its shape is absent on the revolution side, breaking the symmetry the directive implies.
2. `min(1.0, ·)` is a hard clip that destroys all information above 1 — a class with 5× organizational advantage is indistinguishable from one with 1.01×.
3. `ε` is a division guard, not a material quantity (III.11 loud-failure territory).

Proposed symmetric form: **`P(S|R)ᵢ = μᵢ({m : organizational cover(m) ≥ repression exposure(m)}) / Nᵢ`** — a coverage-versus-exposure measure, in [0,1] **by construction** (no cap needed, no ε). The carceral/ControlRatio estate already carries repression-incidence state that could supply the exposure distribution.

### 2.6 Where the sigmoid legitimately reappears — the Director's reading, made precise

Rupture fires when `P(S|R) > P(S|A)`. With both sides as population measures, the class-level rupture propensity is the measure of the joint distribution over the region `{r > a}`. Because **both** margins are threshold-crossings over heterogeneous populations, the aggregate rupture response to *any* material driver — falling wage, rising imperial rent, rising repression, a Φ-disruption — is automatically S-shaped:

- ~0 while the driver is far from the crossing region (nobody has crossed),
- rising through the bulk of the population's dispersion (the crossing sweeps the distribution),
- saturating at 1 once nearly everyone has crossed.

**The aggregate sigmoid is the CDF of the crossing point across the population.** Its steepness is the dispersion of that crossing point — a compound of wealth inequality and repression heterogeneity, both material. That is, as literally as I can render it, *"the sigmoid should be the result of P(revolution) and P(acquiescence)."*

Two further dividends: `calculate_crossover_threshold` becomes a quantile lookup (`w̄* : 1 − F(s; w̄*) = P(S|R)`) rather than an analytic inversion of a designer curve; and the empirical-invariants ruling becomes enforceable in its intended contrapositive form, because nothing pins the distribution to its observed shape any more.

### 2.7 What this does to the BSL kernel-intrinsic surface

Per `docs/reference/bsl-language.rst` §2.7 and §4.3 ✅, transcendentals are **never** BSL primitives — `sigmoid, exp, log, tanh, sqrt, entropy` exist only as named kernel intrinsics with pinned implementations and written tolerance derivations. P27 design §13 open ruling #2 (*polynomial approximation vs pinned deterministic libm*) is the hardest cross-language determinism question in the program, and the global CLAUDE.md contract rule #4 states the reason plainly: basic IEEE-754 ops reproduce across languages; **libm transcendentals do not.**

De-imposition shrinks that surface materially:

| Intrinsic | Production call sites today | Under de-imposition |
|---|---|---|
| `sigmoid` | survival_calculus (1.1), reserve_army (1.4), precarity_state (2.7), bifurcation consciousness (1.2), reactionary defection (2.3) | **Every one is an imposed response curve on a class-mean scalar.** If all five become population measures, `sigmoid` has **no remaining call site** and leaves the table entirely. Largest single reduction available. |
| `tanh` | exactly one: `calculate_scissors_balance` (2.1) | If the canonical opposition uses the `(p−v)/(p+v)` ratio algebra already used elsewhere, **zero call sites** — leaves the table. |
| `exp` (non-sigmoid) | chauvinist Gaussian (2.4), imperial-rent normalization (2.6), `_legacy` metro kernel, financialization inverse transform | First three are imposed or build-time-synthesizable. The fourth is a genuine `exp(log(x))` round-trip — eliminable by carrying the ratio instead of its log (a representation choice, not a theory change). |
| `log` | monetary anchor coordinate change (verified clean ⟨L1⟩), market-scissors log space | If 2.2 is re-derived, `log`'s remaining role is an **empirical coordinate change computable at data-build time** and hashed as declared input — not a per-tick kernel intrinsic. |
| `sqrt` | Allegiance, deprecated `contradiction.py:151` (no production caller after Phase C), offline Monte Carlo, geodesy | The benign one: `sqrt` is IEEE-754 **correctly-rounded** and reproduces bit-exactly across conforming implementations. Needs no tolerance policy. Geodesy is substrate/build-time anyway. |
| `entropy` | fenced read-only in `consciousness_routing` (exemplary); leaks into mechanics only at `ooda/action_effects.py:95` (Tier 3) | Close the leak and entropy is projection-lane only — **outside the tick hash** by Amendment S, so it needs no intrinsic at all. |

**Net:** the transcendental intrinsic table could plausibly shrink from `{sigmoid, exp, log, tanh, sqrt, entropy}` to `{sqrt}`, with `{exp, log}` surviving at most as a matched pair. Since `sqrt` is the one member that reproduces bit-exactly, **open Director ruling §13 item 2 would become moot rather than answered.** That is a large determinism-budget dividend, and it argues that the theory correction and the Rust refoundation are the *same work*, not competing work.

Three supporting notes:

- **BSL expresses the emergent forms natively.** Population measures are folds over queries with comparison and division — `(fold count (nodes …) …)`, arithmetic, `if`. No intrinsic call, hence no `:cost 5 + callee` fuel charge, hence tighter static bounds.
- **The intensivity law actively *prefers* the emergent form.** §3.4 makes `mean` over an intensive field legal *only* with an explicit extensive `:weight`. A population measure is precisely a weighted mean of an intensive indicator with population as the extensive weight — the canonical legal shape. The current class-mean sigmoid is closer to the pattern §3.4 exists to reject.
- **The fixed-point lane opens up.** `P(S|A)` as a count ratio is `Int ÷ Int → Coefficient` — **exact integer arithmetic, zero binary64 anywhere in the core survival path**, fitting §3.2's Currency/Int rounding table. The most theoretically load-bearing quantity in the game would become exactly reproducible by construction.
- **Honest cost, flagged not minimized:** quantile interpolation over per-county empirical CDFs costs table lookups per class per tick, and §3.7 charges folds at the declared ceiling. That is an engineering question for the porting contract, not a theory objection — but it is real and should not be discovered late.

---

## 3. TEST-ESTATE REMEDIATION LIST

The estate is what will *resist* de-imposition — several suites pin the imposed forms as contracts. For each, the contract that should be pinned instead.

### 3.1 Outcome-as-asserted-subject (direct tension with the standing emergent-endgames ruling)

| Site | Pinned today | Should pin instead |
|---|---|---|
| `tests/unit/engine/systems/test_electoral_goldens.py:287` ⟨L2,L4⟩ | `first.winning_coalition == "org/party-fascist"` after 10 engine ticks under per-arc tuned defines | The **resolver arithmetic**: given the engineered vote inputs, the plurality winner is whoever the tallies say. The named ending becomes an observation recorded in the baseline, never a unit-test equality. (`test_the_same_operator_routes_the_twins_apart`, :266-272, is the clean model already present in the same file — one operator, direction decided by SOLIDARITY-bridge topology alone.) |
| same file :137-140, :165, :185 ⟨L2,L4⟩ | `arm == "capitulate"`; `seated["party_id"] == "org/party-socdem"`; `formed_tick == 8` | The **governance predicate table**: no-organs ⟹ capitulate; bridges + Φ-starved ⟹ synthesis. Assert the predicate, let the arc land where it lands. |
| same file :193-208 ⟨L2⟩ | `debt_stock == approx(64_630_747.63)`, `delivery_ratio == approx(0.962253)` | These are spot-run-observed values frozen as unit assertions — golden-regression pins in unit-test clothing. They belong **only** in the byte baselines, where the ceremony gate governs drift. |
| `tests/scenarios/test_endgame_flow.py:233, :427` ⟨L2⟩ | `outcome == GameOutcome.FASCIST_CONSOLIDATION` — while the class docstring says the fixture is *"only the cheapest vehicle"* and *"Babylon does not test for specific endgame outcomes"* | `outcome == detector.recognized_pattern` **and** `outcome != IN_PROGRESS` — the wiring contract the docstring declares. Lines :273 and :518 in the same file already show the compliant form. Cheapest fix in the estate. |

### 3.2 Form-pins that would block an emergent replacement

| Site | Pinned today | Should pin instead |
|---|---|---|
| `tests/unit/formulas/test_survival_calculus.py:61-73, :93-110, :342-400` ⟨L2⟩ | exact 0.5 at threshold; `k`'s effect on transition width; expected crossover values computed in the test via `ln(1/p − 1)` — the designer curve's own closed-form inverse | **Behavioral laws that survive a form change**: bounds in [0,1]; monotone increasing in wealth; `P(S|A) → 0` as `w → 0` and `→ 1` as `w → ∞`; crossover exists and is unique where `P(S|A) = P(S|R)`. Add the *new* falsifiable law: **steepness is inversely monotone in intra-class Gini** — a prediction with content, replacing a pin with none. The bounds/monotonicity properties in `test_survival_calculus_properties.py` already survive as-is and are the model. |
| `tests/unit/bifurcation/test_consciousness.py:68` ⟨L2⟩ | 0.5 at midpoint to 1e-10; "breakage cliff" values at CI=0.1 and CI=0.8 under the tuned `(0.4, 10.0)` | The **material claim**: under an identical crisis shock, solidarity edges between low-CI (assimilated) nodes break at a higher rate than those between high-CI nodes. Assert the differential in the *dynamics*, not the weighting function's shape. |
| `test_bridges.py:320-329`, `test_assimilation_trap.py:127-231` ⟨L2⟩ | "expected" values computed by calling `consciousness_sigmoid` itself | Tautological with respect to the form under review — self-referential expectations cannot detect that the form is wrong. Replace with end-to-end differential assertions as above. |
| `tests/unit/formulas/test_consciousness_routing.py:558` ⟨L2⟩ | fascist dominance at the magic pair `(solidarity=0.6, pressure=0.5)` | Property form: *for any solidarity s, there exists a chauvinist pressure p that flips the dominant pole* — the reachability claim is legitimate MLM-TW theory; the specific flip point is a defines artifact. |

### 3.3 Baselines and gate coverage

- **The six outcome-named baselines** (`weimar/mitterrand/syriza/debs/bernie_valve.json` + `fascist_bifurcation.json`) ⟨L2,L4⟩. Byte-pinning *whatever happens* is legitimate and well-governed (ADR090 + §6.5 ceremony). **The ratchet is naming the required ending in the scenario's identity** — `tools/regression_scenarios.py:96` describes them as *"Fascist consolidation through the ballot,"* and that string is stamped into each baseline. Any theory change that alters an ending then forces a ceremony declaring *"weimar no longer ends in fascist consolidation,"* which is structural pressure to re-tune inputs until the analogy's ending recurs. **Proposal for the Director:** decouple scenario *identity* (initial material conditions: `weimar_conditions`) from scenario *outcome* (an observation in the baseline, not in the name). The gate keeps its full byte-identity power; only the naming stops asserting.
- **The politically one-sided gate estate** ⟨L4⟩ — `tools/regression_scenarios.py:353` et al. require `fascist_drift`, `fascist_revanchism`, and positive `fascist_alignment` deltas as *evidence rows* (gate-coverage-truth reds if they stop firing), while RUPTURE, MASS_AWAKENING, CONSCIOUSNESS_TRANSMISSION and CLASS_DECOMPOSITION **never fire in any of the 11 canon scenarios** and are parked as declared gaps (all SOLIDARITY edges seeded `solidarity_strength = 0.0` in the five original scenarios). Net: **the regression estate cannot detect a change that kills the revolutionary path, but reds if the fascist path stops firing.** Whatever the Director rules on the sigmoids, this asymmetry deserves a separate ruling — the fix is a scenario whose evidence rows require revolutionary-pole events, not a change to the fascist ones.
- **Two input calibrations that suppress the core mechanic** ⟨L4⟩: `engine/scenarios/_legacy.py:256` seeds `periphery_wealth = 0.6  # Calibrated: P(S|A) > P(S|R) prevents immediate revolt` and `comprador_cut = 0.90  # Calibrated to prevent Comprador Liquidation`, underlying 4 of 11 canon scenarios; `engine/scenarios/electoral_goldens.py:96` raises worker repression in all five electoral goldens with the comment that *"zero repression makes P(S|R) infinite — the worker revolts at tick 1 and Struggle severs its exploitation edge (a topology change the dense contract forbids)."* The second is the sharper one: **a gate-infrastructure constraint (static topology for the dense byte contract) is being satisfied by suppressing the game's core rupture mechanic.** That is a real tension between III.11's dense contract and emergent dynamics, and it is an architecture question, not a tuning one.

### 3.4 What the estate already does right (the convergence target)

Named so the review knows the standard is achievable in-house, not imported: **all 17 files in `tests/unit/engine/laws/`** ⟨L2⟩ pin clamps, monotonicity, no-op gates, conservation, state-machine closure, idempotence and argmax soundness with file:line grounding and explicit sections *refusing* false laws — zero outcome trajectories. `tests/integration/test_induced_crisis.py` intervenes **only on the material base** (`imperial_rent_pool=0`, `extraction_efficiency=0`, docstring: *"touching NO StruggleSystem severing or consciousness gating"*), then asserts the superstructural consequence **disjunctively** and phase-contrasts it against 20 pacified ticks that must not rupture — the Fundamental Theorem and its contrapositive as a falsifiable prediction. `tests/integration/mechanics/test_rupture_events.py` pins the rupture gate in **both** directions. These are the emergence-style contracts; the remediation above is asking the rest of the estate to converge on them.

---

## 4. TUNING LEDGER

Every coefficient below is outcome-tuned or underived. Column 3 is a proposal for what would discharge it.

### 4.1 Curve-shape knobs of imposed forms — retire with the form

| Coefficient | Value / declared rationale | Disposition |
|---|---|---|
| `SurvivalDefines.steepness_k` ✅ | `10.0`, *"Game design: sigmoid sharpness"* | **Becomes derived**: `k = 1/σ`, `σ = √2·Φ⁻¹((G+1)/2)` from the class Gini (§2.2). Also **stop the propagation** — `BifurcationDefines.consciousness_sigmoid_steepness` cites this value as its *sole* justification ("codebase precedent"). |
| `ReserveArmyDefines.sigmoid_k` / `sigmoid_r0` ✅ | `20.0` / `0.08`, descriptions name only the consumer | Retire with the imposed curve; wage pressure emerges from employed/reserve replacement flows the same package already models. Fails the Aleksandrov Test as written — steepness and midpoint of an imposed curve trace to nothing material. |
| `MarketDefines.scissors_balance_scale` ✅ | `0.5`, *"Engineering: … saturates near 65% divergence"* | Retire with `tanh`; saturation emerges from `(p−v)/(p+v)` ratio algebra. |
| `MarketDefines.price_reversion` / `price_damping` / `fictitious_reversion` ⟨L3⟩ | `0.02 / 0.15 / 0.01`, *"underdamped … so prices oscillate around values"* | The oscillation is a **coefficient-guaranteed property of the chosen form**. Needs either derivation from inter-sector capital mobility (`substrate/equalization.py` already models it) or explicit declaration as a Θ_feel pacing knob whose *sign* is theory-fixed and whose *magnitude* is not. |
| `reactionary.py` implicit `k=1`, `midpoint=0` ⟨L1,L3⟩ | hardcoded in the function body | Unauditable and un-moddable; violates the never-hardcode-a-coefficient rule regardless of the theory ruling. |
| `field_registry` `/10` rent scale ⟨L1⟩ | hardcoded, *"maps to reasonable field values"* | Not a define, not derived, and mechanically upstream of CollapseTransition. |
| `_CRISIS_FRAGILE_THRESHOLD = 0.3` ⟨L1⟩ | inline in `bifurcation/consciousness.py:142` | Not a define; stamps a political label (`crisis_fragile`) by threshold rather than demonstrating collapse. |
| `domain/institution/balance.py` `+0.1` ⟨L1⟩ | inline magic offset | Undeclared floor guaranteeing nonzero contestation under total hegemony. |

### 4.2 Thresholds placed to guarantee a theory-approved branch

These are the subtlest category: each is **theory-cited** (the Director's own line — Cope, Amin, hegemony-held), but the calibration loop is self-referential — the sim's own trajectories were measured and the threshold slotted between them, so the discrimination **cannot fail and therefore cannot evidence the theory.**

| Coefficient | The admission | Proposal |
|---|---|---|
| `bribery_tension_threshold = 0.7` ⟨L3⟩ | *"sits just above the bridged aristocracy's 0.667 peak (so BRIBERY holds whenever the pool is high) and below the periphery's ~0.85 floor (so the super-exploited are never bribed)"* | Derive the bribe/repress choice from the **Φ arithmetic itself** — bribery is chosen when the rent pool can cover the wage premium at lower cost than repression. Then class-differential treatment is an *output*, and the theory has been tested rather than installed. |
| `iron_fist_tension_threshold = 0.5` ⟨L3⟩ | same calibration block | As above. |
| `rupture_gap_threshold = 0.9` ⟨L3⟩ | *"keeps the pacified bridged decade (empirical gap band ~[0.03, 0.67]) rupture-free"* | The historical fact (hegemony held) is the right anchor; encoding it by sliding the trigger above the observed band means rupture-*absence* is imposed. Under §2.6 the rupture condition is a measure comparison with no free threshold at all — this coefficient would simply cease to exist. |
| `jackson_threshold = 0.4` + `revolutionary_agitation_boost 0.5` / `fascist_identity_boost 0.2` / `fascist_acquiescence_boost 0.2` ⟨L3⟩ | all bare *"Game design"*; the two branch outcomes are **stamped directly onto class attributes** | The fascist turn should emerge from SOLIDARITY-edge routing (Constitution I.4), not be written as an attribute signature once a threshold branch is taken. Highest-value item in this table after 4.1's top row. |
| `fascist_majority_fraction 0.75 → 0.9` ⟨L3⟩ | raised *"keeping first_recognition past the no-pattern-before-tick-520 gate under null play"* | Mitigations are real (quantization degeneracy with 6 archetypal entities; detector is observes-only). The reviewable part is the **asymmetry**: the fascist recognizer got a documented pacing calibration while the sibling revolutionary thresholds (0.7/0.8) carry bare "Game design" labels. Equal scrutiny either way. |

### 4.3 Outcome-encoded baselines

| Item | Disposition |
|---|---|
| `equilibrium_w1..w4 = 0.305/0.382/0.294/0.02` + `beta`/`omega` ✅⟨L1,L3⟩ | The observed FRED-DFA distribution as a **spring attractor**, with five of six extraction alphas at `0.0000`. Proposal: keep the fitted first-order alphas (that half is exemplary — `tools/analyze_wealth_distribution.py`), **retire the restoring spring**, and move the observed distribution into a contrapositive runtime check per the standing empirical-invariants ruling. The Phase-1 shadow status makes this the cheapest core-line correction available. |
| `consciousness_sigmoid_midpoint = 0.4` ✅ | *"Below-center so breakage cliff catches assimilated communities."* No material derivation is possible for a parameter defined by the classification it must produce; this one deserves retirement rather than derivation. |
| `consciousness_filter_threshold = 0.2` ✅ | *"Derived: sigmoid(CI=0.27, midpoint=0.4, k=10)"* — derived **from** the tuned curve. Falls with it. |
| `chauvinist_peak_location 0.1` / `chauvinist_peak_falloff 0.3` ⟨L1⟩ | Self-declared `PROVISIONAL`. The qualitative shape has the best written derivation in the estate (Emmanuel/MIM/Amin, sentinel-pinned non-monotonicity); only the Gaussian realization and its two parameters need replacing — by the measure of the stratum within threat-distance of losing its bribe under the current Φ trend. |

### 4.4 The census finding — the largest single audit gap

⟨L3⟩ A regex parse of `description=` across the 29 defines modules: **724 fields total; ~471 (≈65%) carry no rationale label and no citation.** Declared-tuned: 140 `"Game design:"` + 35 Θ_feel + 4 `"Behavior-tuned"` + 4 PROVISIONAL. Materially cited: 52. Engineering guards: 17.

An undocumented coefficient cannot be *classified* as tuned or derived — for this review, each bare value is an unfalsifiable free parameter. **The in-repo cure already exists and works**: `config/defines/politics.py`'s Θ-tier convention (ADR127) declares at birth whether each of its ~45 fields is terrain fact (Θ_data), theory-fixed sign/bound (Θ_theory), or pacing knob (Θ_feel), with Θ_feel explicitly scoped to *"how long hope takes to die, never whether the ceiling exists."* That is precisely the emergence-preserving discipline the directive asks for, and it currently covers **one module out of twenty-nine**. Extending the tier convention estate-wide would convert this entire report's category-4 findings into a standing, checkable property.

Also worth a line: **duplicate magic-number signature defaults** shadowing defines values across `class_dynamics.py:205`, `dynamic_balance.py:28-39`, `trpf.py:25`, `solidarity.py:14`, `metabolic_rift.py:14`, `community.py:21-22` ⟨L3⟩. Dead at verified production call sites, but `dynamic_balance.py`'s `bribery_tension_threshold` default (`0.3`) **has already diverged** from the recalibrated defines value (`0.7`) — any test exercising defaults instead of defines is pinning outcomes to values the moddable source of truth no longer contains.

---

## 5. OPEN THEORY QUESTIONS — for the Director

Clearly separated. Each is a decision I am proposing be *made*, not one I am making.

**Q1 — Which reading of the directive governs?**
I developed the reading that the sigmoid should appear in the **aggregate rupture response** (§2.6: the CDF of the crossing point across a heterogeneous population), with neither `P(S|A)` nor `P(S|R)` individually curve-shaped. An alternative reading is that each of `P(S|A)` and `P(S|R)` may remain smooth so long as it is *derived*, and only their *composition* is what may not be tuned. These give different targets for 1.1 and 1.4. **The Director's reading should be recorded before any code moves.**

**Q2 — Does de-imposition require an amendment?**
NORTH_STAR §3 declares the formalism surface **closed for v1.0**; new formalism requires a constitutional amendment. Removing an imposed form and replacing it with a G-family coarse-graining of an existing thresholded opposition arguably mints no new formalism — but `P(S|A) = Sigmoid(Wealth − Subsistence)` is written into `CLAUDE.md`'s Mathematical Core and into `docs/reference/`. Is de-imposition (a) a documentation correction, (b) an ADR, or (c) an amendment? Note that Amendment AE (v3.0.0, 2026-07-29) already reopened the engine substrate, so the closure's scope in the Program 27 era may itself need restating.

**Q3 — Which within-class distribution is canon?**
Lognormal (sigmoid aggregate, steepness `1/σ`), Pareto (power-law aggregate — the repo already carries `compute_pareto_gini`), or the empirical ACS bracket CDF (7.2M rows, verified present, per-county, zero free parameters)? The empirical option is strongest on Aleksandrov grounds and weakest on runtime cost and on coverage (it is US household *income*, not class-attributed *wealth*; the mapping from ACS households to `SocialClass` blocks is itself a modelling decision). A hybrid — empirical where county-attributed, two-parameter analytic elsewhere — is possible but would need a declared honesty fence (`NoDataSentinel` / `{absence}`), not a silent fallback.

**Q4 — Should the survival and vitality thresholds be unified?**
§2.4's structural observation: mortality and acquiescence are two level-sets of one below-subsistence measure (`s_bio` vs `s_bio + s_class`, both already declared fields). Unifying them removes a curve and a duplication and gives "precariat" a measured definition. But it couples two systems currently at positions @1 and @15 in the materialist-causality order, and Vitality's Grinding Attrition is the earlier, cruder approximation — the Director may prefer them to stay independent readings for pedagogical clarity.

**Q5 — Does the theoretical claim "more unequal ⟹ flatter survival response" survive scrutiny?**
This is the substantive content the emergent form *adds*: a class with high intra-class inequality has a wider crossing-point dispersion and therefore a *less* switch-like rupture response. Is that MLM-TW-correct? The intuition cuts both ways — high inequality also means a larger already-below-subsistence tail (which Grinding Attrition already encodes as *more* mortality). I can construct the argument in both directions and cannot adjudicate it; this is squarely reserved-line.

**Q6 — The gate estate's political asymmetry (§3.3).** Whatever is ruled on the sigmoids: the regression estate currently **requires** fascist-pole events as evidence rows and **never exercises** the revolutionary pole in any of the 11 canon scenarios. Under the Director's compass ("mechanics that are engaging AND instill correct revolutionary theory"), a gate that cannot detect the death of the revolutionary path seems worth its own ruling, independent of this review.

**Q7 — The dense-contract vs rupture tension (§3.3).** All five electoral goldens tune repression upward specifically because *"the worker revolts at tick 1 and Struggle severs its exploitation edge — a topology change the dense contract forbids."* The gate's static-topology assumption and the game's core mechanic are in direct conflict; the current resolution suppresses the mechanic. This is an architecture ruling (does the dense golden format need to admit topology change?), not a coefficient one.

**Q8 — Transcendental intrinsics: is §13 item 2 answerable by subtraction?**
The P27 open ruling asks polynomial-approximation vs pinned-libm for `sigmoid/exp/log/tanh/sqrt/entropy`. §2.7 argues that under full de-imposition the table plausibly reduces to `{sqrt}` — which is IEEE-754 correctly-rounded and needs no tolerance policy — making the ruling moot rather than answered. **If the Director finds that argument sound, the sequencing question follows immediately: does the theory review land before the intrinsic table is pinned by conformance vectors?** I am explicitly not proposing a schedule; I am flagging that the two decisions are coupled and that pinning the table first would freeze the imposed forms into the language contract.

**Q9 — The `inequality` hydration gap.** `SocialClass.inequality` is declared, typed `Gini`, required-read by VitalitySystem, and **seeded by nothing** — 0.0 across the canon. Whether this is (a) a wiring defect to repair independently of this review, (b) evidence that the imposed sigmoid *exists because* the distribution channel was never hydrated, or (c) both, is a judgement about project history I can document but not make.

---

## Files most relevant to any follow-up (absolute paths)

- `/home/user/projects/game/babylon/src/babylon/formulas/survival_calculus.py` — the core-line imposed form (111 lines, read in full)
- `/home/user/projects/game/babylon/src/babylon/engine/systems/survival.py` — its sole production consumer, via the hot-swappable registry seam
- `/home/user/projects/game/babylon/src/babylon/formulas/vitality.py` — the in-repo emergent-form precedent (Grinding Attrition)
- `/home/user/projects/game/babylon/src/babylon/models/entities/social_class.py` — `population` (:406), `inequality` (:411), `s_bio`/`s_class` (:386/:391), `county_fips` (:426)
- `/home/user/projects/game/babylon/src/babylon/domain/bifurcation/consciousness.py` + `/home/user/projects/game/babylon/src/babylon/config/defines/consciousness.py:459-486` — the behavior-tuned bifurcation cliff
- `/home/user/projects/game/babylon/src/babylon/formulas/class_dynamics.py:228` + `/home/user/projects/game/babylon/src/babylon/config/defines/economy_class.py:43-107` — the outcome-attractor spring
- `/home/user/projects/game/babylon/src/babylon/domain/economics/reserve_army/calculator.py:52-65` — the wage-pressure sigmoid
- `/home/user/projects/game/babylon/src/babylon/config/defines/politics.py` — the Θ-tier convention (ADR127), the estate's own cure for §4.4
- `/home/user/projects/game/babylon/docs/reference/bsl-language.rst` §2.7, §3.4, §4.3 — the intrinsic surface and the intensivity law
- `/home/user/projects/game/babylon/docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md` §6.4 (float-hazard inventory), §13 item 2 (the open transcendental ruling)
- `/home/user/projects/game/babylon/tests/unit/engine/laws/` — the convergence target for §3

---

## Appendix A — Minor findings (unverified tier)

- MINOR src/babylon/engine/scenarios/_legacy.py:628 [outcome-encoded-baseline]: Scenario seeding derives population from a hardcoded Gaussian metro kernel (sigma=2.0°) and rent from log1p(me
- MINOR src/babylon/domain/institution/balance.py:83 [tuned-coefficient]: Institutional internal contestation is min(1, 1 − max_weight + 0.1) with a hardcoded +0.1 floor offset.
- MINOR tests/unit/formulas/test_consciousness_routing.py:558 [tuned-coefficient]: test_chauvinist_pressure_can_flip_the_dominant_pole asserts fascist dominance (df > dr) at the specific magic
- MINOR tests/integration/mechanics/test_ideological_bifurcation.py:252 [outcome-assertion]: test_wage_cut_without_solidarity_amplifies_fascist_drift asserts an isolated worker's consciousness stays <= 0
- MINOR tests/integration/test_constant_hydration.py:100 [tuned-coefficient]: Asserts the hydrated `sigmoid_r0` estimate falls in [0.02, 0.15] — a plausibility window on a sigmoid paramete
- MINOR tests/unit/formulas/test_survival_calculus.py:14 [other]: Lane-1 handoff: the loss-aversion λ=2.25 (Kahneman-Tversky) and the P(S|A) sigmoid are imposed behavioral-econ
- MINOR src/babylon/config/defines/consciousness.py:147 [tuned-coefficient]: sustained_exploitation_sensitivity=0.02 is sized 'to avoid snapping consciousness to its 1.0 ceiling in a hand
- MINOR src/babylon/formulas/class_dynamics.py:205 [tuned-coefficient]: Duplicate magic-number signature defaults shadow the GameDefines values across several formula modules — dead
- MINOR src/babylon/config/defines/consciousness.py:31 [tuned-coefficient]: mass_awakening_threshold=0.6 is described as a 'target consciousness' for the MASS_AWAKENING event with no der
- MINOR tools/regression_test.py:1211 [other]: compare_baselines checks the `final_outcome` field specifically (SURVIVED/DIED), an outcome-flag comparison ra
- MINOR tests/unit/engine/systems/test_electoral_goldens.py:311 [outcome-assertion]: test_atomized_despair_routes_to_the_vehicle's closing assertion is vacuous — `fascist_alignment >= 0.0` on a [

## Appendix B — Raised and REFUTED (the adversarial pass killed these)

- REFUTED src/babylon/formulas/survival_calculus.py:90: calculate_crossover_threshold analytically inverts the imposed sigmoid (math.log(1/p_rev − — The finding fails as an independent defect at this site on three grounds. (1) No new stipulation: calculate_cr
- REFUTED src/babylon/domain/economics/reserve_army/calculator.py:52: Wage pressure is a stipulated bounded/baseline-normalized sigmoid of reserve_ratio (k=20,  — REFUTED on all three legs of the "imposed-form / short-circuited operation" charge.

(1) The finding's central
- REFUTED src/babylon/config/defines/economy_labor.py:62: sigmoid_k=20.0 and sigmoid_r0=0.08 are pure curve-shape parameters with no material deriva — The finding is refuted on its own terms. (1) sigmoid_r0 is not a pure curve-shape parameter: it is the natural
- REFUTED src/babylon/formulas/market.py:107: The CANONICAL price_value opposition Balance is tanh(log_ratio/scale) — an arbitrary satur — The finding is mathematically self-refuting. Its proposed 'principled' alternative (p−v)/(p+v) is IDENTICALLY
- REFUTED src/babylon/formulas/market.py:87: Price⟷value dynamics are stipulated as a linear damped-driven harmonic oscillator in log s — REFUTED. The finding's central charge — the oscillator "short-circuits" gravitation that would emerge from the
- REFUTED src/babylon/formulas/sustained_exploitation.py:198: Chauvinist agitation of the bribed strata is a stipulated Gaussian bump exp(-(balance-peak — REFUTED on four independent grounds after reading the site in full (src/babylon/formulas/sustained_exploitatio
- REFUTED src/babylon/formulas/reactionary.py:91: Cadre defection probability is sigmoid(chauvinism − discipline) with implicit unit steepne — The finding's factual predicate is wrong, and its preferred operation is what the code already computes one le
- REFUTED src/babylon/domain/bifurcation/consciousness.py:66: The bifurcation predictor's input weights are a sigmoid whose midpoint is explicitly 'Beha — REFUTED on three independent grounds, each verified against the code in full.

(1) LAYER MISATTRIBUTION — the
- REFUTED src/babylon/domain/bifurcation/consciousness.py:142: _CRISIS_FRAGILE_THRESHOLD = 0.3 is hardcoded inline (not a define) and directly stamps the — The finding fails on three independent grounds. (1) Factual mischaracterization: consciousness_weighted_solida
- REFUTED src/babylon/formulas/class_dynamics.py:228: National wealth distribution is spring-pinned to the empirically observed FRED-DFA outcome — The site is a materially-derived calibration boundary contract, not question-begging. (1) The empirical-invari
- REFUTED src/babylon/engine/field_registry.py:194: Imperial-rent field normalization stipulates a saturating exponential 10·(1−e^(−raw/10)) w — The finding's load-bearing claim — "this normalization curve is mechanically live" — is false. _normalize_impe
- REFUTED src/babylon/ooda/action_effects.py:95: The Shannon-entropy contestation measure — documented elsewhere as a read-only DIAGNOSTIC  — Refuted on four independent grounds. (1) Provenance inversion: the AGITATE-EDUCATE coupling is a ratified Feat
- REFUTED src/babylon/config/defines/survival.py:18: steepness_k=10.0 is self-described as 'Game design: sigmoid sharpness in acquiescence prob — REFUTED on six grounds. (1) Wrong clause: CONSTITUTION.md III.8 (Aleksandrov) scopes itself to "formalism rath
- REFUTED src/babylon/formulas/politics.py:68: Electoral hope (counterfactual_hope_gain, hence the whole hope_field → Allegiance/Electora — The site is a boundary contract, not an imposed form. counterfactual_hope_gain (formulas/politics.py:54-72) in
- REFUTED tests/unit/engine/systems/test_electoral_goldens.py:287: The Weimar golden asserts a specific political winner as the test subject: `assert first.w — The finding fails on all three of its load-bearing claims. (1) Not question-begging in the strict sense: the f
- REFUTED tests/unit/engine/systems/test_electoral_goldens.py:43: GOLDEN_OVERRIDES tunes coefficients per-scenario so each named historical outcome occurs — — REFUTED on six grounds, each independently damaging; jointly fatal. (1) The finding misquotes the tuning targe
- REFUTED tests/unit/engine/systems/test_electoral_goldens.py:160: Mitterrand and Syriza goldens assert the seated party identity and exact fiscal numbers as — REFUTED on three grounds. (1) The seated-party assertions are fixture-vehicle + clock contracts, not observed
- REFUTED tests/unit/formulas/test_survival_calculus.py:61: Pins P(S|A)'s sigmoid SHAPE — exact 0.5 crossing at threshold, steepness sharpening, and t — The finding's central claim — that these tests pin the logistic SHAPE and are "the enforcement mechanism" agai
- REFUTED tests/unit/bifurcation/test_consciousness.py:68: TestConsciousnessSigmoid pins the imposed consciousness_sigmoid's functional shape and its — The finding collapses on five independent grounds. (1) Not question-begging in the testing sense: every TestCo
- REFUTED tests/baselines/weimar.json:1: The five electoral-golden byte baselines (weimar.json, mitterrand.json, syriza.json, debs. — REFUTED on the finding's central factual premise: the baselines do not encode political outcomes. (1) The comp
- REFUTED src/babylon/config/defines/consciousness.py:459: A second sigmoid (beyond the sanctioned P(S|A) survival sigmoid) is imposed on bifurcation — The finding misidentifies the site's role and its severity collapses with it. (1) The core-loop claim is false

## Appendix C — Areas checked and found clean

- src/babylon/formulas/{balkanization,community,consciousness,constants,curvature,dynamic_balance,fundamental_theorem,lifecycle,metabolic_rift,solidarity,state_ai,trpf,unequal_exchange,vitality}.py — zero transcendental call sites (verified by precise rg sweep); linear/ratio/clamp algebra only; vitality's Grinding Attrition is linear-clamped, no curve stipulated
- src/babylon/domain/{dialectics,doctrine,organizations,politics}/ — zero transcendental call sites in the sweep; the reformist doctrine trunk moves through measured practice, not curves
- src/babylon/domain/economics/monetary/anchor.py:89 — math.log(ratio) is a coordinate change of an empirical FRED ratio into the oscillator's log space, with NoDataSentinel honest-absence guards on every degenerate input; derived quantity, not an imposed response
- src/babylon/engine/systems/contradiction.py:455 — financialization_index = math.exp(clamped fictitious_log) is the exact inverse transform of a log-ratio back to ratio space (documented: 'exp() returns it as the fictitious/real ratio directly'); derived quantity
- src/babylon/domain/economics/substrate/equalization.py:43 — math.ldexp(1.0, -1000) is an exact power-of-two overflow guard with a written rationale (subnormal cv producing inf); numeric guard, not a curve
- src/babylon/domain/economics/temporal/anomaly.py:66 — math.sqrt(variance) in rolling z-score; standard statistics on real data, method-selection is tiered and honest about data availability
- src/babylon/domain/geography/{snapping.py:116-193, nonlocal_edges.py:55-56} + src/babylon/engine/scenarios/_legacy_wayne.py:103-106 + src/babylon/data/game/balkanization/compute_seed_influences.py:114-117 — haversine/equirectangular spherical geometry (sin/cos/atan2/sqrt); exact geodesy on the immutable substrate
- src/babylon/engine/optimization/monte_carlo.py:169-193 — variance**0.5 in confidence-interval statistics; offline tooling, standard estimator math
- src/babylon/projection/topology/layout.py:58-59 — cos/sin circular layout; presentation only, never read back by the engine
- src/babylon/formulas/politics.py:145,227-228 — Euclidean norms for unit-normalizing platform/interest vectors (cosine-fit inner products); standard vector algebra on declared axes, and the Przeworski–Sprague dilution emerges from the composition rather than being scripted
- src/babylon/formulas/consciousness_routing.py:45,470 — normalized ternary entropy H/log3 explicitly fenced as a read-only diagnostic with a written argument for WHY it cannot carry the George Jackson asymmetry (§9.4 note; the asymmetry lives in the directed apply_fr_gate flow constraint, which is itself a materially-derived gate, not a potential function) — exemplary self-aware boundary; the boundary violation is at the entity twin's consumer (see finding at ooda/action_effects.py:95)
- src/babylon/formulas/market.py:36-58 — EMA anchor and relative-growth drive are declared arithmetic on flows with an honest-zero guard (III.11), no curve stipulated; correction snap/severity/overhang (:150-201) are linear/clamped identities on declared quantities
- src/babylon/formulas/contradiction.py:151 — (centrality_a·centrality_b)**0.5 geometric mean sits in a function explicitly deprecated by spec-lawverian-C1 with 'no production caller after Phase C'
- src/babylon/formulas/sustained_exploitation.py negative branch (:195-196) — linear -balance·sensitivity, no imposed curve on the exploited side
- web/game/engine_bridge.py exp(price_log)/exp(fictitious_log) — display-space inverse transforms in the LEGACY web client (Amendment V: failures don't gate; observes only)
- Formula registry seam (src/babylon/engine/formula_registry.py:106) — 'acquiescence_probability' is registered hot-swappable, so the core-line sigmoid has exactly one production injection point for an emergent replacement; verified SurvivalSystem consumes it via services.formulas.get, not a direct import
- tests/unit/engine/laws/ — ALL 17 files read line-by-line (ooda, reserve_army, transport, territory_system, collapse_transition, epistemic_horizon, metabolism, faction_influence, control_ratio, solidarity, community_system, substrate, dispossession_events, doctrine_system, decomposition_system, edge_transition, sovereignty): every law pins clamps/bounds, monotonicity, inactivity/no-op gates, conservation, state-machine closure, idempotence latches, or argmax soundness, each with file:line source grounding and explicit caveat sections refusing false laws (e.g. decomposition's rejected whole-graph conservation, doctrine's negative-tag caveat). Zero outcome trajectories pinned. This suite is the model the rest of the estate should converge to.
- tests/unit/engine/test_endgame_detector.py — recognizer-predicate contracts only; the RED_OGV reachability tests explicitly invoke the outcome-as-fixture-vehicle pattern (docstring cites the ruling), assert axis-gate predicates against hand-built graphs, and carry a companion negative test pinning the pre-repair blocked topology; pattern-dissolution and fraction-vs-count boundary tests are two-sided.
- tests/integration/mechanics/test_rupture_events.py — pins the condition-AND-level rupture gate in BOTH directions (static extreme gap does not rupture; falling gap above threshold does not rupture; rising gap above threshold does) — the theorem's stated form, with the atomization-dominates result asserted as a dialectical consequence, not a tuned outcome.
- tests/integration/test_induced_crisis.py — the best emergence-style scenario test found: intervenes ONLY on the material base (imperial_rent_pool=0, extraction_efficiency=0; docstring: 'touching NO StruggleSystem severing or consciousness gating'), then asserts the superstructural consequence disjunctively (RUPTURE OR LEVEL_TRANSITION), phase-contrasted against 20 pacified ticks that must NOT rupture — the Fundamental Theorem and its contrapositive as a falsifiable prediction.
- tests/unit/engine/systems/test_electoral.py — resolver-rule contracts: plurality winner, spoiler arithmetic, L-SUSPEND, disillusion routing decided by bridge topology; winners asserted only as arithmetic consequences of engineered vote inputs, never as tuned political trajectories.
- tests/unit/domain/politics/test_governance_endgame.py — pure predicate-table contracts (phi_share, betrayal_crossed, dual_power_live, arm resolution, rupture geometry) with boundary-inclusivity pinned; no simulation runs, no tuning. (The predicate table itself — no-organs-capitulates, bridges+Φ-starved=synthesis — is imposed political logic, but that is Lane-1 subject matter; the tests are honest pins of it.)
- tests/unit/projection/vault/test_epilogues.py, tests/unit/projection/test_endgame.py, test_briefing.py, tests/unit/game/test_pacing.py, test_session.py, tests/unit/tui/test_rust_host_m2.py, tests/unit/ai/test_prompt_builder.py, test_narrative_director.py — every GameOutcome reference is fixture data for projection/pacing/rendering/AI-formatting (vehicle usage per the ruling); epilogue tests pin content coverage and distinctness, not which outcome occurs.
- tests/integration/mechanics/test_control_ratio_crisis.py — threshold-predicate contracts of the coded revolution/genocide branch (org >= 0.5, boundary inclusive, once-only latches, delay gates); direction is the coded rule's contract, not tuned emergence. The hardcoded binary branch itself is Lane-1 material.
- tests/unit/engine/systems/test_survival.py — population-normalization contracts (per-capita equality, inactive-entity skip, zero-pop safety); no direction or outcome begging.
- tests/unit/formulas/test_survival_calculus.py (behavioral portions) and test_survival_calculus_properties.py (bounds/monotonicity properties) — bounds-in-[0,1], monotone-in-wealth, monotone-in-cohesion, inverse-in-repression, crossover-consistency assertions are genuine behavioral contracts that would survive a form change; only the shape pins reported above are findings.
- tests/unit/formulas/test_consciousness_routing.py (all but the flip test) — stage contracts: zero-input inactivity, additive combination, defines passthrough, simplex normalization, backward-compat identities, clamp-at-floor; the sign-agnostic wage_balance tests (ADR082) actively GUARD AGAINST a proscriptive quiescence-gating bug.
- tests/scenarios/test_endgame_flow.py (except lines 233/427) — max-ticks bounds, IN_PROGRESS continuation, observer protocol, determinism of repeated runs, `outcome != IN_PROGRESS` vehicle-compliant assertions.
- src/babylon/engine/systems/economic.py:689-717 — the bourgeoisie-decision call site routes ALL 12 thresholds/deltas through services.defines.economy; no live magic constants in the decision path.
- src/babylon/engine/systems/control_ratio.py:132-221 — revolution_threshold, control_capacity, and all delays read from defines.carceral; the revolution-vs-genocide fork is defines-routed (its 0.5 value is bare-labeled, but the routing is clean).
- src/babylon/engine/systems/survival.py:104-157 + src/babylon/formulas/survival_calculus.py — the sanctioned P(S|A)/P(S|R) pair: pure functions, steepness/subsistence routed through defines, P25 social-wage offset enters as data; the sigmoid here is the constitutional primitive, not an imposition on another mechanic.
- src/babylon/formulas/consciousness_routing.py — every sensitivity/scale coefficient reads from the defines object (d.*); no unrouted steepness or midpoints; the ternary-split entropy normalization uses log(3), pure math.
- src/babylon/engine/systems/contradiction.py:453-456 — the financialization exp() clamp bound comes from defines.market.max_abs_log; documented as corruption-guard, not tuning.
- src/babylon/engine/systems/distribution.py:17,69-83 — PIRT split defaults are explicitly declared unit-test scaffolds; production rates documented as derived from BEA REIS (FR-032).
- src/babylon/config/defines/politics.py (entire module) — the Θ_data/Θ_theory/Θ_feel tier convention (ADR127): every one of ~45 fields declares whether it is terrain fact, theory-fixed sign/bound, or pacing knob; Θ_feel is explicitly scoped to 'how long hope takes to die, never whether the ceiling exists' — the emergence-preserving discipline the rest of the defines estate lacks. Mitterrand/bernie_valve golden calibrations anchor VALUES to historical trajectories while theory fixes only SHAPE.
- Materially-derived exemplars verified: loss_aversion_lambda=2.25 (Kahneman-Tversky), solidarity_gain_per_uprising=0.2 (Pew 2020 George Floyd shift), activation_threshold=0.3 (percolation threshold for ⟨k⟩≈3-4 social graphs), consciousness sensitivity k=λ/(1−α)=0.1/0.2 (derivation shown), equity_factor (65%×0.6≈40% LA share), class_dynamics first-order alphas (fitted FRED DFA 2015-2025), transport neglect (HPMS-calibrated), ooda action_base_provide_service ([C-empirical] BPP), Sparrow observation ceilings.
- src/babylon/config/defines/endgame.py:77-95 — campaign_horizon_years and pattern_lock_ticks encode the emergent-endgames ruling in the data itself ('outcomes are recognized patterns, never terminators'); the detector estate is observes-only.
- src/babylon/formulas/reactionary.py:33-67,94-144 — calculate_fascist_pull, spontaneous_riot_risk, entitlement_effective: pure multiplicative/clamp forms, coefficients routed through ReactionaryDefines, Aleksandrov provenance documented (the one exception is the defection sigmoid, reported above).
- src/babylon/data/defines.yaml — verified generated-from-schema (descriptions mirror the Pydantic fields); no independent rationale layer hiding in the YAML; single moddable SoT holds.
- src/babylon/engine/systems/ position ClassVars (2.0, 9.5, 17.42, 21.5, ...) — ordering declarations, not tuned coefficients; field_derivative.py:229 is a standard second-difference stencil; wealth_distribution.py:157 kick/3.0 is a structural equal split.
- Dense byte-identity gate mechanics (tools/regression_test.py compare_dense_csv_bytes / compare_dense_trace): compares the whole per-tick trace byte-exactly with no outcome-field special cases — legitimate whatever-happens regression pinning; the sampled compare_checkpoints (:1150-1192) likewise compares 9 state variables with tolerance, not outcomes (the one final_outcome flag is reported separately).
- Terminal endgame outcomes never enter baselines: all 11 canon baseline JSONs record final_outcome=SURVIVED; no GameOutcome (REVOLUTIONARY_VICTORY/FASCIST_CONSOLIDATION/…) appears in any tests/baselines/** file — the five terminal outcomes are not byte-ratcheted anywhere.
- tests/scenarios/test_endgame_flow.py: compliant with the emergent-endgames ruling — the FASCIST_CONSOLIDATION assertions (:233, :427) run on a fixture that directly stamps the pattern (create_fascist_state, 9-of-10 national_identity>consciousness), and the class docstring (:191-198) explicitly states 'Babylon does not test for specific endgame outcomes… the fixture is only the cheapest vehicle'; the subject is detector/termination wiring, not simulated convergence.
- tests/scenarios/test_fascist_bifurcation.py: tests the constitutional bifurcation routing SYMMETRICALLY — the revolutionary path (solidarity ⟹ class_consciousness rises, :146) and the fascist path (:242) are asserted with equal weight as mechanism contracts of the ruled routing law, not one-sided outcome tuning.
- Vault estate (tools/vault_regression.py, tests/baselines/vault/*/manifest.json): hash-pins whole rendered pages for single_county and detroit_tri_county only — no electoral golden is vault-baked; grep of src/babylon/projection/ and vault/templates/ for conclusive political prose (inevitab*/always/must capitulate/cannot win/proves/confirms/betray) found only technical usages; epilogue pages are not in either manifest.
- Ceremony gates (tools/check_baseline_ceremony.py, tools/generate_ceremony_message.py): process/provenance enforcement only (trailer + drift table); no outcome fields are read or compared.
- COVERAGE_GAPS_DATA honesty mechanism (tools/regression_scenarios.py:2610-2750): uncovered systems are declared loudly with live-verified reasons and remediation paths rather than faked evidence rows; the probe method note (:290-300) records that plausible-but-unverified rows (RUPTURE, ECOLOGICAL_OVERSHOOT) were DROPPED rather than asserted — the mechanism is sound even though the gap CONTENT feeds the asymmetry finding.
- Scenario factories are pure literal constructors (electoral_goldens.py docstring: 'factories are pure constructors — every value below is a literal; contingency in-run is the engine's own seeded ξ_t'): no in-factory search/optimization loops tune toward outcomes at runtime; the tuning that occurred was offline and is DECLARED in ADR140, not hidden.
- Non-electoral e2e baselines (detroit-tri-county-5t.json, michigan-e2e.json, storage-budget-5t.json, mutation_baseline.json): scale/infrastructure fixtures with no named political outcome in their documented intent; detroit_tri_county's one bundle_field row (terminal_state.max_tension with forbidden_values 0.1/0.0, regression_scenarios.py:2017-2037) pins that the tension computation RAN, not any political conclusion.
- single_county / two_node / imperial_circuit registry descriptions ('4-node default scenario', 'Minimal worker vs owner', 'Wayne-seeded minimal county…') describe topology and exercised layers, not political endings — the outcome-naming pattern is confined to fascist_bifurcation and the five electoral goldens.
