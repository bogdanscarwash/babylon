//! Top-form declarations (`bsl-language.rst` §2.2's `<top-form>`): the
//! `deffield` and `intrinsic` forms, read into the registries the
//! typechecker and the fuel bound checker consume. Every check here is a
//! **load** check (§3): there is no partial load and no "skip the bad
//! declaration" mode.
//!
//! R9 chapter C1 lands three rulings in this module:
//!
//! - a `deffield`'s first segment may name a `NodeType`, an `EdgeType` **or**
//!   a `HyperedgeType` member (D31) — an unregistered segment is
//!   `E-LOAD-023`, resolved through [`crate::vocabulary`];
//! - every `EdgeType` carries one **implicitly declared** field,
//!   `<edge-type>/strength`, `:type coefficient` `:kind extensive` (D32;
//!   lowercase per D94 — type names are lowercase symbols, ADR191 R4).
//!   Re-declaring it explicitly is `E-LOAD-001`, a duplicate field
//!   declaration, so there is exactly one home for its type and kind;
//! - every §5.2 form-head symbol is **reserved against the intrinsic
//!   namespace** (D33): declaring an intrinsic with one is `E-LOAD-024`,
//!   checked against the §5.2 list itself so adding a form tag automatically
//!   reserves it.

use crate::reader::{Atom, SExpr};
use crate::types::{BslType, EnumRegistry, EnumRegistryError, EnumTypeId, FieldDecl, FieldKind};
use crate::vocabulary::{render_member, ClosedVocabulary, EnumKind, VocabularyError};
use std::collections::HashMap;

/// The §5.2 form tags that are valid `symbol`s, and are therefore reserved
/// against the intrinsic namespace (D33). The operator tags (`<`, `+`, …)
/// are excluded because an `intrinsic` name is a `symbol` (§1.4) and cannot
/// spell them, so no collision is expressible there. `opt` — the synthetic
/// keyword-option tag — is included: it is a §5.2 tag like any other.
pub const RESERVED_FORM_TAGS: [&str; 52] = [
    "add",
    "add-edge",
    "add-hyperedge",
    "add-node",
    "and",
    "anchor",
    "binding",
    "bindings",
    "branch",
    "ceiling",
    "choose",
    "deffield",
    "domain",
    "edge-between",
    "edges",
    "effects",
    "emit",
    "exists",
    "field-of",
    "fold",
    "for-each",
    "forall",
    "guard",
    "hyperedges",
    "hyperedges-of",
    "if",
    "intrinsic",
    "manifest",
    "members",
    "members-of",
    "metric",
    "metric-of",
    "neighbors",
    "nodes",
    "not",
    "opt",
    "or",
    "quantize-mass",
    "remove-edge",
    "remove-hyperedge",
    "remove-node",
    "rule",
    "scale",
    "select-max",
    "select-min",
    "set",
    "sub",
    "the",
    "update-edge",
    "update-hyperedge",
    "update-node",
    "when",
];

/// The declarable intrinsic set (`bsl-language.rst` §3.10). The Program 28
/// roadmap's R10 row holds the **transcendental** cap at `{exp, log}` **at
/// most**, citing ADR176 r21; r21's own text pins a *mechanism*
/// (transcendentals cross via a pinned soft-float libm crate with golden
/// vectors per intrinsic) and does not enumerate a membership, so the
/// enumeration is the roadmap's rendering of it. **R10 is operative** for
/// the transcendental pair.
///
/// `floor` joins the set under a **separate** authority: ADR188 Row 2 (the
/// intrinsic-cap rider slate, Director-disposed 2026-08-10), affirmed by
/// ADR191 R3's consequence note that the mortality family's mechanical half
/// rides this rider. It is not a transcendental and does not cross via the
/// pinned libm crate r21 governs — `f64::floor` is IEEE-754's
/// `roundToIntegralTowardNegative`, exactly specified by the standard
/// itself (not by §4.3, whose basic-op list is `+ − × ÷` and comparison
/// only), so it reproduces bit-exactly across conforming implementations
/// without needing r21's golden-vector machinery. See `bsl-language.rst`
/// §3.10 and Draft-Ruling Register D97 for the ratified name/domain.
///
/// Amendment AJ retires the former author-callable draw name. ADR188 Row
/// 11 and D69 still govern the deterministic engine seam, but a finite
/// kernel consumes that one draw privately: it is neither a member of this
/// set nor a row in [`kernel_signature`].
///
/// **ADR219 (Director ruling 2026-08-22): the exact-arithmetic rider
/// train** grows the set by six. `sqrt` takes ADR188 Row 6's fallback
/// rider (the Director's measure-preferred disposition superseded by the
/// same Director's explicit approval); `round-half-even` lands ADR188
/// Row 3's ratified housekeeping rider, resolving D70's "recorded, not
/// resolved" (§3.2/§2.7 oblige the kernel to expose the half-even
/// algorithm to rules; this constant was the missing half); and
/// `min`/`max`/`abs`/`clamp` supersede ADR188 Rows 4/5's "no rider"
/// dispositions on #591 item 2's accumulated port evidence ("my top
/// BSL-improvement candidate", Director directive 2026-08-15), approved
/// by the same ruling. All six cross via IEEE-754 exactly-specified
/// operations — correctly-rounded `sqrt`, `roundTiesToEven`,
/// comparison/copysign arithmetic — so like `floor` they need no pinned
/// soft-float libm and owe no §4.3 golden-vector family (that disposition
/// recorded per-name in §3.10's normative paragraphs). The
/// **transcendental** roster stays the R10 pair alone: `tanh`, `entropy`,
/// `renormalize`, `trunc` remain outside, and `sigmoid` remains
/// prohibited outright (below).
pub const DECLARABLE_INTRINSICS: [&str; 9] = [
    "exp",
    "log",
    "floor",
    "sqrt",
    "round-half-even",
    "min",
    "max",
    "abs",
    "clamp",
];

/// Intrinsic names that are **prohibited outright**, not merely undeclared
/// (§3.10, D71). `sigmoid` would hand content the exact mechanism ADR172
/// ruling 5 forbids, pre-packaged and named; it is the one part of the
/// doctrine gate that can be made mechanical, so it is.
pub const PROHIBITED_INTRINSIC_NAMES: [&str; 1] = ["sigmoid"];

/// A declaration-surface rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclError {
    /// `E-LOAD-001` — a duplicate rule id, field declaration or intrinsic
    /// declaration across the content set (§2.2). Re-declaring an implicit
    /// `<edge-type>/strength` lands here too (D32).
    Duplicate {
        /// What was declared twice.
        name: String,
        /// The declaration kind, for the message.
        what: &'static str,
    },
    /// `E-LOAD-022` — a `deffield` whose type or kind disagrees with the
    /// kernel's model registration (D9: the kernel is checked against
    /// content, not the reverse).
    KernelDisagreement {
        /// The field.
        field: String,
        /// What disagrees.
        detail: String,
    },
    /// `E-LOAD-023` / `E-LOAD-030` / … — a closed-vocabulary failure.
    Vocabulary(VocabularyError),
    /// `E-LOAD-024` — an `intrinsic` declared with a reserved §5.2 form-head
    /// name, or with a prohibited name (§3.10's `sigmoid`).
    ReservedIntrinsicName {
        /// The offending name.
        name: String,
        /// Why it is reserved.
        reason: &'static str,
    },
    /// `E-LOAD-020` — an `intrinsic` declaration whose `:params`/`:returns`
    /// disagrees with the kernel's registration for that name (§2.7: "A
    /// declaration whose signature disagrees with the kernel's
    /// registration is `E-LOAD-020`").
    SignatureMismatch {
        /// The declared name.
        name: String,
        /// What disagrees.
        detail: String,
    },
    /// A form off the §2.9 grammar at a point this reader must destructure.
    Malformed {
        /// What was expected, and what was found.
        message: String,
    },
    /// `E-LOAD-053` (§2.13, D101) — a `:type enum` `deffield` missing
    /// `:enum-type`, carrying a `:kind` it structurally forbids, or a
    /// non-enum `deffield` carrying `:enum-type`. The two slots are
    /// mutually exclusive and jointly exhaustive against `:type`.
    EnumKindShapeViolation {
        /// The offending field.
        field: String,
        /// Which half of the exclusivity rule was violated.
        detail: &'static str,
    },
    /// `E-LOAD-054` (§2.13, D101) — an `:enum-type` naming a `defenum`
    /// type the registry does not carry.
    UnknownEnumRegistryType {
        /// The offending field.
        field: String,
        /// The unresolved type name.
        enum_type: String,
    },
    /// A `defenum` declaration's own construction failure — `E-LOAD-001`
    /// for a duplicate type/member (§2.13: "like any other duplicate
    /// declaration"), uncoded for an empty member list (the grammar's
    /// `<enum-member>+` is one-or-more, never zero — a shape violation,
    /// not a semantic duplicate).
    Enum(EnumRegistryError),
}

