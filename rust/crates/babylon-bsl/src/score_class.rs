//! A **total** static classifier over §2.7's `<expr>` grammar, into the
//! coarse classes the R9 chapters' two new type rules need:
//!
//! - D46 / `E-TYPE-016`: a `select-max`/`select-min` score must have a
//!   comparable scalar static type — `Int`, `Currency`, `Probability`,
//!   `Intensity`, `Coefficient` or `Real`. `Bool`, `Enum<T>`, `Str`,
//!   references and sets are rejected.
//! - D67 / `E-TYPE-017`: references compare by **identity**, with `=` and
//!   `!=` only, and only against a reference of the same kind.
//!
//! It is deliberately *total* rather than partial. §3.1 gives every
//! expression exactly one static type, and a classifier with an
//! "undecidable → accept" arm would be a silent pass-through — the shape
//! III.11 forbids and the shape `rule_pipeline`'s compound-fold rejection
//! already refuses. Every §2.7 production therefore has an arm here, and a
//! form outside the grammar classifies as [`ScoreClass::Unknown`], which
//! callers treat as loud.
//!
//! It is **coarser** than §3.1's type system on purpose: nothing in the two
//! rules above distinguishes `Currency` from `Real`, so this module does not
//! either. The full bottom-up typechecker is Phase-2 work; this classifier
//! answers exactly the two questions the chapters ask and no more.

use crate::bindings::{BindSource, BindingDecl};
use crate::reader::{Atom, SExpr};
use crate::types::{BslType, FieldDecl};
use std::collections::HashMap;

/// The coarse static class of an expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreClass {
    /// A comparable scalar: `Int`, `Currency`, `Probability`, `Intensity`,
    /// `Coefficient` or `Real`.
    Scalar,
    /// Exact finite-kernel Mass: deliberately not comparable.
    Mass,
    /// `Bool` — the result type of every `<cond>`.
    Bool,
    /// A member of a closed enum.
    Enum,
    /// `Str` — admitted at `:material-basis` and vector ids only.
    Str,
    /// A `NodeRef`, `EdgeRef` or `HyperedgeRef`.
    NodeReference,
    /// An `EdgeRef` specifically — distinguished because D67's identity
    /// comparison is legal only **within** one reference kind.
    EdgeReference,
    /// A `HyperedgeRef`.
    HyperedgeReference,
    /// A `NodeSet` / `EdgeSet` / `HyperedgeSet`: only `fold`, `exists`,
    /// `forall` and the selections consume one.
    Set,
    /// A form outside §2.7's grammar. Callers treat this as loud.
    Unknown,
}

impl ScoreClass {
    /// Whether this class is one of the reference kinds (§3.1).
    #[must_use]
    pub fn is_reference(self) -> bool {
        matches!(
            self,
            Self::NodeReference | Self::EdgeReference | Self::HyperedgeReference
        )
    }

    /// Whether §2.7's selection accepts this as a score (D46).
    #[must_use]
    pub fn is_comparable_scalar(self) -> bool {
        self == Self::Scalar
    }
}

/// What the classifier resolves names against.
pub struct ClassEnv<'a> {
    /// The rule's declared bindings, for a `:field`/`:const`/… reference.
    pub bindings: &'a [BindingDecl],
    /// Declared field types, for `field-of` and for a `:field` binding.
    pub fields: &'a HashMap<String, FieldDecl>,
    /// `:as` names in scope, with the class of their form's element.
    pub element_names: &'a HashMap<String, ScoreClass>,
}

fn of_type(ty: &BslType) -> ScoreClass {
    match ty {
        BslType::Mass => ScoreClass::Mass,
        BslType::Bool => ScoreClass::Bool,
        BslType::Enum(_) => ScoreClass::Enum,
        BslType::NodeSet(_) | BslType::EdgeSet(_) => ScoreClass::Set,
        BslType::Int
        | BslType::Real
        | BslType::Currency
        | BslType::Probability
        | BslType::Intensity
        | BslType::Coefficient => ScoreClass::Scalar,
    }
}

