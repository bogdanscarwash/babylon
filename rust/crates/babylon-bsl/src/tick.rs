//! The tick: subject iteration over one loaded rule (P27 Phase 2 Slice 1).
//!
//! This is the loop the engine was missing. Every other piece existed —
//! [`crate::rule_pipeline::load_rule`] gates a rule, [`crate::evaluator`]
//! evaluates an expression, [`crate::structural_verbs::EffectExecutor`]
//! mutates a substrate — but nothing drove them over a world.
//!
//! **A rule does not loop OVER SUBJECTS; the engine does.** BSL rules are
//! written against `self` (`(update-node self social-class/agitation (add
//! 0.05i))`), so the rule states what happens to *one* subject and the
//! engine applies it to each in turn. **Corrected (#519 fix round):** this
//! used to say "that is why there is no `for-each` in the grammar" — false
//! since Task 10 (§2.8 chapter C6) added `for-each` in effect position
//! (`crate::structural_verbs`). What remains true, and is what this module
//! actually owns, is narrower: `for-each` iterates a QUERY RESULT within
//! ONE subject's effect list; it never iterates the SUBJECT population
//! itself — that population, and the order it fires in, is `run_tick`'s
//! alone, not a rule's to express.
//!
//! # The subject type, and how it is derived
//!
//! [`crate::bindings::BindSource::Field`]'s own contract is "a declared field
//! of **`self`'s node type**", so a rule's field bindings name their subject:
//! `social-class/wages` can only be read off a `social-class` node. Slice 1
//! therefore derives the subject type from the shared namespace of the
//! rule's `:field` bindings, and rejects a rule whose fields disagree —
//! because two namespaces mean two subject types, and picking one silently
//! would run the rule over the wrong population.
//!
//! **This is a slice-1 derivation, not a spec ruling.** The durable source
//! should be the `deffield` registry (Phase-2 content), which knows each
//! field's owning type without inference. Recorded here so the assumption is
//! visible rather than load-bearing-and-forgotten.
//!
//! # Determinism
//!
//! Subjects come from [`GraphSubstrate::nodes`], which is contractually
//! ascending by id, so the order rules fire in is fixed by the substrate and
//! not by storage.
//!
//! **Superseded (Task 12, P27 Phase 2 query-evaluation plan, 2026-08-11):**
//! this section used to say each subject reads through the same graph it
//! mutates and sees prior subjects' mutations, matching the frozen Python
//! engine's in-place behaviour. That was an admitted implementation/spec
//! divergence (D-row Q1), not a ruling — §4.2 chapter C4 says *"all firings
//! of one rule observe the same pre-state … and the effects they collect
//! are applied in that subject order."* [`run_tick`] now follows the spec:
//! it runs in two passes, collecting every subject's writes (via
//! [`crate::structural_verbs::EffectExecutor::collect_effects`]) against
//! the SAME pre-tick graph before applying any of them (via
//! [`crate::structural_verbs::EffectExecutor::apply_pending_write`]), in
//! subject order. Verified byte-neutral for every rule pack landed at the
//! time of the repair (`rust/crates/babylon-tick/tests/tick_goldens.rs`) —
//! none reads another node's field, so the divergence was unobservable
//! until a rule does.
//!
//! **Scope of this claim, resolved by ADR224:** this module owns the
//! within-one-rule half (subject-to-subject). Across rules, the tick driver
//! deliberately runs each rule to completion in canonical phase/rank/id
//! order against one working graph. A later same-rank rule therefore sees
//! earlier writes. The live rank-aware composition check rejects the two
//! hazardous patterns that would make this sequential contract ambiguous:
//! stale optional-default reads and unreset multiwriter fan-in.
//!
//! # Fuel
//!
//! Each subject gets its own budget, taken from the rule's DECLARED
//! `:fuel` — never from `static_bound`, which is `check_rule`'s computed
//! proof that the rule fits, not the allowance it runs under.
//!
//! The budget is per-SUBJECT because the §3.7 bound is per-subject:
//! `check_rule` accepted the rule against one subject's worth of work, so
//! sharing one meter across a population would make a rule's admissibility
//! depend on how many nodes happened to exist, which is not a property of
//! the rule.

use crate::bindings::{BindSource, BindingDecl};
use crate::evaluator::{charge, evaluate, EvalEnv, EvalError, Value};
use crate::intrinsic_host::{
    draw_finite_kernel_ticket, DrawContext, DrawIdentityContext, IntrinsicCallCtx, IntrinsicHost,
};
use crate::probability::{
    evaluate_kernel_masses, forecast_event_likelihoods as exact_pushforward, BranchProjectionV1,
    EventLikelihoodV1, KernelInstanceIdentityV1, KernelRealizationV1, FINITE_KERNEL_DRAW_BASE,
};
use crate::reader::{Atom, SExpr};
use crate::rule_pipeline::LoadedRule;
use crate::structural_verbs::{EffectExecutor, EventSink};
use crate::typecheck::TypeEnv;
use crate::types::{BslType, EnumRegistry};
use crate::write_log::WriteObserver;
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_graph::working_copy::DetachedCopy;
use babylon_kernel::replay::{RngDomainV2, RngSeedContext};
use babylon_kernel::SessionId;
use std::collections::HashMap;

/// Why a tick would not run.
#[derive(Debug, Clone, PartialEq)]
pub struct TickError {
    /// Human-readable detail.
    pub message: String,
}

impl std::fmt::Display for TickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TickError {}

impl From<EvalError> for TickError {
    fn from(err: EvalError) -> Self {
        Self {
            message: format!("{err}"),
        }
    }
}

fn err(message: impl Into<String>) -> TickError {
    TickError {
        message: message.into(),
    }
}

/// What one tick did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickOutcome {
    /// The node type the rule ran over.
    pub subject_type: String,
    /// How many subjects the guard was evaluated against.
    pub considered: usize,
    /// How many passed the guard and had their effects executed.
    pub fired: usize,
    /// Successful choices in subject encounter order.
    pub kernel_realizations: Vec<KernelRealizationV1>,
    /// Stable subject provenance aligned one-for-one with emitted events.
    pub event_provenance: Vec<EmittedEventProvenanceV1>,
}

/// Stable carrier provenance for one emitted event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedEventProvenanceV1 {
    /// Graph-owned stable subject identity; never inferred from payload.
    pub subject: StableElementKeyV1,
}

/// Read-only dependencies for one bounded detached-state forecast.
pub struct ForecastContextV1<'a> {
    /// Declared field types.
    pub types: &'a TypeEnv,
    /// Closed enum declarations.
    pub enums: &'a EnumRegistry,
    /// Deterministic named intrinsic host.
    pub host: &'a dyn IntrinsicHost,
    /// Declared intrinsic costs.
    pub costs: &'a crate::fuel::IntrinsicCosts,
    /// Constant values.
    pub defines: &'a DefinesEnv,
    /// Tick whose calendar bindings are evaluated.
    pub tick: i64,
    /// Closed vocabulary, when the content set declares one.
    pub vocabulary: Option<&'a crate::vocabulary::ClosedVocabulary>,
}

/// The defines environment §4.2 names — coefficients by qualified name, the
/// values a `:const` binding reads (§2.5).
///
/// Its *contents* are content: `GameDefines`/`defines.yaml` in Phase 2, the
/// scenario's `(defconst …)` rows until then. Its *shape* belongs here
/// because the tick is where a coefficient meets a rule.
pub type DefinesEnv = HashMap<String, Value>;

/// `social-class` → `SOCIAL_CLASS`: the field namespace as a `NodeType`
/// member.
///
/// The verb layer stamps the enum member verbatim
/// (`(add-node NodeType/SOCIAL_CLASS …)` → `"SOCIAL_CLASS"`), and
/// [`crate::scenario`] mints the same, so this is the one conversion that
/// makes a rule's fields and the substrate's nodes name the same population.
///
/// `pub(crate)` (Task 8, P27 Phase 2 Slice 1): `evaluator::field_of_node`
/// needs the SAME rendering to compare a `field-of` qname's owning segment
/// against `GraphSubstrate::node_type_of`'s result (§2.10 discipline 1) —
/// reused rather than re-derived, so the two readings cannot drift apart.
/// A second reuse caller landed at T2 (issue #559):
/// `evaluator::check_edge_referent_type` renders a qname's owning segment
/// to an `EdgeType` member through this SAME function (the rendering rule
/// is identical for both enum kinds — uppercase, `-` → `_`), comparing it
/// against an `EdgeKey`'s inline `edge_type` — the `EdgeRef` half of the
/// same §2.10 discipline 1, on the same no-drift reasoning.
pub(crate) fn namespace_to_node_type(namespace: &str) -> String {
    namespace.to_uppercase().replace('-', "_")
}

/// The subject type a rule's `:field` bindings agree on.
///
/// **D29 filter (Community port train, Task 6, E2b):** a `:field` binding
/// is node-scoped and stays node-scoped — an edge's or a hyperedge's field
/// is read by the `field-of` accessor, never by a binding (§2.5's draft
/// ruling, `bsl-language.rst` D29). When the closed vocabulary is in
/// force, a binding whose qname owns off an `EdgeType`/`HyperedgeType`
/// member is therefore INVISIBLE to this derivation — exactly the filter
/// `domain.rs::self_scoped_segments` already applies at load, kept in
/// agreement with it so the load-time domain and the tick-time population
/// can never fork. A rule whose only `:field` bindings are hyperedge-owned
/// then names no subject type at all — the same refusal a binding-free
/// rule gets, which is the mechanism §8c of the Community port plan cites
/// (`no_field_binding_uses_the_community_namespace` quotes this function).
/// Without a vocabulary (`None` — the registry-free unit-test lane) the
/// owner kind is unknowable and every namespace counts, exactly as before.
pub(crate) fn subject_type_of_bindings(
    bindings: &[BindingDecl],
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
) -> Result<String, TickError> {
    let mut namespaces: Vec<&str> = Vec::new();
    for binding in bindings {
        if let BindSource::Field(qname) = &binding.source {
            let namespace = qname.split('/').next().unwrap_or_default();
            if let Some(vocabulary) = vocabulary {
                // Filter ONLY on a POSITIVE owner-kind answer: a segment the
                // vocabulary does not know (`Err(UnknownFieldOwner)`) is
                // E-LOAD-023's lane at load, not this filter's — a scenario
                // may declare a partial vocabulary (e.g. HyperedgeType
                // members only), and treating its unknown node namespaces as
                // filtered would empty every subject population.
                if let Ok((kind, _)) = vocabulary.owner_of(namespace) {
                    if kind != crate::vocabulary::EnumKind::NodeType {
                        continue;
                    }
                }
            }
            if !namespaces.contains(&namespace) {
                namespaces.push(namespace);
            }
        }
    }
    match namespaces.as_slice() {
        [] => Err(err(
            "the rule declares no :field binding, so it names no subject type — \
             slice 1 runs rules over a population, not over the graph as a whole",
        )),
        [one] => Ok(namespace_to_node_type(one)),
        many => Err(err(format!(
            "the rule's :field bindings span {} namespaces ({}), so its subject type \
             is ambiguous; a field is a field OF self's node type",
            many.len(),
            many.join(", ")
        ))),
    }
}

/// Resolve the runtime subject population for a loaded rule using the same
/// closed-vocabulary filtering as authoritative tick execution.
///
/// This read-only seam exists for source analysis and detached forecasting;
/// callers must not re-infer a carrier from `LoadedRule::domain`, because the
/// runtime also supports the registry-free namespace inference used here.
///
/// # Errors
///
/// Returns the same [`TickError`] as tick execution when the rule names no
/// node-scoped field population or names more than one population.
pub fn subject_type_of_rule(
    loaded: &LoadedRule,
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
) -> Result<String, TickError> {
    subject_type_of_bindings(&loaded.bindings, vocabulary)
}

