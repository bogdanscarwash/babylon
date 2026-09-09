# Organization Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the Organization contract's floor — the `enum` deffield row (language change), the
`defenum`/`defvocabulary` declaration forms, production `ClosedVocabulary` enforcement,
`NodeType/ORGANIZATION` + the six founding edges (vocabulary ceremony ADR), and org-seeding
canonical scenarios pinned in the Rust tick goldens and both Python byte gates.

**Architecture:** Enum values are stored as **declared-order ordinals in the existing f64
attribute lane** (zero bytes of any existing golden move — the proven Half-1 argument), but the
ordinal is **never surfaced**: writes accept only an `EnumRef` of the declared enum type, reads
produce `Value::Enum`, and equality is the only comparison (already enforced). The vocabulary is
**explicitly declared** in content (`defvocabulary`), never inferred from what a scenario happens
to seed — a closed vocabulary that infers its members would make every typo self-legalizing.

**Tech Stack:** Rust (babylon-bsl, babylon-tick, babylon-graph), Python 3.12 (qa gates only —
the frozen engine CAPABILITY estate `src/babylon/engine/systems/`, `src/babylon/formulas/`, and
domain math is untouched; the scenarios/tools/baselines estate that Tasks 11–13 touch is
post-freeze-ACTIVE by design, Scout 5 verified), Sphinx RST (bsl-language.rst), pytest sync tests.

**Normative sources:** `docs/superpowers/specs/2026-08-11-organization-game-object-design.md`
(§1 rulings Q1/Q12/Q15, §2, §9, §11 — Director-approved 2026-08-11);
`reports/organization-design-inputs-2026-08-11.md`; ADR176 (34); ADR187 OQ-7.

## Global Constraints — these are LAW

1. **TDD red → green → refactor, per task.** Write the failing test, run it and see it fail for
   the stated reason, implement, run green, commit. A task that skipped its red phase is not done.
2. **Determinism is not negotiable.** No `HashMap`/`HashSet` iteration reaches any result,
   ordinal assignment, or error-report order. `EnumRegistry` preserves **declaration order** —
   that order is normative (it IS the storage ordinal). Never reuse `ClosedVocabulary` for
   ordinals: it sorts-and-dedups by design (`vocabulary.rs:230`).
3. **Zero movement of existing goldens.** Every task carries the proof: `cargo test -p
   babylon-tick --test tick_goldens` byte-identical, and no change to `state_hash.rs` — the
   encoder is untouched by this entire plan (an enum ordinal is an ordinary §0x02 f64 row).
4. **Loud refusal, never a silent no-op.** Everything scoped out refuses naming the construct,
   the reason, and the follow-on that will serve it (Constitution III.11).
