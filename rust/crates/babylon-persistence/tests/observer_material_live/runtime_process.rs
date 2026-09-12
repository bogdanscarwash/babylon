//! Actual stdio-process qualification on the existing disposable runtime clone.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use babylon_persistence::michigan_content::MichiganContentPreset;
use babylon_persistence::michigan_material::MichiganDeliveryPreset;
use babylon_persistence::runtime_session::{
    RuntimeSessionErrorCode, RuntimeSessionPreset, RuntimeSessionRequest, RuntimeSessionResponse,
    RuntimeSessionScope, RuntimeSessionTarget, RUNTIME_SESSION_MAX_LINE_BYTES,
    RUNTIME_SESSION_PROTOCOL_VERSION,
};

use super::{
    advance_material_period, validate_connection_target, CampaignId, Config, DisposableTarget,
    DurableMaterialRuntime, Uuid,
};

const STARTUP_LIMIT: Duration = Duration::from_secs(60);
const EXIT_LIMIT: Duration = Duration::from_secs(30);
const CLEANUP_LIMIT: Duration = Duration::from_secs(5);

struct RuntimeChild(Child, Arc<Mutex<Vec<u8>>>);

impl RuntimeChild {
    fn start(target: &DisposableTarget) -> Self {
        let dsn = child_dsn(&target.writer);
        let diagnostics = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = Self(
            Command::new(env!("CARGO_BIN_EXE_babylon-runtime"))
                .args([
                    "session",
                    "--stdio",
                    "--defines",
                    concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../content/scenarios/michigan/defines.toml"
                    ),
                ])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .env_clear()
                .env("BABYLON_RUNTIME_DSN", dsn)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("start the actual runtime binary"),
            Arc::clone(&diagnostics),
        );
        let mut stderr = runtime.0.stderr.take().unwrap();
        let _diagnostics = thread::spawn(move || capture_diagnostics(&mut stderr, &diagnostics));
        runtime
    }

    fn diagnostics(&self) -> String {
        String::from_utf8_lossy(&self.1.lock().unwrap()).into_owned()
    }

    fn receive(
        &self,
        input: BufReader<ChildStdout>,
    ) -> (RuntimeSessionResponse, BufReader<ChildStdout>) {
        let (send, receive) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let mut input = input;
            let mut bytes = Vec::new();
            let result = input
                .by_ref()
                .take((RUNTIME_SESSION_MAX_LINE_BYTES + 1) as u64)
                .read_until(b'\n', &mut bytes)
                .ok()
                .filter(|size| {
                    *size > 0 && *size <= RUNTIME_SESSION_MAX_LINE_BYTES && bytes.ends_with(b"\n")
                })
                .and_then(|_| serde_json::from_slice::<RuntimeSessionResponse>(&bytes).ok());
            let _ = send.send((result, input));
        });
        let (response, input) = receive
            .recv_timeout(STARTUP_LIMIT)
            .unwrap_or_else(|_| panic!("runtime response deadline: {}", self.diagnostics()));
        reader.join().unwrap();
        (
            response.unwrap_or_else(|| panic!("invalid runtime response: {}", self.diagnostics())),
            input,
        )
    }

    fn hello(&mut self) -> BufReader<ChildStdout> {
        let stdout = self.0.stdout.take().unwrap();
        let (response, input) = self.receive(BufReader::new(stdout));
        assert!(
            matches!(response,
                RuntimeSessionResponse::Hello { protocol_version, scope }
                if protocol_version == RUNTIME_SESSION_PROTOCOL_VERSION
                    && scope.epoch == 0 && scope.campaign_id.is_none()
            ),
            "runtime did not emit Hello: {}",
            self.diagnostics()
        );
        input
    }

    fn ready(&mut self, campaign: CampaignId) -> BufReader<ChildStdout> {
        let input = self.hello();
        self.send(&RuntimeSessionRequest::Switch {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id: 1,
            scope: RuntimeSessionScope {
                epoch: 0,
                campaign_id: None,
            },
            target: RuntimeSessionTarget::Open {
                campaign_id: campaign.as_uuid().to_string(),
            },
        })
        .unwrap();
        let (response, input) = self.receive(input);
        assert!(
            matches!(response, RuntimeSessionResponse::Switching { request_id: 1, scope, .. } if scope == campaign_scope(campaign, 1))
        );
        let (response, input) = self.receive(input);
        assert!(
            matches!(response, RuntimeSessionResponse::Ready { request_id: 1, scope, tail, .. } if scope == campaign_scope(campaign, 1) && tail.resolve_tick == 1),
            "runtime did not emit Ready: {}",
            self.diagnostics()
        );
        input
    }

    fn send(&mut self, request: &RuntimeSessionRequest) -> std::io::Result<()> {
        let mut bytes = serde_json::to_vec(request).unwrap();
        bytes.push(b'\n');
        let input = self.0.stdin.as_mut().unwrap();
        input.write_all(&bytes).and_then(|()| input.flush())
    }

    fn wait(&mut self, limit: Duration) -> Option<ExitStatus> {
        let started = Instant::now();
        loop {
            if let Some(status) = self.0.try_wait().expect("observe exact runtime child") {
                return Some(status);
            }
            if started.elapsed() >= limit {
                return None;
            }
            thread::park_timeout(Duration::from_millis(20));
        }
    }
}

