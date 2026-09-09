# The Social Topology Spine — Verbs, Computation, Persistence (Audit Dossier)

**Commissioned:** Director directive 2026-07-30 — *"how our Verbs affect the Topology and how the
Social Topology itself is calculated and stored in POSTGRES and what not."*

**Method:** five parallel read-only code audits (structural inventory; verb→topology mutation map;
derived/computed topology; persistence fidelity; the P27/Rust forward path), synthesized by hand
after the synthesis agent died on a server error. Every claim below carries a `file:line`. Where an
audit could not reach a conclusion it is marked **UNKNOWN** rather than guessed. Nothing was
modified; no test suite was run.

**Binding law observed:** the spatial substrate is immutable (political claims are overlays); every
tick produces a deterministic hash and non-determinism is a bug; Amendment D = native hyperedges in
the exposed model; Amendment AE = Rust is the engine language and the Python engine is frozen
reference; ADR178 C1 = historical figures calibrate, never stipulate.

---

## 0. The question, and why it is now the critical path

Three separate investigations on 2026-07-30 dead-ended at this spine. The heat-system reformulation
(`reports/heat-system-dossier.md`) needs a real organizational topology for the state to target, and
found the Okhrana-grounded targeting estate inert for want of an organization-to-organization edge.
G3 needs to know what verbs actually move, and found verb cost dead four times over. The
organizational-methods dossier proposes PRESENCE and MEMBERSHIP mechanics on a layer nobody had
audited. And a Director-reserved question (P-A) already noted 510 of 520 ticks writing zero hex rows.

The audit answers all four, and surfaces two findings nobody was looking for. Stated bluntly, before
any detail:

1. **The canonical run persists NO topology.** The batch driver used for `qa:regression` and
   `mise run sim:status` writes zero rows to `node_state`, `edge_state` and `graph_metadata`. Asking
   "how is the social topology stored in Postgres" of the canonical database finds it **empty** — the
   graph exists only in that runner's memory and is discarded at process exit.
2. **The determinism hash cannot detect topology loss.** `tick_commit.determinism_hash` is
   `sha256(f"{session_id}:{tick}:{rng_seed}")` — three scalars. It hashes nothing about persisted
   content. An edge-attribute loss, a dropped node, or wholesale corruption of `node_state` passes
   every existing gate silently.

The good news, equally load-bearing: **where topology IS persisted, the round-trip is essentially
lossless**, and the two losses recorded in project memory are either fixed or confined to a
projection that never adjudicates anything.

---

## 1. The vocabulary and its production reality

### 1.1 Node types — 14 declared, 7 stamped

`src/babylon/models/enums/topology.py:60-74`. Every stamped type is stamped from **one place**,
`WorldState.to_graph()`:

| Node type | Status | Writer | Notes |
|---|---|---|---|
| `TERRITORY` | **LIVE** | `world_state.py:746` | queried by territory/production/substrate/electoral |
| `SOCIAL_CLASS` | **LIVE** | `world_state.py:742` | the dialectic's carrier |
| `ORGANIZATION` | **LIVE** | `world_state.py:750`, `ooda/npc_stub.py:302` | the player's own node |
| `INSTITUTION` | **LIVE** | `world_state.py:758` | |
| `INDUSTRY` | **LIVE** | `world_state.py:770` | backing model is `IndustryHyperedge` with `member_business_ids: frozenset` (`models/entities/industry.py:18`) but it is stamped as **one ordinary node** — "hyperedge" is domain language here, not a graph primitive |
| `SOVEREIGN` | **LIVE** | `world_state.py:786` | |
| `FACTION` | **LIVE** | `world_state.py:788` | |
| `HEX` | **INERT-no-writer** | — | hex state rides on TERRITORY attributes via `domain/economics/substrate/hex_graph_bridge.py`; two still-live dead queries iterate an empty set every tick (`sentinels/vocabulary/registry.py:82-97`) |
| `COMMUNITY` | **INERT-no-writer** | — | lives only in the separate XGI hypergraph, never a BabylonGraph node; a second dead query flagged at `registry.py:98-116` |
| `PERSON`, `KEY_FIGURE`, `ENTITY`, `EXTERNAL`, `COUNTY` | **INERT-no-writer** | — | declared fixture vocabulary; `KEY_FIGURE`'s backing model was retired by ADR084 |

