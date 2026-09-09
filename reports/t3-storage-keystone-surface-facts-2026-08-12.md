# T3 Storage-Keystone Surface Facts — Verified 2026-08-12

Scout pass for Program 29 train T3 (issue #560, the "storage keystone"): edge-attribute storage
per ADR198 R1-R3 — full symmetric `deffield` edge rows, the empty-elided fifth `CanonicalState`
section, full `update-edge` write parity. Read-only (`Read`/`rg`/`gh` only, no cargo, no writes
outside this file) against `dev` in the main checkout (`/home/user/projects/game/babylon`).

**Provenance.** `dev` at `4d1ed154` (merge of PR #573, the T2 slice-2 plan doc) throughout this
pass. **PR #574 (`feat/t2-edge-reads-substrate`, T2 PR A) is OPEN, not merged** — its 7 commits
were read via `gh pr diff 574` and every fact drawn from them is marked **[post-T2-PR-A]** below;
facts without that marker were verified against the `dev` working tree directly. T2 PR B
(`feat/t2-edge-reads-evaluation`, Tasks 3-7 of the T2 plan) does not exist yet; its planned
surfaces are summarized in §7 from the plan document only. Program 29 train issues verified live:
T1 #558 (CLOSED), T2 #559, T3 #560, T4 #561 (director-gate), T5 #562, T6 #563, plus rider #572
(`the` disposition).

**Executive summary (10 lines).** T3's static surface is ALREADY BUILT, exactly as T2's was:
`update-edge` parses (arity 3, `grammar.rs:657`), carries §3.7 cost rows (`bound_checker.rs:26-30`),
is reserved against the intrinsic namespace, and add-edge's `<field-init>` static checks
(E-PARSE-041/E-TYPE-014) are load-time-live (`grammar.rs:119,174`) — what is missing is storage,
hash, and the apply path. The REAL keystone inside the keystone: production's tick path is
`collect_effects`, which serves ONLY `update-node`/`emit`/`guard`/`for-each`
(`structural_verbs.rs:693-716`) and whose `PendingWrite` is NodeId-keyed by construction
(`structural_verbs.rs:136-140`) — R3's parity therefore means widening the collect-then-apply
machinery itself, not just retiring the two refusal arms (`:387-398`, `:709-716`) and the
add-edge field-init refusal (`:918-925`). The hash side is smaller than it sounds: NO section of
`CanonicalState` is elided today (all four written unconditionally even when empty,
`state_hash.rs:257-283`) so R2's elision is a genuinely NEW, deliberately ASYMMETRIC convention —
the exact asymmetry ADR198 R2 pre-authorizes recording — and it is what keeps the pinned digests
(`state_hash.rs:381,550`; `hypergraph_store.rs:525` [post-T2-PR-A generalizes its message]) and
all 6+1 tick goldens byte-identical at landing. Two scope tensions need surfacing, not solving:
the §6.2 chapter-C2 vector family T3 inherits includes an I.15-illegal-mode-transition vector
(E-EVAL-030) while the I.15 machine is a declared, uncharted Phase-2 gap; and `update-edge`
against `<edge-type>/strength` must write the EXISTING 0x03 slot, never a fifth-section row —
a double-storage hazard no ruling text names.

---

## 1. The ruling

### 1.1 Issue #560 (verbatim core)

`gh issue view 560`, title "P29-T3: edge-attribute storage — the D35/D65 keystone (ADR198 R1-R3)":

> THE ruled lane (ADR198, Constitution III.7 escalation discharged 2026-08-12): **full symmetric
> edge attributes** — deffield rows per edge type, same closed type vocabulary as nodes (enum
> included; Currency storage still refused); **fifth canonical section, ELIDED WHEN EMPTY** (every
> existing golden/baseline/save byte-identical at landing — the landing PR must PROVE this with
> the full golden suite untouched); **full update-node write parity** (set/add/sub/scale through
> PendingWrite collect-then-apply, D104 apply-time accumulation; enum set included; update-edge's
> one-f64 refusal retires).
>
> Its own landing ADR records the implementation; mutation-verified law tests per the standing
> sentinel rule; the elision rule documented + versioned in state_hash's canonical-layout contract
> (if the hyperedge section's convention differs, record the deliberate asymmetry). AG(i)
> membership payloads explicitly NOT here (R4 — #536 charters that ceremony). Opens Wave C with
> T2. Gates 11 systems (survey §3).

### 1.2 ADR198 — the three rulings T3 implements, in the ADR's own words

`ai/decisions/ADR198_program29_substrate_widening_charter.yaml` (status accepted, 2026-08-12):

- **R1** (`ADR198…yaml:25-34`): "**FULL SYMMETRIC EDGE ATTRIBUTES.** Edges get declared, typed
  fields exactly as nodes have them: deffield rows per edge type, the same closed type vocabulary
  (int / probability / intensity / coefficient / enum; Currency STORAGE still refused per the
  2026-08-11 first-consumer ruling), read via field-of over an EdgeRef once Slice 2 serves it. One
  general mechanism, one escalation; the named-field-grants alternative … and the edge-reification
  alternative … were both declined."
- **R2** (`:36-44`): "**NEW SECTION, ELIDED WHEN EMPTY.** CanonicalState gains a fifth section
  (edge attributes, mirroring how node attributes sit beside nodes) hashed ONLY when at least one
  edge attribute exists. Consequence: every existing golden, baseline and save stays
  byte-identical at landing — no estate-wide rebless ceremony. The elision rule is documented and
  versioned in the hash contract (babylon-graph/src/state_hash.rs's canonical-layout doc); if the
  hyperedge section's existing convention differs, the implementing train's ADR records the
  deliberate asymmetry."
- **R3** (`:46-51`): "**FULL UPDATE-NODE PARITY AT LANDING.** set / add / sub / scale through the
  SAME PendingWrite collect-then-apply machinery (apply-time accumulation, D104), enum set
  included. The set-only alternative was declined: frozen systems (EdgeTransition, Solidarity) do
  read-modify-write on edge attributes, so set-only would force a second ceremony almost
  immediately."
- **R4 boundary** (`:53-58`): AG(i) membership payloads "does NOT bundle into this storage
  ceremony. Community's own train (#536) charters it."
- Consequences (`:90-104`): "T2+T3 unblock Wave C (Solidarity, Sovereignty, EdgeTransition, the
  storage class)… T3 is the one train that widens CanonicalState; its implementation lands with
  mutation-verified law tests and the empty-elision proof (existing goldens byte-identical). The
  formalism surface stays CLOSED (AE ii) … R1-R3 widen storable STATE using existing constructs."
- R8 (`:81-89`): "Checkpoint A's gates are now NAMED: T2, T3, T4, T6."

### 1.3 The Program 29 spec's T3 row and its T5 interaction

`docs/superpowers/specs/2026-08-12-program-29-substrate-widening-design.md:46`:

> | T3 | Edge-attribute storage per R1–R3 — deffield edge rows, the empty-elided fifth section,
> `update-edge` verb parity; mutation-verified law tests; the empty-elision proof (existing
> goldens byte-identical) | engineering (its own landing ADR) | the 11-system storage class
> (write half) |

and `:45` gives T2 the "read half of the storage class"; `:58`: "**Wave C (after T2+T3):**
Solidarity, Sovereignty, EdgeTransition, and the storage-class". The T5 row (`:48`) carves the
boundary explicitly: "Structural-verb execution surface — §2.8 `add-node`/`remove-node`/
`add-edge`/`remove-edge` **(+ `update-edge` beyond strength, folded into T3)** at tick time" —
i.e., `update-edge` in full belongs to T3, the six SHAPE verbs at tick time belong to T5.

### 1.4 The 11 gated systems, named

`reports/port-estate-survey-2026-08-12.md:123`:

> | **D35/D65 — `GraphSubstrate` edge-attribute STORAGE** | **11**: ImperialRent, Sovereignty,
> Contradiction, ContradictionField, FieldDerivative, FascistFaction, EdgeTransition, Electoral
> (C2), Doctrine, ReserveArmy, Struggle | **Unscheduled on all four slices.** A Constitution III.7
> hash-widening decision needing its own ADR | `substrate.rs:80-248` has *no* edge-attribute
> accessor; `add_edge`'s own `:strength` has no reader. Refusal text at
> `structural_verbs.rs:387-398` |

(The survey's "no edge-attribute accessor" claim is now stale for the READ half once PR #574
merges — see §6 — but the WRITE half stands verbatim on `dev` today.)

### 1.5 No issue-text/ADR conflict found

Issue #560's "full update-node write parity" phrase reads oddly in isolation but matches ADR198
R3's own heading ("WRITE SURFACE: FULL UPDATE-NODE PARITY AT LANDING") — both mean `update-edge`
gains parity WITH `update-node`'s four ops. Every other clause of the issue maps 1:1 onto R1/R2/R3/R4.
The genuine open tensions are code-vs-charter, not issue-vs-ADR — collected in §11.

## 2. Current storage reality (`rust/crates/babylon-graph`)

### 2.1 Both backends store edges identically; strength is the ONLY edge datum

- `MemoryGraph::edges: HashMap<(String, NodeId, NodeId), f64>` — "`(edge_type, from, to)` ->
  strength" (`memory.rs:45-47`). No other edge-resident storage exists anywhere in the crate.
- `HypergraphStore::edges: HashMap<(String, NodeId, NodeId), f64>` — "Dyadic half — identical in
  shape to `MemoryGraph`'s" (`hypergraph_store.rs:74-78`). **The dyadic half never touches
  hypergraph-rs at all** — "the dyadic half (`nodes`, `attributes`, `edges`) is native Rust maps"
  (`hypergraph_store.rs:4-8`); `add_edge`/`remove_edge` on the store write only `self.edges`
  (`hypergraph_store.rs:194-226`). Consequence: T3's edge-attribute storage requires ZERO
  hypergraph-rs library changes — both backends widen with a plain new map/field.
- `add_edge` carries the mandatory `:strength` (`substrate.rs:105-117`); duplicate add and absent
  remove are loud (`memory.rs:147-177`, `hypergraph_store.rs:194-226`); node attributes have the
  read point `node_attribute` with honest-null discipline (`substrate.rs:135-142`,
  `memory.rs:189-203`) — the exact division T3's edge writes must mirror.
- Attribute values are `f64` — "the binary64 lane only; typed attribute storage (Currency's i128
  exactness) is a declared Phase-2 gap" (`substrate.rs:25-28`), matching R1's "Currency STORAGE
  still refused."
- Node attributes are keyed by FULL QNAME strings in practice (production writes
  `"social-class/wages"` etc. — `hypergraph_store.rs:512-514`'s own baseline fixture), the
  convention the edge-attribute store will inherit.

### 2.2 The state hash today — four sections, none elided, layout normative

`state_hash.rs`:

- Section tags: `TAG_NODES 0x01`, `TAG_ATTRIBUTES 0x02`, `TAG_EDGES 0x03`, `TAG_HYPEREDGES 0x04`
  (`state_hash.rs:58-61`). The canonical byte layout is declared **normative, a cross-language
  contract** in the module doc (`state_hash.rs:10-27`): big-endian integers, u32-length-prefixed
  UTF-8 strings, per-section `tag ‖ u32 count ‖ entries`.
- **Edges (0x03)**: per edge, ascending `(type, from, to)`: `str type ‖ u64 from ‖ u64 to ‖ u64
  strength-bits` (`state_hash.rs:22-23`, implementation `:154-171`). Strength IS already hashed —
  the fifth section holds only what does not exist yet.
- **NO EMPTY-ELISION EXISTS ANYWHERE TODAY.** `encode_state` writes all four sections
  unconditionally (`state_hash.rs:257-283`); the test helper `encoder_with_one_attribute`
  explicitly writes empty `write_edges(&[])`/`write_hyperedges(&[])` sections
  (`state_hash.rs:454-463`); the golden byte vector includes every section (`:490-518`). So R2's
  "elided when empty" is a NEW convention, asymmetric with 0x04's (a graph with zero hyperedges
  still writes `0x04 ‖ 0x00000000` today) — the deliberate asymmetry ADR198 R2's own text
  pre-authorizes the landing ADR to record. **Verified: the convention DOES differ; the asymmetry
  record is owed.**
- Float discipline the fifth section must inherit: `-0.0` canonicalized to `+0.0`, NaN/non-finite
  refused loudly (`state_hash.rs:44-53,106-121`; tests `:573-599`).
- Sort discipline: "the sort contract belongs to the encoder, never to the store's internal
  order" — `encode_state` sorts nodes by id, attributes by `(id, name)`, edges by
  `(type, from, to)`, hyperedges by id + member lists ascending (`state_hash.rs:244-283`).
  The fifth section needs its own ruled sort key (the natural mirror of 0x02's `(id, name)` is
  `(type, from, to, qname)` — a design choice, not a settled fact).
- `StateEncoder::push_count` writes `tag + u32 count` (`:92-99`); `as_bytes()` exists specifically
  for byte-level differentials (`:197-203`).
- "**Versioned**" (R2's word) has no existing anchor: the canonical-layout doc carries NO version
  marker today (`state_hash.rs:10-27` — read in full). T3 must mint the versioning convention, not
  merely bump one.

### 2.3 The one-encoder design (what "symmetric" means mechanically)

`CanonicalState` (`state_hash.rs:231-294`) is a four-listing trait (`all_nodes`/`all_attributes`/
`all_edges`/`all_hyperedges`) with ONE provided `encode_state`/`state_hash` — "a second store
cannot move the bytes by encoding differently, because it does not encode" (`:228-230`). T3 widens
this trait with a fifth listing (e.g. `all_edge_attributes`) consumed by the ONE `encode_state`.
Three implementors exist: `MemoryGraph` (`memory.rs:77-107`), `HypergraphStore`
(`hypergraph_store.rs:427-486`), and the test-only `Facts` fixture (`state_hash.rs:305-325`).
Design hazard to weigh explicitly: a REQUIRED fifth method forces all three to update (loud,
good); a DEFAULT-empty method would let a store silently forget to report edge attributes — the
exact "reporting different facts" failure the one-encoder design exists to surface.

### 2.4 Pinned digests T3 must NOT move (the empty-elision proof's teeth)

1. The golden byte vector, field-annotated (`state_hash.rs:479-527`) — its own assertion message:
   "the canonical encoding moved — if this was deliberate it is a CONTRACT CHANGE… not a test to
   re-bless casually". **Elision-when-empty leaves this byte-identical (no edge attributes in the
   fixture); an unconditional fifth section would break it.**
2. The pinned digest over that vector (`state_hash.rs:533-554`,
   `5e0041a4…fa1c0d8`) and the provided-method twin (`:331-384`, same hex).
3. `adding_a_read_only_query_method_does_not_move_the_state_hash`
   (`hypergraph_store.rs:507-529`): the III.7 baseline fixture pinned at hex
   `9577d95124a7c4ed6faad2c4aca5980b435fb73e7b58813413500a5fdef798ed`. **[post-T2-PR-A]** PR #574
   generalizes its doc/message to "a read-only GraphSubstrate method addition (node_type_of or
   edge_attribute) must be III.7-clean" — same fixture, same hex (PR #574 diff, hypergraph_store.rs
   hunk at `:533-560` of the patched file). T3's fixture has edges but zero edge attributes, so
   the elided section keeps this pin intact too.
4. The six tick goldens (`rust/crates/babylon-tick/tests/tick_goldens.rs:48,65,89,128,164,211` —
   two-classes, vitality, us-counties-lifecycle, organization-foundation, territory-conformance,
   production-conformance) plus `babylon-client/tests/engine_link.rs`'s pin — the "6+1" set PR
   #574's own gate ran byte-identical (PR #574 commit messages, patches 2/3). Issue #560: "the
   landing PR must PROVE this with the full golden suite untouched."

### 2.5 Iteration-order / determinism conventions already ruled

- `edges()` returns ascending `(source, target)` on both backends (`memory.rs:220-229`,
  `hypergraph_store.rs:270-279`), pinned by conformance (`conformance.rs:243-252,446-465`).
- Substrate trait doc: "Iteration order is part of the CONTRACT… never graph-internal storage
  order" (`substrate.rs:151-156`). A new edge-attribute read is a keyed point lookup (no exposed
  iteration); the fifth-section listing may return storage order because `encode_state` sorts
  (§2.3) — the same split `all_hyperedges` already documents (`hypergraph_store.rs:449-453`).