/// Pull `(when …)` and `(effects …)` out of a loaded rule form.
fn guard_and_effects(rule: &SExpr) -> Result<(Option<&SExpr>, &[SExpr]), TickError> {
    let SExpr::List(items) = rule else {
        return Err(err("a rule is a list form"));
    };
    let mut guard = None;
    let mut effects: &[SExpr] = &[];
    for item in items {
        let SExpr::List(parts) = item else { continue };
        let Some(SExpr::Atom(Atom::Symbol(tag))) = parts.first() else {
            continue;
        };
        match tag.as_str() {
            "when" => guard = parts.get(1),
            "effects" => effects = &parts[1..],
            _ => {}
        }
    }
    Ok((guard, effects))
}

/// Read one subject's **external** bindings out of the world.
///
/// An `:optional` binding with no stored value falls back to its declared
/// `:default`; a required one that was never written propagates the
/// substrate's loud error, because III.11 says absence is not zero.
///
/// The match over `<bind-src>` is **exhaustive on purpose**. It used to
/// read `let BindSource::Field(qname) = … else { continue };`, which
/// silently skipped every other source: a rule declaring a `:expr`, a
/// `:const`, a `:metric` or a calendar binding loaded clean and then died
/// at guard evaluation with a generic unbound-variable error — the
/// load-passes/execute-dies shape. Every arm now either produces a value or
/// is refused by [`check_sources_servable`] before any subject runs.
fn bind_subject(
    subject: NodeId,
    bindings: &[BindingDecl],
    graph: &dyn GraphSubstrate,
    defines: &DefinesEnv,
    tick: i64,
    types: &TypeEnv,
    enums: &EnumRegistry,
) -> Result<HashMap<String, Value>, TickError> {
    let mut env = HashMap::from([("self".to_owned(), Value::NodeRef(subject))]);
    for binding in bindings {
        let qname = match &binding.source {
            BindSource::Field(qname) => qname,
            // §2.5/§4.2: a coefficient out of the defines environment. The
            // value is the same for every subject, so this is a lookup
            // rather than a read — but it happens HERE, per subject, so that
            // one code path owns "what a binding resolves to".
            BindSource::Const(qname) => {
                let Some(value) = defines.get(qname) else {
                    // Unreachable when the vocabulary and the environment
                    // come from one scenario, which is what the message
                    // says: `resolve_bindings` already rejected an unknown
                    // qname with E-LOAD-010, and `check_sources_servable`
                    // already refused an unsupplied one at entry. Arriving
                    // here is a driver wiring bug, never content's fault.
                    return Err(err(format!(
                        "binding `{}`: :const {qname} resolved at load but the \
                         defines environment holds no value for it — the \
                         binding vocabulary and the defines environment have \
                         drifted apart",
                        binding.name
                    )));
                };
                env.insert(binding.name.clone(), value.clone());
                continue;
            }
            // §2.5: the current tick, as `Int`. The driver knows its own
            // tick number, so this is served rather than refused.
            BindSource::Tick => {
                env.insert(binding.name.clone(), Value::Int(tick));
                continue;
            }
            // `tick mod k` for a LITERAL k (D68), which needs only the tick.
            // `k > 0` is guaranteed by `E-PARSE-014` at load.
            BindSource::TickInCycle(length) => {
                env.insert(binding.name.clone(), Value::Int(tick.rem_euclid(*length)));
                continue;
            }
            // Resolved AFTER this pass, in declaration order, against the
            // external bindings above (§2.5/§4.2) — see `resolve_expr_bindings`.
            BindSource::Expr(_) => continue,
            // Refused at entry by `check_sources_servable`; reaching here
            // would mean that check and this match had drifted apart.
            BindSource::Metric(_) | BindSource::Year | BindSource::TickOfYear => {
                return Err(err(format!(
                    "binding `{}`: unservable source reached the subject loop —                      check_sources_servable and bind_subject have drifted",
                    binding.name
                )))
            }
        };
        // T3 #491, OQ-J: a `currency`-declared field lives in the substrate's
        // SEPARATE i128 map — `node_attribute`'s f64 lane never holds it, so
        // the field's declared type must be consulted BEFORE choosing which
        // read method to call, not after. Every other declared type is
        // unchanged: still `node_attribute` + `bind_field_value`.
        let value = if matches!(
            types.fields.get(qname).map(|decl| &decl.ty),
            Some(BslType::Currency)
        ) {
            match graph.node_attribute_currency(subject, qname) {
                Ok(currency) => Value::Currency(currency),
                Err(graph_err) => field_default_or_err(binding, subject, &graph_err)?,
            }
        } else {
            match graph.node_attribute(subject, qname) {
                Ok(value) => bind_field_value(qname, value, types, enums)?,
                Err(graph_err) => field_default_or_err(binding, subject, &graph_err)?,
            }
        };
        env.insert(binding.name.clone(), value);
    }
    Ok(env)
}

/// The shared "unwritten field" fallback both branches of the `:field`
/// bind-source arm above use: an `:optional` binding with no stored value
/// falls back to its declared `:default`; a required one that was never
/// written propagates the substrate's loud error (III.11 — absence is not
/// zero). Extracted (T3 #491, OQ-J) so the Currency-lane fork and the f64
/// fork share ONE copy of this fallback rather than two that could drift.
fn field_default_or_err(
    binding: &BindingDecl,
    subject: NodeId,
    graph_err: &babylon_graph::substrate::GraphError,
) -> Result<Value, TickError> {
    let Some(default) = binding.default.as_ref().filter(|_| binding.optional) else {
        return Err(err(format!("subject {subject:?}: {}", graph_err.message)));
    };
    atom_to_value(default).ok_or_else(|| {
        err(format!(
            "binding `{}` has a :default that is not a recognized literal \
             (numeric, boolean, or currency)",
            binding.name
        ))
    })
}