### 1.2 Edge types — 24 declared (recount note: the briefing's "20-odd" is low)

`src/babylon/models/enums/topology.py:100-121`. The material-base edges are **scenario-seeded once
at world build and never created by a system thereafter** — systems only annotate them.

| Edge type | Status | Writer | Reader highlights |
|---|---|---|---|
| `EXPLOITATION` | **LIVE** | scenario seeders `_legacy.py:113,410`, `_legacy_wayne.py:439,486,498` | `economic.py:268`, `struggle.py:718` (severs on revolt), `contradiction.py:389` |
| `WAGES` | **LIVE** | `_legacy.py:139,430`, `_legacy_wayne.py:510`, `single_county.py:107` | `economic.py:440`, `production.py:241`, `reactionary.py:317` |
| `TENANCY` | **LIVE** | `_legacy.py:159,481,490`, `bridge.py:882`, `single_county.py:125` | `territory.py:361`, `electoral.py:309`, `fog/reach.py:166` |
| `TRIBUTE` | **LIVE** | `_legacy.py:420`, `_legacy_wayne.py:447` | `economic.py:359` |
| `CLIENT_STATE` | **LIVE** | `_legacy.py:440`, `_legacy_wayne.py:455` | `economic.py:562` — connects two *social_class* nodes (comprador → periphery), not organizations, despite the name |
| `SOLIDARITY` | **LIVE**, two populations | scenario class↔class (`_legacy.py:451`) **static**; org→class `_mass_work.py:117-120` **accumulator-with-decay** | 14+ readers incl. `solidarity.py:121`, `survival.py:55`, `action_effects.py:283` (repression propagation) |
| `MEMBERSHIP` | **LIVE** | `mobilize.py:214-231` — the engine's **only** producer, its own docstring says so (`mobilize.py:21`) | entryism/electoral paths |
| `TRANSACTIONAL` | **LIVE** | `negotiate.py:117` (org→org, **no node-type restriction**) and `electoral.py:494-534` (org→sovereign, `edge_mode=CO_OPTIVE`) | `negotiate.py:46` flip-check |
| `REPRESSION` | **LIVE (since 2026-07-20)** | `ooda/action_effects.py:216-221` `_bump_repression_edge` | its own docstring records that this type had **zero producers and three read-only consumers** before that date |
| `CLAIMS` | **LIVE** | `collapse_transition.py:170-179`, removed at `:201`, bulk-rewired at `:244` | sovereignty |
| `ADJACENCY` | **INERT-no-writer** | **none** | readers exist and matter: `territory.py:173,284` (heat spillover), `bifurcation/ceiling.py` (contiguity), `graph.py:1008`. Both real scenarios state in their own comments that "no real county-adjacency reference source exists… This scenario emits NO ADJACENCY edges" (`_legacy.py:728-732`, `:986-989`) |
| `COMPETITION` | **INERT-no-writer** | **none** | live read paths over a permanently empty set: `negotiate.py:45`, `bifurcation/analysis.py:325`, `axis.py:30,163` |
| `TARGETS`, `OWNED_BY`, `JURISDICTION` | **RETIRED by ruling** | — | ADR176 ruled these dead |
| `RECRUITMENT`, `EMPLOYMENT` | **chartered, unbuilt** | — | ADR176 chartered them to the org loop |

**The `EdgeType` values are lowercase strings** (`"exploitation"`, `"solidarity"`, …) against
CLAUDE.md's uppercase display convention — a vocabulary-normalization mismatch already flagged in the
Postgres brief (D2), not a data bug.

### 1.3 Hyperedges — Amendment D is ratified law with zero code

`ai/_inbox/amendment-d-analysis-p27.md` (715 lines) found **three inconsistent treatments of n-ary
membership in Python today**: community (transient `xgi.Hypergraph`, rebuilt every tick, never a
node), Industry (`frozenset` membership attributes on an ordinary node), and dialectic pole
(off-graph). The Director ruled **D-1 NATIVE HYPEREDGE** — first-class in the exposed model, with
Levi/incidence sanctioned as *internal storage only*, unobservable at the API. **D-4 explicitly
declines to grandfather** `ECONOMIC_SECTOR`'s current frozenset-on-node shape: it must migrate to a
true hyperedge. Recorded at `CONSTITUTION.md:436` and `:639`. **No hyperedge primitive exists in any
codebase today** — see §6.

