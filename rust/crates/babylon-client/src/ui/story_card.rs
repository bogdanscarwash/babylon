//! The tick-0 story card, the catalog + controls legend it carries, the
//! `N`-key restart, and the §2.11 map-absence banner (B3 wave-1 Task 5,
//! plan `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.5/§2.11/§3.1). Registered inside `loop_ui::TickLoopPlugin` (this
//! module owns no `Plugin` of its own — the same house pattern
//! `ui::time`/`ui::beats`/`ui::admin` already establish: a module of
//! systems `TickLoopPlugin::build` wires in, not a nested plugin).

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::engine_link::EngineSession;
use crate::severity::SeverityTier;
use crate::story::{Story, STORIES};
use bevy::prelude::*;

/// Whether the tick-0 (or recalled) story card is showing. Defaults to
/// `true` — unlike `ui::admin::AdminPanelVisible` (an unrequested, opt-in
/// instrument), the story card IS the first thing a fresh run shows (§2.5:
/// "the tick-0 story card").
#[derive(Resource, Debug, Clone, Copy)]
pub struct StoryCardVisible(pub bool);

impl Default for StoryCardVisible {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component)]
pub struct StoryCardText;

/// `Startup` system: spawns the (initially empty) story card text entity —
/// `refresh_story_card`'s own first `Update` pass fills it in.
pub fn spawn_story_card(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            top: px(4),
            left: px(200),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::StoryCard),
        StoryCardText,
    ));
}

/// Counts only `critical`/`warning` beats — an `Informational` beat (every
/// `LIFECYCLE_TRANSITION`, every `LEGITIMATION_RECOVERY`, …) never moves
/// the card's own counter, matching 5.3's own brief.
#[must_use]
fn beats_seen(log: &crate::ui::beats::BeatLog) -> usize {
    log.beats
        .iter()
        .filter(|b| b.tier != SeverityTier::Informational)
        .count()
}

/// `story.arc`'s own beat total, rendered — `"?"` for a story with no fixed
/// arc (counties: an ambient world, not a scripted one) rather than a
/// fabricated number (III.11).
#[must_use]
fn beat_total_label(story: &Story) -> String {
    story
        .arc
        .map_or_else(|| "?".to_owned(), |arc| arc.beat_count.to_string())
}

/// One catalog row for the card's own roster listing — id, title, the
/// premise's FIRST transcribed line (one-line, never the whole multi-line
/// block), and the story's own beat total.
#[must_use]
fn catalog_row(entry: &Story, current: &Story) -> String {
    let marker = if std::ptr::eq(entry, current) {
        "*"
    } else {
        " "
    };
    // task-5-review.md Minor 1: the un-prefixed fallback must be the FIRST
    // LINE only, never the whole (possibly multi-line) `entry.premise` —
    // `first_line` is captured once so both the stripped and un-stripped
    // paths agree on what "one line" means.
    let first_line = entry.premise.lines().next().unwrap_or("");
    let one_line = first_line.strip_prefix("; ").unwrap_or(first_line);
    format!(
        "  {marker} {} \u{2014} {} \u{2014} {one_line} \u{2014} {} beats",
        entry.id,
        entry.title,
        beat_total_label(entry)
    )
}

const CONTROLS_LEGEND: &[&str] = &[
    "  Space  advance one tick",
    "  P      play/pause",
    "  , .    speed down/up",
    "  B      run to next beat",
    "  Tab    cycle map lens",
    "  \u{2191} \u{2193}    select roster node (no-map stories)",
    "  F3     admin panel",
    "  N      restart into the next story",
    "  ?      show/hide this card",
];

/// Renders the whole card — title, premise, `X/N beats`, the whole
/// catalog, and the full controls legend (§2.5's own tick-0 card spec).
/// Pure and independently testable, matching this crate's own
/// `format_*` idiom (`ui::admin::format_tick_report`,
/// `ui::beats::format_latch_card`, …).
#[must_use]
pub fn format_story_card(story: &Story, beats_now: usize) -> String {
    let mut lines = vec![
        story.title.to_owned(),
        String::new(),
        story.premise.to_owned(),
        String::new(),
        format!("{beats_now}/{} beats", beat_total_label(story)),
        String::new(),
        "Stories (* = currently playing):".to_owned(),
    ];
    for entry in STORIES {
        lines.push(catalog_row(entry, story));
    }
    lines.push(String::new());
    lines.push("Controls:".to_owned());
    lines.extend(CONTROLS_LEGEND.iter().map(|s| (*s).to_owned()));
    lines.join("\n")
}

