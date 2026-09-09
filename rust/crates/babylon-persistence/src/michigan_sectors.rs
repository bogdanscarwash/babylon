//! Pinned observed county-sector cells, without enterprise or mechanics inference.

use std::fmt::Write as _;
use std::io::Read as _;
use std::sync::OnceLock;

use babylon_kernel::sha256_of;
use serde::{Deserialize, Serialize};

use crate::michigan_economy::digest_hex;

/// Exact compressed artifact identity from `QcewCountySectorsV1`.
pub const QCEW_SECTORS_ARTIFACT_SHA256_V1: &str =
    "1cac80bee20c086be2e1f268643b0caab5d8d63030251817a9f149123755d71a";
/// Exact typed-row identity, including disclosure and source lineage.
pub const QCEW_SECTORS_SEMANTIC_SHA256_V1: &str =
    "e117a5cbbe22e6a133c212cb254aaeeb543732344ca049de8a404eb4fd90e6d9";
/// Fixed public-record vintage, not the campaign's simulated year.
pub const QCEW_SECTORS_VINTAGE_V1: u16 = 2024;
const SOURCE_MANIFEST_SHA256: &str =
    "048c02b5890115e655e0a61553472adf2cbf8ef5c016731c0ffc0bfd0f2f667e";
const ARTIFACT: &[u8] = include_bytes!(
    "../../../../src/babylon/data/reference/economy/qcew_county_sectors_mi_2024.csv.gz"
);
const SOURCE_MANIFEST: &[u8] =
    include_bytes!("../../../../tools/qcew_county_economics_v1_source_manifest.json");
const MAX_BYTES: usize = 1_048_576;
const MAX_SOURCE_BYTES: usize = 131_072;
const MAX_ROWS: usize = 83 * 20;
const DOMAIN: &str = "babylon.qcew-county-sectors.v1\0";
const COLUMNS: [&str; 11] = [
    "county_geoid",
    "sector_code",
    "sector_title",
    "sector_disposition",
    "disclosure_code",
    "annual_avg_estabs_count",
    "annual_avg_emplvl",
    "total_annual_wages",
    "annual_avg_wkly_wage",
    "source_file",
    "source_sha256",
];
const SECTOR_CODES: [&str; 20] = [
    "11", "21", "22", "23", "31-33", "42", "44-45", "48-49", "51", "52", "53", "54", "55", "56",
    "61", "62", "71", "72", "81", "99",
];

/// One exact admitted code. Composite sectors retain their complete identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MichiganSectorCodeV1(&'static str);
impl MichiganSectorCodeV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
    #[must_use]
    pub fn disposition(self) -> MichiganSectorDispositionV1 {
        if self.0 == "99" {
            MichiganSectorDispositionV1::Unclassified
        } else {
            MichiganSectorDispositionV1::Classified
        }
    }
    fn parse(value: &str) -> Result<Self, MichiganSectorsErrorV1> {
        SECTOR_CODES
            .iter()
            .find(|code| **code == value)
            .copied()
            .map(Self)
            .ok_or(MichiganSectorsErrorV1::RowIdentity)
    }
}

/// Code 99 remains unclassified, without an invented classified membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganSectorDispositionV1 {
    Classified,
    Unclassified,
}
impl MichiganSectorDispositionV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classified => "classified",
            Self::Unclassified => "unclassified",
        }
    }
}

/// Establishments remain observed in suppressed cells; other measures are absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganSectorDisclosureV1 {
    Disclosed,
    Suppressed,
}
impl MichiganSectorDisclosureV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disclosed => "",
            Self::Suppressed => "N",
        }
    }
}