## 3. How parity is currently PROVED (the harness patterns T3 extends)

### 3.1 The shared conformance suite

`conformance.rs::run_substrate_conformance` is `pub` (not `#[cfg(test)]`) and takes a store
factory (`conformance.rs:23-46`); rows are plain functions registered in the dispatch list
(`:33-45`). It runs against BOTH backends: `memory_graph_passes_the_conformance_suite`
(`conformance.rs:517-526`) and `hypergraph_store_passes_the_conformance_suite`
(`hypergraph_store.rs:496-499`). Edge-related rows on `dev` today:

| Row | What it pins | Cite |
|---|---|---|
| `removal_cascades_edges_memberships_and_attributes` | ADR185 R2 cascade takes incident edges + attributes | `conformance.rs:54-115` |
| `duplicate_add_and_absent_remove_are_loud_on_both_halves` | §2.8 edge existence discipline (E-EVAL-031's substrate half) | `:120-156` |
| `nodes_edges_neighbors_hold_contractual_order_and_dedup` | `edges()` ascending (source,target) | `:230-280` |
| `state_hash_is_stable_and_order_invariant_and_sensitive` | an edge removal moves the hash | `:368-413` |
| `declared_order_never_leaks_through_any_ranged_accessor` | edges declared against id order still sort | `:446-483` |

**[post-T2-PR-A]** PR #574 adds four `edge_attribute` rows to the dispatch list:
`edge_attribute_reads_back_the_seeded_strength`, `edge_attribute_on_a_missing_edge_is_loud_not_zero`,
`edge_attribute_of_an_unstored_qname_is_loud` ("T2 stores `<edge-type>/strength` only — a
different edge-owned qname is 'never written'"), and
`edge_attribute_does_not_check_the_owner_segment` (a PINNED NEGATIVE: the owner segment is
deliberately NOT checked — "so a future reader does not 'fix' the suffix check into an owner check
by surprise"). The third row's own doc names T3 directly: "T3 (ADR198 R1) widens this" — T3 will
REWRITE that row's expectation (a stored non-strength qname becomes readable), which is a
conformance-row supersession the landing ADR should record, not a silent edit.

### 3.2 The differential harness

`tests/differential.rs`: a `Twin` drives every mutating op against `MemoryGraph` and
`HypergraphStore` in lockstep, asserting `encode_state().as_bytes()` equality **after every single
operation** (`differential.rs:1-49`); the script covers `-0.0`, decade boundaries, mixed types,
cascades and direct hyperedge removal (`:108-169`). T3's edge-attribute write op joins this script
(a new `Twin` method mirroring `update_node`'s, `:59-63`), or the two stores' fifth-section facts
are never differentially compared.

### 3.3 The covenants gate

`tests/covenants.rs` — source-level + behavioral checks specific to `HypergraphStore`:
- Covenant 6 enumerates **the 7 mutating methods** by name and requires `check_not_frozen()?` in
  each method head (`covenants.rs:167-196`; the frozen pre-check itself
  `hypergraph_store.rs:100-117`). **Any new mutating substrate method T3 adds (an edge-attribute
  write point) grows this list to 8 and must both call the pre-check and be added to the test's
  array — the test will NOT catch the omission by itself** (it iterates a hard-coded list).
- Covenant 5 pins exact f64 round-trip of strength through `all_edges` (`covenants.rs:137-165`) —
  the precedent shape for a fifth-section round-trip check.
- Covenant 1 pins the library's ingest surface — irrelevant to T3 since the dyadic half never
  touches the library (§2.1).

### 3.4 The law-test standing rule

Issue #560: "mutation-verified law tests per the standing sentinel rule." Precedent in-crate: PR
#574's own commit messages record commenting-out-the-wiring mutation evidence per test (patches
3/5); `conformance.rs`'s rows are the substrate-law vehicle. The memory-file standing rule
("sentinel every error CLASS, mutation-validated") applies to each new refusal class T3 mints.

## 4. The write surface today — what R3 actually retires, site by site

### 4.1 The refusal sites (all verbatim on `dev`)

1. **Execute path** — `structural_verbs.rs:387-398`: `verb @ ("update-edge" | "update-hyperedge")
   => Err(…)`: "has no substrate storage: GraphSubstrate keys an edge to one f64 strength and
   gives a hyperedge no attributes at all. Widening that state widens the canonical state_hash
   field set, which is a declared Phase-2/substrate decision (Constitution III.7), never a
   silently-dropped write". **The arm is SHARED between `update-edge` and `update-hyperedge` —
   T3 retires only the `update-edge` half; the arm must be SPLIT**, because hyperedge own-field
   storage (D65's runtime half) is chartered NOWHERE in ADR198 (R1 is edges; R4 defers AG(i)
   payloads; hyperedge own fields are neither).
2. **Collect path (production's)** — `structural_verbs.rs:709-716`: identical refusal text, in
   `collect_item`'s match.
3. **add-edge `<field-init>` tail** — `structural_verbs.rs:918-925`: "an add-edge <field-init>
   has no substrate storage… a declared substrate gap (R9 chapter C2), never silently dropped".
   The grammar's static checks for that tail are already live (E-PARSE-041 on a `strength` init:
   `grammar.rs:30,119,174`, tested `grammar.rs:1119`; E-TYPE-014 on a foreign owner:
   `grammar.rs:533`) — only the execution refusal retires.
4. **NOT retired (stays):** `add-hyperedge`'s field-init refusal (`structural_verbs.rs:994-1002`)
   and `update-hyperedge`/`update-membership` (`evaluator.rs:468-491` reserves them; AG-era).

### 4.2 The production path is collect-then-apply, and it is node-only by type

- `run_tick`'s production path uses `collect_effects` (`structural_verbs.rs:611-641`,
  `tick.rs:595-622` collects `Vec<PendingWrite>` into `all_pending`), NOT `execute_effects`
  ("retired from production (Task 12) and stays only as a test/corpus harness",
  `structural_verbs.rs:405-410,699-707`).
- `PendingWrite { id: NodeId, field: String, op: UpdateOp, value … }`
  (`structural_verbs.rs:136-145`) — its own doc: "**Only `update-node` defers via this type**"
  (`:109`). R3's "through the SAME PendingWrite collect-then-apply machinery" therefore requires
  either widening `PendingWrite`'s target to an edge-or-node key or minting a parallel
  pending-edge-write joined into the same ordered batch — **a batch-ordering decision**, because
  the batch's algebra is documented and load-bearing: "the collected batch is the free monoid on
  writes… application is a monoid action… `Add`/`Scale` do not commute… it may NOT reorder the
  APPLICATION phase" (`structural_verbs.rs:119-134`). Interleaved node and edge writes must apply
  in collection order under that same law.
- The load-time gate `check_no_deferred_shape_verbs` (`structural_verbs.rs:1388-1405`, called at
  `rule_pipeline.rs:269`) refuses only the SIX shape verbs (`DEFERRED_SHAPE_VERBS`,
  `structural_verbs.rs:1352-1359`) — **`update-edge` is NOT in it, so an `update-edge` rule LOADS
  CLEAN today and dies at its first admitted tick** in `collect_item`'s arm (`:709-716`). T3 makes
  that runtime death a real write; no load-gate change is needed for `update-edge` itself.

### 4.3 The node-side write machinery R3 mirrors (parity means THIS list)

All in `structural_verbs.rs`, all reachable from both execute and collect paths:

- `update_node` (`:508`) / `collect_update_node` (`:731-739`) — parse, resolve referent, §2.10
  discipline-1 type check, reduce operand pre-state; apply deferred.
- `apply_pending_write` (`:799`) — apply-time read-modify-write for `add`/`sub`/`scale` (D104's
  apply-time accumulation; the test at `:3153` proves it guards enum arithmetic independently).
- `numeric_write_value` (`:1196-1235`) — the ONE funnel every written value crosses: non-finite
  refused (`EvalCode::NonFinite`), `Int` widened deterministically, **Currency refused loudly**
  (`:1225-1230` — "the Director ruled (2026-08-11) that this lands with Currency's first real
  consumer — refusing the lossy f64 cast"; this is R1's "Currency storage still refused" already
  implemented, reusable as-is).
- `enum_write_value` (`:1245-1281`) — E-EVAL-042 (`EvalCode::EnumWriteShapeViolation`): enum
  fields written ONLY as `<EnumType>/<MEMBER>`, stored as declared-order ordinal in the binary64
  lane (ADR195/D101). R3's "enum set included" reuses this function unchanged.
- `refuse_arithmetic_on_enum_field` (`:1304-…`) — `add`/`sub`/`scale` on an enum field is
  E-EVAL-042 at all three sites. **Its own doc already names T3**: "and, if a storage-bearing
  `update-edge` is ever built, would reuse this exact combine shape too" (`:1302-1303`).
- `store_range_check` (`:1321`) — §3.3's E-EVAL-020 range law (unit-interval types only, never a
  clamp).
- The write-log: `Write::EdgeAdded/EdgeRemoved` records exist (`:940-945,970-974`); an
  edge-attribute write record (the `update-node` analogue) presumably joins `write_log.rs`'s
  `Write` enum — not verified which variants that enum has today (see UNVERIFIED).

### 4.4 The static surface already landed (T2-pattern: only execution is missing)

- Grammar: `("update-edge", 3, 3, "exactly 3")` in ARITIES (`grammar.rs:657`); update-op shape
  checking shared with update-node/update-hyperedge (`grammar.rs:819`).
- Cost: `update-edge` is in `bound_checker.rs`'s "nine typed structural verbs plus `emit`" table
  (`bound_checker.rs:26-30`); module doc: "The grammar, the §3.7 cost rows, the §2.8 static
  checks and the error codes land now; the storage is a declared substrate gap"
  (`structural_verbs.rs:23-26`).
- Reservation: `"update-edge"` in `declarations.rs`'s reserved form tags (`declarations.rs:78`).
- Refusal-message tests exist that will FLIP at T3 (the T2-plan "serving a previously-refused head
  flips every landed refusal assertion" lesson, T2 plan `:1691-1699`): at minimum
  `evaluator.rs:2263-2302` pins `update-edge`'s effect-position storage refusal (message contains
  "Constitution III.7") — T3 owes the same enumerate-and-flip sweep T2's plan did for `edges`.

## 5. What the spec already promises (the contract T3 implements, not designs)

All cites `docs/reference/bsl-language.rst`:

- **Grammar** (`:1330-1351`): `(update-edge <expr> <qname> <update-op>)`; add-edge's
  `<field-init>*` tail (`:1334-1335`); the four update-ops closed (`:1343-1346,1353-1356`).
- **D36 — `update-edge` takes an `EdgeRef`, not a triple** (`:1382-1393`): "It mirrors
  `update-node` operand for operand… Rules that hold the endpoints instead of the ref reach the
  edge through §2.10's `edge-between` accessor." So T3's write referent type is T2's `EdgeKey`
  (§7) — the two trains share the type, unmodified (worked example `:1910-1913`:
  `(update-edge (edge-between EdgeType/SOLIDARITY self other) solidarity/strength (scale 0.95c))`).
- **Owner/type law** (`:1395-1403`): E-TYPE-014 static on add-edge field-inits; on
  `update-edge` the disagreement surfaces at evaluation as E-EVAL-033; a field-init naming the
  implicit `<edge-type>/strength` is E-PARSE-041 ("the `:strength` operand is that field's only
  writer at mint time").
- **Range + mode law** (`:1405-1409`): "a write that would take an edge to a mode the machine
  does not admit from its current one is `E-EVAL-030`, and a store outside the target field's
  declared range is `E-EVAL-020` — never a clamp, never a silent no-op." (I.15 tension: §11.1.)
- **D32 — implicit strength** (`:1709-1715`): every EdgeType carries `<edge-type>/strength`,
  `:type coefficient :kind extensive`, re-declaration E-LOAD-001. **[post-T2-PR-A]** now
  production-wired (§6.2).
- **deffield owner widening** (`:1690-1695`): a deffield's first segment may name an EdgeType;
  unknown owner is E-LOAD-023. Rendering law + E-LOAD-032/E-LOAD-033 (`:1697-1707`).
- **deffield closed type vocabulary** (`:1635-1640` grammar; the seven-row table with `enum`
  superseding D94's six-row closure, `:2165-2171,2384-2393`; Currency in the non-storable
  category, `:2405-2410`) — R1's "same closed type vocabulary" is this table.
- **The write dual framing** (`:1872`): "§2.8's `update-edge` (R9 chapter C2) is the write dual
  of" the edge read lane; `:1907`: `edge-between` absence-is-error is what stops `update-edge`
  degrading to a no-op write.
- **§6.2 chapter C2 — T3's required-vector family** (`:4193-4203`), verbatim scope: "`update-edge`
  under each of the four `<update-op>` forms, against `<edge-type>/strength` and against a
  `deffield`-declared edge field; the same write reaching a range boundary (`E-EVAL-020`) and an
  I.15-illegal mode transition (`E-EVAL-030`); `edge-between` resolving, and failing to resolve
  (`E-EVAL-034`); `add-edge` carrying `<field-init>`s, one of them naming `strength`
  (`E-PARSE-041`) and one owning off the wrong type (`E-TYPE-014`); an `update-edge` whose
  referent is of another edge type (`E-EVAL-033`); an `edge-between` whose enum-ref names a
  `NodeType` (`E-TYPE-011`); and a hydration seeding two edges of one type between one ordered
  pair (`E-LOAD-044`)." Several of these land with T2 PR B (edge-between vectors); E-LOAD-044 is
  already implemented (`babylon-bsl/src/scenario.rs:1338-1352`).
- **for-each over edges applying update-edge** (chapter C6 vector, `:4232-4239`) — the
  effect-position iteration vector family also names `update-edge` per element.
- **Hydration cannot seed edge attributes**: the scenario loader's `load_edge` accepts exactly
  `(edge <EdgeType/MEMBER> <from> <to> <int>)` — an INT strength literal, no field-inits
  (`babylon-bsl/src/scenario.rs:1285-1336`; the int-literal restriction stated at `:1323-1324`).
  The §6 vector-file grammar's `<edge-lit>` likewise carries no field-inits while `<node-lit>`
  does (`bsl-language.rst:4088-4089`). **So at T3's landing the ONLY writers of a non-strength
  edge attribute are `update-edge` and `add-edge <field-init>*` — a scenario cannot seed one at
  hydration.** Whether T3 widens hydration is an undecided scope question (§11.3).

## 6. The post-T2-PR-A surface (everything in this section is [post-T2-PR-A], PR #574, OPEN)

Read via `gh pr diff 574` — 7 commits on `feat/t2-edge-reads-substrate`:

1. **`GraphSubstrate::edge_attribute(edge_type, from, to, attribute) -> Result<f64, GraphError>`**
   — the 16th trait method, appended after `node_type_of` (patch 2, substrate.rs hunk). Contract,
   from its own rustdoc: `attribute` is the FULL QNAME (mirrors `node_attribute`; never a bare
   segment); **the body checks ONLY that the qname ends in `/strength`** (the one thing T2's
   storage holds, D32) and **deliberately does NOT check the owner segment** — ownership is the
   CALLER's obligation (`field_of_edge`'s `check_edge_referent_type`, landing in PR B). Both
   backends implement it as a `HashMap::get` on the existing `edges` map behind the suffix check;
   a non-`/strength` qname or a missing triple is a loud `GraphError`, "never a default 0.0".
   **T3's contract with this method (T2 plan D141, `:1589-1606`): the suffix check widens to a
   real per-qname lookup; ownership validation NEVER migrates into the substrate; THE SIGNATURE
   DOES NOT CHANGE.**
2. **Four conformance rows** (§3.1) registered in `run_substrate_conformance`.
3. **The D32 implicit-strength wiring** (patch 3, `babylon-tick/src/lib.rs`): `prepare_rules` now
   seeds `<edge-type>/strength` `TypeEnv` rows via
   `FieldRegistry::with_implicit_edge_strength(vocabulary).type_env_fields()` from the scenario's
   `defvocabulary EdgeType` members, byte-sorted (patch 5) before an E-LOAD-001 collision check
   that refuses an explicit re-declaration. Recorded narrowing: a scenario with NO
   `defvocabulary EdgeType` block gets no seeding. Consequence for T3: the typecheck pipeline
   already resolves edge-owned qnames in aggregation position; T3's `deffield`-declared edge
   fields flow through `scenario.fields` exactly as node fields do (deffield with an EdgeType
   owner is already legal at load).
4. **`FieldRegistry` doc repaired to "Half-wired"** (patch 4, `declarations.rs:285-…`):
   `with_implicit_edge_strength` IS production-wired; `FieldRegistry::declare` still has no
   production caller (the Phase-2 content-pack registry remains dormant).
5. **The III.7 pin generalized** (§2.4 item 3) and the `rust:check` gate grew a babylon-graph
   pedantic clippy leg (patch 6, `.mise.toml:1647`).

## 7. What T2 PR B will add (collision-avoidance summary)

Source: `docs/superpowers/plans/2026-08-12-t2-slice2-edge-reads-plan.md` (1731 lines; Tasks 3-7 =
PR B, branch `feat/t2-edge-reads-evaluation`, plan `:1665-1683`).

- **Task 3** (`:502-…`): `query::EdgeKey { source: NodeId, target: NodeId, edge_type: String }` —
  field order deliberately `(source, target, edge_type)` so derived `Ord` matches §2.6's total
  order (`:521-533`); `Element::Edge(EdgeKey)` with `Element` DROPPING `Copy` (nine call sites,
  `:541-544`); cross-kind Ord RULED Node-before-Edge (`:534-540`, filed as D140);
  `Value::EdgeRef(EdgeKey)`; `materialize_edges`; the three-file refusal-assertion sweep.
- **Task 4** (`:1089-1202`): `eval_edge_between` — evaluates endpoints to NodeRefs, calls
  `edge_attribute(edge_type, from, to, "<type>/strength")` for the existence check, absence →
  `EvalCode::NoSuchEdge`/E-EVAL-034 (`:1108-1163`); static/runtime fuel agreement pinned at 3
  (`:1186-1195`).
- **Task 5** (`:1204-1322`): `field_of_edge` + `check_edge_referent_type` — generic over any
  edge-owned qname, passes the full qname UNMODIFIED to `edge_attribute`; "a T3-era field name
  resolves through this same function with zero changes to it — only
  `GraphSubstrate::edge_attribute`'s body widens" (`:1212-1219,1252-1257`); the not-yet-stored
  branch maps to E-EVAL-033 and its doc proves E-EVAL-034 is unreachable from inside it
  (`:1259-1267`).
- **Task 6** (`:1324-1519`): `edge-lane-e2e.bscn` + `tests/edge_lane_e2e.rs` — the e2e vector
  file T3's own vectors will sit beside (the `query_lane_e2e.rs` provenance discipline,
  `:1700-1708`).
