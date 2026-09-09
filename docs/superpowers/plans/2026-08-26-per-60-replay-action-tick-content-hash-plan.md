<!-- vale off -->

# PER-60 Replay, Action, and Tick Content Hash Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ratify and implement Babylon's versioned replay seed, ordered Practice action identity, stable world identity, and canonical `TickContentHashV1`, producing the database-free identity boundary needed by an honest future `CommittedTickEnvelope`.

**Architecture:** Kernel owns replay primitives, seed-aware RNG V2, nominal digest types, and the fixed outer hash. Practice owns actor and ordered-action identity. Graph owns stable element, carrier, resolver, and stable-state identity. BSL owns semantic discriminants and the real typed RNG carrier path. Tick composes prepared mechanics, world registers, payload, and one shared detached transaction used by the current `TickSession` and the identified `ReplayTickSession`. One schema and JSONL corpus bind all exact bytes to Rust and an independent Python verifier without making Python authoritative.

**Tech Stack:** Rust 2021 workspace, `sha2`, `rand_chacha` 0.10, BSL, `babylon-kernel`, `babylon-practice-contract`, `babylon-graph`, `babylon-bsl`, `babylon-tick`, `babylon-persistence`, Python 3.12, Pytest, YAML, JSONL, Cargo, Clippy, Vale

**Spec:** `docs/superpowers/specs/2026-08-26-per-60-replay-action-tick-content-hash-design.md`

## File Map

### Kernel identity and RNG

- Create `rust/crates/babylon-kernel/src/replay.rs`: `ReplaySessionIdV1`, `ReplaySeed`, `RngLayoutVersion`, `RngDomainV2`, checked codec helpers, and typed `RngSeedContext`.
- Modify `rust/crates/babylon-kernel/src/rng.rs`: preserve V1 exactly and add the low-level V2 seed and stream entry points.
- Create `rust/crates/babylon-kernel/src/tick_content_hash.rs`: digest newtypes, exact ten-section composer, and checked canonical preimage.
- Modify `rust/crates/babylon-kernel/src/lib.rs`: expose the new typed production surface.
- Create `rust/crates/babylon-kernel/tests/replay_identity.rs`: replay-session, replay-seed, and layout boundary tests.
- Create `rust/crates/babylon-kernel/tests/rng_v2.rs`: exact V1 preservation and cross-block V2 stream tests.
- Create `rust/crates/babylon-kernel/tests/tick_content_hash.rs`: typed outer-composer tests and mutation coverage.
- Modify `rust/crates/babylon-persistence/src/hashes.rs`: remove the unused duplicate `RefDigest` and `TickContentHash` types; callers use the kernel-owned `RefDigestV1` and `TickContentHashV1` directly.
- Modify `rust/crates/babylon-persistence/src/lib.rs`: expose no second names or re-exports for the kernel-owned identity types.

### Practice actor and action identity

- Create `rust/crates/babylon-practice-contract/src/actor_v2.rs`: opaque eight-byte `ActorOrganizationIdV2`.
- Modify `rust/crates/babylon-practice-contract/src/authority_v2.rs`: use the actor newtype without moving V2 bytes.
- Modify `rust/crates/babylon-practice-contract/src/intent_v2.rs`: use the actor newtype in intent and proposal identity.
- Modify `rust/crates/babylon-practice-contract/src/batch_v2.rs`: validate and order the actor newtype.
- Modify `rust/crates/babylon-practice-contract/src/resource_v2.rs`: use the actor newtype for ownership and proposal rows.
- Modify `rust/crates/babylon-practice-contract/src/strike_v2.rs`: use the actor newtype in organization relations.
- Create `rust/crates/babylon-practice-contract/src/ordered_action_v1.rs`: private action and batch types, ActionId derivation, empty constructor, and trusted non-empty projector.
- Modify `rust/crates/babylon-practice-contract/src/lib.rs`: expose the actor and ordered-action modules.
- Modify `rust/crates/babylon-practice-contract/tests/authority_v2.rs`, `authority_v2_contract.rs`, `batch_v2.rs`, `intent_v2.rs`, `intent_v2_contract.rs`, `resource_v2.rs`, `strike_v2.rs`, and `strike_v2_contract.rs`: convert numeric fixture values through `ActorOrganizationIdV2::from_bytes(value.to_be_bytes())` while preserving expected bytes.
- Create `rust/crates/babylon-practice-contract/tests/actor_v2.rs`: opacity, byte ordering, and frozen-wire regression tests.
- Create `rust/crates/babylon-practice-contract/tests/ordered_action_v1.rs`: empty and structural non-empty projection tests.

### Stable graph identity

- Create `rust/crates/babylon-graph/src/stable_element.rs`: stable binary keys, ASCII carrier segments, sealed resolver, manifest, topology validation, and graph-owned carrier builder.
- Create `rust/crates/babylon-graph/src/stable_state.rs`: exact eight-section stable graph encoder and digest.
- Modify `rust/crates/babylon-graph/src/lib.rs`: expose stable identity modules.
- Create `rust/crates/babylon-graph/tests/stable_identity.rs`: cross-substrate, allocation-order, topology-seal, bounds, and codec tests.

### BSL semantics and real RNG path

- Modify `rust/crates/babylon-bsl/src/scenario.rs`: retain scenario scope and both node and hyperedge authored-name maps.
- Modify `rust/crates/babylon-bsl/src/types.rs`: expose checked snapshots for fields, enum types, and member declaration order.
- Modify `rust/crates/babylon-bsl/src/fuel.rs`: expose intrinsic-cost rows under exact limits.
- Modify `rust/crates/babylon-bsl/src/causal_contract.rs`: expose governed role, evidence, effect, and shape canonicalization.
- Create `rust/crates/babylon-bsl/src/identity_codec.rs`: exact `ValueV1`, `BslTypeV1`, and governed discriminant encoders.
- Create `rust/crates/babylon-bsl/src/identity_sections.rs`: prepared-environment section and tick-payload section snapshots.
- Modify `rust/crates/babylon-bsl/src/intrinsic_host.rs`: carry typed V1/V2 RNG context and graph-validated carrier bytes.
- Modify `rust/crates/babylon-bsl/src/evaluator.rs`: resolve subjects and active elements through the sealed graph resolver.
- Modify `rust/crates/babylon-bsl/src/tick.rs`: thread one typed draw context into the real `rng-draw` intrinsic.
- Modify `rust/crates/babylon-bsl/src/lib.rs`: expose the identity encoders and typed section values.
- Create `rust/crates/babylon-bsl/tests/tick_identity_contract.rs`: exact discriminants, order semantics, refusal, and bound tests.
- Modify `rust/crates/babylon-bsl/tests/r9_chapters.rs`: prove real V1/V2 RNG dispatch and V1 preservation.

### Tick composition and replay seam

- Modify `rust/crates/babylon-tick/src/phase_order.rs`: expose the already governed 34-slot/four-accepted-name-mapping bytes and digest without changing them.
- Create `rust/crates/babylon-tick/src/replay_identity.rs`: prepared environment, register manifest/set, stable world, and exact payload composition.
- Create `rust/crates/babylon-tick/src/replay_session.rs`: `ReplayTickSession` and `IdentifiedTickReportV1`.
- Modify `rust/crates/babylon-tick/src/session.rs`: route the current `TickSession` through the shared typed transaction.
- Modify `rust/crates/babylon-tick/src/lib.rs`: retain full preparation identity and host the single detached causal loop.
- Modify `rust/crates/babylon-tick/Cargo.toml`: promote `babylon-practice-contract` to a production dependency for the typed action batch.
- Create `rust/crates/babylon-tick/tests/replay_session.rs`: end-to-end identity, seed propagation, atomicity, and empty-action enforcement.
- Create `rust/crates/babylon-tick/tests/tick_content_hash_v1_contract.rs`: shared corpus consumption and bounded test parser.

