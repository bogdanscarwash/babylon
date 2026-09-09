//! Exemptions from the intensivity aggregation law (§3.4), mirroring the
//! governance shape of Python's `SentinelExemption`
//! (`babylon.sentinels.exemptions`): every row needs a reason, an owner,
//! and a date, and adding one takes the same sign-off as a sentinel
//! exemption. An exemption suppresses `E-TYPE-041/042/043` **for the named
//! field only** and is itself content, inside `rules_hash`.
//!
//! This ledger starts EMPTY in Phase 1 — no BSL content exists yet to need
//! an exemption. Phase 2's transcription work is expected to populate it,
//! never this crate's own tasks.

/// One declared exemption row.
#[derive(Debug, Clone)]
pub struct IntensiveAggregationExemption {
    /// The exempted field, by its BSL name (kebab-case).
    pub field_name: &'static str,
    /// Why the aggregation law does not apply — mandatory, never empty.
    pub reason: &'static str,
    /// Who signed the row off.
    pub owner: &'static str,
    /// Sign-off date, ISO 8601.
    pub date: &'static str,
}

/// The declared ledger (spec §5). Empty until Phase 2 content needs a row.
pub const EXTENSIVE_INTENSIVE_EXEMPTIONS: &[IntensiveAggregationExemption] = &[];

#[cfg(test)]
mod tests {
    use super::EXTENSIVE_INTENSIVE_EXEMPTIONS;

    #[test]
    fn every_exemption_row_carries_full_governance_metadata() {
        for exemption in EXTENSIVE_INTENSIVE_EXEMPTIONS {
            assert!(!exemption.field_name.is_empty());
            assert!(!exemption.reason.is_empty());
            assert!(!exemption.owner.is_empty());
            assert!(!exemption.date.is_empty());
        }
    }
}
