use crate::{
    validate_draft, AudienceV1, CoverageV1, DecisionSurfaceV1, DimensionCoordinateV1, DurabilityV1,
    DyadKindV1, DyadV1, EvidenceClassV1, FacetFamilyV1, FacetV1, FlowKindV1, GapReasonV1, GapV1,
    HyperedgeKindV1, HyperedgeV1, MembershipKindV1, ProvenanceV1, ReferenceDigestV1,
    ReferenceFlowV1, RelationalTerritoryDossierV1, RtdDossierDraftV1, RtdError, ScaleMembershipV1,
    StatusV1, TypedIdentityV1, ValueKindV1, RTD_MAX_CANONICAL_BYTES,
};
use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::fmt;
use unicode_normalization::UnicodeNormalization;

const MAX_COLLECTION_ITEMS: usize = 65_535;
const MAX_VECTOR_BYTES: usize = 1_048_576;
const MAX_VECTOR_LINES: usize = 256;
const MAX_VECTOR_LINE_BYTES: usize = 262_144;
const MAX_CASE_ID_BYTES: usize = 128;
const MAX_JSON_DEPTH: usize = 32;
const IDENTITY_COMPONENTS: usize = 3;
const ERROR_REGISTRY_ITEMS: usize = 20;
const MAX_CANONICAL_STRING_BYTES: usize = 67_108_864;
const SHA256_WIDTH: usize = 32;
const HASH_DOMAIN: &[u8] = b"babylon.relational-territory-dossier.v1";

/// One closed line from the shared RTD V1 conformance corpus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RtdVectorCaseV1 {
    /// A valid draft with exact canonical bytes and projection hash.
    Valid {
        /// Stable NFC case identity.
        case_id: String,
        /// Fully consumed JSON bytes for the draft object.
        draft_json: Vec<u8>,
        /// Lowercase canonical UTF-8 bytes encoded as hex.
        canonical_utf8_hex: String,
        /// Lowercase domain-separated SHA-256.
        projection_hash: String,
    },
    /// An invalid draft or bounded-reader taxonomy witness.
    Invalid {
        /// Stable NFC case identity.
        case_id: String,
        /// Fully consumed JSON bytes for the draft object.
        draft_json: Vec<u8>,
        /// Exact stable refusal identity.
        error: String,
    },
}

struct UniqueVectorValue(Value);
struct UniqueVectorValueVisitor;

impl<'de> Deserialize<'de> for UniqueVectorValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVectorValueVisitor)
    }
}

