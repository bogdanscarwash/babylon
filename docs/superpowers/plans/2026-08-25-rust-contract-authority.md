<!-- vale off -->

# Rust Contract Authority Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retire the duplicated Python Neel gameplay-contract layer and make the existing Rust contract crates the sole executable authority.

**Architecture:** The practice, relational territory dossier, and T3 evidence contracts stay in their existing Rust crates. Versioned YAML, JSONL, and fixture files remain language-neutral behavioral contracts. Rust tests bind their bytes and behavior, while a Rust `bsl-lint` repository check prevents the retired Python authority paths from returning.

**Tech Stack:** Rust 2021 workspace, BSL, `babylon-kernel`, `babylon-practice-contract`, `babylon-rtd`, `babylon-evidence`, `bsl-lint`, Cargo tests, Clippy, Vale

**Spec:** `docs/superpowers/specs/2026-08-25-rust-contract-authority-design.md`

## Global Constraints

- Do not add a Radon exclusion.
- Do not add a mathematical primitive or activate Gate 5.
- Keep `babylon-kernel` theory-neutral.
- Keep `contracts/*.yaml`, `contracts/*.jsonl`, and fixed fixtures language-neutral.
- Preserve historical ADRs and completed plans unchanged.
- Show RED before each behavioral implementation.
- Run Rust gates separately; do not run documentation-generating umbrella tasks.

---

### Task 1: Practice contract source authority

**Files:**

- Move: `rust/crates/babylon-practice-contract/src/generated.rs` to `rust/crates/babylon-practice-contract/src/schema.rs`
- Move: `rust/crates/babylon-practice-contract/tests/generated_contract.rs` to `rust/crates/babylon-practice-contract/tests/schema_contract.rs`
- Modify: `rust/crates/babylon-practice-contract/src/lib.rs`
- Modify: `rust/crates/babylon-practice-contract/tests/schema_contract.rs`

**Interfaces:**

- Consumes: exact bytes from `contracts/practice_contract_v1.yaml`
- Produces: `PRACTICE_CONTRACT_SOURCE_SHA256: [u8; 32]` and `DEFAULT_PRACTICE_BUDGET_TERMS_V1: PracticeBudgetTermsV1`

- [ ] **Step 1: Write the failing source and defaults tests**

Add these tests to the renamed `schema_contract.rs`:

```rust
const PRACTICE_SCHEMA: &[u8] =
    include_bytes!("../../../../contracts/practice_contract_v1.yaml");

#[test]
fn language_neutral_schema_bytes_are_bound_to_rust() {
    assert_eq!(
        babylon_kernel::sha256_of(PRACTICE_SCHEMA),
        PRACTICE_CONTRACT_SOURCE_SHA256
    );
}

#[test]
fn designed_budget_defaults_are_typed_and_exact() {
    assert_eq!(
        DEFAULT_PRACTICE_BUDGET_TERMS_V1,
        PracticeBudgetTermsV1 {
            initial: 1,
            weekly_credit_cap: 1,
            storage_ceiling: 4,
            organize_cost: 1,
            agitate_cost: 1,
            mutual_aid_cost: 1,
        }
    );
    assert!(DEFAULT_PRACTICE_BUDGET_TERMS_V1.initial
        <= DEFAULT_PRACTICE_BUDGET_TERMS_V1.storage_ceiling);
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
cd rust
cargo test -p babylon-practice-contract --test schema_contract --locked
```

Expected: compile failure because both public constants do not exist.

- [ ] **Step 3: Make the schema module and constants Rust-owned**

In `lib.rs`, replace `mod generated;` and its re-export with `mod schema;` and `pub use schema::*;`. Add:

