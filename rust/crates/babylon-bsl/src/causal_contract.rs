//! Role-sensitive causal composition for BSL rules and built-in native
//! compositions (PER-19 / ADR224 / ADR255).

use crate::reader::{Atom, SExpr};
use crate::write_log::{Write, WriteRecord};

/// Maximum source-tree depth inspected by a load-time AST walker.
///
/// The reader itself remains iterative and accepts deeper syntax so it can
/// return a positioned parse result. Semantic load passes then refuse trees
/// beyond this explicit resource contract rather than recursing until the
/// process stack fails.
pub const MAX_AST_WALK_DEPTH: usize = 256;
/// Maximum nodes inspected by one load-time AST walker.
pub const MAX_AST_WALK_NODES: usize = 1_048_576;
/// Maximum pending nodes held by one load-time AST walker's explicit stack.
pub const MAX_AST_WALK_STACK: usize = 65_536;

const CAUSAL_WALKER: &str = "causal effect footprint";

/// Which statically declared AST-walk resource boundary was exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstWalkLimit {
    /// Maximum root-to-node depth.
    Depth,
    /// Maximum inspected node count.
    Nodes,
    /// Maximum pending-node stack size.
    Stack,
}

/// A loud, typed load-time AST resource refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstWalkError {
    /// Static analyzer that refused the tree.
    pub analyzer: &'static str,
    /// Resource boundary reached.
    pub limit: AstWalkLimit,
    /// Inclusive configured maximum.
    pub maximum: usize,
}

impl AstWalkError {
    pub(crate) const fn new(analyzer: &'static str, limit: AstWalkLimit, maximum: usize) -> Self {
        Self {
            analyzer,
            limit,
            maximum,
        }
    }
}

impl std::fmt::Display for AstWalkError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} exceeded its {:?} bound of {} while inspecting the rule AST",
            self.analyzer, self.limit, self.maximum
        )
    }
}

impl std::error::Error for AstWalkError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AstWalkLimits {
    depth: usize,
    nodes: usize,
    stack: usize,
}

impl AstWalkLimits {
    pub(crate) const fn new(depth: usize, nodes: usize, stack: usize) -> Self {
        assert!(depth <= MAX_AST_WALK_DEPTH);
        assert!(nodes <= MAX_AST_WALK_NODES);
        assert!(stack <= MAX_AST_WALK_STACK);
        Self {
            depth,
            nodes,
            stack,
        }
    }

    pub(crate) const fn depth(self) -> usize {
        self.depth
    }

    pub(crate) const fn nodes(self) -> usize {
        self.nodes
    }

    pub(crate) const fn stack(self) -> usize {
        self.stack
    }
}

pub(crate) const AST_WALK_LIMITS: AstWalkLimits =
    AstWalkLimits::new(MAX_AST_WALK_DEPTH, MAX_AST_WALK_NODES, MAX_AST_WALK_STACK);

fn ast_walk_error(analyzer: &'static str, limit: AstWalkLimit, maximum: usize) -> AstWalkError {
    AstWalkError::new(analyzer, limit, maximum)
}

fn checked_child_depth(
    depth: usize,
    limits: AstWalkLimits,
    analyzer: &'static str,
) -> Result<usize, AstWalkError> {
    let child_depth = depth
        .checked_add(1)
        .ok_or_else(|| ast_walk_error(analyzer, AstWalkLimit::Depth, limits.depth))?;
    if child_depth > limits.depth {
        return Err(ast_walk_error(analyzer, AstWalkLimit::Depth, limits.depth));
    }
    Ok(child_depth)
}

fn check_stack_capacity(
    current: usize,
    additional: usize,
    limits: AstWalkLimits,
    analyzer: &'static str,
) -> Result<(), AstWalkError> {
    let required = current
        .checked_add(additional)
        .ok_or_else(|| ast_walk_error(analyzer, AstWalkLimit::Stack, limits.stack))?;
    if required > limits.stack {
        return Err(ast_walk_error(analyzer, AstWalkLimit::Stack, limits.stack));
    }
    Ok(())
}

/// Prove depth, total-node, and explicit-stack limits before a semantic walk.
///
/// The fixed `0..MAX_AST_WALK_NODES` loop is the traversal's static iteration
/// ceiling. A lower test limit refuses before that ceiling. Each body consumes
/// exactly one pending node; children enter the explicit stack only after a
/// checked capacity preflight.
pub(crate) fn validate_ast_walk_bounds(
    root: &SExpr,
    limits: AstWalkLimits,
    analyzer: &'static str,
) -> Result<(), AstWalkError> {
    if limits.nodes == 0 {
        return Err(ast_walk_error(analyzer, AstWalkLimit::Nodes, 0));
    }
    check_stack_capacity(0, 1, limits, analyzer)?;
    let mut stack = vec![(root, 0_usize)];
    for visited in 0..MAX_AST_WALK_NODES {
        if visited >= limits.nodes() {
            break;
        }
        let Some((expr, depth)) = stack.pop() else {
            return Ok(());
        };
        let SExpr::List(items) = expr else { continue };
        if items.is_empty() {
            continue;
        }
        let child_depth = checked_child_depth(depth, limits, analyzer)?;
        check_stack_capacity(stack.len(), items.len(), limits, analyzer)?;
        for child in items.iter().rev() {
            stack.push((child, child_depth));
        }
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err(ast_walk_error(
            analyzer,
            AstWalkLimit::Nodes,
            limits.nodes(),
        ))
    }
}

/// The causal role a BSL rule declares. These values belong to the parser,
/// not to content-owned `defenum` declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleRole {
    /// An endogenous state transformation.
    Mechanic,
    /// An endogenous pattern recognizer.
    Recognizer,
    /// Exogenous event content, including shocks.
    ExternalEvent,
    /// A next-week player or non-player intent.
    Intent,
}

impl RuleRole {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "mechanic" => Some(Self::Mechanic),
            "recognizer" => Some(Self::Recognizer),
            "external-event" => Some(Self::ExternalEvent),
            "intent" => Some(Self::Intent),
            _ => None,
        }
    }
}

/// The constitutional evidence class carried by a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceClass {
    /// Directly transcribed from a named source and vintage.
    Observed,
    /// Computed through a declared transformation.
    Derived,
    /// Selected against a declared signature or operating range.
    Calibrated,
    /// Chosen as an explicit game-design liberty.
    Designed,
}

impl EvidenceClass {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "observed" => Some(Self::Observed),
            "derived" => Some(Self::Derived),
            "calibrated" => Some(Self::Calibrated),
            "designed" => Some(Self::Designed),
            _ => None,
        }
    }
}

/// The complete role/evidence attribution of one BSL rule or native composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleContract {
    /// The BSL rule's qualified identifier or an admitted native composition ID.
    pub rule_id: String,
    /// Its causal role.
    pub role: RuleRole,
    /// Its constitutional evidence class.
    pub evidence: EvidenceClass,
}

/// One independently governed production rule attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernedRuleAttribution {
    /// Exact production rule identity.
    pub rule_id: &'static str,
    /// Role assigned by governance rather than by content.
    pub role: RuleRole,
    /// Evidence class assigned by governance rather than by content.
    pub evidence: EvidenceClass,
    /// Approving authority.
    pub owner: &'static str,
    /// Approval date.
    pub date: &'static str,
    /// Recording architecture decision.
    pub adr: &'static str,
}

const ATTRIBUTION_OWNER: &str = "Director";
const ATTRIBUTION_DATE: &str = "2026-08-23";
const ATTRIBUTION_ADR: &str = "ADR224";
const PROBABILITY_OWNER: &str = "Director";
const PROBABILITY_DATE: &str = "2026-09-01";
const PROBABILITY_ADR: &str = "ADR248";

const fn governed_attribution(rule_id: &'static str, role: RuleRole) -> GovernedRuleAttribution {
    GovernedRuleAttribution {
        rule_id,
        role,
        evidence: EvidenceClass::Derived,
        owner: ATTRIBUTION_OWNER,
        date: ATTRIBUTION_DATE,
        adr: ATTRIBUTION_ADR,
    }
}

