use crate::{
    AudienceV1, DurabilityV1, DyadKindV1, DyadV1, EvidenceClassV1, FacetV1, FlowKindV1,
    GapReasonV1, MembershipKindV1, MetricRepresentationV1, ReferenceFlowV1, RelationPayloadModeV1,
    RtdCollectionKindV1, RtdDossierDraftV1, RtdMetricRegistryRowV1,
    RtdRelationBindingRegistryRowV1, StatusV1, TypedIdentityLiteralV1, TypedIdentityV1,
    ValueKindV1, RTD_MAX_COORDINATES, RTD_MAX_DECISION_SURFACE_REFS, RTD_MAX_DYADS, RTD_MAX_FACETS,
    RTD_MAX_FLOWS, RTD_MAX_FOCUS, RTD_MAX_GAPS, RTD_MAX_HYPEREDGES, RTD_MAX_HYPEREDGE_MEMBERS,
    RTD_MAX_PAYLOAD_FACETS, RTD_MAX_PROVENANCE, RTD_MAX_PROVENANCE_REFS, RTD_MAX_REFERENCE_DIGESTS,
    RTD_MAX_SCALE_MEMBERSHIPS, RTD_V1_ERROR_REGISTRY, RTD_V1_METRIC_REGISTRY,
    RTD_V1_RELATION_BINDING_REGISTRY, RTD_V1_SCHEMA_ID,
};
use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use unicode_normalization::UnicodeNormalization;

const MAX_COLLECTION_ITEMS: usize = 65_535;
const MAX_FOCUS: usize = 64;
const MAX_REFERENCE_DIGESTS: usize = 4_096;
const MAX_SCALE_MEMBERSHIPS: usize = 65_535;
const MAX_FACETS: usize = 65_535;
const MAX_DYADS: usize = 65_535;
const MAX_HYPEREDGES: usize = 65_535;
const MAX_FLOWS: usize = 65_535;
const MAX_GAPS: usize = 65_535;
const MAX_PROVENANCE: usize = 65_535;
const MAX_COORDINATES: usize = 32;
const MAX_HYPEREDGE_MEMBERS: usize = 1_024;
const MAX_PAYLOAD_FACETS: usize = 256;
const MAX_DISPLAY_REFS: usize = 256;
const MAX_PROVENANCE_REFS: usize = 8_192;
const MAX_IDENTITY_BYTES: usize = 256;
const MAX_VINTAGE_BYTES: usize = 256;
const MAX_LOCATOR_BYTES: usize = 1_024;
const MAX_PRODUCER_BYTES: usize = 64;
const MAX_JSON_BYTES: usize = 67_108_864;
const MAX_JSON_DEPTH: usize = 32;
const METRIC_ROWS: usize = 18;
const BINDING_ROWS: usize = 6;

type IdentityKey = (String, String, String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtdError {
    Json,
    JsonDepth,
    SchemaVersion,
    UnknownField,
    Enum,
    Identity,
    Digest,
    NonNfc,
    LimitExceeded,
    DuplicateKey,
    DanglingReference,
    StatusValue,
    NativeGrain,
    UnsupportedDownscale,
    H3BeforePer21,
    MsaEvidence,
    CanadaControl,
    ForbiddenReduction,
    VectorLimit,
    CanonicalSize,
}

impl RtdError {
    fn code(self) -> &'static str {
        RTD_V1_ERROR_REGISTRY[self.registry_index()]
    }

    const fn registry_index(self) -> usize {
        match self {
            Self::Json => 0,
            Self::JsonDepth => 1,
            Self::SchemaVersion => 2,
            Self::UnknownField => 3,
            Self::Enum => 4,
            Self::Identity => 5,
            Self::Digest => 6,
            Self::NonNfc => 7,
            Self::LimitExceeded => 8,
            Self::DuplicateKey => 9,
            Self::DanglingReference => 10,
            Self::StatusValue => 11,
            Self::NativeGrain => 12,
            Self::UnsupportedDownscale => 13,
            Self::H3BeforePer21 => 14,
            Self::MsaEvidence => 15,
            Self::CanadaControl => 16,
            Self::ForbiddenReduction => 17,
            Self::VectorLimit => 18,
            Self::CanonicalSize => 19,
        }
    }
}

impl fmt::Display for RtdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for RtdError {}

struct UniqueValue(Value);
struct UniqueValueVisitor;

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor)
    }
}

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded JSON without duplicate object keys")
    }

    fn visit_bool<E: DeError>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_f64<E: DeError>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value).map_or_else(
            || Err(E::custom("non-finite JSON number")),
            |number| Ok(UniqueValue(Value::Number(number))),
        )
    }

    fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.to_owned())))
    }

    fn visit_string<E: DeError>(self, value: String) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value)))
    }

    fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_unit<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut output = Vec::new();
        for _index in 0..=MAX_JSON_BYTES {
            match sequence.next_element::<UniqueValue>()? {
                Some(value) => output.push(value.0),
                None => return Ok(UniqueValue(Value::Array(output))),
            }
        }
        Err(A::Error::custom("RTD_VECTOR_LIMIT"))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut output = Map::new();
        for _index in 0..=MAX_JSON_BYTES {
            let Some(key) = map.next_key::<String>()? else {
                return Ok(UniqueValue(Value::Object(output)));
            };
            if output.contains_key(&key) {
                return Err(A::Error::custom("RTD_DUPLICATE_KEY"));
            }
            output.insert(key, map.next_value::<UniqueValue>()?.0);
        }
        Err(A::Error::custom("RTD_VECTOR_LIMIT"))
    }
}

