//! End-to-end determinism guard through the client's OWN composed seam —
//! `EngineSession::start` + repeated `advance()` — the actual path a
//! player's key presses drive. `babylon-tick`'s own `session.rs` test
//! (`two_independent_sessions_over_the_same_content_hash_identically`)
//! proves the same property at the `TickSession` level; this test proves
//! it again through the client's own composed seam.
use babylon_client::engine_link::EngineSession;
use babylon_client::story;
use bevy::asset::AssetPlugin;
use bevy::image::ImagePlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::texture::TexturePlugin;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

fn press_space_via_production_input(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: KeyCode::Space,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
}

#[test]
fn same_content_same_tick_count_yields_the_same_hash() {
    let mut a = EngineSession::start(story::counties()).expect("session a");
    let mut b = EngineSession::start(story::counties()).expect("session b");
    for tick in 1..=5 {
        let ra = a.advance().expect("a advances");
        let rb = b.advance().expect("b advances");
        assert_eq!(
            ra.world_after, rb.world_after,
            "tick {tick}: two independent EngineSessions over the same content must have identical world hashes"
        );
        assert_eq!(
            ra.per_rule_fired, rb.per_rule_fired,
            "tick {tick}: per-rule detail must also match — the order proof, not just the hash"
        );
    }
}

#[test]
fn five_ticks_produce_five_distinct_hashes() {
    // Regression guard against a driver that silently re-runs tick 1 —
    // exactly the bug TickSession's own tick-numbering exists to prevent;
    // this test watches for it at the client's seam too.
    let mut session = EngineSession::start(story::counties()).expect("session");
    let mut hashes = std::collections::HashSet::new();
    for _ in 0..5 {
        let report = session.advance().expect("advance");
        hashes.insert(report.world_after);
    }
    assert_eq!(
        hashes.len(),
        5,
        "each completed tick must produce a distinct nominal world hash"
    );
}

#[test]
fn visual_presentation_leaves_the_app_owned_engine_tick_report_unchanged() {
    let mut visual_app = App::new();
    visual_app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::text::TextPlugin,
        TexturePlugin,
    ));
    visual_app.add_plugins((
        babylon_client::visual_assets::VisualAssetsPlugin,
        babylon_client::visual_assets::VisualPresentationPlugin,
    ));
    visual_app.add_plugins(babylon_client::map::MapPlugin);
    visual_app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    visual_app.insert_resource(story::SelectedStory(story::counties()));
    visual_app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    visual_app.update(); // Startup inserts the production EngineSession resource.

    press_space_via_production_input(&mut visual_app);
    visual_app.update(); // The shipped TickLoopPlugin system advances the app-owned session.

    let visual_report = visual_app
        .world()
        .resource::<babylon_client::ui::admin::LastTickReport>()
        .0
        .as_ref()
        .expect("production tick system records its report");
    assert_eq!(
        visual_app
            .world()
            .resource::<babylon_client::loop_ui::TickCounter>()
            .0,
        1,
        "the visual app must advance exactly one production tick"
    );

    let mut control = EngineSession::start(story::counties()).expect("control session starts");
    let control_report = control.advance().expect("control tick advances");

    assert_eq!(visual_report.after, control_report.after);
    assert_eq!(visual_report.per_rule_fired, control_report.per_rule_fired);
}
