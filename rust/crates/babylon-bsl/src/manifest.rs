//! The scenario `manifest` form (`bsl-language.rst` §2.9) — the declared
//! cardinality ceilings §3.7 computes the static fuel bound against, plus
//! the two flags the R9 chapters put on a `ceiling` row.
//!
//! ```text
//! <manifest> ::= "(" "manifest" <symbol> <ceiling>+ ")"
//! <ceiling>  ::= "(" "ceiling" <enum-ref> ":ceiling" <int-lit>
//!                    ( ":max-members" <int-lit> )? ":invariant"? ")"
//! ```
//!
//! Three rulings live here:
//!
//! - `:max-members` is **mandatory** on a `HyperedgeType` row and **illegal**
//!   on the other two; `:invariant` is legal on a `NodeType`/`EdgeType` row
//!   and illegal on a `HyperedgeType` row. Any mismatch is `E-LOAD-042`
//!   (D27, D63).
//! - `:invariant` marks substrate no structural verb may add to or remove
//!   from: an `add-node`/`remove-node`/`add-edge`/`remove-edge` naming an
//!   invariant type is `E-LOAD-013`, checked at load off the verb's
//!   `<enum-ref>` operand. **Field writes are unaffected** — the flag
//!   constrains structure, not state (D63).
//! - the manifest must be **complete for the types the content set actually
//!   uses**: a type queried, mutated, or reached with `the` and carrying no
//!   row is `E-LOAD-045` (D76). The omission is not survivable by
//!   defaulting — `ceiling(query)` is not computable without the row,
//!   `E-LOAD-043`'s "other than 1" test cannot fire on a missing row, and
//!   `:invariant`'s check would silently never run.

use crate::fuel::CardinalityCeilings;
use crate::reader::{Atom, SExpr};
use crate::vocabulary::EnumKind;
use std::collections::{HashMap, HashSet};

/// A manifest rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// `E-LOAD-042` — a misplaced or missing `:max-members` / `:invariant`.
    RowFlagMismatch {
        /// The row's type, as `EnumType/MEMBER`.
        row: String,
        /// What is wrong.
        detail: &'static str,
    },
    /// `E-LOAD-013` — a structural verb naming an `:invariant` type.
    InvariantStructuralVerb {
        /// The verb head.
        verb: String,
        /// The invariant type, as `EnumType/MEMBER`.
        row: String,
    },
    /// `E-LOAD-043` — `the` against a type whose declared `:ceiling` is not
    /// exactly 1 (D40).
    TheAgainstNonSingleton {
        /// The type, as `EnumType/MEMBER`.
        row: String,
        /// Its declared ceiling.
        ceiling: u64,
    },
    /// `E-LOAD-045` — the content set uses a type the manifest has no row
    /// for (D76).
    MissingRow {
        /// The type, as `EnumType/MEMBER`.
        row: String,
        /// How the content set reached it.
        used_as: &'static str,
    },
    /// A form off the §2.9 grammar.
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
}

impl ManifestError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::RowFlagMismatch { .. } => Some("E-LOAD-042"),
            Self::InvariantStructuralVerb { .. } => Some("E-LOAD-013"),
            Self::TheAgainstNonSingleton { .. } => Some("E-LOAD-043"),
            Self::MissingRow { .. } => Some("E-LOAD-045"),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RowFlagMismatch { row, detail } => {
                write!(f, "E-LOAD-042: manifest row {row}: {detail} (§2.9)")
            }
            Self::InvariantStructuralVerb { verb, row } => write!(
                f,
                "E-LOAD-013: ({verb} …) names {row}, declared :invariant — the \
                 spatial substrate is written by hydration alone; field writes \
                 stay legal (§3.9)"
            ),
            Self::TheAgainstNonSingleton { row, ceiling } => write!(
                f,
                "E-LOAD-043: (the {row}) needs a declared :ceiling of exactly 1, \
                 found {ceiling} (§2.10)"
            ),
            Self::MissingRow { row, used_as } => write!(
                f,
                "E-LOAD-045: the content set {used_as} {row} and the manifest \
                 declares no ceiling row for it; the omission is not survivable \
                 by defaulting (§2.9, D76)"
            ),
            Self::Malformed { message } => write!(f, "malformed manifest: {message}"),
        }
    }
}

impl std::error::Error for ManifestError {}

fn malformed(message: impl Into<String>) -> ManifestError {
    ManifestError::Malformed {
        message: message.into(),
    }
}

