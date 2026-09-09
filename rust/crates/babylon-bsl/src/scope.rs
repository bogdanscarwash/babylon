//! Reference scoping (`bsl-language.rst` §2.5): where a `:field` binding of
//! a **foreign** node type may be read, and how many candidate bodies may be
//! in scope when it is.
//!
//! §2.5 has said since this document's first revision that a `:field` qname
//! whose first segment names a node type other than the subject's "is only
//! legal inside a fold body over that type" (`E-TYPE-010`). R9 chapter C1
//! adds the other half (D30): a reference under **two or more** enclosing
//! bodies ranging over that same node type is `E-TYPE-013` — the reference
//! is ambiguous, and the author names an element with `:as` (§2.6) and reads
//! it through `field-of` instead. Single-body code is unaffected.
//!
//! The walk is purely structural and terminates by construction (an `SExpr`
//! is a finite tree — the same argument `canonical_ast` and `bound_checker`
//! rely on).

use crate::bindings::{BindSource, BindingDecl};
use crate::reader::{Atom, SExpr};
use crate::vocabulary::{render_member, ClosedVocabulary, EnumKind};

/// A scoping rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeError {
    /// `E-TYPE-010` — a foreign-node-type `:field` binding referenced
    /// outside every body ranging over that node type.
    ForeignFieldOutsideBody {
        /// The binding name.
        binding: String,
        /// The node-type segment it owns off.
        owner: String,
    },
    /// `E-TYPE-013` — the same reference under two or more enclosing bodies
    /// of that node type: ambiguous, and repaired by naming an element.
    AmbiguousForeignField {
        /// The binding name.
        binding: String,
        /// The node-type segment it owns off.
        owner: String,
        /// How many enclosing bodies range over that type.
        candidates: usize,
    },
}

impl ScopeError {
    /// The spec's error code.
    #[must_use]
    pub fn spec_code(&self) -> &'static str {
        match self {
            Self::ForeignFieldOutsideBody { .. } => "E-TYPE-010",
            Self::AmbiguousForeignField { .. } => "E-TYPE-013",
        }
    }
}

impl std::fmt::Display for ScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignFieldOutsideBody { binding, owner } => write!(
                f,
                "E-TYPE-010: binding {binding} reads a field of the foreign \
                 node type '{owner}' outside every body ranging over it (§2.5)"
            ),
            Self::AmbiguousForeignField {
                binding,
                owner,
                candidates,
            } => write!(
                f,
                "E-TYPE-013: binding {binding} ('{owner}') is referenced under \
                 {candidates} enclosing bodies of that node type — name an \
                 element with :as and read it with field-of (§2.5, D30)"
            ),
        }
    }
}

impl std::error::Error for ScopeError {}

/// The node-type **segment** a query's elements range over, if the query
/// annotates one. Only `nodes` and `neighbors` do: `members-of` yields
/// `NodeRef`s of no annotated type, and the edge/hyperedge queries yield no
/// nodes at all.
#[must_use]
pub fn query_node_type_segment(query_items: &[SExpr]) -> Option<String> {
    let head = match query_items.first() {
        Some(SExpr::Atom(Atom::Symbol(s))) => s.as_str(),
        _ => return None,
    };
    let index = match head {
        "nodes" => 1,
        // §2.6 (C8): the mandatory FOURTH operand is the result NodeType.
        "neighbors" => 4,
        _ => return None,
    };
    match query_items.get(index) {
        Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) if enum_type == "NodeType" => {
            Some(render_member(member))
        }
        _ => None,
    }
}

/// Whether a form is one of §2.6's six query heads.
pub(crate) fn is_query(items: &[SExpr]) -> bool {
    matches!(
        items.first(),
        Some(SExpr::Atom(Atom::Symbol(s)))
            if matches!(
                s.as_str(),
                "nodes" | "edges" | "neighbors" | "hyperedges" | "members-of" | "hyperedges-of"
            )
    )
}

/// The iterating forms whose bodies range over a query's elements (§2.7's
/// `fold`/`select-*`, §2.4's `exists`/`forall`, §2.8's `for-each`). Each
/// takes its `<query>` as its first operand after the head — except `fold`,
/// whose `<fold-op>` comes first.
pub(crate) fn iterating_query_index(head: &str) -> Option<usize> {
    match head {
        "fold" => Some(2),
        "exists" | "forall" | "select-max" | "select-min" | "for-each" => Some(1),
        _ => None,
    }
}

