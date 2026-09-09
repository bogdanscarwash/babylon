# BSL Surface-Ergonomics Train B — Design Charter (items 6, 3, 4)

**Status: RULED 2026-08-15** — Director approved the charter wholesale ("approved"); all six §8 decision points adopted as recommended (additive `real`; full-inventory migration; sequencing 6→3→4; identical-declaration recognition with `DuplicateType` preserved for differences; `wages-received` retired on un-narrowing; §1's AE(ii) posture signed off).

Authority: issue #591 items 6/3/4 (Director directive 2026-08-15) + the Director's goal text
("Goal B — the deep machinery … it wants its own charter and it'll move some of this train's
pins by declared ceremony"). Evidence: `ai/scratch/2026-08-15-bsl-ergonomics-b-archaeology.md`
(the digest — every file:line citation below is verified there). Companion: ADR207 (the W10
handoff whose narrowings this train discharges).

## 1. Constitutional posture (AE ii)

All three items are **surface ergonomics minting no new mathematics**:

- **Item 6 (`real`)** gives an honest declared home to what the store already *does*. The store
  is kind-blind f64 end to end (`structural_verbs.rs:1563-1603`); verbatim-f64-in-int is ruled
  law (W10 controller amendment 1; consciousness.bsl D-record 5; ADR207 §3a). `BslType` is DSL
  storage typing — load-time metadata, **never hashed** (`scenario.rs:1044-1054`) — not a
  constitutional primitive. Representation-level, the same shape as AG(i)'s attributed
  membership: a payload on existing elements, not a new element kind.
- **Item 3 (edge-attribute seeding)** extends the loader to what the store already *holds*:
  runtime edge attributes exist (section 0x05, `substrate.rs:168-175`; the `update-edge` write
  law, T3/ADR198 R1–R3; the declarable-but-unseedable precedent `edge-write-lane-e2e.bscn:27`).
- **Item 4 (scenario-declaration sharing)** is loader composition. Zero hash movement; the
  ordinal law (declaration order = stored ordinal) is unchanged.

Standing guardrails: no imposed functional forms; the frozen Python engine stays reference-only;
any divergence earns a D-row. The formalism surface is not re-opened — these are loader, type,
and content-migration mechanics.

## 2. Item 6 — the `real` numeric type

**Facts.** `Value::Real(f64)` exists at runtime (`evaluator.rs:55-62`) with no declared-field
home. ~20 write sites across 6 committed packs hold non-integral f64 in `int`-typed fields
(digest table: production `wealth` 11.538…, vitality `wealth` 999.95, lifecycle `pop-p`
6962.099…, metabolism `biocapacity` 1.9000…, dispossession `dispossession-intensity` 0.358…,
consciousness `agitation` 0.135, …). Seeds are the one strict door (`attribute_value_int`,
`scenario.rs:1144-1163`: fractional seeds into int fields are loader-refused).

**The fork:**

- **(a) Additive `BslType::Real` (recommended).** Compiler-driven arms: `types.rs:214`,
  `parse_type_name` (`declarations.rs:650-679`), `load_deffield` (`scenario.rs:919-931`),
  `attribute_value` (`scenario.rs:1062-1086` — new arm accepting int + scaled literals without
  the [0,1] check), `store_range_check` joins int's no-check lane
  (`structural_verbs.rs:1699-1703`), typecheck fold/arith (`typecheck.rs:166-207`),
  `score_class.rs:87`. `numeric_write_value` needs nothing (:1588 already verbatim). The
  literal lane rides the existing `r` suffix (E-LEX-023 r-cap, `reader.rs:923-929`); real
  fields gain seed acceptance. **Byte-neutral: type tags are never hashed, so adding the type
  AND re-typing every committed field moves zero byte pins.**
- **(b) Tighten `int` to integer-only — evidence-REJECTED.** One integrality clause in
  `store_range_check` and every row of the digest's inventory table dies at the store boundary:
  ~20 write sites across 6 packs, multiple goldens move. A content massacre, not a type fix.

**Migration scope fork:** re-type the full digest inventory int→real (recommended — byte-neutral,
retires the awkwardness in one mechanical pass) vs. only W10's three consciousness fields.

**Done:** `real` landed; the inventory migrated; the W10 narrowing D-rows amended ("int-lane
typing retired by `real`"); every existing golden byte-identical (proof obligation, not hope).

## 3. Item 3 — edge-attribute seeding + the WAGES un-narrowing

**Facts.** `load_edge` is strength-only (exact 5-slot pattern, `scenario.rs:1296-1302`; duplicate
guard E-LOAD-044). Edges already carry attributes at runtime (`substrate.rs:168-175`; the
`/strength` double-storage fork D143). Edge deffields are declarable but unseedable —
`solidarity/tension` is committed and seeds nothing (`edge-write-lane-e2e.bscn:27,34-35`).

**Design.** Extend the scenario surface to seed edge attributes. Mechanism (an extended edge
form vs. a sibling `(edge-attr …)` form) is a plan-level decision; the binding constraint is to
reuse `attribute_value`'s per-type literal law (including the Currency refusal) and to preserve
the duplicate-guard posture.

**The un-narrowing (D151's discharge).** The W10 narrowing — frozen's incoming-WAGES
`value_flow` fold-sum (`ideology.py:299-309`) cut to one class-side `wages-received` field —
was recorded as "exact for single-employer content" precisely because seeding was unserved.
With seeding landed: `wages/value-flow` returns as an edge deffield declared `real` (per §5
sequencing); the three WAGES edges return with seeded values; `p5-agitation`'s wage-change read
becomes the frozen fold-sum over incoming WAGES edges (`consciousness.bsl:243,255` re-point);
`p7-persist-baselines` re-points (`:320,324`); the scenario seeds move from class-side fields
onto edge forms (`consciousness-ternary-conformance.bscn:210,237,250,264`).

**Ceremony.** By the recorded single-employer exactness, post-tick values are byte-identical;
the **pre-tick** hash moves (new edges + moved fields) — one re-pin at `tick_goldens.rs:333`,
with post-tick (:340) expected unmoved. If post-tick moves: STOP and report. Conformance rows
re-measured (crate goldens are measurements-not-ceremonies by precedent,
`tick_goldens.rs:172-177`). D151 gains its "narrowing retired" note; the train ADR records the
ceremony.

## 4. Item 4 — scenario-declaration sharing

**Facts.** One `(scenario …)` form per source (`scenario.rs:312-318`). The WorldView defenum is
minted at `worldview-foundation.bscn:34` and re-declared verbatim at
`consciousness-ternary-conformance.bscn:190`. Declaration order IS the stored ordinal
(`types.rs:35-46`), which is why the re-declaration carries a parity test
(`tick_goldens.rs:361-372` — its own header states it exists only because of the
re-declaration).

**Design constraint (binding).** Sharing = include/merge semantics that recognize an
**identical** declaration as the same one. `EnumRegistry`'s `DuplicateType` guard
(`types.rs:131-135`) is preserved for *differing* declarations — no silent relaxation.
Mechanism (an `(include …)` form vs. multi-source composition at the entry points,
`lib.rs:72-148`) is a plan-level decision.

**Done:** the conformance scenario consumes the mint's WorldView declaration; the parity test
dies (declared test death); the mint's own ordinal test (`tick_goldens.rs:285-296`) survives as
the single ordinal home. Zero pins move (defenum declarations are unhashed).

## 5. Sequencing: 6 → 3 → 4

Digest-recommended, matching #591's own order. Item 6 first means item 3's edge seeds never
answer "can an edge attribute hold an unbounded f64?" with the int-lane hack — `wages/value-flow`
declares `real` on arrival. Item 3's un-narrowing is the train's only re-pin. Item 4 is purely
loader-compositional, independent of the type system, and its test-killing lands cleanest once
content settles. (3↔4 are disjoint — `load_edge` vs. the top loop — and could swap without
conflict.)

## 6. Ceremony and gates plan

- **Byte-neutrality proofs:** items 6 and 4 move zero pins — proved by the hash-covering goldens
  and every exact-f64 conformance suite staying byte-identical at each landing.
- **Item 3's single pre-tick re-pin:** declared ceremony, recorded in the train ADR. No
  `tests/baselines/**` movement anywhere in this train — III.13/§6.5 is the Python estate and
  nothing here crosses into it.
- **Per-task gates:** full `rust:check`; `qa:regression` as Python-side hygiene; Copilot harvest
  and `mise run pr:merge` per PR; Conventional Commits + the `Co-Authored-By` trailer.
- **Process:** subagent-driven development per the established pattern — fresh implementer per
  task, per-task reviews, scoped re-reviews on fix rounds, final whole-branch review, ledger-first
  in `.superpowers/sdd/`.

## 7. Non-goals

#591's other half — item 1 (push-over-pull as a named canonical pattern in `bsl-language.rst`),
item 2 (`min`/`max`/`abs`/`clamp` intrinsics), item 5 (scientific-notation literals) — verified
not yet landed (`intrinsic_host.rs:62` registers only `floor`; no push-over-pull section in the
language reference); queued in #591, not this train. AG(i) payloads ride #536. No Python engine
changes. No client work.

## 8. Director decision points

1. **Posture** — sign off §1's AE(ii) framing (all three items: representation-level, no new
   mathematics).
2. **Item 6 fork** — additive `real` (recommended; byte-neutral) over int-tightening
   (evidence-rejected: ~20-site content massacre).
3. **Migration scope** — the full digest inventory re-typed to `real` (recommended) vs. only
   W10's agitation/wage-balance/solidarity-inbox.
4. **Sequencing** — 6 → 3 → 4 (recommended).
5. **Item 4 constraint** — identical-declaration recognition with `DuplicateType` preserved for
   differences (mechanism delegated to the plan).
6. **`wages-received` disposition** — retire the field on un-narrowing (recommended: the fold-sum
   is exact and now honest) vs. keep it as a derived convenience readout.
