//! Checked immutable H3 runtime foundations owned below persistence.

use babylon_kernel::{sha256_of, H3CellId};

/// Canonical domain separator for Michigan Dynamic-Hex Foundation V1.
pub const MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1: &[u8] =
    b"babylon.michigan-dynamic-hex-foundation.v1\0";
/// Canonical layout version for Michigan Dynamic-Hex Foundation V1.
pub const MICHIGAN_DYNAMIC_HEX_FOUNDATION_LAYOUT_V1: u32 = 1;
/// Exact governed Michigan R7 cell count.
pub const MICHIGAN_DYNAMIC_HEX_FOUNDATION_ROWS_V1: usize = 45_572;
/// Exact canonical artifact byte count.
pub const MICHIGAN_DYNAMIC_HEX_FOUNDATION_BYTES_V1: usize = 8_750_055;
/// Governed digest of the exact numeric-H3 R7 identity set.
pub const MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1: [u8; 32] = [
    0x7f, 0x8d, 0x12, 0x6e, 0xe8, 0x13, 0x56, 0xa6, 0x06, 0x05, 0x01, 0x3b, 0x4b, 0x1c, 0x23, 0x94,
    0x2a, 0x77, 0xa4, 0xb2, 0xd6, 0xf8, 0x90, 0x12, 0x5d, 0x6c, 0x93, 0x8d, 0xae, 0x70, 0x22, 0x8b,
];
/// Governed digest of the unchanged base H3 reference cohort.
pub const MICHIGAN_DYNAMIC_HEX_BASE_REFERENCE_COHORT_DIGEST_V1: [u8; 32] = [
    0x92, 0xb2, 0x1f, 0xf3, 0x25, 0xbd, 0xe6, 0x7f, 0x26, 0x56, 0x5f, 0x52, 0x88, 0x2d, 0x36, 0x64,
    0xda, 0xac, 0xd6, 0xd5, 0x14, 0x23, 0xf2, 0xa5, 0x88, 0x34, 0x4d, 0xa0, 0x12, 0xfd, 0x41, 0x61,
];
/// Self-framing tag for the complete R8-child-to-R7-parent section.
pub const MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1: &[u8] =
    b"babylon.h3.reference-r8-child-parent.v1\0";
/// Exact number of canonical immediate R8 children of the governed R7 set.
pub const MICHIGAN_DYNAMIC_HEX_R8_CHILD_ROWS_V1: usize = 319_004;
/// Governed digest of the complete self-framed R8 child-parent section.
pub const MICHIGAN_DYNAMIC_HEX_R8_SECTION_DIGEST_V1: [u8; 32] = [
    0xb5, 0xeb, 0xf4, 0x05, 0x14, 0x0f, 0x6f, 0x79, 0xdd, 0xbc, 0x44, 0xfa, 0x10, 0x05, 0xb1, 0x95,
    0xbe, 0xd0, 0xbc, 0x28, 0xe0, 0xea, 0xcf, 0x2d, 0x8e, 0x16, 0x97, 0xcd, 0x9c, 0x83, 0x94, 0x91,
];
/// Governed digest of the composite base-cohort plus R8 reference bundle.
pub const MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1: [u8; 32] = [
    0x84, 0xbb, 0xff, 0xa9, 0xb2, 0x38, 0x8a, 0xa1, 0x68, 0xc0, 0x65, 0xe7, 0x10, 0xa6, 0x13, 0x13,
    0xfb, 0xd4, 0x65, 0x22, 0xd2, 0x02, 0x2b, 0x62, 0x8f, 0x09, 0x19, 0xec, 0xff, 0xec, 0x98, 0x31,
];
/// Governed SHA-256 of the sole canonical foundation artifact.
pub const MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1: [u8; 32] = [
    0x81, 0xee, 0x8f, 0x8a, 0xbb, 0xee, 0x67, 0x27, 0x65, 0x5d, 0x52, 0xc6, 0xd5, 0x6a, 0x6f, 0x29,
    0x67, 0xaf, 0x9d, 0xfd, 0xf0, 0x1d, 0xa5, 0x3d, 0xd5, 0x93, 0xda, 0x83, 0x39, 0xd6, 0x50, 0xa4,
];

