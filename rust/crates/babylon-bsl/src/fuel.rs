//! The per-AST-node fuel cost model (`bsl-language.rst` §3.7 / §4.5 — the
//! NORMATIVE source for every constant below; this module transcribes it, it
//! does not originate it). The determinism contract's fuel chapter is a
//! pointer to that table, deliberately: exactly one normative table exists.
//!
//! Two tiers, distinguished per §3.7: the five BASE rows are copied from the
//! design document's Phase-0 cost model and are **pinned by conformance
//! vector — revising one is a vector re-bless**; the remaining rows are the
//! language reference's completion of that model and are
//! `[draft ruling — Phase 1 review]`. Neither tier is a tuning knob.

use babylon_kernel::sha256_of;
use std::collections::HashMap;

use crate::identity_codec::{validate_intrinsic_identity, IntrinsicIdentityViolation};

const MAX_SFS_IDENTITY_ROWS: usize = 64;

/// A refusal while constructing a complete synthetic-audit fuel-table identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsFuelIdentityError {
    /// A complete table contains more than 64 rows.
    RowLimit {
        /// Which table failed.
        table: &'static str,
        /// Complete row count.
        actual: usize,
    },
    /// Two complete encoded rows are byte-identical.
    DuplicateRow {
        /// Which table failed.
        table: &'static str,
        /// The duplicate row.
        row: String,
    },
    /// A key is empty.
    KeyEmpty {
        /// Which table failed.
        table: &'static str,
    },
    /// A key exceeds 96 bytes.
    KeyTooLong {
        /// Which table failed.
        table: &'static str,
        /// Actual byte length.
        actual: usize,
    },
    /// A key is not strict ASCII.
    KeyNonAscii {
        /// Which table failed.
        table: &'static str,
    },
    /// A key contains a row delimiter.
    KeyContainsDelimiter {
        /// Which table failed.
        table: &'static str,
    },
}

fn validate_identity_key(table: &'static str, key: &str) -> Result<(), SfsFuelIdentityError> {
    validate_intrinsic_identity(key).map_err(|violation| match violation {
        IntrinsicIdentityViolation::Empty => SfsFuelIdentityError::KeyEmpty { table },
        IntrinsicIdentityViolation::TooLong { actual } => {
            SfsFuelIdentityError::KeyTooLong { table, actual }
        }
        IntrinsicIdentityViolation::NonAscii { .. } => SfsFuelIdentityError::KeyNonAscii { table },
        IntrinsicIdentityViolation::Delimiter { .. } => {
            SfsFuelIdentityError::KeyContainsDelimiter { table }
        }
    })
}

fn append_rows(
    table: &'static str,
    prefix: &str,
    values: &HashMap<String, u64>,
    rows: &mut Vec<String>,
) -> Result<(), SfsFuelIdentityError> {
    let mut entries = values.iter();
    for _index in 0..MAX_SFS_IDENTITY_ROWS {
        let Some((key, value)) = entries.next() else {
            return Ok(());
        };
        validate_identity_key(table, key)?;
        rows.push(format!("{prefix}|{key}|{value}\n"));
    }
    if entries.next().is_some() {
        return Err(SfsFuelIdentityError::RowLimit {
            table,
            actual: values.len(),
        });
    }
    Ok(())
}

fn table_identity(
    table: &'static str,
    domain: &[u8],
    rows: &mut [String],
) -> Result<[u8; 32], SfsFuelIdentityError> {
    if rows.len() > MAX_SFS_IDENTITY_ROWS {
        return Err(SfsFuelIdentityError::RowLimit {
            table,
            actual: rows.len(),
        });
    }
    rows.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for index in 1..MAX_SFS_IDENTITY_ROWS {
        if index >= rows.len() {
            break;
        }
        if rows[index - 1] == rows[index] {
            return Err(SfsFuelIdentityError::DuplicateRow {
                table,
                row: rows[index].clone(),
            });
        }
    }
    let mut preimage = Vec::new();
    preimage.extend_from_slice(domain);
    preimage.push(0);
    for index in 0..MAX_SFS_IDENTITY_ROWS {
        let Some(row) = rows.get(index) else { break };
        preimage.extend_from_slice(row.as_bytes());
    }
    Ok(sha256_of(&preimage))
}

