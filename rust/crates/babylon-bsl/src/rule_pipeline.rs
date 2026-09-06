//! The composed rule-loading pipeline — the single "load a rule" entry
//! point Task 17 requires (no earlier task created it; each earlier task
//! tests its own layer in isolation by design). Composition follows the
//! §4.6 error-class ordering: lexical/syntactic (`E-LEX`/`E-PARSE`) →
//! static/type (`E-TYPE`) → load/link (`E-LOAD`), so the FIRST failure a
//! bad rule reports is the earliest-class one, matching "there is no
//! partial load and no skip-the-bad-rule mode".
//!
//! Stages, in order: read (§1/§2) → rule surface (`:material-basis`
//! `E-PARSE-011`, `:fuel` range `E-PARSE-012`) → binding declarations
//! (`E-PARSE-013/022/030/031`) → fold aggregation typecheck (§3.4,
//! `E-TYPE-041/042/043`) → selection-score / reference-comparison / no-
//! enum-arithmetic typecheck (`E-TYPE-016/017`, D118) → expression-kind
//! typecheck (§3.4, `E-TYPE-040` — #491 T1, ADR202 R1(c)/OQ-I) → anchor
//! placement (`E-LOAD-002`) → binding resolution (`E-LOAD-010/011`) →
//! free variables (`E-LOAD-010`) → static fuel bound + member-list
//! ceilings (`E-LOAD-040/042`). The `:default` allowlist lint runs LAST
//! and is carried as findings, not an error — §3.5 item 4 makes it a
//! sign-off gate, not a load rejection.
//!
//! **Fold typecheck adapter (recorded gap):** the §3.4 checker (Task 10)
//! takes the aggregation shape `(op field (:weight wfield)?)`; this
//! pipeline adapts each real `(fold <op> <query> <body> (:weight <w>)?)`
//! whose body (and weight) are FIELD REFERENCES. `count` needs no kind and
//! always passes. A fold whose body is a compound expression needs §3.4's
//! kind PROPAGATION rules, which no Phase-1 task implements — such a fold
//! is rejected LOUDLY as unverifiable rather than passed unchecked (III.11:
//! an unverified pass-through is the silent-degradation shape).

use crate::bindings::{
    check_free_variables, parse_bindings, resolve_bindings, BindingDecl, BindingError,
    BindingVocabulary,
};
use crate::bound_checker::BoundError;
use crate::causal_contract::{
    authorize_rule_effects, parse_rule_contract, validate_ast_walk_bounds,
    validate_governed_attribution, ContractError, RuleContract, AST_WALK_LIMITS,
};
use crate::declarations::DeclError;
use crate::default_lint::{lint_defaults, DefaultLintFinding};
use crate::domain::{resolve_domain, DomainError, RuleDomain};
use crate::evaluator::{evaluate, EvalEnv, Value};
use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
use crate::grammar::{
    check_arities_and_closed_sets, check_enum_ref_kinds, check_enum_ref_membership,
    check_field_init_owners, check_graph_flag_placement, check_string_positions,
    check_type_operands_are_enum_refs, GrammarError,
};
use crate::material_basis::{check_rule_surface, SurfaceError};
use crate::mod_anchors::{check_anchor, AnchorDecl, AnchorError};
use crate::probability::{
    compile_rule_probability, CompiledProbabilityFactsV1, FiniteKernelV1, FiniteProjectionV1,
    ProbabilityError,
};
use crate::reader::{read, read_all, Atom, FormPath, ReadError, SExpr};
use crate::same_tick_order::SameTickOrderError;
use crate::scope::{
    check_element_names, check_foreign_field_scoping, declared_element_names, ElementNameError,
    ScopeError,
};
use crate::structural_verbs::check_no_deferred_shape_verbs;
use crate::typecheck::{
    check_kind_mixing, check_no_arithmetic_on_enum_field, check_reference_comparisons,
    check_selection_scores, typecheck_aggregation, TypeEnv, TypeError,
};
use crate::types::EnumRegistry;
use std::collections::{HashMap, HashSet};

/// Everything a rule loads against. Phase 1 takes each registry as an
/// opaque input; their contents are Phase 2/3 content and engine data.
pub struct LoadContext<'a> {
    /// Declared fields / defines keys / registered metrics (§3.5).
    pub vocabulary: &'a BindingVocabulary,
    /// Declared field types and kinds (§3.4).
    pub types: &'a TypeEnv,
    /// Scenario-declared closed enums used by finite-kernel branch exhaustiveness.
    pub enums: &'a EnumRegistry,
    /// Scenario constants with their evaluated semantic values. Finite-kernel
    /// Mass typing consults this rather than assuming every `:const` is Mass.
    pub const_values: &'a HashMap<String, crate::evaluator::Value>,
    /// Declared cardinality ceilings (§3.7).
    pub ceilings: &'a CardinalityCeilings,
    /// Declared intrinsic costs (§2.7).
    pub intrinsics: &'a IntrinsicCosts,
    /// Registered system names, for the anchor default (§2.3).
    pub systems: &'a HashSet<String>,
    /// The closed graph vocabulary (§3.6). `None` skips the checks that
    /// need it — D74's enum-ref class rule still runs (it is a *kind*
    /// check, independent of membership), but D37's field-init owner rule
    /// cannot resolve an owner without it and is therefore not run.
    pub vocabulary_registry: Option<&'a crate::vocabulary::ClosedVocabulary>,
    /// The rule's source file, for the `:default` allowlist lint.
    pub rule_file: &'a str,
}

/// A rule that survived every load-time gate.
#[derive(Debug, Clone)]
pub struct LoadedRule {
    /// Source identity supplied by the loader, retained for typed authoring analysis.
    pub source_id: String,
    /// Exact root path of this rule in its original source forest.
    pub root_path: FormPath,
    /// The parsed rule form.
    pub rule: SExpr,
    /// Its declared bindings.
    pub bindings: Vec<BindingDecl>,
    /// Its anchor declaration, or `None` for the anchor default.
    pub anchor: Option<AnchorDecl>,
    /// What the rule fires over and how many times (§2.3, R9 chapter C4).
    /// `None` when no vocabulary was supplied, since the inference resolves
    /// a field qname's owning node type through the registry.
    pub domain: Option<RuleDomain>,
    /// The rule's governed causal role and constitutional evidence class.
    pub contract: RuleContract,
    /// The rule's one compiled finite material kernel, when present.
    pub kernel: Option<FiniteKernelV1>,
    /// The rule's exact finite recognizer projection, when declared.
    pub projection: Option<FiniteProjectionV1>,
    /// Probability authoring facts retained by the one loader compilation.
    pub probability_facts: CompiledProbabilityFactsV1,
    /// Runtime-resolved subject population for a kernel or projection.
    /// Retained so schedule linkage cannot drift from tick execution.
    pub probability_carrier: Option<String>,
    /// The §3.7 static bound `check_rule` computed and accepted — the
    /// load-time PROOF that the rule fits its budget.
    pub static_bound: u64,
    /// The rule's declared `:fuel` (§2.2) — the budget it RUNS under.
    ///
    /// Distinct from `static_bound` on purpose. Metering on the computed
    /// bound would couple runtime to whatever the checker currently returns
    /// and would silently under-fund a rule whose author allotted more.
    pub declared_fuel: u64,
    /// `:default` declarations with no allowlist row — sign-off findings,
    /// never a rejection (§3.5 item 4).
    pub default_findings: Vec<DefaultLintFinding>,
}

/// A load-time rejection, tagged by the stage that raised it.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadError {
    /// §1/§2 — the reader.
    Read(ReadError),
    /// §2.2 mandatory keywords.
    Surface(SurfaceError),
    /// §2.5/§3.5 binding declarations and resolution.
    Binding(BindingError),
    /// §3.4 aggregation law.
    Type(TypeError),
    /// Amendment AJ finite-kernel/projection analysis.
    Probability(ProbabilityError),
    /// §2's static shape rules — D74's enum-ref operand class rule and
    /// D37's field-init owner rule — plus, since Task 8 (Organization
    /// foundation plan, #534 fix round item 7), §3.6's closed-vocabulary
    /// membership/field-owner rejections
    /// (`GrammarError::Vocabulary`'s `E-LOAD-023`/`E-LOAD-030`/`E-LOAD-031`)
    /// surfacing through this module's own checks.
    Grammar(GrammarError),
    /// §2.3 anchors.
    Anchor(AnchorError),
    /// §2.3's rule domain (R9 chapter C4).
    Domain(DomainError),
    /// §2.5's foreign-`:field` reference scoping (R9 chapter C1).
    Scope(ScopeError),
    /// §2.6's `:as` element naming (R9 chapter C8).
    ElementName(ElementNameError),
    /// §3.7 static bound and member-list ceilings.
    Bound(BoundError),
    /// ADR224's mandatory role/evidence metadata and role-sensitive effect
    /// authorization (`E-PARSE-015` / `E-LOAD-060`).
    Causal(ContractError),
    /// An `(intrinsic …)` top-form's own declaration errors — `E-LOAD-001`
    /// (duplicate name across the content set), `E-LOAD-020` (signature
    /// disagreeing with the kernel's registration), `E-LOAD-024`
    /// (reserved/prohibited/uncapped name), or uncoded malformed shapes
    /// ([`crate::declarations::parse_intrinsic_decls`]).
    Intrinsic(DeclError),
    /// No numbered code (the no-invented-codes precedent): a content set's
    /// own composition rule — exactly one `(rule …)` top-form — is
    /// [`split_content`]'s discipline, not a §2 grammar production with a
    /// reserved `E-LOAD` number.
    Content(String),
    /// `E-LOAD-001` — one rule id occurs more than once in the aggregate
    /// content set. This stays distinct from [`Self::Content`] so callers
    /// can consume the governed code as structured data instead of parsing
    /// the display message.
    DuplicateRuleId {
        /// The duplicated rule id.
        rule_id: String,
    },
    /// No numbered code, same precedent as [`Self::Content`]: a rule using
    /// one of the six graph-shape verbs Task 12's collect-then-apply
    /// pre-state split does not yet defer (§4.2 chapter C4) — every one of
    /// those verbs IS legal §2.8 content; this is the LOAD-time half of
    /// `check_no_deferred_shape_verbs`'s own composition limit, not a
    /// grammar violation (#519 fix round, fix 4).
    DeferredShapeVerb(String),
    /// No numbered code, same precedent as [`Self::Content`]: a non-
    /// `<enum-ref>` child at the type-operand position of `emit`/
    /// `add-node`/`add-edge`/`remove-edge` is a shape defect the §3.7
    /// static cost pass does not itself catch (unlike its sibling
    /// positions, `bound_checker::enum_ref_key` — see
    /// [`crate::grammar::check_type_operands_are_enum_refs`]'s own
    /// doc); #528 fix round Item D (`remove-edge` added #528
    /// delta-verify rider R1). Variant name kept as-is despite the
    /// underlying check's rename — `remove-edge` doesn't mint, but this
    /// error still names the class of defect the type-operand position
    /// shares with the three minting verbs.
    MintingTypeOperand(String),
    /// §4.2/D116 rank-aware aggregate ordering — `E-LOAD-058`
    /// (stale-default read) or `E-LOAD-059` (unreset fan-in). The tick
    /// loader raises this only after executable phase ranks are compiled.
    SameTickOrder(SameTickOrderError),
}

