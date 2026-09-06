//! `babylon-tick`'s `run_once` seam — the Phase 2 Slice 1 flow (scenario load
//! -> rule load -> one tick -> state hash) as a library function, so both
//! the CLI driver (`main.rs`) and `babylon-client`'s engine link (B0) call
//! exactly one implementation. See `main.rs` for the CLI-facing docs; this
//! module is the seam itself.

// `PrepareError` (§2.3, issue #652 Task 3) wraps `ScenarioError`, which is
// already past clippy's 128-byte `result_large_err` threshold on its own
// (`babylon-bsl/src/scenario.rs:69-77`'s own citation) — wrapping it behind
// one more enum layer cannot shrink that. The lint fires on every function
// in THIS module that returns `Result<_, PrepareError>`
// (`seed_implicit_edge_strength_fields`, `build_shared_load_inputs`,
// `prepare_rules`) — the same "one error type, many signatures" shape
// `scenario.rs`'s own module-scope allow exists for, so this follows that
// precedent at module scope rather than three separate function-level
// allows. The load path is cold (a content set either loads once or the
// whole run fails), so paying the extra stack bytes on every one of these
// rarely-taken error branches is the right trade against boxing a field
// §2.3 specifies unboxed, or `Box`-wrapping every wrapped error type for a
// branch that is rarely taken.
#![allow(clippy::result_large_err)]

use babylon_bsl::causal_contract::{reduce_audit_receipts, AuditReceipt};
use babylon_bsl::declarations::{parse_intrinsic_decls, DeclError, FieldRegistry};
use babylon_bsl::error_identity::ErrorIdentity;
use babylon_bsl::evaluator::Value;
use babylon_bsl::fuel::{CardinalityCeilings, IntrinsicCosts};
use babylon_bsl::intrinsic_host::KernelIntrinsicHost;
use babylon_bsl::probability::{KernelInstanceIdentityV1, ProbabilityError};
use babylon_bsl::reader::{FormPath, SExpr};
use babylon_bsl::rule_pipeline::{
    check_unique_rule_ids, load_rule_form, split_content, LoadContext, LoadError, LoadedRule,
};
use babylon_bsl::same_tick_order::{diagnose_ranked, ENFORCE_RANK_AWARE_AGGREGATE_ORDERING};
use babylon_bsl::scenario::{
    load_scenario, load_scenario_with_prelude, LoadedScenario, ScenarioError,
};
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_bsl::tick::run_tick_observed;
use babylon_bsl::typecheck::TypeEnv;
use babylon_bsl::types::{EnumRegistry, FieldDecl};
use babylon_bsl::write_log::CollectingWriteLog;
use babylon_bsl::BindingVocabulary;
use babylon_graph::allocator_state::AllocatorState;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::{GraphError, GraphSubstrate, HyperedgeId, NodeId};
use babylon_graph::working_copy::DetachedCopy;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1, RngSeedContext};
use babylon_kernel::SessionId;
use std::collections::{HashMap, HashSet};

pub mod choice_receipt;
pub mod committed_event;
pub mod h3_runtime;
pub mod kernel_slot;
pub mod material_replay;
pub mod material_state;
pub mod material_world;
mod phase_order;
pub mod replay_identity;
pub mod replay_session;
pub mod session;
mod world_hash;
pub use session::TickSession;

use replay_session::{IdentifiedTickReportV2, ReplayTickError};

/// The result of running one or more rules over one scenario for one tick:
/// graph and nominal-world hashes around the commit, plus guard and firing counts.
#[derive(Debug)]
pub struct TickReport {
    /// Canonical graph-state hash before adjudication.
    pub before: [u8; 32],
    /// Canonical graph-state hash after successful adjudication.
    pub after: [u8; 32],
    /// Nominal graph-plus-current-auxiliary world hash before adjudication.
    pub world_before: [u8; 32],
    /// Nominal graph-plus-current-auxiliary world hash after commit.
    pub world_after: [u8; 32],
    /// Total subjects whose guards were evaluated across every rule.
    pub considered: usize,
    /// The TOTAL fired-subject count across every rule this tick ran —
    /// unchanged in meaning and type for a single-rule content set (today
    /// every existing caller: `run_once`, the CLI, B0's engine-link probe,
    /// every `*_conformance.rs` test). For a multi-rule tick this is the
    /// SUM across rules — kept a plain `usize` rather than widened to
    /// `Vec<usize>` specifically so `report.fired == N` assertions across
    /// this crate's `tests/*_conformance.rs` and `tests/floor_intrinsic_e2e.rs`
    /// keep compiling and keep passing unmodified.
    pub fired: usize,
    /// Per-rule guard-evaluation detail in governed causal order.
    pub per_rule_considered: Vec<(String, usize)>,
    /// Per-rule detail in governed causal order — `(rule_id, fired)`.
    /// Rules resolve through the 34-slot phase registry; rules sharing one
    /// position use D16's ascending rule-ID byte order. Declaration and file
    /// order are never observable. Length 1 for every existing single-rule
    /// content set (`fired == per_rule_fired[0].1` always holds); length N
    /// for an N-rule content set.
    pub per_rule_fired: Vec<(String, usize)>,
    /// Identity-free events and writes observed from successful rule effects,
    /// in executable rule order. Failed ticks publish no receipts.
    pub audit_receipts: Vec<AuditReceipt>,
    /// Exact finite-choice evidence in tick-wide encounter order. A branch
    /// that makes no material change still contributes one receipt.
    pub choice_receipts: Vec<choice_receipt::ChoiceReceiptV1>,
    /// Successful event observations with engine-owned emitting-rule and
    /// optional adjacent finite-projection provenance.
    pub committed_events: Vec<committed_event::CommittedEventV2>,
}

pub(crate) type EventRecord = (String, Vec<(String, Value)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HashBoundary {
    Pre,
    Post,
}

/// Fallible preparation plus infallible commit for one already-buffered tick
/// event batch. Implementations must leave their logical event sequence
/// unchanged when preparation refuses.
pub(crate) trait PreparedEventBatchSink {
    fn try_prepare(&mut self, additional: usize) -> Result<(), String>;
    fn commit_prepared(&mut self, events: Vec<EventRecord>);
}

impl PreparedEventBatchSink for CollectingSink {
    fn try_prepare(&mut self, additional: usize) -> Result<(), String> {
        self.events
            .try_reserve(additional)
            .map_err(|error| format!("event commit refused: {error}"))
    }

    fn commit_prepared(&mut self, events: Vec<EventRecord>) {
        debug_assert!(self.events.capacity() - self.events.len() >= events.len());
        self.events.extend(events);
    }
}

/// Render a 32-byte hash as lowercase hex — the same format the CLI driver
/// prints and the engine-link probe logs.
///
/// **#652 Task 6 pedantic repair, recorded:** `babylon-ls` (Task 6) is the
/// first PEDANTIC-gated crate (`rust/.mise.toml`'s `rust:check`) to depend
/// on `babylon-tick` at all — clippy lints a workspace path dependency
/// under its DEPENDENT's flags, so adding that edge exposed 4 pre-existing
/// pedantic findings this module never had to satisfy before. `#[must_use]`
/// plus a `fold`+`write!` build (clippy's own `format_collect` finding)
/// replace the original one-liner; the rendered bytes are unchanged.
#[must_use]
pub fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

/// Load `scenario_src`, load `rule_src` through every gate, run one tick,
/// and return the pre/post state hashes plus the fired-subject count.
///
/// This is the Slice 1 contract: the CLI driver (`main.rs`) and
/// `babylon-client`'s engine link (Program 28 B0) both call exactly this
/// function, so "the client links the engine in-process" means literally
/// sharing this code path, not a lookalike reimplementation.
///
/// `rule_src` may carry zero or more `(intrinsic …)` top-forms (§2.2's
/// `<intrinsic-decl>`) alongside its one `(rule …)` form, in EITHER order —
/// `(intrinsic …)` is ordinary content, not a side channel a caller must
/// route through a separate parameter (§2.2: "file boundaries and file
/// names carry no semantics"). A rule calling an undeclared intrinsic
/// refuses loudly (`E-LOAD-021`); a declared name outside
/// `declarations::DECLARABLE_INTRINSICS` refuses the WHOLE load
/// (`E-LOAD-020`/`E-LOAD-024`/`E-LOAD-001`), never a partial admission.
///
/// # Errors
///
/// A description of the first failing stage — an intrinsic declaration, a
/// scenario load, a rule load, a state hash, or the tick itself (the same
/// class `run_once_into` documents, since this delegates to it).
pub fn run_once(scenario_src: &str, rule_src: &str) -> Result<TickReport, String> {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(scenario_src, rule_src, &mut graph, &mut sink)
}

/// `run_once`, with the scenario load routed through a **declaration
/// prelude** first (Train B item 4, issue #591, D157) — the
/// scenario-declaration sharing seam. `prelude_src` MAY declare `defenum` /
/// `defvocabulary` / `defconst` / `deffield` forms the scenario's own
/// `deffield`s and node/edge seeds resolve against, exactly as if the
/// scenario had declared them itself; the scenario MAY re-declare a
/// `defenum` the prelude declared, verbatim (`EnumRegistry::declare`'s
/// identical-recognition arm, this train — `defenum`-only: `deffield`,
/// `defconst`, and `defvocabulary` still refuse ANY re-declaration,
/// identical or not), and a disagreeing `defenum` re-declaration still
/// refuses loudly.
///
/// Argument order is `(scenario, prelude, rule)` — `scenario_src` leads,
/// matching `run_once`'s own lead argument; `prelude_src` slots second,
/// between the two sources it sits between in the load pipeline.
///
/// # Errors
///
/// A description of the first failing stage — the prelude, an intrinsic
/// declaration, a scenario load, a rule load, a state hash, or the tick
/// itself.
pub fn run_once_with_prelude(
    scenario_src: &str,
    prelude_src: &str,
    rule_src: &str,
) -> Result<TickReport, String> {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into_with_prelude(scenario_src, prelude_src, rule_src, &mut graph, &mut sink)
}

/// Everything `run_once_into` does before running a tick: parse the
/// intrinsic declarations, load the scenario into `graph`, and load every
/// `(rule …)` form `split_content` returns against the vocabulary/types/
/// ceilings that scenario declared — compiled into the governed phase order
/// before this returns, so every later stage (`TickSession::advance`,
/// `run_once_into`) just iterates the already-correct order. D16's ascending
/// rule-ID byte order breaks ties at one resolved position. Shared by
/// `run_once_into` (which still runs exactly tick 1) and, from `session.rs`
/// on, `TickSession::new` (Program 28 B2,
/// `docs/superpowers/plans/2026-08-11-b2-tick-loop-plan.md` Phase A Task 4).
///
/// `Debug` (T2, issue #559): needed so `Result<PreparedRules, PrepareError>`
/// (`String` before #652 Task 3's `PrepareError`) can be formatted with
/// `{:?}` in a test assertion message (every field already derives `Debug`,
/// so this is additive only).
#[derive(Debug)]
pub(crate) struct PreparedRules {
    pub rules: Vec<(String, LoadedRule)>,
    /// Parsed rule forms retained so replay preparation can independently
    /// recompute the canonical rules hash over what this engine loaded.
    pub rule_forms: Vec<SExpr>,
    pub types: TypeEnv,
    pub intrinsics: IntrinsicCosts,
    pub consts: HashMap<String, Value>,
    /// **§2.13 addendum (D101).** The scenario's `defenum` registry —
    /// `run_tick`'s write path (`update-node` on an enum-typed field) and
    /// read path (`bind_subject` rendering a stored ordinal back to its
    /// member) both resolve against this.
    pub enums: EnumRegistry,
    /// **Content-stable node identity (plan §3.4, Task 3).** `LoadedScenario
    /// ::node_content_ids`, threaded through unchanged — `TickSession` holds
    /// it for the tick's lifetime by holding this whole struct. **First
    /// production consumer landed (Task 4, #576 intrinsic-host train):**
    /// `run_prepared_tick` passes `&prepared.node_content_ids` into
    /// `run_tick`, which builds one [`babylon_bsl::intrinsic_host::
    /// DrawContext`] per subject from it (plan §3.3's `subject` key
    /// component) — the same `require_graph` precedent's lifecycle
    /// (`babylon-bsl/src/evaluator.rs`) this field's Task-3 doc named:
    /// dropped its `#[allow(dead_code)]` the moment a real caller landed.
    /// Still reaches no `babylon-graph` write path and carries no
    /// canonical-state weight — `state_hash` is computed over the substrate
    /// alone.
    pub node_content_ids: HashMap<NodeId, String>,
    /// Validated scenario qname retained for stable replay identity.
    #[allow(
        dead_code,
        reason = "PER-60 Task 7 retains this for Task 9's prepared-environment composer"
    )]
    pub scenario_scope: String,
    /// Authored hyperedge identities retained from scenario hydration.
    #[allow(
        dead_code,
        reason = "PER-60 Task 7 retains this for Task 10's sealed stable resolver"
    )]
    pub hyperedge_content_ids: HashMap<HyperedgeId, String>,
    /// The scenario's closed vocabulary, when it declared one —
    /// `run_tick`'s D29 owner-kind filter (`subject_type_of`, Community
    /// port train Task 6) reads it so a hyperedge/edge-owned `:field`
    /// binding can never masquerade as a subject type. `None` for a
    /// scenario declaring no `defvocabulary` at all, exactly the
    /// load-side `LoadContext::vocabulary_registry` convention.
    pub vocabulary: Option<babylon_bsl::vocabulary::ClosedVocabulary>,
}