### Shared contract and decision evidence

- Create `contracts/tick_content_hash_v1.yaml`: one language-neutral schema for every PER-60 layout, discriminant, ceiling, and exclusion.
- Create `contracts/tick_content_hash_v1_vectors.jsonl`: valid, refused, mutation, and end-to-end vectors.
- Create `tools/verify_tick_content_hash_v1.py`: independent bounded Python verifier for the schema and vectors.
- Create `tests/unit/tools/test_verify_tick_content_hash_v1.py`: verifier acceptance and mutation-refusal tests.
- Create `ai/decisions/ADR240_replay_action_tick_content_identity.yaml`: accepted ownership, P27 disposition, Gate 3 boundary, and supersession record.
- Modify `ai/decisions/index.yaml`: add ADR240 and advance the index version according to the existing sequence.
- Create `tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py`: bind the exact decision and index entry.
- Modify `src/babylon/kernel/tick_hash.py`: correct the live module wording so P27 remains frozen reference evidence outside authoritative Rust `TickContentHashV1`.
- Modify `docs/reference/determinism-contract.rst`: record accepted V1 layouts, vectors, and resolved P27 disposition.
- Modify `docs/concepts/architecture.rst`: distinguish replay-session physics identity from separate campaign durability identity.

## Global Constraints

- Every change serves the playable game slice by making one detached player-decision tick replayable and auditable. Do not add infrastructure without a direct identity-boundary consumer in this plan.
- Keep the existing P27 Python bytes, digests, fixtures, and tests unchanged. P27 remains executable reference evidence outside authoritative `TickContentHashV1`.
- Keep RNG V1's unversioned preimage, validation behavior, four-draw vector, and `TickSession` results byte-identical.
- Do not edit accepted Practice V2 YAML, pinned source SHA constants, canonical field descriptions, existing JSONL bytes, or manifests. Their `actor_org_id_u64` and `type: u64` descriptions remain frozen.
- `ActorOrganizationIdV2` is an opaque `[u8; 8]`; add no numeric, graph-handle, arithmetic, `From<u64>`, or `TryFrom<u64>` API.
- `StableCarrierKeyV2` belongs to `babylon-graph`. Kernel accepts validated bytes only because the existing dependency DAG forbids a graph type in kernel.
- Production code is typed encoder/composer-only. Add no production raw decoder for replay, action batch, graph identity, prepared environment, world, payload, or outer hash.
- The live Gate 3 replay path accepts only the exact empty action batch. A non-empty projection exists only for structural codec tests and confers no admission provenance.
- Do not add PostgreSQL schema, migration, hydration, writer, Archive outbox, `CommittedTickEnvelope`, cutover, player action execution, BSL practice effects, dynamic topology, or Bevy integration.
- Keep the current `GraphStateHash` and `NominalWorldHash` unchanged and outside `TickContentHashV1`.
- Use fixed-width big-endian integers for every new PER-60 canonical field, raw SHA-256 bytes, exact validated string bytes, checked arithmetic, fallible reservations, bounded iteration, and explicit typed errors. The explicitly governed ChaCha8 state words, block words, and current V1 preimage retain their specified little-endian layouts.
- Preserve semantic order where the design says order matters: rule execution, enum member declaration, event arrival, event payload source pairs including duplicates, and receipt publication. Sort only the declared canonical map/set lanes.
- Show RED before each behavioral implementation, make the smallest GREEN change, refactor only after GREEN, and run the smallest applicable test after every edit.
- Do not overlap heavy gates. Use `BLAS=1` for repository gates. Do not run Sphinx, `cargo doc`, `mise run ci:rust`, `mise run rust:check`, or another documentation-generating task.
- Never stage `.codex/` or unrelated user changes. Commit each task with `SKIP=rust-full-gate mise run commit -- "..."` only after its scoped tests, format, and Clippy checks pass.

---

### Task 1: Kernel replay identity and exact RNG V2

**Files:**

- Create: `rust/crates/babylon-kernel/src/replay.rs`
- Modify: `rust/crates/babylon-kernel/src/rng.rs`
- Modify: `rust/crates/babylon-kernel/src/lib.rs`
- Create: `rust/crates/babylon-kernel/tests/replay_identity.rs`
- Create: `rust/crates/babylon-kernel/tests/rng_v2.rs`

**Interfaces:**

```rust
pub struct ReplaySessionIdV1(Vec<u8>);
pub struct ReplaySeed(i64);
pub enum RngLayoutVersion { V1, V2 }
pub struct RngDomainV2(String);
pub enum RngSeedContext<'a> {
    V1 { session: &'a SessionId },
    V2 { session: &'a ReplaySessionIdV1, seed: ReplaySeed },
}

impl TryFrom<u32> for RngLayoutVersion {
    type Error = ReplayIdentityError;
}

pub fn seed_for_v2(
    session: &ReplaySessionIdV1,
    seed: ReplaySeed,
    tick: u64,
    domain: &RngDomainV2,
    validated_carrier_key: &[u8],
) -> Result<[u8; 32], ReplayIdentityError>;
```

- [ ] **Step 1: Write replay primitive and layout boundary tests**

  Add tests for 1-byte and 256-byte graphic ASCII sessions, canonical `u16` length framing, space/control/DEL/non-ASCII/empty/257-byte refusal, and exact `i64::{MIN,-1,0,1,MAX}` seed bytes. Assert `RngLayoutVersion::try_from` maps `1` and `2` exactly and returns the typed version error for `0`, `3`, and `u32::MAX`. Add compile-time API assertions that construct sessions only from checked bytes/strings and seeds only from `i64`.

- [ ] **Step 2: Run the primitive test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-kernel --test replay_identity --locked
  ```

  Expected: compile failure because the replay types and module do not exist.

- [ ] **Step 3: Implement checked replay primitives**

  Implement private fields, exact layout constants, borrowed byte accessors, canonical encoders, and a specific `ReplayIdentityError` for semantic string, version, length, conversion, and allocation failures. Do not add any campaign/persistence conversion or a Unicode normalization dependency.

- [ ] **Step 4: Run the primitive test and verify GREEN**

  ```bash
  cd rust
  cargo test -p babylon-kernel --test replay_identity --locked
  ```

- [ ] **Step 5: Write exact RNG V2 and V1-regression tests**

  Pin the asymmetric V2 preimage and SHA-256 key, first nine `u64` draws, and fresh-stream first `f64::to_bits()`. Add one mutation assertion for seed, session, tick, domain, and carrier bytes. Copy the current V1 four-draw expected values into a regression test without changing them.

- [ ] **Step 6: Run the RNG test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-kernel --test rng_v2 --locked
  ```

  Expected: compile failure because `seed_for_v2` and the V2 carrier constructor do not exist.

- [ ] **Step 7: Implement the exact V2 derivation and stream**

  Keep `seed_for`, `KernelRng::for_carrier`, and V1 validation untouched. Add the exact `babylon.rng-stream\0` big-endian preimage and `KernelRng::for_carrier_v2`. Use `rand_chacha` only behind tests that prove the design's language-neutral ChaCha8 state words, four double rounds, counter placement, little-endian block words, `next_u64`, and `next_f64` results. Offer no stream id, seek, or tick-global API.

- [ ] **Step 8: Refactor dispatch through one parsed layout**

  Add the typed `RngSeedContext` dispatch in kernel so BSL and tick do not branch on numeric version values. Keep the low-level V2 byte API documented as contract-test support only; the graph-owned production caller provides provenance later.

