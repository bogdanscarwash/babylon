//! The B2 tick loop's own UI plumbing: Space advances the tick, a text
//! node shows the counter and the deterministic hash — the honesty proof
//! (III.7) rendered where the player can see it move.
//!
//! **Mechanical fixes, recorded rather than silently applied.**
//!
//! 1. The plan's own code for this task writes
//!    `mut lens_changed: EventWriter<crate::map::LensChanged>`. Bevy 0.18
//!    renamed the whole buffered pub/sub family (`map/bands.rs`'s own
//!    module doc has the full citation): `LensChanged` is a `Message`,
//!    read/written through `MessageReader`/`MessageWriter` — the
//!    `.write(...)` method call the plan's own code already uses is
//!    unchanged, since that method name carried over to the new API.
//! 2. The plan's own code constructs `crate::map::CurrentLensData { .. }`.
//!    `CurrentLensData` (and `LensReading`) are declared in `lens.rs`
//!    (Task 8's own File Structure entry), never re-exported through
//!    `map` — the correct path is `crate::lens::CurrentLensData`.
//! 3. `recolor_on_lens_changed`/`hud::refresh_hud` were deferred out of
//!    `MapPlugin` (Task 12's own recorded deviation, `map/mod.rs`'s module
//!    doc) precisely because they need `CurrentLensData`, which
//!    `TickLoopPlugin` is what actually provides — this plugin registers
//!    both, re-exported `pub(crate)` from `map` for exactly this purpose.

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::engine_link::EngineSession;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct TickCounter(pub i64);

#[derive(Component)]
pub struct HashReadout;

#[derive(Component)]
pub struct TickCounterReadout;

pub struct TickLoopPlugin;

