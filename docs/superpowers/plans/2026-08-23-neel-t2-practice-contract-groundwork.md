<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- The mandated plan template, literal APIs, wire layouts, and command lines
     require local exemptions from heuristic prose rules. -->
<!-- vale Vale.Terms = NO -->
<!-- vale ste.NounClusters = NO -->
<!-- vale ste.Dictionary = NO -->
<!-- vale write-good.TooWordy = NO -->
<!-- vale ste.SentenceLength = NO -->
<!-- vale ste.Gerunds = NO -->
<!-- vale ste.Semicolon = NO -->
<!-- vale ste.Modals = NO -->
<!-- vale write-good.E-Prime = NO -->
<!-- vale ste.PassiveVoice = NO -->
<!-- vale strunk.ActiveVoice = NO -->
<!-- vale ste.Ambiguity = NO -->
<!-- vale ste.OneInstruction = NO -->
<!-- vale ste.Articles = NO -->

# Situated Practice Contract Groundwork (T2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the closed, bounded, language-neutral practice vocabulary, input, budget, topology, and declaration-prelude contracts needed by T2 without admitting an input, advancing a practice, writing a receipt, or changing game physics.

**Architecture:** A new pure `babylon-practice-contract` boundary owns contract values and codecs that must agree across Rust, Python, BSL declarations, and future durability. It depends only on `babylon-kernel` hashing and does not depend on graph, BSL execution, tick session, persistence, client, or database crates. The BSL prelude declares organization fields and the ruled mode enum but seeds no graph and loads no rules. Every activation boundary remains a typed refusal until the separately owned Gate 3 and Gate 5 surfaces exist.

**Tech Stack:** YAML/PyYAML and Pydantic 2; Rust 2021 and `babylon-kernel`; BSL declaration preludes; SHA-256; checked `u32` arithmetic; checked-in JSONL codec vectors; scoped pytest and Cargo tests.

**Spec:** `docs/superpowers/specs/2026-08-23-neel-relational-territory-practice-design.md`, §§8.1–8.3, §8.6, §10, §11. The committed spec preserves the Director's 2026-08-18 ruling and is the portable implementation authority; the untracked inbox source is evidence, not an execution dependency.

## Global Constraints

- The T2 contract groundwork is not T2 activation. It must not add a pending input ledger, `TickSession::advance_with_inputs`, player gateway, policy producer, practice resolver, BSL practice rule, effect allowance, receipt, Archive subject, outbox, graph write, budget write, or durable row.
- Execution belongs to PER-55. Refresh Linear before Task 1, require PER-52 `Done`, and set PER-55 `In Progress`. PER-55 must continue to block PER-56. The separately owned activation sequence remains PER-20 → PER-22 → PER-26 → PER-27; this contract never provides an alternate player-action rail around it.
- The only V1 enum-key/display-label/machine mappings are `ORGANIZE / ORGANIZE = 1 = mobilize:canvass`, `AGITATE / AGITATE = 2 = mobilize:agitate`, and `MUTUAL_AID / MUTUAL-AID = 3 = aid` with no mode. The underscore form is the generated enum key; the hyphenated form is the exact player-facing label. Zero and all unassigned values refuse.
- `VerbModeV1` is exactly `CANVASS = 1` and `AGITATE = 2`. `aid` has `None` mode, not a synthetic `NONE` enum member. Display strings are not executable BSL verbs.
- `PracticeAuthorityKindV1` is exactly `PLAYER_SEAT = 1` and `DETERMINISTIC_POLICY = 2`. `PracticeTargetDomainV1` is the stable wire code table naming closed `NodeType` semantics; its only grounded V1 member is `SOCIAL_CLASS = 1`. This groundwork serializes that contract-local code but does not resolve a graph target. Gate 5/PER-27 owns the admission adapter that must resolve the textual `NodeType/SOCIAL_CLASS` member through `LoadedScenario.vocabulary`, require `GraphSubstrate::node_type_of(target) == "SOCIAL_CLASS"`, and require one sealed-snapshot territory shared by the actor's outbound `PRESENCE` and the target's outbound `TENANCY`. It refuses a missing or differently typed target and a missing locality join. No layer serializes a scenario-local BSL enum ordinal.
- All three V1 practice parameter allowlists are empty. Organize and Agitate admit no magnitude or other parameter, and this groundwork does not invent Mutual Aid goods parameters before the inventory contract exists. The schema retains the designed 16-parameter ceiling for a future schema version, but V1 vectors prove empty succeeds and the first parameter refuses; they do not fabricate a valid maximum-parameter intent.
- The BSL prelude declares `ConsciousnessTendency (LIBERAL FASCIST REVOLUTIONARY)` in its existing community order, plus `VerbMode (CANVASS AGITATE)`. Declaration order is hash-bearing registry identity and receives a mismatch sentinel and raw-content SHA-256 pin.
- The shared organization prelude declares one `organization/consciousness-tendency` field, one `organization/cadre-level`, one `organization/cohesion`, one `organization/active`, and one `organization/action-budget`. It cannot add a second line field, XP, Capacity, money, inventory, or an effect rule. Campaign setup remains the future sole initializer/writer for line and budget.
- `organization/action-budget` is an integral non-Currency decision quota. It has no conversion to Capacity, goods, labor, money, treasury, or escrow. The groundwork computes only detached values and does not access graph storage.
- Its BSL declaration is `int intensive`. The existing typechecker therefore rejects summation and an unweighted mean; `min`, `max`, and `count` remain legal intensive aggregations. This plan pins sum refusal only and does not describe the field as universally non-aggregating.
- Storage validation accepts only finite, non-negative binary64 values with zero fractional part and an exact `u32 -> f64 -> u32` round trip. Arithmetic uses checked `u32` before the sole conversion back to `f64`; overflow and invalid storage values refuse.
- `MAX_ORGANIZATIONS = 4_096` and `MAX_ORG_SOLIDARITY_EDGES_PER_ORG = 256` are contract and scenario ceilings, not `GameDefines`. A bounded, graph-free scenario preflight rejects the first 4,097th organization or 257th outbound organization-to-social-class `SOLIDARITY` edge before the real loader mutates the caller's graph. It does not filter an unbounded graph after load.
- `PracticeIntentV1` limits are exact: 16 parameters; 256 parameter bytes; 64 sorted unique evidence digests; 16,384 canonical bytes; and checked `resolve_tick = submit_after_tick + 1`.
- One resolve-tick batch contains at most 4,096 intents and at most one intent per organization. Detached authority-pair, quote-context, and batch validators enforce actor equality, source registration, last-committed tick, content digest, governed cost, shared resolve tick, actor uniqueness, and 4,096/4,097 before any future admission owner can persist a row.
- `PracticeInputAuthorityV1`, `PracticeIntentV1`, `OrganizationBudgetDeltaV1`, and `PracticeSubmissionRejectionV1` are sealed non-live values. Their codecs are public contract APIs only. Construction has no admission side effect. The rejection layout is exactly `schema_version_u16_be || submitted_bytes_digest_32 || reason_code_u16_be || last_committed_tick_u64_be || content_digest_32`; it has no nullable intent digest, detail field, retry tick, receipt identity, or durable identity.
- The only generated `GameDefines` additions are practice-budget terms consumed by the detached `compute_budget_delta` contract function: initial balance `1`, weekly credit cap `1`, storage ceiling `4`, and governed cost `1` for each closed practice id. No gain, efficacy, repression, aid-stock, or money coefficient is added.
- Current `mobilize` settings are not reinterpreted as T2 budget, organize, or agitate constants. The new terms do not wire an existing mechanic.
- Mutual Aid remains unavailable with `E-PRACTICE-UNWIRED`; this work creates neither inventory nor aid accounting. Repression, Backfire, line-return, and membership production remain unavailable.
- Contract input is bounded before parsing: 262,144 raw YAML bytes, 65,536 YAML events, nesting depth 16, 64 record declarations, 64 enum declarations, 64 fields per record, 256 members per enum, and 256 error codes. Aliases and duplicate mapping keys refuse. The JSONL corpus permits 2,097,152 raw bytes, 512 cases, 65,536 bytes per line, 128 UTF-8 bytes per case id, and JSON depth 32. Runtime sequence loops use a literal `.take(limit + 1)` in Rust or `itertools.islice(..., limit + 1)` in Python; no loop relies only on an earlier dynamic check.
- Do not run `mise run rust:check`. Run the listed Cargo legs serially; do not overlap heavy Cargo work. Use `SKIP=rust-full-gate` for any local push.

## Architectural Decision

The placement is architectural because it establishes one cross-language input/wire boundary that later Gate 3 persistence, Gate 5 admission, BSL declarations, and the client must consume without duplicating codecs. Task 1 creates `ADR227_practice_contract_groundwork.yaml` and its `ai/decisions/index.yaml` row. The ADR states that this crate is pure and non-live, that only the listed Python and BSL declarations mirror it, and that it cannot own the pending ledger, resolver, or durable envelope.

## Linear Ownership Preflight

| Issue | Required state at execution | Exact ownership |
|---|---|---|
| PER-52 | `Done` | T1 must have landed the governed `contracts` root and repository-hygiene test before this plan writes there. |
| PER-55 | Set to `In Progress` before Task 1 | This plan: freeze practice contract values, detached admission laws, budget math, real loader ceilings, and shared declarations. |
| PER-56 | `Todo`, blocked by PER-55 and PER-27 | Live Organize/Agitate admission and mechanics. This plan cannot satisfy it. |
| PER-20 / PER-22 / PER-26 / PER-27 | Refresh, but do not absorb | Committed envelope, fog/outbox, pending-input acceptance, and next-week bridge remain separate owners. |

Verify `contracts` remains an exact allowed repository root and `contractz` still fails the T1 hygiene test. Record the refreshed states in a PER-55 implementation comment. If PER-52 is not done or the blocker graph differs, stop before source changes and reconcile Linear.

## File Structure