impl DeclError {
    /// The spec's error code, where the reference names one.
    #[must_use]
    pub fn spec_code(&self) -> Option<&'static str> {
        match self {
            Self::Duplicate { .. }
            | Self::Enum(
                EnumRegistryError::DuplicateType { .. } | EnumRegistryError::DuplicateMember { .. },
            ) => Some("E-LOAD-001"),
            Self::KernelDisagreement { .. } => Some("E-LOAD-022"),
            Self::Vocabulary(e) => Some(e.spec_code()),
            Self::ReservedIntrinsicName { .. } => Some("E-LOAD-024"),
            Self::SignatureMismatch { .. } => Some("E-LOAD-020"),
            Self::Malformed { .. } | Self::Enum(EnumRegistryError::EmptyMemberList { .. }) => None,
            Self::EnumKindShapeViolation { .. } => Some("E-LOAD-053"),
            Self::UnknownEnumRegistryType { .. } => Some("E-LOAD-054"),
        }
    }
}

impl std::fmt::Display for DeclError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate { name, what } => {
                write!(f, "E-LOAD-001: duplicate {what} declaration: {name}")
            }
            Self::KernelDisagreement { field, detail } => write!(
                f,
                "E-LOAD-022: deffield {field} disagrees with the kernel's \
                 registration: {detail}"
            ),
            Self::Vocabulary(e) => write!(f, "{e}"),
            Self::ReservedIntrinsicName { name, reason } => write!(
                f,
                "E-LOAD-024: intrinsic name '{name}' is reserved — {reason}"
            ),
            Self::SignatureMismatch { name, detail } => write!(
                f,
                "E-LOAD-020: intrinsic {name} disagrees with the kernel's \
                 registration: {detail}"
            ),
            Self::Malformed { message } => write!(f, "malformed declaration: {message}"),
            Self::EnumKindShapeViolation { field, detail } => {
                write!(f, "E-LOAD-053: deffield {field}: {detail}")
            }
            Self::UnknownEnumRegistryType { field, enum_type } => write!(
                f,
                "E-LOAD-054: deffield {field}: :enum-type {enum_type} is not a \
                 registered defenum type (§2.13)"
            ),
            Self::Enum(
                e @ (EnumRegistryError::DuplicateType { .. }
                | EnumRegistryError::DuplicateMember { .. }),
            ) => {
                write!(f, "E-LOAD-001: {e}")
            }
            Self::Enum(e @ EnumRegistryError::EmptyMemberList { .. }) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for DeclError {}

impl From<VocabularyError> for DeclError {
    fn from(e: VocabularyError) -> Self {
        Self::Vocabulary(e)
    }
}

impl From<EnumRegistryError> for DeclError {
    fn from(e: EnumRegistryError) -> Self {
        Self::Enum(e)
    }
}

fn malformed(message: impl Into<String>) -> DeclError {
    DeclError::Malformed {
        message: message.into(),
    }
}

/// The declared type + kind of a field, plus the graph-element kind that
/// owns it (§2.9's first segment).
#[derive(Debug, Clone)]
pub struct OwnedFieldDecl {
    /// The field's type and intensivity kind.
    pub decl: FieldDecl,
    /// Which enum kind the owning type belongs to.
    pub owner_kind: EnumKind,
    /// The owning type's member identifier, e.g. `SOCIAL_CLASS`.
    pub owner_member: String,
    /// Whether this row is the implicit `<edge-type>/strength` (D32) rather
    /// than an authored `deffield`.
    pub implicit: bool,
}

/// Every declared field of a content set, keyed by qname.
///
/// **Half-wired (ADR109's declared-not-wired discipline; #528 fix round,
/// MINOR item 5; wiring updated by T2, issue #559).**
/// [`Self::with_implicit_edge_strength`] IS production-wired — the D32
/// implicit-strength wiring: `babylon_tick`'s `prepare_rules`
/// (`rust/crates/babylon-tick/src/lib.rs`, on both `run_once_into`'s and
/// `TickSession::new`'s path) calls it to seed the implicit
/// `<edge-type>/strength` `TypeEnv` rows from the scenario's declared
/// `defvocabulary EdgeType` members. [`Self::declare`] (the only minting
/// path for `E-LOAD-053`/`E-LOAD-054`'s
/// `EnumKindShapeViolation`/`UnknownEnumRegistryType` in production shape)
/// remains uncalled from any live tick path: every reference to it outside
/// this module's own `#[cfg(test)]` tests and `tests/r9_chapters.rs` is
/// zero. Authored fields still reach production's `TypeEnv` straight from
/// a loaded scenario's own `deffield` forms — `prepare_rules`'s own
/// comment names the remaining gap from the other side: "the scenario's
/// `deffield` forms ARE the registries for slice 1. When Phase 2's content
/// registries land they replace this wholesale." Rule-side `deffield`
/// CONTENT PACKS (fields declared alongside `.bsl` rule content rather
/// than re-declared per scenario) are what will wire `declare` in — until
/// then it stays fully built, fully tested, and correctly unreferenced
/// from any live tick.
#[derive(Debug, Clone, Default)]
pub struct FieldRegistry {
    fields: HashMap<String, OwnedFieldDecl>,
}

impl FieldRegistry {
    /// Seed the registry with the implicit `<edge-type>/strength` field of
    /// every registered `EdgeType` (D32): `Coefficient`, `extensive`. The
    /// `extensive` kind is load-bearing — an intensive `strength` would
    /// make §2.4's `sum_strength` coverage row `E-TYPE-041` and therefore
    /// inexpressible.
    #[must_use]
    pub fn with_implicit_edge_strength(vocabulary: &ClosedVocabulary) -> Self {
        let mut fields = HashMap::new();
        for member in vocabulary.members(EnumKind::EdgeType) {
            let qname = format!("{}/strength", render_member(member));
            fields.insert(
                qname,
                OwnedFieldDecl {
                    decl: FieldDecl {
                        ty: BslType::Coefficient,
                        kind: FieldKind::Extensive,
                    },
                    owner_kind: EnumKind::EdgeType,
                    owner_member: member.clone(),
                    implicit: true,
                },
            );
        }
        Self { fields }
    }

    /// Read one `(deffield <qname> :type <T> :kind <k>)` form into the
    /// registry.
    ///
    /// # Errors
    ///
    /// [`DeclError::Vocabulary`] (`E-LOAD-023`) for an unregistered owning
    /// segment; [`DeclError::Duplicate`] (`E-LOAD-001`) for a second
    /// declaration of one field — including a re-declaration of an implicit
    /// `<edge-type>/strength`; [`DeclError::Malformed`] off the grammar;
    /// §2.13's `E-LOAD-053`/`E-LOAD-054` for a `:type enum` shape violation
    /// or an unresolved `:enum-type` (see this module's own `parse_deffield`).
    pub fn declare(
        &mut self,
        form: &SExpr,
        vocabulary: &ClosedVocabulary,
        enums: &EnumRegistry,
    ) -> Result<(), DeclError> {
        let (qname, ty, kind) = parse_deffield(form, enums)?;
        let (owner_kind, owner_member) = vocabulary.owner_of_field(&qname)?;
        if let Some(existing) = self.fields.get(&qname) {
            let what = if existing.implicit {
                "field (the implicit <edge-type>/strength, D32)"
            } else {
                "field"
            };
            return Err(DeclError::Duplicate { name: qname, what });
        }
        self.fields.insert(
            qname,
            OwnedFieldDecl {
                decl: FieldDecl { ty, kind },
                owner_kind,
                owner_member: owner_member.to_owned(),
                implicit: false,
            },
        );
        Ok(())
    }

