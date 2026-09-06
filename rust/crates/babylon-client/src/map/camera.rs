//! The county map's bounded pan/zoom camera (B1 Task 7).
//!
//! `PanCamera` (`bevy::camera_controller::pan_camera`, the `pan_camera`
//! feature) drives panning (WASD) and zoom (mouse wheel / +/-) by writing
//! its `zoom_factor` straight into the camera entity's own `Transform`
//! scale (`run_pancamera_controller` in the crate: `transform.scale =
//! Vec3::splat(controller.zoom_factor)`). Under `ScalingMode::WindowSize`
//! with `OrthographicProjection::scale` left at its default `1.0` (this
//! module never touches it), that transform scale IS "world metres visible
//! per screen pixel" — so `zoom_factor` and this module's `clamp_camera`'s
//! `zoom` parameter are the same number.
//!
//! **A naming collision worth flagging explicitly** (the plan's Task 7
//! Step 1 prose vs. `PanCamera`'s own fields): the plan describes
//! `MAX_ZOOM` as the CLOSEST-IN bound ("one median county fills a third of
//! the viewport"). But `PanCamera.zoom_factor` runs the opposite way from
//! that prose: a SMALLER `zoom_factor` means fewer world metres map into
//! the same pixel count, i.e. MORE zoomed in; a LARGER `zoom_factor` means
//! more zoomed out. (Confirmed by reading `run_pancamera_controller`:
//! `zoom_factor -= zoom_amount`, and `zoom_amount` is positive for a
//! forward/up scroll — the intuitive "scroll up to zoom in" gesture
//! decreases `zoom_factor`.) So the plan's prose `MAX_ZOOM` (closest-in)
//! is `PanCamera.min_zoom` here, and its `MIN_ZOOM` (whole map visible,
//! "fully zoomed out") is `PanCamera.max_zoom`. This module names its own
//! constants and functions by their EFFECT rather than reusing the
//! ambiguous MIN/MAX prose, and every doc comment says which `PanCamera`
//! field it feeds.
//!
//! **Fix round (adversarial verification of PR #490, F1/F10).** The first
//! cut left `pan_speed`/`zoom_speed` at `PanCamera::default()`'s values
//! (500.0, 0.1) — WORLD units, and this world is Albers metres (atlas
//! `world_bounds` roughly 4.6M x 4.3M m). Measured against the committed
//! atlas at the opening (whole-map) zoom on a 1280x720 viewport: panning
//! moved about 0.084 px/s (hours to cross one screen), and reaching the
//! zoomed-in limit took tens of thousands of scroll notches — the camera
//! was, in practice, immobile. `pan_speed_for_zoom` and
//! `zoom_speed_for_range` derive both from the atlas's own zoom bounds
//! instead (Task 7 Step 1's "compute it, don't guess" instruction, applied
//! to the two fields the first cut missed), and `clamp_camera_system`
//! re-derives `pan_speed` from the LIVE `zoom_factor` every frame so
//! panning keeps a constant on-screen speed at every zoom level, not just
//! at the opening one. Separately (**F10**, an upstream naming bug this
//! module inherits rather than causes): `run_pancamera_controller` reads
//! `key_zoom_in` and DECREASES `zoom_factor` when it's pressed — but
//! `PanCamera::default()` binds `key_zoom_in` to `+`/`Equal` while ALSO
//! having a forward scroll (the intuitive "zoom in" gesture) decrease
//! `zoom_factor`. Those two defaults contradict each other: with the
//! crate's own defaults, pressing `+` INCREASES `zoom_factor`, i.e. `+`
//! zooms OUT. `spawn_camera` swaps the two keys explicitly below so `+`
//! zooms in and `-` zooms out, matching the scroll wheel's own (already
//! intuitive) direction.

use crate::atlas::CountyAtlas;
use bevy::camera::ScalingMode;
use bevy::camera_controller::pan_camera::PanCamera;
use bevy::input::keyboard::KeyCode;
use bevy::math::{Rect, Vec2};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/map/county_atlas.bin"
));

/// Fallback viewport used only when no primary window exists — the
/// headless test path (`MinimalPlugins` carries no `WindowPlugin`).
/// Matches the size the `mesh2d_vertex_color_texture`-style examples and
/// this crate's own `WindowPlugin` default settle on in practice.
const FALLBACK_VIEWPORT: Vec2 = Vec2::new(1280.0, 720.0);