impl Plugin for TickLoopPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickCounter::default());
        // B3 wave-1 Task 2 (plan §2.1): the clock's own state, owned by
        // `crate::ui::time`, not this module — `advance_ticks` is the
        // sole writer of all three.
        app.insert_resource(crate::ui::time::RunState::default());
        app.insert_resource(crate::ui::time::TickPhase::default());
        app.insert_resource(crate::ui::time::LastBatch::default());
        // B3 wave-1 Task 3 (plan §2.6/§3.3): the declared admin surface's
        // own state — `advance_ticks` (below) is the sole writer of
        // `LastTickReport`; `crate::ui::admin::toggle_admin_panel` is the
        // sole writer of `AdminPanelVisible`.
        app.insert_resource(crate::ui::admin::LastTickReport::default());
        app.insert_resource(crate::ui::admin::AdminPanelVisible::default());
        // B3 wave-1 Task 4 (plan §2.2): the drained, bounded beat history —
        // `advance_ticks` (via `ui::beats::drain_tick_into_beat_log`) is
        // the sole writer.
        app.insert_resource(crate::ui::beats::BeatLog::default());
        // B3 wave-1 Task 5 (plan §2.5): the tick-0 story card's own
        // visibility — `dismiss_story_card_on_first_advance` and
        // `recall_story_card_on_question_mark` are its two writers, plus
        // `restart_on_n_key` (which always re-shows it on restart).
        app.insert_resource(crate::ui::story_card::StoryCardVisible::default());
        // B3 wave-1 Task 7 (§2.11): the selected-node panel's own no-map
        // roster selection — `cycle_selected_roster_index` is the sole
        // writer, `restart_on_n_key` resets it on every restart.
        app.insert_resource(crate::ui::roster_panel::SelectedRosterIndex::default());
        // Bevy's own `Time<Virtual>` silently caps `delta_secs()` at 250ms
        // per frame (`Virtual::DEFAULT_MAX_DELTA` — its own spiral-of-death
        // protection) BEFORE `advance_ticks` ever sees it. Left at that
        // default, a single slow/stalled frame (or, in tests, an injected
        // `TimeUpdateStrategy::ManualDuration` beyond 250ms) would be
        // silently truncated by an UNDOCUMENTED 0.25s ceiling that has
        // nothing to do with this crate's own, speed-scaled stall
        // protection (`ticks_due`'s `MAX_TICKS_PER_FRAME * interval` clamp,
        // up to 8 SECONDS at the slowest 1 t/s speed) — the two clamps
        // would silently fight, and Bevy's would usually win first, making
        // `ticks_due`'s own documented bound never the operative one.
        // Raised generously (1 hour) so `ticks_due` stays the SOLE,
        // documented stall-protection mechanism. Discovered via
        // `tests/time_controls.rs`'s injected-duration rows reading zero
        // ticks at speed index 0 with 2.5s of injected time (I4).
        app.insert_resource(Time::<Virtual>::from_max_delta(
            std::time::Duration::from_secs(3600),
        ));
        // `.after(map::spawn_map_surface)`: this system fires the FIRST
        // `LensChanged` at tick 0, and `recolor_on_lens_changed` reads the
        // `MapSurface` resource `MapPlugin`'s OWN Startup system creates.
        // Bevy does not order same-schedule systems by plugin-registration
        // order alone — this ordering constraint must be explicit, not
        // implied by `main.rs` listing `MapPlugin` before `TickLoopPlugin`.
        app.add_systems(
            Startup,
            (
                spawn_engine_session_and_hud.after(crate::map::spawn_map_surface),
                spawn_state_panel,
                crate::ui::time::spawn_controls_readout,
                crate::ui::admin::spawn_admin_banner,
                crate::ui::admin::spawn_admin_panel,
                crate::ui::beats::spawn_beat_feed,
                crate::ui::beats::spawn_latch_card,
                crate::ui::story_card::spawn_story_card,
                crate::ui::story_card::spawn_map_absence_banner,
                crate::ui::countdown::spawn_countdown_pane,
            ),
        );
        app.add_systems(
            Update,
            (
                crate::ui::time::advance_ticks,
                // B3 wave-1 Task 5 (plan §2.5 I8): `N` restarts into the
                // next catalog entry — ordered right after `advance_ticks`
                // so a same-frame N-press always wins over whatever that
                // frame's own advance just did (a restart is a hard reset,
                // never something an in-flight batch should partially
                // survive).
                crate::ui::story_card::restart_on_n_key,
                crate::ui::time::refresh_controls_readout,
                refresh_readouts,
                // B3 wave-1 Task 7 (§2.11): must observe THIS frame's
                // arrow-key press before `refresh_state_panel` reads
                // `SelectedRosterIndex` — chain position, same discipline
                // as every other reader here.
                crate::ui::roster_panel::cycle_selected_roster_index,
                refresh_state_panel,
                // B3 wave-1 Task 4: retires `refresh_event_feed` — the beat
                // feed + latch card are `advance_ticks`' own `BeatLog`
                // write, read here in position (both after it in the
                // chain, same discipline as the admin panel below).
                crate::ui::beats::refresh_beat_feed,
                crate::ui::beats::refresh_latch_card,
                // B3 wave-1 Task 6 (plan §2.4/§3.3): the countdown/pressure
                // pane — reads THIS frame's `TickCounter`/`EngineSession`
                // writes from `advance_ticks`/`restart_on_n_key` above, same
                // chain-position discipline as every other reader here.
                crate::ui::countdown::refresh_countdown_pane,
                // B3 wave-1 Task 3: `toggle_admin_panel` must observe THIS
                // frame's F3 press before `refresh_admin_panel` reads
                // visibility, and `refresh_admin_panel` must observe THIS
                // frame's `advance_ticks` write to `LastTickReport` — both
                // are satisfied by position alone inside one `.chain()`,
                // the same discipline every other reader here already
                // follows.
                crate::ui::admin::toggle_admin_panel,
                crate::ui::admin::refresh_admin_panel,
                // B3 wave-1 Task 5: dismiss-on-advance must see THIS
                // frame's `advance_ticks`/`restart_on_n_key` writes to
                // `TickCounter` before it reads it; `refresh_story_card`
                // must see THIS frame's `BeatLog`/visibility writes before
                // it renders — both satisfied by chain position, same
                // discipline as every reader above.
                crate::ui::story_card::dismiss_story_card_on_first_advance,
                crate::ui::story_card::recall_story_card_on_question_mark,
                crate::ui::story_card::refresh_story_card,
            )
                .chain(),
        );
        // B3 wave-1 Task 5 (plan §2.11): reactive on
        // `EngineSession.is_changed()` — fires once after Startup inserts
        // the launched story's session, and again whenever
        // `restart_on_n_key` swaps in a fresh one. `.after(...)` both: the
        // Startup ordering is automatic (Startup fully precedes Update),
        // named here only for the SAME-FRAME `N`-press case.
        app.add_systems(
            Update,
            crate::ui::story_card::sync_map_to_story.after(crate::ui::story_card::restart_on_n_key),
        );
        // Deferred here from MapPlugin (Task 12's recorded deviation) —
        // both need Res<CurrentLensData>, which spawn_engine_session_and_hud
        // (above) inserts at Startup, strictly before either can run as an
        // Update system.
        //
        // `.after(crate::ui::time::advance_ticks)` (renamed from
        // `.after(advance_on_space)` when B3 wave-1 Task 2 replaced that
        // system — plan §2.3, the fix itself is untouched): without this,
        // Bevy may schedule either system BEFORE `advance_ticks` on a
        // given frame (no ordering is implied by two separate
        // `add_systems` calls) — a press that advances the tick and
        // writes THIS frame's `LensChanged` would then go unseen by a
        // recolor/HUD pass that already ran, deferring the repaint to the
        // NEXT press instead (an off-by-one-frame lag). Verified:
        // `eyes_on_smoke.rs`'s
        // `a_known_demo_county_actually_recolors_after_a_space_press` failed
        // with `before == after` until this ordering was added — the exact
        // failure mode this comment describes, caught by that test rather
        // than assumed fixed.
        //
        // `.after(crate::map::cycle_lens_on_tab)` (FB7, adversarial-panel
        // MINOR): the SAME class of gap, cross-plugin — a Tab press
        // (`MapPlugin`'s own system) writes `LensChanged` too, and without
        // this ordering the recolor/HUD pass could run before it in a given
        // frame, deferring the repaint by one press. Less severe than the
        // Space case (self-correcting the very next frame regardless, and
        // `eyes_on_smoke.rs`'s own Tab-cycle test pumps one `app.update()`
        // per press so it never observed the lag), but trivially
        // expressible with the same fix once `cycle_lens_on_tab` is
        // `pub(crate)`, so fixed rather than merely noted.
        app.add_systems(
            Update,
            (crate::map::recolor_on_lens_changed, crate::map::refresh_hud)
                .after(crate::ui::time::advance_ticks)
                .after(crate::map::cycle_lens_on_tab)
                // B3 wave-1 Task 5: `restart_on_n_key` also rebuilds
                // `CurrentLensData` and fires `LensChanged` — without this,
                // a same-frame N-press could recolor against the STALE
                // pre-restart lens data (the two systems conflict on
                // `CurrentLensData`, so SOME order applies regardless; this
                // names the correct one explicitly rather than leaving it
                // to chance).
                .after(crate::ui::story_card::restart_on_n_key),
        );
    }
}

