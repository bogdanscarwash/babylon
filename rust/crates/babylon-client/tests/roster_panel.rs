//! Task 7.5's own headless real-`App` proof (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.11,
//! task-7-brief.md): `\u{2191}`/`\u{2193}` select through carceral's own
//! derived roster and the real `StatePanelText` entity renders the
//! selected node's own published fields, live off the graph — §2.8's own
//! "read the actual rendered `Text` component" discipline, never a
//! hand-built fixture. Also proves counties (a `MapBinding::Fips` story)
//! is unaffected: the arrow keys are a no-op there, and the panel keeps
//! rendering its own `SelectedCounty` path.

use babylon_client::story;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;

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

fn new_app(selected_story: &'static story::Story) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(story::SelectedStory(selected_story));
    app.update(); // Startup.
    app
}

fn state_panel_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<babylon_client::loop_ui::StatePanelText>>();
    query.single(world).map(|t| t.0.clone()).unwrap_or_default()
}

#[test]
fn arrow_down_selects_the_first_carceral_roster_node_and_renders_its_live_fields() {
    let mut app = new_app(story::carceral());

    assert_eq!(
        state_panel_text(&mut app),
        "",
        "nothing selected yet must render the honest empty panel"
    );

    press_key_via_real_event(&mut app, KeyCode::ArrowDown);
    app.update();
    release_key(&mut app, KeyCode::ArrowDown);

    let label = {
        let session = app
            .world()
            .resource::<babylon_client::engine_link::EngineSession>();
        session.full_roster[0].0.clone()
    };

    let text = state_panel_text(&mut app);
    assert!(text.starts_with(&format!("{label} (1/6)")), "got {text:?}");
    // la-approaching (index 0 by NodeId — the scenario's own declaration
    // order) is a SOCIAL_CLASS node: population 600, wealth 515, per
    // carceral-arc-conformance.bscn's own seeded values.
    assert!(
        text.contains("social-class/population: 600.00"),
        "got {text:?}"
    );
    assert!(text.contains("social-class/wealth: 515.00"), "got {text:?}");
}

#[test]
fn arrow_down_seven_times_from_the_start_wraps_back_to_the_first_node() {
    let mut app = new_app(story::carceral());
    // Six presses visit all six distinct roster entries (index 0..=5); the
    // SEVENTH is the first one that wraps back to index 0.
    for _ in 0..7 {
        press_key_via_real_event(&mut app, KeyCode::ArrowDown);
        app.update();
        release_key(&mut app, KeyCode::ArrowDown);
    }
    let text = state_panel_text(&mut app);
    assert!(
        text.contains("(1/6)"),
        "seven presses over a six-entry roster must wrap back to the first, got {text:?}"
    );
}

#[test]
fn arrow_up_from_the_start_also_lands_on_the_first_node_never_a_negative_index() {
    let mut app = new_app(story::carceral());
    press_key_via_real_event(&mut app, KeyCode::ArrowUp);
    app.update();
    release_key(&mut app, KeyCode::ArrowUp);
    let text = state_panel_text(&mut app);
    assert!(text.contains("(1/6)"), "got {text:?}");
}

/// Presses `\u{2193}` six times (index 0..=5, one entry per press) to land on
/// `carceral-register`, the carrier — it sorts LAST by `NodeId` among the
/// 6-entry roster (5 `SOCIAL_CLASS` nodes declared first in the scenario,
/// the carrier declared last, per `carceral_arc_conformance.rs`'s own
/// `CARCERAL_REGISTER = NodeId(5)`).
fn select_the_institution_carrier(app: &mut App) {
    for _ in 0..6 {
        press_key_via_real_event(app, KeyCode::ArrowDown);
        app.update();
        release_key(app, KeyCode::ArrowDown);
    }
}

#[test]
fn the_institution_carrier_is_selectable_and_renders_its_own_published_fields() {
    let mut app = new_app(story::carceral());
    select_the_institution_carrier(&mut app);
    let text = state_panel_text(&mut app);
    assert!(text.starts_with("carceral-register (6/6)"), "got {text:?}");
    assert!(
        text.contains("institution/enforcer-population: 0.00"),
        "every census field is seeded 0 at tick 0, got {text:?}"
    );
    // The seeded-0 trap (§2.4): decomposition-fire-tick/control-crisis-tick
    // are ALSO seeded literal 0 at tick 0, but neither beat has fired yet —
    // rendering that seeded 0 as a numeral would fabricate "fired at tick
    // 0". Both must render the honest not-yet-latched reason instead, the
    // SAME gating `ui::countdown::resolve` already applies to these exact
    // fields.
    for field in [
        "institution/decomposition-fire-tick",
        "institution/control-crisis-tick",
    ] {
        assert!(
            text.contains(&format!("{field}: not computed by this port")),
            "{field} must render the honest not-yet-latched reason before its own beat fires, \
             got {text:?}"
        );
        assert!(
            !text.contains(&format!("{field}: 0.00")),
            "{field} must never render its seeded 0 as a fabricated fired-at-tick-0 claim, \
             got {text:?}"
        );
    }
}

#[test]
fn decomposition_fire_tick_renders_the_real_value_once_class_decomposition_fires_at_53() {
    let mut app = new_app(story::carceral());
    select_the_institution_carrier(&mut app);

    // 53 real ticks via Space — CLASS_DECOMPOSITION fires exactly at tick
    // 53 (the derived schedule, `carceral_arc_conformance.rs`'s own
    // module doc), writing decomposition-fire-tick/-fired-known together
    // in the SAME effects block.
    for _ in 0..53 {
        press_key_via_real_event(&mut app, KeyCode::Space);
        app.update();
        release_key(&mut app, KeyCode::Space);
    }

    let text = state_panel_text(&mut app);
    assert!(
        text.contains("institution/decomposition-fire-tick: 53.00"),
        "once the latch flips at tick 53 the real material value must render, got {text:?}"
    );
    // control-crisis-tick's own beat (CONTROL_RATIO_CRISIS) does not fire
    // until tick 105 — still honestly not-yet-latched here.
    assert!(
        text.contains("institution/control-crisis-tick: not computed by this port"),
        "got {text:?}"
    );
}

#[test]
fn counties_the_map_bound_story_ignores_the_arrow_keys() {
    let mut app = new_app(story::counties());
    press_key_via_real_event(&mut app, KeyCode::ArrowDown);
    app.update();
    release_key(&mut app, KeyCode::ArrowDown);
    assert_eq!(
        app.world()
            .resource::<babylon_client::ui::roster_panel::SelectedRosterIndex>()
            .0,
        None,
        "a MapBinding::Fips story must never move SelectedRosterIndex"
    );
    assert_eq!(
        state_panel_text(&mut app),
        "",
        "counties' own state panel must stay the SelectedCounty-driven (empty until a click) \
         path, not the roster panel"
    );
}
