//! PER-24: the shipped Bevy viewer must enumerate every visible surface and
//! keep administrative exemptions outside every gameplay gate.

use babylon_client::decision_surface::{
    contract_for, DecisionSurfaceContract, DecisionSurfaceRole, DeclaredSurface, SurfaceActionV1,
    SurfaceId, SHIPPED_SURFACE_MANIFEST,
};
use bevy::asset::AssetPlugin;
use bevy::image::ImagePlugin;
use bevy::prelude::*;
use bevy::render::texture::TexturePlugin;
use bevy::time::TimeUpdateStrategy;
use std::collections::HashSet;
use std::time::Duration;

fn conformance_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::text::TextPlugin,
        TexturePlugin,
    ));
    app.add_plugins((
        babylon_client::visual_assets::VisualAssetsPlugin,
        babylon_client::visual_assets::VisualPresentationPlugin,
    ));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    // Retained graph-viewer conformance composition, including its dossier.
    app.add_plugins(babylon_client::ui::dossier_card::DossierCardPlugin);
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.finish();
    app.update(); // Startup.
    app.world_mut()
        .resource_mut::<babylon_client::map::SelectedCounty>()
        .0 = Some(0);
    app.update(); // Exercise the selection-outline spawn path too.
    app
}

fn observer_app() -> App {
    use babylon_client::observer::{ObserverSession, SessionPhase};
    use babylon_client::observer_io::ObserverSet;
    use babylon_client::observer_ui::{ObserverFrame, ObserverUiState};
    use babylon_persistence::{CampaignId, ObserverEconomySnapshotV1, ObserverVisibilityV1};

    let campaign = CampaignId::from_uuid(uuid::Uuid::nil());
    let mut session = ObserverSession::new(campaign);
    session.foundation_digest = Some("f".repeat(64));
    session.ready(2, Some("b".repeat(64)));
    session.inspect_tick(1);
    assert!(session.installed(&session.context()));
    assert_eq!(session.phase, SessionPhase::Ready);
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default(),
        bevy::text::TextPlugin,
        TexturePlugin,
        WindowPlugin {
            primary_window: None,
            exit_condition: bevy::window::ExitCondition::DontExit,
            ..default()
        },
        bevy::picking::DefaultPickingPlugins,
    ));
    // Mirror app.rs's visual composition. IO is supplied by an immutable,
    // historical read fixture; audio settings need no device. There is no
    // runtime pipe, credential lookup, gameplay session, or database writer.
    app.add_plugins(babylon_client::visual_assets::VisualAssetsPlugin)
        .add_plugins(babylon_client::map::MapPlugin)
        .add_plugins(babylon_client::observer_ui::ObserverShellPlugin)
        .add_plugins(babylon_client::observer_map3d::ObserverMap3dPlugin)
        .add_plugins(babylon_client::production::ProductionPlugin)
        .add_plugins(babylon_client::campaign_browser::CampaignBrowserPlugin)
        .add_plugins(babylon_client::observer_history::ObserverHistoryPlugin)
        .add_plugins(babylon_client::ui::dossier_card::DossierCardPlugin)
        .add_plugins(babylon_client::session_log::SessionLogPlugin)
        .init_resource::<babylon_client::observer_audio::ObserverAudioSettings>()
        .init_resource::<UiScale>()
        .insert_resource(session)
        .insert_resource(babylon_client::ui::dossier_card::DossierCampaignId(
            campaign,
        ))
        .insert_resource(ObserverFrame(Some(ObserverEconomySnapshotV1 {
            campaign_id: campaign.as_uuid().to_string(),
            resolve_tick: 1,
            foundation_digest: "f".repeat(64),
            nominal_world_hash: Some("c".repeat(64)),
            tick_content_hash: Some("a".repeat(64)),
            envelope_digest: Some("d".repeat(64)),
            visibility: ObserverVisibilityV1::FullObserver,
            counties: Vec::new(),
            production: Some(production_observation()),
        })))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .configure_sets(
            Update,
            (
                ObserverSet::Input,
                ObserverSet::Receive,
                ObserverSet::Install,
                ObserverSet::Paint,
            )
                .chain(),
        );
    app.finish();
    app.update();
    {
        let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
        ui.splash_visible = false;
        ui.menu_open = false;
        ui.history_open = true;
    }
    app.update(); // Exercise committed event buttons without requesting history.
    app
}

