use bevy::asset::AssetPlugin;
use bevy::camera_controller::pan_camera::PanCamera;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use std::time::{Duration, Instant};

/// F7 (adversarial verification of PR #490): `map/camera.rs`'s unit tests
/// call `clamp_camera`/`closest_in_zoom`/`whole_map_zoom` directly — none
/// of them exercise `spawn_camera`'s actual `PanCamera` field assignment
/// or `clamp_camera_system`'s `Update`-schedule wiring. This headless
/// integration test does: build the real app (`MinimalPlugins` +
/// `AssetPlugin`, never `DefaultPlugins` — CI has no display server or
/// GPU), let `MapPlugin` spawn the camera at Startup, then confirm both
/// the spawned component's own bounds AND that a teleported camera
/// actually gets pulled back by the live system.
#[test]
fn a_teleported_camera_snaps_back_inside_the_grown_bounds() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update(); // Startup: spawns the map + camera, inserts MapBounds.

    let bounds = app
        .world()
        .get_resource::<babylon_client::map::MapBounds>()
        .expect("MapBounds inserted at Startup")
        .world_bounds;

    // min_zoom < max_zoom AS ACTUALLY WIRED on the spawned component —
    // camera.rs's own test of the same property only checks the pure
    // functions, never that spawn_camera assigns them correctly.
    {
        let world = app.world_mut();
        let mut query = world.query::<&PanCamera>();
        let pan_camera = query.single(world).expect("exactly one camera");
        assert!(
            pan_camera.min_zoom < pan_camera.max_zoom,
            "min_zoom {} must be < max_zoom {} as assigned on the spawned component",
            pan_camera.min_zoom,
            pan_camera.max_zoom
        );
    }

    // Teleport the camera far outside the map, then let the Update
    // schedule's clamp_camera_system pull it back. Filtered by
    // `With<PanCamera>` — the fill and border mesh entities also carry
    // `Transform`, so an unfiltered query matches three entities.
    {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&mut Transform, With<PanCamera>>();
        let mut transform = query.single_mut(world).expect("exactly one camera");
        transform.translation.x = 1.0e9;
        transform.translation.y = 1.0e9;
    }
    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<&Transform, With<PanCamera>>();
    let transform = query.single(world).expect("exactly one camera");

    // Same 10%-grown-bounds formula clamp_camera itself uses (5% margin
    // per side).
    let margin = bounds.size() * 0.05;
    let grown_min = bounds.min - margin;
    let grown_max = bounds.max + margin;
    assert!(
        transform.translation.x >= grown_min.x && transform.translation.x <= grown_max.x,
        "camera x {} escaped the grown bounds [{}, {}]",
        transform.translation.x,
        grown_min.x,
        grown_max.x
    );
    assert!(
        transform.translation.y >= grown_min.y && transform.translation.y <= grown_max.y,
        "camera y {} escaped the grown bounds [{}, {}]",
        transform.translation.y,
        grown_min.y,
        grown_max.y
    );
}

/// F10: `spawn_camera` swaps `PanCamera`'s `key_zoom_in`/`key_zoom_out`
/// from the crate's own (backwards-feeling) defaults — proves the swap on
/// the actually-spawned component, not just as an unverified code comment.
#[test]
fn spawn_camera_swaps_the_backwards_zoom_keys() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&PanCamera>();
    let pan_camera = query.single(world).expect("exactly one camera");
    assert_eq!(
        pan_camera.key_zoom_in,
        Some(KeyCode::Minus),
        "key_zoom_in must be swapped to Minus so scrolling/pressing zoom-in \
         actually decreases zoom_factor (zooms in) — see camera.rs's F10 doc"
    );
    assert_eq!(
        pan_camera.key_zoom_out,
        Some(KeyCode::Equal),
        "key_zoom_out must be swapped to Equal — the mirror of key_zoom_in's swap"
    );
}

