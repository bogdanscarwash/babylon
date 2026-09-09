<!-- vale off -->

# PER-20 Tick Commit Claim V1 Implementation Plan

**Goal:** Land the smallest database-free PER-20 contract that binds a durable
campaign and tick to the existing `TickContentHashV1` and gives future
`tick_commit` retries exact, loud content-identity semantics.

**Architecture:** `babylon-persistence` owns only the durability wrapper.
`CampaignId` supplies UUID bytes and the kernel's owning type supplies the
tick-content digest. A fixed-size claim codec, shared YAML and JSONL contract,
Rust consumer, and independent Python verifier agree on the bytes. No database
or writer authority is added.

**Spec:**
`docs/superpowers/specs/2026-08-27-per-20-tick-commit-claim-design.md`

## Task 1: Ratify the boundary

- Create ADR242 with exact ownership, retry semantics, and exclusions.
- Add ADR242 to `ai/decisions/index.yaml`.
- Add a focused decision test that binds the ADR, index, design, and writer
  exclusion.
- Verify with the focused Pytest and Vale.

## Task 2: Write the shared contract RED

- Create `contracts/tick_commit_claim_v1.yaml` with fixed constants, exact
  layout, retry predicates, and exclusions.
- Create `contracts/tick_commit_claim_v1_vectors.jsonl` with semantic valid,
  mutation, retry, and refusal rows.
- Create `tools/verify_tick_commit_claim_v1.py` and its unit tests.
- First run must fail because the verifier or corpus implementation is absent.
- Implement only enough independent verification to make the focused tests
  pass. Keep all reads bounded and all error classes explicit.

## Task 3: Write the Rust claim RED

- Add `CampaignId::canonical_bytes` without adding campaign-to-replay
  conversion.
- Create `babylon-persistence::tick_commit_claim::TickCommitClaimV1` and typed
  retry results.
- Create a Rust contract test that consumes the shared vectors and calls the
  production type.
- First run must fail because the module and API are absent.
- Implement the fixed 93-byte composer and comparison rules. Add no decoder,
  SQL, migration, or writer-gate change.

## Task 4: Refactor and verify

- Run the Python verifier tests and decision test.
- Run the scoped `babylon-persistence` contract test and crate tests.
- Run Rust format and scoped Clippy with warnings denied.
- Run targeted Vale on the new Markdown files.
- Run the canonical non-documentation Rust gate if scoped checks are green.
- Inspect the final diff for placeholders, aliases, database I/O, migration
  changes, and writer activation.

## Success criteria

- Rust and Python independently reconstruct identical claim bytes.
- Campaign, tick, and content mutations each move the canonical claim.
- Exact claim retry is `Idempotent`.
- Same-key different-content retry is `ContentIdentityMismatch`.
- Different-key comparison is `KeyMismatch`.
- `TickContentHashV1` is imported from its owning kernel module and never
  aliased or re-exported.
- No SQL, migration, database connection, `CommittedTickEnvelope`, or writer
  authority is introduced.
