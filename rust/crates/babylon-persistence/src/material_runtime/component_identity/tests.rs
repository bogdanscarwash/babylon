use super::*;

fn identity() -> MaterialComponentIdentity {
    MaterialComponentIdentity::from_parts(
        b"resolver",
        b"environment",
        &ReplaySessionId::try_from("fixture/component-identity").unwrap(),
        ReplaySeed::new(7),
        &ContentDigest {
            defines_hash: [3; 32],
            rules_hash: [4; 32],
        },
        RefDigest::from_bytes([5; 32]),
    )
}

#[test]
fn every_immutable_checkpoint_section_is_exact_and_mandatory() {
    let expected = identity();
    let mut sections = vec![Vec::new(); 9];
    sections[2..8].clone_from_slice(&expected.sections);
    expected.validate_sections(&sections).unwrap();
    for tag in 2..8 {
        let mut altered = sections.clone();
        altered[tag][0] ^= 1;
        assert!(matches!(
            expected.validate_sections(&altered),
            Err(MaterialRuntimeError::InvalidCheckpoint)
        ));
    }
    sections.pop();
    assert!(matches!(
        expected.validate_sections(&sections),
        Err(MaterialRuntimeError::InvalidCheckpoint)
    ));
}

#[test]
fn empty_actions_bind_exact_session_tick_layout_digest_and_bytes() {
    let expected = identity();
    let actions = OrderedPracticeActionBatch::empty(expected.session_id.clone(), 3).unwrap();
    expected
        .validate_actions(3, 1, actions.digest().as_bytes(), actions.canonical_bytes())
        .unwrap();
    let foreign = OrderedPracticeActionBatch::empty(
        ReplaySessionId::try_from("fixture/foreign-session").unwrap(),
        3,
    )
    .unwrap();
    for (tick, layout, digest, bytes) in [
        (
            4,
            1,
            actions.digest().as_bytes().as_slice(),
            actions.canonical_bytes(),
        ),
        (
            3,
            2,
            actions.digest().as_bytes().as_slice(),
            actions.canonical_bytes(),
        ),
        (
            3,
            1,
            foreign.digest().as_bytes().as_slice(),
            foreign.canonical_bytes(),
        ),
        (
            3,
            1,
            foreign.digest().as_bytes().as_slice(),
            actions.canonical_bytes(),
        ),
        (
            3,
            1,
            actions.digest().as_bytes().as_slice(),
            foreign.canonical_bytes(),
        ),
    ] {
        assert!(matches!(
            expected.validate_actions(tick, layout, digest, bytes),
            Err(MaterialRuntimeError::InvalidCheckpoint)
        ));
    }
}

#[test]
fn captured_foundation_and_live_session_have_identical_component_admission() {
    let foundation = crate::michigan_content::MichiganContentPreset::FourWeekStandard
        .create_foundation(&crate::test_support::catalog())
        .unwrap();
    let retained = MaterialComponentIdentity::from_foundation(foundation.graph_foundation());
    let session = foundation.into_session().unwrap();
    assert_eq!(
        retained,
        MaterialComponentIdentity::from_session(session.graph_session())
    );
}
