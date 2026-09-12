//! The complete disclosed economy as a geographic, schematic owner network.
//! Display offsets separate county aggregates; they are never factory locations.

use super::*;
use crate::observer_ui::NetworkSector;
use crate::production::ProductionCommand;
use babylon_persistence::{
    production_observation::ProductionSite, production_observation::ProductionSiteRole,
};
use bevy::picking::pointer::PointerButton;
use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum NodeKey {
    Site(String),
    EndBuyers(String),
}

#[derive(Clone, Debug, PartialEq)]
struct NetworkNode {
    key: NodeKey,
    site_id: String,
    county: String,
    sector: NetworkSector,
    position: Vec3,
    caption: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct NetworkLink {
    from: NodeKey,
    to: NodeKey,
    good: String,
    unit: String,
}

#[derive(Default)]
struct NetworkProjection {
    nodes: BTreeMap<NodeKey, NetworkNode>,
    links: BTreeSet<NetworkLink>,
    total_cohorts: usize,
    total_links: usize,
    available: bool,
}

fn site_sector(site: &ProductionSite) -> NetworkSector {
    match site.role {
        ProductionSiteRole::Wholesale => NetworkSector::Wholesale,
        ProductionSiteRole::Retail => NetworkSector::Retail,
        ProductionSiteRole::Production => match site.sector_code.as_str() {
            "11" => NetworkSector::Agriculture,
            "21" => NetworkSector::Mining,
            _ => NetworkSector::Manufacturing,
        },
    }
}

fn node_caption(site: &ProductionSite, snapshot: &ProductionSnapshot) -> String {
    let mut text = format!(
        "{}\n{} · NAICS {}",
        site.name,
        site_sector(site).label(),
        site.industry_code
    );
    for process in &site.processes {
        write!(text, "\n{} / {}", process.output_good, process.output_unit).expect("String write");
    }
    if let Some(work) = snapshot
        .staffing_accounts
        .iter()
        .find(|row| row.site_id == site.id)
    {
        write!(
            text,
            "\n{} employed · {} reserve / Derived",
            work.employed, work.reserve
        )
        .expect("String write");
    }
    text.push_str("\nClick: trace connections · Circuit [P]: accounts");
    text
}

#[allow(
    clippy::cast_precision_loss,
    reason = "The bounded county node rank is only a schematic display offset."
)]
fn node_position(anchor: Vec3, rank: usize) -> Vec3 {
    anchor
        + Vec3::new(
            (rank % 3) as f32 * 12.0 - 12.0,
            0.0,
            (rank / 3) as f32 * 12.0 - 6.0,
        )
}

fn project_network(
    frame: &ObserverFrame,
    session: &ObserverSession,
    anchors: &CountyAnchors,
    sector: NetworkSector,
    good: Option<&crate::map_economy_lens::MaterialGoodKey>,
) -> NetworkProjection {
    let Some(snapshot) = disclosed_snapshot(frame, session) else {
        return NetworkProjection::default();
    };
    let mut result = NetworkProjection {
        available: true,
        ..default()
    };
    let mut sites: Vec<_> = snapshot.sites.iter().collect();
    sites.sort_by(|a, b| {
        (&a.county_geoid, site_sector(a), &a.id).cmp(&(&b.county_geoid, site_sector(b), &b.id))
    });
    let mut county_ranks = BTreeMap::<&str, usize>::new();
    for site in sites {
        let Some(anchor) = anchors.0.get(&site.county_geoid) else {
            continue;
        };
        let rank = county_ranks.entry(&site.county_geoid).or_default();
        let key = NodeKey::Site(site.id.clone());
        result.nodes.insert(
            key.clone(),
            NetworkNode {
                key,
                site_id: site.id.clone(),
                county: site.county_geoid.clone(),
                sector: site_sector(site),
                position: node_position(anchor.position, *rank),
                caption: node_caption(site, snapshot),
            },
        );
        *rank += 1;
    }
    result.total_cohorts = result.nodes.len();
    for (relation, _) in declared_relations(snapshot) {
        let from = NodeKey::Site(relation.supplier);
        let to = NodeKey::Site(relation.buyer);
        if result.nodes.contains_key(&from) && result.nodes.contains_key(&to) {
            result.links.insert(NetworkLink {
                from,
                to,
                good: relation.good,
                unit: relation.unit,
            });
        }
    }
    project_final_demand(snapshot, anchors, &county_ranks, &mut result);
    result.total_links = result.links.len();
    result.links.retain(|link| {
        good.is_none_or(|good| good.good_id == link.good && good.unit_id == link.unit)
            && (sector == NetworkSector::All
                || [&link.from, &link.to].into_iter().any(|key| {
                    result
                        .nodes
                        .get(key)
                        .is_some_and(|node| node.sector == sector)
                }))
    });
    if sector != NetworkSector::All || good.is_some() {
        let connected: BTreeSet<_> = result
            .links
            .iter()
            .flat_map(|link| [&link.from, &link.to])
            .cloned()
            .collect();
        result.nodes.retain(|key, node| {
            connected.contains(key) || (good.is_none() && node.sector == sector)
        });
    }
    result
}

