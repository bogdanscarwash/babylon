//! Bounded runtime control pipes and asynchronous role-scoped observations.

use std::io::{BufRead, Read, Write};
use std::sync::{mpsc, Mutex};

use babylon_persistence::{
    observer_reader::ObserverEconomyReader, observer_reader::ObserverEconomySnapshot,
    observer_reader::ObserverVisibility, runtime_session::RuntimeSessionPreset,
    runtime_session::RuntimeSessionRequest, runtime_session::RuntimeSessionResponse,
    runtime_session::RuntimeSessionScope, runtime_session::RuntimeSessionTail,
    runtime_session::RuntimeSessionTarget, runtime_session::RUNTIME_SESSION_MAX_LINE_BYTES,
    runtime_session::RUNTIME_SESSION_PROTOCOL_VERSION,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};

use crate::observer::{ObservationContext, ObserverSession, Perspective, SessionPhase};
use crate::observer_controls::{availability, ControlAvailability};
use crate::observer_ui::{ObserverCommand, ObserverFeedback, ObserverFrame, ObserverUiState};
use crate::ui::dossier_card::{
    ActiveCountyDossier, DossierCampaignId, DossierFetchState, DossierPageView, DossierRefresh,
};

#[cfg(test)]
#[path = "observer_io/lifecycle_tests.rs"]
mod lifecycle_tests;

pub(crate) const LAUNCHER_REQUIRED: &str =
    "This window has no launcher connection. Close it and start Babylon through its launcher.";

// Allows the bounded 120-second storage statement to finish before recovery cleanup.
const SHUTDOWN_TIMEOUT_SECS: f64 = 150.0;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObserverSet {
    Input,
    Receive,
    Install,
    Paint,
}

#[derive(Resource)]
pub(crate) struct RuntimePipe {
    requests: mpsc::SyncSender<RuntimeSessionRequest>,
    responses: Mutex<mpsc::Receiver<Result<RuntimeSessionResponse, String>>>,
}

#[cfg(test)]
impl RuntimePipe {
    pub(crate) fn detached_fixture() -> Self {
        let (requests, _) = mpsc::sync_channel(1);
        let (_, responses) = mpsc::channel();
        Self {
            requests,
            responses: Mutex::new(responses),
        }
    }
}

#[derive(Resource, Default)]
struct PendingObservation(
    Option<(
        ObservationContext,
        Task<Result<ObserverEconomySnapshot, String>>,
    )>,
);

#[derive(Resource, Default)]
struct PlaybackClock {
    elapsed: f64,
}

#[derive(Resource, Default)]
struct ShutdownProgress {
    started_at: Option<f64>,
    stop_sent: bool,
    exit_sent: bool,
}

#[derive(Resource)]
struct ContinuationPreference(std::path::PathBuf);

fn start_pipe(mut commands: Commands, mut state: ResMut<ObserverSession>) {
    if std::env::var("BABYLON_SESSION_STDIO").as_deref() != Ok("1") {
        state.fail("Open this campaign with mise run play to connect its durable runtime.".into());
        return;
    }
    match crate::campaign_browser::preference_path() {
        Ok(path) => {
            commands.insert_resource(ContinuationPreference(path));
        }
        Err(error) => {
            state.fail(error);
            return;
        }
    }
    let (request_tx, request_rx) = mpsc::sync_channel::<RuntimeSessionRequest>(1);
    let (response_tx, response_rx) =
        mpsc::sync_channel::<Result<RuntimeSessionResponse, String>>(8);
    let errors = response_tx.clone();
    let writer = std::thread::Builder::new()
        .name("observer-control-writer".into())
        .spawn(move || {
            let mut output = std::io::stdout().lock();
            while let Ok(request) = request_rx.recv() {
                let result = serde_json::to_vec(&request)
                    .map_err(|error| error.to_string())
                    .and_then(|mut bytes| {
                        if bytes.len() >= RUNTIME_SESSION_MAX_LINE_BYTES {
                            return Err("Runtime request exceeds protocol bound".into());
                        }
                        bytes.push(b'\n');
                        output
                            .write_all(&bytes)
                            .and_then(|()| output.flush())
                            .map_err(|error| error.to_string())
                    });
                if let Err(error) = result {
                    let _ = errors.send(Err(error));
                    break;
                }
            }
        });
    if let Err(error) = writer {
        state.fail(format!("Cannot start control writer: {error}"));
        return;
    }
    let reader = std::thread::Builder::new()
        .name("observer-control-reader".into())
        .spawn(move || {
            let mut input = std::io::stdin().lock();
            loop {
                let mut line = Vec::new();
                let result = (&mut input)
                    .take((RUNTIME_SESSION_MAX_LINE_BYTES + 1) as u64)
                    .read_until(b'\n', &mut line);
                match result {
                    Ok(0) => {
                        let _ = response_tx.send(Err(
                            "Runtime disconnected; reopen to reconcile the committed campaign."
                                .into(),
                        ));
                        break;
                    }
                    Ok(size) if size <= RUNTIME_SESSION_MAX_LINE_BYTES && line.ends_with(b"\n") => {
                        let response = serde_json::from_slice(&line)
                            .map_err(|error| format!("Invalid runtime response: {error}"));
                        if response_tx.send(response).is_err() {
                            break;
                        }
                    }
                    Ok(_) => {
                        let _ =
                            response_tx.send(Err("Runtime response exceeds protocol bound".into()));
                        break;
                    }
                    Err(error) => {
                        let _ = response_tx.send(Err(error.to_string()));
                        break;
                    }
                }
            }
        });
    if let Err(error) = reader {
        state.fail(format!("Cannot start control reader: {error}"));
        return;
    }
    commands.insert_resource(RuntimePipe {
        requests: request_tx,
        responses: Mutex::new(response_rx),
    });
}

fn send_advance(pipe: &RuntimePipe, state: &mut ObserverSession) {
    let Some(request_id) = state.begin_advance() else {
        return;
    };
    let request = RuntimeSessionRequest::Advance {
        protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
        scope: state
            .runtime_scope()
            .expect("admitted runtime scope")
            .clone(),
        request_id,
        expected_tail: RuntimeSessionTail {
            resolve_tick: state.durable_tick,
            tick_content_hash: state.content_hash.clone(),
        },
    };
    if let Err(error) = pipe.requests.try_send(request) {
        state.fail(format!("Cannot request next period: {error}"));
    }
}

fn next_response(
    receiver: &mpsc::Receiver<Result<RuntimeSessionResponse, String>>,
) -> Result<Option<RuntimeSessionResponse>, String> {
    match receiver.try_recv() {
        Ok(response) => response.map(Some),
        Err(mpsc::TryRecvError::Disconnected) => {
            Err("Runtime disconnected; reopen to reconcile committed progress.".into())
        }
        Err(mpsc::TryRecvError::Empty) => Ok(None),
    }
}

#[derive(SystemParam)]
struct CampaignReset<'w> {
    preference: Option<Res<'w, ContinuationPreference>>,
    frame: Option<ResMut<'w, ObserverFrame>>,
    pending: Option<ResMut<'w, PendingObservation>>,
    campaign: Option<ResMut<'w, DossierCampaignId>>,
    dossier: Option<ResMut<'w, ActiveCountyDossier>>,
    fetch: Option<ResMut<'w, DossierFetchState>>,
    view: Option<ResMut<'w, DossierPageView>>,
    ui: Option<ResMut<'w, ObserverUiState>>,
}

impl CampaignReset<'_> {
    fn clear(&mut self, state: &ObserverSession) {
        if let Some(frame) = &mut self.frame {
            frame.0 = None;
        }
        if let Some(pending) = &mut self.pending {
            pending.0 = None;
        }
        if let Some(campaign) = &mut self.campaign {
            campaign.0 = state.campaign;
        }
        if let Some(dossier) = &mut self.dossier {
            dossier.0 = None;
        }
        if let Some(fetch) = &mut self.fetch {
            **fetch = DossierFetchState::WaitingForObservation;
        }
        if let Some(view) = &mut self.view {
            **view = DossierPageView::Card;
        }
        if let Some(ui) = &mut self.ui {
            ui.comparison_open = false;
        }
    }
}

