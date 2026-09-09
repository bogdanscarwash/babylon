<!-- vale off -->

# Program 27 — Phase 1 Exit Checklist

**Status: Phase 1 (Language & Kernel) COMPLETE at this document's merge.**
Phase 2 (Content & Intrinsics) starts from the state recorded here, with no
outstanding constitutional gate: Amendment D ruled 2026-07-29 (native
hyperedge), the sigmoid ruling is in hand (ADR176 r21), and every reserved
call the phase surfaced was ruled in ADR180.

Verification battery (Task 18 Step 1), run green 2026-07-30: workspace
tests, workspace clippy `-D warnings`, per-crate pedantic clippy on
`babylon-kernel`/`babylon-bsl`/`babylon-graph`, `cargo fmt --check`, and
`RUSTDOCFLAGS='-D warnings' cargo doc` — **zero errors on every leg**.

Current follow-through (2026-08-23): the deferred anchor total order and
E-LOAD-003 item landed in `babylon-tick` through PER-17 and ADR222. PER-18 and
ADR223 then added detached whole-tick rollback, checked allocator exhaustion,
buffered event publication, and the versioned nominal world hash. These are
in-memory contracts; the durable Gate 3 envelope remains planned. The table
below remains the Phase 1 handoff snapshot rather than rewriting its history.

## DONE — merged, tested, gate-clean

| Task | Deliverable | Landed |
|---|---|---|
| 1 | Crate shells `babylon-kernel` + `babylon-bsl` | PR #374 |
| 2 | The BSL Language Reference (`docs/reference/bsl-language.rst`) — lexis, grammar, typing, evaluation, fuel, CAS; the ONE normative home | merged 2026-07-29; Amendment D revision ratified |
| 3 | Kernel scalars: `quantize` (conformance-pinned against live Python), bounded sorts, `Ratio` grid rejection, `Currency` i128 with the four pinned operators incl. real half-even i256 division | PR #419 |
| 4 | Deterministic sim clock — pure function of `(session_id, tick)` | PR #421 |
| 5 | RNG service: `ChaCha8Rng`, SHA-256 seed derivation, **per-carrier streams** `(session, tick, domain, stable_key)` per the ADR176 r20 rider — no tick-global constructor exists | PR #422 |
| 6 | Deterministic event bus (four ordering guarantees as tests; Block logs the original payload) | PR #423 |
| 7 | `ContentDigest` defines half, conformance-pinned against the real 25,184-byte Python canonical JSON | PR #423 |
| 8 | Sigmoid ruling IN HAND (not implemented — that is Phase 2 by design): ADR176 r21 — pinned soft-float libm crate + bit-exact golden vectors per intrinsic; surviving tick-time intrinsic set is **{exp, log} at most, possibly empty** (`ai/_inbox/sigmoid-ruling-p27.md`) | PR #424 |
| 9 | The reader — full §1 lexical grammar, E-LEX vectors, iterative parse; **spec self-contradiction found and repaired** (the operator atom class, §1.4 draft note) | PR #425 |
| 10 | §3.4 intensivity typechecker (E-TYPE-041/042/043) + the exemptions ledger | PR #426 |
| 11 | `babylon-graph`: the ruled native-hyperedge `GraphSubstrate` trait + placeholder | PR #415 |
| 12 | Canonical AST serialization — §5 binary CAS reproduced the then-current 421-byte worked example byte-for-byte; `rules_hash` MANDATORY on `ContentDigest` | PR #428 |
| 13 | Load-time fuel bound checker — §3.7 cost table (two documented tiers), `bound(rule)`, E-LOAD-040/042, per-query-head ceiling-axis dispatch | PR #429 |
| 14 | Fuel-metered expression evaluator — §3.3 value lanes, §4.3 arithmetic codes, §4.5 per-node charging, `IntrinsicHost` seam | PR #431 |
| 15 | Bindings + `:material-basis` load surface + the `:default` allowlist lint (findings, not errors) | PR #432 |
| 16 | Typed structural verb algebra (all seven + `emit`, guards, the §3.3 store-boundary range check E-EVAL-020, E-EVAL-031 discipline) + modding anchors (E-LOAD-002); trait completions (`&str`, `:strength`, `node_attribute`) | PR #433 |
| 17 | Conformance corpus: 899 Python lines transcribed with the 4-point M8 delta; the composed `rule_pipeline::load_rule` + `bind_environment`; `DEFAULT_ALLOWLIST` populated (6 rows); delta ledger `reports/p27-conformance-corpus-transcription.md` | PR #434 |

