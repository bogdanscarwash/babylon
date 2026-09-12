//! Wire-level client lifecycle regressions using the real receive/dispatch systems.

use super::tests::{quit_app, snapshot_with_event};
use super::*;
use babylon_persistence::{identity::CampaignId, runtime_session::RuntimeSessionErrorCode};

type Replies = mpsc::Sender<Result<RuntimeSessionResponse, String>>;

fn campaign(number: u128) -> CampaignId {
    CampaignId::from_uuid(uuid::Uuid::from_u128(number))
}

fn open(number: u128) -> RuntimeSessionTarget {
    RuntimeSessionTarget::Open {
        campaign_id: campaign(number).as_uuid().to_string(),
    }
}

fn enqueue(app: &mut App, target: RuntimeSessionTarget) {
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .queue_campaign(target)
        .unwrap();
    app.update();
}

struct Switch {
    request_id: u64,
    previous: RuntimeSessionScope,
    scope: RuntimeSessionScope,
}

fn take_switch(requests: &mpsc::Receiver<RuntimeSessionRequest>) -> Switch {
    let RuntimeSessionRequest::Switch {
        request_id,
        scope: previous,
        target,
        ..
    } = requests.try_recv().unwrap()
    else {
        panic!("expected one campaign switch");
    };
    let (RuntimeSessionTarget::New { campaign_id, .. }
    | RuntimeSessionTarget::Open { campaign_id }) = target;
    Switch {
        request_id,
        scope: RuntimeSessionScope {
            epoch: previous.epoch + 1,
            campaign_id: Some(campaign_id),
        },
        previous,
    }
}

fn switching(app: &mut App, responses: &Replies, switch: &Switch) {
    responses
        .send(Ok(RuntimeSessionResponse::Switching {
            request_id: switch.request_id,
            previous_scope: switch.previous.clone(),
            scope: switch.scope.clone(),
        }))
        .unwrap();
    app.update();
}

fn admitted(app: &mut App, responses: &Replies, switch: &Switch, period: u64) {
    responses
        .send(Ok(RuntimeSessionResponse::Ready {
            request_id: switch.request_id,
            scope: switch.scope.clone(),
            foundation_digest: "foundation".into(),
            tail: RuntimeSessionTail {
                resolve_tick: period,
                tick_content_hash: None,
            },
        }))
        .unwrap();
    app.update();
}

fn refused(app: &mut App, responses: &Replies, switch: &Switch) {
    responses
        .send(Ok(RuntimeSessionResponse::Error {
            request_id: Some(switch.request_id),
            scope: switch.scope.clone(),
            code: RuntimeSessionErrorCode::CampaignAbsent,
            tail: None,
        }))
        .unwrap();
    app.update();
}

fn command(app: &mut App, command: ObserverCommand) {
    app.world_mut()
        .resource_mut::<Messages<ObserverCommand>>()
        .write(command);
    app.update();
}

fn initial() -> (App, mpsc::Receiver<RuntimeSessionRequest>, Replies) {
    let (mut app, requests, responses) = quit_app();
    app.insert_resource(ObserverSession::with_initial_target(open(1)).unwrap())
        .insert_resource(DossierCampaignId(campaign(1)))
        .init_resource::<ActiveCountyDossier>()
        .init_resource::<DossierFetchState>()
        .init_resource::<DossierPageView>()
        .init_resource::<PendingObservation>();
    (app, requests, responses)
}

fn hello(app: &mut App, responses: &Replies) {
    responses
        .send(Ok(RuntimeSessionResponse::Hello {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            scope: RuntimeSessionScope::default(),
        }))
        .unwrap();
    app.update();
}

