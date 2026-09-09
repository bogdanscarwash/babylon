//! B3 wave-1 Task 5.4's own proof (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md` §2.11):
//! `MapBinding::None` hides the county fill/border meshes and renders the
//! declared absence banner. Launching `carceral` leaves zero counties
//! painted and the banner present — and no county is painted a stale color
//! (the FB1 class of bug, one axis over; `map/bands.rs:171-190` documents
//! the original: without a pre-clear, a county absent from the incoming
//! lens's own resolved cells keeps whatever color a PREVIOUSLY active lens
//! painted there). Real `MapPlugin` + `TickLoopPlugin`, zero hand-installed
//! resources beyond `SelectedStory` — the same wiring-proof shape every
//! other headless test in this crate uses.

use babylon_client::map::{MapBorders, MapFill, MapSurface, PANEL};
use babylon_client::story::{carceral, counties, SelectedStory};
use babylon_client::ui::story_card::MapAbsenceBannerText;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;

fn new_app(story: &'static babylon_client::story::Story) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(SelectedStory(story));
    app.update(); // Startup + first Update pass.
    app
}

fn banner_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<MapAbsenceBannerText>>();
    query
        .single(world)
        .expect("exactly one map-absence banner entity")
        .0
        .clone()
}

fn mesh_visibilities(app: &mut App) -> Vec<Visibility> {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Visibility, Or<(With<MapFill>, With<MapBorders>)>>();
    query.iter(world).copied().collect()
}

/// These `[f32; 4]` arrays are exact byte-for-byte copies read straight off
/// the live fill mesh's vertex buffer — exact comparison is correct, same
/// justification as `map/bands.rs`'s own recolor tests.
#[allow(clippy::float_cmp)]
fn all_fill_vertices_are_panel(app: &App) -> bool {
    let surface = app.world().resource::<MapSurface>();
    let meshes = app.world().resource::<Assets<Mesh>>();
    let mesh = meshes
        .get(&surface.fill_mesh)
        .expect("fill mesh is registered");
    let expected_panel = PANEL.to_linear().to_f32_array();
    match mesh
        .attribute(Mesh::ATTRIBUTE_COLOR)
        .expect("fill mesh carries per-vertex color")
    {
        bevy::mesh::VertexAttributeValues::Float32x4(colors) => {
            colors.iter().all(|c| *c == expected_panel)
        }
        other => panic!("unexpected color attribute shape: {other:?}"),
    }
}

#[test]
fn launching_carceral_hides_the_map_and_shows_the_absence_banner() {
    let mut app = new_app(carceral());

    // The declared absence banner (§2.11) must name the story and the
    // "not applicable" fact.
    let banner = banner_text(&mut app);
    assert!(
        !banner.is_empty(),
        "the §2.11 absence banner must render for a MapBinding::None story"
    );
    assert!(banner.contains(carceral().title));
    assert!(banner.contains("0 territories"));

    // The fill/border mesh entities must both be hidden — "hides the
    // fill/border meshes", not merely leaves them all-PANEL.
    let visibilities = mesh_visibilities(&mut app);
    assert_eq!(
        visibilities.len(),
        2,
        "exactly one MapFill and one MapBorders entity carry Visibility"
    );
    assert!(
        visibilities.iter().all(|v| *v == Visibility::Hidden),
        "both the fill and border mesh entities must be Visibility::Hidden \
         for a MapBinding::None story, got {visibilities:?}"
    );

    // Zero counties painted — every fill vertex reads PANEL (the FB1-class
    // proof, one axis over: no county was left showing a stale color from
    // some earlier lens computation, because carceral's own derived roster
    // is empty, so every lens's own resolved-cells loop touches nothing).
    assert!(
        all_fill_vertices_are_panel(&app),
        "every county must read PANEL under a MapBinding::None story — \
         zero counties painted, none left showing a stale color"
    );
}

#[test]
fn launching_counties_shows_the_map_with_no_absence_banner() {
    let mut app = new_app(counties());

    let banner = banner_text(&mut app);
    assert!(
        banner.is_empty(),
        "a MapBinding::Fips story must render no absence banner, got {banner:?}"
    );

    let visibilities = mesh_visibilities(&mut app);
    assert_eq!(visibilities.len(), 2);
    assert!(
        visibilities.iter().all(|v| *v == Visibility::Inherited),
        "counties' own fill/border meshes must stay visible, got {visibilities:?}"
    );
}