/// The fraction of the smaller viewport dimension a median county's
/// bounding-box diagonal should fill at the closest permitted zoom.
const MEDIAN_COUNTY_VIEWPORT_FRACTION: f32 = 1.0 / 3.0;

/// Target on-screen pan speed, in pixels per second, at any zoom level
/// (F1). `pan_speed_for_zoom` converts this into the world-metres-per-
/// second `PanCamera.pan_speed` actually wants, scaled by the current
/// `zoom_factor` so the ON-SCREEN speed stays constant as the player
/// zooms — 600 px/s crosses a 1280px-wide viewport in about 2 seconds,
/// which is brisk without being uncontrollable.
const TARGET_PAN_PIXELS_PER_SEC: f32 = 600.0;

/// How many scroll notches should traverse the full `[min_zoom, max_zoom]`
/// range (F1) — the verifier's suggested 30-40 notch band, taken at its
/// midpoint.
const TARGET_ZOOM_NOTCHES: f32 = 35.0;

/// The atlas's `world_bounds()` and median county bbox diagonal, stashed
/// at Startup so neither the per-frame clamp system nor the window-resize
/// recompute system (F5) needs to re-parse the embedded atlas.
#[derive(Resource, Clone, Copy)]
pub struct MapBounds {
    pub world_bounds: Rect,
    /// Static atlas geometry — computed once, reused by
    /// `resize_camera_bounds_system` whenever the viewport changes rather
    /// than only at Startup.
    pub median_county_diagonal: f32,
}

/// The atlas's median county bounding-box diagonal, in world metres —
/// static geometry, independent of any viewport.
#[must_use]
pub fn median_county_diagonal(atlas: &CountyAtlas) -> f32 {
    let mut diagonals: Vec<f32> = (0..atlas.len())
        .map(|i| {
            let county = atlas.county(i).expect("index is within 0..atlas.len()");
            (county.bbox.max - county.bbox.min).length()
        })
        .collect();
    diagonals.sort_by(|a, b| a.partial_cmp(b).expect("bbox diagonals are finite"));
    diagonals[diagonals.len() / 2]
}

/// Feeds `PanCamera.min_zoom` (the SMALLEST `zoom_factor`, i.e. the
/// CLOSEST-in bound the camera may reach): the zoom at which a bbox
/// diagonal of `median_diagonal` fills `MEDIAN_COUNTY_VIEWPORT_FRACTION`
/// of the viewport's smaller dimension.
#[must_use]
pub fn closest_in_zoom_from_diagonal(median_diagonal: f32, viewport: Vec2) -> f32 {
    let target_pixels = viewport.x.min(viewport.y) * MEDIAN_COUNTY_VIEWPORT_FRACTION;
    median_diagonal / target_pixels
}

/// `closest_in_zoom_from_diagonal`, computing the median diagonal from
/// `atlas` first. Computed from the atlas rather than a guessed magic
/// number (Task 7 Step 1's own instruction).
#[must_use]
pub fn closest_in_zoom(atlas: &CountyAtlas, viewport: Vec2) -> f32 {
    closest_in_zoom_from_diagonal(median_county_diagonal(atlas), viewport)
}

/// Feeds `PanCamera.max_zoom` (the LARGEST `zoom_factor`, i.e. the
/// FARTHEST-out bound the camera may reach): the zoom at which `bounds`
/// exactly fits inside `viewport`.
#[must_use]
pub fn whole_map_zoom(bounds: Rect, viewport: Vec2) -> f32 {
    (bounds.width() / viewport.x).max(bounds.height() / viewport.y)
}

/// Feeds `PanCamera.pan_speed` (world metres/second): `zoom_factor` IS
/// world-metres-per-pixel at this wiring (see the module doc), so
/// multiplying it by a target screen-pixels/second gives a world speed
/// that traces that same screen speed regardless of zoom level (F1).
#[must_use]
pub fn pan_speed_for_zoom(zoom_factor: f32) -> f32 {
    TARGET_PAN_PIXELS_PER_SEC * zoom_factor
}

/// Feeds `PanCamera.zoom_speed` (world metres/pixel added or removed from
/// `zoom_factor` per scroll notch — the crate's zoom stepping is linear/
/// additive, not exponential, so a step sized for a huge range feels
/// twitchy near the zoomed-in end; that is an inherited property of the
/// crate's own zoom model, not something this function can fix). Sized so
/// `TARGET_ZOOM_NOTCHES` notches cross the full `[min_zoom, max_zoom]`
/// range (F1).
#[must_use]
pub fn zoom_speed_for_range(min_zoom: f32, max_zoom: f32) -> f32 {
    (max_zoom - min_zoom) / TARGET_ZOOM_NOTCHES
}

