; community.bsl — CommunitySystem (Material Base @6.0, "Community
; hypergraph: n-ary membership, consciousness from the org landscape,
; reproduction cost modification, state decay"). Frozen source:
; `src/babylon/engine/systems/community.py` (`CommunitySystem`, class at
; :309-323, `step` at :325-370 — 675 lines), with the ternary math at
; `src/babylon/formulas/consciousness.py:55-108` and the cost modifier at
; `src/babylon/formulas/community.py:150-175`. Issue #667, Tasks 8-11 —
; docs/superpowers/plans/2026-08-18-community-port.md.
;
; NAME COLLISION, said first (plan §2.3): `lifecycle.bsl:4,78,104,372`
; mentions "community" and a field `community_tendency` — an INBOUND
; routing weight for the lifecycle/consciousness seeding law, unrelated to
; anything CommunitySystem reads or writes. Nothing here touches it.
;
; §3.7a CARRIER DISCLOSURE: this pack mints NO carrier node. Its
; carrier-subject rules (c00 here; c05-c08/c11 in later tasks) anchor on
; the ONE `NodeType/INSTITUTION` node a world already has, via
; `institution/community-carrier` — a SUBJECT-TYPE ANCHOR ONLY, bound and
; never read again, never gating anything (the binding exists because
; tick.rs::subject_type_of requires >=1 :field binding to derive
; INSTITUTION; this sentence exists so no reader mistakes it for a gate —
; control-ratio.bsl's pattern, verbatim in intent). A second
; INSTITUTION node in any world loading this pack double-applies every
; hyperedge write (each carrier-subject rule iterates EVERY institution);
; §8c guard 4 (`exactly_one_institution_carrier`,
; tests/community_conformance.rs) is the executable bar.
;
; BYTE-ORDER MAP (plan §2; execution order is rule-id byte order, D16):
; c00-census-reset < c01-member-census < c02-org-weight-reset <
; c03f-… < c03l-… < c03r-… < c04-community-contribution-push. The three
; c03 rules run f-then-l-then-r (NOT the plan table's reading order); the
; order is UNOBSERVABLE because the three write disjoint weight fields and
; their one shared field (`org-count`) accumulates integer-exact `add 1`s —
; the mirror transcribes the true order and its output is byte-identical
; either way (proven at Task 7). §8b's D116 ledger (the same-tick
; cross-rule reads this pack relies on) is reproduced in the plan, §8b.
; Task 9 extends the map: … < c05-normalize < c06a-floor-dispatch <
; c06b-floor-redistribute. THE c06 SPLIT (D204): the plan's c06 was ONE
; rule — "the pack's ONLY 14-arm dispatch" driving the redistribution —
; but the pre-state law (§4.2 C4) makes a write-then-read WITHIN one
; rule's body impossible: the floor written by an `update-hyperedge` is
; not visible to a later operand's `field-of` in the same collect pass.
; The dispatch therefore lands as c06a (writes the per-community
; `community/substrate-floor` cache through the pack's only 14-arm chain)
; and c06b (reads the cache — a same-tick CROSS-RULE read, the §8b shape
; spike (f) proved at Task 7). The math is unchanged; the pack's rule
; count moves 14 -> 15 (DG-2's "12 if no-publish" becomes 13), recorded
; in the D-row register. §8a gains a row: the c05 total/unorganized
; expressions are inlined per write (no defexpr exists) — the bit-exact
; mirror assertions are the copies-agree mechanism.
;
; §5 DISCLOSURE — what does NOT land (the #653-gated half, hard-sequenced):
; threat scoring (community.py:579-608), solidarity amplification
; (:527-576), and the infrastructure line WHOLE (its maintenance term is
; non-monotone; a monotone-only port would be a different law, and
; port-as-is (ADR183) forbids it) all wait on the AG(i)
; attributed-membership ceremony — a Director act, never improvised. The
; four repression helpers (:210-279) await a verb layer (D-NF+11). c07/c08
; are DG-2-gated (Director question, unresolved at authoring).
;
; THE EXPLICIT-DOMAIN NOTE (PR #688 review, Copilot finding 1 — a REAL
; latent engine gap, named here for the future pack reader): c00's carrier
; binding is never referenced (its guard is `#t`), which makes the rule
; E-LOAD-004-undeterminable at LOAD — domain.rs::resolve_domain's
; None-branch candidate set is REFERENCE-fed, so an anchor that is bound
; but never read contributes nothing there, while tick.rs's
; subject_type_of counts every :field binding, referenced or not. The two
; derivations can therefore disagree and NOTHING cross-checks them
; (run_tick ignores the loaded domain entirely) — control-ratio.bsl's
; (when #t) precedent passes only because its :expr fold bodies feed the
; candidate set. This pack declares `(domain NodeType/INSTITUTION)`
; explicitly on every carrier rule whose guard is vacuous, which keeps
; both derivations honest by construction. Filed as a follow-up, not
; silently absorbed.
;
; THE FLOOR TABLE: the 14 ADR214 defconst rows live in each WORLD's .bscn
; (the scenario's defconst registry IS the driver's defines env — §6.2's
; "re-declared per scenario" precedent), never in this pack; the
; cross-world parity test (Task 9) pins every world's rows equal to the
; ADR's values.
;
; THE QUANTIZATION DIVERGENCE (found by the Task 7 corroboration artifact,
; reports/community-frozen-corroboration-2026-08-18.md): frozen's
; TernaryConsciousness fields are Probability-typed, snapped to a 10^-6
; ROUND_HALF_UP grid at the Pydantic boundary (kernel/math.py:41,
; _PRECISION=6). This pack computes and stores the UNQUANTIZED f64 chain —
; the mirror is the oracle, frozen's stored values are its grid-snapped
; twins. Where the pack's Tasks-9+ assertions differ from frozen's stored
; prints, this is why; it is recorded here and in the D-row register at
; this pack's landing, not discovered downstream.
;
; LATENT REFUSAL, NAMED (not handled — the plan gates nothing here): c04
; divides by `community/member-count`, the ACTIVE census. A community whose
; members are ALL inactive has count 0, and its members' c04 pushes refuse
; loudly (non-finite store, E-EVAL-014's discipline) rather than write a
; lie. World 1 has no such community (every seeded community has >=1 active
; member); the first world that seeds one owes the gate decision its own
; task's D-row.
;
; Reserved D-N rows this task consumes: D-NF+3 (the per-class org-weight
; decomposition — frozen's per-org weight sum re-expressed as per-class
; accumulators divided by the census, bit-exact in world 1's dyadic
; values), D-NF+25 (frozen's tendency-less org skip (:405-407) is
; inexpressible — the tendency gate is a rule-level `when`, three rules
; PARTITION every org, because fold bodies are bare accessors only (D138)).
; The pack's landed register rows are D203+ in docs/reference/
; bsl-language.rst.

; The pack's one intrinsic declaration (DG-2 = PUBLISH, the 2026-08-18
; sitting): `log` for c07's entropy divisor. First content pack to declare
; it, so this sets its :cost (the spec's ":cost provenance" paragraph —
; the first declarer sets the number, pinned by the conformance vector
; thereafter); 40 follows the crate's own declaration test
; (declarations.rs:1345).
(intrinsic log :params (real) :returns real :cost 40)

; ============================================================ c00 — the reset
(rule community/c00-census-reset
  :role mechanic
  :evidence derived
  :material-basis "The per-tick rebuild (community.py:392-397 mints a fresh community_agents map on every step, and community.py:460-462 writes back via model_copy): every accumulator this pack reads derives from THIS tick's writes, so the tick begins by zeroing them. The institution/community-carrier binding is a SUBJECT-TYPE ANCHOR ONLY (tick.rs::subject_type_of requires >=1 :field binding to derive INSTITUTION) — never read again, never gating anything (`when #t`), so the domain is declared EXPLICITLY: an unreferenced anchor plus a vacuous guard is E-LOAD-004-undeterminable at load (domain.rs's candidate set is reference-fed), and control-ratio.bsl's precedent passes only because its fold bodies feed that set — recorded in the pack header."
  :fuel 104
  (bindings
    (binding carrier :field institution/community-carrier))
  (domain NodeType/INSTITUTION)
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-hyperedge it community/member-count (set 0))
      (update-hyperedge it community/r-raw (set 0))
      (update-hyperedge it community/l-raw (set 0))
      (update-hyperedge it community/f-raw (set 0))
      (update-hyperedge it community/density-sum (set 0)))))

; ============================================================ c01 — the census
(rule community/c01-member-census
  :role mechanic
  :evidence derived
  :material-basis "The member census (community.py:465-479's _collect_memberships + :392-397's community_agents): ACTIVE classes only (the :472-474 gate — an inactive member is excluded from the count AND from every downstream write), one +1 per (class, community) membership. The adds collect against the pre-state and combine at apply, so N active members land N exactly."
  :fuel 27
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges-of self HyperedgeType/COMMUNITY)
      (update-hyperedge it community/member-count (add 1)))))

