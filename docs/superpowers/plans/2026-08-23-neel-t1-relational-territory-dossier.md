<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- The mandated plan template, literal APIs, command lines, and contract vocabulary
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

# Relational Territory Dossier V1 (T1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a language-neutral, bounded `RelationalTerritoryDossierV1` contract with independent Rust and Python canonical encoders and an unfogged Detroit administrative control fixture that represents unavailable MSA, H3, and provenance-conflicted Census material as typed gaps.

**Architecture:** The checked-in YAML contract is the sole declaration of V1’s closed records, enums, required fields, identity forms, sorting rules, and limits. A deterministic generator renders sealed Rust and Python structural types from that contract; hand-written validators and encoders in the two languages remain independent of one another and consume the same vector corpus. The fixture is a static administrative proof artifact, not graph state, an Archive record, player knowledge, a live data producer, or a Bevy surface.

**Tech Stack:** YAML/PyYAML and Pydantic 2 (Python); Rust 2021 (`serde`, `serde_json`, `unicode-normalization`, and `babylon-kernel::sha256_of`); PyArrow for the bounded source-verification tool; checked-in YAML, JSON, and JSONL vectors; scoped `pytest` and Cargo tests.

**Spec:** `docs/superpowers/specs/2026-08-23-neel-relational-territory-practice-design.md`, §§7.1–7.7, §10, §11.

## Global Constraints

- T1 is an **unfogged administrative** projection only. It cannot satisfy a gameplay, player-agency, emergence, committed-Archive, or Bevy milestone.
- Execution belongs to PER-52. Before Task 0, refresh Linear and require PER-51 `Done`; then set PER-52 `In Progress`. PER-53 remains the sole H3-activation owner and stays blocked by both PER-52 and PER-21. If those facts differ, stop this plan before a source change and reconcile Linear rather than bypassing the dependency.
- V1 is a closed schema: unknown fields, enum members, references, versions, and duplicate canonical keys fail before encoding or hashing.
- Every identity and every `*_id` outside `TypedIdentityV1` is a full `{domain, authority, local_id}` typed identity; plain local strings never substitute for an identity.
- The only digest algorithm is SHA-256. Hash input is `ASCII("babylon.relational-territory-dossier.v1") || 0x00 || canonical_json(draft)`.
- Canonical JSON is UTF-8, no whitespace, sorted object keys by UTF-8 bytes, explicit nulls, lowercase JSON literals, shortest required escapes, NFC-only strings, no unpaired surrogate, and no control character outside JSON’s required escapes.
- All encoding/validation bounds are exact: 64 focus, 4,096 reference digests, 65,535 top-level relation/provenance records, 32 facet coordinates, 1,024 hyperedge members, 256 payload facets, 256 display references per decision-surface list, 8,192 provenance references per record, 256 UTF-8 bytes per identity component, 1,024 bytes per provenance locator, and 67,108,864 canonical bytes.
- Contract loading is also bounded: 262,144 raw YAML bytes, 65,536 YAML events, nesting depth 16, 32 record declarations, 64 enum declarations, 64 fields per record, 256 members per enum, and 512 limit/registry entries. Aliases and duplicate mapping keys refuse. Vector loading is bounded to 1,048,576 file bytes, 256 lines, 262,144 bytes per line, 128 UTF-8 bytes per case id, and JSON nesting depth 32.
- Every loop uses a named compile-time constant as its syntactically visible upper bound. The absolute container ceiling is `RTD_MAX_COLLECTION_ITEMS = 65_535`; smaller collections use their own literal generated constants. JSON/YAML scanners use fixed indexed ranges over `RTD_MAX_JSON_INPUT_BYTES + 1` and `RTD_MAX_YAML_EVENTS + 1`. No public or private traversal accepts a caller-supplied integer limit, and no iterable is materialized before the applicable closed count preflight.
- A bounded builder selects a closed collection kind, checks its generated constant before append, and never accepts a raw `limit`. An encoder stages privately, uses checked `u64` length accounting, and publishes no partial bytes or hash after any error.
- `PRESENT` alone carries `value_bits_or_null`; `ABSENT`, `UNKNOWN`, `NOT_COMPUTED`, and `REDACTED` require it to be null. Measured zero is `PRESENT`, never a missingness substitute.
- Top-level relation arrays, focus, coordinates, reference digests, member references, payload facets, and provenance references are canonical sets. The four `DecisionSurfaceV1` lists retain declared display order.
- Scale memberships, facets, dyads, hyperedges, and reference flows remain distinct record families. A flow is not a graph edge; an external endpoint is not a geographic parent; an n-member hyperedge stays n-ary.
- Before PER-21 is accepted for RTD consumption, the control emits no H3 `TypedIdentityV1`, facet, membership, or canonical vector. Each requested H3 measure is one `GapV1` with `status: NOT_COMPUTED` and `required_producer_or_null: PER-21`.
- The fixture emits one OMB-MSA `GapV1` with `reason_code: MISSING_GOVERNED_OMB_DELINEATION`, no MSA `ScaleMembershipV1`, and no legacy `19820` identity.
- The pinned Census fact rows carry 2023 time coordinates but source 2, while the pinned source dimension identifies source 2 as ACS 2010 and source 4 as ACS 2023. The control emits no Census facet from those rows. It emits exactly three `UNKNOWN` gaps with `reason_code: PROVENANCE_COORDINATE_CONFLICT` and `required_producer_or_null: PER-28`; the extraction ledger preserves the conflicting fact and dimension rows as audit evidence.
- The canonical Detroit control has no Canadian LODES commute flow, no Canadian H3/county/metro/jurisdiction identity, no weighted-overlap claim, no territorial score/rank/stage/radial label, and no response curve. A separately named opt-in vector adds exactly one `BORDER_SYNTHESIS` reference flow from the existing Detroit Census-place endpoint to an `EXTERNAL canada` endpoint and its one provenance row; a closed default-builder case-ID allowlist excludes it without adding a non-schema field. It never mutates canonical LODES or mints Windsor geography.
- Do not create a Rust writer, database schema, Archive outbox, fog policy, player input, client surface, live data loader, MSA producer, H3 producer, enabled Canadian synthesis, weighted-overlap producer, or any game-mechanic effect in this plan. The bounded fixture builder and source verifier are test-artifact tools only. Runtime ownership remains with PER-20/PER-21/PER-22/PER-23/PER-24/PER-28.
- Python remains the sole live writer until the authorized cutover. This plan produces checked-in contract artifacts only.
- Use TDD in every task: record the named red failure, make the smallest change green, then refactor only after the scoped tests pass. Do not run `mise run rust:check`; run the listed Cargo legs serially.
- Run `vale docs/superpowers/plans/2026-08-23-neel-t1-relational-territory-dossier.md` after editing this plan and run targeted Vale on any new Markdown prose created during execution.

## Linear Ownership Preflight

| Issue | Required state at execution | Exact ownership |
|---|---|---|
| PER-51 | `Done` | T0 canonical theory and source-exclusion prerequisite. It blocks PER-52. |
| PER-52 | Set to `In Progress` before Task 0 | This plan: freeze the V1 RTD contract, independent encoders, and Detroit administrative control. |
| PER-53 | `Todo`, blocked by PER-52 and PER-21 | Consume the shared H3 identity contract and activate H3-backed RTD facets. This plan cannot emit an H3 identity or vector. |
| PER-21 | Refresh, but do not absorb | Shared PostgreSQL/H3 identity and overlap prerequisites owned outside T1. |

Record the refreshed issue states in the PER-52 implementation comment. Do not use this dated plan snapshot as a substitute for the live preflight.

## File Structure

| Path | Responsibility |
|---|---|
| `contracts/relational_territory_dossier_v1.yaml` | The language-neutral closed V1 schema: records, enums, error registry, complete typed-identity, metric, and relation-binding registries, JSON forms, canonical set keys, exact bounds, and the hash domain separator. |
| `contracts/relational_territory_dossier_v1_vectors.jsonl` | Shared valid/invalid vector corpus. Each line carries a stable case id, case kind, draft or malformed payload, expected canonical UTF-8 hex/hash, or stable error code. |
| `contracts/fixtures/detroit_windsor_rtd_v1_admin_control.json` | The sealed, no-H3 administrative fixture produced from the shared schema and vectors. |
| `contracts/fixtures/detroit_windsor_rtd_v1_extraction.yaml` | Closed extraction ledger with artifact digests, row coordinates, exact scalar bit strings, provenance locators, and the mandatory gap registry. It is fixture input, not a runtime loader declaration. |
| `contracts/fixtures/detroit_windsor_rtd_v1_admin_world.bscn` | Minimal three-county administrative graph scenario used only to verify the fixture's real tick/world identity. |
| `contracts/fixtures/detroit_windsor_rtd_v1_world_identity.json` | Checked result of the Rust tick identity witness: scenario digest, definitions digest, verified tick, graph-state hash, and nominal-world hash. No null or zero placeholder is legal. |
| `tools/generate_rtd_v1_types.py` | Deterministic checked generator from the YAML contract to the two structural type outputs; supports `--check` without writing. |
| `tools/build_detroit_rtd_control.py` | Bounded fixture builder plus optional pinned-source verifier. It reads only the extraction ledger and explicitly named Parquet/CSV artifacts. |
| `tools/check_repo_hygiene.py` | Adds the governed root `contracts` directory to the exact repository-root allowlist. |
| `src/babylon/contracts/rtd_v1_generated.py` | Generated frozen Pydantic models, closed Python enums, error/metric registries, and contract constants. No encoder or producer policy belongs here. |
| `src/babylon/contracts/relational_territory_dossier_v1.py` | Python parser, bounded validator, canonical encoder, projection-hash sealer, and explicit error taxonomy. |
| `rust/Cargo.toml` | Registers the isolated `babylon-rtd` crate in the workspace. |
| `rust/Cargo.lock` | Records the new workspace package and the already-declared `babylon-tick` test dependencies after one serialized unlocked refresh; every later Cargo command is locked. |
| `rust/crates/babylon-rtd/Cargo.toml` | Declares the RTD contract crate and only its serialization, kernel-hash, and Unicode-normalization dependencies. |
| `rust/crates/babylon-rtd/src/generated.rs` | Generated closed Rust enums, records, and constants. |
| `rust/crates/babylon-rtd/src/lib.rs` | Narrow public API and stable `RtdError` re-exports. |
| `rust/crates/babylon-rtd/src/validate.rs` | Rust bounded semantic validation with explicit error variants. |
| `rust/crates/babylon-rtd/src/canonical.rs` | Rust-only canonical JSON writer and projection-hash sealer. It must not call Python or shell out. |
| `tests/unit/contracts/test_rtd_v1_codegen.py` | Generator freshness, closed-schema, and contract-shape tests. |
| `tests/unit/contracts/test_rtd_v1_validation.py` | Python validation and refusal boundary tests. |
| `tests/unit/contracts/test_rtd_v1_canonical.py` | Python vector-corpus and canonical-byte/hash tests. |
| `tests/unit/contracts/test_rtd_v1_detroit_control.py` | Administrative-control assertions, including typed MSA/H3 gaps and forbidden-output checks. |
| `tests/unit/tools/test_repo_hygiene.py` | Pins `contracts` as the sole new governed root and proves an unrelated root still refuses. |
| `rust/crates/babylon-rtd/tests/generated_contract.rs` | Rust generated-type and contract-constant conformance tests. |
| `rust/crates/babylon-rtd/tests/validation.rs` | Rust semantic-refusal tests. |
| `rust/crates/babylon-rtd/tests/canonical_vectors.rs` | Rust consumption of the same root JSONL vectors and fixture bytes. |
| `rust/crates/babylon-tick/tests/rtd_admin_fixture_identity.rs` | Executes the minimal scenario and compares real tick, graph, and world identities against the checked world-identity witness. It does not activate RTD in the tick. |
| `ai/decisions/ADR228_relational_territory_dossier_contract.yaml` | Architecture record for the YAML source of truth, generated structural types, independent encoders, isolated crate, and administrative-only boundary. |
| `ai/decisions/index.yaml` | Registers ADR228 under its exact stem. |