fn production_observation() -> babylon_persistence::ProductionSnapshotV1 {
    use babylon_persistence::{
        ProductionEventV1, ProductionFreightV1, ProductionRouteV1, ProductionSiteV1,
        ProductionSnapshotV1,
    };
    let site = |id: &str| ProductionSiteV1 {
        id: id.into(),
        county_geoid: "26163".into(),
        name: format!("Surface fixture {id}"),
        industry_code: "331".into(),
        observed_employment: None,
        output_good_id: "a".repeat(64),
        output_unit_id: "b".repeat(64),
        output_good: "sheet".into(),
        output_unit: "kg".into(),
        output_per_batch: 1,
        available_batches: 10,
        planned_batches: Some(10),
        produced_batches: Some(10),
        inventory: Vec::new(),
        inputs: Vec::new(),
        labor: Vec::new(),
    };
    ProductionSnapshotV1 {
        material_balance: None,
        labor_accounts: Vec::new(),
        scenario_label: "Read-only surface fixture".into(),
        horizon_week: 16,
        sites: vec![site("source"), site("destination")],
        routes: vec![ProductionRouteV1 {
            id: "route".into(),
            supplier_site_id: "source".into(),
            buyer_site_id: "destination".into(),
            good_id: "a".repeat(64),
            unit_id: "b".repeat(64),
            good: "sheet".into(),
            unit: "kg".into(),
            travel_weeks: 1,
            ordered: 10,
            shipped: 10,
            delivered: 0,
            lost: 0,
            realized: 0,
            backlog: 0,
        }],
        freight: vec![ProductionFreightV1 {
            id: "lot".into(),
            route_id: "route".into(),
            source_site_id: "source".into(),
            destination_site_id: "destination".into(),
            good_id: "a".repeat(64),
            unit_id: "b".repeat(64),
            good: "sheet".into(),
            unit: "kg".into(),
            quantity: 10,
            dispatch_week: 1,
            arrival_week: 2,
        }],
        events: vec![ProductionEventV1 {
            id: "dispatch".into(),
            week: 1,
            subject_site_ids: vec!["source".into(), "destination".into()],
            kind: "dispatch".into(),
            description: "Surface fixture dispatch".into(),
            receipt_digest: "e".repeat(64),
            delivery_evidence: None,
        }],
        observed_contexts: Vec::new(),
        process_attributions: Vec::new(),
        provenance: vec!["Designed read-only test fixture".into()],
    }
}

#[test]
fn manifest_is_unique_exhaustive_and_valid() {
    let ids: HashSet<_> = SHIPPED_SURFACE_MANIFEST
        .iter()
        .map(|contract| contract.id)
        .collect();

    assert_eq!(
        ids.len(),
        SHIPPED_SURFACE_MANIFEST.len(),
        "a shipped surface may have exactly one authoritative contract"
    );
    assert_eq!(
        ids,
        SurfaceId::ALL.into_iter().collect(),
        "every shipped surface id must have one manifest row"
    );

    for contract in SHIPPED_SURFACE_MANIFEST {
        contract
            .validate()
            .unwrap_or_else(|error| panic!("{} has an invalid contract: {error}", contract.id));
        assert_eq!(
            contract_for(contract.id),
            contract,
            "manifest lookup must resolve the authoritative row"
        );
    }
}