; ============================================================ c02 — the per-class accumulator reset
(rule community/c02-org-weight-reset
  :role mechanic
  :evidence derived
  :material-basis "Port scaffolding (D-NF+3): the per-class org accumulators frozen never needed (it sums per-org in one pass) are this pack's decomposition of the same sum — reset per tick, all four, EVERY social class (no active gate: the reset is not the census)."
  :fuel 21
  (bindings
    (binding active :field social-class/active))
  (when #t)
  (effects
    (update-node self social-class/org-r-weight (set 0))
    (update-node self social-class/org-l-weight (set 0))
    (update-node self social-class/org-f-weight (set 0))
    (update-node self social-class/org-count (set 0))))

; ============================================ c03f/c03l/c03r — the org pushes
; The tendency gate is a rule-level `when`, NOT a fold body (D138: fold
; bodies are bare accessors), so the partition is three rules — D-NF+25.
; Byte order runs f, l, r; the order is unobservable (disjoint weight
; fields, integer-exact shared count) — see the header's byte-order map.
(rule community/c03f-org-weight-push
  :role mechanic
  :evidence derived
  :material-basis "The FASCIST arm of the org-weight push (community.py:403-426 + formulas/consciousness.py:63-72): each FASCIST org pushes cadre x cohesion onto each member class's org-f-weight and +1 onto its org-count. The org iterates only ITS OWN outbound MEMBERSHIP edges (the D136 push idiom, solidarity.bsl's mandatory form) — every edge visited exactly once, by its unique source. Frozen's tendency-less skip (community.py:405-407) is inexpressible as a fold guard, so the three tendency rules partition every org (D-NF+25); a tendency-less org is never written by frozen either, and no world may seed one."
  :fuel 72
  (bindings
    (binding cadre :field organization/cadre-level)
    (binding cohesion :field organization/cohesion)
    (binding tendency :field organization/consciousness-tendency))
  (when (= tendency ConsciousnessTendency/FASCIST))
  (effects
    (for-each (neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/org-f-weight (add (* cadre cohesion)))
      (update-node it social-class/org-count (add 1)))))

(rule community/c03l-org-weight-push
  :role mechanic
  :evidence derived
  :material-basis "The LIBERAL arm — same law as c03f (community.py:403-426), one tendency over."
  :fuel 72
  (bindings
    (binding cadre :field organization/cadre-level)
    (binding cohesion :field organization/cohesion)
    (binding tendency :field organization/consciousness-tendency))
  (when (= tendency ConsciousnessTendency/LIBERAL))
  (effects
    (for-each (neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/org-l-weight (add (* cadre cohesion)))
      (update-node it social-class/org-count (add 1)))))

(rule community/c03r-org-weight-push
  :role mechanic
  :evidence derived
  :material-basis "The REVOLUTIONARY arm — same law as c03f (community.py:403-426), one tendency over."
  :fuel 72
  (bindings
    (binding cadre :field organization/cadre-level)
    (binding cohesion :field organization/cohesion)
    (binding tendency :field organization/consciousness-tendency))
  (when (= tendency ConsciousnessTendency/REVOLUTIONARY))
  (effects
    (for-each (neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/org-r-weight (add (* cadre cohesion)))
      (update-node it social-class/org-count (add 1)))))

; ============================================================ c04 — the contribution push
(rule community/c04-community-contribution-push
  :role mechanic
  :evidence derived
  :material-basis "The density decomposition (plan §1.3, D-NF+3): frozen's per-org weight (overlap/comm_size) x cadre x cohesion re-expressed as per-class sums divided by the census count — each active-in-census class pushes org-weight/member-count onto its communities' raw ternary accumulators and org-count/member-count onto density-sum. The divisor is c01's SAME-TICK census (§8b's D116 ledger row 1: fatal if apply-in-place is ever repaired to collect-across-rules — the Q14 train's acceptance input). The `active` gate is FIDELITY, not caution: frozen's community_agents is built from the active-only membership set (community.py:472-474 -> :392-397), so an inactive class's org weights (c03 pushes to members regardless of the target's active flag) must NEVER enter the sum — gated here, c03's push onto an inactive class stays inert (c02 resets it next tick)."
  :fuel 151
  (bindings
    (binding active :field social-class/active)
    (binding rw :field social-class/org-r-weight)
    (binding lw :field social-class/org-l-weight)
    (binding fw :field social-class/org-f-weight)
    (binding orgs :field social-class/org-count))
  (when (= active 1))
  (effects
    (for-each (hyperedges-of self HyperedgeType/COMMUNITY)
      (update-hyperedge it community/r-raw
        (add (/ rw (field-of it community/member-count))))
      (update-hyperedge it community/l-raw
        (add (/ lw (field-of it community/member-count))))
      (update-hyperedge it community/f-raw
        (add (/ fw (field-of it community/member-count))))
      (update-hyperedge it community/density-sum
        (add (/ orgs (field-of it community/member-count)))))))

; ============================================================ c05 — the normalization
; (Task 9 Step 2; the §6.2 I5 routing: the degenerate branch emits
; (0, 1, 0) and NOTHING else — the floor routes through c06a/c06b,
; bit-identically to frozen's fused branch, formulas/consciousness.py:89-91,
; because x1.0 and /1.0 are exact in IEEE-754.)
(rule community/c05-normalize
  :role mechanic
  :evidence derived
  :material-basis "Normalize the raw accumulators to the simplex (formulas/consciousness.py:78-95): unorganized = max(0, 1 - density-sum) folds into l (Jackson: passive acceptance is liberal hegemony); total = (r + (l+u)) + f, frozen's exact association; total < 1e-10 is the degenerate branch -> (0, 1, 0) and NOTHING else (the floor routes through c06a/c06b, bit-identically — 6.2 I5). The density-sum > 0 guard is frozen's community.py:452 skip gate (no org_landscape -> keep the prior ternary). Frozen never stores the unorganized-folded l-raw, so neither does this rule — only the normalized ternary is written. The total/unorganized expressions are inlined per write (no defexpr exists — 8a ledger); the epsilon is the landed (/ 1c 10000000000) quotient (consciousness.bsl's idiom), bit-identical to frozen's 1e-10 literal."
  :fuel 596
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (> (field-of it community/density-sum) 0.0c)
        (update-hyperedge it community/revolutionary
          (set
            (if (< (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw)) (/ 1.0c 10000000000))
              0.0p
              (/ (field-of it community/r-raw) (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw))))))
        (update-hyperedge it community/liberal
          (set
            (if (< (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw)) (/ 1.0c 10000000000))
              1.0p
              (/ (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum)))) (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw))))))
        (update-hyperedge it community/fascist
          (set
            (if (< (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw)) (/ 1.0c 10000000000))
              0.0p
              (/ (field-of it community/f-raw) (+ (+ (field-of it community/r-raw) (+ (field-of it community/l-raw) (if (< (- 1.0c (field-of it community/density-sum)) 0.0c) 0.0c (- 1.0c (field-of it community/density-sum))))) (field-of it community/f-raw))))))))))

