; VitalitySystem (Material Base @1.0) — The Drain and The Reaper.
;
; Living costs wealth. A class burns subsistence every tick in proportion to
; its numbers and to the standard of living its position requires, and a
; block that can no longer cover its own reproduction stops existing.
;
; ONE rule, not three. §4.2: rules within one system position observe the
; same pre-state, so a three-rule decomposition would have to restate the
; drain algebra in each downstream rule. The `:expr` bindings of R9 chapter
; C7 name the intermediates once instead.
;
; GRINDING ATTRITION, LANDED — this header's old "WHAT THIS RULE
; DELIBERATELY DOES NOT DO" block, rewritten as the record of what the
; engine now DOES (T6.5, #491 Phase 3b, 2026-08-21). The frozen system's
; Phase 2 lives in `vitality-attrition.bsl` as `vitality/subsistence-mortality`:
; `deaths = floor(population × failing-certain × κ)` — the mortality driver
; is the measured certainly-failing mass of the K=16 rung ladder (H2',
; DP-6 = B, D199), κ the derived-not-picked `1.0c` defconst (ADR210 R14;
; the derivation and divergence surface are D198, exhibited by
; `vitality_attrition_conformance.py`'s own sweep printer). The two
; blockers this block once recorded are both discharged: blocker 1 (the
; Real→Int demotion) by ADR188 Row 2's `floor` intrinsic, blocker 2 (the
; rate as stipulated form + tuned knob, `deficit × (attrition_base_factor +
; inequality)`) by the measure itself — `attrition_base_factor` is retired,
; NOT transcribed (ADR191 R3), and `social-class/inequality`'s dispersion-
; surrogate duty with it (§3.3b: entered the frozen form twice, threshold
; and slope; explained there, superseded by the ladder, retired at this
; port). What this file's rule still does not do: the drain and the reaper
; below remain this pack's whole behavior — mortality lives in the carrier
; pack, and the two packs are not co-loaded by any committed scenario
; today (D197's collision-surface note covers the one shared intrinsic
; declaration).
;
; HISTORY (immutable): the pre-T6 text of this block, and the two-blocker
; analysis it stood on, are in git at the `p27-python-freeze` pin and in
; the sweep's integration history (#678); docs/superpowers/plans/
; 2026-08-10-vitality-bsl-rule-pack.md §6 remains the blockers' design
; record.
(rule vitality/subsistence-and-death
  :role mechanic
  :evidence derived
  :material-basis "a class reproduces itself out of its own wealth every tick, at a cost set by its numbers and by the standard of living its position in production requires; a block that cannot meet that cost ceases to exist as a class"
  :fuel 512
  (bindings
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding subsistence-multiplier :field social-class/subsistence-multiplier)
    (binding s-bio :field social-class/s-bio)
    (binding s-class :field social-class/s-class)
    (binding base-subsistence :const economy/base-subsistence)
    (binding death-threshold :const economy/death-threshold)
    ; The frozen engine's association order, transcribed exactly:
    ; (base_subsistence * population) * multiplier. `<arith>` is strictly
    ; binary (E-PARSE-040), so the source states it rather than implying it,
    ; and binary64 reproduces the same double.
    (binding cost :expr (* (* base-subsistence population) subsistence-multiplier))
    ; The frozen engine writes max(0.0, wealth - cost): it destroys the part
    ; a class cannot pay rather than carrying a debt. That is a modelling
    ; choice of the reference and part of the structure contract, so this
    ; transcribes it — stated positively, as what the class actually hands
    ; over: subsistence, or everything it has, whichever is less.
    ;
    ; Written this way and not as `(if (> (- wealth cost) 0) … 0)` for two
    ; reasons. §3.10's rider slate row 5 declines a scalar min/max precisely
    ; so a saturation stays legible in the source rather than hiding in an
    ; operator — and §1.5 admits NO bare non-integer literal, so the zero
    ; branch of that form would be an `Int` where the other branch is a
    ; `Real`: two static types under one `if`, in a language that declares
    ; no coercions (§3.1). Subtracting what was paid keeps one type
    ; throughout and lands on exactly 0.0 when a class is wiped out.
    (binding paid :expr (if (> wealth cost) cost wealth))
    (binding drained :expr (- wealth paid))
    (binding consumption-needs :expr (+ s-bio s-class)))
  ; The two `continue`s at the top of the frozen loop.
  (when (and (= active 1) (> population 0)))
  (effects
    (update-node self social-class/wealth (set drained))
    ; The Reaper. `is_extinct` (population <= 0 after attrition) cannot fire
    ; here — the guard above already excludes it and no attrition decrement
    ; runs — so the two surviving causes remain, both of which require a
    ; block of one and both of which set the same two fields:
    ;   wealth_threshold: drained < death_threshold  (the zombie failsafe)
    ;   starvation:       drained < s_bio + s_class
    (guard (and (= population 1)
                (or (< drained death-threshold)
                    (< drained consumption-needs)))
      (update-node self social-class/active (set 0))
      (update-node self social-class/population (set 0))
      ; The frozen payload's `cause` string does not travel: §2.8 admits no
      ; string in a payload and §1.5 admits string literals at
      ; :material-basis and vector ids only (E-PARSE-010). A discriminant
      ; would need a registered closed enum, which is a vocabulary addition
      ; and therefore spec-first. It stays recoverable from the payload:
      ; drained < death_threshold means wealth_threshold, else starvation.
      (emit EventType/ENTITY_DEATH
        (entity-id self)
        (wealth drained)
        (consumption-needs consumption-needs)
        (s-bio s-bio)
        (s-class s-class)))))
