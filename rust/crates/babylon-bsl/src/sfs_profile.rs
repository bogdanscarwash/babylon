//! Opt-in, downstream-only footprint auditing for synthetic-emergence evidence.

use crate::bindings::{parse_bindings, BindSource};
use crate::bound_checker::{check_rule, declared_fuel, BoundError};
use crate::canonical_ast::canonical_bytes;
use crate::causal_contract::{
    effect_footprint, validate_ast_walk_bounds, EffectSignature, ShapeVerb, AST_WALK_LIMITS,
    MAX_AST_WALK_DEPTH, MAX_AST_WALK_NODES, MAX_AST_WALK_STACK,
};
use crate::fuel::{CardinalityCeilings, IntrinsicCosts, SfsFuelIdentityError};
use crate::reader::{Atom, SExpr};
use crate::vocabulary::{ClosedVocabulary, EnumKind};
use babylon_kernel::sha256_of;
use std::collections::{BTreeMap, BTreeSet};

const MAX_POLICY_ENTRIES: usize = 64;
const AUDITOR: &str = "synthetic-emergence footprint audit";
const QUERY_HEADS: [&str; 6] = [
    "nodes",
    "edges",
    "neighbors",
    "hyperedges",
    "members-of",
    "hyperedges-of",
];
const FORBIDDEN_INTRINSICS: [&str; 4] = ["exp", "log", "rng-draw", "sigmoid"];
const FORBIDDEN_OBSERVABLE_SUFFIXES: [&str; 5] = [
    "aggregate",
    "classification",
    "hinterland-class",
    "political-subjectivity",
    "wave-stage",
];

/// The digest-pinned, byte-sorted registry that the non-authorability sentinel
/// permits only at this declaration site.
pub const FORBIDDEN_AUTHORITATIVE_IDENTIFIERS_V1: [&str; 10] = [
    "SfsAggregate",
    "SfsClassification",
    "SfsHinterlandClass",
    "SfsPoliticalSubjectivity",
    "SfsWaveStage",
    "sfs/aggregate",
    "sfs/classification",
    "sfs/hinterland-class",
    "sfs/political-subjectivity",
    "sfs/wave-stage",
];

/// A forbidden source of time state in an opted-in rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ForbiddenBindingSource {
    /// Current tick.
    Tick,
    /// Current year.
    Year,
    /// Current position within a year.
    TickOfYear,
    /// Current position within a declared cycle.
    TickInCycle,
}

/// Governed purpose of one comparison or clamp site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfsComparisonContext {
    /// Reject invalid input.
    InputValidity,
    /// Decide eligibility without selecting an effect magnitude.
    EligibilityNoEffect,
    /// Refuse a non-conserving transfer.
    ConservationRefusal,
    /// Route a fixed material amount.
    MaterialRouting,
    /// Enforce a domain ceiling.
    DomainCeiling,
}

impl SfsComparisonContext {
    /// Stable lowercase wire/profile code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputValidity => "input-validity",
            Self::EligibilityNoEffect => "eligibility-no-effect",
            Self::ConservationRefusal => "conservation-refusal",
            Self::MaterialRouting => "material-routing",
            Self::DomainCeiling => "domain-ceiling",
        }
    }
}

/// One literal AST path whose comparison purpose is governed externally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedComparisonSite {
    site_digest: [u8; 32],
    source_digest: [u8; 32],
    context: SfsComparisonContext,
}

/// A complete, exact footprint of one opted-in rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsRuleFootprint {
    rule_id: String,
    source_digest: [u8; 32],
    computed_bound: u64,
    field_reads: BTreeSet<String>,
    edge_reads: BTreeSet<String>,
    constant_reads: BTreeSet<String>,
    queries: BTreeSet<String>,
    operators: BTreeSet<String>,
    intrinsics: BTreeSet<String>,
    comparison_clamp_contexts: BTreeSet<String>,
    effects: BTreeSet<String>,
}

/// A sealed footprint plus every static input that can change its fuel proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsRuleAuditResult {
    footprint: SfsRuleFootprint,
    declared_fuel: u64,
    cardinality_input_digest: [u8; 32],
    intrinsic_cost_input_digest: [u8; 32],
}

/// Exact expected profile and governed comparison sites for one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfsAuditPolicy {
    expected: SfsRuleFootprint,
    comparison_sites: Vec<GovernedComparisonSite>,
}

