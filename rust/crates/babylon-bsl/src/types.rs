//! The BSL type universe (spec §5 Types, `bsl-language.rst` §3): kernel
//! scalars, closed enums, booleans, `Int`, and typed node/edge-set
//! references. Intensivity is a per-field DECLARATION (`:kind
//! intensive|extensive` on `deffield` forms, §3.4), not a property of the
//! scalar type — `wealth` (Currency) is extensive while `consciousness`
//! (Intensity) is intensive, and no type makes that decidable. `FieldKind`
//! therefore travels alongside a field's `BslType`, and the typechecker
//! (not the type) enforces the aggregation law.
//!
//! **§2.13 addendum (Director ruling 2026-08-11, Organization-as-game-object
//! spec §1 Q12 — approved twice, live popup and written spec review; D101).**
//! `BslType::Enum` widens from a bare `&'static str` name to an
//! [`EnumTypeId`] — a content-declared closed `defenum` type, stored as the
//! declared-order ordinal in the existing binary64 attribute lane. The
//! ordinal is never a surface value: written and read only as an
//! `<EnumType>/<MEMBER>` enum-ref (§1.4), comparable with `=`/`!=` only.
//! [`EnumRegistry`] is the registry backing it — deliberately NOT
//! [`crate::vocabulary::ClosedVocabulary`], which sorts-and-dedups its
//! members by design (`vocabulary.rs`): correct for a set with no ordinal
//! to preserve, and wrong for one where declaration order IS the stored
//! value (bsl-language.rst §2.13).

/// A closed `defenum` type's identity within one [`EnumRegistry`] — the
/// registry's own index, `Copy` so it travels through `BslType`/`FieldDecl`
/// exactly as freely as any other scalar type tag.
///
/// An `EnumTypeId` is only ever meaningful against the [`EnumRegistry`]
/// that minted it (`EnumRegistry::declare`'s return value) — using one
/// against a DIFFERENT registry instance is a caller bug, the same
/// out-of-band-identity risk `babylon_graph`'s own id newtypes (`NodeId`,
/// …) already carry and document there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumTypeId(pub u32);

/// One `defenum` declaration: its type name and its members, in the
/// **declaration order** that is normative (bsl-language.rst §2.13: "member
/// order inside a `defenum` is normative — it is the ordinal §3.1's `enum`
/// row stores").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    /// The `defenum` type name, e.g. `"OrgKind"`.
    pub name: String,
    /// The declared members, in declaration order — index `i` IS ordinal
    /// `i` (§3.1's `enum` row).
    pub members: Vec<String>,
}

/// A `defenum` declaration-time rejection (§2.13). Reused, not reinvented:
/// wrapped by [`crate::declarations::DeclError`] via `From`, which is what
/// assigns the spec code — this type carries no code of its own, matching
/// [`crate::vocabulary::VocabularyError`]'s own layering (a lower-layer
/// module states WHAT went wrong; the higher-layer caller states which
/// spec code that is, since the same underlying shape can serve different
/// codes at different call sites).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumRegistryError {
    /// A `defenum` type name declared twice in one content set.
    DuplicateType {
        /// The offending type name.
        name: String,
    },
    /// A member repeated within one `defenum` declaration.
    DuplicateMember {
        /// The declaring type name.
        name: String,
        /// The repeated member.
        member: String,
    },
    /// A `defenum` with zero members — the grammar's `<enum-member>+`
    /// (bsl.ebnf, §2.13) is one-or-more, never zero.
    EmptyMemberList {
        /// The offending type name.
        name: String,
    },
}

impl std::fmt::Display for EnumRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateType { name } => {
                write!(f, "duplicate defenum type name: {name}")
            }
            Self::DuplicateMember { name, member } => {
                write!(f, "defenum {name}: member {member} declared twice")
            }
            Self::EmptyMemberList { name } => write!(
                f,
                "defenum {name}: at least one member is required (§2.13's \
                 <enum-member>+ is one-or-more, never zero)"
            ),
        }
    }
}

impl std::error::Error for EnumRegistryError {}

