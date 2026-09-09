//! Amendment AJ's exact finite material probability boundary.
//!
//! `Mass` is an exact, non-storable authoring value. Finite kernels compile
//! to typed IR before either realization or forecasting consumes them. Ticket
//! allocation is integer-only and covers the complete `2^64` draw space.

use crate::bindings::{BindSource, BindingDecl};
use crate::causal_contract::{effect_footprint, EffectSignature, RuleContract, RuleRole};
use crate::evaluator::{evaluate, EvalEnv, EvalError, Value};
use crate::intrinsic_host::IntrinsicHost;
use crate::reader::{Atom, FormPath, SExpr, ScaledKind};
use crate::typecheck::TypeEnv;
use crate::types::{BslType, EnumRegistry, EnumTypeId};
use babylon_graph::stable_element::StableElementKeyV1;
use babylon_kernel::sha256_of;

/// The fixed decimal scale of a BSL `m` literal.
pub const MASS_NANOUNITS_PER_UNIT: u64 = 1_000_000_000;

/// The exact number of tickets in every finite-kernel allocation.
pub const TICKET_DENOMINATOR: u128 = 1_u128 << 64;

/// Amendment AJ V1 fixed fuel cost for one private integer ticket draw.
/// This deliberately aliases the governed intrinsic-call base without
/// widening the frozen SFS V1 fuel-source identity.
pub const FINITE_KERNEL_DRAW_BASE: u64 = crate::fuel::cost::INTRINSIC_CALL_BASE;

/// Amendment AJ V1 fixed fuel cost for an explicit numeric-to-Mass crossing.
/// This deliberately aliases the governed intrinsic-call base without
/// widening the frozen SFS V1 fuel-source identity.
pub const QUANTIZE_MASS_BASE: u64 = crate::fuel::cost::INTRINSIC_CALL_BASE;

/// Exact finite-kernel allocation input, represented as unsigned nanounits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mass(u64);

impl Mass {
    /// Construct from the canonical unsigned nanounit representation.
    #[must_use]
    pub const fn from_nanounits(nanounits: u64) -> Self {
        Self(nanounits)
    }

    /// Return the canonical unsigned nanounit representation.
    #[must_use]
    pub const fn nanounits(self) -> u64 {
        self.0
    }

    /// Checked exact addition.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::MassOverflow`] when the sum exceeds the
    /// canonical `u64` nanounit representation.
    pub fn checked_add(self, rhs: Self) -> Result<Self, ProbabilityError> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(ProbabilityError::MassOverflow)
    }

    /// Checked exact subtraction. Negative `Mass` is not representable.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::MassUnderflow`] when `rhs` is greater than
    /// `self`.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, ProbabilityError> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(ProbabilityError::MassUnderflow)
    }

    /// Explicitly quantize one finite, non-negative binary64 value to Mass.
    /// Rounding is nearest nanounit with ties to even.
    ///
    /// # Errors
    ///
    /// Returns a typed probability error for a negative or non-finite input,
    /// or when the rounded nanounit representation exceeds `u64`.
    pub fn quantize(value: f64) -> Result<Self, ProbabilityError> {
        if !value.is_finite() {
            return Err(ProbabilityError::NonFiniteMassInput);
        }
        // IEEE signed zero has numeric value zero, not a negative value.
        // AJ says to refuse a negative result, so both zero encodings
        // canonicalize to the same zero Mass.
        if value < 0.0 {
            return Err(ProbabilityError::NegativeMassInput);
        }

        // Decode the exact represented binary64 value as
        // `significand * 2^binary_exponent`. Multiplying the significand by
        // 10^9 needs at most 83 bits, so a u128 carries the complete rational
        // numerator. No binary64 multiply or intermediate rounding occurs.
        let bits = value.to_bits();
        let exponent_field =
            i32::try_from((bits >> 52) & 0x7ff).map_err(|_| ProbabilityError::MassOverflow)?;
        let fraction = bits & ((1_u64 << 52) - 1);
        let (significand, binary_exponent) = if exponent_field == 0 {
            // Subnormal and zero: no implicit leading one.
            (fraction, -1_074)
        } else {
            ((1_u64 << 52) | fraction, exponent_field - 1_023 - 52)
        };
        let numerator = u128::from(significand) * u128::from(MASS_NANOUNITS_PER_UNIT);
        let rounded = if binary_exponent >= 0 {
            let shift =
                u32::try_from(binary_exponent).map_err(|_| ProbabilityError::MassOverflow)?;
            if shift >= u128::BITS || numerator > (u128::MAX >> shift) {
                return Err(ProbabilityError::MassOverflow);
            }
            numerator << shift
        } else {
            let shift = binary_exponent.unsigned_abs();
            if shift >= u128::BITS {
                // The numerator is at most 83 bits, hence strictly below
                // half of a 2^128 (or larger) denominator.
                0
            } else {
                let denominator = 1_u128 << shift;
                let quotient = numerator >> shift;
                let remainder = numerator & (denominator - 1);
                let halfway = denominator >> 1;
                if remainder > halfway || (remainder == halfway && quotient & 1 == 1) {
                    quotient
                        .checked_add(1)
                        .ok_or(ProbabilityError::MassOverflow)?
                } else {
                    quotient
                }
            }
        };
        u64::try_from(rounded)
            .map(Self)
            .map_err(|_| ProbabilityError::MassOverflow)
    }
}

/// One enum-ordered half-open interval in the exact ticket allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketIntervalV1 {
    /// Inclusive ticket start.
    pub start: u128,
    /// Exclusive ticket end.
    pub end: u128,
    /// Number of tickets assigned to this branch.
    pub count: u128,
}

/// One exact Mass literal and its source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MassLiteralFactV1 {
    /// Path directly consumable by `SpanTable`.
    pub form_path: FormPath,
    /// Canonical exact value.
    pub mass: Mass,
}

/// One compiled branch of a finite material transition kernel.
#[derive(Debug, Clone, PartialEq)]
pub struct KernelBranchV1 {
    /// The common branch enum type's written name.
    pub enum_type: String,
    /// The outcome member, in declaration order.
    pub member: String,
    /// The declared-order ordinal.
    pub ordinal: u32,
    /// Exact or dynamically quantized Mass expression.
    pub mass: SExpr,
    /// Existing deterministic mechanic effect items for this alternative.
    pub effects: Vec<SExpr>,
    /// Path of this `(branch ...)` form within the rule AST.
    pub form_path: FormPath,
    /// Path of the `branch` head token.
    pub head_path: FormPath,
    /// Path of the branch's `:mass` expression.
    pub mass_path: FormPath,
    /// Mass literal token paths within the mass expression.
    pub mass_literals: Vec<MassLiteralFactV1>,
    /// `quantize-mass` head-token paths within the mass expression.
    pub quantize_mass_paths: Vec<FormPath>,
    /// Exact load-time Mass when the static analyzer can fold the expression.
    pub static_mass: Option<Mass>,
}

/// Typed IR for one direct finite-kernel choice.
#[derive(Debug, Clone, PartialEq)]
pub struct FiniteKernelV1 {
    /// Content-set-unique stable sample identity.
    pub sample: String,
    /// Path of the sample `QName` token.
    pub sample_path: FormPath,
    /// Append-only author-declared draw slot.
    pub slot: u32,
    /// Path of the slot's canonical `u32` literal token.
    pub slot_path: FormPath,
    /// Registry identity of the common branch enum.
    pub enum_type: EnumTypeId,
    /// Written common enum type name.
    pub enum_type_name: String,
    /// Branches in the enum's declaration order.
    pub branches: Vec<KernelBranchV1>,
    /// Path of the `(choose ...)` form within the rule AST.
    pub form_path: FormPath,
    /// Path of the `choose` head token.
    pub head_path: FormPath,
}

/// Typed IR for one exact subject-local recognizer projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteProjectionV1 {
    /// Stable sample identity of the immediately preceding kernel.
    pub sample: String,
    /// Path of the rule-level `:projects-kernel` keyword.
    pub form_path: FormPath,
    /// Path of the projection sample `QName` value.
    pub sample_path: FormPath,
    /// Loader-proven event members this recognizer can emit, in first source order.
    ///
    /// Retaining the complete static footprint lets exact forecasting publish a
    /// zero numerator when no kernel branch reaches an authored event.
    pub event_types: Vec<String>,
}

/// Enum-ordered allocation facts retained by a realization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedBranchV1 {
    /// Outcome member.
    pub member: String,
    /// Exact evaluated Mass.
    pub mass: Mass,
    /// Exact ticket interval.
    pub tickets: TicketIntervalV1,
}

/// Replay-keyed stable carrier identity for one choice instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelInstanceIdentityV1 {
    /// Exact checked replay-session bytes.
    pub replay_session: Vec<u8>,
    /// Canonical signed replay-seed bytes.
    pub replay_seed: [u8; 8],
    /// Positive tick being adjudicated.
    pub tick: i64,
    /// Firing rule identity.
    pub rule_id: String,
    /// Stable subject identity.
    pub subject: StableElementKeyV1,
    /// Ordered active-element stable identities.
    pub active_elements: Vec<StableElementKeyV1>,
}

/// Engine-neutral result of realizing one compiled choice.
///
/// `ChoiceReceiptV1` is deliberately not defined here: the tick transaction
/// owns its durable receipt type. This record supplies the complete exact
/// material from which that receipt is constructed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelRealizationV1 {
    /// Firing rule identity.
    pub rule_id: String,
    /// Stable sample identity.
    pub sample: String,
    /// Append-only slot.
    pub slot: u32,
    /// Common outcome enum type.
    pub enum_type: String,
    /// Stable subject identity.
    pub subject: StableElementKeyV1,
    /// Ordered active-element stable identities.
    pub active_elements: Vec<StableElementKeyV1>,
    /// Enum-ordered masses and ticket intervals.
    pub branches: Vec<RealizedBranchV1>,
    /// The one private integer draw.
    pub draw: u64,
    /// Selected outcome member.
    pub selected_outcome: String,
    /// Digest of the enum-ordered allocation.
    pub allocation_digest: [u8; 32],
    /// Digest binding the realization instance and allocation.
    pub instance_digest: [u8; 32],
}

/// Authoring-relevant typed probability node retained by loader analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbabilityAnalysisNodeKindV1 {
    /// `choose` head token.
    Choose,
    /// `branch` head token.
    Branch,
    /// One branch Mass expression.
    BranchMass,
    /// One exact `m` literal token.
    MassLiteral,
    /// One `quantize-mass` head token.
    QuantizeMass,
    /// Rule-level `:projects-kernel` keyword.
    ProjectionKeyword,
    /// Projection sample `QName` token.
    ProjectionSample,
}

/// One typed probability node and its parser-stable source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbabilityAnalysisNodeV1 {
    /// Semantic node kind.
    pub kind: ProbabilityAnalysisNodeKindV1,
    /// Path directly consumable by the parser's `SpanTable`.
    pub form_path: FormPath,
}

