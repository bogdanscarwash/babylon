use super::*;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_tick::material_state::MaterialStateV1;

fn persisted_graph_copy(original: &CampaignFoundationV1) -> CampaignFoundationV1 {
    persisted_graph_with_layout(original, original.content_bundle().layout()).unwrap()
}

fn persisted_graph_with_layout(
    original: &CampaignFoundationV1,
    layout: crate::FoundationContentLayout,
) -> Result<CampaignFoundationV1, RustPersistenceRuntimeErrorV2> {
    let bundle = original.content_bundle();
    CampaignFoundationV1::from_persisted(
        original.stable_graph_bytes().to_vec(),
        original.world_register_bytes().to_vec(),
        original.resolver_manifest_bytes().to_vec(),
        original.prepared_environment_bytes().to_vec(),
        std::str::from_utf8(original.replay_session_identity().as_bytes()).unwrap(),
        i64::from_be_bytes(original.rng_seed().to_be_bytes()),
        original.content_digest().defines_hash,
        original.content_digest().rules_hash,
        *original.reference_digest().as_bytes(),
        std::str::from_utf8(bundle.scenario_source_bytes()).unwrap(),
        bundle
            .prelude_source_bytes()
            .map(|bytes| std::str::from_utf8(bytes).unwrap()),
        std::str::from_utf8(bundle.rule_source_bytes()).unwrap(),
        bundle.defines_bytes(),
        bundle.reference_bundle_manifest_bytes(),
        sha256_of(original.canonical_bytes()),
        layout,
    )
}

fn stored_copy(original: &MaterialRuntimeFoundationV2) -> StoredMaterialFoundationV2 {
    StoredMaterialFoundationV2 {
        spec: original.spec.clone(),
        initial_register_bytes: original.register.canonical_bytes().to_vec(),
        foundation_bytes: original.canonical_bytes().to_vec(),
        foundation_digest: original.digest(),
        graph_foundation_digest: sha256_of(original.graph_foundation.canonical_bytes()),
    }
}

fn alternate_foundation() -> MaterialRuntimeFoundationV2 {
    let original = michigan_material_runtime_foundation_v2(
        crate::michigan_material::MichiganDeliveryPresetV1::Standard,
    )
    .unwrap();
    let bundle = original.graph_foundation.content_bundle();
    let source = std::str::from_utf8(bundle.scenario_source_bytes())
        .unwrap()
        .replace(
            "production/michigan-observer-v1",
            "fixture/stored-content-v2",
        );
    assert_ne!(source.as_bytes(), bundle.scenario_source_bytes());
    let graph = ReplayTickSession::new(
        &source,
        None,
        "",
        HypergraphStore::new(),
        ReplaySessionIdV1::try_from("fixture/stored-content-v2").unwrap(),
        ReplaySeed::new(9821),
        bundle.content_digest().clone(),
        bundle.reference_digest(),
        MaterialStateV1::try_new(crate::michigan_dynamic_hex_foundation_v1().unwrap()).unwrap(),
    )
    .unwrap();
    let revised_bundle = FoundationContentBundleV1::try_new(
        &source,
        None,
        "",
        bundle.defines_bytes(),
        bundle.reference_bundle_manifest_bytes(),
    )
    .unwrap();
    MaterialRuntimeFoundationV2::capture(
        graph,
        revised_bundle,
        original.register.state().clone(),
        MaterialFoundationSpecV2 {
            content_digest: sha256_of(b"fixture/stored-content-v2"),
            ..original.spec.clone()
        },
    )
    .unwrap()
}

#[test]
fn stored_content_reconstructs_without_the_current_scenario_seed_or_preset_factory() {
    let original = alternate_foundation();
    let stored = stored_copy(&original);
    let digest = original.digest();
    let reconstructed = reconstruct_material_foundation_v2(
        stored,
        persisted_graph_copy(original.graph_foundation()),
        digest,
    )
    .unwrap();
    assert_eq!(reconstructed.canonical_bytes(), original.canonical_bytes());
    assert_eq!(
        reconstructed.initial_register(),
        original.initial_register()
    );
    assert_eq!(reconstructed.spec(), original.spec());
    let uninterrupted = original.into_session().unwrap();
    let reopened = reconstructed.into_session().unwrap();
    assert_eq!(
        reopened.graph_session().session_identity(),
        uninterrupted.graph_session().session_identity()
    );
    let actions = OrderedPracticeActionBatchV1::empty(
        uninterrupted.graph_session().session_identity().clone(),
        1,
    )
    .unwrap();
    let first = uninterrupted.prepare_advance(&actions).unwrap();
    let restored_first = reopened.prepare_advance(&actions).unwrap();
    assert_eq!(first.identity(), restored_first.identity());
    assert_eq!(
        first.material().register().canonical_bytes(),
        restored_first.material().register().canonical_bytes()
    );
    assert_eq!(
        first.material().receipt_bytes(),
        restored_first.material().receipt_bytes()
    );
}