/// Clamps a camera's world-space translation so its visible rect never
/// leaves `bounds` grown by 10% on every side — except when the visible
/// rect is itself at least as large as the grown bounds (fully zoomed
/// out), in which case there is nothing left to clamp against and the
/// camera centres on `bounds.center()`. Pure and renderer-free so it can
/// be unit tested directly.
#[must_use]
pub fn clamp_camera(translation: Vec2, zoom: f32, viewport: Vec2, bounds: Rect) -> Vec2 {
    let half_extent = viewport * zoom * 0.5;
    let grown = grow(bounds, 0.10);

    let x = if half_extent.x * 2.0 >= grown.width() {
        bounds.center().x
    } else {
        translation
            .x
            .clamp(grown.min.x + half_extent.x, grown.max.x - half_extent.x)
    };
    let y = if half_extent.y * 2.0 >= grown.height() {
        bounds.center().y
    } else {
        translation
            .y
            .clamp(grown.min.y + half_extent.y, grown.max.y - half_extent.y)
    };
    Vec2::new(x, y)
}

/// `rect` grown by `fraction` of its own size, symmetric around its
/// center — a rect grown by 0.10 is 10% larger on each axis, 5% added to
/// each side.
fn grow(rect: Rect, fraction: f32) -> Rect {
    let margin = rect.size() * fraction * 0.5;
    Rect {
        min: rect.min - margin,
        max: rect.max + margin,
    }
}

#[derive(Component)]
pub struct MapCamera;

fn primary_viewport(windows: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(FALLBACK_VIEWPORT)
}

/// `Startup` system: spawns the map camera, sized and bounded from the
/// atlas. Re-parses the embedded atlas independently of
/// `map::mesh::spawn_map_surface`'s own parse rather than reaching into
/// its `MapSurface` resource — that keeps this system free of any
/// same-schedule command-flush-ordering assumption between two Startup
/// systems, at the cost of one extra (cheap, parse-only, no
/// tessellation) atlas decode at boot.
pub(super) fn spawn_camera(mut commands: Commands, windows: Query<&Window, With<PrimaryWindow>>) {
    let viewport = primary_viewport(&windows);
    let atlas = CountyAtlas::parse(ATLAS_BYTES)
        .unwrap_or_else(|e| panic!("county atlas failed to parse at startup: {e}"));

    let world_bounds = atlas.world_bounds();
    let median_diagonal = median_county_diagonal(&atlas);
    let min_zoom = closest_in_zoom_from_diagonal(median_diagonal, viewport);
    let max_zoom = whole_map_zoom(world_bounds, viewport);
    let center = world_bounds.center();

    commands.spawn((
        Camera2d,
        MapCamera,
        Transform::from_xyz(center.x, center.y, 0.0),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            ..OrthographicProjection::default_2d()
        }),
        PanCamera {
            // Start fully zoomed out (the whole map fits) rather than at
            // the crate's own `zoom_factor: 1.0` default, which would
            // start the player looking at an arbitrary few-pixel sliver
            // of one county.
            zoom_factor: max_zoom,
            min_zoom,
            max_zoom,
            // F1: world-metres/second and world-metres/notch, not the
            // crate's own (screen-scale-irrelevant) defaults — see the
            // module doc's "Fix round" section.
            pan_speed: pan_speed_for_zoom(max_zoom),
            zoom_speed: zoom_speed_for_range(min_zoom, max_zoom),
            // F10: swapped from the crate's own defaults so `+` zooms IN
            // and `-` zooms OUT — see the module doc's "F10" paragraph.
            key_zoom_in: Some(KeyCode::Minus),
            key_zoom_out: Some(KeyCode::Equal),
            // Rotation off: a rotated map disorients the player, and no
            // ruling asks for one.
            rotation_speed: 0.0,
            key_rotate_ccw: None,
            key_rotate_cw: None,
            ..default()
        },
    ));

    commands.insert_resource(MapBounds {
        world_bounds,
        median_county_diagonal: median_diagonal,
    });
}