/// A bounded footprint-audit refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsProfileError {
    /// The shared semantic AST bound was exceeded.
    AstWalkLimit,
    /// Existing static fuel/cardinality rejection.
    Bound(BoundError),
    /// A fuel input table lacks canonical bounded identity.
    FuelIdentity(SfsFuelIdentityError),
    /// The checked static bound differs from the governed expectation.
    ComputedBoundMismatch { expected: u64, actual: u64 },
    /// Canonical AST serialization failed.
    CanonicalAst,
    /// A read field has no declared graph owner.
    UnknownFieldOwner { field: String },
    /// A forbidden time binding was read.
    ForbiddenBindingSource(ForbiddenBindingSource),
    /// A forbidden intrinsic call was present.
    ForbiddenIntrinsic { name: String },
    /// A time binding selected an absolute schedule.
    ForbiddenAbsoluteSchedule,
    /// Fixed response magnitudes were selected by sibling branches.
    ForbiddenResponseTable,
    /// Nested thresholds formed a fixed ladder.
    ForbiddenThresholdLadder,
    /// A downstream observable was read, written, or authorized.
    ForbiddenObservable { entry: String },
    /// An ungoverned read was present.
    UnexpectedRead { entry: String },
    /// An ungoverned effect was present.
    UnexpectedEffect { entry: String },
    /// A comparison/clamp site lacks an exact governed purpose.
    MissingComparisonContext { site_digest: [u8; 32] },
    /// A governed purpose names no live site.
    DeadComparisonContext { site_digest: [u8; 32] },
    /// A comparison selects among fixed effect magnitudes.
    ForbiddenComparisonUse { site_digest: [u8; 32] },
    /// One complete set differs from its policy.
    FootprintMismatch { set: &'static str },
    /// Canonical source identity differs from its policy.
    SourceDigestMismatch,
    /// A policy set repeats one row.
    DuplicatePolicyEntry { set: &'static str },
    /// A policy set exceeds its fixed entry ceiling.
    PolicyEntryLimit { set: &'static str, actual: usize },
    /// A governed form path omitted the root component.
    EmptyFormPath,
    /// A governed form path exceeds the shared depth contract.
    FormPathLimit { actual: usize },
    /// A governed form path does not resolve.
    UnknownFormPath,
    /// A governed form path resolves to neither comparison nor clamp.
    NotComparisonOrClamp,
}

impl From<BoundError> for SfsProfileError {
    fn from(value: BoundError) -> Self {
        Self::Bound(value)
    }
}

impl From<SfsFuelIdentityError> for SfsProfileError {
    fn from(value: SfsFuelIdentityError) -> Self {
        Self::FuelIdentity(value)
    }
}

impl GovernedComparisonSite {
    /// Resolve a literal bounded path and seal its source-relative identity.
    ///
    /// # Errors
    ///
    /// [`SfsProfileError`] when the AST, path, or selected form is invalid.
    pub fn from_rule_path(
        rule: &SExpr,
        path: &[u32],
        context: SfsComparisonContext,
    ) -> Result<Self, SfsProfileError> {
        preflight_ast(rule)?;
        if path.is_empty() {
            return Err(SfsProfileError::EmptyFormPath);
        }
        if path.len() > MAX_AST_WALK_DEPTH + 1 {
            return Err(SfsProfileError::FormPathLimit { actual: path.len() });
        }
        if path[0] != 0 {
            return Err(SfsProfileError::UnknownFormPath);
        }
        let target = resolve_form_path(rule, path)?;
        if !is_comparison_or_clamp(target) {
            return Err(SfsProfileError::NotComparisonOrClamp);
        }
        let source_digest = canonical_digest(rule)?;
        Ok(Self {
            site_digest: site_digest(source_digest, path)?,
            source_digest,
            context,
        })
    }

    /// Stable site identity.
    #[must_use]
    pub const fn site_digest(&self) -> &[u8; 32] {
        &self.site_digest
    }

    /// Context code and lowercase site digest joined for the footprint set.
    #[must_use]
    pub fn profile_entry(&self) -> String {
        format!("{}:{}", self.context.code(), lower_hex(&self.site_digest))
    }
}

impl SfsRuleFootprint {
    /// Rule identifier.
    #[must_use]
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }

    /// Canonical AST digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Static fuel bound.
    #[must_use]
    pub const fn computed_bound(&self) -> u64 {
        self.computed_bound
    }

    /// Declared node-field reads.
    #[must_use]
    pub const fn field_reads(&self) -> &BTreeSet<String> {
        &self.field_reads
    }

    /// Declared edge-field reads.
    #[must_use]
    pub const fn edge_reads(&self) -> &BTreeSet<String> {
        &self.edge_reads
    }

    /// Declared constant reads.
    #[must_use]
    pub const fn constant_reads(&self) -> &BTreeSet<String> {
        &self.constant_reads
    }

    /// Query heads.
    #[must_use]
    pub const fn queries(&self) -> &BTreeSet<String> {
        &self.queries
    }

    /// Arithmetic/comparison operator heads.
    #[must_use]
    pub const fn operators(&self) -> &BTreeSet<String> {
        &self.operators
    }

    /// Declared intrinsic call heads.
    #[must_use]
    pub const fn intrinsics(&self) -> &BTreeSet<String> {
        &self.intrinsics
    }

    /// Governed comparison/clamp context rows.
    #[must_use]
    pub const fn comparison_clamp_contexts(&self) -> &BTreeSet<String> {
        &self.comparison_clamp_contexts
    }

    /// Canonical effect rows.
    #[must_use]
    pub const fn effects(&self) -> &BTreeSet<String> {
        &self.effects
    }
}

impl SfsRuleAuditResult {
    /// Complete audited footprint.
    #[must_use]
    pub const fn footprint(&self) -> &SfsRuleFootprint {
        &self.footprint
    }

    /// Author-declared fuel budget.
    #[must_use]
    pub const fn declared_fuel(&self) -> u64 {
        self.declared_fuel
    }

    /// Complete cardinality-table identity.
    #[must_use]
    pub const fn cardinality_input_digest(&self) -> &[u8; 32] {
        &self.cardinality_input_digest
    }

    /// Complete intrinsic-cost-table identity.
    #[must_use]
    pub const fn intrinsic_cost_input_digest(&self) -> &[u8; 32] {
        &self.intrinsic_cost_input_digest
    }
}

impl SfsAuditPolicy {
    /// Construct one closed expected profile.
    ///
    /// # Errors
    ///
    /// [`SfsProfileError`] for duplicate, oversized, or forbidden rows.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rule_id: &'static str,
        expected_source_digest: [u8; 32],
        expected_computed_bound: u64,
        field_reads: impl IntoIterator<Item = &'static str>,
        edge_reads: impl IntoIterator<Item = &'static str>,
        constant_reads: impl IntoIterator<Item = &'static str>,
        queries: impl IntoIterator<Item = &'static str>,
        operators: impl IntoIterator<Item = &'static str>,
        intrinsics: impl IntoIterator<Item = &'static str>,
        comparison_sites: Vec<GovernedComparisonSite>,
        effects: impl IntoIterator<Item = &'static str>,
    ) -> Result<Self, SfsProfileError> {
        let field_reads = policy_set("field_reads", field_reads)?;
        let edge_reads = policy_set("edge_reads", edge_reads)?;
        let constant_reads = policy_set("constant_reads", constant_reads)?;
        let queries = policy_set("queries", queries)?;
        let operators = policy_set("operators", operators)?;
        let intrinsics = policy_set("intrinsics", intrinsics)?;
        let effects = policy_set("effects", effects)?;
        reject_forbidden_expected([
            &field_reads,
            &edge_reads,
            &constant_reads,
            &queries,
            &operators,
            &intrinsics,
            &effects,
        ])?;
        let comparison_clamp_contexts = comparison_policy_set(&comparison_sites)?;
        Ok(Self {
            expected: SfsRuleFootprint {
                rule_id: rule_id.to_owned(),
                source_digest: expected_source_digest,
                computed_bound: expected_computed_bound,
                field_reads,
                edge_reads,
                constant_reads,
                queries,
                operators,
                intrinsics,
                comparison_clamp_contexts,
                effects,
            },
            comparison_sites,
        })
    }

    /// Exact expected footprint.
    #[must_use]
    pub const fn expected_footprint(&self) -> &SfsRuleFootprint {
        &self.expected
    }
}

