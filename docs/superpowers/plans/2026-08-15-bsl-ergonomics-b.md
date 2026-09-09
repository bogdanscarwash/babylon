# BSL Surface-Ergonomics Train B (items 6, 3, 4) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give unbounded f64 an honest declared home (`real`), make edge attributes seedable, and let scenarios share declarations — discharging W10's recorded narrowings (D151, the verbatim-f64 int lane) with zero unintended byte drift.

**Architecture:** Three additive loader/type-surface extensions to `babylon-bsl` plus one content retrofit in `babylon-tick` content. Everything is representation-level: `BslType` is load-time metadata (never hashed), edge attributes already exist at runtime, and declaration sharing is loader composition. The only byte movement is Task 4's declared re-pin ceremony.

**Tech Stack:** Rust workspace (`rust/crates/babylon-bsl`, `babylon-graph`, `babylon-tick`), BSL content (`.bsl` rules + `.bscn` scenarios), the dual-implementation Python oracle (`consciousness_ternary_conformance.py`), cargo + mise gates.

**Spec:** `docs/superpowers/specs/2026-08-15-bsl-ergonomics-b-charter.md` — Director-ruled 2026-08-15 (all six decision points adopted as recommended: additive `real`; full-inventory migration; sequencing 6→3→4; identical-declaration recognition with `DuplicateType` preserved for differences; `wages-received` retired on un-narrowing; AE(ii) posture signed off). Archaeology digest: `ai/scratch/2026-08-15-bsl-ergonomics-b-archaeology.md` (every file:line below is verified there or re-verified by the controller against dev @17971664).

## Global Constraints

- **AE(ii):** no new mathematics, no new formal constructs. Type kinds, grammar forms, and loader composition only.
- **Byte-neutrality:** Tasks 1, 2, 3, 5 move ZERO byte pins (type tags and declarations are never hashed — `scenario.rs:1044-1054`, `tick_goldens.rs:239-245`). Task 4 is the train's ONE declared re-pin ceremony (see its ceremony step).
- **No `tests/baselines/**` movement anywhere** — III.13/§6.5 is the Python estate; nothing here crosses.
- **Frozen Python engine untouched** (reference-only); any divergence earns a D-row.
- **Conventional Commits** + `Co-Authored-By: Kimi Code <noreply@moonshot.ai>` trailer; `mise run commit` with a staged tree.
- **`cargo test -p babylon-bsl` / `-p babylon-tick` green after every step**; full `mise run rust:check` before every commit that touches `rust/`.
- Merges only via `mise run pr:merge`; Copilot review harvested per PR (zero unaddressed comments).
- Every new law gets a register row (D-number) in `docs/reference/bsl-language.rst` in the task that mints it; the implementer picks the next free D-number (D155+ as of writing — verify with `rg -n '^D1[0-9]+' docs/reference/bsl-language.rst | tail -5`... actually `rg -o 'D1[0-9][0-9]' docs/reference/bsl-language.rst | sort -u | tail`).
- Error codes: new codes take the next free number in their family (`rg -o 'E-LOAD-0[0-9]+' rust/crates/babylon-bsl/src docs/reference/bsl-language.rst | sort -u | tail`), with the spec-code registry entry.
- **Worktree:** execution happens in `/home/user/projects/game/bsl-ergonomics-b` (branch `feature/bsl-ergonomics-b` from dev); created at execution time with the `data/` symlink farm + `.env` copied and `mise run install` if any Python leg is run.

---

### Task 1: `BslType::Real` — the type, the seed arm, the compiler-driven arms

**Files:**
- Modify: `rust/crates/babylon-bsl/src/types.rs:212-237` (the `BslType` enum)
- Modify: `rust/crates/babylon-bsl/src/scenario.rs:919-931` (`load_deffield`'s type-name match), `:1055-1087` (`attribute_value`), after `:1163` (new `attribute_value_real`)
- Modify: `rust/crates/babylon-bsl/src/declarations.rs:650-679` (`parse_type_name`)
- Modify: `rust/crates/babylon-bsl/src/typecheck.rs:166-207` (fold/arith typing — compiler-flagged arms)
- Modify: `rust/crates/babylon-bsl/src/score_class.rs:~87` (compiler-flagged arm)
- Modify: `docs/reference/bsl-language.rst` (the new register row + §1.5/§2.9 prose)

**Interfaces:**
- Produces: `BslType::Real` (unit variant); `"real"` accepted by both type-name parsers; `attribute_value_real(atom, local, field) -> Result<f64, ScenarioError>`; the store's no-check lane now formally includes `Real` (no edit — `store_range_check`'s `matches!` at `structural_verbs.rs:1699-1703` already excludes it). Task 2 relies on `"real"` being a lawful deffield type; Task 3 relies on `attribute_value` handling `Real` for edge-attribute values.