---

### Task 0: Record the RTD contract and crate boundary

**Files:**

- Create: `ai/decisions/ADR228_relational_territory_dossier_contract.yaml`
- Modify: `ai/decisions/index.yaml`
- Modify: `tools/check_repo_hygiene.py`
- Modify: `tests/unit/tools/test_repo_hygiene.py`

**Interfaces:**

- Records: the checked-in YAML contract as the only V1 structural source of truth; generated Rust/Python record declarations; independently hand-written validators and canonical encoders; the one-way `babylon-rtd -> babylon-kernel` dependency; and the prohibition on engine, persistence-writer, Archive, fog, or player-surface authority in T1.
- Rejects: hand-maintained duplicate record schemas, an engine dependency on `babylon-rtd`, a generic schema framework, self-comparison between encoders, reuse of an administrative fixture as player evidence, or an ungoverned repository-root exception.

- [ ] **Step 1: Write repository-hygiene red tests.** Assert that a tracked root named `contracts` is accepted only when it is present in the exact `ALLOWED_TOP_LEVEL_DIRS` declaration, while `contractz` and another unknown root still fail. Run the test before modifying the checker.

  Run: `mise run test:q -- tests/unit/tools/test_repo_hygiene.py`

  Expected: FAIL because `contracts` is absent from the current exact root allowlist.

- [ ] **Step 2: Govern the new root surgically.** Add only `"contracts"` to `ALLOWED_TOP_LEVEL_DIRS` in `tools/check_repo_hygiene.py`; do not add a prefix, glob, or generic exemption. Make the behavioral test green.

  Run: `mise run test:q -- tests/unit/tools/test_repo_hygiene.py`

  Expected: PASS while the two unknown-root witnesses still refuse.

- [ ] **Step 3: Create the ADR file without its index row and record the namespace red state.** Use top-level key `ADR228_relational_territory_dossier_contract` and the repository's modern record fields (`status`, `date`, `context`, `decision`, `consequences`, and `verification`).

  Run: `mise run check:bsl-sentinels`

  Expected: FAIL because ADR228 exists but `ai/decisions/index.yaml` has no matching stem.

- [ ] **Step 4: Register ADR228 in `ai/decisions/index.yaml`.** The index key and `file` value must match the exact ADR stem and filename. The title must state that this is a language-neutral administrative projection contract, not a live game or persistence boundary.

- [ ] **Step 5: Make the architecture record green.**

  Run: `mise run check:bsl-sentinels`

  Run: `uv run yamllint -c .yamllint.yaml ai/decisions/ADR228_relational_territory_dossier_contract.yaml ai/decisions/index.yaml`

  Run: `mise run check:hygiene`

  Expected: PASS with exact ADR-number/index synchronization.

- [ ] **Step 6: Commit the architecture decision and governed root before implementation.**

  Commit: `docs(architecture): record RTD contract boundary`

### Task 1: Establish the closed YAML contract and deterministic type generator

**Files:**

- Create: `contracts/relational_territory_dossier_v1.yaml`
- Create: `tools/generate_rtd_v1_types.py`
- Create: `tests/unit/contracts/test_rtd_v1_codegen.py`
- Create: `src/babylon/contracts/rtd_v1_generated.py`
- Create: `rust/crates/babylon-rtd/src/generated.rs`

**Interfaces:**

- Consumes: the T1 record table and limits in the governing design spec.
- Produces: `load_contract(path: Path) -> RtdContractSpec`; `render_python(spec: RtdContractSpec) -> str`; `render_rust(spec: RtdContractSpec) -> str`; generated `RtdDossierDraftV1`, `RelationalTerritoryDossierV1`, all eleven nested V1 records, all closed enums, `RTD_V1_LIMITS`, `RTD_V1_ERROR_REGISTRY`, `RTD_V1_IDENTITY_REGISTRY`, `RTD_V1_METRIC_REGISTRY`, `RTD_V1_RELATION_BINDING_REGISTRY`, and `RTD_V1_SCHEMA_ID` in both languages.
- Generator CLI: `uv run python tools/generate_rtd_v1_types.py [--check] [--contract PATH] [--python-out PATH] [--rust-out PATH]`. Defaults name the checked-in contract and outputs. `--check` compares exact UTF-8 generated bytes and exits nonzero without writing.

- [ ] **Step 1: Write the generator red tests.** In `test_rtd_v1_codegen.py`, define these assertions before creating generated output:

  ```python
  def test_contract_declares_every_sealed_record() -> None:
      contract = load_contract(CONTRACT_PATH)
      assert contract.records == {
          "TypedIdentityV1", "ReferenceDigestV1", "DimensionCoordinateV1",
          "ScaleMembershipV1", "FacetV1", "DyadV1", "HyperedgeV1",
          "ReferenceFlowV1", "GapV1", "ProvenanceV1", "DecisionSurfaceV1",
          "RtdDossierDraftV1", "RelationalTerritoryDossierV1",
      }

  def test_generated_files_are_current() -> None:
      assert run_generator_check() == 0
  ```

  Add assertions that `schema == "babylon.relational-territory-dossier"`, `schema_version == 1`, every fixed limit equals the Global Constraints value, every enum is closed, all 20 error rows are exact and ordered, every typed-identity registry row has the exact three components declared below, all 18 expanded metric rows equal the complete table below byte for byte, all six relation-binding rows are exact, and both generated sources contain `projection_hash` only on the sealed dossier type. Assert each generated metric row stores complete `TypedIdentityV1` values for its metric, unit, native scale, coordinates, producer, and optional reference artifact; no generated metric row stores a symbolic registry key.