fn project_final_demand(
    snapshot: &ProductionSnapshot,
    anchors: &CountyAnchors,
    county_ranks: &BTreeMap<&str, usize>,
    result: &mut NetworkProjection,
) {
    // End buyers are county demand accounts, not additional firms or households.
    // Draw only an explicit retail order; a buyer marker has no invented stock.
    let mut demand: Vec<_> = snapshot.final_demand_accounts.iter().collect();
    demand.sort_by(|a, b| {
        (&a.county_geoid, &a.good_id, &a.unit_id).cmp(&(&b.county_geoid, &b.good_id, &b.unit_id))
    });
    for account in demand {
        let Some(anchor) = anchors.0.get(&account.county_geoid) else {
            continue;
        };
        let to = NodeKey::EndBuyers(account.county_geoid.clone());
        let mut orders: Vec<_> = account.orders.iter().collect();
        orders.sort_by(|a, b| {
            (&a.retailer_site_id, &a.order_id).cmp(&(&b.retailer_site_id, &b.order_id))
        });
        for order in orders {
            let from = NodeKey::Site(order.retailer_site_id.clone());
            if !result.nodes.contains_key(&from) {
                continue;
            }
            let rank = county_ranks
                .get(account.county_geoid.as_str())
                .copied()
                .unwrap_or(0);
            result.nodes.entry(to.clone()).or_insert_with(|| NetworkNode {
                key: to.clone(), site_id: order.retailer_site_id.clone(),
                county: account.county_geoid.clone(), sector: NetworkSector::EndBuyers,
                position: node_position(anchor.position, rank),
                caption: format!("{} · end buyers\nFinite orders · delivery, not consumption\nClick: trace retail · Circuit [P]: fulfillment", county_label(&anchor.name)),
            });
            result.links.insert(NetworkLink {
                from,
                to: to.clone(),
                good: account.good_id.clone(),
                unit: account.unit_id.clone(),
            });
        }
    }
}

#[derive(Component)]
struct EconomyEntity;
#[derive(Component, Clone)]
struct EconomyNode {
    context: ObservationContext,
    node: NetworkNode,
}
#[derive(Component)]
struct EconomyLabel {
    key: NodeKey,
    position: Vec3,
}
#[derive(Component)]
struct NetworkLegend;
#[derive(Resource, Default)]
struct HoveredNode(Option<NodeKey>);
#[derive(Resource, Default)]
struct NetworkScope(Option<ObservationContext>);

fn sector_color(sector: NetworkSector) -> Color {
    match sector {
        NetworkSector::Agriculture => Color::srgb_u8(151, 190, 128),
        NetworkSector::Mining => theme::COPPER,
        NetworkSector::Manufacturing => theme::BLUE,
        NetworkSector::Wholesale => Color::srgb_u8(185, 155, 217),
        NetworkSector::Retail => theme::YELLOW,
        NetworkSector::All | NetworkSector::EndBuyers => theme::PAPER,
    }
}

#[derive(Resource)]
struct NetworkAssets {
    marker: Handle<Mesh>,
    materials: BTreeMap<NetworkSector, Handle<StandardMaterial>>,
    links: BTreeMap<NetworkSector, Handle<StandardMaterial>>,
    selected: Handle<StandardMaterial>,
}

