# TickDynamics @4.0 (Feature-016 Class-Dynamics Engine) — Task 0 Surface-Facts Dossier

**Date:** 2026-08-18. **Worktree:** `/media/user/data/worktrees/wt-tickdynamics`,
branch `feature/tickdynamics-port-bsl`, base `72a7e02b` (== `origin/dev` at measurement time,
verified zero-diff both directions). **Plan:** `docs/superpowers/plans/2026-08-18-tickdynamics-port.md`
(rev 2.1). **Issue:** #669 (implementation tracking, project 8). **Docket:** #564 (DG-1..DG-11
posted as a single comment, this session).

This dossier executes Task 0 of the plan: independent re-measurement of every claim the plan's
own text stakes a downstream task on, so later tasks build on re-verified facts rather than
inherited ones.

---

## 1. Confirmed findings

- **Base tail, independently re-measured:** `D180` (`docs/reference/bsl-language.rst:8158`),
  `ADR214` (`ai/decisions/index.yaml`, `ADR214_national_incidence_artifact_train.yaml`) — matches
  the plan's own Step-3 claim exactly.
- **`lib.rs:277-352`'s registered-systems `HashSet`** holds exactly **13 string literals**:
  `economics, vitality, consciousness, lifecycle, dispossession, metabolism, territory,
  organization, production, social-class, solidarity, decomposition, control-ratio`. **Zero**
  spelling of `tick-dynamics`/`tickdynamics`/`class-dynamics` appears. The corrected range
  `:277-351` (not rev 1's `:277-343`) is exact — line 351 is the closing `]);` (line 352 is
  blank; the task-0 review corrected this dossier's own off-by-one here).
- **`tick_goldens.rs` carries exactly 18 `#[test]` functions, 16 of them `*_hashes_are_pinned`.**
  The other 2 are `worldview_member_order_is_the_ruled_ordinal` and
  `worldview_prelude_member_order_is_the_ruled_ordinal`. (A naive `grep -c` for the pattern
  returns 17 because line 564's own comment contains the literal substring `` `fn .*hashes_are_pinned` ``
  — not a real function; the true count is 16, confirmed by listing every `fn` name.)
- **The 17 pre-existing pinned hashes, pasted verbatim as the byte-identity baseline:**

  | test | before | after |
  |---|---|---|
  | `two_classes_fundamental_theorem_hashes_are_pinned` | `5a44ab0c426eca240a0010cc70321bd0ff944d2eee2408454899a942dc85a205` | `783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679` |
  | `vitality_conformance_hashes_are_pinned` | `20dbc24fc6ba17067cb26eb4ce4c2792c51cb0402395dc55363a5e4e38572fea` | `4c7f95d967e2bf28cd5be91bbd439b61652d2c8d4103e8b5d7a3a8ad789baf64` |
  | `us_counties_lifecycle_demo_hashes_are_pinned` | `c190053e6d5d6eb261f1325bf87a6347dad8bb99f4e6fb7f2e297d355ccc28ab` | `f4ea98647520ca8e5b2b74e4970626a179236b48efde144c91850c52640f2b5d` |
  | `organization_foundation_hashes_are_pinned` | `5d8d5c43088440787f993ce91bd9a676d4adf60fa35904b2afbafeccaab93a1e` | `5d8d5c43088440787f993ce91bd9a676d4adf60fa35904b2afbafeccaab93a1e` (before==after: fires once, emit-only — no mutation) |
  | `territory_conformance_hashes_are_pinned` | `3794b114d302a8466889795573ecf3f87547af5c200e1ead11c4fc9fcac88ad6` | `510091298354429a755e6b851c9db356b2b1d7c35e74d092447535a7883e1af8` |
  | `production_conformance_hashes_are_pinned` | `83192431e51d9be36aea347cec0861ebe352e47ee8f9bce4f39840f3e581ad4b` | `1538162e443afd4b1dcc020bec886e616c91bc680dffce50e52d48df4af8f1eb` |
  | `worldview_foundation_hashes_are_pinned` | `098ef6bd62ebc072de94d370242430d84b1b8cf2223b3b190b359ed6e871edbf` | `098ef6bd62ebc072de94d370242430d84b1b8cf2223b3b190b359ed6e871edbf` (before==after) |
  | `consciousness_ternary_foundation_hashes_are_pinned` | `e2582dd4f3537a6baa26fdb273e9aaf39299ab4994cf0dcf2664a90b920821fe` | `4346278b3e075b338b5d4b847de054da6738d74f05895f8945dad78b13f46da9` |
  | `solidarity_conformance_hashes_are_pinned` | `20124f5ca91da3cb30fba41bc373175fdf3b06dc82f3c3b162da172951bb29de` | `978dbe30363c3b306bd7fa668e25d48de18c36b93930e9c4d195b5997ed67312` |
  | `decomposition_conformance_hashes_are_pinned` | `4001e15449fbf467624417f3c4a9cca22e27bdea3320c81669808c5940a7eb8a` | `6bcc49d18b1e2494adf96bada45425616b955373293494d314ecdf20679d9b0f` |
  | `decomposition_delay_conformance_hashes_are_pinned` | `40f0facb177fb535af415f99f70244663cc0ffe4fc26352efc91d308301f5e1e` | `0eaf7f1459559645510efd57c71739f3ef8813409f3944b9eba51492d141748b` |
  | `control_ratio_conformance_hashes_are_pinned` | `54f7a559a3c047561979994bd058460a3bd12ba361511117bb5227a32f4ad583` | `cececdab38bc6ba483baf60ee4df32cb4043073ce18fdd54ce9c866c922b6e5b` |
  | `control_ratio_revolution_conformance_hashes_are_pinned` | `af67a81e16e480adfc621e8617eb1edef99921a45e67b5544451d8f10edc4c1f` | `0ebd2a90c4868a84dd8547c5c37a99fd44cd612f2cbc53c06163847e7c34cb0a` |
  | `control_ratio_within_capacity_conformance_hashes_are_pinned` | `f4c8d6b0a12047e713ec3d995cb70f519a4136dadb852192116d237ecdb0834a` | `67aa4f7bfcc2ad807331354ea786001a6dc46a7ea5a7514c87ad963f90860470` |
  | `control_ratio_zero_enforcer_conformance_hashes_are_pinned` | `62f02edb2de87305b34ec7efd5b0a638929300a60ac8473aace3e9b86ccad100` | `897c1939b9f798026ddc41d9732b0b676a0b628f00b8a845a1c8261d5f725204` |
  | `carceral_arc_conformance_hashes_are_pinned` | `504a4515c4e6d4d4c369a535c58a21ab98e8ee37ba852819c7b4893473881e74` | `04b2a84623e25fdf7fd7761e3c591baa8b42aa96300c76b02caca59e0c74b3d6` |
  | `babylon-client/tests/engine_link.rs::startup_tick_matches_the_pinned_hash` (17th, another crate) | — (no `before` assertion) | `783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679` (same post-tick hash as row 1) |

- **The shared-file consumer enumeration (§2.2.1/Step 4b) re-confirmed exactly: 4 consumers, 2
  pinned hashes, `per_rule_fired.len() == 1`.** `rg -n 'fundamental-theorem' rust/ --glob '!*.md'`
  and `rg -n 'two-classes.bscn' rust/` surface exactly: `babylon-tick/tests/tick_goldens.rs`,
  `babylon-client/src/engine_link.rs`, `babylon-client/tests/engine_link.rs`,
  `babylon-tick/src/lib.rs`. No fifth consumer exists. `babylon-bsl/tests/fundamental_theorem_tick.rs`
  independently confirmed to carry its own inline rule/scenario text and use no `include_str!` of
  either file. `lib.rs` carries exactly the three unit tests the plan names, beginning at
  `:556`, `:564`, and `:576` (rev 1's `:556-569` span covered only the first two — corrected
  by the task-0 review):
  `run_once_is_deterministic`, `single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired`
  (`report.per_rule_fired.len() == 1`), `node_content_ids_reach_prepared_rules_through_the_real_wiring_seam`.
- **Owed re-reads (Step 5), verbatim, re-verified:**
  - (a) `tick.rs:456` `:year` refusal text and `:462` `:tick-of-year` refusal text match the
    plan's quotes exactly. `bindings.rs:55-59` (`TickInCycle(i64)` doc) and `:410-416`
    (`"tick-in-cycle" => …`) confirmed.
  - (b) `typecheck.rs:130-236` — the five `FoldOp` arms (`Sum`/`Mean`/`Min`/`Max`/`Count`),
    `UnweightedMeanOfIntensive` (`:183`), `NonExtensiveWeight` (`:192`), and
    `destructure_aggregation`'s `(op field)` / `(op field :weight weight-field)` shapes
    (`:213-236`) all confirmed at the byte.
  - (c) `rule_pipeline.rs:640-708`'s `field_ref_for` doc comment literally states **"Three shapes
    reduce"** (bare qname, `field-of`, binding-resolved-through-source including `:expr`
    recursion) — the code additionally special-cases a 4th match arm, a nested fold
    (`:684-701`), which is what the plan's "four-shape law" phrasing counts. `:744-760`'s weight
    adapter (`field_ref_for` run over `:weight` exactly as over the body) and
    `compound_fold_error()` on either `None` confirmed.
  - (d) `scenario.rs:1093-1330` — `load_deffield`'s `int`/`enum` 4th-slot dispatch and
    `load_node`'s attribute loop confirmed; `attribute_value`'s match (`:1314-1321`) has
    **exactly 5 arms** (`Int`, `Real`, `Probability|Intensity|Coefficient` combined, `Currency`
    refused, `Enum`) matching §4.3's seeding table row-for-row. **`:1236-1275`'s node hydration
    loop confirmed as the proof §2.2.3 cites**: the `for attr in attrs` loop iterates only the
    scenario-authored `(field value)` pairs actually written on a `(node …)` form; there is no
    companion loop over the full `declared` field-decl map, so a declared-but-unseeded field is
    never stamped. **`load_defconst` is at `scenario.rs:731`, outside the cited `1093-1330`
    range** — a citation-accuracy note, not a content defect (its own arms — `Int`,
    `Scaled(Ratio)`, `Scaled(p/i/c)`, `Bool`, `Currency` refused — independently confirmed).
  - (e) `grammar.rs:672-683` (`pub enum FoldOp`, 5 variants) confirmed exact. `:724`
    (`const ARITH: [&str; 4] = ["+", "-", "*", "/"]`) confirmed exact. **`"if"`'s arity row is
    actually at `grammar.rs:650`, not `:649`** — `:649` is `"domain"`'s row, immediately
    preceding `"if"`'s in the same `OPERATOR_ARITY`-shaped array. One-line citation
    inaccuracy, content (`("if", 3, 3, "exactly 3")`) correct.
  - (f) `reader.rs:869-886` (`classify_unit_interval`) — the 9-fractional-digit cap
    (`ExcessScale`) and the `[0,1]` bound (`UnitIntervalOutOfRange`) confirmed exact.
  - (g) `declarations.rs:125` = `DECLARABLE_INTRINSICS: [&str; 4] = ["exp", "log", "floor",
    "rng-draw"]`; `:131` = `PROHIBITED_INTRINSIC_NAMES: [&str; 1] = ["sigmoid"]` — **both exact,
    confirming rev 2's `:131` measurement over rev 1's `:132`.** Both `(intrinsic floor …)`
    declarations confirmed byte-identical: `territory.bsl:78` and `decomposition.bsl:212`, both
    `(intrinsic floor :params (real) :returns int :cost 5)`.
  - (h) `bindings.rs:448-451` doc (`:optional` licenses absence of a VALUE, never an unknown
    NAME) and `tick.rs:439-451` (`check_sources_servable`, the `BindSource::Const(qname) if
    !defines.contains_key(qname)` arm) both confirmed exact.
  - (i) `state_hash.rs:10-30`'s canonical byte layout (sections `0x01`-`0x05`, big-endian,
    length-prefixed strings) quoted and confirmed exact — this is the evidence §2.2.3 rests on.
  - (j) `evaluator.rs:139-147` — `E-EVAL-020` (`StoreRangeViolation`) and `E-EVAL-021`
    (`EmptyAggregate`, covering both empty fold and empty `select-max`/`select-min`) confirmed
    exact. `territory.bsl:168-172`'s `exists`-then-`fold` protector idiom confirmed present and
    matches the cited pattern.