```rust
pub const PRACTICE_CONTRACT_SOURCE_SHA256: [u8; 32] = [
    0xe9, 0xed, 0x6d, 0xba, 0xf0, 0x1f, 0x89, 0xf1,
    0x29, 0x4f, 0x2e, 0x6d, 0x28, 0x94, 0x6e, 0x73,
    0xb0, 0x5d, 0x9a, 0x4d, 0x75, 0x47, 0x2d, 0x5b,
    0x2d, 0xd3, 0x52, 0x35, 0x0d, 0x33, 0x2f, 0x79,
];

pub const DEFAULT_PRACTICE_BUDGET_TERMS_V1: PracticeBudgetTermsV1 =
    PracticeBudgetTermsV1 {
        initial: 1,
        weekly_credit_cap: 1,
        storage_ceiling: 4,
        organize_cost: 1,
        agitate_cost: 1,
        mutual_aid_cost: 1,
    };
```

Replace generated-code claims in `schema.rs` and the test names with Rust-schema wording. Do not change any discriminant, field order, limit, or codec behavior.

- [ ] **Step 4: Run the crate gate and verify GREEN**

Run:

```bash
cd rust
cargo fmt --all -- --check
cargo test -p babylon-practice-contract --locked
cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings
```

Expected: all commands pass.

- [ ] **Step 5: Commit the practice boundary**

```bash
git add rust/crates/babylon-practice-contract
mise run commit -- "refactor(contract): make practice schema Rust-owned"
```

---

### Task 2: RTD source and administrative fixture authority

**Files:**

- Move: `rust/crates/babylon-rtd/src/generated.rs` to `rust/crates/babylon-rtd/src/schema.rs`
- Move: `rust/crates/babylon-rtd/tests/generated_contract.rs` to `rust/crates/babylon-rtd/tests/schema_contract.rs`
- Create: `rust/crates/babylon-rtd/tests/administrative_fixture.rs`
- Modify: `rust/crates/babylon-rtd/src/lib.rs`
- Modify: `rust/crates/babylon-rtd/tests/schema_contract.rs`

**Interfaces:**

- Consumes: RTD YAML, JSONL vectors, Detroit-Windsor control JSON, and extraction ledger
- Produces: `RTD_CONTRACT_SOURCE_SHA256: [u8; 32]` and a Rust-only fixture identity contract

- [ ] **Step 1: Write the failing schema and fixture tests**

Add the source-byte test to `schema_contract.rs`:

```rust
const RTD_SCHEMA: &[u8] =
    include_bytes!("../../../../contracts/relational_territory_dossier_v1.yaml");

#[test]
fn language_neutral_schema_bytes_are_bound_to_rust() {
    assert_eq!(
        babylon_kernel::sha256_of(RTD_SCHEMA),
        RTD_CONTRACT_SOURCE_SHA256
    );
}
```

Create `administrative_fixture.rs` with a test that:

```rust
use babylon_rtd::{
    canonical_draft_bytes, parse_draft_json, parse_vector_corpus,
    projection_hash, RtdVectorCaseV1,
};

const CONTROL: &[u8] = include_bytes!(
    "../../../../contracts/fixtures/detroit_windsor_rtd_v1_admin_control.json"
);
const VECTORS: &[u8] =
    include_bytes!("../../../../contracts/relational_territory_dossier_v1_vectors.jsonl");

#[test]
fn administrative_control_is_the_shared_rust_vector() {
    let control = parse_draft_json(CONTROL).expect("closed control draft");
    let control_bytes = canonical_draft_bytes(&control).expect("canonical control bytes");
    let cases = parse_vector_corpus(VECTORS).expect("closed RTD vector corpus");
    let mut vector_bytes = None;
    for index in 0..256 {
        if index == cases.len() {
            break;
        }
        if let RtdVectorCaseV1::Valid { case_id, draft_json, .. } = &cases[index] {
            if case_id == "detroit-windsor-admin-control" {
                vector_bytes = Some(
                    canonical_draft_bytes(
                        &parse_draft_json(draft_json).expect("vector control draft"),
                    )
                    .expect("vector canonical bytes"),
                );
                break;
            }
        }
    }
    let vector_bytes = vector_bytes.expect("Detroit-Windsor control vector");
    assert_eq!(control_bytes, vector_bytes);
    assert_eq!(projection_hash(&control).expect("control hash").len(), 32);
}
```

- [ ] **Step 2: Run the tests and verify RED**

