use super::*;
use crate::runtime_session::RuntimeSessionTailV2;
use std::cell::Cell;
use std::io::{self, Cursor, Read};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct DriverState {
    stopped: AtomicBool,
    exited: AtomicBool,
    joined: AtomicBool,
    tick: AtomicU64,
    refreshes: Mutex<Vec<u64>>,
    sink: Mutex<Option<ArchiveEventSink>>,
}

struct Driver {
    state: Arc<DriverState>,
    finishes_on_stop: bool,
    refresh_full: bool,
    join_fails: bool,
    finishing_checks: Cell<usize>,
}

impl ArchiveControl for Driver {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCodeV2> {
        if self.refresh_full {
            return Err(RuntimeSessionErrorCodeV2::StorageBusy);
        }
        self.state.refreshes.lock().unwrap().push(request_id);
        if let Some(sink) = self.state.sink.lock().unwrap().as_ref() {
            assert!(sink(progress(
                Some(request_id),
                self.state.tick.load(Ordering::SeqCst),
                0
            )));
        }
        Ok(())
    }
    fn stop(&self) {
        self.state.stopped.store(true, Ordering::SeqCst);
    }
    fn finished(&self) -> bool {
        if self.state.exited.load(Ordering::SeqCst) {
            return true;
        }
        if !self.finishes_on_stop || !self.state.stopped.load(Ordering::SeqCst) {
            return false;
        }
        let remaining = self.finishing_checks.get();
        self.finishing_checks.set(remaining.saturating_sub(1));
        remaining == 0
    }
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV2> {
        assert!(
            self.finished(),
            "never join an unfinished synchronous driver"
        );
        self.state.joined.store(true, Ordering::SeqCst);
        if self.join_fails {
            Err(RuntimeSessionErrorCodeV2::ArchiveRefused)
        } else {
            Ok(())
        }
    }
}

struct Backend {
    tick: u64,
    fail_commit: bool,
    notify_inside_advance: bool,
    state: Arc<DriverState>,
}

impl SessionBackend for Backend {
    fn tail(&self) -> RuntimeSessionTailV2 {
        RuntimeSessionTailV2 {
            resolve_tick: self.tick,
            tick_content_hash: (self.tick > 0).then(|| format!("{:064x}", self.tick)),
        }
    }
    fn advance(
        &mut self,
        expected: &RuntimeSessionTailV2,
    ) -> Result<RuntimeSessionTailV2, RuntimeSessionErrorCodeV2> {
        if expected != &self.tail() {
            return Err(RuntimeSessionErrorCodeV2::StaleExpectedTail);
        }
        if self.fail_commit {
            return Err(RuntimeSessionErrorCodeV2::CommitRefused);
        }
        self.tick += 1;
        self.state.tick.store(self.tick, Ordering::SeqCst);
        if self.notify_inside_advance {
            let guard = self.state.sink.lock().unwrap();
            assert!(guard.as_ref().unwrap()(progress(
                None, self.tick, self.tick
            )));
        }
        Ok(self.tail())
    }
}

fn backend() -> Backend {
    Backend {
        tick: 0,
        fail_commit: false,
        notify_inside_advance: false,
        state: Arc::default(),
    }
}

fn driver(state: &Arc<DriverState>) -> Driver {
    Driver {
        state: Arc::clone(state),
        finishes_on_stop: true,
        refresh_full: false,
        join_fails: false,
        finishing_checks: Cell::new(0),
    }
}

fn progress(
    request_id: Option<u64>,
    durable_tick: u64,
    verified_tick: u64,
) -> ArchiveDriverEventV1 {
    ArchiveDriverEventV1::Progress {
        request_id,
        durable_tick,
        verified_tick,
        retention_ready: true,
    }
}

fn advance() -> RuntimeSessionRequestV2 {
    RuntimeSessionRequestV2::Advance {
        protocol_version: 2,
        campaign_id: "campaign".into(),
        request_id: 1,
        expected_tail: RuntimeSessionTailV2 {
            resolve_tick: 0,
            tick_content_hash: None,
        },
    }
}

