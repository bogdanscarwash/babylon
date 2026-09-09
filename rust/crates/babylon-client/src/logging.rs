//! File logging through Bevy's own `tracing` stack (Director directive
//! 2026-07-28 for the sink location + rotation; Director-approved switch
//! 2026-08-11, #503 item 7: `LogPlugin` is the ONE global subscriber and
//! the file sink attaches to it as a [`LogPlugin::custom_layer`], so
//! Bevy's engine diagnostics land in the file alongside client events).
//!
//! The B2 `log4rs` sink this replaces raced `LogPlugin` for the global
//! logger slot — whoever initialized second failed (`bevy_log`'s
//! `LogTracer::init()` is the losing line the Director's eyes-on session
//! surfaced). With `log4rs` gone, `LogPlugin` installs `LogTracer`
//! itself, so this crate's `log::debug!`/`log::info!` call sites bridge
//! into `tracing` unchanged.
//!
//! **No wall-clock in client source.** Timestamps come from the fmt
//! layer's own timer inside the subscriber, exactly as `log4rs`'s pattern
//! encoder did before.
//!
//! [`LogPlugin::custom_layer`]: bevy::log::LogPlugin

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use bevy::app::App;
use bevy::log::tracing_subscriber;
use bevy::log::{BoxedFmtLayer, BoxedLayer};

const LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
const LOG_ARCHIVES: u32 = 5;
const LOG_FILE: &str = "babylon-client.log";

/// `$XDG_DATA_HOME/babylon/logs` else `~/.local/share/babylon/logs` —
/// mirrors `src/babylon/config/paths.py::player_data_dir()` /
/// `src/babylon/config/base.py::LOG_DIR` exactly, transcribed rather than
/// re-derived (no `PyO3` in the play path, Amendment AF, so this cannot call
/// the Python function — it reproduces its two-line rule instead).
#[must_use]
pub fn log_dir() -> PathBuf {
    log_dir_from(
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

/// The pure resolution rule behind [`log_dir`], with both environment
/// inputs injected — the test exercises this directly, so it never mutates
/// process-global env vars (cargo runs tests in parallel; a `set_var` in
/// one test races every other test's threads).
fn log_dir_from(xdg_data_home: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
    let base = xdg_data_home
        .unwrap_or_else(|| home.expect("HOME must be set").join(".local").join("share"));
    base.join("babylon").join("logs")
}

/// A size-rotating file sink: `babylon-client.log` live, archives shifted
/// to `babylon-client.{0..N-1}.log` (0 newest) when the live file would
/// exceed the byte cap — the same naming and window the `log4rs`
/// `FixedWindowRoller` used, so archives from before the switch stay
/// coherent. `Clone` is cheap (shared state behind a mutex); the fmt
/// layer's `MakeWriter` hands a clone to every event.
#[derive(Clone)]
pub struct RotatingSink {
    inner: Arc<Mutex<SinkState>>,
}

struct SinkState {
    dir: PathBuf,
    max_bytes: u64,
    archives: u32,
    /// `None` only after a failed rotation — every later write then fails
    /// loudly instead of silently dropping records.
    file: Option<File>,
    written: u64,
}

impl RotatingSink {
    /// Open (append) the live log file under `dir`, rotating at
    /// `max_bytes` and keeping `archives` shifted archive files.
    ///
    /// # Errors
    /// `archives` is zero, or the directory or live file cannot be
    /// created/opened, or its size cannot be read.
    pub fn open(dir: &Path, max_bytes: u64, archives: u32) -> Result<Self, String> {
        if archives == 0 {
            return Err("log rotation needs at least one archive slot".to_string());
        }
        std::fs::create_dir_all(dir).map_err(|e| format!("log dir {}: {e}", dir.display()))?;
        let live = dir.join(LOG_FILE);
        let file = open_live(&live)?;
        let written = file
            .metadata()
            .map_err(|e| format!("log file size {}: {e}", live.display()))?
            .len();
        Ok(Self {
            inner: Arc::new(Mutex::new(SinkState {
                dir: dir.to_path_buf(),
                max_bytes,
                archives,
                file: Some(file),
                written,
            })),
        })
    }
}

fn open_live(live: &Path) -> Result<File, String> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(live)
        .map_err(|e| format!("log file {}: {e}", live.display()))
}

fn archive_path(dir: &Path, index: u32) -> PathBuf {
    dir.join(format!("babylon-client.{index}.log"))
}

/// Shift the fixed archive window one slot (oldest dropped, live becomes
/// archive 0) and reopen a fresh live file — the same window the `log4rs`
/// `FixedWindowRoller` kept, so pre-switch archives shift coherently.
fn rotate(state: &mut SinkState) -> io::Result<()> {
    let oldest = archive_path(&state.dir, state.archives - 1);
    if oldest.exists() {
        std::fs::remove_file(&oldest)?;
    }
    for index in (0..state.archives - 1).rev() {
        let from = archive_path(&state.dir, index);
        if from.exists() {
            std::fs::rename(&from, archive_path(&state.dir, index + 1))?;
        }
    }
    let live = state.dir.join(LOG_FILE);
    // Drop the live handle BEFORE the rename: Unix renames an open file
    // happily, Windows refuses (sharing violation) — Amendment AA makes
    // Windows binding post-1.0, so the order is load-bearing, not style.
    state.file = None;
    std::fs::rename(&live, archive_path(&state.dir, 0))?;
    state.file = Some(open_live(&live).map_err(io::Error::other)?);
    state.written = 0;
    Ok(())
}

impl io::Write for RotatingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut state = self
            .inner
            .lock()
            .map_err(|e| io::Error::other(format!("log sink mutex poisoned: {e}")))?;
        let incoming = buf.len() as u64;
        if state.written > 0 && state.written + incoming > state.max_bytes {
            rotate(&mut state)?;
        }
        let file = state.file.as_mut().ok_or_else(|| {
            io::Error::other("log sink has no live file (a prior rotation failed)")
        })?;
        file.write_all(buf)?;
        state.written += incoming;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|e| io::Error::other(format!("log sink mutex poisoned: {e}")))?;
        match state.file.as_mut() {
            Some(file) => file.flush(),
            None => Ok(()),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for RotatingSink {
    type Writer = RotatingSink;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// The file lane against an explicit directory — [`file_layer`] minus the
/// environment resolution, so tests drive it at a temp dir and exercise
/// the REAL production layer (never a copied pipeline).
fn file_layer_in(dir: &Path) -> Option<BoxedLayer> {
    match RotatingSink::open(dir, LOG_MAX_BYTES, LOG_ARCHIVES) {
        Ok(sink) => {
            install_panic_hook();
            let layer = tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer(sink);
            Some(Box::new(layer))
        }
        Err(e) => {
            eprintln!("warning: client file logging did not start: {e}");
            None
        }
    }
}

/// [`bevy::log::LogPlugin::custom_layer`] — the rolling-file lane. On any
/// sink failure the game still runs: a stderr warning, no layer.
#[must_use]
pub fn file_layer(_app: &mut App) -> Option<BoxedLayer> {
    file_layer_in(&log_dir())
}

/// [`bevy::log::LogPlugin::fmt_layer`] — the stderr lane, capped at INFO
/// so the file's DEBUG traffic never spams the terminal (the pre-switch
/// behavior: `log4rs` logged DEBUG to file only).
#[must_use]
pub fn stderr_fmt_layer(_app: &mut App) -> Option<BoxedFmtLayer> {
    use bevy::log::tracing_subscriber::Layer as _;
    let layer = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr)
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);
    Some(Box::new(layer))
}