#[test]
fn lifecycle_delayed_hello_and_ready_keep_window_and_save_only_admitted_target() {
    let (mut app, requests, responses) = initial();
    let directory =
        std::env::temp_dir().join(format!("babylon-lifecycle-{}", uuid::Uuid::new_v4()));
    let path = directory.join("observer-campaign");
    app.insert_resource(ContinuationPreference(path.clone()));
    let window = app.world_mut().spawn(Window::default()).id();
    app.world_mut()
        .resource_mut::<ObserverUiState>()
        .reduced_motion = true;
    app.world_mut()
        .resource_mut::<ObserverSession>()
        .perspective = Perspective::PlayerKnowledge;
    for _ in 0..3 {
        app.update();
    }
    assert!(requests.try_recv().is_err());
    enqueue(&mut app, open(2)); // Replaces the unsent initial destination.
    hello(&mut app, &responses);
    let switch = take_switch(&requests);
    assert_eq!(switch.request_id, 1);
    assert_eq!(
        switch.scope.campaign_id,
        Some(campaign(2).as_uuid().to_string())
    );
    app.update();
    assert!(requests.try_recv().is_err());
    assert!(!path.exists());
    switching(&mut app, &responses, &switch);
    assert_eq!(app.world().resource::<DossierCampaignId>().0, campaign(2));
    assert_eq!(
        app.world().resource::<ObserverSession>().phase,
        SessionPhase::Connecting
    );
    assert!(!path.exists());
    admitted(&mut app, &responses, &switch, 0);
    let saved = std::fs::read_to_string(&path).unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert_eq!(saved, format!("{}\n", campaign(2).as_uuid()));
    assert!(app.world().get::<Window>(window).is_some());
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    assert!(app.world().resource::<ObserverUiState>().reduced_motion);
    assert_eq!(
        app.world().resource::<ObserverSession>().perspective,
        Perspective::PlayerKnowledge
    );
    assert_eq!(
        app.world().resource::<ObserverSession>().phase,
        SessionPhase::Loading
    );
}

#[test]
fn lifecycle_quit_before_hello_cancels_initial_switch_and_stops_at_epoch_zero() {
    let (mut app, requests, responses) = initial();
    command(&mut app, ObserverCommand::Quit);
    assert!(requests.try_recv().is_err());
    hello(&mut app, &responses);
    let RuntimeSessionRequest::Stop {
        request_id, scope, ..
    } = requests.try_recv().unwrap()
    else {
        panic!("Quit must replace initial Switch");
    };
    assert_eq!(request_id, 1);
    assert_eq!(scope, RuntimeSessionScope::default());
    responses
        .send(Ok(RuntimeSessionResponse::Stopped { request_id, scope }))
        .unwrap();
    app.update();
    assert!(requests.try_recv().is_err());
    assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
}

#[test]
fn lifecycle_quit_during_admission_retains_latest_scope_and_discards_queued_target() {
    for fail in [false, true] {
        let (mut app, requests, responses) = initial();
        hello(&mut app, &responses);
        let switch = take_switch(&requests);
        switching(&mut app, &responses, &switch);
        enqueue(&mut app, open(2));
        command(&mut app, ObserverCommand::Quit);
        assert!(requests.try_recv().is_err());
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        if fail {
            refused(&mut app, &responses, &switch);
        } else {
            admitted(&mut app, &responses, &switch, 0);
        }
        let RuntimeSessionRequest::Stop {
            request_id, scope, ..
        } = requests.try_recv().unwrap()
        else {
            panic!("Quit must settle the latest accepted scope");
        };
        assert_eq!(request_id, switch.request_id + 1);
        assert_eq!(scope, switch.scope);
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        responses
            .send(Ok(RuntimeSessionResponse::Stopped { request_id, scope }))
            .unwrap();
        app.update();
        assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
        assert!(requests.try_recv().is_err());
    }
}

#[test]
fn lifecycle_switch_waits_for_commit_ack_then_clears_scoped_observations() {
    let (mut app, requests, responses) = quit_app();
    let state = app.world().resource::<ObserverSession>();
    let original = state.context();
    let snapshot = snapshot_with_event(state, "production", 3);
    app.insert_resource(ObserverFrame(Some(snapshot)))
        .insert_resource(DossierCampaignId(original.campaign))
        .init_resource::<ActiveCountyDossier>()
        .init_resource::<DossierFetchState>();
    command(&mut app, ObserverCommand::Step);
    let RuntimeSessionRequest::Advance {
        request_id, scope, ..
    } = requests.try_recv().unwrap()
    else {
        panic!("one period advance");
    };
    enqueue(&mut app, open(2));
    assert!(requests.try_recv().is_err());
    assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
    responses
        .send(Ok(RuntimeSessionResponse::Committed {
            request_id,
            scope,
            tail: RuntimeSessionTail {
                resolve_tick: 4,
                tick_content_hash: None,
            },
        }))
        .unwrap();
    app.update();
    let switch = take_switch(&requests);
    assert_eq!(switch.request_id, request_id + 1);
    assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 4);
    switching(&mut app, &responses, &switch);
    assert!(app.world().resource::<ObserverFrame>().0.is_none());
    assert!(app.world().resource::<ActiveCountyDossier>().0.is_none());
    assert!(matches!(
        app.world().resource::<DossierFetchState>(),
        DossierFetchState::WaitingForObservation
    ));
    assert_eq!(app.world().resource::<DossierCampaignId>().0, campaign(2));
    assert!(!app.world().resource::<ObserverSession>().accepts(&original));
    assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 0);
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn lifecycle_failed_b_then_return_to_a_rejects_old_epoch_and_async_a_results() {
    for failed_b in [false, true] {
        return_to_a_rejects_stale_results(failed_b);
    }
}

