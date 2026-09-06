use super::*;
use babylon_tick::material_replay::MaterialLaborV1;

#[test]
fn current_staffed_foundation_keeps_observed_cohorts_separate_from_five_designed_pools() {
    for preset in MICHIGAN_CONTENT_PRESETS_V1 {
        let foundation = preset.create_foundation().unwrap();
        let expected = preset.admitted().unwrap();
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
            .all(|row| row.week == 1));
        assert_eq!(foundation.initial_register().state().capacities.len(), 80);
        assert_eq!(
            MichiganContentPresetV1::new_campaign(preset.delivery()),
            preset
        );
    }
}

#[test]
fn unsupported_michigan_saves_are_refused_without_a_predecessor_factory() {
    for version in 1..=3 {
        for delivery in ["standard", "delayed"] {
            let id = format!("michigan-material-{delivery}-v{version}");
            assert_eq!(MichiganContentPresetV1::from_id(&id), None);
            assert!(matches!(
                admit_michigan_content_v1(&id, 16, &[0; 32], &[0; 32], 0),
                Err(MichiganContentErrorV1::UnknownPreset)
            ));
        }
    }
}

#[test]
fn admission_refuses_mixed_headers_graphs_and_unadmitted_versions() {
    for preset in MICHIGAN_CONTENT_PRESETS_V1 {
        let expected = preset.admitted().unwrap();
        assert!(std::ptr::eq(
            expected,
            admit_michigan_content_v1(
                preset.id(),
                16,
                &expected.content_digest,
                &expected.digest,
                16
            )
            .unwrap()
        ));
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
            let mixed = other.admitted().unwrap();
            assert!(admit_michigan_content_v1(
                preset.id(),
                16,
                &mixed.content_digest,
                &mixed.digest,
                0
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
            "michigan-material-standard-v5",
            16,
            &expected.content_digest,
            &expected.digest,
            0
        )
        .is_err());
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &expected.content_digest[..31],
            &expected.digest,
            0
        )
        .is_err());
        assert!(admit_michigan_content_v1(
            preset.id(),
            16,
            &expected.content_digest,
            &expected.digest[..31],
            0
        )
        .is_err());
    }
}