fn receive(
    pipe: Option<Res<RuntimePipe>>,
    mut state: ResMut<ObserverSession>,
    mut refresh: ResMut<DossierRefresh>,
    mut reset: CampaignReset,
) {
    if state.phase == SessionPhase::Closed || state.runtime_disconnected() {
        return;
    }
    let Some(pipe) = pipe else {
        return;
    };
    let Ok(receiver) = pipe.responses.lock() else {
        state.disconnect("Runtime response lock failed".into());
        return;
    };
    for _ in 0..8 {
        let response = match next_response(&receiver) {
            Ok(Some(response)) => response,
            Ok(None) => break,
            Err(error) => {
                state.disconnect(error);
                break;
            }
        };
        if let Err(error) = apply_response(response, &mut state, &mut refresh, &mut reset) {
            state.disconnect(error);
            break;
        }
        if state.phase == SessionPhase::Closed {
            break;
        }
    }
}

fn response_scope(response: &RuntimeSessionResponse) -> &RuntimeSessionScope {
    match response {
        RuntimeSessionResponse::Hello { scope, .. }
        | RuntimeSessionResponse::Switching { scope, .. }
        | RuntimeSessionResponse::Ready { scope, .. }
        | RuntimeSessionResponse::Committed { scope, .. }
        | RuntimeSessionResponse::ArchiveProgress { scope, .. }
        | RuntimeSessionResponse::Error { scope, .. }
        | RuntimeSessionResponse::Stopped { scope, .. } => scope,
    }
}

fn admits_response_scope(
    response: &RuntimeSessionResponse,
    state: &ObserverSession,
) -> Result<bool, String> {
    let scope = response_scope(response);
    if let Some(current) = state.runtime_scope() {
        if scope.epoch < current.epoch && !matches!(response, RuntimeSessionResponse::Hello { .. })
        {
            log::debug!("Discarded an earlier runtime lifecycle response");
            return Ok(false);
        }
        if !matches!(response, RuntimeSessionResponse::Switching { .. }) && scope != current {
            return Err("Runtime response lifecycle identity mismatch".into());
        }
    } else if !matches!(response, RuntimeSessionResponse::Hello { .. }) {
        return Err("Runtime did not begin with Hello".into());
    }
    Ok(true)
}

fn apply_response(
    response: RuntimeSessionResponse,
    state: &mut ObserverSession,
    refresh: &mut DossierRefresh,
    reset: &mut CampaignReset,
) -> Result<(), String> {
    if !admits_response_scope(&response, state)? {
        return Ok(());
    }
    match response {
        RuntimeSessionResponse::Hello {
            protocol_version,
            scope,
        } => {
            if protocol_version != RUNTIME_SESSION_PROTOCOL_VERSION {
                return Err("Runtime protocol version mismatch".into());
            }
            state.hello(scope)?;
        }
        RuntimeSessionResponse::Switching {
            request_id,
            previous_scope,
            scope,
        } => {
            state.switching(request_id, &previous_scope, scope)?;
            reset.clear(state);
            refresh.bump();
        }
        RuntimeSessionResponse::Ready {
            request_id,
            foundation_digest,
            tail,
            ..
        } => {
            state.admitted(request_id, foundation_digest, tail)?;
            refresh.bump();
            if let Some(preference) = &reset.preference {
                if let Err(error) = crate::campaign_browser::write_preference(
                    &preference.0,
                    state.campaign,
                    state.generation,
                ) {
                    log::warn!(
                        "Campaign opened, but its continuation preference could not be saved: {error}"
                    );
                }
            }
        }
        RuntimeSessionResponse::Committed {
            request_id, tail, ..
        } => {
            if !state.acknowledge(request_id, tail.resolve_tick, tail.tick_content_hash) {
                return Err("Unexpected committed acknowledgement; reopen the campaign.".into());
            }
            refresh.bump();
        }
        RuntimeSessionResponse::ArchiveProgress {
            durable_tick,
            verified_tick,
            ..
        } => {
            if state.foundation_digest.is_none()
                || durable_tick != state.durable_tick
                || verified_tick > durable_tick
                || verified_tick < state.archive_verified_tick
            {
                return Err("Archive progress did not match the acknowledged campaign tail".into());
            }
            if state.archive_verified_tick != verified_tick {
                state.archive_verified_tick = verified_tick;
            }
            refresh.bump();
        }
        RuntimeSessionResponse::Error {
            request_id,
            code,
            tail,
            ..
        } => {
            let complete = code
                == babylon_persistence::runtime_session::RuntimeSessionErrorCode::HorizonComplete
                && tail.as_ref().is_some_and(|tail| {
                    tail.resolve_tick == state.durable_tick
                        && tail.tick_content_hash == state.content_hash
                });
            log::warn!("Runtime campaign request was refused: {code}");
            if !state.refuse_request(request_id, code) {
                return Err("Runtime refusal did not match an outstanding request".into());
            }
            if complete {
                state.complete();
            }
        }
        RuntimeSessionResponse::Stopped { request_id, .. } => {
            if !state.stopped(request_id) {
                return Err("Unexpected shutdown acknowledgement; reopen the campaign.".into());
            }
        }
    }
    Ok(())
}

#[derive(SystemParam)]
struct CommandContext<'w> {
    state: ResMut<'w, ObserverSession>,
    ui: ResMut<'w, ObserverUiState>,
    pipe: Option<Res<'w, RuntimePipe>>,
    frame: Res<'w, ObserverFrame>,
    refresh: ResMut<'w, DossierRefresh>,
    ui_scale: ResMut<'w, UiScale>,
    audio: ResMut<'w, crate::observer_audio::ObserverAudioSettings>,
    feedback: ResMut<'w, ObserverFeedback>,
    time: Res<'w, Time>,
}

fn handle_commands(mut commands: MessageReader<ObserverCommand>, mut context: CommandContext) {
    for command in commands.read().copied() {
        if context.ui.splash_visible {
            if command != ObserverCommand::Quit {
                continue;
            }
        } else if context.ui.comparison_open {
            continue;
        }
        if let ControlAvailability::Disabled(reason) = availability(command, &context.state) {
            let now = context.time.elapsed_secs_f64();
            context.feedback.reject(reason, now);
            continue;
        }
        context.feedback.message = None;
        apply_command(command, &mut context);
    }
}

fn apply_command(command: ObserverCommand, context: &mut CommandContext) {
    let CommandContext {
        state,
        ui,
        pipe,
        refresh,
        feedback,
        time,
        ..
    } = context;
    match command {
        ObserverCommand::Quit => {
            state.pause_playback();
            state.quit_requested = true;
            ui.menu_open = true;
            ui.disclosure = None;
        }
        ObserverCommand::TogglePlay => {
            if state.playing {
                state.pause_playback();
            } else if !state.start_playback() {
                feedback.reject(
                    "Cannot start playback; reopen to reconcile progress.",
                    time.elapsed_secs_f64(),
                );
            }
        }
        ObserverCommand::Step => {
            state.pause_playback();
            if let Some(pipe) = pipe {
                send_advance(pipe, state);
            } else {
                feedback.reject(
                    "No campaign connection. Reopen the campaign from Menu.",
                    time.elapsed_secs_f64(),
                );
            }
        }
        ObserverCommand::Speed => {
            state.periods_per_second = match state.periods_per_second {
                1.0 => 2.0,
                2.0 => 5.0,
                _ => 1.0,
            }
        }
        ObserverCommand::Perspective => {
            let perspective = match state.perspective {
                Perspective::FullObserver => Perspective::PlayerKnowledge,
                Perspective::PlayerKnowledge => Perspective::FullObserver,
            };
            state.set_perspective(perspective);
            ui.disclosure = None;
            ui.evidence_open = false;
            refresh.bump();
        }
        ObserverCommand::PreviousPeriod | ObserverCommand::NextPeriod | ObserverCommand::Live => {
            let tick = match command {
                ObserverCommand::PreviousPeriod => state.viewed_tick.saturating_sub(1),
                ObserverCommand::NextPeriod => state.viewed_tick.saturating_add(1),
                _ => state.durable_tick,
            };
            state.inspect_tick(tick);
            refresh.bump();
        }
        ObserverCommand::NewCampaign
        | ObserverCommand::ReopenCampaign
        | ObserverCommand::NewDelayedCampaign
        | ObserverCommand::NewSharedFreightAmpleCampaign
        | ObserverCommand::NewSharedFreightConstrainedCampaign
        | ObserverCommand::NewStatewideBaselineCampaign
        | ObserverCommand::NewStatewideFreightConstraintCampaign
        | ObserverCommand::NewStatewidePackagingShortageCampaign
        | ObserverCommand::NewStatewideBothCampaign => {
            if pipe.is_none() {
                feedback.reject(LAUNCHER_REQUIRED, time.elapsed_secs_f64());
                return;
            }
            state.pause_playback();
            let target = match command {
                ObserverCommand::ReopenCampaign => RuntimeSessionTarget::Open {
                    campaign_id: state.campaign.as_uuid().to_string(),
                },
                _ => RuntimeSessionTarget::New {
                    campaign_id: uuid::Uuid::new_v4().to_string(),
                    preset: campaign_preset(command),
                },
            };
            if let Err(error) = state.queue_campaign(target) {
                state.fail(error);
            }
        }
        _ => apply_presentation_command(command, context),
    }
}