/// Immutable source cell. Jobs are annual averages, never allocated workers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganCountySectorV1 {
    county_geoid: String,
    sector_code: MichiganSectorCodeV1,
    sector_title: String,
    disclosure: MichiganSectorDisclosureV1,
    annual_avg_estabs_count: u64,
    annual_avg_emplvl: Option<u64>,
    total_annual_wages: Option<u64>,
    annual_avg_wkly_wage: Option<u64>,
    source_file: String,
    source_sha256: String,
}
impl MichiganCountySectorV1 {
    #[must_use]
    pub fn county_geoid(&self) -> &str {
        &self.county_geoid
    }
    #[must_use]
    pub const fn sector_code(&self) -> MichiganSectorCodeV1 {
        self.sector_code
    }
    #[must_use]
    pub fn sector_title(&self) -> &str {
        &self.sector_title
    }
    #[must_use]
    pub const fn disclosure(&self) -> MichiganSectorDisclosureV1 {
        self.disclosure
    }
    #[must_use]
    pub const fn annual_avg_estabs_count(&self) -> u64 {
        self.annual_avg_estabs_count
    }
    #[must_use]
    pub const fn annual_avg_emplvl(&self) -> Option<u64> {
        self.annual_avg_emplvl
    }
    #[must_use]
    pub const fn total_annual_wages(&self) -> Option<u64> {
        self.total_annual_wages
    }
    #[must_use]
    pub const fn annual_avg_wkly_wage(&self) -> Option<u64> {
        self.annual_avg_wkly_wage
    }
    #[must_use]
    pub fn source_file(&self) -> &str {
        &self.source_file
    }
    #[must_use]
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
}

/// Fully checked reference rows. A missing county-sector row does not mean zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MichiganCountySectorsV1 {
    rows: Vec<MichiganCountySectorV1>,
}
impl MichiganCountySectorsV1 {
    #[must_use]
    pub fn rows(&self) -> &[MichiganCountySectorV1] {
        &self.rows
    }
}

/// Closed admission failures, without dumping withheld cells or source contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MichiganSectorsErrorV1 {
    ArtifactDigest,
    SourceDigest,
    SourceManifest,
    ArtifactSize,
    ArtifactDecode,
    CsvShape,
    RowIdentity,
    Value,
    Disclosure,
    Provenance,
    Ordering,
    Coverage,
    SemanticDigest,
}
impl std::fmt::Display for MichiganSectorsErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Michigan county sectors refused: {self:?}")
    }
}
impl std::error::Error for MichiganSectorsErrorV1 {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceManifest {
    contract: String,
    version: u8,
    entries: Vec<SourcePin>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePin {
    file: String,
    sha256: String,
}

fn source_pins(raw: &[u8]) -> Result<Vec<SourcePin>, MichiganSectorsErrorV1> {
    if raw.len() > MAX_SOURCE_BYTES || digest_hex(&sha256_of(raw)) != SOURCE_MANIFEST_SHA256 {
        return Err(MichiganSectorsErrorV1::SourceDigest);
    }
    let manifest: SourceManifest =
        serde_json::from_slice(raw).map_err(|_| MichiganSectorsErrorV1::SourceManifest)?;
    if manifest.contract != "QcewCountyEconomicsV1"
        || manifest.version != 1
        || manifest.entries.len() != 83
    {
        return Err(MichiganSectorsErrorV1::SourceManifest);
    }
    for (index, pin) in manifest.entries.iter().enumerate() {
        let prefix = format!("2024.annual {} ", 26_001 + index * 2);
        if !pin.file.starts_with(&prefix)
            || !pin.file.ends_with(" County, Michigan.csv")
            || pin.sha256.len() != 64
            || !pin
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(MichiganSectorsErrorV1::SourceManifest);
        }
    }
    Ok(manifest.entries)
}

fn decode_gzip(raw: &[u8]) -> Result<String, MichiganSectorsErrorV1> {
    if raw.len() > MAX_BYTES {
        return Err(MichiganSectorsErrorV1::ArtifactSize);
    }
    if raw.len() < 10 || raw[..8] != [0x1f, 0x8b, 8, 0, 0, 0, 0, 0] {
        return Err(MichiganSectorsErrorV1::ArtifactDecode);
    }
    let mut decoder = flate2::bufread::GzDecoder::new(raw);
    let mut source_text = String::new();
    decoder
        .by_ref()
        .take(u64::try_from(MAX_BYTES).expect("bounded constant") + 1)
        .read_to_string(&mut source_text)
        .map_err(|_| MichiganSectorsErrorV1::ArtifactDecode)?;
    if source_text.len() > MAX_BYTES {
        return Err(MichiganSectorsErrorV1::ArtifactSize);
    }
    if !decoder.into_inner().is_empty() {
        return Err(MichiganSectorsErrorV1::ArtifactDecode);
    }
    Ok(source_text)
}

// The contract forbids control/newline characters within fields. Parse its
// single-line CSV records with quoted commas and doubled quotes, never split(',').
fn csv_record(line: &str) -> Result<Vec<String>, MichiganSectorsErrorV1> {
    let mut input = line.chars().peekable();
    let mut fields = Vec::with_capacity(COLUMNS.len());
    loop {
        if fields.len() == COLUMNS.len() {
            return Err(MichiganSectorsErrorV1::CsvShape);
        }
        let mut field = String::new();
        if input.peek() == Some(&'"') {
            input.next();
            loop {
                match input.next() {
                    Some('"') if input.peek() == Some(&'"') => {
                        input.next();
                        field.push('"');
                    }
                    Some('"') => break,
                    Some(value) if !value.is_control() => field.push(value),
                    _ => return Err(MichiganSectorsErrorV1::CsvShape),
                }
            }
        } else {
            while input.peek().is_some_and(|value| *value != ',') {
                let value = input.next().expect("peeked character");
                if value == '"' || value.is_control() {
                    return Err(MichiganSectorsErrorV1::CsvShape);
                }
                field.push(value);
            }
        }
        fields.push(field);
        match input.next() {
            Some(',') => {}
            None if fields.len() == COLUMNS.len() => return Ok(fields),
            _ => return Err(MichiganSectorsErrorV1::CsvShape),
        }
    }
}

fn integer(value: &str) -> Result<u64, MichiganSectorsErrorV1> {
    if value.is_empty()
        || value.len() > 19
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(MichiganSectorsErrorV1::Value);
    }
    let result = value
        .parse::<u64>()
        .map_err(|_| MichiganSectorsErrorV1::Value)?;
    i64::try_from(result).map_err(|_| MichiganSectorsErrorV1::Value)?;
    Ok(result)
}

