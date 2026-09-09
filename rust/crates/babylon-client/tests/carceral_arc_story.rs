//! Task 7 of the B3 null-hypothesis-viewer train (plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.8/§2.9, task-7-brief.md): the carceral arc as the first shipped
//! story, proved three separate ways.
//!
//! - **G1 — the engine-unchanged proof.** A fresh, independent
//!   `TickSession` advanced 110 times, with its own sink, is the REAL
//!   oracle — no blessed constant, computed live in this same test. Its
//!   per-tick `TickReport.after` sequence and final `state_hash` must be
//!   byte-identical to the real client `App`'s own.
//! - **G2 — the drift alarm.** The tick-110 hash, pinned as a `const`
//!   here (never in `babylon-tick` — Global Constraint 6), its value taken
//!   from G1's own independent session.
//! - **G3 — the beat conformance.** Restated over the *rendered feed* what
//!   `babylon-tick/tests/carceral_arc_conformance.rs` asserts about the
//!   engine: the four beats at ticks 1/53/105/106, each exactly once and
//!   four total, `TERMINAL_DECISION`'s numeric GENOCIDE encoding, and the
//!   latch card rendering with the run paused at tick 106 and no verdict
//!   vocabulary on screen (Task 4.5's negative assertion, restated
//!   end-to-end).
//!
//! Plus a text golden (§2.8's third test layer): the rendered beat feed
//! for the whole 110-tick run, byte-for-byte.
//!
//! Headless real `App`, real `KeyboardInput`, virtual time only (I4) — the
//! same house pattern every other test file in this crate uses
//! (`tests/autopause.rs`'s own module doc names it in full).

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_client::engine_link::EngineSession;
use babylon_client::loop_ui::TickCounter;
use babylon_client::story;
use babylon_client::ui::admin::LastTickReport;
use babylon_client::ui::beats::{BeatFeedText, LatchCardText};
use babylon_client::ui::time::{AutopauseMode, RunState};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::state_hash::CanonicalState;
use babylon_kernel::SessionId;
use babylon_tick::TickSession;
use bevy::asset::AssetPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use std::time::Duration;

/// The last tick this whole file ever drives to — matches
/// `babylon-tick/tests/carceral_arc_conformance.rs::LAST_TICK` (110,
/// "comfortably past the derived `TERMINAL_DECISION` tick (106), with margin
/// to prove nothing fires a fifth time afterward") and
/// `story::carceral().validated_horizon`.
const LAST_TICK: i64 = 110;

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

/// Builds a real App launched directly on the carceral story — exactly
/// `main.rs`'s own `--story carceral` wiring, matching `tests/autopause.rs`'s
/// own `new_carceral_app`. `RunState.speed_index` is pinned to `0` (1 t/s,
/// interval exactly `1.0/1.0 = 1.0` seconds — no fractional division, so
/// zero rounding-error risk the way any OTHER speed/duration pairing in
/// this crate's own `ticks_due` doc names as a real hazard) and
/// `autopause` to `Never` — this file's own driving helper
/// (`advance_one_tick_via_autorun`) depends on EXACTLY one tick firing per
/// `app.update()` call, which only `TERMINAL_DECISION`'s own UNCONDITIONAL
/// pause (never gated on `AutopauseMode`) may interrupt.
fn new_carceral_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.insert_resource(story::SelectedStory(story::carceral()));
    app.update(); // Startup — spawns the real carceral EngineSession.
    {
        let mut run_state = app.world_mut().resource_mut::<RunState>();
        run_state.autopause = AutopauseMode::Never;
        run_state.speed_index = 0;
    }
    app
}

/// Advances the real App by exactly one tick via the auto-run clock —
/// never `Space`, never a hand-rolled reimplementation of `advance_ticks`.
/// Valid only paired with `new_carceral_app`'s own speed/duration choice
/// (interval `1.0s` == the injected duration below, so `ticks_due` always
/// resolves to exactly `1`, zero leftover). Returns that one tick's own
/// `TickReport.after`, read back off [`LastTickReport`] — `advance_ticks`
/// binds it every tick, so after a single-tick frame it holds exactly this
/// call's own report.
///
/// # Panics
/// If the frame advanced zero ticks (a wiring bug in this helper's own
/// speed/duration pairing, or a caller that let the run stay paused) —
/// loud, not a silently-stale report.
fn advance_one_tick_via_autorun(app: &mut App) -> [u8; 32] {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(1)));
    app.update();
    app.world()
        .resource::<LastTickReport>()
        .0
        .as_ref()
        .unwrap_or_else(|| panic!("advance_one_tick_via_autorun: no tick fired this frame"))
        .after
}

