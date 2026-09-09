<!-- vale ste.UnapprovedWords = NO -->
<!-- vale Vale.Spelling = NO -->
<!-- vale ste.ProcedureLength = NO -->
<!-- The mandated plan template, exact API names, commands, and test prose conflict with
     the following heuristic plain-language rules. Keep their exemptions local to this file. -->
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

# Rust PostgreSQL Contract Keel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the dependency-light `babylon-persistence` contract crate with nominal database identities, honest hash names, the frozen legacy migration digests, and a typed failure taxonomy, without opening PostgreSQL or changing the live Python writer.

**Architecture:** The crate sits downstream of `babylon-kernel` and owns persistence-only types. It reuses `SessionId`, `ContentDigest`, and `sha256_of`, wraps `uuid::Uuid` for storage identity, and hashes ordered UTF-8 chunks with one NUL byte after every chunk. A small Python build tool exports the two frozen legacy byte sequences as test fixtures; Rust verifies them without importing Python or connecting to a database.

**Tech Stack:** Rust 2021, MSRV 1.87, `babylon-kernel`, `uuid` 1.24, SHA-256, Python 3.12 standard library for fixture generation, Cargo, pytest, Mise, Vale.

**Spec:** `docs/superpowers/specs/2026-08-22-rust-postgresql-persistence-boundary-design.md`

## Global Constraints

- The current Python writer remains the only live PostgreSQL writer and migrator throughout this plan.
- The new crate has no `postgres`, `tokio-postgres`, `tokio`, `h3o`, `babylon-graph`, or `babylon-tick` dependency.
- `babylon-tick` never depends on `babylon-persistence`.
- Reuse `babylon_kernel::SessionId`, `ContentDigest`, and `sha256_of`; do not duplicate them.
- `CampaignId(uuid::Uuid)` is storage identity only and never enters `seed_for` or engine physics.
- Keep `ReplayIdentityHash`, `GraphStateHash`, `TickContentHash`, `RefDigest`, and `MigrationSetDigest` as distinct nominal types.
- Do not construct `TickContentHash` from the current graph-only `state_hash`; the full P27 encoder remains a writer-cutover gate.
- Legacy framing is exact: SHA-256 over each ordered UTF-8 chunk followed by one `0x00` byte.
- The shared advisory-lock key is decimal `3132163383`, hexadecimal `0xBAB10537`.
- At pinned source `dev@ae5b2615`, `POSTGRES_SCHEMA_DDL` has 112 chunks and digest `0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b`.
- At pinned source `dev@ae5b2615`, migrations `0010` through `0044` have 35 chunks and digest `4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db`.
- Bound manifest input at 256 chunks and 1,048,576 framed bytes. Every source loop uses that explicit bound.
- Explicit campaign RNG-seed integration is outside this keel because changing `seed_for` changes stochastic streams. It requires its own typed-motion plan, golden vectors, and any resulting baseline ceremony before checkpoint support can claim replay completeness.
- H3 validation and the checked positive-`BIGINT` codec are PER-21's next independently testable plan; this keel adds no H3 library.

## Parallel-Safe Ownership

- Rust contract lane owns `rust/Cargo.toml`, `rust/Cargo.lock`, and `rust/crates/babylon-persistence/**` except generated fixture bytes.
- Legacy fixture lane owns `tools/export_legacy_postgres_contract.py`, `tests/unit/persistence/test_rust_legacy_contract_fixtures.py`, and `rust/crates/babylon-persistence/tests/fixtures/*.bin`.
- The integration lane owns `.mise.toml`, `ai/state.yaml`, Linear reconciliation, commits, and all heavy gates.
- Only the integration lane runs `mise run rust:lock-refresh`, `mise run rust:check`, or a full Python gate. Scoped unit tests may run after the owning lane reports that it is idle.

---

### Task 1: Persistence identity, hash, and failure contracts

**Files:**

- Modify: `rust/Cargo.toml`
- Modify mechanically: `rust/Cargo.lock`
- Create: `rust/crates/babylon-persistence/Cargo.toml`
- Create: `rust/crates/babylon-persistence/src/lib.rs`
- Create: `rust/crates/babylon-persistence/src/identity.rs`
- Create: `rust/crates/babylon-persistence/src/hashes.rs`
- Create: `rust/crates/babylon-persistence/src/error.rs`
- Test: `rust/crates/babylon-persistence/tests/contract_keel.rs`

**Interfaces:**

