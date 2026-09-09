//! Frozen synthetic proof-profile, intervention, and persistence records.

use std::cmp::Ordering;

use crate::classifier::{classify_persistence, PersistenceClass, PersistenceClassError};
use crate::digest::Digest32;
use crate::wire::{PayloadCursor, PayloadEncoder, SfsWireError, T3Record};

const MAX_PROFILE_ITEMS: usize = 64;
const MAX_DELTA_ROWS: usize = 65_535;
const COMPONENT_ID_MAX_BYTES: usize = 256;
const PROFILE_ENTRY_MAX_BYTES: usize = 96;
const DELTA_ROW_BYTES: u32 = 97;
const AUDIT_SEMANTICS_ID: &str = "babylon.sfs.audit.v1";

/// Exact structural refusals for frozen T3 proof records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsProfileRecordError {
    /// A uniform-envelope or primitive wire rule failed.
    Wire(SfsWireError),
    /// A component-kind byte is outside the closed V1 registry.
    InvalidComponentKind { value: u8 },
    /// The audit-semantics identifier is not the exact V1 identifier.
    InvalidAuditSemanticsId,
    /// A ledger-kind byte is outside the closed V1 registry.
    InvalidLedgerKind { value: u8 },
    /// An intervention operation is outside the closed V1 registry.
    InvalidInterventionOperation { value: u8 },
    /// An intervention row violates its operation-specific digest law.
    InvalidInterventionRow,
    /// Two component profiles declare the same canonical component ID.
    DuplicateComponentId,
    /// One causal-cone set contains a duplicate canonical ID.
    DuplicateConeId { set: &'static str },
    /// The persistence classifier refused the declared shape or values.
    Classification(PersistenceClassError),
    /// A valid stored persistence class differs from the recomputed class.
    ClassificationMismatch { stored: u8, computed: u8 },
}

impl From<SfsWireError> for SfsProfileRecordError {
    fn from(value: SfsWireError) -> Self {
        Self::Wire(value)
    }
}

impl From<PersistenceClassError> for SfsProfileRecordError {
    fn from(value: PersistenceClassError) -> Self {
        Self::Classification(value)
    }
}

/// Closed component kinds admitted by the synthetic proof profile.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKindV1 {
    /// One BSL rule component.
    BslRule = 0,
    /// One governed Rust boundary component.
    RustBoundary = 1,
    /// One deterministic reducer component.
    Reducer = 2,
    /// One post-commit evidence producer.
    PostCommitProducer = 3,
}

impl ComponentKindV1 {
    const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::BslRule),
            1 => Some(Self::RustBoundary),
            2 => Some(Self::Reducer),
            3 => Some(Self::PostCommitProducer),
            _ => None,
        }
    }
}

/// One sorted unique bounded set embedded in a component profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalProfileSet {
    entries: Vec<String>,
}

impl CanonicalProfileSet {
    /// Validates, sorts, and seals one profile allowlist.
    ///
    /// # Errors
    /// Returns exact count, NFC, string-length, or duplicate refusals.
    pub fn new(
        field: &'static str,
        mut entries: Vec<String>,
    ) -> Result<Self, SfsProfileRecordError> {
        validate_profile_count(field, entries.len())?;
        for index in 0..64 {
            if index >= entries.len() {
                break;
            }
            validate_nfc(field, &entries[index], PROFILE_ENTRY_MAX_BYTES)?;
        }
        entries.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        reject_profile_duplicates(field, &entries)?;
        Ok(Self { entries })
    }

    /// Returns the immutable canonical entries.
    #[must_use]
    pub fn entries(&self) -> &[String] {
        &self.entries
    }
}

/// One sealed component audit profile with eight canonical sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsComponentProofProfileV1 {
    component_id: String,
    component_kind: ComponentKindV1,
    component_source_digest: Digest32,
    field_reads: CanonicalProfileSet,
    edge_reads: CanonicalProfileSet,
    constant_reads: CanonicalProfileSet,
    queries: CanonicalProfileSet,
    operators: CanonicalProfileSet,
    intrinsics: CanonicalProfileSet,
    comparison_clamp_contexts: CanonicalProfileSet,
    effects: CanonicalProfileSet,
}

