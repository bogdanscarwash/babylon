//! The scriptable half of the B2 eyes-on gate (Task 18). Never a
//! replacement for the human pass (nothing here can see a color on
//! screen), but it proves every INPUT-DRIVEN, hash-provable claim that
//! pass makes, so a future change that silently breaks the loop reds THIS
//! gate in CI before a human ever has to notice by eye.
//!
//! **Mechanical fixes, recorded rather than silently applied.**
//!
//! 1. Event-type strings at the sink boundary are BARE enum members —
//!    `"LIFECYCLE_TRANSITION"`, `"ENTITY_DEATH"` — never
//!    `"EventType/LIFECYCLE_TRANSITION"`/`"EventType/ENTITY_DEATH"`, the
//!    literal spelling the plan's own code block used. Verified against
//!    every live event-name assertion in `babylon-tick`'s own conformance
//!    tests (`lifecycle_conformance.rs`, `vitality_conformance.rs`) and
//!    against `structural_verbs::EffectExecutor::enum_member`, which
//!    returns the bare `Atom::EnumRef { member, .. }` — the same gotcha
//!    Task 7's `us_counties_demo.rs` and this plan's handoff notes already
//!    name (bit the plan twice before this).
//! 2. Directly mutating `ButtonInput` (`.press()`/`.release()`) from test
//!    code, then calling `app.update()`, does not reliably drive
//!    `just_pressed` here: `MapPlugin` conditionally self-adds
//!    `InputPlugin`, whose `PreUpdate` `keyboard_input_system`
//!    unconditionally clears `just_pressed` every frame to make room for
//!    real events, wiping a manually-set flag before an `Update`-scheduled
//!    system (`advance_ticks`/`cycle_lens_on_tab`) ever observes it —
//!    the same gotcha `map/mod.rs`'s own module doc and Tasks 11/12/14
//!    already found and fixed. Pressing through a REAL `KeyboardInput`
//!    message (`Entity::PLACEHOLDER` for the unread `window` field) lets
//!    `keyboard_input_system`'s own event-driven logic set `just_pressed`
//!    correctly; releasing directly between presses is safe (`clear()`
//!    never touches the `pressed` set, only `just_pressed`/`just_released`).
use babylon_graph::state_hash::CanonicalState; // trait import — .state_hash() below needs it in scope
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use std::collections::HashSet;

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

/// Reads county 0's (atlas index 0 = `roster[0]`, fips 01001) own vertex
/// color range straight off the live fill mesh. Shared by the two tests
/// below that both need the real, wired-app color, not a fixture.
fn county_zero_colors(app: &App) -> Vec<[f32; 4]> {
    let surface = app.world().resource::<babylon_client::map::MapSurface>();
    let meshes = app.world().resource::<Assets<Mesh>>();
    let mesh = meshes
        .get(&surface.fill_mesh)
        .expect("fill mesh is registered");
    let (start, end) = surface.tessellation.county_vertex_range[0];
    match mesh
        .attribute(Mesh::ATTRIBUTE_COLOR)
        .expect("fill mesh carries per-vertex color")
    {
        bevy::mesh::VertexAttributeValues::Float32x4(colors) => {
            colors[start as usize..end as usize].to_vec()
        }
        other => panic!("unexpected color attribute shape: {other:?}"),
    }
}

#[test]
fn five_space_presses_advance_five_distinct_ticks_and_fire_both_packs_events() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.update(); // Startup

    let mut hashes = HashSet::new();
    for _ in 0..5 {
        press_key_via_real_event(&mut app, KeyCode::Space);
        app.update();
        release_key(&mut app, KeyCode::Space);

        let session = app
            .world()
            .resource::<babylon_client::engine_link::EngineSession>();
        hashes.insert(session.inner.graph().state_hash().expect("hash"));
    }
    assert_eq!(hashes.len(), 5, "five presses, five distinct hashes");

    // B3 wave-1 Task 4 (plan §2.2, the #503 fix): `advance_ticks` now
    // drains `session.sink.events` into `BeatLog` every tick, so the raw
    // sink is empty again immediately after each press — the drained,
    // bounded `BeatLog` is the event feed's own canonical history now.
    let log = app.world().resource::<babylon_client::ui::beats::BeatLog>();
    assert!(
        log.beats
            .iter()
            .any(|beat| beat.event_type == "LIFECYCLE_TRANSITION"),
        "the beat log must carry lifecycle's own emitted events"
    );
    assert!(
        log.beats
            .iter()
            .any(|beat| beat.event_type == "ENTITY_DEATH"),
        "the beat log must carry vitality's own emitted events too — \
         proving both packs actually ran, not just lifecycle"
    );
}

#[test]
fn defaults_to_population_trend_and_tab_cycles_through_all_three() {
    // B3 wave-1 Task 8 (§2.10): `ActiveLens` is an index into
    // `babylon_client::map::LENSES`, not a closed enum — index-based
    // cycling replaces enum cycling, the ONE substance-preserving diff
    // this task's own gate calls for. LENSES order: [0] Tension,
    // [1] Legitimation, [2] Population Trend.
    use babylon_client::map::ActiveLens;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update();

    // Task 8's finding: Tension has zero data on this demo content, so the
    // app must not default to it.
    assert_eq!(
        *app.world().resource::<babylon_client::map::ActiveLens>(),
        ActiveLens(2) // Population Trend
    );

    let mut seen = vec![*app.world().resource::<babylon_client::map::ActiveLens>()];
    for _ in 0..3 {
        press_key_via_real_event(&mut app, KeyCode::Tab);
        app.update();
        release_key(&mut app, KeyCode::Tab);
        seen.push(*app.world().resource::<babylon_client::map::ActiveLens>());
    }
    assert_eq!(
        seen,
        vec![ActiveLens(2), ActiveLens(0), ActiveLens(1), ActiveLens(2)],
        "three presses from the default must visit every lens once and return to start"
    );
}