impl<'de> Visitor<'de> for UniqueVectorValueVisitor {
    type Value = UniqueVectorValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded vector JSON without duplicate keys")
    }

    fn visit_bool<E: DeError>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::Bool(value)))
    }

    fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::Number(value.into())))
    }

    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::Number(value.into())))
    }

    fn visit_f64<E: DeError>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value).map_or_else(
            || Err(E::custom("non-finite vector number")),
            |number| Ok(UniqueVectorValue(Value::Number(number))),
        )
    }

    fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::String(value.to_owned())))
    }

    fn visit_string<E: DeError>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::String(value)))
    }

    fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::Null))
    }

    fn visit_unit<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(UniqueVectorValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut output = Vec::new();
        for _index in 0..=MAX_VECTOR_BYTES {
            match sequence.next_element::<UniqueVectorValue>()? {
                Some(value) => output.push(value.0),
                None => return Ok(UniqueVectorValue(Value::Array(output))),
            }
        }
        Err(A::Error::custom("RTD_VECTOR_LIMIT"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut output = Map::new();
        for _index in 0..=MAX_VECTOR_BYTES {
            let Some(key) = map.next_key::<String>()? else {
                return Ok(UniqueVectorValue(Value::Object(output)));
            };
            if output.contains_key(&key) {
                return Err(A::Error::custom("RTD_DUPLICATE_KEY"));
            }
            output.insert(key, map.next_value::<UniqueVectorValue>()?.0);
        }
        Err(A::Error::custom("RTD_VECTOR_LIMIT"))
    }
}

/// Parses the closed shared JSONL corpus with exact raw and structural bounds.
///
/// # Errors
///
/// Returns the stable vector, JSON, duplicate-key, NFC, or unknown-field
/// refusal before exposing a partial case collection.
pub fn parse_vector_corpus(payload: &[u8]) -> Result<Vec<RtdVectorCaseV1>, RtdError> {
    if payload.len() > MAX_VECTOR_BYTES {
        return Err(RtdError::VectorLimit);
    }
    let mut output = Vec::new();
    let mut case_ids = Vec::new();
    for (line_index, line) in payload
        .split_inclusive(|byte| *byte == b'\n')
        .take(257)
        .enumerate()
    {
        if line_index == MAX_VECTOR_LINES {
            return Err(RtdError::VectorLimit);
        }
        let case = parse_vector_line(line)?;
        let case_id = vector_case_id(&case);
        for existing_index in 0..MAX_VECTOR_LINES {
            if existing_index == case_ids.len() {
                break;
            }
            if case_ids[existing_index] == case_id {
                return Err(RtdError::DuplicateKey);
            }
        }
        case_ids.push(case_id.to_owned());
        output.push(case);
    }
    Ok(output)
}

fn vector_case_id(case: &RtdVectorCaseV1) -> &str {
    match case {
        RtdVectorCaseV1::Valid { case_id, .. } | RtdVectorCaseV1::Invalid { case_id, .. } => {
            case_id
        }
    }
}

fn parse_vector_line(line: &[u8]) -> Result<RtdVectorCaseV1, RtdError> {
    if line.len() > MAX_VECTOR_LINE_BYTES {
        return Err(RtdError::VectorLimit);
    }
    let line = match line.strip_suffix(b"\n") {
        Some(value) => value,
        None => line,
    };
    let line = match line.strip_suffix(b"\r") {
        Some(value) => value,
        None => line,
    };
    scan_vector_depth(line)?;
    let value = serde_json::from_slice::<UniqueVectorValue>(line)
        .map_err(|error| classify_vector_json_error(&error))?
        .0;
    let Value::Object(mut object) = value else {
        return Err(RtdError::Json);
    };
    let case_id = take_case_id(&mut object)?;
    let kind = take_string(&mut object, "kind")?;
    if kind == "valid" {
        parse_valid_vector(object, case_id)
    } else if kind == "invalid" {
        parse_invalid_vector(object, case_id)
    } else {
        Err(RtdError::VectorLimit)
    }
}

fn parse_valid_vector(
    mut object: Map<String, Value>,
    case_id: String,
) -> Result<RtdVectorCaseV1, RtdError> {
    if object.len() != 3 {
        return Err(RtdError::UnknownField);
    }
    let draft_json = take_draft_json(&mut object)?;
    let canonical_utf8_hex = take_string(&mut object, "canonical_utf8_hex")?;
    let projection_hash = take_string(&mut object, "projection_hash")?;
    if !object.is_empty()
        || canonical_utf8_hex.len() % 2 != 0
        || !is_lower_hex(&canonical_utf8_hex)
        || projection_hash.len() != 64
        || !is_lower_hex(&projection_hash)
    {
        return Err(RtdError::Json);
    }
    Ok(RtdVectorCaseV1::Valid {
        case_id,
        draft_json,
        canonical_utf8_hex,
        projection_hash,
    })
}

fn parse_invalid_vector(
    mut object: Map<String, Value>,
    case_id: String,
) -> Result<RtdVectorCaseV1, RtdError> {
    if object.len() != 2 {
        return Err(RtdError::UnknownField);
    }
    let draft_json = take_draft_json(&mut object)?;
    let error = take_string(&mut object, "error")?;
    if !object.is_empty() || !is_registered_error(&error) {
        return Err(RtdError::Enum);
    }
    Ok(RtdVectorCaseV1::Invalid {
        case_id,
        draft_json,
        error,
    })
}

fn take_case_id(object: &mut Map<String, Value>) -> Result<String, RtdError> {
    let case_id = take_string(object, "case_id")?;
    let length = case_id.len();
    if length == 0 || length > MAX_CASE_ID_BYTES || !case_id.nfc().eq(case_id.chars()) {
        return Err(RtdError::VectorLimit);
    }
    Ok(case_id)
}

fn take_string(object: &mut Map<String, Value>, key: &str) -> Result<String, RtdError> {
    object
        .remove(key)
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .ok_or(RtdError::Json)
}

fn take_draft_json(object: &mut Map<String, Value>) -> Result<Vec<u8>, RtdError> {
    let draft = object.remove("draft").ok_or(RtdError::Json)?;
    if !draft.is_object() {
        return Err(RtdError::Json);
    }
    serde_json::to_vec(&draft).map_err(|_error| RtdError::Json)
}

fn classify_vector_json_error(error: &serde_json::Error) -> RtdError {
    let message = error.to_string();
    if message.contains("RTD_DUPLICATE_KEY") {
        RtdError::DuplicateKey
    } else if message.contains("RTD_VECTOR_LIMIT") {
        RtdError::VectorLimit
    } else {
        RtdError::Json
    }
}

fn is_registered_error(error: &str) -> bool {
    for index in 0..ERROR_REGISTRY_ITEMS {
        if crate::RTD_V1_ERROR_REGISTRY[index] == error {
            return true;
        }
    }
    false
}

fn scan_vector_depth(payload: &[u8]) -> Result<(), RtdError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for index in 0..=MAX_VECTOR_LINE_BYTES {
        if index == payload.len() {
            return Ok(());
        }
        let byte = payload[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'{' || byte == b'[' {
            depth = depth.checked_add(1).ok_or(RtdError::JsonDepth)?;
            if depth > MAX_JSON_DEPTH {
                return Err(RtdError::JsonDepth);
            }
        } else if byte == b'}' || byte == b']' {
            depth = depth.checked_sub(1).ok_or(RtdError::Json)?;
        }
    }
    Err(RtdError::VectorLimit)
}

fn is_lower_hex(value: &str) -> bool {
    let bytes = value.as_bytes();
    for index in 0..MAX_VECTOR_LINE_BYTES {
        if index == bytes.len() {
            return true;
        }
        let byte = bytes[index];
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return false;
        }
    }
    false
}

