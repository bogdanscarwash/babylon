//! Checked, immutable PER-278 spatial reference products.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::{sha256_of, H3CellId};

use crate::{H3ReferenceCohort, H3ReferenceOrigin};

const FIXTURE_PARTS: [&[u8]; 3] = [
    include_bytes!("fixtures/spatial_reference_products_v1.part-00.bin"),
    include_bytes!("fixtures/spatial_reference_products_v1.part-01.bin"),
    include_bytes!("fixtures/spatial_reference_products_v1.part-02.bin"),
];
static FIXTURE: OnceLock<Box<[u8]>> = OnceLock::new();
const FIXTURE_MAGIC: &[u8; 16] = b"BABYLONSPATREF1\0";
const FIXTURE_VERSION: u32 = 1;
const EXPECTED_FIXTURE_DIGEST: [u8; 32] = [
    0xde, 0xa8, 0xa3, 0x68, 0xd5, 0xb7, 0xc5, 0xa0, 0xf1, 0x26, 0x3a, 0xf2, 0x94, 0x5f, 0xfd, 0xd9,
    0x4e, 0x39, 0x14, 0x39, 0x4e, 0xc5, 0x79, 0xca, 0x28, 0x50, 0x86, 0x61, 0xe4, 0x46, 0x44, 0x54,
];
const EXPECTED_COUNTS: [usize; 7] = [3_285, 745, 45_572, 22_509, 11_833, 31_881, 4_813];
const MAX_TEXT_BYTES: usize = 255;

/// Observed or derived authority carried by one installed product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceProductEvidenceClass {
    /// Identity and artifact facts read from a pinned source.
    Observed,
    /// A deterministic measure produced from pinned source facts.
    Derived,
}

impl ReferenceProductEvidenceClass {
    /// Stable SQL literal.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "Observed",
            Self::Derived => "Derived",
        }
    }
}

/// Exact provenance and measure framing for one governed product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceProduct {
    code: &'static str,
    artifact_sha256: RefDigestV1,
    semantic_sha256: Option<RefDigestV1>,
    row_count: u64,
    evidence_class: ReferenceProductEvidenceClass,
    measure_unit: &'static str,
    denominator: Option<&'static str>,
}

impl ReferenceProduct {
    /// Stable product identifier.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// SHA-256 of the exact source artifact bytes.
    #[must_use]
    pub const fn artifact_sha256(&self) -> RefDigestV1 {
        self.artifact_sha256
    }

    /// Container-independent digest when the predecessor published one.
    #[must_use]
    pub const fn semantic_sha256(&self) -> Option<RefDigestV1> {
        self.semantic_sha256
    }

    /// Exact row count.
    #[must_use]
    pub const fn row_count(&self) -> u64 {
        self.row_count
    }

    /// Governed evidence class.
    #[must_use]
    pub const fn evidence_class(&self) -> ReferenceProductEvidenceClass {
        self.evidence_class
    }

    /// Stable unit of the normalized relation.
    #[must_use]
    pub const fn measure_unit(&self) -> &'static str {
        self.measure_unit
    }

    /// Stable denominator when the measure is a fraction or share.
    #[must_use]
    pub const fn denominator(&self) -> Option<&'static str> {
        self.denominator
    }
}

/// One canonical nationwide county subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountyIdentityRow {
    county_id: u32,
    county_geoid: &'static str,
    state_id: u16,
    county_fips: &'static str,
    county_name: &'static str,
}

impl CountyIdentityRow {
    #[must_use]
    pub const fn county_id(&self) -> u32 {
        self.county_id
    }

    #[must_use]
    pub const fn county_geoid(&self) -> &'static str {
        self.county_geoid
    }

    #[must_use]
    pub const fn state_id(&self) -> u16 {
        self.state_id
    }

    #[must_use]
    pub const fn county_fips(&self) -> &'static str {
        self.county_fips
    }

    #[must_use]
    pub const fn county_name(&self) -> &'static str {
        self.county_name
    }
}

