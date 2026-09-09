//! The four generic locator strategies (issue #652 Task 6, plan §6.2):
//! given WHAT a loader error is about (an [`ErrorIdentity`], minted by
//! `babylon-bsl`'s `identity_of`/`ScenarioError::identity`), find WHERE it
//! is in one file's already-parsed `(SExpr forest, SpanTable)`. The
//! server never re-derives identity from message text (sentinel 7.2) —
//! everything here reads typed data the loader already computed.
//!
//! **Four strategies, not 52 predicates** (§6.2): `by_atom`, `by_qname`,
//! `by_head_and_operand` (covering both [`ErrorIdentity::Enum`] via a
//! dedicated `by_enum_ref` search and [`ErrorIdentity::Edge`]),
//! `by_keyword_operand`/`by_operand_index`. [`locate`] is the wildcard-free
//! dispatcher from an [`ErrorIdentity`] variant to the strategy that
//! serves it (§6.2's own roster).
//!
//! **The three outcomes** (§6.2): a strategy either finds exactly one
//! candidate ([`LocateOutcome::Unique`], the diagnostic's own range), two
//! or more ([`LocateOutcome::Ambiguous`], the diagnostic falls back to
//! file level with one `relatedInformation` entry per candidate, sorted
//! into document order so "the second declaration" names
//! `candidates[1]`), or none ([`LocateOutcome::Absent`], file level, no
//! `relatedInformation`).
//!
//! **On [`ErrorIdentity::Edge`]:** no construction site in `babylon-bsl`
//! produces this variant today (verified: `rg -n
//! 'ErrorIdentity::Edge' crates/babylon-bsl/src` matches only a doc
//! comment). `by_head_and_operand`'s Edge arm is implemented to the
//! documented contract in good faith but is untested against any real
//! error — wave 2, whichever construction site first produces an `Edge`
//! identity, should add a conformance row and revisit this arm rather
//! than trust it unverified.

use std::collections::HashSet;

use babylon_bsl::{Atom, ErrorIdentity, SExpr, Span, SpanTable};

/// What a locator strategy found (§6.2's three outcomes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocateOutcome {
    /// Exactly one candidate.
    Unique(Span),
    /// Two or more candidates, sorted by `(start, end)` ascending —
    /// document order, so the Nth declaration is `candidates[N - 1]`.
    Ambiguous(Vec<Span>),
    /// No candidate found in this file's forest.
    Absent,
}

/// The wildcard-free dispatch from an [`ErrorIdentity`] to the strategy
/// that locates it (§6.2's roster). A new `ErrorIdentity` variant is a
/// compile error here, the same exhaustiveness guarantee `identity_of`
/// (`babylon-bsl`) already gives at the identity-minting end.
#[must_use]
pub fn locate(identity: &ErrorIdentity, forest: &[SExpr], spans: &SpanTable) -> LocateOutcome {
    match identity {
        ErrorIdentity::Name(token)
        | ErrorIdentity::NodeLocal(token)
        | ErrorIdentity::RuleId(token) => by_atom(token, forest, spans),
        ErrorIdentity::Field(qname) => by_qname(qname, forest, spans),
        ErrorIdentity::Enum { enum_type, member } => {
            by_enum_ref(enum_type, member.as_deref(), forest, spans)
        }
        ErrorIdentity::Edge {
            edge_type,
            from,
            to: _,
        } => by_head_and_operand("edge", &[edge_type.as_str(), from.as_str()], forest, spans),
        ErrorIdentity::Keyword(keyword) => by_keyword_operand(keyword, forest, spans),
        ErrorIdentity::Operand { form, index } => by_operand_index(form, *index, forest, spans),
        // A candidate's own identity is itself outcome-shaped — locate
        // each candidate independently, then union the outcome as
        // Ambiguous over WHATEVER spans resolve (skipping a candidate
        // that resolves Absent), by construction (the loader itself could
        // not narrow it further; the locator does not guess where it
        // could not). Zero resolvable candidates degrades to `Absent`,
        // never a panic over an empty `Ambiguous(vec![])` — a data shape
        // this crate did not mint (`DomainError::Undeterminable`'s own
        // `candidates` is non-empty by construction, `domain.rs`), but
        // this function does not assume that of an arbitrary caller.
        ErrorIdentity::Ambiguous(candidates) => {
            let mut resolved = Vec::with_capacity(candidates.len());
            // Bounded by `candidates.len()` (Power-of-10 rule 2).
            for candidate in candidates {
                match locate(candidate, forest, spans) {
                    LocateOutcome::Unique(span) => resolved.push(span),
                    LocateOutcome::Ambiguous(spans) => resolved.extend(spans),
                    LocateOutcome::Absent => {}
                }
            }
            finish(resolved)
        }
    }
}

