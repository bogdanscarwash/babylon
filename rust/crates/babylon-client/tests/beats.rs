//! B3 wave-1 Task 4.3's wired-feed proof (plan §2.2/§2.4/§2.7): the drained,
//! tick-stamped, severity-collapsed beat feed, and the counties story's
//! measured validated horizon. Headless real-`App` assertions, real
//! `KeyboardInput` messages, virtual time only (I4) — the same house
//! pattern `tests/time_controls.rs`/`tests/projection.rs` already
//! establish.
//!
//! RED at this commit: `babylon_client::ui::beats` exports none of
//! `BeatLog`, `BeatFeedText`, `BEAT_LOG_CAPACITY`, `COUNTIES_VALIDATED_HORIZON`
//! yet — mirrors the `d4f353d9`/`b9deddbc` "module absent" precedent.

use babylon_client::engine_link::EngineSession;
use babylon_client::ui::beats::{BeatFeedText, BEAT_LOG_CAPACITY, COUNTIES_VALIDATED_HORIZON};
use babylon_client::ui::time::RunState;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

/// Presses `key` through the REAL `KeyboardInput` message pipeline — see
/// every other test file in this crate's own module docs for the full
/// citation.
fn press_key_via_real_event(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<Messages<KeyboardInput>>()
        .write(KeyboardInput {
            key_code: key,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
}

fn release_key(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

/// The real app: `MapPlugin` + `TickLoopPlugin` together, on the counties
/// story (`main.rs`'s own default, explicitly threaded here — B3 wave-1
/// Task 5, plan §2.5 Minor 7: `SelectedStory` has no `Default`).
fn new_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(babylon_client::story::SelectedStory(
        babylon_client::story::counties(),
    ));
    app
}

fn beat_feed_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<BeatFeedText>>();
    query
        .single(world)
        .expect("exactly one beat feed entity")
        .0
        .clone()
}

fn step_one_tick(app: &mut App) {
    press_key_via_real_event(app, KeyCode::Space);
    app.update();
    release_key(app, KeyCode::Space);
}

// ---- The wired feed: a real sentence, tick-stamped, never the terse form ----

#[test]
fn legitimation_recovery_renders_a_tick_stamped_sentence_not_the_terse_at_format() {
    let mut app = new_app();
    app.update(); // Startup
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));

    // Task 7's own recovering-county archetype (county-01013/01015/01017)
    // fires LEGITIMATION_RECOVERY on tick 1 (`us_counties_demo.rs`).
    step_one_tick(&mut app);

    let text = beat_feed_text(&mut app);
    assert!(
        text.contains("legitimation recovers"),
        "the feed must surface the transcribed sentence \
         (chronicle_adapter.py's own wording), got {text:?}"
    );
    assert!(
        text.contains("tick 1: 01013 legitimation recovers"),
        "the beat must be tick-stamped and lead with the resolved subject, got {text:?}"
    );
    assert!(
        !text.contains("LEGITIMATION_RECOVERY @ 01013"),
        "the terse retired format must not appear — the beat must render its \
         transcribed sentence instead, got {text:?}"
    );
}

// ---- The collapse rule: same-tick LIFECYCLE_TRANSITIONs, one line, a real count + magnitude ----

/// Minor 1's own invariant (§3.3): the collapsed `LIFECYCLE_TRANSITION`
/// count equals THAT tick's `per_rule_fired["lifecycle/dpd-circuit"]` —
/// true ONLY because that rule's emit is unconditional and one-per-subject
/// (`lifecycle.bsl:388-393`); a future rule that emits the same event type
/// conditionally would correctly break this equality.
#[test]
fn same_tick_lifecycle_transitions_collapse_to_one_line_with_a_real_count_and_magnitude() {
    let mut app = new_app();
    app.update(); // Startup
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));

    // Two ticks: the first seeds the per-territory Σ|Δ| tracker (honestly
    // 0.0 on first sighting — no prior tick to diff against), the second
    // is where the magnitude term is a REAL nonzero delta.
    // `TickReport` carries no `Clone` (its fixed-size hash arrays are the
    // reason, `babylon-tick/src/lib.rs`) — extract just the one count each
    // tick's report holds instead of cloning the whole struct.
    let dpd_fired_after = |app: &App| -> usize {
        app.world()
            .resource::<babylon_client::ui::admin::LastTickReport>()
            .0
            .as_ref()
            .expect("a TickReport must exist by this point")
            .per_rule_fired
            .iter()
            .find(|(rule, _)| rule == "lifecycle/dpd-circuit")
            .map(|(_, count)| *count)
            .expect("lifecycle/dpd-circuit must have fired")
    };

    step_one_tick(&mut app);
    let dpd_fired_tick_1 = dpd_fired_after(&app);
    step_one_tick(&mut app);
    let dpd_fired = dpd_fired_after(&app);
    assert_eq!(
        dpd_fired_tick_1, dpd_fired,
        "the demo's 12 territories fire lifecycle/dpd-circuit every tick, identically"
    );

    let text = beat_feed_text(&mut app);
    let collapsed_line = text
        .lines()
        .find(|line| line.contains("D-P-D") && line.contains("tick 2"))
        .unwrap_or_else(|| panic!("no tick-2 collapsed LIFECYCLE_TRANSITION line in {text:?}"));
    assert!(
        collapsed_line.contains(&format!("{dpd_fired} territories")),
        "the collapsed line's count must equal per_rule_fired[\"lifecycle/dpd-circuit\"] \
         ({dpd_fired}), got {collapsed_line:?}"
    );
    assert!(
        !collapsed_line.contains("= 0.0)"),
        "tick 2's magnitude must be a REAL nonzero Σ|Δpop-d-prime| (the territories moved \
         between ticks 1 and 2), got {collapsed_line:?}"
    );
}

