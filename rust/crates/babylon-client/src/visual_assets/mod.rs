//! Embedded visual assets and their typed Bevy loading surface.

mod catalog;
mod fonts;
mod gallery;
mod presentation;

pub use catalog::{
    FrameKind, InterfaceIcon, MarkerIcon, ProvenanceIcon, SurfaceKind, VisualAssetDescriptor,
    VisualAssetId, VISUAL_ASSET_CATALOG,
};
pub use fonts::ObserverFonts;
pub use gallery::{GalleryAssetLabel, GalleryScrollRoot, VisualAssetGalleryPlugin};
pub use presentation::{ReadableTitle, StoryBanner, TitleMark, VisualPresentationPlugin};

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::asset::{AssetApp, AssetServer, Assets, Handle};
use bevy::image::{Image, ImageLoaderSettings, ImageSampler, TextureAtlasLayout};
use bevy::prelude::{App, Plugin, Resource, UVec2};
use std::path::{Path, PathBuf};

fn nearest_image_settings(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::nearest();
}

fn linear_image_settings(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::linear();
}

const EMBEDDED_VISUALS: [(&str, &[u8]); 16] = [
    (
        "banner-carceral.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/banner-carceral.webp"
        )) as &[u8],
    ),
    (
        "banner-collapse.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/banner-collapse.webp"
        )) as &[u8],
    ),
    (
        "banner-counties.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/banner-counties.webp"
        )) as &[u8],
    ),
    (
        "banner-topology.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/banner-topology.webp"
        )) as &[u8],
    ),
    (
        "concept-bunker-oracle.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/concept-bunker-oracle.webp"
        )) as &[u8],
    ),
    (
        "concept-carceral-circuit.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/concept-carceral-circuit.webp"
        )) as &[u8],
    ),
    (
        "concept-living-map.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/concept-living-map.webp"
        )) as &[u8],
    ),
    (
        "concept-metabolic-rift.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/concept-metabolic-rift.webp"
        )) as &[u8],
    ),
    (
        "frame-atlas.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/frame-atlas.png"
        )) as &[u8],
    ),
    (
        "hero-empire-anatomized.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/hero-empire-anatomized.webp"
        )) as &[u8],
    ),
    (
        "hero-red-apparatus.webp",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/hero-red-apparatus.webp"
        )) as &[u8],
    ),
    (
        "interface-atlas.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/interface-atlas.png"
        )) as &[u8],
    ),
    (
        "marker-atlas.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/marker-atlas.png"
        )) as &[u8],
    ),
    (
        "provenance-atlas.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/provenance-atlas.png"
        )) as &[u8],
    ),
    (
        "surface-atlas.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/surface-atlas.png"
        )) as &[u8],
    ),
    (
        "title-mark.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/visual/title-mark.png"
        )) as &[u8],
    ),
];

/// Loads the complete embedded visual-asset family and its atlas layouts.
#[derive(Default)]
pub struct VisualAssetsPlugin;

impl Plugin for VisualAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TextureAtlasLayout>();
        fonts::install(app);

        let registry = app.world().resource::<EmbeddedAssetRegistry>();
        for (name, bytes) in EMBEDDED_VISUALS {
            registry.insert_asset(
                PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/visual"
                ))
                .join(name),
                &Path::new("visual").join(name),
                bytes,
            );
        }
        let server = app.world().resource::<AssetServer>();

        let title_mark =
            server.load_with_settings("embedded://visual/title-mark.png", nearest_image_settings);
        let interface_atlas = server.load_with_settings(
            "embedded://visual/interface-atlas.png",
            nearest_image_settings,
        );
        let marker_atlas =
            server.load_with_settings("embedded://visual/marker-atlas.png", nearest_image_settings);
        let provenance_atlas = server.load_with_settings(
            "embedded://visual/provenance-atlas.png",
            nearest_image_settings,
        );
        let frame_atlas =
            server.load_with_settings("embedded://visual/frame-atlas.png", nearest_image_settings);
        let surface_atlas = server.load_with_settings(
            "embedded://visual/surface-atlas.png",
            nearest_image_settings,
        );
        let hero_red_apparatus = server.load_with_settings(
            "embedded://visual/hero-red-apparatus.webp",
            linear_image_settings,
        );
        let hero_empire_anatomized = server.load_with_settings(
            "embedded://visual/hero-empire-anatomized.webp",
            linear_image_settings,
        );
        let concept_bunker_oracle = server.load_with_settings(
            "embedded://visual/concept-bunker-oracle.webp",
            linear_image_settings,
        );
        let concept_living_map = server.load_with_settings(
            "embedded://visual/concept-living-map.webp",
            linear_image_settings,
        );
        let concept_carceral_circuit = server.load_with_settings(
            "embedded://visual/concept-carceral-circuit.webp",
            linear_image_settings,
        );
        let concept_metabolic_rift = server.load_with_settings(
            "embedded://visual/concept-metabolic-rift.webp",
            linear_image_settings,
        );
        let banner_counties = server.load_with_settings(
            "embedded://visual/banner-counties.webp",
            linear_image_settings,
        );
        let banner_carceral = server.load_with_settings(
            "embedded://visual/banner-carceral.webp",
            linear_image_settings,
        );
        let banner_topology = server.load_with_settings(
            "embedded://visual/banner-topology.webp",
            linear_image_settings,
        );
        let banner_collapse = server.load_with_settings(
            "embedded://visual/banner-collapse.webp",
            linear_image_settings,
        );

        let (interface_layout, marker_layout, provenance_layout, frame_layout, surface_layout) =
            create_atlas_layouts(app);

        app.insert_resource(VisualAssets {
            hero_red_apparatus,
            hero_empire_anatomized,
            concept_bunker_oracle,
            concept_living_map,
            concept_carceral_circuit,
            concept_metabolic_rift,
            banner_counties,
            banner_carceral,
            banner_topology,
            banner_collapse,
            title_mark,
            interface_atlas,
            marker_atlas,
            provenance_atlas,
            frame_atlas,
            surface_atlas,
            interface_layout,
            marker_layout,
            provenance_layout,
            frame_layout,
            surface_layout,
        });
    }
}

