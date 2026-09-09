//! Static shape checks over the §2 grammar that no other module owns: the
//! **enum-ref operand class rule** (D74) and the **field-init owner rule**
//! (D37). Both are load-time and both are stated once here rather than per
//! form, which is exactly the repair D74 records — "the R9 chapters added
//! four such positions and a per-form restatement left each new one without
//! a rejection".
//!
//! `E-TYPE-011` is a **kind** check and nothing more: whether the named type
//! and member exist at all is `E-LOAD-030`/`E-LOAD-031`
//! ([`crate::vocabulary`]).

use crate::reader::{Atom, SExpr};
use crate::vocabulary::{ClosedVocabulary, EnumKind, VocabularyError};

/// A static shape rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrammarError {
    /// `E-TYPE-011` — an `<enum-ref>` operand naming a member of the wrong
    /// enum kind for its position (D74).
    WrongEnumKind {
        /// The form head whose operand is wrong.
        form: String,
        /// Which operand (1-based, after the head).
        operand: usize,
        /// The kind the position demands.
        expected: EnumKind,
        /// What was written, as `EnumType/MEMBER`.
        found: String,
    },
    /// `E-PARSE-041` — an `add-edge` `<field-init>` naming the implicit
    /// `<edge-type>/strength` field, whose only writer at mint time is the
    /// `:strength` operand (D37).
    StrengthFieldInit {
        /// The offending qname.
        field: String,
    },
    /// `E-PARSE-040` — arithmetic that is not strictly binary; D75 keeps
    /// this as the arithmetic-specific spelling of the arity class.
    ArithmeticArity {
        /// The operator.
        operator: String,
        /// How many operands were written.
        found: usize,
    },
    /// `E-PARSE-042` — a form whose operand count differs from the one its
    /// §2 production fixes (D75). The three-operand `neighbors` of the
    /// pre-C8 grammar is the case this revision creates (D51).
    Arity {
        /// The form head.
        form: String,
        /// How many operands were written (after `:as` is stripped).
        found: usize,
        /// What the production fixes.
        expected: &'static str,
    },
    /// `E-PARSE-015` — a head symbol that is not a member of a closed
    /// terminal set: `<fold-op>`, `<cmp>`, `<update-op>` or `<arith>`
    /// (D75). Two of §6.3's four silent-degradation corrections land here.
    NotInClosedSet {
        /// The offending symbol.
        symbol: String,
        /// Which closed set it was checked against.
        set: &'static str,
    },
    /// `E-PARSE-010` — a string literal in expression position (D75). §1.5
    /// admits strings at `:material-basis` and at conformance-vector
    /// identifiers only; `<expr>` has no string form and `Str` has no
    /// operations, so a string in a payload or a comparison is an atom
    /// rejected by **position**, not by lexis.
    StringInExpressionPosition {
        /// The offending literal.
        literal: String,
    },
    /// `E-PARSE-013` — the `:graph` flag outside a `domain` form (D42).
    /// The keyword set is closed and a misplaced keyword is never ignored.
    GraphFlagOutsideDomain,
    /// `E-TYPE-014` — a `<field-init>` whose owning type is not the element
    /// type the minting verb's `<enum-ref>` names (D37). Static on
    /// `add-node`/`add-edge`/`add-hyperedge`; the same disagreement on the
    /// update verbs surfaces at evaluation as `E-EVAL-033`.
    FieldInitOwnerMismatch {
        /// The offending qname.
        field: String,
        /// The type the verb mints, as `EnumType/MEMBER`.
        verb_type: String,
        /// The type the field owns off, as `EnumType/MEMBER`.
        owner: String,
    },
    /// `E-LOAD-023` / `E-LOAD-030` / `E-LOAD-031` (Task 8, Organization
    /// foundation plan) — a closed-vocabulary failure surfacing through
    /// this module's own checks: `check_field_init_owners`'s `owner_of`
    /// lookup (`E-LOAD-023`, previously silently skipped — "the
    /// declaration reader's rejection" was true only for a field's OWN
    /// `deffield`, never for a field-init naming a segment no `deffield`
    /// ever declared) and [`check_enum_ref_membership`]
    /// (`E-LOAD-030`/`E-LOAD-031`).
    Vocabulary {
        /// The verb/form whose enum-ref or field-init failed (F6, #534 fix
        /// round item 6) — §4.6's own house style ("load-time errors
        /// report the offending form") for a class this crate did not
        /// itself originate.
        form: String,
        /// The underlying vocabulary fact.
        error: VocabularyError,
    },
}

impl GrammarError {
    /// The spec's error code.
    #[must_use]
    pub fn spec_code(&self) -> &'static str {
        match self {
            Self::WrongEnumKind { .. } => "E-TYPE-011",
            Self::ArithmeticArity { .. } => "E-PARSE-040",
            Self::Arity { .. } => "E-PARSE-042",
            Self::NotInClosedSet { .. } => "E-PARSE-015",
            Self::StringInExpressionPosition { .. } => "E-PARSE-010",
            Self::GraphFlagOutsideDomain => "E-PARSE-013",
            Self::StrengthFieldInit { .. } => "E-PARSE-041",
            Self::FieldInitOwnerMismatch { .. } => "E-TYPE-014",
            Self::Vocabulary { error, .. } => error.spec_code(),
        }
    }
}

impl std::fmt::Display for GrammarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongEnumKind {
                form,
                operand,
                expected,
                found,
            } => write!(
                f,
                "E-TYPE-011: ({form} …) operand {operand} takes a {} member, \
                 found {found} (§2.6's class rule, D74)",
                expected.type_name()
            ),
            Self::ArithmeticArity { operator, found } => write!(
                f,
                "E-PARSE-040: ({operator} …) is strictly binary; {found} \
                 operands were written. The reduction order stays explicit in \
                 the source rather than implied by a left-fold convention \
                 (§2.7)"
            ),
            Self::Arity {
                form,
                found,
                expected,
            } => write!(
                f,
                "E-PARSE-042: ({form} …) takes {expected} operands, found \
                 {found} (§2.7, D75)"
            ),
            Self::NotInClosedSet { symbol, set } => write!(
                f,
                "E-PARSE-015: '{symbol}' is not a member of the closed {set} \
                 set — it is never ignored and never reads as false (§6.3)"
            ),
            Self::StringInExpressionPosition { literal } => write!(
                f,
                "E-PARSE-010: the string {literal:?} is in expression position; \
                 §1.5 admits strings at :material-basis and vector ids only, and \
                 Str has no operations (§3.1) — an atom rejected by position"
            ),
            Self::GraphFlagOutsideDomain => write!(
                f,
                "E-PARSE-013: :graph is a flag keyword of the `domain` form and \
                 is illegal elsewhere (§1.6, D42)"
            ),
            Self::StrengthFieldInit { field } => write!(
                f,
                "E-PARSE-041: the :strength operand is {field}'s only writer at \
                 mint time — two writers for one field in one form is an \
                 authoring bug (§2.8)"
            ),
            Self::FieldInitOwnerMismatch {
                field,
                verb_type,
                owner,
            } => write!(
                f,
                "E-TYPE-014: field-init {field} owns off {owner}, but the verb \
                 mints a {verb_type} (§2.8)"
            ),
            Self::Vocabulary { form, error } => write!(f, "({form} …): {error}"),
        }
    }
}

impl std::error::Error for GrammarError {}