fn policy_set(
    name: &'static str,
    values: impl IntoIterator<Item = &'static str>,
) -> Result<BTreeSet<String>, SfsProfileError> {
    let mut staged = Vec::new();
    let mut entries = values.into_iter();
    for _index in 0..=MAX_POLICY_ENTRIES {
        let Some(value) = entries.next() else {
            break;
        };
        staged.push(value);
        if staged.len() > MAX_POLICY_ENTRIES {
            return Err(SfsProfileError::PolicyEntryLimit {
                set: name,
                actual: staged.len(),
            });
        }
    }
    let mut output = BTreeSet::new();
    for index in 0..MAX_POLICY_ENTRIES {
        let Some(value) = staged.get(index) else {
            break;
        };
        if !output.insert((*value).to_owned()) {
            return Err(SfsProfileError::DuplicatePolicyEntry { set: name });
        }
    }
    Ok(output)
}

fn comparison_policy_set(
    sites: &[GovernedComparisonSite],
) -> Result<BTreeSet<String>, SfsProfileError> {
    if sites.len() > MAX_POLICY_ENTRIES {
        return Err(SfsProfileError::PolicyEntryLimit {
            set: "comparison_clamp_contexts",
            actual: sites.len(),
        });
    }
    let mut output = BTreeSet::new();
    let mut digests = BTreeSet::new();
    for index in 0..MAX_POLICY_ENTRIES {
        let Some(site) = sites.get(index) else { break };
        if !digests.insert(site.site_digest) || !output.insert(site.profile_entry()) {
            return Err(SfsProfileError::DuplicatePolicyEntry {
                set: "comparison_clamp_contexts",
            });
        }
    }
    Ok(output)
}

fn forbidden_observable(entry: &str) -> bool {
    for suffix in FORBIDDEN_OBSERVABLE_SUFFIXES {
        let forbidden = format!("sfs/{suffix}");
        if entry == forbidden || entry.ends_with(&format!(":{forbidden}")) {
            return true;
        }
    }
    false
}

fn reject_forbidden_expected(sets: [&BTreeSet<String>; 7]) -> Result<(), SfsProfileError> {
    for set in sets {
        for index in 0..=MAX_POLICY_ENTRIES {
            let Some(entry) = set.iter().nth(index) else {
                break;
            };
            if forbidden_observable(entry) {
                return Err(SfsProfileError::ForbiddenObservable {
                    entry: entry.clone(),
                });
            }
        }
    }
    Ok(())
}

fn preflight_ast(rule: &SExpr) -> Result<(), SfsProfileError> {
    validate_ast_walk_bounds(rule, AST_WALK_LIMITS, AUDITOR)
        .map_err(|_error| SfsProfileError::AstWalkLimit)
}

fn canonical_digest(rule: &SExpr) -> Result<[u8; 32], SfsProfileError> {
    canonical_bytes(rule)
        .map(|bytes| sha256_of(&bytes))
        .map_err(|_error| SfsProfileError::CanonicalAst)
}

fn resolve_form_path<'a>(root: &'a SExpr, path: &[u32]) -> Result<&'a SExpr, SfsProfileError> {
    let mut selected = root;
    for index in 1..=MAX_AST_WALK_DEPTH {
        if index >= path.len() {
            return Ok(selected);
        }
        let SExpr::List(items) = selected else {
            return Err(SfsProfileError::UnknownFormPath);
        };
        let child =
            usize::try_from(path[index]).map_err(|_error| SfsProfileError::UnknownFormPath)?;
        selected = items.get(child).ok_or(SfsProfileError::UnknownFormPath)?;
    }
    if path.len() == MAX_AST_WALK_DEPTH + 1 {
        Ok(selected)
    } else {
        Err(SfsProfileError::UnknownFormPath)
    }
}

fn form_head(expr: &SExpr) -> Option<&str> {
    let SExpr::List(items) = expr else {
        return None;
    };
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(head) | Atom::Operator(head))) => Some(head),
        _ => None,
    }
}