/// One canonical Michigan Census place subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceIdentityRow {
    place_geoid: &'static str,
    state_fips: &'static str,
    place_fips: &'static str,
    place_ns: &'static str,
    name: &'static str,
    name_lsad: &'static str,
    lsad: &'static str,
    class_fp: &'static str,
    principal_city_indicator: &'static str,
    mtfcc: &'static str,
    functional_status: &'static str,
}

macro_rules! place_getter {
    ($name:ident, $field:ident) => {
        #[must_use]
        pub const fn $name(&self) -> &'static str {
            self.$field
        }
    };
}

impl PlaceIdentityRow {
    place_getter!(place_geoid, place_geoid);
    place_getter!(state_fips, state_fips);
    place_getter!(place_fips, place_fips);
    place_getter!(place_ns, place_ns);
    place_getter!(name, name);
    place_getter!(name_lsad, name_lsad);
    place_getter!(lsad, lsad);
    place_getter!(class_fp, class_fp);
    place_getter!(principal_city_indicator, principal_city_indicator);
    place_getter!(mtfcc, mtfcc);
    place_getter!(functional_status, functional_status);
}

/// One fixed-scale land-fraction observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H3LandFractionRow {
    cell_id: H3CellId,
    source_county_geoid: &'static str,
    parts_per_million: u32,
}

impl H3LandFractionRow {
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    #[must_use]
    pub const fn source_county_geoid(&self) -> &'static str {
        self.source_county_geoid
    }

    #[must_use]
    pub const fn parts_per_million(&self) -> u32 {
        self.parts_per_million
    }
}

/// One positive count attached to a canonical H3 cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H3CountRow {
    cell_id: H3CellId,
    count: u64,
}

impl H3CountRow {
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }
}

/// One true county/H3 land-area slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountyH3LandAreaRow {
    cell_id: H3CellId,
    county_geoid: &'static str,
    land_area_m2: u64,
}

impl CountyH3LandAreaRow {
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    #[must_use]
    pub const fn county_geoid(&self) -> &'static str {
        self.county_geoid
    }

    #[must_use]
    pub const fn land_area_m2(&self) -> u64 {
        self.land_area_m2
    }
}

/// One true place/county/H3 land-area slice and its fixed denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountyPlaceH3LandAreaRow {
    cell_id: H3CellId,
    county_geoid: &'static str,
    place_geoid: &'static str,
    place_land_area_m2: u64,
    cell_mi_land_area_m2: u64,
    place_land_area_share_ppb: u32,
}

impl CountyPlaceH3LandAreaRow {
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    #[must_use]
    pub const fn county_geoid(&self) -> &'static str {
        self.county_geoid
    }

    #[must_use]
    pub const fn place_geoid(&self) -> &'static str {
        self.place_geoid
    }

    #[must_use]
    pub const fn place_land_area_m2(&self) -> u64 {
        self.place_land_area_m2
    }

    #[must_use]
    pub const fn cell_mi_land_area_m2(&self) -> u64 {
        self.cell_mi_land_area_m2
    }

    #[must_use]
    pub const fn place_land_area_share_ppb(&self) -> u32 {
        self.place_land_area_share_ppb
    }
}

/// One checked immutable bundle ready for an exact-epoch installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialReferenceProducts {
    ref_digest: RefDigestV1,
    products: Box<[ReferenceProduct]>,
    counties: Box<[CountyIdentityRow]>,
    places: Box<[PlaceIdentityRow]>,
    land_fractions: Box<[H3LandFractionRow]>,
    population_counts: Box<[H3CountRow]>,
    workplace_counts: Box<[H3CountRow]>,
    county_land_areas: Box<[CountyH3LandAreaRow]>,
    county_place_land_areas: Box<[CountyPlaceH3LandAreaRow]>,
}

