//! Tick-owned material-state sources and detached canonical report rows.
//!
//! Material state is deliberately outside [`crate::replay_identity::StableWorldV1`],
//! tick payload, and tick-content identity. The replay session owns one explicit
//! checked dynamic-H3 source and publishes separately owned graph-derived and
//! dynamic canonical projections only after every identity and allocation check succeeds.

use std::collections::TryReserveError;

use babylon_bsl::identity_codec::{
    encode_stable_bsl_value_v1, project_stored_field_value_v1, IdentityCodecError, StableBslValueV1,
};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::EnumRegistry;
use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, StableIdentityError,
};
use babylon_graph::stable_state::{StableGraphEdgeRowV1, StableGraphStateV1};
use babylon_kernel::{sha256_of, H3CellId};

use crate::h3_runtime::{
    MichiganDynamicHexFoundationErrorV1, MichiganDynamicHexFoundationV1,
    MichiganDynamicHexValueBitsV1, MichiganDynamicHexValuesV1,
};
#[cfg(test)]
use crate::h3_runtime::{
    MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1,
    MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1, MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1,
};

const MATERIAL_ROW_DOMAIN: &[u8] = b"babylon.material-state-row\0";
const MATERIAL_SOURCE_DOMAIN: &[u8] = b"babylon.material-state-source\0";
const MATERIAL_ROWS_DOMAIN: &[u8] = b"babylon.material-state-rows\0";
const MATERIAL_LAYOUT_VERSION_V1: u32 = 1;
const MAX_MATERIAL_BYTES_V1: usize = 64 * 1024 * 1024;

/// A typed material-state construction or projection refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialStateErrorV1 {
    /// The replay reference differs from the checked dynamic-H3 foundation.
    ReferenceBundleMismatch {
        expected: [u8; 32],
        actual: [u8; 32],
    },
    /// A test-only dynamic-H3 source value violated the foundation domains.
    DynamicFoundation(MichiganDynamicHexFoundationErrorV1),
    /// One territory identity was malformed or absent from the sealed graph.
    TerritoryIdentity(StableIdentityError),
    /// One organization identity was malformed or absent from the sealed graph.
    OrganizationIdentity(StableIdentityError),
    /// A sealed territory identity did not carry `NodeType/TERRITORY`.
    TerritoryNodeType,
    /// A stable territory attribute had no declared BSL field.
    TerritoryFieldUndeclared,
    /// A stable territory attribute did not have exact `territory/` ownership.
    TerritoryFieldOwner,
    /// A stable organization attribute had no declared BSL field.
    OrganizationFieldUndeclared,
    /// A stable organization attribute did not have exact `organization/` ownership.
    OrganizationFieldOwner,
    /// A stable organization did not carry the required declared `OrgKind` value.
    OrganizationKind,
    /// A sealed organization identity did not carry `NodeType/ORGANIZATION`.
    OrganizationNodeType,
    /// An organization named a territory without the exact sealed PRESENCE edge.
    OrganizationTerritoryPresence,
    /// A stable named value failed its governed codec.
    StableValue(IdentityCodecError),
    /// A named field sequence was not strictly ordered and duplicate-free.
    NamedFieldOrder { family: &'static str },
    /// A source row sequence was not strictly ordered and duplicate-free.
    SourceRowOrder { family: &'static str },
    /// A closed material enum carried another type or member.
    ClosedEnum { field: &'static str },
    /// The derived world-register row was not the one governed row.
    WorldRegister,
    /// Checked size arithmetic overflowed.
    CapacityOverflow { field: &'static str },
    /// A collection length could not fit the governed u32 lane.
    IntegerConversion { field: &'static str, value: usize },
    /// Canonical bytes exceeded the governed material ceiling.
    ByteLimit {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    /// One checked owned allocation failed.
    Allocation {
        field: &'static str,
        requested: usize,
    },
}

impl std::fmt::Display for MaterialStateErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "material state refused: {self:?}")
    }
}

impl std::error::Error for MaterialStateErrorV1 {}

/// Testable pre-reservation boundary used by every detached material copy.
pub(crate) trait MaterialAllocationGate {
    fn before_reserve(
        &self,
        field: &'static str,
        requested: usize,
    ) -> Result<(), MaterialStateErrorV1>;
}

pub(crate) struct ProductionMaterialAllocationGate;

pub(crate) struct MaterialProjectionContextV1<'a> {
    stable_graph: &'a StableGraphStateV1,
    scenario_scope: &'a str,
    types: &'a TypeEnv,
    enums: &'a EnumRegistry,
    resolver: &'a StableElementResolverV1,
    gate: &'a dyn MaterialAllocationGate,
}

impl<'a> MaterialProjectionContextV1<'a> {
    pub(crate) fn new(
        stable_graph: &'a StableGraphStateV1,
        scenario_scope: &'a str,
        types: &'a TypeEnv,
        enums: &'a EnumRegistry,
        resolver: &'a StableElementResolverV1,
        gate: &'a dyn MaterialAllocationGate,
    ) -> Self {
        Self {
            stable_graph,
            scenario_scope,
            types,
            enums,
            resolver,
            gate,
        }
    }
}

impl MaterialAllocationGate for ProductionMaterialAllocationGate {
    fn before_reserve(
        &self,
        _field: &'static str,
        _requested: usize,
    ) -> Result<(), MaterialStateErrorV1> {
        Ok(())
    }
}

/// One session-owned dynamic-H3 runtime row without report bytes.
#[derive(Debug, PartialEq, Eq)]
struct DynamicHexRuntimeRowV1 {
    cell_id: H3CellId,
    value_bits: [u64; 9],
}

/// The opaque tick-owned dynamic-H3 runtime authority.
///
/// This holds only source values and the three identities proved by the
/// checked foundation. Canonical report rows are a later fallible projection.
#[derive(Debug, PartialEq, Eq)]
struct DynamicHexRuntimeV1 {
    rows: Vec<DynamicHexRuntimeRowV1>,
    source_r7_digest: [u8; 32],
    reference_bundle_digest: [u8; 32],
    artifact_sha256: [u8; 32],
}

impl DynamicHexRuntimeV1 {
    fn try_from_foundation(
        foundation: &MichiganDynamicHexFoundationV1,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        let mut rows = reserve_vec("material dynamic rows", foundation.rows().len(), gate)?;
        for source in foundation.rows() {
            rows.push(DynamicHexRuntimeRowV1 {
                cell_id: source.cell_id(),
                value_bits: source.value_bits(),
            });
        }
        Ok(Self {
            rows,
            source_r7_digest: foundation.source_r7_digest(),
            reference_bundle_digest: foundation.reference_bundle_digest(),
            artifact_sha256: foundation.artifact_sha256(),
        })
    }

    fn try_detached(
        &self,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        let mut rows = reserve_vec("material dynamic rows", self.rows.len(), gate)?;
        for row in &self.rows {
            rows.push(DynamicHexRuntimeRowV1 {
                cell_id: row.cell_id,
                value_bits: row.value_bits,
            });
        }
        Ok(Self {
            rows,
            source_r7_digest: self.source_r7_digest,
            reference_bundle_digest: self.reference_bundle_digest,
            artifact_sha256: self.artifact_sha256,
        })
    }

    fn try_restore_from_rows(
        &self,
        stored: &[DynamicHexStateRowV1],
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        if stored.len() != self.rows.len()
            || stored
                .iter()
                .zip(&self.rows)
                .any(|(left, right)| left.cell_id() != right.cell_id)
        {
            return Err(MaterialStateErrorV1::SourceRowOrder {
                family: "dynamic hex checkpoint identity",
            });
        }
        let mut restored = self.try_detached(gate)?;
        for (stored_row, runtime_row) in stored.iter().zip(&mut restored.rows) {
            runtime_row.value_bits = stored_row.value_bits();
        }
        Ok(restored)
    }

    #[cfg(test)]
    fn try_fixture(
        rows: Vec<(H3CellId, MichiganDynamicHexValueBitsV1)>,
    ) -> Result<Self, MaterialStateErrorV1> {
        let mut runtime_rows = reserve_vec(
            "material dynamic rows",
            rows.len(),
            &ProductionMaterialAllocationGate,
        )?;
        for (cell_id, value_bits) in rows {
            let values = MichiganDynamicHexValuesV1::try_new(value_bits)
                .map_err(MaterialStateErrorV1::DynamicFoundation)?;
            runtime_rows.push(DynamicHexRuntimeRowV1 {
                cell_id,
                value_bits: values.value_bits(),
            });
        }
        Ok(Self {
            rows: runtime_rows,
            source_r7_digest: MICHIGAN_DYNAMIC_HEX_SOURCE_R7_DIGEST_V1,
            reference_bundle_digest: MICHIGAN_DYNAMIC_HEX_REFERENCE_BUNDLE_DIGEST_V1,
            artifact_sha256: MICHIGAN_DYNAMIC_HEX_FOUNDATION_ARTIFACT_SHA256_V1,
        })
    }

    fn rows(&self) -> &[DynamicHexRuntimeRowV1] {
        &self.rows
    }

    const fn reference_bundle_digest(&self) -> [u8; 32] {
        self.reference_bundle_digest
    }
}

struct MaterialWriter<'a> {
    field: &'static str,
    bytes: Vec<u8>,
    gate: &'a dyn MaterialAllocationGate,
}

