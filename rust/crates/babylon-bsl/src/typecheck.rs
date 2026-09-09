//! The BSL typechecker's first slice: the §3.4 aggregation law
//! (`bsl-language.rst` — "the extensive/intensive lexicon becomes real
//! types"). The load-bearing rule the plan calls out: unweighted
//! aggregation of an intensive-kinded field is a `TypeError` at load, not
//! a runtime surprise — the intensive-aggregation-variance-error class,
//! caught by the compiler.
//!
//! **Deviation from the Phase 1 plan's sketch, recorded:** the sketch
//! rejected every unweighted op except `count` on an intensive field. The
//! normative law is §3.4's five-row PER-OPERATOR table — `min`/`max` are
//! kind-neutral and always legal, and `sum` of an intensive field is
//! `E-TYPE-041` even WITH a weight (summing an intensive quantity is
//! meaningless; no weight rescues it). This module implements the table.
//!
//! Scope (deliberately narrow, per the plan): the fold BODY here is a bare
//! field reference, so its kind is always a declared `FieldKind`.
//!
//! **The expression-kind arm (#491 T1, ADR202 R1(c)/OQ-I).** Kind-NEUTRAL
//! bodies (literals, `:const` bindings) and `E-TYPE-040` kind mixing over
//! `<arith>` and `if` are implemented below, by [`check_kind_mixing`] and
//! its private `expr_kind` family — a SEPARATE walk from
//! [`typecheck_aggregation`]'s fold-specific one, extending the crate's
//! existing dispatch (the sequence of typecheck passes `rule_pipeline`
//! composes) rather than restructuring the fold arm. §3.4's `*`/`/` bullet
//! is split by this repair: extensive × extensive stays `E-TYPE-040` (an
//! area-of-an-area), but extensive ÷ extensive is now LICENSED as
//! **intensive** — `w̄ = wealth ÷ population` is the textbook definition of
//! an intensive quantity (density = mass ÷ volume), the same unit-algebra
//! standing D90 already took for the symmetric weighted-mean gap.
//! Extensive × intensive is ALSO legal (result extensive) — found reading
//! `lifecycle.bsl`'s real, committed `new-wealth-d-prime = wealth-d-prime
//! × surviving-fraction` while gating this repair (not in the plan): a
//! stock scaled by a fraction is the ordinary case, not an area-of-an-area,
//! and a first draft that refused it broke committed content. `if` absorbs
//! a kind-neutral branch the same way `+`, `-` and `*` do — found reading
//! the SAME rule's `surviving-fraction` binding, whose two `if` branches
//! are Intensive and Neutral respectively. **Intensive × intensive is ALSO
//! legal (result intensive)** — controller adjudication, 2026-08-18,
//! delegated Director provenance, the third instance of this defect class
//! found reading `consciousness.bsl`'s real, committed `p6-route`:
//! `delta-r = (* (* consumed eff-sol) routing-scale)` where `consumed`
//! (`agitation × consumption-rate`) and `eff-sol` (a solidarity/chauvinist
//! ratio) are both intensive, and the product feeds `r1 = r + delta-r`
//! where `r` (`social-class/revolutionary`) is itself intensive — the
//! standard dimensional rule: the product of two intensive quantities (a
//! rate scaled by a dimensionless coefficient) is intensive, the same
//! "scale by a dimensionless factor" logic already licensed for extensive
//! × intensive, applied to the other pairing. **Intensive ÷ intensive is
//! ALSO legal (result intensive)** — same sitting, same delegated
//! provenance, the fourth straddle site: licensing `*` above exposed
//! `p6-route`'s own next site, `r2`/`l2`/`f2`'s simplex renormalization
//! (`(/ r1 total)` and siblings, `r1`/`l1`/`f1`/`total` all intensive). A
//! ratio of two intensive quantities is a dimensionless share — simplex
//! normalization is the canonical intensive operation this coarse
//! two-kind algebra has a name for. A complete static sweep of every
//! `<arith>`/`if` site across all 13 committed rule files (not just
//! `p6-route`) found no other site either ruling's shape would still
//! refuse. Every combination still unlicensed — extensive mixed with
//! intensive under `/` specifically (division is not commutative, so
//! `*`'s licensed mixed case does not carry over) — stays conservatively
//! refused, matching the bullet's own "deliberately conservative,
//! Phase-1 review item" framing for the one case it did name, extended
//! (not invented) to the cases it never named. `:metric`
//! bindings and `metric-of` reads decline to constrain (`None`, not a
//! guess): §2.11's metric-kind registry is not threaded through `TypeEnv`
//! yet, a disclosed Phase-1 gap, not a silent pass — matching this
//! module's own established policy of naming gaps rather than hiding them
//! (see `rule_pipeline`'s compound-fold-body doc for the same shape).
//!
//! **Why the refusal is principled, not stylistic (CT4P B4, issue #525).**
//! Extensive quantities close under an associative combine with an honest
//! identity — they are **monoids**. Intensive quantities are not: a mean
//! exists only as the **quotient of two extensive monoids**,
//! `Σ(wᵢ·xᵢ) ÷ Σwᵢ` (exactly what [`crate::evaluator`]'s `fold_mean` carries
//! as `(sum_wx, sum_w)`, dividing once at the end). An unweighted mean of an
//! intensive field has discarded the denominator monoid — which is why
//! `E-TYPE-042` refuses it, not because the number would look wrong. The
//! litmus test for any future fold this table might grow to cover: does it
//! close under an associative combine with an honest identity, or is it
//! secretly a quotient of two things that do?

use crate::exemptions::IntensiveAggregationExemption;
use crate::reader::{Atom, SExpr};
use crate::score_class::{classify, selection_result_class, ClassEnv, ScoreClass};
use crate::types::{BslType, FieldDecl, FieldKind};
use std::collections::HashMap;

/// The §3.4 aggregation-law error codes, plus the two the R9 chapters add
/// to this module's remit (`E-TYPE-016` on a selection score, `E-TYPE-017`
/// on a reference comparison).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCode {
    /// `E-TYPE-016` — a `select-max`/`select-min` score whose static type
    /// is not a comparable scalar (D46).
    NonComparableScore,
    /// `E-TYPE-017` — a reference compared with an ordering operator, with
    /// a reference of a different kind, or with a non-reference (D67).
    BadReferenceComparison,
    /// `E-TYPE-041` — summing an intensive quantity is meaningless.
    SumOfIntensive,
    /// `E-TYPE-042` — unweighted `mean` of an intensive field.
    UnweightedMeanOfIntensive,
    /// `E-TYPE-043` — a `mean` weight that is not extensive-kinded.
    NonExtensiveWeight,
    /// `E-TYPE-044` — a `sum`/`mean`/`min`/`max` fold body naming an
    /// `:enum-type`-declared field (#551 closure). `count` is exempt: its
    /// body is never evaluated (§3.4 row 6), so an enum-declared body
    /// there is inert rather than a content error.
    EnumFoldBody,
    /// `E-TYPE-040` — an `<arith>` or `if` expression mixes intensive and
    /// extensive kinds (§3.4): `+`/`-` never mixes kind; `*`/`/` never
    /// mixes kind and additionally refuses extensive × extensive — the
    /// licensed same-kind cases are extensive ÷ extensive → intensive
    /// (D90/D181, the `w̄ = wealth ÷ population` shape, ADR202 R1(c),
    /// #491 T1), intensive × intensive → intensive (D182), and
    /// intensive ÷ intensive → intensive (D183); `if` never lets its
    /// branches disagree in kind.
    KindMixing,
}

impl TypeCode {
    /// The spec's code string, e.g. `"E-TYPE-042"`.
    #[must_use]
    pub fn spec_code(self) -> &'static str {
        match self {
            Self::NonComparableScore => "E-TYPE-016",
            Self::BadReferenceComparison => "E-TYPE-017",
            Self::SumOfIntensive => "E-TYPE-041",
            Self::UnweightedMeanOfIntensive => "E-TYPE-042",
            Self::NonExtensiveWeight => "E-TYPE-043",
            Self::EnumFoldBody => "E-TYPE-044",
            Self::KindMixing => "E-TYPE-040",
        }
    }
}

/// A typechecking failure: spec-coded where the spec codes it, structural
/// otherwise (`code: None`) — no code is invented.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    /// The spec code, when §3.4 assigns one.
    pub code: Option<TypeCode>,
    /// Human-readable detail.
    pub message: String,
}

/// The typechecker's environment: declared fields and the exemption ledger.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Declared fields by BSL name (kebab-case symbols or qnames).
    pub fields: HashMap<String, FieldDecl>,
    /// The `EXTENSIVE_INTENSIVE_EXEMPTIONS` ledger in force — an exemption
    /// suppresses `E-TYPE-041/042/043` for its named field only (§3.4).
    pub exemptions: &'static [IntensiveAggregationExemption],
}