impl SfsComponentProofProfileV1 {
    /// Constructs one exact component profile.
    ///
    /// # Errors
    /// Returns the exact component-ID NFC or byte-length refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        component_id: &str,
        component_kind: ComponentKindV1,
        component_source_digest: Digest32,
        field_reads: CanonicalProfileSet,
        edge_reads: CanonicalProfileSet,
        constant_reads: CanonicalProfileSet,
        queries: CanonicalProfileSet,
        operators: CanonicalProfileSet,
        intrinsics: CanonicalProfileSet,
        comparison_clamp_contexts: CanonicalProfileSet,
        effects: CanonicalProfileSet,
    ) -> Result<Self, SfsProfileRecordError> {
        validate_nfc("component_id", component_id, COMPONENT_ID_MAX_BYTES)?;
        Ok(Self {
            component_id: component_id.to_owned(),
            component_kind,
            component_source_digest,
            field_reads,
            edge_reads,
            constant_reads,
            queries,
            operators,
            intrinsics,
            comparison_clamp_contexts,
            effects,
        })
    }

    /// Returns the canonical component ID.
    #[must_use]
    pub fn component_id(&self) -> &str {
        &self.component_id
    }

    /// Returns the closed component kind.
    #[must_use]
    pub const fn component_kind(&self) -> ComponentKindV1 {
        self.component_kind
    }

    /// Returns the exact source identity digest.
    #[must_use]
    pub const fn component_source_digest(&self) -> Digest32 {
        self.component_source_digest
    }
}

impl T3Record for SfsComponentProofProfileV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-component-proof-profile.v1";
    const MAX_PAYLOAD_BYTES: usize = 50_483;
    type Error = SfsProfileRecordError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_nfc_utf8("component_id", &self.component_id, COMPONENT_ID_MAX_BYTES)?;
        out.push_u8(self.component_kind as u8)?;
        out.push_digest(self.component_source_digest)?;
        encode_profile_set(out, &self.field_reads)?;
        encode_profile_set(out, &self.edge_reads)?;
        encode_profile_set(out, &self.constant_reads)?;
        encode_profile_set(out, &self.queries)?;
        encode_profile_set(out, &self.operators)?;
        encode_profile_set(out, &self.intrinsics)?;
        encode_profile_set(out, &self.comparison_clamp_contexts)?;
        encode_profile_set(out, &self.effects)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let component_id = cursor.read_nfc_utf8("component_id", COMPONENT_ID_MAX_BYTES)?;
        let kind_value = cursor.read_u8()?;
        let component_kind = ComponentKindV1::from_code(kind_value)
            .ok_or(SfsProfileRecordError::InvalidComponentKind { value: kind_value })?;
        Self::new(
            &component_id,
            component_kind,
            cursor.read_digest()?,
            decode_profile_set(cursor, "field_reads")?,
            decode_profile_set(cursor, "edge_reads")?,
            decode_profile_set(cursor, "constant_reads")?,
            decode_profile_set(cursor, "queries")?,
            decode_profile_set(cursor, "operators")?,
            decode_profile_set(cursor, "intrinsics")?,
            decode_profile_set(cursor, "comparison_clamp_contexts")?,
            decode_profile_set(cursor, "effects")?,
        )
    }
}

/// Complete closed proof profile for one synthetic causal cone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsProofProfileV1 {
    governed_manifest_digest: Digest32,
    forbidden_corpus_digest: Digest32,
    audit_semantics_id: String,
    audit_source_digest: Digest32,
    causal_cone_digest: Digest32,
    components: Vec<SfsComponentProofProfileV1>,
}