/// Why `prepare_rules` (or [`diagnose_content_set`]) refused a content set —
/// the structured seam #652 Task 3 gives the load path, so a caller (a test,
/// or wave 2's `bsl-ls`) can read WHAT stage failed and WHAT code it carries
/// without scanning a formatted string (§2.3, issue #652's `ErrorIdentity`
/// discipline — this crate reuses that enum rather than minting a parallel
/// one; see `error_identity.rs`'s own module doc for the one-brain
/// rationale).
///
/// `ScenarioError`/`LoadError` already exceed clippy's 128-byte
/// `result_large_err` threshold on their own (`scenario.rs:69-77`'s own
/// citation); wrapping either behind one more enum layer cannot shrink that.
/// This module carries the resulting `#![allow(clippy::result_large_err)]`
/// at module scope — see this file's own top-of-file citation.
#[derive(Debug, Clone, PartialEq)]
pub enum PrepareError {
    /// The scenario failed to hydrate (`load_scenario`/
    /// `load_scenario_with_prelude`).
    Scenario(ScenarioError),
    /// One `(rule …)` form was rejected. `rule_id` is `None` for a
    /// composition-level [`split_content`] failure (a malformed content
    /// source or duplicate rule id) raised before any individual rule's own
    /// id is in scope; `Some` for a specific rule's own load rejection.
    Rule {
        /// The rejected rule's id, when the failure is attributable to one.
        rule_id: Option<String>,
        /// The rejecting stage's own error.
        error: LoadError,
    },
    /// An `(intrinsic …)` top-form's own declaration was rejected
    /// (`parse_intrinsic_decls`).
    Intrinsic(DeclError),
    /// The finite-kernel content-set law failed after every individual rule
    /// loaded and the governed schedule was resolved.
    Probability {
        /// The offending scheduled rule, when the exact probability error
        /// identifies one rule rather than the content set as a whole.
        rule_id: Option<String>,
        /// The typed probability layer's exact refusal.
        error: ProbabilityError,
    },
    /// A scheduled finite kernel did not exactly match the permanent
    /// append-only sample/slot ledger.
    KernelSlot(kernel_slot::KernelSlotLedgerErrorV1),
    /// A composition-level refusal raised by `prepare_rules` itself — no
    /// earlier crate's error type to wrap. `code`/`identity` are `Option`
    /// because some composition rules are genuinely uncoded (the
    /// [`LoadError::Content`] precedent) — but the D32 implicit-`strength`
    /// duplicate (`seed_implicit_edge_strength_fields`) is NOT one of
    /// them: its code and identity are threaded through as DATA, even though
    /// (fallback taken, see that function's own doc) the message text stays
    /// the hand-built string a generic `DeclError::Duplicate` cannot
    /// reproduce byte-for-byte.
    Composition {
        /// The spec's error code, when this composition rule names one.
        code: Option<&'static str>,
        /// WHAT the refusal is about, when this composition rule can name a
        /// located identity.
        identity: Option<ErrorIdentity>,
        /// Human-readable detail, reproduced verbatim from what this crate
        /// raised before Task 3 structured it.
        message: String,
    },
}

impl PrepareError {
    /// The spec's error code, where the failing stage names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::Scenario(e) => e.code,
            Self::Rule { error, .. } => error.spec_code(),
            Self::Intrinsic(e) => e.spec_code(),
            Self::Probability { .. } => None,
            Self::KernelSlot(_) => None,
            Self::Composition { code, .. } => *code,
        }
    }
}

impl std::fmt::Display for PrepareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scenario(e) => write!(f, "{e}"),
            // Byte-identical to `prepare_rules`'s own pre-Task-3 wrapping
            // (`format!("rule {id} rejected: {e}")`) for a specific rule's
            // rejection; a composition-level `split_content` failure (no
            // rule id in scope yet) carries the wrapped error's own text
            // unprefixed, matching that pre-Task-3 call site's bare
            // `.map_err(|e| e.to_string())`.
            Self::Rule {
                rule_id: Some(id),
                error,
            } => write!(f, "rule {id} rejected: {error}"),
            Self::Rule {
                rule_id: None,
                error,
            } => write!(f, "{error}"),
            Self::Intrinsic(e) => write!(f, "{e}"),
            Self::Probability {
                rule_id: Some(id),
                error,
            } => write!(f, "rule {id} rejected: {error}"),
            Self::Probability {
                rule_id: None,
                error,
            } => write!(f, "{error}"),
            Self::KernelSlot(error) => write!(f, "{error}"),
            Self::Composition { message, .. } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PrepareError {}

/// The rule-pack namespaces this driver registers before loading any rule
/// (`LoadContext::systems`) — extracted (Task 3, #652) so `prepare_rules`
/// and [`diagnose_content_set`] build the IDENTICAL set from one place
/// rather than two copies drifting apart. PER-17 replaces the former partial
/// inline set with the canonical 34-slot registry and its accepted names.
fn registered_systems() -> HashSet<String> {
    phase_order::registered_systems()
}

fn prepare_error_from_schedule(error: phase_order::ScheduleError) -> PrepareError {
    let message = error.to_string();
    match error {
        phase_order::ScheduleError::Anchor { rule_id, source } => PrepareError::Rule {
            rule_id: Some(rule_id),
            error: LoadError::Anchor(source),
        },
        phase_order::ScheduleError::MaterialBaseInterleave { rule_id, .. } => {
            PrepareError::Composition {
                code: Some("E-LOAD-003"),
                identity: Some(ErrorIdentity::RuleId(rule_id)),
                message,
            }
        }
        phase_order::ScheduleError::Registry { .. }
        | phase_order::ScheduleError::Allocation { .. }
        | phase_order::ScheduleError::CapacityOverflow { .. } => PrepareError::Composition {
            code: None,
            identity: None,
            message,
        },
        phase_order::ScheduleError::Plan { rule_id, .. } => PrepareError::Composition {
            code: None,
            identity: rule_id.map(ErrorIdentity::RuleId),
            message,
        },
    }
}

fn enforce_ranked_composition(
    plan: &phase_order::RuleOrderPlan,
    rule_forms: &[(String, SExpr)],
) -> Result<(), PrepareError> {
    let ranked = plan
        .ranked_rules(rule_forms)
        .map_err(prepare_error_from_schedule)?;
    if ENFORCE_RANK_AWARE_AGGREGATE_ORDERING {
        diagnose_ranked(&ranked)
            .into_enforced_result()
            .map_err(|error| PrepareError::Rule {
                rule_id: None,
                error: LoadError::SameTickOrder(error),
            })?;
    }
    Ok(())
}

fn hydrate_scenario<G: GraphSubstrate>(
    scenario_src: &str,
    prelude_src: Option<&str>,
    graph: &mut G,
) -> Result<LoadedScenario, PrepareError> {
    match prelude_src {
        Some(prelude) => {
            load_scenario_with_prelude(prelude, scenario_src, graph).map_err(PrepareError::Scenario)
        }
        None => load_scenario(scenario_src, graph).map_err(PrepareError::Scenario),
    }
}

/// The D32 implicit-`<edge-type>/strength` collision check (D32,
/// `bsl-language.rst` §2.9): every `EdgeType` carries one implicit
/// `<edge-type>/strength` field, needing no `deffield`.
/// `FieldRegistry::with_implicit_edge_strength` already builds this seed
/// set, fully tested (`declarations.rs`, `r9_chapters.rs`'s own `type_env()`
/// fixture) but had no production caller until T2 (issue #559) — this is
/// that caller. Seeded from the scenario's declared `defvocabulary
/// EdgeType` members (`scenario.vocabulary`; `None` for a scenario
/// declaring no `defvocabulary` at all — the loop below is then a no-op,
/// matching every pre-T2 scenario's behavior exactly). An explicit
/// `deffield` re-declaring an implicit field is D32's own named violation
/// (`E-LOAD-001`, `FieldRegistry::declare`'s own duplicate guard) — checked
/// HERE rather than through that guard, because `scenario.rs`'s simpler
/// `load_deffield` builds `scenario.fields` with no notion of "implicit"
/// to check against; this is that check's only home until Phase 2's
/// content-pack field registries replace `scenario.fields` wholesale
/// (`declarations.rs`'s own module doc).
///
/// **Preferred-vs-fallback record (#652 Task 3, plan §3.2).** The plan's
/// preferred fix — construct a real `DeclError::Duplicate{name: qname, what:
/// "field"}` and delete this hand-rolled string — was evaluated, not taken.
/// `DeclError::Duplicate`'s `Display` is a FIXED template, `"E-LOAD-001:
/// duplicate {what} declaration: {name}"` (`declarations.rs`), qname LAST.
/// The message this call site must keep byte-identical (pinned by
/// `tests`' `a_two_collision_e_load_001_refusal_always_names_the_byte_least_field`,
/// this crate) puts the qname FIRST inside an entirely different sentence —
/// no choice of `what` can produce it; reaching byte-identity would need a
/// bespoke `Display` override on `DeclError::Duplicate` itself (or a new
/// variant) built for this one caller, which is not a clean, minimal
/// migration confined to this crate's own two files. **Fallback taken**
/// (plan-sanctioned): the check stays exactly where it is and how it reads,
/// but now returns a structured [`PrepareError::Composition`] carrying
/// `code`/`identity` as DATA rather than only inside a formatted string —
/// closing the "revision 1 silently lost a real code" gap without touching
/// `DeclError`'s general-purpose shape.
///
/// # Errors
///
/// [`PrepareError::Composition`] with `code: Some("E-LOAD-001")` and
/// `identity: Some(ErrorIdentity::Field(qname))` for the first (byte-least
/// qname, ascending) implicit-field collision.
fn seed_implicit_edge_strength_fields(
    scenario: &LoadedScenario,
) -> Result<HashMap<String, FieldDecl>, PrepareError> {
    let mut fields = scenario.fields.clone();
    if let Some(vocabulary) = scenario.vocabulary.as_ref() {
        let mut implicit: Vec<_> = FieldRegistry::with_implicit_edge_strength(vocabulary)
            .type_env_fields()
            .into_iter()
            .collect();
        // Byte order, the same convention D16 uses for same-position rule
        // ties: the Err/Ok verdict never depended on
        // `type_env_fields()`'s HashMap iteration order, but the refusal
        // TEXT did — with two or more colliding qnames it
        // nondeterministically named a different field per process.
        // Sorting makes the message always name the byte-least colliding
        // qname (`declarations.rs`'s own reporting sorts the same way
        // before its first-failure checks).
        implicit.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
        for (qname, decl) in implicit {
            if fields.contains_key(&qname) {
                let message = format!(
                    "E-LOAD-001: {qname} is the implicit <edge-type>/strength field (D32) — \
                     re-declaring it with an explicit deffield is a duplicate declaration, never \
                     a silent override (bsl-language.rst §2.9)"
                );
                return Err(PrepareError::Composition {
                    code: Some("E-LOAD-001"),
                    identity: Some(ErrorIdentity::Field(qname)),
                    message,
                });
            }
            fields.insert(qname, decl);
        }
    }
    Ok(fields)
}

/// Everything a rule form's own [`LoadContext`] needs, built from a
/// successfully-hydrated scenario — shared by `prepare_rules` (which owns
/// the values into [`PreparedRules`]) and [`diagnose_content_set`] (which
/// needs the SAME values just to build a throwaway `LoadContext` for one
/// diagnostic pass). A named struct rather than a tuple so a caller
/// destructures by field name, not by position.
struct SharedLoadInputs {
    /// Declared field types and kinds (§3.4), D32-implicit-seeded.
    types: TypeEnv,
    /// Declared fields / defines keys / registered metrics (§3.5).
    vocabulary: BindingVocabulary,
    /// Declared cardinality ceilings (§3.7).
    ceilings: CardinalityCeilings,
    /// Registered system names, for the anchor default (§2.3).
    systems: HashSet<String>,
}

/// Build [`SharedLoadInputs`] from a hydrated scenario — the SAME
/// construction `prepare_rules` ran inline before Task 3 (#652), relocated
/// so [`diagnose_content_set`] does not duplicate ~80 lines of the
/// `registered_systems`/ceilings/vocabulary literals verbatim.
///
/// # Errors
///
/// [`PrepareError::Composition`] from [`seed_implicit_edge_strength_fields`]
/// on a D32 implicit-field collision.
fn build_shared_load_inputs(scenario: &LoadedScenario) -> Result<SharedLoadInputs, PrepareError> {
    let fields = seed_implicit_edge_strength_fields(scenario)?;
    let types = TypeEnv {
        fields,
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        // The scenario's `(defconst …)` rows ARE the defines environment for
        // slice 1, exactly as its `deffield` rows are the field registry.
        // Taking the vocabulary and the values from ONE declaration is what
        // keeps `E-LOAD-010` (unknown coefficient, at load) and the tick's
        // lookup from ever disagreeing.
        consts: scenario.consts.keys().cloned().collect(),
        probability_consts: scenario.probability_consts.clone(),
        metrics: HashSet::new(),
    };
    // One ceiling per node type the scenario ACTUALLY minted, keyed as the
    // bound checker expects (`NodeType/MEMBER`). Hard-coding a single type
    // would make this driver silently specific to `SOCIAL_CLASS`: any rule
    // querying another type would fail load with `MissingCeiling`, which is
    // a confusing way to say "this driver only ever supported one type".
    //
    // A type the scenario declared zero of still gets no ceiling — and that
    // is correct: a rule querying a population that does not exist should
    // fail loudly at load rather than quietly iterate nothing.
    //
    // Same for `EdgeType/MEMBER` (query-evaluation plan, Task 15, P27 Phase
    // 2 PR 5): `bound_checker::neighbors_ceiling` bounds a `(neighbors …)`
    // fold against the LESSER of the queried edge type's ceiling and the
    // annotated result NodeType's, so a rule using `neighbors` needs an
    // edge-type entry too, or the load fails `MissingCeiling` on the edge
    // axis specifically. `scenario.node_types` and `scenario.edge_types`
    // key disjoint namespaces (`NodeType/…` vs `EdgeType/…`), so merging
    // them into one flat map — which is what `CardinalityCeilings` already
    // is — cannot collide.
    //
    // Same again for `HyperedgeType/MEMBER` (Community port train, Task 4
    // — plan `docs/superpowers/plans/2026-08-18-community-port.md`):
    // `hyperedges`/`members-of`/`hyperedges-of` folds refuse load without
    // the type's ceiling (E-LOAD-045) or its max-members axis (E-LOAD-042).
    // Both are fed from the scenario's OWN census (`hyperedge_types`/
    // `max_members_seen`, Task 1) — derived from the population the
    // scenario actually built, never an invented constant (the D200
    // record).
    let ceilings = CardinalityCeilings::new(
        scenario
            .node_types
            .iter()
            .map(|(member, count)| (format!("NodeType/{member}"), *count))
            .chain(
                scenario
                    .edge_types
                    .iter()
                    .map(|(member, count)| (format!("EdgeType/{member}"), *count)),
            )
            .chain(
                scenario
                    .hyperedge_types
                    .iter()
                    .map(|(member, count)| (format!("HyperedgeType/{member}"), *count)),
            )
            .collect(),
        // `max_members_seen` is member-keyed at the census
        // (`scenario.rs`'s Task-1 maps mirror `node_types`' shape); the
        // checker queries `HyperedgeType/<member>` (`enum_ref_key`) — the
        // prefix lands at THIS seam, the same place the count map gets
        // its own.
        scenario
            .max_members_seen
            .iter()
            .map(|(member, longest)| (format!("HyperedgeType/{member}"), *longest))
            .collect(),
    );
    Ok(SharedLoadInputs {
        types,
        vocabulary,
        ceilings,
        systems: registered_systems(),
    })
}

