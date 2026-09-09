//! B3 wave-1 Task 6's own RED phase (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.4/§3.3): the countdown/pressure pane (6.1), the HUD's `B` hint (6.2),
//! and the executable cadence gate (6.3) — the wave-1 cadence claim, made
//! byte-checkable, for each shipped story.
//!
//! **What "the rendered HUD" means for 6.3.** This crate's own module
//! (`map/hud.rs`, "the county HUD") names `CountyHudText` as the thing that
//! literal noun refers to — never `ui::time::ControlsReadout` (this crate's
//! own doc calls THAT "the controls readout"). This distinction is
//! load-bearing, not cosmetic: `ControlsReadout` has carried the live tick
//! number since Task 2 (`▶ 5 t/s · tick N`), so capturing it here would make
//! the cadence gate pass trivially on the raw tick counter alone — for
//! EVERY story, even a broken one, even before this task's own countdown
//! pane existed. `CountyHudText` carries no such trivial motion: it is
//! empty whenever nothing is hovered/selected (always true for carceral,
//! which has no map — §2.11), so the whole carceral leg of this gate rests
//! on the countdown pane, genuinely. For counties the county HUD's own
//! Legitimation/PopulationTrend lens lines ALSO carry a bare
//! "(live, tick N)" suffix (`map/hud.rs::format_lens_line`) — the SAME
//! trivial-motion risk one level down — so the counties leg below pins
//! `ActiveLens(0)` (Tension) explicitly (Tension's own line carries no tick
//! suffix, `format_lens_line`'s own match arm), and a SEPARATE dedicated
//! test isolates the state panel alone (6.4's own "assert that path
//! separately so 6.3 cannot pass for counties by accident of some
//! unrelated moving glyph").
//!
//! RED→GREEN record: authored RED when none of
//! `babylon_client::ui::countdown`'s items existed (every reference below
//! failed to resolve, mirroring the `d4f353d9`/`c48d752c` "module absent"
//! RED-commit precedent this train used for every prior task); GREEN since
//! the countdown pane landed — this file is now the countdown pane's +
//! cadence gate's live regression guard.

use babylon_client::loop_ui::{StatePanelText, TickCounter};
use babylon_client::map::{ActiveLens, CountyHudText, SelectedCounty};
use babylon_client::story;
use babylon_client::ui::countdown::CountdownPaneText;
use babylon_client::ui::time::ControlsReadout;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn press_key_via_real_event(app: &mut App, key: KeyCode) {
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

fn release_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

/// Builds a real App launched directly on `story` — the same
/// `SelectedStory`-before-Startup wiring `main.rs`'s `--story` flag and
/// `tests/autopause.rs::new_carceral_app` both use.
fn new_app_for(story: &'static story::Story) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(story::SelectedStory(story));
    app.update(); // Startup
    app
}

/// Advances exactly one tick via a real `Space` press — I4: pinned to zero
/// injected sim time so ONLY the Space press's own unconditional
/// one-tick-and-reset-the-accumulator path fires, never `RunState.running`'s
/// own wall-clock-driven batch (`ui::time::advance_ticks`'s own doc).
fn advance_one_tick(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    press_key_via_real_event(app, KeyCode::Space);
    app.update();
    release_key(app, KeyCode::Space);
}

fn text_of<T: Component>(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<T>>();
    query.single(world).map(|t| t.0.clone()).unwrap_or_default()
}

/// The longest run of consecutive, byte-identical captures — the cadence
/// gate's own evidence metric (§6's "the per-story cadence evidence, the
/// longest identical window found, per story"). Loop bound: `captures.len()`
/// (Power-of-10 rule 2).
fn longest_identical_run(captures: &[String]) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    let mut last: Option<&String> = None;
    for s in captures {
        current = if last == Some(s) { current + 1 } else { 1 };
        longest = longest.max(current);
        last = Some(s);
    }
    longest
}

// ---- 6.1: the countdown pane, wired ----