fn setup_network(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = |color| StandardMaterial {
        base_color: color,
        unlit: true,
        ..default()
    };
    commands.insert_resource(NetworkAssets {
        marker: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        materials: NetworkSector::GROUPS
            .into_iter()
            .map(|sector| (sector, materials.add(material(sector_color(sector)))))
            .collect(),
        links: NetworkSector::GROUPS
            .into_iter()
            .map(|sector| {
                (
                    sector,
                    materials.add(material(sector_color(sector).mix(&theme::INK, 0.4))),
                )
            })
            .collect(),
        selected: materials.add(material(theme::PAPER)),
    });
}

fn heading(projection: &NetworkProjection, filter: NetworkSector) -> String {
    if !projection.available {
        return "ECONOMY NETWORK\nUnavailable in this observation".into();
    }
    let cohorts = projection
        .nodes
        .keys()
        .filter(|key| matches!(key, NodeKey::Site(_)))
        .count();
    let counties = projection
        .nodes
        .values()
        .map(|node| &node.county)
        .collect::<BTreeSet<_>>()
        .len();
    format!("ECONOMY NETWORK\n{cohorts} cohorts · {counties} counties\n{} / {} commodity links · {}\nArrows: supplier → buyer, not roads\nDots: county aggregates, not factories\nClick a dot to trace · Map lens to filter", projection.links.len(), projection.total_links, filter.label())
}

fn spawn_legend(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(8)),
                row_gap: px(3),
                ..default()
            },
            BackgroundColor(theme::INK.with_alpha(0.9)),
            ZIndex(5),
            Visibility::Hidden,
            EconomyEntity,
            NetworkLegend,
            Pickable::IGNORE,
            DeclaredSurface::new(SurfaceId::ObserverProduction),
        ))
        .with_children(|legend| {
            for sector in NetworkSector::GROUPS {
                legend.spawn((
                    Text::new(format!("■ {}", sector.label())),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(sector_color(sector)),
                    crate::observer_ui::ObserverFontRole::Body,
                    Pickable::IGNORE,
                ));
            }
        });
}

fn rebuild_network(
    mut commands: Commands,
    observation: RelationshipObservation,
    assets: Res<NetworkAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut scope: ResMut<NetworkScope>,
    mut hovered: ResMut<HoveredNode>,
    old: Query<(Entity, Option<&Mesh3d>), With<EconomyEntity>>,
) {
    let context = observation.session.context();
    if scope.0.as_ref() == Some(&context)
        && !observation.frame.is_changed()
        && !observation.navigation.is_changed()
        && !observation.ui.is_changed()
        && !observation.session.is_changed()
        && !observation.view.is_changed()
        && !observation.selected.is_changed()
    {
        return;
    }
    scope.0 = Some(context.clone());
    hovered.0 = None;
    for (entity, mesh) in &old {
        if let Some(mesh) = mesh {
            if mesh.0 != assets.marker {
                meshes.remove(mesh.0.id());
            }
        }
        commands.entity(entity).despawn();
    }
    if *observation.view != PrimaryView::Map
        || observation.ui.road_layer != RoadLayer::EconomyNetwork
    {
        return;
    }
    let good = match &observation.ui.lens {
        crate::map_economy_lens::MapLens::Material { good, .. } => good.as_ref(),
        _ => None,
    };
    let mut projection = project_network(
        &observation.frame,
        &observation.session,
        &observation.anchors,
        observation.ui.network_sector,
        good,
    );
    if matches!(
        observation.ui.lens,
        crate::map_economy_lens::MapLens::Relationships
    ) {
        for node in projection.nodes.values_mut() {
            node.position.y = BASE_HEIGHT + 8.0;
        }
    }
    commands
        .spawn(label_bundle(
            heading(&projection, observation.ui.network_sector),
            None,
            0,
            theme::PAPER,
        ))
        .remove::<RelationshipEntity>()
        .insert((EconomyEntity, Pickable::IGNORE));
    if projection.available {
        spawn_legend(&mut commands);
    }
    spawn_network_connections(
        &mut commands,
        &assets,
        &mut meshes,
        &projection,
        observation.navigation.selected_site.as_deref(),
    );
    spawn_network_nodes(
        &mut commands,
        &assets,
        projection,
        &context,
        observation.navigation.selected_site.as_deref(),
    );
}

