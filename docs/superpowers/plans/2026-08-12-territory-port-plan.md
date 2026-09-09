# Territory Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transcribe the frozen `TerritorySystem` (`src/babylon/engine/systems/territory.py`, 378 lines, four phases) into BSL content — one `.bscn` conformance estate plus a `territory.bsl` rule pack — entering both Rust byte gates, port-as-is with declared deviations.

**Architecture:** Four sequential phase rules (`territory/p1-…` … `territory/p4-…`) whose rule-id byte order reproduces the frozen phase order under D16 execution; a prerequisite babylon-bsl slice discharges the D102 field-of-enum deferral (Territory is its chartered first consumer) and closes #551's enum-fold load gap in the same seam window. Conformance oracles are hand-built fixtures (no canonical scenario seeds the special types or ADJACENCY edges — inventory §5).

**Tech Stack:** Rust (babylon-bsl, babylon-tick), BSL content (`.bscn`/`.bsl`), deterministic table-driven tests only (no proptest).

**Evidence base (read all four before starting):**
- `reports/territory-port-phase1-inventory-2026-08-11.md` — the adjudicated inventory (its §6 BLOCKED rows are stale: the query lane landed, ADR197).
- `ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml` — what slice 1 serves.
- `rust/crates/babylon-tick/tests/query_lane_e2e.rs` — the four Territory-shaped vectors; the port's working templates.
- The scout dossier facts repeated inline below where load-bearing (each was verified at dev 4e0faf22).

## Global Constraints

- **Port-as-is (Director ruling):** frozen defects are transcribed and D-recorded, never silently repaired. The frozen engine is a structure/ordering contract, NOT a byte oracle (ADR183) — conformance expecteds are measured from the BSL engine and pinned; divergences from frozen float arithmetic are D-recorded, not chased.
- **No bool fields on the live pipeline:** `scenario.rs::load_deffield` (scenario.rs:918-930) has no `bool` arm and `update-node` has no Bool store path. The eviction latch is an `int` 0/1 flag (the `social-class/active` / `organization/active` precedent).
- **No Currency-typed attribute STORAGE:** a `currency` deffield row parses (scenario.rs:918-930 accepts the type name), but every path that would put a Currency VALUE into storage refuses — `defconst` Currency literals (scenario.rs:539-548) and node-attribute seeds (scenario.rs:1067) — so under the No-defaults contract a currency-declared field can never be read. Money-like fields therefore declare `int`; `rent_level` uses the scaled-bare-Int lane (metabolism.bsl D-1 `entropy-factor-x1e6` precedent).
- **No scalar min/max in the grammar:** clamps are nested `if` with the `(- 0 0c)` / `(- 1 0c)` Real-promotion idiom (dispossession.bsl:356-364).
- **No defaults:** an unwritten field errors on read (scenario.rs:56-58); every fixture seeds every field its rules read. The frozen `attrs.get(k, default)` affordance is not transcribable — D-record.
- **Enum declaration order is hash-bearing (ADR195):** `OperationalProfile (LOW_PROFILE HIGH_PROFILE)`, `TerritoryType (CORE PERIPHERY RESERVATION PENAL_COLONY CONCENTRATION_CAMP)` — the frozen Python declaration orders (`src/babylon/models/enums/territory.py:28-29,78-82`), verbatim.
- **Defines (defines.yaml:235-240 + :242):** heat_decay_rate 0.1, high_profile_heat_gain 0.15, eviction_heat_threshold 0.8, rent_spike_multiplier 1.5, displacement_rate 0.1, heat_spillover_rate 0.05, concentration_camp_decay_rate 0.2. Each becomes a `defconst` row citing its defines.yaml line (dispossession.bscn:29-42 comment style). Line :241 (`clarity_profile_coefficient`) sits inside the territory block but is read by NO TerritorySystem code path (verified) — it belongs to the fog/clarity estate and is deliberately NOT a defconst here; :243 (`displacement_priority_mode`) is the WS1-ledgered EXTRACTION-const (Global Constraints above).
- **displacement_mode is provably EXTRACTION** on every production path (inventory §5) — transcribed as the EXTRACTION priority order directly; the override machinery goes to the #502 WS1 ledger (Metabolism D-2 "provably uniform" reasoning).
- **Gates per commit:** `cargo test -p babylon-tick --test tick_goldens --locked` byte-identical (all pre-existing pins); full six-leg gate (`mise run rust:check` incl. `-D clippy::pedantic` on babylon-bsl) before each PR. Conventional commits with the Co-Authored-By trailer; never push to dev; ADR181 merge protocol.
- **Verification arc (Director ruling, recorded on #525):** the adversarial review runs BOTH lenses — substrate/overlay separation AND FIPS/data-integrity — layered on the standing composed-path baseline.
- **Machine safety:** cargo single-flight; no parallel test fan-out.

## Sequencing note (D116 / anchor positions)

`mod_anchors.rs:7-13`: anchor-order resolution is deferred to a future engine registry; today execution order is ascending rule-id byte order over the whole content set (`babylon-tick/src/lib.rs:229-242`, D16). The frozen system's phases are SEQUENTIAL by design (eviction reads this-tick post-Phase-1 heat; camp decay eats this-tick displaced arrivals), so the pack deliberately RELIES on current run-to-completion cross-rule semantics (D116's recorded divergence) with `p1 < p2 < p3 < p4` byte-ordered ids. **D-record obligation (Task 8):** when the Q14 repair train promotes same-position rules to shared pre-state, Territory's four phase rules must be recognized as four distinct positions (sub-positions of territory @2.0) — the D-record is the repair train's acceptance-criterion input. The `territory` system name is already registered (`babylon-tick/src/lib.rs:190-197`, added as an explicitly-not-the-port placeholder by the query-eval train); Task 4 updates that comment to record the port landing.