fn stop() -> RuntimeSessionRequestV2 {
    RuntimeSessionRequestV2::Stop {
        protocol_version: 2,
        campaign_id: "campaign".into(),
        request_id: 8,
    }
}

fn refresh() -> RuntimeSessionRequestV2 {
    RuntimeSessionRequestV2::RefreshArchive {
        protocol_version: 2,
        campaign_id: "campaign".into(),
        request_id: 31,
    }
}

fn wire(requests: &[RuntimeSessionRequestV2]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for request in requests {
        serde_json::to_writer(&mut bytes, request).unwrap();
        bytes.push(b'\n');
    }
    bytes
}

fn responses(output: &[u8]) -> Vec<RuntimeSessionResponseV2> {
    output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}

fn run(
    input: Vec<u8>,
    backend: &mut Backend,
    output: &mut impl Write,
) -> Result<(), RuntimeSessionErrorCodeV2> {
    let shared = Arc::clone(&backend.state);
    serve(
        Cursor::new(input),
        output,
        backend,
        "campaign",
        "digest".into(),
        move |sink| {
            *shared.sink.lock().unwrap() = Some(sink);
            Ok(driver(&shared))
        },
    )
}

#[test]
fn completion_queued_inside_advance_cannot_precede_committed_acknowledgement() {
    let mut backend = backend();
    backend.notify_inside_advance = true;
    let mut output = Vec::new();
    run(
        wire(&[advance(), stop(), advance()]),
        &mut backend,
        &mut output,
    )
    .unwrap();
    let rows = responses(&output);
    assert_eq!(rows.len(), 4);
    assert!(matches!(rows[0], RuntimeSessionResponseV2::Ready { .. }));
    assert!(matches!(
        rows[1],
        RuntimeSessionResponseV2::Committed { request_id: 1, .. }
    ));
    assert!(matches!(
        rows[2],
        RuntimeSessionResponseV2::ArchiveProgress {
            request_id: None,
            durable_tick: 1,
            verified_tick: 1,
            ..
        }
    ));
    assert!(matches!(
        rows[3],
        RuntimeSessionResponseV2::Stopped { request_id: 8 }
    ));
    assert_eq!(backend.tick, 1);
    assert!(
        backend.state.refreshes.lock().unwrap().is_empty(),
        "Advance did not invoke a synchronous Archive fallback"
    );
    assert!(backend.state.joined.load(Ordering::SeqCst));
}

#[test]
fn ready_precedes_archive_completion_queued_during_driver_start() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let mut output = Vec::new();
    serve(
        Cursor::new(wire(&[stop()])),
        &mut output,
        &mut backend,
        "campaign",
        "digest".into(),
        move |sink| {
            assert!(sink(progress(None, 0, 0)));
            Ok(driver(&state))
        },
    )
    .unwrap();
    let rows = responses(&output);
    assert!(matches!(rows[0], RuntimeSessionResponseV2::Ready { .. }));
    assert!(matches!(
        rows[1],
        RuntimeSessionResponseV2::ArchiveProgress {
            durable_tick: 0,
            ..
        }
    ));
    assert_eq!(backend.tick, 0);
}

#[test]
fn explicit_refresh_uses_the_driver_and_preserves_its_request_identity() {
    let mut backend = backend();
    let mut output = Vec::new();
    run(wire(&[refresh(), stop()]), &mut backend, &mut output).unwrap();
    assert_eq!(*backend.state.refreshes.lock().unwrap(), [31]);
    assert_eq!(backend.tick, 0);
    assert!(responses(&output).iter().any(|row| matches!(
        row,
        RuntimeSessionResponseV2::ArchiveProgress {
            request_id: Some(31),
            durable_tick: 0,
            ..
        }
    )));
}

