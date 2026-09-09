//! Typed error identity at the source (Task 2, issue #652 `bsl-ls`).
//!
//! WHAT the loader is complaining about, as data a locator can find in a
//! parsed tree. Never prose. Populated at the raise/conversion seam, in the
//! crate that knows its own errors — **the one-brain rationale**: the crate
//! that raises an error names what it is about, because it is the only
//! layer that knows. A language server built later needs no knowledge of
//! the 52 (and counting) error struct variants across nine modules; it
//! needs four generic locator strategies over this one shared shape.
//!
//! [`identity_of`] is the wildcard-free map from [`LoadError`] (the
//! rule-loading pipeline's error type, `rule_pipeline.rs`) to an
//! [`ErrorIdentity`]. `ScenarioError` (`scenario.rs`, the `.bscn`-loading
//! error type) is a sibling surface fed by different wrapped error types
//! (`ReadError`, `GraphError`, `VocabularyError`, `DeclError`) — its seven
//! construction sites populate `identity` directly, some delegating to this
//! module's own per-type helpers (`vocabulary_identity`, `decl_identity` —
//! `pub(crate)`, not linked here: rustdoc's public-doc pass cannot resolve
//! a link to a private item) where the typed error is the same shape
//! `identity_of` already knows how to read.

use crate::bindings::BindingError;
use crate::bound_checker::BoundError;
use crate::causal_contract::{ContractError, EffectSignature};
use crate::declarations::DeclError;
use crate::domain::DomainError;
use crate::grammar::GrammarError;
use crate::material_basis::SurfaceError;
use crate::mod_anchors::AnchorError;
use crate::rule_pipeline::LoadError;
use crate::same_tick_order::{RankedRuleInputError, SameTickOrderError};
use crate::scope::{ElementNameError, ScopeError};
use crate::types::EnumRegistryError;
use crate::vocabulary::VocabularyError;

/// WHAT the loader is complaining about, as data a locator can find in a
/// parsed tree. Never prose. Populated at the raise/conversion seam, in the
/// crate that knows its own errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorIdentity {
    /// A bare symbol: a binding name, an intrinsic name, an element name.
    Name(String),
    /// A field qname, e.g. `social-class/wages`.
    Field(String),
    /// A rule id, e.g. `control-ratio/c01-prisoner-census`.
    RuleId(String),
    /// An enum type and (optionally) one of its members.
    Enum {
        /// The enum type name, e.g. `NodeType`.
        enum_type: String,
        /// The member, when the error names one.
        member: Option<String>,
    },
    /// A scenario-local node name.
    NodeLocal(String),
    /// A scenario edge, by type and endpoints.
    Edge {
        /// The edge's enum type, e.g. `EdgeType`.
        edge_type: String,
        /// The source endpoint's local name.
        from: String,
        /// The target endpoint's local name.
        to: String,
    },
    /// A keyword operand, e.g. `:fuel` — located by keyword position.
    Keyword(String),
    /// A form head plus a zero-based operand index (grammar-arity errors).
    Operand {
        /// The form's head symbol.
        form: String,
        /// The offending operand's index.
        index: usize,
    },
    /// More than one candidate; the locator refuses to guess and reports
    /// all of them as related information.
    Ambiguous(Vec<ErrorIdentity>),
}

/// Split an `EnumType/MEMBER` enum-ref string into its two halves. Falls
/// back to a bare [`ErrorIdentity::Name`] for a string that (contrary to
/// its field's own doc) carries no `/` — never a panic over a message-shape
/// assumption (III.11).
fn enum_ref_identity(enum_ref: &str) -> ErrorIdentity {
    match enum_ref.split_once('/') {
        Some((enum_type, member)) => ErrorIdentity::Enum {
            enum_type: enum_type.to_owned(),
            member: Some(member.to_owned()),
        },
        None => ErrorIdentity::Name(enum_ref.to_owned()),
    }
}