impl Drop for RuntimeChild {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(Some(_))) {
            return;
        }
        // Cleanup only: target exactly this spawned PID, never another process.
        // A timeout still fails the test; forceful cleanup cannot make it pass.
        let _ = self.0.kill();
        if self.wait(CLEANUP_LIMIT).is_none() {
            eprintln!("test cleanup could not reap its exact runtime child within five seconds");
        }
    }
}

fn assert_broken_stdout_exits_with_stdin_open(target: &DisposableTarget, campaign: CampaignId) {
    let mut child = RuntimeChild::start(target);
    drop(child.ready(campaign));
    let refresh = RuntimeSessionRequest::RefreshArchive {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
        scope: campaign_scope(campaign, 1),
        request_id: 91,
    };
    match child.send(&refresh) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {
            // Startup progress may already have detected the closed response pipe.
        }
        Err(error) => panic!("unexpected control-pipe error: {error}"),
    }
    assert!(child.0.stdin.is_some(), "the parent has not sent EOF");
    let status = child
        .wait(EXIT_LIMIT)
        .unwrap_or_else(|| panic!("broken stdout exit deadline: {}", child.diagnostics()));
    assert!(
        !status.success(),
        "broken output cannot report a successful session: {}",
        child.diagnostics()
    );
    assert!(child.0.stdin.is_some());
}

fn assert_orderly_exit(target: &DisposableTarget, campaign: CampaignId, explicit_stop: bool) {
    let mut child = RuntimeChild::start(target);
    let mut output = child.ready(campaign);
    let drain = thread::spawn(move || {
        let mut bytes = Vec::new();
        output
            .by_ref()
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() <= 1_048_576, "bounded test transcript");
        bytes
    });
    if explicit_stop {
        child
            .send(&RuntimeSessionRequest::Stop {
                protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
                scope: campaign_scope(campaign, 1),
                request_id: 92,
            })
            .unwrap();
    } else {
        drop(child.0.stdin.take());
    }
    let status = child
        .wait(EXIT_LIMIT)
        .unwrap_or_else(|| panic!("ordinary Stop/EOF exit deadline: {}", child.diagnostics()));
    assert!(
        status.success(),
        "runtime exit refusal: {}",
        child.diagnostics()
    );
    let bytes = drain.join().unwrap();
    let responses = bytes
        .split(|byte| *byte == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| serde_json::from_slice::<RuntimeSessionResponse>(row).unwrap())
        .collect::<Vec<_>>();
    assert!(responses.iter().all(|response| matches!(
        response,
        RuntimeSessionResponse::ArchiveProgress {
            durable_tick: 1,
            ..
        } | RuntimeSessionResponse::Stopped { request_id: 92, .. }
    )));
    assert_eq!(
        responses
            .iter()
            .filter(|response| matches!(
                response,
                RuntimeSessionResponse::Stopped { request_id: 92, .. }
            ))
            .count(),
        usize::from(explicit_stop)
    );
    if explicit_stop {
        assert!(matches!(
            responses.last(),
            Some(RuntimeSessionResponse::Stopped { request_id: 92, scope }) if scope == &campaign_scope(campaign, 1)
        ));
    }
}