/// The class of `<qname>`'s declared field, or [`ScoreClass::Unknown`] when
/// no declaration is in scope (an undeclared field is `E-LOAD-010`'s
/// rejection, not this classifier's).
fn field_class(env: &ClassEnv<'_>, qname: &str) -> ScoreClass {
    env.fields
        .get(qname)
        .map_or(ScoreClass::Unknown, |decl| of_type(&decl.ty))
}

/// Classify one expression. Total over §2.7.
#[must_use]
pub fn classify(expr: &SExpr, env: &ClassEnv<'_>) -> ScoreClass {
    match expr {
        SExpr::Atom(atom) => classify_atom(atom, env),
        SExpr::List(items) => classify_form(items, env),
    }
}

fn classify_atom(atom: &Atom, env: &ClassEnv<'_>) -> ScoreClass {
    match atom {
        Atom::Mass(_) => ScoreClass::Mass,
        Atom::Int(_) | Atom::Currency(_) | Atom::Scaled(_) => ScoreClass::Scalar,
        Atom::Bool(_) => ScoreClass::Bool,
        Atom::EnumRef { .. } => ScoreClass::Enum,
        Atom::Str(_) => ScoreClass::Str,
        Atom::Symbol(name) => classify_symbol(name, env),
        // A qname is a static field path and a keyword an option marker;
        // neither is an expression (§1.6 rejects both by position). A bare
        // `enum-type` name (§2.13) is a declaration-only construct — it
        // never appears in `<when>`/`<effects>` expression position, only
        // in `defenum`/`defvocabulary`'s own operand and `deffield`'s
        // `:enum-type` keyword, both consumed by their own parsers before
        // this classifier ever runs — so it is unclassified here too.
        Atom::QName(_) | Atom::Keyword(_) | Atom::Operator(_) | Atom::BareUpperIdent(_) => {
            ScoreClass::Unknown
        }
    }
}

fn classify_symbol(name: &str, env: &ClassEnv<'_>) -> ScoreClass {
    // `self` is a NodeRef; `it` and the `:as` names take their form's
    // element class (§2.6's result table).
    if name == "self" {
        return ScoreClass::NodeReference;
    }
    if let Some(class) = env.element_names.get(name) {
        return *class;
    }
    let Some(decl) = env.bindings.iter().find(|d| d.name == name) else {
        return ScoreClass::Unknown;
    };
    match &decl.source {
        BindSource::Field(qname) => field_class(env, qname),
        // A coefficient, a registered metric and every calendar read are
        // scalars (§2.5: `:tick`/`:year`/`:tick-of-year`/`:tick-in-cycle`
        // all bind `Int`).
        BindSource::Const(_)
        | BindSource::Metric(_)
        | BindSource::Tick
        | BindSource::Year
        | BindSource::TickOfYear
        | BindSource::TickInCycle(_) => ScoreClass::Scalar,
        BindSource::Expr(expr) => classify(expr, env),
    }
}

fn classify_form(items: &[SExpr], env: &ClassEnv<'_>) -> ScoreClass {
    if let Some(SExpr::Atom(Atom::Operator(op))) = items.first() {
        return match op.as_str() {
            "+" | "-" | "*" | "/" => ScoreClass::Scalar,
            _ => ScoreClass::Bool, // the six comparisons
        };
    }
    let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() else {
        return ScoreClass::Unknown;
    };
    match head.as_str() {
        "and" | "or" | "not" | "exists" | "forall" => ScoreClass::Bool,
        // §2.7: both branches of `if` have the same static type
        // (`E-TYPE-020`), so the `then` branch decides.
        "if" => items
            .get(2)
            .map_or(ScoreClass::Unknown, |b| classify(b, env)),
        "fold" => classify_fold(items, env),
        "nodes" | "edges" | "neighbors" | "hyperedges" | "members-of" | "hyperedges-of" => {
            ScoreClass::Set
        }
        "select-max" | "select-min" => items
            .get(1)
            .map_or(ScoreClass::Unknown, selection_result_class),
        "field-of" => match items.get(2) {
            Some(SExpr::Atom(Atom::QName(qname))) => field_class(env, qname),
            _ => ScoreClass::Unknown,
        },
        "edge-between" => ScoreClass::EdgeReference,
        "the" => ScoreClass::NodeReference,
        // `metric-of`'s declared `:type` is a §3.1 scalar (§2.11's grammar
        // admits no other), so the read is a scalar without consulting the
        // registry — and so is an intrinsic call's `:returns`, which is what
        // §2.7 makes any other symbol head in expression position. The two
        // share this arm because they share the reason, not by accident.
        _ => ScoreClass::Scalar,
    }
}