/// Parses and validates one bounded, fully consumed RTD JSON payload.
///
/// # Errors
///
/// Returns the exact closed RTD refusal for malformed structure, exceeded
/// bounds, or invalid dossier semantics.
pub fn parse_draft_json(payload: &[u8]) -> Result<RtdDossierDraftV1, RtdError> {
    if payload.len() > MAX_JSON_BYTES {
        return Err(RtdError::CanonicalSize);
    }
    scan_json_depth(payload)?;
    let value = serde_json::from_slice::<UniqueValue>(payload)
        .map_err(|error| classify_json_error(&error))?
        .0;
    preflight_schema_version(&value)?;
    preflight_enum_shapes(&value)?;
    let mut draft = serde_json::from_value::<RtdDossierDraftV1>(value)
        .map_err(|error| classify_structural_error(&error))?;
    preflight_limits(&draft)?;
    normalize_negative_zero(&mut draft);
    validate_draft(&draft)?;
    Ok(draft)
}

fn preflight_schema_version(value: &Value) -> Result<(), RtdError> {
    let Value::Object(object) = value else {
        return Ok(());
    };
    let Some(schema) = object.get("schema") else {
        return Ok(());
    };
    if schema.as_str() != Some(RTD_V1_SCHEMA_ID) {
        return Err(RtdError::SchemaVersion);
    }
    let Some(schema_version) = object.get("schema_version") else {
        return Err(RtdError::SchemaVersion);
    };
    if schema_version.as_u64() != Some(1) {
        return Err(RtdError::SchemaVersion);
    }
    let Some(projection_version) = object.get("projection_version") else {
        return Err(RtdError::SchemaVersion);
    };
    if projection_version.as_u64() != Some(1) {
        return Err(RtdError::SchemaVersion);
    }
    Ok(())
}

fn preflight_enum_shapes(value: &Value) -> Result<(), RtdError> {
    let Value::Object(object) = value else {
        return Ok(());
    };
    require_string_enum(object, "audience")?;
    require_string_enum(object, "durability")?;
    preflight_records::<MAX_REFERENCE_DIGESTS>(object, "reference_digests", reference_enums)?;
    preflight_records::<MAX_SCALE_MEMBERSHIPS>(object, "scale_memberships", membership_enums)?;
    preflight_records::<MAX_FACETS>(object, "facets", facet_enums)?;
    preflight_records::<MAX_DYADS>(object, "dyads", dyad_enums)?;
    preflight_records::<MAX_HYPEREDGES>(object, "hyperedges", hyperedge_enums)?;
    preflight_records::<MAX_FLOWS>(object, "flows", flow_enums)?;
    preflight_records::<MAX_GAPS>(object, "gaps", gap_enums)?;
    preflight_records::<MAX_PROVENANCE>(object, "provenance", provenance_enums)
}

fn preflight_records<const N: usize>(
    object: &Map<String, Value>,
    field: &str,
    validate: fn(&Map<String, Value>) -> Result<(), RtdError>,
) -> Result<(), RtdError> {
    let Some(Value::Array(records)) = object.get(field) else {
        return Ok(());
    };
    for index in 0..N {
        if index == records.len() {
            return Ok(());
        }
        if let Value::Object(record) = &records[index] {
            validate(record)?;
        }
    }
    Ok(())
}

fn require_string_enum(object: &Map<String, Value>, field: &str) -> Result<(), RtdError> {
    if object.get(field).is_some_and(|value| !value.is_string()) {
        Err(RtdError::Enum)
    } else {
        Ok(())
    }
}

fn reference_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "evidence_class")
}

fn membership_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "membership_kind")?;
    require_string_enum(object, "status")?;
    require_string_enum(object, "weight_status")?;
    require_string_enum(object, "coverage")?;
    require_string_enum(object, "evidence_class")
}

fn facet_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "family")?;
    require_string_enum(object, "status")?;
    require_string_enum(object, "value_kind")?;
    require_string_enum(object, "coverage")?;
    require_string_enum(object, "evidence_class")
}

fn dyad_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "relation_kind")?;
    require_string_enum(object, "status")?;
    require_string_enum(object, "coverage")?;
    require_string_enum(object, "evidence_class")
}

fn hyperedge_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "hyperedge_kind")?;
    require_string_enum(object, "status")?;
    require_string_enum(object, "coverage")?;
    require_string_enum(object, "evidence_class")
}

fn flow_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "flow_kind")?;
    require_string_enum(object, "status")?;
    require_string_enum(object, "coverage")?;
    require_string_enum(object, "evidence_class")
}

fn gap_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "status")?;
    require_string_enum(object, "reason_code")
}

fn provenance_enums(object: &Map<String, Value>) -> Result<(), RtdError> {
    require_string_enum(object, "evidence_class")
}

fn classify_json_error(error: &serde_json::Error) -> RtdError {
    let message = error.to_string();
    if message.contains("RTD_DUPLICATE_KEY") {
        RtdError::DuplicateKey
    } else if message.contains("RTD_VECTOR_LIMIT") {
        RtdError::VectorLimit
    } else {
        RtdError::Json
    }
}

fn classify_structural_error(error: &serde_json::Error) -> RtdError {
    let message = error.to_string();
    if message.contains("unknown field") {
        RtdError::UnknownField
    } else if message.contains("unknown variant") {
        RtdError::Enum
    } else {
        RtdError::Json
    }
}

