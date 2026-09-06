//! Embedded reading and subject-display font faces.

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::asset::{AssetServer, Handle};
use bevy::prelude::{App, Font, Resource};
use std::path::{Path, PathBuf};

const EMBEDDED_FONTS: [(&str, &[u8]); 2] = [
    (
        "SourceSans3[wght].ttf",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/fonts/SourceSans3[wght].ttf"
        )),
    ),
    (
        "BarlowCondensed-SemiBold.ttf",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/fonts/BarlowCondensed-SemiBold.ttf"
        )),
    ),
];

/// Font faces loaded through the same embedded asset server as the visual art.
#[derive(Clone, Resource)]
pub struct ObserverFonts {
    /// Source Sans 3, with variable weights from 200 through 900.
    pub body: Handle<Font>,
    /// Barlow Condensed's static semibold face for short subject headings.
    pub display: Handle<Font>,
}

/// Requires Bevy's font asset support, supplied by the native `TextPlugin`.
pub(super) fn install(app: &mut App) {
    let registry = app.world().resource::<EmbeddedAssetRegistry>();
    for (name, bytes) in EMBEDDED_FONTS {
        registry.insert_asset(
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/fonts"
            ))
            .join(name),
            &Path::new("fonts").join(name),
            bytes,
        );
    }
    let server = app.world().resource::<AssetServer>();
    let fonts = ObserverFonts {
        body: server.load("embedded://fonts/SourceSans3[wght].ttf"),
        display: server.load("embedded://fonts/BarlowCondensed-SemiBold.ttf"),
    };
    app.insert_resource(fonts);
}
