# WorldView Vocabulary Ceremony Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Mint the `WorldView` defenum (REVOLUTIONARY / LIBERAL / FASCIST) into the Rust closed BSL vocabulary via the ADR195/196 ceremony machinery, pinned by a new tick golden — the second content-enum consumer of ADR195's `enum` deffield row, per ADR204 W10.

**Architecture:** A pure content mint, following the OrgKind precedent (PR #550): one new conformance scenario `.bscn` carrying the `defenum`, one comment-only rules pack, one new pinning test in `tick_goldens.rs`, and the mint-and-retire record ADR (format per ADR176 (34)/ADR187 OQ-7, executed shape per ADR196). Zero Rust source changes — content enums are data-driven through `EnumRegistry`/`parse_defenum`.

**Tech Stack:** Rust workspace (`rust/crates/babylon-tick` content + tests), BSL scenario dialect (`.bscn`/`.bsl`), ADR yaml estate (`ai/decisions/`).

**Spec:** `docs/superpowers/specs/2026-08-14-worldview-ternary-unification-design.md` (§4.1 the mint content, §4.5.1 the ceremony scope, §5 the reserved ledger). Archaeology digest with every mechanical fact verified line-by-line: `ai/scratch/2026-08-15-worldview-ceremony-archaeology.md`.

## Global Constraints

- **Ruled member order, verbatim:** `REVOLUTIONARY / LIBERAL / FASCIST` (design doc §4.1, ADR204 W1–W12). Declaration order IS the storage ordinal (ADR195), so this order is load-bearing: REVOLUTIONARY=0, LIBERAL=1, FASCIST=2. The frozen Python enum order (LIBERAL, FASCIST, REVOLUTIONARY — `src/babylon/models/enums/consciousness.py:83-85`) is a DIFFERENT fact and does not govern the Rust mint.
- **Pole names are Director-reserved content (§5)** — already ruled; the mint transcribes, never invents.
- The faction-classification enum (W9) is NOT in this ceremony — the Director has not ruled its member list content-complete; it charters separately.
- **Nothing retires in this ceremony.** The ADR176 (34)/ADR187 OQ-7 citation is to the record FORMAT. W11's `Ideology`-scalar strike already landed in ADR204's own train (dev, `ai/THE_FORMALISM.md`).
- No frozen-Python edits (Amendment AE). No new deffield row (ADR195's `enum` row covers it). No `bsl-language.rst`/EBNF/grammar change (pure content mint mints no D-rows). No `tests/baselines/**` touch — no §6.5 ceremony owed.
- Lexical law (`reader.rs` §1.4): the type name `WorldView` matches `UPPER (UPPER|LOWER|DIGIT)*` (no underscore — `World_View` refuses at lex); members match `UPPER (UPPER|DIGIT|"_")*` (all three conform).
- **Loader law (discovered at execution, plan amended):** the rule pipeline refuses a zero-rule content set (`rule_pipeline.rs:381-388` — "a content set needs at least one (rule …) top-form"). The established idiom for a load-only smoke is a NEVER-FIRING probe rule (`production_conformance.rs:76-110`, `territory_conformance.rs`'s no-op rule). The mint pack therefore carries one probe, anchored under the ALREADY-registered `consciousness` namespace (`lib.rs:224` — no Rust source change; the worldview estate is the consciousness domain's content), guarded false so `fired == 0` and `before == after` still hold by construction.
- Branch from `dev`; Conventional Commits; merge only via `mise run pr:merge`; verification gate is `mise run rust:check` (the pre-push suite now runs it automatically for `rust/`-touching pushes).

**Setup (before Task 1):** `git fetch origin dev && git checkout -b feat/worldview-vocabulary-ceremony origin/dev`

---

### Task 1: The mint content — scenario, rules pack, golden pin

**Files:**
- Create: `rust/crates/babylon-tick/content/scenarios/worldview-foundation.bscn`
- Create: `rust/crates/babylon-tick/content/rules/worldview.bsl`
- Test: `rust/crates/babylon-tick/tests/tick_goldens.rs` (append)

**Interfaces:**
- Consumes: `babylon_tick::{run_once, hex}`; the `.bscn` top-forms `defvocabulary`/`defenum`/`node` (machinery landed: `scenario.rs:768` `load_defenum`, `scenario.rs:811` `load_defvocabulary`).
- Produces: the `WorldView` enum registered in the crate's content-enum surface; the `worldview_foundation_hashes_are_pinned` golden the port train extends.

- [ ] **Step 1: Write the scenario (the mint itself)**

Create `rust/crates/babylon-tick/content/scenarios/worldview-foundation.bscn` with exactly this content:

```lisp
; The WorldView mint — the worldview-ternary unification's (ADR204, W10)
; first Rust content: the political simplex's three vertices as a
; content-declared closed enum, the SECOND consumer of ADR195's `enum`
; deffield row (after ADR196's OrgKind).
;
; Member order is RULED content (design doc §4.1, Director-approved
; 2026-08-14): REVOLUTIONARY / LIBERAL / FASCIST. Declaration order IS
; the storage ordinal (ADR195), so the ruled order makes REVOLUTIONARY=0,
; LIBERAL=1, FASCIST=2. The frozen Python enum's order (LIBERAL, FASCIST,
; REVOLUTIONARY — models/enums/consciousness.py:83-85) is a different,
; frozen fact and does NOT govern here.
;
; Asymmetric payloads (W3), declared here as the members' natures:
; REVOLUTIONARY is the articulated pole (its content home is the doctrine
; tree — the line IS the world view); LIBERAL is hegemonic common sense
; (no tree; content computed per conjuncture from the ruling bloc);
; FASCIST is the capture/degeneration terminus (no tree; parasitism-
; defense affect keyed to the rent gradient, plus a demagogy flag).
;
; This mint declares NO enum deffield and seeds NO enum value: the
; measured ternary's carriers land with the class-surface migration port
; (W10's second half), and R-MEASURED (ADR070) forbids assigning
; alignments. The one class node carries only `social-class/population`
; so the pack's never-firing load-probe rule has a legal binding (the
; rule pipeline refuses a zero-rule content set — rule_pipeline.rs's
; §2.2 check; the never-firing-probe idiom is production_conformance.rs's
; own precedent).
;
; Vocabulary ceremony: ADR206 (worldview_vocabulary_ceremony) mints the
; WorldView enum into the Rust closed BSL vocabulary; ADR195 minted the
; `enum` deffield row; ADR204 (W10) chartered this ceremony.
(scenario worldview/foundation
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
  (deffield social-class/population int extensive)

  ; The minimal world the tick runs over.
  (node workers NodeType/SOCIAL_CLASS (social-class/population 1000)))
```

- [ ] **Step 2: Write the probe rules pack**

Create `rust/crates/babylon-tick/content/rules/worldview.bsl` with exactly this content:

```lisp
; The worldview/* rule pack at the MINT (ADR206): ONE never-firing load
; probe. The rule pipeline refuses a zero-rule content set outright
; (rule_pipeline.rs's §2.2 check — "a content set needs at least one
; (rule …) top-form, found 0"), so a comment-only pack cannot exercise
; the load-and-tick path at all; the never-firing probe is the in-repo
; idiom for exactly this (production_conformance.rs's scenario-load
; smoke, territory_conformance.rs's no-op rule). The guard is false for
; every legal population, so `fired == 0` and `before == after` hold by
; construction. What the byte pin guards is the substrate LOAD of the
; mint scenario — the canonical state hash covers graph facts only
; (nodes/attributes/edges/hyperedges/edge attributes), so the `defenum`
; declaration itself does NOT move it; the ruled member ORDER is guarded
; by the explicit EnumRegistry ordinal assertion in the same test file
; (worldview_member_order_is_the_ruled_ordinal), not by the hash.
;
; The rule anchors under the ALREADY-registered `consciousness`
; namespace (babylon-tick/src/lib.rs's systems set): the worldview
; estate IS the consciousness domain's content kind, and a content mint
; changes no Rust source. The WorldView enum's first real consumers
; arrive with the class-surface migration port (ADR204 W10's second
; half).
(rule consciousness/worldview-mint-probe
  :material-basis "load-only smoke: the mint scenario loads and ticks; the mint's pins are the substrate-load hash plus the registry ordinal assertion"
  :fuel 8
  (bindings (binding population :field social-class/population))
  (when (< population 0))
  (effects
    (update-node self social-class/population (set population))))
```

- [ ] **Step 3: Write the failing pin test**

Append to `rust/crates/babylon-tick/tests/tick_goldens.rs` — first the const block (place after the `PRODUCTION_RULE` const, line 45):

```rust
const WORLDVIEW_SCENARIO: &str =
    include_str!("../content/scenarios/worldview-foundation.bscn");
const WORLDVIEW_RULES: &str = include_str!("../content/rules/worldview.bsl");
```

Then the test (append at end of file, after `production_conformance_hashes_are_pinned`):

```rust
/// The WorldView mint's own golden (ADR204 W10, ceremony ADR206): the
/// substrate LOAD of the mint scenario, byte-pinned. What this pin
/// guards is the world's graph facts — the canonical state hash covers
/// nodes/attributes/edges/hyperedges/edge attributes ONLY, so the
/// `defenum` declaration itself does not move these bytes; the ruled
/// member ORDER (REVOLUTIONARY=0 / LIBERAL=1 / FASCIST=2 — declaration
/// order IS the storage ordinal, ADR195) is guarded by the explicit
/// registry assertion in `worldview_member_order_is_the_ruled_ordinal`
/// below, not by this hash. The pack's one rule is a never-firing load
/// probe (the rule pipeline refuses a zero-rule content set; the idiom
/// is production_conformance.rs's own): the guard is false for every
/// legal population, so `fired == 0` and `before == after` are the mint
/// stage's honest expectations (the measured-ternary consumers land
/// with the port train), NOT a bug — exactly the emit-only logic the
/// organization golden's own header spells out, one step further.
#[test]
fn worldview_foundation_hashes_are_pinned() {
    let report = run_once(WORLDVIEW_SCENARIO, WORLDVIEW_RULES).expect("worldview-foundation tick");
    assert_eq!(
        hex(&report.before),
        "MEASURE_AT_EXECUTION",
        "pre-tick hash moved — this is the SUBSTRATE'S load of \
         worldview-foundation.bscn (the mint world's graph-fact pin)"
    );
    assert_eq!(
        hex(&report.after),
        "MEASURE_AT_EXECUTION",
        "post-tick hash moved — the probe rule never fires, so this \
         equals `before` by construction; a divergence here means the \
         tick mutated state without a firing rule, which is its own bug"
    );
    assert_eq!(
        report.fired, 0,
        "the worldview mint pack's load probe never fires (its guard is \
         false for every legal population)"
    );
}

/// The ruled ordinal order, guarded EXPLICITLY (task-review finding,
/// plan amended 2026-08-15): the canonical state hash covers graph
/// facts only, so the `defenum` declaration never moves the byte pin
/// above — THIS registry assertion, not the hash, is what guards
/// REVOLUTIONARY=0 / LIBERAL=1 / FASCIST=2. Declaration order IS the
/// storage ordinal (ADR195); a reordered, renamed, or dropped member
/// fails here loudly.
#[test]
fn worldview_member_order_is_the_ruled_ordinal() {
    let mut graph = HypergraphStore::new();
    let loaded =
        load_scenario(WORLDVIEW_SCENARIO, &mut graph).expect("worldview-foundation loads clean");
    let ty = loaded
        .enums
        .resolve("WorldView")
        .expect("the WorldView defenum is declared");
    assert_eq!(loaded.enums.ordinal(ty, "REVOLUTIONARY"), Some(0));
    assert_eq!(loaded.enums.ordinal(ty, "LIBERAL"), Some(1));
    assert_eq!(loaded.enums.ordinal(ty, "FASCIST"), Some(2));
}
```

The ordinal test needs two imports the golden tests don't use — add them to the file's use block (mirroring `production_conformance.rs`'s own imports, the working precedent for calling `load_scenario` from a tick test):

```rust
use babylon_bsl::scenario::load_scenario;
use babylon_graph::hypergraph_store::HypergraphStore;
```

- [ ] **Step 4: Run the test to verify it fails (red)**

Run: `cargo test -p babylon-tick --test tick_goldens worldview_foundation_hashes_are_pinned -- --nocapture` (from `rust/`)
Expected: FAIL — the assertion output prints the REAL measured hash as the "left" value against the `MEASURE_AT_EXECUTION` right value. A failure INSIDE `run_once` (a loader error) instead is a stop-and-report case — do not route around it.

- [ ] **Step 5: Insert the measured hashes (green)**

Replace both `"MEASURE_AT_EXECUTION"` strings with the hash printed by the failing assertion (the same value goes in both — the probe never fires). Measured, never derived — the repo's own golden discipline (see the `tick_goldens.rs` module header).

- [ ] **Step 6: Run the full tick + bsl suites to verify they pass**

Run: `cargo test -p babylon-tick --locked && cargo test -p babylon-bsl --locked` (from `rust/`)
Expected: PASS, including the new pin and ALL six pre-existing goldens byte-identical (untouched by this content pair's own load).

- [ ] **Step 7: Commit**

```bash
git add rust/crates/babylon-tick/content/scenarios/worldview-foundation.bscn \
        rust/crates/babylon-tick/content/rules/worldview.bsl \
        rust/crates/babylon-tick/tests/tick_goldens.rs
mise run commit -- "feat(tick): mint the WorldView defenum — worldview-foundation scenario + byte pin (ADR204 W10, ceremony ADR206)

Co-Authored-By: Kimi Code <noreply@moonshot.ai>"
```

---

### Task 2: The ceremony record — ADR206 + index.yaml

**Files:**
- Create: `ai/decisions/ADR206_worldview_vocabulary_ceremony.yaml`
- Modify: `ai/decisions/index.yaml` (append after the ADR205 row)

**Interfaces:**
- Consumes: ADR196's executed record shape (`ai/decisions/ADR196_org_vocabulary_ceremony.yaml`); the frozen-Python string values at `src/babylon/models/enums/consciousness.py:83-85`.
- Produces: the mint-and-retire record ADR176 (34)/ADR187 OQ-7 require; the ADR204 consequences' "next train" citation target.

- [ ] **Step 1: Verify the frozen-Python string values (the record's pre-writing verification)**

Run: `sed -n '80,86p' src/babylon/models/enums/consciousness.py`
Expected: the `ConsciousnessTendency` members `LIBERAL = "liberal"`, `FASCIST = "fascist"`, `REVOLUTIONARY = "revolutionary"` (lines 83-85). This is the file:line citation the ADR's context carries; the Python enum is NOT edited (frozen estate — "retiring" is never a statement about it).

- [ ] **Step 2: Write ADR206**

Create `ai/decisions/ADR206_worldview_vocabulary_ceremony.yaml` with exactly this content:

```yaml
ADR206_worldview_vocabulary_ceremony:
  status: "accepted"
  date: "2026-08-15"
  title: >
    The WorldView vocabulary ceremony (ADR204 W10's first train) — the
    WorldView defenum MINTED into the Rust closed BSL vocabulary as the
    second content-enum consumer of ADR195's enum deffield row, in the
    ruled member order REVOLUTIONARY / LIBERAL / FASCIST (declaration
    order IS the storage ordinal, ADR195); NOTHING retired (the ADR176
    (34)/ADR187 OQ-7 citation is to the record FORMAT — W11's
    Ideology-scalar strike already landed in ADR204's own train); the
    faction-classification enum (W9) does NOT join — its member list is
    not ruled content-complete — and charters separately.
  context: |
    ADR204 (the World-View Ternary unification, twelve Director rulings
    W1-W12) ruled the political simplex onto spec 034's (r, l, f) ternary
    as DECLARED CONTENT (W2: ideology is a world view, not a meter) and
    chartered this ceremony as W10's first train: "the vocabulary
    ceremony PR minting the `WorldView` defenum via the ADR195/196
    ceremony machinery, with the mint-and-retire record per ADR176
    (34)/ADR187 OQ-7." The design of record is
    `docs/superpowers/specs/2026-08-14-worldview-ternary-unification-design.md`
    (§4.1 the mint content, §4.5.1 the ceremony scope, §5 the reserved
    ledger).

    Pre-writing verification by string value (ADR196's record
    discipline): the frozen Python estate's own consciousness enum is
    `ConsciousnessTendency(StrEnum)` at
    `src/babylon/models/enums/consciousness.py:68-85`, members
    `LIBERAL = "liberal"` (:83), `FASCIST = "fascist"` (:84),
    `REVOLUTIONARY = "revolutionary"` (:85). Spec 034 A-005
    (`specs/034-ternary-consciousness/spec.md:268`) rules that this enum
    maps directly to the ternary vertices and that no new VALUES are
    needed; the mint transcribes the Director-ruled names. MECHANICALLY
    LOAD-BEARING ORDER FACT: the Python declaration order (LIBERAL,
    FASCIST, REVOLUTIONARY) differs from the ruled Rust mint order
    (REVOLUTIONARY, LIBERAL, FASCIST — design doc §4.1) — because
    declaration order IS the storage ordinal (ADR195), the Rust ordinals
    are REVOLUTIONARY=0, LIBERAL=1, FASCIST=2, and any future
    cross-implementation reading must map by NAME, never by index.

    The mint needs ZERO Rust-source change: content enums are data-driven
    through `EnumRegistry`/`parse_defenum`
    (`rust/crates/babylon-bsl/src/declarations.rs:591`,
    `types.rs:107-110`); the structural `ClosedVocabulary`'s four
    `EnumKind` variants (`vocabulary.rs:29-41`) gain nothing. The lexical
    shapes conform: `WorldView` satisfies §1.4's enum-type production
    (no underscore), the three members the enum-member production.
  decision: |
    ONE CEREMONY, ONE RECORD (ADR196's shape).

    MINTED (into the Rust closed BSL vocabulary, content-enum registry):
    - defenum `WorldView`, members in the RULED order:
      1. `REVOLUTIONARY` — the articulated pole; content home is the
         doctrine tree (the line IS the world view, W3).
      2. `LIBERAL` — hegemonic common sense; no tree; content computed
         per conjuncture from the ruling bloc (W3).
      3. `FASCIST` — the capture/degeneration terminus; no tree;
         parasitism-defense affect keyed to the rent gradient, plus a
         demagogy flag (W3).

    RETIRED: nothing. The ADR176 (34)/ADR187 OQ-7 citation in ADR204 is
    to the mint-and-retire record FORMAT. W11's strike of the `Ideology`
    [-1,1] sort from THE_FORMALISM's LAW table was executed in ADR204's
    own train (`ai/THE_FORMALISM.md` II.1, pointer note included); the
    five dead edge types (ADR176 (34)) and ActionType.STRIKE's dead
    member (ADR187 OQ-7) retire in their own Phase-2 ceremony, not here.
    The frozen Python enums are NOT edited — "minting" and "retiring"
    are facts about the Rust content-authoring surface only (ADR196's
    disclaimer, restated).

    NOT IN THIS CEREMONY: the faction-classification enum (W9) — the
    Director has not ruled its member list content-complete; it charters
    separately (ADR204 consequences, verbatim).
  consequences: |
    - The pins: `rust/crates/babylon-tick/tests/tick_goldens.rs`'s
      `worldview_foundation_hashes_are_pinned` anchors the substrate LOAD
      of the mint scenario byte-for-byte, and its sibling
      `worldview_member_order_is_the_ruled_ordinal` guards the ruled
      member order EXPLICITLY through the EnumRegistry — the canonical
      state hash covers graph facts only (nodes/attributes/edges/
      hyperedges/edge attributes), so the `defenum` declaration moves no
      hashed bytes and the ordinal assertion, not the hash, is the member
      order's guard (a task-review finding of this ceremony's own
      execution, recorded here so no successor train repeats the claim).
      The pack's one rule is a never-firing load probe (the rule pipeline
      refuses zero-rule content sets): `fired == 0` and
      `before == after` are the honest expectations.
    - The mint declares NO deffield and seeds NO enum value: the measured
      ternary's carriers (probability-lane alignment shares, UNPOSITIONED
      on absence) land with the class-surface migration port — W10's
      second half, the next train. R-MEASURED (ADR070) forbids assigning
      alignments; this ceremony only makes the kind NAMEABLE.
    - `ai/decisions/index.yaml` gains this ADR.
    - ADR204's consequences close their "next train is the vocabulary
      ceremony PR" clause with this ADR cited as its disposition; the
      class-surface migration port and the faction-enum charter remain
      open backlog.
  supersedes: []
  related:
    - ADR195_enum_deffield_row
    - ADR196_org_vocabulary_ceremony
    - ADR204_worldview_ternary_unification
    - ADR176_director_rulings_batch_gds_dispositions
    - ADR187_article_v_3x3_ratified
```

- [ ] **Step 3: Append the index.yaml row**

Append to `ai/decisions/index.yaml`, immediately after the `ADR205_t3_update_edge_parity_handoff` block (the current tail):

```yaml
  ADR206_worldview_vocabulary_ceremony:
    title: 'The WorldView vocabulary ceremony (ADR204 W10''s first train) — the WorldView defenum MINTED into the Rust closed BSL vocabulary as the second content-enum consumer of ADR195''s enum deffield row, in the ruled member order REVOLUTIONARY / LIBERAL / FASCIST (declaration order IS the storage ordinal); NOTHING retired — the ADR176 (34)/ADR187 OQ-7 citation is to the record FORMAT, and W11''s Ideology-scalar strike already landed in ADR204''s own train; the faction-classification enum (W9) does NOT join (member list not ruled content-complete) and charters separately'
    status: accepted
    date: '2026-08-15'
    file: ADR206_worldview_vocabulary_ceremony.yaml
```

- [ ] **Step 4: Validate the YAML**

Run: `uv run python -c "import yaml; [yaml.safe_load(open(f)) for f in ['ai/decisions/ADR206_worldview_vocabulary_ceremony.yaml','ai/decisions/index.yaml']]; print('yaml ok')"`
Expected: `yaml ok`

- [ ] **Step 5: Commit**

```bash
git add ai/decisions/ADR206_worldview_vocabulary_ceremony.yaml ai/decisions/index.yaml
mise run commit -- "docs(p29): ADR206 — the WorldView vocabulary ceremony record (mint-and-retire format per ADR176 (34)/ADR187 OQ-7)

Co-Authored-By: Kimi Code <noreply@moonshot.ai>"
```

---

### Task 3: The full gate + PR

**Files:**
- None (verification and PR only).

**Interfaces:**
- Consumes: Tasks 1-2's landed commits.
- Produces: the merged ceremony PR on `dev`.

- [ ] **Step 1: Run the full Rust gate**

Run: `mise run rust:check` (from the repo root)
Expected: PASS — fmt, clippy (`-D warnings -D clippy::cognitive_complexity`), workspace tests (including the new pin), pedantic legs, doc.

- [ ] **Step 2: Push and open the PR**

```bash
git push -u origin feat/worldview-vocabulary-ceremony
gh pr create --base dev --title "feat(tick): mint the WorldView defenum — the W10 vocabulary ceremony (ADR206)"
```

PR body must state: the ruled member order and its ordinal law; zero Rust source changes (data-driven content enum); no retirements in this ceremony; the faction enum charters separately; no baselines touched (no §6.5 ceremony); the golden is a NEW pin, all six pre-existing pins unmoved. The pre-push suite will run `rust:check` again automatically (the branch touches `rust/`) — let it.

- [ ] **Step 3: CI watch, Copilot harvest, merge**

Watch `gh pr checks N --watch` until green; harvest the Copilot review (for EACH inline comment: push a fix or post a reply — zero unaddressed comments is the merge precondition); merge with `mise run pr:merge -- N`.

- [ ] **Step 4: Close the loop on ADR204**

Comment on the merged PR (or the Program 29 umbrella #557) noting ADR206 is the disposition of ADR204 consequences' "next train is the vocabulary ceremony PR" clause; the class-surface migration port remains as W10's second half.