fn campaign_preset(command: ObserverCommand) -> RuntimeSessionPreset {
    match command {
        ObserverCommand::NewStatewideBaselineCampaign => RuntimeSessionPreset::StatewideBaseline,
        ObserverCommand::NewStatewideFreightConstraintCampaign => {
            RuntimeSessionPreset::StatewideFreightConstraint
        }
        ObserverCommand::NewStatewidePackagingShortageCampaign => {
            RuntimeSessionPreset::StatewidePackagingShortage
        }
        ObserverCommand::NewStatewideBothCampaign => RuntimeSessionPreset::StatewideBoth,

        ObserverCommand::NewSharedFreightAmpleCampaign => RuntimeSessionPreset::SharedFreightAmple,
        ObserverCommand::NewSharedFreightConstrainedCampaign => {
            RuntimeSessionPreset::SharedFreightConstrained
        }
        ObserverCommand::NewDelayedCampaign => RuntimeSessionPreset::Delayed,
        _ => RuntimeSessionPreset::Standard,
    }
}

fn apply_presentation_command(command: ObserverCommand, context: &mut CommandContext) {
    let CommandContext {
        state,
        ui,
        frame,
        ui_scale,
        audio,
        ..
    } = context;
    match command {
        ObserverCommand::Relationships => {
            ui.lens = crate::map_economy_lens::MapLens::Relationships;
            ui.disclosure = None;
        }
        ObserverCommand::NetworkSector(sector) => {
            ui.network_sector = sector;
            ui.road_layer = crate::observer_ui::RoadLayer::EconomyNetwork;
            ui.disclosure = None;
        }
        ObserverCommand::RoadLayer(layer) => {
            ui.road_layer = layer;
            ui.disclosure = None;
        }
        ObserverCommand::EconomicDetails => ui.economic_details_open = !ui.economic_details_open,
        ObserverCommand::Workforce(metric) => {
            ui.lens = crate::map_economy_lens::MapLens::Workforce(metric);
            ui.disclosure = None;
        }
        ObserverCommand::Lens(metric) => {
            ui.economic_details_open = true;
            ui.lens = crate::map_economy_lens::MapLens::Qcew(metric);
            ui.disclosure = None;
        }
        ObserverCommand::MaterialLens(kind) => {
            ui.economic_details_open = true;
            let good = match &ui.lens {
                crate::map_economy_lens::MapLens::Material { good, .. } => good.clone(),
                crate::map_economy_lens::MapLens::Relationships
                | crate::map_economy_lens::MapLens::Qcew(_)
                | crate::map_economy_lens::MapLens::Workforce(_) => None,
            };
            ui.lens = crate::map_economy_lens::MapLens::Material { kind, good };
            ui.lens.reconcile(frame.for_session(state), false);
            ui.disclosure = None;
        }
        ObserverCommand::CycleGood(backwards) => {
            ui.lens.cycle_good(frame.for_session(state), backwards);
        }
        ObserverCommand::Archive => ui.archive_open = !ui.archive_open,
        ObserverCommand::Menu => {
            ui.menu_open = !ui.menu_open;
            ui.disclosure = None;
            state.pause_playback();
        }
        ObserverCommand::UiScale => ui_scale.0 = if ui_scale.0 < 1.1 { 1.15 } else { 1.0 },
        ObserverCommand::ReducedMotion => ui.reduced_motion = !ui.reduced_motion,
        ObserverCommand::MusicVolume => {
            audio.music_volume = if audio.music_volume < 0.2 {
                0.25
            } else if audio.music_volume < 0.4 {
                0.5
            } else {
                0.0
            }
        }
        ObserverCommand::EffectsVolume => {
            audio.effects_volume = if audio.effects_volume < 0.2 {
                0.4
            } else if audio.effects_volume < 0.6 {
                0.75
            } else {
                0.0
            }
        }
        ObserverCommand::MusicTrack => audio.track = (audio.track + 1) % 2,
        ObserverCommand::History => {
            ui.history_open = !ui.history_open;
            if ui.history_open {
                ui.archive_open = false;
                ui.disclosure = None;
                state.pause_playback();
            }
        }
        ObserverCommand::StopOnDelivery => ui.stop_on_delivery = !ui.stop_on_delivery,
        ObserverCommand::Disclosure(disclosure) => {
            ui.disclosure = if ui.disclosure == Some(disclosure) {
                None
            } else {
                Some(disclosure)
            }
        }
        ObserverCommand::Evidence => ui.evidence_open = !ui.evidence_open,
        _ => unreachable!("transport commands are applied before presentation dispatch"),
    }
}

fn start_observation(
    state: Res<ObserverSession>,
    mut pending: ResMut<PendingObservation>,
    mut frame: ResMut<ObserverFrame>,
) {
    if state.quit_requested {
        pending.0 = None;
        return;
    }
    if let Some((context, _)) = &pending.0 {
        if !state.accepts(context) {
            pending.0 = None;
            frame.0 = None;
        }
    }
    if state.phase != SessionPhase::Loading || pending.0.is_some() {
        return;
    }
    frame.0 = None;
    let context = state.context();
    let requested = context.clone();
    let task = AsyncComputeTaskPool::get().spawn(async move {
        let started = std::time::Instant::now();
        let reader = match requested.perspective {
            Perspective::FullObserver => ObserverEconomyReader::from_observer_env(),
            Perspective::PlayerKnowledge => ObserverEconomyReader::from_known_env(),
        }
        .map_err(|error| error.to_string())?;
        let result = reader
            .snapshot(requested.campaign, requested.tick)
            .map_err(|error| error.to_string());
        bevy::log::info!(target: "babylon_client::timing",
            stage = "authenticated_observer_read",
            campaign = %requested.campaign.as_uuid(),
            tick = requested.tick,
            generation = requested.generation,
            perspective = ?requested.perspective,
            elapsed_us = started.elapsed().as_micros(),
            success = result.is_ok(),
            "observer read completed");
        result
    });
    pending.0 = Some((context, task));
}

fn collect_observation(
    mut state: ResMut<ObserverSession>,
    mut pending: ResMut<PendingObservation>,
    mut frame: ResMut<ObserverFrame>,
    ui: Res<ObserverUiState>,
) {
    let Some((context, task)) = &mut pending.0 else {
        return;
    };
    let Some(result) = block_on(bevy::tasks::futures_lite::future::poll_once(task)) else {
        return;
    };
    let context = context.clone();
    pending.0 = None;
    if !state.accepts(&context) {
        return;
    }
    match result {
        Ok(snapshot) => install_observation(
            &mut state,
            &context,
            snapshot,
            &mut frame,
            ui.stop_on_delivery,
        ),
        Err(error) => state.fail(error),
    }
}

fn install_observation(
    state: &mut ObserverSession,
    context: &ObservationContext,
    snapshot: ObserverEconomySnapshot,
    frame: &mut ObserverFrame,
    stop_on_delivery: bool,
) {
    if !state.accepts(context) {
        return;
    }
    let visibility = match context.perspective {
        Perspective::FullObserver => ObserverVisibility::FullObserver,
        Perspective::PlayerKnowledge => ObserverVisibility::KnownPreview,
    };
    if snapshot.campaign_id != context.campaign.as_uuid().to_string()
        || snapshot.resolve_tick != context.tick
        || snapshot.visibility != visibility
        || state.foundation_digest.as_deref() != Some(snapshot.foundation_digest.as_str())
        || (context.tick == state.durable_tick && snapshot.tick_content_hash != state.content_hash)
    {
        state.fail(
            "Observation identity mismatch; the last committed campaign was preserved.".into(),
        );
        return;
    }
    if let Some(production) = &snapshot.production {
        state.horizon_tick = Some(production.horizon_period);
    }
    if state.installed(context) {
        // Only newly installed, disclosed events from this committed period can
        // interrupt transport. Historical and hidden material cannot pause it.
        if state.playing
            && state.viewed_tick == state.durable_tick
            && snapshot.production.as_ref().is_some_and(|production| {
                production.events.iter().any(|event| {
                    event.period == snapshot.resolve_tick
                        && (event.kind == "freight loss"
                            || (stop_on_delivery && event.kind == "delivery"))
                })
            })
        {
            state.pause_playback();
        }
        frame.0 = Some(snapshot);
    }
}

