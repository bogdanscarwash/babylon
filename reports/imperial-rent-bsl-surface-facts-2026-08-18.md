# Imperial Rent BSL Surface Facts — Verified 2026-08-18

Task 0 of the ImperialRent BSL port train (`docs/superpowers/plans/2026-08-18-imperialrent-port.md`,
rev 2). Scout pass against `dev` @ `b5a3268a` (worktree
`/media/user/data/worktrees/wt-imperialrent`, branch `feature/imperialrent-port-bsl`), read-only
(`rg`/`Read` only, no cargo build or test — the box runs two other builds and this task's gate is
`vale`, not `cargo`). Every fact below carries a `file:line` anchor as measured in this worktree on
2026-08-18; where the brief's or the plan's own citation drifted, both numbers appear so a later
task can re-anchor without re-deriving. This dossier produces the CORRECTIONS every later task in
the train cites and settles the four questions the plan deliberately left open rather than guessing.

---

## CORRECTIONS

Read this section first — later tasks cite it directly.

1. **This dossier STRIKES B7 outright rather than merely discharging it.** No landed pack publishes a scalar
   `social-class/class-consciousness` qname, but the reason is not absence — a landed pack already
   re-points the frozen scalar to an existing field. `solidarity.bsl:18-24` documents the re-modelling
   verbatim: frozen `ideology.class_consciousness` ports to the **already-declared**
   `social-class/revolutionary` field. `r01-extraction`'s consciousness term reads
   `social-class/revolutionary`, not a net-new field. §8's rule table and its "Net-new" field list
   both need this fix (see Step 3 below for the full evidence chain).
2. **The ADR allocation is NEXT-FREE-AT-LANDING, not absolute — and it already moved once.** At this
   dossier's original `b5a3268a` baseline, `ai/decisions/index.yaml` tailed at ADR213 and the plan
   assumed ADR214. By the time this fix round merged `origin/dev` (`45c0b64d`, pulling in PR #665),
   `ai/decisions/ADR214_national_incidence_artifact_train.yaml` had already landed — **this train's
   ADR number is ADR215 as of 2026-08-18**, and whichever task actually files the ADR must
   re-measure `ai/decisions/index.yaml`'s tail at filing time rather than trust this number. The
   D181–D201 half stays clean: `#665` touched only `ai/decisions/index.yaml`, `bsl-language.rst`
   still tails at D180 on `origin/dev`, so the 21 contiguous D-rows stay free (Step 5 below). **The
   commit body of `d0adc52f` still states the ADR214 claim absolutely — this correction supersedes
   that claim; the landed commit message itself cannot be rewritten.**
3. **`edge_lane_e2e.rs`'s own "self-anchored" vector is a `fold`, not a `for-each`.** The plan's §3.2
   citation of `edge_lane_e2e.rs:196-206` as the landed self-anchored idiom is directionally right but
   names the wrong verb: its Shape 3 resolves the per-element edge inside a `fold`, not a `for-each`.
   Precision correction on the earlier draft of this dossier: the file ships four `update-node`
   writes (`:76,126,145,203`, all to `social-class/fold-total`) — what it ships zero of is an
   **edge** write (`update-edge`: zero hits); its own header (`:5`) says only "this ships no
   Solidarity content," not "read-only." The write-side self-anchored idiom (`neighbors` +
   `edge-between` + `update-edge` together) lives in `edge_write_lane_e2e.rs` Shape 2 instead
   (Step 2 below).
4. **Step 4(a)'s deciding axis is literal kind, not lexical position.** The plan frames the open
   question as "is a negative literal legal in `defconst` position." The lexer decides negativity by
   **literal kind** (bare `Int` vs. any decimal/suffixed literal) before any `defconst`-vs-`binding`
   distinction exists. Three landed scenario files already carry negative bare-`Int` defconsts.
   Every suffix's lexer arm unconditionally refuses negative *fractional* literals, in every
   position — no legal BSL spelling of `-0.05c` exists. The plan's `(- 0 x)`/`sub` fallback is the
   **only** legal shape for `austerity-wage-delta`/`crisis-wage-delta` (Step 4(a) below), not a
   caution-driven fallback.
5. **Every other plan citation this dossier checked held, within ordinary ±1–6 line drift.** The
   `p0`/`p4`/`p5`/`p7`-not-`p6`/`p8` attribution (§2.2), the D194/D198/D200 shape questions, the
   namespace list, the hyphen precedent, and the `decomposition.bsl`/`fundamental-theorem.bsl`
   transcriptions all confirm as stated. Later citations note their own drift inline; this section
   does not restate it.

---

## Step 1 — The edge lane's owed read

**Crate disambiguation (D146, repo-relative paths).** Steps 1, 2 and 4(g) cite four test-file
basenames without their crate path; two crates carry `tests/` directories, so a bare basename is
ambiguous. For the record: `edge_lane_e2e.rs`, `edge_write_lane_e2e.rs` and `query_lane_e2e.rs` are
all `rust/crates/babylon-tick/tests/<name>`; `r9_chapters.rs` is
`rust/crates/babylon-bsl/tests/r9_chapters.rs` — a **different crate**. Every bare basename below
resolves per this table.

### `materialize_edges` — `rust/crates/babylon-bsl/src/query.rs:263-294`

The brief cited `:135, :282`; the function itself spans `:263-294` (`:282` sits inside the real body,
at the `require_graph(env, "edges")?` line — `enum_member(type_ref)?` is one line earlier, `:281`;
`:135` does not correspond to this function).

```rust
fn materialize_edges(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    fuel: &mut u64,
) -> Result<Vec<Element>, EvalError> {
    charge(fuel, cost::QUERY_BASE)?;
    let [_, type_ref, extra @ ..] = items else {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>?) — missing the EdgeType operand",
        ));
    };
    if !extra.is_empty() {
        return Err(EvalError::plain(
            "(edges <enum-ref> <edge-pred>) — the element-predicate operand is a real §2.6 \
             production this evaluator does not yet serve (no exercised vector or content rule \
             needs it); T2 serves the unpredicated (edges <enum-ref>) form only",
        ));
    }
    let edge_type = enum_member(type_ref)?;
    let graph = require_graph(env, "edges")?;
    Ok(graph
        .edges(edge_type)
        .into_iter()
        .map(|(source, target)| {
            Element::Edge(EdgeKey {
                source,
                target,
                edge_type: edge_type.to_owned(),
            })
        })
        .collect())
}
```

The doc comment above it (`:257-262`) states the function performs no sort of its own —
`GraphSubstrate::edges` already returns a canonically sorted `Vec<(NodeId, NodeId)>`.

### `SERVED_QUERY_HEADS` and `UNSERVED_EXPRESSION_HEADS` — `evaluator.rs`

Both line numbers match the brief exactly.

```rust
// evaluator.rs:544
const UNSERVED_EXPRESSION_HEADS: [(&str, &str); 6] = [
    ("the", "slice 2"),
    ("hyperedges", "slice 3"),
    ("members-of", "slice 3"),
    ("hyperedges-of", "slice 3"),
    ("metric-of", "slice 3"),
    ("membership-field-of", "slice 4"),
];

// evaluator.rs:567
const SERVED_QUERY_HEADS: [&str; 3] = ["nodes", "neighbors", "edges"];
```

