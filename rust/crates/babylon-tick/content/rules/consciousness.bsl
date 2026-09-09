; ConsciousnessSystem (Consequences @17.0) — the class-surface measured
; ternary (issue #588, ADR204 W10). Frozen source:
; src/babylon/engine/systems/ideology.py (ConsciousnessSystem, :94-442) with
; the routing law at src/babylon/formulas/consciousness_routing.py:288-370.
; Port posture (the design's own ruling): measured-ternary read path +
; UNPOSITIONED first — transcribe the INPUTS and the ROUTING LAW re-pointed
; at (r, l, f), NOT a cc/ni transcription; the cc/ni bridge mapping is the
; read path's spec, and it lands in a later task.
;
; TASK 1+2+3 SHIP: the NINE-rule pack — `consciousness/p0-position` (the
; class-seeding law, A-001), p1-inbox-reset, p2-org-solidarity-push,
; p3-class-solidarity-push, p4-wage-balance, p5-agitation, p6-route (the
; ADR016 bifurcation law RE-POINTED at the stored ternary — the headliner),
; p7-persist-baselines, and `consciousness/p8-dominant-worldview` (the
; measured readout — the hegemonic tie-break's ONE declared home) — on top
; of the exact qnames consciousness-ternary-conformance.bscn declares. p8
; sorts LAST so the readout reflects the same tick's routing (D116) —
; matching the frozen step's post-update read.
;
; UNPOSITIONED IDIOM (L-ABS, the row-19 disease's death certificate): the
; ternary fields are never defaulted into existence — a never-positioned
; class carries NO r/l/f and NO dominant-worldview, and a raw store read
; errors loud (III.11). Inside a rule, absence is observed ONLY through
; declared literals: `:optional` + `:default 0.0p` bindings, gated on
; `(= (+ r (+ l f)) 0)` — a positioned class is a simplex point (shares sum
; to 1), so a zero sum is EXACTLY never-positioned, by construction, never
; by epsilon and never by a fabricated 0.5 (bindings.rs's own §3.5 law: "no
; rule observes absence — it observes a declared default"). Every later
; rule in this pack optional-binds the ternary the same way and gates on
; `(> (+ r (+ l f)) 0)` for the positive direction.
;
; D116 BYTE-ORDER MAP (docs/reference/bsl-language.rst, the recorded
; cross-rule same-tick visibility divergence this pack deliberately relies
; on, production.bsl-header style): rules run to completion in ascending
; rule-id byte order against the same mutable graph, so the ten rules'
; reads see every earlier rule's same-tick writes. The ordering obligation
; binds every later addition — keep the pN prefixes monotone in the frozen
; engine's own causality order:
;
;   rule                        subject       reads                         writes
;   p0-position                 SOCIAL_CLASS  active, anchors, ternary      r/l/f, agitation (seed)
;   p1-inbox-reset              SOCIAL_CLASS  ternary (sum-guard)           solidarity-inbox, wages-inbox <- 0
;   p2-org-solidarity-push      ORGANIZATION  active; per-edge strength     targets' solidarity-inbox (add),
;                                           strength > 0.01 gate
;   p2-wages-push               SOCIAL_CLASS  active; per-edge value-flow   targets' wages-inbox (add)
;   p3-class-solidarity-push    SOCIAL_CLASS  own r (optional); per-edge    targets' solidarity-inbox (add),
;                                           strength                      r > 0.3 percolation gate
;   p4-wage-balance             SOCIAL_CLASS  wages-paid, value-produced    wage-balance (verbatim f64)
;                                           (optional sentinels)
;   p5-agitation                SOCIAL_CLASS  wages-inbox, previous-*,      agitation (UNDECAYED)
;                                           repression-faced, ternary
;                                           sum-guard, anchors, consts
;   p6-route                    SOCIAL_CLASS  agitation, inbox,             r/l/f (routed + closure),
;                                           wage-balance, ternary, consts   agitation (decayed store)
;   p7-persist-baselines        SOCIAL_CLASS  wages-inbox, wealth,          previous-wages,
;                                           anchors                         previous-wealth
;   p8-dominant-worldview       SOCIAL_CLASS  ternary                       dominant-worldview
;
; Standing witnesses in the conformance world: p0 positions class-emergent
; at (0, 1, 0) and p5/p6 route it THE SAME TICK; p3 reads class-exploited's
; PRE-route r (0.5 > 0.3) for the percolation gate; p8 reads every routed
; ternary the same tick it is written.
;
; D-RECORDS this pack carries (full spike evidence in
; consciousness-ternary-conformance.bscn's header — the two files' records
; are one ledger, split by surface):
;   1. UNPOSITIONED is expressed as `:optional` + `:default 0.0p` + the
;      zero-sum guard (above) — the lawful absence shape in current BSL
;      (no has-field combinator exists; archaeology digest gap 1). The
;      `:default` declarations are deliberate and content-visible;
;      default_lint.rs's allowlist mechanism governs the migration-corpus
;      sites and does not gate this pack's load.
;   2. Anchor presence is tested by SENTINEL, not by a guard combinator:
;      wages-paid / value-produced are non-negative whenever present, so
;      `:default -1` makes `(>= wages 0)` the presence test. An active
;      class with neither anchor (the conformance world's `employer`) is
;      never positioned and never routed.
;   3. ONE-HOME LAW for dominant-worldview (controller ruling 1, fix
;      round): the dominant readout's ONLY writer is
;      consciousness/p8-dominant-worldview (below, landed Task 2) — p0
;      does NOT record dominance at positioning. Task 1 pinned the field
;      ABSENT on a freshly-positioned class as the one-home guard; Task 2
;      flipped that pin to the positive LIBERAL readout (the handoff is
;      recorded in consciousness_ternary_conformance.rs's header), and the
;      absence pins survive only for the genuinely unread
;      (class-unpositioned, employer). History, honestly
;      recorded: this pack's first draft carried a fifth p0 effect
;      writing LIBERAL here, defended as A-001's strict argmax of the
;      rest state; the controller ruled the Task-1 brief's test skeleton
;      had asserted a Task-2 artifact — a plan defect — and the write
;      came OUT. A-001's liberal-default content lives in the (0, 1, 0)
;      positioning itself; the dominance readout is a separate law with
;      its own single home. Task-2 addendum (controller ruling
;      2026-08-15, the Task-2 NEEDS_CONTEXT): the tie-break's strict
;      `< 1e-6` is transcribed verbatim — a decimal-1e-6 gap is NOT a
;      tie (strict `<` excludes the boundary; the conformance world's
;      tv-strict-gap / tv-tie-all-true pair witnesses both arms against
;      the frozen ground truth).
;   4. consciousness/simplex-epsilon has NO defconst row anywhere
;      (controller ruling 4, fix round): the frozen _EPSILON = 1e-10
;      (consciousness_routing.py:41) is inexpressible as a p/i/c literal
;      (E-LEX-023, scale <= 9, reader.rs::classify_unit_interval), so the
;      consuming rule (Task 3) binds it as the expr quotient
;      `(/ 1c 10000000000)` — bit-identical to Python's 1e-10 via one
;      correctly-rounded IEEE-754 division, dodging the lex bound
;      entirely.
;   5. Rounding law (spike 4's verdict; controller ruling 3's verbatim
;      text, the Task-3 conformance mirror's D-row): "no implicit
;      truncation at the store; the int lane holds verbatim f64; `floor`
;      is the content-explicit truncation intrinsic, unused in this
;      pack." The two fix rounds mapped the FULL law: fractional WRITES
;      land verbatim in the int lane, fractional SEEDS are loader-refused
;      (scenario.rs::attribute_value_int, exact error quoted in the
;      scenario header). The fix-round-1 escape — agitation declared
;      `intensity` to admit fractional seeds — was RETIRED by the
;      fix-round-2 ruling: the unit-interval E-EVAL-020 [0,1] ceiling is
;      tick-fatal under the frozen [0,∞) agitation domain — the zero-seed
;      tick-1 undecayed write is already 1.0 (class-bribed), and the
;      accumulator crosses the ceiling at tick 2 (0.9 decayed + 1.0 fresh
;      = 1.9). All three
;      machinery accumulators were therefore int-typed verbatim-f64;
;      agitation seeds at 0 (a produced accumulator, R-MEASURED), the
;      others unseeded until their writing tasks. The verbatim-f64 int
;      lane this row records was RETIRED by Train B item 6
;      (`BslType::Real`): the fields it governed now declare `real`; the
;      store law itself (verbatim f64, no implicit truncation, `floor`
;      available) is unchanged — only the declared home moved.
;   6. The wage flow rides a CLASS-SIDE field (controller ruling 2, fix
;      round): social-class/wages-received carries the per-tick wage
;      inflow as declared content — the frozen engine's incoming-WAGES
;      value_flow fold-sum (ideology.py:299-309) narrows to one declared
;      value per class per tick, exact for single-employer content. The
;      first draft's WAGES edge machinery (three edges + the
;      wages/value-flow edge deffield + social-class/wages-inbox) came
;      OUT: scenario-side edge-attribute seeding is unserved (load_edge
;      is strength-only), a gap recorded for the port train's closing
;      ADR.
;      RETIRED by Train B item 3 (#591, D151's discharge): the WAGES edge
;      machinery described above as having come OUT is back IN —
;      edge-attribute seeding is served (D156's (edge-attr ...) form) and
;      social-class/wages-received is gone; the wage flow now rides the
;      pushed wages-inbox accumulator (consciousness/p2-wages-push). See
;      D151's header row below for the retirement note.
;   7. The `social-class/active` / `organization/active` latches are
;      declared `intensive` here against production-conformance.bscn's and
;      organization-foundation.bscn's `extensive` — a per-node state is
;      never a summed quantity. Kinds are scenario-local; no fold in this
;      pack sums a latch.
;
; REGISTER ROWS this pack landed (docs/reference/bsl-language.rst — full
; text + file:line evidence there; drafted in this header by Task 3, moved
; to the register by Task 4, one line per row here per the production.bsl
; pointer form):
;   D146. The RE-POINTED ACCUMULATOR (the headliner) — the ternary is
;      stored and updated directly; Δl APPLIED; verbatim
;      normalize_to_simplex closure replaces the frozen per-axis clamps.
;   D147. Curve-5 Gaussian NOT transcribed (ADR202 R7); the linear
;      chauvinist pass-through IS (Director flag 2).
;   D148. wage_deterioration stubbed 0.0c (no graph-attr BSL surface).
;   D149. popular_front_suppression stubbed 0.0c (electoral register
;      absent; exact under register-absent content).
;   D150. material_conditions buffer write not ported (no ported
;      consumers).
;   D151. Solidarity pull→push redesign (the D136 fix-round pattern) +
;      the per-pair multi-edge unreachability record (both backends
;      refuse a duplicate (type, from, to) edge), the strength <= 0
;      skip, and the WAGES fold-sum → wages-received narrowing —
;      narrowing retired by Train B item 3 — the wage flow rides seeded
;      WAGES-edge wages/value-flow via push-over-pull; wages-received is
;      gone.
;   D152. Class-source percolation re-pointed at the source's r share.
;   D153. Positioned-only agitation (the anchored ∧ positioned guard).
;   D154. Same-tick closure heal observed (D116; tv-tie-all-true).
;
; "consciousness" is an ALREADY-registered system namespace
; (babylon-tick/src/lib.rs's systems set — the worldview mint's probe
; anchored under it first), so this pack changes no Rust source.

(rule consciousness/p0-position
  :role mechanic
  :evidence derived
  :material-basis "A-001 as the class-seeding law (Director flag 1): a class with material anchors (wages-paid + value-produced present) and no ternary record is positioned at the ruled unorganized rest state (0, 1, 0) — liberal hegemonic default, spec 034 A-001, THE one home (the seven scattered frozen sites are named in docs/concepts/consciousness-taxonomy.rst, not re-homed here). Data-absent classes are never positioned: UNPOSITIONED (L-ABS) — the row-19 disease's death certificate. Positioning does NOT record dominance: dominant-worldview's only writer is the read-path task's dominant rule (one-home law, pack D-record 3) — a freshly-positioned class reads it ABSENT until then. The agitation accumulator initializes to zero so later routing rules read a positioned class's agitation as present."
  :fuel 64
  (bindings
    (binding active :field social-class/active)
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p))
  (when (and (= active 1)
             (>= wages 0)
             (>= value 0)
             (= (+ r (+ l f)) 0)))
  (effects
    (update-node self social-class/revolutionary (set 0.0p))
    (update-node self social-class/liberal (set 1.0p))
    (update-node self social-class/fascist (set 0.0p))
    (update-node self social-class/agitation (set 0))))

(rule consciousness/p1-inbox-reset
  :role mechanic
  :evidence derived
  :material-basis "Per-tick accumulator reset (the production p0 idiom; D103/D104 collect-then-apply makes reset-then-accumulate safe): the solidarity inbox and the wages inbox are both machinery, not state — each carries this tick's pushed contributions only. UNCONDITIONAL (W2 repair, adjudication (d)): the positioned-only sum-guard this rule carried until now was strictly WEAKER than p7-persist-baselines' anchored-only read guard on wages-inbox, so an anchored-but-inactive class (never positioned, so never reset) could accumulate un-reset WAGES-edge pushes indefinitely and later read them as garbage previous-wages on its first positioned tick — a real, reachable latent defect on wages-inbox (solidarity-inbox was already safe: p0 positions a class the same tick p1's old guard would first pass). Resetting BOTH inbox fields unconditionally, every tick, for every subject, discharges both at once and is the E-LOAD-059 unconditional-set shape refusal 2 checks for."
  :fuel 32
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p))
  (when #t)
  (effects
    (update-node self social-class/solidarity-inbox (set 0))
    (update-node self social-class/wages-inbox (set 0))))

(rule consciousness/p2-org-solidarity-push
  :role mechanic
  :evidence derived
  :material-basis "Org-sourced solidarity: strength above negligible_transmission counts (frozen ideology.py:339-356's org arm — org mass work has no ideology of its own to gate on; the edge's strength IS the signal, ADR087). Push form (the D136 fix-round pattern; exact vs the frozen pull at :337-356 — each edge is pushed exactly once by its unique source). One SOLIDARITY edge per (source, target) pair is STRUCTURAL, not discipline: both GraphSubstrate backends key edges on (type, from, to) and refuse a duplicate loudly — the frozen per-edge fold's multi-edge sum is unrepresentable here (recorded, D151)."
  :fuel 128
  (bindings
    (binding active :field organization/active)
    (binding negligible :const consciousness/negligible-transmission))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (guard (> (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength) negligible)
        (update-node it social-class/solidarity-inbox
          (add (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength)))))))

(rule consciousness/p2-wages-push
  :role mechanic
  :evidence derived
  :material-basis "The wage flow, un-narrowed (D151's narrowing 3 discharged, Train B item 3, #591): every WAGES edge's seeded wages/value-flow is pushed into the receiving class's wages-inbox (frozen ideology.py:299-309's incoming-WAGES fold-sum, expressed per the D136 push-over-pull idiom — D138 forbids the filter-in-fold pull). Push form (the D136 fix-round pattern, mirroring p2-org-solidarity-push's own shape exactly): each edge is pushed exactly once by its unique source. No transmission-floor gate — every seeded wage flow counts; the WAGES edge's own strength slot (1.0c, seeded above) is a structural presence marker with no semantic consumer (the register row) — nothing reads it."
  :fuel 128
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/WAGES :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/wages-inbox
        (add (field-of (edge-between EdgeType/WAGES self it) wages/value-flow))))))

(rule consciousness/p3-class-solidarity-push
  :role mechanic
  :evidence derived
  :material-basis "Class-sourced solidarity transmits only past the percolation threshold (frozen: source class_consciousness > activation_threshold, ideology.py:339-356) — re-pointed to the source's revolutionary share (the same quantity post-W1 unification; D152). An UNPOSITIONED source reads r = 0.0p by the idiom and never transmits: absence is not organization. The frozen loop's strength <= 0 skip is not transcribed (inert on declared content; recorded narrowing, D151)."
  :fuel 128
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding threshold :const consciousness/solidarity-activation-threshold))
  (when (> r threshold))
  (effects
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/solidarity-inbox
        (add (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength))))))

