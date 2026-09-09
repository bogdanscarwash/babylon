//! The clock, the pacing defaults, and the virtual-time discipline (B3
//! wave-1 Task 2, plan `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.1/§2.4). Owns `RunState`, the fixed speed table, the bounded
//! catch-up loop's pure arithmetic core (`ticks_due`), the phase-locked
//! heartbeat (`TickPhase`) and the controls readout — `advance_ticks`
//! (below) replaces `loop_ui::advance_on_space` as the crate's ONE
//! tick-advancing system; `loop_ui.rs`'s own `.chain()` registers it in
//! exactly the position `advance_on_space` held, so the two ordering
//! fixes recorded there (the `LensChanged`/HUD repaint lag) keep working
//! unchanged, just renamed.
//!
//! **I4 — the virtual-time discipline.** `time.delta_secs()` is read in
//! exactly one place in this whole crate: the line inside `advance_ticks`
//! that adds it to `RunState.accumulator`. Everything downstream of that
//! line — `ticks_due`, `TickPhase`, the readout — is pure arithmetic over
//! that accumulator, never a second read of the clock. Tests drive this
//! deterministically via `bevy::time::TimeUpdateStrategy::ManualDuration`,
//! never the wall clock (memory `program-15-gauntlet`: "wall-clock tests
//! = determinism poison").
//!
//! **Task 2's own staged GREEN split (history, not a TODO — both landed).**
//! The 2.3 commit built the mechanics — `RunState`, the speed table,
//! `ticks_due`, `TickPhase`, `advance_ticks`, the readout entity and its
//! TEXT. The 2.4 commit layered the heartbeat's three-discrete-step
//! palette COLOR (`heartbeat_color`) onto that same readout without
//! touching anything 2.3 built — kept as two commits so each one's own
//! test rows could go green independently (`tests/time_controls.rs`'s
//! `..._steps_through_three_discrete_palette_colors...` row is the one
//! that stayed red through 2.3 and only 2.4 makes pass).

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::engine_link::EngineSession;
use crate::loop_ui::TickCounter;
use bevy::prelude::*;

/// Whether/how an auto-run should stop itself at a critical beat. Only
/// `OnCritical` is wired ANYWHERE this wave — the event-feed severity
/// path that would actually trigger a stop (plan §2.2, a later task's own
/// deliverable) does not exist yet. `Never` exists so the type is a real,
/// two-member choice rather than a single-variant placeholder standing in
/// for a bool — spelled `Never`, not `Off`, per the master plan's own
/// enum (plan line 923: `AutopauseMode` ∈ `{ Never, OnCritical }`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutopauseMode {
    Never,
    OnCritical,
}

/// The clock's own state — one plain resource, no Bevy `States` machine
/// (§2.1's own rejected alternative: a state-gated schedule would pause
/// every render/HUD system too, not just the advance, and no `States`
/// machine exists anywhere else in this crate). `accumulator` is
/// sim-domain seconds-until-the-next-tick, written ONLY by
/// [`advance_ticks`] — the sole place in this crate that reads
/// [`bevy::time::Time::delta_secs`] (I4).
///
/// **Defaults are a design decision, not an implementation detail**
/// (GDS §3, `docs/superpowers/specs/2026-07-29-game-design-standard-design.md:65`):
/// `running = true`, `autopause = OnCritical`, `speed_index = 2` (5 t/s)
/// — autoplay-until-event. A fresh run starts moving and stops itself at
/// the first critical beat; it never waits for a keypress it never told
/// anyone about.
#[derive(Resource, Debug, Clone, Copy)]
pub struct RunState {
    pub running: bool,
    pub speed_index: usize,
    pub accumulator: f32,
    pub autopause: AutopauseMode,
}

impl Default for RunState {
    fn default() -> Self {
        Self {
            running: true,
            speed_index: 2,
            accumulator: 0.0,
            autopause: AutopauseMode::OnCritical,
        }
    }
}

/// The fixed speed table `,`/`.` step through — index 2 (5 t/s) is
/// `RunState::default`'s own starting point.
pub const SPEEDS_PER_SECOND: [f32; 5] = [1.0, 2.0, 5.0, 10.0, 25.0];

/// The catch-up loop's own bound (§2.1) — a stalled frame (window drag,
/// breakpoint) can advance at most this many ticks in one call, and the
/// leftover backlog is itself clamped to at most one more batch's worth
/// (see [`ticks_due`]'s own doc) — never fast-forwarding further no
/// matter how long the stall was.
pub const MAX_TICKS_PER_FRAME: usize = 8;