impl SfsProofProfileV1 {
    /// Constructs and canonicalizes one complete synthetic proof profile.
    ///
    /// # Errors
    /// Returns exact audit-ID, count, or component-identity refusals.
    pub fn new(
        governed_manifest_digest: Digest32,
        forbidden_corpus_digest: Digest32,
        audit_semantics_id: &str,
        audit_source_digest: Digest32,
        causal_cone_digest: Digest32,
        mut components: Vec<SfsComponentProofProfileV1>,
    ) -> Result<Self, SfsProfileRecordError> {
        validate_ascii("audit_semantics_id", audit_semantics_id, 64)?;
        if audit_semantics_id != AUDIT_SEMANTICS_ID {
            return Err(SfsProfileRecordError::InvalidAuditSemanticsId);
        }
        validate_profile_count("components", components.len())?;
        components.sort_by(component_order);
        reject_component_duplicates(&components)?;
        Ok(Self {
            governed_manifest_digest,
            forbidden_corpus_digest,
            audit_semantics_id: audit_semantics_id.to_owned(),
            audit_source_digest,
            causal_cone_digest,
            components,
        })
    }

    /// Returns the immutable canonical component profiles.
    #[must_use]
    pub fn components(&self) -> &[SfsComponentProofProfileV1] {
        &self.components
    }

    /// Returns the governed host-manifest digest input.
    #[must_use]
    pub const fn governed_manifest_digest(&self) -> Digest32 {
        self.governed_manifest_digest
    }

    /// Returns the causal-cone record digest input.
    #[must_use]
    pub const fn causal_cone_digest(&self) -> Digest32 {
        self.causal_cone_digest
    }
}

impl T3Record for SfsProofProfileV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-proof-profile.v1";
    const MAX_PAYLOAD_BYTES: usize = 3_233_944;
    type Error = SfsProfileRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the profile contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_digest(self.governed_manifest_digest)?;
        out.push_digest(self.forbidden_corpus_digest)?;
        out.push_ascii("audit_semantics_id", &self.audit_semantics_id, 64)?;
        out.push_digest(self.audit_source_digest)?;
        out.push_digest(self.causal_cone_digest)?;
        out.push_u16(u16::try_from(self.components.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "components",
            }
        })?)?;
        for index in 0..64 {
            if index >= self.components.len() {
                break;
            }
            out.push_complete_envelope(&self.components[index])?;
        }
        Ok(())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let governed = cursor.read_digest()?;
        let forbidden = cursor.read_digest()?;
        let audit_id = cursor.read_ascii("audit_semantics_id", 64)?;
        let audit_source = cursor.read_digest()?;
        let cone = cursor.read_digest()?;
        let count = decode_profile_count(cursor, "components")?;
        let components = decode_components(cursor, count)?;
        Self::new(
            governed,
            forbidden,
            &audit_id,
            audit_source,
            cone,
            components,
        )
    }
}

/// Exact root, sink, and reachable component identity sets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalConeV1 {
    roots: Vec<String>,
    sinks: Vec<String>,
    components: Vec<String>,
}

impl CausalConeV1 {
    /// Constructs and independently canonicalizes all three cone sets.
    ///
    /// # Errors
    /// Returns exact count, string, or named duplicate refusals.
    pub fn new(
        roots: Vec<String>,
        sinks: Vec<String>,
        components: Vec<String>,
    ) -> Result<Self, SfsProfileRecordError> {
        Ok(Self {
            roots: canonicalize_cone_set("roots", roots)?,
            sinks: canonicalize_cone_set("sinks", sinks)?,
            components: canonicalize_cone_set("components", components)?,
        })
    }

    /// Returns canonical root component IDs.
    #[must_use]
    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    /// Returns canonical sink component IDs.
    #[must_use]
    pub fn sinks(&self) -> &[String] {
        &self.sinks
    }

    /// Returns canonical reachable component IDs.
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

impl T3Record for CausalConeV1 {
    const DOMAIN: &'static [u8] = b"babylon.sfs-causal-cone.v1";
    const MAX_PAYLOAD_BYTES: usize = 49_542;
    type Error = SfsProfileRecordError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        encode_string_set(out, &self.roots)?;
        encode_string_set(out, &self.sinks)?;
        encode_string_set(out, &self.components)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let roots = decode_cone_set(cursor, "roots")?;
        let sinks = decode_cone_set(cursor, "sinks")?;
        let components = decode_cone_set(cursor, "components")?;
        Self::new(roots, sinks, components)
    }
}

/// Closed ledger kinds that may differ between synthetic twins.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferingLedgerKindV1 {
    /// Governed exogenous-input ledger.
    ExogenousInput = 0,
    /// Governed practice-attempt ledger.
    PracticeAttempt = 1,
}

