//! Canonical AST serialization — CAS (`bsl-language.rst` §5): the byte
//! layout `rules_hash` hashes. Whitespace-, comment-, file-layout- and
//! option-order-insensitive: a formatting edit produces identical bytes,
//! while any change to a rule's meaning — including its `:material-basis`
//! string and its `:fuel` budget — produces different bytes.
//!
//! **Deviation from the Phase 1 plan's sketch, recorded:** the sketch
//! rendered atoms as text with `0x00` separators and insertion-order
//! hashing. The normative §5 defines a BINARY tagged encoding (§5.1–5.2),
//! canonical child reordering with `opt`-wrapped keyword options (§5.3),
//! and `rules_hash = SHA-256(0x03 ‖ u32 N ‖ CAS(r_1) ‖ …)` over rules
//! **sorted by rule id** (§5.5) — implemented as written, and proved
//! against §5.6's own pinned worked example (500 bytes, both digests).
//!
//! Per §5.4 there are no stringify fallbacks: an unencodable node (an
//! operator outside form-head position, a form whose head is not a
//! symbol/operator, an empty form) fails loudly.

use crate::reader::{Atom, SExpr, ScaledKind};
use sha2::{Digest, Sha256};

/// A loud CAS failure (§5.4 bans silent fallbacks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasError {
    /// Human-readable detail.
    pub message: String,
}

/// The canonical bytes of one node (§5.1–§5.3).
///
/// # Errors
/// [`CasError`] on any unencodable node — never a stringified fallback.
pub fn canonical_bytes(expr: &SExpr) -> Result<Vec<u8>, CasError> {
    let mut out = Vec::new();
    encode_node(expr, &mut out)?;
    Ok(out)
}

/// `rules_hash = SHA-256(0x03 ‖ u32 N ‖ CAS(r_1) ‖ … ‖ CAS(r_N))` with
/// the rule forms sorted by rule id in ascending ASCII byte order (§5.5).
/// Duplicate rule ids are a LOAD error (`E-LOAD-001`), not CAS's concern;
/// with duplicates present the sort is still deterministic (stable).
///
/// # Errors
/// [`CasError`] if any form is not a `rule` form with a qname id, or any
/// node is unencodable.
pub fn rules_hash_of(rules: &[SExpr]) -> Result<[u8; 32], CasError> {
    let mut ids: Vec<(&str, &SExpr)> = Vec::with_capacity(rules.len());
    for rule in rules {
        ids.push((rule_id(rule)?, rule));
    }
    ids.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut hasher = Sha256::new();
    hasher.update([0x03]);
    let count = u32::try_from(ids.len()).map_err(|_| CasError {
        message: "more than u32::MAX rules".into(),
    })?;
    hasher.update(count.to_be_bytes());
    for (_, rule) in ids {
        hasher.update(canonical_bytes(rule)?);
    }
    Ok(hasher.finalize().into())
}

/// A `rule` form's id: the qname immediately after the head (§2.3).
///
/// `pub(crate)` (Program 28 B2, Phase A Task 2) so `rule_pipeline::
/// split_content`'s multi-rule duplicate-id check reuses this strict
/// extractor instead of writing a third one — `babylon-bsl` already carried
/// two (`canonical_ast::rule_id` here; `bound_checker::rule_id`, a lenient
/// error-reporting helper that never fails). Body unchanged.
pub(crate) fn rule_id(expr: &SExpr) -> Result<&str, CasError> {
    let SExpr::List(items) = expr else {
        return Err(CasError {
            message: "a rule must be a form".into(),
        });
    };
    match items.as_slice() {
        [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::QName(id)), ..] if head == "rule" => {
            Ok(id)
        }
        _ => Err(CasError {
            message: "rules_hash input must be (rule <qname-id> ...) forms".into(),
        }),
    }
}

fn encode_node(expr: &SExpr, out: &mut Vec<u8>) -> Result<(), CasError> {
    match expr {
        SExpr::Atom(atom) => encode_atom(atom, out),
        SExpr::List(items) => encode_form(items, out),
    }
}