/// Whole-rule material-effect locality retained for finite projection linking.
///
/// A standalone kernel may lawfully require a joint carrier. The retained
/// refusal becomes active only when an adjacent recognizer asks the finite
/// projection boundary to enumerate that kernel exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelProjectionLocalityV1 {
    /// Every material effect in the kernel rule is local to its firing subject.
    CarrierLocal,
    /// Exact V1 projection cannot enumerate a material effect over another or
    /// shared carrier.
    RequiresJointCarrier {
        /// Stable author-facing explanation retained at compile time.
        message: String,
        /// Exact source path of the material effect that establishes refusal.
        form_path: FormPath,
    },
}

/// Rule-local authoring facts retained by the probability compiler.
///
/// These facts are produced while the loader owns the raw rule form. Later
/// content analysis and the LSP consume this typed record and never walk the
/// rule's `SExpr` again.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompiledProbabilityFactsV1 {
    /// Typed probability token/form nodes in source order.
    pub nodes: Vec<ProbabilityAnalysisNodeV1>,
    /// Every exact Mass literal in the rule, including Mass bindings.
    pub mass_literals: Vec<MassLiteralFactV1>,
    /// Whole-rule kernel locality, when this rule compiled a finite kernel.
    pub kernel_projection_locality: Option<KernelProjectionLocalityV1>,
}

/// Complete probability product of compiling one loaded rule.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledRuleProbabilityV1 {
    /// Typed finite kernel, when the rule declares `choose`.
    pub kernel: Option<FiniteKernelV1>,
    /// Typed deterministic projection, when the rule declares
    /// `:projects-kernel`.
    pub projection: Option<FiniteProjectionV1>,
    /// Loader-retained authoring facts from the same compilation.
    pub facts: CompiledProbabilityFactsV1,
}

/// Exact static allocation available without a world state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactKernelAllocationV1 {
    /// Enum-ordered exact masses.
    pub masses: Vec<Mass>,
    /// Enum-ordered half-open ticket intervals.
    pub intervals: Vec<TicketIntervalV1>,
}

/// Whether exact allocation is available during content analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocationAnalysisV1 {
    /// Every Mass was state-independent and allocation is exact.
    Exact(ExactKernelAllocationV1),
    /// At least one Mass was not determined by load-time static analysis.
    Unavailable {
        /// Stable author-facing explanation.
        reason: String,
    },
}

/// Per-loaded-rule probability facts from the one typed loader path.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProbabilityAnalysisV1 {
    /// Source identity retained by `LoadedRule`.
    pub source_id: String,
    /// Governed rule `QName`.
    pub rule_id: String,
    /// Typed kernel IR, when this rule declares one.
    pub kernel: Option<FiniteKernelV1>,
    /// Typed projection IR, when this rule declares one.
    pub projection: Option<FiniteProjectionV1>,
    /// Token/form nodes for diagnostics, hover, and semantic tokens.
    pub nodes: Vec<ProbabilityAnalysisNodeV1>,
    /// Every exact Mass literal in this loaded rule, including Mass bindings.
    pub mass_literals: Vec<MassLiteralFactV1>,
    /// Static allocation facts for a kernel rule.
    pub allocation: Option<AllocationAnalysisV1>,
}

/// One loader-confirmed Mass constant literal from a scenario or prelude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MassDeclarationAnalysisV1 {
    /// Caller-owned source identity used by authoring tools.
    pub source_id: String,
    /// Declared constant `QName`.
    pub qname: String,
    /// Exact literal token path in this source's parser span table.
    pub form_path: FormPath,
    /// Canonical exact Mass value confirmed against the loaded const registry.
    pub mass: Mass,
}

/// Why analysis cannot publish an exact projected event likelihood yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LikelihoodAnalysisV1 {
    /// Exact bounded detached-state likelihoods for a paired scenario.
    Exact(Vec<EventLikelihoodV1>),
    /// A bounded state clone and deterministic projection are still required.
    StateDependent {
        /// Stable author-facing explanation.
        reason: String,
    },
}

/// One validated adjacent kernel/projection relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelProjectionLinkV1 {
    /// Shared stable sample `QName`.
    pub sample: String,
    /// Mechanic rule `QName`.
    pub kernel_rule_id: String,
    /// Adjacent recognizer rule `QName`.
    pub projection_rule_id: String,
    /// Content-only likelihood availability.
    pub likelihood: LikelihoodAnalysisV1,
}

/// Typed analysis for one already resolved content-set schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ContentSetAnalysisV1 {
    /// Loader-confirmed scenario/prelude Mass constants, populated by the
    /// source-level content orchestrator.
    pub mass_declarations: Vec<MassDeclarationAnalysisV1>,
    /// Rules in resolved schedule order.
    pub rules: Vec<RuleProbabilityAnalysisV1>,
    /// Validated adjacent links in projection encounter order.
    pub links: Vec<KernelProjectionLinkV1>,
}

/// Deterministic recognizer result for one enum-ordered branch forecast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchProjectionV1 {
    /// Outcome member, used to validate enum order and prevent cross-kernel joins.
    pub outcome: String,
    /// Event types emitted after applying this branch to a cloned pre-choice state.
    pub event_types: Vec<String>,
}

/// Exact finite preimage measure of one projected event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLikelihoodV1 {
    /// Projected event enum reference/name.
    pub event_type: String,
    /// Favorable kernel outcomes in enum declaration order.
    pub favorable_outcomes: Vec<String>,
    /// Exact favorable ticket count.
    pub numerator: u128,
    /// Always exactly `2^64` in V1.
    pub denominator: u128,
}

/// Located details for an adjacent kernel/projection carrier mismatch.
///
/// Boxed inside [`ProbabilityError`] so this authoring-rich refusal does not
/// enlarge every unrelated probability result on the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectCarrierMismatchV1 {
    /// Shared sample `QName`.
    pub sample: String,
    /// Mechanic rule declaring the kernel.
    pub kernel_rule_id: String,
    /// Recognizer rule declaring the projection.
    pub projection_rule_id: String,
    /// Runtime-resolved mechanic subject type.
    pub kernel_carrier: String,
    /// Runtime-resolved recognizer subject type.
    pub projection_carrier: String,
    /// Path of the projection sample `QName` token.
    pub form_path: FormPath,
}

/// A refusal at the exact finite-probability boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbabilityError {
    /// Exact Mass addition or quantization exceeded `u64` nanounits.
    MassOverflow,
    /// Exact Mass subtraction would be negative.
    MassUnderflow,
    /// `quantize-mass` received a negative input.
    NegativeMassInput,
    /// `quantize-mass` received NaN or infinity.
    NonFiniteMassInput,
    /// A finite kernel's evaluated total mass was zero.
    ZeroTotalMass,
    /// Summing branch masses overflowed the exact `u128` allocation accumulator.
    TotalMassOverflow,
    /// Largest-remainder allocation gave a positive branch zero tickets.
    PositiveMassHasZeroTickets { index: usize },
    /// A ticket was not covered by the supposedly exhaustive allocation.
    TicketNotCovered { draw: u64 },
    /// The BSL kernel/projection surface was malformed or unauthorized.
    InvalidForm {
        /// Precise human-readable refusal.
        message: String,
        /// AST node that owns the refusal where known.
        form_path: FormPath,
    },
    /// A sample `QName` was declared by more than one kernel.
    DuplicateSample {
        /// Repeated sample `QName`.
        sample: String,
        /// Rule containing the offending later declaration.
        rule_id: String,
        /// Path of its `:sample` `QName` token.
        form_path: FormPath,
    },
    /// A projection did not name its immediately preceding kernel.
    ProjectionNotAdjacent {
        /// Projected sample `QName`.
        sample: String,
        /// Projection rule owning the refusal.
        rule_id: String,
        /// Path of its `:projects-kernel` sample `QName` token.
        form_path: FormPath,
    },
    /// An adjacent projection would observe a different subject population.
    SubjectCarrierMismatch(Box<SubjectCarrierMismatchV1>),
}

impl ProbabilityError {
    /// The path that owns this error, when it is a single-form refusal.
    #[must_use]
    pub fn form_path(&self) -> Option<&[u32]> {
        match self {
            Self::InvalidForm { form_path, .. }
            | Self::DuplicateSample { form_path, .. }
            | Self::ProjectionNotAdjacent { form_path, .. } => Some(form_path),
            Self::SubjectCarrierMismatch(details) => Some(&details.form_path),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProbabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MassOverflow => write!(f, "Mass nanounit arithmetic overflowed u64"),
            Self::MassUnderflow => write!(f, "Mass subtraction would be negative"),
            Self::NegativeMassInput => write!(f, "quantize-mass refuses a negative input"),
            Self::NonFiniteMassInput => write!(f, "quantize-mass refuses NaN or infinity"),
            Self::ZeroTotalMass => write!(f, "a finite kernel's total mass must be positive"),
            Self::TotalMassOverflow => write!(f, "a finite kernel's total mass overflowed u128"),
            Self::PositiveMassHasZeroTickets { index } => write!(
                f,
                "positive branch mass at enum ordinal {index} received zero tickets"
            ),
            Self::TicketNotCovered { draw } => {
                write!(f, "ticket draw {draw} was not covered by the allocation")
            }
            Self::InvalidForm { message, .. } => write!(f, "{message}"),
            Self::DuplicateSample {
                sample, rule_id, ..
            } => {
                write!(f, "finite-kernel sample `{sample}` in `{rule_id}` is not content-set unique")
            }
            Self::ProjectionNotAdjacent {
                sample, rule_id, ..
            } => write!(
                f,
                "projection `{rule_id}` for `{sample}` is not immediately after that resolved kernel"
            ),
            Self::SubjectCarrierMismatch(details) => write!(
                f,
                "projection `{}` for `{}` resolves subject carrier `{}`, but adjacent kernel `{}` resolves `{}`",
                details.projection_rule_id,
                details.sample,
                details.projection_carrier,
                details.kernel_rule_id,
                details.kernel_carrier,
            ),
        }
    }
}

impl std::error::Error for ProbabilityError {}

fn invalid(path: &[u32], message: impl Into<String>) -> ProbabilityError {
    ProbabilityError::InvalidForm {
        message: message.into(),
        form_path: path.to_vec(),
    }
}

fn head(expr: &SExpr) -> Option<&str> {
    let SExpr::List(items) = expr else {
        return None;
    };
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(value))) => Some(value),
        _ => None,
    }
}

fn child_path(parent: &[u32], index: usize) -> Result<FormPath, ProbabilityError> {
    let index = u32::try_from(index).map_err(|_| invalid(parent, "form path exceeds u32"))?;
    let mut path = parent.to_vec();
    path.push(index);
    Ok(path)
}