fn campaign_scope(campaign: CampaignId, epoch: u64) -> RuntimeSessionScope {
    RuntimeSessionScope {
        epoch,
        campaign_id: Some(campaign.as_uuid().to_string()),
    }
}

fn switch_campaign(
    child: &mut RuntimeChild,
    mut input: BufReader<ChildStdout>,
    request_id: u64,
    previous: &RuntimeSessionScope,
    target: RuntimeSessionTarget,
    expected: &RuntimeSessionScope,
) -> (RuntimeSessionResponse, BufReader<ChildStdout>) {
    child
        .send(&RuntimeSessionRequest::Switch {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id,
            scope: previous.clone(),
            target,
        })
        .unwrap();
    for _ in 0..64 {
        let (response, next) = child.receive(input);
        input = next;
        match response {
            RuntimeSessionResponse::ArchiveProgress { scope, .. } => assert_eq!(&scope, previous),
            RuntimeSessionResponse::Switching {
                request_id: observed,
                previous_scope,
                scope,
            } => {
                assert_eq!(observed, request_id);
                assert_eq!(&previous_scope, previous);
                assert_eq!(&scope, expected);
                let (result, input) = child.receive(input);
                assert!(
                    matches!(&result,
                    RuntimeSessionResponse::Ready { request_id: observed, scope, .. }
                    if *observed == request_id && scope == expected)
                        || matches!(&result,
                    RuntimeSessionResponse::Error { request_id: Some(observed), scope, tail: None, .. }
                    if *observed == request_id && scope == expected),
                    "new-scope admission must precede progress: {result:?}"
                );
                return (result, input);
            }
            other => panic!("unexpected pre-switch response: {other:?}"),
        }
    }
    panic!("switch exceeded bounded response transcript");
}

fn open_target(campaign: CampaignId) -> RuntimeSessionTarget {
    RuntimeSessionTarget::Open {
        campaign_id: campaign.as_uuid().to_string(),
    }
}

fn next_control_response(
    child: &RuntimeChild,
    mut input: BufReader<ChildStdout>,
    expected: &RuntimeSessionScope,
) -> (RuntimeSessionResponse, BufReader<ChildStdout>) {
    for _ in 0..64 {
        let (response, next) = child.receive(input);
        input = next;
        if let RuntimeSessionResponse::ArchiveProgress { scope, .. } = response {
            assert_eq!(&scope, expected);
        } else {
            return (response, input);
        }
    }
    panic!("control response exceeded bounded transcript");
}

fn assert_missing_open_and_new_collision(
    target: &DisposableTarget,
    child: &mut RuntimeChild,
    input: BufReader<ChildStdout>,
    first: CampaignId,
    missing: CampaignId,
) -> BufReader<ChildStdout> {
    let empty = RuntimeSessionScope {
        epoch: 0,
        campaign_id: None,
    };
    let missing_scope = campaign_scope(missing, 1);
    let first_scope = campaign_scope(first, 2);
    let refused_scope = campaign_scope(first, 3);
    let (reply, input) = switch_campaign(
        child,
        input,
        1,
        &empty,
        open_target(missing),
        &missing_scope,
    );
    assert!(matches!(
        reply,
        RuntimeSessionResponse::Error {
            code: RuntimeSessionErrorCode::CampaignAbsent,
            ..
        }
    ));
    let missing_count: i64 = target
        .writer
        .connect(postgres::NoTls)
        .unwrap()
        .query_one(
            "SELECT count(*) FROM babylon_state.campaign WHERE campaign_id=$1",
            &[missing.as_uuid()],
        )
        .unwrap()
        .get(0);
    assert_eq!(missing_count, 0, "Open must never found an absent campaign");
    let (reply, input) = switch_campaign(
        child,
        input,
        2,
        &missing_scope,
        open_target(first),
        &first_scope,
    );
    assert!(matches!(reply, RuntimeSessionResponse::Ready { tail, .. } if tail.resolve_tick == 1));
    let (reply, input) = switch_campaign(
        child,
        input,
        3,
        &first_scope,
        RuntimeSessionTarget::New {
            campaign_id: first.as_uuid().to_string(),
            preset: RuntimeSessionPreset::Delayed,
        },
        &refused_scope,
    );
    assert!(matches!(
        reply,
        RuntimeSessionResponse::Error {
            code: RuntimeSessionErrorCode::CampaignAlreadyExists,
            ..
        }
    ));
    input
}