/// The §3.7 cost table, one constant per row. Changing any constant here
/// requires the conformance-vector re-bless the reference chapter mandates
/// — never edit silently.
pub mod cost {
    /// Base row (vector-pinned): `cost(literal) = 0`. Enum-refs and field
    /// paths (qnames) share this row per the draft
    /// `cost(field path | enum-ref) = 0` — static, like a literal.
    pub const LITERAL: u64 = 0;
    /// Base row (vector-pinned): `cost(variable-ref) = 1`.
    pub const VARIABLE_REF: u64 = 1;
    /// Base row (vector-pinned): `cost(arith | cmp | bool) = 1 + Σ children`.
    /// One row, one constant — the three operator families share it.
    pub const ARITH_CMP_BOOL_BASE: u64 = 1;
    /// Base row (vector-pinned):
    /// `cost(intrinsic call) = 5 + declared_cost(callee) + Σ cost(args)`.
    pub const INTRINSIC_CALL_BASE: u64 = 5;
    /// Base row (vector-pinned):
    /// `cost(fold) = 2 + cost(query) + ceiling(query) × (cost(body) + cost(weight))`.
    pub const FOLD_BASE: u64 = 2;

    /// Draft row (Phase 1 review):
    /// `cost(if) = 1 + cost(cond) + max(cost(then), cost(else))`.
    pub const IF_BASE: u64 = 1;
    /// Draft row (Phase 1 review):
    /// `cost(exists | forall) = 2 + cost(query) + ceiling(query) × cost(body)`.
    pub const EXISTS_FORALL_BASE: u64 = 2;
    /// Draft row (Phase 1 review):
    /// `cost(query) = 1 + cost(element predicate, if any)`.
    pub const QUERY_BASE: u64 = 1;
    /// Draft row (Phase 1 review): `cost(update-op) = 1 + cost(operand)`
    /// — `add` | `sub` | `set` | `scale`.
    pub const UPDATE_OP_BASE: u64 = 1;
    /// Draft row (Phase 1 review):
    /// `cost(structural verb) = 3 + Σ cost(operands)`.
    pub const STRUCTURAL_VERB_BASE: u64 = 3;
    /// Draft row (Phase 1 review):
    /// `cost(guard) = 1 + cost(cond) + Σ cost(effect-items)`.
    pub const GUARD_BASE: u64 = 1;
    /// Draft row (R9 chapters C1/C2/C3/C9, D38): every §2.10 accessor —
    /// `field-of`, `edge-between`, `the`, `metric-of` — charges a
    /// variable-reference base of 1 plus its operands, and is **never
    /// multiplied by a ceiling**: none of them ranges over a set. That is
    /// what keeps the Power-of-10 Rule 2 claim static as the accessor set
    /// grows. `cost(the) = 1` falls out of the same row (it has no operand).
    pub const ACCESSOR_BASE: u64 = 1;
    /// Draft row (R9 chapter C5): `cost(select-max | select-min) =
    /// 2 + cost(query) + ceiling(query) × cost(score)`.
    pub const SELECTION_BASE: u64 = 2;
    /// Draft row (R9 chapter C6): `cost(for-each) =
    /// 2 + cost(query) + ceiling(query) × Σ cost(effect-items)`.
    pub const FOR_EACH_BASE: u64 = 2;
    // cost(members list) = Σ cost(members) — grouping, no base cost: no
    // constant exists on purpose, so no code path can charge one.
    // cost(domain) = 0 and cost(:as name) = 0 (§3.7, R9 C4/C8) likewise get
    // no constant: a row that charges nothing must have no charging path.
}

/// Declared per-`NodeType` / per-`EdgeType` / per-`HyperedgeType` cardinality
/// ceilings from a scenario manifest (§3.7: the bound is computed "against
/// declared cardinality ceilings, not the runtime graph"; §2.9 `manifest`).
/// Phase 1 takes this as an opaque lookup; parsing the manifest form itself
/// is Phase 2 content work.
///
/// **Two axes, since Amendment D** (§3.7 member-count axis): a hyperedge type
/// declares both how many hyperedges may exist (`:ceiling`) and how many
/// members any one of them may carry (`:max-members`). A fold over
/// `members-of` bounds against the second; without it there is no static
/// bound at all.
///
/// Keys are the enum-ref as written in content — `"NodeType/SOCIAL_CLASS"`,
/// `"EdgeType/SOLIDARITY"`, `"HyperedgeType/ECONOMIC_SECTOR"`.
#[derive(Debug, Clone, Default)]
pub struct CardinalityCeilings {
    ceilings: HashMap<String, u64>,
    max_members: HashMap<String, u64>,
}

impl CardinalityCeilings {
    /// Build from the two declared maps (`:ceiling` rows, `:max-members`
    /// values on `HyperedgeType` rows).
    #[must_use]
    pub fn new(ceilings: HashMap<String, u64>, max_members: HashMap<String, u64>) -> Self {
        Self {
            ceilings,
            max_members,
        }
    }