/// Builds this session's tick-0 (or post-restart) `CurrentLensData` — a
/// shared core so `spawn_engine_session_and_hud`'s own Startup build,
/// `ui::time::advance_ticks`'s own post-batch recompute and
/// `ui::story_card`'s `N`-key restart all compute it identically (B3
/// wave-1 Task 8, §2.10: the ONE place that iterates `crate::map::LENSES`,
/// replacing what used to be three separate call sites each naming all
/// three lens functions by hand). Safe to call unconditionally regardless
/// of `session.story.map_binding`: `roster` is naturally empty for a
/// `None`-binding story (§2.11), and both `county_legitimation`/
/// `county_population_trend` are no-ops (empty `cells`, no panic) over an
/// empty roster — the same code path that derives the map-absence story
/// derives the lens-absence one too, with no branch needed.
#[must_use]
pub(crate) fn build_lens_data(session: &EngineSession) -> crate::lens::CurrentLensData {
    let inputs = crate::lens::LensInputs {
        graph: session.inner.graph(),
        roster: &session.roster,
        baseline: &session.population_baseline,
    };
    crate::lens::CurrentLensData(
        crate::map::LENSES
            .iter()
            .map(|spec| (spec.compute)(&inputs))
            .collect(),
    )
}

fn spawn_engine_session_and_hud(
    mut commands: Commands,
    selected: Res<crate::story::SelectedStory>,
    mut lens_changed: MessageWriter<crate::map::LensChanged>,
) {
    let session = EngineSession::start(selected.0)
        .unwrap_or_else(|e| panic!("engine session failed to start: {e}"));
    // Tick 0's own LensReadings — the map must show something correct
    // (or correctly absent) on first launch, before any Space press. The
    // Population Trend reading is `Some(0.0)` (DIM) everywhere at this
    // point, since `population_baseline` IS the tick-0 state — real
    // divergence appears only after the first `advance()`.
    let lens_data = build_lens_data(&session);
    commands.insert_resource(lens_data);
    commands.insert_resource(session);
    lens_changed.write(crate::map::LensChanged);
    commands.spawn((
        Text::new("tick 0"),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            right: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TickTransport),
        TickCounterReadout,
    ));
    commands.spawn((
        Text::new("hash: (not yet run)"),
        TextColor(crate::palette::DIM),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(4),
            right: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TickTransport),
        HashReadout,
    ));
}