#[test]
fn failed_commit_and_duplicate_tail_never_publish_a_second_week() {
    for failed in [false, true] {
        let mut backend = backend();
        backend.fail_commit = failed;
        let mut output = Vec::new();
        run(wire(&[advance(), advance()]), &mut backend, &mut output).unwrap();
        let rows = responses(&output);
        assert_eq!(
            rows.iter()
                .filter(|row| matches!(row, RuntimeSessionResponseV2::Committed { .. }))
                .count(),
            usize::from(!failed)
        );
        assert_eq!(backend.tick, u64::from(!failed));
        assert!(backend.state.refreshes.lock().unwrap().is_empty());
        let expected = if failed {
            RuntimeSessionErrorCodeV2::CommitRefused
        } else {
            RuntimeSessionErrorCodeV2::StaleExpectedTail
        };
        assert!(rows.iter().any(
            |row| matches!(row, RuntimeSessionResponseV2::Error { code, .. } if *code == expected)
        ));
        assert!(
            !rows
                .iter()
                .any(|row| matches!(row, RuntimeSessionResponseV2::Stopped { .. })),
            "EOF does not manufacture Stop"
        );
    }
}

#[test]
fn malformed_actions_versions_campaigns_and_overlong_frames_cannot_advance() {
    assert!(serde_json::from_str::<RuntimeSessionRequestV2>(r#"{"type":"advance","protocol_version":2,"campaign_id":"campaign","request_id":1,"expected_tail":{"resolve_tick":0,"tick_content_hash":null},"actions":[1]}"#).is_err());
    for (version, campaign, expected) in [
        (1, "campaign", RuntimeSessionErrorCodeV2::UnsupportedVersion),
        (3, "campaign", RuntimeSessionErrorCodeV2::UnsupportedVersion),
        (2, "other", RuntimeSessionErrorCodeV2::CampaignMismatch),
    ] {
        let mut request = advance();
        if let RuntimeSessionRequestV2::Advance {
            protocol_version,
            campaign_id,
            ..
        } = &mut request
        {
            *protocol_version = version;
            *campaign_id = campaign.into();
        }
        let mut backend = backend();
        let mut output = Vec::new();
        run(wire(&[request]), &mut backend, &mut output).unwrap();
        assert_eq!(backend.tick, 0);
        assert!(
            matches!(responses(&output)[1], RuntimeSessionResponseV2::Error { code, .. } if code == expected)
        );
    }
    let mut backend = backend();
    let mut output = Vec::new();
    assert_eq!(
        run(
            vec![b' '; super::super::RUNTIME_SESSION_MAX_LINE_BYTES_V2 + 1],
            &mut backend,
            &mut output
        ),
        Err(RuntimeSessionErrorCodeV2::InvalidRequest)
    );
    assert_eq!(backend.tick, 0);
    assert!(serde_json::from_str::<RuntimeSessionResponseV2>(r#"{"type":"archive_progress","request_id":0,"campaign_id":"campaign","durable_tick":1,"verified_tick":1}"#).is_err());
}

#[test]
fn old_future_and_invalid_progress_never_relabel_the_current_durable_tail() {
    let mut backend = backend();
    backend.tick = 3;
    let archive = driver(&backend.state);
    let mut output = Vec::new();
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        coordinator.ready("digest".into()).unwrap();
        coordinator.archive_event(&progress(None, 2, 2)).unwrap();
        coordinator.archive_event(&progress(Some(7), 2, 2)).unwrap();
        coordinator.archive_event(&progress(None, 4, 4)).unwrap();
        coordinator.archive_event(&progress(None, 3, 4)).unwrap();
        coordinator.archive_event(&progress(None, 3, 2)).unwrap();
        coordinator.archive_event(&progress(None, 3, 1)).unwrap();
        coordinator.archive_event(&progress(None, 3, 2)).unwrap();
    }
    let rows = responses(&output);
    let progresses: Vec<_> = rows
        .iter()
        .filter(|row| matches!(row, RuntimeSessionResponseV2::ArchiveProgress { .. }))
        .collect();
    assert_eq!(
        progresses.len(),
        2,
        "both genuine equal-P publications remain observable"
    );
    assert!(progresses.iter().all(|row| matches!(
        row,
        RuntimeSessionResponseV2::ArchiveProgress {
            durable_tick: 3,
            verified_tick: 2,
            ..
        }
    )));
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(
                row,
                RuntimeSessionResponseV2::Error {
                    code: RuntimeSessionErrorCodeV2::StaleExpectedTail,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(
                row,
                RuntimeSessionResponseV2::Error {
                    code: RuntimeSessionErrorCodeV2::ArchiveRefused,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(backend.tick, 3);
}

#[test]
fn timeout_refuses_stopped_and_never_joins_an_unfinished_driver() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let mut archive = driver(&state);
    archive.finishes_on_stop = false;
    let mut output = Vec::new();
    let (_sender, receiver) = mpsc::sync_channel(1);
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        assert_eq!(
            coordinator.shutdown(&receiver, Some(8), Duration::ZERO),
            Err(RuntimeSessionErrorCodeV2::StorageCanceled)
        );
    }
    assert!(state.stopped.load(Ordering::SeqCst));
    assert!(!state.joined.load(Ordering::SeqCst));
    assert!(matches!(
        responses(&output).as_slice(),
        [RuntimeSessionResponseV2::Error {
            request_id: Some(8),
            code: RuntimeSessionErrorCodeV2::StorageCanceled,
            ..
        }]
    ));
}

#[test]
fn finished_worker_drains_full_event_queue_without_accepting_queued_advance() {
    let mut backend = backend();
    let mut output = Vec::new();
    let archive = driver(&backend.state);
    let (sender, receiver) = mpsc::sync_channel(2);
    sender
        .send(SessionEvent::Archive(progress(None, 0, 0)))
        .unwrap();
    sender
        .send(SessionEvent::Input(InputEvent::Frame(wire(&[advance()]))))
        .unwrap();
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        coordinator
            .shutdown(&receiver, Some(8), Duration::from_secs(1))
            .unwrap();
    }
    assert_eq!(backend.tick, 0);
    assert!(matches!(
        responses(&output).as_slice(),
        [
            RuntimeSessionResponseV2::ArchiveProgress { .. },
            RuntimeSessionResponseV2::Stopped { request_id: 8 }
        ]
    ));
}

struct BrokenOutput;
impl Write for BrokenOutput {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
    fn flush(&mut self) -> io::Result<()> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn broken_output_requests_stop_without_waiting_for_sync_driver_teardown() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let mut archive = driver(&state);
    archive.finishes_on_stop = false;
    let result = serve(
        Cursor::new(Vec::new()),
        &mut BrokenOutput,
        &mut backend,
        "campaign",
        "digest".into(),
        |_| Ok(archive),
    );
    assert_eq!(result, Err(RuntimeSessionErrorCodeV2::PipeFailure));
    assert!(state.stopped.load(Ordering::SeqCst));
    assert!(!state.joined.load(Ordering::SeqCst));
    assert_eq!(backend.tick, 0);
}

#[test]
fn full_refresh_queue_refuses_the_request_without_fabricating_progress() {
    let mut backend = backend();
    let mut output = Vec::new();
    let mut archive = driver(&backend.state);
    archive.refresh_full = true;
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        coordinator.request(&wire(&[refresh()])).unwrap();
    }
    assert!(matches!(
        responses(&output).as_slice(),
        [RuntimeSessionResponseV2::Error {
            request_id: Some(31),
            code: RuntimeSessionErrorCodeV2::StorageBusy,
            ..
        }]
    ));
    assert!(backend.state.refreshes.lock().unwrap().is_empty());
}

#[test]
fn unexpected_driver_stop_refuses_active_session_before_queued_advance() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let mut output = Vec::new();
    let result = serve(
        Cursor::new(wire(&[advance()])),
        &mut output,
        &mut backend,
        "campaign",
        "digest".into(),
        move |sink| {
            assert!(sink(ArchiveDriverEventV1::Stopped));
            Ok(driver(&state))
        },
    );
    assert_eq!(result, Err(RuntimeSessionErrorCodeV2::ArchiveRefused));
    assert_eq!(backend.tick, 0);
    assert!(matches!(
        responses(&output).as_slice(),
        [
            RuntimeSessionResponseV2::Ready { .. },
            RuntimeSessionResponseV2::Error {
                request_id: None,
                code: RuntimeSessionErrorCodeV2::ArchiveRefused,
                ..
            }
        ]
    ));
}

#[test]
fn retrying_manual_failure_keeps_correlation_without_claiming_success() {
    let mut backend = backend();
    let archive = driver(&backend.state);
    let mut output = Vec::new();
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        for request_id in [Some(31), None] {
            coordinator
                .archive_event(&ArchiveDriverEventV1::Failure {
                    request_id,
                    failure: crate::archive_driver::ArchiveDriverFailureV1::Disconnected,
                    retrying: true,
                })
                .unwrap();
        }
        coordinator.archive_event(&progress(None, 0, 0)).unwrap();
    }
    assert!(matches!(
        responses(&output).as_slice(),
        [
            RuntimeSessionResponseV2::Error {
                request_id: Some(31),
                code: RuntimeSessionErrorCodeV2::ArchiveRefused,
                ..
            },
            RuntimeSessionResponseV2::ArchiveProgress {
                request_id: None,
                ..
            }
        ]
    ));
    assert_eq!(backend.tick, 0);
}

struct OpenInput {
    started: mpsc::SyncSender<()>,
    release: Receiver<()>,
}
impl Read for OpenInput {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        self.fill_buf().map(|_| 0)
    }
}
impl BufRead for OpenInput {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let _ = self.started.try_send(());
        let _ = self.release.recv();
        Ok(&[])
    }
    fn consume(&mut self, _: usize) {}
}