| Path | Responsibility |
|---|---|
| `ai/decisions/ADR227_practice_contract_groundwork.yaml` | Records the placement, ownership, allowed dependencies, exact deferred boundaries, and no-live-activation decision. |
| `ai/decisions/index.yaml` | Registers ADR227 in the accepted-decision index. |
| `contracts/practice_contract_v1.yaml` | Sole language-neutral declaration of IDs, modes, wire fields, enum codes, byte layouts, limits, error codes, budget terms, and topology ceilings. |
| `contracts/practice_contract_v1_vectors.jsonl` | Shared valid/invalid authority, intent, budget-delta, rejection, and mapping codec vectors plus topology validation recipes with no canonical hex or digest. |
| `tools/generate_practice_contract_types.py` | Deterministically renders Python and Rust sealed structural types from the YAML contract; `--check` performs no write. |
| `src/babylon/contracts/practice_contract_v1_generated.py` | Generated frozen Pydantic records, closed enums, limits, and wire constants. |
| `src/babylon/contracts/practice_contract_v1.py` | Python parser, independent codecs/decoders, authority/quote/batch validators, mapping lookup, detached budget math, topology validation, and non-live dependency lookup. |
| `src/babylon/config/defines/organizations.py` | Defines `PracticeBudgetDefines` only for terms consumed by `compute_budget_delta`. |
| `src/babylon/config/defines/_assembler.py` | Adds `practice_budget: PracticeBudgetDefines` to `GameDefines` and YAML loading. |
| `src/babylon/data/defines.yaml` | Generated canonical YAML rendering of the six practice-budget terms. |
| `tests/unit/test_public_import_surface.py` | Proves the new category does not widen the pinned package-level public surface. |
| `tests/baselines/*.json` | Receives only the intentional `defines_hash` movement from the six new designed rows, through the required baseline ceremony; dense trace values must remain byte-identical. |
| `rust/Cargo.toml` | Registers the pure `babylon-practice-contract` crate. |
| `rust/Cargo.lock` | Records the new workspace package after one serialized lock refresh. |
| `rust/crates/babylon-practice-contract/Cargo.toml` | Declares `babylon-kernel` as its sole production dependency and `serde_json` only for JSONL-vector test parsing. |
| `rust/crates/babylon-practice-contract/src/generated.rs` | Generated closed Rust types, code tables, bounds, and wire constants. |
| `rust/crates/babylon-practice-contract/src/lib.rs` | Public contract API, crate-level safety lints, stable `PracticeContractError`, and non-live dependency lookup. |
| `rust/crates/babylon-practice-contract/src/codec.rs` | Independent Rust encoders/decoders and SHA-256 domain framing. |
| `rust/crates/babylon-practice-contract/src/budget.rs` | Checked detached `u32` budget transition and binary64 storage validator. |
| `rust/crates/babylon-practice-contract/src/admission.rs` | Pure authority-pair, quote-context, and resolve-batch validation with no persistence side effect. |
| `rust/crates/babylon-practice-contract/src/topology.rs` | Detached footprint validator and incremental scenario-load counter. |
| `rust/crates/babylon-practice-contract/tests/*.rs` | Rust structural, codec, budget, topology, and refusal vector tests. |
| `rust/crates/babylon-bsl/Cargo.toml` | Adds the one-way loader dependency on the pure contract crate. The practice crate remains graph/BSL-free. |
| `rust/crates/babylon-bsl/src/scenario.rs` | Feeds each resolved organization node and qualifying solidarity edge into the incremental counter before returning a loaded scenario. |
| `rust/crates/babylon-bsl/tests/practice_topology_admission.rs` | Real scenario-loader 4,096/4,097 organization and 256/257 edge refusal fixtures. |
| `rust/crates/babylon-tick/content/declarations/organization-practice.bscn` | Declaration-only organization/VerbMode prelude. |
| `rust/crates/babylon-tick/content/scenarios/organization-practice-contract.bscn` | Minimal no-rule topology fixture that consumes the prelude. |
| `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` | Removes promoted duplicate declarations and seeds the shared fields, including ActionBudget. |
| `rust/crates/babylon-tick/content/scenarios/community-*-conformance.bscn` | The nine enumerated community fixtures remove promoted duplicate declarations and seed ActionBudget on every active organization. |
| `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn` | Removes promoted organization declarations while retaining its separate worldview prelude. |
| `rust/crates/babylon-tick/content/content-sets.toml` | Declares the scenario/prelude/consumer relation. |
| `rust/crates/babylon-tick/tests/organization_practice_contract.rs` | Prelude registry order, identity pin, topology, and mismatch/refusal tests. |
| `rust/crates/babylon-tick/tests/tick_goldens.rs` | Re-pins only measured graph/world hashes whose scenario bytes/state intentionally change after declaration promotion. |
| `rust/crates/babylon-tick/src/bin/bsl_fuel_report.rs` | Supplies the same shared declaration preludes to every migrated fuel-report scenario. |
| `rust/crates/babylon-ls/src/pass.rs` | Composes every content-set prelude in manifest order instead of silently taking the first. |
| `tests/unit/contracts/test_practice_contract_v1_*.py` | Python codegen, codec, budget, topology, and refusal tests. |
| `tests/unit/config/test_practice_budget_defines.py` | Generated-YAML and detached-consumer parity test. |

---

### Task 1: Record ADR227 and create the closed practice-contract schema

**Files:**

- Create: `ai/decisions/ADR227_practice_contract_groundwork.yaml`
- Modify: `ai/decisions/index.yaml`
- Create: `contracts/practice_contract_v1.yaml`
- Create: `tests/unit/contracts/test_practice_contract_v1_codegen.py`

**Interfaces:**

- Produces `PracticeIdV1`, `VerbStemV1`, `VerbModeV1`, `PracticeAuthorityKindV1`, `PracticeTargetDomainV1`, `PracticeRejectionCodeV1`, `PracticeActivationBlockerV1`, `PracticeInputAuthorityV1`, `PracticeIntentV1`, `PracticeParameterV1`, `PolicyAuthorityPairV1`, `PracticeAuthorityContextV1`, `PracticeQuoteContextV1`, `SolidarityFootprintEdgeV1`, `OrganizationPracticeTopologyEdgeV1`, `OrganizationPracticeTopologyRowV1`, `OrganizationPracticeTopologyV1`, `OrganizationBudgetDeltaV1`, `PracticeSubmissionRejectionV1`, `PracticeBudgetTermsV1`, and the fixed contract-error registry.
- The contract declares exact byte layouts: `babylon.practice-input-authority.v1`, `babylon.practice-intent.v1`, and `babylon.organization-budget-delta.v1`, each followed by `0x00` before fixed-order big-endian fields.

- [ ] **Step 1: Write the ADR/index and schema red tests.** Assert ADR227 names `babylon-practice-contract` as pure, has no graph/BSL/tick/persistence/client dependency, and lists PER-20/PER-22/PER-26/PER-27 as activation blockers. Assert the index maps `ADR227_practice_contract_groundwork` to its exact file. Assert schema code tables are exactly:

  ```python
  assert contract.practice_ids == {"ORGANIZE": 1, "AGITATE": 2, "MUTUAL_AID": 3}
  assert contract.display_label("ORGANIZE") == "ORGANIZE"
  assert contract.display_label("AGITATE") == "AGITATE"
  assert contract.display_label("MUTUAL_AID") == "MUTUAL-AID"
  assert contract.machine_mapping("ORGANIZE") == ("mobilize", "CANVASS")
  assert contract.machine_mapping("AGITATE") == ("mobilize", "AGITATE")
  assert contract.machine_mapping("MUTUAL_AID") == ("aid", None)
  ```

  Assert the authority codes are `PLAYER_SEAT = 1` and `DETERMINISTIC_POLICY = 2`, and the only target-domain code is `SOCIAL_CLASS = 1`. Assert the exact rejection table is:

  ```text
  PRACTICE_UNWIRED = 1
  PRACTICE_STALE_CONTENT = 2
  PRACTICE_COST_MISMATCH = 3
  PRACTICE_AUTHORITY_UNREGISTERED = 4
  PRACTICE_ACTOR_MISMATCH = 5
  PRACTICE_DUPLICATE_ACTOR = 6
  PRACTICE_BATCH_LIMIT = 7
  PRACTICE_TICK_MISMATCH = 8
  PRACTICE_BUDGET_INSUFFICIENT = 9
  PRACTICE_TARGET_INELIGIBLE = 10
  PRACTICE_PENDING_DUPLICATE = 11
  ```

  Zero, 12, and every unknown `u16` refuse. Add failures for id `0`, id `4`, authority kind `0` or `3`, target domain `0` or `2`, mode `0`, mode `3`, `aid` with a mode, a non-`aid` mapping without a mode, a missing domain separator, and a contract field that lacks a declared byte order.

  Assert the exact `PracticeContractError` identity table is:

  ```text
  PRACTICE_DOMAIN = 1
  PRACTICE_SCHEMA_VERSION = 2
  PRACTICE_ENUM_CODE = 3
  PRACTICE_LENGTH = 5
  PRACTICE_TRUNCATED = 6
  PRACTICE_TRAILING_BYTES = 7
  PRACTICE_BOOLEAN = 9
  PRACTICE_PARAMETER = 10
  PRACTICE_PARAMETER_LIMIT = 11
  PRACTICE_PARAMETER_LENGTH = 12
  PRACTICE_EVIDENCE_LIMIT = 13
  PRACTICE_EVIDENCE_ORDER = 14
  PRACTICE_EVIDENCE_DUPLICATE = 15
  PRACTICE_TICK_OVERFLOW = 16
  PRACTICE_TICK_MISMATCH = 17
  PRACTICE_AUTHORITY_REGISTRY_LIMIT = 18
  PRACTICE_AUTHORITY_REGISTRY_ORDER = 19
  PRACTICE_AUTHORITY_REGISTRY_DUPLICATE = 20
  PRACTICE_AUTHORITY_UNREGISTERED = 21
  PRACTICE_ACTOR_MISMATCH = 22
  PRACTICE_AUTHORITY_CONTENT_MISMATCH = 23
  PRACTICE_QUOTE_CONTENT_MISMATCH = 24
  PRACTICE_QUOTE_COST_MISMATCH = 25
  PRACTICE_BATCH_LIMIT = 26
  PRACTICE_DUPLICATE_ACTOR = 27
  PRACTICE_BUDGET_NONFINITE = 28
  PRACTICE_BUDGET_NEGATIVE = 29
  PRACTICE_BUDGET_FRACTIONAL = 30
  PRACTICE_BUDGET_RANGE = 31
  PRACTICE_BUDGET_ROUNDTRIP = 32
  PRACTICE_BUDGET_INSUFFICIENT = 33
  PRACTICE_BUDGET_ARITHMETIC = 34
  PRACTICE_FOOTPRINT_LIMIT = 35
  PRACTICE_FOOTPRINT_ORDER = 36
  PRACTICE_FOOTPRINT_DUPLICATE = 37
  PRACTICE_FOOTPRINT_SOURCE = 38
  PRACTICE_FOOTPRINT_STRENGTH_NONFINITE = 39
  PRACTICE_FOOTPRINT_STRENGTH_NONPOSITIVE = 40
  PRACTICE_TOPOLOGY_ORGANIZATION_LIMIT = 41
  PRACTICE_TOPOLOGY_ORGANIZATION_ORDER = 42
  PRACTICE_TOPOLOGY_ORGANIZATION_DUPLICATE = 43
  PRACTICE_TOPOLOGY_BUDGET_MISSING = 44
  PRACTICE_TOPOLOGY_EDGE_ORDER = 45
  PRACTICE_TOPOLOGY_EDGE_DUPLICATE = 46
  ```

  Zero, the unassigned codes 4 and 8, code 47, and every other unknown `u16`
  refuse. Code 32 names non-canonical
  binary64 storage whose bits differ from `f64::from(value).to_bits()` after
  checked conversion; `-0.0` is its exact witness. The error type distinguishes these
  contract failures from the separate rejection-reason enum, even when both
  registries describe related admission facts. The YAML loader cannot trust a
  broken file's own table, so its non-wire `PracticeSchemaError` has the exact
  variants `SourceBytes`, `EventLimit`, `Alias`, `DuplicateKey`, `Depth`,
  `UnknownKey`, `MissingKey`, `DuplicateCode`, `MissingCode`, `InvalidLimit`,
  `CollectionLimit`, `FieldOrder`, and `MappingMismatch`.

  Freeze the only contract-error to submission-rejection aliases:

  ```text
  16,17 -> PRACTICE_TICK_MISMATCH (8)
  21,23 -> PRACTICE_AUTHORITY_UNREGISTERED (4)
  22 -> PRACTICE_ACTOR_MISMATCH (5)
  24 -> PRACTICE_STALE_CONTENT (2)
  25 -> PRACTICE_COST_MISMATCH (3)
  26 -> PRACTICE_BATCH_LIMIT (7)
  27 -> PRACTICE_DUPLICATE_ACTOR (6)
  33 -> PRACTICE_BUDGET_INSUFFICIENT (9)
  ```

  Assigned codes 1 through 3, 5 through 7, 9 through 15, 18 through 20,
  28 through 32, and 34 through 46 have no submission-rejection alias.
  `PRACTICE_UNWIRED`,
  `PRACTICE_TARGET_INELIGIBLE`, and `PRACTICE_PENDING_DUPLICATE` remain
  rejection-only values whose live production is deferred.

