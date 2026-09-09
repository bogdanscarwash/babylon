//! B3 wave-1 Task 4.5's autopause + latch-card proof (plan §3.6/C2):
//! `B` = run-to-next-beat, unconditional pause on `TERMINAL_DECISION`, and
//! the latch card — never an end card, never a verdict, never the
//! five-outcome vocabulary. Headless real-`App`, real `KeyboardInput`,
//! virtual time only (I4), same house pattern as every other test file in
//! this crate.
//!
//! **B3 wave-1 Task 5 update.** The story catalog now exists — this file
//! launches the carceral story the SAME way `main.rs`'s `--story carceral`
//! flag would: `SelectedStory(story::carceral())` inserted before Startup,
//! so `spawn_engine_session_and_hud` builds the real carceral session
//! directly. Replaces the Task-4-era `EngineSession::start_over` narrow
//! seam (deleted; absorbed into `EngineSession::start(story)`, exactly as
//! its own doc comment named) and the post-Startup counties-then-swap
//! hack it required.

use babylon_client::story;
use babylon_client::ui::time::{AutopauseMode, RunState};
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

/// Builds a real App launched directly on the carceral story — exactly
/// `main.rs`'s own `--story carceral` wiring, not a post-Startup swap.
fn new_carceral_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(babylon_client::story::SelectedStory(story::carceral()));
    app.update(); // Startup — spawns the real carceral EngineSession.
    app
}

/// Presses `B` (run-to-next-beat) and pumps `app.update()` — with a
/// GENEROUS injected duration each frame so `ticks_due`'s own
/// `MAX_TICKS_PER_FRAME` batches through as fast as possible — until
/// `RunState.running` goes false (autopaused) or `max_frames` is
/// exhausted (Power-of-10 rule 2: a real, finite bound; 40 frames is
/// comfortably more than `ceil(110 / MAX_TICKS_PER_FRAME)` = 14).
fn press_b_and_run_to_next_beat(app: &mut App) {
    // I4: pin the B-press's own update to zero injected sim time — this
    // frame only needs to flip `RunState.running`/`autopause`, not advance
    // anything, and leaving `TimeUpdateStrategy` at its (real-wall-clock)
    // default here would be exactly the determinism-poison class this
    // crate has already paid for once (`program-15-gauntlet`).
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    press_key_via_real_event(app, KeyCode::KeyB);
    app.update();
    release_key(app, KeyCode::KeyB);

    for _ in 0..40 {
        if !app.world().resource::<RunState>().running {
            return;
        }
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)));
        app.update();
    }
    panic!("run-to-next-beat did not autopause within 40 frames — a beat is missing or the autopause wiring is broken");
}

fn latch_card_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<babylon_client::ui::beats::LatchCardText>>();
    query.single(world).map(|t| t.0.clone()).unwrap_or_default()
}

#[test]
fn b_runs_to_each_carceral_beat_in_turn_and_the_fourth_renders_the_latch_card() {
    let mut app = new_carceral_app();

    press_b_and_run_to_next_beat(&mut app);
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        1,
        "the first B press must stop at tick 1 (SUPERWAGE_CRISIS, critical)"
    );
    assert!(!app.world().resource::<RunState>().running);

    press_b_and_run_to_next_beat(&mut app);
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        53,
        "the second B press must stop at tick 53 (CLASS_DECOMPOSITION)"
    );

    press_b_and_run_to_next_beat(&mut app);
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        105,
        "the third B press must stop at tick 105 (CONTROL_RATIO_CRISIS)"
    );

    press_b_and_run_to_next_beat(&mut app);
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        106,
        "the fourth B press must stop at tick 106 (TERMINAL_DECISION)"
    );

    let card = latch_card_text(&mut app);
    assert!(
        !card.is_empty(),
        "the latch card must render on TERMINAL_DECISION"
    );

    // Assert the negative too (§4.5, the fog/epistemic discipline made
    // executable — never weaken this list): the five-outcome vocabulary,
    // and the words "verdict"/"campaign", must never appear.
    for forbidden in [
        "REVOLUTIONARY_VICTORY",
        "FASCIST_CONSOLIDATION",
        "RED_OGV",
        "ECOLOGICAL_COLLAPSE",
        "FRAGMENTED_COLLAPSE",
        "verdict",
        "campaign",
    ] {
        assert!(
            !card.to_lowercase().contains(&forbidden.to_lowercase()),
            "the latch card must never render {forbidden:?} — got {card:?}"
        );
    }

    // Resume: P must un-pause and the run must be able to advance again.
    // I4: pin this update's own injected time to zero too — otherwise the
    // PREVIOUS loop iteration's `ManualDuration(2s)` is still the active
    // strategy, and resuming (`running` flips true) would let that stale
    // 2s immediately batch several more ticks on THIS SAME frame.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    press_key_via_real_event(&mut app, KeyCode::KeyP);
    app.update();
    release_key(&mut app, KeyCode::KeyP);
    assert!(
        app.world().resource::<RunState>().running,
        "P must resume the run after the latch"
    );
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        107,
        "the run must be able to advance past the latch"
    );
}

#[test]
fn autopause_mode_never_does_not_stop_on_a_critical_beat() {
    let mut app = new_carceral_app();
    // Startup's own RunState default is already `running = true` — carried
    // over verbatim; only `autopause` is overridden for this test.
    app.world_mut().resource_mut::<RunState>().autopause = AutopauseMode::Never;

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)));
    app.update(); // 8 ticks due (the MAX_TICKS_PER_FRAME ceiling), tick 1's SUPERWAGE_CRISIS among them

    assert!(
        app.world().resource::<RunState>().running,
        "AutopauseMode::Never must not stop the batch on a merely-critical beat"
    );
    assert_eq!(
        app.world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        8,
        "with autopause disabled the full 8-tick batch must complete, unhindered by tick 1's beat"
    );
}
