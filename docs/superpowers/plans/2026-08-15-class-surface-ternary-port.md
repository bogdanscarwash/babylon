# W10 Class-Surface Ternary Port — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the class-surface consciousness estate onto the ruled (r, l, f) ternary in the Rust engine — probability-lane ternary deffields with UNPOSITIONED-on-absence, the dominant-pole read path with the LIBERAL-first tie-break in one declared home, and the measured update law (agitation pipeline + routing law) re-pointed from the legacy cc/ni accumulator onto the ternary directly (issue #588; ADR204 W10's second half).

**Architecture:** New BSL content only — one new scenario, one new rule pack (a ten-rule byte-ordered chain `p0..p9` anchored under the already-registered `consciousness` system namespace), one Python reference implementation of the re-pointed law (the dual-implementation conformance oracle), one Rust conformance test, additive hash pins. No Rust source changes; no frozen-Python changes; no edits to any existing content file.

**Tech Stack:** Rust workspace (`rust/crates/{babylon-bsl,babylon-graph,babylon-tick}`), BSL content (grammar: `docs/reference/bsl.ebnf` + `docs/reference/bsl-language.rst`), cargo via `mise run rust:check`, Python 3.12 host venv for the reference generator.

**Spec:** `docs/superpowers/specs/2026-08-14-worldview-ternary-unification-design.md` (§4.2 measured ternary, §4.3 preserved laws, §4.5.2 port staging) — the plan argues from it; read it first.
**Archaeology digest:** `ai/scratch/2026-08-15-class-surface-port-archaeology.md` (verified file:line citations for every frozen site transcribed here; read it second).
**Issue:** #588. **Mint:** PR #586 / ADR206 (`WorldView` defenum landed).

## Global Constraints

Every task's requirements implicitly include this section.

- **Frozen Python is reference-only.** Every behavioral divergence from it earns a D-row in the register (`docs/reference/bsl-language.rst`; the tail was D145 at plan time — verify the next free numbers at execution) and a citation in the closing ADR. Port-as-is discipline (production precedent): transcribe exactly, D-record honestly, never silently repair.
- **Every pre-existing golden byte-identical at landing.** This train adds NEW files; the one modified Rust file is `tests/tick_goldens.rs` (additive pins). If any existing pin, golden, or vault page moves: STOP.
- **No new formalism (AE ii).** The ternary is the measured alignment; no stored gap-dynamic, no new node/edge kinds, no new verbs, no minted math. The Curve-5 Gaussian is NOT transcribed (ADR202 R7 — see Task 3).
- **UNPOSITIONED by law (L-ABS / ADR070).** Absence over fabrication: no 0.5 defaults, no hidden fallbacks. The port's absence idiom is `:optional` + `:default` with an explicit sum-guard — declared, content-visible, never a fabricated reading.
- **The ternary deffields are the FIRST `probability`-typed fields in committed content** (digest B.2). Pin that by name in the conformance test header.
- **Director-reserved content is not touched:** pole names, taxonomy membership, floor values, the five outcomes. Two flag questions ride the plan gate (below).
- **Branches from `dev`, in an isolated worktree** created via the superpowers:using-git-worktrees skill at execution time: `feature/w10-class-surface-port` (PR A, Tasks 1-2), then a fresh branch off the merged dev for PR B (Tasks 3-4). Conventional Commits with the `Co-Authored-By: Kimi Code <noreply@moonshot.ai>` trailer via `mise run commit`. Merges only via `mise run pr:merge`.
- **`mise run rust:check` green after every task.** Task 4 additionally runs `mise run check`, `mise run qa:regression`, and `mise run qa:vault-regression-ci` once, proving the frozen estate untouched. No baseline under `tests/baselines/**` may move; if one does, STOP — that is a §6.5 ceremony, not a side effect.
- **Token economy:** subagents write artifacts to files and return ≤15-line summaries. No subagent reads back its own full output.
- **Field naming:** node fields are node-type-segmented (`social-class/*` — subject-type derivation, production.bsl header item (a)); rule ids anchor under the `consciousness` system namespace; defconst namespaces are free-form (`consciousness/*`, the `economy/*` precedent). Reuse exact carrier qnames already declared in `two-classes.bscn` / `production-conformance.bscn` where the concept is the same (read both files first; the names below assume the reuse check).

## Director flags at the plan gate