impl DifferingLedgerKindV1 {
    const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::ExogenousInput),
            1 => Some(Self::PracticeAttempt),
            _ => None,
        }
    }
}

/// Closed intervention row operations.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionOperationV1 {
    /// Add one canonical ledger row.
    Add = 0,
    /// Remove one canonical ledger row.
    Remove = 1,
    /// Replace one canonical ledger row.
    Replace = 2,
}

impl InterventionOperationV1 {
    const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Add),
            1 => Some(Self::Remove),
            2 => Some(Self::Replace),
            _ => None,
        }
    }
}

/// One fixed canonical row-level ledger delta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionDeltaRowV1 {
    operation: InterventionOperationV1,
    stable_row_id_digest: Digest32,
    control_row_digest: Digest32,
    intervention_row_digest: Digest32,
}

impl InterventionDeltaRowV1 {
    /// Constructs a row under the exact operation-specific zero rules.
    ///
    /// # Errors
    /// Returns `InvalidInterventionRow` when its digest sides violate V1.
    pub fn new(
        operation: InterventionOperationV1,
        stable_row_id_digest: Digest32,
        control_row_digest: Digest32,
        intervention_row_digest: Digest32,
    ) -> Result<Self, SfsProfileRecordError> {
        let valid = match operation {
            InterventionOperationV1::Add => {
                control_row_digest.is_zero() && !intervention_row_digest.is_zero()
            }
            InterventionOperationV1::Remove => {
                !control_row_digest.is_zero() && intervention_row_digest.is_zero()
            }
            InterventionOperationV1::Replace => {
                !control_row_digest.is_zero()
                    && !intervention_row_digest.is_zero()
                    && control_row_digest != intervention_row_digest
            }
        };
        if !valid {
            return Err(SfsProfileRecordError::InvalidInterventionRow);
        }
        Ok(Self {
            operation,
            stable_row_id_digest,
            control_row_digest,
            intervention_row_digest,
        })
    }
}

/// Canonical row-level difference for one selected synthetic ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterventionDeltaV1 {
    ledger_kind: DifferingLedgerKindV1,
    rows: Vec<InterventionDeltaRowV1>,
}

impl InterventionDeltaV1 {
    /// Preflights, bounds, sorts, and rejects duplicate stable row IDs.
    ///
    /// # Errors
    /// Returns exact count or duplicate-entry refusals before publication.
    pub fn new(
        ledger_kind: DifferingLedgerKindV1,
        rows: Vec<InterventionDeltaRowV1>,
    ) -> Result<Self, SfsProfileRecordError> {
        validate_delta_count(rows.len())?;
        let mut admitted = Vec::with_capacity(rows.len());
        for row in rows.into_iter().take(65_536) {
            admitted.push(row);
        }
        admitted.sort_by(|left, right| left.stable_row_id_digest.cmp(&right.stable_row_id_digest));
        reject_delta_duplicates(&admitted)?;
        Ok(Self {
            ledger_kind,
            rows: admitted,
        })
    }

    /// Returns the selected differing-ledger kind.
    #[must_use]
    pub const fn ledger_kind(&self) -> DifferingLedgerKindV1 {
        self.ledger_kind
    }
}

impl T3Record for InterventionDeltaV1 {
    const DOMAIN: &'static [u8] = b"babylon.intervention-delta.v1";
    const MAX_PAYLOAD_BYTES: usize = 6_356_900;
    type Error = SfsProfileRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the delta contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u8(self.ledger_kind as u8)?;
        out.push_u32(u32::try_from(self.rows.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "intervention_rows",
            }
        })?)?;
        for index in 0..65_535 {
            if index >= self.rows.len() {
                break;
            }
            encode_delta_row(out, &self.rows[index])?;
        }
        Ok(())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let kind_value = cursor.read_u8()?;
        let ledger_kind = DifferingLedgerKindV1::from_code(kind_value)
            .ok_or(SfsProfileRecordError::InvalidLedgerKind { value: kind_value })?;
        let declared = cursor.read_u32()?;
        let product =
            declared
                .checked_mul(DELTA_ROW_BYTES)
                .ok_or(SfsWireError::ArithmeticOverflow {
                    field: "intervention_rows",
                })?;
        let count = usize::try_from(declared).map_err(|_| SfsWireError::ArithmeticOverflow {
            field: "intervention_rows",
        })?;
        validate_delta_count(count)?;
        let expected = usize::try_from(product).map_err(|_| SfsWireError::ArithmeticOverflow {
            field: "intervention_rows",
        })?;
        validate_exact_remaining(cursor.remaining(), expected)?;
        let rows = decode_delta_rows(cursor, count)?;
        Self::new(ledger_kind, rows)
    }
}

