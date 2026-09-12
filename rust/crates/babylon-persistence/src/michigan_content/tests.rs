use super::*;

const REGIONAL_PRESETS: [MichiganContentPreset; 4] = [
    MichiganContentPreset::FourWeekStandard,
    MichiganContentPreset::FourWeekDelayed,
    MichiganContentPreset::SharedFreightAmple,
    MichiganContentPreset::SharedFreightConstrained,
];

#[test]
fn current_staffed_foundation_keeps_observed_cohorts_separate_from_five_designed_pools() {
    for preset in REGIONAL_PRESETS {
        let foundation = preset
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let expected = preset.admitted(&crate::test_support::catalog()).unwrap();
        assert_eq!(foundation.canonical_bytes(), expected.canonical_bytes);
        assert_eq!(foundation.initial_register(), &expected.register);
        assert_eq!(expected.horizon_ticks, 16);
        assert_eq!(
            expected.physical_projection,
            MichiganPhysicalProjection::Normalized
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
        assert!(!crate::michigan_cohorts::michigan_cohorts()
            .unwrap()
            .scenario_source()
            .contains("SOCIAL_CLASS"));
        let composition = foundation.labor();
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
            MichiganContentPreset::new_campaign(preset.delivery()),
            preset
        );
    }
}

#[test]
fn unsupported_michigan_saves_are_refused_without_a_predecessor_factory() {
    for version in 1..=6 {
        for delivery in [
            "standard",
            "delayed",
            "shared-freight-ample",
            "shared-freight-constrained",
        ] {
            let id = format!("michigan-material-{delivery}-v{version}");
            assert_eq!(MichiganContentPreset::from_id(&id), None);
            assert!(matches!(
                admit_michigan_content(&id, 16, &[0; 32], &[0; 32], 0, &[]),
                Err(MichiganContentError::UnknownPreset)
            ));
        }
    }
}

#[test]
fn admission_refuses_mixed_headers_graphs_and_unadmitted_versions() {
    for preset in REGIONAL_PRESETS {
        let expected = preset.admitted(&crate::test_support::catalog()).unwrap();
        let reopened = admit_michigan_content(
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
                Err(MichiganContentError::IdentityMismatch)
            );
        }
        assert_eq!(
            expected.validate_header(16, &expected.content_digest, &expected.digest, 17),
            Err(MichiganContentError::IdentityMismatch)
        );
        for other in REGIONAL_PRESETS {
            if other == preset {
                continue;
            }
            let mixed = other.admitted(&crate::test_support::catalog()).unwrap();
            assert!(admit_michigan_content(
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
        assert!(admit_michigan_content(
            "michigan-material-standard-v8",
            16,
            &expected.content_digest,
            &expected.digest,
            0,
            &expected.canonical_bytes
        )
        .is_err());
        assert!(admit_michigan_content(
            preset.id(),
            16,
            &expected.content_digest[..31],
            &expected.digest,
            0,
            &expected.canonical_bytes
        )
        .is_err());
        assert!(admit_michigan_content(
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
    let preset = MichiganContentPreset::FourWeekStandard;
    let original = preset.admitted(&catalog).unwrap();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../content/scenarios/michigan/defines.toml"
    ));
    let edited = MichiganMaterialCatalog::from_defines_toml(
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
        next.register
            .state()
            .labor
            .iter()
            .find(|row| {
                row.site_id
                    == edited
                        .processes()
                        .iter()
                        .find(|process| process.key == "sheet-rolling")
                        .unwrap()
                        .site_id()
            })
            .unwrap()
            .available,
        3600
    );
    let reopened = admit_michigan_content(
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
    assert!(admit_michigan_content(
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
    assert!(admit_michigan_content(
        preset.id(),
        16,
        &original.content_digest,
        &original.digest,
        0,
        &corrupted
    )
    .is_err());
    for length in [0, 32, original.canonical_bytes.len() - 1] {
        assert!(admit_michigan_content(
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
    let authored = MichiganMaterialCatalog::from_defines_toml(
        &source
            .replace("AMPLE_UNITS_PER_WEEK = 200", "AMPLE_UNITS_PER_WEEK = 201")
            .replace(
                "CONSTRAINED_UNITS_PER_WEEK = 40",
                "CONSTRAINED_UNITS_PER_WEEK = 41",
            ),
    )
    .unwrap();
    for (preset, capacity) in [
        (MichiganContentPreset::SharedFreightAmple, 804),
        (MichiganContentPreset::SharedFreightConstrained, 164),
    ] {
        let original = preset.admitted(&authored).unwrap();
        let default_campaign = preset.admitted(&defaults).unwrap();
        assert_ne!(original.digest, default_campaign.digest);
        let reopened = admit_michigan_content(
            preset.id(),
            16,
            &original.content_digest,
            &original.digest,
            0,
            &original.canonical_bytes,
        )
        .unwrap();
        assert_eq!(
            reopened.catalog.defines_bytes(),
            authored
                .with_preset(preset.delivery())
                .unwrap()
                .defines_bytes()
        );
        assert_ne!(reopened.catalog.defines_bytes(), defaults.defines_bytes());
        assert_eq!(reopened.register, original.register);
        assert_eq!(reopened.canonical_bytes, original.canonical_bytes);
        let sheet = reopened
            .catalog
            .routes()
            .iter()
            .find(|route| route.key == "sheet-transfer")
            .unwrap();
        let crate::michigan_material::MichiganMaterialPath::Routed { capacity_keys, .. } =
            &sheet.path
        else {
            panic!("regional transfer requires freight");
        };
        assert_eq!(capacity_keys.len(), 1);
        let shared = reopened
            .catalog
            .corridors()
            .iter()
            .find(|row| row.key == capacity_keys[0])
            .unwrap();
        assert_eq!(shared.capacity_grams_per_period, capacity * 1000);
        let capacities: Vec<_> = reopened
            .register
            .state()
            .corridor_capacities
            .iter()
            .filter(|row| row.corridor_id == shared.id())
            .collect();
        assert_eq!(capacities.len(), 16);
        assert!(capacities
            .iter()
            .all(|row| row.available_grams == capacity * 1000));
        assert!(admit_michigan_content(
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

#[test]
fn statewide_presets_refuse_an_unqualified_regional_catalog() {
    let catalog = crate::test_support::catalog();
    for preset in MICHIGAN_CONTENT_PRESETS
        .into_iter()
        .filter(|preset| !REGIONAL_PRESETS.contains(preset))
    {
        assert!(preset.create_foundation(&catalog).is_err());
        assert!(preset.admitted(&catalog).is_err());
    }
}