- [ ] **Step 2: Run the focused generator tests and record the red state.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py`

  Expected: FAIL because the YAML contract and generator module do not exist.

- [ ] **Step 3: Write the contract as exact data, not prose.** Freeze the complete enum vocabulary below. These literals are the JSON strings and generated discriminants; no alias or unknown value is accepted.

  | Enum | Exact members in declaration order |
  |---|---|
  | `AudienceV1` | `ADMIN_MATERIAL`, `PLAYER_KNOWLEDGE` |
  | `DurabilityV1` | `IN_MEMORY`, `COMMITTED` |
  | `EvidenceClassV1` | `Observed`, `Derived`, `Calibrated`, `Designed` |
  | `StatusV1` | `PRESENT`, `ABSENT`, `UNKNOWN`, `NOT_COMPUTED`, `REDACTED` |
  | `ValueKindV1` | `UINT64_BITS`, `FLOAT64_BITS` |
  | `CoverageV1` | `COMPLETE`, `PARTIAL`, `NOT_APPLICABLE`, `UNKNOWN` |
  | `MembershipKindV1` | `ADMINISTRATIVE`, `NATIONAL`, `COMMUTING_ZONE`, `METROPOLITAN`, `WEIGHTED_OVERLAP` |
  | `FacetFamilyV1` | `COMMAND_ADMINISTRATION`, `PRODUCTION_CIRCULATION`, `REPRODUCTION_SETTLEMENT_ACCESS`, `EXTRACTION_ABANDONMENT_CARCERAL`, `ECOLOGY_CARE`, `ORGANIZATION_ROOTEDNESS` |
  | `DyadKindV1` | `PRESENCE`, `MEMBERSHIP`, `SOLIDARITY`, `COMMAND` |
  | `HyperedgeKindV1` | `PUBLIC_RELATION` |
  | `FlowKindV1` | `COMMUTER_JOBS`, `BORDER_SYNTHESIS` |
  | `RelationPayloadModeV1` | `EMPTY`, `SINGLE_METRIC_FACET`, `IMPLICIT_RELATION` |
  | `GapReasonV1` | `MISSING_GOVERNED_OMB_DELINEATION`, `IDENTITY_CONTRACT_PENDING`, `MISSING_GOVERNED_PRODUCER`, `REFERENCE_COVERAGE_UNAVAILABLE`, `PLAYER_BOUNDARY_UNAVAILABLE`, `PROVENANCE_COORDINATE_CONFLICT` |
  | `MetricRepresentationV1` | `FACET`, `REFERENCE_FLOW`, `DYAD` |
  | `AggregationRuleV1` | `NONE`, `PUBLISHED_ROLLUP`, `LOAD_TIME_SUM`, `BLOCK_INTERNAL_POINT_ASSIGNMENT`, `BLOCK_COORDINATE_ASSIGNMENT`, `EQUAL_AREA_WATER_INTERSECTION`, `TYPED_RELATION_PROJECTION` |
  | `RtdCollectionKindV1` | `FOCUS`, `REFERENCE_DIGESTS`, `SCALE_MEMBERSHIPS`, `FACETS`, `DYADS`, `HYPEREDGES`, `FLOWS`, `GAPS`, `PROVENANCE`, `COORDINATES`, `MEMBER_REFS`, `PAYLOAD_FACETS`, `DISPLAY_REFS`, `PROVENANCE_REFS` |

  Freeze `projection_version` as JSON unsigned integer `u16` and `verified_tick` as JSON unsigned integer `u64`. Freeze every digest/hash field as exactly 64 lowercase hexadecimal characters. Freeze `value_bits_or_null` and `weight_bits_or_null` as null or exactly 16 lowercase hexadecimal characters: `UINT64_BITS` means the unsigned 64-bit value's big-endian bit pattern; `FLOAT64_BITS` means finite IEEE-754 binary64 bits with negative zero normalized to positive zero before validation and encoding. A status other than `PRESENT` requires null bits. A present weight is always `FLOAT64_BITS` and finite in `[0, 1]`.

  Declare the thirteen records and their exact spec field order, scalar/container type, nullability, and bound. Every `*_ref`, every field outside `TypedIdentityV1` ending in `*_id`, and `actor` is `TypedIdentityV1`; `family`, kind, coverage, status, value kind, evidence class, audience, and durability use only the enums above. `required_producer_or_null` is null or an NFC string of 1 through 64 UTF-8 bytes matching `PER-[1-9][0-9]*`. `vintage` is NFC, 1 through 256 UTF-8 bytes. Include explicit `draft_record: RtdDossierDraftV1` with all top-level spec fields except `projection_hash`, and `sealed_record: RelationalTerritoryDossierV1` with the same fields plus mandatory `projection_hash`.

  Freeze canonical set keys explicitly: top-level identity lists by their complete identity bytes; `coordinates` by `dimension_ref`; `reference_digests` by `reference_id`; every top-level record family by its record identity; `member_refs`, `payload_facets`, and `provenance_refs` by referenced identity. A repeated sort key is `RTD_DUPLICATE_KEY`. The four `DecisionSurfaceV1` lists alone retain input order. Canonical JSON object keys always sort by UTF-8 bytes, independently of generated declaration order. Record/field declaration order controls generated source readability only; it never changes wire bytes.

  The YAML also contains a closed `identity_registry`. It expands every symbolic
  key in the metric table to one complete `TypedIdentityV1`; generated code and
  validators never compare the symbolic key itself:

  - every exact metric literal below expands to `{domain: "metric", authority:
    "babylon.rtd.v1", local_id: <the complete metric literal>}`;
  - unit keys expand under domain `unit`, authority `babylon.rtd.v1`: `JOBS` →
    `jobs`, `ESTABLISHMENTS` → `establishments`, `USD_CURRENT` → `usd-current`,
    `HOUSEHOLDS` → `households`, `PERSONS` → `persons`, `FACILITIES` →
    `facilities`, `FRACTION` → `fraction`, and `TYPED_RELATION` →
    `typed-relation`;
  - coordinate keys expand under domain `dimension`, authority
    `babylon.rtd.v1`: `county`, `naics6`, `ownership`, `home_county` →
    `home-county`, `work_county` → `work-county`, `source`, `tenure`, `race`,
    `burden`, `h3_cell` → `h3-cell`, `coercive_type` → `coercive-type`, `actor`,
    and `node`;
  - native-scale keys expand under domain `native-scale`, authority
    `babylon.rtd.v1`, with these local ids in declaration order:
    `county-naics6-ownership-year`, `county-ownership-year`,
    `home-county-work-county-year`, `county-source-tenure-time-race`,
    `county-source-time-race`, `county-source-burden-time-race`,
    `h3-r7-vintage`, `county-coercive-type-source`, and
    `actor-node-verified-tick`;
  - producer keys for every exact artifact basename below expand under domain
    `producer`, authority `babylon.data.v7`, and the unchanged basename as
    `local_id`; `committed typed graph` alone expands to `{domain: "producer",
    authority: "babylon.engine", local_id:
    "typed-graph-relations-at-verified-tick"}`;
  - each non-null reference key uses the same artifact basename but expands
    under domain `reference-artifact`, authority `babylon.data.v7`; its digest
    remains the separate exact 32-byte reference-digest field.

  A registry row that omits or duplicates a key, changes any component, maps
  two keys to the same identity, or refers to an unknown key refuses before
  code generation. The YAML's closed `metric_registry` then contains one row
  per metric. Each row declares identity-registry keys for the metric, unit,
  native scale, every coordinate dimension, producer, and optional reference;
  it also declares representation, value kind or null, allowed evidence
  classes, and aggregation rule. The grouped rows below expand to exactly 18
  rows; no implementation may add a private registry:

  | Metric identity keys | Representation / unit identity key / value | Native-scale identity key; coordinate identity keys (temporal value stays in `vintage`) | Evidence / aggregation / producer identity key / reference identity key / digest |
  |---|---|---|---|
  | `production/qcew-leaf-employment` | `FACET` / `JOBS` / `UINT64_BITS` | `COUNTY_NAICS6_OWNERSHIP_YEAR`; `county,naics6,ownership` | `Observed,Derived` / `NONE` / `fact_qcew_annual` / `fact_qcew_annual` / `ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248` |
  | `production/qcew-leaf-establishments` | `FACET` / `ESTABLISHMENTS` / `UINT64_BITS` | `COUNTY_NAICS6_OWNERSHIP_YEAR`; `county,naics6,ownership` | `Observed,Derived` / `NONE` / `fact_qcew_annual` / `fact_qcew_annual` / `ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248` |
  | `production/qcew-leaf-total-wages-usd`, `production/qcew-leaf-average-annual-pay-usd` | `FACET` / `USD_CURRENT` / `FLOAT64_BITS` | `COUNTY_NAICS6_OWNERSHIP_YEAR`; `county,naics6,ownership` | `Observed,Derived` / `NONE` / `fact_qcew_annual` / `fact_qcew_annual` / `ca3825a3d60831479313632073b7fc9a941d57dcf9b8940181c4713b6d442248`; expand to two rows |
  | `production/qcew-county-employment` | `FACET` / `JOBS` / `UINT64_BITS` | `COUNTY_OWNERSHIP_YEAR`; `county,ownership` | `Observed,Derived` / `PUBLISHED_ROLLUP` / `fact_qcew_county_rollup` / `fact_qcew_county_rollup` / `34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13` |
  | `production/qcew-county-establishments` | `FACET` / `ESTABLISHMENTS` / `UINT64_BITS` | `COUNTY_OWNERSHIP_YEAR`; `county,ownership` | `Observed,Derived` / `PUBLISHED_ROLLUP` / `fact_qcew_county_rollup` / `fact_qcew_county_rollup` / `34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13` |
  | `production/qcew-county-total-wages-usd` | `FACET` / `USD_CURRENT` / `FLOAT64_BITS` | `COUNTY_OWNERSHIP_YEAR`; `county,ownership` | `Observed,Derived` / `PUBLISHED_ROLLUP` / `fact_qcew_county_rollup` / `fact_qcew_county_rollup` / `34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13` |
  | `circulation/lodes-county-commuter-total-jobs` | `REFERENCE_FLOW` / `JOBS` / `UINT64_BITS` | `HOME_COUNTY_WORK_COUNTY_YEAR`; `home_county,work_county` | `Derived` / `LOAD_TIME_SUM` / `fact_lodes_commuter_flow` / `fact_lodes_commuter_flow` / `d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d` |
  | `reproduction/census-housing-households` | `FACET` / `HOUSEHOLDS` / `UINT64_BITS` | `COUNTY_SOURCE_TENURE_TIME_RACE`; `county,source,tenure,race` | `Observed` / `NONE` / `fact_census_housing` / `fact_census_housing` / `09ff2d9666b3f5ef267b65cbc77c14e99384f0157b6a4c898ac37df2e67ca59f` |
  | `reproduction/census-median-rent-usd` | `FACET` / `USD_CURRENT` / `FLOAT64_BITS` | `COUNTY_SOURCE_TIME_RACE`; `county,source,race` | `Observed` / `NONE` / `fact_census_rent` / `fact_census_rent` / `4c8cc134ec490ca75961d83485fc97c6bf240b32128e9d0517e00e62d578a99e` |
  | `reproduction/census-rent-burden-households` | `FACET` / `HOUSEHOLDS` / `UINT64_BITS` | `COUNTY_SOURCE_BURDEN_TIME_RACE`; `county,source,burden,race` | `Observed` / `NONE` / `fact_census_rent_burden` / `fact_census_rent_burden` / `8a42a51c17bf3ebee09f0b0b5145d5c8253c7e3446eec8c75714f9951b20df12` |
  | `reproduction/h3-population-persons` | `FACET` / `PERSONS` / `UINT64_BITS` | `H3_R7_VINTAGE`; `h3_cell` | `Derived` / `BLOCK_INTERNAL_POINT_ASSIGNMENT` / `h3_res7_population` / `h3_res7_population` / `b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc` |
  | `production/h3-workplace-jobs` | `FACET` / `JOBS` / `UINT64_BITS` | `H3_R7_VINTAGE`; `h3_cell` | `Derived` / `BLOCK_COORDINATE_ASSIGNMENT` / `h3_res7_workplace` / `h3_res7_workplace` / `ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6` |
  | `carceral/facility-count` | `FACET` / `FACILITIES` / `UINT64_BITS` | `COUNTY_COERCIVE_TYPE_SOURCE`; `county,coercive_type,source` | `Observed` / `NONE` / `fact_coercive_infrastructure` / `fact_coercive_infrastructure` / `33e6558d2b438e7aea672021f0e15f743f1ea331ab82407c0805a428b29cf808` |
  | `ecology/h3-land-fraction` | `FACET` / `FRACTION` / `FLOAT64_BITS` | `H3_R7_VINTAGE`; `h3_cell` | `Derived` / `EQUAL_AREA_WATER_INTERSECTION` / `h3_res7_land_mask` / `h3_res7_land_mask` / `4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194` |
  | `rootedness/presence`, `rootedness/solidarity`, `rootedness/membership` | `DYAD` / `TYPED_RELATION` / null | `ACTOR_NODE_VERIFIED_TICK`; `actor,node` | `Derived` / `TYPED_RELATION_PROJECTION` / committed typed graph / null / null; expand to three rows |

  The YAML also contains one closed `relation_binding_registry`. It is not a
  second metric registry. It binds each closed relation discriminant to an
  optional expanded metric identity and one generated
  `RelationPayloadModeV1` value:

  | Record family / exact kind | Expanded metric identity key or null | Payload mode |
  |---|---|---|
  | `REFERENCE_FLOW` / `COMMUTER_JOBS` | `circulation/lodes-county-commuter-total-jobs` | `SINGLE_METRIC_FACET` |
  | `REFERENCE_FLOW` / `BORDER_SYNTHESIS` | null | `EMPTY` |
  | `DYAD` / `PRESENCE` | `rootedness/presence` | `IMPLICIT_RELATION` |
  | `DYAD` / `MEMBERSHIP` | `rootedness/membership` | `IMPLICIT_RELATION` |
  | `DYAD` / `SOLIDARITY` | `rootedness/solidarity` | `IMPLICIT_RELATION` |
  | `DYAD` / `COMMAND` | null | `EMPTY` |

  `SINGLE_METRIC_FACET` requires exactly one `payload_facets` reference. The
  referenced `FacetV1.subject_ref` equals the flow identity, its metric equals
  the binding's complete expanded identity, and the metric row representation
  is `REFERENCE_FLOW`. An unreferenced `REFERENCE_FLOW` facet, a facet shared
  by two flows, a second payload facet, or a kind-to-metric mismatch refuses.
  `IMPLICIT_RELATION` requires an empty payload list: the dyad itself realizes
  the bound typed-relation metric, with `from_ref` and `to_ref` occupying the
  metric row's `actor,node` coordinates in that order. Its supplied native
  scale and evidence class must equal the expanded metric row. `EMPTY`
  requires no payload facet and makes no metric claim.

  For every supplied facet whose metric row has a non-null reference identity,
  the dossier must contain one exact `ReferenceDigestV1` with that identity
  and digest. Aggregation and producer identities remain governed registry and
  fixture-builder metadata because no V1 dossier record carries those fields;
  generic semantic validation must not pretend to compare absent wire data.
  Task 5 verifies them against the extraction ledger. A missing, extra,
  duplicated, or contradictory relation-binding row refuses generation.

  The YAML's `error_registry` is also closed and generated into both languages:
  `RTD_JSON`, `RTD_JSON_DEPTH`, `RTD_SCHEMA_VERSION`, `RTD_UNKNOWN_FIELD`,
  `RTD_ENUM`, `RTD_IDENTITY`, `RTD_DIGEST`, `RTD_NON_NFC`,
  `RTD_LIMIT_EXCEEDED`, `RTD_DUPLICATE_KEY`, `RTD_DANGLING_REF`,
  `RTD_STATUS_VALUE`, `RTD_NATIVE_GRAIN`, `RTD_UNSUPPORTED_DOWNSCALE`,
  `RTD_H3_BEFORE_PER21`, `RTD_MSA_EVIDENCE`, `RTD_CANADA_CONTROL`,
  `RTD_FORBIDDEN_REDUCTION`, `RTD_VECTOR_LIMIT`, and
  `RTD_CANONICAL_SIZE`. The generator refuses a missing, extra, duplicated, or
  reordered error row.

- [ ] **Step 4: Implement the deterministic, bounded generator.** Refuse raw contract input larger than 262,144 bytes before YAML parsing. Drive the event reader with `for event_index in range(RTD_MAX_YAML_EVENTS + 1)`, refuse the 65,537th event and every alias event, enforce nesting depth at most 16, and use a strict safe loader whose mapping constructor rejects duplicate keys before creating a dictionary. Validate the closed generator meta-model with ceilings of 32 records, 64 enums, 64 fields per record, 256 members per enum, and 512 limit/registry rows. Each record, enum, field, member, limit, error, and metric traversal uses its literal meta-model ceiling; no traversal accepts a caller-supplied bound or depends on an unchecked document count.

  Render both outputs in contract-declared record/field/enum order. Emit a generated-file header containing only the contract-relative path and SHA-256 of raw contract bytes. The output imports no application model. Every Python record sets `ConfigDict(frozen=True, extra="forbid")`.

  Every Rust record derives `Clone`, `Debug`, `PartialEq`, `Eq`, and `serde::Deserialize`, has `#[serde(deny_unknown_fields)]`, and uses `String`, `Vec`, `Option`, unsigned integers, and fixed `[u8; 32]` only where the contract defines binary digest storage. Rust enums derive `Deserialize` with exact string renames and no `other` variant. Neither generated output derives or supplies the canonical encoder.

  Fail before writing either output for an unknown top-level/meta-schema key, unknown record/enum/identity/metric/relation-binding/error key, duplicated field, enum, identity, metric, relation binding, or error name, two registry keys mapped to the same typed identity, an identity with a missing or changed component, unsupported scalar type, incomplete or contradictory sort declaration, a limit outside unsigned 64-bit range, a declared container without a named ceiling, a metric with an unknown unit/scale/coordinate/evidence/aggregation/producer/reference combination, or a relation binding with the wrong representation, metric, or payload mode. Resolve every metric and relation-binding registry key to its complete typed identity before rendering. Stage both generated texts, then replace both only after every check succeeds.