fn county_index(geoid: &str) -> Result<usize, MichiganSectorsErrorV1> {
    let value = integer(geoid).map_err(|_| MichiganSectorsErrorV1::RowIdentity)?;
    if geoid.len() != 5 || !(26_001..=26_165).contains(&value) || value % 2 == 0 {
        return Err(MichiganSectorsErrorV1::RowIdentity);
    }
    usize::try_from((value - 26_001) / 2).map_err(|_| MichiganSectorsErrorV1::RowIdentity)
}

fn parse_row(
    fields: &[String],
    sources: &[SourcePin],
) -> Result<MichiganCountySectorV1, MichiganSectorsErrorV1> {
    let index = county_index(&fields[0])?;
    let sector_code = MichiganSectorCodeV1::parse(&fields[1])?;
    if fields[2].is_empty()
        || fields[2].chars().count() > 256
        || fields[2]
            .chars()
            .any(|ch| ch.is_control() || (ch.is_whitespace() && ch != ' '))
    {
        return Err(MichiganSectorsErrorV1::Value);
    }
    if fields[3] != sector_code.disposition().as_str() {
        return Err(MichiganSectorsErrorV1::RowIdentity);
    }
    let disclosure = match fields[4].as_str() {
        "" => MichiganSectorDisclosureV1::Disclosed,
        "N" => MichiganSectorDisclosureV1::Suppressed,
        _ => return Err(MichiganSectorsErrorV1::Disclosure),
    };
    let annual_avg_estabs_count = integer(&fields[5])?;
    if annual_avg_estabs_count == 0 {
        return Err(MichiganSectorsErrorV1::Value);
    }
    let measures = if disclosure == MichiganSectorDisclosureV1::Suppressed {
        if fields[6..9].iter().any(|field| !field.is_empty()) {
            return Err(MichiganSectorsErrorV1::Disclosure);
        }
        [None; 3]
    } else {
        [
            Some(integer(&fields[6])?),
            Some(integer(&fields[7])?),
            Some(integer(&fields[8])?),
        ]
    };
    let source = sources
        .get(index)
        .ok_or(MichiganSectorsErrorV1::Provenance)?;
    if fields[9] != source.file || fields[10] != source.sha256 {
        return Err(MichiganSectorsErrorV1::Provenance);
    }
    Ok(MichiganCountySectorV1 {
        county_geoid: fields[0].clone(),
        sector_code,
        sector_title: fields[2].clone(),
        disclosure,
        annual_avg_estabs_count,
        annual_avg_emplvl: measures[0],
        total_annual_wages: measures[1],
        annual_avg_wkly_wage: measures[2],
        source_file: source.file.clone(),
        source_sha256: source.sha256.clone(),
    })
}