---

## 2. What the verbs actually do to structure

The blunt summary: **of nine verbs, exactly one creates an edge between two organizations, two
create org→class edges, one is edge-generative only after a round-trip, and five are purely scalar.**

| Verb (cell) | Structural effect | Reachable? |
|---|---|---|
| **EDUCATE** (build/pop) | CREATE-or-MODIFY org→`social_class` SOLIDARITY via `apply_mass_work_solidarity` (`_mass_work.py:107-122`); Doctrine sub-verb writes `study_target_id` scalar on the actor (`educate.py:122`) | yes |
| **CAMPAIGN** (build/pop) | same SOLIDARITY producer (`campaign.py:120-127`), scaled by `debs_solidarity_efficiency` in `election:run`; `election:boycott` is scalar-only | yes |
| **AID** (mgmt/pop) | same SOLIDARITY producer (`aid.py:67-69`) — applied **unconditionally, including when the material transfer fails** | yes |
| **MOBILIZE** (project/pop) | `canvass` sub-mode CREATEs org→`social_class` MEMBERSHIP (`mobilize.py:214-231`), the engine's only MEMBERSHIP producer; refuses rather than clobbers a foreign edge type (`:232-243`) | yes, capability-gated |
| **NEGOTIATE** (build/other) | **the one org→org structural verb**: flips an antagonistic edge to TRANSACTIONAL or CREATEs a fresh org→target TRANSACTIONAL (`negotiate.py:117-124`), with **no node-type restriction on the target** (`:80-85`) | yes |
| **MOVE** (project/org) | writes only scalar `territory_ids`/`headquarters_id` (`move.py:66-74`) — **but** `WorldState.to_graph()` **synthesizes** an org→territory PRESENCE edge per id on every round-trip (`world_state.py:748-754`) | edge-generative **one hop removed** |
| **ATTACK** (project/other) | scalar `heat` on the actor; the target effect is a scalar `infrastructure` decrement in `ooda/layer3.py:193-201`, plus a uniform corridor-mesh edge-attribute degrade at `:204-214`. No create/delete anywhere | yes |
| **REPRODUCE** (build/org) | scalars only on the actor (`reproduce.py:76-96`) | yes |
| **INVESTIGATE** (mgmt/other) | "mutates NO MATERIAL graph state" (`investigate.py:3-4`); one scalar `investigation_intel` on a territory, player-org only | yes |

**Two corrections to the heat dossier**, both important:

- Its claim that *no production writer creates an org-to-org edge* is **wrong for TRANSACTIONAL**
  (NEGOTIATE creates it with no node-type restriction; ElectoralSystem creates org→sovereign
  CO_OPTIVE at `electoral.py:494-534`) and **right for SOLIDARITY** (`_mass_work.py:96-97`
  early-returns unless the target is a `social_class`, because mass work organizes classes).
- **PRESENCE is not a verb product at all** — `investigate.py:20-21` states outright that "no verb
  can create PRESENCE edges yet." PRESENCE exists **only** as a `to_graph()` synthesis artifact from
  the scalar `territory_ids`. Any mechanic proposing to read PRESENCE co-projections must either give
  it a real writer or read `territory_ids` directly.

---

## 3. What the systems compute

Structural creation by systems is **rare and concentrated in four places**; everything else annotates
a fixed node/edge set. Grepping all 34 systems for `add_edge|remove_edge|add_node|remove_node`:

1. **DecompositionSystem** (@11.0) mints a `CARCERAL_ENFORCER` or `INTERNAL_PROLETARIAT`
   `social_class` node on demand when the labour aristocracy decomposes and the target role does not
   exist (`decomposition.py:241`). Never removes: the LA is soft-deactivated (`active=False`, `:339`).
2. **CollapseTransitionSystem** (@20.5) is **the only genuine rewirer** — creates `sovereign` nodes on
   collapse-partition and secession (`:157-168`, `:229-240`), creates CLAIMS edges (`:170-179`),
   removes the old sovereign's claim per transferred territory (`:201`), bulk-rewires via an O(K)
   protocol method (`:244`), and deletes orphaned sovereigns at end of tick (`:290`).
3. **StruggleSystem** (@16.0) **severs** every outgoing EXPLOITATION edge from the periphery
   proletariat when `p_revolution > p_acquiescence` (`struggle.py:677-724`).
