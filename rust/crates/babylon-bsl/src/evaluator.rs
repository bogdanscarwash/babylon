//! The fuel-metered BSL expression evaluator (`bsl-language.rst` §4) — the
//! RUNTIME backstop to Task 13's static bound (§3.7 computes the worst case
//! at load; §4.5's meter still runs, because the static bound's own
//! soundness is exactly what an adversarial reviewer should doubt).
//!
//! Scope (Phase 1 Task 14): the **expression core** — literals, variable
//! references, the two numeric lanes (§3.3), strictly binary arithmetic,
//! comparison, `and`/`or`/`not`, `if`, and the [`crate::intrinsic_host`]
//! boundary. `guard` and the §2.8 effect verbs are grammar errors here
//! (they are EFFECT-position only, and already served there by
//! [`crate::structural_verbs`]); folds, queries, selections and accessors
//! are a loud error naming the query-evaluation-plan slice that will serve
//! them (P27 Phase 2 Task 1 — `EFFECT_POSITION_ONLY` /
//! `UNSERVED_EXPRESSION_HEADS`), never a default.
//!
//! Semantics held to the letter of §4:
//! - **§4.1**: strict, call-by-value, left to right; `and`/`or`
//!   short-circuit and `if` evaluates only the taken branch — the two
//!   deliberate exceptions.
//! - **§4.3**: binary64 ops are the IEEE-754 basic set; `Int` overflow is
//!   `E-EVAL-011`; binary64 division by zero is `E-EVAL-012`; a non-finite
//!   result is `E-EVAL-014` — never representable. Currency follows the
//!   §3.2 operator table via `babylon_kernel::Currency`'s pinned operators
//!   (the panic preconditions checked here first, so a rule failure is a
//!   structured tick-abort, not a process abort).
//! - **§4.5**: each AST node charges its **base** cost when it is evaluated
//!   (the §3.7 numbers without the ceiling multiplication); reaching or
//!   passing zero is `E-EVAL-040`.
//!
//! Two implementation-discovered notes, recorded in the reference:
//! - The §3.7/§4.5 boundary is off by one: a rule whose worst case consumes
//!   exactly its `:fuel` loads (§3.7 rejects only `bound > :fuel`) yet
//!   `E-EVAL-040`s at runtime, because the meter must stay strictly
//!   positive ("reaching or passing zero"). Noted in §4.5.
//! - `Int ÷ Int` has no pinned semantics: truncation is never implicit
//!   (§3.2) and §3.3 promotes `Int` only "in a binary64 expression". It is
//!   a loud error here pending the Phase-1 review.

use crate::fuel::{cost, IntrinsicCosts};
use crate::intrinsic_host::{
    DrawActiveElement, DrawContext, DrawIdentityContext, IntrinsicCallCtx, IntrinsicHost,
};
use crate::probability::{Mass, QUANTIZE_MASS_BASE};
use crate::query::{EdgeKey, Element};
use crate::reader::{Atom, SExpr, ScaledKind};
use crate::typecheck::TypeEnv;
use crate::types::{BslType, EnumRegistry};
use babylon_graph::stable_element::{StableElementKeyV1, StableElementResolverV1};
use babylon_graph::substrate::GraphSubstrate;
use babylon_kernel::{Coefficient, Currency, Ratio};
use std::collections::HashMap;

/// A runtime BSL value. The static type system (§3.1) is finer than this —
/// `Probability`/`Intensity`/`Coefficient`/`Real` are all the binary64 lane
/// at runtime; their `[0,1]` domains are load-time knowledge and the store
/// boundary's range check (`E-EVAL-020`, Task 16), per §3.3's
/// promote-to-`Real` ruling.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Exact, non-storable finite-kernel allocation input.
    Mass(Mass),
    /// `Int` — `i64`, checked arithmetic (§4.3).
    Int(i64),
    /// `Currency` — i128 micro-units, the §3.2 operator table.
    Currency(Currency),
    /// The binary64 lane (§3.3): `Probability` / `Intensity` /
    /// `Coefficient` / `Real`, always finite (`E-EVAL-014` otherwise).
    Real(f64),
    /// `Ratio` — §3.2 addendum (Director ruling 2026-08-11, #492/ADR194): a
    /// declared-domain positive scalar, `𝔾 ∩ (0, ∞)`, kept as its OWN
    /// variant rather than folded into `Real` precisely because the `[0,1]`
    /// cap must NOT silently widen for `Probability`/`Intensity`/
    /// `Coefficient` — those stay `Value::Real` exactly as before. The
    /// single legal use is the new `Currency × Ratio` operator
    /// (`apply_arith`'s `*` arm); `Ratio` has no other operator. `floor`
    /// and `cap` are the declared bounds this value's origin narrowed the
    /// sort to (a `defconst`'s `:floor`/`:cap`, `scenario.rs`) — `None` for
    /// a bare literal or an undeclared bound, matching `Ratio`'s own
    /// `(0, ∞)` domain exactly on that end. `floor` is EXCLUSIVE (a value
    /// must be strictly greater than it — matching `Ratio`'s own open-at-
    /// zero law and a `>`-bounded consumer like `entropy_factor`'s
    /// `(1.0, 3.0]`); `cap` is INCLUSIVE (matching a `<=`-bounded consumer
    /// like `pareto_alpha`'s `(0, 10]`). Re-checked at THIS multiply
    /// (`E-EVAL-041`), not just at the `defconst`'s own load
    /// (`E-LOAD-052`) — defense in depth, per III.11: the operation does
    /// not trust that nothing between declaration and use could hand it a
    /// stale or foreign value.
    Ratio {
        /// The scale factor itself — always finite and `> 0` by
        /// construction (the kernel sort's own invariant).
        value: Ratio,
        /// The declared EXCLUSIVE floor this value's `Ratio` sort was
        /// narrowed to, if any.
        floor: Option<Ratio>,
        /// The declared INCLUSIVE ceiling this value's `Ratio` sort was
        /// narrowed to, if any.
        cap: Option<Ratio>,
    },
    /// `Bool`.
    Bool(bool),
    /// A member of a closed enum — comparable with `=`/`!=` only, and only
    /// to the same enum type (§3.1).
    Enum {
        /// The closed enum's type name.
        enum_type: String,
        /// The member identifier.
        member: String,
    },
    /// `NodeRef` (§3.1) — produced by `self`, `add-node`, and node-query
    /// elements. No arithmetic, no ordering; refs are identities.
    NodeRef(babylon_graph::substrate::NodeId),
    /// `HyperedgeRef` (§3.1) — produced by `add-hyperedge` and hyperedge-
    /// query elements. Does NOT carry its `HyperedgeType` statically, which
    /// is why §2.6's hyperedge queries take the type as an operand.
    HyperedgeRef(babylon_graph::substrate::HyperedgeId),
    /// `EdgeRef` (§3.1) — produced by `edges`/`edge-between` query elements.
    /// No arithmetic, no ordering; refs are identities, same discipline as
    /// `NodeRef`/`HyperedgeRef`.
    EdgeRef(EdgeKey),
}

/// The spec's evaluation error codes (§4.6 / §3.2), one variant per code
/// this module can raise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalCode {
    /// `E-EVAL-010` — a `Currency` result below zero (domain `[0, ∞)`).
    CurrencyNegative,
    /// `E-EVAL-011` — `Int` or `Currency` overflow.
    Overflow,
    /// `E-EVAL-012` — division by zero (binary64 lane, `Currency ÷
    /// Currency`, or `Currency ÷ integer` with a zero-or-negative divisor).
    DivisionByZero,
    /// `E-EVAL-013` — a `Currency ÷ Currency` ratio outside `[0, 1]`.
    CoefficientOutOfRange,
    /// `E-EVAL-041` — a `Currency × Ratio` operand falls outside the
    /// declared domain its origin narrowed it to — at or below an
    /// EXCLUSIVE `:floor`, or above an INCLUSIVE `:cap` (§3.2 addendum,
    /// Director ruling 2026-08-11, #492/ADR194). Distinct from
    /// `E-LOAD-052`, which is the SAME domain rule checked at the
    /// `defconst` declaration itself — this is the operation's own
    /// re-check at the point of use.
    RatioOutsideDeclaredDomain,
    /// `E-EVAL-014` — a binary64 operation producing a non-finite result.
    NonFinite,
    /// `E-EVAL-020` — a store whose resulting value falls outside the
    /// target field's declared range (§3.3: the range check happens once,
    /// at the store boundary — a loud failure, never a clamp).
    StoreRangeViolation,
    /// `E-EVAL-021` — `mean`/`min`/`max` over an empty set (§4.4), and —
    /// the same code for the same reason (D45) — a `select-max`/
    /// `select-min` over an empty query: there is no element to return and
    /// there is no null.
    EmptyAggregate,
    /// `E-EVAL-031` — the §2.8 existence discipline: removing what does
    /// not exist, adding what exists, an unknown or duplicated hyperedge
    /// member. Absence is never treated as success.
    ExistenceDiscipline,
    /// `E-EVAL-032` — a `members-of`/`hyperedges-of` whose referent is not
    /// of the annotated `HyperedgeType` (D24) — never a silently empty set.
    HyperedgeTypeMismatch,
    /// `E-EVAL-033` — an accessor (§2.10) whose referent is not of the
    /// qname's owning type, or which carries no value for the named
    /// declared field. **Never** a default value and never an absent read
    /// (D34); also the runtime half of `E-TYPE-014` on the update verbs,
    /// whose element is a reference §3.1 gives no static type.
    AccessorTypeOrValueMismatch,
    /// `E-EVAL-034` — an `edge-between` that resolves to no edge (§2.10).
    /// The accessor never yields an absent reference and never degrades to
    /// a no-op write.
    NoSuchEdge,
    /// `E-EVAL-035` — `the` against a graph that hydrated no node of the
    /// `:ceiling 1` carrier type (§2.10): a carrier the scenario forgot to
    /// hydrate fails loudly rather than reading as zero.
    UnhydratedCarrier,
    /// `E-EVAL-036` — a `metric-of` whose referent is not of the metric's
    /// declared domain type (§2.11).
    MetricDomainMismatch,
    /// `E-EVAL-037` — a metric the provider produced no value for (§2.11).
    /// Absence is never a zero.
    MetricValueAbsent,
    /// `E-EVAL-040` — the fuel meter reached or passed zero.
    FuelExhausted,
    /// `E-EVAL-039` — the `floor` intrinsic's argument is outside its
    /// ratified domain (ADR188 Row 2, §3.10 / D97): negative, non-finite,
    /// or a result that does not fit `Int`'s `i64` domain (§3.1). Never a
    /// silent wraparound, and never a silently-chosen rounding convention
    /// for the disputed (negative) domain floor/trunc diverge on — a loud
    /// failure instead (III.11).
    DemotionOutOfDomain,
    /// `E-EVAL-042` (§2.13, D101) — a structural-verb write (`update-node`
    /// and siblings) to an `:enum-type`-declared field, evaluating to
    /// anything other than a matching `<enum-ref>` of that exact declared
    /// type. The load-time law (`E-LOAD-056`, `scenario.rs::
    /// attribute_value_enum`) re-checked at the ONE boundary content
    /// cannot be checked once and for all at load — the same two-site
    /// pattern the store boundary (`E-EVAL-020`) and range checks
    /// (`E-EVAL-041`) already use.
    EnumWriteShapeViolation,
    /// `E-EVAL-043` (§3.10, R10/ADR176 r21, ADR188 cap) — an `exp`/`log`
    /// argument outside the pinned crossing's accepted domain: a non-finite
    /// argument to either intrinsic (`NaN`, `±inf` are not real numbers, so
    /// never a legal `e^x`/`ln(x)` input), or a non-positive argument to
    /// `log` (`x <= 0.0`, `-0.0` included — `-0.0 <= 0.0` is `true`, the
    /// mirror of `floor`'s negative-zero *acceptance*: there `-0.0 < 0.0` is
    /// `false`). This is the ARGUMENT-side check, before
    /// `babylon_kernel::transcendental::{exp, ln}` ever runs — distinct from
    /// [`Self::NonFinite`] (`E-EVAL-014`), which is the RESULT-side check
    /// (e.g. `exp` overflowing to `+inf`).
    TranscendentalOutOfDomain,
    /// `E-EVAL-044` (§3.10, ADR219 — the exact-arithmetic rider train,
    /// Director ruling 2026-08-22) — an argument outside one of the six
    /// exact-arithmetic intrinsics' ratified domain: a negative argument to
    /// `sqrt` (domain `[0, ∞)`, `-0.0` included per IEEE — `-0.0 < 0.0` is
    /// `false`, the `floor` precedent), `clamp`'s `lo > hi` (never a silent
    /// swap of the bounds — §3.3's silent-clamping prohibition), or a
    /// non-finite argument to any of `sqrt`/`round-half-even`/`min`/`max`/
    /// `abs`/`clamp` (`NaN`/`±inf` are not real numbers, and §4.3 makes
    /// them unrepresentable at any observable point, so this half is
    /// defense in depth). No result-side check exists: each of the six maps
    /// finite inputs to a finite, IEEE-754 exactly-specified result, so a
    /// result guard would be dead code — the disposition is recorded in
    /// §3.10's normative paragraphs.
    IntrinsicOutOfDomain,
}

impl EvalCode {
    /// The spec's error code string.
    #[must_use]
    pub fn spec_code(self) -> &'static str {
        match self {
            Self::CurrencyNegative => "E-EVAL-010",
            Self::Overflow => "E-EVAL-011",
            Self::DivisionByZero => "E-EVAL-012",
            Self::CoefficientOutOfRange => "E-EVAL-013",
            Self::RatioOutsideDeclaredDomain => "E-EVAL-041",
            Self::NonFinite => "E-EVAL-014",
            Self::StoreRangeViolation => "E-EVAL-020",
            Self::EmptyAggregate => "E-EVAL-021",
            Self::ExistenceDiscipline => "E-EVAL-031",
            Self::HyperedgeTypeMismatch => "E-EVAL-032",
            Self::AccessorTypeOrValueMismatch => "E-EVAL-033",
            Self::NoSuchEdge => "E-EVAL-034",
            Self::UnhydratedCarrier => "E-EVAL-035",
            Self::MetricDomainMismatch => "E-EVAL-036",
            Self::MetricValueAbsent => "E-EVAL-037",
            Self::FuelExhausted => "E-EVAL-040",
            Self::DemotionOutOfDomain => "E-EVAL-039",
            Self::EnumWriteShapeViolation => "E-EVAL-042",
            Self::TranscendentalOutOfDomain => "E-EVAL-043",
            Self::IntrinsicOutOfDomain => "E-EVAL-044",
        }
    }
}

/// A loud evaluation failure (§4.6: aborts the tick, never converted into
/// a default value, a skipped effect, or a log line). `code` is `None`
/// where the reference names no `E-EVAL` code for the condition — no
/// invented codes (the Task 10 precedent).
#[derive(Debug, Clone, PartialEq)]
pub struct EvalError {
    /// The spec's code, where one is named.
    pub code: Option<EvalCode>,
    /// What failed, precisely.
    pub message: String,
}

impl EvalError {
    /// A coded error.
    #[must_use]
    pub fn coded(code: EvalCode, message: impl Into<String>) -> Self {
        Self {
            code: Some(code),
            message: message.into(),
        }
    }

    /// A loud error the reference names no code for.
    #[must_use]
    pub fn plain(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(code) => write!(f, "{}: {}", code.spec_code(), self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for EvalError {}

/// The evaluation environment (§4.2): resolved binding values, the declared
/// intrinsic costs, the graph a query-evaluating form reads, and the §2.6
/// chapter C8 element stack `it`/`:as` resolve through.
pub struct EvalEnv<'a> {
    /// Resolved rule bindings, name → value (binding resolution itself is
    /// the loader's job, §3.5 — unbound here means the loader failed).
    /// `self` lives here, bound once per subject (`tick.rs::bind_subject`)
    /// — distinct from `it`, which is never a binding (see `elements`).
    pub bindings: HashMap<String, Value>,
    /// Declared `:cost` per intrinsic (§2.7), for the §4.5 charge.
    pub intrinsic_costs: &'a IntrinsicCosts,
    /// The graph a query-evaluating form reads. `None` for the
    /// pure-expression callers (`:expr` binding resolution, the arithmetic
    /// conformance vectors) — a query head reached with no graph is a LOUD
    /// driver error (`require_graph`), never an empty set.
    pub graph: Option<&'a dyn GraphSubstrate>,
    /// Declared field types/kinds (§3.4/§3.1). Threaded so `field-of`
    /// (`field_of_node`, D102) can render an `:enum-type`-declared field's
    /// stored ordinal to `Value::Enum` through the SAME rendering
    /// `tick.rs::bind_field_value` uses for a `:field` binding's read path
    /// (§2.5) — the whole point of discharging D102 is that both read
    /// routes agree. `None` for the pure-expression / graph-free callers
    /// `graph`'s own doc above already names: `field-of` requires a graph
    /// before it would ever consult this, so those callers lose nothing.
    pub types: Option<&'a TypeEnv>,
    /// The enum member registry a `types` field's `BslType::Enum(id)`
    /// resolves against (an `EnumTypeId` alone names no members — see
    /// `types::EnumRegistry`'s own doc). Same threading rule as `types`.
    pub enums: Option<&'a EnumRegistry>,
    /// The §2.6 chapter C8 element stack, innermost-last. `it` always reads
    /// the last entry's element; a `:as` name reads by name (wired in a
    /// later task) — the paired `Option<String>` is that declared name,
    /// `None` for an iterating form with no `:as`.
    pub elements: Vec<(Option<String>, Element)>,
    /// Engine-owned deterministic draw context for the current subject, or
    /// `None` for a graph-free expression caller. Amendment AJ makes finite
    /// kernel drawing private: author-declarable intrinsics are context-free,
    /// while the typed kernel realization path consumes this context directly.
    pub draw_context: Option<&'a DrawContext<'a>>,
}

/// Evaluate `expr`, decrementing `*fuel` per §4.5. Fuel exhaustion should
/// be unreachable when Task 13's static bound accepted the rule with a
/// strictly larger budget — reaching it anyway is diagnostic of a
/// static-bound soundness bug, not merely "the rule was expensive".
///
/// # Errors
///
/// The coded §4.3/§3.2 arithmetic failures, `E-EVAL-040` on fuel
/// exhaustion, and loud uncoded errors for shapes outside the §2 grammar
/// or outside this task's expression core.
pub fn evaluate(
    expr: &SExpr,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    match expr {
        SExpr::Atom(atom) => {
            charge(fuel, atom_charge(atom))?;
            atom_value(atom, env)
        }
        SExpr::List(items) => eval_form(items, env, host, fuel),
    }
}

/// §4.5: subtract `amount`, erroring when the meter would reach or pass
/// zero — it stays strictly positive. `pub(crate)` so the effect executor
/// (`structural_verbs`, Task 16) charges through the SAME meter — one §4.5
/// accounting, not two.
pub(crate) fn charge(fuel: &mut u64, amount: u64) -> Result<(), EvalError> {
    if amount >= *fuel {
        return Err(EvalError::coded(
            EvalCode::FuelExhausted,
            "fuel meter reached zero — if the static bound accepted this rule, \
             the bound is unsound: escalate, do not raise :fuel blindly",
        ));
    }
    *fuel -= amount;
    Ok(())
}

/// The §3.7 base charge for an atom: literals, enum-refs and field paths
/// cost 0; a symbol is a variable reference and costs 1.
fn atom_charge(atom: &Atom) -> u64 {
    match atom {
        Atom::Symbol(_) => cost::VARIABLE_REF,
        _ => cost::LITERAL,
    }
}

fn atom_value(atom: &Atom, env: &EvalEnv<'_>) -> Result<Value, EvalError> {
    match atom {
        Atom::Mass(mass) => Ok(Value::Mass(*mass)),
        Atom::Int(n) => Ok(Value::Int(*n)),
        Atom::Currency(c) => Ok(Value::Currency(*c)),
        Atom::Scaled(s) if s.kind == ScaledKind::Ratio => {
            // §1.5 addendum: scale ≤ 9, so the unscaled integer is < 10⁹ —
            // exact in f64 by construction, same reasoning as the p/i/c arm
            // below. A bare literal carries no declared bound of its own
            // (`floor`/`cap: None`, i.e. the full `(0, ∞)` domain) — a bound
            // is something only a `defconst`'s `:floor`/`:cap` narrows
            // (`scenario.rs`).
            #[allow(clippy::cast_precision_loss)]
            let raw = s.unscaled as f64 / 10f64.powi(i32::from(s.scale));
            let value = Ratio::new(raw).map_err(|e| {
                EvalError::plain(format!(
                    "r literal {raw} failed Ratio construction at eval \
                     ({e:?}) — the reader's E-LEX-027 should have refused \
                     this at lex time; reaching here is a reader/kernel \
                     sort disagreement, not a content error"
                ))
            })?;
            Ok(Value::Ratio {
                value,
                floor: None,
                cap: None,
            })
        }
        Atom::Scaled(s) => {
            // A p/i/c literal is unit-interval with scale ≤ 9, so the
            // unscaled integer is < 10⁹ — exact in f64 by construction.
            #[allow(clippy::cast_precision_loss)]
            let value = s.unscaled as f64 / 10f64.powi(i32::from(s.scale));
            Ok(Value::Real(value))
        }
        Atom::Bool(b) => Ok(Value::Bool(*b)),
        Atom::EnumRef { enum_type, member } => Ok(Value::Enum {
            enum_type: enum_type.clone(),
            member: member.clone(),
        }),
        // §2.6 chapter C8: `it` denotes the element of the innermost
        // enclosing iterating form — resolved through the element stack,
        // NEVER through `env.bindings` (it is not a binding, and never has
        // been one). `scope.rs::walk_names` already refuses a bare `it`
        // outside a body at LOAD time (E-TYPE-012); reaching an empty
        // element stack here is defense in depth, exactly like the unbound-
        // variable arm below.
        Atom::Symbol(name) if name == "it" => env
            .elements
            .last()
            .map(|(_, element)| element.to_value())
            .ok_or_else(|| {
                EvalError::plain(
                    "it — §2.6 chapter C8: it denotes the element of the \
                     innermost enclosing iterating form, and the element \
                     stack is empty here, so there is no such form. This \
                     should already be refused at load time \
                     (scope.rs::walk_names, E-TYPE-012); reaching it here is \
                     defense in depth, never a stale or default read"
                        .to_owned(),
                )
            }),
        // §2.6 chapter C8 (D54): a `:as` name is in scope for the whole
        // body of its form, INCLUDING NESTED BODIES — so the whole element
        // stack is searched, not just the innermost entry (that is `it`'s
        // own rule, above). `:as` names are rule-scoped-unique
        // (`scope.rs::check_element_names`, E-PARSE-030 at load), so at
        // most one stack entry can ever match; `rev()` costs nothing and
        // keeps the search innermost-first, symmetric with `it`.
        Atom::Symbol(name) => env
            .elements
            .iter()
            .rev()
            .find(|(declared, _)| declared.as_deref() == Some(name.as_str()))
            .map(|(_, element)| element.to_value())
            .or_else(|| env.bindings.get(name).cloned())
            .ok_or_else(|| {
                EvalError::plain(format!(
                    "unbound variable: {name} — binding resolution is a \
                     load-time gate (E-LOAD-010, §3.5); reaching this at \
                     evaluation is a loader bug"
                ))
            }),
        other => Err(EvalError::plain(format!(
            "atom is not a value in expression position: {other:?}"
        ))),
    }
}

/// The graph a query-evaluating form needs (§4.2). `EvalEnv.graph` is
/// `None` only for the pure-expression callers; a query head reached with
/// no graph is a LOUD driver error — the caller built an environment with
/// no graph for a form that needs one, which is not the same fact as "the
/// query found nothing" and must never read as an empty set or a `0`. Every
/// query head Task 4 onward dispatches charges through this one seam.
///
/// # Errors
///
/// A loud, uncoded [`EvalError`] naming `form` and the driver-error reading
/// when `env.graph` is `None`.
// Task 2 landed this seam with no caller yet (hence its own `#[allow(dead_code)]`
// at the time); Task 4's `query::materialize` is the first production caller
// (`materialize_nodes`/`materialize_neighbors`), so the exemption is dropped.
pub(crate) fn require_graph<'a>(
    env: &EvalEnv<'a>,
    form: &str,
) -> Result<&'a dyn GraphSubstrate, EvalError> {
    env.graph.ok_or_else(|| {
        EvalError::plain(format!(
            "({form} …) needs the graph substrate (§4.2) but this EvalEnv \
             carries none — a driver error: the caller built an environment \
             with no graph for a query-evaluating form, never an empty set \
             and never a 0"
        ))
    })
}