/// Return the semantically executable children of one form with their exact
/// source paths.
///
/// An `emit` payload item is `(<symbol> <expr>)`: its first element is a
/// label, not a form head. Once the enclosing `emit` has its mandatory event
/// enum operand, probability analysis therefore descends only into each
/// payload value. Malformed `emit` forms retain the generic traversal so this
/// pass never hides a nested form from the grammar diagnostics that own it.
fn semantic_children_with_paths<'a>(
    expr: &'a SExpr,
    path: &[u32],
) -> Result<Vec<(&'a SExpr, FormPath)>, ProbabilityError> {
    let SExpr::List(items) = expr else {
        return Ok(Vec::new());
    };
    let is_typed_emit = matches!(
        items.as_slice(),
        [
            SExpr::Atom(Atom::Symbol(form)),
            SExpr::Atom(Atom::EnumRef { .. }),
            ..
        ] if form == "emit"
    );
    if !is_typed_emit {
        return items
            .iter()
            .enumerate()
            .map(|(index, child)| Ok((child, child_path(path, index)?)))
            .collect();
    }

    let mut children = Vec::new();
    for (payload_index, payload_item) in items.iter().enumerate().skip(2) {
        let SExpr::List(pair) = payload_item else {
            continue;
        };
        let payload_path = child_path(path, payload_index)?;
        for (value_index, value) in pair.iter().enumerate().skip(1) {
            children.push((value, child_path(&payload_path, value_index)?));
        }
    }
    Ok(children)
}

fn mass_expression_is_typed(
    expr: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> bool {
    match expr {
        SExpr::Atom(Atom::Mass(_)) => true,
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return false;
            }
            let Some(binding) = bindings.iter().find(|binding| binding.name == *name) else {
                return false;
            };
            resolving.push(name.clone());
            let is_mass = match &binding.source {
                BindSource::Const(qname) => matches!(consts.get(qname), Some(Value::Mass(_))),
                BindSource::Expr(source) => {
                    mass_expression_is_typed(source, types, bindings, consts, resolving)
                }
                BindSource::Field(_)
                | BindSource::Metric(_)
                | BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => false,
            };
            resolving.pop();
            is_mass
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs] if matches!(op.as_str(), "+" | "-") => {
                mass_expression_is_typed(lhs, types, bindings, consts, resolving)
                    && mass_expression_is_typed(rhs, types, bindings, consts, resolving)
            }
            [SExpr::Atom(Atom::Symbol(form)), operand] if form == "quantize-mass" => {
                ordinary_numeric_expression_is_typed(operand, types, bindings, consts, resolving)
            }
            _ => false,
        },
        SExpr::Atom(_) => false,
    }
}

fn bsl_type_is_quantizable(ty: &BslType) -> bool {
    matches!(
        ty,
        BslType::Int
            | BslType::Real
            | BslType::Probability
            | BslType::Intensity
            | BslType::Coefficient
    )
}

fn declared_numeric_type(types: &TypeEnv, name: &str) -> bool {
    types
        .fields
        .get(name)
        .is_some_and(|declaration| bsl_type_is_quantizable(&declaration.ty))
}

#[allow(clippy::too_many_lines)]
fn ordinary_numeric_expression_is_typed(
    expr: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> bool {
    match expr {
        SExpr::Atom(Atom::Int(_)) => true,
        SExpr::Atom(Atom::Scaled(value)) => value.kind != crate::reader::ScaledKind::Ratio,
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return false;
            }
            let Some(binding) = bindings.iter().find(|binding| binding.name == *name) else {
                return false;
            };
            resolving.push(name.clone());
            let is_numeric = match &binding.source {
                BindSource::Const(qname) => {
                    matches!(consts.get(qname), Some(Value::Int(_) | Value::Real(_)))
                }
                BindSource::Expr(source) => {
                    ordinary_numeric_expression_is_typed(source, types, bindings, consts, resolving)
                }
                BindSource::Field(qname) | BindSource::Metric(qname) => {
                    declared_numeric_type(types, qname)
                }
                BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => true,
            };
            resolving.pop();
            is_numeric
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs]
                if matches!(op.as_str(), "+" | "-" | "*" | "/") =>
            {
                let operands_are_numeric =
                    ordinary_numeric_expression_is_typed(lhs, types, bindings, consts, resolving)
                        && ordinary_numeric_expression_is_typed(
                            rhs, types, bindings, consts, resolving,
                        );
                operands_are_numeric
                    && (op != "/"
                        || !(ordinary_numeric_expression_is_statically_int(
                            lhs, types, bindings, consts, resolving,
                        ) && ordinary_numeric_expression_is_statically_int(
                            rhs, types, bindings, consts, resolving,
                        )))
            }
            [SExpr::Atom(Atom::Symbol(form)), _, then_value, else_value] if form == "if" => {
                ordinary_numeric_expression_is_typed(then_value, types, bindings, consts, resolving)
                    && ordinary_numeric_expression_is_typed(
                        else_value, types, bindings, consts, resolving,
                    )
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::QName(qname))]
                if form == "field-of" =>
            {
                declared_numeric_type(types, qname)
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::Symbol(metric))]
                if form == "metric-of" =>
            {
                declared_numeric_type(types, metric)
            }
            [SExpr::Atom(Atom::Symbol(form)), SExpr::Atom(Atom::Symbol(op)), tail @ ..]
                if form == "fold" =>
            {
                if op == "count" {
                    true
                } else {
                    let body = if matches!(tail.get(1), Some(SExpr::Atom(Atom::Keyword(keyword))) if keyword == "as")
                    {
                        tail.get(3)
                    } else {
                        tail.get(1)
                    };
                    body.is_some_and(|body| {
                        ordinary_numeric_expression_is_typed(
                            body, types, bindings, consts, resolving,
                        )
                    })
                }
            }
            [SExpr::Atom(Atom::Symbol(form)), operands @ ..]
                if matches!(
                    form.as_str(),
                    "min"
                        | "max"
                        | "abs"
                        | "clamp"
                        | "floor"
                        | "round-half-even"
                        | "sqrt"
                        | "exp"
                        | "log"
                ) =>
            {
                !operands.is_empty()
                    && operands.iter().all(|operand| {
                        ordinary_numeric_expression_is_typed(
                            operand, types, bindings, consts, resolving,
                        )
                    })
            }
            _ => false,
        },
        SExpr::Atom(
            Atom::Mass(_)
            | Atom::Bool(_)
            | Atom::Currency(_)
            | Atom::QName(_)
            | Atom::EnumRef { .. }
            | Atom::Keyword(_)
            | Atom::BareUpperIdent(_)
            | Atom::Str(_)
            | Atom::Operator(_),
        ) => false,
    }
}

fn ordinary_numeric_expression_is_statically_int(
    expr: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> bool {
    match expr {
        SExpr::Atom(Atom::Int(_)) => true,
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return false;
            }
            let Some(binding) = bindings.iter().find(|binding| binding.name == *name) else {
                return false;
            };
            resolving.push(name.clone());
            let is_int = match &binding.source {
                BindSource::Const(qname) => matches!(consts.get(qname), Some(Value::Int(_))),
                BindSource::Expr(source) => ordinary_numeric_expression_is_statically_int(
                    source, types, bindings, consts, resolving,
                ),
                BindSource::Field(qname) | BindSource::Metric(qname) => types
                    .fields
                    .get(qname)
                    .is_some_and(|declaration| declaration.ty == BslType::Int),
                BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => true,
            };
            resolving.pop();
            is_int
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs]
                if matches!(op.as_str(), "+" | "-" | "*") =>
            {
                ordinary_numeric_expression_is_statically_int(
                    lhs, types, bindings, consts, resolving,
                ) && ordinary_numeric_expression_is_statically_int(
                    rhs, types, bindings, consts, resolving,
                )
            }
            [SExpr::Atom(Atom::Symbol(form)), _, then_value, else_value] if form == "if" => {
                ordinary_numeric_expression_is_statically_int(
                    then_value, types, bindings, consts, resolving,
                ) && ordinary_numeric_expression_is_statically_int(
                    else_value, types, bindings, consts, resolving,
                )
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::QName(qname))]
                if form == "field-of" =>
            {
                types
                    .fields
                    .get(qname)
                    .is_some_and(|declaration| declaration.ty == BslType::Int)
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::Symbol(metric))]
                if form == "metric-of" =>
            {
                types
                    .fields
                    .get(metric)
                    .is_some_and(|declaration| declaration.ty == BslType::Int)
            }
            [SExpr::Atom(Atom::Symbol(form)), SExpr::Atom(Atom::Symbol(op)), tail @ ..]
                if form == "fold" =>
            {
                if op == "count" {
                    true
                } else if matches!(op.as_str(), "sum" | "min" | "max") {
                    let body = if matches!(tail.get(1), Some(SExpr::Atom(Atom::Keyword(keyword))) if keyword == "as")
                    {
                        tail.get(3)
                    } else {
                        tail.get(1)
                    };
                    body.is_some_and(|body| {
                        ordinary_numeric_expression_is_statically_int(
                            body, types, bindings, consts, resolving,
                        )
                    })
                } else {
                    false
                }
            }
            _ => false,
        },
        SExpr::Atom(_) => false,
    }
}

