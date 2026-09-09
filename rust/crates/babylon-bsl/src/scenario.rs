//! The world-load path: a scenario file becomes a populated
//! [`GraphSubstrate`] (P27 Phase 2 Slice 1).
//!
//! Before this module the Rust engine had no way to obtain a world at all —
//! every graph in the tree was built by hand in a test. A rule cannot run
//! against a world that does not exist, so this is the first half of the
//! vertical slice; the tick loop is the second.
//!
//! # Why s-expressions and not a new format
//!
//! The reader, its error codes, and its NFC/escape discipline already exist
//! ([`crate::reader`]). A scenario is content, and the project already has a
//! content language — inventing a second one would mean a second lexer, a
//! second error vocabulary, and a second place for encoding bugs to live.
//!
//! ```text
//! (scenario ft/two-classes
//!   (node core NodeType/SOCIAL_CLASS
//!     (social-class/wages 120)
//!     (social-class/value-produced 80))
//!   (node periphery NodeType/SOCIAL_CLASS
//!     (social-class/wages 20)
//!     (social-class/value-produced 90))
//!   (edge EdgeType/SOLIDARITY core periphery 1))
//! ```
//!
//! **Local names resolve edges at load time; [`LoadedScenario::node_content_ids`]
//! retains them as content identity.** `core` and `periphery` let an edge
//! name its endpoints, resolved to [`NodeId`]s during the load — the
//! substrate itself still knows nodes only by that opaque handle. But a
//! handle is hydration-order-dependent (inserting a node earlier in the file
//! shifts every later one), which is unusable as a stable identity for
//! anything computed FROM the scenario's content rather than its insertion
//! order — the private finite-kernel draw carrier chief among them. So the
//! loader retains the inverse of its load-time local-name table on
//! [`LoadedScenario`], keyed by the [`NodeId`] each name resolved to. This is
//! content identity, not the substrate's — `babylon-graph` gains no stable-id
//! accessor, and canonical state (`state_hash`) is untouched.
//!
//! **Declaration order is the id order.** Nodes are minted top to bottom, so
//! the same file always produces the same [`NodeId`] assignment and hence the
//! same state hash. Reordering two `node` forms is a real change to the
//! scenario, not a cosmetic one — which is honest, since it changes what
//! `NodeId(0)` denotes.
//!
//! # What this deliberately does not do
//!
//! - **`currency`-declared node attributes now seed** (T3 #491, OQ-J — Half
//!   2 of the typed-attribute-seeding design,
//!   `reports/typed-attribute-seeding-design-2026-08-11.md`, landed on
//!   Currency's first real consumer per the Director's 2026-08-11 popup
//!   ruling). A `$`-suffixed literal into a `currency`-declared field routes
//!   through [`babylon_graph::substrate::GraphSubstrate::update_node_currency`]
//!   — the i128 lane, a map PARALLEL to the ordinary `f64` attribute lane
//!   this module otherwise writes through, never the same map. Everywhere
//!   ELSE a `$` literal is refused exactly as before: `:default`, a
//!   non-`currency`-declared field, an edge attribute (edge-scoped Currency
//!   storage is out of scope for this train) — the refusal message now
//!   names the actual reason (wrong declared type, or no edge lane), not
//!   "no Currency storage exists at all".
//! - **`int`- and fractional-typed attributes only (Half 1).** An
//!   `int`-declared field takes an integer literal, exact in `f64` to 2^53.
//!   A `probability`/`intensity`/`coefficient`-declared field takes any
//!   literal that widens to `[0, 1]` — see `attribute_value` (private, this
//!   module) for the conversion contract and its determinism argument.
//! - **Hyperedges: minting only, no attributes yet** (2026-08-18, hyperedge-lane
//!   Task 1, `docs/superpowers/plans/2026-08-18-community-port.md`). A
//!   `(hyperedge <local-name> <HyperedgeType/MEMBER> (members <local-name>+))`
//!   form mints one hyperedge through [`GraphSubstrate::add_hyperedge`],
//!   resolving each member through the same local-name table `node`/`edge`
//!   share — see [`LoadedScenario::hyperedge_types`]/
//!   [`LoadedScenario::max_members_seen`] for the population maps it feeds.
//!   Two gaps remain, each a later task's: initial hyperedge fields
//!   (`hyperedge-attr`, Task 6) and per-membership payload (Amendment AG,
//!   issue #653's own ceremony) — both still refuse loudly rather than
//!   silently dropping a write. **The doctrine sentence this paragraph
//!   replaces survives**: an unused form is an untested form — it is why
//!   `metric-of` still does not land either.
//! - **No defaults.** A node with no attributes gets no attributes. An
//!   unwritten field errors on read (III.11), and seeding zeros here would
//!   defeat that at the one place it is easiest to defeat.

// `ScenarioError` grew past clippy's 128-byte `result_large_err` threshold
// once `identity: Option<ErrorIdentity>` (§2.3, issue #652 Task 2) joined
// `message`/`code`/`position` — `ErrorIdentity::Edge`'s three `String`s are
// the largest variant. The load path is cold (a scenario either loads once
// or the whole run fails), so paying the extra stack bytes on every
// `Result<_, ScenarioError>` is the right trade against boxing a field
// §2.3 specifies unboxed, or `Box`-wrapping every one of this module's
// ~20 pre-existing fallible signatures for a rarely-taken error branch.
#![allow(clippy::result_large_err)]

use crate::error_identity::{decl_identity, vocabulary_identity, ErrorIdentity};
use crate::evaluator::Value;
use crate::probability::MassDeclarationAnalysisV1;
use crate::reader::{
    read_all, Atom, FormPath, LexCode, ReadError, ReadErrorKind, SExpr, ScaledKind,
};
use crate::types::{BslType, EnumRegistry, EnumTypeId, FieldDecl, FieldKind};
use crate::vocabulary::{ClosedVocabulary, EnumKind, VocabularyError};
use babylon_graph::substrate::{GraphError, GraphSubstrate, HyperedgeId, NodeId};
use babylon_kernel::{Currency, Ratio};
use babylon_practice_contract::{
    PracticeContractError, PracticeTargetDomainV1, PracticeTopologyLoadCounter,
};
use std::collections::{HashMap, HashSet};

const MAX_SCENARIO_SOURCE_BYTES: usize = 4_194_304;
const MAX_SCENARIO_BODY_FORMS: usize = 65_536;
const MAX_SCENARIO_AST_NODES: usize = 1_048_576;
const MAX_SCENARIO_WALKER_DEPTH: usize = 256;
const MAX_SCENARIO_WALKER_STACK: usize = 65_536;
const MAX_SCENARIO_LOCAL_NAMES: usize = 65_536;
const UNNAMED_SCENARIO_SOURCE_ID: &str = "<scenario>";
const UNNAMED_PRELUDE_SOURCE_ID: &str = "<prelude>";

/// Why a scenario would not load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError {
    /// Human-readable detail, naming the offending form.
    pub message: String,
    /// The spec's error code, where §3.9 names one for a hydration
    /// failure. `None` where it does not — no invented codes.
    pub code: Option<&'static str>,
    /// The byte offset the reader detected the failure at (`E-LEX` only —
    /// `From<ReadError>` is the sole producer; §2.3, issue #652 Task 2).
    pub position: Option<usize>,
    /// WHAT the failure is about, as data a locator can find in a parsed
    /// tree — never derived from `message`'s own text (§2.3, issue #652
    /// Task 2). `None` where the underlying error is prose-only or carries
    /// no typed error at all.
    pub identity: Option<ErrorIdentity>,
}