/// One computed post-intervention persistence comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistenceComparisonV1 {
    control_trace_digest: Digest32,
    intervention_trace_digest: Digest32,
    differing_ledger_kind: DifferingLedgerKindV1,
    control_differing_ledger_digest: Digest32,
    intervention_differing_ledger_digest: Digest32,
    intervention_delta_digest: Digest32,
    last_intervention_tick: u64,
    post_width: u16,
    separations: Vec<f64>,
    persistence_class: PersistenceClass,
}

impl PersistenceComparisonV1 {
    /// Constructs exact `P + 1` separations and computes their class.
    ///
    /// # Errors
    /// Returns the exact persistence width, length, or non-finite refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control_trace_digest: Digest32,
        intervention_trace_digest: Digest32,
        differing_ledger_kind: DifferingLedgerKindV1,
        control_differing_ledger_digest: Digest32,
        intervention_differing_ledger_digest: Digest32,
        intervention_delta_digest: Digest32,
        last_intervention_tick: u64,
        post_width: u16,
        mut separations: Vec<f64>,
    ) -> Result<Self, SfsProfileRecordError> {
        let persistence_class = classify_persistence(post_width, &separations)?;
        normalize_separation_zeros(&mut separations);
        Ok(Self {
            control_trace_digest,
            intervention_trace_digest,
            differing_ledger_kind,
            control_differing_ledger_digest,
            intervention_differing_ledger_digest,
            intervention_delta_digest,
            last_intervention_tick,
            post_width,
            separations,
            persistence_class,
        })
    }

    /// Returns the control trace digest.
    #[must_use]
    pub const fn control_trace_digest(&self) -> Digest32 {
        self.control_trace_digest
    }

    /// Returns the intervention trace digest.
    #[must_use]
    pub const fn intervention_trace_digest(&self) -> Digest32 {
        self.intervention_trace_digest
    }

    /// Returns the selected differing-ledger kind.
    #[must_use]
    pub const fn differing_ledger_kind(&self) -> DifferingLedgerKindV1 {
        self.differing_ledger_kind
    }

    /// Returns the control differing-ledger digest.
    #[must_use]
    pub const fn control_differing_ledger_digest(&self) -> Digest32 {
        self.control_differing_ledger_digest
    }

    /// Returns the intervention differing-ledger digest.
    #[must_use]
    pub const fn intervention_differing_ledger_digest(&self) -> Digest32 {
        self.intervention_differing_ledger_digest
    }

    /// Returns the complete intervention-delta digest.
    #[must_use]
    pub const fn intervention_delta_digest(&self) -> Digest32 {
        self.intervention_delta_digest
    }
}

impl T3Record for PersistenceComparisonV1 {
    const DOMAIN: &'static [u8] = b"babylon.persistence-comparison.v1";
    const MAX_PAYLOAD_BYTES: usize = 598;
    type Error = SfsProfileRecordError;