- **Task 7** (`:1520-1659`): register rows **D139-D142** ("next free is D139, RE-CHECK at PR time
  per the D105 discipline", `:1523-1524`); **ADR201** ("next free ADR number… currently ADR200",
  `:1525-1526`); the hash-free proof; and the `the` disposition — recommendation: `the` becomes
  its OWN micro-train (T2.5) because its load-time legality machinery (`manifest.rs`) is wired to
  nothing production (`:1609-1637`). Issue #572 stays open.

**Collision consequences for T3's plan:** T3 reserves D-rows from **D143** and ADR from
**ADR202**, both RE-CHECKED at PR-open time (the D105 discipline, worked instance
`bsl-language.rst:5788-5798`); T3 must not touch `eval_edge_between`/`field_of_edge`/`EdgeKey`
(only `edge_attribute`'s body + storage beneath them); and T3's conformance edits supersede PR A's
`edge_attribute_of_an_unstored_qname_is_loud` row (§3.1).

## 8. The attributed-membership estate (Amendment AG / ADR189) — what exists, what is fenced off

- **In babylon-graph, the entire estate is one dormant unit struct**: `MembershipPayload`
  (`hypergraph_store.rs:52-66`) — the `M` slot in
  `Hypergraph<(), String, MembershipPayload>` (`:88`). Its own doc: "**Carried, empty, unhashed —
  nothing in this train writes it.** The library exposes the slot with zero accessors (six
  construction sites hard-code `M::default()`, zero reads — `percy-raskova/hypergraph-rs#2`)…
  the moment an AG task adds a write through it, this comment is the first thing it must
  revisit." No other membership/attributed/lattice machinery exists in the crate (rg sweep over
  `src/`+`tests/`: every other "membership" hit is the dyadic MEMBERSHIP edge-type string in
  fixtures, `backfire.rs`/`induced.rs`).