struct CanonicalWriter {
    bytes: Vec<u8>,
    count: u64,
}

impl CanonicalWriter {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            count: 0,
        }
    }

    fn write(&mut self, value: &[u8]) -> Result<(), RtdError> {
        let length = u64::try_from(value.len()).map_err(|_error| RtdError::CanonicalSize)?;
        let next = self
            .count
            .checked_add(length)
            .ok_or(RtdError::CanonicalSize)?;
        if next > RTD_MAX_CANONICAL_BYTES {
            return Err(RtdError::CanonicalSize);
        }
        self.bytes.extend_from_slice(value);
        self.count = next;
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

fn write_string(writer: &mut CanonicalWriter, value: &str) -> Result<(), RtdError> {
    let bytes = value.as_bytes();
    writer.write(b"\"")?;
    for index in 0..=MAX_CANONICAL_STRING_BYTES {
        if index == bytes.len() {
            break;
        }
        match bytes[index] {
            b'\x08' => writer.write(b"\\b")?,
            b'\t' => writer.write(b"\\t")?,
            b'\n' => writer.write(b"\\n")?,
            b'\x0c' => writer.write(b"\\f")?,
            b'\r' => writer.write(b"\\r")?,
            b'"' => writer.write(b"\\\"")?,
            b'\\' => writer.write(b"\\\\")?,
            byte @ 0x00..=0x1f => {
                let escaped = format!("\\u{byte:04x}");
                writer.write(escaped.as_bytes())?;
            }
            byte => writer.write(&[byte])?,
        }
    }
    writer.write(b"\"")
}

fn write_field(writer: &mut CanonicalWriter, key: &str, first: bool) -> Result<bool, RtdError> {
    if !first {
        writer.write(b",")?;
    }
    write_string(writer, key)?;
    writer.write(b":")?;
    Ok(false)
}

fn write_u64(writer: &mut CanonicalWriter, value: u64) -> Result<(), RtdError> {
    writer.write(value.to_string().as_bytes())
}

fn write_optional_string(
    writer: &mut CanonicalWriter,
    value: Option<&str>,
) -> Result<(), RtdError> {
    if let Some(text) = value {
        write_string(writer, text)
    } else {
        writer.write(b"null")
    }
}

// The repository's fixed-loop rule requires the literal three-component bound;
// an iterator rewrite would hide that proof from the static source check.
#[allow(clippy::needless_range_loop)]
fn identity_key(identity: &TypedIdentityV1) -> Vec<u8> {
    let mut output = Vec::new();
    let components = [&identity.domain, &identity.authority, &identity.local_id];
    for index in 0..IDENTITY_COMPONENTS {
        let bytes = components[index].as_bytes();
        let length = u16::try_from(bytes.len()).unwrap_or(u16::MAX);
        output.extend_from_slice(&length.to_be_bytes());
        output.extend_from_slice(bytes);
    }
    output
}

fn write_identity(
    writer: &mut CanonicalWriter,
    identity: &TypedIdentityV1,
) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "authority", true)?;
    write_string(writer, &identity.authority)?;
    let first = write_field(writer, "domain", first)?;
    write_string(writer, &identity.domain)?;
    let _first = write_field(writer, "local_id", first)?;
    write_string(writer, &identity.local_id)?;
    writer.write(b"}")
}