impl LoadError {
    /// The spec's error code, where the failing stage names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::Read(e) => match &e.kind {
                crate::reader::ReadErrorKind::Lex(code) => Some(code.spec_code()),
                _ => None,
            },
            Self::Surface(e) => e.spec_code(),
            Self::Binding(e) => e.spec_code(),
            Self::Type(e) => e.code.map(crate::typecheck::TypeCode::spec_code),
            Self::Grammar(e) => Some(e.spec_code()),
            Self::Anchor(e) => e.spec_code(),
            Self::Domain(e) => e.spec_code(),
            Self::Scope(e) => Some(e.spec_code()),
            Self::ElementName(e) => Some(e.spec_code()),
            Self::Bound(e) => e.spec_code(),
            Self::Causal(e) => e.spec_code(),
            Self::Intrinsic(e) => e.spec_code(),
            Self::SameTickOrder(e) => e.spec_code(),
            Self::DuplicateRuleId { .. } => Some("E-LOAD-001"),
            // ADR248 names no numbered probability code; do not invent one.
            Self::Probability(_)
            | Self::Content(_)
            | Self::DeferredShapeVerb(_)
            | Self::MintingTypeOperand(_) => None,
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "read: {}", e.message),
            Self::Surface(e) => write!(f, "{e}"),
            Self::Binding(e) => write!(f, "{e}"),
            Self::Type(e) => write!(f, "{}", e.message),
            Self::Probability(e) => write!(f, "{e}"),
            Self::Grammar(e) => write!(f, "{e}"),
            Self::Anchor(e) => write!(f, "{e}"),
            Self::Domain(e) => write!(f, "{e}"),
            Self::Scope(e) => write!(f, "{e}"),
            Self::ElementName(e) => write!(f, "{e}"),
            Self::Bound(e) => write!(f, "{e}"),
            Self::Causal(e) => write!(f, "{e}"),
            Self::Intrinsic(e) => write!(f, "{e}"),
            Self::SameTickOrder(e) => write!(f, "{e}"),
            Self::DuplicateRuleId { rule_id } => write!(
                f,
                "E-LOAD-001: duplicate rule id: {rule_id} (§2.2 — rule ids must be \
                 content-set-unique, the same duplicate-name discipline \
                 parse_intrinsic_decls already enforces for intrinsic declarations)"
            ),
            Self::Content(message)
            | Self::DeferredShapeVerb(message)
            | Self::MintingTypeOperand(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Load one rule from source through every gate, in §4.6 class order.
///
/// This reads the WHOLE `source` as one `(rule …)` form — it is the
/// single-form entry point every earlier test in this crate was written
/// against, and it stays exactly as it was. A `source` that also carries
/// `(intrinsic …)` top-forms (§2.2) needs [`crate::declarations::
/// parse_intrinsic_decls`]'s composed caller instead (`babylon-tick::
/// run_once_into` splits a combined source via `read_all` before reaching
/// either this function or that one) — see [`load_rule_form`], the half of
/// this function that a multi-form source calls directly, having already
/// isolated the one rule form itself.
///
/// # Errors
///
/// The first-failing stage's [`LoadError`]; a loaded content set has no
/// partially-loaded rules.
pub fn load_rule(source: &str, ctx: &LoadContext<'_>) -> Result<LoadedRule, LoadError> {
    let (rule, _) = read(source).map_err(LoadError::Read)?;
    load_rule_form(rule, vec![0], ctx)
}

fn resolve_probability_carrier(
    kernel: Option<&FiniteKernelV1>,
    projection: Option<&FiniteProjectionV1>,
    bindings: &[BindingDecl],
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
) -> Result<Option<String>, LoadError> {
    let probability_form = kernel
        .map(|kernel| ("kernel", &kernel.sample_path))
        .or_else(|| projection.map(|projection| ("projection", &projection.sample_path)));
    let Some((kind, sample_path)) = probability_form else {
        return Ok(None);
    };
    crate::tick::subject_type_of_bindings(bindings, vocabulary)
        .map(Some)
        .map_err(|error| {
            LoadError::Probability(ProbabilityError::InvalidForm {
                message: format!("finite {kind} requires a stable subject carrier: {error}"),
                form_path: sample_path.clone(),
            })
        })
}

/// [`load_rule`]'s body, taking an already-parsed rule form instead of
/// re-reading one from source — the seam a multi-top-form content set
/// (§2.2) needs: its `(intrinsic …)` declarations are split out and parsed
/// BEFORE this runs (their costs feed `ctx.intrinsics`, which the static
/// fuel bound below consumes), leaving exactly the isolated `(rule …)` form
/// for every gate `load_rule` already ran.
///
/// # Errors
///
/// The first-failing stage's [`LoadError`]; a loaded content set has no
/// partially-loaded rules.
pub fn load_rule_form(
    rule: SExpr,
    root_path: FormPath,
    ctx: &LoadContext<'_>,
) -> Result<LoadedRule, LoadError> {
    check_rule_surface(&rule).map_err(LoadError::Surface)?;
    let contract = parse_rule_contract(&rule).map_err(LoadError::Causal)?;
    // The reader is iterative, while several older semantic passes below are
    // recursive. Refuse hostile trees before any such pass can consume the
    // process stack. Ordinary, in-bound rules retain the established
    // E-PARSE/E-TYPE-before-causal-authority ordering.
    validate_ast_walk_bounds(&rule, AST_WALK_LIMITS, "rule load preflight")
        .map_err(|error| LoadError::Causal(ContractError::AstWalkLimit(error)))?;
    let bindings = parse_bindings(&rule).map_err(LoadError::Binding)?;
    let binding_names: Vec<String> = bindings.iter().map(|d| d.name.clone()).collect();
    check_element_names(&rule, &binding_names).map_err(LoadError::ElementName)?;
    // §2's static shape rules run with the other E-TYPE-class checks: the
    // enum-ref class rule (D74) needs nothing but the form, and the
    // field-init owner rule (D37) needs the vocabulary.
    check_arities_and_closed_sets(&rule).map_err(LoadError::Grammar)?;
    check_string_positions(&rule).map_err(LoadError::Grammar)?;
    check_enum_ref_kinds(&rule).map_err(LoadError::Grammar)?;
    // The type-operand position of emit/add-node/add-edge/remove-edge is
    // the one §2.6 class-rule position `check_enum_ref_kinds` does not
    // itself enforce as MANDATORY-enum-ref (it only checks the KIND of an
    // enum-ref that is already there) — #528 fix round Item D
    // (remove-edge added by #528 delta-verify rider R1), see this
    // function's own doc for why these four specifically need it.
    check_type_operands_are_enum_refs(&rule).map_err(LoadError::MintingTypeOperand)?;
    check_graph_flag_placement(&rule).map_err(LoadError::Grammar)?;
    let mut domain = None;
    if let Some(vocabulary) = ctx.vocabulary_registry {
        // Task 8 (Organization foundation plan): the closed-vocabulary
        // MEMBERSHIP pass — sibling to `check_enum_ref_kinds` above, which
        // is unconditional and already proved every typed enum-ref's KIND.
        // Runs first in this block: it is the most basic fact about an
        // enum-ref (does it even name something registered), and nothing
        // below assumes it has run.
        check_enum_ref_membership(&rule, vocabulary).map_err(LoadError::Grammar)?;
        check_field_init_owners(&rule, vocabulary).map_err(LoadError::Grammar)?;
        // The domain resolves BEFORE the scoping check, which needs the
        // subject node type to know which `:field` bindings are foreign.
        let resolved = resolve_domain(&rule, &bindings, vocabulary).map_err(LoadError::Domain)?;
        let subject = match &resolved {
            RuleDomain::Node(segment) => Some(segment.clone()),
            RuleDomain::Graph => None,
        };
        check_foreign_field_scoping(&rule, &bindings, subject.as_deref(), vocabulary)
            .map_err(LoadError::Scope)?;
        domain = Some(resolved);
    }
    let compiled_probability = compile_rule_probability(
        &rule,
        &root_path,
        &contract,
        ctx.enums,
        ctx.types,
        &bindings,
        ctx.const_values,
        &ctx.vocabulary.probability_consts,
    )
    .map_err(LoadError::Probability)?;
    let kernel = compiled_probability.kernel;
    let projection = compiled_probability.projection;
    let probability_facts = compiled_probability.facts;
    typecheck_rule_folds(&rule, ctx.types, &bindings).map_err(LoadError::Type)?;
    check_selection_scores(&rule, ctx.types, &bindings).map_err(LoadError::Type)?;
    check_reference_comparisons(&rule, ctx.types, &bindings).map_err(LoadError::Type)?;
    // §2.13 (D101/D102): the D102 field-of-on-enum-field DEFERRAL gate that
    // used to run here is DISCHARGED (Task 1, P27 territory-port train) —
    // `field-of` over an enum-declared field now typechecks (the §2.7
    // classifier types it `Enum`, `score_class::classify`) and evaluates
    // for real (`evaluator::field_of_node`). Its two surviving refusals
    // each have their own independent mechanism, not a third load gate:
    // D46/E-TYPE-016 (`check_selection_scores`, above) refuses it as a
    // select-max/select-min SCORE; §2.13's no-arithmetic law refuses it as
    // an arithmetic operand at evaluation (`evaluator::apply_arith`
    // refuses `Value::Enum` unconditionally — the same runtime funnel
    // `check_no_arithmetic_on_enum_field` below's own doc names for the
    // update-node-target shape).
    //
    // §2.13's no-arithmetic law (D101), the static half (D118, #528 fix
    // round Item C) — statically decidable from the field's declared
    // type and the update-op's own symbol, so it belongs at load, not
    // left to the three eval-time guards alone (which stay, as defense
    // in depth).
    check_no_arithmetic_on_enum_field(&rule, ctx.types).map_err(LoadError::Type)?;
    // §3.4's expression-kind arm (#491 T1, ADR202 R1(c)/OQ-I): `<arith>`
    // and `if` never mix intensive with extensive kind, `E-TYPE-040` — a
    // SEPARATE walk from `typecheck_rule_folds`/`typecheck_aggregation`
    // above (the fold arm), extending this pipeline's existing dispatch
    // rather than restructuring it.
    check_kind_mixing(&rule, ctx.types, &bindings).map_err(LoadError::Type)?;
    // Causal attribution and authority are E-LOAD-class checks. They run
    // only after every E-PARSE/E-TYPE pass, so a governed mismatch or
    // E-LOAD-060 cannot mask an earlier-class defect. Role authority stays
    // more specific than the engine's current inability to defer graph-
    // shape writes: restricted roles receive E-LOAD-060; a mechanic using
    // the same well-formed verb reaches DeferredShapeVerb below.
    validate_governed_attribution(&contract).map_err(LoadError::Causal)?;
    authorize_rule_effects(&rule, &contract).map_err(LoadError::Causal)?;
    // #519 fix round, fix 4: a rule using one of the six graph-shape verbs
    // Task 12's collect-then-apply split cannot yet defer must be refused
    // HERE, at load — not left to load clean and abort the first tick
    // whose guard admits a subject (structural_verbs.rs's own doc names
    // the regression this closes).
    check_no_deferred_shape_verbs(&rule).map_err(LoadError::DeferredShapeVerb)?;
    let anchor = check_anchor(&rule, ctx.systems).map_err(LoadError::Anchor)?;
    resolve_bindings(&bindings, ctx.vocabulary).map_err(LoadError::Binding)?;
    check_free_variables(&rule, &bindings, &declared_element_names(&rule))
        .map_err(LoadError::Binding)?;
    let probability_carrier = resolve_probability_carrier(
        kernel.as_ref(),
        projection.as_ref(),
        &bindings,
        ctx.vocabulary_registry,
    )?;
    let static_bound = crate::bound_checker::check_rule_with_kernel(
        &rule,
        kernel.as_ref(),
        ctx.ceilings,
        ctx.intrinsics,
    )
    .map_err(LoadError::Bound)?;
    let default_findings = lint_defaults(ctx.rule_file, &bindings);
    let SExpr::List(rule_items) = &rule else {
        unreachable!("check_rule_surface accepted a non-list rule form")
    };
    let declared_fuel =
        crate::bound_checker::declared_fuel(rule_items).map_err(LoadError::Bound)?;
    Ok(LoadedRule {
        source_id: ctx.rule_file.to_owned(),
        root_path,
        rule,
        bindings,
        anchor,
        domain,
        contract,
        kernel,
        projection,
        probability_facts,
        probability_carrier,
        static_bound,
        declared_fuel,
        default_findings,
    })
}

/// One rule split from a source forest, retaining its original top-form path.
///
/// `form` is deliberately not reparsed in the loading pipeline: `root_path`
/// remains the canonical coordinate space for loader diagnostics and typed
/// authoring analysis over the source's [`crate::reader::SpanTable`].
#[derive(Debug, Clone, PartialEq)]
pub struct SplitRuleFormV1 {
    /// Declared rule id.
    pub rule_id: String,
    /// Parsed rule form.
    pub form: SExpr,
    /// Exact top-form path in the original source forest.
    pub root_path: FormPath,
}

/// Split one content source into its `(intrinsic …)` top-forms and its one
/// or more `(rule …)` top-forms, each paired with its own rule id.
///
/// §2.2 is normative that `<top-form> ::= <rule> | <deffield> |
/// <intrinsic-decl> | <manifest>` and that "file boundaries and file names
/// carry no semantics" — an `(intrinsic …)` declaration is ordinary
/// content, not a side channel a caller must route through a separate
/// parameter. This function is what makes that true for the standard
/// rule-loading path: it accepts intrinsic declarations and every rule
/// mixed, in ANY order, within one source string, exactly as `read_all`
/// would hand back any other multi-top-form content set.
///
/// §2.2's grammar (`<top-form>*`, zero-or-more) never limited a content set
/// to exactly one rule — that was this function's OWN cardinality check,
/// with no textual basis in the spec (Program 28 B2, Phase A Task 2). This
/// widening makes the driver honor what the grammar always admitted: one or
/// more `(rule …)` forms, in whatever order the reader encounters them
/// (this function makes no ordering claim — `babylon-tick::prepare_rules`
/// compiles executable phase placement; D16 orders only same-position
/// ties), with duplicate ids across the content set refused as
/// `E-LOAD-001`.
///
/// (`deffield` and `manifest` top-forms are not split out here — nothing
/// in this crate's Slice 1 content path reads them from a rule source yet;
/// adding a case is mechanical when one does.)
///
/// # Errors
///
/// [`LoadError::Read`] for a parse failure; [`LoadError::Content`] (uncoded)
/// when a nonempty source contains zero `(rule …)` top-forms; or
/// [`LoadError::DuplicateRuleId`] (`E-LOAD-001`) when two rule forms share
/// the same id.
pub fn split_content(source: &str) -> Result<(Vec<SExpr>, Vec<SplitRuleFormV1>), LoadError> {
    split_content_unchecked(source)
}

/// [`split_content`]'s parsing body: the `(intrinsic …)` split and
/// `E-LOAD-001` duplicate-id enforcement only. Rank-aware aggregate
/// ordering belongs to the tick loader after phase compilation.
///
/// `pub(crate)` keeps the same-tick analyzer's source adapters on the exact
/// production splitter without introducing a second parsing path.
///
/// # Errors
///
/// Same as [`split_content`]. This function cannot produce
/// [`LoadError::SameTickOrder`] because it has no execution ranks.
pub(crate) fn split_content_unchecked(
    source: &str,
) -> Result<(Vec<SExpr>, Vec<SplitRuleFormV1>), LoadError> {
    let forms = read_all(source.as_bytes()).map_err(LoadError::Read)?;
    // An empty parsed program is an explicit zero-rule transition. Comments
    // and whitespace carry no content; intrinsic-only programs still refuse.
    if forms.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut intrinsic_forms = Vec::new();
    let mut rule_forms = Vec::new();
    for (top_index, form) in forms.into_iter().enumerate() {
        if is_intrinsic_form(&form) {
            intrinsic_forms.push(form);
        } else {
            let rule_id = crate::canonical_ast::rule_id(&form)
                .map_err(|e| LoadError::Content(e.message))?
                .to_owned();
            let root_index = u32::try_from(top_index).map_err(|_| {
                LoadError::Content(
                    "a content source has more top-forms than FormPath can address".to_owned(),
                )
            })?;
            rule_forms.push(SplitRuleFormV1 {
                rule_id,
                form,
                root_path: vec![root_index],
            });
        }
    }
    if rule_forms.is_empty() {
        return Err(LoadError::Content(
            "a content set needs at least one (rule …) top-form, found 0 \
             (§2.2 — intrinsic declarations do not count; deffield/manifest/metric-decl \
             top-forms are not yet split out by this function and would also land here)"
                .to_owned(),
        ));
    }
    check_unique_rule_id_refs(rule_forms.iter().map(|rule| rule.rule_id.as_str()))?;
    Ok((intrinsic_forms, rule_forms))
}

/// Refuse a duplicate rule id across an aggregate content set.
///
/// File boundaries carry no semantics, so callers that assemble forms from
/// more than one source must run this over the aggregate, not merely rely on
/// each source's [`split_content`] call. If several ids are duplicated, the
/// byte-least id is named so source order cannot select the diagnostic.
///
/// # Errors
///
/// [`LoadError::DuplicateRuleId`] (`E-LOAD-001`) for the byte-least repeated
/// id.
pub fn check_unique_rule_ids(rules: &[(String, SExpr)]) -> Result<(), LoadError> {
    check_unique_rule_id_refs(rules.iter().map(|(id, _)| id.as_str()))
}

fn check_unique_rule_id_refs<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), LoadError> {
    let mut ids: Vec<&str> = ids.collect();
    ids.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let duplicate = ids
        .windows(2)
        .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]));
    duplicate.map_or(Ok(()), |rule_id| {
        Err(LoadError::DuplicateRuleId {
            rule_id: rule_id.to_owned(),
        })
    })
}