```bash
cd rust
cargo test -p babylon-rtd --test schema_contract --test administrative_fixture --locked
```

Expected: compile failure because `RTD_CONTRACT_SOURCE_SHA256` does not exist.

- [ ] **Step 3: Make the schema module and source identity Rust-owned**

Replace `mod generated;` with `mod schema;`, keep `pub use schema::*;`, and add:

```rust
pub const RTD_CONTRACT_SOURCE_SHA256: [u8; 32] = [
    0x5f, 0x0e, 0x27, 0x1d, 0x46, 0x78, 0x3b, 0xd8,
    0x2f, 0xb5, 0xc9, 0x33, 0x6c, 0x46, 0x6f, 0x4c,
    0x36, 0x31, 0xa4, 0x99, 0xb4, 0x3c, 0x83, 0xc1,
    0x1b, 0x85, 0x4d, 0xb2, 0x3e, 0xa5, 0x9e, 0x40,
];
```

Replace generated-code claims in `schema.rs` and test names. Keep every closed registry row unchanged.

- [ ] **Step 4: Add extraction-ledger immutability**

Extend `administrative_fixture.rs` with:

```rust
const EXTRACTION_LEDGER: &[u8] = include_bytes!(
    "../../../../contracts/fixtures/detroit_windsor_rtd_v1_extraction.yaml"
);
const EXTRACTION_LEDGER_SHA256: [u8; 32] = [
    0x89, 0x40, 0x61, 0x47, 0x6e, 0x2a, 0x82, 0xa9,
    0x0c, 0x78, 0xf1, 0x47, 0xde, 0xba, 0x0b, 0x4f,
    0x26, 0xb1, 0x8d, 0x72, 0x7c, 0xb6, 0xbf, 0x38,
    0x0c, 0xcb, 0xa0, 0x8c, 0xf2, 0xed, 0x33, 0x9c,
];

#[test]
fn extraction_ledger_bytes_are_pinned() {
    assert_eq!(
        babylon_kernel::sha256_of(EXTRACTION_LEDGER),
        EXTRACTION_LEDGER_SHA256
    );
}
```

The literal comes from:

```bash
sha256sum contracts/fixtures/detroit_windsor_rtd_v1_extraction.yaml
```

- [ ] **Step 5: Run the RTD gate and verify GREEN**

```bash
cd rust
cargo fmt --all -- --check
cargo test -p babylon-rtd --locked
cargo clippy -p babylon-rtd --all-targets --locked -- -D warnings
```

Expected: all commands pass.

- [ ] **Step 6: Commit the RTD boundary**

```bash
git add rust/crates/babylon-rtd
mise run commit -- "refactor(rtd): make dossier schema Rust-owned"
```

---

### Task 3: Rust repository authority check

**Files:**

- Create: `rust/crates/bsl-lint/src/rust_contract_authority.rs`
- Create: `rust/crates/bsl-lint/tests/rust_contract_authority.rs`
- Modify: `rust/crates/bsl-lint/src/main.rs`
- Modify: `rust/crates/bsl-lint/Cargo.toml`
- Modify: `.mise.toml`

**Interfaces:**

- Consumes: zero or one repository root
- Produces: `bsl-lint rust-contract-authority [ROOT]`, with exit 1 for any retired authority path

- [ ] **Step 1: Write failing check tests**

The integration test must create a bounded scratch root and write one retired path. It must assert that the report contains these exact phrases:

```text
E-SENTINEL rust-contract-authority
retired Python gameplay authority exists
executable authority belongs to the Rust contract crates
```

It must also assert that an empty scratch root and the real repository both exit 0 after Task 4.

- [ ] **Step 2: Run the test and verify RED**

```bash
cd rust
cargo test -p bsl-lint --test rust_contract_authority --locked
```

Expected: compile or dispatch failure because the check is not registered.

- [ ] **Step 3: Implement the bounded path check**

Use this exact closed list:

