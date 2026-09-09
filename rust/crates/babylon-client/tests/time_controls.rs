//! B3 wave-1 Task 2's wired-clock proof (plan §2.1/§2.2/§2.4). Headless
//! real-`App` assertions, real `KeyboardInput` messages, virtual time
//! only (`bevy::time::TimeUpdateStrategy::ManualDuration` — never
//! `Time::delta` off the wall clock, I4). RED at this commit:
//! `babylon_client::ui::time` exports none of `RunState`, `AutopauseMode`,
//! `SPEEDS_PER_SECOND`, `MAX_TICKS_PER_FRAME`, `ticks_due`, `TickPhase`,
//! `advance_ticks`, `ControlsReadout` yet.
//!
//! Row numbering below matches `task-2-brief.md` §2.2 verbatim (rows 1-8).
use babylon_client::engine_link::EngineSession;
use babylon_client::loop_ui::TickCounter;
use babylon_client::ui::admin::LastTickReport;
use babylon_client::ui::time::{ticks_due, AutopauseMode, RunState, SPEEDS_PER_SECOND};
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

/// Presses `key` through the REAL `KeyboardInput` message pipeline —
/// necessary, not stylistic, once `MapPlugin` is in the App (see every
/// other test file in this crate's own module docs for the full
/// citation: a direct `ButtonInput::press()` call from test code is wiped
/// by `InputPlugin`'s `PreUpdate` clear before an `Update` system ever
/// observes it).
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

/// The real app: `MapPlugin` + `TickLoopPlugin` together, exactly as
/// `main.rs` wires them and every other test file in this crate builds
/// them — `TickLoopPlugin`'s own Startup system orders itself
/// `.after(map::spawn_map_surface)`.
fn new_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app
}

fn controls_readout_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query =
        world.query_filtered::<&Text, With<babylon_client::ui::time::ControlsReadout>>();
    query
        .single(world)
        .expect("exactly one controls readout entity")
        .0
        .clone()
}

fn controls_readout_color(app: &mut App) -> Color {
    let world = app.world_mut();
    let mut query =
        world.query_filtered::<&TextColor, With<babylon_client::ui::time::ControlsReadout>>();
    query
        .single(world)
        .expect("exactly one controls readout entity")
        .0
}

// ---- Row 1: RunState's defaults are autoplay-until-event ----

#[test]
fn run_state_defaults_are_autoplay_until_event() {
    let mut app = new_app();
    app.update(); // Startup
    let run_state = app.world().resource::<RunState>();
    assert!(run_state.running, "the run must start moving, not paused");
    assert_eq!(
        run_state.speed_index, 2,
        "the default speed must be index 2 (5 t/s)"
    );
    assert_eq!(
        run_state.autopause,
        AutopauseMode::OnCritical,
        "the run must be primed to stop itself at the first critical beat"
    );
}

// ---- Row 2: P toggles RunState.running; the readout text changes ----

#[test]
fn p_toggles_running_and_the_readout_text_changes_to_match() {
    let mut app = new_app();
    app.update(); // Startup
    let before = controls_readout_text(&mut app);
    assert!(
        before.starts_with('\u{25b6}'),
        "the default running state must render the play glyph, got {before:?}"
    );

    press_key_via_real_event(&mut app, KeyCode::KeyP);
    app.update();
    release_key(&mut app, KeyCode::KeyP);

    assert!(!app.world().resource::<RunState>().running);
    let after = controls_readout_text(&mut app);
    assert!(
        after.starts_with("\u{275a}\u{275a} paused"),
        "pausing must render the paused glyph and word, got {after:?}"
    );
}

// ---- Row 3: at speed index 0, 2.5 intervals advance exactly 2 ticks ----