- Consumes: `babylon_kernel::{seed_for, ContentDigest, SessionId}` and `uuid::Uuid`.
- Produces: `CampaignId::from_uuid(Uuid)`, `CampaignId::as_uuid()`, the five 32-byte digest newtypes with `from_bytes`, `as_bytes`, and `to_hex`, and `PersistenceError::{connection,migration,serialization,constraint,commit}`.

- [ ] **Step 1: Add the workspace member and a deliberately empty crate root**

Add `"crates/babylon-persistence"` immediately after `"crates/babylon-tick"` in `rust/Cargo.toml`. Create this manifest:

```toml
[package]
name = "babylon-persistence"
description = "Rust-owned PostgreSQL persistence contracts and adapters"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish = false

[dependencies]
babylon-kernel = { path = "../babylon-kernel" }
uuid = { version = "1.24", default-features = false }
```

Create `src/lib.rs` with only the crate documentation and lint attributes so the public-contract test compiles far enough to fail on missing exports:

```rust
//! Rust-owned PostgreSQL persistence contracts and adapters.
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
```

Refresh the lockfile before the red run so the failure names the missing API rather than an outdated lock:

```bash
mise run rust:lock-refresh
```

- [ ] **Step 2: Write the failing public-contract test**

Create `tests/contract_keel.rs`:

```rust
//! Public contracts that prevent persistence identity and hash-name collapse.

use babylon_kernel::{seed_for, ContentDigest, SessionId};
use babylon_persistence::{
    CampaignId, GraphStateHash, PersistenceError, PersistenceFailureKind, RefDigest,
    ReplayIdentityHash, TickContentHash,
};
use std::any::TypeId;
use uuid::Uuid;

#[test]
fn campaign_uuid_is_not_an_rng_input() {
    let first = CampaignId::from_uuid(Uuid::from_u128(1));
    let second = CampaignId::from_uuid(Uuid::from_u128(2));
    let session = SessionId::new("contract-keel").expect("literal is non-empty");
    assert_ne!(first, second);
    assert_eq!(
        seed_for(&session, 7, "contract", "carrier"),
        seed_for(&session, 7, "contract", "carrier")
    );
    assert_eq!(first.as_uuid(), &Uuid::from_u128(1));
}

#[test]
fn honest_hashes_are_nominally_distinct() {
    assert_ne!(TypeId::of::<ReplayIdentityHash>(), TypeId::of::<GraphStateHash>());
    assert_ne!(TypeId::of::<GraphStateHash>(), TypeId::of::<TickContentHash>());
    assert_ne!(TypeId::of::<TickContentHash>(), TypeId::of::<RefDigest>());
    let bytes = [0x07; 32];
    assert_eq!(GraphStateHash::from_bytes(bytes).as_bytes(), &bytes);
    assert_eq!(RefDigest::from_bytes(bytes).to_hex(), "07".repeat(32));
}

#[test]
fn content_digest_remains_the_kernel_pair() {
    let digest = ContentDigest {
        defines_hash: [1; 32],
        rules_hash: [2; 32],
    };
    assert_ne!(digest.defines_hash, digest.rules_hash);
}

#[test]
fn failures_keep_five_distinct_stages() {
    let cases = [
        (PersistenceError::connection("connect"), PersistenceFailureKind::Connection),
        (PersistenceError::migration("adopt"), PersistenceFailureKind::Migration),
        (PersistenceError::serialization("encode"), PersistenceFailureKind::Serialization),
        (PersistenceError::constraint("foreign key"), PersistenceFailureKind::Constraint),
        (PersistenceError::commit("commit"), PersistenceFailureKind::Commit),
    ];
    for (error, expected) in cases {
        assert_eq!(error.kind(), expected);
        assert!(!error.to_string().is_empty());
    }
}
```

- [ ] **Step 3: Run the test and verify the red phase**

Run from `rust/`:

```bash
cargo test -p babylon-persistence --test contract_keel --locked
```

Expected: compilation fails with unresolved imports from `babylon_persistence`.

- [ ] **Step 4: Implement the nominal types and error taxonomy**

Implement `CampaignId` as a private-field newtype around `Uuid`. In `hashes.rs`, use a local `digest_type!` macro to generate five private-field `[u8; 32]` newtypes without a cross-type conversion. `to_hex` uses a bounded 32-byte iterator and lowercase two-digit formatting. Implement `PersistenceError` as five data-carrying variants, with no generic fallback variant, and expose its stage through `kind()`.

