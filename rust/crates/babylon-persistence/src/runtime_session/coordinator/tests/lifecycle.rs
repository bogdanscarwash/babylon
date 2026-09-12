use super::*;
use std::cell::RefCell;
use std::collections::VecDeque;

fn stop_at(scope: RuntimeSessionScope) -> RuntimeSessionRequest {
    RuntimeSessionRequest::Stop {
        protocol_version: 3,
        request_id: 99,
        scope,
    }
}

fn lifecycle(
    requests: &[RuntimeSessionRequest],
    admissions: Vec<Result<Backend, RuntimeSessionErrorCode>>,
) -> Vec<RuntimeSessionResponse> {
    let mut admissions: VecDeque<_> = admissions.into();
    let current: RefCell<Option<Arc<DriverState>>> = RefCell::new(None);
    let mut output = Vec::new();
    serve(
        Cursor::new(wire(requests)),
        &mut output,
        |_| {
            if let Some(previous) = current.borrow_mut().take() {
                assert!(previous.stopped.load(Ordering::SeqCst));
                assert!(previous.joined.load(Ordering::SeqCst));
            }
            let backend = admissions.pop_front().expect("exact admission count")?;
            *current.borrow_mut() = Some(Arc::clone(&backend.state));
            Ok((backend, "digest".into()))
        },
        |_, sink| {
            let current = current.borrow();
            let state = current.as_ref().unwrap();
            *state.sink.lock().unwrap() = Some(sink);
            Ok(driver(state))
        },
    )
    .unwrap();
    assert!(admissions.is_empty());
    if let Some(state) = current.into_inner() {
        assert!(state.joined.load(Ordering::SeqCst));
    }
    wire_responses(&output)
}

#[test]
fn hello_and_initial_stop_need_no_campaign_or_database_admission() {
    let rows = lifecycle(&[stop_at(RuntimeSessionScope::default())], vec![]);
    assert_eq!(
        rows,
        [
            RuntimeSessionResponse::Hello {
                protocol_version: 3,
                scope: RuntimeSessionScope::default(),
            },
            RuntimeSessionResponse::Stopped {
                request_id: 99,
                scope: RuntimeSessionScope::default(),
            },
        ]
    );
}

#[derive(Clone, Default)]
struct SharedOutput(Arc<Mutex<Vec<u8>>>);
impl Write for SharedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn hello_and_switching_are_written_before_fallible_target_admission() {
    let mut output = SharedOutput::default();
    let observed = output.clone();
    serve(
        Cursor::new(wire(&[
            switching(RuntimeSessionScope::default(), A, 1),
            stop_at(scope(1, A)),
        ])),
        &mut output,
        |_| -> Result<(Backend, String), RuntimeSessionErrorCode> {
            let bytes = observed.0.lock().unwrap();
            assert!(matches!(
                wire_responses(&bytes).as_slice(),
                [
                    RuntimeSessionResponse::Hello {
                        protocol_version: 3,
                        ..
                    },
                    RuntimeSessionResponse::Switching { request_id: 1, .. },
                ]
            ));
            Err(RuntimeSessionErrorCode::StorageRefused)
        },
        |_, _| -> Result<Driver, RuntimeSessionErrorCode> {
            panic!("failed admission cannot start an Archive driver")
        },
    )
    .unwrap();
    assert!(matches!(
        wire_responses(&output.0.lock().unwrap()).as_slice(),
        [
            RuntimeSessionResponse::Hello { .. },
            RuntimeSessionResponse::Switching { .. },
            RuntimeSessionResponse::Error {
                code: RuntimeSessionErrorCode::StorageRefused,
                tail: None,
                ..
            },
            RuntimeSessionResponse::Stopped { .. },
        ]
    ));
}

