//! Shared atlas and county selection for the durable observer's geographic views.

use crate::atlas::CountyAtlas;
use bevy::input::InputPlugin;
use bevy::prelude::*;

/// The atlas index under the cursor this frame, or `None`.
#[derive(Resource, Default)]
pub struct HoveredCounty(pub Option<usize>);

/// The atlas index selected by the observer, or `None`.
#[derive(Resource, Default)]
pub struct SelectedCounty(pub Option<usize>);

/// Loads the embedded geographic substrate before the observer builds its scene.
///
/// # Panics
/// If the embedded county atlas fails validation.
pub fn load_county_atlas(mut commands: Commands) {
    let atlas = CountyAtlas::parse(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    )))
    .unwrap_or_else(|error| panic!("county atlas failed to parse at startup: {error}"));
    commands.insert_resource(atlas);
}

/// Registers the resources required by the observer's map and Archive selection.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::mesh::MeshPlugin>() {
            app.add_plugins(bevy::mesh::MeshPlugin);
        }
        if !app.is_plugin_added::<bevy::sprite_render::ColorMaterialPlugin>() {
            app.add_plugins(bevy::sprite_render::ColorMaterialPlugin);
        }
        if !app.is_plugin_added::<InputPlugin>() {
            app.add_plugins(InputPlugin);
        }
        app.init_resource::<HoveredCounty>()
            .init_resource::<SelectedCounty>()
            .add_systems(Startup, load_county_atlas);
    }
}