fn write_optional_identity(
    writer: &mut CanonicalWriter,
    identity: Option<&TypedIdentityV1>,
) -> Result<(), RtdError> {
    if let Some(value) = identity {
        write_identity(writer, value)
    } else {
        writer.write(b"null")
    }
}

fn write_array<T: Clone>(
    writer: &mut CanonicalWriter,
    items: &[T],
    emit: fn(&mut CanonicalWriter, &T) -> Result<(), RtdError>,
    sort_key: Option<fn(&T) -> Vec<u8>>,
) -> Result<(), RtdError> {
    let mut ordered = items.to_vec();
    if let Some(key) = sort_key {
        ordered.sort_by_key(key);
    }
    writer.write(b"[")?;
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == ordered.len() {
            break;
        }
        if index != 0 {
            writer.write(b",")?;
        }
        emit(writer, &ordered[index])?;
    }
    writer.write(b"]")
}

fn write_identity_array(
    writer: &mut CanonicalWriter,
    items: &[TypedIdentityV1],
    preserve_order: bool,
) -> Result<(), RtdError> {
    write_array(
        writer,
        items,
        write_identity,
        if preserve_order {
            None
        } else {
            Some(identity_key)
        },
    )
}

trait CanonicalEnum {
    fn literal(&self) -> &'static str;
}

macro_rules! canonical_enum {
    ($kind:ty, {$($variant:path => $literal:literal),+ $(,)?}) => {
        impl CanonicalEnum for $kind {
            fn literal(&self) -> &'static str {
                match self { $($variant => $literal),+ }
            }
        }
    };
}

canonical_enum!(AudienceV1, {AudienceV1::AdminMaterial => "ADMIN_MATERIAL", AudienceV1::PlayerKnowledge => "PLAYER_KNOWLEDGE"});
canonical_enum!(DurabilityV1, {DurabilityV1::InMemory => "IN_MEMORY", DurabilityV1::Committed => "COMMITTED"});
canonical_enum!(EvidenceClassV1, {EvidenceClassV1::Observed => "Observed", EvidenceClassV1::Derived => "Derived", EvidenceClassV1::Calibrated => "Calibrated", EvidenceClassV1::Designed => "Designed"});
canonical_enum!(StatusV1, {StatusV1::Present => "PRESENT", StatusV1::Absent => "ABSENT", StatusV1::Unknown => "UNKNOWN", StatusV1::NotComputed => "NOT_COMPUTED", StatusV1::Redacted => "REDACTED"});
canonical_enum!(ValueKindV1, {ValueKindV1::Uint64Bits => "UINT64_BITS", ValueKindV1::Float64Bits => "FLOAT64_BITS"});
canonical_enum!(CoverageV1, {CoverageV1::Complete => "COMPLETE", CoverageV1::Partial => "PARTIAL", CoverageV1::NotApplicable => "NOT_APPLICABLE", CoverageV1::Unknown => "UNKNOWN"});
canonical_enum!(MembershipKindV1, {MembershipKindV1::Administrative => "ADMINISTRATIVE", MembershipKindV1::National => "NATIONAL", MembershipKindV1::CommutingZone => "COMMUTING_ZONE", MembershipKindV1::Metropolitan => "METROPOLITAN", MembershipKindV1::WeightedOverlap => "WEIGHTED_OVERLAP"});
canonical_enum!(FacetFamilyV1, {FacetFamilyV1::CommandAdministration => "COMMAND_ADMINISTRATION", FacetFamilyV1::ProductionCirculation => "PRODUCTION_CIRCULATION", FacetFamilyV1::ReproductionSettlementAccess => "REPRODUCTION_SETTLEMENT_ACCESS", FacetFamilyV1::ExtractionAbandonmentCarceral => "EXTRACTION_ABANDONMENT_CARCERAL", FacetFamilyV1::EcologyCare => "ECOLOGY_CARE", FacetFamilyV1::OrganizationRootedness => "ORGANIZATION_ROOTEDNESS"});
canonical_enum!(DyadKindV1, {DyadKindV1::Presence => "PRESENCE", DyadKindV1::Membership => "MEMBERSHIP", DyadKindV1::Solidarity => "SOLIDARITY", DyadKindV1::Command => "COMMAND"});
canonical_enum!(HyperedgeKindV1, {HyperedgeKindV1::PublicRelation => "PUBLIC_RELATION"});
canonical_enum!(FlowKindV1, {FlowKindV1::CommuterJobs => "COMMUTER_JOBS", FlowKindV1::BorderSynthesis => "BORDER_SYNTHESIS"});
canonical_enum!(GapReasonV1, {GapReasonV1::MissingGovernedOmbDelineation => "MISSING_GOVERNED_OMB_DELINEATION", GapReasonV1::IdentityContractPending => "IDENTITY_CONTRACT_PENDING", GapReasonV1::MissingGovernedProducer => "MISSING_GOVERNED_PRODUCER", GapReasonV1::ReferenceCoverageUnavailable => "REFERENCE_COVERAGE_UNAVAILABLE", GapReasonV1::PlayerBoundaryUnavailable => "PLAYER_BOUNDARY_UNAVAILABLE", GapReasonV1::ProvenanceCoordinateConflict => "PROVENANCE_COORDINATE_CONFLICT"});