/// The child index of a query's element **predicate**, if its production
/// has one (§2.6). Only the three predicate queries do: `nodes`, `edges`
/// and `hyperedges` take a `<…-pred>?` at index 2. `neighbors`,
/// `members-of` and `hyperedges-of` carry an element *operand* instead,
/// which §2.6 evaluates in the outer scope.
fn query_predicate_index(items: &[SExpr]) -> Option<usize> {
    match items.first() {
        Some(SExpr::Atom(Atom::Symbol(head)))
            if matches!(head.as_str(), "nodes" | "edges" | "hyperedges") =>
        {
            Some(2)
        }
        _ => None,
    }
}

/// Count the enclosing bodies of node type `owner` at every reference to
/// `binding`, and apply §2.5's two rules.
fn walk(
    expr: &SExpr,
    binding: &str,
    owner: &str,
    depth: usize,
    result: &mut Result<(), ScopeError>,
) {
    if result.is_err() {
        return;
    }
    match expr {
        SExpr::Atom(Atom::Symbol(name)) if name == binding => {
            if depth == 0 {
                *result = Err(ScopeError::ForeignFieldOutsideBody {
                    binding: binding.to_owned(),
                    owner: owner.to_owned(),
                });
            } else if depth > 1 {
                *result = Err(ScopeError::AmbiguousForeignField {
                    binding: binding.to_owned(),
                    owner: owner.to_owned(),
                    candidates: depth,
                });
            }
        }
        SExpr::Atom(_) => {}
        SExpr::List(items) => {
            let head = match items.first() {
                Some(SExpr::Atom(Atom::Symbol(s))) => Some(s.as_str()),
                _ => None,
            };
            // A query form's element **predicate** is a body over that
            // query's elements, exactly as a fold body is — but its
            // element **operand** is not. §2.6's `neighbors` yields
            // "nodes reachable from the operand", so the operand is the
            // *source*, evaluated in the outer scope before any element
            // exists, and `neighbors` has no predicate at all. Charging it
            // at the raised depth both accepted a rule-scope foreign read
            // (E-TYPE-010 missed) and turned a legal one-body read into a
            // spurious E-TYPE-013.
            if is_query(items) {
                let inner_depth = depth + usize::from(matches_owner(items, owner));
                for (i, child) in items.iter().enumerate().skip(1) {
                    let child_depth = if Some(i) == query_predicate_index(items) {
                        inner_depth
                    } else {
                        depth
                    };
                    walk(child, binding, owner, child_depth, result);
                }
                return;
            }
            if let Some(query_index) = head.and_then(iterating_query_index) {
                let ranges_over = match items.get(query_index) {
                    Some(SExpr::List(q)) if is_query(q) => matches_owner(q, owner),
                    _ => false,
                };
                let inner_depth = depth + usize::from(ranges_over);
                for (i, child) in items.iter().enumerate().skip(1) {
                    // The query operand itself is evaluated in the OUTER
                    // scope; only the body sees the new element.
                    let child_depth = if i == query_index { depth } else { inner_depth };
                    walk(child, binding, owner, child_depth, result);
                }
                return;
            }
            for child in &items[1..] {
                walk(child, binding, owner, depth, result);
            }
        }
    }
}

/// The `:expr` operands of one `(binding …)` row — the only expressions a
/// binding declaration contains (§2.5, R9 chapter C7).
fn expr_operands(row: &SExpr) -> Vec<&SExpr> {
    let SExpr::List(items) = row else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for window in items.windows(2) {
        if let [SExpr::Atom(Atom::Keyword(kw)), operand] = window {
            if kw == "expr" {
                out.push(operand);
            }
        }
    }
    out
}

fn matches_owner(query_items: &[SExpr], owner: &str) -> bool {
    query_node_type_segment(query_items).is_some_and(|segment| segment == owner)
}

