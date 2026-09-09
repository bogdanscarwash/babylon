# Graph storage capability delta — `GraphSubstrate` vs. hypergraph-rs

> **CARRIED OUT (2026-08-11, ADR193 — `ai/decisions/ADR193_hypergraph_storage_swap.yaml`).**
> `HypergraphStore` (`rust/crates/babylon-graph/src/hypergraph_store.rs`) implements every §8
> covenant this document lists, each with its own test:
> covenant 1 (sole writer) and 2 (feature declaration) and 6 (frozen pre-check) —
> `tests/covenants.rs` source-level checks; covenant 3 (loud preamble) and 9 (sort on the
> ruled key) — `run_substrate_conformance` (`src/conformance.rs`), run against
> `HypergraphStore` in `hypergraph_store.rs`'s own test module; covenants 4 and 5 (node
> universes coincide; never a silent default) and 8 (a deterministic id-reverse-map
> assignment) — `tests/covenants.rs` behavioural checks; covenant 7 (two stores, not one) —
> no test, only the doc comment at the top of `hypergraph_store.rs`. The adapter absorbs
> CD1–CD7 exactly as this document predicts. This train fixes the one genuine upstream item
> (§6, the `EdgeError::EmptyMembers` documentation defect) and pushes it to
> `percy-raskova/hypergraph-rs` (PR #1, now MERGED as `032a5af8`). The membership-payload
> accessor gap this document did not carry (found while writing the swap plan) stays
> enumerated, not closed — `percy-raskova/hypergraph-rs#2`. A mutation-verified differential
> harness (`tests/differential.rs`) proves byte-identity across the swap, rather than
> asserting it. **A second, unrelated upstream item surfaced under adversarial review of PR
> #494's own performance claim: `members()`/`memberships()` (`hypergraph.rs:277-292`/
> `:256-269`) resolve each `petgraph::NodeIndex` by a LINEAR SCAN of the id reverse map per
> neighbor, making `HypergraphStore::encode_state` and `::build` QUADRATIC in hyperedge
> count — not the "worse and super-linear" the first measurement pass understated it as.
> Extended benchmark (n=2,000..20,000, `tests/storage_benchmark.rs::measure_the_quadratic_cliff`):
> doubling n roughly quadruples `encode_state`'s time (3.8x-5.5x measured against both n×2
> and n×2.5 steps), while `MemoryGraph` stays linear over the same range. Filed as
> `percy-raskova/hypergraph-rs#3`, unowned, no fix landed in this train — see ADR193's
> measurement section for the full table and the production-scale reading (3.8s/tick of
> pure hashing at n=20,000, since `state_hash()` runs twice per tick).** This body stays as
> written — it records what its author knew when she made the choice.

**Authority:** ADR179 ruling T3, open field: *"The capability delta is unwritten. It gates
Phase 2 storage work."* (`ai/decisions/ADR179_topology_spine_director_rulings.yaml:98`).
This document closes that field.

**Audience:** the Director, and the engineer who writes the storage adapter next.

**Status:** reference. Seven deltas enumerated; two reserved for Director/spec ruling; no
storage code exists yet on either side of the boundary. **Superseded by execution, not by
correction — see the CARRIED OUT header above.**

**Repo roots for every citation below**

| Prefix | Root |
|---|---|
| (none) | `/home/user/projects/game/wt-reachability` — this repo |
| `hg-rs/` | `/home/user/projects/game/hypergraph-rs` — the sibling library repo |

Library citations are anchored at `crates/hypergraph-rs/src/core/`. Babylon consumes
hypergraph-rs at rev `0c95db0663737b492af27f85e70b223833a18c2e`
(`rust/crates/babylon-tui/Cargo.toml:53`). `git diff --numstat 0c95db06 HEAD --
crates/hypergraph-rs/src/core/` is **empty** and `0c95db06` is an ancestor of HEAD
(`adde3303`), so every library quote below is valid at the revision we actually consume as
well as at tip.

**Delta ids.** Deltas here are **CD1–CD7**. They are *not* hypergraph-rs's own divergence
register D1–D17 (`hg-rs/plans/reconciliation-plan.md` §2), whose numbering collides with
ours coincidentally and whose D1 (empty edges) and D5 (insertion order) sit adjacent to our
subject matter. Where this document means the library's register it says so explicitly.

---

## 1. Verdict

**hypergraph-rs can back `GraphSubstrate` only behind an adapter, and not yet.** Not "as-is"
— every one of the seven deltas requires interception at the boundary. Not "upstream work"
as the main lane — six of seven are absorbable behind the trait, and the one genuine upstream
item is a documentation defect, not a behavioural one. And not yet, for three reasons that
are independent of each other:

1. ~~**Two rulings are reserved and unmade.**~~ **BOTH RULED 2026-07-31 (ADR185), after this
   document was drafted.** Node-removal **CASCADES** (R2): removal is whole, a member list is
   therefore a set of *live* nodes, and the cascade must be made OBSERVABLE through the
   ADR182 write log — cascade is the semantics, silence is not. Iteration order is **numeric
   at the id layer, lexicographic byte order for BSL surface symbols** (R4), which resolves
   the `[draft ruling — Phase 1 review]` clause with no code change. §5.1 and §5.2 below are
   preserved as the analysis that framed the choice; their "reserved" status is spent.
2. **No adapter exists, and neither does its prerequisite.** `rust/crates/babylon-graph/Cargo.toml`
   depends on `babylon-kernel` alone. There is no `impl GraphSubstrate` over hypergraph-rs
   anywhere in `rust/`, and the `NodeId ⇄ String` bimap that every delta's fix rides on is
   unwritten. The workspace's only hypergraph-rs edge today is `babylon-tui`'s optional
   `raster` feature — the 3D/raster path, not storage.