/// One declared `ceiling` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeilingRow {
    /// The row's enum kind.
    pub kind: EnumKind,
    /// How many elements of that type may exist.
    pub ceiling: u64,
    /// How many members one hyperedge of that type may carry — `Some` on a
    /// `HyperedgeType` row, `None` on the other two.
    pub max_members: Option<u64>,
    /// Whether structural verbs are barred from the type (D63).
    pub invariant: bool,
}

/// A parsed scenario manifest.
#[derive(Debug, Clone, Default)]
pub struct Manifest {
    /// The manifest's own name (`(manifest <symbol> …)`).
    pub name: String,
    rows: HashMap<String, CeilingRow>,
}

impl Manifest {
    /// Read one `(manifest <symbol> <ceiling>+)` form.
    ///
    /// # Errors
    ///
    /// [`ManifestError::RowFlagMismatch`] (`E-LOAD-042`) for a misplaced or
    /// missing flag; [`ManifestError::Malformed`] off the §2.9 grammar.
    pub fn parse(form: &SExpr) -> Result<Self, ManifestError> {
        let SExpr::List(items) = form else {
            return Err(malformed("a manifest must be a form"));
        };
        let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Symbol(name)), rows @ ..] =
            items.as_slice()
        else {
            return Err(malformed(
                "(manifest <symbol> <ceiling>+) — unrecognized shape",
            ));
        };
        if head != "manifest" {
            return Err(malformed(format!(
                "expected (manifest …), found ({head} …)"
            )));
        }
        if rows.is_empty() {
            return Err(malformed("(manifest …) requires at least one ceiling row"));
        }
        let mut parsed = HashMap::new();
        for row in rows {
            let (key, ceiling) = parse_ceiling_row(row)?;
            if parsed.insert(key.clone(), ceiling).is_some() {
                return Err(malformed(format!("duplicate manifest row for {key}")));
            }
        }
        Ok(Self {
            name: name.clone(),
            rows: parsed,
        })
    }

    /// The §3.7 lookup the bound checker consumes.
    #[must_use]
    pub fn ceilings(&self) -> CardinalityCeilings {
        let ceilings = self
            .rows
            .iter()
            .map(|(key, row)| (key.clone(), row.ceiling))
            .collect();
        let max_members = self
            .rows
            .iter()
            .filter_map(|(key, row)| row.max_members.map(|m| (key.clone(), m)))
            .collect();
        CardinalityCeilings::new(ceilings, max_members)
    }

    /// One row, by `EnumType/MEMBER` key.
    #[must_use]
    pub fn row(&self, key: &str) -> Option<&CeilingRow> {
        self.rows.get(key)
    }
}

