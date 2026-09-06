//! The observer's geographic scene: actual county polygons, extruded solely as
//! a visual encoding of the selected observation. Heights are not terrain or
//! simulated buildings. All economic bytes come from the installed frame.

use bevy::asset::RenderAssetUsages;
use bevy::camera::{visibility::RenderLayers, Viewport};
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::light::CascadeShadowConfigBuilder;
use bevy::mesh::PrimitiveTopology;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::map::{HoveredCounty, MapBounds, SelectedCounty};
use crate::map_economy_lens::{project_map_lens, MapLens};
use crate::observer::ObserverSession;
use crate::observer_focus::ObserverKeyboardClaim;
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{grouped, ObserverFrame, ObserverUiState, ObserverViewport};
use crate::production::PrimaryView;

#[path = "map_relationships.rs"]
mod relationships;

const MAP_LAYER: usize = 2;
const METRES_TO_SCENE: f32 = 0.001;
const BASE_HEIGHT: f32 = 3.0;
const DATA_HEIGHT: f32 = 125.0;

pub(crate) const MAP_VIEW_HELP: &str = "Relationships keeps county slabs level. Dashed arrows are schematic dependencies, not physical route geometry. Economic lenses compare county readings by height; gray means unavailable or not modeled, while zero is a measured account. Heights never locate terrain, buildings or factories.\nRight-drag orbit | middle-drag pan | wheel / +/- zoom | Home reset";

/// Identifies the observer's perspective map camera, separate from production.
#[derive(Component)]
pub struct ObserverMapCamera;

#[derive(Component)]
struct CountySlab {
    atlas_index: usize,
    fips: String,
    outline: Handle<StandardMaterial>,
}

#[derive(Component)]
struct MapLegend;

#[derive(Resource)]
struct MichiganMapGeometry {
    extent: Vec2,
}

/// Camera motion is direct, without inertia or automatic drift. This also
/// keeps reduced-motion mode free of incidental movement.
#[derive(Resource)]
struct MapOrbit {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    fitted_distance: f32,
}

impl MapOrbit {
    fn new(span: f32) -> Self {
        let distance = span * 1.7;
        Self {
            target: Vec3::new(0.0, DATA_HEIGHT * 0.25, 0.0),
            yaw: -0.2,
            pitch: 0.88,
            distance,
            fitted_distance: distance,
        }
    }

    fn transform(&self) -> Transform {
        let offset = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ) * self.distance;
        Transform::from_translation(self.target + offset).looking_at(self.target, Vec3::Y)
    }
}

#[derive(SystemParam)]
struct MapObservation<'w> {
    frame: Res<'w, ObserverFrame>,
    session: Res<'w, ObserverSession>,
    ui: Res<'w, ObserverUiState>,
    view: Res<'w, PrimaryView>,
    selected: Res<'w, SelectedCounty>,
    hovered: Res<'w, HoveredCounty>,
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn height(value: Option<u64>, maximum: Option<u64>) -> f32 {
    let ratio = match (value, maximum) {
        (Some(value), Some(maximum)) if maximum > 0 => {
            (value as f64 / maximum as f64).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };
    BASE_HEIGHT + DATA_HEIGHT * ratio as f32
}

fn scene_point(point: Vec2, origin: Vec2, y: f32) -> Vec3 {
    let point = (point - origin) * METRES_TO_SCENE;
    Vec3::new(point.x, y, -point.y)
}

fn push_triangle(positions: &mut Vec<[f32; 3]>, normals: &mut Vec<[f32; 3]>, points: [Vec3; 3]) {
    let normal = (points[1] - points[0]).cross(points[2] - points[0]);
    if let Some(normal) = normal.try_normalize() {
        positions.extend(points.map(|point| point.to_array()));
        normals.extend([normal.to_array(); 3]);
    }
}

/// Build one closed prism from the atlas's existing triangulation and every
/// exterior/hole boundary. No bounding-box approximation or invented site.
fn county_prism(
    atlas: &CountyAtlas,
    triangles: &crate::tessellate::Tessellation,
    index: usize,
    origin: Vec2,
) -> (Mesh, Mesh) {
    let county = atlas.county(index).expect("atlas index was enumerated");
    let (start, end) = triangles.county_vertex_range[index];
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let first = triangles.indices.partition_point(|index| *index < start);
    let last = triangles.indices.partition_point(|index| *index < end);
    for triangle in triangles.indices[first..last].chunks_exact(3) {
        let mut top = [triangle[0], triangle[1], triangle[2]].map(|index| {
            let point = triangles.positions[index as usize];
            scene_point(Vec2::new(point[0], point[1]), origin, 1.0)
        });
        if (top[1] - top[0]).cross(top[2] - top[0]).y < 0.0 {
            top.swap(1, 2);
        }
        push_triangle(&mut positions, &mut normals, top);
        push_triangle(
            &mut positions,
            &mut normals,
            [
                Vec3::new(top[0].x, 0.0, top[0].z),
                Vec3::new(top[2].x, 0.0, top[2].z),
                Vec3::new(top[1].x, 0.0, top[1].z),
            ],
        );
    }
    let mut edges = Vec::new();
    for ring in county.rings {
        let start = ring.vertex_start as usize;
        let points = &atlas.vertices()[start..start + ring.vertex_count as usize];
        let signed_area: f64 = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(a, b)| f64::from(a.x) * f64::from(b.y) - f64::from(b.x) * f64::from(a.y))
            .sum();
        let forward = (signed_area > 0.0) != ring.is_hole;
        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
            let a0 = scene_point(*a, origin, 0.0);
            let b0 = scene_point(*b, origin, 0.0);
            let a1 = scene_point(*a, origin, 1.0);
            let b1 = scene_point(*b, origin, 1.0);
            let faces = if forward {
                [[a0, b0, b1], [a0, b1, a1]]
            } else {
                [[a0, b1, b0], [a0, a1, b1]]
            };
            for face in faces {
                push_triangle(&mut positions, &mut normals, face);
            }
            edges.extend([
                scene_point(*a, origin, 1.004).to_array(),
                scene_point(*b, origin, 1.004).to_array(),
            ]);
        }
    }
    let mut body = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    body.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    body.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    let mut outline = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
    outline.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; edges.len()]);
    outline.insert_attribute(Mesh::ATTRIBUTE_POSITION, edges);
    (body, outline)
}