impl<'a> MaterialWriter<'a> {
    fn new(field: &'static str, gate: &'a dyn MaterialAllocationGate) -> Self {
        Self {
            field,
            bytes: Vec::new(),
            gate,
        }
    }

    fn push(&mut self, value: u8) -> Result<(), MaterialStateErrorV1> {
        self.extend(&[value])
    }

    fn extend(&mut self, value: &[u8]) -> Result<(), MaterialStateErrorV1> {
        let requested = checked_add(self.field, self.bytes.len(), value.len())?;
        if requested > MAX_MATERIAL_BYTES_V1 {
            return Err(MaterialStateErrorV1::ByteLimit {
                field: self.field,
                actual: requested,
                maximum: MAX_MATERIAL_BYTES_V1,
            });
        }
        self.gate.before_reserve(self.field, value.len())?;
        self.bytes
            .try_reserve_exact(value.len())
            .map_err(|_: TryReserveError| MaterialStateErrorV1::Allocation {
                field: self.field,
                requested: value.len(),
            })?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    fn str32(&mut self, value: &str) -> Result<(), MaterialStateErrorV1> {
        self.extend(&checked_u32(self.field, value.len())?.to_be_bytes())?;
        self.extend(value.as_bytes())
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

/// One derived world-register material row.
#[derive(Debug, PartialEq, Eq)]
pub struct WorldRegisterRowV1 {
    qname: String,
    value: StableBslValueV1,
    canonical_bytes: Vec<u8>,
}

impl WorldRegisterRowV1 {
    /// Construct the exact derived material register row.
    ///
    /// # Errors
    /// Returns a world-register, stable-value, or allocation refusal.
    pub fn try_new(qname: String, value: StableBslValueV1) -> Result<Self, MaterialStateErrorV1> {
        Self::try_new_with_gate(qname, value, &ProductionMaterialAllocationGate)
    }

    fn try_new_with_gate(
        qname: String,
        value: StableBslValueV1,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        if qname != "world/completed-tick"
            || !matches!(value, StableBslValueV1::Int(completed_tick) if completed_tick >= 0)
        {
            return Err(MaterialStateErrorV1::WorldRegister);
        }
        let canonical_bytes = encode_world_register(&qname, &value, gate)?;
        Ok(Self {
            qname,
            value,
            canonical_bytes,
        })
    }

    /// Borrow exact canonical row bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Borrow the governed world-register name.
    #[must_use]
    pub fn qname(&self) -> &str {
        &self.qname
    }

    /// Borrow the exact stable register value.
    #[must_use]
    pub const fn value(&self) -> &StableBslValueV1 {
        &self.value
    }
}

/// One stable territory material row.
#[derive(Debug, PartialEq, Eq)]
pub struct TerritoryStateRowV1 {
    territory_id: StableElementKeyV1,
    ordered_fields: Vec<(String, StableBslValueV1)>,
    primary_key: Vec<u8>,
    canonical_bytes: Vec<u8>,
}

impl TerritoryStateRowV1 {
    /// Construct one exact externally loaded territory row.
    ///
    /// # Errors
    /// Returns the first stable identity, field-order, value, or allocation
    /// refusal.
    pub fn try_new(
        territory_id: StableElementKeyV1,
        ordered_fields: Vec<(String, StableBslValueV1)>,
    ) -> Result<Self, MaterialStateErrorV1> {
        Self::try_from_projection(
            territory_id,
            ordered_fields,
            &ProductionMaterialAllocationGate,
        )
    }

    fn try_from_projection(
        territory_id: StableElementKeyV1,
        ordered_fields: Vec<(String, StableBslValueV1)>,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        if !matches!(territory_id, StableElementKeyV1::Node { .. }) {
            return Err(MaterialStateErrorV1::TerritoryIdentity(
                StableIdentityError::ElementNotSealed,
            ));
        }
        validate_name_order(
            "territory",
            ordered_fields.iter().map(|(name, _)| name.as_str()),
        )?;
        gate.before_reserve("material territory key", 1)?;
        let primary_key = territory_id
            .canonical_bytes()
            .map_err(map_territory_identity)?;
        gate.before_reserve("material territory row", 1)?;
        let canonical_bytes = encode_territory(&territory_id, &ordered_fields, gate)?;
        Ok(Self {
            territory_id,
            ordered_fields,
            primary_key,
            canonical_bytes,
        })
    }

    /// Borrow the exact stable territory identity.
    #[must_use]
    pub const fn territory_id(&self) -> &StableElementKeyV1 {
        &self.territory_id
    }

    /// Borrow the strict UTF-8 field-name ordered stable values.
    #[must_use]
    pub fn ordered_fields(&self) -> &[(String, StableBslValueV1)] {
        &self.ordered_fields
    }

    /// Borrow exact canonical row bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// One mutable dynamic-H3 material row.
#[derive(Debug, PartialEq, Eq)]
pub struct DynamicHexStateRowV1 {
    cell_id: H3CellId,
    c: u64,
    v: u64,
    s: u64,
    k: u64,
    biocapacity_stock: u64,
    energy_stock: u64,
    raw_material_stock: u64,
    internet_access_pct: u64,
    surveillance_coupling: u64,
    canonical_bytes: Vec<u8>,
}

impl DynamicHexStateRowV1 {
    /// Construct one exact externally loaded dynamic-H3 row.
    ///
    /// # Errors
    /// Returns a cell-domain, value-domain, or allocation refusal.
    pub fn try_new(
        cell_id: H3CellId,
        value_bits: MichiganDynamicHexValueBitsV1,
    ) -> Result<Self, MaterialStateErrorV1> {
        let values = MichiganDynamicHexValuesV1::try_new(value_bits)
            .map_err(MaterialStateErrorV1::DynamicFoundation)?;
        Self::try_from_validated(
            cell_id,
            values.value_bits(),
            &ProductionMaterialAllocationGate,
        )
    }

    fn try_from_runtime(
        source: &DynamicHexRuntimeRowV1,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        Self::try_from_validated(source.cell_id, source.value_bits, gate)
    }

    fn try_from_validated(
        cell_id: H3CellId,
        value_bits: [u64; 9],
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        gate.before_reserve("material dynamic row", 1)?;
        let [c, v, s, k, biocapacity_stock, energy_stock, raw_material_stock, internet_access_pct, surveillance_coupling] =
            value_bits;
        let canonical_bytes = encode_dynamic_hex(
            cell_id,
            [
                c,
                v,
                s,
                k,
                biocapacity_stock,
                energy_stock,
                raw_material_stock,
                internet_access_pct,
                surveillance_coupling,
            ],
            gate,
        )?;
        Ok(Self {
            cell_id,
            c,
            v,
            s,
            k,
            biocapacity_stock,
            energy_stock,
            raw_material_stock,
            internet_access_pct,
            surveillance_coupling,
            canonical_bytes,
        })
    }

    /// Return the literal validated H3 identity.
    #[must_use]
    pub const fn cell_id(&self) -> H3CellId {
        self.cell_id
    }

    /// Return all nine exact source bits in governed lane order.
    #[must_use]
    pub const fn value_bits(&self) -> [u64; 9] {
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

    /// Borrow exact canonical row bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// One mutable organization row with stable territory PRESENCE identities.
#[derive(Debug, PartialEq, Eq)]
pub struct OrganizationStateRowV1 {
    organization_id: StableElementKeyV1,
    organization_kind: StableBslValueV1,
    ordered_territory_ids: Vec<StableElementKeyV1>,
    ordered_fields: Vec<(String, StableBslValueV1)>,
    primary_key: Vec<u8>,
    canonical_bytes: Vec<u8>,
}

impl OrganizationStateRowV1 {
    /// Construct one exact externally loaded organization row.
    ///
    /// # Errors
    /// Returns the first stable identity, organization-kind, ordering, value,
    /// or allocation refusal.
    pub fn try_new(
        organization_id: StableElementKeyV1,
        organization_kind: StableBslValueV1,
        ordered_territory_ids: Vec<StableElementKeyV1>,
        ordered_fields: Vec<(String, StableBslValueV1)>,
    ) -> Result<Self, MaterialStateErrorV1> {
        Self::try_from_projection(
            organization_id,
            organization_kind,
            ordered_territory_ids,
            ordered_fields,
            &ProductionMaterialAllocationGate,
        )
    }

    fn try_from_projection(
        organization_id: StableElementKeyV1,
        organization_kind: StableBslValueV1,
        ordered_territory_ids: Vec<StableElementKeyV1>,
        ordered_fields: Vec<(String, StableBslValueV1)>,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        require_node_key(&organization_id, MaterialIdentityRole::Organization)?;
        if !matches!(
            &organization_kind,
            StableBslValueV1::Enum { enum_type, .. } if enum_type == "OrgKind"
        ) {
            return Err(MaterialStateErrorV1::OrganizationKind);
        }
        for territory in &ordered_territory_ids {
            require_node_key(territory, MaterialIdentityRole::Territory)?;
        }
        validate_name_order(
            "organization",
            ordered_fields.iter().map(|(name, _)| name.as_str()),
        )?;
        gate.before_reserve("material organization key", 1)?;
        let primary_key = organization_id
            .canonical_bytes()
            .map_err(map_organization_identity)?;
        gate.before_reserve("material organization row", 1)?;
        let canonical_bytes = encode_organization(
            &organization_id,
            &organization_kind,
            &ordered_territory_ids,
            &ordered_fields,
            gate,
        )?;
        Ok(Self {
            organization_id,
            organization_kind,
            ordered_territory_ids,
            ordered_fields,
            primary_key,
            canonical_bytes,
        })
    }

    /// Borrow the exact stable organization identity.
    #[must_use]
    pub const fn organization_id(&self) -> &StableElementKeyV1 {
        &self.organization_id
    }

    /// Borrow the exact declared `OrgKind` stable value.
    #[must_use]
    pub const fn organization_kind(&self) -> &StableBslValueV1 {
        &self.organization_kind
    }

    /// Borrow outgoing PRESENCE territories in framed canonical-key order.
    #[must_use]
    pub fn ordered_territory_ids(&self) -> &[StableElementKeyV1] {
        &self.ordered_territory_ids
    }

    /// Borrow the strict UTF-8 field-name ordered stable values.
    #[must_use]
    pub fn ordered_fields(&self) -> &[(String, StableBslValueV1)] {
        &self.ordered_fields
    }

    /// Borrow exact canonical row bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

/// One borrowed closed material-state row in contract order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialStateRowRefV1<'a> {
    WorldRegister(&'a WorldRegisterRowV1),
    Territory(&'a TerritoryStateRowV1),
    DynamicHex(&'a DynamicHexStateRowV1),
    Organization(&'a OrganizationStateRowV1),
}

impl MaterialStateRowRefV1<'_> {
    /// Borrow exact canonical row bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        match self {
            Self::WorldRegister(row) => row.canonical_bytes(),
            Self::Territory(row) => row.canonical_bytes(),
            Self::DynamicHex(row) => row.canonical_bytes(),
            Self::Organization(row) => row.canonical_bytes(),
        }
    }
}

/// The session-owned exact dynamic-H3 runtime.
#[derive(Debug, PartialEq, Eq)]
pub struct MaterialStateV1 {
    dynamic_hexes: DynamicHexRuntimeV1,
}

impl MaterialStateV1 {
    /// Construct the sole checked dynamic-H3 runtime.
    ///
    /// # Errors
    /// Returns the first dynamic-runtime allocation or ordering refusal.
    pub fn try_new(
        foundation: &MichiganDynamicHexFoundationV1,
    ) -> Result<Self, MaterialStateErrorV1> {
        let dynamic_hexes = DynamicHexRuntimeV1::try_from_foundation(
            foundation,
            &ProductionMaterialAllocationGate,
        )?;
        Self::try_from_runtime(dynamic_hexes)
    }

    fn try_from_runtime(dynamic_hexes: DynamicHexRuntimeV1) -> Result<Self, MaterialStateErrorV1> {
        if dynamic_hexes
            .rows()
            .windows(2)
            .any(|rows| rows[0].cell_id.as_u64() >= rows[1].cell_id.as_u64())
        {
            return Err(MaterialStateErrorV1::SourceRowOrder {
                family: "dynamic hex",
            });
        }
        Ok(Self { dynamic_hexes })
    }

    pub(crate) const fn reference_bundle_digest(&self) -> [u8; 32] {
        self.dynamic_hexes.reference_bundle_digest()
    }

    #[cfg(test)]
    pub(crate) fn try_dynamic_runtime_fixture_for_test(
        rows: Vec<(H3CellId, MichiganDynamicHexValueBitsV1)>,
    ) -> Result<Self, MaterialStateErrorV1> {
        Self::try_from_runtime(DynamicHexRuntimeV1::try_fixture(rows)?)
    }

    pub(crate) fn try_detached(
        &self,
        gate: &dyn MaterialAllocationGate,
    ) -> Result<Self, MaterialStateErrorV1> {
        Ok(Self {
            dynamic_hexes: self.dynamic_hexes.try_detached(gate)?,
        })
    }

    pub(crate) fn try_restore_from_rows(
        &self,
        rows: &MaterialStateRowsV1,
    ) -> Result<Self, MaterialStateErrorV1> {
        Self::try_from_runtime(self.dynamic_hexes.try_restore_from_rows(
            rows.dynamic_hexes().rows(),
            &ProductionMaterialAllocationGate,
        )?)
    }

    pub(crate) fn project_rows(
        &self,
        resolve_tick: i64,
        context: &MaterialProjectionContextV1<'_>,
    ) -> Result<MaterialStateRowsV1, MaterialStateErrorV1> {
        MaterialStateRowsV1::compose(self, resolve_tick, context)
    }
}

fn project_dynamic_rows(
    source: &DynamicHexRuntimeV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<DynamicHexStateRowV1>, MaterialStateErrorV1> {
    let mut rows = reserve_vec("material dynamic rows", source.rows().len(), gate)?;
    for row in source.rows() {
        rows.push(DynamicHexStateRowV1::try_from_runtime(row, gate)?);
    }
    Ok(rows)
}

macro_rules! material_batch {
    ($name:ident, $row:ty, $tag:expr) => {
        #[doc = "One source-owned typed material-family batch."]
        #[derive(Debug, PartialEq, Eq)]
        pub struct $name {
            rows: Vec<$row>,
            canonical_bytes: Vec<u8>,
            source_digest: [u8; 32],
        }

        impl $name {
            fn compose(
                rows: Vec<$row>,
                gate: &dyn MaterialAllocationGate,
            ) -> Result<Self, MaterialStateErrorV1> {
                let canonical_bytes = encode_source_batch(
                    $tag,
                    rows.iter().map(|row| row.canonical_bytes()),
                    rows.len(),
                    gate,
                )?;
                let source_digest = sha256_of(&canonical_bytes);
                Ok(Self {
                    rows,
                    canonical_bytes,
                    source_digest,
                })
            }

            /// Return the exact source row count.
            #[must_use]
            pub const fn source_count(&self) -> usize {
                self.rows.len()
            }

            /// Borrow the exact typed source rows.
            #[must_use]
            pub fn rows(&self) -> &[$row] {
                &self.rows
            }

            /// Borrow exact canonical source bytes, including true emptiness.
            #[must_use]
            pub fn canonical_bytes(&self) -> &[u8] {
                &self.canonical_bytes
            }

            /// Return SHA-256 of the exact canonical source bytes.
            #[must_use]
            pub const fn source_digest(&self) -> [u8; 32] {
                self.source_digest
            }
        }
    };
}

material_batch!(WorldRegisterRowsV1, WorldRegisterRowV1, 0x01);
material_batch!(TerritoryStateRowsV1, TerritoryStateRowV1, 0x02);
material_batch!(DynamicHexStateRowsV1, DynamicHexStateRowV1, 0x03);
material_batch!(OrganizationStateRowsV1, OrganizationStateRowV1, 0x08);

/// One independently owned material projection from a completed replay tick.
#[derive(Debug, PartialEq, Eq)]
pub struct MaterialStateRowsV1 {
    world_registers: WorldRegisterRowsV1,
    territories: TerritoryStateRowsV1,
    dynamic_hexes: DynamicHexStateRowsV1,
    organizations: OrganizationStateRowsV1,
    canonical_bytes: Vec<u8>,
    source_digest: [u8; 32],
    source_count: usize,
}

/// Owned typed material rows loaded from an external durable store.
#[derive(Debug)]
pub struct MaterialStateRowsInputV1 {
    /// Exact world-register rows.
    pub world_registers: Vec<WorldRegisterRowV1>,
    /// Exact territory rows.
    pub territories: Vec<TerritoryStateRowV1>,
    /// Exact dynamic-H3 rows.
    pub dynamic_hexes: Vec<DynamicHexStateRowV1>,
    /// Exact organization rows.
    pub organizations: Vec<OrganizationStateRowV1>,
}

impl MaterialStateRowsV1 {
    /// Compose the canonical material checkpoint section from typed stored rows.
    ///
    /// # Errors
    /// Returns the first row-order, bound, arithmetic, or allocation refusal.
    pub fn try_from_rows(input: MaterialStateRowsInputV1) -> Result<Self, MaterialStateErrorV1> {
        validate_source_order(&input.world_registers, "world register", |row| {
            row.qname().as_bytes()
        })?;
        validate_source_order(&input.territories, "territory", |row| &row.primary_key)?;
        if input
            .dynamic_hexes
            .windows(2)
            .any(|pair| pair[0].cell_id().as_u64() >= pair[1].cell_id().as_u64())
        {
            return Err(MaterialStateErrorV1::SourceRowOrder {
                family: "dynamic hex",
            });
        }
        validate_source_order(&input.organizations, "organization", |row| &row.primary_key)?;
        let gate = &ProductionMaterialAllocationGate;
        let world_registers = WorldRegisterRowsV1::compose(input.world_registers, gate)?;
        let territories = TerritoryStateRowsV1::compose(input.territories, gate)?;
        let dynamic_hexes = DynamicHexStateRowsV1::compose(input.dynamic_hexes, gate)?;
        let organizations = OrganizationStateRowsV1::compose(input.organizations, gate)?;
        let source_count = checked_sum(
            "material row count",
            [
                world_registers.source_count(),
                territories.source_count(),
                dynamic_hexes.source_count(),
                organizations.source_count(),
            ],
        )?;
        let canonical_bytes = encode_material_batches(
            &world_registers,
            &territories,
            &dynamic_hexes,
            &organizations,
            gate,
        )?;
        let source_digest = sha256_of(&canonical_bytes);
        Ok(Self {
            world_registers,
            territories,
            dynamic_hexes,
            organizations,
            canonical_bytes,
            source_digest,
            source_count,
        })
    }

    fn compose(
        source: &MaterialStateV1,
        resolve_tick: i64,
        context: &MaterialProjectionContextV1<'_>,
    ) -> Result<Self, MaterialStateErrorV1> {
        let gate = context.gate;
        let mut world_rows = reserve_vec("material world register rows", 1, gate)?;
        world_rows.push(WorldRegisterRowV1::try_new_with_gate(
            copy_string(
                "material world register qname",
                "world/completed-tick",
                gate,
            )?,
            StableBslValueV1::Int(resolve_tick),
            gate,
        )?);
        let world_registers = WorldRegisterRowsV1::compose(world_rows, gate)?;
        let territory_rows = derive_territory_rows(
            context.stable_graph,
            context.scenario_scope,
            context.types,
            context.enums,
            context.resolver,
            gate,
        )?;
        let territories = TerritoryStateRowsV1::compose(territory_rows, gate)?;
        let dynamic_hexes = DynamicHexStateRowsV1::compose(
            project_dynamic_rows(&source.dynamic_hexes, gate)?,
            gate,
        )?;
        let organization_rows = derive_organization_rows(
            context.stable_graph,
            context.scenario_scope,
            context.types,
            context.enums,
            context.resolver,
            gate,
        )?;
        let organizations = OrganizationStateRowsV1::compose(organization_rows, gate)?;
        let source_count = checked_sum(
            "material row count",
            [
                world_registers.source_count(),
                territories.source_count(),
                dynamic_hexes.source_count(),
                organizations.source_count(),
            ],
        )?;
        let canonical_bytes = encode_material_batches(
            &world_registers,
            &territories,
            &dynamic_hexes,
            &organizations,
            gate,
        )?;
        let source_digest = sha256_of(&canonical_bytes);

        Ok(Self {
            world_registers,
            territories,
            dynamic_hexes,
            organizations,
            canonical_bytes,
            source_digest,
            source_count,
        })
    }

    /// Iterate all rows in exact family-tag/key order without allocation.
    #[must_use]
    pub fn rows(&self) -> impl ExactSizeIterator<Item = MaterialStateRowRefV1<'_>> + '_ {
        MaterialStateRowsIterV1 {
            rows: self,
            family: 0,
            index: 0,
            remaining: self.source_count,
        }
    }
    /// Return the aggregate row count, including the derived world register.
    #[must_use]
    pub const fn source_count(&self) -> usize {
        self.source_count
    }
    /// Borrow exact aggregate canonical bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    /// Return SHA-256 of the aggregate canonical bytes.
    #[must_use]
    pub const fn source_digest(&self) -> [u8; 32] {
        self.source_digest
    }
    /// Borrow the derived world-register batch.
    #[must_use]
    pub const fn world_registers(&self) -> &WorldRegisterRowsV1 {
        &self.world_registers
    }
    /// Borrow the territory batch.
    #[must_use]
    pub const fn territories(&self) -> &TerritoryStateRowsV1 {
        &self.territories
    }
    /// Borrow the dynamic-H3 batch.
    #[must_use]
    pub const fn dynamic_hexes(&self) -> &DynamicHexStateRowsV1 {
        &self.dynamic_hexes
    }
    /// Borrow the organization batch.
    #[must_use]
    pub const fn organizations(&self) -> &OrganizationStateRowsV1 {
        &self.organizations
    }
}

struct MaterialStateRowsIterV1<'a> {
    rows: &'a MaterialStateRowsV1,
    family: u8,
    index: usize,
    remaining: usize,
}

impl<'a> Iterator for MaterialStateRowsIterV1<'a> {
    type Item = MaterialStateRowRefV1<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = match self.family {
                0 => self
                    .rows
                    .world_registers
                    .rows()
                    .get(self.index)
                    .map(MaterialStateRowRefV1::WorldRegister),
                1 => self
                    .rows
                    .territories
                    .rows()
                    .get(self.index)
                    .map(MaterialStateRowRefV1::Territory),
                2 => self
                    .rows
                    .dynamic_hexes
                    .rows()
                    .get(self.index)
                    .map(MaterialStateRowRefV1::DynamicHex),
                3 => self
                    .rows
                    .organizations
                    .rows()
                    .get(self.index)
                    .map(MaterialStateRowRefV1::Organization),
                _ => return None,
            };
            if let Some(row) = next {
                self.index += 1;
                self.remaining -= 1;
                return Some(row);
            }
            self.family += 1;
            self.index = 0;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for MaterialStateRowsIterV1<'_> {
    fn len(&self) -> usize {
        self.remaining
    }
}
impl std::iter::FusedIterator for MaterialStateRowsIterV1<'_> {}

fn derive_territory_rows(
    stable_graph: &StableGraphStateV1,
    scenario_scope: &str,
    types: &TypeEnv,
    enums: &EnumRegistry,
    resolver: &StableElementResolverV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<TerritoryStateRowV1>, MaterialStateErrorV1> {
    let stable_rows = stable_graph.rows();
    let territory_count = stable_rows
        .nodes()
        .iter()
        .filter(|(_, node_type)| node_type == "TERRITORY")
        .count();
    let mut output = reserve_vec("material territory rows", territory_count, gate)?;
    let mut f64_index = 0_usize;
    let mut currency_index = 0_usize;
    for (local_name, node_type) in stable_rows.nodes() {
        if node_type != "TERRITORY" {
            continue;
        }
        let key = StableElementKeyV1::Node {
            scenario: copy_string("material territory key", scenario_scope, gate)?,
            local_name: copy_string("material territory key", local_name, gate)?,
        };
        resolver
            .validate_sealed_key(&key)
            .map_err(map_territory_identity)?;
        validate_territory(resolver, &key)?;

        let f64_fields = take_f64_fields(stable_rows.node_f64(), &mut f64_index, local_name);
        let currency_fields =
            take_currency_fields(stable_rows.node_currency(), &mut currency_index, local_name);
        let field_count = checked_add(
            "material territory field count",
            f64_fields.len(),
            currency_fields.len(),
        )?;
        let mut fields = reserve_vec("material territory fields", field_count, gate)?;
        let (mut binary_index, mut money_index) = (0_usize, 0_usize);
        while binary_index < f64_fields.len() || money_index < currency_fields.len() {
            let binary = f64_fields.get(binary_index);
            let money = currency_fields.get(money_index);
            let (qname, binary64_bits, currency_micro_units) = match (binary, money) {
                (Some((_, binary_qname, bits)), Some((_, money_qname, micro_units))) => {
                    match binary_qname.as_bytes().cmp(money_qname.as_bytes()) {
                        std::cmp::Ordering::Less => {
                            binary_index += 1;
                            (binary_qname.as_str(), Some(*bits), None)
                        }
                        std::cmp::Ordering::Greater => {
                            money_index += 1;
                            (money_qname.as_str(), None, Some(*micro_units))
                        }
                        std::cmp::Ordering::Equal => {
                            binary_index += 1;
                            money_index += 1;
                            (binary_qname.as_str(), Some(*bits), Some(*micro_units))
                        }
                    }
                }
                (Some((_, qname, bits)), None) => {
                    binary_index += 1;
                    (qname.as_str(), Some(*bits), None)
                }
                (None, Some((_, qname, micro_units))) => {
                    money_index += 1;
                    (qname.as_str(), None, Some(*micro_units))
                }
                (None, None) => break,
            };
            let suffix = qname
                .strip_prefix("territory/")
                .filter(|suffix| !suffix.is_empty())
                .ok_or(MaterialStateErrorV1::TerritoryFieldOwner)?;
            let declaration = types
                .fields
                .get(qname)
                .ok_or(MaterialStateErrorV1::TerritoryFieldUndeclared)?;
            let name = copy_string("material territory field name", suffix, gate)?;
            gate.before_reserve("material territory field value", 1)?;
            let value = project_stored_field_value_v1(
                declaration,
                binary64_bits,
                currency_micro_units,
                enums,
            )
            .map_err(map_stable_value)?;
            fields.push((name, value));
        }
        output.push(TerritoryStateRowV1::try_from_projection(key, fields, gate)?);
    }
    output.sort_unstable_by(|left, right| left.primary_key.cmp(&right.primary_key));
    validate_source_order(&output, "territory", |row| &row.primary_key)?;
    Ok(output)
}

#[allow(
    clippy::too_many_lines,
    reason = "one ordered merge must advance the field and presence cursors together"
)]
fn derive_organization_rows(
    stable_graph: &StableGraphStateV1,
    scenario_scope: &str,
    types: &TypeEnv,
    enums: &EnumRegistry,
    resolver: &StableElementResolverV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<OrganizationStateRowV1>, MaterialStateErrorV1> {
    let stable_rows = stable_graph.rows();
    let organizations = stable_rows
        .nodes()
        .iter()
        .filter(|(_, node_type)| node_type == "ORGANIZATION")
        .count();
    let presence_edges = validate_presence_topology(stable_graph, gate)?;
    let mut output = reserve_vec("material organization rows", organizations, gate)?;
    let mut f64_index = 0_usize;
    let mut currency_index = 0_usize;
    let mut presence_index = 0_usize;
    for (local_name, node_type) in stable_rows.nodes() {
        if node_type != "ORGANIZATION" {
            continue;
        }
        let key = StableElementKeyV1::Node {
            scenario: copy_string("material organization identity", scenario_scope, gate)?,
            local_name: copy_string("material organization identity", local_name, gate)?,
        };
        resolver
            .validate_sealed_key(&key)
            .map_err(map_organization_identity)?;
        validate_organization(resolver, &key)?;

        let f64_fields = take_f64_fields(stable_rows.node_f64(), &mut f64_index, local_name);
        let currency_fields =
            take_currency_fields(stable_rows.node_currency(), &mut currency_index, local_name);
        let field_count = checked_add(
            "material organization field count",
            f64_fields.len(),
            currency_fields.len(),
        )?;
        let mut fields = reserve_vec("material organization fields", field_count, gate)?;
        let mut organization_kind = None;
        let (mut binary_index, mut money_index) = (0_usize, 0_usize);
        while binary_index < f64_fields.len() || money_index < currency_fields.len() {
            let binary = f64_fields.get(binary_index);
            let money = currency_fields.get(money_index);
            let (qname, binary64_bits, currency_micro_units) = match (binary, money) {
                (Some((_, binary_qname, bits)), Some((_, money_qname, micro_units))) => {
                    match binary_qname.as_bytes().cmp(money_qname.as_bytes()) {
                        std::cmp::Ordering::Less => {
                            binary_index += 1;
                            (binary_qname.as_str(), Some(*bits), None)
                        }
                        std::cmp::Ordering::Greater => {
                            money_index += 1;
                            (money_qname.as_str(), None, Some(*micro_units))
                        }
                        std::cmp::Ordering::Equal => {
                            binary_index += 1;
                            money_index += 1;
                            (binary_qname.as_str(), Some(*bits), Some(*micro_units))
                        }
                    }
                }
                (Some((_, qname, bits)), None) => {
                    binary_index += 1;
                    (qname.as_str(), Some(*bits), None)
                }
                (None, Some((_, qname, micro_units))) => {
                    money_index += 1;
                    (qname.as_str(), None, Some(*micro_units))
                }
                (None, None) => break,
            };
            let suffix = qname
                .strip_prefix("organization/")
                .filter(|suffix| !suffix.is_empty())
                .ok_or(MaterialStateErrorV1::OrganizationFieldOwner)?;
            let declaration = types
                .fields
                .get(qname)
                .ok_or(MaterialStateErrorV1::OrganizationFieldUndeclared)?;
            gate.before_reserve("material organization field value", 1)?;
            let value = project_stored_field_value_v1(
                declaration,
                binary64_bits,
                currency_micro_units,
                enums,
            )
            .map_err(map_stable_value)?;
            if suffix == "kind" {
                if organization_kind.replace(value).is_some() {
                    return Err(MaterialStateErrorV1::OrganizationKind);
                }
            } else {
                fields.push((
                    copy_string("material organization field name", suffix, gate)?,
                    value,
                ));
            }
        }
        let organization_kind = organization_kind.ok_or(MaterialStateErrorV1::OrganizationKind)?;
        if !matches!(
            &organization_kind,
            StableBslValueV1::Enum { enum_type, .. } if enum_type == "OrgKind"
        ) {
            return Err(MaterialStateErrorV1::OrganizationKind);
        }

        while presence_edges
            .get(presence_index)
            .is_some_and(|(_, source, _, _)| source.as_bytes() < local_name.as_bytes())
        {
            presence_index += 1;
        }
        let presence_start = presence_index;
        while presence_edges
            .get(presence_index)
            .is_some_and(|(_, source, _, _)| source == local_name)
        {
            presence_index += 1;
        }
        let organization_presence = &presence_edges[presence_start..presence_index];
        let mut territory_keys = reserve_vec(
            "material organization territory keys",
            organization_presence.len(),
            gate,
        )?;
        for (_, _, target, _) in organization_presence {
            let territory = StableElementKeyV1::Node {
                scenario: copy_string(
                    "material organization territory identity",
                    scenario_scope,
                    gate,
                )?,
                local_name: copy_string("material organization territory identity", target, gate)?,
            };
            resolver
                .validate_sealed_key(&territory)
                .map_err(map_territory_identity)?;
            validate_territory(resolver, &territory)?;
            gate.before_reserve("material organization territory key", 1)?;
            let primary_key = territory
                .canonical_bytes()
                .map_err(map_territory_identity)?;
            territory_keys.push((primary_key, territory));
        }
        territory_keys.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        for pair in territory_keys.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err(MaterialStateErrorV1::SourceRowOrder {
                    family: "organization territories",
                });
            }
        }
        let mut territory_ids = reserve_vec(
            "material organization territory ids",
            territory_keys.len(),
            gate,
        )?;
        for (_, territory) in territory_keys {
            territory_ids.push(territory);
        }
        output.push(OrganizationStateRowV1::try_from_projection(
            key,
            organization_kind,
            territory_ids,
            fields,
            gate,
        )?);
    }
    output.sort_unstable_by(|left, right| left.primary_key.cmp(&right.primary_key));
    validate_source_order(&output, "organization", |row| &row.primary_key)?;
    Ok(output)
}

fn validate_presence_topology<'a>(
    stable_graph: &'a StableGraphStateV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<&'a StableGraphEdgeRowV1>, MaterialStateErrorV1> {
    let stable_rows = stable_graph.rows();
    let presence_count = stable_rows
        .edges()
        .iter()
        .filter(|(edge_type, _, _, _)| edge_type == "PRESENCE")
        .count();
    let mut presence_edges = reserve_vec(
        "material organization presence topology",
        presence_count,
        gate,
    )?;
    for edge @ (edge_type, source, target, _) in stable_rows.edges() {
        if edge_type != "PRESENCE" {
            continue;
        }
        let source_type = stable_node_type(stable_rows.nodes(), source)
            .ok_or(MaterialStateErrorV1::OrganizationTerritoryPresence)?;
        let target_type = stable_node_type(stable_rows.nodes(), target)
            .ok_or(MaterialStateErrorV1::OrganizationTerritoryPresence)?;
        if source_type != "ORGANIZATION" || target_type != "TERRITORY" {
            return Err(MaterialStateErrorV1::OrganizationTerritoryPresence);
        }
        presence_edges.push(edge);
    }
    Ok(presence_edges)
}

fn stable_node_type<'a>(nodes: &'a [(String, String)], local_name: &str) -> Option<&'a str> {
    nodes
        .binary_search_by(|(candidate, _)| candidate.as_bytes().cmp(local_name.as_bytes()))
        .ok()
        .and_then(|index| nodes.get(index))
        .map(|(_, node_type)| node_type.as_str())
}

fn take_f64_fields<'a>(
    rows: &'a [(String, String, u64)],
    index: &mut usize,
    local_name: &str,
) -> &'a [(String, String, u64)] {
    while rows
        .get(*index)
        .is_some_and(|(row_local, _, _)| row_local.as_bytes() < local_name.as_bytes())
    {
        *index += 1;
    }
    let start = *index;
    while rows
        .get(*index)
        .is_some_and(|(row_local, _, _)| row_local == local_name)
    {
        *index += 1;
    }
    &rows[start..*index]
}

fn take_currency_fields<'a>(
    rows: &'a [(String, String, i128)],
    index: &mut usize,
    local_name: &str,
) -> &'a [(String, String, i128)] {
    while rows
        .get(*index)
        .is_some_and(|(row_local, _, _)| row_local.as_bytes() < local_name.as_bytes())
    {
        *index += 1;
    }
    let start = *index;
    while rows
        .get(*index)
        .is_some_and(|(row_local, _, _)| row_local == local_name)
    {
        *index += 1;
    }
    &rows[start..*index]
}

