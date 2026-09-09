use babylon_client::observer::{ObservationContext, ObserverSession, Perspective, SessionPhase};
use babylon_persistence::CampaignId;
use uuid::Uuid;

fn session() -> ObserverSession {
    ObserverSession::new(CampaignId::from_uuid(Uuid::from_u128(17)))
}

#[test]
fn pause_finishes_one_outstanding_commit_and_never_queues_a_second() {
    let mut state = session();
    state.ready(0, None);
    state.installed(&state.context());
    state.playing = true;
    let request = state.begin_advance().expect("one request");
    assert!(state.begin_advance().is_none());
    state.playing = false;
    assert!(state.acknowledge(request, 1, Some("committed".into())));
    assert_eq!(state.durable_tick, 1);
    assert_eq!(state.phase, SessionPhase::Loading);
    assert!(state.begin_advance().is_none(), "render must install first");
    assert!(!state.playing);
}

#[test]
fn perspective_and_history_changes_invalidate_inflight_observations() {
    let mut state = session();
    state.ready(4, Some("head".into()));
    let old = state.context();
    state.set_perspective(Perspective::PlayerKnowledge);
    assert!(!state.accepts(&old));
    let current = state.context();
    assert!(state.accepts(&current));
    state.inspect_tick(2);
    assert!(!state.accepts(&current));
    assert_eq!(state.viewed_tick, 2);
    assert!(!state.playing);
    assert!(state.begin_advance().is_none());
    state.return_live();
    assert_eq!(state.viewed_tick, 4);
}

#[test]
fn foreign_campaign_and_out_of_order_acknowledgements_do_not_install() {
    let mut state = session();
    state.ready(2, Some("head".into()));
    state.installed(&state.context());
    let request = state.begin_advance().unwrap();
    assert!(!state.acknowledge(request + 1, 3, Some("wrong".into())));
    assert_eq!(state.durable_tick, 2);
    assert!(!state.acknowledge(request, 4, Some("skipped".into())));
    let foreign = ObservationContext {
        campaign: CampaignId::from_uuid(Uuid::from_u128(18)),
        ..state.context()
    };
    assert!(!state.accepts(&foreign));
}

#[test]
fn completed_scenario_can_inspect_history_without_reopening_transport() {
    let mut state = session();
    state.ready(16, Some("complete".into()));
    state.horizon_tick = Some(16);
    state.installed(&state.context());
    assert_eq!(state.phase, SessionPhase::Complete);
    assert!(state.begin_advance().is_none());
    state.inspect_tick(4);
    state.installed(&state.context());
    assert_eq!(state.phase, SessionPhase::Ready);
    assert!(
        state.begin_advance().is_none(),
        "historical state never advances"
    );
    state.return_live();
    state.installed(&state.context());
    assert_eq!(state.phase, SessionPhase::Complete);
}

#[test]
fn perspective_switch_while_advancing_does_not_allow_a_second_request() {
    let mut state = session();
    state.ready(0, None);
    state.installed(&state.context());
    let request = state.begin_advance().unwrap();
    state.set_perspective(Perspective::PlayerKnowledge);
    state.installed(&state.context());
    assert!(state.begin_advance().is_none());
    assert!(state.acknowledge(request, 1, Some("committed".into())));
    assert_eq!(state.perspective, Perspective::PlayerKnowledge);
    assert!(!state.playing);
}
