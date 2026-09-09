//! Governed replay-identity discriminants and scalar codecs.

use std::collections::TryReserveError;

use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, StableIdentityError,
};

use crate::causal_contract::{
    canonical_event_type, EffectSignature, EvidenceClass, RuleRole, ShapeVerb,
};
use crate::evaluator::Value;
use crate::types::{BslType, EnumRegistry, EnumTypeId, FieldDecl, FieldKind};
use crate::vocabulary::EnumKind;

/// Maximum bytes one BSL-owned identity section may contain.
pub const MAX_IDENTITY_SECTION_BYTES_V1: usize = 67_108_864;
/// Maximum exact UTF-8 bytes in one governance string without a narrower grammar.
pub const MAX_GOVERNANCE_UTF8_BYTES_V1: usize = 4_194_304;
/// Maximum bytes in one intrinsic identity shared by fuel and replay identity.
pub(crate) const MAX_INTRINSIC_IDENTITY_BYTES_V1: usize = 96;
const MAX_EXACT_BINARY64_INTEGER: f64 = 9_007_199_254_740_992.0;

/// One grammar violation in the shared intrinsic identity validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntrinsicIdentityViolation {
    Empty,
    TooLong { actual: usize },
    NonAscii { index: usize },
    Delimiter { index: usize },
}

/// Validate the one canonical intrinsic identity grammar.
pub(crate) fn validate_intrinsic_identity(value: &str) -> Result<(), IntrinsicIdentityViolation> {
    if value.is_empty() {
        return Err(IntrinsicIdentityViolation::Empty);
    }
    if value.len() > MAX_INTRINSIC_IDENTITY_BYTES_V1 {
        return Err(IntrinsicIdentityViolation::TooLong {
            actual: value.len(),
        });
    }
    for (index, byte) in value
        .bytes()
        .enumerate()
        .take(MAX_INTRINSIC_IDENTITY_BYTES_V1)
    {
        if !byte.is_ascii() {
            return Err(IntrinsicIdentityViolation::NonAscii { index });
        }
        if matches!(byte, b'|' | b'\n' | b'\r') {
            return Err(IntrinsicIdentityViolation::Delimiter { index });
        }
    }
    Ok(())
}

/// A governed BSL identity-codec refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityCodecError {
    /// A graph reference did not resolve through the sealed stable resolver.
    StableIdentity(StableIdentityError),
    /// A binary64 value was NaN or infinite.
    NonFiniteValue,
    /// A stable ratio component was zero or negative.
    NonPositiveRatio,
    /// A stable ratio component was not already on the kernel ratio grid.
    NonCanonicalRatio,
    /// Stable ratio bounds did not contain the value in `(floor, cap]`.
    InvalidRatioBounds,
    /// A stable graph-value tag carried a different stable-element kind.
    StableKeyKindMismatch,
    /// A stored graph lane did not match the field's declared BSL type.
    StoredLaneMismatch,
    /// A stored binary64 value was not an exact governed `Int`.
    NonCanonicalStoredInt {
        /// Supplied binary64 bits.
        bits: u64,
    },
    /// A stored unit-interval scalar fell outside `[0, 1]`.
    StoredUnitInterval {
        /// Supplied binary64 bits.
        bits: u64,
    },
    /// A stored enum ordinal was fractional, negative, or outside its registry.
    InvalidStoredEnumOrdinal {
        /// Supplied binary64 bits.
        bits: u64,
    },
    /// A field type has no scalar graph-storage projection.
    UnsupportedStoredFieldType,
    /// A constant used a runtime enum or graph-reference form.
    InvalidConstantKind,
    /// An option byte was neither absent nor present.
    InvalidOptionByte {
        /// Invalid byte.
        value: u8,
    },
    /// An enum id did not belong to the supplied registry.
    UnknownEnumType {
        /// Foreign or missing enum id.
        id: EnumTypeId,
    },
    /// A governed string failed its grammar or length limit.
    InvalidString {
        /// Semantic field name.
        field: &'static str,
        /// Offending byte position or received length boundary.
        index: usize,
    },
    /// A governed section exceeded its row ceiling.
    RowLimit {
        /// Section name.
        section: &'static str,
        /// Received rows.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Aggregate rows exceeded their governed ceiling.
    AggregateRowLimit {
        /// Received aggregate rows.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// Checked size arithmetic overflowed.
    CapacityOverflow {
        /// Object being sized.
        field: &'static str,
    },
    /// A checked integer conversion failed.
    IntegerConversion {
        /// Converted field.
        field: &'static str,
        /// Unrepresentable value.
        value: usize,
    },
    /// A governed byte ceiling was exceeded.
    ByteLimit {
        /// Encoded object name.
        field: &'static str,
        /// Received byte count.
        actual: usize,
        /// Governed maximum.
        maximum: usize,
    },
    /// A bounded allocation failed.
    Allocation {
        /// Allocated object name.
        field: &'static str,
        /// Requested capacity.
        requested: usize,
    },
    /// A fired count could not fit the governed u64 lane.
    FiredCountOverflow {
        /// Unrepresentable fired count.
        value: usize,
    },
}

/// A BSL value projected onto stable, persistence-safe identities.
///
/// Numeric values carry their canonical scalar representation, and graph
/// references carry sealed [`StableElementKeyV1`] values rather than runtime
/// allocation handles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StableBslValueV1 {
    /// Signed integer.
    Int(i64),
    /// Currency micro-units.
    CurrencyMicroUnits(i128),
    /// Canonical finite binary64 bits.
    RealBits(u64),
    /// Canonical positive ratio bits and its optional declared bounds.
    RatioBits {
        /// Ratio value bits.
        value: u64,
        /// Optional exclusive floor bits.
        floor: Option<u64>,
        /// Optional inclusive cap bits.
        cap: Option<u64>,
    },
    /// Boolean.
    Bool(bool),
    /// Closed enum identity.
    Enum {
        /// Enum type name.
        enum_type: String,
        /// Enum member name.
        member: String,
    },
    /// Stable node identity.
    Node(StableElementKeyV1),
    /// Stable hyperedge identity.
    Hyperedge(StableElementKeyV1),
    /// Stable dyadic-edge identity.
    Edge(StableElementKeyV1),
}