/// Check every foreign-node-type `:field` binding of a rule against §2.5's
/// two rules, given the rule's subject node-type **segment** (`None` for a
/// `(domain :graph)` rule, where every `:field` binding is foreign).
///
/// # Errors
///
/// [`ScopeError::ForeignFieldOutsideBody`] (`E-TYPE-010`) or
/// [`ScopeError::AmbiguousForeignField`] (`E-TYPE-013`).
pub fn check_foreign_field_scoping(
    rule: &SExpr,
    decls: &[BindingDecl],
    subject_segment: Option<&str>,
    vocabulary: &ClosedVocabulary,
) -> Result<(), ScopeError> {
    let SExpr::List(items) = rule else {
        return Ok(());
    };
    for decl in decls {
        let BindSource::Field(qname) = &decl.source else {
            continue;
        };
        let segment = qname.split('/').next().unwrap_or(qname);
        // Only NODE-type fields are read by a `:field` binding at all (D29);
        // an edge/hyperedge owner is the grammar checker's rejection, not
        // this pass's.
        match vocabulary.owner_of(segment) {
            Ok((EnumKind::NodeType, _)) => {}
            _ => continue,
        }
        if subject_segment == Some(segment) {
            continue; // the subject's own field: always in scope
        }
        let mut result = Ok(());
        for child in items {
            let SExpr::List(inner) = child else { continue };
            match inner.first() {
                Some(SExpr::Atom(Atom::Symbol(h))) if h == "when" || h == "effects" => {
                    for body in &inner[1..] {
                        walk(body, &decl.name, segment, 0, &mut result);
                    }
                }
                // A `(bindings …)` form's rows carry their own NAMES in
                // value-shaped positions, so the whole form must not be
                // walked: only a `:expr` operand (§2.5, C7) is a reference
                // site, and a `:expr` is evaluated at rule scope (depth 0).
                Some(SExpr::Atom(Atom::Symbol(h))) if h == "bindings" => {
                    for row in &inner[1..] {
                        for expr in expr_operands(row) {
                            walk(expr, &decl.name, segment, 0, &mut result);
                        }
                    }
                }
                _ => {}
            }
        }
        result?;
    }
    Ok(())
}

/// Whether `name` is referenced anywhere **outside every** iterating body —
/// the "at rule scope" test §2.3's domain inference is stated in terms of
/// ("referenced at least once outside every query body"). Unlike the
/// foreign-field walk above, this counts *any* iterating form, not only
/// ones ranging over one type: the inference asks where a reference sits,
/// not what it ranges over.
///
/// It shares `child_is_inside` with every other element-scope walk in the
/// crate, which is what keeps "a query's element operand is evaluated in
/// the OUTER scope" one rule rather than three copies of it. It previously
/// carried its own copy and had drifted: it treated *all* of a query
/// form's children as inside the body, contradicting its own comment, so a
/// `:field` binding referenced only in a `neighbors`/`members-of` operand
/// at rule scope was invisible to §2.3's domain inference.
#[must_use]
pub fn referenced_at_rule_scope(expr: &SExpr, name: &str) -> bool {
    fn go(expr: &SExpr, name: &str, inside_body: bool) -> bool {
        match expr {
            SExpr::Atom(Atom::Symbol(s)) => !inside_body && s == name,
            SExpr::Atom(_) => false,
            SExpr::List(items) => {
                let iterating = is_iterating(items);
                items.iter().enumerate().skip(1).any(|(i, child)| {
                    let child_inside = inside_body || (iterating && child_is_inside(items, i));
                    go(child, name, child_inside)
                })
            }
        }
    }
    go(expr, name, false)
}

/// D54: `:as` optionally names an iterating form's element. The name is in
/// scope for the whole body of its form, **including nested bodies**, and
/// shares the rule's binding namespace:
///
/// - colliding with a binding or another `:as` name is `E-PARSE-030`;
/// - naming `self` or `it` is `E-PARSE-022`;
/// - referenced outside its body it is `E-TYPE-012`, the same code and the
///   same reason as `it` outside a query context.
///
/// D53 is a reading, not a repeal: `it` always denotes the element of the
/// **innermost** enclosing iterating form, which is rebinding by
/// construction — `it` is never *declared*, so there is no declaration for
/// an inner form to shadow, and `E-PARSE-022`'s prohibition is untouched.
///
/// **`:as` names are rule-scoped-unique.** §2.6 says they "share the rule's
/// binding namespace", and a binding name is unique per rule
/// (`E-PARSE-030`), so a second `:as` of one name anywhere in the rule —
/// nested or sibling — collides. That is also what makes the `E-TYPE-012`
/// test below unambiguous: a name is either open here or referenced
/// outside its one body.
///
/// The walk covers the `<when>` and `<effects>` bodies **and every `:expr`
/// binding operand** (§2.5, C7): a `:expr` is evaluated at rule scope, so
/// `it` inside one is `E-TYPE-012` — while a fold *nested* inside a
/// `:expr` binds `it` normally, which the element stack already models.
///
/// # Errors
///
/// [`ElementNameError`] as above.
pub fn check_element_names(
    rule: &SExpr,
    declared_bindings: &[String],
) -> Result<(), ElementNameError> {
    let mut names = ElementScope {
        bindings: declared_bindings.to_vec(),
        all_declared: declared_element_names(rule),
        seen: Vec::new(),
        open: Vec::new(),
        element_depth: 0,
    };
    for site in reference_sites(rule) {
        walk_names(site, &mut names)?;
    }
    Ok(())
}

