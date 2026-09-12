use super::readings::{describe_flow, describe_freight, describe_material_balance, describe_work};
use super::*;
use crate::atlas::CountyAtlas;
use crate::map::SelectedCounty;
use babylon_persistence::{
    identity::CampaignId, observer_reader::ObserverEconomySnapshot,
    observer_reader::ObserverVisibility, production_observation::ProductionInput,
    production_observation::ProductionRoute,
};

fn site(id: &str, suppliers: &[&str]) -> ProductionSite {
    ProductionSite {
        id: id.into(),
        county_geoid: "26163".into(),
        name: format!("Cohort {id}"),
        industry_code: "331".into(),
        observed_employment: Some(20),
        inventory: Vec::new(),
        role: babylon_persistence::production_observation::ProductionSiteRole::Production,
        sector_code: "31-33".into(),
        processes: vec![
            babylon_persistence::production_observation::ProductionProcess {
                id: "fixture-process".into(),
                name: "Fixture process".into(),
                output_good_id: "a".repeat(64),
                output_unit_id: "b".repeat(64),
                output_good: "steel".into(),
                output_unit: "kg".into(),
                output_per_batch: 10,
                available_batches: 8,
                planned_batches: Some(8),
                produced_batches: Some(7),
                labor: Vec::new(),
                inputs: vec![ProductionInput {
                    good_id: "a".repeat(64),
                    unit_id: "b".repeat(64),
                    good: "input".into(),
                    unit: "kg".into(),
                    quantity_per_batch: 1,
                    on_hand: 20,
                    supplier_site_ids: suppliers.iter().map(|id| (*id).into()).collect(),
                }],
            },
        ],
    }
}

fn snapshot() -> ProductionSnapshot {
    ProductionSnapshot {
        content_authority_sha256: "a".repeat(64),
        road_source: None,
        physical_edges: Vec::new(),
        merchant_handling_accounts: Vec::new(),
        final_demand_accounts: Vec::new(),
        freight_capacity_accounts: Vec::new(),
        material_balance: None,
        labor_accounts: Vec::new(),
        staffing_accounts: Vec::new(),
        observed_contexts: Vec::new(),
        process_attributions: Vec::new(),
        scenario_label: "Navigation fixture".into(),
        horizon_period: 8,
        sites: vec![
            site("a", &[]),
            site("b", &["a", "withheld"]),
            site("c", &["b"]),
        ],
        routes: vec![ProductionRoute {
            physical_edge_ids: Vec::new(),
            distance_mm: None,
            transport_kind:
                babylon_persistence::production_observation::ProductionRouteTransport::Staged,
            grams_per_unit: 1000,
            stages: Vec::new(),
            id: "a-b".into(),
            supplier_site_id: "a".into(),
            buyer_site_id: "b".into(),
            good_id: "a".repeat(64),
            unit_id: "b".repeat(64),
            good: "steel".into(),
            unit: "kg".into(),
            travel_periods: 1,
            ordered: 20,
            shipped: 10,
            delivered: 10,
            lost: 0,
            realized: 10,
            backlog: 10,
        }],
        freight: Vec::new(),
        events: Vec::new(),
        provenance: Vec::new(),
    }
}

#[test]
fn focused_circuit_pages_six_incident_groups_and_keeps_unrelated_owners_out() {
    let mut snapshot = snapshot();
    snapshot.sites = vec![site("center", &[])];
    for index in 0..14 {
        snapshot
            .sites
            .push(site(&format!("buyer-{index:02}"), &["center"]));
    }
    snapshot.sites.push(site("unrelated", &[]));
    snapshot.routes.clear();
    let first = ProductionLayout::focused(&snapshot, Some("center"), 0);
    let second = ProductionLayout::focused(&snapshot, Some("center"), 1);
    assert_eq!(first.positions.len(), 7);
    assert_eq!(second.positions.len(), 7);
    assert_eq!(first.links.len(), 6);
    assert_eq!(second.links.len(), 6);
    assert!(first.positions.contains_key("center"));
    assert!(!first.positions.contains_key("unrelated"));
    assert!(first
        .positions
        .keys()
        .filter(|id| id.as_str() != "center")
        .all(|id| !second.positions.contains_key(id)));
    snapshot.sites.reverse();
    let permuted = ProductionLayout::focused(&snapshot, Some("center"), 0);
    assert_eq!(first.positions, permuted.positions);
    assert_eq!(first.links, permuted.links);
    assert!(ProductionLayout::focused(&snapshot, Some("withheld"), 0)
        .positions
        .is_empty());
}

#[test]
fn reading_headline_uses_exact_output_identity_and_keeps_absence_distinct_from_zero() {
    use babylon_persistence::{CompletedMaterialBalance, ProductionMaterialBalanceRow};
    let mut snapshot = snapshot();
    let mut selected = snapshot.sites[0].clone();
    selected.processes[0].planned_batches = None;
    selected.processes[0].produced_batches = None;
    let foundation = reading_headline(&selected, &snapshot, 0);
    assert!(foundation.contains("Foundation / Designed"));
    assert!(foundation.contains("no committed production"));
    assert!(foundation.contains("Modeled workforce not disclosed"));
    assert!(!foundation.contains("0 employed"));
    snapshot.material_balance = Some(CompletedMaterialBalance {
        period: 5,
        rows: vec![ProductionMaterialBalanceRow {
            local_received: 0,
            local_transferred: 0,
            final_demand_fulfilled: 0,
            site_id: selected.id.clone(),
            good_id: selected.processes[0].output_good_id.clone(),
            unit_id: "another-unit".into(),
            good: selected.processes[0].output_good.clone(),
            unit: "other unit".into(),
            opening: 0,
            arrivals: 0,
            produced: 999,
            consumed: 0,
            dispatched: 0,
            closing: 999,
        }],
    });
    assert!(!reading_headline(&selected, &snapshot, 5).contains("999"));
    let row = &mut snapshot.material_balance.as_mut().unwrap().rows[0];
    row.unit_id
        .clone_from(&selected.processes[0].output_unit_id);
    row.unit.clone_from(&selected.processes[0].output_unit);
    row.produced = 0;
    row.closing = 0;
    snapshot
        .staffing_accounts
        .push(staffing_account(&selected.id));
    let completed = reading_headline(&selected, &snapshot, 5);
    assert!(completed.contains(&format!(
        "0 {} produced / Derived",
        selected.processes[0].output_unit
    )));
    assert!(completed.contains("2 employed · 2 reserve / Derived"));
    assert!(!completed.contains("Foundation"));
}

#[test]
fn stock_readings_keep_units_and_subjects_separate_and_do_not_invent_foundation_flows() {
    use babylon_persistence::{CompletedMaterialBalance, ProductionMaterialBalanceRow};

    let mut snapshot = snapshot();
    let selected = snapshot.sites[0].clone();
    let mut value = String::new();
    describe_material_balance(&mut value, &selected, &snapshot);
    assert!(value.contains("No completed stock-movement account"));
    assert!(!value.contains("Opened 0"));
    let kilograms = ProductionMaterialBalanceRow {
        local_received: 0,
        local_transferred: 0,
        final_demand_fulfilled: 0,
        site_id: selected.id.clone(),
        good_id: "ore".into(),
        unit_id: "kg".into(),
        good: "Ore".into(),
        unit: "kg".into(),
        opening: 10,
        arrivals: 5,
        produced: 4,
        consumed: 3,
        dispatched: 6,
        closing: 10,
    };
    let tonnes = ProductionMaterialBalanceRow {
        unit_id: "tonne".into(),
        unit: "tonne".into(),
        opening: 1,
        arrivals: 2,
        produced: 0,
        consumed: 0,
        dispatched: 0,
        closing: 3,
        ..kilograms.clone()
    };
    let unrelated = ProductionMaterialBalanceRow {
        local_received: 0,
        local_transferred: 0,
        final_demand_fulfilled: 0,
        site_id: "b".into(),
        good: "Unrelated stock".into(),
        ..kilograms.clone()
    };
    snapshot.material_balance = Some(CompletedMaterialBalance {
        period: 5,
        rows: vec![kilograms, tonnes, unrelated],
    });
    value.clear();
    describe_material_balance(&mut value, &selected, &snapshot);
    assert!(value.contains("STOCK MOVEMENT / PERIOD 5"));
    assert!(value.contains(
        "Ore / kg\nOpened 10 + arrived 5 + produced 4\n= consumed 3 + dispatched 6 + closed 10"
    ));
    assert!(value.contains(
        "Ore / tonne\nOpened 1 + arrived 2 + produced 0\n= consumed 0 + dispatched 0 + closed 3"
    ));
    assert!(!value.contains("Unrelated stock"));
    value.clear();
    describe_material_balance(&mut value, &snapshot.sites[2], &snapshot);
    assert!(value.contains("No stock-movement account disclosed for this subject"));
    assert!(!value.contains("Opened"));
}