- [ ] **Step 5: Generate the checked-in structural types and make the red tests green.**

  Run: `uv run python tools/generate_rtd_v1_types.py`

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py`

  Expected: PASS, including an exact `--check` freshness assertion.

- [ ] **Step 6: Add generation and loader-bound mutations.** In the test, copy the contract to `tmp_path`; remove `GapV1`; change `max_focus` to `65`; remove and add one metric row; change one metric's unit, native scale, coordinate set, evidence set, and aggregation rule separately; remove, duplicate, and remap one relation-binding row; give `COMMUTER_JOBS` the wrong metric and payload mode separately; remove one identity-registry row; duplicate one identity key; change each of one identity's `domain`, `authority`, and `local_id` separately; map two keys to the same complete identity; remove and add one error row; add an unknown top-level key; add an unknown record key; duplicate a YAML mapping key; add an alias; supply 65,537 events; use depth 17; and exceed each meta-model ceiling by one. Assert `load_contract` returns the exact contract-schema or contract-limit error before either generated output changes. Restore the canonical contract unchanged.

- [ ] **Step 7: Format, verify, and commit the standalone contract keel.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py`

  Run: `uv run python tools/generate_rtd_v1_types.py --check`

  Run: `cd rust && cargo fmt --all -- --check`

  Commit: `feat(rtd): add closed v1 contract and generated structural types`

### Task 2: Implement Python semantic validation and bounded draft construction

**Files:**

- Create: `src/babylon/contracts/relational_territory_dossier_v1.py`
- Create: `tests/unit/contracts/test_rtd_v1_validation.py`
- Modify: `src/babylon/contracts/rtd_v1_generated.py` only through Task 1’s generator and only if its generated imports require it.

**Interfaces:**

- Consumes: generated `RtdDossierDraftV1`, nested records, constants, and `contracts/relational_territory_dossier_v1.yaml`.
- Produces:

  ```python
  class RtdValidationError(ValueError):
      code: str
      path: tuple[str | int, ...]

  def parse_draft(payload: Mapping[str, object]) -> RtdDossierDraftV1: ...
  def parse_draft_json(payload: bytes) -> RtdDossierDraftV1: ...
  def validate_draft(draft: RtdDossierDraftV1) -> None: ...
  def append_bounded[T](items: tuple[T, ...], item: T, kind: RtdCollectionKindV1, path: str) -> tuple[T, ...]: ...
  ```

  `parse_draft` is the typed internal mapping boundary. `parse_draft_json` first refuses more than 67,108,864 bytes, then scans with `for byte_index in range(RTD_MAX_JSON_INPUT_BYTES + 1)` and an explicit end-of-input branch in its JSON string/escape/depth state machine. It refuses nesting depth 33 before calling Pydantic and then performs closed-shape validation. `validate_draft` performs all cross-record, status/payload, duplicate, reference, metric-registry, native-grain, bound, and T1 boundary checks. None writes files or accepts a player/fog context.

- [ ] **Step 1: Write the validation red tests.** Add exact tests for the following named errors:

  ```python
  def test_present_zero_is_legal_but_unknown_zero_is_refused() -> None:
      assert validate_draft(present_zero_facet_draft()) is None
      with pytest.raises(RtdValidationError, match="RTD_STATUS_VALUE"):
          validate_draft(unknown_zero_facet_draft())

  def test_h3_identity_is_refused_before_per_21() -> None:
      with pytest.raises(RtdValidationError, match="RTD_H3_BEFORE_PER21"):
          validate_draft(draft_with_h3_typed_identity())

  def test_limit_plus_one_refuses_before_container_growth() -> None:
      items = tuple(identity(i) for i in range(64))
      with pytest.raises(RtdValidationError, match="RTD_LIMIT_EXCEEDED"):
          append_bounded(items, identity(64), RtdCollectionKindV1.FOCUS, "focus")
      assert len(items) == 64
  ```

  Also cover: unknown Pydantic field through both mapping and JSON entry points; unknown enum value; malformed JSON; depth 33; invalid/non-NFC identity component; 257-byte component; invalid digest case/length; duplicate typed identity; duplicate canonical key in each order-insensitive array; dangling reference; status/value mismatch in both directions; present non-finite float bits; valid negative-zero normalization to positive-zero bits; coordinate duplicate and native-grain mismatch; unknown metric; missing/extra coordinate; wrong facet unit, native scale, and evidence class; a missing or wrong reference-digest row; each of the six exact relation bindings; a `COMMUTER_JOBS` flow with zero, two, shared, wrong-subject, wrong-metric, or `FACET`-representation payloads; a typed dyad with a payload facet or wrong kind-to-metric/native-scale/evidence mapping; a nonempty `BORDER_SYNTHESIS` or `COMMAND` payload; an MSA membership; legacy `19820`; a Canadian geography identity; a Canadian canonical LODES flow; a hyperedge flattened into dyads; an unsupported downscale claim; and every closed collection limit plus one specified in Global Constraints. Aggregation and producer mutations belong to generated-registry and Task 5 extraction-ledger tests because those fields do not occur in a dossier record.

- [ ] **Step 2: Run validation tests and record the red state.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_validation.py`

  Expected: FAIL because `relational_territory_dossier_v1.py` does not exist.

- [ ] **Step 3: Implement parsing and the stable error taxonomy.** Use `TypeAdapter`/generated Pydantic types for closed-shape validation and translate Pydantic failures into `RtdValidationError` with one exact code from generated `RTD_V1_ERROR_REGISTRY` and a tuple path. Python and Rust tests assert equality with the generated registry; neither implementation redeclares the list. No catch-all error identity is allowed.

- [ ] **Step 4: Implement bounded semantic validation.** Accept only generated tuple fields, never an arbitrary iterable. Select limits only through `RtdCollectionKindV1`; perform a direct length preflight against its generated constant, then traverse with a fixed indexed range whose syntactic bound is the named compile-time maximum. Use checked integer comparisons and a private `set` of canonical encoded keys per set-like collection. Validate every reference before it is used; compare key bytes, not Python object identity.

  Enforce supplied metric representation, value kind, unit, native scale, exact coordinate-identity set, evidence class, relation payload mode, relation kind-to-metric binding, reference-artifact identity, and reference digest solely from generated `RTD_V1_METRIC_REGISTRY` and `RTD_V1_RELATION_BINDING_REGISTRY`. Compare each supplied identity's complete canonical bytes with the complete `TypedIdentityV1` expanded into those registries; never compare a symbolic key or `local_id` alone. Treat aggregation and producer identities as registry-only metadata here because the V1 records do not carry them; Task 5 checks those fields against the closed extraction ledger. Reject `PLAYER_KNOWLEDGE`, `COMMITTED`, non-null fog/knowledge/actor fields, or non-empty action/receipt/Archive display references in this T1 administrative module with `RTD_FORBIDDEN_REDUCTION` only when the payload would misrepresent the projection; otherwise use the more specific error code above.

- [ ] **Step 5: Make the Python validation suite green.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_validation.py`

  Expected: PASS.

- [ ] **Step 6: Prove atomic refusal.** Add a test that builds a draft with 65,535 gaps, attempts the 65,536th via `append_bounded(..., RtdCollectionKindV1.GAPS, ...)`, and asserts the original tuple remains byte-for-byte equal to its pre-attempt encoding. Add a test that validation failure never returns a draft, encoded bytes, or projection hash.

