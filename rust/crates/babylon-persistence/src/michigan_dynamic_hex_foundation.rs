//! Static decoding boundary for the checked Michigan Dynamic-Hex Foundation V1.

use std::sync::OnceLock;

use babylon_kernel::{sha256_of, H3CellId, H3CellIdError};
use babylon_tick::h3_runtime::{
    MichiganDynamicHexFoundationErrorV1, MichiganDynamicHexFoundationRowV1,
    MichiganDynamicHexFoundationV1, MichiganDynamicHexValueBitsV1, MichiganDynamicHexValuesV1,
    MichiganH3R8ChildParentV1, MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1,
    MICHIGAN_DYNAMIC_HEX_FOUNDATION_BYTES_V1, MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1,
    MICHIGAN_DYNAMIC_HEX_FOUNDATION_LAYOUT_V1, MICHIGAN_DYNAMIC_HEX_FOUNDATION_ROWS_V1,
    MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1, MICHIGAN_DYNAMIC_HEX_R8_CHILD_ROWS_V1,
};

const FIXTURE_PARTS: [&[u8]; 9] = [
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-00.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-01.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-02.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-03.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-04.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-05.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-06.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-07.bin"),
    include_bytes!("fixtures/michigan_dynamic_hex_foundation_v1.part-08.bin"),
];
static FOUNDATION: OnceLock<
    Result<MichiganDynamicHexFoundationV1, MichiganDynamicHexFoundationDecodeErrorV1>,
> = OnceLock::new();

/// Closed decoding failures for the immutable Michigan foundation artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MichiganDynamicHexFoundationDecodeErrorV1 {
    /// The supplied byte sequence is shorter than the one governed artifact.
    Truncated,
    /// The supplied byte sequence carries bytes after the one governed artifact.
    TrailingBytes,
    /// SHA-256 differs from the pinned canonical artifact.
    ArtifactDigest,
    /// The domain separator differs from the closed V1 layout.
    Domain,
    /// The layout version differs from V1.
    Layout,
    /// The separately tagged R8 child-parent section is absent or different.
    R8SectionDomain,
    /// A raw cell is not a valid H3 identity.
    H3Identity(H3CellIdError),
    /// The tick-owned checked value refused the decoded structure.
    Foundation(MichiganDynamicHexFoundationErrorV1),
    /// Fixture joining allocation refused.
    Allocation,
}

impl std::fmt::Display for MichiganDynamicHexFoundationDecodeErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid Michigan dynamic-H3 fixture: {self:?}")
    }
}

impl std::error::Error for MichiganDynamicHexFoundationDecodeErrorV1 {}

/// Return the nine exact bounded plain-Git fixture parts.
#[must_use]
pub const fn michigan_dynamic_hex_foundation_fixture_parts_v1() -> [&'static [u8]; 9] {
    FIXTURE_PARTS
}