#[test]
fn missing_initial_open_remains_recoverable_and_failed_target_has_no_old_tail() {
    let rows = lifecycle(
        &[
            switching(RuntimeSessionScope::default(), A, 1),
            switching(scope(1, A), B, 2),
            switching(scope(2, B), A, 3),
            switching(scope(3, A), B, 4),
            stop_at(scope(4, B)),
        ],
        vec![
            Err(RuntimeSessionErrorCode::CampaignAbsent),
            Ok(backend()),
            Err(RuntimeSessionErrorCode::StorageRefused),
            Ok(backend()),
        ],
    );
    let errors: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Error {
                scope, code, tail, ..
            } => Some((scope, code, tail)),
            _ => None,
        })
        .collect();
    assert_eq!(
        errors,
        [
            (
                &scope(1, A),
                &RuntimeSessionErrorCode::CampaignAbsent,
                &None
            ),
            (
                &scope(3, A),
                &RuntimeSessionErrorCode::StorageRefused,
                &None
            ),
        ]
    );
    let ready: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Ready { scope, .. } => Some(scope.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ready, [scope(2, B), scope(4, B)]);
}

#[test]
fn repeated_campaign_epochs_retire_each_driver_and_refuse_old_commands() {
    let initial = backend();
    let initial_state = Arc::clone(&initial.state);
    let rows = lifecycle(
        &[
            switching(RuntimeSessionScope::default(), A, 1),
            advance(),
            switching(scope(1, A), B, 3),
            advance(),
            switching(scope(2, B), A, 4),
            advance(),
            switching(scope(1, A), B, 5),
            stop(),
            stop_at(scope(3, A)),
        ],
        vec![Ok(initial), Ok(backend()), Ok(backend())],
    );
    let switches: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Switching {
                previous_scope,
                scope,
                ..
            } => Some((previous_scope.clone(), scope.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        switches,
        [
            (RuntimeSessionScope::default(), scope(1, A)),
            (scope(1, A), scope(2, B)),
            (scope(2, B), scope(3, A)),
        ]
    );
    assert_eq!(initial_state.tick.load(Ordering::SeqCst), 1);
    let committed: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Committed { scope, tail, .. } => {
                Some((scope, tail.resolve_tick))
            }
            _ => None,
        })
        .collect();
    assert_eq!(committed, [(&scope(1, A), 1)]);
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(
                row,
                RuntimeSessionResponse::Error {
                    code: RuntimeSessionErrorCode::SessionMismatch,
                    ..
                }
            ))
            .count(),
        4
    );
    let ack = rows
        .iter()
        .position(|row| matches!(row, RuntimeSessionResponse::Committed { .. }))
        .unwrap();
    let next = rows
        .iter()
        .position(|row| matches!(row, RuntimeSessionResponse::Switching { request_id: 3, .. }))
        .unwrap();
    assert!(ack < next);
    assert!(
        matches!(rows.last().unwrap(), RuntimeSessionResponse::Stopped { scope: actual, .. } if *actual == scope(3, A))
    );
}

#[test]
fn late_archive_events_cannot_enter_a_reopened_campaign_epoch() {
    let mut backend = backend();
    let archive = driver(&backend.state);
    let mut output = Vec::new();
    {
        let mut coordinator = active_coordinator(&mut output, &mut backend, archive);
        coordinator.scope = scope(3, A);
        for old in [scope(1, A), scope(2, B)] {
            coordinator
                .archive_event(&old, &progress(Some(8), 0, 0))
                .unwrap();
            coordinator
                .archive_event(&old, &ArchiveDriverEvent::Stopped)
                .unwrap();
            coordinator
                .archive_event(
                    &old,
                    &ArchiveDriverEvent::Failure {
                        request_id: None,
                        failure: crate::archive_driver::ArchiveDriverFailure::Refused(
                            crate::SemanticArchiveError::StoredPageMismatch,
                        ),
                        retrying: false,
                    },
                )
                .unwrap();
        }
        coordinator
            .archive_event(&scope(3, A), &progress(None, 0, 0))
            .unwrap();
    }
    assert!(matches!(wire_responses(&output).as_slice(), [
        RuntimeSessionResponse::ArchiveProgress { scope: actual, .. }
    ] if *actual == scope(3, A)));
}