    #[allow(clippy::needless_range_loop)] // Literal ceiling is part of the comparison contract.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_digest(self.control_trace_digest)?;
        out.push_digest(self.intervention_trace_digest)?;
        out.push_u8(self.differing_ledger_kind as u8)?;
        out.push_digest(self.control_differing_ledger_digest)?;
        out.push_digest(self.intervention_differing_ledger_digest)?;
        out.push_digest(self.intervention_delta_digest)?;
        out.push_u64(self.last_intervention_tick)?;
        out.push_u16(self.post_width)?;
        out.push_u16(u16::try_from(self.separations.len()).map_err(|_| {
            SfsWireError::ArithmeticOverflow {
                field: "separations",
            }
        })?)?;
        for index in 0..53 {
            if index >= self.separations.len() {
                break;
            }
            out.push_finite_f64("separations", self.separations[index])?;
        }
        out.push_u8(self.persistence_class.code())
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error> {
        let control_trace = cursor.read_digest()?;
        let intervention_trace = cursor.read_digest()?;
        let kind_value = cursor.read_u8()?;
        let ledger_kind = DifferingLedgerKindV1::from_code(kind_value)
            .ok_or(SfsProfileRecordError::InvalidLedgerKind { value: kind_value })?;
        let control_ledger = cursor.read_digest()?;
        let intervention_ledger = cursor.read_digest()?;
        let delta = cursor.read_digest()?;
        let last_tick = cursor.read_u64()?;
        let post_width = cursor.read_u16()?;
        let count = usize::from(cursor.read_u16()?);
        validate_persistence_shape(post_width, count)?;
        let separations = decode_separations(cursor, count)?;
        let stored_code = cursor.read_u8()?;
        let stored = PersistenceClass::from_code(stored_code).ok_or(SfsWireError::InvalidCode {
            field: "persistence_class",
            value: stored_code,
        })?;
        let value = Self::new(
            control_trace,
            intervention_trace,
            ledger_kind,
            control_ledger,
            intervention_ledger,
            delta,
            last_tick,
            post_width,
            separations,
        )?;
        if stored != value.persistence_class {
            return Err(SfsProfileRecordError::ClassificationMismatch {
                stored: stored.code(),
                computed: value.persistence_class.code(),
            });
        }
        Ok(value)
    }
}

fn validate_nfc(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), SfsProfileRecordError> {
    let mut encoder = PayloadEncoder::new(maximum + 2);
    encoder.push_nfc_utf8(field, value, maximum)?;
    Ok(())
}

fn validate_ascii(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), SfsProfileRecordError> {
    let mut encoder = PayloadEncoder::new(maximum + 2);
    encoder.push_ascii(field, value, maximum)?;
    Ok(())
}

fn validate_profile_count(field: &'static str, actual: usize) -> Result<(), SfsProfileRecordError> {
    if actual > MAX_PROFILE_ITEMS {
        Err(SfsWireError::CountTooLarge {
            field,
            limit: MAX_PROFILE_ITEMS,
            actual,
        }
        .into())
    } else {
        Ok(())
    }
}

fn decode_profile_count(
    cursor: &mut PayloadCursor<'_>,
    field: &'static str,
) -> Result<usize, SfsProfileRecordError> {
    let count = usize::from(cursor.read_u16()?);
    validate_profile_count(field, count)?;
    Ok(count)
}

fn reject_profile_duplicates(
    field: &'static str,
    entries: &[String],
) -> Result<(), SfsProfileRecordError> {
    for index in 1..64 {
        if index >= entries.len() {
            break;
        }
        if entries[index - 1].as_bytes() == entries[index].as_bytes() {
            return Err(SfsWireError::DuplicateEntry { field }.into());
        }
    }
    Ok(())
}

#[allow(clippy::needless_range_loop)] // Literal ceiling is part of the set contract.
fn encode_string_set(out: &mut PayloadEncoder, entries: &[String]) -> Result<(), SfsWireError> {
    out.push_u16(
        u16::try_from(entries.len()).map_err(|_| SfsWireError::ArithmeticOverflow {
            field: "profile_set",
        })?,
    )?;
    for index in 0..64 {
        if index >= entries.len() {
            break;
        }
        out.push_nfc_utf8("profile_set", &entries[index], COMPONENT_ID_MAX_BYTES)?;
    }
    Ok(())
}

#[allow(clippy::needless_range_loop)] // Literal ceiling is part of the set contract.
fn encode_profile_set(
    out: &mut PayloadEncoder,
    set: &CanonicalProfileSet,
) -> Result<(), SfsWireError> {
    out.push_u16(u16::try_from(set.entries.len()).map_err(|_| {
        SfsWireError::ArithmeticOverflow {
            field: "profile_set",
        }
    })?)?;
    for index in 0..64 {
        if index >= set.entries.len() {
            break;
        }
        out.push_nfc_utf8("profile_set", &set.entries[index], PROFILE_ENTRY_MAX_BYTES)?;
    }
    Ok(())
}