#[test]
fn merchant_reading_has_no_fake_production_and_separates_local_goods_from_arrivals() {
    use babylon_persistence::{
        production_observation::ProductionRouteTransport,
        production_observation::ProductionSiteRole, CompletedMaterialBalance,
        ProductionMaterialBalanceRow,
    };
    let mut snapshot = snapshot();
    let merchant = &mut snapshot.sites[1];
    merchant.role = ProductionSiteRole::Retail;
    merchant.processes.clear();
    snapshot.routes[0].transport_kind = ProductionRouteTransport::Local;
    snapshot.routes[0].travel_periods = 0;
    snapshot.material_balance = Some(CompletedMaterialBalance {
        period: 1,
        rows: vec![ProductionMaterialBalanceRow {
            site_id: "b".into(),
            good_id: "meal".into(),
            unit_id: "kg".into(),
            good: "Meal".into(),
            unit: "kg".into(),
            opening: 10,
            arrivals: 0,
            local_received: 2,
            produced: 0,
            consumed: 0,
            dispatched: 0,
            local_transferred: 3,
            final_demand_fulfilled: 4,
            closing: 5,
        }],
    });
    let flow = describe_flow(&snapshot.sites[1], &snapshot);
    assert!(flow.contains("Retail / delivery to final demand"));
    assert!(!flow.contains("batch"));
    assert!(flow.contains("arrived 0 + received locally 2"));
    assert!(flow.contains("transferred locally 3 + final demand 4 + closed 5"));
    let freight = describe_freight(&snapshot.sites[1], &snapshot);
    assert!(freight.contains("Local inter-owner transfer"));
    assert!(!freight.contains("periods travel"));
}

#[test]
fn merchant_handling_reading_uses_exact_kilograms_without_changing_work_hours() {
    use babylon_persistence::{
        production_observation::CompletedProductionMerchantHandling,
        production_observation::ProductionMerchantHandlingAccount,
        production_observation::ProductionSiteRole,
    };
    let mut snapshot = snapshot();
    snapshot.sites[1].role = ProductionSiteRole::Retail;
    snapshot.sites[1].processes.clear();
    snapshot.merchant_handling_accounts = vec![ProductionMerchantHandlingAccount {
        site_id: "b".into(),
        capacity_id: "merchant-handling".into(),
        labor_unit_id: "hours".into(),
        coefficients: Vec::new(),
        completed: Some(CompletedProductionMerchantHandling {
            period: 1,
            needed_hours: 8,
            used_hours: 3,
            handled_grams: 160_001,
            orders: Vec::new(),
        }),
    }];
    let reading = describe_work(&snapshot.sites[1], &snapshot);
    assert!(reading.contains("Period 1: 160.001 kg handled · 3 / 8 labor-hours used / needed"));
    assert!(
        reading.contains("Handling moves existing goods; it does not create productive output.")
    );
    assert!(!describe_work(&snapshot.sites[0], &snapshot).contains("MERCHANT HANDLING"));
}

#[test]
fn inspector_separates_committed_work_time_from_next_opening_and_other_sites() {
    use babylon_persistence::{
        production_observation::CompletedProductionLabor,
        production_observation::ProductionLaborAccount,
    };

    let mut snapshot = snapshot();
    snapshot.labor_accounts = vec![
        ProductionLaborAccount {
            site_id: "a".into(),
            unit_id: "hours".into(),
            unit: "labor-hours".into(),
            next_opening_period: 6,
            next_opening_available: 160,
            completed: Some(CompletedProductionLabor {
                handling_needed: 0,
                handling_used: 0,
                period: 5,
                opening: 120,
                planned: 100,
                used: 80,
                unused: 40,
            }),
        },
        ProductionLaborAccount {
            site_id: "b".into(),
            unit_id: "other-hours".into(),
            unit: "other site's private work time".into(),
            next_opening_period: 6,
            next_opening_available: 987,
            completed: None,
        },
    ];
    let text = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(text.contains("COMMITTED WORK TIME / PERIOD 5 / DERIVED"));
    assert!(text.contains("80 used + 40 unused = 120 available"));
    assert!(text.contains("Production planned: 100 labor-hours"));
    assert!(text.contains("Next opening (period 6): 160 labor-hours (Derived)"));
    assert!(!text.contains("private work time"));
    assert!(!text.contains("987"));
    assert!(text.contains("Time accounts do not measure job losses."));
}

#[test]
fn foundation_labor_account_does_not_invent_a_completed_work_period() {
    use babylon_persistence::production_observation::ProductionLaborAccount;

    let mut snapshot = snapshot();
    snapshot.labor_accounts = vec![ProductionLaborAccount {
        site_id: "a".into(),
        unit_id: "hours".into(),
        unit: "labor-hours".into(),
        next_opening_period: 1,
        next_opening_available: 120,
        completed: None,
    }];
    let text = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(!text.contains("COMMITTED WORK TIME"));
    assert!(text.contains("Next opening (period 1): 120 labor-hours (Derived)"));
}

fn staffing_account(
    site_id: &str,
) -> babylon_persistence::production_observation::ProductionStaffingAccount {
    use babylon_persistence::{
        production_observation::CompletedProductionStaffing,
        production_observation::ProductionStaffingAccount,
        production_observation::ProductionStaffingSubject,
    };
    ProductionStaffingAccount {
        pool_id: format!("pool-{site_id}"),
        site_id: site_id.into(),
        unit_id: "labor-hours".into(),
        subject: ProductionStaffingSubject {
            scenario: "fixture".into(),
            local_name: format!("workers-{site_id}"),
        },
        hours_per_person: 40,
        labor_force: 4,
        employed: 2,
        reserve: 2,
        previous_unretained_hours: 40,
        next_opening_period: 6,
        next_opening_hours: 80,
        completed: Some(CompletedProductionStaffing {
            period: 5,
            opening_employed: 4,
            opening_reserve: 0,
            previous_unretained_hours: 80,
            current_unretained_hours: 40,
            retained_hours: 80,
            target_employed: 2,
            hires: 0,
            separations: 2,
        }),
    }
}

#[test]
fn workforce_readings_use_exact_people_and_retention_for_only_the_selected_site() {
    let mut snapshot = snapshot();
    let mut unrelated = staffing_account("b");
    unrelated.employed = 987;
    snapshot.staffing_accounts = vec![staffing_account("a"), unrelated];
    snapshot.labor_accounts.push(
        babylon_persistence::production_observation::ProductionLaborAccount {
            site_id: "a".into(),
            unit_id: "labor-hours".into(),
            unit: "labor-hours".into(),
            next_opening_period: 6,
            next_opening_available: 80,
            completed: Some(
                babylon_persistence::production_observation::CompletedProductionLabor {
                    handling_needed: 0,
                    handling_used: 0,
                    period: 5,
                    opening: 160,
                    planned: 40,
                    used: 40,
                    unused: 120,
                },
            ),
        },
    );
    let text = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(text.contains("MODELED WORKFORCE / DERIVED"));
    assert!(text.contains("2 employed + 2 reserve = 4 people"));
    assert!(text.contains("40 hours per person / period (Designed)"));
    assert!(text.contains("STAFFING / PERIOD 5"));
    assert!(text.contains("Opening: 4 employed, 0 reserve"));
    assert!(text.contains("Hires: 0 | separations: 2 | target: 2 employed"));
    assert!(text.contains("Work request: 40 hours | prior period: 80 hours"));
    assert!(text.contains("One-period retention: 80 hours"));
    assert!(text.contains("Next opening (period 6): 80 labor-hours (Derived)"));
    assert_eq!(text.matches("Next opening").count(), 1);
    assert!(text.contains("Observed QCEW jobs are separate; these accounts record no payments."));
    assert!(!text.contains("987"));
    assert!(!text.contains("workers-b"));
}

#[test]
fn workforce_foundation_absence_and_zero_completed_flows_remain_distinct() {
    let mut snapshot = snapshot();
    let mut account = staffing_account("a");
    account.next_opening_period = 1;
    account.completed = None;
    snapshot.staffing_accounts.push(account);
    snapshot.labor_accounts.push(
        babylon_persistence::production_observation::ProductionLaborAccount {
            site_id: "a".into(),
            unit_id: "labor-hours".into(),
            unit: "labor-hours".into(),
            next_opening_period: 1,
            next_opening_available: 80,
            completed: None,
        },
    );
    let foundation = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(foundation.contains("Opening workforce; no completed staffing period."));
    assert!(foundation.contains("MODELED WORKFORCE / DESIGNED"));
    assert!(!foundation.contains("MODELED WORKFORCE / DERIVED"));
    assert!(!foundation.contains("Hires:"));
    assert!(!foundation.contains("STAFFING / PERIOD"));
    assert!(foundation.contains("Next opening (period 1): 80 labor-hours (Derived)"));
    assert_eq!(foundation.matches("Next opening").count(), 1);
    let missing = describe(
        &snapshot.sites[2],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(missing.contains("No workforce account disclosed for this subject."));
    assert!(!missing.contains("0 employed"));
    let completed = staffing_account("a").completed.unwrap();
    snapshot.staffing_accounts[0].completed = Some(
        babylon_persistence::production_observation::CompletedProductionStaffing {
            opening_employed: 2,
            opening_reserve: 2,
            hires: 0,
            separations: 0,
            ..completed
        },
    );
    snapshot.staffing_accounts[0].next_opening_period = 6;
    snapshot.labor_accounts[0].next_opening_period = 6;
    snapshot.labor_accounts[0].completed = Some(
        babylon_persistence::production_observation::CompletedProductionLabor {
            handling_needed: 0,
            handling_used: 0,
            period: 5,
            opening: 80,
            planned: 40,
            used: 40,
            unused: 40,
        },
    );
    let quiet = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Work,
    );
    assert!(quiet.contains("Hires: 0 | separations: 0"));
    assert!(!quiet.contains("no completed staffing period"));
    assert!(quiet.contains("Next opening (period 6): 80 labor-hours (Derived)"));
    assert_eq!(quiet.matches("Next opening").count(), 1);
}