- [ ] **Step 9: Run the kernel gate and verify GREEN**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-kernel --locked
  cargo clippy -p babylon-kernel --all-targets --locked -- -D warnings
  ```

- [ ] **Step 10: Commit kernel replay and RNG identity**

  ```bash
  git add rust/crates/babylon-kernel
  SKIP=rust-full-gate mise run commit -- "feat(kernel): add replay identity and RNG V2"
  ```

---

### Task 2: Kernel-owned digest types and fixed outer TickContentHash

**Files:**

- Create: `rust/crates/babylon-kernel/src/tick_content_hash.rs`
- Modify: `rust/crates/babylon-kernel/src/lib.rs`
- Create: `rust/crates/babylon-kernel/tests/tick_content_hash.rs`
- Modify: `rust/crates/babylon-persistence/src/hashes.rs`
- Modify: `rust/crates/babylon-persistence/src/lib.rs`
- Modify: `rust/crates/babylon-persistence/Cargo.toml` only if a direct kernel dependency is absent at execution time

**Interfaces:**

```rust
pub struct RefDigestV1([u8; 32]);
pub struct PreparedEnvironmentDigestV1([u8; 32]);
pub struct StableWorldDigestV1([u8; 32]);
pub struct OrderedPracticeActionBatchDigestV1([u8; 32]);
pub struct TickPayloadDigestV1([u8; 32]);
pub struct TickContentHashV1([u8; 32]);
pub struct TickContentPreimageV1(Vec<u8>);

pub struct TickContentPartsV1<'a> {
    pub session: &'a ReplaySessionIdV1,
    pub resolve_tick: u64,
    pub seed: ReplaySeed,
    pub content: &'a ContentDigest,
    pub reference: RefDigestV1,
    pub prepared: PreparedEnvironmentDigestV1,
    pub prior_world: StableWorldDigestV1,
    pub actions: OrderedPracticeActionBatchDigestV1,
    pub result_world: StableWorldDigestV1,
    pub payload: TickPayloadDigestV1,
}
```

- [ ] **Step 1: Write the exact outer-composer test**

  Build asymmetric typed parts and assert the complete preimage is `349 + session.len()` bytes, begins with `babylon.tick-content\0`, carries layout 1, and contains tags `0x01` through `0x0a` exactly once in order. Assert the SHA-256 digest and one mutation per field and nested layout version.

- [ ] **Step 2: Run the outer test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-kernel --test tick_content_hash --locked
  ```

  Expected: compile failure because the nominal digest types and composer do not exist.

- [ ] **Step 3: Implement private digest wrappers and composer**

  Give every digest a private `[u8; 32]`, exact byte constructor/accessor, and no default. These constructors wrap an already-computed SHA-256 value for nominal typing and direct persistence callers; they do not decode a canonical identity object, prove provenance, or let `ReplayTickSession` accept caller-supplied prepared/world/payload digests. The authoritative replay session derives every such digest from its owning typed encoder, and only the outer composer creates its published `TickContentHashV1`. Implement the ten mandatory sections with checked capacity and `try_reserve_exact`; accept no optional/extension field. Keep campaign, database, P27, current graph/world hashes, allocator cursors, and wall time out.

- [ ] **Step 4: Make persistence reuse kernel authority**

  Remove only persistence's duplicate `TickContentHash` and `RefDigest` macro invocations. At each persistence caller, use `babylon_kernel::tick_content_hash::{RefDigestV1, TickContentHashV1}` directly. Add contract assertions that `babylon-persistence` exposes no short identity name or re-export. Add no calculation logic and change no persistence schema or H3 reference-cohort meaning.

- [ ] **Step 5: Run kernel and persistence gates**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-kernel -p babylon-persistence --locked
  cargo clippy -p babylon-kernel -p babylon-persistence --all-targets --locked -- -D warnings
  ```

- [ ] **Step 6: Commit the digest boundary**

  ```bash
  git add rust/crates/babylon-kernel rust/crates/babylon-persistence
  SKIP=rust-full-gate mise run commit -- "feat(kernel): own canonical tick content hash"
  ```

---

### Task 3: Opaque Practice V2 actor identity with frozen bytes

**Files:**

- Create: `rust/crates/babylon-practice-contract/src/actor_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/authority_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/batch_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/intent_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/resource_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/strike_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/authority_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/authority_v2_contract.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/batch_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/intent_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/intent_v2_contract.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/resource_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/strike_v2.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/strike_v2_contract.rs`
- Create: `rust/crates/babylon-practice-contract/tests/actor_v2.rs`

**Interface:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ActorOrganizationIdV2([u8; 8]);

impl ActorOrganizationIdV2 {
    pub const fn from_bytes(bytes: [u8; 8]) -> Self;
    pub const fn to_bytes(self) -> [u8; 8];
}
```

- [ ] **Step 1: Record frozen Practice V2 checksums and write actor tests**

  Capture `sha256sum` for every accepted Practice V2 YAML and JSONL file in the task log. Add tests that the newtype orders by unsigned bytes, round-trips `[u8; 8]`, has no numeric accessor in production usage, and preserves the exact existing intent, authority, batch, resource, and strike vector bytes.

- [ ] **Step 2: Run the actor test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-practice-contract --test actor_v2 --locked
  ```

  Expected: compile failure because `ActorOrganizationIdV2` does not exist.

- [ ] **Step 3: Add the actor type and migrate every V2 lane**

  Replace V2 actor `u64` fields in authority lookup, `PracticeIntentV2`, `PracticeProposalKeyV2`, resolved batches, resource ownership, and strike relations. Decode eight bytes with `ActorOrganizationIdV2::from_bytes`; encode with `to_bytes`. Numeric JSON adapters may call `value.to_be_bytes()` only while constructing test values. Leave all V1 fields and the V1-only `admission.rs`, `codec.rs`, `tests/admission.rs`, `tests/codec_vectors.rs`, and `tests/schema_contract.rs` unchanged.

- [ ] **Step 4: Search for incomplete actor migration**

  ```bash
  rg -n "actor_org_id:\s*u64|ActorOrganizationIdV2.*NodeId|From<u64>|TryFrom<u64>" rust/crates/babylon-practice-contract
  ```

  Expected: no V2 production actor remains `u64`, and no forbidden conversion exists. Any V1/schema-description hit must be named and intentionally preserved.

- [ ] **Step 5: Prove all frozen bytes remain identical**

  Run the whole crate and compare the previously captured checksums:

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-practice-contract --locked
  cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings
  cd ..
  sha256sum contracts/practice_*_v2.yaml contracts/practice_*_v2_vectors.jsonl contracts/resolved_practice_batch_v2.yaml contracts/resolved_practice_batch_v2_vectors.jsonl
  ```

  Expected: tests pass and every contract checksum equals the recorded pre-change value.

- [ ] **Step 6: Commit the actor refinement**

  ```bash
  git add rust/crates/babylon-practice-contract
  SKIP=rust-full-gate mise run commit -- "refactor(practice): make V2 actor identity opaque"
  ```

---

### Task 4: Ordered Practice action identity and empty runtime batch

**Files:**

- Create: `rust/crates/babylon-practice-contract/src/ordered_action_v1.rs`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`
- Create: `rust/crates/babylon-practice-contract/tests/ordered_action_v1.rs`

**Interfaces:**

```rust
pub struct PracticeActionIdV1([u8; 32]);
pub struct OrderedPracticeActionV1 {
    canonical_input_ordinal: u16,
    action_id: PracticeActionIdV1,
    intent: PracticeIntentV2,
}
pub struct OrderedPracticeActionBatchV1 {
    session: ReplaySessionIdV1,
    resolve_tick: u64,
    items: Vec<OrderedPracticeActionV1>,
    canonical_bytes: Vec<u8>,
    digest: OrderedPracticeActionBatchDigestV1,
}