fn expression_contains_typed_mass(
    expr: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> bool {
    if mass_expression_is_typed(expr, types, bindings, consts, resolving) {
        return true;
    }
    match expr {
        SExpr::List(items) => items
            .iter()
            .any(|item| expression_contains_typed_mass(item, types, bindings, consts, resolving)),
        SExpr::Atom(_) => false,
    }
}

fn declared_probability_type(types: &TypeEnv, name: &str) -> bool {
    types
        .fields
        .get(name)
        .is_some_and(|declaration| declaration.ty == BslType::Probability)
}

/// Whether an expression's bottom-up static type is `Probability`.
///
/// This deliberately follows §3.3 rather than treating every expression
/// containing a probability as probability-typed: binary64 arithmetic
/// promotes to `Real`, comparisons produce `Bool`, and the declarable
/// intrinsic set returns only `Real` or `Int`. Accessors, folds, `if`, and
/// binding aliases retain the exact type of their source. Until the complete
/// bottom-up type checker exists, either `if` branch is a refusal witness:
/// well-typed branches agree, while an unlike pair must not let its
/// Probability-capable branch escape the authored-event prohibition.
fn expression_is_typed_probability(
    expr: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    probability_consts: &std::collections::HashSet<String>,
    resolving: &mut Vec<String>,
) -> bool {
    match expr {
        SExpr::Atom(Atom::Scaled(value)) => value.kind == ScaledKind::Probability,
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return false;
            }
            let Some(binding) = bindings.iter().find(|binding| binding.name == *name) else {
                return false;
            };
            resolving.push(name.clone());
            let is_probability = match &binding.source {
                BindSource::Const(qname) => probability_consts.contains(qname),
                BindSource::Expr(source) => expression_is_typed_probability(
                    source,
                    types,
                    bindings,
                    probability_consts,
                    resolving,
                ),
                BindSource::Field(qname) | BindSource::Metric(qname) => {
                    declared_probability_type(types, qname)
                }
                BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => false,
            };
            resolving.pop();
            is_probability
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Symbol(form)), _, then_value, else_value] if form == "if" => {
                expression_is_typed_probability(
                    then_value,
                    types,
                    bindings,
                    probability_consts,
                    resolving,
                ) || expression_is_typed_probability(
                    else_value,
                    types,
                    bindings,
                    probability_consts,
                    resolving,
                )
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::QName(qname))]
                if form == "field-of" =>
            {
                declared_probability_type(types, qname)
            }
            [SExpr::Atom(Atom::Symbol(form)), _, SExpr::Atom(Atom::Symbol(metric))]
                if form == "metric-of" =>
            {
                declared_probability_type(types, metric)
            }
            [SExpr::Atom(Atom::Symbol(form)), SExpr::Atom(Atom::Symbol(op)), tail @ ..]
                if form == "fold" && op != "count" =>
            {
                let body = if matches!(tail.get(1), Some(SExpr::Atom(Atom::Keyword(keyword))) if keyword == "as")
                {
                    tail.get(3)
                } else {
                    tail.get(1)
                };
                body.is_some_and(|body| {
                    expression_is_typed_probability(
                        body,
                        types,
                        bindings,
                        probability_consts,
                        resolving,
                    )
                })
            }
            // Every arithmetic expression promotes bounded binary64 inputs
            // to Real; comparisons and all other governed forms likewise do
            // not produce Probability.
            _ => false,
        },
        SExpr::Atom(_) => false,
    }
}

fn validate_authored_event_payloads(
    rule: &SExpr,
    root_path: &[u32],
    types: &TypeEnv,
    bindings: &[BindingDecl],
    probability_consts: &std::collections::HashSet<String>,
) -> Result<(), ProbabilityError> {
    let mut stack = vec![(rule, root_path.to_vec())];
    while let Some((current, path)) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        let is_typed_emit = matches!(
            items.as_slice(),
            [
                SExpr::Atom(Atom::Symbol(form)),
                SExpr::Atom(Atom::EnumRef { .. }),
                ..
            ] if form == "emit"
        );
        if is_typed_emit {
            for (payload_index, payload_item) in items.iter().enumerate().skip(2) {
                let SExpr::List(pair) = payload_item else {
                    continue;
                };
                for (value_index, value) in pair.iter().enumerate().skip(1) {
                    if expression_is_typed_probability(
                        value,
                        types,
                        bindings,
                        probability_consts,
                        &mut Vec::new(),
                    ) {
                        let payload_path = child_path(&path, payload_index)?;
                        return Err(invalid(
                            &child_path(&payload_path, value_index)?,
                            "Probability cannot be authored in an event payload; event likelihood is derived from a finite kernel projection",
                        ));
                    }
                }
            }
        }
        stack.extend(
            semantic_children_with_paths(current, &path)?
                .into_iter()
                .rev(),
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StaticNumeric {
    Int(i64),
    Real(f64),
}

impl StaticNumeric {
    #[allow(clippy::cast_precision_loss)]
    fn as_f64(self) -> f64 {
        match self {
            Self::Int(value) => value as f64,
            Self::Real(value) => value,
        }
    }
}

fn static_numeric_expression(
    expr: &SExpr,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> Option<StaticNumeric> {
    match expr {
        SExpr::Atom(Atom::Int(value)) => Some(StaticNumeric::Int(*value)),
        SExpr::Atom(Atom::Scaled(value)) if value.kind != ScaledKind::Ratio => {
            #[allow(clippy::cast_precision_loss)]
            let numeric = value.unscaled as f64 / 10f64.powi(i32::from(value.scale));
            Some(StaticNumeric::Real(numeric))
        }
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return None;
            }
            let binding = bindings.iter().find(|binding| binding.name == *name)?;
            resolving.push(name.clone());
            let value = match &binding.source {
                BindSource::Const(qname) => match consts.get(qname) {
                    Some(Value::Int(value)) => Some(StaticNumeric::Int(*value)),
                    Some(Value::Real(value)) => Some(StaticNumeric::Real(*value)),
                    _ => None,
                },
                BindSource::Expr(source) => {
                    static_numeric_expression(source, bindings, consts, resolving)
                }
                BindSource::Field(_)
                | BindSource::Metric(_)
                | BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => None,
            };
            resolving.pop();
            value
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs]
                if matches!(op.as_str(), "+" | "-" | "*" | "/") =>
            {
                let lhs = static_numeric_expression(lhs, bindings, consts, resolving)?;
                let rhs = static_numeric_expression(rhs, bindings, consts, resolving)?;
                match (lhs, rhs) {
                    (StaticNumeric::Int(lhs), StaticNumeric::Int(rhs)) => match op.as_str() {
                        "+" => lhs.checked_add(rhs).map(StaticNumeric::Int),
                        "-" => lhs.checked_sub(rhs).map(StaticNumeric::Int),
                        "*" => lhs.checked_mul(rhs).map(StaticNumeric::Int),
                        // BSL does not define Int / Int.
                        "/" => None,
                        _ => unreachable!("the arithmetic-head guard admits only +, -, *, and /"),
                    },
                    (lhs, rhs) => {
                        let lhs = lhs.as_f64();
                        let rhs = rhs.as_f64();
                        if op == "/" && rhs == 0.0 {
                            return None;
                        }
                        let value = match op.as_str() {
                            "+" => lhs + rhs,
                            "-" => lhs - rhs,
                            "*" => lhs * rhs,
                            "/" => lhs / rhs,
                            _ => return None,
                        };
                        value.is_finite().then_some(StaticNumeric::Real(value))
                    }
                }
            }
            [SExpr::Atom(Atom::Symbol(form)), SExpr::Atom(Atom::Bool(condition)), then_value, else_value]
                if form == "if" =>
            {
                static_numeric_expression(
                    if *condition { then_value } else { else_value },
                    bindings,
                    consts,
                    resolving,
                )
            }
            _ => None,
        },
        SExpr::Atom(_) => None,
    }
}

fn static_mass_expression(
    expr: &SExpr,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    resolving: &mut Vec<String>,
) -> Result<Option<Mass>, ProbabilityError> {
    match expr {
        SExpr::Atom(Atom::Mass(mass)) => Ok(Some(*mass)),
        SExpr::Atom(Atom::Symbol(name)) => {
            if resolving.contains(name) {
                return Ok(None);
            }
            let Some(binding) = bindings.iter().find(|binding| binding.name == *name) else {
                return Ok(None);
            };
            resolving.push(name.clone());
            let value = match &binding.source {
                BindSource::Const(qname) => match consts.get(qname) {
                    Some(Value::Mass(mass)) => Some(*mass),
                    _ => None,
                },
                BindSource::Expr(source) => {
                    static_mass_expression(source, bindings, consts, resolving)?
                }
                BindSource::Field(_)
                | BindSource::Metric(_)
                | BindSource::Tick
                | BindSource::Year
                | BindSource::TickOfYear
                | BindSource::TickInCycle(_) => None,
            };
            resolving.pop();
            Ok(value)
        }
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs] if matches!(op.as_str(), "+" | "-") => {
                let Some(lhs) = static_mass_expression(lhs, bindings, consts, resolving)? else {
                    return Ok(None);
                };
                let Some(rhs) = static_mass_expression(rhs, bindings, consts, resolving)? else {
                    return Ok(None);
                };
                if op == "+" {
                    lhs.checked_add(rhs).map(Some)
                } else {
                    lhs.checked_sub(rhs).map(Some)
                }
            }
            [SExpr::Atom(Atom::Symbol(form)), operand] if form == "quantize-mass" => {
                let Some(value) = static_numeric_expression(operand, bindings, consts, resolving)
                else {
                    return Ok(None);
                };
                Mass::quantize(value.as_f64()).map(Some)
            }
            _ => Ok(None),
        },
        SExpr::Atom(_) => Ok(None),
    }
}

fn collect_mass_analysis_paths(
    expr: &SExpr,
    path: &[u32],
    literals: &mut Vec<MassLiteralFactV1>,
    quantize: &mut Vec<FormPath>,
) -> Result<(), ProbabilityError> {
    match expr {
        SExpr::Atom(Atom::Mass(mass)) => literals.push(MassLiteralFactV1 {
            form_path: path.to_vec(),
            mass: *mass,
        }),
        SExpr::List(items) => {
            if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(form))) if form == "quantize-mass")
            {
                quantize.push(child_path(path, 0)?);
            }
            for (child, child_path) in semantic_children_with_paths(expr, path)? {
                collect_mass_analysis_paths(child, &child_path, literals, quantize)?;
            }
        }
        SExpr::Atom(_) => {}
    }
    Ok(())
}

fn validate_quantize_mass_operands(
    rule: &SExpr,
    root_path: &[u32],
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
) -> Result<(), ProbabilityError> {
    let mut stack = vec![(rule, root_path.to_vec())];
    while let Some((current, path)) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(form))) if form == "quantize-mass")
        {
            let [_, operand] = items.as_slice() else {
                return Err(invalid(
                    &path,
                    "quantize-mass takes exactly one ordinary numeric operand",
                ));
            };
            if expression_contains_typed_mass(operand, types, bindings, consts, &mut Vec::new())
                || !ordinary_numeric_expression_is_typed(
                    operand,
                    types,
                    bindings,
                    consts,
                    &mut Vec::new(),
                )
            {
                return Err(invalid(
                    &child_path(&path, 1)?,
                    "quantize-mass operand must be statically Int/Real-lane numeric and cannot contain Mass",
                ));
            }
        }
        for (child, child_path) in semantic_children_with_paths(current, &path)?
            .into_iter()
            .rev()
        {
            stack.push((child, child_path));
        }
    }
    Ok(())
}

fn contains_forbidden_branch_form(expr: &SExpr) -> Option<&str> {
    let mut stack = vec![expr];
    while let Some(current) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        if let Some(SExpr::Atom(Atom::Symbol(form))) = items.first() {
            if matches!(form.as_str(), "emit" | "choose") {
                return Some(form);
            }
        }
        stack.extend(items.iter().rev());
    }
    None
}