fn is_comparison_or_clamp(expr: &SExpr) -> bool {
    matches!(
        form_head(expr),
        Some("<" | "<=" | ">" | ">=" | "=" | "!=" | "clamp")
    )
}

fn site_digest(source_digest: [u8; 32], path: &[u32]) -> Result<[u8; 32], SfsProfileError> {
    let count = u16::try_from(path.len())
        .map_err(|_error| SfsProfileError::FormPathLimit { actual: path.len() })?;
    let mut preimage = b"babylon.sfs-comparison-site.v1\0".to_vec();
    preimage.extend_from_slice(&source_digest);
    preimage.extend_from_slice(&count.to_be_bytes());
    for index in 0..=MAX_AST_WALK_DEPTH {
        let Some(component) = path.get(index) else {
            break;
        };
        preimage.extend_from_slice(&component.to_be_bytes());
    }
    Ok(sha256_of(&preimage))
}

fn lower_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for index in 0..32 {
        output.push(char::from(HEX[usize::from(bytes[index] >> 4)]));
        output.push(char::from(HEX[usize::from(bytes[index] & 0x0f)]));
    }
    output
}

struct Preflight {
    source_digest: [u8; 32],
    computed_bound: u64,
    declared_fuel: u64,
    cardinality_digest: [u8; 32],
    intrinsic_digest: [u8; 32],
}

#[derive(Default)]
struct SemanticFacts {
    rule_id: String,
    field_reads: BTreeSet<String>,
    edge_reads: BTreeSet<String>,
    constant_reads: BTreeSet<String>,
    queries: BTreeSet<String>,
    operators: BTreeSet<String>,
    intrinsics: BTreeSet<String>,
    effects: BTreeSet<String>,
    comparison_sites: BTreeSet<[u8; 32]>,
    time_sources: BTreeSet<ForbiddenBindingSource>,
    time_bindings: BTreeMap<String, ForbiddenBindingSource>,
    forbidden_observables: BTreeSet<String>,
    forbidden_intrinsics: BTreeSet<String>,
    forbidden_comparison_uses: BTreeSet<[u8; 32]>,
    response_table: bool,
    threshold_ladder: bool,
    absolute_schedule: bool,
}

struct Located<'a> {
    expression: &'a SExpr,
    path: Vec<u32>,
    guard_scope: Option<Vec<u32>>,
}

#[derive(Default)]
struct ThresholdIndex {
    boolean_paths: BTreeSet<Vec<u32>>,
    direct: ThresholdLiterals,
    descendants: ThresholdLiterals,
}

type ThresholdKey = (Vec<u32>, String, Vec<u32>);
type ThresholdLiterals = BTreeMap<ThresholdKey, BTreeSet<Vec<u8>>>;

fn rule_items(rule: &SExpr) -> Result<&[SExpr], SfsProfileError> {
    let SExpr::List(items) = rule else {
        return Err(SfsProfileError::CanonicalAst);
    };
    if form_head(rule) != Some("rule") {
        return Err(SfsProfileError::CanonicalAst);
    }
    Ok(items)
}

fn rule_id(rule: &SExpr) -> Result<String, SfsProfileError> {
    match rule_items(rule)?.get(1) {
        Some(SExpr::Atom(Atom::QName(value))) => Ok(value.clone()),
        _ => Err(SfsProfileError::CanonicalAst),
    }
}

fn preflight(
    rule: &SExpr,
    ceilings: &CardinalityCeilings,
    intrinsic_costs: &IntrinsicCosts,
) -> Result<Preflight, SfsProfileError> {
    preflight_ast(rule)?;
    let source_digest = canonical_digest(rule)?;
    let cardinality_digest = ceilings.sfs_identity_digest()?;
    let intrinsic_digest = intrinsic_costs.sfs_identity_digest()?;
    let computed_bound = check_rule(rule, ceilings, intrinsic_costs)?;
    let declared_fuel = declared_fuel(rule_items(rule)?)?;
    Ok(Preflight {
        source_digest,
        computed_bound,
        declared_fuel,
        cardinality_digest,
        intrinsic_digest,
    })
}

fn binding_facts(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    facts: &mut SemanticFacts,
) -> Result<(), SfsProfileError> {
    let bindings = parse_bindings(rule).map_err(|_error| SfsProfileError::CanonicalAst)?;
    for index in 0..MAX_AST_WALK_NODES {
        let Some(binding) = bindings.get(index) else {
            break;
        };
        match &binding.source {
            BindSource::Field(field) => classify_field(field, vocabulary, facts)?,
            BindSource::Const(value) => {
                if forbidden_observable(value) {
                    facts.forbidden_observables.insert(value.clone());
                } else {
                    facts.constant_reads.insert(value.clone());
                }
            }
            BindSource::Tick => {
                facts.time_sources.insert(ForbiddenBindingSource::Tick);
                facts
                    .time_bindings
                    .insert(binding.name.clone(), ForbiddenBindingSource::Tick);
            }
            BindSource::Year => {
                facts.time_sources.insert(ForbiddenBindingSource::Year);
                facts
                    .time_bindings
                    .insert(binding.name.clone(), ForbiddenBindingSource::Year);
            }
            BindSource::TickOfYear => {
                facts
                    .time_sources
                    .insert(ForbiddenBindingSource::TickOfYear);
                facts
                    .time_bindings
                    .insert(binding.name.clone(), ForbiddenBindingSource::TickOfYear);
            }
            BindSource::TickInCycle(_) => {
                facts
                    .time_sources
                    .insert(ForbiddenBindingSource::TickInCycle);
                facts
                    .time_bindings
                    .insert(binding.name.clone(), ForbiddenBindingSource::TickInCycle);
            }
            BindSource::Expr(expression) => {
                if uses_time_binding(expression, &facts.time_bindings)? {
                    if let Some(source) = first_time_source(&facts.time_sources) {
                        facts.time_bindings.insert(binding.name.clone(), source);
                    }
                }
            }
            BindSource::Metric(_) => {}
        }
    }
    Ok(())
}