4. **ElectoralSystem** (@17.45) creates the CO_OPTIVE TRANSACTIONAL edge described above.

Every other structurally-suspicious system — Solidarity, ImperialRent, Contradiction,
ContradictionField, FieldDerivative, EdgeTransition, Territory, FactionInfluence, Sovereignty,
MarketScissors, WealthDistribution, Doctrine, Reactionary — **never** creates or removes structure.
`SovereigntySystem` and `FactionInfluenceSystem` never touch the graph object at all; they write
`context.persistent_data`.

**Recomputed vs accumulated** — the distinction is load-bearing for heat and for persistence:

- `value_flow` on EXPLOITATION/WAGES/TENANCY/TRIBUTE is written **fresh every tick** by
  ImperialRentSystem (a flow, not a stock).
- `tension` is written **fresh every tick** by ContradictionSystem, whose docstring calls this "the
  fresh per-edge tension, which replaces the add-only accumulator" — a deliberate move away from
  accumulation.
- Org→class `solidarity_strength` is a genuine **accumulator with decay**: written by mass work,
  decayed multiplicatively every tick by `DoctrineSystem._decay_mass_work_solidarity_edges`
  (`doctrine.py:116-139`) unless renewed. CommunitySystem only *amplifies* existing edges
  (`community.py:527-576`), never creates.

**Graph algorithms actually called in a tick:** effectively none beyond neighbourhood queries and
one contiguity walk (`graph.py:1008 query_contiguous_component_under_predicate`, used by
bifurcation ceiling checks). **There is no centrality, betweenness, cutset or percolation computation
in production today** — the percolation-theory documentation describes design, not called code. This
is what the heat proposal's topological terms would be adding, and it also means the ADJACENCY gap
(§1.2) currently starves the one spatial algorithm that does exist.

---

## 4. How it is stored, and what is lost

### 4.1 Two runtimes, disjoint persistence — the finding that reframes the question

| | Interactive `GameSession` | Canonical/batch `WorldStateBridge` |
|---|---|---|
| Entry | `session.py:1625-1633` → `persist_tick_atomic(envelope, graph=self.graph)` | `bridge.py:567` → `persist_tick_atomic(envelope)` — **no `graph=`** |
| `node_state` / `edge_state` / `graph_metadata` | **WRITTEN** (the only production writer, `_legacy.py:2700-2833`) | **ZERO rows, ever** |
| `dynamic_hex_state` | **ZERO rows, ever** (envelope carries only session/tick/hash/boundary rows) | written (delta-selected) |
| consciousness / demographics / employment / audit rows | **ZERO rows, ever** | written |

Independently corroborated by row count in `reports/postgres-brief-2026-07-29.md` §C1: the
hex-heavy 520-tick canonical session has `node_state` count **= 0**; every topology measurement in
that brief comes from a different, 17-tick interactive session.

**Consequence:** `qa:regression` and `mise run sim:status` — the runs we call canonical — prove
nothing about topology persistence, because they never persist any. Conversely the interactive game
never exercises the hex/demographic write paths.

### 4.2 Where topology IS persisted, the round-trip is essentially lossless

`_persist_nodes`/`_persist_edges` (`_legacy.py:2700-2790`) serialize the **full**
`graph.nodes(data=True)`/`graph.edges(data=True)` attribute dicts verbatim into JSONB; the promoted
scalar columns are duplicates for indexing, not the source of truth.
`_persist_graph_attrs` (`:2792-2833`) stashes the **entire** `dict(graph.graph)` —
`institution_relations`, `economy`, `state_finances`, `contradiction_frames`, `opposition_states`,
`field_stack`, `superstructure_registers`, `player_org_id`, `market`, … — into `graph_metadata.extra`,
and `hydrate_graph` (`:268-363`) reads every key back via `set_graph_attr`.

**Two corrections to project memory:**

- **"`from_graph()` drops `institution_relations`" is FIXED** — `world_state.py:733` writes it,
  `:938-940` reads it back; it round-trips through Postgres inside `extra` as well.
