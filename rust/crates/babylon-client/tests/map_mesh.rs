use bevy::asset::AssetPlugin;
use bevy::prelude::*;

/// The CI-shaped pattern for the whole B1 milestone: `MinimalPlugins` plus
/// `AssetPlugin`, never `DefaultPlugins` — the `rust-gate` CI runner has no
/// display server and no GPU (Global Constraints, "CI reality").
#[test]
fn map_plugin_builds_the_fill_mesh_headless() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update();

    let world = app.world_mut();
    let handle = world
        .query_filtered::<&Mesh2d, With<babylon_client::map::MapFill>>()
        .single(world)
        .expect("exactly one map fill entity");
    let meshes = world.resource::<Assets<Mesh>>();
    let mesh = meshes.get(&handle.0).expect("fill mesh is registered");
    assert_eq!(
        mesh.primitive_topology(),
        bevy::mesh::PrimitiveTopology::TriangleList
    );
    assert!(
        mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some(),
        "choropleth needs vertex colors"
    );
    // Startup already loaded this exact atlas. Assert that every source
    // vertex reached the spawned mesh without another parse or tessellation.
    let atlas = world.resource::<babylon_client::atlas::CountyAtlas>();
    assert_eq!(mesh.count_vertices(), atlas.vertices().len());
}