fn begin_row<'a>(
    tag: u8,
    key: &[u8],
    gate: &'a dyn MaterialAllocationGate,
) -> Result<MaterialWriter<'a>, MaterialStateErrorV1> {
    let mut writer = MaterialWriter::new("material row", gate);
    writer.extend(MATERIAL_ROW_DOMAIN)?;
    writer.extend(&MATERIAL_LAYOUT_VERSION_V1.to_be_bytes())?;
    writer.push(tag)?;
    append_bytes32(&mut writer, key)?;
    Ok(writer)
}

fn append_bytes32(
    writer: &mut MaterialWriter<'_>,
    value: &[u8],
) -> Result<(), MaterialStateErrorV1> {
    writer.extend(&checked_u32(writer.field, value.len())?.to_be_bytes())?;
    writer.extend(value)
}

fn append_stable_value(
    writer: &mut MaterialWriter<'_>,
    value: &StableBslValueV1,
) -> Result<(), MaterialStateErrorV1> {
    let mut bytes = Vec::new();
    encode_stable_bsl_value_v1(value, &mut bytes).map_err(map_stable_value)?;
    append_bytes32(writer, &bytes)
}

fn append_stable_key(
    writer: &mut MaterialWriter<'_>,
    key: &StableElementKeyV1,
    role: MaterialIdentityRole,
) -> Result<(), MaterialStateErrorV1> {
    let bytes = key.canonical_bytes().map_err(|error| match role {
        MaterialIdentityRole::Territory => map_territory_identity(error),
        MaterialIdentityRole::Organization => map_organization_identity(error),
    })?;
    append_bytes32(writer, &bytes)
}