fn classify_field(
    field: &str,
    vocabulary: &ClosedVocabulary,
    facts: &mut SemanticFacts,
) -> Result<(), SfsProfileError> {
    if forbidden_observable(field) {
        facts.forbidden_observables.insert(field.to_owned());
        return Ok(());
    }
    let (owner, _member) =
        vocabulary
            .owner_of_field(field)
            .map_err(|_error| SfsProfileError::UnknownFieldOwner {
                field: field.to_owned(),
            })?;
    if owner == EnumKind::EdgeType {
        facts.edge_reads.insert(field.to_owned());
    } else {
        facts.field_reads.insert(field.to_owned());
    }
    Ok(())
}

fn children_to_stack<'a>(
    items: &'a [SExpr],
    located: &Located<'a>,
    stack: &mut Vec<Located<'a>>,
) -> Result<(), SfsProfileError> {
    let child_count = items.len();
    if child_count > MAX_AST_WALK_STACK
        || stack.len().saturating_add(child_count) > MAX_AST_WALK_STACK
    {
        return Err(SfsProfileError::AstWalkLimit);
    }
    for offset in 0..MAX_AST_WALK_STACK {
        if offset >= child_count {
            break;
        }
        let index = child_count - 1 - offset;
        let mut path = located.path.clone();
        path.push(u32::try_from(index).map_err(|_error| SfsProfileError::AstWalkLimit)?);
        let parent_head = form_head(located.expression);
        let starts_guard =
            (parent_head == Some("when") || parent_head == Some("guard")) && index == 1;
        let guard_scope = if starts_guard {
            Some(located.path.clone())
        } else {
            located.guard_scope.clone()
        };
        stack.push(Located {
            expression: &items[index],
            path,
            guard_scope,
        });
    }
    Ok(())
}

fn record_head(head: &str, intrinsic_costs: &IntrinsicCosts, facts: &mut SemanticFacts) {
    if QUERY_HEADS.contains(&head) {
        facts.queries.insert(head.to_owned());
    }
    if FORBIDDEN_INTRINSICS.contains(&head) {
        facts.forbidden_intrinsics.insert(head.to_owned());
    }
    if intrinsic_costs.declared_cost(head).is_some() {
        facts.intrinsics.insert(head.to_owned());
    }
}

fn operator_head(expr: &SExpr) -> Option<&str> {
    let SExpr::List(items) = expr else {
        return None;
    };
    match items.first() {
        Some(SExpr::Atom(Atom::Operator(head))) => Some(head),
        _ => None,
    }
}

fn is_numeric(expr: &SExpr) -> bool {
    matches!(
        expr,
        SExpr::Atom(Atom::Int(_) | Atom::Currency(_) | Atom::Scaled(_))
    )
}

fn uses_time_binding(
    expression: &SExpr,
    bindings: &BTreeMap<String, ForbiddenBindingSource>,
) -> Result<bool, SfsProfileError> {
    let mut stack = vec![expression];
    for _visited in 0..MAX_AST_WALK_NODES {
        let Some(current) = stack.pop() else {
            return Ok(false);
        };
        if let SExpr::Atom(Atom::Symbol(name)) = current {
            if bindings.contains_key(name) {
                return Ok(true);
            }
            continue;
        }
        let SExpr::List(items) = current else {
            continue;
        };
        if stack.len().saturating_add(items.len()) > MAX_AST_WALK_STACK {
            return Err(SfsProfileError::AstWalkLimit);
        }
        for offset in 0..MAX_AST_WALK_STACK {
            if offset >= items.len() {
                break;
            }
            stack.push(&items[items.len() - 1 - offset]);
        }
    }
    Err(SfsProfileError::AstWalkLimit)
}

fn magnitude_selector_digest(
    expression: &SExpr,
    path: &[u32],
    source_digest: [u8; 32],
) -> Result<Option<[u8; 32]>, SfsProfileError> {
    let SExpr::List(items) = expression else {
        return Ok(None);
    };
    if form_head(expression) != Some("if") || items.len() != 4 {
        return Ok(None);
    }
    if !is_comparison_or_clamp(&items[1]) || !is_numeric(&items[2]) || !is_numeric(&items[3]) {
        return Ok(None);
    }
    if items[2] == items[3] {
        return Ok(None);
    }
    let mut comparison_path = path.to_vec();
    comparison_path.push(1);
    site_digest(source_digest, &comparison_path).map(Some)
}

fn numeric_key(expr: &SExpr) -> Option<Vec<u8>> {
    if !is_numeric(expr) {
        return None;
    }
    canonical_bytes(expr).ok()
}

fn threshold_pair(expr: &SExpr) -> Option<(String, Vec<u8>)> {
    let SExpr::List(items) = expr else {
        return None;
    };
    if !is_comparison_or_clamp(expr) || items.len() < 3 {
        return None;
    }
    let SExpr::Atom(Atom::Symbol(binding)) = &items[1] else {
        return None;
    };
    numeric_key(&items[2]).map(|literal| (binding.clone(), literal))
}

fn has_distinct_literal(values: Option<&BTreeSet<Vec<u8>>>, literal: &[u8]) -> bool {
    let Some(values) = values else { return false };
    values.len() > 1 || (values.len() == 1 && !values.contains(literal))
}