/// Whether `expr` is `(intrinsic …)` — the one top-form kind this module's
/// content-loading seam treats specially, since its declarations must be
/// parsed and turned into `IntrinsicCosts` BEFORE the rule form they feed
/// is loaded (`ctx.intrinsics` is an input to [`load_rule_form`]'s static
/// bound check, not an output of it).
fn is_intrinsic_form(expr: &SExpr) -> bool {
    matches!(
        expr,
        SExpr::List(items)
            if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "intrinsic")
    )
}

/// §3.5's evaluation-side half: build the evaluator environment from the
/// declared bindings and the values the world supplied. A required binding
/// with no supplied value is loud (`E-LOAD-010`-shaped — the loader should
/// have proven it present); an `:optional` binding takes its declared
/// default. No rule ever observes absence.
///
/// # Errors
///
/// [`BindingError::Unresolved`] for a missing required value.
pub fn bind_environment<S: std::hash::BuildHasher>(
    decls: &[BindingDecl],
    supplied: &HashMap<String, Value, S>,
) -> Result<HashMap<String, Value>, BindingError> {
    let mut env = HashMap::with_capacity(decls.len());
    for decl in decls {
        if let Some(value) = supplied.get(&decl.name) {
            env.insert(decl.name.clone(), value.clone());
            continue;
        }
        match &decl.default {
            Some(literal) => {
                env.insert(decl.name.clone(), literal_value(literal));
            }
            None => {
                return Err(BindingError::Unresolved {
                    name: decl.name.clone(),
                    what: "binding value (required, not supplied)",
                })
            }
        }
    }
    Ok(env)
}