fn append_named_stable_values(
    writer: &mut MaterialWriter<'_>,
    values: &[(String, StableBslValueV1)],
) -> Result<(), MaterialStateErrorV1> {
    writer.extend(&checked_u32(writer.field, values.len())?.to_be_bytes())?;
    for (name, value) in values {
        writer.str32(name)?;
        append_stable_value(writer, value)?;
    }
    Ok(())
}

fn encode_world_register(
    qname: &str,
    value: &StableBslValueV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let mut writer = begin_row(0x01, qname.as_bytes(), gate)?;
    writer.str32("qname")?;
    writer.str32(qname)?;
    writer.str32("value")?;
    append_stable_value(&mut writer, value)?;
    Ok(writer.finish())
}

fn encode_territory(
    territory_id: &StableElementKeyV1,
    ordered_fields: &[(String, StableBslValueV1)],
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let key = territory_id
        .canonical_bytes()
        .map_err(map_territory_identity)?;
    let mut writer = begin_row(0x02, &key, gate)?;
    writer.str32("territory_id")?;
    append_stable_key(&mut writer, territory_id, MaterialIdentityRole::Territory)?;
    writer.str32("ordered_fields")?;
    append_named_stable_values(&mut writer, ordered_fields)?;
    Ok(writer.finish())
}

