//! Bounded canonical content envelope around the existing exact material codec.

use super::{
    decode_material_circuit_state, encode_material_circuit_state, ProcessId, SectorBundle,
    SectorBundleError, SectorBundleGood, SectorBundleOwner, SectorBundleProcess,
    SectorBundleSources, StableElementKey, UnitId, BUNDLE_DOMAIN, BUNDLE_VERSION, MAX_BUNDLE_BYTES,
    MAX_BUNDLE_GOODS, MAX_BUNDLE_PROCESSES, MAX_BUNDLE_TEXT_BYTES, MICHIGAN_MAX_HORIZON_PERIODS,
};
use babylon_material_circuit::GoodId;

pub(super) fn encode(bundle: &SectorBundle) -> Result<Vec<u8>, SectorBundleError> {
    let mut bytes = BUNDLE_DOMAIN.to_vec();
    bytes.extend_from_slice(&BUNDLE_VERSION.to_be_bytes());
    let StableElementKey::Node {
        scenario,
        local_name,
    } = &bundle.owner.subject
    else {
        return Err(SectorBundleError::Owner);
    };
    for value in [
        scenario,
        local_name,
        &bundle.owner.county_geoid,
        &bundle.owner.sector_code,
    ] {
        text(&mut bytes, value)?;
    }
    bytes.extend_from_slice(&MICHIGAN_MAX_HORIZON_PERIODS.to_be_bytes());
    text(&mut bytes, &bundle.sources.county_source_file)?;
    for digest in [
        bundle.sources.county_source_sha256,
        bundle.sources.sector_artifact_sha256,
        bundle.sources.sector_semantic_sha256,
        bundle.sources.industry_artifact_sha256,
        bundle.sources.designed_scenario_sha256,
    ] {
        bytes.extend_from_slice(&digest);
    }
    bytes.extend_from_slice(&bundle.labor_unit.as_bytes());
    count(&mut bytes, bundle.goods.len())?;
    for good in &bundle.goods {
        bytes.extend_from_slice(&good.good_id.as_bytes());
        bytes.extend_from_slice(&good.unit_id.as_bytes());
    }
    count(&mut bytes, bundle.processes.len())?;
    for process in &bundle.processes {
        bytes.extend_from_slice(&process.process_id.as_bytes());
        text(&mut bytes, &process.industry_code)?;
    }
    let rows = encode_material_circuit_state(&bundle.rows)?;
    let length = u32::try_from(rows.len()).map_err(|_| SectorBundleError::Bound)?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(&rows);
    if bytes.len() > MAX_BUNDLE_BYTES {
        return Err(SectorBundleError::Bound);
    }
    Ok(bytes)
}

fn count(bytes: &mut Vec<u8>, length: usize) -> Result<(), SectorBundleError> {
    bytes.extend_from_slice(
        &u16::try_from(length)
            .map_err(|_| SectorBundleError::Bound)?
            .to_be_bytes(),
    );
    Ok(())
}

fn text(bytes: &mut Vec<u8>, value: &str) -> Result<(), SectorBundleError> {
    if value.is_empty() || value.len() > MAX_BUNDLE_TEXT_BYTES || value.as_bytes().contains(&0) {
        return Err(SectorBundleError::Bound);
    }
    count(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

pub(super) fn decode(bytes: &[u8]) -> Result<SectorBundle, SectorBundleError> {
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(BUNDLE_DOMAIN.len())? != BUNDLE_DOMAIN {
        return Err(SectorBundleError::WireDomain);
    }
    if u16::from_be_bytes(cursor.array()?) != BUNDLE_VERSION {
        return Err(SectorBundleError::WireVersion);
    }
    let owner = SectorBundleOwner {
        subject: StableElementKey::Node {
            scenario: cursor.text()?,
            local_name: cursor.text()?,
        },
        county_geoid: cursor.text()?,
        sector_code: cursor.text()?,
    };
    if u64::from_be_bytes(cursor.array()?) != MICHIGAN_MAX_HORIZON_PERIODS {
        return Err(SectorBundleError::Resource);
    }
    let sources = SectorBundleSources {
        county_source_file: cursor.text()?,
        county_source_sha256: cursor.array()?,
        sector_artifact_sha256: cursor.array()?,
        sector_semantic_sha256: cursor.array()?,
        industry_artifact_sha256: cursor.array()?,
        designed_scenario_sha256: cursor.array()?,
    };
    let labor_unit = UnitId::from_bytes(cursor.array()?);
    let mut goods = Vec::new();
    for _ in 0..cursor.count(MAX_BUNDLE_GOODS)? {
        goods.push(SectorBundleGood {
            good_id: GoodId::from_bytes(cursor.array()?),
            unit_id: UnitId::from_bytes(cursor.array()?),
        });
    }
    let mut processes = Vec::new();
    for _ in 0..cursor.count_allow_zero(MAX_BUNDLE_PROCESSES)? {
        processes.push(SectorBundleProcess {
            process_id: ProcessId::from_bytes(cursor.array()?),
            industry_code: cursor.text()?,
        });
    }
    let row_bytes = usize::try_from(u32::from_be_bytes(cursor.array()?))
        .map_err(|_| SectorBundleError::Bound)?;
    let rows = decode_material_circuit_state(cursor.take(row_bytes)?)?;
    if cursor.offset != bytes.len() {
        return Err(SectorBundleError::WireTrailing);
    }
    let result = SectorBundle::from_parts(owner, sources, goods, processes, labor_unit, &rows)?;
    if result.canonical_bytes() != bytes {
        return Err(SectorBundleError::WireNoncanonical);
    }
    Ok(result)
}

pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    pub(super) fn finished(&self) -> bool {
        self.offset == self.bytes.len()
    }
    pub(super) fn take(&mut self, length: usize) -> Result<&'a [u8], SectorBundleError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SectorBundleError::Bound)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(SectorBundleError::WireTruncated)?;
        self.offset = end;
        Ok(value)
    }
    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], SectorBundleError> {
        self.take(N)?
            .try_into()
            .map_err(|_| SectorBundleError::WireTruncated)
    }
    pub(super) fn count_allow_zero(&mut self, bound: usize) -> Result<usize, SectorBundleError> {
        let count = usize::from(u16::from_be_bytes(self.array()?));
        if count > bound {
            return Err(SectorBundleError::Bound);
        }
        Ok(count)
    }
    pub(super) fn count(&mut self, bound: usize) -> Result<usize, SectorBundleError> {
        let count = usize::from(u16::from_be_bytes(self.array()?));
        if count == 0 || count > bound {
            return Err(SectorBundleError::Bound);
        }
        Ok(count)
    }
    fn text(&mut self) -> Result<String, SectorBundleError> {
        let count = self.count(MAX_BUNDLE_TEXT_BYTES)?;
        let bytes = self.take(count)?;
        if bytes.contains(&0) {
            return Err(SectorBundleError::Bound);
        }
        String::from_utf8(bytes.to_vec()).map_err(|_| SectorBundleError::Bound)
    }
}