`identity.rs`:

```rust
//! Durable database identities that never enter deterministic engine physics.

use uuid::Uuid;

/// Durable PostgreSQL campaign key, distinct from the deterministic session namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CampaignId(Uuid);

impl CampaignId {
    /// Wrap an already-minted UUID as a campaign storage identity.
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Return the UUID used by PostgreSQL foreign keys and partitions.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}
```

`hashes.rs`:

```rust
//! Honest, nominal names for persistence-layer SHA-256 values.

macro_rules! digest_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name([u8; 32]);

        impl $name {
            /// Wrap one already-computed SHA-256 value.
            #[must_use]
            pub fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            /// Return the canonical 32 bytes.
            #[must_use]
            pub fn as_bytes(&self) -> &[u8; 32] {
                &self.0
            }

            /// Render lowercase, two-digit hexadecimal without a prefix.
            #[must_use]
            pub fn to_hex(&self) -> String {
                use std::fmt::Write as _;
                self.0.iter().take(32).fold(
                    String::with_capacity(64),
                    |mut output, byte| {
                        let _ = write!(output, "{byte:02x}");
                        output
                    },
                )
            }
        }
    };
}

digest_type!(
    /// Legacy replay-lineage and idempotency stamp; it does not prove state equality.
    ReplayIdentityHash
);
digest_type!(
    /// Diagnostic hash over the canonical graph state only.
    GraphStateHash
);
digest_type!(
    /// Complete constitutional tick, seed, state, and ordered-action hash.
    TickContentHash
);
digest_type!(
    /// Identity of one immutable reference-data cohort.
    RefDigest
);
digest_type!(
    /// Ordered-NUL SHA-256 identity of one migration set.
    MigrationSetDigest
);
```

`error.rs`:

```rust
//! Failure categories at the engine-to-PostgreSQL boundary.

/// The five failure stages callers must handle separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceFailureKind {
    /// A bounded connection operation failed.
    Connection,
    /// Schema adoption or migration failed.
    Migration,
    /// A typed row or envelope could not be serialized.
    Serialization,
    /// PostgreSQL rejected a declared invariant.
    Constraint,
    /// The final transaction commit failed.
    Commit,
}

/// One persistence failure with no generic catch-all variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    /// Connection-stage detail.
    Connection(Box<str>),
    /// Migration-stage detail.
    Migration(Box<str>),
    /// Serialization-stage detail.
    Serialization(Box<str>),
    /// Constraint-stage detail.
    Constraint(Box<str>),
    /// Commit-stage detail.
    Commit(Box<str>),
}

impl PersistenceError {
    /// Construct a connection failure.
    pub fn connection(detail: impl Into<Box<str>>) -> Self {
        Self::Connection(detail.into())
    }

    /// Construct a migration failure.
    pub fn migration(detail: impl Into<Box<str>>) -> Self {
        Self::Migration(detail.into())
    }

    /// Construct a serialization failure.
    pub fn serialization(detail: impl Into<Box<str>>) -> Self {
        Self::Serialization(detail.into())
    }

    /// Construct a constraint failure.
    pub fn constraint(detail: impl Into<Box<str>>) -> Self {
        Self::Constraint(detail.into())
    }

    /// Construct a commit failure.
    pub fn commit(detail: impl Into<Box<str>>) -> Self {
        Self::Commit(detail.into())
    }

    /// Return the stage without discarding its detail.
    #[must_use]
    pub fn kind(&self) -> PersistenceFailureKind {
        match self {
            Self::Connection(_) => PersistenceFailureKind::Connection,
            Self::Migration(_) => PersistenceFailureKind::Migration,
            Self::Serialization(_) => PersistenceFailureKind::Serialization,
            Self::Constraint(_) => PersistenceFailureKind::Constraint,
            Self::Commit(_) => PersistenceFailureKind::Commit,
        }
    }

    fn detail(&self) -> &str {
        match self {
            Self::Connection(detail)
            | Self::Migration(detail)
            | Self::Serialization(detail)
            | Self::Constraint(detail)
            | Self::Commit(detail) => detail,
        }
    }
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?} failure: {}", self.kind(), self.detail())
    }
}

impl std::error::Error for PersistenceError {}
```

The crate root must export only these public names:

```rust
pub mod error;
pub mod hashes;
pub mod identity;

pub use error::{PersistenceError, PersistenceFailureKind};
pub use hashes::{
    GraphStateHash, MigrationSetDigest, RefDigest, ReplayIdentityHash, TickContentHash,
};
pub use identity::CampaignId;
```