fn probability_prepare_error(rules: &[LoadedRule], error: ProbabilityError) -> PrepareError {
    let rule_id = match &error {
        ProbabilityError::DuplicateSample { rule_id, .. }
        | ProbabilityError::ProjectionNotAdjacent { rule_id, .. } => Some(rule_id.clone()),
        ProbabilityError::SubjectCarrierMismatch(details) => {
            Some(details.projection_rule_id.clone())
        }
        ProbabilityError::InvalidForm { .. }
        | ProbabilityError::ZeroTotalMass
        | ProbabilityError::TotalMassOverflow
        | ProbabilityError::PositiveMassHasZeroTickets { .. } => rules
            .iter()
            .find(|rule| rule.kernel.is_some())
            .map(|rule| rule.contract.rule_id.clone()),
        ProbabilityError::MassOverflow
        | ProbabilityError::MassUnderflow
        | ProbabilityError::NegativeMassInput
        | ProbabilityError::NonFiniteMassInput
        | ProbabilityError::TicketNotCovered { .. } => None,
    };
    PrepareError::Probability { rule_id, error }
}

/// Load `scenario_src` (optionally through `prelude_src`) and every rule
/// source in `rule_srcs` through the SAME staged sequence `prepare_rules`
/// runs, but COLLECTING every independent failure instead of stopping at
/// the first — the `bsl-ls` diagnostics seam (#652, Task 3) needs a full
/// report of a content set's problems, not just its first one.
///
/// Continuation discipline, staged:
/// - each element of `rule_srcs` is [`split_content`] independently — one
///   malformed source cannot hide the forms a SIBLING source parses
///   cleanly, so a failure here is recorded and the NEXT source still gets
///   its own chance;
/// - aggregate rule-id uniqueness is checked after splitting, because file
///   boundaries carry no semantics; a duplicate split across sources is one
///   structured `E-LOAD-001` diagnostic rather than a false-clean result;
/// - intrinsic-declaration parsing runs once, over every collected
///   `(intrinsic …)` form — a failure here BLOCKS rule loading, because
///   every rule's static fuel-bound check needs `IntrinsicCosts` to exist
///   at all;
/// - scenario validation hydrates a disposable graph and BLOCKS rule loading
///   on failure — caller-owned state is never in scope here;
/// - the D32 implicit-`<edge-type>/strength` collision check (the same one
///   `prepare_rules` runs, `seed_implicit_edge_strength_fields`) BLOCKS
///   rule loading on failure — the seeded field registry would be
///   incomplete past the first collision, so no `LoadContext` can be built;
/// - every `(rule …)` form collected across every source then loads
///   INDEPENDENTLY — one rule's rejection never hides a sibling's, matching
///   `prepare_rules`'s own "no partial admission, but every unit gets its
///   own chance" discipline one radius wider;
/// - an aggregate duplicate rule ID suppresses phase placement because one
///   identity cannot own two potentially different anchors;
/// - phase placement compiles over the independently admitted rule forms after
///   per-rule loading when their identities are unique, so a broken sibling
///   cannot hide an unrelated causal composition failure.
///
/// Ordering within the returned `Vec` is: split-stage failures in source
/// order, an aggregate duplicate-id failure when present, one blocking
/// intrinsic/scenario/composition failure when present, then per-rule failures
/// in encounter order, then a phase-composition failure over admitted
/// siblings. Executable phase ordering applies to admitted rules, not
/// diagnostic display order.
///
/// An empty return means the content set loads clean end to end — the SAME
/// success condition `prepare_rules` reports as `Ok`.
#[must_use]
pub fn diagnose_content_set(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_srcs: &[&str],
) -> Vec<PrepareError> {
    let rule_sources = rule_srcs
        .iter()
        .map(|source| ContentRuleSourceV1 {
            // Preserve the historical loader identity used by this unnamed
            // compatibility surface. Callers that need exact source
            // ownership use `diagnose_content_set_sources` below.
            source_id: "rule",
            source,
        })
        .collect::<Vec<_>>();
    diagnose_content_set_sources(scenario_src, prelude_src, &rule_sources)
        .into_iter()
        .map(|sourced| sourced.error)
        .collect()
}

/// One diagnostic refusal plus the exact rule source that owns it, when the
/// production loader can determine an owner.
///
/// Scenario, declaration-set, duplicate-identity, and genuinely aggregate
/// composition failures remain ownerless. A rule-load, scheduled-rule, or
/// finite-kernel refusal is attributed to the manifest-relative source that
/// supplied that rule. Authoring clients must not infer a sibling source from
/// a coincidentally valid [`babylon_bsl::FormPath`].
#[derive(Debug, Clone, PartialEq)]
pub struct SourcedPrepareErrorV1 {
    /// Manifest-relative rule source identity, or `None` for a content-set
    /// wide refusal.
    pub source_id: Option<String>,
    /// The production preparation refusal.
    pub error: PrepareError,
}

fn prepare_error_source_id(
    error: &PrepareError,
    rule_sources: &HashMap<String, String>,
) -> Option<String> {
    let rule_id = match error {
        PrepareError::Rule {
            rule_id: Some(rule_id),
            ..
        }
        | PrepareError::Probability {
            rule_id: Some(rule_id),
            ..
        } => Some(rule_id),
        PrepareError::Composition {
            identity: Some(ErrorIdentity::RuleId(rule_id)),
            ..
        } => Some(rule_id),
        PrepareError::Scenario(_)
        | PrepareError::Rule { rule_id: None, .. }
        | PrepareError::Intrinsic(_)
        | PrepareError::Probability { rule_id: None, .. }
        | PrepareError::KernelSlot(_)
        | PrepareError::Composition { .. } => None,
    };
    rule_id.and_then(|rule_id| rule_sources.get(rule_id).cloned())
}

/// Diagnose a named multi-source content set through the same loader,
/// governed phase schedule, and finite-kernel analysis as executable
/// preparation while retaining exact source ownership.
///
/// Ordering and continuation match [`diagnose_content_set`]. The additional
/// source identity is authoring metadata only and cannot affect mechanics.
#[must_use]
pub fn diagnose_content_set_sources(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
) -> Vec<SourcedPrepareErrorV1> {
    let mut errors = Vec::new();
    let mut intrinsic_forms: Vec<SExpr> = Vec::new();
    let mut sourced_rule_forms: Vec<(String, SExpr, FormPath, String)> = Vec::new();
    for rule_source in rule_sources {
        match split_content(rule_source.source) {
            Ok((mut intr, mut rules)) => {
                intrinsic_forms.append(&mut intr);
                sourced_rule_forms.extend(rules.drain(..).map(|rule| {
                    (
                        rule.rule_id,
                        rule.form,
                        rule.root_path,
                        rule_source.source_id.to_owned(),
                    )
                }));
            }
            Err(error) => errors.push(SourcedPrepareErrorV1 {
                source_id: Some(rule_source.source_id.to_owned()),
                error: PrepareError::Rule {
                    rule_id: None,
                    error,
                },
            }),
        }
    }
    let rule_forms = sourced_rule_forms
        .iter()
        .map(|(rule_id, form, _, _)| (rule_id.clone(), form.clone()))
        .collect::<Vec<_>>();
    let rule_source_by_id = sourced_rule_forms
        .iter()
        .map(|(rule_id, _, _, source_id)| (rule_id.clone(), source_id.clone()))
        .collect::<HashMap<_, _>>();
    let unique_rule_ids = match check_unique_rule_ids(&rule_forms) {
        Ok(()) => true,
        Err(error) => {
            errors.push(SourcedPrepareErrorV1 {
                source_id: None,
                error: PrepareError::Rule {
                    rule_id: None,
                    error,
                },
            });
            false
        }
    };

    let declared = match parse_intrinsic_decls(&intrinsic_forms) {
        Ok(declared) => declared,
        Err(e) => {
            errors.push(SourcedPrepareErrorV1 {
                source_id: None,
                error: PrepareError::Intrinsic(e),
            });
            return errors;
        }
    };
    let intrinsics = IntrinsicCosts::new(
        declared
            .into_iter()
            .map(|(name, decl)| (name, decl.cost))
            .collect(),
    );

    let mut graph = HypergraphStore::new();
    let scenario = match hydrate_scenario(scenario_src, prelude_src, &mut graph) {
        Ok(scenario) => scenario,
        Err(e) => {
            errors.push(SourcedPrepareErrorV1 {
                source_id: None,
                error: e,
            });
            return errors;
        }
    };

    let inputs = match build_shared_load_inputs(&scenario) {
        Ok(inputs) => inputs,
        Err(e) => {
            errors.push(SourcedPrepareErrorV1 {
                source_id: None,
                error: e,
            });
            return errors;
        }
    };

    let mut admitted_rule_forms = Vec::with_capacity(rule_forms.len());
    let mut admitted_rules = Vec::with_capacity(rule_forms.len());
    for (id, form, root_path, source_id) in &sourced_rule_forms {
        let context = LoadContext {
            vocabulary: &inputs.vocabulary,
            types: &inputs.types,
            enums: &scenario.enums,
            const_values: &scenario.consts,
            ceilings: &inputs.ceilings,
            intrinsics: &intrinsics,
            systems: &inputs.systems,
            vocabulary_registry: scenario.vocabulary.as_ref(),
            rule_file: source_id,
        };
        match load_rule_form(form.clone(), root_path.clone(), &context) {
            Ok(loaded) => {
                admitted_rule_forms.push((id.clone(), form.clone()));
                admitted_rules.push((id.clone(), loaded));
            }
            Err(error) => errors.push(SourcedPrepareErrorV1 {
                source_id: Some(source_id.clone()),
                error: PrepareError::Rule {
                    rule_id: Some(id.clone()),
                    error,
                },
            }),
        }
    }

    if unique_rule_ids && !admitted_rule_forms.is_empty() {
        match phase_order::compile(&admitted_rule_forms) {
            Ok(plan) => {
                if let Err(error) = enforce_ranked_composition(&plan, &admitted_rule_forms) {
                    errors.push(SourcedPrepareErrorV1 {
                        source_id: prepare_error_source_id(&error, &rule_source_by_id),
                        error,
                    });
                } else {
                    match plan.apply(admitted_rules) {
                        Ok(scheduled) => {
                            let loaded = scheduled
                                .into_iter()
                                .map(|(_, rule)| rule)
                                .collect::<Vec<_>>();
                            if let Err(error) =
                                babylon_bsl::probability::analyze_content_set(&loaded)
                            {
                                let error = probability_prepare_error(&loaded, error);
                                errors.push(SourcedPrepareErrorV1 {
                                    source_id: prepare_error_source_id(&error, &rule_source_by_id),
                                    error,
                                });
                            }
                        }
                        Err(error) => {
                            let error = prepare_error_from_schedule(error);
                            errors.push(SourcedPrepareErrorV1 {
                                source_id: prepare_error_source_id(&error, &rule_source_by_id),
                                error,
                            });
                        }
                    }
                }
            }
            Err(error) => {
                let error = prepare_error_from_schedule(error);
                errors.push(SourcedPrepareErrorV1 {
                    source_id: prepare_error_source_id(&error, &rule_source_by_id),
                    error,
                });
            }
        }
    }

    errors
}

/// One caller-owned BSL source and its stable content-set identity.
///
/// Rule identities are retained in every resulting `LoadedRule` analysis;
/// scenario and prelude identities locate loader-confirmed declaration facts.
/// File names do not change mechanics. They only map typed `FormPath`s back
/// to diagnostics and authoring facts in the correct source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRuleSourceV1<'a> {
    /// Manifest-relative source identity.
    pub source_id: &'a str,
    /// Exact current source bytes decoded as UTF-8.
    pub source: &'a str,
}

/// A source-aware analysis refusal, optionally retaining the typed analysis
/// that preceded permanent kernel-slot validation.
///
/// Tooling needs the retained paths to locate a moved or rebound reservation
/// in the author's source.  The partial analysis is present only when every
/// loader, schedule, and finite-probability check succeeded and the final
/// permanent ledger check refused the content set; it is never an executable
/// admission.
#[derive(Debug)]
pub struct ContentSetSourceAnalysisErrorV1 {
    /// The governed preparation refusal.
    pub error: PrepareError,
    /// Typed, non-executable facts available for locating a kernel-slot
    /// refusal. `None` for every earlier preparation failure.
    pub partial_analysis: Option<babylon_bsl::probability::ContentSetAnalysisV1>,
}

impl std::fmt::Display for ContentSetSourceAnalysisErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for ContentSetSourceAnalysisErrorV1 {}

