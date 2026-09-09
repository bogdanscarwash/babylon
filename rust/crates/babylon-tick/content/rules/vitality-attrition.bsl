; The K=16 rung-ladder DUAL MEASURE — clearing / failing_certain /
; straddle_band, the within-class subsistence measure P(S|A) stands in for
; (#491 T5, Phase 3a; ADR173, ADR194 R1). Phase 1 (T4) declared and seeded
; the carrier INERT; this task gives it its first REAL rule. T6 (Phase 3b,
; Grinding Attrition) is a SEPARATE, later rule that reads whichever mass
; DP-6 rules the mortality driver — this rule decides nothing about that
; (landing both duals is DP-6-neutral, design doc §9/T5.4).
;
; FILE, CORRECTED FROM THE BRIEF'S OWN "Files:" LINE (disclosed, not
; silent): the plan text (§9/T5, `docs/superpowers/plans/
; 2026-08-17-491-rung-ladder.md:1293`) names `content/rules/vitality.bsl`
; as this task's target. That file cannot host this rule: `vitality.bsl`
; is loaded VERBATIM by `vitality_conformance.rs`'s and `tick_goldens.rs`'s
; PINNED tests together with `vitality-conformance.bscn`, which declares
; NONE of the sixteen `wealth-mass-*` fields, the fifteen `cut-*`
; defconsts, or `vitality/subsistence-horizon` — a `:field`/`:const`
; binding naming an undeclared qname is `E-LOAD-010`
; (`rust/crates/babylon-bsl/src/bindings.rs:220`), UNCONDITIONALLY, `
; :optional` or not, so landing this rule's bindings in `vitality.bsl`
; would break every one of the eighteen pre-existing pins at LOAD, not
; drift them. `vitality-attrition-conformance.bscn` (T4) already declares
; every construct this rule reads, and `vitality-attrition.bsl` is the
; ONE rule file this task's own test harness (`vitality_attrition_
; conformance.rs`'s `RULE` const, `tick_goldens.rs`'s
; `VITALITY_ATTRITION_RULE` const) already pairs with it — T4's own probe
; header even names this file as what "a future task may extend in place
; or replace outright." This task does the latter.
;
; THE LEVEL SET (ADR210 R13, register row D188). `S = s_bio + s_class` —
; the ACQUIESCENCE level set, not the mortality level set (`s_bio` alone).
; This IS the P(S|A) reading: OQ-B's "does this class's own wealth clear
; the standard of living its position requires" question, at the
; acquiescence standard. No divergence D-row is owed BY THIS ROW: R13's
; owed divergence (frozen `s_bio + s_class` used as a MORTALITY threshold)
; is T6's business, not this one's — this rule never reads mortality's
; `s_bio`-alone level set at all.
;
; THE S-7 DERIVATION (§8/T5.5, NO IMPOSED SIGMOIDS, ADR172 ruling 5,
; ADR173). Every operation below is a multiply, an add, a comparison, or a
; rung-membership test — nothing else. `clearing`, read as a function of
; `S*tau / w-bar`, IS the class's own complementary empirical CDF over its
; sixteen-rung wealth distribution: as the threshold ratio rises, fewer
; rungs clear it, and `clearing` falls monotonically — a STEP function
; with sixteen risers, not a stipulated curve. No exponent, no steepness
; constant, no `sigmoid` (a prohibited intrinsic name, `E-LOAD-024`), no
; interpolation between rungs. The S-curve this train's whole mandate asks
; for EMERGES from the within-class mass dispersion the K=16 carrier
; holds — sharper for a concentrated distribution, gentler for a spread
; one (ADR202 R2's asserted sign; T5.1(3)'s hand-authored property test
; carries it, not this fixture, C-6).
;
; THE HORIZON IDENTITY (H3, design doc §6.2). Dividing H2's rung condition
; through by `S` gives `H_k = (cut_{k-1} * w-bar) / S`, rung k's own
; hold-out horizon in TICKS, with `c_k = 1 iff H_k >= tau` — an EXPOSITION
; of the identity only; this rule never computes that quotient, because
; `S / w-bar`-shaped division lands outside `[0,1]` for exactly the
; below-subsistence class and trips `E-EVAL-013` (D187, the money-vs-money
; law) — the comparison below stays money-vs-money throughout, `cut *
; w-bar >= S * tau`, never the other order. `clearing(S, tau)` therefore
; reads identically as "the mass whose hold-out horizon reaches tau" — the
; SAME measure ReserveArmy's wage-pressure `L` and Allegiance/
; FascistFaction each read at a DIFFERENT horizon (H3's whole point: ONE
; measure, many horizons, no copy-drift). This rule reads the tick's own
; horizon (tau = vitality/subsistence-horizon, DP-5 = A); a future
; ReserveArmy port reads the SAME `clearing_failing_straddle` shape at its
; own `L`, never a re-derived curve under a different name.
;
; H2' — THE DUAL PAIR (C-7 repair, design doc §6.2). `clearing`'s mass is
; a LOWER BOUND ("the ladder ESTABLISHES reproduces itself") because a
; rung counts only when its WHOLE span clears; `failing_certain` is the
; dual lower bound on certain failure; the gap between them —
; `straddle_band` — is the mass of the ONE rung the K=16 grid's threshold
; actually cuts through, published as its own quantity rather than folded
; silently into either side (L-ABS/ADR070: declared resolution, not
; fabrication). `clearing`'s bottom-rung floor (rung 1's implicit,
; unspellable lower edge) means `mass-01` never counts toward `clearing`
; by CONSTRUCTION (the sixteen `edge-*` bindings below span cuts 1..15
; only — there is no `edge-00`), not by an omission a reader has to take
; on faith. `straddle-band` complements against the BOUND `mass-sum`
; (review I-1), never a stipulated `1.0c`: the guard admits any class
; with `mass-sum > 0`, not `mass-sum = 1`, and a partially-seeded class
; (some of the sixteen masses genuinely absent, the `:optional :default
; 0.0c` idiom) must never have its UNSEEDED mass silently reported as
; "the rung the threshold straddles" — that is a measurement claim about
; members whose position was never recorded, fabrication under §8's
; L-ABS/ADR070 no-exemption clause. Complementing against `mass-sum`
; keeps the identity exact wherever the masses DO sum to 1 (the
; committed fixture, by construction) and degrades to a true partial
; measure otherwise.
;
; DP-6-NEUTRAL, EXPLICITLY. Both `clearing` and `failing_certain` are
; bound and exposed here; NEITHER drives a mortality write in this rule —
; this rule's only effect is `emit`, which never touches graph state
; (`update-node`/`update-edge`/`update-hyperedge`/`add-*`/`remove-*` never
; appear below), so the K=16 carrier's own state-hash pin
; (`tick_goldens.rs::vitality_attrition_carrier_hashes_are_pinned`) stays
; BYTE-IDENTICAL to T4's measurement even though this rule now fires for
; real (T5.7: "a binding and a condition, no effect"). T6 is the task that
; picks which of the two duals kills and writes the first REAL effect
; against this carrier.
;
; THE EVENT TYPE. `EventType/SUBSISTENCE_CLEARANCE_MEASURED` is NOT yet a
; member of `src/babylon/models/enums/events.py`'s hundred-value catalog —
; disclosed here rather than silently minted. Nothing in this checkout
; enforces `EventType` membership for this scenario: neither
; `vitality-attrition-conformance.bscn` nor this file declares a
; `defvocabulary` naming `EventType`, and `check_enum_ref_membership`'s own
; test suite (`rust/crates/babylon-bsl/src/grammar.rs`,
; `an_undeclared_kind_under_a_partial_vocabulary_is_inert_at_the_rule_producer`)
; documents that an UNDECLARED kind's checking "must stay inert" — so this
; name loads clean today. `babylon-kernel::event_bus`'s own module doc
; states the 100-value domain enum itself "lands with babylon-domain/
; babylon-engine in Phase 2/3, not here" — there is no Rust-side registry
; to mis-sync with yet. Minting the Python member is a forward-declared,
; deliberately out-of-scope follow-up (Python is frozen/reference-only,
; ADR172/Amendment AE) rather than a change this content task makes to a
; file outside its own Files list.
;
; THE REAL-LANE POPULATION FINDING (empirically measured, not paper-
; derived — the SAME species of gap T4.3's Currency-drain spike found,
; one operator over). T2's reconciliation record (§3 unit table) states
; "w-bar = wealth / population ... licensed, ADR202 R1(c) / D181" — TRUE
; of the KIND axis (extensive / extensive -> intensive) but SILENT on a
; SEPARATE question D181 never reaches: `social-class/population` is
; declared `int`, but `tick.rs::bind_field_value` renders EVERY non-enum,
; non-currency field as `Value::Real` at runtime regardless of its
; declared type (D101's own doc: "every non-enum field is unchanged:
; Value::Real(stored)") -- so a plain `(/ wealth population)` is, at
; runtime, `Currency / Real`, which `apply_arith`'s own match arms refuse
; unconditionally as `E-TYPE-030` (measured directly against this rule:
; "Currency / Real(100.0) is not in the §3.2 operator table" -- there is
; no Currency/Real fallback for `/`, unlike `*`'s in-[0,1]-coefficient
; arm). The fix is the SAME `floor` demotion `vitality.bsl`'s own header
; already cites as clearing Grinding Attrition's Real->Int blocker
; (ADR188 Row 2, D97: `(real) -> int`, exact IEEE-754, population is
; always non-negative and integer-valued by construction so the
; demotion loses nothing): `population-int` binds `(floor population)`,
; and `w-bar` divides by THAT, landing on the pinned "÷ integer" leg of
; the five legal Currency operations. **BOTH bindings are `if`-GUARDED**
; (review I-2, a SECOND real finding, empirically confirmed): `:expr`
; bindings resolve for EVERY subject BEFORE the `when` guard runs
; (`tick.rs`'s `collect_pass` order), so `(> population 0)` in the guard
; below does NOT protect an unguarded division -- a `population = 0`
; class would trip `E-EVAL-012` (division by zero) and a NEGATIVE
; population would trip `floor`'s own `E-EVAL-039`, and EITHER ABORTS
; THE WHOLE TICK, not just that subject. Nested `if` (never a clamp)
; makes both bindings TOTAL: `population <= 0` short-circuits to a value
; that is never observed downstream, because the `when` guard excludes
; that subject from firing regardless. **Kind-checking cost, disclosed,
; UPDATED by the guard fix**: `expr_kind` cannot see through an
; intrinsic call (`list_kind`'s dispatch has no `floor` arm), so before
; guarding, `population-int`'s kind was UNDETERMINED (`None`). Guarding
; it changes this: `if_kind`'s documented F8 behavior (typecheck.rs,
; "when only one branch is determined, propagate that branch's kind
; rather than declining") now fires, because the guard's ELSE branch is
; a bare literal (`0` / `0$`, always kind-NEUTRAL) while the THEN branch
; stays undetermined -- so `population-int` resolves DETERMINED-but-
; WRONG as Neutral, and `w-bar` resolves DETERMINED-but-WRONG as
; Extensive (absorbing that Neutral through the licensed `/`
; neutral-absorption rule) rather than the dimensionally correct
; Intensive D181 licenses. `check_kind_mixing` never RAISES here --
; nothing downstream cross-checks `w-bar`'s kind against an
; independently-kinded sibling, since every `edge-k`/`clearing`/
; `failing-certain`/`straddle-band` binding stays inside this SAME
; self-contained chain -- so the mislabeling has zero functional
; consequence, confirmed by the full test suite loading and running
; green. The values themselves are unaffected; only a static label is,
; and it was already undetermined (not correct) before this fix. **Co-load hazard,
; disclosed** (the SAME shape as the filed #646): `territory.bsl` and
; `decomposition.bsl` each already declare a byte-identical
; `(intrinsic floor ...)`, and the loader refuses a duplicate declaration
; BY NAME (`declarations.rs:1010-1017`), so a content set that ever
; bundles this file with either of those two dies at load with
; `E-LOAD-001`. Not live today: this file loads ALONE, paired only with
; `vitality-attrition-conformance.bscn`, in every test this crate runs
; against it — recorded here so a future bundling does not rediscover
; the collision from scratch.
(intrinsic floor :params (real) :returns int :cost 5)

(rule vitality/subsistence-clearing
  :role mechanic
  :evidence derived
  :material-basis "The dual within-class subsistence measure (H2', design doc §6.2; ADR173's P(S|A)): w-bar = wealth/population, guarded total for population<=0; s-stock = (s-bio + s-class) * tau, the ADR210 R13 acquiescence level set. edge-k = cut-k * w-bar. clearing = mass in rungs 2..16 whose lower edge (cut_{k-1}) clears s-stock (rung 1 excluded by construction, its lower edge is the unspellable 0.0r). failing-certain = mass in rungs 1..15 whose upper edge (cut_k) sits wholly below s-stock (rung 16 open above, f-16 definitionally 0). straddle-band = mass-sum - clearing - failing-certain, the straddled rung, complemented against the BOUND total not a stipulated 1 (C-7). Every op is a multiply, add, comparison or rung-membership test -- no exponent, sigmoid or interpolation (S-7, ADR172 r5). DP-6-neutral: both duals land, neither is the mortality driver; T6 picks."
  ; §3.7 cost model: cost(literal)=0, cost(var-ref)=1, cost(arith|cmp)=
  ; 1+Sigma-children, cost(if)=1+cost(cond)+max(then,else), cost(intrinsic
  ; call)=5+declared-cost+Sigma-args, cost(:expr binding)=cost(expr),
  ; bound(rule)=Sigma cost(:expr bindings)+cost(when cond)+Sigma
  ; cost(effect-items). Measured (not guessed): temporarily lowering
  ; :fuel to 1 and reading the E-LOAD-040 message gives the exact static
  ; bound -- "rule vitality/subsistence-clearing static bound 324 exceeds
  ; its declared :fuel 1" -- the same technique consciousness.bsl's own
  ; :fuel re-measurement note uses. 512 leaves 188 units of documented
  ; slack, this pack's own convention (solidarity: measured+1;
  ; consciousness: ~156 slack; the existing vitality/subsistence-and-death
  ; rule already uses this same round number).
  :fuel 512
  (bindings
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding s-bio :field social-class/s-bio)
    (binding s-class :field social-class/s-class)
    ; H1's absence fence (design doc §6.2, D192's own citation of the
    ; UNPOSITIONED idiom, `content/rules/consciousness.bsl`'s `:optional
    ; :default 0.0p`, applied one construct over at `0.0c` -- the field's
    ; own declared Coefficient kind, so no L-5 kind-straddle: the default
    ; literal's suffix matches the target field's declared type exactly,
    ; unlike an optional CURRENCY field defaulted with a bare `0` (T3
    ; review carry, L-5) -- this rule declares no such field, so that gap
    ; is avoided by shape rather than patched.
    (binding mass-01 :field social-class/wealth-mass-01 :optional :default 0.0c)
    (binding mass-02 :field social-class/wealth-mass-02 :optional :default 0.0c)
    (binding mass-03 :field social-class/wealth-mass-03 :optional :default 0.0c)
    (binding mass-04 :field social-class/wealth-mass-04 :optional :default 0.0c)
    (binding mass-05 :field social-class/wealth-mass-05 :optional :default 0.0c)
    (binding mass-06 :field social-class/wealth-mass-06 :optional :default 0.0c)
    (binding mass-07 :field social-class/wealth-mass-07 :optional :default 0.0c)
    (binding mass-08 :field social-class/wealth-mass-08 :optional :default 0.0c)
    (binding mass-09 :field social-class/wealth-mass-09 :optional :default 0.0c)
    (binding mass-10 :field social-class/wealth-mass-10 :optional :default 0.0c)
    (binding mass-11 :field social-class/wealth-mass-11 :optional :default 0.0c)
    (binding mass-12 :field social-class/wealth-mass-12 :optional :default 0.0c)
    (binding mass-13 :field social-class/wealth-mass-13 :optional :default 0.0c)
    (binding mass-14 :field social-class/wealth-mass-14 :optional :default 0.0c)
    (binding mass-15 :field social-class/wealth-mass-15 :optional :default 0.0c)
    (binding mass-16 :field social-class/wealth-mass-16 :optional :default 0.0c)
    (binding tau :const vitality/subsistence-horizon)
    (binding cut-01 :const wealth-sketch/cut-01)
    (binding cut-02 :const wealth-sketch/cut-02)
    (binding cut-03 :const wealth-sketch/cut-03)
    (binding cut-04 :const wealth-sketch/cut-04)
    (binding cut-05 :const wealth-sketch/cut-05)
    (binding cut-06 :const wealth-sketch/cut-06)
    (binding cut-07 :const wealth-sketch/cut-07)
    (binding cut-08 :const wealth-sketch/cut-08)
    (binding cut-09 :const wealth-sketch/cut-09)
    (binding cut-10 :const wealth-sketch/cut-10)
    (binding cut-11 :const wealth-sketch/cut-11)
    (binding cut-12 :const wealth-sketch/cut-12)
    (binding cut-13 :const wealth-sketch/cut-13)
    (binding cut-14 :const wealth-sketch/cut-14)
    (binding cut-15 :const wealth-sketch/cut-15)
    ; The sum-guard (design doc §6.2 H1): a sum of zero IS "no
    ; distribution" -- the `when` clause below excludes a class carrying
    ; it, never a fabricated uniform share.
    (binding mass-sum :expr
      (+ mass-01 (+ mass-02 (+ mass-03 (+ mass-04 (+ mass-05 (+ mass-06
      (+ mass-07 (+ mass-08 (+ mass-09 (+ mass-10 (+ mass-11 (+ mass-12
      (+ mass-13 (+ mass-14 (+ mass-15 mass-16))))))))))))))))
    ; population-int demotes the real-lane population read to a genuine
    ; Int (see this file's header, "THE REAL-LANE POPULATION FINDING") so
    ; `w-bar` can land on Currency's pinned "÷ integer" leg. GUARDED
    ; (review I-2): `:expr` bindings resolve for EVERY subject BEFORE the
    ; `when` guard runs (tick.rs's collect_pass order), so `(> population
    ; 0)` in the guard below does NOT protect this expression -- an
    ; unguarded `(floor population)` trips `E-EVAL-039` for a negative
    ; population, and an unguarded division by a zero population trips
    ; `E-EVAL-012`, ABORTING THE WHOLE TICK, not just this subject. The
    ; nested `if` (never a clamp) makes both bindings TOTAL: population
    ; <= 0 short-circuits to a value that is never observed, because the
    ; `when` guard excludes that subject from firing regardless. PR #680
    ; review: the w-bar guard reads `population-int` (the FLOORED value),
    ; not `population` — a fractional population in (0, 1) would floor to
    ; 0 and the old `(> population 0)` guard would still divide by it
    ; (E-EVAL-012). Latent only — an `int` field refuses a fractional seed
    ; at read (E-LEX-021) and the mortality rule's own writes stay integral
    ; — but the totality claim above now holds literally, not just at
    ; integer reads.
    (binding population-int :expr (if (> population 0) (floor population) 0))
    ; w-bar = wealth / population (Currency / member, D181's licensed
    ; extensive-div-extensive -> intensive).
    (binding w-bar :expr (if (> population-int 0) (/ wealth population-int) 0$))
    ; The ADR210 R13 acquiescence level set: S = s-bio + s-class.
    (binding s-level :expr (+ s-bio s-class))
    ; s-stock = S * tau -- held out for tau ticks (Currency/member, D188).
    (binding s-stock :expr (* s-level tau))
    ; The fifteen `cut-k :expr` bindings (T5.4): each cut's dollar edge at
    ; THIS class's own mean wealth. `edge-k` names `cut_k * w-bar` --
    ; Ratio(neutral) x Currency(intensive), D181's licensed extensive-
    ; scaling arm.
    (binding edge-01 :expr (* cut-01 w-bar))
    (binding edge-02 :expr (* cut-02 w-bar))
    (binding edge-03 :expr (* cut-03 w-bar))
    (binding edge-04 :expr (* cut-04 w-bar))
    (binding edge-05 :expr (* cut-05 w-bar))
    (binding edge-06 :expr (* cut-06 w-bar))
    (binding edge-07 :expr (* cut-07 w-bar))
    (binding edge-08 :expr (* cut-08 w-bar))
    (binding edge-09 :expr (* cut-09 w-bar))
    (binding edge-10 :expr (* cut-10 w-bar))
    (binding edge-11 :expr (* cut-11 w-bar))
    (binding edge-12 :expr (* cut-12 w-bar))
    (binding edge-13 :expr (* cut-13 w-bar))
    (binding edge-14 :expr (* cut-14 w-bar))
    (binding edge-15 :expr (* cut-15 w-bar))
    ; clearing's fifteen guarded terms, rungs 2..16, lower edge cut_{k-1}
    ; (edge-(k-1)) -- rung 1 carries NO term (its lower edge is the
    ; implicit, unspellable 0). Guards are nested `if`, never a clamp.
    (binding c-02 :expr (if (>= edge-01 s-stock) mass-02 0.0c))
    (binding c-03 :expr (if (>= edge-02 s-stock) mass-03 0.0c))
    (binding c-04 :expr (if (>= edge-03 s-stock) mass-04 0.0c))
    (binding c-05 :expr (if (>= edge-04 s-stock) mass-05 0.0c))
    (binding c-06 :expr (if (>= edge-05 s-stock) mass-06 0.0c))
    (binding c-07 :expr (if (>= edge-06 s-stock) mass-07 0.0c))
    (binding c-08 :expr (if (>= edge-07 s-stock) mass-08 0.0c))
    (binding c-09 :expr (if (>= edge-08 s-stock) mass-09 0.0c))
    (binding c-10 :expr (if (>= edge-09 s-stock) mass-10 0.0c))
    (binding c-11 :expr (if (>= edge-10 s-stock) mass-11 0.0c))
    (binding c-12 :expr (if (>= edge-11 s-stock) mass-12 0.0c))
    (binding c-13 :expr (if (>= edge-12 s-stock) mass-13 0.0c))
    (binding c-14 :expr (if (>= edge-13 s-stock) mass-14 0.0c))
    (binding c-15 :expr (if (>= edge-14 s-stock) mass-15 0.0c))
    (binding c-16 :expr (if (>= edge-15 s-stock) mass-16 0.0c))
    (binding clearing :expr
      (+ c-02 (+ c-03 (+ c-04 (+ c-05 (+ c-06 (+ c-07 (+ c-08 (+ c-09
      (+ c-10 (+ c-11 (+ c-12 (+ c-13 (+ c-14 (+ c-15 c-16)))))))))))))))
    ; failing-certain's fifteen guarded terms, rungs 1..15, upper edge
    ; cut_k (edge-k) -- the SAME fifteen edges, the opposite comparison
    ; (§6.2 H2'). Rung 16 carries NO term (f-16 is definitionally 0, open
    ; above -- nothing establishes its failure).
    (binding f-01 :expr (if (< edge-01 s-stock) mass-01 0.0c))
    (binding f-02 :expr (if (< edge-02 s-stock) mass-02 0.0c))
    (binding f-03 :expr (if (< edge-03 s-stock) mass-03 0.0c))
    (binding f-04 :expr (if (< edge-04 s-stock) mass-04 0.0c))
    (binding f-05 :expr (if (< edge-05 s-stock) mass-05 0.0c))
    (binding f-06 :expr (if (< edge-06 s-stock) mass-06 0.0c))
    (binding f-07 :expr (if (< edge-07 s-stock) mass-07 0.0c))
    (binding f-08 :expr (if (< edge-08 s-stock) mass-08 0.0c))
    (binding f-09 :expr (if (< edge-09 s-stock) mass-09 0.0c))
    (binding f-10 :expr (if (< edge-10 s-stock) mass-10 0.0c))
    (binding f-11 :expr (if (< edge-11 s-stock) mass-11 0.0c))
    (binding f-12 :expr (if (< edge-12 s-stock) mass-12 0.0c))
    (binding f-13 :expr (if (< edge-13 s-stock) mass-13 0.0c))
    (binding f-14 :expr (if (< edge-14 s-stock) mass-14 0.0c))
    (binding f-15 :expr (if (< edge-15 s-stock) mass-15 0.0c))
    (binding failing-certain :expr
      (+ f-01 (+ f-02 (+ f-03 (+ f-04 (+ f-05 (+ f-06 (+ f-07 (+ f-08
      (+ f-09 (+ f-10 (+ f-11 (+ f-12 (+ f-13 (+ f-14 f-15)))))))))))))))
    ; straddle-band = mass-sum - clearing - failing-certain (C-7 repair,
    ; review I-1): complements against the BOUND mass-sum, not a
    ; stipulated `1.0c`. The guard below admits any `mass-sum > 0`, not
    ; `mass-sum = 1` -- a partially-seeded class (some of the sixteen
    ; masses absent, defaulting `0.0c`) would otherwise have its UNSEEDED
    ; mass silently reported as "the rung the threshold straddles", which
    ; is a measurement claim about members whose position was never
    ; recorded -- fabrication under §8's L-ABS/ADR070 no-exemption
    ; clause. Complementing against `mass-sum` instead: exact wherever
    ; the masses do sum to 1 (the committed fixture, by construction, per
    ; `every_seeded_class_reads_all_sixteen_masses_summing_to_exactly_one`),
    ; and a true partial measure otherwise -- absent mass stays absent
    ; from every one of the three quantities, never assigned to any of
    ; them.
    (binding straddle-band :expr (- mass-sum (+ clearing failing-certain))))
  (when (and (= active 1) (> population 0) (> mass-sum 0)))
  (effects
    ; The measure's only observable channel: `emit` never touches graph
    ; state (III.11's own boundary — no `update-node`/`update-edge`/
    ; `update-hyperedge`/`add-*`/`remove-*` verb appears anywhere in this
    ; rule), so the K=16 carrier's own pin
    ; (`vitality_attrition_carrier_hashes_are_pinned`) stays byte-
    ; identical to T4's measurement even though this rule now fires for
    ; four of the six classes (T5.7: "a binding and a condition, no
    ; effect"). `entity-id` carries `self` — the same self-identification
    ; idiom `vitality/subsistence-and-death`'s own `ENTITY_DEATH` emit
    ; uses, so a multi-subject tick's events are attributable back to
    ; their originating class. `mass-sum` rides the payload too (review
    ; I-1): so a consumer -- or this crate's own conformance suite -- can
    ; assert the dual-plus-straddle identity against the ACTUAL bound
    ; total rather than a hardcoded `1.0`, making a partially-seeded
    ; class's short mass-sum VISIBLE instead of silently absorbed.
    (emit EventType/SUBSISTENCE_CLEARANCE_MEASURED
      (entity-id self)
      (w-bar w-bar)
      (s-stock s-stock)
      (mass-sum mass-sum)
      (clearing clearing)
      (failing-certain failing-certain)
      (straddle-band straddle-band))))

; Grinding Attrition, ported (#491 T6, Phase 3b; ADR191 R3, ADR194 R1). The
; MEASURE (`vitality/subsistence-clearing`, above) is DP-6-neutral by
; construction -- this rule is the one that picks a mass and turns it into a
; population write. DP-6 = B (delegated Director provenance, posted on #491,
; 2026-08-18): the mortality driver is `failing-certain` (H2''s dual), NOT
; `(- 1.0c clearing)` -- so this rule owes a D-row recording the departure
; from OQ-H's originally-ruled `failing = 1 - clearing` form (D199), never
; neither. `deaths = floor(population * failing-certain * kappa)`,
; `kappa` a DERIVED (not picked) `.bscn` defconst (ADR210 R14; derivation +
; the divergence-surface exhibit: D198).
;
; WHY THIS RULE RE-DERIVES failing-certain FROM SCRATCH. BSL has no
; cross-rule binding reuse (§4's rule model: one rule, one self-contained
; chain) and `vitality/subsistence-clearing` is emit-only, so its published
; `failing-certain` cannot be read back as an input here -- this rule's own
; `active`..`f-15` bindings below are the SAME H2' chain
; `vitality/subsistence-clearing` computes, independently re-run, not a
; second derivation under a different name (H3's own law, design doc §6.2:
; "one measure, many horizons/consumers, never a second implementation" --
; the CHAIN is identical; only the consequence attached to its output
; differs between the two rules).
;
; THE RULE-ORDERING HAZARD, disclosed (D200; the SAME latent gap
; `babylon-tick::run_prepared_tick`'s own header names -- "rules within one
; system position observe the same pre-state" per §4.2 is NOT what today's
; multi-rule sequencing gives for free; a later rule sees an EARLIER rule's
; WRITES from the same tick, not genuine shared pre-state, D-row Q14 --
; latent until now because every landed pack kept one rule per system
; position). `vitality-attrition.bsl` carries two rules as of this task,
; making Q14 LIVE for the first time. This rule's own name is chosen
; DELIBERATELY so ascending rule-id BYTE ORDER (§4.2, D16 -- the only order
; `prepare_rules` honors) runs `vitality/subsistence-clearing` FIRST:
; "subsistence-clearing" < "subsistence-mortality" (`c` < `m`), so the
; MEASURE always reads genuine pre-tick state, never a population this
; rule has already decremented in the same tick. This rule's own
; correctness does not depend on the ordering either way (it never reads
; anything the OTHER rule writes -- `vitality/subsistence-clearing` writes
; nothing at all), but `vitality/subsistence-clearing`'s published readings
; would be silently wrong for a subject this rule kills, in the OTHER
; order. Not a general fix for Q14 -- a naming discipline this ONE pack
; adopts until Q14 lands its own repair.
;
; deaths REDUCE POPULATION AND NEVER WEALTH (ADR183, transcribed from
; `engine/systems/vitality.py:114-131`, re-verified against
; `p27-python-freeze` this pass): the frozen loop's Phase 2 (Grinding
; Attrition) writes ONLY `population` via `graph.update_node(node.id,
; population=new_population)` -- Phase 1's OWN wealth write (the drain) is
; a SEPARATE, EARLIER statement this rule pack does not carry (see below).
; THE DECREMENT IS FLOORED: `deaths = int(population * attrition_rate)`
; (`_calculate_deaths`, truncation toward zero on a non-negative operand,
; i.e. floor) -- this rule's own `(floor raw-deaths)` is the direct port.
; THE TWO CONTINUE GUARDS (`if not attrs.get("active", True): continue` /
; `if population <= 0: continue`, lines 106-107 and 112-113) are this
; rule's `when` clause below, verbatim in effect -- no THIRD guard
; (`vitality/subsistence-clearing`'s own `mass-sum > 0`) is added: an
; unseeded class's sixteen absent masses default `0.0c` (H1's own idiom),
; so `failing-certain` reads a true `0` for it, `deaths` floors to `0`, and
; the inner `(guard (> deaths 0) ...)` below suppresses the emit -- no
; claim is published about a class whose distribution is unmeasured,
; without needing a THIRD outer guard to say so.
;
; WHAT THIS RULE DOES NOT TRANSCRIBE (ADR183: structure contract, not
; correctness oracle; ADR191 R3: SUBSTITUTE, never transcribe the SHAPE).
; "Attrition runs after the drain, off the re-read post-drain node"
; (`vitality.py:114-131`'s own ordering) has no material analogue in this
; rule pack: T4's own scope ruling (OQ-D, design doc §12 item 1, restated
; verbatim by this file's own header above) is carrier-only, and T4.3's
; Currency-drain spike (this file's header, `currency-drain-spike-attempt`)
; PROVED the frozen drain's association order is not expressible in the
; Currency lane as written -- so no drain rule exists in this pack for a
; "runs after" ordering to hold against. The fact is preserved here as a
; citation for a future consolidated pack (one that lands the hydrated
; `currency extensive` cost field this file's header names as the drain's
; own fallback route), not as an executable ordering test in this one.
; `calculate_mortality_rate`'s internal `clamp(0, 1)` and its
; `attrition_base_factor + inequality` slope are the SHAPE being
; substituted (kappa's whole job, §3.5) -- not transcribed, and not owed a
; structure-contract row: SHAPE substitution is this task's mandate, not
; a divergence from it.
(rule vitality/subsistence-mortality
  :role mechanic
  :evidence derived
  :material-basis "Grinding Attrition, ported (DP-6 = B, design doc S6.2 H2'): deaths = floor(population * failing-certain * kappa); the driver is failing-certain (H2''s dual), never (- 1.0c clearing) -- D199 records the departure from OQ-H's ruled form. w-bar/s-stock/edge-k/f-k re-run vitality/subsistence-clearing's H2' chain independently (no cross-rule binding reuse in BSL). kappa (Coefficient, DERIVED not picked, ADR210 R14; D198 records the fixture + divergence surface) converts the certainly-failing share into a per-tick death flow; the product stays in [0,1] by construction, so no clamp (S3.10's rider against scalar min/max). Transcribed from the frozen engine (ADR183, engine/systems/vitality.py:114-131): deaths reduce population never wealth, floored, with the two continue guards (active, population>0). Retires attrition_base_factor (ADR191 R3) and the inequality-dispersion surrogate (S3.3b) -- shape lives in the measured K=16 distribution, not a tuned knob."
  ; Fuel: re-measured the same way vitality/subsistence-clearing's own
  ; comment documents -- temporarily lower :fuel to 1 and read the
  ; E-LOAD-040 message for the exact static bound, then round up leaving
  ; documented slack (this pack's own convention).
  :fuel 512
  (bindings
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding s-bio :field social-class/s-bio)
    (binding s-class :field social-class/s-class)
    (binding mass-01 :field social-class/wealth-mass-01 :optional :default 0.0c)
    (binding mass-02 :field social-class/wealth-mass-02 :optional :default 0.0c)
    (binding mass-03 :field social-class/wealth-mass-03 :optional :default 0.0c)
    (binding mass-04 :field social-class/wealth-mass-04 :optional :default 0.0c)
    (binding mass-05 :field social-class/wealth-mass-05 :optional :default 0.0c)
    (binding mass-06 :field social-class/wealth-mass-06 :optional :default 0.0c)
    (binding mass-07 :field social-class/wealth-mass-07 :optional :default 0.0c)
    (binding mass-08 :field social-class/wealth-mass-08 :optional :default 0.0c)
    (binding mass-09 :field social-class/wealth-mass-09 :optional :default 0.0c)
    (binding mass-10 :field social-class/wealth-mass-10 :optional :default 0.0c)
    (binding mass-11 :field social-class/wealth-mass-11 :optional :default 0.0c)
    (binding mass-12 :field social-class/wealth-mass-12 :optional :default 0.0c)
    (binding mass-13 :field social-class/wealth-mass-13 :optional :default 0.0c)
    (binding mass-14 :field social-class/wealth-mass-14 :optional :default 0.0c)
    (binding mass-15 :field social-class/wealth-mass-15 :optional :default 0.0c)
    (binding mass-16 :field social-class/wealth-mass-16 :optional :default 0.0c)
    (binding tau :const vitality/subsistence-horizon)
    (binding kappa :const vitality/kappa)
    (binding cut-01 :const wealth-sketch/cut-01)
    (binding cut-02 :const wealth-sketch/cut-02)
    (binding cut-03 :const wealth-sketch/cut-03)
    (binding cut-04 :const wealth-sketch/cut-04)
    (binding cut-05 :const wealth-sketch/cut-05)
    (binding cut-06 :const wealth-sketch/cut-06)
    (binding cut-07 :const wealth-sketch/cut-07)
    (binding cut-08 :const wealth-sketch/cut-08)
    (binding cut-09 :const wealth-sketch/cut-09)
    (binding cut-10 :const wealth-sketch/cut-10)
    (binding cut-11 :const wealth-sketch/cut-11)
    (binding cut-12 :const wealth-sketch/cut-12)
    (binding cut-13 :const wealth-sketch/cut-13)
    (binding cut-14 :const wealth-sketch/cut-14)
    (binding cut-15 :const wealth-sketch/cut-15)
    ; population-int/w-bar: the SAME guarded-total pattern
    ; vitality/subsistence-clearing's own header derives at length (review
    ; I-2/D197) -- :expr bindings resolve for EVERY subject BEFORE the
    ; `when` guard runs, so both must be TOTAL on their own, not rely on
    ; `(> population 0)` below to protect them. The w-bar guard reads the
    ; FLOORED value (`population-int`), so a fractional (0, 1) read cannot
    ; route the division to zero (PR #680 review; latent-only today —
    ; E-LEX-021 refuses a fractional seed on the int-declared field).
    (binding population-int :expr (if (> population 0) (floor population) 0))
    (binding w-bar :expr (if (> population-int 0) (/ wealth population-int) 0$))
    (binding s-level :expr (+ s-bio s-class))
    (binding s-stock :expr (* s-level tau))
    (binding edge-01 :expr (* cut-01 w-bar))
    (binding edge-02 :expr (* cut-02 w-bar))
    (binding edge-03 :expr (* cut-03 w-bar))
    (binding edge-04 :expr (* cut-04 w-bar))
    (binding edge-05 :expr (* cut-05 w-bar))
    (binding edge-06 :expr (* cut-06 w-bar))
    (binding edge-07 :expr (* cut-07 w-bar))
    (binding edge-08 :expr (* cut-08 w-bar))
    (binding edge-09 :expr (* cut-09 w-bar))
    (binding edge-10 :expr (* cut-10 w-bar))
    (binding edge-11 :expr (* cut-11 w-bar))
    (binding edge-12 :expr (* cut-12 w-bar))
    (binding edge-13 :expr (* cut-13 w-bar))
    (binding edge-14 :expr (* cut-14 w-bar))
    (binding edge-15 :expr (* cut-15 w-bar))
    ; failing-certain's fifteen guarded terms, rungs 1..15, upper edge
    ; cut_k (edge-k) -- verbatim the SAME chain
    ; vitality/subsistence-clearing computes (H2', design doc S6.2). Rung
    ; 16 carries no term (f-16 is definitionally 0, open above).
    (binding f-01 :expr (if (< edge-01 s-stock) mass-01 0.0c))
    (binding f-02 :expr (if (< edge-02 s-stock) mass-02 0.0c))
    (binding f-03 :expr (if (< edge-03 s-stock) mass-03 0.0c))
    (binding f-04 :expr (if (< edge-04 s-stock) mass-04 0.0c))
    (binding f-05 :expr (if (< edge-05 s-stock) mass-05 0.0c))
    (binding f-06 :expr (if (< edge-06 s-stock) mass-06 0.0c))
    (binding f-07 :expr (if (< edge-07 s-stock) mass-07 0.0c))
    (binding f-08 :expr (if (< edge-08 s-stock) mass-08 0.0c))
    (binding f-09 :expr (if (< edge-09 s-stock) mass-09 0.0c))
    (binding f-10 :expr (if (< edge-10 s-stock) mass-10 0.0c))
    (binding f-11 :expr (if (< edge-11 s-stock) mass-11 0.0c))
    (binding f-12 :expr (if (< edge-12 s-stock) mass-12 0.0c))
    (binding f-13 :expr (if (< edge-13 s-stock) mass-13 0.0c))
    (binding f-14 :expr (if (< edge-14 s-stock) mass-14 0.0c))
    (binding f-15 :expr (if (< edge-15 s-stock) mass-15 0.0c))
    (binding failing-certain :expr
      (+ f-01 (+ f-02 (+ f-03 (+ f-04 (+ f-05 (+ f-06 (+ f-07 (+ f-08
      (+ f-09 (+ f-10 (+ f-11 (+ f-12 (+ f-13 (+ f-14 f-15)))))))))))))))
    ; attrition-rate = failing-certain * kappa (Intensive x Neutral-const
    ; -> Intensive, D181's kind algebra; Real x Real -> Real at the value
    ; level, both operands already Value::Real -- no Currency involved,
    ; no new kind-straddle). raw-deaths = population * attrition-rate
    ; (Extensive x Intensive -> Extensive, the licensed "stock scaled by a
    ; dimensionless rate" arm, typecheck.rs's mul_div_kind -- the SAME arm
    ; `lifecycle.bsl`'s new-wealth-d-prime already uses). deaths floors
    ; the product (ADR188 Row 2, D97 -- `floor` already declared at this
    ; file's own top, no re-declaration). new-population subtracts an Int
    ; from population's own Real-lane read (Real - Int -> Real,
    ; apply_arith's fallback real_lane arm) -- written back to the
    ; `int`-declared population field, which `numeric_write_value`
    ; accepts uniformly (both Value::Real and Value::Int fold to f64 at
    ; the store boundary, structural_verbs.rs).
    (binding attrition-rate :expr (* failing-certain kappa))
    (binding raw-deaths :expr (* population attrition-rate))
    (binding deaths :expr (floor raw-deaths))
    (binding new-population :expr (- population deaths)))
  (when (and (= active 1) (> population 0)))
  (effects
    ; The frozen loop's own `if deaths > 0:` gate (vitality.py:130) --
    ; only a subject whose driver actually crosses one whole member gets a
    ; write or an event; a `core`-shaped fed class (failing-certain = 0)
    ; or a fractional product below one member (floor to zero) passes the
    ; `when` guard above but produces neither.
    (guard (> deaths 0)
      (update-node self social-class/population (set new-population))
      (emit EventType/POPULATION_ATTRITION
        (entity-id self)
        (deaths deaths)
        (remaining-population new-population)
        (failing-certain failing-certain)
        (attrition-rate attrition-rate)))))