/// Drives `app` through the whole 110-tick arc via the real auto-run clock,
/// collecting each tick's own `TickReport.after` in firing order.
/// `TERMINAL_DECISION` pauses the run UNCONDITIONALLY at tick 106
/// (`ui::time::advance_ticks`'s own `outcome.terminal_decision ||` — never
/// gated on `AutopauseMode`, so `AutopauseMode::Never` does not skip it) —
/// this asserts that pause is real, then presses `P` once to resume past
/// it (`ManualDuration::ZERO` on that one frame, matching
/// `tests/autopause.rs`'s own documented reason: a stale nonzero strategy
/// left active on the resume frame would batch several more ticks on the
/// SAME frame `running` flips true on). Every other tick fires purely from
/// the clock — this is the only key this whole file ever presses.
fn drive_full_arc(app: &mut App) -> Vec<[u8; 32]> {
    let mut afters = Vec::new();
    for expected_tick in 1..=LAST_TICK {
        if expected_tick == 107 {
            app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
            press_key_via_real_event(app, KeyCode::KeyP);
            app.update();
            release_key(app, KeyCode::KeyP);
            assert!(
                app.world().resource::<RunState>().running,
                "P must resume the run after the tick-106 latch"
            );
        }
        afters.push(advance_one_tick_via_autorun(app));
        assert_eq!(
            app.world().resource::<TickCounter>().0,
            expected_tick,
            "auto-run must advance EXACTLY one tick per call under this speed/duration pairing"
        );
        if expected_tick == 106 {
            assert!(
                !app.world().resource::<RunState>().running,
                "TERMINAL_DECISION must pause the run unconditionally at tick 106, even under \
                 AutopauseMode::Never"
            );
        }
    }
    afters
}

fn beat_feed_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<BeatFeedText>>();
    query.single(world).map(|t| t.0.clone()).unwrap_or_default()
}

fn latch_card_text(app: &mut App) -> String {
    let world = app.world_mut();
    let mut query = world.query_filtered::<&Text, With<LatchCardText>>();
    query.single(world).map(|t| t.0.clone()).unwrap_or_default()
}

/// §4.5's own negative assertion (`tests/autopause.rs`), restated
/// end-to-end in this file: the five canonical campaign outcomes, and the
/// words "verdict"/"campaign", must never appear. `GENOCIDE`/`REVOLUTION`
/// are NOT in this list on purpose — they are `control-ratio.bsl`'s own
/// numeric-outcome encoding NAMES, a subsystem citation, not the
/// Director-reserved five-outcome campaign vocabulary
/// (`ui::beats::format_latch_card`'s own I2 fix; `narration.rs`'s
/// `TERMINAL_DECISION_GENOCIDE` template).
const FORBIDDEN_VERDICT_VOCABULARY: [&str; 7] = [
    "REVOLUTIONARY_VICTORY",
    "FASCIST_CONSOLIDATION",
    "RED_OGV",
    "ECOLOGICAL_COLLAPSE",
    "FRAGMENTED_COLLAPSE",
    "verdict",
    "campaign",
];

fn assert_no_verdict_vocabulary(text: &str) {
    for forbidden in FORBIDDEN_VERDICT_VOCABULARY {
        assert!(
            !text.to_lowercase().contains(&forbidden.to_lowercase()),
            "must never render {forbidden:?} — got {text:?}"
        );
    }
}

// ---------------------------------------------------------------------
// 7.1 — the golden.
// ---------------------------------------------------------------------

/// The rendered beat feed for the whole 110-tick carceral run, byte-for-
/// byte. Filled from the first green run and pinned thereafter (the
/// `rng.rs:191-192` precedent — a later divergence is a regression, never
/// "the text improved"). `FEED_DEPTH` (`ui/beats.rs`) is 10 and this arc
/// only ever fires 4 beats total, so nothing scrolls out of the visible
/// window across the whole run — the final rendered feed genuinely holds
/// every beat the 110-tick run ever produced, `because:` lines included.
///
/// The golden file carries exactly one trailing newline (the repo's own
/// `end-of-file-fixer` pre-commit hook enforces this on every text file it
/// touches) while `format_beat_feed`'s own `.join("\n")` never emits a
/// trailing one — `strip_suffix` removes exactly that one mandated byte
/// before comparing, so the assertion stays genuinely byte-for-byte on
/// every byte the RENDER controls.
#[test]
fn the_carceral_arc_beat_feed_matches_the_golden_byte_for_byte() {
    let mut app = new_carceral_app();
    drive_full_arc(&mut app);
    let feed = beat_feed_text(&mut app);
    let golden = include_str!("goldens/carceral_arc_beats.txt");
    let golden = golden.strip_suffix('\n').unwrap_or(golden);
    assert_eq!(
        feed, golden,
        "the rendered carceral beat feed drifted from the pinned golden — if this is an \
         intentional content change, regenerate tests/goldens/carceral_arc_beats.txt from a \
         fresh green run and record the drift in the PR body; a silent drift is a regression, \
         never \"the text improved\""
    );
}