impl ScenarioError {
    /// Attach identity after construction, for the one call site
    /// (`load_defconst`'s duplicate check) whose typed context is a local
    /// variable rather than a wrapped typed error `err()`'s own signature
    /// could delegate to. Not a `ScenarioError { .. }` construction site —
    /// the seven the compiler forces stay exactly seven.
    #[must_use]
    fn with_identity(mut self, identity: ErrorIdentity) -> Self {
        self.identity = Some(identity);
        self
    }
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(code) => write!(f, "{code}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ScenarioError {}

impl From<ReadError> for ScenarioError {
    fn from(err: ReadError) -> Self {
        // §6.2's own precision table: E-LEX is located via this position,
        // never via `ErrorIdentity` — `identity` stays `None`. The code was
        // discarded before Task 2 even for a genuine `E-LEX` failure; only
        // `ReadErrorKind::Lex` carries one (structural read failures like
        // an unterminated list have none, correctly).
        let code = match err.kind {
            ReadErrorKind::Lex(lex_code) => Some(lex_code.spec_code()),
            _ => None,
        };
        Self {
            message: format!(
                "scenario read failed at byte {}: {}",
                err.position, err.message
            ),
            code,
            position: Some(err.position),
            identity: None,
        }
    }
}

impl From<GraphError> for ScenarioError {
    fn from(err: GraphError) -> Self {
        // `GraphError` is `{ message: String }` (`babylon-graph`'s
        // `substrate.rs`) — no typed field beyond the message it already
        // formats from, so there is nothing to populate here.
        Self {
            message: format!("substrate refused the scenario: {}", err.message),
            code: None,
            position: None,
            identity: None,
        }
    }
}

/// Task 8 (Organization foundation plan): a closed-vocabulary failure
/// (`load_defvocabulary`'s own build, or `load_node`/`load_edge`'s
/// membership check below) carries the same code/message shape §3.6
/// already uses.
impl From<VocabularyError> for ScenarioError {
    fn from(err: VocabularyError) -> Self {
        Self {
            code: Some(err.spec_code()),
            identity: Some(vocabulary_identity(&err)),
            message: err.to_string(),
            position: None,
        }
    }
}

fn err(message: impl Into<String>) -> ScenarioError {
    // A bare structural/prose message, no typed error behind it — nothing
    // to derive `position`/`identity` from. Callers with local typed
    // context (e.g. `load_defconst`'s duplicate check) attach identity
    // afterward via `ScenarioError::with_identity`.
    ScenarioError {
        message: message.into(),
        code: None,
        position: None,
        identity: None,
    }
}

/// F2 (#534 fix round item 2), corrected by G2 (#534 fix round 2 item 2):
/// demand that a `.bscn` node/edge form's `<enum-ref>` type-operand names
/// the POSITION's own kind — a static fact about the WRITTEN type name,
/// independent of whether any `defvocabulary` was declared for it at all
/// (mirrors `grammar::check_enum_ref_kinds`'s D74 class rule for rules,
/// which is also unconditional — it runs whether or not
/// `ctx.vocabulary_registry` is `Some`; authority: §3.9 clause 1,
/// "hydration is not a back door into the closed vocabulary"). Without
/// this, `load_node`/`load_edge` called `ClosedVocabulary::check_enum_ref`
/// and discarded its returned [`EnumKind`], never comparing it against the
/// position it came from, so e.g. `(node x EdgeType/SOLIDARITY)` minted a
/// node silently typed SOLIDARITY under a declared vocabulary that
/// registers `EdgeType/SOLIDARITY` (panel-proven, mutation-reproduced:
/// hardcoding "`NodeType`" at the call site flipped zero tests before
/// this).
///
/// **G2's own correction.** §2.6's class rule (bsl-language.rst:972-974)
/// splits two facts F2's original landing conflated under one code and one
/// message: whether the WRITTEN kind is the position's own kind
/// (`E-TYPE-011`) is independent of whether the type/member exist AT ALL
/// (`E-LOAD-030`/`E-LOAD-031`). `enum_type` naming a REAL type — one of
/// the four structural kinds, or a type this scenario declared via
/// `defenum` (`enums`) — that simply is not the kind this position demands
/// is `E-TYPE-011`; `enum_type` naming nothing real anywhere is the
/// genuine `E-LOAD-030` case (bsl-language.rst D119). This split is
/// POSITIONAL, not vocabulary-gated: it holds whether or not the WRITTEN
/// kind was itself `defvocabulary`-declared in this scenario, and whether
/// or not any `defvocabulary` form appears in it at all — the SEPARATE
/// membership check a threaded `ClosedVocabulary` performs (below, in
/// `load_node`/`load_edge`) is the only opt-in half.
///
/// **H2 (#534 fix round 3): the split is ALSO registry-relative, not just
/// positional.** `enums` is `vocabulary_so_far`'s sibling — the running
/// `defenum` registry as it stands at THIS point in the top-to-bottom
/// load — so a scenario-declared type participates in "is this a REAL
/// type" only from its OWN declaration point down, the same
/// "declaration must precede use" discipline `deffield`/`defconst`/
/// `defvocabulary` already enforce. `(defenum OrgKind (BUSINESS)) (node x
/// OrgKind/BUSINESS)` is `E-TYPE-011` (`OrgKind` genuinely exists by the
/// time the node form runs); `(node x OrgKind/BUSINESS) (defenum OrgKind
/// (BUSINESS))` is `E-LOAD-030` (`OrgKind` genuinely names nothing YET at
/// that point in the load) — not a bug, the same ordering sensitivity
/// every other declared-registry lookup in this loader already has.
///
/// # Errors
///
/// [`VocabularyError::WrongEnumKind`] (`E-TYPE-011`) for a real type at
/// the wrong position; [`VocabularyError::UnknownEnumType`] (`E-LOAD-030`)
/// for a type name that is not registered anywhere at all.
fn demand_enum_kind(
    enum_type: &str,
    member: &str,
    demanded: EnumKind,
    enums: &EnumRegistry,
) -> Result<(), VocabularyError> {
    if EnumKind::from_type_name(enum_type) == Some(demanded) {
        return Ok(());
    }
    let is_real_type =
        EnumKind::from_type_name(enum_type).is_some() || enums.resolve(enum_type).is_some();
    if is_real_type {
        return Err(VocabularyError::WrongEnumKind {
            enum_type: enum_type.to_owned(),
            member: member.to_owned(),
            expected: demanded,
        });
    }
    Err(VocabularyError::UnknownEnumType {
        enum_type: enum_type.to_owned(),
        member: member.to_owned(),
    })
}

/// F6 (#534 fix round item 6): wrap a [`VocabularyError`] with the
/// offending form's identity — §4.6's own house style ("load-time errors
/// report the offending file, line, column, form, and code"). A raw
/// `VocabularyError` names the type/member but not WHICH node or edge
/// form wrote it; this prefixes that, at the two scenario-hydration
/// producers (`load_node`/`load_edge`).
fn vocab_err(form: impl std::fmt::Display, error: &VocabularyError) -> ScenarioError {
    ScenarioError {
        code: Some(error.spec_code()),
        identity: Some(vocabulary_identity(error)),
        message: format!("{form}: {error}"),
        position: None,
    }
}

/// A hydration failure the reference gives a code (§3.9). Takes a bare
/// code + message, not a typed error — nothing structured behind either
/// argument to populate `identity` from (same footing as `err()`).
fn coded_err(code: &'static str, message: impl Into<String>) -> ScenarioError {
    ScenarioError {
        message: message.into(),
        code: Some(code),
        position: None,
        identity: None,
    }
}

/// What a loaded scenario declared, beyond the graph itself.
#[derive(Debug, Clone)]
pub struct LoadedScenario {
    /// The scenario's qualified id, for the run record.
    pub id: String,
    /// How many nodes it minted.
    pub node_count: usize,
    /// How many dyadic edges it minted.
    pub edge_count: usize,
    /// How many nodes of each `NodeType` member the scenario minted.
    ///
    /// The load-time bound checker needs a cardinality ceiling per queried
    /// type; taking it from the population the scenario ACTUALLY built means
    /// the static bound is checked against a real number rather than an
    /// invented one.
    pub node_types: HashMap<String, u64>,
    /// How many dyadic edges of each `EdgeType` member the scenario minted.
    ///
    /// The SAME argument as `node_types`, one axis over: `neighbors_ceiling`
    /// (`bound_checker.rs`) bounds a `(neighbors …)` fold against the
    /// **lesser** of the queried edge type's ceiling and the annotated
    /// result `NodeType`'s — so a rule using `neighbors` needs an
    /// `EdgeType/…` entry in `CardinalityCeilings` too, or `check_rule`
    /// raises `MissingCeiling` for the edge axis specifically. Added by the
    /// query-evaluation plan's Task 15 (P27 Phase 2 PR 5) — the FIRST
    /// consumer of `neighbors` through the scenario-driven `run_once_into`
    /// path (every rule pack landed before it read only `:field`s, never a
    /// query), so this gap was latent, not exercised, until now.
    pub edge_types: HashMap<String, u64>,
    /// How many hyperedges of each `HyperedgeType` member the scenario
    /// minted.
    ///
    /// The SAME argument as `node_types`/`edge_types`, a third axis over:
    /// "taking it from the population the scenario ACTUALLY built means the
    /// static bound is checked against a real number rather than an invented
    /// one" (`:242-248`'s own words, quoted verbatim) — Task 4's
    /// `HyperedgeType/*` ceiling entries are built from this map (hyperedge-
    /// lane Task 1, `docs/superpowers/plans/2026-08-18-community-port.md`).
    pub hyperedge_types: HashMap<String, u64>,
    /// The longest member list seeded for each `HyperedgeType` member.
    ///
    /// Task 4's `max_members` ceiling axis (`CardinalityCeilings::new`'s
    /// second argument, `fuel.rs`) is bound to `members-of`'s fold the same
    /// way `edge_types` bounds `neighbors`' — from what the scenario
    /// ACTUALLY seeded, never a manifest-declared constant independent of
    /// the world (hyperedge-lane Task 1, same plan).
    pub max_members_seen: HashMap<String, u64>,
    /// The fields the scenario DECLARED, keyed by qname.
    ///
    /// This is the `deffield` registry in miniature: a rule's typechecker
    /// and binding vocabulary both need to know a field's type and
    /// intensivity kind, and until Phase 2's content registries exist the
    /// scenario is the only place that knowledge can honestly come from.
    /// Deriving it from usage instead would mean guessing a kind, and
    /// intensivity is exactly what §3.4 refuses to guess.
    pub fields: HashMap<String, FieldDecl>,
    /// The DEFINES ENVIRONMENT a `:const` binding reads (§2.5, §4.2),
    /// keyed by qualified name.
    ///
    /// The same registry-in-miniature argument as `fields`, one level up: a
    /// coefficient's *value* is content, and until Phase 2's registry reads
    /// `GameDefines`/`defines.yaml` there is nowhere else it can honestly
    /// come from. Writing the number into the rule instead would put a
    /// magnitude in the shape's file, which is the one thing the project's
    /// coefficient discipline forbids — so the scenario declares it and
    /// cites the `defines.yaml` line it was taken from.
    pub consts: HashMap<String, Value>,
    /// The exact qualified names in `consts` whose declaration used a
    /// `p`-suffixed Probability literal.
    ///
    /// Scaled `p`, `i`, and `c` literals all evaluate through
    /// [`Value::Real`], so their numeric carrier cannot recover this authored
    /// type identity later. The loader retains it at the declaration
    /// boundary instead; values never determine types by inference.
    pub probability_consts: HashSet<String>,
    /// Loader-confirmed exact Mass constants with source-local parser paths.
    ///
    /// The named multi-source load path retains these facts during the same
    /// semantic pass that admits each `defconst`; authoring tools therefore
    /// never reparse scenario or prelude source to rediscover them. Facts are
    /// ordered scenario-first, followed by preludes in caller order.
    pub mass_declarations: Vec<MassDeclarationAnalysisV1>,
    /// **§2.13 addendum (D101, Organization spec §1 Q12).** Every
    /// `defenum` type the scenario declared — the registry `enum`-typed
    /// `deffield`s resolve against, and the read path (`tick.rs::
    /// bind_subject`) renders stored ordinals back through. Empty for a
    /// scenario with no `defenum` forms — unlike `fields`/`consts`, there
    /// is no "the scenario is the only registry" claim here: `EnumRegistry`
    /// is §2.13's own new construct, not a stand-in for a Phase-2 content
    /// registry that predates it.
    pub enums: EnumRegistry,
    /// **§2.13 addendum (D101), §3.6.** The closed graph vocabulary this
    /// scenario declared via `defvocabulary`, or `None` for a scenario
    /// declaring none — opt-in per scenario, so every EXISTING content set
    /// (which declares no `defvocabulary` at all) is unaffected (Task 7's
    /// own backward-compatibility proof). Enforcement against this
    /// registry (Task 8 of the Organization foundation plan) is out of
    /// this train's scope; this field only carries what was declared.
    pub vocabulary: Option<ClosedVocabulary>,
    /// **Content-stable node identity (plan §3.4, this train's Task 3).**
    /// The inverse of the load-time `local name -> NodeId` table, retained
    /// rather than discarded. A [`NodeId`] is an opaque handle minted in
    /// insertion order — it moves if a node is added earlier in the file —
    /// so it cannot serve as a stable key for anything that must be
    /// insertion-order-independent (the grain-invariance guard this train's
    /// tests exercise). The scenario-declared local name IS stable under
    /// that axis: it names WHAT the node is, not WHERE it was minted.
    /// Built once, at the end of `load_scenario_inner`, by inverting
    /// `named` — never touches `babylon-graph` or canonical state
    /// (`state_hash` is computed over the substrate alone and does not see
    /// this field).
    pub node_content_ids: HashMap<NodeId, String>,
    /// Content-stable authored hyperedge identities, inverted at hydration.
    /// Runtime handles, types, and member sets never substitute for names.
    pub hyperedge_content_ids: HashMap<HyperedgeId, String>,
}

/// One caller-named declaration prelude for a multi-source scenario load.
///
/// `source_id` is authoring identity only. It does not enter graph state or
/// mechanics, and exists so retained [`FormPath`] values resolve against the
/// correct source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedDeclarationPreludeV1<'a> {
    /// Caller-owned stable source identity.
    pub source_id: &'a str,
    /// Exact decoded source text.
    pub source: &'a str,
}

/// The registries a **prelude** may pre-seed (§2.13 addendum, Train B item
/// 4, issue #591, D157) before a scenario loads against them.
/// [`load_scenario`] passes [`Self::default`] (no prelude: every field
/// starts empty, exactly as `load_scenario_inner`'s own locals did before
/// this extraction); [`load_scenario_with_prelude`] passes [`load_prelude`]'s
/// return value instead.
///
/// Bundled into one struct rather than six parameters: `load_scenario_inner`
/// already carries `source` and `graph`, and six more positional arguments
/// would trip `clippy::too_many_arguments` for no gain over one named group.
/// The tally/dedup locals (`named`, `node_types`, `edge_types`,
/// `node_count`, `edge_count`, `seeded_edges`, `seeded_attrs`) do NOT travel
/// here — a prelude never touches the graph, so none of them has a
/// meaningful prelude-time value.
#[derive(Default)]
struct PreludeRegistries {
    /// The `deffield` registry in miniature — §2.13/§3.4's declared
    /// type+intensivity-kind pair per qname, read by `load_node`/
    /// `load_edge`/`load_edge_attr`'s field-init paths.
    fields: HashMap<String, FieldDecl>,
    /// The DEFINES ENVIRONMENT a `:const` binding reads (§2.5, §4.2).
    consts: HashMap<String, Value>,
    /// Exact qnames of `consts` declared with a Probability literal. This
    /// travels beside `consts` so prelude declarations keep their authored
    /// type identity when the scenario load continues.
    probability_consts: HashSet<String>,
    /// Mass declarations admitted while loading named preludes, retained in
    /// caller source order until the scenario load publishes them after its
    /// own declarations.
    mass_declarations: Vec<MassDeclarationAnalysisV1>,
    /// §2.13 (D101): every `defenum` type declared so far, top to bottom —
    /// a `deffield ... enum <Type>` resolves against this AS IT IS AT THAT
    /// POINT, the same "declaration must precede use" discipline
    /// `fields`/`consts` already enforce.
    enums: EnumRegistry,
    /// §2.13/§3.6: the closed graph vocabulary declared so far, per kind —
    /// collected and fed to `ClosedVocabulary::new` after EVERY
    /// `defvocabulary` form, since that constructor runs the
    /// whole-vocabulary rendering-disjointness check (`E-LOAD-032`) over
    /// every kind at once.
    vocabulary_members: HashMap<EnumKind, Vec<String>>,
    /// The "one form per kind" guard (`E-LOAD-001`) — `ClosedVocabulary::
    /// new` itself MERGES same-kind entries via `.extend` rather than
    /// rejecting a second one, so this check has to live here, before the
    /// merge ever happens.
    vocabulary_kinds_declared: HashSet<EnumKind>,
    /// Rebuilt after EVERY `defvocabulary` form, so `load_node`/`load_edge`
    /// can check membership BEFORE minting. `None` until the first
    /// `defvocabulary` form (Task 7's backward-compatibility proof), and —
    /// by construction — equals exactly `ClosedVocabulary::
    /// new(vocabulary_members)` once loading ends, so it doubles as the
    /// FINAL `LoadedScenario.vocabulary` value with no separate
    /// end-of-load construction needed.
    vocabulary_so_far: Option<ClosedVocabulary>,
}

struct AstWalkFrame<'a> {
    nodes: &'a [SExpr],
    next: usize,
    depth: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PreflightNodeKind {
    Organization,
    SocialClass,
    Other,
}

#[derive(Clone, Copy)]
struct PreflightNode {
    ordinal: u64,
    kind: PreflightNodeKind,
}

fn bounded_scenario_read(source: &str) -> Result<Vec<SExpr>, ScenarioError> {
    if source.len() > MAX_SCENARIO_SOURCE_BYTES {
        return Err(err(format!(
            "scenario source exceeds the 4,194,304-byte load bound: {} bytes",
            source.len()
        )));
    }
    Ok(read_all(source.as_bytes())?)
}

fn check_scenario_ast_bounds(forms: &[SExpr]) -> Result<(), ScenarioError> {
    let mut stack = vec![AstWalkFrame {
        nodes: forms,
        next: 0,
        depth: 0,
    }];
    let mut ast_nodes = 0_usize;
    for _step in 0..=(MAX_SCENARIO_AST_NODES * 2 + 2) {
        let Some(frame) = stack.last_mut() else {
            return Ok(());
        };
        if frame.next == frame.nodes.len() {
            stack.pop();
            continue;
        }
        let node = &frame.nodes[frame.next];
        frame.next += 1;
        let depth = frame.depth + 1;
        if depth > MAX_SCENARIO_WALKER_DEPTH {
            return Err(err("scenario AST exceeds the walker depth bound of 256"));
        }
        ast_nodes += 1;
        if ast_nodes > MAX_SCENARIO_AST_NODES {
            return Err(err("scenario AST exceeds the 1,048,576-node load bound"));
        }
        if let SExpr::List(children) = node {
            if depth > 1 && children.len() > MAX_SCENARIO_WALKER_STACK {
                return Err(err(
                    "scenario AST exceeds the 65,536-entry walker stack bound",
                ));
            }
            if stack.len() == MAX_SCENARIO_WALKER_STACK {
                return Err(err(
                    "scenario AST exceeds the 65,536-entry walker stack bound",
                ));
            }
            stack.push(AstWalkFrame {
                nodes: children,
                next: 0,
                depth,
            });
        }
    }
    Err(err(
        "scenario AST walker exhausted its fixed iteration bound",
    ))
}

fn practice_registry_trigger(body: &[SExpr], prelude_fields: &HashMap<String, FieldDecl>) -> bool {
    if prelude_fields.contains_key("organization/action-budget") {
        return true;
    }
    for form in body.iter().take(MAX_SCENARIO_BODY_FORMS + 1) {
        let SExpr::List(parts) = form else {
            continue;
        };
        let [SExpr::Atom(Atom::Symbol(tag)), SExpr::Atom(Atom::QName(qname)), ..] =
            parts.as_slice()
        else {
            continue;
        };
        if tag == "deffield" && qname == "organization/action-budget" {
            return true;
        }
    }
    false
}

fn final_practice_fields(
    body: &[SExpr],
    prelude_fields: &HashMap<String, FieldDecl>,
    prelude_enums: &EnumRegistry,
) -> Result<HashMap<String, FieldDecl>, ScenarioError> {
    let mut fields = prelude_fields.clone();
    let mut enums = prelude_enums.clone();
    for form in body.iter().take(MAX_SCENARIO_BODY_FORMS + 1) {
        let SExpr::List(parts) = form else {
            continue;
        };
        match parts.first() {
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defenum" => {
                load_defenum(form, &mut enums)?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "deffield" => {
                load_deffield(parts, &mut fields, &enums)?;
            }
            _ => {}
        }
    }
    Ok(fields)
}

fn require_practice_field_signatures(
    fields: &HashMap<String, FieldDecl>,
) -> Result<(), ScenarioError> {
    for qname in ["organization/action-budget", "organization/active"] {
        let Some(field) = fields.get(qname) else {
            return Err(coded_err(
                "E-LOAD-065",
                format!("practice topology requires `{qname}` as exact `int intensive`"),
            ));
        };
        if !matches!(field.ty, BslType::Int) || field.kind != FieldKind::Intensive {
            return Err(coded_err(
                "E-LOAD-065",
                format!("practice topology requires `{qname}` as exact `int intensive`"),
            ));
        }
    }
    Ok(())
}

fn map_practice_load_error(error: PracticeContractError) -> ScenarioError {
    match error {
        PracticeContractError::PracticeTopologyOrganizationLimit => coded_err(
            "E-LOAD-061",
            "practice topology exceeds 4,096 exact organization rows",
        ),
        PracticeContractError::PracticeFootprintLimit => coded_err(
            "E-LOAD-062",
            "practice topology exceeds 256 qualifying solidarity edges for one organization",
        ),
        PracticeContractError::PracticeTopologyBudgetMissing => coded_err(
            "E-LOAD-063",
            "an active organization lacks organization/action-budget",
        ),
        PracticeContractError::PracticeBudgetNonfinite
        | PracticeContractError::PracticeBudgetNegative
        | PracticeContractError::PracticeBudgetFractional
        | PracticeContractError::PracticeBudgetRange
        | PracticeContractError::PracticeBudgetRoundtrip => coded_err(
            "E-LOAD-064",
            format!(
                "organization/action-budget has invalid canonical storage (contract code {})",
                error as u16
            ),
        ),
        PracticeContractError::PracticeTopologyEdgeDuplicate => coded_err(
            "E-LOAD-044",
            "hydration seeds one canonical SOLIDARITY (source, target, type) triple twice",
        ),
        other => err(format!(
            "practice topology preflight returned unmapped contract code {}",
            other as u16
        )),
    }
}

fn preflight_node_kind(enum_type: &str, member: &str) -> PreflightNodeKind {
    if enum_type != "NodeType" {
        return PreflightNodeKind::Other;
    }
    match member {
        "ORGANIZATION" => PreflightNodeKind::Organization,
        "SOCIAL_CLASS" => PreflightNodeKind::SocialClass,
        _ => PreflightNodeKind::Other,
    }
}

fn effective_practice_attributes(attrs: &[SExpr]) -> (bool, Option<f64>) {
    let mut active = false;
    let mut budget = None;
    for attr in attrs.iter().take(MAX_SCENARIO_AST_NODES + 1) {
        let SExpr::List(pair) = attr else {
            continue;
        };
        let [SExpr::Atom(Atom::QName(field)), SExpr::Atom(value)] = pair.as_slice() else {
            continue;
        };
        if field == "organization/active" {
            active = matches!(value, Atom::Int(1));
        } else if field == "organization/action-budget" {
            budget = match value {
                Atom::Int(value) => {
                    #[allow(clippy::cast_precision_loss)]
                    let storage = *value as f64;
                    Some(storage)
                }
                _ => None,
            };
        }
    }
    (active, budget)
}

fn collect_preflight_nodes(
    body: &[SExpr],
    counter: &mut PracticeTopologyLoadCounter,
) -> Result<HashMap<String, PreflightNode>, ScenarioError> {
    let mut nodes = HashMap::new();
    for form in body.iter().take(MAX_SCENARIO_BODY_FORMS + 1) {
        let SExpr::List(parts) = form else {
            continue;
        };
        let [SExpr::Atom(Atom::Symbol(tag)), SExpr::Atom(Atom::Symbol(local)), SExpr::Atom(Atom::EnumRef { enum_type, member }), attrs @ ..] =
            parts.as_slice()
        else {
            continue;
        };
        if tag != "node" {
            continue;
        }
        if nodes.len() == MAX_SCENARIO_LOCAL_NAMES {
            return Err(err(
                "practice topology exceeds the 65,536-name local table bound",
            ));
        }
        let ordinal = u64::try_from(nodes.len())
            .map_err(|_| err("practice topology local ordinal does not fit u64"))?;
        let kind = preflight_node_kind(enum_type, member);
        if nodes
            .insert(local.clone(), PreflightNode { ordinal, kind })
            .is_some()
        {
            return Err(err(format!(
                "duplicate scenario name `{local}` — a local name denotes exactly one node"
            )));
        }
        if kind == PreflightNodeKind::Organization {
            let (active, budget) = effective_practice_attributes(attrs);
            counter
                .observe_organization(ordinal, active, budget)
                .map_err(map_practice_load_error)?;
        }
    }
    Ok(nodes)
}

fn preflight_practice_edges(
    body: &[SExpr],
    nodes: &HashMap<String, PreflightNode>,
    counter: &mut PracticeTopologyLoadCounter,
) -> Result<(), ScenarioError> {
    for form in body.iter().take(MAX_SCENARIO_BODY_FORMS + 1) {
        let SExpr::List(parts) = form else {
            continue;
        };
        let [SExpr::Atom(Atom::Symbol(tag)), SExpr::Atom(Atom::EnumRef { enum_type, member }), SExpr::Atom(Atom::Symbol(from)), SExpr::Atom(Atom::Symbol(to)), _strength] =
            parts.as_slice()
        else {
            continue;
        };
        if tag != "edge" || enum_type != "EdgeType" || member != "SOLIDARITY" {
            continue;
        }
        let (Some(source), Some(target)) = (nodes.get(from), nodes.get(to)) else {
            continue;
        };
        if source.kind != PreflightNodeKind::Organization
            || target.kind != PreflightNodeKind::SocialClass
        {
            continue;
        }
        counter
            .observe_solidarity_edge(
                source.ordinal,
                PracticeTargetDomainV1::SocialClass,
                target.ordinal,
            )
            .map_err(map_practice_load_error)?;
    }
    Ok(())
}

fn run_practice_topology_preflight(
    body: &[SExpr],
    prelude_fields: &HashMap<String, FieldDecl>,
    prelude_enums: &EnumRegistry,
) -> Result<(), ScenarioError> {
    if !practice_registry_trigger(body, prelude_fields) {
        return Ok(());
    }
    let fields = final_practice_fields(body, prelude_fields, prelude_enums)?;
    require_practice_field_signatures(&fields)?;
    let mut counter = PracticeTopologyLoadCounter::new();
    let nodes = collect_preflight_nodes(body, &mut counter)?;
    preflight_practice_edges(body, &nodes, &mut counter)?;
    counter.finish().map_err(map_practice_load_error)
}

/// Read `source` and populate `graph` with it.
///
/// # Errors
///
/// [`ScenarioError`] if the source does not read, the top-level form is not a
/// single `scenario`, a node or edge form is malformed, a local name is
/// duplicated or unknown, or the substrate refuses a write.
pub fn load_scenario(
    source: &str,
    graph: &mut dyn GraphSubstrate,
) -> Result<LoadedScenario, ScenarioError> {
    load_scenario_inner(
        UNNAMED_SCENARIO_SOURCE_ID,
        source,
        graph,
        PreludeRegistries::default(),
    )
}

/// Read `prelude_src` as a **declaration prelude**, then read
/// `scenario_src` as an ordinary scenario against the registries the
/// prelude built — the scenario-declaration sharing seam (Train B item 4,
/// issue #591, D157).
///
/// A prelude is content, but not a scenario: no `(scenario <qname> …)`
/// wrapper, and only the four DECLARATION top-forms are legal in it
/// (`defenum` / `defvocabulary` / `defconst` / `deffield`) — `node` /
/// `edge` / `edge-attr` never touch a prelude's ungraphed pass, so admitting
/// them would either silently drop a graph write or force a `graph`
/// parameter this call never needs. See this module's private
/// `load_prelude` (its refusal) and `load_scenario_inner` (the shared load
/// core) for the mechanism.
///
/// The scenario that follows MAY re-declare a `defenum` type the prelude
/// already declared, verbatim — [`crate::types::EnumRegistry::declare`]'s
/// identical-recognition arm (also this train) returns the prelude's own
/// [`crate::types::EnumTypeId`] rather than refusing — but a re-declaration
/// that disagrees (reordered, renamed, added, or dropped a member) still
/// refuses loudly, exactly as two colliding `defenum` forms in one file
/// always have. **This is `defenum`-only.** `deffield`, `defconst`, and
/// `defvocabulary` gained no equivalent arm: each still refuses ANY second
/// declaration of the same name, identical or not (`fields.insert(...)
/// .is_some()`, `consts.insert(...).is_some()`, and `defvocabulary`'s
/// `E-LOAD-001` kind-guard are all unconditional collision checks) — a
/// scenario must NOT re-declare a prelude-supplied `deffield`/`defconst`/
/// `defvocabulary`, even verbatim.
///
/// # Errors
///
/// [`ScenarioError`] if either source does not read, the prelude names a
/// non-declaration form, or the scenario half fails for any of
/// [`load_scenario`]'s own reasons.
pub fn load_scenario_with_prelude(
    prelude_src: &str,
    scenario_src: &str,
    graph: &mut dyn GraphSubstrate,
) -> Result<LoadedScenario, ScenarioError> {
    let registries = load_prelude(UNNAMED_PRELUDE_SOURCE_ID, prelude_src)?;
    load_scenario_inner(UNNAMED_SCENARIO_SOURCE_ID, scenario_src, graph, registries)
}

/// Load one named scenario against ordered, caller-named declaration preludes.
///
/// Each source is parsed exactly once. Prelude registries accumulate in caller
/// order, and exact Mass declarations are retained by the same semantic pass
/// that admits their `defconst` forms. The returned facts use paths local to
/// their own source and are ordered scenario-first, then by prelude source.
///
/// The prelude byte-shape and count checks are exactly
/// [`compose_declaration_preludes`]'s checks and run before parsing. An
/// offset-zero BOM remains legal only for the first prelude: in the historical
/// composed byte stream, a BOM beginning any later source was misplaced.
///
/// # Errors
///
/// [`ScenarioError`] for the same source-shape, declaration, registry, and
/// scenario hydration refusals as composing the preludes and calling
/// [`load_scenario_with_prelude`].
pub fn load_scenario_with_named_preludes(
    scenario_source_id: &str,
    scenario_src: &str,
    prelude_sources: &[NamedDeclarationPreludeV1<'_>],
    graph: &mut dyn GraphSubstrate,
) -> Result<LoadedScenario, ScenarioError> {
    let registries = load_named_preludes(prelude_sources)?;
    load_scenario_inner(scenario_source_id, scenario_src, graph, registries)
}

/// Compose ordered declaration preludes without changing any admitted byte.
///
/// Each source must be non-empty, contain no carriage return, and end in
/// exactly one line feed. At most 16 sources, 262,144 bytes per source, and
/// 1,048,576 bytes in total are admitted. The existing single-source loader
/// remains the authority for declaration syntax and registry collisions.
///
/// # Errors
///
/// Returns [`ScenarioError`] when any source or combined bound is exceeded,
/// or when a source violates the byte-shape contract.
pub fn compose_declaration_preludes(sources: &[&str]) -> Result<String, ScenarioError> {
    if sources.len() > 16 {
        return Err(err(
            "a content set may compose at most 16 declaration preludes",
        ));
    }
    let mut combined_bytes = 0_usize;
    for source_index in 0..16 {
        if source_index == sources.len() {
            break;
        }
        let source = sources[source_index];
        let bytes = source.as_bytes();
        if bytes.is_empty() {
            return Err(err("a declaration prelude source must not be empty"));
        }
        if bytes.len() > 262_144 {
            return Err(err("a declaration prelude source exceeds 262144 bytes"));
        }
        if bytes.last() != Some(&b'\n') || (bytes.len() >= 2 && bytes[bytes.len() - 2] == b'\n') {
            return Err(err(
                "a declaration prelude source must have exactly one terminal line feed",
            ));
        }
        for byte_index in 0..262_144 {
            if byte_index == bytes.len() {
                break;
            }
            if bytes[byte_index] == b'\r' {
                return Err(err(
                    "a declaration prelude source must not contain carriage return",
                ));
            }
        }
        combined_bytes = combined_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| err("declaration prelude combined byte count overflow"))?;
        if combined_bytes > 1_048_576 {
            return Err(err("composed declaration preludes exceed 1048576 bytes"));
        }
    }

    let mut composed = String::with_capacity(combined_bytes);
    for source_index in 0..16 {
        if source_index == sources.len() {
            break;
        }
        composed.push_str(sources[source_index]);
    }
    Ok(composed)
}

/// A prelude's own load pass: every top-level form in `prelude_src`, in
/// order, dispatched to exactly the four declaration handlers
/// [`load_scenario_inner`]'s own loop uses — never touching a graph, since
/// a prelude declares, it never seeds.
///
/// # Errors
///
/// [`ScenarioError`] if `prelude_src` does not read, a top-level form is not
/// a list, or a form's head is not `defenum` / `defvocabulary` / `defconst`
/// / `deffield`.
fn load_prelude(source_id: &str, prelude_src: &str) -> Result<PreludeRegistries, ScenarioError> {
    let forms = bounded_scenario_read(prelude_src)?;
    let mut registries = PreludeRegistries::default();
    load_prelude_forms(source_id, &forms, &mut registries)?;
    Ok(registries)
}

fn load_named_preludes(
    sources: &[NamedDeclarationPreludeV1<'_>],
) -> Result<PreludeRegistries, ScenarioError> {
    let source_texts = sources
        .iter()
        .map(|source| source.source)
        .collect::<Vec<_>>();
    let _validated_composition = compose_declaration_preludes(&source_texts)?;
    let mut registries = PreludeRegistries::default();
    let mut preceding_bytes = 0_usize;
    for (source_index, source) in sources.iter().enumerate() {
        if source_index != 0 && source.source.starts_with('\u{feff}') {
            return Err(ScenarioError::from(ReadError {
                kind: ReadErrorKind::Lex(LexCode::InvalidUtf8OrBom),
                message: "a BOM is only accepted at offset 0 (§1.1)".to_owned(),
                position: preceding_bytes,
            }));
        }
        let forms = bounded_scenario_read(source.source)?;
        load_prelude_forms(source.source_id, &forms, &mut registries)?;
        preceding_bytes = preceding_bytes
            .checked_add(source.source.len())
            .ok_or_else(|| err("declaration prelude combined byte count overflow"))?;
    }
    Ok(registries)
}

fn load_prelude_forms(
    source_id: &str,
    forms: &[SExpr],
    registries: &mut PreludeRegistries,
) -> Result<(), ScenarioError> {
    for (form_index, form) in forms.iter().enumerate() {
        let SExpr::List(parts) = form else {
            return Err(err("a prelude form must be a list — (defenum ...), \
                 (defvocabulary ...), (deffield ...) or (defconst ...)"));
        };
        match parts.first() {
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defenum" => {
                load_defenum(form, &mut registries.enums)?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defvocabulary" => {
                load_defvocabulary(
                    form,
                    &mut registries.vocabulary_members,
                    &mut registries.vocabulary_kinds_declared,
                )?;
                registries.vocabulary_so_far = Some(ClosedVocabulary::new(
                    registries.vocabulary_members.clone(),
                )?);
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "deffield" => {
                load_deffield(parts, &mut registries.fields, &registries.enums)?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defconst" => {
                load_defconst(
                    parts,
                    &mut registries.consts,
                    &mut registries.probability_consts,
                )?;
                retain_mass_declaration(
                    source_id,
                    parts,
                    vec![checked_form_index(form_index)?, 2],
                    &mut registries.mass_declarations,
                );
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) => {
                return Err(err(format!(
                    "a prelude form must be `defenum`, `defvocabulary`, `deffield` \
                     or `defconst` — found `{tag}` (node/edge/edge-attr/hyperedge forms \
                     belong in the scenario, never the prelude — a prelude never touches \
                     the graph)"
                )))
            }
            _ => {
                return Err(err(
                    "a prelude form must begin with a symbol naming `defenum`, \
                     `defvocabulary`, `deffield` or `defconst`",
                ))
            }
        }
    }
    Ok(())
}

fn checked_form_index(index: usize) -> Result<u32, ScenarioError> {
    u32::try_from(index).map_err(|_| err("source has too many forms for a parser-stable path"))
}

fn retain_mass_declaration(
    source_id: &str,
    parts: &[SExpr],
    literal_path: FormPath,
    declarations: &mut Vec<MassDeclarationAnalysisV1>,
) {
    let [_, SExpr::Atom(Atom::QName(qname)), SExpr::Atom(Atom::Mass(mass)), ..] = parts else {
        return;
    };
    declarations.push(MassDeclarationAnalysisV1 {
        source_id: source_id.to_owned(),
        qname: qname.clone(),
        form_path: literal_path,
        mass: *mass,
    });
}

/// The shared load core used by all scenario entry points: read `source` once
/// and hydrate its already parsed forms against `registries`.
///
/// # Errors
///
/// See [`load_scenario`]'s own doc — every failure mode is identical; this
/// is that function's body, extracted so a prelude pass can seed it.
// G2 (#534 fix round 2 item 2): threading `&enums` symmetrically into
// `load_edge`'s call site (alongside `load_node`'s pre-existing one), so
// `demand_enum_kind` can recognize a scenario-declared `defenum` type as a
// REAL type — not just the four structural kinds — crosses the ~100-line
// soft cap by exactly one line. Splitting this single, cohesive top-to-
// bottom load loop (whose own doc states "declaration order is the id
// order") into smaller pieces would trade that linear narrative for
// indirection over a one-line breach; not worth it. Train B item 4 (#591):
// this function IS that split's one legitimate exception — the extraction
// is HORIZONTAL (this whole function, called twice with different starting
// registries via `PreludeRegistries`), not a division of the loop's own
// body, so the "smaller pieces" argument above is untouched by it;
// `load_scenario` itself dropped back under the cap by this move and now
// carries no attribute of its own.
fn load_scenario_inner(
    source_id: &str,
    source: &str,
    graph: &mut dyn GraphSubstrate,
    registries: PreludeRegistries,
) -> Result<LoadedScenario, ScenarioError> {
    let forms = bounded_scenario_read(source)?;
    load_scenario_forms(source_id, &forms, graph, registries)
}

#[allow(clippy::too_many_lines)]
fn load_scenario_forms(
    source_id: &str,
    forms: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    registries: PreludeRegistries,
) -> Result<LoadedScenario, ScenarioError> {
    check_scenario_ast_bounds(forms)?;
    let [SExpr::List(items)] = forms else {
        return Err(err(format!(
            "a scenario file holds exactly one (scenario ...) form; found {}",
            forms.len()
        )));
    };

    let (head, id, body) =
        match items.as_slice() {
            [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::QName(id)), body @ ..] => {
                (head.as_str(), id.clone(), body)
            }
            _ => return Err(err(
                "expected (scenario <qname> <form>*) — the id must be a qname, e.g. ft/two-classes",
            )),
        };
    if head != "scenario" {
        return Err(err(format!(
            "expected a (scenario ...) form, found ({head} ...)"
        )));
    }
    if body.len() > MAX_SCENARIO_BODY_FORMS {
        return Err(err("scenario body exceeds the 65,536-form load bound"));
    }

    run_practice_topology_preflight(body, &registries.fields, &registries.enums)?;

    // The four declaration registries plus the vocabulary trio, pre-seeded
    // by a prelude pass (`load_scenario_with_prelude`) or empty
    // (`load_scenario`'s own call) — see `PreludeRegistries`'s own field
    // docs for why each exists.
    let PreludeRegistries {
        mut fields,
        mut consts,
        mut probability_consts,
        mass_declarations: prelude_mass_declarations,
        mut enums,
        mut vocabulary_members,
        mut vocabulary_kinds_declared,
        mut vocabulary_so_far,
    } = registries;
    let mut scenario_mass_declarations = Vec::new();

    // Local name -> minted id. Load-time only; it does not outlive this call.
    let mut named: HashMap<String, NodeId> = HashMap::new();
    let mut node_types: HashMap<String, u64> = HashMap::new();
    let mut edge_types: HashMap<String, u64> = HashMap::new();
    // Hyperedge-lane Task 1: the census `load_hyperedge` builds, mirroring
    // `node_types`/`edge_types` exactly — see `LoadedScenario::hyperedge_types`'s
    // own doc for the argument.
    let mut hyperedge_types: HashMap<String, u64> = HashMap::new();
    let mut max_members_seen: HashMap<String, u64> = HashMap::new();
    // Fix round 1, review I3: the local-name table `load_hyperedge` checks
    // duplicates against — same purpose as `named` does for nodes
    // (`load_node`'s own duplicate refusal). Task 6's `(hyperedge-attr …)`
    // landed the resolve-by-name consumer this comment's predecessor
    // anticipated, so the table is now a name→HyperedgeId map — the
    // duplicate check's own behaviour is unchanged (`contains_key` where
    // the set's `insert` returned false).
    let mut seeded_hyperedge_names: HashMap<String, HyperedgeId> = HashMap::new();
    // Task 6's `(hyperedge-attr …)` KEY, mirroring `seeded_attrs` one element
    // kind over: a second seeding of one (hyperedge, field) pair silently
    // overwrites the first — E-LOAD-057's argument, one axis narrower.
    let mut seeded_hyperedge_attrs: HashSet<(HyperedgeId, String)> = HashSet::new();
    let mut node_count = 0_usize;
    let mut edge_count = 0_usize;
    // §3.9 clause 5 (D73): hydration may not seed two dyadic edges sharing
    // one `(source-id, target-id, edge-type)` triple. This set is what
    // makes the triple a KEY rather than a sort field — without it §2.6's
    // edge iteration order is not a total order and §2.10's `edge-between`
    // has no rule for resolving two.
    let mut seeded_edges: HashSet<(String, NodeId, NodeId)> = HashSet::new();
    // Train B item 3 (#591, D156): the `(edge-attr …)` form's own key, one
    // axis wider than E-LOAD-044's triple — a declared FIELD of an already-
    // seeded edge. The quadruple is a KEY for the same reason the triple
    // is: a second seeding of one key silently overwrites the first, the
    // file carrying two values for one datum with only the later surviving.
    let mut seeded_attrs: HashSet<(String, NodeId, NodeId, String)> = HashSet::new();

    for (body_index, form) in body.iter().enumerate() {
        let SExpr::List(parts) = form else {
            return Err(err(
                "a scenario body holds only (defenum ...), (defvocabulary ...), \
                 (deffield ...), (defconst ...), (node ...), (edge ...), \
                 (edge-attr ...), (hyperedge ...) and (hyperedge-attr ...) forms",
            ));
        };
        match parts.first() {
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defenum" => {
                load_defenum(form, &mut enums)?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defvocabulary" => {
                load_defvocabulary(
                    form,
                    &mut vocabulary_members,
                    &mut vocabulary_kinds_declared,
                )?;
                vocabulary_so_far = Some(ClosedVocabulary::new(vocabulary_members.clone())?);
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "deffield" => {
                load_deffield(parts, &mut fields, &enums)?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "defconst" => {
                load_defconst(parts, &mut consts, &mut probability_consts)?;
                let form_index = body_index
                    .checked_add(2)
                    .ok_or_else(|| err("scenario form path index overflow"))?;
                retain_mass_declaration(
                    source_id,
                    parts,
                    vec![0, checked_form_index(form_index)?, 2],
                    &mut scenario_mass_declarations,
                );
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "node" => {
                let minted = load_node(
                    parts,
                    graph,
                    &mut named,
                    &fields,
                    &enums,
                    vocabulary_so_far.as_ref(),
                )?;
                *node_types.entry(minted).or_insert(0) += 1;
                node_count += 1;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "edge" => {
                let minted = load_edge(
                    parts,
                    graph,
                    &named,
                    &mut seeded_edges,
                    &enums,
                    vocabulary_so_far.as_ref(),
                )?;
                *edge_types.entry(minted).or_insert(0) += 1;
                edge_count += 1;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "edge-attr" => {
                load_edge_attr(
                    parts,
                    graph,
                    &named,
                    &seeded_edges,
                    &mut seeded_attrs,
                    &fields,
                    &enums,
                    vocabulary_so_far.as_ref(),
                )?;
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "hyperedge" => {
                let (minted, member_count) = load_hyperedge(
                    parts,
                    graph,
                    &named,
                    &mut seeded_hyperedge_names,
                    &enums,
                    vocabulary_so_far.as_ref(),
                )?;
                *hyperedge_types.entry(minted.clone()).or_insert(0) += 1;
                let longest = max_members_seen.entry(minted).or_insert(0);
                if member_count > *longest {
                    *longest = member_count;
                }
            }
            Some(SExpr::Atom(Atom::Symbol(tag))) if tag == "hyperedge-attr" => {
                load_hyperedge_attr(
                    parts,
                    graph,
                    &seeded_hyperedge_names,
                    &mut seeded_hyperedge_attrs,
                    &fields,
                    &enums,
                )?;
            }
            _ => {
                return Err(err(
                    "a scenario body form must begin with `defenum`, `defvocabulary`, \
                     `deffield`, `defconst`, `node`, `edge`, `edge-attr`, `hyperedge` \
                     or `hyperedge-attr`",
                ))
            }
        }
    }

    // Opt-in per scenario (Task 7): no `defvocabulary` forms at all leaves
    // `vocabulary_so_far` at its initial `None`, which is what keeps every
    // scenario predating this section loading exactly as it did before it.
    // Already fully built (`vocabulary_so_far` is rebuilt after EVERY
    // `defvocabulary` form above, so by here it reflects all of them).
    let vocabulary = vocabulary_so_far;

    // Task 3 (plan §3.4): retain content-stable node identity by inverting
    // the load-time local-name table before it goes out of scope.
    let node_content_ids = invert_content_ids(&named);
    let hyperedge_content_ids = invert_hyperedge_content_ids(&seeded_hyperedge_names);
    scenario_mass_declarations.extend(prelude_mass_declarations);

    Ok(LoadedScenario {
        id,
        node_count,
        edge_count,
        node_types,
        edge_types,
        hyperedge_types,
        max_members_seen,
        fields,
        consts,
        probability_consts,
        mass_declarations: scenario_mass_declarations,
        enums,
        vocabulary,
        node_content_ids,
        hyperedge_content_ids,
    })
}

/// Invert `named` (local name -> [`NodeId`]) into the content-id map
/// [`LoadedScenario::node_content_ids`] exposes.
///
/// # Panics
///
/// If two DIFFERENT content ids resolve to the SAME `NodeId`. Through every
/// reachable call site this is unconstructible: `load_node` mints a fresh id
/// via `graph.add_node()` and inserts exactly one `(local, id)` pair into
/// `named` per `(node ...)` form (`load_node`, this module), and
/// `named.contains_key(local)` (also `load_node`) already refuses a second
/// `(node ...)` form reusing a local name before this function ever runs. So
/// a collision here means the loader started minting a NON-fresh id for some
/// node — a hydration bug, and the injectivity this function asserts must
/// fail LOUDLY rather than silently keep whichever entry `HashMap` iteration
/// happened to visit last (this module's own test
/// `two_content_ids_colliding_onto_one_node_id_is_a_loud_hydration_bug_not_a_silent_overwrite`
/// exercises this directly, at this function, since the loader itself
/// cannot construct the violating input).
fn invert_content_ids(named: &HashMap<String, NodeId>) -> HashMap<NodeId, String> {
    let mut content_ids: HashMap<NodeId, String> = HashMap::with_capacity(named.len());
    for (local, &id) in named {
        if let Some(existing) = content_ids.insert(id, local.clone()) {
            panic!(
                "hydration bug: NodeId {id:?} is bound to two different content ids \
                 (`{existing}` and `{local}`) — two content ids must never collide onto \
                 one NodeId handle, and silently keeping one would be indistinguishable \
                 from a lost node"
            );
        }
    }
    content_ids
}

fn invert_hyperedge_content_ids(
    named: &HashMap<String, HyperedgeId>,
) -> HashMap<HyperedgeId, String> {
    let mut content_ids = HashMap::with_capacity(named.len());
    for (local, &id) in named {
        if let Some(existing) = content_ids.insert(id, local.clone()) {
            panic!(
                "hydration bug: HyperedgeId {id:?} is bound to two different content ids \
                 (`{existing}` and `{local}`)"
            );
        }
    }
    content_ids
}

/// `(defconst <qname> <literal>)`, or `(defconst <qname> <ratio-literal>
/// [:floor <ratio-literal>] [:cap <ratio-literal>])` (§3.2 addendum,
/// Director ruling 2026-08-11, #492/ADR194 — the declared-domain scale
/// operation).
///
/// The defines environment in miniature (§2.5's `:const`, §4.2's
/// "the defines environment (coefficients)"). Slice 1 has no
/// `GameDefines`/`defines.yaml` reader, and §2.5 gives `:const` no other
/// source, so the alternative to declaring the value here is writing it
/// into the rule — putting a magnitude in the file that owns the shape.
///
/// Of the five literal atom classes §2.2 admits at `:default` plus §1.5's
/// addendum, all five are legal here — `Int`, `Scaled` (`p`/`i`/`c`/`r`),
/// `Bool` (defines carry toggles as well as magnitudes), and — since T3
/// #491/OQ-J — `Currency`, carried exactly as `:default` now carries it
/// (`tick.rs::atom_to_value`). The literal-only rule holds for the same
/// reason `:default`'s does: a
/// define is a value, not an expression, and an expression would need an
/// evaluation environment that does not exist at scenario-load time.
///
/// **`:floor`/`:cap` (#492/ADR194).** `defconst`/`node`/`edge`/`scenario`
/// are the `.bscn`-dialect construct `bsl-language.rst` §7 records as OUT
/// of the consolidated grammar's scope (D93: "no section specifies it") —
/// so both keywords are Rust-implementation machinery, not an RST
/// production, and do NOT go through the closed §1.6 keyword vocabulary or
/// its `E-PARSE-013` enforcement ([`parse_bound_keywords`] is that
/// enforcement, same as this function's own hand-rolled positional match is
/// for `defconst`'s base shape). Both are legal ONLY on a `Ratio`
/// (`r`-suffixed) literal: `Ratio`'s own domain is already `(0, ∞)` (§1.5
/// addendum), and each keyword narrows ONE end of it —
/// `:cap` to `(0, cap]` (INCLUSIVE, matching a `<=`-bounded consumer like
/// `pareto_alpha`'s `(0, 10]`), `:floor` to `(floor, ∞)` (EXCLUSIVE,
/// matching Ratio's own open-at-zero law AND a `>`-bounded consumer like
/// `entropy_factor`'s `(1.0, 3.0]`), together `(floor, cap]`. Stated at
/// declaration, loud and visible in the source text, and each bound checked
/// twice: HERE at load (`E-LOAD-052`, the literal must not itself violate
/// the domain it declares) and again at every `Currency × Ratio` evaluation
/// (`E-EVAL-041`, `evaluator::currency_mul_ratio`) — defense in depth, per
/// III.11. An UNDECLARED (bare) `Ratio` defconst is exactly as legal as
/// before this addendum; the `[0,1]` cap on `p`/`i`/`c` defconsts is
/// completely untouched — this is a new, disjoint literal kind, not a
/// widening of the existing three.
fn load_defconst(
    parts: &[SExpr],
    consts: &mut HashMap<String, Value>,
    probability_consts: &mut HashSet<String>,
) -> Result<(), ScenarioError> {
    let [_, SExpr::Atom(Atom::QName(qname)), SExpr::Atom(literal), rest @ ..] = parts else {
        return Err(err(
            "expected (defconst <qname> <literal>) — one qualified name, one literal",
        ));
    };
    let (floor_literal, cap_literal) = parse_bound_keywords(qname, rest)?;
    let value = match literal {
        Atom::Mass(mass) => {
            reject_stray_bounds(qname, floor_literal, cap_literal, "a Mass literal")?;
            Value::Mass(*mass)
        }
        Atom::Int(value) => {
            reject_stray_bounds(qname, floor_literal, cap_literal, "an Int")?;
            Value::Int(*value)
        }
        Atom::Scaled(scaled) if scaled.kind == ScaledKind::Ratio => {
            load_ratio_defconst(qname, scaled, floor_literal, cap_literal)?
        }
        Atom::Scaled(scaled) => {
            reject_stray_bounds(qname, floor_literal, cap_literal, "a p/i/c literal")?;
            // `unscaled / 10^scale`, the canonical minimal-scale form, and
            // the SAME arithmetic `tick.rs::atom_to_value` performs on a
            // `:default` literal — so a scaled coefficient reads identically
            // whichever door it enters by.
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            Value::Real(numerator / 10_f64.powi(i32::from(scaled.scale)))
        }
        Atom::Bool(value) => {
            reject_stray_bounds(qname, floor_literal, cap_literal, "a Bool")?;
            Value::Bool(*value)
        }
        // Carried, not refused (T3 #491, OQ-J — Half 2 landed): Currency's
        // typed storage exists now, `tick.rs::atom_to_value` carries a
        // Currency `:default` and `attribute_value_currency` carries a
        // Currency node attribute, both into `Value::Currency` — a
        // `defconst` follows the same rule for the same reason every other
        // literal kind here does: one literal, one legality, regardless of
        // which door it enters by. `Value::Currency` needs no NEW carrier
        // (`consts: &mut HashMap<String, Value>` already holds any `Value`
        // variant); this is a pure environment lookup, never hashed graph
        // state, so it touches no `CanonicalState` section.
        Atom::Currency(c) => {
            reject_stray_bounds(qname, floor_literal, cap_literal, "a Currency literal")?;
            Value::Currency(*c)
        }
        other => {
            return Err(err(format!(
                "defconst `{qname}`: expected an int, Mass, scaled or boolean \
                 literal, found {other:?}"
            )))
        }
    };
    if consts.insert(qname.clone(), value).is_some() {
        // `qname` is local context `err()`'s own signature has no slot for
        // (§2.3, issue #652 Task 2) — attach it after construction rather
        // than adding an eighth `ScenarioError { .. }` site.
        return Err(err(format!(
            "duplicate defconst `{qname}` — a coefficient has one value, and \
             silently rebinding it would make the rule reading it depend on \
             declaration order"
        ))
        .with_identity(ErrorIdentity::Name(qname.clone())));
    }
    if matches!(
        literal,
        Atom::Scaled(scaled) if scaled.kind == ScaledKind::Probability
    ) {
        probability_consts.insert(qname.clone());
    }
    Ok(())
}

/// Scan the trailing `[:floor <literal>] [:cap <literal>]` keyword-operand
/// pairs after a `defconst`'s required `<qname> <literal>` (in either
/// order, either or both present) — the same hand-rolled enforcement
/// [`load_defconst`]'s doc describes: `.bscn` machinery, not the closed
/// §1.6 vocabulary.
///
/// # Errors
/// A structural (uncoded) [`ScenarioError`] on an unrecognized keyword, a
/// duplicated one, a keyword with no operand, or a non-keyword atom in this
/// position — never a silent drop of the extra tokens.
fn parse_bound_keywords<'a>(
    qname: &str,
    rest: &'a [SExpr],
) -> Result<(Option<&'a Atom>, Option<&'a Atom>), ScenarioError> {
    let mut floor_literal = None;
    let mut cap_literal = None;
    let mut i = 0;
    while i < rest.len() {
        let SExpr::Atom(Atom::Keyword(kw)) = &rest[i] else {
            return Err(err(format!(
                "defconst `{qname}`: expected :floor or :cap here, found {:?}",
                rest[i]
            )));
        };
        let Some(SExpr::Atom(operand)) = rest.get(i + 1) else {
            return Err(err(format!(
                "defconst `{qname}`: :{kw} takes an operand but ends the form"
            )));
        };
        let slot = match kw.as_str() {
            "floor" => &mut floor_literal,
            "cap" => &mut cap_literal,
            other => {
                return Err(err(format!(
                    "defconst `{qname}`: unrecognized keyword :{other} — only \
                     :floor and :cap are legal here (§3.2 addendum, #492/ADR194)"
                )))
            }
        };
        if slot.replace(operand).is_some() {
            return Err(err(format!(
                "defconst `{qname}`: :{kw} given twice — a bound has one \
                 value, and silently rebinding it would make the check \
                 depend on declaration order"
            )));
        }
        i += 2;
    }
    Ok((floor_literal, cap_literal))
}

/// `:floor`/`:cap` are legal only on a `Ratio` literal (see
/// [`load_defconst`]'s doc); this names the refusal by the OTHER literal
/// kind found instead, rather than routing through the generic "expected an
/// int, scaled or boolean literal" arm, which would misdiagnose a
/// well-formed-but-misplaced bound as a malformed literal.
fn reject_stray_bounds(
    qname: &str,
    floor_literal: Option<&Atom>,
    cap_literal: Option<&Atom>,
    found: &str,
) -> Result<(), ScenarioError> {
    if floor_literal.is_some() || cap_literal.is_some() {
        return Err(err(format!(
            "defconst `{qname}`: :floor/:cap are legal only on a Ratio \
             (r-suffixed) literal (§3.2 addendum, #492/ADR194), found {found}"
        )));
    }
    Ok(())
}

/// The `Ratio`-literal half of [`load_defconst`]: builds
/// `Value::Ratio { value, floor, cap }`, checking the declared bounds at
/// load (`E-LOAD-052`) — the bounds against EACH OTHER first (when both are
/// present), then the value against each declared bound.
///
/// **Order matters, and is checked by
/// `a_floor_not_strictly_below_its_cap_is_e_load_052`.** `(floor, cap]` is
/// only a non-empty domain when `floor < cap`; when it is NOT, `floor >=
/// cap` implies `value > floor` and `value <= cap` can never BOTH hold (if
/// they could, `floor < value <= cap` would prove `floor < cap`, the
/// contrapositive of what triggered this branch) — so checking the value
/// against either bound FIRST would always fire on ONE of them, misreporting
/// a self-inconsistent DECLARATION as a bad VALUE. Checking the bounds
/// against each other first names the actual defect.
fn load_ratio_defconst(
    qname: &str,
    scaled: &crate::reader::ScaledLit,
    floor_literal: Option<&Atom>,
    cap_literal: Option<&Atom>,
) -> Result<Value, ScenarioError> {
    let value = ratio_from_scaled(qname, scaled)?;
    let floor = parse_ratio_bound(qname, "floor", floor_literal)?;
    let cap = parse_ratio_bound(qname, "cap", cap_literal)?;
    if let (Some(floor), Some(cap)) = (floor, cap) {
        if floor.get() >= cap.get() {
            return Err(coded_err(
                "E-LOAD-052",
                format!(
                    "defconst `{qname}`: declared :floor {} is not strictly \
                     below its own :cap {} — (floor, cap] is empty unless \
                     floor < cap (§3.2 addendum, #492/ADR194)",
                    floor.get(),
                    cap.get()
                ),
            ));
        }
    }
    if let Some(floor) = floor {
        // EXCLUSIVE: value must be STRICTLY greater than its declared floor
        // (matches Ratio's own open-at-zero law and entropy_factor's own
        // `> 1.0`). `floor > 0` needs no separate check here: `floor` is
        // itself a `Ratio`, whose constructor already refused a
        // non-positive value (and the reader's `E-LEX-027` refused it
        // earlier still) — re-checking it would be provably redundant, the
        // same class of already-guaranteed check D-4 of
        // `content/rules/lifecycle.bsl` records rather than re-asserting.
        if value.get() <= floor.get() {
            return Err(coded_err(
                "E-LOAD-052",
                format!(
                    "defconst `{qname}`: declared value {} does not exceed its \
                     own :floor {} — a defconst's :floor states the const's OWN \
                     domain floor EXCLUSIVE, so the literal must be strictly \
                     greater than it (§3.2 addendum, #492/ADR194)",
                    value.get(),
                    floor.get()
                ),
            ));
        }
    }
    if let Some(cap) = cap {
        if value.get() > cap.get() {
            return Err(coded_err(
                "E-LOAD-052",
                format!(
                    "defconst `{qname}`: declared value {} exceeds its own \
                     :cap {} — a defconst's :cap states the const's OWN \
                     domain ceiling INCLUSIVE, so the literal must satisfy it \
                     (§3.2 addendum, #492/ADR194)",
                    value.get(),
                    cap.get()
                ),
            ));
        }
    }
    Ok(Value::Ratio { value, floor, cap })
}

/// One `:floor`/`:cap` operand, as a [`Ratio`] — `None` when the keyword was
/// absent, else the parsed bound or a structural refusal if its literal is
/// not itself a `Ratio`.
fn parse_ratio_bound(
    qname: &str,
    keyword: &str,
    literal: Option<&Atom>,
) -> Result<Option<Ratio>, ScenarioError> {
    match literal {
        None => Ok(None),
        Some(Atom::Scaled(scaled)) if scaled.kind == ScaledKind::Ratio => {
            ratio_from_scaled(qname, scaled).map(Some)
        }
        Some(other) => Err(err(format!(
            "defconst `{qname}`: :{keyword}'s operand must be a Ratio \
             (r-suffixed) literal, found {other:?}"
        ))),
    }
}

/// Convert a canonicalized `r`-literal to a kernel [`Ratio`]. Should never
/// fail — the reader's `E-LEX-027` already refused a non-positive value at
/// lex time — but a defensive, named error (rather than an `expect`) keeps
/// a reader/kernel sort disagreement a loud `ScenarioError`, not a panic.
fn ratio_from_scaled(
    qname: &str,
    scaled: &crate::reader::ScaledLit,
) -> Result<Ratio, ScenarioError> {
    #[allow(clippy::cast_precision_loss)]
    let raw = scaled.unscaled as f64 / 10_f64.powi(i32::from(scaled.scale));
    Ratio::new(raw).map_err(|e| {
        err(format!(
            "defconst `{qname}`: r literal {raw} failed Ratio construction \
             ({e:?}) — the reader's E-LEX-027 should have refused this at lex \
             time; reaching here is a reader/kernel sort disagreement, not a \
             content error"
        ))
    })
}

/// `(defenum <enum-type> (<enum-member>+))` — §2.13, D101. Delegates to
/// `declarations::parse_defenum`, the SAME parser `.bsl` rule content uses
/// (D93's own dialect split is about `defconst`/`node`/`edge`/`scenario`,
/// which the RST grammar disclaims; `defenum` is an RST `<top-form>`, so
/// there is exactly one grammar for it and no reason for a second reader).
/// Members are written BARE (`STATE_APPARATUS`) — see that function's own
/// doc for the #528 fix round's corrected reading.
///
/// # Errors
///
/// A [`ScenarioError`] wrapping the underlying [`crate::declarations::DeclError`]
/// — `E-LOAD-001` for a duplicate type/member, uncoded for an empty member
/// list or a malformed shape.
fn load_defenum(form: &SExpr, enums: &mut EnumRegistry) -> Result<(), ScenarioError> {
    crate::declarations::parse_defenum(form, enums)
        .map(|_| ())
        .map_err(|e| ScenarioError {
            code: e.spec_code(),
            identity: decl_identity(&e),
            message: e.to_string(),
            position: None,
        })
}

/// `(defvocabulary <enum-kind> (<enum-member>+))` — §2.13, D101; §3.6's own
/// closed graph vocabulary, populated **explicitly**, never inferred from
/// what a scenario happens to seed. `<enum-kind>` is syntactically an
/// `<enum-type>` (a bare, slash-free [`Atom::BareUpperIdent`] — see that
/// variant's doc for why the reader lexes it that way) but SEMANTICALLY
/// restricted to `NodeType` / `EdgeType` / `HyperedgeType` / `EventType` —
/// an unknown name is `E-LOAD-030`, the same code §3.6 already uses for an
/// unregistered `<enum-ref>` type name (this is a load-time check, not a
/// lexical one). That closed-set check already subsumes a separate
/// `<enum-type>`-shape check on the kind-name operand: none of the four
/// valid kind names contains `_`, so a shape-invalid name (`Node_Type`)
/// fails `EnumKind::from_type_name` on its own — a redundant
/// `is_enum_type_shape` check here would only pre-empt that existing,
/// already-coded `E-LOAD-030` with a less specific uncoded one, so this
/// function does not add one (contrast `declarations::parse_defenum`,
/// which mints a NEW type name with nothing to check it against and so
/// DOES need the direct shape check).
///
/// **Members are written BARE** (`SOCIAL_CLASS`, not `NodeType/
/// SOCIAL_CLASS`) — the SAME reading `load_defenum`/`parse_defenum` takes,
/// for the same reason (§2.13's own EBNF; see that function's doc for the
/// #528 fix round's corrected reading). A member written as a full
/// enum-ref is grammar-nonconforming and refuses loudly.
///
/// Only inserts into `collected`/`declared` — `ClosedVocabulary::new` (the
/// caller, once at end-of-load) runs the whole-vocabulary
/// rendering-disjointness check over every kind together.
///
/// # Errors
///
/// `E-LOAD-030` for an unregistered `<enum-kind>`; `E-LOAD-001` for a
/// second `defvocabulary` naming a kind already declared; an uncoded
/// [`ScenarioError`] off the grammar, including a member not shaped like
/// `<enum-member>` or written as a full enum-ref.
fn load_defvocabulary(
    form: &SExpr,
    collected: &mut HashMap<EnumKind, Vec<String>>,
    declared: &mut HashSet<EnumKind>,
) -> Result<(), ScenarioError> {
    let SExpr::List(items) = form else {
        return Err(err("a defvocabulary must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::BareUpperIdent(kind_name)), SExpr::List(member_items)] =
        items.as_slice()
    else {
        return Err(err("expected (defvocabulary <enum-kind> (<enum-member>+))"));
    };
    if head != "defvocabulary" {
        return Err(err(format!("expected (defvocabulary …), found ({head} …)")));
    }
    let Some(kind) = EnumKind::from_type_name(kind_name) else {
        return Err(coded_err(
            "E-LOAD-030",
            format!(
                "defvocabulary: `{kind_name}` is not one of NodeType / \
                 EdgeType / HyperedgeType / EventType (§3.6)"
            ),
        ));
    };
    if !declared.insert(kind) {
        return Err(coded_err(
            "E-LOAD-001",
            format!(
                "duplicate defvocabulary for {kind_name} — a kind's \
                 vocabulary is declared once (§2.13)"
            ),
        ));
    }
    let mut members = Vec::with_capacity(member_items.len());
    for item in member_items {
        let SExpr::Atom(Atom::BareUpperIdent(member)) = item else {
            return Err(err(format!(
                "defvocabulary {kind_name}: member {item:?} must be a bare \
                 <enum-member> (§2.13, §1.4) — never a full \
                 `{kind_name}/<MEMBER>` enum-ref"
            )));
        };
        if !crate::reader::is_enum_member_shape(member) {
            return Err(err(format!(
                "defvocabulary {kind_name}: member `{member}` is not a valid \
                 <enum-member> (§1.4: UPPER (UPPER|DIGIT|\"_\")* — no lowercase)"
            )));
        }
        members.push(member.clone());
    }
    collected.entry(kind).or_default().extend(members);
    Ok(())
}

/// `(deffield <qname> <type-symbol> <kind-symbol>)`, or — since §2.13
/// (D101, Organization spec §1 Q12) — `(deffield <qname> enum
/// <EnumTypeName>)`: the 4th slot holds the enum type name instead of a
/// kind symbol, because an enum-typed field carries no aggregation kind
/// at all (there is no `intensive`/`extensive` reading of a member
/// identity) — this dialect has no separate `:enum-type` keyword the way
/// the `.bsl` `deffield` production does; the type-symbol slot itself
/// being `enum` is what selects the 4th slot's alternate meaning.
///
/// The `deffield` registry in miniature. A field's TYPE and INTENSIVITY KIND
/// cannot be inferred from a stored value — `120` is an `Int` whether it is a
/// head-count that sums or a rate that does not — and §3.4 exists precisely
/// to stop that being guessed. So the scenario declares them.
fn load_deffield(
    parts: &[SExpr],
    fields: &mut HashMap<String, FieldDecl>,
    enums: &EnumRegistry,
) -> Result<(), ScenarioError> {
    let [_, SExpr::Atom(Atom::QName(qname)), SExpr::Atom(Atom::Symbol(ty)), fourth] = parts else {
        return Err(err(
            "expected (deffield <field-qname> <type> <intensive|extensive>) or \
             (deffield <field-qname> enum <EnumTypeName>)",
        ));
    };
    let decl = if ty == "enum" {
        let SExpr::Atom(Atom::BareUpperIdent(type_name)) = fourth else {
            return Err(err(format!(
                "deffield `{qname}`: an enum-typed field's 4th slot is the \
                 declared enum type name — (deffield {qname} enum \
                 <EnumTypeName>), found {fourth:?}"
            )));
        };
        let Some(id) = enums.resolve(type_name) else {
            return Err(coded_err(
                "E-LOAD-054",
                format!(
                    "deffield `{qname}`: enum type `{type_name}` was never \
                     declared — add a (defenum {type_name} (…)) form ABOVE \
                     this deffield"
                ),
            ));
        };
        FieldDecl {
            ty: BslType::Enum(id),
            kind: FieldKind::NotApplicable,
        }
    } else {
        let SExpr::Atom(Atom::Symbol(kind)) = fourth else {
            return Err(err(format!(
                "deffield `{qname}`: expected an intensive|extensive kind \
                 symbol, found {fourth:?}"
            )));
        };
        let ty = match ty.as_str() {
            "int" => BslType::Int,
            "real" => BslType::Real,
            "probability" => BslType::Probability,
            "intensity" => BslType::Intensity,
            "coefficient" => BslType::Coefficient,
            "currency" => BslType::Currency,
            other => {
                return Err(err(format!(
                    "deffield `{qname}`: unknown type `{other}` — one of \
                     int / real / probability / intensity / coefficient / currency / enum"
                )))
            }
        };
        let kind = match kind.as_str() {
            "intensive" => FieldKind::Intensive,
            "extensive" => FieldKind::Extensive,
            other => {
                return Err(err(format!(
                    "deffield `{qname}`: unknown kind `{other}` — intensive or extensive. \
                     An intensive field averaged without an extensive weight is the \
                     variance error §3.4 exists to catch, so this is not optional"
                )))
            }
        };
        FieldDecl { ty, kind }
    };
    if fields.insert(qname.clone(), decl).is_some() {
        return Err(err(format!(
            "duplicate deffield `{qname}` — a field has one declared type and kind"
        )));
    }
    Ok(())
}

/// `(node <local-name> <enum-ref> (<qname> <int>)*)`
fn load_node(
    parts: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    named: &mut HashMap<String, NodeId>,
    declared: &HashMap<String, FieldDecl>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<String, ScenarioError> {
    let [_, SExpr::Atom(Atom::Symbol(local)), SExpr::Atom(Atom::EnumRef { enum_type, member }), attrs @ ..] =
        parts
    else {
        return Err(err(
            "expected (node <local-name> <NodeType/MEMBER> (<field-qname> <int>)*)",
        ));
    };
    if named.contains_key(local) {
        return Err(err(format!(
            "duplicate scenario name `{local}` — a local name denotes exactly one node, \
             and silently rebinding it would make later edges ambiguous"
        )));
    }
    // F2 (#534 fix round item 2): a node's own enum-ref position demands
    // NodeType — a STATIC fact about the written type name, independent of
    // whether any `defvocabulary` was declared at all (mirrors
    // `grammar::check_enum_ref_kinds`'s D74 class rule for rules, which is
    // ALSO unconditional). Without this, `(node x EdgeType/SOLIDARITY)`
    // minted a node silently typed SOLIDARITY under a declared vocabulary
    // that registers `EdgeType/SOLIDARITY` — `check_enum_ref`'s returned
    // `EnumKind` was discarded and never compared against the position it
    // came from (panel-proven, mutation-reproduced).
    demand_enum_kind(enum_type, member, EnumKind::NodeType, enums)
        .map_err(|e| vocab_err(format!("node `{local}`"), &e))?;
    // Task 8 (Organization foundation plan): the scenario-load half of
    // closed-vocabulary enforcement — checked BEFORE minting, so a typo'd
    // type never even reaches the substrate. `None` (no `defvocabulary`
    // declared, or none declared YET — declaration must precede use, same
    // as `deffield`/`defenum`/`defconst`) is exactly today's unchecked
    // behavior (Task 7's backward-compatibility proof).
    if let Some(vocabulary) = vocabulary {
        vocabulary
            .check_enum_ref(enum_type, member)
            .map_err(|e| vocab_err(format!("node `{local}`"), &e))?;
    }

    // The node type string is the enum MEMBER verbatim, matching what
    // `(add-node NodeType/SOCIAL_CLASS ...)` stamps in the verb layer. Any
    // other convention here would produce nodes production queries cannot see.
    let id = graph.add_node(member)?;
    named.insert(local.clone(), id);

    for attr in attrs {
        let SExpr::List(pair) = attr else {
            return Err(err(format!(
                "node `{local}`: an attribute is a (<field-qname> <int>) form"
            )));
        };
        let [SExpr::Atom(Atom::QName(field)), SExpr::Atom(value)] = pair.as_slice() else {
            return Err(err(format!(
                "node `{local}`: an attribute is a (<field-qname> <int>) form"
            )));
        };
        // The registry contract, ENFORCED rather than merely documented: an
        // undeclared qname is a typo, and accepting it would silently mint a
        // field no typechecker knows about — the rule that meant to read it
        // would then fail far from the mistake. Declaration must precede
        // use, so a scenario reads top to bottom exactly as its edges do.
        let Some(decl) = declared.get(field) else {
            return Err(err(format!(
                "node `{local}`: field `{field}` was never declared — add a \
                 (deffield {field} <type> <intensive|extensive>) form ABOVE this node"
            )));
        };
        let descriptor = format!("node `{local}`");
        // T3 #491, OQ-J: a `currency`-declared field routes to the i128
        // lane — a SEPARATE trait method and a SEPARATE store map, never
        // `update_node`'s f64 one. Every other declared type is unchanged
        // (`attribute_value`'s own dispatch, below).
        if matches!(decl.ty, BslType::Currency) {
            graph.update_node_currency(
                id,
                field,
                attribute_value_currency(value, &descriptor, field)?,
            )?;
        } else {
            graph.update_node(
                id,
                field,
                // `attribute_value`'s first parameter is the ELEMENT
                // DESCRIPTOR, so the noun — quoting included — travels from
                // the call site: the node path passes "node `…`" here, the
                // edge-attr path passes "edge (… → …)", and the family's
                // format strings name `{local}` BARE. Emitted text for this
                // path is byte-identical to the pre-descriptor rendering,
                // one wording apart: the unreachable defense-in-depth arm's
                // "node attributes" generalizes to "attributes", correct
                // for both element kinds (Train B item 3, #591).
                attribute_value(value, &descriptor, field, decl, enums)?,
            )?;
        }
    }
    Ok(member.clone())
}

/// Attribute values: `int` literals into `int`-declared fields (unchanged
/// since slice 1), and — Half 1 of the typed-attribute-seeding design
/// (`reports/typed-attribute-seeding-design-2026-08-11.md`) — a fractional
/// literal into a `probability`/`intensity`/`coefficient`-declared field.
///
/// The declaration is checked, not just consulted. A `120` written into a
/// field declared `intensity` is out of that type's `[0, 1]` domain — the
/// store lying about what it holds. A `currency`-declared field never
/// reaches this function's `BslType::Currency` arm from the NODE path
/// (`load_node` dispatches to [`attribute_value_currency`] before calling
/// here, T3 #491/OQ-J); it is still reachable from the EDGE path
/// ([`load_edge_attr`]), which this function refuses — there is no
/// edge-scoped Currency lane in this train.
///
/// **Half 1 needs no new typed storage.** `GraphSubstrate` attributes are
/// already `f64` in and out (`rust/crates/babylon-graph/src/substrate.rs`);
/// nothing about the trait restricts values to integers, and
/// `CanonicalState`'s section `0x02`
/// (`rust/crates/babylon-graph/src/state_hash.rs`) is a bare `f64`
/// regardless of which `BslType` declared the field — a `0.358` seeded into a
/// `coefficient`-declared field and one seeded into an (illegally)
/// `int`-declared field hash byte-identically. The restriction lived
/// entirely in this function; widening it changes zero bytes for any
/// existing scenario, since every one seeds only `int` fields today.
///
/// `local` is the ELEMENT DESCRIPTOR for error text, noun and quoting
/// included — `load_node` passes "node \`core\`", `load_edge_attr` passes
/// "edge (a → b)", and this family's format strings name `{local}` BARE, so
/// the node path's emitted text is byte-identical to the pre-descriptor
/// rendering and no refusal calls an edge a "node" (Train B item 3, #591;
/// the same form-parameter convention `vocab_err` and `evaluator.rs`'s
/// `check_*_referent_type` already model).
fn attribute_value(
    atom: &Atom,
    local: &str,
    field: &str,
    decl: &FieldDecl,
    enums: &EnumRegistry,
) -> Result<f64, ScenarioError> {
    match &decl.ty {
        BslType::Int => attribute_value_int(atom, local, field),
        BslType::Real => attribute_value_real(atom, local, field),
        BslType::Probability | BslType::Intensity | BslType::Coefficient => {
            attribute_value_unit_interval(atom, local, field, &decl.ty)
        }
        BslType::Currency => Err(err(currency_edge_unsupported_message(local, field))),
        BslType::Enum(ty) => attribute_value_enum(atom, local, field, *ty, enums),
        // Defense in depth, not a reachable content error: `load_deffield`
        // is the SOLE populator of the `declared` map `attribute_value` is
        // called against, and its own match on the type symbol admits only
        // int/real/probability/intensity/coefficient/currency/enum (anything
        // else is refused AT DECLARATION, before a `node` form naming the
        // field can even be read). Reaching here is a wiring bug in the
        // deffield parser, not a content error — kept anyway because
        // `BslType` is not a closed match the compiler can prove exhaustive
        // against this function's actual call graph, and a silent
        // `unreachable!()` would panic rather than name the field that
        // triggered it.
        other => Err(err(format!(
            "{local}: field `{field}` is declared {other:?}, and the scenario \
             loader stores only `int`, `real`, `probability`, `intensity`, `coefficient` or \
             `enum`-declared attributes through this function (currency is refused \
             separately here, or routed to node-scoped typed storage by the caller) \
             — {other:?} has no representation as a GraphSubstrate f64 attribute at all"
        ))),
    }
}

/// `enum`-declared fields (§2.13, D101 — the Q12 enum row's own lane in the
/// Half-1 typed-attribute-seeding design). Accepts **only** a matching
/// `<enum-ref>` atom, resolves its member through `enums`, and stores
/// `ordinal as f64` — the SAME binary64 attribute lane every other declared
/// type here already uses (zero bytes of any existing golden move: no
/// existing scenario declares an enum field). The ordinal is never a
/// surface value: a bare number, an `<enum-ref>` of a different declared
/// type, or any other atom is `E-LOAD-056`; an `<enum-ref>` of the RIGHT
/// type naming a member that type does not declare is `E-LOAD-055` — there
/// is no "seed it as a number and let the engine resolve the member" path
/// and no default member (§3.6's own "a name outside the registry is a
/// load error, never a fallback" law, restated here for the content-declared
/// registry).
fn attribute_value_enum(
    atom: &Atom,
    local: &str,
    field: &str,
    ty: EnumTypeId,
    enums: &EnumRegistry,
) -> Result<f64, ScenarioError> {
    let Atom::EnumRef { enum_type, member } = atom else {
        return Err(coded_err(
            "E-LOAD-056",
            format!(
                "{local} field `{field}`: an enum-typed field is seeded \
                 ONLY as <EnumType>/<MEMBER> — the ordinal is never a surface \
                 value; found {atom:?}"
            ),
        ));
    };
    let declared_type = enums.name(ty);
    if enum_type != declared_type {
        return Err(coded_err(
            "E-LOAD-056",
            format!(
                "{local} field `{field}`: declared enum type is \
                 {declared_type}, found {enum_type}/{member} — an <enum-ref> \
                 of a different declared type is exactly as illegal as a bare \
                 number here"
            ),
        ));
    }
    let Some(ordinal) = enums.ordinal(ty, member) else {
        return Err(coded_err(
            "E-LOAD-055",
            format!(
                "{local} field `{field}`: {declared_type} has no \
                 member {member} — never a default"
            ),
        ));
    };
    Ok(f64::from(ordinal))
}

/// `int`-declared fields — unchanged behavior, Half 1 does not touch this arm.
fn attribute_value_int(atom: &Atom, local: &str, field: &str) -> Result<f64, ScenarioError> {
    match atom {
        Atom::Int(value) => {
            // Exact in f64 to 2^53; past that the stored value would differ
            // from the declared one, which is a lie the state hash would
            // faithfully record.
            if value.unsigned_abs() > (1_u64 << 53) {
                return Err(err(format!(
                    "{local} field `{field}`: {value} exceeds f64's exact integer range"
                )));
            }
            #[allow(clippy::cast_precision_loss)]
            Ok(*value as f64)
        }
        Atom::Currency(_) => Err(err(currency_refusal_message(local, field))),
        other => Err(err(format!(
            "{local} field `{field}`: expected an integer literal, found {other:?}"
        ))),
    }
}

/// `real`-declared fields (Train B item 6). Accepts the three literal lanes
/// whose own lex laws already bound them — int (exact to 2^53, the same
/// guard `attribute_value_int` states), p/i/c (`[0,1]` at lex), r (`(0,∞)`
/// at lex) — each converted by the crate's one scaled-literal contract
/// (`unscaled / 10^scale`). Currency is refused (the same deferral every
/// other arm states). There is NO arbitrary-precision fractional literal:
/// E-LEX-021 still refuses bare floats; that is #591 item 5's territory,
/// not this train's.
fn attribute_value_real(atom: &Atom, local: &str, field: &str) -> Result<f64, ScenarioError> {
    match atom {
        Atom::Int(value) => {
            if value.unsigned_abs() > (1_u64 << 53) {
                return Err(err(format!(
                    "{local} field `{field}`: {value} exceeds f64's exact integer range"
                )));
            }
            #[allow(clippy::cast_precision_loss)]
            Ok(*value as f64)
        }
        Atom::Scaled(scaled) => {
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            Ok(numerator / 10_f64.powi(i32::from(scaled.scale)))
        }
        Atom::Currency(_) => Err(err(currency_refusal_message(local, field))),
        other => Err(err(format!(
            "{local} field `{field}`: expected an int, p/i/c or r literal for a \
             real field, found {other:?}"
        ))),
    }
}

/// `probability`/`intensity`/`coefficient`-declared fields (Half 1).
///
/// Mirrors `structural_verbs.rs::store_range_check`'s runtime predicate
/// VERBATIM, one call frame earlier — not a new invented rule, the same
/// `[0,1]` rule the runtime write boundary already lives by:
/// `matches!(decl.ty, Probability | Intensity | Coefficient)` then
/// `(0.0..=1.0).contains(&value)`. That runtime check is itself kind-blind
/// among the three unit-interval types — `Value::Real` carries no `p`/`i`/`c`
/// tag once evaluated, so a rule computing a value for a `coefficient` field
/// is checked by the exact same predicate as one for a `probability` field —
/// so this load-time mirror is equally kind-blind: an `Int` literal or any
/// `p`/`i`/`c`-suffixed literal is accepted for ANY of the three declared
/// types, provided its magnitude is in range. `Ratio` (`r`-suffixed) is
/// EXCLUDED even though its value may numerically fall in `[0,1]` for a
/// given literal — `Ratio` is a genuinely distinct runtime `Value` variant
/// with its own `(0, ∞)` domain and its own restricted operator
/// (`Currency × Ratio` only, §3.2 addendum); a field read can never legally
/// produce one (`bind_subject` wraps every `:field` read `Value::Real`,
/// never `Value::Ratio`), so admitting one here would store a value under a
/// type the read path cannot represent.
///
/// # The conversion contract, and why it is NOT `babylon_kernel::grid::quantize`
///
/// A scaled (`p`/`i`/`c`) literal converts as `unscaled / 10^scale` — one
/// IEEE-754 division of two exactly-representable operands (`unscaled` as an
/// integer, `10^scale` exact in `f64` for `scale <= 9`, the literal's own
/// bound) — the SAME arithmetic `tick.rs::atom_to_value` and
/// `scenario.rs::load_defconst` already perform for a `:default`/`:const`
/// literal of the same value. This is deliberate, not an oversight: `grid.rs`'s
/// `(value * 1e6 + 0.5).floor() / 1e6` half-up quantization is a
/// `babylon_kernel` **scalar newtype** invariant
/// (`Probability::new`/`Intensity::new`/`Coefficient::new` call it on
/// construction) that nothing in `babylon-bsl` reaches today — the
/// evaluator's binary64 lane is raw `f64` (`Value::Real`) end to end, and a
/// value entering storage via `(update-node self field (+ x y))` during a
/// tick is not grid-quantized either. Snapping only the seed path to the
/// grid would create exactly the asymmetry it would be trying to prevent: a
/// rule-computed write and a scenario-seeded value of the identical field
/// would then follow two different rounding rules for the same declared
/// type. A single correctly-rounded IEEE-754 division is a "basic IEEE-754
/// op" — it reproduces bit-identically across conforming implementations,
/// the same guarantee class CLAUDE.md's Tests-as-Behavioral-Contracts
/// principle 4 names as safe ("basic IEEE-754 ops reproduce across
/// languages") — so no new rule is minted; the existing rule is extended
/// verbatim to the node-attribute seed path.
fn attribute_value_unit_interval(
    atom: &Atom,
    local: &str,
    field: &str,
    ty: &BslType,
) -> Result<f64, ScenarioError> {
    let value = match atom {
        Atom::Int(value) => {
            // Bare Int literals carry NO domain check at lex time
            // (`E-LEX-024` bounds only `p`/`i`/`c`-suffixed literals) — the
            // `[0,1]` check below is this arm's ONLY domain enforcement, and
            // it is load-bearing: an out-of-range bare Int (e.g. `6`) must
            // still be refused here.
            #[allow(clippy::cast_precision_loss)]
            let widened = *value as f64;
            widened
        }
        Atom::Scaled(scaled) if scaled.kind == ScaledKind::Ratio => {
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            let value = numerator / 10_f64.powi(i32::from(scaled.scale));
            return Err(err(format!(
                "{local} field `{field}`: {value}r is a Ratio (r-suffixed) literal, \
                 not a legal {ty:?} attribute value — Ratio is its own runtime type with \
                 domain (0, ∞), and a :field read can never legally produce one"
            )));
        }
        Atom::Scaled(scaled) => {
            // See this function's doc comment for the conversion contract
            // and why it is not `grid::quantize`.
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            numerator / 10_f64.powi(i32::from(scaled.scale))
        }
        Atom::Currency(_) => return Err(err(currency_refusal_message(local, field))),
        other => {
            return Err(err(format!(
                "{local} field `{field}`: expected an int or scaled (p/i/c) \
                 literal, found {other:?}"
            )))
        }
    };
    if !(0.0..=1.0).contains(&value) {
        return Err(err(format!(
            "{local} field `{field}`: storing {value} leaves its declared \
             {ty:?} [0,1] domain — a loud failure, never a clamp (mirrors \
             structural_verbs.rs::store_range_check's runtime rule, checked one \
             call frame earlier)"
        )));
    }
    Ok(value)
}

/// `currency`-declared fields (T3 #491, OQ-J — Half 2 of the
/// typed-attribute-seeding design). Accepts ONLY a `$`-suffixed literal —
/// `load_node`'s caller routes this to
/// [`babylon_graph::substrate::GraphSubstrate::update_node_currency`], the
/// i128 lane, never the ordinary f64 attribute map. No domain re-check is
/// needed here: `$` literals are already lex-bound to `[0, ∞)`
/// (`E-LEX-022` refuses a negative one at parse time,
/// `docs/reference/bsl-language.rst` §1.5's Currency row) — the SAME
/// guarantee `attribute_value_int`'s 2^53 comment leans on for `Int`,
/// applied to a domain instead of a range. Anything else — an `Int`, a
/// scaled `p`/`i`/`c`/`r` literal, an enum-ref — is refused, naming what
/// was found: those literal lanes exist for the OTHER declared types, and
/// admitting one here would let a `currency`-declared field silently widen
/// a non-Currency literal into i128 micro-units, which is exactly the
/// "entry point decides the type system" drift the sibling `attribute_value_*`
/// functions all refuse.
fn attribute_value_currency(
    atom: &Atom,
    local: &str,
    field: &str,
) -> Result<Currency, ScenarioError> {
    match atom {
        Atom::Currency(c) => Ok(*c),
        other => Err(err(format!(
            "{local} field `{field}`: a currency-declared field is seeded ONLY by a \
             $-suffixed Currency literal, found {other:?}"
        ))),
    }
}

/// The KIND-MISMATCH refusal: a `$`-suffixed (Currency) literal written
/// into a field NOT declared `currency` — worded identically at every site
/// it fires (`attribute_value_int`/`_real`/`_unit_interval`).
///
/// Wording note (T3 #491, OQ-J — Half 2 landed): earlier revisions of this
/// message said Currency attributes "need typed attribute storage" at all,
/// because none existed. That storage now exists
/// ([`babylon_graph::substrate::GraphSubstrate::update_node_currency`]), so
/// the honest reason a `$` literal is refused HERE is narrower: this
/// particular field is declared something else, and f64 cannot hold i128
/// micro-units without lying about what it stores — never a lossy cast.
fn currency_refusal_message(local: &str, field: &str) -> String {
    format!(
        "{local} field `{field}`: a Currency ($-suffixed) literal needs a \
         currency-declared field — this field is declared something else, \
         and f64 cannot hold i128 micro-units without lying about what it \
         stores, so this refuses rather than casting lossily"
    )
}

/// The EDGE-SCOPE refusal: a field genuinely declared `currency`, targeted
/// through the edge-attribute path (`load_edge_attr`) — Currency's typed
/// storage is node-scoped ONLY in this train (T3 #491, OQ-J); there is no
/// `update_edge_currency`. Distinct wording from
/// [`currency_refusal_message`] on purpose: this is not a kind mismatch,
/// it is a real, named scope gap.
fn currency_edge_unsupported_message(local: &str, field: &str) -> String {
    format!(
        "{local} field `{field}`: currency-declared fields have typed storage \
         for NODE attributes only (T3 #491, OQ-J) — there is no edge-scoped \
         Currency lane, so this refuses rather than casting lossily into the \
         f64 edge-attribute store"
    )
}

/// `(edge <enum-ref> <local-name> <local-name> <strength>)` — `<strength>`
/// is an int or a `p`/`i`/`c`-suffixed unit-interval literal (kind-blind,
/// mirroring the runtime `:strength` position; D32 kinds the field
/// Coefficient; T2 plan Task 6a) — returns the minted
/// `EdgeType` member (verbatim, matching `load_node`'s own return
/// convention) so the caller can build the `edge_types` census.
fn load_edge(
    parts: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    named: &HashMap<String, NodeId>,
    seeded: &mut HashSet<(String, NodeId, NodeId)>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<String, ScenarioError> {
    let [_, SExpr::Atom(Atom::EnumRef { enum_type, member }), SExpr::Atom(Atom::Symbol(from)), SExpr::Atom(Atom::Symbol(to)), SExpr::Atom(strength)] =
        parts
    else {
        return Err(err(
            "expected (edge <EdgeType/MEMBER> <from-name> <to-name> <strength: int | p/i/c-lit>)",
        ));
    };
    // F6 (#534 fix round item 6): an edge form has no local name of its
    // own (unlike a node) — its endpoints are what identifies it in a
    // refusal message.
    let form = format!("edge ({from} → {to})");
    // F2 (#534 fix round item 2): see `load_node`'s identical comment —
    // the edge position demands EdgeType, unconditionally.
    demand_enum_kind(enum_type, member, EnumKind::EdgeType, enums)
        .map_err(|e| vocab_err(&form, &e))?;
    // Task 8 (Organization foundation plan): see `load_node`'s identical
    // comment — the same check, before the substrate's own `add_edge`.
    if let Some(vocabulary) = vocabulary {
        vocabulary
            .check_enum_ref(enum_type, member)
            .map_err(|e| vocab_err(&form, &e))?;
    }
    let resolve = |name: &String| -> Result<NodeId, ScenarioError> {
        named.get(name).copied().ok_or_else(|| {
            err(format!(
                "edge names unknown node `{name}` — a node must be declared before an \
                 edge referring to it, so a scenario reads top to bottom"
            ))
        })
    };
    // An edge strength is not a node field, so no deffield governs it; the
    // literal restriction is stated directly here: an integer, or a
    // p/i/c-suffixed unit-interval literal — KIND-BLIND among the three,
    // mirroring both attribute_value_unit_interval (kinds do not survive
    // evaluation; its doc records the choice) and the runtime writer of
    // this exact position (structural_verbs.rs::add_edge accepts any
    // Value::Real at :strength). D32 kinds the field Coefficient; the
    // loader does not narrow to c-only because the runtime cannot (T2 plan
    // Task 6a, amendment 2026-08-12, adjudicated in its fix round). Ratio
    // (r) stays out for the node path's own recorded reason — Value::Ratio
    // is not the binary64 lane, and the runtime :strength match refuses it
    // identically. No range check on either arm — ints were never checked
    // here (an OPEN asymmetry, recorded in the T2 plan's Task 6a design
    // decision 3, not silently resolved), and a p/i/c literal is already
    // [0,1]-bounded at lex (E-LEX-024).
    let strength = match strength {
        Atom::Int(value) => {
            #[allow(clippy::cast_precision_loss)]
            let widened = *value as f64;
            widened
        }
        Atom::Scaled(scaled)
            if matches!(
                scaled.kind,
                ScaledKind::Probability | ScaledKind::Intensity | ScaledKind::Coefficient
            ) =>
        {
            // `unscaled / 10^scale` — the crate's one scaled-literal
            // conversion contract (attribute_value_unit_interval's doc
            // comment is its normative home), copied verbatim.
            #[allow(clippy::cast_precision_loss)]
            let numerator = scaled.unscaled as f64;
            numerator / 10_f64.powi(i32::from(scaled.scale))
        }
        other => {
            return Err(err(format!(
                "edge {member}: expected an integer or p/i/c-suffixed \
                 unit-interval strength literal, found {other:?}"
            )))
        }
    };
    let (from_id, to_id) = (resolve(from)?, resolve(to)?);
    // §3.9 clause 5 / D73 — reported as E-LOAD-044 HERE rather than left to
    // the substrate's generic refusal, because the two failures are
    // different facts: `E-EVAL-031` is a verb adding an edge that already
    // exists, and `E-LOAD-044` is a scenario seeding one key twice.
    if !seeded.insert((member.clone(), from_id, to_id)) {
        return Err(coded_err(
            "E-LOAD-044",
            format!(
                "hydration seeds two {member} edges between one ordered pair \
                 ({from} → {to}); the (source, target, type) triple is a KEY, \
                 which is what makes §2.6's edge order total and \
                 `edge-between` well defined"
            ),
        ));
    }
    graph.add_edge(member, from_id, to_id, strength)?;
    Ok(member.clone())
}

/// `(edge-attr <enum-ref> <local-name> <local-name> <field-qname> <value>)` —
/// Train B item 3 (#591, D156): seed one DECLARED field of an edge the same
/// scenario already minted, through the same binary64 attribute lane every
/// node field uses (`GraphSubstrate::update_edge`, the fifth-section store —
/// the D143 `/strength` fork never engages because the strength guard below
/// fires first). The value converts through [`attribute_value`], the crate's
/// ONE per-type literal law — Currency is STILL refused here (T3 #491,
/// OQ-J's typed storage is node-scoped only), now for a narrower, named
/// reason ([`currency_edge_unsupported_message`]) rather than "no storage
/// exists at all".
///
/// The reading law is `load_edge`'s own, one form later in the file: the
/// enum-ref demands `EdgeType` unconditionally, the declared vocabulary (if
/// any) checks membership, both endpoints resolve through `named`, and the
/// `(member, from, to)` triple must already be in `seeded` — an edge-attr
/// for an edge not yet seeded in THIS scenario is a loud refusal naming the
/// endpoints, the same top-to-bottom discipline as every other
/// declared-registry lookup here. Three further refusals, in order:
///
/// 1. The qname's OWNER segment must name the edge's own type — §2.10
///    discipline 1's ownership law, checked at hydration through the same
///    rendering (`tick::namespace_to_node_type`)
///    `evaluator.rs::check_edge_referent_type` uses at evaluation.
/// 2. A `/strength`-suffixed qname is refused UNCONDITIONALLY — not merely
///    via the registry miss (D32's implicit field is never in
///    `scenario.fields`): `load_deffield` would ACCEPT an explicit
///    `(deffield <edge-type>/strength …)` (only `prepare_rules`'s E-LOAD-001
///    refuses it, later, at `TypeEnv` construction), and without this guard
///    the write would fall into the substrate's `/strength` fork (D143) and
///    silently rewrite the edge's mint strength slot. One datum, one writer:
///    the `(edge …)` form's own 4th slot.
/// 3. An undeclared qname is a typo, not a new field — `load_node`'s
///    registry contract verbatim.
///
/// The `(edge-type, source, target, field)` quadruple is a KEY — a second
/// seeding of one key is `E-LOAD-057`, E-LOAD-044's own argument one axis
/// wider: the file would carry two values for one datum with only the later
/// surviving, a silent overwrite.
// Eight arguments, over clippy's seven: `parts`/`graph` are the form and its
// target, `named`/`seeded`/`seeded_attrs`/`declared`/`enums`/`vocabulary` are
// the six load-time registries the top-to-bottom law consults — the same
// shape `load_node`/`load_edge` already take, plus this form's own second
// seed set. Bundling them into a struct would rename, not reduce, them.
#[allow(clippy::too_many_arguments)]
fn load_edge_attr(
    parts: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    named: &HashMap<String, NodeId>,
    seeded: &HashSet<(String, NodeId, NodeId)>,
    seeded_attrs: &mut HashSet<(String, NodeId, NodeId, String)>,
    declared: &HashMap<String, FieldDecl>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<(), ScenarioError> {
    let [_, SExpr::Atom(Atom::EnumRef { enum_type, member }), SExpr::Atom(Atom::Symbol(from)), SExpr::Atom(Atom::Symbol(to)), SExpr::Atom(Atom::QName(field)), SExpr::Atom(value)] =
        parts
    else {
        return Err(err(
            "expected (edge-attr <EdgeType/MEMBER> <from-name> <to-name> <field-qname> <value>)",
        ));
    };
    // An edge-attr form has no local name of its own — its edge's endpoints
    // identify it (load_edge's own F6 convention).
    let form = format!("edge-attr ({from} → {to})");
    // The enum-ref position demands EdgeType, unconditionally — see
    // `load_node`'s identical comment (F2, #534 fix round item 2).
    demand_enum_kind(enum_type, member, EnumKind::EdgeType, enums)
        .map_err(|e| vocab_err(&form, &e))?;
    // See `load_node`'s identical comment (Task 8, Organization foundation
    // plan) — the same membership check, before anything is written.
    if let Some(vocabulary) = vocabulary {
        vocabulary
            .check_enum_ref(enum_type, member)
            .map_err(|e| vocab_err(&form, &e))?;
    }
    let resolve = |name: &String| -> Result<NodeId, ScenarioError> {
        named.get(name).copied().ok_or_else(|| {
            err(format!(
                "edge-attr names unknown node `{name}` — a node must be declared before an \
                 edge-attr referring to it, so a scenario reads top to bottom"
            ))
        })
    };
    let (from_id, to_id) = (resolve(from)?, resolve(to)?);
    // Edge-existence, against the SAME `seeded` set `load_edge` populates:
    // the edge must have been seeded ABOVE this form in this scenario — an
    // attribute write never mints an edge, exactly as `update_edge` never
    // mints an attribute row for one that does not exist.
    if !seeded.contains(&(member.clone(), from_id, to_id)) {
        return Err(err(format!(
            "edge-attr ({from} → {to}): no such edge — no {member} edge between this ordered \
             pair was seeded ABOVE this form; an edge must exist before its attributes are \
             seeded, so a scenario reads top to bottom"
        )));
    }
    // §2.10 discipline 1 at hydration: the qname's owner segment must name
    // the edge's own type, through the same rendering
    // `check_edge_referent_type` uses at evaluation.
    let owner_segment = field.split('/').next().unwrap_or(field);
    let owner_type = crate::tick::namespace_to_node_type(owner_segment);
    if owner_type != *member {
        return Err(err(format!(
            "edge-attr ({from} → {to}): field `{field}` is owned by {owner_type}, not \
             {member} — an edge attribute's qname owner must name the edge's own type \
             (§2.10 discipline 1, checked at hydration exactly as \
             `check_edge_referent_type` checks it at evaluation)"
        )));
    }
    // The strength guard — see this function's doc, refusal 2.
    if field.ends_with("/strength") {
        return Err(err(format!(
            "edge-attr ({from} → {to}): `{field}` — strength seeds via the (edge ...) form's \
             own 4th slot only. D32 kinds <edge-type>/strength an IMPLICIT field (never in a \
             scenario's deffield registry), and the substrate's /strength write fork (D143) \
             would route this write onto the edge's existing strength slot — a silent rewrite \
             of the mint datum, never a second home"
        )));
    }
    // The registry contract, ENFORCED — load_node's undeclared-field refusal
    // verbatim, one element kind over.
    let Some(decl) = declared.get(field) else {
        return Err(err(format!(
            "edge-attr ({from} → {to}): field `{field}` was never declared — add a \
             (deffield {field} <type> <intensive|extensive>) form ABOVE the node and edge \
             forms that use it"
        )));
    };
    // The ONE per-type literal law — Currency refusal included. `attribute_value`
    // takes the element descriptor, so the noun is honest on this path too.
    let converted = attribute_value(value, &format!("edge ({from} → {to})"), field, decl, enums)?;
    // §3.9 clause 7 / D156 — E-LOAD-044's key argument one axis wider.
    if !seeded_attrs.insert((member.clone(), from_id, to_id, field.clone())) {
        return Err(coded_err(
            "E-LOAD-057",
            format!(
                "hydration seeds the {member} edge ({from} → {to})'s attribute `{field}` \
                 twice; the (edge-type, source, target, field) quadruple is a KEY, exactly \
                 as E-LOAD-044's triple is — a second seeding silently overwrites the first, \
                 the file carrying two values for one datum with only the later surviving"
            ),
        ));
    }
    graph.update_edge(member, from_id, to_id, field, converted)?;
    Ok(())
}

/// `(hyperedge <local-name> <HyperedgeType/MEMBER> (members <local-name>+))` —
/// hyperedge-lane Task 1 (`docs/superpowers/plans/2026-08-18-community-port.md`).
/// Mirrors `load_node`: the enum-ref position demands `HyperedgeType`
/// unconditionally (`demand_enum_kind`, same as `node`/`edge`), the declared
/// vocabulary (if any) checks membership — an unregistered member is
/// [`VocabularyError::UnknownEnumMember`] (`E-LOAD-031`), NOT
/// `UnknownFieldOwner` (`E-LOAD-023`, which fires only for a *field qname's*
/// first segment, `vocabulary.rs:145-150`) — and each member name resolves
/// through the SAME `named` local-name table `node`/`edge` share, so a
/// hyperedge may only name nodes this scenario already minted (declaration
/// must precede use, the same top-to-bottom discipline every other
/// declared-registry lookup in this loader already has).
///
/// **The empty-member-list refusal fires HERE, before the substrate is ever
/// called**, reproducing `MemoryGraph`/`HypergraphStore::add_hyperedge`'s own
/// wording verbatim (`babylon-graph/src/memory.rs:357-361`,
/// `hypergraph_store.rs:410-414`: "hyperedge must have at least one
/// member") — an explicit, load-time-local check rather than a generic
/// `GraphError` bubble-up, the same reasoning `load_edge`'s `E-LOAD-044`
/// check states for its own pre-substrate refusal: a scenario-authoring
/// mistake is a different fact from a runtime verb's existence-discipline
/// error, even when the two substrates would refuse the empty case anyway.
///
/// **Member canonicalization mirrors the executor's already-landed law
/// verbatim** (`structural_verbs.rs`'s `EffectExecutor::add_hyperedge`,
/// `:1361-1366`: "Membership is a SET and declared member order is never
/// observable … Canonicalize HERE so the write log cannot become the one
/// surface that leaks source order back"). `members.sort_unstable()` runs
/// HERE, before `GraphSubstrate::add_hyperedge` is called, exactly where the
/// executor runs it — both `MemoryGraph` and `HypergraphStore` ALSO sort
/// on insert (defense in depth), but the loader does not rely on that: a
/// third substrate is free to trust caller order (the trait is "silent on
/// representation"), so the loader's own sort is the one guarantee that
/// does not depend on which store is behind the trait object. Duplicates
/// stay the substrate's error to raise — sorting does not mask them, same
/// as the executor's own comment states.
///
/// **Fix round 1 (review I3): duplicate local names refuse loudly**, the
/// same law `load_node:1249-1254`'s own duplicate check states — checked
/// against `seeded_hyperedge_names`, a SEPARATE table from `named` (nodes
/// and hyperedges are different kinds, Amendment D; a hyperedge name is
/// never resolvable as a member — `named` only ever holds `NodeId`s). Inert
/// today (nothing yet resolves a hyperedge by its declared name), but
/// Task 6's `(hyperedge-attr <name> …)` will resolve by exactly this name,
/// at which point a silently-accepted duplicate would attribute the wrong
/// hyperedge rather than refuse.
///
/// Returns the minted `HyperedgeType` member (verbatim, matching
/// `load_node`/`load_edge`'s own return convention) plus the seeded member
/// count, so the caller can build BOTH `LoadedScenario::hyperedge_types` and
/// `LoadedScenario::max_members_seen` from one call.
fn load_hyperedge(
    parts: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    named: &HashMap<String, NodeId>,
    seeded_hyperedge_names: &mut HashMap<String, HyperedgeId>,
    enums: &EnumRegistry,
    vocabulary: Option<&ClosedVocabulary>,
) -> Result<(String, u64), ScenarioError> {
    let [_, SExpr::Atom(Atom::Symbol(local)), SExpr::Atom(Atom::EnumRef { enum_type, member }), members_form] =
        parts
    else {
        return Err(err(
            "expected (hyperedge <local-name> <HyperedgeType/MEMBER> \
             (members <local-name>+))",
        ));
    };
    // load_node:1249-1254's identical check, one element kind over — a
    // local name denotes exactly one hyperedge, and silently rebinding it
    // would make a later `hyperedge-attr` form (Task 6) ambiguous about
    // which hyperedge it targets.
    if seeded_hyperedge_names.contains_key(local) {
        return Err(err(format!(
            "duplicate scenario name `{local}` — a local name denotes exactly one \
             hyperedge, and silently rebinding it would make later hyperedge-attr \
             forms ambiguous"
        )));
    }
    // F2/G2's identical comment, load_node's own — see there for the full
    // argument: the position demands HyperedgeType unconditionally, then
    // (opt-in, Task 7's backward-compatibility proof) the declared
    // vocabulary checks membership.
    demand_enum_kind(enum_type, member, EnumKind::HyperedgeType, enums)
        .map_err(|e| vocab_err(format!("hyperedge `{local}`"), &e))?;
    if let Some(vocabulary) = vocabulary {
        vocabulary
            .check_enum_ref(enum_type, member)
            .map_err(|e| vocab_err(format!("hyperedge `{local}`"), &e))?;
    }
    let SExpr::List(member_parts) = members_form else {
        return Err(err(format!(
            "hyperedge `{local}`: expected a (members <local-name>+) form"
        )));
    };
    let [SExpr::Atom(Atom::Symbol(head)), member_names @ ..] = member_parts.as_slice() else {
        return Err(err(format!(
            "hyperedge `{local}`: expected a (members <local-name>+) form"
        )));
    };
    if head != "members" {
        return Err(err(format!(
            "hyperedge `{local}`: expected a (members <local-name>+) form, found ({head} ...)"
        )));
    }
    if member_names.is_empty() {
        return Err(err(format!(
            "hyperedge `{local}`: hyperedge must have at least one member — matches \
             GraphSubstrate::add_hyperedge's own refusal (babylon-graph/src/memory.rs:357-361)"
        )));
    }
    let mut members = Vec::with_capacity(member_names.len());
    for name_expr in member_names {
        let SExpr::Atom(Atom::Symbol(name)) = name_expr else {
            return Err(err(format!(
                "hyperedge `{local}`: a member is a bare local node name"
            )));
        };
        let id = named.get(name).copied().ok_or_else(|| {
            err(format!(
                "hyperedge `{local}` names unknown node `{name}` — a node must be \
                 declared before a hyperedge referring to it, so a scenario reads \
                 top to bottom"
            ))
        })?;
        members.push(id);
    }
    #[allow(clippy::cast_possible_truncation)]
    let member_count = members.len() as u64;
    // Member canonicalization is the executor's already-landed law — see
    // this function's doc comment.
    members.sort_unstable();
    let id = graph.add_hyperedge(member, &members)?;
    seeded_hyperedge_names.insert(local.clone(), id);
    Ok((member.clone(), member_count))
}

/// `(hyperedge-attr <local-name> <field-qname> <value>)` — Community port
/// train Task 6 (E2b): seed one DECLARED own-field of a hyperedge the same
/// scenario already minted, through the Task-5 substrate lane
/// (`GraphSubstrate::update_hyperedge_attribute`). Mirrors
/// [`load_edge_attr`]'s discipline one element kind over:
///
/// 1. The local name resolves through `seeded_hyperedge_names` — a
///    hyperedge must have been seeded ABOVE this form (the top-to-bottom
///    discipline every declared-registry lookup here shares), and the map
///    (not a set) is what Task 6 grew it into for exactly this resolution.
/// 2. §2.10 discipline 1 at hydration: the qname's OWNER segment must
///    render to the hyperedge's own declared type — read back from the
///    substrate (`hyperedge_type_of`), through the same
///    `tick::namespace_to_node_type` rendering
///    `evaluator.rs::check_hyperedge_referent_type` uses at evaluation.
/// 3. An undeclared qname is a typo, not a new field — `load_node`'s
///    registry contract verbatim.
/// 4. The value converts through [`attribute_value`], the crate's ONE
///    per-type literal law — the int/fractional contract and the enum
///    lane (`community/kind` seeded as `<CommunityType>/<MEMBER>`,
///    ordinal-stored) included; Currency refuses, there being no
///    hyperedge-scoped Currency lane (the same ruling as the edge lane).
///
/// The `(hyperedge, field)` pair is a KEY — a second seeding is
/// `E-LOAD-057`, the same silent-overwrite argument `load_edge_attr`'s
/// quadruple makes, one axis narrower.
fn load_hyperedge_attr(
    parts: &[SExpr],
    graph: &mut dyn GraphSubstrate,
    seeded: &HashMap<String, HyperedgeId>,
    seeded_attrs: &mut HashSet<(HyperedgeId, String)>,
    declared: &HashMap<String, FieldDecl>,
    enums: &EnumRegistry,
) -> Result<(), ScenarioError> {
    let [_, SExpr::Atom(Atom::Symbol(local)), SExpr::Atom(Atom::QName(field)), SExpr::Atom(value)] =
        parts
    else {
        return Err(err(
            "expected (hyperedge-attr <local-name> <field-qname> <value>)",
        ));
    };
    let form = format!("hyperedge-attr `{local}`");
    let id = seeded.get(local).copied().ok_or_else(|| {
        err(format!(
            "hyperedge-attr names unknown hyperedge `{local}` — a hyperedge must be \
             declared before a hyperedge-attr referring to it, so a scenario reads \
             top to bottom"
        ))
    })?;
    // §2.10 discipline 1 at hydration — refusal 2, checked against the
    // hyperedge's OWN seeded type (read back from the substrate, never
    // re-derived from the form, which carries no type operand).
    let actual_type = graph.hyperedge_type_of(id)?;
    let owner_segment = field.split('/').next().unwrap_or(field);
    let owner_type = crate::tick::namespace_to_node_type(owner_segment);
    if owner_type != actual_type {
        return Err(err(format!(
            "{form}: field `{field}` is owned by {owner_type}, not {actual_type} — a \
             hyperedge attribute's qname owner must name the hyperedge's own type \
             (§2.10 discipline 1, checked at hydration exactly as \
             `check_hyperedge_referent_type` checks it at evaluation)"
        )));
    }
    // The registry contract, ENFORCED — load_node's undeclared-field refusal
    // verbatim, one element kind over.
    let Some(decl) = declared.get(field) else {
        return Err(err(format!(
            "{form}: field `{field}` was never declared — add a \
             (deffield {field} <type> <intensive|extensive>) form ABOVE the \
             hyperedge and hyperedge-attr forms that use it"
        )));
    };
    // The ONE per-type literal law — Currency refusal included. `attribute_value`
    // takes the element descriptor, so the noun is honest on this path too.
    let converted = attribute_value(value, &format!("hyperedge `{local}`"), field, decl, enums)?;
    // E-LOAD-057, one axis narrower than the edge lane's quadruple.
    if !seeded_attrs.insert((id, field.clone())) {
        return Err(coded_err(
            "E-LOAD-057",
            format!(
                "hydration seeds hyperedge `{local}`'s attribute `{field}` twice; the \
                 (hyperedge, field) pair is a KEY, exactly as the edge lane's quadruple \
                 is — a second seeding silently overwrites the first, the file carrying \
                 two values for one datum with only the later surviving"
            ),
        ));
    }
    graph.update_hyperedge_attribute(id, field, converted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        invert_content_ids, load_scenario, load_scenario_with_named_preludes,
        load_scenario_with_prelude, BslType, EnumKind, EnumRegistry, ErrorIdentity, FieldDecl,
        FieldKind, NamedDeclarationPreludeV1,
    };
    use crate::bindings::BindingVocabulary;
    use crate::fuel::{CardinalityCeilings, IntrinsicCosts};
    use crate::intrinsic_host::EmptyIntrinsicHost;
    use crate::reader::read_all_spanned;
    use crate::rule_pipeline::{load_rule, LoadContext};
    use crate::structural_verbs::CollectingSink;
    use crate::tick::{run_tick, DefinesEnv};
    use crate::typecheck::TypeEnv;
    use babylon_graph::memory::MemoryGraph;
    use babylon_graph::state_hash::{CanonicalState, StateEncoder};
    use babylon_graph::substrate::{Direction, GraphSubstrate, HyperedgeId, NodeId};
    use babylon_kernel::{Currency, SessionId};
    use std::collections::{HashMap, HashSet};

    const TWO_CLASSES: &str = r"
(scenario ft/two-classes
  (deffield social-class/wages int extensive)
  (deffield social-class/value-produced int extensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/wages 120)
    (social-class/value-produced 80))
  (node periphery NodeType/SOCIAL_CLASS
    (social-class/wages 20)
    (social-class/value-produced 90))
  (edge EdgeType/SOLIDARITY core periphery 1))
";

    #[test]
    fn a_scenario_becomes_a_populated_graph() {
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(TWO_CLASSES, &mut graph).unwrap();
        assert_eq!(loaded.id, "ft/two-classes");
        assert_eq!(loaded.node_count, 2);
        assert_eq!(loaded.edge_count, 1);

        let classes = graph.nodes("SOCIAL_CLASS");
        assert_eq!(classes.len(), 2, "both classes exist and are queryable");
        assert!(
            (graph
                .node_attribute(classes[0], "social-class/wages")
                .unwrap()
                - 120.0)
                .abs()
                < 1e-12
        );
        assert_eq!(
            graph
                .neighbors(classes[0], "SOLIDARITY", Direction::Out)
                .unwrap(),
            vec![classes[1]],
            "the edge resolved both endpoints by local name"
        );
    }

    #[test]
    fn the_same_file_always_produces_the_same_state() {
        // Declaration order fixes id assignment, so a scenario is replayable.
        let mut first = MemoryGraph::new();
        let mut second = MemoryGraph::new();
        load_scenario(TWO_CLASSES, &mut first).unwrap();
        load_scenario(TWO_CLASSES, &mut second).unwrap();
        assert_eq!(first.state_hash().unwrap(), second.state_hash().unwrap());
    }

    // Task 3.1 RED #1 (plan §3.4): the field must exist and map each minted
    // node's handle back to the local name the scenario declared for it.
    // `TWO_CLASSES` mints `core` then `periphery`, top to bottom, so their
    // handles are `NodeId(0)`/`NodeId(1)` (the module doc's own "declaration
    // order is the id order" invariant).
    #[test]
    fn node_content_ids_map_each_node_id_back_to_its_declared_local_name() {
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(TWO_CLASSES, &mut graph).unwrap();
        assert_eq!(
            loaded.node_content_ids.get(&NodeId(0)),
            Some(&"core".to_owned())
        );
        assert_eq!(
            loaded.node_content_ids.get(&NodeId(1)),
            Some(&"periphery".to_owned())
        );
        assert_eq!(
            loaded.node_content_ids.len(),
            2,
            "exactly the two minted nodes, no more, no fewer"
        );
    }

    // Task 3.1 RED #2 (plan §3.4): the grain-invariance guard. `S` and `S'`
    // are the same two named nodes, `S'` with one extra node INSERTED
    // BEFORE them — the exact shape a scenario edit (or an LOD refinement,
    // ADR176 r20's "adding a single carrier shifts every later draw")
    // produces. Every pre-existing `NodeId` handle shifts by one in `S'`;
    // the whole point of a content id is that the shift must not touch it.
    // A private deterministic draw key built from `NodeId` instead of this
    // content id would be exactly the insertion-order dependence D69
    // forbids.
    #[test]
    fn shared_nodes_keep_their_content_id_across_an_inserted_earlier_node_even_though_the_node_id_handles_move(
    ) {
        const S: &str = r"
(scenario ft/grain-s
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS))
";
        const S_PRIME: &str = r"
(scenario ft/grain-s-prime
  (node inserted NodeType/SOCIAL_CLASS)
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS))
";
        let mut graph_s = MemoryGraph::new();
        let loaded_s = load_scenario(S, &mut graph_s).unwrap();
        let mut graph_s_prime = MemoryGraph::new();
        let loaded_s_prime = load_scenario(S_PRIME, &mut graph_s_prime).unwrap();

        // `S` mints top to bottom starting at NodeId(0); `S'`'s inserted
        // node takes NodeId(0) instead, pushing `core`/`periphery` one
        // handle later. Pinned explicitly so this test's own fixture proves
        // it is exercising a REAL shift, not accidentally testing nothing.
        assert_eq!(
            loaded_s.node_content_ids.get(&NodeId(0)),
            Some(&"core".to_owned())
        );
        assert_eq!(
            loaded_s.node_content_ids.get(&NodeId(1)),
            Some(&"periphery".to_owned())
        );
        assert_eq!(
            loaded_s_prime.node_content_ids.get(&NodeId(0)),
            Some(&"inserted".to_owned())
        );
        assert_eq!(
            loaded_s_prime.node_content_ids.get(&NodeId(1)),
            Some(&"core".to_owned()),
            "core's handle moved from NodeId(0) to NodeId(1)"
        );
        assert_eq!(
            loaded_s_prime.node_content_ids.get(&NodeId(2)),
            Some(&"periphery".to_owned()),
            "periphery's handle moved from NodeId(1) to NodeId(2)"
        );

        // The grain-invariance guard itself: whichever handle `core`/
        // `periphery` ended up with, their content id is exactly what the
        // scenario declared — recoverable by searching the map for the
        // NAME, independent of where that name's node happened to land.
        let find_id_for = |loaded: &super::LoadedScenario, name: &str| -> NodeId {
            *loaded
                .node_content_ids
                .iter()
                .find(|(_, content_id)| content_id.as_str() == name)
                .unwrap_or_else(|| panic!("`{name}` not present in node_content_ids"))
                .0
        };
        assert_eq!(find_id_for(&loaded_s, "core"), NodeId(0));
        assert_eq!(find_id_for(&loaded_s_prime, "core"), NodeId(1));
        assert_ne!(
            find_id_for(&loaded_s, "core"),
            find_id_for(&loaded_s_prime, "core"),
            "the handle really did move — otherwise this test would prove nothing"
        );
        assert_eq!(find_id_for(&loaded_s, "periphery"), NodeId(1));
        assert_eq!(find_id_for(&loaded_s_prime, "periphery"), NodeId(2));
    }

    // Injectivity at construction (plan §3.4, this train's Task 3): two
    // content ids must never collide onto one `NodeId` silently. Through
    // `load_scenario` this is UNCONSTRUCTIBLE — `load_node` mints a fresh id
    // per `(node ...)` form and refuses a second form reusing a local name
    // (`a_duplicate_local_name_is_loud`, above) before a second `named`
    // entry can ever be written — so the violating input is constructed
    // directly here, at `invert_content_ids` itself, to prove the assertion
    // actually fires rather than trusting it exists.
    #[test]
    #[should_panic(expected = "hydration bug")]
    fn two_content_ids_colliding_onto_one_node_id_is_a_loud_hydration_bug_not_a_silent_overwrite() {
        let mut named: HashMap<String, NodeId> = HashMap::new();
        named.insert("core".to_owned(), NodeId(0));
        named.insert("ghost".to_owned(), NodeId(0));
        let _ = invert_content_ids(&named);
    }

    #[test]
    fn an_unwritten_field_still_errors_after_a_load() {
        // The loader seeds no defaults, so III.11's honest null survives it.
        let mut graph = MemoryGraph::new();
        load_scenario(TWO_CLASSES, &mut graph).unwrap();
        let err = graph
            .node_attribute(NodeId(0), "social-class/agitation")
            .unwrap_err();
        assert!(err.message.contains("never a default"), "{}", err.message);
    }

    #[test]
    fn a_duplicate_local_name_is_loud() {
        let source = r"
(scenario ft/dup
  (node core NodeType/SOCIAL_CLASS)
  (node core NodeType/SOCIAL_CLASS))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("duplicate scenario name"),
            "{}",
            err.message
        );
    }

    #[test]
    fn an_edge_to_an_undeclared_name_is_loud() {
        let source = r"
(scenario ft/dangling
  (node core NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY core ghost 1))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("unknown node `ghost`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_currency_attribute_seeds_the_i128_lane_not_a_lossy_f64_cast() {
        // T3 #491, OQ-J: the Phase-2 gap this test used to pin as a refusal
        // is closed for node attributes — the literal routes through
        // `GraphSubstrate::update_node_currency`, the i128 lane, never a
        // truncating cast into f64.
        let source = r"
(scenario ft/money
  (deffield social-class/wages currency extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 120$)))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        assert_eq!(
            graph.node_attribute_currency(NodeId(0), "social-class/wages"),
            Ok(Currency::from_micro_units(120_000_000))
        );
        // And the ordinary f64 lane never received a shadow row.
        assert!(graph
            .node_attribute(NodeId(0), "social-class/wages")
            .is_err());
    }

    #[test]
    fn an_undeclared_field_is_a_typo_not_a_new_field() {
        // The registry contract enforced. Accepting an undeclared qname
        // would mint a field no typechecker knows about, and the rule that
        // meant to read it would fail far from the mistake.
        let source = r"
(scenario ft/typo
  (deffield social-class/wages int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wagez 120)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("never declared"), "{}", err.message);
    }

    #[test]
    fn a_bare_int_above_1_into_a_unit_interval_field_is_refused() {
        // 120 in a field declared `intensity` is outside that type's [0,1]
        // domain — storing it would make the store lie about what it holds.
        //
        // Renamed from `an_int_into_a_non_int_declared_field_is_refused`
        // (F3, typed-attribute-seeding fix round): before Half 1, EVERY
        // non-int-declared field refused EVERY value outright, so that name
        // described the whole refusal. Half 1 legalized the field's type; a
        // bare Int is now syntactically admissible for `intensity`
        // (`attribute_value_unit_interval`'s bare-Int arm — no lex-time
        // domain check on an unsuffixed literal) and refused ONLY because
        // 120 leaves [0,1] — the name now says what's actually tested. The
        // assertion (`"declared"`) is unmodified from before Half 1 and
        // continues to pass unchanged: existing behavior at this exact
        // boundary held, part of the additivity proof.
        let source = r"
(scenario ft/mistyped
  (deffield social-class/agitation intensity intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation 120)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("declared"), "{}", err.message);
    }

    // ---- Half 1 (typed-attribute seeding, P27 Phase 2) ----
    // `reports/typed-attribute-seeding-design-2026-08-11.md` §A/§E.

    #[test]
    fn a_scaled_literal_seeds_correctly_into_each_unit_interval_type() {
        // Exact-bit pins, NOT a tolerance. A `< 1e-12` comparison here would
        // hide exactly the transcription error it would appear to absorb —
        // the repo's own rule (`vitality_conformance.rs`). The conversion
        // CONTRACT `attribute_value_unit_interval`'s doc comment documents is
        // bit-for-bit IEEE-754, not approximately-equal.
        //
        // Expected bits computed INDEPENDENTLY of this crate — Python's
        // `struct.pack('>d', v)`, a different language's own correctly-
        // rounded decimal-to-binary conversion — not by re-running this
        // crate's own division and calling the result the oracle.
        //
        // `0.7c` and `0.123456789c` are DISCRIMINATING: `numerator *
        // 10_f64.powi(-scale)` (a real, non-equivalent reciprocal-multiply
        // substitution for the `/` this function actually uses) diverges
        // from `numerator / 10_f64.powi(scale)` at exactly these values
        // (`0.7c` seeds as `0x3fe6666666666667` under the mutation, against
        // `0x3fe6666666666666` here) while AGREEING at 0.75/0.5/0.358 — a
        // test using only the latter three is blind to that whole mutation
        // class, which is exactly what the F1 fix round found.
        for (ty, literal, expected_bits) in [
            ("probability", "0.75p", 0x3fe8_0000_0000_0000_u64),
            ("intensity", "0.5i", 0x3fe0_0000_0000_0000_u64),
            ("coefficient", "0.358c", 0x3fd6_e978_d4fd_f3b6_u64),
            ("coefficient", "0.7c", 0x3fe6_6666_6666_6666_u64),
            ("coefficient", "0.123456789c", 0x3fbf_9add_3739_635f_u64),
        ] {
            let source = format!(
                "(scenario ft/unit-interval\n  \
                 (deffield social-class/x {ty} intensive)\n  \
                 (node core NodeType/SOCIAL_CLASS (social-class/x {literal})))"
            );
            let mut graph = MemoryGraph::new();
            load_scenario(&source, &mut graph).unwrap();
            let value = graph.node_attribute(NodeId(0), "social-class/x").unwrap();
            assert_eq!(
                value.to_bits(),
                expected_bits,
                "{ty} {literal}: got 0x{:016x}, want 0x{expected_bits:016x}",
                value.to_bits()
            );
        }
    }

    #[test]
    fn a_coefficient_strength_literal_seeds_bit_exactly() {
        // Task 6a (T2 plan amendment, 2026-08-12): D32 rules
        // <edge-type>/strength Coefficient-kinded — a c-suffixed literal is
        // hydration's idiomatic way (D32 kinds the field Coefficient; p/i
        // seed identically — see the companion test) to seed a fractional
        // strength (a bare decimal
        // is E-LEX-021, and pre-6a this loader refused every non-int
        // strength). Bit-exact pin, NOT a tolerance — the same conversion
        // contract `attribute_value_unit_interval`'s doc comment states;
        // 0.125 = 2^-3 is dyadic-exact, so 125/1000 divides to it exactly.
        let source = r"
(scenario ft/coeff-strength
  (node core NodeType/SOCIAL_CLASS)
  (node periphery NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY core periphery 0.125c))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let strength = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
            .unwrap();
        assert_eq!(
            strength.to_bits(),
            (0.125_f64).to_bits(),
            "a c-suffixed Coefficient strength must seed bit-exactly, got {strength}"
        );
    }

    #[test]
    fn a_probability_or_intensity_strength_literal_seeds_identically() {
        // The kind-blind half of Task 6a's ruling (fix round, MAJOR-1
        // option (i)), pinned GREEN: kinds do not survive evaluation
        // (evaluator.rs's atom arm makes every p/i/c literal an untagged
        // Value::Real), so hydration accepts p/i exactly as it accepts c —
        // mirroring the runtime writer (structural_verbs.rs::add_edge, any
        // Value::Real at :strength).
        for literal in ["0.125p", "0.125i"] {
            let source = format!(
                "(scenario ft/kind-blind-strength\n  \
                 (node core NodeType/SOCIAL_CLASS)\n  \
                 (node periphery NodeType/SOCIAL_CLASS)\n  \
                 (edge EdgeType/SOLIDARITY core periphery {literal}))"
            );
            let mut graph = MemoryGraph::new();
            load_scenario(&source, &mut graph).unwrap();
            let strength = graph
                .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
                .unwrap();
            assert_eq!(strength.to_bits(), (0.125_f64).to_bits(), "{literal}");
        }
    }

    #[test]
    fn a_ratio_or_currency_strength_literal_stays_refused() {
        // What stays out under the kind-blind widening, each mirroring the
        // runtime :strength match (which accepts only Value::Real): r
        // evaluates to Value::Ratio, $ to Value::Currency. r is a ScaledLit
        // kind the guard refuses; $ (Currency) is a DIFFERENT Atom variant
        // that falls to the same refusal arm without ever reaching the kind
        // guard (see the Step-6 mutation split in the T2 plan).
        for literal in ["0.5r", "1$"] {
            let source = format!(
                "(scenario ft/wrong-kind-strength\n  \
                 (node core NodeType/SOCIAL_CLASS)\n  \
                 (node periphery NodeType/SOCIAL_CLASS)\n  \
                 (edge EdgeType/SOLIDARITY core periphery {literal}))"
            );
            let mut graph = MemoryGraph::new();
            let err = load_scenario(&source, &mut graph).unwrap_err();
            assert!(
                err.message.contains(
                    "expected an integer or p/i/c-suffixed unit-interval strength literal"
                ),
                "{literal}: {}",
                err.message
            );
        }
    }

    #[test]
    fn a_seeded_literal_bit_matches_the_same_literal_written_by_a_rule() {
        // F1 (typed-attribute-seeding fix round): the conversion contract
        // pinned as the RELATION between the two paths, which is what
        // `attribute_value_unit_interval`'s doc comment actually promises —
        // not two independently-eyeballed numbers that happen to agree.
        // Seeds a field via the scenario loader (`attribute_value`) with a
        // literal, and separately has a rule write the IDENTICAL literal
        // text to a parallel field via `(update-node self … (set …))` —
        // the runtime path (`tick.rs::atom_to_value` /
        // `structural_verbs.rs::numeric_write_value`) — then compares
        // `to_bits()` off the live graph. Mutation-caught: the same
        // reciprocal-multiply substitution the previous test's doc comment
        // describes leaves this test's OTHER two literals agreeing but
        // diverges `0.7c` between the two paths.
        for literal in ["0.7c", "0.5c", "0.123456789c"] {
            let source = format!(
                "(scenario ft/bit-equality\n  \
                 (deffield social-class/seeded coefficient extensive)\n  \
                 (deffield social-class/written coefficient extensive)\n  \
                 (node core NodeType/SOCIAL_CLASS (social-class/seeded {literal})))"
            );
            let mut graph = MemoryGraph::new();
            // Review round 1 (#576, Minor): thread the REAL `node_content_ids`
            // this hydration produces, rather than discarding it and passing
            // an empty map — this test genuinely hydrates a scenario, so it
            // should exercise the hydrated path honestly, not the
            // empty-map-fixture fallback (`evaluator::element_content_id`'s
            // own doc names the two shapes explicitly).
            let loaded_scenario = load_scenario(&source, &mut graph).unwrap();

            let types = TypeEnv {
                fields: HashMap::from([
                    (
                        "social-class/seeded".to_owned(),
                        FieldDecl {
                            ty: BslType::Coefficient,
                            kind: FieldKind::Extensive,
                        },
                    ),
                    (
                        "social-class/written".to_owned(),
                        FieldDecl {
                            ty: BslType::Coefficient,
                            kind: FieldKind::Extensive,
                        },
                    ),
                ]),
                exemptions: &[],
            };
            let vocabulary = BindingVocabulary {
                fields: types.fields.keys().cloned().collect(),
                consts: HashSet::new(),
                probability_consts: HashSet::new(),
                metrics: HashSet::new(),
            };
            let ceilings = CardinalityCeilings::new(
                HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), 100)]),
                HashMap::new(),
            );
            let intrinsics = IntrinsicCosts::default();
            let systems = HashSet::from(["ft".to_owned()]);
            let enums = EnumRegistry::default();
            let const_values = HashMap::new();
            let ctx = LoadContext {
                vocabulary: &vocabulary,
                types: &types,
                enums: &enums,
                const_values: &const_values,
                ceilings: &ceilings,
                intrinsics: &intrinsics,
                systems: &systems,
                vocabulary_registry: None,
                rule_file: "ft/bit-equality.bsl",
            };
            let rule = format!(
                "(rule ft/mirror\n  \
                 :role mechanic :evidence derived :material-basis \"pins the seed path and the runtime write path to the \
                 SAME conversion, as one relation rather than two independently-eyeballed \
                 numbers\"\n  \
                 :fuel 64\n  \
                 (bindings (binding seeded :field social-class/seeded))\n  \
                 (when (>= seeded 0.0c))\n  \
                 (effects (update-node self social-class/written (set {literal}))))"
            );
            let loaded =
                load_rule(&rule, &ctx).unwrap_or_else(|e| panic!("{literal}: rule must load: {e}"));
            let mut sink = CollectingSink::default();
            let enums = EnumRegistry::default();
            run_tick(
                &loaded,
                &types,
                &enums,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &intrinsics,
                &DefinesEnv::new(),
                1,
                Some(&loaded_scenario.node_content_ids),
                &SessionId::new("scenario-bit-equality-test").expect("literal is non-empty"),
                None,
            )
            .unwrap_or_else(|e| panic!("{literal}: tick must run: {e}"));

            let seeded = graph
                .node_attribute(NodeId(0), "social-class/seeded")
                .unwrap();
            let written = graph
                .node_attribute(NodeId(0), "social-class/written")
                .unwrap();
            assert_eq!(
                seeded.to_bits(),
                written.to_bits(),
                "{literal}: seed path 0x{:016x} != runtime path 0x{:016x}",
                seeded.to_bits(),
                written.to_bits()
            );
        }
    }

    #[test]
    fn a_seeded_currency_literal_bit_matches_the_same_literal_written_by_a_rule() {
        // T3 #491, OQ-J: extends the PR #505 bit-equality contract
        // (`a_seeded_literal_bit_matches_the_same_literal_written_by_a_rule`,
        // above) to the Currency lane — a seeded `$`-suffixed literal (the
        // scenario-load path, `attribute_value_currency`) must bit-match
        // the SAME literal written by a rule's `(update-node self …
        // (set …))` (the runtime path, `tick.rs::atom_to_value` /
        // `structural_verbs.rs::update_node_currency_op`), computed
        // INDEPENDENTLY through two different code paths — not two
        // numbers that merely happen to agree.
        for literal in ["120$", "0.5$", "0$", "1000000$"] {
            let source = format!(
                "(scenario ft/currency-bit-equality\n  \
                 (deffield social-class/seeded currency extensive)\n  \
                 (deffield social-class/written currency extensive)\n  \
                 (node core NodeType/SOCIAL_CLASS (social-class/seeded {literal})))"
            );
            let mut graph = MemoryGraph::new();
            let loaded_scenario = load_scenario(&source, &mut graph).unwrap();

            let types = TypeEnv {
                fields: HashMap::from([
                    (
                        "social-class/seeded".to_owned(),
                        FieldDecl {
                            ty: BslType::Currency,
                            kind: FieldKind::Extensive,
                        },
                    ),
                    (
                        "social-class/written".to_owned(),
                        FieldDecl {
                            ty: BslType::Currency,
                            kind: FieldKind::Extensive,
                        },
                    ),
                ]),
                exemptions: &[],
            };
            let vocabulary = BindingVocabulary {
                fields: types.fields.keys().cloned().collect(),
                consts: HashSet::new(),
                probability_consts: HashSet::new(),
                metrics: HashSet::new(),
            };
            let ceilings = CardinalityCeilings::new(
                HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), 100)]),
                HashMap::new(),
            );
            let intrinsics = IntrinsicCosts::default();
            let systems = HashSet::from(["ft".to_owned()]);
            let enums = EnumRegistry::default();
            let const_values = HashMap::new();
            let ctx = LoadContext {
                vocabulary: &vocabulary,
                types: &types,
                enums: &enums,
                const_values: &const_values,
                ceilings: &ceilings,
                intrinsics: &intrinsics,
                systems: &systems,
                vocabulary_registry: None,
                rule_file: "ft/currency-bit-equality.bsl",
            };
            let rule = format!(
                "(rule ft/currency-mirror\n  \
                 :role mechanic :evidence derived :material-basis \"pins the Currency seed path and the Currency runtime \
                 write path to the SAME i128 value, as one relation rather than two \
                 independently-eyeballed numbers\"\n  \
                 :fuel 64\n  \
                 (bindings (binding seeded :field social-class/seeded))\n  \
                 (when (>= seeded 0$))\n  \
                 (effects (update-node self social-class/written (set {literal}))))"
            );
            let loaded =
                load_rule(&rule, &ctx).unwrap_or_else(|e| panic!("{literal}: rule must load: {e}"));
            let mut sink = CollectingSink::default();
            let enums = EnumRegistry::default();
            run_tick(
                &loaded,
                &types,
                &enums,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &intrinsics,
                &DefinesEnv::new(),
                1,
                Some(&loaded_scenario.node_content_ids),
                &SessionId::new("scenario-currency-bit-equality-test").expect("non-empty"),
                None,
            )
            .unwrap_or_else(|e| panic!("{literal}: tick must run: {e}"));

            let seeded = graph
                .node_attribute_currency(NodeId(0), "social-class/seeded")
                .unwrap();
            let written = graph
                .node_attribute_currency(NodeId(0), "social-class/written")
                .unwrap();
            assert_eq!(
                seeded, written,
                "{literal}: seed path {seeded:?} != runtime path {written:?}"
            );
            assert_eq!(
                seeded.micro_units(),
                written.micro_units(),
                "{literal}: micro-unit i128 values must match exactly, not just Eq"
            );
        }
    }

    #[test]
    fn a_currency_free_scenario_encodes_byte_identically_to_before_this_train() {
        // 3.1's second encoder leg: a scenario declaring no `currency` field
        // at all must produce EXACTLY the same state hash it always has —
        // no 0x06 tag, no byte moved — since the sixth section is elided
        // when empty (D189). Proven at the scenario-load level, not just
        // the raw `StateEncoder` level `state_hash.rs`'s own tests already
        // cover, so a regression in `load_node`'s NEW branch (routing on
        // `BslType::Currency`) would be caught here too.
        let source = r"
(scenario ft/no-currency
  (deffield social-class/wages int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 120)))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let bytes = graph.encode_state().unwrap();
        // Hand-built expected encoding: one node, one int attribute, no
        // edges, no hyperedges, no edge attributes, no currency attributes
        // — sections 0x01/0x02 only, 0x03-0x06 all empty-but-present or
        // elided per their own rules (0x03/0x04 unconditional per version
        // 1; 0x05/0x06 elided when empty per version 2/3).
        let mut manual = StateEncoder::new();
        manual
            .write_nodes(&[(NodeId(0), "SOCIAL_CLASS".to_owned())])
            .unwrap();
        manual
            .write_attributes(&[(NodeId(0), "social-class/wages".to_owned(), 120.0)])
            .unwrap();
        manual.write_edges(&[]).unwrap();
        manual.write_hyperedges(&[]).unwrap();
        assert_eq!(
            bytes.as_bytes(),
            manual.as_bytes(),
            "a Currency-free scenario must encode byte-identically to the pre-T3 shape — \
             no 0x06 bytes at all"
        );
    }

    #[test]
    fn a_scenario_with_one_currency_attribute_grows_exactly_one_sixth_section() {
        // 3.1's third encoder leg: a scenario declaring ONE currency field
        // grows the encoding by exactly one 0x06 section, appended after
        // whatever the Currency-free encoding already produced — proven as
        // a byte-length/prefix relationship, not merely "the hash differs".
        let bare_source = r"
(scenario ft/bare
  (deffield social-class/wages int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 120)))