const SOURCE_IDENTITY_DOMAIN: &[u8] = b"babylon.h3.reference-source.v1\0";
const REFERENCE_BUNDLE_DOMAIN: &[u8] = b"babylon.h3.reference-bundle-composite.v1\0";
const VALUE_LANES: usize = 9;
const ROW_BYTES: usize = 8 + VALUE_LANES * 8;
const R8_CHILD_PARENT_ROW_BYTES: usize = 16;

/// Named exact binary64 inputs for one dynamic-H3 foundation row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MichiganDynamicHexValueBitsV1 {
    /// Constant capital.
    pub c: u64,
    /// Variable capital.
    pub v: u64,
    /// Surplus value.
    pub s: u64,
    /// Capital stock.
    pub k: u64,
    /// Biocapacity stock.
    pub biocapacity_stock: u64,
    /// Energy stock.
    pub energy_stock: u64,
    /// Raw-material stock.
    pub raw_material_stock: u64,
    /// Internet access proportion.
    pub internet_access_pct: u64,
    /// Surveillance coupling proportion.
    pub surveillance_coupling: u64,
}

impl MichiganDynamicHexValueBitsV1 {
    const fn as_array(self) -> [u64; VALUE_LANES] {
        [
            self.c,
            self.v,
            self.s,
            self.k,
            self.biocapacity_stock,
            self.energy_stock,
            self.raw_material_stock,
            self.internet_access_pct,
            self.surveillance_coupling,
        ]
    }
}

/// Checked named dynamic-H3 foundation values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MichiganDynamicHexValuesV1 {
    bits: MichiganDynamicHexValueBitsV1,
}

impl MichiganDynamicHexValuesV1 {
    /// Validate all nine existing dynamic-H3 value domains.
    ///
    /// # Errors
    /// Returns a lane-specific refusal for non-finite, negative-zero,
    /// negative stock/value, or out-of-unit-interval values.
    pub fn try_new(
        bits: MichiganDynamicHexValueBitsV1,
    ) -> Result<Self, MichiganDynamicHexFoundationErrorV1> {
        let lanes = bits.as_array();
        for (lane, raw) in lanes.iter().copied().enumerate() {
            if !f64::from_bits(raw).is_finite() {
                return Err(MichiganDynamicHexFoundationErrorV1::NonFiniteValue { lane });
            }
            if raw == (-0.0_f64).to_bits() {
                return Err(MichiganDynamicHexFoundationErrorV1::NegativeZero { lane });
            }
        }
        for (lane, raw) in lanes[..7].iter().copied().enumerate() {
            if f64::from_bits(raw) < 0.0 {
                return Err(MichiganDynamicHexFoundationErrorV1::NegativeValue { lane });
            }
        }
        for (lane, raw) in lanes[7..].iter().copied().enumerate() {
            if !(0.0..=1.0).contains(&f64::from_bits(raw)) {
                return Err(MichiganDynamicHexFoundationErrorV1::UnitIntervalValue {
                    lane: lane + 7,
                });
            }
        }
        Ok(Self { bits })
    }

    /// Return the exact named binary64 inputs.
    #[must_use]
    pub const fn named_bits(&self) -> MichiganDynamicHexValueBitsV1 {
        self.bits
    }

    /// Return all lanes in the frozen artifact order.
    #[must_use]
    pub const fn value_bits(&self) -> [u64; VALUE_LANES] {
        self.bits.as_array()
    }
}

/// One exact P27-consensus dynamic-H3 foundation row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MichiganDynamicHexFoundationRowV1 {
    cell_id: H3CellId,
    values: MichiganDynamicHexValuesV1,
}

