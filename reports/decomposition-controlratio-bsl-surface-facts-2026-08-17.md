# Decomposition (@11.0) + ControlRatio (@12.0) BSL Surface Facts — Verified 2026-08-17

Task 0 of the joint port train (worktree `/home/user/projects/game/wt-decr`, branch
`feature/decomposition-controlratio-port`, base `dev@1242e0c7`). Discharges the two
INADEQUATE-COVERAGE re-reads the survey names as owed before MB-3 opens
(`reports/port-estate-survey-2026-08-12.md:346-354`, §6): Decomposition's `babylon-bsl`
archaeology and ControlRatio's D102/write-path archaeology. Read-only (`Read`/`grep` only, no
`cargo`), every citation re-verified at the byte **in this worktree**, none merely copied from the
plan. Where the plan (`docs/superpowers/plans/2026-08-17-decomposition-controlratio-port.md`) and
this tree disagree, the tree wins — see **§3 CORRECTIONS**.

**Headline: nothing in the plan's substance is wrong.** Every load-bearing claim in the plan's §4
(BLOCKERS) and §5 (byte-order hazard) checks out byte-for-byte. The corrections found are all
**citation drift** — a handful of the plan's file:line pointers land on the wrong function in this
tree (functions moved between when the plan's sources were read and now) — plus one **new,
positive finding**: Step 4's registration-symbol question, which the plan's own self-review
flagged as unresolved, is now settled definitively in favor of the hyphen.

---

## 1. Decomposition's owed sources

### 1.1 The six refused verbs (`structural_verbs.rs:1723-1730`)

```rust
pub(crate) const DEFERRED_SHAPE_VERBS: [&str; 6] = [
    "add-node",
    "remove-node",
    "add-edge",
    "remove-edge",
    "add-hyperedge",
    "remove-hyperedge",
];
```

Confirmed exactly at the cited lines. The doc comment (`:1717-1722`) names the reason: "deferring a
MINTING verb needs a placeholder-id scheme that repair does not specify."

### 1.2 The refusal, verbatim (`check_no_deferred_shape_verbs`, `structural_verbs.rs:1759-1776`)

The gate (confirmed at `:1759`, matching the plan's citation exactly):

```rust
pub fn check_no_deferred_shape_verbs(rule: &SExpr) -> Result<(), String> {
    if let Some(verb) = find_deferred_shape_verb(rule) {
        return Err(format!(
            "({verb} …) is one of the six graph-shape verbs Task 12's \
             collect-then-apply pre-state repair (§4.2 chapter C4) does \
             not yet serve — deferring a MINTING verb needs a \
             placeholder-id scheme this repair does not specify, so \
             run_tick's two-pass split cannot defer {verb} the way it \
             defers update-node. Refused HERE, at load (§3's own law: \
             every check in this chapter runs at content load, before any \
             tick executes), rather than letting the rule load clean and \
             abort the first tick whose guard admits a subject. The \
             follow-on that will serve {verb} is the placeholder-id \
             design EffectExecutor::collect_effects's own doc escalates."
        ));
    }
    Ok(())
}
```

Called **unconditionally** at `rule_pipeline.rs:269`
(`check_no_deferred_shape_verbs(&rule).map_err(LoadError::DeferredShapeVerb)?;` — confirmed exact
line), inside `load_rule_form`, before any tick executes. This is the load-time gate; a defense-in-depth
runtime refusal also exists in `EffectExecutor::collect_items` per the doc comment, but the load gate
fires first for every rule `load_rule_form` accepts.

**`GraphSubstrate` (`babylon-graph/src/substrate.rs`) DOES declare all six as trait methods**
(`add_node::72,80`; `remove_node::103`; `add_edge::111`; `remove_edge::124`; `add_hyperedge::235`;
`remove_hyperedge::246` — confirmed by `grep -n 'fn add_node\|fn remove_node\|...'`). The Rust-API
surface exists; the BSL-content surface is what is refused. This is exactly BLOCKER-1's shape: the
substrate CAN mint, content MAY NOT ask it to.

### 1.3 The served query heads (`evaluator.rs:546`)

```rust
const SERVED_QUERY_HEADS: [&str; 3] = ["nodes", "neighbors", "edges"];
```

Confirmed exactly at line 546. Doc comment (`:532-545`) confirms `edges` joined this table "at T2,
issue #559" — i.e. `edges` is now served (as a query operand only, never a bare `<expr>`), leaving
`UNSERVED_EXPRESSION_HEADS` (`:523-530`) at exactly 6 entries:

```rust
const UNSERVED_EXPRESSION_HEADS: [(&str, &str); 6] = [
    ("the", "slice 2"),
    ("hyperedges", "slice 3"),
    ("members-of", "slice 3"),
    ("hyperedges-of", "slice 3"),
    ("metric-of", "slice 3"),
    ("membership-field-of", "slice 4"),
];
```

`("the", "slice 2")` is the entry this train cares about (BLOCKER-2). Confirmed exactly at `:523-530`.

### 1.4 The `Int ÷ Int` module-doc note (`evaluator.rs:35-37`)

```
//! - `Int ÷ Int` has no pinned semantics: truncation is never implicit
//!   (§3.2) and §3.3 promotes `Int` only "in a binary64 expression". It is
//!   a loud error here pending the Phase-1 review.
```

Confirmed exactly at lines 35-37 (the plan's citation is byte-exact here).

---

## 2. ControlRatio's owed sources

### 2.1 D102 — DISCHARGED, but the plan's own line citations for `typecheck.rs` have drifted (see §3.2)

The real discharge documentation in this tree lives at `typecheck.rs:862-868` (comment) and
`:877-919` (the two proving tests), not at the plan's cited `:246-289` (which in this tree is
`field_ref_name`/`resolve_field`/`check_selection_scores`'s doc/signature — real, but not about
D102). Verbatim, `typecheck.rs:862-868`:

```rust
    // ---- D102 discharge (Task 1, P27 territory-port train): field-of over
    // an enum-declared field now TYPECHECKS AS THE ENUM, not `Real`, and not
    // refused. `check_no_field_of_on_enum_field` (the unconditional D102
    // deferral gate) is deleted rather than narrowed: score-position (D46)
    // and arithmetic (D118/`apply_arith`) are each enforced by their OWN
    // independent mechanism below, so nothing was left for a third gate to
    // decide once the deferral itself lifted.
```

`rule_pipeline.rs:293-301` (the plan's citation for the D102 gate) **does** land correctly — this
is the comment block right after `check_reference_comparisons` explaining the same discharge:

```rust
    check_reference_comparisons(&rule, ctx.types, &bindings).map_err(LoadError::Type)?;
    // §2.13 (D101/D102): the D102 field-of-on-enum-field DEFERRAL gate that
    // used to run here is DISCHARGED (Task 1, P27 territory-port train) —
    // `field-of` over an enum-declared field now typechecks (the §2.7
    // classifier types it `Enum`, `score_class::classify`) and evaluates
    // for real (`evaluator::field_of_node`). Its two surviving refusals
    // each have their own independent mechanism, not a third load gate:
    // D46/E-TYPE-016 (`check_selection_scores`, above) refuses it as a
    // select-max/select-min SCORE; ...
```

### 2.2 The binding law that actually shapes the census: `field_ref_for`'s compound-body refusal (D138) + `E-TYPE-044` (`EnumFoldBody`)

Two **independent** mechanisms combine to reject the "int-ordinal `role`" workaround both 2026-08-12
inventories recommend — confirmed as two separate code paths, not one:

**(a) `field_ref_for` (`rule_pipeline.rs:633-686`)** reduces a fold body to a declared field's kind
through exactly three shapes (bare `<qname>`, `field-of` accessor, nested carrying-fold); anything
else — including an `if`-based filter — is `None`, turned into `compound_fold_error()`
(`:764-778`, uncoded, register row **D138**, confirmed at `docs/reference/bsl-language.rst:6735`).
This refuses a *compound* (arithmetic/conditional) fold body.

**(b) `TypeCode::EnumFoldBody` / `E-TYPE-044`** — a **separate** check refuses a fold `sum`/`mean`/
`min`/`max` whose body IS a legal (non-compound) field reference, but that field is **enum-typed**.
Confirmed live and reachable through BOTH read routes in `rule_pipeline.rs`'s own test module
(`enum_fold_body_tests`, `:1021-1128`):
- Route 1, a `:field`-bound symbol: `a_fold_sum_over_a_field_bound_enum_symbol_refuses_at_load_e_type_044`
  (`:1087-1104`) — `(fold sum (nodes NodeType/ORGANIZATION) kind)` where `kind` binds
  `organization/kind` (enum) refuses with `TypeCode::EnumFoldBody`, `spec_code() == "E-TYPE-044"`.
- Route 2, a `field-of` accessor (newly reachable now that D102 lifted the load-refusal that used to
  block this route before it ever reached the fold-kind law):
  `a_fold_sum_over_a_field_of_enum_accessor_refuses_at_load_e_type_044` (`:1111-1128`) — same verdict.

**Together these two laws close BOTH escape routes for an enum-typed `role`/`SocialRole` field
inside a fold body**: read it directly (E-TYPE-044 refuses) or wrap it in a filter/conditional to
dodge the enum-kind check (D138's compound-body refusal catches that instead). This is exactly why
this train's `p01-la-census`/`c01-prisoner-census` rules (plan §3) gate `role` on the **subject
side** (`when (= role SocialRole/...)`, never inside a fold body) — the same design Production's
own port train independently converged on for the identical reason
(`reports/production-bsl-surface-facts-2026-08-12.md:176-194`). **The int-ordinal `role` workaround
is correctly rejected by the plan — confirmed unnecessary and confirmed it would contradict landed
content** (`production-conformance.bscn:110` uses the real `SocialRole` enum on the subject side).

### 2.3 `field_of_node` and the write catch-all (evaluator.rs) — one citation lands exactly, one has drifted

**`evaluator.rs:1315-1320` is exact** — this is the tail of the doc comment on
`check_node_referent_type`, which IS the shared "write catch-all" the brief asks for:

```rust
/// §2.10 discipline 1, shared by every accessor AND update verb whose
/// referent is a reference: the qname's owning type (§2.9) must match the
/// referent's declared type. ... `field_of_node` (§2.10) and `structural_verbs::
/// update_node` (§2.7's worked example, Task 11) share this exact
/// comparison, reusing `tick::namespace_to_node_type`'s rendering rather
/// than a third one.
///
/// # Errors
///
/// `E-EVAL-033` if `id` names no live node, or if it does but is not of the
```

Confirmed live on both the read side and the write side by direct call-site grep:
`field_of_node` calls it at `evaluator.rs:1381`; `structural_verbs::update_node` calls the
identical function at `structural_verbs.rs:596` and `:909`. **This is the write catch-all**: one
function, `check_node_referent_type`, enforces the qname-owner/referent-type agreement for both
reads (`field-of`) and writes (`update-node`).

**`evaluator.rs:1274-1292` has drifted**: in this tree that range is `eval_field_of` (the dispatch
function that evaluates the referent and matches on `NodeRef`/`EdgeRef`/`HyperedgeRef`), not
`field_of_node` itself. `field_of_node` — the function that actually reads the graph attribute and
type-coerces it via `tick::bind_field_value` — is at **`evaluator.rs:1375-1404`** in this tree.

**`evaluator.rs:1594-1632` does not contain a write catch-all in this tree.** That range is the
`Int ÷ Int` refusal arm inside `arith_int` (`:1580-1584`) plus the `arith_real`/`arith_currency`
helpers (division-by-zero and non-finite guards, `:1591-1632`) — real code, correctly documenting
the division-safety discipline this train's `avg-organization = prisoner-org-weighted /
prisoner-population` expression rides on, but not a *write* catch-all. The actual enum-arithmetic
catch-all `rule_pipeline.rs`'s own D102-discharge comment points at (`apply_arith` "refuses
`Value::Enum` unconditionally") is `apply_arith`'s final match arm at **`evaluator.rs:1560-1565`**:

```rust
        _ => match (real_lane(lhs), real_lane(rhs)) {
            (Some(a), Some(b)) => arith_real(op, a, b),
            _ => Err(EvalError::plain(format!(
                "no arithmetic is defined on ({op} {lhs:?} {rhs:?})"
            ))),
        },
```

None of this changes the plan's conclusions — the D46/D118 refusals it names as "surviving" are
real and confirmed live — only the exact byte pointers for two of the six ControlRatio citations
needed correction.

---

## 3. CORRECTIONS

Recorded per Task 0's own discipline: the tree wins over the plan wherever they disagree. All six
items below are **citation-precision** corrections (a line range points at the wrong function after
intervening commits shifted the file), never substance corrections — every load-bearing claim in
the plan's own §4/§5 is independently reverified TRUE in §1/§2 above.

1. **BLOCKER-1 — no correction, exact match.** `DEFERRED_SHAPE_VERBS` (`structural_verbs.rs:1723`),
   `check_no_deferred_shape_verbs` (`:1759`), and its call site (`rule_pipeline.rs:269`) all land
   exactly where the plan cites them. The survey's row 11.0 ("PORTABLE WITH D-RECORDS — none
   blocking") is confirmed **wrong** — decomposition inventory never read `structural_verbs.rs`,
   exactly as the plan's own §4 callout states.

2. **D102 discharge — the plan's `typecheck.rs:246-289` citation has drifted; use `:862-919`
   instead.** In this tree, `:246-289` is `field_ref_name`/`resolve_field`/`check_selection_scores`'s
   doc and signature — unrelated to D102. The real discharge comment + proving tests are at
   `:862-868` and `:877-919` (§2.1 above, quoted verbatim). `rule_pipeline.rs:293-301` is exact,
   no correction needed there.

3. **`field_of_node` — the plan's `evaluator.rs:1274-1292` citation names the wrong function.**
   That range is `eval_field_of` (the `field-of` dispatcher); `field_of_node` itself (the function
   that actually reads the node attribute and applies the declared-type coercion) is at
   `evaluator.rs:1375-1404`. `evaluator.rs:1315-1320` — the "write catch-all" citation — IS exact
   (§2.3 above): the tail of `check_node_referent_type`'s doc comment names the function literally
   shared between the read path (`field_of_node`) and the write path
   (`structural_verbs::update_node`, confirmed calling it at `structural_verbs.rs:596,909`).

4. **The plan's `evaluator.rs:1594-1632` citation does not contain a write catch-all in this
   tree.** That range is the `Int ÷ Int` refusal plus `arith_real`/`arith_currency`'s
   division-safety guards. The enum-arithmetic catch-all `apply_arith` uses (the mechanism
   `rule_pipeline.rs`'s own D102 comment names) is at `evaluator.rs:1560-1565`. This does not
   change any conclusion the plan draws — `avg-organization`'s division is Real ÷ Real via the
   `arith_real` path this range DOES correctly document — only the specific "catch-all" label was
   pointed at the wrong lines.

5. **Events-are-observable, confirmed exactly as the plan states, with the precise struct shape.**
   `CollectingSink` (`structural_verbs.rs:85-87`): `pub events: Vec<(String, Vec<(String,
   Value)>)>`. `run_once_into(SCENARIO, RULE, &mut graph, &mut sink)` (used by
   `dispossession_conformance.rs:67-71`) asserts events key-by-key
   (`dispossession_conformance.rs:139-200`, e.g. the `("total-transferred".to_owned(),
   Value::Real(3_580.0...))` pattern). All four of this train's events (`SUPERWAGE_CRISIS`,
   `CLASS_DECOMPOSITION`, `CONTROL_RATIO_CRISIS`, `TERMINAL_DECISION`) are testable this way —
   confirmed, no correction.

6. **`EventType`-inert-when-undeclared — confirmed exactly, byte-exact line match.**
   `lib.rs:563-592` is precisely the "Task-10 detonation pin" test: a scenario declaring
   `NodeType`/`EdgeType` vocabularies but **no** `EventType` vocabulary still loads clean and
   fires `(emit EventType/ORGANIZATION_SEEDED (probe 1))` through the real `run_once` production
   seam (`the_task_10_scenario_shape_loads_clean_and_fires_through_run_once`, asserting
   `report.fired == 1`). No correction — this is the plan's most load-bearing "not a blocker"
   claim and it holds exactly.

7. **`the`-unserved / `select-max`-over-`nodes` idiom — confirmed, with a bare-literal-score
   precedent found beyond what the plan cites.** `UNSERVED_EXPRESSION_HEADS` tags `("the", "slice
   2")` at `evaluator.rs:524` (part of the exact `:523-530` block). `manifest.rs`'s `E-LOAD-043`
   singleton guard (`TheAgainstNonSingleton`, doc at `:51-53`, `Display` impl at `:100-104` — both
   byte-exact) only fires when a `.bscn` declares a `manifest` form at all, and §3's finding below
   proves no landed `.bscn` can. The `(select-max (nodes NodeType/X) 1)` idiom (a bare Int literal
   as score, picking a singleton deterministically via D45's ascending-id tiebreak) is directly
   precedented in landed, currently-tested content: `tick.rs`'s `pool-contribution` rule uses
   `(select-max (neighbors self EdgeType/ADJACENCY :out NodeType/ORGANIZATION) 1)`, and
   `r9_chapters.rs` exercises the `(select-max (nodes NodeType/ORGANIZATION) ...)` shape repeatedly
   (`:1091,1109,1126,1141,1150,1176,1310,1342,1357`). No correction to the plan.

8. **`manifest`-is-test-corpus-only — confirmed exactly, exhaustive by construction.**
   `scenario.rs:378-437` (byte-exact range) is the scenario-body form dispatcher; its trailing
   catch-all (`:447-452`) refuses anything outside `defenum | defvocabulary | deffield | defconst |
   node | edge | edge-attr` with the exact message `"a scenario body form must begin with
   \`defenum\`, \`defvocabulary\`, \`deffield\`, \`defconst\`, \`node\`, \`edge\` or
   \`edge-attr\`"`. `check_rule_against_manifest` is `pub use`-re-exported at `babylon-bsl/src/
   lib.rs:54` and exercised only by `manifest.rs`'s own unit tests (`:408-529`) — grepped for call
   sites in `rule_pipeline.rs`, `tick.rs`, and `babylon-tick/src/lib.rs`: **zero hits**. No path
   from a `.bscn` scenario or from the load pipeline ever reaches it. No correction to the plan.

**No Director-gated finding and no reversal of the plan's own verdict anywhere in this pass** — the
corrections above are only line-pointer drift on two of the six ControlRatio citations
(§2.3, items 3-4) and one of the two Decomposition/ControlRatio-shared citations (item 2), plus
independent re-confirmation (not correction) of every other cited fact.

---

## 4. Type-trap list (from plan §4, each re-verified at the byte)

1. **`Int ÷ Int` is a loud error.** `evaluator.rs:35-37` (module doc, byte-exact) states it;
   `arith_int`'s `"/"` arm (`evaluator.rs:1580-1584`) implements the refusal in code:
   `Err(EvalError::plain("Int ÷ Int has no pinned semantics: ..."))`. `avg-organization =
   prisoner-org-weighted / prisoner-population` is Real ÷ (a field-sourced value whose fold result
   type must be confirmed `Real`, not `Int`, before Task 7 writes `c04-terminal` — flagged, not
   resolved here, since it depends on how the census fold declares its result kind).

2. **`:tick` is one of exactly five servable bind-sources.** `tick.rs::check_sources_servable`
   (`:438-481`, byte-exact) confirms the servable set is `{Const (if driver-supplied), Field, Tick,
   TickInCycle, Expr}`; `Metric`, `Year`, `TickOfYear` are refused. Every comparison mixing `:tick`
   with a field-sourced value is legal via this set — no trap beyond confirming the binding source
   itself resolves.

3. **`if` takes exactly three operands; both branches must share one static type.**
   `grammar.rs:649`: `("if", 3, 3, "exactly 3")` — byte-exact, part of the arity table. The
   `(- 0 0c)` / `(- 1 0c)` Currency-promotion idiom (cited from `dispossession.bsl`/
   `lifecycle.bsl:284`) is the landed workaround for a branch that must produce the SAME static
   type as its sibling; this train's `p01`/`c01` census rules (the D127 hash-neutral zero-write
   idiom) will need the identical promotion pattern.

4. **The fold element name defaults to implicit `it`; an explicit name is grammatically legal.**
   `grammar.rs:797-816` (byte-exact): `(fold <fold-op> <query> <elem-name>? <expr> (:weight
   <expr>)?)` — arity 3 (implicit `it`) or 5 (with `:weight`). Confirmed by direct read of the
   arity-check block, which special-cases `head == "fold"` before the general arity table.

5. **`exists` legally takes 1 or 2 operands.** `grammar.rs`'s arity table:
   `("exists", 1, 2, "1 (or 2 with a body)")` — confirmed in the same table block as `if`'s row
   (both read in the same `Read` call, `grammar.rs:640-658`).

6. **`E-LEX-023` caps scaled/suffixed literals at 9 fractional digits; `E-LEX-024` bounds `p`/`i`/`c`
   literals to `[0, 1]`.** `reader.rs:169-201` (the `LexError` variant + `spec_code()` match)
   confirms both codes; `reader.rs:911` confirms the scale bound is literally `scale ≤ 9`. Every
   `carceral/*` coefficient defconst this train needs (`0.15c`, `0.85c`, `0.5c`) is well inside
   both bounds — confirmed by inspection, no literal in the Global Constraints table risks either
   code.

---

## 5. Byte-order analysis (from plan §5, re-derived at the byte, not assumed)

**The sort is real, global (cross-pack), and ascending-byte.** `babylon-tick/src/lib.rs:310`,
inside `prepare_rules` (function starts `:105`, no function boundary between `:105` and `:310` —
confirmed by scanning for `fn `/`pub fn `/`pub(crate) fn ` in that span, zero hits):

```rust
rules.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
```

The comment immediately above it (`:297-304`) confirms this is "the ONE place execution order gets
decided (§4.2, register row D16)" and that `rule_forms` is loaded in "WHATEVER order split_content
returned them (reader-encounter order, unspecified)" before this sort — i.e. concatenating both
pack sources into one `rule_src` string (as the joint-arc scenario does, per plan Task 8) feeds
`prepare_rules` a single `rule_forms` list that this ONE sort call orders globally, across packs,
by rule-id byte comparison. Since `"control-ratio/..."` (`'c'` = 0x63) sorts strictly before
`"decomposition/..."` (`'d'` = 0x64) under `<[u8]>::cmp`, **Pack B's rules do execute before Pack
A's within a tick when both are loaded together** — confirmed exactly as the plan's §5 states, not
merely asserted.

**The hazard's own scope is confirmed narrow.** The plan's claim that the only cross-pack datum is
`institution/decomposition-fire-tick`, read by Pack B behind a `tick >= fire-tick +
control-ratio-delay` gate, is a content-design claim (Pack A/B don't exist yet in this worktree —
`content/rules/decomposition.bsl` and `content/rules/control-ratio.bsl` are both Task-1-onward
deliverables per the File Structure table) rather than something Task 0 can verify at the byte; it
is accepted as stated and flagged for Task 8's own executable constraint test (plan Task 8, Step 4)
to prove rather than assume, exactly as the plan itself already requires.

---

## 6. The `deffield` surface (`scenario.rs:899-972`) and the two-readers fact

**`load_deffield` (`scenario.rs:899-972`, byte-exact range)** is the `.bscn` scenario-file reader.
Confirmed exhaustive grammar, by direct read of the full function body:

- `(deffield <qname> <type> <intensive|extensive>)` — six non-enum type tokens (`:939-951`): `int`,
  `real`, `probability`, `intensity`, `coefficient`, `currency`. Any other symbol in the type slot
  is a load error naming exactly this list.
- `(deffield <qname> enum <EnumTypeName>)` — the seventh accepted shape (`:910-931`), requiring the
  named enum type to have been declared by an earlier `(defenum ...)` in the same source
  (`E-LOAD-054` otherwise).
- **No `bool` type token anywhere in this function.** Confirmed by reading the full 74-line body:
  the type-token match arm (`:939-951`) has no `"bool"` case, and the function has no keyword
  branch of any kind beyond the mandatory 4th slot (`ty`) and 5th-position `kind` check — **no
  `:optional`, no `:default`, no fifth operand of any kind is read or accepted.**
- Duplicate `deffield` for the same qname is refused (`:966-970`).

**`declarations.rs:648-665` (byte-exact range) is a DIFFERENT reader — `parse_type_name`, consumed
by `parse_deffield` (rule-file top-form `deffield`, `declarations.rs:446`) and by
`metrics.rs`'s metric declarations (`metrics.rs:43`), never by `scenario.rs`.** Confirmed by
`grep`ing every call site of `declarations::parse_type_name`/`parse_deffield` across the crate:
`scenario.rs` never imports or calls it. This reader accepts **eight** rows (`:652-675`):
`int`, `bool`, `currency`, `probability`, `intensity`, `coefficient`, `real`, `enum` — **`bool` IS
legal here**, directly contradicting the scenario-file reader's own seven-token table.

**This is the disagreement the plan's Task 0 brief anticipated and asked to be recorded "as a fact
about two readers, not a usable route."** Confirmed as exactly that: this train's carrier and
per-node census fields are declared inside `.bscn` conformance scenarios (Task 1 onward, per the
File Structure table), which are hydrated only through `scenario.rs::load_deffield` — the
seven-token, no-`bool` reader. `declarations::parse_type_name`'s `bool` row governs a **different**
surface (`.bsl` rule-file top-form `deffield`s and `metric` declarations) that this train's content
plan does not use for field declarations at all (its `deffield` forms all live inside `.bscn`
scenarios, per plan §3's rule layout and §2's reformulation). **`bool` thus remains unusable
for this train's carrier/census fields regardless of its existence in the other reader** — the
existing `int extensive` 0/1 workaround (matching every other landed pack's `active`/flag fields)
is the only route, exactly as the plan's own bool-avoidance already assumes throughout §2/§3.

---

## 7. Step 4 — the registration-symbol hyphen verdict

**VERDICT: YES — a rule-id first segment (and any qname segment generally) admits a hyphen.**
`control-ratio` is fully legal; **no fallback to `controlratio` is needed**, and every later task
in this plan should use `control-ratio` exactly as §3's rule layout already specifies. This settles
the plan's own self-review's "could not verify" item (`docs/superpowers/plans/
2026-08-17-decomposition-controlratio-port.md:525`) with three independent, converging proofs, all
re-verified at the byte in this worktree — no `cargo` run was needed:

1. **Grammar-level.** `reader.rs::validate_symbol` (`:524-537`, byte-exact):
   ```rust
   fn validate_symbol(s: &str) -> Result<(), SymbolIssue> {
       let mut chars = s.chars();
       match chars.next() {
           Some(c) if c.is_ascii_lowercase() => {}
           _ => return Err(SymbolIssue::Invalid),
       }
       if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
           return Err(SymbolIssue::Invalid);
       }
       ...
   }
   ```
   A symbol segment must start with a lowercase ASCII letter, then any run of lowercase letters,
   digits, or `-`. `control-ratio` — `c` followed by `ontrol-ratio` — satisfies this exactly.
   `classify_name` (`:581-613`) splits a qname on `/` and validates each segment independently
   through this same function, so `control-ratio/c01-crisis` (2 segments, well under the 4-segment/
   128-byte cap, `:605`) lexes to a legal `Atom::QName`.

2. **Registration-level, already live.** `babylon-tick/src/lib.rs:270` already registers a
   hyphenated system name: `"social-class".to_owned()`, inside `prepare_rules`'s `systems` HashSet
   (confirmed no function boundary between `:105` (`prepare_rules` start) and `:270`). Neither
   `"decomposition"` nor `"control-ratio"`/`"controlratio"` is present anywhere in this HashSet
   today (grepped, zero hits) — both are genuinely new registrations for Task 1, matching the
   plan's own File Structure line.

3. **Usage-level, already executed through the real pipeline.** `edge_lane_e2e.rs` contains landed,
   currently-passing rules whose id's first segment is the SAME hyphenated string:
   `(rule social-class/edges-fold-e2e ...)` (`:70`) and `(rule social-class/edge-between-resolves-
   e2e ...)` (`:120`), both anchoring under the registered `"social-class"` system via the
   anchor-default path (no explicit `(anchor ...)` needed) — this is not merely a grammar
   theory reading; it stands as a concrete, exercised precedent for exactly this shape (hyphenated system
   registration string + hyphenated rule-id first segment + anchor-default resolution).

No `cargo test` run was needed to settle this — the three proofs above are conclusive at the source
level, consistent with the brief's instruction to verify via the lexer/parser path rather than a
throwaway load test where a cheap byte-level check already settles it.

---

## Summary for later tasks

- **BLOCKER-1 through BLOCKER-5 (plan §4): all independently reverified TRUE at the byte.** No
  reversal of any plan verdict.
- **CORRECTIONS are citation-precision only** (§3, items 2-4): two `evaluator.rs` line ranges and
  one `typecheck.rs` line range in the plan point at functions that have since moved; the corrected
  line numbers are recorded above for Tasks 1-9 to cite instead.
- **The int-ordinal `role` workaround is confirmed rejected**, for the precise combined reason
  (D138 compound-body refusal + E-TYPE-044 EnumFoldBody), not merely "D102 still blocks it" (D102
  is discharged; two OTHER, independent laws are what actually close this route).
- **`control-ratio` (hyphenated) is confirmed legal for the namespace** — use it as specified in
  plan §3, no `controlratio` fallback.
- **The `deffield` bool gap is confirmed real and confirmed irrelevant** — this train's fields all
  route through the scenario-file reader, which has no `bool`, regardless of the rule-file reader's
  `bool` row.

No Director-gated finding surfaced in this pass. No file under `tests/baselines/**` was touched.
This dossier makes no code or content changes — read-only per Task 0's own scope.