/// Heads whose grammatical home is EFFECT position (§2.8) or an update-op/
/// grouping form. Meeting one in EXPRESSION position (this module) is a
/// grammar error, not an unimplemented seam — §2.7's `<expr>` production has
/// no production for any of them. (Whether the head is SERVED in effect
/// position varies: most dispatch in [`crate::structural_verbs`]; `for-each`
/// and `update-membership` refuses there too, pending the AG(i) lane (#653)
/// — `for-each`, `update-edge` and (since Community port train Task 6)
/// `update-hyperedge` all serve there today; the message below claims only the
/// grammar, never service.) §2.8's `<verb>` production has ELEVEN
/// alternatives: the ten structural verbs (`update-node`, `update-edge`,
/// `update-hyperedge`, `update-membership`, `add-node`, `remove-node`,
/// `add-edge`, `remove-edge`, `add-hyperedge`, `remove-hyperedge`) plus
/// `emit`. Those eleven, plus the probability kernel's `choose`, `guard` and
/// `for-each`, the four update-ops
/// (`add`/`sub`/`set`/`scale`), and the two list forms (`members`,
/// `member`) — 20 in all. (`update-membership`/`member` are Amendment-AG-era
/// heads absent from `RESERVED_FORM_TAGS`; that reservation gap is a filed
/// follow-up, not this table's concern.)
const EFFECT_POSITION_ONLY: [&str; 20] = [
    "choose",
    "guard",
    "for-each",
    "update-node",
    "update-edge",
    "update-hyperedge",
    "add-node",
    "remove-node",
    "add-edge",
    "remove-edge",
    "add-hyperedge",
    "remove-hyperedge",
    "update-membership",
    "emit",
    "add",
    "sub",
    "set",
    "scale",
    "members",
    "member",
];

/// Expression heads the query evaluator does not yet serve, each mapped to
/// the slice of `docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md`
/// that will: the node-set shapes of the polymorphic heads land as Tasks 5-8
/// of THIS plan remove their rows one by one (`fold`/`exists`/`forall`/
/// `select-max`/`select-min`/`field-of`, each moved to [`EVALUATOR_SERVED`]
/// once its task lands — their string names the edge/hyperedge shapes that
/// still ride slices 2-3); `the` (slice 2, the carrier-read lane —
/// `edges`/`edge-between` served as of T2, issue #559);
/// `hyperedges`/`members-of`/`hyperedges-of`/`metric-of`
/// (slice 3, the hyperedge + metric lane); `membership-field-of` (the
/// AG(i) membership-payload lane, #653 — Director-ruled deferred to
/// first consumer). `hyperedges`/`members-of`/`hyperedges-of` LEFT this
/// table at slice 3's landing (Community port train Task 3, 2026-08-21) —
/// they are served, as query operands of iterating forms only, per the
/// `SERVED_QUERY_HEADS` law below. `nodes`/`neighbors` moved to [`SERVED_QUERY_HEADS`] at
/// Task 4 — they are served, but only as the query operand of an iterating
/// form, never as a bare `<expr>` (§2.7 has no query production of its
/// own). Together with [`EFFECT_POSITION_ONLY`] and [`SERVED_QUERY_HEADS`]
/// this is exhaustive over the pre-Task-1 `GRAPH_SEAM_HEADS` set AND the
/// grammar's §2.8/§2.10 heads: a head in none of the three tables is
/// `eval_intrinsic`'s.
const UNSERVED_EXPRESSION_HEADS: [(&str, &str); 3] = [
    ("the", "slice 2"),
    ("metric-of", "slice 3"),
    (
        "membership-field-of",
        "the AG(i) membership-payload lane — #653",
    ),
];

/// The §2.6 query heads Task 4 onward serves — but **only** as the query
/// operand of an iterating form (`fold`/`exists`/`forall`/`select-max`/
/// `select-min`/`for-each`), which extracts and materializes the query
/// directly (`query::materialize`) without ever calling [`evaluate`] on the
/// query form itself. §2.7's `<expr>` production has no query alternative
/// (unlike `<fold>`/`<accessor>`/`<selection>`, a bare `<query>` is not one
/// of `<expr>`'s productions), so reaching one of these heads HERE — through
/// generic expression dispatch — means it was written somewhere the grammar
/// does not admit a query: a shape error, not an unimplemented seam.
/// `nodes`/`neighbors`/`edges`/`hyperedges`/`members-of`/`hyperedges-of`
/// are here (`edges` joined at T2, issue #559; the three hyperedge heads at
/// slice 3's landing, Community port train Task 3, 2026-08-21). Serving a
/// query head never means giving it a bare `<expr>` reading.
const SERVED_QUERY_HEADS: [&str; 6] = [
    "nodes",
    "neighbors",
    "edges",
    "hyperedges",
    "members-of",
    "hyperedges-of",
];

fn eval_form(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    if let Some(SExpr::Atom(Atom::Operator(op))) = items.first() {
        return eval_operator(op, &items[1..], env, host, fuel);
    }
    let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
        return Err(EvalError::plain(format!(
            "a form must be headed by a symbol or operator in expression \
             position, found {:?}",
            items.first()
        )));
    };
    match head.as_str() {
        "and" | "or" => eval_and_or(head, &items[1..], env, host, fuel),
        "not" => {
            charge(fuel, cost::ARITH_CMP_BOOL_BASE)?;
            let [operand] = &items[1..] else {
                return Err(EvalError::plain("(not <cond>) takes exactly one operand"));
            };
            let value = as_bool(evaluate(operand, env, host, fuel)?)?;
            Ok(Value::Bool(!value))
        }
        "if" => eval_if(&items[1..], env, host, fuel),
        "fold" => eval_fold(items, env, host, fuel),
        "exists" | "forall" => eval_exists_forall(head, items, env, host, fuel),
        "select-max" | "select-min" => eval_selection(head, items, env, host, fuel),
        "field-of" => eval_field_of(items, env, host, fuel),
        "edge-between" => eval_edge_between(items, env, host, fuel),
        "quantize-mass" => eval_quantize_mass(items, env, host, fuel),
        name => {
            if EFFECT_POSITION_ONLY.contains(&name) {
                return Err(EvalError::plain(format!(
                    "({name} …) is an effect-position verb or grouping form \
                     (§2.8) — using it in expression position is a grammar \
                     error, not an unimplemented seam: §2.7's <expr> \
                     production has no ({name} …) form; its grammatical home \
                     is effect position (§2.8)"
                )));
            }
            if SERVED_QUERY_HEADS.contains(&name) {
                return Err(EvalError::plain(format!(
                    "({name} …) is a §2.6 query head with no <expr> \
                     production of its own (§2.7) — it is legal only as the \
                     query operand of fold/exists/forall/select-max/\
                     select-min/for-each, which materialize it directly; \
                     reaching it here means it was written somewhere the \
                     grammar does not admit a query, a shape error rather \
                     than an unimplemented seam"
                )));
            }
            if let Some((_, slice)) = UNSERVED_EXPRESSION_HEADS.iter().find(|(h, _)| *h == name) {
                return Err(EvalError::plain(format!(
                    "({name} …) is a query/selection/accessor form the \
                     evaluator does not yet serve (§2.6/§2.7/§2.10) — it \
                     lands with {slice}, never as a default here"
                )));
            }
            eval_intrinsic(name, &items[1..], env, host, fuel)
        }
    }
}

fn eval_quantize_mass(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, QUANTIZE_MASS_BASE)?;
    let [_, operand] = items else {
        return Err(EvalError::plain(
            "(quantize-mass <numeric-expr>) takes exactly one operand",
        ));
    };
    let evaluated = evaluate(operand, env, host, fuel)?;
    let numeric = match evaluated {
        Value::Real(value) => value,
        // `quantize-mass` is the explicit governed binary64 crossing; exact
        // nanounit rounding begins from the represented f64 value afterward.
        #[allow(clippy::cast_precision_loss)]
        Value::Int(value) => value as f64,
        other => {
            return Err(EvalError::plain(format!(
                "quantize-mass requires an Int or Real-lane operand, found {other:?}"
            )))
        }
    };
    Mass::quantize(numeric)
        .map(Value::Mass)
        .map_err(|error| EvalError::plain(error.to_string()))
}

fn eval_operator(
    op: &str,
    operands: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::ARITH_CMP_BOOL_BASE)?;
    let [lhs_expr, rhs_expr] = operands else {
        return Err(EvalError::plain(format!(
            "({op} …) is strictly binary — ({op} a b c) is E-PARSE-040 (§2.7)"
        )));
    };
    let lhs = evaluate(lhs_expr, env, host, fuel)?;
    let rhs = evaluate(rhs_expr, env, host, fuel)?;
    match op {
        "+" | "-" | "*" | "/" => apply_arith(op, &lhs, &rhs),
        "<" | "<=" | ">" | ">=" => apply_ordering(op, &lhs, &rhs),
        "=" | "!=" => apply_equality(op, &lhs, &rhs),
        other => Err(EvalError::plain(format!("unknown operator: {other}"))),
    }
}

fn eval_and_or(
    head: &str,
    operands: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::ARITH_CMP_BOOL_BASE)?;
    if operands.is_empty() {
        return Err(EvalError::plain(format!(
            "({head}) with no operands is E-PARSE-021 — there is no implicit \
             identity element (§2.4)"
        )));
    }
    let stop_on = head == "or"; // and stops on #f, or stops on #t (§4.1)
    for operand in operands {
        let value = as_bool(evaluate(operand, env, host, fuel)?)?;
        if value == stop_on {
            return Ok(Value::Bool(stop_on)); // deliberate short-circuit
        }
    }
    Ok(Value::Bool(!stop_on))
}

fn eval_if(
    operands: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::IF_BASE)?;
    let [cond, then_branch, else_branch] = operands else {
        return Err(EvalError::plain(
            "(if <cond> <expr> <expr>) takes exactly three operands",
        ));
    };
    let taken = if as_bool(evaluate(cond, env, host, fuel)?)? {
        then_branch
    } else {
        else_branch
    };
    // §4.1: only the taken branch is evaluated (and therefore charged).
    evaluate(taken, env, host, fuel)
}

/// Build a child environment for one iteration of an iterating form: the
/// bindings/intrinsic-costs/graph carry over unchanged, and the §2.6
/// chapter C8 element stack gains ONE entry (innermost-last) for `it` and,
/// once named, a `:as` reference. `bindings`/`elements` are cloned rather
/// than shared because [`EvalEnv`] owns them by value — the cost is one
/// small `HashMap`/`Vec` clone per element, not per AST node, and is not on
/// any hot path this crate has (fold ceilings are declared, bounded
/// quantities, not an unbounded stream).
///
/// `pub(crate)` (Task 10, P27 Phase 2 Slice 1): `structural_verbs::for_each`
/// pushes an element for `for-each`'s body exactly the way every EXPRESSION-
/// position iterating form here does — one element stack, one rule for `it`
/// and `:as`, whether the body is an `<expr>` or an `<effect-item>+`.
pub(crate) fn with_element<'a>(
    env: &EvalEnv<'a>,
    name: Option<String>,
    element: Element,
) -> EvalEnv<'a> {
    let mut elements = env.elements.clone();
    elements.push((name, element));
    EvalEnv {
        bindings: env.bindings.clone(),
        intrinsic_costs: env.intrinsic_costs,
        graph: env.graph,
        types: env.types,
        enums: env.enums,
        elements,
        // Carried over unchanged: an iterating form's child environment is
        // still evaluated for the SAME subject, at the SAME tick — only
        // the element stack grows.
        draw_context: env.draw_context,
    }
}

/// Strip an optional leading `:as <symbol>` pair from an iterating form's
/// operand tail (after its query), mirroring `bound_checker::strip_elem_name`
/// but returning the extracted name too — the evaluator needs it to push
/// onto the element stack, where the bound checker only needs to zero its
/// cost.
///
/// `pub(crate)` (Task 10): `for-each`'s `<elem-name>?` is stripped the same
/// way `exists`/`forall`/`select-*`'s is — one parser for the shared
/// `<query> <elem-name>? …` shape, not a second one in `structural_verbs`.
pub(crate) fn strip_as_name(items: &[SExpr]) -> (Option<String>, &[SExpr]) {
    if let [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Symbol(name)), rest @ ..] = items {
        if kw == "as" {
            return (Some(name.clone()), rest);
        }
    }
    (None, items)
}