#[test]
fn gameplay_contract_requires_every_decision_field() {
    const PRESENT: &[&str] = &["declared"];
    const ACTIONS: &[SurfaceActionV1] = &[SurfaceActionV1::available("declared")];
    let complete = DecisionSurfaceContract {
        id: SurfaceId::CountyMap,
        role: DecisionSurfaceRole::Gameplay,
        decision_question: Some("What should I do here next week?"),
        visible_signals: PRESENT,
        visible_uncertainty: PRESENT,
        fog_requirements: PRESENT,
        actions: ACTIONS,
        expected_receipts: PRESENT,
        archive_subjects: PRESENT,
        admin_debug_exempt: false,
    };
    assert!(complete.validate().is_ok());
    assert!(complete.satisfies_gameplay_gate());

    let omissions = [
        DecisionSurfaceContract {
            decision_question: None,
            ..complete
        },
        DecisionSurfaceContract {
            visible_signals: &[],
            ..complete
        },
        DecisionSurfaceContract {
            visible_uncertainty: &[],
            ..complete
        },
        DecisionSurfaceContract {
            fog_requirements: &[],
            ..complete
        },
        DecisionSurfaceContract {
            actions: &[],
            ..complete
        },
        DecisionSurfaceContract {
            expected_receipts: &[],
            ..complete
        },
        DecisionSurfaceContract {
            archive_subjects: &[],
            ..complete
        },
    ];

    for omitted in omissions {
        assert!(omitted.validate().is_err());
        assert!(!omitted.satisfies_gameplay_gate());
    }
}

#[test]
fn gameplay_contract_rejects_blank_entries_in_every_required_list() {
    const PRESENT: &[&str] = &["declared"];
    const ACTIONS: &[SurfaceActionV1] = &[SurfaceActionV1::available("declared")];
    const CONTAINS_BLANK: &[&str] = &["declared", " \t\n"];
    const CONTAINS_BLANK_ACTION: &[SurfaceActionV1] = &[
        SurfaceActionV1::available("declared"),
        SurfaceActionV1::available(" \t\n"),
    ];
    let complete = DecisionSurfaceContract {
        id: SurfaceId::CountyMap,
        role: DecisionSurfaceRole::Gameplay,
        decision_question: Some("What should I do here next week?"),
        visible_signals: PRESENT,
        visible_uncertainty: PRESENT,
        fog_requirements: PRESENT,
        actions: ACTIONS,
        expected_receipts: PRESENT,
        archive_subjects: PRESENT,
        admin_debug_exempt: false,
    };

    let blank_entries = [
        DecisionSurfaceContract {
            visible_signals: CONTAINS_BLANK,
            ..complete
        },
        DecisionSurfaceContract {
            visible_uncertainty: CONTAINS_BLANK,
            ..complete
        },
        DecisionSurfaceContract {
            fog_requirements: CONTAINS_BLANK,
            ..complete
        },
        DecisionSurfaceContract {
            actions: CONTAINS_BLANK_ACTION,
            ..complete
        },
        DecisionSurfaceContract {
            expected_receipts: CONTAINS_BLANK,
            ..complete
        },
        DecisionSurfaceContract {
            archive_subjects: CONTAINS_BLANK,
            ..complete
        },
    ];

    for contract in blank_entries {
        assert!(contract.validate().is_err());
        assert!(!contract.satisfies_gameplay_gate());
    }
}

#[test]
fn admin_inspector_manifest_matches_its_pre_tick_rendering() {
    const PRE_TICK_REPORT: &str = "tick report \u{2014} not yet run";
    const ROSTER_STATUS: &str = "roster \u{2014} no county selected";

    let contract = contract_for(SurfaceId::AdminInspector);
    assert!(contract.visible_signals.contains(&PRE_TICK_REPORT));
    assert!(contract.visible_signals.contains(&ROSTER_STATUS));

    let mut app = conformance_app();
    app.world_mut()
        .resource_mut::<babylon_client::map::SelectedCounty>()
        .0 = None;
    app.world_mut()
        .resource_mut::<babylon_client::ui::admin::AdminPanelVisible>()
        .0 = true;
    app.update();

    let world = app.world_mut();
    let mut query =
        world.query_filtered::<&Text, With<babylon_client::ui::admin::AdminPanelText>>();
    let rendered = &query
        .single(world)
        .expect("exactly one admin inspector text entity")
        .0;
    assert!(rendered.contains(PRE_TICK_REPORT), "got {rendered:?}");
    assert!(rendered.contains(ROSTER_STATUS), "got {rendered:?}");
}