impl MichiganDynamicHexFoundationRowV1 {
    /// Construct one checked R7 row from named checked values.
    ///
    /// # Errors
    /// Returns a typed refusal for a non-R7 identity.
    pub fn try_new(
        cell_id: H3CellId,
        values: MichiganDynamicHexValuesV1,
    ) -> Result<Self, MichiganDynamicHexFoundationErrorV1> {
        if cell_id.resolution() != 7 {
            return Err(MichiganDynamicHexFoundationErrorV1::NonR7Cell { cell: cell_id });
        }
        Ok(Self { cell_id, values })
    }

    /// Return the sole checked H3 identity.
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    /// Return the named checked lane values.
    #[must_use]
    pub const fn values(&self) -> &MichiganDynamicHexValuesV1 {
        &self.values
    }

    /// Return all nine exact observed bits in frozen artifact order.
    #[must_use]
    pub const fn value_bits(&self) -> [u64; VALUE_LANES] {
        self.values.value_bits()
    }
}

/// One governed R8 child and its exact immediate R7 parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MichiganH3R8ChildParentV1 {
    child_id: H3CellId,
    parent_r7: H3CellId,
}

impl MichiganH3R8ChildParentV1 {
    /// Construct one checked immediate-child relation.
    ///
    /// # Errors
    /// Returns a typed refusal for the wrong resolutions or parent relation.
    pub fn try_new(
        child_id: H3CellId,
        parent_r7: H3CellId,
    ) -> Result<Self, MichiganDynamicHexFoundationErrorV1> {
        if child_id.resolution() != 8 {
            return Err(MichiganDynamicHexFoundationErrorV1::NonR8Child { cell: child_id });
        }
        if parent_r7.resolution() != 7 {
            return Err(MichiganDynamicHexFoundationErrorV1::NonR7Parent { cell: parent_r7 });
        }
        if child_id.immediate_parent() != Some(parent_r7) {
            return Err(MichiganDynamicHexFoundationErrorV1::R8ParentMismatch {
                child: child_id,
                parent: parent_r7,
            });
        }
        Ok(Self {
            child_id,
            parent_r7,
        })
    }

    /// Return the governed R8 child identity.
    #[must_use]
    pub const fn child_cell_id(&self) -> H3CellId {
        self.child_id
    }

    /// Return the child's immediate governed R7 parent.
    #[must_use]
    pub const fn parent_r7_cell_id(&self) -> H3CellId {
        self.parent_r7
    }
}

fn validate_r8_coverage(
    actual: &[MichiganH3R8ChildParentV1],
    expected: &[MichiganH3R8ChildParentV1],
) -> Result<(), MichiganDynamicHexFoundationErrorV1> {
    if actual.len() != expected.len() {
        return Err(
            MichiganDynamicHexFoundationErrorV1::R8CoverageLengthMismatch {
                expected: expected.len(),
                actual: actual.len(),
            },
        );
    }
    for (index, (actual_row, expected_row)) in actual.iter().zip(expected).enumerate() {
        if actual_row != expected_row {
            return Err(MichiganDynamicHexFoundationErrorV1::R8CoverageMismatch { index });
        }
    }
    Ok(())
}

/// Tick-owned checked Michigan Dynamic-Hex Foundation V1.
#[derive(Debug, PartialEq, Eq)]
pub struct MichiganDynamicHexFoundationV1 {
    rows: Vec<MichiganDynamicHexFoundationRowV1>,
    r8_child_parent_rows: Vec<MichiganH3R8ChildParentV1>,
    canonical_bytes: Vec<u8>,
    artifact_sha256: [u8; 32],
}