; ============================================ c06a — the floor dispatch (Task 9 Step 3)
; THE PACK'S ONLY 14-arm community/kind dispatch (6.2 — no map or lookup
; construct exists). Writes the per-community floor to the
; community/substrate-floor cache; c06b reads it the SAME TICK (the
; pre-state law forces the split — a write inside one rule's collect pass
; is invisible to a later operand's field-of, so dispatch-and-redistribute
; cannot be one rule; D204).
(rule community/c06a-floor-dispatch
  :role mechanic
  :evidence derived
  :material-basis "The 14-row ADR214 floor table, dispatched on community/kind (6.2; the pack's ONLY 14-arm chain — spike shape (e) proved it at Task 7). Values and per-row provenance: the world's .bscn defconst block (ADR214 Ruling 1 + erratum 9 for the measured three; Ruling 2's demotions for the LOW five; INCARCERATED's named unreachability; SETTLER/PATRIARCHAL/YOUTH/ADULT structural zeros; ELDER estimated). The density-sum > 0 guard is the SAME skip gate as c05's — a skipped community's cache is never written, and c06b's guard never reads it (the honest-null discipline, 9 item 5)."
  :fuel 252
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier)
    (binding floor-settler :const community/floor-settler)
    (binding floor-patriarchal :const community/floor-patriarchal)
    (binding floor-new-afrikan :const community/floor-new-afrikan)
    (binding floor-first-nations :const community/floor-first-nations)
    (binding floor-chicano :const community/floor-chicano)
    (binding floor-women :const community/floor-women)
    (binding floor-trans :const community/floor-trans)
    (binding floor-disabled :const community/floor-disabled)
    (binding floor-queer :const community/floor-queer)
    (binding floor-undocumented :const community/floor-undocumented)
    (binding floor-incarcerated :const community/floor-incarcerated)
    (binding floor-youth :const community/floor-youth)
    (binding floor-adult :const community/floor-adult)
    (binding floor-elder :const community/floor-elder))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (> (field-of it community/density-sum) 0.0c)
        (update-hyperedge it community/substrate-floor
          (set
            (if (= (field-of it community/kind) CommunityType/SETTLER) floor-settler
            (if (= (field-of it community/kind) CommunityType/PATRIARCHAL) floor-patriarchal
            (if (= (field-of it community/kind) CommunityType/NEW_AFRIKAN) floor-new-afrikan
            (if (= (field-of it community/kind) CommunityType/FIRST_NATIONS) floor-first-nations
            (if (= (field-of it community/kind) CommunityType/CHICANO) floor-chicano
            (if (= (field-of it community/kind) CommunityType/WOMEN) floor-women
            (if (= (field-of it community/kind) CommunityType/TRANS) floor-trans
            (if (= (field-of it community/kind) CommunityType/DISABLED) floor-disabled
            (if (= (field-of it community/kind) CommunityType/QUEER) floor-queer
            (if (= (field-of it community/kind) CommunityType/UNDOCUMENTED) floor-undocumented
            (if (= (field-of it community/kind) CommunityType/INCARCERATED) floor-incarcerated
            (if (= (field-of it community/kind) CommunityType/YOUTH) floor-youth
            (if (= (field-of it community/kind) CommunityType/ADULT) floor-adult
              floor-elder)))))))))))))))
      ))))