**RULED 2026-08-15, both as recommended:** flag 1 — the seed is **(0, 1, 0)**, pure A-001. Flag 2 — **transcribe the linear chauvinist term** (R7's site = the Gaussian only). Execution: subagent-driven. The plan stands as written; the two D-row rosters cite these rulings.

Two judgment calls sat at the boundary of the reserved ledger:

## Controller amendments (2026-08-15, post-Task-1)

Task 1's spikes + adjudications amended the port's mechanics. **Where any task text below conflicts with this section, this section wins.** (Preserved as written below for history; the amendments are the law.)

1. **No micros anywhere.** The int lane stores verbatim f64 (structural_verbs.rs:1576-1592; production-conformance's `wealth` holds 11.538461538461538 in an `int extensive` field). Fractional SEED literals on int fields are loader-refused; fractional WRITES store verbatim. Therefore: `social-class/agitation`, `social-class/wage-balance`, `social-class/solidarity-inbox` hold RAW f64 values in `int intensive` fields; every `(* 1000000 …)` / `(/ … 1000000)` in the Task-3 rule text and every `to_micros`/MICROS line in the generator is struck; the rounding D-row becomes: "no implicit truncation at the store; `floor` is the content-explicit truncation intrinsic, unused in this pack."
2. **`agitation` seeds are integer `0`** (fractional seeds refuse; agitation is a produced accumulator — R-MEASURED-honest content). The anchor table's tick-1 values therefore derive from produced agitation only: class-exploited → r 0.5072, l 0.382, f 0.1108, agitation-out 0.135, dominant REVOLUTIONARY; class-bribed → r 0.1, l 0.48, f 0.42, agitation-out 0.9, dominant FASCIST; class-emergent → r 0.009, l 0.982, f 0.009, agitation-out 0.135, dominant LIBERAL (all hand-verified by the controller against the update-law arithmetic).
3. **Wage flow rides `social-class/wages-received` (int extensive, declared per class per tick as content input).** Scenario-side edge-attribute seeding is unserved (`load_edge` is strength-only, scenario.rs:1296-1302); the WAGES edges, the `wages/value-flow` edge deffield, and the `social-class/wages-inbox` field all died in Task 1. The frozen incoming-WAGES fold-sum narrows to one declared flow value per class per tick (exact for single-employer content) — a recorded narrowing. Task 3's `p2-wages-push` rule and the p1 wages-inbox reset die with it; `p5-agitation`'s wage-change reads `wages-received − previous-wages` directly; `p7-persist-baselines` persists `wages-received`. Edge-attribute seeding is filed as BSL-ergonomics issue #591 item 3.
4. **The pack is NINE rules:** `p0-position` (landed), `p1-inbox-reset` (solidarity-inbox only), `p2-org-solidarity-push`, `p3-class-solidarity-push`, `p4-wage-balance`, `p5-agitation`, `p6-route`, `p7-persist-baselines`, `p8-dominant-worldview`. Task 2's dominant rule is `p8-`, not `p9-`; the Task-2 text's "p9" reads as `p8` everywhere, and Task 3's rule ids shift accordingly (Task-3 text's p1..p8 map to the nine-rule ids per this list).
5. **`simplex-epsilon` rides an expr quotient.** E-LEX-023 caps the literal lane at 9 fractional digits; the route rule binds `eps :expr (/ 1c 10000000000)` — correctly-rounded division, bit-identical to Python's `1e-10`. The defconst row is dead.
6. **`p0-position` has exactly four effects** (no dominant write — the one-home law; dominant's only writer is `p8-dominant-worldview`). Task 1's posture test pins dominant ABSENT on class-emergent until Task 2 lands the writer; Task 2's vectors add the positive class-emergent dominant==LIBERAL assertion.
7. **defenum sharing across scenarios is unserved** (one `(scenario …)` form per source, scenario.rs:313-318): the conformance scenario re-declares `(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))` and carries an ordinal-parity test mirroring the mint's. Filed as #591 item 4.
8. **Optional-bind `wages-received`** in every consuming rule — employer/class-unpositioned deliberately carry no seed.

Task-1 landed state these amendments describe: commits `de10e09f..fcbb9a5a` on `feature/w10-class-surface-port` (review clean).

1. **The class seed posture.** A class with material anchors (wages-paid + value-produced present) but no ternary record is positioned by `p0-position` at the seed **(0, 1, 0)** — A-001 (unorganized = liberal hegemonic default, spec 034) applied as the class-seeding law. The frozen bridge default (0, .5, .5) is the row-19 disease and dies by law; the community-model default (0.3, 0.6, 0.1) is floor content ruled for communities, never for classes. *Recommendation: (0, 1, 0) — `normalize_to_simplex`'s remainder-to-liberal branch already encodes the same default in the dynamics.* Floor values are reserved — confirm or overrule.
2. **The linear chauvinist pass-through.** ADR202 R7 retired the Gaussian at `sustained_exploitation.py:198` (the agitation *magnitude* component; replacement rides #491). The plan reads R7's "site" precisely — the Gaussian function only — and **transcribes** the separate linear term `chauvinist_pressure = max(0, balance) · chauvinist_pressure_scale` inside the routing law (pure arithmetic; the Emmanuel/MIM direction content, defines.yaml:228; ADR016 untouched). The wage-balance agitation *component* is absent (not zero-stubbed) pending #491. *Alternative reading: the entire wage-balance channel (linear term included) rides #491; the port stubs chauvinist pressure at 0.0c for now.* Confirm which reading holds.

## File Structure

| File | Responsibility |
|---|---|
| Create `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn` | All declarations + the seed world (classes, org, employer, edges) with spike verdicts recorded in the header |
| Create `rust/crates/babylon-tick/content/rules/consciousness.bsl` | The ten-rule pack `p0..p9` with the header carrying the D-record enumeration + D116 byte-order map |
| Create `rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py` | The re-pointed law's Python reference implementation; prints repr floats; mirrors the BSL binding order term-for-term |
| Create `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs` | Behavior vectors (read path, gates, UNPOSITIONED absence) + exact-f64 update-law conformance against the generator's output |
| Modify `rust/crates/babylon-tick/tests/tick_goldens.rs` | Additive hash pins for the new scenario |
| Modify `docs/reference/bsl-language.rst` | Register rows D146+ (Task 3 drafts, Task 4 lands) |
| Modify `docs/concepts/consciousness-taxonomy.rst` | A-001 one-home declaration, UNPOSITIONED law, hegemonic-community semantic-inversion page (spec §8 discharge) |
| Create `ai/decisions/ADR207_class_surface_ternary_port_handoff.yaml` + `index.yaml` row | The handoff record (verify next free ADR number at execution; index ended at ADR206) |
| Modify `ai/state.yaml` | Closing entry |

---

### Task 1: Content declaration + the UNPOSITIONED idiom + `p0-position`

**Files:**
- Create: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn`
- Create: `rust/crates/babylon-tick/content/rules/consciousness.bsl`
- Create: `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`

**Interfaces:**
- Consumes: the `WorldView` defenum (minted, `worldview-foundation.bscn`); the registered `consciousness` rule namespace (`babylon-tick/src/lib.rs:221-271`); the `:optional`/`:default` binding grammar (`bsl-language.rst` §3.5 — `:optional` is a flag requiring `:default <literal>`; both illegal on `:expr`).
- Produces: the deffield set every later task reads/writes (exact qnames below); the UNPOSITIONED idiom (optional `0.0p` bindings + sum-guard) every later rule follows; the spike verdicts recorded in the scenario header.

**The four spelling spikes.** Each has a named authority file and lands as a scenario-header comment. Fix spellings to the authority, never weaken an assertion.

1. **Probability-lane literals** — seed/`:default` form `0.0p` / `1.0p` (p/i/c unit-interval literals, ADR201's kind-blind widening). Fallback: `docs/reference/bsl.ebnf`'s literal production; record the lawful form.
2. **defenum sharing across scenario files** — try loading `worldview-foundation.bscn`'s declarations alongside the new scenario in one session. If the harness loads one scenario per session, re-declare `(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))` in the new scenario and add an ordinal-parity test mirroring `worldview_member_order_is_the_ruled_ordinal` (`tick_goldens.rs:281-293`).
3. **Edge declaration + edge-attribute deffield syntax** — authority: `content/scenarios/organization-foundation.bscn` (dyadic edges) and `content/scenarios/edge-write-lane-e2e.bscn` + `rust/crates/babylon-tick/tests/edge_write_lane_e2e.rs` (edge-attribute deffield rows, ADR203; the `<edge-type-lower>/<field>` naming, the `:strength` literal form). Needed for the WAGES `value-flow` attribute and the SOLIDARITY `:strength` literals.
4. **Float-expr → int-field write coercion** — production.bsl writes float exprs into int fields (`(add output)` into `social-class/wealth`); pin the rounding law (expected: truncation toward zero, mirrored by Python `int()`). The Task-3 conformance test proves it byte-exactly; if the store rounds differently, correct the generator's mirror and record the law in the rounding D-row.

- [ ] **Step 1: Write the failing posture test**

Create `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs` (harness imports mirror `territory_conformance.rs`; node-field reads use the same store API that file asserts with; absence assertions expect the store's absent-read error — digest B.2's "unwritten field errors on read (III.11)"):

```rust
//! Consciousness class-surface ternary port (issue #588, ADR204 W10).
//! FIRST consumer of the `probability` deffield lane in committed content
//! (digest B.2) — pinned by name here. UNPOSITIONED idiom: ternary fields
//! are never defaulted into existence; readers optional-bind with `0.0p`
//! defaults and gate on `(> (+ r (+ l f)) 0)` — a sum of zero IS "no
//! reading" (L-ABS), never a fabricated share.

#[test]
fn unpositioned_class_gets_no_reading() {
    // Load consciousness-ternary-conformance.bscn + consciousness.bsl; advance 1 tick.
    // class-unpositioned (no anchors, no ternary seed):
    //   - read of social-class/revolutionary errors (absent)
    //   - read of social-class/dominant-worldview errors (absent)
    // class-emergent (anchors, no ternary seed): p0 positioned it at
    //   (0.0, 1.0, 0.0) exactly; dominant == WorldView::LIBERAL member.
    // class-exploited (seeded (0.5, 0.4, 0.1)): p0 did NOT touch it.
    // employer (active, population, NO anchors): p0 did NOT position it.
}
```

Add the additive pin skeleton to `tests/tick_goldens.rs` mirroring the mint's pin (`:252-272`): `consciousness_ternary_foundation_hashes_are_pinned` — measure-and-pin in Step 4.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p babylon-tick --test consciousness_ternary_conformance`
Expected: FAIL — the scenario/rule files do not exist yet (loader error).

- [ ] **Step 3: Write the scenario + pack skeleton**

`consciousness-ternary-conformance.bscn` — header records the four spike verdicts + the first-probability-consumer note + the reuse-check note (which carrier qnames came from `two-classes.bscn` verbatim). Declarations:

```lisp
; ---- carriers (reuse exact qnames from two-classes.bscn where present) ----
(deffield social-class/population int extensive)
(deffield social-class/active int intensive)            ; 0/1 latch convention (no bool on the live path)
(deffield social-class/wealth int extensive)
(deffield social-class/wages-paid int extensive)        ; frozen w_paid (ideology.py:239)
(deffield social-class/value-produced int extensive)    ; frozen v_produced (ideology.py:240)
(deffield organization/active int intensive)            ; latch convention (production.bsl header item 2)

; ---- the ternary surface (FIRST probability deffields in committed content) ----
(deffield social-class/revolutionary probability intensive)
(deffield social-class/liberal probability intensive)
(deffield social-class/fascist probability intensive)
(deffield social-class/dominant-worldview enum WorldView)   ; spike 2 decides the declaration home

; ---- update-law machinery (int lanes are x1e6-scaled micros where noted) ----
(deffield social-class/agitation int intensive)          ; [0,∞) micros — no unbounded unit lane exists
(deffield social-class/wage-balance int intensive)       ; [-1,1] signed micros
(deffield social-class/solidarity-inbox int intensive)   ; [0,n) strength-sum micros, reset per tick
(deffield social-class/wages-inbox int intensive)        ; raw currency units, reset per tick
(deffield social-class/repression-faced intensity intensive) ; declared input only; nothing in Rust writes it yet
(deffield social-class/previous-wages int intensive)     ; raw units (the persistent_data re-home, digest gap 4)
(deffield social-class/previous-wealth int intensive)

; ---- edge attributes (spike 3 syntax; ADR203) ----
; (deffield wages/value-flow ...) per edge-write-lane-e2e.bscn

; ---- defines environment (transcribed; line cites = src/babylon/data/defines.yaml) ----
(defconst consciousness/routing-scale 0.2c)                    ; :213
(defconst consciousness/agitation-decay-rate 0.1c)             ; :214
(defconst consciousness/exploitation-sensitivity 0.15c)        ; :215
(defconst consciousness/rent-decline-sensitivity 0.2c)         ; :216
(defconst consciousness/reproduction-visibility-coefficient 0.1c) ; :217 — term is 0.0 verbatim (ideology.py:375)
(defconst consciousness/agitation-consumption-rate 0.6c)       ; :220
(defconst consciousness/chauvinist-pressure-scale 1.0c)        ; :228
(defconst consciousness/repression-level-sensitivity 0.02c)    ; :229
(defconst consciousness/default-repression-faced 0.5c)         ; :167 (default_repression; verify the DEFAULT_REPRESSION_FACED alias target at execution)
(defconst consciousness/solidarity-activation-threshold 0.3c)  ; :184 (activation_threshold)
(defconst consciousness/negligible-transmission 0.01c)         ; :186
(defconst consciousness/simplex-epsilon 0.0000000001c)         ; consciousness_routing.py:41 (_EPSILON = 1e-10)
(defconst consciousness/wage-deterioration-stub 0.0c)          ; D-row: opposition_states graph attr has no BSL surface
(defconst consciousness/popular-front-suppression-stub 0.0c)   ; D-row: electoral register absent (exact under register-absent content, ideology.py:401-409)
```

Seed world (small ints by design — increments stay order-1 so the exact-f64 anchors are hand-checkable):

```lisp
; class-exploited — positioned, anchored, wage cut this tick, one org
; SOLIDARITY edge (strength 0.4p): the revolutionary-routing vector.
(node class-exploited NodeType/SOCIAL_CLASS
  (social-class/population 1000) (social-class/active 1)
  (social-class/wealth 50) (social-class/wages-paid 9) (social-class/value-produced 10)
  (social-class/revolutionary 0.5p) (social-class/liberal 0.4p) (social-class/fascist 0.1p)
  (social-class/agitation 100000)              ; 0.1 in micros
  (social-class/previous-wages 10)             ; cut: 10 -> 9 (edge below carries 9)
  (social-class/previous-wealth 50))

; class-bribed — positioned, positive balance (12/10 -> +0.0909…), NO
; solidarity: the ADR016 fascist-routing vector; wealth declines 95 -> 90
; (the rent component). Also its outgoing SOLIDARITY 0.9p to class-emergent
; is gate-BLOCKED (its r = 0.1 <= 0.3): the percolation fail arm.
(node class-bribed NodeType/SOCIAL_CLASS
  (social-class/population 800) (social-class/active 1)
  (social-class/wealth 90) (social-class/wages-paid 12) (social-class/value-produced 10)
  (social-class/revolutionary 0.1p) (social-class/liberal 0.6p) (social-class/fascist 0.3p)
  (social-class/agitation 200000)              ; 0.2
  (social-class/previous-wages 12)             ; flat
  (social-class/previous-wealth 95))

; class-unpositioned — NO anchors, NO ternary: the UNPOSITIONED witness.
(node class-unpositioned NodeType/SOCIAL_CLASS
  (social-class/population 500) (social-class/active 1))

; class-emergent — anchors, NO ternary: p0 positions it at (0,1,0) this tick,
; then the whole pipeline runs on it same-tick (D116). Wage cut 9 -> 8 keeps
; its agitation live so the solidarity push lands on a routing class.
(node class-emergent NodeType/SOCIAL_CLASS
  (social-class/population 600) (social-class/active 1)
  (social-class/wealth 30) (social-class/wages-paid 8) (social-class/value-produced 10)
  (social-class/previous-wages 9) (social-class/previous-wealth 30))

; employer — active, NO anchors: the WAGES/SOLIDARITY source that must never
; be positioned (p0 guard) nor routed (sum-guard).
(node employer NodeType/SOCIAL_CLASS
  (social-class/population 50) (social-class/active 1))

; org-solid — the org solidarity source (active latch convention).
(node org-solid NodeType/ORGANIZATION (organization/active 1))
```

Edges (spike-3 syntax): `WAGES` employer→class-exploited (value-flow 9), employer→class-bribed (12), employer→class-emergent (8); `SOLIDARITY` org-solid→class-exploited (`:strength 0.4p`), class-exploited→class-emergent (`0.5p`), class-bribed→class-emergent (`0.9p`, gate-blocked).

`consciousness.bsl` — pack header (D-record enumeration, the D116 byte-order map, the UNPOSITIONED idiom statement, the production.bsl-header shape as the model) plus the first rule:

```lisp
(rule consciousness/p0-position
  :material-basis "A-001 as the class-seeding law (Director flag 1): a class with material anchors (wages-paid + value-produced present) and no ternary record is positioned at the ruled unorganized rest state (0, 1, 0) — liberal hegemonic default, spec 034 A-001, THE one home (the seven scattered frozen sites are named in docs/concepts/consciousness-taxonomy.rst, not re-homed here). Data-absent classes are never positioned: UNPOSITIONED (L-ABS) — the row-19 disease's death certificate."
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
```

- [ ] **Step 4: Run, fix spellings per the spike authorities, pin**

Run the test; fix load errors by consulting the spike authority files (never by weakening assertions). When green: measure the scenario's before/after hashes and fill the additive pin in `tick_goldens.rs`; assert p0 fired exactly once (class-emergent). Add the ordinal-parity test if spike 2 landed on re-declaration.

- [ ] **Step 5: Gate + commit**

`mise run rust:check` green. Commit: `feat(tick): declare the consciousness ternary surface + p0-position (W10 port, #588)` with trailer; body records the spike verdicts.

---

### Task 2: The read path — `p8-dominant-worldview` (one-home A-001 tie-break)

> **AMENDED by the Controller-amendments section above (items 4 and 6):** the rule id is `p8-`, not `p9-`; p0 writes no dominant — the class-emergent positive dominant assertion joins this task's vectors. Where this task's text conflicts with the amendments, the amendments win.

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/consciousness.bsl`
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn` (additive: tie-vector classes)
- Modify: `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs`

**Interfaces:**
- Consumes: Task 1's deffields + idiom; the frozen tie-break transcribed from `models/entities/consciousness.py:177-192` (verbatim below).
- Produces: `social-class/dominant-worldview` written per positioned class per tick (hash-neutral repeat writes, the D127 idiom); THE declared home of the hegemonic tie-break (taxonomy doc cross-ref lands in Task 4).

The frozen law (digest A.2, transcription source): `max_val = max(r, l, f)`; then in the order LIBERAL → REVOLUTIONARY → FASCIST, the first component within `1e-6` of `max_val` wins. The rule id sorts last (`p9-`) so once Task 3 lands, the readout reflects the same tick's update (D116) — matching the frozen step's post-update read.

- [ ] **Step 1: Write the failing tie-break vectors**

Add to the scenario (read-path fixtures: ternary + `agitation 0` seeded, NO anchors and NO edges, so Task 3's rules never touch them — p0 skips (ternary present), p5-p8 skip (anchors absent), p9 reads):

| node | seed (r, l, f) | expected dominant |
|---|---|---|
| tv-liberal-clear | (0.2, 0.5, 0.3) | LIBERAL |
| tv-revolutionary-clear | (0.6, 0.4, 0.0) | REVOLUTIONARY |
| tv-fascist-clear | (0.2, 0.3, 0.5) | FASCIST |
| tv-tie-lr | (0.5, 0.5, 0.0) | LIBERAL |
| tv-tie-rf | (0.5, 0.0, 0.5) | REVOLUTIONARY |
| tv-tie-lf | (0.0, 0.5, 0.5) | LIBERAL |
| tv-tie-all | (0.333333p, 0.333333p, 0.333334p) | LIBERAL |

Each also seeds `(social-class/population N)` (any N) — subject-derivation fields must exist — and NOT `social-class/active`… wait: p9's guard reads `active`; give every tv class `(social-class/active 1)`. Then in the .rs test: assert each tv class's `dominant-worldview` reads back as the expected `WorldView` member (the `Value::Enum` read-back shape, digest B.2; mirror the OrgKind read-back pattern), and `class-unpositioned` still has no `dominant-worldview` after the tick.

- [ ] **Step 2: Run to verify red** — `p9-dominant-worldview` does not exist; dominant reads error.

- [ ] **Step 3: Implement `p9-dominant-worldview`**

Append to `consciousness.bsl` (abs-via-if and clamp-via-if idioms follow production.bsl:180,261 — the expr language has no `abs`/`min`/`max` intrinsics):

```lisp
(rule consciousness/p9-dominant-worldview
  :material-basis "The measured readout: dominant pole = argmax with the ruled tie order LIBERAL > REVOLUTIONARY > FASCIST at 1e-6 (frozen: models/entities/consciousness.py:177-192, transcribed verbatim). ONE DECLARED HOME for the hegemonic tie-break — the frozen estate smeared it across five sites (digest A.5c); here it lives exactly once. UNPOSITIONED classes (sum 0) are skipped: no reading, ever."
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
```

- [ ] **Step 4: Run to verify green**; re-measure additive pins (the tv classes' post-tick state now carries `dominant-worldview`).

- [ ] **Step 5: Gate + commit + PR A**

`mise run rust:check` green. Commit: `feat(tick): dominant-worldview read path — one-home A-001 tie-break (#588)`. Open PR A (`feat(tick): W10 class-surface port — content + read path`, body cites #588/ADR204 W10/this plan), wait CI green, harvest Copilot (every inline comment gets a fix or a `gh api .../comments/CID/replies` reply), merge via `mise run pr:merge -- <N>`.

---

### Task 3: The measured update law — `p1..p7` + the dual-implementation conformance

> **AMENDED by the Controller-amendments section above (items 1-5 and 8):** no micros (raw f64 in int lanes), agitation seeds 0 with the recomputed anchor table in amendment item 2, wage flow rides `wages-received` (the WAGES-edge machinery is dead), nine-rule ids `p0..p8` per the amendment's map, epsilon rides the expr quotient, `wages-received` is optional-bound. Where this task's text conflicts with the amendments, the amendments win.

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/consciousness.bsl`
- Create: `rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py`
- Modify: `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`
- Modify: `docs/reference/bsl-language.rst` (register rows, drafted here, landed in Task 4)

**Interfaces:**
- Consumes: Tasks 1-2's deffields, idiom, spike verdicts; the frozen transcription sources (exact lines below); the confirmed effect grammar — `for-each` + `guard` effect forms (`bsl-language.rst` §2.8), `edge-between` / `field-of` accessors (§2.10/§3.8), the `solidarity/strength` implicit field and the `<edge-type-lower>/<field>` edge-attribute names (`tests/edge_write_lane_e2e.rs:67-91`).
- Produces: the ten-rule pack complete; the Python reference generator; exact-f64 conformance; the D-row roster (D146+; verify next free) for Task 4's register landing.

**Branch:** PR A merged — open `feature/w10-class-surface-port-b` from the updated `dev` in the same worktree (per the using-git-worktrees skill). Every file Tasks 3-4 touches already exists on dev via PR A (the pack, the scenario, the test, `tick_goldens.rs`), so the fresh branch starts from the landed state directly.

**Frozen transcription sources** (verified by the digest; re-read at execution before transcribing):
- `src/babylon/engine/systems/ideology.py:115-442` — the step: input reads (:236-317), the agitation call (:372-380), the routing call (:394-400), the popular-front throttle (:409), the accumulator clamps (:410-411), the decay (:413-414), the writes (:418-438).
- `src/babylon/formulas/consciousness_routing.py:48-200` — `compute_agitation_delta`; `:288-370` — `route_agitation_to_ternary`; `:373-409` — `normalize_to_simplex`; `:41` — `_EPSILON = 1e-10`.
- `src/babylon/formulas/contradiction.py:67-85+` — `calculate_wealth_asymmetry_balance`: `(W_b − W_a)/(W_a + W_b)` clamped to [−1, 1], zero-guard below 1e-9 → 0.0. Called as `(v_produced, w_paid)` — positive = wages dominate = the imperial bribe.

**The re-point (the headliner D-row):** the frozen engine accumulates `cc' = min(1, cc+Δr)`, `ni' = min(1, ni+Δf)` and discards `_delta_l` at the class call-site (ideology.py:394,410-411); the ternary is bridged at read (`r = cc·(1−ni)` etc., aggregation.py:86-98). The port stores the ternary directly: `r += Δr`, `l += Δl` (APPLIED, not discarded), `f += Δf·(1−suppression)`, then closure via a verbatim `normalize_to_simplex` transcription. The bridge and the cc/ni estate are retired (W1/W11). The port therefore diverges from frozen trajectories BY CONSTRUCTION — the conformance oracle is the dual implementation (this task's .py generator), not frozen floats (ADR183: the frozen engine is a structure/ordering contract, not a byte oracle).

**Byte-order map (D116 reliance, documented in the pack header exactly like production.bsl's):**

| rule | subject | reads | writes |
|---|---|---|---|
| p0-position | SOCIAL_CLASS | active, anchors, ternary | r/l/f, agitation (seed) |
| p1-inbox-reset | SOCIAL_CLASS | ternary (sum-guard) | solidarity-inbox, wages-inbox ← 0 |
| p2-wages-push | SOCIAL_CLASS (employer side) | active; per-edge value-flow via edge-between | targets' wages-inbox (add) |
| p3-org-solidarity-push | ORGANIZATION | active; per-edge strength | targets' solidarity-inbox (add), strength > 0.01 gate |
| p4-class-solidarity-push | SOCIAL_CLASS | own r (optional); per-edge strength | targets' solidarity-inbox (add), r > 0.3 percolation gate |
| p5-wage-balance | SOCIAL_CLASS | wages-paid, value-produced (optional) | wage-balance (micros) |
| p6-agitation | SOCIAL_CLASS | inboxes, previous-*, repression-faced, ternary sum-guard, consts | agitation (undecayed) |
| p7-route | SOCIAL_CLASS | agitation, inbox, wage-balance, ternary, consts | r/l/f (routed + closure), agitation (decayed) |
| p8-persist-baselines | SOCIAL_CLASS | wages-inbox, wealth, anchors | previous-wages, previous-wealth |
| p9-dominant-worldview | SOCIAL_CLASS | ternary | dominant-worldview |

- [ ] **Step 1: Write the Python reference generator (the executable spec)**

Create `consciousness_ternary_conformance.py`. It mirrors the pack's binding order operation-for-operation — reassociation is a conformance bug. Pure IEEE-754 basic ops only; no transcendentals exist anywhere in the re-pointed law (the Curve-5 Gaussian is retired, ADR202 R7). Core:

```python
#!/usr/bin/env python3
"""Reference implementation of the RE-POINTED class-surface consciousness law
(issue #588, ADR204 W10). NOT the frozen engine's behavior: the frozen engine
accumulates cc/ni and bridges to the ternary at read; the port accumulates the
ternary directly with simplex closure (D-row: re-pointed accumulator). This
script is the dual-implementation conformance oracle — it mirrors
consciousness.bsl's binding order term-for-term and prints repr floats for the
Rust test to pin exactly (pure basic ops; no libm transcendentals)."""

MICROS = 1_000_000
ROUTING_SCALE = 0.2                     # defines.yaml:213
AGITATION_DECAY_RATE = 0.1              # :214
EXPLOITATION_SENSITIVITY = 0.15         # :215
RENT_DECLINE_SENSITIVITY = 0.2          # :216
AGITATION_CONSUMPTION_RATE = 0.6        # :220
CHAUVINIST_PRESSURE_SCALE = 1.0         # :228
REPRESSION_LEVEL_SENSITIVITY = 0.02     # :229
DEFAULT_REPRESSION_FACED = 0.5          # :167
ACTIVATION_THRESHOLD = 0.3              # :184
NEGLIGIBLE_TRANSMISSION = 0.01          # :186
SIMPLEX_EPSILON = 1e-10                 # consciousness_routing.py:41
WAGE_DETERIORATION_STUB = 0.0           # D-row: no graph-attr surface
POPULAR_FRONT_SUPPRESSION_STUB = 0.0    # D-row: register absent

def to_micros(x: float) -> int:
    return int(x * MICROS)  # int() truncates toward zero — mirrors the store coercion (spike 4)

def normalize_simplex(r: float, l: float, f: float):
    # consciousness_routing.py:389-409, transcribed verbatim
    r = max(0.0, r)
    l = max(0.0, l)
    f = max(0.0, f)
    total = r + l + f
    if total < SIMPLEX_EPSILON:
        return 0.0, 1.0, 0.0
    if total > 1.0 + SIMPLEX_EPSILON:
        r /= total
        l /= total
        f /= total
    elif total < 1.0 - SIMPLEX_EPSILON:
        l += 1.0 - total
    return r, l, f

def tick_class(node, wages_in, solidarity_in, r, l, f, agitation_micros):
    """One positioned class's p5->p9 chain. wages_in: current WAGES sum (raw
    units). solidarity_in: gated strength sum (raw float). Mirrors the pack's
    binding order exactly."""
    # p5-wage-balance — contradiction.py:67-85, (v, w) order, zero-guard
    wages_paid = node["wages_paid"]
    value_produced = node["value_produced"]
    if wages_paid + value_produced > 0:
        balance = (wages_paid - value_produced) / (value_produced + wages_paid)
    else:
        balance = 0.0
    balance_micros = to_micros(balance)
    # p6-agitation — ideology.py:298-317, 372-380 + consciousness_routing.py:154-200
    wage_change = wages_in - node["previous_wages"]
    exploitation_delta = abs(wage_change) if wage_change < 0 else 0.0
    wealth_change = node["wealth"] - node["previous_wealth"]
    rent_delta = wealth_change  # frozen passes wealth_change as imperial_rent_delta (ideology.py:374)
    visibility_delta = 0.0      # verbatim (ideology.py:375)
    exploit_component = max(0.0, exploitation_delta) * EXPLOITATION_SENSITIVITY
    rent_component = max(0.0, -rent_delta) * RENT_DECLINE_SENSITIVITY
    vis_component = max(0.0, visibility_delta) * 0.1  # reproduction_visibility_coefficient, defines.yaml:217
    # Curve-5 wage-balance component: ABSENT (ADR202 R7 — replacement rides #491)
    repression_level = max(0.0, node.get("repression_faced", DEFAULT_REPRESSION_FACED) - DEFAULT_REPRESSION_FACED)
    repression_component = max(0.0, repression_level) * REPRESSION_LEVEL_SENSITIVITY
    increment = exploit_component + rent_component + vis_component + repression_component
    new_agitation = agitation_micros / MICROS + increment + WAGE_DETERIORATION_STUB
    agitation_undecayed_micros = to_micros(new_agitation)
    # p7-route — consciousness_routing.py:345-370, re-pointed
    if new_agitation <= 0:
        delta_r = delta_l = delta_f = 0.0
    else:
        consumed = new_agitation * AGITATION_CONSUMPTION_RATE
        solidarity_factor = min(1.0, solidarity_in)
        chauvinist = max(0.0, balance_micros / MICROS) * CHAUVINIST_PRESSURE_SCALE
        eff_sol = min(1.0, solidarity_factor + 0.0)
        eff_sol = max(0.0, min(1.0, eff_sol - chauvinist))
        delta_r = consumed * eff_sol * ROUTING_SCALE
        delta_f = consumed * (1.0 - eff_sol) * ROUTING_SCALE
        delta_f = delta_f * (1.0 - POPULAR_FRONT_SUPPRESSION_STUB)
        delta_l = -(delta_r + delta_f)  # APPLIED (frozen discards at ideology.py:394) — the re-point
    r2, l2, f2 = normalize_simplex(r + delta_r, l + delta_l, f + delta_f)
    agitation_out = to_micros(max(0.0, new_agitation * (1.0 - AGITATION_DECAY_RATE)))
    # p8-persist + p9-dominant (argmax, tie order L > R > F at 1e-6)
    mx = max(r2, l2, f2)
    dominant = "LIBERAL" if abs(l2 - mx) < 1e-6 else ("REVOLUTIONARY" if abs(r2 - mx) < 1e-6 else "FASCIST")
    return r2, l2, f2, agitation_undecayed_micros, agitation_out, balance_micros, dominant
```

The scenario table in the generator encodes the seed world plus the edge sums (wages_in: exploited 9, bribed 12, emergent 8; solidarity_in: exploited 0.4, emergent 0.5 — the class-bribed→class-emergent 0.9 push is gate-blocked at source r = 0.1 ≤ 0.3) and prints every positioned class's `(r2, l2, f2, agitation_undecayed_micros, agitation_out, balance_micros, dominant)` as repr floats, plus the p0 seed result for class-emergent's tick-1 start.

**Hand-computed anchor table (direction checks for the reviewer; the exact repr floats come from the generator):**

| class | balance (micros) | increment | consumed | eff_sol | (r′, l′, f′) approx | agitation out (micros) | dominant |
|---|---|---|---|---|---|---|---|
| class-exploited | trunc(−1/19·1e6) = −52631 | 0.15 | 0.15 | 0.4 | (0.512, 0.37, 0.118) | 225000 | REVOLUTIONARY |
| class-bribed | trunc(2/22·1e6) = 90909 | 1.0 | 0.72 | 0.0 (chauvinist-clamped) | (0.1, 0.456, 0.444) | 1080000 | FASCIST |
| class-emergent | trunc(−2/18·1e6) = −111111 | 0.15 | 0.09 | 0.5 | (0.009, 0.982, 0.009) | 135000 | LIBERAL |
| class-unpositioned | — (absent) | — | — | — | absent | absent | absent |
| employer | — (absent) | — | — | — | absent | absent | absent |

- [ ] **Step 2: Run the generator; verify against the anchor table by hand**

Run: `uv run python rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py`
Expected: printed repr floats whose decimal values match the anchor table's approximations (sign of balance, gate pass/fail, dominant pole, sum-to-1 within 1e-9). Any mismatch: the generator is wrong (fix it against the frozen citations) — never adjust the anchors to fit code.

- [ ] **Step 3: Write the failing conformance test**

Add to `consciousness_ternary_conformance.rs`: one tick; assert every positioned class's seven outputs against the generator's repr floats EXACTLY (no tolerance — the estate norm, `lifecycle_conformance.rs` header); assert `class-unpositioned` and `employer` carry none of the seven fields; assert `class-emergent`'s wages-inbox is 0 post-tick (p1 reset ran after the push — byte-order proof) and its solidarity-inbox is 500000 micros (the class-bribed 0.9 push did NOT leak — percolation-gate proof). Additive pins in `tick_goldens.rs`.

Expected red: rules p1–p8 do not exist; only p0/p9 fire.

- [ ] **Step 4: Implement `p1..p8` in `consciousness.bsl`**

Full rule text, using only confirmed grammar (`for-each`/`guard` §2.8, `neighbors` 4-operand, `edge-between`, `field-of`, `solidarity/strength` implicit field, `it` as the default element name — the `:as` naming spelling is NOT assumed; use `it`):

```lisp
(rule consciousness/p1-inbox-reset
  :material-basis "Per-tick accumulator reset (production p0 idiom, D103/D104 collect-then-apply makes reset-then-accumulate safe): the inboxes are machinery, not state — they carry this tick's pushed contributions only."
  :fuel 32
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p))
  (when (> (+ r (+ l f)) 0))
  (effects
    (update-node self social-class/solidarity-inbox (set 0))
    (update-node self social-class/wages-inbox (set 0))))

(rule consciousness/p2-wages-push
  :material-basis "The wage-flow sum as a producer-side PUSH (the D136 fix-round pattern; exact vs the frozen pull at ideology.py:299-302 — each edge is pushed exactly once by its unique source). Content discipline: at most one WAGES edge per (employer, class) pair — a multi-edge pair sums per-neighbor here vs per-edge in the frozen engine (recorded narrowing, D-row)."
  :fuel 128
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/WAGES :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/wages-inbox
        (add (field-of (edge-between EdgeType/WAGES self it) wages/value-flow))))))