/// Destructure one `(ceiling <enum-ref> :ceiling <n> (:max-members <m>)? :invariant?)`.
fn parse_ceiling_row(form: &SExpr) -> Result<(String, CeilingRow), ManifestError> {
    let SExpr::List(items) = form else {
        return Err(malformed("a ceiling row must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::EnumRef { enum_type, member }), options @ ..] =
        items.as_slice()
    else {
        return Err(malformed(
            "(ceiling <enum-ref> :ceiling <int> …) — unrecognized shape",
        ));
    };
    if head != "ceiling" {
        return Err(malformed(format!("expected (ceiling …), found ({head} …)")));
    }
    let key = format!("{enum_type}/{member}");
    let Some(kind) = EnumKind::from_type_name(enum_type) else {
        return Err(malformed(format!(
            "{key} names no registered enum type (§3.6)"
        )));
    };
    let mut ceiling: Option<u64> = None;
    let mut max_members: Option<u64> = None;
    let mut invariant = false;
    let mut cursor = options;
    while let [SExpr::Atom(Atom::Keyword(kw)), tail @ ..] = cursor {
        match (kw.as_str(), tail) {
            ("ceiling", [SExpr::Atom(Atom::Int(n)), rest @ ..]) => {
                ceiling = Some(non_negative(*n, ":ceiling")?);
                cursor = rest;
            }
            ("max-members", [SExpr::Atom(Atom::Int(n)), rest @ ..]) => {
                max_members = Some(non_negative(*n, ":max-members")?);
                cursor = rest;
            }
            ("invariant", rest) => {
                invariant = true;
                cursor = rest;
            }
            (other, _) => {
                return Err(malformed(format!(
                    "a ceiling row takes :ceiling, :max-members and :invariant, found :{other}"
                )))
            }
        }
    }
    if !cursor.is_empty() {
        return Err(malformed(format!(
            "unexpected trailing items in a ceiling row: {cursor:?}"
        )));
    }
    let Some(ceiling) = ceiling else {
        return Err(malformed(format!("{key}: :ceiling is mandatory (§2.9)")));
    };
    // §2.9's two placement rules, both E-LOAD-042.
    match kind {
        EnumKind::HyperedgeType => {
            if max_members.is_none() {
                return Err(ManifestError::RowFlagMismatch {
                    row: key,
                    detail: ":max-members is mandatory on a HyperedgeType row — \
                             without it a members-of fold has no static bound",
                });
            }
            if invariant {
                return Err(ManifestError::RowFlagMismatch {
                    row: key,
                    detail: ":invariant is illegal on a HyperedgeType row",
                });
            }
        }
        EnumKind::NodeType | EnumKind::EdgeType => {
            if max_members.is_some() {
                return Err(ManifestError::RowFlagMismatch {
                    row: key,
                    detail: ":max-members is illegal on a NodeType or EdgeType row",
                });
            }
        }
        EnumKind::EventType => {
            return Err(ManifestError::RowFlagMismatch {
                row: key,
                detail: "a ceiling row's enum-ref is a NodeType, EdgeType or \
                         HyperedgeType member",
            })
        }
    }
    Ok((
        key,
        CeilingRow {
            kind,
            ceiling,
            max_members,
            invariant,
        },
    ))
}

fn non_negative(n: i64, what: &'static str) -> Result<u64, ManifestError> {
    u64::try_from(n).map_err(|_| malformed(format!("a negative {what} ({n}) is meaningless")))
}

/// The structural verbs `:invariant` bars, with the operand index of their
/// `<enum-ref>` (§3.9: `add-node`, `remove-node`, `add-edge`,
/// `remove-edge`). `remove-node` takes a reference, not a type, so it is
/// unreachable off the verb's operand and is checked at hydration instead —
/// the flag's job is to stop a rule *minting or severing* invariant
/// structure, which the three type-carrying verbs express.
const INVARIANT_BARRED_VERBS: [&str; 3] = ["add-node", "add-edge", "remove-edge"];

/// Every `EnumType/MEMBER` the form tree reaches, tagged by how.
fn used_types(expr: &SExpr, out: &mut Vec<(String, &'static str)>) {
    let SExpr::List(items) = expr else { return };
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        let (index, used_as) = match head.as_str() {
            "nodes" | "edges" | "hyperedges" => (1, "queries"),
            // `neighbors` reaches its EdgeType here and its result
            // NodeType at operand 4, handled just below.
            "members-of" | "hyperedges-of" | "neighbors" => (2, "queries"),
            "the" => (1, "reaches with `the`"),
            "add-node" | "add-edge" | "remove-edge" | "add-hyperedge" | "edge-between" => {
                (1, "names in a structural verb")
            }
            _ => (usize::MAX, ""),
        };
        if let Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) = items.get(index) {
            out.push((format!("{enum_type}/{member}"), used_as));
        }
        // `neighbors` reaches a second type at operand 4 (C8).
        if head == "neighbors" {
            if let Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) = items.get(4) {
                out.push((format!("{enum_type}/{member}"), "queries"));
            }
        }
    }
    for child in items {
        used_types(child, out);
    }
}

/// Apply D76 (manifest completeness, `E-LOAD-045`), D63 (`:invariant`,
/// `E-LOAD-013`) and D40 (`the` against a non-singleton, `E-LOAD-043`) to
/// one rule form.
///
/// # Errors
///
/// [`ManifestError::MissingRow`] / [`ManifestError::InvariantStructuralVerb`]
/// / [`ManifestError::TheAgainstNonSingleton`].
pub fn check_rule_against_manifest(rule: &SExpr, manifest: &Manifest) -> Result<(), ManifestError> {
    let mut used = Vec::new();
    used_types(rule, &mut used);
    // Deterministic first-failure reporting over a set that may repeat.
    let mut seen: HashSet<(String, &'static str)> = HashSet::new();
    used.retain(|entry| seen.insert(entry.clone()));
    for (key, used_as) in &used {
        if manifest.row(key).is_none() {
            return Err(ManifestError::MissingRow {
                row: key.clone(),
                used_as,
            });
        }
    }
    check_invariant_and_the(rule, manifest)
}

fn check_invariant_and_the(rule: &SExpr, manifest: &Manifest) -> Result<(), ManifestError> {
    let SExpr::List(items) = rule else {
        return Ok(());
    };
    if let Some(SExpr::Atom(Atom::Symbol(head))) = items.first() {
        if let Some(SExpr::Atom(Atom::EnumRef { enum_type, member })) = items.get(1) {
            let key = format!("{enum_type}/{member}");
            if INVARIANT_BARRED_VERBS.contains(&head.as_str())
                && manifest.row(&key).is_some_and(|row| row.invariant)
            {
                return Err(ManifestError::InvariantStructuralVerb {
                    verb: head.clone(),
                    row: key,
                });
            }
            if head == "the" {
                if let Some(row) = manifest.row(&key) {
                    if row.ceiling != 1 {
                        return Err(ManifestError::TheAgainstNonSingleton {
                            row: key,
                            ceiling: row.ceiling,
                        });
                    }
                }
            }
        }
    }
    for child in items {
        check_invariant_and_the(child, manifest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_rule_against_manifest, Manifest};
    use crate::reader::read;

    fn e(source: &str) -> crate::reader::SExpr {
        read(source).expect("test source must parse").0
    }

    const MANIFEST: &str = "(manifest demo
       (ceiling NodeType/SOCIAL_CLASS :ceiling 100)
       (ceiling NodeType/POLITY :ceiling 1)
       (ceiling NodeType/TERRITORY :ceiling 3000 :invariant)
       (ceiling EdgeType/SOLIDARITY :ceiling 40)
       (ceiling EdgeType/IN_SCALE :ceiling 5000 :invariant)
       (ceiling HyperedgeType/COMMUNITY :ceiling 200 :max-members 64))";

    fn manifest() -> Manifest {
        Manifest::parse(&e(MANIFEST)).expect("the fixture manifest is well formed")
    }

    fn rule(body: &str) -> crate::reader::SExpr {
        e(&format!(
            "(rule demo/m :role mechanic :evidence derived :material-basis \"the wage relation\" :fuel 64 \
             (bindings) (effects {body}))"
        ))
    }

    #[test]
    fn a_manifest_carries_both_ceiling_axes() {
        let m = manifest();
        let c = m.ceilings();
        assert_eq!(c.ceiling("NodeType/SOCIAL_CLASS"), Some(100));
        assert_eq!(c.max_members("HyperedgeType/COMMUNITY"), Some(64));
        assert_eq!(
            c.max_members("NodeType/SOCIAL_CLASS"),
            None,
            "the member-count axis exists only on a HyperedgeType row"
        );
    }

    #[test]
    fn a_hyperedge_row_without_max_members_is_e_load_042() {
        let err = Manifest::parse(&e(
            "(manifest m (ceiling HyperedgeType/COMMUNITY :ceiling 200))",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    #[test]
    fn max_members_on_a_node_row_is_e_load_042() {
        let err = Manifest::parse(&e(
            "(manifest m (ceiling NodeType/POLITY :ceiling 1 :max-members 3))",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    #[test]
    fn invariant_on_a_hyperedge_row_is_e_load_042() {
        let err = Manifest::parse(&e(
            "(manifest m (ceiling HyperedgeType/COMMUNITY :ceiling 2 :max-members 3 :invariant))",
        ))
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-042"));
    }

    #[test]
    fn a_structural_verb_naming_an_invariant_type_is_e_load_013() {
        for body in [
            "(add-node NodeType/TERRITORY t1)",
            "(add-edge EdgeType/IN_SCALE a b :strength 0.5c)",
            "(remove-edge EdgeType/IN_SCALE a b)",
        ] {
            let err = check_rule_against_manifest(&rule(body), &manifest()).expect_err(body);
            assert_eq!(err.spec_code(), Some("E-LOAD-013"), "{body}");
        }
    }

    #[test]
    fn a_field_write_on_an_invariant_type_accepts() {
        // D63: the flag constrains STRUCTURE, not state — a territory's
        // stock changes every tick while its existence does not.
        assert_eq!(
            check_rule_against_manifest(
                &rule("(update-node self territory/wage-bill (add 5$))"),
                &manifest()
            ),
            Ok(())
        );
    }

    #[test]
    fn the_against_a_non_singleton_ceiling_is_e_load_043() {
        let err = check_rule_against_manifest(
            &rule("(update-node (the NodeType/SOCIAL_CLASS) social-class/wealth (add 5$))"),
            &manifest(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-043"));
    }

    #[test]
    fn the_against_a_ceiling_one_carrier_accepts() {
        assert_eq!(
            check_rule_against_manifest(
                &rule("(update-node (the NodeType/POLITY) polity/imperial-rent-pool (sub 5$))"),
                &manifest()
            ),
            Ok(())
        );
    }

    #[test]
    fn a_type_the_manifest_has_no_row_for_is_e_load_045() {
        let err = check_rule_against_manifest(
            &rule("(update-node (the NodeType/SOVEREIGN) social-class/wealth (add 5$))"),
            &manifest(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-045"));
        // …and a queried type, likewise.
        let err = check_rule_against_manifest(
            &rule("(guard (exists (nodes NodeType/SOVEREIGN)) (remove-node self))"),
            &manifest(),
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-045"));
    }
}