fn projection_is_subject_local(rule: &SExpr, root_path: &[u32]) -> Result<bool, ProbabilityError> {
    const NONLOCAL: [&str; 14] = [
        "nodes",
        "edges",
        "hyperedges",
        "neighbors",
        "members-of",
        "hyperedges-of",
        "fold",
        "exists",
        "forall",
        "select-max",
        "select-min",
        "for-each",
        "edge-between",
        "the",
    ];
    let mut stack = vec![(rule, root_path.to_vec())];
    while let Some((current, path)) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        if let Some(SExpr::Atom(Atom::Symbol(form))) = items.first() {
            if NONLOCAL.contains(&form.as_str()) {
                return Ok(false);
            }
            if form == "field-of"
                && !matches!(items.get(1), Some(SExpr::Atom(Atom::Symbol(name))) if name == "self")
            {
                return Ok(false);
            }
        }
        stack.extend(
            semantic_children_with_paths(current, &path)?
                .into_iter()
                .rev(),
        );
    }
    Ok(true)
}

fn classify_kernel_projection_locality(
    rule: &SExpr,
    root_path: &[u32],
) -> Result<KernelProjectionLocalityV1, ProbabilityError> {
    const SHARED_MATERIAL_EFFECTS: [&str; 9] = [
        "update-edge",
        "update-hyperedge",
        "update-membership",
        "add-node",
        "remove-node",
        "add-edge",
        "remove-edge",
        "add-hyperedge",
        "remove-hyperedge",
    ];
    let mut stack = vec![(rule, root_path.to_vec())];
    while let Some((current, path)) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        if let Some(SExpr::Atom(Atom::Symbol(form))) = items.first() {
            if form == "update-node"
                && !matches!(items.get(1), Some(SExpr::Atom(Atom::Symbol(target))) if target == "self")
            {
                return Ok(KernelProjectionLocalityV1::RequiresJointCarrier {
                    message: "finite-projection kernel material effects must be carrier-local; update-node target must be literal `self`".to_owned(),
                    form_path: child_path(&path, 1)?,
                });
            }
            if SHARED_MATERIAL_EFFECTS.contains(&form.as_str()) {
                return Ok(KernelProjectionLocalityV1::RequiresJointCarrier {
                    message: "finite-projection kernel material effects must be carrier-local; shared or graph-shape writes require one joint kernel over that carrier and are not exactly enumerable in V1".to_owned(),
                    form_path: child_path(&path, 0)?,
                });
            }
        }
        stack.extend(
            semantic_children_with_paths(current, &path)?
                .into_iter()
                .rev(),
        );
    }
    Ok(KernelProjectionLocalityV1::CarrierLocal)
}

fn path_is_within(path: &[u32], root: &[u32]) -> bool {
    path.starts_with(root)
}

fn validate_mass_usage(
    rule: &SExpr,
    root_path: &[u32],
    kernel: Option<&FiniteKernelV1>,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
) -> Result<(), ProbabilityError> {
    let mut allowed_roots: Vec<FormPath> = kernel
        .into_iter()
        .flat_map(|kernel| {
            kernel
                .branches
                .iter()
                .map(|branch| branch.mass_path.clone())
        })
        .collect();
    let mass_bindings: std::collections::BTreeSet<&str> = bindings
        .iter()
        .filter(|binding| match &binding.source {
            BindSource::Const(qname) => matches!(consts.get(qname), Some(Value::Mass(_))),
            BindSource::Expr(expr) => {
                mass_expression_is_typed(expr, types, bindings, consts, &mut Vec::new())
            }
            _ => false,
        })
        .map(|binding| binding.name.as_str())
        .collect();

    let root = root_path.to_vec();
    let mut stack = vec![(rule, root.clone())];
    while let Some((current, path)) = stack.pop() {
        if let SExpr::List(items) = current {
            if head(current) == Some("binding") {
                if matches!(items.get(1), Some(SExpr::Atom(Atom::Symbol(name))) if mass_bindings.contains(name.as_str()))
                {
                    // The symbol in `(binding <name> ...)` declares the Mass
                    // binding; it is not a value-position use. Exempt this one
                    // token path only. Every reference to the same name remains
                    // subject to the second traversal below.
                    allowed_roots.push(child_path(&path, 1)?);
                }
                for (index, item) in items.iter().enumerate() {
                    if matches!(item, SExpr::Atom(Atom::Keyword(keyword)) if keyword == "expr") {
                        if let Some(expr) = items.get(index + 1) {
                            if mass_expression_is_typed(
                                expr,
                                types,
                                bindings,
                                consts,
                                &mut Vec::new(),
                            ) {
                                allowed_roots.push(child_path(&path, index + 1)?);
                            }
                        }
                    }
                }
            }
        }
        for (child, child_path) in semantic_children_with_paths(current, &path)?
            .into_iter()
            .rev()
        {
            stack.push((child, child_path));
        }
    }

    let mut stack = vec![(rule, root)];
    while let Some((current, path)) = stack.pop() {
        if allowed_roots
            .iter()
            .any(|allowed| path_is_within(&path, allowed))
        {
            continue;
        }
        match current {
            SExpr::Atom(Atom::Mass(_)) => {
                return Err(invalid(
                    &path,
                    "Mass is transient allocation input and is not legal in this rule position",
                ));
            }
            SExpr::Atom(Atom::Symbol(name)) if mass_bindings.contains(name.as_str()) => {
                return Err(invalid(
                    &path,
                    "a Mass binding is legal only in another Mass binding or branch :mass expression",
                ));
            }
            SExpr::List(items) if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(form))) if form == "quantize-mass") =>
            {
                return Err(invalid(
                    &child_path(&path, 0)?,
                    "quantize-mass is legal only in a Mass binding or branch :mass expression",
                ));
            }
            SExpr::List(_) => stack.extend(
                semantic_children_with_paths(current, &path)?
                    .into_iter()
                    .rev(),
            ),
            SExpr::Atom(_) => {}
        }
    }
    Ok(())
}

fn parse_branch(
    expr: &SExpr,
    path: FormPath,
    enums: &EnumRegistry,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
) -> Result<(EnumTypeId, KernelBranchV1), ProbabilityError> {
    let SExpr::List(items) = expr else {
        return Err(invalid(
            &path,
            "a choose body contains only (branch ...) forms",
        ));
    };
    let [SExpr::Atom(Atom::Symbol(branch)), SExpr::Atom(Atom::EnumRef { enum_type, member }), SExpr::Atom(Atom::Keyword(mass_keyword)), mass, SExpr::List(effect_form)] =
        items.as_slice()
    else {
        return Err(invalid(
            &path,
            "canonical branch surface is (branch <EnumType/MEMBER> :mass <mass-expr> (effects <effect-item>*))",
        ));
    };
    if branch != "branch" || mass_keyword != "mass" {
        return Err(invalid(
            &path,
            "canonical branch surface requires the `branch` head and `:mass` keyword",
        ));
    }
    if !matches!(effect_form.first(), Some(SExpr::Atom(Atom::Symbol(form))) if form == "effects") {
        return Err(invalid(
            &path,
            "a branch body must be one (effects ...) form",
        ));
    }
    if !mass_expression_is_typed(mass, types, bindings, consts, &mut Vec::new()) {
        return Err(invalid(
            &child_path(&path, 3)?,
            "branch :mass must have static Mass type; use an m literal, Mass binding, checked +/- or quantize-mass",
        ));
    }
    for effect in effect_form.iter().skip(1) {
        if let Some(forbidden) = contains_forbidden_branch_form(effect) {
            return Err(invalid(
                &path,
                format!("a branch body cannot contain `{forbidden}`"),
            ));
        }
    }
    let Some(enum_id) = enums.resolve(enum_type) else {
        return Err(invalid(
            &child_path(&path, 1)?,
            format!("choose branch names undeclared defenum type `{enum_type}`"),
        ));
    };
    let Some(ordinal) = enums.ordinal(enum_id, member) else {
        return Err(invalid(
            &child_path(&path, 1)?,
            format!("choose branch names unknown `{enum_type}/{member}`"),
        ));
    };
    let mass_path = child_path(&path, 3)?;
    let static_mass = static_mass_expression(mass, bindings, consts, &mut Vec::new())?;
    Ok((
        enum_id,
        KernelBranchV1 {
            enum_type: enum_type.clone(),
            member: member.clone(),
            ordinal,
            mass: mass.clone(),
            effects: effect_form.iter().skip(1).cloned().collect(),
            head_path: child_path(&path, 0)?,
            mass_path,
            mass_literals: Vec::new(),
            quantize_mass_paths: Vec::new(),
            static_mass,
            form_path: path,
        },
    ))
}

fn parse_choose(
    expr: &SExpr,
    path: FormPath,
    enums: &EnumRegistry,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
) -> Result<FiniteKernelV1, ProbabilityError> {
    let SExpr::List(items) = expr else {
        return Err(invalid(&path, "choose must be a list form"));
    };
    let [SExpr::Atom(Atom::Symbol(choose)), SExpr::Atom(Atom::Keyword(sample_keyword)), SExpr::Atom(Atom::QName(sample)), SExpr::Atom(Atom::Keyword(slot_keyword)), SExpr::Atom(Atom::Int(slot)), branch_forms @ ..] =
        items.as_slice()
    else {
        return Err(invalid(
            &path,
            "canonical choose surface is (choose :sample <sample-qname> :slot <u32-int> (branch ...)+)",
        ));
    };
    if choose != "choose" || sample_keyword != "sample" || slot_keyword != "slot" {
        return Err(invalid(
            &path,
            "choose options must be exactly `:sample` then `:slot`",
        ));
    }
    let slot = u32::try_from(*slot).map_err(|_| {
        invalid(
            &child_path(&path, 4).unwrap_or_else(|_| path.clone()),
            "choose :slot must be a non-negative u32 literal",
        )
    })?;
    if branch_forms.is_empty() {
        return Err(invalid(&path, "choose requires at least one branch"));
    }
    let mut branches = Vec::with_capacity(branch_forms.len());
    let mut common_enum = None;
    for (offset, branch_form) in branch_forms.iter().enumerate() {
        let branch_path = child_path(&path, offset + 5)?;
        let (enum_id, branch) =
            parse_branch(branch_form, branch_path, enums, types, bindings, consts)?;
        if common_enum.is_some_and(|existing| existing != enum_id) {
            return Err(invalid(
                &branch.form_path,
                "every choose branch must use one common defenum type",
            ));
        }
        common_enum = Some(enum_id);
        branches.push(branch);
    }
    let enum_type = common_enum.ok_or_else(|| invalid(&path, "choose requires a branch"))?;
    let declaration = enums.declaration(enum_type).ok_or_else(|| {
        invalid(
            &path,
            "choose branch enum no longer resolves in the loaded registry",
        )
    })?;
    if branches.len() != declaration.members.len()
        || branches
            .iter()
            .zip(&declaration.members)
            .any(|(branch, expected)| branch.member != *expected)
    {
        return Err(invalid(
            &path,
            format!(
                "choose branches must exhaust defenum `{}` exactly once in declaration order: {:?}",
                declaration.name, declaration.members
            ),
        ));
    }
    Ok(FiniteKernelV1 {
        sample: sample.clone(),
        sample_path: child_path(&path, 2)?,
        slot,
        slot_path: child_path(&path, 4)?,
        enum_type,
        enum_type_name: declaration.name.clone(),
        branches,
        head_path: child_path(&path, 0)?,
        form_path: path,
    })
}