fn return_to_a_rejects_stale_results(failed_b: bool) {
    let (mut app, requests, responses) = quit_app();
    let state = app.world().resource::<ObserverSession>();
    let old_scope = state.runtime_scope().unwrap().clone();
    let old_context = state.context();
    let old_snapshot = snapshot_with_event(state, "production", 3);
    enqueue(&mut app, open(2));
    let b = take_switch(&requests);
    switching(&mut app, &responses, &b);
    if failed_b {
        refused(&mut app, &responses, &b);
    } else {
        admitted(&mut app, &responses, &b, 3);
    }
    assert_eq!(
        app.world().resource::<ObserverSession>().campaign,
        campaign(2)
    );
    assert_eq!(
        app.world().resource::<ObserverSession>().phase,
        if failed_b {
            SessionPhase::Failed
        } else {
            SessionPhase::Loading
        }
    );
    assert!(app.world().resource::<ObserverFrame>().0.is_none());
    enqueue(&mut app, open(1));
    let a = take_switch(&requests);
    switching(&mut app, &responses, &a);
    admitted(&mut app, &responses, &a, 3);
    let current = app.world().resource::<ObserverSession>().context();
    let refresh = app.world().resource::<DossierRefresh>().0;
    assert_eq!(current.campaign, old_context.campaign);
    assert_eq!(current.tick, old_context.tick);
    assert!(current.generation > old_context.generation);
    for stale in [
        RuntimeSessionResponse::Ready {
            request_id: 0,
            scope: old_scope.clone(),
            foundation_digest: "stale".into(),
            tail: RuntimeSessionTail {
                resolve_tick: 99,
                tick_content_hash: None,
            },
        },
        RuntimeSessionResponse::Error {
            request_id: None,
            scope: old_scope.clone(),
            code: RuntimeSessionErrorCode::StorageRefused,
            tail: None,
        },
        RuntimeSessionResponse::ArchiveProgress {
            request_id: None,
            scope: old_scope,
            durable_tick: 3,
            verified_tick: 3,
        },
    ] {
        responses.send(Ok(stale)).unwrap();
    }
    app.update();
    assert_eq!(app.world().resource::<ObserverSession>().context(), current);
    assert_eq!(
        app.world().resource::<ObserverSession>().phase,
        SessionPhase::Loading
    );
    assert_eq!(app.world().resource::<DossierRefresh>().0, refresh);
    let mut frame = ObserverFrame::default();
    install_observation(
        &mut app.world_mut().resource_mut::<ObserverSession>(),
        &old_context,
        old_snapshot,
        &mut frame,
        false,
    );
    assert!(frame.0.is_none());
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn lifecycle_future_scope_and_wrong_switch_correlation_refuse_without_rebinding() {
    for wrong_id in [false, true] {
        let (mut app, requests, responses) = initial();
        hello(&mut app, &responses);
        let mut switch = take_switch(&requests);
        if wrong_id {
            switch.request_id += 1;
        } else {
            switch.scope.epoch += 1;
        }
        let original = app.world().resource::<ObserverSession>().context();
        switching(&mut app, &responses, &switch);
        assert_eq!(
            app.world().resource::<ObserverSession>().context(),
            original
        );
        assert_eq!(
            app.world().resource::<ObserverSession>().phase,
            SessionPhase::Failed
        );
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
        assert!(requests.try_recv().is_err());
    }
}

#[test]
fn lifecycle_pending_destination_is_bounded_and_never_resends_an_accepted_switch() {
    let (mut app, requests, responses) = initial();
    hello(&mut app, &responses);
    let first = take_switch(&requests);
    switching(&mut app, &responses, &first);
    enqueue(&mut app, open(2));
    enqueue(&mut app, open(3));
    for _ in 0..3 {
        app.update();
    }
    assert!(requests.try_recv().is_err());
    admitted(&mut app, &responses, &first, 0);
    let next = take_switch(&requests);
    assert_eq!(next.request_id, first.request_id + 1);
    assert_eq!(next.previous, first.scope);
    assert_eq!(
        next.scope.campaign_id,
        Some(campaign(3).as_uuid().to_string())
    );
    app.update();
    assert!(requests.try_recv().is_err());
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn lifecycle_request_ids_remain_unique_across_switch_advance_and_stop() {
    let (mut app, requests, responses) = initial();
    hello(&mut app, &responses);
    let switch = take_switch(&requests);
    switching(&mut app, &responses, &switch);
    admitted(&mut app, &responses, &switch, 0);
    {
        let mut state = app.world_mut().resource_mut::<ObserverSession>();
        let context = state.context();
        assert!(state.installed(&context));
    }
    command(&mut app, ObserverCommand::Step);
    let RuntimeSessionRequest::Advance {
        request_id,
        scope,
        expected_tail,
        ..
    } = requests.try_recv().unwrap()
    else {
        panic!("period advance");
    };
    assert_eq!(request_id, switch.request_id + 1);
    assert_eq!(scope, switch.scope);
    assert_eq!(expected_tail.resolve_tick, 0);
    responses
        .send(Ok(RuntimeSessionResponse::Committed {
            request_id,
            scope: scope.clone(),
            tail: RuntimeSessionTail {
                resolve_tick: 1,
                tick_content_hash: Some("1".repeat(64)),
            },
        }))
        .unwrap();
    app.update();
    command(&mut app, ObserverCommand::Quit);
    let RuntimeSessionRequest::Stop {
        request_id: stop_id,
        scope: stop_scope,
        ..
    } = requests.try_recv().unwrap()
    else {
        panic!("terminal Stop");
    };
    assert_eq!(stop_id, request_id + 1);
    assert_eq!(stop_scope, scope);
    responses
        .send(Ok(RuntimeSessionResponse::Stopped {
            request_id: stop_id,
            scope: stop_scope,
        }))
        .unwrap();
    app.update();
    assert_eq!(app.world().resource::<Messages<AppExit>>().len(), 1);
}

#[test]
fn lifecycle_pipe_death_retains_sent_new_identity_without_resend_or_fact_adoption() {
    let (mut app, requests, responses) = quit_app();
    let original = app.world().resource::<ObserverSession>().context();
    let target = campaign(99).as_uuid().to_string();
    enqueue(
        &mut app,
        RuntimeSessionTarget::New {
            campaign_id: target.clone(),
            preset: RuntimeSessionPreset::Standard,
        },
    );
    let switch = take_switch(&requests);
    assert_eq!(switch.scope.campaign_id.as_deref(), Some(target.as_str()));
    responses
        .send(Err("Runtime pipe closed before Switching".into()))
        .unwrap();
    app.update();
    let state = app.world().resource::<ObserverSession>();
    assert!(state.runtime_disconnected());
    assert_eq!(state.uncertain_campaign_id(), Some(target.as_str()));
    assert_eq!(state.context(), original);
    assert_eq!(state.runtime_scope(), Some(&switch.previous));
    assert!(state.error.as_ref().unwrap().contains(&target));
    assert!(crate::observer_controls::turn_presentation(state)
        .status
        .contains(&target));
    assert!(app.world().resource::<ObserverFrame>().0.is_none());
    for _ in 0..3 {
        app.update();
    }
    assert!(requests.try_recv().is_err());
    assert!(app.world().resource::<Messages<AppExit>>().is_empty());
}

#[test]
fn lifecycle_admission_refusals_explain_the_available_campaign_choice() {
    for (code, expected) in [
        (
            RuntimeSessionErrorCode::CampaignAbsent,
            "The selected campaign was not found. Choose another campaign or create New.",
        ),
        (
            RuntimeSessionErrorCode::CampaignAlreadyExists,
            "That campaign already exists. Choose Open to continue it.",
        ),
    ] {
        let (mut app, requests, responses) = initial();
        hello(&mut app, &responses);
        let switch = take_switch(&requests);
        switching(&mut app, &responses, &switch);
        responses
            .send(Ok(RuntimeSessionResponse::Error {
                request_id: Some(switch.request_id),
                scope: switch.scope,
                code,
                tail: None,
            }))
            .unwrap();
        app.update();
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.error.as_deref(), Some(expected));
        assert_eq!(
            crate::observer_controls::turn_presentation(state).status,
            expected
        );
        assert!(!state.runtime_disconnected());
        enqueue(&mut app, open(2));
        assert_eq!(take_switch(&requests).request_id, switch.request_id + 1);
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    }
}