fn threshold_containers(located: &Located<'_>, index: &ThresholdIndex) -> Vec<Vec<u32>> {
    let mut output = Vec::new();
    let Some(scope) = &located.guard_scope else {
        return output;
    };
    for depth in 1..=MAX_AST_WALK_DEPTH {
        if depth >= located.path.len() {
            break;
        }
        let candidate = located.path[..depth].to_vec();
        if candidate.len() > scope.len() && index.boolean_paths.contains(&candidate) {
            output.push(candidate);
        }
    }
    output
}

fn record_threshold(
    located: &Located<'_>,
    binding: &str,
    literal: &[u8],
    index: &mut ThresholdIndex,
) -> bool {
    let Some(scope) = &located.guard_scope else {
        return false;
    };
    let containers = threshold_containers(located, index);
    let Some(current) = containers.last() else {
        return false;
    };
    let current_key = (scope.clone(), binding.to_owned(), current.clone());
    if has_distinct_literal(index.descendants.get(&current_key), literal) {
        return true;
    }
    for container_index in 0..MAX_AST_WALK_DEPTH {
        if container_index + 1 >= containers.len() {
            break;
        }
        let key = (
            scope.clone(),
            binding.to_owned(),
            containers[container_index].clone(),
        );
        if has_distinct_literal(index.direct.get(&key), literal) {
            return true;
        }
    }
    index
        .direct
        .entry(current_key)
        .or_default()
        .insert(literal.to_owned());
    for container_index in 0..MAX_AST_WALK_DEPTH {
        if container_index + 1 >= containers.len() {
            break;
        }
        let key = (
            scope.clone(),
            binding.to_owned(),
            containers[container_index].clone(),
        );
        index
            .descendants
            .entry(key)
            .or_default()
            .insert(literal.to_owned());
    }
    false
}

fn fixed_guard_effect(expr: &SExpr) -> Option<(String, Vec<u8>)> {
    let SExpr::List(guard) = expr else {
        return None;
    };
    if form_head(expr) != Some("guard") || guard.len() < 3 {
        return None;
    }
    let SExpr::List(update) = &guard[2] else {
        return None;
    };
    if !matches!(
        form_head(&guard[2]),
        Some("update-node" | "update-edge" | "update-hyperedge")
    ) {
        return None;
    }
    let SExpr::Atom(Atom::QName(target)) = update.get(2)? else {
        return None;
    };
    let SExpr::List(operation) = update.get(3)? else {
        return None;
    };
    if form_head(update.get(3)?) != Some("set") {
        return None;
    }
    numeric_key(operation.get(1)?).map(|value| (target.clone(), value))
}

fn has_response_table(expr: &SExpr) -> bool {
    let SExpr::List(items) = expr else {
        return false;
    };
    if form_head(expr) != Some("effects") {
        return false;
    }
    let mut rows: BTreeMap<String, BTreeSet<Vec<u8>>> = BTreeMap::new();
    for index in 1..MAX_AST_WALK_STACK {
        let Some(child) = items.get(index) else { break };
        if let Some((target, value)) = fixed_guard_effect(child) {
            rows.entry(target).or_default().insert(value);
        }
    }
    let mut values = rows.values();
    for _index in 0..MAX_AST_WALK_STACK {
        let Some(value_set) = values.next() else {
            break;
        };
        if value_set.len() >= 2 {
            return true;
        }
    }
    false
}

fn effect_row(effect: EffectSignature) -> String {
    match effect {
        EffectSignature::NodeField(field) => format!("node:{field}"),
        EffectSignature::EdgeField(field) => format!("edge:{field}"),
        EffectSignature::HyperedgeField(field) => format!("hyperedge:{field}"),
        EffectSignature::Event(event) => format!("event:{event}"),
        EffectSignature::Shape(verb) => match verb {
            ShapeVerb::AddNode => "shape:add-node".to_owned(),
            ShapeVerb::RemoveNode => "shape:remove-node".to_owned(),
            ShapeVerb::AddEdge => "shape:add-edge".to_owned(),
            ShapeVerb::RemoveEdge => "shape:remove-edge".to_owned(),
            ShapeVerb::AddHyperedge => "shape:add-hyperedge".to_owned(),
            ShapeVerb::RemoveHyperedge => "shape:remove-hyperedge".to_owned(),
        },
    }
}

fn collect_effects(rule: &SExpr, facts: &mut SemanticFacts) -> Result<(), SfsProfileError> {
    let effects = effect_footprint(rule).map_err(|_error| SfsProfileError::AstWalkLimit)?;
    for index in 0..MAX_AST_WALK_NODES {
        let Some(effect) = effects.get(index) else {
            break;
        };
        let row = effect_row(effect.clone());
        if forbidden_observable(&row) {
            facts.forbidden_observables.insert(row);
        } else {
            facts.effects.insert(row);
        }
    }
    Ok(())
}

fn scan_expression(
    located: &Located<'_>,
    vocabulary: &ClosedVocabulary,
    intrinsic_costs: &IntrinsicCosts,
    source_digest: [u8; 32],
    thresholds: &mut ThresholdIndex,
    facts: &mut SemanticFacts,
) -> Result<(), SfsProfileError> {
    let SExpr::List(items) = located.expression else {
        return Ok(());
    };
    let Some(head) = form_head(located.expression) else {
        return Ok(());
    };
    if matches!(head, "and" | "or") {
        thresholds.boolean_paths.insert(located.path.clone());
    }
    record_head(head, intrinsic_costs, facts);
    if operator_head(located.expression).is_some() {
        facts.operators.insert(head.to_owned());
    }
    if head == "field-of" {
        if let Some(SExpr::Atom(Atom::QName(field))) = items.get(2) {
            classify_field(field, vocabulary, facts)?;
        }
    }
    if is_comparison_or_clamp(located.expression) {
        let digest = site_digest(source_digest, &located.path)?;
        facts.comparison_sites.insert(digest);
        if located.guard_scope.is_some()
            && uses_time_binding(located.expression, &facts.time_bindings)?
        {
            facts.absolute_schedule = true;
        }
        if let Some((binding, literal)) = threshold_pair(located.expression) {
            facts.threshold_ladder |= record_threshold(located, &binding, &literal, thresholds);
        }
    }
    if let Some(digest) =
        magnitude_selector_digest(located.expression, &located.path, source_digest)?
    {
        facts.forbidden_comparison_uses.insert(digest);
    }
    facts.response_table |= has_response_table(located.expression);
    Ok(())
}