- [ ] **Step 7: Verify and commit.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py tests/unit/contracts/test_rtd_v1_validation.py`

  Commit: `feat(rtd): validate bounded administrative dossier drafts in python`

### Task 3: Add the isolated Rust RTD contract crate and matching semantic refusals

**Files:**

- Modify: `rust/Cargo.toml`
- Modify: `rust/Cargo.lock`
- Modify: `rust/crates/babylon-tick/Cargo.toml`
- Create: `rust/crates/babylon-rtd/Cargo.toml`
- Create: `rust/crates/babylon-rtd/src/lib.rs`
- Create: `rust/crates/babylon-rtd/src/validate.rs`
- Create: `rust/crates/babylon-rtd/tests/generated_contract.rs`
- Create: `rust/crates/babylon-rtd/tests/validation.rs`
- Modify: `rust/crates/babylon-rtd/src/generated.rs` only through Task 1’s generator.

**Interfaces:**

- Consumes: generated `RtdDossierDraftV1` and records in `generated.rs`; `babylon_kernel::sha256_of` is reserved for Task 4, not validation.
- Produces:

  ```rust
  pub enum RtdError {
      Json, JsonDepth, SchemaVersion, UnknownField, Enum, Identity, Digest,
      NonNfc, LimitExceeded, DuplicateKey, DanglingReference, StatusValue,
      NativeGrain, UnsupportedDownscale, H3BeforePer21, MsaEvidence,
      CanadaControl, ForbiddenReduction, VectorLimit, CanonicalSize,
  }

  pub fn validate_draft(draft: &RtdDossierDraftV1) -> Result<(), RtdError>;
  pub fn parse_draft_json(payload: &[u8]) -> Result<RtdDossierDraftV1, RtdError>;
  pub fn append_bounded<T: Clone>(items: &[T], item: T, kind: RtdCollectionKindV1) -> Result<Vec<T>, RtdError>;
  ```

  `RtdError` carries no catch-all variant and implements `Display` with its stable `RTD_*` identity. `parse_draft_json` is the public untrusted-input boundary and enforces the same raw-byte and depth ceilings as Python before `serde_json` deserialization. The Task 4 vector reader calls this API rather than maintaining a second draft parser.

- [ ] **Step 1: Write Rust red tests.** `generated_contract.rs` asserts the generated schema id/version/limits, exact 20-row error registry, exact expanded identity registry, exact 18-row metric registry, exact six-row relation-binding registry, `Deserialize` closure, `deny_unknown_fields`, and that each contract enum rejects an unrecognized discriminant. `validation.rs` constructs the same minimal semantic cases as Task 2 and asserts exact `RtdError` variants for malformed JSON, JSON depth 33, unknown field, present-zero, unknown-zero, negative-zero normalization, duplicate key, dangling ref, every closed collection limit plus one, unknown metric, missing/extra coordinate, wrong facet unit/scale/evidence, missing or wrong reference digest, every relation payload/binding mutation, H3, MSA, and Canadian control attempts. Generated-registry tests, not dossier validation, mutate aggregation and producer metadata.

- [ ] **Step 2: Run the isolated Rust tests and record the red state.**

  Run: `cd rust && cargo test -p babylon-rtd --test generated_contract --locked`

  Expected: FAIL because `babylon-rtd` is not yet a workspace member.

- [ ] **Step 3: Register the crate and refresh all T1 manifest changes once.** Add `crates/babylon-rtd` to `rust/Cargo.toml`. Create `Cargo.toml` with workspace package metadata and dependencies `babylon-kernel = { path = "../babylon-kernel" }`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, and `unicode-normalization = "0.1"`. Add `serde` with derive and `serde_json` to `babylon-tick` dev-dependencies now for Task 5's identity witness. Do not add a direct SHA package or a general schema framework. Put `#![forbid(unsafe_code)]` and `#![warn(clippy::pedantic)]` at the crate root.

  Run one serialized unlocked `cd rust && cargo check -p babylon-rtd --tests`, then inspect `rust/Cargo.lock` and require that only the new workspace package entry/dependency references and the declared `babylon-tick` dev-dependency references moved. Every later Cargo command in T1 uses `--locked`; no Task 5 refresh is permitted.

- [ ] **Step 4: Implement type-facing parsing and validation.** Re-export generated records from `lib.rs`; generate every `RtdError` variant/display code from the one error registry; scan with `for byte_index in 0..=RTD_MAX_JSON_INPUT_BYTES` after the raw length preflight and refuse nesting depth 33 before `serde_json`; require full JSON consumption; validate sorted/unique canonical key sequences, reference closure, metric-registry semantics, relation-binding semantics, native grain, status/payload forms, administration-only fields, and every bounded collection. Select limits only through `RtdCollectionKindV1`; after a direct length preflight, traverse with a fixed indexed range bounded by the generated compile-time maximum, never `.take(limit + 1)` or a caller integer. Use `u64::try_from`, `checked_add`, and exact `usize` conversions. No `unwrap`, unsafe code, or generic fallback error is permitted in library paths.

- [ ] **Step 5: Make Rust validation green.**

  Run: `cd rust && cargo test -p babylon-rtd --test generated_contract --locked`

  Run: `cd rust && cargo test -p babylon-rtd --test validation --locked`

  Expected: PASS.

- [ ] **Step 6: Add refusal and canonicalization mutations.** In the Rust test, change the H3 gap’s `required_producer_or_null` from `PER-21` to a different value and assert `H3BeforePer21`; add the forbidden MSA membership and assert `MsaEvidence`; repeat one focus identity and assert `DuplicateKey`. Separately swap two distinct focus identities, require successful validation, and require byte-identical canonical output to the original order.

- [ ] **Step 7: Format, lint, and commit.**

  Run: `cd rust && cargo fmt --all -- --check`

  Run: `cd rust && cargo clippy -p babylon-rtd --all-targets --locked -- -D warnings`

  Commit: `feat(rtd): add rust bounded dossier validation contract`

### Task 4: Implement independent canonical encoders and the shared vector corpus

**Files:**

- Create: `contracts/relational_territory_dossier_v1_vectors.jsonl`
- Create: `src/babylon/contracts/relational_territory_dossier_v1.py` additions for encoding/sealing
- Create: `rust/crates/babylon-rtd/src/canonical.rs`
- Modify: `rust/crates/babylon-rtd/src/lib.rs`
- Create: `tests/unit/contracts/test_rtd_v1_canonical.py`
- Create: `rust/crates/babylon-rtd/tests/canonical_vectors.rs`

**Interfaces:**

- Python produces:

  ```python
  def canonical_draft_bytes(draft: RtdDossierDraftV1) -> bytes: ...
  def projection_hash(draft: RtdDossierDraftV1) -> str: ...
  def seal_draft(draft: RtdDossierDraftV1) -> RelationalTerritoryDossierV1: ...
  ```

- Rust produces:

  ```rust
  pub fn canonical_draft_bytes(draft: &RtdDossierDraftV1) -> Result<Vec<u8>, RtdError>;
  pub fn projection_hash(draft: &RtdDossierDraftV1) -> Result<[u8; 32], RtdError>;
  pub fn seal_draft(draft: RtdDossierDraftV1) -> Result<RelationalTerritoryDossierV1, RtdError>;
  ```

- The JSONL corpus has only two line forms:

  ```json
  {"case_id":"minimal-admin","kind":"valid","draft":{},"canonical_utf8_hex":"","projection_hash":""}
  {"case_id":"duplicate-focus","kind":"invalid","draft":{},"error":"RTD_DUPLICATE_KEY"}
  ```

  Each actual vector fills the contract-valid draft object and exact expected fields. Python and Rust consume the repository-root bytes, refuse more than 1,048,576 file bytes before line parsing, inspect at most 257 lines, refuse a line larger than 262,144 bytes or JSON depth 33, require a 1-through-128-byte NFC `case_id`, require full JSON consumption, and reject an unknown/extra line field, unknown `kind`, or duplicate `case_id`.

- [ ] **Step 1: Write cross-language vector red tests.** Add a minimal valid administrative draft, a permuted-focus equivalent, a present-zero facet, one n-member hyperedge, and one semantic mutation. Assert Python’s canonical bytes equal the listed hex exactly; assert `projection_hash` equals the listed lowercase hex; assert the permutation produces identical bytes; assert the mutation changes hash. Add 1,048,577-byte, 257-line, 262,145-byte-line, 129-byte-case-id, depth-33, duplicate-case, unknown-kind, trailing-token, and unknown-line-field refusal witnesses. Rust consumes the exact same checked-in JSONL bytes with `include_str!("../../../../contracts/relational_territory_dossier_v1_vectors.jsonl")`; temporary over-limit witnesses exercise the same bounded reader. Do not copy the corpus into a crate-local fixture.

- [ ] **Step 2: Run both encoder tests and record the red state.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_canonical.py`

  Run: `cd rust && cargo test -p babylon-rtd --test canonical_vectors --locked`

  Expected: FAIL because the vector corpus and canonical APIs do not exist.

- [ ] **Step 3: Write the vector corpus from the contract, not generated encoder output.** Hand-author the initial expected canonical UTF-8 hex and SHA-256 entries once from the normative field/sort/escape rules, then verify each is consumed by both implementations. Include exact invalid vectors for every Task 2 stable error and valid vectors for empty optionals, explicit null, non-ASCII NFC strings, positive zero normalized from negative-zero input bits, and each order-preservation exception in `DecisionSurfaceV1`.

- [ ] **Step 4: Implement the Python vector reader and canonical writer.** Bound raw corpus bytes first and use `itertools.islice(lines, 257)` before any per-case work. Each line passes the same bounded JSON scanner as `parse_draft_json`. For canonical output, do not call `json.dumps` on an arbitrary Pydantic `model_dump`. Emit bytes through a private `bytearray`; validate before emission; sort every object key by `key.encode("utf-8")`; apply only the contract's declared array-set sorting; write required escaping; encode integers without leading zeros; emit typed numeric values only as the contract's sixteen lowercase bit hex; use checked byte-count comparisons against the 67,108,864 maximum. On error, discard the private buffer and raise `RtdValidationError`.

- [ ] **Step 5: Implement the Rust vector reader and canonical writer independently.** Refuse raw corpus bytes first; traverse `.split_inclusive('\n').take(257)` and refuse the 257th line before case parsing; apply the exact line/case/depth/full-consumption rules above. Do not serialize a `serde_json::Value` or reuse Python fixture output for canonical output. Validate first; write to a private `Vec<u8>` via small dedicated functions for string, identity, null, enum, unsigned integer, and each record; sort object keys and contract-declared sets by raw UTF-8 bytes; maintain a `u64` checked count before every write. Typed values are already bit strings; validation normalizes negative-zero float bits and rejects non-finites before emission. Call `babylon_kernel::sha256_of` only after writing `domain-separator || 0x00 || canonical-draft-bytes`.

- [ ] **Step 6: Seal only after a successful hash.** `seal_draft` validates and encodes the complete draft, computes the projection hash, then creates the sealed record. A failure returns no sealed record. Test that a 67,108,865-byte witness causes `RTD_CANONICAL_SIZE` and does not expose bytes/hash in Python or Rust.

- [ ] **Step 7: Make both implementations green and prove independence.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_canonical.py`

  Run: `cd rust && cargo test -p babylon-rtd --test canonical_vectors --locked`

  Expected: PASS with every valid vector byte-for-byte equal and every invalid vector mapped to the same stable error identity.