/// Typecheck one aggregation form: `(op field)` or
/// `(mean field :weight weight-field)`, with `op` one of
/// `sum | mean | min | max | count`, applying §3.4's per-operator table.
///
/// **#551 closure:** an `:enum-type`-declared field (`FieldKind::
/// NotApplicable` — §2.13/D101's "no aggregation kind" ruling) is refused,
/// `E-TYPE-044`, for every op that actually EVALUATES its body —
/// `sum`/`mean`/`min`/`max` — before this table's per-op law even runs.
/// `count` is the sole exemption, for the same reason §3.4 row 6 already
/// makes it kind-blind for `Intensive`/`Extensive`: it never evaluates its
/// body at all (`evaluator::fold_count` takes no body argument), so an
/// enum-declared name there is inert content, not a content error.
/// `min`/`max` are NOT exempted despite being "kind-neutral" elsewhere in
/// this table (§3.4 row 5) — kind-neutral there means "extensive vs.
/// intensive doesn't matter", not "any type works": `min`/`max` compare
/// via `apply_ordering`, which itself refuses `Value::Enum` (§3.1, "Enum
/// and Bool compare with =/!= alone") — this closure just catches at load
/// what would otherwise die at evaluation on the SECOND element only (a
/// single-element enum-body fold would silently "succeed", returning the
/// one value with no ordering ever invoked — a population-size-dependent
/// landmine this refusal removes).
///
/// # Errors
/// [`TypeError`] with the spec code for a §3.4 violation, or with
/// `code: None` for a malformed form or an unknown field.
pub fn typecheck_aggregation(expr: &SExpr, env: &TypeEnv) -> Result<BslType, TypeError> {
    let (op, field_name, weight_name) = destructure_aggregation(expr)?;
    let field = resolve_field(env, field_name)?;
    let weight = match weight_name {
        Some(name) => Some(resolve_field(env, name)?),
        None => None,
    };
    let exempted = env.exemptions.iter().any(|e| e.field_name == field_name);
    // CT4P A3 (issue #525): `op` converts to the closed `FoldOp` sum type
    // ONCE, here, and every arm below matches it EXHAUSTIVELY — no wildcard.
    // The message text on the unrecognized-operator path is preserved
    // byte-for-byte; only the dispatch mechanism changed.
    let Some(fold_op) = crate::grammar::FoldOp::parse(op) else {
        return Err(TypeError {
            code: None,
            message: format!("unknown aggregation operator '{op}'"),
        });
    };
    // #551 closure: exhaustive over `FoldOp` (mirrors `rule_pipeline::
    // carries_body_kind`'s own Sum/Mean/Min/Max-vs-Count split, no
    // wildcard, so a 6th fold-op forces a decision here too) — every op
    // that evaluates its body refuses an enum-declared one; `count` alone
    // discards its body unevaluated and is unaffected.
    let evaluates_body = match fold_op {
        crate::grammar::FoldOp::Sum
        | crate::grammar::FoldOp::Mean
        | crate::grammar::FoldOp::Min
        | crate::grammar::FoldOp::Max => true,
        crate::grammar::FoldOp::Count => false,
    };
    if evaluates_body && field.kind == FieldKind::NotApplicable {
        return Err(TypeError {
            code: Some(TypeCode::EnumFoldBody),
            message: format!(
                "E-TYPE-044: {op} over enum-declared field '{field_name}': \
                 Enum<T> has no aggregation kind (§2.13, D101) — sum, mean, \
                 min and max are all undefined over it. Only count may name \
                 an enum-declared field, because count never evaluates its \
                 body (§3.4 row 6)"
            ),
        });
    }
    match fold_op {
        crate::grammar::FoldOp::Sum => {
            if field.kind == FieldKind::Intensive && !exempted {
                return Err(TypeError {
                    code: Some(TypeCode::SumOfIntensive),
                    message: format!(
                        "sum of intensive field '{field_name}': summing an intensive \
                         quantity is meaningless — no weight rescues it (§3.4)"
                    ),
                });
            }
            Ok(field.ty.clone())
        }
        crate::grammar::FoldOp::Mean => {
            if field.kind == FieldKind::Intensive && !exempted {
                match weight {
                    None => {
                        return Err(TypeError {
                            code: Some(TypeCode::UnweightedMeanOfIntensive),
                            message: format!(
                                "unweighted mean of intensive field '{field_name}': add an \
                                 explicit extensive :weight term (§3.4)"
                            ),
                        })
                    }
                    Some(w) if w.kind != FieldKind::Extensive => {
                        return Err(TypeError {
                            code: Some(TypeCode::NonExtensiveWeight),
                            message: format!(
                                "the :weight of a mean over intensive field '{field_name}' \
                                 must be extensive-kinded (§3.4)"
                            ),
                        })
                    }
                    Some(_) => {}
                }
            }
            Ok(field.ty.clone())
        }
        // §3.4 row 5: min/max are kind-neutral operations, legal on any kind.
        crate::grammar::FoldOp::Min | crate::grammar::FoldOp::Max => Ok(field.ty.clone()),
        // §3.4 row 6: count is always legal; result Int, extensive.
        crate::grammar::FoldOp::Count => Ok(BslType::Int),
    }
}

/// Destructure `(op field)` / `(op field :weight weight-field)` into its
/// three named parts, rejecting every other shape loudly.
fn destructure_aggregation(expr: &SExpr) -> Result<(&str, &str, Option<&str>), TypeError> {
    let malformed = || TypeError {
        code: None,
        message: "aggregation form must be (op field) or (op field :weight weight-field)".into(),
    };
    let SExpr::List(items) = expr else {
        return Err(malformed());
    };
    let [op_expr, field_expr, rest @ ..] = items.as_slice() else {
        return Err(malformed());
    };
    let SExpr::Atom(Atom::Symbol(op)) = op_expr else {
        return Err(malformed());
    };
    let field_name = field_ref_name(field_expr).ok_or_else(malformed)?;
    let weight_name = match rest {
        [] => None,
        [SExpr::Atom(Atom::Keyword(kw)), weight_expr] if kw == "weight" => {
            Some(field_ref_name(weight_expr).ok_or_else(malformed)?)
        }
        _ => return Err(malformed()),
    };
    Ok((op, field_name, weight_name))
}

/// A field reference is a `symbol` or a `qname` (§1.4's own example:
/// `social-class/wealth`).
fn field_ref_name(expr: &SExpr) -> Option<&str> {
    match expr {
        SExpr::Atom(Atom::Symbol(name) | Atom::QName(name)) => Some(name),
        _ => None,
    }
}

fn resolve_field<'e>(env: &'e TypeEnv, name: &str) -> Result<&'e FieldDecl, TypeError> {
    env.fields.get(name).ok_or_else(|| TypeError {
        code: None,
        message: format!("unknown field: '{name}'"),
    })
}