/// Governed role/evidence assignments for built-in BSL rules and native compositions.
///
/// Unknown fixture and mod rule IDs remain self-declared. The production
/// corpus sentinel independently proves that no built-in rule is absent from
/// this table.
pub const GOVERNED_RULE_ATTRIBUTIONS: &[GovernedRuleAttribution] = &[
    governed_attribution("community/c00-census-reset", RuleRole::Mechanic),
    governed_attribution("community/c01-member-census", RuleRole::Mechanic),
    governed_attribution("community/c02-org-weight-reset", RuleRole::Mechanic),
    governed_attribution("community/c03f-org-weight-push", RuleRole::Mechanic),
    governed_attribution("community/c03l-org-weight-push", RuleRole::Mechanic),
    governed_attribution("community/c03r-org-weight-push", RuleRole::Mechanic),
    governed_attribution(
        "community/c04-community-contribution-push",
        RuleRole::Mechanic,
    ),
    governed_attribution("community/c05-normalize", RuleRole::Mechanic),
    governed_attribution("community/c06a-floor-dispatch", RuleRole::Mechanic),
    governed_attribution("community/c06b-floor-redistribute", RuleRole::Mechanic),
    governed_attribution("community/c07-contestation", RuleRole::Mechanic),
    governed_attribution("community/c08-dominant-tendency", RuleRole::Mechanic),
    governed_attribution("community/c09-cost-modifier-reset", RuleRole::Mechanic),
    governed_attribution("community/c10-cost-modifier-accumulate", RuleRole::Mechanic),
    governed_attribution("community/c11-state-decay", RuleRole::Mechanic),
    governed_attribution("consciousness/p0-position", RuleRole::Mechanic),
    governed_attribution("consciousness/p1-inbox-reset", RuleRole::Mechanic),
    governed_attribution("consciousness/p2-org-solidarity-push", RuleRole::Mechanic),
    governed_attribution("consciousness/p2-wages-push", RuleRole::Mechanic),
    governed_attribution("consciousness/p3-class-solidarity-push", RuleRole::Mechanic),
    governed_attribution("consciousness/p4-wage-balance", RuleRole::Mechanic),
    governed_attribution("consciousness/p5-agitation", RuleRole::Mechanic),
    governed_attribution("consciousness/p6-route", RuleRole::Mechanic),
    governed_attribution("consciousness/p7-persist-baselines", RuleRole::Mechanic),
    governed_attribution("consciousness/p8-dominant-worldview", RuleRole::Mechanic),
    governed_attribution("consciousness/worldview-mint-probe", RuleRole::Mechanic),
    governed_attribution("control-ratio/c01-prisoner-census", RuleRole::Mechanic),
    governed_attribution("control-ratio/c02-publish-census", RuleRole::Mechanic),
    governed_attribution("control-ratio/c03-crisis", RuleRole::Recognizer),
    governed_attribution("control-ratio/c04-terminal", RuleRole::Recognizer),
    governed_attribution("decomposition/p01-la-census", RuleRole::Mechanic),
    governed_attribution("decomposition/p02-superwage-warning", RuleRole::Mechanic),
    governed_attribution("decomposition/p03-trigger", RuleRole::Mechanic),
    governed_attribution("decomposition/p04-enforcer-intake", RuleRole::Mechanic),
    governed_attribution("decomposition/p05-ip-intake", RuleRole::Mechanic),
    governed_attribution("decomposition/p06-la-deactivate", RuleRole::Mechanic),
    governed_attribution("dispossession/territory-transfer", RuleRole::Mechanic),
    governed_attribution("economics/fundamental-theorem", RuleRole::Mechanic),
    GovernedRuleAttribution {
        rule_id: "g4-workforce-staffing",
        role: RuleRole::Mechanic,
        evidence: EvidenceClass::Designed,
        owner: "Director",
        date: "2026-09-05",
        adr: "ADR255",
    },
    governed_attribution("imperial-rent/r00-tick-reset", RuleRole::Mechanic),
    governed_attribution("imperial-rent/r01-extraction", RuleRole::Mechanic),
    governed_attribution("imperial-rent/r02-extraction-credit", RuleRole::Mechanic),
    governed_attribution("imperial-rent/r03-tribute", RuleRole::Mechanic),
    governed_attribution("imperial-rent/r04-tribute-credit", RuleRole::Mechanic),
    governed_attribution("lifecycle/dpd-circuit", RuleRole::Mechanic),
    governed_attribution("metabolism/biocapacity-update", RuleRole::Mechanic),
    governed_attribution("organization/p0-territory-inbox-reset", RuleRole::Mechanic),
    governed_attribution("organization/p1-presence-attribution", RuleRole::Mechanic),
    governed_attribution("organization/p1-rooted-work", RuleRole::Mechanic),
    governed_attribution("organization/p2-territorial-relay", RuleRole::Mechanic),
    governed_attribution("organization/p3-rooted-capacity-apply", RuleRole::Mechanic),
    governed_attribution("organization/p5-command-response", RuleRole::Mechanic),
    governed_attribution("production/p0-production-total-reset", RuleRole::Mechanic),
    governed_attribution("production/p1-direct-production", RuleRole::Mechanic),
    governed_attribution("production/p2-employed-routing", RuleRole::Mechanic),
    governed_attribution("production/p3-employed-fallback", RuleRole::Mechanic),
    governed_attribution("production/p4-extraction-intensity", RuleRole::Mechanic),
    governed_attribution("solidarity/p0-transmit", RuleRole::Mechanic),
    GovernedRuleAttribution {
        rule_id: "struggle/spark-mechanic",
        role: RuleRole::Mechanic,
        evidence: EvidenceClass::Designed,
        owner: PROBABILITY_OWNER,
        date: PROBABILITY_DATE,
        adr: PROBABILITY_ADR,
    },
    GovernedRuleAttribution {
        rule_id: "struggle/spark-recognizer",
        role: RuleRole::Recognizer,
        evidence: EvidenceClass::Derived,
        owner: PROBABILITY_OWNER,
        date: PROBABILITY_DATE,
        adr: PROBABILITY_ADR,
    },
    governed_attribution("territory/p1-heat-dynamics", RuleRole::Mechanic),
    governed_attribution("territory/p2-eviction-pipeline", RuleRole::Mechanic),
    governed_attribution("territory/p3-spillover", RuleRole::Mechanic),
    governed_attribution("territory/p4-camp-decay", RuleRole::Mechanic),
    governed_attribution("territory/p4-penal-suppression", RuleRole::Mechanic),
    governed_attribution("vitality/subsistence-and-death", RuleRole::Mechanic),
    governed_attribution("vitality/subsistence-clearing", RuleRole::Mechanic),
    governed_attribution("vitality/subsistence-mortality", RuleRole::Mechanic),
];

/// One of the six graph-shape operations BSL can express.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShapeVerb {
    /// `add-node`.
    AddNode,
    /// `remove-node`.
    RemoveNode,
    /// `add-edge`.
    AddEdge,
    /// `remove-edge`.
    RemoveEdge,
    /// `add-hyperedge`.
    AddHyperedge,
    /// `remove-hyperedge`.
    RemoveHyperedge,
}

impl ShapeVerb {
    fn parse(head: &str) -> Option<Self> {
        match head {
            "add-node" => Some(Self::AddNode),
            "remove-node" => Some(Self::RemoveNode),
            "add-edge" => Some(Self::AddEdge),
            "remove-edge" => Some(Self::RemoveEdge),
            "add-hyperedge" => Some(Self::AddHyperedge),
            "remove-hyperedge" => Some(Self::RemoveHyperedge),
            _ => None,
        }
    }
}

/// A canonical, identity-free description of one rule effect.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectSignature {
    /// A node attribute qualified name.
    NodeField(String),
    /// An edge attribute qualified name.
    EdgeField(String),
    /// A hyperedge attribute qualified name.
    HyperedgeField(String),
    /// A full `EventType/MEMBER` event reference.
    Event(String),
    /// A graph-shape operation. Element identities are deliberately absent.
    Shape(ShapeVerb),
}

/// Why a narrowly allowed non-mechanic effect exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowanceKind {
    /// The event announces a recognized endogenous pattern.
    RecognitionEvent,
    /// The field prevents repeated recognition of the same pattern.
    RecognitionLatch,
    /// A declared exogenous burden.
    ExogenousBurden,
    /// A declared exogenous capacity change.
    ExogenousCapacityChange,
    /// A declared exogenous pressure.
    ExogenousPressure,
    /// A declared material effect deferred to the next tick.
    NextWeekIntent,
}