(rule consciousness/p3-org-solidarity-push
  :material-basis "Org-sourced solidarity: strength above negligible_transmission counts (frozen ideology.py:339-356). Push form — the per-edge gate lives on the source side where it is expressible (D138 forbids filter-in-fold)."
  :fuel 128
  (bindings
    (binding active :field organization/active)
    (binding negligible :const consciousness/negligible-transmission))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (guard (> (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength) negligible)
        (update-node it social-class/solidarity-inbox
          (add (* 1000000 (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength))))))))

(rule consciousness/p4-class-solidarity-push
  :material-basis "Class-sourced solidarity transmits only past the percolation threshold (frozen: source class_consciousness > activation_threshold, ideology.py:339-356) — re-pointed to the source's revolutionary share (the same quantity post-W1 unification; D-row). An UNPOSITIONED source reads r = 0.0p by the idiom and never transmits: absence is not organization."
  :fuel 128
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding threshold :const consciousness/solidarity-activation-threshold))
  (when (> r threshold))
  (effects
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/solidarity-inbox
        (add (* 1000000 (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength)))))))

(rule consciousness/p5-wage-balance
  :material-basis "The per-class wage-value balance (contradiction.py:67-85: (w−v)/(v+w), zero-guard) — the imperial-bribe measure. Frozen reads the per-class pair when present (ideology.py:239-259), which is the ONLY path the port carries: data-absent classes are UNPOSITIONED, never the graph-attr fallback (that attr has no BSL surface — D-row). Stored in signed micros."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding balance :expr (if (> (+ wages value) 0)
                               (/ (- wages value) (+ value wages))
                               (- 0 0c))))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/wage-balance (set (* 1000000 balance)))))

