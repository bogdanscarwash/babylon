//! The coverage ledger — B3 wave-1 Task 9, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §3.4/I5.
//! **This is the M22 artifact**: the register's M22 is a MUST — "a
//! null-hypothesis viewer that silently omits a computed field without
//! declaring it is itself a wiring-completeness defect", dispositions being
//! "visual home, or explicitly RULED-ABSENT per GDS §9 W.2". [`FIELD_COVERAGE`]
//! is one row per field the wave-1 stories' own two content packs
//! (`lifecycle.bsl`/`vitality.bsl` for `counties`, `decomposition.bsl`/
//! `control-ratio.bsl` for `carceral`) write via `update-node`, transcribed
//! from those packs' own `update-node` call sites (`written_by`), not
//! surveyed after the fact — the same "derivation, not a survey" discipline
//! §3.4 itself asks for.
//!
//! **One row, one home — chosen, not invented.** Several fields genuinely
//! reach the screen through more than one path (e.g. `territory/pop-d`
//! renders in the state panel AND feeds `LIFECYCLE_TRANSITION`'s narration
//! template). [`Home`] names ONE real, verified home per field — the most
//! direct rendering site — rather than enumerating every path; the gate
//! below only requires that the named home be real, never that it be
//! exhaustive.
//!
//! **The admin panel's raw dump is a real home, asymmetric by story.**
//! `ui::admin::refresh_admin_panel` renders `graph.all_attributes()`
//! filtered to the selected node — literally every field ever written on
//! it, deliberately not curated, deliberately not routed through
//! `Projector` (the admin surface's own declared job, §2.6). It is keyed
//! off `EngineSession::roster` (the FIPS-territory roster derived in
//! `MapBinding::Fips` stories only), so it is a genuine fallback home for
//! `counties`' `territory/*` fields that have no other panel — but it
//! covers NOTHING for `carceral` (`MapBinding::None`, `roster` is always
//! empty; `refresh_admin_panel` falls through to "roster — no county
//! selected" regardless of `F3`). This asymmetry is why `counties` has only
//! two genuinely `RuledAbsent` rows below (two `vitality.bsl` fields the
//! death guard writes but nothing reads) while `carceral` has many: no
//! panel in this train reads `EngineSession::full_roster` generically the
//! way the admin dump reads `roster` — `ui::roster_panel`'s
//! `SOCIAL_CLASS_FIELDS`/`INSTITUTION_FIELDS` are curated, fixed lists
//! (§2.11), not a raw dump, and nothing else stands in for one on the
//! no-map path.
//!
//! **Countdown is a home, not a panel id.** [`Home::Countdown`] names
//! `ui::countdown::CARCERAL_STEPS` (a private table, not a public doc
//! link) — the one field it renders as a live operand
//! (`institution/superwage-crisis-tick`, the countdown row's own left-hand
//! side, §3.3) is the only row that uses it; the two gated tick fields
//! `CARCERAL_STEPS` also reads (`decomposition-fire-tick`,
//! `control-crisis-tick`) already have a more direct home — they are
//! rendered as their own numeral in `ui::roster_panel`, so that is the home
//! recorded for them instead.

