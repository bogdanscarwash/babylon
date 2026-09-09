//! The admitted G4 workforce composition's exact graph effects.
//!
//! Staffing state is a transient projection of the supplied detached graph.
//! This module neither owns durable state nor publishes a tick. A caller must
//! discard its entire candidate if an effect or a later tick stage refuses.

use std::collections::BTreeSet;

use babylon_bsl::causal_contract::{
    reduce_audit_receipts, AuditReceipt, ContractError, EvidenceClass, RuleContract, RuleRole,
};
use babylon_bsl::evaluator::{EvalCode, EvalError, Value};
use babylon_bsl::identity_codec::{
    project_stored_field_value_v1, IdentityCodecError, StableBslValueV1,
};
use babylon_bsl::structural_verbs::{
    EffectExecutor, PendingWrite, UpdateOp, WriteOperand, WriteTarget,
};
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{BslType, EnumRegistry, FieldKind};
use babylon_bsl::vocabulary::ClosedVocabulary;
use babylon_bsl::write_log::{CollectingWriteLog, WriteRecord};
use babylon_graph::stable_element::{
    StableElementKeyV1, StableElementResolverV1, StableIdentityError,
};
use babylon_graph::substrate::{GraphError, GraphSubstrate, NodeId};
use babylon_material_circuit::{
    advance_staffing_v1, LaborCapacityRowV1, StaffingErrorV1, StaffingPoolBindingV1,
    StaffingPoolStateV1, StaffingReceiptV1, StaffingStateV1, StaffingWorkRequestV1,
    MAX_MATERIAL_CIRCUIT_ROWS_V1,
};

use crate::committed_event::CommittedEventV2;

/// This identity's placement and content must be independently admitted by the caller.
pub const STAFFING_COMPOSITION_ID_V1: &str = "g4-workforce-staffing";
/// Graph fields retain exact integers only through this inclusive boundary.
pub const MAX_EXACT_STAFFING_INTEGER_V1: u64 = 1_u64 << 53;
/// Exact employed persons in the modeled site-bound pool.
pub const EMPLOYED_POPULATION: &str = "social-class/employed-population";
/// Exact reserve persons in that same closed pool.
pub const RESERVE_POPULATION: &str = "social-class/reserve-population";
/// Last period's actual unretained work request, never its retained maximum.
pub const PREVIOUS_UNRETAINED_HOURS: &str = "social-class/previous-unretained-labor-hours";
/// The composition's complete write footprint, in application order per pool.
pub const STAFFING_FIELDS_V1: [&str; 3] = [
    EMPLOYED_POPULATION,
    RESERVE_POPULATION,
    PREVIOUS_UNRETAINED_HOURS,
];

/// Closed composition, graph projection and effect refusals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialStaffingErrorV1 {
    EmptyBindings,
    NodeBinding,
    NodeOwner,
    DuplicateNode,
    ExactInteger,
    EvidenceInteger,
    FieldDeclaration(&'static str),
    FieldRead {
        field: &'static str,
        source: GraphError,
    },
    Graph(GraphError),
    Stable(StableIdentityError),
    StoredValue(IdentityCodecError),
    Core(StaffingErrorV1),
    Effect {
        code: Option<EvalCode>,
        message: String,
    },
    Audit(ContractError),
    Allocation,
}

impl std::fmt::Display for MaterialStaffingErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "material staffing refused: {self:?}")
    }
}
impl std::error::Error for MaterialStaffingErrorV1 {}
impl From<StaffingErrorV1> for MaterialStaffingErrorV1 {
    fn from(error: StaffingErrorV1) -> Self {
        Self::Core(error)
    }
}
impl From<StableIdentityError> for MaterialStaffingErrorV1 {
    fn from(error: StableIdentityError) -> Self {
        Self::Stable(error)
    }
}
impl From<EvalError> for MaterialStaffingErrorV1 {
    fn from(error: EvalError) -> Self {
        Self::Effect {
            code: error.code,
            message: error.message,
        }
    }
}

/// One authored workforce node and its distinct, closed physical labor pool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffingNodeBindingV1 {
    subject: StableElementKeyV1,
    pool: StaffingPoolBindingV1,
}