- [ ] **Step 1: The grep audit (before any edit — the catch-all hunt)**

`BslType` matches with a wildcard arm compile silently past a new variant. Enumerate every match site and adjudicate it in the task report BEFORE writing code:

```bash
rg -n 'BslType::' rust/crates/babylon-bsl/src --glob '!*tests*' | grep -v '^.*//'
rg -n 'other =>|_ =>' rust/crates/babylon-bsl/src/scenario.rs rust/crates/babylon-bsl/src/typecheck.rs rust/crates/babylon-bsl/src/score_class.rs
```

Known catch-all (MUST gain an explicit arm, compiler will NOT flag it): `attribute_value`'s `other =>` at `scenario.rs:1079-1085` — its refusal text ("stores only `int`, `probability`, `intensity`, `coefficient` or `enum`-declared …") is updated to name `real` too. Known no-edit site: `store_range_check` (`structural_verbs.rs:1699-1703`) — its `matches!` lists only the three unit-interval types; `Real` falls through to `Ok(())`, which IS the intended lane. Verify no other catch-all swallows `BslType`.

- [ ] **Step 2: Write the failing tests**

In `scenario.rs`'s `#[cfg(test)] mod tests` (the module at :1387; mirror its existing deffield/seed test idiom):

```rust
#[test]
fn real_deffield_seeds_int_scaled_and_ratio_verbatim() {
    // (deffield social-class/balance real intensive), node seeds:
    //   int 9            -> 9.0
    //   0.25c            -> 0.25
    //   1.5r             -> 1.5
    // assert stored attribute == expected f64, bit-exact
}

#[test]
fn real_deffield_refuses_currency_and_bare_ident() {
    // 9$ -> the currency refusal message (same text family as the int arm's)
    // bare ident -> loud error naming node + field
}

#[test]
fn real_field_store_has_no_range_check() {
    // engine-side (structural_verbs.rs tests): writing 6962.099999999999 and
    // -0.05263157894736842 into a real-declared field succeeds verbatim —
    // E-EVAL-020 never fires for Real (negative seeds can't be written by
    // literals, but WRITES may be negative; the store does not care).
}
```

