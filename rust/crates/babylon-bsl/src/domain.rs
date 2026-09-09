//! The rule domain (`bsl-language.rst` §2.3, R9 chapter C4): **what a rule
//! fires over, and how many times**.
//!
//! §4.2 said a rule evaluates against "the subject node" and §5.6 let the
//! reader *infer* the subject's type from a binding's qname prefix; the
//! document never stated the inference rule, and said nothing at all about a
//! rule whose only bindings are `:const`, `:metric` and `:tick`. D43 states
//! both, and this module computes them at load:
//!
//! - `<domain>` is **optional**, with an inference as its default. Let `U`
//!   be the set of node types owning every `:field` binding referenced at
//!   least once **outside every query body**, plus every `<qname>` of an
//!   `update-node` and of a `field-of` whose element operand is `self`.
//!   `|U| = 1` gives the domain; `|U| = 0` and `|U| > 1` are both
//!   `E-LOAD-004`, and both are repaired by writing `<domain>` explicitly.
//! - **The surprise the gap analysis names is removed by construction**: a
//!   binding referenced only inside a fold body never enters `U`, so adding
//!   one cannot change how many times a rule fires.
//! - `(domain NodeType/…)` replaces the inference outright; a self-scoped
//!   reference owning off a different node type is `E-TYPE-010`.
//! - `(domain :graph)` fires the rule **exactly once per tick**. `self` is
//!   not bound: any reference to `self`, and any `:field` binding referenced
//!   outside a query body, is `E-TYPE-015`.
//!
//! `cost(domain) = 0` needs no code: `bound(rule)` sums the `<when>`
//! condition and the effect items, and a `domain` child is neither — a row
//! that charges nothing has no charging path.

use crate::bindings::{BindSource, BindingDecl};
use crate::reader::{Atom, SExpr};
use crate::scope::referenced_at_rule_scope;
use crate::vocabulary::{render_member, ClosedVocabulary, EnumKind};
use std::collections::BTreeSet;

/// What a rule fires over (§2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleDomain {
    /// One firing per node of this node type, in ascending node-id byte
    /// order (§4.2, D44). Carried as the type's §2.9 **segment** rendering.
    Node(String),
    /// Exactly one firing per tick, at the rule's anchor position. `self`
    /// is not bound.
    Graph,
}

/// A domain-resolution rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// `E-LOAD-004` — the inference is undeterminable (`|U| = 0`) or
    /// ambiguous (`|U| > 1`); both are repaired by writing `<domain>`.
    Undeterminable {
        /// The self-scoped node types found, ascending.
        candidates: Vec<String>,
    },
    /// `E-TYPE-010` — an explicit `<domain>` and a self-scoped reference
    /// disagree. This is the existing code for a foreign node type read
    /// outside a fold body over that type, which is exactly what such a
    /// reference is.
    DomainDisagreement {
        /// The declared domain's segment.
        declared: String,
        /// The self-scoped reference's owning segment.
        found: String,
    },
    /// `E-TYPE-015` — `self`, or a `:field` binding read outside a query
    /// body, in a `(domain :graph)` rule.
    SelfInGraphDomain {
        /// What was referenced.
        reference: String,
    },
    /// A `domain` form off the §2.3 grammar.
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
}

impl DomainError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::Undeterminable { .. } => Some("E-LOAD-004"),
            Self::DomainDisagreement { .. } => Some("E-TYPE-010"),
            Self::SelfInGraphDomain { .. } => Some("E-TYPE-015"),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undeterminable { candidates } => write!(
                f,
                "E-LOAD-004: the rule domain is undeterminable — {} self-scoped \
                 node type(s) {candidates:?}; write <domain> explicitly (§2.3)",
                candidates.len()
            ),
            Self::DomainDisagreement { declared, found } => write!(
                f,
                "E-TYPE-010: (domain …{declared}…) but a self-scoped reference \
                 owns off '{found}' (§2.3)"
            ),
            Self::SelfInGraphDomain { reference } => write!(
                f,
                "E-TYPE-015: a (domain :graph) rule fires once per tick and \
                 binds no `self`; '{reference}' has nothing to resolve against \
                 (§2.3)"
            ),
            Self::Malformed { message } => write!(f, "malformed domain form: {message}"),
        }
    }
}

impl std::error::Error for DomainError {}

fn malformed(message: impl Into<String>) -> DomainError {
    DomainError::Malformed {
        message: message.into(),
    }
}

/// The `<domain>` child of a rule, if one is written.
fn domain_form(rule_items: &[SExpr]) -> Option<&[SExpr]> {
    rule_items.iter().find_map(|child| match child {
        SExpr::List(inner)
            if matches!(inner.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "domain") =>
        {
            Some(inner.as_slice())
        }
        _ => None,
    })
}

