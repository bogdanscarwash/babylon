; DecompositionSystem (Material Base @11.0) — the terminal crisis that splits
; the Labor Aristocracy into carceral enforcers and internal proletariat when
; super-wages can no longer be paid. Frozen source:
; src/babylon/engine/systems/decomposition.py (370 lines, one step()).
; Port posture: ADR183 (structure/ordering contract, not a byte oracle) —
; conformance expecteds are measured from THIS engine, never copied from the
; frozen mirror's printed floats (decomposition_conformance.py's own header
; makes the same point for its persistent_data dump).
;
; TASK 2 SHIP: `decomposition/p01-la-census` + `decomposition/p02-superwage-
; warning` — the per-node LA census publication and the early-warning latch.
; TASK 3 SHIP: `decomposition/p03-trigger` — the carrier-side trigger and the
; frozen transfer-amount arithmetic. TASK 4 SHIP: `decomposition/p04-
; enforcer-intake` + `decomposition/p05-ip-intake` + `decomposition/p06-la-
; deactivate` — the two intake rules and the LA deactivation, closing Pack A.
;
; `(intrinsic floor :params (real) :returns int :cost 5)` is declared here
; (Pack A) — `p03-trigger` (Task 3) is the first caller (`(floor (*
; la-population enforcer-fraction))` etc); p01 and p02 never call it.
; CORRECTED (final review I1 — this header previously claimed the
; declaration "lives in exactly one file", which is FALSE):
; `territory.bsl:67` ALREADY declares a BYTE-IDENTICAL `(intrinsic floor
; :params (real) :returns int :cost 5)`, and
; `babylon-bsl/src/declarations.rs:1010-1017` (`parse_intrinsic_decls`)
; refuses a duplicate declaration BY NAME ONLY — `if
; decls.contains_key(&decl.name)` — with NO content comparison, so two
; BYTE-IDENTICAL declarations refuse just as hard as two conflicting ones.
; The REAL constraint: the `floor` intrinsic is declared PER-FILE (not
; deduplicated across files), the loader refuses a duplicate name
; unconditionally, and Territory @3.0 + Decomposition @11.0 are both
; Material Base systems — so ANY content set that co-loads
; `decomposition.bsl` with `territory.bsl` dies at load with
; `E-LOAD-001`, a landmine on the Checkpoint A path (all 13 Material Base
; systems loaded together). No content set does this today (`floor_
; intrinsic_e2e.rs:137-138`'s test proves the REFUSAL mechanism, not an
; exemption from it) — checked directly: `load_scenario_with_prelude`
; (the ONLY prelude mechanism in this estate, ADR209) operates on
; SCENARIO-side declarations (`defenum`/`deffield`/`defconst` via
; `EnumRegistry::declare`'s identical-recognition arm, `scenario.rs`) and
; has NO counterpart for RULE-file `intrinsic` declarations at all —
; `split_content`/`declarations.rs`'s intrinsic-parsing path is a
; completely separate, prelude-unaware load path. A dedup/prelude rule
; for rule-file intrinsics does not exist yet; until one lands, packs
; declaring `floor` must never co-load in one content set. Follow-up
; filed: **#646** (final review I1, Checkpoint A implication).
;
; D116 BYTE-ORDER MAP (docs/reference/bsl-language.rst) — rules run to
; completion in ascending rule-id byte order against the same mutable graph,
; so every rule below sees every earlier rule's same-tick writes. Every
; same-tick read across this pack is a DELIBERATE reliance on that order,
; production.bsl/consciousness.bsl-header style:
;
;   rule                  subject       reads                               writes
;   p01-la-census         SOCIAL_CLASS  role, active, population, wealth,   la-census-population,
;                                       subsistence-threshold, s-bio,       la-census-wealth,
;                                       s-class, :const                     la-approaching-flag,
;                                                                           la-dying-flag
;   p02-superwage-warning SOCIAL_CLASS  role, active,                       carrier superwage-
;                                       la-approaching-flag (p01, SAME     crisis-known/-tick
;                                       TICK), carrier superwage-crisis-
;                                       known
;   p03-trigger           INSTITUTION   folded la-census-* (p01, SAME       decomposition-fire-tick,
;                                       TICK via nodes-fold), carrier       -fired-known, -complete,
;                                       superwage-crisis-known/-tick        the four transfer amounts
;                                       (p02, SAME TICK)
;   p04-enforcer-intake   SOCIAL_CLASS  carrier decomposition-fire-tick,    population/wealth (add),
;                                       enforcer-pop-gain/-wealth-gain      active (set 1)
;                                       (p03, SAME TICK)
;   p05-ip-intake         SOCIAL_CLASS  carrier decomposition-fire-tick,    population/wealth (set),
;                                       ip-population/-wealth (p03, SAME    active (set 1)
;                                       TICK)
;   p06-la-deactivate     SOCIAL_CLASS  carrier decomposition-fire-tick,    active (set 0),
;                                       the four transfer amounts (p03,     CLASS_DECOMPOSITION emit
;                                       SAME TICK), self population/wealth
;                                       (pre-state — nothing earlier in
;                                       this pack writes them)
;
; D-RECORDS this pack transcribes (full register rows D166-D174 landed in
; Task 9, global-D-number cross-references added inline below per the
; final review's I5 finding; D165 alone landed in Task 5, per that row's
; own text — it lives in this file's p01/p03 :material-basis prose, not
; in the numbered list below). production.bsl/consciousness.bsl's own
; header-first convention followed here — the file is the record, the
; register catalogs it):
;   1. (global D166) THE CARRIER REFORMULATION — the frozen `persistent_data` dict
;      (`_superwage_crisis_tick`, `_decomposition_complete`,
;      `_class_decomposition_tick`, …) becomes `institution/*` fields on the
;      single `carceral-register` carrier (plan §2). Every `None`-sentinel
;      the frozen dict carries becomes a companion `*-known` int 0/1 flag
;      (III.11 loud-absence encoding) rather than a magic sentinel value,
;      since no `:optional`/`:default` route exists for a `.bscn`-seeded
;      field. Reads reach the carrier via `(field-of (select-max (nodes
;      NodeType/INSTITUTION) 1) institution/…)`; writes via `(update-node
;      (select-max (nodes NodeType/INSTITUTION) 1) institution/… (set …))`
;      — the D103/D104 accumulate-into-a-non-self-target lane.
;   2. (global D167) THE OMITTED `add-node` BRANCH — `_create_target_entity`/spec-071's
;      create-on-demand path (`decomposition.py:225-261`, `_ENFORCER_ID_
;      OFFSET`/`_INTERNAL_PROLETARIAT_ID_OFFSET`) is OMITTED entirely:
;      `add-node` is refused at content load (`DEFERRED_SHAPE_VERBS`,
;      `structural_verbs.rs`), so every conformance world pre-seeds its own
;      CARCERAL_ENFORCER/INTERNAL_PROLETARIAT targets instead (already
;      recorded as BLOCKER-1 in `decomposition-conformance.bscn`'s header);
;      p04/p05 read/write the pre-seeded nodes and never create one.
;   3. (global D168) THE OMITTED HISTORY READ — the frozen engine's `services.event_bus.
;      get_history()` scan for `SUPERWAGE_CRISIS` events
;      (`decomposition.py:164-175`), which recovers `_superwage_crisis_tick`
;      from event history on a tick where `persistent_data` alone lost it,
;      is OMITTED: BSL has no same-tick or cross-tick event-history query
;      (`bsl-language.rst`'s own gap item 3 — "the emitting rule also stamps
;      a field" is the prescribed re-modelling). The carrier's
;      `superwage-crisis-known`/`-tick` latch, written by p02 the same tick
;      it emits, is the sole source of truth — exactly the re-modelling the
;      language document itself names, not an invented shortcut.
;   4. (global D171 item 1) THE PAYLOAD FLATTENING — `SUPERWAGE_CRISIS`'s frozen payload carries
;      `payer_id` (a second NodeRef, always `CORE_BOURGEOISIE_ID`) and
;      `narrative_hint` (a string) that this port DROPS (item 5 below);
;      `CLASS_DECOMPOSITION`'s frozen payload nests `population_transferred`/
;      `wealth_transferred` as sub-dicts (`decomposition.py:352-359`) that
;      p06 flattens to four top-level numeric keys
;      (`population-transferred-to-enforcer`/`-to-proletariat`,
;      `wealth-transferred-to-enforcer`/`-to-proletariat`), since
;      `<payload-item>` values are flat `<expr>` (number/bool/enum-ref/
;      NodeRef) — no dict, no string.
;   5. (global D171 item 2) THE DROPPED NARRATIVE HINTS AND `trigger_event` — every
;      `narrative_hint` string (`decomposition.py:189-192`, `361-365`) AND
;      `CLASS_DECOMPOSITION`'s `trigger_event` string
;      (`decomposition.py:360`, always `"superwage_crisis"`) are OMITTED
;      from every emitted payload: `emit` carries no string payloads at all
;      (`Str` has no `<payload-item>` production, `bsl-language.rst`'s gap
;      item 4) — `trigger_event` is the same class of omission as
;      `narrative_hint`, not a separate divergence, since BSL's `p03`-`p06`
;      chain already makes the trigger unambiguous through the carrier's
;      own `fire-tick` latch (D-record 3, global D168).
;   6. D116 BYTE-ORDER RELIANCE — the map above; p02 reads p01's
;      `la-approaching-flag` write from THIS tick, p03 reads p01's/p02's
;      SAME-TICK carrier writes, and p04-p06 each read p03's this-tick
;      carrier write in turn.
;   7. (global D173 item 4) THE BARE-`2` LITERAL — `carceral/approaching-consumption-multiple`
;      (defconst value `2`) has NO `CarceralDefines` backing anywhere in the
;      frozen source: it is a bare `2 * consumption` literal at
;      `decomposition.py:155` (`_APPROACHING_CONSUMPTION_MULTIPLE`-shaped,
;      never named as a define). Transcription note, not an escalation
;      (already recorded in the Global Constraints table and the scenario's
;      own header). Task 2 mutation evidence (Step 6): flipping this
;      scenario constant 2 -> 1 (narrowing the approaching-bound from 520 to
;      510) flips NO Task 2 test — `la-dying`'s wealth (400) sits below BOTH
;      bounds, so the multiplier is load-bearing content this fixture does
;      not witness. Predicted by the plan itself; DISCHARGED by Task 4's
;      delay scenario (`decomposition-delay-conformance.bscn`'s
;      `la-approaching`, wealth 515 strictly between subsistence + 1x
;      consumption (510) and subsistence + 2x consumption (520)) — flipping
;      2 -> 1 there makes `la-approaching-flag` stay 0 forever, so
;      SUPERWAGE_CRISIS never fires and the delay path never reaches
;      CLASS_DECOMPOSITION, flipping
;      `the_delay_path_emits_the_warning_at_tick_1_and_decomposes_at_tick_53`
;      red.
;   8. (global D173 item 1) DOCSTRING DRIFT — the module docstring (`decomposition.py:4-5`) claims
;      "30% of Labor Aristocracy becomes CARCERAL_ENFORCER" / "70% falls into
;      INTERNAL_PROLETARIAT", but the CODE reads the split from
;      `CarceralDefines` (`enforcer_fraction = 0.15`, `proletariat_fraction =
;      0.85`, `defines.yaml:295-296`) — 15%/85%, not 30%/70%. ADR183: the
;      code is the port's oracle, not its own docstring; transcribed as
;      15%/85% exactly, the drift noted here rather than "corrected" in
;      either direction.
;   9. (global D173 item 3) NON-CONSERVATION — `enforcer_pop_gain = int(la_population *
;      enforcer_fraction)` and `proletariat_pop = int(la_population *
;      proletariat_fraction)` (`decomposition.py:298-299`) truncate
;      INDEPENDENTLY; for an `la_population` not evenly split by 0.15/0.85
;      their sum can be STRICTLY LESS than `la_population` (population is
;      lost, never gained, since both operations floor). A real defect in
;      the frozen engine, transcribed verbatim by Task 3's `p03-trigger`
;      (which computes both amounts with the same two independent floors).
;      This train's own fixture (`la-dying`, population 1000) happens to
;      split exactly (150 + 850 = 1000) and does not witness the loss —
;      noted here as inherited-defect transcription, not proven by this
;      world.
;
; TASK 4 DISCHARGED MUTATION OBLIGATIONS (carried from Tasks 2/3 reviews,
; not separate D-records — both gates were already correctly transcribed,
; only unwitnessed by the fixtures that existed at the time):
;   10. `p03-trigger`'s `(> la-population 0)` conjunct (`decomposition.py:
;       290-291`'s early return): Task 3's own mutation evidence found NO
;       test flips `>` -> `>=` because that fixture's folded la-population
;       is always 1000, never zero. DISCHARGED by a dedicated inline
;       zero-population world (`decomposition_conformance.rs`'s
;       `p03_gate_blocks_zero_population_la_even_when_the_delay_has_elapsed`)
;       — a single LA node seeded with population 0 and the carrier's
;       `superwage-crisis-known`/`-tick` SEEDED directly (bypassing p02
;       entirely, since a population-0 LA's own approaching/dying flags can
;       never go non-zero — p01's flags read `population` verbatim, not
;       through the census fold), so `delay-elapsed-fire` is true from tick
;       1 while folded la-population stays 0 for the life of the world —
;       isolated from every other fixture's non-zero LA population (the
;       fold is global over ALL SOCIAL_CLASS nodes, so this could not share
;       a world with the delay scenario's own `la-approaching`).
;   11. `p01-la-census`'s `(= active 1)` conjunct: no existing fixture
;       (Tasks 2-3) seeded an INACTIVE Labor Aristocracy node, so nothing
;       proved the active gate load-bearing on its own (only the role gate
;       was mutation-tested, Task 2's commit). DISCHARGED by
;       `decomposition-delay-conformance.bscn`'s `la-inactive` (active 0,
;       wealth 1 < its own subsistence 500 — would flag dying if the active
;       conjunct were dropped) — mirrors `production-conformance.bscn:161-
;       166`'s `worker-la-idle` vector.
;
; p01's OWN transcription note (not a divergence, so no numbered row above):
; the frozen `la_approaching_death`/`la_about_to_die` gates both carry a
; `la_pop > 0` conjunct alongside the wealth comparison
; (`decomposition.py:155,158`) — preserved verbatim in p01's nested `if`
; below, even though every fixture in this train's conformance world seeds a
; strictly positive LA population.

(intrinsic floor :params (real) :returns int :cost 5)

(rule decomposition/p01-la-census
  :role mechanic
  :evidence derived
  :material-basis "Per-node LA census publication, reformulating the frozen engine's single `_find_entity_by_role(graph, LABOR_ARISTOCRACY)` graph-scope lookup (decomposition.py:143-159) as a per-node gated write every SOCIAL_CLASS subject performs (plan §2's fold-body compound-expression restriction: the role/active filter cannot live in p03's carrier-side fold, so it lives here instead, production.bsl's D138 precedent). No `when` clause: every subject fires and a non-LA (or inactive LA) writes zero to all four fields, keeping the census fresh every tick rather than stale (the D127 hash-neutral idiom, territory.bsl's own row)."
  :fuel 73
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding subsistence-threshold :field social-class/subsistence-threshold)
    (binding s-bio :field social-class/s-bio)
    (binding s-class :field social-class/s-class)
    (binding approaching-multiple :const carceral/approaching-consumption-multiple)
    (binding consumption :expr (+ s-bio s-class))
    (binding approaching-bound :expr (+ subsistence-threshold (* approaching-multiple consumption)))
    (binding census-population :expr (if (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1))
                                         population
                                         0))
    (binding census-wealth :expr (if (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1))
                                     wealth
                                     (- 0 0c)))
    (binding approaching-flag :expr (if (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1))
                                        (if (and (< wealth approaching-bound) (> population 0)) 1 0)
                                        0))
    (binding dying-flag :expr (if (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1))
                                  (if (and (< wealth subsistence-threshold) (> population 0)) 1 0)
                                  0)))
  (when #t)
  (effects
    (update-node self social-class/la-census-population (set census-population))
    (update-node self social-class/la-census-wealth (set census-wealth))
    (update-node self social-class/la-approaching-flag (set approaching-flag))
    (update-node self social-class/la-dying-flag (set dying-flag))))

(rule decomposition/p02-superwage-warning
  :role mechanic
  :evidence derived
  :material-basis "The early warning: when the active LA is approaching subsistence and no crisis has been latched yet, emit SUPERWAGE_CRISIS and latch the carrier so this fires at most once (decomposition.py:179-197). Reads p01's la-approaching-flag from THIS tick (D116). Transcribed order: emit first, then the latch (:180-197)."
  :fuel 33
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding approaching-flag :field social-class/la-approaching-flag)
    (binding crisis-known :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                          institution/superwage-crisis-known))
    (binding tick :tick))
  (when (and (= role SocialRole/LABOR_ARISTOCRACY)
             (= active 1)
             (= approaching-flag 1)
             (= crisis-known 0)))
  (effects
    (emit EventType/SUPERWAGE_CRISIS
      (receiver self)
      (desired-wages 0.0c)
      (available-pool 0.0c))
    (update-node (select-max (nodes NodeType/INSTITUTION) 1)
                 institution/superwage-crisis-known
                 (set 1))
    (update-node (select-max (nodes NodeType/INSTITUTION) 1)
                 institution/superwage-crisis-tick
                 (set tick))))

(rule decomposition/p03-trigger
  :role mechanic
  :evidence derived
  :material-basis "The carrier trigger + frozen split (decomposition.py:150-208, 296-299). Folds p01's four SAME-TICK census fields onto the carrier unconditionally (D127 idiom on the carrier side). should-decompose = la_about_to_die (now la-dying-count > 0) OR (superwage_tick not None (the known-flag, III.11) AND tick >= superwage_tick + delay). Gated on decomposition-complete == 0 (:129-130) AND la-population > 0 (_execute_decomposition's early return, :290-291). Writes fire-tick/-fired-known/-complete and the four amounts: enforcer_pop_gain = int(pop * enforcer_fraction), proletariat_pop = int(pop * proletariat_fraction) — each floors INDEPENDENTLY (D-record 9/D173 item 3's non-conservation, transcribed verbatim); the two wealth amounts are NOT int()-demoted. fire-tick == tick is the idiom p04-p06 key off below; this rule's own complete gate makes its OWN re-fire idempotent."
  :fuel 177
  (bindings
    (binding decomposition-complete :field institution/decomposition-complete)
    (binding superwage-crisis-known :field institution/superwage-crisis-known)
    (binding superwage-crisis-tick :field institution/superwage-crisis-tick)
    (binding tick :tick)
    (binding decomposition-delay :const carceral/decomposition-delay)
    (binding enforcer-fraction :const carceral/enforcer-fraction)
    (binding proletariat-fraction :const carceral/proletariat-fraction)
    (binding la-population :expr (fold sum (nodes NodeType/SOCIAL_CLASS)
                                       (field-of it social-class/la-census-population)))
    (binding la-wealth :expr (fold sum (nodes NodeType/SOCIAL_CLASS)
                                   (field-of it social-class/la-census-wealth)))
    (binding la-approaching-count :expr (fold sum (nodes NodeType/SOCIAL_CLASS)
                                              (field-of it social-class/la-approaching-flag)))
    (binding la-dying-count :expr (fold sum (nodes NodeType/SOCIAL_CLASS)
                                        (field-of it social-class/la-dying-flag)))
    (binding fallback-fire :expr (> la-dying-count 0))
    (binding delay-elapsed-fire :expr (and (= superwage-crisis-known 1)
                                            (>= tick (+ superwage-crisis-tick decomposition-delay))))
    (binding should-fire :expr (and (or fallback-fire delay-elapsed-fire)
                                    (= decomposition-complete 0)
                                    (> la-population 0)))
    (binding enforcer-pop-gain :expr (floor (* la-population enforcer-fraction)))
    (binding ip-population :expr (floor (* la-population proletariat-fraction)))
    (binding enforcer-wealth-gain :expr (* la-wealth enforcer-fraction))
    (binding ip-wealth :expr (* la-wealth proletariat-fraction)))
  (when #t)
  (effects
    (update-node self institution/la-population (set la-population))
    (update-node self institution/la-wealth (set la-wealth))
    (update-node self institution/la-approaching-count (set la-approaching-count))
    (update-node self institution/la-dying-count (set la-dying-count))
    (guard should-fire
      (update-node self institution/decomposition-fire-tick (set tick))
      (update-node self institution/decomposition-fired-known (set 1))
      (update-node self institution/decomposition-complete (set 1))
      (update-node self institution/enforcer-pop-gain (set enforcer-pop-gain))
      (update-node self institution/ip-population (set ip-population))
      (update-node self institution/enforcer-wealth-gain (set enforcer-wealth-gain))
      (update-node self institution/ip-wealth (set ip-wealth)))))

(rule decomposition/p04-enforcer-intake
  :role mechanic
  :evidence derived
  :material-basis "The carceral-enforcer intake — ADDITIVE (decomposition.py:323-332's `graph.update_node(enforcer_id, population=current_pop + enforcer_pop_gain, wealth=current_wealth + enforcer_wealth_gain, active=True)`): reads the pre-seeded CARCERAL_ENFORCER's CURRENT population/wealth off self (nothing earlier in this pack writes them) and ADDS p03's SAME-TICK carrier enforcer-pop-gain/enforcer-wealth-gain (D116) — the field names say \"gain\" for exactly this reason, unlike p05's OVERWRITE. Gated on role == CARCERAL_ENFORCER and carrier decomposition-fire-tick == tick, the idiom p03's own header names as what this rule keys off."
  :fuel 36
  (bindings
    (binding role :field social-class/role)
    (binding tick :tick)
    (binding fire-tick :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                       institution/decomposition-fire-tick))
    (binding enforcer-pop-gain :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                               institution/enforcer-pop-gain))
    (binding enforcer-wealth-gain :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                                  institution/enforcer-wealth-gain)))
  (when (and (= role SocialRole/CARCERAL_ENFORCER) (= fire-tick tick)))
  (effects
    (update-node self social-class/population (add enforcer-pop-gain))
    (update-node self social-class/wealth (add enforcer-wealth-gain))
    (update-node self social-class/active (set 1))))

(rule decomposition/p05-ip-intake
  :role mechanic
  :evidence derived
  :material-basis "The internal-proletariat intake — OVERWRITE (decomposition.py:335-336's `graph.update_node(ip_id, population=proletariat_pop, wealth=proletariat_wealth, active=True)`): population/wealth are SET to p03's SAME-TICK carrier ip-population/ip-wealth (D116), REPLACING the pre-seeded baseline rather than accumulating onto it — unlike p04's additive enforcer write, the frozen code never reads the internal proletariat's CURRENT population/wealth at all. Gated on role == INTERNAL_PROLETARIAT and carrier decomposition-fire-tick == tick."
  :fuel 36
  (bindings
    (binding role :field social-class/role)
    (binding tick :tick)
    (binding fire-tick :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                       institution/decomposition-fire-tick))
    (binding ip-population :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                           institution/ip-population))
    (binding ip-wealth :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                       institution/ip-wealth)))
  (when (and (= role SocialRole/INTERNAL_PROLETARIAT) (= fire-tick tick)))
  (effects
    (update-node self social-class/population (set ip-population))
    (update-node self social-class/wealth (set ip-wealth))
    (update-node self social-class/active (set 1))))

(rule decomposition/p06-la-deactivate
  :role mechanic
  :evidence derived
  :material-basis "Deactivates the decomposed LA — active SET to 0 ONLY, population/wealth left UNTOUCHED (decomposition.py:339's `graph.update_node(la_id, active=False)`, no population/wealth keys at all — the non-conservation vector: the frozen engine never zeroes the source class's own numbers, it only flips the active latch). Emits CLASS_DECOMPOSITION with the flattened payload (D-record 4/D171 item 1: the two frozen nested dicts population_transferred/wealth_transferred become four flat keys; D-record 5/D171 item 2: narrative_hint and trigger_event, both strings, are dropped — emit carries no string payloads at all). Reads p03's SAME-TICK carrier fire-tick and the four transfer amounts (D116); population/wealth read off self are PRE-STATE (nothing earlier in this pack writes LA's own population/wealth). Gated on role == LABOR_ARISTOCRACY, active == 1 (only the currently-active LA decomposes), and carrier decomposition-fire-tick == tick."
  :fuel 46
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding tick :tick)
    (binding fire-tick :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                       institution/decomposition-fire-tick))
    (binding enforcer-fraction :const carceral/enforcer-fraction)
    (binding proletariat-fraction :const carceral/proletariat-fraction)
    (binding enforcer-pop-gain :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                               institution/enforcer-pop-gain))
    (binding ip-population :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                           institution/ip-population))
    (binding enforcer-wealth-gain :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                                  institution/enforcer-wealth-gain))
    (binding ip-wealth :expr (field-of (select-max (nodes NodeType/INSTITUTION) 1)
                                       institution/ip-wealth)))
  (when (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1) (= fire-tick tick)))
  (effects
    (update-node self social-class/active (set 0))
    (emit EventType/CLASS_DECOMPOSITION
      (source-class self)
      (source-population population)
      (source-wealth wealth)
      (enforcer-fraction enforcer-fraction)
      (proletariat-fraction proletariat-fraction)
      (population-transferred-to-enforcer enforcer-pop-gain)
      (population-transferred-to-proletariat ip-population)
      (wealth-transferred-to-enforcer enforcer-wealth-gain)
      (wealth-transferred-to-proletariat ip-wealth))))