// ---- The BeatLog stays bounded; the sink is drained (#503) ----

#[test]
fn beat_log_never_exceeds_capacity_after_a_long_auto_run_and_the_sink_stays_drained() {
    let mut app = new_app();
    app.update(); // Startup — running = true, autopause = OnCritical by default.
    app.world_mut().resource_mut::<RunState>().autopause =
        babylon_client::ui::time::AutopauseMode::Never;

    // 12 LIFECYCLE_TRANSITIONs/tick x ~60 ticks > BEAT_LOG_CAPACITY (512) —
    // enough batches to prove the cap holds, bounded at 40 update() calls
    // (Power-of-10 rule 2: a real, finite test-loop bound).
    for _ in 0..40 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            1600,
        )));
        app.update();
    }

    let log = app.world().resource::<babylon_client::ui::beats::BeatLog>();
    assert!(
        log.beats.len() <= BEAT_LOG_CAPACITY,
        "BeatLog must never exceed its capacity ({BEAT_LOG_CAPACITY}), got {}",
        log.beats.len()
    );

    let session = app.world().resource::<EngineSession>();
    assert!(
        session.sink.events.is_empty(),
        "the sink must be drained every tick (#503) — it must never accumulate, \
         got {} stale events",
        session.sink.events.len()
    );
}

// ---- §2.7/I3: the counties story stays numerically sane to its measured horizon ----

/// Measured at implementation, not assumed: across `COUNTIES_VALIDATED_HORIZON`
/// ticks, every listed field stays finite and non-negative. This test runs
/// the bare `EngineSession` directly (no Bevy needed — a pure engine-level
/// numeric-sanity claim), the same `EngineSession::start(story)`/`.advance()`
/// idiom `tests/time_controls.rs`'s row 6 already uses.
#[test]
fn counties_stay_numerically_sane_to_the_validated_horizon() {
    use babylon_graph::substrate::GraphSubstrate;

    let mut session =
        EngineSession::start(babylon_client::story::counties()).expect("counties session starts");
    // Loop bound: COUNTIES_VALIDATED_HORIZON, a compile-time const (Power-of-10 rule 2).
    for tick in 1..=COUNTIES_VALIDATED_HORIZON {
        session
            .advance()
            .unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        let graph = session.inner.graph();
        for (fips, id) in &session.roster {
            for field in [
                "territory/pop-d",
                "territory/pop-p",
                "territory/pop-d-prime",
                "territory/wealth-d-prime",
                "territory/dependency-ratio",
            ] {
                let value = graph
                    .node_attribute(*id, field)
                    .unwrap_or_else(|e| panic!("tick {tick} {fips} {field}: {}", e.message));
                assert!(
                    value.is_finite(),
                    "tick {tick} {fips} {field} went non-finite: {value}"
                );
                assert!(
                    value >= 0.0,
                    "tick {tick} {fips} {field} went negative: {value}"
                );
            }
        }
        // vitality.bsl's own six SOCIAL_CLASS fixture nodes — not on
        // `roster` (that map is territory-only), so queried directly.
        for id in graph.nodes("SOCIAL_CLASS") {
            for field in ["social-class/population", "social-class/wealth"] {
                let value = graph
                    .node_attribute(id, field)
                    .unwrap_or_else(|e| panic!("tick {tick} {id:?} {field}: {}", e.message));
                assert!(
                    value.is_finite(),
                    "tick {tick} {id:?} {field} went non-finite: {value}"
                );
                assert!(
                    value >= 0.0,
                    "tick {tick} {id:?} {field} went negative: {value}"
                );
            }
        }
    }
}