fn install_panic_hook() {
    static HOOK: OnceLock<()> = OnceLock::new();
    HOOK.get_or_init(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            log::error!(target: "panic", "client panic: {info}");
            previous(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "babylon-client-logtest-{tag}-{}",
            std::process::id()
        ));
        std::fs::remove_dir_all(&dir).ok();
        std::fs::create_dir_all(&dir).expect("test log dir");
        dir
    }

    #[test]
    fn log_dir_honors_xdg_data_home() {
        // Injected inputs, no env mutation — parallel-safe by construction
        // (the deleted TUI module's test set XDG_DATA_HOME process-globally
        // and leaned on a single-threaded assumption; not transcribed).
        let xdg = log_dir_from(Some(PathBuf::from("/tmp/xdg-probe")), None);
        assert_eq!(xdg, PathBuf::from("/tmp/xdg-probe/babylon/logs"));
        let fallback = log_dir_from(None, Some(PathBuf::from("/home/probe")));
        assert_eq!(
            fallback,
            PathBuf::from("/home/probe/.local/share/babylon/logs")
        );
    }

    #[test]
    fn the_sink_appends_across_instances_never_truncates() {
        let dir = temp_dir("append");
        {
            let mut sink = RotatingSink::open(&dir, 1024, 2).expect("first open");
            sink.write_all(b"first line\n").expect("write");
        }
        let mut sink = RotatingSink::open(&dir, 1024, 2).expect("re-open");
        sink.write_all(b"second line\n").expect("write");
        let live = std::fs::read_to_string(dir.join(LOG_FILE)).expect("live file");
        assert!(live.contains("first line"), "re-open truncated the log");
        assert!(live.contains("second line"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_sink_rotates_at_the_cap_and_shifts_archives_zero_newest() {
        let dir = temp_dir("rotate");
        let mut sink = RotatingSink::open(&dir, 24, 2).expect("open");
        sink.write_all(b"aaaaaaaaaaaaaaaaaaaa\n").expect("gen a"); // 21 bytes
        sink.write_all(b"bbbbbbbbbbbbbbbbbbbb\n").expect("gen b"); // would exceed 24
        sink.write_all(b"cccccccccccccccccccc\n").expect("gen c");
        let live = std::fs::read_to_string(dir.join(LOG_FILE)).expect("live");
        let arch0 = std::fs::read_to_string(dir.join("babylon-client.0.log")).expect("archive 0");
        let arch1 = std::fs::read_to_string(dir.join("babylon-client.1.log")).expect("archive 1");
        assert!(live.contains("ccc"), "live holds the newest generation");
        assert!(
            arch0.contains("bbb"),
            "archive 0 is the previous generation"
        );
        assert!(arch1.contains("aaa"), "archive 1 is the oldest kept");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_three_deep_shift_preserves_generation_order() {
        // A 2-archive window cannot tell a reversed shift loop from a
        // forward one (both collapse to a single rename); three archives
        // can — a forward loop clobbers the middle generation.
        let dir = temp_dir("three-deep");
        let mut sink = RotatingSink::open(&dir, 8, 3).expect("open");
        for gen in [b"aaaaaaaa\n", b"bbbbbbbb\n", b"cccccccc\n", b"dddddddd\n"] {
            sink.write_all(gen).expect("write generation");
        }
        let arch0 = std::fs::read_to_string(dir.join("babylon-client.0.log")).expect("archive 0");
        let arch1 = std::fs::read_to_string(dir.join("babylon-client.1.log")).expect("archive 1");
        let arch2 = std::fs::read_to_string(dir.join("babylon-client.2.log")).expect("archive 2");
        assert!(arch0.contains("ccc"), "archive 0 is the newest archived");
        assert!(arch1.contains("bbb"), "archive 1 is the middle generation");
        assert!(arch2.contains("aaa"), "archive 2 is the oldest kept");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_single_write_larger_than_the_cap_lands_without_rotating() {
        // An empty live file must never rotate: the oversized record lands,
        // and rotation waits for the NEXT write (log4rs size-trigger
        // semantics — the cap bounds the file, not the record).
        let dir = temp_dir("oversize");
        let mut sink = RotatingSink::open(&dir, 8, 2).expect("open");
        sink.write_all(b"one oversized record\n").expect("write");
        let live = std::fs::read_to_string(dir.join(LOG_FILE)).expect("live");
        assert!(live.contains("one oversized record"));
        assert!(
            !dir.join("babylon-client.0.log").exists(),
            "an empty live file must not be archived"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rotation_drops_the_oldest_archive_beyond_the_window() {
        let dir = temp_dir("window");
        let mut sink = RotatingSink::open(&dir, 8, 2).expect("open");
        for gen in [b"aaaaaaaa\n", b"bbbbbbbb\n", b"cccccccc\n", b"dddddddd\n"] {
            sink.write_all(gen).expect("write generation");
        }
        assert!(
            !dir.join("babylon-client.2.log").exists(),
            "window is 2 archives; a third index must never appear"
        );
        let arch1 = std::fs::read_to_string(dir.join("babylon-client.1.log")).expect("archive 1");
        assert!(
            arch1.contains("bbb"),
            "the aaa generation fell off the window"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_production_file_layer_writes_events_to_the_live_file() {
        use bevy::log::tracing_subscriber::layer::SubscriberExt as _;
        // The REAL layer `file_layer` returns (via `file_layer_in`), driven
        // through a registry exactly as `LogPlugin` composes it — not a
        // rebuilt copy of the pipeline (the #504 test-vacuity lesson).
        let dir = temp_dir("layer");
        let layer = file_layer_in(&dir).expect("layer builds against a writable dir");
        let subscriber = bevy::log::tracing_subscriber::registry().with(layer);
        bevy::log::tracing::subscriber::with_default(subscriber, || {
            bevy::log::info!(target: "probe", "file lane probe line");
        });
        let live = std::fs::read_to_string(dir.join(LOG_FILE)).expect("live file");
        assert!(live.contains("file lane probe line"));
        assert!(live.contains("probe"), "the event target is encoded");
        assert!(
            !live.contains("\u{1b}["),
            "the file lane must carry no ANSI escapes"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_bad_directory_yields_no_layer_instead_of_a_panic() {
        assert!(
            file_layer_in(Path::new("/proc/definitely/not/writable")).is_none(),
            "an unwritable dir degrades to no file lane, the game still runs"
        );
    }

    #[test]
    fn the_stderr_lane_exists() {
        let mut app = App::new();
        assert!(
            stderr_fmt_layer(&mut app).is_some(),
            "the INFO-capped stderr formatter replaces LogPlugin's default"
        );
    }
}