fn write_enum<T: CanonicalEnum>(writer: &mut CanonicalWriter, value: &T) -> Result<(), RtdError> {
    write_string(writer, value.literal())
}

fn write_reference(writer: &mut CanonicalWriter, row: &ReferenceDigestV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "artifact_schema_id_or_null", true)?;
    write_optional_identity(writer, row.artifact_schema_id_or_null.as_ref())?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "reference_id", first)?;
    write_identity(writer, &row.reference_id)?;
    let first = write_field(writer, "sha256_hex", first)?;
    write_string(writer, &row.sha256_hex)?;
    let _first = write_field(writer, "vintage", first)?;
    write_string(writer, &row.vintage)?;
    writer.write(b"}")
}

fn reference_key(row: &ReferenceDigestV1) -> Vec<u8> {
    identity_key(&row.reference_id)
}
fn membership_key(row: &ScaleMembershipV1) -> Vec<u8> {
    identity_key(&row.membership_id)
}
fn facet_key(row: &FacetV1) -> Vec<u8> {
    identity_key(&row.facet_id)
}
fn dyad_key(row: &DyadV1) -> Vec<u8> {
    identity_key(&row.relation_id)
}
fn hyperedge_key(row: &HyperedgeV1) -> Vec<u8> {
    identity_key(&row.hyperedge_id)
}
fn flow_key(row: &ReferenceFlowV1) -> Vec<u8> {
    identity_key(&row.flow_id)
}
fn gap_key(row: &GapV1) -> Vec<u8> {
    identity_key(&row.gap_id)
}
fn provenance_key(row: &ProvenanceV1) -> Vec<u8> {
    identity_key(&row.provenance_id)
}
fn coordinate_key(row: &DimensionCoordinateV1) -> Vec<u8> {
    identity_key(&row.dimension_ref)
}

fn write_coordinate(
    writer: &mut CanonicalWriter,
    row: &DimensionCoordinateV1,
) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "dimension_ref", true)?;
    write_identity(writer, &row.dimension_ref)?;
    let _first = write_field(writer, "member_ref", first)?;
    write_identity(writer, &row.member_ref)?;
    writer.write(b"}")
}

fn write_membership(writer: &mut CanonicalWriter, row: &ScaleMembershipV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "coverage", true)?;
    write_enum(writer, &row.coverage)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "member_ref", first)?;
    write_identity(writer, &row.member_ref)?;
    let first = write_field(writer, "membership_id", first)?;
    write_identity(writer, &row.membership_id)?;
    let first = write_field(writer, "membership_kind", first)?;
    write_enum(writer, &row.membership_kind)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let first = write_field(writer, "scale_ref", first)?;
    write_identity(writer, &row.scale_ref)?;
    let first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    let first = write_field(writer, "weight_bits_or_null", first)?;
    write_optional_string(writer, row.weight_bits_or_null.as_deref())?;
    let _first = write_field(writer, "weight_status", first)?;
    write_enum(writer, &row.weight_status)?;
    writer.write(b"}")
}

