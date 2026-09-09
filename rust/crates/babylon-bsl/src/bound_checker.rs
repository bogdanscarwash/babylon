//! Load-time static fuel bound checking (`bsl-language.rst` §3.7): "a rule
//! whose bound exceeds its budget is rejected **at content load**" — this is
//! the machinery that makes that claim true, plus §3.7's other *static*
//! ceiling check (an `add-hyperedge` member list longer than the declared
//! `:max-members`, `E-LOAD-042`). The hydration-time ceiling check
//! (`E-LOAD-041`) and the runtime meter (§4.5) are the evaluator's, not ours.
//!
//! Structural recursion here terminates by construction: the reader (§1)
//! cannot produce a cyclic structure, so an `SExpr` is a finite tree — the
//! same argument `canonical_ast` relies on. All arithmetic is saturating: a
//! bound that overflows `u64` is astronomically over any declarable budget,
//! and the `bound > :fuel` comparison still rejects it loudly.
//!
//! **Implementation note on `cost(query)` (recorded in §3.7, 2026-07-30):**
//! the draft row reads `1 + cost(element predicate, if any)`; this checker
//! additionally charges a query's *operand expression* (the first operand of
//! `neighbors` / `members-of` / `hyperedges-of`). §4.5 charges every
//! evaluated AST node, and omitting an evaluated operand would UNDER-count
//! the bound — the exact inversion of III.11 loud failure. Enum-refs and
//! direction keywords cost 0, so the two readings agree everywhere except
//! that operand.

use crate::fuel::{cost, CardinalityCeilings, IntrinsicCosts};
use crate::probability::{FINITE_KERNEL_DRAW_BASE, QUANTIZE_MASS_BASE};
use crate::reader::{Atom, SExpr};

/// The nine typed structural verbs plus `emit` (§2.8) — `update-edge` and
/// `update-hyperedge` are R9 chapters C2 and C12's additions (D35, D65).
const STRUCTURAL_VERBS: [&str; 10] = [
    "update-node",
    "update-edge",
    "update-hyperedge",
    "add-node",
    "remove-node",
    "add-edge",
    "remove-edge",
    "add-hyperedge",
    "remove-hyperedge",
    "emit",
];

/// The four §2.10 element accessors (R9 chapters C1/C2/C3/C9). Each is a
/// **keyed lookup**, charged `1 + Σ cost(operands)` and never multiplied by
/// a ceiling (D38).
const ACCESSORS: [&str; 4] = ["field-of", "edge-between", "the", "metric-of"];

/// The six query heads (§2.6).
const QUERY_HEADS: [&str; 6] = [
    "nodes",
    "edges",
    "neighbors",
    "hyperedges",
    "members-of",
    "hyperedges-of",
];

/// The four update operations (§2.8) — today's four-operation effect enum.
const UPDATE_OPS: [&str; 4] = ["add", "sub", "set", "scale"];

/// A load-time bound-check failure. Variants carry a `spec_code` only where
/// the language reference names one — no invented codes (the Task 10
/// precedent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundError {
    /// `E-LOAD-040` — `bound(rule) > :fuel`; the rule is rejected at content
    /// load, before any tick.
    BoundExceeded {
        /// The offending rule's id (its `<qname>`).
        rule_id: String,
        /// `bound(rule)` per the §3.7 table.
        computed_bound: u64,
        /// The rule's declared `:fuel`.
        declared_budget: u64,
    },
    /// `E-LOAD-042` — a `members-of` fold (or an `add-hyperedge`) over a
    /// hyperedge type whose manifest row declares no `:max-members`; without
    /// it there is no static bound at all (§3.7 member-count axis).
    MissingMaxMembers {
        /// The hyperedge type missing its `:max-members`.
        hyperedge_type: String,
    },
    /// `E-LOAD-042` — an `add-hyperedge` whose `<members>` list is longer
    /// than the declared `:max-members`. The list length is fixed in source
    /// text, so this check is static (§3.7).
    MemberListOverCeiling {
        /// The hyperedge type whose ceiling is violated.
        hyperedge_type: String,
        /// How many members the source lists.
        member_count: usize,
        /// The declared `:max-members`.
        max_members: u64,
    },
    /// `E-LOAD-021` — a call to an intrinsic with no declared `:cost`;
    /// never a default cost (§2.7).
    UndeclaredIntrinsic {
        /// The undeclared callee.
        name: String,
    },
    /// `E-LOAD-045` — a queried type with no declared `:ceiling` in the
    /// manifest (D76, R9 chapter C3's verification repair). Until that row
    /// the reference named no code for this case and this variant carried
    /// none; D76 supplies it, and the reasoning is the one already written
    /// here — a silent `0` would under-count the bound.
    MissingCeiling {
        /// The enum-ref of the queried type.
        queried_type: String,
    },
    /// `E-PARSE-020` — `(when)` with zero conditions (§2.3): the empty
    /// precondition set is not expressible; "always" is written by
    /// *omitting* the `<when>` clause (or `(when #t)`), never by an empty
    /// one. **#652 Task 6 repair, recorded:** before this variant existed
    /// this case fell through to [`Self::Malformed`], whose `spec_code()`
    /// is always `None` by construction — the loader's own `Display`
    /// embedded "E-PARSE-020" as plain prose (`docs/reference/
    /// bsl-language.rst:686` already documents the code; the census at
    /// plan §1.1 counts it "live" purely on that raise-site string, not on
    /// a structured `spec_code()` path). Surfacing what the loader already
    /// documents — not inventing a code (global constraint 4) — is exactly
    /// what a unit variant with a real `spec_code()` arm gives a caller
    /// (`bsl-ls`'s diagnostics mapping layer, §6.2) without it scanning
    /// `Display`'s message text, which sentinel 7.2 forbids outright. A
    /// unit variant, not a struct one: the offending form is `(when)`
    /// itself — there is no field to carry, and no locatable identity
    /// either (`error_identity.rs`'s `bound_identity` maps it to `None`,
    /// the same File-tier verdict `Malformed` already got).
    EmptyWhenCondition,
    /// A form whose shape contradicts the §2 grammar at a point the checker
    /// must destructure (proper `E-PARSE` classification is the parser's
    /// job; the checker refuses to guess a cost for a shape it cannot read).
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
}