/// A static effect key suitable for a governed constant table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedEffect {
    /// A node field.
    NodeField(&'static str),
    /// An edge field.
    EdgeField(&'static str),
    /// A hyperedge field.
    HyperedgeField(&'static str),
    /// A full event reference.
    Event(&'static str),
}

impl AllowedEffect {
    fn matches(self, actual: &EffectSignature) -> bool {
        match (self, actual) {
            (Self::NodeField(expected), EffectSignature::NodeField(actual))
            | (Self::EdgeField(expected), EffectSignature::EdgeField(actual))
            | (Self::HyperedgeField(expected), EffectSignature::HyperedgeField(actual))
            | (Self::Event(expected), EffectSignature::Event(actual)) => expected == actual,
            _ => false,
        }
    }
}

/// One Director-governed exception to the non-mechanic default-deny policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernedEffectAllowance {
    /// Exact rule identifier.
    pub rule_id: &'static str,
    /// Exact rule role.
    pub role: RuleRole,
    /// Exact effect signature.
    pub effect: AllowedEffect,
    /// The semantic class of the allowance.
    pub kind: AllowanceKind,
    /// Why this effect is causally lawful.
    pub reason: &'static str,
    /// Approving authority.
    pub owner: &'static str,
    /// Approval date.
    pub date: &'static str,
    /// Recording architecture decision.
    pub adr: &'static str,
}

const RECOGNITION_OWNER: &str = "Director";
const RECOGNITION_DATE: &str = "2026-08-23";
const RECOGNITION_ADR: &str = "ADR224";

/// The complete governed allowance table. External-event and intent roles
/// have no rows and therefore cannot produce effects yet.
pub const GOVERNED_EFFECT_ALLOWANCES: &[GovernedEffectAllowance] = &[
    GovernedEffectAllowance {
        rule_id: "control-ratio/c03-crisis",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::Event("EventType/CONTROL_RATIO_CRISIS"),
        kind: AllowanceKind::RecognitionEvent,
        reason: "announce the endogenous over-capacity pattern",
        owner: RECOGNITION_OWNER,
        date: RECOGNITION_DATE,
        adr: RECOGNITION_ADR,
    },
    GovernedEffectAllowance {
        rule_id: "control-ratio/c03-crisis",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::NodeField("institution/control-crisis-emitted"),
        kind: AllowanceKind::RecognitionLatch,
        reason: "prevent duplicate control-ratio recognition events",
        owner: RECOGNITION_OWNER,
        date: RECOGNITION_DATE,
        adr: RECOGNITION_ADR,
    },
    GovernedEffectAllowance {
        rule_id: "control-ratio/c03-crisis",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::NodeField("institution/control-crisis-tick"),
        kind: AllowanceKind::RecognitionLatch,
        reason: "record when the endogenous crisis pattern was recognized",
        owner: RECOGNITION_OWNER,
        date: RECOGNITION_DATE,
        adr: RECOGNITION_ADR,
    },
    GovernedEffectAllowance {
        rule_id: "control-ratio/c04-terminal",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::Event("EventType/TERMINAL_DECISION"),
        kind: AllowanceKind::RecognitionEvent,
        reason: "announce the governed terminal-pattern recognition",
        owner: RECOGNITION_OWNER,
        date: RECOGNITION_DATE,
        adr: RECOGNITION_ADR,
    },
    GovernedEffectAllowance {
        rule_id: "control-ratio/c04-terminal",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::NodeField("institution/terminal-decision-emitted"),
        kind: AllowanceKind::RecognitionLatch,
        reason: "prevent duplicate terminal-pattern recognition events",
        owner: RECOGNITION_OWNER,
        date: RECOGNITION_DATE,
        adr: RECOGNITION_ADR,
    },
    GovernedEffectAllowance {
        rule_id: "struggle/spark-recognizer",
        role: RuleRole::Recognizer,
        effect: AllowedEffect::Event("EventType/EXCESSIVE_FORCE"),
        kind: AllowanceKind::RecognitionEvent,
        reason: "announce the projected excessive-force outcome of the adjacent struggle kernel",
        owner: PROBABILITY_OWNER,
        date: PROBABILITY_DATE,
        adr: PROBABILITY_ADR,
    },
];

/// Director-governed node fields that NO rule — mechanic included — may
/// write (PER-22 ruling D1, 2026-09-02). `territory/county-fips` is governed
/// geography identity: the declared territory→county mapping is frozen at
/// campaign foundation (`babylon_meta.territory_county_map_v1`), so a rule
/// write could rewrite county identity after foundation and leave mechanics
/// and Archive resolution with contradictory geography. The prohibition is
/// name-exact and load-time, and it binds every role: the mechanic's
/// otherwise-unrestricted write surface explicitly stops here.
pub const GOVERNED_WRITE_PROHIBITED_NODE_FIELDS: &[&str] = &["territory/county-fips"];

/// A causal-contract rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// A mandatory valued keyword is absent.
    MissingMetadata { keyword: &'static str },
    /// A mandatory keyword does not carry one symbol value exactly once.
    MalformedMetadata { keyword: &'static str },
    /// A parser-owned closed set does not contain the written value.
    UnknownMetadataValue {
        keyword: &'static str,
        value: String,
    },
    /// Built-in content disagrees with its independently governed assignment.
    GovernedAttributionMismatch {
        rule_id: String,
        expected_role: RuleRole,
        actual_role: RuleRole,
        expected_evidence: EvidenceClass,
        actual_evidence: EvidenceClass,
    },
    /// The input is not a `(rule <qname> ...)` form.
    MalformedRule,
    /// A restricted role tried an effect absent from its exact allowlist.
    UnauthorizedEffect {
        rule_id: String,
        role: RuleRole,
        effect: EffectSignature,
    },
    /// A rule of any role tried to write a governed write-prohibited node
    /// field (PER-22 ruling D1: county identity is frozen at campaign
    /// foundation, so no mechanic or mod rule may rewrite it afterwards).
    GovernedFieldWriteProhibited {
        /// Exact refused rule identifier.
        rule_id: String,
        /// The governed field qualified name.
        field: String,
    },
    /// A caller paired a parsed contract with a different rule AST.
    MismatchedRuleContract {
        ast_contract: RuleContract,
        supplied_contract: RuleContract,
    },
    /// A bounded semantic AST walk exceeded one declared resource limit.
    AstWalkLimit(AstWalkError),
    /// A write record was attributed to a different rule.
    MismatchedWriteAttribution { expected: String, actual: String },
    /// A write record's producer ordinal is not the next dense ordinal.
    MismatchedWriteOrdinal { expected: u32, actual: u32 },
    /// An observed event used an empty member or a non-`EventType` namespace.
    MalformedEventType { value: String },
    /// More effects were presented than a `u32` receipt ordinal can hold.
    ReceiptOrdinalOverflow,
}

impl ContractError {
    /// The governed diagnostic code, where the contract assigns one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::UnknownMetadataValue { .. } => Some("E-PARSE-015"),
            Self::UnauthorizedEffect { .. } => Some("E-LOAD-060"),
            Self::MissingMetadata { .. }
            | Self::MalformedMetadata { .. }
            | Self::GovernedAttributionMismatch { .. }
            | Self::MalformedRule
            | Self::GovernedFieldWriteProhibited { .. }
            | Self::MismatchedRuleContract { .. }
            | Self::AstWalkLimit(_)
            | Self::MismatchedWriteAttribution { .. }
            | Self::MismatchedWriteOrdinal { .. }
            | Self::MalformedEventType { .. }
            | Self::ReceiptOrdinalOverflow => None,
        }
    }
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMetadata { keyword } => {
                write!(formatter, ":{keyword} is mandatory on every rule")
            }
            Self::MalformedMetadata { keyword } => {
                write!(formatter, ":{keyword} must occur once with a symbol value")
            }
            Self::UnknownMetadataValue { keyword, value } => write!(
                formatter,
                "E-PARSE-015: :{keyword} has unknown closed-set value {value}"
            ),
            Self::GovernedAttributionMismatch {
                rule_id,
                expected_role,
                actual_role,
                expected_evidence,
                actual_evidence,
            } => write!(
                formatter,
                "{rule_id} declares {actual_role:?}/{actual_evidence:?}, but ADR224 pins \
                 {expected_role:?}/{expected_evidence:?}"
            ),
            Self::MalformedRule => write!(formatter, "expected (rule <qname> ...)"),
            Self::UnauthorizedEffect {
                rule_id,
                role,
                effect,
            } => write!(
                formatter,
                "E-LOAD-060: {rule_id} ({role:?}) is not allowed to perform {effect:?}"
            ),
            Self::GovernedFieldWriteProhibited { rule_id, field } => write!(
                formatter,
                "{rule_id} is not allowed to write {field}: the field is governed geography \
                 identity frozen at campaign foundation (PER-22 ruling D1), and every role — \
                 mechanic included — is refused a write to it at load"
            ),
            Self::MismatchedRuleContract {
                ast_contract,
                supplied_contract,
            } => write!(
                formatter,
                "rule AST declares {ast_contract:?}, but the authorizer received \
                 {supplied_contract:?}"
            ),
            Self::AstWalkLimit(error) => write!(formatter, "{error}"),
            Self::MismatchedWriteAttribution { expected, actual } => write!(
                formatter,
                "write attribution names {actual}, expected {expected}"
            ),
            Self::MismatchedWriteOrdinal { expected, actual } => write!(
                formatter,
                "write ordinal is {actual}, expected dense ordinal {expected}"
            ),
            Self::MalformedEventType { value } => write!(
                formatter,
                "observed event {value:?} is neither a bare member nor EventType/<member>"
            ),
            Self::ReceiptOrdinalOverflow => {
                write!(formatter, "receipt sequence exceeds the u32 ordinal domain")
            }
        }
    }
}