impl OrderedPracticeActionBatchV1 {
    pub fn empty(
        session: ReplaySessionIdV1,
        resolve_tick: u64,
    ) -> Result<Self, OrderedPracticeActionError>;
    pub fn project(
        session: ReplaySessionIdV1,
        source: &ResolvedPracticeBatchV2,
        trusted_ledger: &PracticeInputAuthorityLedgerV2,
    ) -> Result<Self, OrderedPracticeActionError>;
    pub fn session(&self) -> &ReplaySessionIdV1;
    pub const fn resolve_tick(&self) -> u64;
    pub fn items(&self) -> &[OrderedPracticeActionV1];
    pub fn is_empty(&self) -> bool;
}
```

- [ ] **Step 1: Write empty-batch and ActionId tests**

  Assert exact `babylon.practice-action-id.v1\0` bytes and a 69-to-324-byte preimage. Assert exact empty batch bytes are `55 + session.len()`, the item count is zero, and session/tick are bound. Assert ordinal is absent from ActionId and changing an unrelated lower-sorting proposal changes later ordinals without moving the original ActionId.

- [ ] **Step 2: Write trusted projector refusal tests**

  Build a validated asymmetric `ResolvedPracticeBatchV2` plus `PracticeInputAuthorityLedgerV2`. Assert projection validates the source batch first, assigns contiguous `canonical_input_ordinal` values 0 through N-1, recomputes every ActionId, and refuses a ledger mismatch, unordered/duplicate proposal keys, mismatched tick, oversized intent, or more than 4,096 items. Add a compile-fail or visibility test proving callers cannot construct private actions from raw bytes, caller ordinals, or caller hashes.

- [ ] **Step 3: Run the action tests and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-practice-contract --test ordered_action_v1 --locked
  ```

  Expected: compile failure because the private ordered-action types do not exist.

- [ ] **Step 4: Implement the exact action and batch codecs**

  Implement the fixed domains, schema versions, `u16` intent length, checked action bounds, private fields, borrowed accessors, public exact-empty constructor, and public deliberately constrained non-empty structural projector shown above. Call the existing full `ResolvedPracticeBatchV2` validation before projection. The projector is the only cross-crate way to construct a non-empty typed batch for the runtime guard test; it takes the complete batch and trusted ledger, and it still does not prove accepted-input provenance. Do not add a production decoder.

- [ ] **Step 5: Run the Practice gate and verify GREEN**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-practice-contract --locked
  cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings
  ```

- [ ] **Step 6: Commit ordered action identity**

  ```bash
  git add rust/crates/babylon-practice-contract
  SKIP=rust-full-gate mise run commit -- "feat(practice): add ordered action identity"
  ```

---

### Task 5: Stable graph element resolver and carrier identity

**Files:**

- Create: `rust/crates/babylon-graph/src/stable_element.rs`
- Modify: `rust/crates/babylon-graph/src/lib.rs`
- Create: `rust/crates/babylon-graph/tests/stable_identity.rs`

**Interfaces:**

```rust
pub enum StableElementKeyV1 {
    Node { scenario: String, local_name: String },
    Edge {
        scenario: String,
        edge_type: String,
        source_local_name: String,
        target_local_name: String,
    },
    Hyperedge { scenario: String, local_name: String },
}
pub struct StableElementCarrierSegmentV1(String);
pub struct StableCarrierKeyV2(String);
pub struct StableElementResolverManifestV1 { canonical_bytes: Vec<u8>, digest: [u8; 32] }
pub struct StableElementResolverV1 {
    scenario_scope: String,
    node_by_handle: HashMap<NodeId, StableElementKeyV1>,
    node_by_name: BTreeMap<String, NodeId>,
    hyperedge_by_handle: HashMap<HyperedgeId, StableElementKeyV1>,
    hyperedge_by_name: BTreeMap<String, HyperedgeId>,
    sealed_topology: SealedTopologyV1,
    manifest: StableElementResolverManifestV1,
}

impl StableElementResolverV1 {
    pub fn seal<G: GraphSubstrate + CanonicalState>(
        graph: &G,
        scenario_scope: &str,
        node_names: &HashMap<NodeId, String>,
        hyperedge_names: &HashMap<HyperedgeId, String>,
    ) -> Result<Self, StableIdentityError>;
    pub fn node_key(&self, node: NodeId) -> Result<&StableElementKeyV1, StableIdentityError>;
    pub fn hyperedge_key(
        &self,
        hyperedge: HyperedgeId,
    ) -> Result<&StableElementKeyV1, StableIdentityError>;
    pub fn edge_key(
        &self,
        edge_type: &str,
        source: NodeId,
        target: NodeId,
    ) -> Result<StableElementKeyV1, StableIdentityError>;
    pub fn carrier_key(
        &self,
        subject: &StableElementKeyV1,
        active: &[StableElementKeyV1],
        draw_slot: i64,
    ) -> Result<StableCarrierKeyV2, StableIdentityError>;
}
```

- [ ] **Step 1: Write exact stable-key and carrier tests**

  Assert standalone binary node, directed edge, and hyperedge keys; their distinct ASCII carrier segments; zero-active and mixed node/edge/hyperedge carrier framing; outermost-to-innermost active order; canonical signed decimal draw slots; 256-element and 105,962-byte maxima; and maximum-plus-one refusal before allocation.

- [ ] **Step 2: Write resolver and manifest tests**

  Assert exact mandatory scenario/node/hyperedge sections, canonical sorting, 65,536 combined-row and 8 MiB ceilings, handle↔name bijections, parallel authored hyperedges, and refusal for missing, duplicate, dangling, non-ASCII, added, removed, or membership-mutated topology. Assert no Debug/raw-id fallback.

- [ ] **Step 3: Run graph stable identity test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-graph --test stable_identity --locked
  ```

  Expected: compile failure because the stable identity module does not exist.

- [ ] **Step 4: Implement graph-owned codecs and sealed resolver**

  Encode exact binary fields and ASCII framed segments once in graph. Store forward and inverse node/hyperedge mappings plus the sealed topology witness. Expose checked `node_key`, `hyperedge_key`, and `edge_key(edge_type, source, target)` resolution over graph-native ids so BSL never reimplements or forges stable keys. Validate edges through named endpoints. Keep node/hyperedge types out of element identity but inside the resolver manifest; keep edge type and ordered endpoints in edge identity.

- [ ] **Step 5: Implement the private carrier builder**

  Accept resolved stable elements only. Reframe each already-framed element segment as one opaque outer segment, append canonical slot text, use checked aggregate arithmetic, and expose only `validated_bytes()` from the private `StableCarrierKeyV2` field.

- [ ] **Step 6: Run scoped graph checks**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-graph --test stable_identity --locked
  cargo clippy -p babylon-graph --all-targets --locked -- -D warnings
  ```

- [ ] **Step 7: Commit stable element identity**

  ```bash
  git add rust/crates/babylon-graph
  SKIP=rust-full-gate mise run commit -- "feat(graph): add stable element resolver"
  ```

---

### Task 6: Stable graph-state bytes across substrates and handle allocation

**Files:**

- Create: `rust/crates/babylon-graph/src/stable_state.rs`
- Modify: `rust/crates/babylon-graph/src/lib.rs`
- Modify: `rust/crates/babylon-graph/tests/stable_identity.rs`

**Interface:**

```rust
pub struct StableGraphStateV1 { canonical_bytes: Vec<u8>, digest: StableGraphStateHashV1 }

