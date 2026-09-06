//! PER-23 Slice 4 (ADR249 R5/R6): markdown-free dossier card composition.
//!
//! The card is atom-driven (Slice 4 decision 1): place-link chips, R6
//! placeholder pages, signal rows, and the chronicle strip are all pure
//! functions over in-memory projection data. There is deliberately NO
//! Markdown parser here — page-Markdown in-app viewing belongs to the later
//! wiki surface — and nothing in this module touches the Archive reader:
//! placeholder text is a rendering policy synthesized from public structure,
//! never persisted, hashed, indexed, or searchable (R6).
//!
//! The chip rule (R5): a place link the acknowledged Archive resolves gets a
//! labeled chip; a link with no acknowledged known page synthesizes the
//! pinned `fog_chip_v1` string ("unknown place · 2674900" for that id), carrying zero label
//! bytes. Strikethrough semantics from the Markdown profile
//! (`~~[Detroit](subject:…)~~`) render as DIM text plus a "· pending" suffix
//! (decision 2) — no combining-glyph tricks, testable headless.

use babylon_persistence::archive_revision::{ArchiveAtomChangeV2, ArchiveDossierStateV2};
use babylon_persistence::ArchiveSignalV1;
use babylon_persistence::{fog_chip_v1, ArchiveAtomV1, ArchiveAtomValueV1};

/// The one decision question the dossier card answers (ADR249 R9), pinned in
/// exactly one place so the manifest row and the rendered card cannot drift.
pub const DOSSIER_DECISION_QUESTION: &str =
    "What is true here, and what would Investigation reveal?";

/// The sealed actions-footer chip text (the card's visible-unavailable
/// Investigate slot, R9).
pub const INVESTIGATE_SEALED_CHIP: &str = "INVESTIGATE — SEALED UNTIL GATE 5";

/// The honest reason carried by the manifest's unavailable Investigate
/// action — the same sentence the R6 placeholders seal with.
pub const INVESTIGATE_UNAVAILABLE_REASON: &str =
    "Investigation opens this page — unavailable until Gate 5.";

/// R6(a) stub seal: a granted-but-uninvestigated public subject shows its
/// cursory tier and seals the earned tier.
pub const STUB_SEALED_LINE: &str =
    "Investigation opens the rest of this page — unavailable until Gate 5.";

/// R6(b) vague seal: a fully ungranted subject outside public reference.
pub const VAGUE_SEALED_LINE: &str = "Investigation opens this page — unavailable until Gate 5.";

/// Player-facing tone of one rendered text segment. The card maps each tone
/// to a palette constant at the call site (palette.rs is parity-guarded, so
/// alpha derivation lives there, never here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DossierTone {
    /// Primary content (BONE).
    Bone,
    /// De-emphasized content (BONE with alpha).
    BoneDim,
    /// Accent: headers, rules, caught-up verification (GOLD).
    Gold,
    /// Honest problem state: pending, failure (CRIMSON).
    Crimson,
    /// Inert chrome and citations (DIM).
    Dim,
}

/// One piece of a rendered line with its tone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DossierSegment {
    pub text: String,
    pub tone: DossierTone,
}

impl DossierSegment {
    #[must_use]
    fn new(text: impl Into<String>, tone: DossierTone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
}

/// One place-link chip on the dossier card (ADR249 R5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaceChip {
    geoid: String,
    label: Option<String>,
    pending: bool,
}

impl PlaceChip {
    /// A labeled chip: the Archive acknowledges a known page for the place.
    /// `pending` marks the Archive-lag case (the place page's verified tick
    /// sits behind the durable tick) — rendered DIM with a "· pending"
    /// suffix instead of a glyph strikethrough.
    #[must_use]
    pub fn known(geoid: impl Into<String>, label: impl Into<String>, pending: bool) -> Self {
        Self {
            geoid: geoid.into(),
            label: Some(label.into()),
            pending,
        }
    }

    /// A fog chip: no acknowledged known page — the synthesized
    /// kind-and-id form, zero label bytes.
    #[must_use]
    pub fn unknown(geoid: impl Into<String>) -> Self {
        Self {
            geoid: geoid.into(),
            label: None,
            pending: false,
        }
    }

    /// Borrow the public place GEOID.
    #[must_use]
    pub fn geoid(&self) -> &str {
        &self.geoid
    }

    /// Return whether the Archive acknowledges a known page for this place.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        self.label.is_some()
    }

    /// Return whether this chip renders in the pending (Archive-lag) state.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.pending
    }
}

/// The chip's visible text: the granted label, or the pinned fog-chip string
/// synthesized from public structure alone. Pending chips carry the
/// "· pending" suffix (decision 2).
#[must_use]
pub fn chip_text(chip: &PlaceChip) -> String {
    match &chip.label {
        Some(label) if chip.pending => format!("{label} · pending"),
        Some(label) => label.clone(),
        None => fog_chip_v1("place", &chip.geoid),
    }
}