- [ ] **Step 2: Run the focused test and record red.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py`

  Expected: FAIL because the ADR, index entry, and contract schema are absent.

- [ ] **Step 3: Write ADR227.** State the new crate’s exact dependency direction; make no claim of a practice admission path; preserve ADR224’s default-deny Intent allowance table; record that this contract’s types are future inputs to Gate 3/Gate 5 rather than a persistence substitute; and list all forbidden current outputs from Global Constraints.

- [ ] **Step 4: Write the closed YAML schema.** Define unsigned widths, big-endian field order, every enum member/code, all limits, stable error identities, the exact enum-key/display-label/machine mapping table, and the designed terms `{initial: 1, weekly_credit_cap: 1, storage_ceiling: 4, organize_cost: 1, agitate_cost: 1, mutual_aid_cost: 1}`. Freeze `PracticeParameterV1`'s structural `(key_u8, value_kind_u8, value_length_u16_be, value_bytes)` layout but give every V1 practice an empty key allowlist; any non-empty V1 parameter sequence returns `PRACTICE_PARAMETER`. Define authority-context policy pairs as a sorted unique set bounded to 4,096.

  Define `SolidarityFootprintEdgeV1` with `source_org_node_id_u64`,
  `target_domain_u8`, `target_class_node_id_u64`, and
  `strength_f64_bits_u64`. Its relation type is intrinsically the textual
  `EdgeType/SOLIDARITY` member. Its canonical graph identity is the existing
  `(source, target, type)` triple; `target_domain_u8` qualifies the witness but
  does not enter graph identity. No synthetic dyadic `EdgeId` or scenario-local
  ordinal enters the contract.

  The only admitted domain is
  `PracticeTargetDomainV1::SocialClass`. Strength must be finite and strictly
  positive, and Gate 5 constructs this detached witness only after its
  adapter verifies the target's actual textual node type. The schema and
  both generated structs retain that exact source/domain/target/strength field
  order. This detached witness has no standalone V1 wire envelope; callers
  cannot infer one from in-memory layout.

  Freeze these detached topology records:

  - `OrganizationPracticeTopologyEdgeV1` has exact generated field order
    `target_domain`, `target_class_node_id_u64`. Its source is the enclosing
    organization row and its type is intrinsically `EdgeType/SOLIDARITY`. It has
    no strength field because the topology ceiling counts typed edges regardless
    of replenishment qualification.

  - `OrganizationPracticeTopologyRowV1` has exact generated field order
    `node_id_u64`, `active_bool`, optional
    `action_budget_storage_f64_bits_u64`, and a bounded sequence named `edges`
    of topology-edge rows.

  - `OrganizationPracticeTopologyV1` has one bounded sequence named
    `organizations`. It admits at most 4,096 rows in strictly ascending unique
    `node_id_u64` order.

  - Each child sequence admits at most 256 edges in strictly ascending unique
    `target_class_node_id_u64` order, and each typed domain must be
    `SocialClass`.

  - An active row requires budget bits. An inactive row may omit them. When any
    row supplies bits, the exact binary64-to-`u32` validator still applies.

  - These three topology records have no V1 wire envelope and no graph
    authority.

  Define the rejection payload exactly as fixed fields `schema_version`, `submitted_bytes_digest`, closed `reason_code`, `last_committed_tick`, and `content_digest`; it carries no nullable field, retry hint, receipt, event, or durable identity. Record each substantive limit, code table, and budget term as `Designed` with its play-purpose statement.

- [ ] **Step 5: Implement the bounded schema reader used only by tests and generation.** Refuse more than 262,144 raw bytes; scan at most 65,537 YAML events; reject alias events, duplicate mapping keys, and depth 17 before model creation. Reject an unknown schema field, a duplicate or missing numeric code, a mismatched mapping, a zero or overflowed limit, any meta-model collection beyond its literal ceiling, and every undeclared field. Every collection walk uses `islice(collection, declared_limit + 1)` and refuses the extra witness. Stage the normalized model and never write generated output.

- [ ] **Step 6: Make the schema/ADR test green and commit.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py`

  Run: `PYTHONPATH="$PWD/src" mise run check:vocabulary`

  Run: `vale ai/decisions/ADR227_practice_contract_groundwork.yaml`

  Commit: `docs(adr): record pure T2 practice-contract groundwork boundary`

### Task 2: Generate sealed Python/Rust types and establish the pure crate

**Files:**

- Create: `tools/generate_practice_contract_types.py`
- Create: `src/babylon/contracts/practice_contract_v1_generated.py`
- Create: `rust/crates/babylon-practice-contract/Cargo.toml`
- Create: `rust/crates/babylon-practice-contract/src/generated.rs`
- Create: `rust/crates/babylon-practice-contract/src/lib.rs`
- Modify: `rust/Cargo.toml`
- Modify: `rust/Cargo.lock`
- Create: `rust/crates/babylon-practice-contract/tests/generated_contract.rs`
- Modify: `tests/unit/contracts/test_practice_contract_v1_codegen.py`

**Interfaces:**

```python
def load_practice_contract(path: Path) -> PracticeContractSpec: ...
def render_python(spec: PracticeContractSpec) -> str: ...
def render_rust(spec: PracticeContractSpec) -> str: ...
```

```rust
pub enum PracticeIdV1 { Organize = 1, Agitate = 2, MutualAid = 3 }
pub enum VerbStemV1 { Mobilize = 1, Aid = 2 }
pub enum VerbModeV1 { Canvass = 1, Agitate = 2 }
pub enum PracticeAuthorityKindV1 { PlayerSeat = 1, DeterministicPolicy = 2 }
pub enum PracticeTargetDomainV1 { SocialClass = 1 }
pub enum PracticeRejectionCodeV1 { /* exact YAML-declared u16 codes 1..=11 */ }
pub struct MachineVerbV1 { pub stem: VerbStemV1, pub mode: Option<VerbModeV1> }
pub struct PracticeInputAuthorityV1 { /* YAML-declared fields */ }
pub struct PracticeIntentV1 { /* YAML-declared fields */ }
pub struct PracticeAuthorityContextV1 { /* player and bounded policy authority */ }
pub struct PracticeQuoteContextV1 { /* committed tick/content/cost terms */ }
pub struct SolidarityFootprintEdgeV1 { /* exact source/domain/target/strength */ }
pub struct OrganizationPracticeTopologyEdgeV1 { /* exact domain/target */ }
pub struct OrganizationPracticeTopologyRowV1 { /* exact node/active/budget/edges */ }
pub struct OrganizationPracticeTopologyV1 { /* bounded ordered rows */ }
pub struct OrganizationBudgetDeltaV1 { /* YAML-declared fields */ }
pub struct PracticeSubmissionRejectionV1 { /* YAML-declared fields */ }
pub enum PracticeContractError { /* fixed YAML-declared error codes */ }
```

- [ ] **Step 1: Write generator/crate red tests.** Assert `--check` fails before
  outputs exist; generated Python models use
  `ConfigDict(frozen=True, extra="forbid", strict=True)`; Rust
  conversion `PracticeIdV1::try_from(0_u8)` and `try_from(4_u8)` return named
  errors; and all three valid practice codes map to the exact machine stem/mode
  table. Pin all 44 assigned contract-error names and codes, plus refusal of
  unassigned codes 4 and 8. Pin the exact
  source/domain/target/strength footprint field order, exact domain/target
  topology-edge order, exact node/active/budget/edges topology-row order, and
  sole `organizations` parent field. Assert the crate has only `babylon-kernel`
  in production dependencies,
  with `serde_json` allowed solely for test-vector parsing.

  Assert strict Python construction rejects integer `0` or `1` and strings for
  `active_bool`, raw integers or another enum type for closed enum fields,
  text for bytes, 31-byte and 33-byte values for every digest field, and mutable
  lists for tuple fields. Assert each generated unsigned integer field,
  including optional `u64` storage bits, rejects `-1` and its declared width's
  max-plus-one value during model construction. Those width and digest-shape
  refusals are local Pydantic errors, not `PracticeContractError` values. Keep
  tuple length unconstrained at model construction and prove the public
  validators return their assigned max-plus-one errors.

- [ ] **Step 2: Run the red tests.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test generated_contract --locked`

  Expected: FAIL because generated outputs and crate registration do not exist.

- [ ] **Step 3: Implement the deterministic generator.** Its CLI is `uv run python tools/generate_practice_contract_types.py [--check] [--contract PATH] [--python-out PATH] [--rust-out PATH]`. It writes both outputs only after parsing and rendering both into private strings, and generated Python values use exact-length 32-byte fields for digests, `bool`, width-constrained strict integers for every YAML-declared `u8`/`u16`/`u32`/`u64`, an optional width-constrained strict `u64` for budget bits, and shape-only tuples for declared sequences. Every Pydantic record uses `ConfigDict(frozen=True, extra="forbid", strict=True)`, so integers, strings, cross-enum values, and mutable sequences cannot coerce into contract fields, while the public codecs and validators own every sequence ceiling and max-plus-one `PracticeContractError` witness.

  The generator expresses unsigned widths and exact digest length with Pydantic constraints, so Python constructor shape agrees with Rust's `[u8; 32]`, `u8`, `u16`, `u32`, `u64`, and `Option<u64>` types. Generated Rust values use `Vec` only at the contract's declared sequences. The generated records are ordinary typed values and do not derive a general-purpose serialization format.

  All generator loops carry the literal meta-model ceilings from Global Constraints. Neither output has a graph node, EdgeType, BSL effect, session, database, or filesystem API.

- [ ] **Step 4: Register the crate and public surface.** Add `crates/babylon-practice-contract` to `rust/Cargo.toml`. Declare `babylon-kernel` as the sole production path dependency and `serde_json` solely in `[dev-dependencies]` for root JSONL-vector parsing. Put `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` at the crate root. `lib.rs` exports generated types plus pure `codec`, `admission`, `budget`, and `topology` modules, but no stateful admission, persistence, or execution method.

  Run one serialized `cd rust && cargo check -p babylon-practice-contract` to register the package, review that only the new workspace package changes `Cargo.lock`, then use `--locked` for every later Cargo command.

- [ ] **Step 5: Generate source and make the type tests green.**

  Run: `uv run python tools/generate_practice_contract_types.py`

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test generated_contract --locked`