impl SpatialReferenceProducts {
    #[must_use]
    pub const fn ref_digest(&self) -> RefDigestV1 {
        self.ref_digest
    }

    #[must_use]
    pub fn products(&self) -> &[ReferenceProduct] {
        &self.products
    }

    #[must_use]
    pub fn counties(&self) -> &[CountyIdentityRow] {
        &self.counties
    }

    #[must_use]
    pub fn places(&self) -> &[PlaceIdentityRow] {
        &self.places
    }

    #[must_use]
    pub fn land_fractions(&self) -> &[H3LandFractionRow] {
        &self.land_fractions
    }

    #[must_use]
    pub fn population_counts(&self) -> &[H3CountRow] {
        &self.population_counts
    }

    #[must_use]
    pub fn workplace_counts(&self) -> &[H3CountRow] {
        &self.workplace_counts
    }

    #[must_use]
    pub fn county_land_areas(&self) -> &[CountyH3LandAreaRow] {
        &self.county_land_areas
    }

    #[must_use]
    pub fn county_place_land_areas(&self) -> &[CountyPlaceH3LandAreaRow] {
        &self.county_place_land_areas
    }
}

/// Closed fixture or cross-product refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialReferenceProductsError {
    FixtureDigest,
    FixtureMagic,
    FixtureVersion {
        actual: u32,
    },
    FixtureRefDigest,
    FixtureCount {
        section: usize,
        expected: usize,
        actual: usize,
    },
    FixtureTruncated {
        field: &'static str,
    },
    FixtureText {
        field: &'static str,
    },
    FixtureTrailingBytes {
        actual: usize,
    },
    InvalidCell {
        field: &'static str,
        raw: u64,
    },
    NonDirectCell {
        field: &'static str,
        cell: H3CellId,
    },
    InvalidOrder {
        section: &'static str,
    },
    InvalidSubject {
        section: &'static str,
    },
    InvalidMeasure {
        section: &'static str,
    },
    ArithmeticOverflow {
        section: &'static str,
    },
}

impl std::fmt::Display for SpatialReferenceProductsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid spatial reference products: {self:?}")
    }
}

impl std::error::Error for SpatialReferenceProductsError {}

/// Decode and cross-check the exact Michigan reference-product fixture.
///
/// # Errors
/// Refuses byte drift, malformed framing, unknown H3 identities, subject drift,
/// measure drift, or conservation failure before returning any publishable rows.
pub fn michigan_spatial_reference_products_v1(
    cohort: &H3ReferenceCohort,
) -> Result<SpatialReferenceProducts, SpatialReferenceProductsError> {
    let fixture = fixture_bytes();
    if sha256_of(fixture) != EXPECTED_FIXTURE_DIGEST {
        return Err(SpatialReferenceProductsError::FixtureDigest);
    }
    let mut reader = FixtureReader::new(fixture);
    if reader.array::<16>("magic")? != *FIXTURE_MAGIC {
        return Err(SpatialReferenceProductsError::FixtureMagic);
    }
    let version = reader.u32("version")?;
    if version != FIXTURE_VERSION {
        return Err(SpatialReferenceProductsError::FixtureVersion { actual: version });
    }
    let ref_digest = RefDigestV1::from_bytes(reader.array::<32>("ref_digest")?);
    if ref_digest != cohort.receipt().ref_digest() {
        return Err(SpatialReferenceProductsError::FixtureRefDigest);
    }
    let counts = read_counts(&mut reader)?;
    let direct_cells = direct_resolution_seven_cells(cohort);
    let counties = read_counties(&mut reader, counts[0])?;
    let county_geoids = counties
        .iter()
        .map(CountyIdentityRow::county_geoid)
        .collect::<BTreeSet<_>>();
    let places = read_places(&mut reader, counts[1])?;
    let place_geoids = places
        .iter()
        .map(PlaceIdentityRow::place_geoid)
        .collect::<BTreeSet<_>>();
    let land_fractions =
        read_land_fractions(&mut reader, counts[2], &direct_cells, &county_geoids)?;
    if land_fractions.len() != direct_cells.len() {
        return Err(SpatialReferenceProductsError::InvalidSubject {
            section: "land_fractions",
        });
    }
    let population_counts = read_counts_section(
        &mut reader,
        counts[3],
        "population_counts",
        &direct_cells,
        10_066_869,
    )?;
    let workplace_counts = read_counts_section(
        &mut reader,
        counts[4],
        "workplace_counts",
        &direct_cells,
        3_931_809,
    )?;
    let (county_land_areas, denominators, county_keys) =
        read_county_land_areas(&mut reader, counts[5], &direct_cells, &county_geoids)?;
    let county_place_land_areas = read_county_place_land_areas(
        &mut reader,
        counts[6],
        &county_keys,
        &place_geoids,
        &denominators,
    )?;
    if reader.remaining() != 0 {
        return Err(SpatialReferenceProductsError::FixtureTrailingBytes {
            actual: reader.remaining(),
        });
    }
    Ok(SpatialReferenceProducts {
        ref_digest,
        products: products().into(),
        counties: counties.into_boxed_slice(),
        places: places.into_boxed_slice(),
        land_fractions: land_fractions.into_boxed_slice(),
        population_counts: population_counts.into_boxed_slice(),
        workplace_counts: workplace_counts.into_boxed_slice(),
        county_land_areas: county_land_areas.into_boxed_slice(),
        county_place_land_areas: county_place_land_areas.into_boxed_slice(),
    })
}