fn extract_semantic_facts(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    intrinsic_costs: &IntrinsicCosts,
    source_digest: [u8; 32],
) -> Result<SemanticFacts, SfsProfileError> {
    let mut facts = SemanticFacts {
        rule_id: rule_id(rule)?,
        ..SemanticFacts::default()
    };
    binding_facts(rule, vocabulary, &mut facts)?;
    let mut thresholds = ThresholdIndex::default();
    let mut stack = vec![Located {
        expression: rule,
        path: vec![0],
        guard_scope: None,
    }];
    for _visited in 0..MAX_AST_WALK_NODES {
        let Some(located) = stack.pop() else { break };
        scan_expression(
            &located,
            vocabulary,
            intrinsic_costs,
            source_digest,
            &mut thresholds,
            &mut facts,
        )?;
        if let SExpr::List(items) = located.expression {
            children_to_stack(items, &located, &mut stack)?;
        }
    }
    if !stack.is_empty() {
        return Err(SfsProfileError::AstWalkLimit);
    }
    collect_effects(rule, &mut facts)?;
    Ok(facts)
}

fn semantic_refusal(facts: &SemanticFacts) -> Option<SfsProfileError> {
    if facts.absolute_schedule {
        return Some(SfsProfileError::ForbiddenAbsoluteSchedule);
    }
    if let Some(source) = first_time_source(&facts.time_sources) {
        return Some(SfsProfileError::ForbiddenBindingSource(source));
    }
    if let Some(entry) = facts.forbidden_observables.first() {
        return Some(SfsProfileError::ForbiddenObservable {
            entry: entry.clone(),
        });
    }
    if let Some(name) = facts.forbidden_intrinsics.first() {
        return Some(SfsProfileError::ForbiddenIntrinsic { name: name.clone() });
    }
    if facts.response_table {
        return Some(SfsProfileError::ForbiddenResponseTable);
    }
    if facts.threshold_ladder {
        return Some(SfsProfileError::ForbiddenThresholdLadder);
    }
    facts
        .forbidden_comparison_uses
        .first()
        .map(|digest| SfsProfileError::ForbiddenComparisonUse {
            site_digest: *digest,
        })
}

fn first_time_source(sources: &BTreeSet<ForbiddenBindingSource>) -> Option<ForbiddenBindingSource> {
    if sources.contains(&ForbiddenBindingSource::Tick) {
        Some(ForbiddenBindingSource::Tick)
    } else if sources.contains(&ForbiddenBindingSource::TickInCycle) {
        Some(ForbiddenBindingSource::TickInCycle)
    } else if sources.contains(&ForbiddenBindingSource::TickOfYear) {
        Some(ForbiddenBindingSource::TickOfYear)
    } else if sources.contains(&ForbiddenBindingSource::Year) {
        Some(ForbiddenBindingSource::Year)
    } else {
        None
    }
}

fn comparison_contexts(
    actual_sites: &BTreeSet<[u8; 32]>,
    governed_sites: &[GovernedComparisonSite],
) -> Result<BTreeSet<String>, SfsProfileError> {
    if governed_sites.len() > MAX_POLICY_ENTRIES {
        return Err(SfsProfileError::PolicyEntryLimit {
            set: "comparison_clamp_contexts",
            actual: governed_sites.len(),
        });
    }
    let mut governed = BTreeMap::new();
    for index in 0..MAX_POLICY_ENTRIES {
        let Some(site) = governed_sites.get(index) else {
            break;
        };
        if governed.insert(site.site_digest, site).is_some() {
            return Err(SfsProfileError::DuplicatePolicyEntry {
                set: "comparison_clamp_contexts",
            });
        }
    }
    let mut actual = actual_sites.iter();
    for _index in 0..MAX_AST_WALK_NODES {
        let Some(actual) = actual.next() else {
            break;
        };
        if !governed.contains_key(actual) {
            return Err(SfsProfileError::MissingComparisonContext {
                site_digest: *actual,
            });
        }
    }
    for index in 0..MAX_POLICY_ENTRIES {
        let Some(governed_digest) = governed.keys().nth(index) else {
            break;
        };
        if !actual_sites.contains(governed_digest) {
            return Err(SfsProfileError::DeadComparisonContext {
                site_digest: *governed_digest,
            });
        }
    }
    let mut output = BTreeSet::new();
    for index in 0..MAX_POLICY_ENTRIES {
        let Some(site) = governed.values().nth(index) else {
            break;
        };
        output.insert(site.profile_entry());
    }
    Ok(output)
}