/// **§2.13 addendum (D101).** Render one stored field value through its
/// declared type. Every non-enum field is unchanged: `Value::Real(stored)`,
/// exactly as before this section existed. An `enum`-declared field's
/// stored ordinal is rendered back to its member as a `Value::Enum` — the
/// write/read law's read half (§2.13): a `when` guard or any other
/// comparison always compares members, never an ordinal a rule author
/// could confuse for magnitude.
///
/// A stored value that fails to round-trip against the field's declared
/// type — non-integral, negative, or `>=` the type's member count — is a
/// LOUD integrity failure, never a clamp and never a silently-substituted
/// default member (Constitution III.11 binds a corrupted store exactly as
/// it binds a malformed load). This is the one place a write bug elsewhere
/// (or a hand-corrupted store, exercised by this task's own mutation-style
/// integrity test) would surface.
///
/// `pub(crate)` (D102 discharge, Task 1 P27 territory-port train):
/// `evaluator::field_of_node` reuses this EXACT rendering for `field-of`
/// over an enum-declared field, rather than re-deriving the ordinal→member
/// conversion a second time — the same "reused, not re-derived" precedent
/// `namespace_to_node_type`'s own doc set for the sibling §2.10 discipline.
pub(crate) fn bind_field_value(
    qname: &str,
    stored: f64,
    types: &TypeEnv,
    enums: &EnumRegistry,
) -> Result<Value, TickError> {
    let Some(decl) = types.fields.get(qname) else {
        // Unregistered field: unchanged behavior. `resolve_bindings`
        // already rejects an unknown qname at load (E-LOAD-010); reaching
        // here with one is a driver wiring bug, not content's fault — the
        // SAME "defense in depth, not a reachable content error" shape
        // `scenario.rs::attribute_value`'s own catch-all documents.
        return Ok(Value::Real(stored));
    };
    let BslType::Enum(ty) = decl.ty else {
        return Ok(Value::Real(stored));
    };
    if !stored.is_finite() || stored.fract() != 0.0 || stored < 0.0 {
        return Err(err(format!(
            "field {qname}: the stored value {stored} is not a valid enum \
             ordinal (non-integral or negative) — a loud integrity failure, \
             never a clamp and never a silently-substituted default member \
             (§2.13)"
        )));
    }
    let member_count = enums.member_count(ty);
    if member_count == 0 {
        // `EnumRegistry::declare` refuses an empty member list
        // (types.rs:126), so `member_count == 0` can only mean `ty` was
        // never minted by THIS registry — a driver wiring bug (e.g. a
        // `TypeEnv` resolved against one registry, handed to this
        // function alongside a DIFFERENT one), never a legitimately empty
        // `defenum`. Caught HERE, before the range check below folds it
        // into `member_count as f64`: `stored >= 0.0` is true for every
        // non-negative stored value already accepted above, so every one
        // of them would otherwise silently reach the range-error branch's
        // `enums.name(ty)` call — an out-of-bounds index PANIC for a `ty`
        // this registry never minted (`EnumRegistry::name`'s own doc
        // names this exact caller-bug shape). #528 fix round Item A
        // (Copilot finding, confirmed real).
        return Err(err(format!(
            "field {qname}: enum type id {} was not minted by the \
             executing registry — a driver wiring bug, not a content \
             error (§2.13)",
            ty.0
        )));
    }
    #[allow(clippy::cast_precision_loss)]
    let member_count = member_count as f64;
    if stored >= member_count {
        return Err(err(format!(
            "field {qname}: the stored ordinal {stored} is outside \
             {}'s [0, {member_count}) member range — a loud integrity \
             failure, never a clamp (§2.13)",
            enums.name(ty)
        )));
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ordinal = stored as u32;
    let member = enums
        .member(ty, ordinal)
        .expect("range-checked against enums' own member_count above");
    Ok(Value::Enum {
        enum_type: enums.name(ty).to_owned(),
        member: member.to_owned(),
    })
}

fn atom_to_value(atom: &Atom) -> Option<Value> {
    match atom {
        Atom::Int(value) => Some(Value::Int(*value)),
        Atom::Scaled(scaled) => {
            // `unscaled / 10^scale`, the canonical minimal-scale form. Both
            // operands are exact in f64 at these magnitudes (scale <= 9).
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            Some(Value::Real(
                numerator / 10_f64.powi(i32::from(scaled.scale)),
            ))
        }
        Atom::Bool(value) => Some(Value::Bool(*value)),
        // T3 #491, OQ-J: Half 2 landed — a Currency `:default` literal now
        // carries into `Value::Currency`, the SAME "open the door" this
        // train opened at `scenario.rs::load_defconst`'s identical arm and
        // `scenario.rs::attribute_value_currency`'s node-attribute lane, so
        // the literal's legality does not depend on which of the three
        // doors it entered by.
        Atom::Currency(c) => Some(Value::Currency(*c)),
        _ => None,
    }
}

/// Refuse, **at entry and by name**, every `<bind-src>` slice 1 cannot
/// honestly serve (§2.5). The alternative — letting the subject loop skip
/// them — is the inert-no-caller shape: the rule loads, then dies mid-guard
/// with a generic unbound-variable error that names neither the source nor
/// the reason.
///
/// What slice 1 CAN serve, and why:
///
/// - `:field` — the substrate holds it.
/// - `:tick` and `:tick-in-cycle` — the driver knows its own tick number,
///   and D68's cycle length is a literal, so both are exact.
/// - `:expr` — self-contained: it needs only the bound environment, the
///   evaluator and the fuel meter, all of which the per-subject path has.
/// - `:const` — **when, and only when, the driver supplied the coefficient.**
///   §4.2 puts the defines environment in a rule's environment alongside the
///   graph and the tick, and the driver now passes one in. Until the Phase-2
///   `GameDefines`/`defines.yaml` registry exists, its contents are the
///   scenario's `(defconst …)` rows. A rule reading a coefficient the
///   environment does not hold is refused BY NAME below, never defaulted.
///
/// What it cannot, and why each is a *seam* rather than a bug:
///
/// - `:metric` — §2.11 metrics come from a registered kernel provider, and
///   slice 1 registers none.
/// - `:year` / `:tick-of-year` — §2.5 pins the epoch and the ticks-per-year
///   figure to the kernel's clock in the determinism contract, and slice 1
///   pins neither. Serving them would mean inventing a calendar, which is
///   exactly the guess III.11 forbids. `:tick`/`:tick-in-cycle` need no
///   epoch, which is why they are served and these are not.
fn check_sources_servable(bindings: &[BindingDecl], defines: &DefinesEnv) -> Result<(), TickError> {
    for binding in bindings {
        let reason = match &binding.source {
            // Servable exactly when the driver supplied the coefficient.
            // Checked AT ENTRY, by name, for the same reason as every other
            // row: a rule that dies mid-guard on an unbound variable names
            // neither the source nor the reason.
            BindSource::Const(qname) if !defines.contains_key(qname) => Some(format!(
                ":const {qname} — the driver supplied no such coefficient. The \
                 defines environment is the scenario's (defconst …) rows until \
                 the Phase-2 GameDefines/defines.yaml registry lands"
            )),
            BindSource::Metric(name) => Some(format!(
                ":metric {name} — slice 1 registers no metric provider; §2.11 \
                 providers are Phase-2 kernel services"
            )),
            BindSource::Year => Some(
                ":year — slice 1 pins no epoch; §2.5 puts the epoch and the \
                 ticks-per-year figure in the kernel's clock (determinism \
                 contract), and inventing one here would be a guess"
                    .to_owned(),
            ),
            BindSource::TickOfYear => Some(
                ":tick-of-year — slice 1 pins no ticks-per-year figure (§2.5, \
                 as for :year)"
                    .to_owned(),
            ),
            // Every servable source, including a `:const` the driver DID
            // supply — the guard arm above owns the unsupplied case.
            BindSource::Const(_)
            | BindSource::Field(_)
            | BindSource::Tick
            | BindSource::TickInCycle(_)
            | BindSource::Expr(_) => None,
        };
        if let Some(reason) = reason {
            return Err(err(format!(
                "binding `{}` is not servable in slice 1: {reason}",
                binding.name
            )));
        }
    }
    Ok(())
}

/// Run one rule over every subject of its type.
///
/// **The §4.2 chapter C4 pre-state law (Task 12, P27 Phase 2 query-
/// evaluation plan):** *"All firings of one rule observe the same
/// pre-state … and the effects they collect are applied in that subject
/// order."* This function runs in two passes over `subjects` rather than
/// one, and that split IS the repair, not an optimisation:
///
/// - **Pass 1 — collect**, via `collect_pass`. **Repaired (CT4P
///   hardening train, issue #525, item A1):** this doc used to record an
///   admitted gap — "NLL re-acquires a `&mut` reborrow per subject … so
///   nothing at the TYPE level stops a future Pass-1 caller from mutating
///   between subjects. That Pass 1's *loop* never calls a mutating method
///   between subjects is a convention … not the compiler." `collect_pass`
///   closes that gap by taking `graph: &dyn GraphSubstrate` — an IMMUTABLE
///   substrate, for the whole loop, not just for one callee. The type
///   system now enforces the pre-state law for every subject in Pass 1: no
///   call inside `collect_pass`'s loop can mutate `graph`, because nothing
///   in scope holds a `&mut` to it. `run_tick` reborrows `&*graph` ONCE,
///   for the single call into `collect_pass` — the two existing
///   pre-state tests
///   (`all_firings_of_one_rule_observe_the_same_pre_state`,
///   `accumulation_into_a_shared_target_reduces_in_subject_order_and_keeps_every_contribution`)
///   are now redundancy on top of a type-level guarantee, not the only
///   defence. `update-node`'s writes come out as
///   [`crate::structural_verbs::PendingWrite`]s, appended to one flat,
///   RULE-wide list in subject order.
/// - **Pass 2 — apply.** `run_tick` reborrows `&mut *graph` for this pass,
///   after `collect_pass`'s immutable borrow has ended. Each collected
///   write applies in the order it was collected — subject order outer,
///   source order inner, by construction of the flat list — and `add`/
///   `sub`/`scale` read the target's CURRENT value HERE, at apply time
///   (D-row Q2), which is what lets several subjects each contribute to
///   one shared carrier without losing any contribution.
///
/// **Scope.** Only `update-node` participates in this two-pass split.
/// `emit`, `guard` and `for-each` are served inside Pass 1 (`emit` never
/// touches the graph; its payload evaluates against the same pre-state).
/// The six graph-shape verbs (`add-node`, `remove-node`, `add-edge`,
/// `remove-edge`, `add-hyperedge`, `remove-hyperedge`) are NOT served by
/// this function — nothing in the landed rule-pack estate uses them
/// (verified by grep over `rust/crates/babylon-tick/content/rules/*.bsl`),
/// and correctly deferring a MINTING verb needs a placeholder-id scheme
/// this plan does not specify. A rule that needs one is a declared,
/// escalated gap — see `EffectExecutor::collect_effects`'s own doc.
///
/// # Errors
///
/// [`TickError`] if the subject type cannot be derived, a rule reads a
/// coefficient `defines` does not hold, a required field was never written,
/// the guard does not evaluate to a `Bool`, evaluation or collection fails,
/// or a collected write fails to apply.
///
/// # Private deterministic draw context
///
/// The loaded contract's governed rule id, `node_content_ids`, and `session`
/// carry no weight of their own here — `run_tick` only forwards the latter
/// two to `collect_pass`, which builds one
/// [`crate::intrinsic_host::DrawContext`] per subject. Amendment AJ permits
/// only a compiled finite kernel to consume that context; BSL authors cannot
/// invoke a draw. The rule identity has no parallel caller-supplied seam:
/// the loaded contract also owns observed write attribution.
///
/// `node_content_ids: Option<&HashMap<NodeId, String>>` fixes the standard
/// hasher rather than generalizing over `S: BuildHasher` (`clippy::
/// implicit_hasher`'s own preferred fix, and this crate's own precedent —
/// `bind_environment`/`resolve_expr_bindings`, `rule_pipeline.rs`):
/// unlike those two, this reference is stored VERBATIM into
/// [`crate::intrinsic_host::DrawIdentityContext::V1`], which crosses
/// the [`IntrinsicHost`] trait boundary — generalizing here would cascade
/// the hasher type parameter through `DrawContext`, `IntrinsicCallCtx`,
/// `EvalEnv`, and every `IntrinsicHost` impl for a map that, in every
/// production call site (`babylon_bsl::scenario::invert_content_ids`), is
/// always the default `HashMap::new()`/`RandomState` — a caller-supplied
/// alternate hasher is not a real requirement here, so the wide
/// generalization would buy nothing (§ Simplicity, no abstraction for
/// single-use code).
///
/// **`Option`, not a bare reference (review round 2, #576 I2).** `None`
/// means "no scenario was hydrated in this call path" (this crate's own
/// hand-built `MemoryGraph` fixtures); `Some(map)` means "hydrated", even
/// when `map` is empty — see [`crate::intrinsic_host::DrawContext::
/// node_content_ids`]'s own doc for the failure scenario the type
/// distinction closes that an `is_empty()`-gated bare reference could not.
#[allow(clippy::too_many_arguments, clippy::implicit_hasher)]
pub fn run_tick(
    loaded: &LoadedRule,
    types: &TypeEnv,
    enums: &EnumRegistry,
    host: &dyn IntrinsicHost,
    graph: &mut dyn GraphSubstrate,
    sink: &mut dyn EventSink,
    costs: &crate::fuel::IntrinsicCosts,
    defines: &DefinesEnv,
    tick: i64,
    node_content_ids: Option<&HashMap<NodeId, String>>,
    session: &SessionId,
    // The scenario's closed vocabulary, when one was declared — the D29
    // owner-kind filter `subject_type_of` applies (Community port train,
    // Task 6). `None` is the registry-free unit-test lane, where the
    // filter cannot run and every binding namespace counts, exactly as
    // before this parameter existed.
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
) -> Result<TickOutcome, TickError> {
    run_tick_with_observer(
        loaded,
        types,
        enums,
        host,
        graph,
        sink,
        costs,
        defines,
        tick,
        node_content_ids,
        RngSeedContext::V1 { session },
        None,
        vocabulary,
        None,
    )
}

/// Run one rule while observing each successful apply-boundary mutation.
///
/// Observation is passive: it does not alter evaluation, state, fuel, or
/// event behavior. Callers publish the resulting log only after their own
/// transaction boundary succeeds.
///
/// # Errors
///
/// The same [`TickError`] conditions as [`run_tick`].
#[allow(clippy::too_many_arguments, clippy::implicit_hasher)]
pub fn run_tick_observed<G>(
    loaded: &LoadedRule,
    types: &TypeEnv,
    enums: &EnumRegistry,
    host: &dyn IntrinsicHost,
    graph: &mut G,
    sink: &mut dyn EventSink,
    costs: &crate::fuel::IntrinsicCosts,
    defines: &DefinesEnv,
    tick: i64,
    node_content_ids: Option<&HashMap<NodeId, String>>,
    rng_seed: RngSeedContext<'_>,
    stable_resolver: Option<&StableElementResolverV1>,
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
    observer: &mut dyn WriteObserver,
) -> Result<TickOutcome, TickError>
where
    G: GraphSubstrate + CanonicalState,
{
    if matches!(rng_seed, RngSeedContext::V2 { .. }) {
        let resolver = stable_resolver.ok_or_else(|| {
            err("finite-kernel V2 requires a sealed StableElementResolverV1".to_owned())
        })?;
        resolver.validate_topology(graph).map_err(|error| {
            err(format!(
                "finite-kernel V2 sealed topology refused: {error:?}"
            ))
        })?;
    }
    run_tick_with_observer(
        loaded,
        types,
        enums,
        host,
        graph,
        sink,
        costs,
        defines,
        tick,
        node_content_ids,
        rng_seed,
        stable_resolver,
        vocabulary,
        Some(observer),
    )
}

#[allow(clippy::too_many_arguments, clippy::implicit_hasher)]
fn run_tick_with_observer(
    loaded: &LoadedRule,
    types: &TypeEnv,
    enums: &EnumRegistry,
    host: &dyn IntrinsicHost,
    graph: &mut dyn GraphSubstrate,
    sink: &mut dyn EventSink,
    costs: &crate::fuel::IntrinsicCosts,
    defines: &DefinesEnv,
    tick: i64,
    node_content_ids: Option<&HashMap<NodeId, String>>,
    rng_seed: RngSeedContext<'_>,
    stable_resolver: Option<&StableElementResolverV1>,
    vocabulary: Option<&crate::vocabulary::ClosedVocabulary>,
    observer: Option<&mut dyn WriteObserver>,
) -> Result<TickOutcome, TickError> {
    check_sources_servable(&loaded.bindings, defines)?;
    let subject_type = subject_type_of_bindings(&loaded.bindings, vocabulary)?;
    let (guard, effects) = guard_and_effects(&loaded.rule)?;
    let subjects = graph.nodes(&subject_type);

    // ---- Pass 1: collect. `collect_pass` takes `&dyn GraphSubstrate` — an
    // IMMUTABLE reborrow of `*graph`, held for the whole call — so the
    // borrow checker, not a convention, is what stops any subject in this
    // pass from observing another subject's write (A1, CT4P hardening
    // train, issue #525; see this function's own doc for the repair).
    let (all_pending, fired, kernel_realizations, event_provenance) = collect_pass(
        &*graph,
        &subjects,
        loaded,
        guard,
        effects,
        types,
        enums,
        host,
        sink,
        costs,
        defines,
        tick,
        node_content_ids,
        rng_seed,
        stable_resolver,
    )?;

    // ---- Pass 2: apply, in the order collected (subject order outer,
    // source order inner) — `graph` is mutable again, `collect_pass`'s
    // immutable borrow having already ended. ----
    if let Some(observer) = observer {
        let mut applier = EffectExecutor::observed(
            types,
            enums,
            None,
            loaded.contract.rule_id.as_str(),
            observer,
        );
        for write in &all_pending {
            applier.apply_pending_write(write, graph)?;
        }
    } else {
        let mut applier = EffectExecutor::new(types, enums, None);
        for write in &all_pending {
            applier.apply_pending_write(write, graph)?;
        }
    }

    Ok(TickOutcome {
        subject_type,
        considered: subjects.len(),
        fired,
        kernel_realizations,
        event_provenance,
    })
}

/// Pass 1 of [`run_tick`]: collect every subject's guard/effects against the
/// SAME pre-tick state, without ever holding a mutable graph.
///
/// **This signature IS the A1 repair (CT4P hardening train, issue #525).**
/// Before this function existed, Pass 1 was a loop inlined in `run_tick`,
/// reborrowing `&*graph` fresh each iteration from a `&mut dyn
/// GraphSubstrate` parameter that stayed in scope for the whole function —
/// so nothing at the TYPE level stopped a mutation of `graph` between
/// subjects; only this module's own tests enforced it, by convention.
/// Extracting the loop into its own function taking `graph: &dyn
/// GraphSubstrate` moves the enforcement to the type: there is no `&mut` to
/// `graph` anywhere in this function's scope, so the compiler — not a
/// reviewer, not a test — refuses any code path that would try to mutate
/// the GRAPH SUBSTRATE mid-loop.
///
/// **Scope of that guarantee, named explicitly (verifier fix round,
/// NOTE-1):** it covers `graph` alone. `sink: &mut dyn EventSink` IS
/// mutable and IS in scope for the whole loop — `emit` legitimately
/// collects into it every iteration, and that is by design (`emit` never
/// touches the graph, §2.8, so it has nothing to do with the pre-state
/// law this function's type signature enforces). The claim above is never
/// "this function performs no side effect between subjects"; it is
/// narrower and load-bearing precisely because it is narrower: "this
/// function cannot OBSERVE THE GRAPH differently between subjects."
///
/// Returns the collected [`crate::structural_verbs::PendingWrite`]s in
/// subject order (source order within each subject, by construction of the
/// flat list) alongside how many subjects fired — [`run_tick`]'s
/// [`TickOutcome::fired`] needs the count, not just the writes, since a
/// subject can fire with zero writes (an `emit`-only effect list).
///
/// # Errors
///
/// [`TickError`] if a rule reads a coefficient `defines` does not hold, a
/// required field was never written, the guard does not evaluate to a
/// `Bool`, or collection fails.
///
/// `node_content_ids`'s fixed (non-generalized) hasher: see [`run_tick`]'s
/// own doc — the same reasoning applies verbatim, one level down the call
/// stack.
#[allow(
    clippy::too_many_arguments,
    clippy::implicit_hasher,
    clippy::too_many_lines,
    clippy::type_complexity
)]
fn collect_pass(
    graph: &dyn GraphSubstrate,
    subjects: &[NodeId],
    loaded: &LoadedRule,
    guard: Option<&SExpr>,
    effects: &[SExpr],
    types: &TypeEnv,
    enums: &EnumRegistry,
    host: &dyn IntrinsicHost,
    sink: &mut dyn EventSink,
    costs: &crate::fuel::IntrinsicCosts,
    defines: &DefinesEnv,
    tick: i64,
    // Private deterministic draw context: `loaded.contract.rule_id` is
    // `domain`, `node_content_ids`
    // resolves `subject` (and any `it`/`:as` element `eval_intrinsic` meets)
    // to a Task-3 content id, and `session` is `DrawContext`'s own non-
    // operand half (D69). All three are constant for the whole rule; only
    // `subject` varies per iteration below.
    node_content_ids: Option<&HashMap<NodeId, String>>,
    rng_seed: RngSeedContext<'_>,
    stable_resolver: Option<&StableElementResolverV1>,
) -> Result<
    (
        Vec<crate::structural_verbs::PendingWrite>,
        usize,
        Vec<KernelRealizationV1>,
        Vec<EmittedEventProvenanceV1>,
    ),
    TickError,