";
        let mut bare_graph = MemoryGraph::new();
        load_scenario(bare_source, &mut bare_graph).unwrap();
        let bare_bytes = bare_graph.encode_state().unwrap();

        let currency_source = r"
(scenario ft/with-currency
  (deffield social-class/wages int extensive)
  (deffield social-class/treasury currency extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 120) (social-class/treasury 5$)))
";
        let mut currency_graph = MemoryGraph::new();
        load_scenario(currency_source, &mut currency_graph).unwrap();
        let currency_bytes = currency_graph.encode_state().unwrap();

        assert!(
            currency_bytes.as_bytes().starts_with(bare_bytes.as_bytes()),
            "the four version-1 sections (plus the elided 0x05) are an untouched PREFIX"
        );
        let grown = &currency_bytes.as_bytes()[bare_bytes.as_bytes().len()..];
        assert_eq!(
            grown[0], 0x06,
            "the grown suffix is exactly one 0x06 section"
        );
        // L-6 (#491 T3 review): `grown[0] == 0x06` alone would still pass if
        // a SECOND section were hypothetically appended after it — this
        // test's own name claims "exactly one", so pin the suffix's exact
        // LENGTH too: tag(1) + u32 count(4) + one entry's
        // [u64 id(8) + u32 name-len-prefix(4) + name bytes + i128 value(16)].
        let expected_len = 1 + 4 + 8 + 4 + "social-class/treasury".len() + 16;
        assert_eq!(
            grown.len(),
            expected_len,
            "the grown suffix must be EXACTLY one entry's worth of 0x06 bytes, not more"
        );
        assert_eq!(
            currency_graph
                .node_attribute_currency(NodeId(0), "social-class/treasury")
                .unwrap(),
            Currency::from_micro_units(5_000_000)
        );
    }

    #[test]
    fn unit_interval_literal_acceptance_is_kind_blind_among_p_i_c() {
        // `store_range_check`'s own runtime predicate does not distinguish
        // Probability/Intensity/Coefficient from one another — `Value::Real`
        // carries no p/i/c tag once a literal is evaluated. This load-time
        // mirror does not either: a `p`-suffixed literal is legal for a
        // `coefficient`-declared field.
        let source = r"
(scenario ft/kind-blind
  (deffield social-class/agitation coefficient intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation 0.5p)))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let value = graph
            .node_attribute(NodeId(0), "social-class/agitation")
            .unwrap();
        assert!((value - 0.5).abs() < 1e-12);
    }

    #[test]
    fn a_bare_int_within_0_1_seeds_a_unit_interval_field() {
        // Bare Int literals carry no lex-time domain check, so an IN-range
        // one (unlike `a_bare_int_above_1_into_a_unit_interval_field_is_refused`'s
        // 120) must be accepted, exactly as `store_range_check` would accept
        // a rule-computed `Value::Int` widened to the same f64 at runtime.
        let source = r"
(scenario ft/bare-int-in-range
  (deffield social-class/agitation intensity intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation 1)))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let value = graph
            .node_attribute(NodeId(0), "social-class/agitation")
            .unwrap();
        assert!((value - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_negative_bare_int_into_a_unit_interval_field_is_refused() {
        // The mirrored predicate's message, pinned precisely: mentions the
        // [0,1] domain and states the never-a-clamp discipline, matching
        // `store_range_check`'s own wording family (`E-EVAL-020`).
        let source = r"
(scenario ft/negative-bare-int
  (deffield social-class/agitation probability intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation -3)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("[0,1]"), "{}", err.message);
        assert!(err.message.contains("never a clamp"), "{}", err.message);
    }

    #[test]
    fn a_ratio_literal_is_refused_for_a_unit_interval_field_even_in_range() {
        // 0.5r numerically falls in [0,1], but Ratio is a distinct runtime
        // `Value` variant with its own (0, ∞) domain, and a `:field` read
        // can never legally produce one (`bind_subject` wraps every field
        // read `Value::Real`) — a kind refusal, not a range violation.
        let source = r"
(scenario ft/ratio-into-coefficient
  (deffield social-class/agitation coefficient intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation 0.5r)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("Ratio"), "{}", err.message);
    }

    #[test]
    fn an_int_exceeding_f64_exact_range_is_refused() {
        // The 2^53 guard, unchanged by Half 1 — still lives in
        // `attribute_value_int`, still fires for an `int`-declared field.
        let source = format!(
            "(scenario ft/too-big\n  (deffield social-class/population int extensive)\n  \
             (node core NodeType/SOCIAL_CLASS (social-class/population {})))",
            (1_i64 << 53) + 1
        );
        let mut graph = MemoryGraph::new();
        let err = load_scenario(&source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("exceeds f64's exact integer range"),
            "{}",
            err.message
        );
    }

    #[test]
    fn an_int_at_exactly_2_pow_53_still_loads() {
        // The boundary is inclusive — exact in f64 up to and including 2^53.
        let source = format!(
            "(scenario ft/at-boundary\n  (deffield social-class/population int extensive)\n  \
             (node core NodeType/SOCIAL_CLASS (social-class/population {})))",
            1_i64 << 53
        );
        let mut graph = MemoryGraph::new();
        load_scenario(&source, &mut graph).unwrap();
    }

    #[test]
    fn a_currency_literal_into_an_int_declared_field_is_refused() {
        // The OTHER Currency-refusal site: the field is legally declared
        // `int`, but the literal itself is `$`-suffixed — a kind mismatch,
        // not the "no storage at all" gap (T3 #491/OQ-J closed that one).
        let source = r"
(scenario ft/currency-into-int
  (deffield social-class/wages int extensive)
  (node core NodeType/SOCIAL_CLASS (social-class/wages 120$)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("needs a currency-declared field"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_currency_literal_into_a_unit_interval_field_is_refused() {
        let source = r"
(scenario ft/currency-into-coefficient
  (deffield social-class/agitation coefficient intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/agitation 120$)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("needs a currency-declared field"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_coefficient_attribute_hashes_identically_to_the_same_f64_as_an_int() {
        // The "provably additive" claim, made concrete at the byte level:
        // seeding a `coefficient`-declared field with `0.358c` must produce
        // the EXACT same section-0x02 row shape `StateEncoder` would produce
        // for an `int`-declared field holding the same f64 — proving
        // `attribute_value`'s widening introduced no format branch keyed on
        // `BslType`. `CanonicalState`'s section 0x02 is a bare
        // `u64 id ‖ str name ‖ u64 value-bits` regardless of which BSL type
        // declared the field (`rust/crates/babylon-graph/src/state_hash.rs`
        // module docs).
        let source = r"
(scenario ft/mixed-types
  (deffield social-class/population int extensive)
  (deffield social-class/agitation coefficient intensive)
  (node core NodeType/SOCIAL_CLASS
    (social-class/population 120)
    (social-class/agitation 0.358c)))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let hash = graph.state_hash().unwrap();

        // Hand-built via StateEncoder directly, bypassing attribute_value
        // (and the whole scenario loader) entirely — a second, independent
        // encoding of the identical facts, attributes sorted by name
        // ascending ("social-class/agitation" < "social-class/population").
        let mut enc = StateEncoder::new();
        enc.write_nodes(&[(NodeId(0), "SOCIAL_CLASS".to_owned())])
            .unwrap();
        enc.write_attributes(&[
            (
                NodeId(0),
                "social-class/agitation".to_owned(),
                358.0 / 1000.0,
            ),
            (NodeId(0), "social-class/population".to_owned(), 120.0),
        ])
        .unwrap();
        enc.write_edges(&[]).unwrap();
        enc.write_hyperedges(&[]).unwrap();
        assert_eq!(hash, enc.finish());
    }

    // ---- Train B item 6 (#591): the `real` deffield type ----

    #[test]
    fn real_deffield_seeds_int_scaled_and_ratio_verbatim() {
        // Exact-bit pins, the same discipline
        // `a_scaled_literal_seeds_correctly_into_each_unit_interval_type`
        // states above — the conversion is the crate's ONE scaled-literal
        // contract (`unscaled / 10^scale`), not a tolerance. A `real`
        // field accepts the three literal lanes whose own lex laws already
        // bound them: int, p/i/c, and r (Ratio is `Atom::Scaled` with
        // `ScaledKind::Ratio`, so one arm covers both scaled kinds).
        for (literal, expected) in [("9", 9.0_f64), ("0.25c", 0.25_f64), ("1.5r", 1.5_f64)] {
            let source = format!(
                "(scenario ft/real\n  \
                 (deffield social-class/balance real intensive)\n  \
                 (node core NodeType/SOCIAL_CLASS (social-class/balance {literal})))"
            );
            let mut graph = MemoryGraph::new();
            load_scenario(&source, &mut graph).unwrap();
            let value = graph
                .node_attribute(NodeId(0), "social-class/balance")
                .unwrap();
            assert_eq!(
                value.to_bits(),
                expected.to_bits(),
                "{literal}: got 0x{:016x}, want 0x{:016x}",
                value.to_bits(),
                expected.to_bits()
            );
        }
    }

    #[test]
    fn real_deffield_refuses_currency_and_bare_ident() {
        // Currency: a kind mismatch, not the retired "no storage at all"
        // gap (T3 #491/OQ-J closed that one for currency-declared fields) —
        // `real` is a different declared type, and f64 cannot hold i128
        // micro-units without lying about what it stores.
        let source = r"
(scenario ft/real-currency
  (deffield social-class/balance real intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/balance 9$)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("needs a currency-declared field"),
            "{}",
            err.message
        );

        // A bare identifier is not a numeric literal at all — loud, naming
        // the node and the field.
        let source = r"
(scenario ft/real-ident
  (deffield social-class/balance real intensive)
  (node core NodeType/SOCIAL_CLASS (social-class/balance someident)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("`core`") && err.message.contains("social-class/balance"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_defconst_becomes_the_defines_environment() {
        // §2.5's `:const` reads the defines environment, and slice 1 has no
        // GameDefines reader — so the scenario declares it, exactly as it
        // declares fields.
        let source = r"
(scenario ft/coefficients
  (defconst economy/base-subsistence 0.0005c)
  (defconst economy/tick-budget 12))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        assert_eq!(loaded.consts.len(), 2);
        // 5 / 10^4, the canonical minimal-scale form — the same conversion
        // the tick applies to a `:default` literal.
        assert_eq!(
            loaded.consts["economy/base-subsistence"],
            crate::evaluator::Value::Real(5.0 / 10_f64.powi(4))
        );
        assert_eq!(
            loaded.consts["economy/tick-budget"],
            crate::evaluator::Value::Int(12)
        );
    }

    #[test]
    fn probability_defconst_identity_is_retained_without_numeric_inference() {
        let source = r"
(scenario ft/typed-const-identities
  (defconst event/likelihood 0.5p)
  (defconst event/intensity 0.5i)
  (defconst event/coefficient 0.5c)
  (defconst event/ticket-mass 0.5m))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();

        assert_eq!(loaded.consts.len(), 4);
        assert_eq!(
            loaded.probability_consts,
            HashSet::from(["event/likelihood".to_owned()])
        );
    }

    // ---- §3.2 addendum, #492/ADR194: Ratio defconsts + :cap ----

    #[test]
    fn a_bare_ratio_defconst_carries_no_cap() {
        // No :cap — the domain is Ratio's own (0, ∞), unbounded above.
        // Matches rent_spike_multiplier's declared domain exactly (Territory
        // eviction pipeline; "moddable to 2.0" is well inside it).
        let source = r"
(scenario ft/uncapped-ratio
  (defconst territory/rent-spike-multiplier 2.0r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["territory/rent-spike-multiplier"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 2.0).abs() < 1e-12);
                assert_eq!(*floor, None);
                assert_eq!(*cap, None);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn a_capped_ratio_defconst_within_bounds_loads() {
        // pareto_alpha's shape: declared domain (0, 10], value 1.5.
        let source = r"
(scenario ft/capped-ratio
  (defconst lifecycle/pareto-alpha 1.5r :cap 10r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["lifecycle/pareto-alpha"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 1.5).abs() < 1e-12);
                assert_eq!(*floor, None);
                assert!((cap.expect("cap declared").get() - 10.0).abs() < 1e-12);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn a_ratio_defconst_exceeding_its_own_declared_cap_is_e_load_052() {
        let source = r"
(scenario ft/over-cap
  (defconst lifecycle/pareto-alpha 12r :cap 10r))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-052"));
        assert!(
            err.message.contains("exceeds its own :cap"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_ratio_defconst_exactly_at_its_cap_loads() {
        // Closed at the top, like every other closed-interval-at-the-top
        // domain this spec uses (p/i/c's [0,1] accepts the endpoint).
        let source = r"
(scenario ft/at-cap
  (defconst lifecycle/pareto-alpha 10r :cap 10r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["lifecycle/pareto-alpha"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 10.0).abs() < 1e-12);
                assert_eq!(*floor, None);
                assert!((cap.unwrap().get() - 10.0).abs() < 1e-12);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn cap_on_a_non_ratio_literal_is_refused() {
        // :cap narrows Ratio's own (0, ∞) domain specifically — it has no
        // meaning on a p/i/c literal (already capped [0,1] at lex time) or
        // an Int, and silently ignoring it would hide an authoring mistake.
        for (label, src) in [
            ("c", "(defconst economy/rate 0.5c :cap 10r)"),
            ("int", "(defconst economy/count 5 :cap 10r)"),
        ] {
            let source = format!("(scenario ft/stray-cap {src})");
            let mut graph = MemoryGraph::new();
            let err = load_scenario(&source, &mut graph).unwrap_err();
            assert!(
                err.message
                    .contains(":floor/:cap are legal only on a Ratio"),
                "{label}: {}",
                err.message
            );
        }
    }

    #[test]
    fn caps_own_operand_must_itself_be_a_ratio_literal() {
        let source = r"
(scenario ft/bad-cap-operand
  (defconst lifecycle/pareto-alpha 1.5r :cap 0.5c))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains(":cap's operand must be a Ratio"),
            "{}",
            err.message
        );
    }

    // ---- :floor (DEFECT 1 fix round, adversarial verification of #500) ----

    #[test]
    fn a_floored_ratio_defconst_within_bounds_loads() {
        // entropy_factor's exact shape: declared domain (1.0, 3.0], value
        // 1.5. `content/rules/metabolism.bsl`'s eventual port will read
        // this exact const once that train lands.
        let source = r"
(scenario ft/floored-ratio
  (defconst metabolism/entropy-factor 1.5r :floor 1r :cap 3r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["metabolism/entropy-factor"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 1.5).abs() < 1e-12);
                assert!((floor.expect("floor declared").get() - 1.0).abs() < 1e-12);
                assert!((cap.expect("cap declared").get() - 3.0).abs() < 1e-12);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn floor_alone_with_no_cap_loads() {
        let source = r"
(scenario ft/floor-only
  (defconst metabolism/entropy-factor 1.5r :floor 1r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["metabolism/entropy-factor"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 1.5).abs() < 1e-12);
                assert!((floor.expect("floor declared").get() - 1.0).abs() < 1e-12);
                assert_eq!(*cap, None);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn cap_before_floor_in_source_order_still_parses() {
        // parse_bound_keywords accepts either order.
        let source = r"
(scenario ft/cap-then-floor
  (defconst metabolism/entropy-factor 1.5r :cap 3r :floor 1r))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        match &loaded.consts["metabolism/entropy-factor"] {
            crate::evaluator::Value::Ratio { value, floor, cap } => {
                assert!((value.get() - 1.5).abs() < 1e-12);
                assert!((floor.unwrap().get() - 1.0).abs() < 1e-12);
                assert!((cap.unwrap().get() - 3.0).abs() < 1e-12);
            }
            other => panic!("expected Value::Ratio, got {other:?}"),
        }
    }

    #[test]
    fn a_value_at_exactly_its_declared_floor_is_e_load_052() {
        // EXCLUSIVE: the declared value must be STRICTLY greater than its
        // own floor — matching entropy_factor's `> 1.0`, not `>= 1.0`.
        let source = r"
(scenario ft/at-floor
  (defconst metabolism/entropy-factor 1r :floor 1r :cap 3r))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-052"));
        assert!(
            err.message.contains("does not exceed its own :floor"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_value_below_its_declared_floor_is_e_load_052() {
        let source = r"
(scenario ft/below-floor
  (defconst metabolism/entropy-factor 0.5r :floor 1r :cap 3r))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-052"));
        assert!(
            err.message.contains("does not exceed its own :floor"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_floor_not_strictly_below_its_cap_is_e_load_052() {
        // (floor, cap] is empty unless floor < cap — floor == cap and
        // floor > cap are both refused.
        for (label, src) in [
            (
                "equal",
                "(defconst metabolism/entropy-factor 2r :floor 2r :cap 2r)",
            ),
            (
                "inverted",
                "(defconst metabolism/entropy-factor 2r :floor 3r :cap 2r)",
            ),
        ] {
            let source = format!("(scenario ft/bad-floor-cap {src})");
            let mut graph = MemoryGraph::new();
            let err = load_scenario(&source, &mut graph).unwrap_err();
            assert_eq!(err.code, Some("E-LOAD-052"), "{label}: {}", err.message);
            assert!(
                err.message.contains("is not strictly below its own :cap"),
                "{label}: {}",
                err.message
            );
        }
    }

    #[test]
    fn floors_own_operand_must_itself_be_a_ratio_literal() {
        let source = r"
(scenario ft/bad-floor-operand
  (defconst metabolism/entropy-factor 1.5r :floor 0.5c))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains(":floor's operand must be a Ratio"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_repeated_bound_keyword_is_loud_never_last_one_wins() {
        let source = r"
(scenario ft/dup-floor
  (defconst metabolism/entropy-factor 1.5r :floor 1r :floor 0.5r))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("given twice"), "{}", err.message);
    }

    #[test]
    fn an_unrecognized_bound_keyword_is_refused() {
        let source = r"
(scenario ft/bad-keyword
  (defconst metabolism/entropy-factor 1.5r :ceiling 3r))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("unrecognized keyword"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_currency_defconst_is_carried_exactly_as_a_currency_default_is() {
        // T3 #491, OQ-J: one literal, one legality, now landed as ACCEPTANCE
        // rather than refusal — `tick.rs::atom_to_value` carries a Currency
        // `:default` into `Value::Currency`, and `load_defconst` now carries
        // a Currency literal the same way, for the same "one form, one
        // legality" reason the retired refusal test named.
        let source = r"
(scenario ft/money-coefficient
  (defconst economy/floor-wage 15$))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        assert_eq!(
            loaded.consts["economy/floor-wage"],
            crate::evaluator::Value::Currency(Currency::from_micro_units(15_000_000))
        );
    }

    #[test]
    fn a_currency_defconst_rejects_stray_floor_and_cap() {
        // `:floor`/`:cap` are legal ONLY on a Ratio (`r`-suffixed) literal
        // (#492/ADR194) — Currency joins Int/Bool's existing refusal here,
        // the same `reject_stray_bounds` call every non-Ratio literal kind
        // makes.
        let source = r"
(scenario ft/money-coefficient-bounded
  (defconst economy/floor-wage 15$ :cap 20$))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("Currency literal"), "{}", err.message);
    }

    #[test]
    fn a_duplicate_defconst_is_loud_never_last_one_wins() {
        // A coefficient has one value. Silently rebinding it would make the
        // rule reading it depend on declaration order.
        let source = r"
(scenario ft/twice
  (defconst economy/base-subsistence 0.0005c)
  (defconst economy/base-subsistence 0.5c))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("duplicate defconst"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_duplicate_defconst_carries_a_name_identity() {
        // Task 2 (issue #652 §2.3/§2.4): the qname is local context `err()`
        // discards, attached via `ScenarioError::with_identity` — the
        // one construction site with no wrapped typed error to delegate to.
        let source = r"
(scenario ft/twice
  (defconst economy/base-subsistence 0.0005c)
  (defconst economy/base-subsistence 0.5c))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(
            err.identity,
            Some(ErrorIdentity::Name("economy/base-subsistence".to_owned()))
        );
    }

    #[test]
    fn a_lexical_error_carries_position_and_code() {
        // Task 2 (issue #652 §2.1a): `From<ReadError>` stops discarding the
        // reader's own byte offset and, for a genuine `E-LEX` failure, its
        // spec code. `#true` is not a legal token anywhere in the grammar
        // (`reader.rs`'s own `LexCode::UnclassifiableToken` tests use the
        // same probe).
        let source = "(scenario ft/lex-error\n  (defconst economy/base-subsistence #true))\n";
        let raw = crate::reader::read_all(source.as_bytes())
            .expect_err("the fixture must not read cleanly");
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.position, Some(raw.position));
        assert_eq!(err.code, Some("E-LEX-003"));
    }

    #[test]
    fn a_defconst_taking_an_expression_is_refused() {
        // A coefficient is a number. An expression would need an evaluation
        // environment that does not exist at scenario-load time, and
        // accepting the form while ignoring the operand is the silent-drop
        // shape.
        let source = r"
(scenario ft/computed
  (defconst economy/base-subsistence (* 2 3)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("<literal>"), "{}", err.message);
    }

    #[test]
    fn a_file_holding_two_scenarios_is_refused() {
        let source = "(scenario a/one) (scenario a/two)";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("exactly one"), "{}", err.message);
    }

    // ---- §2.13 `.bscn` dialect: defenum, enum deffield, EnumRef-only
    // seeding (Organization spec §1 Q12, D101) ----

    const ORG_KIND_SOURCE: &str = r"
(scenario org/t
  (defenum OrgKind (STATE_APPARATUS BUSINESS
                     POLITICAL_FACTION CIVIL_SOCIETY))
  (deffield organization/kind enum OrgKind)
  (node acme NodeType/ORGANIZATION (organization/kind OrgKind/BUSINESS)))
";

    #[test]
    fn an_enum_field_seeds_by_member_ref_and_stores_the_declared_ordinal() {
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(ORG_KIND_SOURCE, &mut graph).expect("loads");
        assert_eq!(loaded.node_count, 1);
        let id = graph.nodes("ORGANIZATION")[0];
        let stored = graph.node_attribute(id, "organization/kind").unwrap();
        assert!(
            (stored - 1.0).abs() < 1e-12,
            "BUSINESS is declaration-order index 1, stored: {stored}"
        );
        assert!(loaded.enums.resolve("OrgKind").is_some());
    }

    #[test]
    fn a_bare_number_into_an_enum_field_refuses_naming_the_law() {
        let source = r"
(scenario org/t
  (defenum OrgKind (STATE_APPARATUS BUSINESS))
  (deffield organization/kind enum OrgKind)
  (node acme NodeType/ORGANIZATION (organization/kind 1)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-056"));
        assert!(
            err.message.contains("<EnumType>/<MEMBER>") && err.message.contains("never"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_wrong_enum_type_member_refuses() {
        let source = r"
(scenario org/t
  (defenum OrgKind (STATE_APPARATUS BUSINESS))
  (deffield organization/kind enum OrgKind)
  (node acme NodeType/ORGANIZATION (organization/kind NodeType/SOCIAL_CLASS)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-056"));
        assert!(err.message.contains("OrgKind"), "{}", err.message);
    }

    #[test]
    fn an_undeclared_member_refuses() {
        let source = r"
(scenario org/t
  (defenum OrgKind (STATE_APPARATUS BUSINESS))
  (deffield organization/kind enum OrgKind)
  (node acme NodeType/ORGANIZATION (organization/kind OrgKind/NOWHERE)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-055"));
        assert!(err.message.contains("NOWHERE"), "{}", err.message);
    }

    #[test]
    fn an_enum_deffield_naming_an_undeclared_type_is_e_load_054() {
        let source = r"
(scenario org/t
  (deffield organization/kind enum Nowhere)
  (node acme NodeType/ORGANIZATION))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-054"));
    }

    #[test]
    fn enum_seeding_moves_no_existing_golden_bytes() {
        // A scenario with NO defenum/enum-deffield forms loads exactly as
        // it did before this train — an untouched, empty `EnumRegistry`.
        // `tick_goldens.rs` re-proves this at the byte level for
        // vitality-conformance.bscn on every run; this is the same claim
        // at the unit level, for the fixture this test module already uses.
        let mut first = MemoryGraph::new();
        let mut second = MemoryGraph::new();
        let a = load_scenario(TWO_CLASSES, &mut first).unwrap();
        let b = load_scenario(TWO_CLASSES, &mut second).unwrap();
        assert_eq!(first.state_hash().unwrap(), second.state_hash().unwrap());
        assert_eq!(a.node_count, 2);
        assert_eq!(b.node_count, 2);
    }

    // ---- §2.13/§3.6 `defvocabulary`: the closed graph vocabulary is
    // declared, never inferred (Task 7) ----

    /// #528 fix round, RED before the fix: the tree-sitter corpus's own
    /// worked example (`test/corpus/declarations.txt:145`) — bare
    /// `<enum-member>` items, never full `Type/MEMBER` refs. Today's
    /// `load_defvocabulary` requires the latter, so this fails with an
    /// uncoded "must be written `{kind_name}`/<MEMBER>" error before the fix.
    #[test]
    fn defvocabulary_accepts_the_corpus_line_verbatim() {
        let source = r"
(scenario org/vocab-corpus
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))
  (node acme NodeType/ORGANIZATION))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph)
            .expect("the corpus's own bare-member shape must load");
        let vocabulary = loaded.vocabulary.expect("a declared vocabulary is Some");
        assert_eq!(
            vocabulary
                .check_enum_ref("NodeType", "SOCIAL_CLASS")
                .unwrap(),
            EnumKind::NodeType
        );
    }

    #[test]
    fn a_scenario_declaring_the_vocabulary_loads_and_is_some() {
        let source = r"
(scenario org/vocab
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))
  (defvocabulary EdgeType
    (MEMBERSHIP PRESENCE COMMAND
     TRANSACTIONAL SOLIDARISTIC SOLIDARITY))
  (node acme NodeType/ORGANIZATION))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).expect("loads");
        let vocabulary = loaded.vocabulary.expect("a declared vocabulary is Some");
        assert_eq!(
            vocabulary
                .check_enum_ref("NodeType", "ORGANIZATION")
                .unwrap(),
            EnumKind::NodeType
        );
        assert_eq!(
            vocabulary.check_enum_ref("EdgeType", "SOLIDARITY").unwrap(),
            EnumKind::EdgeType
        );
    }

    #[test]
    fn a_scenario_with_no_defvocabulary_forms_yields_none() {
        // Backward compatibility: existing content (this test module's own
        // TWO_CLASSES fixture, and every scenario predating §2.13) declares
        // no `defvocabulary` at all — enforcement stays exactly as inert
        // as it is today.
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(TWO_CLASSES, &mut graph).expect("loads");
        assert!(loaded.vocabulary.is_none());
    }

    #[test]
    fn closed_vocabularys_own_rendering_collision_propagates_as_e_load_032() {
        // TENANCY under two structural kinds — §2.9's disjointness
        // obligation, `ClosedVocabulary::new`'s own check, reached through
        // `load_defvocabulary`'s collected map rather than reinvented here.
        let source = r"
(scenario org/collision
  (defvocabulary NodeType (TENANCY))
  (defvocabulary EdgeType (TENANCY))
  (node acme NodeType/TENANCY))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-032"));
    }

    #[test]
    fn an_unknown_enum_kind_symbol_refuses() {
        let source = r"
(scenario org/badkind
  (defvocabulary SovereignType (USA)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-030"));
    }

    #[test]
    fn two_defvocabulary_forms_for_one_kind_is_e_load_001() {
        let source = r"
(scenario org/twice
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary NodeType (TERRITORY)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-001"));
    }

    /// #528 fix round: repurposed from `a_defvocabulary_member_naming_a_
    /// different_kind_refuses` — under the bare-member reading a member
    /// carries no kind prefix to mismatch AT ALL (that whole error class is
    /// now structurally unreachable), so the meaningful sibling check is
    /// the grammar-conformance direction: a member written as a full
    /// enum-ref (even one that LOOKS like a plausible different kind, the
    /// way `EdgeType/SOLIDARITY` does here) still refuses.
    #[test]
    fn a_defvocabulary_member_written_as_a_full_enum_ref_refuses() {
        let source = r"
(scenario org/mismatched
  (defvocabulary NodeType (EdgeType/SOLIDARITY)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("bare"), "{}", err.message);
    }

    #[test]
    fn a_defvocabulary_kind_name_shaped_like_an_enum_member_refuses() {
        // Node_Type lexes fine (Atom::BareUpperIdent admits the union
        // charset) but is not one of the closed four kind names — the
        // EXISTING E-LOAD-030 check catches it (see `load_defvocabulary`'s
        // own doc for why no separate shape check is added here).
        let source = r"
(scenario org/badkind-shape
  (defvocabulary Node_Type (SOCIAL_CLASS)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-030"));
    }

    #[test]
    fn a_defvocabulary_member_shaped_like_an_enum_type_refuses() {
        // `SocialClass` has lowercase letters and no slash: it lexes fine
        // as Atom::BareUpperIdent, but is not a valid <enum-member> (which
        // permits no lowercase at all) — the parser must catch this, not
        // the reader.
        let source = r"
(scenario org/badmember-shape
  (defvocabulary NodeType (SocialClass)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert!(err.message.contains("enum-member"), "{}", err.message);
    }

    // ---- Task 8 (Organization foundation plan): closed-vocabulary
    // enforcement at hydration — `load_node`/`load_edge` check membership
    // BEFORE minting, when the scenario declared one ----

    #[test]
    fn an_unregistered_node_member_under_a_declared_vocabulary_is_e_load_031() {
        let source = r"
(scenario org/typo-node
  (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION))
  (node x NodeType/FOO))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-031"));
        assert!(err.message.contains("FOO"), "{}", err.message);
        // The node must never have minted — a loud refusal is not a
        // best-effort partial hydration.
        assert_eq!(graph.nodes("FOO").len(), 0);
    }

    #[test]
    fn an_unregistered_edge_member_under_a_declared_vocabulary_is_e_load_031() {
        let source = r"
(scenario org/typo-edge
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/NOWHERE a b 1))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-031"));
        assert!(err.message.contains("NOWHERE"), "{}", err.message);
    }

    #[test]
    fn a_registered_node_and_edge_member_load_clean_under_a_declared_vocabulary() {
        let source = r"
(scenario org/vocab-clean
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 1))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).expect("registered members must load");
        assert_eq!(loaded.node_count, 2);
        assert_eq!(loaded.edge_count, 1);
    }

    // ---- F2 (#534 fix round item 2): `load_node`/`load_edge` demand the
    // POSITION'S OWN kind — a node minted from `EdgeType/SOLIDARITY`
    // silently typed itself "SOLIDARITY" before this (panel-proven:
    // hardcoding "NodeType" at the call site flipped zero tests). ----

    #[test]
    fn load_node_demands_nodetype_regardless_of_what_is_declared() {
        // The matrix: declared-right-kind, undeclared-kind (F1
        // interaction — inert), wrong-kind — all three must enforce the
        // node position's own kind identically.

        // declared-right-kind: NodeType is declared, member is a typo.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/matrix-declared-right-kind
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node x NodeType/NOWHERE))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-031"), "{}", err.message);

        // undeclared-kind: NodeType is never declared (only EdgeType is) —
        // the KIND matches the node position, and F1 leaves NodeType's own
        // membership inert, so this loads clean.
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(
            r"
(scenario org/matrix-undeclared-kind
  (defvocabulary EdgeType (SOLIDARITY))
  (node x NodeType/ANYTHING))
",
            &mut graph,
        )
        .expect("NodeType was never declared — position-kind matches, membership stays inert");
        assert_eq!(loaded.node_count, 1);

        // wrong-kind: NodeType IS declared, but the written ref names
        // EdgeType — refused as a kind mismatch, never silently minted as
        // a node typed "SOLIDARITY". G2 (#534 fix round 2 item 2):
        // EdgeType is a REAL structural kind, just the wrong one for a
        // node's position — E-TYPE-011, never E-LOAD-030 (that code is now
        // reserved for a type name registered nowhere at all).
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/matrix-wrong-kind
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (node x EdgeType/SOLIDARITY))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-TYPE-011"), "{}", err.message);
        assert_eq!(graph.nodes("SOLIDARITY").len(), 0);
    }

    #[test]
    fn load_node_refuses_the_org_kind_business_probe_verbatim() {
        // The panel's exact probe: `OrgKind` is not declared ANYWHERE in
        // this scenario — no `defvocabulary` (it is not a structural kind
        // name to begin with) and no `defenum` either — so it names
        // nothing real at all: the genuine E-LOAD-030 case. Contrast
        // `load_node_refuses_the_org_kind_business_probe_when_orgkind_is_
        // declared_too` below, the G2 sibling where OrgKind IS real (via
        // `defenum`) but still wrong for a node's position (E-TYPE-011).
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/org-kind-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node x OrgKind/BUSINESS))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-030"), "{}", err.message);
    }

    #[test]
    fn load_node_with_an_unregistered_enum_carries_an_enum_identity() {
        // Task 2 (issue #652 §2.1c): the same OrgKind/BUSINESS probe as
        // `load_node_refuses_the_org_kind_business_probe_verbatim`, this
        // time asserting the `Enum` identity `vocab_err` now derives from
        // the wrapped `VocabularyError::UnknownEnumType` instead of
        // discarding it.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/org-kind-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node x OrgKind/BUSINESS))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(
            err.identity,
            Some(ErrorIdentity::Enum {
                enum_type: "OrgKind".to_owned(),
                member: Some("BUSINESS".to_owned()),
            }),
            "{}",
            err.message
        );
    }

    #[test]
    fn load_node_refuses_the_org_kind_business_probe_when_orgkind_is_declared_too() {
        // G2's sibling probe (#534 fix round 2 item 2): the SAME `(node x
        // OrgKind/BUSINESS)` shape, but `OrgKind` IS a real,
        // scenario-declared `defenum` type this time — not nothing at
        // all, just the wrong kind for a node's own position.
        // `demand_enum_kind` must tell these two facts apart: E-LOAD-030
        // above (OrgKind exists nowhere); E-TYPE-011 here (OrgKind exists,
        // but a node's position demands NodeType, not a content-declared
        // enum type).
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/org-kind-declared-probe
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum OrgKind (BUSINESS))
  (node x OrgKind/BUSINESS))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-TYPE-011"), "{}", err.message);
    }

    #[test]
    fn demand_enum_kinds_split_is_also_registry_relative_not_just_positional() {
        // H2 (#534 fix round 3): disclosure pin — the E-TYPE-011/
        // E-LOAD-030 split is POSITIONAL (G3(c) below: unconditional, not
        // vocabulary-gated) AND REGISTRY-RELATIVE: a scenario-declared
        // `defenum` type participates in "is this a REAL type" only from
        // ITS OWN declaration point down, the same "declaration must
        // precede use" discipline `deffield`/`defconst`/`defvocabulary`
        // already enforce, consistent with `vocabulary_so_far`. The SAME
        // `(node x OrgKind/BUSINESS)` probe, both orderings — that pair of
        // facts IS the contract.
        let mut declared_first = MemoryGraph::new();
        let err_declared_first = load_scenario(
            r"
(scenario org/org-kind-order-declared-first
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defenum OrgKind (BUSINESS))
  (node x OrgKind/BUSINESS))
",
            &mut declared_first,
        )
        .unwrap_err();
        assert_eq!(
            err_declared_first.code,
            Some("E-TYPE-011"),
            "OrgKind exists by the time the node form runs: {}",
            err_declared_first.message
        );

        let mut declared_after = MemoryGraph::new();
        let err_declared_after = load_scenario(
            r"
(scenario org/org-kind-order-declared-after
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node x OrgKind/BUSINESS)
  (defenum OrgKind (BUSINESS)))
",
            &mut declared_after,
        )
        .unwrap_err();
        assert_eq!(
            err_declared_after.code,
            Some("E-LOAD-030"),
            "OrgKind names nothing YET at this point in the load — not a bug: {}",
            err_declared_after.message
        );
    }

    // ---- G3(c) (#534 fix round 2): F1×F2 interaction pins —
    // `demand_enum_kind`'s split is POSITIONAL, never vocabulary-gated. ----

    #[test]
    fn a_wrong_kind_ref_refuses_even_when_that_kind_was_never_declared_via_defvocabulary() {
        // Only NodeType is declared here; EdgeType is not. Whether
        // EdgeType itself was ever `defvocabulary`-declared in THIS
        // scenario is irrelevant to whether it names a REAL structural
        // kind — EdgeType/… at a node's position is still E-TYPE-011,
        // never conflated with F1's own inertness rule (which governs
        // MEMBERSHIP checking of a kind, a separate, opt-in concern from
        // this KIND-position check).
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/wrong-kind-undeclared-vocab
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node x EdgeType/SOLIDARITY))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-TYPE-011"), "{}", err.message);
    }

    #[test]
    fn a_wrong_kind_ref_refuses_even_with_no_defvocabulary_at_all() {
        // The unconditional half of `demand_enum_kind`'s own doc: a wrong
        // KIND at a node/edge's position is checked independent of
        // whether ANY `defvocabulary` was declared — unlike the SEPARATE
        // membership check a threaded `ClosedVocabulary` performs, this
        // one is not opt-in (§3.9 clause 1: "hydration is not a back door
        // into the closed vocabulary").
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/wrong-kind-no-vocab-at-all
  (node x EdgeType/SOLIDARITY))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-TYPE-011"), "{}", err.message);
    }

    #[test]
    fn load_edge_demands_edgetype_regardless_of_what_is_declared() {
        // The EdgeType-position mirror of
        // `load_node_demands_nodetype_regardless_of_what_is_declared`.

        // declared-right-kind: typo under a declared EdgeType.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/edge-matrix-declared-right-kind
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/NOWHERE a b 1))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-031"), "{}", err.message);

        // undeclared-kind: EdgeType never declared (only NodeType is) —
        // kind matches the edge position, membership stays inert (F1).
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(
            r"
(scenario org/edge-matrix-undeclared-kind
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/ANYTHING a b 1))
",
            &mut graph,
        )
        .expect("EdgeType was never declared — position-kind matches, membership stays inert");
        assert_eq!(loaded.edge_count, 1);

        // wrong-kind: EdgeType IS declared, but the written ref names
        // NodeType — refused as a kind mismatch. G2 (#534 fix round 2 item
        // 2): NodeType is a REAL structural kind, just the wrong one for
        // an edge's position — E-TYPE-011.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario org/edge-matrix-wrong-kind
  (defvocabulary NodeType (SOCIAL_CLASS))
  (defvocabulary EdgeType (SOLIDARITY))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge NodeType/SOCIAL_CLASS a b 1))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-TYPE-011"), "{}", err.message);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn an_edge_under_an_undeclared_edge_type_is_inert_not_e_load_031() {
        // F1 (#534 fix round item 1): a vocabulary declaring ONLY NodeType
        // must leave EdgeType's own membership checking inert — an
        // undeclared MEMBER of an UNDECLARED kind must never be conflated
        // with an undeclared member of a DECLARED kind (that stays
        // E-LOAD-031, `an_unregistered_edge_member_under_a_declared_
        // vocabulary_is_e_load_031` above).
        let source = r"
(scenario org/edge-type-undeclared
  (defvocabulary NodeType (SOCIAL_CLASS))
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/ANYTHING a b 1))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph)
            .expect("EdgeType was never declared — its checking must stay inert");
        assert_eq!(loaded.edge_count, 1);
    }

    #[test]
    fn the_same_typo_source_loads_with_no_defvocabulary_declared_backward_compat_pin() {
        // Task 8's own backward-compat pin: the SAME node-type typo, with
        // NO `defvocabulary` form at all, loads exactly as it did before
        // this task — membership is opt-in per scenario.
        let source = r"
(scenario org/typo-node-unchecked
  (node x NodeType/FOO))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph)
            .expect("with no declared vocabulary, membership is unchecked (backward compat)");
        assert_eq!(loaded.node_count, 1);
        assert!(loaded.vocabulary.is_none());
    }

    #[test]
    fn a_node_before_any_defvocabulary_form_is_unchecked_declaration_precedes_use() {
        // A vocabulary declared LATER in the file cannot retroactively
        // check a node minted before it — same "declaration must precede
        // use" discipline `deffield`/`defenum`/`defconst` already carry.
        let source = r"
(scenario org/vocab-after
  (node x NodeType/FOO)
  (defvocabulary NodeType (SOCIAL_CLASS)))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph)
            .expect("a node minted before any defvocabulary form is unchecked");
        assert_eq!(loaded.node_count, 1);
    }

    // ---- Train B item 3 (#591): the (edge-attr ...) scenario form ----

    #[test]
    fn edge_attr_seeds_a_declared_edge_field() {
        // The positive lane: a declared edge field seeded onto an edge the
        // same scenario already minted, read back through the substrate's
        // fifth-section edge-attribute store (section 0x05 — the D143 fork
        // does not engage, the qname does not end in `/strength`). Bit-exact
        // pin, not a tolerance: 0.25 = 2^-2 is dyadic-exact, so 25/100
        // divides to it exactly under the crate's one scaled-literal
        // conversion contract.
        let source = r"
(scenario ft/edge-attr
  (deffield solidarity/tension intensity intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/tension 0.25i))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        assert_eq!(loaded.edge_count, 1, "edge-attr mints no edge of its own");
        let value = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/tension")
            .unwrap();
        assert_eq!(value.to_bits(), (0.25_f64).to_bits());
        // …and the strength slot the `edge` form seeded is untouched.
        let strength = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
            .unwrap();
        assert_eq!(strength.to_bits(), (0.5_f64).to_bits());
    }

    #[test]
    fn edge_attr_refuses_unknown_edge_undeclared_field_strength_and_currency() {
        // The four refusals, each a fresh scenario:
        //
        // 1. An edge-attr naming an edge this scenario never seeded — both
        //    endpoints exist as NODES, which is what makes this the
        //    edge-existence refusal rather than the unknown-node one. Loud,
        //    naming the endpoints (the form has no local name of its own —
        //    load_edge's own F6 convention).
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-unknown-edge
  (deffield solidarity/tension intensity intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node c NodeType/SOCIAL_CLASS)
  (edge-attr EdgeType/SOLIDARITY a c solidarity/tension 0.5i))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("a → c") && err.message.contains("no such edge"),
            "{}",
            err.message
        );

        // 2. An undeclared field qname is a typo, not a new field — the
        //    same registry contract load_node enforces, one element kind
        //    over. Loud, naming the field.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-undeclared
  (deffield solidarity/tension intensity intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/nope 0.5i))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("solidarity/nope") && err.message.contains("never declared"),
            "{}",
            err.message
        );

        // 3. `solidarity/strength` — D32 kinds `<edge-type>/strength` an
        //    IMPLICIT field: it is never in a scenario's deffield registry,
        //    and strength seeds via the (edge ...) form's own 4th slot
        //    only. Refused unconditionally (NOT merely via the registry
        //    miss): a scenario CAN `(deffield solidarity/strength …)` —
        //    `load_deffield` accepts it and only `prepare_rules`'s E-LOAD-001
        //    refuses it later — and without this guard the write would fall
        //    into the substrate's `/strength` fork (D143) and silently
        //    rewrite the edge's mint strength slot.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-strength
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/strength 0.5c))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("strength") && err.message.contains("IMPLICIT"),
            "{}",
            err.message
        );

        // 4. Currency — node-scoped typed storage now exists (T3 #491,
        //    OQ-J), but there is still no edge-scoped Currency lane, so this
        //    refusal survives with an UPDATED reason.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-currency
  (deffield solidarity/cost currency intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/cost 5$))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("NODE attributes only"),
            "{}",
            err.message
        );
        assert!(err.message.contains("no edge-scoped"), "{}", err.message);
    }

    #[test]
    fn edge_attr_refuses_a_field_owned_by_another_type() {
        // The qname's owner segment must name the edge's own type — §2.10
        // discipline 1's ownership law, checked at hydration exactly as
        // `check_edge_referent_type` checks it at evaluation. `wages/
        // value-flow` is a legal WAGES-edge field (Task 4's consumer); on a
        // SOLIDARITY edge it is an owner mismatch.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-foreign-owner
  (deffield wages/value-flow real intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b wages/value-flow 5))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("wages/value-flow") && err.message.contains("SOLIDARITY"),
            "{}",
            err.message
        );
    }

    #[test]
    fn edge_attr_refuses_a_double_seed() {
        // Two edge-attr forms with the same (edge-type, source, target,
        // field) key — mirrors E-LOAD-044's own argument one axis wider:
        // the quadruple is a KEY, and a second seeding would silently
        // overwrite the first (only the later value surviving in the file
        // that carries both).
        let source = r"
(scenario ft/edge-attr-dup
  (deffield solidarity/tension intensity intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/tension 0.25i)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/tension 0.5i))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-057"), "{}", err.message);
    }

    #[test]
    fn edge_attr_refuses_a_declared_strength_field_too() {
        // F2 (Task 3 fix round 1): the guard's motivating variant.
        // `load_deffield` has no implicit-field notion and ACCEPTS
        // `(deffield solidarity/strength …)` (only `prepare_rules`'s
        // E-LOAD-001 refuses it, later, at `TypeEnv` construction — D139),
        // so here the deffield-registry lookup would SUCCEED and the
        // unconditional `/strength` guard is the SOLE refusal between this
        // form and the D143 fork's silent rewrite of the mint strength
        // slot. The refusal must name the field and the D32 reason.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/edge-attr-declared-strength
  (deffield solidarity/strength coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/strength 0.25c))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("solidarity/strength") && err.message.contains("IMPLICIT"),
            "{}",
            err.message
        );
        // And the mint strength slot is untouched — the guard fired before
        // any write reached the substrate.
        let strength = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/strength")
            .unwrap();
        assert_eq!(strength.to_bits(), (0.5_f64).to_bits());
    }

    // ---- Community port train Task 6 (E2b): the (hyperedge-attr ...) scenario form ----

    #[test]
    fn hyperedge_attr_seeds_a_declared_hyperedge_field() {
        // The positive lane, mirroring `edge_attr_seeds_a_declared_edge_field`
        // one element kind over: the seed lands in the substrate's Task-5
        // hyperedge-attribute store and reads back bit-exact (0.25 = 2^-2 is
        // dyadic-exact). The write travels NO write log — hydration has none
        // (the write log is a tick-time construct; `load_edge_attr`'s own
        // direct `update_edge` is the precedent), which is exactly why this
        // test reads the substrate rather than a log.
        let source = r"
(scenario ft/hyperedge-attr
  (deffield community/heat coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a b))
  (hyperedge-attr cell community/heat 0.25c))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario(source, &mut graph).unwrap();
        let value = graph
            .hyperedge_attribute(HyperedgeId(0), "community/heat")
            .unwrap();
        assert_eq!(value.to_bits(), (0.25_f64).to_bits());
        assert_eq!(
            loaded.hyperedge_content_ids.get(&HyperedgeId(0)),
            Some(&"cell".to_owned())
        );
    }

    #[test]
    fn hyperedge_attr_seeds_an_enum_typed_field_as_its_ordinal() {
        // The enum lane (`community/kind`, Task 6 Step 4's named case):
        // seeded ONLY as <EnumType>/<MEMBER> (`attribute_value_enum`'s law),
        // stored as the ordinal — never a surface number.
        let source = r"
(scenario ft/hyperedge-attr-enum
  (defenum CommunityType (REVOLUTIONARY LIBERAL))
  (deffield community/kind enum CommunityType)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a b))
  (hyperedge-attr cell community/kind CommunityType/LIBERAL))
";
        let mut graph = MemoryGraph::new();
        load_scenario(source, &mut graph).unwrap();
        let value = graph
            .hyperedge_attribute(HyperedgeId(0), "community/kind")
            .unwrap();
        assert_eq!(value.to_bits(), (1.0_f64).to_bits(), "LIBERAL is ordinal 1");
    }

    #[test]
    fn hyperedge_attr_refuses_a_fractional_seed_on_an_int_declared_field() {
        // The int/fractional conversion contract, verbatim — the same
        // refusal `load_node`'s field-init path pins executably above.
        let source = r"
(scenario ft/hyperedge-attr-int
  (deffield community/count int extensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a b))
  (hyperedge-attr cell community/count 0.5c))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(
            err.message,
            "hyperedge `cell` field `community/count`: expected an integer \
             literal, found Scaled(ScaledLit { kind: Coefficient, unscaled: 5, scale: 1 })"
        );
    }

    #[test]
    fn hyperedge_attr_refuses_unknown_name_undeclared_field_owner_mismatch_and_double_seed() {
        // 1. A hyperedge-attr naming a hyperedge this scenario never
        //    seeded — the top-to-bottom discipline, by name.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/hyperedge-attr-unknown
  (deffield community/heat coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (hyperedge-attr ghost community/heat 0.5c))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message
                .contains("hyperedge-attr names unknown hyperedge `ghost`"),
            "{}",
            err.message
        );

        // 2. A field no deffield declared is a typo, not a new field.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/hyperedge-attr-typo
  (deffield community/heat coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a b))
  (hyperedge-attr cell community/heta 0.5c))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message.contains("community/heta") && err.message.contains("never declared"),
            "{}",
            err.message
        );

        // 3. §2.10 discipline 1 at hydration: the qname's owner must name
        //    the hyperedge's own type, read back from the substrate.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/hyperedge-attr-owner
  (deffield economic-sector/output coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a))
  (hyperedge-attr cell economic-sector/output 0.5c))