#[test]
fn admin_exemptions_never_satisfy_gameplay_gates() {
    const PRESENT: &[&str] = &["declared"];
    const ACTIONS: &[SurfaceActionV1] = &[SurfaceActionV1::available("declared")];
    let exempt_gameplay = DecisionSurfaceContract {
        id: SurfaceId::CountyMap,
        role: DecisionSurfaceRole::Gameplay,
        decision_question: Some("Looks complete, but is exempt"),
        visible_signals: PRESENT,
        visible_uncertainty: PRESENT,
        fog_requirements: PRESENT,
        actions: ACTIONS,
        expected_receipts: PRESENT,
        archive_subjects: PRESENT,
        admin_debug_exempt: true,
    };
    assert!(exempt_gameplay.validate().is_err());
    assert!(!exempt_gameplay.satisfies_gameplay_gate());

    let disguised_admin = DecisionSurfaceContract {
        id: SurfaceId::CountyMap,
        role: DecisionSurfaceRole::AdminDebug,
        decision_question: Some("Looks complete, but is administrative"),
        visible_signals: PRESENT,
        visible_uncertainty: PRESENT,
        fog_requirements: PRESENT,
        actions: ACTIONS,
        expected_receipts: PRESENT,
        archive_subjects: PRESENT,
        admin_debug_exempt: true,
    };

    assert!(disguised_admin.validate().is_ok());
    assert!(!disguised_admin.satisfies_gameplay_gate());
    for contract in SHIPPED_SURFACE_MANIFEST {
        if contract.admin_debug_exempt {
            assert!(!contract.satisfies_gameplay_gate(), "{}", contract.id);
        }
    }
}

#[test]
fn every_shipped_visual_entity_declares_a_manifest_surface() {
    let mut conformance = conformance_app();
    let mut observer = observer_app();
    let conformance_ids = visual_surface_ids(conformance.world_mut());
    let observer_ids = visual_surface_ids(observer.world_mut());
    let observer_surfaces = HashSet::from([
        SurfaceId::TitleLockup,
        SurfaceId::CountyDossier,
        SurfaceId::ObserverShell,
        SurfaceId::ObserverProduction,
    ]);
    assert_eq!(observer_ids, observer_surfaces);
    assert_eq!(
        conformance_ids,
        SurfaceId::ALL
            .into_iter()
            .filter(|id| !matches!(id, SurfaceId::ObserverShell | SurfaceId::ObserverProduction))
            .collect(),
        "the retained conformance composition must still instantiate all of its surfaces"
    );
    assert_eq!(
        conformance_ids
            .union(&observer_ids)
            .copied()
            .collect::<HashSet<_>>(),
        SurfaceId::ALL.into_iter().collect(),
        "the actual observer and retained conformance compositions cover the full manifest"
    );

    let world = observer.world_mut();
    let mesh_surfaces: HashSet<_> = world
        .query_filtered::<&DeclaredSurface, With<Mesh3d>>()
        .iter(world)
        .map(|surface| surface.id)
        .collect();
    assert_eq!(
        mesh_surfaces,
        HashSet::from([SurfaceId::ObserverShell, SurfaceId::ObserverProduction]),
        "the observer fixture must instantiate geographic and production meshes"
    );
    assert!(
        world
            .query::<&Text>()
            .iter(world)
            .any(|text| { text.0.contains("Surface fixture dispatch") }),
        "the history fixture must instantiate a committed-event button"
    );
}

