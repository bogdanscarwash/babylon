# Community @6.0 — the Hypergraph Layer Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port `CommunitySystem` (`src/babylon/engine/systems/community.py`, 675 lines, Material Base
**position 6.0**) into ONE BSL rule pack, `community.bsl`, and — because no content-reachable
hyperedge surface exists at HEAD — **take the BSL hyperedge lane live as a NAMED deliverable**
(#536 charter rider 3). Four of the frozen system's six phases land: the community census, the
per-community ternary consciousness from the org landscape (with ADR214's ruled floor table), the
per-agent reproduction-cost modifier, and the heat/cohesion/education-pressure decay. **Two phases
do not land** — threat scoring and solidarity amplification, plus infrastructure decay's
CORE_ORGANIZER maintenance term — because all three read **per-membership** payload
(role / strength / visibility), which is Amendment AG(i)'s object and **#653's ceremony**, pulled
out of this charter by explicit Director act (#536 sequencing comment, 2026-08-17; #564 row 22).

**Architecture:** Two estates in one train, in strict order.
1. **The engine lane** (`babylon-graph` + `babylon-bsl` + the `babylon-tick` DRIVER): a
   type-scoped hyperedge **enumerator** on `GraphSubstrate` (absent at HEAD — §3.2 row 0),
   hyperedge scenario seeding, the three slice-3 hyperedge query heads with the `Element`
   cross-kind Ord ruling rider 4 requires, the **ceiling supply chain** that makes a
   hyperedge-querying rule loadable at all (`LoadedScenario.hyperedge_types` + the driver's two
   ceiling maps — §3.2 rows 9-10), hyperedge **own-field** storage with a sixth `CanonicalState`
   section (`0x06`) and a sixth REQUIRED `CanonicalState` listing under the ADR198 R2
   empty-elision discipline, and the BSL read/write surface for it (`field-of` over a
   `HyperedgeRef`, an executing `update-hyperedge`).
2. **The content pack** (`babylon-tick` content): `community.bsl` — **14 rules** across 12 rule-table
   rows, one `HyperedgeType` member, the estate's **singleton** INSTITUTION carrier (reused, not a
   second one — §3.7a), a 14-row ADR214 `defconst` floor table, **eight conformance worlds** with two
   Python frozen mirrors, additive golden pins, **25 D-rows** and **ADR-NF**.

**No new formalism.** The hyperedge own-field lane widens storable STATE using existing constructs
(`deffield`, `update-hyperedge`, `field-of`) exactly as ADR198 R1 / ADR203 did for dyadic edges; it
mints no verb, no intrinsic, no element kind. Content mints **instances only** — one
`HyperedgeType` member and 14 `CommunityType` enum members transcribed from the frozen enums. The
one intrinsic this pack declares (`log`) is already in `DECLARABLE_INTRINSICS`
(`declarations.rs:125`) and ADR213 names **Community entropy** as one of its two ready consumers.

**Tech Stack:** Rust workspace (`rust/crates/{babylon-bsl,babylon-graph,babylon-tick}`), BSL
content, cargo via `mise run rust:check`, Python 3.12 host venv for the frozen mirrors.

**Rulings that govern:** ADR183 port-as-is (the frozen system is the structure/ordering contract,
never a correctness oracle); Amendment AG(i)+(ii) / ADR189 (attributed membership — **out of this
charter**, #653); Amendment D / AE (vi) NATIVE HYPEREDGE (hyperedges are first-class in
`babylon-graph`'s exposed model; Levi/incidence is internal storage only); Anti-Pattern VIII.9 +
INV-010 (a member list crosses WHOLE, never C(n,2); **communities are never graph nodes**);
ADR198 R1/R2/R4 + ADR203 (the dyadic-edge storage precedent this lane mirrors, and R4's explicit
hand-off of the membership half to #536/#653); ADR214 T7 Rulings 1-4 + erratum 9 (the floor table:
F-B shape, the LOW-five demotion, the measured ordering, the `defconst` + deferred-§6.5-ceremony
entry path); ADR208 R14 (Checkpoint A = all 13 Material Base systems ported); ADR173 / ADR172
ruling 5 (**no imposed functional forms**); ADR195 (enum member order is hash-bearing); ADR181
(merge protocol + Copilot harvest); Constitution III.7 (a canonical-state field-set widening is a
declared substrate decision).

**Prior art to read before Task 1 (in this order):**
1. The four charter dossiers for this train (frozen estate; governance; Rust substrate; content
   precedents & numbering), in
   `/tmp/claude-1000/-home-user-projects-game-babylon/*/scratchpad/community-charter/`. Their
   **boundary analysis governs scope** and their **substrate verdict governs §3**.
2. `rust/crates/babylon-bsl/src/scenario.rs` **in full** — the loader's own "**No hyperedges yet.
   The grammar has room for them; nothing in slice 1 needs one, and an unused form is an untested
   form**" (`:63-64`) is Task 1's whole brief, and its declaration-order-is-id-order law (`:41-45`)
   is the law Task 1 must extend to hyperedges without disturbing node ids.
3. `rust/crates/babylon-bsl/src/{query,evaluator,structural_verbs,typecheck,score_class}.rs` — **BOTH**
   unserved-head tables and the served one (`query.rs:99-103` `UNSERVED_QUERY_HEADS`, the table
   `materialize()` actually consults, whose own doc at `:91-98` says it is kept in sync with
   `evaluator.rs:544`'s `UNSERVED_EXPRESSION_HEADS`; `evaluator.rs:567`'s `SERVED_QUERY_HEADS`),
   **`query.rs:56-75` — the `Element` enum's own standing cross-kind Ord instruction** (register row
   D140, CT4P A5 / #525) and `query.rs:17`'s "`Hyperedge(HyperedgeId)` (slice 3) is deliberately not
   added", the `field-of`-over-`HyperedgeRef` refusal (`evaluator.rs:1318-1322`), the
   `update-hyperedge` refusals (`structural_verbs.rs:452-466` execute path, `:873-879` collect path),
   `add_hyperedge`'s field-init refusal and its **member-list canonicalization**
   (`structural_verbs.rs:1335-1343`; the sort at `:1366`, the "recorded WHOLE" note at `:1371-1373`
   — D25), and `DEFERRED_SHAPE_VERBS` + its load gate (`:1723-1776`).
3a. `rust/crates/babylon-bsl/src/{fuel,bound_checker,manifest}.rs` **and**
   `rust/crates/babylon-tick/src/lib.rs:263-276` — the **already-landed** hyperedge fuel axis
   (`fuel.rs:112-118` `max_members`; `bound_checker.rs:544-572` `ceiling_of_query`, which bounds
   `hyperedges`/`hyperedges-of` against `ceiling("HyperedgeType/X")` and `members-of` against
   `max_members("HyperedgeType/X")`; the `E-LOAD-042` errors at `:74-90`, messages `:144-157`, tests
   `:897-917`, `:995-1002`; `manifest.rs:11-15`'s "`:max-members` is **mandatory** on a
   `HyperedgeType` row") **and the gap that makes it inert**: the driver builds
   `CardinalityCeilings::new(<NodeType+EdgeType counts>, HashMap::new())` — the `max_members` map is
   **empty** and no `HyperedgeType/*` ceiling is ever produced. This is §3.2 rows 9-10 and Task 4's
   whole brief.
4. `rust/crates/babylon-graph/src/{substrate,memory,hypergraph_store,state_hash,conformance}.rs` —
   the hyperedge API (`substrate.rs:235-276`) and **the enumerator that is NOT in it** (`nodes` `:204`
   and `edges` `:208` are type-scoped; nothing hyperedge-side is — §3.2 row 0), `HyperedgeId`'s
   deliberate type-level separation from `NodeId` (`substrate.rs:35-41`), both backends' hyperedge
   stores (`memory.rs:57` `HashMap<HyperedgeId, (String, Vec<NodeId>)>`; `hypergraph_store.rs:90-91`'s
   `hyperedge_keys` + **`hyperedge_type_index`**, an adapter-side type index that already exists),
   the empty `MembershipPayload` marker (`hypergraph_store.rs:53-67`), `CanonicalState`'s
   **"five-way listing"** doc and its "**why this trait exists rather than widening
   `GraphSubstrate`**" paragraph (`state_hash.rs:292-320`) plus the REQUIRED-not-defaulted fifth
   listing argument, the five hash sections with `write_edge_attributes`' **"the elision decision is
   the CALLER's"** note (`state_hash.rs:100-104`, `:250-256`) — the exact shape section `0x06`
   copies — and `conformance.rs`'s `run_substrate_conformance` (`:28-45`), the both-backends suite
   every landed `GraphSubstrate` accessor has a row in.
5. `rust/crates/babylon-tick/content/rules/consciousness.bsl` **in full** — `p8-dominant-worldview`
   (`:353-370`, the `:material-basis` on `:354`) claims "**ONE DECLARED HOME for the hegemonic
   tie-break**"; this pack would become a second home for the community surface **if DG-2 returns
   "publish"** (§2.2, §8a, D-NF+8), and `p0`-`p7`'s ternary discipline is the idiom the community
   simplex copies. Note the landed comment cites frozen `consciousness.py:177-192` for the argmax;
   the property actually spans `:167-191` with the `1e-6` epsilon at `:189` (measured 2026-08-18) —
   a pre-existing anchor drift in a landed pack, corrected by the same one-line amendment if that
   amendment fires.
6. `rust/crates/babylon-tick/content/rules/{solidarity,decomposition,control-ratio,production}.bsl`
   — the push idiom (D136), **the INSTITUTION-SUBJECT carrier idiom** (`decomposition.bsl:273-279`
   and `control-ratio.bsl:277-289` — `control-ratio.bsl:277` documents the SUBJECT-TYPE ANCHOR
   trick verbatim, and `:289` is its `(when #t)`; the `(select-max (nodes NodeType/INSTITUTION) 1)`
   sites at `decomposition.bsl:254,266,269,323` are a DIFFERENT idiom — carrier reads from inside
   SOCIAL_CLASS-subject rules — and are not what this pack's six carrier rules need), the
   reset-then-accumulate idiom (`production.bsl`'s `p0-production-total-reset`), and the
   bare-accessor fold-body law (D138, quoted at `control-ratio.bsl:277`).
6a. `rust/crates/babylon-tick/content/scenarios/carceral-arc-conformance.bscn:229-236` and
   `rust/crates/babylon-tick/tests/tick_goldens.rs:709-735` — the landed combined world seeds
   **exactly ONE** `NodeType/INSTITUTION` node (`carceral-register`), and the golden's assertion text
   pins that fact arithmetically ("five social classes + one carrier", `:716`; "c02:1 (the carrier)",
   `:730`). §3.7a's carrier verdict rests on these two files.
7. `ai/bsl-architecture-standard.md` §3.2 / §4.5 / §6.2 — no imposed functional forms, the fuel
   declare-bound+1 readback discipline, III.11 loud absence, the two-homes D-record convention.

---

## Global Constraints

Every task's requirements implicitly include this section.

- **Port-as-is (Director law, ADR183).** The frozen Python is the **structure and ordering
  contract, not a correctness oracle**. Transcribe exactly; every divergence earns a D-row.
  Defects transcribe verbatim (§1.6 lists three). **Never silently repair.**
- **Communities are NEVER graph nodes (Anti-Pattern VIII.9, INV-010, `topology.py:48-49`).** The
  port's community is a **hyperedge**, and this pack mints **no node type and no second carrier
  node** — it anchors its carrier-subject rules on the estate's SINGLETON `NodeType/INSTITUTION`
  carrier, which this train's own worlds name `community-register` and a co-loaded world seeds
  exactly once for every pack at the table (§3.7a, D-NF+23). Three mechanical traps make this more
  than a slogan and all three get permanent tests (§8c):
  1. `tick.rs::subject_type_of` (`:166-189`) derives a rule's subject NODE type from its `:field`
     binding namespace via `namespace_to_node_type` (`:161-163`, uppercase + `-`→`_`). **A
     `community/`-namespaced `:field` binding literally instructs the tick loop to iterate
     `NodeType/COMMUNITY` nodes.** No rule in this pack may take a `:field` binding in the
     community namespace; hyperedge fields are read through `(field-of <hyperedge-expr> …)` only.
  2. No membership may be encoded as a dyadic edge. `add-hyperedge`'s member list is sorted
     ascending (`structural_verbs.rs:1366`) and recorded WHOLE (`:1371-1373`); the seeding form
     Task 1 adds must preserve both.
  3. **A second INSTITUTION carrier node is the same failure one layer out.** `subject_type_of`
     makes EVERY INSTITUTION-subject rule iterate EVERY `NodeType/INSTITUTION` node in the world,
     and every such rule's `:field` binding must exist on every one of them or III.11 fires
     (`tick.rs:212-216`). Minting a second carrier therefore (a) double-applies this pack's
     hyperedge writes and (b) hard-fails the landed carceral packs in any co-loaded world. §3.7a
     rules the reuse; world 5c executes it.
- **The AG(i) attributed-membership ceremony is OUT OF SCOPE (Director act, #536 sequencing
  comment 2026-08-17; #564 row 22; #653 owns it).** No task may implement `update-membership`,
  `membership-field-of`, `deffield … :member`, `add-hyperedge` `<field-init>`, the
  `hypergraph-rs#2` accessor pair, or a membership-payload hash section. Task 11 writes the
  **co-sponsored design note** and nothing else. If a step appears to require a membership
  payload: it belongs to §5's blocked half — **STOP and record, do not improvise**.
- **Kinds are closed (Amendment AE (ii), re-sealed by AG (iii)).** Content may mint lattice-rung
  and adjunction **INSTANCES** only. This pack mints no adjunction, no rung, no verb, no severity
  rule, and no new element kind — the hyperedge own-field lane stores state on an element kind
  Amendment D already ratified as first-class.
- **No imposed functional forms (ADR173, ADR172 ruling 5, NORTH_STAR.md:26-28).** Nothing in this
  pack may stipulate a sigmoid, logistic, tanh, Gaussian, or threshold shape. The three shapes
  that look like candidates are all **measures**, and each is named as such where it lands: the
  normalized Shannon entropy (a measure of the simplex point, ADR213's named `log` consumer), the
  substrate floor (a **measured** excess-over-`p_bar` per ADR214 Ruling 1, not a stipulated
  minimum), and the argmax tie-break (a readout, transcribed from `consciousness.py:177-192`).
  `sigmoid` is additionally a prohibited BSL intrinsic name (`E-LOAD-024`); spelling a logistic out
  of `log`/`exp` is the same prohibited motion.
- **Every theory call not already ruled goes to §10's DIRECTOR GATE, popup-ready.** No task
  decides one. Specifically: the national-question pole shape and the `county_extraction`
  `BoundOpposition` registration (#664 assigns them here; they are ADR171-line items) are **not
  decided by this plan and not needed by its scope** — this pack mints no coupling.
- **The engine lane respects crate boundaries, and where it must cross one the crossing is a
  DECLARED task, never an improvisation** (§3.6). Storage and structural accessors live in
  `babylon-graph` (`GraphSubstrate` + both backends + `conformance` + `state_hash`); the language
  surface lives in `babylon-bsl` (`scenario`, `query`, `evaluator`, `structural_verbs`, `typecheck`,
  `bound_checker`); registration, the DRIVER and content live in `babylon-tick`; `babylon-graph`
  gains no BSL knowledge. **Revision 2 amends this constraint's earlier "`babylon-tick` gains no
  hyperedge logic beyond one registration string" cap, openly and for a named reason:** the ceiling
  supply chain (§3.2 rows 9-10) is driver logic — `babylon-tick/src/lib.rs:263-276` is the ONLY
  place `CardinalityCeilings` is constructed, and today it passes an empty `max_members` map, so
  every hyperedge-querying rule fails at LOAD. Task 4 owns that change; nothing else in
  `babylon-tick` outside `content/` and that one function changes. `hypergraph-rs` is touched by
  **nothing in this train** (its `MembershipEdge<M>` accessor gap, upstream issue #2, is #653's).
- **Section `0x06` is a Constitution III.7 substrate decision and lands with an empty-elision
  proof.** Every landed golden must stay byte-identical because every landed world has zero
  hyperedge attributes and `encode_state` elides an empty listing (the ADR198 R2 discipline,
  `state_hash.rs:250-256`: "the elision decision is the CALLER's"). If a pre-existing pin moves:
  **STOP** — the elision is wrong, not the pin.
- **The 16 pre-existing golden pins are byte-identical at landing. This train's OWN pins are
  expected to move, and moving them is a declared step, not a STOP.** `tick_goldens.rs` holds 18
  `#[test]` functions, 16 of them `*_hashes_are_pinned` (dossier 3 §4, verified 2026-08-17). Two
  obligations, never conflated: (1) the 16 stay byte-identical in every commit of every PR; (2) a
  pin this train adds is re-measured whenever a later rule changes its world, with the per-rule-id
  `fired` arithmetic explaining the delta recorded in the commit body. A pin that moves **without**
  a matching new rule id in the `fired` breakdown is the STOP condition — not motion itself.
- **Golden pins MEASURED, never derived.** Run `run_once` once against the committed content, read
  the printed hash back, paste it. Never hand-compute, never carry a hash forward by reasoning.
  Same law for every `report.fired` count, which gets an inline per-rule-id arithmetic breakdown in
  its assertion message.
- **Fuel is MEASURED, never guessed (declare-bound+1 readback).** For every rule: declare a
  deliberately low `:fuel N`, load, read the `E-LOAD-040: … static bound B exceeds its declared
  :fuel N` refusal, set `:fuel B+1`, confirm it clears load **and** runtime against **every**
  scenario that loads the rule. **The hyperedge cardinality axis in `bound_checker.rs` is ALREADY
  LANDED** (`ceiling_of_query` `:544-572`, `fuel.rs:112-118`, tests `:897-917`) — revision 2
  corrects revision 1's claim that Task 2 must build it. What is missing is the SUPPLY of the two
  ceiling maps (§3.2 rows 9-10), which **Task 4** lands; until Task 4, every hyperedge-querying rule
  fails at load with `MissingCeiling`/`E-LOAD-042 MissingMaxMembers` and no `:fuel` figure in
  Tasks 7-10 means anything. **`:max-members` is derived from the seeded population (D-NF+22), so a
  rule's static bound is per-WORLD**: the declared `:fuel` is the MAXIMUM over every world that
  loads the pack, re-measured after each new world lands, and a later, larger world that reds the
  load is the intended loud failure. Landed `:fuel` values span 1 → 4096; treat no range as an
  estimate.
- **Mutation evidence per rule commit:** break → a **named** test flips red → restore
  byte-identical (`git diff` clean), recorded in the commit body with the exact AST mutation. Every
  rule owes at least one vector; every clamp, guard, constant and dispatch arm owes one. A clamp
  whose fixture cannot make it bind is not exempt — it owes a **converse** vector plus a recorded
  reachability proof. The 14-arm floor dispatch owes a vector **per arm a world exercises** plus
  one proving the unexercised arms are unreachable in that world.
- **Every oracle exists or is created by a named task.** The mirrors are created by Task 7
  (`community_conformance.py`) and Task 10 (`community_decay_arc_conformance.py`); the frozen-engine
  corroboration artifact is created by Task 7 Step 6; the ADR214 values come from
  `ai/decisions/ADR214_national_incidence_artifact_train.yaml` (landed at this HEAD). **No task may
  cite an oracle no task creates.**
- **Frozen mirrors pasted verbatim + dated.** Each Rust conformance file's doc-comment header
  carries the plan path, the frozen source file + line count, the exact
  `PYTHONPATH="$PWD/src" uv run python <mirror>.py` command, its **full verbatim stdout**, the
  date it was captured, and the "why exact equality, no tolerance" paragraph citing
  `bsl-language.rst` §4.3 + ADR183. The mirror is a STANDALONE dependency-free script transcribing
  the RULES' binding order term-for-term over a literal `WORLD` dict — **the oracle, not the frozen
  engine** (the D146/ADR183 convention). §9 is its recipe.
- **No Python source changes, none.** The frozen engine is read-only reference. `mise run
  qa:regression` and `mise run qa:vault-regression-ci` are therefore byte-identical trivially — run
  them once anyway as proof (Task 12). **No file under `tests/baselines/**` may move**; if one
  does, STOP — that is a §6.5 ceremony, not a side effect. ADR214 Ruling 4's deferred ceremony is
  **checked** at Task 12 and fires only if a downstream golden actually moved (§6.3).
- **Declare only what this pack's own rules read.** No speculative `deffield`, no speculative
  `defconst`. In particular do **not** declare `community/visibility`,
  `community/rent-access-modifier`, `community/legal-status`, `community/infrastructure`, or any
  membership-shaped field — the first two have no reader anywhere (D-NF+9), the last three are
  §5's blocked half.
- **Vocabulary discipline.** `CommunityType`'s 14 members transcribe from
  `src/babylon/models/enums/community.py:38-55` in the landed order (`SETTLER, PATRIARCHAL,
  NEW_AFRIKAN, FIRST_NATIONS, CHICANO, WOMEN, TRANS, DISABLED, QUEER, UNDOCUMENTED, INCARCERATED,
  YOUTH, ADULT, ELDER`); `ConsciousnessTendency` from `enums/consciousness.py:83-85`
  (`LIBERAL, FASCIST, REVOLUTIONARY`). **`HyperedgeCategory` (`enums/community.py:75-78`) is NOT
  transcribed** — revision 2 removes it: no rule in §8 and no field in §2.4 reads it, so declaring
  it violates this plan's own declare-only-what-you-read law. Frozen publishes `category` on the
  hyperedge (`community.py:104`) and reads it nowhere in `step()`; that absence is D-NF+9's, not a
  vocabulary row. **Enum member order is hash-bearing (ADR195)** — never re-group. `defenum` is not
  shared across scenarios: every world re-declares, and the suite carries one ordinal-parity test
  per mint.
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`; `cargo clippy --workspace
  --all-targets -- -D warnings`; `cargo test --workspace`; `cargo clippy -p babylon-kernel
  --all-targets -- -D warnings -D clippy::pedantic` and same for `-p babylon-bsl`;
  `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`; `cargo test -p babylon-tick --test
  tick_goldens --locked`. `mise run rust:check` green after every task.
- **Machine safety — heavy runs are SINGLE-FLIGHT.** The box is a 12-core solo dev box. Run
  `mise run rust:check` / `cargo test --workspace` **one at a time, never fanned out across
  parallel agents**, and never concurrently with a Python `test:unit` leg. Each task below states
  its gate explicitly; a task's gate is a serial step, not a background job. Parallel agents in
  this train are for read-only investigation and doc work only.
- **After any `docs/reference/bsl-language.rst` edit:** `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run
  pytest tests/unit/reference/test_bsl_grammar_sync.py -q`. If a register probe reds because a new
  row cross-references an earlier D-code, repair the **test anchor** — never weaken an assertion.
- **Numbering is NEXT-FREE-AT-LANDING.** This plan writes **ADR-NF** and **D-NF+1 … D-NF+25**, never
  literals — **including when citing another train's rows**: the ImperialRent train's same-tick
  cross-rule row is written here as "the ImperialRent train's own D116 row (allocated next-free at
  its landing)", never as a literal (revision 2 removes the one "D197" literal revision 1 wrote).
  The measured tail on 2026-08-17 was **D180** (`bsl-language.rst:8158`) and **ADR214**
  (`ai/decisions/index.yaml`), but the ImperialRent train claims ten rows against it and #491 also
  allocates from the same tail. **Task 0 re-measures both tails and fixes this train's allocation;
  Task 12 re-measures again immediately before filing** and uses whatever is free then.
- **Branch from `dev` in an isolated worktree.** The worktree exists at
  `/media/user/data/worktrees/wt-community` on `feature/community-port-bsl` (PR A). Each later PR
  branches off **merged dev** — never stacked (#193). Conventional commits via `mise run commit`;
  merges only via `mise run pr:merge -- N`, after harvesting the Copilot review (ADR181).
- **Token economy:** subagents write artifacts to files and return ≤15-line summaries.

---

## 0. What this train is — two estates, one charter, and the one it does not own

| | estate | crates / files | owner | in this train? |
|---|---|---|---|---|
| **E1** | the hyperedge **REACH** lane — the type-scoped substrate enumerator, scenario seeding, the three slice-3 query heads + the `Element` Ord ruling, and the ceiling supply chain that makes a hyperedge-querying rule LOADABLE | `babylon-graph` (`substrate.rs`, `memory.rs`, `hypergraph_store.rs`, `conformance.rs`) + `babylon-bsl` (`scenario.rs`, `query.rs`, `evaluator.rs`, `typecheck.rs`, `score_class.rs`) + `babylon-tick` (`src/lib.rs`, the driver only) | **chartered by nobody before this plan** | **YES** — Tasks 1-4 |
| **E2** | the hyperedge **OWN-FIELD** lane — attribute storage + a sixth `CanonicalState` listing + hash section `0x06` + the BSL read/write surface | `babylon-graph` (`substrate.rs`, `memory.rs`, `hypergraph_store.rs`, `state_hash.rs`, `conformance.rs`) + `babylon-bsl` | **chartered by nobody** — `structural_verbs.rs:455-458` says so in its own refusal text: "hyperedge own-field storage is chartered by no Program 29 train" | **YES** — Tasks 5-6 |
| **E3** | the AG(i) **MEMBERSHIP-PAYLOAD** lane — `(member, hyperedge)` typed payload, `membership-field-of`, `update-membership`, a seventh hash section, `hypergraph-rs#2` | `hypergraph-rs` + `babylon-graph` + `babylon-bsl` | **#653** (Director act, 2026-08-17) | **NO** — hard sequenced dependency, §5 |
| **C** | the content pack `community.bsl` + its worlds, mirrors, goldens, records | `babylon-tick` (`content/`) | this charter | **YES** — Tasks 7-12 |
| **W** | the `community_memberships` population-weighted writer (#664's A2 seeder) | — | #664 assigns it here; it needs E3's weights | **NO** — §10 DG-6 proposes the re-home |

**This train's slice of #536's four charter riders**, stated against the issue body:
- Rider 1 (shape-verb collect-then-apply, unlocking all six `DEFERRED_SHAPE_VERBS`): **NOT taken.**
  §3.3 shows the port does not need it — communities are **seeded by the scenario**, never minted
  by a rule, exactly as nodes and edges are. The gate stays up; the placeholder-id design stays
  unowned. Recorded as D-NF+15's re-open trigger.
- Rider 2 (the membership heads against §2.12): **NOT taken** — #653's, per the Director act.
- Rider 3 ("BSL hyperedge lane live" as a NAMED deliverable with its own conformance vectors):
  **TAKEN, for the hyperedge half.** E1+E2 make hyperedges **enumerable**, seedable, queryable,
  loadable-against, readable, writable and hash-covered from content, with their own conformance
  vectors (Tasks 1-6) and the first content pack that uses them (Tasks 7-10). The **membership**
  half of the lane stays dark until
  #653; the plan says so in the deliverable's own name rather than implying completion.
- Rider 4 — **quoted in FULL**, because revision 1 satisfied its second clause and never mentioned
  its first:

  > 4. Query-side hyperedge heads stay with query-eval slices 2/3 (**CT4P #525's A5 Element-Ord pin
  >    precedes them**); the WS2 duality principle (#502 comment 2026-08-12: algebra closed under the
  >    dual) governs their eventual shape.

  **PARTIALLY TAKEN, and this is the one rider this plan renegotiates — on BOTH clauses, explicitly.**
  (a) *Clause 1, the A5 Element-Ord pin.* Serving `hyperedges`/`hyperedges-of` adds the
  `Element::Hyperedge(HyperedgeId)` variant `query.rs:17` says is "deliberately not" added yet, which
  puts a THIRD kind into a cross-kind ordering that `query.rs:56-63` carries a standing instruction
  about (register row D140, CT4P A5 / #525): the order is "pinned anyway, per this enum's own
  standing instruction, rather than left to whatever `#[derive(Ord)]` happens to produce from
  declaration order". The rider's "precedes them" is therefore honoured by **discharging the pin in
  this train, at the moment the variant lands**: Task 3 declares `Hyperedge` THIRD, rules
  `Node < Edge < Hyperedge` by declaration order in the enum's own doc, lands the companion test
  beside `node_sorts_before_edge_regardless_of_id`, amends `query.rs:17`'s "deliberately not added"
  paragraph, and takes **D-NF+24**. (b) *Clause 2, WS2 duality.* `members-of` and `hyperedges-of`
  land **together**, as duals, never one alone. (c) *What still does not land:* `metric-of` and the
  `the` head — §3.4 argues that split. Task 0 Step 5 records the whole renegotiation, both clauses,
  to #536 rather than letting it pass silently.

---

## 1. Frozen-source archaeology

All citations `file:line` against this worktree (HEAD `72a7e02b`).

### 1.1 Class surface (`community.py:309-323`)

`CommunitySystem(SystemBase)`: `partition = MATERIAL_BASE`, `position = 6.0`, `name = "community"`,
`creates_value = False` (spec-053 INV-001: does not mutate hex c+v+s). The third `step()`
parameter is `_context` (`:329`) and is **never read** — `TickContext` does not reach this system.
**Zero events are emitted**: a whole-file grep for `emit|EventType|event_bus` returns nothing
(contrast `solidarity.bsl`'s two emits). The pack therefore declares no `EventType` vocabulary and
its conformance suite asserts an **empty** sink (D-NF+18).

### 1.2 `step()`'s eight steps, in order (`community.py:325-370`)

```
1. community_states = _get_community_states_from_services(services)   :332
   -> EARLY RETURN if empty                                           :333-334
2. all_memberships, agent_memberships = _collect_memberships(graph)   :336
   -> EARLY RETURN if all_memberships empty                           :337-338
3. _compute_consciousness_from_orgs(...)                              :340-344
4. hypergraph = build_community_hypergraph(...)                       :346
5. _amplify_solidarity_edges(...)                                     :348-354   [E3-GATED]
6. _compute_threat_scores(...)                                        :355-360   [E3-GATED]
7. _compute_cost_modifiers(...)                                       :361-365
8. _apply_community_decay(...)                                        :366-370   [heat/cohesion/edu port; infrastructure E3-GATED]
```

Steps 1 and 2 are the two inactivity gates the frozen law tests L1/L2 pin
(`tests/unit/engine/laws/test_law_community_system.py:7-22`): with no config, or with config but
zero memberships, `step()` is a byte-exact no-op. **The port's analogue is different in kind and
must be recorded — and revision 2 re-derives it, because D-NF+22's derived ceilings changed the
answer:**

- **L1's analogue (no config) is a LOAD REFUSAL, not a runtime no-op.** A world with no
  `HyperedgeType/COMMUNITY` hyperedge gives the driver no ceiling for that type, so every
  carrier rule fails `MissingCeiling` (`E-LOAD-045`) at load. The driver's own comment rules this
  correct — "*A type the scenario declared zero of still gets no ceiling — and that is correct: a
  rule querying a population that does not exist should fail loudly at load rather than quietly
  iterate nothing*" (`babylon-tick/src/lib.rs:250-252`). The port therefore **refuses** where
  frozen silently no-ops, which is the estate's preferred direction. It is pinned as a **refusal
  test**, not a golden-pinned world (Task 10 Step 1).
- **L2's analogue (config, zero memberships) is representable and IS world 6** — but not as
  "communities with no members", which the substrate makes **unrepresentable**
  (`memory.rs:357-361`: "*hyperedge must have at least one member*"). The representable form is a
  community whose members are all **inactive** classes: `c01`'s `active` gate skips them, so
  `member-count` stays 0, `c04` pushes nothing, `density-sum` stays 0, the `c05`-`c08` skip gate
  preserves the seeded simplex, and `c09`/`c10` write nothing (the same `active` guard, §1.4). The
  tick is a byte-exact no-op on every community field **with all fourteen rules loaded and the
  carrier rules firing** — a stronger statement than "nothing ran".

**Step 4 is the reformulation this whole port turns on.** `build_community_hypergraph`
(`:57-110`) constructs a **fresh, ephemeral `xgi.Hypergraph`** every tick from the current
membership list, and it is never merged into `BabylonGraph`. In Rust the hypergraph **IS** the
substrate (`HypergraphStore`, Amendment D NATIVE HYPEREDGE) — so the rebuild collapses entirely:
the communities are seeded once by the scenario and persist. D-NF+1.

### 1.3 Step 3 — `_compute_consciousness_from_orgs` (`:373-462`) and its formula

1. Build `community → member agent set` from `agent_memberships` (`:392-397`).
2. Enumerate ORGANIZATION nodes (`:403`), reading `consciousness_tendency` (skip if `None`,
   `:405-407`), `cadre_level` (default 0.0, `:413`), `cohesion` (default 0.0, `:414`), and the org's
   member agents via `query_edges(source_id=org, edge_type=MEMBERSHIP)` → `edge.target_id`
   (`:418-419`). Orgs with **no** members are skipped (`:421`). Capped at `max_orgs = 500`
   (`:401,424-426`).
3. Per community with ≥1 agent (`:429-432`): `comm_size = len(agents_in_comm)`; per org,
   `overlap = |org_members ∩ agents_in_comm|`, skip if 0, `density = overlap / comm_size`
   (`:437-449`).
4. **Only if `org_landscape` is non-empty** (`:452`) — otherwise the community KEEPS its existing
   consciousness, a skip gate world 3 pins — call
   `compute_ternary_consciousness(community_type, org_landscape, substrate_floor)` with
   `substrate_floor = SUBSTRATE_FLOOR_DEFAULTS[comm_type].floor_value` (0.0 if absent, `:453-454`).

`formulas/consciousness.py:29-109`, transcribed term-for-term by §8's `c05`-`c06` (**anchors
re-measured 2026-08-18; revision 1's were off by 2-3 throughout**):

```
w_i        = density_i * cadre_i * cohesion_i          (:63)
total_dens = Σ density_i                               (:64-65)
r/l/f_raw  = Σ w_i over orgs of that tendency          (:67-72)
unorganized= max(0, 1 - total_dens)  -> added to l_raw (:78-79)
total      = r_raw + l_raw + f_raw                     (:82)
if total < 1e-10:  r=floor, l=1-floor, f=0             (:83-91)   [degenerate branch, assignment :89-91]
else:              r,l,f = raw/total                   (:93-95)
if r < floor:                                          (:98-107) [floor branch]
    remaining = 1 - floor;  lf = l + f                 (:99-100)
    if lf > 1e-10:  l *= remaining/lf ; f *= remaining/lf   (:101-103)
    else:           l = remaining     ; f = 0               (:104-106)
    r = floor                                               (:107)
```

**Linearity is what makes this portable.** `w_i` is linear in `density_i`, and
`density_i = Σ_{m ∈ org_i ∩ comm} 1/comm_size`. So the whole per-org aggregate decomposes exactly
**One frozen skip has no port analogue and is recorded as a divergence, not absorbed.**
`community.py:405-407` skips an ORGANIZATION whose `consciousness_tendency` is `None`.
`ConsciousnessTendency` has exactly three members (`enums/consciousness.py:83-85`) and a BSL
enum-typed field must carry one of them, so `c03-{r,l,f}`'s three rule-level guards **partition**
every org — the port cannot express a tendency-less org at all. That is a divergence by
inexpressibility, of exactly the kind D-NF+5 records for the 500-org cap; it takes **D-NF+25**, and
world 1's `n9` pins only the *no-member* skip (`:421`), which is a different gate.

Otherwise the aggregate decomposes exactly into a per-**class** push: each active class `m`
contributes, to each community it belongs to,
`(Σ_{orgs of m with tendency T} cadre·cohesion) / comm_size` to that tendency's raw accumulator and
`(count of m's orgs) / comm_size` to `total_dens`. §8's `c02`-`c04` compute the per-class org
weights from the ORG side (the D136 push idiom); `c05` divides by the census count. The only
divergence is **floating-point summation ORDER** (frozen sums per-org; the port sums per-class),
measured and pinned by world 1 — D-NF+3.

### 1.4 Step 7 — cost modifiers, and Step 8 — decay

`_compute_cost_modifiers` (`:611-621`, the write at `:621`) writes `community_cost_modifier` on
every agent **in `agent_memberships`** — including agents with zero memberships, which get exactly
`1.0` (`formulas/community.py:164-165`). The value is the **product** of
`reproduction_cost_modifier` across the agent's communities (`:166-174`, multiplied in the agent's
membership-list order). There is no `product` fold-op in BSL (`grammar.rs:672-683`:
`Sum, Mean, Min, Max, Count`), so the port uses reset + repeated `scale` (§3.5, D-NF+13).

**The `active` gate is part of this step's contract, and revision 2 ports it.** `agent_memberships`
is built by `_collect_memberships` (`:465-479`), whose loop **skips inactive nodes before the dict
is populated**:

```python
472    for node in graph.query_nodes(node_type=NodeType.SOCIAL_CLASS):
473        if not node.attributes.get("active", True):
474            continue
```

So an inactive `SOCIAL_CLASS` node is absent from `agent_memberships` and receives **no
`community_cost_modifier` write at all** — not `1.0`, *nothing*. Revision 1's `c09` wrote `1` on
**every** class, and `c10` would then have scaled that fabricated `1.0` by NEW_AFRIKAN's
`reproduction-cost-modifier` for world 1's inactive `n5`. **Revision 2 takes exact-port fidelity:
`c09` and `c10` both carry the rule-level `(when (= active 1))` guard `c01` already carries**, so
no divergence exists and no D-row is owed. The obligation this creates is an assertion that can
SEE the difference: world 1's `n5` is asserted **absent**, not `1.0` — `node_attribute(n5,
"social-class/community-cost-modifier")` must return the substrate's loud honest-null error (III.11,
`tick.rs:212-216`). **That assertion shape is a LANDED idiom, not a hope:**
`consciousness_ternary_conformance.rs:244-245` asserts
`graph.node_attribute(CLASS_UNPOSITIONED, field).is_err()` with the message "*unpositioned: {field}
must error absent (III.11), never default*", repeated at `:346-347`, `:654-655` and `:802-803`, over
that file's own header law at `:131-135` ("*Absence errors on read (III.11): an unwritten field is a
loud `GraphError`, never a default `0.0`*"). The mirror models the field as unwritten rather than
defaulted, and the mutation
vector "delete `c09`'s `active` guard" flips exactly that assertion red (Task 10 Step 1). The
frozen-side corroboration artifact (Task 7 Step 6) carries the independent half: it drives the real
`CommunitySystem.step()` over an inactive-member world and records that frozen writes nothing for
it — if it writes something, the archaeology is wrong and the task STOPs.

`_apply_community_decay` (`:624-675`), one loop, four writes per community (**decay anchors
re-measured 2026-08-18 — the `max(0, ·)` sites are one line earlier than revision 1 wrote**):

```
heat        = max(0, heat * (1 - heat_decay_alpha))                            :650,670
cohesion    = max(0, cohesion * (1 - cohesion_decay_alpha))                    :653,671
infrastructure = clamp01(infra*(1-infra_alpha) + min(core_count*maint,1)*infra_alpha)  :657-662  [E3-GATED]
education_pressure = max(0, edu * (1 - education_pressure_decay))              :665-666,673
```

`core_count` is the number of `MembershipRole.CORE_ORGANIZER` memberships per community
(`:640-646`) — **per-membership role**, hence E3-gated, hence the whole infrastructure line defers
(§5, DG-7).

### 1.5 Constants and defines this pack transcribes

| value | frozen site | lands as |
|---|---|---|
| `heat_decay_alpha = 0.05` | `config/defines/organizations.py:22-27` | `(defconst community/heat-decay-alpha 0.05c)` |
| `cohesion_decay_alpha = 0.03` | `organizations.py:28-33` | `(defconst community/cohesion-decay-alpha 0.03c)` |
| `education_pressure_decay = 0.1` | `config/defines/consciousness.py:138-143` | `(defconst community/education-pressure-decay 0.1c)` |
| `infrastructure_decay_alpha = 0.04`, `core_organizer_maintenance_factor = 0.1` | `organizations.py:34-39, 54-59` | **NOT declared** — §5's blocked half |
| `community_overlap_bonus = 0.1`, `rent_differential_penalty = 0.05` | `organizations.py:42-51` | **NOT declared** — they back `calculate_solidarity_potential`, a registered formula (`formula_registry.py:124`) `step()` **never calls** (D-NF+11) |
| the 5×5 class-pair solidarity matrix (15 upper-triangle values) | `config/defines/economy_class.py:249-283` | **NOT declared** — read only by `_amplify_solidarity_edges`, §5's blocked half |
| `SUBSTRATE_FLOOR_DEFAULTS`, 14 rows | `models/entities/consciousness.py:356-455` | the ADR214 `defconst` table, §6 |
| `ROLE_STRENGTH_WEIGHTS` (5), `LEGAL_STATUS_MULTIPLIERS` (5), `LEGAL_STATUS_ORDER` | `models/entities/community.py:25-49` | **NOT declared** — §5's blocked half |
| the TWO `1e-10` epsilons (degeneracy, `lf`-sum) and the `1e-6` argmax epsilon | `formulas/consciousness.py:83` and `:101`; `entities/consciousness.py:189` | inline literals, matching `consciousness.bsl:362`'s `eps` binding idiom (anchors re-measured 2026-08-18: `:97` is a `# Step 4` comment, and `consciousness.bsl:346` is a `wages-inbox` binding) |

### 1.6 The three frozen defects — transcribed verbatim, each with its (non-)mutation evidence

1. **The `max_orgs = 500` truncation appears TWICE with different semantics.**
   `community.py:401,424-426` breaks *after* appending, so it enumerates at most 500 orgs;
   `formulas/consciousness.py:61,74-75` checks `if idx >= max_orgs: break` *after* processing index
   500, i.e. 501 contributions. Both are insertion-order-dependent, hence non-deterministic under
   any re-ordering of the graph. **Not ported** (no early-break construct exists); the port's bound
   is the fuel ceiling, which refuses loudly instead of truncating silently. D-NF+5 carries the
   reachability proof (no world approaches 500) and states plainly that this is a divergence, not
   an equivalence.
2. **`_get_class_position_name` falls back to `"PROLETARIAT"` for an unknown role**
   (`community.py:520-524`) — a silent mis-classification, not an error. Belongs to §5's blocked
   half (only the amplification path calls it); recorded now so the later train does not "fix" it.
3. **`community_cost_modifier` is a write with no reader.** It is a declared `SocialClass` field
   (`models/entities/social_class.py:442-446`) that round-trips through `WorldState`, but no
   formula, system, or bridge multiplies by it anywhere in `src/`. Ported anyway (it is frozen's
   observable output); D-NF+17 names the re-open trigger.

Also recorded, not a defect: `threat_score` is a **transient graph-only** attribute dropped by
`WorldState.from_graph()` (`models/world_state.py:75-78`, pinned by
`tests/unit/models/test_graph_roundtrip.py:690-702`). It belongs to §5's blocked half.

### 1.7 Test estate (read, do not modify)

`tests/unit/engine/systems/test_community_system.py` (**540** lines, `wc -l` 2026-08-18);
`tests/unit/engine/laws/test_law_community_system.py` (**282** lines — the four Hypothesis laws L1-L4);
`tests/unit/formulas/test_community_formulas.py` (275); `tests/unit/models/test_community_models.py`
(743); `tests/property/invariants/test_community_membership_lint.py` (143 — the INV-010 linter, three
predicates). **None is modified.** L1-L4 are the behavioural laws Task 7 re-expresses as Rust
conformance rows (§8c), because a law that only exists in the frozen estate dies with it.
**What L4 actually pins, quoted, because §5's deferral argument cites it:** "*heat/cohesion never
increase per tick; infrastructure stays in [0, 1]*" (`:221`). The **non-monotonicity of
infrastructure is filed in the same file under "Caveats (not laws)"** (`:49-52`: "*Infrastructure is
NOT monotonically non-increasing like heat/cohesion — CORE_ORGANIZER maintenance can raise it
tick-over-tick. Only the [0, 1] clamp is pinned as a law for infrastructure.*"). See §5 / DG-7.

---

## 2. THE BOUNDARY — what this pack owns, what it may only read, what it produces

Audited in **both** directions (the ImperialRent train's own lesson: "does another pack already
WAIT on this?" is the question a boundary audit forgets).

### 2.1 PRODUCE-side seam — `solidarity_strength`, and why the port makes it QUIETER, not louder

`solidarity.bsl` **reads** `solidarity/strength` as the transmission coefficient
(`delta = strength * (source_r - target_r)`), citing frozen `formulas/solidarity.py:36`. The reads
are at **`:186, 192, 196, 201, 217, 225, 236, 242, 260, 263`** (and **twelve** more through
**`:401`** — 22 read lines in all, `grep -c` 2026-08-18; `grep -n update-edge` on that file returns
**nothing**, so the pack reads the attribute and never writes it) — re-measured 2026-08-18;
revision 1's `:116,171` anchors pointed at a comment block and the
`(rule solidarity/p0-transmit` header, not at a read. In the
frozen Python estate the only post-baseline writer of that edge attribute is
`CommunitySystem._amplify_solidarity_edges` (`community.py:570-576`). **This train does not port
that writer** (E3-gated), so `solidarity.bsl` keeps reading exactly the scenario-seeded strength it
reads today. Nothing moves; **no landed golden can move through this seam**, and the conformance
suite proves it with a combined `solidarity` + `community` world (world 5b, Task 10) asserting the
SOLIDARITY edge attribute is byte-identical after a community tick. **World 5b proves the
attribute seam and NOTHING ELSE, and the reason is a property of `solidarity.bsl`, not of any node
census:** that pack is ONE rule, `solidarity/p0-transmit` (`:170`), whose `:field` bindings are all
`social-class/…` (`:174-175`) — it declares **no INSTITUTION-subject rule at all**, so a carrier in
a solidarity world creates no cross-pack contention to detect. Detecting the collision needs a
landed pack that *does* run INSTITUTION-subject rules; that is world **5c**'s job, against
`control-ratio.bsl`. **The zero-INSTITUTION measurement belongs to the LANDED
`solidarity-conformance.bscn`** (`grep -c NodeType/INSTITUTION` → 0, measured 2026-08-18) — a fact
about *that file*, not about world 5b, which is a NEW world this train writes and which therefore
seeds one carrier and one community like every other world here (§3.7a, §8c guard 4). D-NF+12
records the seam and
its re-open trigger (#653 lands → the amplification rule lands → `solidarity.bsl`'s inputs move,
and *that* train re-measures solidarity's pins).

### 2.2 DUPLICATION seam — `consciousness.bsl`'s "ONE DECLARED HOME" claim

`consciousness/p8-dominant-worldview` (`consciousness.bsl:353-370`, its `:material-basis` on `:354`)
transcribes the argmax with the ruled tie order `LIBERAL > REVOLUTIONARY > FASCIST` at `1e-6`, and
claims "**ONE DECLARED HOME for the hegemonic tie-break** — the frozen estate smeared it across five
sites; here it lives exactly once." This pack's `c08-community-dominant-tendency` would transcribe
the **same** tie-break over the **community** simplex (a different subject, a different enum —
frozen's `ConsciousnessTendency`, not `WorldView`).

**Revision 2 corrects a governance inversion here.** Revision 1 recorded option (c) as "**adopted**"
— including an edit to another pack's declared `:material-basis` theory claim — while DG-2
simultaneously asked the Director whether the community readout should be published at all. A plan
may not both decide and ask; and amending a landed pack's theory claim is exactly the kind of edit
that belongs *behind* its gate. So:

- The three options are recorded as the plan's **analysis and RECOMMENDATION**, not its decision:
  (a) re-home into `consciousness.bsl` — **rejected on evidence**, it would make a class-surface pack
  iterate hyperedges and cross a subject-type boundary `subject_type_of` (`tick.rs:166-189`) does not
  admit; (b) drop the community readout — possible, it costs `c07`/`c08` and the frozen published
  attributes `consciousness_tendency`/`consciousness_contestation` (`community.py:106-107`);
  (c) **recommended** — transcribe a second time, with §8a row 1's copies-agree test and a one-line
  amendment to `consciousness.bsl:354`'s claim.
- **DG-2 decides between (b) and (c).** Task 9 Step 5 is **gated**: it runs only on a "publish"
  answer. If DG-2 returns "do not publish", `c07`, `c08`, §8a row 1, the `consciousness.bsl`
  amendment and D-NF+8 are all **void before any of them is written** — the pack ships 12 rules, the
  hyperedge carries only the `(r,l,f)` simplex, and readers argmax for themselves. Task 0 Step 1
  posts DG-2 to the docket so the answer is in hand before PR D opens; if it is not, Task 9 lands
  `c05`-`c06` and STOPs at `c07`.
- If (c) is taken, the same one-line amendment also corrects that comment's own frozen anchor:
  `consciousness.bsl:354` cites `models/entities/consciousness.py:177-192`, and the property is
  `:167-191` with the epsilon at `:189` (re-measured 2026-08-18).

### 2.3 FALSE FRIEND — `lifecycle.bsl`'s `community_tendency`

`lifecycle.bsl:4,78,104,372` mentions "community" and a field `community_tendency` — an **inbound
routing weight** for the lifecycle/consciousness seeding law, unrelated to anything
`CommunitySystem` reads or writes. The pack header must say so in its first paragraph so the next
reader is not trapped; no boundary work follows from it.

### 2.4 Boundary-rule table

| quantity | who owns it | this pack |
|---|---|---|
| `solidarity/strength` (edge) | scenario seed today; `CommunitySystem` amplification post-#653 | **does not write** |
| `social-class/revolutionary|liberal|fascist` (node) | `consciousness.bsl` (class surface, ADR204/W10) | **does not write** — the community simplex is a *different* subject |
| `community/revolutionary|liberal|fascist` (hyperedge) | **this pack**, first writer | writes |
| `social-class/community-cost-modifier` | **this pack**, first and only writer | writes |
| `social-class/org-r-weight|org-l-weight|org-f-weight|org-count` | **this pack**, per-tick accumulators | writes + resets |
| `social-class/threat-score` | §5's blocked half | **not declared** |
| MEMBERSHIP edges (org→class) | seeded by scenarios; read by this pack and `organization.bsl` | reads only |
| the community hyperedges themselves | seeded by scenarios (Task 1's form) | reads + writes attributes; **never mints, never removes** |

---

## 3. THE HYPEREDGE LANE — the sizing verdict

### 3.1 What exists at HEAD

- **`babylon-graph` substrate**: `HyperedgeId` as a type-level-distinct identity
  (`substrate.rs:35-41`); `add_hyperedge` `:235` / `remove_hyperedge` `:246` / `members_of` `:253` /
  `hyperedges_of` `:272` on `GraphSubstrate`, implemented by both `MemoryGraph`
  (`memory.rs:352-419`) and `HypergraphStore`. **Those four are the WHOLE hyperedge surface** — see
  §3.2 row 0 for what is missing beside them.
- **Canonical state**: section `TAG_HYPEREDGES = 0x04` hashes hyperedge identity, type and member
  list (`state_hash.rs:103,235-246`), reported through `CanonicalState::all_hyperedges` `:329`.
  **Hyperedge structure is already hash-covered.**
- **Grammar**: every hyperedge/membership production is specced and shape-checked —
  `bsl.ebnf:408-410,506-509,625`; `grammar.rs:202-212` (`ENUM_REF_POSITIONS`), `:529`
  (`MINTING_VERBS`), `:636-659` (the `ARITIES` table: `hyperedges` 1-2, `members-of` 2,
  `hyperedges-of` 2, `update-hyperedge` 3).
- **Executor**: `add-hyperedge` / `remove-hyperedge` execute as library calls, with member lists
  canonicalized ascending (`structural_verbs.rs:1366`) and recorded WHOLE (`:1371-1373`).
- **THE FUEL AXIS IS LANDED** (revision 1 got this backwards): `bound_checker::ceiling_of_query`
  (`:544-572`) already bounds `hyperedges`/`hyperedges-of` against `ceiling("HyperedgeType/X")` and
  `members-of` against `max_members("HyperedgeType/X")`, with `E-LOAD-042`
  `MissingMaxMembers`/`MemberListOverCeiling` (`:74-90`, messages `:144-157`) and landed tests
  (`:897-917`, `:995-1002`). `manifest.rs:11-15` rules `:max-members` **mandatory** on a
  `HyperedgeType` row. Nothing in `bound_checker.rs` needs building — see §3.2 rows 9-10 for what
  does.
- **`log`**: declarable and evaluable (`declarations.rs:125`, `intrinsic_host.rs:196`) — ADR213
  names Community's entropy as one of its two ready consumers.

### 3.2 What is absent — TWELVE rows, each with its evidence

Revision 2 adds rows **0, 9 and 10**; each is engine work revision 1 either declared absent when it
was landed, or declared landed when it was absent.

| # | gap | evidence | needed by |
|---|---|---|---|
| **0** | **`GraphSubstrate` has NO type-scoped hyperedge enumerator.** `nodes(node_type)` `:204` and `edges(edge_type)` `:208` are type-keyed ranges; there is no `hyperedges(hyperedge_type)`. The only whole-store hyperedge listing is `CanonicalState::all_hyperedges` (`state_hash.rs:329`) — a DIFFERENT trait whose own doc (`:294-299`) says the substrate trait "*offers only type-keyed ranges … no way to list which types exist*" and that listing the store "*is a storage capability a store must declare separately, on a trait about serialization rather than about the structural-verb surface*" | `grep -n 'fn ' substrate.rs` (17 methods, four hyperedge-side); `state_hash.rs:292-320` | `(hyperedges …)` — i.e. **six** of this pack's rules |
| 1 | **A `.bscn` cannot seed a hyperedge at all** | `scenario.rs:63-64` "No hyperedges yet"; the accepted top-form set is `defenum/defvocabulary/deffield/defconst/node/edge/edge-attr` (`:570-611`) | every world |
| 2 | A rule cannot mint one either | `DEFERRED_SHAPE_VERBS` + the unconditional LOAD gate (`structural_verbs.rs:1723-1776`, wired at `rule_pipeline.rs:269`) | not needed — §3.3 |
| 3 | `hyperedges`, `members-of`, `hyperedges-of` are UNSERVED (slice 3) in **TWO** tables: `evaluator::UNSERVED_EXPRESSION_HEADS` (`:544-551`) and `query::UNSERVED_QUERY_HEADS` (`:99-103`), the one `materialize()` actually consults; `evaluator::SERVED_QUERY_HEADS` (`:567`) must gain all three | `query.rs:91-98`'s own "kept in sync … the two tables answer different questions" | every rule |
| 3a | `Element` has no `Hyperedge` variant, and the enum carries a **standing cross-kind Ord instruction** (D140, CT4P A5 / #525) that a third kind must discharge | `query.rs:56-75`, `:17` ("deliberately not added") | Task 3, rider 4 clause 1 |
| 4 | a hyperedge has **no attributes at all** | `update-hyperedge` refusals (`structural_verbs.rs:452-466` execute path, `:873-879` collect path): "GraphSubstrate gives a hyperedge no attributes at all … Constitution III.7" | community state |
| 5 | `field-of` over a `HyperedgeRef` is refused | `evaluator.rs:1318-1322` | community state |
| 6 | no membership payload: `membership-field-of` unserved (slice 4), `update-membership` matched in **neither** dispatch table, `add-hyperedge` refuses `<field-init>`, `MembershipPayload` is empty/unhashed, upstream accessor issue `hypergraph-rs#2` OPEN | `evaluator.rs:544-551`; `structural_verbs.rs:406-469,800-883,1335-1343`; `hypergraph_store.rs:53-67` | §5's blocked half — **#653** |
| 7 | no membership-payload hash section (layout stops at `0x05`) | `state_hash.rs:100-104` | **#653** |
| 8 | `typecheck.rs` carries **zero** hyperedge-aware logic (`rg -ni hyperedge` → no matches in a 974-line file) | dossier 3 §3, re-verified 2026-08-18 | E1 + E2 |
| **9** | **`LoadedScenario` has no `hyperedge_types` count map** — it carries `node_types` (`:242-248`) and `edge_types` (`:249-261`) and nothing hyperedge-side, so the driver has no population to build a ceiling from | `scenario.rs:234-270` | the ceiling for `HyperedgeType/COMMUNITY` |
| **10** | **The driver passes an EMPTY `max_members` map and mints no `HyperedgeType/*` ceiling** — `CardinalityCeilings::new(<NodeType+EdgeType counts>, HashMap::new())` | `babylon-tick/src/lib.rs:263-276`; `fuel.rs:95` | **every rule in the pack fails at LOAD without it** |

### 3.3 The verdict

**The hyperedge lane is ABSENT at every content-reachable layer, ABSENT in the substrate's own
enumeration surface, INERT at the driver, and PARTIAL at storage.** A community cannot today be
seeded, enumerated, named, queried, read, written, or even LOADED against from BSL content. **There
is no partial content slice of this port that lands without engine work** — unlike Territory or
Production, Community's subject does not exist in the language.

**The sizing verdict, restated honestly (revision 2).** Revision 1 claimed "*this train is majority
ENGINE work, and its content pack is the smaller half*" while its own per-task hours made engine 41%.
Revision 2's re-scoped engine lane (Tasks 1-6) is **~34h of ~69h — 49%**, against ~32h of content
and governance (Tasks 7-12) and 3h of Task 0. The honest statement is therefore: **the engine lane
is HALF this train, not a prelude to it; anyone estimating this as "another content port like
Solidarity" is wrong by roughly a factor of two.** The arithmetic is in §Estimate and no sentence in
this plan claims a majority the hours do not show.

Three consequences the plan takes rather than papers over:
1. **E1 and E2 land IN-TRAIN** (Tasks 1-6, PRs A and B), because rows 0-5, 8, 9 and 10 are chartered
   by nobody — `structural_verbs.rs:455-458` says so in its own refusal text, and ADR198 R4 fenced
   the membership half off to #536/#653 while saying nothing about hyperedge own-fields.
2. **The boundary crossings are DECLARED, not improvised.** Row 0 is `babylon-graph` work and gets
   its own task (Task 2) with its own `conformance.rs` row; row 10 is `babylon-tick` DRIVER work and
   gets its own task (Task 4), which amends §3.6's "one registration string" cap openly. §3.6's own
   rule — "a task that needs to cross one of these boundaries is a re-plan" — is honoured by
   re-planning, which is what revision 2 is.
3. **E3 is a HARD SEQUENCED DEPENDENCY on #653**, not an assumption. §5 lists exactly what waits on
   it, and no task in this plan touches it.

**Row 2 is deliberately NOT lifted.** The port does not need to mint hyperedges: frozen's per-tick
rebuild collapses into a seeded substrate (§1.2, D-NF+1), so `DEFERRED_SHAPE_VERBS` stays intact
and #536's rider 1 stays unspent. This is the single largest scope saving in the plan — it removes
the placeholder-id design (an unowned, unspecified piece of work the gate's own text escalates)
from the critical path.

### 3.4 Which query heads land, and why not all four

`hyperedges` (type-scoped, arity 1-2), `members-of` (a hyperedge → its member nodes), and
`hyperedges-of` (a node + a type → its hyperedges) are the port's minimum: `c00`/`c05`/`c06`/`c07`/
`c08`/`c11` iterate communities from the carrier, `c01`/`c04`/`c10` iterate a class's communities,
and the census counts members. **`metric-of` does not land** (no rule needs a hypergraph metric; an
unused form is an untested form — `scenario.rs:64`'s own doctrine) and **the `the` head does not
land** (slice 2, unrelated). Rider 4 of #536 parks all four with query-eval slices 2/3; this plan
takes three of them early, discharges the rider's A5 Element-Ord precondition in the same task that
mints the variant (§0 rider 4(a), Task 3, D-NF+24), and records the whole renegotiation at Task 0
Step 5 rather than letting it pass as a side effect. The WS2 duality principle (#502, algebra closed
under the dual) is respected: `members-of` and `hyperedges-of` land **together**, as duals, never one
alone. **`hyperedges` cannot be served from the language alone** — it needs §3.2 row 0's substrate
enumerator first, which is why Task 2 precedes Task 3.

### 3.5 Two language shapes the content needs that no landed pack uses

1. **A `for-each` over a hyperedge query, with `update-hyperedge` as the body's effect.** Landed
   `for-each` bodies write nodes and edges; nothing writes a hyperedge, because nothing could.
   Task 7's SPIKE proves it before fourteen rules depend on it.
2. **Repeated `scale` accumulation** for the cost-modifier product (no `product` fold-op exists).
   The reset-then-accumulate split across two rules (`c09`/`c10`) is the landed
   `production.bsl::p0-production-total-reset` idiom, and it relies on same-tick cross-rule
   apply-in-place — the **D116 recorded gap** that `decomposition.bsl` and the ImperialRent train
   also rely on (that train's own D116 row, allocated next-free at its landing — never cited here as
   a literal). §8b enumerates every such read in this pack.

### 3.6 Crate boundaries (binding, as AMENDED by revision 2)

```
hypergraph-rs   : UNTOUCHED. (Its MembershipEdge<M> accessor gap, issue #2, is #653's.)
babylon-graph   : the STRUCTURAL and STORAGE surface — the type-scoped hyperedge enumerator
                  (Task 2), hyperedge ATTRIBUTE storage, the sixth CanonicalState listing,
                  state_hash section 0x06 + the caller-side elision, and a conformance.rs row
                  per new accessor. No BSL knowledge.
babylon-bsl     : the language surface — scenario seeding forms + LoadedScenario.hyperedge_types,
                  query materialization, Element::Hyperedge + its Ord ruling, served/unserved head
                  tables (BOTH of them), field-of/update-hyperedge dispatch, typecheck.
                  No storage layout decisions. (bound_checker needs NO change — §3.1.)
babylon-tick    : (a) ONE registration string ("community"); (b) THE DRIVER's ceiling
                  construction in src/lib.rs:263-276 — the HyperedgeType ceiling map and the
                  max_members map, Task 4, the one amendment revision 2 makes to this table and
                  the reason it makes it; (c) all content, worlds, mirrors, goldens.
                  tick.rs is NOT here — it lives in babylon-bsl (crates/babylon-bsl/src/tick.rs).
```

A task that needs to cross one of these boundaries is a re-plan, not an improvisation. **Revision 2
IS that re-plan** for rows 0, 9 and 10 of §3.2: each crossing is a named task with its own gate,
rather than smuggled into a `babylon-bsl`-scoped step.

### 3.7 Rejected alternatives (recorded so a reviewer sees they were considered)

| alternative | why rejected |
|---|---|
| **A per-community carrier NODE** (14 INSTITUTION nodes holding community state) | It is "community as node" with the label filed off — the exact thing `topology.py:48-49`, INV-010 and VIII.9 forbid, and it still cannot answer "which communities does this class belong to" without hyperedge membership. It saves only E2, at the cost of the theory line. |
| **One carrier node with 14 × N fields** | Same objection in miniature, plus a 112-field roster and a per-community dispatch on every read. |
| **Levi/incidence encoding in content** | Amendment D's sub-ruling D-1 confines the Levi construction to **internal storage**; expressing it in content re-exposes exactly what AG(i) rejected. |
| **Pairwise MEMBERSHIP-edge fan-out** | Anti-Pattern VIII.9 verbatim; INV-010's linter would red. A hard port-time regression, not a style choice. |
| **`HyperedgeType` as identity (14 members, one per CommunityType)** | `(hyperedges …)` is type-scoped (`grammar.rs:202-212`), so every uniform law (census, decay, normalization) would need 14 near-identical copies. Adopted instead: **one** `HyperedgeType/COMMUNITY` member + a `community/kind` enum field. D-NF+16. |
| **Seeding the substrate floor as a per-hyperedge literal** | It would put a ruled political value in fixtures instead of the `defconst` table ADR214 Ruling 4 ruled. Adopted instead: the 14-row table + a kind dispatch (§6.2). |
| **A SECOND INSTITUTION carrier node** (`community-register` beside the landed `carceral-register`) | §3.7a — rejected on landed evidence; this was revision 1's unstated assumption and it breaks the estate in exactly the world Checkpoint A builds. |
| **Serving `(hyperedges …)` through `CanonicalState::all_hyperedges`** instead of a substrate enumerator | `state_hash.rs:294-299` rules that trait's listings a *serialization* capability deliberately kept off "the structural-verb surface Amendment D ratified"; routing a query head through it would make `query::materialize` depend on the encoder's trait and would return the whole store untyped. Adopted instead: a type-scoped `GraphSubstrate::hyperedges`, symmetric with `nodes`/`edges`. D-NF+21. |
| **Declaring `:max-members` via a content `manifest` form** | No landed content declares a manifest at all (`grep -rn '(manifest' content/` → zero), so this would make Community the first content set to carry one AND would put an invented cap in fixtures. Adopted instead: derive both ceilings from the seeded population, symmetric with `node_types`/`edge_types`. D-NF+22; re-open trigger: the first content set that genuinely needs a cap larger than any world it ships with. |

### 3.7a THE CARRIER VERDICT — reuse the estate's singleton, mint nothing (C3)

**Decision: this pack mints NO carrier node.** Its six carrier-subject rules anchor on the ONE
`NodeType/INSTITUTION` node a world already has; this train's own worlds name that node
`community-register` because they load no other carrier-bearing pack, and any world that co-loads
one seeds **exactly one** INSTITUTION node carrying every pack's anchor fields.

**Why, from landed evidence, not preference:**

1. `subject_type_of` derives a rule's subject from its `:field` namespace and **errors when a rule
   declares none** (`tick.rs:166-189`), so every INSTITUTION-subject rule iterates **every**
   `NodeType/INSTITUTION` node in the world. Two carriers ⇒ this pack's `c00`/`c05`-`c08`/`c11` each
   fire twice, and each firing runs a `for-each` over **all** communities — so `c11`'s decay is
   applied twice per tick and `c05`'s normalization runs against its own output. That is not a
   double-counted `fired` figure; it is a wrong world.
2. An unwritten field on a bound subject is a hard error (`tick.rs:212-216`, "*the substrate's loud
   error, because III.11 says absence is not zero*"). So the FOREIGN pack breaks too: the first
   `institution/decomposition-fire-tick` bind on `community-register` is a III.11 tick failure.
3. The landed estate already assumes the singleton and says so arithmetically:
   `carceral-arc-conformance.bscn:229` seeds one `carceral-register`, and
   `tick_goldens.rs:716` / `:730` pin "*five social classes + one carrier*" and "*c02:1 (the
   carrier)*". A second carrier moves those two landed pins — a STOP condition by this plan's own
   golden discipline.
4. The anchor mechanism is the landed one, cited exactly: `control-ratio.bsl:277` documents the
   "*SUBJECT-TYPE ANCHOR ONLY*" binding, `:287` is its `(when #t)` and the rule spans `:276-291`
   (re-measured 2026-08-18 — the earlier `:289` pointed at the first `update-node`);
   `decomposition.bsl:273-279` is the sibling. This pack's anchor is
   `institution/community-carrier`, seeded `1`, bound and never read again — with the same
   one-sentence disclosure in each rule's `:material-basis` so no reader mistakes it for a gate.

**What this obliges (all landed, none deferred):**

- **EVERY world this train writes seeds exactly ONE `NodeType/INSTITUTION` node** — worlds 1, 2, 3,
  4, 5, **5b**, 5c and 6 alike. There is no "carrier-free" world in this pack's estate: six of its
  rules are carrier-subject, so a world without a carrier runs the community half **inert** and
  reds §8c guard 4. This is stated here because it is the one place the C3 ruling could be misread:
  the "zero INSTITUTION nodes" measurement in §2.1 is about the **landed**
  `solidarity-conformance.bscn`, not about world 5b, which this train writes fresh.
- **World 5b** (`community-solidarity-seam-conformance.bscn`, Task 10) is that landed world's shape
  **plus one carrier and one community hyperedge** — the carrier so the six carrier rules are LIVE
  (an inert community half would make the seam proof vacuous: "nothing moved" is worthless if
  nothing ran), and the hyperedge because `(hyperedges HyperedgeType/COMMUNITY)` needs a declared
  population to bound against (§3.2 row 10 / D-NF+22: a type the scenario declared zero of gets no
  ceiling, and the driver's own comment calls that correct — "*a rule querying a population that
  does not exist should fail loudly at load rather than quietly iterate nothing*",
  `babylon-tick/src/lib.rs:250-252`). Adding a carrier there is safe precisely because
  `solidarity.bsl` has no INSTITUTION-subject rule to contend with it (§2.1).
- **World 5c** (`community-carrier-collision-conformance.bscn`, Task 10) co-loads `community.bsl`
  **with `control-ratio.bsl`** — a pack that DOES run INSTITUTION-subject rules — over ONE carrier
  node carrying both packs' anchor fields, and pins the **per-rule-id `fired` arithmetic** in the
  landed `carceral_arc` style. This is the world that can detect the collision; world 5b cannot,
  and revision 1 named only world 5b.
- **A permanent guard** (§8c row 4): `exactly_one_institution_carrier` — every world this pack
  loads into has exactly one `NodeType/INSTITUTION` node, failing with the reason quoted from
  `tick.rs:166-189`.
- **D-NF+23** records the invariant and its re-open trigger: the first design that genuinely needs
  two carriers needs a subject SELECTOR the language does not have (a rule cannot say "the
  INSTITUTION node with field X"), so that design is an escalation, not a fixture change.

---

## 4. E2's storage design — the sixth section, mirroring T3 exactly

`babylon-graph` gains, in the shape ADR198 R1 / ADR203 landed for dyadic edges:

- `GraphSubstrate::set_hyperedge_attribute(HyperedgeId, &str, f64) -> Result<(), GraphError>` and
  `hyperedge_attribute(HyperedgeId, &str) -> Result<f64, GraphError>` (loud on absence — III.11,
  matching `node_attribute`'s honest-null shape at `substrate.rs:184`), implemented by `MemoryGraph`
  and `HypergraphStore`. **Storage shape: `HashMap<(HyperedgeId, String), f64>`** — T3's
  `edge_attributes` shape *exactly* (`hypergraph_store.rs:85`:
  `HashMap<(String, NodeId, NodeId, String), f64>`), one key component narrower. **Revision 2
  removes revision 1's "a `BTreeMap` … sorted by construction"**, which contradicted the same
  paragraph's own "mirror T3 exactly" instruction and duplicated a guarantee the encoder already
  owns: **the sort belongs to `encode_state`** (`state_hash.rs:385-395` for the `0x05` sort), and
  one datum must not have two sort authorities.
- `CanonicalState` gains a **SIXTH REQUIRED listing** — `all_hyperedge_attributes(&self) ->
  Vec<(HyperedgeId, String, f64)>` — required, never defaulted, on the trait's own stated argument
  for the fifth (`state_hash.rs:314-319`: "*a default-empty `all_edge_attributes` would let a store
  silently forget to report edge attributes … A required method makes every implementor answer the
  question out loud, at compile time*"). Both stores implement it; the trait's "five-way listing"
  doc (`:292`) becomes six-way in the same commit.
- `TAG_HYPEREDGE_ATTRIBUTES = 0x06`, written by
  `write_hyperedge_attributes(&[(HyperedgeId, String, f64)])` sorted ascending by `(id, qname)`,
  with **the elision decision in `encode_state`** — an empty sixth listing contributes **zero
  bytes** (`state_hash.rs:250-256`'s own note is the contract to copy). Layout version bumps per
  that module's "Layout versions" doc.
- **A `conformance.rs` row per new accessor** (I13): `run_substrate_conformance` (`:28-45`) is the
  both-backends suite every landed `GraphSubstrate` accessor has a row in — the two attribute
  accessors here, and Task 2's enumerator, each land one. A `GraphSubstrate` method with no
  conformance row is a method only one backend is proved to hold.
- **The empty-elision proof is the gate**: all 16 landed pins byte-identical, because every landed
  world has zero hyperedges and therefore zero hyperedge attributes. This is the same proof T3
  shipped and it is what makes a III.7 widening cheap.

`babylon-bsl` gains: `update-hyperedge` executing in **both** dispatch sites (`execute_item`
`:452-466` and `collect_items` `:873-879` — the M4 mutation-gap lesson at
`structural_verbs.rs:539-545` is why both, and why each owes its own vector); `field-of` serving a
`HyperedgeRef` referent (replacing the refusal at
`evaluator.rs:1318-1322` — and the refusal's *own* text must be updated, since it currently asserts
a hyperedge "carries no attributes of its own"); a `deffield` lane for hyperedge-subject fields
whose namespace `subject_type_of` never sees; `(hyperedge-attr <hyperedge-name> <qname> <literal>)`
scenario seeding mirroring `(edge-attr …)`; and the typecheck coverage row 8 names.

**Save-compat:** section `0x06` changes the canonical state layout version. `docs/versioning.md`'s
policy applies; Task 12 records it in ADR-NF as a declared III.7 decision with the elision proof
attached. **DG-8 asks the Director whether the ADR path suffices here** (as ADR198 R1 did for
edges) or whether hyperedge own-fields want their own amendment (as membership payloads did in AG).

---

## 5. What does NOT land — the E3-gated half, as a hard sequenced dependency

| frozen phase | frozen site | what it needs from AG(i) | blocked on |
|---|---|---|---|
| `_compute_threat_scores` | `community.py:579-608` + `formulas/community.py:52-74` | per-membership `role` → `ROLE_STRENGTH_WEIGHTS`, per-membership `effective_visibility` (`entities/community.py:407-417`), plus community `legal_status` → `LEGAL_STATUS_MULTIPLIERS` | **#653** |
| `_amplify_solidarity_edges` | `community.py:527-576` + `formulas/community.py:111-141` | per-membership `strength` on **both** endpoints; also community `infrastructure`, and the 5×5 class-pair matrix | **#653** |
| infrastructure decay's maintenance term | `community.py:655-661` + `formulas/community.py:77-108` | the per-community CORE_ORGANIZER **count** = per-membership `role` | **#653** |

**The whole infrastructure line defers, not half of it.** Landing `infra * (1 - alpha)` without the
maintenance term is not a partial port — it is a **different law**: monotone decay, where frozen's
is non-monotone because `calculate_infrastructure_decay` (`formulas/community.py:77-108`) adds
`min(core_count·maintenance, 1)·alpha` and CORE_ORGANIZER maintenance can raise infrastructure
tick-over-tick. **Port-as-is (ADR183) forbids it, and that argument stands alone.** Revision 2
corrects the evidence revision 1 handed the Director: **frozen law L4 does NOT pin the
non-monotonicity** — L4 pins "*heat/cohesion never increase per tick; infrastructure stays in
[0, 1]*" (`test_law_community_system.py:221`), and the file's own header files the non-monotonicity
under "**Caveats (not laws)**" (`:49-52`), i.e. the exact opposite of "explicitly pins". DG-7 puts
the call to the Director anyway, because "defer the whole law" costs Checkpoint A a phase — but it
now goes with a citation that survives being opened.

**Consequence for Checkpoint A (ADR208 R14), stated explicitly rather than implied:** with this
train landed, Community @6.0 is **partially ported** — four of six phases. Whether that counts
toward "ALL 13 Material Base systems ported" is **DG-1**, a Director call, not this plan's to make.
WS3 stays HELD either way.

**Not this train's scope, and not smuggled in:** the `community_memberships` population-weighted
writer (#664 — needs E3's weights; DG-6 proposes the re-home), the `county_extraction`
`BoundOpposition` registration and the pole-shape choice (ADR171-line items; DG-5), the four
repression helpers `legal_status_escalate`/`designate_community`/`infiltrate_community`/
`disrupt_infrastructure` (`community.py:210-279` — never called from `step()`; they await a verb
layer, D-NF+11), and `community_overlap_matrix`/`communities_spanning_axis`/`shared_communities` as
standalone helpers (`:113-207` — only `shared_communities` is on `step()`'s path, via the
amplification phase).

---

## 6. The floor table — consuming ADR214's T7 rulings exactly

### 6.1 The values (ADR214 Ruling 1 + erratum 9, verbatim in provenance comments)

Shape **F-B**: `floor = q_pole - p_bar`, the pole's excess over the settler reference rate.

| CommunityType | frozen value | **this pack's value** | provenance |
|---|---|---|---|
| NEW_AFRIKAN | 0.12 | **0.136** | measured, `unrestricted_3218`; full precision `0.232693906 - 0.096223732 = 0.136470174`; 3dp `0.136` (erratum 9 — the T7-posted 0.137 was a double-rounding artifact) |
| FIRST_NATIONS | 0.12 | **0.155** | measured, same variant |
| CHICANO | 0.08 | **0.113** | measured, same variant |
| SETTLER | 0.0 | **0.0** | identically zero by construction (the settler pole IS the norm); preserves the ratified hegemonic-default claim (`consciousness.py:420-426`) |
| PATRIARCHAL, YOUTH, ADULT | 0.0 | 0.0 (unchanged) | structural (hegemonic default / lifecycle) |
| INCARCERATED | 0.18 | 0.18 (unchanged) | Ruling 2, **with its problem named in the comment**: unreachable in principle from `B17001` (the universe excludes the institutionalized); a future ruling needs BJS/Vera-class data |
| WOMEN 0.04, TRANS 0.06, DISABLED 0.03, QUEER 0.04, UNDOCUMENTED 0.10 | unchanged | unchanged, **confidence demoted** per Ruling 2 (their only cited provenance is the literal string `"estimated"`) | comment records the demotion |
| ELDER | 0.02 | 0.02 (unchanged) | estimated (generational memory) |

**Ruling 3 is load-bearing for the fixtures:** FIRST_NATIONS (0.155) > NEW_AFRIKAN (0.136) breaks
the frozen table's exact tie for the first time — an emergent output of the chosen measure, not a
fresh stipulation. World 2 seeds both so the ordering is **executed**, not asserted.

### 6.2 Entry path (ADR214 Ruling 4)

A **14-row `defconst` table** with ADR-cited provenance, re-declared per scenario (the
"`defenum` is not shared across scenarios" precedent), plus **ONE** 14-arm `if`-chain dispatch on
`community/kind`, in `c06` — because no map or lookup construct exists in the language. **The
precedent claim is weakened to what is actually landed** (revision 1 overstated it):
`territory.bsl:130-137` is a **three**-arm nested `if` inside a `select-max` score, not a 14-arm
dispatch in a binding, so the shape is *plausible by extension*, not *demonstrated* — Task 7 Step 5's
spike arm (e) is the real evidence and it is STOP-gated. A **cross-world parity test** pins every
world's 14 constants equal to each other and to the ADR's values, so a typo in one world cannot pass.
D-NF+6. **DG-3** asks whether all 14 rows are declared in a world that seeds three communities
(a complete ruled table) or only the rows a world reads (declare-what-you-read) — the plan's
recommendation is **all 14, with the parity test**, because a partial transcription of a ruled table
is how ruled values drift.

**Exactly ONE dispatch exists, and that is a decision (I5).** Revision 1 instructed `c05` to
transcribe the floor inside its degenerate branch ("*do not route it through `c06`*"), which would
have minted a SECOND undeclared 14-arm dispatch — absent from §8a's duplication ledger, from the
fuel plan and from the mutation budget. Revision 2 routes it, because **routing is bit-identical**:
`c05`'s degenerate branch yields `(r, l, f) = (0, 1, 0)`, so `c06` computes `lf = 1.0`,
`l = 1.0 · (1 − floor) / 1.0`, `f = 0`, `r = floor` — and `×1.0` and `÷1.0` are exact in IEEE-754,
so the result is frozen's `r_norm = substrate_floor; l_norm = 1.0 − substrate_floor`
(`formulas/consciousness.py:89-91` — revision 1 cited `:87-89`, which is the log call) to the bit.
The floor-branch guard `r < floor` is false when `floor = 0`, so the SETTLER control is unaffected
either way. `c05` therefore carries no floor and no `community/kind` read at all.

### 6.3 The deferred §6.5 ceremony

ADR214 Ruling 4 defers the ceremony and fires it "only if a downstream golden actually moves at that
later landing." **This train changes no Python source**, so `tests/baselines/**` cannot move and
`qa:regression` / `qa:vault-regression-ci` are byte-identical trivially. Task 12 Step 4 **runs both
anyway and records the result**; if nothing moved, ADR-NF records the ceremony as **NOT FIRED, with
the evidence**, discharging the deferral honestly rather than by silence.

**Ruling 4's stated TRIGGER was superseded, not met — and ADR-NF must say so** (M11). Ruling 4 makes
the `defconst` landing conditional on the Community port's "*`community_memberships`
node-local-list-of-structs blocker (#536, port-estate-survey row 6.0, filed BLOCKED)*" **clearing**
(`ADR214_national_incidence_artifact_train.yaml:222-231`). This train does not clear that blocker —
it **sidesteps** it, by porting unattributed memberships as seeded hyperedges (D-NF+19) and leaving
the list-of-structs shape to #653. The landing is still sanctioned: the Director's #536 sequencing
comment (2026-08-17) pulls AG(i) out to #653 and re-sequences the floor-table rework behind #334
alone, which is landed at this HEAD. Task 12's ADR-NF records the trigger as **superseded by
Director act**, with both citations, rather than citing Ruling 4 as though its precondition had been
satisfied.

---

## 7. Transcendentals and functional forms — the verdict

**Two `log` calls, both measures, both required.** `_shannon_entropy_normalized`
(`entities/consciousness.py:274-291`, the guard at `:288`, the divisor at `:291`) computes
`-Σ p·log(p)` over `(r, l, f)` with a `p > 1e-10`
guard, divided by `log(3)`. The pack declares `(intrinsic log :params (real) :returns real :cost 40)`
— already in `DECLARABLE_INTRINSICS` (`declarations.rs:125`), evaluated through the ADR213 libm
0.2.16 dispatch with per-intrinsic golden vectors, and named by ADR213 as one of `log`'s two ready
consumers. `log(3)` is computed with the same intrinsic rather than pasted as a literal, so the
division's bytes match the frozen expression exactly; the cost shows up in the fuel measurement.

**No `exp`.** Nothing in this system exponentiates. **No stipulated shape anywhere**: the floor is
measured (§6), the entropy is a measure of a state, the argmax is a readout, and the alpha-smoothed
decays are frozen coefficients transcribed from `defines.yaml`, not curves fitted to anything. If a
later step appears to want a curve — STOP; that is ADR173 territory.

---

## 8. Rule layout — **14 rules** across 12 table rows, one pack (`community.bsl`, namespace `community/`)

**The count is 14, not 11** (revision 2 corrects it everywhere): twelve rows below, one of which
(`c03`) is three rules ⇒ 12 − 1 + 3 = **14 `(rule …)` forms**, exactly the fourteen ids §9's mirror
order enumerates (`c00, c01, c02, c03r, c03l, c03f, c04, c05, c06, c07, c08, c09, c10, c11`). Every
downstream figure derives from 14: the mutation budget ("every rule owes at least one vector"), the
per-rule fuel measurements (Task 8 measures **seven** rules for `c00`-`c04`, not six), and every
`fired` arithmetic breakdown. **If DG-2 returns "do not publish", the pack is 12 rules** (`c07` and
`c08` never written) and every count re-derives from 12.

Subject types are derived by `babylon-bsl`'s `tick.rs::subject_type_of` (`:166-189`) from each rule's
`:field` binding namespace, so the namespace column below is load-bearing, not cosmetic. All six
`institution/`-subject rules bind the **shared singleton carrier's** `institution/community-carrier`
anchor (§3.7a), never a second node of their own.

| id | subject (`:field` ns) | what it does | frozen site |
|---|---|---|---|
| `c00-census-reset` | `institution/` (carrier) | for-each community: `set` `member-count`, `r-raw`, `l-raw`, `f-raw`, `density-sum` to 0 | the per-tick rebuild, `:346`/`:392` |
| `c01-member-census` | `social-class/` | active classes only: for-each `hyperedges-of self` → `add 1` to `community/member-count` | `_collect_memberships` `:465-479` + `community_agents` `:392-397` |
| `c02-org-weight-reset` | `social-class/` | `set` the four per-class accumulators (`org-r-weight`, `org-l-weight`, `org-f-weight`, `org-count`) to 0 | (port scaffolding — D-NF+3) |
| `c03-org-weight-push-{r,l,f}` | `organization/` | **three rules**, one per tendency guard: for-each `neighbors self EdgeType/MEMBERSHIP :out NodeType/SOCIAL_CLASS` → `add (cadre × cohesion)` to `it`'s matching weight and `add 1` to `org-count`. The three guards **partition** every org — frozen's tendency-less skip (`:405-407`) is inexpressible, D-NF+25 | `:403-426` + `formulas/consciousness.py:63-72` |
| `c04-community-contribution-push` | `social-class/` | for-each `hyperedges-of self` → `add (org-r-weight / member-count)` to `community/r-raw` (same for l, f) and `add (org-count / member-count)` to `density-sum` | the density decomposition, §1.3 |
| `c05-normalize` | `institution/` (carrier) | for-each community: `unorganized = max(0, 1 − density-sum)` folded into `l-raw`; `total = r+l+f`; degenerate branch (`total < 1e-10`) → `(0, 1, 0)` **and nothing else** (the floor routes through `c06`, bit-identically — §6.2) vs normalize | `formulas/consciousness.py:78-95` |
| `c06-substrate-floor` | `institution/` (carrier) | for-each community: **the pack's ONLY** 14-arm `community/kind` dispatch → floor; the `r < floor` redistribution with its `lf > 1e-10` two-arm split | `formulas/consciousness.py:98-107` + §6 |
| `c07-contestation` **(DG-2-gated)** | `institution/` (carrier) | for-each community: normalized Shannon entropy of `(r,l,f)` with the `1e-10` guard, `/ (log 3)` | `entities/consciousness.py:274-291` |
| `c08-dominant-tendency` **(DG-2-gated)** | `institution/` (carrier) | for-each community: argmax with the `LIBERAL > REVOLUTIONARY > FASCIST` tie order at `1e-6` | `entities/consciousness.py:167-191`, epsilon `:189` (§2.2) |
| `c09-cost-modifier-reset` | `social-class/` | **active classes only** (`(when (= active 1))`, frozen `:472-474`): `set social-class/community-cost-modifier 1`. An INACTIVE class is written **nothing** — see §1.4 | `formulas/community.py:164-165` gated by `community.py:472-474` |
| `c10-cost-modifier-accumulate` | `social-class/` | **active classes only**, same guard: for-each `hyperedges-of self` → `scale` by `community/reproduction-cost-modifier`, in ascending `HyperedgeId` order (D-NF+13 records the float-product-order divergence) | `formulas/community.py:166-174` |
| `c11-state-decay` | `institution/` (carrier) | for-each community: three writes — heat, cohesion, education-pressure, each `max(0, x·(1−α))` | `:648-675` (infrastructure excluded, §5) |

(`c03` is three rules, so the pack is **14 `(rule …)` forms** across these 12 table rows — 12 if
DG-2 returns "do not publish" and `c07`/`c08` are never written.)

**The `c05`-skip gate.** Frozen recomputes consciousness only if `org_landscape` is non-empty
(`:452`) — a community with no overlapping orgs KEEPS its prior value. The port's equivalent is a
guard on `density-sum > 0` inside `c05`/`c06`/`c07`/`c08`; world 3 pins that a community with
members but no orgs is byte-unchanged across a tick.

### 8a. The duplication ledger — every copied expression and the row that proves the copies agree

| # | expression | copies | copies-agree row |
|---|---|---|---|
| 1 | the argmax tie-break (`1e-6`, LIBERAL-first) | `c08` here; `consciousness/p8-dominant-worldview` there | `dominant_tendency_ties_match_the_class_surface_rule` — a world seeding a community and a class at the same exact three-way tie, asserting both readouts pick LIBERAL; mutation: perturb this pack's epsilon → this test reds, the class-surface suite stays green (D-NF+8) |
| 2 | `max(0, x·(1−α))` decay | three times inside `c11` | `decay_arms_are_independent` — three mutation vectors, one per α constant, each flipping exactly one assertion |
| 3 | the `/ member-count` divisor | `c04` × four accumulators | `contribution_divisor_is_the_census_count` — mutate one arm's divisor to a literal, exactly one assertion reds |
| 4 | the `1e-10` degeneracy epsilon | `c05` (total) and `c07` (per-component entropy guard) | `degenerate_epsilons_are_independent` — the two guards answer different questions and must not be single-sourced into one constant |

Single-sourcing is **not available** in the language (no `defexpr`, no macro, no cross-rule binding);
"single-source" would mean "merge the rules", which costs the independent mutation-killability the
split buys. D-NF+20 records that reasoning once.

### 8b. The D116 ledger — every same-tick cross-rule read this pack relies on

| reader | writer | what breaks if apply-in-place is repaired to collect-across-rules |
|---|---|---|
| `c04` reads `community/member-count` | `c01` | the divisor becomes the previous tick's census — every consciousness value shifts by one tick of membership change (nothing today, since membership is static per world; **the re-open trigger is the first world that changes membership mid-run**) |
| `c04` reads `social-class/org-{r,l,f}-weight`, `org-count` | `c03-*` | the weights become one tick stale, same class of shift |
| `c05` reads `r-raw`/`l-raw`/`f-raw`/`density-sum` | `c00`, `c04` | the normalization would read pre-reset accumulators — **this one is fatal, not merely stale** |
| `c06`/`c07`/`c08` read the normalized `r/l/f` | `c05` | same, fatal |
| `c10` reads its own reset value | `c09` | the product would compound across ticks without bound |

**This table is the acceptance-criterion input for the Q14 collect-across-rules-then-apply train**
for `community.bsl` specifically — a stale row here feeds a wrong criterion into a future train, so
Task 12 states it as such in ADR-NF. D-NF+4.

### 8c. Permanent anti-pattern guards (the INV-010 estate's Rust half)

**Four** tests that outlive the frozen linter, landed in Task 7 and never deleted:

1. `no_community_typed_node_exists` — the world's node census contains no node whose type name
   contains `COMMUNITY`; communities appear only as hyperedges.
2. `no_field_binding_uses_the_community_namespace` — a source-level assertion over
   `community.bsl` that no `(binding … :field community/…)` exists, with the reason quoted from
   `babylon-bsl`'s `tick.rs:161-189` in the failure message.
3. `membership_crosses_whole` — the write log for a seeded community records ONE hyperedge with N
   members, never `C(n,2)` dyadic edges (VIII.9 verbatim).
4. **`exactly_one_institution_carrier`** (new in revision 2, §3.7a / C3) — every world this pack
   loads into contains exactly ONE `NodeType/INSTITUTION` node, with the failure message quoting
   `tick.rs:166-189` (every INSTITUTION-subject rule iterates every INSTITUTION node) and
   `tick.rs:212-216` (an unwritten bound field is a III.11 hard error). **Exactly ONE means
   never zero either**: a carrier-free world runs six of this pack's fourteen rules over an empty
   subject population, which is silent inertness dressed as a passing test — the failure mode
   world 5b would have had before revision 2's N1 fix. **All eight content worlds** (1, 2, 3, 4, 5,
   5b, 5c, 6) satisfy it. Two mutation vectors, not one: world 5c's **second** carrier (add one →
   this test reds before the `fired` arithmetic does) and world 5b's **removed** carrier (drop it →
   this test reds alongside `seam_world_community_half_actually_ran`).

---

## 9. The frozen-mirror recipe

Per the landed convention (D146/ADR183): the mirror is a **standalone, dependency-free Python
script** — no `babylon` import, no pytest — that transcribes **the rules'** binding order and
collect-then-apply semantics term-for-term over a literal `WORLD` dict matching the `.bscn`
node-for-node, hyperedge-for-hyperedge, seed-for-seed. It is **the oracle**; the frozen engine is
not (re-running frozen prints frozen's own, sometimes-diverged answer — here, D-NF+3's summation
order and D-NF+5's 500-org cap).

The mirror's header must state, for this pack specifically:
1. the exact rule order `c00 → c01 → c02 → c03r → c03l → c03f → c04 → c05 → c06 → c07 → c08 → c09
   → c10 → c11`, and that each rule applies before the next reads (§8b);
2. that `for-each` reads **pre-state** within a rule, so `c01`'s `add 1` accumulations are computed
   against the pre-rule value;
3. the exact `f64` operation order for each accumulator (the D-NF+3 divergence lives here) **and for
   `c10`'s product** — ascending `HyperedgeId`, which is where D-NF+13's multiplication-order
   divergence lives;
4. the floor-dispatch arm each seeded community takes, with its ADR214 provenance line;
5. **which fields are UNWRITTEN, per node** — the mirror models an inactive class's
   `community-cost-modifier` as **absent**, never as `1.0`, so the Rust assertion that reads it
   expects the substrate's honest-null error (§1.4, C4). A mirror that defaults an unwritten field
   is a mirror that cannot catch a fabricated write.

**Additionally, and separately from the oracle:** Task 7 Step 6 produces a one-off
**corroboration artifact** (`reports/community-frozen-corroboration-2026-08-18.md`) by driving the
**real** `CommunitySystem.step()` over a hand-seeded `BabylonGraph` — legal because
`_extract_memberships_from_node` accepts dicts (`:288-293`) and `services.community_hypergraph` is a
plain dict (`:296-306`), so the frozen path that is `STRUCTURALLY_IMPOSSIBLE` in production
(`sentinels/seam/registry.py:2171`) **is** runnable in a script. It is evidence for §1's
archaeology, **never** the conformance oracle, and the plan says so in both files.

**One named obligation on that artifact (C4).** Because the mirror transcribes the RULES, it cannot
by itself prove the rules match frozen — it would transcribe a divergence as faithfully as a
fidelity. The corroboration artifact carries the independent half for the one place revision 1 got
this wrong: it MUST seed an **inactive** SOCIAL_CLASS node holding a community membership, drive
`step()`, and record the resulting `graph.nodes[...]` dict for that node verbatim. Frozen's
`_collect_memberships` (`community.py:472-474`) skips it, so `community_cost_modifier` must be
**absent** from that dict. If it is present, `c09`/`c10`'s `active` guard is wrong and Task 10
STOPs. The same node is world 1's `n5`.

---

## 10. DIRECTOR GATE — the questions this plan does not decide

Popup-ready. None is resolved by any task; each blocks only what its row names.

- **DG-1 — Checkpoint A accounting.** This train ports four of Community @6.0's six phases; threat
  scoring, solidarity amplification and infrastructure maintenance wait on #653. Does @6.0 count as
  **ported** for ADR208 R14's "ALL 13 Material Base systems" test, or does Checkpoint A wait for the
  E3 half? *(Blocks: the Checkpoint A tally in ADR-NF and `ai/state.yaml`. Not the code.)*
- **DG-2 — the community consciousness readout.** Should the community hyperedge publish
  `contestation` and `dominant-tendency` (frozen does, `community.py:106-107`), accepting a **second
  declared home** for the hegemonic tie-break `consciousness.bsl:354` claims to hold alone — or
  should communities carry only the `(r,l,f)` simplex and let readers argmax? *(Blocks: `c07`, `c08`,
  §8a row 1, D-NF+8, the pack's rule count (14 vs 12), one golden pin's `fired` figure, **and the
  one-line amendment to another pack's declared `:material-basis`** — which is a theory-line edit and
  therefore must not be written before this answer arrives. Task 0 Step 1 posts this question;
  Task 9 Step 5 is gated on it and STOPs without it. Plan's recommendation: publish, with §8a's
  copies-agree row and the amended comment.)*
- **DG-3 — the floor table's declared width.** Declare all 14 ADR214 rows in every world (a
  complete ruled table, parity-tested), or only the rows a world's communities read
  (declare-what-you-read)? *(Blocks: §6.2. Recommendation: all 14.)*
- **DG-4 — `CommunityType` member order is hash-bearing (ADR195).** The frozen enum groups by
  category (hegemonic first: `SETTLER, PATRIARCHAL`, then marginalized, then institutional
  exclusion, then lifecycle). Confirm the port transcribes that order verbatim and does not
  re-group — including that the two hegemonic poles lead. *(Blocks: every world's `defenum`.)*
- **DG-5 — the national-question pole shape.** #664 rules the `county_extraction`
  `BoundOpposition` registration and the "per-nation sibling rows vs one binary opposition" choice
  to be "the Community port train's own work". This plan does **not** decide them (they are
  ADR171-line items) and does **not** need them (it mints no coupling). Confirm the deferral, or
  name the train that owns them. *(Blocks: nothing in this plan; blocks #664's Phase-2 closure.)*
- **DG-6 — the `community_memberships` writer's home.** #664 assigns the population-weighted
  seeder here, but a weighted membership **is** an AG(i) payload (#653). Propose: the writer lands
  in the post-#653 Community slice, not this train. Confirm the re-home. *(Blocks: #664's closure
  text.)*
- **DG-7 — infrastructure decay.** Defer the **whole** law until #653 (plan's recommendation), or
  land the decay half now? The argument for deferral is **port-as-is (ADR183)**: `infra·(1−α)`
  without the CORE_ORGANIZER maintenance term is a monotone law where frozen's
  (`formulas/community.py:77-108`) is non-monotone — a *different* law, not a partial one. *(Note
  for the Director: the frozen law suite does NOT pin the non-monotonicity — L4 pins
  "heat/cohesion never increase; infrastructure stays in [0, 1]"
  (`test_law_community_system.py:221`), and the non-monotonicity is filed there under "Caveats (not
  laws)" `:49-52`. Revision 1 mis-cited this; the deferral argument does not depend on it.)*
  *(Blocks: `c11`'s fourth write.)*
- **DG-8 — the III.7 path for hyperedge own-field storage.** Section `0x06` widens the canonical
  state field set **and adds a sixth REQUIRED `CanonicalState` listing**. ADR198 R1 landed the
  analogous dyadic-edge widening on a **program charter + ADR**; Amendment AG took the **amendment**
  path for membership payloads. Does hyperedge own-field storage need its own amendment, or does
  ADR-NF's III.7 declaration suffice? *(Blocks: Task 5's merge, and ADR-NF's framing.
  Recommendation: ADR path, on the ADR198 R1 precedent, since Amendment D already ratified
  hyperedges as first-class objects.)*

---

## File Structure

| File | Responsibility |
|---|---|
| Create `reports/community-bsl-surface-facts-2026-08-18.md` | Task 0's dossier |
| Modify `rust/crates/babylon-graph/src/{substrate,memory,hypergraph_store,conformance}.rs` | **Task 2** — the type-scoped `hyperedges(hyperedge_type)` enumerator on `GraphSubstrate`, both backends, its `run_substrate_conformance` row (§3.2 row 0) |
| Modify `rust/crates/babylon-bsl/src/scenario.rs` | `(hyperedge …)` + `(hyperedge-attr …)` top-forms; the id-order law extended; **`LoadedScenario.hyperedge_types`** (§3.2 row 9) |
| Modify `rust/crates/babylon-bsl/src/{query,evaluator,score_class,typecheck}.rs` | the three served hyperedge heads in **both** unserved tables + `SERVED_QUERY_HEADS`; `Element::Hyperedge` + the cross-kind Ord ruling; typecheck coverage. **`bound_checker.rs` is NOT modified** — its hyperedge axis is landed (§3.1) |
| Modify `rust/crates/babylon-graph/src/{substrate,memory,hypergraph_store,state_hash,conformance}.rs` | **Task 5** — hyperedge attribute storage + the sixth `CanonicalState` listing + section `0x06` + caller-side elision + conformance rows |
| Modify `rust/crates/babylon-bsl/src/structural_verbs.rs` | `update-hyperedge` executes in **both** dispatch sites; `field-of` refusal text corrected |
| Create `rust/crates/babylon-bsl/tests/hyperedge_lane_e2e.rs` | E1+E2's own conformance vectors (the NAMED deliverable's evidence) |
| Modify `rust/crates/babylon-tick/src/lib.rs` | **two** changes, in two different tasks: the DRIVER's ceiling construction at `:263-276` (**Task 4** — the `HyperedgeType/*` ceiling map + the `max_members` map, §3.2 row 10, the declared §3.6 amendment) and one registration string `"community"` (**Task 7**) |
| Create `rust/crates/babylon-tick/tests/hyperedge_ceilings.rs` | **Task 4** — the driver-level proof: `MissingCeiling`/`E-LOAD-042` before, a computed `E-LOAD-040` bound after |
| Create `rust/crates/babylon-tick/content/rules/community.bsl` | the pack: **14 rules** (12 if DG-2 declines) + the `defconst` block + the file-local `D-N` header |
| Create `content/scenarios/community-conformance.bscn` + `community_conformance.py` | world 1 + the primary mirror (drives worlds 1-4) |
| Create `content/scenarios/community-floor-conformance.bscn` | world 2 — the floor-binding + ordering witnesses |
| Create `content/scenarios/community-degenerate-conformance.bscn` | world 3 — the degenerate branch + the no-org skip gate |
| Create `content/scenarios/community-cost-modifier-conformance.bscn` | world 4 — the product accumulation + the exactly-1.0 witness |
| Create `content/scenarios/community-decay-arc-conformance.bscn` + `community_decay_arc_conformance.py` | world 5 — the multi-tick decay arc + its mirror |
| Create `content/scenarios/community-solidarity-seam-conformance.bscn` | world 5b — §2.1's combined `solidarity` + `community` attribute-seam witness. **One carrier + one community, like every world here** (§3.7a): the community half must be LIVE for the seam proof to mean anything. It cannot detect the carrier collision — not for lack of a carrier, but because `solidarity.bsl` runs no INSTITUTION-subject rule |
| Create `content/scenarios/community-carrier-collision-conformance.bscn` | **world 5c (new in revision 2, §3.7a)** — `community` + `control-ratio` co-loaded over ONE shared INSTITUTION carrier, with per-rule-id `fired` arithmetic |
| Create `content/scenarios/community-empty-conformance.bscn` | world 6 — frozen L2's analogue: one carrier, one community whose members are all **inactive** classes, so a fully-loaded tick is a byte-exact no-op (§1.2). **L1's analogue is a refusal test, not this world** — a scenario with no community hyperedge fails `MissingCeiling` at load |
| Create `rust/crates/babylon-tick/tests/community_conformance.rs` | worlds 1-4, 6 + mutation vectors + §8a rows + §8c's **four** guards |
| Create `rust/crates/babylon-tick/tests/community_arc_conformance.rs` | worlds 5, 5b, 5c via `TickSession` + §8b's cross-tick assertions |
| Modify `rust/crates/babylon-tick/tests/tick_goldens.rs` | additive pins (**8 new** — one per content world, each pinning `before` + `after`); the 16 pre-existing untouched |
| Create `reports/community-frozen-corroboration-2026-08-18.md` | §9's corroboration artifact (never the oracle) — carries the inactive-node obligation |
| Modify `docs/reference/bsl-language.rst` | register rows **D-NF+1 … D-NF+25**; the §2.8/§2.9/§2.10 hyperedge sections move from "slice 3/4" to served, for the three heads that land |
| Modify `rust/crates/babylon-tick/content/rules/consciousness.bsl` | **only if DG-2 returns "publish"**: one comment amendment at `:354` (§2.2), which also corrects that comment's own frozen anchor |
| Create `ai/decisions/ADR-NF_community_port_handoff.yaml` + `index.yaml` row | handoff record |
| Modify `ai/state.yaml` | closing entry |

---

### Task 0: Governance, measurement, and the starting line

**Files:** Create `reports/community-bsl-surface-facts-2026-08-18.md`.

- [ ] **Step 1: Open the implementation issue** on project 8 under the Checkpoint A umbrella,
      linking #536 (charter), #653 (the E3 dependency), #664 (the writer), ADR214 (the floor
      rulings), ADR198 R4 (the hand-off), and this plan. State the four-of-six-phases scope in the
      issue body so the scope is public before code lands. **Post the eight DIRECTOR GATE questions
      to the docket (#564) in the same step, not at Task 12** — **DG-2 gates content this train
      writes** (§2.2: `c07`/`c08` and a theory-line edit to `consciousness.bsl`), so its answer must
      be in hand before PR D opens or Task 9 STOPs at `c07`.
- [ ] **Step 2: RE-MEASURE the numbering tails** — `rg -no 'D[0-9]+' docs/reference/bsl-language.rst
      | sort -u | tail` and `tail ai/decisions/index.yaml`. Measured 2026-08-17: **D180**, **ADR214**.
      **Both are contended** (the ImperialRent train claims ten rows against the same tail; #491
      allocates from it too). Record the tail measured **today** and fix this train's allocation as
      `D<tail+1> … D<tail+25>` and `ADR<tail+1>`; every later task uses that allocation, and Task 12
      re-measures once more before filing.
- [ ] **Step 3: Observe the starting line** — run `mise run rust:check` **single-flight** and record:
      the exact count of `#[test]` functions and `*_hashes_are_pinned` pins in `tick_goldens.rs`
      (expected 18 / 16), and the 16 pins' current hashes pasted into the dossier as the
      byte-identity baseline every later task's gate compares against.
- [ ] **Step 4: Owed re-reads, recorded verbatim** — the §3.2 refusals with their exact text and
      line numbers (they are error-text contract surface and Tasks 1-6 must preserve or deliberately
      amend each), **including both `update-hyperedge` sites (`:452-466`, `:873-879`) and both
      unserved-head tables (`query.rs:99-103`, `evaluator.rs:544-551`)**; `babylon-bsl`'s
      `tick.rs::subject_type_of` + `namespace_to_node_type` + the III.11 site `:212-216`;
      `scenario.rs`'s top-form dispatch (`:570-611`) and its id-order law; `state_hash.rs`'s
      `encode_state` elision site **and its `CanonicalState` "five-way listing" doc `:292-320`**;
      `babylon-tick/src/lib.rs:263-276`'s ceiling construction, verbatim.
- [ ] **Step 5: Record the rider-4 renegotiation, BOTH clauses** (§0, §3.4) — three of the four
      slice-3 heads land early, AND the A5 Element-Ord precondition is discharged in-train rather
      than waited on. Quote the rider in full in the dossier, state the `Node < Edge < Hyperedge`
      ruling Task 3 will land, and post the same paragraph to #536 so the Director sees a decision,
      not a drift.
- [ ] **Step 6: Settle the open shape questions at the byte**, each with the source line that
      decides it. **(a)** Can a `for-each` body target a hyperedge element for `update-hyperedge`
      (i.e. is `it` a legal first operand)? **(b)** Does `scale` accumulate multiplicatively across
      repeated writes in one rule the way `add` accumulates (`PendingWrite` reads `current` at
      APPLY)? **(c)** Does a `defconst` reference resolve inside an `if`-chain 14 levels deep, and
      is there an arity or nesting ceiling? **(d)** Does `(field-of h community/kind)` over an
      `:enum-type` hyperedge field typecheck for equality against an enum-ref (the D102 discharge
      applies to nodes — confirm it generalizes)? **(e)** ~~Where does the bound checker need a new
      axis?~~ **RESOLVED BEFORE THE TASK, revision 2: nowhere.** `ceiling_of_query`
      (`bound_checker.rs:544-572`) already bounds all three heads; record instead **where the two
      ceiling MAPS come from** (`LoadedScenario` → the driver → `CardinalityCeilings::new`) and the
      `:max-members` derivation D-NF+22 rules. **(f)** Are negative literals needed anywhere here
      (they are not — every constant is non-negative), confirm. **(g)** What error does an unknown
      `HyperedgeType` member raise in a scenario type position? (Measured 2026-08-18: **`E-LOAD-031`
      `UnknownEnumMember`**, `vocabulary.rs:118,160`; `E-LOAD-023` is `UnknownFieldOwner`, a FIELD
      QNAME's first segment `:145,163` — revision 1 prescribed the wrong code, so confirm at the
      byte before Task 1 writes the refusal.)
- [ ] **Step 7: Commit** `docs(port): community BSL surface-facts dossier (the hyperedge-lane
      refusals, the subject-type trap, and the numbering allocation)`.

**Gate:** none (docs only) — but run `vale` over the dossier, and `mise run rust:check` once for
Step 3's baseline (single-flight).
**Estimate:** ~3h · ~45k tokens.

---

### Task 1: E1a — hyperedge scenario seeding + `LoadedScenario.hyperedge_types` (PR A)

**Files:** Modify `rust/crates/babylon-bsl/src/scenario.rs`; create
`rust/crates/babylon-bsl/tests/hyperedge_lane_e2e.rs`.

**Interfaces:** Produces the `(hyperedge …)` form every world depends on, the `HyperedgeId`-order law
every later id literal depends on, and **the `hyperedge_types` population map Task 4's ceilings are
built from** (§3.2 row 9).

- [ ] **Step 1: Failing tests** in `hyperedge_lane_e2e.rs` — a scenario with
      `(defvocabulary HyperedgeType (COMMUNITY))` and
      `(hyperedge new-afrikan HyperedgeType/COMMUNITY (members alpha beta))` fails to load today
      ("unknown top-form"). Assert the refusal text first, so the change of behaviour is visible in
      the diff.
- [ ] **Step 2: Add the top-form** to the `scenario.rs` dispatch, mirroring `node`/`edge`: resolve
      the type through the closed `HyperedgeType` vocabulary (**`E-LOAD-031` `UnknownEnumMember`** on
      an unknown member — `vocabulary.rs:118,160,192`; revision 1 prescribed `E-LOAD-023`, which is
      `UnknownFieldOwner`, a *field qname's* first segment, `:145,163,213`), resolve each member name
      through the same local-name table nodes use, refuse an empty member list (matching
      `memory.rs:357-361`'s "hyperedge must have at least one member"), and mint through
      `GraphSubstrate::add_hyperedge`. **Member canonicalization is the executor's already-landed
      law** (`structural_verbs.rs:1366` sorts ascending, `:1371-1373` records WHOLE) — the loader
      must produce byte-identical results, and a test asserts the two paths agree on a deliberately
      unsorted member list.
- [ ] **Step 3: Extend the id-order law** — `HyperedgeId`s are minted top-to-bottom exactly as
      `NodeId`s are, and **the two counters are independent** (`substrate.rs:35-41`'s type-level
      separation). A test proves inserting a `hyperedge` form does not shift any `NodeId`, so no
      landed world's ids can move.
- [ ] **Step 4: `LoadedScenario.hyperedge_types`** — a `HashMap<String, u64>` counting hyperedges per
      `HyperedgeType` member as they are seeded, **plus `max_members_seen: HashMap<String, u64>`**
      (the longest member list per type). Both mirror `node_types`/`edge_types`' own stated argument
      verbatim (`scenario.rs:242-248`: "*taking it from the population the scenario ACTUALLY built
      means the static bound is checked against a real number rather than an invented one*"), and
      both are the inputs Task 4 turns into ceilings. Doc each with that citation; a test pins both
      against a three-hyperedge fixture with unequal member lists.
- [ ] **Step 5: Update the module doc** — `scenario.rs:63-64`'s "No hyperedges yet" paragraph is
      replaced by what now exists and what still does not (attributes land in Task 6; membership
      payloads are #653's), dated and citing this plan. Note the doctrine sentence it contains ("*an
      unused form is an untested form*", `:64`) survives — it is why `metric-of` still does not land.
- [ ] **Step 6: Mutation vectors** — delete the member-sort → the unsorted-input test reds; drop the
      empty-member-list refusal → its test reds; swap the two id counters → the node-id-stability
      test reds; return a `0` from `max_members_seen` → the count test reds.
- [ ] **Step 7: Commit** `feat(bsl): scenario hyperedge seeding — the (hyperedge …) top-form and its
      population maps`.

**Gate:** six cargo legs, **single-flight**; the 16 pre-existing pins byte-identical (no hash
surface changes — `TAG_HYPEREDGES` already existed and no landed world seeds one).
**Estimate:** ~5h · ~70k tokens.

---

### Task 2: E1b — the type-scoped hyperedge enumerator on `GraphSubstrate` (PR A) — **NEW in revision 2 (C1)**

**Files:** Modify `rust/crates/babylon-graph/src/{substrate,memory,hypergraph_store,conformance}.rs`.

**Why this is its own task, in another crate.** `(hyperedges HyperedgeType/COMMUNITY)` — the head
**six** of this pack's rules iterate — has no substrate accessor to materialize from (§3.2 row 0).
Revision 1 scoped that head to a `babylon-bsl`-only task, which would have forced a
`babylon-graph` change inside a step forbidden to make one (§3.6). **This is the re-plan §3.6
demands, not an improvisation inside Task 3.**

**Interfaces:** Produces `GraphSubstrate::hyperedges(&self, hyperedge_type: &str) -> Vec<HyperedgeId>`
— ascending, deduplicated, symmetric with `nodes` `:204` and `edges` `:208`.

- [ ] **Step 1: Failing conformance rows first** — add `hyperedges_by_type_is_ascending_and_typed`
      and `hyperedges_of_an_undeclared_type_is_empty_not_loud` to `conformance.rs`'s
      `run_substrate_conformance` (`:28-45`), which both backends run. They do not compile today:
      the method does not exist. **A `GraphSubstrate` method with no conformance row is a method
      only one backend is proved to hold** (I13).
- [ ] **Step 2: The trait method**, doc-commented against `nodes`/`edges`' own contract text: total
      order is the accessor's own guarantee (ascending `HyperedgeId`), never the caller's, and an
      unknown type yields an empty `Vec` exactly as `nodes` does for an unpopulated type — the
      loudness lives at the BOUND checker (`MissingCeiling`), not here.
- [ ] **Step 3: Both backends.** `MemoryGraph`: filter `hyperedges:
      HashMap<HyperedgeId, (String, Vec<NodeId>)>` (`memory.rs:57`) by type, `sort_unstable`, exactly
      as `hyperedges_of` `:398-419` already does. `HypergraphStore`: read the **already-present**
      `hyperedge_type_index: HashMap<String, Vec<HyperedgeId>>` (`hypergraph_store.rs:91`) and sort
      the result — do not add a second index for one datum.
- [ ] **Step 4: The declared-order leak test** — extend
      `declared_order_never_leaks_through_any_ranged_accessor` (`conformance.rs:478-514`) to the new
      accessor: seed hyperedges in descending order, read ascending.
- [ ] **Step 5: Mutation vectors** — drop the `sort_unstable` → the ordering row reds on BOTH
      backends; make `HypergraphStore` return the index unsorted → same; widen the type filter → the
      typed row reds.
- [ ] **Step 6: Commit** `feat(graph): type-scoped hyperedge enumeration on GraphSubstrate`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical (**a read-only accessor
touches no hash surface** — `CanonicalState` is untouched by this task; that is Task 5).
**Estimate:** ~4h · ~55k tokens.

---

### Task 3: E1c — the three hyperedge query heads + the `Element` Ord ruling (PR A)

**Files:** Modify `rust/crates/babylon-bsl/src/{query,evaluator,score_class,typecheck}.rs`; extend
`tests/hyperedge_lane_e2e.rs`. **`bound_checker.rs` is NOT in this list** — its hyperedge axis is
landed (§3.1) and revision 1's Step 4 was written against a false premise.

**Interfaces:** Produces `(hyperedges …)`, `(members-of …)`, `(hyperedges-of …)` as servable query
operands, and `Element::Hyperedge` with its ruled cross-kind order.

- [ ] **Step 1: Failing tests** — each head, used as a `for-each` operand and as a `fold` operand,
      currently refuses through **both** tables. Pin the current refusal text of each before changing
      it: `query::UNSERVED_QUERY_HEADS` (`:99-103`, what `materialize()` consults) and
      `evaluator::UNSERVED_EXPRESSION_HEADS` (`:544-551`, the expression-position classifier).
- [ ] **Step 2: `Element::Hyperedge(HyperedgeId)` + THE ORD RULING (rider 4 clause 1, C5).** Declare
      the variant **THIRD** in `query.rs`'s enum (`:69-75`), and extend that enum's own standing
      cross-kind instruction (`:56-63`, register row D140 / CT4P A5 / #525) with the explicit
      ruling: **`Node` sorts before `Edge` sorts before `Hyperedge`, by declaration order —
      arbitrary, deliberate, tested**, on exactly the reasoning the landed paragraph gives ("*pinned
      anyway, per this enum's own standing instruction, rather than left to whatever
      `#[derive(Ord)]` happens to produce*"). Land the companion test beside
      `node_sorts_before_edge_regardless_of_id`:
      `edge_sorts_before_hyperedge_regardless_of_id`, plus a three-kind vector. Amend `query.rs:17`'s
      "`Hyperedge(HyperedgeId)` (slice 3) is deliberately not added" paragraph in the same commit.
      Extend `Element::to_value` with the `HyperedgeRef` arm. **D-NF+24.**
- [ ] **Step 3: Materialize the three heads** in `query.rs` — `Element::Hyperedge` for
      `hyperedges` (via Task 2's enumerator) and `hyperedges-of`, plain `Element::Node` for
      `members-of`. **Iteration order is the ruled order**: ascending `HyperedgeId` for hyperedge
      results, ascending `NodeId` for members (D25, already the substrate's contract via
      `members_of`). A test pins both orders against a deliberately reversed seeding.
- [ ] **Step 4: Move the three heads across BOTH tables** — out of `query::UNSERVED_QUERY_HEADS`
      (which becomes a zero-row table, or is deleted with its refusal arm — decide at the byte and
      record which), out of `evaluator::UNSERVED_EXPRESSION_HEADS`, and **into
      `evaluator::SERVED_QUERY_HEADS` (`:567`, 3 entries → 6)**, preserving the query-operand-only
      law (`:553-567`: served heads are legal as the query operand of an iterating form, never as a
      bare `<expr>`). Keep the two tables' cross-referencing docs true after the move — a duplicated
      table named once is this plan's own M4 failure mode applied to itself (I6). `metric-of` and
      `membership-field-of` stay unserved, with `membership-field-of`'s "slice 4" note amended to
      cite **#653** by number rather than a slice.
- [ ] **Step 5: Verify — do not build — the fuel axis.** Add a readback test proving a rule with
      `(fold count (hyperedges HyperedgeType/COMMUNITY) it)` reports the `E-LOAD-040` static bound
      `ceiling_of_query` already computes, and one proving a `members-of` fold without a
      `max_members` entry is `E-LOAD-042` (`bound_checker.rs:995-1002` is the landed unit-level
      analogue; this is its through-the-language counterpart). **If either behaves differently than
      §3.1 states: STOP and re-plan Task 4** before content depends on it.
- [ ] **Step 6: Typecheck + score-class coverage** — `typecheck.rs` gains its first hyperedge logic
      (the enum-ref operand positions, the element type of each head, and the refusal for a
      hyperedge element in a numeric position); `score_class.rs`'s `HyperedgeReference` arm
      (`:238`) gets an executed test rather than a declared one.
- [ ] **Step 7: Mutation vectors** — invert the hyperedge iteration order → the order test reds;
      re-declare `Hyperedge` FIRST in the enum → the Ord companion test reds; drop the
      query-operand-only guard → the bare-`<expr>` refusal test reds; leave one of the two unserved
      tables un-updated → a `materialize()` refusal test reds (the I6 vector).
- [ ] **Step 8: Commit** `feat(bsl): serve hyperedges / members-of / hyperedges-of + the
      Element cross-kind Ord ruling (slice 3, the three heads Community needs)`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical.
**Estimate:** ~7h · ~95k tokens.

---

### Task 4: E1d — the ceiling supply chain (PR A) — **NEW in revision 2 (C2)**

**Files:** Modify `rust/crates/babylon-tick/src/lib.rs` (the ceiling construction at `:263-276`
only); create `rust/crates/babylon-tick/tests/hyperedge_ceilings.rs`.

**Why this task exists, and what it amends.** Revision 1 asserted the bound checker needed a new
axis and that `babylon-tick` would gain "no hyperedge logic beyond one registration string". Both
are false: the axis is landed, and the DRIVER is where the axis is starved —
`CardinalityCeilings::new(<NodeType+EdgeType counts>, HashMap::new())` (`lib.rs:263-276`,
`fuel.rs:95`) passes an **empty** `max_members` map and mints no `HyperedgeType/*` ceiling, so
**every rule in `community.bsl` fails at LOAD** with `MissingCeiling` on
`HyperedgeType/COMMUNITY` and `E-LOAD-042 MissingMaxMembers` on the census reads. This task
**amends §3.6's cap openly** (see that section) and is the reason the amendment is in the plan text
rather than discovered at Task 8.

**Interfaces:** Produces loadability. Nothing downstream of it has a meaningful `:fuel` figure
without it.

- [ ] **Step 1: Failing tests** in `hyperedge_ceilings.rs` — a minimal scenario seeding two
      `HyperedgeType/COMMUNITY` hyperedges plus a rule folding `(hyperedges …)` fails to load today;
      pin the exact `MissingCeiling` text. A second rule folding `(members-of …)` fails with
      `E-LOAD-042`; pin that too.
- [ ] **Step 2: Feed both maps from the scenario** — extend the `ceilings` construction to chain
      `scenario.hyperedge_types` as `HyperedgeType/{member}` into the ceiling map (exactly as
      `node_types`/`edge_types` are chained; the three namespaces are disjoint so the flat map
      cannot collide — the landed comment at `:250-262` makes that argument for two and it extends
      to three) and to pass `scenario.max_members_seen` as the **second** argument in place of
      `HashMap::new()`.
- [ ] **Step 3: Record the `:max-members` DECISION, not just the code (D-NF+22).** The value is
      **derived from the seeded population** — the longest member list of that hyperedge type in the
      world being loaded — not stipulated by a constant and not declared by a content `manifest`
      form. Argued, with provenance:
      - **Symmetry with the landed law.** `node_types`/`edge_types` already take ceilings from "*the
        population the scenario ACTUALLY built*… rather than an invented one" (`scenario.rs:242-248`).
        A stipulated hyperedge cap would be the only invented number in the map.
      - **The gameplay-and-pedagogy compass (Director, CLAUDE.md).** Community membership is
        *content* — which classes belong to NEW_AFRIKAN is the political claim the pack teaches, and
        a modder editing it (#531's HOI4-style packs, the first external modder already waiting) must
        not silently blow an engine-side cap. A derived ceiling means the modder's world is bounded
        by the modder's world; an invented one means a constant no player can see decides whether
        their scenario loads. Where it *does* fail — a world larger than the `:fuel` a rule
        declares — it fails **loudly at LOAD** with a number to raise, which is the estate's own
        preference (III.11's spirit: absence and overflow are announced, never defaulted).
      - **The fuel consequence, stated plainly:** a rule's static bound is **per-world**, so this
        pack's declared `:fuel` is the MAXIMUM over every world that loads it, re-measured whenever a
        world is added (Global Constraints), and a later, larger world reds the load until someone
        raises it. That is the intended failure, not a defect.
      - **Rejected alternative:** a content `manifest` form declaring `:max-members`
        (`manifest.rs:11-15`'s route). No landed content declares a manifest at all (`grep -rn
        '(manifest' content/` → zero hits), so this would make Community the first, and would put an
        invented cap in fixtures. **Re-open trigger:** the first content set that genuinely needs a
        cap larger than any world it ships with (an unbounded runtime population), which is where a
        manifest earns its keep.
- [ ] **Step 4: The through-the-driver proof** — the two Step-1 tests now load, and a third pins the
      **measured** `E-LOAD-040` bound for a `(members-of …)` fold at exactly
      `2 + query(2) + max_members × body`, matching `bound_checker.rs:897-905`'s landed arithmetic
      shape. Read the number back from the refusal; never hand-compute it.
- [ ] **Step 5: Mutation vectors** — restore `HashMap::new()` for the second argument → the
      `members-of` load test reds with `E-LOAD-042`; drop the `HyperedgeType/` prefix when chaining
      → `MissingCeiling` reds; feed `max_members_seen` a constant → the measured-bound test reds.
- [ ] **Step 6: Commit** `feat(tick): supply hyperedge ceilings to the bound checker (the
      HyperedgeType ceiling map and :max-members)`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical (**no landed world seeds a
hyperedge, so no landed ceiling map changes**; assert that explicitly — a moved pin here means the
chaining touched the node/edge maps).
**Estimate:** ~4h · ~55k tokens.

---

### Task 5: E2a — hyperedge attribute storage + `CanonicalState` section `0x06` (PR B)

**Files:** Modify
`rust/crates/babylon-graph/src/{substrate,memory,hypergraph_store,state_hash,conformance}.rs`.
(**`conformance.rs` added in revision 2, I13** — the both-backends suite every landed accessor has a
row in; two new accessors land two new rows.)

**Interfaces:** Produces the storage and hash contract every community field depends on. **This is
the Constitution III.7 step** — it is its own PR for exactly that reason.

- [ ] **Step 1: Failing tests** — `set_hyperedge_attribute` / `hyperedge_attribute` do not exist;
      write the round-trip rows **in `conformance.rs`'s `run_substrate_conformance`** (so both
      backends run them) first: a round-trip, an honest-null read of an unwritten attribute, a loud
      read against an unknown `HyperedgeId`, and the removal cascade (removing a hyperedge drops its
      attributes — extend `removal_cascades_edges_memberships_and_attributes`, `:31`).
- [ ] **Step 2: Storage on both backends**, mirroring T3's `edge_attributes` shape **exactly**
      (`hypergraph_store.rs:85` is the model: `HashMap<(String, NodeId, NodeId, String), f64>`) —
      i.e. `HashMap<(HyperedgeId, String), f64>`, `f64` values, loud `GraphError` on a non-finite
      value or an unknown hyperedge, never a silent insert. **The sort is `encode_state`'s, not the
      map's** (§4, I14): revision 1 said "a `BTreeMap` … sorted by construction" in one place and
      "mirror T3 exactly" in another, and following either violated the other. One datum, one sort
      authority.
- [ ] **Step 3: The SIXTH `CanonicalState` listing** — `all_hyperedge_attributes(&self) ->
      Vec<(HyperedgeId, String, f64)>`, **required, never defaulted**, on the trait's own argument
      for the fifth (`state_hash.rs:314-319`). Implement on both stores; update the trait's "five-way
      listing" doc (`:292`) and its `encode_state` contract prose in the same commit.
- [ ] **Step 4: Section `0x06`** — `TAG_HYPEREDGE_ATTRIBUTES = 0x06` and
      `write_hyperedge_attributes`, byte layout copied from `write_edge_attributes`
      (`state_hash.rs:259-274`) with `(id, qname, value)` in place of `(type, from, to, qname,
      value)`; sorted ascending by `(id, qname)` **inside `encode_state`**; the layout-version doc
      updated.
- [ ] **Step 5: THE ELISION PROOF** — `encode_state` decides, as it does for `0x05`, that an **empty**
      sixth listing contributes **zero bytes** (ADR198 R2). Two tests: (a) a graph with hyperedges
      but no hyperedge attributes hashes byte-identically to the pre-change encoder (pin the literal
      pre-change hash from Task 0's baseline); (b) one attribute changes the hash. **The 16 landed
      pins are the third, decisive proof** and they are this task's gate.
- [ ] **Step 6: Mutation vectors** — remove the elision → the 16 pins move (the loudest possible
      vector; restore byte-identical); change the section sort key → the ordering test reds; drop
      the non-finite check → its refusal test reds; give `all_hyperedge_attributes` a default empty
      body on the trait → the second backend's conformance row reds (the "answer out loud, at
      compile time" argument, executed).
- [ ] **Step 7: Commit** `feat(graph): hyperedge attribute storage + CanonicalState section 0x06
      (III.7, empty-elided)`.

**Gate:** six cargo legs, **single-flight**; **all 16 pre-existing pins byte-identical — if one
moves, STOP**, the elision is wrong.
**Estimate:** ~7h · ~90k tokens.

---

### Task 6: E2b — the BSL read/write surface for hyperedge fields (PR B)

**Files:** Modify `rust/crates/babylon-bsl/src/{structural_verbs,evaluator,scenario,typecheck}.rs`;
extend `tests/hyperedge_lane_e2e.rs`.

- [ ] **Step 1: Failing tests** — `(update-hyperedge h community/heat (set 0.5c))` and
      `(field-of h community/heat)` both refuse today; pin both refusal texts first (they are
      contract surface and both are about to change meaning).
- [ ] **Step 2: `update-hyperedge` executes in BOTH dispatch sites** — `execute_item`
      (`structural_verbs.rs:452-466`) and `collect_items` (`:873-879`, re-measured — revision 1
      cited `:868-879`, which starts inside the `update-edge` arm). The M4 lesson
      (`:539-545`: a mutation deleting the collect path's loop flipped zero tests because every
      test drove the execute path) means **each site owes its own mutation vector**, and the
      collect path owes a `for-each`-driven test.
- [ ] **Step 3: `field-of` over a `HyperedgeRef`** — replace the refusal (`evaluator.rs:1318-1322`)
      with a lookup, and **rewrite the message that remains** for the membership case so it cites
      #653 rather than asserting a hyperedge "carries no attributes of its own", which is about to
      be false.
- [ ] **Step 4: The `deffield` hyperedge lane** — hyperedge-subject field declarations that
      `subject_type_of` never sees, plus the enum-typed field case (`community/kind`). A test
      asserts a hyperedge-namespace `:field` **binding** still refuses with the subject-type error,
      so the §8c guard has a mechanism behind it.
- [ ] **Step 5: `(hyperedge-attr <name> <qname> <literal>)` seeding** in `scenario.rs`, mirroring
      `(edge-attr …)` (`consciousness-ternary-conformance.bscn:309-313` is the shape precedent),
      with the same int/fractional literal conversion contract and the same refusal for a
      fractional seed on an `int`-declared field.
- [ ] **Step 6: Typecheck coverage** for both directions (a write of the wrong scalar type; a read
      in an arithmetic position of an enum field).
- [ ] **Step 7: Commit** `feat(bsl): hyperedge own-field read/write + (hyperedge-attr …) seeding`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical.
**Estimate:** ~7h · ~90k tokens.

---

### Task 7: Registration + world 1 + the mirror + THE SPIKE (PR C)

**Files:** Modify `rust/crates/babylon-tick/src/lib.rs` (the systems `HashSet` only — the driver's
ceiling construction changed back in Task 4); create
`content/scenarios/community-conformance.bscn`, `content/scenarios/community_conformance.py`,
`rust/crates/babylon-tick/tests/community_conformance.rs`,
`reports/community-frozen-corroboration-2026-08-18.md`.

- [ ] **Step 1: Failing load-smoke test** — `community_conformance.rs` with
      `include_str!` of the scenario and an empty rule source; expected FAIL (unregistered system).
- [ ] **Step 2: Register** `"community".to_owned()` in `lib.rs`'s systems `HashSet` (`:277-351`),
      with a comment citing **Material Base @6.0** in the shape the landed `"decomposition"` /
      `"control-ratio"` rows use (`:335-350`), confirming the entry is genuinely new.
- [ ] **Step 3: Write world 1** — `community-conformance.bscn`. Declaration order is id order;
      **declare in this order and never renumber when extending**:

  | # | name | kind | seeds | why it exists |
  |---|---|---|---|---|
  | n0 | `community-register` | INSTITUTION | `institution/community-carrier 1` | **THE world's one and only INSTITUTION node** (§3.7a) — the subject-type anchor, not a mint of this pack's own; §8c row 4 asserts there is exactly one |
  | n1 | `na-worker` | SOCIAL_CLASS | `active 1` | member of NEW_AFRIKAN + QUEER — the multi-community witness |
  | n2 | `na-organizer` | SOCIAL_CLASS | `active 1` | member of NEW_AFRIKAN only |
  | n3 | `settler-la` | SOCIAL_CLASS | `active 1` | member of SETTLER — the 0.0-floor control |
  | n4 | `unaffiliated` | SOCIAL_CLASS | `active 1` | **no** membership — pins `community-cost-modifier == 1.0` exactly |
  | n5 | `inactive-member` | SOCIAL_CLASS | `active 0` | member of NEW_AFRIKAN. Excluded from the census (`:472-474`) **and from the cost-modifier write entirely** — its `community-cost-modifier` must read as the substrate's honest-null error, never `1.0` (§1.4, C4) |
  | n6 | `rev-org` | ORGANIZATION | `cadre-level 0.5p`, `cohesion 0.8p` | REVOLUTIONARY tendency; MEMBERSHIP → n1, n2 |
  | n7 | `lib-org` | ORGANIZATION | `cadre-level 0.25p`, `cohesion 0.5p` | LIBERAL; MEMBERSHIP → n1, n3 |
  | n8 | `fash-org` | ORGANIZATION | `cadre-level 0.5p`, `cohesion 0.25p` | FASCIST; MEMBERSHIP → n3 |
  | n9 | `no-member-org` | ORGANIZATION | `cadre-level 1p`, `cohesion 1p` | **zero** MEMBERSHIP edges — pins frozen's `:421` skip |

  Hyperedges (id order, independent counter): h0 `new-afrikan` (members n1, n2, n5), h1 `settler`
  (n3), h2 `queer` (n1). Each seeded with `community/kind`, `community/heat`, `community/cohesion`,
  `community/education-pressure`, `community/reproduction-cost-modifier`, and the prior-tick
  `community/revolutionary|liberal|fascist` (so the no-org skip gate has something to preserve).
  All 14 floor `defconst`s declared (§6.2 / DG-3).
  **Every literal is an exact dyadic rational** with the `c`/`p` suffix its field's type requires,
  except where a witness needs a value a `[0,1]`-bounded literal cannot carry — each such case gets
  its own comment naming why.
- [ ] **Step 4: Write the mirror** `community_conformance.py` per §9 — standalone, no `babylon`
      import, a literal `WORLD` dict matching the `.bscn` node-for-node and hyperedge-for-hyperedge,
      transcribing the rule order and the pre-state law. Run it; paste stdout **verbatim + dated**
      into the Rust test's doc comment with the "why exact equality, no tolerance" paragraph.
- [ ] **Step 5: THE SPIKE — prove SEVEN shapes before fourteen rules depend on them.** A throwaway
      rule (deleted at the end of this step; verdict recorded in the scenario header, the
      `solidarity-conformance.bscn:9-20` precedent) proving against the real driver and this
      `.bscn`: **(a)** a `for-each` over `(hyperedges HyperedgeType/COMMUNITY)` from a carrier-subject
      rule fires; **(b)** `(update-hyperedge it community/heat (set …))` inside that body writes;
      **(c)** `(for-each (hyperedges-of self HyperedgeType/COMMUNITY) …)` from a SOCIAL_CLASS-subject
      rule fires and `(field-of it community/member-count)` reads inside it; **(d)** repeated
      `(scale …)` accumulates multiplicatively across a `for-each`; **(e)** a 14-arm `if`-chain over
      an enum-field equality loads and evaluates (**the only landed `if`-chain precedent is
      3-arm — `territory.bsl:130-137` — so this arm is the real evidence, not a formality**);
      **(f)** a same-tick cross-rule read of a hyperedge field written by an earlier rule sees the
      new value (§8b's fatal rows); **(g)** **NEW in revision 2:** the whole world LOADS with a
      measured `E-LOAD-040` bound on a hyperedge-querying rule — i.e. Task 4's ceiling supply chain
      reaches this scenario. **If any refuses: STOP, record the refusal text and its `E-` code, and
      re-plan §8's rule split before Task 8** — do not work around it inside a later task.
- [ ] **Step 6: The corroboration artifact** (§9) — drive the real `CommunitySystem.step()` over a
      hand-seeded `BabylonGraph` mirroring world 1, capture its output into
      `reports/community-frozen-corroboration-2026-08-18.md`, and state in both that file and the
      mirror that it is **evidence, not the oracle**. **Named obligation (C4):** the hand-seeded
      graph includes world 1's `n5` (inactive, with a membership), and the artifact records that
      node's post-`step()` attribute dict verbatim, showing `community_cost_modifier` **absent**. If
      it is present, STOP — §1.4's reading of `:472-474` is wrong and `c09`/`c10`'s guard is wrong
      with it.
- [ ] **Step 7: Load-smoke green** + the `defenum` ordinal-parity test (14 `CommunityType` members,
      3 `ConsciousnessTendency` members, in frozen order — ADR195) + **§8c's four permanent
      anti-pattern guards** (including `exactly_one_institution_carrier`), which land here and are
      never deleted.
- [ ] **Step 8: Commit** `test(tick): community conformance world + frozen mirror + system
      registration`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical.
**Estimate:** ~6h · ~85k tokens (the seven-shape spike dominates).

---

### Task 8: `c00`-`c04` — the census and the org-weight decomposition (PR C)

**Files:** Create `content/rules/community.bsl` (header + `c00`-`c04` + the `defconst` block);
extend `tests/community_conformance.rs`.

**Interfaces:** Produces `community/member-count`, the four per-class org accumulators, and
`community/{r,l,f}-raw` + `density-sum` — read by `c05`-`c08`.

- [ ] **Step 1: Failing tests** — `census_counts_only_active_members` (n5 excluded, `:472-474`);
      `orgs_with_no_members_contribute_nothing` (n9, `:421`); `org_weight_is_cadre_times_cohesion`;
      `contribution_divides_by_the_census_count`; `density_sum_counts_org_memberships_not_orgs`.
      Each asserts bit-exact against the mirror via the `.to_bits()` idiom.
- [ ] **Step 2: The pack header** — the name-collision paragraph (§2.3's `lifecycle.bsl` false
      friend), the frozen source citation with line count, the reserved `D-N` block, the
      byte-order map (§2), the §5 disclosure of what does not land, the ADR214 provenance for
      the floor table, **and the §3.7a carrier disclosure** (this pack anchors on the world's
      singleton INSTITUTION node and mints none of its own; a second carrier double-applies every
      hyperedge write).
- [ ] **Step 3: `c00` + `c01`** — the reset and the member census. `c01`'s `active` gate mirrors
      `:472-474`, and — per the solidarity precedent's D-record 6 — `social-class/active` is seeded
      on **every** node in every world, because an unwritten attribute is an honest-null load error.
      `c00`'s `:material-basis` carries the one-sentence SUBJECT-TYPE ANCHOR disclosure in
      `control-ratio.bsl:277`'s own words.
- [ ] **Step 4: `c02` + `c03-{r,l,f}`** — the per-class accumulator reset and the three
      tendency-gated org pushes. The tendency gate is a **rule-level `when`**, not a fold body:
      fold bodies are bare accessors only (D138, `control-ratio.bsl:277`), so a gated fold is not
      expressible and three rules is the shape, not a preference.
- [ ] **Step 5: `c04`** — the per-(class, community) contribution push, four `add`s.
- [ ] **Step 6: Fuel** — measure each of the **SEVEN** rules this task lands (`c00`, `c01`, `c02`,
      `c03r`, `c03l`, `c03f`, `c04` — revision 1 said "six", which under-measures one rule) by the
      declare-low/read-`E-LOAD-040`/set bound+1 cycle, against **every** world that loads the pack.
      Remember the bound is per-world because `:max-members` is derived (D-NF+22). Record the
      measured bounds in the commit body.
- [ ] **Step 7: Mutation vectors** — one per rule minimum (**seven** rules, seven vectors floor):
      drop the `active` gate; drop the no-member-org skip; swap `cadre × cohesion` for
      `cadre + cohesion`; replace the census divisor with a literal; delete `c02`'s reset (proves
      accumulators would compound across ticks — caught only by the arc world, so this vector is
      re-run in Task 10); collapse `c03r`/`c03l` into one rule (proves the tendency partition is
      load-bearing).
- [ ] **Step 8: Commit** `feat(tick): community.bsl c00-c04 — the census and the org-weight
      decomposition`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical; world 1's own pins
measured for the first time here.
**Estimate:** ~6h · ~85k tokens.

---

### Task 9: `c05`-`c08` — normalization, the ADR214 floor, entropy, the tie-break (PR D)

**Files:** Extend `content/rules/community.bsl`; create `content/scenarios/community-floor-conformance.bscn`
and `content/scenarios/community-degenerate-conformance.bscn`; extend `tests/community_conformance.rs`;
**conditionally** modify `content/rules/consciousness.bsl` (Step 5, DG-2-gated).

**Interfaces:** Produces the published community simplex, and — **only if DG-2 returns "publish"** —
contestation and dominant tendency.

**GATE ON DG-2 BEFORE STEP 4.** Steps 1-3 are unconditional. **Steps 4 and 5 (`c07`, `c08`, §8a
row 1, the `consciousness.bsl` amendment) require DG-2's answer in hand** (§2.2, I10). If it has not
arrived, land Steps 1-3 + 6-8 for `c05`/`c06` and **STOP**; do not write a theory-line edit ahead of
the gate that decides whether it is true.

- [ ] **Step 1: Failing tests** — `unorganized_fraction_defaults_to_liberal`;
      `degenerate_total_yields_floor_and_remainder`; `floor_binds_and_redistributes_proportionally`;
      `floor_redistribution_handles_zero_lf`; `settler_floor_is_identically_zero`;
      `first_nations_floor_exceeds_new_afrikan` (ADR214 Ruling 3, executed);
      `contestation_is_normalized_shannon_entropy`; `dominant_tendency_breaks_ties_liberal_first`;
      `community_without_orgs_keeps_its_prior_consciousness` (frozen `:452`'s skip gate).
- [ ] **Step 2: `c05`** — the unorganized fold into `l-raw`, the `total < 1e-10` degenerate branch,
      the normalization. **The degenerate branch emits `(0, 1, 0)` and NOTHING else** — the floor
      routes through `c06`, bit-identically (§6.2's IEEE-754 argument), so `c05` carries no floor
      constant, no `community/kind` read and no second dispatch. (Revision 1 instructed the
      opposite, citing `consciousness.py:87-89`; the degenerate assignment is `:89-91` and routing
      reproduces it to the bit.)
- [ ] **Step 3: The 14-row `defconst` table + `c06`'s dispatch — the pack's ONLY 14-arm chain** —
      values from §6.1, each with its ADR214 provenance line in a comment (including INCARCERATED's
      named unreachability and the LOW-five demotion). Both new worlds re-declare the table; the
      **cross-world parity test** lands here.
- [ ] **Step 4 (DG-2-gated): `c07`** — `(intrinsic log …)` declared once in the pack header; the
      `p > 1e-10` guard per component; the `/ (log 3)` divisor computed, not pasted.
- [ ] **Step 5 (DG-2-gated): `c08`** — the tie-break, plus **§8a row 1's copies-agree test** and the
      one-line amendment to `consciousness.bsl:354`'s "ONE DECLARED HOME" claim in the same commit,
      which also corrects that comment's own frozen anchor (`:177-192` → `:167-191`, epsilon `:189`).
      **This step is a theory-line edit to a landed pack and does not run without DG-2.**
- [ ] **Step 6: Worlds 2 and 3** — world 2 seeds NEW_AFRIKAN and FIRST_NATIONS with an org landscape
      whose normalized `r` falls **below** both floors (so both floors bind and the ordering is
      observable) plus SETTLER as the 0.0 control; world 3 seeds the degenerate case (a community
      whose only org has zero cadre) **and** the no-org skip gate.
- [ ] **Step 7: Mutation vectors** — per floor arm the worlds exercise (three), plus one converse
      proving the unexercised arms cannot fire in these worlds; the two epsilons independently
      (§8a row 4); the tie order (swap LIBERAL and REVOLUTIONARY → the tie test reds); the
      `remaining/lf` redistribution (drop the proportional scaling → world 2 reds).
- [ ] **Step 8: Fuel re-measured** for every rule (the two new worlds change the worst-case
      ceilings). **Commit** `feat(tick): community.bsl c05-c08 — normalization, the ADR214 floor
      table, entropy, the tie-break`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical; **this train's own pins
re-measured** with the per-rule-id `fired` arithmetic in the commit body.
**Estimate:** ~8h · ~110k tokens.

---

### Task 10: `c09`-`c11` + the arc + the seam world + the CARRIER-COLLISION world (PR D)

**Files:** Extend `content/rules/community.bsl`; create
`content/scenarios/community-cost-modifier-conformance.bscn`,
`content/scenarios/community-decay-arc-conformance.bscn` + `community_decay_arc_conformance.py`,
`content/scenarios/community-solidarity-seam-conformance.bscn`,
**`content/scenarios/community-carrier-collision-conformance.bscn`**,
`content/scenarios/community-empty-conformance.bscn`; create `tests/community_arc_conformance.rs`;
modify `tests/tick_goldens.rs`.

- [ ] **Step 1: Failing tests** — `cost_modifier_is_the_product_over_communities`;
      `cost_modifier_without_membership_is_exactly_one` (n4);
      **`inactive_class_receives_no_cost_modifier_write`** (n5 — asserts
      `node_attribute(n5, "social-class/community-cost-modifier")` is the substrate's honest-null
      **error**, not `1.0`, in the landed `.is_err()` shape of
      `consciousness_ternary_conformance.rs:244-245`; this is C4's detector and its mutation vector
      is "delete `c09`'s `active` guard");
      `cost_modifier_does_not_compound_across_ticks` (the `c09` reset, provable only across two
      ticks); `heat_cohesion_education_decay_independently`;
      `decay_is_monotone_non_increasing` (frozen law L4's ported half, `:221`);
      `community_tick_leaves_solidarity_strength_byte_identical` **and
      `seam_world_community_half_actually_ran`** (§2.1 — the second asserts world 5b's carrier
      rules fired and at least one hyperedge attribute moved, so the seam proof cannot pass
      vacuously);
      **`co_loaded_packs_fire_once_each_on_one_carrier`** (world 5c, §3.7a);
      **`all_inactive_members_make_the_tick_a_no_op`** (world 6, frozen L2's analogue — every rule
      loads, the carrier rules fire, `pre == post`) and
      **`a_world_with_no_community_hyperedge_refuses_at_load`** (frozen L1's analogue — a
      `MissingCeiling`/`E-LOAD-045` **refusal** test, not a golden-pinned world; §1.2).
- [ ] **Step 2: `c09` + `c10`** — reset then `scale`-accumulate (§3.5, D-NF+13), **both carrying the
      `(when (= active 1))` guard** frozen's `_collect_memberships` imposes (`:472-474`, §1.4/C4),
      with the spike's Step-5(d) verdict cited inline and the ascending-`HyperedgeId` product order
      named in `c10`'s `:material-basis` (D-NF+13's float-order divergence).
- [ ] **Step 3: `c11`** — the three decay writes; **infrastructure is absent and the header says
      why**, citing §5 and DG-7 so no reader mistakes it for an oversight.
- [ ] **Step 4: Worlds 4, 5, 5b, 5c, 6 — each seeding ONE carrier and ≥1 community** (§3.7a; a
      carrier-free world reds §8c guard 4 and a community-free world reds `MissingCeiling`).
      The cost-modifier world (which re-seeds an inactive member, so C4's detector runs in two
      worlds); the three-tick decay arc via `TickSession` (the
      `control_ratio_conformance.rs:1172-1181` idiom, with `SessionId::new("run-once")`'s
      deterministic-identity law, D179); **world 5b — the landed `solidarity-conformance.bscn`'s
      shape PLUS one `community-register` carrier and one community hyperedge with its seeded
      fields**, so all fourteen rules fire and the seam assertion compares a moved community state
      against an unmoved `solidarity/strength` (the zero-INSTITUTION fact in §2.1 is about the
      landed file, not this new one); **world 5c — `community.bsl` + `control-ratio.bsl` co-loaded
      over ONE INSTITUTION node carrying both packs' anchor fields**
      (`institution/community-carrier` AND the carceral fields `control-ratio.bsl` binds),
      asserting per-rule-id `fired` arithmetic in the `carceral_arc_conformance` style
      (`tick_goldens.rs:726-733`) and that every community's decay applied **once**; and **world 6
      — one carrier, one community whose members are all inactive classes** (§1.2's L2 analogue),
      with L1's analogue landing beside it as the load-refusal test rather than as a world.
- [ ] **Step 5: The arc mirror** `community_decay_arc_conformance.py`, per §9, with its stdout
      pasted verbatim + dated.
- [ ] **Step 6: Golden pins** — add **8** `*_hashes_are_pinned` tests, **one per content world**
      (1, 2, 3, 4, 5, 5b, 5c, 6), each pinning `before` and `after` in one test as the landed pins do
      (the arc's is its tick-3 `after`). Each carries the doc-comment triad the landed pins carry:
      what it summarizes that the conformance suite already pins; "Measured, never derived… new in
      this train, so this is a measurement, not a ceremony (III.13 applies to `tests/baselines/**`,
      not this crate's own goldens)"; and the confirmation it touches none of the 16 prior pins.
      (Revision 1 said "7 tests, one per world, two for the arc" over a seven-world list — arithmetic
      that could not close.)
- [ ] **Step 7: Mutation vectors** — the `c09` reset (Task 8 Step 7's deferred vector, now provable);
      **`c09`'s `active` guard** (delete it → `inactive_class_receives_no_cost_modifier_write` reds —
      the C4 vector); each decay α independently (§8a row 2); the `max(0, ·)` clamps with a converse
      vector each; **a second INSTITUTION node in world 5c** (add one → `exactly_one_institution_carrier`
      reds first, then the `fired` arithmetic); **remove world 5b's carrier** (→ guard 4 reds AND
      `seam_world_community_half_actually_ran` reds, proving the seam proof cannot pass vacuously);
      **activate one of world 6's members** (→ `all_inactive_members_make_the_tick_a_no_op` reds).
- [ ] **Step 8: Commit** `feat(tick): community.bsl c09-c11 — cost modifiers, state decay, the arc,
      and the carrier-collision world`.

**Gate:** six cargo legs, **single-flight**; the 16 pins byte-identical; this train's pins
re-measured.
**Estimate:** ~6h · ~90k tokens.

---

### Task 11: The #653 co-sponsored membership design note, and the blocked half filed (PR D)

**Files:** Create `reports/community-membership-shape-note-2026-08-18.md`; issues only otherwise.

**This task writes a design note and files issues. It implements nothing.**

- [ ] **Step 1: The co-sponsored shape note** — what Community needs from AG(i), stated as
      requirements against `#653`'s ceremony: three payload fields (`role` as an **int-ordinal**,
      D102 kills the `defenum` route per the port-estate survey; `strength` as a coefficient;
      `visibility` as a probability, plus the `overt` override that makes `effective_visibility`
      `1.0`, `entities/community.py:407-417`); ascending-member-id iteration (AG(i)'s own
      obligation); hash participation; and the observation ADR198 R4 already recorded — **the
      node-local list-of-structs shape may not be AG(i)'s shape**, with this train's verdict on
      whether the two consumers' needs (Community's three fields vs Electoral's open-cardinality
      `allegiance` map) want one mechanism or two.
- [ ] **Step 2: Post the note** to #653 and #536, and link it from #664.
- [ ] **Step 3: File the blocked-half issue** — threat scoring, solidarity amplification and
      infrastructure maintenance, with §5's table as the body, `#653` as an explicit blocking
      dependency, and "cannot open until #653 lands" as its gate.
- [ ] **Step 4: Commit** `docs(port): community membership shape note — the #653 co-sponsored
      requirements`.

**Gate:** none (docs/issues) — `vale` over the note.
**Estimate:** ~2h · ~30k tokens.

---

### Task 12: Records, docs, gates, handoff (PR D)

**Files:** Modify `docs/reference/bsl-language.rst`, `content/rules/community.bsl` (finalize the
file-local `D-N` block); create `ai/decisions/ADR-NF_community_port_handoff.yaml` +
`ai/decisions/index.yaml` row; modify `ai/state.yaml`.

- [ ] **Step 1: RE-MEASURE the tails one more time** (Task 0 Step 2's allocation may have been
      overtaken by the ImperialRent or #491 trains) and register **twenty-five** rows at whatever is
      free — the D-record table below, in order.
- [ ] **Step 2: Amend the spec's hyperedge sections** — §2.6's three heads move from "slice 3" to
      served; §2.8's `update-hyperedge` and §2.10's `field-of` notes move from "no storage" to
      served; `membership-field-of` / `update-membership` / `deffield :member` / family-23 stay
      unserved and now cite **#653** by number. **Then run**
      `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest tests/unit/reference/test_bsl_grammar_sync.py -q`.
- [ ] **Step 3: ADR-NF** — records: the two-estate structure and the E1/E2/E3 split with §3.3's
      sizing verdict **as measured, not as claimed** (engine ≈ half the train); **the three declared
      boundary crossings** (the `GraphSubstrate` enumerator, `LoadedScenario.hyperedge_types`, the
      `babylon-tick` driver ceilings) and the §3.6 amendment that sanctioned the third; the III.7
      declaration for section `0x06` **with the elision proof attached**, the sixth `CanonicalState`
      listing, and DG-8's disposition; **the `Element` cross-kind Ord ruling** (`Node < Edge <
      Hyperedge`) as the discharge of rider 4's A5 precondition, quoting the rider in full;
      **`:max-members` derived from the seeded population**, with its argument and its fuel
      consequence; **§3.7a's singleton-carrier verdict** and world 5c as its executed proof; the
      four-of-six-phase scope and the §5 blocked half with #653 named; the ADR214 floor table's
      landing (values, provenance, the parity test), the **superseded-not-met status of Ruling 4's
      stated trigger** (§6.3), and the **deferred §6.5 ceremony's disposition — NOT FIRED, with the
      `qa:regression` / `qa:vault-regression-ci` evidence**; §2.1's producer seam and §2.2's
      duplication verdict **with DG-2's answer recorded as the thing that decided it**; the D116
      reliance table as the Q14 train's acceptance-criterion input; the eight DIRECTOR GATE
      questions with their dispositions (open, or answered and how); and the gate evidence.
- [ ] **Step 4: Full gates, once, single-flight** — `mise run rust:check`; `mise run check`;
      `mise run qa:regression`; `mise run qa:vault-regression-ci`; the grammar-sync pytest; `vale`
      over every touched Markdown/RST. **Nothing under `tests/baselines/**` may move**; record that
      it did not, since that recording is what discharges ADR214 Ruling 4's deferral.
- [ ] **Step 5: Issue hygiene** — close the Task 0 issue with evidence; update the Checkpoint A
      tally **honestly**: Community @6.0 is partially ported (four of six phases), Checkpoint A is
      **not** reached by this train, WS3 stays HELD, and DG-1 is the open question about how @6.0
      counts. Post the DIRECTOR GATE list to the docket issue (#564) as a popup-ready block.
- [ ] **Step 6: `ai/state.yaml` closing entry** + commit `docs(p27): community port handoff —
      the hyperedge lane live, the ADR214 floor table, the #653 blocked half`.
- [ ] **Step 7: Open PR D.** Review lens: (a) the floor table read as a **theory** artifact — ADR214's
      values, provenance and ordering, transcribed exactly; (b) the DIRECTOR GATE section read as a
      Director-facing artifact — DG-2 answered before the code that depends on it, the rest stated
      as open; (c) the **twenty-five** register rows read as a set — every divergence has exactly one
      row and every row names its re-open trigger. Harvest Copilot; merge via
      `mise run pr:merge -- N`.

**Estimate:** ~4h · ~60k tokens.

---

## PR structure — FOUR PRs, and why the split falls where it does

| | branch | tasks | commits | contents | review lens |
|---|---|---|---|---|---|
| **PR A** | `feature/community-port-bsl` (worktree exists) | 0-4 | 5 | the dossier; hyperedge scenario seeding + the population maps; **the `GraphSubstrate` type-scoped enumerator**; the three query heads + `Element::Hyperedge` + the Ord ruling + the first hyperedge typecheck logic; **the driver's ceiling supply chain** | **the REACH lane** — served/unserved tables (both), iteration order, the cross-kind Ord pin, and loadability. Spans `babylon-graph` (one read-only accessor), `babylon-bsl` and the `babylon-tick` driver; **no hash surface touched — `CanonicalState` is untouched in this PR** |
| **PR B** | `feature/hyperedge-attribute-storage`, off **merged dev** | 5-6 | 2 | section `0x06` + the sixth `CanonicalState` listing + both backends + conformance rows + the elision proof; `update-hyperedge` executing in both dispatch sites; `field-of` over a `HyperedgeRef`; `(hyperedge-attr …)` seeding | **substrate widening** — Constitution III.7, save-compat, the elision proof, and DG-8. A reviewer reading `state_hash.rs` and ADR203 side by side |
| **PR C** | `feature/community-content-spine`, off **merged dev** | 7-8 | 2 | registration; world 1; the mirror; THE SPIKE (seven shapes); `c00`-`c04`; §8c's **four** permanent anti-pattern guards | **transcription fidelity + the anti-pattern guards** — is the census frozen's census, can a community still never be a node, and is there still exactly one carrier? |
| **PR D** | `feature/community-consciousness-laws`, off **merged dev** | 9-12 | 4 | `c05`-`c11`; worlds 2-6 incl. **5c**; the ADR214 floor table; the arc; the seam and collision worlds; **8** golden pins; the #653 design note; D-rows, ADR-NF, handoff | **theory + governance closure** — the floor table as a political artifact, DG-2's answer and what it decided, the register rows as a set |

**Why four, not two.** PRs A and B are **engine** work with two different failure modes — a wrong
served-head table or a starved ceiling map fails loudly at load, while a wrong hash section fails
silently everywhere until a golden moves months later. Bundling them would put the III.7 widening in
a diff whose reviewer is reading query semantics. PR A now spans three crates, which is a real cost
paid deliberately: its three pieces (enumerate → query → supply the ceiling) are **one dependency
chain that is untestable in halves** — a served `(hyperedges …)` head with no enumerator cannot
materialize, and a materializing head with no ceiling cannot load. Splitting it would produce two
PRs neither of which can prove anything. PRs C and D are **content**, and they split at the point
where the pack stops being arithmetic and starts being political: PR D carries the ruled floor
values, the (DG-2-gated) second home for the tie-break, and every Director-facing question.

**The dependency direction is clean.** A: enumerate + seed + query + load. B: storage on top of A's
forms. C: content that reads A and writes B. D: content that reads C's state. **No earlier PR's rule
reads anything a later PR writes.** What is *not* true is that PR C's pins survive PR D — they will
move, by construction, and Task 9 Step 8 / Task 10 Step 6 re-measure them with the `fired`
arithmetic.

**Never stacked** (#193). Each PR branches off merged `dev`.

---

## Worlds / conformance matrix

**Eight content worlds** (1, 2, 3, 4, 5, 5b, 5c, 6) plus the engine world E-1 — the count revision 1
gave as "six" in its header and "seven" in its own tables. **Eight golden pins**, one per content
world, each pinning `before` + `after` in a single test. **Every content world seeds exactly ONE
`NodeType/INSTITUTION` carrier and at least one `HyperedgeType/COMMUNITY` hyperedge** — the first
because six rules are carrier-subject (§3.7a, §8c guard 4), the second because a type declared zero
of gets no ceiling and the pack then fails at load (§1.2, D-NF+22).

| world | file | proves | rules exercised | mirror | golden pin |
|---|---|---|---|---|---|
| **E-1** | `babylon-bsl/tests/hyperedge_lane_e2e.rs` (+ `babylon-graph`'s `conformance.rs` rows and `babylon-tick`'s `hyperedge_ceilings.rs`) | enumeration on both backends, seeding, id-order independence, the three heads' iteration order, the `Element` cross-kind Ord, ceiling supply + measured bounds, own-field round-trip, the elision | — (engine) | — | — (crate tests) |
| **1** | `community-conformance.bscn` | the census, the active gate, the no-member-org skip, the org-weight decomposition, a non-binding floor, **the inactive class's ABSENT cost modifier** | all 14 | `community_conformance.py` | pre + post |
| **2** | `community-floor-conformance.bscn` | the floor **binds**; proportional redistribution; the measured FIRST_NATIONS > NEW_AFRIKAN ordering (ADR214 R3); SETTLER identically 0.0 | `c00`-`c08` | world 1's mirror, second `WORLD` | pre + post |
| **3** | `community-degenerate-conformance.bscn` | `total < 1e-10` → `(0,1,0)` in `c05`, floor + remainder after `c06`; the zero-`lf` redistribution arm; the no-org **skip gate** (`:452`) | `c00`-`c08` | world 1's mirror, third `WORLD` | pre + post |
| **4** | `community-cost-modifier-conformance.bscn` | the product over multiple communities in ascending `HyperedgeId` order; **exactly 1.0** for the unaffiliated class; **no write at all** for an inactive one | `c00`-`c01`, `c09`-`c10` | world 1's mirror, fourth `WORLD` | pre + post |
| **5** | `community-decay-arc-conformance.bscn` | three ticks of heat/cohesion/education decay; monotone non-increase (frozen L4's ported half, `:221`); no accumulator compounding | all 14, ×3 ticks | `community_decay_arc_conformance.py` | pre + tick-3 post |
| **5b** | `community-solidarity-seam-conformance.bscn` | §2.1 — the community half runs **LIVE** (all 14 rules fire, hyperedge attributes move) and `solidarity/strength` is still **byte-identical**. One carrier + one community, like every world here; it proves the attribute seam and not the carrier one, because `solidarity.bsl` runs no INSTITUTION-subject rule | all 14 + `solidarity/p0-transmit` | (asserted, not mirrored) | pre + post |
| **5c** | `community-carrier-collision-conformance.bscn` **(new, §3.7a/C3)** | ONE shared INSTITUTION carrier with both packs' anchor fields: each pack's carrier rules fire **once**, every community's decay applies **once**, and no III.11 absence error fires on the foreign fields | `community/*` + `control-ratio/c01`-`c04` | (asserted, `fired` arithmetic) | pre + post |
| **6** | `community-empty-conformance.bscn` | **frozen L2's analogue** — one carrier, one community whose members are all INACTIVE classes: every rule LOADS and the carrier rules FIRE, and the tick is still byte-exact `pre == post` on every community field (§1.2). "Communities with no members" is unrepresentable (`memory.rs:357-361`), and **L1's analogue is a separate refusal test**: a scenario with no community hyperedge fails `MissingCeiling` at load | all 14 (carrier rules fire, write nothing) | — | pre == post |

---

## D-record table (register rows, allocated **next-free-at-landing** at Task 12 Step 1)

| row | subject | one-line rationale |
|---|---|---|
| D-NF+1 | The ephemeral XGI rebuild collapses into the substrate | Frozen rebuilds a throwaway hypergraph every tick (`:346`); in Rust the hypergraph IS the graph (Amendment D), so communities are seeded once and persist — a structural reformulation, not an optimization |
| D-NF+2 | The `community-register` carrier is not a community node | The world's singleton INSTITUTION registry gives carrier-subject rules a subject; the `community/` namespace is banned from `:field` bindings because `subject_type_of` would demand `NodeType/COMMUNITY` — the mechanical face of VIII.9. (The *singleton* half is D-NF+23's.) |
| D-NF+3 | The org-density aggregate decomposes into per-class pushes | `w = density × cadre × cohesion` is linear in density, so the frozen per-org sum equals the port's per-class sum exactly; only the FP summation ORDER differs, and world 1 measures it |
| D-NF+4 | The D116 same-tick cross-rule reliance | Five reads (§8b), two of them fatal if collect-across-rules lands; recorded as the Q14 train's acceptance-criterion input for this pack |
| D-NF+5 | The 500-org cap does not port | No early-break construct exists, and the frozen cap truncates in insertion order (non-deterministic); the port's bound is fuel, which refuses loudly instead of truncating silently |
| D-NF+6 | The floor table is a `defconst` table + **exactly one** 14-arm dispatch | ADR214 R4 ruled the entry path; no map/lookup construct exists, so dispatch is the shape; it lives once, in `c06`, because `c05`'s degenerate branch routes through it bit-identically (`×1.0`/`÷1.0` are exact); the cross-world parity test is what keeps 14 ruled values from drifting across eight worlds |
| D-NF+7 | Shannon entropy lands on the `log` intrinsic | ADR213's named Community consumer; `log(3)` is computed, not pasted, so the division's bytes match frozen; the `1e-10` per-component guard transcribes verbatim |
| D-NF+8 | A second declared home for the hegemonic tie-break **(only if DG-2 says publish)** | `consciousness.bsl:354` claims exactly one; the community simplex needs the same readout over a different subject and enum — re-homing was declined on subject-type grounds, so §8a's copies-agree row plus an amendment to that comment is the price. **The row is void if DG-2 declines**, and the plan does not pre-decide it |
| D-NF+9 | Unread published state is not declared | Frozen publishes `visibility`, `rent_access_modifier` **and `category`** (`community.py:99,102,104`) onto the hyperedge but reads none of them in `step()`; declaring them — or transcribing `HyperedgeCategory` for `category`'s sake — would enter the tick hash write-only, against declare-what-you-read |
| D-NF+10 | The E3-gated half, itemized | Threat score, solidarity amplification and infrastructure maintenance each need per-membership payload (AG(i), #653); §5's table is the row's body |
| D-NF+11 | The off-`step()` estate is not ported | The four repression helpers, `community_overlap_matrix`, `communities_spanning_axis`, and `calculate_solidarity_potential` are never called from `step()`; they await a verb layer, and `solidarity_potential`'s two defines stay undeclared |
| D-NF+12 | The `solidarity_strength` producer seam stays quiet | The port does not amplify, so `solidarity.bsl` reads exactly what it reads today; world 5b proves non-interference, and #653 is the re-open trigger |
| D-NF+13 | Cost modifier via reset + repeated `scale`, **and its multiplication ORDER** | No `product` fold-op exists (`grammar.rs:672-683`); the two-rule reset/accumulate split is `production.bsl`'s landed idiom, and the compounding hazard is provable only across two ticks. **The divergence this row also carries:** frozen multiplies in the agent's membership-list order (`formulas/community.py:166-174`); `c10` multiplies in ascending `HyperedgeId` (the ruled order, `substrate.rs:255-256`). Float multiplication is not associative, so this is the same class of divergence D-NF+3 records for the sums — measured by world 4, never assumed away |
| D-NF+14 | `CanonicalState` section `0x06` | A III.7 widening mirroring ADR198 R1/ADR203, with the caller-side empty elision that keeps all 16 landed pins byte-identical — the proof, not the promise |
| D-NF+15 | Hyperedge scenario seeding, and why no rule mints one | `(hyperedge …)` extends `scenario.rs`'s id-order law with an independent counter; `DEFERRED_SHAPE_VERBS` stays intact because the port needs no minting verb — #536's rider 1 is unspent |
| D-NF+16 | One `HyperedgeType` member + a `community/kind` enum field | Type-as-identity would force 14 copies of every uniform law, since `(hyperedges …)` is type-scoped |
| D-NF+17 | `community-cost-modifier` is a dead write, ported anyway | Frozen declares and writes it with zero readers anywhere in `src/`; ported because it is frozen's observable output, with the reproduction-cost consumer named as the re-open trigger |
| D-NF+18 | The pack emits no events and reads no `TickContext` | Frozen emits nothing (whole-file grep) and names its context parameter `_context`; the absence is declared so a later reader does not assume an emitter was dropped |
| D-NF+19 | Memberships are UNATTRIBUTED in this train | No payload exists, so the worlds carry no role/strength/visibility columns — **deliberately absent, never defaulted**, since a defaulted role would fabricate the exact data #653 exists to carry |
| D-NF+20 | The duplication ledger | Four expressions transcribed more than once (§8a); single-sourcing is unavailable in the language, so each pair owes a copies-agree row and a perturb-one-copy vector |
| **D-NF+21** | **The type-scoped hyperedge enumerator widens `GraphSubstrate`, not `CanonicalState`** | `(hyperedges …)` had no accessor at all: `nodes`/`edges` are type-keyed ranges, nothing hyperedge-side was (`substrate.rs`, `grep -n 'fn '`). Routing through `CanonicalState::all_hyperedges` was rejected on that trait's own ruling (`state_hash.rs:294-299` keeps its listings a serialization capability, off "the structural-verb surface Amendment D ratified"), so the substrate gains a symmetric `hyperedges(hyperedge_type)` with a `conformance.rs` row on both backends. **A declared crate-boundary crossing (§3.6), tasked separately** |
| **D-NF+22** | **`:max-members` and the `HyperedgeType` ceiling are DERIVED from the seeded population** | The bound-checker axis was landed and starved: the driver passed an empty `max_members` map (`babylon-tick/src/lib.rs:263-276`), so every hyperedge-querying rule failed at LOAD. Both maps now come from `LoadedScenario`, on `node_types`/`edge_types`' own stated argument ("*the population the scenario ACTUALLY built … rather than an invented one*", `scenario.rs:242-248`). **Chosen over a content `manifest` row** because no landed content declares a manifest and an invented cap would decide, invisibly, whether a modder's scenario loads (#531's modding line; the gameplay-and-pedagogy compass). **Consequence, accepted:** a rule's static bound is per-world, so `:fuel` is the max over every world that loads the pack and a later larger world reds the load loudly. **Re-open trigger:** the first content set needing a cap larger than any world it ships with |
| **D-NF+23** | **The singleton INSTITUTION carrier is an estate-wide invariant; this pack mints no second one** | `subject_type_of` makes every INSTITUTION-subject rule iterate every INSTITUTION node (`tick.rs:166-189`) and an unwritten bound field is a III.11 hard error (`:212-216`), so a second carrier double-applies this pack's hyperedge writes AND breaks the landed carceral packs in any co-loaded world. The landed estate already assumes one (`carceral-arc-conformance.bscn:229`; `tick_goldens.rs:716,730` pin "one carrier" arithmetically). World 5c executes the co-load; §8c row 4 guards it. **Re-open trigger:** a design needing two carriers needs a subject SELECTOR the language does not have — an escalation, not a fixture change |
| **D-NF+24** | **`Element::Hyperedge` and the cross-kind Ord ruling** | Serving `hyperedges`/`hyperedges-of` mints the variant `query.rs:17` deliberately withheld, putting a third kind into an ordering `query.rs:56-63` carries a standing instruction about (D140, CT4P A5 / #525 — rider 4's first clause). **RULED: `Node < Edge < Hyperedge`, by declaration order — arbitrary, deliberate, tested**, with a companion test beside `node_sorts_before_edge_regardless_of_id` and the enum's doc amended in the same commit. The rider's "the A5 pin precedes them" is discharged **in-train**, at the moment the variant lands |
| **D-NF+25** | **Frozen's tendency-less-organization skip has no port analogue** | `community.py:405-407` skips an ORGANIZATION whose `consciousness_tendency` is `None`; `ConsciousnessTendency` has exactly three members and a BSL enum field must carry one, so `c03-{r,l,f}`'s guards partition every org and a tendency-less org is **inexpressible**. Same class as D-NF+5's cap: a divergence by inexpressibility, recorded rather than absorbed. **Re-open trigger:** any future optional-enum field mechanism (an `:optional` enum with a declared absent state) |

---

## Estimate

**13 tasks · 13 commits · 4 PRs · ~69 agent-hours** — a **point estimate**, being the exact sum of
the per-task figures, not a range with an underived lower bound (revision 1 wrote "~46-58h … ~58h
upper, ~52h midpoint", where 58 was the sum and 46 had no derivation anywhere):

| task | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | **Σ** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| hours | 3 | 5 | 4 | 7 | 4 | 7 | 7 | 6 | 6 | 8 | 6 | 2 | 4 | **69** |
| tokens (k) | 45 | 70 | 55 | 95 | 55 | 90 | 90 | 85 | 85 | 110 | 90 | 30 | 60 | **960** |

**~960k tokens**, plus **four** Copilot-harvest review cycles. Commits: one per task = **13**
(PR A 5, PR B 2, PR C 2, PR D 4 — the PR table's column sums to 13, which revision 1's did not).

**Where the weight sits.** Engine lane (Tasks 1-6) = 5+4+7+4+7+7 = **34h**. Content and governance
(Tasks 7-12) = 6+6+8+6+2+4 = **32h**. Task 0 = 3h. So the engine lane is **49%** of the train —
**half, not a majority**, and revision 2 says so rather than repeating revision 1's "the content pack
is the smaller half", which its own hours refuted (engine was 41% there). The point that survives is
the one that matters: Community's subject **does not exist in the language**, so half this train
builds the lane rather than walking it, and any estimate treating it as "another content port like
Solidarity" is wrong by roughly a factor of two.

**Growth against revision 1, itemized** (so a reviewer can audit the increase rather than trust it):
`+4h` Task 2 (the `GraphSubstrate` enumerator, C1), `+4h` Task 4 (the ceiling supply chain, C2),
`+1h` Task 1 (the `hyperedge_types`/`max_members_seen` maps), `+1h` Task 5 (the sixth
`CanonicalState` listing + conformance rows), `+1h` Task 10 (world 5c + its pin, C3). Total
`+11h` over revision 1's 58h sum.

**Highest-variance step: Task 7 Step 5's spike — seven shapes**, four of which (a hyperedge
`for-each` body, an `update-hyperedge` from content, `scale` accumulation, a 14-arm enum dispatch
where the landed precedent is 3-arm) have **no landed content precedent at all**. If any refuses,
§8's rule split is re-planned before Task 8, not worked around inside it.

**Second-highest: Task 5's elision proof.** Its failure mode is the quiet one — a section that
hashes correctly for new worlds and subtly wrong for empty ones would pass every new test and move
a landed pin only under a condition no test in this train exercises. The 16 pins are the gate for
exactly that reason.

**Third: the DG-2 dependency.** It is the only Director answer this train's CODE waits on, and it
arrives (or does not) on someone else's clock. Task 0 Step 1 posts it on day one for exactly that
reason; the fallback is a 12-rule pack, stated in §2.2 so nobody improvises one.

---

## Self-review notes (plan author)

- **The finding that reshaped this plan** is `scenario.rs:63-64` — "**No hyperedges yet. The
  grammar has room for them; nothing in slice 1 needs one, and an unused form is an untested
  form.**" Every earlier framing of #536 treated the hyperedge lane as "parked verbs to unpark".
  It is narrower and wider than that at once: **narrower**, because the port needs no minting verb
  (the scenario seeds, the rules read — rider 1 is unspent, which is the single biggest scope
  saving here); **wider**, because there is no seeding form, no query head, no attribute storage and
  no `field-of` — four absences, not one deferral.
- **What revision 2 adds to that finding, and it is the harder half:** the absences do not stop at
  the language. **There is no substrate ENUMERATOR** for `(hyperedges …)` (`GraphSubstrate` offers
  type-keyed ranges for nodes and edges and nothing for hyperedges), and the **fuel axis that DOES
  exist is starved by the driver** (an empty `max_members` map, no `HyperedgeType` ceiling), so the
  pack would not have failed at a rule — it would have failed at LOAD, on every world, with an error
  revision 1's task list had no owner for. Seven absences across three crates, not four in one.
  A charter that assumed the gap was `babylon-bsl`-shaped would have discovered a `babylon-graph`
  requirement inside a task forbidden to make one.
- **The second finding worth a reviewer's attention** is `babylon-bsl`'s `tick.rs::subject_type_of` +
  `namespace_to_node_type`: a `community/`-namespaced `:field` binding **mechanically instructs the
  engine to look for `NodeType/COMMUNITY` nodes**. The theory line "communities are never graph
  nodes" has a trapdoor in the engine, and it opens by naming a field the obvious way. §8c's second
  guard exists because of it — **and its third-order consequence, found only in revision 2, is that
  the same mechanism makes the INSTITUTION carrier a SINGLETON across the whole estate** (§3.7a):
  every carrier rule iterates every carrier node, so a second one silently doubles this pack's writes
  and loudly breaks the landed carceral packs. §8c's fourth guard and world 5c exist because of that.
- **Every construct this plan relies on is either landed and cited, or built by a named task.**
  Landed: `(nodes …)`/`(neighbors …)` as served heads (`evaluator.rs:567`); the INSTITUTION-subject
  carrier idiom (`decomposition.bsl:273-279`, `control-ratio.bsl:277-289` — **not**
  `decomposition.bsl:254,266,269,323`, which is carrier READS from class-subject rules, a different
  idiom); reset-then-accumulate (`production.bsl::p0`); the push idiom (D136,
  `consciousness.bsl:243-245`); `if`-chains as dispatch (`territory.bsl:130-137` — **3-arm**, so this
  one is a weaker precedent than revision 1 claimed); `:optional :default` bindings; the closed
  `add|sub|set|scale` update-op set (`grammar.rs:718`); the five-member `FoldOp` set
  (`grammar.rs:672-683`); the hyperedge fuel axis (`bound_checker.rs:544-572`); `(intrinsic log …)`
  (`declarations.rs:125`, `intrinsic_host.rs:196`); `TickSession` multi-tick driving
  (`control_ratio_conformance.rs:1172-1181`). Built here: the substrate enumerator (Task 2),
  hyperedge seeding + population maps (Task 1), the three query heads + `Element::Hyperedge`
  (Task 3), the ceiling supply chain (Task 4), own-field storage (Tasks 5-6).
- **Three capability risks, not one.** (a) A `for-each` body targeting a hyperedge — the whole pack
  shape depends on it and nothing landed does it. (b) `scale` accumulation semantics — `c10`'s shape
  depends on the answer and no landed pack `scale`s repeatedly in one tick. (c) The 14-arm dispatch
  — expressible in principle, precedented only at 3 arms, unmeasured in fuel. Task 7 Step 5 converts
  all three into evidence before fourteen rules depend on them; a reviewer should check the spike
  landed as a real spike and not as a comment.
- **What this plan deliberately does NOT do:** implement any part of AG(i) (#653's ceremony — no
  `update-membership`, no `membership-field-of`, no `deffield :member`, no payload hash section, no
  `hypergraph-rs` change); lift the `DEFERRED_SHAPE_VERBS` gate; serve `metric-of` or the `the`
  head; port threat scoring, solidarity amplification or infrastructure maintenance (§5); build the
  `community_memberships` writer (#664, DG-6); rule the pole shape or register `county_extraction`'s
  `BoundOpposition` (DG-5); decide whether Community counts as ported for Checkpoint A (DG-1);
  repair any frozen defect (§1.6 transcribes three and defers one to the blocked half); or declare
  a single field, constant or intrinsic no rule in this pack reads.
- **Numbers this plan asserts that the implementer must re-derive, not trust:** every `:fuel` figure
  (measured, never guessed — and **per-world**, because `:max-members` is derived from the seeded
  population, D-NF+22, and because Task 4 must land before any of them means anything); every
  `report.fired` count; every hash; every arithmetic result in §1 and §6 (they come from the mirrors,
  and the mirrors are the contract); and both numbering tails, which are contended by two other live
  trains and get re-measured twice.

---

## What revision 2 changed

Revision 1 was reviewed adversarially (5 Critical, 14 Important, 14 Minor). Every Critical and
Important is resolved below; the Minor triage is appended to the critique file as a disposition
table. Where revision 1 asserted something the code refutes, revision 2 says so in place rather than
quietly correcting it — the plan's own port-as-is discipline, applied to the plan.

**Criticals**

- **C1 — `(hyperedges …)` had no substrate accessor.** `GraphSubstrate` offers type-keyed ranges for
  nodes (`:204`) and edges (`:208`) and nothing hyperedge-side; `CanonicalState::all_hyperedges` is a
  different trait that `state_hash.rs:294-299` deliberately keeps off the structural-verb surface.
  Serving the head is therefore `babylon-graph` work: **new Task 2** (trait method + both backends +
  a `conformance.rs` row), §3.2 row 0, §3.7's rejected-alternative row, D-NF+21 — a **declared**
  boundary crossing under §3.6's own re-plan rule, with PR A re-scoped to span three crates and
  hours raised to match.
- **C2 — the ceiling supply chain was missing and Task 2 Step 4's premise was false.** The
  bound-checker axis is LANDED (`ceiling_of_query` `:544-572`, `fuel.rs:112-118`,
  `manifest.rs:11-15`); what is missing is the SUPPLY — `LoadedScenario` has no `hyperedge_types`
  (§3.2 row 9) and the driver passes `HashMap::new()` as `max_members` (§3.2 row 10,
  `babylon-tick/src/lib.rs:263-276`), so the pack could not have loaded at all. **New Task 4** owns
  it, Task 1 Step 4 supplies the maps, §3.6's "one registration string" cap is **amended openly**,
  and **D-NF+22 declares the `:max-members` choice** — derived from the seeded population, argued
  from the landed `node_types`/`edge_types` law and the gameplay-and-pedagogy compass (a modder's
  world must not be silently capped), with its per-world fuel consequence and re-open trigger stated.
- **C3 — a second INSTITUTION carrier collides with the landed packs.** Resolved by **reuse**, on
  evidence (§3.7a): `subject_type_of` iterates every INSTITUTION node and III.11 punishes the
  foreign pack's bind, and `carceral-arc-conformance.bscn:229` / `tick_goldens.rs:716,730` show the
  estate already assumes a singleton. This pack mints no carrier. The non-interference world is
  re-aimed: world 5b (solidarity, **zero** INSTITUTION nodes) keeps only the attribute seam it can
  actually prove, and **new world 5c** co-loads `control-ratio` over ONE shared carrier with
  per-rule-id `fired` arithmetic. Plus §8c guard 4 and D-NF+23.
- **C4 — `c09` wrote a field frozen never writes.** Resolved by **exact-port fidelity**, not a
  divergence: `_collect_memberships` (`community.py:472-474`) skips inactive nodes before
  `agent_memberships` exists, so `c09`/`c10` now carry the same `(when (= active 1))` guard `c01`
  carries, and no D-row is owed. The detector the old design lacked is named: world 1's `n5` is
  asserted **absent** (`node_attribute` → honest-null error, III.11), the mirror models it as
  unwritten rather than defaulted (§9 point 5), the mutation vector is "delete the guard", and the
  frozen corroboration artifact carries the independent half (§9: seed an inactive member, record
  that frozen writes nothing for it, STOP if it does).
- **C5 — rider 4's first clause was dropped.** The rider is now **quoted in full** in §0, both
  clauses answered: clause 1 (CT4P #525's A5 Element-Ord pin) is **discharged in-train** — Task 3
  declares `Element::Hyperedge` third, rules `Node < Edge < Hyperedge` in the enum's own standing
  instruction (`query.rs:56-63`, D140), lands the companion test beside
  `node_sorts_before_edge_regardless_of_id`, amends `query.rs:17`, and takes **D-NF+24**; clause 2
  (WS2 duality) keeps `members-of`/`hyperedges-of` landing together.

**Importants (all 14 resolved)**

- *Counts and arithmetic (I1, I2, I3, I4):* the pack is **14 rules** across 12 rows (12 if DG-2
  declines) with the seven-rule fuel measurement in Task 8 corrected; **eight content worlds** and
  **eight golden pins**, one per world; the sizing verdict restated as **49% engine, half not
  majority**, with the per-task table shown; **13 tasks · 13 commits**, the PR column summing to 13,
  and the estimate a point sum (~69h) with its `+11h` growth itemized.
- *Engine-surface facts (I6, I13, I14):* both unserved-head tables named (`query.rs:99-103` and
  `evaluator.rs:544-551`) plus `SERVED_QUERY_HEADS`; `babylon-graph/src/conformance.rs` added to the
  storage task's files with a row per new accessor; the storage shape fixed to
  `HashMap<(HyperedgeId, String), f64>` with **the sort belonging to `encode_state`** (the
  BTreeMap-vs-mirror-T3 contradiction removed), and the sixth REQUIRED `CanonicalState` listing added.
- *Transcription fidelity (I5, I7, I8):* `c05`'s degenerate branch **routes through `c06`**, proved
  bit-identical (`×1.0`/`÷1.0` exact), removing an undeclared second 14-arm dispatch and fixing the
  `:87-89` → `:89-91` citation; the float **product**-order divergence joins D-NF+13; frozen's
  tendency-less-org skip takes **D-NF+25**.
- *Governance (I9, I10):* DG-7's evidence corrected — L4 pins the clamp, the non-monotonicity is a
  documented **caveat** (`test_law_community_system.py:49-52`), and the deferral stands on ADR183
  alone; §2.2 no longer both decides and asks — the second-home question is a **recommendation**,
  DG-2 decides, Task 9 Steps 4-5 are **gated** on it, and the fallback (12 rules, simplex only) is
  named.
- *Citations (I11, I12):* `solidarity/strength`'s real read anchors (`:186,192,196,201,217,225,236,
  242,260,263`) replace `:116,171`; `HyperedgeCategory` is **removed** from vocabulary discipline
  (no reader) and `category` joins D-NF+9's unread-published list.

**Everything else:** anchors throughout §1, §2, §3, §6, §8 and the self-review re-measured at the
byte on 2026-08-18 (the formula block, the decay sites, the argmax property, the entropy function,
`consciousness.bsl`'s `eps`, the two `1e-10` epsilons, the test-file line counts, the
`structural_verbs` collect path, `ARITIES`, the carrier idiom, the `if`-chain precedent); the
`E-LOAD-023` prescription corrected to **`E-LOAD-031`**; the one numbering literal ("D197") removed;
ADR214 Ruling 4's **superseded-not-met** trigger recorded; and five new D-rows (D-NF+21…25) bring the
register set to **25**.

### Revision 2.1 — the one residual the re-verify found (N1), plus its knock-on

The rev-2 re-verify returned 0 Critical / 1 Important / 9 Minor. The Important was **self-inflicted
by the C3 fix**: world 5b inherited "zero INSTITUTION nodes" from the LANDED
`solidarity-conformance.bscn`, but world 5b is a NEW world that loads this pack, so §8c guard 4
("exactly ONE in every world this pack loads into") would have redded at Task 10 — and the seam
oracle would have run with all six carrier rules **inert**, making "nothing moved" a vacuous proof.

- **World 5b now seeds one `community-register` carrier and one community hyperedge**, like every
  other world here — corrected at all four specification sites (§2.1, §3.7a, File Structure, worlds
  matrix) plus Task 10 Step 4. Its "rules exercised" column moves from `community/*` to **all 14 +
  `solidarity/p0-transmit`**, and a new assertion `seam_world_community_half_actually_ran` plus a
  new mutation vector (remove the carrier → guard 4 **and** that assertion red) make the vacuity
  failure mode detectable rather than merely avoided. The zero-INSTITUTION measurement is re-stated
  as a fact about the landed file, and §2.1 now gives the *real* reason 5b cannot detect the carrier
  collision: `solidarity.bsl` is one SOCIAL_CLASS-subject rule (`:170,174-175`) with no
  INSTITUTION-subject rule to contend with.
- **Knock-on, re-derived rather than left stale (§1.2, world 6):** D-NF+22's derived ceilings mean a
  world with **zero** `HyperedgeType/COMMUNITY` hyperedges gets no ceiling and the pack fails
  `MissingCeiling` at LOAD — the driver's own comment rules that correct
  (`babylon-tick/src/lib.rs:250-252`). So revision 1's "a world with no community hyperedges makes
  every `for-each` iterate an empty element list" was false, and "communities with no members" is
  **unrepresentable** (`memory.rs:357-361` refuses an empty member list). Frozen L1's analogue is
  therefore a **load-refusal test**, and world 6 becomes L2's analogue: one carrier, one community
  whose members are all inactive classes, every rule loaded and the carrier rules firing, `pre ==
  post`. Every content world now seeds **one carrier and ≥1 community**, stated once in the worlds
  matrix header so no world can drift back to a zero.
- **Nine Minors:** all nine fixed (eight anchor corrections re-measured at the byte —
  `control-ratio.bsl:287`, `hypergraph_store.rs:91`, `query.rs:99-103`/`:91-98`,
  `test_law_community_system.py:221`, `state_hash.rs:314-319`, `substrate.rs:255-256`, §2.1's
  read-count tail (22 lines, last at `:401`), §3.2's headline count (**twelve** rows) — plus M-g's
  landed honest-null precedent (`consciousness_ternary_conformance.rs:244-245`) now cited where C4
  needs it, and M-i's orphan table header removed).

**No estimates, task boundaries, PR structure, D-row set or counts changed in 2.1** — the world-5b
and world-6 corrections are content-of-fixture changes inside Task 10's existing Steps 1/4/7, and
the eight golden pins stay eight.