    /// The declared `:ceiling` of a node/edge/hyperedge type. `None` means
    /// the manifest declares no ceiling for that type — the bound checker
    /// treats that as a loud load error, never as `0` (a silent `0` would
    /// UNDER-count the bound, the exact inversion of III.11).
    #[must_use]
    pub fn ceiling(&self, graph_element_type: &str) -> Option<u64> {
        self.ceilings.get(graph_element_type).copied()
    }

    /// The declared `:max-members` of a hyperedge type. `None` for a
    /// node/edge type — and a `None` here on a `members-of` fold is
    /// `E-LOAD-042` (§2.9: `:max-members` is mandatory on a `HyperedgeType`
    /// row), never a silent zero.
    #[must_use]
    pub fn max_members(&self, hyperedge_type: &str) -> Option<u64> {
        self.max_members.get(hyperedge_type).copied()
    }

    /// Hash the complete bounded ceiling and max-member tables.
    ///
    /// # Errors
    ///
    /// [`SfsFuelIdentityError`] for a non-canonical key or more than 64
    /// combined rows.
    pub fn sfs_identity_digest(&self) -> Result<[u8; 32], SfsFuelIdentityError> {
        let row_count = self
            .ceilings
            .len()
            .checked_add(self.max_members.len())
            .ok_or(SfsFuelIdentityError::RowLimit {
                table: "cardinality",
                actual: usize::MAX,
            })?;
        if row_count > MAX_SFS_IDENTITY_ROWS {
            return Err(SfsFuelIdentityError::RowLimit {
                table: "cardinality",
                actual: row_count,
            });
        }
        let mut rows = Vec::with_capacity(row_count);
        append_rows("cardinality", "ceiling", &self.ceilings, &mut rows)?;
        append_rows("cardinality", "max-members", &self.max_members, &mut rows)?;
        table_identity(
            "cardinality",
            b"babylon.sfs-cardinality-ceilings.v1",
            &mut rows,
        )
    }
}

/// The declared `:cost` of each kernel intrinsic (§2.7 `intrinsic-decl`),
/// keyed by intrinsic name. The table's *contents* are Phase 2 work; the
/// bound checker only needs the lookup so `cost(intrinsic call)` is
/// computable from content alone. A call to a name absent here is
/// `E-LOAD-021` — never a default cost.
#[derive(Debug, Clone, Default)]
pub struct IntrinsicCosts {
    costs: HashMap<String, u64>,
}

impl IntrinsicCosts {
    /// Build from declared `(intrinsic <name> … :cost <n>)` rows.
    #[must_use]
    pub fn new(costs: HashMap<String, u64>) -> Self {
        Self { costs }
    }

    /// The declared cost of `name`, or `None` if undeclared.
    #[must_use]
    pub fn declared_cost(&self, name: &str) -> Option<u64> {
        self.costs.get(name).copied()
    }

    /// Iterate the declared intrinsic-cost rows without imposing map order.
    pub fn identity_rows(&self) -> impl Iterator<Item = (&str, u64)> {
        self.costs.iter().map(|(name, cost)| (name.as_str(), *cost))
    }

    /// Hash the complete bounded intrinsic-cost table.
    ///
    /// # Errors
    ///
    /// [`SfsFuelIdentityError`] for a non-canonical key or more than 64 rows.
    pub fn sfs_identity_digest(&self) -> Result<[u8; 32], SfsFuelIdentityError> {
        if self.costs.len() > MAX_SFS_IDENTITY_ROWS {
            return Err(SfsFuelIdentityError::RowLimit {
                table: "intrinsic",
                actual: self.costs.len(),
            });
        }
        let mut rows = Vec::with_capacity(self.costs.len());
        append_rows("intrinsic", "intrinsic", &self.costs, &mut rows)?;
        table_identity("intrinsic", b"babylon.sfs-intrinsic-costs.v1", &mut rows)
    }
}

#[cfg(test)]
mod sfs_profile_tests {
    use super::{table_identity, SfsFuelIdentityError};

    #[test]
    fn sfs_profile_identity_encoder_rejects_duplicate_rows() {
        let mut rows = vec!["intrinsic|same|1\n".to_owned(); 2];
        assert_eq!(
            table_identity("intrinsic", b"test", &mut rows),
            Err(SfsFuelIdentityError::DuplicateRow {
                table: "intrinsic",
                row: "intrinsic|same|1\n".to_owned(),
            })
        );
    }
}