fn spawn_network_connections(
    commands: &mut Commands,
    assets: &NetworkAssets,
    meshes: &mut Assets<Mesh>,
    projection: &NetworkProjection,
    selected_site: Option<&str>,
) {
    let mut connections = BTreeMap::<(NodeKey, NodeKey), NetworkSector>::new();
    for link in &projection.links {
        connections.insert(
            (link.from.clone(), link.to.clone()),
            projection.nodes[&link.from].sector,
        );
    }
    let mut batches = BTreeMap::<(NetworkSector, bool), RoadSegments>::new();
    for (index, ((from_key, to_key), sector)) in connections.into_iter().enumerate() {
        let from = projection.nodes[&from_key].position;
        let to = projection.nodes[&to_key].position;
        let focused = [&from_key, &to_key]
            .into_iter()
            .any(|key| matches!(key, NodeKey::Site(id) if selected_site == Some(id.as_str())));
        let points = [from, from.lerp(to, 0.5) + Vec3::Y * 4.0, to];
        let batch = batches.entry((sector, focused)).or_default();
        for (part, pair) in points.windows(2).enumerate() {
            // These integer keys only identify display segments, never infrastructure.
            let id = i64::try_from(index).expect("bounded disclosed network");
            batch.insert(
                RoadSegment([[id, i64::try_from(part).unwrap()], [id, 3]]),
                [pair[0], pair[1]],
            );
        }
        let direction = (to - points[1]).normalize_or_zero();
        let tip = points[1].lerp(to, 0.78);
        let side = direction.cross(Vec3::Y).normalize_or_zero() * 2.0;
        for (part, wing) in [side, -side].into_iter().enumerate() {
            let id = i64::try_from(index).expect("bounded disclosed network");
            batch.insert(
                RoadSegment([[id, 4 + i64::try_from(part).unwrap()], [id, 7]]),
                [tip - direction * 4.0 + wing, tip],
            );
        }
    }
    for ((sector, focused), segments) in batches {
        if let Some(mesh) = road_mesh(&segments, if focused { 1.6 } else { 0.7 }) {
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(if focused {
                    assets.selected.clone()
                } else {
                    assets.links[&sector].clone()
                }),
                Transform::default(),
                bevy::camera::visibility::RenderLayers::layer(MAP_LAYER),
                Pickable::IGNORE,
                EconomyEntity,
                DeclaredSurface::new(SurfaceId::ObserverProduction),
            ));
        }
    }
}
fn spawn_network_nodes(
    commands: &mut Commands,
    assets: &NetworkAssets,
    projection: NetworkProjection,
    context: &ObservationContext,
    selected_site: Option<&str>,
) {
    for node in projection.nodes.into_values() {
        let focused = selected_site == Some(node.site_id.as_str());
        let scale = if focused { 6.0 } else { 4.2 };
        commands
            .spawn((
                Mesh3d(assets.marker.clone()),
                MeshMaterial3d(if focused {
                    assets.selected.clone()
                } else {
                    assets.materials[&node.sector].clone()
                }),
                Transform::from_translation(node.position).with_scale(Vec3::splat(scale)),
                bevy::camera::visibility::RenderLayers::layer(MAP_LAYER),
                EconomyEntity,
                EconomyNode {
                    context: context.clone(),
                    node: node.clone(),
                },
                DeclaredSurface::new(SurfaceId::ObserverProduction),
            ))
            .observe(hover_node)
            .observe(leave_node)
            .observe(select_node);
        commands
            .spawn(label_bundle(
                node.caption,
                Some(node.position),
                1,
                sector_color(node.sector),
            ))
            .remove::<(RelationshipEntity, RelationshipLabel)>()
            .insert((
                EconomyEntity,
                Pickable::IGNORE,
                EconomyLabel {
                    key: node.key,
                    position: node.position,
                },
            ));
    }
}