- **In babylon-bsl**: `update-membership` is grammar-reserved and refused ("Amendment-AG-era",
  `evaluator.rs:468-491`, test `:2253-2257`); `membership-field-of` is slice-4-unserved
  (`evaluator.rs:526`, test `:2259-2261`); the `(member <enum-ref> <expr> <field-init>*)` grammar
  row exists (`bsl-language.rst:1349`).
- **The fence**: ADR198 R4 + issue #560 — "AG(i) membership payloads explicitly NOT here (R4 —
  #536 charters that ceremony)." T3 must not touch `MembershipPayload`, `update-membership`,
  `membership-field-of`, or hyperedge own-fields. The ONE structural interaction: the shared
  refusal arm (§4.1 item 1) must split so `update-hyperedge` keeps refusing after `update-edge`
  serves.

## 9. Symmetry/parity precedents — the exact pattern T3 repeats

The `node_type_of` / `edge_attribute` precedent, generalized (both were III.7 read-only
widenings; T3 is the first WRITE widening since the trait landed):

1. One trait change in `substrate.rs`, implemented identically on both backends, placed
   consistently (after their `edges` methods for `edge_attribute` — PR #574 patch 2).
2. New conformance rows registered once in `run_substrate_conformance`'s dispatch
   (`conformance.rs:33-45`+) and thereby executed against both stores for free.
3. The differential Twin gains the new op (§3.2) so per-operation byte equality covers it.
4. The covenant-6 mutating-method list grows (§3.3) — manual, easy to miss.
5. Hash claims proven by pinned-digest fixture tests, not asserted (§2.4) — for T3 this splits
   into (a) the elision proof: attribute-free fixtures keep every existing pin, and (b) the dual:
   writing ONE edge attribute moves the hash (the `any_real_change_moves_the_state_hash` pattern,
   `memory.rs:436-457`, `conformance.rs:389-413`).
6. e2e vectors through `run_once_into` against `HypergraphStore` with independently-derived
   values and a same-file determinism leg (`query_lane_e2e.rs` precedent; T2's
   `edge_lane_e2e.rs` sits beside it after PR B).

## 10. Error-code and register inventory

### 10.1 Families and next-free numbers

The normative contiguity paragraph (`bsl-language.rst:3646-3659`): "every decade block of every
family is contiguous, with no reserved and no skipped number — `E-LOAD` 001–004, 010–013,
020–025, 030–033, **040–056**; `E-PARSE` 010–015, 020–022, 030–033, **040–042**; `E-TYPE`
010–017, 020, 030, **040–044**; `E-EVAL` 010–014, 020–021, **030–042**; `E-LEX` 001–003, 010–011,
**020–027**." Therefore next free at scout time: **E-LOAD-057, E-PARSE-043, E-TYPE-045,
E-EVAL-043, E-LEX-028**. E-EVAL-043 is only ever MENTIONED as "next free" in D105's row
(`:5796-5797`), never allocated. T2 PR B mints NO new codes (plan self-review; grep of the plan
finds no new-number allocations), so these stand for T3 — re-check at PR time per the D105
discipline, which exists precisely because ADR197's draft was burned by an interim allocation
(`:5788-5798`).

### 10.2 Codes T3's laws already have (no new mint apparently required)

- E-EVAL-020 (range at store boundary, `store_range_check`), E-EVAL-030 (I.15 mode — see §11.1),
  E-EVAL-031 (substrate existence discipline), E-EVAL-033 (referent/owner mismatch at eval),
  E-EVAL-034 (edge-between absence; minted, PR B makes it live), E-EVAL-042 (enum write shape +
  enum arithmetic, `EvalCode::EnumWriteShapeViolation`), E-TYPE-014 (static field-init owner),
  E-PARSE-041 (strength field-init), E-LOAD-001 (implicit-strength re-declaration — live in
  `prepare_rules` [post-T2-PR-A]), E-LOAD-022 (kernel agreement), E-LOAD-023 (deffield owner),
  E-LOAD-044 (hydration duplicate triple, `scenario.rs:1338-1352`). Everything §6.2 chapter C2
  names is already minted. A genuinely NEW failure class would appear only if T3 adds hydration
  seeding of edge attributes (§11.3) or a fifth-section-specific load law — flag in the plan, do
  not pre-allocate.

### 10.3 Register and ADR numbering

- Draft-Ruling Register (`docs/reference/bsl-language.rst`): highest existing row is **D138**
  (`:6651`; D132-D138 filed by the Production port, ADR200). **T2 PR B takes D139-D142** (plan
  `:1574-1607`). T3 drafts from **D143**, re-checked at PR-open.
- ADR index (`ai/decisions/index.yaml`, tail verified): highest is **ADR200**
  (production-port handoff). **T2 PR B takes ADR201** (plan `:1525-1526`). T3's landing ADR
  drafts as **ADR202**, re-checked at PR-open. Issue #560: "Its own landing ADR records the
  implementation."

## 11. Scope tensions the plan author must adjudicate (surfaced, not reconciled)

1. **The I.15/E-EVAL-030 vector vs. the missing I.15 machine.** §6.2 chapter C2 — the vector
   family T3's `update-edge` completes — requires "an I.15-illegal mode transition
   (`E-EVAL-030`)" vector (`bsl-language.rst:4196`), and the spec's own update-edge text binds
   the verb to the I.15 edge-mode state machine (`:1405-1409`). But I.15 is a **declared,
   unimplemented Phase-2 gap** in the Rust estate (`structural_verbs.rs:46-49`: "I.15's edge-mode
   transition law and typed attribute storage… are declared Phase-2 gaps"), and NOTHING in ADR198
   R1-R3, issue #560, or the P29 spec's T3 row charters building it. Either T3's vector family is
   served minus its I.15 leg (a recorded gap), or T3 grows an unchartered subsystem. UNVERIFIED
   which is intended — no document read in this pass decides it.
2. **`update-edge` against `<edge-type>/strength` and the double-storage hazard.** The vector
   family requires update-edge "against `<edge-type>/strength` AND against a deffield-declared
   edge field" (`:4193-4195`). Strength lives in the `edges` map and is hashed in section 0x03
   (§2.2); the fifth section holds the new attributes. An `update-edge … solidarity/strength
   (scale 0.95c)` (the spec's own worked example, `:1910-1913`) must therefore write the 0x03
   slot, NOT mint a fifth-section row shadowing it — otherwise one datum exists in two hashed
   homes. No ruling text names this; the storage design must route strength writes to the
   existing map explicitly. (The suffix check in [post-T2-PR-A] `edge_attribute` already treats
   strength as special on the READ side — the write side needs the same fork.)
3. **Hydration cannot seed edge attributes** (§5 last bullet): `load_edge` takes an int strength
   literal only. If T3's e2e vectors want pre-seeded edge attributes, either a rule's effects
   seed them mid-vector (no loader change; matches the "add-edge field-inits" write path) or
   `load_edge` grows a field-init tail (a scenario-format change with its own E-code and
   spec-sync obligations). The §6.2 family's own E-LOAD-044 hydration vector needs no attribute
   seeding, so the minimal reading requires no loader change.
4. **The refusal-arm split** (§4.1 item 1): `update-hyperedge` shares both refusal arms with
   `update-edge` and stays refused (its storage — hyperedge OWN fields, D65's runtime half — is
   chartered by no P29 train). The retained refusal's message must stop claiming an edge is "one
   f64 strength" the moment that stops being true.
5. **`PendingWrite`'s NodeId key** (§4.2): "through the SAME PendingWrite machinery" (R3) cannot
   be satisfied by the type as it stands; the widening choice (sum-type target vs. parallel batch
   merged in order) interacts with the documented non-commutative application law
   (`structural_verbs.rs:119-134`) and with `tick.rs`'s `all_pending` flat batch
   (`tick.rs:595-622`).
6. **Where does `field-of` learn a T3-era edge field's type?** `bind_field_value`
   (`tick.rs:318`) renders through `TypeEnv`; deffield-declared edge fields reach
   `scenario.fields` via the ordinary loader (E-LOAD-023 owner check per §5). This looks
   already-plumbed [post-T2-PR-A], but only a run proves the enum-typed edge-field read renders
   an EnumRef (not a bare ordinal) — PLAN-MUST-VERIFY.

## 12. Surprises, with evidence

1. **No section is elided today — R2's elision is a first, not a continuation.** The empty
   sections are load-bearing in existing tests (`state_hash.rs:454-463,602-618`), and the
   hyperedge section 0x04 writes `tag+count=0` on every attribute-free graph. The "deliberate
   asymmetry" ADR198 R2 conditionally requires recording is UNCONDITIONALLY required — the
   convention definitely differs.
2. **The production write path is narrower than the refusal sites suggest.** Retiring
   `structural_verbs.rs:387-398` alone would revive `update-edge` only in the retired-from-
   production `execute_effects` harness; production's `collect_item` (`:709-716`) and the
   NodeId-typed `PendingWrite` are the real work (§4.2).
3. **T3's static surface is already fully landed** (grammar, arity, §3.7 cost, reservations,
   E-PARSE-041/E-TYPE-014 field-init checks) — the same "only execution is missing" shape the T2
   scout found for the read lane (§4.4), which materially shrinks T3 relative to a cold reading
   of "the storage keystone."
4. **The enum machinery is 100% reusable and its own doc anticipates T3**
   (`structural_verbs.rs:1302-1303`: a storage-bearing `update-edge` "would reuse this exact
   combine shape too") — enum-set parity (R3) is a call-site addition, not new law code.
5. **T3 touches zero hypergraph-rs surface** — the dyadic half is adapter-native on both
   backends (§2.1), so the fifth section's storage is two plain map fields plus one listing
   method, with no upstream-library dependency (unlike anything AG(i) will need).
6. **An `update-edge` rule loads clean TODAY and aborts at its first admitted tick** — the
   load-time gate covers only the six shape verbs (§4.2). T3 converts a runtime abort into a
   write; no content currently in `content/rules/*.bsl` uses update-edge (the T2 plan's scoped
   grep, plan `:1549-1555`), so no shipped content's behavior flips.
7. **PR A's conformance row `edge_attribute_of_an_unstored_qname_is_loud` is written to be
   superseded by T3** — its doc says so ("T3 (ADR198 R1) widens this") — a rare case of a
   landed test that the next train is EXPECTED to rewrite; the landing ADR should cite it by
   name rather than let the edit look like a covenant weakening.

## UNVERIFIED (honest boundaries of a read-only pass)

1. **Whether T3 is expected to implement the I.15 edge-mode machine** (§11.1) — no document
   decides it; the plan author should put the question to the Director or record the gap.
2. **`write_log.rs`'s `Write` enum variants** — I verified `Write::EdgeAdded/EdgeRemoved` calls
   exist (`structural_verbs.rs:940-945,970-974`) but did not read `write_log.rs`; whether an
   edge-attribute-updated record variant exists or must be minted is unchecked.
3. **Whether `bind_field_value` renders an enum-typed EDGE field correctly end-to-end** (§11.6)
   — plumbing looks symmetric by reading; needs a run.
4. **The exact PR B landing state** — everything in §7 is from the PLAN, not landed code; if PR B
   deviates (its own deviation-recording discipline makes that likely in small ways), T3's plan
   must re-verify `EdgeKey`'s final shape and `field_of_edge`'s final error mapping against the
   merged tree, not against plan text.
5. **Save-format impact** — issue #560 says "every existing golden/baseline/**save**
   byte-identical". I found the hash estate (§2.4) but did not locate any Rust save/persistence
   estate for graph state in this pass; whether "save" means anything beyond the hash pins in the
   Rust engine today is unverified.
6. **UNVERIFIED-CLAIM (repeated from the survey, not re-proven here):** the survey's assertion
   that the D35/D65 gap "gates eleven systems" is its adjudication, not this pass's; the
   eleven-name list (§1.4) is quoted, and its per-system correctness was not re-derived.
7. **The director-gate register issue (#564) and umbrella issue numbering** — taken from the
   project memory file, not re-verified via `gh` in this pass (only #558-#563 and #572 were).