fn scan_json_depth(payload: &[u8]) -> Result<(), RtdError> {
    let mut depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;
    for index in 0..=MAX_JSON_BYTES {
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
    Err(RtdError::CanonicalSize)
}

fn normalize_negative_zero(draft: &mut RtdDossierDraftV1) {
    for index in 0..MAX_FACETS {
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
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        let row = &mut draft.scale_memberships[index];
        if row.weight_bits_or_null.as_deref() == Some("8000000000000000") {
            row.weight_bits_or_null = Some("0000000000000000".to_owned());
        }
    }
}

/// Clones a collection and appends one item when its governed bound permits it.
///
/// # Errors
///
/// Returns [`RtdError::LimitExceeded`] without mutating the input when the
/// selected closed collection is full.
pub fn append_bounded<T: Clone>(
    items: &[T],
    item: T,
    kind: RtdCollectionKindV1,
) -> Result<Vec<T>, RtdError> {
    let length = u64::try_from(items.len()).map_err(|_| RtdError::LimitExceeded)?;
    if length >= collection_limit(kind) {
        return Err(RtdError::LimitExceeded);
    }
    let capacity = items.len().checked_add(1).ok_or(RtdError::LimitExceeded)?;
    let mut output = Vec::with_capacity(capacity);
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == items.len() {
            break;
        }
        output.push(items[index].clone());
    }
    output.push(item);
    Ok(output)
}

const fn collection_limit(kind: RtdCollectionKindV1) -> u64 {
    match kind {
        RtdCollectionKindV1::Focus => RTD_MAX_FOCUS,
        RtdCollectionKindV1::ReferenceDigests => RTD_MAX_REFERENCE_DIGESTS,
        RtdCollectionKindV1::ScaleMemberships => RTD_MAX_SCALE_MEMBERSHIPS,
        RtdCollectionKindV1::Facets => RTD_MAX_FACETS,
        RtdCollectionKindV1::Dyads => RTD_MAX_DYADS,
        RtdCollectionKindV1::Hyperedges => RTD_MAX_HYPEREDGES,
        RtdCollectionKindV1::Flows => RTD_MAX_FLOWS,
        RtdCollectionKindV1::Gaps => RTD_MAX_GAPS,
        RtdCollectionKindV1::Provenance => RTD_MAX_PROVENANCE,
        RtdCollectionKindV1::Coordinates => RTD_MAX_COORDINATES,
        RtdCollectionKindV1::MemberRefs => RTD_MAX_HYPEREDGE_MEMBERS,
        RtdCollectionKindV1::PayloadFacets => RTD_MAX_PAYLOAD_FACETS,
        RtdCollectionKindV1::DisplayRefs => RTD_MAX_DECISION_SURFACE_REFS,
        RtdCollectionKindV1::ProvenanceRefs => RTD_MAX_PROVENANCE_REFS,
    }
}

/// Validates one administrative RTD draft without publishing it.
///
/// # Errors
///
/// Returns the first exact closed RTD semantic refusal.
pub fn validate_draft(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    preflight_limits(draft)?;
    validate_scalar_fields(draft)?;
    validate_all_identities(draft)?;
    validate_unique_records(draft)?;
    validate_canadian_flows(draft)?;
    validate_reference_closure(draft)?;
    validate_memberships_and_gaps(draft)?;
    validate_admin_boundary(draft)?;
    validate_facets(draft)?;
    validate_relations(draft)
}

fn validate_canadian_flows(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            return Ok(());
        }
        let flow = &draft.flows[index];
        if flow.flow_kind == FlowKindV1::CommuterJobs
            && (is_canada_endpoint(&flow.origin_ref) || is_canada_endpoint(&flow.destination_ref))
        {
            return Err(RtdError::CanadaControl);
        }
    }
    Ok(())
}

fn preflight_limits(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    check_len(draft.focus.len(), MAX_FOCUS)?;
    check_len(draft.reference_digests.len(), MAX_REFERENCE_DIGESTS)?;
    check_len(draft.scale_memberships.len(), MAX_SCALE_MEMBERSHIPS)?;
    check_len(draft.facets.len(), MAX_FACETS)?;
    check_len(draft.dyads.len(), MAX_DYADS)?;
    check_len(draft.hyperedges.len(), MAX_HYPEREDGES)?;
    check_len(draft.flows.len(), MAX_FLOWS)?;
    check_len(draft.gaps.len(), MAX_GAPS)?;
    check_len(draft.provenance.len(), MAX_PROVENANCE)?;
    let surface = &draft.decision_surface;
    check_len(surface.signal_refs.len(), MAX_DISPLAY_REFS)?;
    check_len(surface.action_refs.len(), MAX_DISPLAY_REFS)?;
    check_len(surface.receipt_refs.len(), MAX_DISPLAY_REFS)?;
    check_len(surface.archive_subject_refs.len(), MAX_DISPLAY_REFS)?;
    validate_nested_limits(draft)
}

const fn check_len(length: usize, limit: usize) -> Result<(), RtdError> {
    if length > limit {
        Err(RtdError::LimitExceeded)
    } else {
        Ok(())
    }
}

fn validate_nested_limits(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        check_len(draft.facets[index].coordinates.len(), MAX_COORDINATES)?;
        check_len(
            draft.facets[index].provenance_refs.len(),
            MAX_PROVENANCE_REFS,
        )?;
    }
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        let row = &draft.hyperedges[index];
        check_len(row.member_refs.len(), MAX_HYPEREDGE_MEMBERS)?;
        check_len(row.payload_facets.len(), MAX_PAYLOAD_FACETS)?;
        check_len(row.provenance_refs.len(), MAX_PROVENANCE_REFS)?;
    }
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        check_len(draft.dyads[index].payload_facets.len(), MAX_PAYLOAD_FACETS)?;
        check_len(
            draft.dyads[index].provenance_refs.len(),
            MAX_PROVENANCE_REFS,
        )?;
    }
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        check_len(draft.flows[index].payload_facets.len(), MAX_PAYLOAD_FACETS)?;
        check_len(
            draft.flows[index].provenance_refs.len(),
            MAX_PROVENANCE_REFS,
        )?;
    }
    validate_nested_provenance_limits(draft)
}