fn fixture_bytes() -> &'static [u8] {
    FIXTURE
        .get_or_init(|| {
            let capacity = FIXTURE_PARTS.iter().map(|part| part.len()).sum();
            let mut fixture = Vec::with_capacity(capacity);
            for part in FIXTURE_PARTS {
                fixture.extend_from_slice(part);
            }
            fixture.into_boxed_slice()
        })
        .as_ref()
}

fn read_counts(
    reader: &mut FixtureReader<'static>,
) -> Result<[usize; 7], SpatialReferenceProductsError> {
    let mut counts = [0_usize; 7];
    for (index, expected) in EXPECTED_COUNTS.into_iter().enumerate() {
        let actual = usize::try_from(reader.u32("section count")?).map_err(|_| {
            SpatialReferenceProductsError::FixtureCount {
                section: index,
                expected,
                actual: usize::MAX,
            }
        })?;
        if actual != expected {
            return Err(SpatialReferenceProductsError::FixtureCount {
                section: index,
                expected,
                actual,
            });
        }
        counts[index] = actual;
    }
    Ok(counts)
}

fn direct_resolution_seven_cells(cohort: &H3ReferenceCohort) -> BTreeSet<H3CellId> {
    cohort
        .rows()
        .iter()
        .filter(|row| row.resolution() == 7 && row.origin() == H3ReferenceOrigin::Direct)
        .map(crate::H3ReferenceCellRow::cell_id)
        .collect()
}

fn read_counties(
    reader: &mut FixtureReader<'static>,
    count: usize,
) -> Result<Vec<CountyIdentityRow>, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior_geoid = None;
    let mut county_ids = BTreeSet::new();
    for _ in 0..count {
        let row = CountyIdentityRow {
            county_id: reader.u32("county_id")?,
            county_geoid: reader.ascii(5, "county_geoid")?,
            state_id: reader.u16("state_id")?,
            county_fips: reader.ascii(3, "county_fips")?,
            county_name: reader.framed_text("county_name")?,
        };
        if row.county_id == 0
            || row.state_id == 0
            || row.county_geoid.get(2..) != Some(row.county_fips)
            || prior_geoid.is_some_and(|prior| prior >= row.county_geoid)
            || !county_ids.insert(row.county_id)
        {
            return Err(SpatialReferenceProductsError::InvalidSubject {
                section: "counties",
            });
        }
        prior_geoid = Some(row.county_geoid);
        rows.push(row);
    }
    Ok(rows)
}