fn setup_map(
    mut commands: Commands,
    atlas: Res<CountyAtlas>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let counties: Vec<_> = (0..atlas.len())
        .filter(|index| {
            atlas
                .county(*index)
                .is_some_and(|county| county.fips.starts_with("26"))
        })
        .collect();
    assert_eq!(
        counties.len(),
        83,
        "Michigan geography must contain all 83 counties"
    );
    let (min, max) = counties
        .iter()
        .filter_map(|index| atlas.county(*index))
        .map(|county| (county.bbox.min, county.bbox.max))
        .reduce(|(a, b), (c, d)| (a.min(c), b.max(d)))
        .expect("Michigan has geography");
    let origin = (min + max) * 0.5;
    let extent = (max - min) * METRES_TO_SCENE;
    let mut diagonals: Vec<_> = counties
        .iter()
        .filter_map(|index| atlas.county(*index))
        .map(|county| (county.bbox.max - county.bbox.min).length())
        .collect();
    diagonals.sort_by(f32::total_cmp);
    commands.insert_resource(MapBounds {
        world_bounds: Rect { min, max },
        median_county_diagonal: diagonals[diagonals.len() / 2],
    });
    let triangles = crate::tessellate::tessellate(&atlas);
    commands.insert_resource(relationships::CountyAnchors::from_atlas(
        &atlas, &counties, origin,
    ));
    for index in counties {
        let county = atlas.county(index).expect("Michigan index exists");
        let (body, edge) = county_prism(&atlas, &triangles, index, origin);
        let outline = materials.add(StandardMaterial {
            base_color: theme::INK,
            unlit: true,
            ..default()
        });
        commands
            .spawn((
                Mesh3d(meshes.add(body)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: theme::LAND,
                    perceptual_roughness: 0.78,
                    metallic: 0.08,
                    ..default()
                })),
                Transform::from_scale(Vec3::new(1.0, BASE_HEIGHT, 1.0)),
                RenderLayers::layer(MAP_LAYER),
                CountySlab {
                    atlas_index: index,
                    fips: county.fips.to_owned(),
                    outline: outline.clone(),
                },
                DeclaredSurface::new(SurfaceId::ObserverShell),
            ))
            .with_child((
                Mesh3d(meshes.add(edge)),
                MeshMaterial3d(outline),
                RenderLayers::layer(MAP_LAYER),
                Pickable::IGNORE,
                DeclaredSurface::new(SurfaceId::ObserverShell),
            ))
            .observe(hover_county)
            .observe(leave_county)
            .observe(select_county);
    }
    spawn_map_scene(&mut commands, extent, &mut meshes, &mut materials);
}