fn validate_nested_provenance_limits(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        check_len(
            draft.scale_memberships[index].provenance_refs.len(),
            MAX_PROVENANCE_REFS,
        )?;
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        check_len(draft.gaps[index].provenance_refs.len(), MAX_PROVENANCE_REFS)?;
    }
    Ok(())
}

fn validate_scalar_fields(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    if draft.schema != RTD_V1_SCHEMA_ID
        || draft.schema_version != 1
        || draft.projection_version != 1
    {
        return Err(RtdError::SchemaVersion);
    }
    validate_digest(&draft.graph_state_hash)?;
    validate_digest(&draft.nominal_world_hash)?;
    validate_digest(&draft.definitions_digest)?;
    validate_digest(&draft.template_digest)?;
    if let Some(value) = &draft.fog_policy_digest {
        validate_digest(value)?;
    }
    if let Some(value) = &draft.knowledge_context_digest {
        validate_digest(value)?;
    }
    validate_reference_scalars(draft)?;
    validate_record_scalars(draft)
}

fn validate_reference_scalars(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_REFERENCE_DIGESTS {
        if index == draft.reference_digests.len() {
            break;
        }
        let row = &draft.reference_digests[index];
        validate_digest(&row.sha256_hex)?;
        validate_text(&row.vintage, MAX_VINTAGE_BYTES, false)?;
    }
    Ok(())
}

fn validate_record_scalars(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        let facet = &draft.facets[index];
        validate_status_value(
            facet.status,
            facet.value_bits_or_null.as_deref(),
            facet.value_kind,
        )?;
        validate_text(&facet.vintage, MAX_VINTAGE_BYTES, false)?;
    }
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        let row = &draft.scale_memberships[index];
        validate_status_value(
            row.weight_status,
            row.weight_bits_or_null.as_deref(),
            ValueKindV1::Float64Bits,
        )?;
    }
    validate_gap_and_provenance_scalars(draft)
}

fn validate_gap_and_provenance_scalars(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        if let Some(producer) = &draft.gaps[index].required_producer_or_null {
            validate_producer(producer)?;
        }
    }
    for index in 0..MAX_PROVENANCE {
        if index == draft.provenance.len() {
            break;
        }
        let row = &draft.provenance[index];
        validate_digest(&row.artifact_digest)?;
        if let Some(value) = &row.transformation_digest_or_null {
            validate_digest(value)?;
        }
        validate_text(&row.locator, MAX_LOCATOR_BYTES, true)?;
        validate_text(&row.vintage, MAX_VINTAGE_BYTES, false)?;
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), RtdError> {
    validate_nfc(value)?;
    if value.len() != 64 {
        return Err(RtdError::Digest);
    }
    for index in 0..64 {
        let byte = value.as_bytes()[index];
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return Err(RtdError::Digest);
        }
    }
    Ok(())
}

fn validate_text(value: &str, limit: usize, allow_empty: bool) -> Result<(), RtdError> {
    validate_nfc(value)?;
    if value.len() > limit || (!allow_empty && value.is_empty()) {
        Err(RtdError::Identity)
    } else {
        Ok(())
    }
}

fn validate_nfc(value: &str) -> Result<(), RtdError> {
    let normalized: String = value.nfc().collect();
    if normalized == value {
        Ok(())
    } else {
        Err(RtdError::NonNfc)
    }
}

fn validate_producer(value: &str) -> Result<(), RtdError> {
    validate_text(value, MAX_PRODUCER_BYTES, false)?;
    let bytes = value.as_bytes();
    if bytes.len() < 5 || &bytes[0..4] != b"PER-" || bytes[4] == b'0' {
        return Err(RtdError::Identity);
    }
    for index in 4..MAX_PRODUCER_BYTES {
        if index == bytes.len() {
            return Ok(());
        }
        if !bytes[index].is_ascii_digit() {
            return Err(RtdError::Identity);
        }
    }
    Ok(())
}

fn validate_status_value(
    status: StatusV1,
    value: Option<&str>,
    kind: ValueKindV1,
) -> Result<(), RtdError> {
    if status == StatusV1::Present {
        validate_bits(value.ok_or(RtdError::StatusValue)?, kind)
    } else if value.is_some() {
        Err(RtdError::StatusValue)
    } else {
        Ok(())
    }
}

fn validate_bits(value: &str, kind: ValueKindV1) -> Result<(), RtdError> {
    if value.len() != 16 {
        return Err(RtdError::StatusValue);
    }
    for index in 0..16 {
        let byte = value.as_bytes()[index];
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return Err(RtdError::StatusValue);
        }
    }
    let raw = u64::from_str_radix(value, 16).map_err(|_| RtdError::StatusValue)?;
    if kind == ValueKindV1::Float64Bits && ((raw >> 52) & 0x7ff) == 0x7ff {
        Err(RtdError::StatusValue)
    } else {
        Ok(())
    }
}

fn identity_key(identity: &TypedIdentityV1) -> IdentityKey {
    (
        identity.domain.clone(),
        identity.authority.clone(),
        identity.local_id.clone(),
    )
}