impl MichiganDynamicHexFoundationV1 {
    /// Construct only the exact complete governed Michigan foundation.
    ///
    /// # Errors
    /// Returns a closed refusal if the row count, strict numeric-H3 order,
    /// recomputed source identity, exact artifact SHA, or allocation is invalid.
    #[allow(
        clippy::too_many_lines,
        reason = "the bounded constructor verifies one indivisible canonical artifact"
    )]
    pub fn try_new(
        rows: Vec<MichiganDynamicHexFoundationRowV1>,
        r8_child_parent_rows: Vec<MichiganH3R8ChildParentV1>,
    ) -> Result<Self, MichiganDynamicHexFoundationErrorV1> {
        if rows.len() != MICHIGAN_DYNAMIC_HEX_FOUNDATION_ROWS_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::RowCount { actual: rows.len() });
        }
        for (index, pair) in rows.windows(2).enumerate() {
            if pair[0].cell_id.as_u64() >= pair[1].cell_id.as_u64() {
                return Err(MichiganDynamicHexFoundationErrorV1::H3Order {
                    right_index: index + 1,
                });
            }
        }

        let mut source_identity = Vec::new();
        let source_capacity = SOURCE_IDENTITY_DOMAIN
            .len()
            .checked_add(8)
            .and_then(|header| header.checked_add(rows.len().checked_mul(8)?))
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        source_identity
            .try_reserve_exact(source_capacity)
            .map_err(|_| MichiganDynamicHexFoundationErrorV1::Allocation)?;
        source_identity.extend_from_slice(SOURCE_IDENTITY_DOMAIN);
        source_identity.extend_from_slice(
            &u64::try_from(rows.len())
                .map_err(|_| MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?
                .to_be_bytes(),
        );
        for row in &rows {
            source_identity.extend_from_slice(&row.cell_id.to_be_bytes());
        }
        if sha256_of(&source_identity) != MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::SourceR7Digest);
        }

        if r8_child_parent_rows.len() != MICHIGAN_DYNAMIC_HEX_R8_CHILD_ROWS_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::R8RowCount {
                actual: r8_child_parent_rows.len(),
            });
        }
        for (index, pair) in r8_child_parent_rows.windows(2).enumerate() {
            if pair[0].child_id.as_u64() >= pair[1].child_id.as_u64() {
                return Err(MichiganDynamicHexFoundationErrorV1::R8Order {
                    right_index: index + 1,
                });
            }
        }
        let mut expected_r8 = Vec::new();
        expected_r8
            .try_reserve_exact(MICHIGAN_DYNAMIC_HEX_R8_CHILD_ROWS_V1)
            .map_err(|_| MichiganDynamicHexFoundationErrorV1::Allocation)?;
        for row in &rows {
            let children = row.cell_id.immediate_children().map_err(|_| {
                MichiganDynamicHexFoundationErrorV1::R8ChildDerivation {
                    parent: row.cell_id,
                }
            })?;
            for child in children.iter() {
                expected_r8.push(MichiganH3R8ChildParentV1 {
                    child_id: child,
                    parent_r7: row.cell_id,
                });
            }
        }
        expected_r8.sort_unstable_by_key(|row| row.child_id);
        validate_r8_coverage(&r8_child_parent_rows, &expected_r8)?;

        let r8_rows_bytes = r8_child_parent_rows
            .len()
            .checked_mul(R8_CHILD_PARENT_ROW_BYTES)
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        let r8_section_capacity = MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1
            .len()
            .checked_add(8)
            .and_then(|header| header.checked_add(r8_rows_bytes))
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        let mut r8_section = Vec::new();
        r8_section
            .try_reserve_exact(r8_section_capacity)
            .map_err(|_| MichiganDynamicHexFoundationErrorV1::Allocation)?;
        r8_section.extend_from_slice(MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1);
        r8_section.extend_from_slice(
            &u64::try_from(r8_child_parent_rows.len())
                .map_err(|_| MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?
                .to_be_bytes(),
        );
        for row in &r8_child_parent_rows {
            r8_section.extend_from_slice(&row.child_id.to_be_bytes());
            r8_section.extend_from_slice(&row.parent_r7.to_be_bytes());
        }
        let r8_child_parent_digest = sha256_of(&r8_section);
        if r8_child_parent_digest != MICHIGAN_DYNAMIC_HEX_R8_SECTION_DIGEST_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::R8SectionDigest);
        }

        let mut reference_bundle = Vec::new();
        let reference_capacity = REFERENCE_BUNDLE_DOMAIN
            .len()
            .checked_add(64)
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        reference_bundle
            .try_reserve_exact(reference_capacity)
            .map_err(|_| MichiganDynamicHexFoundationErrorV1::Allocation)?;
        reference_bundle.extend_from_slice(REFERENCE_BUNDLE_DOMAIN);
        reference_bundle.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_BASE_REFERENCE_COHORT_DIGEST_V1);
        reference_bundle.extend_from_slice(&r8_child_parent_digest);
        if sha256_of(&reference_bundle) != MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::ReferenceBundleDigest);
        }

        let row_bytes = rows
            .len()
            .checked_mul(ROW_BYTES)
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        let capacity = MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1
            .len()
            .checked_add(4 + 32 * 4 + 8)
            .and_then(|header| header.checked_add(row_bytes))
            .and_then(|without_r8| without_r8.checked_add(r8_section.len()))
            .ok_or(MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?;
        let mut canonical_bytes = Vec::new();
        canonical_bytes
            .try_reserve_exact(capacity)
            .map_err(|_| MichiganDynamicHexFoundationErrorV1::Allocation)?;
        canonical_bytes.extend_from_slice(MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1);
        canonical_bytes.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_FOUNDATION_LAYOUT_V1.to_be_bytes());
        canonical_bytes.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1);
        canonical_bytes.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_BASE_REFERENCE_COHORT_DIGEST_V1);
        canonical_bytes.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_R8_SECTION_DIGEST_V1);
        canonical_bytes.extend_from_slice(&MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1);
        canonical_bytes.extend_from_slice(
            &u64::try_from(rows.len())
                .map_err(|_| MichiganDynamicHexFoundationErrorV1::CapacityOverflow)?
                .to_be_bytes(),
        );
        for row in &rows {
            canonical_bytes.extend_from_slice(&row.cell_id.to_be_bytes());
            for bits in row.value_bits() {
                canonical_bytes.extend_from_slice(&bits.to_be_bytes());
            }
        }
        canonical_bytes.extend_from_slice(&r8_section);
        debug_assert_eq!(canonical_bytes.len(), capacity);
        let artifact_sha256 = sha256_of(&canonical_bytes);
        if artifact_sha256 != MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1 {
            return Err(MichiganDynamicHexFoundationErrorV1::ArtifactDigest);
        }
        Ok(Self {
            rows,
            r8_child_parent_rows,
            canonical_bytes,
            artifact_sha256,
        })
    }

    /// Return the exact canonical domain separator.
    #[must_use]
    pub const fn domain(&self) -> &'static [u8] {
        MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1
    }

    /// Return the closed layout version.
    #[must_use]
    pub const fn layout(&self) -> u32 {
        MICHIGAN_DYNAMIC_HEX_FOUNDATION_LAYOUT_V1
    }

    /// Return the governed source R7 identity digest.
    #[must_use]
    pub const fn source_r7_digest(&self) -> [u8; 32] {
        MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1
    }

    /// Return the unchanged governed base H3 reference-cohort digest.
    #[must_use]
    pub const fn base_reference_cohort_digest(&self) -> [u8; 32] {
        MICHIGAN_DYNAMIC_HEX_BASE_REFERENCE_COHORT_DIGEST_V1
    }

    /// Return the digest of the complete self-framed R8 child-parent section.
    #[must_use]
    pub const fn r8_section_digest(&self) -> [u8; 32] {
        MICHIGAN_DYNAMIC_HEX_R8_SECTION_DIGEST_V1
    }

    /// Return the distinct governed reference-bundle digest.
    #[must_use]
    pub const fn reference_bundle_digest(&self) -> [u8; 32] {
        MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1
    }

    /// Return the exact numeric-H3 ordered observed rows.
    #[must_use]
    pub fn rows(&self) -> &[MichiganDynamicHexFoundationRowV1] {
        &self.rows
    }

    /// Return every governed R8 child with its immediate R7 parent.
    #[must_use]
    pub fn r8_child_parent_rows(&self) -> &[MichiganH3R8ChildParentV1] {
        &self.r8_child_parent_rows
    }

    /// Return the canonical artifact bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Return SHA-256 of the exact canonical artifact bytes.
    #[must_use]
    pub const fn artifact_sha256(&self) -> [u8; 32] {
        self.artifact_sha256
    }
}