- **"non-core Relationship attrs dropped" is STILL TRUE but harmless where it matters.** The
  whitelist lives in `_reconstruct_relationships` (`world_state.py:357-393`) whose own docstring says
  so: only `edge_type/value_flow/tension/description/subsidy_cap/solidarity_strength/influence_level/
  support_type/control_level/legal_status` survive. **But** `WorldState.from_graph()` is built only
  for narration, tick-summary and endgame detection (`session.py:1592`) — the adjudicating state is
  `self.graph`, mutated in place and never round-tripped for adjudication, and **resume bypasses this
  layer entirely** (`hydrate_graph` rebuilds from `node_state`/`edge_state` directly). So the loss
  affects what the *narrator and summary* can see, not what the engine decides or what a save
  restores.

The only genuine persistence-layer loss is `_make_serializable` (`:2932-2961`), which drops a value
that fails both `json.dumps` attempts — **with a warning log**, never silently.

### 4.3 The determinism hash proves lineage, not content

Both loops compute it identically — `session.py:442-451` and `headless_runner/runner.py:1665-1667`:

```
determinism_hash = sha256(f"{session_id}:{tick}:{rng_seed}")
```

Three scalars. **No node, edge, or graph-metadata value participates.** The column is
`CHAR(64) NOT NULL` with no derivation check (`migrations/0029_tick_commit.sql:18`), and every
`tick_commit` row in the local instance holds the literal 64-character placeholder `aaaa…aaaa` — a
test fixture value, demonstrating that nothing anywhere enforces derivation.

So the hash chain proves **replay lineage** (same session+tick+seed reproduces the same label) and
says nothing about **state content**. An edge-attribute loss, a dropped node, or corrupted
`node_state` would pass this hash unchanged. A real content digest exists only as a forward P27
concept (`docs/reference/determinism-contract.rst`), and its column
`babylon_meta.campaign.content_digest` is documented to "stay honestly NULL in the Python era."

> **Correction (2026-07-30, on implementing R1).** This section originally said such a loss "would
> pass **every gate we have**." That overstated the gap, and the sharper statement is more useful.
> The dense goldens are a second, independent gate, and they *do* see topology — but only
> **WorldState's** projection of it. `_dense_header` (`tools/regression_test.py:434-473`) derives
> its columns from `state.entities`, `state.relationships`, and `state.territories`, so a dropped
> entity or relationship changes the header and fails the byte comparison, and `_dense_row` raises
> loudly if the set drifts mid-run. What is genuinely invisible is anything living **only on the
> `BabylonGraph`**: the harness contains no `to_graph`/`from_graph` call anywhere, so graph-only
> attributes, node types, edge types outside the projected set, and (once they exist) hyperedges
> never reach a gate at all. That is a narrower hole than first stated and a more precise brief for
> the content hash — which is why it hashes the *graph*, not the WorldState projection. The claim
> about `determinism_hash` itself is unchanged and was always the load-bearing one.

**This is the audit's most actionable finding**, and it is exactly the artifact the rewrite test
demands: a content digest is what would let us prove a Rust rewrite correct.

### 4.4 The hex question (P-A) — answered: neither reading was right

The write path is **working correctly** and the upstream input is **a static template**:
`delta.py`'s own docstring records the measured basis ("zero of 1,045 hex rows change any value
across consecutive ticks"); `select_hex_rows_for_emission` (`delta.py:59-91`) emits a full frame only
on checkpoint ticks (`tick % 52 == 0`) and otherwise only changed rows — 520/52 = **exactly 10**
frames, matching the observed count precisely. The root cause is stated in the bridge's own comment
(`bridge.py:487-490`): `hex_frame` is `self._hex_template` re-stamped with `tick` only, because "the
engine doesn't yet mutate hex-resolution state."

So: the delta **selector** is live and correct; the layer is materially static because **no engine
system feeds real per-tick hex mutation into the candidate frame.** An honestly-documented gap, not a
silent defect — and separately, the interactive session writes **zero** hex rows including zero
checkpoint frames. `v_hex_state_asof` and the sparse-read discipline remain correct as coded and are
still the only sanctioned read path.

### 4.5 Checkpoint-only hydration (ADR176 ruling 31) is satisfied by accident

`hydrate_graph` correctly resolves "latest tick" via the `tick_commit` marker rather than
`MAX(tick)` (`_legacy.py:292-299`), but the row fetch is a direct
`SELECT … WHERE session_id=%s AND tick=%s` — **no interval math is needed only because
`node_state`/`edge_state` store a full frame every single tick** (verified: 17 ticks × 92 nodes =
1,564 rows exactly, no delta compression on topology).