/// `atom ::= 0x01  u8 len(kind)  kind_ascii  u32 len(payload)  payload`
/// with the §5.2 kind/payload table.
fn encode_atom(atom: &Atom, out: &mut Vec<u8>) -> Result<(), CasError> {
    let (kind, payload): (&str, Vec<u8>) = match atom {
        Atom::Int(v) => ("int", v.to_be_bytes().to_vec()),
        Atom::Currency(c) => ("cur", c.micro_units().to_be_bytes().to_vec()),
        Atom::Mass(mass) => ("mass", mass.nanounits().to_be_bytes().to_vec()),
        Atom::Scaled(s) => {
            let kind = match s.kind {
                ScaledKind::Probability => "prob",
                ScaledKind::Intensity => "intn",
                ScaledKind::Coefficient => "coef",
                // §1.5 addendum (#492/ADR194): additive — no existing kind
                // tag is reused, so no prior canonical bytes move.
                ScaledKind::Ratio => "ratio",
            };
            let mut payload = s.unscaled.to_be_bytes().to_vec();
            payload.push(s.scale);
            (kind, payload)
        }
        Atom::Bool(b) => ("bool", vec![u8::from(*b)]),
        Atom::Symbol(s) => ("sym", s.clone().into_bytes()),
        Atom::QName(q) => ("qname", q.clone().into_bytes()),
        Atom::Keyword(k) => ("kw", k.clone().into_bytes()),
        Atom::EnumRef { enum_type, member } => {
            ("enum", format!("{enum_type}/{member}").into_bytes())
        }
        // §2.13 (Organization contract, Q12): reuses the EXISTING "enum"
        // CAS kind tag rather than minting a new one (bsl-language.rst's
        // own §5.2 note: "needing... no new atom kind" — D117 corrects
        // that note's premise for THIS payload shape, which is not an
        // `<enum-ref>` value; the kind tag itself still doesn't move). No
        // collision with an `EnumRef` payload is possible — that payload
        // always contains exactly one `/`, this one never does (D117).
        Atom::BareUpperIdent(name) => ("enum", name.clone().into_bytes()),
        Atom::Str(s) => ("str", s.clone().into_bytes()),
        Atom::Operator(op) => {
            return Err(CasError {
                message: format!(
                    "operator '{op}' outside form-head position is unencodable (§5.4)"
                ),
            })
        }
    };
    write_atom(kind, &payload, out)
}

fn write_atom(kind: &str, payload: &[u8], out: &mut Vec<u8>) -> Result<(), CasError> {
    out.push(0x01);
    write_len_prefixed(kind.as_bytes(), payload, out)
}