#[test]
fn at_speed_index_zero_two_point_five_intervals_advance_exactly_two_ticks() {
    let mut app = new_app();
    app.update(); // Startup
    app.world_mut().resource_mut::<RunState>().speed_index = 0; // 1 t/s -> interval 1.0s
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        2500,
    )));
    app.update();

    assert_eq!(app.world().resource::<TickCounter>().0, 2);
    let accumulator = app.world().resource::<RunState>().accumulator;
    assert!(
        (accumulator - 0.5).abs() < 1e-4,
        "0.5 intervals must remain in the accumulator, got {accumulator}"
    );
}

// ---- Row 4: ,/. move speed_index within bounds and saturate, never wrap ----

#[test]
fn comma_and_period_move_speed_index_and_saturate_at_both_ends() {
    let mut app = new_app();
    app.update(); // Startup: speed_index = 2
                  // I4: this test only cares about speed_index, but new_app()'s default
                  // RunState.running = true means every app.update() below is ALSO an
                  // auto-run frame — without a pinned zero duration, real wall-clock
                  // deltas (TimeUpdateStrategy::Automatic) would flow into
                  // RunState.accumulator and, under load, could cross a tick boundary
                  // and fire real EngineSession::advance() calls this test never asked
                  // for (fix-round M1: "wall-clock tests = determinism poison").
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));

    for _ in 0..3 {
        press_key_via_real_event(&mut app, KeyCode::Comma);
        app.update();
        release_key(&mut app, KeyCode::Comma);
    }
    assert_eq!(
        app.world().resource::<RunState>().speed_index,
        0,
        "three ',' presses from index 2 must saturate at 0, never go negative"
    );

    for _ in 0..6 {
        press_key_via_real_event(&mut app, KeyCode::Period);
        app.update();
        release_key(&mut app, KeyCode::Period);
    }
    assert_eq!(
        app.world().resource::<RunState>().speed_index,
        SPEEDS_PER_SECOND.len() - 1,
        "six '.' presses from index 0 must saturate at the top of the table, never wrap to 0"
    );
}

// ---- Row 5: Space advances exactly one tick, paused or running, never double ----

#[test]
fn space_advances_exactly_one_tick_while_paused() {
    let mut app = new_app();
    app.update(); // Startup

    press_key_via_real_event(&mut app, KeyCode::KeyP);
    app.update();
    release_key(&mut app, KeyCode::KeyP);
    assert!(!app.world().resource::<RunState>().running);

    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);

    assert_eq!(app.world().resource::<TickCounter>().0, 1);
}

#[test]
fn space_does_not_double_advance_when_auto_run_also_elapses_the_same_frame() {
    let mut app = new_app(); // running = true, speed_index = 2 by default
    app.update(); // Startup

    // A full second of injected running-time on the SAME frame a Space
    // press lands — at 5 t/s that alone would be "due" for 5 ticks if the
    // auto-run path fired too. It must not.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);

    assert_eq!(
        app.world().resource::<TickCounter>().0,
        1,
        "a Space press must win the frame outright — exactly one tick, \
         never Space's step plus the auto-run batch"
    );
}

// ---- Row 6: determinism — stepped vs. batched hash identically ----

/// The engine-level half of the claim: `EngineSession::advance()` calls
/// driven directly by `ticks_due` (the SAME pure function `advance_ticks`
/// calls, not a reimplementation) must produce the identical per-tick
/// hash sequence whether called ten times discretely or across the fewer
/// "frames" a bounded batch of ten would actually take
/// (`MAX_TICKS_PER_FRAME` caps any one call at 8, so this genuinely
/// exercises two real batches: 8 then 2).
#[test]
fn ten_discrete_advances_and_a_ticks_due_batch_of_ten_hash_identically() {
    let mut stepped =
        EngineSession::start(babylon_client::story::counties()).expect("stepped session starts");
    let mut batched =
        EngineSession::start(babylon_client::story::counties()).expect("batched session starts");

    let mut stepped_hashes = Vec::new();
    for _ in 0..10 {
        let report = stepped.advance().expect("stepped advance");
        stepped_hashes.push(report.world_after);
    }

    let interval = 1.0 / SPEEDS_PER_SECOND[0];
    let mut accumulator = 10.0 * interval;
    let mut batched_hashes = Vec::new();
    while batched_hashes.len() < 10 {
        let (due, remainder) = ticks_due(
            accumulator,
            interval,
            babylon_client::ui::time::MAX_TICKS_PER_FRAME,
        );
        assert!(
            due > 0,
            "this test's own setup must make progress every call"
        );
        accumulator = remainder;
        for _ in 0..due {
            let report = batched.advance().expect("batched advance");
            batched_hashes.push(report.world_after);
        }
    }

    assert_eq!(stepped.inner.tick(), 10);
    assert_eq!(batched.inner.tick(), 10);
    assert_eq!(
        stepped_hashes, batched_hashes,
        "10 discrete advances and one ticks_due-batched run over the same story \
         must produce identical per-tick hash sequences"
    );
}