fn playback(
    time: Res<Time>,
    ui: Res<ObserverUiState>,
    pipe: Option<Res<RuntimePipe>>,
    mut clock: ResMut<PlaybackClock>,
    mut state: ResMut<ObserverSession>,
) {
    if state.quit_requested {
        return;
    }
    let Some(pipe) = pipe else {
        return;
    };
    if ui.splash_visible {
        if state.playing {
            state.pause_playback();
        }
        clock.elapsed = 0.0;
    }
    if !ui.splash_visible && state.playback_due() {
        clock.elapsed += time.delta_secs_f64();
        if clock.elapsed >= state.periods_per_second.recip() {
            clock.elapsed = 0.0;
            send_advance(&pipe, &mut state);
        }
    } else {
        clock.elapsed = 0.0;
    }
}

fn send_campaign_switch(pipe: Option<Res<RuntimePipe>>, mut state: ResMut<ObserverSession>) {
    let Some(pipe) = pipe else {
        return;
    };
    if !state.switch_send_due() {
        return;
    }
    let Some(request) = state.pending_switch_request() else {
        return;
    };
    match pipe.requests.try_send(request) {
        Ok(()) => state.switch_sent(),
        Err(mpsc::TrySendError::Full(_)) => {}
        Err(mpsc::TrySendError::Disconnected(_)) => {
            state.disconnect("Runtime disconnected; close and relaunch Babylon.".into());
        }
    }
}

fn finish_shutdown(
    pipe: Option<Res<RuntimePipe>>,
    mut state: ResMut<ObserverSession>,
    mut shutdown: ResMut<ShutdownProgress>,
    time: Res<Time<Real>>,
    mut exits: MessageWriter<AppExit>,
) {
    if !state.quit_requested || shutdown.exit_sent {
        return;
    }
    if state.playing {
        state.playing = false;
    }
    let started = *shutdown.started_at.get_or_insert(time.elapsed_secs_f64());
    if time.elapsed_secs_f64() - started >= SHUTDOWN_TIMEOUT_SECS {
        state.disconnect(
            "Runtime shutdown timed out; reopen to reconcile committed progress.".into(),
        );
    }
    if pipe.is_none() || state.runtime_disconnected() || state.phase == SessionPhase::Closed {
        shutdown.exit_sent = true;
        exits.write(AppExit::Success);
        return;
    }
    if shutdown.stop_sent {
        return;
    }
    let Some(request) = state.pending_stop_request() else {
        return;
    };
    match pipe
        .expect("connection checked above")
        .requests
        .try_send(request)
    {
        Ok(()) => {
            state.stop_sent();
            shutdown.stop_sent = true;
        }
        Err(mpsc::TrySendError::Full(_)) => {}
        Err(mpsc::TrySendError::Disconnected(_)) => {
            state.disconnect("Runtime disconnected during shutdown; reopen to reconcile.".into());
            shutdown.exit_sent = true;
            exits.write(AppExit::Success);
        }
    }
}