- [ ] **Step 5: Run the scoped green gates**

Run from the repository root, single-flight:

```bash
cd rust
cargo test -p babylon-persistence --test contract_keel --locked
cargo test -p babylon-persistence --lib --locked
cargo clippy -p babylon-persistence --all-targets --locked -- -D warnings -D clippy::pedantic
RUSTDOCFLAGS='-D warnings' cargo doc -p babylon-persistence --no-deps --locked
```

Expected: every command exits 0; no warning suppression is added.

- [ ] **Step 6: Commit the identity/hash unit**

Stage only the workspace, lockfile, and new crate files. Commit with:

```text
feat(persistence): add Rust contract keel
```

### Task 2: Ordered-NUL migration manifest

**Files:**

- Create: `rust/crates/babylon-persistence/src/migration_manifest.rs`
- Modify: `rust/crates/babylon-persistence/src/lib.rs`
- Test: `rust/crates/babylon-persistence/tests/migration_manifest.rs`

**Interfaces:**

- Consumes: `babylon_kernel::sha256_of` and `MigrationSetDigest::from_bytes([u8; 32])`.
- Produces: `MigrationManifest::from_chunks(&'static str, &[&[u8]])`, `MigrationManifest::from_nul_framed(&'static str, &[u8])`, `name`, `chunk_count`, `digest`, `SCHEMA_ADVISORY_LOCK_KEY`, `MAX_MANIFEST_CHUNKS`, and `MAX_MANIFEST_BYTES`.

- [ ] **Step 1: Write the failing framing tests**

Create `tests/migration_manifest.rs` with these exact behavioral cases:

```rust
//! Language-neutral legacy migration framing contracts.

use babylon_persistence::{
    ManifestError, MigrationManifest, MAX_MANIFEST_BYTES, MAX_MANIFEST_CHUNKS,
    SCHEMA_ADVISORY_LOCK_KEY,
};

#[test]
fn ordered_chunks_hash_with_one_trailing_nul_each() {
    let manifest = MigrationManifest::from_chunks(
        "small",
        &[b"a".as_slice(), b"bc".as_slice()],
    )
        .expect("two non-empty chunks are valid");
    assert_eq!(manifest.chunk_count(), 2);
    assert_eq!(
        manifest.digest().to_hex(),
        "aa795aa4bbb6117911ef062e271bcb05ccfd58ea439da7d46a44e3a3fcefa790"
    );
}

#[test]
fn framing_is_order_and_boundary_sensitive() {
    let left = MigrationManifest::from_chunks(
        "left",
        &[b"a".as_slice(), b"bc".as_slice()],
    )
    .unwrap();
    let right = MigrationManifest::from_chunks(
        "right",
        &[b"ab".as_slice(), b"c".as_slice()],
    )
    .unwrap();
    assert_ne!(left.digest(), right.digest());
}

#[test]
fn nul_framed_bytes_parse_to_the_same_manifest() {
    let parsed = MigrationManifest::from_nul_framed("small", b"a\0bc\0").unwrap();
    let direct = MigrationManifest::from_chunks(
        "small",
        &[b"a".as_slice(), b"bc".as_slice()],
    )
    .unwrap();
    assert_eq!(parsed, direct);
}

#[test]
fn malformed_or_unbounded_inputs_fail_loudly() {
    assert_eq!(MigrationManifest::from_chunks("", &[b"a"]), Err(ManifestError::EmptyName));
    assert_eq!(MigrationManifest::from_chunks("empty", &[]), Err(ManifestError::EmptySet));
    assert_eq!(
        MigrationManifest::from_nul_framed("unterminated", b"a"),
        Err(ManifestError::MissingTrailingNul)
    );
    let too_many = vec![b"a".as_slice(); MAX_MANIFEST_CHUNKS + 1];
    assert!(matches!(
        MigrationManifest::from_chunks("many", &too_many),
        Err(ManifestError::TooManyChunks { .. })
    ));
    let too_large = vec![b'a'; MAX_MANIFEST_BYTES + 1];
    assert!(matches!(
        MigrationManifest::from_nul_framed("large", &too_large),
        Err(ManifestError::TooManyBytes { .. })
    ));
}

#[test]
fn the_cross_language_lock_key_is_pinned() {
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 0xBAB1_0537_i64);
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 3_132_163_383_i64);
}
```