- [ ] **Step 6: Add generator mutations.** Use `/tmp/practice-contract-invalid.yaml` to remove `AGITATE`, then to add a fourth mode. Assert the schema reader returns a stable contract-schema error and `--check` does not update either checked-in generated file. Restore source files exactly.

- [ ] **Step 7: Format, lint, and commit.**

  Run: `uv run python tools/generate_practice_contract_types.py --check`

  Run: `cd rust && cargo fmt --all -- --check`

  Run: `cd rust && cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings -D clippy::pedantic`

  Commit: `feat(contract): generate closed T2 practice wire types`

### Task 3: Implement independent wire codecs, vectors, and refusal values

**Files:**

- Create: `contracts/practice_contract_v1_vectors.jsonl`
- Create: `src/babylon/contracts/practice_contract_v1.py`
- Create: `rust/crates/babylon-practice-contract/src/admission.rs`
- Create: `rust/crates/babylon-practice-contract/src/codec.rs`
- Create: `tests/unit/contracts/test_practice_contract_v1_codec.py`
- Create: `tests/unit/contracts/test_practice_contract_v1_admission.py`
- Create: `rust/crates/babylon-practice-contract/tests/codec_vectors.rs`
- Create: `rust/crates/babylon-practice-contract/tests/admission.rs`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`

**Interfaces:**

```python
class PracticeContractViolation(ValueError):
    error: PracticeContractError

def encode_input_authority(value: PracticeInputAuthorityV1) -> bytes: ...
def encode_intent(value: PracticeIntentV1) -> bytes: ...
def encode_budget_delta(value: OrganizationBudgetDeltaV1) -> bytes: ...
def encode_rejection(value: PracticeSubmissionRejectionV1) -> bytes: ...
def decode_input_authority(payload: bytes) -> PracticeInputAuthorityV1: ...
def decode_intent(payload: bytes) -> PracticeIntentV1: ...
def decode_budget_delta(payload: bytes) -> OrganizationBudgetDeltaV1: ...
def decode_rejection(payload: bytes) -> PracticeSubmissionRejectionV1: ...
def input_authority_digest(value: PracticeInputAuthorityV1) -> bytes: ...
def encode_intent_parameters(value: PracticeIntentV1) -> bytes: ...
def intent_digest(value: PracticeIntentV1) -> bytes: ...
def parameter_bytes_digest(value: PracticeIntentV1) -> bytes: ...
def target_selection_policy_digest(
    target_domain: PracticeTargetDomainV1, target_node_id: int,
) -> bytes: ...
def budget_delta_digest(value: OrganizationBudgetDeltaV1) -> bytes: ...
def submission_rejection_alias(
    error: PracticeContractError,
) -> PracticeRejectionCodeV1 | None: ...
def rejection_for(
    *, submitted_bytes_digest: bytes, reason_code: PracticeRejectionCodeV1,
    last_committed_tick: int, content_digest: bytes,
) -> PracticeSubmissionRejectionV1: ...
def validate_authority_pair(
    authority: PracticeInputAuthorityV1, intent: PracticeIntentV1,
    context: PracticeAuthorityContextV1,
) -> None: ...
def validate_quote_context(
    intent: PracticeIntentV1, context: PracticeQuoteContextV1,
) -> None: ...
def validate_resolve_batch(
    intents: Sequence[PracticeIntentV1], expected_resolve_tick: int,
) -> None: ...
```

Every Python codec, decoder, admission validator, budget transition, and
topology validator raises `PracticeContractViolation` for a governed contract
refusal. Its immutable `error` field is the exact generated
`PracticeContractError`; tests inspect that field and never exception text. The
constructor requires `type(error) is PracticeContractError`. Pydantic record
shape failures and wrong Python API argument types remain local validation or
`TypeError`/`ValueError` misuse and never receive this wrapper.

```rust
pub fn encode_input_authority(value: &PracticeInputAuthorityV1) -> Result<Vec<u8>, PracticeContractError>;
pub fn input_authority_digest(value: &PracticeInputAuthorityV1) -> Result<[u8; 32], PracticeContractError>;
pub fn encode_intent(value: &PracticeIntentV1) -> Result<Vec<u8>, PracticeContractError>;
pub fn encode_intent_parameters(value: &PracticeIntentV1) -> Result<Vec<u8>, PracticeContractError>;
pub fn intent_digest(value: &PracticeIntentV1) -> Result<[u8; 32], PracticeContractError>;
pub fn parameter_bytes_digest(value: &PracticeIntentV1) -> Result<[u8; 32], PracticeContractError>;
pub fn target_selection_policy_digest(
    target_domain: PracticeTargetDomainV1,
    target_node_id: u64,
) -> [u8; 32];
pub fn encode_budget_delta(value: &OrganizationBudgetDeltaV1) -> Result<Vec<u8>, PracticeContractError>;
pub fn budget_delta_digest(value: &OrganizationBudgetDeltaV1) -> Result<[u8; 32], PracticeContractError>;
pub const fn submission_rejection_alias(
    error: PracticeContractError,
) -> Option<PracticeRejectionCodeV1>;
pub fn encode_rejection(value: &PracticeSubmissionRejectionV1) -> Result<Vec<u8>, PracticeContractError>;
pub fn decode_input_authority(payload: &[u8]) -> Result<PracticeInputAuthorityV1, PracticeContractError>;
pub fn decode_intent(payload: &[u8]) -> Result<PracticeIntentV1, PracticeContractError>;
pub fn decode_budget_delta(payload: &[u8]) -> Result<OrganizationBudgetDeltaV1, PracticeContractError>;
pub fn decode_rejection(payload: &[u8]) -> Result<PracticeSubmissionRejectionV1, PracticeContractError>;
pub fn validate_authority_pair(
    authority: &PracticeInputAuthorityV1,
    intent: &PracticeIntentV1,
    context: &PracticeAuthorityContextV1,
) -> Result<(), PracticeContractError>;
pub fn validate_quote_context(
    intent: &PracticeIntentV1,
    context: &PracticeQuoteContextV1,
) -> Result<(), PracticeContractError>;
pub fn validate_resolve_batch(
    intents: &[PracticeIntentV1],
    expected_resolve_tick: u64,
) -> Result<(), PracticeContractError>;

impl PracticeIntentV1 {
    pub const fn resolve_tick(&self) -> u64;
    pub const fn practice_id(&self) -> PracticeIdV1;
    pub const fn target_domain(&self) -> PracticeTargetDomainV1;
    pub const fn target_node_id(&self) -> u64;
    pub const fn quoted_action_budget_cost(&self) -> u32;
}
```

- [ ] **Step 1: Write valid/invalid codec and admission vector red tests.** The root JSONL corpus contains canonical hex and SHA-256 for one authority per source kind, each practice mapping, one empty-parameter intent for each practice, a maximum-evidence intent, a valid budget delta, and rejection values for all eleven exact numeric reason codes; every valid codec vector must round-trip through the independent decoder and re-encode to the same bytes. Invalid byte entries cover bad domain, bad version, unknown authority/target/rejection code, resolve-tick overflow, resolve tick not equal to submit plus one, zero/unassigned practice id, first V1 parameter, 257-byte structural parameter input, evidence duplicate/unsorted/65th, oversized intent, truncation at every field boundary, and a trailing byte. Typed fixture readers separately reject 31-byte and 33-byte values for every digest field, `-1` and max-plus-one for representative `u8`/`u16`/`u32`/`u64` fields, and every attempted missing, extra, or nullable rejection field before a wire codec call.

  Those object-shape failures are not cross-language `PracticeContractError` values. Bad stem/mode pairs remain Task 1 `PracticeSchemaError::MappingMismatch` cases because no wire record carries a caller-supplied machine mapping. The manifest records that the 16-parameter ceiling has no valid V1 witness because every practice allowlist is empty; do not invent one.

  Require exact contract errors through `PracticeContractViolation.error`, not
  exception text: bad domain returns code 1,
  version 2, and unknown enum 3. Oversize returns 5, truncation 6, trailing bytes
  7, and a non-`0`/`1` encoded boolean 9. The first structurally valid but
  unsupported V1 parameter returns 10; parameter count 17 returns 11; and
  structural parameter byte 257 returns 12. Evidence item 65 returns 13,
  descending order 14, and a duplicate 15. Checked tick overflow returns 16 and
  a semantic tick mismatch 17. The authority, quote, batch, budget, footprint,
  and topology suites pin codes 18 through 46 at each
  corresponding boundary from Task 1's table.

  Add authority-pair vectors for a valid player seat, valid registered policy,
  wrong player actor, wrong gateway digest, unregistered policy pair, duplicate
  policy registration, and authority/intent actor mismatch. Add quote-context
  vectors for current content and cost, stale content digest, stale submit tick,
  wrong resolve tick, zero quote, and every practice cost mismatch. Pin
  `input_authority_digest`, `intent_digest`, `parameter_bytes_digest`,
  `target_selection_policy_digest`, and `budget_delta_digest` in both runtimes.
  The fixed-target vector hashes the exact domain-separated preimage from the
  specification; neither runtime may duplicate that framing at a call site.
  Add compact generated-batch vector recipes for counts 0, 4,096, and 4,097;
  shared or mismatched resolve ticks; and duplicate actor. The test expands each
  recipe with a literal bounded loop, so no JSONL line exceeds its line limit.

  Assert Python and Rust consume the exact same corpus: Rust uses `include_str!("../../../../contracts/practice_contract_v1_vectors.jsonl")`; do not copy vectors into a crate fixture. Both readers refuse more than 2,097,152 raw bytes before splitting; inspect at most 513 lines; refuse a 65,537-byte line, 129-byte case id, depth 33, duplicate case, unknown case kind/field, or trailing JSON token. Every case walk uses the literal plus-one witness.

- [ ] **Step 2: Run codec tests and record red.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codec.py tests/unit/contracts/test_practice_contract_v1_admission.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test codec_vectors --test admission --locked`

  Expected: FAIL because codec APIs and vectors do not exist.