/// The bounded catch-up loop's pure arithmetic core (§2.1) — zero Bevy,
/// stated entirely in tick-domain terms. Consumes `interval`-sized chunks
/// out of `accumulator`, at most `max` of them, and returns
/// `(ticks_due_this_call, leftover_accumulator)`.
///
/// **A single division, never `max` repeated subtractions.** A first
/// implementation subtracted `interval` from `accumulator` in a loop and
/// undercounted: 1.6s of accumulated time at a 0.2s interval landed on 7
/// ticks, not 8 — `f32`'s rounding of `1.0 / 5.0` carries a ~3e-8 error
/// that seven successive subtractions compound until the eighth
/// comparison falls just short (mutation-proof: this exact case is
/// `tests/time_controls.rs`'s
/// `a_catch_up_frame_of_more_than_one_tick_shows_the_batch_size_in_the_readout`).
/// One division and one multiplication carry at most a single rounding
/// step each, which is why this shape is exact on every case this crate
/// exercises: `accumulator / interval`, truncated toward zero (exactly
/// `floor` for the non-negative values this function ever sees) and
/// capped at `max`, is `due`; `due * interval` subtracted back out of
/// `accumulator` is the leftover.
///
/// This is statically bounded by construction (Power-of-10 rule 2) —
/// zero loops, not merely a bounded one — and a non-positive
/// `accumulator` is sanitized to `0.0` up front, so it yields zero ticks
/// without a special case.
///
/// The leftover is clamped to `max * interval` even when fewer than
/// `max` ticks were due (a no-op in that case, since the natural leftover
/// is already smaller) — so a stalled frame's backlog can never exceed
/// one more full batch, no matter how large `accumulator` was on entry.
#[must_use]
pub fn ticks_due(accumulator: f32, interval: f32, max: usize) -> (usize, f32) {
    let accumulator = accumulator.max(0.0);
    let due_unclamped = accumulator / interval;
    // Truncating a non-negative f32 toward zero (Rust's `as` cast is a
    // saturating truncation, never UB) is exactly `floor` here — this
    // crate never calls `ticks_due` with a negative or NaN quotient.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let due = (due_unclamped as usize).min(max);
    // `due`/`max` are MAX_TICKS_PER_FRAME-scale (8) at every real call
    // site — trivially exact in f32 (23-bit mantissa covers every usize
    // this crate will ever pass here).
    #[allow(clippy::cast_precision_loss)]
    let consumed = due as f32 * interval;
    // `.max(0.0)`: `consumed` can overshoot `accumulator` by a single
    // rounding step when `due_unclamped` rounds fractionally above the
    // true quotient — never by more than one ULP, but real regardless.
    let remaining = (accumulator - consumed).max(0.0);
    #[allow(clippy::cast_precision_loss)]
    let ceiling = max as f32 * interval;
    (due, remaining.min(ceiling))
}

/// The auto-run clock's phase within the CURRENT interval,
/// `accumulator / interval` clamped to `[0, 1]` — the heartbeat's own
/// timing source (S2), consumed by 2.4's palette-stepping. Frozen
/// (left untouched) while `RunState.running` is `false`: "an unmoving
/// clock must look unmoving" (Minor 4).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct TickPhase(pub f32);

/// How many ticks [`advance_ticks`] actually advanced on the MOST RECENT
/// frame — `0` on a frame that advanced nothing (paused, or running with
/// nothing yet due), never anything else. The controls readout reads
/// this to render the honest "+K ticks" catch-up line (Minor 4) instead
/// of a single `tick N` that would understate what just happened.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct LastBatch(pub usize);