- [ ] **Step 8: Add encoder mutations.** Change Python focus sorting to `local_id` only and assert the typed-identity permutation vector fails. Change Rust decision-surface list sorting to canonical-set sorting and assert the display-order vector fails. Restore both implementations and re-run both suites.

- [ ] **Step 9: Verify and commit.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py tests/unit/contracts/test_rtd_v1_validation.py tests/unit/contracts/test_rtd_v1_canonical.py`

  Run: `cd rust && cargo test -p babylon-rtd --test generated_contract --test validation --test canonical_vectors --locked`

  Run: `cd rust && cargo clippy -p babylon-rtd --all-targets --locked -- -D warnings`

  Commit: `feat(rtd): pin independent canonical dossier encoders and vectors`

### Task 5: Add the Detroit-Windsor unfogged administrative control fixture

**Files:**

- Create: `contracts/fixtures/detroit_windsor_rtd_v1_admin_control.json`
- Create: `contracts/fixtures/detroit_windsor_rtd_v1_extraction.yaml`
- Create: `contracts/fixtures/detroit_windsor_rtd_v1_admin_world.bscn`
- Create: `contracts/fixtures/detroit_windsor_rtd_v1_admin_noop.bsl`
- Create: `contracts/fixtures/detroit_windsor_rtd_v1_world_identity.json`
- Create: `tools/build_detroit_rtd_control.py`
- Create: `tests/unit/contracts/test_rtd_v1_detroit_control.py`
- Create: `rust/crates/babylon-tick/tests/rtd_admin_fixture_identity.rs`
- Modify: `contracts/relational_territory_dossier_v1_vectors.jsonl`

**Interfaces:**

- Consumes: `parse_draft`, `validate_draft`, `seal_draft`, the shared vector corpus, the checked extraction ledger, the minimal scenario/rule, and the world-identity witness. The optional source verification root is exactly `/media/user/data/babylon-data/backups/data-artifacts-v7`; the checked output does not embed that host path.
- Produces: a checked-in sealed `RelationalTerritoryDossierV1` with `audience: ADMIN_MATERIAL`, `durability: IN_MEMORY`, null fog/knowledge/actor fields, three county focus identities (`26163`, `26125`, `26099`), independent Michigan/United States/CZ memberships, exact verified QCEW/LODES/carceral facts, three explicit Census provenance-conflict gaps, the rest of the complete typed-gap registry, and a projection hash pinned through both encoders.
- Builder CLI: `uv run python tools/build_detroit_rtd_control.py [--check] [--verify-source-root PATH]`. The default build reads only checked inputs. `--check` stages bytes and compares without writing. `--verify-source-root` additionally verifies the exact pinned relative paths, artifact digests, selectors, row counts, values, and bit strings; it performs no discovery, database write, live projection, or network access.
- World identity: `verified_tick = 1`; `graph_state_hash` is `TickReport.after`; `nominal_world_hash` is `TickReport.world_after`; `scenario_digest` and `rule_digest` are raw-file SHA-256; `definitions_digest` is the current Python `canonical_defines_hash(GameDefines.load_default())`; and `template_digest` is raw SHA-256 of the extraction ledger. A literal all-zero digest, null digest, manually invented tick hash, or self-authenticating copy is forbidden.

- [ ] **Step 1: Write fixture, extraction, and world-identity red tests.** Assert the control parses and seals to its listed hash; it is `ADMIN_MATERIAL` and `IN_MEMORY`; `fog_policy_digest`, `knowledge_context_digest`, and `actor` are null; the county focus is exactly the canonical full typed identities for Wayne/Oakland/Macomb; every county has independent Michigan `26`, United States `US`, and ERS CZ `11600` memberships; and `DecisionSurfaceV1.question_id` is the T1 territorial-relation question while its action, receipt, and Archive-subject lists are empty.

  Assert the builder's `--check` result is green only when the exact extraction, world-identity, and control bytes agree. Assert `scenario_digest`, `rule_digest`, `definitions_digest`, and `template_digest` recompute from their authoritative bytes/APIs. Assert `verified_tick == 1`, both tick hashes are nonzero lowercase 64-hex, and the world hash differs from the graph hash. Add the Rust test red state: it includes the scenario/rule, executes public `run_once`, and compares `hex(report.after)` and `hex(report.world_after)` with the JSON witness rather than comparing the JSON to itself.

  Assert that no Census housing, rent, or rent-burden facet exists; instead the
  three exact requested metric identities occur once each as `UNKNOWN` gaps
  with `PROVENANCE_COORDINATE_CONFLICT` and `PER-28`. Add the following negative
  assertions:

  ```python
  assert not any(is_h3_identity(value) for value in all_typed_identities(control))
  assert not any(m.scale_ref.local_id == "19820" for m in control.scale_memberships)
  assert not any(flow.destination_ref.authority == "canada" for flow in control.flows)
  assert not any("score" in field_name or "stage" in field_name for field_name in wire_field_names(control))
  ```

- [ ] **Step 2: Run the fixture test and record the red state.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_detroit_control.py`

  Expected: FAIL because the fixture inputs and outputs do not exist.

- [ ] **Step 3: Create the closed extraction ledger with exact artifact coordinates.** Require exactly 19 artifact rows, allow at most 128 selectors, 128 selected rows, 32 provenance locators per artifact, and exactly 20 gap rows. Reject aliases, duplicate YAML keys, unknown fields, over-limit raw bytes, and every fixed-bound plus-one witness before a row is materialized. Pin these artifact digests and exact selected values; each numeric cell stores both its source decimal and its required 16-character bit string so the builder never recomputes fixture meaning by aggregation:

  | Artifact / coordinate | Exact selected values |
  |---|---|
  | `fact_qcew_county_rollup`, SHA-256 `34c2bbb935f79b3c8076a97092b004b14cca120e8272b93c35b3ac9dc2721d13`, time 28 / ownership 1 | Macomb establishments `19678` / employment `336295` / wages `23892513876.0`; Oakland `43047` / `723862` / `56401482100.0`; Wayne `36727` / `725504` / `55436615328.0`. Require `disclosure_code = null` and `is_imputed = false`; the dimensions resolve time 28 to annual 2024 and ownership 1 to code 0. |
  | `fact_lodes_commuter_flow`, SHA-256 `d3745f8def09cd8c7a38e1870e6ec2c1853e210b777d8e8358cfce36665bd64d`, time 24 | Macomb→Macomb `154024`, Macomb→Oakland `111077`, Macomb→Wayne `65374`; Oakland→Macomb `51184`, Oakland→Oakland `309277`, Oakland→Wayne `111785`; Wayne→Macomb `48174`, Wayne→Oakland `144397`, Wayne→Wayne `363507`. The dimension resolves time 24 to annual 2020. |
  | `fact_census_housing`, SHA-256 `09ff2d9666b3f5ef267b65cbc77c14e99384f0157b6a4c898ac37df2e67ca59f`, source 2 / time 27 / race 1 | Preserve Macomb total `356426`, owner `267161`, renter `89265`; Oakland `528681`, `382057`, `146624`; Wayne `693446`, `447120`, `246326` only as conflicting extraction evidence. Source 2 and time 27 fail the pinned dimension join, so none becomes a facet. |
  | `fact_census_rent`, SHA-256 `4c8cc134ec490ca75961d83485fc97c6bf240b32128e9d0517e00e62d578a99e`, source 2 / time 27 / race 1 | Preserve Macomb `1175.0` / `40925c0000000000`; Oakland `1319.0` / `40949c0000000000`; Wayne `1087.0` / `4090fc0000000000` only as conflicting extraction evidence. |
  | `fact_census_rent_burden`, SHA-256 `8a42a51c17bf3ebee09f0b0b5145d5c8253c7e3446eec8c75714f9951b20df12`, burden 9 / source 2 / time 27 / race 1 | Preserve Macomb `20707`, Oakland `31178`, Wayne `67261` only as conflicting extraction evidence. |
  | `fact_coercive_infrastructure`, SHA-256 `33e6558d2b438e7aea672021f0e15f743f1ea331ab82407c0805a428b29cf808`, source 11 / coercive types 2 and 3 | Macomb state-prison count `1`, local-jail count `2`; Oakland local-jail `4`; Wayne state-prison `1`, local-jail `4`. Do not project capacity. |
  | `bridge_county_cz.csv`, SHA-256 `a04cc4fc2bf0b6e96bc0d2a47c1fc91d29d41174e3e339e15f81481f862808df` | Counties `26099`, `26125`, and `26163` each map independently to CZ `11600`. |
  | `dim_county`, SHA-256 `130b7679d0441d5c3c2183a2bef858073d3011039550bfbf015b380566c72032`; `dim_state`, SHA-256 `22245af6240648b4f3c50b748ad142204c2c985017db80b2451c5140b8898398` | County ids Macomb `1281`, Oakland `1294`, Wayne `1313` all join state id `23`; state id `23` is Michigan FIPS `26`. The United States `US` membership is `Designed` contract identity, not falsely attributed to either dimension. |
  | Three H3 metric artifact references | Population `b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc`; workplace `ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6`; land fraction `4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194`. The inspected `data-artifacts-v7` backup does not contain these three files, so the ledger marks them `verification_mode: REFERENCE_DIGEST_ONLY`. They remain provenance-only contract references before PER-21; the verifier must not claim it rehashed an absent file. |

  Each present Parquet ledger row pins this complete physical metadata. Every
  Parquet artifact below has exactly one row group; field order and Arrow type
  are part of the pinned schema string:

  | Artifact | Bytes / rows | Exact schema |
  |---|---:|---|
  | `fact_qcew_county_rollup` | `2231358` / `240488` | `county_id:int64,time_id:int64,ownership_id:int64,establishments:int64,employment:int64,total_wages_usd:double,disclosure_code:string,is_imputed:bool` |
  | `fact_lodes_commuter_flow` | `19056915` / `2645347` | `home_county_id:int64,work_county_id:int64,time_id:int64,total_jobs:int64,jobs_age_29_under:int64,jobs_age_30_54:int64,jobs_age_55_plus:int64,jobs_earn_low:int64,jobs_earn_mid:int64,jobs_earn_high:int64` |
  | `fact_census_housing` | `2386094` / `1351380` | `county_id:int64,source_id:int64,tenure_id:int64,time_id:int64,race_id:int64,household_count:int64` |
  | `fact_census_rent` | `77226` / `44997` | `county_id:int64,source_id:int64,time_id:int64,race_id:int64,median_rent_usd:double` |
  | `fact_census_rent_burden` | `792752` / `450450` | `county_id:int64,source_id:int64,burden_id:int64,time_id:int64,race_id:int64,household_count:int64` |
  | `fact_coercive_infrastructure` | `18531` / `3867` | `county_id:int64,coercive_type_id:int64,source_id:int64,facility_count:int64,total_capacity:int64` |
  | `dim_county` | `36199` / `3285` | `county_id:int64,fips:string,state_id:int64,county_fips:string,county_name:string,h3_res4:string` |
  | `dim_state` | `2099` / `52` | `state_id:int64,state_fips:string,state_name:string,state_abbrev:string` |
  | `dim_data_source` | `5628` / `21` | `source_id:int64,source_code:string,source_name:string,source_url:string,description:string,source_year:int64,source_agency:string,coverage_start_year:int64,coverage_end_year:int64` |
  | `dim_time` | `3102` / `485` | `time_id:int64,year:int64,month:int64,quarter:int64,is_annual:bool` |
  | `dim_ownership` | `1601` / `7` | `ownership_id:int64,own_code:string,own_title:string,is_government:bool,is_private:bool` |
  | `dim_housing_tenure` | `1448` / `3` | `tenure_id:int64,tenure_type:string,tenure_label:string,is_owner:bool` |
  | `dim_race` | `2591` / `10` | `race_id:int64,race_code:string,race_name:string,race_short_name:string,is_hispanic_ethnicity:bool,is_indigenous:bool,display_order:int64` |
  | `dim_rent_burden` | `2876` / `11` | `burden_id:int64,bracket_code:string,burden_bracket:string,burden_min_pct:double,burden_max_pct:double,is_cost_burdened:bool,is_severely_burdened:bool,bracket_order:int64` |
  | `dim_coercive_type` | `2101` / `15` | `coercive_type_id:int64,code:string,name:string,category:string,command_chain:string` |

  The remaining present artifact is
  `src/babylon/data/reference/bridge_county_cz.csv`, exactly 103,192 bytes,
  3,142 LF-terminated lines including header
  `county_fips,cz_id,cz_name`, and 3,141 data rows. The other three artifact
  rows are the explicitly absent H3 references. Thus the ledger has exactly 19
  artifact rows.

  Pin these dimension digests and selected mappings in addition to the fact
  digests above:

  | Dimension digest | Required selected mappings |
  |---|---|
  | `dim_data_source` `cf03d6be85ef94da5cee948896c9147993cabbfdf574be3eba30e0069379929f` | source 2 = `ACS5Y2010_API`, source year 2010, coverage 2006–2010; source 4 = `ACS5Y2023_API`, source year 2023, coverage 2019–2023; source 11 = `HIFLD_PRISONS_2024`, source year 2024. |
  | `dim_time` `6049f93d5686eea1aa954d831d19aad91320e449b284250b9dd7a61775ed849b` | 24 = annual 2020; 27 = annual 2023; 28 = annual 2024; each has null month and quarter. |
  | `dim_ownership` `d8a50b0d2f293b7daab6771b1fd718faaf2a4fc137da1976b9cac587bc97c54e` | ownership 1 = code `0`, title `Ownership 0`, neither government nor private. |
  | `dim_housing_tenure` `f8afedd901c152680f405984c778f97d5c926c6ceeacd152d825ecc06bf28085` | 1 = total, 2 = owner, 3 = renter. |
  | `dim_race` `e7fe6e44956d3e3fbdab9aa1099cdd1d402e2ea4d3c1a9e448620ca1d227a02d` | race 1 = code `T`, `Total (all races)`, display order 0. |
  | `dim_rent_burden` `9f9c850c4c66cbe45faac638a77d65e13ddac693a34ea8990393743b37a558bc` | burden 9 = `B25070_010`, 50 percent or more, minimum 50, severe and cost burdened, order 9. |
  | `dim_coercive_type` `bc3301f4037e23459a43629e8b8e3c1bd7ffad05e0abd277b19214555da29ecd` | 2 = `prison_state`, state command; 3 = `prison_local`, local command. |
  | `dim_county` `130b7679d0441d5c3c2183a2bef858073d3011039550bfbf015b380566c72032` | 1281 = Macomb `26099`, 1294 = Oakland `26125`, 1313 = Wayne `26163`; all reference state id 23 and have null H3. |
  | `dim_state` `22245af6240648b4f3c50b748ad142204c2c985017db80b2451c5140b8898398` | state 23 = Michigan, FIPS `26`, abbreviation `MI`. |

  The verifier also requires zero selected source-4 rows for all three Census
  tables at time 27 and the three selected counties. That zero is evidence of
  the conflict, not evidence that the requested value is zero.

  Integer values use zero-padded 16-character unsigned-bit hex in the ledger (`19678 = 0000000000004cde`, for example); all other integer bit strings are produced and then independently asserted from the exact decimal table above. Floating values use the exact strings listed. The verifier applies exact equality—never tolerance—to integer, bit-string, ID, boolean, digest, and null coordinates.