pub fn encode_stable_graph_state_v1<G: CanonicalState>(
    graph: &G,
    resolver: &StableElementResolverV1,
) -> Result<StableGraphStateV1, StableIdentityError>;
```

- [ ] **Step 1: Write exact eight-section state tests**

  Build one asymmetric graph containing nodes, node real and currency fields, a directed edge with strength and another field, a named hyperedge with unsorted members, and a hyperedge field. Assert all tags `0x01` through `0x08` occur once in order, empty sections still emit counts, declared sorting is exact, negative zero becomes positive zero, and finite bits and signed `i128` micro-units are exact.

- [ ] **Step 2: Write cross-substrate and handle-allocation tests**

  Build semantically equal `MemoryGraph` and `HypergraphStore` values in different insertion orders and with genuinely different `NodeId` and `HyperedgeId` allocation histories. Assert stable bytes/digests equal while at least one current `GraphStateHash` differs. Assert unordered map, registry, and hyperedge-member insertion cannot move stable output.

- [ ] **Step 3: Write refusal and bound tests**

  Cover unknown owners, absent edges, empty hyperedges, unknown/duplicate members, duplicate fact keys, numeric/currency cross-lane collision, `/strength` duplication, NaN/infinity, per-section maxima, aggregate 1,048,576 fact units, and 64 MiB bytes. Every maximum-plus-one case must refuse before partial bytes escape.

- [ ] **Step 4: Run the stable state test and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-graph --test stable_identity --locked
  ```

  Expected: compile failure because `StableGraphStateV1` and its encoder do not exist.

- [ ] **Step 5: Implement stable-state encoding over the seven live listings**

  Reuse `CanonicalState` listings rather than widening the substrate. Resolve every runtime handle through the sealed resolver, validate topology again at encoding, sort only the specified identity tuples, reject equal keys, and drop no empty section. Keep current graph hashing untouched.

- [ ] **Step 6: Run graph gate and verify GREEN**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-graph --locked
  cargo clippy -p babylon-graph --all-targets --locked -- -D warnings
  ```

- [ ] **Step 7: Commit stable graph state**

  ```bash
  git add rust/crates/babylon-graph
  SKIP=rust-full-gate mise run commit -- "feat(graph): encode stable graph state"
  ```

---

### Task 7: BSL identity discriminants and prepared/payload sections

**Files:**

- Create: `rust/crates/babylon-bsl/src/identity_codec.rs`
- Create: `rust/crates/babylon-bsl/src/identity_sections.rs`
- Modify: `rust/crates/babylon-bsl/src/{causal_contract,fuel,scenario,types,lib}.rs`
- Create: `rust/crates/babylon-bsl/tests/tick_identity_contract.rs`
- Modify: `rust/crates/babylon-tick/src/lib.rs` only to retain scenario scope and hyperedge authored-name maps in `PreparedRules`

**Interfaces:**

```rust
pub struct PreparedBslSectionsV1 {
    fields_and_exemptions: Vec<u8>,
    intrinsic_costs: Vec<u8>,
    constants: Vec<u8>,
    enum_types: Vec<u8>,
    vocabulary: Vec<u8>,
    aggregate_rows: u32,
}
pub struct TickPayloadSectionsV1 {
    rule_outcomes: Vec<u8>,
    events: Vec<u8>,
    receipts: Vec<u8>,
    aggregate_rows: u32,
}

pub fn encode_value_v1(
    value: &Value,
    resolver: &StableElementResolverV1,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError>;
```

- [ ] **Step 1: Write one test for every governed discriminant**

  Pin `ValueV1` tags `0x01..0x09`; `BslTypeV1` `0x01..0x0a`; field, role, evidence, effect, shape, and enum-kind tags; exact option/Boolean bytes; `ConstValueV1` restriction to value tags `0x01..0x05`; enum type-name resolution; and stable node/edge/hyperedge references through graph.

- [ ] **Step 2: Write ordering and presence tests**

  Assert resolved rule execution order, field and exemption sorting, intrinsic-cost sorting, constant sorting, enum-type sorting with member declaration order preserved, and closed-vocabulary outer/per-kind presence. Prove absent vocabulary differs from present-empty and that event arrival, duplicate payload labels in live source order, and receipt vector order remain semantic.

- [ ] **Step 3: Run BSL identity tests and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-bsl --test tick_identity_contract --locked
  ```

  Expected: compile failure because identity codecs and section snapshots do not exist.

- [ ] **Step 4: Retain stable scenario identity at preparation**

  Extend `LoadedScenario`/`PreparedRules` with the validated scenario scope, node handle↔authored local-name map, and hyperedge handle↔authored local-name map already created during hydration. Do not infer names from runtime ids, types, members, or Debug. Preserve existing preparation behavior and all load refusals.

- [ ] **Step 5: Implement one governed BSL tag table**

  Put exact discriminants and canonical f64 logic in `identity_codec.rs`. Add checked helpers for `str32`, counts, options, and total bytes. Reuse graph's stable keys for reference values and one event-name canonicalizer for effects and payload events. Reject unknown enum ids, reference kinds, noncanonical values, non-finite floats, and semantic string violations.

- [ ] **Step 6: Implement bounded section snapshots**

  Expose read-only snapshots from live registries rather than duplicating registries in tick. Include the `TypeEnv` exemption ledger explicitly and distinguish no vocabulary from present-empty kinds. Apply the 65,536/64/1,048,576 row limits and 64 MiB ceiling before reserve.

- [ ] **Step 7: Run BSL and tick preparation tests**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-bsl --locked
  cargo test -p babylon-tick --lib --locked
  cargo clippy -p babylon-bsl -p babylon-tick --all-targets --locked -- -D warnings
  ```

- [ ] **Step 8: Commit semantic identity sections**

  ```bash
  git add rust/crates/babylon-bsl rust/crates/babylon-tick/src/lib.rs
  SKIP=rust-full-gate mise run commit -- "feat(bsl): encode replay identity sections"
  ```

---

### Task 8: Thread seed-aware RNG V2 through the real BSL intrinsic

**Files:**

- Modify: `rust/crates/babylon-bsl/src/intrinsic_host.rs`
- Modify: `rust/crates/babylon-bsl/src/evaluator.rs`
- Modify: `rust/crates/babylon-bsl/src/tick.rs`
- Modify: `rust/crates/babylon-bsl/tests/r9_chapters.rs`
- Modify: `rust/crates/babylon-bsl/tests/tick_identity_contract.rs`
- Modify: `rust/crates/babylon-tick/src/lib.rs`

**Interface change:**

```rust
pub enum DrawIdentityContext<'a> {
    V1 {
        session: &'a SessionId,
        domain: &'a str,
        subject: &'a str,
    },
    V2 {
        session: &'a ReplaySessionIdV1,
        seed: ReplaySeed,
        domain: RngDomainV2,
        resolver: &'a StableElementResolverV1,
        subject: StableElementKeyV1,
    },
}
pub struct DrawContext<'a> {
    pub identity: DrawIdentityContext<'a>,
    pub tick: u64,
}
```

  V1 retains its current domain and subject/carrier behavior exactly. The authoritative V2 branch must receive validated `RngDomainV2` and resolver-produced graph types and may not enter the V1 content-id/Debug fallback.

- [ ] **Step 1: Write real-intrinsic mutation tests**

  Execute one loaded rule containing `rng-draw`. Hold mechanics fixed and mutate V2 seed, session, tick, firing rule qname, subject node, nested active node, edge, hyperedge, and draw slot. Assert exact written `f64::to_bits()` changes for every mutation. Assert the stable active-element order is outermost first.

- [ ] **Step 2: Write current V1 preservation and missing-provenance tests**

  Assert the current V1 rule and four-draw results remain exact. Assert V2 refuses a missing resolver, unresolvable subject/element, dynamic topology, invalid domain, and caller-supplied raw stable-key bytes.

- [ ] **Step 3: Run intrinsic tests and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-bsl --test r9_chapters --test tick_identity_contract --locked
  ```

  Expected: V2 tests fail because `DrawContext` still carries only current V1 session/content-id data.

- [ ] **Step 4: Replace the split helper path with typed dispatch**

  Thread `RngSeedContext` through `run_tick_observed`, evaluator call context, `KernelIntrinsicHost`, and tick's shared `run_prepared_tick_with` seam. For V2, resolve subject plus active elements through `StableElementResolverV1`, call graph's `carrier_key`, and pass only `validated_bytes()` to kernel. For V1, call the unchanged `KernelRng::for_carrier` route with no V2 validation. Preserve the current V1 tests and API. Task 8 prepares the live internal seam; Task 10 creates `ReplayTickSession` as its authoritative V2 caller.