The served heads, verbatim: **`nodes`, `neighbors`, `edges`**. The doc comment (`:553-566`) states
these serve only as the query operand of an iterating form (`fold`/`exists`/`forall`/`select-max`/
`select-min`/`for-each`), never as a bare `<expr>` — `<expr>`'s own grammar production has no query
alternative. `edges` joined the served set at T2 (issue #559); `the`, `hyperedges`, `members-of`,
`hyperedges-of`, `metric-of` and `membership-field-of` stay unserved, deferred to their own slices.

### `for-each` — `structural_verbs.rs:511-528` (prelude `:552-572`)

The brief bracketed the whole treatment as `:473-570`; the measured breakdown: the doc comment naming
the grammar form starts at `:473`, the `fn for_each` body itself sits at `:511-528`, and the shared
`for_each_prelude` (destructuring plus query materialization, used by both the execute path and the
collect path) sits at `:552-572`. The brief's bracket holds as a span around both.

Grammar form, quoted verbatim (`:473`):

```
/// `(for-each <query> <elem-name>? <effect-item>+)` (§2.8 chapter C6).
```

Confirmed as the real arity/argument form: query, an optional `:as`-stripped element name, one or
more effect items — enforced at `:565-569` ("requires at least one effect item").

```rust
// structural_verbs.rs:511-528
fn for_each(
    &mut self,
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    graph: &mut dyn GraphSubstrate,
    sink: &mut dyn EventSink,
    fuel: &mut u64,
) -> Result<(), EvalError> {
    let (elem_name, effect_items, elements) = Self::for_each_prelude(items, env, host, fuel)?;
    for element in elements {
        let child = crate::evaluator::with_element(env, elem_name.clone(), element);
        for effect_item in effect_items {
            self.execute_item(effect_item, &child, host, graph, sink, fuel)?;
        }
    }
    Ok(())
}
```

Doc comment (`:504-509`): application order is total — the body runs once per element in iteration
order (outer), and its own items apply in source order (inner); an empty query applies nothing and is
not an error. This `for_each` is the execute path's own copy (immediate-apply, driving unit tests and
the conformance corpus); production `run_tick` calls `collect_item`'s `"for-each"` arm instead
(dispatch at `:840`, doc comment `:474-478`) — both share the same `for_each_prelude`.

### The pre-state law — two paragraphs, both in `structural_verbs.rs`, both matching the brief's lines exactly

**`:112-120`** (`PendingWrite`'s doc, first paragraph — the accumulation-clause paragraph; a second
paragraph immediately follows, `:122-136`, whose own `:129-131` carries the emit-reads-pre-state
sentence the brief also names):

```rust
/// One collected, not-yet-applied `update-node`/`update-edge` mutation
/// (Task 12, P27 Phase 2 query-evaluation plan, §4.2 chapter C4 + §2.8
/// chapter C6; widened to edge targets by T3, ADR198 R3, issue #560). The
/// evaluator has ALREADY reduced the operand expression against the rule's
/// PRE-STATE during collection (`EffectExecutor::collect_effects`); the
/// accumulating ops read the target's CURRENT value at APPLY time (D-row
/// Q2) — §4.2's carrier-accumulation clause is only satisfiable that way:
/// reading the target at collect time would make three subjects each
/// adding to one carrier lose two of the three contributions.
```

**`:740-752`** (the brief cited `:740-751`; the block runs one line further):

```rust
// structural_verbs.rs:738 section header
// ---- Task 12: the pre-state law — collect-then-apply ----

/// COLLECT phase (§2.8 chapter C6 + §4.2 chapter C4): evaluate
/// `effect_items` against `env`'s pre-state, returning the
/// `update-node` writes they would perform WITHOUT applying any of
/// them. This method takes no mutable graph at all — that is what
/// makes "every firing observes the same pre-state" a property of the
/// TYPE, not a convention a caller could violate by forgetting to
/// re-read: nothing this method calls CAN mutate a graph.
///
/// `emit` fires immediately even here (it never touched a graph, and
/// its payload evaluates against the same frozen `env`, matching §2.8's
/// own worked `for-each` example, whose `emit` reads the PRE-scale
/// `solidarity/strength`). `guard` and `for-each` recurse the same way
/// `Self::execute_item` does, over this collecting path instead.
```

Both paragraphs make the same claim — collect-phase evaluation reads pre-state; apply-phase
accumulating ops (`add`/`sub`/`scale`) read the target's current value at apply time. `:112-120` is
`PendingWrite`'s own doc; `:740-752` is `collect_effects`'s own doc. This is the whole pre-state law;
no third site restates it.

### `update-edge` arm — `structural_verbs.rs:660-734` (execute path)

- Grammar: `(update-edge <expr> <qname> <update-op>)`, `update-op ::= (add|sub|set|scale <expr>)`.
- `check_edge_referent_type` (`:679`) enforces that the qname's owner segment matches the referent's
  declared edge type — `E-EVAL-033` on mismatch (mirroring `field-of` over an EdgeRef).
- `set` reads the pre-tick value only for the write log's `previous` probe (`:694-697`), never for the
  new value.
- `add`/`sub`/`scale` (`:698-715`) read `graph.edge_attribute(...)` — the current, apply-time value —
  and refuse arithmetic on an enum field.
- A collect-path arm also exists (dispatch at `:868`, more `update-edge` refusal sites at `:971-994`
  and `:1082`); the execute-path body above is the one the brief's "`update-edge` arm" names most
  directly, and both paths share the `numeric_write_value`/`check_edge_referent_type` helpers.

### `edge_lane_e2e.rs:186-189` — the exact absence-of-endpoint-accessor comment

Matches the brief's lines exactly:

```rust
/// The self-anchored `neighbors`+`edge-between` idiom — §3.8 item 8's worked
/// example, Solidarity's own anticipated read shape: per TARGET node, walk
/// incoming SOLIDARITY neighbours and resolve each edge's strength by key,
/// needing no `(edges …)` iteration and no `source-of`/`target-of` endpoint
/// accessor (the language does not have one — §3.8 item 8's own open item,
/// dossier §8). This is the vector that unblocks Solidarity's own port
/// train.
```

The full paragraph spans `:183-189`; `:186-189` is the exact sentence the brief quotes.

---

## Step 2 — The write lane's owed read

### `edge_write_lane_e2e.rs` — read in full (197 lines), two shapes, both through `run_once_into`

**Shape 1** (`:61-132`, test `shape_1_for_each_over_edges_writes_every_edge_and_the_emit_reads_pre_state`
at `:75`) — `for-each` over `(edges EdgeType/SOLIDARITY)`, both write kinds in one body:

```
(for-each (edges EdgeType/SOLIDARITY)
  (update-edge it solidarity/strength (scale 0.5c))
  (update-edge it solidarity/tension (set 0.75i))
  (emit EventType/PROBE (s (field-of it solidarity/strength))))
```