impl BoundError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::BoundExceeded { .. } => Some("E-LOAD-040"),
            Self::MissingMaxMembers { .. } | Self::MemberListOverCeiling { .. } => {
                Some("E-LOAD-042")
            }
            Self::UndeclaredIntrinsic { .. } => Some("E-LOAD-021"),
            Self::MissingCeiling { .. } => Some("E-LOAD-045"),
            Self::EmptyWhenCondition => Some("E-PARSE-020"),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for BoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundExceeded {
                rule_id,
                computed_bound,
                declared_budget,
            } => write!(
                f,
                "E-LOAD-040: rule {rule_id} static bound {computed_bound} \
                 exceeds its declared :fuel {declared_budget}"
            ),
            Self::MissingMaxMembers { hyperedge_type } => write!(
                f,
                "E-LOAD-042: hyperedge type {hyperedge_type} declares no \
                 :max-members; a members-of fold has no static bound"
            ),
            Self::MemberListOverCeiling {
                hyperedge_type,
                member_count,
                max_members,
            } => write!(
                f,
                "E-LOAD-042: add-hyperedge lists {member_count} members but \
                 {hyperedge_type} declares :max-members {max_members}"
            ),
            Self::UndeclaredIntrinsic { name } => {
                write!(f, "E-LOAD-021: call to undeclared intrinsic {name}")
            }
            Self::MissingCeiling { queried_type } => write!(
                f,
                "E-LOAD-045: no :ceiling declared for queried type \
                 {queried_type}; the static bound is not computable, so the \
                 omission is not survivable by defaulting (§2.9, D76)"
            ),
            Self::EmptyWhenCondition => write!(
                f,
                "(when <cond>) takes exactly one condition — (when) is E-PARSE-020"
            ),
            Self::Malformed { message } => write!(f, "malformed form: {message}"),
        }
    }
}

impl std::error::Error for BoundError {}

fn malformed(message: impl Into<String>) -> BoundError {
    BoundError::Malformed {
        message: message.into(),
    }
}

/// The head symbol of a form, if its first item is a `Symbol` atom.
fn head_symbol(items: &[SExpr]) -> Option<&str> {
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(s))) => Some(s.as_str()),
        _ => None,
    }
}

/// Render an enum-ref atom as its manifest key, `EnumType/MEMBER`.
fn enum_ref_key(expr: &SExpr) -> Result<String, BoundError> {
    match expr {
        SExpr::Atom(Atom::EnumRef { enum_type, member }) => Ok(format!("{enum_type}/{member}")),
        other => Err(malformed(format!(
            "expected an enum-ref where the grammar requires one, found {other:?}"
        ))),
    }
}

/// Σ `cost(item)` with saturating addition.
fn sum_costs(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let mut total: u64 = 0;
    for item in items {
        total = total.saturating_add(expr_cost(item, ceilings, intrinsics)?);
    }
    Ok(total)
}

/// `cost(n)` over the AST — the §3.7 table, one arm per row.
///
/// # Errors
///
/// [`BoundError::MissingCeiling`] / [`BoundError::MissingMaxMembers`] when a
/// query's type has no declared ceiling on the axis the query bounds
/// against; [`BoundError::UndeclaredIntrinsic`] on a call to an intrinsic
/// with no declared cost; [`BoundError::Malformed`] on a shape the §2
/// grammar does not admit.
pub fn expr_cost(
    expr: &SExpr,
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = match expr {
        SExpr::Atom(atom) => return atom_cost(atom),
        SExpr::List(items) => items,
    };
    if let Some(SExpr::Atom(Atom::Operator(_))) = items.first() {
        // arith | cmp — one shared row with the boolean operators.
        return Ok(cost::ARITH_CMP_BOOL_BASE.saturating_add(sum_costs(
            &items[1..],
            ceilings,
            intrinsics,
        )?));
    }
    if let Some(SExpr::Atom(Atom::QName(_))) = items.first() {
        // A field-init `(<qname> <expr>)` (§2.8) — grouping: the qname is a
        // static field path (cost 0), only the value expression is charged.
        return sum_costs(&items[1..], ceilings, intrinsics);
    }
    let Some(head) = head_symbol(items) else {
        return Err(malformed(format!(
            "a form must be headed by a symbol, operator or qname, found {:?}",
            items.first()
        )));
    };
    match head {
        "and" | "or" | "not" => Ok(cost::ARITH_CMP_BOOL_BASE.saturating_add(sum_costs(
            &items[1..],
            ceilings,
            intrinsics,
        )?)),
        "if" => if_cost(items, ceilings, intrinsics),
        "quantize-mass" => {
            let [_, operand] = items.as_slice() else {
                return Err(malformed(
                    "(quantize-mass <numeric-expr>) takes exactly one operand",
                ));
            };
            Ok(QUANTIZE_MASS_BASE.saturating_add(expr_cost(operand, ceilings, intrinsics)?))
        }
        "exists" | "forall" => exists_forall_cost(items, ceilings, intrinsics),
        "fold" => fold_cost(items, ceilings, intrinsics),
        "select-max" | "select-min" => selection_cost(items, ceilings, intrinsics),
        "for-each" => for_each_cost(items, ceilings, intrinsics),
        "guard" => {
            // cost(guard) = 1 + cost(cond) + Σ cost(effect-items); the cond
            // and the effect items are all `items[1..]`.
            Ok(cost::GUARD_BASE.saturating_add(sum_costs(&items[1..], ceilings, intrinsics)?))
        }
        "members" => sum_costs(&items[1..], ceilings, intrinsics), // grouping, no base
        // §3.7: `cost(metric-of) = 1 + cost(element expr)` — the metric
        // NAME is a static registry key, not a variable reference, so it
        // charges 0 like a fold-op or an enum-ref.
        "metric-of" => {
            Ok(cost::ACCESSOR_BASE.saturating_add(sum_costs(&items[1..2], ceilings, intrinsics)?))
        }
        h if ACCESSORS.contains(&h) => {
            Ok(cost::ACCESSOR_BASE.saturating_add(sum_costs(&items[1..], ceilings, intrinsics)?))
        }
        h if QUERY_HEADS.contains(&h) => query_cost(items, ceilings, intrinsics),
        h if UPDATE_OPS.contains(&h) => {
            Ok(cost::UPDATE_OP_BASE.saturating_add(sum_costs(&items[1..], ceilings, intrinsics)?))
        }
        h if STRUCTURAL_VERBS.contains(&h) => verb_cost(items, ceilings, intrinsics),
        name => {
            // §2.7: any other symbol head in expression position is an
            // intrinsic call — declared cost or E-LOAD-021, never a default.
            let Some(declared) = intrinsics.declared_cost(name) else {
                return Err(BoundError::UndeclaredIntrinsic {
                    name: name.to_owned(),
                });
            };
            Ok(cost::INTRINSIC_CALL_BASE
                .saturating_add(declared)
                .saturating_add(sum_costs(&items[1..], ceilings, intrinsics)?))
        }
    }
}