(rule consciousness/p4-wage-balance
  :role mechanic
  :evidence derived
  :material-basis "The per-class wage-value balance (formulas/contradiction.py:67-100, called (v_produced, w_paid) at ideology.py:241-244, so balance = (w−v)/(v+w)): positive = wages dominate = the imperial bribe. Frozen reads the per-class pair when present (ideology.py:239-259), which is the ONLY path the port carries: data-absent classes are UNPOSITIONED, never the graph-attr fallback (that attr has no BSL surface). The frozen [-1,1] clamp is inert-by-construction under the non-negative anchored sentinel guard (|w−v| <= v+w). The frozen 1e-9 zero-guard (formulas/contradiction.py:98-100: total <= 1e-9 -> 0.0) is NARROWED, not inert — the port's guard is `(+ wages value) > 0`, so for 0 < v+w <= 1e-9 the frozen yields 0.0 where the port yields the quotient; content-inert on declared content, whose sums sit orders of magnitude above 1e-9 — a recorded narrowing. Stored verbatim-f64, signed (spike 4's lane)."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding balance :expr (if (> (+ wages value) 0)
                               (/ (- wages value) (+ value wages))
                               (- 0 0c))))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/wage-balance (set balance))))

; KIND-COHERENCE REPAIR, shape C1 (Director sitting 2026-08-18, popup:
; repair-now+ceremony; #491 T1's kind-straddle dossier,
; reports/kind-straddle-repair-options-2026-08-18.md §1.1). The frozen call
; site (ideology.py:374) passes raw `wealth_change` — an ABSOLUTE Currency
; delta — as `imperial_rent_delta`. `wealth`/`previous-wealth` are both
; extensive (T1's `previous-wealth` fix, D181), so that raw difference is
; extensive too, and summing it against the exploit/repression terms'
; intensive kind was E-TYPE-040. DELIBERATELY DIVERGED from the frozen
; figure: `wealth-rate = Δwealth ÷ previous-wealth` (extensive ÷ extensive,
; licensed intensive — D90/D181's own unit-algebra standing) replaces the
; absolute currency figure with the PROPORTIONAL wealth-decline rate this
; tick. Aleksandrov test: the term now measures what FRACTION of a class's
; material stake it lost, not how many currency units — matching how
; imperial rent Φ is treated as a ratio everywhere else in this engine
; (MarketScissors's price_value opposition, wage/value ratios), not an
; absolute figure. Value-changing for any class with a nonzero wealth delta
; (this scenario: class-bribed only, increment 1.0 -> 0.010526315789473684
; — see the ceremony record, tests/consciousness_ternary_conformance.rs
; and ai/state.yaml's kind_straddle_estate_491_t1 entry.
; :fuel re-measured post-C1 (review finding F9, #491 T1): the new
; wealth-rate binding's static bound is 68 (measured by temporarily
; lowering :fuel and reading the E-LOAD-040 "static bound N exceeds"
; message, the same method solidarity's :fuel re-measurement used);
; :fuel stays 224, this pack's own round-block convention
; (64/32/128/224/256/96), leaving ~156 units of documented slack rather
; than solidarity's "measured + 1" convention.
(rule consciousness/p5-agitation
  :role mechanic
  :evidence derived
  :material-basis "compute_agitation_delta (consciousness_routing.py:48-200) + the frozen call-site's exact argument mapping (ideology.py:372-380): exploitation_delta = |wage_change| when wages fall; visibility 0.0 verbatim; the Curve-5 balance component ABSENT (ADR202 R7 — the replacement rides #491, D147); repression as produced-excess-over-baseline, absent contributing zero (MEDIUM-2 discipline). The wage flow rides the pushed wages-inbox accumulator (D151's narrowing 3 discharged, Train B item 3, #591 — the frozen incoming-WAGES fold-sum now rides the seeded WAGES-edge wages/value-flow via p2-wages-push's push-over-pull, exact for single-employer content since each class receives exactly one employer edge). Guarded anchored AND positioned (D153: an unpositioned class never accumulates — the frozen step accumulated on every active class). Writes the UNDECAYED level; p6 routes on it and writes the decayed store. wealth-rate (Δwealth ÷ prev-wealth, a RATE, comment above) replaces frozen's wealth_change (#491 T1 C1)."
  :fuel 224
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p)
    (binding wages-in :field social-class/wages-inbox :optional :default 0)
    (binding prev-wages :field social-class/previous-wages :optional :default 0)
    (binding wealth :field social-class/wealth :optional :default 0)
    (binding prev-wealth :field social-class/previous-wealth :optional :default 0)
    (binding rf :field social-class/repression-faced :optional :default 0.5i)
    (binding agitation :field social-class/agitation :optional :default 0)
    (binding exploit-sens :const consciousness/exploitation-sensitivity)
    (binding rent-sens :const consciousness/rent-decline-sensitivity)
    (binding rep-sens :const consciousness/repression-level-sensitivity)
    (binding rep-base :const consciousness/default-repression-faced)
    (binding vis-coeff :const consciousness/reproduction-visibility-coefficient)
    (binding wd-stub :const consciousness/wage-deterioration-stub)
    (binding wage-change :expr (- wages-in prev-wages))
    (binding exploit-delta :expr (if (< wage-change 0) (- 0 wage-change) 0))
    (binding wealth-change :expr (- wealth prev-wealth))
    (binding wealth-rate :expr (if (> prev-wealth 0) (/ wealth-change prev-wealth) 0c))
    (binding increment :expr
      (+ (* (if (> exploit-delta 0) exploit-delta 0) exploit-sens)
         (+ (* (if (> (- 0 wealth-rate) 0) (- 0 wealth-rate) 0) rent-sens)
            (+ (* 0.0c vis-coeff)
               (* (if (> (- rf rep-base) 0) (- rf rep-base) 0) rep-sens)))))
    (binding new-agitation :expr (+ agitation (+ increment wd-stub))))
  (when (and (>= wages 0) (>= value 0) (> (+ r (+ l f)) 0)))
  (effects
    (update-node self social-class/agitation (set new-agitation))))

(rule consciousness/p6-route
  :role mechanic
  :evidence derived
  :material-basis "The ratified bifurcation law (ADR016; route_agitation_to_ternary, consciousness_routing.py:345-370) RE-POINTED at the stored ternary: solidarity routes agitation revolutionary-ward, its absence fascist-ward; chauvinist pressure (the positive-balance imperial bribe, Director flag 2's ruling) biases the split fascist-ward; Δl APPLIED here (the frozen engine discards it at the class call-site, ideology.py:394) — the re-point, D146 — with closure by a verbatim normalize_to_simplex (:373-409). Epsilon rides the expr quotient (/ 1c 10000000000), bit-identical to Python's 1e-10 via one correctly-rounded IEEE-754 division (pack D-record 4). The decay store follows ideology.py:413-414."
  :fuel 256
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p)
    (binding agitation :field social-class/agitation :optional :default 0)   ; the UNPOSITIONED idiom (pack D-record 1): a required binding is TICK-FATAL on an absent field for a same-subject-type node (bindings.rs's resolve-or-error law), never a skip — p5 wrote agitation this tick (D116) for every anchored-positioned class and the tv fixtures carry the zero seed, so the default is unobservable under the sum-guard
    (binding inbox :field social-class/solidarity-inbox :optional :default 0)
    (binding balance :field social-class/wage-balance :optional :default 0)
    (binding consumption :const consciousness/agitation-consumption-rate)
    (binding routing-scale :const consciousness/routing-scale)
    (binding chauv-scale :const consciousness/chauvinist-pressure-scale)
    (binding decay :const consciousness/agitation-decay-rate)
    (binding suppression :const consciousness/popular-front-suppression-stub)
    (binding eps :expr (/ 1c 10000000000))
    (binding consumed :expr (* agitation consumption))
    (binding sol-factor :expr (if (< inbox 1) inbox (- 1 0c)))
    (binding chauvinist :expr (* (if (> balance 0) balance 0) chauv-scale))
    (binding eff-raw-arg :expr (+ sol-factor 0.0c))
    (binding eff-raw :expr (if (< eff-raw-arg 1) eff-raw-arg (- 1 0c)))
    (binding eff-arg :expr (- eff-raw chauvinist))
    (binding eff-sol :expr (if (> eff-arg 0) (if (< eff-arg 1) eff-arg (- 1 0c)) (- 0 0c)))
    (binding delta-r :expr (* (* consumed eff-sol) routing-scale))
    (binding delta-f :expr (* (* (* consumed (- 1 eff-sol)) routing-scale) (- 1 suppression)))
    (binding delta-l :expr (- 0 (+ delta-r delta-f)))
    (binding r1 :expr (if (> (+ r delta-r) 0) (+ r delta-r) (- 0 0c)))
    (binding l1 :expr (if (> (+ l delta-l) 0) (+ l delta-l) (- 0 0c)))
    (binding f1 :expr (if (> (+ f delta-f) 0) (+ f delta-f) (- 0 0c)))
    (binding total :expr (+ r1 (+ l1 f1)))
    (binding r2 :expr (if (> total (+ 1 eps)) (/ r1 total) r1))
    (binding l2 :expr (if (> total (+ 1 eps)) (/ l1 total)
                        (if (< total (- 1 eps)) (+ l1 (- 1 total)) l1)))
    (binding f2 :expr (if (> total (+ 1 eps)) (/ f1 total) f1))
    (binding r-out :expr (if (< total eps) 0.0p r2))
    (binding l-out :expr (if (< total eps) 1.0p l2))
    (binding f-out :expr (if (< total eps) 0.0p f2))
    (binding decayed-arg :expr (* agitation (- 1 decay)))
    (binding decayed :expr (if (> decayed-arg 0) decayed-arg (- 0 0c))))
  (when (> (+ r (+ l f)) 0))
  (effects
    (update-node self social-class/revolutionary (set r-out))
    (update-node self social-class/liberal (set l-out))
    (update-node self social-class/fascist (set f-out))
    (update-node self social-class/agitation (set decayed))))

(rule consciousness/p7-persist-baselines
  :role mechanic
  :evidence derived
  :material-basis "The persistent previous-values re-homed to node fields (digest gap 4 — context.persistent_data has no BSL analog): next tick's deltas read this tick's declared flow (frozen: persistent[PREVIOUS_WAGES_KEY] = current_wages / PREVIOUS_WEALTH_KEY, ideology.py:441-442). Anchored classes only."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding wages-in :field social-class/wages-inbox :optional :default 0)
    (binding wealth :field social-class/wealth :optional :default 0))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/previous-wages (set wages-in))
    (update-node self social-class/previous-wealth (set wealth))))

