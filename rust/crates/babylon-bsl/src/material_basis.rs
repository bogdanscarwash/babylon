//! The rule-surface checks of §2.2's two mandatory material keywords —
//! `:material-basis` and `:fuel` — at their §2.2-pinned strictness.
//!
//! `:material-basis` is the Aleksandrov Test's **parse-time half**
//! (adversarial finding M3, scoped honestly): the parser enforces presence
//! and non-emptiness ONLY — a length-0 or whitespace-only string is
//! `E-PARSE-011`. The semantic III.8 obligation (does the named material
//! process actually ground this construct?) is *not* checked here and stays
//! with Director review and the sentinel successor's aleksandrov family,
//! never automated.
//!
//! `:fuel` must be present, `> 0` and `≤ 1_000_000` (`E-PARSE-012`).

use crate::reader::{Atom, SExpr};

/// §2.2's upper budget bound: `:fuel ≤ 1_000_000` (`E-PARSE-012`).
pub const MAX_FUEL: i64 = 1_000_000;

/// A rule-surface rejection. `code` is `None` where the reference names no
/// numbered code (a missing mandatory keyword violates the §2.3 production
/// itself; only the empty-string and range cases carry numbers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceError {
    /// `E-PARSE-011` — a `:material-basis` string of length 0 or consisting
    /// solely of whitespace.
    EmptyMaterialBasis,
    /// `E-PARSE-012` — a `:fuel` budget outside `1..=1_000_000`.
    FuelOutOfRange {
        /// The declared value.
        declared: i64,
    },
    /// A rule missing a mandatory keyword or otherwise off the §2.3
    /// production, at a point this checker must destructure.
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
}

impl SurfaceError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::EmptyMaterialBasis => Some("E-PARSE-011"),
            Self::FuelOutOfRange { .. } => Some("E-PARSE-012"),
            Self::Malformed { .. } => None,
        }
    }
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMaterialBasis => write!(
                f,
                "E-PARSE-011: :material-basis is empty or whitespace-only — \
                 the Aleksandrov Test's parse-time half requires a non-empty \
                 grounding string"
            ),
            Self::FuelOutOfRange { declared } => {
                write!(f, "E-PARSE-012: :fuel {declared} is outside 1..={MAX_FUEL}")
            }
            Self::Malformed { message } => write!(f, "malformed rule surface: {message}"),
        }
    }
}

impl std::error::Error for SurfaceError {}

fn malformed(message: impl Into<String>) -> SurfaceError {
    SurfaceError::Malformed {
        message: message.into(),
    }
}

/// Find the value atom following keyword `name` among a rule's children.
/// `pub(crate)` (Task W2, BSL Hygiene Knock-out) so `same_tick_order`'s
/// D127-citation check reuses this strict `:keyword value` extractor
/// instead of writing a second one — the same reuse-over-recreation move
/// `canonical_ast::rule_id` documents its own promotion with.
pub(crate) fn keyword_value<'a>(items: &'a [SExpr], name: &str) -> Option<&'a Atom> {
    items.windows(2).find_map(|pair| match pair {
        [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(value)] if kw == name => Some(value),
        _ => None,
    })
}

/// Check a `(rule …)` form's material keyword surface: `:material-basis`
/// present and non-empty (`E-PARSE-011`), and `:fuel` present and in
/// `1..=1_000_000` (`E-PARSE-012`). Role and evidence are checked by the
/// causal-contract loader.
///
/// # Errors
///
/// [`SurfaceError`] as above; [`SurfaceError::Malformed`] when either
/// mandatory keyword is absent or the form is not a rule list.
pub fn check_rule_surface(rule: &SExpr) -> Result<(), SurfaceError> {
    let SExpr::List(items) = rule else {
        return Err(malformed(format!(
            "expected a (rule …) form, found {rule:?}"
        )));
    };
    match keyword_value(items, "material-basis") {
        Some(Atom::Str(text)) => {
            if text.trim().is_empty() {
                return Err(SurfaceError::EmptyMaterialBasis);
            }
        }
        Some(other) => {
            return Err(malformed(format!(
                ":material-basis takes a string, found {other:?}"
            )))
        }
        None => {
            return Err(malformed(
                ":material-basis is mandatory on every rule (§2.2) and absent",
            ))
        }
    }
    match keyword_value(items, "fuel") {
        Some(Atom::Int(declared)) => {
            if *declared <= 0 || *declared > MAX_FUEL {
                return Err(SurfaceError::FuelOutOfRange {
                    declared: *declared,
                });
            }
        }
        Some(other) => {
            return Err(malformed(format!(
                ":fuel takes an integer, found {other:?}"
            )))
        }
        None => {
            return Err(malformed(
                ":fuel is mandatory on every rule (§2.2) and absent",
            ))
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read;

    fn rule_with(material_basis: &str, fuel: &str) -> SExpr {
        let source = format!(
            "(rule demo/surface :role mechanic :evidence derived :material-basis {material_basis} :fuel {fuel} \
             (bindings) (effects (update-node self social-class/agitation (add 0.05i))))"
        );
        read(&source).expect("test rule must parse").0
    }

    #[test]
    fn a_grounded_rule_passes() {
        let rule = rule_with("\"subsistence deficit at the point of reproduction\"", "64");
        assert_eq!(check_rule_surface(&rule), Ok(()));
    }

    #[test]
    fn an_empty_or_whitespace_material_basis_is_e_parse_011() {
        for text in ["\"\"", "\"   \""] {
            let err = check_rule_surface(&rule_with(text, "64")).unwrap_err();
            assert_eq!(err, SurfaceError::EmptyMaterialBasis, "{text}");
            assert_eq!(err.spec_code(), Some("E-PARSE-011"));
        }
    }

    #[test]
    fn a_missing_material_basis_violates_the_rule_production() {
        let (rule, _) = read(
            "(rule demo/ungrounded :fuel 64 (bindings) \
             (effects (update-node self social-class/agitation (add 0.05i))))",
        )
        .unwrap();
        let err = check_rule_surface(&rule).unwrap_err();
        assert!(matches!(err, SurfaceError::Malformed { .. }));
        assert_eq!(err.spec_code(), None);
    }

    #[test]
    fn fuel_must_be_positive_and_bounded_e_parse_012() {
        for fuel in ["0", "-5", "1000001"] {
            let err = check_rule_surface(&rule_with("\"wage relation\"", fuel)).unwrap_err();
            assert!(
                matches!(err, SurfaceError::FuelOutOfRange { .. }),
                "{fuel}: {err}"
            );
            assert_eq!(err.spec_code(), Some("E-PARSE-012"));
        }
        assert_eq!(
            check_rule_surface(&rule_with("\"wage relation\"", "1000000")),
            Ok(())
        );
    }
}