/// Every `<enum-ref>` operand position the language has, as
/// `(form head, 1-based operand index, demanded kind)` — D74's list plus
/// §2.8's own per-verb typing, which D74 states as one class.
const ENUM_REF_POSITIONS: [(&str, usize, EnumKind); 16] = [
    ("nodes", 1, EnumKind::NodeType),
    ("edges", 1, EnumKind::EdgeType),
    ("neighbors", 2, EnumKind::EdgeType),
    ("neighbors", 4, EnumKind::NodeType),
    ("hyperedges", 1, EnumKind::HyperedgeType),
    ("members-of", 2, EnumKind::HyperedgeType),
    ("hyperedges-of", 2, EnumKind::HyperedgeType),
    ("the", 1, EnumKind::NodeType),
    ("edge-between", 1, EnumKind::EdgeType),
    ("domain", 1, EnumKind::NodeType),
    ("emit", 1, EnumKind::EventType),
    ("add-node", 1, EnumKind::NodeType),
    ("add-edge", 1, EnumKind::EdgeType),
    ("remove-edge", 1, EnumKind::EdgeType),
    ("add-hyperedge", 1, EnumKind::HyperedgeType),
    ("remove-node", usize::MAX, EnumKind::NodeType), // no enum-ref operand
];

/// The kind demanded at `(head, operand)`, if that position is typed.
fn demanded_kind(head: &str, operand: usize) -> Option<EnumKind> {
    ENUM_REF_POSITIONS
        .iter()
        .find(|(h, i, _)| *h == head && *i == operand)
        .map(|(_, _, kind)| *kind)
}

/// Walk a form tree and apply D74's class rule to every typed operand
/// position. Untyped positions are left alone: an enum-ref elsewhere is a
/// value (§3.1's `Enum<T>`), not a mis-kinded operand.
///
/// # Errors
///
/// [`GrammarError::WrongEnumKind`] (`E-TYPE-011`).
pub fn check_enum_ref_kinds(expr: &SExpr) -> Result<(), GrammarError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        for (operand, child) in items.iter().enumerate().skip(1) {
            let SExpr::Atom(Atom::EnumRef { enum_type, member }) = child else {
                continue;
            };
            let Some(expected) = demanded_kind(head, operand) else {
                continue;
            };
            let written = EnumKind::from_type_name(enum_type);
            if written != Some(expected) {
                return Err(GrammarError::WrongEnumKind {
                    form: head.clone(),
                    operand,
                    expected,
                    found: format!("{enum_type}/{member}"),
                });
            }
        }
    }
    for child in items {
        check_enum_ref_kinds(child)?;
    }
    Ok(())
}

/// Task 8 (Organization foundation plan): the SIBLING pass to
/// [`check_enum_ref_kinds`] over the SAME sixteen typed positions —
/// [`check_enum_ref_kinds`] proves an already-present enum-ref names the
/// right KIND for its position (D74); this proves it names a REGISTERED
/// MEMBER of the scenario's declared closed vocabulary (§3.6). It runs
/// only when a vocabulary was declared (`rule_pipeline::load_rule_form`
/// gates the call on `ctx.vocabulary_registry`), and it runs AFTER
/// [`check_enum_ref_kinds`] in that pipeline, so every enum-ref this walk
/// inspects is already guaranteed kind-correct for its position —
/// `ClosedVocabulary::check_enum_ref` can therefore only ever raise its
/// MEMBER half here (`E-LOAD-031`); its type half (`E-LOAD-030`) is
/// [`check_enum_ref_kinds`]'s own `WrongEnumKind`/`E-TYPE-011` failure
/// mode for these positions, not unreachable by design — just refused
/// earlier, under a different code.
///
/// Restricted to the SAME typed positions (not every enum-ref anywhere in
/// the rule) on purpose: an untyped enum-ref may legitimately name a
/// CONTENT-DECLARED custom enum type (`OrgKind`, [`crate::types::
/// EnumRegistry`]'s own registry, Tasks 3–6) rather than one of the four
/// structural kinds [`ClosedVocabulary`] governs. Walking every position
/// indiscriminately would refuse `(= kind OrgKind/BUSINESS)` the moment
/// ANY `defvocabulary` was declared anywhere in the scenario — coupling
/// two registries that must stay independent.
///
/// **Payload-LABEL-only at `emit`, corrected (G1, #534 fix round 2 — one
/// root cause with the sibling fix in `structural_verbs::
/// find_deferred_shape_verb`; supersedes F5(a), #534 fix round item 5,
/// which itself mirrored `check_type_operands_are_enum_refs`'s R2 fix,
/// #528 delta-verify rider).** `emit`'s `<payload-item>` label is an
/// unconstrained `Atom::Symbol` (§2.8's `<payload-item> ::= (<symbol>
/// <expr>)`) — nothing stops content from naming one `emit`, with a VALUE
/// that happens to be a coincidentally well-kinded-but-unregistered
/// `<enum-ref>`. Before F5(a), the unconditional recursion below
/// re-inspected such a payload item as a FRESH `emit` invocation and
/// wrongly refused its "operand 1" — a value, not `emit`'s own type
/// operand — under `E-LOAD-031`.
///
/// F5(a)'s own fix over-corrected: returning `Ok(())` once `head_is_emit`
/// skipped `emit`'s ENTIRE subtree, not just the payload LABELS — so a
/// payload item's VALUE (element 1 of `(<symbol> <expr>)`, an arbitrary
/// `<expr>` that may itself carry a typed position, e.g. `(the
/// NodeType/NOWHERE)`) escaped this walk too, silently. `emit` having
/// exactly ONE typed position of its own (`ENUM_REF_POSITIONS`, checked in
/// the loop above) bounds `emit`'s OWN operand list, not what a payload
/// VALUE may nest. The corrected discipline (G1): once `emit`'s own
/// operand-1 check has run, recurse into each payload item's element-1
/// VALUE ONLY — never treating the payload item itself as a form headed by
/// its LABEL (element 0).
///
/// **G1's own fix still assumed two invariants no check here established
/// (H1, #534 fix round 3 — one root cause with `structural_verbs::
/// find_deferred_shape_verb`'s own sibling fix): that `items[1]` really is
/// `emit`'s type operand (so `items[2..]` are genuinely payload items),
/// and that a payload item has exactly two elements (so `pair.get(1)`
/// finds its whole value).** Neither holds once a SECOND malformation is
/// present. `(m (emit (the NodeType/NOWHERE)))` is a payload item whose
/// value is a NESTED `emit` MISSING its own type operand — `items[1]`
/// there is `(the NodeType/NOWHERE)` itself, not an `<enum-ref>`, and the
/// old unconditional `skip(2)` silently treated it as "the confirmed type
/// operand" and skipped it, so its own typed position never got checked.
/// `(m 1 (the NodeType/NOWHERE))` is an OVER-ARITY payload item — three
/// elements, not two — so `pair.get(1)` (the literal `1`) never reached
/// `pair[2]`. Corrected: `items[1]` is only trusted as `emit`'s type
/// operand when it is genuinely an `<enum-ref>` (`items[2..]` payload,
/// `pair[1..]` every value); otherwise no positional assumption is safe
/// once the type-operand slot itself is broken, so every item after the
/// head is recursed into in full (`items[1..]`, ordinary recursion) —
/// `check_type_operands_are_enum_refs` is the earlier load-time gate that
/// refuses this exact malformed-nested-`emit` shape outright, but this
/// function must not rely on that gate having run first (it is driven
/// directly by this module's own tests, same as its two siblings).
///
/// **Several of the sixteen typed positions cannot fire through the
/// production load pipeline in slice 1.** `add-node`, `add-edge`,
/// `remove-edge` and `add-hyperedge` are four of `ENUM_REF_POSITIONS`'
/// rows, but every rule using any of them is ALREADY refused, earlier and
/// unconditionally, by [`crate::structural_verbs::
/// check_no_deferred_shape_verbs`] (`rule_pipeline::load_rule_form` calls
/// it before this function's own gating block even opens) — so a rule
/// reaching this walk at all has none of the six graph-shape verbs
/// anywhere in it, and these four rows' own membership checks are dead
/// code in that pipeline (live only for this module's own direct unit
/// tests, which drive `check_enum_ref_membership` without that earlier
/// gate). The remaining twelve positions (`nodes`, `edges`, `neighbors`
/// ×2, `hyperedges`, `members-of`, `hyperedges-of`, `the`,
/// `edge-between`, `domain`, `emit`) carry no such earlier refusal and
/// fire in production exactly as this doc otherwise describes.
///
/// # Errors
///
/// [`GrammarError::Vocabulary`] wrapping [`VocabularyError::UnknownEnumMember`]
/// (`E-LOAD-031`) in practice at these positions.
pub fn check_enum_ref_membership(
    expr: &SExpr,
    vocabulary: &ClosedVocabulary,
) -> Result<(), GrammarError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    let mut head_is_emit = false;
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        head_is_emit = head == "emit";
        for (operand, child) in items.iter().enumerate().skip(1) {
            let SExpr::Atom(Atom::EnumRef { enum_type, member }) = child else {
                continue;
            };
            if demanded_kind(head, operand).is_none() {
                continue;
            }
            vocabulary
                .check_enum_ref(enum_type, member)
                .map_err(|error| GrammarError::Vocabulary {
                    form: head.clone(),
                    error,
                })?;
        }
    }
    if head_is_emit {
        // Payload-LABEL-only (G1), guarded (H1, #534 fix round 3):
        // `items[1]` is emit's type operand ONLY when it is genuinely an
        // `<enum-ref>` — a malformed nested emit (missing its own type
        // operand) has no such guarantee when this function is driven
        // directly. Well-formed: skip the confirmed type operand
        // (`items[2..]`) and descend into every element AFTER each
        // payload item's label (`pair[1..]`, not just `pair.get(1)` — an
        // over-arity payload item has more than one value). Malformed:
        // fall back to full recursion over every item after the head,
        // since no positional assumption is safe once the type-operand
        // slot itself is broken.
        if matches!(items.get(1), Some(SExpr::Atom(Atom::EnumRef { .. }))) {
            for payload_item in items.iter().skip(2) {
                if let SExpr::List(pair) = payload_item {
                    for value in pair.iter().skip(1) {
                        check_enum_ref_membership(value, vocabulary)?;
                    }
                }
            }
        } else {
            for item in items.iter().skip(1) {
                check_enum_ref_membership(item, vocabulary)?;
            }
        }
        return Ok(());
    }
    for child in items {
        check_enum_ref_membership(child, vocabulary)?;
    }
    Ok(())
}