fn read_places(
    reader: &mut FixtureReader<'static>,
    count: usize,
) -> Result<Vec<PlaceIdentityRow>, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior_geoid = None;
    for _ in 0..count {
        let row = PlaceIdentityRow {
            place_geoid: reader.ascii(7, "place_geoid")?,
            state_fips: reader.ascii(2, "state_fips")?,
            place_fips: reader.ascii(5, "place_fips")?,
            place_ns: reader.ascii(8, "place_ns")?,
            name: reader.framed_text("place name")?,
            name_lsad: reader.framed_text("place name_lsad")?,
            lsad: reader.ascii(2, "lsad")?,
            class_fp: reader.ascii(2, "class_fp")?,
            principal_city_indicator: reader.ascii(1, "principal_city_indicator")?,
            mtfcc: reader.ascii(5, "mtfcc")?,
            functional_status: reader.ascii(1, "functional_status")?,
        };
        if row.state_fips != "26"
            || row.place_geoid.get(..2) != Some(row.state_fips)
            || row.place_geoid.get(2..) != Some(row.place_fips)
            || prior_geoid.is_some_and(|prior| prior >= row.place_geoid)
        {
            return Err(SpatialReferenceProductsError::InvalidSubject { section: "places" });
        }
        prior_geoid = Some(row.place_geoid);
        rows.push(row);
    }
    Ok(rows)
}

fn read_land_fractions(
    reader: &mut FixtureReader<'static>,
    count: usize,
    direct_cells: &BTreeSet<H3CellId>,
    county_geoids: &BTreeSet<&'static str>,
) -> Result<Vec<H3LandFractionRow>, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior = None;
    for _ in 0..count {
        let cell_id = reader.cell("land_fraction cell")?;
        let row = H3LandFractionRow {
            cell_id,
            source_county_geoid: reader.ascii(5, "source_county_geoid")?,
            parts_per_million: reader.u32("land_fraction_ppm")?,
        };
        require_direct(cell_id, "land_fractions", direct_cells)?;
        if row.parts_per_million > 1_000_000
            || !county_geoids.contains(row.source_county_geoid)
            || prior.is_some_and(|previous| previous >= cell_id)
        {
            return Err(SpatialReferenceProductsError::InvalidMeasure {
                section: "land_fractions",
            });
        }
        prior = Some(cell_id);
        rows.push(row);
    }
    Ok(rows)
}

fn read_counts_section(
    reader: &mut FixtureReader<'static>,
    count: usize,
    section: &'static str,
    direct_cells: &BTreeSet<H3CellId>,
    expected_total: u64,
) -> Result<Vec<H3CountRow>, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior = None;
    let mut total = 0_u64;
    for _ in 0..count {
        let row = H3CountRow {
            cell_id: reader.cell(section)?,
            count: reader.u64(section)?,
        };
        require_direct(row.cell_id, section, direct_cells)?;
        if row.count == 0 || prior.is_some_and(|previous| previous >= row.cell_id) {
            return Err(SpatialReferenceProductsError::InvalidMeasure { section });
        }
        prior = Some(row.cell_id);
        total = total
            .checked_add(row.count)
            .ok_or(SpatialReferenceProductsError::ArithmeticOverflow { section })?;
        rows.push(row);
    }
    if total != expected_total {
        return Err(SpatialReferenceProductsError::InvalidMeasure { section });
    }
    Ok(rows)
}