/// Collapse a candidate-span list into the three-outcome shape, sorted
/// into document order.
fn finish(mut spans: Vec<Span>) -> LocateOutcome {
    spans.sort_by_key(|s| (s.start, s.end));
    match spans.len() {
        0 => LocateOutcome::Absent,
        1 => LocateOutcome::Unique(spans[0]),
        _ => LocateOutcome::Ambiguous(spans),
    }
}

/// The textual content of an atom a locator strategy can search by text —
/// `None` for atoms with no name-shaped text (`Int`, `Bool`, `Currency`,
/// `Mass`, `Scaled`, `Str`, `Operator`, `EnumRef` — the last has its own dedicated
/// [`by_enum_ref`] search, since it carries TWO components, not one).
/// Reads the atom's own typed field, never `Display`/message text
/// (sentinel 7.2).
fn atom_text(atom: &Atom) -> Option<&str> {
    match atom {
        Atom::Symbol(s) | Atom::QName(s) | Atom::Keyword(s) | Atom::BareUpperIdent(s) => Some(s),
        Atom::EnumRef { .. }
        | Atom::Bool(_)
        | Atom::Operator(_)
        | Atom::Int(_)
        | Atom::Currency(_)
        | Atom::Mass(_)
        | Atom::Scaled(_)
        | Atom::Str(_) => None,
    }
}

/// Walk `forest` in pre-order, calling `visit` with each node's `FormPath`
/// and the node itself. Recursion terminates by construction: the reader
/// can never produce a cyclic `SExpr` (the same finite-tree argument
/// `canonical_ast::encode_node` and `bound_checker`'s own module doc
/// already rely on for the identical data structure) — structural
/// recursion over a provably finite tree, not an unbounded loop.
fn walk<'a>(forest: &'a [SExpr], visit: &mut dyn FnMut(&[u32], &'a SExpr)) {
    // Bounded by `forest.len()` (Power-of-10 rule 2) at this level; each
    // recursive descent is bounded by that node's own finite depth.
    for (index, node) in forest.iter().enumerate() {
        let path = [index_to_u32(index)];
        walk_node(&path, node, visit);
    }
}

fn walk_node<'a>(path: &[u32], node: &'a SExpr, visit: &mut dyn FnMut(&[u32], &'a SExpr)) {
    visit(path, node);
    if let SExpr::List(children) = node {
        for (index, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(index_to_u32(index));
            walk_node(&child_path, child, visit);
        }
    }
}

/// `FormPath` components are `u32`; a form with more than 4 billion
/// siblings is not a real document (the whole content estate is ~440 KB
/// total) — saturate rather than panic on the never-taken branch.
fn index_to_u32(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// `by_atom(token)` (§6.2): every atom anywhere in the forest whose text
/// equals `token` exactly. Serves [`ErrorIdentity::Name`],
/// [`ErrorIdentity::NodeLocal`], [`ErrorIdentity::RuleId`].
#[must_use]
pub fn by_atom(token: &str, forest: &[SExpr], spans: &SpanTable) -> LocateOutcome {
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        if let SExpr::Atom(atom) = node {
            if atom_text(atom) == Some(token) {
                if let Some(span) = spans.span_of(path) {
                    found.push(span);
                }
            }
        }
    });
    finish(found)
}