> {
    let mut fired = 0_usize;
    let mut all_pending: Vec<crate::structural_verbs::PendingWrite> = Vec::new();
    let mut kernel_realizations = Vec::new();
    let mut event_provenance = Vec::new();
    let mut collected_events = crate::structural_verbs::CollectingSink::default();
    // D69: never negative in practice (`run_tick`'s own callers start at
    // tick 1 and only ever increment), but checked rather than cast blind
    // — a silently wrapped tick would corrupt every draw key this rule's
    // subjects derive (III.11).
    let draw_tick = u64::try_from(tick).map_err(|_| {
        err(format!(
            "tick {tick} is negative — DrawContext (plan §3.5) requires a \
             non-negative tick, and III.7/III.11 forbid silently wrapping it"
        ))
    })?;
    let replay_rule =
        replay_rule_identity(rng_seed, stable_resolver, loaded.contract.rule_id.as_str())?;

    for subject in subjects {
        let mut values = bind_subject(
            *subject,
            &loaded.bindings,
            graph,
            defines,
            tick,
            types,
            enums,
        )?;
        // Per-subject budget, from the rule's DECLARED `:fuel` — not from
        // `static_bound`, which is the load-time proof that the rule fits
        // rather than the allowance it runs under. Metering on the computed
        // bound would couple runtime to whatever the checker returns and
        // would under-fund a rule whose author allotted more.
        //
        // Per SUBJECT, because the §3.7 bound is per-subject: one shared
        // meter would make a rule's admissibility depend on how many nodes
        // happened to exist, which is not a property of the rule.
        let mut fuel = loaded.declared_fuel;

        // The private draw carrier uses THIS subject's stable content
        // identity, never its `NodeId` handle — keying
        // on the handle would be replay-deterministic but insertion-
        // history-dependent (plan §3.4), the exact butterfly ADR176 r20
        // forbids.
        //
        // **Review round 2 (#576 I2): `node_content_ids` is `Option`-typed,
        // not `is_empty()`-gated.** `None` is the actual, observed shape of
        // every hand-built `MemoryGraph` fixture in this crate's own tests
        // (none go through scenario hydration) — there, a `NodeId`'s own
        // `Debug` rendering stands in, honestly: no scenario means no
        // declared name, not a hydration bug. Against `Some(map)` a miss is
        // a hard `TickError`, `map.is_empty()` or not — see
        // `evaluator::element_content_id`'s own doc for the full
        // cross-file invariant this closes: every scenario-hydrated
        // `NodeId` is named (`scenario::invert_content_ids`), and the only
        // OTHER way to mint one — the six graph-shape verbs,
        // `structural_verbs::DEFERRED_SHAPE_VERBS` — is refused
        // unconditionally at load (`check_no_deferred_shape_verbs`,
        // `rule_pipeline.rs:269`), a gate a NAMED FUTURE TASK will lift.
        // Whoever lifts it must also update `node_content_ids` for any
        // mid-tick-minted node, or this hard error is the trip wire that
        // catches the gap — the alternative (an unconditional fallback)
        // would have silently fed a raw, insertion-order-dependent
        // `NodeId` handle into the finite-kernel draw key, precisely the
        // ADR176 r20 butterfly the content-id design exists to prevent.
        //
        // `Cow` (review round 2, #576 M2): the found-in-map arm BORROWS
        // straight out of `node_content_ids` (which outlives this loop)
        // instead of allocating a fresh `String` per subject per rule per
        // tick — only the Debug-rendering fallback arm allocates.
        let v1_subject = match rng_seed {
            RngSeedContext::V1 { .. } => Some(v1_subject_identity(*subject, node_content_ids)?),
            RngSeedContext::V2 { .. } => None,
        };
        let stable_subject = match rng_seed {
            RngSeedContext::V1 { .. } => None,
            RngSeedContext::V2 { .. } => {
                let (_, resolver) = replay_rule
                    .as_ref()
                    .expect("V2 rule identity checked before subject loop");
                Some(
                    resolver
                        .node_key(*subject)
                        .map_err(|error| {
                            err(format!(
                                "V2 subject has no sealed stable identity: {error:?}"
                            ))
                        })?
                        .clone(),
                )
            }
        };
        let identity = match rng_seed {
            RngSeedContext::V1 { session } => DrawIdentityContext::V1 {
                session,
                domain: loaded.contract.rule_id.as_str(),
                subject: v1_subject
                    .as_deref()
                    .expect("V1 always resolves its subject"),
                node_content_ids,
            },
            RngSeedContext::V2 { session, seed } => {
                let (domain, resolver) = replay_rule
                    .as_ref()
                    .expect("V2 rule identity checked before subject loop");
                DrawIdentityContext::V2 {
                    session,
                    seed,
                    domain: domain.clone(),
                    resolver,
                    subject: stable_subject
                        .as_ref()
                        .expect("V2 stable subject resolved above")
                        .clone(),
                }
            }
        };
        let draw_context = DrawContext {
            identity,
            tick: draw_tick,
        };

        // §2.5/§4.2: `:expr` bindings resolve in DECLARATION order against
        // the bindings already resolved, and §4.5 charges each expression
        // ONCE, through this subject's meter. Doing it here — after the
        // external sources, before the guard — is what makes a computed
        // binding an abbreviation rather than a sequencing construct: it
        // cannot observe an effect, because no effect has run.
        //
        // `Some(graph)` (Territory port train, Task 6 — the P6 closure):
        // this function's own `graph` parameter is a live, read-only
        // `&dyn GraphSubstrate` for the whole of `collect_pass` (this
        // function's own doc), so a `:expr` binding whose body contains a
        // query form (`territory/p3-spillover`'s `inflow`) now resolves
        // through the same substrate the guard/effects environment below
        // uses — never a graph-less environment silently missing it.
        //
        // `draw_context` is constructed before expression resolution so the
        // same subject identity is available to the later engine-private
        // finite-kernel realization. No author expression can consume a draw.
        crate::rule_pipeline::resolve_expr_bindings(
            &loaded.bindings,
            &mut values,
            costs,
            types,
            enums,
            Some(graph),
            Some(&draw_context),
            host,
            &mut fuel,
        )?;

        let env = EvalEnv {
            bindings: values,
            intrinsic_costs: costs,
            // A real, live reference — safe to hold alongside `run_tick`'s
            // later mutable use of `graph` in Pass 2 precisely because this
            // function never performs one: `graph` here has no `&mut` form
            // anywhere in scope, by the SIGNATURE, not by discipline.
            graph: Some(graph),
            // D102 discharge (Task 1, P27 territory-port train): the same
            // registries `bind_subject` above already resolved this
            // subject's `:field` bindings against, so `field-of` over an
            // enum-declared field renders through the identical path.
            types: Some(types),
            enums: Some(enums),
            elements: Vec::new(),
            draw_context: Some(&draw_context),
        };

        if let Some(guard) = guard {
            match evaluate(guard, &env, host, &mut fuel)? {
                Value::Bool(true) => {}
                Value::Bool(false) => continue,
                other => {
                    return Err(err(format!(
                        "a (when …) guard must evaluate to Bool, got {other:?}"
                    )))
                }
            }
        }

        // `vocabulary_registry: None` — the collect path never reaches a
        // minting verb (`add-node`/`add-edge`/`add-hyperedge`) at all: every
        // rule `collect_item` sees here already survived
        // `rule_pipeline::check_no_deferred_shape_verbs`'s unconditional
        // load-time refusal of the six graph-shape verbs, so threading a
        // registry through this specific construction site would change
        // nothing observable (Task 8, Organization foundation plan — see
        // `EffectExecutor`'s own field doc).
        let mut executor = EffectExecutor::new(types, enums, None);
        let events_before = collected_events.events.len();
        let pending = if let Some(kernel) = &loaded.kernel {
            charge(&mut fuel, FINITE_KERNEL_DRAW_BASE)?;
            let masses = evaluate_kernel_masses(kernel, &env, host, &mut fuel)?;
            let call_context = IntrinsicCallCtx {
                draw_context: Some(&draw_context),
                active_elements: Vec::new(),
            };
            let draw = draw_finite_kernel_ticket(&kernel.sample, kernel.slot, &call_context)?;
            let (session, seed) = match rng_seed {
                RngSeedContext::V2 { session, seed } => (session, seed),
                RngSeedContext::V1 { .. } => {
                    return Err(err(
                        "finite-kernel realization requires the sealed V2 replay path; V1 has no signed replay seed",
                    ));
                }
            };
            let instance = KernelInstanceIdentityV1 {
                replay_session: session.as_bytes().to_vec(),
                replay_seed: seed.to_be_bytes(),
                tick,
                rule_id: loaded.contract.rule_id.clone(),
                subject: stable_subject
                    .as_ref()
                    .expect("a V2 choice has a stable subject")
                    .clone(),
                active_elements: Vec::new(),
            };
            let realization = crate::probability::realize_kernel(&instance, kernel, &masses, draw)
                .map_err(|error| err(error.to_string()))?;
            let selected = kernel
                .branches
                .iter()
                .position(|branch| branch.member == realization.selected_outcome)
                .ok_or_else(|| err("realized outcome is absent from its compiled kernel"))?;
            let pending = collect_selected_kernel_effects(
                &mut executor,
                effects,
                kernel,
                selected,
                &env,
                host,
                &mut collected_events,
                &mut fuel,
            )?;
            kernel_realizations.push(realization);
            pending
        } else {
            executor.collect_effects(effects, &env, host, &mut collected_events, &mut fuel)?
        };
        if let Some(subject_identity) = stable_subject {
            for _ in events_before..collected_events.events.len() {
                event_provenance.push(EmittedEventProvenanceV1 {
                    subject: subject_identity.clone(),
                });
            }
        }
        all_pending.extend(pending);
        fired += 1;
    }

    for (event_type, payload) in collected_events.events {
        sink.emit(&event_type, payload);
    }
    Ok((all_pending, fired, kernel_realizations, event_provenance))
}

