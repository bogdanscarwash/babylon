; TerritorySystem (Material Base @2.0) — the settler-colonial spatial
; substrate: "Legibility over Stealth" (state legibility accumulates as
; heat; heat past a threshold triggers eviction; eviction routes displaced
; population to carceral sinks; carceral sinks eliminate/suppress).
;
; Frozen source: src/babylon/engine/systems/territory.py (378 lines, four
; SEQUENTIAL phases run in one step()). Port-as-is (Director ruling):
; frozen defects are transcribed and D-recorded, never silently repaired.
; The frozen engine is a structure/ordering contract, NOT a byte oracle
; (ADR183) — conformance expecteds are measured from THIS BSL engine and
; pinned in territory_conformance.rs, not copied from the frozen mirror's
; printed floats.
;
; FOUR RULES, ONE PER PHASE, BYTE-ORDERED `p1 < p2 < p3 < p4-camp-decay <
; p4-penal-suppression` — deliberately relying on D116's recorded
; cross-rule divergence (docs/reference/bsl-language.rst): today's
; run_once_into/TickSession::advance run each rule in a content set to
; COMPLETION before the next starts, against the SAME mutable graph, so a
; later rule at the same anchor position sees an EARLIER rule's
; already-applied writes from THIS tick. The frozen phases are
; SEQUENTIAL BY DESIGN (eviction reads this-tick post-Phase-1 heat; camp
; decay eats this-tick displaced arrivals) — this pack RELIES on that
; divergence rather than fighting it, and D-record #1 (Task 8) names the
; dependency explicitly for when the Q14 repair train lands a real anchor
; registry.
;
; D-RECORDS this pack transcribes (full text + file:line evidence in the
; Task 8 register rows, docs/reference/bsl-language.rst):
;   1. Phase order relies on D116's byte-order/run-to-completion semantics.
;   2. `under-eviction` is an int 0/1 latch — no Bool store path exists on
;      the live `.bscn` pipeline (reports/territory-bsl-surface-facts-
;      2026-08-12.md §1(c)).
;   3. `rent-level-x1e6` is the scaled bare-Int lane (metabolism.bsl's
;      entropy-factor-x1e6 D-1 precedent) — retires with #502 WS3's
;      Real x Ratio operator.
;   4. The sink walk is DIRECTED (`:out`, territory.py:174) while spillover
;      is `:any` (ADR179-T1's canonical-pair caveat, territory.py:279-284)
;      — the frozen asymmetry, transcribed.
;   5. Same-type multi-sink tiebreak: frozen enumeration-order last-wins
;      vs this language's D45 ascending-id first-wins.
;   6. Two-clamp inconsistency: p1 clamps [0,1] both sides
;      (system_base.py::_write_clamped), p3 clamps upper-only
;      (territory.py:315, `min(1.0, …)`) — transcribed faithfully.
;   7. No-defaults: every fixture seeds every field every rule reads; the
;      frozen `attrs.get(k, default)` affordance is not transcribable.
;   8. Hash-neutral no-op writes: p2's no-sink `(add 0)`, p3's isolated
;      unchanged `(set clamped)` where frozen skips the write entirely.
;   9. Summation/apply order vs the frozen engine's float op sequence
;      (the rent lane's scaled multiply-then-divide vs the frozen engine's
;      ONE `current_rent * rent_spike_multiplier` multiply; p3's pull-side
;      `rate x Σheat` vs the frozen per-edge `Σ(heat x rate)`) — measured
;      BSL expecteds are the oracle (ADR183), never chased to bit-match.
;  10. displacement_mode -> EXTRACTION const (provably uniform on every
;      production path, per the inventory's own finding, mechanism
;      corrected at D129 fix round: TickContext.get's own default is
;      never reached — _PRIORITY_BY_MODE.get's own fallback is what
;      actually delivers EXTRACTION); the override machinery,
;      defines.yaml:243 (displacement_priority_mode), and the four dead
;      AUTO-mode threshold defines (:244-247) go to the #502 WS1 ledger.
;      defines.yaml:241 (clarity_profile_coefficient) is UNRELATED (a
;      separate fog/clarity estate) and is not WS1-ledgered.
;
; `territory` is already a registered system (babylon-tick/src/lib.rs) —
; added earlier by the query-evaluation train as a namespace placeholder;
; this pack is the content that namespace was reserved for.
;
; CO-LOAD LANDMINE (final review I1, Decomposition+ControlRatio port
; train, 2026-08-17): `decomposition.bsl` ALSO declares this SAME
; byte-identical `(intrinsic floor …)`. The loader refuses a duplicate
; declaration BY NAME ONLY (`declarations.rs:1010-1017`, no content
; comparison), so co-loading `territory.bsl` with `decomposition.bsl` in
; one content set dies at load with `E-LOAD-001` — see
; `decomposition.bsl`'s own header for the full disclosure and the filed
; follow-up issue, **#646**. Territory @3.0 and Decomposition @11.0 are both
; Material Base systems, so this is a live Checkpoint A hazard, not a
; hypothetical one.

(intrinsic floor :params (real) :returns int :cost 5)

(rule territory/p1-heat-dynamics
  :role mechanic
  :evidence derived
  :material-basis "state legibility: HIGH_PROFILE visibility accumulates heat linearly, LOW_PROFILE opacity decays it geometrically (territory.py:107-137)"
  :fuel 128
  (bindings
    (binding profile :field territory/profile)
    (binding heat :field territory/heat)
    (binding gain :const territory/high-profile-heat-gain)
    (binding decay :const territory/heat-decay-rate)
    (binding raw :expr (if (= profile OperationalProfile/HIGH_PROFILE)
                           (+ heat gain)
                           (* heat (- 1 decay))))
    ; _write_clamped [0,1] (system_base.py:189) — nested-if idiom, floor then
    ; ceiling (D-3 class: dispossession.bsl:356-364's Real-zero-promotion
    ; trick — `(- 0 0c)`/`(- 1 0c)` are Real 0.0/1.0 so both `if` branches
    ; share one static type, §3.3/E-TYPE-020):
    (binding floored :expr (if (> raw 0) raw (- 0 0c)))
    (binding clamped :expr (if (< floored 1) floored (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/heat (set clamped))))

(rule territory/p2-eviction-pipeline
  :role mechanic
  :evidence derived
  :material-basis "rent as a weapon: crossing the legibility threshold latches eviction; each latched tick spikes rent and displaces population toward the carceral sinks (territory.py:196-267; EXTRACTION mode is provably uniform, WS1 ledger)"
  :fuel 512
  (bindings
    (binding heat :field territory/heat)
    (binding flag :field territory/under-eviction)
    (binding rent-x1e6 :field territory/rent-level-x1e6)
    (binding pop :field territory/population)
    (binding threshold :const territory/eviction-heat-threshold)
    (binding spike-x1e6 :const territory/rent-spike-multiplier-x1e6)
    (binding rate :const territory/displacement-rate)
    (binding displaced :expr (floor (* pop rate)))
    (binding rent-spiked :expr (/ (* rent-x1e6 spike-x1e6) 1000000)))
  (when (or (= flag 1) (>= heat threshold)))
  (effects
    (update-node self territory/under-eviction (set 1))
    (update-node self territory/rent-level-x1e6 (set rent-spiked))
    (update-node self territory/population (sub displaced))
    ; The sink-priority query is written out in full at each site (BSL has
    ; no local query naming, §2.6). The `exists` guard is SINK-TYPED — a
    ; three-way `or` over the frozen `_PRIORITY_BY_MODE[EXTRACTION]`
    ; membership test (territory.py:166-193), never a bare non-emptiness
    ; check — so a CORE/PERIPHERY-only neighbourhood correctly takes the
    ; fallback branch (frozen: population disappears), matching the
    ; `if territory_type in priority_order` membership test exactly. The
    ; score is the same three-way priority as an Int (D102's field-of
    ; discharge renders the enum; the SCORE itself stays Int, D46 stands).
    (update-node
      (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                  (if (= (field-of it territory/territory-type) TerritoryType/PENAL_COLONY) #t
                    (if (= (field-of it territory/territory-type) TerritoryType/RESERVATION) #t
                      (= (field-of it territory/territory-type) TerritoryType/CONCENTRATION_CAMP))))
          (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                      (if (= (field-of it territory/territory-type) TerritoryType/PENAL_COLONY) 3
                        (if (= (field-of it territory/territory-type) TerritoryType/RESERVATION) 2
                          (if (= (field-of it territory/territory-type) TerritoryType/CONCENTRATION_CAMP) 1 0))))
          self)
      territory/population
      (add (if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
                       (if (= (field-of it territory/territory-type) TerritoryType/PENAL_COLONY) #t
                         (if (= (field-of it territory/territory-type) TerritoryType/RESERVATION) #t
                           (= (field-of it territory/territory-type) TerritoryType/CONCENTRATION_CAMP))))
               displaced
               0)))))

(rule territory/p3-spillover
  :role mechanic
  :evidence derived
  :material-basis "heat is not contained by parcel lines: each ADJACENCY edge spills symmetrically, each endpoint receiving a fraction of the other's pre-phase heat (territory.py:269-316; pull-side fold reformulation is mathematically identical, ADR197/D103)"
  :fuel 512
  (bindings
    (binding heat :field territory/heat)
    (binding rate :const territory/heat-spillover-rate)
    ; DEVIATION from the plan's literal per-edge-product fold body
    ; `(* (field-of it territory/heat) rate)`: `rule_pipeline.rs::
    ; field_ref_for`'s own §3.4 fold-body reduction (the "Fold typecheck
    ; adapter" the module doc names) reduces exactly two shapes — a bare
    ; `field-of` accessor and a nested fold — never an arithmetic form; a
    ; compound body is rejected LOUDLY at load as unverifiable (the
    ; module's own recorded gap, confirmed reading rule_pipeline.rs, not
    ; assumed). The fold body here is therefore the bare accessor
    ; (matching query_lane_e2e.rs's own Shape A precedent exactly); the
    ; `* rate` scaling moves OUTSIDE the fold, applied once to the summed
    ; total. `Σ(heatᵢ) * rate` and `Σ(heatᵢ * rate)` are the SAME
    ; real-valued function computed by a DIFFERENT binary64 program — a
    ; D-9-class divergence, not a repair; the fold body itself STILL
    ; requires this binding to carry a graph-evaluating `:expr` (`exists`
    ; over the same query, then `fold`), which is exactly what P6's graph
    ; threading through `resolve_expr_bindings` (this task) exists to serve.
    (binding inflow :expr (if (exists (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY) #t)
                              (* (fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY)
                                       (field-of it territory/heat))
                                 rate)
                              (- 0 0c)))
    (binding raw :expr (+ heat inflow))
    ; frozen phase 3 clamps UPPER-ONLY (min(1.0, …), territory.py:315) — NOT
    ; the two-sided _write_clamped p1 uses; the two-clamp inconsistency is
    ; transcribed faithfully, D-recorded (#6):
    (binding clamped :expr (if (< raw 1) raw (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/heat (set clamped))))

; Byte order p4-camp-decay < p4-penal-suppression: the frozen engine
; interleaves both branches in one node loop (territory.py:335-378), but
; their write sets are disjoint (territory population vs class
; organization), so rule-order equivalence is exact here — noted, not a
; D-record. Camp decay runs against post-p2 population (this-tick
; displaced arrivals decay same tick, frozen-faithful via the sequential
; D116 reliance p1-p3 already established).

(rule territory/p4-camp-decay
  :role mechanic
  :evidence derived
  :material-basis "elimination: the camp's population decays every tick — the necropolitical endpoint (territory.py:344-347)"
  :fuel 64
  (bindings
    (binding ttype :field territory/territory-type)
    (binding pop :field territory/population)
    (binding decay :const territory/concentration-camp-decay-rate))
  (when (= ttype TerritoryType/CONCENTRATION_CAMP))
  (effects
    (update-node self territory/population (set (floor (* pop (- 1 decay)))))))

(rule territory/p4-penal-suppression
  :role mechanic
  :evidence derived
  :material-basis "atomization via incarceration: every class tenant of a penal colony has its organization zeroed (territory.py:349-378)"
  :fuel 128
  (bindings
    (binding ttype :field territory/territory-type))
  (when (= ttype TerritoryType/PENAL_COLONY))
  (effects
    (for-each (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
      (update-node it social-class/organization (set 0)))))
