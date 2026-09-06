use super::*;

#[test]
fn standard_v1_matches_foundations_saved_before_cohort_content_evolution() {
    // Independently read from three pre-existing native QA saves on 2026-09-05,
    // before the V2 content format was installed. Do not regenerate these pins
    // from the current factory: they protect continuation of those old worlds.
    let baseline = MichiganContentPresetV1::BaselineStandardV1
        .admitted()
        .unwrap();
    assert_eq!(
        crate::michigan_economy::digest_hex(&baseline.digest),
        "6c24dfb1cdd1ca2b6fe19f99a5c44c8f413043c4ee61dbead816285de84e0695"
    );
    assert_eq!(
        crate::michigan_economy::digest_hex(&baseline.graph_digest),
        "a5b141825fa5199eddc27a0f0e4f58a30b11a70facac3544c5d646a01fb319f3"
    );
}

#[test]
fn old_and_new_graphs_keep_exact_v1_foundations_and_the_same_physical_catalog() {
    for (old, new) in [
        (
            MichiganContentPresetV1::BaselineStandardV1,
            MichiganContentPresetV1::CohortsStandardV2,
        ),
        (
            MichiganContentPresetV1::BaselineDelayedV1,
            MichiganContentPresetV1::CohortsDelayedV2,
        ),
    ] {
        let legacy = michigan_material_runtime_foundation_v2(old.delivery()).unwrap();
        let baseline = old.admitted().unwrap();
        let cohorts = new.admitted().unwrap();
        assert_eq!(baseline.canonical_bytes, legacy.canonical_bytes());
        assert_eq!(baseline.digest, legacy.digest());
        assert_eq!(baseline.register, cohorts.register);
        assert_eq!(baseline.horizon_ticks, 16);
        assert_eq!(cohorts.horizon_ticks, 16);
        assert_eq!(
            cohorts.physical_projection,
            MichiganPhysicalProjectionV1::FiveProcessV1
        );
        assert_ne!(baseline.digest, cohorts.digest);
        assert_ne!(baseline.content_digest, cohorts.content_digest);
        assert_ne!(baseline.graph_digest, cohorts.graph_digest);
        assert_ne!(baseline.scenario_digest, cohorts.scenario_digest);
        let created = new.create_foundation().unwrap();
        assert_eq!(created.canonical_bytes(), cohorts.canonical_bytes);
        assert_eq!(created.initial_register(), &baseline.register);
        let source = std::str::from_utf8(
            created
                .graph_foundation()
                .content_bundle()
                .scenario_source_bytes(),
        )
        .unwrap();
        assert!(source.starts_with(&format!("(scenario {}\n", new.scenario())));
        assert_eq!(source.matches("(node business-").count(), 1_603);
        assert_eq!(source.matches("(hyperedge sector-").count(), 19);
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
            "michigan-material-standard-v4",
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

#[test]
fn executable_bundles_change_content_identity_without_rewriting_v2_sources_or_physics() {
    use crate::sector_bundle::foundation::decode_stored_bundle_defines_v1;
    for (previous, current) in [
        (
            MichiganContentPresetV1::CohortsStandardV2,
            MichiganContentPresetV1::BundlesStandardV3,
        ),
        (
            MichiganContentPresetV1::CohortsDelayedV2,
            MichiganContentPresetV1::BundlesDelayedV3,
        ),
    ] {
        let old = previous.create_foundation().unwrap();
        let new = current.create_foundation().unwrap();
        let old_bundle = old.graph_foundation().content_bundle();
        let new_bundle = new.graph_foundation().content_bundle();
        let decoded = decode_stored_bundle_defines_v1(
            new_bundle.defines_bytes(),
            new.graph_foundation().content_digest().defines_hash,
        )
        .unwrap();
        assert_eq!(decoded.observed_defines(), old_bundle.defines_bytes());
        assert_eq!(
            new_bundle.scenario_source_bytes(),
            old_bundle.scenario_source_bytes()
        );
        assert_eq!(
            new_bundle.reference_bundle_manifest_bytes(),
            old_bundle.reference_bundle_manifest_bytes()
        );
        assert_eq!(new.initial_register(), old.initial_register());
        assert_ne!(new.digest(), old.digest());
        assert_ne!(new.spec().content_digest, old.spec().content_digest);
        assert_ne!(
            new.graph_foundation().canonical_bytes(),
            old.graph_foundation().canonical_bytes()
        );
        assert_eq!(decoded.bundles().len(), 4);
        // The same observed scenario identifies every cohort in both revisions.
        assert_eq!(
            current.admitted().unwrap().scenario_digest,
            previous.admitted().unwrap().scenario_digest
        );
        assert_eq!(
            previous.create_foundation().unwrap().canonical_bytes(),
            old.canonical_bytes()
        );
    }
}

#[test]
fn new_creation_and_old_identity_are_distinct_from_logical_delivery() {
    for preset in MICHIGAN_CONTENT_PRESETS_V1 {
        assert_eq!(MichiganContentPresetV1::from_id(preset.id()), Some(preset));
        let new = MichiganContentPresetV1::new_campaign(preset.delivery());
        assert_eq!(new.delivery(), preset.delivery());
        assert_eq!(new.scenario(), MICHIGAN_COHORT_SCENARIO_V2);
        assert!(new.id().ends_with("-v3"));
    }
    assert_eq!(MichiganContentPresetV1::from_id("standard"), None);
    assert_eq!(
        MichiganContentPresetV1::from_id("michigan-material-standard-v01"),
        None
    );
}
