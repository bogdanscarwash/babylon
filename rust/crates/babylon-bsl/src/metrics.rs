//! Metric registration (`bsl-language.rst` §2.11, R9 chapter C9).
//!
//! §2.5 said `:metric` "reads a registered graph-level metric" and stopped
//! there. It never said who may register one, what determinism obligations
//! a registration carries, or whether a metric may be **indexed by
//! element** — and the last question is the one that blocks work: every
//! topological score the OODA seam needs is per-node, and a graph-scope
//! scalar cannot carry any of them to content.
//!
//! ```text
//! <metric-decl> ::= "(" "metric" <symbol>
//!                       ":type" <type-name>
//!                       ":kind" ( "intensive" | "extensive" )
//!                       <domain>
//!                       ":provider" <symbol>
//!                   ")"
//! ```
//!
//! `<domain>` is §2.3's production reused unchanged: `(domain :graph)`
//! declares a graph-scope metric read by a `:metric` **binding**, and
//! `(domain NodeType/…)` an element-indexed one read by the `metric-of`
//! **accessor** (D56 — not a `:metric-of` bind-src, which would have been
//! the only bind-src needing two operands).
//!
//! D55: a `metric` form **declares**, it does not define — exactly as
//! `intrinsic` does, and for D9's reason: the typechecker and the fuel
//! checker must be computable from content alone for III.12(a) to hold.
//! Kernel disagreement is `E-LOAD-025`; an unregistered name stays
//! `E-LOAD-011`, never `0.0`.
//!
//! **What enters which hash** (§2.11): a metric's *name, type, kind and
//! domain* are content and hash into their own digest. A metric's *value*
//! is runtime and appears in **no** content hash — it reaches the tick hash
//! only through the fields rules write from it. An implementation that
//! hashed metric values directly would be hashing the provider's schedule
//! rather than the game's state, so nothing in this module touches CAS.
//!
//! **Fuel** (D57): the provider's computation is *not* metered against the
//! reading rule. The read costs `1 + cost(operand)` like any other accessor
//! ([`crate::bound_checker`]'s `ACCESSORS` row); the kernel's own budget
//! for provider work lives in the determinism contract.

use crate::declarations::{parse_type_name, DeclError};
use crate::reader::{Atom, SExpr};
use crate::types::{BslType, FieldDecl, FieldKind};
use crate::vocabulary::{render_member, EnumKind};
use std::collections::HashMap;

/// What a metric is indexed by (§2.11, reusing §2.3's `<domain>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricDomain {
    /// `(domain :graph)` — a graph-scope scalar, read by a `:metric`
    /// binding.
    Graph,
    /// `(domain NodeType/…)` — element-indexed, read by `metric-of`.
    /// Carried as the type's §2.9 segment rendering.
    Element(String),
}

/// One registered metric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDecl {
    /// The metric's name (a `symbol`).
    pub name: String,
    /// Its declared scalar type.
    pub ty: BslType,
    /// Its declared intensivity kind — which a `:metric` binding and a
    /// `metric-of` accessor both carry (D55, superseding D12's metric
    /// clause).
    pub kind: FieldKind,
    /// What it is indexed by.
    pub domain: MetricDomain,
    /// The kernel service that provides it.
    pub provider: String,
}

/// A metric-registration rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricError {
    /// `E-LOAD-011` — a metric name outside the registry; never `0.0`.
    Unregistered {
        /// The unregistered name.
        name: String,
    },
    /// `E-LOAD-012` — a metric read through the wrong form for its declared
    /// domain: a graph metric via `metric-of`, or an element-indexed metric
    /// via a `:metric` binding. Both are static.
    WrongReadingForm {
        /// The metric.
        name: String,
        /// What is wrong.
        detail: &'static str,
    },
    /// `E-LOAD-025` — a declaration disagreeing with the kernel's
    /// registration (D55).
    KernelDisagreement {
        /// The metric.
        name: String,
        /// What disagrees.
        detail: String,
    },
    /// A duplicate metric declaration across the content set.
    Duplicate {
        /// The metric declared twice.
        name: String,
    },
    /// A form off the §2.11 grammar.
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
}

