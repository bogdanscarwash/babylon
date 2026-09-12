//! Selected-county supply relationships and captured physical road geometry.
//! County anchors locate aggregate owners; they never locate factories.

use std::collections::{BTreeMap, BTreeSet};

use babylon_persistence::{
    production_observation::ProductionPhysicalEdge, production_observation::ProductionSnapshot,
};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use super::{scene_point, ObserverMapCamera, BASE_HEIGHT, DATA_HEIGHT, MAP_LAYER};
use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::map::SelectedCounty;
use crate::observer::{ObservationContext, ObserverSession};
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{ObserverFrame, ObserverUiState, ObserverViewport, RoadLayer};
use crate::production::PrimaryView;
use crate::production_layout::place_label;

#[path = "map_network.rs"]
mod network;

const MAX_RELATIONSHIPS: usize = 6;
const ROAD_CREDIT: &str = "© OpenStreetMap contributors\nODbL · openstreetmap.org/copyright";
const CONNECTION_HEIGHT: f32 = BASE_HEIGHT + DATA_HEIGHT + 22.0;

/// Display-only ellipsoidal Albers conversion for the atlas's EPSG:5070.
/// Parameters match its pinned PROJ definition (GRS80, parallels 29.5/45.5,
/// origin 23/-96). No routing distance or material quantity is derived here.
/// Formula: <https://pubs.usgs.gov/pp/1395/report.pdf>, section 14.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn geographic_map_point([longitude, latitude]: [i64; 2]) -> Option<Vec2> {
    if !(-1_800_000_000..=1_800_000_000).contains(&longitude)
        || !(-900_000_000..=900_000_000).contains(&latitude)
    {
        return None;
    }
    let longitude = (longitude as f64 / 10_000_000.0).to_radians();
    let latitude = (latitude as f64 / 10_000_000.0).to_radians();
    let flattening = 1.0 / 298.257_222_101;
    let eccentricity: f64 = (flattening * (2.0_f64 - flattening)).sqrt();
    let e2 = eccentricity * eccentricity;
    let q = |phi: f64| {
        let sin = phi.sin();
        (1.0 - e2)
            * (sin / (1.0 - e2 * sin * sin)
                - babylon_kernel::transcendental::ln(
                    (1.0 - eccentricity * sin) / (1.0 + eccentricity * sin),
                ) / (2.0 * eccentricity))
    };
    let m2 = |phi: f64| phi.cos().powi(2) / (1.0 - e2 * phi.sin().powi(2));
    let p1 = 29.5_f64.to_radians();
    let p2 = 45.5_f64.to_radians();
    let n = (m2(p1) - m2(p2)) / (q(p2) - q(p1));
    let c = m2(p1) + n * q(p1);
    let rho = |phi: f64| 6_378_137.0 * (c - n * q(phi)).sqrt() / n;
    let theta = n * (longitude - (-96.0_f64).to_radians());
    let radius = rho(latitude);
    let point = Vec2::new(
        (radius * theta.sin()) as f32,
        (rho(23.0_f64.to_radians()) - radius * theta.cos()) as f32,
    );
    point.is_finite().then_some(point)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RoadSegment([[i64; 2]; 2]);

type RoadSegments = BTreeMap<RoadSegment, [Vec3; 2]>;

fn projected_segments<'a>(
    edges: impl IntoIterator<Item = &'a ProductionPhysicalEdge>,
    origin: Vec2,
) -> Option<RoadSegments> {
    let mut segments = BTreeSet::new();
    for edge in edges {
        if edge.shape_e7.len() < 2 || edge.shape_e7.windows(2).all(|pair| pair[0] == pair[1]) {
            return None;
        }
        for pair in edge.shape_e7.windows(2).filter(|pair| pair[0] != pair[1]) {
            let mut ends = [pair[0], pair[1]];
            ends.sort_unstable();
            segments.insert(RoadSegment(ends));
        }
    }
    // Opposite directed edges and overlapping paths have one exact geometry
    // key before any lossy display conversion. This never changes routing.
    segments
        .into_iter()
        .map(|segment| {
            let from = geographic_map_point(segment.0[0])?;
            let to = geographic_map_point(segment.0[1])?;
            Some((
                segment,
                [
                    scene_point(from, origin, CONNECTION_HEIGHT),
                    scene_point(to, origin, CONNECTION_HEIGHT),
                ],
            ))
        })
        .collect()
}

fn disclosed_snapshot<'a>(
    frame: &'a ObserverFrame,
    session: &ObserverSession,
) -> Option<&'a ProductionSnapshot> {
    if crate::observer_controls::inspection_availability(session)
        != crate::observer_controls::ControlAvailability::Enabled
    {
        return None;
    }
    frame
        .for_session(session)
        .filter(|frame| {
            frame.visibility
                == babylon_persistence::observer_reader::ObserverVisibility::FullObserver
        })?
        .production
        .as_ref()
}

fn physical_index(
    snapshot: &ProductionSnapshot,
) -> Option<BTreeMap<&str, &ProductionPhysicalEdge>> {
    let mut edges = BTreeMap::new();
    for edge in &snapshot.physical_edges {
        if edges.insert(edge.id.as_str(), edge).is_some() {
            return None;
        }
    }
    Some(edges)
}

#[derive(Clone)]
struct CountyAnchor {
    index: usize,
    name: String,
    position: Vec3,
}

#[derive(Resource)]
pub(super) struct CountyAnchors(BTreeMap<String, CountyAnchor>, Vec2);

impl CountyAnchors {
    pub(super) fn from_atlas(atlas: &CountyAtlas, indices: &[usize], origin: Vec2) -> Self {
        Self(
            indices
                .iter()
                .filter_map(|&index| {
                    let county = atlas.county(index)?;
                    Some((
                        county.fips.to_owned(),
                        CountyAnchor {
                            index,
                            name: county.name.to_owned(),
                            position: scene_point(county.centroid, origin, CONNECTION_HEIGHT),
                        },
                    ))
                })
                .collect(),
            origin,
        )
    }