fn footprint_from_facts(
    facts: SemanticFacts,
    preflight: &Preflight,
    comparison_clamp_contexts: BTreeSet<String>,
) -> SfsRuleFootprint {
    SfsRuleFootprint {
        rule_id: facts.rule_id,
        source_digest: preflight.source_digest,
        computed_bound: preflight.computed_bound,
        field_reads: facts.field_reads,
        edge_reads: facts.edge_reads,
        constant_reads: facts.constant_reads,
        queries: facts.queries,
        operators: facts.operators,
        intrinsics: facts.intrinsics,
        comparison_clamp_contexts,
        effects: facts.effects,
    }
}

fn seal_result(footprint: SfsRuleFootprint, preflight: &Preflight) -> SfsRuleAuditResult {
    SfsRuleAuditResult {
        footprint,
        declared_fuel: preflight.declared_fuel,
        cardinality_input_digest: preflight.cardinality_digest,
        intrinsic_cost_input_digest: preflight.intrinsic_digest,
    }
}

fn first_extra(actual: &BTreeSet<String>, expected: &BTreeSet<String>) -> Option<String> {
    let mut values = actual.iter();
    for _index in 0..MAX_AST_WALK_NODES {
        let Some(value) = values.next() else { break };
        if !expected.contains(value) {
            return Some(value.clone());
        }
    }
    None
}

fn stage_extra(
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    extras: &mut BTreeSet<String>,
) {
    if let Some(entry) = first_extra(actual, expected) {
        extras.insert(entry);
    }
}

fn unexpected_read(
    actual: &SfsRuleFootprint,
    expected: &SfsRuleFootprint,
) -> Option<SfsProfileError> {
    let mut extras = BTreeSet::new();
    stage_extra(&actual.field_reads, &expected.field_reads, &mut extras);
    stage_extra(&actual.edge_reads, &expected.edge_reads, &mut extras);
    stage_extra(
        &actual.constant_reads,
        &expected.constant_reads,
        &mut extras,
    );
    stage_extra(&actual.queries, &expected.queries, &mut extras);
    stage_extra(&actual.operators, &expected.operators, &mut extras);
    stage_extra(&actual.intrinsics, &expected.intrinsics, &mut extras);
    extras.first().map(|entry| SfsProfileError::UnexpectedRead {
        entry: entry.clone(),
    })
}

fn exact_set_mismatch(
    actual: &SfsRuleFootprint,
    expected: &SfsRuleFootprint,
) -> Option<SfsProfileError> {
    let ordered = [
        ("field_reads", &actual.field_reads, &expected.field_reads),
        ("edge_reads", &actual.edge_reads, &expected.edge_reads),
        (
            "constant_reads",
            &actual.constant_reads,
            &expected.constant_reads,
        ),
        ("queries", &actual.queries, &expected.queries),
        ("operators", &actual.operators, &expected.operators),
        ("intrinsics", &actual.intrinsics, &expected.intrinsics),
        (
            "comparison_clamp_contexts",
            &actual.comparison_clamp_contexts,
            &expected.comparison_clamp_contexts,
        ),
        ("effects", &actual.effects, &expected.effects),
    ];
    for (set, actual_set, expected_set) in ordered {
        if actual_set != expected_set {
            return Some(SfsProfileError::FootprintMismatch { set });
        }
    }
    None
}

/// Extract and seal one complete opt-in footprint without engine authority.
///
/// # Errors
///
/// [`SfsProfileError`] at the exact bounded and semantic refusal boundary.
pub fn audit_rule_footprint(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    ceilings: &CardinalityCeilings,
    intrinsic_costs: &IntrinsicCosts,
    comparison_sites: &[GovernedComparisonSite],
) -> Result<SfsRuleAuditResult, SfsProfileError> {
    let checked = preflight(rule, ceilings, intrinsic_costs)?;
    let facts = extract_semantic_facts(rule, vocabulary, intrinsic_costs, checked.source_digest)?;
    if let Some(error) = semantic_refusal(&facts) {
        return Err(error);
    }
    let contexts = comparison_contexts(&facts.comparison_sites, comparison_sites)?;
    let footprint = footprint_from_facts(facts, &checked, contexts);
    Ok(seal_result(footprint, &checked))
}

/// Validate an opted-in rule against one exact, symmetric footprint policy.
///
/// # Errors
///
/// [`SfsProfileError`] in the precedence fixed by the synthetic audit contract.
pub fn validate_sfs_rule_profile(
    rule: &SExpr,
    vocabulary: &ClosedVocabulary,
    ceilings: &CardinalityCeilings,
    intrinsic_costs: &IntrinsicCosts,
    policy: &SfsAuditPolicy,
) -> Result<SfsRuleAuditResult, SfsProfileError> {
    let checked = preflight(rule, ceilings, intrinsic_costs)?;
    if checked.computed_bound != policy.expected.computed_bound {
        return Err(SfsProfileError::ComputedBoundMismatch {
            expected: policy.expected.computed_bound,
            actual: checked.computed_bound,
        });
    }
    let facts = extract_semantic_facts(rule, vocabulary, intrinsic_costs, checked.source_digest)?;
    if let Some(error) = semantic_refusal(&facts) {
        return Err(error);
    }
    if checked.source_digest != policy.expected.source_digest {
        return Err(SfsProfileError::SourceDigestMismatch);
    }
    let contexts = comparison_contexts(&facts.comparison_sites, &policy.comparison_sites)?;
    let footprint = footprint_from_facts(facts, &checked, contexts);
    if let Some(error) = unexpected_read(&footprint, &policy.expected) {
        return Err(error);
    }
    if let Some(entry) = first_extra(&footprint.effects, &policy.expected.effects) {
        return Err(SfsProfileError::UnexpectedEffect { entry });
    }
    if let Some(error) = exact_set_mismatch(&footprint, &policy.expected) {
        return Err(error);
    }
    Ok(seal_result(footprint, &checked))
}