/// `cost(literal) = 0`, `cost(variable-ref) = 1`,
/// `cost(field path | enum-ref) = 0`.
fn atom_cost(atom: &Atom) -> Result<u64, BoundError> {
    match atom {
        Atom::Int(_)
        | Atom::Currency(_)
        | Atom::Mass(_)
        | Atom::Scaled(_)
        | Atom::Bool(_)
        | Atom::Str(_)
        | Atom::QName(_)
        | Atom::EnumRef { .. }
        | Atom::Keyword(_)
        | Atom::BareUpperIdent(_) => Ok(cost::LITERAL),
        Atom::Symbol(_) => Ok(cost::VARIABLE_REF),
        Atom::Operator(op) => Err(malformed(format!(
            "operator {op} is valid only in form-head position"
        ))),
    }
}

/// `cost(if) = 1 + cost(cond) + max(cost(then), cost(else))` — the worse
/// branch, because the bound is a static worst case (§3.7).
fn if_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let [_, cond, then_branch, else_branch] = items else {
        return Err(malformed(
            "(if <cond> <expr> <expr>) takes exactly three operands",
        ));
    };
    let cond_cost = expr_cost(cond, ceilings, intrinsics)?;
    let then_cost = expr_cost(then_branch, ceilings, intrinsics)?;
    let else_cost = expr_cost(else_branch, ceilings, intrinsics)?;
    Ok(cost::IF_BASE
        .saturating_add(cond_cost)
        .saturating_add(then_cost.max(else_cost)))
}

/// `cost(exists | forall) = 2 + cost(query) + ceiling(query) × cost(body)`.
/// `(exists <query>)` with no body is "the query is non-empty" (§2.4) —
/// body cost 0.
fn exists_forall_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = strip_elem_name(items)?;
    let (query, body) = match items.as_slice() {
        [_, query] => (query, None),
        [_, query, body] => (query, Some(body)),
        _ => {
            return Err(malformed(
                "(exists <query> <elem-name>? <cond>?) / (forall <query> <elem-name>? <cond>) \
             take a query and at most one body",
            ))
        }
    };
    let query_items = query_form(query)?;
    let ceiling = ceiling_of_query(query_items, ceilings)?;
    let body_cost = body.map_or(Ok(0), |b| expr_cost(b, ceilings, intrinsics))?;
    Ok(cost::EXISTS_FORALL_BASE
        .saturating_add(query_cost(query_items, ceilings, intrinsics)?)
        .saturating_add(ceiling.saturating_mul(body_cost)))
}

/// `cost(fold) = 2 + cost(query) + ceiling(query) × (cost(body) + cost(weight))`.
/// Shape (§2.7): `(fold <fold-op> <query> <expr> (:weight <expr>)?)` — the
/// fold-op symbol is structural, not a variable reference.
fn fold_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = strip_elem_name(items)?;
    let (op, query, body, weight) = match items.as_slice() {
        [_, op, query, body] => (op, query, body, None),
        [_, op, query, body, SExpr::Atom(Atom::Keyword(kw)), weight] if kw == "weight" => {
            (op, query, body, Some(weight))
        }
        _ => {
            return Err(malformed(
                "(fold <fold-op> <query> <elem-name>? <expr> (:weight <expr>)?) \
                 — unrecognized fold shape",
            ))
        }
    };
    if !matches!(op, SExpr::Atom(Atom::Symbol(_))) {
        return Err(malformed(format!("fold-op must be a symbol, found {op:?}")));
    }
    let query_items = query_form(query)?;
    let ceiling = ceiling_of_query(query_items, ceilings)?;
    let body_cost = expr_cost(body, ceilings, intrinsics)?;
    let weight_cost = weight.map_or(Ok(0), |w| expr_cost(w, ceilings, intrinsics))?;
    Ok(cost::FOLD_BASE
        .saturating_add(query_cost(query_items, ceilings, intrinsics)?)
        .saturating_add(ceiling.saturating_mul(body_cost.saturating_add(weight_cost))))
}

/// `cost(select-max | select-min) = 2 + cost(query) + ceiling(query) ×
/// cost(score)` (§3.7, R9 chapter C5). A selection returns the *element*
/// that extremises a score where a fold returns the extremised *value*, so
/// it costs the same shape as a fold with one body and no weight.
fn selection_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = strip_elem_name(items)?;
    let [_, query, score] = items.as_slice() else {
        return Err(malformed(
            "(select-max|select-min <query> <elem-name>? <expr>) — unrecognized shape",
        ));
    };
    let query_items = query_form(query)?;
    let ceiling = ceiling_of_query(query_items, ceilings)?;
    let score_cost = expr_cost(score, ceilings, intrinsics)?;
    Ok(cost::SELECTION_BASE
        .saturating_add(query_cost(query_items, ceilings, intrinsics)?)
        .saturating_add(ceiling.saturating_mul(score_cost)))
}

