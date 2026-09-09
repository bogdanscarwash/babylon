use babylon_practice_contract::{
    activation_blockers, unwired_reason, PracticeActivationBlockerV1, PracticeIdV1,
    PracticeRejectionCodeV1,
};

const SHARED_BLOCKERS: &[PracticeActivationBlockerV1] = &[
    PracticeActivationBlockerV1::Gate3CommittedEnvelope,
    PracticeActivationBlockerV1::Gate5PendingInput,
];
const MUTUAL_AID_BLOCKERS: &[PracticeActivationBlockerV1] = &[
    PracticeActivationBlockerV1::Gate3CommittedEnvelope,
    PracticeActivationBlockerV1::Gate5PendingInput,
    PracticeActivationBlockerV1::Per30OrdersInventory,
    PracticeActivationBlockerV1::Per31FreightRealization,
];
const ORGANIZE_REASON: PracticeRejectionCodeV1 = unwired_reason(PracticeIdV1::Organize);
const ORGANIZE_BLOCKERS: &[PracticeActivationBlockerV1] =
    activation_blockers(PracticeIdV1::Organize);

#[test]
fn every_closed_practice_returns_the_unwired_reason() {
    for practice in [
        PracticeIdV1::Organize,
        PracticeIdV1::Agitate,
        PracticeIdV1::MutualAid,
    ] {
        assert_eq!(
            unwired_reason(practice),
            PracticeRejectionCodeV1::PracticeUnwired
        );
    }
}

#[test]
fn activation_blockers_are_exact_ordered_static_slices() {
    assert_eq!(ORGANIZE_REASON, PracticeRejectionCodeV1::PracticeUnwired);
    assert_eq!(ORGANIZE_BLOCKERS, SHARED_BLOCKERS);
    assert_eq!(activation_blockers(PracticeIdV1::Organize), SHARED_BLOCKERS);
    assert_eq!(activation_blockers(PracticeIdV1::Agitate), SHARED_BLOCKERS);
    assert_eq!(
        activation_blockers(PracticeIdV1::MutualAid),
        MUTUAL_AID_BLOCKERS
    );
}

#[test]
fn mutual_aid_blockers_do_not_invent_goods_or_universal_dependencies() {
    let names = format!("{:?}", activation_blockers(PracticeIdV1::MutualAid));
    for forbidden in [
        "ActionBudget",
        "Capacity",
        "wealth",
        "rent_pool",
        "money",
        "Per36",
        "Per44",
        "repression",
    ] {
        assert!(!names.contains(forbidden));
    }
}