// ---------------------------------------------------------------------
// 7.2 — the arc through the UI (G3, §2.9).
// ---------------------------------------------------------------------

/// Mirrors `babylon-tick/tests/carceral_arc_conformance.rs::
/// the_full_carceral_arc_runs_in_order`'s own tick assertions, restated
/// over the RENDERED feed rather than the raw event log — this is the
/// viewer's own proof, distinct from the engine's. `format_beat_feed`
/// renders newest-first (`ui/beats.rs::format_beat_feed`'s own doc), so
/// "in tick order" is a REVERSED position check: tick 106's own line
/// starts EARLIEST in the string, tick 1's own line starts LAST.
#[test]
fn the_four_carceral_beats_appear_in_the_feed_in_tick_order() {
    let mut app = new_carceral_app();
    drive_full_arc(&mut app);
    let feed = beat_feed_text(&mut app);

    let pos = |needle: &str| {
        feed.find(needle)
            .unwrap_or_else(|| panic!("{needle:?} must appear in the rendered feed, got {feed:?}"))
    };
    let p1 = pos("tick 1: ");
    let p53 = pos("tick 53: ");
    let p105 = pos("tick 105: ");
    let p106 = pos("tick 106: ");
    assert!(
        p106 < p105 && p105 < p53 && p53 < p1,
        "newest-first rendering must show tick 106 first and tick 1 last — got offsets \
         106={p106} 105={p105} 53={p53} 1={p1} in {feed:?}"
    );
}

/// Mirrors `carceral_arc_conformance.rs::the_arc_emits_each_event_exactly_once`:
/// each of the four beats appears exactly once, and exactly four beat
/// lines total, in the rendered feed. A beat headline line always starts
/// with `"tick "` (`ui/beats.rs::format_single_beat`'s own format string);
/// a `because:` continuation line never does, so counting lines with that
/// prefix counts BEATS, not lines.
#[test]
fn each_carceral_beat_appears_exactly_once_and_four_total() {
    let mut app = new_carceral_app();
    drive_full_arc(&mut app);
    let feed = beat_feed_text(&mut app);

    let headline_lines: Vec<&str> = feed.lines().filter(|l| l.starts_with("tick ")).collect();
    assert_eq!(
        headline_lines.len(),
        4,
        "exactly four beat headlines total, got {headline_lines:?}"
    );
    for needle in ["tick 1: ", "tick 53: ", "tick 105: ", "tick 106: "] {
        let count = feed.matches(needle).count();
        assert_eq!(
            count, 1,
            "{needle:?} must appear exactly once, got {count} in {feed:?}"
        );
    }
}

/// Mirrors `carceral_arc_conformance.rs::the_arc_ends_in_genocide_with_no_organization`'s
/// own `outcome == Value::Int(0)` assertion, restated through the rendered
/// latch card: `format_latch_card`'s own dispatch (`ui/beats.rs`) only
/// prints `"outcome 0"` paired with `"numeric GENOCIDE encoding"` when the
/// payload's `outcome` key really is `Value::Int(0)` — anything else (a
/// missing key, a different int, a non-int `Value`) routes through the
/// honest not-computed class instead (I2), so this render IS the
/// end-to-end proof, not a restatement of the engine's own assertion.
/// Also proves the run is paused at tick 106 with no verdict vocabulary on
/// screen (Task 4.5's negative assertion, restated end-to-end).
#[test]
fn terminal_decision_renders_the_genocide_encoding_and_pauses_with_no_verdict_vocabulary() {
    let mut app = new_carceral_app();
    drive_full_arc(&mut app);
    // drive_full_arc's own tick-106 assertion already proved `!running` at
    // that moment and resumed past it — re-read the FINAL latch card here
    // (BeatLog is never cleared mid-run, so the card, once rendered, stays
    // populated for the rest of this run).
    let card = latch_card_text(&mut app);
    assert!(
        !card.is_empty(),
        "the latch card must render on TERMINAL_DECISION"
    );
    assert!(card.contains("outcome 0"), "got {card:?}");
    assert!(
        card.contains("numeric GENOCIDE encoding"),
        "outcome must be the verified numeric GENOCIDE encoding, got {card:?}"
    );
    assert!(!card.contains("REVOLUTION"), "got {card:?}");
    assert_no_verdict_vocabulary(&card);
}