/// `cost(for-each) = 2 + cost(query) + ceiling(query) × Σ cost(effect-items)`
/// (§3.7, R9 chapter C6) — charged exactly as `exists`/`forall` are, which
/// is what keeps the totality argument syntactic: the set is materialized
/// before the body runs and its size is bounded by the declared ceiling, so
/// this is a bounded iteration and not a loop.
fn for_each_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = strip_elem_name(items)?;
    let [_, query, effect_items @ ..] = items.as_slice() else {
        return Err(malformed(
            "(for-each <query> <elem-name>? <effect-item>+) — unrecognized shape",
        ));
    };
    if effect_items.is_empty() {
        return Err(malformed(
            "(for-each …) requires at least one effect item (§2.8)",
        ));
    }
    let query_items = query_form(query)?;
    let ceiling = ceiling_of_query(query_items, ceilings)?;
    let body_cost = sum_costs(effect_items, ceilings, intrinsics)?;
    Ok(cost::FOR_EACH_BASE
        .saturating_add(query_cost(query_items, ceilings, intrinsics)?)
        .saturating_add(ceiling.saturating_mul(body_cost)))
}

/// Strip an optional `:as <symbol>` element name from an operand list
/// (§2.6's `<elem-name>?`, R9 chapter C8). `cost(:as name) = 0` — the name
/// is a binding, not a charged node; a *reference* to it costs 1 like any
/// other variable reference. Returns the list with the two tokens removed.
fn strip_elem_name(items: &[SExpr]) -> Result<Vec<SExpr>, BoundError> {
    let mut out = Vec::with_capacity(items.len());
    let mut i = 0;
    while i < items.len() {
        if let SExpr::Atom(Atom::Keyword(kw)) = &items[i] {
            if kw == "as" {
                match items.get(i + 1) {
                    Some(SExpr::Atom(Atom::Symbol(_))) => {
                        i += 2;
                        continue;
                    }
                    other => {
                        return Err(malformed(format!(
                            ":as names an element with a symbol, found {other:?}"
                        )))
                    }
                }
            }
        }
        out.push(items[i].clone());
        i += 1;
    }
    Ok(out)
}

/// Destructure a query position into its form items, rejecting non-query
/// shapes loudly.
fn query_form(query: &SExpr) -> Result<&[SExpr], BoundError> {
    let SExpr::List(items) = query else {
        return Err(malformed(format!("expected a query form, found {query:?}")));
    };
    match head_symbol(items) {
        Some(h) if QUERY_HEADS.contains(&h) => Ok(items),
        other => Err(malformed(format!(
            "expected one of the six §2.6 query heads, found {other:?}"
        ))),
    }
}

/// `cost(query) = 1 +` the charged children — the predicate cond and (see
/// the module note) any operand expression; enum-refs and direction
/// keywords cost 0.
fn query_cost(
    query_items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    Ok(cost::QUERY_BASE.saturating_add(sum_costs(&query_items[1..], ceilings, intrinsics)?))
}

/// `ceiling(neighbors)` is the **lesser** of the queried edge type's
/// ceiling and the annotated result node type's (§3.7, revised by D52):
/// neither bound can be exceeded, so the smaller is the honest one, and the
/// fourth operand C8 makes mandatory is what makes the second number
/// available. (A per-node degree ceiling would be tighter still and remains
/// the review item D15 recorded.)
fn neighbors_ceiling(
    query_items: &[SExpr],
    ceilings: &CardinalityCeilings,
) -> Result<u64, BoundError> {
    // (neighbors <expr> <EdgeType> <direction> <NodeType>) — D51 makes the
    // fourth operand mandatory, so a three-operand form has no second
    // number and is E-PARSE-042 at the grammar pass, not a silent bound.
    let edge_ref = query_items
        .get(2)
        .ok_or_else(|| malformed("(neighbors …) is missing its EdgeType operand"))?;
    let node_ref = query_items.get(4).ok_or_else(|| {
        malformed(
            "(neighbors <expr> <EdgeType> <direction> <NodeType>) — the result \
             NodeType operand is MANDATORY (D51); the pre-C8 three-operand form \
             is E-PARSE-042",
        )
    })?;
    let edge_key = enum_ref_key(edge_ref)?;
    let node_key = enum_ref_key(node_ref)?;
    let edge_ceiling = ceilings
        .ceiling(&edge_key)
        .ok_or(BoundError::MissingCeiling {
            queried_type: edge_key,
        })?;
    let node_ceiling = ceilings
        .ceiling(&node_key)
        .ok_or(BoundError::MissingCeiling {
            queried_type: node_key,
        })?;
    Ok(edge_ceiling.min(node_ceiling))
}

/// `ceiling(query)` — the §3.7 axis dispatch. `nodes`/`edges`/`hyperedges`
/// bound against the queried type's `:ceiling`; `neighbors` against the
/// queried *edge type's* `:ceiling` (draft ruling); `hyperedges-of` against
/// the hyperedge type's `:ceiling`; `members-of` against its
/// `:max-members`. Using the wrong axis silently mis-bounds every
/// membership fold — hence the loud per-head dispatch.
fn ceiling_of_query(
    query_items: &[SExpr],
    ceilings: &CardinalityCeilings,
) -> Result<u64, BoundError> {
    let head = head_symbol(query_items)
        .ok_or_else(|| malformed("a query form must be headed by a symbol"))?;
    if head == "neighbors" {
        return neighbors_ceiling(query_items, ceilings);
    }
    let enum_ref_index = match head {
        "nodes" | "edges" | "hyperedges" => 1,
        "members-of" | "hyperedges-of" => 2,
        other => return Err(malformed(format!("not a query head: {other}"))),
    };
    let type_ref = query_items
        .get(enum_ref_index)
        .ok_or_else(|| malformed(format!("({head} …) is missing its type enum-ref operand")))?;
    let key = enum_ref_key(type_ref)?;
    if head == "members-of" {
        ceilings
            .max_members(&key)
            .ok_or(BoundError::MissingMaxMembers {
                hyperedge_type: key,
            })
    } else {
        ceilings
            .ceiling(&key)
            .ok_or(BoundError::MissingCeiling { queried_type: key })
    }
}