/// Every `:as` name declared anywhere in a rule's expression positions —
/// the set [`crate::bindings::check_free_variables`] must admit, since a
/// reference to one is a legal variable reference and not an undeclared
/// name. Scope correctness (a reference *outside* the name's body) is this
/// module's `E-TYPE-012`, raised earlier in the pipeline.
#[must_use]
pub fn declared_element_names(rule: &SExpr) -> Vec<String> {
    let mut out = Vec::new();
    for site in reference_sites(rule) {
        collect_elem_names(site, &mut out);
    }
    out
}

fn collect_elem_names(expr: &SExpr, out: &mut Vec<String>) {
    let SExpr::List(items) = expr else { return };
    if is_iterating(items) {
        if let Some(name) = elem_name(items) {
            if !out.iter().any(|n| n == name) {
                out.push(name.to_owned());
            }
        }
    }
    for child in items {
        collect_elem_names(child, out);
    }
}

/// Every expression position of a rule: the `<when>` and `<effects>`
/// bodies, plus each `:expr` binding operand (§2.5).
fn reference_sites(rule: &SExpr) -> Vec<&SExpr> {
    let SExpr::List(items) = rule else {
        return Vec::new();
    };
    let mut sites = Vec::new();
    for child in items {
        let SExpr::List(inner) = child else { continue };
        match inner.first() {
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "when" || h == "effects" => {
                sites.extend(&inner[1..]);
            }
            Some(SExpr::Atom(Atom::Symbol(h))) if h == "bindings" => {
                for row in &inner[1..] {
                    sites.extend(expr_operands(row));
                }
            }
            _ => {}
        }
    }
    sites
}

/// A `:as` element-name rejection (D54).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementNameError {
    /// `E-PARSE-022` — `:as it` or `:as self`.
    ReservedElementName {
        /// The reserved name.
        name: String,
    },
    /// `E-PARSE-030` — a `:as` name colliding with a binding or another
    /// `:as` name.
    ElementNameCollision {
        /// The colliding name.
        name: String,
    },
    /// `E-TYPE-012` — a `:as` name (or `it`) referenced outside its body.
    NameOutsideItsBody {
        /// The out-of-scope name.
        name: String,
    },
}

impl ElementNameError {
    /// The spec's error code.
    #[must_use]
    pub fn spec_code(&self) -> &'static str {
        match self {
            Self::ReservedElementName { .. } => "E-PARSE-022",
            Self::ElementNameCollision { .. } => "E-PARSE-030",
            Self::NameOutsideItsBody { .. } => "E-TYPE-012",
        }
    }
}

impl std::fmt::Display for ElementNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReservedElementName { name } => write!(
                f,
                "E-PARSE-022: :as {name} — {name} is reserved and always in \
                 scope; it is never declared and never shadowed (§2.5)"
            ),
            Self::ElementNameCollision { name } => write!(
                f,
                "E-PARSE-030: the :as name {name} collides with a binding or \
                 another :as name — they share the rule's namespace (§2.6)"
            ),
            Self::NameOutsideItsBody { name } => write!(
                f,
                "E-TYPE-012: {name} is referenced outside its body — the same \
                 code and the same reason as `it` outside a query context (§2.6)"
            ),
        }
    }
}

impl std::error::Error for ElementNameError {}

/// The `:as <symbol>` name an iterating form declares, if any.
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

/// Whether a form introduces an element scope: a `<query>` (whose element
/// predicate ranges over its elements) or one of §2.7/§2.8's iterating
/// forms.
pub(crate) fn is_iterating(items: &[SExpr]) -> bool {
    let head = match items.first() {
        Some(SExpr::Atom(Atom::Symbol(s))) => Some(s.as_str()),
        _ => None,
    };
    is_query(items) || head.is_some_and(|h| iterating_query_index(h).is_some())
}