/// `by_qname(qname)` (§6.2): every `QName` atom anywhere in the forest
/// whose text equals `qname` exactly. Serves [`ErrorIdentity::Field`].
/// Kept as its own named strategy (distinct from [`by_atom`]) per §6.2's
/// four-strategy roster, even though the two share an implementation
/// shape — `by_qname` narrows the atom KIND to `QName`, since a `Field`
/// identity's value is always qname-shaped text (`social-class/wages`),
/// where `by_atom`'s callers (`Name`/`NodeLocal`/`RuleId`) may be either
/// `Symbol` or `QName` atoms.
#[must_use]
pub fn by_qname(qname: &str, forest: &[SExpr], spans: &SpanTable) -> LocateOutcome {
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        if let SExpr::Atom(Atom::QName(text)) = node {
            if text == qname {
                if let Some(span) = spans.span_of(path) {
                    found.push(span);
                }
            }
        }
    });
    finish(found)
}

/// The enum-ref half of `by_head_and_operand` (§6.2): every `EnumRef`
/// atom whose `enum_type` equals `enum_type` and, when `member` is given,
/// whose `member` also matches. Serves [`ErrorIdentity::Enum`] — a
/// written enum-ref is lexed as one `EnumRef` atom regardless of whether
/// its member is registered (registry membership is a load-time check,
/// never a lexical one), so an *unknown* member's own offending written
/// occurrence is still findable this way.
#[must_use]
pub fn by_enum_ref(
    enum_type: &str,
    member: Option<&str>,
    forest: &[SExpr],
    spans: &SpanTable,
) -> LocateOutcome {
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        if let SExpr::Atom(Atom::EnumRef {
            enum_type: et,
            member: m,
        }) = node
        {
            let type_matches = et == enum_type;
            let member_matches = member.is_none_or(|want| m == want);
            if type_matches && member_matches {
                if let Some(span) = spans.span_of(path) {
                    found.push(span);
                }
            }
        }
    });
    finish(found)
}

/// `by_head_and_operand(head, operands)` (§6.2): every `List` whose first
/// element is `Atom::Symbol(head)` and which contains, among its DIRECT
/// children, an atom whose text equals every one of `operands` (order
/// unconstrained — [`ErrorIdentity::Edge`]'s `edge_type`/`from` need not
/// appear adjacent in a written `(edge EdgeType/X from to)` form). Serves
/// [`ErrorIdentity::Edge`] (see this module's own doc: untested against a
/// real producer today). The matched LIST's own span is the candidate —
/// not a sub-span of one operand — since the identity names the FORM the
/// error is about, not one token inside it.
#[must_use]
pub fn by_head_and_operand(
    head: &str,
    operands: &[&str],
    forest: &[SExpr],
    spans: &SpanTable,
) -> LocateOutcome {
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        let SExpr::List(children) = node else {
            return;
        };
        let Some(SExpr::Atom(head_atom)) = children.first() else {
            return;
        };
        if atom_text(head_atom) != Some(head) {
            return;
        }
        let child_texts: HashSet<String> = children
            .iter()
            .filter_map(|c| match c {
                // An `EnumRef` operand's comparable text is its
                // reconstructed `EnumType/MEMBER` form — an `ErrorIdentity
                // ::Edge`'s `edge_type` is written exactly that way in
                // source (`(edge EdgeType/SOLIDARITY a b)`), lexed as ONE
                // `EnumRef` atom rather than a `Symbol`/`QName` whose text
                // `atom_text` alone can read.
                SExpr::Atom(Atom::EnumRef { enum_type, member }) => {
                    Some(format!("{enum_type}/{member}"))
                }
                SExpr::Atom(a) => atom_text(a).map(str::to_owned),
                SExpr::List(_) => None,
            })
            .collect();
        if operands.iter().all(|want| child_texts.contains(*want)) {
            if let Some(span) = spans.span_of(path) {
                found.push(span);
            }
        }
    });
    finish(found)
}