/// The content-declared closed-enum registry (§2.13, D101) — every
/// `defenum` type one content set declared, keyed by [`EnumTypeId`].
///
/// A plain `Vec`, deliberately: declaration order IS the storage (the
/// registry's own index doubles as the ordinal §3.1's `enum` row persists),
/// so nothing here may reorder or deduplicate-by-sort the way
/// [`crate::vocabulary::ClosedVocabulary`] does for the structural kinds —
/// see this module's own header. Member lookup is a linear scan; a
/// `defenum`'s member count is a handful by construction (four for
/// `OrgKind`'s worked example), so this is not a hot path.
#[derive(Debug, Clone, Default)]
pub struct EnumRegistry {
    types: Vec<EnumDecl>,
}

impl EnumRegistry {
    /// Declare one `defenum` type. Rejects a duplicate type name, a
    /// duplicate member within the same declaration, and an empty member
    /// list — never silently drops or reorders.
    ///
    /// # Errors
    ///
    /// [`EnumRegistryError::EmptyMemberList`] / [`EnumRegistryError::DuplicateType`]
    /// / [`EnumRegistryError::DuplicateMember`].
    pub fn declare(
        &mut self,
        name: &str,
        members: &[String],
    ) -> Result<EnumTypeId, EnumRegistryError> {
        if members.is_empty() {
            return Err(EnumRegistryError::EmptyMemberList {
                name: name.to_owned(),
            });
        }
        // Train B item 4 (#591, D157, the prelude-sharing seam): an
        // IDENTICAL re-declaration — same name, same members, in the same
        // order — is not a conflict; it is the same fact stated twice (a
        // scenario re-declaring what its prelude already declared).
        // `EnumDecl` already derives `PartialEq`/`Eq`, and `Vec<String>:
        // PartialEq` is exact-order, exact-length element comparison for
        // free, so recognizing this needs no new comparison logic — only
        // the call `declare` never made. A DIFFERING member list under the
        // same name (reordered, renamed, added, or dropped) still refuses
        // exactly as before: this is recognition of an identical fact, not
        // a merge or an override.
        if let Some(existing) = self.types.iter().position(|d| d.name == name) {
            if self.types[existing].members.as_slice() == members {
                #[allow(clippy::cast_possible_truncation)]
                let id = existing as u32;
                return Ok(EnumTypeId(id));
            }
            return Err(EnumRegistryError::DuplicateType {
                name: name.to_owned(),
            });
        }
        let mut seen: Vec<&str> = Vec::with_capacity(members.len());
        for member in members {
            if seen.contains(&member.as_str()) {
                return Err(EnumRegistryError::DuplicateMember {
                    name: name.to_owned(),
                    member: member.clone(),
                });
            }
            seen.push(member.as_str());
        }
        #[allow(clippy::cast_possible_truncation)]
        let id = EnumTypeId(self.types.len() as u32);
        self.types.push(EnumDecl {
            name: name.to_owned(),
            members: members.to_vec(),
        });
        Ok(id)
    }

    /// Resolve a `defenum` type name to its id.
    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<EnumTypeId> {
        self.types.iter().position(|d| d.name == name).map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            let id = i as u32;
            EnumTypeId(id)
        })
    }

    /// The declared-order ordinal of `member` within `ty` — the value
    /// §3.1's `enum` row stores in the binary64 attribute lane.
    #[must_use]
    pub fn ordinal(&self, ty: EnumTypeId, member: &str) -> Option<u32> {
        let decl = self.types.get(ty.0 as usize)?;
        decl.members.iter().position(|m| m == member).map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            let ordinal = i as u32;
            ordinal
        })
    }

    /// The member name at `ordinal` within `ty` — the read-path inverse of
    /// [`Self::ordinal`], rendering a stored ordinal back to its member
    /// (§2.13's write/read law: a read renders the member, never a bare
    /// number).
    #[must_use]
    pub fn member(&self, ty: EnumTypeId, ordinal: u32) -> Option<&str> {
        self.types
            .get(ty.0 as usize)?
            .members
            .get(ordinal as usize)
            .map(String::as_str)
    }

    /// How many members `ty` declares — the read-path integrity check's
    /// upper bound (a stored ordinal outside `[0, member_count)` is a loud
    /// integrity failure, §2.13).
    #[must_use]
    pub fn member_count(&self, ty: EnumTypeId) -> usize {
        self.types.get(ty.0 as usize).map_or(0, |d| d.members.len())
    }

    /// The type name of `ty`.
    ///
    /// # Panics
    ///
    /// If `ty` was not minted by THIS registry (see [`EnumTypeId`]'s own
    /// doc) — a caller bug, not a content error; every legitimate `ty` in
    /// circulation was resolved through [`Self::resolve`]/[`Self::declare`]
    /// against this same registry.
    #[must_use]
    pub fn name(&self, ty: EnumTypeId) -> &str {
        &self.types[ty.0 as usize].name
    }

    /// Resolve an enum id without panicking on a foreign registry identity.
    #[must_use]
    pub fn declaration(&self, ty: EnumTypeId) -> Option<&EnumDecl> {
        self.types.get(ty.0 as usize)
    }

    /// Borrow all declarations in registry order for a read-only snapshot.
    #[must_use]
    pub fn declarations(&self) -> &[EnumDecl] {
        &self.types
    }
}