    /// Check every declared field against the kernel's model registration
    /// (`E-LOAD-022`, D9: the two must agree and the kernel is checked
    /// against content, not the reverse). Implicit rows are exempt: they
    /// have no authored declaration to disagree with.
    ///
    /// # Errors
    ///
    /// [`DeclError::KernelDisagreement`].
    pub fn check_against_kernel(
        &self,
        kernel: &HashMap<String, FieldDecl>,
    ) -> Result<(), DeclError> {
        let mut names: Vec<&String> = self.fields.keys().collect();
        names.sort(); // deterministic first-failure reporting
        for name in names {
            let owned = &self.fields[name];
            if owned.implicit {
                continue;
            }
            let Some(registered) = kernel.get(name) else {
                continue; // an unregistered field is the vocabulary's business
            };
            if registered.ty != owned.decl.ty || registered.kind != owned.decl.kind {
                return Err(DeclError::KernelDisagreement {
                    field: name.clone(),
                    detail: format!(
                        "content declares {:?}/{:?}, the kernel registers {:?}/{:?}",
                        owned.decl.ty, owned.decl.kind, registered.ty, registered.kind
                    ),
                });
            }
        }
        Ok(())
    }

    /// Look one field up by qname.
    #[must_use]
    pub fn get(&self, qname: &str) -> Option<&OwnedFieldDecl> {
        self.fields.get(qname)
    }

    /// The registry as the §3.4 typechecker's `fields` map.
    #[must_use]
    pub fn type_env_fields(&self) -> HashMap<String, FieldDecl> {
        self.fields
            .iter()
            .map(|(name, owned)| (name.clone(), owned.decl.clone()))
            .collect()
    }