/// `by_keyword_operand(keyword)` (§6.2): every `Keyword` atom anywhere in
/// the forest whose text equals `keyword` (a leading `:`, if present in
/// `keyword`, is stripped before comparing — `ErrorIdentity::Keyword`'s
/// producers are inconsistent about carrying it, e.g.
/// `SurfaceError::EmptyMaterialBasis` writes `":material-basis"` while a
/// reader-derived `BindingError::UnknownKeyword{keyword}` carries the
/// reader's own colon-less `Atom::Keyword` text verbatim) — the candidate
/// span is the OPERAND immediately following the keyword in its own
/// list, when one exists; a keyword with no following operand (the last
/// element of its list) contributes no candidate from that occurrence,
/// since there is no operand span to point at. Serves
/// [`ErrorIdentity::Keyword`].
#[must_use]
pub fn by_keyword_operand(keyword: &str, forest: &[SExpr], spans: &SpanTable) -> LocateOutcome {
    let wanted = keyword.strip_prefix(':').unwrap_or(keyword);
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        let SExpr::List(children) = node else {
            return;
        };
        // Bounded by `children.len()` (Power-of-10 rule 2).
        for (index, child) in children.iter().enumerate() {
            let SExpr::Atom(Atom::Keyword(text)) = child else {
                continue;
            };
            if text != wanted {
                continue;
            }
            if children.len() > index + 1 {
                let mut operand_path = path.to_vec();
                operand_path.push(index_to_u32(index + 1));
                if let Some(span) = spans.span_of(&operand_path) {
                    found.push(span);
                }
            }
        }
    });
    finish(found)
}

/// `by_operand_index(form, index)` (§6.2): every `List` whose first
/// element's text equals `form` (a `Symbol` head like `neighbors`, or an
/// `Operator` head like `+` — `GrammarError::ArithmeticArity`'s `form`
/// values are the ten operator tokens) and whose child at position
/// `index + 1` (position `0` is the head itself) exists — that child's
/// own span is the candidate. A list too short to have that position
/// contributes no candidate from that occurrence: an arity error naming
/// an index past what the offending form actually wrote has nothing
/// there to underline, and file level (via [`LocateOutcome::Absent`]) is
/// the honest fallback rather than a guessed span. Serves
/// [`ErrorIdentity::Operand`].
#[must_use]
pub fn by_operand_index(
    form: &str,
    index: usize,
    forest: &[SExpr],
    spans: &SpanTable,
) -> LocateOutcome {
    let mut found = Vec::new();
    walk(forest, &mut |path, node| {
        let SExpr::List(children) = node else {
            return;
        };
        let Some(head) = children.first() else {
            return;
        };
        let head_text = match head {
            SExpr::Atom(Atom::Symbol(s) | Atom::Operator(s)) => Some(s.as_str()),
            _ => None,
        };
        if head_text != Some(form) {
            return;
        }
        let operand_position = index + 1;
        if children.len() > operand_position {
            let mut operand_path = path.to_vec();
            operand_path.push(index_to_u32(operand_position));
            if let Some(span) = spans.span_of(&operand_path) {
                found.push(span);
            }
        }
    });
    finish(found)
}

#[cfg(test)]
mod tests {
    use super::{
        by_atom, by_enum_ref, by_head_and_operand, by_keyword_operand, by_operand_index, by_qname,
        locate, LocateOutcome,
    };
    use babylon_bsl::{read_all_spanned, ErrorIdentity};

    fn parse(source: &str) -> (Vec<babylon_bsl::SExpr>, babylon_bsl::SpanTable) {
        read_all_spanned(source.as_bytes()).expect("fixture source must parse")
    }