(rule consciousness/p6-agitation
  :material-basis "compute_agitation_delta (consciousness_routing.py:48-200) + the frozen call-site's exact argument mapping (ideology.py:372-380): exploitation_delta = |wage_change| when wages fall; wealth_change passed as imperial_rent_delta; visibility 0.0 verbatim; the Curve-5 balance component ABSENT (ADR202 R7); repression as produced-excess-over-baseline, absent contributing zero (MEDIUM-2 discipline). Writes the UNDECAYED level; p7 routes on it and writes the decayed store."
  :fuel 224
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p)
    (binding inbox :field social-class/wages-inbox :optional :default 0)
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
    (binding wage-change :expr (- inbox prev-wages))
    (binding exploit-delta :expr (if (< wage-change 0) (- 0 wage-change) 0))
    (binding wealth-change :expr (- wealth prev-wealth))
    (binding increment :expr
      (+ (* (if (> exploit-delta 0) exploit-delta 0) exploit-sens)
         (+ (* (if (> (- 0 wealth-change) 0) (- 0 wealth-change) 0) rent-sens)
            (+ (* 0.0c vis-coeff)
               (* (if (> (- rf rep-base) 0) (- rf rep-base) 0) rep-sens)))))
    (binding new-agitation :expr (+ (/ agitation 1000000) (+ increment wd-stub))))
  (when (and (>= wages 0) (>= value 0) (> (+ r (+ l f)) 0)))
  (effects
    (update-node self social-class/agitation (set (* 1000000 new-agitation)))))