; ============================================================ c06b — the redistribution (Task 9 Step 3)
(rule community/c06b-floor-redistribute
  :role mechanic
  :evidence derived
  :material-basis "The substrate floor, applied post-normalization (formulas/consciousness.py:98-107): if r < floor, r = floor and the remaining (1 - floor) redistributes to l and f PROPORTIONALLY (l x remaining / lf, f x remaining / lf) when lf > 1e-10, else l = remaining and f = 0 (the two-arm split). Reads c06a's cache the SAME TICK (8b's ledger; spike (f) proved the shape). The SETTLER control: floor 0.0 makes r < floor identically false — the 0.0-floor community is untouched (6.2)."
  :fuel 272
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (> (field-of it community/density-sum) 0.0c)
        (guard (< (field-of it community/revolutionary) (field-of it community/substrate-floor))
          (update-hyperedge it community/liberal
            (set
              (if (> (+ (field-of it community/liberal) (field-of it community/fascist)) (/ 1.0c 10000000000))
                (/ (* (field-of it community/liberal) (- 1.0c (field-of it community/substrate-floor))) (+ (field-of it community/liberal) (field-of it community/fascist)))
                (- 1.0c (field-of it community/substrate-floor)))))
          (update-hyperedge it community/fascist
            (set
              (if (> (+ (field-of it community/liberal) (field-of it community/fascist)) (/ 1.0c 10000000000))
                (/ (* (field-of it community/fascist) (- 1.0c (field-of it community/substrate-floor))) (+ (field-of it community/liberal) (field-of it community/fascist)))
                0.0p)))
          (update-hyperedge it community/revolutionary
            (set (field-of it community/substrate-floor))))))))