/// The type-operand position (operand 1) of `emit`/`add-node`/`add-edge`
/// — unlike the six §2.6 query heads and `add-hyperedge`, whose own type
/// operand IS extracted and validated at load by
/// `bound_checker::enum_ref_key` (`bound_checker.rs:189-196`), these
/// three verbs' type operand is folded into the §3.7 static cost pass's
/// generic per-operand sum (`bound_checker::verb_cost`/`atom_cost`),
/// which prices a `BareUpperIdent` atom identically to an `<enum-ref>`
/// (cost 0) — so a `Type_MEMBER` typo (slash typo'd as underscore,
/// lexing as `Atom::BareUpperIdent` since the §2.13 lexer widening,
/// D101) survives every load-time check and dies mid-tick, uncoded, at
/// `structural_verbs.rs`'s own `enum_member` (#528 fix round Item D).
///
/// **`remove-edge`** (#528 delta-verify rider R1) shares the exact same
/// uncovered shape — its type operand is ALSO folded into the generic
/// per-operand cost sum rather than extracted by `enum_ref_key` — even
/// though it mints nothing (it is a REMOVAL verb); the const/function
/// names below were widened, and renamed off "minting", accordingly.
///
/// `add-node`/`add-edge`/`remove-edge` are ALSO three of the six
/// graph-shape verbs `structural_verbs::check_no_deferred_shape_verbs`
/// refuses unconditionally at load (§4.2 chapter C4's collect-then-apply
/// gap, #519 fix round) — every rule using any of them is refused there
/// regardless of this gate, so in `rule_pipeline::load_rule_form`'s full
/// pipeline THIS check changes no observable outcome for them today. It
/// still earns its place: it runs EARLIER in that pipeline (naming the
/// actual shape defect rather than the unrelated deferred-verb gap when
/// a rule happens to carry both), it is the only gate at all for a
/// caller that drives this module's checks directly (as this module's
/// own tests do), and it stops being redundant the day these verbs gain
/// collect-then-apply support. `emit` carries no such second gate — this
/// is its ONLY load-time shape check.
const TYPE_OPERAND_HEADS: [&str; 4] = ["emit", "add-node", "add-edge", "remove-edge"];