fn attributed_snapshot() -> ProductionSnapshot {
    use babylon_persistence::{
        production_observation::DesignedProcessAttribution,
        production_observation::ObservedSectorContext,
        production_observation::ProductionBusinessSubject, ArchiveEvidenceClass,
    };
    let mut snapshot = snapshot();
    let subject = ProductionBusinessSubject {
        scenario: "observed-fixture".into(),
        local_name: "business-26163-31-33".into(),
    };
    snapshot.observed_contexts.push(ObservedSectorContext {
        subject: subject.clone(),
        county_geoid: "26163".into(),
        sector_code: "31-33".into(),
        sector_title: "Manufacturing".into(),
        vintage: 2024,
        annual_avg_estabs_count: 11,
        annual_avg_emplvl: Some(1_234),
        total_annual_wages: Some(12_345_678),
        annual_avg_wkly_wage: Some(987),
        source_url: "https://www.bls.gov/cew/".into(),
        source_file: "county-source.csv".into(),
        source_sha256: "a".repeat(64),
        artifact_sha256: "b".repeat(64),
        evidence_class: ArchiveEvidenceClass::Observed,
    });
    for site in &snapshot.sites[..2] {
        snapshot
            .process_attributions
            .push(DesignedProcessAttribution {
                process_id: format!("process-{}", site.id),
                site_id: site.id.clone(),
                industry_code: site.industry_code.clone(),
                cohort_subject: subject.clone(),
                scenario_artifact_sha256: "c".repeat(64),
                industry_artifact_sha256: "d".repeat(64),
                evidence_class: ArchiveEvidenceClass::Designed,
            });
    }
    snapshot
}

#[test]
fn inspector_distinguishes_shared_sector_context_from_process_workers() {
    let snapshot = attributed_snapshot();
    let text = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Sources,
    );
    assert!(text.contains("SECTOR CONTEXT / OBSERVED 2024"));
    assert!(text.contains("Manufacturing | NAICS 31-33"));
    assert_eq!(text.matches("1,234 annual-average jobs").count(), 1);
    assert!(text.contains("USD 12,345,678 annual payroll"));
    assert!(text.contains("USD 987 mean weekly wage"));
    assert!(text.contains("Modeled processes sharing this context: Cohort a; Cohort b"));
    assert!(text.contains("This county-sector total does not assign workers to a process."));
    assert!(!text.contains("2,468"));
    assert!(!describe(
        &snapshot.sites[2],
        &snapshot,
        ProductionReadingSection::Sources
    )
    .contains("SECTOR CONTEXT"));
}

#[test]
fn inspector_keeps_undisclosed_sector_metrics_distinct_from_zero() {
    let mut snapshot = attributed_snapshot();
    let context = &mut snapshot.observed_contexts[0];
    context.annual_avg_emplvl = None;
    context.total_annual_wages = Some(0);
    context.annual_avg_wkly_wage = None;
    let text = describe(
        &snapshot.sites[0],
        &snapshot,
        ProductionReadingSection::Sources,
    );
    assert!(text.contains("Annual-average jobs: not disclosed"));
    assert!(text.contains("USD 0 annual payroll"));
    assert!(text.contains("Mean weekly wage: not disclosed"));
}

#[test]
fn dependency_buttons_deduplicate_real_relations_and_exclude_withheld_endpoints() {
    let snapshot = snapshot();
    let links = dependency_sites(&snapshot.sites[1], &snapshot);
    assert_eq!(
        links
            .into_iter()
            .map(|(direction, site)| (direction, site.id.as_str()))
            .collect::<Vec<_>>(),
        [
            (DependencyDirection::Upstream, "a"),
            (DependencyDirection::Downstream, "c")
        ],
    );
}

#[test]
fn flat_camera_projects_the_scene_and_plinths_inside_its_clip_volume() {
    use bevy::camera::CameraProjection;

    let mut app = App::new();
    app.insert_resource(PrimaryView::Production)
        .insert_resource(ProductionNavigation {
            flat: true,
            ..default()
        })
        .init_resource::<ObserverUiState>()
        .init_resource::<ProductionOrbit>()
        .init_resource::<ObserverViewport>()
        .init_resource::<UiScale>()
        .init_resource::<ObserverFrame>()
        .insert_resource(ObserverSession::new(CampaignId::from_uuid(
            uuid::Uuid::nil(),
        )))
        .add_systems(Update, paint_scene);
    let camera = app
        .world_mut()
        .spawn((
            Camera::default(),
            Transform::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            ProductionCamera,
        ))
        .id();
    app.update();
    let transform = *app.world().get::<Transform>(camera).unwrap();
    let Projection::Orthographic(mut projection) =
        app.world().get::<Projection>(camera).unwrap().clone()
    else {
        panic!("flat view must install an orthographic projection");
    };
    projection.update(934.0, 552.0);
    let clip_from_world = projection.get_clip_from_view() * transform.to_matrix().inverse();
    let mut production = snapshot();
    production.sites.extend([site("d", &[]), site("e", &["d"])]);
    let layout = ProductionLayout::focused(&production, Some("b"), 0);
    let mut points = vec![Vec3::ZERO];
    for position in layout.positions.values() {
        points.extend([*position, *position + Vec3::Y * 110.0]);
    }
    for (center, size) in &layout.platforms {
        for x in [-size.x * 0.5, size.x * 0.5] {
            for z in [-size.y * 0.5, size.y * 0.5] {
                points.push(*center + Vec3::new(x, -5.0, z));
            }
        }
    }
    for point in points {
        let ndc = clip_from_world.project_point3(point);
        assert!(
            ndc.is_finite()
                && ndc.x.abs() <= 1.0
                && ndc.y.abs() <= 1.0
                && ndc.z > 0.0
                && ndc.z < 1.0,
            "scene point {point:?} is clipped at {ndc:?}"
        );
    }
}

#[test]
fn labels_remain_inside_the_scene_when_history_reduces_a_small_window() {
    let scene = Rect::new(16.0, 96.0, 965.0, 378.0);
    let size = Vec2::new(180.0, 64.0);
    let position = place_label(Vec2::new(955.0, 375.0), scene, size, &[]).expect("visible anchor");
    assert!(scene.contains(position.min));
    assert!(scene.contains(position.max));
    assert!(place_label(Vec2::new(500.0, 420.0), scene, size, &[]).is_none());
    assert!(place_label(Vec2::new(20.0, 100.0), scene, Vec2::splat(1_000.0), &[]).is_none());
}

#[test]
fn open_drawers_block_scene_gestures_without_replaying_them_on_close() {
    use crate::observer_ui::ObserverDisclosure;
    use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

    let mut app = App::new();
    let mut window = Window::default();
    window.set_cursor_position(Some(Vec2::splat(50.0)));
    let window = app.world_mut().spawn((window, PrimaryWindow)).id();
    app.add_plugins(MinimalPlugins)
        .insert_resource(PrimaryView::Production)
        .insert_resource(ObserverUiState {
            menu_open: false,
            splash_visible: false,
            ..default()
        })
        .insert_resource(ObserverViewport(Some(Rect::new(0.0, 0.0, 200.0, 200.0))))
        .init_resource::<ProductionOrbit>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_message::<MouseMotion>()
        .add_message::<MouseWheel>()
        .add_systems(Update, orbit_input);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);
    for disclosure in [ObserverDisclosure::Time, ObserverDisclosure::Lens] {
        app.world_mut().resource_mut::<ObserverUiState>().disclosure = Some(disclosure);
        app.world_mut()
            .resource_mut::<Messages<MouseMotion>>()
            .write(MouseMotion {
                delta: Vec2::new(10.0, 5.0),
            });
        app.world_mut()
            .resource_mut::<Messages<MouseWheel>>()
            .write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: 1.0,
                window,
            });
        app.update();
        let orbit = app.world().resource::<ProductionOrbit>();
        assert_eq!(orbit.yaw.to_bits(), 0.0_f32.to_bits());
        assert_eq!(
            orbit.distance.to_bits(),
            ProductionOrbit::default().distance.to_bits()
        );
    }
    app.world_mut().resource_mut::<ObserverUiState>().disclosure = None;
    app.update();
    assert_eq!(
        app.world().resource::<ProductionOrbit>().distance.to_bits(),
        ProductionOrbit::default().distance.to_bits()
    );
    app.world_mut()
        .resource_mut::<Messages<MouseWheel>>()
        .write(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window,
        });
    app.update();
    assert_eq!(
        app.world().resource::<ProductionOrbit>().distance.to_bits(),
        (ProductionOrbit::default().distance - 65.0).to_bits()
    );
}