type CountyLandRead = (
    Vec<CountyH3LandAreaRow>,
    BTreeMap<H3CellId, u64>,
    BTreeSet<(H3CellId, &'static str)>,
);

fn read_county_land_areas(
    reader: &mut FixtureReader<'static>,
    count: usize,
    direct_cells: &BTreeSet<H3CellId>,
    county_geoids: &BTreeSet<&'static str>,
) -> Result<CountyLandRead, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior = None;
    let mut total = 0_u64;
    let mut denominators = BTreeMap::new();
    let mut keys = BTreeSet::new();
    for _ in 0..count {
        let row = CountyH3LandAreaRow {
            cell_id: reader.cell("county_land_areas")?,
            county_geoid: reader.ascii(5, "county_geoid")?,
            land_area_m2: reader.u64("land_area_m2")?,
        };
        require_direct(row.cell_id, "county_land_areas", direct_cells)?;
        let key = (row.cell_id, row.county_geoid);
        if row.land_area_m2 == 0
            || !county_geoids.contains(row.county_geoid)
            || prior.is_some_and(|previous| previous >= key)
        {
            return Err(SpatialReferenceProductsError::InvalidMeasure {
                section: "county_land_areas",
            });
        }
        prior = Some(key);
        keys.insert(key);
        let denominator = denominators.entry(row.cell_id).or_insert(0_u64);
        *denominator = denominator.checked_add(row.land_area_m2).ok_or(
            SpatialReferenceProductsError::ArithmeticOverflow {
                section: "county_land_areas",
            },
        )?;
        total = total.checked_add(row.land_area_m2).ok_or(
            SpatialReferenceProductsError::ArithmeticOverflow {
                section: "county_land_areas",
            },
        )?;
        rows.push(row);
    }
    if total != 146_426_246_267 {
        return Err(SpatialReferenceProductsError::InvalidMeasure {
            section: "county_land_areas",
        });
    }
    Ok((rows, denominators, keys))
}