/// `Update` system: repaints `StoryCardText` from `StoryCardVisible` +
/// `EngineSession` + `BeatLog` — renders nothing while hidden, the same
/// "empty string is the honest render of nothing to show" idiom
/// `ui::admin::refresh_admin_panel` already established.
pub fn refresh_story_card(
    visible: Res<StoryCardVisible>,
    session: Res<EngineSession>,
    log: Res<crate::ui::beats::BeatLog>,
    mut card_text: Query<&mut Text, With<StoryCardText>>,
) {
    let Ok(mut text) = card_text.single_mut() else {
        return;
    };
    if !visible.0 {
        text.0 = String::new();
        return;
    }
    text.0 = format_story_card(session.story, beats_seen(&log));
}

/// `Update` system: dismisses the card the first time the tick counter
/// moves off `0` — "dismissed on first advance" (§2.5). Ordered
/// `.after(ui::time::advance_ticks)` so this frame's own advance (if any)
/// is visible to `TickCounter.is_changed()` before this system reads it.
pub fn dismiss_story_card_on_first_advance(
    counter: Res<crate::loop_ui::TickCounter>,
    mut visible: ResMut<StoryCardVisible>,
) {
    if counter.is_changed() && counter.0 > 0 {
        visible.0 = false;
    }
}

/// `Update` system: `?` toggles the card back on (or off) — "recallable
/// with `?`" (§2.5). Bound to the bare physical `Slash` key rather than a
/// Shift-aware `?` check: `?` is Shift+`/` on virtually every layout, and
/// this crate's own single-physical-key precedent (`F3`'s admin toggle,
/// `Tab`'s lens cycle, `N`'s restart) never gates on a modifier either.
pub fn recall_story_card_on_question_mark(
    keys: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<StoryCardVisible>,
) {
    if keys.just_pressed(KeyCode::Slash) {
        visible.0 = !visible.0;
    }
}

/// `Update` system: `N` restarts into the NEXT catalog entry (I8) — a
/// fresh `EngineSession`, the counter/clock/beat-log/admin-report state all
/// reset to their own Startup defaults, and the story card shown again.
/// `sync_map_to_story` (below) reacts to the `EngineSession` swap this
/// system performs to flip the map's own visibility+banner for the new
/// story — ordered `.after(this)` in `TickLoopPlugin::build`, not repeated
/// here. The dossier card clears through the same signal (PER-23 Slice 4):
/// this system's `selected_county.0 = None` write is the selection change
/// `ui::dossier_card::drive_dossier_fetch` reacts to, dropping any in-flight
/// fetch, clearing the projection and any placeholder view — the card
/// renders nothing until the player selects a county again.
///
/// # Panics
/// If the next catalog story fails to start — cannot happen for either
/// shipped story (both are proven-loading content); a genuine failure here
/// is exactly as loud as the Startup path's own `unwrap_or_else(panic!)`.
#[allow(clippy::too_many_arguments)]
pub fn restart_on_n_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<crate::story::SelectedStory>,
    mut session: ResMut<EngineSession>,
    mut counter: ResMut<crate::loop_ui::TickCounter>,
    mut run_state: ResMut<crate::ui::time::RunState>,
    mut tick_phase: ResMut<crate::ui::time::TickPhase>,
    mut last_batch: ResMut<crate::ui::time::LastBatch>,
    mut lens_data: ResMut<crate::lens::CurrentLensData>,
    mut lens_changed: MessageWriter<crate::map::LensChanged>,
    mut hud_tick: ResMut<crate::map::HudTick>,
    mut last_tick_report: ResMut<crate::ui::admin::LastTickReport>,
    mut beat_log: ResMut<crate::ui::beats::BeatLog>,
    mut selected_county: ResMut<crate::map::SelectedCounty>,
    mut selected_roster: ResMut<crate::ui::roster_panel::SelectedRosterIndex>,
    mut visible: ResMut<StoryCardVisible>,
) {
    if !keys.just_pressed(KeyCode::KeyN) {
        return;
    }
    let next = crate::story::next_story(selected.0);
    let fresh =
        EngineSession::start(next).unwrap_or_else(|e| panic!("restart into {:?}: {e}", next.id));

    *lens_data = crate::loop_ui::build_lens_data(&fresh);
    *session = fresh;
    selected.0 = next;
    counter.0 = 0;
    *run_state = crate::ui::time::RunState::default();
    tick_phase.0 = 0.0;
    last_batch.0 = 0;
    last_tick_report.0 = None;
    *beat_log = crate::ui::beats::BeatLog::default();
    selected_county.0 = None;
    selected_roster.0 = None;
    hud_tick.0 = 0;
    visible.0 = true;
    lens_changed.write(crate::map::LensChanged);
}

#[derive(Component)]
pub struct MapAbsenceBannerText;

/// `Startup` system: spawns the (initially empty) §2.11 map-absence banner
/// text entity.
pub fn spawn_map_absence_banner(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::CRIMSON),
        Node {
            position_type: PositionType::Absolute,
            top: px(48),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::MapAbsenceNotice),
        MapAbsenceBannerText,
    ));
}

