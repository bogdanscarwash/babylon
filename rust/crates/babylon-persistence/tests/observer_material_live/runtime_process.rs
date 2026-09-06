//! Actual stdio-process qualification on the existing disposable runtime clone.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use babylon_persistence::michigan_content::MichiganContentPresetV1;
use babylon_persistence::runtime_session::{
    RuntimeSessionRequestV2, RuntimeSessionResponseV2, RUNTIME_SESSION_MAX_LINE_BYTES_V2,
    RUNTIME_SESSION_PROTOCOL_VERSION_V2,
};

use super::{
    advance_material_week, validate_legacy_connection_target, CampaignId, Config, DisposableTarget,
    DurableMaterialRuntimeV3, Uuid,
};

const STARTUP_LIMIT: Duration = Duration::from_secs(60);
const EXIT_LIMIT: Duration = Duration::from_secs(30);
const CLEANUP_LIMIT: Duration = Duration::from_secs(5);

struct RuntimeChild(Child, Arc<Mutex<Vec<u8>>>);

impl RuntimeChild {
    fn start(target: &DisposableTarget, campaign: CampaignId) -> Self {
        let dsn = child_dsn(&target.writer);
        let diagnostics = Arc::new(Mutex::new(Vec::new()));
        let mut runtime = Self(
            Command::new(env!("CARGO_BIN_EXE_babylon-runtime"))
                .args(["session", "--stdio"])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .env_clear()
                .env("BABYLON_RUNTIME_DSN", dsn)
                .env("BABYLON_CAMPAIGN_ID", campaign.as_uuid().to_string())
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

    fn ready(&mut self, campaign: CampaignId) -> BufReader<ChildStdout> {
        let stdout = self.0.stdout.take().unwrap();
        let (send, receive) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let mut input = BufReader::new(stdout);
            let mut bytes = Vec::new();
            let result = input
                .by_ref()
                .take((RUNTIME_SESSION_MAX_LINE_BYTES_V2 + 1) as u64)
                .read_until(b'\n', &mut bytes)
                .ok()
                .filter(|size| {
                    *size > 0
                        && *size <= RUNTIME_SESSION_MAX_LINE_BYTES_V2
                        && bytes.ends_with(b"\n")
                })
                .and_then(|_| serde_json::from_slice::<RuntimeSessionResponseV2>(&bytes).ok());
            let _ = send.send((result, input));
        });
        let (response, input) = receive
            .recv_timeout(STARTUP_LIMIT)
            .unwrap_or_else(|_| panic!("runtime Ready deadline: {}", self.diagnostics()));
        reader.join().unwrap();
        assert!(
            matches!(response,
                Some(RuntimeSessionResponseV2::Ready { protocol_version, campaign_id, tail, .. })
                if protocol_version == RUNTIME_SESSION_PROTOCOL_VERSION_V2
                    && campaign_id == campaign.as_uuid().to_string()
                    && tail.resolve_tick == 1
            ),
            "runtime did not emit Ready: {}",
            self.diagnostics()
        );
        input
    }

    fn send(&mut self, request: &RuntimeSessionRequestV2) -> std::io::Result<()> {
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
    let mut child = RuntimeChild::start(target, campaign);
    drop(child.ready(campaign));
    let refresh = RuntimeSessionRequestV2::RefreshArchive {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V2,
        campaign_id: campaign.as_uuid().to_string(),
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
    let mut child = RuntimeChild::start(target, campaign);
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
            .send(&RuntimeSessionRequestV2::Stop {
                protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V2,
                campaign_id: campaign.as_uuid().to_string(),
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
        .map(|row| serde_json::from_slice::<RuntimeSessionResponseV2>(row).unwrap())
        .collect::<Vec<_>>();
    assert!(responses.iter().all(|response| matches!(
        response,
        RuntimeSessionResponseV2::ArchiveProgress {
            durable_tick: 1,
            ..
        } | RuntimeSessionResponseV2::Stopped { request_id: 92 }
    )));
    assert_eq!(
        responses
            .iter()
            .filter(|response| matches!(
                response,
                RuntimeSessionResponseV2::Stopped { request_id: 92 }
            ))
            .count(),
        usize::from(explicit_stop)
    );
    if explicit_stop {
        assert!(matches!(
            responses.last(),
            Some(RuntimeSessionResponseV2::Stopped { request_id: 92 })
        ));
    }
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
    let preset = MichiganContentPresetV1::BundlesStandardV3;
    let admitted = preset.admitted().unwrap();
    let mut runtime = DurableMaterialRuntimeV3::create(
        &target.writer,
        campaign,
        preset.create_foundation().unwrap(),
    )
    .unwrap();
    advance_material_week(&mut runtime);
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
            DurableMaterialRuntimeV3::open(&target.writer, campaign, admitted.digest()).unwrap();
        assert_eq!(reopened.session().completed_tick(), 1);
        assert_eq!(reopened.tail(), expected_tail.as_ref());
        assert_eq!(
            reopened.session().current_world_hash().unwrap(),
            expected_world
        );
    }
}

fn child_dsn(config: &Config) -> String {
    validate_legacy_connection_target(config).unwrap();
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
        validate_legacy_connection_target(&parsed).unwrap();
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