/// `vocabulary.rs`'s 6 variants — every one locatable, none prose-only, so
/// (unlike this module's other per-type helpers) this one never returns
/// `None` — clippy's `unnecessary_wraps` is right that today's enum has no
/// failing arm. Shared with `scenario.rs`'s `From<VocabularyError>` and
/// `vocab_err`, the two `ScenarioError` construction sites wrapping the
/// same type, both of which wrap the result in `Some` themselves.
pub(crate) fn vocabulary_identity(err: &VocabularyError) -> ErrorIdentity {
    match err {
        // `enum_type`/`member` name the position's own written enum-ref.
        VocabularyError::UnknownEnumType { enum_type, member }
        | VocabularyError::WrongEnumKind {
            enum_type, member, ..
        }
        | VocabularyError::UnknownEnumMember {
            enum_type, member, ..
        } => ErrorIdentity::Enum {
            enum_type: enum_type.clone(),
            member: Some(member.clone()),
        },
        // `symbol` is the colliding rendering itself — a bare symbol token.
        VocabularyError::RenderingCollision { symbol, .. } => ErrorIdentity::Name(symbol.clone()),
        // `member` is already `EnumType/MEMBER`; split it back into the
        // two-field shape the locator strategies expect.
        VocabularyError::InvalidRendering { member, .. } => enum_ref_identity(member),
        // `segment` is the qname's unresolved owner segment.
        VocabularyError::UnknownFieldOwner { segment } => ErrorIdentity::Name(segment.clone()),
    }
}

/// `declarations.rs`'s `EnumRegistryError` — the `DeclError::Enum` wrapped
/// sub-error the re-critique (§14 corrigendum) found missing from the
/// original roster. All 3 variants carry the offending `defenum` type name,
/// so (like [`vocabulary_identity`]) this never returns `None`.
fn enum_registry_identity(err: &EnumRegistryError) -> ErrorIdentity {
    match err {
        EnumRegistryError::DuplicateType { name }
        | EnumRegistryError::DuplicateMember { name, .. }
        | EnumRegistryError::EmptyMemberList { name } => ErrorIdentity::Name(name.clone()),
    }
}

/// `declarations.rs`'s 7 direct `DeclError` variants, plus its two
/// delegates (`Vocabulary` to `vocabulary.rs`'s 6, `Enum` to
/// `EnumRegistryError`'s 3). Shared with `scenario.rs`'s `load_defenum`
/// inline `.map_err`, the one `ScenarioError` construction site wrapping a
/// bare `DeclError` rather than going through `LoadError::Intrinsic`.
pub(crate) fn decl_identity(err: &DeclError) -> Option<ErrorIdentity> {
    match err {
        DeclError::Duplicate { name, .. }
        | DeclError::ReservedIntrinsicName { name, .. }
        | DeclError::SignatureMismatch { name, .. } => Some(ErrorIdentity::Name(name.clone())),
        DeclError::KernelDisagreement { field, .. }
        | DeclError::EnumKindShapeViolation { field, .. }
        | DeclError::UnknownEnumRegistryType { field, .. } => {
            Some(ErrorIdentity::Field(field.clone()))
        }
        DeclError::Vocabulary(e) => Some(vocabulary_identity(e)),
        DeclError::Malformed { .. } => None,
        DeclError::Enum(e) => Some(enum_registry_identity(e)),
    }
}

fn binding_identity(err: &BindingError) -> Option<ErrorIdentity> {
    match err {
        BindingError::BadCycleLength { name, .. }
        | BindingError::ForwardOrSelfReference { name, .. }
        | BindingError::OptionalOnExpr { name }
        | BindingError::ReservedName { name }
        | BindingError::DuplicateName { name }
        | BindingError::OptionalWithoutDefault { name }
        | BindingError::Unresolved { name, .. }
        | BindingError::UnregisteredMetric { name } => Some(ErrorIdentity::Name(name.clone())),
        BindingError::UnknownKeyword { keyword } => Some(ErrorIdentity::Keyword(keyword.clone())),
        BindingError::Malformed { .. } => None,
    }
}