#[allow(clippy::too_many_arguments)]
fn collect_selected_kernel_effects(
    executor: &mut EffectExecutor<'_>,
    effects: &[SExpr],
    kernel: &crate::probability::FiniteKernelV1,
    selected: usize,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    sink: &mut dyn EventSink,
    fuel: &mut u64,
) -> Result<Vec<crate::structural_verbs::PendingWrite>, EvalError> {
    let choose_component =
        kernel.form_path.last().copied().ok_or_else(|| {
            EvalError::plain("compiled choose has no effect-list path".to_owned())
        })?;
    let choose_index = usize::try_from(choose_component)
        .ok()
        .and_then(|index| index.checked_sub(1))
        .ok_or_else(|| {
            EvalError::plain("compiled choose path does not name an effect item".to_owned())
        })?;
    let branch = kernel.branches.get(selected).ok_or_else(|| {
        EvalError::plain("selected finite-kernel ordinal is outside its branch list".to_owned())
    })?;
    let mut pending = Vec::new();
    for (index, effect) in effects.iter().enumerate() {
        let items = if index == choose_index {
            branch.effects.as_slice()
        } else {
            std::slice::from_ref(effect)
        };
        pending.extend(executor.collect_effects(items, env, host, sink, fuel)?);
    }
    Ok(pending)
}

fn replay_rule_identity<'a>(
    rng_seed: RngSeedContext<'_>,
    resolver: Option<&'a StableElementResolverV1>,
    rule_id: &str,
) -> Result<Option<(RngDomainV2, &'a StableElementResolverV1)>, TickError> {
    match rng_seed {
        RngSeedContext::V1 { .. } => Ok(None),
        RngSeedContext::V2 { .. } => {
            let domain = RngDomainV2::try_from(rule_id).map_err(|error| {
                err(format!(
                    "finite-kernel V2 firing-rule domain refused: {error:?}"
                ))
            })?;
            let stable = resolver.ok_or_else(|| {
                err("finite-kernel V2 requires a sealed StableElementResolverV1".to_owned())
            })?;
            Ok(Some((domain, stable)))
        }
    }
}

fn v1_subject_identity(
    subject: NodeId,
    node_content_ids: Option<&HashMap<NodeId, String>>,
) -> Result<std::borrow::Cow<'_, str>, TickError> {
    let Some(map) = node_content_ids else {
        return Ok(std::borrow::Cow::Owned(format!("{subject:?}")));
    };
    map.get(&subject)
        .map(|content_id| std::borrow::Cow::Borrowed(content_id.as_str()))
        .ok_or_else(|| {
            err(format!(
                "subject {subject:?} carries no Task-3 content id, but \
                 node_content_ids IS hydrated ({} entries) — a hydration bug: every \
                 scenario-hydrated node is named (scenario::invert_content_ids), so a \
                 NodeId reaching here means something minted a node outside hydration \
                 without recording its content id (review round 2, #576 I2)",
                map.len()
            ))
        })
}