- [ ] **Step 2: Run the red test**

Run from `rust/`:

```bash
cargo test -p babylon-persistence --test migration_manifest --locked
```

Expected: compilation fails because the manifest API is absent.

- [ ] **Step 3: Implement bounded parsing and hashing**

Implement `ManifestError` with the exact variants exercised above plus `EmptyChunk { index: usize }`. Reject the name, byte count, chunk count, missing final NUL, and empty interior chunks before hashing. Bound collection with `.take(MAX_MANIFEST_CHUNKS + 1)` and bound both byte-copy loops with `.take(MAX_MANIFEST_CHUNKS)`. Build one buffer containing `chunk` then `0x00` for each chunk and pass it to `sha256_of`.

Create `migration_manifest.rs`:

```rust
//! Bounded, language-neutral migration-set framing.

use babylon_kernel::sha256_of;

use crate::MigrationSetDigest;

/// Shared PostgreSQL session advisory-lock key inherited from the Python writer.
pub const SCHEMA_ADVISORY_LOCK_KEY: i64 = 0xBAB1_0537;
/// Hard ceiling on chunks in one manifest.
pub const MAX_MANIFEST_CHUNKS: usize = 256;
/// Hard ceiling on the full NUL-framed byte sequence.
pub const MAX_MANIFEST_BYTES: usize = 1_048_576;

/// Invalid migration manifest input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The stable manifest name was empty.
    EmptyName,
    /// The manifest carried no chunks.
    EmptySet,
    /// The manifest exceeded its fixed chunk ceiling.
    TooManyChunks { actual: usize, max: usize },
    /// The framed representation exceeded its fixed byte ceiling.
    TooManyBytes { actual: usize, max: usize },
    /// One source chunk was empty.
    EmptyChunk { index: usize },
    /// One source chunk contained the framing delimiter.
    EmbeddedNul { index: usize },
    /// A serialized manifest omitted the final delimiter.
    MissingTrailingNul,
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid migration manifest: {self:?}")
    }
}

impl std::error::Error for ManifestError {}

/// Validated identity of one ordered migration sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationManifest {
    name: &'static str,
    chunk_count: usize,
    digest: MigrationSetDigest,
}

impl MigrationManifest {
    /// Validate and hash ordered raw chunks.
    ///
    /// # Errors
    /// Returns [`ManifestError`] for an empty, malformed, or unbounded set.
    pub fn from_chunks(name: &'static str, chunks: &[&[u8]]) -> Result<Self, ManifestError> {
        validate_header(name, chunks.len())?;
        let framed_len = framed_len(chunks)?;
        let mut framed = Vec::with_capacity(framed_len);
        for chunk in chunks.iter().take(MAX_MANIFEST_CHUNKS) {
            framed.extend_from_slice(chunk);
            framed.push(0);
        }
        Ok(Self {
            name,
            chunk_count: chunks.len(),
            digest: MigrationSetDigest::from_bytes(sha256_of(&framed)),
        })
    }

    /// Parse a sequence in which every chunk, including the last, ends in NUL.
    ///
    /// # Errors
    /// Returns [`ManifestError`] for missing framing or an unbounded set.
    pub fn from_nul_framed(name: &'static str, bytes: &[u8]) -> Result<Self, ManifestError> {
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooManyBytes {
                actual: bytes.len(),
                max: MAX_MANIFEST_BYTES,
            });
        }
        if bytes.last() != Some(&0) {
            return Err(ManifestError::MissingTrailingNul);
        }
        let chunks: Vec<&[u8]> = bytes[..bytes.len() - 1]
            .split(|byte| *byte == 0)
            .take(MAX_MANIFEST_CHUNKS + 1)
            .collect();
        Self::from_chunks(name, &chunks)
    }

    /// Stable manifest name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Number of ordered chunks.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    /// Ordered-NUL SHA-256 digest.
    #[must_use]
    pub fn digest(&self) -> MigrationSetDigest {
        self.digest
    }
}

fn validate_header(name: &str, chunk_count: usize) -> Result<(), ManifestError> {
    if name.is_empty() {
        return Err(ManifestError::EmptyName);
    }
    if chunk_count == 0 {
        return Err(ManifestError::EmptySet);
    }
    if chunk_count > MAX_MANIFEST_CHUNKS {
        return Err(ManifestError::TooManyChunks {
            actual: chunk_count,
            max: MAX_MANIFEST_CHUNKS,
        });
    }
    Ok(())
}

fn framed_len(chunks: &[&[u8]]) -> Result<usize, ManifestError> {
    let mut total = 0_usize;
    for (index, chunk) in chunks.iter().enumerate().take(MAX_MANIFEST_CHUNKS) {
        if chunk.is_empty() {
            return Err(ManifestError::EmptyChunk { index });
        }
        if chunk.contains(&0) {
            return Err(ManifestError::EmbeddedNul { index });
        }
        total = total
            .checked_add(chunk.len().saturating_add(1))
            .ok_or(ManifestError::TooManyBytes {
                actual: usize::MAX,
                max: MAX_MANIFEST_BYTES,
            })?;
        if total > MAX_MANIFEST_BYTES {
            return Err(ManifestError::TooManyBytes {
                actual: total,
                max: MAX_MANIFEST_BYTES,
            });
        }
    }
    Ok(total)
}
```