; ============================================ c09/c10 — the cost modifier (Task 10 Step 2)
(rule community/c09-cost-modifier-reset
  :role mechanic
  :evidence derived
  :material-basis "formulas/community.py:164-165's per-tick reset to 1, gated ACTIVE-ONLY by community.py:472-474 (1.4/C4): an INACTIVE class is written NOTHING — its social-class/community-cost-modifier reads the substrate's honest-null error, never 1.0. The reset is what makes c10's product per-tick rather than compounding across ticks (the Task-8-deferred vector, proven in Task 10's arc world)."
  :fuel 8
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (update-node self social-class/community-cost-modifier (set 1))))

(rule community/c10-cost-modifier-accumulate
  :role mechanic
  :evidence derived
  :material-basis "formulas/community.py:166-174: the modifier is the PRODUCT of the member communities' reproduction-cost-modifier — one `scale` per membership, ascending HyperedgeId (D25; D-NF+13 records the float-product-order divergence: frozen iterates a dict, this pack's order is the substrate's). The active guard is community.py:472-474's (C4). Spike shape (d) verdict, cited: `scale` accumulates multiplicatively across a for-each because the combine reads the current value at APPLY (the operand is pre-computed per collected write)."
  :fuel 35
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (hyperedges-of self HyperedgeType/COMMUNITY)
      (update-node self social-class/community-cost-modifier
        (scale (field-of it community/reproduction-cost-modifier))))))