#[test]
fn reconstruction_refuses_component_changes_and_an_unadmitted_expected_identity() {
    let original = michigan_material_runtime_foundation_v2(
        crate::michigan_material::MichiganDeliveryPresetV1::Standard,
    )
    .unwrap();
    let expected = original.digest();
    for mutation in 0..6 {
        let mut stored = stored_copy(&original);
        match mutation {
            0 => stored.spec.content_digest[0] ^= 1,
            1 => stored.spec.preset_id.push_str("-changed"),
            2 => stored.spec.horizon_ticks += 1,
            3 => stored.foundation_bytes[0] ^= 1,
            4 => stored.foundation_digest[0] ^= 1,
            5 => stored.graph_foundation_digest[0] ^= 1,
            _ => unreachable!(),
        }
        assert!(matches!(
            reconstruct_material_foundation_v2(
                stored,
                persisted_graph_copy(original.graph_foundation()),
                expected,
            ),
            Err(MaterialRuntimeErrorV3::FoundationMismatch)
        ));
    }
    assert!(matches!(
        reconstruct_material_foundation_v2(
            stored_copy(&original),
            persisted_graph_copy(original.graph_foundation()),
            [0; 32],
        ),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
}

#[test]
fn reconstruction_rejects_a_different_valid_graph_and_a_nonzero_initial_register() {
    let original = michigan_material_runtime_foundation_v2(
        crate::michigan_material::MichiganDeliveryPresetV1::Standard,
    )
    .unwrap();
    let alternate = alternate_foundation();
    let mut mixed = stored_copy(&original);
    mixed.graph_foundation_digest = sha256_of(alternate.graph_foundation().canonical_bytes());
    assert!(matches!(
        reconstruct_material_foundation_v2(
            mixed,
            persisted_graph_copy(alternate.graph_foundation()),
            original.digest(),
        ),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
    let mut stored = stored_copy(&original);
    let graph = persisted_graph_copy(original.graph_foundation());
    let expected = original.digest();
    let session = original.into_session().unwrap();
    let actions =
        OrderedPracticeActionBatchV1::empty(session.graph_session().session_identity().clone(), 1)
            .unwrap();
    stored.initial_register_bytes = session
        .prepare_advance(&actions)
        .unwrap()
        .material()
        .register()
        .canonical_bytes()
        .to_vec();
    assert!(matches!(
        reconstruct_material_foundation_v2(stored, graph, expected),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
}

#[test]
fn persisted_layout_is_exact_even_when_both_encoders_accept_the_source() {
    let original = alternate_foundation();
    assert!(matches!(
        persisted_graph_with_layout(
            original.graph_foundation(),
            crate::FoundationContentLayout::V2
        ),
        Err(RustPersistenceRuntimeErrorV2::ReplaySource)
    ));
    assert_eq!(
        crate::FoundationContentLayout::from_persisted(1).unwrap(),
        crate::FoundationContentLayout::V1
    );
    assert_eq!(
        crate::FoundationContentLayout::from_persisted(2).unwrap(),
        crate::FoundationContentLayout::V2
    );
    for tag in [-1, 0, 3, i16::MAX] {
        assert_eq!(
            crate::FoundationContentLayout::from_persisted(tag),
            Err(RustPersistenceRuntimeErrorV2::ReplaySource)
        );
    }
}

#[test]
fn large_v2_stored_sources_reconstruct_the_same_circuit_without_factory_substitution() {
    let original = crate::michigan_content::MichiganContentPresetV1::CohortsStandardV2
        .create_foundation()
        .unwrap();
    assert_eq!(
        original.graph_foundation().content_bundle().layout(),
        crate::FoundationContentLayout::V2
    );
    assert!(
        original
            .graph_foundation()
            .content_bundle()
            .scenario_source_bytes()
            .len()
            > 65_535
    );
    assert!(persisted_graph_with_layout(
        original.graph_foundation(),
        crate::FoundationContentLayout::V1
    )
    .is_err());
    let restored = reconstruct_material_foundation_v2(
        stored_copy(&original),
        persisted_graph_copy(original.graph_foundation()),
        original.digest(),
    )
    .unwrap();
    assert_eq!(restored.canonical_bytes(), original.canonical_bytes());
    let continued = original.into_session().unwrap();
    let reopened = restored.into_session().unwrap();
    let actions = OrderedPracticeActionBatchV1::empty(
        continued.graph_session().session_identity().clone(),
        1,
    )
    .unwrap();
    let left = continued.prepare_advance(&actions).unwrap();
    let right = reopened.prepare_advance(&actions).unwrap();
    assert_eq!(
        left.material().register().canonical_bytes(),
        right.material().register().canonical_bytes()
    );
    assert_eq!(left.identity(), right.identity());
}

#[test]
fn admitted_bundle_foundations_reconstruct_exactly_through_dispatch_transit_and_arrival() {
    use crate::michigan_content::MichiganContentPresetV1;
    for preset in [
        MichiganContentPresetV1::BundlesStandardV3,
        MichiganContentPresetV1::BundlesDelayedV3,
    ] {
        let original = preset.create_foundation().unwrap();
        let restored = reconstruct_material_foundation_v2(
            stored_copy(&original),
            persisted_graph_copy(original.graph_foundation()),
            preset.admitted().unwrap().digest(),
        )
        .unwrap();
        assert_eq!(restored.canonical_bytes(), original.canonical_bytes());
        assert_eq!(
            restored.graph_foundation().content_bundle().defines_bytes(),
            original.graph_foundation().content_bundle().defines_bytes()
        );
        let mut continued = original.into_session().unwrap();
        let mut reopened = restored.into_session().unwrap();
        let mut left_sink = CollectingSink::default();
        let mut right_sink = CollectingSink::default();
        for week in 1..=6 {
            let actions = OrderedPracticeActionBatchV1::empty(
                continued.graph_session().session_identity().clone(),
                week,
            )
            .unwrap();
            let left = continued.prepare_advance(&actions).unwrap();
            let right = reopened.prepare_advance(&actions).unwrap();
            assert_eq!(left.identity(), right.identity());
            assert_eq!(
                left.material().register().canonical_bytes(),
                right.material().register().canonical_bytes()
            );
            assert_eq!(
                left.material().receipt_bytes(),
                right.material().receipt_bytes()
            );
            continued
                .commit_prepared_and_publish(&mut left_sink, left, |_| {
                    Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
                })
                .unwrap();
            reopened
                .commit_prepared_and_publish(&mut right_sink, right, |_| {
                    Ok::<_, ()>(ReplayCommitDispositionV1::Committed)
                })
                .unwrap();
        }
    }
}

#[test]
fn bundle_reconstruction_refuses_alternate_content_and_individually_valid_changed_stock() {
    use crate::michigan_content::MichiganContentPresetV1;
    let preset = MichiganContentPresetV1::BundlesStandardV3;
    let original = preset.create_foundation().unwrap();
    let expected = preset.admitted().unwrap().digest();
    let previous = MichiganContentPresetV1::CohortsStandardV2
        .create_foundation()
        .unwrap();
    // The old graph has the same subjects and opening material, but lacks the
    // independently admitted executable bundle content and cannot replace it.
    let mut changed = stored_copy(&original);
    changed.graph_foundation_digest = sha256_of(previous.graph_foundation().canonical_bytes());
    assert!(matches!(
        reconstruct_material_foundation_v2(
            changed,
            persisted_graph_copy(previous.graph_foundation()),
            expected
        ),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
    let mut changed = stored_copy(&original);
    let mut state = original.initial_register().state().clone();
    state.inventory[0].quantity += 1;
    changed.initial_register_bytes = MaterialWorldRegisterV2::try_new(0, state)
        .unwrap()
        .canonical_bytes()
        .to_vec();
    assert!(matches!(
        reconstruct_material_foundation_v2(
            changed,
            persisted_graph_copy(original.graph_foundation()),
            expected
        ),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
    // A whole valid bundle successor cannot nominate its own trust anchor.
    assert!(matches!(
        reconstruct_material_foundation_v2(
            stored_copy(&original),
            persisted_graph_copy(original.graph_foundation()),
            previous.digest()
        ),
        Err(MaterialRuntimeErrorV3::FoundationMismatch)
    ));
}