fn decode_profile_set(
    cursor: &mut PayloadCursor<'_>,
    field: &'static str,
) -> Result<CanonicalProfileSet, SfsProfileRecordError> {
    let count = decode_profile_count(cursor, field)?;
    let mut entries = Vec::with_capacity(count);
    for index in 0..64 {
        if index >= count {
            break;
        }
        let value = cursor.read_nfc_utf8(field, PROFILE_ENTRY_MAX_BYTES)?;
        validate_next_string(entries.last(), &value, field, false)?;
        entries.push(value);
    }
    CanonicalProfileSet::new(field, entries)
}

fn component_order(
    left: &SfsComponentProofProfileV1,
    right: &SfsComponentProofProfileV1,
) -> Ordering {
    left.component_id
        .as_bytes()
        .cmp(right.component_id.as_bytes())
}

fn reject_component_duplicates(
    components: &[SfsComponentProofProfileV1],
) -> Result<(), SfsProfileRecordError> {
    for index in 1..64 {
        if index >= components.len() {
            break;
        }
        if component_order(&components[index - 1], &components[index]) == Ordering::Equal {
            return Err(SfsProfileRecordError::DuplicateComponentId);
        }
    }
    Ok(())
}

fn decode_components(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
) -> Result<Vec<SfsComponentProofProfileV1>, SfsProfileRecordError> {
    let mut components = Vec::with_capacity(count);
    for index in 0..64 {
        if index >= count {
            break;
        }
        let value = cursor.read_complete_envelope::<SfsComponentProofProfileV1>()?;
        validate_next_component(components.last(), &value)?;
        components.push(value);
    }
    Ok(components)
}

fn validate_next_component(
    previous: Option<&SfsComponentProofProfileV1>,
    value: &SfsComponentProofProfileV1,
) -> Result<(), SfsProfileRecordError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    match component_order(previous, value) {
        Ordering::Less => Ok(()),
        Ordering::Equal => Err(SfsProfileRecordError::DuplicateComponentId),
        Ordering::Greater => Err(SfsWireError::OutOfOrder {
            field: "components",
        }
        .into()),
    }
}

fn canonicalize_cone_set(
    field: &'static str,
    mut entries: Vec<String>,
) -> Result<Vec<String>, SfsProfileRecordError> {
    validate_profile_count(field, entries.len())?;
    for index in 0..64 {
        if index >= entries.len() {
            break;
        }
        validate_nfc(field, &entries[index], COMPONENT_ID_MAX_BYTES)?;
    }
    entries.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    for index in 1..64 {
        if index >= entries.len() {
            break;
        }
        if entries[index - 1].as_bytes() == entries[index].as_bytes() {
            return Err(SfsProfileRecordError::DuplicateConeId { set: field });
        }
    }
    Ok(entries)
}

fn decode_cone_set(
    cursor: &mut PayloadCursor<'_>,
    field: &'static str,
) -> Result<Vec<String>, SfsProfileRecordError> {
    let count = decode_profile_count(cursor, field)?;
    let mut entries = Vec::with_capacity(count);
    for index in 0..64 {
        if index >= count {
            break;
        }
        let value = cursor.read_nfc_utf8(field, COMPONENT_ID_MAX_BYTES)?;
        validate_next_string(entries.last(), &value, field, true)?;
        entries.push(value);
    }
    Ok(entries)
}

fn validate_next_string(
    previous: Option<&String>,
    value: &str,
    field: &'static str,
    cone: bool,
) -> Result<(), SfsProfileRecordError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    match previous.as_bytes().cmp(value.as_bytes()) {
        Ordering::Less => Ok(()),
        Ordering::Greater => Err(SfsWireError::OutOfOrder { field }.into()),
        Ordering::Equal if cone => Err(SfsProfileRecordError::DuplicateConeId { set: field }),
        Ordering::Equal => Err(SfsWireError::DuplicateEntry { field }.into()),
    }
}