fn usable(ui: &ObserverUiState, view: PrimaryView) -> bool {
    view == PrimaryView::Map
        && ui.road_layer == RoadLayer::EconomyNetwork
        && !ui.menu_open
        && !ui.splash_visible
        && !ui.comparison_open
        && ui.disclosure.is_none()
}

fn hover_node(
    event: On<Pointer<Over>>,
    nodes: Query<&EconomyNode>,
    mut hovered: ResMut<HoveredNode>,
    ui: Res<ObserverUiState>,
    view: Res<PrimaryView>,
    session: Res<ObserverSession>,
) {
    if !usable(&ui, *view) {
        return;
    }
    if let Ok(node) = nodes.get(event.entity) {
        if session.accepts(&node.context) {
            hovered.0 = Some(node.node.key.clone());
        }
    }
}
fn leave_node(
    event: On<Pointer<Out>>,
    nodes: Query<&EconomyNode>,
    mut hovered: ResMut<HoveredNode>,
) {
    if nodes
        .get(event.entity)
        .is_ok_and(|node| hovered.0.as_ref() == Some(&node.node.key))
    {
        hovered.0 = None;
    }
}
fn select_node(
    event: On<Pointer<Click>>,
    nodes: Query<&EconomyNode>,
    mut commands: MessageWriter<ProductionCommand>,
    ui: Res<ObserverUiState>,
    view: Res<PrimaryView>,
    session: Res<ObserverSession>,
) {
    if event.button != PointerButton::Primary || !usable(&ui, *view) {
        return;
    }
    if let Ok(node) = nodes.get(event.entity) {
        if session.accepts(&node.context) {
            commands.write(ProductionCommand::Focus {
                site_id: node.node.site_id.clone(),
                context: node.context.clone(),
            });
        }
    }
}

#[derive(SystemParam)]
struct NetworkLabelPlacement<'w, 's> {
    observation: RelationshipObservation<'w>,
    hovered: Res<'w, HoveredNode>,
    viewport: Res<'w, ObserverViewport>,
    scale: Res<'w, UiScale>,
    windows: Query<'w, 's, &'static Window, With<bevy::window::PrimaryWindow>>,
    camera: Query<'w, 's, (&'static Camera, &'static Transform), With<ObserverMapCamera>>,
    labels: Query<
        'w,
        's,
        (
            &'static EconomyLabel,
            &'static ComputedNode,
            &'static mut Node,
            &'static mut Visibility,
        ),
    >,
}