Expose the module from `lib.rs`:

```rust
pub mod migration_manifest;
pub use migration_manifest::{
    ManifestError, MigrationManifest, MAX_MANIFEST_BYTES, MAX_MANIFEST_CHUNKS,
    SCHEMA_ADVISORY_LOCK_KEY,
};
```

- [ ] **Step 4: Run the scoped green gates**

Run from `rust/`:

```bash
cargo test -p babylon-persistence --test migration_manifest --locked
cargo test -p babylon-persistence --lib --locked
cargo clippy -p babylon-persistence --all-targets --locked -- -D warnings -D clippy::pedantic
```

Expected: every command exits 0.

- [ ] **Step 5: Commit the manifest unit**

Commit the module, crate-root export, and test with:

```text
feat(persistence): pin migration manifest framing
```

### Task 3: Frozen legacy byte vectors

**Files:**

- Create: `tools/export_legacy_postgres_contract.py`
- Create mechanically: `rust/crates/babylon-persistence/tests/fixtures/legacy_schema_ddl_v1.bin`
- Create mechanically: `rust/crates/babylon-persistence/tests/fixtures/legacy_migrations_0010_0044_v1.bin`
- Create: `tests/unit/persistence/test_rust_legacy_contract_fixtures.py`
- Create: `rust/crates/babylon-persistence/tests/legacy_vectors.rs`

**Interfaces:**

- Consumes: Python `POSTGRES_SCHEMA_DDL`, exactly one migration file for every numeric prefix 0010 through 0044, `MigrationManifest::from_nul_framed`, and the shared advisory-lock constant.
- Produces: two NUL-framed, byte-exact fixtures and a `--check` command that fails when either source sequence drifts.

- [ ] **Step 1: Write the failing Python and Rust fixture guards**

The Python test invokes the exporter without changing files:

```python
from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def test_frozen_rust_legacy_postgres_fixtures_match_python_sources() -> None:
    root = Path(__file__).resolve().parents[3]
    result = subprocess.run(
        [sys.executable, str(root / "tools/export_legacy_postgres_contract.py"), "--check"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
```

The Rust test pins both exact sets:

```rust
//! Frozen Python-to-Rust migration adoption vectors at `dev@ae5b2615`.

use babylon_persistence::{MigrationManifest, SCHEMA_ADVISORY_LOCK_KEY};

#[test]
fn legacy_schema_ddl_vector_is_exact() {
    let bytes = include_bytes!("fixtures/legacy_schema_ddl_v1.bin");
    let manifest = MigrationManifest::from_nul_framed("POSTGRES_SCHEMA_DDL", bytes).unwrap();
    assert_eq!(manifest.chunk_count(), 112);
    assert_eq!(
        manifest.digest().to_hex(),
        "0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b"
    );
}

#[test]
fn legacy_numbered_migration_vector_is_exact() {
    let bytes = include_bytes!("fixtures/legacy_migrations_0010_0044_v1.bin");
    let manifest = MigrationManifest::from_nul_framed("migrations-0010-0044", bytes).unwrap();
    assert_eq!(manifest.chunk_count(), 35);
    assert_eq!(
        manifest.digest().to_hex(),
        "4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db"
    );
    assert_eq!(SCHEMA_ADVISORY_LOCK_KEY, 0xBAB1_0537_i64);
}
```

- [ ] **Step 2: Run both guards and verify the red phase**

Run:

```bash
UV_FROZEN=1 .venv/bin/python -m pytest -q tests/unit/persistence/test_rust_legacy_contract_fixtures.py
cd rust
cargo test -p babylon-persistence --test legacy_vectors --locked
```