/// R6(a): the granted-but-uninvestigated stub page, synthesized from the
/// granted cursory tier only. `containment` is the place's public-record
/// containment ("Oakland County, Michigan"); `None` renders the identity
/// line without a containment claim rather than inventing one.
#[must_use]
pub fn compose_stub(name: &str, containment: Option<&str>) -> Vec<String> {
    let identity = match containment {
        Some(place) => format!("{name} — place in {place}."),
        None => format!("{name} — place."),
    };
    vec![identity, STUB_SEALED_LINE.to_owned()]
}

/// R6(b): the fully ungranted vague placeholder, kind and public id only.
#[must_use]
pub fn compose_vague(kind: &str, id: &str) -> Vec<String> {
    vec![
        format!("You don't have enough detail on {kind} {id}."),
        VAGUE_SEALED_LINE.to_owned(),
    ]
}

/// Render one typed atom value with the statblock's `%.6f` discipline for
/// floats, matching the pinned citation-line and chronicle spellings.
#[must_use]
pub fn atom_value_text(value: &ArchiveAtomValueV1) -> String {
    match value {
        ArchiveAtomValueV1::Text(text) => text.clone(),
        ArchiveAtomValueV1::F64(number) => format!("{number:.6}"),
        ArchiveAtomValueV1::U64(number) => number.to_string(),
        ArchiveAtomValueV1::Bool(flag) => flag.to_string(),
    }
}

/// The atoms that become signal rows: everything except the subject-identity
/// atom (the card title) and link atoms (place chips). Position order is the
/// Archive's composition order, preserved.
pub fn signal_atoms(atoms: &[ArchiveAtomV1]) -> impl Iterator<Item = &ArchiveAtomV1> {
    atoms
        .iter()
        .filter(|atom| atom.signal_key() != "subject" && atom.signal_key() != "link")
}

/// One signal row as toned segments: label BONE-dim, value BONE, citation
/// DIM — `median-wage: 31.400000 — committed-tick-v1; campaign/12/…`.
#[must_use]
pub fn signal_row_segments(atom: &ArchiveAtomV1) -> Vec<DossierSegment> {
    vec![
        DossierSegment::new(format!("{}: ", atom.signal_key()), DossierTone::BoneDim),
        DossierSegment::new(atom_value_text(atom.value()), DossierTone::Bone),
        DossierSegment::new(
            format!(
                " — {}; {}",
                atom.citation().source_id(),
                atom.citation().locator()
            ),
            DossierTone::Dim,
        ),
    ]
}

/// Exact retained display labels and values, with evidence expanded inline.
#[must_use]
pub fn retained_signal_segments(signal: &ArchiveSignalV1, details: bool) -> Vec<DossierSegment> {
    let mut parts = vec![
        DossierSegment::new(format!("{}: ", signal.label()), DossierTone::BoneDim),
        DossierSegment::new(signal.value(), DossierTone::Bone),
    ];
    if details {
        parts.push(DossierSegment::new(
            format!(
                "\nSource: {}\nLocator: {}\nField: {}",
                signal.citation().source_id(),
                signal.citation().locator(),
                signal.grant_key()
            ),
            DossierTone::Dim,
        ));
    }
    parts
}

/// A bounded or unfinished change page never implies that no changes occurred.
pub(crate) fn chronicle_header(state: &ArchiveDossierStateV2) -> DossierSegment {
    match state {
        ArchiveDossierStateV2::Pending { .. } => {
            DossierSegment::new("Changes await Archive completion.", DossierTone::Crimson)
        }
        ArchiveDossierStateV2::Ready { page, .. } => {
            let mut text = format!(
                "Retained change coverage starts at week {}.",
                page.changes.coverage_from_tick
            );
            if page.changes.next_cursor.is_some() {
                text.push_str(" More retained changes are available.");
            }
            DossierSegment::new(text, DossierTone::Dim)
        }
        ArchiveDossierStateV2::Unavailable(_) => DossierSegment::new(
            "Change history is unavailable for this observation.",
            DossierTone::Dim,
        ),
    }
}