fn validate_delta_count(actual: usize) -> Result<(), SfsProfileRecordError> {
    if actual > MAX_DELTA_ROWS {
        Err(SfsWireError::CountTooLarge {
            field: "intervention_rows",
            limit: MAX_DELTA_ROWS,
            actual,
        }
        .into())
    } else {
        Ok(())
    }
}

fn reject_delta_duplicates(rows: &[InterventionDeltaRowV1]) -> Result<(), SfsProfileRecordError> {
    for index in 1..65_535 {
        if index >= rows.len() {
            break;
        }
        if rows[index - 1].stable_row_id_digest == rows[index].stable_row_id_digest {
            return Err(SfsWireError::DuplicateEntry {
                field: "intervention_rows",
            }
            .into());
        }
    }
    Ok(())
}

fn validate_exact_remaining(actual: usize, expected: usize) -> Result<(), SfsProfileRecordError> {
    match actual.cmp(&expected) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(SfsWireError::TruncatedEnvelope.into()),
        Ordering::Greater => Err(SfsWireError::TrailingBytes {
            count: actual - expected,
        }
        .into()),
    }
}

fn encode_delta_row(
    out: &mut PayloadEncoder,
    row: &InterventionDeltaRowV1,
) -> Result<(), SfsWireError> {
    out.push_u8(row.operation as u8)?;
    out.push_digest(row.stable_row_id_digest)?;
    out.push_digest(row.control_row_digest)?;
    out.push_digest(row.intervention_row_digest)
}

fn decode_delta_row(
    cursor: &mut PayloadCursor<'_>,
) -> Result<InterventionDeltaRowV1, SfsProfileRecordError> {
    let operation_value = cursor.read_u8()?;
    let operation = InterventionOperationV1::from_code(operation_value).ok_or(
        SfsProfileRecordError::InvalidInterventionOperation {
            value: operation_value,
        },
    )?;
    InterventionDeltaRowV1::new(
        operation,
        cursor.read_digest()?,
        cursor.read_digest()?,
        cursor.read_digest()?,
    )
}

fn decode_delta_rows(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
) -> Result<Vec<InterventionDeltaRowV1>, SfsProfileRecordError> {
    let mut rows = Vec::with_capacity(count);
    for index in 0..65_535 {
        if index >= count {
            break;
        }
        let row = decode_delta_row(cursor)?;
        validate_next_delta(rows.last(), &row)?;
        rows.push(row);
    }
    Ok(rows)
}

fn validate_next_delta(
    previous: Option<&InterventionDeltaRowV1>,
    row: &InterventionDeltaRowV1,
) -> Result<(), SfsProfileRecordError> {
    let Some(previous) = previous else {
        return Ok(());
    };
    match previous.stable_row_id_digest.cmp(&row.stable_row_id_digest) {
        Ordering::Less => Ok(()),
        Ordering::Equal => Err(SfsWireError::DuplicateEntry {
            field: "intervention_rows",
        }
        .into()),
        Ordering::Greater => Err(SfsWireError::OutOfOrder {
            field: "intervention_rows",
        }
        .into()),
    }
}

fn validate_persistence_shape(post_width: u16, actual: usize) -> Result<(), SfsProfileRecordError> {
    if !(2..=52).contains(&post_width) {
        return Err(PersistenceClassError::InvalidPostWidth { found: post_width }.into());
    }
    let expected =
        usize::from(post_width)
            .checked_add(1)
            .ok_or(PersistenceClassError::WrongLength {
                expected: usize::MAX,
                actual,
            })?;
    if actual != expected {
        return Err(PersistenceClassError::WrongLength { expected, actual }.into());
    }
    Ok(())
}

fn decode_separations(
    cursor: &mut PayloadCursor<'_>,
    count: usize,
) -> Result<Vec<f64>, SfsProfileRecordError> {
    let mut separations = Vec::with_capacity(count);
    for index in 0..53 {
        if index >= count {
            break;
        }
        separations.push(f64::from_bits(cursor.read_u64()?));
    }
    Ok(separations)
}

fn normalize_separation_zeros(separations: &mut [f64]) {
    for index in 0..53 {
        if index >= separations.len() {
            break;
        }
        if separations[index] == 0.0 {
            separations[index] = 0.0;
        }
    }
}