#[test]
fn inspector_scroll_resets_for_subject_or_capability_but_survives_tick_refresh() {
    let mut app = App::new();
    app.insert_resource(ObserverSession::new(CampaignId::from_uuid(
        uuid::Uuid::nil(),
    )))
    .insert_resource(ProductionNavigation {
        selected_site: Some("a".into()),
        ..default()
    })
    .add_systems(Update, reset_inspector_scroll);
    let panel = app
        .world_mut()
        .spawn((ProductionPanel, ScrollPosition::default()))
        .id();
    let other = app
        .world_mut()
        .spawn(ScrollPosition(Vec2::new(0.0, 77.0)))
        .id();
    app.update();
    app.world_mut()
        .entity_mut(panel)
        .get_mut::<ScrollPosition>()
        .unwrap()
        .y = 240.0;
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .ready(1, Some("a".repeat(64)));
    app.world_mut().resource_mut::<ProductionNavigation>().flat = true;
    app.update();
    assert_eq!(
        app.world()
            .get::<ScrollPosition>(panel)
            .unwrap()
            .y
            .to_bits(),
        240.0_f32.to_bits()
    );
    for change in 0..4 {
        app.world_mut()
            .entity_mut(panel)
            .get_mut::<ScrollPosition>()
            .unwrap()
            .y = 180.0;
        match change {
            0 => {
                app.world_mut()
                    .resource_mut::<ProductionNavigation>()
                    .selected_site = Some("b".into());
            }
            1 => {
                app.world_mut()
                    .resource_mut::<ObserverSession>()
                    .perspective = crate::observer::Perspective::PlayerKnowledge;
            }
            2 => {
                app.world_mut().resource_mut::<ObserverSession>().campaign =
                    CampaignId::from_uuid(uuid::Uuid::from_u128(1));
            }
            _ => {
                app.world_mut()
                    .resource_mut::<ProductionNavigation>()
                    .reading_section = ProductionReadingSection::Work;
            }
        }
        app.update();
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(panel)
                .unwrap()
                .y
                .to_bits(),
            0.0_f32.to_bits()
        );
        assert_eq!(
            app.world()
                .get::<ScrollPosition>(other)
                .unwrap()
                .y
                .to_bits(),
            77.0_f32.to_bits()
        );
    }
    app.world_mut()
        .entity_mut(panel)
        .get_mut::<ScrollPosition>()
        .unwrap()
        .y = 120.0;
    app.update();
    assert_eq!(
        app.world()
            .get::<ScrollPosition>(panel)
            .unwrap()
            .y
            .to_bits(),
        120.0_f32.to_bits()
    );
}

#[test]
fn map_and_flat_controls_preserve_the_county_selected_on_geography() {
    let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
    let mut state = ObserverSession::new(campaign);
    state.ready(1, Some("a".repeat(64)));
    let context = state.context();
    let atlas = CountyAtlas::parse(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    )))
    .expect("atlas");
    let index = |fips: &str| {
        (0..atlas.len())
            .find(|index| atlas.county(*index).is_some_and(|row| row.fips == fips))
            .expect("Michigan county")
    };
    let macomb = index("26099");
    let wayne = index("26163");
    let mut production = snapshot();
    production.sites[1].industry_code = "332".into();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(state)
        .insert_resource(ObserverFrame(Some(ObserverEconomySnapshot {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 1,
            foundation_digest: "f".repeat(64),
            tick_content_hash: Some("a".repeat(64)),
            nominal_world_hash: None,
            envelope_digest: None,
            visibility: ObserverVisibility::FullObserver,
            counties: Vec::new(),
            production: Some(production),
        })))
        .insert_resource(atlas)
        .insert_resource(PrimaryView::Production)
        .insert_resource(ProductionNavigation {
            selected_site: Some("a".into()),
            ..default()
        })
        .insert_resource(SelectedCounty(Some(macomb)))
        .init_resource::<ObserverFeedback>()
        .insert_resource(ObserverUiState {
            menu_open: false,
            splash_visible: false,
            ..default()
        })
        .add_message::<ProductionCommand>()
        .add_systems(Update, navigate);
    for command in [ProductionCommand::Map, ProductionCommand::Flat] {
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(command);
        app.update();
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(macomb));
        assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
    }
    assert!(app.world().resource::<ProductionNavigation>().flat);
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(ProductionCommand::Select {
            site_id: "b".into(),
            context,
        });
    app.update();
    assert_eq!(app.world().resource::<SelectedCounty>().0, Some(wayne));
    assert_eq!(
        *app.world().resource::<PrimaryView>(),
        PrimaryView::Production
    );
    for _ in 0..2 {
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(ProductionCommand::Open);
        app.update();
        assert_eq!(
            app.world()
                .resource::<ProductionNavigation>()
                .selected_site
                .as_deref(),
            Some("b"),
            "reopening preserves the chosen industry rather than the county's first site"
        );
        assert_eq!(app.world().resource::<SelectedCounty>().0, Some(wayne));
    }
}

#[test]
fn opening_focus_uses_current_capability_and_preserves_selection() {
    let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
    let mut state = ObserverSession::new(campaign);
    state.ready(1, Some("a".repeat(64)));
    let frame = ObserverFrame(Some(ObserverEconomySnapshot {
        campaign_id: campaign.as_uuid().to_string(),
        resolve_tick: 1,
        foundation_digest: "b".repeat(64),
        nominal_world_hash: None,
        tick_content_hash: Some("a".repeat(64)),
        envelope_digest: None,
        visibility: ObserverVisibility::FullObserver,
        counties: Vec::new(),
        production: Some(snapshot()),
    }));
    let mut app = App::new();
    app.insert_resource(state)
        .insert_resource(frame)
        .insert_resource(PrimaryView::Production)
        .insert_resource(
            CountyAtlas::parse(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/map/county_atlas.bin"
            )))
            .expect("atlas"),
        )
        .init_resource::<SelectedCounty>()
        .init_resource::<ProductionNavigation>()
        .add_systems(Update, (invalidate_navigation, focus_opening).chain());
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_some());
    app.world_mut()
        .resource_mut::<ProductionNavigation>()
        .selected_site = Some("c".into());
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("c")
    );
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    // Even an installed frame with the old observer payload cannot select
    // a site while the player capability is still loading.
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
}

#[test]
fn unchanged_scene_does_not_dirty_camera_or_label_components() {
    #[derive(Resource, Default)]
    struct ChangedCounts([usize; 4]);
    type ChangedSurfaceVisibility = (
        Or<(With<ProductionLabel>, With<ProductionPanel>)>,
        Changed<Visibility>,
    );
    fn count_changes(
        cameras: Query<Entity, (With<ProductionCamera>, Changed<Camera>)>,
        transforms: Query<Entity, (With<ProductionCamera>, Changed<Transform>)>,
        nodes: Query<Entity, (With<ProductionLabel>, Changed<Node>)>,
        visibility: Query<Entity, ChangedSurfaceVisibility>,
        mut counts: ResMut<ChangedCounts>,
    ) {
        counts.0 = [
            cameras.iter().count(),
            transforms.iter().count(),
            nodes.iter().count(),
            visibility.iter().count(),
        ];
    }
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ObserverSession::new(CampaignId::from_uuid(
            uuid::Uuid::nil(),
        )))
        .insert_resource(PrimaryView::Production)
        .init_resource::<ProductionNavigation>()
        .init_resource::<ObserverFrame>()
        .init_resource::<ObserverUiState>()
        .init_resource::<ProductionOrbit>()
        .init_resource::<ObserverViewport>()
        .init_resource::<UiScale>()
        .init_resource::<ChangedCounts>()
        .add_systems(Update, (paint_scene, paint_labels, count_changes).chain());
    app.world_mut().spawn((
        Camera::default(),
        Transform::default(),
        Projection::default(),
        ProductionCamera,
    ));
    app.world_mut()
        .spawn((ProductionPanel, Visibility::Visible, Node::default()));
    let leader = app
        .world_mut()
        .spawn((
            ProductionLeader,
            Node::default(),
            UiTransform::IDENTITY,
            Visibility::Hidden,
        ))
        .id();
    app.world_mut().spawn((
        ProductionLabel {
            anchor: Vec3::ZERO,
            site_id: "a".into(),
            selected: false,
            leader,
        },
        Node::default(),
        Visibility::Visible,
    ));
    app.update();
    app.update();
    assert_eq!(app.world().resource::<ChangedCounts>().0, [0; 4]);
}

fn press_site(app: &mut App, id: &str) {
    let world = app.world_mut();
    let entity = world
        .query::<(Entity, &ProductionButton)>()
        .iter(world)
        .find_map(|(entity, button)| match &button.0 {
            ProductionCommand::Select { site_id, .. } if site_id == id => Some(entity),
            _ => None,
        })
        .expect("a visible dependency button");
    world.entity_mut(entity).insert(Interaction::Pressed);
}

fn dependency_navigation_app() -> App {
    let mut app = unstarted_dependency_navigation_app();
    app.update();
    app
}

fn unstarted_dependency_navigation_app() -> App {
    let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
    let mut state = ObserverSession::new(campaign);
    state.ready(1, Some("a".repeat(64)));
    let frame = ObserverFrame(Some(ObserverEconomySnapshot {
        campaign_id: campaign.as_uuid().to_string(),
        resolve_tick: 1,
        foundation_digest: "f".repeat(64),
        tick_content_hash: Some("a".repeat(64)),
        nominal_world_hash: None,
        envelope_digest: None,
        visibility: ObserverVisibility::FullObserver,
        counties: Vec::new(),
        production: Some(snapshot()),
    }));
    let atlas = CountyAtlas::parse(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/map/county_atlas.bin"
    )))
    .expect("committed atlas");
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(state)
        .insert_resource(frame)
        .insert_resource(atlas)
        .init_resource::<PrimaryView>()
        .init_resource::<ProductionNavigation>()
        .init_resource::<SelectedCounty>()
        .init_resource::<ObserverUiState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ObserverFeedback>()
        .add_message::<ProductionCommand>()
        .add_systems(
            Update,
            (
                inputs,
                invalidate_navigation,
                navigate,
                sync_world_county,
                rebuild_dependencies,
                rebuild_county_cohorts,
            )
                .chain(),
        );
    app.world_mut().resource_mut::<ObserverUiState>().menu_open = false;
    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .splash_visible = false;
    app.world_mut()
        .spawn((Node::default(), ProductionDependencies));
    app
}