/// §4.5's `:expr` accounting (D50), executed: a computed binding charges
/// its expression **once**, when the binding resolves, and each later
/// reference charges a variable-reference 1 like any other binding. That
/// asymmetry is the whole of the fuel win C7 buys, and it is why the same
/// algebra written twice inline costs strictly more than the same algebra
/// named once.
///
/// `:expr` bindings resolve in **declaration order** against the bindings
/// already resolved (§4.2), which is exactly the order `parse_bindings`
/// preserved and `E-PARSE-032` made acyclic.
///
/// `graph`: `None` for the pure-expression callers (the R9 chapters'
/// arithmetic-only conformance vectors, which build no substrate at all);
/// `Some(&dyn GraphSubstrate)` from `tick.rs::collect_pass`, the one
/// production caller, which already holds a live, read-only borrow for
/// Pass 1 (P6, PR #514 fix round — closed by the Territory port train,
/// Task 6: the `territory/p3-spillover` rule's `inflow` binding is the
/// first `:expr` body containing a query form — `exists`/`fold`/
/// `field-of` — so this parameter can no longer stay unconditionally
/// `None`). **Threaded alongside `types`/`enums`, never alone** — the PR A
/// verifier fix round closed the `Option`-None hazard class by construction
/// for those two; this parameter joins the same discipline rather than
/// reopening it.
///
/// `draw_context`: `collect_pass` constructs the subject's typed replay
/// identity before resolving expressions and threads it through the shared
/// evaluator environment. Amendment AJ permits only the engine-private
/// finite-kernel realization path to consume a draw; author expressions and
/// every declarable intrinsic remain context-free. Graph-free conformance
/// callers pass `None`, while `collect_pass` passes `Some(&draw_context)` so
/// the later compiled `choose` sees exactly the same subject identity.
///
/// # Errors
///
/// [`crate::evaluator::EvalError`] from the operand expression, including `E-EVAL-040` when
/// the shared meter runs out.
#[allow(clippy::too_many_arguments)] // graph joins types/enums (Territory port, Task 6) — same shape as tick.rs::collect_pass's own exemption
pub fn resolve_expr_bindings<S: std::hash::BuildHasher + Clone>(
    decls: &[BindingDecl],
    env: &mut HashMap<String, Value, S>,
    intrinsic_costs: &IntrinsicCosts,
    types: &TypeEnv,
    enums: &EnumRegistry,
    graph: Option<&dyn babylon_graph::substrate::GraphSubstrate>,
    draw_context: Option<&crate::intrinsic_host::DrawContext<'_>>,
    host: &dyn crate::intrinsic_host::IntrinsicHost,
    fuel: &mut u64,
) -> Result<(), crate::evaluator::EvalError> {
    for decl in decls {
        let crate::bindings::BindSource::Expr(expr) = &decl.source else {
            continue;
        };
        let scope = EvalEnv {
            bindings: env.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            intrinsic_costs,
            // A real, live reference when the caller has one — see this
            // function's own doc for the P6 history. `types`/`enums` are
            // threaded alongside it, never alone (PR A verifier fix round).
            graph,
            types: Some(types),
            enums: Some(enums),
            elements: Vec::new(),
            // `Some(&draw_context)` from `collect_pass` retains the subject's
            // replay identity for the later private finite-kernel seam. `None`
            // is reserved for callers that never construct a `DrawContext`.
            draw_context,
        };
        let value = evaluate(expr, &scope, host, fuel)?;
        env.insert(decl.name.clone(), value);
    }
    Ok(())
}

/// A `:default` literal's runtime value (§2.2: only literals).
fn literal_value(atom: &Atom) -> Value {
    match atom {
        Atom::Int(n) => Value::Int(*n),
        Atom::Currency(c) => Value::Currency(*c),
        Atom::Scaled(s) => {
            // Unit-interval literal, unscaled < 10⁹ — exact in f64.
            #[allow(clippy::cast_precision_loss)]
            let value = s.unscaled as f64 / 10f64.powi(i32::from(s.scale));
            Value::Real(value)
        }
        Atom::Bool(b) => Value::Bool(*b),
        // parse_bindings admits only the four literal atom classes above.
        other => unreachable!("non-literal :default survived parse_bindings: {other:?}"),
    }
}

