; DispossessionEventSystem (Material Base @10.0) — primitive accumulation as
; value transfer.
;
; Every tick, a county's foreclosure, eviction and displacement pressure
; combines with two structural ownership factors into one composite
; intensity, which draws a value transfer out of the territory's wealth —
; clamped to what the territory actually holds, split into what changes
; hands and what simply evaporates as deadweight loss (auction fees,
; vacancy, abandonment). The gap report's own verdict on this system: "One
; per-territory rule: five-term weighted intensity, clamped, wealth
; transfer, two emits. Cleanest fit in the estate"
; (`reports/bsl-gap-analysis-2026-08-10.md` row 10.0). THIS PACK PORTS THE
; WHOLE FROZEN SYSTEM — no phase is left un-ported, unlike Lifecycle/Vitality.
;
; ============================================================================
; MODELING CHOICE — D-1: the five per-territory inputs become `:const`
; ============================================================================
;
; `foreclosure_rate`, `eviction_rate`, `displacement_rate`,
; `concentrated_ownership` and `absentee_landlord_share`
; (`dispossession_events.py:70-88`) are read off the TERRITORY NODE, not off
; `GameDefines` — they are meant to be genuinely per-county data (FRED-derived
; foreclosure/eviction rates, institutional-ownership shares), unlike
; Lifecycle's D-1 fields, which were provably the SAME value everywhere
; because nothing else in the tree ever diverged them. This port cannot make
; the identical claim about these five fields — the whole point of the
; frozen design is that they vary by county.
;
; They become `:const` here anyway, for a reason that is a language
; constraint rather than a material claim: `bsl-language.rst`'s known
; constraint that "the slice-1 scenario loader seeds ONLY int-declared node
; attributes" (`scenario.rs::attribute_value`) accepts an INTEGER literal
; into an `int`-declared field and refuses every other combination outright —
; there is no legal way to seed a genuinely fractional per-node value in
; slice 1 at all, on any field, of any declared type. And unlike a
; scaled-integer workaround, that would not even be an honest fiction here:
; the gap report's own row for this system records it dormant on the
; canonical run TODAY for exactly this reason — "Yes (zero-rate inputs)" —
; nothing hydrates real per-county foreclosure/eviction/displacement data
; yet, so there is no live per-territory variation this port could
; misrepresent by flattening. When real per-county hydration lands (Phase 2's
; Currency/Probability-typed field storage), these five become genuine
; per-territory `:field` reads with NO OTHER CHANGE to the bindings below —
; they slot into the SAME `foreclosure-rate-const`/etc. positions a `:field`
; binding would occupy. (Adversarial-review correction: an earlier revision
; of this note claimed restoring the per-input clamps below would be "no
; change to the weighted-sum algebra" — false, and retracted; see D-2.)
;
; Seven `:const` environments prove every branch this shared-input design
; still discriminates (learn from #493's verification round): the ACTIVE
; environment (`dispossession-conformance.bscn`) with nonzero rates, the
; ZERO-RATE environment (`dispossession-zero-rate-conformance.bscn`, all
; three primary rates 0) that proves this system's actual canonical
; behavior — completely inert, matching the gap report's Class C verdict —
; and the SINGLE-RATE environment
; (`dispossession-single-rate-conformance.bscn`) that discriminates the
; `(when …)` gate's OR from an AND (see the transcription note below). Four
; more, added in the adversarial-review fix round, individually
; mutation-verify every clamp D-2/D-3/D-4 restore:
; `dispossession-saturation-conformance.bscn` (the intensity/transfer-
; amount/deadweight-fraction ceilings, all at once),
; `dispossession-negative-input-conformance.bscn` (`foreclosure-rate`'s
; ceiling plus every OTHER per-input floor),
; `dispossession-ceiling-matrix-conformance.bscn` (`eviction-rate`'s
; ceiling plus every OTHER per-input ceiling, plus `foreclosure-rate`'s
; floor), and `dispossession-negative-weight-conformance.bscn` (D-3's
; total-sum floor against a negative WEIGHT, the one clamp the first six
; scenarios leave mutation-dead once every rate/structural term is
; individually `[0, 1]`-bounded). Wealth stays a genuine per-node `:field`,
; so the ACTIVE scenario alone still discriminates the value-transfer guard
; below (a wealthy county vs. an insolvent one) without needing per-node
; rates at all.
;
; ============================================================================
; TRANSCRIPTION NOTE — the `:const` environment does not hide the frozen
; gate's exact shape
; ============================================================================
;
; `dispossession_events.py:75-76`: `if foreclosure_rate <= 0.0 and
; eviction_rate <= 0.0 and displacement_rate <= 0.0: continue` reads ONLY
; the three primary rates — NOT `concentrated_ownership` or
; `absentee_landlord_share`. A territory with nonzero structural ownership
; factors but all-zero primary rates has NO EFFECTS run: no state written,
; no event published, even though the intensity formula's value would be
; nonzero from the structural terms alone. (In slice 1 the `:expr` bindings
; — the intensity sum included — ARE still evaluated and fuel-charged
; before the guard runs, per `babylon_bsl::tick::run_tick`; what the gate
; guarantees is that no effect consumes them.)
; This is not a defect ADR183 §5.4 asks this port to repair — it is the
; frozen system's own gate, and `(when …)` below transcribes exactly those
; three fields and no others. `dispossession-zero-rate-conformance.bscn`
; sets `concentrated-ownership`/`absentee-landlord-share` to nonzero
; specifically to prove the gate really does ignore them. The gate compares
; the FLOORED (but not ceiling-clamped) rate against zero — see D-4's
; floor/ceiling split below for why `foreclosure-rate` (the binding the gate
; reads) is the floored value and not the raw `:const`.
;
; ============================================================================
; DEAD FIELD — fips_code/year are dropped
; ============================================================================
;
; `TerritoryDispossessionState` (`domain/economics/dispossession/types.py`)
; carries `fips_code` and `year` alongside the five rate/structural fields,
; but neither `compute_intensity` nor `compute_value_transfer`
; (`domain/economics/dispossession/intensity.py`) ever reads either —
; grep-verified against both function bodies. They are passthrough
; identification fields on the Pydantic state object, not formula inputs.
; Dropping them changes no observable output.
;
; ============================================================================
; §5.4 CORRECTION — D-2: the two "clamp to available wealth" checks are
; RESTORED (an earlier revision of this pack argued they were redundant;
; adversarial review found the argument FALSE, by execution)
; ============================================================================
;
; The retracted argument claimed `intensity * transfer-scale <= 1` always,
; because `transfer-scale` is "transcribed as a `c`-suffixed literal, and
; every legal `c` literal is bounded to `[0, 1]` at LOAD time by
; `E-LEX-024`". That is true of a `c`-SUFFIXED literal — but `defconst`
; also accepts a BARE `Atom::Int` (`scenario.rs::load_defconst`), and a bare
; Int carries NO domain check at all; `real_lane` promotes it straight to
; `Real` (`bsl-language.rst` §3.3's Int-promotes-to-Real rule, evaluator.rs).
; `(defconst dispossession/transfer-scale 12)` — no suffix — loads clean and
; is a completely ordinary, legal `:const`. The retracted proof's premise
; ("every legal value a modder could write for `transfer_scale`") was
; therefore false: `:const` enforces NO domain by construction, unlike
; `GameDefines`' `DispossessionDefines.transfer_scale: float =
; Field(ge=0.0, le=1.0)`, which Pydantic validates at OBJECT CONSTRUCTION —
; not merely at YAML-parse time — and which a raw territory-node attribute
; (read via `_get_float` off a plain graph dict, not a validated model) does
; NOT inherit either, which is exactly why the frozen source carries its own
; belt-and-suspenders clamps regardless of Pydantic. `:const` inherits
; NEITHER guardrail. Adversarial probe, run against this pack pre-fix:
; `transfer-scale 12` (unsuffixed) in the ACTIVE environment (whose
; intensity is exactly `0.358`, so `1e6 − 1e6 × 0.358 × 12`)
; produced `wealth = -3_296_000.0` for a `1_000_000`-wealth subject — a
; negative territory wealth, structurally impossible under the frozen
; source's own `min(transfer_amount, territory_wealth)` line, which this
; pack had omitted.
;
; **The fix:** `transfer_amount = territory_wealth * intensity *
; transfer_scale; transfer_amount = min(transfer_amount, territory_wealth)`
; (`dispossession_events.py:95-96`) is transcribed in full below
; (`transfer-amount-raw` / `transfer-amount`). The SAME argument retired
; `compute_value_transfer`'s internal `fraction = min(max(fraction, 0.0),
; 1.0)` (`intensity.py:74`, applied to `deadweight_loss_fraction`) on the
; identical false premise; it is restored below too
; (`deadweight-fraction-floored` / `deadweight-fraction`). Adversarial probe:
; `deadweight-loss-fraction 3` (unsuffixed) pre-fix produced a NEGATIVE
; `net-received` — `total_transferred - total_transferred * 3`. Neither
; probe value (`transfer_scale=12`, `deadweight_loss_fraction=3`) is
; reachable through the real engine's OWN configuration surface
; (`GameDefines`/`ServiceContainer` construct a Pydantic-validated
; `DispossessionDefines` that refuses both at object-construction time,
; confirmed by direct probe: `DispossessionDefines(transfer_scale=12.0)`
; raises `ValidationError` immediately) — which is precisely the point: the
; frozen SOURCE's clamp lines are defense-in-depth against a configuration
; surface Pydantic mostly already blocks, and `:const` reintroduces the gap
; Pydantic exists to close. A port that relies on BSL's weaker guardrail
; matching Python's stronger one is not a faithful transcription; it is an
; accidental strengthening the source code does not itself rely on.
; `compute_value_transfer`'s leading `if total_value <= 0.0: return (0.0,
; 0.0)` (`intensity.py:66-67`) remains untranscribed — the frozen engine
; only ever calls it inside `if transfer_amount > 0.0:`
; (`dispossession_events.py:98-99`), so that branch is unreachable from
; THIS call site regardless of which engine implements it; this is a real
; dead branch, not the kind of false claim the rest of this section retracts.
;
; ============================================================================
; MODELING CHOICE — D-3: the intensity clamp, kept for the same reason as D-2
; ============================================================================
;
; `compute_intensity`'s `min(max(intensity, 0.0), 1.0)` (`intensity.py:48`)
; is transcribed explicitly (`intensity-floor`/`intensity` below) — this was
; never in question, and D-2's retraction only strengthens the reason: `:const`
; enforces no domain, so `weight_foreclosure`/`weight_eviction`/
; `weight_displacement`/`weight_tax_sale`/`weight_eminent_domain` could each
; individually be authored past `1.0` (not just past their SUM, as the
; original version of this note observed about `DispossessionDefines`' own
; `Field(ge=0.0, le=1.0)` constraint having no cross-field validator on the
; sum). The clamp is necessary under an even wider set of authoring mistakes
; than first recorded.
;
; ============================================================================
; §5.4 CORRECTION — D-4: the five per-input clamps are RESTORED (omitted
; from an earlier revision with NO D-record at all — a bare gap, not even an
; argued one)
; ============================================================================
;
; `dispossession_events.py:70-72`: `foreclosure_rate = _get_float(data,
; "foreclosure_rate")` etc. — `_get_float` (`:136-141`) floors EVERY read at
; `0.0` (`max(float(val), 0.0)`; a non-numeric or absent value also reads as
; `0.0`). Then, at `TerritoryDispossessionState` construction
; (`:81-89`), THE SAME THREE outer-scope variables are additionally
; ceiling-clamped to `1.0` (`min(foreclosure_rate, 1.0)` etc.) as KEYWORD
; ARGUMENTS — which does NOT rebind the outer Python variable, so the outer
; `foreclosure_rate`/`eviction_rate`/`displacement_rate` stay FLOOR-ONLY for
; the rest of the function, while `state.foreclosure_rate` etc. are
; FLOOR-AND-CEILING. `concentrated_ownership`/`absentee_landlord_share` are
; floored and ceiling-clamped in the SAME expression (`min(_get_float(...),
; 1.0)`), with no separate outer-scope floor-only variable to diverge from.
;
; Two consequences this pack's earlier revision missed entirely (no D-record
; at all — the omission was structural, not a reasoned redundancy claim):
;
;   1. **The intensity sum uses the CLAMPED values; the `DISPOSSESSION_EVENT`
;      payload and the `(when …)` gate use the FLOOR-ONLY outer variables.**
;      `payload={"foreclosure_rate": foreclosure_rate, ...}`
;      (`:128-130`) reads the OUTER variable — floor-only, not
;      `state.foreclosure_rate`. This pack's `foreclosure-rate`/
;      `eviction-rate`/`displacement-rate` bindings (floor-only) feed BOTH
;      the `(when …)` gate and the payload, exactly matching; a SEPARATE
;      `-clamped` binding (floor AND ceiling) feeds the intensity sum.
;   2. **Per-term clamping is structurally different from clamping only the
;      total.** A negative per-term input does not merely risk pushing the
;      SUM negative (which the total-only floor this pack already had would
;      catch) — the frozen engine floors it to EXACTLY `0.0` before it
;      enters the weighted sum at all, so it contributes NOTHING, not a
;      negative term partially offset by others. Adversarial probe:
;      `foreclosure_rate=1`, `eviction_rate=-3` (unsuffixed, bypassing
;      `E-LEX-024`) — the frozen engine (run directly, confirmed against the
;      real `DispossessionEventSystem`) computes `intensity = 0.4`
;      (`weight_foreclosure * 1`, `eviction_rate` floored to `0` before
;      weighting) and fires `VALUE_TRANSFER`; this pack's total-only-clamped
;      predecessor would have summed the RAW `-3` into the weighted total
;      (`0.4*1 + 0.3*(-3) = -0.5`), which the total floor would then ALSO
;      clamp to `0.0` — a materially different number reached by a
;      materially different (and wrong, for any input where the two
;      floors disagree) route. `dispossession-negative-input-
;      conformance.bscn` pins this exactly.
;
; ============================================================================
; D-5: the SIXTH `_get_float` floor — on `wealth` (`dispossession_events.py:
; 94`) — is deliberately NOT transcribed: `wealth`'s only consumers are
; `transfer-amount-raw`/`transfer-amount`/`new-wealth`, all inside the
; `(guard (> transfer-amount 0))` block, and `transfer-amount =
; min(raw, wealth) <= wealth`, so the guard passes only when `wealth > 0` —
; where the floor is an identity. For `wealth <= 0` BOTH engines skip the
; guarded block, and neither the intensity write nor `DISPOSSESSION_EVENT`
; reads `wealth`. Effects-equivalent by that derivation; recorded rather
; than transcribed.
; ============================================================================
;
; ============================================================================
; TRANSCRIPTION NOTE — effect order matches the frozen source's event order
; ============================================================================
;
; `dispossession_events.py`'s `VALUE_TRANSFER` publish (inside
; `if transfer_amount > 0.0:`, lines 98-115) runs BEFORE the unconditional
; `dispossession_intensity` write and `DISPOSSESSION_EVENT` publish (lines
; 117-133, both OUTSIDE that `if`, at the SAME indentation). The frozen
; engine's own printed event log for a wealthy, active subject confirms the
; order: `value_transfer` then `dispossession_event`
; (`dispossession_conformance.py`'s own run). §2.8's effects apply in
; SOURCE ORDER, so the `(guard …)` wrapping the wealth write and
; `VALUE_TRANSFER` emit comes FIRST in the effects list below, and the
; unconditional intensity write and `DISPOSSESSION_EVENT` emit come second —
; reversed from a naive top-to-bottom reading of the Python function, which
; states the intensity write in a comment ("Store intensity on node") after
; the transfer block only because it is un-indented from it, not because it
; runs first.
;
; ============================================================================
; ENGINE MACHINERY
; ============================================================================
;
; `babylon-tick/src/lib.rs`'s `run_once_into` registered-system set gains
; `dispossession`, the same minimal driver-scaffolding addition Vitality and
; Lifecycle each made for their own anchors.

(rule dispossession/territory-transfer
  :role mechanic
  :evidence derived
  :material-basis "a county's foreclosure, eviction and displacement pressure, combined with how concentrated and absentee-owned its housing stock already is, draws a value transfer out of its wealth every tick primitive accumulation is live there — split between what changes hands and what evaporates as deadweight loss (auction fees, vacancy, abandonment)"
  :fuel 4096
  (bindings
    (binding wealth :field territory/wealth)
    ; D-4: `_get_float`'s floor (`max(x, 0.0)`), matching the outer-scope
    ; Python variable that BOTH the `(when …)` gate and the
    ; DISPOSSESSION_EVENT payload read.
    ; The THEN branch adds Real zero (`(+ … (- 0 0c))`) rather than
    ; returning the `:const` bare — a bare-Int-literal `:const` (D-2's own
    ; escape hatch) would otherwise carry `Value::Int` straight through the
    ; THEN branch while the ELSE branch is `Value::Real`, so the SAME
    ; binding's dynamic type would depend on which branch fired: readable
    ; correctly by `real_lane` either way, but an observable, avoidable
    ; inconsistency in every payload/state read downstream (caught by
    ; `dispossession-negative-input-conformance.bscn`'s own
    ; `foreclosure-rate=5` probe: the payload came back `Int(5)`, not
    ; `Real(5.0)`, before this fix). `X + 0.0 = X` exactly in IEEE-754 for
    ; any finite `X` this pack's fixtures use — an identity, not an
    ; approximation.
    (binding foreclosure-rate-const :const dispossession/foreclosure-rate)
    (binding foreclosure-rate :expr
      (if (> foreclosure-rate-const 0)
          (+ foreclosure-rate-const (- 0 0c))
          (- 0 0c)))
    ; D-4: THEN `min(x, 1.0)`, matching `state.foreclosure_rate` — used only
    ; by the intensity sum, never by the gate or the payload.
    (binding foreclosure-rate-clamped :expr
      (if (< foreclosure-rate 1) foreclosure-rate (- 1 0c)))

    (binding eviction-rate-const :const dispossession/eviction-rate)
    (binding eviction-rate :expr
      (if (> eviction-rate-const 0)
          (+ eviction-rate-const (- 0 0c))
          (- 0 0c)))
    (binding eviction-rate-clamped :expr
      (if (< eviction-rate 1) eviction-rate (- 1 0c)))

    (binding displacement-rate-const :const dispossession/displacement-rate)
    (binding displacement-rate :expr
      (if (> displacement-rate-const 0)
          (+ displacement-rate-const (- 0 0c))
          (- 0 0c)))
    (binding displacement-rate-clamped :expr
      (if (< displacement-rate 1) displacement-rate (- 1 0c)))

    ; `concentrated_ownership`/`absentee_landlord_share`: floor then ceiling
    ; in one Python expression (`min(_get_float(...), 1.0)`), with no
    ; separate outer-scope floor-only reader — neither is ever emitted in a
    ; payload, so only the clamped form is needed.
    (binding concentrated-ownership-const :const dispossession/concentrated-ownership)
    (binding concentrated-ownership-floored :expr
      (if (> concentrated-ownership-const 0)
          (+ concentrated-ownership-const (- 0 0c))
          (- 0 0c)))
    (binding concentrated-ownership-clamped :expr
      (if (< concentrated-ownership-floored 1) concentrated-ownership-floored (- 1 0c)))

    (binding absentee-landlord-share-const :const dispossession/absentee-landlord-share)
    (binding absentee-landlord-share-floored :expr
      (if (> absentee-landlord-share-const 0)
          (+ absentee-landlord-share-const (- 0 0c))
          (- 0 0c)))
    (binding absentee-landlord-share-clamped :expr
      (if (< absentee-landlord-share-floored 1) absentee-landlord-share-floored (- 1 0c)))

    ; `DispossessionDefines` weights, `defines.yaml:425-429` — read
    ; directly, no PER-TICK clamp in the frozen source (only Pydantic's
    ; one-time construction-time validation, which `:const` does not
    ; inherit; D-3 explains why the SUM still needs its own clamp).
    (binding weight-foreclosure :const dispossession/weight-foreclosure)
    (binding weight-eviction :const dispossession/weight-eviction)
    (binding weight-displacement :const dispossession/weight-displacement)
    (binding weight-tax-sale :const dispossession/weight-tax-sale)
    (binding weight-eminent-domain :const dispossession/weight-eminent-domain)
    ; `compute_intensity`'s exact left-to-right association
    ; (`intensity.py:41-47`), over the CLAMPED (floor-and-ceiling) rate/
    ; structural values — `state.*`, not the floor-only outer variables.
    (binding raw-intensity :expr
      (+ (+ (+ (+ (* weight-foreclosure foreclosure-rate-clamped)
                  (* weight-eviction eviction-rate-clamped))
               (* weight-displacement displacement-rate-clamped))
            (* weight-tax-sale concentrated-ownership-clamped))
         (* weight-eminent-domain absentee-landlord-share-clamped)))
    ; D-3: `min(max(raw_intensity, 0.0), 1.0)`, spelled explicitly. The
    ; `(- 0 0c)`/`(- 1 0c)` forms are Real zero/one — Lifecycle's own
    ; promotion trick (`lifecycle.bsl:284`'s header) for the same reason:
    ; `if`'s two branches must share one static type (E-TYPE-020), and a
    ; bare `0`/`1` Int literal would not match `raw-intensity`'s Real type.
    ; The same trick recurs at every clamp in this rule.
    (binding intensity-floor :expr
      (if (> raw-intensity 0) raw-intensity (- 0 0c)))
    (binding intensity :expr
      (if (< intensity-floor 1) intensity-floor (- 1 0c)))
    (binding transfer-scale :const dispossession/transfer-scale)
    ; `transfer_amount = territory_wealth * intensity * transfer_scale`
    ; (`dispossession_events.py:95`), left-to-right, THEN
    ; `min(transfer_amount, territory_wealth)` (`:96`) — D-2: restored.
    (binding wealth-times-intensity :expr (* wealth intensity))
    (binding transfer-amount-raw :expr (* wealth-times-intensity transfer-scale))
    (binding transfer-amount :expr
      (if (< transfer-amount-raw wealth) transfer-amount-raw wealth))
    ; `compute_value_transfer` (`intensity.py:50-78`), reached only when
    ; `transfer_amount > 0.0` at the call site — the guard below reproduces
    ; that condition exactly, so these bindings are safe to compute
    ; unconditionally here (§2.5/§4.2: `:expr` bindings resolve before any
    ; effect, so computing them costs nothing when the guard turns out
    ; false — they are simply not written anywhere). `fraction =
    ; min(max(deadweight_loss_fraction, 0.0), 1.0)` (`intensity.py:74`) —
    ; D-2: restored.
    (binding deadweight-fraction-const :const dispossession/deadweight-loss-fraction)
    (binding deadweight-fraction-floored :expr
      (if (> deadweight-fraction-const 0)
          (+ deadweight-fraction-const (- 0 0c))
          (- 0 0c)))
    (binding deadweight-fraction :expr
      (if (< deadweight-fraction-floored 1) deadweight-fraction-floored (- 1 0c)))
    (binding deadweight :expr (* transfer-amount deadweight-fraction))
    (binding net-received :expr (- transfer-amount deadweight))
    (binding new-wealth :expr (- wealth transfer-amount)))
  ; The frozen loop's one `continue` (`dispossession_events.py:75-76`) —
  ; transcribed exactly: the three PRIMARY rates only (floor-only values —
  ; see D-4), per the header's transcription note.
  (when (or (> foreclosure-rate 0) (> eviction-rate 0) (> displacement-rate 0)))
  (effects
    ; `if transfer_amount > 0.0:` (`dispossession_events.py:98`) — wealth
    ; write and VALUE_TRANSFER emit, FIRST in source order (see the header's
    ; effect-order note).
    (guard (> transfer-amount 0)
      (update-node self territory/wealth (set new-wealth))
      (emit EventType/VALUE_TRANSFER
        (territory self)
        (total-transferred transfer-amount)
        (net-received net-received)
        (deadweight-loss deadweight)))
    ; Unconditional: intensity write + DISPOSSESSION_EVENT emit
    ; (`dispossession_events.py:117-133`, outside the `if` above). The
    ; payload reads the FLOOR-ONLY `foreclosure-rate`/`eviction-rate`/
    ; `displacement-rate` bindings (D-4), not the `-clamped` ones the sum
    ; uses.
    (update-node self territory/dispossession-intensity (set intensity))
    (emit EventType/DISPOSSESSION_EVENT
      (territory self)
      (intensity intensity)
      (foreclosure-rate foreclosure-rate)
      (eviction-rate eviction-rate)
      (displacement-rate displacement-rate))))