fn panel_text<T: Component>(app: &mut App) -> String {
    let world = app.world_mut();
    world
        .query_filtered::<&Text, With<T>>()
        .single(world)
        .unwrap()
        .0
        .clone()
}

fn production_panel_app() -> App {
    use bevy::ecs::system::RunSystemOnce;

    let mut app = dependency_navigation_app();
    app.insert_resource(PrimaryView::Production);
    app.world_mut()
        .run_system_once(setup)
        .expect("production panel");
    app.add_systems(Update, paint_disclosure.after(navigate));
    app.update();
    app
}

fn control_display(app: &mut App, command: &ProductionCommand) -> Display {
    let world = app.world_mut();
    world
        .query::<(&ProductionButton, &Node)>()
        .iter(world)
        .find_map(|(button, node)| {
            (std::mem::discriminant(&button.0) == std::mem::discriminant(command))
                .then_some(node.display)
        })
        .expect("production control")
}

fn send_command(app: &mut App, command: ProductionCommand) {
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(command);
    app.update();
}

#[test]
fn county_selection_keeps_readings_closed_until_a_cohort_is_chosen() {
    let mut app = production_panel_app();
    let context = app.world().resource::<ObserverSession>().context();
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "b".into(),
            context: context.clone(),
        },
    );
    send_command(&mut app, ProductionCommand::Details);
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    send_command(&mut app, ProductionCommand::Map);
    let macomb = {
        let atlas = app.world().resource::<CountyAtlas>();
        (0..atlas.len())
            .find(|index| atlas.county(*index).unwrap().fips == "26099")
            .unwrap()
    };
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(macomb);
    app.update();
    send_command(&mut app, ProductionCommand::Open);
    assert!(app.world().resource::<ProductionNavigation>().county_open);
    assert!(!app.world().resource::<ProductionNavigation>().details_open);
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Details),
        Display::None
    );
    for command in [
        ProductionCommand::Details,
        ProductionCommand::Reading(ProductionReadingSection::Flow),
    ] {
        send_command(&mut app, command);
        assert!(!app.world().resource::<ProductionNavigation>().details_open);
        assert!(app.world().resource::<ObserverFeedback>().message.is_some());
    }
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "a".into(),
            context,
        },
    );
    send_command(&mut app, ProductionCommand::Details);
    assert!(app.world().resource::<ProductionNavigation>().details_open);
}

#[test]
fn production_context_yields_its_rail_to_readings_and_returns_after_close() {
    let mut app = production_panel_app();
    app.init_resource::<ProductionOrbit>()
        .init_resource::<ObserverViewport>()
        .add_systems(Update, paint_scene.after(navigate));
    send_command(&mut app, ProductionCommand::Open);
    let world = app.world_mut();
    let subject = world
        .query_filtered::<Entity, With<ProductionPanel>>()
        .single(world)
        .unwrap();
    assert_eq!(
        world.get::<Node>(subject).unwrap().flex_direction,
        FlexDirection::Column
    );
    assert_eq!(
        *world.get::<Visibility>(subject).unwrap(),
        Visibility::Visible
    );
    send_command(&mut app, ProductionCommand::Details);
    assert_eq!(
        *app.world().get::<Visibility>(subject).unwrap(),
        Visibility::Hidden
    );
    send_command(&mut app, ProductionCommand::Details);
    assert_eq!(
        *app.world().get::<Visibility>(subject).unwrap(),
        Visibility::Visible
    );
}

#[test]
fn expanded_readings_use_the_side_panel_and_yield_to_modal_views() {
    use crate::observer_layout::{ObserverLayout, ObserverRegion};

    let mut app = production_panel_app();
    send_command(&mut app, ProductionCommand::Open);
    send_command(&mut app, ProductionCommand::Details);
    let world = app.world_mut();
    let detail = world
        .query_filtered::<Entity, With<ProductionDetailGroup>>()
        .single(world)
        .expect("one inspector");
    assert!(
        matches!(
            world.get::<ObserverRegion>(detail),
            Some(ObserverRegion::Log)
        ),
        "expanded readings must use the full-height side panel"
    );
    assert!(world.get::<ChildOf>(detail).is_none());
    for size in [Vec2::new(1366.0, 768.0), Vec2::new(1920.0, 1080.0)] {
        let layout = ObserverLayout::new(size, 1.0, false);
        let reading = layout.region(ObserverRegion::Log);
        assert!(reading.height() > 600.0);
        assert!(reading.min.x > layout.world.max.x);
    }
    assert_eq!(world.get::<Node>(detail).unwrap().display, Display::Flex);
    // History owns this same rail while open; closing it restores the
    // current subject's existing reading preference without recreating it.
    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .history_open = true;
    app.update();
    assert_eq!(
        app.world().get::<Node>(detail).unwrap().display,
        Display::None
    );
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .history_open = false;
    app.update();
    assert_eq!(
        app.world().get::<Node>(detail).unwrap().display,
        Display::Flex
    );
    for modal in 0..4 {
        {
            let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
            ui.menu_open = modal == 0;
            ui.archive_open = modal == 1;
            ui.comparison_open = modal == 2;
            ui.splash_visible = modal == 3;
        }
        app.update();
        assert_eq!(
            app.world().get::<Node>(detail).unwrap().display,
            Display::None
        );
    }
    *app.world_mut().resource_mut::<ObserverUiState>() = ObserverUiState {
        menu_open: false,
        splash_visible: false,
        ..default()
    };
    app.update();
    assert_eq!(
        app.world().get::<Node>(detail).unwrap().display,
        Display::Flex
    );
    send_command(&mut app, ProductionCommand::Map);
    assert_eq!(
        app.world().get::<Node>(detail).unwrap().display,
        Display::None
    );
}

#[test]
fn undisclosed_scene_hides_controls_and_explains_keyboard_refusals() {
    let mut app = production_panel_app();
    let full = app.world().resource::<ObserverFrame>().0.clone().unwrap();
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::None
    );
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Details),
        Display::Flex
    );
    for case in 0..4 {
        let mut frame = full.clone();
        let mut perspective = crate::observer::Perspective::FullObserver;
        match case {
            0 => {}
            1 => {
                perspective = crate::observer::Perspective::PlayerKnowledge;
                frame.visibility = ObserverVisibility::KnownPreview;
                frame.production = None;
            }
            2 => frame.production.as_mut().unwrap().sites.clear(),
            _ => frame.resolve_tick += 1,
        }
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(perspective);
        app.world_mut().resource_mut::<ObserverFrame>().0 = (case != 0).then_some(frame);
        app.update();
        for command in [
            ProductionCommand::Back,
            ProductionCommand::Details,
            ProductionCommand::Flat,
        ] {
            assert_eq!(control_display(&mut app, &command), Display::None);
        }
        for (key, reason) in [
            (
                KeyCode::Backspace,
                "There is no previous work view in this observation.",
            ),
            (
                KeyCode::KeyV,
                "Display controls need disclosed production relationships.",
            ),
        ] {
            *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(key);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            assert_eq!(
                app.world()
                    .resource::<crate::observer_ui::ObserverFeedback>()
                    .message,
                Some(reason)
            );
        }
        assert!(!app.world().resource::<ProductionNavigation>().flat);
        assert!(!app.world().resource::<ProductionNavigation>().details_open);
    }
}

#[test]
fn back_ignores_history_without_a_different_disclosed_destination() {
    let mut app = production_panel_app();
    {
        let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
        navigation.selected_site = Some("b".into());
        navigation.history = vec!["b".into(), "undisclosed-site".into()];
    }
    app.update();
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::None
    );
    send_command(&mut app, ProductionCommand::Back);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    assert_eq!(
        app.world().resource::<ObserverFeedback>().message,
        Some("There is no previous work view in this observation.")
    );
}

#[test]
fn scoped_back_and_display_preferences_survive_refresh_without_blocking_recovery() {
    let mut app = production_panel_app();
    let full = app.world().resource::<ObserverFrame>().0.clone();
    let context = app.world().resource::<ObserverSession>().context();
    {
        let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
        navigation.selected_site = Some("b".into());
        navigation.flat = true;
        navigation.details_open = true;
    }
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "a".into(),
            context,
        },
    );
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::Flex
    );
    app.world_mut().resource_mut::<ObserverFrame>().0 = None;
    app.update();
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Details),
        Display::None
    );
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    send_command(&mut app, ProductionCommand::Details);
    assert!(!app.world().resource::<ProductionNavigation>().details_open);
    send_command(&mut app, ProductionCommand::Map);
    assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
    app.world_mut().resource_mut::<ObserverFrame>().0 = full;
    app.update();
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::Flex
    );
    send_command(&mut app, ProductionCommand::Back);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    // Returning through World also supplies the county as a final Back destination.
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::Flex
    );
    send_command(&mut app, ProductionCommand::Back);
    assert!(app.world().resource::<ProductionNavigation>().county_open);
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::None
    );
    assert!(app.world().resource::<ProductionNavigation>().flat);
    send_command(&mut app, ProductionCommand::Details);
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.update();
    let navigation = app.world().resource::<ProductionNavigation>();
    assert!(navigation.flat);
    assert!(!navigation.details_open);
    assert!(navigation.history.is_empty());
    assert!(navigation.selected_site.is_none());
    assert_eq!(
        control_display(&mut app, &ProductionCommand::Back),
        Display::None
    );
}