/// Load and analyze a named multi-source content set through the production
/// rule loader and governed phase schedule, without running a tick or mutating
/// caller state.
///
/// This is the source-aware public wrapper around
/// `babylon_bsl::probability::analyze_content_set`. Each rule form is parsed
/// and compiled exactly once into `LoadedRule`; probability analysis consumes
/// that typed IR and never performs an independent raw-expression walk.
///
/// # Errors
///
/// Returns the same first-failing scenario, intrinsic, rule, phase, or finite
/// probability refusal as executable preparation.
pub fn analyze_content_set_sources(
    scenario_source: ContentRuleSourceV1<'_>,
    prelude_sources: &[ContentRuleSourceV1<'_>],
    rule_sources: &[ContentRuleSourceV1<'_>],
) -> Result<babylon_bsl::probability::ContentSetAnalysisV1, PrepareError> {
    analyze_content_set_sources_with_kernel_slots(
        scenario_source,
        prelude_sources,
        rule_sources,
        kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1,
    )
    .map_err(|error| error.error)
}

/// Load and analyze a named multi-source content set against one explicit,
/// fully validated permanent sample/slot ledger.
///
/// This is the tooling counterpart to executable preparation's caller-owned
/// ledger seam. A final slot refusal retains typed source paths in
/// [`ContentSetSourceAnalysisErrorV1::partial_analysis`] solely so an editor
/// can underline the exact sample or slot; callers must still treat the
/// result as refused.
///
/// # Errors
///
/// Returns the same loader, schedule, probability, and permanent slot
/// refusals as executable preparation.
pub fn analyze_content_set_sources_with_kernel_slots(
    scenario_source: ContentRuleSourceV1<'_>,
    prelude_sources: &[ContentRuleSourceV1<'_>],
    rule_sources: &[ContentRuleSourceV1<'_>],
    kernel_slots: &[kernel_slot::KernelSlotReservationV1<'_>],
) -> Result<babylon_bsl::probability::ContentSetAnalysisV1, ContentSetSourceAnalysisErrorV1> {
    let analysis = analyze_content_set_sources_before_kernel_slots(
        scenario_source,
        prelude_sources,
        rule_sources,
    )
    .map_err(|error| ContentSetSourceAnalysisErrorV1 {
        error,
        partial_analysis: None,
    })?;
    let live_kernels = analysis
        .rules
        .iter()
        .filter_map(|rule| {
            rule.kernel
                .as_ref()
                .map(|kernel| (rule.rule_id.as_str(), kernel))
        })
        .collect::<Vec<_>>();
    if let Err(error) = kernel_slot::validate_live_kernel_slots_v1(kernel_slots, &live_kernels) {
        return Err(ContentSetSourceAnalysisErrorV1 {
            error: PrepareError::KernelSlot(error),
            partial_analysis: Some(analysis),
        });
    }
    Ok(analysis)
}

fn analyze_content_set_sources_before_kernel_slots(
    scenario_source: ContentRuleSourceV1<'_>,
    prelude_sources: &[ContentRuleSourceV1<'_>],
    rule_sources: &[ContentRuleSourceV1<'_>],
) -> Result<babylon_bsl::probability::ContentSetAnalysisV1, PrepareError> {
    let mut intrinsic_forms = Vec::new();
    let mut sourced_rule_forms = Vec::new();
    for rule_source in rule_sources {
        let (mut source_intrinsics, source_rules) =
            split_content(rule_source.source).map_err(|error| PrepareError::Rule {
                rule_id: None,
                error,
            })?;
        intrinsic_forms.append(&mut source_intrinsics);
        sourced_rule_forms.extend(source_rules.into_iter().map(|rule| {
            (
                rule.rule_id,
                rule.form,
                rule.root_path,
                rule_source.source_id.to_owned(),
            )
        }));
    }
    let phase_forms = sourced_rule_forms
        .iter()
        .map(|(rule_id, form, _, _)| (rule_id.clone(), form.clone()))
        .collect::<Vec<_>>();
    check_unique_rule_ids(&phase_forms).map_err(|error| PrepareError::Rule {
        rule_id: None,
        error,
    })?;
    let declared = parse_intrinsic_decls(&intrinsic_forms).map_err(PrepareError::Intrinsic)?;
    let intrinsics = IntrinsicCosts::new(
        declared
            .into_iter()
            .map(|(name, declaration)| (name, declaration.cost))
            .collect(),
    );

    let named_preludes = prelude_sources
        .iter()
        .map(|source| babylon_bsl::NamedDeclarationPreludeV1 {
            source_id: source.source_id,
            source: source.source,
        })
        .collect::<Vec<_>>();
    let mut graph = HypergraphStore::new();
    let scenario = babylon_bsl::load_scenario_with_named_preludes(
        scenario_source.source_id,
        scenario_source.source,
        &named_preludes,
        &mut graph,
    )
    .map_err(PrepareError::Scenario)?;
    let inputs = build_shared_load_inputs(&scenario)?;
    sourced_rule_forms.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut loaded = Vec::with_capacity(sourced_rule_forms.len());
    for (rule_id, form, root_path, source_id) in sourced_rule_forms {
        let context = LoadContext {
            vocabulary: &inputs.vocabulary,
            types: &inputs.types,
            enums: &scenario.enums,
            const_values: &scenario.consts,
            ceilings: &inputs.ceilings,
            intrinsics: &intrinsics,
            systems: &inputs.systems,
            vocabulary_registry: scenario.vocabulary.as_ref(),
            rule_file: &source_id,
        };
        let rule =
            load_rule_form(form, root_path, &context).map_err(|error| PrepareError::Rule {
                rule_id: Some(rule_id.clone()),
                error,
            })?;
        loaded.push((rule_id, rule));
    }
    let schedule = phase_order::compile(&phase_forms).map_err(prepare_error_from_schedule)?;
    enforce_ranked_composition(&schedule, &phase_forms)?;
    let scheduled = schedule
        .apply(loaded)
        .map_err(prepare_error_from_schedule)?;
    let rules = scheduled
        .into_iter()
        .map(|(_, rule)| rule)
        .collect::<Vec<_>>();
    let mut analysis = babylon_bsl::probability::analyze_content_set(&rules)
        .map_err(|error| probability_prepare_error(&rules, error))?;
    analysis.mass_declarations = scenario.mass_declarations;
    Ok(analysis)
}

/// Why a requested event likelihood is outside the exact finite projection
/// boundary or why its real loader/executor refused it.
#[derive(Debug)]
pub enum ForecastErrorV1 {
    /// The content set did not load through the executable preparation path.
    Preparation(PrepareError),
    /// The request asks for something V1 deliberately cannot enumerate.
    NotExactlyEnumerable {
        /// Stable, explicit boundary explanation.
        reason: String,
    },
    /// The real detached mechanic/recognizer execution failed.
    Execution(babylon_bsl::tick::TickError),
}

impl std::fmt::Display for ForecastErrorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation(error) => write!(f, "{error}"),
            Self::NotExactlyEnumerable { reason } => {
                write!(f, "not exactly enumerable: {reason}")
            }
            Self::Execution(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ForecastErrorV1 {}

/// Forecast the exact event likelihoods for one stable subject and one
/// adjacent finite kernel/projection pair in a paired scenario.
///
/// The resolved deterministic schedule prefix is first applied to the local
/// forecast graph. Every target-kernel branch is then applied to an
/// independent detached copy of that exact pre-choice state, and the ordinary
/// deterministic recognizer is evaluated against the resulting post-branch
/// state. An earlier finite kernel is refused instead of implicitly assuming
/// independence or enumerating a cross-sample join.
/// The returned numerator is therefore the recognizer preimage's exact ticket
/// measure, never an authored event attribute or an approximation.
///
/// # Errors
///
/// Refuses content that does not load, unknown/non-node carriers, a sample
/// without one immediately adjacent projection, nonpositive ticks, and every
/// request outside the single-sample finite boundary.
pub fn forecast_event_likelihoods(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
    sample: &str,
    subject: &StableElementKeyV1,
    tick: i64,
) -> Result<Vec<babylon_bsl::probability::EventLikelihoodV1>, ForecastErrorV1> {
    forecast_event_likelihoods_with_kernel_slots(
        scenario_src,
        prelude_src,
        rule_sources,
        kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1,
        sample,
        subject,
        tick,
    )
}

/// Exact finite projection forecast using a caller-supplied, fully validated
/// permanent sample/slot ledger. Content tooling uses this seam for a
/// non-bundled content set; the ordinary API uses the bundled game ledger.
///
/// # Errors
///
/// Returns the same preparation, exact-enumeration, or execution refusal as
/// [`forecast_event_likelihoods`], including a noncanonical or mismatched
/// kernel-slot ledger.
pub fn forecast_event_likelihoods_with_kernel_slots(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
    kernel_slots: &[kernel_slot::KernelSlotReservationV1<'_>],
    sample: &str,
    subject: &StableElementKeyV1,
    tick: i64,
) -> Result<Vec<babylon_bsl::probability::EventLikelihoodV1>, ForecastErrorV1> {
    if tick <= 0 {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: format!("forecast tick must be positive, got {tick}"),
        });
    }
    let mut combined_rules = String::new();
    for source in rule_sources {
        if !combined_rules.is_empty() {
            combined_rules.push('\n');
        }
        combined_rules.push_str(source.source);
    }
    let mut graph = HypergraphStore::new();
    let prepared = prepare_rules_with_kernel_slots(
        scenario_src,
        prelude_src,
        &combined_rules,
        &mut graph,
        kernel_slots,
    )
    .map_err(ForecastErrorV1::Preparation)?;
    let StableElementKeyV1::Node {
        scenario,
        local_name,
    } = subject
    else {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: "V1 finite projections are subject-local node recognizers".to_owned(),
        });
    };
    if scenario != &prepared.scenario_scope {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "stable subject belongs to scenario `{scenario}`, not `{}`",
                prepared.scenario_scope
            ),
        });
    }
    let subject_node = prepared
        .node_content_ids
        .iter()
        .find_map(|(node, name)| (name == local_name).then_some(*node))
        .ok_or_else(|| ForecastErrorV1::NotExactlyEnumerable {
            reason: format!("scenario has no stable node `{local_name}`"),
        })?;
    let rules = prepared
        .rules
        .iter()
        .map(|(_, rule)| rule.clone())
        .collect::<Vec<_>>();
    let kernel_index = rules
        .iter()
        .position(|rule| {
            rule.kernel
                .as_ref()
                .is_some_and(|kernel| kernel.sample == sample)
        })
        .ok_or_else(|| ForecastErrorV1::NotExactlyEnumerable {
            reason: format!("content set has no finite kernel `{sample}`"),
        })?;
    let projection_is_adjacent = rules
        .get(kernel_index + 1)
        .and_then(|rule| rule.projection.as_ref())
        .is_some_and(|projection| projection.sample == sample);
    if !projection_is_adjacent {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "finite kernel `{sample}` has no immediately adjacent deterministic projection"
            ),
        });
    }
    let mechanic_subject_type =
        babylon_bsl::tick::subject_type_of_rule(&rules[kernel_index], prepared.vocabulary.as_ref())
            .map_err(ForecastErrorV1::Execution)?;
    let actual_subject_type = graph.node_type_of(subject_node).map_err(|error| {
        ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "stable subject `{local_name}` has no resolvable node type: {}",
                error.message
            ),
        }
    })?;
    if actual_subject_type != mechanic_subject_type {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "stable subject `{local_name}` has node type `{actual_subject_type}`, but finite kernel `{sample}` mechanic runs over `{mechanic_subject_type}`"
            ),
        });
    }
    let resolver = StableElementResolverV1::seal(
        &graph,
        &prepared.scenario_scope,
        &prepared.node_content_ids,
        &prepared.hyperedge_content_ids,
    )
    .map_err(|error| ForecastErrorV1::NotExactlyEnumerable {
        reason: format!("forecast scenario has no sealed stable carrier map: {error:?}"),
    })?;
    let analysis_session =
        ReplaySessionIdV1::try_from("analysis/finite-projection-v1").map_err(|error| {
            ForecastErrorV1::NotExactlyEnumerable {
                reason: format!("forecast replay identity refused: {error:?}"),
            }
        })?;
    let analysis_seed = ReplaySeed::new(0);
    for prior in rules.iter().take(kernel_index) {
        if let Some(kernel) = &prior.kernel {
            return Err(ForecastErrorV1::NotExactlyEnumerable {
                reason: format!(
                    "resolved pre-choice state depends on earlier finite kernel `{}`; V1 does not enumerate cross-sample paths",
                    kernel.sample
                ),
            });
        }
        let mut ignored_events = CollectingSink::default();
        let mut ignored_writes = CollectingWriteLog::new();
        let outcome = run_tick_observed(
            prior,
            &prepared.types,
            &prepared.enums,
            &KernelIntrinsicHost,
            &mut graph,
            &mut ignored_events,
            &prepared.intrinsics,
            &prepared.consts,
            tick,
            Some(&prepared.node_content_ids),
            RngSeedContext::V2 {
                session: &analysis_session,
                seed: analysis_seed,
            },
            Some(&resolver),
            prepared.vocabulary.as_ref(),
            &mut ignored_writes,
        )
        .map_err(ForecastErrorV1::Execution)?;
        if !outcome.kernel_realizations.is_empty() {
            return Err(ForecastErrorV1::NotExactlyEnumerable {
                reason: "deterministic forecast prefix produced an unexpected finite realization"
                    .to_owned(),
            });
        }
    }
    let context = babylon_bsl::tick::ForecastContextV1 {
        types: &prepared.types,
        enums: &prepared.enums,
        host: &KernelIntrinsicHost,
        costs: &prepared.intrinsics,
        defines: &prepared.consts,
        tick,
        vocabulary: prepared.vocabulary.as_ref(),
    };
    babylon_bsl::tick::forecast_event_likelihoods(
        &rules,
        kernel_index,
        &graph,
        subject_node,
        &context,
    )
    .map_err(ForecastErrorV1::Execution)
}

/// Forecast without guessing a carrier or calendar position when the paired
/// scenario itself proves both irrelevant or unique.
///
/// V1 publishes an exact authoring-time likelihood only when every scheduled
/// rule executed through the adjacent projection is calendar-independent and
/// the mechanic's node domain has exactly one authored carrier. Otherwise
/// this returns the same explicit `not exactly enumerable` boundary the LSP
/// renders as state dependence.
///
/// # Errors
///
/// Returns [`ForecastErrorV1`] for loader/executor failures or when the paired
/// scenario does not uniquely determine an exact finite instance.
pub fn forecast_scenario_determined_event_likelihoods(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
    sample: &str,
) -> Result<Vec<babylon_bsl::probability::EventLikelihoodV1>, ForecastErrorV1> {
    forecast_scenario_determined_event_likelihoods_with_kernel_slots(
        scenario_src,
        prelude_src,
        rule_sources,
        kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1,
        sample,
    )
}