/// The children of `items` that sit **inside** the element scope the form
/// introduces, as `(index, inside)` pairs. A query's element predicate is
/// inside; its element operand is not. An iterating form's body is inside;
/// its `<query>` operand is not (the query is evaluated in the outer
/// scope), and neither is its `:as <symbol>` declaration.
pub(crate) fn child_is_inside(items: &[SExpr], index: usize) -> bool {
    if is_query(items) {
        return query_predicate_index(items) == Some(index);
    }
    let head = match items.first() {
        Some(SExpr::Atom(Atom::Symbol(s))) => s.as_str(),
        _ => return false,
    };
    match iterating_query_index(head) {
        Some(query_index) => index != query_index,
        None => false,
    }
}

/// The names in scope while walking, and how deep inside element scopes we
/// are.
///
/// Three sets, because they answer three different questions and conflating
/// two of them is a defect:
///
/// - `all_declared` — every `:as` name declared **anywhere** in the rule,
///   computed before the walk starts. The reference test consults this one,
///   so a reference appearing structurally *before* its declaration is
///   caught exactly like one appearing after. Testing against a walk-order
///   set instead let a **forward** out-of-body reference load silently — a
///   III.11 inversion, since the pre-repair code at least rejected it
///   (with the wrong code).
/// - `seen` — the names met so far **in walk order**. Only the collision
///   test uses it, and it must be walk-order: a name's own declaration is
///   not yet in it, so a first `:as` never collides with itself while a
///   second one does.
/// - `open` — the subset whose body we are currently inside.
struct ElementScope {
    bindings: Vec<String>,
    all_declared: Vec<String>,
    seen: Vec<String>,
    open: Vec<String>,
    element_depth: usize,
}

impl ElementScope {
    fn declares(&self, name: &str) -> bool {
        self.bindings.iter().any(|n| n == name) || self.seen.iter().any(|n| n == name)
    }

    /// Whether `name` is a `:as` name of this rule referenced outside its
    /// body. A declared **binding** of that name is excluded: a binding
    /// reference is legal at rule scope, and the `:as` reusing its name is
    /// the error — reported at the declaration as `E-PARSE-030`.
    fn is_out_of_body_element(&self, name: &str) -> bool {
        !self.bindings.iter().any(|n| n == name)
            && self.all_declared.iter().any(|n| n == name)
            && !self.open.iter().any(|n| n == name)
    }
}