impl MetricError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::Unregistered { .. } => Some("E-LOAD-011"),
            Self::WrongReadingForm { .. } => Some("E-LOAD-012"),
            Self::KernelDisagreement { .. } => Some("E-LOAD-025"),
            Self::Duplicate { .. } => Some("E-LOAD-001"),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unregistered { name } => write!(
                f,
                "E-LOAD-011: unregistered metric {name} — never 0.0 (§2.11)"
            ),
            Self::WrongReadingForm { name, detail } => {
                write!(f, "E-LOAD-012: metric {name}: {detail} (§2.11)")
            }
            Self::KernelDisagreement { name, detail } => write!(
                f,
                "E-LOAD-025: metric {name} disagrees with the kernel's \
                 registration: {detail}"
            ),
            Self::Duplicate { name } => {
                write!(f, "E-LOAD-001: duplicate metric declaration: {name}")
            }
            Self::Malformed { message } => write!(f, "malformed metric declaration: {message}"),
        }
    }
}

impl std::error::Error for MetricError {}

fn malformed(message: impl Into<String>) -> MetricError {
    MetricError::Malformed {
        message: message.into(),
    }
}

/// Every registered metric of a content set.
#[derive(Debug, Clone, Default)]
pub struct MetricRegistry {
    metrics: HashMap<String, MetricDecl>,
}

impl MetricRegistry {
    /// Read one `(metric <symbol> …)` form into the registry.
    ///
    /// # Errors
    ///
    /// [`MetricError::Duplicate`], [`MetricError::Malformed`].
    pub fn declare(&mut self, form: &SExpr) -> Result<(), MetricError> {
        let decl = parse_metric(form)?;
        if self.metrics.contains_key(&decl.name) {
            return Err(MetricError::Duplicate { name: decl.name });
        }
        self.metrics.insert(decl.name.clone(), decl);
        Ok(())
    }

    /// One registered metric, by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&MetricDecl> {
        self.metrics.get(name)
    }

    /// Check the declarations against the kernel's registrations
    /// (`E-LOAD-025`, D55: the kernel is checked against content).
    ///
    /// # Errors
    ///
    /// [`MetricError::KernelDisagreement`].
    pub fn check_against_kernel(
        &self,
        kernel: &HashMap<String, MetricDecl>,
    ) -> Result<(), MetricError> {
        let mut names: Vec<&String> = self.metrics.keys().collect();
        names.sort(); // deterministic first-failure reporting
        for name in names {
            let declared = &self.metrics[name];
            let Some(registered) = kernel.get(name) else {
                continue; // an unregistered name is E-LOAD-011's rejection
            };
            if registered != declared {
                return Err(MetricError::KernelDisagreement {
                    name: name.clone(),
                    detail: format!(
                        "content declares {declared:?}, kernel registers {registered:?}"
                    ),
                });
            }
        }
        Ok(())
    }

    /// Apply §2.11's reading-form rule to one rule form: a `:metric`
    /// binding may name only a `(domain :graph)` metric, and `metric-of`
    /// only an element-indexed one. Both are static — the declaration and
    /// the reading form are both content.
    ///
    /// # Errors
    ///
    /// [`MetricError::Unregistered`] (`E-LOAD-011`),
    /// [`MetricError::WrongReadingForm`] (`E-LOAD-012`).
    pub fn check_reading_forms(
        &self,
        rule: &SExpr,
        decls: &[crate::bindings::BindingDecl],
    ) -> Result<(), MetricError> {
        for decl in decls {
            let crate::bindings::BindSource::Metric(name) = &decl.source else {
                continue;
            };
            let Some(metric) = self.get(name) else {
                return Err(MetricError::Unregistered { name: name.clone() });
            };
            if metric.domain != MetricDomain::Graph {
                return Err(MetricError::WrongReadingForm {
                    name: name.clone(),
                    detail: "an element-indexed metric is read by the metric-of \
                             accessor, since its value depends on an element a \
                             binding does not name",
                });
            }
        }
        self.walk_metric_of(rule)
    }

    fn walk_metric_of(&self, expr: &SExpr) -> Result<(), MetricError> {
        let SExpr::List(items) = expr else {
            return Ok(());
        };
        if matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(h))) if h == "metric-of") {
            let Some(SExpr::Atom(Atom::Symbol(name))) = items.get(2) else {
                return Err(malformed(
                    "(metric-of <expr> <symbol>) — unrecognized shape",
                ));
            };
            let Some(metric) = self.get(name) else {
                return Err(MetricError::Unregistered { name: name.clone() });
            };
            if metric.domain == MetricDomain::Graph {
                return Err(MetricError::WrongReadingForm {
                    name: name.clone(),
                    detail: "a graph-scope metric is read by a :metric binding, \
                             not by the metric-of accessor",
                });
            }
        }
        for child in items {
            self.walk_metric_of(child)?;
        }
        Ok(())
    }

    /// Register every metric's declared type and kind in the §3.4 kind
    /// environment (D55: a `:metric` binding and a `metric-of` accessor
    /// carry the **declared** `:kind`, superseding D12's metric clause).
    ///
    /// Metric names are `symbol`s and field names are `qname`s, so the two
    /// namespaces are disjoint by §1.4 and this merge cannot collide.
    pub fn merge_into_kind_env(&self, fields: &mut HashMap<String, FieldDecl>) {
        for (name, metric) in &self.metrics {
            fields.insert(
                name.clone(),
                FieldDecl {
                    ty: metric.ty.clone(),
                    kind: metric.kind,
                },
            );
        }
    }

    /// Every registered metric name, ascending.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.metrics.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