- [ ] **Step 4: Freeze all unavailable material as twenty typed gaps.** Every `requested_metric_or_relation` is `{domain: "metric-or-relation", authority: "babylon.rtd.v1", local_id: <literal below>}`. Create exactly these rows and no catch-all gap:

  | Gap id suffix / requested literal | Status / reason / required producer |
  |---|---|
  | `omb-msa-detroit-tri-county` / `scale-membership/omb-msa` | `NOT_COMPUTED` / `MISSING_GOVERNED_OMB_DELINEATION` / null |
  | `h3-population` / `reproduction/h3-population-persons` | `NOT_COMPUTED` / `IDENTITY_CONTRACT_PENDING` / `PER-21` |
  | `h3-workplace` / `production/h3-workplace-jobs` | `NOT_COMPUTED` / `IDENTITY_CONTRACT_PENDING` / `PER-21` |
  | `h3-land-fraction` / `ecology/h3-land-fraction` | `NOT_COMPUTED` / `IDENTITY_CONTRACT_PENDING` / `PER-21` |
  | `census-housing-source-vintage-conflict` / `reproduction/census-housing-households` | `UNKNOWN` / `PROVENANCE_COORDINATE_CONFLICT` / `PER-28` |
  | `census-rent-source-vintage-conflict` / `reproduction/census-median-rent-usd` | `UNKNOWN` / `PROVENANCE_COORDINATE_CONFLICT` / `PER-28` |
  | `census-rent-burden-source-vintage-conflict` / `reproduction/census-rent-burden-households` | `UNKNOWN` / `PROVENANCE_COORDINATE_CONFLICT` / `PER-28` |
  | `command-administrative-centrality` / `command/administrative-centrality` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `freight-road-corridor` / `circulation/freight-road-corridor-intensity` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / `PER-31` |
  | `eviction` / `reproduction/eviction` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `foreclosure` / `reproduction/foreclosure` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `absentee-ownership` / `reproduction/absentee-ownership` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `agricultural-tenure-displacement` / `extraction/agricultural-tenure-displacement` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `indigenous-jurisdiction` / `jurisdiction/indigenous` | `UNKNOWN` / `REFERENCE_COVERAGE_UNAVAILABLE` / null |
  | `care-capacity` / `care/capacity` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `ecology-beyond-land-fraction` / `ecology/beyond-land-fraction` | `NOT_COMPUTED` / `MISSING_GOVERNED_PRODUCER` / null |
  | `windsor-essex-spatial-membership` / `scale-membership/windsor-essex` | `UNKNOWN` / `REFERENCE_COVERAGE_UNAVAILABLE` / null |
  | `player-fog` / `player/fog` | `NOT_COMPUTED` / `PLAYER_BOUNDARY_UNAVAILABLE` / `PER-22` |
  | `action-eligibility` / `player/action-eligibility` | `NOT_COMPUTED` / `PLAYER_BOUNDARY_UNAVAILABLE` / `PER-27` |
  | `organization-practice-state` / `organization/practice-state` | `NOT_COMPUTED` / `PLAYER_BOUNDARY_UNAVAILABLE` / `PER-56` |

  The OMB gap provenance names the rejected legacy export/code and the missing pinned delineation remedy without creating `19820`. The three H3 gaps each reference their available metric artifact plus PER-21. Each Census gap references its fact table, `dim_data_source`, `dim_time`, the applicable categorical dimensions, and PER-28; it carries no numeric value. The fixture inserts no null H3 identity, cell vector, weighted membership, Windsor geography, Canadian endpoint/commute flow, MSA membership, or Census facet to accompany a gap.

- [ ] **Step 5: Build and pin a real administrative world identity.** Write the minimal scenario with exactly three `TERRITORY` nodes and an integer administrative FIPS field for `26099`, `26125`, and `26163`; write a rule whose governed guard is false for all three and whose effect would be identity-preserving if reached. Run the public tick binary twice and require byte-identical reports:

  Run: `cd rust && cargo run -p babylon-tick --locked -- ../contracts/fixtures/detroit_windsor_rtd_v1_admin_world.bscn ../contracts/fixtures/detroit_windsor_rtd_v1_admin_noop.bsl`

  Record its real `graph-after` and `world-after` along with recomputed input/defines/template digests in `world_identity.json`. Task 3 already declared and locked the `babylon-tick` test dependencies. Run `cd rust && cargo check -p babylon-tick --tests --locked`; any lockfile movement fails. The Rust identity test parses the closed JSON record with `deny_unknown_fields`, recomputes raw input digests through `babylon_kernel::sha256_of`, executes `run_once`, asserts `fired == 0`, and compares both real hashes.

  Mutating scenario state must move the scenario digest and must move the graph/world witness whenever that state changes the tick result. Mutating the rule guard must move the raw `rule_digest` and invalidate the checked record; graph and nominal-world hashes may remain equal when the changed rule still produces no material state change. Do not claim that `NominalWorldHash` binds raw rule bytes.

- [ ] **Step 6: Implement the bounded builder and optional source verifier.** The builder uses fixed indexed loops over exactly 19 artifact slots, 128 selector slots, 128 selected-row slots, 32 locator slots per artifact, and 20 gap slots. It constructs the explicit control draft, checks the real world identity, seals privately, and writes only after every validation succeeds. Source verification uses only exact paths declared in the extraction ledger and first requires the pinned byte size, SHA-256, ordered Arrow schema, row count, and one-row-group count. The absolute source ceilings are 19,056,915 bytes, 2,645,347 rows, one row group, 65,536 rows per batch, and 41 batches.

  Before row selection, resolve every extraction-ledger metric through the
  generated registry and require the ledger's producer identity, aggregation
  rule, optional reference-artifact identity, and digest to match that row
  exactly. The registry comparison is the authoritative T1 check for metadata absent
  from dossier records. A mismatch refuses before any source file is opened
  or output bytes are staged.

  Read only required columns through `ParquetFile.iter_batches(row_groups=[0], batch_size=65_536)`. Drive artifact, row-group, batch, and row traversal with those literal constants, apply the closed selectors inside each admitted batch, and require exact selected cardinality and value/bit equality. Never call an unbounded table-wide `read_table` or materialize a caller-supplied iterable. Verify the tracked CSV's exact bytes, line count, header, selected mappings, and digest in a separate fixed 3,142-line pass.

  `REFERENCE_DIGEST_ONLY` entries are accepted only for the three explicitly closed H3 artifact identities and are reported as unrehashable references, never as verified source files. A missing required root/file, metadata or digest mismatch, extra/missing selected row, duplicate selector, source-2/source-4 mutation, changed CZ mapping, or fourth reference-only artifact returns a named error and leaves the checked fixture untouched.