/// Walk the rule's `<when>` and `<effects>` for `(fold …)` forms and run
/// each through the §3.4 aggregation law via the adapter described in the
/// module doc. A fold body written as a BINDING name resolves through the
/// binding's `:field` source for kind purposes — the binding carries its
/// declared kind (§3.4: "a `:field` binding carries its declared kind").
fn typecheck_rule_folds(
    rule: &SExpr,
    types: &TypeEnv,
    bindings: &[BindingDecl],
) -> Result<(), TypeError> {
    let SExpr::List(items) = rule else {
        return Ok(()); // shape errors are earlier stages' business
    };
    for child in items {
        let SExpr::List(inner) = child else { continue };
        match inner.first() {
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "when" || h == "effects" => {
                for body in &inner[1..] {
                    walk_folds(body, types, bindings)?;
                }
            }
            // §2.5 permits a `:expr` to contain a fold of its own, and
            // §3.4's law has no exemption for one. Walking only the rule
            // bodies let `(binding x :expr (fold mean … <intensive>))`
            // escape `E-TYPE-042` entirely while the identical fold written
            // in `<when>` was rejected — the same silent-bypass shape the
            // `:as` blind spot had.
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "bindings" => {
                for row in &inner[1..] {
                    let SExpr::List(cells) = row else { continue };
                    for window in cells.windows(2) {
                        if let [SExpr::Atom(Atom::Keyword(kw)), operand] = window {
                            if kw == "expr" {
                                walk_folds(operand, types, bindings)?;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn walk_folds(expr: &SExpr, types: &TypeEnv, bindings: &[BindingDecl]) -> Result<(), TypeError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "fold") {
        typecheck_one_fold(items, types, bindings)?;
    }
    for item in items {
        walk_folds(item, types, bindings)?;
    }
    Ok(())
}

/// Whether a fold-op's RESULT carries its body's declared kind (§3.4:
/// `sum`/`mean`/`min`/`max` do). `count`'s result is an extensive `Int`
/// naming no declared field, so it is excluded on purpose — see
/// [`field_ref_for`]'s own doc for why a nested `(fold count …)` stays with
/// the loud Phase-1 rejection rather than getting a synthetic field-registry
/// entry.
///
/// CT4P A3 (issue #525): an EXHAUSTIVE match over `FoldOp` — no wildcard —
/// so a sixth fold-op is a compile error here until this function decides
/// it, rather than silently falling through the old `matches!(op.as_str(),
/// "sum" | "mean" | "min" | "max")` string check.
fn carries_body_kind(op: crate::grammar::FoldOp) -> bool {
    match op {
        crate::grammar::FoldOp::Sum
        | crate::grammar::FoldOp::Mean
        | crate::grammar::FoldOp::Min
        | crate::grammar::FoldOp::Max => true,
        crate::grammar::FoldOp::Count => false,
    }
}

/// The declared field a fold body/weight ultimately names, if it names one.
///
/// Three shapes reduce (§3.4: kind propagates through them unchanged):
/// a bare `<qname>`; a `field-of` accessor, whose kind is the
/// declaration's exactly as a `:field` binding's is; and a binding name,
/// resolved through its source — **including a `:expr` binding**, whose
/// kind comes from its expression (§2.5, C7: "Type and kind come from the
/// expression, computed bottom-up like any other"). The `:expr` case is
/// what makes family 16's kind-propagation row expressible.
///
/// A genuinely compound body (arithmetic, an `if`, a nested fold) does
/// **not** reduce and is `None`, which the caller turns into the loud
/// unverifiable rejection — never a silent pass. `depth` bounds the
/// binding-chain walk so a pathological chain cannot loop; the
/// forward-reference ban (`E-PARSE-032`) already makes the graph a DAG, so
/// the bound is a belt on top of a brace.
fn field_ref_for(expr: &SExpr, bindings: &[BindingDecl], depth: u8) -> Option<SExpr> {
    if depth == 0 {
        return None;
    }
    match expr {
        SExpr::Atom(Atom::QName(_)) => Some(expr.clone()),
        SExpr::Atom(Atom::Symbol(name)) => {
            let decl = bindings.iter().find(|decl| decl.name == *name)?;
            match &decl.source {
                crate::bindings::BindSource::Field(qname) => {
                    Some(SExpr::Atom(Atom::QName(qname.clone())))
                }
                crate::bindings::BindSource::Expr(inner) => {
                    field_ref_for(inner, bindings, depth - 1)
                }
                // Not a field-shaped source — hand the symbol through so
                // the §3.4 checker rejects it loudly as an unknown field
                // rather than silently skipping the law.
                _ => Some(expr.clone()),
            }
        }
        SExpr::List(items) => match items.as_slice() {
            // `(field-of <expr> <qname>)` — §3.4: the accessor carries the
            // declaration's kind, identically to a `:field` binding.
            [SExpr::Atom(Atom::Symbol(head)), _, SExpr::Atom(Atom::QName(qname))]
                if head == "field-of" =>
            {
                Some(SExpr::Atom(Atom::QName(qname.clone())))
            }
            // A NESTED fold: §3.4's table says `sum`/`mean`/`min`/`max`
            // carry the body kind, so the outer fold's body kind is the
            // inner fold's body kind. This is what lets §2.6's own two-hop
            // worked example reach the aggregation law instead of the
            // unverifiable rejection. `count` is deliberately absent: its
            // result is an extensive `Int` that names no declared field, so
            // it stays with the loud Phase-1 rejection rather than getting
            // a synthetic entry in the field registry.
            [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Symbol(op)), rest @ ..]
                if head == "fold"
                    && crate::grammar::FoldOp::parse(op.as_str())
                        .is_some_and(carries_body_kind) =>
            {
                let inner = strip_elem_name(rest);
                inner
                    .get(1)
                    .and_then(|body| field_ref_for(body, bindings, depth - 1))
            }
            _ => None,
        },
        // A literal, an enum-ref or any other atom names no field; the
        // caller turns `None` into the loud unverifiable rejection.
        SExpr::Atom(_) => None,
    }
}

/// The maximum `:expr` binding hops `field_ref_for` will follow.
const MAX_BINDING_CHAIN: u8 = 8;

/// Adapt `(fold <op> <query> <body> (:weight <w>)?)` to the Task 10
/// aggregation shape `(op body (:weight w)?)`. See the module doc for the
/// count special case and the compound-body loud rejection.
fn typecheck_one_fold(
    items: &[SExpr],
    types: &TypeEnv,
    bindings: &[BindingDecl],
) -> Result<(), TypeError> {
    // §2.7's `<fold>` carries an optional `<elem-name>` between the query
    // and the body. Matching only the un-named shapes let a `:as` fold fall
    // to the catch-all and skip §3.4 ENTIRELY — appending a never-referenced
    // `:as` name was a silent bypass of the unweighted-mean-of-an-intensive
    // variance error the law exists to reject. Normalize first.
    let items = strip_elem_name(items);
    let (op, body, weight) = match items.as_slice() {
        [_, op, _query, body] => (op, body, None),
        [_, op, _query, body, SExpr::Atom(Atom::Keyword(kw)), w] if kw == "weight" => {
            (op, body, Some(w))
        }
        _ => return Ok(()), // shape errors are the bound checker's business
    };
    // CT4P A3 (issue #525): routed through the same `FoldOp` boundary as
    // every other dispatch site, rather than a hand-rolled string compare.
    let is_count = match op {
        SExpr::Atom(Atom::Symbol(name)) => {
            crate::grammar::FoldOp::parse(name) == Some(crate::grammar::FoldOp::Count)
        }
        _ => false,
    };
    if is_count {
        return Ok(()); // §3.4 row 6: count is always legal, no kind involved
    }
    let body_ref = field_ref_for(body, bindings, MAX_BINDING_CHAIN);
    let weight_ref = match weight {
        None => None,
        Some(w) => match field_ref_for(w, bindings, MAX_BINDING_CHAIN) {
            Some(resolved) => Some(resolved),
            None => {
                return Err(compound_fold_error());
            }
        },
    };
    let Some(body_ref) = body_ref else {
        return Err(compound_fold_error());
    };
    let mut adapted: Vec<SExpr> = vec![op.clone(), body_ref];
    if let Some(w) = weight_ref {
        adapted.push(SExpr::Atom(Atom::Keyword("weight".to_owned())));
        adapted.push(w);
    }
    typecheck_aggregation(&SExpr::List(adapted), types).map(|_| ())
}

/// Drop an optional `:as <symbol>` from a form's operand list (§2.6's
/// `<elem-name>?`). Mirrors `bound_checker::strip_elem_name`; kept separate
/// because that one is fallible on a malformed `:as` and this pass leaves
/// shape errors to the bound checker.
fn strip_elem_name(items: &[SExpr]) -> Vec<SExpr> {
    let mut out = Vec::with_capacity(items.len());
    let mut i = 0;
    while i < items.len() {
        if let SExpr::Atom(Atom::Keyword(kw)) = &items[i] {
            if kw == "as" && matches!(items.get(i + 1), Some(SExpr::Atom(Atom::Symbol(_)))) {
                i += 2;
                continue;
            }
        }
        out.push(items[i].clone());
        i += 1;
    }
    out
}

fn compound_fold_error() -> TypeError {
    TypeError {
        code: None,
        message: "fold body/weight kind-propagation over compound \
                  expressions is not implemented in Phase 1 — rejected \
                  loudly rather than passed unchecked (III.11); use a \
                  field reference, or wait for the Phase-2 checker"
            .to_owned(),
    }
}

#[cfg(test)]
mod split_content_tests {
    use super::{split_content, LoadError};

    const RULE: &str = "(rule vitality/probe :role mechanic :evidence derived :material-basis \"x\" :fuel 8 (when #t))";
    const INTRINSIC: &str = "(intrinsic floor :params (real) :returns int :cost 5)";

    /// §2.2: "file boundaries and file names carry no semantics" — an
    /// intrinsic declaration BEFORE the rule form must work exactly like
    /// one after it.
    #[test]
    fn an_intrinsic_declaration_before_the_rule_splits_out() {
        let source = format!("{INTRINSIC}\n{RULE}");
        let (intrinsics, rules) = split_content(&source).unwrap();
        assert_eq!(intrinsics.len(), 1);
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0].form, crate::reader::SExpr::List(_)));
        assert_eq!(rules[0].root_path, vec![1]);
    }

    #[test]
    fn an_intrinsic_declaration_after_the_rule_splits_out_the_same_way() {
        let source = format!("{RULE}\n{INTRINSIC}");
        let (intrinsics, rules) = split_content(&source).unwrap();
        assert_eq!(intrinsics.len(), 1);
        assert_eq!(rules.len(), 1);
        assert!(matches!(rules[0].form, crate::reader::SExpr::List(_)));
        assert_eq!(rules[0].root_path, vec![0]);
    }

    #[test]
    fn two_intrinsic_declarations_both_split_out() {
        const SECOND: &str = "(intrinsic exp :params (real) :returns real :cost 40)";
        let source = format!("{INTRINSIC}\n{SECOND}\n{RULE}");
        let (intrinsics, _) = split_content(&source).unwrap();
        assert_eq!(intrinsics.len(), 2);
    }

    #[test]
    fn a_source_with_no_rule_form_is_a_loud_content_error() {
        let err = split_content(INTRINSIC).unwrap_err();
        assert!(matches!(err, LoadError::Content(_)));
        assert!(format!("{err}").contains("found 0"));
    }

    #[test]
    fn a_source_with_duplicate_rule_forms_is_a_typed_duplicate_error() {
        let source = format!("{RULE}\n{RULE}");
        let err = split_content(&source).unwrap_err();
        assert!(matches!(err, LoadError::DuplicateRuleId { .. }));
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    #[test]
    fn a_source_with_no_forms_is_an_explicit_empty_program() {
        let (intrinsics, rules) = split_content("").expect("empty program");
        assert!(intrinsics.is_empty());
        assert!(rules.is_empty());
    }

    #[test]
    fn a_read_failure_propagates_as_load_error_read() {
        let err = split_content("(unterminated").unwrap_err();
        assert!(matches!(err, LoadError::Read(_)));
    }

    #[test]
    fn split_content_admits_two_rules_in_source_order() {
        let source = r#"
(rule a/first :role mechanic :evidence derived :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))
(rule b/second :role mechanic :evidence derived :material-basis "y" :fuel 10
  (bindings (binding v :field b/v))
  (effects (update-node self b/v (set v))))
"#;
        let (_intrinsics, rules) = split_content(source).expect("two distinct rule ids load");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].rule_id, "a/first");
        assert_eq!(rules[0].root_path, vec![0]);
        assert_eq!(rules[1].rule_id, "b/second");
        assert_eq!(rules[1].root_path, vec![1]);
    }

    #[test]
    fn split_content_still_admits_exactly_one_rule() {
        // The pre-Task-2 shape stays legal — this widening is additive, never
        // a floor raise. Every existing single-rule content set in the repo
        // must keep loading unchanged.
        let source = r#"(rule a/only :role mechanic :evidence derived :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))"#;
        let (_intrinsics, rules) = split_content(source).expect("one rule still loads");
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn split_content_refuses_nonempty_intrinsic_only_sources() {
        let err = split_content(INTRINSIC).unwrap_err();
        assert!(err.to_string().contains("found 0"));
    }

    #[test]
    fn a_duplicate_rule_id_across_the_content_set_is_e_load_001() {
        let source = r#"
(rule a/dup :role mechanic :evidence derived :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))
(rule a/dup :role mechanic :evidence derived :material-basis "y" :fuel 10
  (bindings (binding v :field a/v2))
  (effects (update-node self a/v2 (set v))))
"#;
        let err = split_content(source).unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
        assert!(err.to_string().contains("E-LOAD-001"));
        assert!(err.to_string().contains("a/dup"));
    }

    /// `split_content` validates and deduplicates individual forms but does
    /// not yet know their phase-anchor ranks. Aggregate tick loading applies
    /// the live rank-aware composition contract after phase compilation, so
    /// this deliberately hazardous lexical shape remains valid at this seam.
    #[test]
    fn split_content_defers_ranked_ordering_to_the_aggregate_tick_loader() {
        let source = r#"
(rule a/reader :role mechanic :evidence derived :material-basis "x" :fuel 10
  (bindings (binding v :field ns/f :optional :default 0))
  (when #t)
  (effects (update-node self ns/other (set 1))))
(rule b/writer :role mechanic :evidence derived :material-basis "y" :fuel 10
  (bindings)
  (when #t)
  (effects (update-node self ns/f (set 1))))
"#;
        let (_intrinsics, rules) = split_content(source)
            .expect("the splitter must defer rank-aware composition to aggregate loading");
        assert_eq!(rules.len(), 2);
    }
}

#[cfg(test)]
mod deferred_shape_verb_tests {
    // #519 fix round, fix 4: the LOAD-time gate for the six graph-shape
    // verbs Task 12's collect-then-apply split does not defer. Before this
    // gate, a rule using one of them loaded clean and only aborted at
    // RUNTIME, the first tick whose guard admitted a subject.
    use super::{load_rule, LoadContext, LoadError};
    use crate::bindings::BindingVocabulary;
    use crate::causal_contract::{ContractError, EffectSignature, RuleRole, ShapeVerb};
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::typecheck::TypeEnv;
    use std::collections::{HashMap, HashSet};

    fn load_ctx() -> LoadContext<'static> {
        // Leaked so the borrows in `LoadContext<'static>` are trivially
        // valid for the whole test — this module needs no drop discipline
        // and every other test-fixture pattern in this crate (tick.rs's
        // own `Fixture`) already owns its registries for the test's
        // lifetime; leaking keeps this one function self-contained rather
        // than threading four extra `&'a` parameters through every caller.
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
            systems: Box::leak(Box::new(HashSet::from([
                "control-ratio".to_owned(),
                "geography".to_owned(),
            ]))),
            vocabulary_registry: None,
            rule_file: "x.bsl",
        }
    }

    #[test]
    fn a_rule_using_remove_node_refuses_at_load_naming_the_verb() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/mint :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (remove-node self)))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(err, LoadError::DeferredShapeVerb(_)),
            "expected LoadError::DeferredShapeVerb, got {err:?}"
        );
        assert!(err.to_string().contains("remove-node"), "{err}");
    }

    #[test]
    fn restricted_roles_receive_e_load_060_for_every_shape_verb() {
        let ctx = load_ctx();
        let effects = [
            ("(add-node NodeType/TERRITORY made)", ShapeVerb::AddNode),
            ("(remove-node self)", ShapeVerb::RemoveNode),
            (
                "(add-edge EdgeType/ADJACENCY self self :strength 1.0c)",
                ShapeVerb::AddEdge,
            ),
            (
                "(remove-edge EdgeType/ADJACENCY self self)",
                ShapeVerb::RemoveEdge,
            ),
            (
                "(add-hyperedge HyperedgeType/COMMUNITY made (self))",
                ShapeVerb::AddHyperedge,
            ),
            (
                "(remove-hyperedge group HyperedgeType/COMMUNITY)",
                ShapeVerb::RemoveHyperedge,
            ),
        ];
        for (role_name, role) in [
            ("recognizer", RuleRole::Recognizer),
            ("external-event", RuleRole::ExternalEvent),
            ("intent", RuleRole::Intent),
        ] {
            for (effect, verb) in effects {
                let source = format!(
                    "(rule geography/mint :role {role_name} :evidence derived \
                     :material-basis \"x\" :fuel 64 \
                     (bindings (binding group :field geography/group)) \
                     (effects {effect}))"
                );
                let error = load_rule(&source, &ctx).unwrap_err();
                assert_eq!(error.spec_code(), Some("E-LOAD-060"), "{source}: {error}");
                assert!(matches!(
                    error,
                    LoadError::Causal(ContractError::UnauthorizedEffect {
                        role: actual_role,
                        effect: EffectSignature::Shape(actual_verb),
                        ..
                    }) if actual_role == role && actual_verb == verb
                ));
            }
        }
    }

    #[test]
    fn an_allowed_recognizer_event_ignores_verb_shaped_payload_labels() {
        let ctx = load_ctx();
        load_rule(
            r#"(rule control-ratio/c03-crisis
  :role recognizer :evidence derived :material-basis "payload labels are data" :fuel 64
  (bindings)
  (effects (emit EventType/CONTROL_RATIO_CRISIS (add-node 1) (emit 2))))"#,
            &ctx,
        )
        .expect("payload labels must not fabricate forbidden causal effects");
    }

    #[test]
    fn a_restricted_shape_inside_an_emit_payload_value_is_e_load_060() {
        let ctx = load_ctx();
        let error = load_rule(
            r#"(rule control-ratio/c03-crisis
  :role recognizer :evidence derived :material-basis "payload values are expressions" :fuel 64
  (bindings)
  (effects (emit EventType/CONTROL_RATIO_CRISIS
    (payload (add-node NodeType/TERRITORY made)))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            LoadError::Causal(ContractError::UnauthorizedEffect {
                effect: EffectSignature::Shape(ShapeVerb::AddNode),
                ..
            })
        ));
    }

    #[test]
    fn a_rule_naming_remove_node_only_inside_a_guard_still_refuses_at_load() {
        // The walk must recurse through `guard` nesting, not just the
        // top-level effect-item list.
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/mint :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (guard #t (remove-node self))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::DeferredShapeVerb(_)), "{err}");
        assert!(err.to_string().contains("remove-node"), "{err}");
    }

    // ---- G1 (#534 fix round 2, delta-verify MAJOR ×2 — one root cause
    // with the sibling fix in `grammar::check_enum_ref_membership`): the
    // F5(b) over-refusal fix (`a_rule_using_remove_node_refuses_at_load_
    // naming_the_verb` above stayed green throughout) itself
    // over-corrected — stopping at a matched `emit` head skipped the
    // WHOLE subtree, including a payload item's own VALUE, which is an
    // arbitrary `<expr>` that may illegally spell a real deferred-shape
    // verb invocation. Before this fix both probes below loaded clean and
    // only died mid-tick, the exact "load-passes/execute-dies" shape this
    // whole gate exists to prevent (this module's own header comment). ----

    #[test]
    fn a_deferred_shape_verb_inside_an_emit_payload_value_still_refuses_at_load() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/mint :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (emit EventType/RUPTURE (payload (add-node NodeType/SOCIAL_CLASS 5)))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(err, LoadError::DeferredShapeVerb(_)),
            "expected LoadError::DeferredShapeVerb, got {err:?}"
        );
        assert!(err.to_string().contains("add-node"), "{err}");
    }

    #[test]
    fn a_deferred_shape_verb_inside_an_emit_payload_value_nested_in_a_guard_still_refuses() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/mint :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (guard #t (emit EventType/RUPTURE (payload (remove-node self))))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::DeferredShapeVerb(_)), "{err}");
        assert!(err.to_string().contains("remove-node"), "{err}");
    }

    #[test]
    fn a_rule_with_no_deferred_shape_verb_is_unaffected_by_the_gate() {
        // The regression guard: a rule that uses only update-node must not
        // be refused by this gate at all — this bare-bones fixture's empty
        // `TypeEnv`/`vocabulary_registry: None` skip every LATER check that
        // would otherwise reject `geography/heat` as an unknown field, so
        // the rule loads clean end to end. Any failure here would mean the
        // gate over-fired on a verb it must never touch.
        let ctx = load_ctx();
        load_rule(
            r#"(rule geography/mint :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (update-node self geography/heat (set 1))))"#,
            &ctx,
        )
        .expect("update-node must never trip the deferred-shape-verb gate");
    }
}