fn assert_switch_stop(
    child: &mut RuntimeChild,
    input: BufReader<ChildStdout>,
    scope: &RuntimeSessionScope,
) {
    child
        .send(&RuntimeSessionRequest::Stop {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id: 7,
            scope: scope.clone(),
        })
        .unwrap();
    let (response, input) = next_control_response(child, input, scope);
    assert!(
        matches!(response, RuntimeSessionResponse::Stopped { request_id: 7, scope: stopped } if &stopped == scope)
    );
    assert!(child
        .wait(EXIT_LIMIT)
        .expect("bounded orderly exit")
        .success());
    let mut after_stop = Vec::new();
    input.take(1).read_to_end(&mut after_stop).unwrap();
    assert!(
        after_stop.is_empty(),
        "Stopped must be the terminal response"
    );
}

#[test]
#[ignore = "requires task-owned disposable PostgreSQL runtime and actual runtime binary"]
fn live_runtime_child_switch_failure_retry_and_epoch_isolation_preserve_campaigns() {
    let target = DisposableTarget::create();
    let first = CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_0081));
    let second = CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_0082));
    let missing = CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_0083));
    let preset = MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard);
    let mut durable = DurableMaterialRuntime::create(
        &target.writer,
        first,
        preset
            .create_foundation(&crate::test_support::catalog())
            .unwrap(),
    )
    .unwrap();
    advance_material_period(&mut durable);
    let original_tail = durable.tail().copied();
    let original_world = durable.session().current_world_hash().unwrap();
    drop(durable);
    let mut child = RuntimeChild::start(&target);
    let process = child.0.id();
    let input = child.hello();
    let input = assert_missing_open_and_new_collision(&target, &mut child, input, first, missing);
    let second_scope = campaign_scope(second, 4);
    let reopened_scope = campaign_scope(first, 5);
    let (reply, input) = switch_campaign(
        &mut child,
        input,
        4,
        &campaign_scope(first, 3),
        RuntimeSessionTarget::New {
            campaign_id: second.as_uuid().to_string(),
            preset: RuntimeSessionPreset::Delayed,
        },
        &second_scope,
    );
    assert!(matches!(reply, RuntimeSessionResponse::Ready { tail, .. } if tail.resolve_tick == 0));
    child
        .send(&RuntimeSessionRequest::Switch {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id: 5,
            scope: campaign_scope(first, 2),
            target: open_target(first),
        })
        .unwrap();
    let (response, input) = next_control_response(&child, input, &second_scope);
    assert!(
        matches!(response, RuntimeSessionResponse::Error { request_id: Some(5), scope, code: RuntimeSessionErrorCode::SessionMismatch, .. } if scope == second_scope)
    );
    let (reply, input) = switch_campaign(
        &mut child,
        input,
        6,
        &second_scope,
        open_target(first),
        &reopened_scope,
    );
    assert!(matches!(reply, RuntimeSessionResponse::Ready { tail, .. } if tail.resolve_tick == 1));
    assert_eq!(
        child.0.id(),
        process,
        "switch must retain the runtime process"
    );
    assert_switch_stop(&mut child, input, &reopened_scope);
    let reopened = DurableMaterialRuntime::open(
        &target.writer,
        first,
        preset
            .admitted(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap();
    assert_eq!(reopened.tail(), original_tail.as_ref());
    assert_eq!(
        reopened.session().current_world_hash().unwrap(),
        original_world
    );
    let created = DurableMaterialRuntime::open(
        &target.writer,
        second,
        MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Delayed)
            .admitted(&crate::test_support::catalog())
            .unwrap()
            .digest(),
    )
    .unwrap();
    assert_eq!(created.session().completed_tick(), 0);
}

enum ExitMode {
    BrokenOutput,
    Stop,
    Eof,
}

