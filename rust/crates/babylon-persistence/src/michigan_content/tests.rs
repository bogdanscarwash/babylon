use super::*;
use babylon_tick::material_replay::MaterialLaborV1;

#[test]
fn current_staffed_foundation_keeps_observed_cohorts_separate_from_five_designed_pools() {
    for preset in MICHIGAN_CONTENT_PRESETS_V1 {
        let foundation = preset
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let expected = preset.admitted(&crate::test_support::catalog()).unwrap();
        assert_eq!(foundation.canonical_bytes(), expected.canonical_bytes);
        assert_eq!(foundation.initial_register(), &expected.register);
        assert_eq!(expected.horizon_ticks, 16);
        assert_eq!(
            expected.physical_projection,
            MichiganPhysicalProjectionV1::FiveProcessV1
        );
        let source = std::str::from_utf8(
            foundation
                .graph_foundation()
                .content_bundle()
                .scenario_source_bytes(),
        )
        .unwrap();
        assert_eq!(source.matches("(node business-").count(), 1_603);
        assert_eq!(source.matches("(hyperedge sector-").count(), 19);
        assert_eq!(source.matches("(node workforce-").count(), 5);
        assert_eq!(source.matches("(deffield social-class/").count(), 3);
        assert!(!crate::michigan_cohorts::michigan_cohorts_v2()
            .unwrap()
            .scenario_source()
            .contains("SOCIAL_CLASS"));
        let MaterialLaborV1::Staffed(composition) = foundation.labor() else {
            panic!("staffed authority required")
        };
        assert_eq!(composition.bindings().len(), 5);
        assert_eq!(foundation.initial_register().state().labor.len(), 5);
        assert!(foundation
            .initial_register()
            .state()
            .labor
            .iter()
            .all(|row| row.period == 1));
        assert_eq!(foundation.initial_register().state().capacities.len(), 80);
        assert_eq!(
            MichiganContentPresetV1::new_campaign(preset.delivery()),
            preset
        );
    }
}

#[test]
fn unsupported_michigan_saves_are_refused_without_a_predecessor_factory() {
    for version in 1..=5 {
        for delivery in ["standard", "delayed"] {
            let id = format!("michigan-material-{delivery}-v{version}");
            assert_eq!(MichiganContentPresetV1::from_id(&id), None);
            assert!(matches!(
                admit_michigan_content_v1(&id, 16, &[0; 32], &[0; 32], 0, &[]),
                Err(MichiganContentErrorV1::UnknownPreset)
            ));
        }
    }
}

#[test]
fn admission_refuses_mixed_headers_graphs_and_unadmitted_versions() {
    for preset in MICHIGAN_CONTENT_PRESETS_V1 {
        let expected = preset.admitted(&crate::test_support::catalog()).unwrap();
        let reopened = admit_michigan_content_v1(
            preset.id(),
            16,
            &expected.content_digest,
            &expected.digest,
            16,
            &expected.canonical_bytes,
        )
        .unwrap();
        assert_eq!(reopened.canonical_bytes, expected.canonical_bytes);
        for tick in [0, 16] {
            assert!(expected
                .validate_header(16, &expected.content_digest, &expected.digest, tick)
                .is_ok());
        }
        for horizon in [-1, 0, 15, 17] {
            assert_eq!(
                expected.validate_header(horizon, &expected.content_digest, &expected.digest, 0),
                Err(MichiganContentErrorV1::IdentityMismatch)
            );
        }
        assert_eq!(
            expected.validate_header(16, &expected.content_digest, &expected.digest, 17),
            Err(MichiganContentErrorV1::IdentityMismatch)
        );
        for other in MICHIGAN_CONTENT_PRESETS_V1 {
            if other == preset {
                continue;
            }
            let mixed = other.admitted(&crate::test_support::catalog()).unwrap();
            assert!(admit_michigan_content_v1(
                preset.id(),
                16,
                &mixed.content_digest,
                &mixed.digest,
                0,
                &expected.canonical_bytes
            )
            .is_err());
            if expected.graph_digest != mixed.graph_digest {
                assert!(expected
                    .validate_graph(&mixed.graph_digest, &expected.scenario_digest)
                    .is_err());
            }
            if expected.scenario_digest != mixed.scenario_digest {
                assert!(expected
                    .validate_graph(&expected.graph_digest, &mixed.scenario_digest)
                    .is_err());
            }
        }
        assert!(admit_michigan_content_v1(
            "michigan-material-standard-v7",
            16,
            &expected.content_digest,
            &expected.digest,
            0,
            &expected.canonical_bytes
        )
        .is_err());
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &expected.content_digest[..31],
            &expected.digest,
            0,
            &expected.canonical_bytes
        )
        .is_err());
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &expected.content_digest,
            &expected.digest[..31],
            0,
            &expected.canonical_bytes
        )
        .is_err());
    }
}