#[cfg(test)]
mod governed_field_write_prohibition_tests {
    // PER-22 ruling D1 (Director, 2026-09-02): `territory/county-fips` is
    // governed geography identity. The declared territory→county mapping is
    // frozen at campaign foundation, so a rule writing the field could
    // rewrite county identity after foundation and contradict the durable
    // mapping. Before this gate a MECHANIC rule — the role whose write
    // surface is otherwise unrestricted — loaded such a write clean and
    // executed it mid-tick; the refusal must happen at load, for every
    // role, through the ordinary `load_rule` entry point.
    use super::{load_rule, LoadContext, LoadError};
    use crate::bindings::BindingVocabulary;
    use crate::causal_contract::{ContractError, GOVERNED_WRITE_PROHIBITED_NODE_FIELDS};
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::typecheck::TypeEnv;
    use std::collections::{HashMap, HashSet};

    fn load_ctx() -> LoadContext<'static> {
        // Same leaked-fixture convention as `deferred_shape_verb_tests::
        // load_ctx`: the empty `TypeEnv` skips later field-declaration
        // checks, so the rule reaches effect authorization and only the
        // governed prohibition can refuse it.
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
            systems: Box::leak(Box::new(HashSet::from([
                "control-ratio".to_owned(),
                "geography".to_owned(),
            ]))),
            vocabulary_registry: None,
            rule_file: "x.bsl",
        }
    }

    #[test]
    fn the_governed_table_names_the_county_mapping_field() {
        assert_eq!(
            GOVERNED_WRITE_PROHIBITED_NODE_FIELDS,
            &["territory/county-fips"]
        );
    }

    #[test]
    fn a_mechanic_rule_writing_governed_geography_identity_refuses_at_load() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/county-probe :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (update-node self territory/county-fips (set 26163))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                LoadError::Causal(ContractError::GovernedFieldWriteProhibited { .. })
            ),
            "expected a governed write prohibition, got {err:?}"
        );
        assert!(err.to_string().contains("territory/county-fips"), "{err}");
    }

    #[test]
    fn a_governed_field_write_nested_in_a_guard_still_refuses_at_load() {
        // The effect walk must recurse through guard nesting, exactly like
        // the deferred-shape-verb gate it extends.
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule geography/county-probe :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (guard #t (update-node self territory/county-fips (set 26163)))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                LoadError::Causal(ContractError::GovernedFieldWriteProhibited { .. })
            ),
            "expected a governed write prohibition, got {err:?}"
        );
    }

    #[test]
    fn rules_writing_ordinary_fields_stay_legal_for_every_role_gate() {
        // The regression guard: update-node on an ordinary field must not
        // trip the prohibition — mechanic write surface stays intact.
        let ctx = load_ctx();
        load_rule(
            r#"(rule geography/heat-probe :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (effects (update-node self geography/heat (set 1))))"#,
            &ctx,
        )
        .expect("ordinary field writes must stay legal");
    }
}