#[test]
#[ignore = "requires the task-owned disposable PostgreSQL runtime and actual runtime binary"]
fn live_runtime_child_pipe_failure_and_orderly_exit_preserve_committed_world() {
    let target = DisposableTarget::create();
    let campaign =
        CampaignId::from_uuid(Uuid::from_u128(0x0044_0000_0000_0000_0000_0000_0000_007f));
    let preset = MichiganContentPreset::new_campaign(MichiganDeliveryPreset::Standard);
    let admitted = preset.admitted(&crate::test_support::catalog()).unwrap();
    let mut runtime = DurableMaterialRuntime::create(
        &target.writer,
        campaign,
        preset
            .create_foundation(&crate::test_support::catalog())
            .unwrap(),
    )
    .unwrap();
    advance_material_period(&mut runtime);
    let expected_tail = runtime.tail().copied();
    let expected_world = runtime.session().current_world_hash().unwrap();
    drop(runtime);
    for mode in [ExitMode::BrokenOutput, ExitMode::Stop, ExitMode::Eof] {
        match mode {
            ExitMode::BrokenOutput => assert_broken_stdout_exits_with_stdin_open(&target, campaign),
            ExitMode::Stop => assert_orderly_exit(&target, campaign, true),
            ExitMode::Eof => assert_orderly_exit(&target, campaign, false),
        }
        let reopened =
            DurableMaterialRuntime::open(&target.writer, campaign, admitted.digest()).unwrap();
        assert_eq!(reopened.session().completed_tick(), 1);
        assert_eq!(reopened.tail(), expected_tail.as_ref());
        assert_eq!(
            reopened.session().current_world_hash().unwrap(),
            expected_world
        );
    }
}

fn child_dsn(config: &Config) -> String {
    validate_connection_target(config).unwrap();
    let [postgres::config::Host::Tcp(host)] = config.get_hosts() else {
        panic!("runtime process fixture requires one validated local TCP host");
    };
    let [port] = config.get_ports() else {
        panic!("runtime process fixture requires its explicit disposable port");
    };
    let password = std::str::from_utf8(config.get_password().expect("test password"))
        .expect("UTF-8 test password");
    format!(
        "host={} port={port} dbname={} user={} password={}",
        quote_dsn_value(host),
        quote_dsn_value(config.get_dbname().expect("owned test database")),
        quote_dsn_value(config.get_user().expect("test user")),
        quote_dsn_value(password),
    )
}

fn quote_dsn_value(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn capture_diagnostics(input: &mut impl Read, captured: &Mutex<Vec<u8>>) {
    let mut buffer = [0_u8; 1024];
    while let Ok(size) = input.read(&mut buffer) {
        if size == 0 {
            return;
        }
        let mut output = captured.lock().unwrap();
        let keep = size.min(8192_usize.saturating_sub(output.len()));
        output.extend_from_slice(&buffer[..keep]);
        // Keep draining after the retained-byte bound so diagnostics cannot stall
        // the child or induce an unrelated broken stderr pipe.
    }
}

#[test]
fn runtime_child_dsn_selects_owned_database_from_uri_and_keyword_configs() {
    for source in [
        "postgresql://test:test@127.0.0.1:5433/postgres",
        "host=127.0.0.1 port=5433 user=test password=test dbname=postgres",
    ] {
        let mut config: Config = source.parse().unwrap();
        config.dbname("per281_runtime_materialobserver_42");
        // Exercise exact libpq quoting without using or printing real credentials.
        config.password("fixture ' quote \\ slash");
        let parsed: Config = child_dsn(&config).parse().unwrap();
        validate_connection_target(&parsed).unwrap();
        assert_eq!(
            parsed.get_dbname(),
            Some("per281_runtime_materialobserver_42")
        );
        assert_eq!(parsed.get_ports(), [5433]);
        assert_eq!(parsed.get_hosts(), config.get_hosts());
        assert_eq!(parsed.get_user(), config.get_user());
        assert_eq!(parsed.get_password(), config.get_password());
    }
}

#[test]
fn runtime_child_diagnostics_are_bounded_and_fully_drained() {
    let mut source = std::io::Cursor::new(vec![b'x'; 20_000]);
    let captured = Mutex::new(Vec::new());
    capture_diagnostics(&mut source, &captured);
    assert_eq!(source.position(), 20_000);
    assert_eq!(captured.into_inner().unwrap(), vec![b'x'; 8192]);
}