/// Walk a form tree and apply D46 to every `select-max`/`select-min`
/// score: it must have a **comparable scalar** static type. `Bool`,
/// `Enum<T>`, `Str`, references and sets are `E-TYPE-016`.
///
/// **Kind is unconstrained on the score, deliberately** (D46): §3.4 polices
/// *aggregation*, where an unweighted mean of an intensive quantity is the
/// recorded variance error. Ranking elements by an intensive field
/// aggregates nothing — it orders — so the weighted-mean obligation has
/// nothing to attach to, and this function never consults a kind.
///
/// # Errors
///
/// [`TypeError`] carrying [`TypeCode::NonComparableScore`].
pub fn check_selection_scores(
    expr: &SExpr,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<(), TypeError> {
    walk_typed(expr, env, bindings, &HashMap::new(), Check::Selections)
}

/// D67: **references compare by identity, with `=` and `!=` only.**
/// Comparing a reference with an ordering operator, with a reference of a
/// different kind, or with any non-reference is `E-TYPE-017`.
///
/// There is no ordering on references *in the language*: §2.6's iteration
/// order is the executor's, and exposing it as a comparison would invite
/// content to depend on id assignment.
///
/// # Errors
///
/// [`TypeError`] carrying [`TypeCode::BadReferenceComparison`].
pub fn check_reference_comparisons(
    expr: &SExpr,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<(), TypeError> {
    walk_typed(expr, env, bindings, &HashMap::new(), Check::References)
}

/// **§2.13's no-arithmetic law (D101), the static half (D118, #528 fix
/// round Item C).** `Enum<T>` supports no arithmetic (§2.13, §3.1) — an
/// `add`/`sub`/`scale` `update-op` targeting an `:enum-type`-declared
/// field is a coherence violation the field's declared type and the
/// op's own symbol already decide, in full, from content alone. Before
/// this function the only guards were THREE eval-time ones
/// (`structural_verbs.rs::refuse_arithmetic_on_enum_field`, `c268b83b`),
/// so a rule shaped exactly like this check's own red-test content
/// loaded clean and died mid-tick on the first admitted subject — the
/// same "always-wrong construct deferred to a runtime surprise" shape
/// D102's own field-of deferral named for its sibling gap (D102 itself
/// was discharged by the Task 1 P27 territory-port train — `field-of`
/// over an enum-declared field now typechecks and evaluates for real,
/// `evaluator::field_of_node` — but the static-decidability argument this
/// function makes stands on its own, unchanged), and the same argument as
/// `rule_pipeline.rs`'s own load-time wiring generally: §3's own law is
/// "every check in this chapter runs at content load, before any tick
/// executes."
///
/// The three eval-time guards STAY, unchanged, as defense in depth
/// (the same two-site discipline `refuse_arithmetic_on_enum_field`'s own
/// doc already names for its three call sites) — this function only
/// moves the FIRST catch earlier, from the first admitted subject to
/// load.
///
/// # Errors
///
/// A structural [`TypeError`] (`code: None` — E-EVAL-042 already covers
/// this refusal at evaluation, D118, so this is a static-decidability
/// repair, not a new failure class).
pub fn check_no_arithmetic_on_enum_field(expr: &SExpr, env: &TypeEnv) -> Result<(), TypeError> {
    if let SExpr::List(items) = expr {
        if let [SExpr::Atom(Atom::Symbol(head)), _node, SExpr::Atom(Atom::QName(qname)), SExpr::List(op_items)] =
            items.as_slice()
        {
            if head == "update-node" {
                if let [SExpr::Atom(Atom::Symbol(op)), _operand] = op_items.as_slice() {
                    if matches!(op.as_str(), "add" | "sub" | "scale") {
                        if let Some(decl) = env.fields.get(qname) {
                            if matches!(decl.ty, BslType::Enum(_)) {
                                return Err(TypeError {
                                    code: None,
                                    message: format!(
                                        "update-node {qname}: ({op} …) is not a coherent \
                                         operation on an enum-typed field — Enum<T> \
                                         supports no arithmetic (§2.13, D118); only `set` \
                                         may write it. Statically decidable from the \
                                         field's declared type and this op's own symbol \
                                         — refused here, at load, rather than left to die \
                                         mid-tick on the first admitted subject (§3's own \
                                         law: every check in this chapter runs at content \
                                         load)"
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
        for child in items {
            check_no_arithmetic_on_enum_field(child, env)?;
        }
    }
    Ok(())
}

// ---- §3.4's expression-kind arm: `<arith>` and `if` (#491 T1, E-TYPE-040) ----

/// The coarse **kind** of an `<expr>` — §3.4's `extensive`/`intensive`
/// axis, widened with the one state no *field* may declare:
/// **kind-neutral** (a literal, a `:const` binding — "a coefficient has no
/// extent"). [`crate::types::FieldKind`] stays field-scoped on purpose
/// (every `deffield` picks intensive or extensive, never neutral); this
/// enum is expression-scoped, where neutral is real.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprKind {
    /// A literal or a `:const` binding — no extent to mix with anything.
    Neutral,
    /// Carries a `deffield`'s (or `field-of`'s) declared extensive kind.
    Extensive,
    /// Carries a `deffield`'s (or `field-of`'s) declared intensive kind.
    Intensive,
}

/// Build the `E-TYPE-040` error for one `<arith>` node.
fn kind_mixing_error(op: &str, left: ExprKind, right: ExprKind) -> TypeError {
    TypeError {
        code: Some(TypeCode::KindMixing),
        message: format!(
            "E-TYPE-040: ({op} …) mixes {left:?}-kinded and {right:?}-kinded operands — \
             §3.4 requires the same kind, or one kind-neutral (the licensed same-kind \
             cases are extensive ÷ extensive, intensive × intensive, and intensive ÷ \
             intensive — D90/D181/D182/D183's unit algebra: w̄ = wealth ÷ population \
             is the definition of an intensive quantity)"
        ),
    }
}

/// `+`/`-` (§3.4): both operands share a kind, or one is kind-neutral — the
/// result carries the non-neutral kind. Mixing intensive with extensive is
/// `E-TYPE-040`. Same-kind is always legal here (unlike `*`/`/` below):
/// addition never creates the "area of an area" concern multiplication
/// does.
fn add_sub_kind(op: &str, left: ExprKind, right: ExprKind) -> Result<ExprKind, TypeError> {
    match (left, right) {
        (ExprKind::Neutral, ExprKind::Neutral) => Ok(ExprKind::Neutral),
        (ExprKind::Neutral, k) | (k, ExprKind::Neutral) => Ok(k),
        (a, b) if a == b => Ok(a),
        _ => Err(kind_mixing_error(op, left, right)),
    }
}

/// `*`/`/` (§3.4, split by this repair): kind-neutral absorbs either way.
/// Same-kind-squared is `E-TYPE-040` for `*` (an area-of-an-area) — the
/// case that stays — but is LICENSED as intensive for `/` when both
/// operands are extensive: `w̄ = wealth ÷ population`, D90's own
/// "unit algebra, not new mathematics" standing extended to its symmetric
/// gap.
///
/// **Extensive × intensive is legal, result extensive** (found reading
/// real content while gating this repair, not in the plan): `lifecycle.bsl`
/// computes `(binding new-wealth-d-prime :expr (* wealth-d-prime
/// surviving-fraction))` — a Currency stock times a computed fraction, the
/// same "total = rate × count" shape as `E-TYPE-030`'s own worked example
/// (`base_subsistence × population`, §8/T4). Refusing it conservatively,
/// as this module's first draft did, broke committed content that the
/// task's own "expect a real find" framing anticipated finding SOMETHING,
/// just not this: multiplying a stock by a dimensionless fraction/rate is
/// not an area-of-an-area, it is the ordinary way an extensive quantity
/// gets scaled. `*` is commutative for this axis (kind carries no operand
/// order), so extensive ÷ intensive is unreachable — division is NOT
/// commutative, and only `*`'s two operand positions are symmetric here.
///
/// **Intensive × intensive is legal, result intensive** (controller
/// adjudication, 2026-08-18, delegated Director provenance — morning-
/// reviewable — the third instance of this defect class, and the one the
/// Director had already ruled repair-now on: this instance's own correct
/// repair sits in the ARM, not the content). `consciousness.bsl`'s
/// `p6-route` computes `delta-r = (* (* consumed eff-sol) routing-scale)`:
/// `consumed` (`agitation × consumption-rate`) and `eff-sol` (a
/// solidarity/chauvinist-derived ratio) are both intensive, and the
/// product feeds `r1 = (+ r delta-r)`, where `r` — `social-class/
/// revolutionary` — is itself declared intensive, so the consumer already
/// expects an intensive result. This is the standard dimensional rule: a
/// rate scaled by a dimensionless coefficient stays a rate. Value-
/// preserving — no arithmetic changes anywhere this licenses, only the
/// kind computed for an expression that was always going to evaluate the
/// same way.
///
/// **Intensive ÷ intensive is ALSO legal, result intensive** — the fourth
/// straddle site (controller adjudication, 2026-08-18, delegated Director
/// provenance, same sitting, same narrow style: division only, nothing
/// else). Licensing the `*` arm above exposed `p6-route`'s OWN next site:
/// `r2`/`l2`/`f2`'s simplex renormalization (`(/ r1 total)` and siblings),
/// where `r1`/`l1`/`f1`/`total` are all intensive (each `r`/`l`/`f` plus
/// the now-licensed `delta-r`/`delta-l`/`delta-f`, summed). A ratio of two
/// intensive quantities is a dimensionless share — simplex normalization
/// (dividing a part by a whole built from parts of the SAME kind) is the
/// canonical intensive operation this coarse two-kind algebra has a name
/// for. Value-preserving, same reasoning as the product case, same
/// defect class — confirmed by a complete static sweep of every
/// `<arith>`/`if` site across all 13 committed rule files (not just
/// `p6-route`) finding NO other site this arm's current shape would still
/// refuse.
///
/// Every combination still unlicensed after this correction — extensive
/// mixed with intensive under `/` specifically (leg (e): an intensive
/// numerator over an extensive denominator must not silently become
/// extensive) — stays conservatively refused, matching the bullet's own
/// "deliberately conservative, Phase-1 review item" framing for the one
/// case it did name, extended (not invented) to the cases it never named.
fn mul_div_kind(op: &str, left: ExprKind, right: ExprKind) -> Result<ExprKind, TypeError> {
    match (left, right) {
        (ExprKind::Neutral, ExprKind::Neutral) => Ok(ExprKind::Neutral),
        (ExprKind::Neutral, k) | (k, ExprKind::Neutral) => Ok(k),
        (ExprKind::Extensive, ExprKind::Extensive) if op == "/" => Ok(ExprKind::Intensive),
        (ExprKind::Extensive, ExprKind::Intensive) | (ExprKind::Intensive, ExprKind::Extensive)
            if op == "*" =>
        {
            Ok(ExprKind::Extensive)
        }
        // `*` and `/` both land here now (2026-08-18: the product ruling,
        // then the division ruling, same sitting) — no `op` guard needed,
        // unlike the extensive-squared cell above, since both operators
        // now agree on this pair's result.
        (ExprKind::Intensive, ExprKind::Intensive) => Ok(ExprKind::Intensive),
        _ => Err(kind_mixing_error(op, left, right)),
    }
}

/// A field's declared kind, widened to [`ExprKind`] — `None` for an
/// `:enum-type`-declared field (`FieldKind::NotApplicable`, D101): §2.13
/// gives it no arithmetic at all (D118), so it carries no kind for this
/// axis to speak of, and `None` here declines to constrain rather than
/// invent one.
fn field_kind(env: &TypeEnv, qname: &str) -> Option<ExprKind> {
    match env.fields.get(qname)?.kind {
        FieldKind::Extensive => Some(ExprKind::Extensive),
        FieldKind::Intensive => Some(ExprKind::Intensive),
        FieldKind::NotApplicable => None,
    }
}

/// A bound symbol's kind, resolved through its declared source (§2.5).
/// `None` for a source this pass cannot yet resolve a kind for —
/// `:metric`/calendar reads — rather than guessing (see this module's
/// header doc: a disclosed Phase-1 gap, not a silent pass).
fn symbol_kind(
    name: &str,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<Option<ExprKind>, TypeError> {
    let Some(decl) = bindings.iter().find(|d| d.name == name) else {
        return Ok(None);
    };
    match &decl.source {
        crate::bindings::BindSource::Field(qname) => Ok(field_kind(env, qname)),
        crate::bindings::BindSource::Const(_) => Ok(Some(ExprKind::Neutral)),
        crate::bindings::BindSource::Metric(_)
        | crate::bindings::BindSource::Tick
        | crate::bindings::BindSource::Year
        | crate::bindings::BindSource::TickOfYear
        | crate::bindings::BindSource::TickInCycle(_) => Ok(None),
        crate::bindings::BindSource::Expr(inner) => expr_kind(inner, env, bindings),
    }
}

/// `(field-of <ref> <qname>)` (§2.10): carries the declaration's kind,
/// identically to a `:field` binding (§3.4).
fn field_of_kind(items: &[SExpr], env: &TypeEnv) -> Option<ExprKind> {
    match items.get(2) {
        Some(SExpr::Atom(Atom::QName(qname))) => field_kind(env, qname),
        _ => None,
    }
}

/// `if` (§3.4): both branches must share a kind. Only RAISES (its own
/// `Err`) when BOTH branches resolve to a determined, differing kind.
/// **Review finding F8 (#491 T1):** when only one branch is determined,
/// this function RETURNS that branch's kind rather than `None` — it
/// propagates partial information upward instead of declining, which
/// could in principle manufacture a refusal at an *enclosing* node from
/// information this node itself never confirmed. Unreachable today (only
/// `:metric` bindings and calendar reads ever yield `None`, and the
/// complete static sweep — #491 T1, D183 — found no site where that
/// matters), and `list_kind`'s own `+`/`-`/`*`/`/` handling correctly
/// declines on any undetermined operand regardless. Recorded here rather
/// than fixed because fixing it would need a live counter-example to
/// choose the right behavior, not a specification default.
fn if_kind(
    items: &[SExpr],
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<Option<ExprKind>, TypeError> {
    let (Some(then_expr), Some(else_expr)) = (items.get(2), items.get(3)) else {
        return Ok(None); // malformed arity — E-PARSE's business, not this pass's
    };
    let then_kind = expr_kind(then_expr, env, bindings)?;
    let else_kind = expr_kind(else_expr, env, bindings)?;
    match (then_kind, else_kind) {
        // Kind-neutral absorbs on EITHER branch, exactly as it does for
        // `+`, `-` and `*` — the "a literal is kind-neutral" propagation
        // rule is a cross-cutting principle (§3.4's own preamble states it
        // before ANY operator-specific bullet), not a `+`/`-`-only carve
        // out. Real content needs this: `lifecycle.bsl`'s
        // `surviving-fraction` binding is `(if (and …) (- 1 (/ deaths
        // pop-d-prime)) (- 1 0c))` — the then-branch resolves Intensive
        // (`1 - (extensive ÷ extensive)`, D90/D181), the else-branch is a
        // canonical-zero identity fallback, `(- 1 0c)`, both operands
        // literal and so kind-neutral throughout. A strict "branches must
        // match exactly" reading would have rejected this genuine,
        // committed rule; nothing in §3.4's `if` bullet forces that
        // reading, and the preamble's own neutral rule forbids it.
        (Some(ExprKind::Neutral), Some(k)) | (Some(k), Some(ExprKind::Neutral)) => Ok(Some(k)),
        (Some(a), Some(b)) if a == b => Ok(Some(a)),
        (Some(a), Some(b)) => Err(TypeError {
            code: Some(TypeCode::KindMixing),
            message: format!(
                "E-TYPE-040: (if …) branches disagree in kind — the then-branch is \
                 {a:?}, the else-branch is {b:?}; §3.4 requires both branches to share \
                 a kind, or one to be kind-neutral"
            ),
        }),
        (Some(a), None) | (None, Some(a)) => Ok(Some(a)),
        (None, None) => Ok(None),
    }
}

/// A fold's own result kind (§3.4's per-operator table), computed
/// independently of [`typecheck_aggregation`] — this function does not
/// touch, call into or restructure that fold arm; it exists so an `<arith>`
/// operand that is itself a `(fold …)` (T1 leg (e): a weighted intensive
/// `mean` divided by an extensive field) propagates the RIGHT kind rather
/// than being silently skipped. `sum` and `count` both always result
/// extensive (row 1 and row 6: the only legal `sum` body kind IS extensive,
/// and `count` is `Int`, extensive, regardless of body); `mean`/`min`/`max`
/// carry the body's own kind unchanged (row 2's weighted-intensive case,
/// D90, included — the body's kind IS intensive there, and the result is
/// documented as intensive too, so "carries the body kind" already covers
/// it without a special case).
fn fold_kind(
    items: &[SExpr],
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<Option<ExprKind>, TypeError> {
    let Some(SExpr::Atom(Atom::Symbol(op))) = items.get(1) else {
        return Ok(None);
    };
    let Some(fold_op) = crate::grammar::FoldOp::parse(op) else {
        return Ok(None);
    };
    match fold_op {
        crate::grammar::FoldOp::Count | crate::grammar::FoldOp::Sum => {
            Ok(Some(ExprKind::Extensive))
        }
        crate::grammar::FoldOp::Mean
        | crate::grammar::FoldOp::Min
        | crate::grammar::FoldOp::Max => {
            // The body follows the query, optionally after `:as <symbol>` —
            // mirrors `score_class::classify_fold`'s own body-location
            // logic exactly (same grammar, same shape).
            let body = match items.get(3) {
                Some(SExpr::Atom(Atom::Keyword(kw))) if kw == "as" => items.get(5),
                other => other,
            };
            match body {
                Some(b) => expr_kind(b, env, bindings),
                None => Ok(None),
            }
        }
    }
}

/// Classify one `<arith>`/`if`/`field-of`/`fold` form's kind. Total over
/// the constructs §3.4 names; every other form (comparisons, `and`/`or`,
/// queries, `select-max`/`select-min`, …) is not kind-bearing and declines
/// (`None`) rather than guesses.
fn list_kind(
    items: &[SExpr],
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<Option<ExprKind>, TypeError> {
    if let Some(SExpr::Atom(Atom::Operator(op))) = items.first() {
        let rule: fn(&str, ExprKind, ExprKind) -> Result<ExprKind, TypeError> = match op.as_str() {
            "+" | "-" => add_sub_kind,
            "*" | "/" => mul_div_kind,
            _ => return Ok(None), // the six comparisons: Bool, not kind-bearing
        };
        let (Some(lhs), Some(rhs)) = (items.get(1), items.get(2)) else {
            return Ok(None); // malformed arity — E-PARSE-040's business, not this pass's
        };
        return match (
            expr_kind(lhs, env, bindings)?,
            expr_kind(rhs, env, bindings)?,
        ) {
            (Some(l), Some(r)) => rule(op, l, r).map(Some),
            _ => Ok(None), // one side undetermined — decline rather than guess
        };
    }
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(head))) if head == "if" => if_kind(items, env, bindings),
        Some(SExpr::Atom(Atom::Symbol(head))) if head == "field-of" => {
            Ok(field_of_kind(items, env))
        }
        Some(SExpr::Atom(Atom::Symbol(head))) if head == "fold" => fold_kind(items, env, bindings),
        _ => Ok(None),
    }
}

/// Classify one expression's kind (§3.4). Total, fallible: `Ok(None)` when
/// this pass declines to constrain (an undetermined source, or a
/// non-kind-bearing construct); `Err` only for a PROVEN `E-TYPE-040`
/// violation.
fn expr_kind(
    expr: &SExpr,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<Option<ExprKind>, TypeError> {
    match expr {
        SExpr::Atom(Atom::Int(_) | Atom::Currency(_) | Atom::Scaled(_) | Atom::Bool(_)) => {
            Ok(Some(ExprKind::Neutral))
        }
        SExpr::Atom(Atom::Symbol(name)) => symbol_kind(name, env, bindings),
        SExpr::Atom(_) => Ok(None),
        SExpr::List(items) => list_kind(items, env, bindings),
    }
}

/// Walk the whole rule tree and validate every `<arith>`/`if` node's kind
/// (§3.4) — independently at each occurrence a node is visited, which is
/// deliberately redundant with `expr_kind`'s own recursion into an
/// enclosing arith/if's operands: this walker is what reaches an arith/if
/// node that is NOT nested inside another kind-checked expression (a
/// `guard` condition, an `update-node` operand, a `select-max` score, a
/// compound fold body, …), since `expr_kind` alone only ever sees what its
/// caller hands it.
///
/// Extends the crate's existing typecheck dispatch (the sequence
/// `rule_pipeline` composes, alongside [`check_selection_scores`],
/// [`check_reference_comparisons`] and [`check_no_arithmetic_on_enum_field`])
/// — it does not touch, call into or restructure [`typecheck_aggregation`],
/// the fold arm.
///
/// # Errors
///
/// [`TypeError`] carrying [`TypeCode::KindMixing`] (`E-TYPE-040`) for a
/// proven violation.
pub fn check_kind_mixing(
    expr: &SExpr,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
) -> Result<(), TypeError> {
    if let SExpr::List(items) = expr {
        let is_kinded_form = matches!(
            items.first(),
            Some(SExpr::Atom(Atom::Operator(op))) if matches!(op.as_str(), "+" | "-" | "*" | "/")
        ) || matches!(
            items.first(),
            Some(SExpr::Atom(Atom::Symbol(head))) if head == "if"
        );
        if is_kinded_form {
            expr_kind(expr, env, bindings)?;
        }
        for child in items {
            check_kind_mixing(child, env, bindings)?;
        }
    }
    Ok(())
}

/// Which §2 rule the shared walker is applying. Both need the same thing —
/// the classes of the element names in scope at each node — and computing
/// that twice in two walkers is how the empty-map defect survived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Check {
    Selections,
    References,
}

const ORDERING_OPERATORS: [&str; 4] = ["<", "<=", ">", ">="];

/// The element names an iterating or query form puts in scope for the
/// children *inside* it, and the class each denotes.
///
/// `it` always denotes the **innermost** enclosing element (D53), so an
/// inner form's insertion overwrites an outer one's. A `:as` name persists
/// through nested bodies (D54), which falls out of carrying the map down.
fn element_bindings_of(items: &[SExpr]) -> HashMap<String, ScoreClass> {
    let mut out = HashMap::new();
    let head = match items.first() {
        Some(SExpr::Atom(Atom::Symbol(s))) => s.as_str(),
        _ => return out,
    };
    // A query form's own predicate ranges over that query's elements; an
    // iterating form's body ranges over its `<query>` operand's.
    let element_class = if crate::scope::is_query(items) {
        selection_result_class(&SExpr::List(items.to_vec()))
    } else {
        match crate::scope::iterating_query_index(head).and_then(|i| items.get(i)) {
            Some(query) => selection_result_class(query),
            None => return out,
        }
    };
    out.insert("it".to_owned(), element_class);
    if let Some(name) = elem_name(items) {
        out.insert(name.to_owned(), element_class);
    }
    out
}

/// The `:as <symbol>` name a form declares, if any.
fn elem_name(items: &[SExpr]) -> Option<&str> {
    let mut i = 1;
    while i + 1 < items.len() {
        if let SExpr::Atom(Atom::Keyword(kw)) = &items[i] {
            if kw == "as" {
                if let SExpr::Atom(Atom::Symbol(name)) = &items[i + 1] {
                    return Some(name);
                }
            }
        }
        i += 1;
    }
    None
}

/// Whether child `index` sits inside the element scope `items` introduces.
///
/// Delegates to [`crate::scope::child_is_inside`] — the crate's single
/// source of truth for "a query's element predicate is a body, its element
/// operand is not". This module used to hardcode its own copy of the rule,
/// which is three implementations of one clause and exactly the shape that
/// lets a future query head diverge silently in one of them.
fn child_is_inside(items: &[SExpr], index: usize) -> bool {
    crate::scope::child_is_inside(items, index)
}

fn walk_typed(
    expr: &SExpr,
    env: &TypeEnv,
    bindings: &[crate::bindings::BindingDecl],
    element_names: &HashMap<String, ScoreClass>,
    check: Check,
) -> Result<(), TypeError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    let class_env = ClassEnv {
        bindings,
        fields: &env.fields,
        element_names,
    };
    match check {
        Check::Selections => check_one_selection(items, &class_env)?,
        Check::References => check_one_comparison(items, &class_env)?,
    }
    let introduced = element_bindings_of(items);
    for (index, child) in items.iter().enumerate() {
        if index > 0 && child_is_inside(items, index) && !introduced.is_empty() {
            let mut inner = element_names.clone();
            inner.extend(introduced.clone());
            walk_typed(child, env, bindings, &inner, check)?;
        } else {
            walk_typed(child, env, bindings, element_names, check)?;
        }
    }
    Ok(())
}

fn check_one_selection(items: &[SExpr], env: &ClassEnv<'_>) -> Result<(), TypeError> {
    let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
        return Ok(());
    };
    if head != "select-max" && head != "select-min" {
        return Ok(());
    }
    // The score is the last operand, after the query and an optional
    // `:as <symbol>`. It is evaluated INSIDE the selection's element scope,
    // so it classifies against the element bindings this form introduces.
    let score = match items.get(2) {
        Some(SExpr::Atom(Atom::Keyword(kw))) if kw == "as" => items.get(4),
        other => other,
    };
    let Some(score) = score else {
        return Err(TypeError {
            code: None,
            message: format!("({head} <query> <elem-name>? <expr>) — missing score"),
        });
    };
    let mut inner = env.element_names.clone();
    inner.extend(element_bindings_of(items));
    let scoped = ClassEnv {
        bindings: env.bindings,
        fields: env.fields,
        element_names: &inner,
    };
    let class = classify(score, &scoped);
    if class.is_comparable_scalar() {
        return Ok(());
    }
    Err(TypeError {
        code: Some(TypeCode::NonComparableScore),
        message: format!(
            "E-TYPE-016: a {head} score must be a comparable scalar (Int, \
             Currency, Probability, Intensity, Coefficient or Real); this one \
             classifies as {class:?} (§2.7)"
        ),
    })
}

fn check_one_comparison(items: &[SExpr], env: &ClassEnv<'_>) -> Result<(), TypeError> {
    let [SExpr::Atom(Atom::Operator(op)), lhs, rhs] = items else {
        return Ok(());
    };
    let (left, right) = (classify(lhs, env), classify(rhs, env));
    if !(left.is_reference() || right.is_reference()) {
        return Ok(());
    }
    let legal = matches!(op.as_str(), "=" | "!=")
        && left.is_reference()
        && right.is_reference()
        && left == right;
    if legal {
        return Ok(());
    }
    let why = if ORDERING_OPERATORS.contains(&op.as_str()) {
        "there is no ordering on references (§2.4)"
    } else if !left.is_reference() || !right.is_reference() {
        "a reference compares only against a reference (§2.4)"
    } else {
        "a reference compares only against one of the SAME kind (§2.4)"
    };
    Err(TypeError {
        code: Some(TypeCode::BadReferenceComparison),
        message: format!("E-TYPE-017: ({op} {left:?} {right:?}) — {why}"),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        check_kind_mixing, check_reference_comparisons, check_selection_scores,
        typecheck_aggregation, TypeCode, TypeEnv,
    };
    use crate::bindings::{BindSource, BindingDecl};
    use crate::exemptions::IntensiveAggregationExemption;
    use crate::reader::read;
    use crate::types::{BslType, FieldDecl, FieldKind};
    use std::collections::HashMap;

    fn env() -> TypeEnv {
        let mut fields = HashMap::new();
        fields.insert(
            "wealth-share".to_string(),
            FieldDecl {
                ty: BslType::Coefficient,
                kind: FieldKind::Intensive,
            },
        );
        fields.insert(
            "consciousness".to_string(),
            FieldDecl {
                ty: BslType::Intensity,
                kind: FieldKind::Intensive,
            },
        );
        fields.insert(
            "population".to_string(),
            FieldDecl {
                ty: BslType::Int,
                kind: FieldKind::Extensive,
            },
        );
        fields.insert(
            "wealth".to_string(),
            FieldDecl {
                ty: BslType::Currency,
                kind: FieldKind::Extensive,
            },
        );
        // #551 closure (Task 2, P27 territory-port train): the NotApplicable
        // (enum-typed) kind cell the legality table below now asserts
        // instead of declining. `EnumTypeId(0)` is a bare id, not a real
        // registry entry — `typecheck_aggregation`'s #551 check only reads
        // `field.kind`, never resolving the id through an `EnumRegistry`.
        fields.insert(
            "org-kind".to_string(),
            FieldDecl {
                ty: BslType::Enum(crate::types::EnumTypeId(0)),
                kind: FieldKind::NotApplicable,
            },
        );
        TypeEnv {
            fields,
            exemptions: &[],
        }
    }

    fn check(source: &str, env: &TypeEnv) -> Result<BslType, super::TypeError> {
        let (expr, _) = read(source).expect("test source should parse");
        typecheck_aggregation(&expr, env)
    }

    fn code_of(source: &str, env: &TypeEnv) -> Option<TypeCode> {
        check(source, env).expect_err("should be a type error").code
    }

    // ---- the five-row table (§3.4) ----

    #[test]
    fn unweighted_mean_of_an_intensive_field_is_e_type_042() {
        // The recorded variance error, caught at load.
        assert_eq!(
            code_of("(mean wealth-share)", &env()),
            Some(TypeCode::UnweightedMeanOfIntensive)
        );
    }

    #[test]
    fn mean_of_an_intensive_field_with_an_extensive_weight_is_legal() {
        assert_eq!(
            check("(mean wealth-share :weight population)", &env()),
            Ok(BslType::Coefficient)
        );
    }

    #[test]
    fn a_weight_that_is_not_extensive_is_e_type_043() {
        // consciousness is intensive — weighting by it is the same error
        // one level up.
        assert_eq!(
            code_of("(mean wealth-share :weight consciousness)", &env()),
            Some(TypeCode::NonExtensiveWeight)
        );
    }

    #[test]
    fn sum_of_an_intensive_field_is_e_type_041_even_with_a_weight() {
        // §3.4: summing an intensive quantity is meaningless — no weight
        // rescues it (deviation from the plan sketch, recorded above).
        assert_eq!(
            code_of("(sum wealth-share)", &env()),
            Some(TypeCode::SumOfIntensive)
        );
        assert_eq!(
            code_of("(sum wealth-share :weight population)", &env()),
            Some(TypeCode::SumOfIntensive)
        );
    }

    #[test]
    fn sum_of_an_extensive_field_is_legal() {
        assert_eq!(check("(sum wealth)", &env()), Ok(BslType::Currency));
    }

    #[test]
    fn unweighted_mean_of_an_extensive_field_is_legal() {
        assert_eq!(check("(mean wealth)", &env()), Ok(BslType::Currency));
    }

    #[test]
    fn min_and_max_are_kind_neutral_and_always_legal() {
        // The plan's sketch would have wrongly rejected these (§3.4 row 5).
        assert_eq!(
            check("(min wealth-share)", &env()),
            Ok(BslType::Coefficient)
        );
        assert_eq!(check("(max consciousness)", &env()), Ok(BslType::Intensity));
    }

    #[test]
    fn count_is_always_legal_and_returns_int() {
        assert_eq!(check("(count wealth-share)", &env()), Ok(BslType::Int));
    }

    /// Train B item 6 (#591): a fold-sum over a `real extensive` field
    /// types as the fold's numeric lane — the field's own `Real`, the same
    /// pass-through every other storable numeric type already gets — and
    /// never raises E-TYPE-041 (the sum-of-intensive refusal), because the
    /// kind law keys on `FieldKind`, not on the type.
    #[test]
    fn sum_over_a_real_extensive_field_types_as_the_numeric_lane() {
        let mut fields = HashMap::new();
        fields.insert(
            "balance".to_string(),
            FieldDecl {
                ty: BslType::Real,
                kind: FieldKind::Extensive,
            },
        );
        let env = TypeEnv {
            fields,
            exemptions: &[],
        };
        assert_eq!(check("(sum balance)", &env), Ok(BslType::Real));
    }

    /// Compile-time trap for the `FieldKind` axis (verifier fix round,
    /// MINOR-2 on issue #525). `typecheck_aggregation` itself decides the
    /// kind law with per-variant EQUALITY checks (`field.kind ==
    /// FieldKind::Intensive`, `w.kind != FieldKind::Extensive`) — a 4th
    /// `FieldKind` variant would compile cleanly there and pass through
    /// every one of those checks silently, the exact silent-widening shape
    /// A3 exists to prevent on the `FoldOp` axis. This function is NOT a
    /// production fix for that (a real fix would need `typecheck_
    /// aggregation` itself rewritten as an exhaustive match, out of scope
    /// for a doc/test fix round) — it is a TRIP-WIRE: an exhaustive match
    /// over `FieldKind`, no wildcard, that breaks compilation THE MOMENT a
    /// 4th variant lands, at this test, forcing a human to reconsider the
    /// table below before it can even build.
    fn field_kind_is_exhaustively_named(kind: FieldKind) -> &'static str {
        match kind {
            FieldKind::Intensive => "intensive",
            FieldKind::Extensive => "extensive",
            FieldKind::NotApplicable => "not-applicable (enum-typed field)",
        }
    }

    /// CT4P A3 (issue #525), honesty correction (verifier fix round,
    /// MINOR-2). **What is actually compiler-enforced, and what is not:**
    /// the `FoldOp` axis IS — `typecheck_aggregation`'s own `match fold_op`
    /// has no wildcard, so a 6th `FoldOp` variant is a compile error there,
    /// in production code, full stop. The `FieldKind` axis is NOT —
    /// `typecheck_aggregation` decides kind with per-variant equality
    /// checks, not an exhaustive match, so a 4th `FieldKind` variant would
    /// compile and silently pass every check unchanged. This table can only
    /// TRAP that axis, via [`field_kind_is_exhaustively_named`] above,
    /// which is called once per row below — not make production exhaustive
    /// over it.
    ///
    /// 5 fold-ops × the three `FieldKind` variants — **all 15 real
    /// `(FoldOp, FieldKind)` cells**, closing the table `#551` left
    /// deliberately incomplete. `Intensive`/`Extensive` use `wealth-share`/
    /// `wealth` from `env()`, the SAME pair the individual tests above
    /// already use; `NotApplicable` (an enum-typed field, §2.13/D101) uses
    /// `org-kind`, also from `env()`.
    ///
    /// **#551 closure (Task 2, P27 territory-port train):** the 5
    /// `NotApplicable` rows below used to be DECLINED — not asserted
    /// either way, because `typecheck_aggregation` never special-cased
    /// `NotApplicable`, so every op silently reached `Ok` for one, and
    /// that was an observation about the code, not a law this table
    /// pinned. `(fold <op> …)` over a `:field`-bound enum symbol (or,
    /// since Task 1's D102 discharge, a `field-of` accessor) passing the
    /// KIND LAW silently was filed as **issue #551** (full reachability
    /// trace there; end-to-end proof through the real `load_rule` pipeline,
    /// both routes, in `rule_pipeline::enum_fold_body_tests`). Now DECIDED:
    /// `sum`/`mean`/`min`/`max` all refuse (`E-TYPE-044`, `TypeCode::
    /// EnumFoldBody`) — each of those ops evaluates its body, and `Enum<T>`
    /// supports neither arithmetic (sum/mean) nor ordering (min/max, §3.1).
    /// `count` alone stays legal (§3.4 row 6's existing "always legal, kind
    /// irrelevant" law): it never evaluates its body at all, so an
    /// enum-declared name there is inert, not a content error — the
    /// narrower verdict the closure's own tracking issue asked the
    /// implementer to weigh. Each row below states its OWN accept/refuse
    /// verdict; nothing here infers one row from another.
    #[test]
    fn fold_op_x_field_kind_legality_table() {
        use crate::grammar::FoldOp;
        let table: [(FoldOp, &str, FieldKind, bool); 15] = [
            (FoldOp::Sum, "wealth-share", FieldKind::Intensive, false), // E-TYPE-041
            (FoldOp::Sum, "wealth", FieldKind::Extensive, true),
            (FoldOp::Sum, "org-kind", FieldKind::NotApplicable, false), // E-TYPE-044 (#551)
            (FoldOp::Mean, "wealth-share", FieldKind::Intensive, false), // E-TYPE-042 (unweighted)
            (FoldOp::Mean, "wealth", FieldKind::Extensive, true),
            (FoldOp::Mean, "org-kind", FieldKind::NotApplicable, false), // E-TYPE-044 (#551)
            (FoldOp::Min, "wealth-share", FieldKind::Intensive, true),   // kind-neutral
            (FoldOp::Min, "wealth", FieldKind::Extensive, true),
            (FoldOp::Min, "org-kind", FieldKind::NotApplicable, false), // E-TYPE-044 (#551): no ordering on Enum<T>
            (FoldOp::Max, "wealth-share", FieldKind::Intensive, true),  // kind-neutral
            (FoldOp::Max, "wealth", FieldKind::Extensive, true),
            (FoldOp::Max, "org-kind", FieldKind::NotApplicable, false), // E-TYPE-044 (#551): no ordering on Enum<T>
            (FoldOp::Count, "wealth-share", FieldKind::Intensive, true), // always legal
            (FoldOp::Count, "wealth", FieldKind::Extensive, true),
            (FoldOp::Count, "org-kind", FieldKind::NotApplicable, true), // narrower verdict: body never evaluated
        ];
        for (op, field, kind, expect_legal) in table {
            let kind_label = field_kind_is_exhaustively_named(kind);
            let source = format!("({} {field})", op.as_str());
            let result = check(&source, &env());
            assert_eq!(
                result.is_ok(),
                expect_legal,
                "{op:?} over {field} ({kind_label}, unweighted): expected legal={expect_legal}, got {result:?}"
            );
            if !expect_legal && kind == FieldKind::NotApplicable {
                assert_eq!(
                    code_of(&source, &env()),
                    Some(TypeCode::EnumFoldBody),
                    "{op:?} over {field}: expected E-TYPE-044"
                );
            }
        }
    }

    // ---- exemptions (§3.4: named-field suppression) ----

    const TEST_EXEMPTION: &[IntensiveAggregationExemption] = &[IntensiveAggregationExemption {
        field_name: "wealth-share",
        reason: "test-only: verifies the suppression path",
        owner: "test-suite",
        date: "2026-07-30",
    }];

    #[test]
    fn an_exemption_suppresses_the_law_for_its_named_field_only() {
        let mut exempted = env();
        exempted.exemptions = TEST_EXEMPTION;
        assert!(check("(mean wealth-share)", &exempted).is_ok());
        assert!(check("(sum wealth-share)", &exempted).is_ok());
        // A different intensive field still rejects.
        assert_eq!(
            code_of("(mean consciousness)", &exempted),
            Some(TypeCode::UnweightedMeanOfIntensive)
        );
    }

    // ---- structural errors (no spec code) ----

    #[test]
    fn an_unknown_field_is_a_loud_uncoded_error() {
        let err = check("(sum imperial-rent)", &env()).unwrap_err();
        assert_eq!(err.code, None);
        assert!(err.message.contains("imperial-rent"));
    }

    #[test]
    fn an_unknown_operator_is_a_loud_uncoded_error() {
        let err = check("(median wealth)", &env()).unwrap_err();
        assert_eq!(err.code, None);
    }

    #[test]
    fn a_malformed_form_is_a_loud_uncoded_error() {
        assert!(check("wealth", &env()).is_err());
        assert!(check("(mean)", &env()).is_err());
        assert!(check("(mean wealth-share :weight)", &env()).is_err());
    }

    // ---- §2.13's enum-typed fields (D101), the D102 field-of deferral
    // (discharged below) and D118's no-arithmetic law (Organization spec
    // §1 Q12) ----

    fn org_env() -> TypeEnv {
        let mut registry = crate::types::EnumRegistry::default();
        let ty = registry
            .declare(
                "OrgKind",
                &["STATE_APPARATUS".to_owned(), "BUSINESS".to_owned()],
            )
            .unwrap();
        TypeEnv {
            fields: HashMap::from([
                (
                    "organization/kind".to_owned(),
                    FieldDecl {
                        ty: BslType::Enum(ty),
                        kind: FieldKind::NotApplicable,
                    },
                ),
                (
                    "organization/budget".to_owned(),
                    FieldDecl {
                        ty: BslType::Currency,
                        kind: FieldKind::Extensive,
                    },
                ),
            ]),
            exemptions: &[],
        }
    }

    // ---- D102 discharge (Task 1, P27 territory-port train): field-of over
    // an enum-declared field now TYPECHECKS AS THE ENUM, not `Real`, and not
    // refused. `check_no_field_of_on_enum_field` (the unconditional D102
    // deferral gate) is deleted rather than narrowed: score-position (D46)
    // and arithmetic (D118/`apply_arith`) are each enforced by their OWN
    // independent mechanism below, so nothing was left for a third gate to
    // decide once the deferral itself lifted.

    /// `score_class::classify` is §2.7's total static classifier — the
    /// same one `check_selection_scores`/`check_reference_comparisons`
    /// consult — so this is the load-bearing proof that `field-of` over an
    /// enum-declared field types AS `Enum`, not `Real` and not `Unknown`,
    /// matching the SAME class a `:field` binding over the identical field
    /// already carries (§2.5's read parity, D102's whole point).
    #[test]
    fn field_of_over_an_enum_declared_field_typechecks_as_enum() {
        use crate::score_class::{classify, ClassEnv, ScoreClass};
        let (expr, _) = read("(field-of self organization/kind)").expect("must parse");
        let env = org_env();
        let class = classify(
            &expr,
            &ClassEnv {
                bindings: &[],
                fields: &env.fields,
                element_names: &HashMap::new(),
            },
        );
        assert_eq!(class, ScoreClass::Enum);
    }

    /// The walk recurses into `(and …)`/`(if …)`/… bodies (D53/D54's own
    /// element-scope rule already covers this at the `classify`/`walk_typed`
    /// level) — proven end to end via `check_reference_comparisons`, which
    /// shares the exact classifier: a nested `field-of` over an enum field
    /// classifies correctly deep inside another form, not just at the top.
    #[test]
    fn field_of_over_an_enum_field_nested_inside_another_form_still_classifies_as_enum() {
        let (expr, _) = read("(and #t (= (field-of self organization/kind) OrgKind/BUSINESS))")
            .expect("must parse");
        // A same-enum-type `=` comparison is D67-legal (both sides Enum, no
        // ordering operator involved) — this must load clean, proving the
        // nested field-of classified correctly rather than as `Unknown`
        // (which `check_reference_comparisons` would also accept, silently,
        // masking a classification regression).
        assert!(check_reference_comparisons(&expr, &org_env(), &[]).is_ok());
    }

    /// D46/`E-TYPE-016` STANDS: ranking by an enum-classed score is refused
    /// exactly as before — D102's discharge widens where `field-of` may
    /// legally APPEAR, not what `select-max`/`select-min` may legally SCORE
    /// BY.
    #[test]
    fn field_of_over_an_enum_field_as_a_select_max_score_still_refuses_e_type_016() {
        let (expr, _) =
            read("(select-max (nodes NodeType/ORGANIZATION) (field-of it organization/kind))")
                .expect("must parse");
        let err = check_selection_scores(&expr, &org_env(), &[]).unwrap_err();
        assert_eq!(err.code, Some(TypeCode::NonComparableScore));
        assert!(err.message.contains("E-TYPE-016"), "{}", err.message);
    }

    // ---- §2.13's no-arithmetic law, the static half (D118, #528 fix
    // round Item C) ----

    #[test]
    fn add_on_an_enum_declared_field_refuses_citing_d118() {
        let (expr, _) = read("(update-node self organization/kind (add 1))").expect("must parse");
        let err = super::check_no_arithmetic_on_enum_field(&expr, &org_env()).unwrap_err();
        assert_eq!(
            err.code, None,
            "D118 mints no error code — E-EVAL-042 already covers it"
        );
        assert!(err.message.contains("D118"), "{}", err.message);
        assert!(err.message.contains("organization/kind"), "{}", err.message);
    }

    #[test]
    fn sub_and_scale_on_an_enum_declared_field_both_refuse() {
        for op in ["sub", "scale"] {
            let source = format!("(update-node self organization/kind ({op} 1))");
            let (expr, _) = read(&source).expect("must parse");
            assert!(
                super::check_no_arithmetic_on_enum_field(&expr, &org_env()).is_err(),
                "{op} must refuse"
            );
        }
    }

    #[test]
    fn set_on_an_enum_declared_field_is_untouched() {
        // `set` is the ONE coherent op on an enum field (§2.13) — the
        // guard must never widen to refuse it.
        let (expr, _) = read("(update-node self organization/kind (set OrgKind/BUSINESS))")
            .expect("must parse");
        assert!(super::check_no_arithmetic_on_enum_field(&expr, &org_env()).is_ok());
    }

    #[test]
    fn add_on_a_non_enum_field_is_untouched() {
        let (expr, _) =
            read("(update-node self organization/budget (add 5$))").expect("must parse");
        assert!(super::check_no_arithmetic_on_enum_field(&expr, &org_env()).is_ok());
    }

    #[test]
    fn add_on_an_enum_field_nested_inside_a_guard_still_refuses() {
        // The walk must recurse into (guard/for-each/…) effect bodies, not
        // just the top-level effect item.
        let (expr, _) =
            read("(guard #t (update-node self organization/kind (add 1)))").expect("must parse");
        assert!(super::check_no_arithmetic_on_enum_field(&expr, &org_env()).is_err());
    }

    // ---- §3.4's expression-kind arm: `<arith>` and `if` (#491 T1,
    // `E-TYPE-040`, ADR202 R1(c)/OQ-I) ----
    //
    // **Mutation-verify (1.5), performed manually against this file at its
    // FINAL design (post real-content fixes), not committed:**
    //
    // (1) Mutated `mul_div_kind`'s `(Extensive, Extensive)` arm's guard
    // from `if op == "/"` to `if op == "MUTATED-NEVER"` (so NEITHER
    // operator licenses same-extensive-squared). Ran
    // `cargo test -p babylon-bsl --locked typecheck::tests`: 31 passed, 2
    // failed —
    // `extensive_divided_by_extensive_is_intensive_the_licensed_case`
    // (leg a) AND
    // `if_with_one_kind_neutral_branch_absorbs_matching_real_content` (which
    // also exercises `/` on two extensive fields inside its `if` branch).
    // Reverted; re-ran clean.
    //
    // (2) Mutated the same guard to `if op == "*" || op == "/"` (so `*`
    // WRONGLY also licenses same-extensive-squared as intensive). Ran the
    // same command: 32 passed, 1 failed —
    // `extensive_times_extensive_is_e_type_040_the_case_that_stays` (leg
    // b), which panicked calling `.unwrap_err()` on the now-`Ok(Intensive)`
    // result. Reverted; re-ran clean (the full crate suite, 794 tests,
    // confirmed green after revert — see the T1 report for the exact
    // command and count).
    //
    // (3) Controller adjudication, 2026-08-18 (delegated Director
    // provenance): mutated the NEW `(Intensive, Intensive) if op == "*"`
    // arm's guard to `if op == "MUTATED-NEVER"` (so intensive × intensive
    // stays refused even after this licensing). Ran `cargo test -p
    // babylon-bsl --locked typecheck::tests`: 34 passed, 1 failed —
    // `intensive_times_intensive_is_legal_result_intensive_matching_real_content`,
    // which asserted `Ok(Some(Intensive))` and got the E-TYPE-040 refusal
    // back instead — the SAME shape as the original (b)/(a) mutations
    // above, confirming the arm is load-bearing for the new case too.
    // Reverted; re-ran clean.
    //
    // (4) Controller adjudication, 2026-08-18, same sitting (delegated
    // Director provenance): once the `(Intensive, Intensive)` arm dropped
    // its `op` guard entirely (merging the `*`- and `/`-licensed cells),
    // split it back into `if op == "*"` plus a `MUTATED-NEVER`-guarded
    // second arm for `/` (so intensive ÷ intensive stays refused). Ran the
    // same command: 35 passed, 1 failed —
    // `intensive_divided_by_intensive_is_legal_result_intensive_matching_real_content`,
    // the E-TYPE-040 refusal returned exactly as expected. Reverted; the
    // arm's real (unguarded) shape re-ran clean.

    fn kind_bindings() -> Vec<BindingDecl> {
        let field = |name: &str, source: &str| BindingDecl {
            name: name.to_owned(),
            source: BindSource::Field(source.to_owned()),
            optional: false,
            default: None,
        };
        vec![
            field("ws", "wealth-share"),    // Intensive, Coefficient
            field("cons", "consciousness"), // Intensive, Intensity
            field("pop", "population"),     // Extensive, Int
            field("wealth-b", "wealth"),    // Extensive, Currency
        ]
    }

    fn kind_of(source: &str) -> Result<Option<super::ExprKind>, super::TypeError> {
        let (expr, _) = read(source).expect("test source should parse");
        super::expr_kind(&expr, &env(), &kind_bindings())
    }

    #[test]
    fn extensive_divided_by_extensive_is_intensive_the_licensed_case() {
        // (a): w̄ = wealth ÷ population — D90/D181's unit algebra, the
        // definition of an intensive quantity (density = mass ÷ volume).
        assert_eq!(
            kind_of("(/ wealth-b pop)"),
            Ok(Some(super::ExprKind::Intensive))
        );
    }

    #[test]
    fn extensive_times_extensive_is_e_type_040_the_case_that_stays() {
        // (b): an area-of-an-area — the one case the old combined bullet
        // already named, and the only one this repair does NOT relicense.
        let err = kind_of("(* wealth-b pop)").unwrap_err();
        assert_eq!(err.code, Some(TypeCode::KindMixing));
        assert!(err.message.contains("E-TYPE-040"), "{}", err.message);
    }

    #[test]
    fn intensive_plus_or_minus_extensive_is_e_type_040_the_plus_minus_bullet_also_unwritten() {
        // (c): the `+`/`-` bullet's own "mixing intensive with extensive is
        // E-TYPE-040" clause — unimplemented before this repair, same as
        // the `*`/`/` one.
        for op in ["+", "-"] {
            let source = format!("({op} ws pop)");
            let err = kind_of(&source).unwrap_err();
            assert_eq!(err.code, Some(TypeCode::KindMixing), "{op}: {err:?}");
            assert!(err.message.contains("E-TYPE-040"), "{op}: {}", err.message);
        }
    }

    #[test]
    fn if_branches_disagreeing_in_kind_is_e_type_040() {
        // (d): consciousness (intensive) vs. population (extensive).
        let err = kind_of("(if #t cons pop)").unwrap_err();
        assert_eq!(err.code, Some(TypeCode::KindMixing));
        assert!(err.message.contains("E-TYPE-040"), "{}", err.message);
    }

    #[test]
    fn if_with_one_kind_neutral_branch_absorbs_matching_real_content() {
        // Regression, found reading `lifecycle.bsl:303-306`'s real,
        // committed `surviving-fraction` binding while gating this repair:
        // `(if (and …) (- 1 (/ deaths pop-d-prime)) (- 1 0c))`. The
        // then-branch resolves Intensive (extensive ÷ extensive, D90/D181,
        // then `1 -` absorbs neutral); the else-branch, `(- 1 0c)`, is a
        // canonical-zero identity fallback over two literals — Neutral
        // throughout. A strict "branches must match" `if` rule would have
        // wrongly rejected this genuine content; kind-neutral absorbs on
        // either branch instead, exactly as it does for `+`, `-` and `*`.
        assert_eq!(
            kind_of("(if #t (- 1 (/ wealth-b pop)) (- 1 0c))"),
            Ok(Some(super::ExprKind::Intensive))
        );
        // The symmetric direction (neutral then-branch, non-neutral else).
        assert_eq!(
            kind_of("(if #t (- 1 0c) (- 1 (/ wealth-b pop)))"),
            Ok(Some(super::ExprKind::Intensive))
        );
    }

    #[test]
    fn extensive_times_intensive_is_legal_result_extensive_matching_real_content() {
        // Regression, found reading `lifecycle.bsl:307`'s real, committed
        // `new-wealth-d-prime = wealth-d-prime × surviving-fraction`: a
        // Currency stock (extensive) times a computed fraction (intensive)
        // — the ordinary "scale a stock by a rate/fraction" shape, not an
        // area-of-an-area. A first draft of this repair conservatively
        // refused every mixed extensive/intensive `*`/`/` combination and
        // broke this genuine content; extensive × intensive is licensed,
        // legal in EITHER operand order (kind carries no operand-order
        // information, unlike the value itself).
        assert_eq!(
            kind_of("(* wealth-b ws)"),
            Ok(Some(super::ExprKind::Extensive))
        );
        assert_eq!(
            kind_of("(* ws wealth-b)"),
            Ok(Some(super::ExprKind::Extensive))
        );
    }

    #[test]
    fn intensive_times_intensive_is_legal_result_intensive_matching_real_content() {
        // Controller adjudication, 2026-08-18 (delegated Director provenance):
        // regression, found reading `consciousness.bsl`'s real, committed
        // `p6-route`: `delta-r = (* (* consumed eff-sol) routing-scale)`,
        // where `consumed` (`agitation × consumption-rate`) and `eff-sol`
        // (a solidarity/chauvinist-derived ratio) are both intensive. The
        // product feeds `r1 = (+ r delta-r)`, and `r` (`social-class/
        // revolutionary`) is itself declared intensive — the consumer
        // already expects an intensive result. Standard dimensional rule:
        // a rate scaled by a dimensionless coefficient is still a rate.
        // Legal in either operand order (kind carries no operand-order
        // information).
        assert_eq!(kind_of("(* ws cons)"), Ok(Some(super::ExprKind::Intensive)));
        assert_eq!(kind_of("(* cons ws)"), Ok(Some(super::ExprKind::Intensive)));
    }

    #[test]
    fn intensive_divided_by_intensive_is_legal_result_intensive_matching_real_content() {
        // Controller adjudication, 2026-08-18 (delegated Director provenance,
        // same sitting, same narrow style as the product ruling above): the
        // fourth straddle site, found reading `consciousness.bsl`'s real,
        // committed `p6-route` simplex renormalization — `r2 = (/ r1
        // total)` and its `l2`/`f2` siblings, where `r1`/`l1`/`f1`/`total`
        // are all intensive (each of `r`/`l`/`f` plus the now-licensed
        // `delta-r`/`delta-l`/`delta-f`, summed). A ratio of two intensive
        // quantities is a dimensionless share — simplex normalization is
        // the canonical intensive operation. Legal in either operand
        // order (kind carries no operand-order information).
        assert_eq!(kind_of("(/ ws cons)"), Ok(Some(super::ExprKind::Intensive)));
        assert_eq!(kind_of("(/ cons ws)"), Ok(Some(super::ExprKind::Intensive)));
    }

    #[test]
    fn extensive_mixed_with_intensive_under_divide_stays_refused_narrow_licensing() {
        // Both the product (2026-08-18) and division (2026-08-18, same
        // sitting) rulings are narrow: same-kind pairs only (E÷E was
        // already licensed by D181; I×I and I÷I by D182/D183). A MIXED
        // extensive/intensive pair under `/`, in either operand position,
        // is a different, still-undecided question — leg (e)'s own
        // intensive-numerator-over-extensive-denominator refusal, and its
        // mirror image, must both still refuse after this ruling.
        for source in ["(/ ws wealth-b)", "(/ wealth-b ws)"] {
            let err = kind_of(source).unwrap_err();
            assert_eq!(err.code, Some(TypeCode::KindMixing), "{source}: {err:?}");
            assert!(
                err.message.contains("E-TYPE-040"),
                "{source}: {}",
                err.message
            );
        }
    }

    #[test]
    fn weighted_mean_of_intensive_divided_by_extensive_does_not_silently_become_extensive() {
        // (e): a weighted `mean` over an intensive body has result kind
        // intensive (D90) — dividing that by another extensive field must
        // not silently classify as extensive. This train's conservative
        // mixed-kind refusal (§3.4, the case the old bullet never named)
        // makes it a proven E-TYPE-040 instead — loud, not silent, and
        // never mislabeled extensive.
        let result =
            kind_of("(/ (fold mean (nodes NodeType/SOCIAL_CLASS) ws :weight pop) wealth-b)");
        assert_ne!(
            result,
            Ok(Some(super::ExprKind::Extensive)),
            "must not silently become extensive: {result:?}"
        );
        let err = result.unwrap_err();
        assert_eq!(err.code, Some(TypeCode::KindMixing));
    }

    #[test]
    fn a_weighted_intensive_mean_alone_classifies_as_intensive_d90() {
        // The fold-kind half of leg (e), isolated: the D90 result kind,
        // computed by this module's OWN independent `fold_kind` (not by
        // `typecheck_aggregation`, which this repair does not touch).
        assert_eq!(
            kind_of("(fold mean (nodes NodeType/SOCIAL_CLASS) ws :weight pop)"),
            Ok(Some(super::ExprKind::Intensive))
        );
    }

    #[test]
    fn kind_neutral_absorbs_on_every_operator() {
        // A literal is kind-neutral (§3.4): mixing it with any single kind
        // is always legal and carries that kind through.
        assert_eq!(
            kind_of("(+ wealth-b 1$)"),
            Ok(Some(super::ExprKind::Extensive))
        );
        assert_eq!(kind_of("(- ws 0.1c)"), Ok(Some(super::ExprKind::Intensive)));
        assert_eq!(kind_of("(* pop 2)"), Ok(Some(super::ExprKind::Extensive)));
        assert_eq!(kind_of("(/ cons 2)"), Ok(Some(super::ExprKind::Intensive)));
    }

    #[test]
    fn check_kind_mixing_walks_a_whole_tree_and_finds_a_nested_violation() {
        // The pipeline-facing entry point: a violation nested two levels
        // deep (not itself an <arith>/if top-level form) must still be
        // reached, mirroring `check_no_arithmetic_on_enum_field`'s own
        // nested-guard coverage above. `check_kind_mixing` is a raw tree
        // walker, agnostic of `guard`'s own effect grammar, so this proves
        // the recursion without needing a full valid rule.
        let (expr, _) = read("(guard #t (+ ws pop))").expect("must parse");
        let err = check_kind_mixing(&expr, &env(), &kind_bindings()).unwrap_err();
        assert_eq!(err.code, Some(TypeCode::KindMixing));
    }

    #[test]
    fn check_kind_mixing_accepts_a_clean_tree() {
        let (expr, _) = read("(guard #t (+ wealth-b 1$))").expect("must parse");
        assert!(check_kind_mixing(&expr, &env(), &kind_bindings()).is_ok());
    }
}