/// Forecast one adjacent finite kernel/projection pair for one subject by
/// exact detached-state enumeration.
///
/// `kernel_index` addresses the mechanic in the already resolved schedule;
/// the projection must be the immediately following rule and name the same
/// sample. Each branch starts from an independent [`DetachedCopy`], applies
/// the real deterministic effect path, runs the real recognizer for the same
/// subject, and then uses exact ticket pushforward. This API cannot express a
/// cross-sample join, sequence, conjunction, payload distribution, or
/// whole-tick path enumeration.
///
/// # Errors
///
/// Refuses malformed/nonadjacent pairs, a guard-false mechanic (no choice
/// instance exists), any evaluator/effect failure, or a projection that
/// produces a material write despite its loader proof.
#[allow(clippy::too_many_lines)]
pub fn forecast_event_likelihoods<G>(
    resolved_rules: &[LoadedRule],
    kernel_index: usize,
    pre_choice: &G,
    subject: NodeId,
    context: &ForecastContextV1<'_>,
) -> Result<Vec<EventLikelihoodV1>, TickError>
where
    G: GraphSubstrate + DetachedCopy,
{
    crate::probability::analyze_content_set(resolved_rules)
        .map_err(|error| err(error.to_string()))?;
    let mechanic = resolved_rules
        .get(kernel_index)
        .ok_or_else(|| err("forecast kernel index is outside the resolved schedule"))?;
    let projection = resolved_rules
        .get(kernel_index + 1)
        .ok_or_else(|| err("forecast requires an immediately adjacent projection rule"))?;
    let kernel = mechanic
        .kernel
        .as_ref()
        .ok_or_else(|| err("forecast target rule has no compiled finite kernel"))?;
    let projected = projection
        .projection
        .as_ref()
        .ok_or_else(|| err("the immediately following rule is not a finite projection"))?;
    if projected.sample != kernel.sample {
        return Err(err(format!(
            "adjacent projection sample `{}` does not match kernel sample `{}`",
            projected.sample, kernel.sample
        )));
    }

    check_sources_servable(&mechanic.bindings, context.defines)?;
    let (mechanic_guard, mechanic_effects) = guard_and_effects(&mechanic.rule)?;
    let mut mechanic_values = bind_subject(
        subject,
        &mechanic.bindings,
        pre_choice,
        context.defines,
        context.tick,
        context.types,
        context.enums,
    )?;
    let mut mechanic_fuel = mechanic.declared_fuel;
    crate::rule_pipeline::resolve_expr_bindings(
        &mechanic.bindings,
        &mut mechanic_values,
        context.costs,
        context.types,
        context.enums,
        Some(pre_choice),
        None,
        context.host,
        &mut mechanic_fuel,
    )?;
    let mechanic_env = EvalEnv {
        bindings: mechanic_values,
        intrinsic_costs: context.costs,
        graph: Some(pre_choice),
        types: Some(context.types),
        enums: Some(context.enums),
        elements: Vec::new(),
        draw_context: None,
    };
    if let Some(guard) = mechanic_guard {
        match evaluate(guard, &mechanic_env, context.host, &mut mechanic_fuel)? {
            Value::Bool(true) => {}
            Value::Bool(false) => {
                return Err(err(
                    "forecast subject does not fire the kernel mechanic in the pre-choice state",
                ));
            }
            other => {
                return Err(err(format!(
                    "a (when …) guard must evaluate to Bool, got {other:?}"
                )));
            }
        }
    }
    charge(&mut mechanic_fuel, FINITE_KERNEL_DRAW_BASE)?;
    let masses = evaluate_kernel_masses(kernel, &mechanic_env, context.host, &mut mechanic_fuel)?;

    check_sources_servable(&projection.bindings, context.defines)?;
    let (projection_guard, projection_effects) = guard_and_effects(&projection.rule)?;
    let mut branch_projections = Vec::with_capacity(kernel.branches.len());
    for (selected, branch) in kernel.branches.iter().enumerate() {
        let mut branch_state = pre_choice.detached_copy();
        let mut branch_fuel = mechanic_fuel;
        // A mechanic may emit an ordinary deterministic observation beside
        // its one `choose`.  Forecasting needs the mechanic's material
        // writes, but that sibling observation is not the adjacent
        // projection whose preimage we are measuring.  Execute it through a
        // real sink so payload evaluation and fuel remain faithful, then
        // deliberately discard it.  Emits inside a branch are already a
        // loader refusal.
        let mut ignored_mechanic_events = crate::structural_verbs::CollectingSink::default();
        let mut mechanic_executor = EffectExecutor::new(context.types, context.enums, None);
        let pending = collect_selected_kernel_effects(
            &mut mechanic_executor,
            mechanic_effects,
            kernel,
            selected,
            &mechanic_env,
            context.host,
            &mut ignored_mechanic_events,
            &mut branch_fuel,
        )?;
        for write in &pending {
            mechanic_executor.apply_pending_write(write, &mut branch_state)?;
        }

        let mut projection_values = bind_subject(
            subject,
            &projection.bindings,
            &branch_state,
            context.defines,
            context.tick,
            context.types,
            context.enums,
        )?;
        let mut projection_fuel = projection.declared_fuel;
        crate::rule_pipeline::resolve_expr_bindings(
            &projection.bindings,
            &mut projection_values,
            context.costs,
            context.types,
            context.enums,
            Some(&branch_state),
            None,
            context.host,
            &mut projection_fuel,
        )?;
        let projection_env = EvalEnv {
            bindings: projection_values,
            intrinsic_costs: context.costs,
            graph: Some(&branch_state),
            types: Some(context.types),
            enums: Some(context.enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let emits = if let Some(guard) = projection_guard {
            match evaluate(guard, &projection_env, context.host, &mut projection_fuel)? {
                Value::Bool(true) => true,
                Value::Bool(false) => false,
                other => {
                    return Err(err(format!(
                        "a projection (when …) guard must evaluate to Bool, got {other:?}"
                    )));
                }
            }
        } else {
            true
        };
        let mut event_sink = crate::structural_verbs::CollectingSink::default();
        if emits {
            let mut projection_executor =
                EffectExecutor::new(context.types, context.enums, context.vocabulary);
            let writes = projection_executor.collect_effects(
                projection_effects,
                &projection_env,
                context.host,
                &mut event_sink,
                &mut projection_fuel,
            )?;
            if !writes.is_empty() {
                return Err(err(
                    "a finite projection produced a material write despite its emit-only loader proof",
                ));
            }
        }
        branch_projections.push(BranchProjectionV1 {
            outcome: branch.member.clone(),
            event_types: event_sink
                .events
                .into_iter()
                .map(|(event_type, _)| event_type)
                .collect(),
        });
    }
    exact_pushforward(kernel, projected, &masses, &branch_projections)
        .map_err(|error| err(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        bind_field_value, check_sources_servable, run_tick, run_tick_observed,
        subject_type_of_bindings, DefinesEnv,
    };
    use crate::bindings::{BindSource, BindingDecl};
    use crate::evaluator::Value;
    use crate::types::EnumRegistry;
    use crate::write_log::CollectingWriteLog;
    use babylon_kernel::replay::RngSeedContext;
    use babylon_kernel::SessionId;
    use std::collections::HashMap;

    /// The private draw seam's session/content-id parameters for this
    /// module's own hand-built `MemoryGraph`
    /// fixtures — none of them go through scenario hydration, so there is
    /// no Task-3 `node_content_ids` map to thread; `None` exercises
    /// `element_content_id`/`collect_pass`'s documented NodeId-Debug
    /// fallback (`evaluator::element_content_id`'s own doc), which is
    /// correct here precisely because these nodes were never named
    /// (review round 2, #576 I2: `None`, not an empty map — see
    /// `crate::intrinsic_host::DrawIdentityContext::V1`'s own doc).
    fn test_session() -> SessionId {
        SessionId::new("tick-test-session").expect("literal is non-empty")
    }

    fn field(name: &str, qname: &str) -> BindingDecl {
        BindingDecl {
            name: name.to_owned(),
            source: BindSource::Field(qname.to_owned()),
            optional: false,
            default: None,
        }
    }

    fn constant(name: &str, qname: &str) -> BindingDecl {
        BindingDecl {
            name: name.to_owned(),
            source: BindSource::Const(qname.to_owned()),
            optional: false,
            default: None,
        }
    }

    #[test]
    fn a_const_the_driver_supplied_is_servable() {
        let env = DefinesEnv::from([("economy/base-subsistence".to_owned(), Value::Real(0.0005))]);
        assert!(check_sources_servable(
            &[constant("base-subsistence", "economy/base-subsistence")],
            &env
        )
        .is_ok());
    }

    #[test]
    fn a_const_the_driver_did_not_supply_is_refused_by_name_at_entry() {
        // Refused AT ENTRY rather than at the read: a rule that dies
        // mid-guard on an unbound variable names neither the source nor the
        // reason, and a defaulted coefficient would be silent degradation.
        let err = check_sources_servable(
            &[constant("floor-wage", "economy/floor-wage")],
            &DefinesEnv::new(),
        )
        .unwrap_err();
        assert!(
            err.message.contains("economy/floor-wage"),
            "{}",
            err.message
        );
        assert!(
            err.message.contains("supplied no such coefficient"),
            "{}",
            err.message
        );
    }

    #[test]
    fn the_calendar_sources_needing_an_epoch_stay_refused() {
        // A defines environment does not buy `:year` — §2.5 puts the epoch
        // and the ticks-per-year figure in the kernel's clock, and this
        // driver pins neither. Guards the const change against widening the
        // refusal set by accident.
        for source in [BindSource::Year, BindSource::TickOfYear] {
            let decl = BindingDecl {
                name: "when".to_owned(),
                source,
                optional: false,
                default: None,
            };
            assert!(check_sources_servable(&[decl], &DefinesEnv::new()).is_err());
        }
    }

    #[test]
    fn the_subject_type_comes_from_the_field_namespace() {
        let bindings = vec![
            field("wages", "social-class/wages"),
            field("value-produced", "social-class/value-produced"),
        ];
        assert_eq!(
            subject_type_of_bindings(&bindings, None).unwrap(),
            "SOCIAL_CLASS"
        );
    }

    #[test]
    fn fields_spanning_two_namespaces_are_ambiguous_not_guessed() {
        // Two namespaces mean two subject types; picking one silently would
        // run the rule over the wrong population.
        let bindings = vec![
            field("wages", "social-class/wages"),
            field("budget", "organization/budget"),
        ];
        let err = subject_type_of_bindings(&bindings, None).unwrap_err();
        assert!(err.message.contains("ambiguous"), "{}", err.message);
    }

    #[test]
    fn a_rule_with_no_field_binding_names_no_population() {
        let err = subject_type_of_bindings(&[], None).unwrap_err();
        assert!(
            err.message.contains("names no subject type"),
            "{}",
            err.message
        );
    }

    // ---- Task 6 (Community port train, E2b): the D29 owner-kind filter.
    // A `:field` binding is node-scoped and stays node-scoped — an edge's
    // or a hyperedge's declared field reads through `field-of`, never a
    // binding — so a non-node-owned binding is INVISIBLE to this
    // derivation when the closed vocabulary is in force, and a rule with
    // only such bindings names no subject type (the §8c guard's
    // mechanism).

    /// A small vocabulary with one member per kind — the filter's truth
    /// table needs all three kinds present.
    fn d29_vocabulary() -> crate::vocabulary::ClosedVocabulary {
        crate::vocabulary::ClosedVocabulary::new([
            (
                crate::vocabulary::EnumKind::NodeType,
                vec!["SOCIAL_CLASS".to_owned()],
            ),
            (
                crate::vocabulary::EnumKind::EdgeType,
                vec!["SOLIDARITY".to_owned()],
            ),
            (
                crate::vocabulary::EnumKind::HyperedgeType,
                vec!["COMMUNITY".to_owned()],
            ),
        ])
        .expect("disjoint member sets build")
    }

    #[test]
    fn a_hyperedge_owned_field_binding_names_no_subject_type() {
        let vocabulary = d29_vocabulary();
        let bindings = vec![field("heat", "community/heat")];
        let err = subject_type_of_bindings(&bindings, Some(&vocabulary)).unwrap_err();
        assert!(
            err.message.contains("names no subject type"),
            "a hyperedge-owned :field binding is invisible to subject \
             derivation (D29) — the subject-type error, not silence: {}",
            err.message
        );
    }

    #[test]
    fn an_edge_owned_field_binding_names_no_subject_type() {
        // D29 is stated for edge AND hyperedge fields alike; the filter
        // keeps both kinds out of subject derivation.
        let vocabulary = d29_vocabulary();
        let bindings = vec![field("strength", "solidarity/strength")];
        let err = subject_type_of_bindings(&bindings, Some(&vocabulary)).unwrap_err();
        assert!(
            err.message.contains("names no subject type"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_hyperedge_owned_binding_is_invisible_next_to_a_node_owned_one() {
        let vocabulary = d29_vocabulary();
        let bindings = vec![
            field("active", "social-class/active"),
            field("heat", "community/heat"),
        ];
        assert_eq!(
            subject_type_of_bindings(&bindings, Some(&vocabulary)).unwrap(),
            "SOCIAL_CLASS",
            "the node-owned binding alone derives the subject type — the \
             hyperedge-owned one must not make the derivation ambiguous"
        );
    }

    #[test]
    fn without_a_vocabulary_every_namespace_still_counts() {
        // The registry-free lane cannot know an owner kind, so the legacy
        // behavior is pinned: `community` renders to a (meaningless)
        // subject type rather than being filtered. Content drivers always
        // pass `Some`, so this lane is unit-test-only.
        let bindings = vec![field("heat", "community/heat")];
        assert_eq!(
            subject_type_of_bindings(&bindings, None).unwrap(),
            "COMMUNITY"
        );
    }

    // ============================================= Task 12 — the pre-state
    // law: collect-then-apply, within and across firings (§4.2 chapter C4).

    /// A minimal, hand-built `LoadContext` — no scenario text, no `deffield`
    /// forms; the world is built directly against `MemoryGraph` and the
    /// field registry is a plain `TypeEnv`, exactly as
    /// `fundamental_theorem_tick.rs` (the crate's own end-to-end template)
    /// does. `vocabulary_registry: None` skips the closed-vocabulary checks
    /// that need a registry this test has no reason to build.
    struct Fixture {
        types: crate::typecheck::TypeEnv,
        vocabulary: crate::bindings::BindingVocabulary,
        ceilings: crate::fuel::CardinalityCeilings,
        intrinsics: crate::fuel::IntrinsicCosts,
        systems: std::collections::HashSet<String>,
        /// No fixture in this module declares an enum-typed field — an
        /// empty registry is the honest "no `defenum`s in scope" input to
        /// `run_tick`.
        enums: EnumRegistry,
    }

    impl Fixture {
        fn new(
            fields: std::collections::HashMap<String, crate::types::FieldDecl>,
            edge_ceilings: std::collections::HashMap<String, u64>,
        ) -> Self {
            let types = crate::typecheck::TypeEnv {
                fields,
                exemptions: &[],
            };
            let vocabulary = crate::bindings::BindingVocabulary {
                fields: types.fields.keys().cloned().collect(),
                consts: std::collections::HashSet::new(),
                probability_consts: std::collections::HashSet::new(),
                metrics: std::collections::HashSet::new(),
            };
            Self {
                types,
                vocabulary,
                ceilings: crate::fuel::CardinalityCeilings::new(edge_ceilings, HashMap::new()),
                intrinsics: crate::fuel::IntrinsicCosts::default(),
                systems: std::collections::HashSet::from(["geography".to_owned()]),
                enums: EnumRegistry::default(),
            }
        }

        fn load(&self, rule_source: &str, rule_file: &str) -> crate::rule_pipeline::LoadedRule {
            self.try_load(rule_source, rule_file)
                .expect("the rule must pass every load gate")
        }

        /// The non-panicking half of [`Self::load`] — for a test proving a
        /// rule is REFUSED at load (e.g. §2.13's no-arithmetic law, D118),
        /// driving the actual production entry point
        /// (`rule_pipeline::load_rule`), not one of `typecheck`'s checks in
        /// isolation.
        fn try_load(
            &self,
            rule_source: &str,
            rule_file: &str,
        ) -> Result<crate::rule_pipeline::LoadedRule, crate::rule_pipeline::LoadError> {
            let ctx = crate::rule_pipeline::LoadContext {
                vocabulary: &self.vocabulary,
                types: &self.types,
                enums: &self.enums,
                const_values: Box::leak(Box::new(HashMap::new())),
                ceilings: &self.ceilings,
                intrinsics: &self.intrinsics,
                systems: &self.systems,
                vocabulary_registry: None,
                rule_file,
            };
            crate::rule_pipeline::load_rule(rule_source, &ctx)
        }
    }

    fn territory_field(kind: crate::types::FieldKind) -> crate::types::FieldDecl {
        crate::types::FieldDecl {
            ty: crate::types::BslType::Int,
            kind,
        }
    }

    #[test]
    fn observed_write_attribution_uses_the_governed_loaded_rule_identity() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let territory = graph.add_node("TERRITORY").unwrap();
        graph
            .update_node(territory, "territory/heat", 10.0)
            .unwrap();
        let mut fixture = Fixture::new(
            HashMap::from([(
                "territory/heat".to_owned(),
                territory_field(crate::types::FieldKind::Extensive),
            )]),
            HashMap::new(),
        );
        fixture.systems.insert("territory".to_owned());
        let loaded = fixture.load(
            r#"(rule territory/p1-heat-dynamics
  :role mechanic :evidence derived :material-basis "the loaded contract alone owns write attribution"
  :fuel 32
  (bindings (binding heat :field territory/heat))
  (effects (update-node self territory/heat (add 1))))"#,
            "caller-supplied-name-must-not-matter.bsl",
        );
        let mut sink = crate::structural_verbs::CollectingSink::default();
        let mut write_log = CollectingWriteLog::new();

        run_tick_observed(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            RngSeedContext::V1 {
                session: &test_session(),
            },
            None,
            None,
            &mut write_log,
        )
        .expect("the governed mechanic must apply its write");

        assert_eq!(write_log.records.len(), 1);
        assert_eq!(loaded.contract.rule_id, "territory/p1-heat-dynamics");
        assert_eq!(write_log.records[0].rule, "territory/p1-heat-dynamics");
    }

    /// §4.2 chapter C4, quoted in the plan: "All firings of one rule
    /// observe the same pre-state … and the effects they collect are
    /// applied in that subject order." Two TERRITORY nodes, `a` (lower id)
    /// and `b` (higher id), joined by ONE `ADJACENCY` edge `a -> b` (so
    /// each is in the other's `:any` neighbourhood) — the Territory
    /// Phase-3 spillover shape (`fold sum` over `neighbors`' `heat`).
    ///
    /// Under IN-PLACE semantics (the pre-Task-12 defect, admitted in this
    /// module's own doc before this task): `a` fires first (ascending
    /// subject id), applies immediately, then `b` fires and reads `a`'s
    /// ALREADY-UPDATED heat — 100 + 110 = 210. Under §4.2 C4, `b` reads
    /// `a`'s PRE-TICK heat — 100 + 10 = 110, matching `a`'s own 10 + 100 =
    /// 110 exactly (both firings observed the SAME pre-state).
    #[test]
    fn all_firings_of_one_rule_observe_the_same_pre_state() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let a = graph.add_node("TERRITORY").unwrap();
        let b = graph.add_node("TERRITORY").unwrap();
        graph.update_node(a, "territory/heat", 10.0).unwrap();
        graph.update_node(b, "territory/heat", 100.0).unwrap();
        graph.add_edge("ADJACENCY", a, b, 1.0).unwrap();

        let fixture = Fixture::new(
            HashMap::from([(
                "territory/heat".to_owned(),
                territory_field(crate::types::FieldKind::Extensive),
            )]),
            HashMap::from([
                ("NodeType/TERRITORY".to_owned(), 100),
                ("EdgeType/ADJACENCY".to_owned(), 100),
            ]),
        );
        let loaded = fixture.load(
            r#"
(rule geography/spillover
  :role mechanic :evidence derived :material-basis "adjacent territories exchange heat; every firing of one rule observes the same pre-state (§4.2 chapter C4)"
  :fuel 256
  (bindings
    (binding heat :field territory/heat))
  (effects
    (update-node self territory/heat
      (add (fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY)
                 (field-of it territory/heat))))))
"#,
            "geography/spillover.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .expect("the tick must run");

        let a_heat = graph.node_attribute(a, "territory/heat").unwrap();
        let b_heat = graph.node_attribute(b, "territory/heat").unwrap();
        assert!(
            (a_heat - 110.0).abs() < 1e-9,
            "a's own 10 + b's PRE-TICK 100 = 110, got {a_heat}"
        );
        assert!(
            (b_heat - 110.0).abs() < 1e-9,
            "b's own 100 + a's PRE-TICK 10 = 110 — NOT 210, which is what \
             in-place semantics would compute by reading a's \
             ALREADY-UPDATED heat; got {b_heat}"
        );
    }

    /// The §6.2 family-12 accumulation vector: three TERRITORY subjects
    /// (`s1`, `s2`, `s3`, ascending id) each `(add …)` to ONE shared
    /// carrier — an `ORGANIZATION` node reached via each subject's own
    /// outgoing `ADJACENCY` edge (an `ORGANIZATION` carrier is never itself
    /// a `TERRITORY` subject, so no guard is needed to exclude it from the
    /// population). Values `1.0, 1e16, -1e16` are the SAME three MAGNITUDES
    /// `fold_reduces_in_iteration_order_and_the_order_is_observable`
    /// (`evaluator.rs`) pins, but reordered ON PURPOSE (#519 fix round): the
    /// original subject order `1e16, 1.0, -1e16` is symmetric under
    /// reversal (`(1e16 + 1.0) + -1e16 == 0.0` forward AND
    /// `(-1e16 + 1.0) + 1e16 == 0.0` reversed), so the Opus verifier proved
    /// it flips no test even when the apply loop runs in the WRONG
    /// (reversed) order — this test could not tell "applies in subject
    /// order" from "applies in the reverse of subject order". This order
    /// breaks that symmetry: forward (ascending subject id, the law)
    /// reduces `(0.0 + 1.0) + 1e16) + -1e16 == 0.0`; reversed reduces
    /// `((0.0 + -1e16) + 1e16) + 1.0 == 1.0` — a DIFFERENT bit pattern,
    /// verified in both directions with a plain f64 accumulator before this
    /// docstring was written. Applied in ascending SUBJECT order this test
    /// still proves both that `add` reads the target's CURRENT value at
    /// APPLY time (D-row Q2: a collect-time read would have every subject
    /// read the carrier's UNCHANGED initial 0.0 and the last applied write
    /// would win, losing two of the three contributions) and that the
    /// binary64 reduction order follows subject order — now for a triple
    /// that can actually distinguish "subject order" from "reversed".
    #[test]
    fn accumulation_into_a_shared_target_reduces_in_subject_order_and_keeps_every_contribution() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let carrier = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(carrier, "organization/pool", 0.0)
            .unwrap();
        for contribution in [1.0, 1.0e16, -1.0e16] {
            let subject = graph.add_node("TERRITORY").unwrap();
            graph
                .update_node(subject, "territory/contribution", contribution)
                .unwrap();
            graph.add_edge("ADJACENCY", subject, carrier, 1.0).unwrap();
        }

        let fixture = Fixture::new(
            HashMap::from([
                (
                    "territory/contribution".to_owned(),
                    territory_field(crate::types::FieldKind::Extensive),
                ),
                (
                    "organization/pool".to_owned(),
                    territory_field(crate::types::FieldKind::Extensive),
                ),
            ]),
            HashMap::from([
                ("NodeType/TERRITORY".to_owned(), 100),
                ("NodeType/ORGANIZATION".to_owned(), 100),
                ("EdgeType/ADJACENCY".to_owned(), 100),
            ]),
        );
        let loaded = fixture.load(
            r#"
(rule geography/pool-contribution
  :role mechanic :evidence derived :material-basis "each territory contributes its share to a shared regional pool; the pool must count every contribution, computed at apply time (D-row Q2)"
  :fuel 256
  (bindings
    (binding contribution :field territory/contribution))
  (effects
    (update-node
      (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/ORGANIZATION) 1)
      organization/pool
      (add contribution))))
"#,
            "geography/pool-contribution.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        let outcome = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .expect("the tick must run");
        assert_eq!(outcome.fired, 3, "all three territories fired");

        let pool = graph.node_attribute(carrier, "organization/pool").unwrap();
        // Exact IEEE-754 equality, deliberately (§4.3): the basic ops are
        // correctly rounded and reproduce bit-exactly, so an epsilon margin
        // here would obscure precisely the reduction-order property this
        // test exists to pin — the same reasoning `apply_equality`'s own
        // exact-comparison arm documents (evaluator.rs).
        #[allow(clippy::float_cmp)]
        let exact = pool == 0.0;
        assert!(
            exact,
            "(1.0 + 1e16) + -1e16 == 0.0 in ascending subject order — every \
             contribution counted, none lost; the REVERSED order reduces to \
             1.0 instead (verified separately), which is exactly what \
             distinguishes this triple from the symmetric one it replaces, \
             got {pool}"
        );
    }

    // ================================================== §2.13 (D101): the
    // read path — `bind_subject` renders the stored ordinal back to its
    // member (Task 6, Organization foundation plan).

    /// A two-member `OrgKind` registry plus the matching `TypeEnv`/`Fixture`.
    /// `Fixture::new` always seeds an EMPTY `EnumRegistry` (no fixture in
    /// this module needs one before this section) — overwritten here with
    /// the populated one, and `systems` widened to `"organization"` so the
    /// probe rule's anchor default resolves.
    fn org_kind_fixture() -> Fixture {
        let mut enums = EnumRegistry::default();
        let ty = enums
            .declare(
                "OrgKind",
                &["STATE_APPARATUS".to_owned(), "BUSINESS".to_owned()],
            )
            .unwrap();
        let mut fixture = Fixture::new(
            HashMap::from([(
                "organization/kind".to_owned(),
                crate::types::FieldDecl {
                    ty: crate::types::BslType::Enum(ty),
                    kind: crate::types::FieldKind::NotApplicable,
                },
            )]),
            HashMap::from([("NodeType/ORGANIZATION".to_owned(), 100)]),
        );
        fixture.enums = enums;
        fixture.systems = std::collections::HashSet::from(["organization".to_owned()]);
        fixture
    }

    #[test]
    fn a_when_guard_comparing_the_bound_enum_field_fires_only_for_the_matching_member() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        // Declaration-order ordinals: STATE_APPARATUS=0, BUSINESS=1.
        let state_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(state_org, "organization/kind", 0.0)
            .unwrap();
        let biz_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(biz_org, "organization/kind", 1.0)
            .unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"
(rule organization/kind-probe
  :role mechanic :evidence derived :material-basis "the state's coercive organs are a distinct material kind; content can see the difference (spec Q1)"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when (= kind OrgKind/STATE_APPARATUS))
  (effects
    (emit EventType/RUPTURE (probe 1))))
"#,
            "organization/kind-probe.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        let outcome = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .expect("the tick must run");
        assert_eq!(outcome.considered, 2, "both organizations are subjects");
        assert_eq!(
            outcome.fired, 1,
            "the guard must discriminate — only the STATE_APPARATUS org matches"
        );
        assert_eq!(sink.events.len(), 1);
    }

    #[test]
    fn ordering_on_a_bound_enum_field_refuses_naming_the_law() {
        // Proves no `Real` leaks through the read path. Deliberately
        // compares against a BARE NUMBER, not another enum-ref: comparing
        // to `OrgKind/BUSINESS` would hit `apply_ordering`'s generic
        // fallback (its message never depends on the LHS's actual runtime
        // type) whether `kind` rendered as `Real` or `Enum` — a mutation
        // check caught this the first time this test was written (see the
        // Task-6 commit body). `Int` DOES promote into the binary64 lane
        // (§3.3), so if `bind_subject` still rendered `Value::Real(0.0)`
        // this comparison would SILENTLY SUCCEED (0.0 < 5) instead of
        // refusing — the exact confusion §2.13's write/read law exists to
        // prevent ("never an ordinal a rule author could confuse for
        // magnitude"). Only the correct `Value::Enum` rendering makes this
        // load-bearing-ly refuse.
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 0.0).unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"
(rule organization/kind-ordering-probe
  :role mechanic :evidence derived :material-basis "an enum has no ordering — this rule must never load-succeed AND run-succeed"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when (< kind 5))
  (effects
    (emit EventType/RUPTURE (probe 1))))
