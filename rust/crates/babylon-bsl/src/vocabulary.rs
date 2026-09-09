//! The closed graph vocabulary (`bsl-language.rst` §3.6) and the
//! segment↔member rendering §2.9 states normatively (R9 chapter C1):
//! **lowercase the enum member identifier and replace each `_` with `-`**.
//! `social-class/wealth` owns off `NodeType/SOCIAL_CLASS` by that rendering
//! and by nothing else.
//!
//! Three properties this module exists to prove, each a load error rather
//! than a fallback (§3.6: "a name that is not in the registry is a load
//! error, never a fallback"):
//!
//! - a rendering that is not a valid `symbol` per §1.4 is `E-LOAD-033`;
//! - two members of different enum kinds rendering to one symbol is
//!   `E-LOAD-032`, checked once per content set over the whole vocabulary
//!   (disjointness is a property of the vocabulary, not of a field);
//! - a `deffield` whose first segment names no registered `NodeType`,
//!   `EdgeType` or `HyperedgeType` member is `E-LOAD-023`.
//!
//! `EventType` is registered here too — `emit`'s operand is kind-checked
//! against it (`E-TYPE-011`, §2.6's class rule) — but it is deliberately
//! **outside** the field-owner rendering namespace: §2.9's disjointness
//! obligation names the three graph-element kinds and no more, and an event
//! type owns no fields.

use std::collections::HashMap;

/// The four closed enum kinds an `<enum-ref>` operand position may demand
/// (§2.6's class rule, D74).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumKind {
    /// `NodeType` — `nodes`, `neighbors`' fourth operand, `the`, `(domain
    /// <enum-ref>)`, `add-node`/`remove-node`.
    NodeType,
    /// `EdgeType` — `edges`, `neighbors`' second operand, `edge-between`,
    /// `add-edge`/`remove-edge`.
    EdgeType,
    /// `HyperedgeType` — `hyperedges`, `members-of`, `hyperedges-of`,
    /// `add-hyperedge`/`remove-hyperedge`.
    HyperedgeType,
    /// `EventType` — `emit`.
    EventType,
}

impl EnumKind {
    /// The enum type name as it is written in an `<enum-ref>`.
    #[must_use]
    pub fn type_name(self) -> &'static str {
        match self {
            Self::NodeType => "NodeType",
            Self::EdgeType => "EdgeType",
            Self::HyperedgeType => "HyperedgeType",
            Self::EventType => "EventType",
        }
    }

    /// Parse an enum-ref's type-name segment.
    #[must_use]
    pub fn from_type_name(name: &str) -> Option<Self> {
        match name {
            "NodeType" => Some(Self::NodeType),
            "EdgeType" => Some(Self::EdgeType),
            "HyperedgeType" => Some(Self::HyperedgeType),
            "EventType" => Some(Self::EventType),
            _ => None,
        }
    }

    /// Whether this kind participates in the §2.9 field-owner rendering
    /// namespace (the three graph-element kinds; `EventType` does not).
    #[must_use]
    pub fn owns_fields(self) -> bool {
        !matches!(self, Self::EventType)
    }
}