",
            &mut graph,
        )
        .unwrap_err();
        assert!(
            err.message
                .contains("is owned by ECONOMIC_SECTOR, not COMMUNITY"),
            "{}",
            err.message
        );

        // 4. The (hyperedge, field) pair is a KEY — a second seeding of one
        //    key is E-LOAD-057, the silent-overwrite refusal.
        let mut graph = MemoryGraph::new();
        let err = load_scenario(
            r"
(scenario ft/hyperedge-attr-double
  (deffield community/heat coefficient intensive)
  (node a NodeType/SOCIAL_CLASS)
  (node b NodeType/SOCIAL_CLASS)
  (hyperedge cell HyperedgeType/COMMUNITY (members a b))
  (hyperedge-attr cell community/heat 0.25c)
  (hyperedge-attr cell community/heat 0.5c))
",
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(err.code, Some("E-LOAD-057"), "{}", err.message);
    }

    // ---- F1 (Task 3 fix round 1): the node-path refusal text, pinned EXECUTABLY ----

    #[test]
    fn a_fractional_seed_into_an_int_field_is_refused_with_the_pinned_verbatim_message() {
        // The descriptor refactor (Train B item 3, deviation 1) drifted
        // every `attribute_value`-family message by two backticks and every
        // gate stayed green — the one committed verbatim record,
        // consciousness-ternary-conformance.bscn:101-106's comment quoting
        // this exact refusal, is not executable. This assertion is the FULL
        // string, not a substring: the descriptor self-quotes ("node `…`")
        // and the family's format strings name `{local}` bare, so the
        // emitted bytes are the pre-Task-3 rendering the .bscn comment
        // quotes. If this message's text ever changes again, this test — not
        // a comment — is what goes red.
        let source = r"
(scenario ft/verbatim-refusal
  (deffield social-class/agitation int intensive)
  (node class-exploited NodeType/SOCIAL_CLASS (social-class/agitation 0.1i)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario(source, &mut graph).unwrap_err();
        assert_eq!(
            err.message,
            "node `class-exploited` field `social-class/agitation`: expected an integer \
             literal, found Scaled(ScaledLit { kind: Intensity, unscaled: 1, scale: 1 })"
        );
    }

    // ---- Train B item 4 (#591, D157): scenario-declaration sharing via
    // `load_scenario_with_prelude` ----

    const WORLDVIEW_PRELUDE: &str = r"
(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
";

    #[test]
    fn a_prelude_defenum_resolves_a_scenario_enum_field() {
        let source = r"
(scenario org/prelude-t
  (deffield social-class/dominant-worldview enum WorldView)
  (node core NodeType/SOCIAL_CLASS (social-class/dominant-worldview WorldView/FASCIST)))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario_with_prelude(WORLDVIEW_PRELUDE, source, &mut graph)
            .expect("the prelude's WorldView type resolves the scenario's enum field");
        let id = graph.nodes("SOCIAL_CLASS")[0];
        let stored = graph
            .node_attribute(id, "social-class/dominant-worldview")
            .unwrap();
        assert!(
            (stored - 2.0).abs() < 1e-12,
            "FASCIST is declaration-order index 2, stored: {stored}"
        );
        assert!(loaded.enums.resolve("WorldView").is_some());
    }

    #[test]
    fn a_prelude_node_form_is_refused_loudly_naming_the_form() {
        let prelude = r"
(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
(node ghost NodeType/SOCIAL_CLASS)
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario_with_prelude(prelude, "(scenario org/t)", &mut graph).unwrap_err();
        // The interpolated head, not the static "node/edge/edge-attr forms
        // belong in the scenario" text every one of these three refusals
        // shares — a broken interpolation (always naming e.g. `defenum`)
        // must fail this, not slip through on the shared substring.
        assert!(err.message.contains("found `node`"), "{}", err.message);
    }

    #[test]
    fn a_prelude_edge_form_is_refused_loudly_naming_the_form() {
        let prelude = r"
(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
(edge EdgeType/SOLIDARITY a b 1)
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario_with_prelude(prelude, "(scenario org/t)", &mut graph).unwrap_err();
        // Not a bare `contains("edge")` — that would also pass against the
        // static "node/edge/edge-attr forms belong in the scenario" text
        // even if `{tag}` interpolated the WRONG head (or "edge-attr",
        // which also contains "edge"). The interpolated head must be
        // exactly `edge`, not merely a substring match.
        assert!(
            err.message.contains("found `edge`") && !err.message.contains("found `edge-attr`"),
            "{}",
            err.message
        );
    }

    #[test]
    fn a_prelude_edge_attr_form_is_refused_loudly_naming_the_form() {
        let prelude = r"
(defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
(edge-attr EdgeType/SOLIDARITY a b solidarity/strength 0.5c)
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario_with_prelude(prelude, "(scenario org/t)", &mut graph).unwrap_err();
        assert!(err.message.contains("found `edge-attr`"), "{}", err.message);
    }

    #[test]
    fn a_scenario_identically_redeclaring_the_preludes_enum_still_loads() {
        // The recognition arm (`EnumRegistry::declare`, this train): a
        // scenario re-declaring exactly what the prelude declared is not a
        // conflict.
        let source = r"
(scenario org/redeclare-identical
  (defenum WorldView (REVOLUTIONARY LIBERAL FASCIST))
  (deffield social-class/dominant-worldview enum WorldView)
  (node core NodeType/SOCIAL_CLASS (social-class/dominant-worldview WorldView/LIBERAL)))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario_with_prelude(WORLDVIEW_PRELUDE, source, &mut graph)
            .expect("an identical re-declaration must not refuse");
        let ty = loaded.enums.resolve("WorldView").unwrap();
        assert_eq!(loaded.enums.ordinal(ty, "LIBERAL"), Some(1));
    }

    #[test]
    fn a_scenario_differently_redeclaring_the_preludes_enum_refuses() {
        let source = r"
(scenario org/redeclare-conflict
  (defenum WorldView (LIBERAL REVOLUTIONARY FASCIST))
  (node core NodeType/SOCIAL_CLASS))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario_with_prelude(WORLDVIEW_PRELUDE, source, &mut graph).unwrap_err();
        assert!(
            err.message.contains("duplicate defenum type name"),
            "{}",
            err.message
        );
    }

    // ---- Final whole-branch review item 1 (#591): the three non-defenum
    // prelude forms were dispatched (a compiler-covered fact — four match
    // arms) but their THREADING into `prepare_rules`'s consumers had zero
    // executable backing behind a claim repeated in five normative places
    // (§2.13, D157, `load_scenario_with_prelude`'s rustdoc, `load_prelude`'s
    // rustdoc, `worldview.bscn`'s header). All six prior prelude tests are
    // `defenum`-centric; these two close the other three admitted kinds. ----

    #[test]
    fn a_prelude_deffield_defconst_and_defvocabulary_thread_into_a_scenario() {
        // One prelude declares a NODE field, an EDGE field, a const and a
        // two-kind vocabulary; the scenario seeds a node against the node
        // field, an `(edge-attr ...)` (D156) against the edge field, and
        // mints a node and an edge against the vocabulary (D101/§3.6) — the
        // three threading paths the final review named as untested:
        // `lib.rs:190,220` (fields), `:226,:357` (`:const`), `:191,:334`
        // (vocabulary), plus `ClosedVocabulary::new`'s own `E-LOAD-032`
        // disjointness check surviving a prelude+scenario split across kinds.
        let prelude = r"
(deffield social-class/agitation intensity intensive)
(deffield solidarity/tension intensity intensive)
(defconst t/coeff 0.5c)
(defconst t/likelihood 0.25p)
(defvocabulary NodeType (SOCIAL_CLASS))
(defvocabulary EdgeType (SOLIDARITY))
";
        let source = r"
(scenario org/prelude-threading
  (node a NodeType/SOCIAL_CLASS (social-class/agitation 0.3i))
  (node b NodeType/SOCIAL_CLASS)
  (edge EdgeType/SOLIDARITY a b 0.5c)
  (edge-attr EdgeType/SOLIDARITY a b solidarity/tension 0.25i))
";
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario_with_prelude(prelude, source, &mut graph).expect(
            "a prelude-declared deffield/defconst/defvocabulary must thread into the scenario",
        );

        // deffield (node field): the prelude's type/kind resolved the
        // scenario's node attribute write.
        let agitation = graph
            .node_attribute(NodeId(0), "social-class/agitation")
            .unwrap();
        assert_eq!(agitation.to_bits(), (0.3_f64).to_bits());

        // deffield (edge field, via D156's edge-attr form): the prelude's
        // field resolved a scenario `(edge-attr ...)` write.
        let tension = graph
            .edge_attribute("SOLIDARITY", NodeId(0), NodeId(1), "solidarity/tension")
            .unwrap();
        assert_eq!(tension.to_bits(), (0.25_f64).to_bits());

        // defconst: threaded into `LoadedScenario.consts`, the exact map
        // `prepare_rules` reads for a `:const` binding.
        assert_eq!(
            loaded.consts.get("t/coeff"),
            Some(&crate::evaluator::Value::Real(0.5))
        );
        assert_eq!(
            loaded.probability_consts,
            HashSet::from(["t/likelihood".to_owned()]),
            "the prelude must retain Probability declaration identity without classifying the numerically equal coefficient"
        );

        // defvocabulary: threaded into `LoadedScenario.vocabulary` — the
        // SAME registry the node/edge minting above checked against BEFORE
        // minting (a typo'd type in either form above would have refused).
        let vocabulary = loaded
            .vocabulary
            .expect("a prelude-declared vocabulary is Some");
        assert_eq!(
            vocabulary
                .check_enum_ref("NodeType", "SOCIAL_CLASS")
                .unwrap(),
            EnumKind::NodeType
        );
        assert_eq!(
            vocabulary.check_enum_ref("EdgeType", "SOLIDARITY").unwrap(),
            EnumKind::EdgeType
        );
    }

    #[test]
    fn a_probability_defconst_duplicate_across_prelude_and_scenario_remains_loud() {
        let prelude = "(defconst event/likelihood 0.25p)\n";
        let source = r"
(scenario org/duplicate-probability-const
  (defconst event/likelihood 0.25p))
";
        let mut graph = MemoryGraph::new();
        let error = load_scenario_with_prelude(prelude, source, &mut graph).unwrap_err();

        assert!(error.message.contains("duplicate defconst"), "{error}");
        assert_eq!(
            error.identity,
            Some(ErrorIdentity::Name("event/likelihood".to_owned()))
        );
    }

    #[test]
    fn named_multi_prelude_mass_facts_are_scenario_first_and_source_local() {
        let scenario = "(scenario org/named-mass (defconst demo/scenario-mass 3m))";
        let prelude_a = "(defenum Mood (CALM ANGRY))\n(defconst demo/prelude-a-mass 2m)\n";
        let prelude_b = "(defconst demo/prelude-b-mass 4m)\n";
        let preludes = [
            NamedDeclarationPreludeV1 {
                source_id: "content/a.bsl",
                source: prelude_a,
            },
            NamedDeclarationPreludeV1 {
                source_id: "content/b.bsl",
                source: prelude_b,
            },
        ];
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario_with_named_preludes(
            "content/scenario.bscn",
            scenario,
            &preludes,
            &mut graph,
        )
        .unwrap();

        let facts = &loaded.mass_declarations;
        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].source_id, "content/scenario.bscn");
        assert_eq!(facts[0].qname, "demo/scenario-mass");
        assert_eq!(facts[0].form_path, [0, 2, 2]);
        assert_eq!(facts[1].source_id, "content/a.bsl");
        assert_eq!(facts[1].qname, "demo/prelude-a-mass");
        assert_eq!(facts[1].form_path, [1, 2]);
        assert_eq!(facts[2].source_id, "content/b.bsl");
        assert_eq!(facts[2].qname, "demo/prelude-b-mass");
        assert_eq!(facts[2].form_path, [0, 2]);

        for (source, fact, expected_literal) in [
            (scenario, &facts[0], "3m"),
            (prelude_a, &facts[1], "2m"),
            (prelude_b, &facts[2], "4m"),
        ] {
            let (_, spans) = read_all_spanned(source.as_bytes()).unwrap();
            let span = spans.span_of(&fact.form_path).unwrap();
            assert_eq!(&source[span.start..span.end], expected_literal);
        }
    }

    #[test]
    fn named_preludes_share_one_registry_in_manifest_order() {
        let preludes = [
            NamedDeclarationPreludeV1 {
                source_id: "content/types.bsl",
                source: "(defenum Mood (CALM ANGRY))\n",
            },
            NamedDeclarationPreludeV1 {
                source_id: "content/fields.bsl",
                source: "(deffield social-class/mood enum Mood)\n",
            },
        ];
        let scenario = r"
(scenario org/named-registry
  (node workers NodeType/SOCIAL_CLASS (social-class/mood Mood/ANGRY)))
";
        let mut graph = MemoryGraph::new();
        load_scenario_with_named_preludes("content/scenario.bscn", scenario, &preludes, &mut graph)
            .expect("the second prelude must resolve the first prelude's enum");
        assert_eq!(
            graph
                .node_attribute(NodeId(0), "social-class/mood")
                .unwrap()
                .to_bits(),
            1.0_f64.to_bits()
        );
    }

    #[test]
    fn named_prelude_duplicate_and_composition_refusals_remain_loud() {
        let duplicate = [
            NamedDeclarationPreludeV1 {
                source_id: "content/first.bsl",
                source: "(defconst demo/mass 1m)\n",
            },
            NamedDeclarationPreludeV1 {
                source_id: "content/second.bsl",
                source: "(defconst demo/mass 1m)\n",
            },
        ];
        let mut graph = MemoryGraph::new();
        let error = load_scenario_with_named_preludes(
            "content/scenario.bscn",
            "(scenario org/duplicate)",
            &duplicate,
            &mut graph,
        )
        .unwrap_err();
        assert!(error.message.contains("duplicate defconst"), "{error}");

        let too_many = [NamedDeclarationPreludeV1 {
            source_id: "content/repeated.bsl",
            source: "(defconst demo/mass 1m)\n",
        }; 17];
        let mut graph = MemoryGraph::new();
        let error = load_scenario_with_named_preludes(
            "content/scenario.bscn",
            "(scenario org/too-many)",
            &too_many,
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(
            error.message,
            "a content set may compose at most 16 declaration preludes"
        );
    }

    #[test]
    fn named_prelude_bom_behavior_matches_the_historical_composed_stream() {
        let first_bom = [NamedDeclarationPreludeV1 {
            source_id: "content/first.bsl",
            source: "\u{feff}(defconst demo/mass 1m)\n",
        }];
        let mut graph = MemoryGraph::new();
        let loaded = load_scenario_with_named_preludes(
            "content/scenario.bscn",
            "(scenario org/first-bom)",
            &first_bom,
            &mut graph,
        )
        .expect("an offset-zero BOM in the first composed source remains legal");
        assert_eq!(loaded.mass_declarations[0].source_id, "content/first.bsl");

        let second_bom = [
            NamedDeclarationPreludeV1 {
                source_id: "content/first.bsl",
                source: "(defconst demo/first 1m)\n",
            },
            NamedDeclarationPreludeV1 {
                source_id: "content/second.bsl",
                source: "\u{feff}(defconst demo/second 2m)\n",
            },
        ];
        let mut graph = MemoryGraph::new();
        let error = load_scenario_with_named_preludes(
            "content/scenario.bscn",
            "(scenario org/second-bom)",
            &second_bom,
            &mut graph,
        )
        .unwrap_err();
        assert_eq!(error.code, Some("E-LEX-001"));
        assert_eq!(error.position, Some(second_bom[0].source.len()));
    }

    #[test]
    fn a_scenario_redeclaring_the_preludes_deffield_refuses() {
        // The asymmetry `load_scenario_with_prelude`'s own rustdoc (:374-380)
        // spends a paragraph on, proved for neither direction before this
        // test: `defenum` alone grew the identical-recognition arm this
        // train; `deffield` (like `defconst`/`defvocabulary`) kept its
        // pre-existing UNCONDITIONAL collision check
        // (`fields.insert(...).is_some()`) — a scenario re-declaring a
        // prelude-supplied `deffield` refuses even byte-for-byte identical.
        let prelude = r"
(deffield social-class/agitation intensity intensive)
";
        let source = r"
(scenario org/redeclare-deffield
  (deffield social-class/agitation intensity intensive)
  (node a NodeType/SOCIAL_CLASS (social-class/agitation 0.3i)))
";
        let mut graph = MemoryGraph::new();
        let err = load_scenario_with_prelude(prelude, source, &mut graph).unwrap_err();
        assert!(
            err.message
                .contains("duplicate deffield `social-class/agitation`"),
            "{}",
            err.message
        );
    }
}