/// Walk a form tree and refuse a non-`<enum-ref>` child at
/// `TYPE_OPERAND_HEADS`'s one typed position (operand 1) — mirrors
/// `bound_checker::enum_ref_key`'s own refusal for the sibling positions
/// it already gates (the six §2.6 query heads, `add-hyperedge`); see that
/// const's own doc for why these four specifically need a SEPARATE gate
/// rather than reuse of that one.
///
/// **Payload-LABEL-only, corrected (H1, #534 fix round 3 — the third
/// walker G1 (#534 fix round 2) should have fixed alongside its two
/// siblings, `grammar::check_enum_ref_membership` and `structural_verbs::
/// find_deferred_shape_verb`; supersedes the original R2 fix, #528
/// delta-verify rider).** `emit`'s `<payload-item>` label is an
/// unconstrained `Atom::Symbol` (§2.8's `<payload-item> ::= (<symbol>
/// <expr>)`), so a payload item happening to be labeled `emit` — a LABEL,
/// never a nested verb invocation — was wrongly refused as if it were one
/// before R2: `(emit EventType/RUPTURE (emit 5) (severity 1))` errored
/// although it is well-formed content.
///
/// R2's own fix over-corrected: stopping entirely once a matched head's
/// own operand 1 is confirmed skipped the WHOLE rest of that head's
/// subtree, not just the payload LABELS — so a payload item's VALUE (an
/// arbitrary `<expr>` that may itself be headed by a `TYPE_OPERAND_HEADS`
/// member, MISSING its own type operand) escaped this check too, silently
/// — the exact malformation this function exists to refuse, just one
/// level down: `(emit EventType/RUPTURE (m (emit (add-node
/// NodeType/SOCIAL_CLASS 5))))`'s inner `(emit (add-node …))` has no
/// `<enum-ref>` at its own operand 1, and R2's stop never reached it. The
/// corrected discipline: once a matched head's own operand 1 is
/// confirmed an `<enum-ref>` (so `items[2..]` are genuinely payload
/// items, never this verb's own further operands), recurse into each
/// payload item's element-1.. VALUES ONLY — never its element-0 LABEL,
/// and never just element 1 alone (an over-arity payload item like
/// `(m 1 (add-node …))` has more than one value to check — H1's own
/// second escape). `guard`/`for-each` are unaffected: neither is in
/// `TYPE_OPERAND_HEADS`, so a form headed by either still falls through
/// to the unconditional recursion below, reaching any REAL verb nested in
/// their bodies exactly as before.
///
/// # Errors
///
/// A plain message naming the form and what was found — uncoded, the
/// same "no §2 production reserves a number for this" precedent a
/// content-composition rejection already sets elsewhere in this crate,
/// and the same uncoded vocabulary `bound_checker::enum_ref_key` itself
/// uses for its sibling refusal ("expected an enum-ref where the grammar
/// requires one").
pub fn check_type_operands_are_enum_refs(expr: &SExpr) -> Result<(), String> {
    if let SExpr::List(items) = expr {
        if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
            if TYPE_OPERAND_HEADS.contains(&head.as_str()) {
                match items.get(1) {
                    Some(SExpr::Atom(Atom::EnumRef { .. })) => {}
                    other => {
                        return Err(format!(
                            "({head} …) operand 1 must be an <enum-ref> — expected an \
                             enum-ref where the grammar requires one, found {other:?} \
                             (#528 fix round Item D; remove-edge added #528 delta-verify \
                             rider R1)"
                        ));
                    }
                }
                // Payload-LABEL-only (H1): `items[1]` is now CONFIRMED an
                // `<enum-ref>` by the match above, so `items[2..]` are
                // genuinely payload items — recurse into each one's
                // element-1.. VALUES (never its element-0 LABEL) so a
                // NESTED occurrence of `TYPE_OPERAND_HEADS` lurking inside
                // a payload value still gets its own arity checked.
                for payload_item in items.iter().skip(2) {
                    if let SExpr::List(pair) = payload_item {
                        for value in pair.iter().skip(1) {
                            check_type_operands_are_enum_refs(value)?;
                        }
                    }
                }
                return Ok(());
            }
        }
        for child in items {
            check_type_operands_are_enum_refs(child)?;
        }
    }
    Ok(())
}

/// The three minting verbs whose element type is an operand, so D37's
/// field-init owner check is **static** there (§2.8).
const MINTING_VERBS: [&str; 3] = ["add-node", "add-edge", "add-hyperedge"];

/// Walk a form tree and apply D37 to every `<field-init>` of a minting
/// verb: a qname naming the implicit `<edge-type>/strength` on `add-edge`
/// is `E-PARSE-041`, and one owning off a different type is `E-TYPE-014`.
///
/// # Errors
///
/// [`GrammarError::StrengthFieldInit`] / [`GrammarError::FieldInitOwnerMismatch`].
pub fn check_field_init_owners(
    expr: &SExpr,
    vocabulary: &ClosedVocabulary,
) -> Result<(), GrammarError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        if MINTING_VERBS.contains(&head.as_str()) {
            check_one_verbs_field_inits(head, items, vocabulary)?;
        }
    }
    for child in items {
        check_field_init_owners(child, vocabulary)?;
    }
    Ok(())
}

fn check_one_verbs_field_inits(
    head: &str,
    items: &[SExpr],
    vocabulary: &ClosedVocabulary,
) -> Result<(), GrammarError> {
    let Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) = items.get(1) else {
        // A mis-shaped `add-hyperedge` type-operand is the bound checker's
        // rejection (`bound_checker::check_member_lists`, via
        // `enum_ref_key`). `add-node`/`add-edge`'s is
        // `check_type_operands_are_enum_refs`, THIS module's own
        // sibling gate — reached earlier in `rule_pipeline::
        // load_rule_form`'s pipeline than this function ever runs (it is
        // only called when `head` is a `MINTING_VERBS` member, gated by
        // `check_field_init_owners`, itself after `check_enum_ref_kinds`
        // wires `check_type_operands_are_enum_refs` alongside it —
        // #528 fix round Item D). Refusing `Ok(())` here too is not
        // reachable through that pipeline for a mis-shaped operand, but
        // stays correct — and honest — for any caller that drives this
        // function directly, bypassing the earlier gate.
        return Ok(());
    };
    let verb_type = format!("{enum_type}/{member}");
    for child in &items[2..] {
        let SExpr::List(pair) = child else { continue };
        let Some(SExpr::Atom(Atom::QName(field))) = pair.first() else {
            continue; // `(members …)` and non-field-init operands
        };
        let segment = field.split('/').next().unwrap_or(field);
        let leaf = field.rsplit('/').next().unwrap_or(field);
        if head == "add-edge" && leaf == "strength" {
            return Err(GrammarError::StrengthFieldInit {
                field: field.clone(),
            });
        }
        // Task 8 (Organization foundation plan): this used to `continue`
        // here on the theory that "E-LOAD-023 is the declaration reader's
        // rejection" — true only for a field's OWN `deffield` (§2.9's own
        // check, `declarations.rs`), never for a field-init HERE naming a
        // segment no `deffield` ever declared at all. That segment is a
        // typo a rule can carry silently past every OTHER load-time gate;
        // propagating makes it loud instead.
        //
        // **Not reachable through the production load pipeline in slice 1
        // (F4, #534 fix round item 4)** — the same caveat
        // `check_enum_ref_membership`'s own doc states for its sibling
        // pass, undisclosed here until now. `check_field_init_owners`
        // (this function's caller) only runs inside
        // `rule_pipeline::load_rule_form`'s `ctx.vocabulary_registry`-gated
        // block, and no shipped `.bscn` content set declares a
        // `defvocabulary` yet — `prepare_rules` (`babylon-tick/src/
        // lib.rs`) threads whatever the scenario declared, which today is
        // always `None`. It becomes reachable the day a real content set
        // opts in — Task 10's `organization-foundation.bscn` (the
        // Organization foundation plan) is the first one that will. Live
        // today for this crate's own direct callers (`check_
        // field_init_owners`'s unit tests below drive it with a
        // hand-built `ClosedVocabulary`), and for whenever that content
        // lands.
        let (owner_kind, owner_member) =
            vocabulary
                .owner_of(segment)
                .map_err(|error| GrammarError::Vocabulary {
                    form: head.to_owned(),
                    error,
                })?;
        let owner = format!("{}/{owner_member}", owner_kind.type_name());
        if owner != verb_type {
            return Err(GrammarError::FieldInitOwnerMismatch {
                field: field.clone(),
                verb_type,
                owner,
            });
        }
    }
    Ok(())
}

