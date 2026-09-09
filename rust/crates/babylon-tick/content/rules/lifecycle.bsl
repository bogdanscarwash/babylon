; LifecycleSystem (Material Base @7.0) — the D-P-D' circuit.
;
; Every county cycles population through three phases every tick: D
; (pre-productive, raised out of household/community labor), P (productive,
; sells labor-power), D' (post-productive, lives on the legitimation
; bargain — pensions, Social Security, home equity, the promise that a
; lifetime of production earns a floor under old age). The circuit computes
; population flow between the phases, the index that measures how credibly
; the D' promise is underwritten, and the ideology a new P-phase cohort
; inherits from its caregivers and its institutions.
;
; THIS PACK PORTS THREE OF THE FROZEN SYSTEM'S FIVE FLAT RULES (the R8 gap
; analysis's own count, reports/bsl-gap-analysis-2026-08-10.md row 7.0):
; DPD population flow, the legitimation index (+ crisis/recovery detection),
; and ideology transmission. Two do NOT land — inheritance flow and class
; mobility — for ONE shared reason, recorded precisely in WHAT DOES NOT LAND
; below. Both blocked rules are confirmed dead ends on the canonical graph
; today (grep-verified against dev tip 6c33b42c): `adjusted_p_to_d_prime`
; and `differential_p_to_d_prime` are read by no other system, and inheritance
; flow's only consumer is chronicle narration off an event, never simulation
; state (`LifecycleSystem.step()` calls no `graph.update_node` for the
; inheritance amounts at all — it is event-emission only). The three ported
; rules carry the system's load-bearing output: `legitimation_index` feeds
; `domain/bifurcation/analysis.py`, `engine/systems/struggle.py` and
; `engine/systems/electoral.py` directly.
;
; ============================================================================
; WHAT DOES NOT LAND, AND EXACTLY WHY
; ============================================================================
;
; `pareto_alpha` (inheritance Gini), `early_mortality_modifier` and
; `carceral_transition_modifier` (class mobility) are runtime-moddable
; LifecycleDefines coefficients whose declared domain is `(0, 10]` /
; `[0, 10]` — `src/babylon/data/defines.yaml:528,533,534`, each comment
; self-documenting the overflow ("Fed SCF... (> 0.0, <= 10.0)", "Chetty...
; (>= 0.0, <= 10.0)" twice). `bsl-language.rst` §1.5 caps every non-Currency
; scaled literal at `[0.0, 1.0]` (the `p`/`i`/`c` suffixes) and §3.1 declares
; "no coercions" — there is no legal `defconst` for a value like `1.5`,
; `2.8` or `1.24` that is not itself a dollar amount (`$`, which is also
; wrong: `Currency`'s own operator table, §3.2, admits no `Currency ×
; Currency`, so two such moddables could not even combine). This is the
; SAME construct gap the Territory BSL assessment (2026-08-11) named
; Blocker-1 and put behind director-gate #492: a runtime-moddable define
; with domain beyond `[0,1]` has no BSL representation today, full stop —
; not "multiplies Currency" specifically, "cannot be written at all"
; generally. Rounding `pareto_alpha` to the nearest int-storable value would
; not dodge the gap; it would silently change `compute_pareto_gini`'s
; output by construction (`1/(2·1.5−1) = 0.5` vs `1/(2·1−1) = 1.0`), which
; the port-as-is directive and the no-silent-degradation standard both
; forbid. Per §6 of `docs/superpowers/plans/2026-08-10-vitality-bsl-rule-
; pack.md`, this port transcribes no formula and invents no substitute —
; it waits on #492's resolution, same as Territory.
;
; What that blocks precisely:
;   - `DefaultInheritanceCalculator.compute_inheritance_flow` (`domain/
;     economics/lifecycle/inheritance.py:108-140`) — `compute_pareto_gini`
;     needs `pareto_alpha`. Blocks the `INHERITANCE_TRANSFER` event only;
;     the frozen system writes no graph state from this branch at all.
;   - `DefaultClassMobilityCalculator.compute_premature_exit_rate` and
;     `DefaultCohortDynamicsCalculator.apply_differential_rates` (system
;     Steps 6-7) — both read `early_mortality_modifier`/
;     `carceral_transition_modifier` off `ClassMobilityParams`. Blocks
;     `adjusted_p_to_d_prime` and `differential_p_to_d_prime`, confirmed
;     dead-end fields (see header).
;
; ============================================================================
; MODELING CHOICE — D-1: the five constant-in-practice inputs become `:const`
; ============================================================================
;
; `LegitimationState`'s five components (`pension_coverage`,
; `ss_replacement_rate`, `healthcare_security`, `home_ownership_rate`,
; `retirement_confidence`) and `DPDState`'s four rate fields (`rate_d_to_p`,
; `rate_p_to_d_prime`, `rate_d_prime_to_death`, `birth_rate`) are read
; per-node-with-defines-fallback in the frozen Python
; (`LifecycleSystem._read_legitimation_state`, `LifecycleSystem.step` lines
; 85-102). But nothing else in the tree ever writes divergent per-territory
; values for any of these nine fields (grep-verified: `legitimation_state`,
; `caregiver_ideology`, `institutional_hegemony`, `community_tendency` and
; `mobility_params` are written ONLY by `LifecycleSystem` itself, and
; `compute_transitions`/`apply_differential_rates` always echo the DPDState
; rate fields back unchanged) — so on tick 1 EVERY territory falls into the
; "initialize from defines" branch, and every tick after that reads back the
; SAME defines-derived value it started with. The per-node storage never
; observably diverges from "read the global constant" in the shipped
; engine. This pack transcribes the OBSERVABLE behavior — one value, shared
; by every territory — as `:const` bindings, which is also the only legal
; move: `bsl-language.rst`'s known constraint that "the slice-1 scenario
; loader seeds ONLY int-declared node attributes" (`scenario.rs::
; attribute_value`) refuses a fractional per-node seed outright, and these
; nine values are fractional by nature. If per-county legitimation or rate
; data is ever wired for real (it is not today — a genuine gap, independent
; of this port), that is new content work, not a retrofit of this rule.
; Mirrors D-2's dead-branch reasoning in the Vitality plan.
;
; `caregiver_ideology` and `institutional_hegemony` are a sharper case of
; the same thing: they are not even `LifecycleDefines` fields. Python reads
; them with a bare inline default of `0.5` (`engine/systems/lifecycle.py:
; 191-192`) that no `defines.yaml` entry backs, and — per the same grep —
; nothing ever writes them either, so they are permanently `0.5` in the
; shipped engine. Recorded as **D-2**: a modding-contract gap in the frozen
; engine (the same class as Vitality's D-3), transcribed as `:const`
; bindings this pack names `lifecycle/caregiver-ideology-default` and
; `lifecycle/institutional-hegemony-default` rather than reading a
; `defines.yaml` line that does not exist. `community_tendency` is always
; absent in production (nothing writes it, confirmed by the same grep), so
; its amplification term (`cohort_dynamics.py:214-215`) is a dead branch
; and this pack drops it — recorded as **D-3**, mirroring the Vitality
; plan's D-2.
;
; ============================================================================
; §5.4 DEFECT REPAIR — D-5: the crisis/recovery edge check is case-broken
; ============================================================================
;
; `engine/systems/lifecycle.py:146,157` compares `prev_crisis` — read off
; the graph, holding whatever a PRIOR tick wrote via `crisis_class.value`
; — against the LITERAL string `"CRISIS"`. But
; `LegitimationClassification` is a `StrEnum` whose values are lowercase
; (`models/enums/legal.py:24-26`: `CRISIS = "crisis"`), so `prev_crisis`
; is always `"crisis"`, `"unstable"`, `"stable"` or `None` — **never**
; `"CRISIS"`. Concretely: `prev_crisis != "CRISIS"` is true on every tick
; regardless of the actual previous state, so `LEGITIMATION_CRISIS` fires
; EVERY tick a territory classifies CRISIS rather than only on the
; transition into it; and `prev_crisis == "CRISIS"` is false on every
; tick, so `LEGITIMATION_RECOVERY` is permanently dead code. No test in
; the frozen estate exercises either branch
; (`rg LEGITIMATION_CRISIS|LEGITIMATION_RECOVERY tests/integration/
; test_lifecycle_system.py` — no hits), which is how this went unnoticed.
;
; This pack's crisis classification has no string-case axis to inherit the
; bug through — `LegitimationClassification` has no BSL representation
; (see Block 2 below), so it is encoded as an `int` this pack controls
; both ends of. Writing the SAME case-mismatch bug back in would mean
; deliberately comparing the int encoding against the wrong constant on
; purpose, which is unauditable nonsense, not a transcription. **Repair:**
; this pack implements genuinely edge-triggered semantics — `!= 2` vs `= 2`
; where both sides share one encoding — which is what the frozen code's
; own structure and comment ("Emit crisis/recovery events") plainly
; intend. This is a real, deliberate behavioral difference from the
; frozen engine for the TWO EVENT TYPES ONLY; every state field this pack
; writes (`legitimation-index`, `legitimation-crisis` itself, and
; everything Blocks 1/3 touch) is unaffected and matches the frozen engine
; exactly. The conformance test asserts the correct edge-triggered firing
; pattern rather than replaying the frozen engine's buggy event log for
; these two event types — see `lifecycle_conformance.rs`'s header for the
; exact vectors on both sides.
;
; ============================================================================
; MODELING CHOICE — D-4: three saturating clamps are provably redundant
; ============================================================================
;
; Three places the frozen Python defensively clamps a value this pack
; leaves unclamped, because the clamp cannot fire given the declared
; domains of its own inputs (all rates/fractions here are `[0,1]`-bounded
; `c`-literals, checked at LOAD by `bsl-language.rst` §1.5's `E-LEX-024`,
; and every population/wealth field is non-negative by construction):
;
;   1. `compute_population_flow`'s three `max(0.0, …)` calls
;      (`formulas/lifecycle.py:57-59`). `d_to_p = rate_d_to_p × pop_d` with
;      `rate_d_to_p ∈ [0,1]` gives `d_to_p ≤ pop_d`, so
;      `(pop_d + births) − d_to_p ≥ births ≥ 0` always — same argument for
;      the other two. §3.10's rider slate declines a scalar min/max
;      operator precisely so a saturation stays legible in the source
;      (the reason Vitality's plan gives for spelling its own clamp as an
;      `if`); here there is nothing to spell, because the saturation never
;      triggers.
;   2. `wealth_d_prime`'s `surviving_fraction = max(0.0, 1.0 − deaths /
;      old_total)` (`cohort_dynamics.py:151`) — `deaths ≤ old_total` by the
;      same rate-bound argument, so the fraction is always in `[0,1]`.
;   3. `compute_legitimation_index`'s `max(0.0, min(1.0, index))`
;      (`formulas/lifecycle.py:140`) — the five `legit_w_*` weights sum to
;      exactly `1.0` at their current `defines.yaml` values (`0.35 + 0.30 +
;      0.20 + 0.10 + 0.05`), and each component is a `[0,1]`-bounded `c`
;      literal, so the weighted sum is a convex combination and cannot
;      leave `[0,1]`.
;
; Recorded honestly: this is a fact about the CURRENT `defines.yaml`
; values, not a law the type system enforces. A mod that changes the
; `legit_w_*` weights to no longer sum to `1.0`, or a modder editing the
; rate defines to something the `c`-literal load-time check would still
; accept, could in principle break case 3 (case 1/2 hold for ANY `[0,1]`
; rate, so they are safe under any legal mod). Python's `max`/`min` would
; silently absorb that; this pack's `int`-declared output fields carry no
; automatic range check at all (§3.3's store-boundary check is specific to
; `Probability`/`Coefficient`/`Intensity`-declared fields, and every
; per-node field in this pack is `int` per the seeding constraint above),
; so an out-of-range legitimation index would silently store instead of
; failing loudly. This is a modding-contract gap inherited from the
; int-workaround every slice-1 rule pack needs, not introduced by this
; port — recorded so a later reader does not have to re-derive it.
;
; ideology transmission's own clamp (`cohort_dynamics.py:217`,
; `max(0.0, min(1.0, raw))`) is redundant by the identical convex-
; combination argument: `caregiver_weight + institutional_weight = 1.0`
; (`0.7 + 0.3`) blends two `[0,1]` inputs, and the regression step
; `raw·(1−r) + 0.5·r` blends `raw` (now proven `[0,1]`) with the constant
; `0.5` by weights that also sum to `1.0`.
;
; verify_conservation (system Step 2) only calls `logger.warning` — no
; graph write, no event. BSL's eight structural verbs (§2.8) include no
; logging verb, so this step has no BSL transcription and is dropped
; without a behavioral difference (nothing it does is observable).
;
; ============================================================================
; TRANSCRIPTION DEVIATION — D-6: the regression-step guard is elided
; ============================================================================
;
; `cohort_dynamics.py:209-211` guards the regression-toward-mean step with
; `if r > 0.0: raw = raw * (1.0 - r) + 0.5 * r`, skipping the update
; entirely at `r == 0.0`. Block 3's `transmitted` binding applies the
; formula unconditionally. This is not case 4 of D-4's redundant-clamp
; argument (there is no saturation here to prove away) — it is a genuine
; transcription deviation, recorded because §5.4 asks for one whenever the
; BSL differs in STRUCTURE from the frozen code, even when the two are
; behaviorally identical everywhere the guarded value can occur. And they
; are identical everywhere on `r`'s declared domain (`ideology_regression_
; coefficient`, `[0,1]`, `defines.yaml:542`), including the boundary the
; guard exists for: at `r = 0.0`, `raw * (1.0 - 0.0) + 0.5 * 0.0 = raw`,
; the exact value the skipped branch would have left in place — a
; multiply-by-zero-and-add-zero is an identity in IEEE-754, not an
; approximation. The guard only has observable effect for `r < 0.0`, which
; `ideology-regression-coefficient`'s declared `c`-literal range (`[0,1]`,
; §1.5) cannot express in the first place.
;
; ============================================================================
; ENGINE MACHINERY — the anchor's registered-system set, and one rule not three
; ============================================================================
;
; `babylon-tick/src/lib.rs`'s `run_once_into` hardcodes the driver's
; registered-system set at `{economics, vitality, consciousness}` (Vitality
; landed the second of those). This port adds `lifecycle`, the same class
; of minimal, precedented change Vitality made to `tick.rs` for `:const`
; serving — driver scaffolding, not rule content, not BSL grammar.
;
; **One rule, not three — a slice-1 driver limit, not a modeling choice.**
; `run_once`/`run_once_into`'s `split_content` accepts exactly one `(rule …)`
; top-form per content set ("a content set needs exactly one (rule …)
; top-form", `rule_pipeline.rs`); Vitality never tested the other case
; because its own three phases already collapsed into one rule for an
; independent reason (§4.2's same-pre-state rule). This pack's three
; material processes ARE independent — none reads another's binding or
; effect — so nothing here is a re-derivation the way Vitality's phases
; were; they are grouped into one `(rule lifecycle/dpd-circuit …)` form
; purely because slice 1 has no multi-rule pack runner yet. Each of the
; three material processes below is commented as its own block and could
; split back into separate rules the moment the driver supports more than
; one per tick.

(rule lifecycle/dpd-circuit
  :role mechanic
  :evidence derived
  :material-basis "a county's population moves through three material phases every tick — pre-productive (raised out of household labor), productive (sells labor-power), post-productive (lives on the legitimation bargain) — at rates set by birth and mortality data and by the age structure of production; a cohort that dies takes a proportional share of whatever wealth its members held with it; the D' bargain's credibility and the ideology a new productive cohort inherits are computed alongside it every tick"
  :fuel 3072
  ; No explicit anchor: the executable registry resolves the `lifecycle`
  ; rule-id namespace to Lifecycle @7. `(anchor :after vitality)` would mean
  ; the literal boundary immediately after Vitality @1, not "somewhere later
  ; than vitality," and would illegally cut through Material Base.
  (bindings
    ; --- Block 1: DPD population flow (formulas/lifecycle.py:16-61,
    ; domain/economics/lifecycle/cohort_dynamics.py:129-163) ---
    (binding pop-d :field territory/pop-d)
    (binding pop-p :field territory/pop-p)
    (binding pop-d-prime :field territory/pop-d-prime)
    (binding wealth-d-prime :field territory/wealth-d-prime)
    (binding birth-rate :const lifecycle/birth-rate)
    (binding rate-d-to-p :const lifecycle/rate-d-to-p)
    (binding rate-p-to-d-prime :const lifecycle/rate-p-to-d-prime)
    (binding rate-d-prime-to-death :const lifecycle/rate-d-prime-to-death)
    ; compute_population_flow, the frozen engine's exact association order
    ; (`formulas/lifecycle.py:52-59`): births/transitions first, then each
    ; new population as (old + inflow) - outflow, left to right.
    (binding births :expr (* birth-rate pop-p))
    (binding d-to-p :expr (* rate-d-to-p pop-d))
    (binding p-to-d-prime :expr (* rate-p-to-d-prime pop-p))
    (binding deaths :expr (* rate-d-prime-to-death pop-d-prime))
    (binding new-pop-d :expr (- (+ pop-d births) d-to-p))
    (binding new-pop-p :expr (- (+ pop-p d-to-p) p-to-d-prime))
    (binding new-pop-d-prime :expr (- (+ pop-d-prime p-to-d-prime) deaths))
    ; `cohort_dynamics.py:150-152`: the surviving fraction only applies when
    ; there was a D' cohort to begin with AND deaths actually occurred;
    ; otherwise wealth is unchanged. The else-branch is not a stray
    ; computation — an `if`'s two branches must share one static type
    ; (E-TYPE-020), and a bare `wealth-d-prime` reference (kind `int`,
    ; unpromoted) would not match the Real-typed `if`-branch below it.
    ; PREVIOUSLY spelled `(- 1 0)`: WRONG, caught by Copilot review on
    ; PR #493 — `Int - Int` does NOT promote to `Real`. §3.3's promotion
    ; only fires when an `Int` operand appears ALONGSIDE a genuine
    ; binary64-float type (`Probability`/`Intensity`/`Coefficient`/`Real`)
    ; in one arithmetic form; `1` and `0` alone are both plain `Int`
    ; literals, so `(- 1 0)` stays `Int`, mismatching the `Real`
    ; then-branch under E-TYPE-020 (`score_class.rs`'s coarse `Scalar`
    ; class does not distinguish `Int` from `Real` and so did not catch
    ; this; the normative rule at `bsl-language.rst:1070` does). Copilot's
    ; own suggested `1c` is equally wrong in the other direction: a BARE
    ; `Coefficient` literal, not mixed with anything, stays typed
    ; `Coefficient` — a third static type, matching neither branch. `(- 1
    ; 0c)` mixes a genuine `Coefficient` operand into the subtraction,
    ; which promotes the WHOLE form — including the `Int` `1` — to `Real`,
    ; matching the then-branch exactly. `0c` is canonical zero (§1.5), so
    ; the value is unchanged: `1.0 - 0.0 = 1.0` exactly, still an identity
    ; multiply below, not an approximation.
    (binding surviving-fraction :expr
      (if (and (> pop-d-prime 0) (> deaths 0))
          (- 1 (/ deaths pop-d-prime))
          (- 1 0c)))
    (binding new-wealth-d-prime :expr (* wealth-d-prime surviving-fraction))
    ; `DPDState.dependency_ratio`, computed off the NEW (post-transition)
    ; populations, matching `new_state.dependency_ratio` in
    ; `engine/systems/lifecycle.py:124`. Undefined at `new-pop-p == 0` —
    ; the frozen Python special-cases that as `math.inf`
    ; (`domain/economics/lifecycle/types.py:84-85`), which `Real`'s
    ; "binary64, finite" domain (§3.1) cannot represent at all.
    ; PREVIOUSLY left unguarded on the theory that every conformance
    ; fixture's post-tick `pop-p` stays strictly positive so the case is
    ; out of scope — Director ruling (popup 2026-08-18, BSL Hygiene
    ; Knock-out Task W5): GUARD IT, not a loud invariant. A fully
    ; liquidated productive class (`new-pop-p == 0`) is a reachable
    ; material state — the D-P-D' circuit itself can drive it there in one
    ; tick, same as a county with no D-phase inflow to replace a vanished
    ; P cohort — not a programming error, so the honest behavior is a
    ; defined inert value, never `E-EVAL-012` aborting the tick. Guarded
    ; with this pack's own `surviving-fraction` idiom above (:303-306: one
    ; guard condition, the real computation as the then-branch, an inert
    ; else) — the identical `(if (> … 0) (/ … …) (- 0 0c))` shape
    ; `production.bsl`'s `bio-ratio` binding uses for the same
    ; guarded-ratio case. `(- 0 0c)` is Real zero (§3.3 promotion, the
    ; same trick :286-301 above derives for `(- 1 0c)`) AND kind-neutral
    ; (§3.4: "a literal is kind-neutral"), so it cannot straddle kinds
    ; with the then-branch (E-TYPE-040) regardless of what kind
    ; extensive-over-extensive division ends up carrying once §3.4's
    ; `*`/`/` kind rule is enforced — not live today (`typecheck.rs`'s own
    ; module doc: "E-TYPE-040 kind mixing" arrives "in later tasks").
    ; PIN LAW: every pre-existing conformance vector keeps post-tick
    ; `pop-p` strictly positive, so this branch is UNREACHABLE on every
    ; landed pin — exactly why the gap stayed latent until now.
    (binding dependency-ratio :expr
      (if (> new-pop-p 0)
          (/ (+ new-pop-d new-pop-d-prime) new-pop-p)
          (- 0 0c)))

    ; --- Block 2: legitimation index + crisis/recovery detection
    ; (formulas/lifecycle.py:89-140,
    ; domain/economics/lifecycle/legitimation.py:99-128) ---
    (binding home-ownership-rate :const lifecycle/home-ownership-rate)
    (binding healthcare-security :const lifecycle/healthcare-security)
    (binding retirement-confidence :const lifecycle/retirement-confidence)
    (binding pension-coverage-rate :const lifecycle/pension-coverage-rate)
    (binding ss-replacement-rate :const lifecycle/ss-replacement-rate)
    (binding w-home :const lifecycle/legit-w-home-ownership)
    (binding w-health :const lifecycle/legit-w-healthcare-security)
    (binding w-retire :const lifecycle/legit-w-retirement-confidence)
    (binding w-pension :const lifecycle/legit-w-pension-coverage)
    (binding w-ss :const lifecycle/legit-w-ss-replacement)
    (binding crisis-threshold :const lifecycle/legitimation-crisis-threshold)
    (binding unstable-threshold :const lifecycle/legitimation-unstable-threshold)
    ; The PRE-tick classification, read before this rule overwrites it —
    ; needed to detect a crisis/recovery EDGE, not just a level.
    (binding prev-crisis :field territory/legitimation-crisis)
    ; `compute_legitimation_index`'s exact left-to-right association
    ; (`formulas/lifecycle.py:133-139`).
    (binding legit-index :expr
      (+ (+ (+ (+ (* w-home home-ownership-rate)
                  (* w-health healthcare-security))
               (* w-retire retirement-confidence))
            (* w-pension pension-coverage-rate))
         (* w-ss ss-replacement-rate)))
    ; `LegitimationClassification` has no BSL representation (§3.1's six
    ; `deffield`-able types exclude every `Enum<T>`; a custom content enum
    ; cannot be a field's static type). Encoded as this pack's own
    ; convention: 0 = STABLE, 1 = UNSTABLE, 2 = CRISIS — matching
    ; `classify_crisis`'s exact threshold ladder
    ; (`legitimation.py:118-128`): index < crisis_threshold -> CRISIS,
    ; elif index < unstable_threshold -> UNSTABLE, else STABLE. Both `if`
    ; branches at every level are bare Int literals (no arithmetic
    ; operator touches them), so they share one static type without
    ; needing the Real-promotion trick the population rule above needs.
    (binding new-crisis-class :expr
      (if (< legit-index crisis-threshold)
          2
          (if (< legit-index unstable-threshold) 1 0)))

    ; --- Block 3: ideology transmission (formulas/lifecycle.py:168-196,
    ; domain/economics/lifecycle/cohort_dynamics.py:193-217) ---
    ; D-2: hardcoded Python literals with no `defines.yaml` backing
    ; (`engine/systems/lifecycle.py:191-192`) — see the pack header.
    (binding caregiver-ideology :const lifecycle/caregiver-ideology-default)
    (binding institutional-hegemony :const lifecycle/institutional-hegemony-default)
    (binding caregiver-weight :const lifecycle/ideology-caregiver-weight)
    (binding institutional-weight :const lifecycle/ideology-institutional-weight)
    (binding regression-coefficient :const lifecycle/ideology-regression-coefficient)
    ; `formulas/lifecycle.py:196` then `cohort_dynamics.py:209-211`. D-3:
    ; the community-tendency amplification term
    ; (`cohort_dynamics.py:214-215`) is dropped — always dead in production,
    ; see the pack header.
    (binding raw :expr
      (+ (* caregiver-weight caregiver-ideology)
         (* institutional-weight institutional-hegemony)))
    (binding transmitted :expr
      (+ (* raw (- 1 regression-coefficient))
         (* 0.5c regression-coefficient))))
  (effects
    ; Block 1 writes + event.
    (update-node self territory/pop-d (set new-pop-d))
    (update-node self territory/pop-p (set new-pop-p))
    (update-node self territory/pop-d-prime (set new-pop-d-prime))
    (update-node self territory/wealth-d-prime (set new-wealth-d-prime))
    (update-node self territory/dependency-ratio (set dependency-ratio))
    (emit EventType/LIFECYCLE_TRANSITION
      (territory-id self)
      (pop-d new-pop-d)
      (pop-p new-pop-p)
      (pop-d-prime new-pop-d-prime)
      (dependency-ratio dependency-ratio))
    ; Block 2 writes + crisis/recovery events. `engine/systems/lifecycle.py:
    ; 144-167`: two mutually exclusive edges (a territory cannot enter
    ; CRISIS and STABLE in the same tick), so two independent guards
    ; reproduce the frozen `if`/`elif` exactly.
    (update-node self territory/legitimation-index (set legit-index))
    (update-node self territory/legitimation-crisis (set new-crisis-class))
    (guard (and (= new-crisis-class 2) (!= prev-crisis 2))
      (emit EventType/LEGITIMATION_CRISIS
        (territory-id self)
        (legitimation-index legit-index)))
    (guard (and (= new-crisis-class 0) (= prev-crisis 2))
      (emit EventType/LEGITIMATION_RECOVERY
        (territory-id self)
        (legitimation-index legit-index)))
    ; Block 3 write.
    (update-node self territory/transmitted-ideology (set transmitted))))