`scale` routes to the strength field (D143's strength-fork); `set` targets a deffield-declared field
(`solidarity/tension`). The `emit`'s payload (`(field-of it solidarity/strength)`) reads the
**pre-tick** strengths — `0.5` and `0.25` — even though the same body's `scale` writes 0.5→0.25 and
0.25→0.125 in the same tick. The assertion (`:106-107`) checks `payload_s(events[0]) == 0.5` and
`payload_s(events[1]) == 0.25`, both annotated "PRE-scale" in the test's own comments. This is the
emit-reads-pre-state test the brief names.

**Shape 2** (`:152-197`, test `shape_2_edge_between_targets_one_edges_write_and_the_other_stays_honest_null`
at `:170`) — a targeted single-edge write via `edge-between`, both write kinds again:

```
(update-edge
  (edge-between EdgeType/SOLIDARITY self
    (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
  solidarity/strength (add 0.25c))
(update-edge
  (edge-between EdgeType/SOLIDARITY self
    (select-max (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) 1))
  solidarity/tension (set 0.5i))
```

`add` on strength (0.5 + 0.25 = 0.75, exact); `set` on tension (deffield). The untouched second edge's
`tension` was never written and reads loud (`.is_err()`, `:190-195`) — never a default 0.0 (III.11).
Both write kinds — `set`, `add`, `scale` — appear in this one file; `sub` on an edge appears only in
`structural_verbs.rs`'s own unit tests (e.g. `update_edge_sub_on_a_declared_edge_field` at `:4532`),
not in this e2e file.

### `edge_lane_e2e.rs` — the self-anchored vector, Shape 3 (`:196-206`)

```
(fold sum (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS)
  (field-of (edge-between EdgeType/SOLIDARITY it self) solidarity/strength))
```

Per CORRECTIONS item 3: this is a `fold`, not a `for-each`. The file is not write-free — it carries
four `update-node` writes to `social-class/fold-total` (`:76,126,145,203`) — but it writes zero
**edges**; its own header (`:5`) says only "this ships no Solidarity content," not "read-only." The
write-side self-anchored idiom (`neighbors` + `edge-between` + `update-edge` together) lives only in
`edge_write_lane_e2e.rs` Shape 2 above. Both
files share the same resolution shape —
`(select-max (neighbors self EdgeType/X :out NodeType/Y) 1)` then
`(edge-between EdgeType/X self it-or-self)` — one reading, one writing.

### `consciousness-ternary-conformance.bscn` — the `edge-attr` seeding form (`:309,311,313`)

Matches the brief's lines exactly:

```scheme
(edge-attr EdgeType/WAGES employer class-exploited wages/value-flow 9)   ; :309
(edge-attr EdgeType/WAGES employer class-bribed wages/value-flow 12)     ; :311
(edge-attr EdgeType/WAGES employer class-emergent wages/value-flow 8)    ; :313
```

Exact form: `(edge-attr <EdgeType-enum-ref> <from-node-name> <to-node-name> <qname> <value>)` — this
position accepts a bare numeric literal (no `0.0c` suffix). The header comment (`:107-109`) names the
form as D156: the language now serves scenario-side edge-attribute seeding, and this world seeds
three WAGES edges plus declares the `wages/value-flow` edge-attribute deffield.

### The self-anchored idiom in landed content

`consciousness.bsl` uses it three times, spanning `:219-245` (brief cited `:219-233`):

```
consciousness.bsl:219-222
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (guard (> (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength) negligible)
        (update-node self social-class/solidarity-inbox
          (add (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength)))))

consciousness.bsl:231-234
    (for-each (neighbors self EdgeType/WAGES :out NodeType/SOCIAL_CLASS)
      (update-node self social-class/wages-inbox
        (add (field-of (edge-between EdgeType/WAGES self it) wages/value-flow))))
```

The second block is `p2-wages-push` — the plan's own citation of the existing reader of
`wages/value-flow`, confirmed live in content, not merely tested.

`production.bsl`'s single-employer idiom, the one §3.2 specifically names, confirms exactly at
`:216` (the brief's `:172-220` span is an accurate bracket around the whole idiom-bearing region; the
wages-specific precedent sits narrowly at this one line):

```
production.bsl:216
    (update-node (select-max (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS) 1)
```

---

## Step 3 — Discharge B7

**Verdict: STRUCK.** A landed pack already re-points the frozen scalar `ideology.class_consciousness`
to `social-class/revolutionary` — this pack must not mint `social-class/class-consciousness`.

`grep -rn "class-consciousness" rust/crates/babylon-tick/content/` returns exactly one hit — a
negative statement, `solidarity.bsl:18-24`:

```
; THE CONTENT-MODEL RE-POINT (plan §1, D-record 1): frozen
; `ideology.class_consciousness` ports to the ALREADY-DECLARED
; `social-class/revolutionary` field — no `social-class/class-consciousness`
; scalar is minted. This is the frozen engine's OWN identification, not an
; invention here: `ideology.py:382-386`'s comment reads "class_consciousness
; <- revolutionary (delta_r)", implemented at `ideology.py:410`
; (`new_class = min(1.0, current_profile["class_consciousness"] + delta_r)`).
```

`src/babylon/kernel/node_access.py:15-35` — `class_consciousness_from_node`'s own docstring states it
consolidates three identical `_get_class_consciousness_from_node` copies: SolidaritySystem,
StruggleSystem, and **ImperialRentSystem**. `economic.py:286` (Phase 1) calls this exact accessor —
the same one `solidarity.py:140,148` calls, which `solidarity.bsl` already re-pointed. Nothing
distinguishes imperial-rent's re-modelling need from solidarity's for the identical accessor.

Independent corroboration: `ai/decisions/index.yaml`'s `ADR211_solidarity_port_train_handoff` entry
states the same re-point — "the ideology.class_consciousness scalar re-pointed to the already-declared
social-class/revolutionary field (no new scalar minted)."

**Scope caveat: the grep above scoped to `content/` — narrower than "no `.bsl` estate writes this
qname."** `social-class/class-consciousness` does appear three more times in the tree, outside
`content/rules/`: `rust/crates/babylon-bsl/tests/conformance_corpus.rs:91,928,936` (declared
and written by a test fixture) and
`rust/crates/babylon-bsl/tests/conformance/event_bifurcation.bsl:15` —
`(update-node self social-class/class-consciousness (add 0.15i))`. **This does not touch the
strike** — the shipped content pack (`content/rules/*.bsl`, the scope B7 actually governs) is what
`r01` loads against, and ADR211 rules the re-point regardless of what a test-only fixture declares —
but a later task grepping the tree for this qname will find three hits after this dossier said none.

**Correction this forces on the plan (folded into CORRECTIONS item 1):** §8's `r01-extraction` binding
reads `social-class/revolutionary` (already declared `probability`, following the `:default 0.0p`
idiom seen at `consciousness.bsl:186` and elsewhere), not a net-new `social-class/class-consciousness`
field — this dossier strikes B7 outright.

---

## Step 4 — The seven shape questions, settled at the byte

### (a) Negative literals — `defconst` value position

**SETTLED — split answer; the deciding axis is literal kind, not lexical position** (CORRECTIONS
item 4).

**Bare `Int` literals — negative lexes everywhere, `defconst` included, and landed content already proves it.**
`classify_numeric` (`reader.rs:742-797`) strips a leading `-` at `:750-753`; `classify_int`
(`reader.rs:807-818`) applies the sign unconditionally, with no range check. Landed proof
(`grep -rn "defconst.*-[0-9]" rust/crates/`):

- `dispossession-ceiling-matrix-conformance.bscn:33` — `(defconst dispossession/foreclosure-rate -6)`
- `dispossession-negative-weight-conformance.bscn:38` — `(defconst dispossession/weight-foreclosure -1)`
- `dispossession-negative-input-conformance.bscn:50-53,62` — five more negative-`Int` defconsts, in a
  scenario file named for exactly this case.

`scenario.rs::load_defconst`'s `Atom::Int` arm (`:742-745`) carries the value through with only a
stray `:floor`/`:cap` check on a non-Ratio type, never a sign check.

**Fractional (decimal-point) literals — every position universally refuses a negative value — and
this is the finding that matters for `austerity-wage-delta`/`crisis-wage-delta` (both fractional).**
`classify_numeric` requires a kind suffix (`$`, `p`, `i`, `c`, `r`) on any decimal-point literal
(`reader.rs:771-776`, the `BareFloat` refusal). Every suffixed kind refuses a negative value:

- `$` (Currency): `classify_currency` (`reader.rs:823-859`) — `if negative { return
  Err(NegativeCurrency) }` at `:829-835`, unconditional.
- `p`/`i`/`c` (Probability/Intensity/Coefficient): `classify_unit_interval` (`reader.rs:863-908`) —
  `if negative && unscaled != 0 { return Err(out_of_range()) }` at `:888-890`; the domain is
  `[0, 1]`, so only a negative *zero* survives, never an actual negative value.
- `r` (Ratio): `classify_ratio` (`reader.rs:917-943`) — `if negative { return Err(non_positive()) }`
  at `:939-943`, unconditional (open interval `(0, ∞)`).

**Consequence: no legal BSL literal syntax, in any position, spells a negative fractional number.**
`-0.05c`/`-0.15c` cannot lex — not in `defconst`, not in a binding default, nowhere — because the
suffix that would carry the decimal point is exactly the suffix whose lexer arm refuses negativity.
`grep -rn "defconst.*-[0-9]*\.[0-9]" rust/crates/` returns zero hits anywhere in the tree, consistent
with structural impossibility rather than mere lack of precedent.

**Disposition for `austerity-wage-delta`/`crisis-wage-delta`:** declare
`(defconst economy/austerity-wage-delta 0.05c)` / `(defconst economy/crisis-wage-delta 0.15c)`
(positive values) and apply the sign at the use site via `sub`, or via the landed `(- 0 x)`
promotion idiom if a rule must bind a signed value. This is the only legal shape, not a fallback of
caution.

### (b) `for-each` body + nested `(guard <pred> <effect-item>+)` — more than one effect

**SETTLED — yes, arbitrary N ≥ 1, no upper bound.**

Both effect paths handle `guard` the same way, as a slice-pattern destructure with only a
non-empty check, never a cardinality cap:

- Execute path (`structural_verbs.rs::execute_item`, `"guard"` arm, `:407-422`):
  `let [_, cond, nested @ ..] = items.as_slice()` collects every remaining item into `nested`; the
  only refusal is `nested.is_empty()` (`:413-415`). When taken, `:418-420` loops over `nested`, one
  recursive `execute_item` call per item.
- Collect path (`:820-833`, the pre-state collect-then-apply half used inside `for-each` bodies):
  byte-identical shape, same emptiness check (`:825-827`), `:829-831` calls `collect_items` over the
  whole slice.
- `for-each` calls `execute_item` per body item, per element (`:511-528`, `:523-525`), the same
  function whose `"guard"` arm admits N ≥ 1 nested effects — a `for-each` body's guard is not a
  special case.

`(for-each <query> (guard <pred> (emit …) (update-node …) (update-node …)))` — three effects nested in
one guard — is already a supported shape. `r05`'s three-effect guard (one `emit`, two latch writes)
needs no new grammar.

**Precedent note.** The closest landed example of a three-effect group gated by one condition is
`decomposition/p02-superwage-warning` (`decomposition.bsl:247-271`, quoted in full under Step 7), but
its three effects sit as top-level `(effects …)` items gated by the rule's own `(when …)` clause, not
nested inside an explicit `(guard …)` form. That is a valid alternative shape for `r05` too, if its
crisis condition folds into the rule-level `when`; this dossier settles the general capability — N ≥ 1
effects under `guard`, inside a `for-each` body — independently of which shape `r05` ultimately picks.

### (c) `(field-of it social-class/role)` in a `for-each` guard, compared for enum equality

**SETTLED — yes, typechecks and evaluates.** D102's discharge is real and stated in-source, not just
implied: `typecheck.rs:304-308` (inside `check_no_arithmetic_on_enum_field`'s doc) reads verbatim —
"D102 itself was discharged by the Task 1 P27 territory-port train — `field-of` over an
enum-declared field now typechecks and evaluates for real, `evaluator::field_of_node`."

The two named refusals are real but neither applies to an equality comparison:

- **`E-TYPE-016`** (`typecheck.rs::check_one_selection`, `:460-499`) refuses only a
  `select-max`/`select-min` **score** whose static class is not a comparable scalar. Exact text
  (`:493-497`): "a {head} score must be a comparable scalar (Int, Currency, Probability, Intensity,
  Coefficient or Real); this one classifies as {class:?}." `check_one_selection` only walks
  `select-max`/`select-min` forms (`:464-466`) — an `=` comparison inside a `when`/`guard` is outside
  its scope entirely.
- **`E-EVAL-042`/D118** (`typecheck.rs::check_no_arithmetic_on_enum_field`, `:294-339`) refuses only
  `add`/`sub`/`scale` update-ops (write-side arithmetic) targeting an `:enum-type`-declared field
  (`:326-333` destructures `update-node <node> <qname> (<op> <operand>)` against
  `"add"|"sub"|"scale"`). An equality read comparison is a different AST shape entirely and this
  function never inspects `=`/`!=`.

**Landed precedent for the exact pattern.** `(= (field-of it <qname>) <Enum>/<MEMBER>)` is pervasive:
`territory.bsl:130-142` (six occurrences of
`(= (field-of it territory/territory-type) TerritoryType/PENAL_COLONY)` and similar, inside
`exists`/`select-max` predicate and score position); the `self`-anchored form (same `field-of`
machinery via a `:field` binding) at `decomposition.bsl:228,231,234,237,257,329,347,374`,
`control-ratio.bsl:261-263`, `production.bsl:183,212,244` — all `(= role SocialRole/<MEMBER>)`.
`(field-of it social-class/role)` compared via `=` to `SocialRole/CORE_BOURGEOISIE` inside a
`for-each` guard, as `r02`/`r04` need, sits squarely inside this landed, tested pattern.

### (d) What `(field-of (select-max …) …)` returns on a `real extensive` field; the `pool-ratio` division

**SETTLED — a `real extensive` field's static/runtime type is `Real`, unconditionally; dividing two
`Real`s never touches the `Int ÷ Int` refusal.**

Extensive/intensive is a `kind` tag, orthogonal to a field's `type`: `types.rs:281-286` —
`FieldDecl { pub ty: BslType, pub kind: FieldKind }`, two independent fields. `FieldKind` (`:262-276`,
three variants: `Intensive`/`Extensive`/`NotApplicable`) governs only aggregation legality (the
`sum`/`mean` rules — `typecheck.rs`'s module-level doc, `:1-27`), never arithmetic or comparison
legality. `field-of` on a `real extensive` field classifies as `ScoreClass::Scalar`
(`score_class.rs::field_class`, `:99-103`, derived purely from `decl.ty`) — exactly the class
`E-TYPE-016` accepts (`typecheck.rs:494-496`'s allowed list: Int, Currency, Probability, Intensity,
Coefficient, Real). A bare `real extensive` field-of is legal both as an ordinary expression and as a
`select-max`/`select-min` score.

`arith_int` refuses `Int ÷ Int`; `arith_real` does not refuse `Real ÷ Real`; `apply_arith` routes
between the two by runtime **value variant**, not by declared kind. `evaluator.rs::apply_arith`
(`:1674-1725`): the first match arm,
`(Value::Int(a), Value::Int(b)) => arith_int(op, *a, *b)` (`:1676`), fires only when both operands are
the `Value::Int` runtime variant. `arith_int`'s `"/"` arm (`:1738-1743`) refuses unconditionally —
"Int ÷ Int has no pinned semantics: truncation is never implicit (§3.2) … divide in the binary64 lane
or use Currency ÷ integer." A `real`-declared field reads back as `Value::Real` regardless of extensive
or intensive kind (`tick.rs::bind_field_value`, `:325-341`: every non-`BslType::Enum` field returns
`Ok(Value::Real(stored))` — the graph substrate stores every non-enum attribute as raw `f64`). Two
`Value::Real` operands miss every specific `apply_arith` match arm and fall to the final `_ =>` branch
(`:1718-1723`), routing to `arith_real` (`:1750-1776`) — ordinary binary64 division with only a
zero-divisor and non-finite-result guard, no `Int ÷ Int`-style refusal.

**Conclusion for `pool-ratio`:** declare `institution/rent-pool` and the pool divisor as `real`
(field / defconst), so both operands of `(/ rent-pool initial-rent-pool)` are `Value::Real` at
evaluation time; the division then executes in `arith_real` and never reaches `arith_int`'s
`Int ÷ Int` refusal. This falls out automatically once both operands are `real`-typed — no special
handling needed in the rule.

### (e) — D200: repeated `set` of one field within one tick

**SETTLED — accepted, never refused; the last write in batch-application order wins for `set`;
`add`/`sub` do not collide — they accumulate against the then-current stored value.**

The collect-then-apply write model decides this, documented and coded in `structural_verbs.rs`:

1. **Collection never deduplicates.** `PendingWrite`'s doc (`:112-152`) names the collected batch
   explicitly as "the free monoid on writes: list concatenation, associative, the empty batch its
   unit, order and multiplicity both meaningful data" (`:140-141`, lowercase in source — the dossier
   does not upper-case a source quote). Nothing rejects two `PendingWrite`s targeting the same
   `(node, field)` pair.
2. **Application is sequential; each `set` overwrites unconditionally.**
   `apply_pending_write`'s node arm (`:1032-1074`): for `UpdateOp::Set`, `new_value = write.operand`
   (already reduced against pre-state at collect time, per `:115-116`) goes straight to the graph —
   `graph.update_node(*id, &write.field, new_value)` (`:1064-1066`) — no read-and-compare, no
   collision detection. **The ordering authority is `tick.rs`, not `apply_pending_write` itself** —
   its own doc (`structural_verbs.rs:1015-1018`) names "subject order outer, source order inner" but
   says in the same breath that "this method performs exactly one write and does not itself impose
   an order over a batch." The order comes from `tick.rs`'s collect pass, which builds `all_pending`
   by iterating subjects in order and extending per subject (`:691` the `Vec`'s declaration,
   `:853` `all_pending.extend(pending)` inside that per-subject loop), and from the apply pass, a
   bare `for write in &all_pending { applier.apply_pending_write(write, graph)?; }` (`:615-616`).
   For two `set`s on the same field, the last one in that fixed batch order is the value left
   standing; every earlier `set` is silently clobbered by construction.
3. **`add`/`sub`/`scale` read the current, already-mutated-this-batch stored value at apply time**
   (`:1039-1043`: `let current = graph.node_attribute(*id, &write.field)…`, then combined at
   `:1044-1049`). Two `add`s to the same field in one batch do not collide the way two `set`s do —
   each reads the running total left by the previous write and adds to it. Stated as design intent at
   `:117-120`: the accumulating ops defer to apply-time reads specifically so that "three subjects
   each adding to one carrier" do not "lose two of the three contributions."
4. **The doc states the algebra formally** (`:141-152`): application is a monoid action on graph state —
   the batch acts as endomorphisms composed left-to-right, and `Add`/`Scale` do not commute; reordering
   a batch changes the result. For `Set`, each write is the constant endomorphism `x ↦ operand`;
   composing two constant endomorphisms left-to-right yields the second (later) one — "last write
   wins," formally.

**Answers to the brief's three sub-questions:** (1) accepted, no refusal path exists; (2) the last
write applied wins, i.e. the one latest in the "subject order outer, source order inner" batch order
`tick.rs` imposes (`:691,853,615-616`, not `apply_pending_write` itself) — for a single rule's own
`for-each`, iteration order over the query's elements, since each iteration contributes one
`PendingWrite` in source order; (3) yes, `add`/`sub` differ — they compose against the running value
rather than overwrite, because they read `current` at apply time rather than carrying a pre-reduced
final value.

**Bearing on `r03`.** `r03-tribute`'s `(update-node self social-class/wealth (set cut))` inside a
`for-each` over TRIBUTE edges hits exactly this shape when a comprador has more than one TRIBUTE edge:
each iteration emits an identical `PendingWrite{Set, cut}` (identical because `cut` is a rule-scoped
binding computed once from pre-state, independent of `it`), so "last write wins" resolves to the same
value every time — the repeated-`set` shape is safe by construction. This matches the frozen Python's
own overwrite semantics (`economic.py:385`, `source.wealth = cut_amount`, unconditional, no
accumulation across TRIBUTE edges) exactly.

### (f) `:optional :default <literal>` on a `real extensive` field; the default literal's form

**SETTLED — `:optional :default 0` (bare `Int` literal) on a `real extensive` field is legal and
already landed; the `:default -1` sentinel on `wages-paid`/`value-produced` continues to typecheck
and run correctly once a scenario redeclares those fields `real extensive`, because nothing in the
load or typecheck path compares a binding's default-literal type against its field's declared type,
and the runtime evaluator promotes Int↔Real transparently wherever the two meet.**

The landed shape, confirmed at the byte with zero line drift from the brief: `consciousness.bsl:271` —
`(binding wealth :field social-class/wealth :optional :default 0)`, a bare `Int` literal `0`
defaulting a field declared `real extensive`. `consciousness.bsl:184-185, 251-252, 264-265, 344-345` —
eight occurrences of `(binding wages :field social-class/wages-paid :optional :default -1)` /
`(binding value :field social-class/value-produced :optional :default -1)`, exactly where the brief
cited them.

No load-time or typecheck-time comparison exists between a `:default` literal's type and its field's
declared type, confirmed by three independent absences:

1. `bindings.rs`'s parser checks only structural shape — literal-vs-expression (`E-PARSE-033`,
   `:198`) and optional-requires-default (`E-PARSE-031`, `:216-217`) — never the literal's numeric
   kind against the bound field's declared type.