fn parse_metric(form: &SExpr) -> Result<MetricDecl, MetricError> {
    let SExpr::List(items) = form else {
        return Err(malformed("a metric declaration must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Symbol(name)), rest @ ..] =
        items.as_slice()
    else {
        return Err(malformed(
            "(metric <symbol> :type <T> :kind <k> <domain> :provider <symbol>) \
             — unrecognized shape",
        ));
    };
    if head != "metric" {
        return Err(malformed(format!("expected (metric …), found ({head} …)")));
    }
    let mut ty: Option<BslType> = None;
    let mut kind: Option<FieldKind> = None;
    let mut domain: Option<MetricDomain> = None;
    let mut provider: Option<String> = None;
    let mut cursor = rest;
    while !cursor.is_empty() {
        match cursor {
            [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Symbol(value)), tail @ ..] => {
                match kw.as_str() {
                    "type" => ty = Some(parse_type_name(value).map_err(|e| from_decl(&e))?),
                    "kind" => {
                        kind = Some(match value.as_str() {
                            "intensive" => FieldKind::Intensive,
                            "extensive" => FieldKind::Extensive,
                            other => {
                                return Err(malformed(format!(
                                    ":kind is intensive|extensive, found {other}"
                                )))
                            }
                        });
                    }
                    "provider" => provider = Some(value.clone()),
                    other => {
                        return Err(malformed(format!(
                            "a metric takes :type, :kind, <domain> and :provider, found :{other}"
                        )))
                    }
                }
                cursor = tail;
            }
            [SExpr::List(inner), tail @ ..] => {
                domain = Some(parse_domain(inner)?);
                cursor = tail;
            }
            other => {
                return Err(malformed(format!(
                    "unexpected item in a metric declaration: {:?}",
                    other.first()
                )))
            }
        }
    }
    match (ty, kind, domain, provider) {
        (Some(ty), Some(kind), Some(domain), Some(provider)) => Ok(MetricDecl {
            name: name.clone(),
            ty,
            kind,
            domain,
            provider,
        }),
        _ => Err(malformed(
            "a metric declares :type, :kind, a <domain> and :provider (§2.11)",
        )),
    }
}

fn parse_domain(items: &[SExpr]) -> Result<MetricDomain, MetricError> {
    match items {
        [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Keyword(kw))]
            if head == "domain" && kw == "graph" =>
        {
            Ok(MetricDomain::Graph)
        }
        [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::EnumRef { enum_type, member })]
            if head == "domain" =>
        {
            // The KIND check is `grammar::check_enum_ref_kinds`' (D74).
            if EnumKind::from_type_name(enum_type) != Some(EnumKind::NodeType) {
                return Err(malformed(format!(
                    "(domain {enum_type}/{member}) — a metric's element domain \
                     is a NodeType member (E-TYPE-011)"
                )));
            }
            Ok(MetricDomain::Element(render_member(member)))
        }
        other => Err(malformed(format!(
            "(domain <enum-ref> | :graph) — unrecognized shape {other:?}"
        ))),
    }
}