/// Report publication differences without calling absence zero or a verification.
#[must_use]
pub fn chronicle_row_segments(row: &ArchiveAtomChangeV2) -> Vec<DossierSegment> {
    let mut segments = vec![DossierSegment::new(
        format!(
            "Published week {} · {} ",
            row.publication_tick, row.signal_key
        ),
        DossierTone::Dim,
    )];
    match (&row.before, &row.after) {
        (Some(before), Some(after)) => {
            segments.push(DossierSegment::new(
                atom_value_text(before.value()),
                DossierTone::Bone,
            ));
            segments.push(DossierSegment::new(" → ", DossierTone::Gold));
            segments.push(DossierSegment::new(
                atom_value_text(after.value()),
                DossierTone::Bone,
            ));
        }
        (None, Some(after)) => {
            segments.push(DossierSegment::new(
                "(added within retained coverage) ",
                DossierTone::Gold,
            ));
            segments.push(DossierSegment::new(
                atom_value_text(after.value()),
                DossierTone::Bone,
            ));
        }
        (Some(before), None) => {
            segments.push(DossierSegment::new(
                atom_value_text(before.value()),
                DossierTone::Bone,
            ));
            segments.push(DossierSegment::new(
                " → removed from this publication",
                DossierTone::Gold,
            ));
        }
        (None, None) => segments.push(DossierSegment::new(
            "No retained value on either side",
            DossierTone::Dim,
        )),
    }
    for (label, atom) in [("Before", &row.before), ("After", &row.after)] {
        if let Some(atom) = atom {
            segments.push(DossierSegment::new(
                format!(
                    "\n{label}: observed week {}; {}; {}",
                    atom.valid_tick(),
                    atom.citation().source_id(),
                    atom.citation().locator()
                ),
                DossierTone::Dim,
            ));
        }
    }
    segments
}