fn validate_identity(identity: &TypedIdentityV1) -> Result<(), RtdError> {
    validate_identity_component(&identity.domain)?;
    validate_identity_component(&identity.authority)?;
    validate_identity_component(&identity.local_id)?;
    if is_h3_identity(identity) {
        return Err(RtdError::H3BeforePer21);
    }
    if identity.local_id == "19820" {
        return Err(RtdError::MsaEvidence);
    }
    if is_canadian_geography(identity) {
        return Err(RtdError::CanadaControl);
    }
    Ok(())
}

fn validate_identity_component(value: &str) -> Result<(), RtdError> {
    validate_nfc(value)?;
    if value.is_empty() || value.len() > MAX_IDENTITY_BYTES {
        Err(RtdError::Identity)
    } else {
        Ok(())
    }
}

fn is_h3_identity(identity: &TypedIdentityV1) -> bool {
    let domain = identity.domain.to_lowercase();
    let local_id = identity.local_id.to_lowercase();
    domain == "h3"
        || domain == "h3-cell"
        || ((domain == "dimension" || domain == "native-scale") && local_id.starts_with("h3-"))
}

fn is_canadian_geography(identity: &TypedIdentityV1) -> bool {
    let domain = identity.domain.to_lowercase();
    if domain == "external" {
        return false;
    }
    let geographic = matches!(
        domain.as_str(),
        "county" | "geography" | "h3" | "jurisdiction" | "metro" | "msa" | "nation" | "state"
    );
    if !geographic {
        return false;
    }
    let text = format!("{}/{}", identity.authority, identity.local_id).to_lowercase();
    text.contains("canada") || text.contains("windsor") || text.contains("essex")
}

fn validate_identity_list(items: &[TypedIdentityV1]) -> Result<(), RtdError> {
    let mut seen = BTreeSet::new();
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == items.len() {
            return Ok(());
        }
        validate_identity(&items[index])?;
        if !seen.insert(identity_key(&items[index])) {
            return Err(RtdError::DuplicateKey);
        }
    }
    Err(RtdError::LimitExceeded)
}

fn validate_identity_sequence(items: &[TypedIdentityV1]) -> Result<(), RtdError> {
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == items.len() {
            return Ok(());
        }
        validate_identity(&items[index])?;
    }
    Err(RtdError::LimitExceeded)
}

fn validate_all_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    validate_identity_list(&draft.focus)?;
    if let Some(actor) = &draft.actor {
        validate_identity(actor)?;
    }
    validate_identity(&draft.decision_surface.question_id)?;
    validate_identity_sequence(&draft.decision_surface.signal_refs)?;
    validate_identity_sequence(&draft.decision_surface.action_refs)?;
    validate_identity_sequence(&draft.decision_surface.receipt_refs)?;
    validate_identity_sequence(&draft.decision_surface.archive_subject_refs)?;
    validate_reference_identities(draft)?;
    validate_membership_identities(draft)?;
    validate_facet_identities(draft)?;
    validate_relation_identities(draft)?;
    validate_other_identities(draft)
}

fn validate_reference_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_REFERENCE_DIGESTS {
        if index == draft.reference_digests.len() {
            break;
        }
        let row = &draft.reference_digests[index];
        validate_identity(&row.reference_id)?;
        if let Some(schema) = &row.artifact_schema_id_or_null {
            validate_identity(schema)?;
        }
    }
    Ok(())
}

fn validate_membership_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        let row = &draft.scale_memberships[index];
        validate_identity(&row.membership_id)?;
        validate_identity(&row.member_ref)?;
        validate_identity(&row.scale_ref)?;
        validate_identity_list(&row.provenance_refs)?;
    }
    Ok(())
}

fn validate_facet_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        let row = &draft.facets[index];
        validate_identity(&row.facet_id)?;
        validate_identity(&row.subject_ref)?;
        validate_identity(&row.metric_id)?;
        validate_identity(&row.unit_id)?;
        validate_identity(&row.native_scale)?;
        let mut dimensions = BTreeSet::new();
        for coordinate_index in 0..MAX_COORDINATES {
            if coordinate_index == row.coordinates.len() {
                break;
            }
            let coordinate = &row.coordinates[coordinate_index];
            validate_identity(&coordinate.dimension_ref)?;
            validate_identity(&coordinate.member_ref)?;
            if !dimensions.insert(identity_key(&coordinate.dimension_ref)) {
                return Err(RtdError::DuplicateKey);
            }
        }
        validate_identity_list(&row.provenance_refs)?;
    }
    Ok(())
}

fn validate_relation_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        let row = &draft.dyads[index];
        validate_identity(&row.relation_id)?;
        validate_identity(&row.from_ref)?;
        validate_identity(&row.to_ref)?;
        validate_identity(&row.native_scale)?;
        if row.relation_id.domain.to_lowercase() == "hyperedge" {
            return Err(RtdError::ForbiddenReduction);
        }
        validate_identity_list(&row.payload_facets)?;
        validate_identity_list(&row.provenance_refs)?;
    }
    validate_flow_identities(draft)
}

fn validate_flow_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        let row = &draft.flows[index];
        validate_identity(&row.flow_id)?;
        validate_identity(&row.origin_ref)?;
        validate_identity(&row.destination_ref)?;
        validate_identity(&row.native_scale)?;
        validate_identity_list(&row.payload_facets)?;
        validate_identity_list(&row.provenance_refs)?;
    }
    Ok(())
}