fn spawn_map_scene(
    commands: &mut Commands,
    extent: Vec2,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let plinth = materials.add(StandardMaterial {
        base_color: theme::INK,
        perceptual_roughness: 0.94,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(extent.x + 60.0, 10.0, extent.y + 60.0))),
        MeshMaterial3d(plinth),
        Transform::from_xyz(0.0, -5.1, 0.0),
        RenderLayers::layer(MAP_LAYER),
        Pickable::IGNORE,
        DeclaredSurface::new(SurfaceId::ObserverShell),
    ));
    commands.spawn((
        DirectionalLight {
            color: theme::PAPER,
            illuminance: 9000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-400.0, 800.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: extent.max_element() * 0.5,
            maximum_distance: extent.max_element() * 6.0,
            ..default()
        }
        .build(),
        RenderLayers::layer(MAP_LAYER),
    ));
    commands.spawn((
        DirectionalLight {
            color: theme::BLUE,
            illuminance: 1800.0,
            ..default()
        },
        Transform::from_xyz(500.0, 300.0, -300.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(MAP_LAYER),
    ));
    let orbit = MapOrbit::new(extent.max_element());
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(theme::INK),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            near: 0.5,
            far: 12_000.0,
            ..default()
        }),
        orbit.transform(),
        RenderLayers::layer(MAP_LAYER),
        ObserverMapCamera,
        Msaa::Sample4,
    ));
    commands.insert_resource(orbit);
    commands.insert_resource(MichiganMapGeometry { extent });
    commands.spawn((
        Text::new("Michigan | 83 counties | geographic boundaries"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(theme::PAPER),
        crate::observer_ui::ObserverFontRole::Body,
        Node {
            position_type: PositionType::Absolute,
            max_width: px(700),
            padding: UiRect::axes(px(10), px(6)),
            ..default()
        },
        BackgroundColor(theme::INK),
        ZIndex(4),
        Pickable::IGNORE,
        Visibility::Hidden,
        MapLegend,
        DeclaredSurface::new(SurfaceId::ObserverShell),
    ));
}