fn ascii_json_line(
    output: &mut String,
    value: &impl Serialize,
) -> Result<(), MichiganSectorsErrorV1> {
    let json = serde_json::to_string(value).map_err(|_| MichiganSectorsErrorV1::SemanticDigest)?;
    for ch in json.chars() {
        if ch.is_ascii() && ch != '\u{007f}' {
            output.push(ch);
        } else {
            let mut units = [0; 2];
            for unit in ch.encode_utf16(&mut units) {
                write!(output, "\\u{unit:04x}").expect("String write");
            }
        }
    }
    output.push('\n');
    Ok(())
}

fn semantic_digest(rows: &[MichiganCountySectorV1]) -> Result<String, MichiganSectorsErrorV1> {
    let mut source = DOMAIN.to_owned();
    ascii_json_line(&mut source, &COLUMNS)?;
    for row in rows {
        ascii_json_line(
            &mut source,
            &(
                row.county_geoid(),
                row.sector_code().as_str(),
                row.sector_title(),
                row.sector_code().disposition().as_str(),
                row.disclosure().as_str(),
                row.annual_avg_estabs_count(),
                row.annual_avg_emplvl(),
                row.total_annual_wages(),
                row.annual_avg_wkly_wage(),
                row.source_file(),
                row.source_sha256(),
            ),
        )?;
    }
    Ok(digest_hex(&sha256_of(source.as_bytes())))
}

fn checked_csv(
    source: &str,
    pins: &[SourcePin],
) -> Result<MichiganCountySectorsV1, MichiganSectorsErrorV1> {
    if source.len() > MAX_BYTES {
        return Err(MichiganSectorsErrorV1::ArtifactSize);
    }
    if !source.ends_with('\n') || source.contains('\r') {
        return Err(MichiganSectorsErrorV1::CsvShape);
    }
    let mut lines = source.split_terminator('\n');
    if lines.next() != Some(COLUMNS.join(",").as_str()) {
        return Err(MichiganSectorsErrorV1::CsvShape);
    }
    let mut rows = Vec::<MichiganCountySectorV1>::with_capacity(MAX_ROWS);
    let mut counties = [false; 83];
    let mut suppressed = 0;
    let mut unclassified = 0;
    for line in lines {
        if rows.len() == MAX_ROWS {
            return Err(MichiganSectorsErrorV1::Coverage);
        }
        let row = parse_row(&csv_record(line)?, pins)?;
        if rows.last().is_some_and(|previous| {
            (previous.county_geoid(), previous.sector_code())
                >= (row.county_geoid(), row.sector_code())
        }) {
            return Err(MichiganSectorsErrorV1::Ordering);
        }
        counties[county_index(row.county_geoid())?] = true;
        suppressed += usize::from(row.disclosure() == MichiganSectorDisclosureV1::Suppressed);
        unclassified += usize::from(
            row.sector_code().disposition() == MichiganSectorDispositionV1::Unclassified,
        );
        rows.push(row);
    }
    if rows.len() != 1603
        || counties.iter().any(|seen| !seen)
        || suppressed != 416
        || unclassified != 81
    {
        return Err(MichiganSectorsErrorV1::Coverage);
    }
    if semantic_digest(&rows)? != QCEW_SECTORS_SEMANTIC_SHA256_V1 {
        return Err(MichiganSectorsErrorV1::SemanticDigest);
    }
    Ok(MichiganCountySectorsV1 { rows })
}

fn admit(raw: &[u8], manifest: &[u8]) -> Result<MichiganCountySectorsV1, MichiganSectorsErrorV1> {
    let pins = source_pins(manifest)?;
    if raw.len() > MAX_BYTES {
        return Err(MichiganSectorsErrorV1::ArtifactSize);
    }
    if digest_hex(&sha256_of(raw)) != QCEW_SECTORS_ARTIFACT_SHA256_V1 {
        return Err(MichiganSectorsErrorV1::ArtifactDigest);
    }
    checked_csv(&decode_gzip(raw)?, &pins)
}

/// Admit the exact observed artifact once, without reading a host acquisition directory.
/// # Errors
/// Refuses changed pins, invalid gzip/CSV, lost source lineage, missing coverage,
/// altered disclosure or noncanonical ordering and values.
pub fn michigan_county_sectors_v1(
) -> Result<&'static MichiganCountySectorsV1, MichiganSectorsErrorV1> {
    static SECTORS: OnceLock<Result<MichiganCountySectorsV1, MichiganSectorsErrorV1>> =
        OnceLock::new();
    SECTORS
        .get_or_init(|| admit(ARTIFACT, SOURCE_MANIFEST))
        .as_ref()
        .map_err(|error| *error)
}

#[cfg(test)]
mod tests;