fn write_facet(writer: &mut CanonicalWriter, row: &FacetV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "coordinates", true)?;
    write_array(
        writer,
        &row.coordinates,
        write_coordinate,
        Some(coordinate_key),
    )?;
    let first = write_field(writer, "coverage", first)?;
    write_enum(writer, &row.coverage)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "facet_id", first)?;
    write_identity(writer, &row.facet_id)?;
    let first = write_field(writer, "family", first)?;
    write_enum(writer, &row.family)?;
    let first = write_field(writer, "metric_id", first)?;
    write_identity(writer, &row.metric_id)?;
    let first = write_field(writer, "native_scale", first)?;
    write_identity(writer, &row.native_scale)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    let first = write_field(writer, "subject_ref", first)?;
    write_identity(writer, &row.subject_ref)?;
    let first = write_field(writer, "unit_id", first)?;
    write_identity(writer, &row.unit_id)?;
    let first = write_field(writer, "value_bits_or_null", first)?;
    write_optional_string(writer, row.value_bits_or_null.as_deref())?;
    let first = write_field(writer, "value_kind", first)?;
    write_enum(writer, &row.value_kind)?;
    let _first = write_field(writer, "vintage", first)?;
    write_string(writer, &row.vintage)?;
    writer.write(b"}")
}

fn write_dyad(writer: &mut CanonicalWriter, row: &DyadV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "coverage", true)?;
    write_enum(writer, &row.coverage)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "from_ref", first)?;
    write_identity(writer, &row.from_ref)?;
    let first = write_field(writer, "native_scale", first)?;
    write_identity(writer, &row.native_scale)?;
    let first = write_field(writer, "payload_facets", first)?;
    write_identity_array(writer, &row.payload_facets, false)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let first = write_field(writer, "relation_id", first)?;
    write_identity(writer, &row.relation_id)?;
    let first = write_field(writer, "relation_kind", first)?;
    write_enum(writer, &row.relation_kind)?;
    let first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    let _first = write_field(writer, "to_ref", first)?;
    write_identity(writer, &row.to_ref)?;
    writer.write(b"}")
}

fn write_hyperedge(writer: &mut CanonicalWriter, row: &HyperedgeV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "coverage", true)?;
    write_enum(writer, &row.coverage)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "hyperedge_id", first)?;
    write_identity(writer, &row.hyperedge_id)?;
    let first = write_field(writer, "hyperedge_kind", first)?;
    write_enum(writer, &row.hyperedge_kind)?;
    let first = write_field(writer, "member_refs", first)?;
    write_identity_array(writer, &row.member_refs, false)?;
    let first = write_field(writer, "native_scale", first)?;
    write_identity(writer, &row.native_scale)?;
    let first = write_field(writer, "payload_facets", first)?;
    write_identity_array(writer, &row.payload_facets, false)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let _first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    writer.write(b"}")
}

fn write_flow(writer: &mut CanonicalWriter, row: &ReferenceFlowV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "coverage", true)?;
    write_enum(writer, &row.coverage)?;
    let first = write_field(writer, "destination_ref", first)?;
    write_identity(writer, &row.destination_ref)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "flow_id", first)?;
    write_identity(writer, &row.flow_id)?;
    let first = write_field(writer, "flow_kind", first)?;
    write_enum(writer, &row.flow_kind)?;
    let first = write_field(writer, "native_scale", first)?;
    write_identity(writer, &row.native_scale)?;
    let first = write_field(writer, "origin_ref", first)?;
    write_identity(writer, &row.origin_ref)?;
    let first = write_field(writer, "payload_facets", first)?;
    write_identity_array(writer, &row.payload_facets, false)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let _first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    writer.write(b"}")
}

fn write_gap(writer: &mut CanonicalWriter, row: &GapV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "gap_id", true)?;
    write_identity(writer, &row.gap_id)?;
    let first = write_field(writer, "provenance_refs", first)?;
    write_identity_array(writer, &row.provenance_refs, false)?;
    let first = write_field(writer, "reason_code", first)?;
    write_enum(writer, &row.reason_code)?;
    let first = write_field(writer, "requested_metric_or_relation", first)?;
    write_identity(writer, &row.requested_metric_or_relation)?;
    let first = write_field(writer, "required_producer_or_null", first)?;
    write_optional_string(writer, row.required_producer_or_null.as_deref())?;
    let _first = write_field(writer, "status", first)?;
    write_enum(writer, &row.status)?;
    writer.write(b"}")
}