/// Closed Michigan foundation construction failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MichiganDynamicHexFoundationErrorV1 {
    /// The H3 cell is not resolution seven.
    NonR7Cell { cell: H3CellId },
    /// One R8-section child is not resolution eight.
    NonR8Child { cell: H3CellId },
    /// One R8-section parent is not resolution seven.
    NonR7Parent { cell: H3CellId },
    /// One R8 child does not identify the supplied immediate parent.
    R8ParentMismatch { child: H3CellId, parent: H3CellId },
    /// One observed binary64 lane is not finite.
    NonFiniteValue { lane: usize },
    /// One observed binary64 lane uses the forbidden negative-zero encoding.
    NegativeZero { lane: usize },
    /// One stock or material value is negative.
    NegativeValue { lane: usize },
    /// One proportion falls outside the closed unit interval.
    UnitIntervalValue { lane: usize },
    /// The recomputed source R7 digest is not the governed identity.
    SourceR7Digest,
    /// The artifact does not carry the exact complete Michigan R7 row count.
    RowCount { actual: usize },
    /// Numeric H3 identities are not strictly ascending and unique.
    H3Order { right_index: usize },
    /// The artifact does not carry the exact complete R8 child count.
    R8RowCount { actual: usize },
    /// R8 child identities are not strictly ascending and unique.
    R8Order { right_index: usize },
    /// Kernel-owned immediate-child derivation unexpectedly refused one R7 parent.
    R8ChildDerivation { parent: H3CellId },
    /// Kernel derivation and the supplied R8 section have different row counts.
    R8CoverageLengthMismatch { expected: usize, actual: usize },
    /// The R8 section differs from the exact kernel-derived child-parent set.
    R8CoverageMismatch { index: usize },
    /// The self-framed R8 child-parent section digest drifted.
    R8SectionDigest,
    /// The composite base-cohort plus R8 reference digest drifted.
    ReferenceBundleDigest,
    /// Canonical bytes do not equal the one governed artifact SHA-256.
    ArtifactDigest,
    /// Canonical byte size arithmetic overflowed.
    CapacityOverflow,
    /// Canonical byte allocation refused.
    Allocation,
}

impl std::fmt::Display for MichiganDynamicHexFoundationErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Michigan dynamic-H3 foundation: {self:?}"
        )
    }
}

impl std::error::Error for MichiganDynamicHexFoundationErrorV1 {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn r8_coverage_refuses_length_mismatch_before_pairwise_comparison() {
        let parent = H3CellId::from_str("872664800ffffff").unwrap();
        let child = H3CellId::from_str("8826648001fffff").unwrap();
        let expected = [MichiganH3R8ChildParentV1::try_new(child, parent).unwrap()];

        assert_eq!(
            validate_r8_coverage(&[], &expected),
            Err(
                MichiganDynamicHexFoundationErrorV1::R8CoverageLengthMismatch {
                    expected: 1,
                    actual: 0,
                }
            )
        );
    }
}