/// The closed terminal sets §2 fixes, and the arities its productions do.
/// `min`/`max` are operand counts **after** an optional `:as <symbol>` is
/// stripped; `usize::MAX` means "no upper bound" (a variadic body).
const ARITIES: [(&str, usize, usize, &str); 21] = [
    ("nodes", 1, 2, "1 (or 2 with a predicate)"),
    ("edges", 1, 2, "1 (or 2 with a predicate)"),
    ("hyperedges", 1, 2, "1 (or 2 with a predicate)"),
    // D51: the mandatory fourth operand — element expr, EdgeType,
    // direction, result NodeType.
    ("neighbors", 4, 4, "exactly 4"),
    ("members-of", 2, 2, "exactly 2"),
    ("hyperedges-of", 2, 2, "exactly 2"),
    ("the", 1, 1, "exactly 1"),
    ("field-of", 2, 2, "exactly 2"),
    ("edge-between", 3, 3, "exactly 3"),
    ("metric-of", 2, 2, "exactly 2"),
    ("domain", 1, 1, "exactly 1"),
    ("if", 3, 3, "exactly 3"),
    ("not", 1, 1, "exactly 1"),
    ("quantize-mass", 1, 1, "exactly 1"),
    ("forall", 2, 2, "exactly 2"),
    ("exists", 1, 2, "1 (or 2 with a body)"),
    ("select-max", 2, 2, "exactly 2"),
    ("select-min", 2, 2, "exactly 2"),
    ("update-node", 3, 3, "exactly 3"),
    ("update-edge", 3, 3, "exactly 3"),
    ("update-hyperedge", 3, 3, "exactly 3"),
];

/// The closed five-member `<fold-op>` set (§2.7), minted as a real sum type
/// (CT4P A3, issue #525) so the downstream dispatch sites
/// (`typecheck.rs::typecheck_aggregation`, `evaluator.rs::eval_fold`,
/// `score_class.rs::classify_fold`, `rule_pipeline.rs`) match over it
/// EXHAUSTIVELY, with no wildcard arm — the closed-vocabulary invariant
/// (S-22, `ai/bsl-architecture-standard.md:648`) enforced by the compiler,
/// not by five independently-maintained `&str` comparisons. Adding a sixth
/// fold-op today is five string edits and zero compile errors; adding a
/// sixth `FoldOp` variant is a compile error at every match that does not
/// yet decide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldOp {
    /// `sum` — extensive-only (§3.4 row 1).
    Sum,
    /// `mean` — the quotient of two extensive monoids (§3.4 rows 2-3).
    Mean,
    /// `min` — kind-neutral, always legal (§3.4 row 5).
    Min,
    /// `max` — kind-neutral, always legal (§3.4 row 5).
    Max,
    /// `count` — always legal, result `Int` (§3.4 row 6).
    Count,
}

impl FoldOp {
    /// The ONE place a `<fold-op>` symbol converts to its typed form — the
    /// grammar boundary. Every downstream site calls this rather than
    /// re-deriving the string-to-variant mapping. `None` for anything
    /// outside the closed set; each caller turns that into its OWN existing
    /// refusal (error text preserved byte-for-byte per site — error text is
    /// contract surface).
    #[must_use]
    pub fn parse(op: &str) -> Option<Self> {
        match op {
            "sum" => Some(Self::Sum),
            "mean" => Some(Self::Mean),
            "min" => Some(Self::Min),
            "max" => Some(Self::Max),
            "count" => Some(Self::Count),
            _ => None,
        }
    }

    /// The spec's own spelling, for error messages and re-derived source
    /// that need to name the op back.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sum => "sum",
            Self::Mean => "mean",
            Self::Min => "min",
            Self::Max => "max",
            Self::Count => "count",
        }
    }
}

/// The closed four-member `<update-op>` set (§2.8).
const UPDATE_OPS: [&str; 4] = ["add", "sub", "set", "scale"];
/// The closed four-member `<arith>` set and six-member `<cmp>` set (§2.4,
/// §2.7) — one lexical class, so one list.
const OPERATORS: [&str; 10] = ["<", "<=", ">", ">=", "=", "!=", "+", "-", "*", "/"];
/// The four `<arith>` members, which are strictly binary.
const ARITH: [&str; 4] = ["+", "-", "*", "/"];

/// Operand count with an optional `:as <symbol>` removed.
fn operand_count(items: &[SExpr]) -> usize {
    let mut count = 0;
    let mut i = 1;
    while i < items.len() {
        if let SExpr::Atom(Atom::Keyword(kw)) = &items[i] {
            if kw == "as" {
                i += 2;
                continue;
            }
        }
        count += 1;
        i += 1;
    }
    count
}

/// D75: apply every fixed arity and every closed terminal set, at load.
///
/// `fold` is checked separately from `ARITIES` because its own operand
/// count depends on whether a `:weight` is present, and because its
/// `<fold-op>` is the closed set this same pass checks.
///
/// # Errors
///
/// [`GrammarError::Arity`] (`E-PARSE-042`),
/// [`GrammarError::ArithmeticArity`] (`E-PARSE-040`),
/// [`GrammarError::NotInClosedSet`] (`E-PARSE-015`).
pub fn check_arities_and_closed_sets(expr: &SExpr) -> Result<(), GrammarError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    match items.first() {
        Some(SExpr::Atom(Atom::Operator(op))) => {
            if ARITH.contains(&op.as_str()) && items.len() != 3 {
                return Err(GrammarError::ArithmeticArity {
                    operator: op.clone(),
                    found: items.len() - 1,
                });
            }
            if !ARITH.contains(&op.as_str()) && OPERATORS.contains(&op.as_str()) && items.len() != 3
            {
                return Err(GrammarError::Arity {
                    form: op.clone(),
                    found: items.len() - 1,
                    expected: "exactly 2",
                });
            }
        }
        Some(SExpr::Atom(Atom::Symbol(head))) => {
            check_head_arity(head, items)?;
        }
        _ => {}
    }
    for child in items {
        check_arities_and_closed_sets(child)?;
    }
    Ok(())
}

fn check_head_arity(head: &str, items: &[SExpr]) -> Result<(), GrammarError> {
    let count = operand_count(items);
    if let Some((_, min, max, expected)) = ARITIES.iter().find(|(h, _, _, _)| *h == head) {
        if count < *min || count > *max {
            return Err(GrammarError::Arity {
                form: head.to_owned(),
                found: count,
                expected,
            });
        }
    }
    if head == "fold" {
        // (fold <fold-op> <query> <elem-name>? <expr> (:weight <expr>)?)
        // — 3 operands, or 5 with the weight keyword and its expression.
        if count != 3 && count != 5 {
            return Err(GrammarError::Arity {
                form: head.to_owned(),
                found: count,
                expected: "3 (or 5 with :weight)",
            });
        }
        if let Some(SExpr::Atom(Atom::Symbol(op))) = items.get(1) {
            if FoldOp::parse(op.as_str()).is_none() {
                return Err(GrammarError::NotInClosedSet {
                    symbol: op.clone(),
                    set: "<fold-op>",
                });
            }
        }
    }
    // An `<update-op>` sits in the third operand of the three update verbs;
    // a fifth head there — the `(unset …)` the frozen estate reaches for —
    // is E-PARSE-015 (§2.8: the set is closed).
    if matches!(head, "update-node" | "update-edge" | "update-hyperedge") {
        if let Some(SExpr::List(op_items)) = items.get(3) {
            if let Some(SExpr::Atom(Atom::Symbol(op))) = op_items.first() {
                if !UPDATE_OPS.contains(&op.as_str()) {
                    return Err(GrammarError::NotInClosedSet {
                        symbol: op.clone(),
                        set: "<update-op>",
                    });
                }
            }
        }
    }
    Ok(())
}