#[test]
fn invalid_targets_and_exhausted_epochs_refuse_without_admission() {
    for (current, target) in [
        (
            RuntimeSessionScope::default(),
            uuid::Uuid::nil().to_string(),
        ),
        (RuntimeSessionScope::default(), "invalid".into()),
        (
            RuntimeSessionScope::default(),
            "00000000-0000-0000-0000-00000000000A".into(),
        ),
        (scope(u64::MAX, A), B.into()),
    ] {
        let mut output = Vec::new();
        let (sender, receiver) = mpsc::sync_channel(1);
        {
            let mut coordinator: Coordinator<'_, _, Backend, Driver> =
                Coordinator::new(&mut output);
            coordinator.scope = current.clone();
            let mut factories = Factories {
                backend: |_: &RuntimeSessionTarget| -> Result<(Backend, String), RuntimeSessionErrorCode> { panic!("invalid switch reached admission") },
                archive: |_, _: ArchiveEventSink| -> Result<Driver, RuntimeSessionErrorCode> { panic!("invalid switch started worker") },
            };
            coordinator
                .request(
                    &wire(&[switching(current.clone(), &target, 1)]),
                    &receiver,
                    &sender,
                    &mut factories,
                )
                .unwrap();
            assert_eq!(coordinator.scope, current);
            assert!(coordinator.active.is_none());
        }
        assert!(matches!(wire_responses(&output).as_slice(), [
            RuntimeSessionResponse::Error { code: RuntimeSessionErrorCode::InvalidRequest, tail: None, scope, .. }
        ] if *scope == current));
    }
}

#[test]
fn wire_targets_are_explicit_and_closed() {
    for target in [
        RuntimeSessionTarget::New {
            campaign_id: A.into(),
            preset: super::super::super::RuntimeSessionPreset::Delayed,
        },
        RuntimeSessionTarget::Open {
            campaign_id: A.into(),
        },
    ] {
        let bytes = serde_json::to_vec(&target).unwrap();
        assert_eq!(
            serde_json::from_slice::<RuntimeSessionTarget>(&bytes).unwrap(),
            target
        );
        assert_eq!(target.campaign().unwrap().as_uuid().to_string(), A);
    }
    for json in [
        r#"{"type":"new","campaign_id":"00000000-0000-0000-0000-000000000001"}"#,
        r#"{"type":"new","campaign_id":"00000000-0000-0000-0000-000000000001","preset":"invented"}"#,
        r#"{"type":"open","campaign_id":"00000000-0000-0000-0000-000000000001","preset":"standard"}"#,
    ] {
        assert!(serde_json::from_str::<RuntimeSessionTarget>(json).is_err());
    }
    let mut request = serde_json::to_value(advance()).unwrap();
    request
        .as_object_mut()
        .unwrap()
        .insert("actions".into(), serde_json::json!([1]));
    assert!(serde_json::from_value::<RuntimeSessionRequest>(request).is_err());
}

#[test]
fn zero_request_ids_cannot_switch_or_stop_before_initial_admission() {
    let mut zero_stop = stop_at(RuntimeSessionScope::default());
    if let RuntimeSessionRequest::Stop { request_id, .. } = &mut zero_stop {
        *request_id = 0;
    }
    let rows = lifecycle(
        &[
            switching(RuntimeSessionScope::default(), A, 0),
            zero_stop,
            switching(RuntimeSessionScope::default(), A, 1),
            stop_at(scope(1, A)),
        ],
        vec![Ok(backend())],
    );
    assert!(matches!(rows.as_slice(), [
        RuntimeSessionResponse::Hello { .. },
        RuntimeSessionResponse::Error {
            request_id: Some(0), code: RuntimeSessionErrorCode::InvalidRequest,
            scope: refused, tail: None,
        },
        RuntimeSessionResponse::Error {
            request_id: Some(0), code: RuntimeSessionErrorCode::InvalidRequest,
            scope: stop_scope, tail: None,
        },
        RuntimeSessionResponse::Switching { request_id: 1, .. },
        RuntimeSessionResponse::Ready { request_id: 1, .. },
        RuntimeSessionResponse::Stopped { request_id: 99, .. },
    ] if *refused == RuntimeSessionScope::default() && *stop_scope == RuntimeSessionScope::default()));
}