    #[test]
    fn by_atom_unique_finds_the_one_matching_symbol() {
        let source = "(rule foo (bindings) (effects))";
        let (forest, spans) = parse(source);
        match by_atom("foo", &forest, &spans) {
            LocateOutcome::Unique(span) => {
                assert_eq!(&source[span.start..span.end], "foo");
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn by_atom_ambiguous_when_the_token_occurs_twice_sorted_by_position() {
        let (forest, spans) = parse("(intrinsic floor :cost 5) (intrinsic floor :cost 6)");
        match by_atom("floor", &forest, &spans) {
            LocateOutcome::Ambiguous(candidates) => {
                assert_eq!(candidates.len(), 2);
                assert!(candidates[0].start < candidates[1].start);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn by_atom_absent_when_the_token_never_occurs() {
        let (forest, spans) = parse("(rule foo (bindings) (effects))");
        assert_eq!(by_atom("nowhere", &forest, &spans), LocateOutcome::Absent);
    }

    #[test]
    fn by_qname_finds_only_qname_atoms_not_bare_symbols() {
        let (forest, spans) = parse("(deffield social-class/wages :type int) (rule wages)");
        match by_qname("social-class/wages", &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
        // "wages" (a bare symbol, not a qname) never matches a by_qname
        // search for the qname text.
        assert_eq!(by_qname("wages", &forest, &spans), LocateOutcome::Absent);
    }

    #[test]
    fn by_enum_ref_matches_type_and_member() {
        let (forest, spans) = parse("(the NodeType/SOCIAL_CLASS)");
        match by_enum_ref("NodeType", Some("SOCIAL_CLASS"), &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
        assert_eq!(
            by_enum_ref("NodeType", Some("TERRITORY"), &forest, &spans),
            LocateOutcome::Absent
        );
    }

    #[test]
    fn by_enum_ref_with_no_member_matches_any_member_of_the_type() {
        let (forest, spans) = parse(
            "(add-hyperedge HyperedgeType/A (members)) (add-hyperedge HyperedgeType/A (members))",
        );
        match by_enum_ref("HyperedgeType", None, &forest, &spans) {
            LocateOutcome::Ambiguous(candidates) => assert_eq!(candidates.len(), 2),
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn by_head_and_operand_finds_the_matching_list_by_head_and_contained_operand() {
        let (forest, spans) = parse("(edge EdgeType/SOLIDARITY alice bob)");
        match by_head_and_operand("edge", &["EdgeType/SOLIDARITY", "alice"], &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
        assert_eq!(
            by_head_and_operand("edge", &["EdgeType/TRIBUTE"], &forest, &spans),
            LocateOutcome::Absent
        );
    }

    #[test]
    fn by_keyword_operand_finds_the_operand_after_the_keyword() {
        let (forest, spans) = parse("(defconst economy/x 5$ :floor 0$)");
        match by_keyword_operand(":floor", &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
        // Colon-less form of the same keyword text resolves identically.
        match by_keyword_operand("floor", &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn by_keyword_operand_absent_when_the_keyword_never_occurs() {
        let (forest, spans) = parse("(defconst economy/x 5$)");
        assert_eq!(
            by_keyword_operand(":cap", &forest, &spans),
            LocateOutcome::Absent
        );
    }

    #[test]
    fn by_operand_index_finds_the_operand_at_that_position() {
        let (forest, spans) = parse("(neighbors self EdgeType/SOLIDARITY)");
        match by_operand_index("neighbors", 1, &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn by_operand_index_absent_when_the_position_does_not_exist() {
        let (forest, spans) = parse("(neighbors self)");
        assert_eq!(
            by_operand_index("neighbors", 5, &forest, &spans),
            LocateOutcome::Absent
        );
    }

    #[test]
    fn locate_dispatches_name_identity_through_by_atom() {
        let (forest, spans) = parse("(rule foo (bindings) (effects))");
        let identity = ErrorIdentity::Name("foo".to_owned());
        match locate(&identity, &forest, &spans) {
            LocateOutcome::Unique(_) => {}
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn locate_dispatches_ambiguous_identity_by_unioning_each_candidates_own_locate() {
        let (forest, spans) = parse("(a) (b)");
        let identity = ErrorIdentity::Ambiguous(vec![
            ErrorIdentity::Name("a".to_owned()),
            ErrorIdentity::Name("b".to_owned()),
        ]);
        match locate(&identity, &forest, &spans) {
            LocateOutcome::Ambiguous(candidates) => assert_eq!(candidates.len(), 2),
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn locate_dispatches_ambiguous_identity_absent_when_no_candidate_resolves() {
        let (forest, spans) = parse("(rule foo (bindings) (effects))");
        let identity = ErrorIdentity::Ambiguous(vec![ErrorIdentity::Name("nowhere".to_owned())]);
        assert_eq!(locate(&identity, &forest, &spans), LocateOutcome::Absent);
    }
}