---

## 2. CORRECTIONS — §9's eleven "NOT blockers", each independently re-verified

1. **Landed content folds over `(nodes …)`** — `decomposition.bsl:284-291`,
   `control-ratio.bsl:281-287` both confirmed present at those files. A 2026-08-17 draft's "no
   landed content folds over `(nodes …)`" claim is **stale**.
2. **Fractional per-node seeding IS legal** — `scenario.rs`'s seeding arms (§1(d) above) confirm
   `probability`/`intensity`/`coefficient`-declared fields accept `p`/`i`/`c`-scaled literals
   directly on a `(node …)` form. `dispossession.bsl:29-40`'s header claim that "there is no
   legal way to seed a genuinely fractional per-node value" is **stale**.
3. **D102 discharges `field-of` over an enum field** — confirmed by `load_deffield`'s
   `ty == "enum"` branch (§1(d)); `territory/crisis-phase` may be a real `enum CrisisPhase`.
4. **The string-identity gap does not bind** — this pack drops FIPS entirely (county is the node
   identity, not a string key); the source does not independently re-derive this claim, taken
   from the plan's own §4.1 argument, consistent with everything else measured.
5. **This train can observe and pin events today** — the source does not independently
   re-verify this against `CollectingSink` this session (out of Task 0's re-read list); no
   contrary evidence found.
6. **`EventType` needs no Rust change** — `rg -n 'defvocabulary EventType' rust/crates/babylon-tick/content/`
   returns zero hits, consistent with the plan's claim that no landed `.bscn` declares one.
7. **`validation.py` carries 32 constants, not 29`** — independently re-counted at the byte:
   `validation.py:29-54` (20 rate thresholds) + `:60-73` (12 share thresholds) = **32**, not the
   trio dossier's "29" (17+12). The `70`-row disposition arithmetic (§1.6/§4.5 of the plan) is
   internally consistent with 32.
8. **`rule_pipeline.rs` serves the `:weight` operand shape for `field-of`** — confirmed at
   §1(c)/(b) above: `rule_pipeline.rs:744-760` runs `field_ref_for` over `:weight` through the
   identical adapter as the fold body. This finding retires BLOCKER-6; it carries no live risk.
9. **Tick 0 is never executed; the first boundary is tick 52** — independently confirmed at the
   byte this session (beyond Task 0's own owed-re-read list, since this fact underwrites the
   whole pin design): `session.rs`'s `TickSession::new`/`new_with_prelude` construct with
   `tick: 0`, and `advance()` computes `let next_tick = self.tick + 1`, so a session's first
   `advance()` call executes tick 1, never tick 0. `lib.rs::run_prepared_tick` passes the literal
   `1` to `run_tick`, with its own comment naming *"the same number the CLI has always
   printed."* Combined with the `:tick-in-cycle` mechanism confirmed at §1(a) (`1 % 52 = 1 ≠ 0`),
   the pack's first boundary is tick 52 under either driver.
10. **No landed `.bscn` declares an `EventType` vocabulary** — same grep as item 6, zero hits.
11. **`production.bsl` contains no fold at all, and the double-count record is D136, not D45`** —
    not independently re-read this session; no contrary evidence found. (Separately,
    `evaluator.rs:143-146` cites "D45" for `EmptyAggregate`'s `select-max`/`select-min` reasoning
    — a *different* D45 usage from the territory-side-fold tiebreak the plan's §2.4 discusses;
    the two references are not in conflict, just two distinct facts sharing one D-number.)

---

## 3. Type-trap list (Task-1-relevant, verified against source this session)

- **`Int ÷ Int` is a loud error** (truncation never implicit) — consistent with `evaluator.rs`'s
  documented range-check discipline (§1(j)); the specific division-error site (`:35`, `:1739`)
  was not re-read this session but nothing found contradicts it.
- **`if` takes exactly three operands, both branches share one static type** — confirmed:
  `grammar.rs:650` (measured location, see §1(e) correction) `("if", 3, 3, "exactly 3")`.
- **The fold element name is the implicit `it`** — consistent with `territory.bsl:168-172`'s
  `(field-of it territory/heat)` usage confirmed at §1(j).
- **`E-LEX-023` caps `p`/`i`/`c`/`r` at 9 fractional digits** — confirmed at `reader.rs:870-876`
  (§1(f)): `if frac_digits.len() > 9 { … ExcessScale … }`.
- **`update-node`'s op set is the closed `add | sub | set | scale`** — confirmed at
  `grammar.rs:719`: `const UPDATE_OPS: [&str; 4] = ["add", "sub", "set", "scale"]`.
- **No `abs`, no `round`, no `min`/`max` scalar intrinsic, no `%`** — consistent with
  `DECLARABLE_INTRINSICS` (§1(g)) being the closed 4-member set `exp, log, floor, rng-draw`, none
  of which is `round`/`abs`/`min`/`max`, and `ARITH` (§1(e)) being the closed 4-member set
  `+ - * /` with no modulo.

---

## 4. The `round()` census (Step 2), independently re-verified

`rg -n '\bround\(' src/babylon/domain/economics/` across all seven `dynamics/` modules **and**
`tick/system/__init__.py` returns:

| site | classification |
|---|---|
| `tick/system/__init__.py:1164` (`cumulative_la_decline`) | presentation — `DISPOSSESSION_CASCADE` event payload |
| `tick/system/__init__.py:1166` (`current_la_share`) | presentation — same event payload |
| `tick/system/__init__.py:1167` (`baseline_la_share`) | presentation — same event payload |
| `tick/system/__init__.py:2336` (`score`) | presentation — `BIFURCATION_THRESHOLD` event payload |
| `tick/system/__init__.py:2338` (`solidarity_density`) | presentation — same event payload |
| `tick/system/__init__.py:2339` (`legitimation`) | presentation — same event payload |
| `tick/system/__init__.py:2340` (`class_burden_ratio`) | presentation — same event payload |

**Zero hits inside any of the seven `dynamics/` modules.** All 7 in-tree sites are `Event(...,
payload={...})` construction arguments — terminal output for narrative/observation, never fed
back into `CountyEconomicState`/`ClassDistribution` or any downstream computation. **All 7 are
presentation, zero are state-affecting.**

Cross-checked against the plan's separately-cited residual-train sites:
`reserve_army/accumulation.py:115` (`mechanization_displacement = round(...)`) and `:121`
(`firm_failures = round(...)`) — both integer demotions feeding directly into returned state
(`_FlowResult`), confirmed **state-affecting**, and confirmed **outside this pack's §0.1
boundary** (residual ReserveArmy @5.0 train, not Feature-016).

**Total: 9 `round()` sites tree-wide relevant to this train's boundary question — 7
presentation (in scope, harmless), 2 state-affecting (out of scope). In-scope state-affecting
count: zero.** BLOCKER-3's "resolved by scope" disposition holds; no STOP triggered.

---

## 5. Numbering allocation + contention (Step 3), independently re-measured 2026-08-18

**Base tail (this worktree, == `origin/dev`):** `D180` (`docs/reference/bsl-language.rst:8158`),
`ADR214` (`ai/decisions/index.yaml`).

**This train's fixed allocation, per the plan's own formula (`D<tail+1>…D<tail+32>`,
`ADR<tail+1>`):**

> **D181 – D212, and ADR215 — PROVISIONAL, re-measured at Task 12 immediately before filing. Not
> a literal to write into any committed content before then.**

**The contention, independently re-verified in each sibling worktree (not merely inherited from
the brief):**

| claimant | worktree / branch | measured state (2026-08-18, this session) |
|---|---|---|
| **#491** (rung/ladder) | `wt-491`, `feature/491-rung-ladder` | **COMMITTED** `D181, D182, D183, D184` in `docs/reference/bsl-language.rst` (four rows — the brief's "D181-D183" is itself now stale by one row; `ai/state.yaml:4702` lists all four). **Working tree** (uncommitted) adds `ai/decisions/ADR216_kind_straddle_repair_ceremony.yaml`, whose own header text records: *"ADR215 is claimed by the not-yet-merged feature/641-entity-voice branch, ADR214 landed on origin/dev during this same session... this document lands as ADR216, confirmed next-free against origin/dev at filing time."* |
| **ImperialRent** | `wt-imperialrent`, `feature/imperialrent-port-bsl` (8 commits ahead of `origin/dev`) | Plan text (`docs/superpowers/plans/2026-08-18-imperialrent-port.md`) claims literal **`D180`–`D201`** (21 own rows, D181-D201) and, in its Task-0 section, still names `ADR214_imperial_rent_port_handoff.yaml` in most places (`:14,557,571,1025,1057,1437,2014,2141` all still read `ADR214`) — **despite** a dedicated correction commit `243e8ad2` ("T0 review corrections — ADR215 re-anchor") whose body states *"naming ADR215 as this train's number as of 2026-08-18"* — the plan body was never fully swept to match its own correction commit. **Zero of these D-rows are yet committed to `docs/reference/bsl-language.rst`** (only the shared `D180` appears there) — the literal range exists only in plan prose so far. |
| **#641** (Entity's voice) | `wt-641`, `feature/641-entity-voice`, PR #668 | **COMMITTED** `ADR215_entity_voice_suite.yaml` on its own branch (commit `6efc678e`). **PR #668 is OPEN, `mergedAt: null`** — not yet landed on `origin/dev` as of this measurement, despite the brief's framing ("just LANDED... PR #668, merging"). |
| **Community** | `wt-community`, `feature/community-port-bsl` | Plan (`docs/superpowers/plans/2026-08-18-community-port.md`) uses **relative** placeholders correctly: exactly **25** distinct `D-NF+1`…`D-NF+25` rows and **13** `ADR-NF` references (one ADR). No literal collision — this is the NEXT-FREE-AT-LANDING pattern working as intended. |

**Net reading:** this train's naive `tail+1` computation (`D181`, `ADR215`) collides with #491's
already-committed `D181-D184` and with #641's already-committed (but not yet merged) `ADR215`; it
also overlaps ImperialRent's claimed-but-not-yet-committed `D181-D201` range and inconsistent
`ADR214`/`ADR215` self-citation. **This dossier does not resolve any of it** — per the plan's own
law, every number this train writes stays `D-NF`/`ADR-NF` until Task 12's own re-measurement
against whatever has actually landed on `origin/dev` by then.

---

## 6. Cross-train collision grep result (Step 4 + Step 4b)

**Zero hits, confirming the precondition.**

- `rg -n 'ternary-net-fascist' rust/crates/babylon-tick/content/` → 0 hits.
- Each of the 23 `territory/*` qnames named in the plan's §4.2/§4.2.1 field roster, grepped
  individually across `rust/crates/babylon-tick/content/` → 0 hits for every one.
- The same two greps repeated against `/media/user/data/worktrees/wt-imperialrent/docs/superpowers/plans/`
  and `/media/user/data/worktrees/wt-community/docs/superpowers/plans/` → 0 hits.
- **Step 4b, the shared-file consumer enumeration:** re-run at this HEAD, `rg -n
  'fundamental-theorem' rust/ --glob '!*.md'` and `rg -n 'two-classes.bscn' rust/` surface
  exactly the same 4 consumers the plan's §2.2.1 table lists (`tick_goldens.rs`,
  `babylon-client/src/engine_link.rs`, `babylon-client/tests/engine_link.rs`,
  `babylon-tick/src/lib.rs`) — no consumer this plan does not already list. See §1 above for the
  full pin/test inventory those four consumers carry.