**The live trap:** the Postgres brief's D4 recommends moving node/edge writes to field-scoped delta
persistence (the churn is ~2 keys out of ~40). If that ships **without** implementing ruling 31's
checkpoint-bounded read, `hydrate_graph`'s direct-fetch becomes precisely the unbounded SQL
reconstruction the ruling forbids. Ruling 31 is binding policy **not yet embodied in any guard.**

### 4.6 Dead and dormant schema

- **`community_state` / `community_membership` — the sharpest loss in the estate.** The writer is
  fully implemented and unit-tested (`_legacy.py:493-560`) with **zero production call sites**. But
  the data never reaches the graph either: `CommunitySystem` reads and writes
  `services.hypergraph_config["community_states"]` — a plain dict in the `ServiceContainer`
  (`community.py:296-304`), neither a node nor graph metadata, therefore invisible to
  `_persist_nodes` and `_persist_graph_attrs` alike. Heat, cohesion, infrastructure, visibility,
  legal status, the R-L-F axis, collective identity, dominant tendency and ideological contestation —
  **an entire tier of the social topology — is computed every tick and discarded at process exit.**
  This is the `community_memberships` blocker that ADR171's national-oppression Phase 2 was waiting
  on, now definitively explained.
- **`contradiction_field` / `edge_curvature`**: real writer, called once at `bridge.py:631` with
  `curvatures=[]` hard-coded and an empty `fields` payload. Fires, stores nothing.
- **The whole Layer-8 game-journal family** (`org_snapshot`, `class_snapshot`, `edge_snapshot`,
  `community_snapshot`, `hex_activity`, `economic_summary`, `tick_event`, bundled by
  `persist_full_tick`) plus the Layer-8b multi-resolution cache (`hex_latest`/`hex_substrate` and its
  five composition views) has **exactly one call site in the repository: the legacy web client**,
  which Amendment V/II.8 ruled superseded. Dead as far as the current game is concerned.
- **`graph_metadata`'s typed columns** (`economy`/`state_finances`/`tick_dynamics`) are written only
  by `SessionRecorder`, which has **zero production instantiation sites**. Confirmed live:
  `tick_dynamics IS NOT NULL` for 0 of 14 rows. The same data survives inside `extra`.
- **The torn-tick two-transaction `persist_tick`** (`_legacy.py:118-172`) still exists but is
  reachable only through `SessionRecorder` — **dormant, not exploitable** today.
- **`game_turn` is genuinely LIVE** (`submit.py`, `session.py:230-238`): the record of which verb an
  org issued survives durably, even though the verb's material effect is mostly scalar deltas inside
  that tick's `node_state` row.

---

## 5. The gap ledger, ordered by what it blocks

| # | Gap | Class | Blocks |
|---|---|---|---|
| G1 | `determinism_hash` covers no content; placeholder values pass | **no verification** | everything structural — a lossy seam cannot be detected; the rewrite test has no artifact |
| G2 | Canonical run persists zero topology; interactive persists zero hex/demographics | **disjoint writers** | any claim that `qa:regression` or the canonical DB exercises topology persistence |
| G3 | `ADJACENCY` has no producer; both real scenarios emit none | **no writer** | heat spillover, contiguity/percolation, the C3 tension lens (already known blocked), every spatial graph algorithm |
| G4 | `community_state` never reaches the graph, let alone Postgres | **never crosses the boundary** | ADR171 national-oppression Phase 2; the community tier of the topology entirely |
| G5 | PRESENCE exists only as a `to_graph()` synthesis artifact; no verb creates it | **synthesized, no writer** | the heat proposal's PRESENCE co-projection; the organizational dossier's PRESENCE mechanics |
| G6 | Ruling 31 (checkpoint-only hydration) has no guard; safe only because topology isn't delta-compressed | **latent trap** | any future delta compression of `node_state`/`edge_state` |
| G7 | `COMPETITION` inert with three live readers | **no writer** | the antagonistic-set flip check and two bifurcation reads run over an empty set |
| G8 | Hex layer materially static: the bridge re-stamps a template | **no upstream mutation** | multi-resolution work; P27 Phase 4 hex persistence |
| G9 | Layer-8 journal + `hex_latest` + typed `graph_metadata` columns dead (single legacy-web caller) | **retired-but-present** | schema clarity; ~30 tables of ambiguity |
| G10 | `contradiction_field`/`edge_curvature` writers fire with empty payloads | **no reader-worthy payload** | the Lawverian field-derivative estate |
| G11 | `_reconstruct_relationships` whitelist drops non-core edge attributes | **lossy projection** (narration only) | what the narrator/summary can see; harmless for adjudication and resume |

