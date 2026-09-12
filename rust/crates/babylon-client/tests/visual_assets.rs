//! Behavioral contracts for the embedded Bevy visual-asset surface.

use bevy::asset::AssetPlugin;
use bevy::image::ImagePlugin;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::texture::TexturePlugin;
use std::sync::{Mutex, MutexGuard};

static BEVY_APP_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn add_visual_asset_plugins(app: &mut App) {
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::text::TextPlugin,
        // Bevy 0.18 registers the concrete image loader in this plugin's finish phase.
        TexturePlugin,
    ));
    app.add_plugins(babylon_client::visual_assets::VisualAssetsPlugin);
}

fn bevy_app_test_guard() -> MutexGuard<'static, ()> {
    match BEVY_APP_TEST_MUTEX.lock() {
        Ok(guard) => guard,
        // Several invariant tests deliberately panic while holding this guard, so later app tests
        // must recover the mutex rather than conceal the assertion.
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn visual_asset_gallery_app() -> App {
    let mut app = App::new();
    add_visual_asset_plugins(&mut app);
    app.add_plugins(babylon_client::visual_assets::VisualAssetGalleryPlugin);
    app.finish();
    app.update();
    app
}

type ExpectedVisualAsset = (
    babylon_client::visual_assets::VisualAssetId,
    &'static str,
    u32,
    u32,
    u32,
    u32,
);

fn expected_interface_assets() -> [ExpectedVisualAsset; 6] {
    use babylon_client::visual_assets::VisualAssetId;

    [
        (VisualAssetId::TitleMark, "Title mark", 768, 192, 1, 1),
        (
            VisualAssetId::InterfaceAtlas,
            "Interface atlas",
            512,
            512,
            4,
            4,
        ),
        (VisualAssetId::MarkerAtlas, "Marker atlas", 384, 256, 3, 2),
        (
            VisualAssetId::ProvenanceAtlas,
            "Provenance atlas",
            256,
            256,
            2,
            2,
        ),
        (VisualAssetId::FrameAtlas, "Frame atlas", 256, 64, 4, 1),
        (VisualAssetId::SurfaceAtlas, "Surface atlas", 384, 128, 3, 1),
    ]
}

fn expected_illustration_assets() -> [ExpectedVisualAsset; 10] {
    use babylon_client::visual_assets::VisualAssetId;

    [
        (
            VisualAssetId::HeroRedApparatus,
            "Red Apparatus",
            1536,
            864,
            1,
            1,
        ),
        (
            VisualAssetId::HeroEmpireAnatomized,
            "Empire Anatomized",
            1536,
            864,
            1,
            1,
        ),
        (
            VisualAssetId::ConceptBunkerOracle,
            "Bunker Oracle",
            1024,
            1024,
            1,
            1,
        ),
        (
            VisualAssetId::ConceptLivingMap,
            "Living Map",
            1024,
            1024,
            1,
            1,
        ),
        (
            VisualAssetId::ConceptCarceralCircuit,
            "Carceral Circuit",
            1024,
            1024,
            1,
            1,
        ),
        (
            VisualAssetId::ConceptMetabolicRift,
            "Metabolic Rift",
            1024,
            1024,
            1,
            1,
        ),
        (
            VisualAssetId::BannerCounties,
            "Counties banner",
            1536,
            384,
            1,
            1,
        ),
        (
            VisualAssetId::BannerCarceral,
            "Carceral banner",
            1536,
            384,
            1,
            1,
        ),
        (
            VisualAssetId::BannerTopology,
            "Topology banner",
            1536,
            384,
            1,
            1,
        ),
        (
            VisualAssetId::BannerCollapse,
            "Collapse banner",
            1536,
            384,
            1,
            1,
        ),
    ]
}

fn assert_visual_asset(
    actual: &babylon_client::visual_assets::VisualAssetDescriptor,
    expected: ExpectedVisualAsset,
    index: usize,
) {
    let (id, label, width, height, columns, rows) = expected;
    assert_eq!(actual.id, id, "catalog id at index {index} drifted");
    assert_eq!(
        actual.label, label,
        "catalog label at index {index} drifted"
    );
    assert_eq!(
        (actual.width, actual.height, actual.columns, actual.rows),
        (width, height, columns, rows),
        "catalog dimensions at index {index} drifted"
    );
}

#[test]
fn typed_catalog_pins_every_asset_id_and_atlas_dimension() {
    let catalog = &babylon_client::visual_assets::VISUAL_ASSET_CATALOG;
    let expected_interfaces = expected_interface_assets();
    for index in 0..6 {
        assert_visual_asset(&catalog[index], expected_interfaces[index], index);
    }

    let expected_illustrations = expected_illustration_assets();
    for (index, expected) in expected_illustrations.into_iter().enumerate() {
        let catalog_index = index + 6;
        assert_visual_asset(&catalog[catalog_index], expected, catalog_index);
    }
}

#[test]
fn every_typed_atlas_enum_variant_has_its_declared_index() {
    use babylon_client::visual_assets::{
        FrameKind, InterfaceIcon, MarkerIcon, ProvenanceIcon, SurfaceKind,
    };

    let interface_indices = [
        InterfaceIcon::Play.index(),
        InterfaceIcon::Pause.index(),
        InterfaceIcon::Step.index(),
        InterfaceIcon::Speed.index(),
        InterfaceIcon::Lens.index(),
        InterfaceIcon::Map.index(),
        InterfaceIcon::Story.index(),
        InterfaceIcon::Beat.index(),
        InterfaceIcon::Roster.index(),
        InterfaceIcon::Material.index(),
        InterfaceIcon::Topology.index(),
        InterfaceIcon::Flow.index(),
        InterfaceIcon::Pin.index(),
        InterfaceIcon::Inspect.index(),
        InterfaceIcon::Warning.index(),
        InterfaceIcon::Close.index(),
    ];
    assert_eq!(
        interface_indices,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
    );
    assert_eq!(InterfaceIcon::COUNT, interface_indices.len());

    let marker_indices = [
        MarkerIcon::Hover.index(),
        MarkerIcon::Selection.index(),
        MarkerIcon::Pin.index(),
        MarkerIcon::Event.index(),
        MarkerIcon::Origin.index(),
        MarkerIcon::Target.index(),
    ];
    assert_eq!(marker_indices, [0, 1, 2, 3, 4, 5]);
    assert_eq!(MarkerIcon::COUNT, marker_indices.len());

    let provenance_indices = [
        ProvenanceIcon::Material.index(),
        ProvenanceIcon::Absent.index(),
        ProvenanceIcon::NotComputed.index(),
        ProvenanceIcon::Redacted.index(),
    ];
    assert_eq!(provenance_indices, [0, 1, 2, 3]);
    assert_eq!(ProvenanceIcon::COUNT, provenance_indices.len());

    let frame_indices = [
        FrameKind::Neutral.index(),
        FrameKind::Selected.index(),
        FrameKind::Critical.index(),
        FrameKind::Absent.index(),
    ];
    assert_eq!(frame_indices, [0, 1, 2, 3]);
    assert_eq!(FrameKind::COUNT, frame_indices.len());

    let surface_indices = [
        SurfaceKind::Concrete.index(),
        SurfaceKind::Hatch.index(),
        SurfaceKind::Paper.index(),
    ];
    assert_eq!(surface_indices, [0, 1, 2]);
    assert_eq!(SurfaceKind::COUNT, surface_indices.len());
}

#[test]
fn gallery_labels_match_the_fixed_visual_asset_catalog() {
    let _guard = bevy_app_test_guard();
    let mut app = visual_asset_gallery_app();
    let world = app.world_mut();
    let mut labels =
        world.query_filtered::<&Text, With<babylon_client::visual_assets::GalleryAssetLabel>>();
    let actual: Vec<_> = labels
        .iter(world)
        .take(17)
        .map(|text| text.0.clone())
        .collect();
    let expected: Vec<_> = babylon_client::visual_assets::VISUAL_ASSET_CATALOG
        .iter()
        .map(|descriptor| descriptor.label.to_owned())
        .collect();

    assert_eq!(actual.len(), 16);
    assert_eq!(actual, expected);
}

#[test]
fn gallery_scroll_position_changes_from_input_and_clamps_at_the_layout_maximum() {
    let _guard = bevy_app_test_guard();
    let mut app = visual_asset_gallery_app();
    let root = {
        let world = app.world_mut();
        let mut roots = world
            .query_filtered::<Entity, With<babylon_client::visual_assets::GalleryScrollRoot>>();
        roots
            .single(world)
            .expect("exactly one gallery scroll root")
    };
    app.world_mut().entity_mut(root).insert(ComputedNode {
        size: Vec2::new(100.0, 100.0),
        content_size: Vec2::new(100.0, 300.0),
        ..ComputedNode::DEFAULT
    });
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -1.0),
    });

    app.update();

    let scroll_position = app
        .world()
        .entity(root)
        .get::<ScrollPosition>()
        .expect("gallery root retains its scroll position");
    assert!(
        (scroll_position.y - 20.0).abs() < f32::EPSILON,
        "one line of input must move the gallery by 20 logical pixels"
    );

    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Pixel,
        delta: Vec2::new(0.0, -1000.0),
    });
    app.update();

    let scroll_position = app
        .world()
        .entity(root)
        .get::<ScrollPosition>()
        .expect("gallery root retains its scroll position");
    assert!(
        (scroll_position.y - 200.0).abs() < f32::EPSILON,
        "bounded input must make the full computed scroll range reachable without overshoot"
    );
}