"#,
            "organization/kind-ordering-probe.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        let err = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("Enum and Bool compare with =/!= alone"),
            "the EXISTING apply_ordering message must fire unchanged: {err}"
        );
    }

    #[test]
    fn a_corrupted_stored_ordinal_is_a_loud_integrity_error_naming_the_field_and_member_count() {
        // A hand-corrupted store (never reachable through the enum write
        // path Task 5 built) — the read boundary's own defense, never a
        // clamp and never a silently-substituted default member (III.11).
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        // OrgKind here declares only 2 members (ordinals 0, 1) — 7.0 is
        // corrupt no matter which write path could have produced it.
        graph.update_node(org, "organization/kind", 7.0).unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"
(rule organization/kind-integrity-probe
  :role mechanic :evidence derived :material-basis "a corrupted store is a loud read-boundary failure, never a clamp"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when #t)
  (effects
    (emit EventType/RUPTURE (probe 1))))
"#,
            "organization/kind-integrity-probe.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        let err = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("organization/kind"), "{err}");
        assert!(err.to_string().contains('7'), "{err}");
        assert!(
            err.to_string().contains('2'),
            "must name the [0, member_count) bound: {err}"
        );
    }

    #[test]
    fn an_enum_type_id_not_minted_by_the_executing_registry_is_a_loud_error_not_a_panic() {
        // Copilot finding, confirmed real (#528 fix round Item A). Two
        // INDEPENDENT registries: `home` mints `OrgKind` (its only entry,
        // index 0); `stranger` is empty. A `ty` resolved against `home`,
        // handed to `bind_field_value` alongside `stranger` — a driver
        // wiring bug, e.g. a `TypeEnv` built against one content set's
        // registry paired with a DIFFERENT tick's `EnumRegistry` — is
        // exactly the mismatch this test proves loud: `EnumRegistry::
        // declare` refuses an empty member list (types.rs:126), so
        // `stranger.member_count(ty) == 0` can only mean "ty was never
        // minted by this registry", never a legitimately empty `defenum`.
        // Pre-fix, `member_count == 0` let ANY non-negative stored ordinal
        // (0.0 here) satisfy `stored >= member_count` and reach
        // `stranger.name(ty)` — an out-of-bounds index PANIC
        // (`EnumRegistry::name`'s own doc names this exact caller-bug
        // shape). Post-fix this is a loud `TickError` instead.
        let mut home = EnumRegistry::default();
        let ty = home
            .declare(
                "OrgKind",
                &["STATE_APPARATUS".to_owned(), "BUSINESS".to_owned()],
            )
            .unwrap();
        let stranger = EnumRegistry::default();

        let types = crate::typecheck::TypeEnv {
            fields: HashMap::from([(
                "organization/kind".to_owned(),
                crate::types::FieldDecl {
                    ty: crate::types::BslType::Enum(ty),
                    kind: crate::types::FieldKind::NotApplicable,
                },
            )]),
            exemptions: &[],
        };

        let err = bind_field_value("organization/kind", 0.0, &types, &stranger).expect_err(
            "a ty not minted by the executing registry must refuse loudly, never panic",
        );
        assert!(err.message.contains("not minted"), "{}", err.message);
        assert!(err.message.contains("organization/kind"), "{}", err.message);
    }

    #[test]
    fn a_rule_using_field_of_over_an_enum_field_now_loads_and_runs_by_the_real_pipeline() {
        // D102 discharge (Task 1, P27 territory-port train): this used to
        // be `..._is_refused_at_load_by_the_real_pipeline`, proving the
        // (then-unconditional) D102 deferral gate was REACHABLE from
        // `rule_pipeline::load_rule`, the actual production entry point
        // (`Fixture::try_load`) — not `typecheck::
        // check_no_field_of_on_enum_field` in isolation. That gate is
        // deleted now (score-position and arithmetic each refuse through
        // their own independent mechanism, typecheck.rs's own tests cover
        // those) — this inverted test is the SAME end-to-end proof, the
        // other direction: `field-of self organization/kind` in a `when`
        // guard now LOADS clean and DISCRIMINATES correctly at
        // evaluation, exactly like the `:field`-bound read of the
        // identical field one test up
        // (`a_when_guard_comparing_the_bound_enum_field_fires_only_for_the_matching_member`)
        // — §2.5's read parity D102's own doc named as the reason to
        // discharge, not merely defer, once Territory became a real
        // consumer.
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        // Declaration-order ordinals: STATE_APPARATUS=0, BUSINESS=1.
        let state_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(state_org, "organization/kind", 0.0)
            .unwrap();
        let biz_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(biz_org, "organization/kind", 1.0)
            .unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"(rule organization/field-of-probe
  :role mechanic :evidence derived :material-basis "field-of over an enum field reads the SAME way a :field binding does (D102 discharge, §2.5 read parity)"
  :fuel 64
  (bindings
    ; Unused in the guard on purpose: the guard reads the field through
    ; field-of, not this binding — this binding exists only so
    ; `subject_type_of` can derive ORGANIZATION as the subject type,
    ; exactly as every other rule's :field binding does.
    (binding kind :field organization/kind))
  (when (= (field-of self organization/kind) OrgKind/BUSINESS))
  (effects (emit EventType/RUPTURE (probe 1))))"#,
            "organization/field-of-probe.bsl",
        );

        let mut sink = crate::structural_verbs::CollectingSink::default();
        let outcome = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .expect("the tick must run — field-of over an enum field is no longer refused");
        assert_eq!(outcome.considered, 2, "both organizations are subjects");
        assert_eq!(
            outcome.fired, 1,
            "the guard must discriminate — only the BUSINESS org matches"
        );
        assert_eq!(sink.events.len(), 1);
    }

    #[test]
    fn arithmetic_on_an_enum_field_is_refused_at_load_not_left_to_die_mid_tick() {
        // §2.13's no-arithmetic law is statically decidable (D118, #528
        // fix round Item C) — before this task the only guards were the
        // three EVAL-time ones (`structural_verbs.rs::
        // refuse_arithmetic_on_enum_field`, c268b83b), so this exact rule
        // shape loaded clean and died mid-tick, uncoded, on the first
        // admitted subject. Drives `rule_pipeline::load_rule` (via
        // `Fixture::try_load`), the actual production entry point — not
        // `typecheck::check_no_arithmetic_on_enum_field` in isolation —
        // proving the D118 load-time gate is REACHABLE, not merely
        // correct, the same discipline the D102 test above uses for its
        // sibling gap.
        let fixture = org_kind_fixture();
        let err = fixture
            .try_load(
                r#"(rule organization/kind-arithmetic-probe
  :role mechanic :evidence derived :material-basis "add/sub/scale on an :enum-type-declared field is statically decidable and refused at load (D118), never left to die mid-tick"
  :fuel 256
  (bindings
    (binding kind :field organization/kind))
  (when #t)
  (effects (update-node self organization/kind (add 1))))"#,
                "organization/kind-arithmetic-probe.bsl",
            )
            .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("D118"), "{message}");
        assert!(message.contains("organization/kind"), "{message}");
    }

    #[test]
    fn a_slash_typo_in_emits_type_operand_is_refused_at_load_not_left_to_die_mid_tick() {
        // `EventType_RUPTURE` (slash typo'd as underscore) lexes as
        // `Atom::BareUpperIdent` since the §2.13 lexer widening (D101) —
        // no load-time check enforced `emit`'s type-operand SHAPE before
        // this task (#528 fix round Item D): the §3.7 static cost pass
        // treats a `BareUpperIdent` atom identically to an `<enum-ref>`
        // (cost 0, `bound_checker::atom_cost`), and `check_enum_ref_kinds`
        // only checks the KIND of an enum-ref that is already there, so
        // this exact rule loaded clean and died mid-tick, uncoded, at
        // `structural_verbs.rs`'s own `enum_member`. `add-node`/
        // `add-edge`'s SAME typo is separately caught in the full
        // pipeline by `check_no_deferred_shape_verbs` (every rule using
        // either verb is refused there regardless, #519 fix round) —
        // `emit` carries no such second net, which is why it is the head
        // this test drives through the real production pipeline
        // (`rule_pipeline::load_rule`, via `Fixture::try_load`) rather
        // than `add-node`/`add-edge`.
        let fixture = org_kind_fixture();
        let err = fixture
            .try_load(
                r#"(rule organization/emit-typo-probe
  :role mechanic :evidence derived :material-basis "a slash typo in emit's type operand must refuse at load, not mid-tick (#528 fix round Item D)"
  :fuel 64
  (bindings)
  (when #t)
  (effects (emit EventType_RUPTURE (probe 1))))"#,
                "organization/emit-typo-probe.bsl",
            )
            .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("emit"), "{message}");
        assert!(message.contains("BareUpperIdent"), "{message}");
    }

    // ============================ Review round 1 (#576): the
    // `collect_pass` subject-content-id fallback gate. Review round 2
    // (#576 I2) sharpened it from `is_empty()`-value-distinct to
    // `Option`-type-distinct — the SAME gate as
    // `evaluator::element_content_id`'s own — see that function's doc for
    // the full cross-file invariant.

    /// (b) The `None` (never-hydrated) fixture path still works — this is
    /// already the shape every OTHER test in this module exercises (all
    /// pass `None` for `node_content_ids`, per `Fixture`'s own hand-built
    /// `MemoryGraph`), but this test names the property directly rather
    /// than leaving it implicit: a subject `NodeId` with no hydration map
    /// at all is served (the `NodeId`'s own `Debug` rendering), not
    /// refused.
    #[test]
    fn a_never_hydrated_node_content_ids_still_serves_the_debug_rendering_fallback() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 0.0).unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"(rule organization/kind-probe
  :role mechanic :evidence derived :material-basis "the state's coercive organs are a distinct material kind; content can see the difference (spec Q1)"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when (= kind OrgKind/STATE_APPARATUS))
  (effects
    (emit EventType/RUPTURE (probe 1))))"#,
            "organization/kind-probe.bsl",
        );
        let mut sink = crate::structural_verbs::CollectingSink::default();
        let outcome = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            None,
            &test_session(),
            None,
        )
        .expect("a never-hydrated (None) node_content_ids must still serve the Debug-rendering fallback");
        assert_eq!(outcome.fired, 1);
    }

    /// (a) The error fires: a subject `NodeId` missing from a `Some`-hydrated
    /// `node_content_ids` map is a hard `TickError`, never a silent
    /// fallback — the review's own recommended fix. The map holds an
    /// entry, just not for `org` (the actual subject) — the
    /// hydration-bug shape a lifted `DEFERRED_SHAPE_VERBS` gate would one
    /// day produce for real.
    #[test]
    fn a_subject_missing_from_a_hydrated_node_content_ids_map_is_a_hard_tick_error() {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 0.0).unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"(rule organization/kind-probe
  :role mechanic :evidence derived :material-basis "the state's coercive organs are a distinct material kind; content can see the difference (spec Q1)"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when (= kind OrgKind/STATE_APPARATUS))
  (effects
    (emit EventType/RUPTURE (probe 1))))"#,
            "organization/kind-probe.bsl",
        );
        let mut sink = crate::structural_verbs::CollectingSink::default();
        // Hydrated (`Some`), but keyed to a DIFFERENT NodeId than `org` —
        // the exact shape a hydration bug (a node minted without its
        // content id recorded) would produce.
        let non_empty_but_missing_subject = HashMap::from([(
            babylon_graph::substrate::NodeId(999),
            "someone-else".to_owned(),
        )]);
        let err = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            Some(&non_empty_but_missing_subject),
            &test_session(),
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("hydration bug"), "{err}");
    }

    /// (c) **The distinguishing case review round 2 (#576 I2) exists to
    /// prove.** A `Some(map)` where `map` is EMPTY — modeling a caller that
    /// pre-populated its own graph (a save load, a Bevy-side world build)
    /// and handed it to `run_tick` alongside a declarations-only scenario
    /// (`node_count == 0`, per `scenario.rs::load_scenario`'s own body
    /// loop). Under the PRE-I2 `is_empty()` gate this was
    /// indistinguishable from "never hydrated" and SILENTLY fed the raw
    /// `NodeId`'s `Debug` rendering into `stable_key` — exactly the
    /// insertion-order-dependent butterfly ADR176 r20 forbids, with no
    /// error and no failing test. Under the `Option`-typed gate,
    /// `Some(empty_map)` unambiguously means "hydrated", so the SAME
    /// subject-missing error fires as it would against a non-empty map —
    /// loud, not silent.
    #[test]
    fn a_subject_against_a_hydrated_but_empty_node_content_ids_map_is_a_hard_tick_error_not_a_silent_handle(
    ) {
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 0.0).unwrap();

        let fixture = org_kind_fixture();
        let loaded = fixture.load(
            r#"(rule organization/kind-probe
  :role mechanic :evidence derived :material-basis "the state's coercive organs are a distinct material kind; content can see the difference (spec Q1)"
  :fuel 64
  (bindings
    (binding kind :field organization/kind))
  (when (= kind OrgKind/STATE_APPARATUS))
  (effects
    (emit EventType/RUPTURE (probe 1))))"#,
            "organization/kind-probe.bsl",
        );
        let mut sink = crate::structural_verbs::CollectingSink::default();
        // A REAL hydration (`Some`) that happens to hold zero entries — the
        // declarations-only-scenario shape, never the never-hydrated one.
        let hydrated_but_empty: HashMap<babylon_graph::substrate::NodeId, String> = HashMap::new();
        let err = run_tick(
            &loaded,
            &fixture.types,
            &fixture.enums,
            &crate::intrinsic_host::EmptyIntrinsicHost,
            &mut graph,
            &mut sink,
            &fixture.intrinsics,
            &DefinesEnv::new(),
            1,
            Some(&hydrated_but_empty),
            &test_session(),
            None,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("hydration bug"),
            "a Some(empty map) miss must hard-error exactly like a Some(non-empty map) miss, \
             never silently fall back to the NodeId's own Debug rendering: {err}"
        );
    }
}
