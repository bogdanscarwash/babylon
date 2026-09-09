; MetabolismSystem (Material Base @13.0) — the metabolic rift between
; extraction and regeneration.
;
; Every tick, a territory's biocapacity regenerates toward its ceiling and
; is depleted by extraction, at an entropic loss (extraction always costs
; more than it yields — the thermodynamic point behind `entropy_factor`).
; Extraction also PERMANENTLY damages the ceiling itself (the Epoch 1
; hysteresis doctrine, "The Earth Remembers") — the earth cannot recover
; its original ceiling even if extraction stops entirely.
;
; THIS PACK PORTS ONLY PHASE 1 of the frozen system (per-territory
; biocapacity delta + hysteresis ratchet + double clamp) — the gap report's
; own verdict on this system: "Per-territory biocapacity/hysteresis rule
; plus one graph-scoped overshoot rule; blocked on graph-scoped evaluation
; only" (`reports/bsl-gap-analysis-2026-08-10.md` row 13.0). The spec-070
; sovereign pre-pass and Phases 2-3 (global overshoot aggregate +
; `ECOLOGICAL_OVERSHOOT`) are BLOCKED — D-3/D-4 below, full derivation in
; `reports/metabolism-port-assessment-2026-08-11.md`.
;
; ============================================================================
; §5.4 FINDING — D-1: `entropy_factor` cannot be a `Ratio` for THIS formula;
; scaled-Int workaround, open language-surface finding
; ============================================================================
;
; The obvious move — `(defconst metabolism/entropy-factor 1.2r :floor 1r
; :cap 3r)`, `bsl-language.rst` Draft-Ruling Register row D99's OWN worked
; example (#492/ADR194, `currency_scale_op_e2e.rs`) — DOES NOT WORK for
; `calculate_biocapacity_delta`'s actual arithmetic. `Ratio` has exactly one
; legal operator, `Currency x Ratio -> Currency` (D99: "Ratio gets exactly
; this one operator and no other"), confirmed in `evaluator.rs::apply_arith`
; — every `Value::Ratio` operand not paired with a `Value::Currency` falls
; to the generic `real_lane` catch-all, which does not recognize
; `Value::Ratio` at all and refuses outright. This formula's multiplicand,
; `raw_extraction = extraction_intensity * current_biocapacity`
; (`metabolic_rift.py:47`), is NEVER `Currency` — both factors are `:field`
; reads (D-2 below explains why `extraction_intensity`/`biocapacity` must
; be per-node fields, not consts), and slice 1's `scenario.rs::
; attribute_value` refuses to store ANY field as anything but
; `int`-declared — there is no Currency-typed field storage to route
; through. `Currency` can only enter a rule as an inline literal
; (`currency_scale_op_e2e.rs`'s own module doc), never as the result of a
; field read. So `Real x Ratio` is the only shape available, and no such
; operator exists.
;
; This is not a new gap this port invented: `reports/bsl-gap-analysis-2026-
; 08-10.md`'s Appendix item 3, written ONE DAY BEFORE D99 landed, already
; named it precisely — "The residual gap is real only for a runtime-valued
; `:const` outside `[0,1]` (`entropy_factor`, domain `(1.0, 3.0]`), which
; authors cannot split at load time." D99 closes director-gate #492 for the
; shape its OWN worked example tests (`Currency x Ratio`); `entropy_factor`'s
; REAL consumer needs `Real x Ratio`, a shape D99 never added an operator
; for — apparently on the assumption (matching `biocapacity`'s `Currency`
; type annotation in the Python Pydantic model, `territory.py:155-163`)
; that the multiplicand would itself be `Currency`-typed in BSL, which it
; is not and cannot be in slice 1.
;
; **Workaround, not a resolution — and NOT bit-exact against the frozen
; engine. Declared, deterministic deviation (term-bounded, output-UNBOUNDED
; under cancellation — see the round-2 correction below), corrected here
; after adversarial review found the original "exact ... for ANY legal
; value" claim FALSE by execution.** `entropy_factor` is declared as a scaled
; bare-`Int` `:const` — `(defconst metabolism/entropy-factor-x1e6
; 1200000)`, `x1,000,000` — and divided back out inline (`ecological-cost`
; below). This is the SAME escape hatch Dispossession's own D-2/D-4 already
; use and document: a bare, unsuffixed `Int` `:const` carries NO domain
; check at all (`E-LEX-024` only bounds SCALED/suffixed literals).
;
; **The frozen engine computes `raw_extraction * entropy_factor` as ONE
; binary64 multiply.** This pack computes `(raw_extraction *
; entropy_factor_x1e6) / 1000000` — this pack's own inner multiply, then a
; correctly-rounded division. Both are the SAME real-valued function; they
; are DIFFERENT floating-point PROGRAMS for it, and correctly-rounded
; operations composed in a different order are not guaranteed to agree.
; **Round 2 correction (adversarial re-verification of the F1 fix round
; found this restatement itself bounded the WRONG quantity and introduced
; two further false claims — corrected here, not merely reworded):**
;
;   1. **Grid quantization** (dominant for an arbitrary modded value). A
;      content author writes `entropy-factor-x1e6` as `round(entropy_factor
;      x 1,000,000)` — an integer, so any TRUE value needing more than 6
;      decimal digits is quantized to the nearest 1e-6, an absolute error
;      up to `5e-7`. `MetabolismDefines.entropy_factor` is a plain `float`
;      with no stated digit-count limit, so this is a real, not merely
;      hypothetical, degradation for SOME legal `(1.0, 3.0]` values — the
;      shipped default `1.2` (and any value exactly representable in <= 6
;      decimal digits) has ZERO quantization error, `1200000 / 1e6 == 1.2`
;      exactly as a real number.
;   2. **Double rounding bounds the `ecological-cost` TERM, NOT this rule's
;      final output.** This pack's `ecological-cost` is a SINGLE
;      correctly-rounded division of an EXACT numerator whenever the inner
;      product `raw_extraction * entropy_factor_x1e6` fits `f64`'s 53-bit
;      significand EXACTLY — a fact about the PRODUCT's magnitude, not
;      about either operand's (an earlier revision claimed "both operands
;      fit well inside 2^53", which is not the relevant condition: two
;      values each under 2^53 can still multiply to a product whose
;      SIGNIFICAND does not fit, if the product itself needs more than 53
;      significant bits — not an issue at the magnitudes this system's
;      conformance fixtures use, but the justification needed to name the
;      actual constraint). Under that condition, `ecological-cost` is
;      DERIVABLY within 2 ULP of the frozen engine's own
;      `raw_extraction * entropy_factor_f64` (a relative-error propagation
;      argument: the real number this pack's division rounds and the real
;      number the frozen engine's multiply rounds differ by a factor of
;      at most `2^-53`, from `entropy_factor_f64`'s own rounding error off
;      the decimal literal — two independent correctly-rounded results of
;      inputs that close land within a small constant number of ULP of
;      each other). **Measured**, over an exhaustive sweep of the
;      `ecological-cost` TERM alone (int `raw_extraction` 1-2000 x every
;      point of the 6-digit `entropy_factor` grid in `(1.0, 3.0]`
;      — 2,000,000 grid points, 4,000,000,000 combinations total): the
;      worst observed deviation is exactly 1 ULP, zero combinations exceed
;      it.
;
; **The rule's OUTPUT (`biocapacity`) is NOT ULP-bounded, and an earlier
; revision of this record implied it was by measuring the wrong quantity
; — `ecological-cost`'s own ~1 ULP deviation feeds into a SUBTRACTION
; against a comparable-magnitude term (`current + delta`) under
; cancellation, then a clamp, and cancellation is exactly the operation
; that turns a tiny relative error into an arbitrarily large ONE.**
; Measured counterexamples, both engines, at seeds no more exotic than
; this pack's own conformance fixtures:
;
;   - A: `biocapacity=149`, `max_biocapacity=150`, `extraction_intensity=1`,
;     `regeneration_rate=0.005`, `entropy_factor=1.005` — frozen engine
;     `0.005000000000023874` (`0x3f747ae147ae8000`) versus this pack
;     `0.0049999999999954525` (`0x3f747ae147ae0000`) — **32768 ULP
;     (2^15) apart**, a relative difference of `5.7e-12`, from a `~1 ULP`
;     input deviation.
;   - B: `biocapacity=1`, `max_biocapacity=10`, `extraction_intensity=3`,
;     `regeneration_rate=0.26`, `entropy_factor=1.2` (the SHIPPED
;     DEFAULT) — frozen engine `4.440892098500626e-16` versus this pack
;     `0.0`: the two engines land on OPPOSITE SIDES of the
;     `max(0.0, ...)` floor. The clamp-branch split is not a corner case
;     invented for this record — a broad round-value sweep found 305 such
;     splits.
;
; **This record's job is therefore not "the deviation is small" — it is
; "the deviation is UNBOUNDED in general, deterministic on each side, and
; accepted anyway under ADR183/III.7" (below), never silently denied.**
;
; **Tick-1 exactness is a special case, not the general rule.** On tick 1
; with every field freshly int-seeded, `raw_extraction` is itself an exact
; integer, which is what makes the inner-product exactness of item 2
; reachable at all in this pack's own conformance fixtures. From tick 2
; onward `biocapacity` feeds back as whatever float the PREVIOUS tick
; computed — generally not an integer — and in live gameplay
; `extraction_intensity` is a genuine float written by `ProductionSystem`
; (`production.py:268`), never an integer at all. Verified directly
; (multi-tick replay, both engines, `extraction_intensity=1` fixed): the
; two engines already disagree by tick 3 for a `biocapacity=5` seed at the
; production defaults, then coincide again at some later ticks and diverge
; at others — divergence is a property of the SPECIFIC trajectory, not a
; fixed tick number, and item 2's `<= 1 ULP` bound on `ecological-cost`
; is NOT claimed to hold once its own inputs stop being exact integers.
;
; **The bare-Int bypass's consequence, not just its mechanism.** D-1's
; workaround uses the SAME unsuffixed-`Int` `:const` escape hatch
; Dispossession's own D-2/D-4 document (`E-LEX-024` only bounds
; SCALED/suffixed literals) — which means, beyond the numeric deviation
; above, the load-time check ALSO admits an `entropy-factor-x1e6` outside
; `MetabolismDefines.entropy_factor`'s declared `(1.0, 3.0]` domain
; entirely, including negative — nothing on the BSL surface refuses it;
; only this rule's OWN in-body clamps (the `new-max`/`new-biocapacity`
; floors) contain the resulting value, the same defense-in-depth gap
; Dispossession's own D-2 already names for its weights.
;
; **Why the numeric deviation is acceptable anyway, and what is not
; resolved (ADR183 §5.4): the frozen Python engine is the contract source
; for STRUCTURE and ORDERING, never a bit-exact correctness oracle.** What
; is constitutional (III.7) is THIS engine's own determinism — the same
; content and the same inputs produce the same output, every time, on any
; conforming implementation — which integer-scaled arithmetic satisfies
; exactly regardless of how far it deviates from the frozen engine's own
; number (`the_metabolism_tick_is_deterministic` and its siblings in the
; conformance suites already prove this). Bit parity with a Python
; reference the Constitution never asked this port to reproduce exactly is
; not the bar; a documented, reproducible, DETERMINISTIC deviation is —
; even an unbounded one. The en-masse retirement of this whole workaround
; class — a genuine `Real x Ratio` operator, or `Ratio`-typed field
; storage, closing the gap at its root instead of per-consumer — is
; chartered as **workstream 3 of the post-port refactor program**
; (Director directive 2026-08-11, tracked at GitHub issue #502), not
; attempted by this port.
;
; `metabolism-entropy-low-conformance.bscn` /
; `metabolism-entropy-high-conformance.bscn` mutation-verify the workaround
; carries the coefficient's EFFECT end to end (a floor-inert result at
; `1.01` swinging to a floor-bound result at `3.0`) — a magnitude check,
; not a bit-exactness one; `metabolism-rounding-divergence-conformance.bscn`
; PINS one concrete, reproducible deviation (`biocapacity=3`, 2 ULP) rather
; than the general (unbounded) case — a PASS there means "matches this
; pack's own prior deterministic output", never "matches the frozen
; engine".
;
; **Recorded as an OPEN finding for the language surface, not resolved
; here**: `Real x Ratio` (or `Ratio`-typed field storage) is a genuine
; follow-up the BSL spec owners should rule on. Minting a new operator or a
; new Draft-Ruling Register row is a language-surface decision outside this
; port's scope — this D-record exists so a later reader does not have to
; re-derive the gap. (Note on numbering: this file's `D-1`..`D-5` are
; LOCAL to this pack, mirroring `dispossession.bsl`'s own `D-1`..`D-5` and
; `lifecycle.bsl`'s own `D-1`..`D-6` exactly — `bsl-language.rst`'s
; Draft-Ruling Register (`D97`-`D99`, sync-guarded by
; `TestTheDraftRulingRegisterHasNoDuplicateRowNumbers`) is a SEPARATE,
; language-spec-level register that neither Dispossession's nor Lifecycle's
; own D-records participate in; this pack follows their actual precedent,
; not a same-named-but-different register.)
;
; ============================================================================
; MODELING CHOICE — D-2: `regeneration_rate` becomes `:const`;
; `extraction_intensity`/`biocapacity`/`max_biocapacity` stay `:field`
; ============================================================================
;
; Applying Dispossession's own D-1 reasoning ("is it PROVABLY uniform, or
; genuinely live?") to each of this system's `attrs.get(field, default)`
; reads gives DIFFERENT answers for different fields — unlike Dispossession,
; where all five rate inputs landed the same way:
;
;   - `regeneration_rate`: grep-verified UNIFORM. No scenario builder in
;     `src/babylon/engine/scenarios/` ever assigns it a value distinct from
;     `Territory.regeneration_rate`'s own Pydantic default (`models/
;     entities/territory.py:165-170`, `default=0.02`), and nothing else in
;     the engine writes it onto a `TERRITORY` node (`SubstrateSystem`'s own
;     `regeneration_rate` reference is a DIFFERENT `SubstrateDefines`
;     coefficient feeding a DIFFERENT formula call on a DIFFERENT attribute,
;     `raw_material_stock` — `substrate.py`'s own module doc: "Does NOT
;     touch `Territory.biocapacity`/`MetabolismSystem`"). Every territory in
;     the shipped engine reads the same `0.02` — exactly the "per-node
;     storage never observably diverges from the global constant" shape
;     Dispossession's D-1 and Lifecycle's D-1 both name.
;   - `extraction_intensity`: NOT uniform, NOT dormant. Written live,
;     per-territory, by `ProductionSystem` (`production.py:268`,
;     `graph.update_node(node.id, extraction_intensity=intensity)`, derived
;     from `total_production / max_biocapacity`) — confirmed by the gap
;     report marking Metabolism "No" under "Dormant on canonical" (row
;     13.0), unlike Dispossession's "Yes (zero-rate inputs)". Treating this
;     as `:const` would misrepresent a live production channel — the
;     OPPOSITE of what justified Dispossession's own choice. Stays `:field`.
;   - `biocapacity`/`max_biocapacity`: self-evidently per-node evolving
;     state (this system's own primary output), AND seeded with genuinely
;     different values per territory at scenario-build time
;     (`_legacy.py:673-677`: `150.0`/`40.0`/`100.0` by sector
;     classification, not a uniform constant). Stay `:field`.
;
; ============================================================================
; TRANSCRIPTION NOTE — one shared `raw-extraction` binding, not two
; ============================================================================
;
; `calculate_biocapacity_delta`'s `raw_extraction = extraction_intensity *
; current_biocapacity` (`metabolic_rift.py:47`) and `calculate_hysteresis_
; damage`'s `raw_extraction = extraction_intensity * current_biocapacity`
; (`:86`) are the IDENTICAL formula over the IDENTICAL inputs — both called
; from `metabolism.py` with the same `attrs.get("extraction_intensity",
; 0.0)` and the same `biocapacity` read (`:98`/`:109`, `:95`/`:99`/`:103`).
; This pack computes it once (`raw-extraction` below) and reuses it for both
; the ecological cost AND the hysteresis damage — a value-preserving
; consolidation, not a transcription deviation (no output changes).
;
; ============================================================================
; BLOCKED — D-3: the spec-070 sovereign pre-pass has no BSL bind-src
; ============================================================================
;
; `metabolism.py:78-88` reads `context.persistent_data["balkanization.
; metabolic_impact_by_territory"]` (a one-tick-lagged handoff from
; `SovereigntySystem` @17.5) and adds it to `territory.habitability` before
; the biocapacity update. `bsl-language.rst` §2.5 closes `<bind-src>` at
; exactly four forms (`:field`/`:const`/`:metric`/`:tick`) — none can name
; `context.persistent_data`. `reports/bsl-gap-analysis-2026-08-10.md`'s Q6
; section ("Graph-scope state... the single most pervasive gap in the
; estate") names Metabolism by name among its 22 affected systems and names
; this EXACT handoff: "three of these values are one-tick-lagged handoffs
; (Sovereignty -> Metabolism..." Dropped whole, matching how Lifecycle's own
; header drops its two blocked rules — this is content-modeling work (Q6's
; own recommended fix routes it onto an ordinary graph field) this port does
; not perform unilaterally.
;
; ============================================================================
; BLOCKED — D-4: Phases 2-3 (global overshoot aggregate + emit)
; ============================================================================
;
; `metabolism.py:120-153` sums `biocapacity` over every `TERRITORY` and
; `(s_bio + s_class) * population` over every ACTIVE `SOCIAL_CLASS`, then
; emits `ECOLOGICAL_OVERSHOOT` when the ratio exceeds a threshold. Confirmed
; BLOCKED at the EXECUTION-ENGINE level, not merely cited: `bsl-language.
; rst`'s `(domain :graph)` construct (the mechanism this exact case needs —
; Q12's own text: "Metabolism's overshoot check... under per-node inference
; [it] would emit once per node") is fully implemented at LOAD time
; (`domain.rs::resolve_domain`/`RuleDomain::Graph`) but
; `tick.rs::run_tick` NEVER reads `loaded.domain` — it unconditionally calls
; `subject_type_of`, which only understands per-node `:field` namespaces.
; `babylon-tick/src/lib.rs` registers zero metrics (`metrics:
; HashSet::new()`), and no scenario/rule/test anywhere in
; `rust/crates/babylon-tick/` exercises `(domain :graph)` or `fold` end to
; end (grep-confirmed, zero hits for both). Even setting the engine gap
; aside, the aggregation spans TWO node-type namespaces in one rule, which
; no landed pack (including `fundamental-theorem.bsl`, single-namespace)
; has ever exercised. Dropped whole; full derivation in
; `reports/metabolism-port-assessment-2026-08-11.md` §4(b).
;
; ============================================================================
; TRANSCRIPTION DEVIATION — D-5: the four `.get(attr, default)` fallbacks
; become LOUD FAILURES on an absent field, not silent defaults (F5 fix
; round, adversarial review of PR #501 — dropped with no disposition in an
; earlier revision, a bare gap rather than an argued one)
; ============================================================================
;
; `metabolism.py:96-109` reads all four Phase 1 inputs via
; `attrs.get(field, default)`: `regeneration_rate` (default `0.02`),
; `max_biocapacity` (`100.0`), `extraction_intensity` (`0.0`) and
; `biocapacity` (`100.0`) — silently substituting the default whenever the
; graph dict lacks the key at all (not merely when it holds a falsy value).
;
; This pack's four corresponding bindings (`regeneration-rate`,
; `max-cap`, `extraction-intensity`, `current`) are all declared without
; `:optional`/`:default` — the ordinary, un-annotated shape every binding
; in this pack (and Dispossession's, and Lifecycle's) uses. On an ABSENT
; value they do NOT silently substitute anything: `regeneration-rate` is a
; `:const`, so a scenario missing its `(defconst metabolism/
; regeneration-rate …)` row fails to LOAD at all, naming the coefficient
; (`E-LOAD-010`); `max-cap`/`extraction-intensity`/`current` are `:field`
; reads, so a `TERRITORY` node missing the attribute fails the TICK loudly
; — `tick.rs::bind_subject`'s field-read arm propagates the substrate's own
; error rather than defaulting (its own comment: "the substrate's loud
; error, because III.11 says absence is not zero").
;
; **This is a DELIBERATE divergence, not an oversight, and not a defect
; this port must repair to match (ADR183 §5.4 asks the opposite: the
; frozen engine's own silent-default pattern here is the kind of thing a
; port need not carry forward).** Constitution III.11 ("Loud Failure") is
; exactly the standard the frozen Python's `.get(field, default)` shape
; violates for a required simulation input — a territory silently missing
; `biocapacity` reads as "fully charged at 100.0" in Python, masking what
; would otherwise be a visible data bug. This pack's every conformance
; fixture supplies all three fields on every node (matching how every
; other landed pack's own fixtures behave), so the loud-failure path is
; untested here by construction rather than by omission — recorded so a
; later reader does not have to re-derive why no vector exercises it.
;
; ============================================================================
; ENGINE MACHINERY
; ============================================================================
;
; `babylon-tick/src/lib.rs`'s `run_once_into` registered-system set gains
; `metabolism`, the same minimal driver-scaffolding addition Vitality,
; Lifecycle and Dispossession each made for their own anchors.

(rule metabolism/biocapacity-update
  :role mechanic
  :evidence derived
  :material-basis "a territory's biocapacity regenerates toward its ceiling and is depleted by extraction at an entropic loss every tick, and extraction permanently damages the ceiling itself — the earth cannot recover its original capacity even if extraction stops"
  :fuel 4096
  (bindings
    (binding current :field territory/biocapacity)
    (binding max-cap :field territory/max-biocapacity)
    (binding extraction-intensity :field territory/extraction-intensity)
    (binding regeneration-rate :const metabolism/regeneration-rate)
    ; D-1: scaled bare-Int workaround for entropy_factor's (1.0, 3.0]
    ; domain — Ratio's only operator is Currency x Ratio, and this
    ; formula's multiplicand is never Currency.
    (binding entropy-factor-x1e6 :const metabolism/entropy-factor-x1e6)
    (binding hysteresis-rate :const metabolism/hysteresis-rate)
    ; `regeneration = regeneration_rate * max_biocapacity; if current >=
    ; max_biocapacity: regeneration = 0.0` (`metabolic_rift.py:40-44`).
    ; The Real-zero promotion trick (dispossession.bsl's D-4 header, cited
    ; above): the THEN branch must share regeneration-raw's Real type.
    (binding regeneration-raw :expr (* regeneration-rate max-cap))
    (binding regeneration :expr
      (if (>= current max-cap) (- 0 0c) regeneration-raw))
    ; Shared by both formulas (see the header's TRANSCRIPTION NOTE). NO
    ; promotion trick needed here (unlike `regeneration` above, or
    ; dispossession.bsl's bare-Int `:const` bindings): a `:field` read
    ; ALWAYS resolves to `Value::Real` regardless of the field's declared
    ; `int` `deffield` type — `tick.rs::bind_subject`'s field-read arm is
    ; unconditionally `Ok(value) => Value::Real(value)`, because
    ; `GraphSubstrate`'s own storage is plain `f64` (`scenario.rs`'s module
    ; doc). `extraction-intensity` and `current` are both `:field`
    ; bindings, so `extraction-intensity * current` is `Real x Real` from
    ; the start — an earlier revision of this line wrapped it in a
    ; `(+ … (- 0 0c))` promotion, on the FALSE premise that the product
    ; would be `Int x Int` (adversarial review of PR #501 caught this: the
    ; premise does not hold for `:field`-sourced operands, only for
    ; `:const`-sourced ones, which CAN be bare, unsuffixed `Int` literals —
    ; D-1/D-4's own escape hatch).
    (binding raw-extraction :expr (* extraction-intensity current))
    ; `ecological_cost = raw_extraction * entropy_factor`
    ; (`metabolic_rift.py:47-50`) — D-1's scaled-Int workaround, descaled
    ; inline.
    (binding ecological-cost-scaled :expr (* raw-extraction entropy-factor-x1e6))
    (binding ecological-cost :expr (/ ecological-cost-scaled 1000000))
    (binding delta :expr (- regeneration ecological-cost))
    ; `damage = raw_extraction * hysteresis_rate`
    ; (`metabolic_rift.py:86-87`).
    (binding damage :expr (* raw-extraction hysteresis-rate))
    ; `new_max = max(0.0, max_cap - damage)` (`metabolism.py:113`) — the
    ; hysteresis ratchet. No scalar min/max in the grammar (the gap
    ; report's own Appendix item 2 recommends against a min/max rider,
    ; "nested `if` is doctrinally preferable under §3.3") — spelled with
    ; `if`, matching Dispossession/Lifecycle's own clamps.
    (binding max-cap-minus-damage :expr (- max-cap damage))
    (binding new-max :expr
      (if (> max-cap-minus-damage 0) max-cap-minus-damage (- 0 0c)))
    ; `new_biocapacity = max(0.0, min(new_max, current + delta))`
    ; (`metabolism.py:116`) — the double clamp, min then max, exactly.
    (binding current-plus-delta :expr (+ current delta))
    (binding capped-at-ceiling :expr
      (if (< current-plus-delta new-max) current-plus-delta new-max))
    (binding new-biocapacity :expr
      (if (> capped-at-ceiling 0) capped-at-ceiling (- 0 0c))))
  ; No `(when ...)` guard: the frozen Phase 1 loop has no `continue`
  ; (`metabolism.py:91-118`) — every TERRITORY node gets its effects
  ; unconditionally, matching Lifecycle's own unconditional Block 1.
  (effects
    (update-node self territory/biocapacity (set new-biocapacity))
    (update-node self territory/max-biocapacity (set new-max))))