---

### Task 1: Discharge the D102 field-of-enum deferral (babylon-bsl)

**Files:**
- Modify: `rust/crates/babylon-bsl/src/typecheck.rs` (the `check_no_field_of_on_enum_field` refusal, ~:255-268, and its tests)
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs` (the `field-of` evaluation arm — render `Value::Enum` for enum-declared fields via the same `EnumRegistry` path `tick.rs::bind_field_value` uses, tick.rs:312-328)
- Test: same files' test modules

**Interfaces:**
- Consumes: `TypeEnv.fields` (`FieldDecl.ty == BslType::Enum(id)`), `EnumRegistry` member rendering.
- Produces: `(field-of <ref> <enum-declared-qname>)` evaluates to `Value::Enum { enum_type, member }`, legal in `=`/`!=` comparisons and `if` conditions; still REFUSED as a select-max/select-min score (D46 `E-TYPE-016` untouched) and in any arithmetic lane (`apply_arith` already refuses `Value::Enum`). Every later task's sink-selection body depends on this.

Why this is in-scope engineering, not an amendment: `field-of` exists, enum fields exist (ADR195), and the subject-side read path already renders enums (`bind_field_value`); this wires their intersection, minting nothing. D102 deferred it to its first consumer; `_find_sink_node` (territory.py:181-187) reads a NEIGHBOR's `territory_type` — Territory is that consumer.

- [ ] **Step 1: Red — write the failing tests.** In typecheck.rs tests: a rule whose guard compares `(field-of it <enum-field>)` to an enum literal LOADS (today it refuses — assert the new behavior, watch it fail). In evaluator.rs tests: evaluating `(= (field-of <ref> org/kind) OrgKind/STATE_APPARATUS)` over a two-node fixture returns the right Bool per node; a cross-enum-type comparison still errors ("compares only to the same enum type"); `(select-max … (field-of it <enum-field>))` still refuses `E-TYPE-016`; `(update-node x f (add (field-of it <enum-field>)))` still refuses arithmetic.
- [ ] **Step 2: Run the new tests; confirm the load-refusal ones fail for the right reason** (current D102 refusal message).
- [ ] **Step 3: Implement.** Remove/narrow `check_no_field_of_on_enum_field` so `field-of` on an enum-declared field passes typecheck AS `BslType::Enum(id)` (the typechecker must type it as the enum, not Real); in the evaluator's field-of arm, when the TypeEnv declares the field enum, render the stored ordinal to `Value::Enum` exactly as `bind_field_value` does (shared helper preferred — do not duplicate the ordinal→member rendering).
- [ ] **Step 4: Update D102's register row** in `docs/reference/bsl-language.rst` (the FIELD-OF DEFERRAL row): discharged by this train, first consumer named (Territory `_find_sink_node`), with the two surviving refusals (score position, arithmetic) restated.
- [ ] **Step 5: Mutation evidence.** Break the ordinal→member rendering (return the wrong member) → the evaluator test flips red; restore byte-identical.
- [ ] **Step 6: Gate + commit** `feat(bsl): discharge the D102 field-of-enum deferral — Territory is the chartered first consumer`.

### Task 2: Close #551 — the enum-fold load gap (same seam window)

**Files:**
- Modify: `rust/crates/babylon-bsl/src/typecheck.rs` (new load-time check + E-code)
- Test: typecheck.rs test module; update the CT4P legality-table test's declined-cells docstring to cite the closure.

**Interfaces:**
- Consumes: the fold kind-law path (typecheck.rs:87-142 region, post-CT4P `FoldOp` matches).
- Produces: `(fold <op> <query> <enum-typed body>)` refuses AT LOAD with a named E-code for all five ops (count included — its body is discarded but a declared-enum body is still a content error worth naming loudly; if the implementer finds count's body-erasure makes refusal wrong there, record the narrower verdict on #551 with reasoning).

- [ ] **Step 1: Verify first (the issue's own instruction):** trace the full `rule_pipeline::load_rule` path for `(fold sum <q> <:field-bound enum symbol>)` — if something already refuses it, close #551 with that citation, update the CT4P test doc, and skip to Task 3.
- [ ] **Step 2: Red — table test over all five FoldOps × an enum-declared body**, asserting a load-time refusal with the new code; watch it fail (today: silent pass, per the #551 probe).
- [ ] **Step 3: Implement the check** (recursing into fold bodies the way `check_no_field_of_on_enum_field` did — that recursion scaffold survives Task 1 for exactly this reuse), allocate the next free E-code per the D105 discipline ("resolve the number when the PR opens, never hard-code it" — check the register's current next-free).
- [ ] **Step 4: Mutation evidence** (disable the check → table test flips), gate, commit `fix(bsl): #551 — enum-typed fold bodies refuse at load`, comment + close #551 with the evidence.