pub struct ObserverIoPlugin;
impl Plugin for ObserverIoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingObservation>()
            .init_resource::<PlaybackClock>()
            .init_resource::<ShutdownProgress>()
            .configure_sets(
                Update,
                (
                    ObserverSet::Input,
                    ObserverSet::Receive,
                    ObserverSet::Install,
                    ObserverSet::Paint,
                )
                    .chain(),
            )
            .add_systems(Startup, start_pipe)
            .add_systems(
                Update,
                (
                    receive,
                    handle_commands,
                    send_campaign_switch,
                    finish_shutdown,
                )
                    .chain()
                    .in_set(ObserverSet::Receive),
            )
            .add_systems(
                Update,
                (start_observation, collect_observation, playback)
                    .chain()
                    .in_set(ObserverSet::Install),
            );
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::observer_audio::ObserverAudioSettings;
    use crate::observer_ui::ObserverDisclosure;
    use babylon_persistence::identity::CampaignId;

    fn command_app() -> (App, mpsc::Receiver<RuntimeSessionRequest>) {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        state.ready(3, None);
        state.connected_fixture();
        state.foundation_digest = Some("foundation".into());
        assert!(state.installed(&state.context()));
        let (requests, receiver) = mpsc::sync_channel(1);
        let (_, responses) = mpsc::channel();
        let mut app = App::new();
        app.insert_resource(state)
            .insert_resource(RuntimePipe {
                requests,
                responses: Mutex::new(responses),
            })
            .insert_resource(ObserverUiState {
                splash_visible: false,
                ..default()
            })
            .init_resource::<ObserverFrame>()
            .init_resource::<DossierRefresh>()
            .init_resource::<UiScale>()
            .init_resource::<ObserverAudioSettings>()
            .init_resource::<ObserverFeedback>()
            .init_resource::<Time>()
            .init_resource::<Time<Real>>()
            .init_resource::<ShutdownProgress>()
            .add_message::<ObserverCommand>()
            .add_message::<AppExit>()
            .add_systems(
                Update,
                (handle_commands, send_campaign_switch, finish_shutdown).chain(),
            );
        (app, receiver)
    }

    fn dispatch(app: &mut App, commands: &[ObserverCommand]) {
        for &command in commands {
            app.world_mut()
                .resource_mut::<Messages<ObserverCommand>>()
                .write(command);
        }
        app.update();
    }

    fn test_scope(campaign_id: String) -> RuntimeSessionScope {
        RuntimeSessionScope {
            epoch: 1,
            campaign_id: Some(campaign_id),
        }
    }

    type ResponseSender = mpsc::Sender<Result<RuntimeSessionResponse, String>>;

    pub(crate) fn quit_app() -> (App, mpsc::Receiver<RuntimeSessionRequest>, ResponseSender) {
        let (mut app, requests) = command_app();
        let (responses, receiver) = mpsc::channel();
        app.world_mut().resource_mut::<RuntimePipe>().responses = Mutex::new(receiver);
        app.init_resource::<PlaybackClock>()
            .add_systems(Update, receive.before(handle_commands));
        (app, requests, responses)
    }

    pub(crate) fn refuse_initial_switch(
        app: &mut App,
        requests: &mpsc::Receiver<RuntimeSessionRequest>,
        responses: &ResponseSender,
    ) -> CampaignId {
        let previous_scope = RuntimeSessionScope::default();
        responses
            .send(Ok(RuntimeSessionResponse::Hello {
                protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
                scope: previous_scope.clone(),
            }))
            .unwrap();
        app.update();
        let RuntimeSessionRequest::Switch {
            request_id, target, ..
        } = requests.try_recv().unwrap()
        else {
            panic!("initial campaign switch");
        };
        let (RuntimeSessionTarget::New { campaign_id, .. }
        | RuntimeSessionTarget::Open { campaign_id }) = target;
        let campaign = CampaignId::from_uuid(uuid::Uuid::parse_str(&campaign_id).unwrap());
        let scope = RuntimeSessionScope {
            epoch: 1,
            campaign_id: Some(campaign_id),
        };
        responses
            .send(Ok(RuntimeSessionResponse::Switching {
                request_id,
                previous_scope,
                scope: scope.clone(),
            }))
            .unwrap();
        responses
            .send(Ok(RuntimeSessionResponse::Error {
                request_id: Some(request_id),
                scope,
                code: babylon_persistence::runtime_session::RuntimeSessionErrorCode::StorageRefused,
                tail: None,
            }))
            .unwrap();
        app.update();
        campaign
    }

    fn exit_count(app: &App) -> usize {
        app.world().resource::<Messages<AppExit>>().len()
    }

    #[test]
    fn opening_history_takes_the_subject_rail_from_archive_without_requesting_a_period() {
        let (mut app, requests) = command_app();
        {
            let mut ui = app.world_mut().resource_mut::<ObserverUiState>();
            ui.archive_open = true;
            ui.disclosure = Some(ObserverDisclosure::Time);
        }
        dispatch(&mut app, &[ObserverCommand::History]);
        let ui = app.world().resource::<ObserverUiState>();
        assert!(ui.history_open);
        assert!(!ui.archive_open);
        assert!(ui.disclosure.is_none());
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
        assert!(requests.try_recv().is_err());
    }

    #[test]
    fn road_layer_commands_are_read_only_and_require_an_inspectable_full_observation() {
        use crate::observer_ui::RoadLayer;
        let (mut app, receiver) = command_app();
        let context = app.world().resource::<ObserverSession>().context();
        dispatch(
            &mut app,
            &[ObserverCommand::RoadLayer(RoadLayer::CapturedRoads)],
        );
        assert_eq!(
            app.world().resource::<ObserverUiState>().road_layer,
            RoadLayer::CapturedRoads
        );
        assert_eq!(app.world().resource::<ObserverSession>().context(), context);
        assert!(receiver.try_recv().is_err());
        for phase in [SessionPhase::Loading, SessionPhase::Failed] {
            app.world_mut().resource_mut::<ObserverSession>().phase = phase;
            dispatch(
                &mut app,
                &[ObserverCommand::RoadLayer(RoadLayer::SelectedPaths)],
            );
            assert_eq!(
                app.world().resource::<ObserverUiState>().road_layer,
                RoadLayer::CapturedRoads
            );
        }
        app.world_mut().resource_mut::<ObserverSession>().phase = SessionPhase::Ready;
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .viewed_tick = 2;
        dispatch(
            &mut app,
            &[ObserverCommand::RoadLayer(RoadLayer::SelectedPaths)],
        );
        assert_eq!(
            app.world().resource::<ObserverUiState>().road_layer,
            RoadLayer::SelectedPaths
        );
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .set_perspective(Perspective::PlayerKnowledge);
        app.world_mut().resource_mut::<ObserverSession>().phase = SessionPhase::Ready;
        dispatch(
            &mut app,
            &[ObserverCommand::RoadLayer(RoadLayer::CapturedRoads)],
        );
        assert_eq!(
            app.world().resource::<ObserverUiState>().road_layer,
            RoadLayer::SelectedPaths
        );
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn relationship_lens_and_economic_disclosure_do_not_advance_the_campaign() {
        let (mut app, requests) = command_app();
        dispatch(&mut app, &[ObserverCommand::EconomicDetails]);
        assert!(
            app.world()
                .resource::<ObserverUiState>()
                .economic_details_open
        );
        dispatch(
            &mut app,
            &[ObserverCommand::Lens(
                crate::map_economy_lens::EconomyMetric::Employment,
            )],
        );
        assert!(matches!(
            app.world().resource::<ObserverUiState>().lens,
            crate::map_economy_lens::MapLens::Qcew(_)
        ));
        dispatch(&mut app, &[ObserverCommand::Relationships]);
        assert_eq!(
            app.world().resource::<ObserverUiState>().lens,
            crate::map_economy_lens::MapLens::Relationships
        );
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
        assert!(requests.try_recv().is_err());
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .splash_visible = true;
        dispatch(&mut app, &[ObserverCommand::EconomicDetails]);
        assert!(
            app.world()
                .resource::<ObserverUiState>()
                .economic_details_open
        );
    }

    #[test]
    fn launcher_handoff_without_pipe_keeps_the_window_and_campaign_open() {
        for command in [
            ObserverCommand::NewCampaign,
            ObserverCommand::NewDelayedCampaign,
            ObserverCommand::NewSharedFreightAmpleCampaign,
            ObserverCommand::NewSharedFreightConstrainedCampaign,
            ObserverCommand::ReopenCampaign,
        ] {
            let (mut app, _) = command_app();
            app.world_mut().remove_resource::<RuntimePipe>();
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .fail("No runtime is connected".into());
            let context = app.world().resource::<ObserverSession>().context();
            dispatch(&mut app, &[command]);
            assert_eq!(
                exit_count(&app),
                0,
                "{command:?} closed a standalone window"
            );
            assert_eq!(app.world().resource::<ObserverSession>().context(), context);
            assert_eq!(
                app.world().resource::<ObserverFeedback>().message,
                Some(
                    "This window has no launcher connection. Close it and start Babylon through its launcher."
                )
            );
        }
    }

    #[test]
    fn campaign_choices_after_admission_failure_request_switch_without_exiting() {
        for command in [
            ObserverCommand::NewCampaign,
            ObserverCommand::NewDelayedCampaign,
            ObserverCommand::NewSharedFreightAmpleCampaign,
            ObserverCommand::NewSharedFreightConstrainedCampaign,
            ObserverCommand::ReopenCampaign,
        ] {
            let (mut app, requests) = command_app();
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .fail("Admission refused".into());
            dispatch(&mut app, &[command]);
            assert_eq!(exit_count(&app), 0);
            let RuntimeSessionRequest::Switch { scope, target, .. } = requests.try_recv().unwrap()
            else {
                panic!("one campaign switch");
            };
            assert_eq!(scope, test_scope(uuid::Uuid::from_u128(1).to_string()));
            match (command, target) {
                (ObserverCommand::ReopenCampaign, RuntimeSessionTarget::Open { campaign_id }) => {
                    assert_eq!(campaign_id, uuid::Uuid::from_u128(1).to_string());
                }
                (
                    ObserverCommand::NewCampaign,
                    RuntimeSessionTarget::New {
                        preset: RuntimeSessionPreset::Standard,
                        ..
                    },
                )
                | (
                    ObserverCommand::NewDelayedCampaign,
                    RuntimeSessionTarget::New {
                        preset: RuntimeSessionPreset::Delayed,
                        ..
                    },
                )
                | (
                    ObserverCommand::NewSharedFreightAmpleCampaign,
                    RuntimeSessionTarget::New {
                        preset: RuntimeSessionPreset::SharedFreightAmple,
                        ..
                    },
                )
                | (
                    ObserverCommand::NewSharedFreightConstrainedCampaign,
                    RuntimeSessionTarget::New {
                        preset: RuntimeSessionPreset::SharedFreightConstrained,
                        ..
                    },
                ) => {}
                _ => panic!("campaign choice changed its target or preset"),
            }
        }
    }

    #[test]
    fn new_before_ready_keeps_transport_and_failed_ui_recoverable() {
        let (mut app, requests, responses) = quit_app();
        let campaign = app.world().resource::<ObserverSession>().campaign;
        app.insert_resource(ObserverSession::new(campaign));

        dispatch(&mut app, &[ObserverCommand::NewCampaign]);
        assert_eq!(
            exit_count(&app),
            0,
            "New closed the runtime response pipe before Ready"
        );
        assert!(app.world().contains_resource::<RuntimePipe>());
        assert_eq!(app.world().resource::<ObserverSession>().campaign, campaign);

        let admitted_target = refuse_initial_switch(&mut app, &requests, &responses);

        assert_eq!(
            exit_count(&app),
            0,
            "Startup failure closed the recovery UI"
        );
        assert!(app.world().contains_resource::<RuntimePipe>());
        assert!(app.world().resource::<ObserverUiState>().menu_open);
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.phase, SessionPhase::Failed);
        assert_eq!(state.campaign, admitted_target);
        assert_eq!(state.durable_tick, 0);
        assert_eq!(
            state.error.as_deref(),
            Some(
                babylon_persistence::runtime_session::RuntimeSessionErrorCode::StorageRefused
                    .to_string()
                    .as_str()
            )
        );
        assert_eq!(
            availability(ObserverCommand::ReopenCampaign, state),
            ControlAvailability::Enabled
        );
    }

    #[test]
    fn launcher_handoff_without_pipe_still_allows_explicit_quit() {
        let (mut app, _) = command_app();
        app.world_mut().remove_resource::<RuntimePipe>();
        dispatch(&mut app, &[ObserverCommand::Quit]);
        assert!(matches!(
            app.world_mut()
                .resource_mut::<Messages<AppExit>>()
                .drain()
                .collect::<Vec<_>>()
                .as_slice(),
            [AppExit::Success]
        ));
    }

    #[test]
    fn opening_warning_refuses_queued_gameplay_commands_but_allows_quit() {
        let (mut app, requests) = command_app();
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .splash_visible = true;
        dispatch(
            &mut app,
            &[
                ObserverCommand::TogglePlay,
                ObserverCommand::Step,
                ObserverCommand::Perspective,
            ],
        );
        let state = app.world().resource::<ObserverSession>();
        assert!(!state.playing);
        assert!(!state.advance_pending());
        assert_eq!(state.durable_tick, 3);
        assert_eq!(state.perspective, Perspective::FullObserver);
        assert!(requests.try_recv().is_err());
        dispatch(&mut app, &[ObserverCommand::Quit]);
        assert!(app.world().resource::<ObserverSession>().quit_requested);
        assert!(matches!(
            requests.try_recv().unwrap(),
            RuntimeSessionRequest::Stop { .. }
        ));
    }

    #[test]
    fn opening_warning_pauses_scheduled_playback_without_sending_an_advance() {
        let (mut app, requests) = command_app();
        app.world_mut()
            .resource_mut::<ObserverUiState>()
            .splash_visible = true;
        assert!(app
            .world_mut()
            .resource_mut::<ObserverSession>()
            .start_playback());
        app.insert_resource(PlaybackClock { elapsed: 10.0 })
            .add_systems(Update, playback.after(handle_commands));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(2));
        app.update();
        let state = app.world().resource::<ObserverSession>();
        assert!(!state.playing);
        assert!(!state.advance_pending());
        assert_eq!(state.durable_tick, 3);
        assert!(requests.try_recv().is_err());
        assert_eq!(
            app.world().resource::<PlaybackClock>().elapsed.to_bits(),
            0.0_f64.to_bits()
        );
    }

    #[test]
    fn obsolete_runtime_handshake_is_refused_before_any_archive_or_advance_work() {
        let (mut app, requests, responses) = quit_app();
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .foundation_digest = None;
        responses
            .send(Ok(RuntimeSessionResponse::Hello {
                protocol_version: 2,
                scope: RuntimeSessionScope {
                    epoch: 0,
                    campaign_id: None,
                },
            }))
            .unwrap();
        app.update();
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.phase, SessionPhase::Failed);
        assert!(state.foundation_digest.is_none());
        assert_eq!(state.durable_tick, 3);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
        assert!(requests.try_recv().is_err());
    }

    #[derive(Resource, Default)]
    struct SessionChanges(Vec<bool>);

    fn record_session_changes(state: Res<ObserverSession>, mut changes: ResMut<SessionChanges>) {
        changes.0.push(state.is_changed());
    }

    #[test]
    fn idle_pipe_and_observation_polling_do_not_invalidate_the_session_each_frame() {
        let (mut app, _requests, responses) = quit_app();
        app.init_resource::<PendingObservation>()
            .init_resource::<SessionChanges>()
            .add_systems(
                Update,
                (
                    start_observation,
                    collect_observation,
                    playback,
                    record_session_changes,
                )
                    .chain()
                    .after(finish_shutdown),
            );
        app.update();
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<SessionChanges>().0,
            [true, false, false]
        );
        let campaign_id = app
            .world()
            .resource::<ObserverSession>()
            .campaign
            .as_uuid()
            .to_string();
        responses
            .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                scope: test_scope(campaign_id),
                durable_tick: 3,
                verified_tick: 1,
                request_id: None,
            }))
            .unwrap();
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<SessionChanges>().0,
            [true, false, false, true, false]
        );
        drop(responses);
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<SessionChanges>().0,
            [true, false, false, true, false, true, false]
        );
    }

    #[test]
    fn archive_push_refreshes_foundation_and_horizon_without_client_polling() {
        for tick in [0, 16] {
            let (mut app, requests, responses) = quit_app();
            {
                let mut state = app.world_mut().resource_mut::<ObserverSession>();
                state.horizon_tick = Some(16);
                state.ready(tick, None);
                let context = state.context();
                assert!(state.installed(&context));
                state.archive_verified_tick = tick;
                if Some(tick) == state.horizon_tick {
                    state.complete();
                }
            }
            app.add_systems(Update, playback.after(finish_shutdown));
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs(2));
            let campaign_id = app
                .world()
                .resource::<ObserverSession>()
                .campaign
                .as_uuid()
                .to_string();
            let context = app.world().resource::<ObserverSession>().context();
            let initial_refresh = app.world().resource::<DossierRefresh>().0;
            for _ in 0..3 {
                app.update();
                assert!(
                    requests.try_recv().is_err(),
                    "Archive work is pushed; a paused client never polls"
                );
            }
            // A partial publication can advance even when the processed tick
            // is unchanged. Every genuine push invalidates the held read.
            for index in 0..3 {
                responses
                    .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                        request_id: None,
                        scope: test_scope(campaign_id.clone()),
                        durable_tick: tick,
                        verified_tick: tick,
                    }))
                    .unwrap();
                app.update();
                assert_eq!(
                    app.world().resource::<DossierRefresh>().0,
                    initial_refresh + u64::try_from(index).unwrap() + 1
                );
                let state = app.world().resource::<ObserverSession>();
                assert_eq!(state.context(), context);
                assert_eq!(state.durable_tick, tick);
                assert_eq!(state.archive_verified_tick, tick);
                assert!(!state.playing);
                assert!(
                    requests.try_recv().is_err(),
                    "progress never requests another sweep or tick"
                );
            }
            app.update();
            assert!(requests.try_recv().is_err());
            assert_eq!(
                app.world().resource::<DossierRefresh>().0,
                initial_refresh + 3
            );
        }
    }

    #[test]
    fn archive_push_rejects_wrong_identity_future_tail_and_regressing_prefix() {
        for (wrong_campaign, durable_tick, verified_tick) in [
            (true, 3, 2),
            (false, 2, 2),
            (false, 4, 2),
            (false, 3, 4),
            (false, 3, 1),
        ] {
            let (mut app, requests, responses) = quit_app();
            app.world_mut()
                .resource_mut::<ObserverSession>()
                .archive_verified_tick = 2;
            let state = app.world().resource::<ObserverSession>();
            let campaign_id = if wrong_campaign {
                uuid::Uuid::from_u128(999).to_string()
            } else {
                state.campaign.as_uuid().to_string()
            };
            let context = state.context();
            responses
                .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                    request_id: None,
                    scope: test_scope(campaign_id),
                    durable_tick,
                    verified_tick,
                }))
                .unwrap();
            app.update();
            let state = app.world().resource::<ObserverSession>();
            assert_eq!(state.phase, SessionPhase::Failed);
            assert_eq!(state.context(), context);
            assert_eq!(state.durable_tick, 3);
            assert_eq!(state.archive_verified_tick, 2);
            assert_eq!(app.world().resource::<DossierRefresh>().0, 0);
            assert!(requests.try_recv().is_err());
        }
    }

    #[test]
    fn archive_push_invalidates_a_held_read_without_certifying_its_page() {
        use crate::ui::dossier_card::{ActiveCountyDossier, DossierRequestScope, InstalledDossier};
        use babylon_persistence::archive_revision::{
            ArchiveDossierPending, ArchiveDossierRead, ArchiveDossierState, ArchiveReadScope,
        };
        use babylon_persistence::{ArchivePageRef, ArchiveSubjectKind};
        let (mut app, requests, responses) = quit_app();
        {
            let mut state = app.world_mut().resource_mut::<ObserverSession>();
            state.content_hash = Some("a".repeat(64));
            state.foundation_digest = Some("foundation".into());
            state.archive_verified_tick = 2;
        }
        let state = app.world().resource::<ObserverSession>();
        let campaign = state.campaign;
        let context = state.context();
        let frame = ObserverFrame(Some(snapshot_with_event(state, "production", 3)));
        let scope = ArchiveReadScope::committed(campaign, 3, [0xaa; 32]).unwrap();
        let subject = ArchivePageRef::try_new(ArchiveSubjectKind::County, "26163".into()).unwrap();
        let read = ArchiveDossierRead {
            scope: scope.clone(),
            subject: subject.clone(),
            durable_tick: 3,
            processed_tick: 2,
            state: ArchiveDossierState::Pending {
                page: None,
                reason: ArchiveDossierPending::ReceiptProcessing,
            },
        };
        let active = ActiveCountyDossier(Some(InstalledDossier {
            scope: DossierRequestScope {
                campaign,
                county_geoid: "26163".into(),
                refresh_generation: 0,
                observer: Some(context.clone()),
                read_scope: scope,
                subject,
            },
            read: read.clone(),
        }));
        assert!(active.for_observer(state, &frame, 0, "26163").is_some());
        app.insert_resource(frame).insert_resource(active);
        responses
            .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                request_id: None,
                scope: test_scope(campaign.as_uuid().to_string()),
                durable_tick: 3,
                verified_tick: 2,
            }))
            .unwrap();
        app.update();
        let world = app.world();
        let active = world.resource::<ActiveCountyDossier>();
        let state = world.resource::<ObserverSession>();
        let refresh = world.resource::<DossierRefresh>().0;
        assert_eq!(refresh, 1);
        assert_eq!(state.context(), context);
        assert!(active
            .for_observer(state, world.resource::<ObserverFrame>(), refresh, "26163")
            .is_none());
        assert_eq!(
            active.0.as_ref().unwrap().read,
            read,
            "a progress notification cannot replace or verify the scoped reader's page"
        );
        assert!(requests.try_recv().is_err());
    }

    #[test]
    fn archive_push_during_an_advance_never_substitutes_for_its_commit_ack() {
        let (mut app, requests, responses) = quit_app();
        dispatch(&mut app, &[ObserverCommand::Step]);
        let RuntimeSessionRequest::Advance {
            request_id, scope, ..
        } = requests.try_recv().unwrap()
        else {
            panic!("one explicit advance request");
        };
        let campaign_id = scope.campaign_id.unwrap();
        responses
            .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                request_id: None,
                scope: test_scope(campaign_id.clone()),
                durable_tick: 3,
                verified_tick: 2,
            }))
            .unwrap();
        app.update();
        let state = app.world().resource::<ObserverSession>();
        assert!(state.advance_pending());
        assert_eq!(state.phase, SessionPhase::Advancing);
        assert_eq!(state.durable_tick, 3);
        responses
            .send(Ok(RuntimeSessionResponse::Committed {
                request_id,
                scope: test_scope(campaign_id.clone()),
                tail: RuntimeSessionTail {
                    resolve_tick: 4,
                    tick_content_hash: Some("4".repeat(64)),
                },
            }))
            .unwrap();
        responses
            .send(Ok(RuntimeSessionResponse::ArchiveProgress {
                request_id: None,
                scope: test_scope(campaign_id),
                durable_tick: 4,
                verified_tick: 3,
            }))
            .unwrap();
        app.update();
        let state = app.world().resource::<ObserverSession>();
        assert!(!state.advance_pending());
        assert_eq!(state.phase, SessionPhase::Loading);
        assert_eq!(state.durable_tick, 4);
        assert_eq!(
            state.content_hash.as_deref(),
            Some("4".repeat(64)).as_deref()
        );
        assert_eq!(state.archive_verified_tick, 3);
        assert_eq!(app.world().resource::<DossierRefresh>().0, 3);
        assert!(requests.try_recv().is_err());
    }

    #[test]
    fn playback_waits_for_each_period_ack_and_observation_then_stops_at_the_horizon() {
        let (mut app, requests) = command_app();
        app.init_resource::<PlaybackClock>()
            .add_systems(Update, playback.after(handle_commands));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        app.world_mut()
            .resource_mut::<ObserverSession>()
            .horizon_tick = Some(5);
        dispatch(&mut app, &[ObserverCommand::TogglePlay]);
        for expected_period in [4, 5] {
            let RuntimeSessionRequest::Advance {
                request_id,
                expected_tail,
                ..
            } = requests.try_recv().unwrap()
            else {
                panic!("playback must send one period advance");
            };
            assert_eq!(expected_tail.resolve_tick, expected_period - 1);
            app.update();
            assert!(
                requests.try_recv().is_err(),
                "never a second outstanding advance"
            );
            {
                let mut state = app.world_mut().resource_mut::<ObserverSession>();
                assert!(state.acknowledge(request_id, expected_period, None));
            }
            app.update();
            assert!(
                requests.try_recv().is_err(),
                "ack alone does not authorize unread progress"
            );
            {
                let mut state = app.world_mut().resource_mut::<ObserverSession>();
                let context = state.context();
                assert!(state.installed(&context));
            }
            app.update();
        }
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.durable_tick, 5);
        assert!(!state.playing);
        assert!(
            requests.try_recv().is_err(),
            "scenario horizon cannot overrun"
        );
    }

    pub(super) fn snapshot_with_event(
        state: &ObserverSession,
        kind: &str,
        period: u64,
    ) -> ObserverEconomySnapshot {
        ObserverEconomySnapshot {
            campaign_id: state.campaign.as_uuid().to_string(),
            resolve_tick: state.viewed_tick,
            foundation_digest: "foundation".into(),
            nominal_world_hash: None,
            tick_content_hash: state.content_hash.clone(),
            envelope_digest: None,
            visibility: ObserverVisibility::FullObserver,
            counties: Vec::new(),
            production: Some(
                babylon_persistence::production_observation::ProductionSnapshot {
                    content_authority_sha256: "a".repeat(64),
                    road_source: None,
                    physical_edges: Vec::new(),
                    merchant_handling_accounts: Vec::new(),
                    final_demand_accounts: Vec::new(),
                    freight_capacity_accounts: Vec::new(),
                    material_balance: None,
                    labor_accounts: Vec::new(),
                    staffing_accounts: Vec::new(),
                    scenario_label: "bounded observer fixture".into(),
                    horizon_period: 16,
                    sites: Vec::new(),
                    routes: Vec::new(),
                    freight: Vec::new(),
                    observed_contexts: Vec::new(),
                    process_attributions: Vec::new(),
                    provenance: Vec::new(),
                    events: vec![
                        babylon_persistence::production_observation::ProductionEvent {
                            id: "committed-event".into(),
                            period,
                            subject_site_ids: Vec::new(),
                            kind: kind.into(),
                            description: "disclosed committed development".into(),
                            receipt_digest: "receipt".into(),
                            delivery_evidence: None,
                        },
                    ],
                },
            ),
        }
    }

    #[test]
    fn playback_interruptions_use_only_newly_installed_disclosed_commit_events() {
        for (kind, event_period, delivery_stop, stays_running) in [
            ("delivery", 3, false, true),
            ("delivery", 3, true, false),
            ("freight loss", 3, false, false),
            ("freight loss", 2, false, true),
            ("freight loss", 4, false, true),
            ("production", 3, true, true),
        ] {
            let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
            state.ready(3, None);
            state.foundation_digest = Some("foundation".into());
            assert!(state.start_playback());
            let context = state.context();
            let snapshot = snapshot_with_event(&state, kind, event_period);
            let mut frame = ObserverFrame::default();
            install_observation(
                &mut state,
                &context,
                snapshot.clone(),
                &mut frame,
                delivery_stop,
            );
            assert_eq!(
                state.playing, stays_running,
                "{kind} at period {event_period}"
            );
            assert!(frame.0.is_some());
            assert!(state.start_playback());
            install_observation(&mut state, &context, snapshot, &mut frame, delivery_stop);
            assert!(
                state.playing,
                "resuming cannot replay an already installed interruption"
            );
        }
    }

    #[test]
    fn stale_and_known_observations_cannot_interrupt_or_install_hidden_period_events() {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        state.ready(3, None);
        state.foundation_digest = Some("foundation".into());
        let stale_context = state.context();
        let stale_snapshot = snapshot_with_event(&state, "freight loss", 3);
        state.set_perspective(Perspective::PlayerKnowledge);
        assert!(state.start_playback());
        let mut frame = ObserverFrame::default();
        install_observation(&mut state, &stale_context, stale_snapshot, &mut frame, true);
        assert!(state.playing);
        assert!(frame.0.is_none());
        let context = state.context();
        let mut known = snapshot_with_event(&state, "freight loss", 3);
        known.visibility = ObserverVisibility::KnownPreview;
        known.production = None;
        install_observation(&mut state, &context, known, &mut frame, true);
        assert!(state.playing);
        assert!(frame.0.as_ref().unwrap().production.is_none());
    }

    #[test]
    fn quit_waits_for_commit_and_stop_acknowledgements_without_advancing_again() {
        let (mut app, requests, responses) = quit_app();
        dispatch(&mut app, &[ObserverCommand::Step]);
        assert!(matches!(
            requests.try_recv().unwrap(),
            RuntimeSessionRequest::Advance { .. }
        ));
        dispatch(&mut app, &[ObserverCommand::Quit]);
        assert!(matches!(
            requests.try_recv().unwrap(),
            RuntimeSessionRequest::Stop { request_id: 2, .. }
        ));
        assert_eq!(exit_count(&app), 0);
        assert!(app.world().resource::<ObserverSession>().advance_pending());
        dispatch(
            &mut app,
            &[ObserverCommand::TogglePlay, ObserverCommand::Step],
        );
        assert!(!app.world().resource::<ObserverSession>().playing);
        assert!(requests.try_recv().is_err());
        responses
            .send(Ok(RuntimeSessionResponse::Committed {
                request_id: 1,
                scope: app
                    .world()
                    .resource::<ObserverSession>()
                    .runtime_scope()
                    .unwrap()
                    .clone(),
                tail: RuntimeSessionTail {
                    resolve_tick: 4,
                    tick_content_hash: Some("committed".into()),
                },
            }))
            .unwrap();
        app.update();
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 4);
        assert_eq!(exit_count(&app), 0);
        responses
            .send(Ok(RuntimeSessionResponse::Stopped {
                request_id: 2,
                scope: app
                    .world()
                    .resource::<ObserverSession>()
                    .runtime_scope()
                    .unwrap()
                    .clone(),
            }))
            .unwrap();
        drop(responses);
        app.update();
        assert_eq!(
            app.world().resource::<ObserverSession>().phase,
            SessionPhase::Closed
        );
        assert_eq!(exit_count(&app), 1);
        app.update();
        assert_eq!(exit_count(&app), 1);
    }

    #[test]
    fn quit_retries_a_full_pipe_and_does_not_duplicate_stop() {
        let (mut app, requests, _responses) = quit_app();
        dispatch(&mut app, &[ObserverCommand::Step, ObserverCommand::Quit]);
        assert_eq!(exit_count(&app), 0);
        assert!(!app.world().resource::<ShutdownProgress>().stop_sent);
        assert!(matches!(
            requests.try_recv().unwrap(),
            RuntimeSessionRequest::Advance { .. }
        ));
        app.update();
        assert!(matches!(
            requests.try_recv().unwrap(),
            RuntimeSessionRequest::Stop { .. }
        ));
        dispatch(&mut app, &[ObserverCommand::Quit]);
        assert!(requests.try_recv().is_err());
        assert_eq!(exit_count(&app), 0);
    }

    #[test]
    fn quit_exits_on_failed_or_disconnected_runtime_without_claiming_pending_commit() {
        for disconnected in [false, true] {
            let (mut app, requests, responses) = quit_app();
            dispatch(&mut app, &[ObserverCommand::Step]);
            requests.try_recv().unwrap();
            if disconnected {
                drop(responses);
            } else {
                responses.send(Err("Runtime pipe failed".into())).unwrap();
            }
            dispatch(&mut app, &[ObserverCommand::Quit]);
            let state = app.world().resource::<ObserverSession>();
            assert_eq!(state.phase, SessionPhase::Failed);
            assert_eq!(state.durable_tick, 3);
            assert!(state.advance_pending());
            assert_eq!(exit_count(&app), 1);
        }
    }

    #[test]
    fn quit_has_a_wall_clock_deadline_when_runtime_never_answers() {
        let (mut app, requests, _responses) = quit_app();
        dispatch(&mut app, &[ObserverCommand::Quit]);
        requests.try_recv().unwrap();
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_secs(149));
        app.update();
        assert_eq!(exit_count(&app), 0);
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_secs(1));
        app.update();
        assert_eq!(exit_count(&app), 1);
        assert_eq!(app.world().resource::<ObserverSession>().durable_tick, 3);
    }

    #[test]
    fn manual_period_advance_sends_once_commits_once_and_remains_paused() {
        let (mut app, receiver) = command_app();
        app.init_resource::<PlaybackClock>()
            .add_systems(Update, playback.after(handle_commands));
        assert!(app
            .world_mut()
            .resource_mut::<ObserverSession>()
            .start_playback());
        dispatch(&mut app, &[ObserverCommand::Step, ObserverCommand::Step]);
        assert!(matches!(
            receiver.try_recv().unwrap(),
            RuntimeSessionRequest::Advance {
                request_id: 1,
                expected_tail: RuntimeSessionTail {
                    resolve_tick: 3,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.durable_tick, 3);
        assert_eq!(state.phase, SessionPhase::Advancing);
        assert!(state.advance_pending());
        let feedback = app.world().resource::<ObserverFeedback>();
        assert_eq!(
            feedback.message,
            Some("Wait for the current period to finish committing")
        );
        assert_eq!(feedback.revision, 1);
        assert!((feedback.expires_at - 4.0).abs() < f64::EPSILON);
        {
            let mut state = app.world_mut().resource_mut::<ObserverSession>();
            assert!(state.acknowledge(1, 4, None));
            let context = state.context();
            assert!(state.installed(&context));
        }
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs(1));
        for _ in 0..5 {
            app.update();
        }
        let state = app.world().resource::<ObserverSession>();
        assert_eq!(state.durable_tick, 4);
        assert!(!state.playing);
        assert!(!state.advance_pending());
        assert!(
            receiver.try_recv().is_err(),
            "one four-week advance cannot schedule extra commits"
        );
    }

    #[test]
    fn loading_can_pause_then_queue_play_without_sending_an_advance() {
        let (mut app, receiver) = command_app();
        {
            let mut state = app.world_mut().resource_mut::<ObserverSession>();
            state.phase = SessionPhase::Loading;
            state.playing = true;
        }
        dispatch(&mut app, &[ObserverCommand::TogglePlay]);
        assert!(!app.world().resource::<ObserverSession>().playing);
        dispatch(&mut app, &[ObserverCommand::TogglePlay]);
        let state = app.world().resource::<ObserverSession>();
        assert!(state.playing);
        assert_eq!(state.phase, SessionPhase::Loading);
        assert!(!state.advance_pending());
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        assert!(app.world().resource::<ObserverFeedback>().message.is_none());
    }

    #[test]
    fn history_disclosure_pauses_further_play_without_cancelling_the_period() {
        let (mut app, receiver) = command_app();
        dispatch(&mut app, &[ObserverCommand::Step]);
        receiver.try_recv().unwrap();
        app.world_mut().resource_mut::<ObserverSession>().playing = true;
        dispatch(&mut app, &[ObserverCommand::History]);
        let state = app.world().resource::<ObserverSession>();
        assert!(!state.playing);
        assert!(state.advance_pending());
        assert_eq!(state.durable_tick, 3);
        assert!(app.world().resource::<ObserverUiState>().history_open);
        dispatch(&mut app, &[ObserverCommand::History]);
        assert!(!app.world().resource::<ObserverUiState>().history_open);
        assert!(app.world().resource::<ObserverSession>().advance_pending());
        assert!(app.world().resource::<ObserverFeedback>().message.is_none());
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }

    #[test]
    fn menu_disclosure_and_settings_commands_apply_without_transport() {
        let (mut app, receiver) = command_app();
        app.world_mut().resource_mut::<ObserverSession>().playing = true;
        dispatch(
            &mut app,
            &[
                ObserverCommand::Disclosure(ObserverDisclosure::Time),
                ObserverCommand::ReducedMotion,
                ObserverCommand::UiScale,
                ObserverCommand::Archive,
                ObserverCommand::StopOnDelivery,
                ObserverCommand::Evidence,
            ],
        );
        let ui = app.world().resource::<ObserverUiState>();
        assert_eq!(ui.disclosure, Some(ObserverDisclosure::Time));
        assert!(ui.reduced_motion && ui.archive_open && ui.stop_on_delivery && ui.evidence_open);
        assert!((app.world().resource::<UiScale>().0 - 1.15).abs() < f32::EPSILON);
        assert!(app.world().resource::<ObserverSession>().playing);
        dispatch(
            &mut app,
            &[ObserverCommand::Disclosure(ObserverDisclosure::Time)],
        );
        assert_eq!(app.world().resource::<ObserverUiState>().disclosure, None);
        dispatch(
            &mut app,
            &[
                ObserverCommand::Disclosure(ObserverDisclosure::Lens),
                ObserverCommand::Menu,
            ],
        );
        let ui = app.world().resource::<ObserverUiState>();
        assert!(!ui.menu_open);
        assert_eq!(ui.disclosure, None);
        assert!(!app.world().resource::<ObserverSession>().playing);
        assert!(matches!(
            receiver.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
    }
}