/// The wiring-level half of the claim: the REAL `advance_ticks` system,
/// driven once by ten real `Space` presses and once by real injected
/// `Time` durations (never a keypress), must reach the identical tick
/// count and nominal world hash at tick 10.
#[test]
fn ten_space_presses_and_an_auto_run_batch_reach_the_same_wired_tick_and_hash() {
    let mut stepped_app = new_app();
    stepped_app.update(); // Startup
    press_key_via_real_event(&mut stepped_app, KeyCode::KeyP);
    stepped_app.update(); // pause — only Space may advance from here
    release_key(&mut stepped_app, KeyCode::KeyP);
    for _ in 0..10 {
        press_key_via_real_event(&mut stepped_app, KeyCode::Space);
        stepped_app.update();
        release_key(&mut stepped_app, KeyCode::Space);
    }
    let stepped_tick = stepped_app.world().resource::<TickCounter>().0;
    let stepped_hash = stepped_app
        .world()
        .resource::<LastTickReport>()
        .0
        .as_ref()
        .expect("stepped tick ten report")
        .world_after;

    let mut batched_app = new_app();
    batched_app.update(); // Startup
    batched_app
        .world_mut()
        .resource_mut::<RunState>()
        .speed_index = 0; // 1 t/s, exact interval
    batched_app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(8)));
    batched_app.update(); // 8 ticks — the MAX_TICKS_PER_FRAME ceiling
    batched_app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)));
    batched_app.update(); // 2 more ticks — tick 10
    let batched_tick = batched_app.world().resource::<TickCounter>().0;
    let batched_hash = batched_app
        .world()
        .resource::<LastTickReport>()
        .0
        .as_ref()
        .expect("batched tick ten report")
        .world_after;

    assert_eq!(stepped_tick, 10);
    assert_eq!(batched_tick, 10);
    assert_eq!(
        stepped_hash, batched_hash,
        "10 discrete Space presses and one auto-run batch of 10 must reach \
         byte-identical nominal world identity at tick 10"
    );
}

// ---- Row 7: LensChanged fires exactly once per frame, regardless of batch size ----

#[derive(Resource, Default)]
struct LensChangedCount(usize);

fn count_lens_changed(
    mut messages: MessageReader<babylon_client::map::LensChanged>,
    mut count: ResMut<LensChangedCount>,
) {
    count.0 += messages.read().count();
}

#[test]
fn lens_changed_fires_exactly_once_per_frame_even_during_a_multi_tick_batch() {
    let mut app = new_app();
    app.init_resource::<LensChangedCount>();
    app.add_systems(
        Update,
        count_lens_changed.after(babylon_client::ui::time::advance_ticks),
    );
    app.update(); // Startup — spawn_engine_session_and_hud's own tick-0 fire counts here too.
    let baseline = app.world().resource::<LensChangedCount>().0;

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        1600,
    ))); // 5 t/s default -> 8 ticks, the MAX_TICKS_PER_FRAME ceiling
    app.update();

    assert_eq!(app.world().resource::<TickCounter>().0, 8);
    assert_eq!(
        app.world().resource::<LensChangedCount>().0 - baseline,
        1,
        "one multi-tick batch frame must fire LensChanged exactly once, not once per tick"
    );
}