/// A closed-vocabulary rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VocabularyError {
    /// `E-LOAD-030` — an enum type name that names NO real type at all:
    /// not one of the four structural kinds, and not any type this
    /// scenario declared via `defenum` either. **Corrected by G2 (#534 fix
    /// round 2 item 2, bsl-language.rst D119)**: F2 (#534 fix round item
    /// 2) originally folded a SECOND, distinct fact in here too — a
    /// syntactically-real type (a genuine structural kind, or a declared
    /// `defenum` type) written at the WRONG position — under the same
    /// code and a message that was false for exactly that case ("`EdgeType`
    /// is not a registered enum type" when it plainly is one, just not
    /// this position's). That case is [`Self::WrongEnumKind`]
    /// (`E-TYPE-011`) now — see its own doc; this variant fires only when
    /// the written name is not registered ANYWHERE, so its own message
    /// ("is not a registered enum type") is true of every case it still
    /// covers.
    UnknownEnumType {
        /// The offending type name.
        enum_type: String,
        /// The member written alongside it — carried for the full
        /// `<enum-ref>` as written (F6, #534 fix round item 6), even
        /// though the type half is what failed.
        member: String,
    },
    /// `E-TYPE-011` — an `<enum-ref>` naming a type that genuinely exists
    /// (one of the four structural kinds, or a scenario-declared
    /// `defenum` type) but is the WRONG kind for the position demanding
    /// it — §2.6's class rule (D74), split from [`Self::UnknownEnumType`]
    /// by G2 (#534 fix round 2 item 2, bsl-language.rst D119). A real
    /// type at the wrong position and a type name that is not registered
    /// anywhere are different facts; the reference implementation's
    /// hydration-side producer (`scenario::demand_enum_kind`) originally
    /// conflated them under one code and a message that was false for
    /// this half.
    WrongEnumKind {
        /// The type name as written.
        enum_type: String,
        /// The member written alongside it.
        member: String,
        /// The kind this position demands.
        expected: EnumKind,
    },
    /// `E-LOAD-031` — a member the registered enum type does not carry.
    UnknownEnumMember {
        /// The enum type.
        enum_type: String,
        /// The offending member identifier.
        member: String,
        /// Every member this kind actually declares, ascending (F6, #534
        /// fix round item 6) — so the refusal states what IS registered,
        /// not only what was not found.
        declared: Vec<String>,
    },
    /// `E-LOAD-032` — two graph-element types rendering to one symbol.
    RenderingCollision {
        /// The colliding rendering.
        symbol: String,
        /// The first member, as `EnumType/MEMBER`.
        first: String,
        /// The second member, as `EnumType/MEMBER`.
        second: String,
    },
    /// `E-LOAD-033` — a member whose rendering is not a valid `symbol`.
    InvalidRendering {
        /// The member, as `EnumType/MEMBER`.
        member: String,
        /// The rendering that failed §1.4's `symbol` production.
        rendering: String,
    },
    /// `E-LOAD-023` — a field qname whose first segment names no registered
    /// `NodeType`, `EdgeType` or `HyperedgeType` member.
    UnknownFieldOwner {
        /// The offending first segment.
        segment: String,
    },
}

impl VocabularyError {
    /// The spec's error code.
    #[must_use]
    pub fn spec_code(&self) -> &'static str {
        match self {
            Self::UnknownEnumType { .. } => "E-LOAD-030",
            Self::WrongEnumKind { .. } => "E-TYPE-011",
            Self::UnknownEnumMember { .. } => "E-LOAD-031",
            Self::RenderingCollision { .. } => "E-LOAD-032",
            Self::InvalidRendering { .. } => "E-LOAD-033",
            Self::UnknownFieldOwner { .. } => "E-LOAD-023",
        }
    }
}

impl std::fmt::Display for VocabularyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownEnumType { enum_type, member } => write!(
                f,
                "E-LOAD-030: {enum_type}/{member} — {enum_type} is not a \
                 registered enum type (§3.6)"
            ),
            Self::WrongEnumKind {
                enum_type,
                member,
                expected,
            } => write!(
                f,
                "E-TYPE-011: this position takes a member of {}, found \
                 {enum_type}/{member} (§2.6's class rule, D74)",
                expected.type_name()
            ),
            Self::UnknownEnumMember {
                enum_type,
                member,
                declared,
            } => write!(
                f,
                "E-LOAD-031: {enum_type}/{member} is not a registered member \
                 — never a default (§1.5); {enum_type} declares: {}",
                format_declared_members(declared)
            ),
            Self::RenderingCollision {
                symbol,
                first,
                second,
            } => write!(
                f,
                "E-LOAD-032: {first} and {second} both render to '{symbol}'; \
                 the three graph-element renderings must be pairwise disjoint \
                 (§2.9)"
            ),
            Self::InvalidRendering { member, rendering } => write!(
                f,
                "E-LOAD-033: {member} renders to '{rendering}', which is not a \
                 valid symbol (§1.4)"
            ),
            Self::UnknownFieldOwner { segment } => write!(
                f,
                "E-LOAD-023: '{segment}' names no registered NodeType, EdgeType \
                 or HyperedgeType member (§2.9)"
            ),
        }
    }
}