fn write_provenance(writer: &mut CanonicalWriter, row: &ProvenanceV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "artifact_digest", true)?;
    write_string(writer, &row.artifact_digest)?;
    let first = write_field(writer, "evidence_class", first)?;
    write_enum(writer, &row.evidence_class)?;
    let first = write_field(writer, "locator", first)?;
    write_string(writer, &row.locator)?;
    let first = write_field(writer, "provenance_id", first)?;
    write_identity(writer, &row.provenance_id)?;
    let first = write_field(writer, "transformation_digest_or_null", first)?;
    write_optional_string(writer, row.transformation_digest_or_null.as_deref())?;
    let _first = write_field(writer, "vintage", first)?;
    write_string(writer, &row.vintage)?;
    writer.write(b"}")
}

fn write_decision_surface(
    writer: &mut CanonicalWriter,
    row: &DecisionSurfaceV1,
) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "action_refs", true)?;
    write_identity_array(writer, &row.action_refs, true)?;
    let first = write_field(writer, "archive_subject_refs", first)?;
    write_identity_array(writer, &row.archive_subject_refs, true)?;
    let first = write_field(writer, "question_id", first)?;
    write_identity(writer, &row.question_id)?;
    let first = write_field(writer, "receipt_refs", first)?;
    write_identity_array(writer, &row.receipt_refs, true)?;
    let _first = write_field(writer, "signal_refs", first)?;
    write_identity_array(writer, &row.signal_refs, true)?;
    writer.write(b"}")
}

fn write_draft(writer: &mut CanonicalWriter, draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    writer.write(b"{")?;
    let first = write_field(writer, "actor", true)?;
    write_optional_identity(writer, draft.actor.as_ref())?;
    let first = write_field(writer, "audience", first)?;
    write_enum(writer, &draft.audience)?;
    let first = write_field(writer, "decision_surface", first)?;
    write_decision_surface(writer, &draft.decision_surface)?;
    let first = write_field(writer, "definitions_digest", first)?;
    write_string(writer, &draft.definitions_digest)?;
    let first = write_field(writer, "durability", first)?;
    write_enum(writer, &draft.durability)?;
    let first = write_field(writer, "dyads", first)?;
    write_array(writer, &draft.dyads, write_dyad, Some(dyad_key))?;
    let first = write_field(writer, "facets", first)?;
    write_array(writer, &draft.facets, write_facet, Some(facet_key))?;
    let first = write_field(writer, "flows", first)?;
    write_array(writer, &draft.flows, write_flow, Some(flow_key))?;
    let first = write_field(writer, "focus", first)?;
    write_identity_array(writer, &draft.focus, false)?;
    let first = write_field(writer, "fog_policy_digest", first)?;
    write_optional_string(writer, draft.fog_policy_digest.as_deref())?;
    let first = write_field(writer, "gaps", first)?;
    write_array(writer, &draft.gaps, write_gap, Some(gap_key))?;
    let first = write_field(writer, "graph_state_hash", first)?;
    write_string(writer, &draft.graph_state_hash)?;
    let first = write_field(writer, "hyperedges", first)?;
    write_array(
        writer,
        &draft.hyperedges,
        write_hyperedge,
        Some(hyperedge_key),
    )?;
    let first = write_field(writer, "knowledge_context_digest", first)?;
    write_optional_string(writer, draft.knowledge_context_digest.as_deref())?;
    let first = write_field(writer, "nominal_world_hash", first)?;
    write_string(writer, &draft.nominal_world_hash)?;
    let first = write_field(writer, "projection_version", first)?;
    write_u64(writer, u64::from(draft.projection_version))?;
    let first = write_field(writer, "provenance", first)?;
    write_array(
        writer,
        &draft.provenance,
        write_provenance,
        Some(provenance_key),
    )?;
    let first = write_field(writer, "reference_digests", first)?;
    write_array(
        writer,
        &draft.reference_digests,
        write_reference,
        Some(reference_key),
    )?;
    let first = write_field(writer, "scale_memberships", first)?;
    write_array(
        writer,
        &draft.scale_memberships,
        write_membership,
        Some(membership_key),
    )?;
    let first = write_field(writer, "schema", first)?;
    write_string(writer, &draft.schema)?;
    let first = write_field(writer, "schema_version", first)?;
    write_u64(writer, u64::from(draft.schema_version))?;
    let first = write_field(writer, "template_digest", first)?;
    write_string(writer, &draft.template_digest)?;
    let _first = write_field(writer, "verified_tick", first)?;
    write_u64(writer, draft.verified_tick)?;
    writer.write(b"}")
}