fn encode_dynamic_hex(
    cell_id: H3CellId,
    values: [u64; 9],
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let mut writer = begin_row(0x03, &cell_id.to_be_bytes(), gate)?;
    writer.str32("cell_id")?;
    writer.extend(&cell_id.to_be_bytes())?;
    for (name, bits) in [
        "c",
        "v",
        "s",
        "k",
        "biocapacity_stock",
        "energy_stock",
        "raw_material_stock",
        "internet_access_pct",
        "surveillance_coupling",
    ]
    .into_iter()
    .zip(values)
    {
        writer.str32(name)?;
        writer.extend(&bits.to_be_bytes())?;
    }
    Ok(writer.finish())
}

fn encode_organization(
    organization_id: &StableElementKeyV1,
    organization_kind: &StableBslValueV1,
    ordered_territory_ids: &[StableElementKeyV1],
    ordered_fields: &[(String, StableBslValueV1)],
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let key = organization_id
        .canonical_bytes()
        .map_err(map_organization_identity)?;
    let mut writer = begin_row(0x08, &key, gate)?;
    writer.str32("organization_id")?;
    append_stable_key(
        &mut writer,
        organization_id,
        MaterialIdentityRole::Organization,
    )?;
    writer.str32("organization_kind")?;
    append_stable_value(&mut writer, organization_kind)?;
    writer.str32("ordered_territory_ids")?;
    writer.extend(&checked_u32(writer.field, ordered_territory_ids.len())?.to_be_bytes())?;
    for territory in ordered_territory_ids {
        append_stable_key(&mut writer, territory, MaterialIdentityRole::Territory)?;
    }
    writer.str32("ordered_fields")?;
    append_named_stable_values(&mut writer, ordered_fields)?;
    Ok(writer.finish())
}