; ============================================================ c11 — the state decay (Task 10 Step 3)
(rule community/c11-state-decay
  :role mechanic
  :evidence derived
  :material-basis "community.py:648-675: per tick per community, heat/cohesion/education-pressure each decay max(0, x·(1−α)) (the three-arm triplication is 8a row 2 — decay_arms_are_independent carries the three mutation vectors). Frozen computes x·(1−α) then clamps at the model_copy (:668-674); the if-form here is max(0, ·) exactly. THE INFRASTRUCTURE ARM IS ABSENT — 5: its CORE_ORGANIZER maintenance term makes frozen's law NON-MONOTONE, and landing x·(1−α) alone would be a DIFFERENT law (monotone decay); port-as-is (ADR183) forbids it. The term waits on #653 (the AG(i) ceremony), DG-7's call. The decay has NO skip gate — frozen decays every community_state (:648's loop over the whole dict), including a no-org-skipped one."
  :fuel 208
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier)
    (binding heat-alpha :const community/heat-decay-alpha)
    (binding cohesion-alpha :const community/cohesion-decay-alpha)
    (binding edu-alpha :const community/education-pressure-decay))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (update-hyperedge it community/heat
        (set (if (< (* (field-of it community/heat) (- 1.0c heat-alpha)) 0.0c) 0.0c (* (field-of it community/heat) (- 1.0c heat-alpha)))))
      (update-hyperedge it community/cohesion
        (set (if (< (* (field-of it community/cohesion) (- 1.0c cohesion-alpha)) 0.0c) 0.0c (* (field-of it community/cohesion) (- 1.0c cohesion-alpha)))))
      (update-hyperedge it community/education-pressure
        (set (if (< (* (field-of it community/education-pressure) (- 1.0c edu-alpha)) 0.0c) 0.0c (* (field-of it community/education-pressure) (- 1.0c edu-alpha))))))))