2. `score_class.rs::field_class` (`:99-103`) — the function `E-TYPE-016`/`E-TYPE-017` rely on —
   derives a `BindSource::Field` binding's class purely from `decl.ty`; it never inspects
   `decl.default`. The module's own header states its scope explicitly (`:19-21`, quoted verbatim):
   "The full bottom-up typechecker is Phase-2 work; this classifier answers exactly the two
   questions the chapters ask and no more" — the module doc's opening lines (`:4-9`) name those two
   questions as D46 (`E-TYPE-016`) and D67 (`E-TYPE-017`).
3. `tick.rs::bind_subject`'s runtime binding resolution (`:285-298`): on presence, the value is
   `bind_field_value`'d from the stored `f64` (`:286`, always `Value::Real` for a non-enum field); on
   absence, `atom_to_value(default)` (`:291`) converts the literal directly — for `Atom::Int(-1)` this
   yields `Value::Int(-1)` (`tick.rs::atom_to_value`, `:393-395`), not `Value::Real`. A single
   binding's two branches can produce different `Value` variants this way, but that is harmless
   because every
   downstream consumer promotes Int↔Real transparently: `evaluator.rs::apply_arith` (`:1674-1725`) and
   `::apply_ordering` (`:1902-1929`) both match `(Int, Int)`/`(Currency, Currency)` as narrow special
   cases first, then fall through to a shared branch that promotes either operand's `Int` to `f64`
   via `real_lane` (`:1662-1672`) before comparing or computing.