3. ~~**`PlaceholderGraph` violates its own trait today**~~ **— REPAIRED 2026-07-31, same
   session** (§4, CD6). As drafted, `remove_node` removed a node and nothing else, leaving
   dangling dyadic edges, hyperedge member lists naming dead nodes, and orphaned attribute
   rows. This was verified by execution, not inspection — and adopting a new store on top of
   a reference implementation that contradicted the contract would have baked the
   contradiction in. ADR185 R2 ruled the semantics (cascade) and the repair landed with it.

The single sentence that organises the whole delta: **hypergraph-rs is complete against XGI
0.10.2, and XGI 0.10.2's failure discipline is the inverse of ours.** Five of seven deltas
(CD1–CD4, CD6) reduce to the library being *silently permissive* — auto-creating unknown
members, deduplicating repeated ones, accepting empty member lists, panicking instead of
returning `Result` — exactly where Constitution III.11 and `substrate.rs:69-71` ("Absence is
never success … a substrate that silently no-ops either is non-conforming") require loudness.
That is not a defect in the library. It is faithful parity with the thing it was built to
replace. It is, however, the precise content of the Director's caveat (§2.3).

---

## 2. Phase reality

### 2.1 What actually landed — and a correction

The brief that commissioned this document stated that hypergraph-rs Phase 4 is paused at task
6 of 19 and that Phases 5, 6 and 9 are not started. **That is the state of the "State at
pause" table in `hg-rs/plans/EXECUTION-STATE.md`, and that table is stale.** The same file's
own header instructs the reader that the dated UPDATE sections supersede it, and UPDATEs 3,
4 and 5 (all 2026-07-22) record Phase 4, Phase 6 and Phase 5 respectively as done, each with
commit ranges. Those commits were verified to exist:

| Phase | Table says | Actual | Evidence (verified by `git log -1`) |
|---|---|---|---|
| 0+1 core | done | **done** | `b604c6d..767267e`, 64 tests |
| 2 di/simplicial/views | done | **done** | 15/15 tasks, 135 tests |
| 3 linalg + algorithms | done | **done** | `a88c8e1..0308138` + fixes `5fdb4a4`,`5630357` |
| reconciliation (D1–D17 register) | done | **done** | HEAD `703b334`, 296 workspace tests |
| 4 generators/stats/readwrite | **paused at task 6/19** | **DONE** | UPDATE 3; tasks 6–19 executed, `e8af8b4` (2026-07-22) |
| 5 layout + viz | **not started** | **DONE** | UPDATE 5; `36532ac` (2026-07-22), 428 tests |
| 6 convert/dynamics/communities | **not started** | **DONE** | UPDATE 4; `e3632da` (2026-07-22), 406 tests, review *approve* |
| 7 python conformance | done | **done** | `50b7377`, fixes `cecf8fa`; **311 pass / 83 skip / 0 fail vs xgi 0.10.2**; 470 Rust tests |
| 8 wasm | deferred | **deferred indefinitely** | GitHub #289 |
| 9 cli | **not started** | **not started** | 11 tasks, GitHub #280 |
| 10 react package | cancelled | **cancelled** | wire contract salvaged into Phase R |
| R rasterizer | done | **done** | `14e5edd`, 531/536 tests on the raster feature legs |
| I topology ingest | done | **done** | `84dd209` + gate `f4eaed2`, 501/538/543 tests across the feature matrix |

The correction matters in both directions and should not be read as good news alone. It is
better news for storage than the brief implied — every phase the storage estate would sit on
(0–3 core, the D1–D17 reconciliation, and Phase 7's conformance gate) is complete and gated.
It is not news at all for the two reserved rulings, which no amount of library work resolves.
Phase 9 (CLI) and Phase 8 (WASM) are irrelevant to storage; Phase 10 is cancelled.

**Do not treat this as a readiness certificate.** The completed phases certify *XGI parity*,
which is the very property §1 identifies as the source of five deltas.

### 2.2 Phase 11 — the swap — and the ADR number that does not exist

Phase 11, "the Babylon swap", is **not done** and is a different thing from what ADR179 T3
authorises. Two facts the next engineer needs:

- **Its ADR is unwritten and its reserved number is taken.** `hg-rs/plans/EXECUTION-STATE.md:103`
  says "Phase 11 (Babylon swap) stays deferred to ADR083". ADR083 was only ever a *proposed*
  number (`docs/superpowers/specs/2026-07-18-hypergraph-rs-design.md:669`, "a new ADR (call it
  ADR083)"). In this repo `ai/decisions/ADR083_bifurcation_county_fips_fix.yaml` is an
  accepted, unrelated ADR dated 2026-07-18. `pyproject.toml:159` still carries an
  "ADR083-pending" comment. Anyone writing the swap ADR must take a fresh number.
- **Its target is frozen code.** The swap is `import xgi` → `import hypergraph_rs as xgi`
  across exactly 7 files, all under `src/babylon/` — `engine/graph_wrappers.py`,
  `engine/systems/community.py`, `engine/bifurcation_monitor.py`,
  `domain/bifurcation/{analysis,axis,bridges,consciousness}.py` (verified: `rg -c "import xgi"
  src/` returns 7 files). Under Amendment AE those files are the **frozen Python engine**,
  reference-only past the `p27-python-freeze` tag (`docs/reference/freeze-tag.md`). Its gate
  artifact `hg-rs/plans/C9-exclusion-crosscheck.md` scores 10 COVERED / 1 UNTESTED / 0
  EXCLUDED with one open action (A1: an undirected `"x" in H.nodes` containment oracle test).

The consequence: **the C9/Phase-11 estate is not evidence for this delta.** It certifies an
11-expression Python surface (`xgi.Hypergraph()`, `add_node`, `add_edge`, `.nodes`, `.edges`,
`num_edges`, `incidence_matrix`) against the frozen engine. `GraphSubstrate` is a
14-method Rust trait with typed directed dyadic edges, strengths, a contractual iteration
order and a loud-failure discipline. The two surfaces barely overlap. A reader who sees
"311 conformance tests pass" and infers storage readiness has read the wrong gate.

### 2.3 The Director caveat, on record

> *"i choose 2 but be aware hypergraph-rs may not be copmletely one-for-one swappable for
> xgi. i still may need to develop that repository"*
> — ADR179 T3, `director_words`

ADR179 T3's own decision text glosses this: *"Adopting it is therefore a decision to INVEST in
the sibling repo, not an assumption that it is already sufficient."* This document is the
enumeration T3 asked for. Its finding is a sharpening of the caveat rather than a refutation:
the gap is not that hypergraph-rs is an *incomplete* xgi, but that it is a *faithful* one, and
faithful-to-xgi is silently permissive where Babylon is loud.

---

## 3. The delta table

`R` = disposition. **ADAPTER** = absorb behind the trait. **TRAIT** = the trait's own text
must change. **SPEC** = a normative document must be amended. **DIRECTOR** = reserved.
**UPSTREAM** = ask the sibling repo. Cost is T-shirt size for the adapter work only.

| # | Delta | Trait requires | Library provides | R | Cost |
|---|---|---|---|---|---|
| **CD1** | No typed directed dyadic edge | `add_edge(edge_type, from, to, strength) -> Result` + type-and-direction-keyed reads `edges(type)`, `neighbors(node, type, dir)` (`substrate.rs:94,145,156`) | Two **hyperedge** constructors over `String` ids, no type dimension, no strength field, no type-keyed query. `Hypergraph::add_edge(members, idx, attrs)` (`hg-rs/hypergraph.rs:141`); `DiHypergraph::add_edge(tail, head, idx, attrs)` (`dihypergraph.rs:154`) — directed, but as directed hyperedges. `rg edge_type core/` = 0; `rg strength core/` = 1 (a doc comment, `kinds.rs:24`) | ADAPTER | **M** |
| **CD2** | Duplicate member is loud | Member list is a SET: repeated `NodeId` is a loud error, `E-EVAL-031`, "never deduplicated or ignored" (`substrate.rs:165-172`) | Silent HashSet dedupe in production (`hg-rs/hypergraph.rs:183-187`), returns `Ok`. Pinned as intended XGI parity by `tests/test_hypergraph.rs::test_add_edge_deduplicates_members`. Three further permissive sites: `add_edges_from` (`:488`), `add_node_to_edge` (`:492-521`, duplicate membership silently no-ops at `:521`), `DiHypergraph::dedup_preserve` (`dihypergraph.rs:19`) | ADAPTER | **S** |
| **CD3** | Unknown / empty member is loud | Same clause: unknown `NodeId` or empty list is a loud error (`substrate.rs:165-172`); `PlaceholderGraph` implements all three (`placeholder.rs:181,188,193`) | Unknown member **silently auto-created** as `NodeKind::Agent(N::default())` (`hg-rs/hypergraph.rs:189-194`); empty list accepted, mints a memberless hyperedge. `EdgeError` has only `NotFound`/`AlreadyExists`/`TooFewEdges` (`error.rs:40-50`). Auto-create is the crate's ingest discipline across ≥7 delegating surfaces | ADAPTER | **S** |
| **CD4** | Failure is a `Result` | Every fallible method returns `Result<_, GraphError>`; "Loud by construction (III.11)" (`substrate.rs:43-44`). No freeze/thaw verb exists anywhere in `babylon-graph` | `assert_not_frozen()` **panics** (`hg-rs/hypergraph.rs:94-97`), called at 13 sites in `hypergraph.rs` + 7 in `dihypergraph.rs`; 6 of the 13 sit in methods with no `Result` channel. Reachable without any caller calling `freeze()`: `subhypergraph()` freezes its return (`globalviews.rs:57`, exported `lib.rs:37`) and there is no unfreeze. The library ships the fix itself at three PyO3 boundaries (`frozen_check`) | ADAPTER | **S** |
| **CD5** | Contractual iteration order | Ascending id / ascending `(source-id, target-id)`, "never graph-internal storage order" (`substrate.rs:132-135`), restated at all five ranged accessors | **Insertion order, deliberately**, on every read path (`hg-rs/hypergraph.rs:290-293`, `:269`, `:246`, `:879`). Not a determinism defect — `IndexMap` + `shift_remove` (9 sites, 0 `swap_remove`) + `StableDiGraph` make it deterministic *by design*, ratified as an owner override (`hg-rs/plans/reconciliation-plan.md:39`). Wrong axis, not unstable | ADAPTER (mechanism) + **DIRECTOR** (key — §5.2) | **S**, gated |
| **CD6** | `remove_node` cascade | Trait declares **nothing**: one clause, "Returns `GraphError` if `id` names no node" (`substrate.rs:85-86`). BSL §2.8 gives `remove-node` a grammar production (`bsl-language.rst:659`) and zero semantics | Forces a three-way choice: `remove_node(node_id, strong, remove_empty)` (`hg-rs/hypergraph.rs:307-312`) — drop every containing edge / detach and drop emptied / detach and leave an empty edge alive | **TRAIT** (dyadic half) + **DIRECTOR** (hyperedge half — §5.1) + UPSTREAM rider | S/M unblocked; hyperedge leg **not costable until ruled** |
| **CD7** | Order *semantics* | `NodeId(pub u64)` with derived numeric `Ord` (`substrate.rs:32-33`), declared "Opaque" (`:30`) | No ordered read path at all; ids are `String`. The spec says "lexicographic byte order" (`bsl-language.rst:586-587`), which is **underdetermined** for a u64 with no ruled byte encoding | **SPEC** + ADAPTER | **M** |

---

## 4. Per-delta notes the table cannot carry

Only what changes an implementation decision. Full audit records with per-citation
verification status are the source; this section carries the load-bearing residue.

### CD1 — the missing dimension is type + strength, not direction

`DiHypergraph` *is* directed (`DiRole::Tail`/`Head`, `hg-rs/kinds.rs:30-35`). What is absent
is the **dyadic + typed + strength** combination and any type-keyed query. Three adapter
constraints follow:

- **Strength cannot ride in `member_data`.** `DiHypergraph::add_edge` hardcodes
  `member_data: M::default()` (`dihypergraph.rs:200`, `:212`), so that payload is unreachable
  through the constructor. Type and strength must ride in `attrs: E`, the per-edge payload —
  adequate for a dyadic edge, where one edge is one `(from, to)` pair.
- **`neighbors()` needs a two-hop composition.** `node_neighbors` exists on `Hypergraph` only
  (`hypergraph.rs:879`); `DiHypergraph` has none. Compose `dimemberships` → `dimembers`.
- **VIII.9 hazard.** A single `DiHypergraph` carrying both dyadic edges (as `tail=[from]`,
  `head=[to]`) and Amendment D hyperedges would let `neighbors()` over an arity-*n* hyperedge
  read as a pairwise expansion — precisely what `substrate.rs:13-17` forbids ("there is no
  method anywhere that expands a member list into pairwise edges"). Use two stores, or
  provably disjoint type namespaces.

A side `(edge_type → edge_ids)` index is mandatory or `edges()`/`neighbors()` degrade to full
scans. That is an adapter detail, not an upstream request.

### CD2/CD3 — the preamble, and the three ways to get it wrong

Both close with the same ~15-line validation pre-pass in the adapter's `add_hyperedge`:
reject empty, sort and check adjacent-equal for duplicates, check every member against
`has_node` — *then* delegate. The sort is already mandatory (D25 requires ascending member
order out of `members_of`), so the duplicate check is free. Guardrails:

1. **"Provably unreachable" only holds if the adapter is the sole entry point.** Excluded or
   separately wrapped: `Hypergraph::{add_edge, add_edges_from, add_node_to_edge}`,
   `DiHypergraph::{add_edge, add_node_to_edge}`, `SimplicialComplex::add_simplex`,
   `convert/mod.rs:94`. Our trait deliberately has no `add_member` verb (S-10,
   `substrate.rs:179-180`), so those must simply never be reached.
2. **Mirror the ruled arity floor exactly.** Empty is the error; **arity 1 is legal**
   (`placeholder.rs:181-184`, "hyperedge must have at least one member"). "Hardening" to
   arity ≥ 2 while writing the preamble would introduce an unruled cardinality constant — a
   model decision smuggled in as a port.
3. **The existence check is vacuous unless node universes coincide.** The adapter must mint
   nodes through the library's `add_node`, or `has_node` proves nothing and auto-create stays
   reachable.
4. **Feature declaration.** `pub mod core;` is *not* feature-gated (`hg-rs/lib.rs:10`), so the
   permissive ingest is compiled under every feature set. `generators` **is** gated and is not
   in the current pin's closure. The adapter crate must declare `default-features = false` and
   never enable `generators`, or it compiles in a second unguarded ingest surface for free.
5. **A conformance test, not discipline.** A future refactor that "optimises away" the bimap
   lookup reopens the hole with no compile error.

### CD4 — the panic is reachable without anyone calling `freeze()`

The obvious refutation — *the adapter never exposes `freeze()`, so the pre-check is dead code*
— fails. `subhypergraph()` ends with `new.freeze()` (`hg-rs/globalviews.rs:57`) and is a
top-level export (`lib.rs:37`); `copy()` propagates the flag (`hypergraph.rs:466`); there is no
`unfreeze`/`thaw`/`set_frozen` anywhere in the repository. Frozen is **terminal**: a frozen
graph can never be made mutable again, only rebuilt. So `if self.inner.is_frozen() { return
Err(...) }` at the head of each of the 8 mutating impl methods is load-bearing, not defensive
(T3/#560 added `update_edge` to the original seven, ADR198 R1/R3).
`is_frozen()` is public (`hypergraph.rs:445`).

A cheaper structural option worth naming: keep frozen graphs out of the substrate entirely by
never calling the library's view functions, composing induced views over the §2.6 query
surface instead — which is what `babylon-graph/src/induced.rs::co_projected_peers` already
does. That does not remove the need for the pre-check; it keeps the reachable surface at zero
by default.

**Do not adopt `freeze()` to enforce substrate immutability.** If anyone proposes using it for
the Constitution's "the spatial substrate is immutable" MUST, that adds a verb to a ratified
trait and is amendment territory, not an adapter detail.

### CD5 — the sort is partly vacuous, and only one test can tell

Under a monotone-mint bimap (which is what `PlaceholderGraph` does — `NodeId(self.next_id)`,
`placeholder.rs:49`), plus the library's order-preserving `shift_remove` and non-compacting
`StableDiGraph`, **ascending-NodeId order *is* insertion order** for `nodes` and `edges`. The
sort then satisfies the letter of `substrate.rs:132-134` while delivering none of
`bsl-language.rst:589-590`'s stated purpose ("independent of insertion history"). The delta is
non-vacuous for `members_of`, `hyperedges_of` and `neighbors`, whose library order is
membership-add or reverse-corrected adjacency order and bears no relation to id order.

The conformance vector must therefore do two things the current estate does not:

- **Span a decade boundary.** No test in `rust/crates/` binds a `NodeId` ≥ 10; the ordering
  tests use ids 0/1/2, so the `"10" < "2"` failure mode is unseen.
- **Build in an order that differs from id order.**
  `nodes_query_ranges_over_one_type_in_ascending_id_order` (`placeholder.rs:349`) asserts
  `vec![class_a, class_b]` — the *binding* order, which under counter-minted ids is insertion
  order. It discriminates sorted-vs-hash-order (the backing `HashMap` is SipHash-randomised
  per process, so that much is real) but **cannot** discriminate sorted-vs-insertion-order,
  which is exactly the failure hypergraph-rs introduces. Only
  `hyperedge_members_come_back_sorted_not_in_declared_order` (`:324`) genuinely discriminates,
  because it declares `[third, first, second]`.
- **The one adversarial case:** hydrate the same scenario with its node declarations shuffled
  and assert the tick hash is unchanged. That is the only test that distinguishes "sorted"
  from "sorted by something that is insertion order in disguise".

### CD6 — a live defect, verified by execution — NOW REPAIRED (ADR185 R2)

> **Status 2026-07-31:** ruled and fixed. `remove_node` now cascades to edges, hyperedge
> memberships and attributes; a hyperedge losing its last member is removed rather than left
> empty. The analysis below is the record of the defect as found — it is what made the
> cascade question urgent rather than academic, and the attribute leg was missed on the first
> repair pass and caught in review.

As drafted, `PlaceholderGraph::remove_node` (`placeholder.rs:55-62`) had exactly one mutation
in its body: `self.nodes.remove(&id)`. It never touched `edges` (`:24`), `hyperedges` (`:28`)
or `attributes` (`:21`). Five scratch integration tests were written against
`rust/crates/babylon-graph`, run (`cargo test -p babylon-graph`, 5 passed) and deleted. They
prove, not argue:

1. The dangling dyadic edge survives: `edges("coordination") == [(hub, dead)]`.
2. A **live** node's query hands out the corpse: `neighbors(live_hub, .., Any) == [dead]`.
3. `add_edge` on a fresh graph **refuses that same pair** — removal manufactures a state the
   constructor forbids (`substrate.rs:91`).
4. The substrate self-contradicts: `members_of(sector) == [a, dead]` while
   `hyperedges_of(dead, ..)` errors. (Before `725fc2d5` the latter was infallible and returned
   `[sector]` — consistently wrong. That commit converted a consistent-but-wrong answer into a
   contradicting pair.)
5. Orphan attribute rows leak; `next_id` never recycles, so it is a leak, not a collision.

Blast radius in current production code:

- `dossier.rs:254` — a second `resolve_neighborhood` on the same live hub re-resolves the
  corpse and puts it back in the drawer. **Re-collection undoes an explicit `reconcile()`.**
- `backfire.rs:76` — a live person whose only tie is to a destroyed org measures
  `protected_fraction == 1.0`; ties to corpses inflate the protective channel.
- `induced.rs:34-38` — a dead *base* makes a legitimate query on a **live** org fail loudly.
- `exposure.rs` is safe on two independent counts (`validated_scope` at `:45` rejects a dead
  id up front, and the peer filters at `:90`/`:208` discard dead peers) — not luck.

**The dyadic half is forced and unblocked.** Three independent supports: (a) `add_edge`
refuses to construct what `remove_node` manufactures, so leaving it is incoherence rather
than a modelling choice; (b) the alternative is **unauthorable** — BSL has no iteration
construct in effect position ("Folds are the only iteration construct",
`bsl-language.rst:645-648`) and `self` is a `NodeRef` (`:497`), so no rule can express
"remove every incident edge, then remove self" for unbounded degree; (c) the frozen engine
already rules it (`src/babylon/topology/graph.py:192-193`, "Remove a node and all incident
edges"; mirrored in `ai/graph-abstraction-spec.yaml:95` and `ai/topology-system.yaml:73`), so
under ADR183 this is structure-transcription, not a new decision. Note those legacy specs are
**silent on hyperedges** because Python had none — the precedent covers exactly the half that
is forced and exactly not the half that is reserved.

The hyperedge half is §5.1.

### CD7 — underdetermined, not contradicted

Nothing proves the spec and the trait disagree. What is proved is that "ascending …
lexicographic byte order" (`bsl-language.rst:586-587`, itself marked
`[draft ruling — Phase 1 review]`) does not *determine* an order for a `u64` with no ruled
byte encoding. Pinning the canonical id encoding as **big-endian fixed-width u64** makes
lexicographic byte order and the derived numeric `Ord` provably the same order, and the delta
dissolves — at the cost of one doc-comment sentence and **zero trait signature changes**,
which is why the disposition is SPEC + ADAPTER and not TRAIT.

Scope the spec amendment must cover: `determinism-contract.rst:817` specifies a *third* order,
"sorted ascending by `node_id` (string comparison)", over a different identity space (`:904`
shows `"node_id":"C001"`, the frozen engine's domain string, where string comparison is
harmless). That is not an inconsistency today, but the encoding ruling must state which
identity space it governs or the ambiguity survives the fix.

---

## 5. Director questions — RULED 2026-07-31 (ADR185)

Two. Both were reserved because each decides what the model *means*, not how code is arranged;
neither could be resolved by an engineer choosing well. **Both are now ruled** — the analysis
below is kept as the record of what the choice was between, per the immutability-of-history
discipline. The rulings:

- **§5.1 node-removal cascade → CASCADE (ADR185 R2).** Removal is whole: incident edges go,
  the node drops from every member list, and a hyperedge losing its last member is removed
  rather than left empty (an empty hyperedge is unrepresentable, so leaving one would create
  by deletion a state that cannot be created directly). The write-log asymmetry this document
  identifies is answered rather than dodged: `remove-node` is a BSL structural verb, so the
  effect executor emits one record per cascaded edge and membership. A cascade that emits
  nothing has implemented half the ruling.
- **§5.2 iteration-order key → NUMERIC at the id layer, BYTE ORDER for BSL symbols
  (ADR185 R4).** No code change; the normative note is owed to `bsl-language.rst` and the
  trait doc.

The live `PlaceholderGraph::remove_node` defect this document proves by execution (§4, CD6)
was repaired in the same session the rulings landed.

### 5.1 Node-removal cascade — what does a hyperedge member list denote?

**The question.** When a node is removed, what happens to the hyperedges it belonged to?

The trait declares nothing (`substrate.rs:82-86`). BSL declares nothing (`remove-node` has a
grammar production at `bsl-language.rst:659` and no semantics; `rg 'cascade|dangling'` over
that document returns 0 hits). Our sole reference implementation dangles (§4, CD6). The
library forces a three-way choice. Something must decide, and the candidates are different
models:

| | Semantics | Consequence |
|---|---|---|
| **H1 STRONG** | A node's death dissolves every containing hyperedge | A 500-member `ECONOMIC_SECTOR` vanishes when one class node is removed |
| **H2 SHRINK** | The member list denotes a set of **live** nodes, maintained by the substrate; a hyperedge losing its last member is removed | Formations outlive their members; rosters are substrate-maintained |
| **H3 SHRINK-WITH-FLOOR** | H2 plus dissolution below a declared minimum | Requires a minimum-member coefficient with a material derivation |

**The materialist reading.** Does dissolving a class dissolve its sector, or leave the sector
with a hole? H1 says a formation is an atomic fact contingent on every member. H2 says a
formation is a standing relation that survives the loss of any particular member and is
reduced by it. That is organisational theory with direct political content, which is why it
sits with the Director rather than the trait author.

**It collides with a ruling we already hold.** The heat estate's central finding is that
decapitation *fails* against redundant structure — `exposure`'s targeting value is a quotient,
`Δφ(v) / |signature-class(v)|` (`exposure.rs:250-258`), precisely so that "distributed orgs
survive" *emerges* rather than being coded. H1 hands every node removal the power to annihilate
a formation outright: free structural decapitation through the back door of a dyadic verb, in
a subsystem whose whole design insists fragmentation be measured. H1 also arguably reopens the
hole S-10 closed — under H1, content changes a roster *implicitly*, by removing a member node.

**Engineering input the ruling needs — the write log is not symmetric across the options.**
`Write`'s doc states "The variants mirror §2.8's structural verb set exactly"
(`write_log.rs:35-37`). Under **H1** all collateral is expressible with existing variants
(`EdgeRemoved`, `HyperedgeRemoved`): R1 replay stays intact and the mirror discipline holds.
Under **H2/H3** the collateral is a member-list *mutation* for which §2.8 has no verb and can
have none (S-10), so the log must either grow a non-mirroring variant (breaking its stated
discipline) or go lossy (breaking R1 replay, whose window has just closed). That asymmetry is
a cost to price before ruling, not to discover after.

**If H3:** the minimum-member floor must be a declared coefficient with a material derivation,
never a substrate literal (ADR172 ruling 5).

**What proceeds regardless of the ruling** — do these now, they cannot be invalidated by it:

1. Fix `PlaceholderGraph::remove_node` to purge incident **dyadic** edges (forced; §4 CD6).
2. Pin the invariant all three candidates share: **no query may hand out a dead `NodeId`, and
   `members_of` must never name a node `node_exists` denies.** A conformance test, so the next
   substrate cannot regress it.
3. Document the dyadic strong-cascade contract on `substrate.rs::remove_node` and mirror it in
   BSL §2.8, leaving the hyperedge clause **explicitly OPEN** with a pointer to this section —
   rather than writing a semantics nobody ruled.

### 5.2 Iteration-order key — byte or numeric, and over which identity?

**The question.** What is the canonical ordering key: byte-lexicographic over a domain string
id, string comparison over `node_id`, or numeric over `NodeId(u64)`?

This is a **spec amendment**, because the clause in question is marked
`[draft ruling — Phase 1 review]` in the normative document. Three of our own texts currently
answer three ways:

| Source | Key | Edge key |
|---|---|---|
| `bsl-language.rst:585-587` *(draft)* | ascending node-id **lexicographic byte order** | `(source-id, target-id, edge-type)` — a **triple** |
| `determinism-contract.rst:817-821` | ascending `node_id`, **string comparison** over a domain string (`"C001"`, `:904`) | `(source_id, target_id)` — a **pair** |
| `substrate.rs:32-33,144-145` | derived **numeric** `Ord` on `NodeId(u64)` | `(source-id, target-id)` — a **pair** |

Two sub-questions ride on it:

- **Is `NodeId` a mint counter or a deterministic function of the stable domain id?** I.e.
  must the tick hash be invariant to scenario declaration order? `bsl-language.rst:589-590`
  promises exactly that ("makes fold results independent of insertion history and of the
  underlying graph library") and a mint counter **cannot deliver it** (§4, CD5). Either the
  sentence is weakened to what it delivers, or identity becomes content-derived — a far larger
  ruling that touches III.7/III.12.
- **Is the edge sort key the pair or the triple?**

**Why it cannot be left to the adapter author.** Writing the sort before the key is ruled
risks a wrong-but-stable order that the tick hash would then bless. The recommended resolution
(pin the canonical encoding as big-endian fixed-width u64, §4 CD7) makes byte and numeric order
coincide and costs one doc sentence — but it is still a ruling, because it decides whether
replay is invariant to declaration order.

---

## 6. Engineering questions — not reserved

Recorded so nobody escalates them by mistake.

- **Where the loud-failure preamble lives.** Empty / duplicate / unknown member are *already*
  ruled at `substrate.rs:165-172` citing `E-EVAL-031` and Amendment D. The adapter decides
  where enforcement sits, not what a hyperedge means. (CD2, CD3.)
- **Frozen-ness.** A mutability/lifecycle policy of the storage library, corresponding to no
  material relation in Babylon's model; the trait was ratified without the verb. (CD4.)
- **The dyadic cascade.** Forced three ways (§4, CD6) — transcription, not decision.
- **Store shape, indices, the bimap.** Two stores vs disjoint namespaces, the
  `(edge_type → edge_ids)` index, the `NodeId ⇄ String` bimap. All arrangement.
- **`substrate.rs:152-155` is stale** and unrelated to any delta: it tells the reader to
  "contrast `hyperedges_of`, whose infallible signature predates this ruling", but `725fc2d5`
  made `hyperedges_of` fallible (`:210-214`). One doc line to delete.

### One genuine upstream item

`hg-rs/hypergraph.rs:213-214` asserts that "the strict core `add_edge` rejects them with
`EdgeError::EmptyMembers` by design". **`EmptyMembers` does not exist**: `EdgeError`
(`error.rs:40-50`) has only `NotFound`, `AlreadyExists`, `TooFewEdges`, and `add_edge`
(`:141-208`) performs no empty check — an empty member list silently mints a lone `Hyperedge`
node. The variant was deliberately deleted by the library's own register D1 for XGI runtime
parity (`hg-rs/plans/reconciliation-plan.md:34`), and the library's integration test
`test_add_edge_empty_members_creates_edge` says so verbatim ("the docstring claim of an error
is inaccurate"). A second stale doc repeats it at `simplicialcomplex.rs:221`.

This is a documentation defect only — cost **S**, one issue on the sibling repo. It is
recorded here mainly as a trap: **anyone citing that docstring as evidence of loud library
behaviour has cited a known-false comment.**

---

## 7. Refuted claims — the corrected record

Verification refuted the following. They are recorded rather than dropped, because the next
reader will otherwise re-derive them.

| Claim | Verdict | Evidence |
|---|---|---|
| The library's constructor is `add_hyperedge` | **Refuted** | `rg add_hyperedge` over the whole hypergraph-rs repo: **0 matches**. It is `add_edge`, which collides head-on with our trait's *dyadic* `add_edge` |
| The library is undirected | **Refuted** | `DiHypergraph::add_edge(tail, head, …)` with `DiRole::Tail`/`Head` (`hg-rs/kinds.rs:30-35`). Direction is present; the missing thing is dyadic + typed + strength |
| Insertion order is a determinism defect caused by `shift_remove` | **Refuted** | `shift_remove` preserves relative order (indexmap `map.rs:1073-1074`); 9 call sites, **0** `swap_remove`; `StableDiGraph` leaves holes rather than compacting (`hg-rs/hypergraph.rs:33-34`). Insertion order is a ratified owner override with conformance vectors (`reconciliation-plan.md:39`). Deterministic but on the wrong axis |
| "No sorted-by-id accessor anywhere in the production core" | **Refuted** | `SimplicialComplex::members()` is sorted by construction via `as_sorted_set` (`simplicialcomplex.rs:29-31`); `FaceLattice.immediate_faces` is a public `BTreeMap` (`:20`). True only of the `Hypergraph`/`DiHypergraph` accessors |
| The library both generates decimal ids and byte-sorts them, so `"10" < "2"` is imported | **Refuted / overstated** | The library never auto-mints a **node** id (`add_node(node_id: &str, …)`, `hypergraph.rs:118`); `edge_uid_counter` mints edge ids only. All four sorts in the file (`:1009,1023,1154,1155`) are merge/equality utilities, never read paths. The hazard is one the *adapter* would create by mapping `NodeId → String` |
| `NodeId` is opaque and only obtainable from `add_node`, so unknown members are exotic | **Refuted** | The tuple field is `pub` (`substrate.rs:33`) — `NodeId(999)` is constructible anywhere — and `PlaceholderGraph::remove_node` left dangling ids in member lists (repaired by ADR185 R2 on 2026-07-31; it cascaded nothing when this was written), so `members_of` handed out dead ids on an ordinary read path. Constructible unknown members remain the reason the guard is load-bearing |
| Node-removal cascade is already forced by rulings we hold, so it is not a Director question | **Refuted (hyperedge half)** | S-10 (`substrate.rs:179`) constrains the *verb set* — what content may author — not what the substrate does when a member dies. The "shrink would change the observable `HyperedgeId`" objection rules out a replace-implementation only; in-place shrink changes no id. See §5.1 |
| The frozen-panic is low severity because only an explicit `freeze()` sets the flag | **Refuted** | `subhypergraph()` freezes its return (`globalviews.rs:57`, exported `lib.rs:37`) and there is no unfreeze anywhere. The pre-check is load-bearing, not dead code |
| "Nothing to ask of hypergraph-rs" | **Refuted** | The `EmptyMembers` doc-vs-code divergence (§6). Not about cascade |
| `set_node_attributes` is a frozen-guarded method with no `Result` channel | **Refuted** | Its body (`hypergraph.rs:917-934`) contains no `assert_not_frozen` call. `add_nodes_from` (`:471`) is guarded only transitively via `add_node` |
| `GraphSubstrate` has 13 methods | **Refuted** | **14** (`substrate.rs:80,86,94,107,116,125,128,141,145,156,173,184,191,210`) |
| `babylon-graph` depending only on `babylon-kernel` proves the workspace has no hypergraph-rs edge | **Refuted** | `babylon-tui/Cargo.toml:53` carries the rev-pinned git dep and `babylon-tui-python/Cargo.toml:16` enables its `raster` feature unconditionally in a default-`uv sync` wheel. `pub mod core;` is un-gated (`hg-rs/lib.rs:10`), so the panicking code is **compiled into the default build today** — it is simply never called. The true statement is: no `GraphSubstrate` impl exists |
| hypergraph-rs Phase 4 is paused at task 6/19; Phases 5, 6 not started | **Refuted** | Stale table; UPDATEs 3/4/5 record all three done, commits verified. §2.1 |
| Phase 11's swap ADR is "ADR083" | **Refuted** | ADR083 is an accepted, unrelated ADR in this repo. §2.2 |

Two further corrections of record, neither load-bearing: a claimed raw call-site baseline of
"129 hits" is unreproducible (actual 194 matched lines / 17 files; the derived 7-production /
50-test split is exact), and twelve `placeholder.rs` test line anchors were stale by a constant
−34, matching no committed revision.

---

## 8. Adapter covenants — binding on whoever writes the storage code

Consolidated from §4 so they can be checked as a list.

1. **Sole writer.** The adapter is the only entry point to the library's ingest surface.
   Excluded or separately wrapped: `Hypergraph::{add_edge, add_edges_from, add_node_to_edge}`,
   `DiHypergraph::{add_edge, add_node_to_edge}`, `SimplicialComplex::add_simplex`,
   `convert/mod.rs:94`.
2. **Feature declaration.** `default-features = false`; never enable `generators`.
3. **Loud preamble** before every delegation: empty / duplicate / unknown member. Mirror the
   ruled arity floor exactly (arity 1 is legal).
4. **Node universes coincide** — mint nodes through the library's `add_node`, or the existence
   check is vacuous.
5. **Never let a strength reach `M::default()`/`N::default()`.** The library's auto-fill is a
   silent-default pattern; III.11-adjacent.
6. **Frozen pre-check** at the head of all 8 mutating methods, via `is_frozen()` (the original
   seven plus T3's `update_edge`, ADR198 R1/R3 — `covenants.rs`'s covenant-6 array is the
   executable list).
7. **Two stores, or provably disjoint type namespaces** — `neighbors()` must never read as a
   pairwise expansion of a member list (VIII.9, `substrate.rs:13-17`).
8. **Deterministic bimap.** `NodeId ⇄ String` assignment must be deterministic, because
   ascending-`NodeId` order is an observable contract.
9. **Sort on the ruled key** (§5.2), at every ranged accessor, plus member canonicalisation at
   write to match `placeholder.rs:187`.
10. **Conformance tests, not discipline**, for 1/3/6/9. The trait cannot enforce its own
    ordering contract at compile time; only tests can. `PlaceholderGraph` is the sole impl
    today, so its 13 unit tests (`placeholder.rs`, `#[cfg(test)]` at `:247`) — together with
    the test-side call sites of `add_edge`/`edges`/`neighbors` across `rust/` — are the
    ready-made suite: parameterise them over the second impl. The census reconciles exactly:
    `rg '\.(add_edge|edges|neighbors)\(' rust/ -g '*.rs'` = **58** = 7 production
    `GraphSubstrate` calls + 50 test-side + 1 production BSL-wrapper dispatch
    (`structural_verbs.rs:231`). The trait's read surface is overwhelmingly exercised by
    tests, which is why re-pointing them is cheap and why they are the only real gate.

---

## 9. What this document does not do

- It does not authorise storage code. ADR179 T3 gates Phase 2 storage on the delta being
  *written*; the two reserved rulings in §5 gate it on being *answered*.
- It does not settle the hyperedge cascade, and no adapter may settle it by stealth — e.g. by
  silently filtering dead members out of `members_of`, which is the same sin as silent dedupe
  in a different coat.
- It does not enumerate deltas beyond the seven audited. Absence from this list is not
  evidence of absence; it is evidence that nobody looked yet. Notably unexamined:
  `SimplicialComplex` (independent silent no-ops, register deviation #5), the attribute
  read/write surface against `update_node`/`node_attribute`, and serialization/persistence
  against ADR179 T4's Postgres object.
- It introduces no constant, coefficient, threshold or functional form (ADR172 ruling 5). The
  only numbers are census counts, T-shirt sizes, and `C(n,2)` quoted from the trait's own doc.
  Two spirit-level flags are recorded rather than smuggled: H3 would *require* a minimum-member
  floor (§5.1), and H1 is a stipulated dissolution law of the same species as an imposed
  functional form, in the one subsystem whose design insists fragmentation be measured.

---

## 10. Provenance

Seven findings, each independently re-verified against source at both repos: every cited
`file:line` re-read, production/test splits re-counted from `#[cfg(test)]` boundaries, the
pinned rev diffed against tip, and CD6's defect reproduced by executing five scratch
integration tests against `babylon-graph` (5 passed; files deleted, worktree clean). Nine
citation defects and eleven claim-level refutations were found and are recorded in §7 rather
than silently corrected.

Verified state at authorship: this repo `725fc2d5`; hypergraph-rs `adde3303`, consumed at
`0c95db06` whose `core/` is byte-identical to tip.