#[test]
fn silent_driver_panic_refuses_while_input_remains_open() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let (started, entered) = mpsc::sync_channel(1);
    let (release, waiting) = mpsc::sync_channel(1);
    let mut output = Vec::new();
    let result = serve(
        OpenInput {
            started,
            release: waiting,
        },
        &mut output,
        &mut backend,
        "campaign",
        "digest".into(),
        |_| {
            state.exited.store(true, Ordering::SeqCst);
            let mut archive = driver(&state);
            archive.join_fails = true;
            Ok(archive)
        },
    );
    // The owned input was genuinely blocked, not an EOF that stopped the worker.
    entered.recv_timeout(Duration::from_secs(1)).unwrap();
    drop(release);
    assert_eq!(result, Err(RuntimeSessionErrorCodeV2::ArchiveRefused));
    assert!(state.joined.load(Ordering::SeqCst));
    assert_eq!(backend.tick, 0);
    assert!(matches!(
        responses(&output).as_slice(),
        [
            RuntimeSessionResponseV2::Ready { .. },
            RuntimeSessionResponseV2::Error {
                request_id: None,
                code: RuntimeSessionErrorCodeV2::ArchiveRefused,
                ..
            }
        ]
    ));
}

#[test]
fn cooperative_shutdown_waits_for_completion_after_last_sender_drops() {
    let mut backend = backend();
    let state = Arc::clone(&backend.state);
    let archive = driver(&state);
    archive.finishing_checks.set(1);
    let (sender, receiver) = mpsc::sync_channel(1);
    drop(sender);
    let mut output = Vec::new();
    {
        let mut coordinator = Coordinator::new(&mut output, &mut backend, archive, "campaign");
        coordinator
            .shutdown(&receiver, Some(8), Duration::from_secs(1))
            .unwrap();
    }
    assert!(state.joined.load(Ordering::SeqCst));
    assert!(matches!(
        responses(&output).as_slice(),
        [RuntimeSessionResponseV2::Stopped { request_id: 8 }]
    ));
}