impl StaffingNodeBindingV1 {
    /// Check the node identity and exact population with the supplied typed policy.
    /// The caller admits the authored schedule as part of campaign content.
    /// # Errors
    /// Refuses another key kind, malformed key or unrepresentable population.
    pub fn try_new(
        subject: StableElementKeyV1,
        pool: StaffingPoolBindingV1,
    ) -> Result<Self, MaterialStaffingErrorV1> {
        if !matches!(subject, StableElementKeyV1::Node { .. }) {
            return Err(MaterialStaffingErrorV1::NodeBinding);
        }
        subject.canonical_bytes()?;
        exact_real(pool.labor_force())?;
        Ok(Self { subject, pool })
    }
    #[must_use]
    pub const fn subject(&self) -> &StableElementKeyV1 {
        &self.subject
    }
    #[must_use]
    pub const fn pool(&self) -> &StaffingPoolBindingV1 {
        &self.pool
    }
}

/// Immutable, complete bindings, ordered by pool identity without duplicate principals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffingCompositionV1 {
    bindings: Vec<StaffingNodeBindingV1>,
}

impl StaffingCompositionV1 {
    /// Validate the complete roster. No population or political role is inferred.
    /// # Errors
    /// Refuses empty, excessive, duplicate or overlapping ownership.
    pub fn try_new(
        mut bindings: Vec<StaffingNodeBindingV1>,
    ) -> Result<Self, MaterialStaffingErrorV1> {
        if bindings.is_empty() {
            return Err(MaterialStaffingErrorV1::EmptyBindings);
        }
        if bindings.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
            return Err(StaffingErrorV1::RowLimit.into());
        }
        bindings.sort_unstable_by_key(|row| row.pool.pool_id());
        let mut nodes = BTreeSet::new();
        let mut pools = BTreeSet::new();
        let mut sites = BTreeSet::new();
        let mut processes = BTreeSet::new();
        for row in &bindings {
            if !nodes.insert(row.subject.canonical_bytes()?) {
                return Err(MaterialStaffingErrorV1::DuplicateNode);
            }
            if !pools.insert(row.pool.pool_id()) {
                return Err(StaffingErrorV1::DuplicatePool.into());
            }
            if !sites.insert((row.pool.site_id(), row.pool.unit_id())) {
                return Err(StaffingErrorV1::DuplicateSiteUnit.into());
            }
            for process in row.pool.processes() {
                if !processes.insert(*process) {
                    return Err(StaffingErrorV1::DuplicateProcess.into());
                }
                if processes.len() > MAX_MATERIAL_CIRCUIT_ROWS_V1 {
                    return Err(StaffingErrorV1::RowLimit.into());
                }
            }
        }
        Ok(Self { bindings })
    }
    #[must_use]
    pub fn bindings(&self) -> &[StaffingNodeBindingV1] {
        &self.bindings
    }
}

/// Exact registries already owned by the prepared replay environment.
#[derive(Clone, Copy)]
pub struct StaffingEffectContextV1<'a> {
    pub types: &'a TypeEnv,
    pub enums: &'a EnumRegistry,
    pub vocabulary: Option<&'a ClosedVocabulary>,
    pub resolver: &'a StableElementResolverV1,
}

/// Completed effects and evidence. Next labor rows remain a transient physical input.
#[derive(Debug)]
pub struct StaffingEffectsV1 {
    staffing_receipts: Vec<StaffingReceiptV1>,
    next_labor: Vec<LaborCapacityRowV1>,
    writes: Vec<WriteRecord>,
    audit_receipts: Vec<AuditReceipt>,
    committed_events: Vec<CommittedEventV2>,
}
impl StaffingEffectsV1 {
    #[must_use]
    pub fn staffing_receipts(&self) -> &[StaffingReceiptV1] {
        &self.staffing_receipts
    }
    #[must_use]
    pub fn next_labor(&self) -> &[LaborCapacityRowV1] {
        &self.next_labor
    }
    #[must_use]
    pub fn writes(&self) -> &[WriteRecord] {
        &self.writes
    }
    #[must_use]
    pub fn audit_receipts(&self) -> &[AuditReceipt] {
        &self.audit_receipts
    }
    /// Derive sink records from these events; never append a second independently built batch.
    #[must_use]
    pub fn committed_events(&self) -> &[CommittedEventV2] {
        &self.committed_events
    }
}

fn reserved<T>(count: usize) -> Result<Vec<T>, MaterialStaffingErrorV1> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| MaterialStaffingErrorV1::Allocation)?;
    Ok(values)
}

fn validate_fields(context: StaffingEffectContextV1<'_>) -> Result<(), MaterialStaffingErrorV1> {
    for field in STAFFING_FIELDS_V1 {
        if !context.types.fields.get(field).is_some_and(|declaration| {
            declaration.ty == BslType::Int && declaration.kind == FieldKind::Extensive
        }) {
            return Err(MaterialStaffingErrorV1::FieldDeclaration(field));
        }
    }
    Ok(())
}