**The `:default` governance surface that does exist stays non-fatal, and names itself as such — the
deciding evidence for that half of the question.** `default_lint.rs`'s `lint_defaults` checks every
`:default` declaration against a Director allowlist, and `rule_pipeline.rs:318` wires it straight
into the load path: `let default_findings = lint_defaults(ctx.rule_file, &bindings);`. It
returns findings, not load errors, and `consciousness.bsl:70-72` says so in the pack's own words:
"The `:default` declarations are deliberate and content-visible; default_lint.rs's allowlist
mechanism governs the migration-corpus sites and does not gate this pack's load." That pair — the
wiring site and the pack's own citation of it — is what proves a content pack's `:default` loads
clean regardless of the allowlist's own contents.

**Conclusion:** redeclaring `wages-paid`/`value-produced` as `real extensive` (per B5, D191) does not
break the `:default -1` sentinel anywhere in the pipeline — no check exists that could catch a
mismatch, and the one place that would notice (arithmetic/comparison) already treats Int and Real as
interchangeable via promotion.

### (g) — D198: bare `(field-of it institution/rent-carrier)` as a `select-max` score

**SETTLED — legal, landed, and execution-proven, not merely typechecked.**

**Precedent 1** — `query_lane_e2e.rs:252-253` (zero drift from the plan's citation):

```
(select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY)
            (field-of it territory/priority))
```

A bare `field-of` score, no `if`-wrapping, inside a landed `.bsl` fragment under test.

**Precedent 2** — `r9_chapters.rs:1129-1130` and `:1179-1180` (zero drift):

```
(select-max (nodes NodeType/ORGANIZATION)
            (field-of it organization/claim-strength))
```

This appears twice: once in a pure typecheck probe
(`an_intensive_score_accepts_because_ordering_is_not_aggregation`, `:1124-1135`, asserting
`score_error(...) == None`) and once in a runtime **execution** test
(`select_max_and_select_min_execute_over_nodes_and_neighbors`, `:1166-1186`) that seeds two nodes,
evaluates the expression, and asserts `result == Value::NodeRef(high)` — proven end-to-end, not
merely loaded. `organization/claim-strength` is explicitly noted (`:1120-1122`) as declared
**intensive**, and the test exists specifically to prove an intensive-kinded field is legal as a score
("ordering is not aggregation") — directly relevant to `institution/rent-carrier`'s own kind
(`int extensive` per §3.1; this precedent shows kind is a non-issue for scores either way, per
`typecheck.rs:258`: "Kind is unconstrained on the score, deliberately").

`E-TYPE-016`'s exact refusal condition (`typecheck.rs:487-498`, `check_one_selection`) refuses only if
`!class.is_comparable_scalar()`; the allowed list, verbatim from the error message (`:494-496`): "Int,
Currency, Probability, Intensity, Coefficient or Real." A bare `int`/`real` `field-of` classifies as
`ScoreClass::Scalar` (`score_class.rs::field_class`, `:99-103`), on the allowed list. Refused classes
are bool, enum, str, references, and sets — none of which `institution/rent-carrier` is.

**Conclusion:** the D198 discriminator idiom —
`(select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))` — is legal by the
same rule two different landed test suites already exercise for real, at both the typecheck and the
execution level. The `territory.bsl:133-136` `if`-chain fallback stays available but is not needed —
two landed test suites already prove the bare-score form, not merely hope for it.

**Summary table**

| Q | Verdict | Deciding citation(s) |
|---|---|---|
| (a) | SETTLED, split | negative `Int`: LANDED (three `dispossession-*-conformance.bscn` files); negative fractional: refused everywhere, `reader.rs:829-835,888-890,939-943` |
| (b) | SETTLED, N ≥ 1 | `structural_verbs.rs:407-422` (execute), `:820-833` (collect), `:511-528` (for-each dispatch) |
| (c) | SETTLED, yes | `typecheck.rs:304-308` (D102 discharge statement), `:460-499` (E-TYPE-016 scope), `:294-339` (E-EVAL-042 scope); precedent `territory.bsl:130-142` and eight `(= role SocialRole/…)` sites |
| (d) | SETTLED, no Int÷Int hazard | `types.rs:262-286`, `score_class.rs:99-103`, `tick.rs:325-341`, `evaluator.rs:1674-1743` |
| (e) | SETTLED, accepted/last-wins/accumulate | `structural_verbs.rs:112-152`, `:1027-1074`; ordering imposed by `tick.rs:691,853,615-616`, not by `apply_pending_write` itself |
| (f) | SETTLED, legal | `consciousness.bsl:271,184-185,251-252,264-265,344-345,70-72`; `score_class.rs:19-21,99-103`; `tick.rs:285-298,325-341,393-395`; `evaluator.rs:1662-1672,1902-1929`; `default_lint.rs`/`rule_pipeline.rs:318` |
| (g) | SETTLED, legal and execution-proven | `query_lane_e2e.rs:252-253`; `r9_chapters.rs:1124-1135,1166-1186`; `typecheck.rs:258,487-498` |

No question stays UNSETTLED. No compiler experiment ran.

---

## Step 5 — Register/ADR tails and the D181–D201 allocation

**Verdict: the D-register tail did not move; the ADR tail did — read this as NEXT-FREE-AT-LANDING,
not as an absolute reservation.** At this dossier's original `b5a3268a` baseline, D180 and ADR213
were both the register's last rows and the plan's D181–D201 / ADR214 allocation was available. This
fix round merged `origin/dev` (`45c0b64d`, PR #665) into the baseline, and PR #665 already claimed
ADR214 — see the re-measured ADR tail below. D181–D201 stays clean: `#665` touched only
`ai/decisions/index.yaml`, not `bsl-language.rst`.

`docs/reference/bsl-language.rst`'s draft-ruling register (the `.. list-table::` starting at line
4881) runs contiguously D150 through D180 with no gaps — every `* - D1NN` row increments by exactly 1.
D180 (`bsl-language.rst:8158-8202`) is the last row; the section immediately after it reads `See Also`
(`:8204`), and `grep -n "D181\|D18[2-9]\|D19[0-9]\|D20[0-9]"` returns zero hits anywhere in the file.
D180's content matches the plan's own citation (the `PROHIBITED_INTRINSIC_NAMES` name-level gate,
`declarations.rs:128`, plus the 2026-08-17 addendum on `control-ratio/c04-terminal`). This train's 21
rows, D181–D201, remain unclaimed and contiguous with the tail — **re-measure again at the PR
boundary that actually files them**, since this register moves upstream independently of this train.

`ai/decisions/index.yaml`'s last entry, re-measured after the `origin/dev` merge:

```yaml
  ADR214_national_incidence_artifact_train:
    title: 'The #334 national-incidence artifact train (ADR171 Phase 0, ADR208 R29/C-12 flip) — ...'
    status: accepted
    date: '2026-08-17'
    file: ADR214_national_incidence_artifact_train.yaml
```

`ls ai/decisions/ | grep -oE 'ADR[0-9]+' | sort -u` now tails at ADR214, not ADR213 — PR #665 claimed
it after this dossier's original `b5a3268a` scout pass. **This train's ADR number is ADR215 as of
2026-08-18, NEXT-FREE-AT-LANDING rather than a reservation: whichever task actually files it must
re-run this same `ls`/`grep` at filing time**, since `ai/decisions/` is a shared, upstream-moving
registry this train does not own.

---

## Step 6 — Registration symbol spelling (the hyphen)

**Verdict: CONFIRMED.** Hyphen is a legal character in the rule-id `symbol` production, and
`control-ratio/…` is a landed precedent for a hyphenated first segment. `imperial-rent` is legal.

Deciding grammar, `rust/crates/babylon-bsl/src/reader.rs:522-537`:

```rust
/// Validate one `symbol` production: `LOWER ( LOWER | DIGIT | "-" )*`,
/// max 64 characters (`E-LEX-010`).
fn validate_symbol(s: &str) -> Result<(), SymbolIssue> {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(SymbolIssue::Invalid),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(SymbolIssue::Invalid);
    }
    if s.len() > 64 {
        return Err(SymbolIssue::TooLong);
    }
    Ok(())
}
```

The doc comment states the production directly: `LOWER ( LOWER | DIGIT | "-" )*`. The grammar accepts
a hyphen anywhere after the first lowercase-only character, in both the namespace segment and the
rest of a rule-id or qname — it carries no per-segment special case.

Landed precedent, `grep -n "^(rule " rust/crates/babylon-tick/content/rules/control-ratio.bsl`:

```
253:(rule control-ratio/c01-prisoner-census
276:(rule control-ratio/c02-publish-census
293:(rule control-ratio/c03-crisis
340:(rule control-ratio/c04-terminal
```

`control-ratio` is a hyphenated namespace segment, landed and loading successfully (ADR212).
`imperial-rent` is the same shape.

---

## Step 7 — The two cross-pack seams' landed shapes, transcribed verbatim

### 7(a) `decomposition.bsl` — the D194 seam

**D168's prescription** (`decomposition.bsl:104-113`, exact match, no drift):

```
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
```

**`p02-superwage-warning` in full** (`decomposition.bsl:247-271`, exact match, no drift):

```
(rule decomposition/p02-superwage-warning
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
```

Three-key emit payload confirmed: `receiver`, `desired-wages`, `available-pool`. Two latch writes
confirmed, both on the constant-score carrier expression
`(select-max (nodes NodeType/INSTITUTION) 1)` — not the discriminator score. The `(= crisis-known 0)`
conjunct sits on the whole rule's `when`, not a nested per-effect guard, so that one guard covers the
entire rule (emit plus both latch writes) together — first-writer-wins by construction.

**`p03-trigger`'s `delay-elapsed-fire` binding** (rule header at `:273`, the binding itself at
`:293-294`; the brief cited `:274-294` for the rule and matched exactly on the binding):