#[test]
fn request_ids_remain_consumed_after_failed_admission_and_across_campaign_epochs() {
    let rows = lifecycle(
        &[
            switching(RuntimeSessionScope::default(), A, 10),
            switching(scope(1, A), B, 10),
            switching(scope(1, A), B, 11),
            switching(scope(1, A), A, u64::MAX),
            switching(scope(2, B), A, 11),
            switching(scope(2, B), A, 12),
            stop_at(scope(3, A)),
        ],
        vec![
            Err(RuntimeSessionErrorCode::CampaignAbsent),
            Ok(backend()),
            Ok(backend()),
        ],
    );
    let errors: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Error {
                request_id,
                scope,
                code,
                tail,
            } => Some((*request_id, scope.clone(), *code, tail.is_some())),
            _ => None,
        })
        .collect();
    assert_eq!(
        errors,
        [
            (
                Some(10),
                scope(1, A),
                RuntimeSessionErrorCode::CampaignAbsent,
                false
            ),
            (
                Some(10),
                scope(1, A),
                RuntimeSessionErrorCode::InvalidRequest,
                false
            ),
            (
                Some(u64::MAX),
                scope(2, B),
                RuntimeSessionErrorCode::SessionMismatch,
                true
            ),
            (
                Some(11),
                scope(2, B),
                RuntimeSessionErrorCode::InvalidRequest,
                true
            ),
        ]
    );
    let accepted: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Switching {
                request_id, scope, ..
            } => Some((*request_id, scope.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        accepted,
        [(10, scope(1, A)), (11, scope(2, B)), (12, scope(3, A))]
    );
    assert!(
        matches!(rows.last(), Some(RuntimeSessionResponse::Stopped { request_id: 99, scope: final_scope }) if *final_scope == scope(3, A))
    );
}

#[test]
fn reused_or_lower_ids_cannot_advance_refresh_or_stop_a_current_scope() {
    let backend = backend();
    let state = Arc::clone(&backend.state);
    let advanced_tail = RuntimeSessionTail {
        resolve_tick: 1,
        tick_content_hash: Some(format!("{:064x}", 1)),
    };
    let rows = lifecycle(
        &[
            switching(RuntimeSessionScope::default(), A, 1),
            RuntimeSessionRequest::Advance {
                protocol_version: 3,
                scope: scope(1, A),
                request_id: 2,
                expected_tail: RuntimeSessionTail {
                    resolve_tick: 0,
                    tick_content_hash: None,
                },
            },
            RuntimeSessionRequest::Advance {
                protocol_version: 3,
                scope: scope(1, A),
                request_id: 2,
                expected_tail: advanced_tail.clone(),
            },
            RuntimeSessionRequest::RefreshArchive {
                protocol_version: 3,
                scope: scope(1, A),
                request_id: 1,
            },
            RuntimeSessionRequest::Stop {
                protocol_version: 3,
                scope: scope(1, A),
                request_id: 2,
            },
            stop_at(scope(1, A)),
        ],
        vec![Ok(backend)],
    );
    assert_eq!(state.tick.load(Ordering::SeqCst), 1);
    assert!(state.refreshes.lock().unwrap().is_empty());
    let errors: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            RuntimeSessionResponse::Error {
                request_id,
                scope,
                code,
                tail,
            } => Some((*request_id, scope.clone(), *code, tail.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(
        errors,
        [2, 1, 2].map(|id| (
            Some(id),
            scope(1, A),
            RuntimeSessionErrorCode::InvalidRequest,
            Some(advanced_tail.clone())
        ))
    );
    assert!(matches!(
        rows.last(),
        Some(RuntimeSessionResponse::Stopped { request_id: 99, .. })
    ));
}