---

## 7. §0.2's Checkpoint-A roster table, measured

| system | position | status at this HEAD (measured 2026-08-18) |
|---|---|---|
| Vitality, Territory, Production, Lifecycle, Solidarity, Dispossession, Decomposition, ControlRatio, Metabolism | 1,2,3,7,8,10,11,12,13 | **PORTED** (9) — all 9 confirmed present in `lib.rs:277-352`'s registered-systems set (§1 above) |
| Community | 6.0 | **IN FLIGHT** — `wt-community`, `feature/community-port-bsl`, not yet merged |
| ImperialRent | 9.0 | **IN FLIGHT** — `wt-imperialrent`, `feature/imperialrent-port-bsl`, 8 commits ahead of `origin/dev`, not yet merged |
| **ReserveArmy** | **5.0** | **UNPORTED, UNSTARTED** — no `reserve-army`/`reserve_army` spelling in `lib.rs`'s registered-systems set; no plan doc found under any sibling worktree's `docs/superpowers/plans/` |
| **TickDynamics** | **4.0** | **THIS TRAIN — Feature-016 (class-dynamics engine) only, per §0.1's scope ruling; Task 11 Step 4 names the residual @4.0 computations rather than dropping them silently** |

**The registered set holds 13 distinct namespace strings total; 9 are Material Base ports and 4 are not**
(`economics`, `consciousness`, `organization`, `social-class` — each a driver-scaffolding entry
for a different train's fixture vectors, not a system port). **This train does not close
Checkpoint A.** WS3 stays HELD per ADR208 R14's own words, quoted in the plan and reiterated on
the docket comment posted this session (§10 DG-10).

---

## 8. Step 6 — the starting-line gate

**Static evidence (source-read, no cargo required) is complete and recorded in §1 above**: 18
`#[test]` / 16 `*_hashes_are_pinned` in `tick_goldens.rs`, all 17 pinned hashes pasted, the
13-string registered-systems list verbatim, the 4-consumer/2-pin/`per_rule_fired.len()==1`
shared-file facts.

**The executable half ran once `/proc/loadavg` dropped below the 24 threshold** (it measured
between 21.4 and 74+ for the first ~40 minutes of this session, sibling worktrees `wt-b3`
(~50-60 processes) and `wt-491` sat mid-build in their own `rust/` trees for sustained periods; a
background poll at 120s intervals tracked the load and sibling-process count until both cleared).
**`mise run rust:check` (single-flight) exited 0**: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --locked -- -D warnings -D clippy::cognitive_complexity`,
`cargo test --workspace --locked`, the three per-crate `-D clippy::pedantic` legs
(`babylon-kernel`, `babylon-bsl`, `babylon-graph`), and `cargo doc --workspace --no-deps --locked`
all passed clean — zero `FAILED`, zero `error[` anywhere in the run's output. **All 18
`tick_goldens.rs` tests passed, individually confirmed by name** (including all 16
`*_hashes_are_pinned` tests), and `babylon-client/tests/engine_link.rs`'s
`startup_tick_matches_the_pinned_hash` passed within that workspace run. **The explicit
crate-scoped leg `cargo test -p babylon-client --test engine_link` was then run separately and
also passed** (`test startup_tick_matches_the_pinned_hash ... ok`, 1 passed, 0 failed). All 17
pre-existing pinned hashes hold byte-identical at this starting line.

---

## 9. STOP conditions checked — none triggered

- §Step 2 STOP condition (an in-scope state-affecting `round()` site): **not triggered** — zero
  in-scope state-affecting sites (§4).
- §Step 4 STOP condition (a qname or `ternary-net-fascist` collision): **not triggered** — zero
  hits everywhere checked (§6).
- §Step 4b STOP condition (an unlisted `fundamental-theorem.bsl`/`two-classes.bscn` consumer):
  **not triggered** — exactly the 4 consumers the plan already lists (§6).

---

## 10. SPIKE RESULTS (Task 1, dated 2026-08-18)

Task 1's own spike, executed against temporary content (`content/rules/class-dynamics.bsl`,
`content/scenarios/class-dynamics-spike.bscn`, `tests/class_dynamics_spike.rs` — all three
deleted at Step 6; this section is the authoritative surviving record. **The pack-header copy of
this block lands when Task 2 creates the real `class-dynamics.bsl` — there is no pack file to
carry it today.**) Rule ids used the already-registered `social-class`/`territory` namespaces
(babylon-tick's `lib.rs` registered-systems set) rather than `class-dynamics` — Task 1 touches no
production code; `class-dynamics` registration is Task 2's own job. All 6 spike tests passed;
verbatim evidence below.

**Three CONFIRMATIONS (source-answered, cited first — never upgraded back into open questions):**

1. **Step 1 — BLOCKER-6, retired (I5).** Source: `rule_pipeline.rs:744-760` reduces the `:weight`
   operand through `field_ref_for` exactly as the fold body. Loaded a rule with
   `(fold mean (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth) :weight
   (field-of it social-class/population))` — **loads and fires** (`fired == 2`, both class-a and
   class-b). No fallback branch, no two-`fold sum` alternative needed. **Verdict: CONFIRMED.**
2. **Step 2 — BLOCKER-7, retired.** Source: `session.rs:60-66,120-124` (`TickSession::new` starts
   `tick: 0`; `advance` computes `next_tick = self.tick + 1`) and `lib.rs:517-531`
   (`run_once`/`run_prepared_tick` hard-code tick `1`) — tick 0 never executes, the first boundary
   is tick 52. Executed a `(binding phase-of-year :tick-in-cycle 52)` +
   `(when (= phase-of-year 0))` rule over a 105-tick `TickSession`. **Measured `fired` series
   (ticks 1..=105, this rule's own `per_rule_fired` count each tick):** zero for ticks 1-51,
   **2 at tick 52**, zero for ticks 53-103, **2 at tick 104**, zero for tick 105 — exactly:
   `[0×51, 2, 0×51, 2, 0]`. This is the arithmetic every world's boundary pin inherits.
   **Verdict: CONFIRMED.**
3. **Step 5 — M11, already answered.** Source: `rg -n 'defvocabulary EventType'
   rust/crates/babylon-tick/content/` returns zero hits — no landed `.bscn` opts into an
   `EventType` vocabulary. Executed `(emit EventType/DISPOSSESSION_CASCADE (fips 1)
   (decline 0.05c))` verbatim (the brief's own literal), no `defvocabulary EventType` declared
   anywhere in the spike scenario. **Loaded and fired clean** — 2 `DISPOSSESSION_CASCADE` events
   (one per SOCIAL_CLASS subject, both carrying the same literal payload), `fips` read back as
   `Value::Int(1)`, `decline` read back as `Value::Real` with `.to_bits() == 0.05_f64.to_bits()`
   bit-exact. **Verdict: CONFIRMED — a world need not opt in.**

**Four REAL spikes (actually run; observed evidence recorded verbatim):**

4. **Step 2b (NEW) — the session-driven golden convention.** Drove `TickSession::advance` ×52
   over a throwaway world (the same content set as Step 2/5b), read `hex(&report.before)`/
   `hex(&report.after)` at tick 52. **Same-process stability:** two independent, freshly
   constructed `TickSession`s (same content, same `SessionId`) produced byte-identical tick-52
   hashes:
   `before=5c2b5dbc01023ab10a1aa38115d8e8b1bb45009c1a99c1c00f799ffaa48a7af1`,
   `after=b666db12f811dfad8757240335ddfe22accb823c9ac102235613d9e29d5b9bac`.
   **Cross-process stability:** ran `cargo test -p babylon-tick --test class_dynamics_spike
   step2_boundary_series_step2b_session_golden_and_step5b_positive -- --nocapture` as two
   genuinely separate OS process invocations, each capturing the printed tick-52 hash lines to a
   file; `diff`'d byte for byte — **identical, zero diff output** across both processes. This is
   exactly the shape Task 6 Step 4 will land as the file's first multi-tick pin. **Verdict:
   SPIKED AND PROVEN — the mechanism is safe to build the real pin on.**
5. **Step 3 — BLOCKER-2, in the legal form.** `(defconst class-dynamics/deep-precaritization-x1e6
   3500000)` bound as the defines-environment coefficient `class-dynamics/deep-precaritization-x1e6`,
   promoted before the divide with `(binding m :expr (/ (- deep-precaritization-x1e6 0c)
   1000000))` (the brief's
   own verbatim spelling — `Int ÷ Int` is a loud error, rev 1's spelling could not have loaded,
   I9). **Measured: `m` reads back bit-exactly `3.5_f64`** (3500000/1000000 is a terminating
   binary fraction, so this promote-then-divide order introduces ZERO rounding of its own —
   confirmed by `m.to_bits() == 3.5_f64.to_bits()`, not merely `m == 3.5`). **`product` (= `m *
   rate`) reads back BIT-EXACTLY equal to the mirror** — computed as `m_readback * rate_readback`
   natively in Rust from the SAME f64 values the engine itself emitted (`product.to_bits() ==
   (m*rate).to_bits()`), never re-derived from an independent decimal-string parse (which is not
   guaranteed bit-identical to the engine's own `unscaled as f64 / 10^scale` conversion — the
   cross-implementation-tolerance discipline). The value reads back bit-exact — this spike needed
   no fallback (a different scale, or a declared tolerance).
   **The deliberate operand-order deviation from `metabolism.bsl:386-387`, with its
   one-rounding-vs-two derivation:** metabolism's D-1 pattern multiplies a per-node, generically
   inexact field value by the scaled-Int constant FIRST (`raw-extraction * entropy-factor-x1e6`),
   THEN divides by `1000000` — two operations, each rounding a generically-inexact intermediate
   (metabolism's own header measures up to 2^15 ULP of divergence from the frozen engine as a
   consequence). THIS spike's order instead descales the **constant alone** first — a division of
   two exact integers-as-reals whose quotient terminates in binary here (`3500000/1000000 = 3.5`
   exactly) — before the result ever touches per-node data. Only the FINAL multiply by `rate` then
   rounds: **one rounding, in general, for THIS operand order, versus metabolism's generically
   two.** This is a property of *this specific pattern* (divide the constant alone, independent of
   any field read, before combining with anything else) — it does **not** generalize to every
   scaled-Int constant: a constant whose descaled decimal has no exact base-2 expansion would still
   round at the constant-alone division step, so a future author reusing this pattern should not
   assume zero-rounding without checking their own constant the same way this spike checked
   `3500000`. **Verdict: SPIKED — bit-exact for this constant, order documented, no fallback
   needed.**
6. **Step 4 — D102's discharge (enum seed + read pair).** Seeded `(node county-deep
   NodeType/TERRITORY (territory/crisis-phase CrisisPhase/DEEP) …)` (the seeding path,
   `E-LOAD-056`'s member-only rule — the scenario loads clean, confirming the seed is legal) and
   read it back via `(binding phase :field territory/crisis-phase)` +
   `(if (= phase CrisisPhase/DEEP) …)` (the read path). **Measured:** the rule fires for
   `county-deep` alone (`fired == 1`; `county-tenanted`/`county-empty` are both `NORMAL` and do not
   fire), and the emitted `phase` payload value reads back as `Value::Enum { enum_type:
   "CrisisPhase", member: "DEEP" }`. **`defenum` ordinal parity (hash-bearing, ADR195), asserted
   directly against the registry:** `NORMAL=0, ONSET=1, EARLY=2, DEEP=3, RECOVERY=4` — declaration
   order is the storage ordinal, confirmed exact. **Verdict: SPIKED AND CONFIRMED.**
7. **Step 5b (NEW) — the empty-TENANCY fold protector (C5).** `county-empty` is a TERRITORY with
   **zero** incoming-TENANCY SOCIAL_CLASS neighbours.
   **WITHOUT the `exists` protector** (isolated inline rule, never co-loaded with anything else —
   it kills every tick of any content set holding it): ran tick 1 (`(when (= phase-of-year 0))` is
   **false** at tick 1, `phase-of-year == 1`). **The tick still died.** Exact verbatim error text:
   ```
   tick failed in rule territory/spike-step5b-without-protector: E-EVAL-021: mean over an empty
   query (§4.4) — there is no element to average
   ```
   This is the whole point: bindings evaluate before the `when` guard, so the boundary gate
   protects nothing against an empty fold.
   **WITH the protector** (`territory.bsl:168-172`'s `exists` idiom copied verbatim ahead of the
   fold, plus a guarded write — `(guard has-classes (update-node self territory/spike-score (set
   score)))`): drove the SAME content set to tick 52. **The tick survived.**
   `county-tenanted/territory/spike-score` became **220.0** exactly
   (`(100*40 + 300*60)/(40+60)`, the population-weighted mean, confirmed via `assert_eq!` against
   the raw f64 value the tick itself wrote). `county-empty/territory/spike-score` **stayed at its
   42 sentinel,
   untouched** — no score at all, not a fabricated zero (III.11). **Verdict: SPIKED AND PROVEN —
   this ONE refusal is the evidence for the C5 repair; without it the protector would read as
   defensive decoration.**

**One INCIDENTAL FINDING (not one of the six named spikes, but load-bearing for Task 8's
`a12`/`a13` and every future emit-only/constant-only/expr-only rule in this pack):**
`(domain NodeType/…)` is **not** what `run_tick` uses to pick a rule's subject population at this
engine slice. `babylon-bsl/src/tick.rs::subject_type_of` derives the subject type from the rule's
own `:field` bindings ALONE — it does not consult an explicit `(domain …)` declaration,
does not look at `update-node` targets, and does not look at `field-of self …` accessors. A rule
declaring `(domain NodeType/SOCIAL_CLASS)` with zero `:field` bindings refuses **at run time**
(not load time) with: `"the rule declares no :field binding, so it names no subject type — slice 1
runs rules over a population, not over the graph as a whole"` — confirmed empirically this task
(Steps 1/2/3/5's first draft all hit this exactly). This independently reproduces
`metabolism.bsl`'s own D-4 finding word for word (`"tick.rs::run_tick NEVER reads loaded.domain"`)
from the content-author's side rather than the engine-reader's. **Consequence for this train's own
plan text:** `a13`, **as literally quoted in the plan
(`docs/superpowers/plans/2026-08-18-tickdynamics-port.md:1137`), carries ZERO `:field` bindings**
— both `has-classes` and `score` are `:expr`-sourced, and `a13`'s own `update-node` target
(`territory/bifurcation-score`) does **not** count toward subject-type inference either. **As
literally written, `a13` would refuse at run time with the same "no :field binding" error the
protector spike above hit before its own fix.** Task 8 must add a genuine self-scoped `:field`
binding to `a13` (any already-declared `territory/*` field the rule can legitimately read, even if
only to anchor the subject type) before landing it — this spike's own `spike-step5b-with-protector`
rule (`(binding subject-anchor :field territory/crisis-phase)`, unused beyond anchoring) is the
minimal worked fix, and `crisis-phase` is not a `class-dynamics`-owned field so `a13`'s own author
should pick a real anchor from among its OWN pack's `territory/*` fields instead (e.g. whichever
field this train's plan already has `a13` reading for real, or a field `a01`-`a11` already publish
onto `territory/*`, if any exist — worth checking at Task 8 time, not re-derived here).

**Step 6 discharged:** this task's own commit deletes every spike artifact (the temporary
`.bsl`/`.bscn`/`tests/*.rs` triple) — see the commit message for the disclosed test-support delta
this spike legitimately needed (none beyond the three deleted files: no production code changed).