/// A syntactic, GRAPH-FREE guess at a fold body's additive identity, for
/// `sum` over an empty query (§4.4: "the additive identity of the body
/// type") — a case with no element to evaluate and therefore no dynamic
/// value to inspect. `EvalEnv` carries no static field-type registry (that
/// lives in `structural_verbs::TypeEnv`, used only for the store-boundary
/// range check), so this recognizes the shapes slice 1's actual bodies take
/// — a bare numeric literal, a `field-of` read (assumed `Real` here — true
/// for an ordinary field, false for an `:enum-type`-declared one which
/// renders `Value::Enum` instead, D102 — but unreachable for the latter:
/// `E-TYPE-044` (#551) refuses any `(fold sum …)` whose body resolves to an
/// enum-declared field at load, before this function ever runs), and
/// homogeneous arithmetic over them — and returns `None` for anything else,
/// which `fold_sum` turns into a loud, named refusal rather than a guess.
fn static_additive_identity(body: &SExpr) -> Option<Value> {
    match body {
        SExpr::Atom(Atom::Int(_)) => Some(Value::Int(0)),
        SExpr::Atom(Atom::Currency(_)) => Some(Value::Currency(Currency::from_micro_units(0))),
        SExpr::Atom(Atom::Scaled(s)) if s.kind != ScaledKind::Ratio => Some(Value::Real(0.0)),
        SExpr::List(items) => match items.as_slice() {
            [SExpr::Atom(Atom::Symbol(head)), ..] if head == "field-of" => Some(Value::Real(0.0)),
            [SExpr::Atom(Atom::Operator(op)), lhs, rhs]
                if matches!(op.as_str(), "+" | "-" | "*") =>
            {
                match (static_additive_identity(lhs), static_additive_identity(rhs)) {
                    // P7: `Currency × Currency` is illegal at runtime
                    // (`arith_currency`'s `*` arm: "an area of money",
                    // E-TYPE-030) even though its additive identity would
                    // trivially match by discriminant below. Unreachable via
                    // the loader today — tightened here, defense in depth,
                    // rather than left to accidentally serve a value the
                    // runtime itself refuses to produce.
                    (Some(Value::Currency(_)), Some(Value::Currency(_))) if op == "*" => None,
                    (Some(a), Some(b))
                        if std::mem::discriminant(&a) == std::mem::discriminant(&b) =>
                    {
                        Some(a)
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        SExpr::Atom(_) => None,
    }
}

/// Evaluate `body` (and, if present, `:weight`) against `element` pushed
/// onto the element stack — the one per-element seam every fold-op below
/// shares, so fuel-fidelity (§3.7's `ceiling × (cost(body) + cost(weight))`
/// row charges weight regardless of op) is one code path, not five.
fn eval_body_and_weight(
    element: Element,
    elem_name: Option<&str>,
    body: &SExpr,
    weight: Option<&SExpr>,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<(Value, Option<Value>), EvalError> {
    let child = with_element(env, elem_name.map(str::to_owned), element);
    let body_val = evaluate(body, &child, host, fuel)?;
    let weight_val = match weight {
        Some(w) => Some(evaluate(w, &child, host, fuel)?),
        None => None,
    };
    Ok((body_val, weight_val))
}

/// `(fold <fold-op> <query> <elem-name>? <expr> (:weight <expr>)?)` (§2.7).
fn eval_fold(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::FOLD_BASE)?;
    let [_, op_atom, query, rest @ ..] = items else {
        return Err(EvalError::plain(
            "(fold <fold-op> <query> <elem-name>? <expr> (:weight <expr>)?) \
             — too few operands",
        ));
    };
    let SExpr::Atom(Atom::Symbol(op)) = op_atom else {
        return Err(EvalError::plain(format!(
            "fold-op must be a symbol, found {op_atom:?}"
        )));
    };
    let (elem_name, rest) = strip_as_name(rest);
    let (body, weight) = match rest {
        [body] => (body, None),
        [body, SExpr::Atom(Atom::Keyword(kw)), weight] if kw == "weight" => (body, Some(weight)),
        _ => {
            return Err(EvalError::plain(
                "(fold …) — the shape after the query must be <expr> or \
                 <expr> :weight <expr>",
            ))
        }
    };
    // M1: §2.7's <fold> grammar admits `( ":weight" <expr> )?` on every
    // fold-op, but §3.4's per-operator table gives `:weight` a reading for
    // `mean` ALONE — `sum`/`min`/`max`/`count` have no weighted semantics to
    // apply it to. Silently evaluating and discarding it (the pre-fix
    // behaviour of `fold_sum`/`fold_min_max`/`fold_count`, each of which
    // destructured `(body_val, _weight_val)`) is exactly the silent-
    // degradation footgun §3.4's kind law exists to close, so this refuses
    // loudly, by name, before the query is even materialized — D-row **Q11**
    // (docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md).
    if weight.is_some() && op.as_str() != "mean" {
        return Err(EvalError::plain(format!(
            "(fold {op} … :weight …) — :weight is legal grammatically on \
             every fold-op (§2.7), but §3.4's per-operator table gives it a \
             reading for mean alone; {op} has no weighted semantics to apply \
             it to, and discarding it silently would be a variance-error \
             footgun. Refused by name — D-row Q11."
        )));
    }
    let elements = crate::query::materialize(query, env, host, fuel)?;
    // CT4P A3 (issue #525): `op` converts to `FoldOp` ONCE, here, and the
    // dispatch below matches it EXHAUSTIVELY — no wildcard. The
    // unrecognized-operator message is preserved byte-for-byte.
    let Some(fold_op) = crate::grammar::FoldOp::parse(op.as_str()) else {
        return Err(EvalError::plain(format!(
            "unknown fold-op '{op}' — the closed set is sum|mean|min|max|count \
             (§2.7; E-PARSE-015 at load)"
        )));
    };
    match fold_op {
        // P4: count is CARDINALITY (§3.4 row 6) — no body/weight/env/host/
        // fuel operand needed; see fold_count's own doc for why.
        crate::grammar::FoldOp::Count => fold_count(&elements),
        crate::grammar::FoldOp::Sum => fold_sum(
            &elements,
            elem_name.as_deref(),
            body,
            weight,
            env,
            host,
            fuel,
        ),
        crate::grammar::FoldOp::Mean => fold_mean(
            &elements,
            elem_name.as_deref(),
            body,
            weight,
            env,
            host,
            fuel,
        ),
        crate::grammar::FoldOp::Min => fold_min_max(
            &elements,
            elem_name.as_deref(),
            body,
            weight,
            env,
            host,
            fuel,
            true,
        ),
        crate::grammar::FoldOp::Max => fold_min_max(
            &elements,
            elem_name.as_deref(),
            body,
            weight,
            env,
            host,
            fuel,
            false,
        ),
    }
}

/// `(exists <query> <elem-name>? <cond>?)` / `(forall <query> <elem-name>?
/// <cond>)` (§2.4/§2.7). §4.4: `exists` over an empty set is `#f`; `forall`
/// over an empty set is `#t`. §4.1: both short-circuit — `exists` stops at
/// the first element whose predicate is true, `forall` at the first false —
/// which is what makes a `:fuel-used` figure strictly smaller when the
/// deciding element is early rather than late.
fn eval_exists_forall(
    head: &str,
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::EXISTS_FORALL_BASE)?;
    let [_, query, rest @ ..] = items else {
        return Err(EvalError::plain(format!(
            "({head} <query> <elem-name>? <cond>?) — missing query"
        )));
    };
    let (elem_name, rest) = strip_as_name(rest);
    let cond = match rest {
        [] => None,
        [c] => Some(c),
        _ => {
            return Err(EvalError::plain(format!(
                "({head} …) — unrecognized shape after the query"
            )))
        }
    };
    let elements = crate::query::materialize(query, env, host, fuel)?;
    let is_exists = head == "exists";
    let Some(cond) = cond else {
        if is_exists {
            // "(exists <query>)" with no body: the query is non-empty (§2.4).
            return Ok(Value::Bool(!elements.is_empty()));
        }
        // P5 (defense in depth): §2.4's grammar makes forall's <cond>
        // MANDATORY (`grammar.rs`'s ARITIES: forall takes exactly 2
        // operands) — `check_arities_and_closed_sets` should already refuse
        // a no-cond forall at LOAD time (E-PARSE-042), so reaching this
        // branch means that gate did not run. Falling through to exists'
        // "query is non-empty" reading would give forall over an EMPTY
        // query #f, contradicting §4.4's own forall-empty-is-#t pin — a
        // defense-in-depth failure, not merely a missed convenience, so
        // this refuses loudly instead of computing the wrong Boolean.
        return Err(EvalError::plain(
            "forall with no <cond> reached evaluation — §2.4's grammar makes \
             forall's <cond> MANDATORY (unlike exists', which is optional); \
             this should already be an arity error at load \
             (grammar.rs/E-PARSE-042), and reaching it here is defense in \
             depth, never a silent fallback to exists' non-emptiness \
             reading",
        ));
    };
    for element in elements.iter().cloned() {
        let child = with_element(env, elem_name.clone(), element);
        let value = as_bool(evaluate(cond, &child, host, fuel)?)?;
        if value == is_exists {
            // exists short-circuits on TRUE; forall short-circuits on FALSE.
            return Ok(Value::Bool(is_exists));
        }
    }
    Ok(Value::Bool(!is_exists))
}

/// `(select-max <query> <elem-name>? <expr>)` / `(select-min …)` (§2.7
/// chapter C5). Returns the query's ELEMENT (not the extremised value, that
/// is `fold`'s job). D45: ties break to the FIRST element in ascending id
/// byte order, for both operators — a single forward pass over the
/// materialized `Vec`, replacing the incumbent only on STRICT improvement,
/// is what makes "first wins" fall out of §2.6's own order rather than
/// arriving as a bolt-on tiebreak rule. An empty query is `E-EVAL-021`
/// (§4.4/D45 — the same code and the same reason as `mean`/`min`/`max`).
fn eval_selection(
    head: &str,
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::SELECTION_BASE)?;
    let [_, query, rest @ ..] = items else {
        return Err(EvalError::plain(format!(
            "({head} <query> <elem-name>? <expr>) — missing query"
        )));
    };
    let (elem_name, rest) = strip_as_name(rest);
    let [score_expr] = rest else {
        return Err(EvalError::plain(format!(
            "({head} …) — expected exactly one score expression after the query"
        )));
    };
    let elements = crate::query::materialize(query, env, host, fuel)?;
    if elements.is_empty() {
        return Err(EvalError::coded(
            EvalCode::EmptyAggregate,
            format!(
                "{head} over an empty query (§4.4/D45) — there is no element \
                 to return and there is no null"
            ),
        ));
    }
    let want_max = head == "select-max";
    let op = if want_max { ">" } else { "<" };
    let mut best_element = elements[0].clone();
    let mut best_score: Option<Value> = None;
    for element in elements.iter().cloned() {
        let child = with_element(env, elem_name.clone(), element.clone());
        let score = evaluate(score_expr, &child, host, fuel)?;
        best_score = Some(match best_score {
            None => {
                best_element = element;
                score
            }
            Some(prev_best) => {
                let strictly_better =
                    matches!(apply_ordering(op, &score, &prev_best)?, Value::Bool(true));
                if strictly_better {
                    best_element = element;
                    score
                } else {
                    prev_best
                }
            }
        });
    }
    Ok(best_element.to_value())
}

/// P4: §3.4 row 6 makes `count`'s result the materialized set's
/// CARDINALITY, independent of the body's VALUE — unlike every other
/// fold-op, whose result depends on evaluating the body. The `<expr>` after
/// the query is still a real §2.7 production (`eval_fold`'s shape match
/// already parsed it and M1's weight guard already ran against it), but
/// count owes it no evaluation: doing so (the pre-fix behaviour, matched
/// fuel-fidelity reasoning that predates §3.4 row 6's reading) meant a body
/// reading a field one element never wrote aborted the WHOLE count with
/// `E-EVAL-033`, even though count owes that element's body no value at
/// all.
///
/// **Fuel.** §3.7's STATIC bound is `2 + cost(query) + ceiling(query) ×
/// (cost(body) + cost(weight))` for EVERY fold op — the formula does not
/// special-case `count`. Not charging the body/weight at RUNTIME here means
/// the meter now charges strictly LESS than the static bound predicted for
/// this op, which is the SAFE direction for `E-EVAL-040`: the runtime meter
/// is a backstop against the static bound being UNSOUND (charging too
/// little), never against it being merely conservative (charging too much,
/// which every other over-approximation in this crate already is). D-row
/// **Q13** records this disposition.
fn fold_count(elements: &[Element]) -> Result<Value, EvalError> {
    let n = i64::try_from(elements.len()).map_err(|_| {
        EvalError::plain(
            "fold count exceeds i64 — the declared ceiling should have bounded \
             this at load",
        )
    })?;
    Ok(Value::Int(n))
}

#[allow(clippy::too_many_arguments)]
fn fold_sum(
    elements: &[Element],
    elem_name: Option<&str>,
    body: &SExpr,
    weight: Option<&SExpr>,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    if elements.is_empty() {
        return static_additive_identity(body).ok_or_else(|| {
            EvalError::plain(format!(
                "(fold sum (empty query) {body:?}) — the additive identity of \
                 this body's type is not statically determinable from its \
                 syntax alone (§4.4); this evaluator recognizes literals, \
                 field-of reads and homogeneous arithmetic over them, and \
                 deliberately no more — a nested fold or a bare \
                 binding-symbol body are both load-legal §2.7 shapes this \
                 classifier does not attempt. Refused by name — D-row Q12 \
                 (empty-sum identity is servable only for classifiable \
                 bodies; extending the classifier speculatively is out of \
                 scope for this refusal)."
            ))
        });
    }
    let mut acc: Option<Value> = None;
    for element in elements.iter().cloned() {
        let (body_val, _weight_val) =
            eval_body_and_weight(element, elem_name, body, weight, env, host, fuel)?;
        acc = Some(match acc {
            None => body_val,
            Some(prev) => apply_arith("+", &prev, &body_val)?,
        });
    }
    Ok(acc.expect("non-empty elements guarantees at least one accumulation"))
}

#[allow(clippy::too_many_arguments)]
fn fold_mean(
    elements: &[Element],
    elem_name: Option<&str>,
    body: &SExpr,
    weight: Option<&SExpr>,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    if elements.is_empty() {
        return Err(EvalError::coded(
            EvalCode::EmptyAggregate,
            "mean over an empty query (§4.4) — there is no element to average",
        ));
    }
    let mut sum_wx = 0.0_f64;
    let mut sum_w = 0.0_f64;
    for element in elements.iter().cloned() {
        let (body_val, weight_val) =
            eval_body_and_weight(element, elem_name, body, weight, env, host, fuel)?;
        let x = match body_val {
            Value::Real(r) => r,
            Value::Int(_) => {
                return Err(EvalError::plain(
                    "fold mean over an Int-typed body refuses loudly (D-row Q6, \
                     Director ruling 2026-08-11): mean serves Real-typed bodies \
                     only; Int has no pinned promotion rule here — divide in \
                     the binary64 lane instead",
                ))
            }
            other => {
                return Err(EvalError::plain(format!(
                    "fold mean body must be Real-typed, got {other:?}"
                )))
            }
        };
        let w = match weight_val {
            Some(Value::Real(r)) => r,
            #[allow(clippy::cast_precision_loss)]
            Some(Value::Int(n)) => n as f64,
            Some(other) => {
                return Err(EvalError::plain(format!(
                    "fold mean :weight must be numeric, got {other:?}"
                )))
            }
            None => 1.0,
        };
        // D-row Q5: Σ(wᵢ·xᵢ) ÷ Σwᵢ, both sums reduced in ITERATION order —
        // sequential accumulation into a local, never a reordering fold.
        // Rust/LLVM does NOT contract `w * x + sum_wx` into a fused
        // multiply-add without an explicit `mul_add` call (no
        // `-ffast-math`-equivalent is in force in this crate), so `w * x`
        // and the `+=` below are two separately-rounded IEEE-754 ops, not
        // one higher-precision FMA — which is exactly what §4.3
        // conformance (and the exact-bits vector this reduction feeds)
        // requires.
        sum_wx += w * x;
        sum_w += w;
    }
    if sum_w == 0.0 {
        return Err(EvalError::coded(
            EvalCode::DivisionByZero,
            "fold mean — the sum of weights is zero",
        ));
    }
    let result = sum_wx / sum_w;
    if !result.is_finite() {
        return Err(EvalError::coded(
            EvalCode::NonFinite,
            "fold mean produced a non-finite result",
        ));
    }
    Ok(Value::Real(result))
}

#[allow(clippy::too_many_arguments)]
fn fold_min_max(
    elements: &[Element],
    elem_name: Option<&str>,
    body: &SExpr,
    weight: Option<&SExpr>,
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
    want_min: bool,
) -> Result<Value, EvalError> {
    if elements.is_empty() {
        return Err(EvalError::coded(
            EvalCode::EmptyAggregate,
            format!(
                "{} over an empty query (§4.4)",
                if want_min { "min" } else { "max" }
            ),
        ));
    }
    let op = if want_min { "<" } else { ">" };
    let mut acc: Option<Value> = None;
    for element in elements.iter().cloned() {
        let (body_val, _weight_val) =
            eval_body_and_weight(element, elem_name, body, weight, env, host, fuel)?;
        acc = Some(match acc {
            None => body_val,
            Some(prev) => {
                let strictly_better =
                    matches!(apply_ordering(op, &body_val, &prev)?, Value::Bool(true));
                if strictly_better {
                    body_val
                } else {
                    prev
                }
            }
        });
    }
    Ok(acc.expect("non-empty elements guarantees at least one accumulation"))
}

/// `(edge-between <enum-ref> <expr> <expr>)` (§2.10). A keyed lookup (§2.10's own "well-defined
/// because the triple is a key" ruling) — never a set, never a silent no-op on absence.
fn eval_edge_between(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::ACCESSOR_BASE)?;
    let [_, type_ref, from_expr, to_expr] = items else {
        return Err(EvalError::plain(
            "(edge-between <enum-ref> <expr> <expr>) — unrecognized shape",
        ));
    };
    let edge_type = crate::query::enum_member(type_ref)?;
    let from = match evaluate(from_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(edge-between …)'s first node operand must evaluate to a NodeRef, got {other:?}"
            )))
        }
    };
    let to = match evaluate(to_expr, env, host, fuel)? {
        Value::NodeRef(id) => id,
        other => {
            return Err(EvalError::plain(format!(
                "(edge-between …)'s second node operand must evaluate to a NodeRef, got {other:?}"
            )))
        }
    };
    let graph = require_graph(env, "edge-between")?;
    // Full qname — `edge_attribute` mirrors `node_attribute`'s own convention and takes the FULL
    // qname, never a bare segment (T2 plan Task 1,
    // `docs/superpowers/plans/2026-08-12-t2-slice2-edge-reads-plan.md`). Constructed here because
    // `render_member` lives in THIS crate (`crate::vocabulary`) and `eval_edge_between` has only
    // the raw enum member (e.g. "SOLIDARITY"), never a content-authored qname to pass through
    // unmodified the way `field_of_edge` does.
    let strength_qname = format!("{}/strength", crate::vocabulary::render_member(edge_type));
    graph
        .edge_attribute(edge_type, from, to, &strength_qname)
        .map_err(|_| {
            EvalError::coded(
                EvalCode::NoSuchEdge,
                format!(
                    "(edge-between EdgeType/{edge_type} …): no edge from {from:?} to {to:?} \
                     (§2.10) — the accessor never yields an absent reference"
                ),
            )
        })?;
    Ok(Value::EdgeRef(EdgeKey {
        source: from,
        target: to,
        edge_type: edge_type.to_owned(),
    }))
}

/// `(field-of <expr> <qname>)` (§2.10). Serves `NodeRef` referents (slice 1),
/// `EdgeRef` referents (slice 2, T2 issue #559), and `HyperedgeRef`
/// referents (Community port train Task 6, E2b — a hyperedge's OWN field,
/// the substrate storage Task 5 minted). A MEMBERSHIP's payload reads
/// through `membership-field-of` (the AG(i) lane, #653), never through
/// `field-of` over a hyperedge.
fn eval_field_of(
    items: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    charge(fuel, cost::ACCESSOR_BASE)?;
    let [_, ref_expr, SExpr::Atom(Atom::QName(qname))] = items else {
        return Err(EvalError::plain(
            "(field-of <expr> <qname>) — unrecognized shape",
        ));
    };
    let referent = evaluate(ref_expr, env, host, fuel)?;
    match referent {
        Value::NodeRef(id) => field_of_node(id, qname, env),
        Value::EdgeRef(key) => field_of_edge(&key, qname, env),
        Value::HyperedgeRef(id) => field_of_hyperedge(id, qname, env),
        other => Err(EvalError::plain(format!(
            "(field-of …)'s first operand must evaluate to a reference, got \
             {other:?} (§2.10)"
        ))),
    }
}

/// §2.10 discipline 1, shared by every accessor AND update verb whose
/// referent is a reference: the qname's owning type (§2.9) must match the
/// referent's declared type. A reference has no static type (§3.1), so this
/// disagreement — `E-TYPE-014` for the operand-typed verbs (`add-node`,
/// `add-edge`, `add-hyperedge`) — can only surface HERE, at evaluation, as
/// `E-EVAL-033` (R9 chapter C2's own words: "the same disagreement surfaces
/// at evaluation as E-EVAL-033"). `form` names the caller for the message
/// only (`"field-of"`, `"update-node"`, …); the check itself is one rule,
/// not one per caller — `field_of_node` (§2.10) and `structural_verbs::
/// update_node` (§2.7's worked example, Task 11) share this exact
/// comparison, reusing `tick::namespace_to_node_type`'s rendering rather
/// than a third one.
///
/// # Errors
///
/// `E-EVAL-033` if `id` names no live node, or if it does but is not of the
/// qname's owning type.
pub(crate) fn check_node_referent_type(
    graph: &dyn GraphSubstrate,
    id: babylon_graph::substrate::NodeId,
    qname: &str,
    form: &str,
) -> Result<(), EvalError> {
    let owner_segment = qname.split('/').next().unwrap_or(qname);
    let expected_type = crate::tick::namespace_to_node_type(owner_segment);
    let actual_type = graph.node_type_of(id).map_err(|e| {
        EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!("{form} {qname}: {} (§2.10 discipline 1)", e.message),
        )
    })?;
    if actual_type != expected_type {
        return Err(EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "{form} {qname}: the referent is a {actual_type} node, not \
                 {expected_type} — the qname's owning type does not match \
                 the referent's declared type (§2.10 discipline 1)"
            ),
        ));
    }
    Ok(())
}

/// The `NodeRef` half of `field-of`'s shared discipline (§2.10):
/// 1. the qname's owning type must match the referent's declared type
///    (`check_node_referent_type`);
/// 2. absence is not a value — a never-written field is the same
///    `E-EVAL-033` as a type mismatch, never a default `0.0`;
/// 3. **(D102 discharge, Task 1 P27 territory-port train)** an
///    `:enum-type`-declared field renders its stored ordinal to
///    `Value::Enum` through `tick::bind_field_value` — the SAME rendering
///    a `:field` binding's read path uses (§2.5) — rather than the raw
///    `Value::Real` every other field gets.
///
/// **`env.types`/`env.enums` are REQUIRED (PR A verifier fix round,
/// 2026-08-12), mirroring [`require_graph`]'s own driver-error shape one
/// line up: a `None` here refuses loudly rather than silently degrading to
/// the `Value::Real` lane.** Before this fix, `_ => Ok(Value::Real(value))`
/// was a coincidental safety net — correct ONLY because every production
/// caller reaching this line with a live graph ALSO happened to thread
/// `types`/`enums` (`tick::collect_pass`, `rule_pipeline::
/// resolve_expr_bindings`), never because an untyped `EvalEnv` was a
/// legitimate shape for a `field-of`-evaluating one. §2.13 makes a field's
/// declared type the authority on how `field-of` renders it; an `EvalEnv`
/// that cannot consult that authority is the same "driver built an
/// environment missing what this form needs" bug `require_graph` already
/// names for a missing graph, not a value this function should ever guess
/// at with a plausible-looking default.
fn field_of_node(
    id: babylon_graph::substrate::NodeId,
    qname: &str,
    env: &EvalEnv<'_>,
) -> Result<Value, EvalError> {
    let graph = require_graph(env, "field-of")?;
    check_node_referent_type(graph, id, qname, "field-of")?;
    let (Some(types), Some(enums)) = (env.types, env.enums) else {
        return Err(EvalError::plain(format!(
            "(field-of …) needs the declared field-type registry (§2.13) but \
             this EvalEnv carries none for {qname} — a driver error, the same \
             shape as require_graph's: the caller built an environment with \
             no declared-type registry for a field-of-evaluating form, never \
             a value this function should guess at with a plausible-looking \
             default"
        )));
    };
    // T3 #491, OQ-J: a `currency`-declared field lives in the substrate's
    // SEPARATE i128 map, never `node_attribute`'s f64 one — the SAME
    // type-first dispatch `tick::bind_subject` uses for a `:field` binding,
    // reused here so `field-of` reads a Currency field correctly rather
    // than mis-reporting a real value as "never written" (the map it would
    // have read `node_attribute` against is simply the wrong one).
    if matches!(
        types.fields.get(qname).map(|decl| &decl.ty),
        Some(BslType::Currency)
    ) {
        return graph
            .node_attribute_currency(id, qname)
            .map(Value::Currency)
            .map_err(|e| {
                EvalError::coded(
                    EvalCode::AccessorTypeOrValueMismatch,
                    format!(
                        "field-of {qname}: {} (§2.10 discipline 2 — absence is not a value)",
                        e.message
                    ),
                )
            });
    }
    let value = graph.node_attribute(id, qname).map_err(|e| {
        EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "field-of {qname}: {} (§2.10 discipline 2 — absence is not a \
                 value)",
                e.message
            ),
        )
    })?;
    crate::tick::bind_field_value(qname, value, types, enums)
        .map_err(|e| EvalError::plain(e.to_string()))
}

/// The `EdgeRef` half of §2.10 discipline 1. Cheaper than `check_node_referent_type`: an `EdgeKey`
/// carries its type inline (§2.6's own key), so no substrate round-trip is needed to learn it —
/// unlike a `NodeRef`, whose type needs `graph.node_type_of`. Shared by the read side
/// (`field_of_edge`) and — since T3 (ADR198 R3, issue #560) — the write side
/// (`structural_verbs.rs`'s `update-edge`, whose E-EVAL-033 obligation is the same law).
pub(crate) fn check_edge_referent_type(
    key: &EdgeKey,
    qname: &str,
    form: &str,
) -> Result<(), EvalError> {
    let owner_segment = qname.split('/').next().unwrap_or(qname);
    let expected_type = crate::tick::namespace_to_node_type(owner_segment);
    if key.edge_type != expected_type {
        return Err(EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "{form} {qname}: the referent is a {} edge, not {expected_type} — the qname's \
                 owning type does not match the referent's declared type (§2.10 discipline 1)",
                key.edge_type
            ),
        ));
    }
    Ok(())
}

/// The `HyperedgeRef` half of §2.10 discipline 1 (Community port train,
/// Task 6, E2b). Unlike an `EdgeKey`, a `HyperedgeId` does NOT carry its
/// type inline, so — like `check_node_referent_type` — the check needs a
/// substrate round-trip through `GraphSubstrate::hyperedge_type_of` (the
/// read-only accessor PR #684 minted for exactly this). The rendering rule
/// is the same one both sibling checks reuse: the stored type string is the
/// verbatim `HyperedgeType` member (e.g. `COMMUNITY`), which
/// `tick::namespace_to_node_type`'s uppercase + dash→underscore transform
/// reproduces from the qname's owning segment. Shared by the read side
/// (`field_of_hyperedge`) and the write side (`structural_verbs.rs`'s
/// `update-hyperedge`), whose E-EVAL-033 obligation is the same law.
///
/// # Errors
///
/// `E-EVAL-033` if `id` names no live hyperedge, or if it does but is not
/// of the qname's owning type.
pub(crate) fn check_hyperedge_referent_type(
    id: babylon_graph::substrate::HyperedgeId,
    qname: &str,
    form: &str,
    graph: &dyn GraphSubstrate,
) -> Result<(), EvalError> {
    let owner_segment = qname.split('/').next().unwrap_or(qname);
    let expected_type = crate::tick::namespace_to_node_type(owner_segment);
    let actual_type = graph.hyperedge_type_of(id).map_err(|e| {
        EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!("{form} {qname}: {} (§2.10 discipline 1)", e.message),
        )
    })?;
    if actual_type != expected_type {
        return Err(EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "{form} {qname}: the referent is a {actual_type} hyperedge, not \
                 {expected_type} — the qname's owning type does not match \
                 the referent's declared type (§2.10 discipline 1)"
            ),
        ));
    }
    Ok(())
}