/// `GrammarError` has no `Malformed`/prose-only variant either (unlike
/// most of this module's other wrapped types) — every arm, including the
/// `Vocabulary` delegate, resolves to a real identity.
fn grammar_identity(err: &GrammarError) -> ErrorIdentity {
    match err {
        GrammarError::WrongEnumKind { form, operand, .. } => ErrorIdentity::Operand {
            form: form.clone(),
            index: *operand,
        },
        GrammarError::Arity { form, found, .. } => ErrorIdentity::Operand {
            form: form.clone(),
            index: *found,
        },
        GrammarError::ArithmeticArity { operator, found } => ErrorIdentity::Operand {
            form: operator.clone(),
            index: *found,
        },
        GrammarError::NotInClosedSet { symbol, .. } => ErrorIdentity::Name(symbol.clone()),
        GrammarError::StringInExpressionPosition { literal } => {
            ErrorIdentity::Name(literal.clone())
        }
        GrammarError::StrengthFieldInit { field }
        | GrammarError::FieldInitOwnerMismatch { field, .. } => ErrorIdentity::Field(field.clone()),
        // No struct fields at all — the identity is the fixed keyword this
        // variant is always about.
        GrammarError::GraphFlagOutsideDomain => ErrorIdentity::Keyword(":graph".to_owned()),
        // §6.2's roster: "delegate to the wrapped VocabularyError, else
        // Operand" — the "else" is unreachable today (`vocabulary_identity`
        // never fails; `form` stays unused for identity purposes, only for
        // the message prefix `vocab_err` already applies at the scenario
        // side).
        GrammarError::Vocabulary { error, .. } => vocabulary_identity(error),
    }
}

fn anchor_identity(err: &AnchorError) -> Option<ErrorIdentity> {
    match err {
        AnchorError::NoSystemForRule { rule_id }
        | AnchorError::UnregisteredAnchorSystem { rule_id, .. } => {
            Some(ErrorIdentity::RuleId(rule_id.clone()))
        }
        AnchorError::UnknownKeyword { keyword } => Some(ErrorIdentity::Keyword(keyword.clone())),
        AnchorError::Malformed { .. } => None,
    }
}

fn domain_identity(err: &DomainError) -> Option<ErrorIdentity> {
    match err {
        // The self-scoped reference is the conflicting token actually
        // written at the error site; `declared` names the (non-conflicting)
        // domain form instead.
        DomainError::DomainDisagreement { found, .. } => Some(ErrorIdentity::Name(found.clone())),
        DomainError::SelfInGraphDomain { reference } => {
            Some(ErrorIdentity::Name(reference.clone()))
        }
        DomainError::Undeterminable { candidates } => Some(ErrorIdentity::Ambiguous(
            candidates
                .iter()
                .cloned()
                .map(ErrorIdentity::Name)
                .collect(),
        )),
        DomainError::Malformed { .. } => None,
    }
}

/// `ScopeError` has no `Malformed`/prose-only variant — both members carry
/// `binding` — so (like [`vocabulary_identity`]) this never returns `None`.
fn scope_identity(err: &ScopeError) -> ErrorIdentity {
    match err {
        ScopeError::ForeignFieldOutsideBody { binding, .. }
        | ScopeError::AmbiguousForeignField { binding, .. } => ErrorIdentity::Name(binding.clone()),
    }
}

/// `ElementNameError` has no `Malformed`/prose-only variant either — all
/// three members carry `name`.
fn element_name_identity(err: &ElementNameError) -> ErrorIdentity {
    match err {
        ElementNameError::ReservedElementName { name }
        | ElementNameError::ElementNameCollision { name }
        | ElementNameError::NameOutsideItsBody { name } => ErrorIdentity::Name(name.clone()),
    }
}

fn bound_identity(err: &BoundError) -> Option<ErrorIdentity> {
    match err {
        BoundError::BoundExceeded { rule_id, .. } => Some(ErrorIdentity::RuleId(rule_id.clone())),
        BoundError::MissingMaxMembers { hyperedge_type }
        | BoundError::MemberListOverCeiling { hyperedge_type, .. } => Some(ErrorIdentity::Enum {
            enum_type: hyperedge_type.clone(),
            member: None,
        }),
        BoundError::MissingCeiling { queried_type } => Some(ErrorIdentity::Enum {
            enum_type: queried_type.clone(),
            member: None,
        }),
        BoundError::UndeclaredIntrinsic { name } => Some(ErrorIdentity::Name(name.clone())),
        // A unit variant naming a fixed shape defect ((when) with zero
        // conditions) — no field to read, so no locatable identity; the
        // §6.2 precision table's File tier, same as `Malformed` (#652
        // Task 6).
        BoundError::EmptyWhenCondition | BoundError::Malformed { .. } => None,
    }
}