fn encode_source_batch<'a>(
    family_tag: u8,
    rows: impl Iterator<Item = &'a [u8]>,
    row_count: usize,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let mut writer = MaterialWriter::new("material source batch", gate);
    writer.extend(MATERIAL_SOURCE_DOMAIN)?;
    writer.extend(&MATERIAL_LAYOUT_VERSION_V1.to_be_bytes())?;
    writer.push(family_tag)?;
    writer.extend(&checked_u32(writer.field, row_count)?.to_be_bytes())?;
    for row in rows {
        append_bytes32(&mut writer, row)?;
    }
    Ok(writer.finish())
}

fn encode_material_batches(
    world_registers: &WorldRegisterRowsV1,
    territories: &TerritoryStateRowsV1,
    dynamic_hexes: &DynamicHexStateRowsV1,
    organizations: &OrganizationStateRowsV1,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<u8>, MaterialStateErrorV1> {
    let total = checked_sum(
        "material row count",
        [
            world_registers.source_count(),
            territories.source_count(),
            dynamic_hexes.source_count(),
            organizations.source_count(),
        ],
    )?;
    let mut writer = MaterialWriter::new("material state rows", gate);
    writer.extend(MATERIAL_ROWS_DOMAIN)?;
    writer.extend(&MATERIAL_LAYOUT_VERSION_V1.to_be_bytes())?;
    writer.extend(&checked_u32(writer.field, total)?.to_be_bytes())?;
    for row in world_registers
        .rows()
        .iter()
        .map(WorldRegisterRowV1::canonical_bytes)
        .chain(
            territories
                .rows()
                .iter()
                .map(TerritoryStateRowV1::canonical_bytes),
        )
        .chain(
            dynamic_hexes
                .rows()
                .iter()
                .map(DynamicHexStateRowV1::canonical_bytes),
        )
        .chain(
            organizations
                .rows()
                .iter()
                .map(OrganizationStateRowV1::canonical_bytes),
        )
    {
        append_bytes32(&mut writer, row)?;
    }
    Ok(writer.finish())
}

#[derive(Clone, Copy)]
enum MaterialIdentityRole {
    Territory,
    Organization,
}

fn require_node_key(
    key: &StableElementKeyV1,
    role: MaterialIdentityRole,
) -> Result<(), MaterialStateErrorV1> {
    if !matches!(key, StableElementKeyV1::Node { .. }) {
        return Err(match role {
            MaterialIdentityRole::Territory => {
                MaterialStateErrorV1::TerritoryIdentity(StableIdentityError::ElementNotSealed)
            }
            MaterialIdentityRole::Organization => {
                MaterialStateErrorV1::OrganizationIdentity(StableIdentityError::ElementNotSealed)
            }
        });
    }
    key.canonical_bytes().map(drop).map_err(|error| match role {
        MaterialIdentityRole::Territory => map_territory_identity(error),
        MaterialIdentityRole::Organization => map_organization_identity(error),
    })
}

fn validate_territory(
    resolver: &StableElementResolverV1,
    key: &StableElementKeyV1,
) -> Result<(), MaterialStateErrorV1> {
    match resolver.sealed_node_has_type(key, "TERRITORY") {
        Ok(true) => Ok(()),
        Ok(false) => Err(MaterialStateErrorV1::TerritoryNodeType),
        Err(error) => Err(map_territory_identity(error)),
    }
}

fn validate_organization(
    resolver: &StableElementResolverV1,
    key: &StableElementKeyV1,
) -> Result<(), MaterialStateErrorV1> {
    match resolver.sealed_node_has_type(key, "ORGANIZATION") {
        Ok(true) => Ok(()),
        Ok(false) => Err(MaterialStateErrorV1::OrganizationNodeType),
        Err(error) => Err(map_organization_identity(error)),
    }
}

fn validate_name_order<'a>(
    family: &'static str,
    names: impl Iterator<Item = &'a str>,
) -> Result<(), MaterialStateErrorV1> {
    let mut prior: Option<&str> = None;
    for name in names {
        validate_nonempty_ascii("material field name", name)?;
        if prior.is_some_and(|prior| prior.as_bytes() >= name.as_bytes()) {
            return Err(MaterialStateErrorV1::NamedFieldOrder { family });
        }
        prior = Some(name);
    }
    Ok(())
}