fn read_county_place_land_areas(
    reader: &mut FixtureReader<'static>,
    count: usize,
    county_keys: &BTreeSet<(H3CellId, &'static str)>,
    place_geoids: &BTreeSet<&'static str>,
    denominators: &BTreeMap<H3CellId, u64>,
) -> Result<Vec<CountyPlaceH3LandAreaRow>, SpatialReferenceProductsError> {
    let mut rows = Vec::with_capacity(count);
    let mut prior = None;
    let mut total = 0_u64;
    let mut share_sums = BTreeMap::new();
    for _ in 0..count {
        let row = CountyPlaceH3LandAreaRow {
            cell_id: reader.cell("county_place_land_areas")?,
            county_geoid: reader.ascii(5, "county_geoid")?,
            place_geoid: reader.ascii(7, "place_geoid")?,
            place_land_area_m2: reader.u64("place_land_area_m2")?,
            cell_mi_land_area_m2: reader.u64("cell_mi_land_area_m2")?,
            place_land_area_share_ppb: reader.u32("place_land_area_share_ppb")?,
        };
        let key = (row.cell_id, row.county_geoid, row.place_geoid);
        let expected_denominator = denominators.get(&row.cell_id).copied();
        let numerator = u128::from(row.place_land_area_m2)
            .checked_mul(1_000_000_000)
            .ok_or(SpatialReferenceProductsError::ArithmeticOverflow {
                section: "county_place_land_areas",
            })?;
        let expected_share = if row.cell_mi_land_area_m2 == 0 {
            None
        } else {
            u32::try_from(numerator / u128::from(row.cell_mi_land_area_m2)).ok()
        };
        if row.place_land_area_m2 == 0
            || row.place_land_area_m2 > row.cell_mi_land_area_m2
            || expected_denominator != Some(row.cell_mi_land_area_m2)
            || expected_share != Some(row.place_land_area_share_ppb)
            || !county_keys.contains(&(row.cell_id, row.county_geoid))
            || !place_geoids.contains(row.place_geoid)
            || prior.is_some_and(|previous| previous >= key)
        {
            return Err(SpatialReferenceProductsError::InvalidMeasure {
                section: "county_place_land_areas",
            });
        }
        prior = Some(key);
        let share_sum = share_sums.entry(row.cell_id).or_insert(0_u64);
        *share_sum = share_sum
            .checked_add(u64::from(row.place_land_area_share_ppb))
            .ok_or(SpatialReferenceProductsError::ArithmeticOverflow {
                section: "county_place_land_areas",
            })?;
        if *share_sum > 1_000_000_000 {
            return Err(SpatialReferenceProductsError::InvalidMeasure {
                section: "county_place_land_areas",
            });
        }
        total = total.checked_add(row.place_land_area_m2).ok_or(
            SpatialReferenceProductsError::ArithmeticOverflow {
                section: "county_place_land_areas",
            },
        )?;
        rows.push(row);
    }
    if total != 7_689_548_061 {
        return Err(SpatialReferenceProductsError::InvalidMeasure {
            section: "county_place_land_areas",
        });
    }
    Ok(rows)
}

fn require_direct(
    cell: H3CellId,
    field: &'static str,
    direct_cells: &BTreeSet<H3CellId>,
) -> Result<(), SpatialReferenceProductsError> {
    if cell.resolution() != 7 || !direct_cells.contains(&cell) {
        return Err(SpatialReferenceProductsError::NonDirectCell { field, cell });
    }
    Ok(())
}

fn products() -> [ReferenceProduct; 7] {
    [
        product(
            "census_county_h3_land_overlap_mi_2023",
            "7054fe2efa378e4db055a6647b9a3834cc382d822a032652b33894732b55b3c3",
            Some("07b243a38a111a01c0c1f509d12e418549691c31b8024809efa82eafcbff7723"),
            31_881,
            ReferenceProductEvidenceClass::Derived,
            "square_metres",
            None,
        ),
        product(
            "census_county_place_h3_land_overlap_mi_2023",
            "fcb7baaf63a5422accce8709997de8e409936f7131fa0ef6b0a28762fdfee42f",
            Some("72b92c705c3d456b471d409bd9b55ece3741deb4fecca1db36768527efd5ea7b"),
            4_813,
            ReferenceProductEvidenceClass::Derived,
            "square_metres",
            Some("cell_michigan_land_area_m2"),
        ),
        product(
            "census_place_identity_mi_2023",
            "cb864b4f6f43902bb821e84fe9a4055a9039e0a74d8b8399f209ae6ed26a8be7",
            Some("cd354f40f798ea2a4bb94d7c4d638ebeb84c152250d5b97da8213c8b5115d47e"),
            745,
            ReferenceProductEvidenceClass::Observed,
            "identity",
            None,
        ),
        product(
            "dim_county",
            "130b7679d0441d5c3c2183a2bef858073d3011039550bfbf015b380566c72032",
            None,
            3_285,
            ReferenceProductEvidenceClass::Observed,
            "identity",
            None,
        ),
        product(
            "h3_res7_land_mask",
            "4e6caba297f0111a9ec93d948a83543bb9f7179361fe5dd318bb8a98a5be5194",
            Some("a896f04a878fa54d67e1d6b0609c93a246ae9f92732c0fd6f158bf36e613eb7c"),
            45_572,
            ReferenceProductEvidenceClass::Derived,
            "parts_per_million",
            Some("one_million"),
        ),
        product(
            "h3_res7_population",
            "b096a5891284f0ca55bedae9d1a9092eb8ea9e9e32d32b6ace430a9833b53afc",
            Some("64cfcb563f2d22c7511b96da3e4c8bd834d837035af32879c1adea1980a1e0c3"),
            22_509,
            ReferenceProductEvidenceClass::Derived,
            "count",
            None,
        ),
        product(
            "h3_res7_workplace",
            "ea2ce1508f4fe51f1e879b9f4a1daf579c4b00349388b12a85f884a8f49eabb6",
            Some("ab326a99673e03bfc5597d5e417a71f98ef2a5346a58df7b05a905b5ba7a03e7"),
            11_833,
            ReferenceProductEvidenceClass::Derived,
            "count",
            None,
        ),
    ]
}

fn product(
    code: &'static str,
    artifact_sha256: &'static str,
    semantic_sha256: Option<&'static str>,
    row_count: u64,
    evidence_class: ReferenceProductEvidenceClass,
    measure_unit: &'static str,
    denominator: Option<&'static str>,
) -> ReferenceProduct {
    ReferenceProduct {
        code,
        artifact_sha256: RefDigestV1::from_bytes(hex_digest(artifact_sha256)),
        semantic_sha256: semantic_sha256.map(|value| RefDigestV1::from_bytes(hex_digest(value))),
        row_count,
        evidence_class,
        measure_unit,
        denominator,
    }
}

fn hex_digest(value: &str) -> [u8; 32] {
    let bytes = value.as_bytes();
    let mut result = [0_u8; 32];
    for (index, output) in result.iter_mut().enumerate() {
        let high = hex_nibble(bytes[index * 2]);
        let low = hex_nibble(bytes[index * 2 + 1]);
        *output = high * 16 + low;
    }
    result
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => unreachable!("static product digest must be lowercase hexadecimal"),
    }
}