/// D75 / §3.8 item 4: a string literal anywhere in a rule's `<when>` or
/// `<effects>` is `E-PARSE-010`. Transcribed systems carrying `predicate`
/// or `description` strings convert them to enum-refs or drop them — the
/// rule id already identifies the rule, and an event whose payload restates
/// its own provenance in prose is carrying a log line, not state.
///
/// The walk covers the `<when>` and `<effects>` bodies **and every `:expr`
/// binding operand** (§2.5) — a `:expr` is an expression position like any
/// other, and the crate's sibling passes (`scope::check_foreign_field_scoping`,
/// `domain::reference_sites`) already treat it as one. It starts *below*
/// the rule's own options, so `:material-basis`'s string — a rule-level
/// option, not an expression — is never reached.
///
/// # Errors
///
/// [`GrammarError::StringInExpressionPosition`].
pub fn check_string_positions(rule: &SExpr) -> Result<(), GrammarError> {
    let SExpr::List(items) = rule else {
        return Ok(());
    };
    for child in items {
        let SExpr::List(inner) = child else { continue };
        match inner.first() {
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "when" || h == "effects" => {
                for body in &inner[1..] {
                    walk_for_strings(body)?;
                }
            }
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "bindings" => {
                for row in &inner[1..] {
                    let SExpr::List(cells) = row else { continue };
                    for window in cells.windows(2) {
                        if let [SExpr::Atom(Atom::Keyword(kw)), operand] = window {
                            if kw == "expr" {
                                walk_for_strings(operand)?;
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

fn walk_for_strings(expr: &SExpr) -> Result<(), GrammarError> {
    match expr {
        SExpr::Atom(Atom::Str(s)) => {
            Err(GrammarError::StringInExpressionPosition { literal: s.clone() })
        }
        SExpr::Atom(_) => Ok(()),
        SExpr::List(items) => {
            for child in items {
                walk_for_strings(child)?;
            }
            Ok(())
        }
    }
}

/// D42: `:graph` is legal only inside a `domain` form; anywhere else it is
/// an unrecognized keyword in that position and `E-PARSE-013` — never
/// ignored, because the keyword set is closed (§1.6).
///
/// # Errors
///
/// [`GrammarError::GraphFlagOutsideDomain`].
pub fn check_graph_flag_placement(expr: &SExpr) -> Result<(), GrammarError> {
    let SExpr::List(items) = expr else {
        return Ok(());
    };
    let in_domain_form =
        matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "domain");
    for child in items {
        if !in_domain_form {
            if let SExpr::Atom(Atom::Keyword(kw)) = child {
                if kw == "graph" {
                    return Err(GrammarError::GraphFlagOutsideDomain);
                }
            }
        }
        check_graph_flag_placement(child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        check_enum_ref_kinds, check_enum_ref_membership, check_field_init_owners, GrammarError,
    };
    use crate::reader::read;
    use crate::vocabulary::{ClosedVocabulary, EnumKind, VocabularyError};

    fn vocabulary() -> ClosedVocabulary {
        ClosedVocabulary::new([
            (
                EnumKind::NodeType,
                vec!["SOCIAL_CLASS".to_owned(), "POLITY".to_owned()],
            ),
            (EnumKind::EdgeType, vec!["SOLIDARITY".to_owned()]),
            (EnumKind::HyperedgeType, vec!["COMMUNITY".to_owned()]),
            (EnumKind::EventType, vec!["RUPTURE".to_owned()]),
        ])
        .unwrap()
    }

    fn e(source: &str) -> crate::reader::SExpr {
        read(source).expect("test source must parse").0
    }

    #[test]
    fn every_typed_operand_position_rejects_the_wrong_enum_kind() {
        for source in [
            "(nodes EdgeType/SOLIDARITY)",
            "(edges NodeType/SOCIAL_CLASS)",
            "(hyperedges NodeType/SOCIAL_CLASS)",
            "(members-of h NodeType/SOCIAL_CLASS)",
            "(hyperedges-of n EdgeType/SOLIDARITY)",
            "(the EdgeType/SOLIDARITY)",
            "(edge-between NodeType/SOCIAL_CLASS a b)",
            "(domain EdgeType/SOLIDARITY)",
            "(emit NodeType/SOCIAL_CLASS)",
            "(add-node EdgeType/SOLIDARITY n1)",
            "(add-hyperedge EdgeType/SOLIDARITY h1 (members a b))",
        ] {
            let err = check_enum_ref_kinds(&e(source)).expect_err(source);
            assert_eq!(err.spec_code(), "E-TYPE-011", "{source}");
        }
    }

    #[test]
    fn neighbors_rejects_the_two_operands_swapped_at_both_positions() {
        // §2.6 (C8): operand 2 is the EdgeType traversed, operand 4 the
        // result NodeType. Swapping them is E-TYPE-011 at both.
        let err = check_enum_ref_kinds(&e(
            "(neighbors self NodeType/SOCIAL_CLASS :out EdgeType/SOLIDARITY)",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-011");
        assert!(check_enum_ref_kinds(&e(
            "(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)"
        ))
        .is_ok());
    }

    #[test]
    fn an_enum_ref_in_an_untyped_position_is_a_value_not_a_mis_kinded_operand() {
        // §3.1's `Enum<T>` compares with =/!=; that position types nothing.
        assert!(check_enum_ref_kinds(&e("(= NodeType/POLITY NodeType/POLITY)")).is_ok());
    }

    // ---- the minting/emitting type-operand shape gate (#528 fix round
    // Item D) — `check_enum_ref_kinds` above only checks the KIND of an
    // enum-ref that is already there; this is the sibling check that the
    // operand IS one in the first place, for the three positions
    // `bound_checker::enum_ref_key` does not itself reach. ----

    #[test]
    fn a_bare_upper_ident_at_the_minting_type_operand_position_is_refused() {
        // `Type_MEMBER` (slash typo'd as underscore) lexes as
        // `Atom::BareUpperIdent` (D101's lexer widening) — exactly the
        // shape this gate exists to catch.
        for source in [
            "(emit NodeType_SOCIAL_CLASS)",
            "(add-node NodeType_SOCIAL_CLASS n1)",
            "(add-edge EdgeType_SOLIDARITY a b :strength 0.5c)",
        ] {
            assert!(
                super::check_type_operands_are_enum_refs(&e(source)).is_err(),
                "{source}"
            );
        }
    }

    #[test]
    fn a_correctly_shaped_enum_ref_at_the_minting_type_operand_position_is_untouched() {
        for source in [
            "(emit EventType/RUPTURE)",
            "(add-node NodeType/SOCIAL_CLASS n1)",
            "(add-edge EdgeType/SOLIDARITY a b :strength 0.5c)",
        ] {
            assert!(
                super::check_type_operands_are_enum_refs(&e(source)).is_ok(),
                "{source}"
            );
        }
    }

    #[test]
    fn a_bare_upper_ident_at_remove_edges_type_operand_position_is_refused() {
        // #528 delta-verify rider R1: remove-edge shares the SAME
        // uncovered type-operand shape as emit/add-node/add-edge — it
        // mints nothing, but its type operand is folded into the same
        // generic per-operand cost sum, so the same typo survives every
        // load-time check the same way. Driven in isolation (this
        // function directly), not through the full rule pipeline: a
        // remove-edge rule is ALSO refused by
        // `structural_verbs::check_no_deferred_shape_verbs` at load,
        // which would mask this check entirely if driven end to end.
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(remove-edge EdgeType_SOLIDARITY a b)"
        ))
        .is_err());
    }

    #[test]
    fn a_correctly_shaped_remove_edge_is_untouched() {
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(remove-edge EdgeType/SOLIDARITY a b)"
        ))
        .is_ok());
    }

    #[test]
    fn a_payload_item_labeled_like_a_type_operand_head_is_never_over_refused() {
        // #528 delta-verify rider R2: a payload item's LABEL is an
        // unconstrained `Atom::Symbol` (§2.8's `<payload-item> ::=
        // (<symbol> <expr>)`) — nothing stops content from naming one
        // `emit`, and that label is not a nested verb invocation. The
        // buggy walk treated every child list's head as a fresh
        // candidate and wrongly refused this exact form; the fix makes
        // the check HEAD-POSITION-ONLY: once `emit`'s own type operand
        // is confirmed, its trailing payload items are never descended
        // into by this check at all.
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(emit EventType/RUPTURE (emit 5) (severity 1))"
        ))
        .is_ok());
        // The same shape with a different label was always Ok — proves
        // the probe isolates the label collision, not some other cause.
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(emit EventType/RUPTURE (foo 5) (severity 1))"
        ))
        .is_ok());
    }

    #[test]
    fn a_nested_emit_missing_its_own_type_operand_in_a_payload_value_still_refuses_type_operand() {
        // H1 (#534 fix round 3, residual of G1 — one root cause with
        // `structural_verbs::find_deferred_shape_verb`'s own sibling
        // fix): R2's head-position-only stop (the test above) never
        // recursed into a payload item's VALUE at all, so a SECOND
        // malformation nested there — here, a payload item whose value is
        // itself headed `emit` but MISSING its own type operand — escaped
        // this check entirely. `(m (emit (add-node NodeType/SOCIAL_CLASS
        // 5)))` is `emit`'s payload item labeled `m`, whose value
        // `(emit (add-node NodeType/SOCIAL_CLASS 5))` is a NESTED `emit`
        // invocation with no `<enum-ref>` at its own operand 1 — the
        // exact shape this whole function exists to refuse, just one
        // level down.
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(emit EventType/RUPTURE (m (emit (add-node NodeType/SOCIAL_CLASS 5))))"
        ))
        .is_err());
    }

    #[test]
    fn a_non_minting_head_with_a_bare_upper_ident_operand_is_untouched() {
        // Query heads and `add-hyperedge` are gated elsewhere
        // (`bound_checker::enum_ref_key`) — this function must not
        // overreach into their positions.
        assert!(
            super::check_type_operands_are_enum_refs(&e("(nodes NodeType_SOCIAL_CLASS)")).is_ok()
        );
    }

    #[test]
    fn a_bare_upper_ident_nested_inside_a_guard_still_refuses() {
        // The walk must recurse into (guard/…) bodies, not just the
        // top-level form.
        assert!(super::check_type_operands_are_enum_refs(&e(
            "(guard #t (emit NodeType_SOCIAL_CLASS))"
        ))
        .is_err());
    }

    #[test]
    fn an_add_edge_field_init_naming_strength_is_e_parse_041() {
        let err = check_field_init_owners(
            &e("(add-edge EdgeType/SOLIDARITY a b :strength 0.5c (solidarity/strength 0.9c))"),
            &vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-PARSE-041");
    }

    #[test]
    fn a_field_init_owning_off_the_wrong_type_is_e_type_014() {
        let err = check_field_init_owners(
            &e("(add-node NodeType/SOCIAL_CLASS n1 (polity/imperial-rent-pool 5$))"),
            &vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-014");
    }

    #[test]
    fn a_field_init_owning_off_the_verbs_own_type_accepts() {
        assert!(check_field_init_owners(
            &e("(add-node NodeType/SOCIAL_CLASS n1 (social-class/wealth 5$))"),
            &vocabulary(),
        )
        .is_ok());
        assert!(check_field_init_owners(
            &e("(add-edge EdgeType/SOLIDARITY a b :strength 0.5c (solidarity/tension 0.1i))"),
            &vocabulary(),
        )
        .is_ok());
    }

    // ---- Task 8 (Organization foundation plan): closed-vocabulary
    // enforcement — `owner_of`'s Err now propagates (E-LOAD-023), and the
    // new membership pass (E-LOAD-030/031) ----

    #[test]
    fn a_field_init_owning_off_an_unregistered_segment_is_e_load_023() {
        // Before this task: silently `continue`d past — "E-LOAD-023 is the
        // declaration reader's rejection" was true only for the field's OWN
        // `deffield`, never for a field-init here naming a segment no
        // `deffield` — nor any registered graph-element type — ever named.
        let err = check_field_init_owners(
            &e("(add-node NodeType/SOCIAL_CLASS n1 (imperium/rent 5$))"),
            &vocabulary(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-023");
        assert!(matches!(
            &err,
            GrammarError::Vocabulary {
                error: VocabularyError::UnknownFieldOwner { segment },
                ..
            } if segment == "imperium"
        ));
        // F6 (#534 fix round item 6): the offending verb is named too.
        assert!(err.to_string().contains("add-node"), "{err}");
    }

    #[test]
    fn an_unregistered_member_at_a_typed_position_is_e_load_031() {
        for source in [
            "(nodes NodeType/NOWHERE)",
            "(edges EdgeType/NOWHERE)",
            "(hyperedges HyperedgeType/NOWHERE)",
            "(the NodeType/NOWHERE)",
            "(domain NodeType/NOWHERE)",
            "(emit EventType/NOWHERE)",
            "(add-node NodeType/NOWHERE n1)",
            "(add-edge EdgeType/NOWHERE a b)",
            "(add-hyperedge HyperedgeType/NOWHERE h1 (members a b))",
        ] {
            let err = check_enum_ref_membership(&e(source), &vocabulary()).expect_err(source);
            assert_eq!(err.spec_code(), "E-LOAD-031", "{source}");
        }
    }

    #[test]
    fn a_registered_member_at_a_typed_position_is_untouched() {
        for source in [
            "(nodes NodeType/SOCIAL_CLASS)",
            "(emit EventType/RUPTURE)",
            "(add-node NodeType/SOCIAL_CLASS n1)",
            "(add-edge EdgeType/SOLIDARITY a b)",
            "(add-hyperedge HyperedgeType/COMMUNITY h1 (members a b))",
        ] {
            assert!(
                check_enum_ref_membership(&e(source), &vocabulary()).is_ok(),
                "{source}"
            );
        }
    }

    #[test]
    fn an_enum_ref_at_an_untyped_position_is_never_checked_for_membership() {
        // A comparison operand is not one of D74's sixteen typed
        // positions — an unregistered member there is a value, not a
        // mis-kinded or unregistered operand, and this pass must not
        // overreach into it (a content-declared custom enum type, e.g.
        // `OrgKind`, lives in a wholly different registry and must stay
        // uncoupled from this one).
        assert!(check_enum_ref_membership(
            &e("(= NodeType/NOWHERE NodeType/NOWHERE)"),
            &vocabulary(),
        )
        .is_ok());
        assert!(check_enum_ref_membership(&e("(= kind OrgKind/BUSINESS)"), &vocabulary()).is_ok());
    }

    #[test]
    fn membership_recurses_into_nested_forms() {
        let err =
            check_enum_ref_membership(&e("(guard #t (emit EventType/NOWHERE))"), &vocabulary())
                .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-031");
    }

    #[test]
    fn the_membership_refusal_names_the_offending_verb() {
        // F6 (#534 fix round item 6): §4.6 requires load-time errors to
        // report the offending form — the raw `VocabularyError` alone
        // names only the type/member, not which verb wrote it.
        let err =
            check_enum_ref_membership(&e("(emit EventType/NOWHERE)"), &vocabulary()).unwrap_err();
        assert!(err.to_string().contains("emit"), "{err}");
        assert!(err.to_string().contains("EventType/NOWHERE"), "{err}");
    }

    #[test]
    fn a_payload_item_labeled_like_emit_is_never_over_refused() {
        // F5(a) (#534 fix round item 5, mirrors R2's exact fix for the
        // sibling `check_type_operands_are_enum_refs` walk). `emit`'s
        // `<payload-item>` label is an unconstrained `Atom::Symbol`
        // (§2.8's `<payload-item> ::= (<symbol> <expr>)`) — nothing stops
        // content from naming one `emit`, with a VALUE that happens to be
        // a coincidentally well-kinded-but-unregistered `<enum-ref>`. The
        // unconditional recursion used to re-inspect this nested list as
        // a FRESH `emit` invocation and refuse its "operand 1" under
        // E-LOAD-031 for a value that is not `emit`'s type operand at
        // all. The regression pair — a GENUINE nested `emit` still
        // getting caught — is already pinned by
        // `membership_recurses_into_nested_forms` above (nested inside a
        // `guard`, not inside `emit`'s own payload).
        assert!(check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (emit EventType/NOWHERE))"),
            &vocabulary(),
        )
        .is_ok());
    }

    #[test]
    fn an_undeclared_kind_under_a_partial_vocabulary_is_inert_at_the_rule_producer() {
        // F1 (#534 fix round item 1): the plan's own Task-10 scenario shape
        // (docs/superpowers/plans/2026-08-11-organization-foundation-plan.md
        // Task 10, ~lines 443-480) declares `NodeType`/`EdgeType` but NOT
        // `EventType`, then its probe rule emits
        // `EventType/ORGANIZATION_SEEDED` — a member of a kind the scenario
        // never declared at all. Before the fix, `check_enum_ref`'s
        // ABSENT-kind case was indistinguishable from a DECLARED kind
        // missing the member, so this load would have wrongly refused
        // E-LOAD-031 for a kind that was never even opted into vocabulary
        // checking.
        let partial =
            ClosedVocabulary::new([(EnumKind::NodeType, vec!["SOCIAL_CLASS".to_owned()])]).unwrap();
        assert!(
            check_enum_ref_membership(&e("(emit EventType/ANYTHING)"), &partial).is_ok(),
            "EventType was never declared in this vocabulary — its checking must stay inert"
        );
    }

    // ---- G1 (#534 fix round 2, delta-verify MAJOR ×2 — one root cause):
    // `a_payload_item_labeled_like_emit_is_never_over_refused` proved the
    // over-refusal fix; these are its own over-CORRECTION. Stopping at
    // `head_is_emit` skipped the whole subtree below `emit`, not just the
    // payload LABELS — so a payload item's own VALUE (an arbitrary
    // `<expr>` per §2.8's `<payload-item> ::= (<symbol> <expr>)`) escaped
    // this walk entirely, including a typed position nested inside it. ----

    #[test]
    fn a_payload_values_own_typed_position_still_refuses_one_level_down() {
        // `(m (the NodeType/NOWHERE))` is a payload item labeled `m` whose
        // VALUE is `(the NodeType/NOWHERE)` — `the`'s own operand 1 is a
        // typed NodeType position (D74), and NOWHERE is unregistered.
        // Before the fix, `head_is_emit` returned `Ok(())` before the
        // recursion ever reached this nested form at all.
        let err = check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (m (the NodeType/NOWHERE)))"),
            &vocabulary(),
        )
        .expect_err("a payload VALUE's own typed position must still be checked");
        assert_eq!(err.spec_code(), "E-LOAD-031");
    }

    #[test]
    fn a_payload_values_own_typed_position_still_refuses_nested_deeper() {
        // The same discipline two levels down: the value is `(fold sum
        // (nodes NodeType/NOWHERE) 1)` — `nodes`'s own operand 1 is still
        // reachable by the ordinary recursive walk once inside the
        // payload VALUE (never re-entering `emit`'s own head-position-only
        // branch, since `fold`/`nodes` are neither `emit` nor a match for
        // `demanded_kind`'s own head set at the wrong operand).
        let err = check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (m (fold sum (nodes NodeType/NOWHERE) 1)))"),
            &vocabulary(),
        )
        .expect_err("a typed position nested inside a payload value must still be checked");
        assert_eq!(err.spec_code(), "E-LOAD-031");
    }

    #[test]
    fn the_r2_over_refusal_fix_stays_green_after_the_g1_repair() {
        // The CONSTRAINT G1 must not regress: a payload item's own LABEL
        // is still never mistaken for a nested verb/typed-position head —
        // `a_payload_item_labeled_like_emit_is_never_over_refused` above
        // already pins the `(emit EventType/RUPTURE (emit
        // EventType/NOWHERE))` shape; this is its sibling probe with a
        // DIFFERENT label, isolating that the fix is about position
        // (label vs. value), not about the specific label spelling.
        assert!(check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (foo EventType/NOWHERE))"),
            &vocabulary(),
        )
        .is_ok());
    }

    // ---- H1 (#534 fix round 3, residual of G1 — one root cause with
    // `structural_verbs::find_deferred_shape_verb`'s own sibling fix):
    // the G1 descent assumed items[1] is emit's type operand (`skip(2)`)
    // and that a payload item has exactly 2 elements (`pair.get(1)`).
    // Neither invariant is established when this function is driven
    // directly — a SECOND malformation (a type-operand-less nested emit,
    // or an over-arity payload item) breaks both assumptions at once. ----

    #[test]
    fn a_nested_emit_missing_its_own_type_operand_inside_a_payload_value_still_refuses() {
        // The `check_enum_ref_membership` twin of `check_type_operands_
        // are_enum_refs`'s own escape-1 probe above: `(m (emit (the
        // NodeType/NOWHERE)))` is a payload item whose value is a NESTED
        // `emit` missing its own type operand — under the OLD `skip(2)`,
        // this nested emit's `items[1]` (`(the NodeType/NOWHERE)`) was
        // silently treated as "the confirmed type operand" and skipped,
        // so the `NodeType/NOWHERE` typed position inside it never got
        // checked. The fallback must fully recurse instead.
        let err = check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (m (emit (the NodeType/NOWHERE))))"),
            &vocabulary(),
        )
        .expect_err("a typed position behind a malformed nested emit must still be checked");
        assert_eq!(err.spec_code(), "E-LOAD-031");
    }

    #[test]
    fn an_over_arity_payload_item_still_checks_every_value_not_just_element_one() {
        // `(m 1 (the NodeType/NOWHERE))` is an over-arity payload item —
        // THREE elements, not the well-formed `(<symbol> <expr>)` two —
        // so `pair.get(1)` alone (the literal `1`) never reached
        // `pair[2]`, the `(the NodeType/NOWHERE)` typed position. The fix
        // descends `pair[1..]`, every element after the label.
        let err = check_enum_ref_membership(
            &e("(emit EventType/RUPTURE (m 1 (the NodeType/NOWHERE)))"),
            &vocabulary(),
        )
        .expect_err("every value after an over-arity payload item's label must still be checked");
        assert_eq!(err.spec_code(), "E-LOAD-031");
    }
}
