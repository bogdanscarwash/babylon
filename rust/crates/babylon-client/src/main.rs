//! `babylon-client` — the Program 28 client. Amendment AF names this crate
//! the v1.0 client: a standalone Bevy executable, engine crates linked
//! in-process, no `PyO3` in the play path.
//!
//! The observer embeds licensed Source Sans 3 for reading and Barlow
//! Condensed for headings. Exact values retain Bevy's bundled Fira Mono.
//!
//! B1 Task 6 wires in `map::MapPlugin`, which spawns the county fill and
//! border meshes at Startup. B1 Task 7 folds the camera into `MapPlugin`
//! too (`map::camera::spawn_camera`, a bounded pan/zoom `PanCamera` sized
//! from the atlas) — B0's bare `spawn_camera` is dropped here so exactly
//! one camera exists.
//!
//! B2 Task 14 wires in `loop_ui::TickLoopPlugin` — Space advances the tick,
//! a HUD readout shows the counter and the deterministic hash. B0's
//! `log_engine_link` Startup system is retired here: `EngineSession::start`
//! (`TickLoopPlugin`'s own Startup system) now IS the engine link, and it
//! panics loudly on failure exactly as `log_engine_link` did —
//! `engine_link::engine_link_probe` itself is untouched (B0's own pinned
//! test, `tests/engine_link.rs`, still exercises it directly).
//!
//! PER-23 Slice 3 (ADR249 R10-R11) adds the headless dossier CLI:
//! `main` is now a pure dispatch shim — parse the hand-rolled CLI
//! (`cli::parse`), build the app for the requested mode (`app::build_app`),
//! and map the Bevy `AppExit` onto the process exit code. All windowed
//! construction lives in `app.rs`; all parsing lives in `cli.rs`.
//!
//! Logging rides Bevy's own `tracing` stack (Director-approved switch
//! 2026-08-11, #503 item 7, replacing B2 Task 16's `log4rs` sink):
//! `LogPlugin` is the ONE global subscriber, the rolling file sink
//! attaches as its `custom_layer`, and the stderr formatter is capped at
//! INFO so the file's DEBUG lane stays off the terminal. The filter
//! grants this crate DEBUG on top of Bevy's default noise floor.

use babylon_client::app::{build_app, AppMode};
use babylon_client::cli::{self, CliRequest};
use bevy::app::AppExit;

fn main() {
    let request = cli::parse(std::env::args_os().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let mode = match request {
        CliRequest::Help(topic) => {
            print!("{}", cli::render_help(topic));
            return;
        }
        CliRequest::Windowed { campaign_id } => AppMode::Windowed { campaign_id },
        CliRequest::Headless {
            command,
            campaign_id,
        } => AppMode::Headless {
            command,
            campaign_id,
        },
    };
    let exit = build_app(mode).run();
    std::process::exit(match exit {
        AppExit::Success => 0,
        AppExit::Error(code) => i32::from(code.get()),
    });
}
