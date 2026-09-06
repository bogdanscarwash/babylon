//! Bounded canonical content envelope around the existing exact material codec.

use super::{
    decode_material_circuit_state_v2, encode_material_circuit_state_v2, ProcessIdV1,
    SectorBundleErrorV1, SectorBundleGoodV1, SectorBundleOwnerV1, SectorBundleProcessV1,
    SectorBundleSourcesV1, SectorBundleV1, StableElementKeyV1, UnitIdV1, BUNDLE_DOMAIN,
    BUNDLE_VERSION, HORIZON_TICKS, MAX_BUNDLE_BYTES, MAX_BUNDLE_GOODS, MAX_BUNDLE_PROCESSES,
    MAX_BUNDLE_TEXT_BYTES,
};
use babylon_material_circuit::GoodIdV1;

pub(super) fn encode(bundle: &SectorBundleV1) -> Result<Vec<u8>, SectorBundleErrorV1> {
    let mut bytes = BUNDLE_DOMAIN.to_vec();
    bytes.extend_from_slice(&BUNDLE_VERSION.to_be_bytes());
    let StableElementKeyV1::Node {
        scenario,
        local_name,
    } = &bundle.owner.subject
    else {
        return Err(SectorBundleErrorV1::Owner);
    };
    for value in [
        scenario,
        local_name,
        &bundle.owner.county_geoid,
        &bundle.owner.sector_code,
    ] {
        text(&mut bytes, value)?;
    }
    bytes.extend_from_slice(&HORIZON_TICKS.to_be_bytes());
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
    let rows = encode_material_circuit_state_v2(&bundle.rows)?;
    let length = u32::try_from(rows.len()).map_err(|_| SectorBundleErrorV1::Bound)?;
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes.extend_from_slice(&rows);
    if bytes.len() > MAX_BUNDLE_BYTES {
        return Err(SectorBundleErrorV1::Bound);
    }
    Ok(bytes)
}

fn count(bytes: &mut Vec<u8>, length: usize) -> Result<(), SectorBundleErrorV1> {
    bytes.extend_from_slice(
        &u16::try_from(length)
            .map_err(|_| SectorBundleErrorV1::Bound)?
            .to_be_bytes(),
    );
    Ok(())
}

fn text(bytes: &mut Vec<u8>, value: &str) -> Result<(), SectorBundleErrorV1> {
    if value.is_empty() || value.len() > MAX_BUNDLE_TEXT_BYTES || value.as_bytes().contains(&0) {
        return Err(SectorBundleErrorV1::Bound);
    }
    count(bytes, value.len())?;
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

pub(super) fn decode(bytes: &[u8]) -> Result<SectorBundleV1, SectorBundleErrorV1> {
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(BUNDLE_DOMAIN.len())? != BUNDLE_DOMAIN {
        return Err(SectorBundleErrorV1::WireDomain);
    }
    if u16::from_be_bytes(cursor.array()?) != BUNDLE_VERSION {
        return Err(SectorBundleErrorV1::WireVersion);
    }
    let owner = SectorBundleOwnerV1 {
        subject: StableElementKeyV1::Node {
            scenario: cursor.text()?,
            local_name: cursor.text()?,
        },
        county_geoid: cursor.text()?,
        sector_code: cursor.text()?,
    };
    if u64::from_be_bytes(cursor.array()?) != HORIZON_TICKS {
        return Err(SectorBundleErrorV1::Resource);
    }
    let sources = SectorBundleSourcesV1 {
        county_source_file: cursor.text()?,
        county_source_sha256: cursor.array()?,
        sector_artifact_sha256: cursor.array()?,
        sector_semantic_sha256: cursor.array()?,
        industry_artifact_sha256: cursor.array()?,
        designed_scenario_sha256: cursor.array()?,
    };
    let labor_unit = UnitIdV1::from_bytes(cursor.array()?);
    let mut goods = Vec::new();
    for _ in 0..cursor.count(MAX_BUNDLE_GOODS)? {
        goods.push(SectorBundleGoodV1 {
            good_id: GoodIdV1::from_bytes(cursor.array()?),
            unit_id: UnitIdV1::from_bytes(cursor.array()?),
        });
    }
    let mut processes = Vec::new();
    for _ in 0..cursor.count(MAX_BUNDLE_PROCESSES)? {
        processes.push(SectorBundleProcessV1 {
            process_id: ProcessIdV1::from_bytes(cursor.array()?),
            industry_code: cursor.text()?,
        });
    }
    let row_bytes = usize::try_from(u32::from_be_bytes(cursor.array()?))
        .map_err(|_| SectorBundleErrorV1::Bound)?;
    let rows = decode_material_circuit_state_v2(cursor.take(row_bytes)?)?;
    if cursor.offset != bytes.len() {
        return Err(SectorBundleErrorV1::WireTrailing);
    }
    let result = SectorBundleV1::from_parts(owner, sources, goods, processes, labor_unit, &rows)?;
    if result.canonical_bytes() != bytes {
        return Err(SectorBundleErrorV1::WireNoncanonical);
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
    pub(super) fn take(&mut self, length: usize) -> Result<&'a [u8], SectorBundleErrorV1> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(SectorBundleErrorV1::Bound)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(SectorBundleErrorV1::WireTruncated)?;
        self.offset = end;
        Ok(value)
    }
    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], SectorBundleErrorV1> {
        self.take(N)?
            .try_into()
            .map_err(|_| SectorBundleErrorV1::WireTruncated)
    }
    pub(super) fn count(&mut self, bound: usize) -> Result<usize, SectorBundleErrorV1> {
        let count = usize::from(u16::from_be_bytes(self.array()?));
        if count == 0 || count > bound {
            return Err(SectorBundleErrorV1::Bound);
        }
        Ok(count)
    }
    fn text(&mut self) -> Result<String, SectorBundleErrorV1> {
        let count = self.count(MAX_BUNDLE_TEXT_BYTES)?;
        let bytes = self.take(count)?;
        if bytes.contains(&0) {
            return Err(SectorBundleErrorV1::Bound);
        }
        String::from_utf8(bytes.to_vec()).map_err(|_| SectorBundleErrorV1::Bound)
    }
}