/// Scenario-determined exact forecast for one explicitly governed content
/// ledger. This is the non-bundled counterpart to
/// [`forecast_scenario_determined_event_likelihoods`].
///
/// # Errors
///
/// Returns the same closed preparation, exact-enumeration, or execution
/// refusal, including any permanent reservation mismatch.
pub fn forecast_scenario_determined_event_likelihoods_with_kernel_slots(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_sources: &[ContentRuleSourceV1<'_>],
    kernel_slots: &[kernel_slot::KernelSlotReservationV1<'_>],
    sample: &str,
) -> Result<Vec<babylon_bsl::probability::EventLikelihoodV1>, ForecastErrorV1> {
    let combined_rules = rule_sources
        .iter()
        .map(|source| source.source)
        .collect::<Vec<_>>()
        .join("\n");
    let mut graph = HypergraphStore::new();
    let prepared = prepare_rules_with_kernel_slots(
        scenario_src,
        prelude_src,
        &combined_rules,
        &mut graph,
        kernel_slots,
    )
    .map_err(ForecastErrorV1::Preparation)?;
    let kernel_index = prepared
        .rules
        .iter()
        .position(|(_, rule)| {
            rule.kernel
                .as_ref()
                .is_some_and(|kernel| kernel.sample == sample)
        })
        .ok_or_else(|| ForecastErrorV1::NotExactlyEnumerable {
            reason: format!("content set has no finite kernel `{sample}`"),
        })?;
    let mechanic = &prepared.rules[kernel_index].1;
    prepared
        .rules
        .get(kernel_index + 1)
        .filter(|(_, rule)| {
            rule.projection
                .as_ref()
                .is_some_and(|projection| projection.sample == sample)
        })
        .ok_or_else(|| ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "finite kernel `{sample}` has no immediately adjacent deterministic projection"
            ),
        })?;
    let reads_calendar = prepared
        .rules
        .iter()
        .take(kernel_index + 2)
        .flat_map(|(_, rule)| &rule.bindings)
        .any(|binding| {
            matches!(
                binding.source,
                babylon_bsl::bindings::BindSource::Tick
                    | babylon_bsl::bindings::BindSource::Year
                    | babylon_bsl::bindings::BindSource::TickOfYear
                    | babylon_bsl::bindings::BindSource::TickInCycle(_)
            )
        });
    if reads_calendar {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: "resolved forecast prefix, kernel, or projection depends on the forecast tick"
                .to_owned(),
        });
    }
    let subject_type =
        babylon_bsl::tick::subject_type_of_rule(mechanic, prepared.vocabulary.as_ref())
            .map_err(ForecastErrorV1::Execution)?;
    let subjects = graph.nodes(&subject_type);
    let [subject_node] = subjects.as_slice() else {
        return Err(ForecastErrorV1::NotExactlyEnumerable {
            reason: format!(
                "paired scenario determines {} `{subject_type}` carriers, not exactly one",
                subjects.len()
            ),
        });
    };
    let local_name = prepared.node_content_ids.get(subject_node).ok_or_else(|| {
        ForecastErrorV1::NotExactlyEnumerable {
            reason: "the unique forecast carrier has no authored stable identity".to_owned(),
        }
    })?;
    let subject = StableElementKeyV1::Node {
        scenario: prepared.scenario_scope.clone(),
        local_name: local_name.clone(),
    };
    // Tick 1 is observationally irrelevant here: the binding scan above
    // proves every rule this bounded forecast executes is independent of all
    // calendar projections.
    forecast_event_likelihoods_with_kernel_slots(
        scenario_src,
        prelude_src,
        rule_sources,
        kernel_slots,
        sample,
        &subject,
        1,
    )
}

// `Result<_, PrepareError>` here is covered by this module's top-of-file
// `#![allow(clippy::result_large_err)]` — see that citation.
pub(crate) fn prepare_rules<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_src: &str,
    graph: &mut G,
) -> Result<PreparedRules, PrepareError> {
    prepare_rules_with_kernel_slots(
        scenario_src,
        prelude_src,
        rule_src,
        graph,
        kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1,
    )
}

fn prepare_rules_with_kernel_slots<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    // Train B item 4 (#591, D157): `None` for every pre-existing caller
    // (`run_once_into`, `TickSession::new`) — behavior unchanged, byte for
    // byte. `Some(prelude)` routes the scenario load through
    // `load_scenario_with_prelude` instead of `load_scenario`.
    prelude_src: Option<&str>,
    rule_src: &str,
    graph: &mut G,
    kernel_slots: &[kernel_slot::KernelSlotReservationV1<'_>],
) -> Result<PreparedRules, PrepareError> {
    // §2.2's `<intrinsic-decl>` top-forms, split from the `(rule …)` forms
    // they may share a source with (`split_content`), then parsed into the
    // `IntrinsicCosts` the loader's static bound check AND the evaluator
    // both need — refusing loudly and wholesale on the first bad
    // declaration, including a duplicate name (`E-LOAD-001`) or a
    // signature disagreeing with the kernel's registration (`E-LOAD-020`),
    // never a partial admission of the ones that do qualify.
    let (intrinsic_forms, rule_forms) =
        split_content(rule_src).map_err(|error| PrepareError::Rule {
            rule_id: None,
            error,
        })?;
    let declared = parse_intrinsic_decls(&intrinsic_forms).map_err(PrepareError::Intrinsic)?;
    let intrinsics = IntrinsicCosts::new(
        declared
            .into_iter()
            .map(|(name, decl)| (name, decl.cost))
            .collect(),
    );

    // Validate scenario declarations and rule surfaces against a disposable
    // graph first. This preserves the established scenario -> rule ->
    // composition error order while ensuring a phase-composition refusal
    // cannot partially hydrate the caller-owned graph.
    let mut validation_graph = HypergraphStore::new();
    let validation_scenario = hydrate_scenario(scenario_src, prelude_src, &mut validation_graph)?;

    // The scenario's `deffield` forms ARE the registries for slice 1. When
    // Phase 2's content registries land they replace this wholesale; until
    // then a field's type and intensivity come from a declaration rather
    // than from a guess about its stored value. The D32 implicit-
    // `<edge-type>/strength` seeding (and its own duplicate-declaration
    // refusal) is [`seed_implicit_edge_strength_fields`]'s own doc — shared
    // with [`diagnose_content_set`] via [`build_shared_load_inputs`].
    let inputs = build_shared_load_inputs(&validation_scenario)?;

    // ONE shared LoadContext for every rule in the content set — the
    // vocabulary/types/ceilings come from the SCENARIO, not from any one
    // rule, and each rule's own load only reads the subset its bindings
    // reference (verified: no cross-rule interference for vitality +
    // lifecycle, whose bindings are wholly disjoint — see the plan's
    // Multi-Rule Decision section's domain-disjointness note).
    let ctx = LoadContext {
        vocabulary: &inputs.vocabulary,
        types: &inputs.types,
        enums: &validation_scenario.enums,
        const_values: &validation_scenario.consts,
        ceilings: &inputs.ceilings,
        intrinsics: &intrinsics,
        systems: &inputs.systems,
        // The R9 chapters' vocabulary-dependent gates (D37's field-init
        // owner rule, D43's domain inference, §2.5's foreign-`:field`
        // scoping) AND Task 8's closed-vocabulary membership enforcement
        // (E-LOAD-030/031) need a `ClosedVocabulary`. Task 7 landed the
        // scenario-side declaration form (`defvocabulary`); Task 8 wires
        // enforcement live — whatever the scenario declared (`Some`, or
        // `None` for a scenario declaring none at all, exactly today's
        // unchecked behavior) is what every rule loads against.
        vocabulary_registry: validation_scenario.vocabulary.as_ref(),
        rule_file: "rule",
    };

    // `rule_forms` retains each rule id, form, and original top-form path
    // from `split_content`, so loader diagnostics stay in the complete source
    // coordinate space. Validate a temporary reference view by ascending
    // rule-id bytes so two invalid source permutations name the same first
    // failing identity.
    // The preflighted plan then places valid rules on the 34-slot causal
    // spine; D16 breaks only same-position execution ties by rule-id bytes.
    // Every later stage just iterates that compiled execution order.
    let mut validation_order: Vec<_> = rule_forms.iter().collect();
    validation_order
        .sort_unstable_by(|left, right| left.rule_id.as_bytes().cmp(right.rule_id.as_bytes()));
    let mut rules = Vec::with_capacity(rule_forms.len());
    for rule in validation_order {
        let loaded =
            load_rule_form(rule.form.clone(), rule.root_path.clone(), &ctx).map_err(|error| {
                PrepareError::Rule {
                    rule_id: Some(rule.rule_id.clone()),
                    error,
                }
            })?;
        rules.push((rule.rule_id.clone(), loaded));
    }
    let phase_forms = rule_forms
        .iter()
        .map(|rule| (rule.rule_id.clone(), rule.form.clone()))
        .collect::<Vec<_>>();
    let rule_order = phase_order::compile(&phase_forms).map_err(prepare_error_from_schedule)?;
    enforce_ranked_composition(&rule_order, &phase_forms)?;
    let rules = rule_order
        .apply(rules)
        .map_err(prepare_error_from_schedule)?;
    let loaded_rules = rules
        .iter()
        .map(|(_, rule)| rule.clone())
        .collect::<Vec<_>>();
    babylon_bsl::probability::analyze_content_set(&loaded_rules)
        .map_err(|error| probability_prepare_error(&loaded_rules, error))?;
    let live_kernels = rules
        .iter()
        .filter_map(|(rule_id, rule)| {
            rule.kernel
                .as_ref()
                .map(|kernel| (rule_id.as_str(), kernel))
        })
        .collect::<Vec<_>>();
    kernel_slot::validate_live_kernel_slots_v1(kernel_slots, &live_kernels)
        .map_err(PrepareError::KernelSlot)?;

    // All non-mutating validation has succeeded. Hydrate the caller graph
    // exactly once and retain this pass's content-to-node identities, which
    // may differ from the disposable graph when the caller was non-empty.
    let scenario = hydrate_scenario(scenario_src, prelude_src, graph)?;

    Ok(PreparedRules {
        rules,
        rule_forms: rule_forms.into_iter().map(|rule| rule.form).collect(),
        types: inputs.types,
        intrinsics,
        consts: scenario.consts,
        enums: scenario.enums,
        node_content_ids: scenario.node_content_ids,
        scenario_scope: scenario.id,
        hyperedge_content_ids: scenario.hyperedge_content_ids,
        vocabulary: scenario.vocabulary,
    })
}

/// `run_once`, with the world and the event sink supplied by the caller so
/// they survive the call.
///
/// `run_once` returns hashes, which prove that state MOVED and that it moves
/// the same way twice — but a conformance vector has to name the values a
/// named class ended the tick holding, and an emitted event has to be
/// inspectable at all. Threading the two outputs through a parameter keeps
/// **one** implementation of the flow: `run_once`'s signature and
/// [`TickReport`] are the seam `babylon-client` consumes and neither moves.
///
/// Generic over the substrate. Atomic adjudication requires
/// [`DetachedCopy`] for a disposable working world; [`AllocatorState`]
/// supplies the non-graph identity cursors covered by the nominal world hash.
///
/// # Errors
///
/// A description of the first failing stage — an intrinsic declaration, a
/// scenario load, a rule load, a state hash, or the tick itself.
pub fn run_once_into<G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy>(
    scenario_src: &str,
    rule_src: &str,
    graph: &mut G,
    sink: &mut CollectingSink,
) -> Result<TickReport, String> {
    let mut candidate = graph.detached_copy();
    let prepared =
        prepare_rules(scenario_src, None, rule_src, &mut candidate).map_err(|e| e.to_string())?;
    let report = run_prepared_tick(&prepared, &mut candidate, sink, &run_once_session(), 1)?;
    *graph = candidate;
    Ok(report)
}

/// `run_once_into`, with the scenario load routed through a **declaration
/// prelude** first — the caller-supplied-graph/sink sibling of
/// [`run_once_with_prelude`], exactly as `run_once_into` is `run_once`'s
/// (Train B item 4, issue #591, D157). Added alongside
/// [`run_once_with_prelude`] because `consciousness_ternary_conformance.rs`
/// needs it directly: once `consciousness-ternary-conformance.bscn` stopped
/// re-declaring `WorldView` itself (this train), every one of its callers —
/// not only `tick_goldens.rs`'s golden — needs the prelude, and this
/// module's other entry points (`run_once`, `TickSession`) are all it has
/// to route through.
///
/// # Errors
///
/// A description of the first failing stage — the prelude, an intrinsic
/// declaration, a scenario load, a rule load, a state hash, or the tick
/// itself.
pub fn run_once_into_with_prelude<
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
>(
    scenario_src: &str,
    prelude_src: &str,
    rule_src: &str,
    graph: &mut G,
    sink: &mut CollectingSink,
) -> Result<TickReport, String> {
    let mut candidate = graph.detached_copy();
    let prepared = prepare_rules(scenario_src, Some(prelude_src), rule_src, &mut candidate)
        .map_err(|e| e.to_string())?;
    let report = run_prepared_tick(&prepared, &mut candidate, sink, &run_once_session(), 1)?;
    *graph = candidate;
    Ok(report)
}

/// Deterministic V1 execution namespace for every one-shot driver in this
/// module. These drivers are pinned at tick 1 and cannot realize a finite
/// kernel; governed choice requires [`replay_session::ReplayTickSession`].
/// The fixed literal remains non-random and never reads a UUID or wall clock.
fn run_once_session() -> SessionId {
    SessionId::new("run-once").expect("literal is non-empty")
}

/// `run_once_with_prelude` (this train) and `run_once_into` (above) share
/// this from the point `prepare_rules` has already returned: clone the
/// committed graph, run every prepared rule to completion against that one
/// working copy, buffer its events, and publish both only after every rule
/// and hash succeeds. Extracted so a prelude-bearing caller does not
/// duplicate this loop — `prepare_rules`'s own `prelude_src` parameter is
/// the only thing that differs between the two callers, and it is fully
/// resolved before this function ever runs.
///
/// # Errors
///
/// An invalid tick number, schedule/world/graph hashing, event reservation,
/// or the tick itself (named to its own rule id). Every error leaves the
/// caller's graph and existing events unchanged.
pub(crate) fn run_prepared_tick<
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
>(
    prepared: &PreparedRules,
    graph: &mut G,
    sink: &mut CollectingSink,
    session: &SessionId,
    tick: i64,
) -> Result<TickReport, String> {
    run_prepared_tick_with(
        prepared,
        graph,
        sink,
        RngSeedContext::V1 { session },
        None,
        tick,
        |_boundary, candidate| candidate.state_hash(),
    )
}

fn checked_fired_total(per_rule_fired: &[(String, usize)]) -> Result<usize, String> {
    per_rule_fired.iter().try_fold(0usize, |total, (_, fired)| {
        total
            .checked_add(*fired)
            .ok_or_else(|| "tick fired-subject total overflowed usize".to_owned())
    })
}

fn checked_considered_total(per_rule_considered: &[(String, usize)]) -> Result<usize, String> {
    per_rule_considered
        .iter()
        .try_fold(0usize, |total, (_, considered)| {
            total
                .checked_add(*considered)
                .ok_or_else(|| "tick considered-subject total overflowed usize".to_owned())
        })
}

enum ExecutionIdentity<'a, C> {
    Current {
        rng_seed: RngSeedContext<'a>,
        stable_resolver: Option<&'a StableElementResolverV1>,
    },
    Replay(replay_session::ReplayExecutionInputs<'a, C>),
}