#[test]
fn a_known_demo_county_actually_recolors_after_a_space_press() {
    // THE test the MEDIUM-HIGH finding asked for: real `TickLoopPlugin` +
    // `MapPlugin` together, no hand-installed `CurrentLensData` (contrast
    // Task 10 Step 4's own test, which deliberately hand-builds a fixture
    // to test `recolor_on_lens_changed`'s LOGIC in isolation — this test
    // proves the real app's WIRING reaches the mesh at all, which a
    // hand-installed resource cannot prove by construction). Before this
    // test existed, every automated check in this plan passed even in a
    // build where `CurrentLensData`/lens-recolor wiring never resolved and
    // the map never recolored — this closes that gap.
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.update(); // Startup — real EngineSession, real CurrentLensData, real MapSurface.

    // Tick 0: PopulationTrend is the default lens, and every county reads
    // `Some(0.0)` (now == baseline, nothing has ticked yet) — DIM.
    let before = county_zero_colors(&app);

    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();

    // Atlas index 0 is a `core` (x0.95) family county — Task 9b's own
    // table has this family net-DECLINING, so after tick 1 it must read
    // CRIMSON, genuinely different from tick 0's DIM.
    let after = county_zero_colors(&app);

    assert_ne!(
        before, after,
        "the demo county at atlas index 0 must actually recolor after one Space press — \
         if this fails, CurrentLensData is not reaching the mesh even though the tick itself \
         advanced (check that advance_ticks's ResMut<CurrentLensData> param and its \
         build_lens_data delegation are wired, and that recolor_on_lens_changed's \
         Res<MapSurface> resolves)"
    );
}

/// FB1 fix proof (adversarial-panel finding, execution-proven): before this
/// fix, `recolor_on_lens_changed` only ever WROTE the cells its incoming
/// `LensReading` could resolve and `continue`d past everything else,
/// meaning a county absent from a lens's own cells kept whatever color a
/// PREVIOUSLY active lens had painted there. `county_tension` resolves ZERO
/// of the twelve demo counties (Task 8's own finding — no rule pack writes
/// the `v`/`s`/`e` fields it needs), so switching from `PopulationTrend`
/// (which paints a real color after a tick) to Tension left the map
/// showing STALE `PopulationTrend` color under the Tension lens — a
/// fabricated reading for a lens honestly reporting no data at all,
/// verified in-app (county 0 stayed CRIMSON across three Update frames
/// after Tab -> Tension while the HUD correctly said "no data this tick").
/// Real `MapPlugin` + `TickLoopPlugin` together, zero hand-installed
/// resources — the same wiring-proof shape as this file's other tests.
// These `[f32; 4]` arrays are exact byte-for-byte copies read straight off
// the live fill mesh's vertex buffer (no floating computation between the
// recolor system's write and this read) — exact comparison is the correct
// check, same justification as `map/bands.rs`'s recolor tests.
#[allow(clippy::float_cmp)]
#[test]
fn switching_from_a_painted_lens_to_an_empty_one_clears_stale_color_to_panel() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.update(); // Startup — PopulationTrend default, tick 0, DIM everywhere.

    // Space: county 0 (fips 01001, a "core" x0.95 county) nets DECLINING at
    // tick 1 (verified: population_baseline 9500.0, tick-1 total 9494.81,
    // delta -5.19) -> CRIMSON under PopulationTrend, unambiguously NOT
    // PANEL and NOT the tick-0 DIM.
    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);

    let after_space = county_zero_colors(&app);
    let expected_crimson = babylon_client::palette::CRIMSON.to_linear().to_f32_array();
    assert!(
        after_space.iter().all(|c| *c == expected_crimson),
        "county 0 must read CRIMSON under PopulationTrend after tick 1 — got {after_space:?}"
    );

    // Tab -> Tension: Tension resolves ZERO cells on this demo content
    // (no rule pack writes the tension fields), so its own LensReading is
    // whole-lens-absent. County 0 must go back to PANEL — never keep
    // showing the CRIMSON the PREVIOUS lens (PopulationTrend) just painted.
    press_key_via_real_event(&mut app, KeyCode::Tab);
    app.update();
    release_key(&mut app, KeyCode::Tab);
    // Pump extra frames — the original bug did NOT self-heal across
    // repeated Update passes (the finding's own "three update frames"
    // observation), so this rules out a lag, not just a same-frame race.
    app.update();
    app.update();

    let after_tab = county_zero_colors(&app);
    let expected_panel = babylon_client::map::PANEL.to_linear().to_f32_array();
    assert!(
        after_tab.iter().all(|c| *c == expected_panel),
        "county 0 must clear to PANEL under the Tension lens (zero resolved cells), \
         not keep PopulationTrend's stale CRIMSON — got {after_tab:?}, want {expected_panel:?} \
         everywhere (FB1 regression: recolor_on_lens_changed must pre-clear every county \
         before painting the incoming lens's own resolved cells)"
    );
}