/// `Update` system: applies `clamp_camera` after `PanCameraPlugin`'s own
/// `RunFixedMainLoop`-scheduled system has moved the camera this frame —
/// `RunFixedMainLoop` runs before `Update` in Bevy's default schedule
/// order, so a plain `Update` system already satisfies "after". Also
/// re-derives `pan_speed` from the camera's OWN CURRENT `zoom_factor`
/// every frame (F1) — a value set once at Startup would only be correct
/// at the zoom level the game opened at; scaling it live keeps on-screen
/// pan speed roughly constant as the player zooms in and out.
pub(super) fn clamp_camera_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    bounds: Option<Res<MapBounds>>,
    observer_viewport: Option<Res<crate::observer_ui::ObserverViewport>>,
    mut cameras: Query<(&mut Transform, &mut PanCamera), With<MapCamera>>,
) {
    let Some(bounds) = bounds else {
        return;
    };
    let viewport = observer_viewport
        .and_then(|viewport| viewport.0)
        .map_or_else(|| primary_viewport(&windows), |rect| rect.size());
    for (mut transform, mut pan_camera) in &mut cameras {
        let clamped = clamp_camera(
            transform.translation.truncate(),
            pan_camera.zoom_factor,
            viewport,
            bounds.world_bounds,
        );
        transform.translation.x = clamped.x;
        transform.translation.y = clamped.y;

        pan_camera.pan_speed = pan_speed_for_zoom(pan_camera.zoom_factor);
    }
}