    /// Every declared qname, ascending.
    #[must_use]
    pub fn qnames(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.fields.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

/// Destructure `(deffield <qname> :type <type-name> :kind
/// intensive|extensive)`, or — since §2.13 (D101, Organization spec §1
/// Q12) — `(deffield <qname> :type enum :enum-type <EnumTypeName>)`. The
/// two shapes are mutually exclusive and jointly exhaustive against
/// `:type` (`E-LOAD-053` either way): every `:type` but `enum` requires
/// `:kind` and forbids `:enum-type`; `:type enum` requires `:enum-type`
/// (naming a type `enums` carries) and forbids `:kind`.
fn parse_deffield(
    form: &SExpr,
    enums: &EnumRegistry,
) -> Result<(String, BslType, FieldKind), DeclError> {
    let SExpr::List(items) = form else {
        return Err(malformed("a deffield must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::QName(qname)), options @ ..] =
        items.as_slice()
    else {
        return Err(malformed(
            "(deffield <qname> :type <T> :kind <k>) — unrecognized shape",
        ));
    };
    if head != "deffield" {
        return Err(malformed(format!(
            "expected (deffield …), found ({head} …)"
        )));
    }
    let mut ty_name: Option<&str> = None;
    let mut kind: Option<FieldKind> = None;
    let mut enum_type_name: Option<&str> = None;
    let mut cursor = options;
    while let [SExpr::Atom(Atom::Keyword(kw)), tail @ ..] = cursor {
        match (kw.as_str(), tail) {
            ("type", [SExpr::Atom(Atom::Symbol(name)), rest @ ..]) => {
                ty_name = Some(name.as_str());
                cursor = rest;
            }
            ("kind", [SExpr::Atom(Atom::Symbol(name)), rest @ ..]) => {
                kind = Some(match name.as_str() {
                    "intensive" => FieldKind::Intensive,
                    "extensive" => FieldKind::Extensive,
                    other => {
                        return Err(malformed(format!(
                            ":kind is intensive|extensive, found {other}"
                        )))
                    }
                });
                cursor = rest;
            }
            ("enum-type", [SExpr::Atom(Atom::BareUpperIdent(name)), rest @ ..]) => {
                enum_type_name = Some(name.as_str());
                cursor = rest;
            }
            (other, _) => {
                return Err(malformed(format!(
                    "a deffield takes :type, :kind and :enum-type, found :{other}"
                )))
            }
        }
    }
    if !cursor.is_empty() {
        return Err(malformed(format!(
            "unexpected trailing items in a deffield: {cursor:?}"
        )));
    }
    let Some(ty_name) = ty_name else {
        return Err(malformed("a deffield declares :type (§2.9)"));
    };
    if ty_name == "enum" {
        return parse_enum_deffield(qname, kind, enum_type_name, enums);
    }
    if enum_type_name.is_some() {
        return Err(DeclError::EnumKindShapeViolation {
            field: qname.clone(),
            detail: ":enum-type is legal only alongside :type enum (§2.13)",
        });
    }
    let ty = parse_type_name(ty_name)?;
    match kind {
        Some(kind) => Ok((qname.clone(), ty, kind)),
        None => Err(malformed("a deffield declares both :type and :kind (§2.9)")),
    }
}

/// The `:type enum` half of [`parse_deffield`] — split out because its own
/// exclusivity checks (`E-LOAD-053`) and registry resolution
/// (`E-LOAD-054`) are unrelated to the non-`enum` concrete-type path.
fn parse_enum_deffield(
    qname: &str,
    kind: Option<FieldKind>,
    enum_type_name: Option<&str>,
    enums: &EnumRegistry,
) -> Result<(String, BslType, FieldKind), DeclError> {
    if kind.is_some() {
        return Err(DeclError::EnumKindShapeViolation {
            field: qname.to_owned(),
            detail: ":type enum forbids :kind — an enum-typed field carries \
                     no aggregation kind, there being no meaningful \
                     extensive-or-intensive reading of a member identity \
                     (§2.13)",
        });
    }
    let Some(enum_type_name) = enum_type_name else {
        return Err(DeclError::EnumKindShapeViolation {
            field: qname.to_owned(),
            detail: ":type enum requires :enum-type naming a defenum-declared \
                     type (§2.13)",
        });
    };
    let Some(id) = enums.resolve(enum_type_name) else {
        return Err(DeclError::UnknownEnumRegistryType {
            field: qname.to_owned(),
            enum_type: enum_type_name.to_owned(),
        });
    };
    Ok((
        qname.to_owned(),
        BslType::Enum(id),
        FieldKind::NotApplicable,
    ))
}

/// `(defenum <enum-type> (<enum-member>+))` (§2.13, D101;
/// `bsl.ebnf`: `defenum ::= "(" "defenum" enum-type "(" enum-member+ ")"
/// ")"`, `enum-type`/`enum-member` "§1.4's, unchanged"). Member order is
/// **normative** — it is the declared-order ordinal §3.1's `enum` row
/// stores (delegated to [`EnumRegistry::declare`], which preserves it).
///
/// **Members are written BARE** (`STATE_APPARATUS`, not `OrgKind/
/// STATE_APPARATUS`) — §2.13's own EBNF says so directly, and
/// `tools/tree-sitter-bsl/test/corpus/declarations.txt`'s own worked
/// example pins exactly this shape.
///
/// **#528 fix round, corrected reading.** An earlier version of this
/// function required a full `<enum-type>/<enum-member>` ref repeating the
/// declaring type instead, reasoning that no atom class existed for a
/// standalone `<enum-member>` because `<enum-type>`'s and `<enum-member>`'s
/// charsets overlap (`BUSINESS` fits both). That reasoning was the bug:
/// neither production CONTAINS the other (`OrgKind` has lowercase,
/// `STATE_APPARATUS` has `_`), so [`Atom::BareUpperIdent`] (see its own
/// doc) admits their UNION at lex time and this function disambiguates
/// POSITIONALLY — the type-name operand against
/// [`crate::reader::is_enum_type_shape`], each member against
/// [`crate::reader::is_enum_member_shape`]. A member written as a full
/// enum-ref (with a `/`) is grammar-nonconforming and refuses loudly:
/// grammar conformance cuts both ways.
///
/// # Errors
///
/// [`DeclError::Enum`] for a duplicate type/member (`E-LOAD-001`) or an
/// empty member list; [`DeclError::Malformed`] off the grammar, including a
/// declared type name not shaped like `<enum-type>`, a member not shaped
/// like `<enum-member>`, or a member written as a full enum-ref.
pub fn parse_defenum(form: &SExpr, registry: &mut EnumRegistry) -> Result<EnumTypeId, DeclError> {
    let SExpr::List(items) = form else {
        return Err(malformed("a defenum must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::BareUpperIdent(name)), SExpr::List(member_items)] =
        items.as_slice()
    else {
        return Err(malformed(
            "(defenum <enum-type> (<enum-member>+)) — unrecognized shape",
        ));
    };
    if head != "defenum" {
        return Err(malformed(format!("expected (defenum …), found ({head} …)")));
    }
    if !crate::reader::is_enum_type_shape(name) {
        return Err(malformed(format!(
            "defenum {name}: the declared type name is not a valid \
             <enum-type> (§1.4: UPPER (UPPER|LOWER|DIGIT)* — no underscore)"
        )));
    }
    let mut members = Vec::with_capacity(member_items.len());
    for item in member_items {
        let SExpr::Atom(Atom::BareUpperIdent(member)) = item else {
            return Err(malformed(format!(
                "defenum {name}: member {item:?} must be a bare <enum-member> \
                 (§2.13, §1.4) — e.g. `STATE_APPARATUS`, never a full \
                 `{name}/<MEMBER>` enum-ref"
            )));
        };
        if !crate::reader::is_enum_member_shape(member) {
            return Err(malformed(format!(
                "defenum {name}: member `{member}` is not a valid \
                 <enum-member> (§1.4: UPPER (UPPER|DIGIT|\"_\")* — no lowercase)"
            )));
        }
        members.push(member.clone());
    }
    registry.declare(name, &members).map_err(DeclError::from)
}

/// The §3.1 type names, as they are spelled in `:type` / `:returns`.
///
/// **Ruled (D94; Director, 2026-08-11, ADR191 R4).** This started as a
/// recorded rst gap (R9 step 3, 2026-08-10): §2.9/§2.11/§2.7 write
/// `<type-name>` as a grammar nonterminal and never assign it a §1.4 atom
/// class, while §3.1's own table spelled the types capitalized (`Currency`,
/// `Int`, …). A bare capitalized run matches **no** §1.4 atom class — an
/// `enum-ref` needs its `/`, and `symbol` is lowercase-only — so the
/// spec's own spelling was unlexable as written. This implementation read
/// `<type-name>` as a lowercase `symbol` (`currency`, `int`, …), which is
/// what the crate's pre-existing `(deffield x :type int)` vector already
/// assumed and the only reading §1.4 admits. The Phase-1 review took that
/// reading: §3.1's six declarable rows and §2.11's worked example are now
/// spelled lowercase, so this function matches the spec rather than
/// diverging from it.
///
/// # Errors
///
/// [`DeclError::Malformed`] for a name outside §3.1's closed table.
pub fn parse_type_name(name: &str) -> Result<BslType, DeclError> {
    match name {
        "int" => Ok(BslType::Int),
        "bool" => Ok(BslType::Bool),
        "currency" => Ok(BslType::Currency),
        "probability" => Ok(BslType::Probability),
        "intensity" => Ok(BslType::Intensity),
        "coefficient" => Ok(BslType::Coefficient),
        // Train B item 6 (#591, D155): `real` IS one of §3.1's declarable
        // rows now — the eighth — superseding D98's "not storable" note
        // (the finite binary64 lane as a STORED type, with no
        // declared-domain range law). The intrinsic-decl position still
        // resolves `real` to `IntrinsicTypeName::Real` first
        // (`parse_intrinsic_type_name`'s intercept), so nothing about
        // `<intrinsic-decl>` parsing changes.
        "real" => Ok(BslType::Real),
        // §2.13 (D101): `enum` IS one of §3.1's seven type names now, but
        // this function only ever receives the bare name string — it
        // cannot resolve a companion `:enum-type` operand, which lives in
        // a DIFFERENT keyword slot a caller must have already read. A
        // deffield's own keyword scan special-cases `:type enum` BEFORE
        // reaching here (`parse_enum_deffield`); a metric declaration
        // (§2.11) has no `:enum-type` slot in its grammar at all, so this
        // arm is metrics' own refusal too — deferred, not silently
        // widened.
        "enum" => Err(malformed(
            "'enum' needs a companion :enum-type naming a defenum-declared \
             type (§2.13) that this function was not given — a deffield's \
             own keyword scan resolves :type enum before calling this \
             function, and a metric declaration (§2.11) does not support \
             enum-typed metrics in this slice",
        )),
        other => Err(malformed(format!(
            "'{other}' is not one of §3.1's type names (lowercase — D94 \
             rules type names lowercase symbols)"
        ))),
    }
}

/// Check one `intrinsic` declaration's NAME against the reserved and
/// prohibited sets (D33 / D71, both `E-LOAD-024`).
///
/// # Errors
///
/// [`DeclError::ReservedIntrinsicName`].
pub fn check_intrinsic_name(name: &str) -> Result<(), DeclError> {
    if PROHIBITED_INTRINSIC_NAMES.contains(&name) {
        return Err(DeclError::ReservedIntrinsicName {
            name: name.to_owned(),
            reason: "declaring it would hand content the imposed functional \
                     form ADR172 ruling 5 forbids, pre-packaged (§3.10)",
        });
    }
    if RESERVED_FORM_TAGS.contains(&name) {
        return Err(DeclError::ReservedIntrinsicName {
            name: name.to_owned(),
            reason: "it is a §5.2 form-head symbol, and a colliding intrinsic \
                     would make the form ambiguous with a call (§2.7)",
        });
    }
    Ok(())
}

/// §3.10 gate 1 — *is the intrinsic declarable?* Mechanical, checked at
/// load against [`DECLARABLE_INTRINSICS`].
///
/// **Gate 2 is not mechanical and is not here.** Cap-legality is not
/// doctrine-legality: `exp` sits inside the cap, and three of its five call
/// sites in the frozen estate stipulate a logistic sigmoid that ADR173 and
/// the standing 2026-07-29 no-imposed-functional-forms ruling retire. A
/// verbatim transcription would pass this check and violate the theory
/// line. That gate belongs to Director review, and its question is always
/// the same: *can this be re-derived as a measure instead?* The one part of
/// it that **can** be made mechanical is `sigmoid`'s prohibition, which
/// [`check_intrinsic_name`] enforces.
///
/// The reference names no numbered code for a declaration outside the cap
/// (§3.10's gate-1 sentence cites `E-LOAD-021`, which is the code for a
/// *call* to an undeclared intrinsic), so this error carries none — the
/// no-invented-codes precedent.
///
/// # Errors
///
/// [`DeclError::Malformed`] naming the cap and its authority chain.
pub fn check_intrinsic_cap(name: &str) -> Result<(), DeclError> {
    check_intrinsic_name(name)?;
    if DECLARABLE_INTRINSICS.contains(&name) {
        return Ok(());
    }
    Err(malformed(format!(
        "'{name}' is outside the declarable intrinsic set {DECLARABLE_INTRINSICS:?} \
         (§3.10) — {{exp, log}} capped by R10/ADR176 r21, 'floor' added separately \
         by ADR188 Row 2/D97, and the six \
         exact-arithmetic names ('sqrt', 'round-half-even', 'min', 'max', 'abs', \
         'clamp') added by ADR219 (Director ruling 2026-08-22 — Row 6's fallback \
         rider taken, Row 3 landed, Rows 4/5 superseded). Amendment AJ keeps \
         finite-kernel drawing engine-private; 'rng-draw' is not declarable. \
         Adding a name is a \
         Director ruling, not an authoring decision"
    )))
}

/// The `<type-name>` vocabulary as it applies inside an `<intrinsic-decl>`'s
/// `:params`/`:returns` position — deliberately **separate** from
/// [`parse_type_name`] (`deffield`'s / `metric`'s `:type`), not a widening
/// of it. §3.1 used to rule `Real` "Not storable" (D98: a field or a
/// registered metric could never be `Real`-typed, and `parse_type_name`
/// stayed the six-row table it was); **Train B item 6 (#591, D155)
/// superseded that** — `real` is now §3.1's eighth declarable row, and
/// `parse_type_name` accepts it. What this type still guarantees is the
/// intrinsic position's OWN resolution: the `name == "real"` intercept in
/// [`parse_intrinsic_type_name`] fires BEFORE `parse_type_name` is
/// consulted, so an intrinsic's `real` denotes the unbounded binary64
/// intermediate (§3.3), never a storable field type.
///
/// An intrinsic's argument routinely IS `Real`-typed: every binary64
/// expression's static type is `Real` (§3.3), and before this row there was
/// no way to spell that anywhere in `<intrinsic-decl>`, which left ADR188
/// Row 2's own `floor` rider undeclarable in content — `(intrinsic floor
/// :params (???) :returns int :cost N)` had no legal filler for `:params`.
/// `real` is admitted HERE, and (since Train B item 6) in the storable
/// `<type-name>` position too.
///
/// **Workforce draft ruling, following the D93/D97 Draft-Ruling Register
/// convention** (recorded as the register's next row): `real` is not new
/// mathematics — §3.3/§4.3 already name the binary64 lane's unbounded
/// intermediate kind — so making it spellable in one more grammar position
/// is machinery, not a widening of what the language can express. This
/// reading is not itself a Director ruling; it is open to correction like
/// every other draft-ruling row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrinsicTypeName {
    /// The unbounded binary64 intermediate (§3.3) — legal only in an
    /// `<intrinsic-decl>`'s `:params`/`:returns` position.
    Real,
    /// One of §3.1's storable scalar types (seven since Train B item 6
    /// added `real` to the six D94 closed).
    Scalar(BslType),
}

/// Parse one `<intrinsic-decl>` `:params`/`:returns` type-name token.
///
/// # Errors
///
/// [`DeclError::Malformed`] for a name outside this position's vocabulary
/// (§3.1's rows — eight since Train B item 6 — with `real` intercepted
/// below as the unbounded intermediate, never `Scalar(BslType::Real)`).
pub fn parse_intrinsic_type_name(name: &str) -> Result<IntrinsicTypeName, DeclError> {
    if name == "real" {
        return Ok(IntrinsicTypeName::Real);
    }
    parse_type_name(name).map(IntrinsicTypeName::Scalar)
}

/// A loaded `<intrinsic-decl>` (§2.7): `(intrinsic <symbol> :params
/// (<type-name>*) :returns <type-name> :cost <int-lit>)`.
#[derive(Debug, Clone, PartialEq)]
pub struct IntrinsicDecl {
    /// The declared name — checked against [`DECLARABLE_INTRINSICS`] by
    /// [`parse_intrinsic_decl`] itself, so a caller never sees a decl for a
    /// name outside the cap.
    pub name: String,
    /// Declared parameter types, in source order — checked against
    /// [`kernel_signature`] by [`parse_intrinsic_decl`] itself, so a caller
    /// never sees a decl whose shape disagrees with the kernel's.
    pub params: Vec<IntrinsicTypeName>,
    /// The declared return type — likewise checked.
    pub returns: IntrinsicTypeName,
    /// The declared `:cost` (§2.7's fuel accounting), non-negative.
    pub cost: u64,
}

/// The kernel's registered signature for each name in
/// [`DECLARABLE_INTRINSICS`] — what a content `(intrinsic …)` declaration's
/// own `:params`/`:returns` must match, or `E-LOAD-020` (§2.7: "A
/// declaration whose signature disagrees with the kernel's registration is
/// `E-LOAD-020`"). `floor`'s is `(real) → int` (ADR188 Row 2, D97); the
/// transcendental pair's is the ordinary one-`Real`-argument mathematical
/// signature — `exp`/`log` both dispatch through `intrinsic_host::
/// KernelIntrinsicHost` since Task 2 of the #576 intrinsic-host train.
///
/// The ADR219 sextet (Director ruling 2026-08-22) is all-`real`-lane:
/// `sqrt`/`round-half-even`/`abs` unary, `min`/`max` binary, `clamp`
/// ternary, every one returning `real`. No `int` demotion anywhere in the
/// six — the Real→Int crossing remains `floor`'s alone (ADR188 Row 2), and
/// `round-half-even` specifically rounds WITHIN the binary64 lane (the
/// D-row reading of §3.2's obligation, §3.10's normative paragraph).
///
/// `None` for a name outside `DECLARABLE_INTRINSICS` is unreachable through
/// [`parse_intrinsic_decl`] (the cap check runs first), and a name INSIDE
/// the cap with no row here is an internal inconsistency this crate keeps
/// as an invariant — [`parse_intrinsic_decl`] refuses loudly rather than
/// silently skipping the check, so a future cap widening that forgot to
/// register a signature fails a test rather than shipping unchecked.
#[must_use]
pub fn kernel_signature(name: &str) -> Option<(Vec<IntrinsicTypeName>, IntrinsicTypeName)> {
    match name {
        "floor" => Some((
            vec![IntrinsicTypeName::Real],
            IntrinsicTypeName::Scalar(BslType::Int),
        )),
        "exp" | "log" => Some((vec![IntrinsicTypeName::Real], IntrinsicTypeName::Real)),
        // The ADR219 sextet — see this function's doc above. Unary first,
        // then binary, then the one ternary.
        "sqrt" | "round-half-even" | "abs" => {
            Some((vec![IntrinsicTypeName::Real], IntrinsicTypeName::Real))
        }
        "min" | "max" => Some((
            vec![IntrinsicTypeName::Real, IntrinsicTypeName::Real],
            IntrinsicTypeName::Real,
        )),
        "clamp" => Some((
            vec![
                IntrinsicTypeName::Real,
                IntrinsicTypeName::Real,
                IntrinsicTypeName::Real,
            ],
            IntrinsicTypeName::Real,
        )),
        _ => None,
    }
}

/// The `:params`/`:returns`/`:cost` clause loop that [`parse_intrinsic_decl`]
/// factors out to stay under this crate's line-count discipline (Power-of-10
/// rule 3). Each keyword is legal at most once — a repeat is refused loudly
/// (the second occurrence would otherwise silently win, and for `:cost`
/// would silently change fuel accounting).
///
/// # Errors
///
/// [`DeclError`] for a repeated keyword, an unrecognized `:params`/
/// `:returns` type-name, a negative `:cost`, an unrecognized clause, or a
/// missing `:params`/`:returns`/`:cost`.
fn parse_intrinsic_clauses(
    rest: &[SExpr],
) -> Result<(Vec<IntrinsicTypeName>, IntrinsicTypeName, u64), DeclError> {
    let mut params: Option<Vec<IntrinsicTypeName>> = None;
    let mut returns: Option<IntrinsicTypeName> = None;
    let mut cost: Option<u64> = None;
    let mut cursor = rest;
    while !cursor.is_empty() {
        match cursor {
            [SExpr::Atom(Atom::Keyword(kw)), SExpr::List(inner), tail @ ..] if kw == "params" => {
                if params.is_some() {
                    return Err(malformed(
                        ":params is declared twice in one intrinsic form — the \
                         second occurrence would silently win",
                    ));
                }
                let mut parsed = Vec::with_capacity(inner.len());
                for item in inner {
                    let SExpr::Atom(Atom::Symbol(type_name)) = item else {
                        return Err(malformed(":params takes a list of type-name symbols"));
                    };
                    parsed.push(parse_intrinsic_type_name(type_name)?);
                }
                params = Some(parsed);
                cursor = tail;
            }
            [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Symbol(value)), tail @ ..]
                if kw == "returns" =>
            {
                if returns.is_some() {
                    return Err(malformed(
                        ":returns is declared twice in one intrinsic form — the \
                         second occurrence would silently win",
                    ));
                }
                returns = Some(parse_intrinsic_type_name(value)?);
                cursor = tail;
            }
            [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Int(n)), tail @ ..]
                if kw == "cost" =>
            {
                if cost.is_some() {
                    return Err(malformed(
                        ":cost is declared twice in one intrinsic form — the \
                         second occurrence would silently win (and silently \
                         change fuel accounting)",
                    ));
                }
                cost =
                    Some(u64::try_from(*n).map_err(|_| {
                        malformed(format!("a negative :cost ({n}) is meaningless"))
                    })?);
                cursor = tail;
            }
            other => {
                return Err(malformed(format!(
                    "an intrinsic declaration takes :params, :returns and :cost, \
                     found {:?}",
                    other.first()
                )))
            }
        }
    }
    let (Some(params), Some(returns), Some(cost)) = (params, returns, cost) else {
        return Err(malformed(
            "an intrinsic declaration must carry :params, :returns and :cost (§2.7)",
        ));
    };
    Ok((params, returns, cost))
}

/// Parse one `(intrinsic …)` top-form (§2.2/§2.7):
///
/// 1. the §3.10 cap check on its name ([`check_intrinsic_cap`]) — a
///    declaration for a name outside [`DECLARABLE_INTRINSICS`] is refused
///    HERE, at content-load time, not admitted into an [`IntrinsicDecl`]
///    and left for some later gate to notice;
/// 2. each of `:params`/`:returns`/`:cost` at most ONCE — a repeated
///    keyword inside one declaration is refused loudly rather than the
///    last occurrence silently winning (the same III.11 class as a
///    duplicate declaration across a content set, just at a smaller
///    radius: see [`parse_intrinsic_decls`] for that one);
/// 3. the declared signature against [`kernel_signature`] — `E-LOAD-020`
///    on any disagreement (wrong arity, wrong parameter type, or wrong
///    return type all land here, since a `Vec` compares by length and by
///    element).
///
/// # Errors
///
/// [`DeclError`] for: a malformed form shape; a reserved/prohibited/
/// uncapped name ([`check_intrinsic_cap`]); a repeated `:params`/
/// `:returns`/`:cost` keyword; an unrecognized `:params`/`:returns`
/// type-name ([`parse_intrinsic_type_name`]); a negative `:cost`; a
/// missing `:params`/`:returns`/`:cost`; or a signature disagreeing with
/// [`kernel_signature`] (`E-LOAD-020`).
pub fn parse_intrinsic_decl(form: &SExpr) -> Result<IntrinsicDecl, DeclError> {
    let SExpr::List(items) = form else {
        return Err(malformed("an intrinsic declaration must be a form"));
    };
    let [SExpr::Atom(Atom::Symbol(head)), SExpr::Atom(Atom::Symbol(name)), rest @ ..] =
        items.as_slice()
    else {
        return Err(malformed(
            "(intrinsic <symbol> :params (<type-name>*) :returns <type-name> \
             :cost <int-lit>) — unrecognized shape",
        ));
    };
    if head != "intrinsic" {
        return Err(malformed(format!(
            "expected (intrinsic …), found ({head} …)"
        )));
    }
    check_intrinsic_cap(name)?;
    let (params, returns, cost) = parse_intrinsic_clauses(rest)?;
    let (expected_params, expected_returns) = kernel_signature(name).ok_or_else(|| {
        malformed(format!(
            "'{name}' is declarable (§3.10) but has no registered kernel signature \
             to check against — an internal inconsistency, not a content error"
        ))
    })?;
    if params != expected_params || returns != expected_returns {
        return Err(DeclError::SignatureMismatch {
            name: name.clone(),
            detail: format!(
                "declared ({params:?}) -> {returns:?}, the kernel registers \
                 ({expected_params:?}) -> {expected_returns:?}"
            ),
        });
    }
    Ok(IntrinsicDecl {
        name: name.clone(),
        params,
        returns,
        cost,
    })
}

/// Parse a content set's `(intrinsic …)` top-forms into a name-keyed table,
/// refusing a duplicate name `E-LOAD-001` (§2.2: "duplicate intrinsic
/// declarations" is normatively listed alongside duplicate rule ids and
/// duplicate field declarations) rather than letting the last declaration
/// silently win — the same silent-load inversion a `HashMap::insert` over
/// the raw forms would commit, at content-set radius rather than
/// single-form radius (that one is [`parse_intrinsic_decl`]'s own
/// repeated-keyword check).
///
/// # Errors
///
/// [`DeclError`] from [`parse_intrinsic_decl`] on the first malformed
/// declaration; [`DeclError::Duplicate`] (`E-LOAD-001`) on a second
/// declaration of one name.
pub fn parse_intrinsic_decls(forms: &[SExpr]) -> Result<HashMap<String, IntrinsicDecl>, DeclError> {
    let mut decls = HashMap::with_capacity(forms.len());
    for form in forms {
        let decl = parse_intrinsic_decl(form)?;
        if decls.contains_key(&decl.name) {
            return Err(DeclError::Duplicate {
                name: decl.name,
                what: "intrinsic",
            });
        }
        decls.insert(decl.name.clone(), decl);
    }
    Ok(decls)
}

#[cfg(test)]
mod tests {
    use super::{
        check_intrinsic_cap, check_intrinsic_name, kernel_signature, parse_intrinsic_decl,
        parse_intrinsic_type_name, DeclError, FieldRegistry, IntrinsicDecl, IntrinsicTypeName,
        DECLARABLE_INTRINSICS, RESERVED_FORM_TAGS,
    };
    use crate::reader::{read, SExpr};
    use crate::types::{BslType, EnumRegistry, EnumRegistryError, FieldDecl, FieldKind};
    use crate::vocabulary::{ClosedVocabulary, EnumKind};
    use std::collections::HashMap;

    /// No content in this module's tests declares an enum-typed field —
    /// an empty registry is the honest "no `defenum`s in scope" input.
    fn enums() -> EnumRegistry {
        EnumRegistry::default()
    }

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
        ])
        .unwrap()
    }

    fn form(source: &str) -> crate::reader::SExpr {
        read(source).expect("test source must parse").0
    }

    #[test]
    fn every_edge_type_carries_an_implicit_extensive_coefficient_strength() {
        let registry = FieldRegistry::with_implicit_edge_strength(&vocabulary());
        let strength = registry.get("solidarity/strength").expect("D32");
        assert_eq!(strength.decl.ty, BslType::Coefficient);
        assert_eq!(
            strength.decl.kind,
            FieldKind::Extensive,
            "the extensive kind is what makes §2.4's sum_strength row \
             honourable under §3.4 without an exemption (D32)"
        );
        assert!(strength.implicit);
        assert!(registry.get("exploitation/strength").is_some());
    }

    #[test]
    fn redeclaring_the_implicit_strength_field_is_e_load_001() {
        let mut registry = FieldRegistry::with_implicit_edge_strength(&vocabulary());
        let err = registry
            .declare(
                &form("(deffield solidarity/strength :type coefficient :kind extensive)"),
                &vocabulary(),
                &enums(),
            )
            .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    #[test]
    fn a_deffield_may_own_off_a_node_edge_or_hyperedge_type() {
        let v = vocabulary();
        let mut registry = FieldRegistry::default();
        for source in [
            "(deffield social-class/wealth :type currency :kind extensive)",
            "(deffield exploitation/tension :type intensity :kind intensive)",
            "(deffield economic-sector/output :type currency :kind extensive)",
        ] {
            registry.declare(&form(source), &v, &enums()).expect(source);
        }
        assert_eq!(
            registry.get("exploitation/tension").unwrap().owner_kind,
            EnumKind::EdgeType
        );
        assert_eq!(
            registry.get("economic-sector/output").unwrap().owner_kind,
            EnumKind::HyperedgeType
        );
    }

    #[test]
    fn a_deffield_owning_off_no_registered_type_is_e_load_023() {
        let mut registry = FieldRegistry::default();
        let err = registry
            .declare(
                &form("(deffield imperium/rent :type currency :kind extensive)"),
                &vocabulary(),
                &enums(),
            )
            .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-023"));
    }

    #[test]
    fn a_duplicate_authored_field_is_e_load_001() {
        let v = vocabulary();
        let mut registry = FieldRegistry::default();
        let source = "(deffield social-class/wealth :type currency :kind extensive)";
        registry.declare(&form(source), &v, &enums()).unwrap();
        assert_eq!(
            registry
                .declare(&form(source), &v, &enums())
                .unwrap_err()
                .spec_code(),
            Some("E-LOAD-001")
        );
    }

    #[test]
    fn kernel_disagreement_is_e_load_022() {
        let v = vocabulary();
        let mut registry = FieldRegistry::default();
        registry
            .declare(
                &form("(deffield social-class/wealth :type currency :kind extensive)"),
                &v,
                &enums(),
            )
            .unwrap();
        let kernel = HashMap::from([(
            "social-class/wealth".to_owned(),
            FieldDecl {
                ty: BslType::Currency,
                kind: FieldKind::Intensive, // the kernel says intensive
            },
        )]);
        assert_eq!(
            registry
                .check_against_kernel(&kernel)
                .unwrap_err()
                .spec_code(),
            Some("E-LOAD-022")
        );
    }

    #[test]
    fn every_form_head_symbol_is_reserved_against_the_intrinsic_namespace() {
        for tag in RESERVED_FORM_TAGS {
            let err = check_intrinsic_name(tag).unwrap_err();
            assert_eq!(err.spec_code(), Some("E-LOAD-024"), "{tag}");
        }
        assert_eq!(check_intrinsic_name("exp"), Ok(()));
        assert_eq!(check_intrinsic_name("log"), Ok(()));
    }

    #[test]
    fn sigmoid_is_prohibited_outright_not_merely_undeclared() {
        let err = check_intrinsic_name("sigmoid").unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-024"));
        assert!(format!("{err}").contains("ADR172"), "{err}");
    }

    /// ADR188 Row 2 (the intrinsic-cap rider slate, Director-disposed
    /// 2026-08-10): `floor` joins the declarable set. Named `floor`, not
    /// `trunc` — the two conventions coincide on the ratified domain
    /// (`[0, ∞)`), and this crate declares only the one name a rule may
    /// call. The ratified call sites (`vitality.py`'s `_calculate_deaths`,
    /// `decomposition.py`'s population splits) guard population `> 0` and
    /// clamp/constrain the multiplied rate to `[0, 1]` before the
    /// demotion, so the argument is always non-negative there — not a
    /// claim of §3.4 (the intensivity kind rule, which asserts nothing
    /// about sign).
    #[test]
    fn floor_is_declarable_under_adr188_row_2() {
        assert_eq!(check_intrinsic_name("floor"), Ok(()));
        assert_eq!(check_intrinsic_cap("floor"), Ok(()));
        // The array grew from four names to ten under ADR219 (Director
        // ruling 2026-08-22) — the exact-arithmetic rider train; see the
        // c13 family in r9_chapters.rs for the full authority chain.
        assert_eq!(
            DECLARABLE_INTRINSICS,
            [
                "exp",
                "log",
                "floor",
                "sqrt",
                "round-half-even",
                "min",
                "max",
                "abs",
                "clamp"
            ]
        );
    }

    /// Amendment AJ retires the author-visible RNG path. The identifier is
    /// lexically valid, but it is outside the declaration cap and has no
    /// kernel signature.
    #[test]
    fn rng_draw_is_not_author_visible_after_amendment_aj() {
        assert_eq!(check_intrinsic_name("rng-draw"), Ok(()));
        assert!(check_intrinsic_cap("rng-draw").is_err());
        assert_eq!(kernel_signature("rng-draw"), None);
    }

    /// The rider ratifies `floor`, not a second `trunc` intrinsic — ADR188
    /// Row 2 says "a Real→Int demotion path" (singular). A future PR that
    /// silently widened the cap to both names would flip this test.
    #[test]
    fn trunc_is_not_a_separate_declarable_name() {
        assert!(check_intrinsic_cap("trunc").is_err());
    }

    /// ADR219 (Director ruling 2026-08-22): the exact-arithmetic rider
    /// train. `sqrt` takes ADR188 Row 6's fallback rider; `round-half-even`
    /// lands Row 3's ratified housekeeping rider (D70's "recorded, not
    /// resolved" is resolved); `min`/`max`/`abs`/`clamp` supersede Rows
    /// 4/5's "no rider" dispositions on #591 item 2's accumulated evidence.
    /// All six cross via IEEE-754 exactly-specified operations — no
    /// transcendental, no pinned soft-float libm, no §4.3 golden-vector
    /// family (the D97 disposition, recorded per-name in §3.10). This test
    /// supersedes `round_half_even_still_sits_outside_the_cap_after_the_
    /// floor_rider`, whose premise ADR219 retires.
    #[test]
    fn the_adr219_six_are_declarable_with_their_kernel_signatures() {
        for (name, params) in [
            ("sqrt", vec![IntrinsicTypeName::Real]),
            ("round-half-even", vec![IntrinsicTypeName::Real]),
            ("abs", vec![IntrinsicTypeName::Real]),
            (
                "min",
                vec![IntrinsicTypeName::Real, IntrinsicTypeName::Real],
            ),
            (
                "max",
                vec![IntrinsicTypeName::Real, IntrinsicTypeName::Real],
            ),
            (
                "clamp",
                vec![
                    IntrinsicTypeName::Real,
                    IntrinsicTypeName::Real,
                    IntrinsicTypeName::Real,
                ],
            ),
        ] {
            assert_eq!(check_intrinsic_cap(name), Ok(()), "{name}");
            assert_eq!(
                kernel_signature(name),
                Some((params, IntrinsicTypeName::Real)),
                "{name}"
            );
        }
    }

    // ---- parse_intrinsic_decl (the declared-cost seam content needs) ----

    fn decl(source: &str) -> Result<IntrinsicDecl, DeclError> {
        let (form, _) = read(source).expect("test source must parse");
        parse_intrinsic_decl(&form)
    }

    #[test]
    fn a_well_formed_floor_declaration_parses() {
        let parsed = decl("(intrinsic floor :params (real) :returns int :cost 5)").unwrap();
        assert_eq!(parsed.name, "floor");
        assert_eq!(parsed.params, vec![IntrinsicTypeName::Real]);
        assert_eq!(parsed.returns, IntrinsicTypeName::Scalar(BslType::Int));
        assert_eq!(parsed.cost, 5);
    }

    /// `real` resolves PER POSITION. Train B item 6 (#591, D155) superseded
    /// this test's original second half — D98's "not storable" meant
    /// `parse_type_name("real")` was an error; `real` is now §3.1's eighth
    /// declarable row, so both parsers accept it. What survives unchanged
    /// is the intrinsic position's own resolution: the `name == "real"`
    /// intercept fires first, so an `<intrinsic-decl>` still gets
    /// `IntrinsicTypeName::Real`, never `Scalar(BslType::Real)`.
    #[test]
    fn real_resolves_per_position_after_train_b_item_6() {
        assert_eq!(
            parse_intrinsic_type_name("real"),
            Ok(IntrinsicTypeName::Real)
        );
        assert_eq!(super::parse_type_name("real"), Ok(BslType::Real));
    }

    /// The cap check runs AS PART OF the parse, not after — a declaration
    /// for `tanh` (ratified nowhere) never becomes an `IntrinsicDecl`.
    #[test]
    fn a_declaration_for_a_name_outside_the_cap_is_refused_at_parse_time() {
        let err = decl("(intrinsic tanh :params (real) :returns real :cost 40)").unwrap_err();
        assert!(format!("{err}").contains("outside the declarable intrinsic set"));
    }

    #[test]
    fn a_negative_cost_is_refused_never_reinterpreted() {
        assert!(decl("(intrinsic floor :params (real) :returns int :cost -1)").is_err());
    }

    #[test]
    fn a_missing_clause_is_refused() {
        assert!(decl("(intrinsic floor :params (real) :returns int)").is_err());
        assert!(decl("(intrinsic floor :returns int :cost 5)").is_err());
        assert!(decl("(intrinsic floor :params (real) :cost 5)").is_err());
    }

    #[test]
    fn an_unrecognized_params_type_name_is_refused() {
        assert!(decl("(intrinsic floor :params (nonsense) :returns int :cost 5)").is_err());
    }

    // ---- E-LOAD-020: declared signature vs. kernel_signature (FR-3) ----

    #[test]
    fn a_wrong_returns_type_is_e_load_020() {
        // floor's kernel signature returns int, not real.
        let err = decl("(intrinsic floor :params (real) :returns real :cost 5)").unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-020"));
    }

    #[test]
    fn a_wrong_arity_is_e_load_020() {
        // floor takes exactly one parameter.
        let zero = decl("(intrinsic floor :params () :returns int :cost 5)").unwrap_err();
        assert_eq!(zero.spec_code(), Some("E-LOAD-020"));
        let two = decl("(intrinsic floor :params (real real) :returns int :cost 5)").unwrap_err();
        assert_eq!(two.spec_code(), Some("E-LOAD-020"));
    }

    #[test]
    fn a_wrong_param_type_is_e_load_020() {
        // floor's kernel signature takes `real`, not `int`.
        let err = decl("(intrinsic floor :params (int) :returns int :cost 5)").unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-020"));
    }

    #[test]
    fn exp_and_log_have_the_ordinary_one_real_argument_signature() {
        assert!(decl("(intrinsic exp :params (real) :returns real :cost 40)").is_ok());
        assert!(decl("(intrinsic log :params (real) :returns real :cost 40)").is_ok());
        assert_eq!(
            decl("(intrinsic exp :params (int) :returns real :cost 40)")
                .unwrap_err()
                .spec_code(),
            Some("E-LOAD-020")
        );
    }

    // ---- repeated keyword inside ONE declaration (FR-7) ----

    #[test]
    fn a_repeated_cost_keyword_is_refused_never_last_one_wins() {
        let err =
            decl("(intrinsic floor :params (real) :returns int :cost 5 :cost 9)").unwrap_err();
        assert!(format!("{err}").contains("declared twice"), "{err}");
    }

    #[test]
    fn a_repeated_returns_keyword_is_refused() {
        assert!(
            decl("(intrinsic floor :params (real) :returns int :returns real :cost 5)").is_err()
        );
    }

    #[test]
    fn a_repeated_params_keyword_is_refused() {
        assert!(
            decl("(intrinsic floor :params (real) :params (real) :returns int :cost 5)").is_err()
        );
    }

    // ---- parse_intrinsic_decls: duplicate NAME across a content set (FR-2) ----

    fn forms(source: &str) -> Vec<SExpr> {
        crate::reader::read_all(source.as_bytes()).expect("test source must parse")
    }

    #[test]
    fn a_duplicate_intrinsic_name_across_a_content_set_is_e_load_001_not_last_one_wins() {
        let two_decls = forms(
            "(intrinsic floor :params (real) :returns int :cost 5) \
             (intrinsic floor :params (real) :returns int :cost 9)",
        );
        let err = super::parse_intrinsic_decls(&two_decls).unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    #[test]
    fn a_content_set_with_one_declaration_per_name_loads_both() {
        let two_decls = forms(
            "(intrinsic floor :params (real) :returns int :cost 5) \
             (intrinsic exp :params (real) :returns real :cost 40)",
        );
        let decls = super::parse_intrinsic_decls(&two_decls).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls["floor"].cost, 5);
        assert_eq!(decls["exp"].cost, 40);
    }

    // ---- §2.13 (Organization contract, Q12, D101): `defenum` and the
    // `:type enum :enum-type` deffield shape ----

    fn org_kind() -> super::EnumRegistry {
        let mut registry = super::EnumRegistry::default();
        super::parse_defenum(
            &form(
                "(defenum OrgKind (STATE_APPARATUS BUSINESS \
                 POLITICAL_FACTION CIVIL_SOCIETY))",
            ),
            &mut registry,
        )
        .expect("OrgKind declares");
        registry
    }

    /// The tree-sitter corpus's own worked example
    /// (`test/corpus/declarations.txt:144`) — bare `<enum-member>` items,
    /// never full `Type/MEMBER` refs (#528 fix round).
    #[test]
    fn parse_defenum_accepts_the_corpus_line_verbatim() {
        let mut registry = super::EnumRegistry::default();
        let ty = super::parse_defenum(
            &form("(defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION CIVIL_SOCIETY))"),
            &mut registry,
        )
        .expect("the corpus's own bare-member shape must load");
        assert_eq!(registry.ordinal(ty, "STATE_APPARATUS"), Some(0));
        assert_eq!(registry.ordinal(ty, "BUSINESS"), Some(1));
        assert_eq!(registry.ordinal(ty, "POLITICAL_FACTION"), Some(2));
        assert_eq!(registry.ordinal(ty, "CIVIL_SOCIETY"), Some(3));
    }

    #[test]
    fn defenum_preserves_declaration_order_as_the_ordinal() {
        let registry = org_kind();
        let ty = registry.resolve("OrgKind").expect("OrgKind is declared");
        assert_eq!(registry.ordinal(ty, "STATE_APPARATUS"), Some(0));
        assert_eq!(registry.ordinal(ty, "BUSINESS"), Some(1));
        assert_eq!(registry.ordinal(ty, "POLITICAL_FACTION"), Some(2));
        assert_eq!(registry.ordinal(ty, "CIVIL_SOCIETY"), Some(3));
    }

    /// #528 fix round: repurposed from `defenum_refuses_a_member_naming_a_
    /// different_type` — under the bare-member reading a member carries no
    /// type prefix to mismatch AT ALL (that whole error class is now
    /// structurally unreachable), so the meaningful sibling check is the
    /// grammar-conformance direction: a member written as a full enum-ref
    /// (even one naming a plausible-looking different type) still refuses.
    #[test]
    fn defenum_refuses_a_member_written_as_a_full_enum_ref() {
        let mut registry = super::EnumRegistry::default();
        let err = super::parse_defenum(
            &form("(defenum OrgKind (NodeType/SOCIAL_CLASS))"),
            &mut registry,
        )
        .unwrap_err();
        assert!(matches!(err, DeclError::Malformed { .. }), "{err:?}");
    }

    #[test]
    fn defenum_refuses_a_type_name_shaped_like_an_enum_member() {
        // Org_Kind lexes fine (Atom::BareUpperIdent admits the union
        // charset) but is not a valid <enum-type> (underscore) — the
        // parser, not the reader, must catch this.
        let mut registry = super::EnumRegistry::default();
        let err = super::parse_defenum(&form("(defenum Org_Kind (BUSINESS))"), &mut registry)
            .unwrap_err();
        assert!(matches!(err, DeclError::Malformed { .. }), "{err:?}");
    }

    #[test]
    fn defenum_refuses_a_member_shaped_like_an_enum_type() {
        // `Business` lexes fine but is not a valid <enum-member>
        // (lowercase) — same split, the other direction.
        let mut registry = super::EnumRegistry::default();
        let err =
            super::parse_defenum(&form("(defenum OrgKind (Business))"), &mut registry).unwrap_err();
        assert!(matches!(err, DeclError::Malformed { .. }), "{err:?}");
    }

    #[test]
    fn defenum_refuses_an_empty_member_list_as_e_load_001_family() {
        // The grammar's <enum-member>+ is one-or-more; an empty list is a
        // shape violation the registry reports as EmptyMemberList (uncoded
        // — never a duplicate-declaration lie).
        let mut registry = super::EnumRegistry::default();
        let err = super::parse_defenum(&form("(defenum OrgKind ())"), &mut registry).unwrap_err();
        assert_eq!(err.spec_code(), None);
        assert!(matches!(
            err,
            DeclError::Enum(EnumRegistryError::EmptyMemberList { .. })
        ));
    }

    #[test]
    fn a_duplicate_defenum_type_name_is_e_load_001() {
        let mut registry = super::EnumRegistry::default();
        super::parse_defenum(&form("(defenum OrgKind (BUSINESS))"), &mut registry).unwrap();
        let err = super::parse_defenum(&form("(defenum OrgKind (CIVIL_SOCIETY))"), &mut registry)
            .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-001"));
    }

    #[test]
    fn deffield_type_enum_requires_enum_type_and_resolves_it() {
        let registry = org_kind();
        let (qname, ty, kind) = super::parse_deffield(
            &form("(deffield organization/kind :type enum :enum-type OrgKind)"),
            &registry,
        )
        .expect("well-formed enum deffield");
        assert_eq!(qname, "organization/kind");
        assert_eq!(kind, FieldKind::NotApplicable);
        let BslType::Enum(id) = ty else {
            panic!("expected BslType::Enum, got {ty:?}")
        };
        assert_eq!(id, registry.resolve("OrgKind").unwrap());
    }

    #[test]
    fn deffield_type_enum_without_enum_type_is_e_load_053() {
        let registry = org_kind();
        let err =
            super::parse_deffield(&form("(deffield organization/kind :type enum)"), &registry)
                .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-053"));
    }

    #[test]
    fn deffield_type_enum_with_kind_is_e_load_053() {
        let registry = org_kind();
        let err = super::parse_deffield(
            &form(
                "(deffield organization/kind :type enum :enum-type OrgKind \
                 :kind extensive)",
            ),
            &registry,
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-053"));
    }

    #[test]
    fn enum_type_keyword_on_a_non_enum_type_is_e_load_053() {
        let registry = org_kind();
        let err = super::parse_deffield(
            &form(
                "(deffield organization/budget :type currency :kind extensive \
                 :enum-type OrgKind)",
            ),
            &registry,
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-053"));
    }

    #[test]
    fn deffield_type_enum_naming_an_unregistered_type_is_e_load_054() {
        let registry = super::EnumRegistry::default();
        let err = super::parse_deffield(
            &form("(deffield organization/kind :type enum :enum-type Nowhere)"),
            &registry,
        )
        .unwrap_err();
        assert_eq!(err.spec_code(), Some("E-LOAD-054"));
    }
}