// ---- Row 8: heartbeat degenerate cases (Minor 4) ----

#[test]
fn paused_freezes_tick_phase_and_the_readout_stays_static() {
    let mut app = new_app();
    app.update(); // Startup
    app.world_mut().resource_mut::<RunState>().speed_index = 0; // 1 t/s

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        300,
    )));
    app.update(); // 0.3 of a 1.0s interval — no tick fires, phase ramps to 0.3
    assert_eq!(
        app.world().resource::<TickCounter>().0,
        0,
        "0.3s at 1 t/s must not fire a tick yet"
    );
    let phase_before = app
        .world()
        .resource::<babylon_client::ui::time::TickPhase>()
        .0;
    assert!((phase_before - 0.3).abs() < 1e-4, "got {phase_before}");

    press_key_via_real_event(&mut app, KeyCode::KeyP);
    app.update(); // pauses this frame
    release_key(&mut app, KeyCode::KeyP);
    let phase_at_pause = app
        .world()
        .resource::<babylon_client::ui::time::TickPhase>()
        .0;

    // Further injected time while paused must not move the phase at all.
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        900,
    )));
    app.update();
    app.update();
    let phase_still = app
        .world()
        .resource::<babylon_client::ui::time::TickPhase>()
        .0;
    assert!(
        (phase_still - phase_at_pause).abs() < 1e-9,
        "paused must freeze TickPhase exactly: {phase_at_pause} -> {phase_still}"
    );

    let text = controls_readout_text(&mut app);
    assert!(
        text.starts_with("\u{275a}\u{275a} paused"),
        "the readout must render the static paused state, got {text:?}"
    );
}

#[test]
fn a_catch_up_frame_of_more_than_one_tick_shows_the_batch_size_in_the_readout() {
    let mut app = new_app(); // running = true, speed_index = 2 (5 t/s, interval 0.2s)
    app.update(); // Startup

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        1600,
    ))); // 8 ticks due — the MAX_TICKS_PER_FRAME ceiling
    app.update();

    assert_eq!(app.world().resource::<TickCounter>().0, 8);
    let text = controls_readout_text(&mut app);
    assert!(
        text.contains("+8 ticks"),
        "a catch-up frame of 8 ticks must render the batch size, got {text:?}"
    );
    assert!(
        !text.contains("tick 8"),
        "the batch-size render must REPLACE the single-tick readout, not append to it: {text:?}"
    );
}

/// Depth beyond the brief's own 8 rows: the heartbeat's three discrete
/// palette steps (§2.1/§2.4), and that pausing freezes the COLOR too, not
/// just the numeric phase (row 8's "the readout stays static" read all
/// the way through to the rendered color, not only the text).
#[test]
fn the_heartbeat_steps_through_three_discrete_palette_colors_and_freezes_when_paused() {
    let mut app = new_app();
    app.update(); // Startup
    app.world_mut().resource_mut::<RunState>().speed_index = 0; // 1 t/s, exact thirds

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update(); // phase 0.1 -> DIM
    assert_eq!(
        controls_readout_color(&mut app),
        babylon_client::palette::DIM
    );

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        400,
    )));
    app.update(); // phase 0.5 -> BONE
    assert_eq!(
        controls_readout_color(&mut app),
        babylon_client::palette::BONE
    );

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        300,
    )));
    app.update(); // phase 0.8 -> GOLD
    assert_eq!(
        controls_readout_color(&mut app),
        babylon_client::palette::GOLD
    );

    press_key_via_real_event(&mut app, KeyCode::KeyP);
    app.update();
    release_key(&mut app, KeyCode::KeyP);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        900,
    )));
    app.update();
    assert_eq!(
        controls_readout_color(&mut app),
        babylon_client::palette::GOLD,
        "paused must freeze the heartbeat color, not just the numeric phase"
    );
}