#[test]
fn the_wired_countdown_pane_renders_not_yet_latched_with_no_digit_at_tick_zero() {
    let mut app = new_app_for(story::carceral());
    let rendered = text_of::<CountdownPaneText>(&mut app);
    assert!(rendered.contains("CLASS_DECOMPOSITION"), "got {rendered:?}");
    assert!(rendered.contains("not yet latched"), "got {rendered:?}");
    assert!(
        !rendered.chars().any(|c| c.is_ascii_digit()),
        "the seeded-0 trap (§2.4): before the latch flag flips the row must contain no digit \
         that could read as a countdown, got {rendered:?}"
    );
}

#[test]
fn the_wired_countdown_pane_reads_39_ticks_at_tick_14_naming_both_operands() {
    let mut app = new_app_for(story::carceral());
    for _ in 0..14 {
        advance_one_tick(&mut app);
    }
    assert_eq!(app.world().resource::<TickCounter>().0, 14);
    let rendered = text_of::<CountdownPaneText>(&mut app);
    assert!(rendered.contains("CLASS_DECOMPOSITION"), "got {rendered:?}");
    assert!(rendered.contains("39 ticks"), "got {rendered:?}");
    assert!(
        rendered.contains("superwage-crisis-tick 1"),
        "the live operand, read off the graph, must be named: got {rendered:?}"
    );
    assert!(
        rendered.contains("carceral/decomposition-delay 52"),
        "the declared constant, from the story's own DeclaredConst, must be named: got \
         {rendered:?}"
    );
    assert!(
        rendered.contains("carceral-arc-conformance.bscn:137"),
        "the declared constant's own .bscn cite must appear: got {rendered:?}"
    );
    assert!(rendered.contains("tick 14"), "got {rendered:?}");
}

#[test]
fn after_class_decomposition_fires_its_row_retires_and_control_ratio_crisis_appears() {
    let mut app = new_app_for(story::carceral());
    for _ in 0..53 {
        advance_one_tick(&mut app);
    }
    let rendered = text_of::<CountdownPaneText>(&mut app);
    assert!(
        !rendered.contains("CLASS_DECOMPOSITION"),
        "the retired row must not still render: got {rendered:?}"
    );
    assert!(
        rendered.contains("CONTROL_RATIO_CRISIS"),
        "got {rendered:?}"
    );
    assert!(rendered.contains("in 52 ticks"), "got {rendered:?}");
    assert!(
        rendered.contains("decomposition-fire-tick 53"),
        "got {rendered:?}"
    );
    assert!(
        rendered.contains("carceral/control-ratio-delay 52"),
        "got {rendered:?}"
    );
}

#[test]
fn after_terminal_decision_fires_the_pane_is_empty() {
    let mut app = new_app_for(story::carceral());
    for _ in 0..108 {
        advance_one_tick(&mut app);
    }
    let rendered = text_of::<CountdownPaneText>(&mut app);
    assert_eq!(
        rendered, "",
        "every declared beat has fired — the pane must render nothing, never a stale row"
    );
}

#[test]
fn counties_declares_no_delays_so_the_pane_stays_empty() {
    let mut app = new_app_for(story::counties());
    for _ in 0..5 {
        advance_one_tick(&mut app);
    }
    assert_eq!(text_of::<CountdownPaneText>(&mut app), "");
}

// ---- 6.2: the HUD's `B` hint ----

#[test]
fn hud_b_hint_reads_next_beat_in_39_ticks_when_a_countdown_is_live() {
    let mut app = new_app_for(story::carceral());
    for _ in 0..14 {
        advance_one_tick(&mut app);
    }
    let readout = text_of::<ControlsReadout>(&mut app);
    assert!(
        readout.contains("B \u{2192} next beat in 39 ticks"),
        "got {readout:?}"
    );
}

#[test]
fn hud_b_hint_is_omitted_before_the_latch_flips() {
    let mut app = new_app_for(story::carceral());
    let readout = text_of::<ControlsReadout>(&mut app);
    assert!(
        !readout.contains("next beat"),
        "never a placeholder hint before the countdown is computable, got {readout:?}"
    );
    assert!(!readout.contains("in ? ticks"), "got {readout:?}");
}

