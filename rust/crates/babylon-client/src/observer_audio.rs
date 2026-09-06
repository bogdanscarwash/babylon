//! Music and restrained feedback from viewer interactions and acknowledged commits.
//! Audio never participates in campaign identity or mechanics.

use bevy::asset::io::embedded::EmbeddedAssetRegistry;
use bevy::audio::{AudioSinkPlayback, Volume};
use bevy::prelude::*;
use std::path::{Path, PathBuf};

use crate::map::SelectedCounty;
use crate::observer::{ObserverSession, SessionPhase};
use crate::observer_io::ObserverSet;
use crate::observer_ui::ObserverCommand;

#[derive(Resource)]
pub struct ObserverAudioSettings {
    pub music_volume: f32,
    pub effects_volume: f32,
    pub track: usize,
}
impl Default for ObserverAudioSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.25,
            effects_volume: 0.4,
            track: 0,
        }
    }
}

#[derive(Resource)]
struct AudioBank {
    select: Handle<AudioSource>,
    tab: Handle<AudioSource>,
    open: Handle<AudioSource>,
    back: Handle<AudioSource>,
    tick: Handle<AudioSource>,
    fault: Handle<AudioSource>,
}
#[derive(Component)]
struct MusicDeck(usize);
#[derive(Component)]
struct EffectLifetime(Timer);

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    for (index, path) in [
        "music/babylon_theme_phi.ogg",
        "music/babylon_theme_panopticon.ogg",
    ]
    .iter()
    .enumerate()
    {
        commands.spawn((
            AudioPlayer::new(server.load(format!("embedded://{path}"))),
            PlaybackSettings {
                volume: Volume::SILENT,
                ..PlaybackSettings::LOOP
            },
            MusicDeck(index),
        ));
    }
    commands.insert_resource(AudioBank {
        select: server.load("embedded://sfx/ui/ui_select.ogg"),
        tab: server.load("embedded://sfx/ui/ui_tab.ogg"),
        open: server.load("embedded://sfx/ui/ui_open.ogg"),
        back: server.load("embedded://sfx/ui/ui_back.ogg"),
        tick: server.load("embedded://sfx/state/tick_advance.ogg"),
        fault: server.load("embedded://sfx/state/state_fault.ogg"),
    });
}

fn effect(commands: &mut Commands, source: Handle<AudioSource>, volume: f32) {
    if volume <= 0.0 {
        return;
    }
    commands.spawn((
        AudioPlayer::new(source),
        PlaybackSettings {
            volume: Volume::Linear(volume),
            ..PlaybackSettings::DESPAWN
        },
        EffectLifetime(Timer::from_seconds(8.0, TimerMode::Once)),
    ));
}

fn feedback(
    mut commands: Commands,
    mut input: MessageReader<ObserverCommand>,
    bank: Res<AudioBank>,
    settings: Res<ObserverAudioSettings>,
    state: Res<ObserverSession>,
    selection: Res<SelectedCounty>,
    mut last: Local<Option<(u64, SessionPhase, Option<usize>)>>,
) {
    let mut cue = None;
    for input in input.read() {
        cue = Some(match input {
            ObserverCommand::Lens(_)
            | ObserverCommand::MaterialLens(_)
            | ObserverCommand::CycleGood(_)
            | ObserverCommand::Perspective => bank.tab.clone(),
            ObserverCommand::Archive | ObserverCommand::Menu => bank.open.clone(),
            ObserverCommand::PreviousWeek | ObserverCommand::Live => bank.back.clone(),
            _ => bank.select.clone(),
        });
    }
    if let Some((tick, phase, selected)) = *last {
        if selection.0 != selected {
            cue = Some(bank.select.clone());
        }
        if state.durable_tick > tick {
            cue = Some(bank.tick.clone());
        }
        if state.phase == SessionPhase::Failed && phase != SessionPhase::Failed {
            cue = Some(bank.fault.clone());
        }
    }
    *last = Some((state.durable_tick, state.phase, selection.0));
    if let Some(cue) = cue {
        effect(&mut commands, cue, settings.effects_volume);
    }
}

fn mix(
    time: Res<Time>,
    settings: Res<ObserverAudioSettings>,
    mut decks: Query<(&MusicDeck, &mut AudioSink)>,
    mut effects: Query<(Entity, &mut EffectLifetime)>,
    mut commands: Commands,
) {
    for (deck, mut sink) in &mut decks {
        let target = if deck.0 == settings.track {
            settings.music_volume
        } else {
            0.0
        };
        let current = sink.volume().to_linear();
        let step = time.delta_secs() * 0.35;
        let volume = if current < target {
            (current + step).min(target)
        } else {
            (current - step).max(target)
        };
        sink.set_volume(Volume::Linear(volume));
    }
    for (entity, mut ttl) in &mut effects {
        ttl.0.tick(time.delta());
        if ttl.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ObserverAudioPlugin;
impl Plugin for ObserverAudioPlugin {
    fn build(&self, app: &mut App) {
        let registry = app.world().resource::<EmbeddedAssetRegistry>();
        for (name, bytes) in [
            (
                "music/babylon_theme_phi.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/music/babylon_theme_phi.ogg"
                )) as &[u8],
            ),
            (
                "music/babylon_theme_panopticon.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/music/babylon_theme_panopticon.ogg"
                )) as &[u8],
            ),
            (
                "sfx/ui/ui_select.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/ui/ui_select.ogg"
                )) as &[u8],
            ),
            (
                "sfx/ui/ui_tab.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/ui/ui_tab.ogg"
                )) as &[u8],
            ),
            (
                "sfx/ui/ui_open.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/ui/ui_open.ogg"
                )) as &[u8],
            ),
            (
                "sfx/ui/ui_back.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/ui/ui_back.ogg"
                )) as &[u8],
            ),
            (
                "sfx/state/tick_advance.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/state/tick_advance.ogg"
                )) as &[u8],
            ),
            (
                "sfx/state/state_fault.ogg",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../../assets/sfx/state/state_fault.ogg"
                )) as &[u8],
            ),
        ] {
            registry.insert_asset(
                PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets")).join(name),
                Path::new(name),
                bytes,
            );
        }
        app.init_resource::<ObserverAudioSettings>()
            .add_systems(Startup, setup)
            .add_systems(Update, (feedback, mix).chain().after(ObserverSet::Install));
    }
}