- [ ] **Step 3: Implement bounded Python codecs and decoders.** Bound corpus bytes/lines/depth before JSON parsing. Validate all record counts and sequence order before allocating encoded output. Use explicit `u16`/`u32`/`u64` maximum checks for total byte length and tick addition. Encode only the exact fixed-order big-endian fields and declared length frames. Decode with an index checked before every read and require exact full consumption; never pad, default, ignore trailing bytes, or map an unknown `u16` reason.

  Expose the read-only intent accessors and symmetric Python/Rust digest APIs
  listed above. `encode_intent_parameters` emits exactly
  `parameters_count_u16_be || length_framed_parameters`;
  `parameter_bytes_digest` hashes
  `ASCII("babylon.practice-parameter-bytes.v1") || 0x00 ||` those bytes.
  The typed Rust `target_selection_policy_digest` is infallible and alone owns
  `ASCII("babylon.fixed-target-selection.v1") || 0x00 || target_domain_u8 ||
  target_node_id_u64_be`, because its closed enum and `u64` inputs have total
  fixed encodings. Python requires
  `type(target_domain) is PracticeTargetDomainV1` and rejects a raw integer or
  another `IntEnum` type as non-wire `TypeError` misuse. It also requires
  `type(target_node_id) is int` and `0 <= target_node_id <= u64::MAX`, so a
  Boolean, negative integer, or max-plus-one integer refuses as local
  `TypeError` or `ValueError` before byte encoding. Python and Rust pin the same
  fixed-target preimage and digest, plus the same empty-V1-parameter bytes and
  digest.

  Both encoders and decoders use this exact parameter refusal precedence:
  preflight count 17 as code 11; validate each declared or actual value length
  and framing as code 12 or code 6; only then refuse the first structurally
  valid V1 parameter as code 10. For evidence, preflight item 65 as code 13,
  then test equality before descending order so a duplicate returns code 15 and
  another nonascending pair returns code 14.

  Generate `submission_rejection_alias` from Task 1's exact alias table. Test
  every mapped and unmapped contract code in both languages. The lookup returns
  metadata only; it cannot construct a context-free rejection or perform live
  admission. Python requires `type(error) is PracticeContractError`; a raw
  integer or `PracticeRejectionCodeV1` with an equal integer value raises a
  non-wire `TypeError` instead of exploiting `IntEnum` equality.

  Use `hashlib.sha256` only over the declared domain-tagged bytes. Bound
  parameter and evidence walks with `islice` at 17 and 65 witnesses
  respectively. Python raises only `PracticeContractViolation` with a fixed
  generated `PracticeContractError` for governed refusals; Rust returns that
  same enum. No generic parse error or exception text becomes a wire identity.

- [ ] **Step 4: Implement the independent Rust codecs and decoders.** Build private `Vec<u8>` buffers and a checked slice cursor; do not invoke Python and do not use `serde_json` for wire records. Walk parameters with `.take(MAX_PARAMETERS + 1)` and evidence with `.take(MAX_EVIDENCE_DIGESTS + 1)`. Use `u64::checked_add`, `u16::try_from`, and `u32::try_from` on every narrowing boundary. A decoder validates domain bytes, exact enums, lengths, sequence order, and full consumption before returning a value.

  Use `babylon_kernel::sha256_of` only after successful byte construction. `PRACTICE_UNWIRED` is a valid rejection code but never substitutes for malformed bytes.

- [ ] **Step 5: Implement pure authority, quote, and batch validation.** Player validation requires authority actor = intent actor = `player_org_id` and exact player-gateway content digest. Policy validation requires authority actor = intent actor and one exact sorted unique `(producer_content_digest, actor_org_id)` registry pair. Quote validation requires `submit_after_tick == last_committed_tick`, checked `resolve_tick == last_committed_tick + 1`, exact content digest, and quoted cost equal to the exhaustive practice-id cost from `PracticeBudgetTermsV1`. Batch validation reads at most 4,097 intents, requires the one expected resolve tick, and rejects the first repeated actor or 4,097th item. These functions return a verdict only; they cannot encode a pending row, mutate a budget, or construct a context-free rejection.

- [ ] **Step 6: Make codec and admission suites green.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codec.py tests/unit/contracts/test_practice_contract_v1_admission.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test codec_vectors --test admission --locked`

- [ ] **Step 7: Add atomic-refusal and permutation mutations.** Swap two evidence digests, mutate one parameter length, append one trailing byte, force `submit_after_tick = u64::MAX`, duplicate one authority registry pair, and duplicate one actor at batch item 4,096. Assert both languages refuse and return no bytes/digest/admission result. Permute the required lexicographic evidence or policy-pair order and assert refusal rather than caller normalization.

- [ ] **Step 8: Verify and commit.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py tests/unit/contracts/test_practice_contract_v1_codec.py tests/unit/contracts/test_practice_contract_v1_admission.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test generated_contract --test codec_vectors --test admission --locked`

  Commit: `feat(contract): pin T2 practice authority and intent codecs`

### Task 4: Add checked ActionBudget math, topology ceilings, and generated defines parity

**Files:**

- Create: `rust/crates/babylon-practice-contract/src/budget.rs`
- Create: `rust/crates/babylon-practice-contract/src/topology.rs`
- Create: `rust/crates/babylon-practice-contract/tests/budget.rs`
- Create: `rust/crates/babylon-practice-contract/tests/topology.rs`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`
- Modify: `src/babylon/contracts/practice_contract_v1.py`
- Modify: `src/babylon/config/defines/organizations.py`
- Modify: `src/babylon/config/defines/_assembler.py`
- Modify: `src/babylon/data/defines.yaml` through `tools/generate_defines_config.py`
- Modify: `contracts/fixtures/detroit_windsor_rtd_v1_world_identity.json` only
  for the dependent definitions and nominal-world identity movement.
- Modify if its sealed reference changes:
  `contracts/fixtures/detroit_windsor_rtd_v1_admin_control.json`.
- Modify if the sealed control changes:
  `contracts/relational_territory_dossier_v1_vectors.jsonl`.
- Modify: `rust/crates/babylon-bsl/Cargo.toml`
- Modify: `rust/crates/babylon-bsl/src/scenario.rs`
- Create: `rust/crates/babylon-bsl/tests/practice_topology_admission.rs`
- Modify: `docs/reference/bsl-language.rst` only to register the five exact practice-topology load errors.
- Modify: `rust/Cargo.lock`
- Create: `tests/unit/contracts/test_practice_contract_v1_budget.py`
- Create: `tests/unit/contracts/test_practice_contract_v1_topology.py`
- Create: `tests/unit/config/test_practice_budget_defines.py`
- Modify: `tests/unit/test_public_import_surface.py` only to assert the existing public-name set remains exact, not to add the new category type.

**Interfaces:**

```rust
pub fn read_action_budget(storage: f64) -> Result<u32, PracticeContractError>;
pub fn write_action_budget(value: u32) -> f64;
pub fn compute_budget_delta(
    tick: u64, actor_node_id: u64, pre_action_world_hash: [u8; 32],
    budget_before: u32, practice: Option<PracticeIdV1>,
    footprint_edges: &[SolidarityFootprintEdgeV1],
    terms: PracticeBudgetTermsV1,
) -> Result<OrganizationBudgetDeltaV1, PracticeContractError>;
pub fn validate_topology(topology: &OrganizationPracticeTopologyV1) -> Result<(), PracticeContractError>;
pub struct PracticeTopologyLoadCounter { /* private bounded counters */ }
```

The Python module exposes exact counterparts and raises the Task 3
`PracticeContractViolation` on every governed refusal. `compute_budget_delta` derives
`governed_cost` from `practice` and derives `footprint_count` from the validated
edge witnesses; `None` is the no-intent zero-cost transition. Each edge must
appear in ascending `(source_org_node_id, target_class_node_id)` order. The
complete fixed-type triple must be unique, the requested actor must be the
source, `target_domain` must equal `SOCIAL_CLASS`, and strength bits must be
positive and finite. The function implements
`min(storage_ceiling, budget_before - governed_cost + min(derived_count,
weekly_credit_cap))`.

Callers cannot quote or substitute a cost or count. The
function does not inspect a graph, prove the target's actual graph type, decide
eligibility, or write a budget.

- [ ] **Step 1: Write budget/topology/defines red tests.** Cover `0`, `-0.0`,
  `u32::MAX`, valid exact binary64 storage, negative, fractional, NaN, infinity,
  `u32::MAX + 1.0`, insufficient-budget subtraction refusal as code 33, checked
  addition overflow as code 34, cap binding, the `None` zero-cost transition,
  and each practice-derived cost.
  Require `-0.0` to return code 32 under the bitwise cast-back check. Prove the
  API has no caller-supplied cost or footprint-count argument. Assert the budget
  delta records the supplied snapshot hash, derived governed cost, exact derived
  footprint count, raw credit, credited credit, bound flag, and final balance.

  For the detached footprint, cover 0, 256, and 257 edges; ascending,
  descending, and duplicate `(source, target, SOLIDARITY)` identity; wrong
  source actor; repeated target as a duplicate edge refusal; zero, negative,
  NaN, positive infinity, and valid positive strength bits. Separately prove
  generated target-domain construction refuses raw code 0 or 2, while the
  detached edge exposes only the typed `SocialClass` value. For topology, cover
  0, 4,096, and 4,097 organizations; ascending, descending, and duplicate
  organization ids; inactive rows with absent and valid budget bits; active
  rows with absent budget bits; an inactive row with invalid supplied budget
  bits that must still return its exact storage code; and 0, 256, and 257
  outbound organization-to-social-class solidarity edges; inactive
  organizations; and active organizations with missing or invalid budget
  storage. Require the exact contract-error codes 28 through 46 for every
  applicable budget, footprint, and topology refusal. The pure validator must
  not accept a
  `PRESENCE`/`TENANCY`/`MEMBERSHIP`/`SOLIDARITY` topology as execution
  authorization. The detached validator bounds authored topology; it does not
  prove the Gate 5 locality join.

  For defines, assert `GameDefines().practice_budget` exactly matches `PracticeBudgetTermsV1.from_defines(GameDefines())`, that `initial >= min(organize_cost, agitate_cost)`, `storage_ceiling >= initial`, all three practice costs are in `1..=u32::MAX`, the other three terms are in `0..=u32::MAX`, and regenerated `defines.yaml` remains current. Add one invalid-config test for each negative, zero-cost, above-`u32::MAX`, initial-above-ceiling, and initial-below-both-wired-cost case. Assert `PracticeBudgetDefines` is not added to the package-level public import surface.

- [ ] **Step 2: Run the red tests.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_budget.py tests/unit/contracts/test_practice_contract_v1_topology.py tests/unit/config/test_practice_budget_defines.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test budget --test topology --locked`

  Run: `cd rust && cargo test -p babylon-bsl --test practice_topology_admission --locked`

  Expected: FAIL because no detached budget/topology implementation, real loader counter, or `practice_budget` define category exists.