---

## 6. What P27 must carry

**Current Rust reality:** `rust/Cargo.toml:2-8` lists five members — `babylon-kernel`, `babylon-bsl`,
`babylon-md`, `babylon-tui`, `babylon-tui-python`. **There is no `babylon-graph` crate.**
`babylon-kernel/src/lib.rs` and `babylon-bsl/src/lib.rs` are **five lines each** — a module doc
comment plus lint attributes, zero structs, zero functions, zero tests. Phase 1 Task 1 (scaffolding)
is done; Tasks 3–17 are unstarted in code.

**But Task 11 is fully drafted.** `docs/superpowers/plans/2026-07-29-program-27-phase-1-language-and-kernel.md:1588-1947`
contains the complete `GraphSubstrate` trait as copy-pasteable Rust — the dyadic half
(`add_node`/`remove_node`/`add_edge`/`remove_edge`/`update_node`) plus the **hyperedge half**
(`add_hyperedge`/`remove_hyperedge`/`members_of`/`hyperedges_of`), with `NodeId` and `HyperedgeId` as
deliberately distinct newtypes — plus a `PlaceholderGraph` toy implementation and five unit tests.
It has not been materialized as files.

**Amendment D's arithmetic is a fuel ruling as well as an ontology ruling.** At n = 3,153 counties:
clique 1-skeleton = 4.97M edges, Levi/incidence = 3,153, simplicial closure ≈ 10⁹⁵⁰. A nested
`members-of`-inside-`hyperedges` fold costs `Σ|members|` — linear, not `C(n,2)`
(`ai/bsl-architecture-standard.md:174-178`). Membership change is **whole-hyperedge replacement**
(S-10): there is no `add-member` verb, and per-membership payload fields are "not expressible in this
revision" — an explicitly stated cost, not an omission.

**Must carry into Rust:** the tick entrypoint and 34-system ordering; the EventBus and the 100-member
`EventType` enum byte-for-byte; the `ContentDigest`/`rules_hash`/`canonical_defines_hash` byte
layout; `PerTickTransactionEnvelope` as the kernel replay unit; the `observe()` projection contract
one-way; and the native-hyperedge `GraphSubstrate` API.

**Stays Python (ADR174):** the data-build pipeline, the out-of-process AI observer and vault baker,
the CLI periphery — **and, critically, every current consumer of the XGI hyperedge layer**
(`CommunitySystem`, `domain/bifurcation/` at 1,655 LOC with zero production callers,
`engine/bifurcation_monitor.py`). These are frozen reference and are **not** what gets ported:
Amendment D changed the target data shape, so the Rust port **re-derives** community and bifurcation
logic against the new hyperedge verbs rather than transliterating dead XGI code.

**`hypergraph-rs`'s role is confirmed:** client-side raster/3D only — `babylon-tui` imports
`layout::convex_hull` and `raster::{Camera, Face, Node3, …}` behind the optional `raster` feature
(`scene3d.rs:20-21`). Amendment AE clause x states its "charter does not expand." Whether
`babylon-graph` will *depend* on it or reimplement the same `NodeKind`/`MembershipEdge` pattern is
**UNRESOLVED** — issue #282, absorbed into P27 Phase N. The spec's working assumption is
`StableDiGraph` via the rustworkx-core re-export, petgraph never directly.

---

## 7. Recommendations, cheapest-first

**R1 — Give the tick a content digest (highest priority, and it is the rewrite-test artifact).**
Add a digest over the persisted node/edge/graph-attr payload alongside the existing replay-identity
hash. Today's gate cannot detect a topology loss; this is the one fix that makes every other
structural change verifiable. Implementing the P27 `ContentDigest` byte layout Python-side now means
the Rust port inherits a specification with goldens rather than a promise. **Does not move a
baseline** if added as a new column/field rather than replacing `determinism_hash`. Naming and III.7
wording are a Director call (§8).