/// Project one runtime BSL value through the sealed stable-element resolver.
///
/// # Errors
/// Returns the same numeric or stable-reference refusal used by
/// [`encode_value_v1`].
pub fn project_stable_value_v1(
    value: &Value,
    resolver: &StableElementResolverV1,
) -> Result<StableBslValueV1, IdentityCodecError> {
    Ok(match value {
        Value::Mass(_) => return Err(IdentityCodecError::InvalidConstantKind),
        Value::Int(value) => StableBslValueV1::Int(*value),
        Value::Currency(value) => StableBslValueV1::CurrencyMicroUnits(value.micro_units()),
        Value::Real(value) => StableBslValueV1::RealBits(canonical_f64_bits(*value)?),
        Value::Ratio { value, floor, cap } => StableBslValueV1::RatioBits {
            value: canonical_f64_bits(value.get())?,
            floor: floor
                .map(|bound| canonical_f64_bits(bound.get()))
                .transpose()?,
            cap: cap
                .map(|bound| canonical_f64_bits(bound.get()))
                .transpose()?,
        },
        Value::Bool(value) => StableBslValueV1::Bool(*value),
        Value::Enum { enum_type, member } => {
            validate_enum_type("ValueV1 enum type", enum_type)?;
            validate_enum_member("ValueV1 enum member", member)?;
            StableBslValueV1::Enum {
                enum_type: copy_identity_string("stable enum type", enum_type)?,
                member: copy_identity_string("stable enum member", member)?,
            }
        }
        Value::NodeRef(node) => StableBslValueV1::Node(resolver.node_key(*node)?.try_owned()?),
        Value::HyperedgeRef(hyperedge) => {
            StableBslValueV1::Hyperedge(resolver.hyperedge_key(*hyperedge)?.try_owned()?)
        }
        Value::EdgeRef(edge) => {
            StableBslValueV1::Edge(resolver.edge_key(&edge.edge_type, edge.source, edge.target)?)
        }
    })
}

