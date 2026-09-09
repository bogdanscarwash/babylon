//! Production title and story-banner presentation backed by embedded assets.

use super::VisualAssets;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use bevy::prelude::*;

/// Marks the Babylon title image.
#[derive(Component)]
pub struct TitleMark;

/// Marks the independently readable Babylon title text.
#[derive(Component)]
pub struct ReadableTitle;

/// Marks the currently selected story's banner image.
#[derive(Component)]
pub struct StoryBanner;

/// Spawns the title lockup and keeps its image and readable text independent.
fn spawn_title_lockup(mut commands: Commands, assets: Res<VisualAssets>) {
    commands.spawn((
        ImageNode::new(assets.title_mark.clone()),
        Node {
            position_type: PositionType::Absolute,
            top: px(20),
            left: px(24),
            width: px(144),
            height: px(36),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TitleLockup),
        TitleMark,
    ));
    commands.spawn((
        Text::new("BABYLON"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(crate::palette::GOLD),
        Node {
            position_type: PositionType::Absolute,
            top: px(58),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TitleLockup),
        ReadableTitle,
    ));
}

/// Returns the embedded banner declared for `story`.
#[must_use]
fn story_banner(assets: &VisualAssets, story: &crate::story::Story) -> Handle<Image> {
    match story.id {
        "counties" => assets.banner_counties.clone(),
        "carceral" => assets.banner_carceral.clone(),
        unknown => panic!("no visual banner declared for story {unknown:?}"),
    }
}

/// Spawns the selected story's banner without covering the map afterward.
fn spawn_story_banner(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    selected_story: Res<crate::story::SelectedStory>,
    story_card_visible: Res<crate::ui::story_card::StoryCardVisible>,
) {
    let visibility = if story_card_visible.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    commands.spawn((
        ImageNode::new(story_banner(&assets, selected_story.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            right: px(24),
            width: px(480),
            height: px(120),
            ..default()
        },
        visibility,
        GlobalZIndex(-1),
        DeclaredSurface::new(SurfaceId::StoryBanner),
        StoryBanner,
    ));
}

/// Keeps the banner's story image and visibility aligned with presentation state.
fn sync_story_banner(
    assets: Res<VisualAssets>,
    selected_story: Res<crate::story::SelectedStory>,
    story_card_visible: Res<crate::ui::story_card::StoryCardVisible>,
    mut banners: Query<(&mut ImageNode, &mut Visibility), With<StoryBanner>>,
) {
    let (mut image, mut visibility) = banners
        .single_mut()
        .unwrap_or_else(|error| panic!("story banner singleton invariant violated: {error}"));
    if selected_story.is_changed() {
        image.image = story_banner(&assets, selected_story.0);
    }
    *visibility = if story_card_visible.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// Registers the production title lockup and card-gated story banner.
#[derive(Default)]
pub struct VisualPresentationPlugin;

impl Plugin for VisualPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_title_lockup, spawn_story_banner));
        app.add_systems(
            Update,
            sync_story_banner
                .after(crate::ui::story_card::dismiss_story_card_on_first_advance)
                .after(crate::ui::story_card::recall_story_card_on_question_mark)
                .after(crate::ui::story_card::restart_on_n_key),
        );
    }
}