### Task 3: The scenario ceremony — `territory-conformance.bscn`

**Files:**
- Create: `rust/crates/babylon-tick/content/scenarios/territory-conformance.bscn`
- Create: `rust/crates/babylon-tick/content/scenarios/territory_conformance.py` (frozen-engine mirror, sibling convention per `metabolism_conformance.py`)

**Interfaces:**
- Produces: the canonical org-style declaration block every phase rule loads against:

```scheme
(scenario territory/conformance
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY))
  (defvocabulary EdgeType (ADJACENCY TENANCY))
  (defenum OperationalProfile (LOW_PROFILE HIGH_PROFILE))
  (defenum TerritoryType (CORE PERIPHERY RESERVATION PENAL_COLONY CONCENTRATION_CAMP))
  (deffield territory/profile enum OperationalProfile)
  (deffield territory/territory-type enum TerritoryType)
  (deffield territory/heat intensity intensive)
  (deffield territory/rent-level-x1e6 int extensive)
  (deffield territory/under-eviction int extensive)
  (deffield territory/population int extensive)
  (deffield social-class/organization coefficient intensive)
  (defconst territory/heat-decay-rate 0.1c)              ; defines.yaml:235
  (defconst territory/high-profile-heat-gain 0.15c)      ; defines.yaml:236
  (defconst territory/eviction-heat-threshold 0.8c)      ; defines.yaml:237
  ; rent_spike_multiplier 1.5 is outside [0,1] — scaled bare-Int lane (D-1 class):
  (defconst territory/rent-spike-multiplier-x1e6 1500000) ; defines.yaml:238
  (defconst territory/displacement-rate 0.1c)            ; defines.yaml:239
  (defconst territory/heat-spillover-rate 0.05c)         ; defines.yaml:240
  (defconst territory/concentration-camp-decay-rate 0.2c) ; defines.yaml:242
  …nodes/edges per Task…)
```

Node population (12 nodes, covering every conformance case in one world): a HIGH_PROFILE CORE territory at sub-threshold heat; a LOW_PROFILE territory with nonzero heat (decay case); a HIGH_PROFILE territory whose gain crosses the 0.8 threshold this tick (latch-tick case); an already-latched territory (`under-eviction 1`) with population 1000 and rent 1e6; an ADJACENCY chain of three territories with distinct heats (spillover asymmetry); an isolated territory (exists-fallback); a source with two adjacent sinks — one PENAL_COLONY, one RESERVATION (EXTRACTION priority pick); a CONCENTRATION_CAMP with population 500 (decay); a PENAL_COLONY with two TENANCY-connected SOCIAL_CLASS nodes carrying `organization 0.6` and one unconnected class (untouched-law case). Every territory seeds ALL of: profile, territory-type, heat, rent-level-x1e6, under-eviction, population (No-defaults contract — see Global Constraints).

