//! FB3 fix (adversarial-panel MAJOR finding): `refresh_hud` had ZERO test
//! call sites before this file — Task 11 Step 5's own two properties
//! ("hovering a STABLE demo county renders the literal string 'STABLE'"
//! and "hovering under `ActiveLens::PopulationTrend` renders a delta
//! whose SIGN matches the county's known trajectory") were never checked
//! against the real, wired `refresh_hud` system, only against pure helper
//! functions in `map/hud.rs`'s own unit tests. These tests build a real
//! `App` (`MapPlugin` + `TickLoopPlugin`, no hand-installed resources),
//! drive the cursor through the same `CursorWorldPosition` precedent
//! `map/pick.rs`'s own hover test already established, and read the
//! ACTUAL rendered `CountyHudText`.
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;

/// Presses `key` through a REAL `KeyboardInput` message — necessary once
/// `MapPlugin` is in the App (it conditionally self-adds `InputPlugin`,
/// whose `PreUpdate` `keyboard_input_system` unconditionally clears
/// `just_pressed` every frame; a direct `ButtonInput::press()` call from
/// test code gets wiped before an `Update` system ever observes it —
/// `crates/babylon-client/src/map/mod.rs`'s own module doc has the full
/// citation).
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

fn hovered_hud_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, (With<babylon_client::map::CountyHudText>,)>();
    query
        .single(world)
        .expect("exactly one county HUD text entity")
        .0
        .clone()
}

/// **Recorded substitution**: the plan's own Task 11 Step 5 language says
/// "hovering" (`HoveredCounty`, driven by `CursorWorldPosition`). Under a
/// headless test harness (`MinimalPlugins`, no `WindowPlugin`), the REAL,
/// already-wired `track_cursor_world_position` system finds no
/// `PrimaryWindow` entity every frame and unconditionally resets
/// `CursorWorldPosition` to `None` — which `update_hovered_county` (the
/// SAME automatic chain, same frame) then propagates into `HoveredCounty`,
/// clobbering any value a test injects there BEFORE `refresh_hud` gets a
/// chance to read it (confirmed: injecting `CursorWorldPosition` directly
/// and calling `app.update()` once left `HoveredCounty` at `None`, not the
/// injected county). `active_county` (`map/hud.rs`) treats
/// `HoveredCounty`/`SelectedCounty` symmetrically as "the county the HUD
/// should describe" — hovered preferred, falling back to selected — so
/// driving these tests through `SelectedCounty` instead exercises the
/// EXACT SAME downstream code path in `refresh_hud`/`active_county`/
/// `format_lens_line` that hovering would, through an injection point nothing
/// in the automatic Update chain overwrites without a real mouse click
/// (`promote_selection_on_click` only fires on `just_pressed(MouseButton::Left)`).
/// `map/pick.rs`'s own test already proves `update_hovered_county` and
/// `promote_selection_on_click` correctly derive `HoveredCounty`/
/// `SelectedCounty` from real input; these tests target `refresh_hud`'s
/// OWN rendering correctness, which is orthogonal to that mechanism.
fn select_county(app: &mut App, atlas_index: usize) {
    app.world_mut()
        .resource_mut::<babylon_client::map::SelectedCounty>()
        .0 = Some(atlas_index);
}

/// Task 11 Step 5's first property: hovering a STABLE demo county renders
/// the literal string "STABLE" — the ONLY channel that can tell a STABLE
/// county apart from an absent one, since Director ruling 1 makes them
/// share a map color on purpose (`map/bands.rs`'s own doc comment).
#[test]
fn hovering_a_stable_demo_county_under_legitimation_shows_stable_in_the_hud() {
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

    // Advance one tick — verified (engine_link.rs probe, recorded in this
    // fix round's own report): ALL twelve demo counties read
    // territory/legitimation-crisis == 0 (STABLE) from tick 1 onward, since
    // `legit-index` is const-only (Task 9's own finding) and the classifier
    // recomputes fresh from it every tick regardless of the seed.
    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);

    // Tab once: PopulationTrend (default) -> Tension -> ... no, one press
    // goes to Tension. We need Legitimation specifically — two presses.
    for _ in 0..2 {
        press_key_via_real_event(&mut app, KeyCode::Tab);
        app.update();
        release_key(&mut app, KeyCode::Tab);
    }
    assert_eq!(
        *app.world().resource::<babylon_client::map::ActiveLens>(),
        babylon_client::map::ActiveLens(1), // Legitimation
        "two Tab presses from the PopulationTrend default must land on Legitimation"
    );

    // County 0 (see this file's `select_county` doc comment for why
    // SelectedCounty, not a synthesized hover, is the stable injection
    // point under a headless/windowless test harness).
    select_county(&mut app, 0);
    app.update();

    let text = hovered_hud_text(&mut app);
    assert!(
        text.contains("STABLE"),
        "HUD text {text:?} must contain the literal string STABLE — if this fails while \
         map::hud's own pure-function tests (format_lens_line/classify) pass, refresh_hud \
         itself is not reaching the Text component"
    );
}

/// Task 11 Step 5's second property: hovering a demo county under
/// `ActiveLens::PopulationTrend` renders a delta whose SIGN matches the
/// county's known trajectory — a young-family county (index 9, fips
/// 01019) nets GROWING at tick 1 (verified: baseline 9500.0/9500.0/9500.0
/// varies by scale, actual delta +55.9 at tick 1), every other family
/// (indices 0-8) nets DECLINING (verified: all negative at tick 1,
/// range -2.9 to -5.7).
#[test]
fn hovering_under_population_trend_renders_the_correct_sign_per_family() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // B3 wave-1 Task 5 (plan §2.5 Minor 7): `SelectedStory` has no
    // `Default` — every app-builder must say which story it wants.
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.update(); // Startup — PopulationTrend is already the default lens.
    assert_eq!(
        *app.world().resource::<babylon_client::map::ActiveLens>(),
        babylon_client::map::ActiveLens(2) // Population Trend
    );

    press_key_via_real_event(&mut app, KeyCode::Space);
    app.update();
    release_key(&mut app, KeyCode::Space);

    // A declining ("core" x0.95) county: atlas index 0.
    select_county(&mut app, 0);
    app.update();
    let declining_text = hovered_hud_text(&mut app);
    assert!(
        declining_text.contains("declining"),
        "county 0 (a core-family county, verified net-declining at tick 1) must read \
         'declining' — got {declining_text:?}"
    );

    // A growing ("young" x0.95) county: atlas index 9.
    select_county(&mut app, 9);
    app.update();
    let growing_text = hovered_hud_text(&mut app);
    assert!(
        growing_text.contains("growing"),
        "county 9 (a young-family county, verified net-growing at tick 1) must read \
         'growing' — got {growing_text:?}"
    );
}