(rule consciousness/p7-route
  :material-basis "The ratified bifurcation law (ADR016; route_agitation_to_ternary, consciousness_routing.py:345-370) RE-POINTED at the stored ternary: solidarity routes agitation revolutionary-ward, its absence fascist-ward; chauvinist pressure (the positive-balance imperial bribe, Director flag 2) biases the split; Δl APPLIED here (frozen discards it at the class call-site) with closure by a verbatim normalize_to_simplex (:373-409). Decay store follows ideology.py:413-414."
  :fuel 256
  (bindings
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding l :field social-class/liberal :optional :default 0.0p)
    (binding f :field social-class/fascist :optional :default 0.0p)
    (binding agitation :field social-class/agitation)   ; required: p6 wrote it this tick (D116) for every positioned class
    (binding inbox :field social-class/solidarity-inbox :optional :default 0)
    (binding balance-micros :field social-class/wage-balance :optional :default 0)
    (binding consumption :const consciousness/agitation-consumption-rate)
    (binding routing-scale :const consciousness/routing-scale)
    (binding chauv-scale :const consciousness/chauvinist-pressure-scale)
    (binding decay :const consciousness/agitation-decay-rate)
    (binding suppression :const consciousness/popular-front-suppression-stub)
    (binding eps :const consciousness/simplex-epsilon)
    (binding new-agitation :expr (/ agitation 1000000))
    (binding consumed :expr (* new-agitation consumption))
    (binding sol-factor :expr (if (< (/ inbox 1000000) 1) (/ inbox 1000000) (- 1 0c)))
    (binding chauvinist :expr (* (if (> balance-micros 0) (/ balance-micros 1000000) 0) chauv-scale))
    (binding eff-raw :expr (if (< (+ sol-factor 0.0c) 1) (+ sol-factor 0.0c) (- 1 0c)))
    (binding eff-sol :expr (if (> (- eff-raw chauvinist) 0) (if (< (- eff-raw chauvinist) 1) (- eff-raw chauvinist) (- 1 0c)) (- 0 0c)))
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
    (binding decayed :expr (if (> (* new-agitation (- 1 decay)) 0)
                               (* new-agitation (- 1 decay))
                               (- 0 0c))))
  (when (> (+ r (+ l f)) 0))
  (effects
    (update-node self social-class/revolutionary (set r-out))
    (update-node self social-class/liberal (set l-out))
    (update-node self social-class/fascist (set f-out))
    (update-node self social-class/agitation (set (* 1000000 decayed)))))