(rule consciousness/p8-dominant-worldview
  :role mechanic
  :evidence derived
  :material-basis "The measured readout: dominant pole = argmax with the ruled tie order LIBERAL > REVOLUTIONARY > FASCIST at 1e-6 (frozen: models/entities/consciousness.py:177-192, transcribed verbatim). TWO DECLARED HOMES for the hegemonic tie-break, parity-guarded (DG-2 = PUBLISH, the 2026-08-18 sitting): this rule over the class surface, and community/c08-dominant-tendency over the community surface — the 8a row-1 copies-agree test (tests/community_conformance.rs::dominant_tendency_ties_match_the_class_surface_rule) pins both to LIBERAL on the same three-way tie. (The frozen anchor above is corrected with this amendment: models/entities/consciousness.py:167-191, epsilon :189 — the earlier :177-192 pointed past the property.) UNPOSITIONED classes (sum 0) are skipped: no reading, ever."
  :fuel 96
  (bindings
    (binding active :field social-class/active)
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p)
    (binding mx :expr (if (>= r l) (if (>= r f) r f) (if (>= l f) l f)))
    (binding eps :expr 0.000001c)
    (binding dr :expr (if (> r mx) (- r mx) (- mx r)))
    (binding dl :expr (if (> l mx) (- l mx) (- mx l)))
    (binding winner :expr (if (< dl eps) WorldView/LIBERAL
                            (if (< dr eps) WorldView/REVOLUTIONARY
                                WorldView/FASCIST))))
  (when (and (= active 1) (> (+ r (+ l f)) 0)))
  (effects
    (update-node self social-class/dominant-worldview (set winner))))