impl<C> ExecutionIdentity<'_, C> {
    fn rng_seed(&self) -> RngSeedContext<'_> {
        match self {
            Self::Current { rng_seed, .. } => *rng_seed,
            Self::Replay(execution) => RngSeedContext::V2 {
                session: execution.session,
                seed: execution.seed,
            },
        }
    }

    fn stable_resolver(&self) -> Option<&StableElementResolverV1> {
        match self {
            Self::Current {
                stable_resolver, ..
            } => *stable_resolver,
            Self::Replay(execution) => Some(execution.resolver),
        }
    }

    const fn is_replay(&self) -> bool {
        matches!(self, Self::Replay(_))
    }
}

struct TickTransactionResult {
    report: TickReport,
    replay: Option<replay_session::ReplayIdentityArtifactsV2>,
}

struct TransactionPrelude {
    schedule_digest: [u8; 32],
    before: [u8; 32],
    world_before: [u8; 32],
    replay_prior: Option<replay_session::ReplayPriorIdentityV1>,
}

struct ExecutedRules<G> {
    graph: G,
    sink: CollectingSink,
    considered: usize,
    fired: usize,
    per_rule_considered: Vec<(String, usize)>,
    per_rule_fired: Vec<(String, usize)>,
    audit_receipts: Vec<AuditReceipt>,
    choice_receipts: Vec<choice_receipt::ChoiceReceiptV1>,
    committed_events: Vec<committed_event::CommittedEventV2>,
}

enum TickTransactionError {
    Current(String),
    Replay(ReplayTickError),
}

fn transaction_error<C>(
    identity: &ExecutionIdentity<'_, C>,
    message: String,
) -> TickTransactionError {
    if identity.is_replay() {
        TickTransactionError::Replay(ReplayTickError::Execution { message })
    } else {
        TickTransactionError::Current(message)
    }
}

fn run_prepared_tick_with<G, B, H>(
    prepared: &PreparedRules,
    graph: &mut G,
    sink: &mut B,
    rng_seed: RngSeedContext<'_>,
    stable_resolver: Option<&StableElementResolverV1>,
    tick: i64,
    state_hash: H,
) -> Result<TickReport, String>
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
    B: PreparedEventBatchSink,
    H: FnMut(HashBoundary, &G) -> Result<[u8; 32], GraphError>,
{
    let identity: ExecutionIdentity<'_, replay_session::ProductionReplayIdentityComposer> =
        ExecutionIdentity::Current {
            rng_seed,
            stable_resolver,
        };
    run_prepared_tick_transaction(prepared, graph, sink, &identity, tick, state_hash)
        .map(|result| result.report)
        .map_err(|error| match error {
            TickTransactionError::Current(message) => message,
            TickTransactionError::Replay(replay) => replay.to_string(),
        })
}

pub(crate) fn run_prepared_replay_tick<G, C>(
    prepared: &PreparedRules,
    graph: &mut G,
    sink: &mut CollectingSink,
    tick: i64,
    execution: replay_session::ReplayExecutionInputs<'_, C>,
) -> Result<IdentifiedTickReportV2, ReplayTickError>
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
    C: replay_session::ReplayIdentityComposer,
{
    let identity = ExecutionIdentity::Replay(execution);
    let result = run_prepared_tick_transaction(
        prepared,
        graph,
        sink,
        &identity,
        tick,
        |_boundary, candidate| candidate.state_hash(),
    )
    .map_err(|error| match error {
        TickTransactionError::Current(message) => ReplayTickError::Execution { message },
        TickTransactionError::Replay(replay) => replay,
    })?;
    let artifacts = result.replay.ok_or_else(|| ReplayTickError::Composer {
        message: "replay transaction returned no identity artifacts".to_owned(),
    })?;
    Ok(replay_session::identified_report(result.report, artifacts))
}

fn run_prepared_tick_transaction<G, B, H, C>(
    prepared: &PreparedRules,
    graph: &mut G,
    sink: &mut B,
    identity: &ExecutionIdentity<'_, C>,
    tick: i64,
    mut state_hash: H,
) -> Result<TickTransactionResult, TickTransactionError>
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
    B: PreparedEventBatchSink,
    H: FnMut(HashBoundary, &G) -> Result<[u8; 32], GraphError>,
    C: replay_session::ReplayIdentityComposer,
{
    let prelude = prepare_tick_transaction(graph, identity, tick, &mut state_hash)?;
    let executed = execute_prepared_rules(prepared, graph, identity, tick)?;
    complete_tick_transaction(
        prepared,
        graph,
        sink,
        identity,
        tick,
        &mut state_hash,
        prelude,
        executed,
    )
}

fn prepare_tick_transaction<G, H, C>(
    graph: &G,
    identity: &ExecutionIdentity<'_, C>,
    tick: i64,
    state_hash: &mut H,
) -> Result<TransactionPrelude, TickTransactionError>
where
    G: CanonicalState + AllocatorState,
    H: FnMut(HashBoundary, &G) -> Result<[u8; 32], GraphError>,
{
    let completed_before = tick
        .checked_sub(1)
        .filter(|completed| *completed >= 0)
        .ok_or_else(|| {
            transaction_error(
                identity,
                format!("tick to adjudicate must be positive, got {tick}"),
            )
        })?;
    if let ExecutionIdentity::Replay(execution) = identity {
        replay_session::validate_replay_actions(execution, tick)
            .map_err(TickTransactionError::Replay)?;
    }
    let schedule_digest = phase_order::schedule_digest()
        .map_err(|error| transaction_error(identity, error.to_string()))?;
    let before = state_hash(HashBoundary::Pre, graph).map_err(|error| {
        transaction_error(identity, format!("pre-tick state: {}", error.message))
    })?;
    let world_before = world_hash::nominal_world_hash(
        before,
        completed_before,
        graph.allocator_cursors(),
        schedule_digest,
    )
    .map_err(|error| transaction_error(identity, error))?;
    let replay_prior = match identity {
        ExecutionIdentity::Current { .. } => None,
        ExecutionIdentity::Replay(execution) => Some(
            replay_session::compose_replay_prior(graph, execution, completed_before)
                .map_err(TickTransactionError::Replay)?,
        ),
    };
    Ok(TransactionPrelude {
        schedule_digest,
        before,
        world_before,
        replay_prior,
    })
}

fn execute_prepared_rules<G, C>(
    prepared: &PreparedRules,
    graph: &G,
    identity: &ExecutionIdentity<'_, C>,
    tick: i64,
) -> Result<ExecutedRules<G>, TickTransactionError>
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
{
    let mut working_graph = graph.detached_copy();
    let mut working_sink = CollectingSink::default();
    let mut per_rule_considered = Vec::with_capacity(prepared.rules.len());
    let mut per_rule_fired = Vec::with_capacity(prepared.rules.len());
    let mut audit_receipts = Vec::new();
    let mut choice_receipts = Vec::new();
    let mut committed_events = Vec::new();
    let mut choice_by_sample_subject = HashMap::new();
    for (id, loaded) in &prepared.rules {
        let event_start = working_sink.events.len();
        let mut write_log = CollectingWriteLog::new();
        let outcome = run_tick_observed(
            loaded,
            &prepared.types,
            &prepared.enums,
            &KernelIntrinsicHost,
            &mut working_graph,
            &mut working_sink,
            &prepared.intrinsics,
            &prepared.consts,
            // One-shot callers pass tick 1; persistent sessions pass their
            // checked next tick. §2.5's `:tick`/`:tick-in-cycle` bindings
            // read it; `:year`/`:tick-of-year` need an epoch slice 1 does
            // not pin and are refused by name at `run_tick` entry.
            tick,
            Some(&prepared.node_content_ids),
            identity.rng_seed(),
            identity.stable_resolver(),
            prepared.vocabulary.as_ref(),
            &mut write_log,
        )
        .map_err(|error| {
            transaction_error(identity, format!("tick failed in rule {id}: {error}"))
        })?;
        for realization in outcome.kernel_realizations {
            let RngSeedContext::V2 { session, seed } = identity.rng_seed() else {
                return Err(transaction_error(
                    identity,
                    format!(
                        "finite choice in rule {id} reached the transaction without a sealed V2 replay identity"
                    ),
                ));
            };
            let encounter_ordinal = u32::try_from(choice_receipts.len()).map_err(|_| {
                transaction_error(
                    identity,
                    "finite-choice receipt count exceeds the u32 encounter-ordinal domain"
                        .to_owned(),
                )
            })?;
            let instance = KernelInstanceIdentityV1 {
                replay_session: session.as_bytes().to_vec(),
                replay_seed: seed.to_be_bytes(),
                tick,
                rule_id: realization.rule_id.clone(),
                subject: realization.subject.clone(),
                active_elements: realization.active_elements.clone(),
            };
            let receipt =
                choice_receipt::ChoiceReceiptV1::try_new(encounter_ordinal, &instance, realization)
                    .map_err(|error| {
                        transaction_error(
                            identity,
                            format!("choice receipt refused in rule {id}: {error}"),
                        )
                    })?;
            let lookup = (
                receipt.sample().to_owned(),
                receipt.stable_carrier().clone(),
            );
            if choice_by_sample_subject
                .insert(lookup, receipt.reference())
                .is_some()
            {
                return Err(transaction_error(
                    identity,
                    format!(
                        "finite choice in rule {id} repeated one sample and stable carrier in the same tick"
                    ),
                ));
            }
            choice_receipts.push(receipt);
        }
        let new_events = &working_sink.events[event_start..];
        let event_subjects = outcome.event_provenance;
        if matches!(identity.rng_seed(), RngSeedContext::V2 { .. })
            && event_subjects.len() != new_events.len()
        {
            return Err(transaction_error(
                identity,
                format!(
                    "event provenance count in rule {id} is {}, but the rule emitted {} events",
                    event_subjects.len(),
                    new_events.len()
                ),
            ));
        }
        for (event_index, (event_type, payload)) in new_events.iter().enumerate() {
            let choice_receipt = if let Some(projection) = loaded.projection.as_ref() {
                let subject = event_subjects.get(event_index).ok_or_else(|| {
                    transaction_error(
                        identity,
                        format!(
                            "finite projection in rule {id} emitted an event without stable subject provenance"
                        ),
                    )
                })?;
                choice_by_sample_subject
                    .get(&(projection.sample.clone(), subject.subject.clone()))
                    .copied()
                    .ok_or_else(|| {
                        transaction_error(
                            identity,
                            format!(
                                "finite projection in rule {id} has no realized receipt for sample `{}` and its stable subject",
                                projection.sample
                            ),
                        )
                    })?
                    .into()
            } else {
                None
            };
            committed_events.push(committed_event::CommittedEventV2::new(
                id.clone(),
                choice_receipt,
                event_type.clone(),
                payload.clone(),
            ));
        }
        let emitted_event_types = working_sink.events[event_start..]
            .iter()
            .map(|(event_type, _)| event_type.clone())
            .collect::<Vec<_>>();
        let mut rule_receipts =
            reduce_audit_receipts(&loaded.contract, &emitted_event_types, &write_log.records)
                .map_err(|error| {
                    transaction_error(
                        identity,
                        format!("causal receipt refused in rule {id}: {error}"),
                    )
                })?;
        audit_receipts.append(&mut rule_receipts);
        per_rule_considered.push((id.clone(), outcome.considered));
        per_rule_fired.push((id.clone(), outcome.fired));
    }
    let considered = checked_considered_total(&per_rule_considered)
        .map_err(|error| transaction_error(identity, error))?;
    let fired =
        checked_fired_total(&per_rule_fired).map_err(|error| transaction_error(identity, error))?;
    Ok(ExecutedRules {
        graph: working_graph,
        sink: working_sink,
        considered,
        fired,
        per_rule_considered,
        per_rule_fired,
        audit_receipts,
        choice_receipts,
        committed_events,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the shared transaction keeps state publication at one explicit boundary"
)]
fn complete_tick_transaction<G, B, H, C>(
    prepared: &PreparedRules,
    graph: &mut G,
    sink: &mut B,
    identity: &ExecutionIdentity<'_, C>,
    tick: i64,
    state_hash: &mut H,
    prelude: TransactionPrelude,
    executed: ExecutedRules<G>,
) -> Result<TickTransactionResult, TickTransactionError>
where
    G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy,
    B: PreparedEventBatchSink,
    H: FnMut(HashBoundary, &G) -> Result<[u8; 32], GraphError>,
    C: replay_session::ReplayIdentityComposer,
{
    let after = state_hash(HashBoundary::Post, &executed.graph).map_err(|error| {
        transaction_error(identity, format!("post-tick state: {}", error.message))
    })?;
    let world_after = world_hash::nominal_world_hash(
        after,
        tick,
        executed.graph.allocator_cursors(),
        prelude.schedule_digest,
    )
    .map_err(|error| transaction_error(identity, error))?;
    let report = TickReport {
        before: prelude.before,
        after,
        world_before: prelude.world_before,
        world_after,
        considered: executed.considered,
        fired: executed.fired,
        per_rule_considered: executed.per_rule_considered,
        per_rule_fired: executed.per_rule_fired,
        audit_receipts: executed.audit_receipts,
        choice_receipts: executed.choice_receipts,
        committed_events: executed.committed_events,
    };
    let replay = match (identity, prelude.replay_prior) {
        (ExecutionIdentity::Current { .. }, None) => None,
        (ExecutionIdentity::Replay(execution), Some(prior)) => Some(
            execution
                .composer
                .compose(replay_session::ReplayIdentityInputs {
                    execution,
                    prepared,
                    prior,
                    result_graph: &executed.graph,
                    report: &report,
                    events: &executed.sink.events,
                    resolve_tick: tick,
                })
                .map_err(TickTransactionError::Replay)?,
        ),
        _ => {
            return Err(transaction_error(
                identity,
                "replay prior identity invariant failed".to_owned(),
            ));
        }
    };
    sink.try_prepare(executed.sink.events.len())
        .map_err(|error| transaction_error(identity, error))?;
    *graph = executed.graph;
    sink.commit_prepared(executed.sink.events);

    Ok(TickTransactionResult { report, replay })
}

/// One rule's declared-vs-computed fuel bound (Task W3, BSL Hygiene
/// Knock-out: "the fuel-bound report mode" — "measure without the red-run
/// ritual"). `declared` is [`LoadedRule::declared_fuel`] (the author's
/// `:fuel`, §2.2); `computed` is [`LoadedRule::static_bound`], the load-time
/// proof `bound_checker::check_rule` already computes on every successful
/// load (`bound_checker.rs:757-769`) — this struct adds no new bound math,
/// it only surfaces two fields `prepare_rules` already produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelBoundRow {
    pub rule_id: String,
    pub declared: u64,
    pub computed: u64,
}