fn resolve_node(
    graph: &impl GraphSubstrate,
    context: StaffingEffectContextV1<'_>,
    row: &StaffingNodeBindingV1,
) -> Result<NodeId, MaterialStaffingErrorV1> {
    if !context
        .resolver
        .sealed_node_has_type(&row.subject, "SOCIAL_CLASS")?
    {
        return Err(MaterialStaffingErrorV1::NodeOwner);
    }
    let StableElementKeyV1::Node { local_name, .. } = &row.subject else {
        return Err(MaterialStaffingErrorV1::NodeBinding);
    };
    let node = context.resolver.node_handle_by_local_name(local_name)?;
    if context.resolver.node_key(node)? != &row.subject {
        return Err(MaterialStaffingErrorV1::NodeBinding);
    }
    if graph
        .node_type_of(node)
        .map_err(MaterialStaffingErrorV1::Graph)?
        != "SOCIAL_CLASS"
    {
        return Err(MaterialStaffingErrorV1::NodeOwner);
    }
    Ok(node)
}

fn read_stock(
    graph: &impl GraphSubstrate,
    node: NodeId,
    field: &'static str,
    context: &StaffingEffectContextV1<'_>,
) -> Result<u64, MaterialStaffingErrorV1> {
    let declaration = context
        .types
        .fields
        .get(field)
        .ok_or(MaterialStaffingErrorV1::FieldDeclaration(field))?;
    let value = graph
        .node_attribute(node, field)
        .map_err(|source| MaterialStaffingErrorV1::FieldRead { field, source })?;
    let stable =
        project_stored_field_value_v1(declaration, Some(value.to_bits()), None, context.enums)
            .map_err(MaterialStaffingErrorV1::StoredValue)?;
    let StableBslValueV1::Int(value) = stable else {
        return Err(MaterialStaffingErrorV1::FieldDeclaration(field));
    };
    u64::try_from(value).map_err(|_| MaterialStaffingErrorV1::ExactInteger)
}

fn exact_real(value: u64) -> Result<f64, MaterialStaffingErrorV1> {
    if value > MAX_EXACT_STAFFING_INTEGER_V1 {
        return Err(MaterialStaffingErrorV1::ExactInteger);
    }
    // Two exact u32 conversions avoid an unchecked integer-to-binary64 cast.
    let high = u32::try_from(value >> 32).map_err(|_| MaterialStaffingErrorV1::ExactInteger)?;
    let low = u32::try_from(value & u64::from(u32::MAX))
        .map_err(|_| MaterialStaffingErrorV1::ExactInteger)?;
    Ok(f64::from(high) * 4_294_967_296.0 + f64::from(low))
}

fn read_opening(
    graph: &impl GraphSubstrate,
    context: StaffingEffectContextV1<'_>,
    composition: &StaffingCompositionV1,
    period: u64,
) -> Result<(StaffingStateV1, Vec<NodeId>), MaterialStaffingErrorV1> {
    validate_fields(context)?;
    let mut pools = reserved(composition.bindings.len())?;
    let mut nodes = reserved(composition.bindings.len())?;
    let mut seen = BTreeSet::new();
    for row in &composition.bindings {
        let node = resolve_node(graph, context, row)?;
        if !seen.insert(node) {
            return Err(MaterialStaffingErrorV1::DuplicateNode);
        }
        pools.push(StaffingPoolStateV1::try_new(
            row.pool.clone(),
            read_stock(graph, node, EMPLOYED_POPULATION, &context)?,
            read_stock(graph, node, RESERVE_POPULATION, &context)?,
            read_stock(graph, node, PREVIOUS_UNRETAINED_HOURS, &context)?,
        )?);
        nodes.push(node);
    }
    Ok((StaffingStateV1::try_new(period, pools)?, nodes))
}