fn validate_source_order<T>(
    rows: &[T],
    family: &'static str,
    bytes: impl Fn(&T) -> &[u8],
) -> Result<(), MaterialStateErrorV1> {
    if rows
        .windows(2)
        .any(|pair| bytes(&pair[0]) >= bytes(&pair[1]))
    {
        return Err(MaterialStateErrorV1::SourceRowOrder { family });
    }
    Ok(())
}

fn validate_nonempty_ascii(field: &'static str, value: &str) -> Result<(), MaterialStateErrorV1> {
    if value.is_empty() || value.len() > 128 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(MaterialStateErrorV1::StableValue(
            IdentityCodecError::InvalidString {
                field,
                index: value.len(),
            },
        ));
    }
    Ok(())
}

fn checked_add(
    field: &'static str,
    left: usize,
    right: usize,
) -> Result<usize, MaterialStateErrorV1> {
    left.checked_add(right)
        .ok_or(MaterialStateErrorV1::CapacityOverflow { field })
}

fn checked_sum<const N: usize>(
    field: &'static str,
    values: [usize; N],
) -> Result<usize, MaterialStateErrorV1> {
    values
        .into_iter()
        .try_fold(0_usize, |total, value| checked_add(field, total, value))
}

fn checked_u32(field: &'static str, value: usize) -> Result<u32, MaterialStateErrorV1> {
    u32::try_from(value).map_err(|_| MaterialStateErrorV1::IntegerConversion { field, value })
}