fn normalize_negative_zero(draft: &mut RtdDossierDraftV1) {
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == draft.facets.len() {
            break;
        }
        let facet = &mut draft.facets[index];
        if facet.value_kind == ValueKindV1::Float64Bits
            && facet.value_bits_or_null.as_deref() == Some("8000000000000000")
        {
            facet.value_bits_or_null = Some("0000000000000000".to_owned());
        }
    }
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == draft.scale_memberships.len() {
            break;
        }
        if draft.scale_memberships[index]
            .weight_bits_or_null
            .as_deref()
            == Some("8000000000000000")
        {
            draft.scale_memberships[index].weight_bits_or_null =
                Some("0000000000000000".to_owned());
        }
    }
}

/// Encodes one validated draft as bounded canonical JSON bytes.
///
/// # Errors
///
/// Returns the exact semantic or canonical-size refusal without exposing
/// partial bytes.
pub fn canonical_draft_bytes(draft: &RtdDossierDraftV1) -> Result<Vec<u8>, RtdError> {
    let mut normalized = draft.clone();
    normalize_negative_zero(&mut normalized);
    validate_draft(&normalized)?;
    let mut writer = CanonicalWriter::new();
    write_draft(&mut writer, &normalized)?;
    Ok(writer.finish())
}

/// Hashes one validated canonical draft with the RTD V1 domain separator.
///
/// # Errors
///
/// Returns the first validation or canonical-size refusal before hashing.
pub fn projection_hash(draft: &RtdDossierDraftV1) -> Result<[u8; 32], RtdError> {
    let canonical = canonical_draft_bytes(draft)?;
    let capacity = HASH_DOMAIN
        .len()
        .checked_add(1)
        .and_then(|size| size.checked_add(canonical.len()))
        .ok_or(RtdError::CanonicalSize)?;
    let mut input = Vec::with_capacity(capacity);
    input.extend_from_slice(HASH_DOMAIN);
    input.push(0);
    input.extend_from_slice(&canonical);
    Ok(babylon_kernel::sha256_of(&input))
}

// Keep the SHA-256 width visible as a fixed source-level loop bound.
#[allow(clippy::needless_range_loop)]
fn digest_hex(digest: &[u8; 32]) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(64);
    for index in 0..SHA256_WIDTH {
        let _ = write!(output, "{:02x}", digest[index]);
    }
    output
}

/// Seals one complete draft only after successful canonical hashing.
///
/// # Errors
///
/// Returns the first validation or canonical-size refusal and no sealed value.
pub fn seal_draft(mut draft: RtdDossierDraftV1) -> Result<RelationalTerritoryDossierV1, RtdError> {
    normalize_negative_zero(&mut draft);
    let digest = projection_hash(&draft)?;
    Ok(RelationalTerritoryDossierV1 {
        schema: draft.schema,
        schema_version: draft.schema_version,
        projection_version: draft.projection_version,
        audience: draft.audience,
        durability: draft.durability,
        verified_tick: draft.verified_tick,
        graph_state_hash: draft.graph_state_hash,
        nominal_world_hash: draft.nominal_world_hash,
        reference_digests: draft.reference_digests,
        definitions_digest: draft.definitions_digest,
        template_digest: draft.template_digest,
        fog_policy_digest: draft.fog_policy_digest,
        knowledge_context_digest: draft.knowledge_context_digest,
        actor: draft.actor,
        focus: draft.focus,
        scale_memberships: draft.scale_memberships,
        facets: draft.facets,
        dyads: draft.dyads,
        hyperedges: draft.hyperedges,
        flows: draft.flows,
        gaps: draft.gaps,
        provenance: draft.provenance,
        decision_surface: draft.decision_surface,
        projection_hash: digest_hex(&digest),
    })
}