fn surface_identity(err: &SurfaceError) -> Option<ErrorIdentity> {
    match err {
        // A unit variant — nothing to read a field from, but the error is
        // always about the same keyword position.
        SurfaceError::EmptyMaterialBasis => {
            Some(ErrorIdentity::Keyword(":material-basis".to_owned()))
        }
        SurfaceError::FuelOutOfRange { .. } => Some(ErrorIdentity::Keyword(":fuel".to_owned())),
        SurfaceError::Malformed { .. } => None,
    }
}

fn causal_identity(err: &ContractError) -> Option<ErrorIdentity> {
    match err {
        ContractError::MissingMetadata { keyword }
        | ContractError::MalformedMetadata { keyword }
        | ContractError::UnknownMetadataValue { keyword, .. } => {
            Some(ErrorIdentity::Keyword(format!(":{keyword}")))
        }
        ContractError::UnauthorizedEffect {
            rule_id, effect, ..
        } => match effect {
            EffectSignature::NodeField(field)
            | EffectSignature::EdgeField(field)
            | EffectSignature::HyperedgeField(field) => Some(ErrorIdentity::Field(field.clone())),
            EffectSignature::Event(event) => {
                let (enum_type, member) = event.split_once('/')?;
                Some(ErrorIdentity::Enum {
                    enum_type: enum_type.to_owned(),
                    member: Some(member.to_owned()),
                })
            }
            EffectSignature::Shape(_) => Some(ErrorIdentity::RuleId(rule_id.clone())),
        },
        ContractError::MismatchedWriteAttribution { actual, .. } => {
            Some(ErrorIdentity::RuleId(actual.clone()))
        }
        ContractError::GovernedAttributionMismatch { rule_id, .. } => {
            Some(ErrorIdentity::RuleId(rule_id.clone()))
        }
        ContractError::MismatchedRuleContract { ast_contract, .. } => {
            Some(ErrorIdentity::RuleId(ast_contract.rule_id.clone()))
        }
        ContractError::GovernedFieldWriteProhibited { field, .. } => {
            Some(ErrorIdentity::Field(field.clone()))
        }
        ContractError::MalformedRule
        | ContractError::AstWalkLimit(_)
        | ContractError::MismatchedWriteOrdinal { .. }
        | ContractError::MalformedEventType { .. }
        | ContractError::ReceiptOrdinalOverflow => None,
    }
}