- [ ] **Step 3: Implement pure checked footprint and budget math.** Reject
  storage before conversion; use `f64::is_finite`, `fract() == 0.0`, explicit
  `0.0..=f64::from(u32::MAX)` range checks, and a cast only after those checks.
  Require `storage.to_bits() == f64::from(value).to_bits()` for canonical
  cast-back equality, then use `checked_sub`, `checked_add`, and `f64::from`.
  Inspect at most 257 footprint edges; validate complete edge
  identity/source/target/strength and derive count only after the entire bounded
  witness is valid. Derive governed cost through an exhaustive `PracticeIdV1`
  match; `None` alone supplies zero.

  Check `budget_before < governed_cost` first and return code 33, so subtraction
  underflow and insufficient budget are one reachable identity. Code 34 belongs
  only to checked addition overflow after valid subtraction. Pin it with
  `budget_before = u32::MAX`, `practice = None`, one qualifying credit, and a
  `u32::MAX` storage ceiling; do not clamp an overflowing intermediate sum.

  Write no fallback/coercion path. Construct a delta only after the entire transition is valid. The returned value's codec is Task 3's codec, not a receipt.

- [ ] **Step 4: Implement detached and real-loader topology validation.** The
  pure `PracticeTopologyLoadCounter` has constant-time
  `observe_organization(organization_key, active, action_budget_storage)` and
  `observe_solidarity_edge(source_organization_key, target_domain, target_key)`
  methods. It admits only `PracticeTargetDomainV1::SocialClass`. It keys dyadic
  identity as the fixed textual `SOLIDARITY` type plus source and target;
  `target_domain` qualifies the row but does not change that identity. It
  refuses organization 4,097 and each source organization's edge 257
  immediately, and caps its `finish` walk at 4,097 organizations.

  `validate_topology` feeds an already-qualified detached record through the
  same counter and refuses unqualified rows rather than scanning or filtering
  them. Counter keys are opaque and validation-local: detached validation uses
  real node ids, while scenario preflight uses bounded declaration ordinals
  assigned before graph mutation. A local key is never emitted, digested, or
  stored in either detached record, so it cannot become contract identity. This
  pure contract does not claim to have read the graph.

  Add `babylon-practice-contract` as a production dependency of `babylon-bsl`
  with no reverse dependency, and apply the 4,194,304 UTF-8 byte source bound to
  every scenario before `read_all`. After the existing bounded reader returns,
  build the final declaration registry from the composed preludes and
  scenario-local declarations without a graph call, then trigger the practice
  preflight whenever that registry contains `organization/action-budget`.
  Before any topology walk, require `organization/action-budget` and
  `organization/active` to exist with exact `int intensive` signatures; a
  missing active declaration, wrong field type, or wrong aggregation kind
  returns `E-LOAD-065` before graph mutation. Walk at most 65,536 scenario
  body forms and 1,048,576 AST nodes with an explicit stack bounded to 65,536
  entries and depth 256. Every traversal uses a literal fixed upper loop after
  a checked count preflight.

  Before the first `GraphSubstrate` call, run one graph-free
  `PracticeTopologyPreflight` over the already parsed body whose local-name
  table admits at most 65,536 names and assigns each declaration a
  validation-local ordinal for pre-mutation joins. A separate
  relevant-organization table admits at most 4,096 rows; the 4,097th
  organization is the refusal, not the 4,097th unrelated node. The preflight
  creates an organization row only after exact textual
  `NodeType/ORGANIZATION` qualification, and that pure row remains detached
  without graph authority. During its bounded source-order attribute walk, it
  retains the effective last occurrence of `organization/active` and
  `organization/action-budget`, matching the current loader rather than adding
  a duplicate-attribute refusal, and sets `active_bool` only when the effective
  active value is exactly `1.0`. A missing value, `0`, or another valid integer
  is false.

  The preflight resolves at most 1,048,576 edge forms against the local-name
  table and feeds
  only exact `EdgeType/SOLIDARITY` organization-to-social-class triples to the
  pure counter, passing `PracticeTargetDomainV1::SocialClass` only after textual
  type qualification. Do not scan the completed graph. The preflight returns
  `E-LOAD-061` for contract code 41 at organization 4,097, `E-LOAD-062` for
  code 35 at one organization's qualifying edge 257, `E-LOAD-063` for code 44
  when an active organization lacks `organization/action-budget`, and
  `E-LOAD-064` for storage codes 28 through 32. `E-LOAD-065` is the loader-owned
  declaration-signature refusal and has no pure-contract error alias. No other
  contract error maps to one of these five loader identities.

  Preserve the existing `E-LOAD-044` identity for a duplicate canonical
  `(source, target, SOLIDARITY)` triple by mapping contract code 46 to that
  existing loader refusal. Add a real-loader duplicate fixture that proves the
  graph hash and allocator position remain unchanged. Contract code 45 applies
  only to an unsorted detached topology record; scenario authoring order is not
  a new loader restriction.

  Register those exact meanings in the language reference. Only after the complete preflight and `finish()` succeed may the existing loader mutate the caller's graph. Do not claim that unrelated later substrate failures are transactional.

  Generate accepted `.bscn` witnesses with fixed `0..4_096` and `0..256` loops
  and rejected witnesses with `0..=4_096` and `0..=256`. Prove 4,096/256 load
  and 4,097/257 return the exact codes through
  `load_scenario_with_prelude`. Add scenario-local `deffield` cases that omit
  or corrupt an active organization's budget and require `E-LOAD-063` or
  `E-LOAD-064`; these cases prove local declarations cannot bypass the trigger.
  Add repeated-attribute fixtures whose effective last active or budget value
  changes the result, and require parity with the existing loader's source-order
  overwrite behavior. Add an unrelated node 4,097 fixture to prove only exact
  textual `NodeType/ORGANIZATION` rows count toward `E-LOAD-061`.

  Add final-registry fixtures for missing `organization/active`, wrong
  action-budget type, extensive action budget, wrong active type, and extensive
  active state. Require exact `E-LOAD-065` from both loader entry points before
  any graph or allocator change. Include scenario-local declarations so the
  shared prelude sentinel cannot mask these cases.

  Seed identical control and subject graphs before each failing load, then add
  the same node afterward and require equal graph hashes and equal newly minted
  node IDs. This proves each topology refusal preserved graph bytes and
  allocator position. Add exact max/max-plus-one tests for source bytes, body
  forms, AST nodes, walker depth, and walker stack. Run one serialized unlocked
  `cd rust && cargo check -p babylon-bsl --tests`, inspect that `Cargo.lock`
  changes only the `babylon-bsl` workspace-package dependency entry, then use
  `--locked`.

- [ ] **Step 5: Add the six real defines and one consumer without widening public exports.** Add frozen `PracticeBudgetDefines` to `organizations.py`: `action_budget_initial`, `action_budget_weekly_credit_cap`, `action_budget_storage_ceiling`, `organize_action_budget_cost`, `agitate_action_budget_cost`, and `mutual_aid_action_budget_cost`. Use explicit Pydantic `u32` bounds, positive-cost fields, and cross-field validators for initial/ceiling/cheapest-wired cost. Wire the category through `GameDefines` and `_from_yaml_dict`; regenerate YAML. Do not re-export `PracticeBudgetDefines` from `babylon.config.defines`.

  `PracticeBudgetTermsV1.from_defines` is the sole new consumer. Do not add organize gain, agitate contribution, solidarity ceiling, decay, stock, money, or repression values.

- [ ] **Step 6: Make all pure contract tests green.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_budget.py tests/unit/contracts/test_practice_contract_v1_topology.py tests/unit/config/test_practice_budget_defines.py tests/unit/config/test_constants_sync.py tests/unit/test_public_import_surface.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test budget --test topology --locked`

- [ ] **Step 7: Add failure mutations.** Change `write_action_budget` to cast through `i32` and assert the max-value test fails. Replace footprint derivation with a caller count and assert the edge-strength/duplicate vectors fail. Count an organization-to-organization solidarity edge toward the 256 limit and assert the topology test fails. Bypass the real loader counter and assert the 4,097 scenario mutation loads when it must refuse.

  Alter one generated YAML value without schema/default change and assert the `--check` test fails. Restore all source values.

- [ ] **Step 8: Refresh the T1 fixture's dependent definitions identity.** Run
  the T1 administrative scenario and no-op rule twice through the public
  `babylon-tick` binary after the six defines land, and require byte-identical
  reports. Recompute `canonical_defines_hash(GameDefines.load_default())` and
  update only the definitions digest and any causally dependent nominal-world
  identity in `detroit_windsor_rtd_v1_world_identity.json`; the scenario, rule,
  extraction-ledger, graph-state, and verified-tick identities must stay exact.
  Run `uv run python tools/build_detroit_rtd_control.py` to propagate the new
  witness only if the sealed control references it. A changed sealed control
  must update its one shared RTD vector through the same builder, never by hand.

  Run twice: `cd rust && cargo run -p babylon-tick --locked -- ../contracts/fixtures/detroit_windsor_rtd_v1_admin_world.bscn ../contracts/fixtures/detroit_windsor_rtd_v1_admin_noop.bsl`

  Inspect the three declared dependent artifact paths and reject any change
  outside that identity closure. Then run:

  Run: `uv run python tools/build_detroit_rtd_control.py --check`

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_detroit_control.py tests/unit/contracts/test_rtd_v1_canonical.py`

  Run: `cd rust && cargo test -p babylon-rtd --test canonical_vectors --locked`

  Run: `cd rust && cargo test -p babylon-tick --test rtd_admin_fixture_identity --locked`

