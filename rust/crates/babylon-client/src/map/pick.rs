//! County hit-testing (Task 11, completing B1's never-built Task 10): a
//! uniform grid over `world_bounds()` narrows a query point to a small
//! candidate list by bounding-box overlap, then an even-odd ring-crossing
//! test against each candidate's own rings (holes inverting membership for
//! free, since the even-odd rule already handles that when every ring's
//! edges are folded into the same crossing count) resolves the real hit,
//! if any.
//!
//! **Sequencing note.** The plan assigns `mod pick; mod hud;`'s declaration
//! to Task 12's edit of `map/mod.rs` — but this task's OWN `cargo test`
//! step cannot compile a file the crate never declares as a module. `mod
//! pick;`/`mod hud;` (private, unqualified — no `pub use` yet) land here,
//! in Task 11's own commit, as the minimal fix that unblocks this task's
//! tests; Task 12 still does the REST of the wiring this file doesn't
//! touch: the `pub use` re-exports, the recolor/lens-picker systems, and
//! registering these systems on `MapPlugin` itself.

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::palette;
use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::MouseButton;
use bevy::input::ButtonInput;
use bevy::math::{Rect, Vec2};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const ATLAS_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../assets/map/county_atlas.bin"
));

/// Cells per axis. 128×128 over the committed atlas's `world_bounds()`
/// (roughly 4.6M × 4.3M m) puts each cell at ~36km × ~34km — small enough
/// that a typical county's bbox overlaps a handful of cells, not
/// thousands.
const GRID_RESOLUTION: usize = 128;

/// A uniform-grid spatial index over one atlas's counties, owning its own
/// copy of every county's ring vertices so `county_at` needs no `&CountyAtlas`
/// parameter (matching the interface this task specifies) and no lifetime.
/// `Resource`: the real app builds one at Startup and reads it from the
/// hover-tracking system every frame.
#[derive(Resource)]
pub struct CountyIndex {
    bounds: Rect,
    /// `[row * GRID_RESOLUTION + col]` -> county indices whose bbox
    /// overlaps that cell.
    cell_candidates: Vec<Vec<u32>>,
    /// `[county_index][ring_index]` -> that ring's vertices, in order.
    county_rings: Vec<Vec<Vec<Vec2>>>,
}

// `GRID_RESOLUTION` (128) is a small, fixed const — its `as f32` cast loses
// no precision at that magnitude. `nx`/`ny` are clamped to [0.0, 0.999999)
// just above, so `nx * GRID_RESOLUTION as f32` always lands in
// [0.0, 128.0), and the `as usize` cast is always non-negative and always
// truncates into [0, 127] — never a sign loss, never an out-of-range
// truncation.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn cell_of(bounds: Rect, p: Vec2) -> (usize, usize) {
    let size = bounds.size();
    let nx = ((p.x - bounds.min.x) / size.x).clamp(0.0, 0.999_999);
    let ny = ((p.y - bounds.min.y) / size.y).clamp(0.0, 0.999_999);
    let col = ((nx * GRID_RESOLUTION as f32) as usize).min(GRID_RESOLUTION - 1);
    let row = ((ny * GRID_RESOLUTION as f32) as usize).min(GRID_RESOLUTION - 1);
    (col, row)
}

/// Builds a `CountyIndex` over every county in `atlas`.
#[must_use]
pub fn build(atlas: &CountyAtlas) -> CountyIndex {
    let bounds = atlas.world_bounds();
    let mut cell_candidates = vec![Vec::new(); GRID_RESOLUTION * GRID_RESOLUTION];
    let mut county_rings = Vec::with_capacity(atlas.len());
    let vertices = atlas.vertices();

    for county_index in 0..atlas.len() {
        let county = atlas.county(county_index).expect("index in range");

        let (c0, r0) = cell_of(bounds, county.bbox.min);
        let (c1, r1) = cell_of(bounds, county.bbox.max);
        for row in r0..=r1 {
            for col in c0..=c1 {
                // `atlas.len()` is the committed atlas's county count
                // (3,222 today) — always far under `u32::MAX`.
                #[allow(clippy::cast_possible_truncation)]
                cell_candidates[row * GRID_RESOLUTION + col].push(county_index as u32);
            }
        }

        let rings: Vec<Vec<Vec2>> = county
            .rings
            .iter()
            .map(|ring| {
                let start = ring.vertex_start as usize;
                let end = start + ring.vertex_count as usize;
                vertices[start..end].to_vec()
            })
            .collect();
        county_rings.push(rings);
    }

    CountyIndex {
        bounds,
        cell_candidates,
        county_rings,
    }
}