fn validate_other_identities(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        let row = &draft.hyperedges[index];
        validate_identity(&row.hyperedge_id)?;
        validate_identity(&row.native_scale)?;
        validate_identity_list(&row.member_refs)?;
        validate_identity_list(&row.payload_facets)?;
        validate_identity_list(&row.provenance_refs)?;
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        let row = &draft.gaps[index];
        validate_identity(&row.gap_id)?;
        validate_identity(&row.requested_metric_or_relation)?;
        validate_identity_list(&row.provenance_refs)?;
    }
    for index in 0..MAX_PROVENANCE {
        if index == draft.provenance.len() {
            break;
        }
        validate_identity(&draft.provenance[index].provenance_id)?;
    }
    Ok(())
}

fn validate_unique_records(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    let mut references = BTreeSet::new();
    for index in 0..MAX_REFERENCE_DIGESTS {
        if index == draft.reference_digests.len() {
            break;
        }
        if !references.insert(identity_key(&draft.reference_digests[index].reference_id)) {
            return Err(RtdError::DuplicateKey);
        }
    }
    let mut records = BTreeSet::new();
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        insert_unique(&mut records, &draft.scale_memberships[index].membership_id)?;
    }
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        insert_unique(&mut records, &draft.facets[index].facet_id)?;
    }
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        insert_unique(&mut records, &draft.dyads[index].relation_id)?;
    }
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        insert_unique(&mut records, &draft.hyperedges[index].hyperedge_id)?;
    }
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        insert_unique(&mut records, &draft.flows[index].flow_id)?;
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        insert_unique(&mut records, &draft.gaps[index].gap_id)?;
    }
    for index in 0..MAX_PROVENANCE {
        if index == draft.provenance.len() {
            break;
        }
        insert_unique(&mut records, &draft.provenance[index].provenance_id)?;
    }
    Ok(())
}

fn insert_unique(
    records: &mut BTreeSet<IdentityKey>,
    identity: &TypedIdentityV1,
) -> Result<(), RtdError> {
    if records.insert(identity_key(identity)) {
        Ok(())
    } else {
        Err(RtdError::DuplicateKey)
    }
}

fn validate_reference_closure(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    let mut provenance = BTreeSet::new();
    for index in 0..MAX_PROVENANCE {
        if index == draft.provenance.len() {
            break;
        }
        provenance.insert(identity_key(&draft.provenance[index].provenance_id));
    }
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        check_refs(&draft.scale_memberships[index].provenance_refs, &provenance)?;
    }
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        check_refs(&draft.facets[index].provenance_refs, &provenance)?;
    }
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        check_refs(&draft.dyads[index].provenance_refs, &provenance)?;
    }
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        check_refs(&draft.hyperedges[index].provenance_refs, &provenance)?;
    }
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        check_refs(&draft.flows[index].provenance_refs, &provenance)?;
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        check_refs(&draft.gaps[index].provenance_refs, &provenance)?;
    }
    validate_payload_and_signal_closure(draft)
}

fn check_refs(refs: &[TypedIdentityV1], declared: &BTreeSet<IdentityKey>) -> Result<(), RtdError> {
    for index in 0..MAX_COLLECTION_ITEMS {
        if index == refs.len() {
            return Ok(());
        }
        if !declared.contains(&identity_key(&refs[index])) {
            return Err(RtdError::DanglingReference);
        }
    }
    Err(RtdError::LimitExceeded)
}

fn validate_payload_and_signal_closure(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    let mut facets = BTreeSet::new();
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        facets.insert(identity_key(&draft.facets[index].facet_id));
    }
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        check_refs(&draft.dyads[index].payload_facets, &facets)?;
    }
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        check_refs(&draft.hyperedges[index].payload_facets, &facets)?;
    }
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        check_refs(&draft.flows[index].payload_facets, &facets)?;
    }
    check_refs(
        &draft.decision_surface.signal_refs,
        &display_subjects(draft),
    )
}

fn display_subjects(draft: &RtdDossierDraftV1) -> BTreeSet<IdentityKey> {
    let mut keys = BTreeSet::new();
    for index in 0..MAX_FOCUS {
        if index == draft.focus.len() {
            break;
        }
        keys.insert(identity_key(&draft.focus[index]));
    }
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        keys.insert(identity_key(&draft.scale_memberships[index].membership_id));
    }
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        keys.insert(identity_key(&draft.facets[index].facet_id));
    }
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        keys.insert(identity_key(&draft.dyads[index].relation_id));
    }
    for index in 0..MAX_HYPEREDGES {
        if index == draft.hyperedges.len() {
            break;
        }
        keys.insert(identity_key(&draft.hyperedges[index].hyperedge_id));
    }
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        keys.insert(identity_key(&draft.flows[index].flow_id));
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        keys.insert(identity_key(&draft.gaps[index].gap_id));
    }
    for index in 0..MAX_PROVENANCE {
        if index == draft.provenance.len() {
            break;
        }
        keys.insert(identity_key(&draft.provenance[index].provenance_id));
    }
    keys
}

fn validate_memberships_and_gaps(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_SCALE_MEMBERSHIPS {
        if index == draft.scale_memberships.len() {
            break;
        }
        match draft.scale_memberships[index].membership_kind {
            MembershipKindV1::WeightedOverlap => return Err(RtdError::UnsupportedDownscale),
            MembershipKindV1::Metropolitan => return Err(RtdError::MsaEvidence),
            MembershipKindV1::Administrative
            | MembershipKindV1::National
            | MembershipKindV1::CommutingZone => {}
        }
    }
    for index in 0..MAX_GAPS {
        if index == draft.gaps.len() {
            break;
        }
        let gap = &draft.gaps[index];
        if is_registered_h3_metric(&gap.requested_metric_or_relation)
            && (gap.status != StatusV1::NotComputed
                || gap.reason_code != GapReasonV1::IdentityContractPending
                || gap.required_producer_or_null.as_deref() != Some("PER-21"))
        {
            return Err(RtdError::H3BeforePer21);
        }
    }
    Ok(())
}