#[cfg(test)]
mod enum_fold_body_tests {
    // #551 closure (Task 2, P27 territory-port train): `(fold <op> <query>
    // <enum-declared body>)` used to pass the §3.4 kind law silently,
    // dying uncoded at evaluation on the first admitted subject — this
    // module proves the load-time refusal (`typecheck::TypeCode::
    // EnumFoldBody`, `E-TYPE-044`) is REACHABLE from the real production
    // entry point (`load_rule`), through BOTH read routes a fold body can
    // take to an enum-declared field: a `:field`-bound SYMBOL (§2.5) and
    // `field-of` (§2.10, D102 — discharged by Task 1 of this same train,
    // making this route newly reachable in the first place).
    use super::{load_rule, LoadContext, LoadError};
    use crate::bindings::BindingVocabulary;
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::typecheck::{TypeCode, TypeEnv};
    use crate::types::{BslType, EnumRegistry, FieldDecl, FieldKind};
    use std::collections::{HashMap, HashSet};

    /// `OrgKind` in declaration order: `STATE_APPARATUS`=0, `BUSINESS`=1 —
    /// the same fixture shape `tick.rs::org_kind_fixture`/
    /// `structural_verbs::org_kind_types_and_enums` use. Leaked, matching
    /// this module's sibling test modules' own `LoadContext<'static>`
    /// convention (`deferred_shape_verb_tests::load_ctx`'s own doc).
    fn load_ctx() -> LoadContext<'static> {
        // `EnumRegistry` itself is not part of `LoadContext` — member
        // *names* only matter at tick RUNTIME (`bind_field_value`); load
        // only needs `TypeEnv`'s `BslType::Enum(id)` + `FieldKind::
        // NotApplicable` kind, so the registry can be dropped right after
        // minting `ty` (`EnumTypeId` is `Copy`, no borrow survives it).
        let mut enums = EnumRegistry::default();
        let ty = enums
            .declare(
                "OrgKind",
                &["STATE_APPARATUS".to_owned(), "BUSINESS".to_owned()],
            )
            .unwrap();
        let fields = HashMap::from([(
            "organization/kind".to_owned(),
            FieldDecl {
                ty: BslType::Enum(ty),
                kind: FieldKind::NotApplicable,
            },
        )]);
        LoadContext {
            vocabulary: Box::leak(Box::new(BindingVocabulary {
                fields: HashSet::from(["organization/kind".to_owned()]),
                consts: HashSet::new(),
                probability_consts: HashSet::new(),
                metrics: HashSet::new(),
            })),
            types: Box::leak(Box::new(TypeEnv {
                fields,
                exemptions: &[],
            })),
            enums: Box::leak(Box::new(enums)),
            const_values: Box::leak(Box::new(HashMap::new())),
            ceilings: Box::leak(Box::new(CardinalityCeilings::new(
                HashMap::from([("NodeType/ORGANIZATION".to_owned(), 100)]),
                HashMap::new(),
            ))),
            intrinsics: Box::leak(Box::new(IntrinsicCosts::default())),
            systems: Box::leak(Box::new(HashSet::from(["organization".to_owned()]))),
            vocabulary_registry: None,
            rule_file: "x.bsl",
        }
    }

    /// Route 1: a `:field`-bound SYMBOL as the fold body — the shape
    /// #551's own title names (`<:field-bound enum symbol>`).
    #[test]
    fn a_fold_sum_over_a_field_bound_enum_symbol_refuses_at_load_e_type_044() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule organization/enum-fold-probe
  :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings (binding kind :field organization/kind))
  (when (> (fold sum (nodes NodeType/ORGANIZATION) kind) 0))
  (effects (emit EventType/RUPTURE (probe 1))))"#,
            &ctx,
        )
        .unwrap_err();
        let LoadError::Type(type_err) = &err else {
            panic!("expected LoadError::Type, got {err:?}");
        };
        assert_eq!(type_err.code, Some(TypeCode::EnumFoldBody));
        assert_eq!(err.spec_code(), Some("E-TYPE-044"));
        assert!(err.to_string().contains("organization/kind"), "{err}");
    }

    /// Route 2: `field-of` as the fold body — newly reachable now that
    /// Task 1 discharged D102 (`field-of` over an enum-declared field used
    /// to refuse unconditionally at load, before ever reaching this fold
    /// kind law at all).
    #[test]
    fn a_fold_sum_over_a_field_of_enum_accessor_refuses_at_load_e_type_044() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule organization/enum-fold-probe
  :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings)
  (when (> (fold sum (nodes NodeType/ORGANIZATION)
                 (field-of it organization/kind)) 0))
  (effects (emit EventType/RUPTURE (probe 1))))"#,
            &ctx,
        )
        .unwrap_err();
        let LoadError::Type(type_err) = &err else {
            panic!("expected LoadError::Type, got {err:?}");
        };
        assert_eq!(type_err.code, Some(TypeCode::EnumFoldBody));
        assert_eq!(err.spec_code(), Some("E-TYPE-044"));
    }

    /// `count` stays legal over an enum-declared body through BOTH routes
    /// — the narrower verdict this closure's own doc records: `count`
    /// never evaluates its body, so naming an enum field there is inert,
    /// not a content error.
    #[test]
    fn a_fold_count_over_an_enum_declared_body_is_unaffected_through_both_routes() {
        let ctx = load_ctx();
        load_rule(
            r#"(rule organization/enum-fold-count-probe
  :role mechanic :evidence derived :material-basis "x" :fuel 256
  (bindings (binding kind :field organization/kind))
  (when (> (fold count (nodes NodeType/ORGANIZATION) kind) 0))
  (effects (emit EventType/RUPTURE (probe 1))))"#,
            &ctx,
        )
        .expect("count over a :field-bound enum symbol must stay legal");

        let ctx = load_ctx();
        load_rule(
            r#"(rule organization/enum-fold-count-probe
  :role mechanic :evidence derived :material-basis "x" :fuel 256
  (bindings)
  (when (> (fold count (nodes NodeType/ORGANIZATION)
                 (field-of it organization/kind)) 0))
  (effects (emit EventType/RUPTURE (probe 1))))"#,
            &ctx,
        )
        .expect("count over a field-of enum accessor must stay legal");
    }
}