/// The dual-tick honesty header (ADR249 R9): durable tick and verified tick
/// always separately visible. Caught up → GOLD verified; Archive lagging →
/// CRIMSON verified plus the explicit "Archive materializing" line; no
/// committed tick → one honest DIM line.
#[must_use]
pub fn dual_tick_segments(durable: Option<u64>, verified: Option<u64>) -> Vec<DossierSegment> {
    match (durable, verified) {
        (None, _) => vec![DossierSegment::new(
            "no committed tick — the Archive is empty",
            DossierTone::Dim,
        )],
        (Some(durable), None) => vec![
            DossierSegment::new(format!("durable {durable}"), DossierTone::Dim),
            DossierSegment::new(" · verified — pending", DossierTone::Crimson),
            DossierSegment::new(
                "\nArchive materializing — no page verified yet",
                DossierTone::Crimson,
            ),
        ],
        (Some(durable), Some(verified)) if verified >= durable => vec![
            DossierSegment::new(format!("durable {durable}"), DossierTone::Dim),
            DossierSegment::new(format!(" · verified {verified}"), DossierTone::Gold),
        ],
        (Some(durable), Some(verified)) => vec![
            DossierSegment::new(format!("durable {durable}"), DossierTone::Dim),
            DossierSegment::new(format!(" · verified {verified}"), DossierTone::Crimson),
            DossierSegment::new(
                format!("\nArchive materializing — verified {verified} of {durable}"),
                DossierTone::Crimson,
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_persistence::{
        ArchiveAtomSubjectKindV1, ArchiveAtomSubjectV1, ArchiveAtomValueV1, ArchiveCitationV1,
        ArchiveEvidenceClassV1, CampaignId,
    };
    use uuid::Uuid;

    fn atom(signal_key: &str, value: &str, valid_tick: u64) -> ArchiveAtomV1 {
        ArchiveAtomV1::try_new(
            CampaignId::from_uuid(Uuid::nil()),
            ArchiveAtomSubjectV1::try_new(ArchiveAtomSubjectKindV1::County, "26163".to_owned())
                .expect("subject admits"),
            signal_key.to_owned(),
            signal_key.to_owned(),
            ArchiveEvidenceClassV1::Observed,
            &ArchiveAtomValueV1::Text(value.to_owned()),
            ArchiveCitationV1::try_new(
                "committed-tick-v1".to_owned(),
                "campaign/12/Wayne".to_owned(),
            )
            .expect("citation admits"),
            valid_tick,
        )
        .expect("atom admits")
    }

    #[test]
    fn granted_chip_renders_its_label() {
        let chip = PlaceChip::known("2622000", "Detroit", false);
        assert!(chip.is_known());
        assert!(!chip.is_pending());
        assert_eq!(chip_text(&chip), "Detroit");
    }

    #[test]
    fn pending_chip_renders_dim_suffix_without_strikethrough_glyphs() {
        let chip = PlaceChip::known("2622000", "Detroit", true);
        assert_eq!(chip_text(&chip), "Detroit · pending");
        // Decision 2: no combining-glyph or strike bytes anywhere.
        assert!(!chip_text(&chip).contains('\u{336}'));
        assert!(!chip_text(&chip).contains("~~"));
    }

    #[test]
    fn fog_chip_synthesizes_kind_and_id_with_zero_label_bytes() {
        let chip = PlaceChip::unknown("2674900");
        assert!(!chip.is_known());
        let text = chip_text(&chip);
        assert_eq!(text, "unknown place · 2674900");
        // The hidden label must not leak into the synthesized bytes (R5).
        assert!(!text.contains("Riverview"));
    }

    #[test]
    fn stub_composes_identity_containment_and_the_pinned_seal() {
        let lines = compose_stub("Southfield", Some("Oakland County, Michigan"));
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Southfield — place in Oakland County, Michigan.");
        assert_eq!(lines[1], STUB_SEALED_LINE);
        assert!(STUB_SEALED_LINE.contains("unavailable until Gate 5"));
    }

    #[test]
    fn stub_without_containment_never_invents_one() {
        let lines = compose_stub("Southfield", None);
        assert_eq!(lines[0], "Southfield — place.");
        assert!(!lines[0].contains("in ."));
    }

    #[test]
    fn vague_placeholder_uses_kind_and_id_only() {
        let lines = compose_vague("place", "2674900");
        assert_eq!(lines[0], "You don't have enough detail on place 2674900.");
        assert_eq!(lines[1], VAGUE_SEALED_LINE);
    }

    #[test]
    fn placeholder_sentences_match_the_adr249_r6_wording() {
        // Pinned against ADR249 R6: the placeholder bytes are a rendering
        // policy, never persisted or searchable, so their spelling is a
        // contract here.
        assert_eq!(
            VAGUE_SEALED_LINE,
            "Investigation opens this page — unavailable until Gate 5."
        );
        assert_eq!(
            STUB_SEALED_LINE,
            "Investigation opens the rest of this page — unavailable until Gate 5."
        );
    }

    #[test]
    fn signal_rows_skip_subject_and_link_atoms() {
        let atoms = vec![
            atom("subject", "Wayne County", 12),
            atom("median-wage", "31.400000", 12),
            atom("link", "place/2622000", 12),
        ];
        let keys: Vec<_> = signal_atoms(&atoms)
            .map(|atom| atom.signal_key().to_owned())
            .collect();
        assert_eq!(keys, vec!["median-wage".to_owned()]);
    }

    #[test]
    fn signal_row_segments_carry_label_value_and_citation_tones() {
        let segments = signal_row_segments(&atom("median-wage", "31.400000", 12));
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].tone, DossierTone::BoneDim);
        assert_eq!(segments[0].text, "median-wage: ");
        assert_eq!(segments[1].tone, DossierTone::Bone);
        assert_eq!(segments[1].text, "31.400000");
        assert_eq!(segments[2].tone, DossierTone::Dim);
        assert!(segments[2]
            .text
            .contains("committed-tick-v1; campaign/12/Wayne"));
    }

    #[test]
    fn f64_values_render_with_the_statblock_six_decimal_discipline() {
        assert_eq!(atom_value_text(&ArchiveAtomValueV1::F64(31.4)), "31.400000");
        assert_eq!(atom_value_text(&ArchiveAtomValueV1::U64(728_576)), "728576");
        assert_eq!(atom_value_text(&ArchiveAtomValueV1::Bool(true)), "true");
    }

    #[test]
    fn publication_changes_distinguish_replacement_addition_and_removal() {
        let before = atom("jobs", "2", 11);
        let after = atom("jobs", "0", 12);
        for (old, new, expected) in [
            (Some(before.clone()), Some(after.clone()), "2 → 0"),
            (None, Some(after), "(added within retained coverage) 0"),
            (Some(before), None, "2 → removed from this publication"),
        ] {
            let row = ArchiveAtomChangeV2 {
                publication_tick: 14,
                signal_key: "jobs".into(),
                before: old,
                after: new,
            };
            let text: String = chronicle_row_segments(&row)
                .iter()
                .map(|segment| segment.text.as_str())
                .collect();
            assert!(text.starts_with("Published week 14"));
            assert!(text.contains(expected));
            assert!(!text.contains("verified"));
            assert!(text.contains("committed-tick-v1; campaign/12/Wayne"));
        }
    }

    #[test]
    fn dual_tick_segments_report_caught_up_pending_and_empty_honestly() {
        let caught_up = dual_tick_segments(Some(12), Some(12));
        assert_eq!(caught_up.len(), 2);
        assert_eq!(caught_up[1].tone, DossierTone::Gold);
        assert_eq!(caught_up[1].text, " · verified 12");

        let lagging = dual_tick_segments(Some(12), Some(11));
        assert_eq!(lagging[1].tone, DossierTone::Crimson);
        assert!(lagging.iter().any(|s| s.tone == DossierTone::Crimson
            && s.text.contains("Archive materializing — verified 11 of 12")));

        let unverified = dual_tick_segments(Some(12), None);
        assert!(unverified
            .iter()
            .any(|s| s.tone == DossierTone::Crimson && s.text.contains("no page verified yet")));

        let empty = dual_tick_segments(None, None);
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].tone, DossierTone::Dim);
        assert!(empty[0].text.contains("no committed tick"));
    }
}