/// Where a wave-1-written field's value actually reaches the screen (§3.4).
#[derive(Debug, Clone, Copy)]
pub enum Home {
    /// A registered text panel id (checked against `KNOWN_PANEL_IDS` — a
    /// private, `#[cfg(test)]`-only module constant, not a public doc link —
    /// no earlier task built a data-driven pane registry the way Task 8
    /// built [`crate::map::LENSES`] for lenses, so this ledger declares its
    /// own minimal cross-check list).
    Panel(&'static str),
    /// A registered `crate::map::LensSpec::id` (checked against
    /// `crate::map::LENSES` directly).
    Lens(&'static str),
    /// A registered `EventType` string (checked against
    /// `crate::narration::NARRATION_TABLE`).
    BeatCard(&'static str),
    /// `ui::countdown::CARCERAL_STEPS`'s live operand row (see the module
    /// doc) — carries no id of its own to check; the gate gives it a free
    /// pass the same way it gives every non-`RuledAbsent` variant a pass
    /// once its own id (if any) resolves.
    Countdown,
    /// No shipped panel/lens/beat card reads this field. `reason` states
    /// why (never empty); `future_home` names where it would land (never
    /// empty) — the M22 discipline itself: an omission must be declared,
    /// not merely absent from every other variant.
    RuledAbsent {
        reason: &'static str,
        future_home: &'static str,
    },
}

/// One field one of the wave-1 stories' own content packs writes.
#[derive(Debug, Clone, Copy)]
pub struct FieldCoverage {
    /// A `story::Story::id` (`"counties"`/`"carceral"`) — NOT checked
    /// against `story::STORIES` here (Task 9 adds no dependency from
    /// `coverage` onto `story`; the two id spellings are cross-checked
    /// visually against `story.rs:402,415` and by `tests/story.rs`'s own
    /// catalog test, which already asserts those two ids exist).
    pub story: &'static str,
    /// The graph field, `<namespace>/<name>` — the wire spelling
    /// `update-node` writes.
    pub field: &'static str,
    /// The `.bsl` pack's own `file` and `line` (or lines), spelled
    /// `pack.bsl:line`, relative to
    /// `rust/crates/babylon-tick/content/rules/`. More than one line when
    /// more than one rule in the pack writes the same field (e.g. `p04`'s
    /// additive write and `p05`'s overwrite both touch
    /// `social-class/population`).
    pub written_by: &'static str,
    pub home: Home,
}

/// The three text panels this train's UI renders, named by the module that
/// owns their repaint system — `loop_ui::refresh_state_panel` (the per-county
/// / per-node stat block), `ui::admin::refresh_admin_panel` (the `F3` raw
/// truth dump), `ui::roster_panel::format_roster_panel` (the no-map curated
/// roster fields). Declared here rather than reused from an existing table
/// because none of the three panels registers itself in a shared list the
/// way `crate::map::LENSES` does for lenses (Task 8's own registry pattern
/// was scoped to lenses, not panes — plan §10's "GDS Layout B's full pane
/// set... wave 1 builds the pane seam, not the panes" names this gap as a
/// wave-2 concern, not this train's to close). `#[cfg(test)]` because its
/// only reader is this module's own gate test — no production code needs
/// a panel-id list today.
#[cfg(test)]
const KNOWN_PANEL_IDS: &[&str] = &["state_panel", "admin", "roster_panel"];

/// The M22 ledger. ~40 rows across the two landed packs per story (§3.4's
/// own estimate; this table lands at exactly 40 — see
/// `the_table_has_exactly_forty_rows_matching_the_plans_own_estimate`).
/// One row per line, matching the repo's own `SEVERITY_TAXONOMY`/
/// `NARRATION_TABLE` regex-friendly convention even though no parity guard
/// reads this table today.
pub const FIELD_COVERAGE: &[FieldCoverage] = &[
    // ---- counties / lifecycle.bsl (8 fields, content/rules/lifecycle.bsl) ----
    FieldCoverage {
        story: "counties",
        field: "territory/pop-d",
        written_by: "lifecycle.bsl:383",
        home: Home::Panel("state_panel"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/pop-p",
        written_by: "lifecycle.bsl:384",
        home: Home::Panel("state_panel"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/pop-d-prime",
        written_by: "lifecycle.bsl:385",
        home: Home::Lens("county_population_trend"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/wealth-d-prime",
        written_by: "lifecycle.bsl:386",
        home: Home::Panel("admin"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/dependency-ratio",
        written_by: "lifecycle.bsl:387",
        home: Home::BeatCard("LIFECYCLE_TRANSITION"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/legitimation-index",
        written_by: "lifecycle.bsl:398",
        home: Home::BeatCard("LEGITIMATION_CRISIS"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/legitimation-crisis",
        written_by: "lifecycle.bsl:399",
        home: Home::Lens("county_legitimation"),
    },
    FieldCoverage {
        story: "counties",
        field: "territory/transmitted-ideology",
        written_by: "lifecycle.bsl:409",
        home: Home::Panel("admin"),
    },
    // ---- counties / vitality.bsl (3 fields, content/rules/vitality.bsl) ----
    FieldCoverage {
        story: "counties",
        field: "social-class/wealth",
        written_by: "vitality.bsl:78",
        home: Home::BeatCard("ENTITY_DEATH"),
    },
    FieldCoverage {
        story: "counties",
        field: "social-class/active",
        written_by: "vitality.bsl:88",
        home: Home::RuledAbsent {
            reason: "written only inside the death guard (the block's terminal state); \
                     ENTITY_DEATH's own payload carries wealth/consumption-needs/s-bio/s-class \
                     but not active, and the admin dump's raw fallback (see module doc) is \
                     territory-scoped, not social-class-scoped, on this Fips-bound story",
            future_home: "a 'network is fully dead' HUD tally, or widening the admin dump's \
                          fallback to the vitality-fixture social-class nodes directly",
        },
    },
    FieldCoverage {
        story: "counties",
        field: "social-class/population",
        written_by: "vitality.bsl:89",
        home: Home::RuledAbsent {
            reason: "written only inside the death guard, same as social-class/active — never a \
                     payload key, never a panel/lens read",
            future_home: "the same widened admin-dump fallback social-class/active names",
        },
    },
    // ---- carceral / decomposition.bsl (20 fields, content/rules/decomposition.bsl) ----
    FieldCoverage {
        story: "carceral",
        field: "social-class/la-census-population",
        written_by: "decomposition.bsl:242",
        home: Home::RuledAbsent {
            reason: "a per-node census intermediate that p03-trigger folds into the carrier's \
                     institution/la-population aggregate the same tick — the aggregate itself \
                     is also never rendered (see institution/la-population below)",
            future_home: "ui::roster_panel's INSTITUTION_FIELDS table, alongside a new \
                          per-class census row, once a curated per-node roster wants it",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/la-census-wealth",
        written_by: "decomposition.bsl:243",
        home: Home::RuledAbsent {
            reason: "the wealth twin of social-class/la-census-population — same fold-sum fate",
            future_home: "the same roster_panel widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/la-approaching-flag",
        written_by: "decomposition.bsl:244",
        home: Home::RuledAbsent {
            reason: "folds into institution/la-approaching-count the same tick; the aggregate \
                     is itself RuledAbsent below",
            future_home: "the same roster_panel widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/la-dying-flag",
        written_by: "decomposition.bsl:245",
        home: Home::RuledAbsent {
            reason: "folds into institution/la-dying-count the same tick; the aggregate is \
                     itself RuledAbsent below",
            future_home: "the same roster_panel widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/population",
        written_by: "decomposition.bsl:331,349",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/wealth",
        written_by: "decomposition.bsl:332,350",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/active",
        written_by: "decomposition.bsl:333,351,376",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/superwage-crisis-known",
        written_by: "decomposition.bsl:266-268",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/superwage-crisis-tick",
        written_by: "decomposition.bsl:269-271",
        home: Home::Countdown,
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/decomposition-fire-tick",
        written_by: "decomposition.bsl:309",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/decomposition-fired-known",
        written_by: "decomposition.bsl:310",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/decomposition-complete",
        written_by: "decomposition.bsl:311",
        home: Home::RuledAbsent {
            reason: "an internal idempotency latch (p03-trigger's own re-fire guard) — never a \
                     payload key and never read by any panel",
            future_home: "ui::roster_panel's INSTITUTION_FIELDS, beside the other three \
                          *-known/*-emitted flags it already curates",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/la-population",
        written_by: "decomposition.bsl:304",
        home: Home::RuledAbsent {
            reason: "the carrier aggregate p04/p05/p06 read the SAME tick it is written — \
                     an internal computation relay, never rendered on its own",
            future_home: "ui::roster_panel's INSTITUTION_FIELDS, or a wave-2 admin dump for \
                          the no-map roster (see module doc's admin-dump asymmetry note)",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/la-wealth",
        written_by: "decomposition.bsl:305",
        home: Home::RuledAbsent {
            reason: "the wealth twin of institution/la-population — same relay fate",
            future_home: "the same wave-2 widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/la-approaching-count",
        written_by: "decomposition.bsl:306",
        home: Home::RuledAbsent {
            reason: "feeds p03's own should-fire condition; never a payload key, never a \
                     panel read",
            future_home: "the same wave-2 widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/la-dying-count",
        written_by: "decomposition.bsl:307",
        home: Home::RuledAbsent {
            reason: "feeds p03's own fallback-fire condition; never a payload key, never a \
                     panel read",
            future_home: "the same wave-2 widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/enforcer-pop-gain",
        written_by: "decomposition.bsl:312",
        home: Home::BeatCard("CLASS_DECOMPOSITION"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/ip-population",
        written_by: "decomposition.bsl:313",
        home: Home::BeatCard("CLASS_DECOMPOSITION"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/enforcer-wealth-gain",
        written_by: "decomposition.bsl:314",
        home: Home::RuledAbsent {
            reason: "reaches the CLASS_DECOMPOSITION payload as wealth-transferred-to-enforcer \
                     (decomposition.bsl:373), but the shipped narration template only binds the \
                     two population-transfer slots — no template, panel or lens names this key",
            future_home: "a wealth-transfer clause added to CLASS_DECOMPOSITION's narration \
                          template, or ui::roster_panel's INSTITUTION_FIELDS",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/ip-wealth",
        written_by: "decomposition.bsl:315",
        home: Home::RuledAbsent {
            reason: "reaches the payload as wealth-transferred-to-proletariat \
                     (decomposition.bsl:375) — same unbound-template fate as \
                     institution/enforcer-wealth-gain",
            future_home: "the same narration-template or roster_panel widening",
        },
    },
    // ---- carceral / control-ratio.bsl (9 fields, content/rules/control-ratio.bsl) ----
    FieldCoverage {
        story: "carceral",
        field: "social-class/enforcer-census-population",
        written_by: "control-ratio.bsl:272",
        home: Home::RuledAbsent {
            reason: "a per-node census contribution folded into institution/enforcer-population \
                     the same tick — the aggregate has a home (below); the per-node \
                     contribution itself does not",
            future_home: "ui::roster_panel's INSTITUTION_FIELDS, alongside a new per-class \
                          census row",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/prisoner-census-population",
        written_by: "control-ratio.bsl:273",
        home: Home::RuledAbsent {
            reason: "the population twin of social-class/enforcer-census-population — same \
                     fold-sum fate",
            future_home: "the same roster_panel widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "social-class/prisoner-census-org-weighted",
        written_by: "control-ratio.bsl:274",
        home: Home::RuledAbsent {
            reason: "folds into institution/prisoner-org-weighted the same tick, mirroring the \
                     two population census rows above",
            future_home: "the same roster_panel widening",
        },
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/enforcer-population",
        written_by: "control-ratio.bsl:289",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/prisoner-population",
        written_by: "control-ratio.bsl:290",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/prisoner-org-weighted",
        written_by: "control-ratio.bsl:291",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/control-crisis-emitted",
        written_by: "control-ratio.bsl:337",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/control-crisis-tick",
        written_by: "control-ratio.bsl:338",
        home: Home::Panel("roster_panel"),
    },
    FieldCoverage {
        story: "carceral",
        field: "institution/terminal-decision-emitted",
        written_by: "control-ratio.bsl:379",
        home: Home::Panel("roster_panel"),
    },
];

#[cfg(test)]
mod tests {
    use super::{Home, FIELD_COVERAGE, KNOWN_PANEL_IDS};

    /// §3.4's own estimate ("roughly 40 rows") — this table lands at
    /// exactly 40, not merely "roughly", because every field was
    /// transcribed from the four packs' own `update-node` call sites, not
    /// estimated.
    #[test]
    fn the_table_has_exactly_forty_rows_matching_the_plans_own_estimate() {
        assert_eq!(FIELD_COVERAGE.len(), 40);
    }

    /// (a) — every `(story, field)` pair is unique. The same field name
    /// legitimately appears once per story that writes it (e.g.
    /// `social-class/wealth` is written by BOTH `vitality.bsl` on
    /// `counties` and `decomposition.bsl` on `carceral`) — that is two
    /// rows, not a duplicate; a genuine duplicate would be the SAME
    /// `(story, field)` pair twice.
    #[test]
    fn every_story_field_pair_appears_exactly_once() {
        for (i, a) in FIELD_COVERAGE.iter().enumerate() {
            for b in &FIELD_COVERAGE[i + 1..] {
                assert!(
                    !(a.story == b.story && a.field == b.field),
                    "duplicate row: story {:?} field {:?}",
                    a.story,
                    a.field
                );
            }
        }
    }

    /// Every row names a real `.bsl` pack `file` and `line` — never empty,
    /// never a bare pack name with no line.
    #[test]
    fn every_row_names_a_real_written_by_citation() {
        for row in FIELD_COVERAGE {
            assert!(
                row.written_by.contains(".bsl:"),
                "row for {:?} must cite a file:line, got {:?}",
                row.field,
                row.written_by
            );
        }
    }

    /// (b) — every `Panel` home names an id this crate's own panels
    /// actually use.
    #[test]
    fn every_panel_home_names_a_known_panel_id() {
        for row in FIELD_COVERAGE {
            if let Home::Panel(id) = row.home {
                assert!(
                    KNOWN_PANEL_IDS.contains(&id),
                    "field {:?} names panel id {:?}, not in KNOWN_PANEL_IDS {:?}",
                    row.field,
                    id,
                    KNOWN_PANEL_IDS
                );
            }
        }
    }

    /// (b) — every `Lens` home names a real, currently-registered
    /// `crate::map::LensSpec::id`.
    #[test]
    fn every_lens_home_names_a_registered_lens_id() {
        for row in FIELD_COVERAGE {
            if let Home::Lens(id) = row.home {
                assert!(
                    crate::map::LENSES.iter().any(|spec| spec.id == id),
                    "field {:?} names lens id {:?}, not found in crate::map::LENSES",
                    row.field,
                    id
                );
            }
        }
    }

    /// (b) — every `BeatCard` home names a real, currently-transcribed
    /// `EventType` in `crate::narration::NARRATION_TABLE`.
    #[test]
    fn every_beat_card_home_names_a_transcribed_event_type() {
        for row in FIELD_COVERAGE {
            if let Home::BeatCard(event_type) = row.home {
                assert!(
                    crate::narration::NARRATION_TABLE
                        .iter()
                        .any(|spec| spec.event_type == event_type),
                    "field {:?} names EventType {:?}, not found in \
                     crate::narration::NARRATION_TABLE",
                    row.field,
                    event_type
                );
            }
        }
    }

    /// (c) — every `RuledAbsent` row carries a non-empty reason AND a
    /// non-empty named future home. A silent omission (dropping the row)
    /// would be exactly the M22 defect this ledger exists to forbid; an
    /// EMPTY `reason`/`future_home` would be the same defect wearing the
    /// ledger's own clothes.
    #[test]
    fn every_ruled_absent_row_carries_a_reason_and_a_future_home() {
        let mut ruled_absent_count = 0;
        for row in FIELD_COVERAGE {
            if let Home::RuledAbsent {
                reason,
                future_home,
            } = row.home
            {
                ruled_absent_count += 1;
                assert!(
                    !reason.is_empty(),
                    "field {:?}'s RuledAbsent reason must not be empty",
                    row.field
                );
                assert!(
                    !future_home.is_empty(),
                    "field {:?}'s RuledAbsent future_home must not be empty",
                    row.field
                );
            }
        }
        // A sanity floor, not a magic number: this train's own carceral
        // pack analysis (module doc) found real, structural absences —
        // zero RuledAbsent rows would mean this test stopped exercising
        // the RuledAbsent match arm at all (a vacuous-coverage risk, the
        // same class the parity guard's own non-vacuity fix (Task 4.6)
        // guards against).
        assert!(
            ruled_absent_count > 0,
            "this table must exercise at least one genuine RuledAbsent row"
        );
    }

    /// Every story named in the table is one of the two wave-1 stories —
    /// guards against a typo'd story id silently creating a third,
    /// uncatalogued "story" that neither `STORIES` nor any reader expects.
    #[test]
    fn every_row_names_a_wave_one_story() {
        for row in FIELD_COVERAGE {
            assert!(
                row.story == "counties" || row.story == "carceral",
                "field {:?} names unknown story {:?}",
                row.field,
                row.story
            );
        }
    }
}