/// Walk one expression position, enforcing D54's three rules.
fn walk_names(expr: &SExpr, scope: &mut ElementScope) -> Result<(), ElementNameError> {
    match expr {
        SExpr::Atom(Atom::Symbol(name)) => {
            // `it` denotes the innermost enclosing element (D53), so it is
            // meaningful only inside one.
            if name == "it" && scope.element_depth == 0 {
                return Err(ElementNameError::NameOutsideItsBody { name: name.clone() });
            }
            // A `:as` name is in scope for its own body only. Declared
            // somewhere in this rule but not open here means the author
            // reached past its body — D54's E-TYPE-012, the same code and
            // the same reason as `it` outside a query context. The test is
            // against the RULE-WIDE set, so it catches a forward reference
            // as well as a backward one.
            if scope.is_out_of_body_element(name) {
                return Err(ElementNameError::NameOutsideItsBody { name: name.clone() });
            }
            Ok(())
        }
        SExpr::Atom(_) => Ok(()),
        SExpr::List(items) => {
            let iterating = is_iterating(items);
            let mut introduced: Option<String> = None;
            if iterating {
                if let Some(name) = elem_name(items) {
                    if crate::bindings::RESERVED_NAMES.contains(&name) {
                        return Err(ElementNameError::ReservedElementName {
                            name: name.to_owned(),
                        });
                    }
                    if scope.declares(name) {
                        return Err(ElementNameError::ElementNameCollision {
                            name: name.to_owned(),
                        });
                    }
                    introduced = Some(name.to_owned());
                }
            }
            for (i, child) in items.iter().enumerate().skip(1) {
                let inside = iterating && child_is_inside(items, i);
                if inside {
                    scope.element_depth += 1;
                    if let Some(name) = &introduced {
                        // `seen` is rule-wide and never popped (uniqueness);
                        // `open` is the body-scoped half.
                        if !scope.seen.iter().any(|n| n == name) {
                            scope.seen.push(name.clone());
                        }
                        scope.open.push(name.clone());
                    }
                }
                let result = walk_names(child, scope);
                if inside {
                    scope.element_depth -= 1;
                    if introduced.is_some() {
                        scope.open.pop();
                    }
                }
                result?;
            }
            // A form that declared a name but whose body never opened (a
            // malformed shape) still records the name, so a later sibling
            // reusing it collides rather than silently shadowing.
            if let Some(name) = introduced {
                if !scope.seen.contains(&name) {
                    scope.seen.push(name);
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::check_foreign_field_scoping;
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
                    "TERRITORY".to_owned(),
                ],
            ),
            (EnumKind::EdgeType, vec!["SOLIDARITY".to_owned()]),
        ])
        .unwrap()
    }

    fn check(source: &str, subject: Option<&str>) -> Result<(), super::ScopeError> {
        let (rule, _) = read(source).expect("test source must parse");
        let decls = parse_bindings(&rule).expect("bindings must parse");
        check_foreign_field_scoping(&rule, &decls, subject, &vocabulary())
    }

    const PREAMBLE: &str =
        ":role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 4096";

    #[test]
    fn the_subjects_own_field_is_always_in_scope() {
        let source = format!(
            "(rule demo/own {PREAMBLE} \
             (bindings (binding wealth :field social-class/wealth)) \
             (when (< wealth 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(check(&source, Some("social-class")), Ok(()));
    }

    #[test]
    fn a_foreign_field_outside_every_body_is_e_type_010() {
        let source = format!(
            "(rule demo/foreign {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< claim 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        let err = check(&source, Some("social-class")).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-010");
    }

    #[test]
    fn a_foreign_field_inside_one_body_over_that_type_is_legal() {
        let source = format!(
            "(rule demo/one-body {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) claim) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(check(&source, Some("social-class")), Ok(()));
    }

    #[test]
    fn two_enclosing_bodies_of_that_type_is_e_type_013() {
        let source = format!(
            "(rule demo/ambiguous {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) \
                        (fold max (nodes NodeType/ORGANIZATION) claim)) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        let err = check(&source, Some("social-class")).unwrap_err();
        assert_eq!(err.spec_code(), "E-TYPE-013");
    }

    #[test]
    fn a_body_over_a_different_node_type_does_not_license_the_read() {
        let source = format!(
            "(rule demo/wrong-body {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/TERRITORY) claim) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(
            check(&source, Some("social-class"))
                .unwrap_err()
                .spec_code(),
            "E-TYPE-010"
        );
    }

    #[test]
    fn nesting_a_different_type_inside_does_not_double_count() {
        let source = format!(
            "(rule demo/nested-mixed {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) \
                        (fold max (nodes NodeType/TERRITORY) claim)) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(check(&source, Some("social-class")), Ok(()));
    }

    #[test]
    fn the_query_operand_is_evaluated_in_the_outer_scope() {
        // A reference in the query's own operand position sits OUTSIDE the
        // body it introduces — reading it there is still E-TYPE-010.
        //
        // **This vector previously used `hyperedges-of`, and pinned
        // nothing**: `query_node_type_segment` returns `None` for that head,
        // so `matches_owner` was false and the assertion held under either
        // reading of operand-vs-predicate. `neighbors` is the one head that
        // both annotates a result `NodeType` and carries an element
        // operand, so it is the only head that can exercise the rule.
        let source = format!(
            "(rule demo/operand {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold count (neighbors claim EdgeType/SOLIDARITY :out \
                                    NodeType/ORGANIZATION) 1) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(
            check(&source, Some("social-class"))
                .unwrap_err()
                .spec_code(),
            "E-TYPE-010"
        );
    }

    /// The dual, which the old walk got wrong in the other direction: a
    /// legal read under exactly ONE enclosing body of that type must not
    /// become `E-TYPE-013` because a `neighbors` operand sits between.
    #[test]
    fn a_neighbors_operand_does_not_double_count_the_enclosing_body() {
        let source = format!(
            // The result type must BE the owner type: with any other type
            // `matches_owner` is false and the vector holds against the
            // pre-fix walk as well, pinning nothing.
            "(rule demo/no-double {PREAMBLE} \
             (bindings (binding claim :field organization/claim-strength)) \
             (when (< (fold sum (nodes NodeType/ORGANIZATION) \
                        (fold count (neighbors claim EdgeType/SOLIDARITY :out \
                                      NodeType/ORGANIZATION) 1)) 5)) \
             (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        assert_eq!(check(&source, Some("social-class")), Ok(()));
    }
}