struct FixtureReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> FixtureReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn array<const N: usize>(
        &mut self,
        field: &'static str,
    ) -> Result<[u8; N], SpatialReferenceProductsError> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or(SpatialReferenceProductsError::FixtureTruncated { field })?;
        let source = self
            .bytes
            .get(self.offset..end)
            .ok_or(SpatialReferenceProductsError::FixtureTruncated { field })?;
        self.offset = end;
        source
            .try_into()
            .map_err(|_| SpatialReferenceProductsError::FixtureTruncated { field })
    }

    fn u16(&mut self, field: &'static str) -> Result<u16, SpatialReferenceProductsError> {
        self.array(field).map(u16::from_be_bytes)
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, SpatialReferenceProductsError> {
        self.array(field).map(u32::from_be_bytes)
    }

    fn u64(&mut self, field: &'static str) -> Result<u64, SpatialReferenceProductsError> {
        self.array(field).map(u64::from_be_bytes)
    }

    fn cell(&mut self, field: &'static str) -> Result<H3CellId, SpatialReferenceProductsError> {
        let raw = self.u64(field)?;
        H3CellId::try_from(raw)
            .map_err(|_| SpatialReferenceProductsError::InvalidCell { field, raw })
    }

    fn ascii(
        &mut self,
        length: usize,
        field: &'static str,
    ) -> Result<&'a str, SpatialReferenceProductsError> {
        let bytes = self.take(length, field)?;
        if !bytes.iter().all(u8::is_ascii_alphanumeric) {
            return Err(SpatialReferenceProductsError::FixtureText { field });
        }
        std::str::from_utf8(bytes).map_err(|_| SpatialReferenceProductsError::FixtureText { field })
    }

    fn framed_text(
        &mut self,
        field: &'static str,
    ) -> Result<&'a str, SpatialReferenceProductsError> {
        let length = usize::from(self.array::<1>(field)?[0]);
        if length == 0 || length > MAX_TEXT_BYTES {
            return Err(SpatialReferenceProductsError::FixtureText { field });
        }
        let bytes = self.take(length, field)?;
        let value = std::str::from_utf8(bytes)
            .map_err(|_| SpatialReferenceProductsError::FixtureText { field })?;
        if value.contains('\0') {
            return Err(SpatialReferenceProductsError::FixtureText { field });
        }
        Ok(value)
    }

    fn take(
        &mut self,
        length: usize,
        field: &'static str,
    ) -> Result<&'a [u8], SpatialReferenceProductsError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SpatialReferenceProductsError::FixtureTruncated { field })?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(SpatialReferenceProductsError::FixtureTruncated { field })?;
        self.offset = end;
        Ok(value)
    }
}