- [ ] **Step 9: Verify and commit.**

  Run: `uv run python tools/generate_defines_config.py --check`

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_budget.py tests/unit/contracts/test_practice_contract_v1_topology.py tests/unit/config/test_practice_budget_defines.py tests/unit/test_public_import_surface.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test budget --test topology --locked`

  Run: `cd rust && cargo test -p babylon-bsl --test practice_topology_admission --locked`

  Run: `cd rust && cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings -D clippy::pedantic`

  Run: `cd rust && cargo clippy -p babylon-bsl --test practice_topology_admission --locked -- -D warnings`

  Run: `vale docs/reference/bsl-language.rst`

  Commit: `feat(contract): add checked ActionBudget and topology groundwork`

- [ ] **Step 10: Perform the required GameDefines baseline ceremony.** After Step 9's source commit, run `mise run qa:regression` and require the only reported regression difference to be the intentional complete-`GameDefines` hash movement. Run `mise run qa:vault-regression-ci`; if it moves, require its drift table to name only pages whose serialized definitions identity changed. Any material simulation value, event, row count, outcome, checkpoint, or unrelated page movement is a fault, not a blessing.

  Run: `mise run qa:regression-generate-dense`

  If and only if the verified vault comparison reported definitions-identity-only drift, run `mise run qa:vault-regression-generate`. Inspect the exact `tests/baselines/**` diff: regression JSON may change only `defines_hash`; every dense CSV must remain byte-identical; vault manifests may change only the verified definitions-bearing page hashes. Stage only those governed baseline files. Generate the required commit body and feed it through the hook-safe commit task:

  Run: `python3 tools/generate_ceremony_message.py --slug neel-practice-budget-defines --summary "Add six Designed ActionBudget terms; simulation traces remain byte-identical and only definitions identity moves" | mise run commit -- -`

  Re-run `mise run qa:regression`, `mise run qa:vault-regression-ci`, and `mise run check:gate-coverage`; all must pass against the blessed artifacts.

### Task 5: Add the declaration-only organization practice prelude and registry sentinel

**Files:**

- Modify: `contracts/practice_contract_v1_vectors.jsonl`
- Create: `rust/crates/babylon-tick/content/declarations/organization-practice.bscn`
- Create: `rust/crates/babylon-tick/content/scenarios/organization-practice-contract.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-carrier-collision-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-cost-modifier-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-decay-arc-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-degenerate-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-empty-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-floor-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-solidarity-seam-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/community-tie-conformance.bscn`
- Modify: `rust/crates/babylon-tick/content/scenarios/consciousness-ternary-conformance.bscn`
- Create: `rust/crates/babylon-tick/tests/organization_practice_contract.rs`
- Modify: `rust/crates/babylon-tick/tests/community_conformance.rs`
- Modify: `rust/crates/babylon-tick/tests/community_arc_conformance.rs`
- Modify: `rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/codec_vectors.rs`
- Modify: `tests/unit/contracts/test_practice_contract_v1_codec.py`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`
- Modify: `rust/crates/babylon-tick/src/bin/bsl_fuel_report.rs`
- Modify: `rust/crates/babylon-tick/content/content-sets.toml`
- Modify: `rust/crates/babylon-bsl/src/scenario.rs` to add bounded declaration-prelude composition.
- Modify: `rust/crates/babylon-bsl/src/lib.rs` to re-export the composition helper.
- Modify: `rust/crates/babylon-ls/src/pass.rs` and its focused tests so diagnostics consume every ordered manifest prelude.
- Modify: `rust/crates/babylon-tick/Cargo.toml` to add a dev-dependency on `babylon-practice-contract`
- Modify: `rust/Cargo.lock`
- Modify: `tests/unit/reference/test_content_set_manifest_sync.py` only if its existing assertions need a new explicit no-rule row case.

**Interfaces:**

- Prelude declarations are exactly:

  ```scheme
  (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))
  (defenum ConsciousnessTendency (LIBERAL FASCIST REVOLUTIONARY))
  (defenum VerbMode (CANVASS AGITATE))
  (deffield organization/kind enum OrgKind)
  (deffield organization/active int intensive)
  (deffield organization/cadre-level probability intensive)
  (deffield organization/cohesion probability intensive)
  (deffield organization/consciousness-tendency enum ConsciousnessTendency)
  (deffield organization/action-budget int intensive)
  ```

- Node and edge vocabularies remain scenario-local <code>defvocabulary</code>
  declarations, whose members have textual identity and no ordinal. The
  contract fixture declares
  <code>(defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))</code>
  and the existing complete organization-foundation
  <code>EdgeType</code> member set
  <code>(MEMBERSHIP PRESENCE COMMAND TRANSACTIONAL SOLIDARISTIC
  SOLIDARITY TENANCY ADJACENCY)</code>. The reusable prelude never narrows those registries or
  treats authored member position as the external
  <code>PracticeTargetDomainV1</code> code.
- The scenario holds one active organization, one territory, one social class,
  the organization's `PRESENCE` edge, the target class's `TENANCY` edge to that
  same territory, one organization-to-class `SOLIDARITY` edge with positive
  finite `solidarity/strength`, and one `MEMBERSHIP` edge. This topology is a
  contract witness, not execution authorization. The scenario has no rule
  source, intrinsic, effect, or tick advance.

- [ ] **Step 1: Write prelude, promotion, and sentinel red tests.** Assert `load_scenario_with_prelude` loads the minimal scenario, including the target class's `TENANCY` edge to the actor's `PRESENCE` territory, and assert `VerbMode/CANVASS` ordinal is 0 and `VerbMode/AGITATE` ordinal is 1, the tendency ordinal is LIBERAL 0 / FASCIST 1 / REVOLUTIONARY 2, and the generated Rust mapping has matching mode names while external wire codes remain CANVASS 1 / AGITATE 2. Assert the exact player-facing labels are `ORGANIZE`, `AGITATE`, and `MUTUAL-AID`. Assert `organization/action-budget` is intensive and any `sum` over it returns exact `E-TYPE-041`; an unweighted mean remains `E-TYPE-042`, while min, max, and count remain legal. Assert the prelude does not declare `NodeType` or `EdgeType`, and the scenario carries the exact existing full organization-foundation edge order. Pin SHA-256 of exact raw prelude bytes through `babylon_kernel::sha256_of`; test a changed enum order, changed prelude text, duplicate line field, narrowed edge vocabulary, changed display label, an extensive ActionBudget declaration, or `MUTUAL_AID` pseudo-mode as a mismatch/refusal.

  Add a corpus assertion over the exact eleven promoted scenarios: organization-foundation, all nine community scenarios, and consciousness-ternary-conformance. None may retain a local `OrgKind`, `ConsciousnessTendency`, or any of the six promoted `organization/*` deffield declarations; every corresponding content-set row must include `declarations/organization-practice.bscn`; and each Rust consumer, the fuel-report binary, and the language-server diagnostic pass must include every manifest prelude in its declared order. This test must fail before migration, which proves the new file is shared rather than isolated. Add a two-prelude diagnostic fixture in which one declaration comes only from each source; either `.first()` or reversed composition must fail.

- [ ] **Step 2: Run the red prelude test.**

  Run: `cd rust && cargo test -p babylon-tick --test organization_practice_contract --locked`

  Expected: FAIL because the prelude, scenario, manifest row, and test target do not exist.

- [ ] **Step 3: Add the prelude and migrate every existing consumer.** Put declarations only in the prelude. Seed the new no-rule scenario's active organization with valid kind, active, cadre, cohesion, tendency, integer action budget, and positive finite solidarity strength. Remove the exact promoted duplicate declarations from `organization-foundation.bscn`, all nine enumerated community scenarios, and `consciousness-ternary-conformance.bscn`; do not remove unrelated enums/fields. Add the prelude to all eleven content-set rows.

  Add `compose_declaration_preludes(&[&str]) -> Result<String, ScenarioError>` beside the real prelude loader and re-export it. It admits at most 16 sources, at most 262,144 bytes per source, and at most 1,048,576 combined bytes; rejects CR, an empty source, or a source without exactly one terminal LF; and concatenates the admitted bytes in caller order without normalization. Every scan uses the literal fixed ceilings, and the existing single-`&str` load/tick APIs remain unchanged while receiving this one checked composite. Route all four Rust consumer modules and `bsl_fuel_report` through that helper, and change the language-server pass to preflight the number of `set.prelude` rows at 16 before its first source read. The pass then reads every admitted row in order, refuses a missing source, composes all rows with the same helper, and passes the composite to `diagnose_content_set` without selecting only the first row.

  In `organization-foundation.bscn`, seed every active organization with explicit fixture-only cadre/cohesion/tendency/action-budget values; use the designed initial budget from the contract and do not infer a gameplay line from `OrgKind`. In community fixtures, preserve every existing cadre/cohesion/tendency seed byte and add the designed initial action budget only to nodes already marked active. An organization without an active field remains outside practice eligibility and does not receive an invented active state, kind, line, or budget. The migration must not change any rule source.

- [ ] **Step 4: Add the new content-manifest row and validate all promoted rows.** Create id `organization/practice-contract` in ascending byte order. Set `scenario = "scenarios/organization-practice-contract.bscn"`, `prelude = ["declarations/organization-practice.bscn"]`, `rules = []`, and the exact test consumer. Preserve `declarations/worldview.bscn` beside the new prelude on both `community/tie` and `consciousness/ternary-conformance` in ascending byte order: organization-practice first, worldview second. Extend the existing bidirectional manifest test only as needed to prove every scenario/prelude/rule is visible and every Rust consumer includes each declared input; do not weaken containment checks. Pin helper refusal at source 17, per-source byte 262,145, combined byte 1,048,577, CR, missing terminal LF, missing language-server source, and a `.first()` mutation.

  Instrument the language-server source reader and prove a manifest with 17
  prelude paths refuses before it reads path 1 or path 17. This preflight is
  separate from the composition helper's source-17 refusal.

- [ ] **Step 5: Implement the registry mismatch sentinel.** Load the prelude
  through the real BSL declaration loader and inspect its enum registry; do not
  write a second parser. Compare it to the generated
  `PracticeIdV1::machine_verb()` result: Organize and Agitate require the same
  named `VerbMode`; Mutual Aid requires `None`. Add one exact
  `organization-practice-prelude` content-identity case to the shared vector
  corpus with the raw UTF-8 hex and its SHA-256 digest. Extend both bounded
  vector readers to hash that supplied raw byte sequence and require the
  declared digest.

  The tick sentinel hashes the real prelude file and compares
  it to this same case. The check is a declaration identity pin, not a
  `rules_hash` change or a live content-set activation.

- [ ] **Step 6: Refresh the direct dev dependency once, then make prelude and manifest tests green.** Add `babylon-practice-contract` under `babylon-tick` dev-dependencies for the generated mapping sentinel. Run one serialized unlocked `cd rust && cargo check -p babylon-tick --tests`, inspect that only the `babylon-tick` workspace-package dependency entry moves in `Cargo.lock`, then use `--locked`.

  Run: `cd rust && cargo test -p babylon-tick --test organization_practice_contract --locked`

  Run: `cd rust && cargo test -p babylon-practice-contract --test codec_vectors --locked`

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codec.py`

  Run: `mise run test:q -- tests/unit/reference/test_content_set_manifest_sync.py`

  Run: `cd rust && cargo test -p babylon-ls pass --locked`

  Run: `mise run bsl:fuel-check`

  Run: `cd rust && cargo test -p babylon-tick --test community_conformance --test community_arc_conformance --test consciousness_ternary_conformance --test tick_goldens --locked`

  Run: `cd rust && cargo test -p babylon-tick --bin bsl-fuel-report --locked`

- [ ] **Step 7: Perform the declaration/content-identity ceremony.** Run the migrated community and organization structural assertions first. Then measure every intentionally changed pre/post graph and nominal-world hash by executing the real tests; update only the literal pins whose scenario state gained promoted declarations/seeded ActionBudget. Record the raw prelude digest, before/after pin table, and why each moved in the commit body. An unrelated golden, firing count, per-rule count, event, or material field change is a fault.

  Run: `mise run check:bsl-sentinels`

  Run: `cd rust && cargo fmt --all -- --check`

  Run: `cd rust && cargo clippy -p babylon-tick --test organization_practice_contract --test community_conformance --test community_arc_conformance --test consciousness_ternary_conformance --test tick_goldens --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-tick --bin bsl-fuel-report --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-ls --all-targets --locked -- -D warnings -D clippy::pedantic`

- [ ] **Step 8: Add mutations and commit.** Change `VerbMode` order and assert ordinal/mapping tests fail. Reinsert one duplicate `organization/consciousness-tendency` declaration and require the real loader to refuse. Remove one manifest prelude path and assert the bidirectional content-set test fails. Restore all bytes, then commit:

  `test(tick): pin organization practice declaration prelude`

### Task 6: Add explicit non-live dependency lookups and freeze the groundwork boundary

**Files:**

- Modify: `src/babylon/contracts/practice_contract_v1.py`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`
- Create: `tests/unit/contracts/test_practice_contract_v1_refusals.py`
- Create: `rust/crates/babylon-practice-contract/tests/refusals.rs`
- Modify: `ai/decisions/ADR227_practice_contract_groundwork.yaml` only if review finds an unrecorded boundary.

**Interfaces:**

```python
def unwired_reason(practice: PracticeIdV1) -> PracticeRejectionCodeV1: ...
def activation_blockers(practice: PracticeIdV1) -> tuple[PracticeActivationBlockerV1, ...]: ...
```

```rust
pub const fn unwired_reason(practice: PracticeIdV1) -> PracticeRejectionCodeV1;
pub const fn activation_blockers(
    practice: PracticeIdV1,
) -> &'static [PracticeActivationBlockerV1];
```

`PracticeActivationBlockerV1` is generated from the Task 1 schema as a closed metadata enum, not a wire field. Organize and Agitate list `GATE3_COMMITTED_ENVELOPE` and `GATE5_PENDING_INPUT`. Mutual Aid additionally lists `PER30_ORDERS_INVENTORY` and `PER31_FREIGHT_REALIZATION`. PER-44 blocks the separate line-return leg, not T2a Organize admission; PER-36 applies only if a later aid route uses money; the shared repression resolver is not a practice-id blocker. Those three secondary boundaries remain ADR227 facts rather than being falsely attached to every rejection. Neither function accepts an actor, target, controller, graph, ledger, committed tick, content digest, or database handle, so neither can construct the exact `PracticeSubmissionRejectionV1`; the future admission owner supplies that context to Task 3's `rejection_for`.

- [ ] **Step 1: Write refusal red tests.** Assert all three practice ids return `PRACTICE_UNWIRED`; Organize and Agitate have exactly the two shared blockers; Mutual Aid has exactly those plus PER-30 and PER-31, in declared byte order. Assert no lookup can debit a detached budget term, mutate a topology value, encode a receipt, create a pending row, expose an outbox payload, or call `TickSession::advance`. Assert Mutual Aid does not name ActionBudget, Capacity, wealth, rent pool, or money as goods, and does not make PER-36 unconditional.

- [ ] **Step 2: Run the red refusal tests.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_refusals.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test refusals --locked`

  Expected: FAIL because the explicit dependency lookup does not exist.

- [ ] **Step 3: Implement immutable dependency lookup.** Return the generated stable reason and static blocker slices through exhaustive practice-id matches. It must use no stateful dependency and must not call the rejection, intent, or budget codecs. Keep malformed-id rejection in codec validation; the lookup accepts only a valid closed practice.

- [ ] **Step 4: Make dependency lookups green.**

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_refusals.py`

  Run: `cd rust && cargo test -p babylon-practice-contract --test refusals --locked`

- [ ] **Step 5: Run the complete focused verification serially.**

  Run: `uv run python tools/generate_practice_contract_types.py --check`

  Run: `uv run python tools/generate_defines_config.py --check`

  Run: `mise run test:q -- tests/unit/contracts/test_practice_contract_v1_codegen.py tests/unit/contracts/test_practice_contract_v1_codec.py tests/unit/contracts/test_practice_contract_v1_admission.py tests/unit/contracts/test_practice_contract_v1_budget.py tests/unit/contracts/test_practice_contract_v1_topology.py tests/unit/contracts/test_practice_contract_v1_refusals.py tests/unit/config/test_practice_budget_defines.py tests/unit/reference/test_content_set_manifest_sync.py tests/unit/test_public_import_surface.py`

  Run: `cd rust && cargo fmt --all -- --check`

  Run: `cd rust && cargo test -p babylon-practice-contract --locked`

  Run: `cd rust && cargo test -p babylon-bsl --test practice_topology_admission --locked`

  Run: `cd rust && cargo test -p babylon-tick --test organization_practice_contract --test community_conformance --test community_arc_conformance --test consciousness_ternary_conformance --test tick_goldens --locked`

  Run: `cd rust && cargo test -p babylon-tick --bin bsl-fuel-report --locked`

  Run: `cd rust && cargo test -p babylon-ls pass --locked`

  Run: `mise run bsl:fuel-check`

  Run: `cd rust && cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings -D clippy::pedantic`

  Run: `cd rust && cargo clippy -p babylon-bsl --test practice_topology_admission --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-tick --test organization_practice_contract --test community_conformance --test community_arc_conformance --test consciousness_ternary_conformance --test tick_goldens --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-tick --bin bsl-fuel-report --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-ls --all-targets --locked -- -D warnings -D clippy::pedantic`

  Run: `mise run check`

  Run: `PYTHONPATH="$PWD/src" mise run check:vocabulary`

  Run: `mise run check:bsl-sentinels`

  Run: `mise run qa:regression`

  Run: `mise run qa:vault-regression-ci`

  Run: `mise run check:gate-coverage`

  Run: `vale ai/decisions/ADR227_practice_contract_groundwork.yaml docs/reference/bsl-language.rst`

- [ ] **Step 6: Perform the boundary review and commit.** Confirm no changed source mentions `advance_with_inputs`, pending ledger implementation, durable input, outbox construction, BSL `:role intent` rule, `GOVERNED_EFFECT_ALLOWANCES` intent row, GraphSubstrate mutation, receipt creation, player gateway, policy producer, membership producer, inventory, aid delivery, repression resolver, or Backfire writer. Confirm the only newly live-shaped fields are declarations and detached contract values. Commit:

  `test(contract): pin T2 groundwork refusal boundary`

- [ ] **Step 7: Publish the PER-55 completion handoff.** Resolve the final T2
  branch SHA with `git rev-parse HEAD`, require the declared T2 surface to be
  clean, and post one PER-55 completion comment with every T2 implementation
  commit SHA, the final branch SHA, all final-gate results, and the explicit
  Gate 3/Gate 5 activation exclusions. Only after that evidence exists, move
  PER-55 from In Progress to Done, refresh it, and require the returned state to
  be Done before T3 starts. Keep PER-56 blocked by PER-27 and this now-complete
  groundwork; do not mark activation complete.

## Scope Cuts and Dependencies

- **PER-20 / PER-22:** this crate has no database/writer/envelope/outbox implementation. Gate 3 owns committed persistence, subject identity, Archive output, and fog-safe evidence.
- **PER-26 / PER-27:** this plan creates no acceptance record and no input-to-tick bridge; Gate 5 owns next-week admission, sealed ledger, `advance_with_inputs`, player gateway, and the first real costed intervention. Its admission adapter also owns textual target-domain resolution through `LoadedScenario.vocabulary` and `GraphSubstrate::node_type_of`; T2 groundwork only carries the stable contract-local domain code.
- **BSL Intent authority:** ADR224 remains default-deny. No `RuleRole::Intent` allowance or live BSL practice effect is added here.
- **T2a effects:** Organize/Agitate mechanics, eligibility, collect/apply reduction, action budget graph write, pre-action snapshot, receipts, and decay need the Gate 3/Gate 5 bridge and are excluded.
- **T2b line path:** contract fields describe the existing line but create no membership. The encounter-to-membership producer remains PER-44-dependent.
- **Mutual Aid:** no organization-owned goods, labor, routing, reserve/debit, delivery, loss, or recipient consumer exists. PER-30/PER-31 own stock-only activation. PER-36 applies only when a route introduces money, credit, or escrow.
- **Antagonism:** no repression-facing resolver, Dossier/exposure reader, Backfire consumer, antagonist API, or privileged controller path exists.
- **Content identity:** declaration promotion intentionally changes the eleven enumerated scenario/prelude compositions and their measured graph/world hashes, but no rule source or effect changes. Task 5 records and re-pins only that exact identity movement; it cannot bless a firing-count, event, or material-rule drift.

## Self-Review

### Spec coverage

| Requirement | Plan coverage |
|---|---|
| Ruled display-to-machine mapping and closed modes | Tasks 1–3 and Task 5 sentinel |
| One organization line declaration, not a replacement field | Task 5 |
| Bounded authority, intent, delta, and rejection wire contracts | Tasks 1–3 |
| Detached authority-pair, quote-context, and 4,096-intent batch laws | Task 3 |
| Checked-u32 ActionBudget math and exact binary64 boundary | Task 4 |
| Solidarity-derived replenishment witness, not caller count | Task 4 |
| Real pre-BSL scenario/topology ceilings and vectors | Task 4 and Task 5 fixture |
| Generated GameDefines parity with a real detached consumer | Task 4 |
| Canonical codec vectors in independent Python/Rust implementations | Task 3 |
| Explicit no-activation reason and blocker lookup | Task 6 |
| ADR/index record for architectural placement | Task 1 |
| Shared declaration promotion across all existing consumers | Task 5 |
| PER-52 → PER-55 and PER-20 → 22 → 26 → 27 preservation | Linear Preflight, Global Constraints, Task 6, Scope Cuts |

### Plan-completeness scan

The plan gives exact file ownership, interfaces, test inputs, red/green commands, mutation witnesses, and commits for each landing. It identifies external dependencies by their governing issue rather than treating them as silent work in this contract slice.

### Type consistency

- `PracticeIdV1`, `VerbStemV1`, `VerbModeV1`, `PracticeRejectionCodeV1`, `PracticeInputAuthorityV1`, `PracticeIntentV1`, `OrganizationBudgetDeltaV1`, and `PracticeSubmissionRejectionV1` originate in Task 2 and keep those names in Tasks 3–6.
- Task 1's `PracticeContractError` registry is generated in Task 2 and reused by
  codec, admission, budget, and topology tests.
- Task 4’s `PracticeBudgetTermsV1.from_defines(GameDefines())` is the only generated-defines consumer and uses Task 1’s six declared terms.
- Task 5’s registry sentinel consumes Task 2’s `machine_verb()` mapping and Task 3’s shared vector corpus without giving BSL an execution path.
- Task 6's dependency API accepts a valid `PracticeIdV1` only and does not overload malformed codec errors or fabricate a context-free rejection.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-23-neel-t2-practice-contract-groundwork.md`. The Director already authorized autonomous execution. Use `superpowers:subagent-driven-development`, review every task before the next commit, preserve the Linear preflight, and keep Cargo gates single-flight.