fn classify_fold(items: &[SExpr], env: &ClassEnv<'_>) -> ScoreClass {
    let Some(SExpr::Atom(Atom::Symbol(op))) = items.get(1) else {
        return ScoreClass::Unknown;
    };
    // CT4P A3 (issue #525): `op` converts to `FoldOp` ONCE, here; the match
    // below is EXHAUSTIVE over it, no wildcard. `None` (op outside the
    // closed set) keeps the original `_ => ScoreClass::Unknown` behaviour.
    let Some(fold_op) = crate::grammar::FoldOp::parse(op.as_str()) else {
        return ScoreClass::Unknown;
    };
    match fold_op {
        crate::grammar::FoldOp::Count => ScoreClass::Scalar, // Int
        crate::grammar::FoldOp::Sum
        | crate::grammar::FoldOp::Mean
        | crate::grammar::FoldOp::Min
        | crate::grammar::FoldOp::Max => {
            // The body follows the query, optionally after `:as <symbol>`.
            let body = match items.get(3) {
                Some(SExpr::Atom(Atom::Keyword(kw))) if kw == "as" => items.get(5),
                other => other,
            };
            body.map_or(ScoreClass::Unknown, |b| classify(b, env))
        }
    }
}

/// A selection returns the query's **element**, so its class is the query's
/// element class (§2.7): `NodeRef` for `nodes`/`neighbors`/`members-of`,
/// `EdgeRef` for `edges`, `HyperedgeRef` for `hyperedges`/`hyperedges-of`.
#[must_use]
pub fn selection_result_class(query: &SExpr) -> ScoreClass {
    let SExpr::List(items) = query else {
        return ScoreClass::Unknown;
    };
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(head))) => match head.as_str() {
            "nodes" | "neighbors" | "members-of" => ScoreClass::NodeReference,
            "edges" => ScoreClass::EdgeReference,
            "hyperedges" | "hyperedges-of" => ScoreClass::HyperedgeReference,
            _ => ScoreClass::Unknown,
        },
        _ => ScoreClass::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{classify, ClassEnv, ScoreClass};
    use crate::bindings::parse_bindings;
    use crate::reader::read;
    use crate::types::{BslType, FieldDecl, FieldKind};
    use std::collections::HashMap;

    fn fields() -> HashMap<String, FieldDecl> {
        HashMap::from([
            (
                "social-class/wealth".to_owned(),
                FieldDecl {
                    ty: BslType::Currency,
                    kind: FieldKind::Extensive,
                },
            ),
            (
                "social-class/organised".to_owned(),
                FieldDecl {
                    ty: BslType::Bool,
                    kind: FieldKind::Extensive,
                },
            ),
        ])
    }

    fn class(source: &str) -> ScoreClass {
        let (expr, _) = read(source).expect("test source must parse");
        let decls = Vec::new();
        let names = HashMap::from([("it".to_owned(), ScoreClass::NodeReference)]);
        let fields = fields();
        classify(
            &expr,
            &ClassEnv {
                bindings: &decls,
                fields: &fields,
                element_names: &names,
            },
        )
    }

    #[test]
    fn literals_classify_by_their_lexical_kind() {
        assert_eq!(class("5"), ScoreClass::Scalar);
        assert_eq!(class("1000.5$"), ScoreClass::Scalar);
        assert_eq!(class("0.5c"), ScoreClass::Scalar);
        assert_eq!(class("#t"), ScoreClass::Bool);
        assert_eq!(class("NodeType/SOCIAL_CLASS"), ScoreClass::Enum);
        assert_eq!(class("\"a basis\""), ScoreClass::Str);
    }

    #[test]
    fn references_classify_by_kind() {
        assert_eq!(class("self"), ScoreClass::NodeReference);
        assert_eq!(class("it"), ScoreClass::NodeReference);
        assert_eq!(class("(the NodeType/POLITY)"), ScoreClass::NodeReference);
        assert_eq!(
            class("(edge-between EdgeType/SOLIDARITY a b)"),
            ScoreClass::EdgeReference
        );
        assert_eq!(
            class("(select-max (hyperedges HyperedgeType/COMMUNITY) 1)"),
            ScoreClass::HyperedgeReference
        );
    }

    #[test]
    fn arithmetic_is_scalar_and_every_comparison_is_bool() {
        assert_eq!(class("(+ 1 2)"), ScoreClass::Scalar);
        assert_eq!(class("(< 1 2)"), ScoreClass::Bool);
        assert_eq!(class("(and #t #f)"), ScoreClass::Bool);
        assert_eq!(
            class("(exists (nodes NodeType/SOCIAL_CLASS))"),
            ScoreClass::Bool
        );
    }

    #[test]
    fn a_query_is_a_set_and_only_the_iterating_forms_consume_one() {
        assert_eq!(class("(nodes NodeType/SOCIAL_CLASS)"), ScoreClass::Set);
    }

    /// `floor` (ADR188 Row 2) is an ordinary intrinsic call to this
    /// classifier — no name-specific arm exists or is needed, since every
    /// symbol-headed form falls to the generic "intrinsic call's `:returns`
    /// is a scalar" arm (`classify_form`'s comment explains why: `int`, like
    /// every declarable `<type-name>`, classifies as `Scalar`). This test
    /// locks that reading in for the new name specifically, so a future
    /// narrowing of the fallback arm cannot silently stop covering it.
    ///
    /// The argument is `(+ 0.5c 0.5c)`, not a bare `Int` literal: `floor`'s
    /// real signature takes a `Real`-lane argument (the result of binary64
    /// arithmetic, §3.3), and `intrinsic_host::eval_floor` refuses a bare
    /// `Int` as a malformed call rather than promoting it — an example
    /// calling `(floor 3)` would misstate that. `classify` itself does not
    /// care (it is a coarse structural classifier, not a runtime
    /// typechecker), but the example should not imply semantics the
    /// evaluator does not have.
    #[test]
    fn a_floor_call_classifies_as_scalar_through_the_generic_intrinsic_arm() {
        assert_eq!(class("(floor (+ 0.5c 0.5c))"), ScoreClass::Scalar);
    }

    #[test]
    fn field_of_carries_the_declarations_type() {
        assert_eq!(
            class("(field-of it social-class/wealth)"),
            ScoreClass::Scalar
        );
        assert_eq!(
            class("(field-of it social-class/organised)"),
            ScoreClass::Bool
        );
    }

    #[test]
    fn a_fold_carries_its_body_class_and_count_is_always_scalar() {
        assert_eq!(
            class("(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/wealth))"),
            ScoreClass::Scalar
        );
        assert_eq!(
            class("(fold max (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/organised))"),
            ScoreClass::Bool
        );
        assert_eq!(
            class("(fold count (nodes NodeType/SOCIAL_CLASS) it)"),
            ScoreClass::Scalar
        );
    }

    #[test]
    fn a_binding_classifies_through_its_declared_source() {
        let (rule, _) = read(
            "(rule demo/c :role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 8 \
             (bindings (binding wealth :field social-class/wealth) \
                       (binding flag :field social-class/organised) \
                       (binding now :tick)) \
             (effects (update-node self social-class/wealth (add 1$))))",
        )
        .unwrap();
        let decls = parse_bindings(&rule).unwrap();
        let fields = fields();
        let names = HashMap::new();
        let env = ClassEnv {
            bindings: &decls,
            fields: &fields,
            element_names: &names,
        };
        let of = |src: &str| classify(&read(src).unwrap().0, &env);
        assert_eq!(of("wealth"), ScoreClass::Scalar);
        assert_eq!(of("flag"), ScoreClass::Bool);
        assert_eq!(of("now"), ScoreClass::Scalar);
    }

    #[test]
    fn an_if_carries_its_branches_shared_type() {
        assert_eq!(class("(if (< 1 2) 3 4)"), ScoreClass::Scalar);
        assert_eq!(class("(if (< 1 2) #t #f)"), ScoreClass::Bool);
    }
}