## DEFERRED — named, owned, never silent

| Item | Where it lands | Why deferred |
|---|---|---|
| Numeric intrinsics (`exp`, `log`; `sigmoid`/`tanh`/`entropy` are never registered) | Phase 2, gated on ADR176 r21's mechanism (pinned soft-float libm + golden vectors) | The ruling is ratified; implementation is content-adjacent work |
| Concrete graph **storage** behind the trait | Phase 2 — hypergraph-rs per ADR179 T3, opening with a written capability delta (Director caveat: may need development, not one-for-one vs xgi) | The trait is the insulation layer; a storage pick is an engineering choice under a settled shape |
| Query/fold **execution** (materialization, §2.6 iteration order, §4.4 empty-aggregate codes) | Phase 2 query evaluator | Needs the concrete storage; load-time verdicts (typecheck + bound) already pin the corpus |
| The closed `NodeType`/`EdgeType`/`HyperedgeType`/`EventType` enums | `babylon-domain` (Phase 2/3) | The trait and verbs are deliberately domain-agnostic (`&str`) until the enums port |
| Anchor total-order resolution + E-LOAD-003 (Material Base interleave) | `babylon-engine` (Phase 3) | Partition boundaries are engine registry data |
| Typed attribute storage (Currency i128 exactness on node fields) | Phase 2 trait revision | `f64` attributes cannot hold i128; Currency writes are LOUD errors today, never lossy casts |
| Per-membership payload + hyperedge field mutation/init | Phase-1 review items the §2.8 ruling itself names | Whole-hyperedge replacement is the ruled shape |
| Full §3.4 kind PROPAGATION over compound fold bodies; E-TYPE-010 (cross-type `:field` scoping); E-TYPE-012 (`it` outside query context) | Phase 2 full typechecker | Compound fold bodies are rejected loudly as unverifiable today, not passed unchecked |
| Hydration ceiling check (E-LOAD-041) + the §4.5 runtime meter wired into a tick | Phase 2/3 engine assembly | The load-time halves are live |

## The two flagged Phase-1-internal TODOs — dispositions

1. **Task 16's `leak_str` interning question — RESOLVED, no leak shipped.**
   The trait was revised to `&str` (recorded in the trait's module doc); no
   `Box::leak` call exists in the workspace — the sole textual occurrence is
   that module doc recording the rejected shape.
2. **Task 17's rule-loading composition — BUILT** (`babylon_bsl::rule_pipeline`).
   Phase-2 generalization notes: extend from one rule to a content SET
   (`read_all` + duplicate-id `E-LOAD-001`), load `deffield`/`intrinsic-decl`/
   `manifest` forms into the registries the pipeline currently takes as opaque
   inputs, and fold `rules_hash_of` over the loaded set into `ContentDigest`.

## Draft rulings accumulated for the Phase-1 review (implementation-discovered)

All recorded in `docs/reference/bsl-language.rst` at their sections, each with
its date: the operator atom class (§1.4), query-operand charging (§3.7), the
§3.7/§4.5 fuel-boundary off-by-one, and id-operands-as-effect-scoped-names
(§2.8). Plus the module-doc-recorded `E-LOAD-010/011/030` per-source
assignment ambiguity (`bindings.rs`).

## What Phase 2's first task should read first

1. `docs/reference/bsl-language.rst` — the normative language, including every
   draft-ruling note above.
2. `reports/p27-conformance-corpus-transcription.md` — what executes vs what
   pins, and the corpus fixtures Phase 2's query evaluator must make run.
3. `ai/_inbox/sigmoid-ruling-p27.md` — the intrinsic-set analysis under
   ADR176 r21.
4. `rust/crates/babylon-bsl/src/rule_pipeline.rs` — the composition seam the
   content pipeline generalizes.
5. `ai/decisions/ADR179_*` (T3: hypergraph-rs as storage) and the
   babylon-graph trait docs — the storage decision's constraints.

<!-- vale on -->