```rust
const RETIRED_PATHS: [&str; 8] = [
    "src/babylon/contracts/practice_contract_v1.py",
    "src/babylon/contracts/practice_contract_v1_generated.py",
    "src/babylon/contracts/relational_territory_dossier_v1.py",
    "src/babylon/contracts/rtd_v1_generated.py",
    "tools/generate_practice_contract_types.py",
    "tools/generate_rtd_v1_types.py",
    "tools/build_detroit_rtd_control.py",
    "tools/sfs_contract_vectors.py",
];
```

Accept at most one root. Iterate exactly `0..RETIRED_PATHS.len()`. Return one `Severity::Fail` finding for each existing file. Register the check in `CHECKS`, update the crate description from three checks to four checks, and update `.mise.toml` task descriptions that enumerate the registry.

- [ ] **Step 4: Run the focused test and verify the injected defect is detected**

```bash
cd rust
cargo test -p bsl-lint --test rust_contract_authority --locked
```

Expected before Task 4 deletion: the scratch mutation tests pass and the real-estate assertion fails on all eight current paths.

---

### Task 4: Retire the Python gameplay authority

**Files:**

- Delete: `src/babylon/contracts/practice_contract_v1.py`
- Delete: `src/babylon/contracts/practice_contract_v1_generated.py`
- Delete: `src/babylon/contracts/relational_territory_dossier_v1.py`
- Delete: `src/babylon/contracts/rtd_v1_generated.py`
- Delete: `tools/generate_practice_contract_types.py`
- Delete: `tools/generate_rtd_v1_types.py`
- Delete: `tools/build_detroit_rtd_control.py`
- Delete: `tools/sfs_contract_vectors.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_admission.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_budget.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_codec.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_codegen.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_refusals.py`
- Delete: `tests/unit/contracts/test_practice_contract_v1_topology.py`
- Delete: `tests/unit/contracts/test_rtd_v1_canonical.py`
- Delete: `tests/unit/contracts/test_rtd_v1_codegen.py`
- Delete: `tests/unit/contracts/test_rtd_v1_detroit_control.py`
- Delete: `tests/unit/contracts/test_rtd_v1_validation.py`
- Delete: `tests/unit/tools/test_sfs_contract_vectors.py`
- Delete: `tests/unit/config/test_practice_budget_defines.py`
- Modify: `src/babylon/config/defines/organizations.py`
- Modify: `src/babylon/config/defines/_assembler.py`
- Modify: `src/babylon/data/defines.yaml`
- Modify: `tests/unit/test_public_import_surface.py`
- Modify: `pyproject.toml`
- Modify: `uv.lock`
- Move: `rust/crates/babylon-evidence/tests/cross_language_vectors.rs` to `rust/crates/babylon-evidence/tests/contract_vectors.rs`

**Interfaces:**

- Consumes: Rust replacements from Tasks 1 through 3
- Produces: no production Python implementation of the Neel gameplay contracts

- [ ] **Step 1: Remove the duplicate Python files and tests**

Delete every path listed above. Rename the evidence vector test and change its module comment from Python-authored parity to language-neutral contract vectors.

- [ ] **Step 2: Remove the duplicate Python budget mirror**

Remove `PracticeBudgetDefines`, `_U32_MAX`, its `_assembler.py` import, field, loader call, and doc-list entry. Remove the `practice_budget:` block from `src/babylon/data/defines.yaml`. Remove the `PracticeBudgetDefines` assertion from `tests/unit/test_public_import_surface.py` because the type no longer exists.

- [ ] **Step 3: Remove the tool-only Python dependency**

Delete `unicodedata2==17.0.1` from `pyproject.toml`, then run:

```bash
uv lock
uv lock --check
```

Verify that `uv.lock` contains no `unicodedata2` package or dependency row.

- [ ] **Step 4: Verify absence before GREEN**

```bash
rg -n 'practice_contract_v1_generated|rtd_v1_generated|generate_practice_contract_types|generate_rtd_v1_types|build_detroit_rtd_control|sfs_contract_vectors|PracticeBudgetDefines|unicodedata2' src tools tests pyproject.toml uv.lock
```

Expected: no matches outside immutable historical documents, which this scoped command excludes.