impl FuelBoundRow {
    /// `declared − computed`: fuel budget unused by the load-time proof.
    /// Saturating, matching `bound_checker`'s own "all arithmetic is
    /// saturating" stance (its module doc) — defense in depth, since a row
    /// this crate actually returns can never have `computed > declared` in
    /// the first place (`check_rule`'s `E-LOAD-040` refuses that AT LOAD,
    /// before `prepare_rules` ever builds a `LoadedRule` to read from).
    #[must_use]
    pub fn headroom(&self) -> u64 {
        self.declared.saturating_sub(self.computed)
    }
}

impl std::fmt::Display for FuelBoundRow {
    /// The report line format the brief specifies verbatim: `rule-id
    /// declared=<n> computed=<m> headroom=<n-m>`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} declared={} computed={} headroom={}",
            self.rule_id,
            self.declared,
            self.computed,
            self.headroom()
        )
    }
}

/// Load one content set (scenario, optional declaration prelude, rule
/// source) through the REAL production pipeline (`prepare_rules` — the same
/// function `run_once`/`TickSession::new` use) and report its fuel bounds,
/// one row per rule, in the SAME governed phase order `prepare_rules`
/// compiles. D16's ascending rule-ID byte order breaks same-position ties.
///
/// This is a REPORT, not a gate: it runs no tick and changes no load
/// behavior. The "same content the loader loads" guarantee is structural,
/// not a separate claim to verify — this function calls the identical
/// `prepare_rules` seam `run_once` calls, never a second reader/parser.
///
/// # Errors
///
/// The first-failing load stage's message, exactly as `run_once`/
/// `run_once_with_prelude` report it — including, redundantly, an
/// `E-LOAD-040` message if some rule's declared budget is under its
/// computed bound. That refusal already prevents such content from
/// loading at all, so a caller only ever reaching content that loads
/// clean never observes this branch from here — see [`any_over_budget`]'s
/// own doc for where this redundancy is exercised on the report side too.
pub fn fuel_bound_report(
    scenario_src: &str,
    prelude_src: Option<&str>,
    rule_src: &str,
) -> Result<Vec<FuelBoundRow>, String> {
    let mut graph = HypergraphStore::new();
    let prepared = prepare_rules(scenario_src, prelude_src, rule_src, &mut graph)
        .map_err(|e| e.to_string())?;
    Ok(prepared
        .rules
        .iter()
        .map(|(id, loaded)| FuelBoundRow {
            rule_id: id.clone(),
            declared: loaded.declared_fuel,
            computed: loaded.static_bound,
        })
        .collect())
}

/// Whether any row's computed bound exceeds its declared budget — a fuel
/// report's non-zero-exit condition (Task W3: "non-zero exit ONLY on
/// declared < computed").
///
/// **Documented redundancy:** [`fuel_bound_report`] can never actually
/// return a row with `computed > declared` for content that loaded at all —
/// `bound_checker::check_rule`'s `E-LOAD-040` already refuses that
/// condition AT LOAD, so `prepare_rules` returns `Err` before any such row
/// exists. This predicate exists anyway so the report path states its own
/// exit contract explicitly rather than assuming the invariant silently,
/// and so it degrades to a named contract violation (not a wrong/missing
/// exit code) if that invariant is ever broken by a future change to
/// either checker.
#[must_use]
pub fn any_over_budget(rows: &[FuelBoundRow]) -> bool {
    rows.iter().any(|row| row.computed > row.declared)
}

#[cfg(test)]
mod tests {
    use super::{
        any_over_budget, checked_fired_total, fuel_bound_report, prepare_rules,
        prepare_rules_with_kernel_slots, run_once, run_once_into, run_once_into_with_prelude,
        FuelBoundRow, PrepareError,
    };
    use babylon_bsl::causal_contract::{AuditReceipt, EffectSignature, EvidenceClass, RuleRole};
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::allocator_state::{AllocatorCursors, AllocatorState};
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::state_hash::CanonicalState;
    use babylon_graph::substrate::{GraphSubstrate, NodeId};
    use babylon_graph::working_copy::DetachedCopy;
    const SCENARIO: &str = include_str!("../content/scenarios/two-classes.bscn");
    const RULE: &str = include_str!("../content/rules/fundamental-theorem.bsl");
    const STRUGGLE_SCENARIO: &str =
        include_str!("../content/scenarios/struggle-spark-conformance.bscn");
    const STRUGGLE_RULES: &str = include_str!("../content/rules/struggle-spark.bsl");

    #[test]
    fn runtime_preparation_consumes_the_permanent_kernel_slot_ledger() {
        let mut exact_graph = HypergraphStore::new();
        prepare_rules(STRUGGLE_SCENARIO, None, STRUGGLE_RULES, &mut exact_graph)
            .expect("the bundled Struggle reservation is exact");

        for (changed_rules, expected) in [
            (
                STRUGGLE_RULES.replacen(":slot 0", ":slot 1", 1),
                "must retain slot 0",
            ),
            (
                STRUGGLE_RULES
                    .replacen("struggle/spark :slot", "struggle/renamed :slot", 1)
                    .replacen(
                        ":projects-kernel struggle/spark",
                        ":projects-kernel struggle/renamed",
                        1,
                    ),
                "must retain sample `struggle/spark`",
            ),
        ] {
            let mut graph = HypergraphStore::new();
            let before = graph.encode_state().unwrap().as_bytes().to_vec();
            let error = prepare_rules(STRUGGLE_SCENARIO, None, &changed_rules, &mut graph)
                .expect_err("a permanent reservation cannot drift");
            assert!(matches!(error, PrepareError::KernelSlot(_)), "{error:?}");
            assert!(error.to_string().contains(expected), "{error}");
            assert_eq!(graph.encode_state().unwrap().as_bytes(), before);
        }

        let mut graph = HypergraphStore::new();
        let before = graph.encode_state().unwrap().as_bytes().to_vec();
        let error = prepare_rules_with_kernel_slots(
            STRUGGLE_SCENARIO,
            None,
            STRUGGLE_RULES,
            &mut graph,
            &[],
        )
        .expect_err("a live kernel requires one permanent row");
        assert!(matches!(error, PrepareError::KernelSlot(_)), "{error:?}");
        assert!(error.to_string().contains("no permanent reservation"));
        assert_eq!(graph.encode_state().unwrap().as_bytes(), before);

        let retained = [
            super::kernel_slot::BUNDLED_KERNEL_SLOT_RESERVATIONS_V1[0],
            super::kernel_slot::KernelSlotReservationV1 {
                ordinal: 1,
                rule: "history/retired-mechanic",
                sample: "history/retired-sample",
                slot: 0,
            },
        ];
        let mut retained_graph = HypergraphStore::new();
        prepare_rules_with_kernel_slots(
            STRUGGLE_SCENARIO,
            None,
            STRUGGLE_RULES,
            &mut retained_graph,
            &retained,
        )
        .expect("a retained historical row needs no live kernel");
    }

    #[test]
    fn fired_total_refuses_usize_overflow_without_allocating_the_claimed_work() {
        let per_rule_fired = vec![("first".to_owned(), usize::MAX), ("second".to_owned(), 1)];

        assert_eq!(
            checked_fired_total(&per_rule_fired),
            Err("tick fired-subject total overflowed usize".to_owned())
        );
    }

    const ONE_SHOT_FAILURE_SCENARIO: &str = r"
(scenario tick/one-shot-atomicity
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/probability probability intensive)
  (node first NodeType/SOCIAL_CLASS (social-class/probability 0.1p))
  (node second NodeType/SOCIAL_CLASS (social-class/probability 0.9p)))
";
    const ONE_SHOT_PRELUDE: &str = "(defvocabulary NodeType (SOCIAL_CLASS))";
    const ONE_SHOT_FAILURE_SCENARIO_WITH_PRELUDE: &str = r"
(scenario tick/one-shot-atomicity
  (deffield social-class/probability probability intensive)
  (node first NodeType/SOCIAL_CLASS (social-class/probability 0.1p))
  (node second NodeType/SOCIAL_CLASS (social-class/probability 0.9p)))
";
    const ONE_SHOT_FAILURE_RULE: &str = r#"(rule vitality/one-shot-atomicity
  :role mechanic :evidence derived :material-basis "PER-18: hydration and adjudication publish as one transaction"
  :fuel 64
  (bindings (binding probability :field social-class/probability))
  (when (> probability 0.0p))
  (effects
    (emit EventType/PROBE)
    (update-node self social-class/probability (add 0.4i))))"#;

    const ONE_SHOT_SUCCESS_SCENARIO: &str = r"
(scenario tick/one-shot-success
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/count int extensive)
  (node only NodeType/SOCIAL_CLASS (social-class/count 1)))
";
    const ONE_SHOT_SUCCESS_SCENARIO_WITH_PRELUDE: &str = r"
(scenario tick/one-shot-success
  (deffield social-class/count int extensive)
  (node only NodeType/SOCIAL_CLASS (social-class/count 1)))