- [ ] **Step 7: Add the opt-in Canadian synthesis vector.** Keep the canonical control unchanged. Add one separately named valid vector whose closed envelope case ID is exactly `border-synthesis-opt-in`, with exactly one `BORDER_SYNTHESIS` `ReferenceFlowV1` from Detroit Census place `{domain: "place", authority: "census", local_id: "2622000"}` to `{domain: "external", authority: "babylon.rtd.v1", local_id: "canada"}`, one `Derived` provenance row, and no Canadian spatial membership or Windsor identity. Do not add `enabled` or any other field to the closed vector envelope or draft. The default fixture builder selects only the exact case ID `detroit-windsor-admin-control`; tests prove it refuses any other requested default case ID and never selects `border-synthesis-opt-in`. Assert the opt-in vector does not use `COMMUTER_JOBS` or cite the LODES artifact as a Canadian observation.

- [ ] **Step 8: Add the sealed fixture as a shared valid vector.** Its JSONL entry carries the complete draft and pins canonical UTF-8 hex and `projection_hash`. Both language suites re-encode that draft; the Python fixture test additionally rebuilds from the independent extraction ledger and real world-identity witness. No test treats a digest stored beside its own payload as sufficient evidence.

- [ ] **Step 9: Make the fixture and source-verification tests green.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_detroit_control.py tests/unit/contracts/test_rtd_v1_canonical.py`

  Run: `uv run python tools/build_detroit_rtd_control.py --check --verify-source-root /media/user/data/babylon-data/backups/data-artifacts-v7`

  Run: `cd rust && cargo test -p babylon-rtd --test canonical_vectors --locked`

  Run: `cd rust && cargo test -p babylon-tick --test rtd_admin_fixture_identity --locked`

  Expected: PASS.

- [ ] **Step 10: Add boundary mutations.** Inject (a) H3 cell `8928308280fffff`, (b) MSA `19820`, (c) a Detroit-to-Canada `COMMUTER_JOBS` flow, (d) an area-overlap weight, (e) a player-facing action reference, (f) an omitted mandatory gap, and (g) a twenty-first unregistered gap, with cardinality 21 asserted before validation. Assert the fixture validator refuses each with its named T1 boundary error and leaves the original fixture, extraction ledger, and world-identity bytes unchanged.

- [ ] **Step 11: Verify and commit.**

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py tests/unit/contracts/test_rtd_v1_validation.py tests/unit/contracts/test_rtd_v1_canonical.py tests/unit/contracts/test_rtd_v1_detroit_control.py`

  Run: `cd rust && cargo test -p babylon-rtd --test generated_contract --test validation --test canonical_vectors --locked`

  Run: `cd rust && cargo test -p babylon-tick --test rtd_admin_fixture_identity --locked`

  Commit: `test(rtd): add Detroit administrative control with typed gaps`

### Task 6: Freeze the T1 contract boundary and perform the focused review

**Files:**

- Modify: `contracts/relational_territory_dossier_v1.yaml` only if review exposes a contract/type/vector disagreement.
- Modify: `contracts/relational_territory_dossier_v1_vectors.jsonl` only if a test demonstrates a missing required acceptance witness.
- Modify: generated outputs only by re-running `tools/generate_rtd_v1_types.py` after an approved contract correction.
- Test: all T1 tests, source verification, `babylon-rtd`, and the `babylon-tick` fixture-identity witness.

**Interfaces:**

- Consumes: all previous task outputs.
- Produces: a PER-52 completion comment that identifies the validated contract
  digest, vector count, final branch SHA, Python/Rust command results, and the
  preserved PER-21/player-fog exclusions. A correction commit may repeat that
  record, but the Linear comment is the required clean-path artifact. No runtime
  API, database row, or player surface is added.

- [ ] **Step 1: Run a discriminating stale-generated-output refusal.** First render the canonical checked contract into `/tmp/rtd-v1-review-generated.py` and `/tmp/rtd-v1-review-generated.rs` and prove `--check` against those exact temporary outputs returns zero. Copy the contract to `/tmp/rtd-v1-review-contract.yaml`, make one valid whitespace-free enum mutation, then run `--check` against the same already-generated temporary outputs and require a nonzero stale result. This proves content drift rather than missing-output failure. Never mutate the checked-in contract.

- [ ] **Step 2: Run the full focused T1 verification serially.**

  Run: `uv run python tools/generate_rtd_v1_types.py --check`

  Run: `mise run test:q -- tests/unit/contracts/test_rtd_v1_codegen.py tests/unit/contracts/test_rtd_v1_validation.py tests/unit/contracts/test_rtd_v1_canonical.py tests/unit/contracts/test_rtd_v1_detroit_control.py`

  Run: `uv run python tools/build_detroit_rtd_control.py --check --verify-source-root /media/user/data/babylon-data/backups/data-artifacts-v7`

  Run: `cd rust && cargo fmt --all -- --check`

  Run: `cd rust && cargo test -p babylon-rtd --locked`

  Run: `cd rust && cargo test -p babylon-tick --test rtd_admin_fixture_identity --locked`

  Run: `cd rust && cargo clippy -p babylon-rtd --all-targets --locked -- -D warnings`

  Run: `cd rust && cargo clippy -p babylon-tick --test rtd_admin_fixture_identity --locked -- -D warnings`

  Run: `PYTHONPATH="$PWD/src" mise run check:vocabulary`

  Run: `mise run check:bsl-sentinels`

  Run: `mise run check`

- [ ] **Step 3: Inspect byte and boundary outcomes manually.** Confirm the Python and Rust tests both consume the same root vector file; the encoder mutation tests fail when either implementation’s ordering changes; the 67,108,865-byte test returns `RTD_CANONICAL_SIZE`; all twenty registered gaps are present exactly once; every H3 gap contains `PER-21`; every Census conflict gap contains `PER-28` and no Census facet exists; no fixture identity is H3; there is exactly one OMB-MSA gap and zero MSA membership rows; `border-synthesis-opt-in` is absent from default fixture bytes; the Rust tick recomputes the checked graph/world identities; and no player/fog/Archive action field is populated.

- [ ] **Step 4: Run prose lint and commit the review correction only if it changed source.**

  Run: `vale docs/superpowers/plans/2026-08-23-neel-t1-relational-territory-dossier.md`

  If Steps 1–3 required a source correction, commit it as `test(rtd): close T1 contract boundary review findings`; otherwise make no empty commit.

- [ ] **Step 5: Publish the PER-52 completion handoff.** Resolve the final T1
  branch SHA with `git rev-parse HEAD`, require the declared T1 surface to be
  clean, and post the review record described above as the PER-52 completion
  comment. Include every T1 implementation commit SHA and the final focused-gate
  results. Only after the comment exists, move PER-52 from In Progress to Done,
  refresh it, and require the returned state to be Done before T2 starts. Leave
  PER-53 blocked by both PER-52 and PER-21 until its separate activation work.

## Scope Cuts and Dependencies

- **PER-21:** T1 consumes neither H3 identity nor H3 vectors. The existing Rust `H3CellId` contract is a useful precedent but does not authorize RTD H3 output. Three H3-specific typed gaps and artifact provenance land inside the complete twenty-gap registry.
- **PER-22 / Gate 3:** no `CommittedTickEnvelope`, durable Archive outbox, fog-filtered dossier, client knowledge row, or player-facing receipt appears. The fixture’s `IN_MEMORY` status is deliberately administrative.
- **PER-23 / PER-24:** no county/place decision-to-consequence surface is claimed. `DecisionSurfaceV1` is present only as an empty-action administrative record shape.
- **PER-28:** no live proving basin, mutable reference-data production, or border synthesis lands. The fixture is a deterministic control artifact that asserts canonical LODES has no Canadian flow.
- **MSA artifact:** V1 does not add, derive, or infer OMB MSA evidence. It refuses the legacy code and carries one typed dependency gap.
- **Weighted overlap:** no centroid-to-area conversion or county/H3 membership is added. Any area measure needs its separately governed producer.
- **T2 and later:** ActionBudget, organization intent, practices, antagonist, decay, receipts, inventory, money, escrow, and membership growth do not enter this implementation.

## Self-Review

### Spec coverage

| Requirement | Plan coverage |
|---|---|
| Standalone closed V1 YAML contract | Task 1 |
| Architecture record for contract/codegen/crate ownership | Task 0 |
| Generated Rust and Python structural types | Task 1 |
| Independent canonical encoders and shared bytes/hash vectors | Task 4 |
| Closed enums, typed identities, exact sorting, null/status law | Tasks 1–4 |
| Bounds, checked counters, atomic staged encoding | Tasks 2–4 |
| Present zero distinct from missingness | Tasks 2 and 4 |
| No H3 before PER-21; typed gaps only | Tasks 2, 3, and 5 |
| MSA refusal and no legacy membership | Tasks 2, 3, and 5 |
| Detroit-Windsor administrative control and no Canadian canonical flow | Task 5 |
| Reproducible Detroit source rows and real graph/world identity | Task 5 |
| Complete twenty-gap missingness registry and policy-disabled Canada synthesis | Task 5 |
| No score/stage/curve/reduction and no player/fog/Archive claim | Global Constraints, Tasks 2 and 5, Scope Cuts |
| Focused reproducible gates and source-only review | Task 6 |

### Plan-completeness scan

The plan contains no unfinished task, unbounded work marker, or unspecified validation instruction. Every implementation task names its files, interfaces, failure cases, verification commands, and commit message. Deferred work is identified as a named external dependency rather than an implied implementation obligation.

### Type consistency

- Task 1 generates `RtdDossierDraftV1` and `RelationalTerritoryDossierV1`; Tasks 2–5 consume those exact names.
- Task 2’s Python `validate_draft`, `canonical_draft_bytes`, `projection_hash`, and `seal_draft` names are reused unchanged by Tasks 4–5.
- Task 3’s Rust `RtdError`, `validate_draft`, and `append_bounded` names are reused unchanged by Task 4.
- Task 4’s root JSONL corpus is the shared cross-language byte/hash witness; Task 5 independently rebuilds its fixture draft from the extraction ledger and verifies its graph/world identity through the real tick.
- The plan never exposes its administrative record through a player or committed-durability interface.

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-23-neel-t1-relational-territory-dossier.md`. The Director already authorized autonomous execution. Use `superpowers:subagent-driven-development`, review every task before the next commit, preserve the Linear preflight, and keep Cargo gates single-flight.