- [ ] **Step 5: Run the Rust authority and contract gates**

```bash
cd rust
cargo fmt --all -- --check
cargo test -p babylon-practice-contract --locked
cargo test -p babylon-rtd --locked
cargo test -p babylon-evidence --locked
cargo test -p bsl-lint --locked
cargo clippy -p babylon-practice-contract --all-targets --locked -- -D warnings
cargo clippy -p babylon-rtd --all-targets --locked -- -D warnings
cargo clippy -p babylon-evidence --all-targets --locked -- -D warnings
cargo clippy -p bsl-lint --all-targets --locked -- -D warnings
cargo run -p bsl-lint --locked -- all
```

Expected: every command passes, including the real-estate authority check.

- [ ] **Step 6: Run the focused Python periphery gates**

```bash
mise run test:q -- tests/unit/config tests/unit/test_public_import_surface.py tests/unit/tools/test_repo_hygiene.py
uv run ruff check src/babylon/config tests/unit/config tests/unit/test_public_import_surface.py
uv run ruff format --check src/babylon/config tests/unit/config tests/unit/test_public_import_surface.py
uv run mypy src/babylon/config
```

Expected: all commands pass without the retired contract layer.

- [ ] **Step 7: Commit the retirement**

```bash
git add src tools tests pyproject.toml uv.lock rust/crates/babylon-evidence rust/crates/bsl-lint .mise.toml
mise run commit -- "refactor(engine): retire Python gameplay contracts"
```

---

### Task 5: Record authority and run the landing gates

**Files:**

- Create: `ai/decisions/ADR229_rust_contract_authority.yaml`
- Modify: `ai/decisions/index.yaml`

**Interfaces:**

- Consumes: verified cutover evidence from Tasks 1 through 4
- Produces: accepted ADR229 and a landing-ready branch

- [ ] **Step 1: Run deterministic gameplay gates**

```bash
mise run qa:regression
mise run qa:vault-regression-ci
mise run check:gate-coverage
```

Expected: unchanged deterministic baselines and a clean declared gate-coverage estate.

- [ ] **Step 2: Add ADR229**

Record these exact rulings:

```text
R1 Rust contract crates are sole executable authority for practice, RTD, and T3 evidence.
R2 YAML, JSONL, and fixtures are language-neutral behavioral contracts, not Python runtime authority.
R3 Semantic BSL invariants belong in language refusals; repository relationships belong in Rust bsl-lint.
R4 The Python gameplay contract modules, tools, tests, and duplicate budget defines retire without exemptions.
R5 Historical ADR226, ADR227, and ADR228 remain immutable evidence and are superseded only on runtime ownership.
```

Include the exact verification commands and results from Tasks 1 through 5. Add the ADR to `ai/decisions/index.yaml` in the established format.

- [ ] **Step 3: Run prose and governance checks**

```bash
vale docs/superpowers/specs/2026-08-25-rust-contract-authority-design.md
uv run yamllint ai/decisions/ADR229_rust_contract_authority.yaml ai/decisions/index.yaml
mise run check:bsl-sentinels
```

Expected: zero Vale errors or warnings, valid YAML, and all Rust relationship checks green.

- [ ] **Step 4: Commit the ADR evidence**

```bash
git add ai/decisions/ADR229_rust_contract_authority.yaml ai/decisions/index.yaml
mise run commit -- "docs(architecture): record Rust contract cutover"
```

- [ ] **Step 5: Run the pre-push gate and prove the original defect is gone**

```bash
SKIP=rust-full-gate git push -u origin feature/neel-relational-practice-circuit
```

Expected: Radon reports no new rank below B, every pre-push hook passes, and the branch reaches origin.

- [ ] **Step 6: Open and land the PR through the governed workflow**

Create a PR to `dev`, wait for all checks at the exact head SHA, address every Copilot comment, and merge only with:

```bash
babylon_pr_number="$(gh pr view --json number --jq .number)"
mise run pr:merge -- "$babylon_pr_number"
```

Do not delete `feature/neel-relational-practice-circuit` after merge.

<!-- vale on -->