type AtlasLayouts = (
    Handle<TextureAtlasLayout>,
    Handle<TextureAtlasLayout>,
    Handle<TextureAtlasLayout>,
    Handle<TextureAtlasLayout>,
    Handle<TextureAtlasLayout>,
);

fn create_atlas_layouts(app: &mut App) -> AtlasLayouts {
    let mut layouts = app.world_mut().resource_mut::<Assets<TextureAtlasLayout>>();
    let interface_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(128, 128),
        4,
        4,
        None,
        None,
    ));
    let marker_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(128, 128),
        3,
        2,
        None,
        None,
    ));
    let provenance_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(128, 128),
        2,
        2,
        None,
        None,
    ));
    let frame_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(64, 64),
        4,
        1,
        None,
        None,
    ));
    let surface_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(128, 128),
        3,
        1,
        None,
        None,
    ));
    (
        interface_layout,
        marker_layout,
        provenance_layout,
        frame_layout,
        surface_layout,
    )
}

/// Named image and layout handles for the complete visual-asset family.
#[derive(Clone, Resource)]
pub struct VisualAssets {
    /// The Red Apparatus hero illustration.
    pub hero_red_apparatus: Handle<Image>,
    /// The Empire Anatomized hero illustration.
    pub hero_empire_anatomized: Handle<Image>,
    /// The Bunker Oracle concept illustration.
    pub concept_bunker_oracle: Handle<Image>,
    /// The Living Map concept illustration.
    pub concept_living_map: Handle<Image>,
    /// The Carceral Circuit concept illustration.
    pub concept_carceral_circuit: Handle<Image>,
    /// The Metabolic Rift concept illustration.
    pub concept_metabolic_rift: Handle<Image>,
    /// The counties-story banner.
    pub banner_counties: Handle<Image>,
    /// The carceral-story banner.
    pub banner_carceral: Handle<Image>,
    /// The topology-story banner.
    pub banner_topology: Handle<Image>,
    /// The collapse-story banner.
    pub banner_collapse: Handle<Image>,
    /// The Babylon title mark.
    pub title_mark: Handle<Image>,
    /// The sixteen-cell interface icon atlas.
    pub interface_atlas: Handle<Image>,
    /// The six-cell map-marker atlas.
    pub marker_atlas: Handle<Image>,
    /// The four-cell projection-provenance atlas.
    pub provenance_atlas: Handle<Image>,
    /// The four-cell frame atlas.
    pub frame_atlas: Handle<Image>,
    /// The three-cell surface-texture atlas.
    pub surface_atlas: Handle<Image>,
    /// The sixteen-cell interface icon layout.
    pub interface_layout: Handle<TextureAtlasLayout>,
    /// The six-cell map-marker layout.
    pub marker_layout: Handle<TextureAtlasLayout>,
    /// The four-cell projection-provenance layout.
    pub provenance_layout: Handle<TextureAtlasLayout>,
    /// The four-cell frame layout.
    pub frame_layout: Handle<TextureAtlasLayout>,
    /// The three-cell surface-texture layout.
    pub surface_layout: Handle<TextureAtlasLayout>,
}

impl VisualAssets {
    /// Returns a clone of the image handle for `id`.
    #[must_use]
    pub fn image(&self, id: VisualAssetId) -> Handle<Image> {
        match id {
            VisualAssetId::TitleMark => self.title_mark.clone(),
            VisualAssetId::InterfaceAtlas => self.interface_atlas.clone(),
            VisualAssetId::MarkerAtlas => self.marker_atlas.clone(),
            VisualAssetId::ProvenanceAtlas => self.provenance_atlas.clone(),
            VisualAssetId::FrameAtlas => self.frame_atlas.clone(),
            VisualAssetId::SurfaceAtlas => self.surface_atlas.clone(),
            VisualAssetId::HeroRedApparatus => self.hero_red_apparatus.clone(),
            VisualAssetId::HeroEmpireAnatomized => self.hero_empire_anatomized.clone(),
            VisualAssetId::ConceptBunkerOracle => self.concept_bunker_oracle.clone(),
            VisualAssetId::ConceptLivingMap => self.concept_living_map.clone(),
            VisualAssetId::ConceptCarceralCircuit => self.concept_carceral_circuit.clone(),
            VisualAssetId::ConceptMetabolicRift => self.concept_metabolic_rift.clone(),
            VisualAssetId::BannerCounties => self.banner_counties.clone(),
            VisualAssetId::BannerCarceral => self.banner_carceral.clone(),
            VisualAssetId::BannerTopology => self.banner_topology.clone(),
            VisualAssetId::BannerCollapse => self.banner_collapse.clone(),
        }
    }
}