/// Reconstruct one declared BSL field from the graph's typed storage lanes.
///
/// This is the sole read-side inverse for material projections from stable
/// graph rows. Binary64 and Currency remain disjoint lanes: the declaration
/// selects exactly one, and a missing, duplicated, or mismatched lane refuses.
/// Stored `Int` values are accepted only inside the exact binary64 integer
/// interval `[-2^53, +2^53]`; a larger integral-looking `f64` may already be
/// a rounded runtime integer and therefore cannot be reconstructed honestly.
///
/// # Errors
/// Returns a lane, numeric-domain, enum-registry, or checked-allocation
/// refusal without producing a partial stable value.
pub fn project_stored_field_value_v1(
    declaration: &FieldDecl,
    binary64_bits: Option<u64>,
    currency_micro_units: Option<i128>,
    enums: &EnumRegistry,
) -> Result<StableBslValueV1, IdentityCodecError> {
    match (&declaration.ty, binary64_bits, currency_micro_units) {
        (BslType::Mass, _, _) => Err(IdentityCodecError::UnsupportedStoredFieldType),
        (BslType::Currency, None, Some(value)) => Ok(StableBslValueV1::CurrencyMicroUnits(value)),
        (BslType::Probability | BslType::Intensity | BslType::Coefficient, Some(bits), None) => {
            let value = f64::from_bits(canonical_f64_bits(f64::from_bits(bits))?);
            if !(0.0..=1.0).contains(&value) {
                return Err(IdentityCodecError::StoredUnitInterval { bits });
            }
            Ok(StableBslValueV1::RealBits(value.to_bits()))
        }
        (BslType::Real, Some(bits), None) => Ok(StableBslValueV1::RealBits(canonical_f64_bits(
            f64::from_bits(bits),
        )?)),
        (BslType::Int, Some(bits), None) => {
            let value = f64::from_bits(bits);
            if !value.is_finite()
                || value.fract() != 0.0
                || value.abs() > MAX_EXACT_BINARY64_INTEGER
            {
                return Err(IdentityCodecError::NonCanonicalStoredInt { bits });
            }
            #[allow(clippy::cast_possible_truncation)]
            let value = value as i64;
            Ok(StableBslValueV1::Int(value))
        }
        (BslType::Enum(enum_type), Some(bits), None) => {
            let value = f64::from_bits(bits);
            if !value.is_finite()
                || value.fract() != 0.0
                || value < 0.0
                || value > f64::from(u32::MAX)
            {
                return Err(IdentityCodecError::InvalidStoredEnumOrdinal { bits });
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let ordinal = value as u32;
            let declaration = enums
                .declaration(*enum_type)
                .ok_or(IdentityCodecError::UnknownEnumType { id: *enum_type })?;
            let ordinal_index = usize::try_from(ordinal)
                .map_err(|_| IdentityCodecError::InvalidStoredEnumOrdinal { bits })?;
            let member = declaration
                .members
                .get(ordinal_index)
                .ok_or(IdentityCodecError::InvalidStoredEnumOrdinal { bits })?;
            Ok(StableBslValueV1::Enum {
                enum_type: copy_identity_string("stable enum type", &declaration.name)?,
                member: copy_identity_string("stable enum member", member)?,
            })
        }
        (BslType::Bool | BslType::NodeSet(_) | BslType::EdgeSet(_), Some(_), None) => {
            Err(IdentityCodecError::UnsupportedStoredFieldType)
        }
        (
            BslType::Currency
            | BslType::Probability
            | BslType::Intensity
            | BslType::Coefficient
            | BslType::Real
            | BslType::Int
            | BslType::Enum(_)
            | BslType::Bool
            | BslType::NodeSet(_)
            | BslType::EdgeSet(_),
            _,
            _,
        ) => Err(IdentityCodecError::StoredLaneMismatch),
    }
}

/// Copy one already-stable BSL value through checked owned allocations.
///
/// This is the sole deep-copy boundary for detached persistence-facing BSL
/// values. It preserves the governed scalar semantics and recursively copies
/// graph identities through [`StableElementKeyV1::try_owned`].
///
/// # Errors
/// Returns the same numeric, enum, stable-key, or allocation refusal used by
/// the canonical stable-value encoder before exposing a partial value.
pub fn try_owned_stable_bsl_value_v1(
    value: &StableBslValueV1,
) -> Result<StableBslValueV1, IdentityCodecError> {
    Ok(match value {
        StableBslValueV1::Int(value) => StableBslValueV1::Int(*value),
        StableBslValueV1::CurrencyMicroUnits(value) => StableBslValueV1::CurrencyMicroUnits(*value),
        StableBslValueV1::RealBits(value) => {
            StableBslValueV1::RealBits(canonical_f64_bits(f64::from_bits(*value))?)
        }
        StableBslValueV1::RatioBits { value, floor, cap } => {
            validate_stable_ratio(*value, *floor, *cap)?;
            StableBslValueV1::RatioBits {
                value: *value,
                floor: *floor,
                cap: *cap,
            }
        }
        StableBslValueV1::Bool(value) => StableBslValueV1::Bool(*value),
        StableBslValueV1::Enum { enum_type, member } => {
            validate_enum_type("StableBslValueV1 enum type", enum_type)?;
            validate_enum_member("StableBslValueV1 enum member", member)?;
            StableBslValueV1::Enum {
                enum_type: copy_identity_string("stable enum type", enum_type)?,
                member: copy_identity_string("stable enum member", member)?,
            }
        }
        StableBslValueV1::Node(key @ StableElementKeyV1::Node { .. }) => {
            StableBslValueV1::Node(key.try_owned()?)
        }
        StableBslValueV1::Hyperedge(key @ StableElementKeyV1::Hyperedge { .. }) => {
            StableBslValueV1::Hyperedge(key.try_owned()?)
        }
        StableBslValueV1::Edge(key @ StableElementKeyV1::Edge { .. }) => {
            StableBslValueV1::Edge(key.try_owned()?)
        }
        StableBslValueV1::Node(_) | StableBslValueV1::Hyperedge(_) | StableBslValueV1::Edge(_) => {
            return Err(IdentityCodecError::StableKeyKindMismatch)
        }
    })
}

fn copy_identity_string(field: &'static str, source: &str) -> Result<String, IdentityCodecError> {
    let mut output = String::new();
    output
        .try_reserve_exact(source.len())
        .map_err(|_: TryReserveError| IdentityCodecError::Allocation {
            field,
            requested: source.len(),
        })?;
    output.push_str(source);
    Ok(output)
}

impl From<StableIdentityError> for IdentityCodecError {
    fn from(value: StableIdentityError) -> Self {
        Self::StableIdentity(value)
    }
}

/// A bounded, allocation-aware identity byte writer.
pub(crate) struct IdentityWriter {
    field: &'static str,
    bytes: Vec<u8>,
}

impl IdentityWriter {
    pub(crate) const fn new(field: &'static str) -> Self {
        Self {
            field,
            bytes: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, value: u8) -> Result<(), IdentityCodecError> {
        self.extend(&[value])
    }

    pub(crate) fn extend(&mut self, value: &[u8]) -> Result<(), IdentityCodecError> {
        let requested = checked_add(self.field, self.bytes.len(), value.len())?;
        if requested > MAX_IDENTITY_SECTION_BYTES_V1 {
            return Err(IdentityCodecError::ByteLimit {
                field: self.field,
                actual: requested,
                maximum: MAX_IDENTITY_SECTION_BYTES_V1,
            });
        }
        self.bytes
            .try_reserve_exact(value.len())
            .map_err(|_: TryReserveError| IdentityCodecError::Allocation {
                field: self.field,
                requested,
            })?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    pub(crate) fn str32(
        &mut self,
        field: &'static str,
        value: &str,
    ) -> Result<(), IdentityCodecError> {
        self.extend(&checked_u32(field, value.len())?.to_be_bytes())?;
        self.extend(value.as_bytes())
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// Encode one exact `ValueV1` into `output`.
///
/// # Errors
/// Returns a semantic, stable-reference, numeric, arithmetic, byte-limit,
/// conversion, or allocation error without modifying `output`.
pub fn encode_value_v1(
    value: &Value,
    resolver: &StableElementResolverV1,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError> {
    let mut writer = IdentityWriter::new("ValueV1");
    encode_value(value, resolver, &mut writer)?;
    append_checked(output, "ValueV1 output", &writer.finish())
}

/// Encode one persistence-safe BSL value without reintroducing runtime handles.
///
/// # Errors
/// Returns a semantic, stable-key-kind, numeric, arithmetic, byte-limit, or
/// allocation error without modifying `output`.
pub fn encode_stable_bsl_value_v1(
    value: &StableBslValueV1,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError> {
    let mut writer = IdentityWriter::new("StableBslValueV1");
    match value {
        StableBslValueV1::Int(value) => encode_int(*value, &mut writer)?,
        StableBslValueV1::CurrencyMicroUnits(value) => encode_currency(*value, &mut writer)?,
        StableBslValueV1::RealBits(value) => encode_real(f64::from_bits(*value), &mut writer)?,
        StableBslValueV1::RatioBits { value, floor, cap } => {
            let (value, floor, cap) = validate_stable_ratio(*value, *floor, *cap)?;
            writer.push(0x04)?;
            writer.extend(&value.get().to_bits().to_be_bytes())?;
            encode_ratio_option(floor, &mut writer)?;
            encode_ratio_option(cap, &mut writer)?;
        }
        StableBslValueV1::Bool(value) => {
            writer.push(0x05)?;
            writer.push(u8::from(*value))?;
        }
        StableBslValueV1::Enum { enum_type, member } => {
            validate_enum_type("ValueV1 enum type", enum_type)?;
            validate_enum_member("ValueV1 enum member", member)?;
            writer.push(0x06)?;
            writer.str32("ValueV1 enum type", enum_type)?;
            writer.str32("ValueV1 enum member", member)?;
        }
        StableBslValueV1::Node(key @ StableElementKeyV1::Node { .. }) => {
            append_stable_key(0x07, key, &mut writer)?;
        }
        StableBslValueV1::Hyperedge(key @ StableElementKeyV1::Hyperedge { .. }) => {
            append_stable_key(0x08, key, &mut writer)?;
        }
        StableBslValueV1::Edge(key @ StableElementKeyV1::Edge { .. }) => {
            append_stable_key(0x09, key, &mut writer)?;
        }
        StableBslValueV1::Node(_) | StableBslValueV1::Hyperedge(_) | StableBslValueV1::Edge(_) => {
            return Err(IdentityCodecError::StableKeyKindMismatch)
        }
    }
    append_checked(output, "StableBslValueV1 output", &writer.finish())
}

fn encode_value(
    value: &Value,
    resolver: &StableElementResolverV1,
    output: &mut IdentityWriter,
) -> Result<(), IdentityCodecError> {
    match value {
        Value::Mass(_) => Err(IdentityCodecError::InvalidConstantKind),
        Value::Int(value) => encode_int(*value, output),
        Value::Currency(value) => encode_currency(value.micro_units(), output),
        Value::Real(value) => encode_real(*value, output),
        Value::Ratio { value, floor, cap } => {
            output.push(0x04)?;
            output.extend(&canonical_f64_bits(value.get())?.to_be_bytes())?;
            encode_ratio_option(*floor, output)?;
            encode_ratio_option(*cap, output)
        }
        Value::Bool(value) => {
            output.push(0x05)?;
            output.push(u8::from(*value))
        }
        Value::Enum { enum_type, member } => {
            validate_enum_type("ValueV1 enum type", enum_type)?;
            validate_enum_member("ValueV1 enum member", member)?;
            output.push(0x06)?;
            output.str32("ValueV1 enum type", enum_type)?;
            output.str32("ValueV1 enum member", member)
        }
        Value::NodeRef(node) => {
            let key = resolver.node_key(*node)?;
            append_stable_key(0x07, key, output)
        }
        Value::HyperedgeRef(hyperedge) => {
            let key = resolver.hyperedge_key(*hyperedge)?;
            append_stable_key(0x08, key, output)
        }
        Value::EdgeRef(edge) => {
            let key = resolver.edge_key(&edge.edge_type, edge.source, edge.target)?;
            append_stable_key(0x09, &key, output)
        }
    }
}

fn encode_int(value: i64, output: &mut IdentityWriter) -> Result<(), IdentityCodecError> {
    output.push(0x01)?;
    output.extend(&value.to_be_bytes())
}

fn encode_currency(value: i128, output: &mut IdentityWriter) -> Result<(), IdentityCodecError> {
    output.push(0x02)?;
    output.extend(&value.to_be_bytes())
}

fn encode_real(value: f64, output: &mut IdentityWriter) -> Result<(), IdentityCodecError> {
    output.push(0x03)?;
    output.extend(&canonical_f64_bits(value)?.to_be_bytes())
}

fn encode_ratio_option(
    value: Option<babylon_kernel::Ratio>,
    output: &mut IdentityWriter,
) -> Result<(), IdentityCodecError> {
    match value {
        None => output.push(0),
        Some(value) => {
            output.push(1)?;
            output.extend(&canonical_f64_bits(value.get())?.to_be_bytes())
        }
    }
}

fn validate_stable_ratio(
    value: u64,
    floor: Option<u64>,
    cap: Option<u64>,
) -> Result<
    (
        babylon_kernel::Ratio,
        Option<babylon_kernel::Ratio>,
        Option<babylon_kernel::Ratio>,
    ),
    IdentityCodecError,
> {
    let value = stable_ratio_component(value)?;
    let floor = floor.map(stable_ratio_component).transpose()?;
    let cap = cap.map(stable_ratio_component).transpose()?;
    if floor.is_some_and(|floor| value <= floor)
        || cap.is_some_and(|cap| value > cap)
        || floor.zip(cap).is_some_and(|(floor, cap)| floor >= cap)
    {
        return Err(IdentityCodecError::InvalidRatioBounds);
    }
    Ok((value, floor, cap))
}

fn stable_ratio_component(value: u64) -> Result<babylon_kernel::Ratio, IdentityCodecError> {
    let raw = f64::from_bits(value);
    canonical_f64_bits(raw)?;
    if raw <= 0.0 {
        return Err(IdentityCodecError::NonPositiveRatio);
    }
    let ratio =
        babylon_kernel::Ratio::new(raw).map_err(|_| IdentityCodecError::NonCanonicalRatio)?;
    if ratio.get().to_bits() != value {
        return Err(IdentityCodecError::NonCanonicalRatio);
    }
    Ok(ratio)
}

fn append_stable_key(
    tag: u8,
    key: &StableElementKeyV1,
    output: &mut IdentityWriter,
) -> Result<(), IdentityCodecError> {
    output.push(tag)?;
    output.extend(&key.canonical_bytes()?)
}

/// Encode one constant, restricted to the six live scalar forms.
///
/// # Errors
/// Returns [`IdentityCodecError::InvalidConstantKind`] for enum or graph
/// references, or the applicable scalar codec failure.
pub fn encode_const_value_v1(
    value: &Value,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError> {
    let mut writer = IdentityWriter::new("ConstValueV1");
    match value {
        Value::Mass(value) => {
            writer.push(0x0a)?;
            writer.extend(&value.nanounits().to_be_bytes())?;
        }
        Value::Int(value) => encode_int(*value, &mut writer)?,
        Value::Currency(value) => encode_currency(value.micro_units(), &mut writer)?,
        Value::Real(value) => encode_real(*value, &mut writer)?,
        Value::Ratio { value, floor, cap } => {
            writer.push(0x04)?;
            writer.extend(&canonical_f64_bits(value.get())?.to_be_bytes())?;
            encode_ratio_option(*floor, &mut writer)?;
            encode_ratio_option(*cap, &mut writer)?;
        }
        Value::Bool(value) => {
            writer.push(0x05)?;
            writer.push(u8::from(*value))?;
        }
        Value::Enum { .. } | Value::NodeRef(_) | Value::HyperedgeRef(_) | Value::EdgeRef(_) => {
            return Err(IdentityCodecError::InvalidConstantKind)
        }
    }
    append_checked(output, "ConstValueV1 output", &writer.finish())
}

/// Encode one governed `BslTypeV1`.
///
/// # Errors
/// Returns an unknown-enum, semantic-string, conversion, byte, or allocation error.
pub fn encode_bsl_type_v1(
    value: &BslType,
    enums: &EnumRegistry,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError> {
    let mut writer = IdentityWriter::new("BslTypeV1");
    match value {
        BslType::Mass => return Err(IdentityCodecError::UnsupportedStoredFieldType),
        BslType::Probability => writer.push(0x01)?,
        BslType::Intensity => writer.push(0x02)?,
        BslType::Coefficient => writer.push(0x03)?,
        BslType::Currency => writer.push(0x04)?,
        BslType::Real => writer.push(0x05)?,
        BslType::Int => writer.push(0x06)?,
        BslType::Bool => writer.push(0x07)?,
        BslType::Enum(id) => {
            let declaration = enums
                .declaration(*id)
                .ok_or(IdentityCodecError::UnknownEnumType { id: *id })?;
            validate_enum_type("BslTypeV1 enum type", &declaration.name)?;
            writer.push(0x08)?;
            writer.str32("BslTypeV1 enum type", &declaration.name)?;
        }
        BslType::NodeSet(member) => {
            validate_enum_member("BslTypeV1 NodeType member", member)?;
            writer.push(0x09)?;
            writer.str32("BslTypeV1 NodeType member", member)?;
        }
        BslType::EdgeSet(member) => {
            validate_enum_member("BslTypeV1 EdgeType member", member)?;
            writer.push(0x0a)?;
            writer.str32("BslTypeV1 EdgeType member", member)?;
        }
    }
    append_checked(output, "BslTypeV1 output", &writer.finish())
}

/// Return the governed `FieldKindV1` tag.
#[must_use]
pub const fn encode_field_kind_v1(value: FieldKind) -> u8 {
    match value {
        FieldKind::Intensive => 0x01,
        FieldKind::Extensive => 0x02,
        FieldKind::NotApplicable => 0x03,
    }
}

/// Return the governed `RuleRoleV1` tag.
#[must_use]
pub const fn encode_rule_role_v1(value: RuleRole) -> u8 {
    match value {
        RuleRole::Mechanic => 0x01,
        RuleRole::Recognizer => 0x02,
        RuleRole::ExternalEvent => 0x03,
        RuleRole::Intent => 0x04,
    }
}

/// Return the governed `EvidenceClassV1` tag.
#[must_use]
pub const fn encode_evidence_class_v1(value: EvidenceClass) -> u8 {
    match value {
        EvidenceClass::Observed => 0x01,
        EvidenceClass::Derived => 0x02,
        EvidenceClass::Calibrated => 0x03,
        EvidenceClass::Designed => 0x04,
    }
}

/// Return the governed `ShapeVerbV1` tag.
#[must_use]
pub const fn encode_shape_verb_v1(value: ShapeVerb) -> u8 {
    match value {
        ShapeVerb::AddNode => 0x01,
        ShapeVerb::RemoveNode => 0x02,
        ShapeVerb::AddEdge => 0x03,
        ShapeVerb::RemoveEdge => 0x04,
        ShapeVerb::AddHyperedge => 0x05,
        ShapeVerb::RemoveHyperedge => 0x06,
    }
}

/// Return the governed `EnumKindV1` tag.
#[must_use]
pub const fn encode_enum_kind_v1(value: EnumKind) -> u8 {
    match value {
        EnumKind::NodeType => 0x01,
        EnumKind::EdgeType => 0x02,
        EnumKind::HyperedgeType => 0x03,
        EnumKind::EventType => 0x04,
    }
}

/// Encode one governed `EffectSignatureV1`.
///
/// # Errors
/// Returns a semantic-string, conversion, byte, or allocation error.
pub fn encode_effect_signature_v1(
    value: &EffectSignature,
    output: &mut Vec<u8>,
) -> Result<(), IdentityCodecError> {
    let mut writer = IdentityWriter::new("EffectSignatureV1");
    match value {
        EffectSignature::NodeField(qname) => encode_effect_field(0x01, qname, &mut writer)?,
        EffectSignature::EdgeField(qname) => encode_effect_field(0x02, qname, &mut writer)?,
        EffectSignature::HyperedgeField(qname) => encode_effect_field(0x03, qname, &mut writer)?,
        EffectSignature::Event(event) => {
            let canonical = canonical_event_name_v1(event)?;
            writer.push(0x04)?;
            writer.str32("effect event", &canonical)?;
        }
        EffectSignature::Shape(verb) => {
            writer.push(0x05)?;
            writer.push(encode_shape_verb_v1(*verb))?;
        }
    }
    append_checked(output, "EffectSignatureV1 output", &writer.finish())
}

fn encode_effect_field(
    tag: u8,
    qname: &str,
    output: &mut IdentityWriter,
) -> Result<(), IdentityCodecError> {
    validate_qname("effect field", qname)?;
    output.push(tag)?;
    output.str32("effect field", qname)
}

/// Canonicalize a bare or full event name through the shared causal codec.
///
/// # Errors
/// Returns a semantic-string error for malformed event identities.
pub fn canonical_event_name_v1(value: &str) -> Result<String, IdentityCodecError> {
    let canonical = canonical_event_type(value).map_err(|_| IdentityCodecError::InvalidString {
        field: "event name",
        index: value.len(),
    })?;
    let member = canonical
        .strip_prefix("EventType/")
        .ok_or(IdentityCodecError::InvalidString {
            field: "event name",
            index: 0,
        })?;
    validate_enum_member("event member", member)?;
    Ok(canonical)
}

/// Decode an exact option-presence byte.
///
/// # Errors
/// Returns [`IdentityCodecError::InvalidOptionByte`] unless `value` is 0 or 1.
pub const fn decode_option_presence_v1(value: u8) -> Result<bool, IdentityCodecError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(IdentityCodecError::InvalidOptionByte { value }),
    }
}

pub(crate) fn validate_symbol(field: &'static str, value: &str) -> Result<(), IdentityCodecError> {
    validate_segment(field, value, false)
}

pub(crate) fn validate_governance_text(
    field: &'static str,
    value: &str,
) -> Result<(), IdentityCodecError> {
    if value.len() <= MAX_GOVERNANCE_UTF8_BYTES_V1 {
        Ok(())
    } else {
        Err(IdentityCodecError::InvalidString {
            field,
            index: value.len(),
        })
    }
}

pub(crate) fn validate_qname(field: &'static str, value: &str) -> Result<(), IdentityCodecError> {
    if value.is_empty() || value.len() > 128 {
        return Err(IdentityCodecError::InvalidString {
            field,
            index: value.len(),
        });
    }
    let mut count = 0usize;
    for segment in value.split('/').take(5) {
        if count == 4 {
            return Err(IdentityCodecError::InvalidString {
                field,
                index: value.len(),
            });
        }
        validate_segment(field, segment, false)?;
        count = count.saturating_add(1);
    }
    Ok(())
}

pub(crate) fn validate_enum_type(
    field: &'static str,
    value: &str,
) -> Result<(), IdentityCodecError> {
    validate_segment(field, value, true)
}

pub(crate) fn validate_enum_member(
    field: &'static str,
    value: &str,
) -> Result<(), IdentityCodecError> {
    if value.is_empty() || value.len() > 64 {
        return Err(IdentityCodecError::InvalidString {
            field,
            index: value.len(),
        });
    }
    for (index, byte) in value.bytes().enumerate().take(64) {
        let valid = if index == 0 {
            byte.is_ascii_uppercase()
        } else {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_'
        };
        if !valid {
            return Err(IdentityCodecError::InvalidString { field, index });
        }
    }
    Ok(())
}

fn validate_segment(
    field: &'static str,
    value: &str,
    enum_type: bool,
) -> Result<(), IdentityCodecError> {
    if value.is_empty() || value.len() > 64 {
        return Err(IdentityCodecError::InvalidString {
            field,
            index: value.len(),
        });
    }
    for (index, byte) in value.bytes().enumerate().take(64) {
        let valid = if enum_type {
            if index == 0 {
                byte.is_ascii_uppercase()
            } else {
                byte.is_ascii_alphanumeric()
            }
        } else if index == 0 {
            byte.is_ascii_lowercase()
        } else {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
        };
        if !valid {
            return Err(IdentityCodecError::InvalidString { field, index });
        }
    }
    Ok(())
}

/// Return canonical finite binary64 bits, normalizing negative zero.
///
/// # Errors
/// Returns [`IdentityCodecError::NonFiniteValue`] for NaN or either infinity.
pub fn canonical_f64_bits(value: f64) -> Result<u64, IdentityCodecError> {
    if !value.is_finite() {
        return Err(IdentityCodecError::NonFiniteValue);
    }
    Ok(if value == 0.0 { 0 } else { value.to_bits() })
}

pub(crate) fn checked_u32(field: &'static str, value: usize) -> Result<u32, IdentityCodecError> {
    u32::try_from(value).map_err(|_| IdentityCodecError::IntegerConversion { field, value })
}

pub(crate) fn checked_add(
    field: &'static str,
    left: usize,
    right: usize,
) -> Result<usize, IdentityCodecError> {
    left.checked_add(right)
        .ok_or(IdentityCodecError::CapacityOverflow { field })
}

fn append_checked(
    output: &mut Vec<u8>,
    field: &'static str,
    value: &[u8],
) -> Result<(), IdentityCodecError> {
    let requested = checked_add(field, output.len(), value.len())?;
    if requested > MAX_IDENTITY_SECTION_BYTES_V1 {
        return Err(IdentityCodecError::ByteLimit {
            field,
            actual: requested,
            maximum: MAX_IDENTITY_SECTION_BYTES_V1,
        });
    }
    output
        .try_reserve_exact(value.len())
        .map_err(|_: TryReserveError| IdentityCodecError::Allocation { field, requested })?;
    output.extend_from_slice(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::stable_element::StableElementResolverV1;

    use babylon_graph::stable_element::StableElementKeyV1;

    use super::{
        encode_stable_bsl_value_v1, project_stable_value_v1, project_stored_field_value_v1,
        IdentityCodecError, StableBslValueV1,
    };
    use crate::evaluator::Value;
    use crate::types::{BslType, EnumRegistry, EnumTypeId, FieldDecl, FieldKind};

    fn empty_resolver() -> StableElementResolverV1 {
        StableElementResolverV1::seal(
            &MemoryGraph::new(),
            "test/identity-codec",
            &HashMap::new(),
            &HashMap::new(),
        )
        .expect("the empty graph has one sealed resolver")
    }

    #[test]
    fn stable_projection_refuses_invalid_enum_identity() {
        let resolver = empty_resolver();
        let invalid_type = Value::Enum {
            enum_type: "bad type".to_owned(),
            member: "VALID".to_owned(),
        };
        let invalid_member = Value::Enum {
            enum_type: "EventType".to_owned(),
            member: "not_valid".to_owned(),
        };

        assert_eq!(
            project_stable_value_v1(&invalid_type, &resolver),
            Err(IdentityCodecError::InvalidString {
                field: "ValueV1 enum type",
                index: 0,
            })
        );
        assert_eq!(
            project_stable_value_v1(&invalid_member, &resolver),
            Err(IdentityCodecError::InvalidString {
                field: "ValueV1 enum member",
                index: 0,
            })
        );
    }

    #[test]
    fn stable_encoder_closes_scalar_and_key_invariants_without_partial_output() {
        let mut real = Vec::new();
        encode_stable_bsl_value_v1(&StableBslValueV1::RealBits((-0.0_f64).to_bits()), &mut real)
            .expect("negative zero canonicalizes");
        assert_eq!(real, [vec![0x03], 0_u64.to_be_bytes().to_vec()].concat());

        let valid_ratio = StableBslValueV1::RatioBits {
            value: 0.5_f64.to_bits(),
            floor: Some(0.25_f64.to_bits()),
            cap: Some(1.0_f64.to_bits()),
        };
        let mut ratio = Vec::new();
        encode_stable_bsl_value_v1(&valid_ratio, &mut ratio).expect("positive ratios encode");
        assert_eq!(ratio[0], 0x04);

        for (invalid, expected) in [
            (
                StableBslValueV1::RatioBits {
                    value: 0.0_f64.to_bits(),
                    floor: None,
                    cap: None,
                },
                IdentityCodecError::NonPositiveRatio,
            ),
            (
                StableBslValueV1::RatioBits {
                    value: f64::NAN.to_bits(),
                    floor: None,
                    cap: None,
                },
                IdentityCodecError::NonFiniteValue,
            ),
            (
                StableBslValueV1::RatioBits {
                    value: 0.123_456_789_f64.to_bits(),
                    floor: None,
                    cap: None,
                },
                IdentityCodecError::NonCanonicalRatio,
            ),
            (
                StableBslValueV1::RatioBits {
                    value: 0.5_f64.to_bits(),
                    floor: Some(0.5_f64.to_bits()),
                    cap: Some(1.0_f64.to_bits()),
                },
                IdentityCodecError::InvalidRatioBounds,
            ),
            (
                StableBslValueV1::RatioBits {
                    value: 0.5_f64.to_bits(),
                    floor: Some(0.25_f64.to_bits()),
                    cap: Some(0.25_f64.to_bits()),
                },
                IdentityCodecError::InvalidRatioBounds,
            ),
            (
                StableBslValueV1::RatioBits {
                    value: 1.0_f64.to_bits(),
                    floor: Some(0.25_f64.to_bits()),
                    cap: Some(0.5_f64.to_bits()),
                },
                IdentityCodecError::InvalidRatioBounds,
            ),
            (
                StableBslValueV1::Node(StableElementKeyV1::Hyperedge {
                    scenario: "test/identity-codec".to_owned(),
                    local_name: "group".to_owned(),
                }),
                IdentityCodecError::StableKeyKindMismatch,
            ),
        ] {
            let mut output = vec![0xaa];
            assert_eq!(
                encode_stable_bsl_value_v1(&invalid, &mut output),
                Err(expected)
            );
            assert_eq!(output, vec![0xaa]);
        }

        let invalid_enum = StableBslValueV1::Enum {
            enum_type: "bad type".to_owned(),
            member: "VALID".to_owned(),
        };
        let mut output = vec![0xbb];
        assert_eq!(
            encode_stable_bsl_value_v1(&invalid_enum, &mut output),
            Err(IdentityCodecError::InvalidString {
                field: "ValueV1 enum type",
                index: 0,
            })
        );
        assert_eq!(output, vec![0xbb]);
    }

    fn field(ty: BslType) -> FieldDecl {
        FieldDecl {
            ty,
            kind: FieldKind::Extensive,
        }
    }

    #[test]
    fn stored_field_projection_closes_numeric_lane_and_enum_semantics() {
        let enums = EnumRegistry::default();
        let int = field(BslType::Int);
        for (exact, expected) in [
            (-9_007_199_254_740_992.0_f64, -9_007_199_254_740_992_i64),
            (-1.0, -1),
            (0.0, 0),
            (1.0, 1),
            (9_007_199_254_740_992.0, 9_007_199_254_740_992),
        ] {
            assert_eq!(
                project_stored_field_value_v1(&int, Some(exact.to_bits()), None, &enums),
                Ok(StableBslValueV1::Int(expected))
            );
        }
        for refused in [1.5_f64, 9_007_199_254_740_994.0, -9_007_199_254_740_994.0] {
            assert_eq!(
                project_stored_field_value_v1(&int, Some(refused.to_bits()), None, &enums),
                Err(IdentityCodecError::NonCanonicalStoredInt {
                    bits: refused.to_bits()
                })
            );
        }
        assert_eq!(
            project_stored_field_value_v1(&int, None, Some(1), &enums),
            Err(IdentityCodecError::StoredLaneMismatch)
        );

        let unit = field(BslType::Intensity);
        assert_eq!(
            project_stored_field_value_v1(&unit, Some((-0.0_f64).to_bits()), None, &enums),
            Ok(StableBslValueV1::RealBits(0.0_f64.to_bits()))
        );
        assert_eq!(
            project_stored_field_value_v1(&unit, Some(1.25_f64.to_bits()), None, &enums),
            Err(IdentityCodecError::StoredUnitInterval {
                bits: 1.25_f64.to_bits()
            })
        );

        let real = field(BslType::Real);
        assert_eq!(
            project_stored_field_value_v1(&real, Some((-0.0_f64).to_bits()), None, &enums),
            Ok(StableBslValueV1::RealBits(0.0_f64.to_bits()))
        );
        let currency = field(BslType::Currency);
        assert_eq!(
            project_stored_field_value_v1(&currency, None, Some(7), &enums),
            Ok(StableBslValueV1::CurrencyMicroUnits(7))
        );
        assert_eq!(
            project_stored_field_value_v1(&currency, Some(7.0_f64.to_bits()), None, &enums),
            Err(IdentityCodecError::StoredLaneMismatch)
        );

        for unsupported in [
            BslType::Bool,
            BslType::NodeSet("TERRITORY"),
            BslType::EdgeSet("PRESENCE"),
        ] {
            assert_eq!(
                project_stored_field_value_v1(
                    &field(unsupported),
                    Some(0.0_f64.to_bits()),
                    None,
                    &enums,
                ),
                Err(IdentityCodecError::UnsupportedStoredFieldType)
            );
        }
    }

    #[test]
    fn stored_enum_projection_refuses_foreign_or_invalid_ordinals() {
        let mut enums = EnumRegistry::default();
        let territory_type = enums
            .declare(
                "TerritoryType",
                &["CORE".to_owned(), "PERIPHERY".to_owned()],
            )
            .unwrap();
        let declared = field(BslType::Enum(territory_type));
        assert_eq!(
            project_stored_field_value_v1(&declared, Some(1.0_f64.to_bits()), None, &enums,),
            Ok(StableBslValueV1::Enum {
                enum_type: "TerritoryType".to_owned(),
                member: "PERIPHERY".to_owned(),
            })
        );
        for invalid in [-1.0_f64, 0.5, 2.0] {
            assert_eq!(
                project_stored_field_value_v1(&declared, Some(invalid.to_bits()), None, &enums,),
                Err(IdentityCodecError::InvalidStoredEnumOrdinal {
                    bits: invalid.to_bits()
                })
            );
        }
        assert_eq!(
            project_stored_field_value_v1(
                &field(BslType::Enum(EnumTypeId(99))),
                Some(0.0_f64.to_bits()),
                None,
                &enums,
            ),
            Err(IdentityCodecError::UnknownEnumType { id: EnumTypeId(99) })
        );
    }
}