#[test]
fn accepted_navigation_closes_drawers_but_display_toggles_keep_them() {
    use crate::observer_ui::ObserverDisclosure;

    let mut app = dependency_navigation_app();
    app.world_mut()
        .resource_mut::<ProductionNavigation>()
        .selected_site = Some("b".into());
    let context = app.world().resource::<ObserverSession>().context();
    for command in [
        ProductionCommand::Open,
        ProductionCommand::Map,
        ProductionCommand::Select {
            site_id: "a".into(),
            context,
        },
        ProductionCommand::Back,
    ] {
        app.world_mut().resource_mut::<ObserverUiState>().disclosure =
            Some(ObserverDisclosure::Lens);
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(command);
        app.update();
        assert!(app
            .world()
            .resource::<ObserverUiState>()
            .disclosure
            .is_none());
    }
    for command in [ProductionCommand::Flat, ProductionCommand::Details] {
        app.world_mut().resource_mut::<ObserverUiState>().disclosure =
            Some(ObserverDisclosure::Time);
        app.world_mut()
            .resource_mut::<Messages<ProductionCommand>>()
            .write(command);
        app.update();
        assert_eq!(
            app.world().resource::<ObserverUiState>().disclosure,
            Some(ObserverDisclosure::Time)
        );
    }
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
}

#[test]
fn exact_readings_are_collapsed_until_requested_and_clear_with_capability() {
    use bevy::ecs::system::RunSystemOnce;

    let mut app = dependency_navigation_app();
    *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
    app.world_mut()
        .run_system_once(setup)
        .expect("production panel");
    app.add_systems(
        Update,
        (paint_readings, paint_disclosure).chain().after(navigate),
    );
    app.world_mut()
        .resource_mut::<ProductionNavigation>()
        .selected_site = Some("b".into());
    app.update();
    let group = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<ProductionDetailGroup>>()
            .single(world)
            .unwrap()
    };
    assert_eq!(
        app.world().get::<Node>(group).unwrap().display,
        Display::None
    );
    assert!(panel_text::<ProductionBrief>(&mut app).contains("Committed plan partly completed"));
    assert!(panel_text::<ProductionDetails>(&mut app).is_empty());
    assert_eq!(
        panel_text::<ProductionDisclosureLabel>(&mut app),
        "READINGS +"
    );
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(ProductionCommand::Details);
    app.update();
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    assert_eq!(
        app.world().get::<Node>(group).unwrap().display,
        Display::Flex
    );
    assert!(panel_text::<ProductionDetails>(&mut app).contains("INPUTS / ON HAND"));
    press_site(&mut app, "a");
    app.update();
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    assert!(panel_text::<ProductionBrief>(&mut app).starts_with("Cohort a"));
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.update();
    assert!(!app.world().resource::<ProductionNavigation>().details_open);
    assert_eq!(
        app.world().get::<Node>(group).unwrap().display,
        Display::None
    );
    assert!(panel_text::<ProductionDetails>(&mut app).is_empty());
    assert!(!panel_text::<ProductionBrief>(&mut app).contains("Cohort"));
    assert!(panel_text::<ProductionReadingSubject>(&mut app).is_empty());
    assert!(panel_text::<ProductionReadingHeadline>(&mut app).is_empty());
}

#[test]
fn shared_freight_participant_buttons_navigate_and_expire_with_observation_scope() {
    use bevy::ecs::system::RunSystemOnce;
    let mut app = dependency_navigation_app();
    app.world_mut()
        .run_system_once(|mut commands: Commands| setup_readings_panel(&mut commands))
        .unwrap();
    app.add_systems(Update, paint_readings.after(navigate));
    app.world_mut()
        .resource_mut::<ObserverFrame>()
        .0
        .as_mut()
        .unwrap()
        .production = Some(crate::production_freight::tests::fixture());
    let context = app.world().resource::<ObserverSession>().context();
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "panels".into(),
            context: context.clone(),
        },
    );
    let world = app.world_mut();
    let text = world
        .query::<&Text>()
        .iter(world)
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(text.matches("Designed regional freight pool").count(), 1);
    assert!(text.contains("OTHER PARTICIPANTS / SHARED FREIGHT"));
    assert!(text.contains("160 kg opening · 160 kg reserved · 0 kg remaining"));
    send_command(
        &mut app,
        ProductionCommand::Reading(ProductionReadingSection::Freight),
    );
    assert!(
        panel_text::<ProductionDetails>(&mut app).contains("Requested 200 kg | dispatched 40 kg")
    );
    send_command(&mut app, ProductionCommand::Details);
    press_site(&mut app, "mill");
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("mill")
    );
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "panels".into(),
            context,
        },
    );
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    let world = app.world_mut();
    assert!(!world
        .query::<&Text>()
        .iter(world)
        .any(|text| text.0.contains("Designed regional freight pool")));
}

#[test]
fn keyboard_dependency_activation_uses_the_pointer_queue_and_rejects_changed_scope() {
    let mut app = dependency_navigation_app();
    app.add_observer(keyboard_activate);
    app.world_mut()
        .resource_mut::<ProductionNavigation>()
        .selected_site = Some("b".into());
    app.update();
    let context = app.world().resource::<ObserverSession>().context();
    let button = app
        .world_mut()
        .spawn(button_node(ProductionCommand::Select {
            site_id: "a".into(),
            context: context.clone(),
        }))
        .id();
    app.world_mut().trigger(ObserverKeyboardActivate {
        entity: button,
        context: Some(context.clone()),
    });
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b"),
        "keyboard activation queues the existing command; it does not mutate navigation in PreUpdate"
    );
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("a")
    );
    assert_eq!(
        app.world().resource::<ProductionNavigation>().history,
        ["b"]
    );
    app.world_mut().trigger(ObserverKeyboardActivate {
        entity: button,
        context: Some(context),
    });
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    assert!(app.world().resource::<ObserverFeedback>().message.is_some());
}

#[test]
fn world_focus_keeps_the_overview_and_reuses_authenticated_circuit_navigation() {
    let mut app = dependency_navigation_app();
    app.update();
    let context = app.world().resource::<ObserverSession>().context();
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(ProductionCommand::Focus {
            site_id: "b".into(),
            context: context.clone(),
        });
    app.update();
    assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(ProductionCommand::Open);
    app.update();
    assert_eq!(
        *app.world().resource::<PrimaryView>(),
        PrimaryView::Production
    );
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.world_mut()
        .resource_mut::<Messages<ProductionCommand>>()
        .write(ProductionCommand::Focus {
            site_id: "a".into(),
            context,
        });
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    assert!(app.world().resource::<ObserverFeedback>().message.is_some());
}

#[test]
fn world_county_list_enters_the_selected_circuit_and_rejects_old_observations() {
    let mut app = dependency_navigation_app();
    app.add_observer(keyboard_activate);
    let county_index = |app: &App, fips: &str| {
        let atlas = app.world().resource::<CountyAtlas>();
        (0..atlas.len())
            .find(|index| atlas.county(*index).unwrap().fips == fips)
            .unwrap()
    };
    let wayne = county_index(&app, "26163");
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(wayne);
    let root = app
        .world_mut()
        .spawn((Node::default(), ProductionCountyCohorts))
        .id();
    app.update();
    assert_eq!(*app.world().resource::<PrimaryView>(), PrimaryView::Map);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .county_geoid
            .as_deref(),
        Some("26163")
    );
    let button = {
        let world = app.world_mut();
        world.query::<(Entity, &ProductionButton, &ChildOf)>().iter(world)
            .find_map(|(entity, button, parent)| {
                (parent.parent() == root && matches!(&button.0, ProductionCommand::Select { site_id, .. } if site_id == "b"))
                    .then_some(entity)
            }).expect("World exposes the real cohort selection control")
    };
    let context = app.world().resource::<ObserverSession>().context();
    app.world_mut().trigger(ObserverKeyboardActivate {
        entity: button,
        context: Some(context.clone()),
    });
    app.update();
    assert_eq!(
        *app.world().resource::<PrimaryView>(),
        PrimaryView::Production
    );
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    send_command(&mut app, ProductionCommand::Map);
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    let macomb = county_index(&app, "26099");
    app.world_mut().resource_mut::<SelectedCounty>().0 = Some(macomb);
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .county_geoid
            .as_deref(),
        Some("26099")
    );
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    send_command(
        &mut app,
        ProductionCommand::Select {
            site_id: "b".into(),
            context,
        },
    );
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    assert!(app
        .world()
        .get::<Children>(root)
        .is_none_or(RelationshipTarget::is_empty));
}