#[test]
fn edited_parameters_change_new_foundations_but_stored_campaign_keeps_its_own_values() {
    let catalog = crate::test_support::catalog();
    let preset = MichiganContentPresetV1::FourWeekStandardV6;
    let original = preset.admitted(&catalog).unwrap();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    ));
    let edited = MichiganMaterialCatalogV1::from_defines_toml(
        &source
            .replace(
                "WORK_HOURS_PER_PERSON_WEEK = 40",
                "WORK_HOURS_PER_PERSON_WEEK = 45",
            )
            .replace("OPENING_INPUT_UNITS = 600", "OPENING_INPUT_UNITS = 700")
            .replace("HORIZON_PERIODS = 16", "HORIZON_PERIODS = 8"),
    )
    .unwrap();
    let next = preset.admitted(&edited).unwrap();
    assert_ne!(original.digest, next.digest);
    assert_eq!(next.horizon_ticks, 8);
    assert!(next
        .validate_header(8, &next.content_digest, &next.digest, 8)
        .is_ok());
    assert!(next
        .validate_header(8, &next.content_digest, &next.digest, 9)
        .is_err());
    assert_eq!(edited.staffing().hours_per_worker_period, 180);
    assert_eq!(
        edited
            .processes()
            .iter()
            .find(|p| p.key == "sheet-rolling")
            .unwrap()
            .labor_capacity_hours_per_period,
        3600
    );
    let reopened = admit_michigan_content_v1(
        preset.id(),
        16,
        &original.content_digest,
        &original.digest,
        0,
        &original.canonical_bytes,
    )
    .unwrap();
    assert_eq!(reopened.catalog.defines_bytes(), catalog.defines_bytes());
    assert_eq!(reopened.catalog.staffing().hours_per_worker_period, 160);
    assert_eq!(reopened.register, original.register);
    assert!(admit_michigan_content_v1(
        preset.id(),
        16,
        &next.content_digest,
        &next.digest,
        0,
        &original.canonical_bytes
    )
    .is_err());
    let mut corrupted = original.canonical_bytes.clone();
    let end = corrupted.len() - 1;
    corrupted[end] ^= 1;
    assert!(admit_michigan_content_v1(
        preset.id(),
        16,
        &original.content_digest,
        &original.digest,
        0,
        &corrupted
    )
    .is_err());
    for length in [0, 32, original.canonical_bytes.len() - 1] {
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &original.content_digest,
            &original.digest,
            0,
            &original.canonical_bytes[..length]
        )
        .is_err());
    }
}

#[test]
fn stored_shared_freight_capacities_reconstruct_without_current_default_substitution() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    ));
    let defaults = crate::test_support::catalog();
    let authored = MichiganMaterialCatalogV1::from_defines_toml(
        &source
            .replace("AMPLE_UNITS_PER_WEEK = 200", "AMPLE_UNITS_PER_WEEK = 201")
            .replace(
                "CONSTRAINED_UNITS_PER_WEEK = 40",
                "CONSTRAINED_UNITS_PER_WEEK = 41",
            ),
    )
    .unwrap();
    for (preset, capacity) in [
        (MichiganContentPresetV1::SharedFreightAmpleV6, 804),
        (MichiganContentPresetV1::SharedFreightConstrainedV6, 164),
    ] {
        let original = preset.admitted(&authored).unwrap();
        let default_campaign = preset.admitted(&defaults).unwrap();
        assert_ne!(original.digest, default_campaign.digest);
        let reopened = admit_michigan_content_v1(
            preset.id(),
            16,
            &original.content_digest,
            &original.digest,
            0,
            &original.canonical_bytes,
        )
        .unwrap();
        assert_eq!(reopened.catalog.defines_bytes(), authored.defines_bytes());
        assert_ne!(reopened.catalog.defines_bytes(), defaults.defines_bytes());
        assert_eq!(reopened.register, original.register);
        assert_eq!(reopened.canonical_bytes, original.canonical_bytes);
        let sheet = reopened
            .catalog
            .routes()
            .iter()
            .find(|route| route.key == "sheet-transfer")
            .unwrap();
        let shared = reopened
            .catalog
            .corridor_for_route(sheet, preset.delivery())
            .unwrap();
        assert_eq!(shared.capacity_per_period(preset.delivery()), capacity);
        let capacities: Vec<_> = reopened
            .register
            .state()
            .corridor_capacities
            .iter()
            .filter(|row| row.corridor_id == shared.id())
            .collect();
        assert_eq!(capacities.len(), 16);
        assert!(capacities.iter().all(|row| row.available == capacity));
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &default_campaign.content_digest,
            &default_campaign.digest,
            0,
            &original.canonical_bytes,
        )
        .is_err());
    }
}
