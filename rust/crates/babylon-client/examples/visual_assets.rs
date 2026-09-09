use babylon_client::visual_assets::{VisualAssetGalleryPlugin, VisualAssetsPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(babylon_client::palette::FIELD))
        .add_plugins((VisualAssetsPlugin, VisualAssetGalleryPlugin))
        .run();
}