/// `Update` system: the crate's ONE tick-advancing system (§2.1),
/// replacing `loop_ui::advance_on_space`. Folds three inputs into one
/// bounded advance path:
///
/// 1. The `P`/`,`/`.` bindings steer `RunState` itself (play/pause,
///    speed) — handled first, so the SAME frame a binding fires already
///    sees its own effect.
/// 2. `Space` always advances exactly one tick, whether paused or
///    running, and RESETS the accumulator — a manual step absorbs
///    whatever auto-run backlog this same frame also accumulated rather
///    than advancing both (no double-advance).
/// 3. Otherwise, while running, [`ticks_due`] says how many ticks
///    (`0..=MAX_TICKS_PER_FRAME`) are due this frame.
///
/// The lens recompute and `LensChanged` fire at most ONCE per frame,
/// after the whole batch — never once per tick (the crate's own
/// expensive-recolor rationale, `loop_ui.rs`'s module doc).
///
/// **B3 wave-1 Task 3 (plan §3.3).** [`crate::ui::admin::LastTickReport`]
/// binds the `TickReport` this loop already computes every iteration and
/// used to discard — ending the batch holding exactly the LAST tick's
/// report, zero new computation.
///
/// # Panics
/// If [`EngineSession::advance`] fails (an intrinsic, scenario, or rule
/// error) — the same loud-failure contract `loop_ui::advance_on_space`
/// held before this system replaced it.
// Every parameter is a distinct, narrow Bevy `SystemParam`
// (`Res`/`ResMut`/`MessageWriter`) the scheduler inspects individually for
// parallel-access analysis — the same shape `map/hud.rs::refresh_hud`'s
// own allow already covers for this crate; see that comment for the full
// rationale against a `#[derive(SystemParam)]` wrapper struct.
/// Advances `session` by one tick, drains its sink into `log` (tagged with
/// the new tick, §2.2), binds the `TickReport` into `last_tick_report`,
/// and reports whether this ONE tick's own drain requires an autopause
/// (§3.6/C2): unconditionally on a `TERMINAL_DECISION`, or on any
/// `critical` beat when `autopause == OnCritical`.
///
/// # Panics
/// If [`EngineSession::advance`] fails (an intrinsic, scenario, or rule
/// error) — the same loud-failure contract this function's caller has
/// always held.
fn advance_one_tick_and_drain(
    session: &mut EngineSession,
    log: &mut crate::ui::beats::BeatLog,
    last_tick_report: &mut crate::ui::admin::LastTickReport,
    autopause: AutopauseMode,
) -> bool {
    let report = session
        .advance()
        .unwrap_or_else(|e| panic!("tick advance failed: {e}"));
    let tick = session.inner.tick();
    last_tick_report.0 = Some(report);
    let outcome = crate::ui::beats::drain_tick_into_beat_log(session, tick, log);
    outcome.terminal_decision || (autopause == AutopauseMode::OnCritical && outcome.any_critical)
}