impl std::error::Error for ContractError {}

fn rule_items(rule: &SExpr) -> Result<(&str, &[SExpr]), ContractError> {
    let SExpr::List(items) = rule else {
        return Err(ContractError::MalformedRule);
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::QName(rule_id)), rest @ ..] =
        items.as_slice()
    else {
        return Err(ContractError::MalformedRule);
    };
    if head != "rule" {
        return Err(ContractError::MalformedRule);
    }
    Ok((rule_id, rest))
}

fn metadata_symbol<'a>(
    items: &'a [SExpr],
    keyword: &'static str,
) -> Result<&'a str, ContractError> {
    let positions = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| match item {
            SExpr::Atom(Atom::Keyword(found)) if found == keyword => Some(index),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [position] = positions.as_slice() else {
        return if positions.is_empty() {
            Err(ContractError::MissingMetadata { keyword })
        } else {
            Err(ContractError::MalformedMetadata { keyword })
        };
    };
    match items.get(position + 1) {
        Some(SExpr::Atom(Atom::Symbol(value))) => Ok(value),
        _ => Err(ContractError::MalformedMetadata { keyword }),
    }
}

/// Parse the mandatory parser-owned `:role` and `:evidence` values.
///
/// # Errors
///
/// [`ContractError`] when the form or either mandatory option is invalid.
pub fn parse_rule_contract(form: &SExpr) -> Result<RuleContract, ContractError> {
    let (rule_id, items) = rule_items(form)?;
    let role_value = metadata_symbol(items, "role")?;
    let evidence_value = metadata_symbol(items, "evidence")?;
    let role = RuleRole::parse(role_value).ok_or_else(|| ContractError::UnknownMetadataValue {
        keyword: "role",
        value: role_value.to_owned(),
    })?;
    let evidence = EvidenceClass::parse(evidence_value).ok_or_else(|| {
        ContractError::UnknownMetadataValue {
            keyword: "evidence",
            value: evidence_value.to_owned(),
        }
    })?;
    let contract = RuleContract {
        rule_id: rule_id.to_owned(),
        role,
        evidence,
    };
    Ok(contract)
}

/// Enforce the independent attribution manifest after parse and type checks.
///
/// # Errors
///
/// [`ContractError::GovernedAttributionMismatch`] when built-in content
/// attempts to relabel its governed role or evidence class.
pub fn validate_governed_attribution(contract: &RuleContract) -> Result<(), ContractError> {
    let Some(expected) = GOVERNED_RULE_ATTRIBUTIONS
        .iter()
        .find(|row| row.rule_id == contract.rule_id.as_str())
    else {
        return Ok(());
    };
    if expected.role == contract.role && expected.evidence == contract.evidence {
        return Ok(());
    }
    Err(ContractError::GovernedAttributionMismatch {
        rule_id: contract.rule_id.clone(),
        expected_role: expected.role,
        actual_role: contract.role,
        expected_evidence: expected.evidence,
        actual_evidence: contract.evidence,
    })
}

fn field_effect(items: &[SExpr], kind: &str) -> Option<EffectSignature> {
    let field = match kind {
        "update-node" | "update-edge" | "update-hyperedge" => items.get(2),
        _ => None,
    };
    let Some(SExpr::Atom(Atom::QName(field))) = field else {
        return None;
    };
    match kind {
        "update-node" => Some(EffectSignature::NodeField(field.clone())),
        "update-edge" => Some(EffectSignature::EdgeField(field.clone())),
        "update-hyperedge" => Some(EffectSignature::HyperedgeField(field.clone())),
        _ => None,
    }
}

fn event_effect(items: &[SExpr]) -> Option<EffectSignature> {
    let Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) = items.get(1) else {
        return None;
    };
    Some(EffectSignature::Event(format!("{enum_type}/{member}")))
}

fn push_effect_children<'a>(
    stack: &mut Vec<(&'a SExpr, usize)>,
    items: &'a [SExpr],
    depth: usize,
    limits: AstWalkLimits,
) -> Result<(), AstWalkError> {
    let children = items.get(1..).unwrap_or_default();
    if children.is_empty() {
        return Ok(());
    }
    let child_depth = checked_child_depth(depth, limits, CAUSAL_WALKER)?;
    check_stack_capacity(stack.len(), children.len(), limits, CAUSAL_WALKER)?;
    for child in children.iter().rev() {
        stack.push((child, child_depth));
    }
    Ok(())
}

fn emit_payload_value_count(items: &[SExpr], limits: AstWalkLimits) -> Result<usize, AstWalkError> {
    let mut count = 0_usize;
    for payload_item in items.iter().skip(2) {
        let SExpr::List(pair) = payload_item else {
            continue;
        };
        count = count
            .checked_add(pair.len().saturating_sub(1))
            .ok_or_else(|| ast_walk_error(CAUSAL_WALKER, AstWalkLimit::Stack, limits.stack))?;
    }
    Ok(count)
}

fn push_emit_payload_values<'a>(
    stack: &mut Vec<(&'a SExpr, usize)>,
    items: &'a [SExpr],
    depth: usize,
    limits: AstWalkLimits,
) -> Result<(), AstWalkError> {
    let value_count = emit_payload_value_count(items, limits)?;
    if value_count == 0 {
        return Ok(());
    }
    let pair_depth = checked_child_depth(depth, limits, CAUSAL_WALKER)?;
    let value_depth = checked_child_depth(pair_depth, limits, CAUSAL_WALKER)?;
    check_stack_capacity(stack.len(), value_count, limits, CAUSAL_WALKER)?;
    for payload_item in items.iter().skip(2).rev() {
        let SExpr::List(pair) = payload_item else {
            continue;
        };
        for value in pair.iter().skip(1).rev() {
            stack.push((value, value_depth));
        }
    }
    Ok(())
}

fn walk_effects_with_limits(
    rule: &SExpr,
    limits: AstWalkLimits,
) -> Result<Vec<EffectSignature>, AstWalkError> {
    validate_ast_walk_bounds(rule, limits, CAUSAL_WALKER)?;
    let mut effects = Vec::new();
    let mut stack = vec![(rule, 0_usize)];
    for visited in 0..MAX_AST_WALK_NODES {
        if visited >= limits.nodes() {
            break;
        }
        let Some((expr, depth)) = stack.pop() else {
            return Ok(effects);
        };
        let SExpr::List(items) = expr else { continue };
        let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
            push_effect_children(&mut stack, items, depth, limits)?;
            continue;
        };
        if head == "emit" {
            if let Some(effect) = event_effect(items) {
                effects.push(effect);
                push_emit_payload_values(&mut stack, items, depth, limits)?;
            } else {
                push_effect_children(&mut stack, items, depth, limits)?;
            }
            continue;
        }
        if let Some(effect) = field_effect(items, head) {
            effects.push(effect);
        } else if let Some(verb) = ShapeVerb::parse(head) {
            effects.push(EffectSignature::Shape(verb));
        }
        push_effect_children(&mut stack, items, depth, limits)?;
    }
    if stack.is_empty() {
        Ok(effects)
    } else {
        Err(ast_walk_error(
            CAUSAL_WALKER,
            AstWalkLimit::Nodes,
            limits.nodes(),
        ))
    }
}

/// Return every possible effect in depth-first source order. Guards and loop
/// bodies are inspected irrespective of whether they execute at runtime.
///
/// # Errors
/// A typed [`ContractError::AstWalkLimit`] when the tree exceeds a declared
/// semantic-walk resource boundary.
pub fn effect_footprint(rule: &SExpr) -> Result<Vec<EffectSignature>, ContractError> {
    walk_effects_with_limits(rule, AST_WALK_LIMITS).map_err(ContractError::AstWalkLimit)
}