/// `cost(structural verb) = 3 + Σ cost(operands)`. Operands include
/// `<members>` lists (grouping, Σ only), `<field-init>` forms (qname head,
/// value charged) and — for `emit` — `<payload-item>` forms, whose name
/// symbol is a static label, not a variable reference.
fn verb_cost(
    items: &[SExpr],
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let is_emit = head_symbol(items) == Some("emit");
    let mut total = cost::STRUCTURAL_VERB_BASE;
    for operand in &items[1..] {
        let operand_cost = match operand {
            // (<symbol> <expr>) under emit is a payload item: the name is a
            // static label (cost 0), only the value expression is charged.
            SExpr::List(inner)
                if is_emit && matches!(inner.first(), Some(SExpr::Atom(Atom::Symbol(_)))) =>
            {
                sum_costs(&inner[1..], ceilings, intrinsics)?
            }
            other => expr_cost(other, ceilings, intrinsics)?,
        };
        total = total.saturating_add(operand_cost);
    }
    Ok(total)
}

/// `bound(rule) = Σ cost(:expr bindings) + cost(cond of <when>) + Σ cost(effect-items)`
/// (§3.7, with D50's row). A
/// rule with no `<when>` is unconditional (§2.3) and contributes 0 for the
/// condition. EXTERNAL bind-srcs never enter the bound — the §5.6 worked
/// example (bound = 7 with a `:field` bindings form present) pins that;
/// only `:expr` bindings do, per D50.
///
/// # Errors
///
/// Everything [`expr_cost`] raises, plus [`BoundError::Malformed`] when the
/// form is not a `(rule <qname> …)` with a well-shaped `<when>`/`<effects>`.
pub fn rule_bound(
    rule: &SExpr,
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = rule_items(rule)?;
    let mut bound: u64 = 0;
    let mut effects_seen = false;
    for child in &items[1..] {
        let SExpr::List(inner) = child else { continue };
        match head_symbol(inner) {
            // §3.7 (D50): `bound(rule)` gains `Σ cost(:expr bindings)`.
            // Every other bind-src names an external source and costs
            // nothing here; a `:expr` is an expression and costs one.
            Some("bindings") => {
                for row in &inner[1..] {
                    let SExpr::List(cells) = row else { continue };
                    for window in cells.windows(2) {
                        if let [SExpr::Atom(Atom::Keyword(kw)), operand] = window {
                            if kw == "expr" {
                                bound =
                                    bound.saturating_add(expr_cost(operand, ceilings, intrinsics)?);
                            }
                        }
                    }
                }
            }
            Some("when") => {
                let [_, cond] = inner.as_slice() else {
                    // §652 Task 6: a real coded variant, not `malformed()`'s
                    // uncoded `Malformed` — see `BoundError::
                    // EmptyWhenCondition`'s own doc for why.
                    return Err(BoundError::EmptyWhenCondition);
                };
                bound = bound.saturating_add(expr_cost(cond, ceilings, intrinsics)?);
            }
            Some("effects") => {
                if inner.len() < 2 {
                    return Err(malformed(
                        "(effects <effect-item>+) requires at least one item",
                    ));
                }
                effects_seen = true;
                bound = bound.saturating_add(sum_costs(&inner[1..], ceilings, intrinsics)?);
            }
            _ => {}
        }
    }
    if !effects_seen {
        return Err(malformed("a rule requires an (effects …) form (§2.3)"));
    }
    Ok(bound)
}

/// Destructure a `(rule <qname> …)` form.
fn rule_items(rule: &SExpr) -> Result<&[SExpr], BoundError> {
    let SExpr::List(items) = rule else {
        return Err(malformed(format!(
            "expected a (rule …) form, found {rule:?}"
        )));
    };
    if head_symbol(items) != Some("rule") {
        return Err(malformed(format!(
            "expected a (rule …) form, found head {:?}",
            items.first()
        )));
    }
    Ok(items)
}

/// The rule's id qname, for error reporting.
fn rule_id(items: &[SExpr]) -> String {
    match items.get(1) {
        Some(SExpr::Atom(Atom::QName(q))) => q.clone(),
        _ => "<unidentified rule>".to_owned(),
    }
}

/// The rule's declared `:fuel` budget — mandatory on every rule (§2.2).
///
/// `pub(crate)` so the tick loop meters on the AUTHOR's declared budget
/// rather than on `check_rule`'s computed bound: the computed bound is the
/// load-time proof that the rule fits, not the allowance it runs under.
pub(crate) fn declared_fuel(items: &[SExpr]) -> Result<u64, BoundError> {
    for window in items.windows(2) {
        if let [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Int(n))] = window {
            if kw == "fuel" {
                return u64::try_from(*n).map_err(|_| {
                    malformed(format!("a negative :fuel budget ({n}) is meaningless"))
                });
            }
        }
    }
    Err(malformed(":fuel is mandatory on every rule (§2.2)"))
}

