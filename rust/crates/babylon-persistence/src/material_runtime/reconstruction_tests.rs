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
    let original = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let bundle = original.graph_foundation.content_bundle();
    let source = std::str::from_utf8(bundle.scenario_source_bytes()).unwrap();
    let graph = ReplayTickSession::new(
        source,
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
    let revised_bundle = FoundationContentBundleV2::try_new(
        source,
        None,
        "",
        bundle.defines_bytes(),
        bundle.reference_bundle_manifest_bytes(),
    )
    .unwrap();
    MaterialRuntimeFoundationV2::capture_v2(
        graph,
        revised_bundle,
        original.register.state().clone(),
        original.spec.clone(),
    )
    .unwrap()
}

#[test]
fn stored_content_reconstructs_exact_alternate_session_and_seed_without_factory_substitution() {
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
    let original = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .create_foundation(&crate::test_support::catalog())
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
    let original = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .create_foundation(&crate::test_support::catalog())
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
fn persisted_layout_is_exact_and_unknown_layouts_are_refused() {
    let original = alternate_foundation();
    let content = original.graph_foundation().content_bundle();
    // The current staffed source exceeds V1's field bound, so its wrong-layout
    // decode refuses before reaching the whole-foundation digest comparison.
    assert!(content.scenario_source_bytes().len() > 65_535);
    assert!(matches!(
        crate::semantic_codec::encode_foundation_content(
            std::str::from_utf8(content.scenario_source_bytes()).unwrap(),
            None,
            "",
            content.defines_bytes(),
            content.reference_bundle_manifest_bytes(),
        ),
        Err(crate::semantic_codec::SemanticCodecErrorV1::Refusal(
            crate::semantic_codec::SemanticRefusalCodeV1::FieldByteBound
        ))
    ));
    assert_eq!(
        persisted_graph_with_layout(
            original.graph_foundation(),
            crate::FoundationContentLayout::V1
        )
        .map(|_| ()),
        Err(RustPersistenceRuntimeErrorV2::SemanticCodec)
    );
    // A smaller graph-only foundation fits both source formats. Its encoded
    // layout still participates in the identity and cannot be substituted.
    let (graph, content) = crate::michigan_economy::michigan_observer_foundation_v1().unwrap();
    let small = CampaignFoundationV1::capture(&graph, content).unwrap();
    assert!(small.content_bundle().scenario_source_bytes().len() <= 65_535);
    assert_eq!(
        persisted_graph_with_layout(&small, crate::FoundationContentLayout::V2).map(|_| ()),
        Err(RustPersistenceRuntimeErrorV2::ReplaySource)
    );
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
    let original = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .create_foundation(&crate::test_support::catalog())
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
        MichiganContentPresetV1::FourWeekStandardV6,
        MichiganContentPresetV1::FourWeekDelayedV6,
    ] {
        let original = preset
            .create_foundation(&crate::test_support::catalog())
            .unwrap();
        let restored = reconstruct_material_foundation_v2(
            stored_copy(&original),
            persisted_graph_copy(original.graph_foundation()),
            preset
                .admitted(&crate::test_support::catalog())
                .unwrap()
                .digest(),
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
        for period in 1..=6 {
            let actions = OrderedPracticeActionBatchV1::empty(
                continued.graph_session().session_identity().clone(),
                period,
            )
            .unwrap();
            let left = continued.prepare_advance(&actions).unwrap();
            let right = reopened.prepare_advance(&actions).unwrap();
            assert_eq!(
                left.graph_report().successful_event_batch().events(),
                right.graph_report().successful_event_batch().events()
            );
            if period == 1 {
                assert_workforce_seed_evidence(&left);
            }
            if preset == MichiganContentPresetV1::FourWeekDelayedV6 {
                assert_delayed_panel_retention(&left, period);
            }
            let checkpoint =
                matches!(period, 2..=4).then(|| reconstructed_checkpoint(preset, &left));
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
            if let Some(checkpoint) = checkpoint {
                reopened = checkpoint;
            }
        }
    }
}

#[test]
fn bundle_reconstruction_refuses_alternate_content_and_individually_valid_changed_stock() {
    use crate::michigan_content::MichiganContentPresetV1;
    let preset = MichiganContentPresetV1::FourWeekStandardV6;
    let original = preset
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let expected = preset
        .admitted(&crate::test_support::catalog())
        .unwrap()
        .digest();
    let previous = alternate_foundation();
    // A valid graph with a different seed/session cannot replace the admitted one.
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

fn assert_delayed_panel_retention(
    candidate: &babylon_tick::material_replay::PreparedMaterialTickV3<HypergraphStore>,
    period: u64,
) {
    use babylon_bsl::identity_codec::StableBslValueV1;
    use babylon_graph::stable_element::StableElementKeyV1;
    let events = candidate.graph_report().successful_event_batch().events();
    assert_eq!(events.len(), 5);
    let panel = events.iter().find(|event| event.fields().iter().any(|(key,value)| {
        key == "subject" && matches!(value, StableBslValueV1::Node(StableElementKeyV1::Node{local_name,..}) if local_name == "workforce-panel-forming")
    })).unwrap();
    let field = |name| match &panel
        .fields()
        .iter()
        .find(|(key, _)| key == name)
        .unwrap()
        .1
    {
        StableBslValueV1::Int(value) => *value,
        _ => panic!("staffing evidence must be exact integer"),
    };
    assert_eq!(field("closing-employed") + field("closing-reserve"), 4);
    if period == 1 {
        assert_eq!(field("previous-unretained-hours"), 640);
        assert_eq!(field("current-unretained-hours"), 0);
        assert_eq!(field("closing-employed"), 4);
    } else if period == 2 {
        assert_eq!(field("previous-unretained-hours"), 0);
        assert_eq!(field("closing-employed"), 0);
        assert_eq!(field("separations"), 4);
    } else if period == 4 {
        assert_eq!(field("current-unretained-hours"), 640);
        assert_eq!(field("closing-employed"), 4);
        assert_eq!(field("hires"), 4);
    }
}

#[test]
fn unwrapped_definitions_and_changed_opening_workforce_are_not_scheduled_fallbacks() {
    let current = crate::michigan_content::MichiganContentPresetV1::FourWeekStandardV6
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let original = current.graph_foundation().content_bundle();
    for change_seed in [false, true] {
        let source = std::str::from_utf8(original.scenario_source_bytes()).unwrap();
        let changed_source = source.replace(
            "(social-class/employed-population 20)",
            "(social-class/employed-population 19)",
        );
        let source = if change_seed {
            changed_source.as_str()
        } else {
            source
        };
        let defines = if change_seed {
            original.defines_bytes()
        } else {
            b"{}"
        };
        let bundle = FoundationContentBundleV2::try_new(
            source,
            None,
            "",
            defines,
            original.reference_bundle_manifest_bytes(),
        )
        .unwrap();
        let graph = ReplayTickSession::new(
            source,
            None,
            "",
            HypergraphStore::new(),
            ReplaySessionIdV1::try_from("fixture/unsupported-authority").unwrap(),
            ReplaySeed::new(319),
            bundle.content_digest().clone(),
            bundle.reference_digest(),
            MaterialStateV1::try_new(crate::michigan_dynamic_hex_foundation_v1().unwrap()).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            MaterialRuntimeFoundationV2::capture_v2(
                graph,
                bundle,
                current.register.state().clone(),
                current.spec.clone()
            ),
            Err(MaterialRuntimeErrorV3::FoundationMismatch)
        ));
    }
}

fn reconstructed_checkpoint(
    preset: crate::michigan_content::MichiganContentPresetV1,
    candidate: &babylon_tick::material_replay::PreparedMaterialTickV3<HypergraphStore>,
) -> MaterialReplaySessionV3<HypergraphStore> {
    let stored = preset
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let mut restored = reconstruct_material_foundation_v2(
        stored_copy(&stored),
        persisted_graph_copy(stored.graph_foundation()),
        preset
            .admitted(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap()
    .into_session()
    .unwrap();
    let graph = candidate.graph_report();
    restored
        .restore_full_checkpoint(
            graph.result_stable_graph(),
            graph.material_state_rows(),
            graph.result_registers().canonical_bytes(),
            candidate.material().register().canonical_bytes(),
        )
        .unwrap();
    restored
}

fn assert_workforce_seed_evidence(
    candidate: &babylon_tick::material_replay::PreparedMaterialTickV3<HypergraphStore>,
) {
    use babylon_bsl::identity_codec::StableBslValueV1;
    use babylon_graph::stable_element::StableElementKeyV1;
    let catalog = crate::test_support::catalog();
    let events = candidate.graph_report().successful_event_batch().events();
    assert_eq!(events.len(), 5);
    for seed in &catalog.staffing().pools {
        let event = events.iter().find(|event| event.fields().iter().any(|(key,value)| {
            key == "subject" && matches!(value, StableBslValueV1::Node(StableElementKeyV1::Node{local_name,..}) if *local_name == seed.local_name())
        })).unwrap();
        for (field, value) in [
            ("opening-employed", seed.employed),
            ("opening-reserve", seed.reserve),
            ("previous-unretained-hours", seed.previous_unretained_hours),
        ] {
            assert!(event.fields().contains(&(
                field.to_owned(),
                StableBslValueV1::Int(i64::try_from(value).unwrap())
            )));
        }
    }
}