/// Decode one exact canonical artifact into the tick-owned checked value.
///
/// # Errors
/// Returns a closed refusal for length, digest, structured identity, or
/// tick-owned foundation violations.
pub fn decode_michigan_dynamic_hex_foundation_v1(
    bytes: &[u8],
) -> Result<MichiganDynamicHexFoundationV1, MichiganDynamicHexFoundationDecodeErrorV1> {
    if bytes.len() < MICHIGAN_DYNAMIC_HEX_FOUNDATION_BYTES_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::Truncated);
    }
    if bytes.len() > MICHIGAN_DYNAMIC_HEX_FOUNDATION_BYTES_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::TrailingBytes);
    }
    if sha256_of(bytes) != MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::ArtifactDigest);
    }

    let mut cursor = Cursor::new(bytes);
    if cursor.take(MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1.len())
        != MICHIGAN_DYNAMIC_HEX_FOUNDATION_DOMAIN_V1
    {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::Domain);
    }
    if cursor.u32() != MICHIGAN_DYNAMIC_HEX_FOUNDATION_LAYOUT_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::Layout);
    }
    let source_r7_digest = cursor.digest();
    let base_reference_cohort_digest = cursor.digest();
    let r8_section_digest = cursor.digest();
    let reference_bundle_digest = cursor.digest();
    let row_count = usize::try_from(cursor.u64())
        .map_err(|_| MichiganDynamicHexFoundationDecodeErrorV1::Truncated)?;
    if row_count != MICHIGAN_DYNAMIC_HEX_FOUNDATION_ROWS_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation(
            MichiganDynamicHexFoundationErrorV1::RowCount { actual: row_count },
        ));
    }

    let mut rows = Vec::new();
    rows.try_reserve_exact(row_count)
        .map_err(|_| MichiganDynamicHexFoundationDecodeErrorV1::Allocation)?;
    for _ in 0..row_count {
        let cell = H3CellId::try_from(cursor.u64())
            .map_err(MichiganDynamicHexFoundationDecodeErrorV1::H3Identity)?;
        let mut value_bits = [0_u64; 9];
        for bits in &mut value_bits {
            *bits = cursor.u64();
        }
        let values = MichiganDynamicHexValuesV1::try_new(MichiganDynamicHexValueBitsV1 {
            c: value_bits[0],
            v: value_bits[1],
            s: value_bits[2],
            k: value_bits[3],
            biocapacity_stock: value_bits[4],
            energy_stock: value_bits[5],
            raw_material_stock: value_bits[6],
            internet_access_pct: value_bits[7],
            surveillance_coupling: value_bits[8],
        })
        .map_err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation)?;
        rows.push(
            MichiganDynamicHexFoundationRowV1::try_new(cell, values)
                .map_err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation)?,
        );
    }
    if cursor.take(MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1.len())
        != MICHIGAN_DYNAMIC_HEX_R8_CHILD_PARENT_DOMAIN_V1
    {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::R8SectionDomain);
    }
    let r8_row_count = usize::try_from(cursor.u64())
        .map_err(|_| MichiganDynamicHexFoundationDecodeErrorV1::Truncated)?;
    if r8_row_count != MICHIGAN_DYNAMIC_HEX_R8_CHILD_ROWS_V1 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation(
            MichiganDynamicHexFoundationErrorV1::R8RowCount {
                actual: r8_row_count,
            },
        ));
    }
    let mut r8_child_parent_rows = Vec::new();
    r8_child_parent_rows
        .try_reserve_exact(r8_row_count)
        .map_err(|_| MichiganDynamicHexFoundationDecodeErrorV1::Allocation)?;
    for _ in 0..r8_row_count {
        let child = H3CellId::try_from(cursor.u64())
            .map_err(MichiganDynamicHexFoundationDecodeErrorV1::H3Identity)?;
        let parent = H3CellId::try_from(cursor.u64())
            .map_err(MichiganDynamicHexFoundationDecodeErrorV1::H3Identity)?;
        r8_child_parent_rows.push(
            MichiganH3R8ChildParentV1::try_new(child, parent)
                .map_err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation)?,
        );
    }
    if cursor.remaining() != 0 {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::TrailingBytes);
    }
    let foundation = MichiganDynamicHexFoundationV1::try_new(rows, r8_child_parent_rows)
        .map_err(MichiganDynamicHexFoundationDecodeErrorV1::Foundation)?;
    if foundation.source_r7_digest() != source_r7_digest
        || foundation.base_reference_cohort_digest() != base_reference_cohort_digest
        || foundation.r8_section_digest() != r8_section_digest
        || foundation.reference_bundle_digest() != reference_bundle_digest
        || foundation.canonical_bytes() != bytes
    {
        return Err(MichiganDynamicHexFoundationDecodeErrorV1::ArtifactDigest);
    }
    Ok(foundation)
}

/// Return the process-wide checked static Michigan foundation.
///
/// # Errors
/// Returns the cached exact decoding refusal if the checked-in fixture is not
/// the governed artifact.
pub fn michigan_dynamic_hex_foundation_v1(
) -> Result<&'static MichiganDynamicHexFoundationV1, MichiganDynamicHexFoundationDecodeErrorV1> {
    match FOUNDATION.get_or_init(|| {
        let mut joined = Vec::new();
        joined
            .try_reserve_exact(MICHIGAN_DYNAMIC_HEX_FOUNDATION_BYTES_V1)
            .map_err(|_| MichiganDynamicHexFoundationDecodeErrorV1::Allocation)?;
        for part in FIXTURE_PARTS {
            joined.extend_from_slice(part);
        }
        decode_michigan_dynamic_hex_foundation_v1(&joined)
    }) {
        Ok(foundation) => Ok(foundation),
        Err(error) => Err(error.clone()),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, count: usize) -> &'a [u8] {
        let end = self.offset + count;
        let output = &self.bytes[self.offset..end];
        self.offset = end;
        output
    }

    fn u32(&mut self) -> u32 {
        u32::from_be_bytes(
            self.take(4)
                .try_into()
                .expect("exact-length fixture was checked"),
        )
    }

    fn u64(&mut self) -> u64 {
        u64::from_be_bytes(
            self.take(8)
                .try_into()
                .expect("exact-length fixture was checked"),
        )
    }

    fn digest(&mut self) -> [u8; 32] {
        self.take(32)
            .try_into()
            .expect("exact-length fixture was checked")
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
}