**R2 — Stop the two runtimes lying past each other.** Either the canonical/batch driver persists
topology, or we stop describing its database as showing the social topology. Cheapest honest version:
have `WorldStateBridge.persist_tick` pass `graph=` (one argument) and record in the digest what that
costs in rows; the alternative is a documented declaration that topology persistence is
interactive-only. **This moves DB size, not physics** — no baseline effect.

**R3 — Give `ADJACENCY` a producer from data we now own.** Phase 0-D already fetched and pinned
TIGER sources and built the res-7 mask (`h3_res7_land_mask.parquet`). County adjacency is derivable
from the same estate. This unblocks heat spillover, contiguity, the C3 tension lens and every spatial
algorithm at once — the highest structural payoff per unit of work. **This DOES move baselines**
(new edges change physics), so it needs a declared ceremony and a Director-visible drift table.

**R4 — Revise the heat plan's org-to-org inducement in light of §2.** The graph is not empty: real
org→org TRANSACTIONAL edges exist via NEGOTIATE, and org→sovereign CO_OPTIVE via ElectoralSystem. The
inducement should compose *those* with MEMBERSHIP co-projection, and must read `territory_ids`
directly rather than a PRESENCE edge that only exists after a round-trip (G5). Still zero new edge
types, still zero new math.

**R5 — Guard ruling 31 before anyone delta-compresses topology.** A test asserting that
`hydrate_graph` reads only from a full-frame checkpoint tick costs almost nothing today and prevents
the trap in G6 from ever arming.

**R6 — Declare the dead schema.** The Layer-8 journal family, `hex_latest`/`hex_substrate` and the
typed `graph_metadata` columns are single-caller-legacy-web or zero-caller. Retire them or mark them
explicitly held-for-Rust in the schema itself, so the next reader is not misled about what the estate
does. Pairs naturally with the ADR176 five-dead-edge retirement.

**R7 — Do not fix community in Python.** G4 is real and blocking for national oppression, but
Amendment D already ruled the answer (native hyperedge) and the Python engine is frozen reference.
Specify the community hyperedge in the Rust `GraphSubstrate` and its persistence contract; hold the
Python side.

**R8 — Materialize Task 11's `GraphSubstrate` trait.** It is written, cited against real code, and
sitting in a plan document. Turning it into `rust/crates/babylon-graph/` with its five drafted tests
is the cheapest possible P27 Phase 1 advance and it forces the hyperedge/`hypergraph-rs` dependency
question (§6) into the open.

---

## 8. Reserved for the Director

Genuinely reserved — modelling and ideological, not engineering.

1. **Should the canonical run persist topology (R2)?** It changes what `qa:regression` proves and it
   grows the estate. The deeper question is which run is the *reference* run for the rewrite test:
   if topology persistence is interactive-only, then the artifact that proves a Rust rewrite correct
   cannot come from the canonical batch driver.
2. **Is the replay-identity hash renamed, or upgraded in place (R1)?** Constitution III.7 calls for a
   deterministic per-tick hash. Today's value satisfies the letter and not the intent. Upgrading it in
   place changes every stored value (and the wording); adding a second field keeps lineage stable and
   admits that the original was mis-named. This touches constitutional wording, so it is yours.
3. **Does ADJACENCY get derived now (R3)?** It moves baselines and introduces spatial contiguity into
   live physics for the first time. That is a materially different game, not a bug fix.
4. **Do `COMPETITION` and the other inert-with-readers types retire, or get producers?** ADR176 already
   retired five edge types; `COMPETITION` was not among them and has three live readers over an empty
   set. Retiring it changes the antagonistic-set semantics that NEGOTIATE's flip check depends on.
5. **Does the community tier wait for Rust (R7)?** Holding it means ADR171 Phase 2 waits on P27; not
   holding it means writing Python against a data shape Amendment D has already superseded.
6. **`ECONOMIC_SECTOR`'s migration (D-4) is ruled un-grandfathered — when?** `NodeType.INDUSTRY`
   currently ships a `frozenset` membership on an ordinary node, which is exactly the shape the ruling
   declines to bless. The Python engine is frozen, so the migration lands in Rust; confirming that
   sequencing avoids an accidental Python-side fix.

---

*Audit performed 2026-07-30 by five parallel read-only agents; synthesized by hand after the
synthesis pass failed. Companions: `reports/heat-system-dossier.md`,
`reports/organizational-methods-dossier.md`, `reports/funding-verb-historical-dossier.md`,
`reports/postgres-brief-2026-07-29.md`.*