/// Even-odd ray-crossing test (PNPOLY): counts crossings of a rightward
/// horizontal ray from `p` against edge `(a, b)`. Folding every ring of a
/// county (exterior AND holes alike) into ONE shared crossing count is
/// what makes a hole invert membership for free — no separate hole-handling
/// branch needed.
fn ray_crosses(a: Vec2, b: Vec2, p: Vec2) -> bool {
    ((a.y > p.y) != (b.y > p.y)) && (p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x)
}

fn point_in_rings(rings: &[Vec<Vec2>], p: Vec2) -> bool {
    let mut crossings = 0u32;
    for ring in rings {
        let n = ring.len();
        if n < 3 {
            continue;
        }
        for i in 0..n {
            let a = ring[i];
            let b = ring[(i + 1) % n];
            if ray_crosses(a, b, p) {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

impl CountyIndex {
    /// The county index whose polygon (exterior minus holes) contains `p`,
    /// or `None` if no candidate's ring actually contains it — either
    /// because `p` sits outside every candidate's bbox (an empty grid
    /// cell) or because it sits inside a candidate's bbox but outside its
    /// actual ring (open water between two counties' bounding boxes, or a
    /// concave/crescent shape's own bbox corners).
    #[must_use]
    pub fn county_at(&self, p: Vec2) -> Option<usize> {
        if p.x < self.bounds.min.x
            || p.x > self.bounds.max.x
            || p.y < self.bounds.min.y
            || p.y > self.bounds.max.y
        {
            return None;
        }
        let (col, row) = cell_of(self.bounds, p);
        let candidates = &self.cell_candidates[row * GRID_RESOLUTION + col];
        for &county_idx in candidates {
            if point_in_rings(&self.county_rings[county_idx as usize], p) {
                return Some(county_idx as usize);
            }
        }
        None
    }
}

/// `Startup` system: builds the `CountyIndex` once (re-parsing the embedded
/// atlas locally — cheap, check-then-decode, matching every other system
/// in this crate that needs the atlas).
pub(super) fn build_county_index(mut commands: Commands) {
    let atlas = CountyAtlas::parse(ATLAS_BYTES)
        .unwrap_or_else(|e| panic!("county atlas failed to parse at startup: {e}"));
    commands.insert_resource(build(&atlas));
}

/// The cursor's world-space position, if the primary window has one and it
/// resolves through the map camera. Written by `track_cursor_world_position`
/// (the real, window/camera-dependent half); `update_hovered_county` reads
/// only this resource, so a test can write it directly and never needs a
/// real window or synthesized window events (this file's own testing
/// precedent, matching B1 Task 10's original design intent).
#[derive(Resource, Default)]
pub struct CursorWorldPosition(pub Option<Vec2>);

/// The county index under the cursor this frame, or `None`.
#[derive(Resource, Default)]
pub struct HoveredCounty(pub Option<usize>);

/// The county index the player last clicked, or `None` before any click.
/// An ATLAS INDEX, matching `county_at`'s own return type — never a
/// `NodeId` (Task 15 resolves the chain: atlas index -> fips -> `NodeId`).
#[derive(Resource, Default)]
pub struct SelectedCounty(pub Option<usize>);

/// `Update` system: converts the primary window's cursor position to world
/// space through the map camera and writes it to `CursorWorldPosition`.
/// Real-window/real-camera dependent — `update_hovered_county` below is
/// the pure, testable half this system feeds.
pub(super) fn track_cursor_world_position(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<super::MapCamera>>,
    mut cursor: ResMut<CursorWorldPosition>,
    viewport: Option<Res<crate::observer_ui::ObserverViewport>>,
    ui: Option<Res<crate::observer_ui::ObserverUiState>>,
    buttons: Query<&Interaction, With<Button>>,
) {
    if ui.as_ref().is_some_and(|ui| ui.menu_open)
        || (viewport.is_some()
            && buttons
                .iter()
                .any(|interaction| *interaction != Interaction::None))
    {
        cursor.0 = None;
        return;
    }
    let Ok(window) = windows.single() else {
        cursor.0 = None;
        return;
    };
    let Some(screen_pos) = window.cursor_position() else {
        cursor.0 = None;
        return;
    };
    if viewport
        .as_ref()
        .and_then(|v| v.0)
        .is_some_and(|rect| !rect.contains(screen_pos))
    {
        cursor.0 = None;
        return;
    }
    let Ok((camera, camera_transform)) = cameras.single() else {
        cursor.0 = None;
        return;
    };
    if !camera.is_active {
        cursor.0 = None;
        return;
    }
    cursor.0 = camera
        .viewport_to_world_2d(camera_transform, screen_pos)
        .ok();
}

/// `Update` system: `CursorWorldPosition` -> `county_at` -> `HoveredCounty`.
/// Pure logic over two resources — testable by writing `CursorWorldPosition`
/// directly, no window needed.
pub(super) fn update_hovered_county(
    cursor: Res<CursorWorldPosition>,
    index: Res<CountyIndex>,
    mut hovered: ResMut<HoveredCounty>,
) {
    hovered.0 = cursor.0.and_then(|p| index.county_at(p));
}

/// `Update` system: a left click promotes the currently-hovered county to
/// `SelectedCounty`.
pub(super) fn promote_selection_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredCounty>,
    mut selected: ResMut<SelectedCounty>,
    cursor: Res<CursorWorldPosition>,
    observer: Option<Res<crate::observer::ObserverSession>>,
) {
    if observer.is_some() && cursor.0.is_none() {
        return;
    }
    if buttons.just_pressed(MouseButton::Left) {
        selected.0 = hovered.0;
    }
}

/// Marks the GOLD selection-outline entity.
#[derive(Component)]
pub struct SelectionOutline;

/// `Update` system: redraws the GOLD outline over `SelectedCounty`'s own
/// rings at `z = 2.0` whenever the selection changes — reusing
/// `map/mesh.rs::build_border_mesh`'s own `LineList`-over-every-ring-edge
/// shape, scoped to one county instead of all of them.
pub(super) fn update_selection_outline(
    selected: Res<SelectedCounty>,
    atlas: Res<CountyAtlas>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing: Query<Entity, With<SelectionOutline>>,
) {
    if !selected.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let Some(idx) = selected.0 else {
        return;
    };
    let county = atlas
        .county(idx)
        .expect("selected index is within 0..atlas.len()");
    let vertices = atlas.vertices();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for ring in county.rings {
        let start = ring.vertex_start as usize;
        let end = start + ring.vertex_count as usize;
        let ring_vertices = &vertices[start..end];
        let n = ring_vertices.len();
        for i in 0..n {
            let a = ring_vertices[i];
            let b = ring_vertices[(i + 1) % n];
            positions.push([a.x, a.y, 0.0]);
            positions.push([b.x, b.y, 0.0]);
        }
    }
    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    commands.spawn((
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(ColorMaterial::from(palette::GOLD))),
        Transform::from_xyz(0.0, 0.0, 2.0),
        DeclaredSurface::new(SurfaceId::CountyMap),
        SelectionOutline,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> CountyAtlas {
        CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses")
    }

    /// Every county's own centroid resolves to itself, measured as a FLOOR
    /// rather than claimed as 100% — a centroid can legitimately fall
    /// outside a crescent-shaped or multi-polygon county. Pinned from a
    /// real run against the committed atlas (2026-08-11): 3209/3222
    /// (99.596%). The 13 named exceptions are real FIPS this atlas
    /// actually carries, not guessed.
    #[test]
    fn every_county_centroid_resolves_to_itself_above_a_measured_floor() {
        const KNOWN_EXCEPTIONS: [&str; 13] = [
            "02016", "06075", "12087", "15003", "22045", "29069", "29093", "51087", "51089",
            "51161", "51199", "55091", "72097",
        ];
        let atlas = atlas();
        let index = build(&atlas);
        let mut misses = Vec::new();
        for i in 0..atlas.len() {
            let county = atlas.county(i).expect("index in range");
            if index.county_at(county.centroid) != Some(i) {
                misses.push(county.fips.to_owned());
            }
        }
        // atlas.len() is 3,222 — trivially exact in f64 (mantissa 52 bits).
        #[allow(clippy::cast_precision_loss)]
        let hit_rate = (atlas.len() - misses.len()) as f64 / atlas.len() as f64;
        assert!(
            hit_rate >= 0.995,
            "centroid hit rate dropped below the measured floor: {hit_rate} (misses: {misses:?})"
        );
        let known: std::collections::HashSet<&str> = KNOWN_EXCEPTIONS.iter().copied().collect();
        for fips in &misses {
            assert!(
                known.contains(fips.as_str()),
                "new, previously-unseen centroid miss {fips} — the atlas geometry moved; \
                 re-run map::pick's own exploratory sweep (see git history) and update this list"
            );
        }
    }

    /// A point in a grid cell with zero county-bbox candidates gives
    /// `None` — the honest "empty region" case (open water or an atlas
    /// coverage gap), computed from the real atlas rather than a guessed
    /// real-world "Gulf of Mexico" coordinate this test cannot
    /// independently verify against the atlas's own Albers projection.
    #[test]
    fn a_point_with_no_bbox_candidates_gives_none() {
        let index = build(&atlas());
        let empty_region_point = Vec2::new(-2_343_514.5, -1_115_947.1);
        assert_eq!(index.county_at(empty_region_point), None);
    }

    /// A point inside county 1's (FIPS 01003, Baldwin County) bounding box
    /// but outside its actual ring gives `None` — proving `county_at` is
    /// more than a bounding-box lookup.
    #[test]
    fn a_point_inside_a_bbox_but_outside_its_ring_gives_none() {
        let atlas = atlas();
        let index = build(&atlas);
        let baldwin = atlas.county(1).expect("index 1 exists");
        assert_eq!(baldwin.fips, "01003");
        let corner_outside_the_ring = Vec2::new(760_603.2, 817_726.1);
        assert!(
            corner_outside_the_ring.x >= baldwin.bbox.min.x
                && corner_outside_the_ring.x <= baldwin.bbox.max.x
                && corner_outside_the_ring.y >= baldwin.bbox.min.y
                && corner_outside_the_ring.y <= baldwin.bbox.max.y,
            "fixture point must actually sit inside Baldwin's own bbox"
        );
        assert_eq!(index.county_at(corner_outside_the_ring), None);
    }

    /// The index is identical across two builds — same query point, same
    /// result, built from the same atlas twice.
    #[test]
    fn the_index_is_identical_across_two_builds() {
        let atlas = atlas();
        let a = build(&atlas);
        let b = build(&atlas);
        for i in [0usize, 1, 100, 1000, 3000] {
            let county = atlas.county(i).expect("index in range");
            assert_eq!(a.county_at(county.centroid), b.county_at(county.centroid));
        }
        let empty_region_point = Vec2::new(-2_343_514.5, -1_115_947.1);
        assert_eq!(
            a.county_at(empty_region_point),
            b.county_at(empty_region_point)
        );
    }

    /// A point outside `world_bounds()` entirely gives `None` directly,
    /// never a clamped false positive from an edge grid cell.
    #[test]
    fn a_point_outside_world_bounds_gives_none() {
        let index = build(&atlas());
        assert_eq!(index.county_at(Vec2::new(f32::MAX, f32::MAX)), None);
        assert_eq!(index.county_at(Vec2::new(f32::MIN, f32::MIN)), None);
    }

    /// Step 3/5's wiring proof: a cursor position written DIRECTLY into
    /// `CursorWorldPosition` (never a synthesized window event — this
    /// crate's own established testing precedent) flows through
    /// `update_hovered_county` into `HoveredCounty`, and a left click
    /// promotes it into `SelectedCounty`.
    #[test]
    fn a_known_world_point_sets_hovered_county_and_a_click_promotes_it() {
        // No `InputPlugin`: its `PreUpdate` `mouse_button_input_system`
        // unconditionally clears `just_pressed`/`just_released` every
        // frame to make room for real `MouseButtonInput` events — since
        // this test drives `ButtonInput` directly (bypassing the event
        // pipeline, matching this crate's own established precedent for
        // input tests), that clear would wipe a manually-set
        // `just_pressed` before `promote_selection_on_click` (an `Update`
        // system, scheduled after `PreUpdate`) ever sees it. Inserting the
        // resource directly gets the type without the clearing system.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.insert_resource(CursorWorldPosition::default());
        app.insert_resource(HoveredCounty::default());
        app.insert_resource(SelectedCounty::default());
        app.add_systems(
            Update,
            (update_hovered_county, promote_selection_on_click).chain(),
        );
        app.add_systems(Startup, build_county_index);
        app.update(); // Startup: build_county_index.

        let atlas = atlas();
        let known = atlas.county(0).expect("index 0 exists");
        app.world_mut().resource_mut::<CursorWorldPosition>().0 = Some(known.centroid);
        app.update();

        assert_eq!(
            app.world().resource::<HoveredCounty>().0,
            Some(0),
            "the known centroid must hover county 0"
        );
        assert_eq!(
            app.world().resource::<SelectedCounty>().0,
            None,
            "no click yet"
        );

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();

        assert_eq!(
            app.world().resource::<SelectedCounty>().0,
            Some(0),
            "a left click promotes the hovered county to selected"
        );
    }
}