- [ ] **Step 5: Prove the production call graph contains the seed**

  Use the end-to-end test plus a focused search:

  ```bash
  rg -n "RngSeedContext|ReplaySeed|carrier_key|seed_for_v2" rust/crates/babylon-{kernel,graph,bsl,tick}/src
  ```

  Expected after Task 8: `run_prepared_tick_with -> run_tick_observed -> DrawContext -> KernelIntrinsicHost -> graph carrier bytes -> kernel V2` is one real executable seam, while all existing callers still select V1. Task 10 must extend the proof to `ReplayTickSession -> run_prepared_tick_with`; no separate test-only hashing helper may substitute for execution.

- [ ] **Step 6: Run BSL gate and verify GREEN**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-bsl --locked
  cargo test -p babylon-tick --lib --locked
  cargo clippy -p babylon-bsl -p babylon-tick --all-targets --locked -- -D warnings
  ```

- [ ] **Step 7: Commit real RNG V2 plumbing**

  ```bash
  git add rust/crates/babylon-bsl rust/crates/babylon-tick/src/lib.rs
  SKIP=rust-full-gate mise run commit -- "feat(bsl): thread replay seed through rng draw"
  ```

---

### Task 9: Prepared environment, registers, stable world, and exact payload

**Files:**

- Create: `rust/crates/babylon-tick/src/replay_identity.rs`
- Modify: `rust/crates/babylon-tick/src/phase_order.rs`
- Modify: `rust/crates/babylon-tick/src/lib.rs`
- Create: `rust/crates/babylon-tick/tests/replay_session.rs`

**Interfaces:**

```rust
pub struct PreparedEnvironmentV1 { canonical_bytes: Vec<u8>, digest: PreparedEnvironmentDigestV1 }
pub struct WorldRegisterManifestV1 { canonical_bytes: Vec<u8>, digest: [u8; 32] }
pub struct WorldRegisterSetV1 { canonical_bytes: Vec<u8>, digest: [u8; 32] }
pub struct StableWorldV1 { canonical_bytes: Vec<u8>, digest: StableWorldDigestV1 }
pub struct TickPayloadV1 { canonical_bytes: Vec<u8>, digest: TickPayloadDigestV1 }
```

- [ ] **Step 1: Write exact prepared-environment tests**

  Assert the domain/layout and mandatory tags `0x01..0x0a`: verified loaded rules hash, unchanged phase schedule layout/digest, governed rule order, fields plus exemption ledger, intrinsic costs, constants, enum types, vocabulary presence, resolver manifest, and register manifest. Mutate every semantic row/order and nested layout and assert the owning digest moves.

- [ ] **Step 2: Write exact register and stable-world tests**

  Assert the register manifest has exactly `world/completed-tick` layout 1. Assert prior/result sets bind that manifest and encode non-negative `i64` completed ticks. Assert stable world contains only stable graph plus register-set layout/digests. Cover resolve tick 1, `i64::MAX`, zero, negative, overflow, wrong manifest, extra/missing/out-of-order register, and allocator-cursor exclusion.

- [ ] **Step 3: Write exact payload tests**

  Assert governed rule outcomes, executable event order, duplicate payload labels in source order, receipt vector order, and exact zero `u16` action-outcome count. Assert `TickReport.fired` equals the checked sum and is not separately encoded. Mutate rule, event, pair, and receipt order and prove the digest moves. Cover reference resolution, non-finite values, row limits, and 64 MiB ceiling.

- [ ] **Step 4: Run replay identity tests and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-tick --test replay_session --locked
  ```

  Expected: compile failure because the tick-owned canonical objects do not exist.

- [ ] **Step 5: Implement exact tick-owned composers**

  Compose from graph and BSL returned objects; do not duplicate their tag tables. Recompute canonical rules hash from loaded forms and compare it to `ContentDigest.rules_hash` before session construction succeeds. Add one fallible `pub(crate)` `PhaseScheduleV1` canonical-bytes object/API beside the existing digest, derive that digest from the object without moving the current vector, and borrow both here. Enforce local and aggregate limits before allocation.

- [ ] **Step 6: Run scoped tick identity checks**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-tick --test replay_session --locked
  cargo clippy -p babylon-tick --all-targets --locked -- -D warnings
  ```

- [ ] **Step 7: Commit tick identity objects**

  ```bash
  git add rust/crates/babylon-tick
  SKIP=rust-full-gate mise run commit -- "feat(tick): compose stable world identity"
  ```

---

### Task 10: One shared detached transaction and executable ReplayTickSession

**Files:**

- Create: `rust/crates/babylon-tick/src/replay_session.rs`
- Modify: `rust/crates/babylon-tick/src/session.rs`
- Modify: `rust/crates/babylon-tick/src/lib.rs`
- Modify: `rust/crates/babylon-tick/src/phase_order.rs`
- Modify: `rust/crates/babylon-tick/Cargo.toml`
- Modify: `rust/crates/babylon-tick/tests/replay_session.rs`

**Interfaces:**

```rust
pub struct ReplayTickSession<G> {
    graph: G,
    prepared: PreparedRules,
    completed_tick: i64,
    session: ReplaySessionIdV1,
    seed: ReplaySeed,
    content: ContentDigest,
    reference: RefDigestV1,
    resolver: StableElementResolverV1,
    register_manifest: WorldRegisterManifestV1,
    prepared_environment: PreparedEnvironmentV1,
}
pub struct IdentifiedTickReportV1 {
    report: TickReport,
    action_batch_bytes: Vec<u8>,
    action_batch_layout_version: u32,
    action_batch_digest: OrderedPracticeActionBatchDigestV1,
    prior_registers: WorldRegisterSetV1,
    prior_world: StableWorldV1,
    result_registers: WorldRegisterSetV1,
    result_world: StableWorldV1,
    payload: TickPayloadV1,
    outer_preimage: TickContentPreimageV1,
    resolver_manifest_digest: [u8; 32],
    prepared_environment_digest: PreparedEnvironmentDigestV1,
    prior_stable_graph_digest: StableGraphStateHashV1,
    result_stable_graph_digest: StableGraphStateHashV1,
    tick_content_hash: TickContentHashV1,
}

trait ReplayIdentityComposer {
    fn compose(
        &self,
        inputs: ReplayIdentityInputs<'_>,
    ) -> Result<ReplayIdentityArtifactsV1, ReplayTickError>;
}