/// The rule's `<when>` and `<effects>` bodies, plus every `:expr` binding
/// operand — every expression position a reference can occupy.
fn reference_sites(rule_items: &[SExpr]) -> Vec<&SExpr> {
    let mut sites = Vec::new();
    for child in rule_items {
        let SExpr::List(inner) = child else { continue };
        match inner.first() {
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "when" || h == "effects" => {
                sites.extend(&inner[1..]);
            }
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "bindings" => {
                for row in &inner[1..] {
                    let SExpr::List(cells) = row else { continue };
                    for window in cells.windows(2) {
                        if let [SExpr::Atom(Atom::Keyword(kw)), operand] = window {
                            if kw == "expr" {
                                sites.push(operand);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    sites
}

/// Collect the owning segment of every `<qname>` whose verb/accessor names
/// `self` as its element operand: `(update-node self <qname> …)` and
/// `(field-of self <qname>)` (§2.3's inference, clause 2).
fn self_scoped_qnames(expr: &SExpr, out: &mut Vec<String>) {
    if let SExpr::List(items) = expr {
        if let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Symbol(elem)), SExpr::Atom(Atom::QName(qname)), ..] =
            items.as_slice()
        {
            if elem == "self" && (head == "update-node" || head == "field-of") {
                out.push(qname.split('/').next().unwrap_or(qname).to_owned());
            }
        }
        for child in items {
            self_scoped_qnames(child, out);
        }
    }
}

/// Whether `self` is referenced anywhere in an expression tree.
fn mentions_self(expr: &SExpr) -> bool {
    match expr {
        SExpr::Atom(Atom::Symbol(s)) => s == "self",
        SExpr::Atom(_) => false,
        SExpr::List(items) => items.iter().any(mentions_self),
    }
}

/// Resolve a rule's domain: the explicit `<domain>` if written, otherwise
/// §2.3's inference.
///
/// # Errors
///
/// [`DomainError::Undeterminable`] (`E-LOAD-004`),
/// [`DomainError::DomainDisagreement`] (`E-TYPE-010`),
/// [`DomainError::SelfInGraphDomain`] (`E-TYPE-015`),
/// [`DomainError::Malformed`] off the §2.3 grammar.
pub fn resolve_domain(
    rule: &SExpr,
    decls: &[BindingDecl],
    vocabulary: &ClosedVocabulary,
) -> Result<RuleDomain, DomainError> {
    let SExpr::List(items) = rule else {
        return Err(malformed("a rule must be a form"));
    };
    let sites = reference_sites(items);
    let self_scoped = self_scoped_segments(decls, &sites, vocabulary);

    match domain_form(items) {
        Some([_, SExpr::Atom(Atom::Keyword(kw))]) if kw == "graph" => {
            check_graph_domain(&sites, &self_scoped)?;
            Ok(RuleDomain::Graph)
        }
        Some([_, SExpr::Atom(Atom::EnumRef { enum_type, member })]) => {
            // The KIND check is `grammar::check_enum_ref_kinds`' (D74,
            // E-TYPE-011); by here the operand is a NodeType or the rule
            // never reached this stage.
            if EnumKind::from_type_name(enum_type) != Some(EnumKind::NodeType) {
                return Err(malformed(format!(
                    "(domain {enum_type}/{member}) — the kind check is \
                     E-TYPE-011's and should have rejected this already"
                )));
            }
            let declared = render_member(member);
            for found in &self_scoped {
                if *found != declared {
                    return Err(DomainError::DomainDisagreement {
                        declared,
                        found: found.clone(),
                    });
                }
            }
            Ok(RuleDomain::Node(declared))
        }
        Some(other) => Err(malformed(format!(
            "(domain <enum-ref> | :graph) — unrecognized shape {other:?}"
        ))),
        None => {
            let candidates: Vec<String> = self_scoped.into_iter().collect();
            match candidates.len() {
                1 => Ok(RuleDomain::Node(
                    candidates.into_iter().next().unwrap_or_default(),
                )),
                _ => Err(DomainError::Undeterminable { candidates }),
            }
        }
    }
}

/// `U`: the node-type segments the rule is self-scoped against.
fn self_scoped_segments(
    decls: &[BindingDecl],
    sites: &[&SExpr],
    vocabulary: &ClosedVocabulary,
) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for decl in decls {
        let BindSource::Field(qname) = &decl.source else {
            continue;
        };
        let segment = qname.split('/').next().unwrap_or(qname);
        if !matches!(vocabulary.owner_of(segment), Ok((EnumKind::NodeType, _))) {
            continue;
        }
        if sites
            .iter()
            .any(|site| referenced_at_rule_scope(site, &decl.name))
        {
            set.insert(segment.to_owned());
        }
    }
    let mut qnames = Vec::new();
    for site in sites {
        self_scoped_qnames(site, &mut qnames);
    }
    for segment in qnames {
        if matches!(vocabulary.owner_of(&segment), Ok((EnumKind::NodeType, _))) {
            set.insert(segment);
        }
    }
    set
}

/// `(domain :graph)`: `self` is not bound, and neither is a `:field`
/// binding read outside a query body.
fn check_graph_domain(sites: &[&SExpr], self_scoped: &BTreeSet<String>) -> Result<(), DomainError> {
    if let Some(segment) = self_scoped.iter().next() {
        return Err(DomainError::SelfInGraphDomain {
            reference: format!("a :field binding or self-scoped qname owning off '{segment}'"),
        });
    }
    if sites.iter().any(|site| mentions_self(site)) {
        return Err(DomainError::SelfInGraphDomain {
            reference: "self".to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{resolve_domain, RuleDomain};
    use crate::bindings::parse_bindings;
    use crate::reader::read;
    use crate::vocabulary::{ClosedVocabulary, EnumKind};

    fn vocabulary() -> ClosedVocabulary {
        ClosedVocabulary::new([
            (
                EnumKind::NodeType,
                vec![
                    "SOCIAL_CLASS".to_owned(),
                    "ORGANIZATION".to_owned(),
                    "POLITY".to_owned(),
                ],
            ),
            (EnumKind::EdgeType, vec!["SOLIDARITY".to_owned()]),
        ])
        .unwrap()
    }

    fn domain(source: &str) -> Result<RuleDomain, super::DomainError> {
        let (rule, _) = read(source).expect("test source must parse");
        let decls = parse_bindings(&rule).expect("bindings must parse");
        resolve_domain(&rule, &decls, &vocabulary())
    }

    const PREAMBLE: &str =
        ":role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 65536";

    #[test]
    fn the_worked_example_infers_its_node_domain() {
        // §5.6's rule, unchanged: one `:field` binding read at rule scope
        // and an `update-node self …`, both owning off social-class.
        let d = domain(&format!(
            "(rule demo/hunger {PREAMBLE} \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 1000.5$)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        ))
        .unwrap();
        assert_eq!(d, RuleDomain::Node("social-class".to_owned()));
    }

    #[test]
    fn no_self_scoped_reference_and_no_domain_is_e_load_004() {
        let err = domain(&format!(
            "(rule demo/graphy {PREAMBLE} \
             (bindings (binding now :tick)) \
             (when (< now 5)) \
             (effects (emit EventType/RUPTURE (t now))))"
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-004"));
    }

    #[test]
    fn two_self_scoped_node_types_is_e_load_004() {
        let err = domain(&format!(
            "(rule demo/two {PREAMBLE} \
             (bindings (binding wealth :field social-class/wealth) \
                       (binding claim :field organization/claim-strength)) \
             (when (< wealth claim)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-004"));
    }

    #[test]
    fn an_explicit_domain_overrides_a_would_be_ambiguity() {
        // The same rule with `(domain …)` written: the inference is
        // replaced outright, and the second type is now a foreign read the
        // author must scope (E-TYPE-010 below, not an ambiguity).
        let err = domain(&format!(
            "(rule demo/two {PREAMBLE} \
             (domain NodeType/SOCIAL_CLASS) \
             (bindings (binding wealth :field social-class/wealth) \
                       (binding claim :field organization/claim-strength)) \
             (when (< wealth claim)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-010"));
    }

    #[test]
    fn an_explicit_domain_agreeing_with_its_references_resolves() {
        let d = domain(&format!(
            "(rule demo/explicit {PREAMBLE} \
             (domain NodeType/SOCIAL_CLASS) \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        ))
        .unwrap();
        assert_eq!(d, RuleDomain::Node("social-class".to_owned()));
    }

    #[test]
    fn a_binding_read_only_inside_a_fold_body_never_enters_the_inference() {
        // D43's stated property: adding a fold-scoped binding cannot change
        // how many times a rule fires.
        let d = domain(&format!(
            "(rule demo/foldy {PREAMBLE} \
             (domain :graph) \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) claim) 5)) \
             (effects (emit EventType/RUPTURE (x 1))))"
        ))
        .unwrap();
        assert_eq!(d, RuleDomain::Graph);
    }

    #[test]
    fn a_graph_domain_rule_referencing_self_is_e_type_015() {
        let err = domain(&format!(
            "(rule demo/graph-self {PREAMBLE} \
             (domain :graph) \
             (bindings) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-015"));
    }

    #[test]
    fn a_graph_domain_rule_reading_a_field_binding_at_rule_scope_is_e_type_015() {
        let err = domain(&format!(
            "(rule demo/graph-field {PREAMBLE} \
             (domain :graph) \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 5)) \
             (effects (emit EventType/RUPTURE (x 1))))"
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-TYPE-015"));
    }

    #[test]
    fn a_graph_domain_rule_over_carriers_and_queries_resolves() {
        let d = domain(&format!(
            "(rule demo/graph-ok {PREAMBLE} \
             (domain :graph) \
             (bindings (binding now :tick)) \
             (when (< now 520)) \
             (effects (update-node (the NodeType/POLITY) \
                                   polity/imperial-rent-pool (set 0$))))"
        ))
        .unwrap();
        assert_eq!(d, RuleDomain::Graph);
    }
}