/// The `EdgeRef` half of `field-of`'s shared discipline (§2.10) — generic over any qname whose
/// owning type is an `EdgeType` (T2 dossier §2 design: not hard-coded to `strength` alone). `qname`
/// is passed to the substrate UNMODIFIED, exactly as `field_of_node` passes it to `node_attribute`
/// — the FULL qname is the key convention both share, `GraphSubstrate::edge_attribute`'s own
/// documented contract (T2 plan Task 1,
/// `docs/superpowers/plans/2026-08-12-t2-slice2-edge-reads-plan.md`); a T3-era edge
/// field resolves through this SAME function unmodified — only `GraphSubstrate::edge_attribute`'s
/// body widens.
///
/// **`edge_attribute`'s "no such edge" failure mode (E-EVAL-034's own condition) is UNREACHABLE
/// here.** Every `Value::EdgeRef` this function's caller (`eval_field_of`) can ever hand it was
/// already validated to reference a LIVE edge before construction: `materialize_edges` builds one
/// only from `graph.edges(edge_type)`'s own output (which by construction never names a dangling
/// triple), and `eval_edge_between` only returns a `Value::EdgeRef` AFTER its own existence check
/// already succeeded (erroring `E-EVAL-034` itself, before ever constructing one, otherwise). So the
/// ONLY way `graph.edge_attribute(...)` can fail from inside this function is the "qname is not
/// `<edge-type>/strength`" branch — the mapping below is therefore always `E-EVAL-033`
/// ("declared/legal but never written") in practice, never a laundered `E-EVAL-034`.
fn field_of_edge(key: &EdgeKey, qname: &str, env: &EvalEnv<'_>) -> Result<Value, EvalError> {
    let graph = require_graph(env, "field-of")?;
    check_edge_referent_type(key, qname, "field-of")?;
    let value = graph
        .edge_attribute(&key.edge_type, key.source, key.target, qname)
        .map_err(|e| {
            EvalError::coded(
                EvalCode::AccessorTypeOrValueMismatch,
                format!(
                    "field-of {qname}: {} (§2.10 discipline 2 — absence is not a value)",
                    e.message
                ),
            )
        })?;
    let (Some(types), Some(enums)) = (env.types, env.enums) else {
        return Err(EvalError::plain(format!(
            "(field-of …) needs the declared field-type registry (§2.13) but this EvalEnv \
             carries none for {qname} — a driver error, the same shape as require_graph's"
        )));
    };
    crate::tick::bind_field_value(qname, value, types, enums)
        .map_err(|e| EvalError::plain(e.to_string()))
}

/// The `HyperedgeRef` half of `field-of`'s shared discipline (§2.10) —
/// Community port train Task 6 (E2b), reading the own-field storage Task 5
/// (E2a) minted. Mirrors [`field_of_edge`] exactly: §2.10 discipline 1
/// through [`check_hyperedge_referent_type`], then the substrate read, with
/// the same "absence is not a value" mapping (E-EVAL-033) — a never-written
/// hyperedge field is a mismatch, never a default `0.0`.
///
/// **`hyperedge_attribute`'s "no such hyperedge" failure mode is UNREACHABLE
/// here**, for the same reason `field_of_edge`'s doc names: every
/// `Value::HyperedgeRef` `eval_field_of` can hand this function was built by
/// `query::materialize` from the substrate's own live census, and
/// [`check_hyperedge_referent_type`] has already errorred on a dangling id
/// before the read. The substrate error that can actually surface is the
/// never-written one — hence always E-EVAL-033, never a laundered existence
/// error.
fn field_of_hyperedge(
    id: babylon_graph::substrate::HyperedgeId,
    qname: &str,
    env: &EvalEnv<'_>,
) -> Result<Value, EvalError> {
    let graph = require_graph(env, "field-of")?;
    check_hyperedge_referent_type(id, qname, "field-of", graph)?;
    let value = graph.hyperedge_attribute(id, qname).map_err(|e| {
        EvalError::coded(
            EvalCode::AccessorTypeOrValueMismatch,
            format!(
                "field-of {qname}: {} (§2.10 discipline 2 — absence is not a value)",
                e.message
            ),
        )
    })?;
    let (Some(types), Some(enums)) = (env.types, env.enums) else {
        return Err(EvalError::plain(format!(
            "(field-of …) needs the declared field-type registry (§2.13) but this EvalEnv \
             carries none for {qname} — a driver error, the same shape as require_graph's"
        )));
    };
    crate::tick::bind_field_value(qname, value, types, enums)
        .map_err(|e| EvalError::plain(e.to_string()))
}

fn eval_intrinsic(
    name: &str,
    args: &[SExpr],
    env: &EvalEnv<'_>,
    host: &dyn IntrinsicHost,
    fuel: &mut u64,
) -> Result<Value, EvalError> {
    let Some(declared) = env.intrinsic_costs.declared_cost(name) else {
        return Err(EvalError::plain(format!(
            "call to undeclared intrinsic {name} — a load-time gate \
             (E-LOAD-021, §2.7); reaching this at evaluation is a loader bug"
        )));
    };
    charge(fuel, cost::INTRINSIC_CALL_BASE.saturating_add(declared))?;
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(evaluate(arg, env, host, fuel)?);
    }
    let ctx = build_intrinsic_call_ctx(env)?;
    host.call(name, &values, ctx)
}

/// Build the [`IntrinsicCallCtx`] every intrinsic call carries. `None`
/// `env.draw_context` (a pure-expression caller) renders an empty,
/// context-free ctx because there is no
/// [`crate::intrinsic_host::DrawContext`] against which to resolve
/// `env.elements`. Author-declarable intrinsics do not consume draw context;
/// the engine-private finite-kernel realization path does.
///
/// `Some(draw_context)` resolves the chapter C8 element stack in the same
/// outermost-first order `env.elements` keeps. V1 uses its current content-id
/// map and fixture fallback. V2 resolves each runtime element through the
/// sealed stable graph resolver and has no string or debug fallback.
///
/// # Errors
///
/// [`EvalError`] if V1 content-id resolution or V2 stable graph resolution
/// refuses an element.
fn build_intrinsic_call_ctx<'a>(env: &EvalEnv<'a>) -> Result<IntrinsicCallCtx<'a>, EvalError> {
    let Some(draw_context) = env.draw_context else {
        return Ok(IntrinsicCallCtx::context_free());
    };
    let active_elements = match &draw_context.identity {
        DrawIdentityContext::V1 {
            node_content_ids, ..
        } => env
            .elements
            .iter()
            .map(|(_, element)| {
                element_content_id(element, *node_content_ids).map(DrawActiveElement::V1)
            })
            .collect::<Result<Vec<_>, _>>()?,
        DrawIdentityContext::V2 { resolver, .. } => env
            .elements
            .iter()
            .map(|(_, element)| stable_element_key(element, resolver).map(DrawActiveElement::V2))
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok(IntrinsicCallCtx {
        draw_context: Some(draw_context),
        active_elements,
    })
}

fn stable_element_key(
    element: &Element,
    resolver: &StableElementResolverV1,
) -> Result<StableElementKeyV1, EvalError> {
    let result = match element {
        Element::Node(node) => resolver.node_key(*node).cloned(),
        Element::Hyperedge(hyperedge) => resolver.hyperedge_key(*hyperedge).cloned(),
        Element::Edge(key) => resolver.edge_key(&key.edge_type, key.source, key.target),
    };
    result.map_err(|error| {
        EvalError::plain(format!(
            "finite-kernel V2 active element has no sealed stable identity: {error:?}"
        ))
    })
}

/// Resolve one §2.6 chapter C8 element to its content-id chain entry
/// (plan §3.5): a [`Element::Node`] resolves to its bare Task-3 content
/// id; a [`Element::Edge`] resolves to its two endpoints' content ids
/// **and its `edge_type`**, composed by [`crate::intrinsic_host::framed`]
/// into ONE entry.
///
/// **`edge_type` joined the chain in review round 2 (#576 I1).** Plan
/// §3.5's original wording ("its two endpoints' content ids, framed")
/// composed only `source`/`target` — `EdgeKey` carries a THIRD field,
/// `edge_type`, and dropping it made two parallel edges of DIFFERENT types
/// between the SAME node pair key-indistinguishable: `(edge SOLIDARITY a
/// b)` and `(edge EXPLOITATION a b)` produced bit-identical `stable_key`s,
/// so a rule drawing once per edge across two `for-each` loops over each
/// type gave the two materially distinct relations perfectly correlated
/// randomness. `framed`'s length-prefix encoding stays injective with the
/// third segment for the same reason it was injective with two (this
/// function's own doc, unchanged): each segment is self-delimiting, so no
/// three-segment chain can collide with a different three-segment chain.
///
/// **Review round 1 (#576) tightened the per-node lookup from an
/// unconditional fallback to a TYPE-distinguished one (review round 2,
/// #576 I2 — see [`crate::intrinsic_host::DrawIdentityContext::V1`]).** `None`
/// means no scenario was ever hydrated — this crate's own hand-built
/// `MemoryGraph` fixtures (`tick.rs`'s tests) are exactly this shape — so
/// a `NodeId` here carries its own `Debug` rendering (`{id:?}`) instead of
/// a content id: a node nothing ever named, not a missing value, so not an
/// III.11 violation. `Some(map)` means a scenario WAS hydrated, even when
/// `map` is empty (a declarations-only scenario, zero `(node …)` forms) —
/// against `Some(map)`, a miss is ALWAYS a hard [`EvalError`], `map.is_empty()`
/// or not.
///
/// This is the self-enforcing half of a cross-file invariant this function
/// does NOT itself control, so it is named here explicitly: every `NodeId`
/// a *scenario-hydrated* graph can hold is named
/// (`scenario::invert_content_ids` inverts the exact `named` table hydration
/// builds), and the only OTHER way to mint a `NodeId` — the six graph-shape
/// verbs (`add-node`/`remove-node`/`add-edge`/`remove-edge`/
/// `add-hyperedge`/`remove-hyperedge`, `structural_verbs::
/// DEFERRED_SHAPE_VERBS`, `structural_verbs.rs:1723`) — is refused
/// unconditionally at content load
/// (`structural_verbs::check_no_deferred_shape_verbs`, wired into every
/// rule load at `rule_pipeline.rs:269`). Lifting that load-time gate is a
/// NAMED FUTURE TASK (`EffectExecutor::collect_effects`'s own doc: "a rule
/// that needs one is a declared, escalated gap"). **Whoever lifts it must
/// also update `node_content_ids` for any node minted mid-tick, or this
/// hard error is exactly the trip wire that catches the gap** — the
/// alternative (an unconditional fallback) would have silently injected
/// the raw, insertion-order-dependent `NodeId` handle into the private
/// finite-kernel draw key via `framed(...)`, precisely the ADR176 r20 butterfly
/// plan §3.4's whole content-id design exists to prevent — with no error,
/// no failing test, only a downstream, hard-to-attribute divergence.
///
/// # Errors
///
/// [`EvalError`] if a `NodeId` this element names is absent from a
/// `Some`-hydrated `node_content_ids` map — a hydration bug (a node the
/// substrate holds that Task-3's map never recorded), never a legitimate
/// "this node has no name" case once the map is known to be hydrated.
fn element_content_id(
    element: &Element,
    node_content_ids: Option<&HashMap<babylon_graph::substrate::NodeId, String>>,
) -> Result<String, EvalError> {
    let content_id_of = |id: &babylon_graph::substrate::NodeId| -> Result<String, EvalError> {
        match node_content_ids {
            None => {
                // Never hydrated (see this function's own doc) — no node
                // here has a declared name. Not a hydration bug: there was
                // never a map to miss.
                Ok(format!("{id:?}"))
            }
            Some(map) => map.get(id).cloned().ok_or_else(|| {
                EvalError::plain(format!(
                    "node {id:?} carries no Task-3 content id, but \
                     node_content_ids IS hydrated ({} entries) — a \
                     hydration bug: every scenario-hydrated node is named \
                     (scenario::invert_content_ids), so a NodeId reaching \
                     here with no entry means something minted a node \
                     outside hydration without recording its content id \
                     (review round 2, #576 I2 — see this function's own doc)",
                    map.len()
                ))
            }),
        }
    };
    match element {
        Element::Node(id) => content_id_of(id),
        // Slice 3 (Community Task 3): hyperedges have no content-id
        // hydration map at all today (no scenario-side hyperedge name table
        // is fed to the tick — the `declared_hyperedges` names live and die
        // with the executor). The `HyperedgeId` IS declaration-ordered and
        // deterministic, so the raw handle rendering is deterministic too —
        // the same shape the `None` arm above gives nodes when no scenario
        // was hydrated. If a hyperedge content-id lane ever lands, this arm
        // moves to it.
        Element::Hyperedge(id) => Ok(format!("{id:?}")),
        Element::Edge(key) => {
            let source = content_id_of(&key.source)?;
            let target = content_id_of(&key.target)?;
            Ok(crate::intrinsic_host::framed(&[
                &source,
                &target,
                &key.edge_type,
            ]))
        }
    }
}

pub(crate) fn as_bool(value: Value) -> Result<bool, EvalError> {
    match value {
        Value::Bool(b) => Ok(b),
        other => Err(EvalError::plain(format!(
            "expected Bool where a <cond> is required, got {other:?}"
        ))),
    }
}

/// The binary64 view of a value, for the promoting lane (§3.3: `Int`
/// promotes to `Real` in a binary64 expression, never to `Currency`).
fn real_lane(value: &Value) -> Option<f64> {
    match value {
        Value::Real(r) => Some(*r),
        // i64 → f64 is round-to-nearest and deterministic; exactness above
        // 2⁵³ is not promised by §3.3's promotion, determinism is.
        #[allow(clippy::cast_precision_loss)]
        Value::Int(n) => Some(*n as f64),
        _ => None,
    }
}

fn apply_arith(op: &str, lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    match (lhs, rhs) {
        (Value::Mass(a), Value::Mass(b)) if op == "+" => a
            .checked_add(*b)
            .map(Value::Mass)
            .map_err(|error| EvalError::plain(error.to_string())),
        (Value::Mass(a), Value::Mass(b)) if op == "-" => a
            .checked_sub(*b)
            .map(Value::Mass)
            .map_err(|error| EvalError::plain(error.to_string())),
        (Value::Mass(_), _) | (_, Value::Mass(_)) => Err(EvalError::plain(
            "Mass permits checked Mass + Mass and Mass - Mass only; it has no implicit coercion",
        )),
        (Value::Int(a), Value::Int(b)) => arith_int(op, *a, *b),
        (Value::Currency(a), Value::Currency(b)) => arith_currency(op, *a, *b),
        (Value::Currency(c), Value::Int(divisor)) if op == "/" => {
            currency_div_integer(*c, *divisor)
        }
        // §3.2 addendum (#492/ADR194): Currency × Ratio, the fifth legal
        // mixed operation. Matched BEFORE the Coefficient arm below so a
        // `Value::Ratio` operand never falls through to the "must be
        // Value::Real" refusal — the two carriers are disjoint by
        // construction (`atom_value` never produces `Real` for an `r`
        // literal), so there is no ordering ambiguity, only two independent
        // positive matches.
        (Value::Currency(c), Value::Ratio { value, floor, cap })
        | (Value::Ratio { value, floor, cap }, Value::Currency(c))
            if op == "*" =>
        {
            currency_mul_ratio(*c, *value, *floor, *cap)
        }
        (Value::Currency(c), other) | (other, Value::Currency(c)) if op == "*" => {
            // §3.2: Currency multiplies by a Coefficient ONLY (Ratio is
            // handled above). `Real` is the runtime coefficient carrier
            // (c-literals and bindings land there; the [0,1] domain is
            // enforced below) — `Int` is a type error at ANY value
            // (bsl-language.rst:849), so it must not slip through the
            // promoting lane even where its f64 image would be a legal
            // coefficient (0 and 1).
            let Value::Real(coeff) = other else {
                return Err(EvalError::plain(format!(
                    "Currency × {other:?} is not in the §3.2 operator table \
                     (E-TYPE-030) — multiply by a Coefficient or a declared-\
                     domain Ratio instead"
                )));
            };
            currency_mul_coefficient(*c, *coeff)
        }
        (Value::Currency(_), other) | (other, Value::Currency(_)) => {
            Err(EvalError::plain(format!(
                "Currency {op} {other:?} is not in the §3.2 operator table \
                 (E-TYPE-030) — the five pinned operations are ± Currency, \
                 × Coefficient, × Ratio, ÷ Currency, ÷ integer"
            )))
        }
        _ => match (real_lane(lhs), real_lane(rhs)) {
            (Some(a), Some(b)) => arith_real(op, a, b),
            _ => Err(EvalError::plain(format!(
                "no arithmetic is defined on ({op} {lhs:?} {rhs:?})"
            ))),
        },
    }
}

fn arith_int(op: &str, a: i64, b: i64) -> Result<Value, EvalError> {
    let overflow = || {
        EvalError::coded(
            EvalCode::Overflow,
            format!("Int overflow: ({op} {a} {b}) leaves i64"),
        )
    };
    match op {
        "+" => a.checked_add(b).map(Value::Int).ok_or_else(overflow),
        "-" => a.checked_sub(b).map(Value::Int).ok_or_else(overflow),
        "*" => a.checked_mul(b).map(Value::Int).ok_or_else(overflow),
        "/" => Err(EvalError::plain(
            "Int ÷ Int has no pinned semantics: truncation is never implicit \
             (§3.2) and §3.3 promotes Int only in a binary64 expression — \
             flagged for the Phase-1 review; divide in the binary64 lane or \
             use Currency ÷ integer",
        )),
        _ => Err(EvalError::plain(format!(
            "unknown arithmetic operator {op}"
        ))),
    }
}

fn arith_real(op: &str, a: f64, b: f64) -> Result<Value, EvalError> {
    if op == "/" && b == 0.0 {
        return Err(EvalError::coded(
            EvalCode::DivisionByZero,
            "division by zero in the binary64 lane",
        ));
    }
    let result = match op {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => a / b,
        _ => {
            return Err(EvalError::plain(format!(
                "unknown arithmetic operator {op}"
            )))
        }
    };
    if result.is_finite() {
        Ok(Value::Real(result))
    } else {
        Err(EvalError::coded(
            EvalCode::NonFinite,
            format!("({op} {a} {b}) produced a non-finite binary64 result"),
        ))
    }
}

fn arith_currency(op: &str, a: Currency, b: Currency) -> Result<Value, EvalError> {
    match op {
        "+" | "-" => {
            let result = if op == "+" {
                a.checked_add(b)
            } else {
                a.checked_sub(b)
            }
            .map_err(|e| EvalError::coded(EvalCode::Overflow, format!("{} left i128", e.op)))?;
            if result.micro_units() < 0 {
                return Err(EvalError::coded(
                    EvalCode::CurrencyNegative,
                    "Currency result below zero — the domain is [0, ∞) (§3.2)",
                ));
            }
            Ok(Value::Currency(result))
        }
        "/" => {
            // Pre-check div_currency's panic preconditions so a rule
            // failure is a structured tick-abort (§4.6), never a process
            // abort: zero divisor is E-EVAL-012; a ratio outside [0,1] is
            // E-EVAL-013.
            if b.micro_units() == 0 {
                return Err(EvalError::coded(
                    EvalCode::DivisionByZero,
                    "Currency ÷ Currency with a zero divisor",
                ));
            }
            let in_domain = if b.micro_units() > 0 {
                (0..=b.micro_units()).contains(&a.micro_units())
            } else {
                (b.micro_units()..=0).contains(&a.micro_units())
            };
            if !in_domain {
                return Err(EvalError::coded(
                    EvalCode::CoefficientOutOfRange,
                    "Currency ÷ Currency ratio outside the Coefficient [0,1] domain",
                ));
            }
            Ok(Value::Real(a.div_currency(b).get()))
        }
        "*" => Err(EvalError::plain(
            "Currency × Currency is E-TYPE-030 (an area of money) — \
             multiply by a Coefficient instead (§3.2)",
        )),
        _ => Err(EvalError::plain(format!(
            "unknown arithmetic operator {op}"
        ))),
    }
}

fn currency_div_integer(c: Currency, divisor: i64) -> Result<Value, EvalError> {
    if divisor <= 0 {
        return Err(EvalError::coded(
            EvalCode::DivisionByZero,
            "Currency ÷ integer with a zero or negative divisor (§3.2)",
        ));
    }
    Ok(Value::Currency(c.div_integer(i128::from(divisor))))
}

fn currency_mul_coefficient(c: Currency, coeff: f64) -> Result<Value, EvalError> {
    let coefficient = Coefficient::new(coeff).map_err(|_| {
        EvalError::plain(format!(
            "Currency × {coeff} — the multiplier is outside the Coefficient \
             [0,1] domain, so the expression is Currency × Real, which is \
             E-TYPE-030 (§3.2); the typechecker is the primary gate here"
        ))
    })?;
    let result = c
        .mul_coefficient(coefficient)
        .map_err(|e| EvalError::coded(EvalCode::Overflow, format!("{} left i128", e.op)))?;
    Ok(Value::Currency(result))
}

