//! Black-box protocol test: spawns the REAL `bsl-ls` binary and speaks
//! framed JSON-RPC over its stdio pipes. This is the workspace's first
//! subprocess-IPC test (plan §5.3: `rg -l 'CARGO_BIN_EXE|Stdio::piped'
//! rust/crates` returns nothing today, so this file establishes the
//! pattern rather than copying one).
//!
//! Task 5.1's required rows: the `initialize` -> `initialized` ->
//! `shutdown` -> `exit(0)` round trip, and a request sent BEFORE
//! `initialize` getting `-32002` (`ServerNotInitialized`). A bonus row
//! (`exit` with no prior `shutdown` -> exit code 1) rides the same harness
//! at negligible extra cost and exercises the SAME production code path
//! `src/lifecycle.rs`'s own `Connection::memory()` unit tests already cover
//! in-process — this file's value is proving the REAL compiled binary
//! behaves the same way over real pipes.
//!
//! **Determinism note** (plan §1 constraint 2): the `recv_timeout(30s)`
//! watchdog below, and the exit-status poll it shares its bound with, are
//! ONE of the two declared carve-outs in the whole train — they can only
//! convert a hang into a FAILURE, never change a diagnostic, and no
//! assertion in this file depends on HOW FAST the server answers.

use std::io::{BufReader, Read};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lsp_server::{ErrorCode, Message, Notification, Request, RequestId};
use lsp_types::notification::{Exit, Initialized, Notification as _};
use lsp_types::request::{Initialize, Request as _, Shutdown};
use lsp_types::{ClientCapabilities, InitializeParams, InitializedParams};

/// Bounds the reader thread's frame loop (Power-of-10 rule 2: every loop
/// statically bounded). No test in this file exchanges anywhere near 64
/// messages; a server that free-runs past this count is itself a bug this
/// loop refuses to chase forever.
const MAX_FRAMES: usize = 64;

/// How long the test body waits for ONE reply, or for the child to exit,
/// before declaring a hang. A watchdog, never a performance assertion (see
/// module doc) — it can only turn a hang into a fast, legible failure; no
/// test below asserts on elapsed time.
const RECV_TIMEOUT: Duration = Duration::from_secs(30);

/// Owns the spawned server so a panicking or failing assertion never
/// orphans the process — `Drop` kills and reaps the child unconditionally
/// (plan §5.3).
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl ChildGuard {
    /// Polls (never blocks past one iteration) for the child's exit,
    /// bounded to `RECV_TIMEOUT` in fixed-size steps — the exit-status
    /// analogue of `TestServer::recv`'s message watchdog, sharing the same
    /// carve-out (module doc). `POLL_INTERVAL * MAX_POLLS == RECV_TIMEOUT`,
    /// so the two constants below stay a fixed, statically bounded loop
    /// (Power-of-10 rule 2), not an open-ended wait.
    fn wait_for_exit(&mut self) -> Option<ExitStatus> {
        const POLL_INTERVAL: Duration = Duration::from_millis(50);
        const MAX_POLLS: usize = 600; // 600 * 50ms == 30s == RECV_TIMEOUT
        for _ in 0..MAX_POLLS {
            if let Ok(Some(status)) = self.0.try_wait() {
                return Some(status);
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        None
    }
}

/// A live `bsl-ls` subprocess plus the plumbing to talk framed JSON-RPC to
/// it without the test body ever blocking on the pipe directly.
struct TestServer {
    child: ChildGuard,
    stdin: ChildStdin,
    rx: mpsc::Receiver<Message>,
    stderr: Arc<Mutex<Vec<u8>>>,
}

impl TestServer {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bsl-ls"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn bsl-ls");

        let stdin = child.stdin.take().expect("child stdin was not piped");
        let stdout = child.stdout.take().expect("child stdout was not piped");
        let stderr = child.stderr.take().expect("child stderr was not piped");

        // The reader thread: owns the child's stdout, parses Content-Length
        // frames via `lsp_server::Message::read` (the same framing our own
        // production code uses — no hand-rolled parser to drift from it),
        // and forwards each parsed message over a channel. The test body
        // never blocks on the pipe directly.
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            for _ in 0..MAX_FRAMES {
                match Message::read(&mut reader) {
                    Ok(Some(msg)) => {
                        if tx.send(msg).is_err() {
                            return;
                        }
                    }
                    Ok(None) | Err(_) => return, // pipe closed / malformed frame
                }
            }
        });

        // Drains stderr into a shared buffer so a timeout's failure message
        // can print it (below) — `read_to_end` is ONE library call, not a
        // loop this file writes, so it carries no Power-of-10 rule 2
        // obligation of its own; it unblocks when the pipe closes, which
        // `ChildGuard::drop` forces if nothing else already did.
        let stderr_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        {
            let stderr_buf = Arc::clone(&stderr_buf);
            let mut stderr = stderr;
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let _ = stderr.read_to_end(&mut buf);
                if let Ok(mut guard) = stderr_buf.lock() {
                    *guard = buf;
                }
            });
        }

        TestServer {
            child: ChildGuard(child),
            stdin,
            rx,
            stderr: stderr_buf,
        }
    }

    fn send(&mut self, msg: &Message) {
        msg.write(&mut self.stdin)
            .expect("failed to write to child stdin");
    }

    fn stderr_text(&self) -> String {
        let buf = self.stderr.lock().expect("stderr mutex poisoned");
        String::from_utf8_lossy(&buf).into_owned()
    }

    /// Waits for the next message, failing fast (never hanging past
    /// `RECV_TIMEOUT`) with the child's captured stderr attached to the
    /// failure — "a fast, legible failure instead of a hung CI job" (plan
    /// §5.3).
    fn recv(&self, context: &str) -> Message {
        match self.rx.recv_timeout(RECV_TIMEOUT) {
            Ok(msg) => msg,
            Err(RecvTimeoutError::Timeout) => panic!(
                "server did not answer `{context}` within {RECV_TIMEOUT:?}\n\
                 --- child stderr ---\n{}",
                self.stderr_text()
            ),
            Err(RecvTimeoutError::Disconnected) => panic!(
                "server closed its output before answering `{context}`\n\
                 --- child stderr ---\n{}",
                self.stderr_text()
            ),
        }
    }
}