    fn selected(&self, index: Option<usize>) -> Option<&str> {
        self.0
            .iter()
            .find_map(|(fips, anchor)| (Some(anchor.index) == index).then_some(fips.as_str()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RelationKey {
    supplier: String,
    buyer: String,
    good: String,
    unit: String,
}

#[derive(Clone)]
struct CountyRelationship {
    key: RelationKey,
    from: Vec3,
    to: Vec3,
    caption: String,
    outbound: bool,
    internal: bool,
    physical: Vec<(RoadSegment, [Vec3; 2])>,
    physical_missing: bool,
}

struct RelationshipProjection {
    rows: Vec<CountyRelationship>,
    total: usize,
    available: bool,
}

fn declared_relations(snapshot: &ProductionSnapshot) -> BTreeMap<RelationKey, (String, String)> {
    crate::material_relations::declared_material_relations(snapshot)
        .map(|relation| {
            (
                RelationKey {
                    supplier: relation.supplier.to_owned(),
                    buyer: relation.buyer.to_owned(),
                    good: relation.good_id.to_owned(),
                    unit: relation.unit_id.to_owned(),
                },
                (relation.good.to_owned(), relation.unit.to_owned()),
            )
        })
        .collect()
}

fn county_label(name: &str) -> &str {
    // Keep the public atlas name; omit only its known Michigan county suffix.
    name.strip_suffix(" County, MI").unwrap_or(name)
}

fn relation_roads(
    snapshot: &ProductionSnapshot,
    key: &RelationKey,
    edges: Option<&BTreeMap<&str, &ProductionPhysicalEdge>>,
    origin: Vec2,
) -> Option<RoadSegments> {
    let ids: BTreeSet<_> = snapshot
        .routes
        .iter()
        .filter(|route| {
            route.supplier_site_id == key.supplier
                && route.buyer_site_id == key.buyer
                && route.good_id == key.good
                && route.unit_id == key.unit
        })
        .flat_map(|route| route.physical_edge_ids.iter().map(String::as_str))
        .collect();
    if ids.is_empty() {
        return Some(BTreeMap::new());
    }
    let edges = edges?;
    let selected: Option<Vec<_>> = ids.into_iter().map(|id| edges.get(id).copied()).collect();
    projected_segments(selected?, origin)
}

fn project(
    frame: &ObserverFrame,
    session: &ObserverSession,
    selected: Option<usize>,
    anchors: &CountyAnchors,
    selected_site: Option<&str>,
    material: Option<&crate::map_economy_lens::MaterialGoodKey>,
) -> RelationshipProjection {
    let snapshot = disclosed_snapshot(frame, session);
    let mut result = RelationshipProjection {
        rows: Vec::new(),
        total: 0,
        available: snapshot.is_some(),
    };
    let (Some(snapshot), Some(county), Some(selected_site)) =
        (snapshot, anchors.selected(selected), selected_site)
    else {
        return result;
    };
    let sites: BTreeMap<_, _> = snapshot
        .sites
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect();
    if sites
        .get(selected_site)
        .is_none_or(|site| site.county_geoid != county)
    {
        return result;
    }
    let edges = physical_index(snapshot);
    for (key, (good, unit)) in declared_relations(snapshot) {
        if material
            .is_some_and(|material| material.good_id != key.good || material.unit_id != key.unit)
        {
            continue;
        }
        if key.supplier != selected_site && key.buyer != selected_site {
            continue;
        }
        let (Some(supplier), Some(buyer)) = (
            sites.get(key.supplier.as_str()),
            sites.get(key.buyer.as_str()),
        ) else {
            continue;
        };
        if supplier.county_geoid != county && buyer.county_geoid != county {
            continue;
        }
        let (Some(from), Some(to)) = (
            anchors.0.get(&supplier.county_geoid),
            anchors.0.get(&buyer.county_geoid),
        ) else {
            continue;
        };
        result.total += 1;
        if result.rows.len() == MAX_RELATIONSHIPS {
            continue;
        }
        let outbound = supplier.county_geoid == county;
        let physical = relation_roads(snapshot, &key, edges.as_ref(), anchors.1);
        let physical_missing = physical.is_none();
        result.rows.push(CountyRelationship {
            key,
            from: from.position,
            to: to.position,
            caption: format!(
                "{} -> {}\n{} | {}",
                county_label(&from.name),
                county_label(&to.name),
                good,
                unit
            ),
            outbound,
            internal: supplier.county_geoid == buyer.county_geoid,
            physical: physical.unwrap_or_default().into_iter().collect(),
            physical_missing,
        });
    }
    result
}

#[derive(Component)]
struct RelationshipEntity;

#[derive(Component)]
struct RelationshipLabel {
    anchor: Option<Vec3>,
    order: usize,
    credit: bool,
}

#[derive(Component)]
struct RelationshipJump {
    context: ObservationContext,
    selected_county: usize,
    key: RelationKey,
}

#[derive(Resource, Default)]
struct RelationshipScope(Option<(ObservationContext, Option<usize>)>);

#[derive(Resource)]
struct RelationshipAssets {
    segment: Handle<Mesh>,
    inbound: Handle<StandardMaterial>,
    outbound: Handle<StandardMaterial>,
    captured: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(RelationshipAssets {
        segment: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        inbound: materials.add(StandardMaterial {
            base_color: theme::BLUE,
            unlit: true,
            ..default()
        }),
        outbound: materials.add(StandardMaterial {
            base_color: theme::COPPER,
            unlit: true,
            ..default()
        }),
        captured: materials.add(StandardMaterial {
            base_color: theme::GRAY.with_alpha(0.45),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum RoadBatch {
    Selected,
    Captured,
}

fn road_mesh(segments: &RoadSegments, width: f32) -> Option<Mesh> {
    let mut positions = Vec::new();
    for [from, to] in segments.values() {
        let delta = *to - *from;
        if delta.length_squared() <= f32::EPSILON {
            continue;
        }
        let side = delta.cross(Vec3::Y).normalize_or_zero() * (width * 0.5);
        positions.extend(
            [
                *from - side,
                *from + side,
                *to + side,
                *from - side,
                *to + side,
                *to - side,
            ]
            .map(|point| point.to_array()),
        );
    }
    if positions.is_empty() {
        return None;
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![[0.0, 1.0, 0.0]; positions.len()],
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    Some(mesh)
}

fn spawn_road_batch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    segments: &RoadSegments,
    batch: RoadBatch,
) -> bool {
    let Some(mesh) = road_mesh(
        segments,
        if batch == RoadBatch::Selected {
            1.8
        } else {
            0.6
        },
    ) else {
        return false;
    };
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(if batch == RoadBatch::Captured {
            Vec3::NEG_Y * 2.0
        } else {
            Vec3::ZERO
        }),
        bevy::camera::visibility::RenderLayers::layer(MAP_LAYER),
        Pickable::IGNORE,
        RelationshipEntity,
        batch,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    ));
    true
}

fn spawn_road_credit(commands: &mut Commands) {
    commands
        .spawn(label_bundle(ROAD_CREDIT.to_owned(), None, 0, theme::GRAY))
        .insert((
            RelationshipLabel {
                anchor: None,
                order: 0,
                credit: true,
            },
            TextFont {
                font_size: 13.0,
                ..default()
            },
            ZIndex(13),
            Pickable::IGNORE,
        ));
}

fn connection_points(from: Vec3, to: Vec3) -> [Vec3; 5] {
    if from == to {
        return [
            from,
            from + Vec3::new(18.0, 12.0, 0.0),
            from + Vec3::new(18.0, 18.0, 18.0),
            from + Vec3::new(0.0, 12.0, 18.0),
            to,
        ];
    }
    // Opposite-direction declarations lie on opposite sides of the centroid
    // chord. The bend is a legibility device, not transport geography.
    let side = (to - from).cross(Vec3::Y).normalize_or_zero() * 14.0;
    [
        from,
        from.lerp(to, 0.25) + side + Vec3::Y * 12.0,
        from.lerp(to, 0.5) + side + Vec3::Y * 18.0,
        from.lerp(to, 0.75) + side + Vec3::Y * 12.0,
        to,
    ]
}

fn segment(
    commands: &mut Commands,
    assets: &RelationshipAssets,
    material: &Handle<StandardMaterial>,
    from: Vec3,
    to: Vec3,
) {
    let delta = to - from;
    if delta.length_squared() <= f32::EPSILON {
        return;
    }
    commands.spawn((
        Mesh3d(assets.segment.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation((from + to) * 0.5)
            .looking_to(delta, Vec3::Y)
            .with_scale(Vec3::new(1.8, 1.8, delta.length())),
        bevy::camera::visibility::RenderLayers::layer(MAP_LAYER),
        Pickable::IGNORE,
        RelationshipEntity,
        DeclaredSurface::new(SurfaceId::ObserverProduction),
    ));
}

fn dashed_segments(from: Vec3, to: Vec3) -> [(Vec3, Vec3); 3] {
    [0.0, 1.0 / 3.0, 2.0 / 3.0].map(|start| (from.lerp(to, start), from.lerp(to, start + 0.23)))
}

fn spawn_connection(
    commands: &mut Commands,
    assets: &RelationshipAssets,
    relation: &CountyRelationship,
) -> Vec3 {
    if relation.physical_missing {
        return relation.from;
    }
    if !relation.physical.is_empty() {
        let points = &relation.physical[relation.physical.len() / 2].1;
        return points[0].lerp(points[1], 0.5);
    }
    let points = connection_points(relation.from, relation.to);
    let material = if relation.outbound {
        &assets.outbound
    } else {
        &assets.inbound
    };
    for pair in points.windows(2) {
        for (from, to) in dashed_segments(pair[0], pair[1]) {
            segment(commands, assets, material, from, to);
        }
    }
    let direction = (points[4] - points[3]).normalize_or_zero();
    let tip = points[3].lerp(points[4], 0.7);
    let side = direction.cross(Vec3::Y).normalize_or_zero() * 4.0;
    for wing in [side, -side] {
        segment(
            commands,
            assets,
            material,
            tip - direction * 8.0 + wing,
            tip,
        );
    }
    points[2]
}

fn label_bundle(caption: String, anchor: Option<Vec3>, order: usize, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            max_width: px(270),
            padding: UiRect::axes(px(8), px(5)),
            border: UiRect::bottom(px(2)),
            ..default()
        },
        BackgroundColor(theme::INK.with_alpha(0.96)),
        BorderColor::all(color),
        ZIndex(5),
        Visibility::Hidden,
        RelationshipEntity,
        RelationshipLabel {
            anchor,
            order,
            credit: false,
        },
        DeclaredSurface::new(SurfaceId::ObserverProduction),
        Text::new(caption),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(theme::PAPER),
        crate::observer_ui::ObserverFontRole::Body,
    )
}

#[derive(SystemParam)]
struct RelationshipObservation<'w> {
    frame: Res<'w, ObserverFrame>,
    session: Res<'w, ObserverSession>,
    selected: Res<'w, SelectedCounty>,
    anchors: Res<'w, CountyAnchors>,
    navigation: Res<'w, crate::production::ProductionNavigation>,
    ui: Res<'w, ObserverUiState>,
    view: Res<'w, PrimaryView>,
}

fn rebuild(
    mut commands: Commands,
    observation: RelationshipObservation,
    assets: Res<RelationshipAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scope: ResMut<RelationshipScope>,
    old: Query<Entity, With<RelationshipEntity>>,
) {
    let current = (observation.session.context(), observation.selected.0);
    if scope.0.as_ref() == Some(&current)
        && !observation.frame.is_changed()
        && !observation.navigation.is_changed()
        && !observation.ui.is_changed()
        && !observation.session.is_changed()
        && !observation.view.is_changed()
    {
        return;
    }
    scope.0 = Some(current.clone());
    for entity in &old {
        commands.entity(entity).despawn();
    }
    if *observation.view != PrimaryView::Map
        || observation.ui.road_layer == RoadLayer::EconomyNetwork
    {
        return;
    }
    let projection = project(
        &observation.frame,
        &observation.session,
        observation.selected.0,
        &observation.anchors,
        observation
            .navigation
            .selected_site
            .as_deref()
            .filter(|id| {
                observation
                    .frame
                    .for_session(&observation.session)
                    .and_then(|frame| frame.production.as_ref())
                    .is_some_and(|snapshot| {
                        snapshot.sites.iter().any(|site| {
                            site.id == *id
                                && Some(site.county_geoid.as_str())
                                    == observation.anchors.selected(observation.selected.0)
                        })
                    })
            }),
        match &observation.ui.lens {
            crate::map_economy_lens::MapLens::Material { good, .. } => good.as_ref(),
            _ => None,
        },
    );
    let selected_roads: RoadSegments = projection
        .rows
        .iter()
        .flat_map(|relation| relation.physical.iter().copied())
        .collect();
    let network = if observation.ui.road_layer == RoadLayer::CapturedRoads {
        disclosed_snapshot(&observation.frame, &observation.session)
            .and_then(physical_index)
            .and_then(|edges| projected_segments(edges.into_values(), observation.anchors.1))
    } else {
        Some(RoadSegments::new())
    };
    let mut physical_drawn = spawn_road_batch(
        &mut commands,
        &mut meshes,
        &assets.inbound,
        &selected_roads,
        RoadBatch::Selected,
    );
    if let Some(network) = &network {
        let remaining = network
            .iter()
            .filter(|(key, _)| !selected_roads.contains_key(*key))
            .map(|(key, value)| (*key, *value))
            .collect();
        physical_drawn |= spawn_road_batch(
            &mut commands,
            &mut meshes,
            &assets.captured,
            &remaining,
            RoadBatch::Captured,
        );
    }
    if physical_drawn {
        spawn_road_credit(&mut commands);
    }
    let heading = relationship_heading(
        &projection,
        observation.selected.0,
        observation.navigation.selected_site.is_some(),
        observation.ui.road_layer,
        network.as_ref(),
    );
    commands.spawn((
        label_bundle(heading, None, 1, theme::PAPER),
        Pickable::IGNORE,
    ));
    spawn_relationships(
        &mut commands,
        &assets,
        &projection,
        &current.0,
        observation.selected.0,
    );
}

fn relationship_heading(
    projection: &RelationshipProjection,
    selected: Option<usize>,
    cohort_selected: bool,
    layer: RoadLayer,
    network: Option<&RoadSegments>,
) -> String {
    let heading = if !projection.available {
        "Supply links unavailable in this observation.".to_owned()
    } else if selected.is_none() {
        "Select a county to trace its supply relationships.".to_owned()
    } else if !cohort_selected {
        "Choose a county cohort, then return to World to trace its shipment paths.".to_owned()
    } else if projection.total == 0 {
        "No disclosed supply links for this cohort and commodity selection.".to_owned()
    } else {
        format!(
            "Selected relationships: {} of {}\nSolid: captured roads · dashed: schematic",
            projection.rows.len(),
            projection.total
        )
    };
    match layer {
        RoadLayer::EconomyNetwork => "Economy network".into(),
        RoadLayer::SelectedPaths => format!("Selected paths\n{heading}"),
        RoadLayer::CapturedRoads if network.is_none_or(BTreeMap::is_empty) => format!("Captured roads unavailable in this observation.\n{heading}"),
        RoadLayer::CapturedRoads => format!("Captured roads / campaign physical route network\nSelected shipment paths highlighted\n{heading}"),
    }
}

fn spawn_relationships(
    commands: &mut Commands,
    assets: &RelationshipAssets,
    projection: &RelationshipProjection,
    context: &ObservationContext,
    selected: Option<usize>,
) {
    for (index, relation) in projection.rows.iter().enumerate() {
        let anchor = spawn_connection(commands, assets, relation);
        let caption = if relation.physical_missing {
            format!(
                "{}\nCaptured physical geometry unavailable",
                relation.caption
            )
        } else if relation.internal {
            format!("{}\nWithin this county | P for Work", relation.caption)
        } else {
            format!(
                "{}\n{}",
                if relation.outbound {
                    "OUT / SUPPLIES"
                } else {
                    "IN / DEPENDS ON"
                },
                relation.caption
            )
        };
        let mut label = commands.spawn(label_bundle(
            caption,
            Some(anchor),
            index + 2,
            if relation.outbound {
                theme::COPPER
            } else {
                theme::BLUE
            },
        ));
        if relation.internal {
            label.insert(Pickable::IGNORE);
        } else if let Some(selected_county) = selected {
            label.insert((
                Button,
                RelationshipJump {
                    context: context.clone(),
                    selected_county,
                    key: relation.key.clone(),
                },
            ));
        }
    }
}

fn jump_target(
    jump: &RelationshipJump,
    frame: &ObserverFrame,
    session: &ObserverSession,
    selected: Option<usize>,
    anchors: &CountyAnchors,
) -> Option<usize> {
    if !session.accepts(&jump.context) || selected != Some(jump.selected_county) {
        return None;
    }
    // Revalidate the exact disclosed relation, independently of which six
    // labels another material filter would have placed on its first page.
    let snapshot = disclosed_snapshot(frame, session)?;
    if !declared_relations(snapshot).contains_key(&jump.key) {
        return None;
    }
    let supplier = snapshot
        .sites
        .iter()
        .find(|site| site.id == jump.key.supplier)?;
    let buyer = snapshot
        .sites
        .iter()
        .find(|site| site.id == jump.key.buyer)?;
    if supplier.county_geoid == buyer.county_geoid {
        return None;
    }
    let county = anchors.selected(selected)?;
    let destination = if supplier.county_geoid == county {
        &buyer.county_geoid
    } else if buyer.county_geoid == county {
        &supplier.county_geoid
    } else {
        return None;
    };
    anchors.0.get(destination).map(|anchor| anchor.index)
}

fn input(
    buttons: Query<(&Interaction, &RelationshipJump), Changed<Interaction>>,
    frame: Res<ObserverFrame>,
    session: Res<ObserverSession>,
    anchors: Res<CountyAnchors>,
    ui: Res<ObserverUiState>,
    view: Res<PrimaryView>,
    mut selected: ResMut<SelectedCounty>,
) {
    if *view != PrimaryView::Map
        || ui.menu_open
        || ui.splash_visible
        || ui.comparison_open
        || ui.disclosure.is_some()
    {
        return;
    }
    for (interaction, jump) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(target) = jump_target(jump, &frame, &session, selected.0, &anchors) {
            selected.0 = Some(target);
        }
    }
}

type RelationshipLabels<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static RelationshipLabel,
        &'static ComputedNode,
        &'static mut Node,
        &'static mut Visibility,
    ),
>;

#[derive(SystemParam)]
struct LabelPlacement<'w, 's> {
    view: Res<'w, PrimaryView>,
    ui: Res<'w, ObserverUiState>,
    viewport: Res<'w, ObserverViewport>,
    scale: Res<'w, UiScale>,
    windows: Query<'w, 's, &'static Window, With<bevy::window::PrimaryWindow>>,
    camera: Query<'w, 's, (&'static Camera, &'static Transform), With<ObserverMapCamera>>,
    labels: RelationshipLabels<'w, 's>,
}

fn place_labels(mut placement: LabelPlacement) {
    let active = *placement.view == PrimaryView::Map
        && !placement.ui.menu_open
        && !placement.ui.splash_visible
        && !placement.ui.comparison_open;
    let mut order: Vec<_> = placement
        .labels
        .iter()
        .map(|(entity, label, ..)| (label.order, entity))
        .collect();
    order.sort_by_key(|(order, _)| *order);
    let mut occupied = Vec::new();
    for (_, entity) in order {
        let Ok((_, label, computed, mut node, mut visibility)) = placement.labels.get_mut(entity)
        else {
            continue;
        };
        let rect = if active && (label.credit || placement.ui.disclosure.is_none()) {
            match (
                placement.viewport.0,
                placement.camera.single(),
                placement.windows.single(),
            ) {
                (Some(bounds), Ok((camera, transform)), Ok(window)) => {
                    let size = computed.size() / window.scale_factor();
                    if size.x <= 0.0 || size.y <= 0.0 {
                        None
                    } else if label.credit {
                        Some(Rect::from_corners(
                            bounds.max - size - Vec2::splat(8.0),
                            bounds.max - Vec2::splat(8.0),
                        ))
                    } else if let Some(anchor) = label.anchor {
                        camera
                            .world_to_viewport(&GlobalTransform::from(*transform), anchor)
                            .ok()
                            .and_then(|anchor| place_label(anchor, bounds, size, &occupied))
                    } else {
                        Some(Rect::from_corners(
                            Vec2::new(bounds.max.x - size.x - 8.0, bounds.min.y + 8.0),
                            Vec2::new(bounds.max.x - 8.0, bounds.min.y + size.y + 8.0),
                        ))
                    }
                }
                _ => None,
            }
        } else {
            None
        };
        visibility.set_if_neq(if rect.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if let Some(rect) = rect {
            let (left, top) = (
                px(rect.min.x / placement.scale.0),
                px(rect.min.y / placement.scale.0),
            );
            if node.left != left || node.top != top {
                node.left = left;
                node.top = top;
            }
            occupied.push(rect);
        }
    }
}

pub(super) fn install(app: &mut App) {
    network::install(app);
    app.init_resource::<RelationshipScope>()
        .add_systems(Startup, setup)
        .add_systems(Update, input.in_set(ObserverSet::Input))
        .add_systems(Update, rebuild.in_set(ObserverSet::Paint))
        .add_systems(Update, place_labels.after(super::sync_camera));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::Perspective;
    use babylon_persistence::{
        identity::CampaignId, observer_reader::ObserverEconomySnapshot,
        observer_reader::ObserverVisibility, production_observation::ProductionInput,
        production_observation::ProductionSite,
    };

    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Probe degrees are range-checked before E7 rounding; expected projected coordinates use the renderer's f32 display precision."
    )]
    fn captured_geographic_shapes_use_the_atlas_projection_and_reject_invalid_coordinates() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/michigan_atlas_land_probes.json"
        ))
        .unwrap();
        for probe in fixture["probes"].as_array().unwrap() {
            let longitude_degrees = probe["epsg4269"][0].as_f64().unwrap();
            let latitude_degrees = probe["epsg4269"][1].as_f64().unwrap();
            assert!((-180.0..=180.0).contains(&longitude_degrees));
            assert!((-90.0..=90.0).contains(&latitude_degrees));
            let projected = [
                probe["epsg5070"][0].as_f64().unwrap(),
                probe["epsg5070"][1].as_f64().unwrap(),
            ];
            assert!(projected
                .iter()
                .all(|value| value.is_finite() && value.abs() <= f64::from(f32::MAX)));
            let longitude = (longitude_degrees * 10_000_000.0).round() as i64;
            let latitude = (latitude_degrees * 10_000_000.0).round() as i64;
            let expected = Vec2::new(projected[0] as f32, projected[1] as f32);
            assert!(
                geographic_map_point([longitude, latitude])
                    .unwrap()
                    .distance(expected)
                    < 1.0,
                "atlas probe {}",
                probe["label"]
            );
        }
        assert!(geographic_map_point([i64::MAX, 0]).is_none());
        assert!(geographic_map_point([0, 900_000_001]).is_none());
    }

    #[test]
    fn schematic_segments_have_visible_gaps_and_preserve_direction() {
        let segments = dashed_segments(Vec3::ZERO, Vec3::X * 90.0);
        for pair in segments.windows(2) {
            assert!(pair[0].0.x < pair[0].1.x);
            assert!(pair[0].1.x < pair[1].0.x);
        }
        assert!(segments[2].1.x < 90.0);
        let reversed = dashed_segments(Vec3::X * 90.0, Vec3::ZERO);
        assert!(reversed.iter().all(|(from, to)| from.x > to.x));
    }

    fn site(id: &str, county: &str) -> ProductionSite {
        ProductionSite {
            id: id.into(),
            county_geoid: county.into(),
            name: format!("{id} county cohort"),
            industry_code: "331".into(),
            observed_employment: None,
            inventory: Vec::new(),
            role: babylon_persistence::production_observation::ProductionSiteRole::Production,
            sector_code: "31-33".into(),
            processes: vec![
                babylon_persistence::production_observation::ProductionProcess {
                    id: "fixture-process".into(),
                    name: "Fixture process".into(),
                    output_good_id: "steel".into(),
                    output_unit_id: "kg".into(),
                    output_good: "steel".into(),
                    output_unit: "kg".into(),
                    output_per_batch: 1,
                    available_batches: 0,
                    planned_batches: None,
                    produced_batches: None,
                    inputs: Vec::new(),
                    labor: Vec::new(),
                },
            ],
        }
    }

    pub(super) fn fixture() -> (ObserverSession, ObserverFrame, CountyAnchors) {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        session.ready(3, Some("committed".into()));
        assert!(session.installed(&session.context()));
        let supplier = site("a", "26163");
        let mut buyer = site("b", "26099");
        for (good, unit) in [("steel", "kg"), ("ore", "tonne")] {
            buyer.processes[0].inputs.push(ProductionInput {
                good_id: good.into(),
                unit_id: unit.into(),
                good: good.into(),
                unit: unit.into(),
                quantity_per_batch: 7,
                on_hand: 2,
                supplier_site_ids: vec![supplier.id.clone()],
            });
        }
        let frame = ObserverFrame(Some(ObserverEconomySnapshot {
            campaign_id: session.campaign.as_uuid().to_string(),
            resolve_tick: 3,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: Some("committed".into()),
            envelope_digest: None,
            visibility: ObserverVisibility::FullObserver,
            counties: Vec::new(),
            production: Some(ProductionSnapshot {
                content_authority_sha256: "a".repeat(64),
                road_source: None,
                physical_edges: Vec::new(),
                merchant_handling_accounts: Vec::new(),
                final_demand_accounts: Vec::new(),
                freight_capacity_accounts: Vec::new(),
                material_balance: None,
                labor_accounts: Vec::new(),
                staffing_accounts: Vec::new(),
                scenario_label: "fixture".into(),
                horizon_period: 16,
                sites: vec![supplier, buyer, site("unrelated", "26161")],
                routes: Vec::new(),
                freight: Vec::new(),
                events: Vec::new(),
                observed_contexts: Vec::new(),
                process_attributions: Vec::new(),
                provenance: Vec::new(),
            }),
        }));
        let anchors = CountyAnchors(
            [
                (
                    "26163".into(),
                    CountyAnchor {
                        index: 1,
                        name: "Wayne County, MI".into(),
                        position: Vec3::new(0.0, CONNECTION_HEIGHT, 0.0),
                    },
                ),
                (
                    "26099".into(),
                    CountyAnchor {
                        index: 2,
                        name: "Macomb County, MI".into(),
                        position: Vec3::new(40.0, CONNECTION_HEIGHT, 10.0),
                    },
                ),
                (
                    "26161".into(),
                    CountyAnchor {
                        index: 3,
                        name: "Washtenaw County, MI".into(),
                        position: Vec3::new(-40.0, CONNECTION_HEIGHT, 5.0),
                    },
                ),
            ]
            .into(),
            Vec2::ZERO,
        );
        (session, frame, anchors)
    }

    #[test]
    fn county_overview_waits_for_an_explicit_cohort_before_drawing_shipments() {
        let (session, frame, anchors) = fixture();
        let overview = project(&frame, &session, Some(1), &anchors, None, None);
        assert!(overview.available);
        assert!(overview.rows.is_empty());
        assert_eq!(overview.total, 0);
        let selected = project(&frame, &session, Some(1), &anchors, Some("a"), None);
        assert_eq!(selected.total, 2);
        assert!(
            project(&frame, &session, Some(2), &anchors, Some("a"), None)
                .rows
                .is_empty(),
            "moving to another county clears the previous cohort's paths"
        );
    }

    #[test]
    fn selected_incident_dependencies_keep_goods_units_and_direction_separate() {
        let (session, mut frame, anchors) = fixture();
        let rows = project(&frame, &session, Some(1), &anchors, Some("a"), None);
        assert_eq!(rows.total, 2);
        assert!(rows.rows.iter().all(|row| row.outbound));
        assert_eq!(rows.rows[0].caption, "Wayne -> Macomb\nore | tonne");
        assert_eq!(rows.rows[1].caption, "Wayne -> Macomb\nsteel | kg");
        assert!(rows.rows.iter().all(|row| row.caption.lines().count() == 2));
        let keys: Vec<_> = rows.rows.iter().map(|row| row.key.clone()).collect();
        frame
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .sites
            .reverse();
        assert_eq!(
            project(&frame, &session, Some(1), &anchors, Some("a"), None)
                .rows
                .iter()
                .map(|row| row.key.clone())
                .collect::<Vec<_>>(),
            keys
        );
        let inbound = project(&frame, &session, Some(2), &anchors, Some("b"), None);
        assert!(inbound.rows.iter().all(|row| !row.outbound));
        assert_eq!(
            project(&frame, &session, Some(3), &anchors, None, None).total,
            0
        );
    }

    #[test]
    fn selected_physical_path_uses_captured_edges_and_exact_material_identity() {
        use babylon_persistence::{
            production_observation::ProductionPhysicalEdge,
            production_observation::ProductionRoute,
            production_observation::ProductionRouteTransport,
        };
        let (session, mut frame, anchors) = fixture();
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        snapshot.physical_edges = vec![ProductionPhysicalEdge {
            id: "road".into(),
            shape_e7: vec![[-830_000_000, 423_000_000], [-829_900_000, 423_100_000]],
            distance_mm: 1_500_000,
        }];
        snapshot.routes = vec![ProductionRoute {
            id: "supply-road".into(),
            supplier_site_id: "a".into(),
            buyer_site_id: "b".into(),
            good_id: "steel".into(),
            unit_id: "kg".into(),
            good: "steel".into(),
            unit: "kg".into(),
            travel_periods: 1,
            transport_kind: ProductionRouteTransport::Staged,
            physical_edge_ids: vec!["road".into(), "road".into()],
            distance_mm: Some(3_000_000),
            stages: Vec::new(),
            grams_per_unit: 1000,
            ordered: 10,
            shipped: 5,
            delivered: 0,
            lost: 0,
            realized: 0,
            backlog: 10,
        }];
        let mut reversed = snapshot.physical_edges[0].clone();
        reversed.id = "road-reversed".into();
        reversed.shape_e7.reverse();
        snapshot.physical_edges.push(reversed);
        snapshot.routes[0]
            .physical_edge_ids
            .push("road-reversed".into());
        let filter = crate::map_economy_lens::MaterialGoodKey {
            good_id: "steel".into(),
            unit_id: "kg".into(),
        };
        let projected = project(
            &frame,
            &session,
            Some(1),
            &anchors,
            Some("a"),
            Some(&filter),
        );
        assert_eq!(projected.rows.len(), 1);
        assert_eq!(
            projected.rows[0].physical.len(),
            1,
            "shared or reversed captured geometry is drawn once, independent of edge IDs"
        );
        assert!(!projected.rows[0].physical_missing);
        assert_eq!(
            projected.rows[0].physical[0].0,
            RoadSegment([[-830_000_000, 423_000_000], [-829_900_000, 423_100_000]])
        );
        let other_unit = crate::map_economy_lens::MaterialGoodKey {
            unit_id: "tonne".into(),
            ..filter.clone()
        };
        assert!(project(
            &frame,
            &session,
            Some(1),
            &anchors,
            Some("a"),
            Some(&other_unit)
        )
        .rows
        .is_empty());
        frame
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .physical_edges
            .clear();
        let missing = project(
            &frame,
            &session,
            Some(1),
            &anchors,
            Some("a"),
            Some(&filter),
        );
        assert!(missing.rows[0].physical_missing);
        assert!(
            missing.rows[0].physical.is_empty(),
            "missing captured geometry cannot become an invented road"
        );
    }

    fn road_layer_app() -> App {
        let (session, mut frame, anchors) = fixture();
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        let points = [
            [-830_000_000, 423_000_000],
            [-829_900_000, 423_100_000],
            [-829_800_000, 423_200_000],
            [-829_700_000, 423_300_000],
        ];
        for (id, shape) in [
            ("road", points.to_vec()),
            ("reverse", points.into_iter().rev().collect()),
            ("overlap", points[1..3].to_vec()),
            ("branch", vec![points[2], [-829_500_000, 423_200_000]]),
        ] {
            snapshot.physical_edges.push(ProductionPhysicalEdge {
                id: id.into(),
                shape_e7: shape,
                distance_mm: 1_000_000,
            });
        }
        let mut route = crate::production_freight::tests::fixture().routes.remove(0);
        route.id = "road-route".into();
        route.supplier_site_id = "a".into();
        route.buyer_site_id = "b".into();
        route.good_id = "steel".into();
        route.good = "Steel".into();
        route.physical_edge_ids = vec!["road".into(), "reverse".into(), "overlap".into()];
        let mut other = route.clone();
        other.id = "other-route".into();
        other.buyer_site_id = "unrelated".into();
        other.good_id = "ore".into();
        other.unit_id = "tonne".into();
        other.physical_edge_ids = vec!["branch".into()];
        snapshot.routes = vec![route, other];
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(frame)
            .insert_resource(anchors)
            .insert_resource(SelectedCounty(Some(1)))
            .insert_resource(PrimaryView::Map)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                road_layer: RoadLayer::SelectedPaths,
                lens: crate::map_economy_lens::MapLens::Material {
                    kind: crate::map_economy_lens::MaterialLensKind::OnHand,
                    good: Some(crate::map_economy_lens::MaterialGoodKey {
                        good_id: "steel".into(),
                        unit_id: "kg".into(),
                    }),
                },
                ..default()
            })
            .init_resource::<crate::production::ProductionNavigation>()
            .init_resource::<RelationshipScope>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Startup, setup)
            .add_systems(Update, rebuild);
        app.world_mut()
            .resource_mut::<crate::production::ProductionNavigation>()
            .selected_site = Some("a".into());
        network::install(&mut app);
        app.init_resource::<ObserverViewport>()
            .init_resource::<UiScale>();
        app
    }

    fn road_batch_vertices(app: &mut App) -> Vec<(RoadBatch, usize)> {
        let handles: Vec<_> = app
            .world_mut()
            .query::<(&RoadBatch, &Mesh3d)>()
            .iter(app.world())
            .map(|(batch, mesh)| (*batch, mesh.0.clone()))
            .collect();
        let meshes = app.world().resource::<Assets<Mesh>>();
        handles
            .into_iter()
            .map(|(batch, handle)| (batch, meshes.get(&handle).unwrap().count_vertices()))
            .collect()
    }

    #[test]
    fn world_opens_with_the_whole_economy_before_a_county_or_cohort_is_selected() {
        let mut app = road_layer_app();
        *app.world_mut().resource_mut::<ObserverUiState>() = ObserverUiState {
            menu_open: false,
            splash_visible: false,
            ..default()
        };
        app.world_mut().resource_mut::<SelectedCounty>().0 = None;
        app.world_mut()
            .resource_mut::<crate::production::ProductionNavigation>()
            .selected_site = None;
        app.update();
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .any(|text| text.0.contains("ECONOMY NETWORK") && text.0.contains("3 cohorts")),
            "World must disclose the whole admitted economy without first selecting a chain"
        );
    }

    #[test]
    fn road_layer_batches_deduplicated_network_and_keeps_selected_paths_highlighted() {
        let mut app = road_layer_app();
        app.world_mut()
            .resource_mut::<crate::production::ProductionNavigation>()
            .selected_site = None;
        app.update();
        assert!(road_batch_vertices(&mut app).is_empty());
        app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::CapturedRoads;
        app.update();
        assert_eq!(road_batch_vertices(&mut app), [(RoadBatch::Captured, 24)]);
        app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::SelectedPaths;
        app.world_mut()
            .resource_mut::<crate::production::ProductionNavigation>()
            .selected_site = Some("a".into());
        app.update();
        assert_eq!(road_batch_vertices(&mut app), [(RoadBatch::Selected, 18)]);
        app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::CapturedRoads;
        app.update();
        let batches = road_batch_vertices(&mut app);
        assert_eq!(
            batches.len(),
            2,
            "road segments share two batched mesh entities"
        );
        assert!(batches.contains(&(RoadBatch::Selected, 18)));
        assert!(
            batches.contains(&(RoadBatch::Captured, 6)),
            "selected segments are excluded from the muted network batch"
        );
        assert!(app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text
                .0
                .contains("Captured roads / campaign physical route network")
                && text.0.contains("Selected shipment paths highlighted")));
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .physical_edges
            .reverse();
        app.update();
        assert_eq!(road_batch_vertices(&mut app).len(), 2);
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
        app.update();
        assert!(road_batch_vertices(&mut app).is_empty());
    }

    #[test]
    fn road_layer_removes_geometry_for_loading_error_preview_or_stale_observations() {
        for invalid in [
            "loading",
            "failed",
            "preview",
            "period",
            "campaign",
            "missing",
            "empty geometry",
        ] {
            let mut app = road_layer_app();
            app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::CapturedRoads;
            app.update();
            assert_eq!(road_batch_vertices(&mut app).len(), 2);
            assert_eq!(
                app.world_mut()
                    .query::<&Text>()
                    .iter(app.world())
                    .filter(|text| text.0
                        == "© OpenStreetMap contributors\nODbL · openstreetmap.org/copyright")
                    .count(),
                1
            );
            match invalid {
                "loading" => {
                    app.world_mut().resource_mut::<ObserverSession>().phase =
                        crate::observer::SessionPhase::Loading;
                }
                "failed" => {
                    app.world_mut().resource_mut::<ObserverSession>().phase =
                        crate::observer::SessionPhase::Failed;
                }
                "preview" => {
                    app.world_mut()
                        .resource_mut::<ObserverSession>()
                        .set_perspective(Perspective::PlayerKnowledge);
                    app.world_mut().resource_mut::<ObserverSession>().phase =
                        crate::observer::SessionPhase::Ready;
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .visibility = ObserverVisibility::KnownPreview;
                }
                "period" => {
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .resolve_tick = 2;
                }
                "campaign" => {
                    app.world_mut()
                        .resource_mut::<ObserverFrame>()
                        .0
                        .as_mut()
                        .unwrap()
                        .campaign_id = uuid::Uuid::from_u128(99).to_string();
                }
                "missing" => app.world_mut().resource_mut::<ObserverFrame>().0 = None,
                "empty geometry" => app
                    .world_mut()
                    .resource_mut::<ObserverFrame>()
                    .0
                    .as_mut()
                    .unwrap()
                    .production
                    .as_mut()
                    .unwrap()
                    .physical_edges
                    .clear(),
                _ => unreachable!(),
            }
            app.update();
            assert!(road_batch_vertices(&mut app).is_empty(), "{invalid}");
            assert!(
                !app.world_mut()
                    .query::<&Text>()
                    .iter(app.world())
                    .any(|text| text.0.contains("OpenStreetMap")),
                "{invalid}"
            );
        }
    }

    #[test]
    fn road_layer_uses_only_the_held_periods_captured_geometry() {
        let mut app = road_layer_app();
        app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::CapturedRoads;
        app.update();
        assert_eq!(road_batch_vertices(&mut app).len(), 2);
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .viewed_tick = 2;
        {
            let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
            let snapshot = frame.0.as_mut().unwrap();
            snapshot.resolve_tick = 2;
            snapshot
                .production
                .as_mut()
                .unwrap()
                .physical_edges
                .retain(|edge| edge.id != "branch");
        }
        app.update();
        assert_eq!(road_batch_vertices(&mut app), [(RoadBatch::Selected, 18)]);
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
        assert_eq!(app.world().resource::<ObserverSession>().viewed_tick, 2);
    }

    #[test]
    fn road_layer_refuses_invalid_network_geometry_without_inventing_roads() {
        let mut app = road_layer_app();
        app.world_mut().resource_mut::<ObserverUiState>().road_layer = RoadLayer::CapturedRoads;
        app.world_mut()
            .resource_mut::<ObserverFrame>()
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .physical_edges
            .last_mut()
            .unwrap()
            .shape_e7[0] = [i64::MAX, 0];
        app.update();
        assert_eq!(road_batch_vertices(&mut app), [(RoadBatch::Selected, 18)]);
        assert!(app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .any(|text| text
                .0
                .contains("Captured roads unavailable in this observation")));
    }

    #[test]
    fn compact_labels_preserve_unrecognized_public_names_without_inference() {
        let (session, frame, mut anchors) = fixture();
        anchors.0.get_mut("26163").unwrap().name = "Disclosed district".into();
        assert_eq!(
            project(&frame, &session, Some(1), &anchors, Some("a"), None).rows[0].caption,
            "Disclosed district -> Macomb\nore | tonne"
        );
        assert_eq!(county_label("Wayne County, NE"), "Wayne County, NE");
    }

    #[test]
    fn missing_disclosed_sites_or_county_anchors_never_create_endpoints() {
        let (session, mut frame, mut anchors) = fixture();
        anchors.0.remove("26099");
        assert_eq!(
            project(&frame, &session, Some(1), &anchors, Some("a"), None).total,
            0
        );
        let (_, _, anchors) = fixture();
        frame
            .0
            .as_mut()
            .unwrap()
            .production
            .as_mut()
            .unwrap()
            .sites
            .retain(|site| site.id != "a");
        assert_eq!(
            project(&frame, &session, Some(2), &anchors, Some("b"), None).total,
            0
        );
    }

    #[test]
    fn scope_and_known_capability_clear_relationships_and_refuse_stale_navigation() {
        let (mut session, mut frame, anchors) = fixture();
        let row = project(&frame, &session, Some(1), &anchors, Some("a"), None)
            .rows
            .remove(0);
        let jump = RelationshipJump {
            context: session.context(),
            selected_county: 1,
            key: row.key,
        };
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(1), &anchors),
            Some(2)
        );
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(2), &anchors),
            None
        );
        frame.0.as_mut().unwrap().resolve_tick = 2;
        assert!(
            project(&frame, &session, Some(1), &anchors, Some("a"), None)
                .rows
                .is_empty()
        );
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(1), &anchors),
            None
        );
        frame.0.as_mut().unwrap().resolve_tick = 3;
        session.set_perspective(Perspective::PlayerKnowledge);
        assert!(!project(&frame, &session, Some(1), &anchors, Some("a"), None).available);
        frame.0.as_mut().unwrap().visibility = ObserverVisibility::KnownPreview;
        frame.0.as_mut().unwrap().production = None;
        assert!(!project(&frame, &session, Some(1), &anchors, Some("a"), None).available);
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(1), &anchors),
            None
        );
        session.set_perspective(Perspective::FullObserver);
        let (_, valid, _) = fixture();
        frame = valid;
        frame.0.as_mut().unwrap().campaign_id = uuid::Uuid::from_u128(9).to_string();
        assert!(
            project(&frame, &session, Some(1), &anchors, Some("a"), None)
                .rows
                .is_empty()
        );
    }

    #[test]
    fn schematic_connections_preserve_direction_and_same_county_loops_without_motion() {
        let from = Vec3::new(0.0, CONNECTION_HEIGHT, 0.0);
        let to = Vec3::new(40.0, CONNECTION_HEIGHT, 0.0);
        let forward = connection_points(from, to);
        let reverse = connection_points(to, from);
        assert_eq!((forward[0], forward[4]), (from, to));
        assert_eq!((reverse[0], reverse[4]), (to, from));
        assert!(forward[2].z * reverse[2].z < 0.0);
        let internal = connection_points(from, from);
        assert_eq!((internal[0], internal[4]), (from, from));
        assert!(internal.windows(2).all(|pair| pair[0] != pair[1]));
        assert!(internal
            .iter()
            .chain(forward.iter())
            .all(|point| point.is_finite()));
    }

    #[test]
    fn display_bound_counts_only_valid_disclosed_relationships() {
        let (session, mut frame, anchors) = fixture();
        let input = frame.0.as_mut().unwrap().production.as_mut().unwrap().sites[1].processes[0]
            .inputs[0]
            .clone();
        for index in 0..10 {
            let mut next = input.clone();
            next.good_id = format!("good-{index:02}");
            frame.0.as_mut().unwrap().production.as_mut().unwrap().sites[1].processes[0]
                .inputs
                .push(next);
        }
        let projection = project(&frame, &session, Some(1), &anchors, Some("a"), None);
        assert_eq!(projection.total, 12);
        assert_eq!(projection.rows.len(), MAX_RELATIONSHIPS);
    }

    #[test]
    fn relationship_button_follows_county_only_in_the_visible_unblocked_map() {
        let (session, frame, anchors) = fixture();
        let key = project(&frame, &session, Some(1), &anchors, Some("a"), None)
            .rows
            .remove(0)
            .key;
        let context = session.context();
        let mut app = App::new();
        app.insert_resource(session)
            .insert_resource(frame)
            .insert_resource(anchors)
            .insert_resource(SelectedCounty(Some(1)))
            .insert_resource(PrimaryView::Production)
            .insert_resource(ObserverUiState {
                menu_open: false,
                splash_visible: false,
                ..default()
            })
            .add_systems(Update, input);
        let button = app
            .world_mut()
            .spawn((
                Interaction::Pressed,
                RelationshipJump {
                    context,
                    selected_county: 1,
                    key,
                },
            ))
            .id();
        app.update();
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(1));
        *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Map;
        app.world_mut().resource_mut::<ObserverUiState>().disclosure =
            Some(crate::observer_ui::ObserverDisclosure::Time);
        app.world_mut()
            .get_mut::<Interaction>(button)
            .unwrap()
            .set_changed();
        app.update();
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(1));
        app.world_mut().resource_mut::<ObserverUiState>().disclosure = None;
        app.world_mut()
            .get_mut::<Interaction>(button)
            .unwrap()
            .set_changed();
        app.update();
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(2));
        assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
    }
}
