//! PER-23 Slice 3 app construction (ADR249 R11): [`build_app`] assembles
//! the Bevy `App` for exactly two modes — the windowed administrative
//! viewer and the headless dossier CLI. Extracting this from `main` lets
//! both modes share one construction site, keeps `main` a pure dispatch
//! shim, and gives tests one function to exercise.

use bevy::log::LogPlugin;
use bevy::prelude::*;

use crate::cli::CliCommand;
use crate::dossier::{run_headless_command, HeadlessInvocation};
use crate::observer::ObserverSession;
use crate::observer_focus::ObserverFocusPlugin;
use crate::observer_io::ObserverIoPlugin;
use crate::observer_ui::ObserverShellPlugin;
use crate::{logging, map, session_log, ui, visual_assets};

/// Which executable shape [`build_app`] assembles.
#[derive(Clone, Debug)]
pub enum AppMode {
    /// The windowed observer of one durable campaign.
    Windowed {
        /// The canonical campaign identity.
        campaign_id: babylon_persistence::CampaignId,
    },
    /// One headless dossier command: JSONL on stdout, logs on stderr,
    /// process exit after the first update.
    Headless {
        /// The parsed dossier command.
        command: CliCommand,
        /// The canonical campaign identity.
        campaign_id: babylon_persistence::CampaignId,
    },
}

/// Assemble the Bevy `App` for one mode. Windowed keeps the pinned
/// `DefaultPlugins` estate (file + stderr logging, window title); headless
/// rides `MinimalPlugins` plus the pinned `LogPlugin` alone — no window,
/// no render, no visual plugins — and runs exactly one Startup system.
pub fn build_app(mode: AppMode) -> App {
    let mut app = App::new();
    match mode {
        AppMode::Windowed { campaign_id } => {
            app.add_plugins(
                DefaultPlugins
                    .set(LogPlugin {
                        filter: format!(
                            "{},babylon_client=debug,session=debug",
                            bevy::log::DEFAULT_FILTER
                        ),
                        custom_layer: logging::file_layer,
                        fmt_layer: logging::stderr_fmt_layer,
                        ..default()
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Babylon — The Fall of America".into(),
                            resolution: (1366, 768).into(),
                            ..default()
                        }),
                        ..default()
                    }),
            );
            // Nothing listens before LogPlugin::build installs the
            // subscriber, so the startup line stays after add_plugins. The
            // line stays static: no parse-channel value is logged.
            log::info!("babylon-client starting (durable observer)");
            app.add_plugins(visual_assets::VisualAssetsPlugin)
                .add_plugins(map::MapPlugin)
                .add_plugins(ObserverFocusPlugin)
                .add_plugins(ObserverShellPlugin)
                .add_plugins(crate::observer_map3d::ObserverMap3dPlugin)
                .add_plugins(ObserverIoPlugin)
                .add_plugins(crate::production::ProductionPlugin)
                .add_plugins(crate::observer_audio::ObserverAudioPlugin)
                .add_plugins(crate::campaign_browser::CampaignBrowserPlugin)
                .add_plugins(crate::observer_history::ObserverHistoryPlugin)
                .add_plugins(crate::observer_performance::ObserverPerformancePlugin)
                // PER-23 Slice 4 (ADR249 R9): the county dossier card — the
                // first Gameplay-role surface. Its resource family must exist
                // identically in windowed and headless compositions, so it
                // rides its own plugin rather than TickLoopPlugin's wiring.
                .add_plugins(ui::dossier_card::DossierCardPlugin)
                // Session observers ride the same resource family (every
                // observer tolerates its absence), so the session log sees
                // exactly what the card renderer saw.
                .add_plugins(session_log::SessionLogPlugin)
                .insert_resource(crate::production::PrimaryView::Map)
                .insert_resource(ObserverSession::new(campaign_id))
                .insert_resource(ui::dossier_card::DossierCampaignId(campaign_id))
                .insert_resource(ClearColor(crate::observer_theme::INK));
        }
        AppMode::Headless {
            command,
            campaign_id,
        } => {
            app.add_plugins(MinimalPlugins)
                .add_plugins(LogPlugin {
                    filter: format!(
                        "{},babylon_client=debug,session=debug",
                        bevy::log::DEFAULT_FILTER
                    ),
                    custom_layer: logging::file_layer,
                    fmt_layer: logging::stderr_fmt_layer,
                    ..default()
                })
                .insert_resource(HeadlessInvocation::new(command, campaign_id))
                .add_systems(Startup, run_headless_command);
        }
    }
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::CliCommand;
    use uuid::Uuid;

    #[test]
    fn headless_mode_builds_a_minimal_app_that_exits_after_startup() {
        // Isolate the test from any ambient reader DSN: a live DSN would
        // make the command succeed and defeat the exit-code assertion. The
        // guard serializes against other env-mutating tests and restores
        // the ambient value on drop.
        let env = crate::test_support::EnvVarGuard::lock(babylon_persistence::READER_DSN_ENV_V1);
        env.remove();
        let mut app = build_app(AppMode::Headless {
            command: CliCommand::TickStatus,
            campaign_id: babylon_persistence::CampaignId::from_uuid(Uuid::nil()),
        });
        assert!(!app.is_plugin_added::<ObserverFocusPlugin>());
        assert!(app
            .world()
            .get_resource::<crate::observer_focus::ObserverFocusPolicy>()
            .is_none());
        assert!(
            app.world().get_resource::<HeadlessInvocation>().is_some(),
            "the invocation resource is installed before Startup"
        );
        app.update();
        // Without a reader DSN the command refuses with exit code 2, and
        // the AppExit message must be queued: the app must not hang.
        let mut messages = app.world_mut().resource_mut::<Messages<AppExit>>();
        let exit = messages.drain().next().expect("the headless run exits");
        assert!(
            matches!(exit, AppExit::Error(code) if code.get() == 2),
            "a refused command exits non-zero, got {exit:?}"
        );
    }
}