fn is_allowed(contract: &RuleContract, effect: &EffectSignature) -> bool {
    GOVERNED_EFFECT_ALLOWANCES.iter().any(|row| {
        row.rule_id == contract.rule_id && row.role == contract.role && row.effect.matches(effect)
    })
}

fn authorize_effect(
    contract: &RuleContract,
    effect: &EffectSignature,
) -> Result<(), ContractError> {
    // The governed geography-identity prohibition binds EVERY role and
    // therefore runs before the mechanic's otherwise-unrestricted early
    // return: no rule may write a field the campaign foundation freezes.
    if let EffectSignature::NodeField(field) = effect {
        if GOVERNED_WRITE_PROHIBITED_NODE_FIELDS.contains(&field.as_str()) {
            return Err(ContractError::GovernedFieldWriteProhibited {
                rule_id: contract.rule_id.clone(),
                field: field.clone(),
            });
        }
    }
    if contract.role == RuleRole::Mechanic {
        return Ok(());
    }
    if matches!(effect, EffectSignature::Shape(_)) || !is_allowed(contract, effect) {
        return Err(ContractError::UnauthorizedEffect {
            rule_id: contract.rule_id.clone(),
            role: contract.role,
            effect: effect.clone(),
        });
    }
    Ok(())
}

/// Enforce a previously parsed contract against every possible rule effect.
///
/// This split entry point lets the composed loader authorize a structurally
/// well-formed rule after every parse- and type-class gate, before later
/// load/link checks.
///
/// # Errors
///
/// [`ContractError::MalformedRule`] when `rule` is not a rule form;
/// [`ContractError::MismatchedRuleContract`] when the supplied contract names
/// another rule; [`ContractError::AstWalkLimit`] when bounded inspection
/// refuses the tree; or `E-LOAD-060` for the first unauthorized effect in
/// source order.
pub fn authorize_rule_effects(rule: &SExpr, contract: &RuleContract) -> Result<(), ContractError> {
    let ast_contract = parse_rule_contract(rule)?;
    if ast_contract != *contract {
        return Err(ContractError::MismatchedRuleContract {
            ast_contract,
            supplied_contract: contract.clone(),
        });
    }
    for effect in effect_footprint(rule)? {
        authorize_effect(contract, &effect)?;
    }
    Ok(())
}

/// Parse and enforce one rule's role-sensitive effect contract.
///
/// Mechanics retain the ordinary BSL write surface. Every other role is
/// default-deny, and graph-shape effects are always prohibited.
///
/// # Errors
///
/// Metadata errors from [`parse_rule_contract`], an independent
/// [`ContractError::GovernedAttributionMismatch`], a bounded
/// [`ContractError::AstWalkLimit`], or `E-LOAD-060` for the first unauthorized
/// effect in source order.
pub fn check_rule_contract(rule: &SExpr) -> Result<RuleContract, ContractError> {
    let contract = parse_rule_contract(rule)?;
    validate_governed_attribution(&contract)?;
    authorize_rule_effects(rule, &contract)?;
    Ok(contract)
}

/// One public causal audit receipt. It contains no graph-element identity or
/// written value: those remain in the authoritative state/write ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReceipt {
    /// Exact producing rule.
    pub rule_id: String,
    /// Producing role.
    pub role: RuleRole,
    /// Producing rule's evidence class.
    pub evidence: EvidenceClass,
    /// Continuous position: collection-boundary events, then apply writes.
    pub ordinal: u32,
    /// Canonical identity-free effect.
    pub effect: EffectSignature,
}

pub(crate) fn canonical_event_type(event_type: &str) -> Result<String, ContractError> {
    if let Some(member) = event_type.strip_prefix("EventType/") {
        if !member.is_empty() && !member.contains('/') {
            return Ok(event_type.to_owned());
        }
    } else if !event_type.is_empty() && !event_type.contains('/') {
        return Ok(format!("EventType/{event_type}"));
    }
    Err(ContractError::MalformedEventType {
        value: event_type.to_owned(),
    })
}

fn write_effect(write: &Write) -> EffectSignature {
    match write {
        Write::NodeAttribute { field, .. } | Write::NodeCurrencyAttribute { field, .. } => {
            EffectSignature::NodeField(field.clone())
        }
        Write::EdgeAttribute { field, .. } => EffectSignature::EdgeField(field.clone()),
        Write::HyperedgeAttribute { field, .. } => EffectSignature::HyperedgeField(field.clone()),
    }
}

fn receipt(
    contract: &RuleContract,
    ordinal: usize,
    effect: EffectSignature,
) -> Result<AuditReceipt, ContractError> {
    let ordinal = u32::try_from(ordinal).map_err(|_| ContractError::ReceiptOrdinalOverflow)?;
    Ok(AuditReceipt {
        rule_id: contract.rule_id.clone(),
        role: contract.role,
        evidence: contract.evidence,
        ordinal,
        effect,
    })
}

fn checked_receipt_capacity(event_count: u64, write_count: u64) -> Result<usize, ContractError> {
    let total = event_count
        .checked_add(write_count)
        .ok_or(ContractError::ReceiptOrdinalOverflow)?;
    if let Some(last_ordinal) = total.checked_sub(1) {
        u32::try_from(last_ordinal).map_err(|_| ContractError::ReceiptOrdinalOverflow)?;
    }
    usize::try_from(total).map_err(|_| ContractError::ReceiptOrdinalOverflow)
}