/// A BSL static type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BslType {
    /// Exact, non-storable finite-kernel allocation input.
    Mass,
    /// The `[0, 1]` probability scalar.
    Probability,
    /// The `[0, 1]` intensity scalar.
    Intensity,
    /// The `[0, 1]` coefficient scalar.
    Coefficient,
    /// Fixed-point i128 micro-unit currency.
    Currency,
    /// An unbounded finite binary64 scalar — the honest declared home for
    /// what the store already does (verbatim f64, `numeric_write_value`).
    /// Carries no range law: seeds accept int / p/i/c / r literals (each
    /// already lex-bounded in its own lane), writes store any finite f64.
    /// minted by Train B item 6 (ADR-pending), AE(ii) representation-level.
    Real,
    /// `int-lit`'s static type (§1.5); also `count`'s result type (§3.4).
    Int,
    /// `#t` / `#f`.
    Bool,
    /// A member of a content-declared closed `defenum` type (§2.13, D101)
    /// — the declared-order ordinal, stored in the binary64 attribute lane,
    /// surfaced only as an `<EnumType>/<MEMBER>` enum-ref. Comparable with
    /// `=`/`!=` only, and only to the same [`EnumTypeId`]; carries no
    /// aggregation kind (§2.9, §3.4).
    Enum(EnumTypeId),
    /// A typed node-set reference, by `NodeType` name.
    NodeSet(&'static str),
    /// A typed edge-set reference, by `EdgeType` name.
    EdgeSet(&'static str),
}

/// A field's declared intensivity kind (§3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Averages meaningfully only under an extensive weight.
    Intensive,
    /// Sums meaningfully over a population or region.
    Extensive,
    /// **§2.13 addendum (D101).** An `enum`-typed field's kind — there is
    /// no meaningful extensive-or-intensive reading of a member identity,
    /// so `:kind` is structurally forbidden on an `enum`-typed `deffield`
    /// (`E-LOAD-053`, §2.9). This variant exists so [`FieldDecl::kind`]
    /// never has to hold a fabricated `Intensive`/`Extensive` value for a
    /// field the grammar itself declares kindless — a stored lie is worse
    /// than an honest third case.
    NotApplicable,
}

/// A model field's declared type + kind — the typechecker's environment
/// entry for anything a fold can read.
#[derive(Debug, Clone)]
pub struct FieldDecl {
    /// The field's scalar type.
    pub ty: BslType,
    /// The field's declared intensivity kind.
    pub kind: FieldKind,
}

#[cfg(test)]
mod tests {
    use super::{EnumRegistry, EnumRegistryError};

    #[test]
    fn declaration_order_is_the_ordinal_order_and_is_preserved() {
        let mut r = EnumRegistry::default();
        let ty = r
            .declare(
                "OrgKind",
                &[
                    "STATE_APPARATUS".into(),
                    "BUSINESS".into(),
                    "POLITICAL_FACTION".into(),
                    "CIVIL_SOCIETY".into(),
                ],
            )
            .unwrap();
        assert_eq!(r.ordinal(ty, "STATE_APPARATUS"), Some(0));
        assert_eq!(r.ordinal(ty, "CIVIL_SOCIETY"), Some(3));
        assert_eq!(r.member(ty, 1), Some("BUSINESS"));
        assert_eq!(r.ordinal(ty, "NOWHERE"), None);
        assert_eq!(r.member_count(ty), 4);
        assert_eq!(r.name(ty), "OrgKind");
    }