// `advance_on_space` (B2 Task 14) lived here until B3 wave-1 Task 2 (plan
// §2.1/§2.3) replaced it with `crate::ui::time::advance_ticks` — the same
// single-tick advance folded into a bounded catch-up path with the
// play/pause/speed bindings. See that module for the system itself; the
// two ordering fixes that named the old function by identifier are above,
// renamed to match, reasoning unchanged.

fn refresh_readouts(
    counter: Res<TickCounter>,
    last_report: Res<crate::ui::admin::LastTickReport>,
    mut tick_text: Query<&mut Text, (With<TickCounterReadout>, Without<HashReadout>)>,
    mut hash_text: Query<&mut Text, With<HashReadout>>,
) {
    if !counter.is_changed() {
        return;
    }
    if let Ok(mut t) = tick_text.single_mut() {
        t.0 = format!("tick {}", counter.0);
    }
    if let Ok(mut h) = hash_text.single_mut() {
        h.0 = match last_report.0.as_ref() {
            Some(report) => format!("hash: {}", babylon_tick::hex(&report.world_after)),
            None => "hash: (not yet run)".to_owned(),
        };
    }
}

// ---- Task 15: the state panel ----
// B3 wave-1 Task 4 retires the event feed that used to live here
// (`EVENT_FEED_DEPTH`/`EventFeedText`/`payload_node_id`/
// `refresh_event_feed`) — the drained, severity-ranked, tick-stamped beat
// feed (`crate::ui::beats`) replaces it (plan §2.2/§2.4).

use crate::atlas::CountyAtlas;
use babylon_graph::substrate::NodeId;

#[derive(Component)]
pub struct StatePanelText;

fn spawn_state_panel(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            top: px(24),
            right: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::StatePanel),
        StatePanelText,
    ));
}

/// Resolves `SelectedCounty`'s ATLAS INDEX (Task 11's own vocabulary,
/// never a `NodeId`) through `atlas.county(idx).fips` to a `(fips, NodeId)`
/// pair via a linear scan of `roster` (twelve entries on the counties
/// story — not worth a `HashMap` at this size, matching Task 13's own
/// call). `None` covers BOTH "nothing selected yet" and "selected a
/// non-demo county" — callers distinguish the two by checking `selected.0`
/// directly when they need to.
///
/// `pub(crate)` (B3 wave-1 Task 3): `ui::admin::refresh_admin_panel` shares
/// this resolution rather than re-deriving it — the same node the state
/// panel describes is the node the admin roster dump describes.
pub(crate) fn selected_demo_node(
    atlas: &CountyAtlas,
    selected: &crate::map::SelectedCounty,
    roster: &[(String, NodeId)],
) -> Option<(String, String, NodeId)> {
    let idx = selected.0?;
    let county = atlas.county(idx)?;
    let (fips, id) = roster.iter().find(|(f, _)| f == county.fips)?;
    Some((fips.clone(), county.name.to_owned(), *id))
}