/// Reduce successful collection-boundary events and apply-boundary writes to
/// one deterministic, continuous receipt sequence.
///
/// # Errors
///
/// [`ContractError::GovernedAttributionMismatch`] for a forged built-in
/// contract; [`ContractError::MismatchedWriteOrdinal`] or
/// [`ContractError::MismatchedWriteAttribution`] for a malformed write log;
/// [`ContractError::MalformedEventType`] for a bad event namespace;
/// [`ContractError::UnauthorizedEffect`] for an unexpected restricted-role
/// runtime effect; or [`ContractError::ReceiptOrdinalOverflow`] when the
/// combined sequence cannot be represented. No error is silently normalized.
pub fn reduce_audit_receipts(
    contract: &RuleContract,
    emitted_event_types: &[String],
    writes: &[WriteRecord],
) -> Result<Vec<AuditReceipt>, ContractError> {
    validate_governed_attribution(contract)?;
    let event_count = u64::try_from(emitted_event_types.len())
        .map_err(|_| ContractError::ReceiptOrdinalOverflow)?;
    let write_count =
        u64::try_from(writes.len()).map_err(|_| ContractError::ReceiptOrdinalOverflow)?;
    let capacity = checked_receipt_capacity(event_count, write_count)?;
    for (expected, record) in writes.iter().enumerate() {
        let expected =
            u32::try_from(expected).map_err(|_| ContractError::ReceiptOrdinalOverflow)?;
        if record.ordinal != expected {
            return Err(ContractError::MismatchedWriteOrdinal {
                expected,
                actual: record.ordinal,
            });
        }
        if record.rule != contract.rule_id {
            return Err(ContractError::MismatchedWriteAttribution {
                expected: contract.rule_id.clone(),
                actual: record.rule.clone(),
            });
        }
    }
    let mut receipts = Vec::with_capacity(capacity);
    for (ordinal, event_type) in emitted_event_types.iter().enumerate() {
        let effect = EffectSignature::Event(canonical_event_type(event_type)?);
        authorize_effect(contract, &effect)?;
        receipts.push(receipt(contract, ordinal, effect)?);
    }
    for (write_index, record) in writes.iter().enumerate() {
        let ordinal = emitted_event_types
            .len()
            .checked_add(write_index)
            .ok_or(ContractError::ReceiptOrdinalOverflow)?;
        let effect = write_effect(&record.write);
        authorize_effect(contract, &effect)?;
        receipts.push(receipt(contract, ordinal, effect)?);
    }
    Ok(receipts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::BindingVocabulary;
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::reader::read;
    use crate::rule_pipeline::{load_rule, LoadContext, LoadError};
    use crate::typecheck::TypeEnv;
    use crate::write_log::{Write, WriteRecord};
    use babylon_graph::substrate::NodeId;
    use std::collections::{HashMap, HashSet};

    fn rule(source: &str) -> SExpr {
        read(source).expect("test rule must lex and parse").0
    }

    fn wrapped(options: &str, effects: &str) -> SExpr {
        rule(&format!(
            "(rule demo/probe {options} :material-basis \"material relation\" :fuel 64 \
             (bindings) (effects {effects}))"
        ))
    }

    fn load_ctx() -> LoadContext<'static> {
        LoadContext {
            vocabulary: Box::leak(Box::new(BindingVocabulary {
                fields: HashSet::new(),
                consts: HashSet::new(),
                probability_consts: HashSet::new(),
                metrics: HashSet::new(),
            })),
            types: Box::leak(Box::new(TypeEnv {
                fields: HashMap::new(),
                exemptions: &[],
            })),
            enums: Box::leak(Box::new(crate::types::EnumRegistry::default())),
            const_values: Box::leak(Box::new(HashMap::new())),
            ceilings: Box::leak(Box::new(CardinalityCeilings::new(
                HashMap::new(),
                HashMap::new(),
            ))),
            intrinsics: Box::leak(Box::new(IntrinsicCosts::default())),
            systems: Box::leak(Box::new(HashSet::from(["demo".to_owned()]))),
            vocabulary_registry: None,
            rule_file: "causal_contract.rs",
        }
    }

    #[test]
    fn composed_load_refuses_effect_nesting_beyond_the_static_depth_bound() {
        let mut effect = "(emit EventType/NOTICE)".to_owned();
        for _ in 0..(MAX_AST_WALK_DEPTH + 64) {
            effect = format!("(guard #t {effect})");
        }
        let source = format!(
            "(rule demo/deep :role mechanic :evidence derived \
             :material-basis \"depth refusal\" :fuel 1000000 \
             (bindings) (effects {effect}))"
        );

        let error = load_rule(&source, &load_ctx()).expect_err("hostile nesting must refuse");
        assert_eq!(error.spec_code(), None);
        assert!(matches!(
            error,
            LoadError::Causal(ContractError::AstWalkLimit(AstWalkError {
                analyzer: "rule load preflight",
                limit: AstWalkLimit::Depth,
                maximum: MAX_AST_WALK_DEPTH,
            }))
        ));
    }

    #[test]
    fn causal_walker_reports_each_bound_without_truncation() {
        let ast = rule("(root (child) sibling)");
        for (limits, expected) in [
            (
                AstWalkLimits::new(0, 16, 16),
                AstWalkError::new("causal effect footprint", AstWalkLimit::Depth, 0),
            ),
            (
                AstWalkLimits::new(16, 16, 1),
                AstWalkError::new("causal effect footprint", AstWalkLimit::Stack, 1),
            ),
            (
                AstWalkLimits::new(16, 1, 16),
                AstWalkError::new("causal effect footprint", AstWalkLimit::Nodes, 1),
            ),
        ] {
            assert_eq!(
                walk_effects_with_limits(&ast, limits).unwrap_err(),
                expected
            );
        }
    }

    #[test]
    fn missing_role_is_an_uncoded_mandatory_metadata_error() {
        let ast = wrapped(":evidence derived", "(emit EventType/NOTICE)");
        let error = parse_rule_contract(&ast).unwrap_err();
        assert_eq!(error.spec_code(), None);
        assert_eq!(error, ContractError::MissingMetadata { keyword: "role" });
    }

    #[test]
    fn missing_evidence_is_an_uncoded_mandatory_metadata_error() {
        let ast = wrapped(":role mechanic", "(emit EventType/NOTICE)");
        let error = parse_rule_contract(&ast).unwrap_err();
        assert_eq!(error.spec_code(), None);
        assert_eq!(
            error,
            ContractError::MissingMetadata {
                keyword: "evidence"
            }
        );
    }

    #[test]
    fn unknown_closed_role_and_evidence_values_are_e_parse_015() {
        for (options, keyword, value) in [
            (":role shock :evidence derived", "role", "shock"),
            (":role mechanic :evidence asserted", "evidence", "asserted"),
        ] {
            let error =
                parse_rule_contract(&wrapped(options, "(emit EventType/NOTICE)")).unwrap_err();
            assert_eq!(error.spec_code(), Some("E-PARSE-015"));
            assert_eq!(
                error,
                ContractError::UnknownMetadataValue {
                    keyword,
                    value: value.to_owned(),
                }
            );
        }
    }

    #[test]
    fn all_four_evidence_classes_parse_as_closed_values() {
        for (source, expected) in [
            ("observed", EvidenceClass::Observed),
            ("derived", EvidenceClass::Derived),
            ("calibrated", EvidenceClass::Calibrated),
            ("designed", EvidenceClass::Designed),
        ] {
            let contract = parse_rule_contract(&wrapped(
                &format!(":role mechanic :evidence {source}"),
                "(emit EventType/NOTICE)",
            ))
            .unwrap();
            assert_eq!(contract.evidence, expected);
        }
    }

    #[test]
    fn external_event_cannot_hide_a_write_or_event_in_untaken_nested_forms() {
        let ast = wrapped(
            ":role external-event :evidence observed",
            "(guard #f (for-each it (nodes NodeType/TERRITORY) \
               (update-node it territory/public-health-pressure (set 1)) \
               (emit EventType/TERMINAL_DECISION)))",
        );
        let error = check_rule_contract(&ast).unwrap_err();
        assert_eq!(error.spec_code(), Some("E-LOAD-060"));
        assert!(matches!(
            error,
            ContractError::UnauthorizedEffect {
                effect: EffectSignature::NodeField(ref field),
                ..
            } if field == "territory/public-health-pressure"
        ));
    }

    #[test]
    fn an_external_pressure_looking_qname_is_still_default_deny() {
        let ast = wrapped(
            ":role external-event :evidence designed",
            "(update-node self territory/pressure (set 1))",
        );
        assert!(matches!(
            check_rule_contract(&ast),
            Err(ContractError::UnauthorizedEffect {
                effect: EffectSignature::NodeField(field),
                ..
            }) if field == "territory/pressure"
        ));
    }

    #[test]
    fn intent_is_default_deny_too() {
        let ast = wrapped(
            ":role intent :evidence designed",
            "(emit EventType/NEXT_WEEK_INTENT)",
        );
        assert!(matches!(
            check_rule_contract(&ast),
            Err(ContractError::UnauthorizedEffect {
                effect: EffectSignature::Event(event),
                ..
            }) if event == "EventType/NEXT_WEEK_INTENT"
        ));
    }

    #[test]
    fn mechanic_attrition_remains_legal() {
        let ast = wrapped(
            ":role mechanic :evidence derived",
            "(update-node self social-class/population (set 0)) \
             (emit EventType/POPULATION_ATTRITION)",
        );
        let contract = check_rule_contract(&ast).unwrap();
        assert_eq!(contract.role, RuleRole::Mechanic);
    }

    #[test]
    fn split_parse_and_authorization_agree_with_the_composed_check() {
        let ast = wrapped(
            ":role mechanic :evidence derived",
            "(update-node self social-class/population (set 0))",
        );
        let parsed = parse_rule_contract(&ast).unwrap();
        validate_governed_attribution(&parsed).unwrap();
        authorize_rule_effects(&ast, &parsed).unwrap();
        assert_eq!(check_rule_contract(&ast).unwrap(), parsed);
    }

    #[test]
    fn public_authorizer_refuses_a_contract_from_another_rule() {
        let restricted_ast = rule(
            "(rule demo/restricted :role intent :evidence designed \
             :material-basis \"intent\" :fuel 8 (bindings) \
             (effects (emit EventType/NEXT_WEEK_INTENT)))",
        );
        let mechanic_contract = RuleContract {
            rule_id: "demo/mechanic".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };

        assert_eq!(
            authorize_rule_effects(&restricted_ast, &mechanic_contract).unwrap_err(),
            ContractError::MismatchedRuleContract {
                ast_contract: RuleContract {
                    rule_id: "demo/restricted".to_owned(),
                    role: RuleRole::Intent,
                    evidence: EvidenceClass::Designed,
                },
                supplied_contract: mechanic_contract,
            }
        );
    }

    #[test]
    fn current_recognizer_allowances_are_exact() {
        let c03 = rule(
            "(rule control-ratio/c03-crisis :role recognizer :evidence derived \
             :material-basis \"recognized crisis\" :fuel 64 (bindings) (effects \
             (emit EventType/CONTROL_RATIO_CRISIS) \
             (update-node self institution/control-crisis-emitted (set 1)) \
             (update-node self institution/control-crisis-tick (set 7))))",
        );
        let c04 = rule(
            "(rule control-ratio/c04-terminal :role recognizer :evidence derived \
             :material-basis \"recognized terminal pattern\" :fuel 64 (bindings) (effects \
             (emit EventType/TERMINAL_DECISION) \
             (update-node self institution/terminal-decision-emitted (set 1))))",
        );
        let spark = rule(
            "(rule struggle/spark-recognizer :role recognizer :evidence derived \
             :material-basis \"recognized excessive force\" :fuel 64 (bindings) \
             (effects (emit EventType/EXCESSIVE_FORCE)))",
        );
        assert!(check_rule_contract(&c03).is_ok());
        assert!(check_rule_contract(&c04).is_ok());
        assert!(check_rule_contract(&spark).is_ok());
        let spark_allowances: Vec<AllowedEffect> = GOVERNED_EFFECT_ALLOWANCES
            .iter()
            .filter(|row| row.rule_id == "struggle/spark-recognizer")
            .map(|row| row.effect)
            .collect();
        assert_eq!(
            spark_allowances,
            vec![AllowedEffect::Event("EventType/EXCESSIVE_FORCE")]
        );
    }

    #[test]
    fn governed_recognizers_cannot_escalate_themselves_to_mechanic() {
        for rule_id in [
            "control-ratio/c03-crisis",
            "control-ratio/c04-terminal",
            "struggle/spark-recognizer",
        ] {
            let ast = rule(&format!(
                "(rule {rule_id} :role mechanic :evidence derived \
                 :material-basis \"role escalation probe\" :fuel 8 (bindings) (effects))"
            ));
            let contract = parse_rule_contract(&ast).unwrap();
            assert_eq!(
                validate_governed_attribution(&contract).unwrap_err(),
                ContractError::GovernedAttributionMismatch {
                    rule_id: rule_id.to_owned(),
                    expected_role: RuleRole::Recognizer,
                    actual_role: RuleRole::Mechanic,
                    expected_evidence: EvidenceClass::Derived,
                    actual_evidence: EvidenceClass::Derived,
                }
            );
        }
    }

    #[test]
    fn governed_evidence_cannot_be_relabelled_by_content() {
        let ast = rule(
            "(rule economics/fundamental-theorem :role mechanic :evidence designed \
             :material-basis \"evidence relabelling probe\" :fuel 8 (bindings) (effects))",
        );
        let contract = parse_rule_contract(&ast).unwrap();
        assert_eq!(
            validate_governed_attribution(&contract).unwrap_err(),
            ContractError::GovernedAttributionMismatch {
                rule_id: "economics/fundamental-theorem".to_owned(),
                expected_role: RuleRole::Mechanic,
                actual_role: RuleRole::Mechanic,
                expected_evidence: EvidenceClass::Derived,
                actual_evidence: EvidenceClass::Designed,
            }
        );
    }

    #[test]
    fn native_staffing_receipts_reject_forged_role_or_evidence() {
        let rule_id = "g4-workforce-staffing";
        let admitted = RuleContract {
            rule_id: rule_id.to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Designed,
        };
        assert_eq!(reduce_audit_receipts(&admitted, &[], &[]), Ok(vec![]));
        for (role, evidence) in [
            (RuleRole::Recognizer, EvidenceClass::Designed),
            (RuleRole::ExternalEvent, EvidenceClass::Designed),
            (RuleRole::Intent, EvidenceClass::Designed),
            (RuleRole::Mechanic, EvidenceClass::Observed),
            (RuleRole::Mechanic, EvidenceClass::Derived),
            (RuleRole::Mechanic, EvidenceClass::Calibrated),
        ] {
            let forged = RuleContract {
                rule_id: rule_id.to_owned(),
                role,
                evidence,
            };
            assert_eq!(
                reduce_audit_receipts(&forged, &[], &[]).unwrap_err(),
                ContractError::GovernedAttributionMismatch {
                    rule_id: rule_id.to_owned(),
                    expected_role: RuleRole::Mechanic,
                    actual_role: role,
                    expected_evidence: EvidenceClass::Designed,
                    actual_evidence: evidence,
                }
            );
        }
    }

    #[test]
    fn effect_allowances_name_matching_pinned_restricted_assignments() {
        for allowance in GOVERNED_EFFECT_ALLOWANCES {
            let attribution = GOVERNED_RULE_ATTRIBUTIONS
                .iter()
                .find(|row| row.rule_id == allowance.rule_id)
                .expect("every allowance rule must have a governed attribution");
            assert_eq!(attribution.role, allowance.role);
            assert_ne!(attribution.role, RuleRole::Mechanic);
            assert_eq!(attribution.evidence, EvidenceClass::Derived);
        }
    }

    #[test]
    fn recognizer_allowance_does_not_extend_to_a_nearby_rule_or_field() {
        for ast in [
            rule(
                "(rule control-ratio/c03-crisis-copy :role recognizer :evidence derived \
                 :material-basis \"copy\" :fuel 64 (bindings) \
                 (effects (emit EventType/CONTROL_RATIO_CRISIS)))",
            ),
            rule(
                "(rule control-ratio/c03-crisis :role recognizer :evidence derived \
                 :material-basis \"near field\" :fuel 64 (bindings) \
                 (effects (update-node self institution/control-crisis-known (set 1))))",
            ),
        ] {
            assert_eq!(
                check_rule_contract(&ast).unwrap_err().spec_code(),
                Some("E-LOAD-060")
            );
        }
    }

    #[test]
    fn every_shape_verb_is_refused_for_every_restricted_role() {
        for role in ["recognizer", "external-event", "intent"] {
            for effect in [
                "(add-node NodeType/TERRITORY county)",
                "(remove-node self NodeType/TERRITORY)",
                "(add-edge EdgeType/ADJACENCY self self :strength 1.0c)",
                "(remove-edge EdgeType/ADJACENCY self self)",
                "(add-hyperedge HyperedgeType/COMMUNITY group (self))",
                "(remove-hyperedge group HyperedgeType/COMMUNITY)",
            ] {
                let ast = wrapped(&format!(":role {role} :evidence designed"), effect);
                let error = check_rule_contract(&ast).unwrap_err();
                assert_eq!(error.spec_code(), Some("E-LOAD-060"));
                assert!(matches!(
                    error,
                    ContractError::UnauthorizedEffect {
                        effect: EffectSignature::Shape(_),
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn footprint_includes_edge_and_hyperedge_writes_under_nested_forms() {
        let ast = wrapped(
            ":role mechanic :evidence derived",
            "(guard #f
               (update-edge (edge-between EdgeType/SOLIDARITY self self)
                 solidarity/strength (set 1.0i))
               (for-each it (hyperedges HyperedgeType/COMMUNITY)
                 (update-hyperedge it community/heat (set 0.5i))))",
        );
        assert_eq!(
            effect_footprint(&ast).unwrap(),
            vec![
                EffectSignature::EdgeField("solidarity/strength".to_owned()),
                EffectSignature::HyperedgeField("community/heat".to_owned()),
            ]
        );
    }

    #[test]
    fn emit_payload_labels_cannot_fabricate_effects() {
        let ast = rule(
            "(rule control-ratio/c03-crisis :role recognizer :evidence derived \
             :material-basis \"payload label probe\" :fuel 8 (bindings) (effects \
             (emit EventType/CONTROL_RATIO_CRISIS (add-node 1) (emit 2))))",
        );
        assert_eq!(
            effect_footprint(&ast).unwrap(),
            vec![EffectSignature::Event(
                "EventType/CONTROL_RATIO_CRISIS".to_owned()
            )]
        );
        assert!(check_rule_contract(&ast).is_ok());
    }

    #[test]
    fn a_genuine_forbidden_effect_inside_an_emit_payload_value_is_still_found() {
        let ast = rule(
            "(rule control-ratio/c03-crisis :role recognizer :evidence derived \
             :material-basis \"payload value probe\" :fuel 8 (bindings) (effects \
             (emit EventType/CONTROL_RATIO_CRISIS \
               (payload (add-node NodeType/TERRITORY county)))))",
        );
        assert_eq!(
            effect_footprint(&ast).unwrap(),
            vec![
                EffectSignature::Event("EventType/CONTROL_RATIO_CRISIS".to_owned()),
                EffectSignature::Shape(ShapeVerb::AddNode),
            ]
        );
        assert!(matches!(
            check_rule_contract(&ast),
            Err(ContractError::UnauthorizedEffect {
                effect: EffectSignature::Shape(ShapeVerb::AddNode),
                ..
            })
        ));
    }

    #[test]
    fn governed_allowance_rows_carry_the_required_provenance() {
        assert_eq!(GOVERNED_EFFECT_ALLOWANCES.len(), 6);
        for row in GOVERNED_EFFECT_ALLOWANCES {
            assert!(!row.reason.is_empty());
            assert_eq!(row.owner, "Director");
            if row.rule_id.starts_with("struggle/") {
                assert_eq!(row.date, "2026-09-01");
                assert_eq!(row.adr, "ADR248");
            } else {
                assert_eq!(row.date, "2026-08-23");
                assert_eq!(row.adr, "ADR224");
            }
        }
    }

    #[test]
    fn governed_attribution_rows_carry_the_required_provenance() {
        assert_eq!(GOVERNED_RULE_ATTRIBUTIONS.len(), 68);
        assert_eq!(
            GOVERNED_RULE_ATTRIBUTIONS
                .iter()
                .filter(|row| row.role == RuleRole::Mechanic)
                .count(),
            65
        );
        assert_eq!(
            GOVERNED_RULE_ATTRIBUTIONS
                .iter()
                .filter(|row| row.role == RuleRole::Recognizer)
                .count(),
            3
        );
        assert!(GOVERNED_RULE_ATTRIBUTIONS
            .windows(2)
            .all(|rows| rows[0].rule_id < rows[1].rule_id));
        for row in GOVERNED_RULE_ATTRIBUTIONS {
            assert_eq!(row.owner, "Director");
            if row.rule_id == "g4-workforce-staffing" {
                assert_eq!(row.evidence, EvidenceClass::Designed);
                assert_eq!(row.date, "2026-09-05");
                assert_eq!(row.adr, "ADR255");
            } else if row.rule_id == "struggle/spark-mechanic" {
                assert_eq!(row.evidence, EvidenceClass::Designed);
                assert_eq!(row.date, "2026-09-01");
                assert_eq!(row.adr, "ADR248");
            } else if row.rule_id == "struggle/spark-recognizer" {
                assert_eq!(row.evidence, EvidenceClass::Derived);
                assert_eq!(row.date, "2026-09-01");
                assert_eq!(row.adr, "ADR248");
            } else {
                assert_eq!(row.evidence, EvidenceClass::Derived);
                assert_eq!(row.date, "2026-08-23");
                assert_eq!(row.adr, "ADR224");
            }
        }
    }

    #[test]
    fn receipt_reduction_places_events_before_writes_with_continuous_ordinals() {
        let contract = RuleContract {
            rule_id: "demo/probe".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        let records = vec![WriteRecord {
            rule: "demo/probe".to_owned(),
            ordinal: 0,
            write: Write::NodeAttribute {
                id: NodeId(42),
                field: "social-class/population".to_owned(),
                previous: Some(10.0),
                value: 9.0,
            },
        }];
        let receipts = reduce_audit_receipts(
            &contract,
            &["FIRST".to_owned(), "EventType/SECOND".to_owned()],
            &records,
        )
        .unwrap();
        assert_eq!(
            receipts
                .iter()
                .map(|receipt| (receipt.ordinal, receipt.effect.clone()))
                .collect::<Vec<_>>(),
            vec![
                (0, EffectSignature::Event("EventType/FIRST".to_owned())),
                (1, EffectSignature::Event("EventType/SECOND".to_owned())),
                (
                    2,
                    EffectSignature::NodeField("social-class/population".to_owned())
                ),
            ]
        );
    }

    #[test]
    fn receipt_capacity_refuses_an_unrepresentable_last_ordinal_before_allocation() {
        assert_eq!(checked_receipt_capacity(2, 1).unwrap(), 3);
        assert_eq!(
            checked_receipt_capacity(u64::from(u32::MAX) + 2, 0).unwrap_err(),
            ContractError::ReceiptOrdinalOverflow
        );
    }

    #[test]
    fn receipt_reduction_refuses_a_sparse_or_reordered_write_log() {
        let contract = RuleContract {
            rule_id: "demo/probe".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        let record = WriteRecord {
            rule: contract.rule_id.clone(),
            ordinal: 1,
            write: Write::NodeAttribute {
                id: NodeId(42),
                field: "social-class/agitation".to_owned(),
                previous: None,
                value: 0.5,
            },
        };
        assert_eq!(
            reduce_audit_receipts(&contract, &[], &[record]).unwrap_err(),
            ContractError::MismatchedWriteOrdinal {
                expected: 0,
                actual: 1,
            }
        );
    }

    #[test]
    fn receipt_reduction_refuses_a_write_attributed_to_another_rule() {
        let contract = RuleContract {
            rule_id: "demo/probe".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        let record = WriteRecord {
            rule: "demo/other".to_owned(),
            ordinal: 0,
            write: Write::NodeAttribute {
                id: NodeId(42),
                field: "social-class/agitation".to_owned(),
                previous: None,
                value: 0.5,
            },
        };
        assert_eq!(
            reduce_audit_receipts(&contract, &[], &[record]).unwrap_err(),
            ContractError::MismatchedWriteAttribution {
                expected: "demo/probe".to_owned(),
                actual: "demo/other".to_owned(),
            }
        );
    }

    #[test]
    fn receipt_events_accept_only_bare_or_event_type_members() {
        let contract = RuleContract {
            rule_id: "demo/probe".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        for invalid in ["", "OtherEnum/X", "EventType/", "EventType/X/Y"] {
            let error = reduce_audit_receipts(&contract, &[invalid.to_owned()], &[])
                .expect_err("a noncanonical event namespace must refuse");
            assert_eq!(
                error,
                ContractError::MalformedEventType {
                    value: invalid.to_owned()
                }
            );
        }
    }

    #[test]
    fn raw_node_ids_and_values_do_not_affect_public_receipt_equality() {
        let contract = RuleContract {
            rule_id: "demo/probe".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        let record = |id, previous, value| WriteRecord {
            rule: "demo/probe".to_owned(),
            ordinal: 0,
            write: Write::NodeAttribute {
                id: NodeId(id),
                field: "social-class/population".to_owned(),
                previous,
                value,
            },
        };
        let first = reduce_audit_receipts(&contract, &[], &[record(1, Some(9.0), 8.0)]).unwrap();
        let second =
            reduce_audit_receipts(&contract, &[], &[record(999, Some(20.0), 3.0)]).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn a_recognizer_unexpected_runtime_effect_is_refused_redundantly() {
        let contract = RuleContract {
            rule_id: "control-ratio/c03-crisis".to_owned(),
            role: RuleRole::Recognizer,
            evidence: EvidenceClass::Derived,
        };
        let error =
            reduce_audit_receipts(&contract, &["TERMINAL_DECISION".to_owned()], &[]).unwrap_err();
        assert_eq!(error.spec_code(), Some("E-LOAD-060"));
        assert!(matches!(
            error,
            ContractError::UnauthorizedEffect {
                effect: EffectSignature::Event(event),
                ..
            } if event == "EventType/TERMINAL_DECISION"
        ));
    }

    #[test]
    fn runtime_reduction_rejects_a_forged_builtin_mechanic_contract() {
        let forged = RuleContract {
            rule_id: "control-ratio/c03-crisis".to_owned(),
            role: RuleRole::Mechanic,
            evidence: EvidenceClass::Derived,
        };
        assert!(matches!(
            reduce_audit_receipts(&forged, &["TERMINAL_DECISION".to_owned()], &[]),
            Err(ContractError::GovernedAttributionMismatch { .. })
        ));
    }

    #[test]
    fn the_allowed_recognizer_runtime_sequence_reduces_to_receipts() {
        let contract = RuleContract {
            rule_id: "control-ratio/c03-crisis".to_owned(),
            role: RuleRole::Recognizer,
            evidence: EvidenceClass::Derived,
        };
        let write = |ordinal: u32, field: &str| WriteRecord {
            rule: contract.rule_id.clone(),
            ordinal,
            write: Write::NodeAttribute {
                id: NodeId(u64::from(ordinal) + 10),
                field: field.to_owned(),
                previous: Some(0.0),
                value: 1.0,
            },
        };
        let receipts = reduce_audit_receipts(
            &contract,
            &["CONTROL_RATIO_CRISIS".to_owned()],
            &[
                write(0, "institution/control-crisis-emitted"),
                write(1, "institution/control-crisis-tick"),
            ],
        )
        .unwrap();
        assert_eq!(receipts.len(), 3);
        assert_eq!(receipts[0].ordinal, 0);
        assert_eq!(receipts[2].ordinal, 2);
    }
}