fn validate_admin_boundary(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    let surface = &draft.decision_surface;
    if draft.audience != AudienceV1::AdminMaterial
        || draft.durability != DurabilityV1::InMemory
        || draft.fog_policy_digest.is_some()
        || draft.knowledge_context_digest.is_some()
        || draft.actor.is_some()
        || !surface.action_refs.is_empty()
        || !surface.receipt_refs.is_empty()
        || !surface.archive_subject_refs.is_empty()
    {
        Err(RtdError::ForbiddenReduction)
    } else {
        Ok(())
    }
}

// The contract requires a statically bounded indexed registry traversal.
#[allow(clippy::needless_range_loop)]
fn metric_row(identity: &TypedIdentityV1) -> Result<&'static RtdMetricRegistryRowV1, RtdError> {
    for index in 0..METRIC_ROWS {
        let row = &RTD_V1_METRIC_REGISTRY[index];
        if literal_matches(&row.metric, identity) {
            return Ok(row);
        }
    }
    Err(RtdError::NativeGrain)
}

fn literal_matches(literal: &TypedIdentityLiteralV1, identity: &TypedIdentityV1) -> bool {
    literal.domain == identity.domain
        && literal.authority == identity.authority
        && literal.local_id == identity.local_id
}

// The contract requires a statically bounded indexed registry traversal.
#[allow(clippy::needless_range_loop)]
fn is_registered_h3_metric(identity: &TypedIdentityV1) -> bool {
    for index in 0..METRIC_ROWS {
        let row = &RTD_V1_METRIC_REGISTRY[index];
        if literal_matches(&row.metric, identity)
            && matches!(
                row.metric.local_id,
                "reproduction/h3-population-persons"
                    | "production/h3-workplace-jobs"
                    | "ecology/h3-land-fraction"
            )
        {
            return true;
        }
    }
    false
}

fn validate_facets(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        let facet = &draft.facets[index];
        let row = metric_row(&facet.metric_id)?;
        if matches!(
            row.metric.local_id,
            "reproduction/h3-population-persons"
                | "production/h3-workplace-jobs"
                | "ecology/h3-land-fraction"
        ) {
            return Err(RtdError::H3BeforePer21);
        }
        if row.value_kind != Some(facet.value_kind)
            || !literal_matches(&row.unit, &facet.unit_id)
            || !literal_matches(&row.native_scale, &facet.native_scale)
            || !coordinate_contract_matches(facet, row)
            || !evidence_allowed(facet.evidence_class, row.evidence_classes)
        {
            return Err(RtdError::NativeGrain);
        }
        validate_required_digest(draft, row)?;
    }
    Ok(())
}

fn coordinate_contract_matches(facet: &FacetV1, row: &RtdMetricRegistryRowV1) -> bool {
    if facet.coordinates.len() != row.coordinates.len() {
        return false;
    }
    let mut supplied = BTreeSet::new();
    for index in 0..MAX_COORDINATES {
        if index == facet.coordinates.len() {
            break;
        }
        supplied.insert(identity_key(&facet.coordinates[index].dimension_ref));
    }
    let mut required = BTreeSet::new();
    for index in 0..MAX_COORDINATES {
        if index == row.coordinates.len() {
            break;
        }
        let value = row.coordinates[index];
        required.insert((
            value.domain.to_owned(),
            value.authority.to_owned(),
            value.local_id.to_owned(),
        ));
    }
    supplied == required
}

fn evidence_allowed(evidence: EvidenceClassV1, allowed: &[EvidenceClassV1]) -> bool {
    for index in 0..4 {
        if index == allowed.len() {
            return false;
        }
        if allowed[index] == evidence {
            return true;
        }
    }
    false
}

fn validate_required_digest(
    draft: &RtdDossierDraftV1,
    row: &RtdMetricRegistryRowV1,
) -> Result<(), RtdError> {
    let (Some(reference), Some(digest)) = (row.reference_artifact, row.reference_digest) else {
        return Ok(());
    };
    for index in 0..MAX_REFERENCE_DIGESTS {
        if index == draft.reference_digests.len() {
            break;
        }
        let supplied = &draft.reference_digests[index];
        if literal_matches(&reference, &supplied.reference_id) {
            return if supplied.sha256_hex == digest {
                Ok(())
            } else {
                Err(RtdError::Digest)
            };
        }
    }
    Err(RtdError::Digest)
}

// The contract requires a statically bounded indexed registry traversal.
#[allow(clippy::needless_range_loop)]
fn binding(family: &str, kind: &str) -> Result<&'static RtdRelationBindingRegistryRowV1, RtdError> {
    for index in 0..BINDING_ROWS {
        let row = &RTD_V1_RELATION_BINDING_REGISTRY[index];
        if row.record_family == family && row.kind == kind {
            return Ok(row);
        }
    }
    Err(RtdError::NativeGrain)
}

const fn dyad_kind(kind: DyadKindV1) -> &'static str {
    match kind {
        DyadKindV1::Presence => "PRESENCE",
        DyadKindV1::Membership => "MEMBERSHIP",
        DyadKindV1::Solidarity => "SOLIDARITY",
        DyadKindV1::Command => "COMMAND",
    }
}

const fn flow_kind(kind: FlowKindV1) -> &'static str {
    match kind {
        FlowKindV1::CommuterJobs => "COMMUTER_JOBS",
        FlowKindV1::BorderSynthesis => "BORDER_SYNTHESIS",
    }
}