/// `Update` system (F5): recomputes `PanCamera.min_zoom`/`max_zoom`/
/// `zoom_speed` whenever the primary window resizes. Computed only once
/// at Startup, these bounds silently drift from their own stated
/// invariants on any resize: shrinking the window stops `max_zoom` from
/// actually fitting the whole map (the player gets stranded unable to
/// reach the whole-map view), and growing it breaks `whole_map_zoom`'s
/// "fits exactly" invariant the other direction (now too small to
/// actually fit). `Changed<Window>` may fire more often than true
/// resizes (window-sync systems can write-touch other `Window` fields,
/// e.g. cursor position) — harmless here since every recompute is a
/// handful of float operations over data already held in `MapBounds`, no
/// allocation and no re-parse.
pub(super) fn resize_camera_bounds_system(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    bounds: Option<Res<MapBounds>>,
    mut cameras: Query<&mut PanCamera, With<MapCamera>>,
) {
    let Some(bounds) = bounds else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let min_zoom = closest_in_zoom_from_diagonal(bounds.median_county_diagonal, viewport);
    let max_zoom = whole_map_zoom(bounds.world_bounds, viewport);
    let zoom_speed = zoom_speed_for_range(min_zoom, max_zoom);
    for mut pan_camera in &mut cameras {
        pan_camera.min_zoom = min_zoom;
        pan_camera.max_zoom = max_zoom;
        pan_camera.zoom_speed = zoom_speed;
        pan_camera.zoom_factor = pan_camera.zoom_factor.clamp(min_zoom, max_zoom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> CountyAtlas {
        CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses")
    }

    #[test]
    fn fully_zoomed_out_centres_on_bounds_center() {
        let bounds = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let viewport = Vec2::new(2000.0, 2000.0); // huge viewport
        let zoom = 1.0; // half_extent = 1000x1000, exceeds the grown bounds
        let far_away = Vec2::new(50_000.0, -50_000.0);
        let result = clamp_camera(far_away, zoom, viewport, bounds);
        assert_eq!(result, bounds.center());
    }

    #[test]
    fn panning_past_the_east_edge_clamps_inside_the_grown_bounds() {
        let bounds = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let viewport = Vec2::new(100.0, 100.0);
        let zoom = 1.0; // half_extent = (50, 50), well inside the grown bounds
        let grown = grow(bounds, 0.10);

        let panned_past_east = Vec2::new(1_000_000.0, bounds.center().y);
        let result = clamp_camera(panned_past_east, zoom, viewport, bounds);

        let expected_x = grown.max.x - 50.0;
        assert!(
            (result.x - expected_x).abs() < 1e-4,
            "expected clamped x near {expected_x}, got {}",
            result.x
        );
        // y was not pushed out of range, so it should pass through
        // unchanged (still within the grown bounds' clamp interval).
        assert!((result.y - bounds.center().y).abs() < 1e-4);
    }

    #[test]
    fn a_visible_rect_within_bounds_is_left_alone() {
        let bounds = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let viewport = Vec2::new(100.0, 100.0);
        let zoom = 1.0; // half_extent = (50, 50)
        let inside = Vec2::new(500.0, 250.0);
        let result = clamp_camera(inside, zoom, viewport, bounds);
        assert!((result - inside).length() < 1e-4);
    }

    #[test]
    fn closest_in_zoom_makes_the_median_county_fill_a_third_of_the_viewport() {
        let atlas = atlas();
        let viewport = Vec2::new(1280.0, 720.0);
        let zoom = closest_in_zoom(&atlas, viewport);
        let median = median_county_diagonal(&atlas);

        let median_screen_extent = median / zoom;
        let target = viewport.x.min(viewport.y) * MEDIAN_COUNTY_VIEWPORT_FRACTION;
        assert!(
            (median_screen_extent - target).abs() < 1e-2,
            "median county should occupy {target} px, got {median_screen_extent} px"
        );
    }

    #[test]
    fn whole_map_zoom_fits_the_bounds_exactly_on_the_binding_axis() {
        let atlas = atlas();
        let bounds = atlas.world_bounds();
        let viewport = Vec2::new(1280.0, 720.0);
        let zoom = whole_map_zoom(bounds, viewport);

        // At this zoom, the visible extent must be at least as large as
        // bounds on both axes...
        assert!(viewport.x * zoom >= bounds.width() - 1e-2);
        assert!(viewport.y * zoom >= bounds.height() - 1e-2);
        // ...and exactly equal to it on whichever axis is the binding
        // constraint.
        let x_binds = (viewport.x * zoom - bounds.width()).abs() < 1e-2;
        let y_binds = (viewport.y * zoom - bounds.height()).abs() < 1e-2;
        assert!(x_binds || y_binds, "neither axis is the binding constraint");
    }

    #[test]
    fn closest_in_zoom_is_smaller_than_whole_map_zoom() {
        let atlas = atlas();
        let viewport = Vec2::new(1280.0, 720.0);
        let min_zoom = closest_in_zoom(&atlas, viewport);
        let max_zoom = whole_map_zoom(atlas.world_bounds(), viewport);
        assert!(
            min_zoom < max_zoom,
            "the closest-in zoom {min_zoom} must be smaller than the whole-map zoom {max_zoom} \
             (PanCamera.min_zoom must be < PanCamera.max_zoom or the clamp is inverted)"
        );
    }

    /// F1 regression: at the REAL atlas's opening (whole-map) zoom, the
    /// old fixed `pan_speed: 500.0` crossed one screen width in hours (the
    /// verifier measured 0.084 px/s on a 1280x720 viewport). Pin an order-
    /// of-magnitude sanity band instead of an exact number, since the
    /// exact seconds depend on `TARGET_PAN_PIXELS_PER_SEC`'s tuning.
    #[test]
    fn pan_speed_crosses_the_whole_map_in_single_digit_seconds_at_the_opening_zoom() {
        let atlas = atlas();
        let viewport = Vec2::new(1280.0, 720.0);
        let bounds = atlas.world_bounds();
        let opening_zoom = whole_map_zoom(bounds, viewport);
        let pan_speed = pan_speed_for_zoom(opening_zoom);

        let seconds_to_cross_width = bounds.width() / pan_speed;
        assert!(
            (0.1..30.0).contains(&seconds_to_cross_width),
            "crossing the map width should take single-digit-to-low-tens \
             seconds at the opening zoom, took {seconds_to_cross_width}s \
             (pan_speed {pan_speed} m/s) — regression guard for F1"
        );
    }

    /// F1 regression: the old fixed `zoom_speed: 0.1` needed roughly
    /// 56,909 scroll notches to cross the real zoom range (the verifier's
    /// measurement). `zoom_speed_for_range` should land within the
    /// suggested 30-40 notch band on the real atlas's own bounds.
    #[test]
    fn zoom_speed_traverses_the_real_range_in_30_to_40_notches() {
        let atlas = atlas();
        let viewport = Vec2::new(1280.0, 720.0);
        let min_zoom = closest_in_zoom(&atlas, viewport);
        let max_zoom = whole_map_zoom(atlas.world_bounds(), viewport);
        let zoom_speed = zoom_speed_for_range(min_zoom, max_zoom);

        let notches = (max_zoom - min_zoom) / zoom_speed;
        assert!(
            (30.0..=40.0).contains(&notches),
            "expected 30-40 notches to cross the range, got {notches}"
        );
    }
}