(rule consciousness/p8-persist-baselines
  :material-basis "The persistent previous-values re-homed to node fields (digest gap 4 — context.persistent_data has no BSL analog): next tick's deltas read this tick's sums. Anchored classes only."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding inbox :field social-class/wages-inbox :optional :default 0)
    (binding wealth :field social-class/wealth :optional :default 0))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/previous-wages (set inbox))
    (update-node self social-class/previous-wealth (set wealth))))
```

Note the `0.0c`/`(- 0 0c)`/`(- 1 0c)` float-literal idioms follow production.bsl exactly; if the typechecker wants a different lane suffix in any position, fix to the authority (`bsl.ebnf` literal production) and record it in the header.

- [ ] **Step 5: Run to verify green; pin**

Run the conformance test. Discrepancy triage order: (1) the .py generator mirrors a binding wrongly (fix the generator); (2) a rule's expr tree associates differently than the generator (fix the rule — the BSL side is the transcription of record); (3) the store's float→int coercion law differs from `int()` truncation (fix the generator's `to_micros` mirror and record the true law in the rounding D-row). Never "adjust" expecteds. When green: measure-and-add the additive pins in `tick_goldens.rs` (post-tick hashes for the update scenario now that all ten rules fire; expected `fired` counts on this content — p0:1 (class-emergent), p1:10 (every positioned class), p2:12 (all classes are active; for-each no-ops where no outgoing WAGES), p3:1 (org-solid), p4:5 (sources with r > 0.3: class-exploited + tv-revolutionary-clear + tv-tie-lr + tv-tie-rf + tv-tie-all), p5:3, p6:3, p7:10 (positioned; tv classes read their seeded `agitation 0` and no-op), p8:3, p9:10 — verify against the harness's actual `fired` semantics, don't assume).

- [ ] **Step 6: Draft the D-row roster**

Draft (do not land — Task 4 lands) register rows D146+ in a scratch block at the bottom of the pack header first, then Task 4 moves them into `docs/reference/bsl-language.rst`:

1. **Re-pointed accumulator** — ternary stored and updated directly; Δl applied (frozen discards at ideology.py:394); closure via transcribed `normalize_to_simplex` replacing the per-axis `min(1,·)` clamps (ideology.py:410-411); cc/ni estate + bridge (aggregation.py:86-98) retired per W1/W11. Trajectories diverge from frozen by construction; oracle = dual implementation.
2. **Curve-5 Gaussian not transcribed** (ADR202 R7) — the wage-balance agitation component is absent; the magnitude-only E/P/S partition replacement rides #491. Linear chauvinist pass-through transcribed per Director flag 2 [amend per ruling].
3. **`wage_deterioration` stubbed 0.0c** — `opposition_states` graph attr has no BSL surface (ideology.py:153-157).
4. **`popular_front_suppression` stubbed 0.0c** — electoral register absent; exact under register-absent content (frozen's own :401-409 note: absent ⟹ 0.0 ⟹ bit-for-bit pre-U12 arithmetic).
5. **`material_conditions` buffer write not ported** (ideology.py:424-437: exploitation_visibility, reification_buffer, working_day_modifier) — no ported consumers; lands with its consumer systems' trains.
6. **Scaled-int lanes + write-coercion rounding law** — agitation/wage-balance/solidarity-inbox ride int micros; float-expr→int-field coercion law pinned by conformance [truncation toward zero, or the empirically corrected law from Step 5]; the .py reference mirrors it with `int()`.
7. **Solidarity/wages pull→push redesign** (D136-pattern) — exact vs the frozen sums (each edge pushed once by its unique source); per-(source,target)-pair multi-edge content narrows to per-neighbor summation (content discipline: one edge per pair).
8. **Class-source percolation re-point** — frozen gates on source `class_consciousness` (cc axis); the port gates on source `revolutionary` share — the same quantity post-W1.
9. **Seed posture (0,1,0) at first anchors** — A-001 as the class-seeding law [Director flag 1's ruling recorded here].
10. **UNPOSITIONED idiom** — optional `0.0p` bindings + sum-guard; no has-field combinator exists (digest gap 1); the idiom is the lawful §3.5 shape and is the row-19 disease's death certificate.

- [ ] **Step 7: Gate + commit**

`mise run rust:check` green. Commit: `feat(tick): measured update law — agitation pipeline + routing re-pointed at the ternary (#588)` with trailer; body carries the D-row roster summary + the anchor-table provenance.

---

### Task 4: Records, docs, issue hygiene, full gates, PR B

**Files:**
- Modify: `docs/reference/bsl-language.rst` (land the D-row roster; renumber against the register tail at execution)
- Modify: `docs/concepts/consciousness-taxonomy.rst`
- Modify: `rust/crates/babylon-tick/content/rules/consciousness.bsl` (header's scratch D-row block becomes a pointer to the register)
- Create: `ai/decisions/ADR207_class_surface_ternary_port_handoff.yaml` (verify next free number)
- Modify: `ai/decisions/index.yaml`
- Modify: `ai/state.yaml`

**Interfaces:**
- Consumes: Tasks 1-3's landed artifacts + D-row roster.
- Produces: the permanent records; the issue/board hygiene; PR B merged.

- [ ] **Step 1: Land the register rows** — move the roster from the pack header into `docs/reference/bsl-language.rst`'s register with full file:line evidence (the D116-D145 rows' format is the model); replace the pack-header block with the one-line-per-row pointer form production.bsl uses.

- [ ] **Step 2: The taxonomy doc page** — in `docs/concepts/consciousness-taxonomy.rst` (read it first; match its section style):
  1. **A-001's one home**: the `consciousness/p0-position` + `consciousness/p9-dominant-worldview` rules + the closure's remainder branch are THE declared homes of the hegemonic-default rule; name the seven frozen sites (digest A.5c: `formulas/consciousness.py:77-79`, `:83-91`; `models/entities/consciousness.py:177-192`, `:76-78`; `formulas/consciousness_routing.py:396-398`, `:405-407`; `projection/aggregation.py:69-73`, `:203-206`) as not-ported-by-law.
  2. **UNPOSITIONED (L-ABS) on the class surface**: absence has no reading; the idiom's mechanics in one paragraph; the row-19 death certificate (the ≥10 frozen 0.5-default sites, digest A.6, named).
  3. **The hegemonic-community semantic inversion** (spec §8 discharge): on SETTLER/PATRIARCHAL communities the `r` share reads as the conscious defense of the extraction position — same math, inverted reading (spec 034 spec.md:178's order, now documented). Record that the inversion has NO frozen code site (digest A.5d — doc-law only) and its Rust code home arrives with the community-carrier port (hyperedge attributes — chartered).
  4. **W12 note**: this train lands the READ couplings (the ternary is computed from the value-flow estate — WAGES/EXPLOITATION-adjacent fields, the balance, SOLIDARITY topology); the PULL couplings (measured shares modulating value flow) are chartered port-stage design work, no new formalism.

- [ ] **Step 3: ADR207 + index.yaml** — the handoff record: scope landed (content + read path + update law), the three corrections to #588's "preserved laws transcribed verbatim" framing (evidence: the digest): (a) the f→r ε-gate (`consciousness_routing.py:474-511`) has NO frozen production caller — porting it is greenfield, not transcription; chartered as its own micro-train (its proletarianization signal needs a definition from ported state); (b) "r→f capacity transfer" names a cluster, not a construct — the router's no-solidarity Δf branch landed here; the FascistFaction pull/capture chain charters behind the FACTION vocabulary (#589, Director-gated) + node-ref fields; StruggleSystem revanchism charters behind the Struggle port; (c) the semantic inversion is doc-law — discharged as docs here. Chartered follow-ons roster: W5 mass-work verbs (org-verb/OODA machinery absent), community-surface port (hyperedge attributes + queries + overlap formula — native Rust + AE(ii) escalation), the gap readout + Curve-5 partition (rides #491), the W7 seeding spec, W12 pull couplings.

- [ ] **Step 4: `ai/state.yaml` entry** — one entry, PR numbers, ADR number, D-rows landed.

- [ ] **Step 5: Full gates**

Run: `mise run rust:check` && `mise run check` && `mise run qa:regression` && `mise run qa:vault-regression-ci`
Expected: all green; every pre-existing golden/vault page byte-identical. Any drift: STOP.

- [ ] **Step 6: Issue hygiene + PR B**

Comment on #588 with the split verdict (title scope — read path + UNPOSITIONED + update law — landed; W5-verb surface chartered as follow-on with the machinery gap named; link ADR207 + the digest) and propose closing #588 with the follow-ons filed as new issues (the W5-verb surface; the ε-gate micro-train; the community-carrier port note riding the existing hyperedge-gap record). Post the row-19 discharge evidence pointer on #564 (the UNPOSITIONED idiom + the death-certificate doc section). Open PR B, CI green, Copilot harvest, `mise run pr:merge -- <N>`.

---

## Self-review notes (plan author)

- **Spec coverage:** §4.2 (measured ternary, UNPOSITIONED, shares-only, gap readout) — Tasks 1/3 land the first three; the gap readout's position-side carrier (E/P/S quantile sketch, #491) is not landed, so the gap readout is chartered (Task 4, ADR roster) — recorded, not silently dropped. §4.3 (preserved laws) — routing + A-001 + closure land; ε-gate and the inversion are corrected-staging (Task 4 ADR). §4.5.2 (staging) — read path first (PR A), updates after (PR B), frozen reference-only, D-records, no golden movement without ceremony. W12 — read couplings land; pull couplings chartered. §8 — the inversion doc page discharges in Task 4 Step 2.3.
- **Placeholder scan:** every BSL rule is written in full; the four spikes name their authority files and fallback ladders; defines values are pinned with line cites; the two genuinely runtime-known spellings (edge-deffield declaration form, probability literal suffix) carry spike steps with authorities.
- **Type consistency:** field qnames, const qnames, and rule ids are uniform across Tasks 1-4 (`social-class/*` fields, `consciousness/*` consts and rule anchors); the anchor table's values derive from the same seed table the scenario declares.
