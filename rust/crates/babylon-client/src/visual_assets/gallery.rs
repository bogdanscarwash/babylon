//! Inspectable gallery presentation for every embedded visual asset.

use super::{VisualAssetDescriptor, VisualAssets, VISUAL_ASSET_CATALOG};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

/// Marks a readable label on a visual-asset gallery card.
#[derive(Component)]
pub struct GalleryAssetLabel;

/// Identifies the gallery root that receives vertical mouse-wheel input.
#[derive(Component)]
pub struct GalleryScrollRoot;

/// Registers the standalone visual-asset gallery.
#[derive(Default)]
pub struct VisualAssetGalleryPlugin;

impl Plugin for VisualAssetGalleryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_gallery)
            .add_systems(Update, update_gallery_scroll);
    }
}

/// Spawns a vertically scrollable, labeled card for each fixed catalog entry.
fn spawn_gallery(mut commands: Commands, assets: Res<VisualAssets>) {
    commands.spawn(Camera2d);
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(16),
                padding: UiRect::all(px(24)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(crate::palette::FIELD),
            GalleryScrollRoot,
            ScrollPosition::default(),
        ))
        .with_children(|root| {
            for entry in VISUAL_ASSET_CATALOG {
                let (width, aspect_ratio) = preview_dimensions(&entry);
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_shrink: 0.0,
                        align_items: AlignItems::Center,
                        row_gap: px(8),
                        padding: UiRect::all(px(12)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(crate::palette::MUTED_DARK),
                    BorderColor::all(crate::palette::GOLD),
                ))
                .with_children(|card| {
                    card.spawn((
                        ImageNode::new(assets.image(entry.id)),
                        Node {
                            width,
                            aspect_ratio: Some(aspect_ratio),
                            ..default()
                        },
                    ));
                    card.spawn((
                        Text::new(entry.label),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(crate::palette::BONE),
                        GalleryAssetLabel,
                    ));
                });
            }
        });
}

/// Applies bounded mouse-wheel input to the gallery's vertical scroll position.
fn update_gallery_scroll(
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    mut root: Query<(&mut ScrollPosition, &ComputedNode), With<GalleryScrollRoot>>,
) {
    let Some(mouse_scroll) = mouse_scroll else {
        return;
    };
    let (mut scroll_position, computed) = root.single_mut().unwrap_or_else(|error| {
        panic!("gallery scroll-root singleton invariant violated: {error}")
    });
    let delta_y = match mouse_scroll.unit {
        MouseScrollUnit::Line => mouse_scroll.delta.y * 20.0,
        MouseScrollUnit::Pixel => mouse_scroll.delta.y,
    };
    let range =
        (computed.content_size().y - computed.size().y).max(0.0) * computed.inverse_scale_factor();

    scroll_position.y = (scroll_position.y - delta_y).clamp(0.0, range);
}

/// Returns a bounded card preview size while preserving each source aspect ratio.
fn preview_dimensions(entry: &VisualAssetDescriptor) -> (Val, f32) {
    match (entry.width, entry.height) {
        (768, 192) => (px(512), 4.0),
        (512, 512) | (256, 256) | (1024, 1024) => (px(256), 1.0),
        (384, 256) => (px(256), 1.5),
        (256, 64) => (px(256), 4.0),
        (384, 128) => (px(256), 3.0),
        (1536, 864) => (px(640), 16.0 / 9.0),
        (1536, 384) => (px(640), 4.0),
        _ => panic!("no gallery dimensions declared for a catalog asset"),
    }
}