; ============================================================ c07 — the contestation readout (DG-2: PUBLISH)
; DG-2 was answered at the 2026-08-18 sitting: PUBLISH contestation +
; dominant-tendency (port fidelity — frozen publishes both,
; community.py:106-107 — plus the events-as-affordances thesis, #502).
(rule community/c07-contestation
  :role mechanic
  :evidence derived
  :material-basis "The normalized Shannon entropy of the published simplex (entities/consciousness.py:274-291): entropy = 0 - t(r) - t(l) - t(f) with t(p) = p*log(p) guarded per component at p > 1e-10 (the (/ 1c 10000000000) quotient), divided by (log (* 1.0c 3)) — the divisor COMPUTED through the pack's declared `log` intrinsic (the pinned libm soft-float crossing), never pasted. DG-2's PUBLISH ruling is why this rule exists. NOTE the divisor divergence, recorded (D-row): libm::log(3) is 1.0986122886681096, ONE ulp below CPython's math.log(3) (1.0986122886681098) — the engine's pinned soft-float crossing is the law of THIS port (ADR176 r21 / ADR188), so the mirror divides by the libm value and the conformance pins are bit-exact against IT."
  :fuel 900
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (> (field-of it community/density-sum) 0.0c)
        (update-hyperedge it community/contestation
          (set
            (/ (- (- (- 0.0c
              (if (> (field-of it community/revolutionary) (/ 1.0c 10000000000)) (* (field-of it community/revolutionary) (log (field-of it community/revolutionary))) 0.0c))
              (if (> (field-of it community/liberal) (/ 1.0c 10000000000)) (* (field-of it community/liberal) (log (field-of it community/liberal))) 0.0c))
              (if (> (field-of it community/fascist) (/ 1.0c 10000000000)) (* (field-of it community/fascist) (log (field-of it community/fascist))) 0.0c))
              (log (* 1.0c 3)))))))))

; ============================================ c08 — the dominant-tendency readout (DG-2: PUBLISH)
(rule community/c08-dominant-tendency
  :role mechanic
  :evidence derived
  :material-basis "The argmax readout with the ruled tie order LIBERAL > REVOLUTIONARY > FASCIST at 1e-6 (entities/consciousness.py:167-191, epsilon :189) — the SAME shape consciousness/p8-dominant-worldview runs over the class surface (the 8a row-1 copies-agree test pins both to LIBERAL on the same seeded three-way tie). The skip gate is c05's (density-sum > 0): a no-org community keeps its prior tendency, never a fabricated readout."
  :fuel 336
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/community-carrier))
  (when #t)
  (effects
    (for-each (hyperedges HyperedgeType/COMMUNITY)
      (guard (> (field-of it community/density-sum) 0.0c)
        (update-hyperedge it community/dominant-tendency
          (set
            (if (< (if (> (field-of it community/liberal) (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist)))) (- (field-of it community/liberal) (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist)))) (- (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist))) (field-of it community/liberal))) 0.000001c)
              ConsciousnessTendency/LIBERAL
              (if (< (if (> (field-of it community/revolutionary) (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist)))) (- (field-of it community/revolutionary) (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist)))) (- (if (>= (field-of it community/revolutionary) (field-of it community/liberal)) (if (>= (field-of it community/revolutionary) (field-of it community/fascist)) (field-of it community/revolutionary) (field-of it community/fascist)) (if (>= (field-of it community/liberal) (field-of it community/fascist)) (field-of it community/liberal) (field-of it community/fascist))) (field-of it community/revolutionary))) 0.000001c)
                ConsciousnessTendency/REVOLUTIONARY
                ConsciousnessTendency/FASCIST))))))))