#[test]
fn hud_b_hint_is_omitted_once_every_declared_beat_has_fired() {
    let mut app = new_app_for(story::carceral());
    for _ in 0..108 {
        advance_one_tick(&mut app);
    }
    let readout = text_of::<ControlsReadout>(&mut app);
    assert!(!readout.contains("next beat"), "got {readout:?}");
}

#[test]
fn hud_b_hint_is_omitted_for_counties_which_declares_no_delays() {
    let mut app = new_app_for(story::counties());
    for _ in 0..5 {
        advance_one_tick(&mut app);
    }
    let readout = text_of::<ControlsReadout>(&mut app);
    assert!(!readout.contains("next beat"), "got {readout:?}");
}

// ---- 6.3: the cadence gate (§2.4) ----

/// The executable form of the wave-1 cadence claim: across a headless
/// auto-run of EACH shipped story to its own validated horizon, no 20-tick
/// window of (county HUD + state panel + countdown pane) rendered text is
/// byte-identical. `println!` lines carry the per-story evidence for the PR
/// body (run with `--nocapture` to see them) — the brief's own "the longest
/// identical window found, per story" requirement.
#[test]
fn no_20_tick_window_renders_byte_identical_hud_state_and_countdown_text_for_each_story() {
    for story in story::STORIES {
        let mut app = new_app_for(story);
        if story.map_binding.is_some() {
            // Tension (never Legitimation/PopulationTrend, both of which
            // carry their own "(live, tick N)" suffix — see the module
            // doc's own trivial-motion warning) selects county atlas index
            // 0 == roster[0] == fips 01001 (the same mapping
            // `loop_ui.rs`'s own `state_panel_renders_live_numbers_...`
            // test relies on).
            app.world_mut().resource_mut::<SelectedCounty>().0 = Some(0);
            *app.world_mut().resource_mut::<ActiveLens>() = ActiveLens(0); // Tension
        }

        let mut captures = Vec::with_capacity(usize::try_from(story.validated_horizon).unwrap());
        for _ in 0..story.validated_horizon {
            advance_one_tick(&mut app);
            let hud = text_of::<CountyHudText>(&mut app);
            let state = text_of::<StatePanelText>(&mut app);
            let countdown = text_of::<CountdownPaneText>(&mut app);
            captures.push(format!("{hud}\n{state}\n{countdown}"));
        }

        let longest = longest_identical_run(&captures);
        println!(
            "cadence evidence: story {:?} longest identical window = {longest} ticks \
             (validated horizon {})",
            story.id, story.validated_horizon
        );
        assert!(
            longest < 20,
            "story {:?}: a {longest}-tick window rendered byte-identical HUD+state+countdown \
             text — the wave-1 cadence claim (§2.4) requires no 20-tick window ever be frozen, \
             and a story that cannot pass this does not ship",
            story.id
        );
    }
}

/// §6.4's own requirement: assert the counties per-tick-delta path
/// SEPARATELY, isolated from the county HUD's own lens line and from the
/// (always-empty, counties declares no delays) countdown pane — so the main
/// gate above cannot be read as passing for counties "by accident of some
/// unrelated moving glyph." The state panel alone (`pop-d`/`pop-p`/
/// `pop-d-prime`, `lifecycle.bsl:383-387`'s own every-tick adjustment) must
/// already satisfy the same no-20-tick-frozen-window standard on its own.
#[test]
fn counties_state_panel_alone_satisfies_the_no_frozen_window_standard() {
    let counties = story::counties();
    let mut app = new_app_for(counties);
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(0);

    let mut captures = Vec::with_capacity(usize::try_from(counties.validated_horizon).unwrap());
    for _ in 0..counties.validated_horizon {
        advance_one_tick(&mut app);
        captures.push(text_of::<StatePanelText>(&mut app));
    }

    let longest = longest_identical_run(&captures);
    println!(
        "counties per-tick-delta evidence (state panel alone): longest identical window = \
         {longest} ticks (validated horizon {})",
        counties.validated_horizon
    );
    assert!(
        longest < 20,
        "counties' own per-tick-delta path (§2.4) must alone satisfy the no-20-tick-frozen- \
         window cadence claim, isolated from every other on-screen instrument — got a \
         {longest}-tick identical window using the state panel alone"
    );
}