impl std::error::Error for VocabularyError {}

/// F6 (#534 fix round item 6): summarize a kind's declared members for a
/// refusal message — the full list when short, a bounded prefix
/// (`SHOWN_MEMBERS`) plus a count when long, so a vocabulary with hundreds
/// of members never blows the message out.
const SHOWN_MEMBERS: usize = 8;

fn format_declared_members(declared: &[String]) -> String {
    if declared.is_empty() {
        return "no members".to_owned();
    }
    if declared.len() <= SHOWN_MEMBERS {
        declared.join(", ")
    } else {
        format!(
            "{}, … (+{} more)",
            declared[..SHOWN_MEMBERS].join(", "),
            declared.len() - SHOWN_MEMBERS
        )
    }
}

/// Render an enum member identifier to its BSL segment symbol (§2.9):
/// lowercase, `_` → `-`. The result must satisfy §1.4's `symbol`
/// production or the member is `E-LOAD-033`.
#[must_use]
pub fn render_member(member: &str) -> String {
    member
        .chars()
        .map(|c| {
            if c == '_' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

/// §1.4's `symbol` production: `LOWER ( LOWER | DIGIT | "-" )*`, max 64.
fn is_valid_symbol(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && s.len() <= 64
}

/// The registered closed vocabulary of graph-element and event types.
#[derive(Debug, Clone, Default)]
pub struct ClosedVocabulary {
    members: HashMap<EnumKind, Vec<String>>,
    /// Rendering → `(kind, member)`, over the three field-owning kinds only.
    renderings: HashMap<String, (EnumKind, String)>,
}

impl ClosedVocabulary {
    /// Build the registry from the declared members of each kind, running
    /// §2.9's rendering-validity (`E-LOAD-033`) and pairwise-disjointness
    /// (`E-LOAD-032`) checks once over the whole vocabulary.
    ///
    /// # Errors
    ///
    /// [`VocabularyError::InvalidRendering`] / [`VocabularyError::RenderingCollision`].
    pub fn new(
        members: impl IntoIterator<Item = (EnumKind, Vec<String>)>,
    ) -> Result<Self, VocabularyError> {
        let mut by_kind: HashMap<EnumKind, Vec<String>> = HashMap::new();
        for (kind, names) in members {
            by_kind.entry(kind).or_default().extend(names);
        }
        // Deterministic iteration: the checks below must report the SAME
        // first collision on every run, so the kinds are walked in a fixed
        // order and each kind's members are sorted.
        let mut renderings: HashMap<String, (EnumKind, String)> = HashMap::new();
        for kind in [
            EnumKind::NodeType,
            EnumKind::EdgeType,
            EnumKind::HyperedgeType,
            EnumKind::EventType,
        ] {
            let Some(names) = by_kind.get_mut(&kind) else {
                continue;
            };
            names.sort();
            names.dedup();
            if !kind.owns_fields() {
                continue;
            }
            for member in names.iter() {
                let rendering = render_member(member);
                if !is_valid_symbol(&rendering) {
                    return Err(VocabularyError::InvalidRendering {
                        member: format!("{}/{member}", kind.type_name()),
                        rendering,
                    });
                }
                if let Some((other_kind, other_member)) = renderings.get(&rendering) {
                    return Err(VocabularyError::RenderingCollision {
                        symbol: rendering,
                        first: format!("{}/{other_member}", other_kind.type_name()),
                        second: format!("{}/{member}", kind.type_name()),
                    });
                }
                renderings.insert(rendering, (kind, member.clone()));
            }
        }
        Ok(Self {
            members: by_kind,
            renderings,
        })
    }

    /// Check an `<enum-ref>` against the registry (§1.5: both checks are
    /// load-time, `E-LOAD-030` / `E-LOAD-031`).
    ///
    /// # Errors
    ///
    /// [`VocabularyError::UnknownEnumType`] / [`VocabularyError::UnknownEnumMember`].
    pub fn check_enum_ref(
        &self,
        enum_type: &str,
        member: &str,
    ) -> Result<EnumKind, VocabularyError> {
        let Some(kind) = EnumKind::from_type_name(enum_type) else {
            return Err(VocabularyError::UnknownEnumType {
                enum_type: enum_type.to_owned(),
                member: member.to_owned(),
            });
        };
        let Some(names) = self.members.get(&kind) else {
            // F1 (#534 fix round item 1; bsl-language.rst §2.13, D119): a
            // kind ABSENT from the declared vocabulary — no `defvocabulary`
            // for it at all — leaves that kind's checking exactly as inert
            // as it is today. Before this, `self.members.get(&kind)`
            // returning `None` fell straight into `is_some_and`'s `false`
            // arm below and was refused as `UnknownEnumMember`
            // (`E-LOAD-031`) — indistinguishable from a DECLARED kind whose
            // members just don't include this one. Never conflate the two:
            // a DECLARED kind's own typo still refuses below.
            return Ok(kind);
        };
        if names.iter().any(|n| n == member) {
            Ok(kind)
        } else {
            Err(VocabularyError::UnknownEnumMember {
                enum_type: enum_type.to_owned(),
                member: member.to_owned(),
                declared: names.clone(),
            })
        }
    }

    /// The graph-element type a field qname's first segment owns off
    /// (§2.9's rendering, read backwards).
    ///
    /// # Errors
    ///
    /// [`VocabularyError::UnknownFieldOwner`] (`E-LOAD-023`).
    pub fn owner_of(&self, segment: &str) -> Result<(EnumKind, &str), VocabularyError> {
        self.renderings
            .get(segment)
            .map(|(kind, member)| (*kind, member.as_str()))
            .ok_or_else(|| VocabularyError::UnknownFieldOwner {
                segment: segment.to_owned(),
            })
    }

    /// The owning type of a whole field qname (`social-class/wealth` →
    /// `(NodeType, "SOCIAL_CLASS")`).
    ///
    /// # Errors
    ///
    /// [`VocabularyError::UnknownFieldOwner`] (`E-LOAD-023`), including for
    /// a qname with no `/`.
    pub fn owner_of_field(&self, qname: &str) -> Result<(EnumKind, &str), VocabularyError> {
        let segment = qname.split('/').next().unwrap_or(qname);
        self.owner_of(segment)
    }

    /// Every registered member of one kind, in ascending byte order.
    #[must_use]
    pub fn members(&self, kind: EnumKind) -> &[String] {
        self.members.get(&kind).map_or(&[], Vec::as_slice)
    }

    /// Whether the scenario declared this kind, including as present-empty.
    #[must_use]
    pub fn contains_kind(&self, kind: EnumKind) -> bool {
        self.members.contains_key(&kind)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClosedVocabulary, EnumKind, VocabularyError};

    fn vocabulary() -> ClosedVocabulary {
        ClosedVocabulary::new([
            (
                EnumKind::NodeType,
                vec!["SOCIAL_CLASS".to_owned(), "POLITY".to_owned()],
            ),
            (
                EnumKind::EdgeType,
                vec!["SOLIDARITY".to_owned(), "EXPLOITATION".to_owned()],
            ),
            (EnumKind::HyperedgeType, vec!["ECONOMIC_SECTOR".to_owned()]),
            (EnumKind::EventType, vec!["RUPTURE".to_owned()]),
        ])
        .expect("the fixture vocabulary is disjoint")
    }

    #[test]
    fn the_rendering_is_lowercase_with_underscores_hyphenated() {
        assert_eq!(super::render_member("SOCIAL_CLASS"), "social-class");
        assert_eq!(super::render_member("SOLIDARITY"), "solidarity");
        assert_eq!(super::render_member("ECONOMIC_SECTOR"), "economic-sector");
    }

    #[test]
    fn a_field_owner_may_be_a_node_edge_or_hyperedge_type() {
        let v = vocabulary();
        assert_eq!(
            v.owner_of_field("social-class/wealth").unwrap(),
            (EnumKind::NodeType, "SOCIAL_CLASS")
        );
        assert_eq!(
            v.owner_of_field("solidarity/strength").unwrap(),
            (EnumKind::EdgeType, "SOLIDARITY")
        );
        assert_eq!(
            v.owner_of_field("economic-sector/output").unwrap(),
            (EnumKind::HyperedgeType, "ECONOMIC_SECTOR")
        );
    }

    #[test]
    fn an_unregistered_first_segment_is_e_load_023() {
        let err = vocabulary().owner_of_field("imperium/rent").unwrap_err();
        assert_eq!(
            err,
            VocabularyError::UnknownFieldOwner {
                segment: "imperium".to_owned()
            }
        );
        assert_eq!(err.spec_code(), "E-LOAD-023");
    }

    #[test]
    fn an_event_type_owns_no_fields() {
        // EventType is registered for `emit`'s kind check but is outside the
        // field-owner rendering namespace (§2.9 names three kinds).
        let err = vocabulary().owner_of_field("rupture/severity").unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-023");
    }

    #[test]
    fn a_node_edge_rendering_collision_is_e_load_032() {
        let err = ClosedVocabulary::new([
            (EnumKind::NodeType, vec!["TENANCY".to_owned()]),
            (EnumKind::EdgeType, vec!["TENANCY".to_owned()]),
        ])
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-032");
        assert!(matches!(err, VocabularyError::RenderingCollision { .. }));
    }

    #[test]
    fn a_node_hyperedge_collision_is_e_load_032_too() {
        let err = ClosedVocabulary::new([
            (EnumKind::NodeType, vec!["COMMUNITY".to_owned()]),
            (EnumKind::HyperedgeType, vec!["COMMUNITY".to_owned()]),
        ])
        .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-032");
    }

    #[test]
    fn an_event_type_never_collides_with_a_graph_element_type() {
        // EventType renderings are not in the namespace, so a same-named
        // event type is legal — the disjointness obligation is over three.
        assert!(ClosedVocabulary::new([
            (EnumKind::NodeType, vec!["RUPTURE".to_owned()]),
            (EnumKind::EventType, vec!["RUPTURE".to_owned()]),
        ])
        .is_ok());
    }

    #[test]
    fn a_member_whose_rendering_is_not_a_symbol_is_e_load_033() {
        let err =
            ClosedVocabulary::new([(EnumKind::NodeType, vec!["_LEADING".to_owned()])]).unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-033");
        let err = ClosedVocabulary::new([(EnumKind::NodeType, vec!["3RD_PARTY".to_owned()])])
            .unwrap_err();
        assert_eq!(err.spec_code(), "E-LOAD-033");
    }

    #[test]
    fn enum_ref_membership_is_checked_at_load_never_defaulted() {
        let v = vocabulary();
        assert_eq!(
            v.check_enum_ref("NodeType", "SOCIAL_CLASS"),
            Ok(EnumKind::NodeType)
        );
        assert_eq!(
            v.check_enum_ref("NodeType", "NOWHERE")
                .unwrap_err()
                .spec_code(),
            "E-LOAD-031"
        );
        assert_eq!(
            v.check_enum_ref("DoctrineTag", "X")
                .unwrap_err()
                .spec_code(),
            "E-LOAD-030"
        );
    }

    #[test]
    fn a_kind_absent_from_the_vocabulary_is_inert_not_e_load_031() {
        // F1 (#534 fix round item 1; docs/reference/bsl-language.rst §2.13:
        // "a content set that declares no defvocabulary for a kind leaves
        // THAT KIND's checking exactly as inert as it is today"). A
        // PARTIAL vocabulary — NodeType declared, EdgeType never declared
        // at all — must leave EdgeType's own checking INERT (`Ok`), never
        // conflated with a DECLARED kind whose members happen not to
        // include the one written (`enum_ref_membership_is_checked_at_
        // load_never_defaulted` above pins THAT case, `NodeType/NOWHERE`,
        // which stays `E-LOAD-031`).
        let partial =
            ClosedVocabulary::new([(EnumKind::NodeType, vec!["SOCIAL_CLASS".to_owned()])])
                .expect("a single-kind vocabulary is trivially disjoint");
        assert_eq!(
            partial.check_enum_ref("EdgeType", "SOLIDARITY"),
            Ok(EnumKind::EdgeType),
            "EdgeType was never declared — its checking must stay inert"
        );
        // The DECLARED kind's own typo check is unaffected by this fix.
        assert_eq!(
            partial
                .check_enum_ref("NodeType", "NOWHERE")
                .unwrap_err()
                .spec_code(),
            "E-LOAD-031"
        );
    }

    // ---- F6 (#534 fix round item 6): refusal message house style — the
    // full ref as written, and the declared members of that kind. ----

    #[test]
    fn the_e_load_031_message_names_the_full_ref_and_the_declared_members() {
        let v = vocabulary();
        let msg = v
            .check_enum_ref("NodeType", "NOWHERE")
            .unwrap_err()
            .to_string();
        assert!(msg.contains("NodeType/NOWHERE"), "{msg}");
        assert!(msg.contains("SOCIAL_CLASS"), "{msg}");
        assert!(msg.contains("POLITY"), "{msg}");
    }

    #[test]
    fn the_e_load_031_message_bounds_a_long_declared_list() {
        let long = ClosedVocabulary::new([(
            EnumKind::NodeType,
            (0..20).map(|i| format!("MEMBER_{i}")).collect(),
        )])
        .unwrap();
        let msg = long
            .check_enum_ref("NodeType", "NOWHERE")
            .unwrap_err()
            .to_string();
        assert!(
            msg.contains("more"),
            "a 20-member kind must summarize: {msg}"
        );
    }

    #[test]
    fn the_e_load_030_message_names_the_full_ref_as_written() {
        let v = vocabulary();
        let msg = v
            .check_enum_ref("DoctrineTag", "X")
            .unwrap_err()
            .to_string();
        assert!(msg.contains("DoctrineTag/X"), "{msg}");
    }

    #[test]
    fn the_e_type_011_message_sidesteps_the_article_for_every_expected_kind() {
        // H3 (#534 fix round 3, cosmetic): "this position takes a {}
        // member" reads as "a EdgeType member" / "a EventType member" for
        // two of the four `EnumKind`s — grammatically wrong, since English
        // needs "an" before a vowel sound. "takes a member of {}" sidesteps
        // the article entirely rather than special-casing which kinds need
        // "an" (a distinction with no game-logic meaning, so not worth a
        // branch).
        for expected in [
            EnumKind::NodeType,
            EnumKind::EdgeType,
            EnumKind::HyperedgeType,
            EnumKind::EventType,
        ] {
            let msg = VocabularyError::WrongEnumKind {
                enum_type: "EdgeType".to_owned(),
                member: "SOLIDARITY".to_owned(),
                expected,
            }
            .to_string();
            assert!(
                msg.contains(&format!("a member of {}", expected.type_name())),
                "{msg}"
            );
            assert!(!msg.contains("a NodeType member"), "{msg}");
            assert!(!msg.contains("a EdgeType member"), "{msg}");
            assert!(!msg.contains("a HyperedgeType member"), "{msg}");
            assert!(!msg.contains("a EventType member"), "{msg}");
        }
    }
}