fn from_decl(e: &DeclError) -> MetricError {
    MetricError::Malformed {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricDomain, MetricRegistry};
    use crate::bindings::parse_bindings;
    use crate::reader::read;
    use crate::types::FieldKind;
    use std::collections::HashMap;

    fn e(source: &str) -> crate::reader::SExpr {
        read(source).expect("test source must parse").0
    }

    const GRAPH_METRIC: &str = "(metric solidarity-density \
        :type coefficient :kind intensive (domain :graph) :provider topology-scores)";
    const ELEMENT_METRIC: &str = "(metric betweenness-centrality \
        :type coefficient :kind intensive (domain NodeType/ORGANIZATION) \
        :provider topology-scores)";

    fn registry() -> MetricRegistry {
        let mut r = MetricRegistry::default();
        r.declare(&e(GRAPH_METRIC)).expect("graph metric");
        r.declare(&e(ELEMENT_METRIC)).expect("element metric");
        r
    }

    fn rule(body: &str) -> crate::reader::SExpr {
        e(&format!(
            "(rule demo/m :role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 4096 {body})"
        ))
    }

    #[test]
    fn a_metric_declares_type_kind_domain_and_provider() {
        let r = registry();
        let graph = r.get("solidarity-density").unwrap();
        assert_eq!(graph.domain, MetricDomain::Graph);
        assert_eq!(graph.kind, FieldKind::Intensive);
        assert_eq!(graph.provider, "topology-scores");
        assert_eq!(
            r.get("betweenness-centrality").unwrap().domain,
            MetricDomain::Element("organization".to_owned())
        );
    }

    #[test]
    fn a_graph_metric_read_by_a_metric_binding_and_an_element_one_by_metric_of() {
        let r = registry();
        let ok = rule(
            "(bindings (binding d :metric solidarity-density) \
                       (binding c :expr (metric-of self betweenness-centrality))) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        );
        let decls = parse_bindings(&ok).unwrap();
        assert_eq!(r.check_reading_forms(&ok, &decls), Ok(()));
    }

    #[test]
    fn each_read_through_the_others_form_is_e_load_012() {
        let r = registry();
        let via_binding = rule(
            "(bindings (binding c :metric betweenness-centrality)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        );
        let decls = parse_bindings(&via_binding).unwrap();
        assert_eq!(
            r.check_reading_forms(&via_binding, &decls)
                .unwrap_err()
                .spec_code(),
            Some("E-LOAD-012")
        );

        let via_accessor = rule(
            "(bindings (binding d :expr (metric-of self solidarity-density))) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        );
        let decls = parse_bindings(&via_accessor).unwrap();
        assert_eq!(
            r.check_reading_forms(&via_accessor, &decls)
                .unwrap_err()
                .spec_code(),
            Some("E-LOAD-012")
        );
    }

    #[test]
    fn an_unregistered_name_is_e_load_011_through_both_forms() {
        let r = registry();
        for body in [
            "(bindings (binding x :metric nowhere)) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
            "(bindings (binding x :expr (metric-of self nowhere))) \
             (effects (update-node self social-class/agitation (add 0.05i)))",
        ] {
            let form = rule(body);
            let decls = parse_bindings(&form).unwrap();
            assert_eq!(
                r.check_reading_forms(&form, &decls)
                    .unwrap_err()
                    .spec_code(),
                Some("E-LOAD-011"),
                "never 0.0 (§6.3): {body}"
            );
        }
    }

    #[test]
    fn kernel_disagreement_is_e_load_025() {
        let r = registry();
        let mut kernel = HashMap::new();
        let mut registered = r.get("solidarity-density").unwrap().clone();
        registered.kind = FieldKind::Extensive; // the kernel says extensive
        kernel.insert("solidarity-density".to_owned(), registered);
        assert_eq!(
            r.check_against_kernel(&kernel).unwrap_err().spec_code(),
            Some("E-LOAD-025")
        );
    }

    #[test]
    fn the_declared_kind_reaches_the_aggregation_law() {
        // D55 supersedes D12's metric clause: a metric carries the kind its
        // registration declares, so an intensive one feeding an unweighted
        // mean is E-TYPE-042 exactly as an intensive field is.
        use crate::typecheck::{typecheck_aggregation, TypeCode, TypeEnv};
        let mut fields = HashMap::new();
        registry().merge_into_kind_env(&mut fields);
        let env = TypeEnv {
            fields,
            exemptions: &[],
        };
        let err = typecheck_aggregation(&e("(mean solidarity-density)"), &env).unwrap_err();
        assert_eq!(err.code, Some(TypeCode::UnweightedMeanOfIntensive));
    }

    #[test]
    fn a_duplicate_metric_declaration_is_e_load_001() {
        let mut r = registry();
        assert_eq!(
            r.declare(&e(GRAPH_METRIC)).unwrap_err().spec_code(),
            Some("E-LOAD-001")
        );
    }
}