#[allow(clippy::too_many_arguments)]
pub fn advance_ticks(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut run_state: ResMut<RunState>,
    mut session: ResMut<EngineSession>,
    mut counter: ResMut<TickCounter>,
    mut lens_data: ResMut<crate::lens::CurrentLensData>,
    mut lens_changed: MessageWriter<crate::map::LensChanged>,
    mut hud_tick: ResMut<crate::map::HudTick>,
    mut tick_phase: ResMut<TickPhase>,
    mut last_batch: ResMut<LastBatch>,
    mut last_tick_report: ResMut<crate::ui::admin::LastTickReport>,
    mut beat_log: ResMut<crate::ui::beats::BeatLog>,
) {
    if keys.just_pressed(KeyCode::KeyP) {
        run_state.running = !run_state.running;
    }
    if keys.just_pressed(KeyCode::Comma) {
        run_state.speed_index = run_state.speed_index.saturating_sub(1);
    }
    if keys.just_pressed(KeyCode::Period) {
        run_state.speed_index = (run_state.speed_index + 1).min(SPEEDS_PER_SECOND.len() - 1);
    }
    // B3 wave-1 Task 4 (§3.6/C2): `B` = run-to-next-beat — resumes running
    // with autopause forced to `OnCritical`, the mode that actually stops
    // the batch below. Not a new advance path: the ordinary catch-up loop
    // already does the stopping; this binding only guarantees the two
    // preconditions it needs.
    if keys.just_pressed(KeyCode::KeyB) {
        run_state.running = true;
        run_state.autopause = AutopauseMode::OnCritical;
    }

    let space_pressed = keys.just_pressed(KeyCode::Space);

    if run_state.running {
        run_state.accumulator += time.delta_secs();
    }
    let interval = 1.0 / SPEEDS_PER_SECOND[run_state.speed_index];

    let batch_size = if space_pressed {
        run_state.accumulator = 0.0;
        1
    } else if run_state.running {
        let (due, remainder) = ticks_due(run_state.accumulator, interval, MAX_TICKS_PER_FRAME);
        run_state.accumulator = remainder;
        due
    } else {
        0
    };

    if run_state.running {
        tick_phase.0 = (run_state.accumulator / interval).clamp(0.0, 1.0);
    }
    last_batch.0 = batch_size;

    if batch_size == 0 {
        return;
    }

    // B3 wave-1 Task 3 (plan §3.3): binds the `TickReport` this loop used
    // to discard — `LastTickReport` ends the batch holding exactly the
    // LAST tick's report, which is the tick the rest of this frame's HUD
    // is also showing. B3 wave-1 Task 4 (§2.2/§3.6): also drains the sink
    // into `BeatLog` every tick and STOPS the batch the moment a tick
    // requires an autopause — never advancing further ticks this frame
    // once that happens (§3.6: "advance_ticks stops the batch the moment
    // a critical beat lands").
    for _ in 0..batch_size {
        let must_pause = advance_one_tick_and_drain(
            &mut session,
            &mut beat_log,
            &mut last_tick_report,
            run_state.autopause,
        );
        if must_pause {
            run_state.running = false;
            // An autopause is a DELIBERATE stop, not a stall — the
            // remainder of this frame's batch never happened and carries
            // no hidden momentum into a later resume (Minor 4's own
            // "an unmoving clock must look unmoving" reasoning, extended
            // to the accumulator): discard it rather than refunding it,
            // so P/B resumes genuinely fresh instead of immediately
            // replaying a multi-tick catch-up burst nobody asked for.
            run_state.accumulator = 0.0;
            break;
        }
    }
    counter.0 = session.inner.tick();
    hud_tick.0 = session.inner.tick();
    // Recompute every registered lens's own LensReading (B3 wave-1 Task 8,
    // §2.10) against the POST-batch graph before firing LensChanged —
    // mirrors spawn_engine_session_and_hud's own wiring rationale
    // (loop_ui.rs), now run once per BATCH rather than once per PRESS.
    // Delegates to `build_lens_data`, the SAME builder
    // `ui::story_card::restart_on_n_key` already uses, rather than
    // re-deriving the `LENSES` loop a third time.
    *lens_data = crate::loop_ui::build_lens_data(&session);
    lens_changed.write(crate::map::LensChanged);
}

#[derive(Component)]
pub struct ControlsReadout;

/// `Startup` system: spawns the controls readout entity. Placed above
/// `loop_ui::TickCounterReadout` (`bottom: 44px` vs. that entity's
/// `bottom: 24px`) so the two never overlap; `refresh_controls_readout`
/// overwrites this placeholder text on the very first `Update` pass of
/// the same frame Startup runs in.
pub fn spawn_controls_readout(mut commands: Commands) {
    commands.spawn((
        Text::new("tick 0"),
        TextColor(crate::palette::DIM),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(44),
            right: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TickTransport),
        ControlsReadout,
    ));
}

/// Renders `RunState`/the live tick/`LastBatch` into the one-line
/// controls readout string — pure and independently testable, so
/// [`refresh_controls_readout`] stays a thin wrapper rather than
/// duplicating this formatting inline (the same discipline
/// `map/hud.rs::format_lens_line` already established for its own
/// render system).
///
/// Three shapes, in priority order: paused (`❚❚ paused · tick N`);
/// running with a multi-tick catch-up batch this frame (`▶ S t/s ·
/// +K ticks` — Minor 4, the readout REPLACES the tick number rather
/// than appending to it, since `TickPhase` can only ever complete one
/// ramp per frame regardless of how many ticks just fired); running,
/// steady state (`▶ S t/s · tick N`).
///
/// # Panics
/// If `run_state.speed_index` is out of bounds for [`SPEEDS_PER_SECOND`]
/// — cannot happen through [`advance_ticks`]'s own saturating `,`/`.`
/// handling, which is the only writer of that field.
#[must_use]
pub fn format_controls_readout(run_state: &RunState, tick: i64, last_batch: usize) -> String {
    if !run_state.running {
        return format!("\u{275a}\u{275a} paused \u{b7} tick {tick}");
    }
    let speed = SPEEDS_PER_SECOND[run_state.speed_index];
    if last_batch > 1 {
        return format!("\u{25b6} {speed:.0} t/s \u{b7} +{last_batch} ticks");
    }
    format!("\u{25b6} {speed:.0} t/s \u{b7} tick {tick}")
}