#[cfg(test)]
mod vocabulary_membership_tests {
    // Task 8 (Organization foundation plan): the load-time half of
    // closed-vocabulary enforcement, wired into `load_rule_form`'s
    // `ctx.vocabulary_registry`-gated block. Exercised through `emit`
    // rather than `add-node`/`add-edge`/`add-hyperedge`: those three are
    // ALREADY refused, unconditionally, by `check_no_deferred_shape_verbs`
    // (the six graph-shape verbs — see `deferred_shape_verb_tests` above
    // and `structural_verbs.rs`'s own doc), so a rule using one would
    // never reach this gate through the full pipeline regardless of
    // vocabulary — `emit` is not one of the six, and its type operand is
    // one of D74's own sixteen typed positions, so it is the one minting
    // form that actually proves THIS gate, not an earlier one.
    use super::{load_rule, LoadContext, LoadError};
    use crate::bindings::BindingVocabulary;
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::grammar::GrammarError;
    use crate::typecheck::TypeEnv;
    use crate::vocabulary::{ClosedVocabulary, EnumKind, VocabularyError};
    use std::collections::{HashMap, HashSet};

    fn vocabulary() -> ClosedVocabulary {
        ClosedVocabulary::new([(EnumKind::EventType, vec!["RUPTURE".to_owned()])]).unwrap()
    }

    fn load_ctx(vocabulary_registry: Option<&ClosedVocabulary>) -> LoadContext<'_> {
        // Same leaking-fixture shape as `deferred_shape_verb_tests::load_ctx`
        // above — this module needs no drop discipline either.
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
            systems: Box::leak(Box::new(HashSet::from(["probe".to_owned()]))),
            vocabulary_registry,
            rule_file: "x.bsl",
        }
    }

    const RULE: &str = r#"(rule probe/vocab :role mechanic :evidence derived :material-basis "x" :fuel 64
  (domain :graph)
  (bindings)
  (when #t)
  (effects (emit EventType/RUPTURE)))"#;

    const TYPO_RULE: &str = r#"(rule probe/vocab :role mechanic :evidence derived :material-basis "x" :fuel 64
  (domain :graph)
  (bindings)
  (when #t)
  (effects (emit EventType/NOWHERE)))"#;

    #[test]
    fn an_unregistered_enum_ref_in_a_rule_is_e_load_031_under_a_declared_vocabulary() {
        let vocab = vocabulary();
        let ctx = load_ctx(Some(&vocab));
        let err = load_rule(TYPO_RULE, &ctx).unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-031"));
        assert!(
            matches!(
                &err,
                LoadError::Grammar(GrammarError::Vocabulary {
                    error: VocabularyError::UnknownEnumMember { enum_type, member, .. },
                    ..
                }) if enum_type == "EventType" && member == "NOWHERE"
            ),
            "{err:?}"
        );
        // F6 (#534 fix round item 6): the offending verb is named too.
        assert!(err.to_string().contains("emit"), "{err}");
    }

    #[test]
    fn a_registered_enum_ref_loads_clean_under_a_declared_vocabulary() {
        let vocab = vocabulary();
        let ctx = load_ctx(Some(&vocab));
        load_rule(RULE, &ctx).expect("a registered EventType member must load");
    }

    #[test]
    fn the_same_typo_source_loads_with_no_vocabulary_declared_backward_compat_pin() {
        // The plan's own backward-compatibility proof: with NO declared
        // vocabulary, `EventType/NOWHERE` is exactly as legal as it was
        // before this task — membership is unchecked, opt-in per scenario.
        let ctx = load_ctx(None);
        load_rule(TYPO_RULE, &ctx)
            .expect("no declared vocabulary means membership is unchecked (backward compat)");
    }
}

/// Review finding F1 (#491 T1): a load-path proof that
/// [`crate::typecheck::check_kind_mixing`] is actually WIRED into
/// [`load_rule`], not merely correct in isolation. Every unit test for the
/// kind arm itself (`typecheck.rs`'s own test module, 36 functions) calls
/// `check_kind_mixing`/`expr_kind` DIRECTLY — until every committed
/// straddle was repaired, the content gate was the de facto wiring proof,
/// but with all four straddles now fixed, no committed content violates
/// the rule, so deleting the `check_kind_mixing(&rule, ctx.types,
/// &bindings)` call at `rule_pipeline.rs:321` would leave every test in
/// this crate green. Mirrors the sibling load-path coverage this crate
/// already has for `E-TYPE-044` (`enum_fold_body_tests`, above) and
/// `E-TYPE-041`/`042` (`r9_chapters.rs`/`conformance_corpus.rs`).
#[cfg(test)]
mod kind_mixing_wiring_tests {
    use super::{load_rule, LoadContext, LoadError};
    use crate::bindings::BindingVocabulary;
    use crate::causal_contract::ContractError;
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::typecheck::{TypeCode, TypeEnv};
    use crate::types::{BslType, FieldDecl, FieldKind};
    use std::collections::{HashMap, HashSet};

    fn load_ctx() -> LoadContext<'static> {
        let fields = HashMap::from([
            (
                "organization/budget".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            ),
            (
                "organization/share".to_owned(),
                FieldDecl {
                    ty: BslType::Coefficient,
                    kind: FieldKind::Intensive,
                },
            ),
        ]);
        LoadContext {
            vocabulary: Box::leak(Box::new(BindingVocabulary {
                fields: HashSet::from([
                    "organization/budget".to_owned(),
                    "organization/share".to_owned(),
                ]),
                consts: HashSet::new(),
                probability_consts: HashSet::new(),
                metrics: HashSet::new(),
            })),
            types: Box::leak(Box::new(TypeEnv {
                fields,
                exemptions: &[],
            })),
            enums: Box::leak(Box::new(crate::types::EnumRegistry::default())),
            const_values: Box::leak(Box::new(HashMap::new())),
            ceilings: Box::leak(Box::new(CardinalityCeilings::new(
                HashMap::new(),
                HashMap::new(),
            ))),
            intrinsics: Box::leak(Box::new(IntrinsicCosts::default())),
            systems: Box::leak(Box::new(HashSet::from(["organization".to_owned()]))),
            vocabulary_registry: None,
            rule_file: "x.bsl",
        }
    }

    #[test]
    fn a_rule_mixing_intensive_and_extensive_under_plus_refuses_at_load_e_type_040() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule organization/kind-mixing-probe
  :role mechanic :evidence derived :material-basis "x" :fuel 64
  (bindings
    (binding budget :field organization/budget)
    (binding share :field organization/share))
  (effects (emit EventType/RUPTURE (probe (+ budget share)))))"#,
            &ctx,
        )
        .unwrap_err();
        let LoadError::Type(type_err) = &err else {
            panic!("expected LoadError::Type, got {err:?}");
        };
        assert_eq!(type_err.code, Some(TypeCode::KindMixing));
        assert_eq!(err.spec_code(), Some("E-TYPE-040"));
        assert!(err.to_string().contains("E-TYPE-040"), "{err}");
    }

    #[test]
    fn governed_attribution_mismatch_cannot_mask_an_e_parse_defect() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule economics/fundamental-theorem
  :role mechanic :evidence designed :material-basis "x" :fuel 64
  (bindings)
  (effects (update-node self organization/budget (unset 1))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Grammar(_)), "{err:?}");
        assert_eq!(err.spec_code(), Some("E-PARSE-015"));
    }

    #[test]
    fn governed_attribution_mismatch_cannot_mask_an_e_type_defect() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule economics/fundamental-theorem
  :role mechanic :evidence designed :material-basis "x" :fuel 64
  (bindings
    (binding budget :field organization/budget)
    (binding share :field organization/share))
  (effects (emit EventType/RUPTURE (probe (+ budget share)))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Type(_)), "{err:?}");
        assert_eq!(err.spec_code(), Some("E-TYPE-040"));
    }

    #[test]
    fn a_well_typed_governed_attribution_mismatch_reaches_the_load_gate() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule economics/fundamental-theorem
  :role mechanic :evidence designed :material-basis "x" :fuel 8
  (bindings)
  (effects))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            LoadError::Causal(ContractError::GovernedAttributionMismatch { .. })
        ));
    }

    #[test]
    fn unauthorized_role_effect_cannot_mask_an_e_type_defect() {
        let ctx = load_ctx();
        let err = load_rule(
            r#"(rule organization/kind-mixing-probe
  :role external-event :evidence designed :material-basis "x" :fuel 64
  (bindings
    (binding budget :field organization/budget)
    (binding share :field organization/share))
  (effects (emit EventType/RUPTURE (probe (+ budget share)))))"#,
            &ctx,
        )
        .unwrap_err();
        assert!(matches!(err, LoadError::Type(_)), "{err:?}");
        assert_eq!(err.spec_code(), Some("E-TYPE-040"));
    }
}

#[cfg(test)]
mod empty_program_tests {
    use super::split_content;
    use crate::rules_hash_of;

    #[test]
    fn empty_program_spelling_has_one_empty_rule_hash() {
        let expected = rules_hash_of(&[]).expect("empty canonical rule set");
        for source in ["", " \n\t", "; immutable observer baseline\n"] {
            let (intrinsics, rules) = split_content(source).expect("explicit empty program");
            assert!(intrinsics.is_empty());
            assert!(rules.is_empty());
            let forms: Vec<_> = rules.into_iter().map(|rule| rule.form).collect();
            assert_eq!(rules_hash_of(&forms).unwrap(), expected);
        }
    }

    #[test]
    fn empty_program_does_not_swallow_malformed_or_intrinsic_only_content() {
        assert!(split_content("(").is_err());
        assert!(split_content("(intrinsic economy/foo :cost 1)").is_err());
        assert!(split_content("(nonsense)").is_err());
    }
}