/// Compile the one canonical finite-kernel/projection surface of a rule.
///
/// # Errors
///
/// Returns [`ProbabilityError`] when the canonical surface, role, Mass type,
/// projection locality, or retained source paths violate the V1 contract.
#[allow(
    clippy::implicit_hasher,
    clippy::too_many_arguments,
    clippy::too_many_lines
)]
pub fn compile_rule_probability(
    rule: &SExpr,
    root_path: &[u32],
    contract: &RuleContract,
    enums: &EnumRegistry,
    types: &TypeEnv,
    bindings: &[BindingDecl],
    consts: &std::collections::HashMap<String, Value>,
    probability_consts: &std::collections::HashSet<String>,
) -> Result<CompiledRuleProbabilityV1, ProbabilityError> {
    let SExpr::List(rule_items) = rule else {
        return Err(invalid(
            root_path,
            "probability analysis requires a rule form",
        ));
    };
    let root = root_path.to_vec();
    validate_quantize_mass_operands(rule, &root, types, bindings, consts)?;
    validate_authored_event_payloads(rule, &root, types, bindings, probability_consts)?;
    let mut effects = None;
    let mut projection = None;
    for (index, item) in rule_items.iter().enumerate() {
        if head(item) == Some("effects") {
            effects = Some((item, child_path(&root, index)?));
        }
        if matches!(item, SExpr::Atom(Atom::Keyword(keyword)) if keyword == "projects-kernel") {
            if projection.is_some() {
                return Err(invalid(
                    &child_path(&root, index)?,
                    ":projects-kernel may appear at most once",
                ));
            }
            let Some(SExpr::Atom(Atom::QName(sample))) = rule_items.get(index + 1) else {
                return Err(invalid(
                    &child_path(&root, index)?,
                    ":projects-kernel takes one sample qname",
                ));
            };
            projection = Some(FiniteProjectionV1 {
                sample: sample.clone(),
                form_path: child_path(&root, index)?,
                sample_path: child_path(&root, index + 1)?,
                event_types: Vec::new(),
            });
        }
    }

    let mut all_choose_paths = Vec::new();
    let mut stack = vec![(rule, root.clone())];
    while let Some((current, path)) = stack.pop() {
        let SExpr::List(items) = current else {
            continue;
        };
        if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(form))) if form == "choose") {
            all_choose_paths.push(path.clone());
        }
        for (child, child_path) in semantic_children_with_paths(current, &path)?
            .into_iter()
            .rev()
        {
            stack.push((child, child_path));
        }
    }
    if all_choose_paths.len() > 1 {
        return Err(invalid(
            &all_choose_paths[1],
            "one Mechanic rule may contain at most one choose",
        ));
    }

    let direct_choose = if let Some((effect_form, effect_path)) = effects {
        let SExpr::List(items) = effect_form else {
            return Err(invalid(&effect_path, "effects must be a list form"));
        };
        let mut direct = None;
        for (index, item) in items.iter().enumerate().skip(1) {
            if head(item) == Some("choose") {
                direct = Some((item, child_path(&effect_path, index)?));
                break;
            }
        }
        direct
    } else {
        None
    };
    if let Some(path) = all_choose_paths.first() {
        if direct_choose
            .as_ref()
            .is_none_or(|(_, direct)| direct != path)
        {
            return Err(invalid(
                path,
                "choose must be one direct child of the rule's (effects ...) form",
            ));
        }
    }
    if let Some((_, direct_path)) = &direct_choose {
        if contract.role != RuleRole::Mechanic {
            return Err(invalid(
                direct_path,
                "only a Mechanic rule may contain choose",
            ));
        }
    }
    if let Some(compiled_projection) = &projection {
        if contract.role != RuleRole::Recognizer {
            return Err(invalid(
                &compiled_projection.form_path,
                ":projects-kernel is legal only on a Recognizer",
            ));
        }
    }
    if let Some(compiled_projection) = projection.as_mut() {
        if bindings
            .iter()
            .any(|binding| matches!(binding.source, BindSource::Metric(_)))
        {
            return Err(invalid(
                &compiled_projection.form_path,
                "a finite projection cannot bind a graph-global :metric source",
            ));
        }
        let footprint =
            effect_footprint(rule).map_err(|error| invalid(&root, error.to_string()))?;
        if !footprint
            .iter()
            .all(|effect| matches!(effect, EffectSignature::Event(_)))
        {
            return Err(invalid(
                &compiled_projection.form_path,
                "a finite projection recognizer is emit-only",
            ));
        }
        if !projection_is_subject_local(rule, &root)? {
            return Err(invalid(
                &compiled_projection.form_path,
                "a finite projection recognizer must be subject-local",
            ));
        }
        for effect in footprint {
            let EffectSignature::Event(event_type) = effect else {
                return Err(invalid(
                    &compiled_projection.form_path,
                    "a finite projection recognizer is emit-only",
                ));
            };
            let member = event_type.strip_prefix("EventType/").ok_or_else(|| {
                invalid(
                    &compiled_projection.form_path,
                    "a finite projection event must use the EventType namespace",
                )
            })?;
            if !compiled_projection
                .event_types
                .iter()
                .any(|declared| declared == member)
            {
                compiled_projection.event_types.push(member.to_owned());
            }
        }
    }
    let mut kernel = direct_choose
        .map(|(form, path)| parse_choose(form, path, enums, types, bindings, consts))
        .transpose()?;
    validate_mass_usage(rule, &root, kernel.as_ref(), types, bindings, consts)?;
    let kernel_projection_locality = kernel
        .as_ref()
        .map(|_| classify_kernel_projection_locality(rule, &root))
        .transpose()?;

    // Retain every authoring fact now, while the loader owns the one raw
    // form. `analyze_content_set` consumes this record and never performs a
    // later semantic S-expression walk.
    let mut mass_literals = Vec::new();
    let mut quantize_mass_paths = Vec::new();
    collect_mass_analysis_paths(rule, &root, &mut mass_literals, &mut quantize_mass_paths)?;
    if let Some(kernel) = &mut kernel {
        for branch in &mut kernel.branches {
            branch.mass_literals = mass_literals
                .iter()
                .filter(|literal| literal.form_path.starts_with(&branch.mass_path))
                .cloned()
                .collect();
            branch.quantize_mass_paths = quantize_mass_paths
                .iter()
                .filter(|path| path.starts_with(&branch.mass_path))
                .cloned()
                .collect();
        }
    }
    let mut nodes = Vec::new();
    if let Some(kernel) = &kernel {
        nodes.push(ProbabilityAnalysisNodeV1 {
            kind: ProbabilityAnalysisNodeKindV1::Choose,
            form_path: kernel.head_path.clone(),
        });
        for branch in &kernel.branches {
            nodes.push(ProbabilityAnalysisNodeV1 {
                kind: ProbabilityAnalysisNodeKindV1::Branch,
                form_path: branch.head_path.clone(),
            });
            nodes.push(ProbabilityAnalysisNodeV1 {
                kind: ProbabilityAnalysisNodeKindV1::BranchMass,
                form_path: branch.mass_path.clone(),
            });
        }
    }
    if let Some(projection) = &projection {
        nodes.push(ProbabilityAnalysisNodeV1 {
            kind: ProbabilityAnalysisNodeKindV1::ProjectionKeyword,
            form_path: projection.form_path.clone(),
        });
        nodes.push(ProbabilityAnalysisNodeV1 {
            kind: ProbabilityAnalysisNodeKindV1::ProjectionSample,
            form_path: projection.sample_path.clone(),
        });
    }
    nodes.extend(
        mass_literals
            .iter()
            .map(|literal| ProbabilityAnalysisNodeV1 {
                kind: ProbabilityAnalysisNodeKindV1::MassLiteral,
                form_path: literal.form_path.clone(),
            }),
    );
    nodes.extend(
        quantize_mass_paths
            .into_iter()
            .map(|form_path| ProbabilityAnalysisNodeV1 {
                kind: ProbabilityAnalysisNodeKindV1::QuantizeMass,
                form_path,
            }),
    );
    nodes.sort_by(|left, right| left.form_path.cmp(&right.form_path));
    Ok(CompiledRuleProbabilityV1 {
        kernel,
        projection,
        facts: CompiledProbabilityFactsV1 {
            nodes,
            mass_literals,
            kernel_projection_locality,
        },
    })
}

/// Validate sample uniqueness and immediate kernel/projection adjacency in
/// resolved execution order. The caller supplies already scheduled rules.
///
/// # Errors
///
/// Returns [`ProbabilityError::DuplicateSample`] for a repeated sample,
/// [`ProbabilityError::ProjectionNotAdjacent`] for a nonadjacent projection,
/// or [`ProbabilityError::SubjectCarrierMismatch`] when an adjacent pair
/// resolves different runtime populations. An adjacent, carrier-matched pair
/// also returns a located [`ProbabilityError::InvalidForm`] when the kernel's
/// material effects are not carrier-local enough for exact V1 projection.
pub fn validate_probability_content_set(
    rules: &[crate::rule_pipeline::LoadedRule],
) -> Result<(), ProbabilityError> {
    let mut samples = std::collections::BTreeSet::new();
    for rule in rules {
        let Some(kernel) = &rule.kernel else {
            continue;
        };
        if !samples.insert(kernel.sample.as_str()) {
            return Err(ProbabilityError::DuplicateSample {
                sample: kernel.sample.clone(),
                rule_id: rule.contract.rule_id.clone(),
                form_path: kernel.sample_path.clone(),
            });
        }
    }
    for (index, projection_rule) in rules.iter().enumerate() {
        let Some(projection) = &projection_rule.projection else {
            continue;
        };
        let adjacent = index
            .checked_sub(1)
            .and_then(|previous| rules.get(previous));
        let Some((kernel_rule, kernel)) = adjacent.and_then(|rule| {
            rule.kernel
                .as_ref()
                .filter(|kernel| kernel.sample == projection.sample)
                .map(|kernel| (rule, kernel))
        }) else {
            return Err(ProbabilityError::ProjectionNotAdjacent {
                sample: projection.sample.clone(),
                rule_id: projection_rule.contract.rule_id.clone(),
                form_path: projection.sample_path.clone(),
            });
        };
        let kernel_carrier = kernel_rule.probability_carrier.as_deref().ok_or_else(|| {
            invalid(
                &kernel.sample_path,
                "finite kernel has no unambiguous resolved subject carrier",
            )
        })?;
        let projection_carrier =
            projection_rule
                .probability_carrier
                .as_deref()
                .ok_or_else(|| {
                    invalid(
                        &projection.sample_path,
                        "finite projection has no unambiguous resolved subject carrier",
                    )
                })?;
        if kernel_carrier != projection_carrier {
            return Err(ProbabilityError::SubjectCarrierMismatch(Box::new(
                SubjectCarrierMismatchV1 {
                    sample: projection.sample.clone(),
                    kernel_rule_id: kernel_rule.contract.rule_id.clone(),
                    projection_rule_id: projection_rule.contract.rule_id.clone(),
                    kernel_carrier: kernel_carrier.to_owned(),
                    projection_carrier: projection_carrier.to_owned(),
                    form_path: projection.sample_path.clone(),
                },
            )));
        }
        let locality = kernel_rule
            .probability_facts
            .kernel_projection_locality
            .as_ref()
            .ok_or_else(|| {
                invalid(
                    &kernel.sample_path,
                    "compiled finite kernel has no retained projection-locality result",
                )
            })?;
        if let KernelProjectionLocalityV1::RequiresJointCarrier { message, form_path } = locality {
            return Err(invalid(form_path, message.clone()));
        }
    }
    Ok(())
}