fn map_territory_identity(error: StableIdentityError) -> MaterialStateErrorV1 {
    match error {
        StableIdentityError::Allocation { field, requested } => {
            MaterialStateErrorV1::Allocation { field, requested }
        }
        other => MaterialStateErrorV1::TerritoryIdentity(other),
    }
}

fn map_organization_identity(error: StableIdentityError) -> MaterialStateErrorV1 {
    match error {
        StableIdentityError::Allocation { field, requested } => {
            MaterialStateErrorV1::Allocation { field, requested }
        }
        other => MaterialStateErrorV1::OrganizationIdentity(other),
    }
}

fn map_stable_value(error: IdentityCodecError) -> MaterialStateErrorV1 {
    match error {
        IdentityCodecError::Allocation { field, requested }
        | IdentityCodecError::StableIdentity(StableIdentityError::Allocation {
            field,
            requested,
        }) => MaterialStateErrorV1::Allocation { field, requested },
        other => MaterialStateErrorV1::StableValue(other),
    }
}

fn copy_string(
    field: &'static str,
    source: &str,
    gate: &dyn MaterialAllocationGate,
) -> Result<String, MaterialStateErrorV1> {
    gate.before_reserve(field, source.len())?;
    let mut output = String::new();
    output
        .try_reserve_exact(source.len())
        .map_err(|_: TryReserveError| MaterialStateErrorV1::Allocation {
            field,
            requested: source.len(),
        })?;
    output.push_str(source);
    Ok(output)
}

fn reserve_vec<T>(
    field: &'static str,
    requested: usize,
    gate: &dyn MaterialAllocationGate,
) -> Result<Vec<T>, MaterialStateErrorV1> {
    gate.before_reserve(field, requested)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(requested)
        .map_err(|_: TryReserveError| MaterialStateErrorV1::Allocation { field, requested })?;
    Ok(output)
}