/// The tick heartbeat's three discrete palette steps (§2.1/§2.4) —
/// `DIM` → `BONE` → `GOLD` on [`TickPhase`]'s own `[0, 1]` range. Three
/// steps, never a continuous fade: the aesthetic ruling (Global
/// Constraint 12) forbids glow/blur, and an alpha or scale animation
/// would read as exactly that. While paused, [`TickPhase`] itself is
/// frozen (never updated — `advance_ticks`'s own doc), so this function
/// re-reading the SAME frozen value every frame is what makes the
/// rendered color freeze too: no separate "is running" branch needed
/// here.
#[must_use]
pub fn heartbeat_color(phase: f32) -> Color {
    if phase < 1.0 / 3.0 {
        crate::palette::DIM
    } else if phase < 2.0 / 3.0 {
        crate::palette::BONE
    } else {
        crate::palette::GOLD
    }
}

/// `Update` system: repaints [`ControlsReadout`]'s text from `RunState`,
/// the live tick and `LastBatch`, and its color from [`TickPhase`] — a
/// thin wrapper over [`format_controls_readout`]/[`heartbeat_color`],
/// never re-deriving either inline.
///
/// **B3 wave-1 Task 6 (plan §2.4: "the HUD shows what it will skip to when
/// a countdown is live").** Appends
/// `crate::ui::countdown::format_beat_hint`'s own live-countdown hint when
/// one exists, leaving [`format_controls_readout`] itself untouched (and
/// its own existing unit tests unchanged) — the composition happens here,
/// in the thin system wrapper, not inside the pure formatter.
pub fn refresh_controls_readout(
    run_state: Res<RunState>,
    counter: Res<TickCounter>,
    last_batch: Res<LastBatch>,
    tick_phase: Res<TickPhase>,
    session: Res<EngineSession>,
    mut readout: Query<(&mut Text, &mut TextColor), With<ControlsReadout>>,
) {
    let Ok((mut text, mut color)) = readout.single_mut() else {
        return;
    };
    let base = format_controls_readout(&run_state, counter.0, last_batch.0);
    let hint =
        crate::ui::countdown::format_beat_hint(session.story, session.inner.graph(), counter.0);
    text.0 = if hint.is_empty() {
        base
    } else {
        format!("{base} \u{b7} {hint}")
    };
    color.0 = heartbeat_color(tick_phase.0);
}

#[cfg(test)]
mod tests {
    use super::ticks_due;

    /// The plan's own worked example (§2.1): "2.5 intervals accumulated
    /// yields exactly 2 ticks and 0.5 remaining."
    #[test]
    fn two_point_five_intervals_yields_two_ticks_and_a_half_remainder() {
        let (ticks, remainder) = ticks_due(2.5, 1.0, 8);
        assert_eq!(ticks, 2);
        assert!((remainder - 0.5).abs() < 1e-6, "got {remainder}");
    }

    /// The plan's own worked example (§2.1): "30 intervals accumulated
    /// yields exactly `max` ticks and a clamped remainder (no
    /// 1,000-tick fast-forward after a stall)." The remainder clamps to
    /// `max * interval`, not to zero — a stalled frame is still allowed
    /// one more full batch's worth of backlog, never more.
    #[test]
    fn thirty_intervals_clamp_to_max_ticks_and_a_clamped_remainder() {
        let (ticks, remainder) = ticks_due(30.0, 1.0, 8);
        assert_eq!(
            ticks, 8,
            "a stalled frame must never advance past `max` ticks in one call"
        );
        assert!(
            (remainder - 8.0).abs() < 1e-6,
            "the leftover accumulator must itself be clamped to at most max*interval, got {remainder}"
        );
    }

    #[test]
    fn zero_accumulator_yields_zero_ticks() {
        let (ticks, remainder) = ticks_due(0.0, 1.0, 8);
        assert_eq!(ticks, 0);
        assert!((remainder - 0.0).abs() < 1e-9, "got {remainder}");
    }

    /// A negative accumulator cannot occur through normal operation (the
    /// only writer, `advance_ticks`, only ever adds a non-negative
    /// `delta_secs()` or resets to `0.0`) — sanitized defensively rather
    /// than trusted, so a future caller cannot propagate a negative
    /// backlog forward.
    #[test]
    fn negative_accumulator_yields_zero_ticks() {
        let (ticks, remainder) = ticks_due(-5.0, 1.0, 8);
        assert_eq!(ticks, 0);
        assert!((remainder - 0.0).abs() < 1e-9, "got {remainder}");
    }
}
