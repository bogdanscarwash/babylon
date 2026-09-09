; ProductionSystem (Material Base @3.0) — "The Soil": workers produce value
; from labor x biocapacity, routed by the Amin/Wallerstein imperial-bribe
; model (RESERVED LINE, Constitution IX.5 — the role partition and the
; WAGES-edge employer lookup are the Director's ideological line,
; transcribed EXACTLY, never touched).
;
; Frozen source: src/babylon/engine/systems/production.py (268 lines, one
; step() over two loops). Port-as-is (Director ruling): frozen defects are
; transcribed and D-recorded, never silently repaired. The frozen engine is
; a structure/ordering contract, NOT a byte oracle (ADR183) — conformance
; expecteds are measured from THIS BSL engine and pinned in
; production_conformance.rs, not copied from the frozen mirror's printed
; floats.
;
; FIVE RULES, byte-ordered `p0-production-total-reset < p1-direct-production
; < p2-employed-routing < p3-employed-fallback < p4-extraction-intensity` —
; deliberately relying on D116's recorded cross-rule divergence
; (docs/reference/bsl-language.rst): today's run_once_into/TickSession::
; advance run each rule in a content set to COMPLETION before the next
; starts, against the SAME mutable graph, so a later rule at the same anchor
; position sees an EARLIER rule's already-applied writes from THIS tick. p4
; reads `territory/production-total`, which p0 resets and p1-p3 accumulate
; into EARLIER the same tick — this pack RELIES on that divergence rather
; than fighting it, the same way territory.bsl's own five-rule chain does
; (D120 is that pack's version of this same record; D132 is this pack's).
;
; The rules split this way because (a) subject type derives from `:field`
; namespaces (SOCIAL_CLASS for p1-p3, TERRITORY for p0/p4), and (b) p2's
; effect ref `(select-max (neighbors self EdgeType/WAGES :in …) 1)` ABORTS
; on an employer-less subject (E-EVAL-021 class), so employer existence must
; be split at the `when` level: p2 guards `(exists …)`, p3 guards `(not
; (exists …))` — `not` is served (grammar.rs:651), and `if` evaluates only
; the taken branch (§4.1, evaluator.rs:18), which is what makes the
; exists-guarded `field-of (select-max …)` bindings below legal. p1/p2/p3
; ALSO now guard `(exists (neighbors self EdgeType/TENANCY :out
; NodeType/TERRITORY))` in their own `when` — their THIRD effect's own
; `select-max` target ref lives in EFFECTS position, which aborts on an
; empty candidate set the same way p2's WAGES ref does (fix round,
; discharging D136 — see item 7 below).
;
; D-RECORDS this pack transcribes. Items 1, 3, 4, 5, 6, 7 have their own
; Task 5 register rows (D132, D137, D133, D134, D135, D136 respectively,
; docs/reference/bsl-language.rst — full text + file:line evidence there,
; MINOR-1 correction, fix round: an earlier draft of this sentence claimed
; ALL items had their own row, which was never true). Item 2 (the active
; int 0/1 latch) cites no register row at all — established convention,
; not a ruling. Item 8 (no-defaults) cites §1.5's own law directly, not a
; row. Item 9 (hash-neutral no-op writes) cites Territory's PRE-EXISTING
; D127, not a new row of its own. Item 10 mirrors D138 (also pre-existing,
; not minted for this pack):
;   1. Byte order relies on D116's cross-rule same-tick visibility — p4
;      reads p0/p1-p3's already-applied writes to `production-total`
;      (D132).
;   2. `social-class/active` is an int 0/1 latch — the established
;      `social-class/active`/`organization/active` CONVENTION (no bool
;      store path on the live `.bscn` pipeline), applied here by precedent
;      rather than by a dedicated register row of its own — no single row
;      exists naming "active fields are int 0/1" as a general law; each
;      consuming pack (vitality, organization, this one) just follows it.
;   3. `economy/base-labor-power-annual` at `1.0c` sits exactly at the
;      coefficient boundary [0,1] — legal today, fragile under modding (the
;      define's own domain is [0,∞), unbounded above) (D137).
;   4. The `fips_code`/`county_fips` dead tensor-registry branch
;      (production.py:160-172) is OMITTED entirely — provably unreachable,
;      nothing to transcribe (no BSL construct exists for an external
;      keyed-cache lookup in any case) (D133).
;   5. `social-class/production-value` is a per-node REFORMULATION of the
;      frozen `la_production` graph-scope dict (production.py:129,194) —
;      keyed by worker node id in the frozen engine, an ordinary node field
;      here. The write WIDENS to all three producer rules (p1/p2/p3), not
;      just the employed branch the frozen ledger covers; the read a future
;      ImperialRentSystem port would perform stays exactly as narrow as the
;      frozen `la_production.get(edge.target_id, ...)` call already is,
;      since only LA workers ever have an incoming WAGES edge (D134).
;   6. Tiebreak divergence: the frozen `_find_tenancy_target`/`_find_employer`
;      return the FIRST match in `query_edges` iteration (insertion) order;
;      this language's `select-max` (D46, constant score) tiebreaks by D45
;      ascending-id. `worker-pp-two-lands` (two TENANCY edges) pins the D45
;      winner (`t-alpha`, the lower NodeId) for its OWN bio-ratio
;      computation AND, since the fix round below, for which territory's
;      `production-total` it feeds (D135).
;   7. **Extraction-intensity attribution — RESOLVED (fix round): producer-
;      side PUSH, not territory-side pull.** An earlier draft of this pack
;      computed `territory/extraction-intensity` with a territory-side
;      `fold sum` pulling `social-class/production-value` off every
;      TENANCY-incident neighbour — which double-counted `worker-pp-two-
;      lands` (its two TENANCY edges) into BOTH `t-alpha`'s and `t-beta`'s
;      totals, diverging from the frozen engine's single-territory
;      `_find_tenancy_target` attribution. That draft's register row
;      (D136) additionally claimed no `.bsl`-level fix was available within
;      a port-as-is mandate — FALSE, caught by adversarial verification: a
;      producer-side push (this pack, now) matches the frozen engine
;      EXACTLY. `territory/production-total` (`int extensive`, seeded `0`
;      on every territory) replaces the pull fold; `production/
;      p0-production-total-reset` zeroes it every tick (byte-ordered
;      first); `p1`/`p2`/`p3` each gain a THIRD effect writing to it via the
;      SAME tiebreak-selected ref (D135) their `bio`/`max-bio` bindings
;      already compute — so a multi-tenancy producer's contribution lands
;      on EXACTLY ONE territory, matching the frozen engine's single-
;      territory attribution up to D135's own (here, non-discriminating)
;      tiebreak. `p4`'s `total` binding is now a plain `:field` read — no
;      fold, no filter — item 10's own D138 mirror is now HISTORY, not this
;      pack's live design: it explains why `production-value` (item 5) was
;      minted as a per-node field in the first place, not what `p4` does
;      today. **The fix round's own semantic cost, honestly recorded:**
;      p1/p2/p3's new `(exists (neighbors self EdgeType/TENANCY :out
;      NodeType/TERRITORY))` `when` conjunct — needed because the new
;      third effect's own `select-max` target ref lives in EFFECTS
;      position and aborts on an empty candidate set — changes tenancy-
;      less-producer semantics: such a producer used to FIRE with `output`
;      forced to `0` by the `active`-gate idiom (writing `production-value`
;      to `0` every tick, hash-neutral); it now does NOT FIRE AT ALL, so
;      its `production-value` goes STALE (holds whatever the previous tick
;      left, or its seed) rather than resetting every tick. No fixture node
;      exercises this today — every seeded producer role carries a TENANCY
;      edge — but a future fixture with a tenancy-less producer would need
;      to choose which reading it wants. Full corrected account: D136
;      (register).
;   8. No-defaults: every fixture seeds every field every rule reads — a
;      direct application of §1.5's own law, not a new ruling; no dedicated
;      register row.
;   9. Hash-neutral no-op writes: an inactive producer (worker-la-idle)
;      still fires p2 (its `when` guard is role+employer-existence+tenancy-
;      existence, not `active`), writing `(add 0)`/`(set 0)` — the D127
;      idiom (Territory's own pre-existing row for the same class; no new
;      row minted for the identical idiom here) — where the frozen engine's
;      own `continue` at the top of the loop skips the iteration (and
;      therefore any write) entirely.
;  10. **The fold-body compound-expression restriction (D138) — historical
;      design rationale, mirrored here two-homes style.** `rule_pipeline.rs::
;      field_ref_for` (§3.4) reduces a fold body to a declared field's kind
;      through exactly three shapes (a bare `<qname>`, a `field-of`
;      accessor, or a nested fold) and refuses anything else, including an
;      `if`-based role filter — which is WHY a naive territory-side fold
;      over `(neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)`
;      reading a neighbour's `role`/`population` directly could never load:
;      the flagship scenario's own TENANCY topology is not role-restricted
;      (`comprador`, a COMPRADOR_BOURGEOISIE tenant, proves this), so the
;      filter has to live somewhere, and a fold body cannot hold it. This
;      restriction is what made `social-class/production-value` (item 5)
;      — the per-node, already-filtered field p1-p3 compute via their OWN
;      `when` guards — the right shape REGARDLESS of which side (pull or
;      push) ultimately reads it. The fix round's push redesign (item 7)
;      does not need this restriction at all — `p4` reads `production-
;      total` via a plain `:field` binding, no fold anywhere in this pack
;      — but the restriction is still what explains why `production-value`
;      exists as a filtered per-node field rather than a raw
;      `population`/`role` read.
;
; "production" is a genuinely NEW registered system (babylon-tick/src/lib.rs)
; — unlike Territory, which inherited a pre-existing placeholder from the
; query-evaluation train.

(rule production/p0-production-total-reset
  :role mechanic
  :evidence derived
  :material-basis "Territory-side accumulator reset for the producer-side PUSH attribution (fix round, discharging D136): production-total must be zeroed before p1-p3 add to it this tick, the same zero-then-accumulate shape a carrier field needs whenever multiple subjects contribute to it in one tick (D103/D104's own collect-then-apply proof is what makes the add-after-reset ordering safe within one tick)."
  :fuel 32
  (bindings
    (binding current :field territory/production-total))
  (when #t)
  (effects
    (update-node self territory/production-total (set (- 0 0c)))))

(rule production/p1-direct-production
  :role mechanic
  :evidence derived
  :material-basis "Fundamental Theorem plumbing: the periphery proletariat produces value with its own labor-power on land it occupies (produced = weekly labor-power x population x biocapacity ratio, production.py:151-175) and, as the direct producer with no wage relation, keeps its own product (production.py:179-181). Fix round: also pushes its product onto the tiebreak-selected territory's production-total (production.py:200-204), discharging D136."
  :fuel 224
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding annual :const economy/base-labor-power-annual)
    (binding weeks :const timescale/weeks-per-year)
    (binding bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                           (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                     territory/biocapacity)
                           (- 0 0c)))
    (binding max-bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                               (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                         territory/max-biocapacity)
                               (- 0 0c)))
    (binding bio-ratio :expr (if (> max-bio 0) (/ bio max-bio) (- 0 0c)))
    (binding produced :expr (* (* (/ annual weeks) population) bio-ratio))
    (binding output :expr (if (= active 1) produced (- 0 0c))))
  (when (and (= role SocialRole/PERIPHERY_PROLETARIAT)
             (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))))
  (effects
    (update-node self social-class/wealth (add output))
    (update-node self social-class/production-value (set output))
    (update-node (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                 territory/production-total
                 (add output))))

(rule production/p2-employed-routing
  :role mechanic
  :evidence derived
  :material-basis "Amin/Wallerstein: the labor aristocracy's product is appropriated by the employing bourgeoisie through the WAGES relation (production.py:184-194). RESERVED LINE -- the routing structure is the Director's ideological line, transcribed exactly. Fix round: also pushes its product onto the tiebreak-selected territory's production-total (production.py:200-204), discharging D136."
  :fuel 256
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding annual :const economy/base-labor-power-annual)
    (binding weeks :const timescale/weeks-per-year)
    (binding bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                           (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                     territory/biocapacity)
                           (- 0 0c)))
    (binding max-bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                               (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                         territory/max-biocapacity)
                               (- 0 0c)))
    (binding bio-ratio :expr (if (> max-bio 0) (/ bio max-bio) (- 0 0c)))
    (binding produced :expr (* (* (/ annual weeks) population) bio-ratio))
    (binding output :expr (if (= active 1) produced (- 0 0c))))
  (when (and (= role SocialRole/LABOR_ARISTOCRACY)
             (exists (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS))
             (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))))
  (effects
    (update-node (select-max (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS) 1)
                 social-class/wealth
                 (add output))
    (update-node self social-class/production-value (set output))
    (update-node (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                 territory/production-total
                 (add output))))

(rule production/p3-employed-fallback
  :role mechanic
  :evidence derived
  :material-basis "The frozen fallback: an employed-role producer with no employer keeps its own product (production.py:196-198). Fix round: also pushes its product onto the tiebreak-selected territory's production-total (production.py:200-204), discharging D136."
  :fuel 224
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding annual :const economy/base-labor-power-annual)
    (binding weeks :const timescale/weeks-per-year)
    (binding bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                           (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                     territory/biocapacity)
                           (- 0 0c)))
    (binding max-bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                               (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                         territory/max-biocapacity)
                               (- 0 0c)))
    (binding bio-ratio :expr (if (> max-bio 0) (/ bio max-bio) (- 0 0c)))
    (binding produced :expr (* (* (/ annual weeks) population) bio-ratio))
    (binding output :expr (if (= active 1) produced (- 0 0c))))
  (when (and (= role SocialRole/LABOR_ARISTOCRACY)
             (not (exists (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS)))
             (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))))
  (effects
    (update-node self social-class/wealth (add output))
    (update-node self social-class/production-value (set output))
    (update-node (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                 territory/production-total
                 (add output))))

(rule production/p4-extraction-intensity
  :role mechanic
  :evidence derived
  :material-basis "Metabolic coupling: extraction intensity = produced value against the territory's max biocapacity, clamped to [0,1] (production.py:246-268). Fix round: total is now a plain :field read of the producer-side-PUSHED production-total (p0 resets it, p1-p3 accumulate into it) -- matches the frozen engine's single-territory attribution exactly, discharging D136. Reads production-total written by p0-p3 THIS TICK -- the pack relies on D116 byte-order cross-rule visibility (see pack D-1)."
  :fuel 64
  (bindings
    (binding max-bio :field territory/max-biocapacity)
    (binding total :field territory/production-total)
    (binding ratio :expr (if (> max-bio 0) (/ total max-bio) (- 0 0c)))
    (binding clamped :expr (if (< ratio 1) ratio (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/extraction-intensity (set clamped))))