```
(rule decomposition/p03-trigger
  ...
  :fuel 177
  (bindings
    (binding decomposition-complete :field institution/decomposition-complete)
    (binding superwage-crisis-known :field institution/superwage-crisis-known)
    (binding superwage-crisis-tick :field institution/superwage-crisis-tick)
    (binding tick :tick)
    (binding decomposition-delay :const carceral/decomposition-delay)
    ...
    (binding delay-elapsed-fire :expr (and (= superwage-crisis-known 1)
                                            (>= tick (+ superwage-crisis-tick decomposition-delay))))
```

**D171 item 1's `payer_id` reasoning** (`decomposition.bsl:114-116`, within the numbered divergence
list running `:114-123`; the brief's `:114-123` bracket matches):

```
;   4. (global D171 item 1) THE PAYLOAD FLATTENING — `SUPERWAGE_CRISIS`'s frozen payload carries
;      `payer_id` (a second NodeRef, always `CORE_BOURGEOISIE_ID`) and
;      `narrative_hint` (a string) that this port DROPS (item 5 below);
```

**Latch field deffield/seed lines**, `decomposition-conformance.bscn` — exact match, zero drift:

```
152:  (deffield institution/superwage-crisis-known int extensive)
153:  (deffield institution/superwage-crisis-tick int extensive)
...
271:    (institution/superwage-crisis-known 0)
272:    (institution/superwage-crisis-tick 0)
```