/// `Update` system (§2.11): hides the county fill/border meshes and
/// renders the declared absence banner for a `MapBinding::None` story;
/// shows them (clearing the banner) for a `MapBinding::Fips` story.
/// Reactive on `session.is_changed()` — a freshly-inserted resource counts
/// as changed, so this fires once on the very first `Update` pass after
/// Startup (covering the launched story) and again whenever the `N`-key
/// restart swaps in a session over a DIFFERENT story (covering both
/// directions of the toggle with one system, never a separate "undo").
// The `Query<&mut Visibility, Or<(With<MapFill>, With<MapBorders>)>>` shape
// trips `clippy::type_complexity` on raw token count alone — it is exactly
// the shape the lint means to flag (a `Query` filter tuple), not a genuine
// readability problem; a `type` alias would need Bevy's own `'w`/`'s` query
// lifetimes threaded through for no clarity gain over the inline form.
#[allow(clippy::type_complexity)]
pub fn sync_map_to_story(
    session: Res<EngineSession>,
    mut mesh_visibility: Query<
        &mut Visibility,
        Or<(With<crate::map::MapFill>, With<crate::map::MapBorders>)>,
    >,
    mut banner: Query<&mut Text, With<MapAbsenceBannerText>>,
) {
    if !session.is_changed() {
        return;
    }
    let has_map = session.story.map_binding.is_some();
    for mut visibility in &mut mesh_visibility {
        *visibility = if has_map {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let Ok(mut text) = banner.single_mut() else {
        return;
    };
    text.0 = if has_map {
        String::new()
    } else {
        format!(
            "{} has no territorial substrate \u{2014} {} nodes, 0 territories; the county map \
             is not applicable.",
            session.story.title, session.node_count
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story;
    use crate::ui::beats::{Beat, BeatLog};

    fn beat(tier: SeverityTier) -> Beat {
        Beat {
            tick: 1,
            event_type: "X".to_owned(),
            payload: Vec::new(),
            tier,
            magnitude_delta: None,
        }
    }

    #[test]
    fn beats_seen_counts_only_critical_and_warning() {
        let mut log = BeatLog::default();
        log.beats.push_back(beat(SeverityTier::Informational));
        log.beats.push_back(beat(SeverityTier::Warning));
        log.beats.push_back(beat(SeverityTier::Critical));
        log.beats.push_back(beat(SeverityTier::Informational));
        assert_eq!(beats_seen(&log), 2);
    }

    #[test]
    fn beat_total_label_is_the_real_count_for_carceral_and_a_question_mark_for_counties() {
        assert_eq!(beat_total_label(story::carceral()), "4");
        assert_eq!(beat_total_label(story::counties()), "?");
    }

    #[test]
    fn format_story_card_names_the_title_premise_counter_catalog_and_controls() {
        let carceral = story::carceral();
        let card = format_story_card(carceral, 0);
        assert!(card.contains(carceral.title));
        assert!(card.contains(carceral.premise));
        assert!(card.contains("0/4 beats"));
        assert!(card.contains("counties"));
        assert!(card.contains("carceral"));
        assert!(card.contains("Space"));
        assert!(card.contains("restart into the next story"));
    }

    /// task-5-review.md Minor 1: `catalog_row`'s one-line-premise fallback
    /// used to fall back to the WHOLE multi-line `premise` (not merely its
    /// un-prefixed first line) whenever the first line lacked the `"; "`
    /// prefix — unreachable on today's two shipped stories (both premises'
    /// first lines verified to start with `"; "`), but a future story
    /// without that prefix would garble the "whole catalog" listing with
    /// its entire multi-line premise embedded in one row.
    #[test]
    fn catalog_row_falls_back_to_only_the_first_line_not_the_whole_premise() {
        let fixture = Story {
            id: "fixture",
            title: "Fixture Story",
            premise: "first line has no semicolon prefix\nsecond line must not appear in the row",
            premise_source: "test fixture, not a real .bscn",
            scenario_src: "",
            rule_srcs: &[],
            session_id: "fixture",
            map_binding: None,
            arc: None,
            validated_horizon: 1,
            delays: &[],
        };
        let row = catalog_row(&fixture, &fixture);
        assert!(
            row.contains("first line has no semicolon prefix"),
            "got {row:?}"
        );
        assert!(
            !row.contains("second line must not appear in the row"),
            "the fallback must be the first line only, never the whole multi-line premise, \
             got {row:?}"
        );
    }

    #[test]
    fn format_story_card_marks_the_current_story_in_the_catalog_listing() {
        let carceral = story::carceral();
        let card = format_story_card(carceral, 0);
        let carceral_row = card
            .lines()
            .find(|l| l.contains("carceral \u{2014}"))
            .expect("carceral's own catalog row");
        assert!(carceral_row.trim_start().starts_with('*'));
        let counties_row = card
            .lines()
            .find(|l| l.contains("counties \u{2014}"))
            .expect("counties' own catalog row");
        assert!(!counties_row.trim_start().starts_with('*'));
    }
}