/// `Currency × Ratio → Currency` (§3.2 addendum, #492/ADR194). `ratio` is
/// already a valid [`Ratio`] by construction (the reader's `E-LEX-027` and
/// `scenario.rs`'s `defconst` loader both gate it before it ever reaches a
/// [`Value`]), so the only thing left to check HERE, at the point of use, is
/// the declared bounds (`E-EVAL-041`) — the re-check `Value::Ratio`'s own
/// doc comment explains (defense in depth, not redundant structure: nothing
/// re-derives `floor`/`cap` from `ratio` itself, so this is the one place
/// that can still catch a bound that drifted from the value it was declared
/// against). `floor` is EXCLUSIVE (`ratio` must be strictly greater than
/// it); `cap` is INCLUSIVE (`ratio` may equal it) — the same asymmetry
/// `scenario.rs::load_ratio_defconst` checks at load.
fn currency_mul_ratio(
    c: Currency,
    ratio: Ratio,
    floor: Option<Ratio>,
    cap: Option<Ratio>,
) -> Result<Value, EvalError> {
    if let Some(floor) = floor {
        if ratio.get() <= floor.get() {
            return Err(EvalError::coded(
                EvalCode::RatioOutsideDeclaredDomain,
                format!(
                    "Currency × {} — the Ratio does not exceed its declared \
                     :floor of {} (EXCLUSIVE; §3.2 addendum, #492/ADR194)",
                    ratio.get(),
                    floor.get()
                ),
            ));
        }
    }
    if let Some(cap) = cap {
        if ratio.get() > cap.get() {
            return Err(EvalError::coded(
                EvalCode::RatioOutsideDeclaredDomain,
                format!(
                    "Currency × {} — the Ratio exceeds its declared :cap of \
                     {} (INCLUSIVE; §3.2 addendum, #492/ADR194)",
                    ratio.get(),
                    cap.get()
                ),
            ));
        }
    }
    let result = c
        .mul_ratio(ratio)
        .map_err(|e| EvalError::coded(EvalCode::Overflow, format!("{} left i128", e.op)))?;
    Ok(Value::Currency(result))
}

fn apply_ordering(op: &str, lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    let ordering = match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => a.cmp(b),
        (Value::Currency(a), Value::Currency(b)) => a.micro_units().cmp(&b.micro_units()),
        _ => match (real_lane(lhs), real_lane(rhs)) {
            // Values are finite by construction (E-EVAL-014), so binary64
            // ordering is total here.
            (Some(a), Some(b)) => a
                .partial_cmp(&b)
                .ok_or_else(|| EvalError::plain("non-finite value reached a comparison"))?,
            _ => {
                return Err(EvalError::plain(format!(
                    "({op} {lhs:?} {rhs:?}) — ordering is defined within one \
                     numeric lane only (Enum and Bool compare with =/!= alone, \
                     §3.1)"
                )))
            }
        },
    };
    let result = match op {
        "<" => ordering.is_lt(),
        "<=" => ordering.is_le(),
        ">" => ordering.is_gt(),
        ">=" => ordering.is_ge(),
        _ => return Err(EvalError::plain(format!("unknown ordering operator {op}"))),
    };
    Ok(Value::Bool(result))
}