fn place_network_labels(mut placement: NetworkLabelPlacement) {
    let active = usable(&placement.observation.ui, *placement.observation.view);
    for (label, computed, mut node, mut visibility) in &mut placement.labels {
        let focused = match &label.key {
            NodeKey::Site(id) => {
                placement.observation.navigation.selected_site.as_ref() == Some(id)
            }
            NodeKey::EndBuyers(_) => false,
        };
        let show = active
            && (placement.hovered.0.as_ref() == Some(&label.key)
                || (placement.hovered.0.is_none() && focused));
        let rect = if show {
            match (
                placement.viewport.0,
                placement.camera.single(),
                placement.windows.single(),
            ) {
                (Some(bounds), Ok((camera, transform)), Ok(window)) => camera
                    .world_to_viewport(&GlobalTransform::from(*transform), label.position)
                    .ok()
                    .and_then(|point| {
                        place_label(point, bounds, computed.size() / window.scale_factor(), &[])
                    }),
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
            node.left = px(rect.min.x / placement.scale.0);
            node.top = px(rect.min.y / placement.scale.0);
        }
    }
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<NetworkScope>()
        .init_resource::<HoveredNode>()
        .add_systems(Startup, setup_network)
        .add_systems(Update, rebuild_network.in_set(ObserverSet::Paint))
        .add_systems(
            Update,
            place_network_labels.after(super::super::sync_camera),
        )
        .add_systems(Update, place_legend.after(super::super::sync_camera));
}

fn place_legend(
    ui: Res<ObserverUiState>,
    view: Res<PrimaryView>,
    viewport: Res<ObserverViewport>,
    scale: Res<UiScale>,
    mut legends: Query<(&mut Node, &mut Visibility), With<NetworkLegend>>,
) {
    for (mut node, mut visibility) in &mut legends {
        let bounds = viewport.0.filter(|_| usable(&ui, *view));
        visibility.set_if_neq(if bounds.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
        if let Some(bounds) = bounds {
            node.left = px((bounds.min.x + 8.0) / scale.0);
            node.top = px((bounds.min.y + 8.0) / scale.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::fixture;
    use super::*;
    use crate::map_economy_lens::MaterialGoodKey;

    #[test]
    fn network_keeps_isolated_cohorts_all_goods_and_retail_endpoints_without_inventing_routes() {
        let (session, mut frame, anchors) = fixture();
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        snapshot.sites[1].role = ProductionSiteRole::Retail;
        snapshot.sites[1].sector_code = "44-45".into();
        snapshot.final_demand_accounts.push(
            babylon_persistence::production_observation::ProductionFinalDemandAccount {
                demand_principal_id: "demand".into(),
                county_geoid: "26099".into(),
                good_id: "steel".into(),
                unit_id: "kg".into(),
                good: "Steel".into(),
                unit: "kg".into(),
                ordered: 10,
                fulfilled: 3,
                outstanding: 7,
                retail_stock_on_hand: 2,
                retailer_site_ids: vec!["b".into()],
                orders: vec![
                    babylon_persistence::production_observation::ProductionFinalDemandOrder {
                        order_id: "final".into(),
                        retailer_site_id: "b".into(),
                        ordered: 10,
                        fulfilled: 3,
                        outstanding: 7,
                    },
                ],
                completed: None,
            },
        );
        let full = project_network(&frame, &session, &anchors, NetworkSector::All, None);
        assert_eq!(full.total_cohorts, 3);
        assert_eq!(full.nodes.len(), 4);
        assert_eq!(
            full.links.len(),
            3,
            "two goods and one local final-demand relationship"
        );
        assert!(
            full.nodes.contains_key(&NodeKey::Site("unrelated".into())),
            "isolated producers stay visible"
        );
        assert!(full
            .links
            .iter()
            .any(|link| link.to == NodeKey::EndBuyers("26099".into())));
        let only = project_network(
            &frame,
            &session,
            &anchors,
            NetworkSector::Retail,
            Some(&MaterialGoodKey {
                good_id: "steel".into(),
                unit_id: "kg".into(),
            }),
        );
        assert_eq!(only.links.len(), 2);
        assert_eq!(
            only.nodes.len(),
            3,
            "retail keeps its source and its end buyers"
        );
        assert!(only
            .links
            .iter()
            .all(|link| link.good == "steel" && link.unit == "kg"));
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        snapshot.sites.reverse();
        snapshot.final_demand_accounts.reverse();
        let permuted = project_network(&frame, &session, &anchors, NetworkSector::All, None);
        assert_eq!(full.nodes, permuted.nodes);
        assert_eq!(full.links, permuted.links);
        assert!(
            snapshot_routes_empty(&frame),
            "schematic end-buyer edges never manufacture a transport route"
        );
    }

    fn snapshot_routes_empty(frame: &ObserverFrame) -> bool {
        frame
            .0
            .as_ref()
            .unwrap()
            .production
            .as_ref()
            .unwrap()
            .routes
            .is_empty()
    }

    #[test]
    fn network_refuses_stale_loading_and_restricted_observations() {
        for invalid in ["stale", "loading", "preview"] {
            let (mut session, mut frame, anchors) = fixture();
            assert!(
                project_network(&frame, &session, &anchors, NetworkSector::All, None).available
            );
            match invalid {
                "stale" => frame.0.as_mut().unwrap().resolve_tick = 2,
                "loading" => session.phase = crate::observer::SessionPhase::Loading,
                "preview" => {
                    session.set_perspective(crate::observer::Perspective::PlayerKnowledge);
                    frame.0.as_mut().unwrap().visibility =
                        babylon_persistence::observer_reader::ObserverVisibility::KnownPreview;
                }
                _ => unreachable!(),
            }
            let hidden = project_network(&frame, &session, &anchors, NetworkSector::All, None);
            assert!(!hidden.available);
            assert!(hidden.nodes.is_empty() && hidden.links.is_empty());
        }
    }
}