#[test]
fn keyboard_pages_the_disclosed_county_then_enters_a_circuit_and_back() {
    let mut app = dependency_navigation_app();
    app.add_observer(keyboard_activate);
    {
        let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
        let snapshot = frame.0.as_mut().unwrap().production.as_mut().unwrap();
        snapshot.sites = (0..14)
            .map(|index| site(&format!("owner-{index:02}"), &[]))
            .collect();
        snapshot.routes.clear();
    }
    {
        let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
        navigation.county_geoid = Some("26163".into());
        navigation.county_open = true;
    }
    app.update();
    let context = app.world().resource::<ObserverSession>().context();
    let page_button = {
        let world = app.world_mut();
        world
            .query::<(Entity, &ProductionButton)>()
            .iter(world)
            .find_map(|(entity, button)| {
                matches!(
                    button.0,
                    ProductionCommand::Page {
                        kind: ProductionPage::Cohorts,
                        page: 1,
                        ..
                    }
                )
                .then_some(entity)
            })
            .expect("a real focusable next-page control")
    };
    app.world_mut().trigger(ObserverKeyboardActivate {
        entity: page_button,
        context: Some(context.clone()),
    });
    app.update();
    assert_eq!(
        app.world().resource::<ProductionNavigation>().cohort_page,
        1
    );
    press_site(&mut app, "owner-06");
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("owner-06")
    );
    assert!(!app.world().resource::<ProductionNavigation>().county_open);
    send_command(&mut app, ProductionCommand::Back);
    assert!(app.world().resource::<ProductionNavigation>().county_open);
    assert_eq!(
        app.world().resource::<ProductionNavigation>().cohort_page,
        1
    );
    send_command(
        &mut app,
        ProductionCommand::Page {
            kind: ProductionPage::Cohorts,
            page: 2,
            context,
        },
    );
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .set_perspective(crate::observer::Perspective::PlayerKnowledge);
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .county_geoid
        .is_none());
}

#[test]
fn accepted_world_buttons_release_focus_and_focused_keys_do_not_run_world_shortcuts() {
    use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusPolicy};
    use bevy::input::{
        keyboard::{Key, KeyboardInput, NativeKey},
        ButtonState, InputPlugin,
    };
    use bevy::input_focus::InputFocus;
    let mut app = dependency_navigation_app();
    app.add_plugins((InputPlugin, ObserverFocusPlugin))
        .add_observer(keyboard_activate)
        .add_systems(
            PreUpdate,
            focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
        );
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let context = app.world().resource::<ObserverSession>().context();
    app.world_mut()
        .resource_mut::<ObserverFocusPolicy>()
        .context = Some(context);
    let group = app
        .world_mut()
        .spawn((Node::default(), TabGroup::new(10)))
        .id();
    let button = app
        .world_mut()
        .spawn((button_node(ProductionCommand::Open), ChildOf(group)))
        .id();
    app.update();
    app.world_mut().resource_mut::<InputFocus>().set(button);
    let key = |app: &mut App, key_code, state| {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    };
    key(&mut app, KeyCode::KeyP, ButtonState::Pressed);
    assert_eq!(
        *app.world().resource::<PrimaryView>(),
        PrimaryView::Map,
        "focused controls own raw P, so it cannot also open the world"
    );
    key(&mut app, KeyCode::KeyP, ButtonState::Released);
    key(&mut app, KeyCode::Enter, ButtonState::Pressed);
    assert_eq!(
        *app.world().resource::<PrimaryView>(),
        PrimaryView::Production
    );
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(window));
    assert!(app
        .world()
        .resource::<ObserverKeyboardClaim>()
        .claimed(KeyCode::Enter));
    key(&mut app, KeyCode::Enter, ButtonState::Released);
    app.world_mut().resource_mut::<InputFocus>().set(button);
    key(&mut app, KeyCode::Enter, ButtonState::Pressed);
    assert_eq!(
        app.world().resource::<InputFocus>().get(),
        Some(window),
        "WORK releases focus even when work is already the active view"
    );
}

struct ReadingsFocusFixture {
    app: App,
    window: Entity,
    body: Entity,
    reading: Entity,
    close: Entity,
    flat: Entity,
    footer: Entity,
}