fn write_len_prefixed(tag: &[u8], payload: &[u8], out: &mut Vec<u8>) -> Result<(), CasError> {
    let tag_len = u8::try_from(tag.len()).map_err(|_| CasError {
        message: "tag/kind name over 255 bytes".into(),
    })?;
    let payload_len = u32::try_from(payload.len()).map_err(|_| CasError {
        message: "payload over u32::MAX bytes".into(),
    })?;
    out.push(tag_len);
    out.extend_from_slice(tag);
    out.extend_from_slice(&payload_len.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(())
}

/// The **closed** set of flag keywords (§1.6's table: the rows whose
/// operand column reads *flag*). A flag takes no operand and encodes as
/// `opt(kw, atom bool 0x01)` under D20; every other keyword consumes the
/// child that follows it.
///
/// **This list replaces an adjacency heuristic, and the replacement is
/// load-bearing.** The encoder used to read "a keyword followed by a
/// non-keyword" as a valued option, which was safe only while every flag
/// sat at its form's end or before another keyword. D51's `neighbors`
/// breaks that: `<query> ::= "(" "neighbors" <expr> <enum-ref> <direction>
/// <enum-ref> ")"` puts a **mandatory positional** `<enum-ref>` *after* the
/// direction flag, so the heuristic swallowed the result `NodeType` as
/// `:out`'s value — three children where §5.3 group 1 requires four, and a
/// `rules_hash` a second implementation derived from the document alone
/// would not reproduce. The set is closed for the same reason §1.6's
/// keyword set is: guessing from adjacency is not derivable from the spec.
const FLAG_KEYWORDS: [&str; 9] = [
    "any",
    "graph",
    "in",
    "invariant",
    "optional",
    "out",
    "tick",
    "tick-of-year",
    "year",
];

/// The count of **fixed positional operands** (§5.3 group 1) for the form
/// tags whose §2 production puts a positional operand *after* a keyword.
///
/// For every other tag the boundary is unambiguous from the source — group
/// 1 is what precedes the first keyword — so this returns `None` and the
/// encoder keeps that reading, which is what leaves §5.6's pinned bytes and
/// every pre-R9 form byte-identical.
///
/// Two productions interleave that way today:
///
/// - `neighbors` (D51): `"(" "neighbors" <expr> <enum-ref> <direction>
///   <enum-ref> ")"` — a mandatory result `NodeType` *after* the direction
///   flag, so 4;
/// - `metric` (§2.11): `"(" "metric" <symbol> ":type" <type-name> ":kind"
///   (…) <domain> ":provider" <symbol> ")"` — the positional `<domain>`
///   form sits *after* `:type`/`:kind`, so 2 (the name and the domain).
///   §5.5 hashes `metric` forms into their own digest, so this one is a
///   hash surface exactly as `rules_hash` is.
///
/// A future form that interleaves the same way adds a row here rather than
/// a new heuristic.
fn fixed_positionals(tag: &str) -> Option<usize> {
    match tag {
        "neighbors" => Some(4),
        "metric" => Some(2),
        "choose" => Some(0),
        "branch" => Some(1),
        _ => None,
    }
}

/// One reordered child: a positional/body node, or an `opt`-wrapped
/// keyword option (a flag keyword wraps `#t`, per the §5.2 draft ruling).
enum Child<'a> {
    Node(&'a SExpr),
    Opt {
        name: &'a str,
        value: Option<&'a SExpr>,
    },
}

/// `form ::= 0x02  u8 len(tag)  tag_ascii  u32 nchildren  child*` with
/// §5.3's canonical child order: positional operands (source order), then
/// keyword options sorted by name in ascending ASCII byte order, then the
/// variadic body (source order). Generically: non-option children keep
/// source order; the sorted option block sits where the first option
/// appeared. Whether a keyword is a flag or takes an operand is read from
/// [`FLAG_KEYWORDS`] — §1.6's closed table — never from adjacency.
fn encode_form(items: &[SExpr], out: &mut Vec<u8>) -> Result<(), CasError> {
    let Some((head, rest)) = items.split_first() else {
        return Err(CasError {
            message: "an empty form () has no tag (§5.2)".into(),
        });
    };
    let tag = match head {
        SExpr::Atom(Atom::Symbol(s)) => s.as_str(),
        SExpr::Atom(Atom::Operator(op)) => op.as_str(),
        _ => {
            return Err(CasError {
                message: "a form's head must be a symbol or operator (§5.2)".into(),
            })
        }
    };
    let mut non_options: Vec<&SExpr> = Vec::new();
    let mut options: Vec<(&str, Option<&SExpr>)> = Vec::new();
    // How many non-option children preceded the FIRST option — the fallback
    // group-1 boundary for a tag with no declared fixed arity.
    let mut before_first_option: Option<usize> = None;
    let mut i = 0;
    while i < rest.len() {
        if let SExpr::Atom(Atom::Keyword(name)) = &rest[i] {
            if before_first_option.is_none() {
                before_first_option = Some(non_options.len());
            }
            // Valued-ness comes from §1.6's closed table, NOT from what
            // happens to follow — see FLAG_KEYWORDS.
            if FLAG_KEYWORDS.contains(&name.as_str()) {
                options.push((name, None));
                i += 1;
            } else {
                let Some(value) = rest.get(i + 1) else {
                    return Err(CasError {
                        message: format!(
                            "keyword :{name} takes an operand (§1.6) but ends its \
                             form; an unencodable form fails loudly rather than \
                             stringifying (§5.4)"
                        ),
                    });
                };
                options.push((name, Some(value)));
                i += 2;
            }
        } else {
            non_options.push(&rest[i]);
            i += 1;
        }
    }
    options.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    // §5.3's group boundary: the first `split` non-option children are the
    // form's fixed positional operands (group 1), the rest are its variadic
    // body (group 3), and the sorted option block sits between them.
    let split = fixed_positionals(tag)
        .unwrap_or_else(|| before_first_option.unwrap_or(non_options.len()))
        .min(non_options.len());
    let (positionals_before, body_after) = non_options.split_at(split);

    out.push(0x02);
    let n_children = positionals_before.len() + options.len() + body_after.len();
    let count = u32::try_from(n_children).map_err(|_| CasError {
        message: "form has more than u32::MAX children".into(),
    })?;
    let tag_len = u8::try_from(tag.len()).map_err(|_| CasError {
        message: "form tag over 255 bytes".into(),
    })?;
    out.push(tag_len);
    out.extend_from_slice(tag.as_bytes());
    out.extend_from_slice(&count.to_be_bytes());

    let children = positionals_before
        .iter()
        .copied()
        .map(Child::Node)
        .chain(
            options
                .into_iter()
                .map(|(name, value)| Child::Opt { name, value }),
        )
        .chain(body_after.iter().copied().map(Child::Node));
    for child in children {
        match child {
            Child::Node(node) => encode_node(node, out)?,
            Child::Opt { name, value } => encode_opt(name, value, out)?,
        }
    }
    Ok(())
}

/// `opt ::= form("opt", atom("kw", <name>), <value node>)`; a flag keyword
/// encodes `atom("bool", 0x01)` as its value.
fn encode_opt(name: &str, value: Option<&SExpr>, out: &mut Vec<u8>) -> Result<(), CasError> {
    out.push(0x02);
    out.push(3); // len("opt")
    out.extend_from_slice(b"opt");
    out.extend_from_slice(&2u32.to_be_bytes());
    write_atom("kw", name.as_bytes(), out)?;
    match value {
        Some(node) => encode_node(node, out),
        None => write_atom("bool", &[0x01], out),
    }
}

#[cfg(test)]
mod tests {
    use super::{canonical_bytes, rules_hash_of};
    use crate::reader::{read, read_all};

    /// The §5.6 worked-example source, comment included (comments are
    /// whitespace and must not affect the bytes).
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

    /// §5.6's pinned canonical bytes — 500 bytes, reproduced verbatim from
    /// the spec (which states both digests are "reproducible from this
    /// document alone").
    const PINNED_HEX: &str = concat!(
        "020472756c65000000080105716e616d650000000b64656d6f2f68756e676572",
        "02036f70740000000201026b770000000865766964656e6365010373796d0000",
        "00076465726976656402036f70740000000201026b77000000046675656c0103",
        "696e7400000008000000000000004002036f70740000000201026b770000000e",
        "6d6174657269616c2d626173697301037374720000003073756273697374656e",
        "636520646566696369742061742074686520706f696e74206f6620726570726f",
        "64756374696f6e02036f70740000000201026b7700000004726f6c6501037379",
        "6d000000086d656368616e6963020862696e64696e677300000001020762696e",
        "64696e6700000002010373796d000000067765616c746802036f707400000002",
        "01026b77000000056669656c640105716e616d6500000013736f6369616c2d63",
        "6c6173732f7765616c746802047768656e0000000102013c0000000201037379",
        "6d000000067765616c7468010363757200000010000000000000000000000000",
        "3ba26b2002076566666563747300000001020b7570646174652d6e6f64650000",
        "0003010373796d0000000473656c660105716e616d6500000016736f6369616c",
        "2d636c6173732f616769746174696f6e0203616464000000010104696e746e00",
        "0000110000000000000000000000000000000502"
    );

    fn demo_rule() -> crate::reader::SExpr {
        read(DEMO_RULE).expect("worked example must parse").0
    }

    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write as _;
        bytes.iter().fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    #[test]
    fn the_spec_worked_example_reproduces_byte_for_byte() {
        let bytes = canonical_bytes(&demo_rule()).unwrap();
        assert_eq!(bytes.len(), 500, "§5.6 pins exactly 500 canonical bytes");
        assert_eq!(hex(&bytes), PINNED_HEX);
    }

    #[test]
    fn the_spec_worked_example_digests_reproduce() {
        use sha2::{Digest, Sha256};
        let bytes = canonical_bytes(&demo_rule()).unwrap();
        let single = Sha256::digest(&bytes);
        assert_eq!(
            hex(&single),
            "9b49c1e87a63ccb64dee96bd179c8ffecd3fe476c81a4f81771daac237156396"
        );
        let rules_hash = rules_hash_of(&[demo_rule()]).unwrap();
        assert_eq!(
            hex(&rules_hash),
            "bf2f15517d1b4d10e44af36983c76249b49d8fa2dc71e6d8c1d2607b37b2f20f"
        );
    }

    #[test]
    fn formatting_and_comments_never_change_the_bytes() {
        let reformatted = "(rule demo/hunger :role mechanic :evidence derived :material-basis \
         \"subsistence deficit at the point of reproduction\" :fuel 64 \
         (bindings (binding wealth :field social-class/wealth)) \
         (when (< wealth 1000.5$)) ; inline comment\n \
         (effects (update-node self social-class/agitation (add 0.05i))))";
        let (expr, _) = read(reformatted).unwrap();
        assert_eq!(
            canonical_bytes(&expr).unwrap(),
            canonical_bytes(&demo_rule()).unwrap()
        );
    }

    #[test]
    fn option_order_is_a_formatting_concern() {
        // §5.6's own note: :fuel sorts before :material-basis regardless of
        // source order ("fuel" < "material-basis" in ASCII).
        let swapped = DEMO_RULE.replacen(
            ":material-basis \"subsistence deficit at the point of reproduction\"\n     :fuel 64",
            ":fuel 64\n     :material-basis \"subsistence deficit at the point of reproduction\"",
            1,
        );
        assert_ne!(
            swapped, DEMO_RULE,
            "the swap must actually rewrite the source"
        );
        let (expr, _) = read(&swapped).unwrap();
        assert_eq!(
            canonical_bytes(&expr).unwrap(),
            canonical_bytes(&demo_rule()).unwrap()
        );
    }

    /// CT4P A6 (issue #525): the DUAL of `option_order_is_a_formatting_
    /// concern` above. Canonicalisation sorts KEYWORD OPTIONS only
    /// (`options.sort_by`, `:283`) — positional operands and the variadic
    /// body keep SOURCE order, on purpose: `evaluator.rs`'s `and`/`or`
    /// short-circuit left-to-right, and short-circuiting charges strictly
    /// LESS fuel when the deciding operand is early
    /// (`exists_and_forall_short_circuit_and_charge_less_fuel_when_
    /// element_one_decides`, `evaluator.rs`). So `(and a b)` and
    /// `(and b a)` agree on VALUE but disagree on FUEL — and fuel is a
    /// load-time admissibility criterion (`E-LOAD-040`, invariant S-3,
    /// `ai/bsl-architecture-standard.md:629`). This test asserts the two
    /// forms canonicalise to DIFFERENT bytes: canonicalisation stops at
    /// keyword options precisely because positional order carries
    /// short-circuit and fuel semantics. A future "normalise boolean
    /// operand order" optimisation is therefore a `rules_hash` change AND
    /// a fuel change — never a formatting edit.
    #[test]
    fn positional_operand_order_is_not_a_formatting_concern() {
        let (ab, _) = read("(and a b)").unwrap();
        let (ba, _) = read("(and b a)").unwrap();
        assert_ne!(
            canonical_bytes(&ab).unwrap(),
            canonical_bytes(&ba).unwrap(),
            "(and a b) and (and b a) must canonicalise to DIFFERENT bytes — \
             positional order is data, not formatting"
        );
    }

    /// §1.5 addendum (#492/ADR194): a `Ratio` literal encodes with its own
    /// `"ratio"` kind tag — a purely additive change, proved by pinning the
    /// §5.6 worked example's byte count unmoved (it declares no `r`
    /// literal) before asserting the new atom encodes at all.
    #[test]
    fn a_ratio_literal_encodes_with_its_own_kind_tag_additively() {
        assert_eq!(
            canonical_bytes(&demo_rule()).unwrap().len(),
            500,
            "the golden program must be unmoved before this row is trusted"
        );
        let bytes = canonical_bytes(&read("2.5r").unwrap().0).unwrap();
        let mut expected = Vec::new();
        push_atom(&mut expected, b"ratio", &{
            let mut payload = 25i128.to_be_bytes().to_vec();
            payload.push(1); // scale
            payload
        });
        assert_eq!(bytes, expected);
    }

    #[test]
    fn ratio_literal_canonicalization_reaches_the_bytes() {
        // §1.5: 2.50r ≡ 2.5r — identical bytes, same law as p/i/c/$.
        let a = read("(x 2.50r)").unwrap().0;
        let b = read("(x 2.5r)").unwrap().0;
        assert_eq!(canonical_bytes(&a).unwrap(), canonical_bytes(&b).unwrap());
    }

    #[test]
    fn literal_canonicalization_reaches_the_bytes() {
        // §5.4: 0.50c ≡ 0.5c and 1000.5$ ≡ 1000.500$ — identical bytes.
        let a = read("(x 0.50c 1000.5$)").unwrap().0;
        let b = read("(x 0.5c 1000.500$)").unwrap().0;
        assert_eq!(canonical_bytes(&a).unwrap(), canonical_bytes(&b).unwrap());
    }

    /// D20: a flag keyword encodes as `form("opt", atom kw <name>,
    /// atom bool 0x01)` so every option has one shape.
    ///
    /// The expectation is **assembled by hand from §5.1–§5.2**, not
    /// obtained by encoding some other source. The previous spelling of
    /// this test compared `(binding x :optional)` against
    /// `(binding x :optional #t)` — but `:optional` takes no operand
    /// (§1.6), so the second is not BSL at all, and the comparison silently
    /// asserted that the encoder consumes whatever follows a flag. That is
    /// exactly the adjacency heuristic `FLAG_KEYWORDS` removed.
    #[test]
    fn a_flag_keyword_encodes_as_a_bool_true_option() {
        let bytes = canonical_bytes(&read("(binding x :optional)").unwrap().0).unwrap();
        let mut expected = Vec::new();
        push_form(&mut expected, b"binding", 2);
        push_atom(&mut expected, b"sym", b"x");
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"optional");
        push_atom(&mut expected, b"bool", &[0x01]);
        assert_eq!(bytes, expected);
    }

    /// The flag/valued split is read from §1.6's closed table, so a
    /// mandatory positional operand AFTER a flag survives: D51's
    /// `neighbors` is four children, not three.
    #[test]
    fn a_positional_operand_after_a_flag_is_not_swallowed() {
        let bytes = canonical_bytes(
            &read("(neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)")
                .unwrap()
                .0,
        )
        .unwrap();
        let mut expected = Vec::new();
        push_form(&mut expected, b"neighbors", 4);
        push_atom(&mut expected, b"sym", b"self");
        push_atom(&mut expected, b"enum", b"EdgeType/SOLIDARITY");
        push_atom(&mut expected, b"enum", b"NodeType/SOCIAL_CLASS");
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"out");
        push_atom(&mut expected, b"bool", &[0x01]);
        assert_eq!(bytes, expected);
    }

    /// A non-flag keyword that ends its form is unencodable and fails
    /// loudly (§5.4 bans stringify fallbacks) — the encoder no longer
    /// quietly re-reads it as a flag.
    #[test]
    fn a_valued_keyword_with_no_operand_is_a_loud_cas_error() {
        assert!(canonical_bytes(&read("(binding x :field)").unwrap().0).is_err());
    }

    /// §2.11's `metric` form places its positional `<domain>` AFTER the
    /// `:type`/`:kind` options, so it needs the same §5.3 group-1 boundary
    /// `neighbors` does — and §5.5 hashes `metric` forms, so getting it
    /// wrong is a hash divergence, not a cosmetic one.
    ///
    /// **Guarded**: §5.6's worked example declares no metric, so adding the
    /// row cannot move its bytes. That is asserted first rather than
    /// assumed — if it ever failed, the row would be the thing to remove.
    #[test]
    fn a_metric_declarations_positional_domain_precedes_its_options() {
        assert_eq!(
            canonical_bytes(&demo_rule()).unwrap().len(),
            500,
            "the golden program must be unmoved before this row is trusted"
        );
        let bytes = canonical_bytes(
            &read(
                "(metric betweenness-centrality :type coefficient :kind intensive \
                 (domain NodeType/ORGANIZATION) :provider topology-scores)",
            )
            .unwrap()
            .0,
        )
        .unwrap();
        let mut expected = Vec::new();
        push_form(&mut expected, b"metric", 5);
        // group 1 — the two positionals, in §2.11's grammar order
        push_atom(&mut expected, b"sym", b"betweenness-centrality");
        push_form(&mut expected, b"domain", 1);
        push_atom(&mut expected, b"enum", b"NodeType/ORGANIZATION");
        // group 2 — options, ascending by keyword name
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"kind");
        push_atom(&mut expected, b"sym", b"intensive");
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"provider");
        push_atom(&mut expected, b"sym", b"topology-scores");
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"type");
        push_atom(&mut expected, b"sym", b"coefficient");
        assert_eq!(bytes, expected);
    }

    /// ADR188 Row 2: `(floor x)` is an ordinary intrinsic call with no
    /// dedicated grammar production, so it needs no `FLAG_KEYWORDS` or
    /// `fixed_positionals` row — the generic path this test pins already
    /// encodes it correctly, and this row is the proof.
    #[test]
    fn a_floor_call_encodes_as_an_ordinary_generic_form() {
        let bytes = canonical_bytes(&read("(floor x)").unwrap().0).unwrap();
        let mut expected = Vec::new();
        push_form(&mut expected, b"floor", 1);
        push_atom(&mut expected, b"sym", b"x");
        assert_eq!(bytes, expected);
    }

    /// The `<intrinsic-decl>` form (§2.7) declaring `floor` needs no new
    /// discipline row either: none of `:params`/`:returns`/`:cost` is a
    /// flag (absent from `FLAG_KEYWORDS`, so each consumes the child that
    /// follows it, per §1.6's closed table), and `"intrinsic"` is absent
    /// from `fixed_positionals`, so the encoder's fallback — group 1 is
    /// whatever precedes the first keyword — already gives the right
    /// boundary: the declared name alone.
    ///
    /// `:params (real)`, not `(int)` — `floor`'s real signature is
    /// `Real → int` (§3.10's floor subsection; `declarations::
    /// parse_intrinsic_type_name` admits `real` in exactly this position).
    /// `real` is an ordinary symbol to this encoder either way: it does not
    /// interpret type-name vocabulary, only shape.
    #[test]
    fn an_intrinsic_declaration_for_floor_encodes_generically() {
        let bytes = canonical_bytes(
            &read("(intrinsic floor :params (real) :returns int :cost 5)")
                .unwrap()
                .0,
        )
        .unwrap();
        let mut expected = Vec::new();
        push_form(&mut expected, b"intrinsic", 4);
        push_atom(&mut expected, b"sym", b"floor");
        // options sorted by ascending ASCII byte order of the keyword name:
        // cost, params, returns.
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"cost");
        push_atom(&mut expected, b"int", &5i64.to_be_bytes());
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"params");
        push_form(&mut expected, b"real", 0);
        push_form(&mut expected, b"opt", 2);
        push_atom(&mut expected, b"kw", b"returns");
        push_atom(&mut expected, b"sym", b"int");
        assert_eq!(bytes, expected);
    }

    fn push_form(out: &mut Vec<u8>, tag: &[u8], nchildren: u32) {
        out.push(0x02);
        out.push(u8::try_from(tag.len()).unwrap());
        out.extend_from_slice(tag);
        out.extend_from_slice(&nchildren.to_be_bytes());
    }

    fn push_atom(out: &mut Vec<u8>, kind: &[u8], payload: &[u8]) {
        out.push(0x01);
        out.push(u8::try_from(kind.len()).unwrap());
        out.extend_from_slice(kind);
        out.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
        out.extend_from_slice(payload);
    }

    #[test]
    fn the_empty_content_set_has_the_pinned_rules_hash() {
        // SHA-256(0x03 ‖ u32 0) — the same value babylon-kernel's
        // ContentDigest test pins independently (cross-crate agreement on
        // the mandatory-rules_hash contract).
        assert_eq!(
            hex(&rules_hash_of(&[]).unwrap()),
            "a665e6b115dd56fd3e0c89be631e6eda8e9666b822e0bd7026bf0822c4bbc68f"
        );
    }

    #[test]
    fn rules_hash_sorts_by_rule_id_not_input_order() {
        let a = read("(rule aa/first :fuel 1 (when #t))").unwrap().0;
        let b = read("(rule zz/last :fuel 1 (when #t))").unwrap().0;
        assert_eq!(
            rules_hash_of(&[a.clone(), b.clone()]).unwrap(),
            rules_hash_of(&[b, a]).unwrap()
        );
    }

    #[test]
    fn unencodable_nodes_fail_loudly_never_stringify() {
        // An operator outside head position (§5.4's no-fallback law).
        let stray_op = read("(x <)").unwrap().0;
        assert!(canonical_bytes(&stray_op).is_err());
        // A form headed by a non-symbol.
        let headless = read("((a) b)").unwrap().0;
        assert!(canonical_bytes(&headless).is_err());
        // An empty form.
        let empty = read("()").unwrap().0;
        assert!(canonical_bytes(&empty).is_err());
        // rules_hash over a non-rule form.
        let not_a_rule = read("(deffield x :type int)").unwrap().0;
        assert!(rules_hash_of(&[not_a_rule]).is_err());
    }

    #[test]
    fn the_encoding_is_self_delimiting() {
        // §5.2: "unambiguously parseable back to the AST" — decode the
        // canonical bytes with an independent test-only decoder, re-encode
        // the decoded tree verbatim, and require byte identity.
        let bytes = canonical_bytes(&demo_rule()).unwrap();
        let (node, consumed) = decode(&bytes);
        assert_eq!(consumed, bytes.len(), "decoder must consume every byte");
        assert_eq!(reencode(&node), bytes);
    }

    #[test]
    fn read_all_of_a_two_rule_file_hashes_deterministically() {
        let forms =
            read_all(b"(rule b/two :fuel 1 (when #t))\n(rule a/one :fuel 1 (when #t))").unwrap();
        assert!(rules_hash_of(&forms).is_ok());
    }

    /// The reconstructed logical value of an `"enum"`-kind CAS atom (D117):
    /// either payload shape the kind admits, discriminated by the presence
    /// of exactly one `/` — an `<enum-ref>` payload always has one (§1.4's
    /// `enum-type`/`enum-member` charsets exclude `/`), a bare
    /// `defenum`/`defvocabulary` operand's payload never does.
    #[derive(Debug, PartialEq, Eq)]
    enum DecodedEnumAtom {
        EnumRef { enum_type: String, member: String },
        BareUpperIdent(String),
    }

    /// Reconstruct an `"enum"`-kind atom's LOGICAL value from its payload
    /// bytes — past the byte level `decode`/`reencode` above stop at
    /// (they never interpret a payload, only re-transcribe it). This is
    /// the round-trip the D117 register row cites: a decoder that only
    /// proved byte-for-byte re-encoding, never a payload-to-atom mapping,
    /// could not actually detect an `EnumRef`/`BareUpperIdent` confusion —
    /// two different logical atoms whose bytes happened to collide would
    /// still re-encode identically.
    fn decode_enum_atom(payload: &[u8]) -> DecodedEnumAtom {
        let text = std::str::from_utf8(payload).expect("an enum payload must be ASCII/UTF-8");
        match text.split_once('/') {
            Some((enum_type, member)) => DecodedEnumAtom::EnumRef {
                enum_type: enum_type.to_owned(),
                member: member.to_owned(),
            },
            None => DecodedEnumAtom::BareUpperIdent(text.to_owned()),
        }
    }

    /// D117's own round-trip proof: both payload shapes the `enum` CAS
    /// kind now admits decode back to the RIGHT logical atom, not just
    /// identical bytes — an `<enum-ref>` value (one `/`), a
    /// `defenum`/`defvocabulary` bare type-name operand (no `/`, mixed
    /// case), and a bare MEMBER-LIST item (no `/`, all-uppercase-plus-
    /// underscore — the #528 fix round's own corpus case,
    /// `test/corpus/declarations.txt:144`).
    #[test]
    fn the_enum_atom_kind_round_trips_both_payload_shapes_without_confusion() {
        let enum_ref_bytes = canonical_bytes(&read("NodeType/SOCIAL_CLASS").unwrap().0).unwrap();
        let (node, consumed) = decode(&enum_ref_bytes);
        assert_eq!(consumed, enum_ref_bytes.len());
        let CasNode::Atom { kind, payload } = &node else {
            panic!("expected an atom node")
        };
        assert_eq!(kind.as_slice(), b"enum");
        assert_eq!(
            decode_enum_atom(payload),
            DecodedEnumAtom::EnumRef {
                enum_type: "NodeType".into(),
                member: "SOCIAL_CLASS".into(),
            },
            "an <enum-ref> payload (one '/') must decode as EnumRef, never BareUpperIdent"
        );

        let bare_type_bytes = canonical_bytes(&read("OrgKind").unwrap().0).unwrap();
        let (node, consumed) = decode(&bare_type_bytes);
        assert_eq!(consumed, bare_type_bytes.len());
        let CasNode::Atom { kind, payload } = &node else {
            panic!("expected an atom node")
        };
        assert_eq!(kind.as_slice(), b"enum");
        assert_eq!(
            decode_enum_atom(payload),
            DecodedEnumAtom::BareUpperIdent("OrgKind".into()),
            "a bare defenum type-name operand (no '/') must decode as \
             BareUpperIdent, never EnumRef"
        );

        let bare_member_bytes = canonical_bytes(&read("STATE_APPARATUS").unwrap().0).unwrap();
        let (node, consumed) = decode(&bare_member_bytes);
        assert_eq!(consumed, bare_member_bytes.len());
        let CasNode::Atom { kind, payload } = &node else {
            panic!("expected an atom node")
        };
        assert_eq!(kind.as_slice(), b"enum");
        assert_eq!(
            decode_enum_atom(payload),
            DecodedEnumAtom::BareUpperIdent("STATE_APPARATUS".into()),
            "a bare defenum MEMBER-list item (no '/', underscore) must also \
             decode as BareUpperIdent"
        );
    }

    /// Minimal independent decoder for the self-delimitation test only.
    enum CasNode {
        Atom {
            kind: Vec<u8>,
            payload: Vec<u8>,
        },
        Form {
            tag: Vec<u8>,
            children: Vec<CasNode>,
        },
    }

    fn decode(bytes: &[u8]) -> (CasNode, usize) {
        let node_type = bytes[0];
        let name_len = usize::from(bytes[1]);
        let name = bytes[2..2 + name_len].to_vec();
        let mut pos = 2 + name_len;
        let count = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        match node_type {
            0x01 => {
                let payload = bytes[pos..pos + count].to_vec();
                (
                    CasNode::Atom {
                        kind: name,
                        payload,
                    },
                    pos + count,
                )
            }
            0x02 => {
                let mut children = Vec::with_capacity(count);
                for _ in 0..count {
                    let (child, used) = decode(&bytes[pos..]);
                    children.push(child);
                    pos += used;
                }
                (
                    CasNode::Form {
                        tag: name,
                        children,
                    },
                    pos,
                )
            }
            other => panic!("unknown node type byte {other:#04x}"),
        }
    }

    fn reencode(node: &CasNode) -> Vec<u8> {
        let mut out = Vec::new();
        match node {
            CasNode::Atom { kind, payload } => {
                out.push(0x01);
                out.push(u8::try_from(kind.len()).unwrap());
                out.extend_from_slice(kind);
                out.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_be_bytes());
                out.extend_from_slice(payload);
            }
            CasNode::Form { tag, children } => {
                out.push(0x02);
                out.push(u8::try_from(tag.len()).unwrap());
                out.extend_from_slice(tag);
                out.extend_from_slice(&u32::try_from(children.len()).unwrap().to_be_bytes());
                for child in children {
                    out.extend_from_slice(&reencode(child));
                }
            }
        }
        out
    }
}