impl<G> ReplayTickSession<G> {
    pub fn new(
        scenario_src: &str,
        prelude_src: Option<&str>,
        rule_src: &str,
        graph: G,
        session: ReplaySessionIdV1,
        seed: ReplaySeed,
        content: ContentDigest,
        reference: RefDigestV1,
    ) -> Result<Self, ReplayTickError>;
    pub fn advance(
        &mut self,
        sink: &mut CollectingSink,
        actions: &OrderedPracticeActionBatchV1,
    ) -> Result<IdentifiedTickReportV1, ReplayTickError>;
    pub fn resolver_manifest_bytes(&self) -> &[u8];
    pub fn register_manifest_bytes(&self) -> &[u8];
    pub fn prepared_environment_bytes(&self) -> &[u8];
}
```

- [ ] **Step 1: Write construction and success-path tests**

  Construct replay sessions for both supported graph substrates with session, seed, content, reference, scenario, optional prelude, and rules. The public constructor has no V1 or caller-selected numeric layout input: it constructs `RngSeedContext::V2` by type, and an API assertion proves a current `SessionId`/V1 context cannot construct it. Assert construction seals topology, verifies rules hash, and owns resolver/register/prepared bytes once. Borrow all three static diagnostic slices before and after two advances and assert the same session-owned storage remains; assert reports contain only their small digests and no cloned static preimages. Advance an exact empty batch and assert the tick report, exact action bytes and direct typed action digest/version, prior/result register and stable-world bytes, exact payload, outer preimage, all versions/digests, and `TickContentHashV1` agree.

- [ ] **Step 2: Write real seed propagation and process identity tests**

  Run the same fixture in two fresh test processes and across substrates. Assert exact bytes/digests agree. Mutate only `ReplaySeed`; assert real `rng-draw` written bits, result stable world, payload when applicable, and outer hash move while prepared environment, prior world, and empty action batch stay fixed.

- [ ] **Step 3: Write runtime guard and atomic-failure tests**

  Assert a non-empty structural batch, mismatched session, mismatched next tick, topology change, resolver failure, deterministic returned hash/codec reservation failure, event-sink reservation failure, and rule failure leave graph, sink events, receipts, completed tick, current hashes, nested identity, and outer hash unpublished. Inject the identity reservation failure through the crate-private `ReplayIdentityComposer` boundary, analogous to the existing prepared-sink/hash seams; do not attempt to provoke process OOM. Assert the action guard runs before detached adjudication.

- [ ] **Step 4: Run replay session tests and verify RED**

  ```bash
  cd rust
  cargo test -p babylon-tick --test replay_session --locked
  ```

  Expected: compile failure because `ReplayTickSession` and the shared execution-identity mode do not exist.

- [ ] **Step 5: Extract one shared internal transaction**

  Replace the session-specific call shape with a private typed execution mode: current V1 has no `TickContentHashV1` composer; replay V2 carries the resolver and a crate-private `ReplayIdentityComposer`. The production implementation runs the exact real codecs; a test implementation returns one deterministic typed reservation error at the same pre-publication boundary. Keep one detached graph, one causal rule loop, one event/receipt buffer, and one publication point. Perform every fallible stable encode, digest, payload, outer compose, and sink reserve before assigning graph, committing events, or incrementing completed tick.

- [ ] **Step 6: Implement ReplayTickSession without per-tick static clones**

  Cache resolver manifest, register manifest, and prepared environment exact bytes in the session and expose the borrowed diagnostic accessors shown above. Drop prior stable-graph bytes after digest and before rule execution; drop result stable-graph bytes after digest. Return only their digests in the report while retaining exact dynamic register/world/action/payload/outer bytes. Carry the accepted-action layout version and `OrderedPracticeActionBatchDigestV1` as direct typed report fields, and expose direct equality with the supplied empty batch digest; do not force PER-20 to decode the outer preimage. Keep the report explicitly non-durable.

- [ ] **Step 7: Keep current TickSession and Bevy paths unchanged**

  Route `TickSession` through the shared transaction with `RngSeedContext::V1`; preserve its public constructor, report, hashes, and V1 draws. Do not change `babylon-client`, Bevy engine link, PostgreSQL, or Python writer code.

- [ ] **Step 8: Run tick and upstream crate gates**

  ```bash
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-kernel -p babylon-graph -p babylon-bsl -p babylon-practice-contract -p babylon-tick --locked
  cargo clippy -p babylon-kernel -p babylon-graph -p babylon-bsl -p babylon-practice-contract -p babylon-tick --all-targets --locked -- -D warnings
  ```

- [ ] **Step 9: Commit the executable replay seam**

  ```bash
  git add rust/crates/babylon-tick
  SKIP=rust-full-gate mise run commit -- "feat(tick): execute identified replay ticks"
  ```

---

### Task 11: One language-neutral schema, vectors, and independent verifier

**Files:**

- Create: `contracts/tick_content_hash_v1.yaml`
- Create: `contracts/tick_content_hash_v1_vectors.jsonl`
- Create: `rust/crates/babylon-tick/tests/tick_content_hash_v1_contract.rs`
- Create: `tools/verify_tick_content_hash_v1.py`
- Create: `tests/unit/tools/test_verify_tick_content_hash_v1.py`

**Corpus row families:**

- `replay_session`, `replay_seed`, `rng_v1`, `rng_v2`
- `stable_element`, `carrier_segment`, `resolver_manifest`, `stable_graph`
- `action_id`, `ordered_action_batch`
- `bsl_discriminant`, `prepared_environment`
- `register_manifest`, `register_set`, `stable_world`, `tick_payload`
- `tick_content_hash`, `mutation`, `refusal`

- [ ] **Step 1: Write verifier tests before the verifier**

  Add Pytest cases that demand one bounded schema/corpus reader; exact valid-byte/digest agreement; an independent four-double-round ChaCha8 reference implementation; recomputation of the V2 SHA-256 key, first nine `u64` draws, and fresh-stream first `f64::to_bits()`; unknown tag/layout, truncation, duplicate/out-of-order mandatory tag, noncanonical Boolean/option, unknown discriminant, and trailing-byte refusal; fixed row/byte ceilings; and one mutation row for every outer input and nested layout.

- [ ] **Step 2: Run verifier tests and verify RED**

  ```bash
  mise run test:q -- tests/unit/tools/test_verify_tick_content_hash_v1.py
  ```

  Expected: import failure because the verifier does not exist.

- [ ] **Step 3: Write the language-neutral schema**

  Record every exact domain, numeric layout, tag, byte order, semantic string bound, row/aggregate/byte ceiling, sorting or preserved-order rule, float rule, ownership boundary, production-decoder prohibition, runtime-empty action rule, and excluded identity. Record P27 and Practice V2 preservation explicitly.

- [ ] **Step 4: Write the asymmetric shared vector corpus**

  Include the unchanged RNG V1 four draws; replay session minimum/maximum and invalid classes; all seed extremes; one asymmetric V2 preimage and SHA-256 key, the first nine `u64` draws crossing a block boundary, and a separate fresh-stream first `f64` bit vector; every stable-element kind and carrier form; resolver and cross-allocation stable graph; asymmetric ActionId/non-empty structural batch plus exact empty runtime batch; every BSL discriminant and vocabulary presence; prepared environment; registers/world; payload; outer hash; mutation rows; and maximum/maximum-plus-one refusal rows.

- [ ] **Step 5: Implement an independent bounded Python verifier**

  Use only Python standard-library byte operations, `hashlib`, `json`, and `yaml` already owned by project tests. Implement the design's exact ChaCha8 constants, little-endian key/state words, four double rounds, quarter-round rotations, zero stream id, 64-bit counter, output word order, `next_u64`, and `next_f64` conversion independently; do not call Rust, `rand_chacha`, or the Python P27 encoder. Recompute and compare every RNG V2 vector literal. Give every parser loop a schema-derived fixed maximum and every cursor a checked end. Reject trailing bytes and malformed order. Keep this verifier test-only contract evidence, not a runtime writer.

- [ ] **Step 6: Implement the bounded Rust contract-test parser**

  In the tick integration test, parse the same JSONL and independently check domains, tags, lengths, counts, discriminants, digests, mutation expectations, and refusals. For the RNG V2 row, invoke the production `KernelRng` path and compare its recomputed key, nine `u64` draws, and fresh-stream `f64` bits to the corpus literals. Do not expose this parser from a production crate or add raw object decoders to production types.

- [ ] **Step 7: Run both consumers and verify GREEN**

  ```bash
  mise run test:q -- tests/unit/tools/test_verify_tick_content_hash_v1.py
  cd rust
  cargo test -p babylon-tick --test tick_content_hash_v1_contract --locked
  cargo clippy -p babylon-tick --test tick_content_hash_v1_contract --locked -- -D warnings
  ```

- [ ] **Step 8: Prove frozen corpora did not move**

  ```bash
  git diff --exit-code origin/dev -- contracts/practice_*_v2.yaml contracts/practice_*_v2_vectors.jsonl contracts/resolved_practice_batch_v2.yaml contracts/resolved_practice_batch_v2_vectors.jsonl
  mise run test:q -- tests/unit/kernel/test_tick_hash.py
  ```

  Expected: no Practice V2 contract diff and all P27 tick-hash tests pass unchanged.

- [ ] **Step 9: Commit shared identity vectors**

  ```bash
  git add contracts/tick_content_hash_v1.yaml contracts/tick_content_hash_v1_vectors.jsonl rust/crates/babylon-tick/tests/tick_content_hash_v1_contract.rs tools/verify_tick_content_hash_v1.py tests/unit/tools/test_verify_tick_content_hash_v1.py
  SKIP=rust-full-gate mise run commit -- "test(contract): bind tick content identity vectors"
  ```

---

### Task 12: Ratify ADR240 and correct live boundary documentation

**Files:**

- Create: `ai/decisions/ADR240_replay_action_tick_content_identity.yaml`
- Modify: `ai/decisions/index.yaml`
- Create: `tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py`
- Modify: `src/babylon/kernel/tick_hash.py`
- Modify: `docs/reference/determinism-contract.rst`
- Modify: `docs/concepts/architecture.rst`

- [ ] **Step 1: Recheck ADR240 availability and write failing decision tests**

  ```bash
  test ! -e ai/decisions/ADR240_replay_action_tick_content_identity.yaml
  rg -n "ADR240" ai/decisions tests/unit/decisions
  ```

  Then add a test requiring accepted status/date/title, crate ownership, Gate 3 live-empty action rule, P27 reference-evidence disposition, separate campaign/replay identity, exact vector path, preserved V1/Practice contracts, partial supersessions, and the exact decision-index entry.

- [ ] **Step 2: Run the decision test and verify RED**

  ```bash
  mise run test:q -- tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py
  ```

  Expected: failure because ADR240 and its index row do not exist.

- [ ] **Step 3: Create ADR240 and update the decision index**

  Record the design's ownership and exact boundaries. State that ADR220 cross-implementation identity means equal accepted `TickContentHashV1` vectors, not equality with P27 JSON. Preserve P27 execution/tests and Python writer authority until cutover. Record that runtime action batch is empty, non-empty projection has no accepted-input provenance, and PER-20/Gate 5 retain their scopes.

- [ ] **Step 4: Correct only live documentation claims**

  In the Python module docstring, describe P27 as frozen reference evidence outside authoritative Rust V1. In the determinism reference, close the open P27 disposition and link exact schema/vectors. In architecture, state replay session is material identity while campaign identity remains separate durability identity. Do not rewrite historical ADRs, specs, reports, or completed plans.

- [ ] **Step 5: Run targeted docs and decision checks**

  ```bash
  mise run test:q -- tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py tests/unit/kernel/test_tick_hash.py
  vale ai/decisions/ADR240_replay_action_tick_content_identity.yaml src/babylon/kernel/tick_hash.py docs/reference/determinism-contract.rst docs/concepts/architecture.rst
  git diff --check
  ```

  Expected: tests and Vale pass with no whitespace errors; no documentation build runs.

- [ ] **Step 6: Commit the ratified boundary**

  ```bash
  git add ai/decisions/ADR240_replay_action_tick_content_identity.yaml ai/decisions/index.yaml tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py src/babylon/kernel/tick_hash.py docs/reference/determinism-contract.rst docs/concepts/architecture.rst
  SKIP=rust-full-gate mise run commit -- "docs(architecture): ratify tick content identity"
  ```

---

### Task 13: Full verification, adversarial review, and Linear evidence

**Files:**

- Modify only files found defective by review, and rerun their owning RED/GREEN task before changing them.
- Update Linear issue `PER-60` with commit, test, and boundary evidence; do not add source status anywhere else.

- [ ] **Step 1: Inspect the complete diff and protected surfaces**

  ```bash
  git status --short
  git diff --stat origin/dev...HEAD
  git diff --check origin/dev...HEAD
  git diff origin/dev...HEAD -- contracts/practice_*_v2.yaml contracts/practice_*_v2_vectors.jsonl contracts/resolved_practice_batch_v2.yaml contracts/resolved_practice_batch_v2_vectors.jsonl
  rg -n "TODO|TBD|FIXME|placeholder|unimplemented!|todo!|panic!" rust/crates/babylon-{kernel,practice-contract,graph,bsl,tick,persistence} contracts/tick_content_hash_v1.yaml tools/verify_tick_content_hash_v1.py ai/decisions/ADR240_replay_action_tick_content_identity.yaml
  ```

  Expected: only intended files differ, protected contract diff is empty, and no incomplete implementation marker exists. Existing unrelated matches must be named and excluded by exact path/line evidence.

- [ ] **Step 2: Run the smallest exact contract gates**

  ```bash
  mise run test:q -- tests/unit/tools/test_verify_tick_content_hash_v1.py tests/unit/decisions/test_adr240_replay_action_tick_content_identity.py tests/unit/kernel/test_tick_hash.py
  cd rust
  cargo fmt --all -- --check
  cargo test -p babylon-kernel -p babylon-graph -p babylon-bsl -p babylon-practice-contract -p babylon-tick -p babylon-persistence --locked
  cargo clippy -p babylon-kernel -p babylon-graph -p babylon-bsl -p babylon-practice-contract -p babylon-tick -p babylon-persistence --all-targets --locked -- -D warnings
  ```

- [ ] **Step 3: Run repository gates sequentially without docs**

  Run each command only after the previous one finishes:

  ```bash
  BLAS=1 mise run check
  BLAS=1 mise run rust:check-no-docs
  BLAS=1 mise run qa:regression
  BLAS=1 mise run qa:vault-regression-ci
  BLAS=1 mise run check:gate-coverage
  ```

  Preserve the complete redirected logs and exact exit codes. Do not substitute watcher completion for the final result.

- [ ] **Step 4: Request independent adversarial review**

  Ask reviewers to compare implementation line-by-line with the committed design, focusing on exact byte layouts, fixed limits, semantic versus canonical order, actor provenance, resolver sealing, RNG V2 real-call propagation, transaction atomicity, raw-decoder absence, memory retention, V1/P27/frozen Practice preservation, and playable-slice scope. Require findings with file/line evidence and severity.

- [ ] **Step 5: Address every review finding with TDD**

  For each accepted finding, add or tighten the smallest failing behavioral test, run it to RED, implement the surgical correction, rerun it to GREEN, and rerun the owning crate's Clippy gate. Record why rejected findings conflict with the ratified design.

- [ ] **Step 6: Repeat final verification at the exact reviewed head**

  Re-run Steps 1 through 3 after the final correction. Record `git rev-parse HEAD`, all exit codes, and protected-file checksums. Do not claim completion from earlier commits or stale logs.

- [ ] **Step 7: Commit any review-only corrections**

  If review produced changes, repeat the owning task's exact `git add` command restricted to that task's reviewed files, inspect `git diff --cached`, and commit:

  ```bash
  SKIP=rust-full-gate mise run commit -- "fix(identity): close PER-60 review findings"
  ```

  If review produced no changes, do not create an empty commit.

- [ ] **Step 8: Record Linear completion evidence**

  Comment on `PER-60` with branch, exact head SHA, design and plan paths, task commits, exact green commands, vector/schema paths, ADR240, P27 and Practice preservation evidence, the live empty-action guard, and explicit PER-20/Gate 5/Bevy/PostgreSQL exclusions. Move issue status only when its own acceptance criteria are fully met.

- [ ] **Step 9: Confirm the playable-slice handoff**

  State the delivered boundary in one sentence: a database-free Rust replay tick now binds seed, mechanics, stable prior/result world, exact empty accepted actions, payload, and canonical hash atomically; PER-20 can next wrap that typed evidence in `CommittedTickEnvelope` without inventing identity.