/// Evaluate every branch Mass in enum order before any selection occurs.
///
/// This is the single runtime Mass-evaluation path used by realization and
/// detached forecasting. Returning anything but `Value::Mass` is a loud
/// compiler/runtime invariant failure.
///
/// # Errors
///
/// Returns [`EvalError`] when a Mass expression exhausts fuel, cannot be
/// evaluated, or violates the compiled Mass type invariant.
pub fn evaluate_kernel_masses(
    kernel: &FiniteKernelV1,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Vec<Mass>, EvalError> {
    let mut masses = Vec::with_capacity(kernel.branches.len());
    for branch in &kernel.branches {
        match evaluate(&branch.mass, env, host, fuel)? {
            Value::Mass(mass) => masses.push(mass),
            other => {
                return Err(EvalError::plain(format!(
                    "compiled branch Mass evaluated to non-Mass value {other:?}"
                )))
            }
        }
    }
    Ok(masses)
}

/// Analyze one already resolved content-set schedule through retained typed IR.
///
/// The LSP maps returned [`FormPath`] values through `SpanTable`; it does not
/// perform a second semantic walk over raw S-expressions.
///
/// # Errors
///
/// Returns [`ProbabilityError`] when schedule linkage, exact static
/// allocation, or a retained source path violates the loaded contract.
#[allow(clippy::too_many_lines)]
pub fn analyze_content_set(
    rules: &[crate::rule_pipeline::LoadedRule],
) -> Result<ContentSetAnalysisV1, ProbabilityError> {
    validate_probability_content_set(rules)?;

    let mut analyzed = Vec::with_capacity(rules.len());
    for rule in rules {
        let nodes = rule.probability_facts.nodes.clone();
        let allocation = if let Some(kernel) = &rule.kernel {
            let masses: Option<Vec<Mass>> = kernel
                .branches
                .iter()
                .map(|branch| branch.static_mass)
                .collect();
            Some(match masses {
                Some(masses) => {
                    let intervals = allocate_tickets(&masses)?;
                    AllocationAnalysisV1::Exact(ExactKernelAllocationV1 { masses, intervals })
                }
                None => AllocationAnalysisV1::Unavailable {
                    reason: "one or more masses depend on runtime state or calendar, or use a source expression outside the load-time static evaluator"
                        .to_owned(),
                },
            })
        } else {
            None
        };
        analyzed.push(RuleProbabilityAnalysisV1 {
            source_id: rule.source_id.clone(),
            rule_id: rule.contract.rule_id.clone(),
            kernel: rule.kernel.clone(),
            projection: rule.projection.clone(),
            nodes,
            mass_literals: rule.probability_facts.mass_literals.clone(),
            allocation,
        });
    }

    let mut links = Vec::new();
    for pair in rules.windows(2) {
        let [kernel_rule, projection_rule] = pair else {
            continue;
        };
        let (Some(kernel), Some(projection)) = (&kernel_rule.kernel, &projection_rule.projection)
        else {
            continue;
        };
        if kernel.sample == projection.sample {
            links.push(KernelProjectionLinkV1 {
                sample: kernel.sample.clone(),
                kernel_rule_id: kernel_rule.contract.rule_id.clone(),
                projection_rule_id: projection_rule.contract.rule_id.clone(),
                likelihood: LikelihoodAnalysisV1::StateDependent {
                    reason: "exact event likelihood requires the bounded pre-choice state"
                        .to_owned(),
                },
            });
        }
    }
    Ok(ContentSetAnalysisV1 {
        mass_declarations: Vec::new(),
        rules: analyzed,
        links,
    })
}

/// Push one enum-ordered deterministic branch projection through the exact
/// ticket measure. The detached-state executor supplies `projections`; this
/// reducer never samples and never assumes independence.
///
/// # Errors
///
/// Returns [`ProbabilityError`] when samples, branch order, projected events,
/// or the exact ticket allocation do not match the compiled pair.
pub fn forecast_event_likelihoods(
    kernel: &FiniteKernelV1,
    projection: &FiniteProjectionV1,
    masses: &[Mass],
    projections: &[BranchProjectionV1],
) -> Result<Vec<EventLikelihoodV1>, ProbabilityError> {
    if projection.sample != kernel.sample {
        return Err(invalid(
            &projection.sample_path,
            "forecast projection sample must match the finite kernel sample",
        ));
    }
    if masses.len() != kernel.branches.len() || projections.len() != kernel.branches.len() {
        return Err(invalid(
            &kernel.form_path,
            "forecast inputs must contain exactly one mass and projection per kernel branch",
        ));
    }
    for (branch, projection) in kernel.branches.iter().zip(projections) {
        if branch.member != projection.outcome {
            return Err(invalid(
                &branch.form_path,
                "forecast projections must follow the kernel's enum declaration order",
            ));
        }
    }
    let allocation = allocate_tickets(masses)?;
    let mut favorable: std::collections::BTreeMap<String, (Vec<String>, u128)> = projection
        .event_types
        .iter()
        .cloned()
        .map(|event_type| (event_type, (Vec::new(), 0)))
        .collect();
    for ((branch, projection), interval) in kernel.branches.iter().zip(projections).zip(&allocation)
    {
        let mut unique = std::collections::BTreeSet::new();
        for event_type in &projection.event_types {
            if unique.insert(event_type.as_str()) {
                let Some(row) = favorable.get_mut(event_type) else {
                    return Err(invalid(
                        &kernel.form_path,
                        format!(
                            "forecast branch emitted undeclared projection event `{event_type}`"
                        ),
                    ));
                };
                row.0.push(branch.member.clone());
                row.1 = row
                    .1
                    .checked_add(interval.count)
                    .ok_or(ProbabilityError::TotalMassOverflow)?;
            }
        }
    }
    Ok(favorable
        .into_iter()
        .map(
            |(event_type, (favorable_outcomes, numerator))| EventLikelihoodV1 {
                event_type,
                favorable_outcomes,
                numerator,
                denominator: TICKET_DENOMINATOR,
            },
        )
        .collect())
}

/// Allocate the complete `2^64` ticket measure using exact largest remainder.
///
/// # Errors
///
/// Returns [`ProbabilityError`] for zero total mass, arithmetic overflow, or
/// a positive branch that cannot receive a ticket.
pub fn allocate_tickets(masses: &[Mass]) -> Result<Vec<TicketIntervalV1>, ProbabilityError> {
    let total = checked_allocation_total(masses.iter().map(|mass| u128::from(mass.nanounits())))?;
    if total == 0 {
        return Err(ProbabilityError::ZeroTotalMass);
    }

    let mut counts = Vec::with_capacity(masses.len());
    let mut remainders = Vec::with_capacity(masses.len());
    let mut assigned = 0_u128;
    for mass in masses {
        let numerator = u128::from(mass.nanounits()) * TICKET_DENOMINATOR;
        let count = numerator / total;
        let remainder = numerator % total;
        counts.push(count);
        remainders.push(remainder);
        assigned = assigned
            .checked_add(count)
            .ok_or(ProbabilityError::TotalMassOverflow)?;
    }
    let remaining = TICKET_DENOMINATOR
        .checked_sub(assigned)
        .ok_or(ProbabilityError::TotalMassOverflow)?;
    let mut order: Vec<usize> = (0..masses.len()).collect();
    order.sort_by(|left, right| {
        remainders[*right]
            .cmp(&remainders[*left])
            .then_with(|| left.cmp(right))
    });
    let remaining = usize::try_from(remaining).map_err(|_| ProbabilityError::TotalMassOverflow)?;
    if remaining > order.len() {
        return Err(ProbabilityError::TotalMassOverflow);
    }
    for index in order.into_iter().take(remaining) {
        counts[index] += 1;
    }
    for (index, (mass, count)) in masses.iter().zip(&counts).enumerate() {
        if mass.nanounits() > 0 && *count == 0 {
            return Err(ProbabilityError::PositiveMassHasZeroTickets { index });
        }
    }

    let mut cursor = 0_u128;
    let mut intervals = Vec::with_capacity(counts.len());
    for count in counts {
        let end = cursor
            .checked_add(count)
            .ok_or(ProbabilityError::TotalMassOverflow)?;
        intervals.push(TicketIntervalV1 {
            start: cursor,
            end,
            count,
        });
        cursor = end;
    }
    if cursor != TICKET_DENOMINATOR {
        return Err(ProbabilityError::TotalMassOverflow);
    }
    Ok(intervals)
}

fn checked_allocation_total(
    nanounit_terms: impl IntoIterator<Item = u128>,
) -> Result<u128, ProbabilityError> {
    nanounit_terms.into_iter().try_fold(0_u128, |sum, term| {
        sum.checked_add(term)
            .ok_or(ProbabilityError::TotalMassOverflow)
    })
}

/// Resolve one `u64` ticket to its enum-ordered branch index.
///
/// # Errors
///
/// Returns [`ProbabilityError::TicketNotCovered`] when the supplied
/// allocation does not cover the draw.
pub fn selected_branch(
    allocation: &[TicketIntervalV1],
    draw: u64,
) -> Result<usize, ProbabilityError> {
    let ticket = u128::from(draw);
    allocation
        .iter()
        .position(|interval| interval.start <= ticket && ticket < interval.end)
        .ok_or(ProbabilityError::TicketNotCovered { draw })
}

fn push_len_prefixed(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProbabilityError> {
    let len = u32::try_from(value.len()).map_err(|_| ProbabilityError::MassOverflow)?;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

fn push_bytes_len_prefixed(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ProbabilityError> {
    let len = u32::try_from(value.len()).map_err(|_| ProbabilityError::MassOverflow)?;
    bytes.extend_from_slice(&len.to_be_bytes());
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_stable_identity(
    bytes: &mut Vec<u8>,
    value: &StableElementKeyV1,
) -> Result<(), ProbabilityError> {
    let encoded = value.canonical_bytes().map_err(|error| {
        invalid(
            &[],
            format!("stable choice carrier identity refused: {error:?}"),
        )
    })?;
    push_bytes_len_prefixed(bytes, &encoded)
}

/// Build the complete engine-neutral realization record from one private draw.
///
/// # Errors
///
/// Returns [`ProbabilityError`] when the evaluated masses do not match the
/// kernel, cannot be allocated exactly, or stable identity encoding fails.
#[allow(clippy::too_many_arguments)]
pub fn realize_kernel(
    identity: &KernelInstanceIdentityV1,
    kernel: &FiniteKernelV1,
    masses: &[Mass],
    draw: u64,
) -> Result<KernelRealizationV1, ProbabilityError> {
    if masses.len() != kernel.branches.len() {
        return Err(ProbabilityError::InvalidForm {
            message: "evaluated mass count does not match compiled branch count".to_owned(),
            form_path: kernel.form_path.clone(),
        });
    }
    let intervals = allocate_tickets(masses)?;
    let branches: Vec<RealizedBranchV1> = kernel
        .branches
        .iter()
        .zip(masses)
        .zip(intervals)
        .map(|((branch, mass), tickets)| RealizedBranchV1 {
            member: branch.member.clone(),
            mass: *mass,
            tickets,
        })
        .collect();

    compose_realization(identity, kernel, branches, draw)
}

fn compose_realization(
    identity: &KernelInstanceIdentityV1,
    kernel: &FiniteKernelV1,
    branches: Vec<RealizedBranchV1>,
    draw: u64,
) -> Result<KernelRealizationV1, ProbabilityError> {
    let selected = selected_branch(
        &branches
            .iter()
            .map(|branch| branch.tickets.clone())
            .collect::<Vec<_>>(),
        draw,
    )?;
    let mut allocation_bytes = b"babylon.finite-kernel-allocation.v1\0".to_vec();
    push_len_prefixed(&mut allocation_bytes, &kernel.enum_type_name)?;
    allocation_bytes.extend_from_slice(
        &u32::try_from(branches.len())
            .map_err(|_| ProbabilityError::MassOverflow)?
            .to_be_bytes(),
    );
    for branch in &branches {
        push_len_prefixed(&mut allocation_bytes, &branch.member)?;
        allocation_bytes.extend_from_slice(&branch.mass.nanounits().to_be_bytes());
        allocation_bytes.extend_from_slice(&branch.tickets.start.to_be_bytes());
        allocation_bytes.extend_from_slice(&branch.tickets.end.to_be_bytes());
    }
    let allocation_digest = sha256_of(&allocation_bytes);

    let mut instance_bytes = b"babylon.finite-kernel-instance.v1\0".to_vec();
    push_bytes_len_prefixed(&mut instance_bytes, &identity.replay_session)?;
    instance_bytes.extend_from_slice(&identity.replay_seed);
    instance_bytes.extend_from_slice(&identity.tick.to_be_bytes());
    push_len_prefixed(&mut instance_bytes, &identity.rule_id)?;
    push_len_prefixed(&mut instance_bytes, &kernel.sample)?;
    instance_bytes.extend_from_slice(&kernel.slot.to_be_bytes());
    push_stable_identity(&mut instance_bytes, &identity.subject)?;
    instance_bytes.extend_from_slice(
        &u32::try_from(identity.active_elements.len())
            .map_err(|_| ProbabilityError::MassOverflow)?
            .to_be_bytes(),
    );
    for element in &identity.active_elements {
        push_stable_identity(&mut instance_bytes, element)?;
    }
    instance_bytes.extend_from_slice(&allocation_digest);
    let instance_digest = sha256_of(&instance_bytes);

    Ok(KernelRealizationV1 {
        rule_id: identity.rule_id.clone(),
        sample: kernel.sample.clone(),
        slot: kernel.slot,
        enum_type: kernel.enum_type_name.clone(),
        subject: identity.subject.clone(),
        active_elements: identity.active_elements.clone(),
        branches,
        draw,
        selected_outcome: kernel.branches[selected].member.clone(),
        allocation_digest,
        instance_digest,
    })
}

/// Recompute and validate all allocation and instance facts of a retained
/// realization. Persistence uses this seam instead of duplicating codecs.
///
/// # Errors
///
/// Returns [`ProbabilityError`] when any stored mass, interval, outcome,
/// identity field, or digest fails exact recomposition.
pub fn validate_kernel_realization(
    realization: &KernelRealizationV1,
    identity: &KernelInstanceIdentityV1,
) -> Result<(), ProbabilityError> {
    let enum_type = EnumTypeId(0);
    let mut branches = Vec::with_capacity(realization.branches.len());
    for (ordinal, branch) in realization.branches.iter().enumerate() {
        branches.push(KernelBranchV1 {
            enum_type: realization.enum_type.clone(),
            member: branch.member.clone(),
            ordinal: u32::try_from(ordinal).map_err(|_| {
                invalid(
                    &[],
                    "stored realization branch count exceeds the u32 ordinal lane",
                )
            })?,
            mass: SExpr::Atom(Atom::Mass(branch.mass)),
            effects: Vec::new(),
            head_path: Vec::new(),
            mass_path: Vec::new(),
            mass_literals: Vec::new(),
            quantize_mass_paths: Vec::new(),
            static_mass: Some(branch.mass),
            form_path: Vec::new(),
        });
    }
    let kernel = FiniteKernelV1 {
        sample: realization.sample.clone(),
        sample_path: Vec::new(),
        slot: realization.slot,
        slot_path: Vec::new(),
        enum_type,
        enum_type_name: realization.enum_type.clone(),
        branches,
        head_path: Vec::new(),
        form_path: Vec::new(),
    };
    let masses: Vec<Mass> = realization
        .branches
        .iter()
        .map(|branch| branch.mass)
        .collect();
    let intervals = allocate_tickets(&masses)?;
    if intervals
        .iter()
        .zip(&realization.branches)
        .any(|(expected, branch)| expected != &branch.tickets)
    {
        return Err(invalid(
            &[],
            "stored realization ticket intervals do not recompose",
        ));
    }
    let recomposed = compose_realization(
        identity,
        &kernel,
        realization.branches.clone(),
        realization.draw,
    )?;
    if &recomposed == realization {
        Ok(())
    } else {
        Err(invalid(
            &[],
            "stored realization identity or digest does not recompose",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn quantize_mass_rounds_the_exact_binary64_rational_with_ties_to_even() {
        // These are exact binary fractions whose products by 10^9 are exact
        // half-integers. 976562 is even; 2929687 is odd.
        assert_eq!(Mass::quantize(1.0 / 1_024.0).unwrap().nanounits(), 976_562);
        assert_eq!(
            Mass::quantize(3.0 / 1_024.0).unwrap().nanounits(),
            2_929_688
        );

        // The nearest binary64 values on either side of one half nanounit
        // must not be double-rounded by a floating-point scale operation.
        let half_nanounit = 0.5 / MASS_NANOUNITS_PER_UNIT as f64;
        let below = f64::from_bits(half_nanounit.to_bits() - 1);
        let above = f64::from_bits(half_nanounit.to_bits() + 1);
        assert_eq!(Mass::quantize(below).unwrap().nanounits(), 0);
        assert_eq!(Mass::quantize(above).unwrap().nanounits(), 1);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn quantize_mass_checks_the_last_representable_binary64_values_near_u64_overflow() {
        let first_obvious_overflow = TICKET_DENOMINATOR as f64 / MASS_NANOUNITS_PER_UNIT as f64;
        let previous = f64::from_bits(first_obvious_overflow.to_bits() - 1);
        assert!(Mass::quantize(previous).is_ok());
        assert_eq!(
            Mass::quantize(first_obvious_overflow),
            Err(ProbabilityError::MassOverflow)
        );
    }

    #[test]
    fn positive_mass_cannot_disappear_from_the_ticket_measure() {
        let masses = [
            Mass::from_nanounits(u64::MAX),
            Mass::from_nanounits(u64::MAX),
            Mass::from_nanounits(1),
        ];
        assert_eq!(
            allocate_tickets(&masses),
            Err(ProbabilityError::PositiveMassHasZeroTickets { index: 2 })
        );
    }

    #[test]
    fn allocation_accumulation_refuses_wide_integer_overflow() {
        assert_eq!(
            checked_allocation_total([u128::MAX, 1]),
            Err(ProbabilityError::TotalMassOverflow)
        );
    }

    #[test]
    fn instance_digest_binds_the_instance_key_and_allocation_not_the_realized_draw() {
        let mut enums = EnumRegistry::default();
        enums
            .declare("Outcome", &["LEFT".to_owned(), "RIGHT".to_owned()])
            .unwrap();
        let rule = crate::reader::read(
            "(rule demo/kernel :role mechanic :evidence designed \
             :material-basis \"digest contract\" :fuel 64 (bindings) \
             (effects (choose :sample demo/kernel :slot 0 \
               (branch Outcome/LEFT :mass 1m (effects)) \
               (branch Outcome/RIGHT :mass 1m (effects)))))",
        )
        .unwrap()
        .0;
        let contract = RuleContract {
            rule_id: "demo/kernel".to_owned(),
            role: RuleRole::Mechanic,
            evidence: crate::causal_contract::EvidenceClass::Designed,
        };
        let compiled = compile_rule_probability(
            &rule,
            &[0],
            &contract,
            &enums,
            &crate::typecheck::TypeEnv {
                fields: std::collections::HashMap::new(),
                exemptions: &[],
            },
            &[],
            &std::collections::HashMap::new(),
            &std::collections::HashSet::new(),
        )
        .unwrap();
        let kernel = compiled.kernel.unwrap();
        let identity = KernelInstanceIdentityV1 {
            replay_session: b"demo/replay".to_vec(),
            replay_seed: 7_i64.to_be_bytes(),
            tick: 1,
            rule_id: "demo/kernel".to_owned(),
            subject: StableElementKeyV1::Node {
                scenario: "demo/world".to_owned(),
                local_name: "subject".to_owned(),
            },
            active_elements: Vec::new(),
        };
        let masses = [Mass::from_nanounits(1), Mass::from_nanounits(1)];
        let left = realize_kernel(&identity, &kernel, &masses, 0).unwrap();
        let right = realize_kernel(&identity, &kernel, &masses, u64::MAX).unwrap();
        assert_ne!(left.selected_outcome, right.selected_outcome);
        assert_eq!(left.allocation_digest, right.allocation_digest);
        assert_eq!(left.instance_digest, right.instance_digest);
        validate_kernel_realization(&left, &identity).unwrap();
        validate_kernel_realization(&right, &identity).unwrap();

        let mut other_tick = identity.clone();
        other_tick.tick = 2;
        let changed = realize_kernel(&other_tick, &kernel, &masses, 0).unwrap();
        assert_ne!(left.instance_digest, changed.instance_digest);
    }
}