/// F1 regression, driven through the REAL compiled `PanCameraPlugin`
/// system (not `clamp_camera` called directly, and not a mock) — the
/// property the coordinator's fix-round eyes-on asked to confirm live.
///
/// This test presses `KeyW` (pan NORTH, +Y) directly on the
/// `ButtonInput<KeyCode>` resource rather than through OS/X11 key events.
/// **Why north, and why not through real X11 input, are two separate
/// findings from one debugging session, recorded here so neither gets
/// re-discovered the hard way:**
///
/// 1. At the real atlas's opening (whole-map) zoom on a 1280x720
///    viewport, the committed geometry's aspect ratio (4,625,368 x
///    4,310,235 m) makes X the axis with EXCESS visible extent —
///    `clamp_camera`'s `half_extent.x * 2 >= grown.width()` is true, so
///    every frame force-centers X (correctly: there is nothing left to
///    pan against east/west at this zoom). Y is the binding axis and
///    genuinely has ~10% slack to pan into. A first draft of this test
///    pressed `KeyD` (east) and asserted zero net movement was a bug;
///    it was `clamp_camera` correctly doing its job on the wrong axis to
///    probe. This is why `spawn_camera`'s eyes-on screenshots (PR body)
///    pan north, not east, from the opening view.
/// 2. Getting here also involved a real interactive session on this dev
///    box's X11 display: `xte`-synthesized mouse scroll reached the
///    window fine (a held scroll produced a large, immediately visible
///    zoom change — confirming `zoom_speed_for_range`'s fix live), but
///    `xte`-synthesized KEY events did not, even after confirming
///    `_NET_ACTIVE_WINDOW` pointed at the client window. That looked
///    exactly like this same "pressed `KeyD`, nothing moved" symptom before
///    finding #1 above — a reminder that an unmoving camera has (at
///    least) two possible causes, and this test isolates the wiring from
///    input delivery so the two cannot be confused again. No
///    `xdotool`/`Xlib` was available on this box to independently force
///    X input focus and rule out the wheel-follows-pointer-vs-keyboard-
///    needs-real-focus explanation with full confidence.
///
/// This test exercises the IDENTICAL production code path
/// (`run_pancamera_controller`, unmodified, from the real
/// `bevy_camera_controller` crate) end to end, with real wall-clock
/// `Time<Real>` deltas driving its `dt`-scaled movement — the only
/// difference from a live session is how the key press reaches
/// `ButtonInput`, not anything under test.
#[test]
fn wasd_panning_through_the_real_plugin_moves_the_camera_at_a_reasonable_speed() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update(); // Startup: spawns the map + camera, inserts MapBounds.

    let bounds = app
        .world()
        .get_resource::<babylon_client::map::MapBounds>()
        .expect("MapBounds inserted at Startup")
        .world_bounds;

    let start_y = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<&Transform, With<PanCamera>>();
        query
            .single(world)
            .expect("exactly one camera")
            .translation
            .y
    };

    // Press KeyW (pan up/north, +Y — the axis with real slack at the
    // opening zoom; see the doc comment above) directly.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);

    let held_for = Duration::from_millis(500);
    let started = Instant::now();
    while started.elapsed() < held_for {
        std::thread::sleep(Duration::from_millis(16));
        app.update();
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyW);

    let (end_y, zoom_factor) = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<(&Transform, &PanCamera), ()>();
        let (transform, pan_camera) = query.single(world).expect("exactly one camera");
        (transform.translation.y, pan_camera.zoom_factor)
    };

    assert!(
        end_y > start_y,
        "holding KeyW for {held_for:?} must move the camera north; start {start_y}, end {end_y} \
         (F1 regression: the pre-fix pan_speed moved about 0.084 px/s — 500ms would be \
         imperceptible, not the clear move this asserts)"
    );

    // The pannable slack on Y (the binding axis at the opening zoom) is
    // only clamp_camera's own 10% grow margin — holding for 500ms at the
    // FIXED pan speed should already reach that clamp boundary (the
    // pre-fix speed would have covered roughly 250 world metres in that
    // time, against a map ~4.3M metres tall — nowhere close). Assert the
    // camera is now AT the expected clamped boundary, using the same
    // formula clamp_camera itself uses.
    let half_extent_y = (720.0 * zoom_factor) * 0.5; // FALLBACK_VIEWPORT.y (no window in this headless test)
    let grown_max_y = bounds.max.y + bounds.size().y * 0.05;
    let expected_clamped_y = grown_max_y - half_extent_y;
    assert!(
        (end_y - expected_clamped_y).abs() < 1.0,
        "expected the camera to have reached the north clamp boundary {expected_clamped_y} \
         within 500ms at the fixed pan speed, got {end_y}"
    );
}