Also a typecheck test (in `typecheck.rs`'s tests) that a fold-sum over a `real extensive` field types as the fold's numeric lane and does not raise E-TYPE-041.

- [ ] **Step 3: Run to verify they fail**

Run: `cargo test -p babylon-bsl real_ 2>&1 | tail -20`
Expected: FAIL — `unknown type `real`` from `load_deffield`'s match.

- [ ] **Step 4: Implement**

`types.rs` (:222-224 area, between `Currency` and `Int` is fine — declaration order here is not content-facing):

```rust
    /// An unbounded finite binary64 scalar — the honest declared home for
    /// what the store already does (verbatim f64, `numeric_write_value`).
    /// Carries no range law: seeds accept int / p/i/c / r literals (each
    /// already lex-bounded in its own lane), writes store any finite f64.
    /// minted by Train B item 6 (ADR-pending), AE(ii) representation-level.
    Real,
```

`scenario.rs` `load_deffield` (:919-931): add `"real" => BslType::Real,` and extend the refusal text's list to `int / real / probability / intensity / coefficient / currency / enum`.

`scenario.rs` `attribute_value` (:1062-1086): add ABOVE the catch-all:

```rust
        BslType::Real => attribute_value_real(atom, local, field),
```

and update the catch-all text (:1079-1085) to include `real` in the listed types.

New function after `attribute_value_int` (:1163):

```rust
/// `real`-declared fields (Train B item 6). Accepts the three literal lanes
/// whose own lex laws already bound them — int (exact to 2^53, the same
/// guard `attribute_value_int` states), p/i/c (`[0,1]` at lex), r (`(0,∞)`
/// at lex) — each converted by the crate's one scaled-literal contract
/// (`unscaled / 10^scale`). Currency is refused (the same deferral every
/// other arm states). There is NO arbitrary-precision fractional literal:
/// E-LEX-021 still refuses bare floats; that is #591 item 5's territory,
/// not this train's.
fn attribute_value_real(atom: &Atom, local: &str, field: &str) -> Result<f64, ScenarioError> {
    match atom {
        Atom::Int(value) => {
            if value.unsigned_abs() > (1_u64 << 53) {
                return Err(err(format!(
                    "node `{local}` field `{field}`: {value} exceeds f64's exact integer range"
                )));
            }
            #[allow(clippy::cast_precision_loss)]
            Ok(*value as f64)
        }
        Atom::Scaled(scaled) => {
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            Ok(numerator / 10_f64.powi(i32::from(scaled.scale)))
        }
        Atom::Currency(_) => Err(err(currency_refusal_message(local, field))),
        other => Err(err(format!(
            "node `{local}` field `{field}`: expected an int, p/i/c or r literal for a \
             real field, found {other:?}"
        ))),
    }
}
```

(Verify `Atom::Scaled`'s `ScaledKind` covers the `r` lane — `classify_ratio` at `reader.rs:917+` returns `Atom::Scaled(ScaledLit { kind: ScaledKind::Ratio, .. })` per the p/i/c shape at :903-907; if Ratio is a separate `Atom` variant, add its arm with the same conversion. The reader source is the authority, not this plan.)

`declarations.rs` `parse_type_name` (:650-679): add the `"real"` arm mirroring `"int"`.

`typecheck.rs` (:166-207) + `score_class.rs` (:~87): add the compiler-flagged arms, giving `Real` the same typing as the numeric f64 lane (fold-sum over `real` = the numeric lane, never E-TYPE-041; a `real` field is not a unit-interval type anywhere). If any arm is genuinely unreachable, say so in the report rather than inventing behavior.

`docs/reference/bsl-language.rst`: new register row (next free D-number) recording the real lane's seed/write law + the "no arbitrary fractional literal yet" note; §1.5's literal table gains the `real`-field acceptance line; §2.9's type list gains `real`.

- [ ] **Step 5: Green + the byte-neutrality proof**

Run: `cargo test -p babylon-bsl 2>&1 | grep -E 'test result|FAILED' | tail -5` then `mise run rust:check 2>&1 | tail -5`.
Expected: all suites pass; then `git diff --stat` shows NO `.bscn`/golden-hash change anywhere (Task 1 touches no content) and `rg -c 'assert_eq!' rust/crates/babylon-tick/tests/tick_goldens.rs` unchanged — every existing golden byte-identical by construction (nothing committed declares `real` yet).

- [ ] **Step 6: Commit**

```bash
git add rust/crates/babylon-bsl/src docs/reference/bsl-language.rst
mise run commit -- "feat(bsl): mint BslType::Real — the honest home for verbatim f64 (#591 item 6)"
```

---

### Task 2: The content migration — every non-integral-f64 int field re-typed `real`

**Files:**
- Modify: `rust/crates/babylon-tick/content/scenarios/production-conformance.bscn:113-114,118`
- Modify: `rust/crates/babylon-tick/content/scenarios/vitality-conformance.bscn:24`
- Modify: `rust/crates/babylon-tick/content/scenarios/lifecycle-conformance.bscn:38-41,44,48,51`
- Modify: `rust/crates/babylon-tick/content/scenarios/metabolism-conformance.bscn:25-26`
- Modify: `rust/crates/babylon-tick/content/scenarios/dispossession-conformance.bscn:22-23`
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn:195,207-210,212-213` + its comment at :83-92
- Modify: `rust/crates/babylon-tick/content/rules/consciousness.bsl:106-122` (the rounding D-row text)

**Interfaces:**
- Consumes: Task 1's `"real"` deffield type.
- Produces: the migrated field roster (below). Task 4 relies on `wages-received` being `real`-typed history — it REMOVES that field outright; every other field migrated here stays.

- [ ] **Step 1: Re-type the roster (the digest's verified inventory)**

In each `.bscn`, change ONLY the type token `int` → `real` on these deffield lines (kind token unchanged):

| File | Fields |
|---|---|
| production-conformance.bscn | `social-class/wealth` (:113), `social-class/production-value` (:114), `territory/production-total` (:118) |
| vitality-conformance.bscn | `social-class/wealth` (:24) |
| lifecycle-conformance.bscn | `territory/pop-d`, `pop-p`, `pop-d-prime`, `wealth-d-prime` (:38-41), `dependency-ratio` (:44), `legitimation-index` (:48), `transmitted-ideology` (:51) |
| metabolism-conformance.bscn | `territory/biocapacity`, `max-biocapacity` (:25-26) |
| dispossession-conformance.bscn | `territory/dispossession-intensity` (:22), `territory/wealth` (:23) |
| consciousness-ternary-conformance.bscn | `social-class/wealth` (:195), `agitation` (:207), `wage-balance` (:208), `solidarity-inbox` (:209), `wages-received` (:210), `previous-wages` (:212), `previous-wealth` (:213) |

Do NOT touch: `social-class/population` / `active` / `wages-paid` / `value-produced` (integral content, stay `int` — the fractional-seed-refusal pin at `consciousness-ternary-conformance.bscn:100-105` depends on an int field staying int), `territory/rent-level-x1e6` (deliberately x1e6-scaled int, `territory.bsl:188`'s explicit `floor`), two-classes' `imperial-rent` (int arithmetic on int seeds), `territory/heat` (already `intensity`), `organization/active` (0/1 latch).

In the same pass, update each migrated line's trailing comment where it says "int lane" / "verbatim-f64 int" to note the retirement — e.g. :207 becomes:

```
  (deffield social-class/agitation real intensive)         ; [0,∞) raw f64 (frozen: ge=0.0, unbounded — social_class.py:95-102). Retired from the verbatim-f64 int lane by Train B item 6 (the real type); zero seeds remain (R-MEASURED: produced, never authored)
```

- [ ] **Step 2: The D-row amendments**

`consciousness.bsl:106-122` (the rounding D-row): append one sentence — "The verbatim-f64 int lane this row records was RETIRED by Train B item 6 (`BslType::Real`): the fields it governed now declare `real`; the store law itself (verbatim f64, no implicit truncation, `floor` available) is unchanged — only the declared home moved."

`consciousness-ternary-conformance.bscn:83-92` (the mirrored note): same retirement sentence, one line.

- [ ] **Step 3: The byte-neutrality proof (this task's whole point)**

Run: `mise run rust:check 2>&1 | tail -5` — every suite green, and CRITICALLY every golden hash assertion passes UNEDITED: this task changes type tokens and comments only, and type tags are never hashed (`scenario.rs:1044-1054`). If ANY golden hash or exact-f64 conformance assertion fails, STOP — the failure means a seed literal that was int-typed now parses differently; find it before touching anything else. Then `mise run qa:regression 2>&1 | tail -3` (Python-side hygiene; untouched estate, 12/12 expected).

- [ ] **Step 4: Commit**

```bash
git add rust/crates/babylon-tick/content docs/reference/bsl-language.rst
mise run commit -- "refactor(bsl-content): re-type the non-integral-f64 roster int -> real (#591 item 6 migration)"
```

- [ ] **Step 5: PR A** — push the branch, open PR A ("Train B items 6: the real type + migration") with the two commits, wait out CI + the Copilot harvest (median ~230s; `skipping` on the Dependabot leg is a TERMINAL state, not a pending one), merge via `mise run pr:merge -- <N>`.

---

### Task 3: `(edge-attr …)` — edge-attribute seeding

**Files:**
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` — new `load_edge_attr` (modeled on `load_edge`, :1290-1385), the body-dispatch arm in `load_scenario`'s top loop, a `seeded_attrs: HashSet<(String, NodeId, NodeId, String)>` local alongside `seeded` (:336-342 area)
- Modify: `docs/reference/bsl-language.rst` (the grammar's scenario-form list + the new register row)
- Tests: `scenario.rs`'s `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: `attribute_value` (Task 1's `Real` arm included) for the value conversion; `GraphSubstrate::update_edge` (`rust/crates/babylon-graph/src/substrate.rs:168-175` — read it before writing the call).
- Produces: the `(edge-attr <EdgeType/MEMBER> <from-name> <to-name> <deffield-qname> <value>)` top-form. Task 4 consumes it for `wages/value-flow`.

- [ ] **Step 1: Write the failing tests** (scenario.rs test module, inline sources, the module's existing idiom):

```rust
#[test]
fn edge_attr_seeds_a_declared_edge_field() {
    // scenario with (deffield solidarity/tension intensity intensive),
    // two nodes, (edge EdgeType/SOLIDARITY a b 0.5c), then
    // (edge-attr EdgeType/SOLIDARITY a b solidarity/tension 0.25i)
    // assert graph.edge_attribute(SOLIDARITY-edge, "solidarity/tension") == Some(0.25)
}

#[test]
fn edge_attr_refuses_unknown_edge_undeclared_field_strength_and_currency() {
    // (edge-attr ... a c ...) where no a→c edge exists     -> loud, names endpoints
    // (edge-attr ... solidarity/nope 0.5i) undeclared      -> loud, names the field
    // (edge-attr ... solidarity/strength 0.5c)             -> loud: strength seeds
    //   via the edge form only (D32's implicit field is not in scenario.fields)
    // (edge-attr ... solidarity/tension 5$)                -> the currency refusal
}

#[test]
fn edge_attr_refuses_a_double_seed() {
    // two (edge-attr ... a b solidarity/tension ...) forms with the same
    // (member, from, to, field) key -> the new E-LOAD-0NN coded refusal
    // (mirrors E-LOAD-044's key argument: the quadruple is a KEY)
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p babylon-bsl edge_attr 2>&1 | tail -10`; expected: the reader/loader has no `edge-attr` head ("expected …" dispatch error).

- [ ] **Step 3: Implement `load_edge_attr`**

Model on `load_edge` (:1296-1385) with these deltas: six slots `(edge-attr <enum-ref> <from> <to> <field-qname> <value-atom>)`; the same `demand_enum_kind` + vocabulary checks; resolve both endpoints via `named` (same "reads top to bottom" law — the edge must already exist: look the `(member, from_id, to_id)` triple up in `seeded`, refuse loudly naming the endpoints if absent); look the field qname up in the scenario's `fields` map (absent → loud refusal naming the field — this is also what refuses `solidarity/strength`, since D32's implicit field is never in `scenario.fields`); verify the qname's prefix matches the edge type's lower-cased name (the `solidarity/tension` ↔ `SOLIDARITY` convention — find the existing prefix check used for node fields and reuse its shape; if none exists for edges, state the check explicitly in the register row); convert the value via `attribute_value(atom, local, field, decl, enums)` (the ONE per-type literal law, Currency refusal included); insert the quadruple key into `seeded_attrs` (double-seed → the new coded refusal); write via `graph.update_edge(...)`.

Add the `"edge-attr"` arm to `load_scenario`'s body dispatch and the `seeded_attrs` local. Register the new E-LOAD code (next free; spec-code registry entry).

`bsl-language.rst`: the scenario grammar's form list gains `(edge-attr …)`; new register row (next D-number) recording the form, the quadruple key, the strength refusal, and the top-to-bottom edge-existence law.

- [ ] **Step 4: Green + byte-neutrality proof** — `cargo test -p babylon-bsl` green, then `mise run rust:check`: every existing golden byte-identical (no committed scenario uses `edge-attr` yet — additive grammar). `git diff` shows zero content/golden changes.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/babylon-bsl/src docs/reference/bsl-language.rst
mise run commit -- "feat(bsl): the (edge-attr ...) scenario form — edge-attribute seeding (#591 item 3)"
```

---

### Task 4: The WAGES un-narrowing — D151's discharge + the re-pin ceremony

**Files:**
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn` (:189 defvocabulary, :204-213 deffields, :237/:250/:264 seeds, ~:267+ employer node, new edge + edge-attr forms)
- Modify: `rust/crates/babylon-tick/content/rules/consciousness.bsl` (header D151 note + inventory; `p1-inbox-reset`; new `p2-wages-push`; `p5-agitation`'s wage-change read; `p7-persist-baselines`)
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py` (the dual-implementation oracle mirrors the machinery)
- Modify: `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs` (re-measured rows)
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs:327-351` (the re-pin + its header history)

**Interfaces:**
- Consumes: Task 1's `real` type (`wages/value-flow`, `wages-inbox` declare it), Task 3's `(edge-attr …)` form.
- Produces: the un-narrowed wage flow — the frozen incoming-WAGES fold-sum (`ideology.py:299-309`) restored via the push-over-pull idiom (D136's canonical pattern).

- [ ] **Step 1: The scenario edits**

`defvocabulary EdgeType (SOLIDARITY WAGES)` (:189). Deffield block: DELETE `(deffield social-class/wages-received …)` (:210) and its three seed pairs (:237, :250, :264's `wages-received` pairs); ADD:

```
  (deffield wages/value-flow real intensive)               ; the per-edge wage flow (ADR203 edge deffield; retired-then-restored: W10's D151 narrowing discharged by Train B item 3)
  (deffield social-class/wages-inbox real intensive)       ; [0,n) flow-sum — the push-over-pull accumulator (D136): reset by p1, pushed by p2-wages-push, read by p5, persisted by p7
```

After the employer node (~:272), add the edges (strength `1.0c` — a structural presence marker with NO semantic consumer; nothing reads WAGES `strength`; stated in the register row) and the seeded flows:

```
  (edge EdgeType/WAGES employer class-exploited 1.0c)
  (edge-attr EdgeType/WAGES employer class-exploited wages/value-flow 9)
  (edge EdgeType/WAGES employer class-bribed 1.0c)
  (edge-attr EdgeType/WAGES employer class-bribed wages/value-flow 12)
  (edge EdgeType/WAGES employer class-emergent 1.0c)
  (edge-attr EdgeType/WAGES employer class-emergent wages/value-flow 9)
```

WAIT — verify against the anchors before committing to these numbers: class-emergent's wage CUT is 9→8 (previous-wages 9, received 8 — :264-265), so its value-flow is **8**, not 9; exploited 10→9 → **9**; bribed 12→12 → **12**. The flows are the CURRENT wage: 9, 12, 8 (plan `2026-08-15-class-surface-ternary-port.md:215` agrees). The implementer re-derives each flow from the node's own comment before writing — the oracle asserts these exact values post-tick.

- [ ] **Step 2: The pack edits (consciousness.bsl)**

Header: D151's row gains its retirement note ("narrowing retired by Train B item 3: the wage flow rides seeded WAGES-edge `wages/value-flow` again via push-over-pull; `wages-received` is gone"); the D116 byte-order map and the per-rule inventory table gain `p2-wages-push` (byte order: `p2-org-solidarity-push` < `p2-wages-push` — 'o' < 'w' — so wages-push fires immediately after org-push, both after p1's reset).

`p1-inbox-reset`: gains a second reset write — `social-class/wages-inbox ← 0` — same subjects, same firing (its fired count is unchanged).

New rule (its text adapts the W10 plan's original draft at `2026-08-15-class-surface-ternary-port.md:497-501`):

```
(rule consciousness/p2-wages-push
  :material-basis "The wage flow: every WAGES edge's seeded value-flow is pushed into the
   receiving class's wages-inbox (frozen ideology.py:299-309's incoming-WAGES fold-sum,
   expressed per the D136 push-over-pull idiom — D138 forbids the filter-in-fold pull)."
  ...subjects SOCIAL_CLASS, for-each (neighbors self EdgeType/WAGES :out NodeType/SOCIAL_CLASS):
       (update-node it social-class/wages-inbox
         (add (field-of (edge-between EdgeType/WAGES self it) wages/value-flow))))
```

(The pack's actual guard/shape idiom is the landed `p2-org-solidarity-push`'s — mirror ITS structure exactly, swapping the edge type, the read field, and dropping the strength gate; the text above shows the load-bearing expressions, not a re-derivation of the pack's rule syntax.)

`p5-agitation`: the wage-change binding's read becomes `wages-inbox − previous-wages` (was `wages-received − previous-wages`) — one qname swap in the binding.

`p7-persist-baselines`: persists `wages-inbox` into `previous-wages` (was `wages-received`) — one qname swap.

- [ ] **Step 3: The fired-count spike (before any re-pin)**

Write a throwaway test (or extend the conformance test's debug output) that prints the per-rule fired counts of the new pack. EXPECTED: `p2-wages-push` fires on `employer` only (+1 → total 51); p1 unchanged (11). If the pack's for-each idiom fires on edgeless subjects too, the total differs — MEASURE, record the truth, and pin the measured number with the arithmetic explained in the golden's header (the house pattern, `tick_goldens.rs:233-236`). Delete the throwaway.

- [ ] **Step 4: The oracle + conformance re-measurement**

`consciousness_ternary_conformance.py`: mirror the machinery — the scenario dict loses `wages-received`, gains the WAGES edges with their flows and `wages-inbox`; the p1/p2/p5/p7 mirrors updated to match; per-class wage flows 9/12/8 produce the SAME post-tick values as before (single-employer exactness: the fold-sum IS the one edge's flow). `consciousness_ternary_conformance.rs`: the field-name assertions re-point (`wages-inbox` where `wages-received` was asserted; the tick-2 accumulation witness's value pins are UNCHANGED — verify, don't assume). Every OTHER value assertion (ternaries, agitation, balances, dominants, tick-2 rows) must pass UNEDITED — that pass IS the ceremony's lawfulness proof.

- [ ] **Step 5: The declared re-pin ceremony**

Run the golden; it fails on the two hash assertions with the new values printed. Verify BEFORE re-pinning: the new pre-tick state differs from the old ONLY by (−3 `wages-received` attributes, +3 WAGES edges, +3 `wages/value-flow` edge attributes, +0 `wages-inbox` — unseeded) and the new post-tick state ONLY by (−`wages-received`, +`wages-inbox` on the reset/pushed classes per the fired spike). The conformance suite's value-level green (Step 4) is the proof that no VALUE drifted. Then re-pin both hashes at `tick_goldens.rs:333,340`, update the fired assertion (:344-350) to the spiked number, and extend the header's re-pin history (:310-326) with this ceremony's row: "Train B item 3 (D151 discharge): WAGES edges + `wages/value-flow` seeding restored the frozen fold-sum via push-over-pull; `wages-received` retired; BOTH hashes re-pinned (attribute-set change, zero value drift — proven by `consciousness_ternary_conformance.rs` passing with only the two field re-points); fired 50 → <measured>."

- [ ] **Step 6: Full gates + commit**

`mise run rust:check` green; `mise run qa:regression` 12/12 (Python-estate hygiene).

```bash
git add rust/crates/babylon-tick/content rust/crates/babylon-tick/tests
mise run commit -- "feat(tick): un-narrow the wage flow onto seeded WAGES edges — D151 discharged (#591 item 3 retrofit)"
```

- [ ] **Step 7: PR B** — push, PR B ("Train B item 3: edge-attribute seeding + the WAGES un-narrowing"), CI + Copilot harvest, `mise run pr:merge -- <N>`. Comment on #591 with the ceremony evidence (old/new hashes, the fired spike number, the value-level green).

---

### Task 5: Scenario-declaration sharing + the train records

**Files:**
- Modify: `rust/crates/babylon-bsl/src/types.rs:121-135` (`EnumRegistry::declare` — identical-declaration recognition)
- Modify: `rust/crates/babylon-bsl/src/scenario.rs:308-333` + new `load_scenario_with_prelude`
- Modify: `rust/crates/babylon-tick/src/lib.rs:72-76,105-148` (`run_once_with_prelude` + `prepare_rules` threading)
- Create: `rust/crates/babylon-tick/content/declarations/worldview.bscn` (the prelude — declaration forms only, header comment stating it is NOT a scenario)
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn:190` (re-declaration dies) + :204's comment
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs` (:329 the call site, :353-372 the parity test dies)
- Modify: `docs/reference/bsl-language.rst` (register row + §2.13 prose)
- Modify: `ai/decisions/ADR208_*.yaml` + `index.yaml`, `ai/state.yaml`

**Interfaces:**
- Consumes: nothing from Tasks 1-4's surfaces (independent by design, §5 of the charter).
- Produces: `load_scenario_with_prelude(prelude_src, scenario_src, graph) -> Result<LoadedScenario, ScenarioError>`; `run_once_with_prelude(scenario_src, prelude_src, rule_src) -> Result<TickReport, String>`; `EnumRegistry::declare`'s identical-recognition arm.

- [ ] **Step 1: Failing tests**

`types.rs` tests: identical re-declaration (same name, same members, same order) returns `Ok` with the SAME `EnumTypeId`; same name with reordered/renamed/extra members still returns `DuplicateType`.

`scenario.rs` tests: `load_scenario_with_prelude` with a defenum prelude + a scenario declaring an enum-typed field resolves; a prelude containing a `node`/`edge`/`edge-attr` form is refused loudly; a scenario that ALSO declares the prelude's enum identically loads (recognition); one that declares it differently refuses.

- [ ] **Step 2: Implement**

`EnumRegistry::declare` (:131-135): replace the bare `any(name ==)` refusal with — same name AND identical member list (order-sensitive compare against the stored declaration) → `Ok(EnumTypeId(existing_index))`; same name, differing members → `DuplicateType` exactly as today. (Member lists are stored per declaration — the registry already keeps them for `ordinal`; the compare is a slice equality.)

`load_scenario_with_prelude`: read the prelude's forms first — each MUST be `defenum` / `defvocabulary` / `defconst` / `deffield` (anything else → loud refusal naming the form head) — threading the SAME registries (`enums`, vocabulary, consts, fields) the subsequent scenario load uses; then run the existing single-scenario path on `scenario_src`. Refactor note: `load_scenario` currently builds those registries as locals (:336-342); the clean shape is an internal `load_scenario_inner(source, graph, registries…)` both entry points call — do the minimal extraction, no heroic refactor.

`lib.rs`: `run_once_with_prelude(scenario_src, prelude_src, rule_src)` — `prepare_rules` gains a `prelude_src: Option<&str>` parameter (the existing `run_once`/`run_once_into` pass `None`; behavior unchanged), calling `load_scenario_with_prelude` when `Some`. `TickSession::new`'s path is NOT extended (no consumer today — noted in the register row as the session-path follow-up, YAGNI until production content shares).

The prelude file `content/declarations/worldview.bscn`: header comment ("declaration prelude — consumed via `load_scenario_with_prelude`; NOT a scenario form") + `(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))`.

- [ ] **Step 3: The conformance switch + the declared test death**

`consciousness-ternary-conformance.bscn`: DELETE :190's `(defenum WorldView …)`; :204's comment becomes "consumes the WorldView declaration from `content/declarations/worldview.bscn` via the prelude (Train B item 4) — no re-declaration". The test-side constant wiring gains `WORLDVIEW_PRELUDE` (`include_str!`) and the golden's call (:329) becomes `run_once_with_prelude(CONSCIOUSNESS_TERNARY_SCENARIO, WORLDVIEW_PRELUDE, CONSCIOUSNESS_TERNARY_RULES)`. DELETE `consciousness_ternary_worldview_member_order_is_the_ruled_ordinal` (:353-372) — a declared test death, recorded here and in the register row: the prelude composition makes the re-declaration it guarded impossible; the mint's own `worldview_member_order_is_the_ruled_ordinal` (:285-296) survives as the single ordinal home.

Byte-neutrality proof: pre- AND post-tick hashes at :333/:340 UNCHANGED (defenum declarations are unhashed; the graph content is identical), fired unchanged — if either hash moves, STOP: the prelude threading touched the graph.

- [ ] **Step 4: Docs + the train records**

`bsl-language.rst`: the prelude mechanism + the identical-recognition law as a register row (next D-number); §2.13's sharing prose; D151's retirement was Task 4's — cross-reference it here.

`ai/decisions/ADR208_bsl_ergonomics_b.yaml` (+ `index.yaml` row): the train record — the three items, the Director's six charter rulings, the Task-4 ceremony (old/new hashes, fired 50→measured, the value-level proof), the declared test death, the follow-ups roster (session-path prelude threading; #591 items 1/2/5 still queued; the D151 citation-range nit folded to item 1's pass).

`ai/state.yaml`: the closing entry.

- [ ] **Step 5: Full gates + commit + PR C**

`mise run rust:check` green; `mise run qa:regression` 12/12.

```bash
git add rust docs ai
mise run commit -- "feat(bsl): scenario-declaration sharing via prelude composition + identical-recognition (#591 item 4); docs(decisions): ADR208 the train record"
```

PR C ("Train B item 4 + records"), CI + Copilot harvest, `mise run pr:merge -- <N>`; #591 commented with the discharge evidence for items 6/3/4 (the issue stays OPEN for items 1/2/5).

---

## Self-review notes (controller)

- **Spec coverage:** charter §2 (item 6) → Tasks 1-2; §3 (item 3) → Tasks 3-4; §4 (item 4) → Task 5; §5 sequencing → task order; §6 ceremonies/gates → per-task proof steps + Task 4 Step 5 + Task 5 Step 3; §7 non-goals — honored (no intrinsics, no sci-notation, no push-over-pull doc item here; the D151 nit rides item 1's future pass, recorded on #591).
- **The one charter refinement disclosed:** the charter's ceremony text expected Task 4's POST-tick hash unmoved. With ruling 6 adopted (`wages-received` retired, not kept-derived), the attribute SET changes (−`wages-received`, +`wages-inbox`), so BOTH hashes re-pin. No VALUE drifts — the exact-f64 conformance suite is the proof, and it passes with only the two field re-points (Task 4 Step 4). This is the mechanical consequence of the adopted ruling, not new drift.
- **Type consistency:** `attribute_value_real` signature mirrors `attribute_value_int`; `load_edge_attr` mirrors `load_edge`'s parameter list plus the field qname; `load_scenario_with_prelude` / `run_once_with_prelude` argument order (scenario first, prelude second) matches `run_once(scenario, rules)`'s existing lead.