fn hover_county(
    over: On<Pointer<Over>>,
    counties: Query<&CountySlab>,
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    mut hovered: ResMut<HoveredCounty>,
) {
    if *view != PrimaryView::Map || ui.menu_open || ui.splash_visible || ui.comparison_open {
        return;
    }
    if let Ok(county) = counties.get(over.entity) {
        if hovered.0 != Some(county.atlas_index) {
            hovered.0 = Some(county.atlas_index);
        }
    }
}
fn leave_county(
    out: On<Pointer<Out>>,
    counties: Query<&CountySlab>,
    mut hovered: ResMut<HoveredCounty>,
) {
    if counties
        .get(out.entity)
        .is_ok_and(|county| hovered.0 == Some(county.atlas_index))
    {
        hovered.0 = None;
    }
}
fn select_county(
    click: On<Pointer<Click>>,
    counties: Query<&CountySlab>,
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    mut selected: ResMut<SelectedCounty>,
) {
    if click.button != PointerButton::Primary
        || *view != PrimaryView::Map
        || ui.menu_open
        || ui.splash_visible
        || ui.comparison_open
    {
        return;
    }
    if let Ok(county) = counties.get(click.entity) {
        if selected.0 != Some(county.atlas_index) {
            selected.0 = Some(county.atlas_index);
        }
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn update_observation(
    input: MapObservation,
    mut counties: Query<(
        &CountySlab,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut legend: Query<&mut Text, With<MapLegend>>,
    mut last_context: Local<Option<(crate::observer::ObservationContext, MapLens)>>,
) {
    let context = (input.session.context(), input.ui.lens.clone());
    let data_changed = input.frame.is_changed() || last_context.as_ref() != Some(&context);
    let outline_changed = input.selected.is_changed() || input.hovered.is_changed();
    if !(data_changed || outline_changed || input.view.is_changed() || input.ui.is_changed()) {
        return;
    }
    if data_changed {
        *last_context = Some(context);
    }
    let frame = input.frame.for_session(&input.session);
    let projection = project_map_lens(frame, &input.ui.lens);
    let maximum = projection.maximum();
    for (county, mut transform, material) in &mut counties {
        let value = projection.county(&county.fips).value();
        let target_height = height(value, maximum);
        if data_changed && transform.scale.y.ne(&target_height) {
            transform.scale.y = target_height;
        }
        let color = if matches!(input.ui.lens, MapLens::Relationships) {
            theme::LAND
        } else {
            value.map_or(theme::GRAY, |value| {
                let weight = maximum
                    .filter(|maximum| *maximum > 0)
                    .map_or(0.0, |maximum| {
                        (value as f64 / maximum as f64).clamp(0.0, 1.0) as f32
                    });
                theme::BLUE.mix(&theme::YELLOW, weight)
            })
        };
        if data_changed {
            if let Some(material) = materials.get_mut(&material.0) {
                material.base_color = color;
            }
        }
        if let Some(outline) = materials.get_mut(&county.outline) {
            outline.base_color = if input.selected.0 == Some(county.atlas_index) {
                theme::PAPER
            } else if input.hovered.0 == Some(county.atlas_index) {
                theme::BLUE
            } else {
                theme::INK
            };
        }
    }
    for mut text in &mut legend {
        if matches!(input.ui.lens, MapLens::Relationships) {
            text.set_if_neq(Text::new(
                "County geography | schematic supply relationships",
            ));
            continue;
        }
        text.set_if_neq(Text::new(format!(
            "{} | week {} | 0..{} {}\nCounty readings | gray: unavailable | controls and encoding: Lenses",
            projection.label, input.session.viewed_tick, maximum.map_or_else(|| "-".into(), grouped), projection.unit,
        )));
    }
}

fn legend_rect(world: Rect, measured: Vec2) -> Option<Rect> {
    let inner = Rect::from_corners(world.min + Vec2::splat(8.0), world.max - Vec2::splat(8.0));
    if !measured.is_finite()
        || measured.min_element() <= 0.0
        || measured.x > inner.width()
        || measured.y > inner.height()
    {
        return None;
    }
    let min = Vec2::new(inner.min.x, inner.max.y - measured.y);
    Some(Rect::from_corners(min, min + measured))
}

fn place_legend(
    view: Res<PrimaryView>,
    ui: Res<ObserverUiState>,
    viewport: Res<ObserverViewport>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut legends: Query<(&ComputedNode, &mut Node, &mut Visibility), With<MapLegend>>,
) {
    let active = *view == PrimaryView::Map
        && !ui.menu_open
        && !ui.splash_visible
        && !ui.comparison_open
        && ui.disclosure.is_none()
        && !matches!(ui.lens, MapLens::Relationships);
    for (computed, mut node, mut visibility) in &mut legends {
        let placement = viewport
            .0
            .zip(windows.single().ok())
            .and_then(|(world, window)| {
                let max_width = px(((world.width() - 16.0) / scale.0).max(0.0));
                if node.max_width != max_width {
                    node.max_width = max_width;
                }
                active
                    .then(|| legend_rect(world, computed.size() / window.scale_factor()))
                    .flatten()
            });
        visibility.set_if_neq(if placement.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if let Some(rect) = placement {
            let (left, top) = (px(rect.min.x / scale.0), px(rect.min.y / scale.0));
            if node.left != left || node.top != top {
                node.left = left;
                node.top = top;
            }
        }
    }
}

#[derive(SystemParam)]
struct NavigationInput<'w, 's> {
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    viewport: Res<'w, ObserverViewport>,
    ui: Res<'w, ObserverUiState>,
    view: Res<'w, PrimaryView>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    keyboard_claim: Option<Res<'w, ObserverKeyboardClaim>>,
    motion: Res<'w, AccumulatedMouseMotion>,
    scroll: Res<'w, AccumulatedMouseScroll>,
    time: Res<'w, Time>,
    buttons: Query<'w, 's, &'static Interaction, With<Button>>,
}

fn navigate(
    input: NavigationInput,
    geometry: Res<MichiganMapGeometry>,
    mut orbit: ResMut<MapOrbit>,
) {
    if *input.view != PrimaryView::Map
        || input.ui.menu_open
        || input.ui.splash_visible
        || input.ui.comparison_open
        || input.ui.disclosure.is_some()
    {
        return;
    }
    let Ok(window) = input.windows.single() else {
        return;
    };
    let over_map = window
        .cursor_position()
        .zip(input.viewport.0)
        .is_some_and(|(cursor, rect)| rect.contains(cursor))
        && !input
            .buttons
            .iter()
            .any(|interaction| *interaction != Interaction::None);
    let dt = input.time.delta_secs().min(0.1);
    if !input
        .keyboard_claim
        .as_ref()
        .is_some_and(|claim| claim.blocks_world_shortcuts())
        && (input.keys.just_pressed(KeyCode::Home)
            || [
                KeyCode::KeyW,
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::Equal,
                KeyCode::Minus,
            ]
            .iter()
            .any(|key| input.keys.pressed(*key)))
    {
        keyboard_orbit(&input.keys, dt, &geometry, &mut orbit);
    }
    if over_map {
        if input.mouse.pressed(MouseButton::Right) && input.motion.delta != Vec2::ZERO {
            orbit.yaw -= input.motion.delta.x * 0.004;
            orbit.pitch = (orbit.pitch + input.motion.delta.y * 0.004).clamp(0.25, 1.45);
        }
        if input.mouse.pressed(MouseButton::Middle) && input.motion.delta != Vec2::ZERO {
            let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin());
            let north = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos());
            let movement = (right * -input.motion.delta.x + north * input.motion.delta.y)
                * orbit.distance
                * 0.0008;
            orbit.target += movement;
        }
        if input.scroll.delta.y != 0.0 {
            let unit = if input.scroll.unit == MouseScrollUnit::Line {
                0.12
            } else {
                0.002
            };
            orbit.distance *= (-input.scroll.delta.y * unit).exp();
        }
    }
    let min_distance = orbit.fitted_distance * 0.08;
    let max_distance = orbit.fitted_distance * 3.0;
    let bounded_distance = orbit.distance.clamp(min_distance, max_distance);
    let bounded_target = Vec3::new(
        orbit.target.x.clamp(-geometry.extent.x, geometry.extent.x),
        orbit.target.y,
        orbit.target.z.clamp(-geometry.extent.y, geometry.extent.y),
    );
    if orbit.distance.ne(&bounded_distance) {
        orbit.distance = bounded_distance;
    }
    if orbit.target != bounded_target {
        orbit.target = bounded_target;
    }
}

fn keyboard_orbit(
    keys: &ButtonInput<KeyCode>,
    dt: f32,
    geometry: &MichiganMapGeometry,
    orbit: &mut MapOrbit,
) {
    let yaw_key = if keys.pressed(KeyCode::KeyD) {
        1.0
    } else {
        0.0
    } - if keys.pressed(KeyCode::KeyA) {
        1.0
    } else {
        0.0
    };
    let pitch_key = if keys.pressed(KeyCode::KeyW) {
        1.0
    } else {
        0.0
    } - if keys.pressed(KeyCode::KeyS) {
        1.0
    } else {
        0.0
    };
    if yaw_key != 0.0 || pitch_key != 0.0 {
        orbit.yaw += yaw_key * dt;
        orbit.pitch = (orbit.pitch + pitch_key * dt).clamp(0.25, 1.45);
    }
    if keys.just_pressed(KeyCode::Home) {
        *orbit = MapOrbit::new(geometry.extent.max_element());
    }
    if keys.pressed(KeyCode::Equal) {
        orbit.distance *= (-dt).exp();
    }
    if keys.pressed(KeyCode::Minus) {
        orbit.distance *= dt.exp();
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn sync_camera(
    orbit: Res<MapOrbit>,
    view: Res<PrimaryView>,
    viewport: Res<ObserverViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&mut Camera, &mut Transform), With<ObserverMapCamera>>,
) {
    let Ok((mut camera, mut transform)) = cameras.single_mut() else {
        return;
    };
    let active = *view == PrimaryView::Map;
    if camera.is_active != active {
        camera.is_active = active;
    }
    if orbit.is_changed() {
        transform.set_if_neq(orbit.transform());
    }
    if let (Some(rect), Ok(window)) = (viewport.0, windows.single()) {
        let value = Viewport {
            physical_position: (rect.min * window.scale_factor()).as_uvec2(),
            physical_size: (rect.size() * window.scale_factor()).as_uvec2(),
            ..default()
        };
        if camera.viewport.as_ref().is_none_or(|old| {
            old.physical_position != value.physical_position
                || old.physical_size != value.physical_size
        }) {
            camera.viewport = Some(value);
        }
    }
}

pub struct ObserverMap3dPlugin;
impl Plugin for ObserverMap3dPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MeshPickingPlugin>() {
            app.add_plugins(MeshPickingPlugin);
        }
        app.init_asset::<StandardMaterial>()
            .add_systems(Startup, setup_map.after(crate::map::spawn_map_surface))
            .add_systems(Update, navigate.in_set(ObserverSet::Input))
            .add_systems(Update, update_observation.in_set(ObserverSet::Paint))
            .add_systems(
                Update,
                (sync_camera, place_legend).after(ObserverSet::Paint),
            );
        relationships::install(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::Perspective;
    use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusTarget, ObserverFocusWorld};
    use babylon_persistence::{ObserverEconomySnapshotV1, ObserverVisibilityV1};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
    use bevy::input::{ButtonState, InputPlugin};
    use bevy::input_focus::tab_navigation::TabGroup;
    use bevy::input_focus::InputFocus;
    use bevy::mesh::VertexAttributeValues;
    use bevy::time::TimeUpdateStrategy;

    fn focused_navigation_app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin, ObserverFocusPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(16),
            ))
            .insert_resource(PrimaryView::Map)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(ObserverViewport(Some(Rect::from_corners(
                Vec2::ZERO,
                Vec2::splat(500.0),
            ))))
            .insert_resource(MichiganMapGeometry {
                extent: Vec2::splat(200.0),
            })
            .insert_resource(MapOrbit::new(200.0))
            .add_systems(Update, navigate);
        let mut window = Window::default();
        window.set_cursor_position(Some(Vec2::splat(100.0)));
        let window = app.world_mut().spawn((window, PrimaryWindow)).id();
        let root = app
            .world_mut()
            .spawn((Node::default(), TabGroup::new(0)))
            .id();
        let mut target = ObserverFocusTarget::reading(None);
        target.available = true;
        let reading = app.world_mut().spawn((Node::default(), target)).id();
        app.world_mut().entity_mut(root).add_child(reading);
        app.update();
        queue_camera_key(&mut app, window, KeyCode::Tab, ButtonState::Pressed);
        app.update();
        queue_camera_key(&mut app, window, KeyCode::Tab, ButtonState::Released);
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
        assert!(app
            .world()
            .resource::<ObserverKeyboardClaim>()
            .blocks_world_shortcuts());
        (app, window, reading)
    }

    fn queue_camera_key(app: &mut App, window: Entity, key_code: KeyCode, state: ButtonState) {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window,
        });
    }

    #[test]
    fn focused_reading_blocks_each_camera_key_until_world_focus_returns() {
        for key in [
            KeyCode::Home,
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyS,
            KeyCode::KeyD,
            KeyCode::Equal,
            KeyCode::Minus,
        ] {
            let (mut app, window, reading) = focused_navigation_app();
            app.world_mut().resource_mut::<MapOrbit>().yaw = 0.4;
            let before = app.world().resource::<MapOrbit>().transform();
            queue_camera_key(&mut app, window, key, ButtonState::Pressed);
            app.update();
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
            assert_eq!(
                app.world().resource::<MapOrbit>().transform(),
                before,
                "{key:?}"
            );
            queue_camera_key(&mut app, window, key, ButtonState::Released);
            app.update();
            app.world_mut().trigger(ObserverFocusWorld);
            app.update();
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
            queue_camera_key(&mut app, window, key, ButtonState::Pressed);
            app.update();
            assert_ne!(
                app.world().resource::<MapOrbit>().transform(),
                before,
                "{key:?}"
            );
        }
    }

    #[test]
    fn first_tab_claims_same_frame_camera_keys_before_navigation() {
        let (mut app, window, reading) = focused_navigation_app();
        app.world_mut().trigger(ObserverFocusWorld);
        app.update();
        app.world_mut().resource_mut::<MapOrbit>().yaw = 0.4;
        let before = app.world().resource::<MapOrbit>().transform();
        for key in [KeyCode::Tab, KeyCode::Home, KeyCode::KeyD] {
            queue_camera_key(&mut app, window, key, ButtonState::Pressed);
        }
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
        assert_eq!(app.world().resource::<MapOrbit>().transform(), before);
    }

    #[test]
    fn focused_reading_preserves_pointer_orbit_pan_and_wheel_over_map() {
        for gesture in [Some(MouseButton::Right), Some(MouseButton::Middle), None] {
            let (mut app, _, reading) = focused_navigation_app();
            let before = app.world().resource::<MapOrbit>().transform();
            if let Some(button) = gesture {
                app.world_mut()
                    .resource_mut::<ButtonInput<MouseButton>>()
                    .press(button);
                app.world_mut()
                    .resource_mut::<AccumulatedMouseMotion>()
                    .delta = Vec2::new(40.0, 10.0);
            } else {
                app.world_mut()
                    .resource_mut::<AccumulatedMouseScroll>()
                    .delta = Vec2::Y;
            }
            // Exercise the same navigator using Bevy's accumulated pointer
            // input while the actual focused-reading claim remains active.
            app.world_mut().run_system_once(navigate).unwrap();
            assert_ne!(app.world().resource::<MapOrbit>().transform(), before);
            assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
            assert!(app
                .world()
                .resource::<ObserverKeyboardClaim>()
                .blocks_world_shortcuts());
        }
    }

    #[derive(Resource, Default)]
    struct OrbitChanged(bool);

    fn record_orbit_change(orbit: Res<MapOrbit>, mut changed: ResMut<OrbitChanged>) {
        changed.0 = orbit.is_changed();
    }

    #[test]
    fn idle_camera_does_not_publish_changes_with_reading_or_world_focus() {
        let (mut app, _, _) = focused_navigation_app();
        app.init_resource::<OrbitChanged>()
            .add_systems(Last, record_orbit_change);
        app.update();
        app.update();
        assert!(!app.world().resource::<OrbitChanged>().0);
        app.world_mut().trigger(ObserverFocusWorld);
        app.update();
        app.update();
        assert!(!app.world().resource::<OrbitChanged>().0);
    }

    #[test]
    fn legend_stays_inside_measured_world_across_layout_and_scope_changes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PrimaryView::Map)
            .insert_resource(ObserverUiState {
                splash_visible: false,
                menu_open: false,
                lens: MapLens::Qcew(crate::map_economy_lens::EconomyMetric::Employment),
                ..default()
            })
            .init_resource::<ObserverViewport>()
            .init_resource::<UiScale>()
            .add_systems(Update, place_legend);
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        let legend = app
            .world_mut()
            .spawn((
                MapLegend,
                ComputedNode::default(),
                Node::default(),
                Visibility::Hidden,
            ))
            .id();
        for window_size in [Vec2::new(1366.0, 768.0), Vec2::new(1920.0, 1080.0)] {
            for scale in [1.0, 1.15, 1.3] {
                for history_open in [false, true] {
                    let layout = crate::observer_layout::ObserverLayout::new(
                        window_size,
                        scale,
                        history_open,
                    );
                    let world =
                        Rect::from_corners(layout.world.min * scale, layout.world.max * scale);
                    app.world_mut().resource_mut::<ObserverViewport>().0 = Some(world);
                    app.world_mut().resource_mut::<UiScale>().0 = scale;
                    app.world_mut()
                        .resource_mut::<ObserverUiState>()
                        .history_open = history_open;
                    let measured = Vec2::new(500.0, 36.0) * scale;
                    app.world_mut()
                        .get_mut::<ComputedNode>(legend)
                        .unwrap()
                        .size = measured;
                    app.update();
                    let node = app.world().get::<Node>(legend).unwrap();
                    let (Val::Px(left), Val::Px(top)) = (node.left, node.top) else {
                        panic!("legend placement uses measured pixel coordinates");
                    };
                    let min = Vec2::new(left, top) * scale;
                    assert!(min.cmpge(world.min).all());
                    assert!((min + measured).cmple(world.max).all());
                    assert_eq!(
                        *app.world().get::<Visibility>(legend).unwrap(),
                        Visibility::Visible
                    );
                }
            }
        }
        app.world_mut().resource_mut::<ObserverUiState>().lens = MapLens::Relationships;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(legend).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().resource_mut::<ObserverUiState>().lens =
            MapLens::Qcew(crate::map_economy_lens::EconomyMetric::Employment);
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(legend).unwrap(),
            Visibility::Hidden
        );
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Map;
        app.world_mut().resource_mut::<ObserverUiState>().disclosure =
            Some(crate::observer_ui::ObserverDisclosure::Lens);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(legend).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn unmeasured_or_oversized_legend_never_overflows_the_world() {
        let world = Rect::new(16.0, 100.0, 620.0, 300.0);
        for measured in [Vec2::ZERO, Vec2::new(100.0, 201.0), Vec2::new(605.0, 36.0)] {
            assert_eq!(legend_rect(world, measured), None);
        }
        let rect = legend_rect(world, Vec2::new(500.0, 36.0)).unwrap();
        assert_eq!(rect.min, Vec2::new(24.0, 256.0));
        assert_eq!(rect.max, Vec2::new(524.0, 292.0));
    }

    #[test]
    fn metric_height_is_linear_bounded_and_unknown_is_only_a_base() {
        assert_eq!(height(None, Some(10)).to_bits(), BASE_HEIGHT.to_bits());
        assert_eq!(height(Some(0), Some(0)).to_bits(), BASE_HEIGHT.to_bits());
        assert_eq!(
            height(Some(5), Some(10)).to_bits(),
            (BASE_HEIGHT + DATA_HEIGHT / 2.0).to_bits()
        );
        assert_eq!(
            height(Some(u64::MAX), Some(u64::MAX)).to_bits(),
            (BASE_HEIGHT + DATA_HEIGHT).to_bits()
        );
    }

    #[test]
    fn stale_campaign_tick_and_visibility_never_feed_geographic_heights() {
        let campaign = babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil());
        let mut session = ObserverSession::new(campaign);
        session.ready(4, Some("a".repeat(64)));
        let mut snapshot = ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 4,
            foundation_digest: "f".repeat(64),
            tick_content_hash: Some("a".repeat(64)),
            nominal_world_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: None,
        };
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_some());
        snapshot.resolve_tick = 3;
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_none());
        snapshot.resolve_tick = 4;
        snapshot.visibility = ObserverVisibilityV1::KnownPreview;
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_none());
        session.perspective = Perspective::PlayerKnowledge;
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_some());
        session.foundation_digest = Some("g".repeat(64));
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_none());
        session.foundation_digest = Some(snapshot.foundation_digest.clone());
        snapshot.tick_content_hash = Some("b".repeat(64));
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_none());
        session.viewed_tick = 3;
        snapshot.resolve_tick = 3;
        assert!(ObserverFrame(Some(snapshot.clone()))
            .for_session(&session)
            .is_some());
        snapshot.campaign_id = uuid::Uuid::from_u128(1).to_string();
        assert!(ObserverFrame(Some(snapshot))
            .for_session(&session)
            .is_none());
    }

    #[test]
    fn open_drawers_block_raw_orbit_and_wheel_until_closed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PrimaryView::Map)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .insert_resource(ObserverViewport(Some(Rect::from_corners(
                Vec2::ZERO,
                Vec2::splat(500.0),
            ))))
            .insert_resource(MichiganMapGeometry {
                extent: Vec2::splat(200.0),
            })
            .insert_resource(MapOrbit::new(200.0))
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(AccumulatedMouseMotion {
                delta: Vec2::new(40.0, 10.0),
            })
            .insert_resource(AccumulatedMouseScroll {
                unit: MouseScrollUnit::Line,
                delta: Vec2::Y,
            })
            .add_systems(Update, navigate);
        let mut window = Window::default();
        window.set_cursor_position(Some(Vec2::splat(100.0)));
        app.world_mut().spawn((window, PrimaryWindow));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        let original = app.world().resource::<MapOrbit>().transform();
        let distance = app.world().resource::<MapOrbit>().distance.to_bits();
        for drawer in [
            crate::observer_ui::ObserverDisclosure::Time,
            crate::observer_ui::ObserverDisclosure::Lens,
        ] {
            app.world_mut().resource_mut::<ObserverUiState>().disclosure = Some(drawer);
            app.update();
            assert_eq!(app.world().resource::<MapOrbit>().transform(), original);
            assert_eq!(
                app.world().resource::<MapOrbit>().distance.to_bits(),
                distance
            );
        }
        app.world_mut().resource_mut::<ObserverUiState>().disclosure = None;
        app.update();
        assert_ne!(app.world().resource::<MapOrbit>().transform(), original);
        assert_ne!(
            app.world().resource::<MapOrbit>().distance.to_bits(),
            distance
        );
    }

    #[test]
    fn native_county_click_changes_selection_only_in_the_open_map() {
        use bevy::picking::backend::HitData;
        use bevy::picking::pointer::{Location, PointerId};
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<SelectedCounty>();
        app.init_resource::<PrimaryView>();
        app.init_resource::<ObserverUiState>();
        let entity = app
            .world_mut()
            .spawn(CountySlab {
                atlas_index: 42,
                fips: "26163".into(),
                outline: Handle::default(),
            })
            .observe(select_county)
            .id();
        let click = || {
            Pointer::new(
                PointerId::Mouse,
                Location {
                    target: bevy::camera::NormalizedRenderTarget::None {
                        width: 0,
                        height: 0,
                    },
                    position: Vec2::ZERO,
                },
                Click {
                    button: PointerButton::Primary,
                    hit: HitData {
                        camera: Entity::PLACEHOLDER,
                        depth: 1.0,
                        position: Some(Vec3::ZERO),
                        normal: Some(Vec3::Y),
                    },
                    duration: std::time::Duration::ZERO,
                },
                entity,
            )
        };
        app.world_mut().trigger(click());
        assert_eq!(app.world().resource::<SelectedCounty>().0, None);
        app.world_mut().resource_mut::<ObserverUiState>().menu_open = false;
        app.world_mut().trigger(click());
        assert_eq!(
            app.world().resource::<SelectedCounty>().0,
            None,
            "splash blocks the world"
        );
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .splash_visible = false;
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .comparison_open = true;
        app.world_mut().trigger(click());
        assert_eq!(
            app.world().resource::<SelectedCounty>().0,
            None,
            "comparison blocks the world"
        );
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .comparison_open = false;
        app.world_mut().trigger(click());
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(42));
        app.world_mut().resource_mut::<SelectedCounty>().0 = None;
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        app.world_mut().trigger(click());
        assert_eq!(app.world().resource::<SelectedCounty>().0, None);
    }

    #[test]
    fn observer_map_loads_the_atlas_without_spawning_the_conformance_2d_surface() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.insert_resource(ObserverSession::new(
            babylon_persistence::CampaignId::from_uuid(uuid::Uuid::nil()),
        ));
        app.add_plugins(crate::map::MapPlugin);
        app.update();
        assert!(app.world().contains_resource::<CountyAtlas>());
        assert!(app.world().contains_resource::<crate::map::CountyIndex>());
        assert!(!app.world().contains_resource::<crate::map::MapSurface>());
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<crate::map::MapCamera>>()
                .iter(world)
                .count(),
            0
        );
        assert_eq!(
            world
                .query_filtered::<Entity, With<Mesh2d>>()
                .iter(world)
                .count(),
            0
        );
    }

    #[test]
    fn all_michigan_counties_have_finite_closed_geometry_on_the_real_projection() {
        let atlas = CountyAtlas::parse(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../assets/map/county_atlas.bin"
        )))
        .expect("committed atlas");
        let triangles = crate::tessellate::tessellate(&atlas);
        let indices: Vec<_> = (0..atlas.len())
            .filter(|index| atlas.county(*index).expect("index").fips.starts_with("26"))
            .collect();
        assert_eq!(indices.len(), 83);
        for index in indices {
            let county = atlas.county(index).expect("county");
            let origin = county.centroid;
            let (mesh, outline) = county_prism(&atlas, &triangles, index, origin);
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("positions");
            };
            assert!(!positions.is_empty());
            assert_eq!(positions.len() % 3, 0);
            assert!(positions.iter().flatten().all(|value| value.is_finite()));
            assert!(positions.iter().any(|point| point[1] == 0.0));
            assert!(positions
                .iter()
                .any(|point| point[1].to_bits() == 1.0_f32.to_bits()));
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                panic!("normals");
            };
            assert_eq!(normals.len(), positions.len());
            assert!(normals.iter().any(|normal| normal[1] > 0.999));
            assert!(normals.iter().any(|normal| normal[1] < -0.999));
            assert_eq!(outline.primitive_topology(), PrimitiveTopology::LineList);
        }
    }
}