Expected: Python fails because the exporter is absent; Rust fails because the fixtures are absent.

- [ ] **Step 3: Implement the bounded exporter**

The exporter accepts exactly one of `--write` or `--check`. It imports `POSTGRES_SCHEMA_DDL`, resolves migration files from the repository root, and uses `range(10, 45)` so the migration loop has a fixed 35-step bound. It rejects a missing or duplicate numeric prefix, an empty chunk, more than 256 chunks, more than 1,048,576 framed bytes, and any embedded NUL. It frames each UTF-8 chunk plus one NUL and either writes both fixture paths or compares both expected byte strings with the committed files. No database connection or secret is read.

Create the exporter with this implementation:

```python
#!/usr/bin/env python3
"""Freeze the two Python-era PostgreSQL DDL sequences as Rust byte fixtures."""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"
FIXTURES = ROOT / "rust/crates/babylon-persistence/tests/fixtures"
SCHEMA_FIXTURE = FIXTURES / "legacy_schema_ddl_v1.bin"
MIGRATION_FIXTURE = FIXTURES / "legacy_migrations_0010_0044_v1.bin"
MAX_CHUNKS = 256
MAX_BYTES = 1_048_576
EXPECTED_LOCK_KEY = 0xBAB1_0537


def _numbered_migrations() -> list[str]:
    migration_dir = SRC / "babylon/persistence/migrations"
    chunks: list[str] = []
    for version in range(10, 45):
        matches = sorted(migration_dir.glob(f"{version:04d}_*.sql"))
        if len(matches) != 1:
            raise RuntimeError(
                f"migration {version:04d}: expected one file, found {len(matches)}"
            )
        chunks.append(matches[0].read_text(encoding="utf-8"))
    return chunks


def _frame(chunks: Sequence[str], *, label: str) -> bytes:
    if not chunks:
        raise ValueError(f"{label}: empty sequence")
    if len(chunks) > MAX_CHUNKS:
        raise ValueError(f"{label}: {len(chunks)} chunks exceeds {MAX_CHUNKS}")
    framed = bytearray()
    for index, chunk in enumerate(chunks[:MAX_CHUNKS]):
        encoded = chunk.encode("utf-8")
        if not encoded:
            raise ValueError(f"{label}: empty chunk {index}")
        if b"\0" in encoded:
            raise ValueError(f"{label}: embedded NUL in chunk {index}")
        framed.extend(encoded)
        framed.append(0)
        if len(framed) > MAX_BYTES:
            raise ValueError(f"{label}: framed bytes exceed {MAX_BYTES}")
    return bytes(framed)


def _expected() -> tuple[bytes, bytes]:
    sys.path.insert(0, str(SRC))
    from babylon.persistence.postgres_schema import (  # noqa: PLC0415
        POSTGRES_SCHEMA_DDL,
        SCHEMA_ADVISORY_LOCK_KEY,
    )

    if SCHEMA_ADVISORY_LOCK_KEY != EXPECTED_LOCK_KEY:
        raise RuntimeError(
            "schema advisory-lock key drifted: "
            f"{SCHEMA_ADVISORY_LOCK_KEY:#x} != {EXPECTED_LOCK_KEY:#x}"
        )
    return (
        _frame(POSTGRES_SCHEMA_DDL, label="POSTGRES_SCHEMA_DDL"),
        _frame(_numbered_migrations(), label="migrations-0010-0044"),
    )


def _check(path: Path, expected: bytes) -> bool:
    if not path.is_file():
        print(f"missing fixture: {path}", file=sys.stderr)
        return False
    if path.read_bytes() != expected:
        print(f"stale fixture: {path}", file=sys.stderr)
        return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    schema, migrations = _expected()
    if args.write:
        FIXTURES.mkdir(parents=True, exist_ok=True)
        SCHEMA_FIXTURE.write_bytes(schema)
        MIGRATION_FIXTURE.write_bytes(migrations)
        print("wrote POSTGRES_SCHEMA_DDL=112 migrations=35")
        return 0
    checks = (
        _check(SCHEMA_FIXTURE, schema),
        _check(MIGRATION_FIXTURE, migrations),
    )
    return 0 if all(checks) else 1


if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 4: Generate the fixtures once and prove check mode**

Run from the repository root:

```bash
UV_FROZEN=1 .venv/bin/python tools/export_legacy_postgres_contract.py --write
UV_FROZEN=1 .venv/bin/python tools/export_legacy_postgres_contract.py --check
```

Expected: write mode reports 112 and 35 chunks; check mode exits 0 without modifying either file.

- [ ] **Step 5: Run both language gates**

Run serially:

```bash
UV_FROZEN=1 .venv/bin/python -m pytest -q tests/unit/persistence/test_rust_legacy_contract_fixtures.py
cd rust
cargo test -p babylon-persistence --test legacy_vectors --locked
cargo clippy -p babylon-persistence --all-targets --locked -- -D warnings -D clippy::pedantic
```

Expected: every command exits 0 and the two Rust digests match the frozen Python values exactly.

- [ ] **Step 6: Commit the cross-language vector unit**

Commit the exporter, fixtures, and both tests with:

```text
test(persistence): freeze legacy migration vectors
```

### Task 4: Gate integration and truthful project state

**Files:**

- Modify: `.mise.toml`
- Modify: `ai/state.yaml`
- Verify: `docs/superpowers/plans/2026-08-22-rust-postgresql-contract-keel.md`

**Interfaces:**

- Consumes: the completed crate and both frozen vectors.
- Produces: a package-specific pedantic Clippy leg inside `rust:check` and a state record that distinguishes the landed contract keel from the still-live Python writer.

- [ ] **Step 1: Add the persistence Clippy leg**

Inside the existing `rust:check` script, after the `babylon-kernel` Clippy command, add:

```bash
cargo clippy -p babylon-persistence --all-targets --locked -- -D warnings -D clippy::pedantic
```

- [ ] **Step 2: Update the state record without claiming writer cutover**

Change `rust_postgresql_persistence_boundary.status` to `"ACCEPTED — contract keel landed; database adapter not yet landed"`. Record the contract-keel commit, the two pinned digests, and this exact current reality: Python remains the sole live runtime writer and migrator; the Rust crate opens no database. Set `next` to the separate PER-21 H3 identity plan followed by the read-only legacy adopter.

Use this shape inside the existing record:

```yaml
status: "ACCEPTED — contract keel landed; database adapter not yet landed"
contract_keel:
  crate: rust/crates/babylon-persistence
  opens_database: false
  postgres_dependency: false
  legacy_schema_ddl:
    chunks: 112
    digest: 0902471053ab7a22cdaf0340978712772990e87a63aaaa1636608894fa52590b
  legacy_migrations_0010_0044:
    chunks: 35
    digest: 4abe69ddc25569d5dff1941b4fbe2973df5cbd70a9bca4c92b9fe26f51dd45db