fn apply_equality(op: &str, lhs: &Value, rhs: &Value) -> Result<Value, EvalError> {
    let equal = match (lhs, rhs) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (
            Value::Enum {
                enum_type: ta,
                member: ma,
            },
            Value::Enum {
                enum_type: tb,
                member: mb,
            },
        ) => {
            if ta != tb {
                return Err(EvalError::plain(format!(
                    "Enum<{ta}> compares only to the same enum type, found \
                     Enum<{tb}> (§3.1)"
                )));
            }
            ma == mb
        }
        (Value::NodeRef(a), Value::NodeRef(b)) => a == b,
        (Value::EdgeRef(a), Value::EdgeRef(b)) => a == b,
        (Value::HyperedgeRef(a), Value::HyperedgeRef(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Currency(a), Value::Currency(b)) => a == b,
        // Exact IEEE-754 equality: the basic ops are correctly rounded and
        // reproduce bit-exactly (§4.3), so exact comparison is
        // deterministic — clippy's suggested epsilon margin would be an
        // invented semantics the spec does not have.
        _ => match (real_lane(lhs), real_lane(rhs)) {
            #[allow(clippy::float_cmp)]
            (Some(a), Some(b)) => a == b,
            _ => {
                return Err(EvalError::plain(format!(
                    "({op} {lhs:?} {rhs:?}) — equality is defined within one \
                     lane only"
                )))
            }
        },
    };
    Ok(Value::Bool(if op == "=" { equal } else { !equal }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intrinsic_host::EmptyIntrinsicHost;
    use crate::reader::read;
    use std::collections::HashMap;

    fn costs() -> IntrinsicCosts {
        IntrinsicCosts::new(HashMap::from([("double".to_owned(), 4)]))
    }

    fn eval_with(
        source: &str,
        bindings: HashMap<String, Value>,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let costs = costs();
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read(source).expect("test source must parse");
        evaluate(&expr, &env, &EmptyIntrinsicHost, fuel)
    }

    fn eval(source: &str) -> Result<Value, EvalError> {
        let mut fuel = 10_000;
        eval_with(source, HashMap::new(), &mut fuel)
    }

    fn wealth_bindings() -> HashMap<String, Value> {
        HashMap::from([(
            "wealth".to_owned(),
            Value::Currency(Currency::from_micro_units(900_000_000)), // 900$
        )])
    }

    #[test]
    fn the_demo_rule_condition_evaluates_with_pinned_fuel() {
        // The §5.6 rule's condition against wealth = 900$: true, and the
        // consumed fuel is exactly the static per-node sum — cmp(1) +
        // variable-ref(1) + literal(0) = 2 of the declared 64.
        let mut fuel = 64;
        let result = eval_with("(< wealth 1000.5$)", wealth_bindings(), &mut fuel).unwrap();
        assert_eq!(result, Value::Bool(true));
        assert_eq!(
            fuel, 62,
            ":fuel-used is a conformance-vector quantity (§6.1)"
        );
    }

    #[test]
    fn arithmetic_is_strictly_binary() {
        let err = eval("(+ 1 2 3)").unwrap_err();
        assert!(err.message.contains("E-PARSE-040"), "{err}");
        assert_eq!(eval("(+ 1 2)").unwrap(), Value::Int(3));
    }

    #[test]
    fn int_arithmetic_is_checked_i64() {
        assert_eq!(eval("(* 3 4)").unwrap(), Value::Int(12));
        let err = eval("(* 9223372036854775807 2)").unwrap_err();
        assert_eq!(err.code, Some(EvalCode::Overflow));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-011");
    }

    #[test]
    fn int_division_is_a_loud_unpinned_gap_not_truncation() {
        let err = eval("(/ 4 2)").unwrap_err();
        assert_eq!(err.code, None);
        assert!(err.message.contains("no pinned semantics"), "{err}");
    }

    #[test]
    fn int_promotes_to_real_in_a_binary64_expression() {
        assert_eq!(eval("(+ 1 0.5c)").unwrap(), Value::Real(1.5));
        assert_eq!(eval("(* 2 0.25p)").unwrap(), Value::Real(0.5));
    }

    #[test]
    fn binary64_division_by_zero_is_e_eval_012() {
        let err = eval("(/ 1 0.0c)").unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DivisionByZero));
    }

    #[test]
    fn a_non_finite_binary64_result_is_e_eval_014() {
        // Square (1e9 + 0.5) six times: finite through 1e288, then inf.
        let mut source = "(+ 1000000000 0.5c)".to_owned();
        for _ in 0..6 {
            source = format!("(* {source} {source})");
        }
        let err = eval(&source).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::NonFinite));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-014");
    }

    #[test]
    fn currency_addition_and_subtraction_are_checked_both_ends() {
        assert_eq!(
            eval("(+ 900$ 100.5$)").unwrap(),
            Value::Currency(Currency::from_micro_units(1_000_500_000))
        );
        // Below zero: E-EVAL-010, the [0, ∞) domain — never a signed result.
        let err = eval("(- 100$ 200$)").unwrap_err();
        assert_eq!(err.code, Some(EvalCode::CurrencyNegative));
        // i128 overflow through a binding: E-EVAL-011.
        let big = HashMap::from([(
            "hoard".to_owned(),
            Value::Currency(Currency::from_micro_units(i128::MAX)),
        )]);
        let mut fuel = 10_000;
        let err = eval_with("(+ hoard hoard)", big, &mut fuel).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::Overflow));
    }

    #[test]
    fn currency_times_coefficient_rounds_half_even_and_commutes() {
        let half = Value::Currency(Currency::from_micro_units(500_000_000));
        assert_eq!(eval("(* 1000$ 0.5c)").unwrap(), half);
        assert_eq!(eval("(* 0.5c 1000$)").unwrap(), half);
    }

    /// §3.2 addendum (#492/ADR194): the whole point — a Ratio OUTSIDE
    /// Coefficient's [0,1] domain multiplies Currency cleanly, in either
    /// operand order, same rounding law as Coefficient.
    #[test]
    fn currency_times_ratio_accepts_values_above_one_and_commutes() {
        let expected = Value::Currency(Currency::from_micro_units(2_000_000_000));
        assert_eq!(eval("(* 1000$ 2r)").unwrap(), expected);
        assert_eq!(eval("(* 2r 1000$)").unwrap(), expected);
        // rent_spike_multiplier's exact moddable-to-2.0 shape from the
        // fixture the Territory port train cites.
        assert_eq!(
            eval("(* 1500.5$ 2.0r)").unwrap(),
            Value::Currency(Currency::from_micro_units(3_001_000_000))
        );
    }

    #[test]
    fn a_bare_ratio_literal_carries_no_cap_so_any_positive_value_is_legal() {
        // A literal `10r` in source has no declared ceiling (`cap: None`) —
        // only a `defconst`'s `:cap` narrows it (`scenario.rs`). Values far
        // beyond any of the four named consumers' domains are still legal
        // arithmetic here; the type itself is unbounded above.
        assert_eq!(
            eval("(* 1$ 1000000r)").unwrap(),
            Value::Currency(Currency::from_micro_units(1_000_000_000_000))
        );
    }

    /// The declared-ceiling re-check (`E-EVAL-041`) needs a `Value::Ratio`
    /// carrying `Some(cap)`, which only a `defconst`'s `:cap` produces in
    /// real content (`scenario.rs`, tested there end-to-end). Constructing
    /// one directly through `eval_with`'s binding map is the evaluator-level
    /// unit test for the SAME check, isolated from the loader.
    #[test]
    fn a_ratio_exceeding_its_declared_cap_is_e_eval_041_at_the_multiply() {
        let bindings = HashMap::from([(
            "k".to_owned(),
            Value::Ratio {
                value: Ratio::new(12.0).unwrap(),
                floor: None,
                cap: Some(Ratio::new(10.0).unwrap()),
            },
        )]);
        let mut fuel = 10_000;
        let err = eval_with("(* 100$ k)", bindings, &mut fuel).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::RatioOutsideDeclaredDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-041");
    }

    /// The mirror image: a capped Ratio AT or under its cap still multiplies
    /// cleanly — the check is `>`, not `>=`, matching every other
    /// closed-interval-at-the-top domain in this spec (e.g. `p`/`i`/`c`'s
    /// `[0,1]` accepts the endpoint) — `:cap` is INCLUSIVE.
    #[test]
    fn a_ratio_at_exactly_its_declared_cap_is_legal() {
        let bindings = HashMap::from([(
            "k".to_owned(),
            Value::Ratio {
                value: Ratio::new(10.0).unwrap(),
                floor: None,
                cap: Some(Ratio::new(10.0).unwrap()),
            },
        )]);
        let mut fuel = 10_000;
        assert_eq!(
            eval_with("(* 100$ k)", bindings, &mut fuel).unwrap(),
            Value::Currency(Currency::from_micro_units(1_000_000_000))
        );
    }

    /// `:floor` is EXCLUSIVE — matching `entropy_factor`'s own `> 1.0` and
    /// `Ratio`'s own open-at-zero law — so a value AT the floor is refused,
    /// the mirror image of `:cap`'s INCLUSIVE endpoint above.
    #[test]
    fn a_ratio_at_exactly_its_declared_floor_is_e_eval_041() {
        let bindings = HashMap::from([(
            "k".to_owned(),
            Value::Ratio {
                value: Ratio::new(1.0).unwrap(),
                floor: Some(Ratio::new(1.0).unwrap()),
                cap: None,
            },
        )]);
        let mut fuel = 10_000;
        let err = eval_with("(* 100$ k)", bindings, &mut fuel).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::RatioOutsideDeclaredDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-041");
    }

    /// A value strictly above its declared floor multiplies cleanly —
    /// `entropy_factor`'s exact worked shape: `(defconst
    /// metabolism/entropy-factor 1.5r :floor 1r :cap 3r)`, `1.5 > 1.0`.
    #[test]
    fn a_ratio_strictly_above_its_declared_floor_is_legal() {
        let bindings = HashMap::from([(
            "k".to_owned(),
            Value::Ratio {
                value: Ratio::new(1.5).unwrap(),
                floor: Some(Ratio::new(1.0).unwrap()),
                cap: Some(Ratio::new(3.0).unwrap()),
            },
        )]);
        let mut fuel = 10_000;
        assert_eq!(
            eval_with("(* 100$ k)", bindings, &mut fuel).unwrap(),
            Value::Currency(Currency::from_micro_units(150_000_000))
        );
    }

    /// A value AT the inclusive cap alongside a floor also multiplies
    /// cleanly — `entropy_factor`'s own upper endpoint, `3.0 <= 3.0`.
    #[test]
    fn a_ratio_at_its_declared_cap_alongside_a_floor_is_legal() {
        let bindings = HashMap::from([(
            "k".to_owned(),
            Value::Ratio {
                value: Ratio::new(3.0).unwrap(),
                floor: Some(Ratio::new(1.0).unwrap()),
                cap: Some(Ratio::new(3.0).unwrap()),
            },
        )]);
        let mut fuel = 10_000;
        assert_eq!(
            eval_with("(* 100$ k)", bindings, &mut fuel).unwrap(),
            Value::Currency(Currency::from_micro_units(300_000_000))
        );
    }

    /// Ratio has exactly one operator (Currency ×) — no addition, no
    /// comparison, no Ratio-Ratio arithmetic. "Scalar multiplication isn't
    /// new mathematics" (the ruling's own framing) — the evaluator must not
    /// quietly grow a second one.
    #[test]
    fn ratio_has_no_operator_but_the_currency_multiply() {
        for src in ["(+ 1 2r)", "(* 2r 3r)", "(< 1r 2r)", "(/ 10$ 2r)"] {
            let err = eval(src).expect_err(src);
            assert_eq!(err.code, None, "{src}: {err}");
        }
    }

    /// §3.2 (bsl-language.rst:849): "``Currency × Int`` [is a] type error;
    /// multiply by a ``Coefficient`` or divide by an ``Int`` instead" — at
    /// ANY value. `Int` must not slip through the promoting lane even where
    /// its f64 image would be a valid coefficient (0 and 1), in either
    /// operand order.
    #[test]
    fn currency_times_int_is_rejected_at_every_value() {
        for src in ["(* 1000$ 1)", "(* 1 1000$)", "(* 1000$ 0)", "(* 3 1000$)"] {
            let err = eval(src).expect_err(src);
            assert!(
                err.message.contains("E-TYPE-030"),
                "{src}: expected the §3.2 operator-table rejection, got: {}",
                err.message
            );
        }
    }

    #[test]
    fn currency_ratio_is_a_coefficient_with_both_domain_codes() {
        assert_eq!(eval("(/ 500$ 1000$)").unwrap(), Value::Real(0.5));
        let over = eval("(/ 1000$ 500$)").unwrap_err();
        assert_eq!(over.code, Some(EvalCode::CoefficientOutOfRange));
        let zero = eval("(/ 500$ 0$)").unwrap_err();
        assert_eq!(zero.code, Some(EvalCode::DivisionByZero));
    }

    #[test]
    fn currency_div_integer_is_half_even_and_rejects_nonpositive_divisors() {
        assert_eq!(
            eval("(/ 1000$ 4)").unwrap(),
            Value::Currency(Currency::from_micro_units(250_000_000))
        );
        let err = eval("(/ 1000$ -4)").unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DivisionByZero));
    }

    #[test]
    fn off_table_currency_mixes_are_loud_type_shaped_errors() {
        for source in ["(+ 100$ 1)", "(* 100$ 2)", "(* 100$ 200$)"] {
            let err = eval(source).unwrap_err();
            assert_eq!(err.code, None, "{source}");
            assert!(err.message.contains("E-TYPE-030"), "{source}: {err}");
        }
    }

    #[test]
    fn comparisons_stay_within_one_lane() {
        assert_eq!(eval("(< 900$ 1000.5$)").unwrap(), Value::Bool(true));
        assert_eq!(eval("(>= 3 3)").unwrap(), Value::Bool(true));
        assert_eq!(eval("(< 0 0.5c)").unwrap(), Value::Bool(true));
        let err = eval("(< 100$ 0.5c)").unwrap_err();
        assert_eq!(err.code, None);
    }

    #[test]
    fn enum_members_compare_with_equality_only_within_one_type() {
        assert_eq!(
            eval("(= NodeType/SOCIAL_CLASS NodeType/SOCIAL_CLASS)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval("(!= NodeType/SOCIAL_CLASS NodeType/TERRITORY)").unwrap(),
            Value::Bool(true)
        );
        assert!(eval("(= NodeType/SOCIAL_CLASS EdgeType/SOLIDARITY)").is_err());
        assert!(eval("(< NodeType/SOCIAL_CLASS NodeType/TERRITORY)").is_err());
    }

    #[test]
    fn references_compare_by_identity_within_their_own_kind() {
        use babylon_graph::substrate::{HyperedgeId, NodeId};

        let edge = EdgeKey {
            source: NodeId(1),
            target: NodeId(2),
            edge_type: "TENANCY".to_owned(),
        };
        let bindings = HashMap::from([
            ("node-a".to_owned(), Value::NodeRef(NodeId(1))),
            ("node-b".to_owned(), Value::NodeRef(NodeId(2))),
            ("edge-a".to_owned(), Value::EdgeRef(edge.clone())),
            ("edge-b".to_owned(), Value::EdgeRef(edge)),
            (
                "hyperedge-a".to_owned(),
                Value::HyperedgeRef(HyperedgeId(3)),
            ),
            (
                "hyperedge-b".to_owned(),
                Value::HyperedgeRef(HyperedgeId(3)),
            ),
        ]);

        for (source, expected) in [
            ("(= node-a node-a)", true),
            ("(!= node-a node-b)", true),
            ("(= edge-a edge-b)", true),
            ("(= hyperedge-a hyperedge-b)", true),
        ] {
            let mut fuel = 32;
            assert_eq!(
                eval_with(source, bindings.clone(), &mut fuel).unwrap(),
                Value::Bool(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn and_and_or_short_circuit_left_to_right() {
        // The unbound second operand would be a loud error if evaluated;
        // the short-circuit means it never is (§4.1's deliberate exception).
        assert_eq!(
            eval("(and (< 1 0) undeclared-var)").unwrap(),
            Value::Bool(false)
        );
        assert_eq!(
            eval("(or (< 0 1) undeclared-var)").unwrap(),
            Value::Bool(true)
        );
        let err = eval("(and)").unwrap_err();
        assert!(err.message.contains("E-PARSE-021"), "{err}");
    }

    #[test]
    fn if_evaluates_and_charges_only_the_taken_branch() {
        let mut fuel = 100;
        let result = eval_with("(if (< 0 1) 1 undeclared-var)", HashMap::new(), &mut fuel);
        assert_eq!(result.unwrap(), Value::Int(1));
        // if(1) + cmp(1) + two literals(0) + taken literal(0) = 2 consumed.
        assert_eq!(fuel, 98);
    }

    #[test]
    fn not_negates_bools_and_rejects_numbers() {
        assert_eq!(eval("(not #f)").unwrap(), Value::Bool(true));
        assert!(eval("(not 1)").is_err());
    }

    #[test]
    fn unbound_variables_are_loud_loader_bugs_never_defaults() {
        let err = eval("(+ 1 undeclared-var)").unwrap_err();
        assert!(err.message.contains("E-LOAD-010"), "{err}");
    }

    /// Task 2 (§2.6 chapter C8): `it` always denotes the element of the
    /// innermost enclosing iterating form, resolved through `EvalEnv`'s new
    /// element stack — NEVER through `env.bindings`, and never a stale
    /// read. `scope.rs::walk_names` already refuses a bare `it` at LOAD time
    /// (`E-TYPE-012`, `NameOutsideItsBody`); this is the same discipline
    /// held at evaluation, defense in depth, exactly as an unbound variable
    /// is (the test above).
    #[test]
    fn it_outside_any_iterating_form_is_loud() {
        let err = eval("it").unwrap_err();
        // The load-bearing assertions pin the ELEMENT-STACK refusal
        // specifically — `contains("it")` alone was vacuous (the verify
        // round showed the unbound-variable fall-through also contains
        // "it"), so the §2.6 citation and the stack wording carry the test.
        assert!(err.message.contains("§2.6"), "{err}");
        assert!(err.message.contains("element stack is empty"), "{err}");
    }

    /// Task 2: `EvalEnv.graph` is `None` for the pure-expression callers
    /// (every existing test in this module). A query-evaluating form
    /// reached with no graph is a loud DRIVER error — the caller forgot to
    /// supply one — never an empty set and never a `0`; `require_graph` is
    /// the one seam every future query head (Task 4+) will call through.
    ///
    /// `fold`/`nodes` themselves still refuse as Task 1's unserved-head seam
    /// (they are not dispatched to the graph yet — that lands with Task 4/5,
    /// a later PR), so this pins `require_graph` directly rather than
    /// through a full `(fold count (nodes NodeType/X) 1)` evaluation, which
    /// would only exercise Task 1's message. See the final report for the
    /// discrepancy this records against the plan's literal wording.
    #[test]
    fn a_query_with_no_graph_is_a_loud_driver_error() {
        let costs = costs();
        let env = EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        // `Result::unwrap_err` needs `T: Debug`; `&dyn GraphSubstrate` isn't
        // one, so the Ok arm is matched out by hand.
        let Err(err) = require_graph(&env, "fold") else {
            panic!("EvalEnv.graph is None; require_graph must refuse")
        };
        assert!(err.message.contains("driver"), "{err}");
        assert!(err.message.contains("fold"), "{err}");
        // Structural, not just textual: `require_graph` returns
        // `Result<&dyn GraphSubstrate, EvalError>` — there is no code path
        // by which its Err arm could be confused with a Value::Int(0) or a
        // Value::Real(0.0); the type itself makes "yields 0" unreachable.
    }

    #[test]
    fn fuel_exhaustion_at_the_exact_boundary_is_e_eval_040() {
        // (< wealth 1000.5$) consumes 2. A meter of 2 reaches zero on the
        // second charge — E-EVAL-040 by §4.5's "reaching or passing zero";
        // a meter of 3 stays strictly positive and succeeds.
        let mut starved = 2;
        let err = eval_with("(< wealth 1000.5$)", wealth_bindings(), &mut starved).unwrap_err();
        assert_eq!(err.code, Some(EvalCode::FuelExhausted));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-040");
        let mut enough = 3;
        assert!(eval_with("(< wealth 1000.5$)", wealth_bindings(), &mut enough).is_ok());
        assert_eq!(enough, 1);
    }

    #[test]
    fn intrinsic_calls_charge_declared_cost_and_cross_the_host_boundary() {
        struct Doubler;
        impl crate::intrinsic_host::IntrinsicHost for Doubler {
            fn call(
                &self,
                name: &str,
                args: &[Value],
                _ctx: crate::intrinsic_host::IntrinsicCallCtx<'_>,
            ) -> Result<Value, EvalError> {
                assert_eq!(name, "double");
                match args {
                    [Value::Int(n)] => Ok(Value::Int(n * 2)),
                    other => Err(EvalError::plain(format!("bad args {other:?}"))),
                }
            }
        }
        let costs = costs();
        let env = EvalEnv {
            bindings: HashMap::new(),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read("(double 5)").unwrap();
        let mut fuel = 100;
        let result = evaluate(&expr, &env, &Doubler, &mut fuel).unwrap();
        assert_eq!(result, Value::Int(10));
        // base(5) + declared(4) + literal arg(0) = 9 consumed.
        assert_eq!(fuel, 91);
        // The empty host fails loud on the same call.
        let mut fuel2 = 100;
        assert!(evaluate(&expr, &env, &EmptyIntrinsicHost, &mut fuel2).is_err());
        // An intrinsic with no declared cost is a loud loader-bug error.
        let err = eval("(sigmoid 0.5c)").unwrap_err();
        assert!(err.message.contains("E-LOAD-021"), "{err}");
    }

    /// End-to-end: `(floor x)` through the real evaluator and the real
    /// `KernelIntrinsicHost` (ADR188 Row 2), not a test double — proves the
    /// fuel-metered call boundary and the intrinsic's own domain check
    /// compose correctly.
    #[test]
    fn floor_call_evaluates_through_the_kernel_intrinsic_host() {
        let costs = IntrinsicCosts::new(HashMap::from([("floor".to_owned(), 3)]));
        let env = EvalEnv {
            bindings: HashMap::from([("x".to_owned(), Value::Real(7.8))]),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read("(floor x)").unwrap();
        let mut fuel = 100;
        let result = evaluate(
            &expr,
            &env,
            &crate::intrinsic_host::KernelIntrinsicHost,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Int(7));

        // A negative binding surfaces the intrinsic's own E-EVAL-039,
        // through the full evaluator, not just the unit-tested host.
        let neg_env = EvalEnv {
            bindings: HashMap::from([("x".to_owned(), Value::Real(-2.5))]),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let mut fuel2 = 100;
        let err = evaluate(
            &expr,
            &neg_env,
            &crate::intrinsic_host::KernelIntrinsicHost,
            &mut fuel2,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::DemotionOutOfDomain));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-039");
    }

    /// Task 1: the old `GRAPH_SEAM_HEADS` conflation reported EVERY one of
    /// these heads with the same "Task 16" misdiagnosis. After the split,
    /// each refusal names what it actually is — an effect-position-only
    /// verb/grouping form is a grammar error, never an unimplemented seam;
    /// an unserved query/selection/accessor form names the slice that will
    /// serve it. (The `update-edge` effect-position leg lived here until
    /// T3 served the head — see `update_edge_is_served_and_update_hyperedge_keeps_refusing`
    /// below, split out when this test outgrew the 100-line lint.)
    #[test]
    fn refusal_messages_name_their_slice() {
        // `emit` is EFFECT-position only (§2.8): in expression position it
        // is a grammar error, never "Task 16".
        let emit_err = eval("(emit EventType/RUPTURE (severity 0.9c))").unwrap_err();
        assert!(!emit_err.message.contains("Task 16"), "{emit_err}");
        assert!(emit_err.message.contains("§2.8"), "{emit_err}");
        assert!(emit_err.message.contains("effect position"), "{emit_err}");

        // `members-of` is SERVED at slice 3 (Community Task 3, 2026-08-21)
        // — in bare-expr position it now refuses with the
        // query-operand-only law (§2.7: no bare `<query>` production),
        // never a slice number: the served heads' refusal is grammatical,
        // not a TODO.
        let members_of_err = eval("(members-of self HyperedgeType/CELL)").unwrap_err();
        assert!(
            members_of_err.message.contains("query operand"),
            "{members_of_err}"
        );
        assert!(
            !members_of_err.message.contains("slice"),
            "{members_of_err}"
        );

        // `for-each` in expression position: a grammar error naming its
        // §2.8 home — and NEVER a claim that it is already served in effect
        // position (it refuses there too, pending Task 10; the verify round
        // caught the earlier message asserting service that does not exist).
        let for_each_err = eval("(for-each (nodes NodeType/X) (emit EventType/E))").unwrap_err();
        assert!(for_each_err.message.contains("§2.8"), "{for_each_err}");
        assert!(
            !for_each_err.message.contains("already served"),
            "the refusal must claim only the grammar, never service: {for_each_err}"
        );

        // `update-membership` (Amendment-AG-era §2.8 verb): same grammar
        // refusal, not E-LOAD-021 "undeclared intrinsic".
        let upd_mem_err = eval("(update-membership self self (set x/y 1))").unwrap_err();
        assert!(upd_mem_err.message.contains("§2.8"), "{upd_mem_err}");
        assert!(!upd_mem_err.message.contains("E-LOAD-021"), "{upd_mem_err}");

        // `membership-field-of` (§2.10) stays unserved — its note now cites
        // #653 by number, never a slice (Task 3 Step 4's own amendment).
        let mem_field_err = eval("(membership-field-of self self x/y)").unwrap_err();
        assert!(mem_field_err.message.contains("#653"), "{mem_field_err}");
    }

    /// The effect-position half of the flip (T3, ADR198 R1-R3, issue #560):
    /// `refusal_messages_name_their_slice`'s `update-edge` leg used to pin
    /// the storage refusal ("Constitution III.7"); serving a
    /// previously-refused head flips the assertion (the T2 plan's own
    /// enumerate-and-flip discipline). What is pinned now: a well-formed
    /// update-edge WRITES through the execute path, and — since Community
    /// port train Task 6 (E2b) discharged the III.7 storage gap via
    /// `GraphSubstrate::update_hyperedge_attribute` — `update-hyperedge`
    /// writes too, through the SAME execute path.
    #[test]
    fn update_edge_and_update_hyperedge_are_both_served() {
        use crate::structural_verbs::{CollectingSink, EffectExecutor};
        use crate::typecheck::TypeEnv;
        use crate::types::EnumRegistry;
        use babylon_graph::memory::MemoryGraph;
        use babylon_graph::substrate::GraphSubstrate;

        let mut graph = MemoryGraph::new();
        let self_id = graph.add_node("SOCIAL_CLASS").unwrap();
        let other_id = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .add_edge("SOLIDARITY", self_id, other_id, 0.5)
            .unwrap();
        let cell = graph
            .add_hyperedge("COMMUNITY", &[self_id, other_id])
            .unwrap();
        let types = TypeEnv {
            fields: HashMap::from([
                (
                    "solidarity/strength".to_owned(),
                    crate::types::FieldDecl {
                        ty: crate::types::BslType::Coefficient,
                        kind: crate::types::FieldKind::Extensive,
                    },
                ),
                (
                    "community/heat".to_owned(),
                    crate::types::FieldDecl {
                        ty: crate::types::BslType::Coefficient,
                        kind: crate::types::FieldKind::Intensive,
                    },
                ),
            ]),
            exemptions: &[],
        };
        let enums = EnumRegistry::default();
        let edge_key = crate::query::EdgeKey {
            source: self_id,
            target: other_id,
            edge_type: "SOLIDARITY".to_owned(),
        };
        let mut executor = EffectExecutor::new(&types, &enums, None);
        let mut sink = CollectingSink::default();
        let costs = costs();
        let effect_env = EvalEnv {
            bindings: HashMap::from([
                ("e".to_owned(), Value::EdgeRef(edge_key)),
                ("h".to_owned(), Value::HyperedgeRef(cell)),
            ]),
            intrinsic_costs: &costs,
            graph: None,
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (form, _) =
            read("(effects (update-edge e solidarity/strength (scale 0.5c)))").expect("must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        executor
            .execute_effects(
                &items[1..],
                &effect_env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .expect("update-edge is served since T3");
        let stored = graph
            .edge_attribute("SOLIDARITY", self_id, other_id, "solidarity/strength")
            .unwrap();
        assert!(
            (stored - 0.25).abs() < 1e-12,
            "0.5 scaled by 0.5, stored: {stored}"
        );

        // The hyperedge half — refused with a III.7 "no substrate storage"
        // message before Task 6, a plain write now. `set` never reads the
        // prior value, so the never-written field is no obstacle.
        let (form, _) =
            read("(effects (update-hyperedge h community/heat (set 0.5c)))").expect("must parse");
        let SExpr::List(items) = form else {
            unreachable!()
        };
        let mut fuel = 128;
        executor
            .execute_effects(
                &items[1..],
                &effect_env,
                &EmptyIntrinsicHost,
                &mut graph,
                &mut sink,
                &mut fuel,
            )
            .expect("update-hyperedge is served since Task 6 (E2b)");
        let stored = graph.hyperedge_attribute(cell, "community/heat").unwrap();
        assert!((stored - 0.5).abs() < 1e-12, "set to 0.5, stored: {stored}");
    }

    /// The sentinel Task 1 exists to install: a form the language HAS can
    /// never fall through to `eval_intrinsic` and report the wrong
    /// diagnosis (`E-LOAD-021`, undeclared intrinsic). Proven MECHANICALLY,
    /// not against a hard-coded historical list (the verify round showed the
    /// original 27-head copy could not catch an unclassified grammar head):
    /// every entry of `declarations::RESERVED_FORM_TAGS` partitions into
    /// exactly one of {evaluator-served, effect-position-only, unserved-
    /// expression, declaration-level}, with no remainder — plus the three
    /// Amendment-AG-era heads that predate their own `RESERVED_FORM_TAGS`
    /// rows (a filed reservation-gap follow-up).
    #[test]
    fn every_seam_head_is_classified() {
        const EVALUATOR_SERVED: [&str; 12] = [
            "and",
            "or",
            "not",
            "if",
            "field-of",
            "edge-between",
            "fold",
            "exists",
            "forall",
            "select-max",
            "select-min",
            "quantize-mass",
        ];
        // Tags that are declaration/top-form/clause vocabulary, never
        // expression-position heads — the load layer owns them.
        const DECLARATION_LEVEL: [&str; 14] = [
            "anchor",
            "binding",
            "bindings",
            "branch",
            "ceiling",
            "deffield",
            "domain",
            "effects",
            "intrinsic",
            "manifest",
            "metric",
            "opt",
            "rule",
            "when",
        ];
        let mut unclassified = Vec::new();
        for tag in crate::declarations::RESERVED_FORM_TAGS {
            let buckets = [
                EVALUATOR_SERVED.contains(&tag),
                EFFECT_POSITION_ONLY.contains(&tag),
                UNSERVED_EXPRESSION_HEADS.iter().any(|(h, _)| *h == tag),
                DECLARATION_LEVEL.contains(&tag),
                SERVED_QUERY_HEADS.contains(&tag),
            ];
            match buckets.iter().filter(|b| **b).count() {
                1 => {}
                0 => unclassified.push(tag),
                n => panic!("{tag} appears in {n} buckets — the partition must be exclusive"),
            }
        }
        assert!(
            unclassified.is_empty(),
            "unclassified RESERVED_FORM_TAGS (each would misdiagnose as \
             E-LOAD-021): {unclassified:?}"
        );
        // The AG-era heads are absent from RESERVED_FORM_TAGS (filed gap)
        // but MUST still be classified here so they refuse with the right
        // diagnosis today.
        for head in ["update-membership", "member", "membership-field-of"] {
            let in_effect_only = EFFECT_POSITION_ONLY.contains(&head);
            let in_unserved = UNSERVED_EXPRESSION_HEADS.iter().any(|(h, _)| *h == head);
            assert!(
                in_effect_only ^ in_unserved,
                "AG-era head {head} must be classified in exactly one table"
            );
        }
    }

    /// Task 4: `nodes`/`neighbors` are SERVED (they moved out of
    /// `UNSERVED_EXPRESSION_HEADS`), but only as an iterating form's query
    /// operand — never as a bare `<expr>`. Reaching one here is a shape
    /// error, not "lands with slice 1" (that claim would now be false: it
    /// already has).
    #[test]
    fn bare_query_heads_are_a_shape_error_not_an_unserved_slice_claim() {
        for source in [
            "(nodes NodeType/SOCIAL_CLASS)",
            "(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)",
        ] {
            let err = eval(source).unwrap_err();
            assert!(!err.message.contains("slice 1"), "{source}: {err}");
            assert!(
                err.message.contains("no <expr> production"),
                "{source}: {err}"
            );
        }
    }

    // ---- Task 8: field-of ----

    /// No test driven through here declares an enum-typed field — an empty
    /// `TypeEnv`/`EnumRegistry` pair is the honest "no `defenum`s in scope"
    /// input, and `tick::bind_field_value`'s own "unregistered field:
    /// unchanged behavior" precedent means every qname these callers name
    /// still renders `Value::Real`, byte-identical to this helper's
    /// pre-verifier-fix-round behavior. **Threading `Some(..)` here (PR A
    /// verifier fix round, 2026-08-12) is not optional dressing**:
    /// `field_of_node` now REFUSES loudly on `None` (mirroring
    /// `require_graph`), so a bare `types: None, enums: None` `EvalEnv` would
    /// make every `field-of` vector below a driver-error test instead of
    /// the field-read test it claims to be — delegates to
    /// [`eval_field_of_over_typed`] rather than duplicating the `EvalEnv`
    /// literal a second time.
    fn eval_field_of_over(
        source: &str,
        graph: &dyn babylon_graph::substrate::GraphSubstrate,
        subject: babylon_graph::substrate::NodeId,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let types = TypeEnv {
            fields: HashMap::new(),
            exemptions: &[],
        };
        let enums = EnumRegistry::default();
        eval_field_of_over_typed(source, graph, subject, &types, &enums, fuel)
    }

    /// [`eval_field_of_over`]'s D102-discharge sibling: the SAME shape,
    /// but with `types`/`enums` actually threaded (`Some`), so `field-of`
    /// over an `:enum-type`-declared field renders `Value::Enum` instead
    /// of falling back to the pre-D102 `Value::Real`.
    fn eval_field_of_over_typed(
        source: &str,
        graph: &dyn babylon_graph::substrate::GraphSubstrate,
        subject: babylon_graph::substrate::NodeId,
        types: &TypeEnv,
        enums: &EnumRegistry,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let costs = costs();
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(subject))]),
            intrinsic_costs: &costs,
            graph: Some(graph),
            types: Some(types),
            enums: Some(enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read(source).expect("test source must parse");
        evaluate(&expr, &env, &EmptyIntrinsicHost, fuel)
    }

    /// `OrgKind` in declaration order: `STATE_APPARATUS`=0, `BUSINESS`=1 —
    /// the same fixture shape `tick.rs::org_kind_fixture` uses, built
    /// standalone here since this module's tests drive `evaluate`
    /// directly rather than through the full rule pipeline.
    fn org_kind_types_and_enums() -> (TypeEnv, EnumRegistry) {
        use crate::types::{BslType, FieldDecl, FieldKind};
        let mut enums = EnumRegistry::default();
        let ty = enums
            .declare(
                "OrgKind",
                &["STATE_APPARATUS".to_owned(), "BUSINESS".to_owned()],
            )
            .unwrap();
        let types = TypeEnv {
            fields: HashMap::from([(
                "organization/kind".to_owned(),
                FieldDecl {
                    ty: BslType::Enum(ty),
                    kind: FieldKind::NotApplicable,
                },
            )]),
            exemptions: &[],
        };
        (types, enums)
    }

    /// D102 discharge (Task 1, P27 territory-port train): `field-of` over
    /// an enum-declared field renders `Value::Enum` — the SAME rendering
    /// `tick::bind_field_value` gives a `:field` binding's read of the
    /// identical field — and a `=` comparison against an enum-ref member
    /// literal returns the right `Bool` per node, not a `Value::Real`
    /// comparison that would silently compare ordinals as magnitudes.
    #[test]
    fn field_of_over_an_enum_field_compares_correctly_per_node() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let state_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(state_org, "organization/kind", 0.0)
            .unwrap();
        let biz_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(biz_org, "organization/kind", 1.0)
            .unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 1_000;

        let state_result = eval_field_of_over_typed(
            "(= (field-of self organization/kind) OrgKind/STATE_APPARATUS)",
            &graph,
            state_org,
            &types,
            &enums,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(state_result, Value::Bool(true));

        let biz_result = eval_field_of_over_typed(
            "(= (field-of self organization/kind) OrgKind/STATE_APPARATUS)",
            &graph,
            biz_org,
            &types,
            &enums,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(biz_result, Value::Bool(false));
    }

    /// §3.1: `Enum<T>` compares only to the same enum type — unaffected by
    /// D102's discharge, whether the `Value::Enum` came from a `:field`
    /// binding or (now) `field-of`. `NodeType/BUSINESS` is a real enum-ref
    /// of the WRONG declared type, deliberately sharing a member NAME with
    /// `OrgKind/BUSINESS` (`apply_equality`'s own doc precedent, `c268b83b`
    /// / `structural_verbs.rs`'s cross-type write test) so a mutant that
    /// skipped the type check and fell through to the member comparison
    /// would find a plausible-looking match instead of an unrelated error.
    #[test]
    fn field_of_over_an_enum_field_cross_enum_type_comparison_still_errors() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let biz_org = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(biz_org, "organization/kind", 1.0)
            .unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 1_000;
        let err = eval_field_of_over_typed(
            "(= (field-of self organization/kind) NodeType/BUSINESS)",
            &graph,
            biz_org,
            &types,
            &enums,
            &mut fuel,
        )
        .unwrap_err();
        assert!(
            err.message.contains("compares only to the same enum type"),
            "{}",
            err.message
        );
    }

    /// §2.13's no-arithmetic law (D101) STANDS: `apply_arith` refuses
    /// `Value::Enum` unconditionally, whether the value came from a
    /// `:field` binding or (now, post-D102) `field-of` — D102's discharge
    /// widened where `field-of` may legally appear, not what Enum<T> may
    /// legally do once evaluated.
    #[test]
    fn field_of_over_an_enum_field_still_refuses_arithmetic() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 0.0).unwrap();
        let (types, enums) = org_kind_types_and_enums();
        let mut fuel = 1_000;
        let err = eval_field_of_over_typed(
            "(+ (field-of self organization/kind) 5)",
            &graph,
            org,
            &types,
            &enums,
            &mut fuel,
        )
        .unwrap_err();
        assert!(
            err.message.contains("no arithmetic is defined"),
            "{}",
            err.message
        );
    }

    /// PR A verifier fix round (2026-08-12): `field_of_node` REFUSES
    /// loudly, mirroring `require_graph`'s own driver-error shape, when an
    /// `EvalEnv` carries a live graph but no declared-type registry — the
    /// EXACT shape `rule_pipeline::resolve_expr_bindings` used to build
    /// unconditionally (safe there only by the coincidence that it also
    /// passed `graph: None`, so `require_graph` refused first). Before
    /// this fix this same construction silently rendered `Value::Real`
    /// off the raw stored ordinal — never surfacing that the field was
    /// enum-declared at all.
    #[test]
    fn field_of_over_an_enum_field_with_no_type_registry_is_a_loud_driver_error() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let org = graph.add_node("ORGANIZATION").unwrap();
        graph.update_node(org, "organization/kind", 1.0).unwrap();
        let costs = costs();
        let env = EvalEnv {
            bindings: HashMap::from([("self".to_owned(), Value::NodeRef(org))]),
            intrinsic_costs: &costs,
            graph: Some(&graph),
            types: None,
            enums: None,
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read("(field-of self organization/kind)").expect("must parse");
        let mut fuel = 1_000;
        let err = evaluate(&expr, &env, &EmptyIntrinsicHost, &mut fuel).unwrap_err();
        assert!(
            err.message.contains("declared field-type registry"),
            "{}",
            err.message
        );
        assert!(err.message.contains("organization/kind"), "{}", err.message);
        assert!(
            err.message.contains("driver error"),
            "must name itself a driver error, the same shape as require_graph: {}",
            err.message
        );
    }

    #[test]
    fn field_of_reads_a_declared_field_of_the_referent() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(subject, "social-class/wealth", 42.5)
            .unwrap();
        let mut fuel = 1_000;
        let result = eval_field_of_over(
            "(field-of self social-class/wealth)",
            &graph,
            subject,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Real(42.5));
    }

    /// §2.10 discipline 1: the qname's first segment names the owning type
    /// (`social-class` → `SOCIAL_CLASS`); reading it off a `TERRITORY` ref is
    /// `E-EVAL-033`, never a default and never an absent read.
    #[test]
    fn field_of_whose_referent_is_of_another_type_is_e_eval_033() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let territory = graph.add_node("TERRITORY").unwrap();
        let mut fuel = 1_000;
        let err = eval_field_of_over(
            "(field-of self social-class/wealth)",
            &graph,
            territory,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::AccessorTypeOrValueMismatch));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-033");
    }

    /// §2.10 discipline 2: absence is not a value — an element of the RIGHT
    /// type that simply never had the field written is ALSO `E-EVAL-033`,
    /// never a fabricated `0.0`.
    #[test]
    fn field_of_a_field_the_element_carries_no_value_for_is_e_eval_033() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let mut fuel = 1_000;
        let err = eval_field_of_over(
            "(field-of self social-class/wealth)",
            &graph,
            subject,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::AccessorTypeOrValueMismatch));
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-033");
    }

    /// `bound_checker`'s own pinned figure: `cost_of("(field-of it \
    /// solidarity/strength)") == 2` — accessor base(1) + `it`'s variable-ref
    /// (1). A keyed lookup, never multiplied by a ceiling.
    #[test]
    fn field_of_is_charged_as_a_keyed_lookup_not_an_iteration() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        graph
            .update_node(subject, "social-class/wealth", 1.0)
            .unwrap();
        let mut fuel = 10;
        eval_field_of_over(
            "(field-of self social-class/wealth)",
            &graph,
            subject,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            fuel, 8,
            ":fuel-used is a conformance-vector quantity (§6.1)"
        );
    }

    // ---- T2 (issue #559): edge-between ----

    /// Evaluate `source` against a graph with `self`/`other` bound to the
    /// two supplied nodes — the shared fixture for the `edge-between`
    /// vectors (and Task 5's `field-of`-over-an-`EdgeRef` vectors, which
    /// compose onto the same two-node shape). Empty-but-`Some`
    /// `TypeEnv`/`EnumRegistry`, per this module's own standing pattern
    /// (`eval_field_of_over`'s doc).
    fn eval_with_pair(
        source: &str,
        graph: &dyn babylon_graph::substrate::GraphSubstrate,
        self_id: babylon_graph::substrate::NodeId,
        other_id: babylon_graph::substrate::NodeId,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let costs = costs();
        let types = TypeEnv {
            fields: HashMap::new(),
            exemptions: &[],
        };
        let enums = EnumRegistry::default();
        let env = EvalEnv {
            bindings: HashMap::from([
                ("self".to_owned(), Value::NodeRef(self_id)),
                ("other".to_owned(), Value::NodeRef(other_id)),
            ]),
            intrinsic_costs: &costs,
            graph: Some(graph),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read(source).expect("test source must parse");
        evaluate(&expr, &env, &EmptyIntrinsicHost, fuel)
    }

    #[test]
    fn edge_between_resolves_to_an_edge_ref() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 0.5).unwrap();
        let mut fuel = 1_000;
        let result = eval_with_pair(
            "(edge-between EdgeType/SOLIDARITY self other)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            result,
            Value::EdgeRef(EdgeKey {
                source: subject,
                target: other,
                edge_type: "SOLIDARITY".to_owned(),
            })
        );
    }

    #[test]
    fn edge_between_on_a_missing_pair_is_e_eval_034() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        let mut fuel = 1_000;
        let err = eval_with_pair(
            "(edge-between EdgeType/SOLIDARITY self other)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, Some(EvalCode::NoSuchEdge), "{err}");
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-034");
    }

    /// The static/runtime fuel agreement (§4.5's loud-failure-inversion
    /// discipline): `r9_chapters.rs::edge_between_costs_one_plus_its_two_
    /// endpoint_operands` pins the STATIC bound at
    /// `cost("(edge-between EdgeType/SOLIDARITY self other)") == 3`
    /// (`ACCESSOR_BASE(1) + self(1) + other(1)`); the runtime meter must
    /// charge the SAME 3 for the identical shape — the same discipline
    /// `query.rs::query_materialization_charges_the_3_7_query_base` already
    /// proves for `nodes`/`neighbors`.
    #[test]
    fn edge_between_charges_the_static_bounds_own_three() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 0.5).unwrap();
        let mut fuel = 10;
        eval_with_pair(
            "(edge-between EdgeType/SOLIDARITY self other)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            fuel, 7,
            ":fuel-used is a conformance-vector quantity (§6.1) — the \
             static bound and the runtime meter must agree"
        );
    }

    // ---- T2 (issue #559): field-of over an EdgeRef ----

    #[test]
    fn field_of_an_edge_ref_reads_the_seeded_strength() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 0.5).unwrap();
        let mut fuel = 1_000;
        let result = eval_with_pair(
            "(field-of (edge-between EdgeType/SOLIDARITY self other) solidarity/strength)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Real(0.5));
    }

    /// §2.10 discipline 1, the `EdgeRef` half: a qname owned by a DIFFERENT
    /// type than the referent's own (here a NodeType-owned qname against a
    /// SOLIDARITY edge) is `E-EVAL-033`, never a default and never an
    /// absent read.
    #[test]
    fn field_of_an_edge_ref_with_a_wrong_owner_qname_is_e_eval_033() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 0.5).unwrap();
        let mut fuel = 1_000;
        let err = eval_with_pair(
            "(field-of (edge-between EdgeType/SOLIDARITY self other) social-class/wealth)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(
            err.code,
            Some(EvalCode::AccessorTypeOrValueMismatch),
            "{err}"
        );
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-033");
    }

    /// A LEGALLY-typed but never-written edge qname (a `deffield`-declarable
    /// `solidarity/tension`, no storage behind it until T3 — ADR198 R1) is
    /// ALSO `E-EVAL-033` — Task 1's "not yet stored" branch degrades
    /// identically to a never-written field, with zero special-casing.
    #[test]
    fn field_of_an_edge_ref_with_an_unstored_qname_is_e_eval_033() {
        use babylon_graph::memory::MemoryGraph;
        let mut graph = MemoryGraph::new();
        let subject = graph.add_node("SOCIAL_CLASS").unwrap();
        let other = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.add_edge("SOLIDARITY", subject, other, 0.5).unwrap();
        let mut fuel = 1_000;
        let err = eval_with_pair(
            "(field-of (edge-between EdgeType/SOLIDARITY self other) solidarity/tension)",
            &graph,
            subject,
            other,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(
            err.code,
            Some(EvalCode::AccessorTypeOrValueMismatch),
            "{err}"
        );
        assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-033");
    }

    // ---- Task 5: fold ----

    /// Evaluate `source` against a graph, with a `self` binding pointing at
    /// `subject` (when one is supplied) — the shared fixture every fold/
    /// exists/forall/select-*/field-of test below builds on.
    ///
    /// An empty `TypeEnv`/`EnumRegistry` pair, threaded `Some` (PR A
    /// verifier fix round, 2026-08-12): none of this section's fold/
    /// selection/field-of vectors declare an enum-typed field, so every
    /// qname stays "unregistered" and renders `Value::Real` exactly as
    /// before — but `field_of_node` now REFUSES on `None` (mirroring
    /// `require_graph`), so a bare `None` pair would turn every `field-of`
    /// vector this helper drives into a driver-error test instead of the
    /// field-read test it claims to be.
    fn eval_over(
        source: &str,
        graph: &dyn babylon_graph::substrate::GraphSubstrate,
        subject: Option<babylon_graph::substrate::NodeId>,
        fuel: &mut u64,
    ) -> Result<Value, EvalError> {
        let costs = costs();
        let bindings = match subject {
            Some(id) => HashMap::from([("self".to_owned(), Value::NodeRef(id))]),
            None => HashMap::new(),
        };
        let types = TypeEnv {
            fields: HashMap::new(),
            exemptions: &[],
        };
        let enums = EnumRegistry::default();
        let env = EvalEnv {
            bindings,
            intrinsic_costs: &costs,
            graph: Some(graph),
            types: Some(&types),
            enums: Some(&enums),
            elements: Vec::new(),
            draw_context: None,
        };
        let (expr, _) = read(source).expect("test source must parse");
        evaluate(&expr, &env, &EmptyIntrinsicHost, fuel)
    }

    /// A graph of `n` `SOCIAL_CLASS` nodes, each carrying
    /// `social-class/wealth` = its index (as `Real`, 0.0, 1.0, 2.0, …) — the
    /// shared fixture for fold-body reads via `field-of`.
    fn wealth_ladder(n: u32) -> babylon_graph::memory::MemoryGraph {
        use babylon_graph::substrate::GraphSubstrate;
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        for i in 0..n {
            let id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph
                .update_node(id, "social-class/wealth", f64::from(i))
                .unwrap();
        }
        graph
    }

    // The §4.4 empty-set table.

    #[test]
    fn mean_min_max_over_an_empty_query_are_e_eval_021() {
        let graph = wealth_ladder(0);
        for op in ["mean", "min", "max"] {
            let mut fuel = 1_000;
            let err = eval_over(
                &format!(
                    "(fold {op} (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))"
                ),
                &graph,
                None,
                &mut fuel,
            )
            .unwrap_err();
            assert_eq!(err.code, Some(EvalCode::EmptyAggregate), "{op}: {err}");
            assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-021");
        }
    }

    #[test]
    fn sum_over_an_empty_query_is_the_additive_identity_of_the_body_type() {
        let graph = wealth_ladder(0);
        let mut fuel = 1_000;
        // The body is a `field-of` read — always `Real` (all node-attribute
        // storage is the binary64 lane) — so the identity is `Real(0.0)`.
        let result = eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Real(0.0));

        // An Int literal body: identity is Int(0).
        let mut fuel2 = 1_000;
        let result2 = eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) 5)",
            &graph,
            None,
            &mut fuel2,
        )
        .unwrap();
        assert_eq!(result2, Value::Int(0));
    }

    /// P3: `static_additive_identity` deliberately recognizes only literals,
    /// `field-of` reads and homogeneous arithmetic over them (its own doc
    /// comment) — it does NOT attempt a nested `fold` or a bare
    /// binding-symbol body, both load-legal §2.7 shapes. §4.4 gives `sum`
    /// over an empty query the body type's additive identity, but that is
    /// only SERVABLE where the identity is statically classifiable; an
    /// unclassifiable body refuses loudly, citing D-row Q12, rather than
    /// guessing or having this fix speculatively widen the classifier.
    #[test]
    fn sum_over_an_empty_query_with_an_unclassifiable_body_refuses_citing_the_d_row() {
        let empty = wealth_ladder(0);
        for body_src in [
            "it", // a bare binding-symbol body
            "(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))", // a nested fold
        ] {
            let mut fuel = 1_000;
            let err = eval_over(
                &format!("(fold sum (nodes NodeType/SOCIAL_CLASS) {body_src})"),
                &empty,
                None,
                &mut fuel,
            )
            .unwrap_err();
            assert!(err.code.is_none(), "{body_src}: {err}");
            assert!(err.message.contains("D-row Q12"), "{body_src}: {err}");
        }
    }

    /// P7: `static_additive_identity` must not classify `(* Currency
    /// Currency)` as `Currency(0)` — that runtime operation is illegal
    /// (`arith_currency`'s `*` arm: "Currency × Currency is E-TYPE-030 (an
    /// area of money)"). Unreachable via the loader today (defense in
    /// depth) — the classifier's own discipline is "recognizes … and
    /// deliberately no more", so this tightens rather than widens it.
    #[test]
    fn static_additive_identity_refuses_currency_times_currency() {
        let (expr, _) = read("(* 5$ 3$)").unwrap();
        assert_eq!(static_additive_identity(&expr), None);
    }

    #[test]
    fn count_over_an_empty_query_is_zero() {
        let graph = wealth_ladder(0);
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold count (nodes NodeType/SOCIAL_CLASS) it)",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Int(0));
    }

    /// P4: `count` is CARDINALITY (§3.4 row 6) — its result does not depend
    /// on the body's VALUE, so it must not evaluate the body per element.
    /// Before this fix `fold_count` called `eval_body_and_weight` for every
    /// element (fuel-fidelity reasoning that predates §3.4 row 6's reading),
    /// so a body reading a field an element never wrote aborted the count
    /// with `E-EVAL-033` even though count owed that element nothing.
    #[test]
    fn fold_count_does_not_evaluate_the_body_so_an_unwritten_field_does_not_abort_it() {
        let graph = wealth_ladder(3); // writes social-class/wealth only
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold count (nodes NodeType/SOCIAL_CLASS) \
             (field-of it social-class/head-count))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    /// Constraint 3: the binary64 lane is not associative, so a fold's
    /// reduction order is its ITERATION order — pinned as exact bits, not a
    /// convention. `(1e16, 1.0, -1e16)` in ascending-id iteration order
    /// gives `((1e16 + 1.0) + -1e16) = 0.0` (the `+1.0` is lost to rounding
    /// at that magnitude), where the opposite association
    /// `(1e16 + (1.0 + -1e16)) = 1.0` would NOT be — the textbook
    /// non-associativity example.
    #[test]
    fn fold_reduces_in_iteration_order_and_the_order_is_observable() {
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        for value in [1.0e16, 1.0, -1.0e16] {
            let id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph.update_node(id, "social-class/wealth", value).unwrap();
        }
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            result,
            Value::Real(0.0),
            "ascending-id iteration order: (1e16 + 1.0) + -1e16 == 0.0, not 1.0"
        );
    }

    /// CT4P A2 (issue #525). The correction to four independent reader
    /// suggestions that `sum`/`count` are monoid homomorphisms and
    /// partition-invariant (`sum(A ∪ B) == sum(A) + sum(B)`): **that law is
    /// FALSE here**, on purpose — `fold sum` reduces binary64 strictly
    /// left-to-right in ascending-id order (`fold_sum`), and IEEE-754 `+` is
    /// not associative, so a different GROUPING of the same elements can
    /// produce a different bit pattern. The classic three-decade witness:
    /// `1e16 + 1.0 + 1.0` reassociates to a different double, because
    /// `1e16 + 1.0` alone rounds away (the ULP near `1e16` is `2.0`), while
    /// `1.0 + 1.0 = 2.0` is exact and survives the second addition.
    ///
    /// This test asserts BOTH halves of the law: the fold's result equals
    /// the LEFT fold in ascending-id order (positive), and it does NOT
    /// equal a reassociated (chunked) fold over the exact same multiset
    /// (negative) — proving the witness is genuinely non-associative before
    /// trusting the "equals the left fold" assertion means anything.
    ///
    /// Mutation evidence: reassociating `fold_sum`'s accumulator (grouping
    /// the last two elements before combining with the first) flips this
    /// test red; restoring left-to-right accumulation is byte-identical
    /// with `git diff` empty. Recorded in the commit body rather than
    /// re-run here, to keep the shipped test itself a pure oracle.
    #[test]
    fn fold_sum_is_the_left_fold_and_is_not_partition_invariant() {
        let values = [1.0e16_f64, 1.0, 1.0];
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        for value in values {
            let id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph.update_node(id, "social-class/wealth", value).unwrap();
        }
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();

        // The LEFT fold in ascending-id order, computed independently of
        // `fold_sum`'s own implementation.
        let left_fold = values[1..].iter().fold(values[0], |acc, &v| acc + v);
        // A reassociated (chunked) fold over the SAME multiset, same source
        // order, different GROUPING: `v0 + (v1 + v2)` instead of
        // `(v0 + v1) + v2`.
        let chunked = values[0] + (values[1] + values[2]);
        assert_ne!(
            left_fold.to_bits(),
            chunked.to_bits(),
            "the witness must be genuinely non-associative, or this test proves nothing"
        );

        assert_eq!(
            result,
            Value::Real(left_fold),
            "fold sum must equal the LEFT fold in ascending-id order"
        );
        assert_ne!(
            result,
            Value::Real(chunked),
            "fold sum must NOT equal the reassociated (chunked) sum — \
             partition invariance is false in the binary64 lane"
        );
    }

    /// CT4P A2's mirror for `fold_mean`'s `Σ(wᵢ·xᵢ)` accumulation
    /// (evaluator.rs's D-row Q5 comment states the discipline in prose;
    /// this is the test that was missing). Weight = 1.0 for every element,
    /// so `sum_wx` reduces through the SAME non-associative family A2 pins
    /// for plain `sum`, and `sum_w` (= 3.0 exactly, three unit weights) adds
    /// no rounding of its own.
    #[test]
    fn fold_mean_sum_wx_is_the_left_fold_and_is_not_partition_invariant() {
        let bodies = [1.0e16_f64, 1.0, 1.0];
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        for value in bodies {
            let id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph.update_node(id, "social-class/wealth", value).unwrap();
            graph
                .update_node(id, "social-class/head-count", 1.0)
                .unwrap();
        }
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold mean (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth) \
             :weight (field-of it social-class/head-count))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();

        let left_sum_wx = bodies[1..].iter().fold(bodies[0], |acc, &v| acc + v);
        let chunked_sum_wx = bodies[0] + (bodies[1] + bodies[2]);
        let sum_w = 3.0_f64; // three unit weights — exact, no rounding
        let expected = left_sum_wx / sum_w;
        let reassociated = chunked_sum_wx / sum_w;
        // Non-vacuity guard (verifier fix round, NOTE-2): assert on the
        // QUOTIENTS the test actually compares `result` against below, not
        // a pre-division proxy — division by the SAME nonzero `sum_w` is
        // injective, so a differing numerator implies a differing quotient
        // here, but asserting the quotient directly is what the rest of
        // this test depends on, and is what a future edit to `sum_w`
        // (e.g. a non-uniform weight set) would otherwise silently stop
        // covering.
        assert_ne!(
            expected.to_bits(),
            reassociated.to_bits(),
            "the witness must be genuinely non-associative, or this test proves nothing"
        );

        assert_eq!(
            result,
            Value::Real(expected),
            "fold mean's sum_wx must reduce as the LEFT fold in ascending-id order"
        );
        assert_ne!(
            result,
            Value::Real(reassociated),
            "fold mean must NOT match the reassociated sum_wx — partition \
             invariance is false here too"
        );
    }

    /// CT4P A4 (issue #525): the deliberate ASYMMETRY with A2. Unlike `sum`,
    /// `min`/`max` genuinely ARE associative, commutative and idempotent
    /// over the live domain — non-finites are already excluded elsewhere
    /// (`EvalCode::NonFinite`), so nothing in the reachable input space can
    /// break the semilattice laws. This test pins that: the SAME multiset
    /// in two different element orders folds to the SAME min/max, and a
    /// duplicated element changes nothing. Paired in the same module as A2
    /// on purpose — naming which fold family reorders safely (this one) and
    /// which does not (`sum`/`mean`, A2) is the whole point; reading only
    /// one half would invite over-generalising A2's negative law into
    /// "never touch any fold's order," which is false for this family.
    ///
    /// Mutation evidence: in `fold_min_max`, discarded the `<`/`>` outcome
    /// and hardcoded `strictly_better = true` — every accumulation step then
    /// keeps whichever element it saw LAST, making the fold's result depend
    /// on iteration/insertion order instead of on the values. Both this
    /// test AND the pre-existing `fold_min_and_max_extremise_the_body_value`
    /// flipped red (min/max of `[3.0, 1.0, 2.0]` in that order becomes
    /// `2.0` for BOTH ops instead of `1.0`/`3.0`, and `graph_a`'s
    /// last-inserted value differs from `graph_b`'s, breaking order
    /// invariance directly). Reverted; `git diff` empty.
    #[test]
    fn fold_min_max_are_order_invariant_and_idempotent_under_duplication() {
        let mut graph_a = babylon_graph::memory::MemoryGraph::new();
        for value in [3.0, 1.0, 2.0] {
            let id = graph_a.add_node("SOCIAL_CLASS").unwrap();
            graph_a
                .update_node(id, "social-class/wealth", value)
                .unwrap();
        }
        // The SAME multiset {1.0, 2.0, 3.0}, a DIFFERENT element order.
        let mut graph_b = babylon_graph::memory::MemoryGraph::new();
        for value in [1.0, 2.0, 3.0] {
            let id = graph_b.add_node("SOCIAL_CLASS").unwrap();
            graph_b
                .update_node(id, "social-class/wealth", value)
                .unwrap();
        }
        // The multiset WITH a duplicated element: {3.0, 1.0, 2.0, 2.0}.
        let mut graph_c = babylon_graph::memory::MemoryGraph::new();
        for value in [3.0, 1.0, 2.0, 2.0] {
            let id = graph_c.add_node("SOCIAL_CLASS").unwrap();
            graph_c
                .update_node(id, "social-class/wealth", value)
                .unwrap();
        }

        for op in ["min", "max"] {
            let query = format!(
                "(fold {op} (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))"
            );
            let mut fuel_a = 1_000;
            let result_a = eval_over(&query, &graph_a, None, &mut fuel_a).unwrap();
            let mut fuel_b = 1_000;
            let result_b = eval_over(&query, &graph_b, None, &mut fuel_b).unwrap();
            let mut fuel_c = 1_000;
            let result_c = eval_over(&query, &graph_c, None, &mut fuel_c).unwrap();

            assert_eq!(result_a, result_b, "{op}: element order must not matter");
            assert_eq!(
                result_a, result_c,
                "{op}: a duplicated element must not change the result"
            );
        }
    }

    /// D-row Q5: weighted mean is `Σ(wᵢ·xᵢ) ÷ Σwᵢ`, both sums reduced in
    /// iteration order. Three nodes, wealth (body) = 1.0, 2.0, 3.0,
    /// head-count (weight) = 10, 20, 30: Σwx = 10+40+90 = 140, Σw = 60,
    /// mean = 140/60 (the exact f64 bit pattern for that division).
    #[test]
    fn weighted_mean_is_sum_of_products_over_sum_of_weights() {
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        for (wealth, head_count) in [(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)] {
            let id = graph.add_node("SOCIAL_CLASS").unwrap();
            graph
                .update_node(id, "social-class/wealth", wealth)
                .unwrap();
            graph
                .update_node(id, "social-class/head-count", head_count)
                .unwrap();
        }
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold mean (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth) \
             :weight (field-of it social-class/head-count))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        // Σ(wᵢ·xᵢ) = 10*1 + 20*2 + 30*3 = 140.0; Σwᵢ = 60.0 — both reduced
        // left-to-right in ascending-id order (D-row Q5's exact shape).
        let sum_wx = 10.0_f64 * 1.0 + 20.0 * 2.0 + 30.0 * 3.0;
        let sum_w = 10.0_f64 + 20.0 + 30.0;
        let expected = sum_wx / sum_w;
        assert_eq!(result, Value::Real(expected));
        // Pin the exact bit pattern, not just the printed value.
        let Value::Real(got) = result else {
            unreachable!()
        };
        assert_eq!(got.to_bits(), expected.to_bits());
    }

    /// M1: `:weight` is admitted by §2.7's `<fold>` grammar for every
    /// fold-op (the production carries `( ":weight" <expr> )?` unconditioned
    /// on `<fold-op>`), but §3.4's per-operator table gives it a reading for
    /// `mean` ALONE. `sum`/`min`/`max`/`count` silently discarding a
    /// supplied `:weight` (the pre-fix behaviour: `eval_body_and_weight`
    /// evaluates it and the three callers drop it as `_weight_val`) is
    /// exactly the class of silent-degradation footgun §3.4's kind law
    /// exists to close — refused by NAME here instead, citing the op,
    /// `:weight`, §3.4, and D-row Q11.
    #[test]
    fn weight_on_a_non_mean_fold_op_refuses_by_name_citing_the_d_row() {
        let graph = wealth_ladder(2);
        for op in ["sum", "min", "max", "count"] {
            let mut fuel = 1_000;
            let err = eval_over(
                &format!(
                    "(fold {op} (nodes NodeType/SOCIAL_CLASS) \
                     (field-of it social-class/wealth) \
                     :weight (field-of it social-class/wealth))"
                ),
                &graph,
                None,
                &mut fuel,
            )
            .unwrap_err();
            assert!(err.code.is_none(), "{op}: {err}");
            assert!(err.message.contains(op), "{op}: {err}");
            assert!(err.message.contains(":weight"), "{op}: {err}");
            assert!(err.message.contains("§3.4"), "{op}: {err}");
            assert!(err.message.contains("D-row Q11"), "{op}: {err}");
        }
    }

    #[test]
    fn fold_charges_the_body_once_per_element() {
        let graph = wealth_ladder(3);
        // fold(2) + query(1) + 3 × field-of(2) = 9.
        let mut fuel = 100;
        eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            fuel, 91,
            ":fuel-used is a conformance-vector quantity (§6.1)"
        );
    }

    /// D-row Q6 (Director ruling 2026-08-11): `mean` serves Real-typed
    /// bodies only; an Int body refuses BY NAME, citing `mean`, `Int` and
    /// the D-row — no promote-then-divide.
    #[test]
    fn mean_over_an_int_body_refuses_by_name() {
        let graph = wealth_ladder(2);
        let mut fuel = 1_000;
        let err = eval_over(
            "(fold mean (nodes NodeType/SOCIAL_CLASS) 5)",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap_err();
        assert!(err.message.contains("mean"), "{err}");
        assert!(err.message.contains("Int"), "{err}");
        assert!(err.message.contains("D-row Q6"), "{err}");
    }

    #[test]
    fn fold_min_and_max_extremise_the_body_value() {
        let graph = wealth_ladder(4); // wealth 0.0, 1.0, 2.0, 3.0
        let mut fuel = 1_000;
        let min = eval_over(
            "(fold min (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(min, Value::Real(0.0));
        let mut fuel2 = 1_000;
        let max = eval_over(
            "(fold max (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel2,
        )
        .unwrap();
        assert_eq!(max, Value::Real(3.0));
    }

    #[test]
    fn fold_count_counts_the_materialized_set() {
        let graph = wealth_ladder(5);
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold count (nodes NodeType/SOCIAL_CLASS) it)",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn fold_sum_over_neighbors_is_the_territory_spillover_shape() {
        // Shape A from the query-lane-e2e vectors: a fold sum over typed
        // neighbors reading a field — the exact motivating consumer shape.
        use babylon_graph::substrate::GraphSubstrate;
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        let subject = graph.add_node("TERRITORY").unwrap();
        let mut total = 0.0;
        for heat in [1.5, 2.5, 3.0] {
            let neighbor = graph.add_node("TERRITORY").unwrap();
            graph.update_node(neighbor, "territory/heat", heat).unwrap();
            graph.add_edge("ADJACENCY", subject, neighbor, 1.0).unwrap();
            total += heat;
        }
        let mut fuel = 1_000;
        let result = eval_over(
            "(fold sum (neighbors self EdgeType/ADJACENCY :any NodeType/TERRITORY) \
             (field-of it territory/heat))",
            &graph,
            Some(subject),
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Real(total));
    }

    // ---- Task 6: exists / forall ----

    #[test]
    fn exists_over_an_empty_query_is_false_forall_over_an_empty_query_is_true() {
        let graph = wealth_ladder(0);
        let mut fuel = 1_000;
        let exists_result = eval_over(
            "(exists (nodes NodeType/SOCIAL_CLASS) (< (field-of it social-class/wealth) 5))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(exists_result, Value::Bool(false));

        let mut fuel2 = 1_000;
        let forall_result = eval_over(
            "(forall (nodes NodeType/SOCIAL_CLASS) (< (field-of it social-class/wealth) 5))",
            &graph,
            None,
            &mut fuel2,
        )
        .unwrap();
        assert_eq!(forall_result, Value::Bool(true));
    }

    /// `(exists <query>)` with NO body: §2.4's reading is "the query is
    /// non-empty".
    #[test]
    fn exists_with_no_body_tests_non_emptiness() {
        let empty = wealth_ladder(0);
        let mut fuel = 1_000;
        assert_eq!(
            eval_over(
                "(exists (nodes NodeType/SOCIAL_CLASS))",
                &empty,
                None,
                &mut fuel
            )
            .unwrap(),
            Value::Bool(false)
        );
        let nonempty = wealth_ladder(1);
        let mut fuel2 = 1_000;
        assert_eq!(
            eval_over(
                "(exists (nodes NodeType/SOCIAL_CLASS))",
                &nonempty,
                None,
                &mut fuel2
            )
            .unwrap(),
            Value::Bool(true)
        );
    }

    /// P5 (defense in depth): §2.4's grammar (line 736 of the reference)
    /// makes `forall`'s `<cond>` MANDATORY — unlike `exists`', which is
    /// optional (`grammar.rs`'s `ARITIES` table pins this: `forall` takes
    /// EXACTLY 2 operands, `exists` 1 or 2). A no-cond `forall` reaching
    /// evaluation means a load-time arity gate (`E-PARSE-042`) did not run.
    /// Before this fix that shape silently fell into the shared
    /// `exists`-style "query is non-empty" branch, which gives `forall`
    /// over an EMPTY query `#f` — contradicting §4.4's own
    /// forall-empty-is-`#t` pin. Refuse loudly instead of computing the
    /// wrong Boolean.
    #[test]
    fn forall_with_no_cond_is_a_loud_defense_error_not_a_silent_fallback() {
        let empty = wealth_ladder(0);
        let mut fuel = 1_000;
        let err = eval_over(
            "(forall (nodes NodeType/SOCIAL_CLASS))",
            &empty,
            None,
            &mut fuel,
        )
        .unwrap_err();
        assert_eq!(err.code, None, "{err}");
        assert!(err.message.contains("forall"), "{err}");
        assert!(err.message.contains("MANDATORY"), "{err}");
    }

    /// §4.1 short-circuit: `exists` stops at the first element whose
    /// predicate is true; `forall` stops at the first false. `:fuel-used`
    /// over a 3-element set must be STRICTLY SMALLER when the deciding
    /// element is the first, not the last.
    #[test]
    fn exists_and_forall_short_circuit_and_charge_less_fuel_when_element_one_decides() {
        // wealth 0.0, 1.0, 2.0 (ascending id order).
        let graph = wealth_ladder(3);

        // exists: element 0 (wealth 0.0 < 1) decides immediately.
        let mut fuel_early = 1_000;
        eval_over(
            "(exists (nodes NodeType/SOCIAL_CLASS) (< (field-of it social-class/wealth) 1))",
            &graph,
            None,
            &mut fuel_early,
        )
        .unwrap();
        let early_used = 1_000 - fuel_early;

        // exists: only element 2 (wealth 2.0 > 1) satisfies — must visit all three.
        let mut fuel_late = 1_000;
        eval_over(
            "(exists (nodes NodeType/SOCIAL_CLASS) (> (field-of it social-class/wealth) 1))",
            &graph,
            None,
            &mut fuel_late,
        )
        .unwrap();
        let late_used = 1_000 - fuel_late;

        assert!(
            early_used < late_used,
            "early={early_used} late={late_used}: short-circuit must charge less fuel"
        );

        // forall: element 0 (wealth 0.0, NOT < 0) decides immediately (false).
        let mut fuel_forall_early = 1_000;
        eval_over(
            "(forall (nodes NodeType/SOCIAL_CLASS) (< (field-of it social-class/wealth) 0))",
            &graph,
            None,
            &mut fuel_forall_early,
        )
        .unwrap();
        let forall_early_used = 1_000 - fuel_forall_early;

        // forall: all three satisfy (< 10) — every element visited.
        let mut fuel_forall_late = 1_000;
        eval_over(
            "(forall (nodes NodeType/SOCIAL_CLASS) (< (field-of it social-class/wealth) 10))",
            &graph,
            None,
            &mut fuel_forall_late,
        )
        .unwrap();
        let forall_late_used = 1_000 - fuel_forall_late;

        assert!(
            forall_early_used < forall_late_used,
            "forall early={forall_early_used} late={forall_late_used}: short-circuit must charge less fuel"
        );
    }

    /// The Territory `_find_sink_node` shape: guarding a selection with
    /// `exists` so an empty neighbourhood takes the fallback branch instead
    /// of raising `E-EVAL-021`. SMALL(b) (PR #514 fix round): `select-max`
    /// IS fully implemented now (Task 7, landed) — this vector still holds
    /// because `if` never evaluates the untaken branch (§4.1), so this
    /// vector's `exists` being false means the branch containing
    /// `select-max` is never EVALUATED, regardless of whether the head is
    /// served.
    #[test]
    fn exists_guards_a_selection_over_a_possibly_empty_query() {
        use babylon_graph::substrate::GraphSubstrate;
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        let subject = graph.add_node("TERRITORY").unwrap();
        // No ADJACENCY edges at all — an empty neighbourhood.
        let mut fuel = 1_000;
        let result = eval_over(
            "(if (exists (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) #t) \
             (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/TERRITORY) \
                         (field-of it territory/heat)) \
             self)",
            &graph,
            Some(subject),
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            result,
            Value::NodeRef(subject),
            "the fallback branch, never E-EVAL-021"
        );
    }

    // ---- Task 7: select-max / select-min ----

    /// D45/§2.7: the tiebreak is a property of the LANGUAGE — the FIRST
    /// element in ascending id byte order wins, for both operators, when two
    /// elements score equally.
    #[test]
    fn tied_scores_break_to_the_smaller_id_for_both_operators() {
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        let first = graph.add_node("SOCIAL_CLASS").unwrap(); // id 0
        let second = graph.add_node("SOCIAL_CLASS").unwrap(); // id 1
        graph
            .update_node(first, "social-class/wealth", 5.0)
            .unwrap();
        graph
            .update_node(second, "social-class/wealth", 5.0)
            .unwrap();
        for op in ["select-max", "select-min"] {
            let mut fuel = 1_000;
            let result = eval_over(
                &format!("({op} (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))"),
                &graph,
                None,
                &mut fuel,
            )
            .unwrap();
            assert_eq!(
                result,
                Value::NodeRef(first),
                "{op}: the smaller id wins a tie"
            );
        }
    }

    #[test]
    fn selection_over_an_empty_query_is_e_eval_021() {
        let graph = wealth_ladder(0);
        for op in ["select-max", "select-min"] {
            let mut fuel = 1_000;
            let err = eval_over(
                &format!("({op} (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))"),
                &graph,
                None,
                &mut fuel,
            )
            .unwrap_err();
            assert_eq!(err.code, Some(EvalCode::EmptyAggregate), "{op}: {err}");
            assert_eq!(err.code.unwrap().spec_code(), "E-EVAL-021");
        }
    }

    /// §2.7: §3.4 polices AGGREGATION, not ordering — an intensive score
    /// ranks correctly at runtime with no evaluator-level kind check (the
    /// evaluator has no `TypeEnv`/field-kind registry to enforce one with;
    /// this guards against ever accidentally adding one where the spec
    /// draws no such line).
    #[test]
    fn an_intensive_score_is_accepted_and_ranks_correctly() {
        let graph = wealth_ladder(4); // wealth (here standing in for any scalar) 0,1,2,3
        let mut fuel = 1_000;
        let result = eval_over(
            "(select-max (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::NodeRef(babylon_graph::substrate::NodeId(3)));
    }

    /// A selection result is the query's element type — usable as
    /// `field-of`'s referent operand, exactly the §2.7 worked example.
    #[test]
    fn a_selection_result_is_the_element_operand_of_field_of() {
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        let low = graph.add_node("SOCIAL_CLASS").unwrap();
        let high = graph.add_node("SOCIAL_CLASS").unwrap();
        graph.update_node(low, "social-class/wealth", 1.0).unwrap();
        graph.update_node(high, "social-class/wealth", 9.0).unwrap();
        let mut fuel = 1_000;
        let result = eval_over(
            "(field-of \
               (select-max (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth)) \
               social-class/wealth)",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(result, Value::Real(9.0));
    }

    // ---- Task 9: :as element naming and nested bodies ----

    /// §6.2 family-17's two-hop shape (D53/D54), built over slice 1's own
    /// served heads (`nodes`/`neighbors`) rather than the spec's
    /// `hyperedges`/`members-of` worked example, which slice 3 serves: an
    /// outer fold over `nodes` names its element `:as outer`; the inner fold
    /// (over `neighbors outer …`) reads `it`, which must resolve to the
    /// INNER element, while `outer` still resolves to the OUTER one — both
    /// live in the element stack at once.
    #[test]
    fn it_resolves_to_the_inner_element_and_the_as_name_to_the_outer() {
        use babylon_graph::substrate::GraphSubstrate;
        let mut graph = babylon_graph::memory::MemoryGraph::new();
        // Two outer SOCIAL_CLASS nodes, each with one ORGANIZATION neighbor
        // via TENANCY, each neighbor carrying a distinguishable field.
        let outer_a = graph.add_node("SOCIAL_CLASS").unwrap();
        let inner_a = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(inner_a, "organization/claim-strength", 10.0)
            .unwrap();
        graph.add_edge("TENANCY", outer_a, inner_a, 1.0).unwrap();

        let outer_b = graph.add_node("SOCIAL_CLASS").unwrap();
        let inner_b = graph.add_node("ORGANIZATION").unwrap();
        graph
            .update_node(inner_b, "organization/claim-strength", 20.0)
            .unwrap();
        graph.add_edge("TENANCY", outer_b, inner_b, 1.0).unwrap();
        let mut fuel = 10_000;
        // The outer fold names its element `outer`; the inner fold's query
        // reads `outer` (the OUTER NodeRef) to find that subject's tenant,
        // and its body reads `it` (the INNER, ORGANIZATION NodeRef).
        let result = eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) :as outer \
               (fold sum (neighbors outer EdgeType/TENANCY :out NodeType/ORGANIZATION) \
                     (field-of it organization/claim-strength)))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            result,
            Value::Real(30.0),
            "10.0 + 20.0 — it and outer must resolve to their OWN elements, not collide"
        );
    }

    /// §3.7: `cost(:as name) = 0` (the name is a binding, not a charged
    /// node) — a REFERENCE to it costs 1, like any other variable
    /// reference. Isolated from the two-hop test above so the fuel pinning
    /// does not ride on graph-shaped fixtures.
    #[test]
    fn an_as_name_costs_zero_and_a_reference_to_it_costs_one() {
        let graph = wealth_ladder(1);
        // fold(2) + query(1) + `:as outer`(0) + 1 × field-of(2) = 5. The
        // body reads `outer` (a reference: variable-ref 1) instead of `it`,
        // through field-of: accessor(1) + outer-ref(1) = 2, matching
        // field-of's own pinned cost for `it` — the SAME shape, proving the
        // name costs the identical 1 a bare reference would.
        let mut fuel = 100;
        eval_over(
            "(fold sum (nodes NodeType/SOCIAL_CLASS) :as outer \
               (field-of outer social-class/wealth))",
            &graph,
            None,
            &mut fuel,
        )
        .unwrap();
        assert_eq!(
            fuel, 95,
            ":fuel-used is a conformance-vector quantity (§6.1)"
        );
    }

    // ============================ Review round 1 (#576): the
    // `element_content_id` fallback gate. Review round 2 (#576 I1/I2)
    // sharpened the gate from `is_empty()`-value-distinct to
    // `Option`-type-distinct, and added `edge_type` to the Edge arm's
    // composition.

    /// (b) The `None` (never-hydrated) fixture path still works: no
    /// scenario was ever hydrated (this crate's OWN hand-built-graph unit
    /// tests are exactly this shape), so a `NodeId` with no map to consult
    /// falls back to its own `Debug` rendering — not a hydration bug, since
    /// there was never a map to miss.
    #[test]
    fn element_content_id_falls_back_to_debug_rendering_when_never_hydrated() {
        let id = babylon_graph::substrate::NodeId(7);
        assert_eq!(
            element_content_id(&Element::Node(id), None).unwrap(),
            format!("{id:?}")
        );
    }

    /// (a) The error fires: a `NodeId` missing from a `Some`-hydrated map is
    /// a hard `EvalError`, never a silent fallback — the review's own
    /// recommended fix, converting "trust me, unreachable" into a
    /// mechanically-checked invariant. Named node 0 is present; node 1 is
    /// absent despite the map holding an entry — the hydration-bug shape.
    #[test]
    fn element_content_id_hard_errors_on_a_miss_against_a_hydrated_map() {
        let mut named: HashMap<babylon_graph::substrate::NodeId, String> = HashMap::new();
        named.insert(babylon_graph::substrate::NodeId(0), "core".to_owned());
        let missing = babylon_graph::substrate::NodeId(1);
        let err = element_content_id(&Element::Node(missing), Some(&named)).unwrap_err();
        assert!(err.message.contains("hydration bug"), "{}", err.message);
    }

    /// (c) Review round 2's own distinguishing case (#576 I2): `Some(map)`
    /// where `map` is EMPTY — a declarations-only scenario, hydrated with
    /// zero `(node …)` forms. Under the pre-I2 `is_empty()` gate this was
    /// indistinguishable from "never hydrated" and silently fell back to
    /// the `NodeId`-Debug rendering; under the `Option`-typed gate,
    /// `Some(empty_map)` IS a hydration, so a miss is a hard error just
    /// like a non-empty map's miss — never a silent fallback, regardless
    /// of how many entries the hydrated map holds.
    #[test]
    fn element_content_id_hard_errors_on_a_miss_against_a_hydrated_but_empty_map() {
        let empty: HashMap<babylon_graph::substrate::NodeId, String> = HashMap::new();
        let id = babylon_graph::substrate::NodeId(7);
        let err = element_content_id(&Element::Node(id), Some(&empty)).unwrap_err();
        assert!(err.message.contains("hydration bug"), "{}", err.message);
    }

    /// The same two properties through `Element::Edge`'s two-endpoint path
    /// (`content_id_of` is a shared closure — both arms must honor the
    /// gate identically). A source hit + a target miss against a
    /// hydrated map must still refuse, not silently frame a Debug string
    /// for the missing half.
    #[test]
    fn element_content_id_edge_variant_hard_errors_on_either_endpoint_missing() {
        let mut named: HashMap<babylon_graph::substrate::NodeId, String> = HashMap::new();
        named.insert(babylon_graph::substrate::NodeId(0), "core".to_owned());
        let edge = EdgeKey {
            source: babylon_graph::substrate::NodeId(0),
            target: babylon_graph::substrate::NodeId(99),
            edge_type: "SOLIDARITY".to_owned(),
        };
        let err = element_content_id(&Element::Edge(edge), Some(&named)).unwrap_err();
        assert!(err.message.contains("hydration bug"), "{}", err.message);
    }

    /// I1 (review round 2, #576): two `EdgeKey`s with the SAME endpoints
    /// but DIFFERENT `edge_type`s must resolve to DIFFERENT chain entries.
    /// Mutation-caught: dropping `&key.edge_type` from the `framed(...)`
    /// call (the exact pre-fix composition) makes this assertion fail —
    /// both edges would resolve to the identical two-segment
    /// `framed([source, target])` string.
    #[test]
    fn element_content_id_edge_variant_distinguishes_parallel_edges_by_type() {
        let mut named: HashMap<babylon_graph::substrate::NodeId, String> = HashMap::new();
        named.insert(babylon_graph::substrate::NodeId(0), "a".to_owned());
        named.insert(babylon_graph::substrate::NodeId(1), "b".to_owned());
        let solidarity = EdgeKey {
            source: babylon_graph::substrate::NodeId(0),
            target: babylon_graph::substrate::NodeId(1),
            edge_type: "SOLIDARITY".to_owned(),
        };
        let exploitation = EdgeKey {
            source: babylon_graph::substrate::NodeId(0),
            target: babylon_graph::substrate::NodeId(1),
            edge_type: "EXPLOITATION".to_owned(),
        };
        let solidarity_key = element_content_id(&Element::Edge(solidarity), Some(&named)).unwrap();
        let exploitation_key =
            element_content_id(&Element::Edge(exploitation), Some(&named)).unwrap();
        assert_ne!(
            solidarity_key, exploitation_key,
            "two parallel edges of different types between the same node \
             pair must resolve to different chain entries — same source, \
             same target, different edge_type"
        );
    }
}