fn visual_surface_ids(world: &mut World) -> HashSet<SurfaceId> {
    let declared_ids: HashSet<_> = world
        .query::<&DeclaredSurface>()
        .iter(world)
        .map(|declared| declared.id)
        .collect();
    let mut text_query = world.query::<(Entity, &Text, Option<&DeclaredSurface>)>();
    for (entity, _, declared) in text_query.iter(world) {
        let declared = declared.unwrap_or_else(|| {
            panic!("shipped Text entity {entity:?} has no DecisionSurfaceContract declaration")
        });
        assert!(contract_for(declared.id).validate().is_ok());
    }

    let mut image_query = world.query::<(Entity, &ImageNode, Option<&DeclaredSurface>)>();
    for (entity, _, declared) in image_query.iter(world) {
        let declared = declared.unwrap_or_else(|| {
            panic!("shipped ImageNode entity {entity:?} has no DecisionSurfaceContract declaration")
        });
        assert!(contract_for(declared.id).validate().is_ok());
    }

    let mut map_query = world.query::<(Entity, &Mesh2d, Option<&DeclaredSurface>)>();
    for (entity, _, declared) in map_query.iter(world) {
        let declared = declared.unwrap_or_else(|| {
            panic!(
                "shipped county-map entity {entity:?} has no DecisionSurfaceContract declaration"
            )
        });
        assert_eq!(declared.id, SurfaceId::CountyMap);
    }

    let mut mesh_query = world.query::<(Entity, &Mesh3d, Option<&DeclaredSurface>)>();
    for (entity, _, declared) in mesh_query.iter(world) {
        let declared = declared.unwrap_or_else(|| {
            panic!("shipped Mesh3d entity {entity:?} has no DecisionSurfaceContract declaration")
        });
        assert!(contract_for(declared.id).validate().is_ok());
    }
    declared_ids
}

#[test]
fn current_client_cannot_claim_a_gameplay_gate() {
    assert!(
        SHIPPED_SURFACE_MANIFEST
            .iter()
            .all(|contract| !contract.satisfies_gameplay_gate()),
        "the current Bevy client is an administrative viewer with no player action"
    );
}

/// ADR249 R9, pinned as executable contract: the county dossier card is the
/// first Gameplay-role row — structurally complete, not exempt — but its
/// only action, Investigate, is declared visibly unavailable, so the row
/// validates yet stays outside every gameplay gate until Gate 5 flips one
/// action to `Available`. The sealed reason is the exact R6 placeholder
/// sentence the rendered card seals with.
#[test]
fn county_dossier_is_gameplay_role_but_gate_ineligible_until_an_action_opens() {
    use babylon_client::decision_surface::ActionAvailabilityV1;

    const OPEN: &[SurfaceActionV1] = &[SurfaceActionV1::available("investigate")];

    let contract = contract_for(SurfaceId::CountyDossier);
    assert_eq!(contract.role, DecisionSurfaceRole::Gameplay);
    assert!(!contract.admin_debug_exempt);
    assert!(contract.validate().is_ok());
    assert!(
        !contract.satisfies_gameplay_gate(),
        "every action sealed means no player agency: the row must not satisfy the gate"
    );

    let opened = DecisionSurfaceContract {
        actions: OPEN,
        ..*contract
    };
    assert!(
        opened.satisfies_gameplay_gate(),
        "flipping one action to Available is exactly the Gate 5 change"
    );

    let [investigate] = contract.actions else {
        panic!("the dossier row declares exactly one action");
    };
    assert_eq!(investigate.name(), "investigate");
    match investigate.availability() {
        ActionAvailabilityV1::Unavailable(reason) => assert_eq!(
            reason,
            babylon_client::ui::dossier_compose::INVESTIGATE_UNAVAILABLE_REASON,
            "the manifest seal and the card's R6 placeholder seal are one sentence"
        ),
        ActionAvailabilityV1::Available => {
            panic!("investigate must be visibly unavailable until Gate 5")
        }
    }
}