fn validate_relations(draft: &RtdDossierDraftV1) -> Result<(), RtdError> {
    for index in 0..MAX_DYADS {
        if index == draft.dyads.len() {
            break;
        }
        validate_dyad(&draft.dyads[index])?;
    }
    let mut facet_indices = BTreeMap::new();
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        facet_indices.insert(identity_key(&draft.facets[index].facet_id), index);
    }
    let mut used = BTreeSet::new();
    for index in 0..MAX_FLOWS {
        if index == draft.flows.len() {
            break;
        }
        validate_flow(&draft.flows[index], draft, &facet_indices, &mut used)?;
    }
    for index in 0..MAX_FACETS {
        if index == draft.facets.len() {
            break;
        }
        let row = metric_row(&draft.facets[index].metric_id)?;
        if row.representation == MetricRepresentationV1::ReferenceFlow
            && !used.contains(&identity_key(&draft.facets[index].facet_id))
        {
            return Err(RtdError::DanglingReference);
        }
    }
    Ok(())
}

fn validate_dyad(dyad: &DyadV1) -> Result<(), RtdError> {
    let row = binding("DYAD", dyad_kind(dyad.relation_kind))?;
    if matches!(
        row.payload_mode,
        RelationPayloadModeV1::Empty | RelationPayloadModeV1::ImplicitRelation
    ) && !dyad.payload_facets.is_empty()
    {
        return Err(RtdError::NativeGrain);
    }
    let Some(metric_literal) = row.metric else {
        return Ok(());
    };
    let metric = metric_from_literal(&metric_literal)?;
    if metric.representation != MetricRepresentationV1::Dyad
        || !literal_matches(&metric.native_scale, &dyad.native_scale)
        || !evidence_allowed(dyad.evidence_class, metric.evidence_classes)
    {
        Err(RtdError::NativeGrain)
    } else {
        Ok(())
    }
}

// The contract requires a statically bounded indexed registry traversal.
#[allow(clippy::needless_range_loop)]
fn metric_from_literal(
    literal: &TypedIdentityLiteralV1,
) -> Result<&'static RtdMetricRegistryRowV1, RtdError> {
    for index in 0..METRIC_ROWS {
        let row = &RTD_V1_METRIC_REGISTRY[index];
        if row.metric == *literal {
            return Ok(row);
        }
    }
    Err(RtdError::NativeGrain)
}

fn validate_flow(
    flow: &ReferenceFlowV1,
    draft: &RtdDossierDraftV1,
    facets: &BTreeMap<IdentityKey, usize>,
    used: &mut BTreeSet<IdentityKey>,
) -> Result<(), RtdError> {
    if flow.flow_kind == FlowKindV1::CommuterJobs
        && (is_canada_endpoint(&flow.origin_ref) || is_canada_endpoint(&flow.destination_ref))
    {
        return Err(RtdError::CanadaControl);
    }
    let row = binding("REFERENCE_FLOW", flow_kind(flow.flow_kind))?;
    if row.payload_mode == RelationPayloadModeV1::Empty {
        return if flow.payload_facets.is_empty() {
            Ok(())
        } else {
            Err(RtdError::NativeGrain)
        };
    }
    if flow.payload_facets.len() != 1 {
        return Err(RtdError::NativeGrain);
    }
    let key = identity_key(&flow.payload_facets[0]);
    if !used.insert(key.clone()) {
        return Err(RtdError::DuplicateKey);
    }
    let facet_index = *facets.get(&key).ok_or(RtdError::DanglingReference)?;
    validate_flow_facet(flow, &draft.facets[facet_index], row)
}

fn validate_flow_facet(
    flow: &ReferenceFlowV1,
    facet: &FacetV1,
    row: &RtdRelationBindingRegistryRowV1,
) -> Result<(), RtdError> {
    let metric_literal = row.metric.ok_or(RtdError::NativeGrain)?;
    let metric = metric_from_literal(&metric_literal)?;
    if identity_key(&facet.subject_ref) != identity_key(&flow.flow_id)
        || !literal_matches(&metric_literal, &facet.metric_id)
        || metric.representation != MetricRepresentationV1::ReferenceFlow
        || !literal_matches(&metric.native_scale, &flow.native_scale)
        || !evidence_allowed(flow.evidence_class, metric.evidence_classes)
    {
        return Err(RtdError::NativeGrain);
    }
    let origin = coordinate_member(facet, &metric.coordinates[0]);
    let destination = coordinate_member(facet, &metric.coordinates[1]);
    if origin.is_none_or(|value| identity_key(value) != identity_key(&flow.origin_ref))
        || destination
            .is_none_or(|value| identity_key(value) != identity_key(&flow.destination_ref))
    {
        Err(RtdError::NativeGrain)
    } else {
        Ok(())
    }
}

fn coordinate_member<'a>(
    facet: &'a FacetV1,
    dimension: &TypedIdentityLiteralV1,
) -> Option<&'a TypedIdentityV1> {
    for index in 0..MAX_COORDINATES {
        if index == facet.coordinates.len() {
            return None;
        }
        if literal_matches(dimension, &facet.coordinates[index].dimension_ref) {
            return Some(&facet.coordinates[index].member_ref);
        }
    }
    None
}

fn is_canada_endpoint(identity: &TypedIdentityV1) -> bool {
    let text = format!(
        "{}/{}/{}",
        identity.domain, identity.authority, identity.local_id
    )
    .to_lowercase();
    text.contains("canada") || text.contains("windsor") || text.contains("essex")
}