### 7(b) `fundamental-theorem.bsl` in full (12 lines, verified — the file is exactly 12 lines)

```
; W_c > V_c: while core wages exceed the value core labour produces, the
; difference is imperial rent and revolution in the core is materially
; foreclosed. The first rule the Rust engine ever ran.
(rule economics/fundamental-theorem
  :material-basis "core wages above the value core labour produces is imperial rent; while the gap holds, revolution in the core is materially foreclosed"
  :fuel 64
  (bindings
    (binding wages :field social-class/wages)
    (binding value-produced :field social-class/value-produced))
  (when (> wages value-produced))
  (effects
    (update-node self social-class/imperial-rent (set (- wages value-produced)))))
```

### `consciousness.bsl` — the six named rules

**`p0-position`** (`:179-197`):

```
(rule consciousness/p0-position
  :material-basis "A-001 as the class-seeding law (Director flag 1): a class with material anchors (wages-paid + value-produced present) and no ternary record is positioned at the ruled unorganized rest state (0, 1, 0) — liberal hegemonic default, spec 034 A-001, THE one home (the seven scattered frozen sites are named in docs/concepts/consciousness-taxonomy.rst, not re-homed here). Data-absent classes are never positioned: UNPOSITIONED (L-ABS) — the row-19 disease's death certificate. Positioning does NOT record dominance: dominant-worldview's only writer is the read-path task's dominant rule (one-home law, pack D-record 3) — a freshly-positioned class reads it ABSENT until then. The agitation accumulator initializes to zero so later routing rules read a positioned class's agitation as present."
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

**`p4-wage-balance`** (`:247-258`):

```
(rule consciousness/p4-wage-balance
  :material-basis "The per-class wage-value balance (contradiction.py:67-100, called (v_produced, w_paid) at ideology.py:241-244, so balance = (w−v)/(v+w)): positive = wages dominate = the imperial bribe. ..."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding balance :expr (if (> (+ wages value) 0)
                               (/ (- wages value) (+ value wages))
                               (- 0 0c))))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/wage-balance (set balance))))
```

**`p5-agitation`** (`:260-292`, bindings block elided to the two pair-binding lines plus the wage-flow
arithmetic the plan names):

```
(rule consciousness/p5-agitation
  :material-basis "compute_agitation_delta (consciousness_routing.py:48-200) + ... The wage flow rides the pushed wages-inbox accumulator ..."
  :fuel 224
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    ...
    (binding wages-in :field social-class/wages-inbox :optional :default 0)
    (binding prev-wages :field social-class/previous-wages :optional :default 0)
    ...
    (binding wage-change :expr (- wages-in prev-wages))
    (binding exploit-delta :expr (if (< wage-change 0) (- 0 wage-change) 0))
    ...)
  (when (and (>= wages 0) (>= value 0) (> (+ r (+ l f)) 0)))
  (effects
    (update-node self social-class/agitation (set new-agitation))))
