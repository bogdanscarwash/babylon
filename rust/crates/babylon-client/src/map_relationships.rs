//! Selected-county supply relationships. County centroids anchor aggregates;
//! every elevated connection is schematic, never a physical transport route.

use std::collections::BTreeMap;

use babylon_persistence::ProductionSnapshotV1;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{scene_point, ObserverMapCamera, BASE_HEIGHT, DATA_HEIGHT, MAP_LAYER};
use crate::atlas::CountyAtlas;
use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::map::SelectedCounty;
use crate::observer::{ObservationContext, ObserverSession};
use crate::observer_io::ObserverSet;
use crate::observer_theme as theme;
use crate::observer_ui::{ObserverFrame, ObserverUiState, ObserverViewport};
use crate::production::PrimaryView;
use crate::production_layout::place_label;

const MAX_RELATIONSHIPS: usize = 6;
const CONNECTION_HEIGHT: f32 = BASE_HEIGHT + DATA_HEIGHT + 22.0;

#[derive(Clone)]
struct CountyAnchor {
    index: usize,
    name: String,
    position: Vec3,
}

#[derive(Resource)]
pub(super) struct CountyAnchors(BTreeMap<String, CountyAnchor>);

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
    target_county: usize,
    outbound: bool,
    internal: bool,
}

struct RelationshipProjection {
    rows: Vec<CountyRelationship>,
    total: usize,
    available: bool,
}

fn declared_relations(snapshot: &ProductionSnapshotV1) -> BTreeMap<RelationKey, (String, String)> {
    let mut relations = BTreeMap::new();
    for buyer in &snapshot.sites {
        for input in &buyer.inputs {
            for supplier in &input.supplier_site_ids {
                relations.insert(
                    RelationKey {
                        supplier: supplier.clone(),
                        buyer: buyer.id.clone(),
                        good: input.good_id.clone(),
                        unit: input.unit_id.clone(),
                    },
                    (input.good.clone(), input.unit.clone()),
                );
            }
        }
    }
    for route in &snapshot.routes {
        relations.insert(
            RelationKey {
                supplier: route.supplier_site_id.clone(),
                buyer: route.buyer_site_id.clone(),
                good: route.good_id.clone(),
                unit: route.unit_id.clone(),
            },
            (route.good.clone(), route.unit.clone()),
        );
    }
    relations
}

fn county_label(name: &str) -> &str {
    // Keep the public atlas name; omit only its known Michigan county suffix.
    name.strip_suffix(" County, MI").unwrap_or(name)
}

fn project(
    frame: &ObserverFrame,
    session: &ObserverSession,
    selected: Option<usize>,
    anchors: &CountyAnchors,
) -> RelationshipProjection {
    let snapshot = frame
        .for_session(session)
        .and_then(|frame| frame.production.as_ref());
    let mut result = RelationshipProjection {
        rows: Vec::new(),
        total: 0,
        available: snapshot.is_some(),
    };
    let (Some(snapshot), Some(county)) = (snapshot, anchors.selected(selected)) else {
        return result;
    };
    let sites: BTreeMap<_, _> = snapshot
        .sites
        .iter()
        .map(|site| (site.id.as_str(), site))
        .collect();
    for (key, (good, unit)) in declared_relations(snapshot) {
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
            target_county: if outbound { to.index } else { from.index },
            outbound,
            internal: supplier.county_geoid == buyer.county_geoid,
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
    });
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
        RelationshipLabel { anchor, order },
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
}