/// The map from a rule-load rejection to WHAT it is about, as data a
/// locator can find in a parsed tree — never derived from `err`'s own
/// message text (sentinel 7.2: no scanning a message for identity).
///
/// **Wildcard-free by design.** Every arm is a named [`LoadError`] variant;
/// there is no `_ =>`. A new variant on any wrapped type is a compile
/// error here, not a silent File-tier downgrade — the exhaustiveness
/// guarantee §6.2 describes. The corrigendum (§14) found the original
/// roster's count stale against the live enums; this match was built by
/// reading each wrapped type's actual current definition, so a live
/// re-count is `rg -c` over each `pub enum \w+Error` variant list, not a
/// re-read of the roster prose.
#[must_use]
pub fn identity_of(err: &LoadError) -> Option<ErrorIdentity> {
    match err {
        LoadError::Surface(e) => surface_identity(e),
        LoadError::Binding(e) => binding_identity(e),
        LoadError::Grammar(e) => Some(grammar_identity(e)),
        LoadError::Anchor(e) => anchor_identity(e),
        LoadError::Domain(e) => domain_identity(e),
        LoadError::Scope(e) => Some(scope_identity(e)),
        LoadError::ElementName(e) => Some(element_name_identity(e)),
        LoadError::Bound(e) => bound_identity(e),
        LoadError::Causal(e) => causal_identity(e),
        LoadError::Probability(e) => match e {
            crate::probability::ProbabilityError::DuplicateSample { sample, .. }
            | crate::probability::ProbabilityError::ProjectionNotAdjacent { sample, .. } => {
                Some(ErrorIdentity::Field(sample.clone()))
            }
            _ => None,
        },
        LoadError::Intrinsic(e) => decl_identity(e),
        LoadError::DuplicateRuleId { rule_id } => Some(ErrorIdentity::RuleId(rule_id.clone())),
        // `Read`: E-LEX is located via `ReadError.position` ->
        // `SpanTable::innermost_at` (§6.2's own table, row 1), never through
        // `ErrorIdentity`. `Type`: `TypeError` is `{code, message}` with no
        // struct variants at all (`typecheck.rs`) — nothing to name (File
        // tier until wave 2 gives it identity at the raise site). `Content`/
        // `DeferredShapeVerb`/`MintingTypeOperand`: no numbered code, no
        // wrapped typed error — each carries only an already-formatted
        // `String` (`rule_pipeline.rs`'s own doc on each variant explains
        // why: composition rules with no §2 grammar production to name a
        // field against).
        LoadError::Read(_)
        | LoadError::Type(_)
        | LoadError::Content(_)
        | LoadError::DeferredShapeVerb(_)
        | LoadError::MintingTypeOperand(_) => None,
        // The two W2 refusals are about a field. A bounded-walk refusal is
        // about the aggregate rule form and therefore stays at file tier. A
        // forged rank input names the form's real identity when available;
        // a non-rule form has no locatable identity.
        LoadError::SameTickOrder(e) => match e {
            SameTickOrderError::RankedRuleInput(error) => match error {
                RankedRuleInputError::InvalidRuleForm { .. } => None,
                RankedRuleInputError::IdentityMismatch { form_rule_id, .. } => {
                    Some(ErrorIdentity::RuleId(form_rule_id.clone()))
                }
                RankedRuleInputError::DuplicateRuleId { rule_id } => {
                    Some(ErrorIdentity::RuleId(rule_id.clone()))
                }
            },
            SameTickOrderError::StaleDefaultRead(v) => Some(ErrorIdentity::Field(v.field.clone())),
            SameTickOrderError::UnresetFanIn(v) => Some(ErrorIdentity::Field(v.field.clone())),
            SameTickOrderError::AstWalkLimit(_) => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_of_names_a_decl_duplicate() {
        let err = LoadError::Intrinsic(DeclError::Duplicate {
            name: "economy/base-subsistence".to_owned(),
            what: "intrinsic",
        });
        assert_eq!(
            identity_of(&err),
            Some(ErrorIdentity::Name("economy/base-subsistence".to_owned()))
        );
    }

    #[test]
    fn identity_of_gives_a_rule_id_for_no_system_for_rule() {
        let err = LoadError::Anchor(AnchorError::NoSystemForRule {
            rule_id: "control-ratio/c01-prisoner-census".to_owned(),
        });
        assert_eq!(
            identity_of(&err),
            Some(ErrorIdentity::RuleId(
                "control-ratio/c01-prisoner-census".to_owned()
            ))
        );
    }

    #[test]
    fn identity_of_gives_a_rule_id_for_a_duplicate_rule() {
        let err = LoadError::DuplicateRuleId {
            rule_id: "vitality/duplicate".to_owned(),
        };
        assert_eq!(
            identity_of(&err),
            Some(ErrorIdentity::RuleId("vitality/duplicate".to_owned()))
        );
    }

    #[test]
    fn identity_of_gives_an_operand_for_grammar_arity() {
        let err = LoadError::Grammar(GrammarError::Arity {
            form: "neighbors".to_owned(),
            found: 3,
            expected: "two operands",
        });
        assert!(matches!(
            identity_of(&err),
            Some(ErrorIdentity::Operand { form, .. }) if form == "neighbors"
        ));
    }

    #[test]
    fn identity_of_is_none_for_a_malformed_prose_only_variant() {
        let err = LoadError::Intrinsic(DeclError::Malformed {
            message: "(defenum ...) — unrecognized shape".to_owned(),
        });
        assert_eq!(identity_of(&err), None);
    }

    #[test]
    fn identity_of_names_causal_metadata_and_unauthorized_fields() {
        let missing = LoadError::Causal(ContractError::MissingMetadata { keyword: "role" });
        assert_eq!(
            identity_of(&missing),
            Some(ErrorIdentity::Keyword(":role".to_owned()))
        );

        let unauthorized = LoadError::Causal(ContractError::UnauthorizedEffect {
            rule_id: "vitality/probe".to_owned(),
            role: crate::causal_contract::RuleRole::ExternalEvent,
            effect: EffectSignature::NodeField("social-class/deaths".to_owned()),
        });
        assert_eq!(
            identity_of(&unauthorized),
            Some(ErrorIdentity::Field("social-class/deaths".to_owned()))
        );

        let mismatch = LoadError::Causal(ContractError::GovernedAttributionMismatch {
            rule_id: "control-ratio/c03-crisis".to_owned(),
            expected_role: crate::causal_contract::RuleRole::Recognizer,
            actual_role: crate::causal_contract::RuleRole::Mechanic,
            expected_evidence: crate::causal_contract::EvidenceClass::Derived,
            actual_evidence: crate::causal_contract::EvidenceClass::Derived,
        });
        assert_eq!(
            identity_of(&mismatch),
            Some(ErrorIdentity::RuleId("control-ratio/c03-crisis".to_owned()))
        );

        let paired_with_wrong_contract = LoadError::Causal(ContractError::MismatchedRuleContract {
            ast_contract: crate::causal_contract::RuleContract {
                rule_id: "vitality/probe".to_owned(),
                role: crate::causal_contract::RuleRole::ExternalEvent,
                evidence: crate::causal_contract::EvidenceClass::Designed,
            },
            supplied_contract: crate::causal_contract::RuleContract {
                rule_id: "control-ratio/c03-crisis".to_owned(),
                role: crate::causal_contract::RuleRole::Recognizer,
                evidence: crate::causal_contract::EvidenceClass::Derived,
            },
        });
        assert_eq!(
            identity_of(&paired_with_wrong_contract),
            Some(ErrorIdentity::RuleId("vitality/probe".to_owned()))
        );

        let bounded_walk = LoadError::Causal(ContractError::AstWalkLimit(
            crate::causal_contract::AstWalkError {
                analyzer: "causal effect footprint",
                limit: crate::causal_contract::AstWalkLimit::Depth,
                maximum: 256,
            },
        ));
        assert_eq!(identity_of(&bounded_walk), None);
    }

    #[test]
    fn identity_of_names_the_field_for_same_tick_order_refusals() {
        let err = LoadError::SameTickOrder(SameTickOrderError::StaleDefaultRead(
            crate::same_tick_order::StaleDefaultRead {
                reader_rule: "solidarity/p0-transmit".to_owned(),
                binding_name: "inbox".to_owned(),
                field: "social-class/solidarity-inbox".to_owned(),
                writer_rule: "solidarity/p1-inbox-reset".to_owned(),
            },
        ));
        assert_eq!(
            identity_of(&err),
            Some(ErrorIdentity::Field(
                "social-class/solidarity-inbox".to_owned()
            ))
        );

        let bounded_walk = LoadError::SameTickOrder(SameTickOrderError::AstWalkLimit(
            crate::causal_contract::AstWalkError {
                analyzer: "same-tick field writes",
                limit: crate::causal_contract::AstWalkLimit::Stack,
                maximum: 65_536,
            },
        ));
        assert_eq!(identity_of(&bounded_walk), None);

        let mismatch = LoadError::SameTickOrder(SameTickOrderError::RankedRuleInput(
            RankedRuleInputError::IdentityMismatch {
                supplied_rule_id: "forged/id".to_owned(),
                form_rule_id: "actual/id".to_owned(),
            },
        ));
        assert_eq!(
            identity_of(&mismatch),
            Some(ErrorIdentity::RuleId("actual/id".to_owned()))
        );

        let duplicate = LoadError::SameTickOrder(SameTickOrderError::RankedRuleInput(
            RankedRuleInputError::DuplicateRuleId {
                rule_id: "duplicate/id".to_owned(),
            },
        ));
        assert_eq!(
            identity_of(&duplicate),
            Some(ErrorIdentity::RuleId("duplicate/id".to_owned()))
        );

        let invalid = LoadError::SameTickOrder(SameTickOrderError::RankedRuleInput(
            RankedRuleInputError::InvalidRuleForm {
                supplied_rule_id: "invalid/form".to_owned(),
            },
        ));
        assert_eq!(identity_of(&invalid), None);
    }
}