fn expect_ok_response(msg: Message, id: &RequestId, context: &str) {
    match msg {
        Message::Response(resp) => {
            assert_eq!(&resp.id, id, "{context}: response id mismatch");
            assert!(
                resp.response_result.is_ok(),
                "{context}: expected a success response, got {:?}",
                resp.response_result
            );
        }
        other => panic!("{context}: expected a Response, got {other:?}"),
    }
}

/// Runs `initialize` -> `initialized` through to (and including) draining
/// the `window/logMessage` degradation warning that fires immediately
/// after, since the default `ClientCapabilities` sent below declare no
/// dynamic-registration support for `workspace/didChangeWatchedFiles`
/// (`src/lifecycle.rs`'s `watch_action_for`) — the SAME production
/// sequence a real, minimally-capable client triggers, not test-only
/// scaffolding.
fn handshake(server: &mut TestServer) {
    server.send(
        &Request::new(
            RequestId::from(1),
            Initialize::METHOD.to_owned(),
            InitializeParams {
                capabilities: ClientCapabilities::default(),
                ..Default::default()
            },
        )
        .into(),
    );
    let response = server.recv("initialize");
    expect_ok_response(response, &RequestId::from(1), "initialize");

    server.send(&Notification::new(Initialized::METHOD.to_owned(), InitializedParams {}).into());

    let degrade = server.recv("post-initialized window/logMessage");
    match degrade {
        Message::Notification(n) => assert_eq!(n.method, "window/logMessage"),
        other => panic!("expected window/logMessage, got {other:?}"),
    }
}

#[test]
fn initialize_initialized_shutdown_exit_round_trip() {
    let mut server = TestServer::spawn();
    handshake(&mut server);

    server.send(&Request::new(RequestId::from(2), Shutdown::METHOD.to_owned(), ()).into());
    let response = server.recv("shutdown");
    expect_ok_response(response, &RequestId::from(2), "shutdown");

    server.send(&Notification::new(Exit::METHOD.to_owned(), ()).into());

    let status = server
        .child
        .wait_for_exit()
        .unwrap_or_else(|| panic!("server did not exit within {RECV_TIMEOUT:?} after exit"));
    assert_eq!(status.code(), Some(0));
}

#[test]
fn request_before_initialize_gets_server_not_initialized() {
    let mut server = TestServer::spawn();

    server.send(&Request::new(RequestId::from(42), Shutdown::METHOD.to_owned(), ()).into());
    let response = server.recv("pre-initialize shutdown");
    match response {
        Message::Response(resp) => {
            assert_eq!(resp.id, RequestId::from(42));
            let err = resp
                .response_result
                .expect_err("expected an error response before initialize");
            assert_eq!(err.code, ErrorCode::ServerNotInitialized as i32);
        }
        other => panic!("expected an error Response, got {other:?}"),
    }

    // `Connection::initialize_start` keeps looping past a rejected request
    // (lsp-server's own handshake discipline) — walk it through a real,
    // clean shutdown so the process exits deterministically rather than
    // relying on `ChildGuard`'s kill-on-drop fallback.
    handshake(&mut server);
    server.send(&Request::new(RequestId::from(2), Shutdown::METHOD.to_owned(), ()).into());
    let shutdown_response = server.recv("shutdown");
    expect_ok_response(shutdown_response, &RequestId::from(2), "shutdown");
    server.send(&Notification::new(Exit::METHOD.to_owned(), ()).into());

    let status = server
        .child
        .wait_for_exit()
        .unwrap_or_else(|| panic!("server did not exit within {RECV_TIMEOUT:?} after exit"));
    assert_eq!(status.code(), Some(0));
}

#[test]
fn exit_without_shutdown_exits_uncleanly() {
    let mut server = TestServer::spawn();
    handshake(&mut server);

    server.send(&Notification::new(Exit::METHOD.to_owned(), ()).into());

    let status = server
        .child
        .wait_for_exit()
        .unwrap_or_else(|| panic!("server did not exit within {RECV_TIMEOUT:?} after exit"));
    assert_eq!(status.code(), Some(1));
}