fn rebuild(
    mut commands: Commands,
    observation: RelationshipObservation,
    assets: Res<RelationshipAssets>,
    mut scope: ResMut<RelationshipScope>,
    old: Query<Entity, With<RelationshipEntity>>,
) {
    let current = (observation.session.context(), observation.selected.0);
    if scope.0.as_ref() == Some(&current) && !observation.frame.is_changed() {
        return;
    }
    scope.0 = Some(current.clone());
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let projection = project(
        &observation.frame,
        &observation.session,
        observation.selected.0,
        &observation.anchors,
    );
    let heading = if !projection.available {
        "Supply links unavailable in this observation.".to_owned()
    } else if observation.selected.0.is_none() {
        "Select a county to trace its supply relationships.".to_owned()
    } else if projection.total == 0 {
        "No disclosed supply links for this county.".to_owned()
    } else {
        format!(
            "Schematic links: {} of {}\nIN: inputs / OUT: supplies",
            projection.rows.len(),
            projection.total
        )
    };
    commands.spawn((
        label_bundle(heading, None, 0, theme::PAPER),
        Pickable::IGNORE,
    ));
    for (index, relation) in projection.rows.iter().enumerate() {
        let anchor = spawn_connection(&mut commands, &assets, relation);
        let caption = if relation.internal {
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
            index + 1,
            if relation.outbound {
                theme::COPPER
            } else {
                theme::BLUE
            },
        ));
        if relation.internal {
            label.insert(Pickable::IGNORE);
        } else if let Some(selected_county) = observation.selected.0 {
            label.insert((
                Button,
                RelationshipJump {
                    context: current.0.clone(),
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
    project(frame, session, selected, anchors)
        .rows
        .into_iter()
        .find(|row| row.key == jump.key && !row.internal)
        .map(|row| row.target_county)
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
        && !placement.ui.comparison_open
        && placement.ui.disclosure.is_none();
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
        let rect = if active {
            match (
                placement.viewport.0,
                placement.camera.single(),
                placement.windows.single(),
            ) {
                (Some(bounds), Ok((camera, transform)), Ok(window)) => {
                    let size = computed.size() / window.scale_factor();
                    if size.x <= 0.0 || size.y <= 0.0 {
                        None
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
        CampaignId, ObserverEconomySnapshotV1, ObserverVisibilityV1, ProductionInputV1,
        ProductionSiteV1,
    };

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

    fn site(id: &str, county: &str) -> ProductionSiteV1 {
        ProductionSiteV1 {
            id: id.into(),
            county_geoid: county.into(),
            name: format!("{id} county cohort"),
            industry_code: "331".into(),
            observed_employment: None,
            output_good_id: "steel".into(),
            output_unit_id: "kg".into(),
            output_good: "steel".into(),
            output_unit: "kg".into(),
            output_per_batch: 1,
            available_batches: 0,
            planned_batches: None,
            produced_batches: None,
            inventory: Vec::new(),
            inputs: Vec::new(),
            labor: Vec::new(),
        }
    }

    fn fixture() -> (ObserverSession, ObserverFrame, CountyAnchors) {
        let mut session = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        session.ready(3, Some("committed".into()));
        assert!(session.installed(&session.context()));
        let supplier = site("a", "26163");
        let mut buyer = site("b", "26099");
        for (good, unit) in [("steel", "kg"), ("ore", "tonne")] {
            buyer.inputs.push(ProductionInputV1 {
                good_id: good.into(),
                unit_id: unit.into(),
                good: good.into(),
                unit: unit.into(),
                quantity_per_batch: 7,
                on_hand: 2,
                supplier_site_ids: vec![supplier.id.clone()],
            });
        }
        let frame = ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: session.campaign.as_uuid().to_string(),
            resolve_tick: 3,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: Some("committed".into()),
            envelope_digest: None,
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: Some(ProductionSnapshotV1 {
                material_balance: None,
                labor_accounts: Vec::new(),
                scenario_label: "fixture".into(),
                horizon_week: 16,
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
        );
        (session, frame, anchors)
    }

    #[test]
    fn selected_incident_dependencies_keep_goods_units_and_direction_separate() {
        let (session, mut frame, anchors) = fixture();
        let rows = project(&frame, &session, Some(1), &anchors);
        assert_eq!(rows.total, 2);
        assert!(rows
            .rows
            .iter()
            .all(|row| row.outbound && row.target_county == 2));
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
            project(&frame, &session, Some(1), &anchors)
                .rows
                .iter()
                .map(|row| row.key.clone())
                .collect::<Vec<_>>(),
            keys
        );
        let inbound = project(&frame, &session, Some(2), &anchors);
        assert!(inbound
            .rows
            .iter()
            .all(|row| !row.outbound && row.target_county == 1));
        assert_eq!(project(&frame, &session, Some(3), &anchors).total, 0);
    }

    #[test]
    fn compact_labels_preserve_unrecognized_public_names_without_inference() {
        let (session, frame, mut anchors) = fixture();
        anchors.0.get_mut("26163").unwrap().name = "Disclosed district".into();
        assert_eq!(
            project(&frame, &session, Some(1), &anchors).rows[0].caption,
            "Disclosed district -> Macomb\nore | tonne"
        );
        assert_eq!(county_label("Wayne County, NE"), "Wayne County, NE");
    }

    #[test]
    fn missing_disclosed_sites_or_county_anchors_never_create_endpoints() {
        let (session, mut frame, mut anchors) = fixture();
        anchors.0.remove("26099");
        assert_eq!(project(&frame, &session, Some(1), &anchors).total, 0);
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
        assert_eq!(project(&frame, &session, Some(2), &anchors).total, 0);
    }

    #[test]
    fn scope_and_known_capability_clear_relationships_and_refuse_stale_navigation() {
        let (mut session, mut frame, anchors) = fixture();
        let row = project(&frame, &session, Some(1), &anchors).rows.remove(0);
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
        assert!(project(&frame, &session, Some(1), &anchors).rows.is_empty());
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(1), &anchors),
            None
        );
        frame.0.as_mut().unwrap().resolve_tick = 3;
        session.set_perspective(Perspective::PlayerKnowledge);
        assert!(!project(&frame, &session, Some(1), &anchors).available);
        frame.0.as_mut().unwrap().visibility = ObserverVisibilityV1::KnownPreview;
        frame.0.as_mut().unwrap().production = None;
        assert!(!project(&frame, &session, Some(1), &anchors).available);
        assert_eq!(
            jump_target(&jump, &frame, &session, Some(1), &anchors),
            None
        );
        session.set_perspective(Perspective::FullObserver);
        let (_, valid, _) = fixture();
        frame = valid;
        frame.0.as_mut().unwrap().campaign_id = uuid::Uuid::from_u128(9).to_string();
        assert!(project(&frame, &session, Some(1), &anchors).rows.is_empty());
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
        let input =
            frame.0.as_mut().unwrap().production.as_mut().unwrap().sites[1].inputs[0].clone();
        for index in 0..10 {
            let mut next = input.clone();
            next.good_id = format!("good-{index:02}");
            frame.0.as_mut().unwrap().production.as_mut().unwrap().sites[1]
                .inputs
                .push(next);
        }
        let projection = project(&frame, &session, Some(1), &anchors);
        assert_eq!(projection.total, 12);
        assert_eq!(projection.rows.len(), MAX_RELATIONSHIPS);
    }

    #[test]
    fn relationship_button_follows_county_only_in_the_visible_unblocked_map() {
        let (session, frame, anchors) = fixture();
        let key = project(&frame, &session, Some(1), &anchors)
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