    #[test]
    fn a_second_declare_call_gets_a_distinct_later_id() {
        // Two types in one registry must not collide — the second type's
        // ordinals live in their OWN member list, never mixed with the
        // first's.
        let mut r = EnumRegistry::default();
        let a = r.declare("A", &["X".into()]).unwrap();
        let b = r.declare("B", &["Y".into(), "Z".into()]).unwrap();
        assert_ne!(a, b);
        assert_eq!(r.ordinal(a, "X"), Some(0));
        assert_eq!(r.ordinal(b, "Z"), Some(1));
        assert_eq!(r.ordinal(a, "Y"), None, "Y belongs to B, not A");
    }

    #[test]
    fn an_empty_member_list_refuses_loudly() {
        let mut r = EnumRegistry::default();
        assert_eq!(
            r.declare("K", &[]).unwrap_err(),
            EnumRegistryError::EmptyMemberList { name: "K".into() }
        );
    }

    #[test]
    fn a_duplicate_member_within_one_declaration_refuses_loudly() {
        let mut r = EnumRegistry::default();
        assert_eq!(
            r.declare("K", &["A".into(), "A".into()]).unwrap_err(),
            EnumRegistryError::DuplicateMember {
                name: "K".into(),
                member: "A".into()
            }
        );
    }

    #[test]
    fn a_duplicate_type_name_refuses_loudly() {
        let mut r = EnumRegistry::default();
        r.declare("K", &["A".into()]).unwrap();
        assert_eq!(
            r.declare("K", &["B".into()]).unwrap_err(),
            EnumRegistryError::DuplicateType { name: "K".into() }
        );
    }

    // Train B item 4 (#591, D157): the prelude-sharing seam's own law — a
    // scenario re-declaring EXACTLY what its prelude already declared must
    // resolve to the SAME id, not refuse.
    #[test]
    fn redeclaring_an_identical_defenum_returns_the_existing_id() {
        let mut r = EnumRegistry::default();
        let first = r
            .declare(
                "WorldView",
                &["REVOLUTIONARY".into(), "LIBERAL".into(), "FASCIST".into()],
            )
            .unwrap();
        let second = r
            .declare(
                "WorldView",
                &["REVOLUTIONARY".into(), "LIBERAL".into(), "FASCIST".into()],
            )
            .unwrap();
        assert_eq!(
            first, second,
            "an identical re-declaration must mint no new id"
        );
        assert_eq!(r.ordinal(second, "FASCIST"), Some(2));
    }

    #[test]
    fn redeclaring_with_a_reordered_member_list_still_refuses() {
        // Order-sensitive: `Vec<String>: PartialEq` demands exact position,
        // not just the same set — a reordered re-declaration is a REAL
        // conflict (it would silently move every existing ordinal), never
        // a recognized identity.
        let mut r = EnumRegistry::default();
        r.declare(
            "WorldView",
            &["REVOLUTIONARY".into(), "LIBERAL".into(), "FASCIST".into()],
        )
        .unwrap();
        assert_eq!(
            r.declare(
                "WorldView",
                &["LIBERAL".into(), "REVOLUTIONARY".into(), "FASCIST".into()]
            )
            .unwrap_err(),
            EnumRegistryError::DuplicateType {
                name: "WorldView".into()
            }
        );
    }

    #[test]
    fn redeclaring_with_an_extra_member_still_refuses() {
        let mut r = EnumRegistry::default();
        r.declare(
            "WorldView",
            &["REVOLUTIONARY".into(), "LIBERAL".into(), "FASCIST".into()],
        )
        .unwrap();
        assert_eq!(
            r.declare(
                "WorldView",
                &[
                    "REVOLUTIONARY".into(),
                    "LIBERAL".into(),
                    "FASCIST".into(),
                    "CENTRIST".into()
                ]
            )
            .unwrap_err(),
            EnumRegistryError::DuplicateType {
                name: "WorldView".into()
            }
        );
    }

    #[test]
    fn resolve_finds_a_declared_type_and_refuses_an_unknown_one() {
        let mut r = EnumRegistry::default();
        let ty = r.declare("OrgKind", &["BUSINESS".into()]).unwrap();
        assert_eq!(r.resolve("OrgKind"), Some(ty));
        assert_eq!(r.resolve("Nowhere"), None);
    }

    #[test]
    fn member_and_ordinal_refuse_an_out_of_range_query() {
        let mut r = EnumRegistry::default();
        let ty = r.declare("OrgKind", &["BUSINESS".into()]).unwrap();
        assert_eq!(r.member(ty, 5), None);
        assert_eq!(r.ordinal(ty, "NOWHERE"), None);
    }
}