#[test]
fn gallery_cards_are_non_shrinking_and_require_scroll_at_1080p() {
    let _guard = bevy_app_test_guard();
    let mut app = visual_asset_gallery_app();
    let world = app.world_mut();
    let mut roots =
        world.query_filtered::<&Node, With<babylon_client::visual_assets::GalleryScrollRoot>>();
    let root_node = roots
        .single(world)
        .expect("exactly one gallery scroll root");
    assert_eq!(root_node.overflow.y, OverflowAxis::Scroll);

    let mut labels =
        world.query_filtered::<Entity, With<babylon_client::visual_assets::GalleryAssetLabel>>();
    let label_entities: [Entity; 16] = labels
        .iter(world)
        .take(17)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|entities: Vec<Entity>| {
            panic!(
                "gallery must contain exactly 16 labeled cards, found {}",
                entities.len()
            )
        });

    let mut minimum_preview_height = 0.0;
    for label_entity in label_entities {
        let card_entity = world
            .entity(label_entity)
            .get::<ChildOf>()
            .expect("gallery label must be parented to its card")
            .parent();
        let card_node = world
            .entity(card_entity)
            .get::<Node>()
            .expect("gallery card must have layout");
        assert!(
            card_node.flex_shrink.abs() < f32::EPSILON,
            "gallery card {card_entity:?} must not shrink away its scroll range"
        );
        let children = world
            .entity(card_entity)
            .get::<Children>()
            .expect("gallery card must retain its image and label children");
        let image_entity = children
            .iter()
            .take(3)
            .find(|entity| world.entity(*entity).contains::<ImageNode>())
            .expect("gallery card must contain an image preview");
        let image_layout = world
            .entity(image_entity)
            .get::<Node>()
            .expect("gallery image preview must have layout");
        let Val::Px(width) = image_layout.width else {
            panic!("gallery image preview width must be an explicit pixel value");
        };
        let aspect_ratio = image_layout
            .aspect_ratio
            .expect("gallery image preview must preserve its aspect ratio");
        minimum_preview_height += width / aspect_ratio;
    }

    assert!(
        minimum_preview_height > 1080.0,
        "non-shrinking image previews total only {minimum_preview_height}px; a 1080p gallery must have positive overflow"
    );
}

#[test]
#[should_panic(
    expected = "gallery scroll-root singleton invariant violated: No entities fit the query"
)]
fn gallery_scroll_rejects_a_missing_scroll_root() {
    let _guard = bevy_app_test_guard();
    let mut app = visual_asset_gallery_app();
    let root = {
        let world = app.world_mut();
        let mut query = world
            .query_filtered::<Entity, With<babylon_client::visual_assets::GalleryScrollRoot>>();
        query
            .single(world)
            .expect("exactly one gallery scroll root")
    };
    assert!(
        app.world_mut().despawn(root),
        "gallery scroll root must still exist"
    );
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -1.0),
    });

    app.update();
}

#[test]
#[should_panic(
    expected = "gallery scroll-root singleton invariant violated: Multiple entities fit the query"
)]
fn gallery_scroll_rejects_multiple_scroll_roots() {
    let _guard = bevy_app_test_guard();
    let mut app = visual_asset_gallery_app();
    app.world_mut().spawn((
        ScrollPosition::default(),
        ComputedNode::DEFAULT,
        babylon_client::visual_assets::GalleryScrollRoot,
    ));
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, -1.0),
    });

    app.update();
}