```

**`p7-persist-baselines`** (`:340-351`):

```
(rule consciousness/p7-persist-baselines
  :material-basis "The persistent previous-values re-homed to node fields (digest gap 4 — context.persistent_data has no BSL analog): next tick's deltas read this tick's declared flow (frozen: persistent[PREVIOUS_WAGES_KEY] = current_wages / PREVIOUS_WEALTH_KEY, ideology.py:441-442). Anchored classes only."
  :fuel 64
  (bindings
    (binding wages :field social-class/wages-paid :optional :default -1)
    (binding value :field social-class/value-produced :optional :default -1)
    (binding wages-in :field social-class/wages-inbox :optional :default 0)
    (binding wealth :field social-class/wealth :optional :default 0))
  (when (and (>= wages 0) (>= value 0)))
  (effects
    (update-node self social-class/previous-wages (set wages-in))
    (update-node self social-class/previous-wealth (set wealth))))
```

**`p2-wages-push`** (`:224-233`, the existing reader of `wages/value-flow`):

```
(rule consciousness/p2-wages-push
  :material-basis "The wage flow, un-narrowed (D151's narrowing 3 discharged, Train B item 3, #591): every WAGES edge's seeded wages/value-flow is pushed into the receiving class's wages-inbox ..."
  :fuel 128
  (bindings
    (binding active :field social-class/active))
  (when (= active 1))
  (effects
    (for-each (neighbors self EdgeType/WAGES :out NodeType/SOCIAL_CLASS)
      (update-node it social-class/wages-inbox
        (add (field-of (edge-between EdgeType/WAGES self it) wages/value-flow))))))
```

**`p6-route`** (`:294-338`) — the four named bindings, verbatim:

```
    (binding chauvinist :expr (* (if (> balance 0) balance 0) chauv-scale))          ; :312
    ...
    (binding eff-sol :expr (if (> eff-arg 0) (if (< eff-arg 1) eff-arg (- 1 0c)) (- 0 0c)))  ; :316
    (binding delta-r :expr (* (* consumed eff-sol) routing-scale))                    ; :317
    (binding delta-f :expr (* (* (* consumed (- 1 eff-sol)) routing-scale) (- 1 suppression)))  ; :318
```

(The plan cited `:312, 315-316, 317, 318`; measured here as `:312, :316, :317, :318` exactly — the
plan's `eff-sol` range collapses to the single binding line `:316`; `eff-arg`, an intermediate rather
than one of the four named bindings, sits at `:315`.)

**Verification of the `p0`/`p4`/`p5`/`p7`-not-`p6`/`p8` attribution — CONFIRMED, the plan is correct.**
The four rules binding both `social-class/wages-paid` and `social-class/value-produced` directly as
`:field` bindings are exactly `p0-position` (`:184-185`), `p4-wage-balance` (`:251-252`),
`p5-agitation` (`:264-265`), `p7-persist-baselines` (`:344-345`) — confirmed by direct read of every
rule's `bindings` block above.

`p6-route`'s only relevant binding is `(binding balance :field social-class/wage-balance
:optional :default 0)` at `:303` — the derived field, not the raw pair. Confirmed by reading the full
`p6-route` bindings block (`:297-332`): no `wages-paid` or `value-produced` binding appears anywhere
in it.

`p8-dominant-worldview` (`:353-370`) binds `active`, `revolutionary`, `liberal`, `fascist`, and a set
of expression bindings — neither `wages-paid`, `value-produced`, nor `wage-balance` appears in its
bindings block. Confirmed by direct read; zero occurrences.

---

## Step 8 — Corrected namespace list, evidence

Reformatted summary of `grep -n "^(rule " rust/crates/babylon-tick/content/rules/*.bsl` — 37 rule
ids across all 13 files, `file:line` stripped and grouped one row per file (not raw grep output;
content verified complete and correct against the real grep):

```
consciousness.bsl:     consciousness/p0-position, p1-inbox-reset, p2-org-solidarity-push,
                        p2-wages-push, p3-class-solidarity-push, p4-wage-balance,
                        p5-agitation, p6-route, p7-persist-baselines, p8-dominant-worldview
control-ratio.bsl:      control-ratio/c01-prisoner-census, c02-publish-census, c03-crisis,
                        c04-terminal
decomposition.bsl:      decomposition/p01-la-census, p02-superwage-warning, p03-trigger,
                        p04-enforcer-intake, p05-ip-intake, p06-la-deactivate
dispossession.bsl:      dispossession/territory-transfer
fundamental-theorem.bsl: economics/fundamental-theorem
lifecycle.bsl:           lifecycle/dpd-circuit
metabolism.bsl:          metabolism/biocapacity-update
organization.bsl:        organization/kind-probe
production.bsl:          production/p0-production-total-reset, p1-direct-production,
                        p2-employed-routing, p3-employed-fallback, p4-extraction-intensity
solidarity.bsl:          solidarity/p0-transmit
territory.bsl:           territory/p1-heat-dynamics, p2-eviction-pipeline, p3-spillover,
                        p4-camp-decay, p4-penal-suppression
vitality.bsl:            vitality/subsistence-and-death
worldview.bsl:           consciousness/worldview-mint-probe
```

**Thirteen files, twelve distinct namespaces — confirmed, exact match to the plan's claim.** Sorted:
`consciousness, control-ratio, decomposition, dispossession, economics, lifecycle, metabolism,
organization, production, solidarity, territory, vitality`.

`fundamental-theorem.bsl`'s only rule is `economics/fundamental-theorem` — confirmed;
`fundamental-theorem/` is not a namespace. `worldview.bsl`'s only rule is
`consciousness/worldview-mint-probe` — confirmed; `worldview/` is not a namespace, and the file shares
`consciousness` with `consciousness.bsl`.

`imperial-rent/…` sorts after `consciousness, control-ratio, decomposition, dispossession,
economics` and before `lifecycle, metabolism, organization, production, solidarity, territory,
vitality` — `economics` < `imperial-rent` < `lifecycle` (`e` < `i` < `l`), confirmed.

---

## Concerns for the parent task

1. **This dossier is evidence, not the pack.** It settles every question the brief poses and corrects
   the plan where citations drifted, but Task 1 still owes the spike work Step 4(e) names before
   `r03` gets written (BLOCKER note in the plan §4: "the accepted answer" needs proof "against the
   real driver," which this dossier's read of the source code establishes but does not itself
   execute).
2. **One background-agent artifact needed discarding.** A sibling research fork misreported that two
   other forks "failed to launch" with a nested-fork error; both of those forks in fact completed
   successfully, and this dossier incorporates their output above. This dossier does not repeat that
   stray claim, and a reader should not treat it as a live concern — this note exists only so a
   reader who saw it in the task log knows this dossier already resolved it.
3. **`p5-agitation`'s full bindings/expression block was not transcribed verbatim end to end** — Step
   7 quotes its two pair-bindings and its wage-change/exploit-delta arithmetic (the parts the plan's
   §2.2 row 4 depends on) but elides the constant bindings and the routing arithmetic between them for
   length. A task that needs the rule's full text should read `consciousness.bsl:260-292` directly
   rather than rely on this dossier's excerpt.