";
    const ONE_SHOT_SUCCESS_RULE: &str = r#"(rule vitality/one-shot-success
  :role mechanic :evidence derived :material-basis "PER-18: successful staging preserves caller-relative identity allocation"
  :fuel 32
  (bindings (binding count :field social-class/count))
  (when (= count 1))
  (effects
    (emit EventType/COMMITTED)
    (update-node self social-class/count (add 1))))"#;

    fn prepopulated_graph<G>() -> G
    where
        G: GraphSubstrate + Default,
    {
        let mut graph = G::default();
        let prior = graph.add_node("PREEXISTING").unwrap();
        graph.update_node(prior, "prior/value", 0.375).unwrap();
        graph.add_hyperedge("PRIOR_GROUP", &[prior]).unwrap();
        graph
    }

    fn assert_one_shot_failure_is_atomic<G>(with_prelude: bool)
    where
        G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
    {
        let mut graph = prepopulated_graph::<G>();
        let before = graph.encode_state().unwrap().as_bytes().to_vec();
        let cursors = graph.allocator_cursors();
        let mut sink = CollectingSink {
            events: vec![("EventType/PRIOR".to_owned(), Vec::new())],
        };
        let events = sink.events.clone();

        let result = if with_prelude {
            run_once_into_with_prelude(
                ONE_SHOT_FAILURE_SCENARIO_WITH_PRELUDE,
                ONE_SHOT_PRELUDE,
                ONE_SHOT_FAILURE_RULE,
                &mut graph,
                &mut sink,
            )
        } else {
            run_once_into(
                ONE_SHOT_FAILURE_SCENARIO,
                ONE_SHOT_FAILURE_RULE,
                &mut graph,
                &mut sink,
            )
        };

        let error = result.expect_err("the second probability write must fail");
        assert!(error.contains("E-EVAL-020"), "{error}");
        assert_eq!(graph.encode_state().unwrap().as_bytes(), before);
        assert_eq!(graph.allocator_cursors(), cursors);
        assert_eq!(sink.events, events);
    }

    fn assert_one_shot_success_preserves_relative_allocation<G>(with_prelude: bool)
    where
        G: GraphSubstrate + CanonicalState + AllocatorState + DetachedCopy + Default,
    {
        let mut graph = prepopulated_graph::<G>();
        let mut sink = CollectingSink::default();
        let report = if with_prelude {
            run_once_into_with_prelude(
                ONE_SHOT_SUCCESS_SCENARIO_WITH_PRELUDE,
                ONE_SHOT_PRELUDE,
                ONE_SHOT_SUCCESS_RULE,
                &mut graph,
                &mut sink,
            )
        } else {
            run_once_into(
                ONE_SHOT_SUCCESS_SCENARIO,
                ONE_SHOT_SUCCESS_RULE,
                &mut graph,
                &mut sink,
            )
        }
        .expect("the staged one-shot tick commits");

        assert_eq!(graph.nodes("SOCIAL_CLASS"), vec![NodeId(1)]);
        assert_eq!(
            graph
                .node_attribute(NodeId(1), "social-class/count")
                .unwrap()
                .to_bits(),
            2.0f64.to_bits()
        );
        assert_eq!(
            graph.allocator_cursors(),
            AllocatorCursors {
                next_node: 2,
                next_hyperedge: 1,
            }
        );
        assert_eq!(report.fired, 1);
        assert_eq!(sink.events.len(), 1);
        assert_eq!(sink.events[0].0, "COMMITTED");
        assert_eq!(
            report.audit_receipts,
            vec![
                AuditReceipt {
                    rule_id: "vitality/one-shot-success".to_owned(),
                    role: RuleRole::Mechanic,
                    evidence: EvidenceClass::Derived,
                    ordinal: 0,
                    effect: EffectSignature::Event("EventType/COMMITTED".to_owned()),
                },
                AuditReceipt {
                    rule_id: "vitality/one-shot-success".to_owned(),
                    role: RuleRole::Mechanic,
                    evidence: EvidenceClass::Derived,
                    ordinal: 1,
                    effect: EffectSignature::NodeField("social-class/count".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn both_one_shot_variants_roll_back_hydration_on_both_backends() {
        assert_one_shot_failure_is_atomic::<MemoryGraph>(false);
        assert_one_shot_failure_is_atomic::<MemoryGraph>(true);
        assert_one_shot_failure_is_atomic::<HypergraphStore>(false);
        assert_one_shot_failure_is_atomic::<HypergraphStore>(true);
    }

    #[test]
    fn both_one_shot_variants_keep_preexisting_allocation_on_success() {
        assert_one_shot_success_preserves_relative_allocation::<MemoryGraph>(false);
        assert_one_shot_success_preserves_relative_allocation::<MemoryGraph>(true);
        assert_one_shot_success_preserves_relative_allocation::<HypergraphStore>(false);
        assert_one_shot_success_preserves_relative_allocation::<HypergraphStore>(true);
    }

    // Task W3 (BSL Hygiene Knock-out): the fuel-bound report's smallest
    // possible fixture — one field binding, one comparison, one
    // `update-node` effect, no query/fold — deliberately shaped so its
    // computed bound is easy to hand-verify. `vitality/*` because
    // `prepare_rules`'s hardcoded `systems` set (this file, above) already
    // registers "vitality" — any other already-registered prefix would do
    // just as well (E-LOAD-002 refuses an unregistered one).
    //
    // The golden numbers below (declared=64, computed=7) were MEASURED, not
    // guessed: a throwaway probe test ran this exact fixture through
    // `prepare_rules` and printed `LoadedRule::{declared_fuel,
    // static_bound}` before this test was written (the same
    // "measured-from-the-engine, never copied/guessed" discipline this
    // repo's other conformance fixtures use) — 7 also matches
    // `bound_checker.rs`'s own pinned `demo/hunger` fixture (`Ok(7)`),
    // which has the identical AST shape (one field binding, one comparison,
    // one update-node effect) — consistent with `check_rule` never
    // consulting a cardinality ceiling for a rule with no query/fold.
    const FUEL_REPORT_FIXTURE_SCENARIO: &str = r"
(scenario probe/fuel-report
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/wealth int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wealth 100)))
";
    const FUEL_REPORT_FIXTURE_RULE: &str = r#"(rule vitality/fuel-report-fixture
  :role mechanic :evidence derived :material-basis "Task W3 fuel-bound report fixture — the simplest legal rule shape, so the report's smallest content set is trivial to hand-verify"
  :fuel 64
  (bindings (binding wealth :field social-class/wealth))
  (when (> wealth 0))
  (effects (update-node self social-class/wealth (add 1))))"#;

    #[test]
    fn fuel_bound_report_measures_declared_computed_and_headroom() {
        let rows = fuel_bound_report(FUEL_REPORT_FIXTURE_SCENARIO, None, FUEL_REPORT_FIXTURE_RULE)
            .expect("the fixture must load clean");
        assert_eq!(
            rows,
            vec![FuelBoundRow {
                rule_id: "vitality/fuel-report-fixture".to_owned(),
                declared: 64,
                computed: 7,
            }]
        );
        assert_eq!(rows[0].headroom(), 57);
    }

    // Pins the report LINE format the brief specifies verbatim: `rule-id
    // declared=<n> computed=<m> headroom=<n-m>`.
    #[test]
    fn fuel_bound_row_display_matches_the_specified_format() {
        let row = FuelBoundRow {
            rule_id: "vitality/fuel-report-fixture".to_owned(),
            declared: 64,
            computed: 7,
        };
        assert_eq!(
            row.to_string(),
            "vitality/fuel-report-fixture declared=64 computed=7 headroom=57"
        );
    }

    // The exit-code contract: non-zero ONLY when a row's computed bound
    // exceeds its declared budget. Tested directly against hand-built rows
    // rather than through the load pipeline — E-LOAD-040 (`check_rule`)
    // already refuses that condition AT LOAD, so `fuel_bound_report` itself
    // can never return such a row for content that loaded at all; this is
    // the documented redundancy the brief asks for, exercised in isolation.
    #[test]
    fn any_over_budget_is_false_when_every_row_fits_its_budget() {
        let rows = vec![
            FuelBoundRow {
                rule_id: "a".to_owned(),
                declared: 10,
                computed: 10,
            },
            FuelBoundRow {
                rule_id: "b".to_owned(),
                declared: 10,
                computed: 3,
            },
        ];
        assert!(!any_over_budget(&rows));
    }

    #[test]
    fn any_over_budget_is_true_when_one_row_exceeds_its_budget() {
        let rows = vec![FuelBoundRow {
            rule_id: "a".to_owned(),
            declared: 5,
            computed: 6,
        }];
        assert!(any_over_budget(&rows));
    }

    #[test]
    fn run_once_is_deterministic() {
        let a = run_once(SCENARIO, RULE).expect("first run");
        let b = run_once(SCENARIO, RULE).expect("second run");
        assert_eq!(a.after, b.after);
        assert_ne!(a.before, a.after, "the rule must move state");
    }

    #[test]
    fn single_rule_content_preserves_considered_and_fired_counts() {
        let report = run_once(SCENARIO, RULE).expect("single-rule run");
        assert_eq!(report.considered, 2);
        assert_eq!(report.per_rule_considered.len(), 1);
        assert_eq!(
            report.per_rule_considered[0].0,
            "economics/fundamental-theorem"
        );
        assert_eq!(report.per_rule_considered[0].1, report.considered);
        assert_eq!(report.per_rule_fired.len(), 1);
        assert_eq!(report.per_rule_fired[0].1, report.fired);
        assert!(report.considered >= report.fired);
    }

    // Task 3.3 (plan §3.4): proves `node_content_ids` reaches `PreparedRules`
    // through the REAL production wiring (`prepare_rules`), not merely
    // `LoadedScenario` in isolation (already covered, `scenario.rs`'s own
    // test module) — same class of proof as the vocabulary-wiring test
    // below.
    #[test]
    fn node_content_ids_reach_prepared_rules_through_the_real_wiring_seam() {
        let mut graph = babylon_graph::hypergraph_store::HypergraphStore::new();
        let prepared =
            prepare_rules(SCENARIO, None, RULE, &mut graph).expect("two-classes.bscn loads");
        assert_eq!(
            prepared
                .node_content_ids
                .get(&babylon_graph::substrate::NodeId(0)),
            Some(&"core".to_owned())
        );
        assert_eq!(
            prepared
                .node_content_ids
                .get(&babylon_graph::substrate::NodeId(1)),
            Some(&"periphery".to_owned())
        );
        assert_eq!(prepared.scenario_scope, "ft/two-classes");
        assert!(prepared.hyperedge_content_ids.is_empty());
    }

    // F3 (#534 fix round item 3, panel-proven): `prepare_rules`'s ONE
    // production wiring line — `vocabulary_registry:
    // scenario.vocabulary.as_ref()` — was unpinned; mutating it to `None`
    // flipped zero tests. This drives `run_once`, the ACTUAL production
    // seam (the CLI driver and `babylon-client`'s engine link both call
    // exactly this function), with a scenario declaring a vocabulary and a
    // rule whose enum-ref typos a member of a DECLARED kind — proving the
    // registry really is threaded end to end through `prepare_rules`, not
    // merely unit-tested in isolation at each of the three producers.
    const VOCAB_WIRING_SCENARIO: &str = r"
(scenario ft/vocab-wiring-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (deffield social-class/wages int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 100)))
";
    const VOCAB_WIRING_RULE: &str = r#"(rule vitality/vocab-wiring-probe
  :role mechanic :evidence derived :material-basis "F3 (#534 fix round item 3): proves the production seam threads a declared vocabulary end to end"
  :fuel 64
  (domain NodeType/SOCIAL_CLA)
  (bindings (binding wages :field social-class/wages))
  (when (> wages 0))
  (effects (emit EventType/PROBE)))"#;

    #[test]
    fn a_declared_vocabulary_typo_refuses_through_the_production_seam() {
        let err = run_once(VOCAB_WIRING_SCENARIO, VOCAB_WIRING_RULE).unwrap_err();
        assert!(err.contains("E-LOAD-031"), "{err}");
    }

    // T2 (issue #559) PLAN-MUST-VERIFY probe: proves the D32 implicit-strength seed reaches
    // typecheck_aggregation through prepare_rules's REAL production wiring, using only
    // already-served slice-1 infrastructure (neighbors) — provable BEFORE edges/field-of-over-
    // EdgeRef land in PR B (Tasks 3-5). The rule LOADS today only if the wiring works; it would
    // still refuse E-EVAL-033 if actually RUN (its fold body reads a NodeRef's field-of an
    // edge-owned qname, a referent-type mismatch) — this probe tests LOADING only, deliberately.
    const D32_WIRING_PROBE_SCENARIO: &str = r"
(scenario ft/d32-wiring-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (deffield social-class/shape int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 1))
";
    const D32_WIRING_PROBE_RULE: &str = r#"(rule vitality/d32-wiring-probe
  :role mechanic :evidence derived :material-basis "PLAN-MUST-VERIFY probe (T2, issue #559): the D32 implicit-strength field must resolve through prepare_rules's real TypeEnv construction, not merely in isolation (r9_chapters.rs::type_env already proves the isolated chain)"
  :fuel 128
  (bindings (binding shape :field social-class/shape))
  (when (= shape 1))
  (effects (emit EventType/PROBE
    (s (fold sum (neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS)
             (field-of it solidarity/strength))))))"#;

    #[test]
    fn the_d32_implicit_strength_field_resolves_through_the_real_wiring_seam() {
        let mut graph = babylon_graph::hypergraph_store::HypergraphStore::new();
        let result = prepare_rules(
            D32_WIRING_PROBE_SCENARIO,
            None,
            D32_WIRING_PROBE_RULE,
            &mut graph,
        );
        assert!(
            result.is_ok(),
            "the D32 implicit-strength field must resolve through prepare_rules's real \
             TypeEnv construction: {result:?}"
        );
    }

    // Task 2 Step 4: proves the E-LOAD-001 half, not just the happy path — a rule-irrelevant
    // EXPLICIT `deffield` re-declaring the implicit `<edge-type>/strength` field must refuse,
    // never silently override. Zero-regression check (verified, not assumed):
    // `rg -n "strength" rust/crates/babylon-tick/content/scenarios/*.bscn` returns no hits today
    // — no committed scenario explicitly declares an edge-owned `strength` field, so this new
    // refusal cannot regress any existing content.
    const D32_WIRING_PROBE_SCENARIO_WITH_EXPLICIT_REDECLARATION: &str = r"
(scenario ft/d32-wiring-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (deffield social-class/shape int extensive)
  (deffield solidarity/strength int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 1))
";

    #[test]
    fn an_explicit_deffield_redeclaring_the_implicit_strength_field_is_e_load_001() {
        let mut graph = babylon_graph::hypergraph_store::HypergraphStore::new();
        let err = prepare_rules(
            D32_WIRING_PROBE_SCENARIO_WITH_EXPLICIT_REDECLARATION,
            None,
            D32_WIRING_PROBE_RULE,
            &mut graph,
        )
        .unwrap_err();
        assert!(err.to_string().contains("E-LOAD-001"), "{err}");
        // #652 Task 3, plan §3.1's fourth row ("the row revision 1
        // missed"): the code must survive as STRUCTURED DATA, not merely as
        // a substring of the formatted message — revision 1's four-variant
        // `PrepareError` design would have passed the `.contains` assertion
        // above (the hand-rolled string still says "E-LOAD-001") while
        // silently returning `None` here, because `Composition` had no
        // `code`/`identity` fields at all.
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    // Adversarial-verifier fix round, Fix 2: with TWO explicit re-declarations colliding against
    // the implicit seed set, the pre-fix loop iterated `type_env_fields()`'s HashMap directly and
    // returned on the first hit — so the refusal nondeterministically named either
    // `solidarity/strength` or `tenancy/strength` across runs (proven empirically over 64 runs).
    // Post-fix the seed pairs are byte-sorted before the collision check, so the refusal always
    // names the byte-least colliding qname. Pinned as an EXACT full-string assertion, looped over
    // fresh `prepare_rules` calls (each loop builds fresh HashMaps with fresh RandomState keys, so
    // a regression to unsorted iteration gets many chances to surface in one test run).
    const D32_TWO_COLLISION_SCENARIO: &str = r"
(scenario ft/d32-two-collision-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY TENANCY))
  (deffield social-class/shape int extensive)
  (deffield solidarity/strength int extensive)
  (deffield tenancy/strength int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/shape 1))
  (node other NodeType/SOCIAL_CLASS (social-class/shape 1))
  (edge EdgeType/SOLIDARITY core other 1))
";

    #[test]
    fn a_two_collision_e_load_001_refusal_always_names_the_byte_least_field() {
        for _ in 0..20 {
            let mut graph = babylon_graph::hypergraph_store::HypergraphStore::new();
            let err = prepare_rules(
                D32_TWO_COLLISION_SCENARIO,
                None,
                D32_WIRING_PROBE_RULE,
                &mut graph,
            )
            .unwrap_err();
            assert_eq!(
                err.to_string(),
                "E-LOAD-001: solidarity/strength is the implicit <edge-type>/strength field \
                 (D32) — re-declaring it with an explicit deffield is a duplicate declaration, \
                 never a silent override (bsl-language.rst §2.9)"
            );
            assert_eq!(err.spec_code(), Some("E-LOAD-001"));
        }
    }

    // G3(b) (#534 fix round 2): the "Task-10 detonation pin". The
    // organization foundation plan's own scenario shape
    // (docs/superpowers/plans/2026-08-11-organization-foundation-plan.md
    // Task 10, ~lines 443-480) declares NodeType/EdgeType vocabularies but
    // NEVER EventType, then its probe rule emits
    // `EventType/ORGANIZATION_SEEDED` from inside an emit form carrying a
    // payload item (`(probe 1)`) — the exact shape G1's tightened
    // `check_enum_ref_membership`/`find_deferred_shape_verb` now recurse
    // one level deeper into (each payload item's VALUE, not just its
    // LABEL). This proves the plan's own real-world content still loads
    // clean and fires through `run_once`, the actual production seam
    // (scenario hydration's F1 inertness for an opted-out EventType kind,
    // AND emit's own operand plus its payload value, all through one
    // pipeline) — not merely each walker's own isolated unit tests.
    const TASK_10_SCENARIO: &str = r"
(scenario organization/foundation-detonation-pin
  (defvocabulary NodeType (SOCIAL_CLASS ORGANIZATION))
  (defvocabulary EdgeType (MEMBERSHIP))
  (deffield organization/active int extensive)
  (node worker NodeType/SOCIAL_CLASS)
  (node cell NodeType/ORGANIZATION (organization/active 1))
  (edge EdgeType/MEMBERSHIP cell worker 1))
";
    const TASK_10_RULE: &str = r#"(rule organization/kind-probe
  :role mechanic :evidence derived :material-basis "Task 10's own probe rule shape (organization foundation plan) — EventType stays undeclared while NodeType/EdgeType are declared, proving hydration and the emit payload both leave an opted-out kind inert through the full production seam"
  :fuel 32
  (bindings (binding active :field organization/active))
  (when (= active 1))
  (effects (emit EventType/ORGANIZATION_SEEDED (probe 1))))"#;

    #[test]
    fn the_task_10_scenario_shape_loads_clean_and_fires_through_run_once() {
        let report = run_once(TASK_10_SCENARIO, TASK_10_RULE).expect(
            "EventType was never declared — its checking must stay inert, at both \
             hydration and the emit payload G1 now recurses into",
        );
        assert_eq!(
            report.fired, 1,
            "the probe rule must fire for exactly the one active organization"
        );
    }
}