current_reality: >
  Python remains the sole live runtime writer and migrator. The Rust
  contract crate opens no database and cannot write a campaign.
next: >
  Land the separate PER-21 H3 identity contract, then the read-only
  legacy schema adopter and census.
```

- [ ] **Step 3: Run prose and configuration gates**

Run:

```bash
vale docs/superpowers/plans/2026-08-22-rust-postgresql-contract-keel.md
UV_FROZEN=1 .venv/bin/python -c 'import yaml; yaml.safe_load(open("ai/state.yaml", encoding="utf-8"))'
git diff --check
```

Expected: Vale reports 0 errors and 0 warnings; YAML parses; the diff check is empty.

- [ ] **Step 4: Run the final single-flight gates**

Run from the repository root:

```bash
mise run check:quick
mise run rust:check
```

Expected: both tasks exit 0. Do not run another heavy gate concurrently.

- [ ] **Step 5: Commit the gate/state unit and reconcile tracking**

Commit with:

```text
docs(state): record Rust persistence contract keel
```

Comment on PER-20 and GitHub #697 with the four commit SHAs and exact gate results. Keep PER-20 and PER-21 open: this plan lands contracts only and leaves Python as the live writer.

## Self-Review Record

- Spec coverage: this plan covers implementation slice 2 only—typed identities and hash names, failure categories, manifest framing, both frozen legacy digest vectors, advisory lock, and the no-database dependency boundary.
- Deliberate scope cuts: explicit RNG-seed wiring, H3 validation, legacy database census/adoption, additive schemas, backfill, writer, hydration, Archive outbox, cutover, and retirement each have independent acceptance gates and do not belong in the contract-keel review unit.
- Placeholder scan: no unspecified code step or missing type remains in this plan.
- Type consistency: `MigrationSetDigest` is generated in `hashes.rs`, consumed by `MigrationManifest`, and never interchangeable with the replay, graph, content, or reference digest types.