fn readings_focus_app() -> ReadingsFocusFixture {
    use crate::observer_focus::{ObserverFocusPlugin, ObserverFocusPolicy};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::InputPlugin;
    let mut app = unstarted_dependency_navigation_app();
    app.add_plugins((InputPlugin, ObserverFocusPlugin))
        .add_observer(keyboard_activate)
        .add_systems(
            PreUpdate,
            focus_eligibility.in_set(ObserverFocusSystems::Eligibility),
        )
        .add_systems(
            Update,
            (paint_readings, paint_disclosure).chain().after(navigate),
        );
    let window = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    let context = app.world().resource::<ObserverSession>().context();
    app.world_mut()
        .resource_mut::<ObserverFocusPolicy>()
        .context = Some(context);
    // The real TabNavigationPlugin attaches its window observer in Startup.
    // Install it before the fixture's first update, as the windowed app does.
    app.update();
    *app.world_mut().resource_mut::<PrimaryView>() = PrimaryView::Production;
    {
        let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
        navigation.selected_site = Some("b".into());
        navigation.details_open = true;
    }
    app.world_mut()
        .run_system_once(|mut commands: Commands| setup_readings_panel(&mut commands))
        .unwrap();
    let body = app
        .world_mut()
        .query_filtered::<Entity, With<ProductionReadingBody>>()
        .single(app.world())
        .unwrap();
    let reading = app
        .world_mut()
        .query_filtered::<Entity, With<ProductionDetails>>()
        .single(app.world())
        .unwrap();
    let find_button = |world: &mut World, command: fn(&ProductionCommand) -> bool| {
        world
            .query::<(Entity, &ProductionButton)>()
            .iter(world)
            .find_map(|(entity, button)| command(&button.0).then_some(entity))
            .unwrap()
    };
    let close = find_button(app.world_mut(), |command| {
        matches!(command, ProductionCommand::Details)
    });
    let flat = find_button(app.world_mut(), |command| {
        matches!(command, ProductionCommand::Flat)
    });
    let group = app
        .world_mut()
        .spawn((Node::default(), TabGroup::new(40)))
        .id();
    let mut target = ObserverFocusTarget::action(None);
    target.available = true;
    let footer = app
        .world_mut()
        .spawn((Node::default(), target, ChildOf(group)))
        .id();
    // Exercise the actual paging system with explicit overflow geometry;
    // rendering/layout itself belongs to the native-window acceptance check.
    app.world_mut().entity_mut(body).insert((
        ComputedNode {
            size: Vec2::new(300.0, 160.0),
            content_size: Vec2::new(300.0, 1200.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        ScrollPosition::default(),
    ));
    app.world_mut().entity_mut(reading).insert((
        ComputedNode {
            size: Vec2::new(300.0, 1100.0),
            content_size: Vec2::new(300.0, 1100.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        UiGlobalTransform::from_translation(Vec2::new(0.0, 470.0)),
    ));
    app.update();
    app.update();
    ReadingsFocusFixture {
        app,
        window,
        body,
        reading,
        close,
        flat,
        footer,
    }
}

fn readings_key(app: &mut App, window: Entity, key_code: KeyCode) {
    use bevy::input::{
        keyboard::{Key, KeyboardInput, NativeKey},
        ButtonState,
    };
    for state in [ButtonState::Pressed, ButtonState::Released] {
        app.world_mut().write_message(KeyboardInput {
            key_code,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state,
            text: None,
            repeat: false,
            window,
        });
        app.update();
    }
}

#[test]
fn reading_sections_switch_by_keyboard_without_changing_the_observed_period() {
    use bevy::input_focus::InputFocus;
    let ReadingsFocusFixture {
        mut app, window, ..
    } = readings_focus_app();
    let period = app.world().resource::<ObserverSession>().viewed_tick;
    let work = {
        let world = app.world_mut();
        world
            .query::<(&Text, &ChildOf)>()
            .iter(world)
            .find_map(|(text, parent)| (text.0 == "Work").then_some(parent.parent()))
            .expect("a visible Work section control")
    };
    assert!(panel_text::<ProductionDetails>(&mut app).contains("INPUTS / ON HAND"));
    assert!(!panel_text::<ProductionDetails>(&mut app).contains("LABOR BUDGET"));
    app.world_mut().resource_mut::<InputFocus>().set(work);
    readings_key(&mut app, window, KeyCode::Enter);
    let reading = panel_text::<ProductionDetails>(&mut app);
    assert!(reading.contains("LABOR BUDGET / DERIVED"));
    assert!(!reading.contains("INPUTS / ON HAND"));
    assert!(!reading.contains("SECTOR CONTEXT"));
    assert_eq!(
        app.world().resource::<ObserverSession>().viewed_tick,
        period
    );
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
}

#[test]
fn readings_tab_order_reaches_the_text_after_controls_and_pages_its_scroll_ancestor() {
    use bevy::input_focus::InputFocus;
    let ReadingsFocusFixture {
        mut app,
        window,
        body,
        reading,
        close,
        flat,
        footer,
    } = readings_focus_app();
    app.world_mut().resource_mut::<InputFocus>().set(close);
    for section in [
        ProductionReadingSection::Flow,
        ProductionReadingSection::Freight,
        ProductionReadingSection::Work,
        ProductionReadingSection::Sources,
    ] {
        readings_key(&mut app, window, KeyCode::Tab);
        let focused = app.world().resource::<InputFocus>().get().unwrap();
        assert!(
            matches!(&app.world().get::<ProductionButton>(focused).unwrap().0,
            ProductionCommand::Reading(current) if *current == section)
        );
    }
    readings_key(&mut app, window, KeyCode::Tab);
    assert_eq!(
        app.world().resource::<InputFocus>().get(),
        Some(reading),
        "the reading follows its section controls"
    );
    let period = app.world().resource::<ObserverSession>().viewed_tick;
    readings_key(&mut app, window, KeyCode::PageDown);
    assert!(app.world().get::<ScrollPosition>(body).unwrap().0.y > 0.0);
    readings_key(&mut app, window, KeyCode::End);
    assert_eq!(
        app.world()
            .get::<ScrollPosition>(body)
            .unwrap()
            .0
            .y
            .to_bits(),
        1040.0_f32.to_bits()
    );
    readings_key(&mut app, window, KeyCode::Home);
    assert_eq!(
        app.world().get::<ScrollPosition>(body).unwrap().0,
        Vec2::ZERO
    );
    readings_key(&mut app, window, KeyCode::Enter);
    assert_eq!(
        app.world().resource::<ObserverSession>().viewed_tick,
        period
    );
    assert!(app.world().resource::<ProductionNavigation>().details_open);
    readings_key(&mut app, window, KeyCode::Tab);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(flat));
    readings_key(&mut app, window, KeyCode::Tab);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(footer));
}

#[test]
fn shared_freight_reading_enters_tab_order_and_pages_its_scroll_ancestor() {
    use bevy::input_focus::{tab_navigation::TabIndex, InputFocus};
    let ReadingsFocusFixture {
        mut app,
        window,
        footer,
        ..
    } = readings_focus_app();
    app.world_mut()
        .resource_mut::<ObserverFrame>()
        .0
        .as_mut()
        .unwrap()
        .production = Some(crate::production_freight::tests::fixture());
    {
        let mut navigation = app.world_mut().resource_mut::<ProductionNavigation>();
        navigation.selected_site = Some("panels".into());
        navigation.details_open = false;
    }
    let body = app
        .world_mut()
        .query_filtered::<Entity, With<ProductionDependencies>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(body).insert((
        Node {
            overflow: Overflow::scroll_y(),
            ..default()
        },
        TabGroup::new(10),
        ComputedNode {
            size: Vec2::new(300.0, 160.0),
            content_size: Vec2::new(300.0, 1200.0),
            inverse_scale_factor: 1.0,
            ..default()
        },
        ScrollPosition::default(),
    ));
    app.update(); // Rebuild the actual shared-pool reading and competitor buttons.
    app.update(); // Ownership must admit the initially unavailable reading.
    let reading = app
        .world_mut()
        .query_filtered::<Entity, With<ProductionFreightReading>>()
        .single(app.world())
        .unwrap();
    let target = app.world().get::<ObserverFocusTarget>(reading).unwrap();
    assert!(target.available);
    assert!(app.world().get::<TabIndex>(reading).is_some());
    app.world_mut().resource_mut::<InputFocus>().set(footer);
    readings_key(&mut app, window, KeyCode::Tab);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(reading));
    readings_key(&mut app, window, KeyCode::PageDown);
    assert!(app.world().get::<ScrollPosition>(body).unwrap().0.y > 0.0);
    let period = app.world().resource::<ObserverSession>().viewed_tick;
    readings_key(&mut app, window, KeyCode::Enter);
    assert_eq!(
        app.world().resource::<ObserverSession>().viewed_tick,
        period
    );
    app.world_mut().resource_mut::<ObserverUiState>().menu_open = true;
    app.update();
    assert!(
        !app.world()
            .get::<ObserverFocusTarget>(reading)
            .unwrap()
            .available
    );
    assert!(app.world().get::<TabIndex>(reading).is_none());
}

#[test]
fn repainted_reading_scope_reenters_tab_order_after_a_committed_period_refresh() {
    use crate::observer_focus::ObserverFocusPolicy;
    use bevy::input_focus::tab_navigation::TabIndex;
    let ReadingsFocusFixture {
        mut app, reading, ..
    } = readings_focus_app();
    let hash = "b".repeat(64);
    {
        let mut session = app.world_mut().resource_mut::<ObserverSession>();
        session.ready(2, Some(hash.clone()));
        let context = session.context();
        assert!(session.installed(&context));
    }
    {
        let mut frame = app.world_mut().resource_mut::<ObserverFrame>();
        let snapshot = frame.0.as_mut().unwrap();
        snapshot.resolve_tick = 2;
        snapshot.tick_content_hash = Some(hash);
    }
    let context = app.world().resource::<ObserverSession>().context();
    app.world_mut()
        .resource_mut::<ObserverFocusPolicy>()
        .context = Some(context.clone());
    app.update(); // old scope is removed, then the reading is repainted
    app.update(); // the repainted target must be admitted without another UI action
    let target = app.world().get::<ObserverFocusTarget>(reading).unwrap();
    assert_eq!(target.context.as_ref(), Some(&context));
    assert!(target.available);
    assert!(app.world().get::<TabIndex>(reading).is_some());
}

#[test]
fn native_dependency_navigation_preserves_back_and_rejects_late_contexts() {
    let mut app = dependency_navigation_app();
    app.world_mut()
        .resource_mut::<ProductionNavigation>()
        .selected_site = Some("b".into());
    app.update();

    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .comparison_open = true;
    press_site(&mut app, "a");
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .comparison_open = false;
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b"),
        "closing the modal does not replay blocked input"
    );
    press_site(&mut app, "a");
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("a")
    );
    assert_eq!(
        app.world().resource::<ProductionNavigation>().history,
        ["b"]
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Backspace);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .history
        .is_empty());

    press_site(&mut app, "a");
    app.world_mut().resource_mut::<ObserverSession>().generation += 1;
    app.update();
    assert_eq!(
        app.world()
            .resource::<ProductionNavigation>()
            .selected_site
            .as_deref(),
        Some("b")
    );
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .perspective = crate::observer::Perspective::PlayerKnowledge;
    app.update();
    assert!(app
        .world()
        .resource::<ProductionNavigation>()
        .selected_site
        .is_none());
    let world = app.world_mut();
    assert_eq!(world.query::<&ProductionButton>().iter(world).count(), 0);
}
#[test]
fn topology_uses_disclosed_endpoints_and_survives_input_reordering() {
    let mut snapshot = snapshot();
    let mut hidden = snapshot.routes[0].clone();
    hidden.id = "hidden-route".into();
    hidden.buyer_site_id = "withheld".into();
    snapshot.routes.push(hidden);
    let original = ProductionLayout::focused(&snapshot, Some("b"), 0);
    assert_eq!(original.positions.len(), 3);
    assert_eq!(
        original.links,
        [("a".into(), "b".into()), ("b".into(), "c".into())]
    );
    assert!(original.positions["a"].x < original.positions["b"].x);
    assert!(original.positions["b"].x < original.positions["c"].x);
    snapshot.sites.reverse();
    snapshot.routes.reverse();
    for site in &mut snapshot.sites {
        for input in &mut site.processes[0].inputs {
            input.supplier_site_ids.reverse();
        }
    }
    let reordered = ProductionLayout::focused(&snapshot, Some("b"), 0);
    assert_eq!(original.positions, reordered.positions);
    assert_eq!(original.links, reordered.links);
    snapshot.sites.retain(|site| site.id != "a");
    let scoped = ProductionLayout::focused(&snapshot, Some("b"), 0);
    assert_eq!(scoped.links, [("b".into(), "c".into())]);
    assert!(!scoped.positions.contains_key("a"));
    assert!(!scoped.positions.contains_key("withheld"));
}

#[test]
fn only_actual_visible_in_transit_lots_get_static_markers() {
    use babylon_persistence::production_observation::ProductionFreight;

    let mut snapshot = snapshot();
    let layout = ProductionLayout::focused(&snapshot, Some("b"), 0);
    assert!(
        freight_markers(&snapshot, &layout, 1).is_empty(),
        "orders and deliveries alone must not generate freight"
    );
    let lot = ProductionFreight {
        current_stage_index: 0,
        grams_per_unit: 1000,
        mass_grams: 1000,
        id: "actual-lot".into(),
        route_id: "a-b".into(),
        source_site_id: "a".into(),
        destination_site_id: "b".into(),
        good_id: "a".repeat(64),
        unit_id: "b".repeat(64),
        good: "steel".into(),
        unit: "kg".into(),
        quantity: 10,
        dispatch_period: 1,
        arrival_period: 4,
    };
    snapshot.freight.push(lot.clone());
    let first = freight_markers(&snapshot, &layout, 1);
    assert_eq!(first.len(), 1);
    assert_eq!(
        first,
        freight_markers(&snapshot, &layout, 2),
        "schematic markers do not invent continuous travel motion"
    );
    assert!(freight_markers(&snapshot, &layout, 0).is_empty());
    assert!(freight_markers(&snapshot, &layout, 4).is_empty());
    for case in 0..4 {
        let mut withheld = lot.clone();
        match case {
            0 => withheld.quantity = 0,
            1 => withheld.route_id = "unavailable-route".into(),
            2 => withheld.destination_site_id = "withheld".into(),
            _ => withheld.good_id = "another-good".into(),
        }
        snapshot.freight = vec![withheld];
        assert!(freight_markers(&snapshot, &layout, 1).is_empty());
    }
}