/// `Update` system: repaints the state panel from `SelectedCounty` — live
/// `pop-d`/`pop-p`/`pop-d-prime`/`legitimation-index` read straight off
/// the graph, proving the panel and the map agree because both read the
/// SAME graph. Reads the atlas through `Res<CountyAtlas>` (FB5 fix — this
/// system ran EVERY Update frame unconditionally, even with nothing
/// selected, and used to re-parse the embedded 1.7 MB atlas on every one
/// of them; `map::mesh::spawn_map_surface` parses it exactly once, at
/// Startup, and this system now shares that parse).
///
/// **B3 wave-1 Task 7 (§2.11).** For a `MapBinding::None` story (no county
/// to click) this delegates to `ui::roster_panel::format_roster_panel`
/// instead — the SAME `StatePanelText` entity, a second sourcing path, not
/// a second panel.
fn refresh_state_panel(
    selected: Res<crate::map::SelectedCounty>,
    roster_selected: Res<crate::ui::roster_panel::SelectedRosterIndex>,
    session: Res<EngineSession>,
    atlas: Res<CountyAtlas>,
    mut panel_text: Query<&mut Text, With<StatePanelText>>,
) {
    let Ok(mut text) = panel_text.single_mut() else {
        return;
    };
    if session.story.map_binding.is_none() {
        text.0 = crate::ui::roster_panel::format_roster_panel(
            session.inner.graph(),
            &session.full_roster,
            roster_selected.0,
        );
        return;
    }
    text.0 = match selected_demo_node(&atlas, &selected, &session.roster) {
        Some((fips, name, id)) => {
            let graph = session.inner.graph();
            // B3 wave-1 Task 3 (plan §2.6): retargeted through the seam —
            // none of these four territory fields is in the projector's
            // `NotComputed` table, so `.value` behaves exactly like the
            // `Result` this replaces: `Material` -> `Some`, `Absent` ->
            // `None`, and the fallback text below is unchanged.
            let projector = crate::projection::Projector::material();
            let pop_d = projector.read(graph, id, "territory/pop-d").value;
            let pop_p = projector.read(graph, id, "territory/pop-p").value;
            let pop_d_prime = projector.read(graph, id, "territory/pop-d-prime").value;
            let legit_class = projector
                .read(graph, id, "territory/legitimation-crisis")
                .value;
            match (pop_d, pop_p, pop_d_prime, legit_class) {
                (Some(d), Some(p), Some(dp), Some(class)) => {
                    let word = match crate::lens::classify(class) {
                        crate::lens::LegitimationClass::Stable => "STABLE",
                        crate::lens::LegitimationClass::Unstable => "UNSTABLE",
                        crate::lens::LegitimationClass::Crisis => "CRISIS",
                    };
                    format!(
                        "{name} ({fips})\n  pop-d:       {d:.0}\n  pop-p:       {p:.0}\n  \
                         pop-d-prime: {dp:.0}\n  legitimation: {word} ({class:.0})"
                    )
                }
                _ => "no data this tick".to_owned(),
            }
        }
        None if selected.0.is_some() => "no data this tick".to_owned(),
        None => String::new(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    // Production code no longer calls a `GraphSubstrate` trait method
    // directly anywhere in this file (B3 wave-1 Task 3 retargeted every
    // read through `crate::projection::Projector`) — this test module's
    // own direct `.node_attribute()` call (below) is the one remaining
    // call site, so the trait import lives here rather than at file scope,
    // where it would be an unused-import warning on the non-test build.
    use babylon_graph::substrate::GraphSubstrate;

    /// Presses `key` through a REAL `KeyboardInput` message — necessary,
    /// not stylistic, once `crate::map::MapPlugin` is in the App (it
    /// conditionally self-adds `InputPlugin`, whose `PreUpdate`
    /// `keyboard_input_system` unconditionally clears `just_pressed` every
    /// frame; a direct `ButtonInput::press()` call from test code gets
    /// wiped before an `Update` system ever observes it — the gotcha
    /// `map/mod.rs`'s own module doc names in full).
    fn press_key_via_real_event(app: &mut App, key: bevy::input::keyboard::KeyCode) {
        use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
        use bevy::input::ButtonState;
        app.world_mut()
            .resource_mut::<Messages<KeyboardInput>>()
            .write(KeyboardInput {
                key_code: key,
                logical_key: Key::Unidentified(NativeKey::Unidentified),
                state: ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
    }

    fn release_key(app: &mut App, key: bevy::input::keyboard::KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<bevy::input::keyboard::KeyCode>>()
            .release(key);
    }

    /// FB3 fix (adversarial-panel finding, mutation-proven): the deleted
    /// predecessor of this test called `selected_demo_node` (a pure
    /// helper) and read the graph directly — it never ran
    /// `refresh_state_panel` (the actual PRODUCTION system) or looked at
    /// the `StatePanelText` component a player actually sees. Gutting the
    /// panel's format string in `refresh_state_panel` left the deleted
    /// test fully green. This version builds a real App
    /// (`crate::map::MapPlugin` + `TickLoopPlugin`), advances two REAL
    /// ticks through `advance_ticks`, sets `SelectedCounty` directly
    /// (this crate's own established precedent for driving a picking
    /// resource — `map::pick`'s own hover test does the same for
    /// `CursorWorldPosition`), and reads the ACTUAL rendered `Text`.
    #[test]
    fn state_panel_renders_live_numbers_for_a_selected_demo_county_after_two_ticks() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.add_plugins(crate::map::MapPlugin);
        app.add_plugins(TickLoopPlugin);
        app.insert_resource(crate::story::SelectedStory(crate::story::counties()));
        app.update(); // Startup

        for _ in 0..2 {
            press_key_via_real_event(&mut app, bevy::input::keyboard::KeyCode::Space);
            app.update();
            release_key(&mut app, bevy::input::keyboard::KeyCode::Space);
        }

        app.world_mut()
            .resource_mut::<crate::map::SelectedCounty>()
            .0 = Some(0); // atlas index 0 = roster[0] = fips 01001
        app.update(); // let refresh_state_panel run against the real selection

        let session = app.world().resource::<EngineSession>();
        let id = session.roster[0].1;
        let pop_d = session
            .inner
            .graph()
            .node_attribute(id, "territory/pop-d")
            .expect("pop-d readable");
        // Task 9b's own table: county family "core" (x0.95) nets DECLINING
        // — pop-d moves away from its seeded 2042 by tick 2. Exact
        // comparison against the literal seed value is the correct check —
        // an epsilon would hide the case where the tick's math regressed
        // to a no-op. Block-scoped (not a statement-level attribute on the
        // macro invocation itself): `assert_ne!`'s internally-generated
        // comparison isn't in the invocation's own lint scope, only the
        // enclosing block's.
        #[allow(clippy::float_cmp)]
        {
            assert_ne!(pop_d, 2042.0);
        }

        let world = app.world_mut();
        let mut query = world.query_filtered::<&Text, With<StatePanelText>>();
        let text = query
            .single(world)
            .expect("exactly one state panel entity")
            .0
            .clone();
        assert!(
            text.contains(&format!("{pop_d:.0}")),
            "state panel text {text:?} must contain the live pop-d value {pop_d:.0} — \
             if this fails while the pure-helper checks above pass, refresh_state_panel \
             itself is not reaching the Text component"
        );
    }
    // The event-feed's own real-App coverage moved to `crate::ui::beats`
    // (B3 wave-1 Task 4, plan §2.2/§2.4) — see `tests/beats.rs::
    // legitimation_recovery_renders_a_tick_stamped_sentence_not_the_terse_at_format`,
    // which exercises the SAME LEGITIMATION_RECOVERY-on-tick-1 archetype
    // through the new `refresh_beat_feed` system instead of this file's
    // retired `refresh_event_feed`.
}