5. **No invented numbers.** D-row and ADR numbers resolve at PR-open time against dev
   (`TestTheDraftRulingRegisterHasNoDuplicateRowNumbers` exists because #500/D99 collided once).
   As of plan-writing the next free are **D101** and **ADR195/196** — RE-CHECK at execution.
6. **The Q12↔D94 supersession is explicit, never silent.** D94 (`bsl-language.rst:2203-2217`,
   ruled 2026-08-11) seals the type table at six rows and names `Enum<T>` non-storable. The
   Director's Q12 ruling (spec §1, Director-approved twice: live popup + spec approval) minted
   the seventh row knowingly ("sealed twice this month" was in the question put to her). Task 1
   REWRITES the sealing paragraph citing Q12, and the new D-row records the supersession. Never
   present this as a workforce draft ruling.
7. **Sequencing / machine safety.** Rust tasks (3–10) touch `evaluator.rs`/`tick.rs` and MUST NOT
   start until the query-evaluation slice-1 train (PR #514 + groups 3–5) has merged — one
   cargo-heavy train at a time, and both trains edit the same files. Python tasks (11–13) and the
   doc tasks (1–2: spec text + ADRs) have no cargo dependency and may run first. Never run workspace-wide cargo
   tests while another agent runs cargo.
8. **Commit after each unit of work**, conventional commits, verify `git log --oneline -1` moved
   (hooks abort silently). Worktree recipe: symlink `.venv`, copy `.env`, data symlink farm
   enumerated FROM THE MAIN CHECKOUT, `mise trust`, commit via
   `UV_FROZEN=1 PYTHONPATH="$PWD/src" git commit`.
9. **Baseline ceremony (§6.5).** Any commit touching `tests/baselines/**` carries the
   `Baselines: blessed(<slug>)` trailer via `python3 tools/generate_ceremony_message.py --slug
   <slug> --summary "..."` piped into `git commit -F -`.

## File Structure

| File | Responsibility |
|---|---|
| `docs/reference/bsl-language.rst` | Modify: type table 7th row (~2141-2198), sealing paragraph rewrite (~2203-2217), `defenum`/`defvocabulary` grammar + prose (new §2.9-adjacent subsections), error codes, Draft-Ruling Register rows |
| `docs/reference/bsl.ebnf` | Modify: `type-name` comment block, new `defenum`/`defvocabulary` productions |
| `tests/unit/reference/test_bsl_grammar_sync.py` | Modify: new `TestTheEnumRowStaysInSync` class (D99-pattern RHS-grain checks) |
| `rust/crates/babylon-bsl/src/types.rs` | Modify: `BslType::Enum(EnumTypeId)`, new `EnumTypeId`, `EnumRegistry`, `EnumDecl` |
| `rust/crates/babylon-bsl/src/declarations.rs` | Modify: `parse_type_name` 7th arm, `parse_deffield` `:enum-type` keyword, `parse_defenum` |
| `rust/crates/babylon-bsl/src/scenario.rs` | Modify: `.bscn` `defenum` + `defvocabulary` + enum `deffield` arm + `attribute_value_enum` + vocabulary checks in `load_node`/`load_edge` |
| `rust/crates/babylon-bsl/src/structural_verbs.rs` | Modify: enum write path in `update-node` (EnumRef-only), vocabulary check in `add_node`/`add_edge`/`add_hyperedge` |
| `rust/crates/babylon-bsl/src/tick.rs` | Modify: `bind_subject` enum read path (registry threaded through `TickCtx`) |
| `rust/crates/babylon-bsl/src/evaluator.rs` | Modify: `field-of` enum-field refusal (D-row), `EvalEnv` untouched otherwise |
| `rust/crates/babylon-bsl/src/rule_pipeline.rs` | Modify: enum-ref membership pass when `vocabulary_registry` is `Some` |
| `rust/crates/babylon-bsl/src/grammar.rs` | Modify: un-swallow the `owner_of` Err at 291-293 (`E-LOAD-023` reachable) |
| `rust/crates/babylon-tick/src/lib.rs` | Modify: construct `Some(ClosedVocabulary)` from `defvocabulary`, extend `systems` set with `"organization"` |
| `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` | Create: the org-seeding canonical scenario |
| `rust/crates/babylon-tick/content/rules/organization.bsl` | Create: the kind-probe rule (exercises the enum chain end-to-end) |
| `rust/crates/babylon-tick/tests/tick_goldens.rs` | Modify: new pinned pre/post hash pair |
| `ai/decisions/ADR<N>_enum_deffield_row.yaml` + `ADR<N+1>_org_vocabulary_ceremony.yaml` + `index.yaml` | Create: the language-change ADR and the mint+retire ceremony ADR |
| `src/babylon/engine/scenarios/org_probe.py` | Create: `create_org_probe_scenario` (Python gate scenario — scenario estate, NOT frozen engine) |
| `tools/regression_scenarios.py` | Modify: 12th `SCENARIOS` entry + dispatch + `ScenarioCoverage` rows (dead-column trap) |
| `tools/vault_regression.py` | Modify: 3rd vault scenario + `_bake_org_probe` + generalize `_build_manifest`'s tick ternary |
| `tests/baselines/org_probe.json`, `tests/baselines/dense/org_probe.csv`, `tests/baselines/vault/org_probe/manifest.json` | Create: generated + ceremony-blessed |
| `.mise.toml` | Modify: `qa:vault-regression-ci` runs the org scenario too |

## PR groups

- **Group A (no cargo, runs anytime): Tasks 1–2** — spec text + ADRs. One PR.
- **Group B (Python gates, no cargo, runs anytime): Tasks 11–13** — org_probe scenario +
  baselines + vault. One PR (ceremony commit inside).
- **Group C (cargo — AFTER query-eval slice 1 merges): Tasks 3–7** — the language change. One PR.
- **Group D (cargo): Tasks 8–9** — vocabulary enforcement. One PR.
- **Group E (cargo): Task 10** — the org scenario + rule + goldens. One PR.

---

### Task 1: The spec text — seventh row, `defenum`, `defvocabulary`, the supersession

**Files:**
- Modify: `docs/reference/bsl-language.rst` (type table ~2141-2198; sealing paragraph ~2203-2217; new subsections after the deffield chapter; error-code section; Draft-Ruling Register ~4365+)
- Modify: `docs/reference/bsl.ebnf`
- Test: `tests/unit/reference/test_bsl_grammar_sync.py`

**Interfaces:**
- Consumes: spec §1 Q12/Q15, D94's exact text, D99's structural template (rst:5208-5349).
- Produces: the normative text every Rust task cites; the D-row numbers (resolved at PR-open);
  grammar productions `defenum ::=` and `defvocabulary ::=` that Task 3's parser implements.

- [ ] **Step 1: Write the failing sync tests** — add to `tests/unit/reference/test_bsl_grammar_sync.py`, following `TestTheRatioLiteralStaysInSync` (line 506) exactly:

```python
class TestTheEnumRowStaysInSync:
    """The enum deffield row (spec §1 Q12 of the Organization contract) —
    RHS-grain checks so the row cannot silently lose its way to be written."""

    def test_the_rst_type_table_has_an_enum_row(self, rst_text: str) -> None:
        assert re.search(r"^\s+\* - ``enum``", rst_text, re.M), (
            "the <type-name> table must carry the enum row (spec §1 Q12)"
        )

    def test_the_sealing_paragraph_counts_seven(self, rst_text: str) -> None:
        assert "seven rows" in rst_text and "no ``<type-name>`` position can name" in rst_text, (
            "D94's sealing paragraph must be rewritten to seven rows with the "
            "Q12 supersession recorded, not silently contradicted"
        )

    def test_the_ebnf_has_defenum_and_defvocabulary(self, ebnf_text: str) -> None:
        assert "defenum" in ebnf_text and "defvocabulary" in ebnf_text

    def test_the_supersession_d_row_is_recorded(self, rst_text: str) -> None:
        # Resolve the number at PR-open; the row must cite Q12 and D94 by name.
        assert re.search(r"supersed\w+ D94", rst_text), (
            "the register row must record that the enum row supersedes D94's "
            "exclusion by Director ruling (spec §1 Q12), never silently"
        )
```

(Use the file's existing `rst_text`/`ebnf_text` fixtures; match their actual names when editing.)

- [ ] **Step 2: Run and see them fail** — `mise run test:q -- tests/unit/reference/test_bsl_grammar_sync.py -k Enum` → 4 failures ("no enum row", etc.).

- [ ] **Step 3: Write the rst text.** (a) Add the seventh table row: `enum` / "a member of one
  content-declared closed enum (see `defenum`); stored as the declared-order ordinal in the
  binary64 attribute lane; written and read ONLY as `<EnumType>/<MEMBER>`; comparable with
  `=`/`!=` only" / "spec §1 Q12; supersedes D94's exclusion — see the register". (b) Rewrite
  the sealing paragraph: "…the **seven** rows above… The `enum` row was minted by Director
  ruling (Organization contract, spec §1 Q12, 2026-08-11), superseding D94's exclusion of a
  declarable closed-enum row; `Enum<T>` as a *typechecker* classification is unchanged."
  (c) New subsection "Declaring enums and the graph vocabulary" with the two productions:

```
<defenum>       ::= "(" "defenum" <enum-type-name> "(" <enum-member>+ ")" ")"
<defvocabulary> ::= "(" "defvocabulary" <enum-kind> "(" <enum-member>+ ")" ")"
```

  where `<enum-type-name>` is an uppercase-initial identifier (the reader's existing enum-ref
  lexing), `<enum-kind>` is one of `NodeType | EdgeType | HyperedgeType | EventType`, and member
  order in `defenum` is **normative** (it is the storage ordinal). State the write/read law
  (EnumRef-only both directions; a bare number into an enum field is a load/eval error), the
  no-`:kind` rule (an enum field has no aggregation kind; any aggregation over it refuses), and
  the `field-of` deferral (enum fields read via `:field` bindings; `field-of` over one refuses
  loudly naming the follow-on D-row). (d) Error codes: take the next free numbers in the E-LOAD
  and E-EVAL families at PR-open (grep first, per constraint 5) for: unknown enum type, unknown
  member, bare-number write, `defvocabulary` membership violation. (e) Register rows: the
  supersession D-row + the field-of-deferral D-row, D99-template format. (f) Mirror the
  productions into `bsl.ebnf` with the D94 comment block updated.

- [ ] **Step 4: Run green** — `mise run test:q -- tests/unit/reference/test_bsl_grammar_sync.py` (the WHOLE file — the appendix-collection classes must also pass, which is what forces the ebnf mirror).

- [ ] **Step 5: Commit** — `docs(bsl): enum deffield row + defenum/defvocabulary — Q12 supersession of D94 recorded`.

### Task 2: The two ADRs

**Files:**
- Create: `ai/decisions/ADR<N>_enum_deffield_row.yaml`, `ai/decisions/ADR<N+1>_org_vocabulary_ceremony.yaml` (numbers at PR-open; N=195 as of writing)
- Modify: `ai/decisions/index.yaml`

**Interfaces:** Produces the ceremony record Tasks 8–10 cite. Style: `ADR085_retire_org_solidarity_mass_link.yaml` (context/decision/consequences/verification/references).

- [ ] **Step 1: Write ADR<N> (language change):** context = Q12 ruling + D94 supersession;
  decision = the seventh row with the ordinal-storage/EnumRef-surface law; consequences = the
  D-rows, the error codes, first consumer `organization/kind`; verification = the sync-test
  class + existing-goldens byte-identity.
- [ ] **Step 2: Write ADR<N+1> (the vocabulary ceremony — ONE ceremony, ONE record per ADR176 (34) + ADR187 OQ-7):**
  MINTS `NodeType/ORGANIZATION`, `EdgeType/{MEMBERSHIP, PRESENCE, COMMAND, TRANSACTIONAL,
  SOLIDARISTIC, SOLIDARITY(org↔org usage note)}`, and enum type `OrgKind
  {STATE_APPARATUS, BUSINESS, POLITICAL_FACTION, CIVIL_SOCIETY}` (spec §1 Q1/Q15);
  RECORDS RETIRED (never to enter the BSL vocabulary): `TARGETS, OWNED_BY, JURISDICTION,
  RECRUITMENT, EMPLOYMENT` (ADR176 (34)) and `ActionType.STRIKE`'s dead member + eligibility row
  + `base_cost_strike` (ADR187 OQ-7) — noting the Python enums are frozen-estate and unedited.
- [ ] **Step 3: Add both to `index.yaml`; commit** — `docs(adr): enum-row language change + the Phase-2 vocabulary ceremony (mint + retire, one record)`.

### Task 3: `EnumRegistry`, `EnumTypeId`, the widened `BslType`

**Files:**
- Modify: `rust/crates/babylon-bsl/src/types.rs`
- Modify: `rust/crates/babylon-bsl/src/score_class.rs:85` (the one existing `BslType::Enum` match arm)
- Modify: `rust/crates/babylon-bsl/src/declarations.rs` (`parse_type_name` 439-452, `parse_deffield` 365-418, new `parse_defenum`)
- Test: in-module `#[cfg(test)]`

**Interfaces:**
- Produces: `pub struct EnumTypeId(pub u32);` (Copy) · `pub struct EnumDecl { pub name: String, pub members: Vec<String> }` ·
  `pub struct EnumRegistry { types: Vec<EnumDecl> }` with
  `pub fn declare(&mut self, name: &str, members: &[String]) -> Result<EnumTypeId, DeclError>` (rejects duplicate type
  names, duplicate members, empty member lists; preserves declaration order),
  `pub fn resolve(&self, name: &str) -> Option<EnumTypeId>`,
  `pub fn ordinal(&self, ty: EnumTypeId, member: &str) -> Option<u32>`,
  `pub fn member(&self, ty: EnumTypeId, ordinal: u32) -> Option<&str>`,
  `pub fn name(&self, ty: EnumTypeId) -> &str` ·
  `BslType::Enum(EnumTypeId)` (replacing `Enum(&'static str)`) ·
  `parse_type_name` gains `"enum"` → returns a marker the caller must complete with `:enum-type`
  (make the signature honest: `parse_type_name(name, enums: Option<(&EnumRegistry, Option<&str>)>)`
  or split a `parse_deffield`-local path — implementer's choice, but `deffield … :type enum`
  WITHOUT `:enum-type <EnumTypeName>` is a loud parse error, and `:enum-type` on any other type
  is one too) · `parse_defenum(parts: &[SExpr], registry: &mut EnumRegistry) -> Result<(), DeclError>`.

- [ ] **Step 1: Red tests** (types.rs + declarations.rs test modules):

```rust
#[test]
fn declaration_order_is_the_ordinal_order_and_is_preserved() {
    let mut r = EnumRegistry::default();
    let ty = r.declare("OrgKind", &["STATE_APPARATUS".into(), "BUSINESS".into(),
        "POLITICAL_FACTION".into(), "CIVIL_SOCIETY".into()]).unwrap();
    assert_eq!(r.ordinal(ty, "STATE_APPARATUS"), Some(0));
    assert_eq!(r.ordinal(ty, "CIVIL_SOCIETY"), Some(3));
    assert_eq!(r.member(ty, 1), Some("BUSINESS"));
    assert_eq!(r.ordinal(ty, "NOWHERE"), None);
}

#[test]
fn a_duplicate_member_or_type_or_empty_list_refuses_loudly() {
    let mut r = EnumRegistry::default();
    assert!(r.declare("K", &[]).is_err());
    assert!(r.declare("K", &["A".into(), "A".into()]).is_err());
    r.declare("K", &["A".into()]).unwrap();
    assert!(r.declare("K", &["B".into()]).is_err());
}

#[test]
fn deffield_type_enum_requires_enum_type_and_resolves_it() {
    // parse a (deffield organization/kind :type enum :enum-type OrgKind)
    // against a registry holding OrgKind → FieldDecl { ty: BslType::Enum(id), .. }
    // and the same form WITHOUT :enum-type is a loud DeclError naming the keyword.
}
```

- [ ] **Step 2: Run red** — `cargo test -p babylon-bsl enum_registry declaration_order` → fail (types missing).
- [ ] **Step 3: Implement.** `EnumRegistry` as specified (a `Vec` — declaration order IS the
  storage; the member lookup is a linear scan over ≤ a handful of members, fine). Change
  `BslType::Enum(&'static str)` → `Enum(EnumTypeId)`; fix `score_class.rs:85` (arm shape
  unchanged, binding ignored). Check `BslType`'s derives — `EnumTypeId` is Copy so nothing
  breaks. `parse_defenum` destructures `(defenum <Symbol-or-EnumRef-type-name> (members…))`
  reusing the reader's uppercase-initial validation shape. Extend `parse_deffield`'s keyword
  loop with `:enum-type`, and thread an `&EnumRegistry` parameter to `parse_deffield`/
  `parse_type_name` callers (`metrics.rs:43,326` passes `None`-equivalent — a `metric` may not be
  enum-typed in this slice; refuse loudly there naming the deferral).
- [ ] **Step 4: Green + gate** — `cargo test -p babylon-bsl`; `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] **Step 5: Commit** — `feat(bsl): EnumRegistry + BslType::Enum(EnumTypeId) + deffield :enum-type (spec Q12)`.

### Task 4: The `.bscn` dialect — `defenum`, enum `deffield`, EnumRef-only seeding

**Files:**
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` (`load_deffield` 556-600, `attribute_value` 676-706, new `attribute_value_enum`, new `load_defenum`, `LoadedScenario` gains `pub enums: EnumRegistry`)
- Test: scenario.rs `#[cfg(test)]` (the Half-1 test block's style, 1073+)

**Interfaces:**
- Consumes: Task 3's `EnumRegistry`.
- Produces: `.bscn` forms `(defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))`
  and positional `(deffield organization/kind enum OrgKind)` (D93 dialect — the 4th slot holds
  the enum type name; there is NO kind symbol for enum fields); node-body seeding
  `(organization/kind OrgKind/BUSINESS)` storing ordinal `1.0`.

- [ ] **Step 1: Red tests:**

```rust
#[test]
fn an_enum_field_seeds_by_member_ref_and_stores_the_declared_ordinal() {
    let src = r#"(scenario org/t
      (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))
      (deffield organization/kind enum OrgKind)
      (node acme NodeType/ORGANIZATION (organization/kind OrgKind/BUSINESS)))"#;
    let mut g = MemoryGraph::default();
    let loaded = load_scenario(src, &mut g).expect("loads");
    let id = /* the single node */;
    assert_eq!(g.node_attribute(id, "organization/kind").unwrap(), 1.0);
    assert!(loaded.enums.resolve("OrgKind").is_some());
}

#[test]
fn a_bare_number_into_an_enum_field_refuses_naming_the_law() {
    // same scenario but (organization/kind 1) → Err whose message contains
    // "enum" and "OrgKind/" — the ordinal is never a surface value.
}

#[test]
fn a_wrong_enum_type_member_refuses() {
    // (organization/kind NodeType/SOCIAL_CLASS) → Err naming OrgKind.
}

#[test]
fn an_undeclared_member_refuses() {
    // (organization/kind OrgKind/NOWHERE) → Err listing the four members.
}

#[test]
fn enum_seeding_moves_no_existing_golden_bytes() {
    // load vitality-conformance.bscn unchanged; hash equals its pinned pre-tick
    // value (include_str! + StateEncoder — the Task-10 goldens also re-prove this).
}
```

- [ ] **Step 2: Run red** — `cargo test -p babylon-bsl an_enum_field_seeds` → fail ("unknown type `enum`").
- [ ] **Step 3: Implement.** `load_defenum` (before any `deffield` referencing it — top-to-bottom
  doctrine); the `load_deffield` 4-item enum arm (`[_, qname, Symbol("enum"), Symbol(type_name)]` →
  `BslType::Enum(registry.resolve(type_name)…)` with a loud unknown-type error); the
  `attribute_value` enum arm dispatching to `attribute_value_enum(atom, …)` which accepts ONLY
  `Atom::EnumRef { enum_type, member }`, checks `enum_type` matches the field's declared enum,
  resolves the ordinal, returns `ordinal as f64` — every other atom refuses with the
  bare-number law. Two-site defense: the same check shape lands at runtime in Task 5.
- [ ] **Step 4: Green + goldens** — `cargo test -p babylon-bsl && cargo test -p babylon-tick --test tick_goldens`.
- [ ] **Step 5: Mutation check** — make `attribute_value_enum` accept `Atom::Int` → the
  bare-number test flips red → revert byte-identical (record in commit body).
- [ ] **Step 6: Commit** — `feat(bsl): .bscn defenum + enum deffield seeding, EnumRef-only (spec Q12)`.

### Task 5: The runtime write path — `update-node` on an enum field

**Files:**
- Modify: `rust/crates/babylon-bsl/src/structural_verbs.rs` (`numeric_write_value` 672-706 untouched for non-enum; new enum branch where the write resolves its target field's decl)
- Modify: `rust/crates/babylon-bsl/src/tick.rs` (thread `&EnumRegistry` + field decls into the executor's context — follow how `intrinsic_costs` already travels)
- Test: structural_verbs/tick test modules

**Interfaces:**
- Consumes: Tasks 3–4. Produces: `(update-node <ref> organization/kind OrgKind/POLITICAL_FACTION)`
  writing ordinal `2.0`; `Value::Enum` reaching a non-enum field, or any non-EnumRef value
  reaching an enum field, refuses loudly (the existing `numeric_write_value` catch-all already
  refuses `Value::Enum` — keep that message for non-enum fields; the NEW branch fires only when
  the target field is enum-declared).

- [ ] **Step 1: Red tests** — an update-node write of `OrgKind/POLITICAL_FACTION` lands `2.0`
  (read back via `node_attribute`); a write of `2.0` (Real) into the enum field refuses naming
  the law; a write of `OrgKind/BUSINESS` into a non-enum field refuses (existing catch-all
  message asserted); cross-enum-type write refuses.
- [ ] **Step 2: Run red** → the enum write dies in `numeric_write_value`'s catch-all today.
- [ ] **Step 3: Implement** — in the `update-node` field-write path, look up the field's decl
  first; enum-declared → evaluate the value expr, demand `Value::Enum` of the matching type,
  resolve ordinal, write; else → existing `numeric_write_value` unchanged. Mirror
  `store_range_check`'s two-site pattern.
- [ ] **Step 4: Green + goldens; Step 5: Mutation check** (enum branch accepts Real → test flips);
  **Step 6: Commit** — `feat(bsl): runtime enum writes — EnumRef in, ordinal stored, everything else loud`.

### Task 6: The read path — `bind_subject` renders the member; `field-of` defers

**Files:**
- Modify: `rust/crates/babylon-bsl/src/tick.rs` (`bind_subject` ~237: consult the field decl; enum-declared reads become `Value::Enum { enum_type, member }` via `EnumRegistry::member(ty, value as u32)`; a stored f64 that is non-integral or out of range is a LOUD integrity error — never a clamp)
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs` (`field-of` on an enum-declared field: loud refusal citing the Task-1 D-row — needs the field-decl map reachable at that site; if `EvalEnv` cannot carry it without disturbing the merged query-eval shape, implement the refusal in `tick.rs`'s dispatch layer where decls are in scope, and record where it lives)
- Test: tick test module + an end-to-end guard test

**Interfaces:**
- Consumes: Tasks 3–5. Produces: `(binding kind :field organization/kind)` then
  `(when (= kind OrgKind/BUSINESS))` working end-to-end — the eligibility-matrix building block
  (spec §1 Q1's "eligibility matrix moves to BSL content").

- [ ] **Step 1: Red tests** — a rule whose `when` compares the bound enum field to an EnumRef
  literal fires for the matching node and not the other (two-node scenario, both kinds); ordering
  on the bound value refuses (existing `apply_ordering` message asserted — proves no Real leaks);
  a corrupted stored value (write `7.0` directly via the substrate in the test) reads as a loud
  integrity error naming the field and the member count.
- [ ] **Step 2: Run red** — the `when` comparison today errors "Enum compares only to…Real" (Real leak visible).
- [ ] **Step 3: Implement**; **Step 4: Green + goldens**; **Step 5: Mutation check** — make the
  read return `Value::Real(ordinal)` → the when-guard test flips red → revert.
- [ ] **Step 6: Commit** — `feat(bsl): enum field reads render the member — the when-guard eligibility seam works end-to-end`.

### Task 7: `defvocabulary` in the `.bscn` dialect

**Files:**
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` (new `load_defvocabulary`; `LoadedScenario` gains `pub vocabulary: Option<ClosedVocabulary>` built from the declared members via `ClosedVocabulary::new`)
- Test: scenario.rs test module

**Interfaces:**
- Produces: `(defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))` /
  `(defvocabulary EdgeType (MEMBERSHIP PRESENCE COMMAND TRANSACTIONAL SOLIDARISTIC SOLIDARITY))`
  forms; a scenario with NO defvocabulary forms yields `vocabulary: None` (existing scenarios
  unchanged — enforcement is opt-in per scenario until content adopts it).

- [ ] **Step 1: Red tests** — a scenario declaring the vocabulary loads and `vocabulary.is_some()`;
  `ClosedVocabulary`'s own errors surface at load (declare `TENANCY` under two kinds → `E-LOAD-032`
  propagates); an unknown `<enum-kind>` symbol refuses.
- [ ] **Step 2: Run red; Step 3: Implement** (collect per-kind member lists in file order, feed
  `ClosedVocabulary::new` once at end-of-load); **Step 4: Green + goldens; Step 5: Commit** —
  `feat(bsl): defvocabulary — the closed graph vocabulary is declared, never inferred`.

### Task 8: Enforcement — the registry goes live

**Files:**
- Modify: `rust/crates/babylon-tick/src/lib.rs` (`prepare_rules` ~188: `vocabulary_registry: scenario.vocabulary.as_ref()`; `systems` set gains `"organization"`)
- Modify: `rust/crates/babylon-bsl/src/rule_pipeline.rs` (`load_rule_form` ~216-265: when `Some`, a NEW pass walks every `Atom::EnumRef` of the four structural kinds in the rule AST and calls `check_enum_ref` — `E-LOAD-030/031` become reachable)
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` (`load_node` 602-654 / `load_edge` 847-901: when the scenario declared a vocabulary, check the member before `add_node`/`add_edge`)
- Modify: `rust/crates/babylon-bsl/src/structural_verbs.rs` (`add_node`/`add_edge`/`add_hyperedge`: same check when a registry is threaded — reuse the Task-5 context threading)
- Modify: `rust/crates/babylon-bsl/src/grammar.rs:291-293` (propagate the `owner_of` Err — `E-LOAD-023` becomes reachable)
- Test: negative tests at every entry point

**Interfaces:** Consumes Tasks 5+7. Produces: with a declared vocabulary, a typo'd
`NodeType/FOO` fails at scenario load, at rule load, AND at verb execution — the three producers
Scout 3 proved all silently mint today. Without one: exactly today's behavior (existing packs
untouched, proven by the goldens).

- [ ] **Step 1: Red tests** — `(node x NodeType/FOO)` under a declared vocabulary → `E-LOAD-031`
  naming `FOO`; a rule using `EdgeType/NOWHERE` in add-edge → load error; a runtime
  `(add-node NodeType/FOO …)` → eval error; the SAME sources with no `defvocabulary` → all load
  and run (backward-compat pin); `grammar.rs` field-init with an unowned segment under `Some` →
  `E-LOAD-023` (was silently skipped).
- [ ] **Step 2: Run red** (typo tests pass today — that's the bug; they must FAIL loud after).
- [ ] **Step 3: Implement; Step 4: Green + goldens + full four-leg gate; Step 5: Mutation check** —
  revert the `load_node` check → scenario-typo test flips green-over-dead-guard → restore.
- [ ] **Step 6: Commit** — `feat(bsl): closed-vocabulary enforcement live at all three producers (E-LOAD-030/031/023 reachable)`.

### Task 9: The r9/regression sweep

**Files:** Modify: any `r9_chapters.rs` / `conformance_corpus.rs` fixtures whose `LoadContext`
shape moved; no production code.

- [ ] **Step 1:** `cargo test -p babylon-bsl` full — fix fixture fallout ONLY (constructor
  signatures, `LoadedScenario` field additions). Zero behavioral edits; any test whose MEANING
  would change is a STOP-and-report.
- [ ] **Step 2:** Full four-leg gate + `cargo test --workspace --locked`. Commit —
  `test(bsl): fixture fallout from the enum/vocabulary landing — no behavioral edits`.

### Task 10: `organization-foundation.bscn` + the probe rule + pinned goldens

**Files:**
- Create: `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn`
- Create: `rust/crates/babylon-tick/content/rules/organization.bsl`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`

**Interfaces:** Consumes everything above. Produces: the Rust half of spec §11's hash anchor.

- [ ] **Step 1: Author the scenario** (vitality-conformance's header discipline — comment block
  explaining the world, citing spec §1 Q1/Q15 and the ceremony ADR):

```
(scenario organization/foundation
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))
  (defvocabulary EdgeType
    (MEMBERSHIP PRESENCE COMMAND TRANSACTIONAL SOLIDARISTIC SOLIDARITY))
  (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))
  (deffield organization/kind enum OrgKind)
  (deffield organization/active int extensive)
  (deffield social-class/population int extensive)
  (node workers NodeType/SOCIAL_CLASS (social-class/population 1000))
  (node county  NodeType/TERRITORY)
  (node reading-group NodeType/ORGANIZATION
    (organization/kind OrgKind/CIVIL_SOCIETY) (organization/active 1))
  (node precinct NodeType/ORGANIZATION
    (organization/kind OrgKind/STATE_APPARATUS) (organization/active 1))
  (edge EdgeType/MEMBERSHIP reading-group workers 1)
  (edge EdgeType/PRESENCE reading-group county 1)
  (edge EdgeType/PRESENCE precinct county 1)
  (edge EdgeType/SOLIDARITY reading-group precinct 1))
```

  (The org↔org SOLIDARITY edge is deliberate: it is the Q15/Q18 edge, present from the first
  golden so the win-gate repair train has a seeded instance to count. A TERRITORY node with no
  fields is legal — verify against `load_node`; if a fieldless node trips anything, seed
  `territory/active int extensive` = 1 instead and note it.)

- [ ] **Step 2: Author the probe rule** (`organization.bsl`) — one rule exercising the enum chain
  inside the golden:

```
(rule organization/kind-probe
  :material-basis "the state's coercive organs are a distinct material kind; content can see the difference (spec Q1)"
  :fuel 32
  (bindings
    (binding kind :field organization/kind)
    (binding active :field organization/active))
  (when (and (= active 1) (= kind OrgKind/STATE_APPARATUS)))
  (effects
    (emit EventType/ORGANIZATION_SEEDED (probe 1))))
```

  Check the anchor/system conventions against `vitality.bsl`'s actual header (system name
  `organization` — Task 8 already registered it) and match the emit-form arity the sink accepts.

- [ ] **Step 3: Measure the hashes** — scratch test printing `hex(&report.before/after)` via
  `run_once(SCENARIO, RULE)`; run once; paste (never hand-derive).
- [ ] **Step 4: Pin them** — new `#[test] fn organization_foundation_hashes_are_pinned()` in
  `tick_goldens.rs`, template lines 57-70, `include_str!` consts, assert messages naming what
  moving means. Confirm `report.fired == 1` in the same test (the probe fired for exactly the
  precinct).
- [ ] **Step 5: Green + FULL gate + all existing goldens byte-identical. Commit** —
  `feat(tick): organization-foundation golden — the org estate enters the Rust byte gate (spec §11)`.

### Task 11: `create_org_probe_scenario` (Python — no cargo; may run before Tasks 3–10)

**Files:**
- Create: `src/babylon/engine/scenarios/org_probe.py`
- Modify: `src/babylon/engine/scenarios/__init__.py` (export)
- Test: `tests/unit/engine/scenarios/test_org_probe.py`

**Interfaces:** Produces `create_org_probe_scenario() -> tuple[WorldState, SimulationConfig, GameDefines]`.
Constraint: touches NO frozen engine CAPABILITY (`src/babylon/engine/systems/` and
`src/babylon/formulas/` untouched); `src/babylon/engine/scenarios/` is deliberately in scope —
that estate is post-freeze-ACTIVE (Scout 5 verified), the freeze covers capability, not qa glue.

- [ ] **Step 1: Red test:**

```python
def test_org_probe_seeds_two_orgs_visible_to_the_graph() -> None:
    state, config, defines = create_org_probe_scenario()
    assert len(state.organizations) == 2
    kinds = {type(o).__name__ for o in state.organizations.values()}
    assert kinds == {"CivilSocietyOrg", "StateApparatus"}
    graph = state.to_graph()
    org_nodes = [n for n, d in graph.nodes(data=True)
                 if d.get("_node_type") == NodeType.ORGANIZATION.value]
    assert len(org_nodes) == 2
```

- [ ] **Step 2: Run red** — `mise run test:q -- tests/unit/engine/scenarios/test_org_probe.py`.
- [ ] **Step 3: Implement** — a compact standalone factory (do NOT touch `_legacy_wayne.py` — it
  is shared substrate under four goldens): one `SocialClass` block + one territory + a
  `CivilSocietyOrg` (INFORMAL, mutual aid, cohesion 0.5, cadre_level 0.1, budget 100.0) + a
  `StateApparatus` (violence 0.6, surveillance 0.5, `FactionBalance(0.2, 0.6, 0.2, 0.5, 0.5)`,
  rng_seed 0) — the Wayne constructors' values, constructed fresh from the public models;
  deterministic ids (`org/probe-civil`, `org/probe-state`); `SimulationConfig(rng_seed=42)`;
  default `GameDefines`. Use `NodeType.*` everywhere (never raw strings — vocabulary sentinel).
- [ ] **Step 4: Green + `mise run check:vocabulary`. Step 5: Commit** —
  `feat(scenarios): org_probe factory — two-org world for the byte gates (spec §11)`.

### Task 12: Register in `qa:regression` + generate + ceremony

**Files:**
- Modify: `tools/regression_scenarios.py` (SCENARIOS entry, `create_scenario` elif, `SCENARIO_COVERAGE_DATA` rows)
- Create (generated): `tests/baselines/org_probe.json`, `tests/baselines/dense/org_probe.csv`

**Interfaces:** Consumes Task 11's factory.

- [ ] **Step 1:** Add the `SCENARIOS` entry (`"org_probe": {"description": "Two-org world — the
  Organization estate's byte-gate anchor (spec §11)", "factory": "create_org_probe_scenario",
  "defines_overrides": {}}`) + the dispatch `elif` + import.
- [ ] **Step 2: The dead-column trap (Scout 5 item 6):** run
  `uv run python tools/regression_test.py generate --dense --scenario org_probe` (check the
  tool's per-scenario flag; if generation is all-scenario only, run full generate and discard
  non-org diffs) — expect the `SystemExit(1)` dead-column abort; add `ScenarioCoverage` rows for
  `org_probe` declaring the legitimately-at-rest columns the run reports, with reasons ("two-org
  probe world; X is structurally at rest because …") — iterate until generation completes.
  Anything at rest you can't honestly justify is a STOP (it may be a real finding).
- [ ] **Step 3:** Re-run compare: `mise run qa:regression` → 12/12 (11 old byte-identical, new
  one green). `mise run test:q -- tests/unit/tools/test_dense_goldens.py` green (dense CSV committed).
- [ ] **Step 4: Ceremony commit** — stage baselines + tool edits;
  `python3 tools/generate_ceremony_message.py --slug org-probe-anchor --summary "org_probe joins qa:regression — the Organization estate's first byte-gate anchor (spec §11, issue #513)" | git commit -F -`;
  verify HEAD moved and the trailer is present.

### Task 13: The vault gate + CI lane

**Files:**
- Modify: `tools/vault_regression.py` (SCENARIOS 3-tuple, `_bake_org_probe` mirroring `_bake_single_county` 79-114, `_bake` dispatch elif ~173-185, **generalize `_build_manifest`'s hardcoded tick ternary at :208 to a per-scenario dict**)
- Modify: `.mise.toml` (`qa:vault-regression-ci` also compares `org_probe`)
- Create (generated): `tests/baselines/vault/org_probe/manifest.json`

- [ ] **Step 1: Red-ish probe** — run `uv run python tools/vault_regression.py generate` with the
  registration in place; inspect the fresh `tests/baselines/vault/org_probe/` — it MUST contain
  `organization/org__probe-civil.md`-style pages (the baker renders org nodes generically,
  Scout 5 §2 — if `org_pages` is empty, STOP: the seed didn't reach the graph).
- [ ] **Step 2:** Fix the `_build_manifest` ternary to `TICKS_BY_SCENARIO: Final[dict[str, int]]`
  (5 ticks for org_probe, matching single_county); wire the CI task:
  `run = "uv run python tools/vault_regression.py compare --scenario single_county && uv run python tools/vault_regression.py compare --scenario org_probe"`
  (match the existing arg shape; if `--scenario` accepts one value only, two invocations as shown).
- [ ] **Step 3:** `mise run qa:vault-regression-ci` green; `single_county`/`detroit_tri_county`
  manifests byte-identical.
- [ ] **Step 4: Ceremony commit** — slug `org-probe-vault`, summary "the vault gate renders its
  first organization pages — org_probe manifest blessed (spec §11)".

---

## Self-review record

- **Spec coverage:** §1 Q12 → Tasks 1–6; Q1 (kind field + eligibility seam) → Tasks 4/6; Q15
  (six edges) → Tasks 2/10; §2 vocabulary registration → Tasks 7–8; §9 rows "Enum deffield row",
  "Vocabulary ceremonies", and the §11 hash anchor → Tasks 1–2, 10, 12–13. NOT this plan (later
  trains, per the spec): budget fields, apparatus hyperedge, doctrine graph-native, state verbs,
  I.16 amendment ceremony (drafted in ADR<N+1>'s references only), endgame floor, data tiers.
- **Placeholder scan:** none — where an executor must resolve something at execution time
  (D-row/ADR numbers, exact error-code numbers, fixture field names), the plan says exactly how
  to resolve it and why it cannot be hard-coded (constraint 5).
- **Type consistency:** `EnumTypeId`/`EnumRegistry::{declare,resolve,ordinal,member,name}` used
  identically in Tasks 3–8; `LoadedScenario.enums`/`vocabulary` consistent across 4/7/8;
  `create_org_probe_scenario` name identical in Tasks 11–12.