fn staffing_event(
    node: NodeId,
    receipt: &StaffingReceiptV1,
) -> Result<CommittedEventV2, MaterialStaffingErrorV1> {
    let fields = [
        ("period", receipt.period()),
        ("opening-employed", receipt.opening_employed()),
        ("opening-reserve", receipt.opening_reserve()),
        (
            "previous-unretained-hours",
            receipt.previous_unretained_hours(),
        ),
        (
            "current-unretained-hours",
            receipt.current_unretained_hours(),
        ),
        ("retained-hours", receipt.retained_hours()),
        ("target-employed", receipt.target_employed()),
        ("hires", receipt.hires()),
        ("separations", receipt.separations()),
        ("closing-employed", receipt.closing_employed()),
        ("closing-reserve", receipt.closing_reserve()),
        ("next-opening-hours", receipt.next_opening_hours()),
    ];
    let mut payload = reserved(fields.len() + 1)?;
    // This is a sealed reference, never an integer encoding of an allocation handle.
    // The normal replay event codec projects it through the same resolver.
    payload.push(("subject".to_owned(), Value::NodeRef(node)));
    for (name, value) in fields {
        payload.push((
            name.to_owned(),
            Value::Int(i64::try_from(value).map_err(|_| MaterialStaffingErrorV1::EvidenceInteger)?),
        ));
    }
    Ok(CommittedEventV2::new(
        STAFFING_COMPOSITION_ID_V1.to_owned(),
        None,
        "WORKFORCE_STAFFING".to_owned(),
        payload,
    ))
}

fn prepare_effects(
    nodes: &[NodeId],
    receipts: &[StaffingReceiptV1],
) -> Result<(Vec<PendingWrite>, Vec<CommittedEventV2>), MaterialStaffingErrorV1> {
    if nodes.len() != receipts.len() {
        return Err(MaterialStaffingErrorV1::NodeBinding);
    }
    let count = nodes
        .len()
        .checked_mul(3)
        .ok_or(MaterialStaffingErrorV1::Allocation)?;
    let mut writes = reserved(count)?;
    let mut events = reserved(nodes.len())?;
    for (node, receipt) in nodes.iter().zip(receipts) {
        for (field, value) in STAFFING_FIELDS_V1.into_iter().zip([
            receipt.closing_employed(),
            receipt.closing_reserve(),
            receipt.current_unretained_hours(),
        ]) {
            writes.push(PendingWrite {
                target: WriteTarget::Node(*node),
                field: field.to_owned(),
                op: UpdateOp::Set,
                operand: WriteOperand::Real(exact_real(value)?),
            });
        }
        events.push(staffing_event(*node, receipt)?);
    }
    Ok((writes, events))
}

/// Resolve one complete workforce transition and apply its three exact graph effects per pool.
///
/// Every source read, core transition and output conversion finishes before the first write.
/// The caller must supply a detached candidate and discard it on any returned error. This
/// function makes no publication or durability claim and returns no persistent staffing owner.
/// # Errors
/// Refuses invalid declarations, ownership, numeric values, requests, core transitions or effects.
pub fn apply_material_staffing_v1(
    graph: &mut impl GraphSubstrate,
    context: StaffingEffectContextV1<'_>,
    composition: &StaffingCompositionV1,
    period: u64,
    requests: &[StaffingWorkRequestV1],
) -> Result<StaffingEffectsV1, MaterialStaffingErrorV1> {
    let (opening, nodes) = read_opening(graph, context, composition, period)?;
    let transition = advance_staffing_v1(&opening, requests)?;
    let (pending, committed_events) = prepare_effects(&nodes, transition.receipts())?;
    let mut staffing_receipts = reserved(transition.receipts().len())?;
    staffing_receipts.extend_from_slice(transition.receipts());
    let mut next_labor = reserved(transition.next_labor().len())?;
    next_labor.extend_from_slice(transition.next_labor());
    let mut log = CollectingWriteLog::new();
    log.records
        .try_reserve_exact(pending.len())
        .map_err(|_| MaterialStaffingErrorV1::Allocation)?;
    {
        let mut executor = EffectExecutor::observed(
            context.types,
            context.enums,
            context.vocabulary,
            STAFFING_COMPOSITION_ID_V1,
            &mut log,
        );
        for write in &pending {
            executor
                .apply_pending_write(write, graph)
                .map_err(MaterialStaffingErrorV1::from)?;
        }
    }
    let event_types = committed_events
        .iter()
        .map(|event| event.event_type().to_owned())
        .collect::<Vec<_>>();
    let contract = RuleContract {
        rule_id: STAFFING_COMPOSITION_ID_V1.to_owned(),
        role: RuleRole::Mechanic,
        evidence: EvidenceClass::Designed,
    };
    let audit_receipts = reduce_audit_receipts(&contract, &event_types, &log.records)
        .map_err(MaterialStaffingErrorV1::Audit)?;
    Ok(StaffingEffectsV1 {
        staffing_receipts,
        next_labor,
        writes: log.records,
        audit_receipts,
        committed_events,
    })
}

#[cfg(test)]
mod tests;