// ---------------------------------------------------------------------
// 7.3 — the engine-unchanged proof (G1, §2.9) + G2's pin.
// ---------------------------------------------------------------------

/// **G2.** Measured from G1's own independent session below (a fresh
/// `TickSession` advanced 110 times over the carceral scenario + joined
/// decomposition/control-ratio rules), 2026-08-18. Mutation-proven per the
/// task brief: flipping one hex digit reds this assertion (reverted after
/// confirming); deleting one `advance()` call from the oracle loop below
/// also reds it (reverted after confirming) — both recorded in the PR
/// body, neither left in this file.
const TICK_110_HASH: &str = "bd5fe4d5b2a982f2bd1191f14bef6a623e0d14a3e2563f882964d03736856dcc";

/// **G1.** A fresh, independent `TickSession` — the REAL oracle, computed
/// live in this same test, no blessed constant — advanced 110 times with
/// its own sink. Its per-tick `TickReport.after` sequence and final
/// `state_hash` must be byte-identical to the real client `App`'s own,
/// driven entirely through the production `advance_ticks` auto-run path
/// above. If the client ever perturbs the engine (a stray write, a
/// duplicated advance, a re-ordered rule load) this reds immediately.
#[test]
fn the_client_apps_tick_sequence_matches_a_fresh_independent_oracle() {
    let mut app = new_carceral_app();
    let client_afters = drive_full_arc(&mut app);
    assert_eq!(
        client_afters.len(),
        usize::try_from(LAST_TICK).expect("LAST_TICK is positive")
    );

    let carceral = story::carceral();
    let rule_src = carceral.rule_srcs.join("\n");
    let mut oracle = TickSession::new(
        carceral.scenario_src,
        &rule_src,
        HypergraphStore::new(),
        SessionId::new(carceral.session_id).expect("carceral session id is non-empty"),
    )
    .expect("independent oracle session starts");
    let mut oracle_afters = Vec::new();
    for _ in 1..=LAST_TICK {
        let mut sink = CollectingSink::default();
        let report = oracle.advance(&mut sink).expect("oracle tick advances");
        oracle_afters.push(report.after);
    }

    assert_eq!(
        client_afters, oracle_afters,
        "G1: the client App's own per-tick TickReport.after sequence must be byte-identical to \
         a fresh independent TickSession's — any divergence means the viewer perturbed the \
         engine"
    );

    let oracle_hash = CanonicalState::state_hash(oracle.graph()).expect("oracle graph hashes");
    let client_hash =
        CanonicalState::state_hash(app.world().resource::<EngineSession>().inner.graph())
            .expect("client graph hashes");
    assert_eq!(
        oracle_hash, client_hash,
        "G1: the final state_hash must also match"
    );

    assert_eq!(
        babylon_tick::hex(&oracle_hash),
        TICK_110_HASH,
        "G2: the tick-110 hash pin drifted — see TICK_110_HASH's own doc comment"
    );
}

/// The C4 interaction (§2.9's own "one honest caveat, made executable"):
/// G1 above pins the client and the oracle to the SAME `SessionId`, so
/// their equality holds by construction — this is the SEPARATE question of
/// whether the session id could change the run at all. A witness, not a
/// law: the day a landed pack calls `rng-draw` this goes red, which is the
/// correct and loud alarm that the session-id choice has become
/// load-bearing. Do NOT delete it then — re-anchor it.
#[test]
fn the_story_session_id_does_not_yet_move_the_run() {
    let carceral = story::carceral();
    let rule_src = carceral.rule_srcs.join("\n");
    let run_to_hash = |session_id: &str| -> [u8; 32] {
        let mut session = TickSession::new(
            carceral.scenario_src,
            &rule_src,
            HypergraphStore::new(),
            SessionId::new(session_id).expect("non-empty literal"),
        )
        .expect("session starts");
        let mut sink = CollectingSink::default();
        for _ in 1..=LAST_TICK {
            session.advance(&mut sink).expect("tick advances");
        }
        CanonicalState::state_hash(session.graph()).expect("graph hashes")
    };

    let with_the_real_id = run_to_hash(carceral.session_id);
    let with_a_different_id = run_to_hash("a-completely-different-session-id");
    assert_eq!(
        with_the_real_id, with_a_different_id,
        "the same 110-tick content under two different SessionIds must hash identically TODAY \
         (no landed pack calls rng-draw, rg-verified against rust/crates/babylon-tick/content/\
         rules/) — if this ever goes red, the session-id choice has become load-bearing"
    );
}