/// §3.7's static `add-hyperedge` check: a `<members>` list longer than the
/// declared `:max-members` is `E-LOAD-042` — the length is fixed in source
/// text. Recurses through `guard` effect items.
fn check_member_lists(
    effect_item: &SExpr,
    ceilings: &CardinalityCeilings,
) -> Result<(), BoundError> {
    let SExpr::List(items) = effect_item else {
        return Ok(());
    };
    match head_symbol(items) {
        Some("guard") => {
            for nested in &items[1..] {
                check_member_lists(nested, ceilings)?;
            }
            Ok(())
        }
        Some("add-hyperedge") => {
            let type_ref = items
                .get(1)
                .ok_or_else(|| malformed("(add-hyperedge …) is missing its type enum-ref"))?;
            let key = enum_ref_key(type_ref)?;
            let max = ceilings
                .max_members(&key)
                .ok_or(BoundError::MissingMaxMembers {
                    hyperedge_type: key.clone(),
                })?;
            let member_count = items
                .iter()
                .find_map(|child| match child {
                    SExpr::List(inner) if head_symbol(inner) == Some("members") => {
                        Some(inner.len() - 1)
                    }
                    _ => None,
                })
                .ok_or_else(|| malformed("(add-hyperedge …) requires a (members <expr>+) list"))?;
            if member_count as u64 > max {
                return Err(BoundError::MemberListOverCeiling {
                    hyperedge_type: key,
                    member_count,
                    max_members: max,
                });
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// The full load-time check for one rule: compute `bound(rule)`, run the
/// static member-list check, and reject `bound > :fuel` as `E-LOAD-040`.
/// Returns the computed bound on acceptance (conformance vectors pin it).
///
/// # Errors
///
/// [`BoundError::BoundExceeded`] (`E-LOAD-040`) when the bound exceeds the
/// declared budget; [`BoundError::MemberListOverCeiling`] /
/// [`BoundError::MissingMaxMembers`] (`E-LOAD-042`) from the static
/// hyperedge checks; plus everything [`rule_bound`] raises.
pub fn check_rule(
    rule: &SExpr,
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let items = rule_items(rule)?;
    let budget = declared_fuel(items)?;
    for child in &items[1..] {
        if let SExpr::List(inner) = child {
            if head_symbol(inner) == Some("effects") {
                for effect_item in &inner[1..] {
                    check_member_lists(effect_item, ceilings)?;
                }
            }
        }
    }
    let computed = rule_bound(rule, ceilings, intrinsics)?;
    if computed > budget {
        return Err(BoundError::BoundExceeded {
            rule_id: rule_id(items),
            computed_bound: computed,
            declared_budget: budget,
        });
    }
    Ok(computed)
}

fn replace_at_path(expr: &mut SExpr, path: &[u32]) -> Result<(), BoundError> {
    // Finite-kernel paths retain the rule's original top-form index. This
    // checker owns only the isolated rule form, so it skips that forest root
    // coordinate and follows the remaining child coordinates unchanged.
    let mut current = expr;
    for component in path.iter().skip(1) {
        let SExpr::List(items) = current else {
            return Err(malformed("finite-kernel FormPath crossed an atom"));
        };
        let index = usize::try_from(*component)
            .map_err(|_| malformed("finite-kernel FormPath index does not fit usize"))?;
        current = items
            .get_mut(index)
            .ok_or_else(|| malformed("finite-kernel FormPath is outside the rule"))?;
    }
    *current = SExpr::Atom(Atom::Int(0));
    Ok(())
}

/// Check one rule using its already compiled finite-kernel IR.
///
/// The deterministic remainder goes through [`check_rule`] unchanged. The
/// typed kernel contributes one fixed private draw, every branch-mass cost,
/// and only the maximum branch-body cost. This is the sole kernel fuel path;
/// the bound checker never reparses `(choose ...)` from raw S-expressions.
///
/// # Errors
///
/// Returns [`BoundError`] when either the deterministic rule or any typed
/// kernel expression exceeds or violates the configured static bounds.
pub fn check_rule_with_kernel(
    rule: &SExpr,
    kernel: Option<&crate::probability::FiniteKernelV1>,
    ceilings: &CardinalityCeilings,
    intrinsics: &IntrinsicCosts,
) -> Result<u64, BoundError> {
    let Some(kernel) = kernel else {
        return check_rule(rule, ceilings, intrinsics);
    };
    for branch in &kernel.branches {
        for effect in &branch.effects {
            check_member_lists(effect, ceilings)?;
        }
    }
    let mut deterministic = rule.clone();
    replace_at_path(&mut deterministic, &kernel.form_path)?;
    let base = check_rule(&deterministic, ceilings, intrinsics)?;
    let mass_cost = kernel.branches.iter().try_fold(0_u64, |total, branch| {
        Ok::<u64, BoundError>(total.saturating_add(expr_cost(&branch.mass, ceilings, intrinsics)?))
    })?;
    let branch_cost = kernel.branches.iter().try_fold(0_u64, |worst, branch| {
        Ok::<u64, BoundError>(worst.max(sum_costs(&branch.effects, ceilings, intrinsics)?))
    })?;
    let computed = base
        .saturating_add(FINITE_KERNEL_DRAW_BASE)
        .saturating_add(mass_cost)
        .saturating_add(branch_cost);
    let items = rule_items(rule)?;
    let budget = declared_fuel(items)?;
    if computed > budget {
        return Err(BoundError::BoundExceeded {
            rule_id: rule_id(items),
            computed_bound: computed,
            declared_budget: budget,
        });
    }
    Ok(computed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;
    use std::collections::HashMap;

    /// The §5.6 worked example, verbatim — the same source `canonical_ast`
    /// pins to 500 bytes; here it pins `bound = 7`.
    const DEMO_RULE: &str = r#"
        ; a rule is data; this comment is not part of the hash
        (rule demo/hunger
          :role mechanic :evidence derived :material-basis "subsistence deficit at the point of reproduction"
          :fuel 64
          (bindings
            (binding wealth :field social-class/wealth))
          (when (< wealth 1000.5$))
          (effects
            (update-node self social-class/agitation (add 0.05i))))
    "#;

    fn e(source: &str) -> crate::reader::SExpr {
        read(source).expect("test source must parse").0
    }

    fn ceilings() -> CardinalityCeilings {
        CardinalityCeilings::new(
            HashMap::from([
                ("NodeType/SOCIAL_CLASS".to_owned(), 100),
                ("EdgeType/SOLIDARITY".to_owned(), 40),
                ("NodeType/COMMITTEE".to_owned(), 5),
                ("HyperedgeType/ECONOMIC_SECTOR".to_owned(), 500),
                ("HyperedgeType/CELL".to_owned(), 200),
                ("HyperedgeType/PAIR".to_owned(), 10),
            ]),
            HashMap::from([
                ("HyperedgeType/ECONOMIC_SECTOR".to_owned(), 32),
                ("HyperedgeType/PAIR".to_owned(), 2),
            ]),
        )
    }

    fn intrinsics() -> IntrinsicCosts {
        IntrinsicCosts::new(HashMap::from([("sigmoid".to_owned(), 40)]))
    }

    fn cost_of(source: &str) -> Result<u64, BoundError> {
        expr_cost(&e(source), &ceilings(), &intrinsics())
    }

    #[test]
    fn the_spec_worked_example_bounds_to_seven() {
        // §5.6: cost(when) = 1 + 1 + 0 = 2; cost(update-node) = 3 + 1 + 0 +
        // (1 + 0) = 5; bound = 7 ≤ :fuel 64 — the rule loads. The bindings
        // form is present and contributes nothing.
        let rule = e(DEMO_RULE);
        assert_eq!(rule_bound(&rule, &ceilings(), &intrinsics()), Ok(7));
        assert_eq!(check_rule(&rule, &ceilings(), &intrinsics()), Ok(7));
    }

    #[test]
    fn literals_and_static_refs_cost_zero_and_variable_refs_cost_one() {
        assert_eq!(cost_of("5"), Ok(0));
        assert_eq!(cost_of("0.5c"), Ok(0));
        assert_eq!(cost_of("1000.5$"), Ok(0));
        assert_eq!(cost_of("#t"), Ok(0));
        assert_eq!(cost_of("NodeType/SOCIAL_CLASS"), Ok(0));
        assert_eq!(cost_of("social-class/wealth"), Ok(0));
        assert_eq!(cost_of("wealth"), Ok(1));
    }

    #[test]
    fn arithmetic_comparison_and_boolean_charge_one_plus_children() {
        assert_eq!(cost_of("(+ 1 2)"), Ok(1));
        assert_eq!(cost_of("(< wealth 1000.5$)"), Ok(2));
        assert_eq!(cost_of("(and (< wealth 1000.5$) #t)"), Ok(3));
        assert_eq!(cost_of("(not #f)"), Ok(1));
    }

    #[test]
    fn if_charges_the_worse_branch_not_both() {
        // 1 + cost(cond)=2 + max(cost(then)=2, cost(else)=0) = 5.
        assert_eq!(cost_of("(if (< wealth 1000.5$) (+ wealth 1) 5)"), Ok(5));
    }

    #[test]
    fn a_fold_multiplies_body_by_the_declared_ceiling() {
        // 2 + query(1) + 100 × body(1) = 103.
        assert_eq!(
            cost_of("(fold sum (nodes NodeType/SOCIAL_CLASS) it)"),
            Ok(103)
        );
    }

    #[test]
    fn a_fold_weight_joins_the_body_inside_the_multiplication() {
        // 2 + query(1) + 100 × (body(1) + weight(2)) = 303.
        assert_eq!(
            cost_of("(fold mean (nodes NodeType/SOCIAL_CLASS) it :weight (+ it 1))"),
            Ok(303)
        );
    }

    #[test]
    fn members_of_bounds_against_max_members_not_the_type_ceiling() {
        // 2 + query(1 + operand h = 2) + 32 × body(1) = 36. Against the
        // WRONG axis (the type :ceiling, 500) this would be 504.
        assert_eq!(
            cost_of("(fold sum (members-of h HyperedgeType/ECONOMIC_SECTOR) it)"),
            Ok(36)
        );
    }

    #[test]
    fn hyperedges_and_hyperedges_of_bound_against_the_type_ceiling() {
        // hyperedges: 2 + query(1) + 500 × 1 = 503.
        assert_eq!(
            cost_of("(fold count (hyperedges HyperedgeType/ECONOMIC_SECTOR) it)"),
            Ok(503)
        );
        // hyperedges-of: 2 + query(1 + self = 2) + 500 × 1 = 504.
        assert_eq!(
            cost_of("(fold count (hyperedges-of self HyperedgeType/ECONOMIC_SECTOR) it)"),
            Ok(504)
        );
    }

    /// **D51/D52, R9 chapter C8 — this test previously pinned the
    /// three-operand `neighbors` and its edge-type-only bound.** The
    /// reference records the change and its price explicitly: the form gains
    /// a mandatory result-`NodeType` operand, no conformance vector or
    /// content rule exercised the old form, and this crate's checker is the
    /// one place that carried it. The bound is now the LESSER of the two
    /// ceilings.
    #[test]
    fn neighbors_bounds_against_the_lesser_of_its_two_ceilings() {
        // EdgeType/SOLIDARITY 40 vs NodeType/SOCIAL_CLASS 100 → 40.
        // 2 + query(1 + self = 2) + 40 × 1 = 44.
        assert_eq!(
            cost_of(
                "(fold sum (neighbors self EdgeType/SOLIDARITY :out \
                 NodeType/SOCIAL_CLASS) it)"
            ),
            Ok(44)
        );
        // The other way round: NodeType/COMMITTEE 5 is the lesser, so the
        // annotation TIGHTENS the bound the old reading would have given.
        // 2 + 2 + 5 × 1 = 9.
        assert_eq!(
            cost_of(
                "(fold sum (neighbors self EdgeType/SOLIDARITY :out \
                 NodeType/COMMITTEE) it)"
            ),
            Ok(9)
        );
    }

    #[test]
    fn the_pre_c8_three_operand_neighbors_no_longer_bounds() {
        // Its second ceiling does not exist, so there is no honest bound to
        // compute; the grammar pass rejects it as E-PARSE-042 (D51/D75) and
        // the checker refuses to guess rather than silently using the old
        // edge-type-only reading.
        assert!(matches!(
            cost_of("(fold sum (neighbors self EdgeType/SOLIDARITY :out) it)"),
            Err(BoundError::Malformed { .. })
        ));
    }

    #[test]
    fn an_accessor_charges_one_plus_operands_and_never_a_ceiling() {
        // §3.7 (D38): `cost(field-of) = 1 + cost(element expr)`; the qname
        // is a static field path (0). A fold over `edges` reading the edge
        // under it therefore pays the accessor per element, not a second
        // ceiling factor: 2 + query(1) + 40 × (1 + 1) = 83.
        assert_eq!(cost_of("(field-of it solidarity/strength)"), Ok(2));
        assert_eq!(
            cost_of(
                "(fold sum (edges EdgeType/SOLIDARITY) \
                 (field-of it solidarity/strength))"
            ),
            Ok(83)
        );
    }

    #[test]
    fn a_missing_ceiling_is_a_loud_error_never_zero() {
        let err = cost_of("(fold sum (nodes NodeType/TERRITORY) it)").unwrap_err();
        assert_eq!(
            err,
            BoundError::MissingCeiling {
                queried_type: "NodeType/TERRITORY".to_owned()
            }
        );
        assert_eq!(
            err.spec_code(),
            Some("E-LOAD-045"),
            "D76 supplies the code this case previously lacked"
        );
    }

    #[test]
    fn members_of_without_max_members_is_e_load_042() {
        // HyperedgeType/CELL declares a :ceiling but no :max-members.
        let err = cost_of("(fold sum (members-of h HyperedgeType/CELL) it)").unwrap_err();
        assert_eq!(
            err,
            BoundError::MissingMaxMembers {
                hyperedge_type: "HyperedgeType/CELL".to_owned()
            }
        );
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    #[test]
    fn an_intrinsic_call_charges_five_plus_declared_cost_plus_args() {
        // 5 + declared(40) + arg(1) = 46.
        assert_eq!(cost_of("(sigmoid wealth)"), Ok(46));
    }

    #[test]
    fn an_undeclared_intrinsic_call_is_e_load_021() {
        let err = cost_of("(entropy wealth)").unwrap_err();
        assert_eq!(
            err,
            BoundError::UndeclaredIntrinsic {
                name: "entropy".to_owned()
            }
        );
        assert_eq!(err.spec_code(), Some("E-LOAD-021"));
    }

    #[test]
    fn exists_without_a_body_charges_the_query_alone() {
        // 2 + query(1) + 100 × 0 = 3.
        assert_eq!(cost_of("(exists (nodes NodeType/SOCIAL_CLASS))"), Ok(3));
        // forall: 2 + query(1) + 100 × body(2) = 203.
        assert_eq!(
            cost_of("(forall (nodes NodeType/SOCIAL_CLASS) (< it 5))"),
            Ok(203)
        );
    }

    #[test]
    fn a_guard_charges_cond_plus_its_effect_items() {
        // 1 + cond(2) + update-node(5) = 8.
        assert_eq!(
            cost_of(
                "(guard (< wealth 1000.5$) \
                 (update-node self social-class/agitation (add 0.05i)))"
            ),
            Ok(8)
        );
    }

    #[test]
    fn verb_operand_shapes_cost_as_grouping_not_as_calls() {
        // add-hyperedge: 3 + enum-ref(0) + id(1) + members(1+1) = 6.
        assert_eq!(
            cost_of("(add-hyperedge HyperedgeType/PAIR h1 (members a b))"),
            Ok(6)
        );
        // emit: 3 + enum-ref(0) + payload item (name static + 0.9c literal) = 3
        // — the payload name is a label, never an undeclared-intrinsic error.
        assert_eq!(cost_of("(emit EventType/RUPTURE (severity 0.9c))"), Ok(3));
        // add-node field-init: 3 + enum-ref(0) + id(1) + (qname 0 + 5$ 0) = 4.
        assert_eq!(
            cost_of("(add-node NodeType/SOCIAL_CLASS n1 (social-class/wealth 5$))"),
            Ok(4)
        );
    }

    #[test]
    fn a_rule_over_its_declared_budget_is_e_load_040() {
        let starved = DEMO_RULE.replacen(":fuel 64", ":fuel 5", 1);
        let err = check_rule(&e(&starved), &ceilings(), &intrinsics()).unwrap_err();
        assert_eq!(
            err,
            BoundError::BoundExceeded {
                rule_id: "demo/hunger".to_owned(),
                computed_bound: 7,
                declared_budget: 5,
            }
        );
        assert_eq!(err.spec_code(), Some("E-LOAD-040"));
    }

    #[test]
    fn an_unconditional_rule_bounds_over_effects_alone() {
        let rule = e(r#"
            (rule demo/always
              :role mechanic :evidence derived :material-basis "unconditional per §2.3"
              :fuel 8
              (bindings)
              (effects
                (update-node self social-class/agitation (add 0.05i))))
        "#);
        assert_eq!(check_rule(&rule, &ceilings(), &intrinsics()), Ok(5));
    }

    #[test]
    fn an_over_long_member_list_is_rejected_statically_as_e_load_042() {
        let rule = e(r#"
            (rule demo/roster
              :role mechanic :evidence derived :material-basis "membership is whole-hyperedge replacement"
              :fuel 64
              (bindings)
              (effects
                (guard #t
                  (add-hyperedge HyperedgeType/PAIR h1 (members a b c)))))
        "#);
        let err = check_rule(&rule, &ceilings(), &intrinsics()).unwrap_err();
        assert_eq!(
            err,
            BoundError::MemberListOverCeiling {
                hyperedge_type: "HyperedgeType/PAIR".to_owned(),
                member_count: 3,
                max_members: 2,
            }
        );
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    #[test]
    fn a_missing_fuel_declaration_is_a_loud_error() {
        let rule = e(r#"
            (rule demo/no-fuel
              :role mechanic :evidence derived :material-basis "fuel is mandatory on every rule"
              (bindings)
              (effects (update-node self social-class/agitation (add 0.05i))))
        "#);
        assert!(matches!(
            check_rule(&rule, &ceilings(), &intrinsics()),
            Err(BoundError::Malformed { .. })
        ));
    }

    #[test]
    fn a_saturated_bound_still_fails_loud() {
        let ceilings = CardinalityCeilings::new(
            HashMap::from([("NodeType/SOCIAL_CLASS".to_owned(), u64::MAX)]),
            HashMap::new(),
        );
        let expr = e("(fold sum (nodes NodeType/SOCIAL_CLASS) it)");
        let bound = expr_cost(&expr, &ceilings, &intrinsics()).unwrap();
        assert_eq!(bound, u64::MAX, "overflow saturates, never wraps");
    }
}
