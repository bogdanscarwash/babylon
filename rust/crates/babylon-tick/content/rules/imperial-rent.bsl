; ImperialRentSystem (Material Base @9.0, NOT @10 — `economic.py:37`'s own
; `position: ClassVar[float] = 9.0`; "@10" was the 10th 0-indexed slot in
; `_DEFAULT_SYSTEMS`, not the position value) — the five-phase Imperial
; Circuit (Extraction -> Tribute -> Wages -> Subsidy -> Decision) that moves
; wealth from the exploited periphery through the core bourgeoisie's pool and
; back out as super-wages or repression. Frozen source:
; src/babylon/engine/systems/economic.py (836 lines, one step()). Port
; posture: ADR183 (structure/ordering contract, not a byte oracle) —
; conformance expecteds are measured from THIS engine, never copied from the
; frozen mirror's printed floats (imperial_rent_conformance.py's own header
; makes the same point, with a further named exception for the mirror's
; QUANTIZED `economy` graph-attribute print — see that file's header fact
; (d); BSL's own arithmetic is never quantized).
;
; **THE NAME COLLISION (plan §0) — read this before anything else.** THREE
; unrelated things in this codebase are called "Imperial Rent" / "Phi". THIS
; pack ports ONLY the first:
;   1. `engine/systems/economic.py::ImperialRentSystem` (@9.0) — the 5-phase
;      pool-based Imperial Circuit (Extraction/Tribute/Wages/Subsidy/
;      Decision). THIS FILE. Phases 1, 2, 3, 5 land; Phase 4 (Subsidy) does
;      NOT (Director-RESERVED, Constitution IX.5, §6 below).
;   2. `domain/economics/tick/system/imperial_rent.py` — the Leontief BEA
;      input-output pipeline invoked as Step 4 of TickDynamicsSystem (@4.0),
;      writing `CountyEconomicState.phi_hour`. NOT this pack — belongs to
;      `#563`. This system NEVER reads `phi_hour`/`tick_phi_hour` (zero grep
;      hits in economic.py) and has NO gamma/Leontief dependency at all.
;   3. `formulas/fundamental_theorem.py` — the W_c > V_c gap, consumed by
;      ContradictionSystem (@18) and the observe() projection. Already
;      landed as `fundamental-theorem.bsl` (12 lines, `economics/
;      fundamental-theorem`, the ONLY rule in the `economics` namespace).
;      THIS pack never calls or imports it and carries ZERO of its surface
;      (§2.3's duplication verdict) — it becomes the SECOND input's producer
;      only (`social-class/value-produced`, via r06, D195), never the first
;      (`social-class/wages` stays `fundamental-theorem.bsl`'s own fixture-
;      seeded input, B2, out of scope).
; A repo-wide rename of any of the three is OUT OF SCOPE for this train — a
; follow-on recommendation only.
;
; TASK 2 SHIP: `imperial-rent/r00-tick-reset` + `imperial-rent/r01-extraction`
; + `imperial-rent/r02-extraction-credit` — the per-tick accumulator reset and
; Phase 1 (Extraction), split into the wealth-transfer rule and the
; CORE_BOURGEOISIE carrier-credit rule (D201's duplication ledger, §8a).
;
; TASK 3 SHIP: `imperial-rent/r03-tribute` + `imperial-rent/r04-tribute-credit`
; — Phase 2 (Tribute), split the SAME way (D201's §8a). r03 transcribes the
; §1.6-c comprador wealth OVERWRITE (D189(b)) verbatim, using D200's
; repeated-`set` semantics (settled Task 0 Step 4(e), proven against the real
; driver Task 1 Step 5(g)); r04 mirrors r02's CORRECTED design exactly — it
; reads r03's SAME-TICK `tribute/value-flow` publication, never a fresh
; comprador-wealth re-read (r03 OVERWRITES that field first — the SAME hazard
; class D116/D197 ledger row 7 already named, ahead of time, for this exact
; rule). D184(b)'s Phase-2 source-re-read divergence (`800 -> 720 -> 648`
; frozen sequential vs. `800 -> 720` ported, world 10, new) is measured, not
; assumed.
;
; r05-r09 (Wages, the two cross-pack seams, Decision, pool decay) are Task 5+
; scope, named below only so the byte-order map and the D-record ledger are
; legible against the WHOLE pack before the rest lands — the same discipline
; `decomposition.bsl`'s own Task-2 commit followed for its not-yet-landed
; p03-p06.
;
; No `intrinsic` declaration in this file. Nothing in the four ported phases
; needs `floor`/`exp`/`log` — the transcendentals verdict (plan §5) is
; PURE-ARITHMETIC, and `#576`'s DECLARABLE_INTRINSICS estate is NOT a
; dependency of this pack.
;
; D116 BYTE-ORDER MAP (docs/reference/bsl-language.rst) — rules run to
; completion in ascending rule-id byte order against the same mutable graph,
; so every rule below sees every earlier rule's same-tick writes.
; `r00 < r01 < r02 < r03 < r04 < r05 < r06 < r07 < r08 < r09` (ASCII byte
; order on the two-digit suffix, contiguous). Every same-tick read across
; this pack is a DELIBERATE reliance on that order, decomposition.bsl/
; control-ratio.bsl-header style. Rules marked "(Task N)" are not yet
; landed; they are named here so the whole pack's shape is visible from
; Task 2 onward, exactly as decomposition.bsl's own Task 2 header named its
; then-unlanded p03-p06.
;
;   rule                     subject       reads                              writes
;   r00-tick-reset           INSTITUTION   nothing (explicit `(domain          rent-tribute-inflow
;                                          NodeType/INSTITUTION)`, §2.3 — the   (set 0), on the D198
;                                          effects never reference `self`, so   discriminator target;
;                                          domain.rs's load-time self-scoped   EVERY TRIBUTE edge's
;                                          inference finds zero candidates;    tribute/value-flow
;                                          an explicit domain is the           (set 0), D202 — a PORT
;                                          documented repair)                  ARTIFACT, no frozen
;                                                                              analog (review fix
;                                                                              round 2, N1)
;   r01-extraction           SOCIAL_CLASS  active, wealth, revolutionary       self wealth (sub rent),
;   (worker)                              (:optional :default 0.0p, B7's      it wealth (add rent),
;                                          re-point target), :const economy/   exploitation/value-flow,
;                                          extraction-efficiency/trpf-*/       SURPLUS_EXTRACTION emit
;                                          negligible-rent, :const timescale/
;                                          weeks-per-year, :tick, per-`it`
;                                          active (EXPLOITATION neighbours)
;   r02-extraction-credit    SOCIAL_CLASS  active; per-`it` active AND role ==  carrier rent-tribute-
;   (worker)                              CORE_BOURGEOISIE; r01's SAME-TICK    inflow (add), carrier
;                                          `exploitation/value-flow` write      rent-pool (add), BOTH on
;                                          (D116/D197 row 7, NOT an             the D198 discriminator-
;                                          independent re-derivation — a        scored carrier — the
;                                          fresh :field wealth re-read here     `field-of (edge-between
;                                          would read r01's ALREADY-MUTATED     ...) exploitation/
;                                          wealth, corrected mid-Task-2)        value-flow` read
;   r03-tribute               SOCIAL_CLASS  active, wealth, :const economy/     self wealth (set cut) —
;   (comprador)                           comprador-cut, per-`it` active      the OVERWRITE, D200's
;                                          (TRIBUTE neighbours)                repeated-set shape
;   r04-tribute-credit       SOCIAL_CLASS  self active; per-`it` active AND    carrier rent-tribute-
;   (comprador)                           role == CORE_BOURGEOISIE AND        inflow/rent-pool (add
;                                          `tribute/value-flow > 0` (per-edge, tribute), BOTH on the
;                                          C1 fix round 1 — NOT a self-level   D198 discriminator-
;                                          wealth re-read, which would read    scored carrier
;                                          r03's OWN OVERWRITE); r03's
;                                          SAME-TICK `tribute/value-flow`
;                                          write (D116/D197 row 7, NOT an
;                                          independent re-derivation)
;   r05-wages-crisis         SOCIAL_CLASS  carrier rent-tribute-inflow/        carrier superwage-crisis-
;   (Task 5, worker)                      rent-pool (r02/r04, SAME TICK, D197  known/-tick (constant-
;                                          ledger row 2), carrier rent-wage-   score carrier, the B8/D194
;                                          rate, employer WAGES edge (`select- exception), SUPERWAGE_
;                                          max` single-employer, D185)         CRISIS emit
;   r06-wages-pay             SOCIAL_CLASS  employer wealth/active, self       employer wealth (sub
;   (Task 5, worker)                      active, production-value (:optional total), self wealth (add
;                                          :default 0), r05's `bonus` re-       total), effective-wealth,
;                                          derived (D201 4th copy)             unearned-increment, ppp-
;                                                                              multiplier, wages-paid,
;                                                                              value-produced (D195's
;                                                                              first-writer seam),
;                                                                              wages/value-flow
;   r07-wages-pool             SOCIAL_CLASS  SAME gate as r06, r06's `total`    carrier rent-pool (sub
;   (Task 6, worker)                        re-derived (D201's 4th copy)       abp), discriminator-scored
;   r08-decision (Task 7)      INSTITUTION   carrier rent-pool (r02/r04/r07,   carrier rent-wage-rate,
;                                            SAME TICK, D197 ledger row 5),    rent-repression-level,
;                                            economy/initial-rent-pool,        ECONOMIC_CRISIS emit on
;                                            rent-aggregate-tension (seeded,   decision == CRISIS
;                                            never written, D187), the five-
;                                            branch decision matrix consts
;   r09-pool-decay (Task 7)    INSTITUTION   carrier rent-pool (r08 reads,     carrier rent-pool (set
;                                            does not write, + r07, D197       decayed), AFTER r08 reads
;                                            ledger row 6), economy/rent-       it — the ordering vector
;                                            pool-decay
;
; **THE D116 LEDGER'S SEVEN SAME-TICK CROSS-RULE READS (D197), named
; individually per §8b — not summarised (row 7 discovered mid-Task-2, a
; genuine correctness fix, not a documentary afterthought):**
;   1. r05, r06 read `institution/rent-tribute-inflow`, written same tick by
;      r02, r04.
;   2. r05, r06 read `institution/rent-pool`, written same tick by r02, r04.
;   3. r02, r04 read `institution/rent-tribute-inflow` (as the `add` TARGET)
;      as reset same tick by r00 — the reset must apply BEFORE r02/r04's
;      first add, or the accumulator compounds across ticks (the exact
;      failure `production.bsl`'s own `p0` reset row records,
;      `bsl-language.rst:6693-6697`).
;   4. r07 reads `institution/rent-pool`, written same tick by r02, r04.
;   5. r08 reads `institution/rent-pool`, written same tick by r02, r04, r07
;      — `pool-ratio` computed on THIS tick's pool, so the decision branch
;      can differ from a world where D116 is repaired.
;   6. r09 reads `institution/rent-pool`, written same tick by r08 (reads,
;      does not write) and r07 — decay applied to a stale base if D116 is
;      repaired.
;   7. (NEW, THIS task) r02 reads `exploitation/value-flow`, written same
;      tick by r01, via `(field-of (edge-between EdgeType/EXPLOITATION self
;      it) exploitation/value-flow)` — NOT an independent re-derivation of
;      `rent` from wealth/consciousness. Discovered as a genuine bug during
;      Task 2's own gate: r01 runs FIRST (byte order) and MUTATES the
;      worker's OWN `social-class/wealth`, so a naive fresh `:field
;      social-class/wealth` re-read in r02 (the earlier design) silently
;      read POST-r01 wealth, producing a WRONG (smaller) rent —
;      `r01_and_r02_agree_on_the_rent` caught it red before the fix landed.
;      This SAME hazard class recurred for r03/r04 (THIS task): r04 reads
;      r03's published `tribute/value-flow`, NOT a re-derived `cut`/`tribute`
;      from a fresh `:field social-class/wealth` read on the comprador,
;      since r03 OVERWRITES that field first. Flagged here explicitly ahead
;      of time so this task did not rediscover it the hard way, and applied
;      unchanged when r04 was written.
; D197 records the whole ledger; when the Q14 collect-across-rules-then-
; apply repair train lands, rows 1-6 silently start reading pre-tick state,
; and no single-tick transfer test would flip for THOSE six (every one of
; their inputs is tick-invariant) — only the two-tick accumulator
; assertions (Task 8) are the guard for rows 1-6. **Row 7 is NOT
; tick-invariant and is better-guarded than that blanket claim (review
; finding, fix round 1, Minor 1):** `exploitation/value-flow` is seeded
; 0.0 and written by r01 in the SAME tick, so under Q14 r02 would credit 0
; instead of `rent`, and TWO single-tick tests would flip red immediately —
; `r01_and_r02_agree_on_the_rent` and
; `r02_credits_only_a_core_bourgeoisie_target` — with no Task 8 wait
; required. This row is that future train's own acceptance-criterion input
; for imperial-rent specifically; the blanket "no single-tick test flips"
; sentence above governs rows 1-6 only.
;
; HISTORICAL §7/D190 RECORD, superseded for execution by the governed phase
; registry (ADR222/PER-17): the former FOUR inter-pack byte-order inversions.
; This pack's
; own namespace, `imperial-rent`, sorts AFTER `consciousness, control-ratio,
; decomposition, dispossession, economics` and BEFORE `lifecycle, metabolism,
; organization, production, solidarity, territory, vitality` (`economics` <
; `imperial-rent` < `lifecycle`, corrected namespace list, plan §7 —
; `fundamental-theorem/` and `worldview/` are NOT namespaces; both files'
; rules live in `economics`/`consciousness` respectively):
;   (7.1) `production/` — frozen Production @3.0 -> ImperialRent @9.0;
;         PORTED `imperial-rent` < `production`, so Phase 3 (r06, Task 5)
;         reads a STALE `social-class/production-value` whenever a combined
;         world's production moves tick-to-tick. Harmless when
;         production-value is seeded and never rewritten (this pack's own
;         Task 2-4 worlds). Constraint: every conformance scenario SEEDS
;         `social-class/production-value` directly rather than relying on
;         same-tick production (the "seed the post-something state" idiom,
;         control-ratio's `_class_decomposition_tick` precedent). Documentary
;         `(anchor :after production)` is NOT declared in this pack (no
;         landed rule in this file needs it yet; Task 5+ revisits). Closed
;         by world 11's two-tick unseeded-production-value fixture (Task 8).
;   (7.2) `consciousness/` — frozen ImperialRent @9.0 -> Consciousness @16;
;         PORTED `consciousness` < `imperial-rent`, so ALL FOUR readers of
;         `wages-paid`/`value-produced` (`p0-position`, `p4-wage-balance`,
;         `p5-agitation`, `p7-persist-baselines`) plus `p2-wages-push` (on
;         `wages/value-flow`) run BEFORE r06 (Task 5) writes them — every
;         §2.2 seam is exactly ONE TICK LATE. Nothing lost, nothing
;         same-tick. Closed by world 12's two-tick combined fixture (Task 8).
;   (7.3) `decomposition/` — frozen ImperialRent @9.0 EMITS, Decomposition
;         @11 READS the same tick's event history and takes `min(e.tick)`;
;         PORTED `decomposition` < `imperial-rent`, so
;         `decomposition/p02-superwage-warning` runs FIRST in a combined
;         world, its OWN `(= crisis-known 0)`-guarded latch fires first, and
;         r05's (Task 5) identically-guarded latch write correctly no-ops.
;         Harmless for the delay clock (both rules latch the SAME tick);
;         harmful only for a consumer keying on r05's own seven-key PAYLOAD
;         (no such consumer exists in the BSL estate today). Closed by world
;         12's two guard-permutation vectors (Task 8).
;   (7.4) `economics/` — NO frozen dependency at all (ImperialRentSystem never
;         imports `formulas/fundamental_theorem.py`); the inversion is a
;         PORT ARTIFACT, not a frozen-order one. PORTED `economics` <
;         `imperial-rent`, so `economics/fundamental-theorem` reads the
;         PREVIOUS tick's `value-produced` against a fixture-seeded
;         `social-class/wages` (B2, out of scope). **This train does NOT
;         build a combined `economics` + `imperial-rent` world** — D195
;         records the half-anchored asymmetry and its re-open trigger (the
;         ContradictionSystem @18 port) instead of pinning a half-anchored
;         result as if it were a contract.
;
; D-RECORDS this pack transcribes — TWENTY-TWO rows, D181-D202 (D202 added
; review fix round 2, N1 — a genuine port artifact discovered mid-train, not
; reserved in advance like D181-D201), RESERVED here in full per Task 2's
; own instruction (global register numbers
; NEXT-FREE-AT-LANDING as of the Task 0 dossier's 2026-08-18 measurement —
; D180 was `docs/reference/bsl-language.rst`'s tail; Task 9 re-measures
; before actually filing — MANDATORY, not merely prudent: `bsl-language.rst`
; and `ai/decisions/index.yaml` are SHARED, upstream-moving registries other
; in-flight trains also allocate from, and the Task 0 dossier's own
; CORRECTIONS 2 records exactly this already happening ONCE to the ADR tail
; between this dossier's original scout pass and its fix round — the SAME
; drift can hit the D-row tail before Task 9 files). Rows about not-yet-
; landed rules are marked "(Task N)"; D181, D182 (partially), D184
; (partially, (a) only — (b) now lands with THIS task), D189 (partially),
; D196, D198 (partially), D199, D200, D201, D202 are load-bearing for
; r00-r04 as landed here — the rest are reserved so the numbering never
; shifts under a later task, the same discipline decomposition.bsl's own
; nine-row Task-2 header already established for this train's precedent
; pack:
;   1. (D181) `GlobalEconomy` + `tick_context` -> the INSTITUTION carrier.
;      The `imperial-rent-register` node (Task 1); the `institution/rent-*`
;      prefix convention and why (field-roster disjointness with the
;      carceral carrier's 18 fields, plan §3.1); r00 (THIS task) as the
;      ported form of the frozen per-tick `tick_context` re-creation
;      (`economic.py:59-66`), `rent-pool`'s EXEMPTION from it (it is the
;      persistent `GlobalEconomy` field, not a per-tick local), and the fact
;      that r00 is provable only across TWO ticks (a one-tick fixture cannot
;      distinguish "resets correctly" from "does nothing" — the exact shape
;      `production.bsl`'s own `p0` reset row hit, `bsl-language.rst:6686-
;      6697`; Task 8's two-tick arc test is r00's mutation killer, deferred
;      there per Step 7 below). Carrier IDENTITY is D198's row, not this one.
;   2. (D182) The push form is FORCED. No `source-of`/`target-of` accessor
;      exists (`edge_lane_e2e.rs:186-189` — "the language does not have
;      one"), so `(edges …)` pull-iteration cannot write both endpoints;
;      every edge-driven phase is SELF-ANCHORED: `(for-each (neighbors self
;      EdgeType/X :out NodeType/SOCIAL_CLASS) …)` with the edge reached via
;      `(edge-between EdgeType/X self it)` — the `edge_lane_e2e.rs:196-206`
;      idiom, already landed in `consciousness.bsl`/`production.bsl`
;      content. r01/r02 (Task 2) anchor on the SOURCE (the worker — `rent`
;      depends only on the source's own fields); r03/r04 (THIS task, Tribute)
;      anchor on the source likewise (the comprador, `tribute`); r06/r07
;      (Task 5-6, Wages) anchor on the TARGET (the worker) instead, since
;      `productivity_value` is the worker's own field and the single
;      employer is reached via `(select-max (neighbors self EdgeType/WAGES
;      :in NodeType/SOCIAL_CLASS) 1)` — the landed `production.bsl:216`
;      idiom (D185's single-employer assumption). Iteration order also
;      changes: frozen order is `query_edges` insertion order; ported order
;      is (subject-node order) x (per-node neighbour order, ascending
;      `(source, target)`, `edge_write_lane_e2e.rs:56-57`) — order-
;      equivalent for per-edge-independent effects (r01/r02's own shape);
;      order-DIVERGENT for float accumulation into a shared carrier, which
;      is D183's row.
;   3. (D183, Task 6) The sequential pool depletion does not port. `for-each`
;      reads PRE-state (`structural_verbs.rs:130,750` — the collect-then-
;      apply law); Phase 3's frozen `available_pool = tick_context[
;      "current_pool"]  # Update for next iteration` (`:544`) is precisely a
;      mid-loop read of a running total, unrepresentable under that law.
;      Disposition (no Director gate — ADR183's own "reformulation with a
;      D-row" class, D165's precedent): transcribe the per-edge arithmetic
;      verbatim against a TICK-START pool snapshot, apply depletion as a
;      BATCHED accumulation (r07 subtracts every edge's `actual_bonus_paid`
;      from `rent-pool` in one pass) — byte-identical to the frozen
;      sequential form whenever `Σ max_bonus <= available_pool` (the pool
;      does not bind), diverging only when it does. World 8 (Task 6) is
;      built specifically to bind it; both numbers (frozen sequential,
;      ported batched) are published side by side.
;   4. (D184) The per-edge sequential SOURCE re-read does not port, in BOTH
;      phases. (a) Phase 1 (r01, Task 2): the frozen loop re-reads
;      `worker_attrs["wealth"]` at the top of EVERY EXPLOITATION-edge
;      iteration (`economic.py:283`); the ported `rent` is ONE rule-scoped
;      binding, computed once from pre-state and applied per edge — a
;      worker with N EXPLOITATION edges ends at `wealth - N*rent`, not the
;      frozen engine's `wealth` after N successive re-reads. Measured on
;      world 8 (Task 6); it is ALSO the world where D196's dropped clamp
;      becomes observable (the two facts share one fixture, not two). (b)
;      Phase 2 (r03, THIS task): `economic.py:375` re-reads
;      `source_attrs["wealth"]` per TRIBUTE edge, so a two-edge comprador
;      takes a second cut off the ALREADY-OVERWRITTEN balance (800 -> 720 ->
;      648) where the ported rule-scoped `cut` writes 720 TWICE (800 ->
;      720). MEASURED on world 10 (THIS task, new) — see that scenario's
;      own header and `the_two_tribute_edges_apply_the_rule_scoped_cut_once`
;      for both numbers published side by side. The earlier plan draft
;      covered only (a); this row now covers both.
;   5. (D185, Task 5) The single-employer `select-max` assumption
;      (`production.bsl:216`'s D45/D145 precedent class). r06's employer
;      binding, `(select-max (neighbors self EdgeType/WAGES :in
;      NodeType/SOCIAL_CLASS) 1)`, resolves exactly ONE employer per worker.
;      A world with TWO WAGES edges into one worker is UNPORTED behavior,
;      not equivalent behavior — no fixture in this train builds one.
;   6. (D186, Task 9-filed, STOP) Phase 4 STOP. The full §6 packet: Phase 4
;      (`_process_subsidy_phase`, `economic.py:546-666`) is UNPORTED — not
;      scoped out for convenience, a Constitution IX.5 reserved-line STOP.
;      M1 (the frozen logistic P(S|A) call is INHERITED per ADR202 R3/ADR173
;      decision 1 — MUST NOT be transcribed; the emergent re-derivation is a
;      SEPARATE train's scope) quoted verbatim; M2 (the `population`/
;      `inequality` CLIENT_STATE-edge-target seeding gate) RECORDED, and the
;      landing gate M2 names is explicitly stated NOT DISCHARGED — never
;      "discharged," per the plan's own wording discipline; R1 (whether the
;      comprador node is the correct carrier for "the client state") and R2
;      (the within-class wealth-dispersion FAMILY — uniform vs. smooth
;      heavy-tail — that decides ramp vs. step vs. S-curve) stated VERBATIM
;      as OPEN and UN-RULED, escalated, never silently resolved; R3 (`#510`'s
;      provisional income-shape proxy expiry) named as binding M2; M3
;      (`steepness_k` removal from `canonical_defines.json`) explicitly NOT
;      discharged by this train and explicitly NOT to be done early (the
;      Phase-4 frozen mirror still needs it); R4 (ADR171's national-question
;      axis, MIM+MLP bribe:deprivation coupling) recorded as CONSIDERED AND
;      DECLINED — coupling it into the Phase-3 super-wage would be NEW
;      reserved scope (Constitution I.1), and §2.2's chauvinism consequence
;      (below, D195) rides the ALREADY-LANDED `wage-balance` path with no new
;      coupling minted. A world with CLIENT_STATE edges is unported
;      behavior. `EdgeType/CLIENT_STATE` is NOT declared anywhere in this
;      train's content (no speculative declarations).
;   7. (D187) `opposition_states` has no producer on the ported estate.
;      ContradictionSystem @18 is unported; nothing in the BSL content
;      estate writes an opposition/tension gap. The frozen read
;      (`_calculate_aggregate_tension`) is ALREADY one-tick-stale in the
;      frozen engine itself and ALREADY defaults to 0.0 when absent.
;      Disposition: `institution/rent-aggregate-tension` is a SEEDED carrier
;      field (the "seed the post-something state directly" idiom,
;      control-ratio's own `_class_decomposition_tick` precedent) — NEVER
;      written by any rule in this pack, not even r08 (Task 7). Re-open
;      trigger: when ContradictionSystem ports, this pack owes a producer-
;      written-field handoff.
;   8. (D188, Task 5/7) Payload divergences — FIVE, not four. `mechanism`
;      ("imperial_rent") and every `narrative_hint` string are DROPPED —
;      there is no `Str` payload value (`emit` carries no string payloads at
;      all). `decision` (r08, Task 7) encodes `0..4` in
;      `BourgeoisieDecision`'s OWN declaration order
;      (`dynamic_balance.py:18-22`: NO_CHANGE=0, BRIBERY=1, AUSTERITY=2,
;      IRON_FIST=3, CRISIS=4 — verify against the source at Task 7, not
;      assumed here). `bourgeoisie_active` (r05, Task 5) encodes 0/1. **The
;      `source_id`/`target_id`/`payer_id`/`receiver_id` -> `source`/
;      `target`/`payer`/`receiver` id-string -> NodeRef key rename
;      (BLOCKER-5b) — landed HERE, r01's own SURPLUS_EXTRACTION payload is
;      the FIRST instance of this rename in this pack**: frozen keys carry
;      Python id STRINGS; there is no `Str` runtime variant, so the ported
;      payload carries the SAME relationship as a NodeRef under a renamed
;      key (`source`/`target`, dropping the `_id` suffix deliberately — a
;      NodeRef is not an id string, and keeping the suffix would misdescribe
;      the value's type) — `decomposition.bsl`'s own D171 item 1 is the
;      precedent that records the identical class of rename. The rejected
;      `(defenum BourgeoisieDecision …)` alternative (Task 7's decision) is
;      recorded as the shape to adopt if a reviewer prefers it over the
;      bare numeric code.
;   9. (D189) The transcribed frozen defects — two still open (Task 5,
;      Task 7), one CLOSED at Task 2 (see item 16/D196 below, split out of
;      this row because it resolved into a DECISION about an AST rather
;      than a plain transcription note), one CLOSED at THIS task ((b)
;      below): (a, Task 5) the dead second conjunct in the
;      SUPERWAGE_CRISIS condition (`available_pool <= negligible and
;      super_wage_bonus <= negligible` — the second conjunct is implied by
;      the first since `super_wage_bonus = min(max_bonus, available_pool) <=
;      available_pool`; transcribe both, the mutation vector showing NO
;      test flips when the second is dropped IS the evidence, not a failure
;      to find a killer). (b, THIS task) the Phase-2 wealth OVERWRITE
;      (`source.wealth = cut_amount`, `:385` — a `set`, not the `sub` every
;      other phase uses; D200's repeated-set question rides the same rule).
;      (c, Task 7) the `bribery_tension_threshold` 0.3-vs-0.7
;      signature-default/docstring drift (`dynamic_balance.py:38` defaults
;      0.3; the module's own worked-example matrix is written against that
;      default; `defines.yaml:97` SHIPS 0.7 and `ImperialRentSystem` passes
;      the define — the SHIPPED 0.7 governs, the code is the port's oracle,
;      not its own docstring).
;   10. (D190, HISTORICAL; superseded by ADR222/PER-17) The inter-pack
;       byte-order inversions — FOUR, each with its
;       own executable constraint. See the §7 disclosure block above (this
;       header, immediately before this D-record list) for the full text of
;       all four (7.1 production, 7.2 consciousness, 7.3 decomposition, 7.4
;       economics); this row is the register's OWN home for that content,
;       restated in full at Task 9 rather than cross-referenced only.
;   11. (D191) The duplication boundary with `economics/fundamental-theorem`
;       (§2.3's verdict, B1-B3 as a record): this pack writes NEITHER
;       `social-class/imperial-rent` NOR `social-class/wages` — both belong
;       to `economics/fundamental-theorem`/its fixture. `social-class/
;       production-value` is READ from `production.bsl`'s publication,
;       NEVER recomputed. The `wages-paid`/`value-produced` cross-scenario
;       `int`-vs-`real` deffield type divergence (`consciousness-ternary-
;       conformance.bscn` declares `int extensive`; THIS pack's own
;       scenario — Task 1 — redeclares `real extensive`, B5, legal
;       per-scenario redeclaration, Task 0 dossier Step 4(f)). The §0
;       three-way name collision (above), named so the next reader is never
;       trapped by it. The PRODUCE-side seams are D194 and D195, not this
;       row — this row is the read-side/duplication verdict only.
;   12. (D192) The provably-absent wiring. The two spec-063 sub-stages
;       (`_invoke_phi_distribution_if_wired`, `_invoke_vol2_circulation_
;       if_wired`, `:88-156,158-199`) and the Phase-1 L-RECEIPTS
;       `boundary_register.record(...)` (`:310-321`) are all SILENT NO-OPS
;       in every unit test and every qa-six scenario — `ServiceContainer.
;       create()` never binds `boundary_register`, `TickContext.
;       persistent_data` defaults to `{}`. No BSL lane exists for any of
;       the three; all omitted, each with a named re-open trigger (phi
;       distribution rides the `#563` Φ estate; Vol-II circulation and the
;       boundary register ride a future session-infrastructure lane).
;   13. (D193) The dead subsistence phase. `_process_subsistence_phase`
;       (`:201-237`) is `.. deprecated:: ADR032` and is NEVER CALLED by
;       `step()` — dead code in the frozen engine itself. NOT PORTED. Its
;       11-test unit file (`test_subsistence.py`) is not a conformance
;       candidate for this train.
;   14. (D194, Task 5, CRITICAL SEAM) The SUPERWAGE_CRISIS producer seam and
;       the latch (§2.1). The frozen `decomposition.py:161-175` `min(e.tick)`
;       event-history scan and its DUAL role (feeds Decomposition's delay
;       clock AND suppresses `p02`'s own emit via the `superwage_tick is
;       None` gate). `decomposition.bsl:104-113`'s own D168 row PRESCRIBES
;       the re-modelling verbatim: "the emitting rule also stamps a field."
;       r05 (Task 5) STAMPS `institution/superwage-crisis-known`/`-tick`
;       under `decomposition.bsl`'s OWN qnames and its OWN constant-score
;       carrier expression (`(select-max (nodes NodeType/INSTITUTION) 1)`,
;       the DELIBERATE exception to this pack's own discriminator
;       convention — D198's own exception clause), behind a `(=
;       crisis-known 0)` guard that reproduces `min` rather than `last`
;       (first-writer-wins). The two-emitter payload divergence
;       (`p02`'s three keys vs. r05's seven; `payer`'s PRESENCE is the
;       discriminator between them) and the reconciliation with
;       `decomposition.bsl`'s own D171 item 1 (`payer_id` is a CONSTANT
;       there — always CORE_BOURGEOISIE_ID — and EDGE-DERIVED here, so the
;       identical rule — "drop a constant, keep a variable" — yields
;       OPPOSITE dispositions on different facts, not a contradiction). The
;       §7.3 inversion and the two tests that close it (world 12, Task 8).
;   15. (D195, Task 5, CRITICAL SEAM) First-writer status over THREE
;       quantities (§2.2). r06 becomes the first writer of `social-class/
;       wages-paid`, `social-class/value-produced` and `wages/value-flow`.
;       The full reader census: `economics/fundamental-theorem:9` (required,
;       half-anchored per §7.4/this row's own re-open trigger);
;       `consciousness/p0-position`, `p4-wage-balance`, `p5-agitation`,
;       `p7-persist-baselines` (all four `:optional :default -1`);
;       `consciousness/p2-wages-push` (on `wages/value-flow`). What the
;       writes turn ON: an UNPOSITIONED class becomes POSITIONED
;       (`p0-position`'s anchor-presence gate); a non-zero `wage-balance`,
;       whose POSITIVE half routes agitation FASCIST-ward through
;       `p6-route`'s `chauvinist` term; a LIVE wage flow, which turns on
;       `p5-agitation`'s previously-dead exploitation term. The sign
;       analysis: `wages - value = min(bonus, employer-wealth -
;       production-value)` — positive exactly when a super-wage is paid,
;       negative ONLY when the employer's own wealth constraint binds below
;       production-value (the capital-constrained-employer fixture, world
;       12). The HALF-ANCHORED `fundamental-theorem` guard is RECORDED, not
;       repaired, with the ContradictionSystem-@18-port re-open trigger.
;       ALL of this is DERIVED from two independently port-as-is
;       transcriptions and mints NO coefficient, NO functional form and NO
;       new axis (R4 above recorded as considered and declined) — stated
;       for the Director's information in ADR214 (Task 9), not as a design
;       proposal.
;   16. (D196, THIS task, r01) The `economic.py:295` clamp is NOT
;       transcribed, and why (§1.6-a). `economic.py:295` reads `source.
;       wealth = max(0.0, worker_wealth - rent)` — a SET WITH A CLAMP
;       against the wealth read at the TOP of THIS iteration. The
;       REJECTED clamp-preserving AST — `(set (if (> diff 0c) diff (- 0
;       0c)))` with `diff` a rule-scoped `(- wealth rent)` binding — is
;       rejected because, under the pre-state law, a `set` inside a
;       `for-each` writes the SAME pre-state-derived value on EVERY
;       iteration, so a worker with TWO EXPLOITATION edges would end at
;       `wealth - rent` where the frozen loop decrements TWICE — `set`
;       would silently REPAIR the frozen per-edge repetition, a behavior
;       change in the OPPOSITE direction from D184's own declared
;       divergence and a port-as-is violation. The ADOPTED AST is `(sub
;       rent)` — accumulates, matching the frozen shape's own direction.
;       REACHABILITY PROOF that the frozen clamp is dead in EVERY
;       frozen-reachable world: `rent = min(eff * worker_wealth * (1 -
;       consciousness), worker_wealth) <= worker_wealth` (`:292`'s own
;       `min`), and the frozen loop RE-READS `worker_attrs["wealth"]` at
;       the top of EVERY iteration (`:283`), so the invariant
;       `worker_wealth - rent >= 0` holds at EVERY SINGLE iteration in the
;       frozen engine — `eff` and `(1 - consciousness)` only SHRINK the
;       product, and the `min` caps it at the balance; the clamp can never
;       bind there. `r01_never_drives_a_single_edge_worker_negative`
;       (Task 2's own test, below) is the CONVERSE witness on THIS pack's
;       own one-edge world — where the invariant DOES hold, matching the
;       frozen proof exactly. Where the invariant does NOT hold: the
;       PORTED `rent` is a rule-scoped binding computed ONCE from pre-state
;       and applied per edge (D184(a)), so a worker with N EXPLOITATION
;       edges ends at `wealth - N*rent`, which CAN go negative for N >= 2
;       and `eff*(1-c) >= 0.5` — NOT a new divergence class, D184's own
;       declared divergence seen from its second face, but the clamp's
;       absence becomes OBSERVABLE in the ported estate where it was
;       unobservable in the frozen one. World 8 (Task 6) seeds exactly that
;       worker (two EXPLOITATION edges) — MEASURED, not assumed; the ported
;       number and the frozen number are published side by side there.
;       r09's own `max(0, pool)` (Task 7) and r07's `max(0, actual_bonus_
;       paid)` (Task 6) are LIVE clamps, transcribed as `if` chains — this
;       row is about `:295` alone.
;   17. (D197) The D116 cross-rule apply-in-place reliance (§8b) — see the
;       "D116 LEDGER" block above (this header) for the full six-row text,
;       restated in full at Task 9 rather than cross-referenced only.
;   18. (D198, THIS task, r02) Carrier identity is FIELD-GUARDED, not
;       id-tiebroken. A CONSTANT-score `select-max` over `(nodes
;       NodeType/INSTITUTION)` is EXACTLY "the lowest-id INSTITUTION node"
;       (D45's ascending-id tiebreak, `evaluator.rs:990-1052`) — in a world
;       holding both `carceral-register` and `imperial-rent-register`,
;       `c` < `i`, so a naive constant score would resolve to
;       `carceral-register` and this pack's carrier would be INERT. The
;       `institution/rent-carrier` discriminator (`int extensive`, `1` on
;       THIS pack's carrier, `0` elsewhere) and its landed-score precedents
;       (`query_lane_e2e.rs:252-253`, `r9_chapters.rs:1129-1130,1179-1180` —
;       a bare `int`/`real` `field-of` classifies as `ScoreClass::Scalar`,
;       on `E-TYPE-016`'s allowed list). The DELIBERATE exception: B8's
;       latch writes (r05, Task 5) use `decomposition.bsl`'s OWN
;       constant-score expression VERBATIM, because a latch is only useful
;       on the node its consumer (`decomposition/p03-trigger`) reads, and
;       that consumer's OWN binding is itself a constant score. The
;       both-packs-world shape: ONE INSTITUTION node, TWO disjoint rosters
;       (the `rent-` prefix keeps them disjoint on that one node). The
;       LATENT HAZARD recorded and NOT repaired: `decomposition.bsl`'s 14
;       constant-score reads would silently tiebreak to the lowest id
;       should a world ever mint TWO INSTITUTION nodes — repairing
;       `decomposition.bsl` moves ITS landed golden pins, a separate
;       ceremony. Re-open trigger: the unserved `the` query head
;       (`evaluator.rs:545`, slice 2) — when it lands, both packs' carrier
;       reads become `(the NodeType/INSTITUTION)` and the discriminator
;       retires. `carrier_discriminator_resolves_over_a_lower_id_decoy`
;       (Task 1, landed) is this row's own executable proof. **STANDING
;       OBLIGATION, adjudicated fix round 1 (review Minor 3, KEPT not
;       relaxed):** every INSTITUTION node in ANY world loading this pack
;       (this pack's own scenarios, or a future combined world) MUST
;       declare `institution/rent-carrier` — r00's anchor binding
;       (`:field institution/rent-carrier`, no `:optional`) is REQUIRED, on
;       purpose. A second INSTITUTION node that omits the field is a hard
;       load/run failure (`bind_subject`'s loud III.11 propagation,
;       `tick.rs:212-224`), not a silent no-op, because the carrier
;       discipline this whole row establishes is exactly the kind of thing
;       a silent no-op would mask: a world-authoring bug (an INSTITUTION
;       node minted without the discriminator declared) must fail loudly,
;       not resolve to whichever carrier happens to win a tiebreak it was
;       never meant to need. `:optional :default 0` was considered and
;       REJECTED for this reason.
;   19. (D199, THIS task, r00) `institution/rent-wages-outflow` is DROPPED —
;       NOT declared anywhere in this pack's scenarios. `tick_context[
;       "wages_outflow"]` is a PER-TICK LOCAL that `_save_economy`
;       (`:827-836`) never persists into `GlobalEconomy` and that nothing
;       reads (not a later phase, not an event payload, not the saved
;       economy). Declaring it as a carrier field would FABRICATE
;       persistence the frozen engine does not have, AND would be
;       hash-bearing with no semantic content (entering graph state, hence
;       the tick hash, for nothing) — port-as-is and declare-what-you-read
;       genuinely conflict here, and this row resolves it in the direction
;       that does NOT move the hash for a write-only field. The equivalent
;       OBSERVABLE stays available in the arithmetic: `Δ rent-pool == -Σ
;       actual_bonus_paid` across the tick (Task 6's own rows assert this
;       directly). Re-open trigger: a later train needing a PUBLISHED
;       per-tick outflow (a Sankey lens, an `observe()` page, a Vol-II
;       circulation consumer) declares the field AND its reader in the SAME
;       landing — never the field alone.
;   20. (D200, THIS task, r03) The repeated `set` of one field within
;       one tick. `r03-tribute`'s `(update-node self social-class/wealth
;       (set cut))` sits inside a `for-each` over TRIBUTE neighbours, so a
;       comprador with TWO TRIBUTE edges collects TWO `set` effects on the
;       SAME field in the SAME tick, both carrying the SAME pre-state-
;       derived value. Settled at Task 0 Step 4(e) and PROVEN against the
;       real driver at Task 1 Step 5(g): accepted, never refused;
;       last-write-wins for `set` (a repeated identical `set` is
;       idempotent by construction here, since `cut` does not vary by
;       edge); `add`/`sub` differ — they read the CURRENT, already-mutated-
;       this-batch stored value at APPLY time and genuinely accumulate. Its
;       interaction with D184(b): the frozen engine's own Phase-2 re-read
;       (`economic.py:375`) is a DIFFERENT divergence (the frozen loop
;       re-reads and re-multiplies against an already-overwritten balance,
;       `800 -> 720 -> 648`) from the ported repeated-`set` shape
;       (`800 -> 720`, twice) — D200 is about the LANGUAGE's own collision
;       semantics; D184(b) is about the ARITHMETIC divergence those
;       semantics produce relative to the frozen engine. World 10 (THIS
;       task, new) measures both together —
;       `the_two_tribute_edges_apply_the_rule_scoped_cut_once` publishes
;       both numbers side by side, exactly the D183 publication discipline.
;   21. (D201, Task 2) The duplication ledger (§8a below). The FOUR
;       expressions transcribed more than once across this pack (`rent`:
;       r01/r02, Task 2; `cut`/`tribute`: r03/r04, THIS task; `super-wage-
;       bonus`: r05/r06/r07, Task 5-6; `total-wages`: r06/r07, Task 5-6) —
;       why single-sourcing is NOT AVAILABLE (the closed `.bsl`/`.bscn`
;       top-form sets carry no `defexpr`, no macro, no cross-rule `let` —
;       `bsl-language.rst:650-652`/`scenario.rs:561-628` — so "single-
;       source" means exactly one thing, "merge the rules," which costs the
;       independent mutation-killability the split bought: r02's role gate
;       and r03's `set`-vs-`sub` asymmetry would stop being independently
;       killable, and the merged rules' fuel bounds would compound); the
;       copies-agree rows (§8a's table); and the perturb-ONE-copy mutation
;       vector that proves each row catches DRIFT rather than restating a
;       transfer test (r02's own vector, Step 7 below, is this row's first
;       executed instance).
;   22. (D202, review fix round 2 — N1) `r00`'s SECOND effect — zeroing
;       EVERY `tribute/value-flow` edge every tick — is a PORT ARTIFACT with
;       NO frozen analog, verified by grep: `value_flow` is written exactly
;       four times in `economic.py` (one per phase), never reset anywhere.
;       The frozen tribute-phase credit reads a FRESH LOCAL `tribute_amount`
;       (computed this tick, inside the SAME loop iteration the
;       `comprador_wealth <= 0` `continue` gates), never the edge's own
;       persisted `value_flow` — so staleness is structurally impossible
;       there. The BSL PORT's `r04` reads the PUBLISHED edge attribute
;       instead (forced by the r03/r04 split + D116/D197's own
;       no-re-derive-from-mutated-wealth hazard, C1), which is exactly what
;       makes staleness POSSIBLE here: a comprador this tick's `r03` skips
;       (self inactive or `wealth <= 0`) leaves its TRIBUTE edge's
;       `tribute/value-flow` at whatever a PRIOR tick (or the world's own
;       seed) left there — a stale POSITIVE value would wrongly pass C1's
;       `> 0` gate without this reset. `r00`'s reset closes it, unconditionally,
;       for every TRIBUTE edge, before `r03`/`r04` ever run (byte order).
;       `r03_skips_a_non_positive_comprador`'s own seeded-positive-flow
;       fixture (restored to `+13`, review fix round 2) is this row's
;       executed proof: the edge reads `0.0` post-tick (the reset fired),
;       and the carrier credits ONLY the real, active comprador's tribute.
;       SIDE EFFECT: this reset makes `r04`'s own `it`-active conjunct
;       (its `for-each` guard's first clause) REACHABILITY-DEAD against any
;       value-based fixture — `tribute/value-flow` can no longer carry a
;       stale positive value on ANY edge, so `> 0` alone already excludes
;       an inactive target regardless of that conjunct's presence (the
;       SAME reachability-proof class D196 already uses for a dead frozen
;       clamp elsewhere in this file). Empirically confirmed, review fix
;       round 2: removing the conjunct, the FULL 25-test suite stays green
;       — kept anyway for defense-in-depth and symmetry with `r02`'s own
;       it-active conjunct (which is NOT dead — Phase 1 has no wealth-based
;       skip, so `r01`/`exploitation/value-flow` never goes stale the way
;       `r03`/`tribute/value-flow` did before this row's own fix).
;
; §8a. THE DUPLICATION LEDGER (D201) — this task's own row, REVISED
; mid-Task-2: `rent` is NOT independently duplicated after all. The
; original plan framed r02 as a second independent transcription of `rent`
; (D201's "no defexpr/macro" reasoning still stands as WHY a true duplicate
; would be needed if r02 had to compute it fresh) — but D116/D197 ledger row
; 7 (above) makes that impossible to do CORRECTLY: r01 runs first and
; mutates the worker's own wealth, so an independent re-read is
; contaminated. r02 instead READS r01's published `exploitation/value-flow`
; (a producer/consumer relationship, not a duplicate). The ledger row below
; is revised to match what actually ships:
;
;   shared expression | producer | consumer | faithful-read row              | asserts
;   rent               | r01      | r02      | r01_and_r02_agree_on_the_rent   | Δ(rent-tribute-inflow) (r02's carrier credit) == the EXPLOITATION edge's own exploitation/value-flow (r01's exact `set`) bit-exact — NOT via Δ(core-bourgeoisie wealth), which suffers a real (measured) binary64 add/subtract rounding artifact through the 10000+rent round-trip
;   cut / tribute      | r03      | r04      | r03_and_r04_agree_on_the_tribute | Δ(rent-tribute-inflow) (r04's carrier credit) == the TRIBUTE edge's own tribute/value-flow (r03's exact `set`) bit-exact — SAME producer/consumer method as the row above, applied on landing (THIS task) rather than re-derived; world 1's own tribute (80.0) happens to round-trip Δ(core-bourgeoisie wealth) exactly too (Sterbenz-exact at this magnitude), but the edge-attribute comparison is the row's METHOD regardless, not a choice made because wealth would fail here
;
; (The remaining two duplication-ledger rows — super-wage-bonus
; r05/r06/r07, total-wages r06/r07 — land with their own rules, Tasks 5/6,
; and MUST apply this SAME producer/consumer correction — see D116/D197
; ledger row 7's own forward note.)
;
; `defconst` TABLE — every value from `src/babylon/data/defines.yaml`'s
; `economy:`/`timescale:` sections (already declared in this pack's own
; scenario, `content/scenarios/imperial-rent-conformance.bscn`, Task 1 —
; `defconst` is a `.bscn`-only top-level form, `bsl-language.rst:650-652`;
; this table is DOCUMENTARY, restating what every scenario loading this
; pack must declare, with its `defines.yaml` citation). Mint no new
; coefficient. Rows marked "(r00-r02)" are the FIVE Task 2 rules actually
; read via `:const`; "(r03-r04)" is THIS task's own one; the rest are
; reserved for Tasks 5-7:
;
;   qname                                      | value  | source              | read by
;   economy/extraction-efficiency               | 0.8    | defines.yaml:71     | r01, r02 (r00-r02)
;   economy/comprador-cut                        | 0.9    | :72                 | r03 (r03-r04)
;   economy/super-wage-rate                      | 0.2    | :74 (the SEED for   | r05 (Task 5) — the
;                                                 |        | rent-wage-rate;     | live value is the
;                                                 |        | the live value is   | carrier field
;                                                 |        | the carrier field)  |
;   economy/superwage-multiplier                 | 1.0    | :75                 | r06 (Task 5)
;   economy/superwage-ppp-impact                 | 0.5    | :76                 | r06 (Task 5)
;   economy/initial-rent-pool                    | 100.0  | :77                 | r08 (Task 7)
;   economy/pool-high-threshold                  | 0.7    | :78                 | r08 (Task 7)
;   economy/pool-low-threshold                   | 0.3    | :79                 | r08 (Task 7)
;   economy/pool-critical-threshold               | 0.1    | :80                 | r08 (Task 7)
;   economy/min-wage-rate                         | 0.05   | :81                 | r08 (Task 7)
;   economy/max-wage-rate                         | 0.35   | :82                 | r08 (Task 7)
;   economy/negligible-rent                       | 0.01   | :86                 | r01 (r00-r02), r05 (Task 5)
;   economy/trpf-coefficient                      | 0.0005 | :90                 | r01, r02 (r00-r02)
;   economy/rent-pool-decay                       | 0.002  | :91                 | r09 (Task 7)
;   economy/bribery-wage-delta                    | 0.05   | :92                 | r08 (Task 7)
;   economy/austerity-wage-delta                  | 0.05   | :93 (POSITIVE       | r08 (Task 7) — sign
;                                                 |        | transcription of a  | applied via `sub` at
;                                                 |        | frozen -0.05; no    | the use site
;                                                 |        | fractional literal
;                                                 |        | spells negative)    |
;   economy/iron-fist-repression-delta            | 0.1    | :94                 | r08 (Task 7)
;   economy/crisis-wage-delta                     | 0.15   | :95 (POSITIVE, same | r08 (Task 7) — same
;                                                 |        | as austerity above) | sign handling
;   economy/crisis-repression-delta               | 0.2    | :96                 | r08 (Task 7)
;   economy/bribery-tension-threshold             | 0.7    | :97 (the SHIPPED    | r08 (Task 7) — the
;                                                 |        | value; the module's | code governs, D189(c)
;                                                 |        | own 0.3 default/
;                                                 |        | docstring is drift)
;   economy/iron-fist-tension-threshold           | 0.5    | :98                 | r08 (Task 7)
;   economy/trpf-efficiency-floor                 | 0.1    | :99                 | r01, r02 (r00-r02)
;   timescale/weeks-per-year                      | 52     | :374                | r01, r02 (r00-r02)
;
; Class fields REUSED (not re-minted), per §8's own roster: `social-class/
; wealth` (real extensive), `social-class/active` (int intensive — no
; `bool` on the `.bscn` seed dialect), `social-class/role` (enum
; SocialRole), `social-class/production-value` (real extensive, read by
; r06 only, Task 5), `social-class/revolutionary` (probability intensive —
; B7 STRUCK, the frozen `class_consciousness_from_node` accessor's own
; re-point to this ALREADY-DECLARED field, dossier CORRECTIONS item 1 — NOT
; a net-new `social-class/class-consciousness` field). Edge attrs: `wages/
; value-flow` (real intensive, reused, written first by r06, Task 5),
; `exploitation/value-flow` (real intensive, net-new, Task 2's r01 is its
; first content-adjacent WRITE — `update-edge`'s first content-adjacent use
; of any kind landed as Task 1's throwaway spike, deleted; Task 2 is the
; first LANDED, permanent one), `tribute/value-flow` (real intensive,
; net-new, THIS task's r03 is its first WRITE — the same class of first as
; `exploitation/value-flow`'s Task 2 landing). Carrier: the `institution/
; rent-*` roster of plan §3.1 (declared Task 1) plus B8's two unprefixed
; latch fields
; (`institution/superwage-crisis-known`/`-tick`, Task 5).

(rule imperial-rent/r00-tick-reset
  :role mechanic
  :evidence derived
  :material-basis "The per-tick re-creation of the frozen `tick_context` dict (economic.py:59-66): `tribute_inflow=0.0`. `rent-wages-outflow` stays unreset (D199); `rent-pool`/`rent-carrier`/the two B8 latches untouched. Domain EXPLICIT `NodeType/INSTITUTION` (`:field institution/rent-carrier` binding, TICK-time). Review fix round 2 (N1, PORT-ARTIFACT, NO frozen analog — D-row below): also zeroes EVERY TRIBUTE edge's `tribute/value-flow` — the frozen engine never needs this (its credit reads a FRESH local `tribute_amount`, never the edge attribute), but r04's own C1 fix reads the PUBLISHED edge value, so a stale prior-tick positive flow on an edge THIS tick's r03 skips (self inactive/wealth<=0) would wrongly pass r04's `> 0` gate without this reset. Both writes idempotent under repeated/multi-subject firing."
  :fuel 23
  (domain NodeType/INSTITUTION)
  (bindings
    (binding carrier :field institution/rent-carrier))
  (when #t)
  (effects
    (update-node
      (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
      institution/rent-tribute-inflow
      (set 0))
    (for-each (edges EdgeType/TRIBUTE)
      (update-edge it tribute/value-flow (set 0)))))

(rule imperial-rent/r01-extraction
  :role mechanic
  :evidence derived
  :material-basis "Phase 1 — Extraction (economic.py:239-345). eff = (extraction-efficiency/weeks-per-year) * max(trpf-efficiency-floor, 1-trpf-coefficient*tick) (:253-262, max as an `if`, §3.4). Per EXPLOITATION edge (source=worker=self, target=exploiter=it): rent = min(eff*wealth*(1-consciousness), wealth) (:289,292); consciousness reads social-class/revolutionary (B7 STRUCK, dossier CORRECTIONS 1). Writes: self wealth (sub rent) — D196's AST: economic.py:295's max(0,·) clamp NOT transcribed (dead in every frozen-reachable world, rent<=wealth always; r01_never_drives_a_single_edge_worker_negative is the converse witness; N>=2-edge negatives are D184(a), world 8, Task 6). it wealth (add rent) (:297); exploitation/value-flow (set rent) (:300-302, D182's self-anchored push). Both self/it active gated (:276,280). Emits SURPLUS_EXTRACTION when rent>negligible-rent (:332-345): source/target NodeRefs (BLOCKER-5b), amount — no mechanism key (BLOCKER-5, D188). Full prose: this file's header."
  :fuel 104
  (bindings
    (binding active :field social-class/active)
    (binding wealth :field social-class/wealth)
    (binding revolutionary :field social-class/revolutionary :optional :default 0.0p)
    (binding extraction-efficiency :const economy/extraction-efficiency)
    (binding weeks-per-year :const timescale/weeks-per-year)
    (binding trpf-coefficient :const economy/trpf-coefficient)
    (binding trpf-efficiency-floor :const economy/trpf-efficiency-floor)
    (binding negligible-rent :const economy/negligible-rent)
    (binding tick :tick)
    (binding base-eff :expr (/ extraction-efficiency weeks-per-year))
    (binding trpf-unclamped :expr (- (- 1 0c) (* trpf-coefficient tick)))
    (binding trpf-mult :expr (if (> trpf-unclamped trpf-efficiency-floor) trpf-unclamped trpf-efficiency-floor))
    (binding eff :expr (* base-eff trpf-mult))
    (binding one-minus-consciousness :expr (- (- 1 0c) revolutionary))
    (binding rent-uncapped :expr (* (* eff wealth) one-minus-consciousness))
    (binding rent :expr (if (< rent-uncapped wealth) rent-uncapped wealth)))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/EXPLOITATION :out NodeType/SOCIAL_CLASS)
      (guard (= (field-of it social-class/active) 1)
        (update-node self social-class/wealth (sub rent))
        (update-node it social-class/wealth (add rent))
        (update-edge (edge-between EdgeType/EXPLOITATION self it) exploitation/value-flow (set rent))
        (guard (> rent negligible-rent)
          (emit EventType/SURPLUS_EXTRACTION
            (source self)
            (target it)
            (amount rent)))))))

(rule imperial-rent/r02-extraction-credit
  :role mechanic
  :evidence derived
  :material-basis "Phase 1's CORE_BOURGEOISIE credit (economic.py:324-329): rent ALSO accumulates into tick_context['tribute_inflow']/['current_pool']. CORRECTED DESIGN (found in Task 2's own gate, not an independent re-derivation): reads r01's SAME-TICK `exploitation/value-flow` write via `(field-of (edge-between ...))` (D116/D197 ledger row 7, NEW) rather than re-deriving eff/wealth/consciousness — r01 runs FIRST (byte order) and MUTATES self's own wealth, so a fresh `:field social-class/wealth` re-read here would silently read POST-r01 wealth, producing a WRONG (smaller) rent; measured, not theorized (r02_credits_only_a_core_bourgeoisie_target caught it red before this fix). r01_and_r02_agree_on_the_rent (§8a) now asserts the READ is faithful, not that two independent formulas coincide. Both carrier writes score the D198 DISCRIMINATOR, never self, never a constant score (unlike B8's r05 exception). Full prose: this file's header, D184/D201 addendum."
  :fuel 75
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/EXPLOITATION :out NodeType/SOCIAL_CLASS)
      (guard (and (= (field-of it social-class/active) 1)
                  (= (field-of it social-class/role) SocialRole/CORE_BOURGEOISIE))
        (update-node
          (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
          institution/rent-tribute-inflow
          (add (field-of (edge-between EdgeType/EXPLOITATION self it) exploitation/value-flow)))
        (update-node
          (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
          institution/rent-pool
          (add (field-of (edge-between EdgeType/EXPLOITATION self it) exploitation/value-flow)))))))

(rule imperial-rent/r03-tribute
  :role mechanic
  :evidence derived
  :material-basis "Phase 2 — Tribute (economic.py:347-400). Per TRIBUTE edge (source=comprador=self, target=recipient=it): cut = wealth * comprador-cut (:381), tribute = wealth - cut (:382) — BOTH rule-scoped, computed ONCE from self's pre-state wealth, independent of `it` (D200/D184(b) — world 10 measures the divergence vs. the frozen engine's own per-edge SOURCE re-read, :375). Writes: self wealth (set cut) — the §1.6-c OVERWRITE, `source.wealth = cut_amount` (:385), a `set` not `sub` (D189(b)) — N TRIBUTE edges collect N identical `(set cut)` writes; D200: accepted, idempotent here (same value). it wealth (add tribute) (:387); tribute/value-flow (set tribute) (:390-392, D182's self-anchored push). Both self/it active gated (:367,371). Self also gated `wealth > 0` (:377-378, strict). No emit (r03_emits_nothing)."
  :fuel 64
  (bindings
    (binding active :field social-class/active)
    (binding wealth :field social-class/wealth)
    (binding comprador-cut :const economy/comprador-cut)
    (binding cut :expr (* wealth comprador-cut))
    (binding tribute :expr (- wealth cut)))
  (when (and (= active 1) (> wealth 0)))
  (effects
    (for-each (neighbors self EdgeType/TRIBUTE :out NodeType/SOCIAL_CLASS)
      (guard (= (field-of it social-class/active) 1)
        (update-node self social-class/wealth (set cut))
        (update-node it social-class/wealth (add tribute))
        (update-edge (edge-between EdgeType/TRIBUTE self it) tribute/value-flow (set tribute))))))

(rule imperial-rent/r04-tribute-credit
  :role mechanic
  :evidence derived
  :material-basis "Phase 2's CORE_BOURGEOISIE credit (economic.py:398-400): tribute accumulates into tick_context['tribute_inflow']/['current_pool'] when the recipient's role is CORE_BOURGEOISIE. Mirrors r02's corrected design: reads r03's SAME-TICK `tribute/value-flow` via `(field-of (edge-between ...))`, NOT a fresh comprador wealth re-read (r03 OVERWRITES it first, D116/D197 row 7). Review fix round 1 (C1): the gate is PER-EDGE `tribute/value-flow > 0`, NOT a self-level `wealth > 0` re-read — self's wealth is ALREADY OVERWRITTEN by r03's own `(set cut)` by the time this rule runs, so a self-level wealth check reads the post-transfer cut, not the frozen `:377-378` pre-transfer check it must mirror (wrong under comprador-cut=0 and under a comprador that is itself a TRIBUTE target). A positive published tribute IS the frozen pre-transfer check having passed AND produced a transfer. Both writes score the D198 discriminator."
  :fuel 85
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/TRIBUTE :out NodeType/SOCIAL_CLASS)
      (guard (and (= (field-of it social-class/active) 1)
                  (= (field-of it social-class/role) SocialRole/CORE_BOURGEOISIE)
                  (> (field-of (edge-between EdgeType/TRIBUTE self it) tribute/value-flow) 0))
        (update-node
          (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
          institution/rent-tribute-inflow
          (add (field-of (edge-between EdgeType/TRIBUTE self it) tribute/value-flow)))
        (update-node
          (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))
          institution/rent-pool
          (add (field-of (edge-between EdgeType/TRIBUTE self it) tribute/value-flow)))))))