- [ ] **Step 1:** Write the `.bscn` with the header documenting: the enum orders as frozen-transcription (hash-bearing), the int-latch and scaled-rent D-record pointers, the seeded-defaults note.
- [ ] **Step 2:** Write a load smoke test (in the pack's test home, Task 4's file) asserting the scenario loads clean and node/edge censuses match; run it red (no rules yet — load-only), then green.
- [ ] **Step 3:** Write `territory_conformance.py` mirroring the fixture into the frozen engine (same graph, one `TerritorySystem.step()`), printing per-node post-state — the STRUCTURE oracle (which fields moved, latch set, transfer landed), explicitly NOT a byte oracle (header says so, ADR183).
- [ ] **Step 4:** Commit `test(tick): territory conformance scenario + frozen mirror (port train, fixture ceremony)`.

### Task 4: Phase 1 rule — `territory/p1-heat-dynamics`

**Files:**
- Create: `rust/crates/babylon-tick/content/rules/territory.bsl` (all four phase rules accrete here, one task at a time)
- Test: `rust/crates/babylon-tick/tests/territory_conformance.rs` (new; the pack's e2e home, `query_lane_e2e.rs` structure)
- Modify: `rust/crates/babylon-tick/src/lib.rs:190-197` (the `territory` registry comment — record that the port landed; entry itself unchanged)

**The rule (exact):**

```scheme
(rule territory/p1-heat-dynamics
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
    ; _write_clamped [0,1] (system_base.py:189) — nested-if idiom, floor then ceiling:
    (binding floored :expr (if (> raw 0) raw (- 0 0c)))
    (binding clamped :expr (if (< floored 1) floored (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/heat (set clamped))))
```

(If `(when #t)` is not a legal guard shape on current dev, use the landed-pack always-true idiom — check `vitality.bsl`'s guard and match it; subjects already self-select by carrying the bound fields.)

- [ ] **Step 1: Red** — e2e test loading the scenario + this rule via `run_once_into`: HIGH_PROFILE gains exactly `+0.15`; LOW_PROFILE decays ×0.9; the near-1.0 case clamps to 1.0; fired-count matches the seeded territory census. Watch it fail (no rule yet).
- [ ] **Step 2:** Write the rule; green.
- [ ] **Step 3: Mutation** — swap the if-branches (decay for HIGH_PROFILE) → test flips; restore byte-identical.
- [ ] **Step 4:** Gate + commit `feat(tick): territory p1 heat dynamics — the port train's first phase rule`.

### Task 5: Phase 2 rule — `territory/p2-eviction-pipeline`

**Files:** same rule pack + test file.

**Frozen semantics being transcribed (territory.py:196-267):** latch-tick has effects immediately (crossing the threshold sets the flag AND spikes/displaces the same tick); rent multiplies by 1.5 every latched tick; `displaced = int(pop × 0.1)` (trunc ≡ floor, non-negative); source loses displaced even when NO sink exists (population disappears); transfers to the ONE highest-priority adjacent sink accumulate across sources (apply-time adds); the frozen sink walk is DIRECTED (`edge.source_id == source`, territory.py:174) — transcribed as `:out`, and the frozen same-type multi-sink tiebreak (dict overwrite, enumeration-order last-wins) differs from BSL's D45 ascending-id first-wins — both D-recorded (Task 8).

**The rule (exact; `S` abbreviates the sink query, written out in full at each site — BSL has no local query naming):**

```scheme
(rule territory/p2-eviction-pipeline
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
    (binding displaced :expr (floor (* pop rate))))
  (when (or (= flag 1) (>= heat threshold)))
  (effects
    (update-node self territory/under-eviction (set 1))
    (update-node self territory/rent-level-x1e6
      (set (/ (* rent-x1e6 spike-x1e6) 1000000)))
    (update-node self territory/population (sub displaced))
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
```

Requires the `floor` intrinsic declared in the scenario: `(intrinsic floor :params (real) :returns int :cost 5)` (floor_intrinsic_e2e.rs:33 syntax) — add to Task 3's fixture in this task. Note the sink-priority scoring reads `field-of it territory/territory-type` — Task 1's discharge; the score itself is Int (nested-if), never the enum (D46 stands). The no-sink fallback writes `(add 0)` to self — numerically and hash-neutral; D-recorded. The frozen `if territory_type in priority_order` membership test = the three-way `or` in the exists body: a CORE/PERIPHERY-only neighborhood is NOT a sink (frozen: population disappears) — the exists guard must be sink-typed, not merely nonempty (this differs from the e2e template's bare `#t` exists — deliberate, frozen-faithful).

- [ ] **Step 1: Red** — e2e cases: latch-tick territory ends flag=1, rent 1.5e6, pop 900, PENAL_COLONY sink +100 (over RESERVATION — EXTRACTION priority); already-latched territory keeps compounding (rent 2.25e6 after two ticks — second-tick case via `TickSession::advance` twice); sub-threshold LOW territory untouched (guard false); no-sink latched territory loses population with nothing gaining it; camp/penal/reservation totals conserve except the no-sink disappearance (assert the conservation arithmetic explicitly).
- [ ] **Step 2:** Green.
- [ ] **Step 3: Mutations** — (a) change EXTRACTION priorities (RESERVATION 3) → sink-pick test flips; (b) drop the `(sub displaced)` write → conservation test flips. Restore byte-identical each.
- [ ] **Step 4:** Gate + commit `feat(tick): territory p2 eviction pipeline — latch, scaled-rent spike, carceral sink routing`.

### Task 6: Phase 3 rule — `territory/p3-spillover`

**The rule (exact):**

```scheme
(rule territory/p3-spillover
  :material-basis "heat is not contained by parcel lines: each ADJACENCY edge spills symmetrically, each endpoint receiving a fraction of the other's pre-phase heat (territory.py:269-316; pull-side fold reformulation is mathematically identical, ADR197/D103)"
  :fuel 512
  (bindings
    (binding heat :field territory/heat)
    (binding rate :const territory/heat-spillover-rate)
    (binding inflow :expr (if (exists (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY) #t)
                              (fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY)
                                    (* (field-of it territory/heat) rate))
                              (- 0 0c)))
    (binding raw :expr (+ heat inflow))
    ; frozen phase 3 clamps UPPER-ONLY (min(1.0, …), territory.py:315) — NOT _write_clamped;
    ; the two-clamp inconsistency is transcribed faithfully, D-recorded:
    (binding clamped :expr (if (< raw 1) raw (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/heat (set clamped))))
```

Fold body is the per-edge PRODUCT `heat × rate` (matching frozen per-edge accumulation shape), summed in ascending-id order — the summation-order difference vs frozen edge-enumeration order is a D-record, not a repair. Isolated territories take the `(- 0 0c)` branch: heat writes unchanged (hash-neutral) where frozen skips the write — D-recorded equivalence.

- [ ] **Step 1: Red** — e2e on the three-chain: middle node gains `rate × (heatₗ + heatᵣ)` products, ends gain one term each; all reads are pre-phase (seed the chain so a wrong post-state read produces a visibly different number — the query_lane_e2e.rs:104-112 discipline); the isolated territory's heat is byte-unmoved; a near-1.0 chain node clamps at exactly 1.0.
- [ ] **Step 2:** Green. **Step 3: Mutation** — drop the `rate` factor inside the fold body → flips; restore. **Step 4:** Gate + commit `feat(tick): territory p3 symmetric heat spillover — pull-side fold`.

### Task 7: Phase 4 rules — `territory/p4-camp-decay` + `territory/p4-penal-suppression`

**The rules (exact):**

```scheme
(rule territory/p4-camp-decay
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
  :material-basis "atomization via incarceration: every class tenant of a penal colony has its organization zeroed (territory.py:349-378)"
  :fuel 128
  (bindings
    (binding ttype :field territory/territory-type))
  (when (= ttype TerritoryType/PENAL_COLONY))
  (effects
    (for-each (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
      (update-node it social-class/organization (set 0)))))
```

Byte order `p4-camp-decay` < `p4-penal-suppression`: frozen interleaves both branches in one node loop, but their write sets are disjoint (territory population vs class organization), so rule-order equivalence is exact — noted in the pack header, not a D-record. Camp decay runs on post-p2 population (this-tick displaced arrivals decay same tick, frozen-faithful via sequential rules).

- [ ] **Step 1: Red** — e2e: camp with 500 + 100 same-tick arrivals ends `floor(600 × 0.8) = 480` (proves p2→p4 sequencing); both TENANCY tenants end organization 0; the unconnected class keeps 0.6 (`test_social_class_without_tenancy_edge_is_untouched` law, inventory §7); RESERVATION territory is untouched by p4.
- [ ] **Step 2:** Green. **Step 3: Mutation** — flip the p4 guard to RESERVATION → both tests flip; restore. **Step 4:** Gate + commit.

### Task 8: Composition golden + the D-record register + docs

**Files:**
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs` (add the territory pair — measured, then pinned)
- Modify: `docs/reference/bsl-language.rst` (the D-row register — new rows, next-free numbers resolved at PR time per D105 discipline)
- Modify: `reports/territory-port-phase1-inventory-2026-08-11.md` (second UPDATE block: the port landed; Q9/D111 discharged by ADR195/196 — enums direct, no bool/ordinal workaround; the four §6 BLOCKED rows discharged by ADR197)
- Create: `ai/decisions/ADR<next>_territory_port_handoff.yaml` + index.yaml row
- Modify: `rust/crates/babylon-tick/content/rules/vitality.bsl:16-19` (rider: the stale floor-is-blocked header — floor landed, ADR188 Row 2, floor_intrinsic_e2e proves it; one-paragraph correction)

**D-record rows (each with file:line evidence, written into the register AND the pack header):**
1. Phase order via rule-id byte prefixes (`p1<p2<p3<p4`) relying on D116's sequential divergence; phase boundaries = position boundaries when the anchor registry lands (the Q14 repair train's acceptance-criterion input).
2. `under-eviction` as int 0/1 (no bool lane on the live pipeline).
3. `rent-level-x1e6` scaled bare-Int lane (entropy_factor D-1 class; retires with #502 WS3's Real×Ratio operator).
4. Sink walk is DIRECTED `:out` (frozen territory.py:174) while spillover is `:any` — the frozen asymmetry transcribed, with the ADR179-T1 canonical-pair caveat quoted from territory.py:279-284.
5. Same-type multi-sink tiebreak: frozen enumeration-order last-wins vs BSL D45 ascending-id first-wins (the comparison query_lane_e2e.rs:206-211 says this port owes).
6. Two-clamp inconsistency: p1 `[0,1]` both sides, p3 upper-only (territory.py:137 vs :315), transcribed faithfully.
7. No-defaults: fixtures seed every read field; frozen dict-defaults not transcribable.
8. Hash-neutral no-op writes (p2's no-sink `(add 0)`, p3's isolated unchanged-set) where frozen skips writes.
9. Summation/apply order vs frozen float order — measured BSL expecteds are the oracle (ADR183).
10. displacement_mode → EXTRACTION const; override machinery + 4 dead AUTO defines → #502 WS1/WS4 ledger rows (post on #502 in this task).

- [ ] **Step 1:** Full-pack e2e: load the scenario + ALL FIVE rules through one `run_once_into`; assert the frozen-mirror STRUCTURE agreement (same nodes moved, same latch set, same sink chosen, same suppression set — values per BSL-measured expecteds).
- [ ] **Step 2:** Run once, read back the before/after hashes, pin them in tick_goldens (doc comment: measured-not-derived; what the before-pin discriminates — enum reorder → ordinal law).
- [ ] **Step 3:** Write the register rows, the inventory UPDATE block, the ADR (four-slice-style handoff: what landed, what discharged — D102, #551, Q9/D111 — what stays open: D116 dependence, WS1 ledger rows, WS3 rent retirement), the vitality.bsl rider, the lib.rs comment update.
- [ ] **Step 4:** Post the WS1/WS4 ledger comment on #502. Gate + commit `docs(p27): the Territory port handoff — D-records, register rows, inventory verdict update`.

## PR grouping

- **PR A (Tasks 1-2):** the babylon-bsl surface slice — D102 discharge + #551 closure. Merges alone so the content PR reviews against a stable language surface.
- **PR B (Tasks 3-8):** the content estate + goldens + docs. Verification arc runs the Director's dual lens (substrate/overlay AND FIPS/data-integrity) on this PR.

## Self-review notes

- Spec coverage: all four frozen phases, the latch semantics, both clamps, the directed sink walk, transfer accumulation, the untouched-class law, and every inventory D-record candidate have named tasks. The enum-storage gap (inventory finding 4) is discharged, not worked around.
- Type consistency: `territory/territory-type` (qname) vs `TerritoryType` (enum name) used consistently; all consts carry the `territory/` namespace; `floor` declared before use.
- The `(when #t)` guard shape and the `(+ heat gain)`/`(sub displaced)` operator spellings must be checked against the landed grammar at implementation time (Task 4 Step 1 says so) — vitality.bsl is the authority; if `add`/`sub`/`set` inner-form spellings differ from the sketches above, the landed pack's spelling wins (the sketches follow query_lane_e2e.rs and structural_verbs.rs test usage).
