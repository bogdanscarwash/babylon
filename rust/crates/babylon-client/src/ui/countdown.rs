//! The countdown/pressure pane and the HUD's `B` hint (B3 wave-1 Task 6,
//! plan `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.4/§3.3): for a story declaring `Story.delays` (carceral only, wave 1
//! — counties declares none, this task's own standing fact), renders one
//! row naming the NEXT pending beat's countdown, computed from a live
//! engine field plus a declared `.bscn` constant — never from the seeded
//! `0` a latch starts at (the seeded-0 trap, §2.4).
//!
//! **`projection.rs`'s own module doc names this file's job.** "The
//! table's fourth row — an `institution/*-tick` latch before its companion
//! flag flips — is CONDITIONAL on that flag's live value; it is the
//! countdown pane's own concern (plan §2.4/§3.3, a later task) and is not
//! reproduced here." This module IS that later task: `resolve` (below, a
//! private module function — not a public doc link) reads the
//! gating flag through [`crate::projection::Projector`] (an ordinary
//! `Material` read, not a static `NotComputed` table entry — the
//! conditionality lives in THIS module's own control flow), and constructs
//! [`crate::projection::Provenance::NotComputed`] itself, by hand, only
//! while the flag still reads `0`.
//!
//! **The delay chain (carceral-specific, like `narration::NARRATION_TABLE`'s
//! own per-`EventType` hardcoding — only carceral declares delays today).**
//! `CARCERAL_STEPS` (below, a private module constant — not a public doc
//! link) walks the three `institution/*-tick` latches in firing
//! order (`decomposition.bsl`/`control-ratio.bsl`, cited in full at
//! `story.rs`'s own `CARCERAL_PREMISE` and
//! `carceral-arc-conformance.bscn:11-18`'s DERIVED TICK SCHEDULE). Each
//! step's `done_field` is the SAME field as the next step's `ready_field`
//! — the flag a beat flips when it fires is exactly what makes the next
//! beat's own countdown operand valid (both written in the SAME `effects`
//! block, `decomposition.bsl:309-310`/`control-ratio.bsl:337-338`), so
//! retiring one row and making the next one live happens on the SAME tick,
//! with no gap — confirmed directly by
//! `after_class_decomposition_fires_its_row_retires_and_control_ratio_crisis_appears`
//! (`tests/countdown.rs`).

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::projection::{Projector, Provenance, Reading};
use crate::story::Story;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use bevy::prelude::*;

/// Reads `operand_field` honestly, gated by its own `ready_field` latch —
/// the seeded-0 trap's own fix (§2.4): `operand_field` is seeded literal
/// `0` at scenario load and only overwritten when the rule that owns it
/// fires, so a bare `Projector::material()` read would fabricate "fired at
/// tick 0" for every tick before that. `NotComputed` (reason "not yet
/// latched") while `ready_field` still reads its seeded `0`; an ordinary
/// `Material` read once it flips to `1`.
///
/// The ONE mechanism both [`resolve`] (below) and `ui::roster_panel`'s own
/// dossier read through for these fields — `roster_panel` publishes two of
/// this module's own three `CountdownStep::operand_field`s
/// (`decomposition-fire-tick`, `control-crisis-tick`) on the carrier's
/// full field list, and calls this function directly rather than
/// reimplementing the gate a second time.
#[must_use]
pub(crate) fn read_gated_operand(
    projector: &Projector,
    graph: &dyn GraphSubstrate,
    id: NodeId,
    ready_field: &str,
    operand_field: &str,
) -> Reading {
    if projector.read(graph, id, ready_field).value == Some(1.0) {
        projector.read(graph, id, operand_field)
    } else {
        Reading {
            value: None,
            provenance: Provenance::NotComputed {
                reason: "not yet latched",
            },
        }
    }
}

/// One row of a story's own delay chain — see the module doc.
struct CountdownStep {
    /// The beat this step's delay counts down to.
    event_type: &'static str,
    /// Matches one `Story.delays[].name` (§2.5) — the declared constant
    /// this step's countdown adds to the live operand.
    delay_const_name: &'static str,
    /// The live `institution/*` field read as this countdown's anchor tick
    /// — valid only once `ready_field` reads `1`.
    operand_field: &'static str,
    /// Display name for `operand_field` — the row's own left operand.
    operand_label: &'static str,
    /// `institution/*` flag: `0` means `operand_field` is still its seeded
    /// `0` (not yet latched, §2.4's own seeded-0 trap); `1` means it holds
    /// a real tick.
    ready_field: &'static str,
    /// `institution/*` flag: `1` means this step's own beat has already
    /// fired — its row retires and the next step (if any) becomes pending.
    done_field: &'static str,
}

/// The carceral rule pack's own three-beat delay chain.
const CARCERAL_STEPS: &[CountdownStep] = &[
    CountdownStep {
        event_type: "CLASS_DECOMPOSITION",
        delay_const_name: "carceral/decomposition-delay",
        operand_field: "institution/superwage-crisis-tick",
        operand_label: "superwage-crisis-tick",
        ready_field: "institution/superwage-crisis-known",
        done_field: "institution/decomposition-fired-known",
    },
    CountdownStep {
        event_type: "CONTROL_RATIO_CRISIS",
        delay_const_name: "carceral/control-ratio-delay",
        operand_field: "institution/decomposition-fire-tick",
        operand_label: "decomposition-fire-tick",
        ready_field: "institution/decomposition-fired-known",
        done_field: "institution/control-crisis-emitted",
    },
    CountdownStep {
        event_type: "TERMINAL_DECISION",
        delay_const_name: "carceral/terminal-decision-delay",
        operand_field: "institution/control-crisis-tick",
        operand_label: "control-crisis-tick",
        ready_field: "institution/control-crisis-emitted",
        done_field: "institution/terminal-decision-emitted",
    },
];

/// `story.id -> its own delay-chain table` — carceral-only today (counties
/// declares no delays); an empty slice for any other story id is what
/// makes [`resolve`] return `None` for counties with no special-casing.
fn steps_for(story_id: &str) -> &'static [CountdownStep] {
    match story_id {
        "carceral" => CARCERAL_STEPS,
        _ => &[],
    }
}

/// The next pending beat's own resolved state (§3.3).
enum Resolved {
    /// `step.ready_field` has not yet flipped — reading `operand_field` now
    /// would be exactly the seeded-0 fabrication §2.4 forbids.
    NotYetLatched { event_type: &'static str },
    /// `step.ready_field` is `1` — the countdown is a real, named
    /// computation over a live field and a declared constant.
    Live {
        event_type: &'static str,
        ticks_remaining: f64,
        operand_label: &'static str,
        operand_value: f64,
        delay_name: &'static str,
        delay_value: f64,
        delay_source: &'static str,
        tick: i64,
    },
}

/// Finds `story`'s own delay chain's first step whose beat has not yet
/// fired, and resolves it — `None` when the story declares no delays
/// (counties) or every declared beat has already fired (carceral past tick
/// 106).
///
/// # Panics
/// If `story` declares countdown steps but its own scenario mints no
/// `INSTITUTION` node, or a step names a `delay_const_name` `story.delays`
/// does not declare — both are wiring bugs in this module's own static
/// tables against `story.rs`'s own catalog, unreachable through either
/// shipped story (`tests/story.rs`'s own derived-roster tests would
/// already have caught a scenario/catalog mismatch first).
fn resolve(story: &Story, graph: &dyn GraphSubstrate, tick: i64) -> Option<Resolved> {
    let steps = steps_for(story.id);
    if steps.is_empty() {
        return None;
    }
    let institution = graph
        .nodes("INSTITUTION")
        .into_iter()
        .next()
        .unwrap_or_else(|| {
            panic!(
                "story {:?} declares countdown steps but its scenario mints no INSTITUTION node",
                story.id
            )
        });
    let projector = Projector::material();
    let step = steps
        .iter()
        .find(|s| projector.read(graph, institution, s.done_field).value != Some(1.0))?;

    let reading = read_gated_operand(
        &projector,
        graph,
        institution,
        step.ready_field,
        step.operand_field,
    );
    let Provenance::Material = reading.provenance else {
        return Some(Resolved::NotYetLatched {
            event_type: step.event_type,
        });
    };
    let operand_value = reading
        .value
        .expect("Material provenance always carries a value");
    let delay = story
        .delays
        .iter()
        .find(|d| d.name == step.delay_const_name)
        .unwrap_or_else(|| {
            panic!(
                "story {:?} step {:?}: no declared delay named {:?}",
                story.id, step.event_type, step.delay_const_name
            )
        });
    // Tick counters and declared delay constants are exact whole-number
    // floats at this crate's own scale (< 1000) — no rounding error is
    // possible in this subtraction, the same "exact at this scale"
    // reasoning `ui::time::ticks_due`'s own doc gives for its own casts.
    #[allow(clippy::cast_precision_loss)]
    let ticks_remaining = operand_value + delay.value - tick as f64;
    Some(Resolved::Live {
        event_type: step.event_type,
        ticks_remaining,
        operand_label: step.operand_label,
        operand_value,
        delay_name: delay.name,
        delay_value: delay.value,
        delay_source: delay.source,
        tick,
    })
}

/// Renders the countdown/pressure pane (§3.3): one row for the next
/// pending beat, or an empty string once the story declares no delays
/// (counties) or every declared beat has already fired — never a stale
/// row (`resolve` returning `None` covers both cases identically).
#[must_use]
pub fn format_countdown_pane(story: &Story, graph: &dyn GraphSubstrate, tick: i64) -> String {
    match resolve(story, graph, tick) {
        None => String::new(),
        Some(Resolved::NotYetLatched { event_type }) => {
            let reading = Reading {
                value: None,
                provenance: Provenance::NotComputed {
                    reason: "not yet latched",
                },
            };
            format!(
                "next beat   {event_type}\n            {}",
                reading.render(0)
            )
        }
        Some(Resolved::Live {
            event_type,
            ticks_remaining,
            operand_label,
            operand_value,
            delay_name,
            delay_value,
            delay_source,
            tick,
        }) => format!(
            "next beat   {event_type}   in {ticks_remaining:.0} ticks\n            \
             {operand_label} {operand_value:.0}  +  {delay_name} {delay_value:.0} \
             ({delay_source})  \u{2212}  tick {tick}"
        ),
    }
}

/// The HUD `B` hint (§2.4: "the HUD shows what it will skip to when a
/// countdown is live") — `""` for `NotYetLatched` (never `in ? ticks`) and
/// for no pending beat, matching [`format_countdown_pane`]'s own `None`
/// case.
#[must_use]
pub fn format_beat_hint(story: &Story, graph: &dyn GraphSubstrate, tick: i64) -> String {
    match resolve(story, graph, tick) {
        Some(Resolved::Live {
            ticks_remaining, ..
        }) => format!("B \u{2192} next beat in {ticks_remaining:.0} ticks"),
        _ => String::new(),
    }
}

#[derive(Component)]
pub struct CountdownPaneText;

/// `Startup` system: spawns the (initially empty) countdown pane text
/// entity — `refresh_countdown_pane`'s own first `Update` pass fills it in.
pub fn spawn_countdown_pane(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::GOLD),
        Node {
            position_type: PositionType::Absolute,
            top: px(340),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::Countdown),
        CountdownPaneText,
    ));
}

/// `Update` system: repaints [`CountdownPaneText`] from `EngineSession` +
/// the live tick — reactive on either changing (a new tick advanced via
/// `ui::time::advance_ticks`, or `ui::story_card::restart_on_n_key` swapped
/// in a fresh session).
pub fn refresh_countdown_pane(
    counter: Res<crate::loop_ui::TickCounter>,
    session: Res<crate::engine_link::EngineSession>,
    mut pane_text: Query<&mut Text, With<CountdownPaneText>>,
) {
    if !counter.is_changed() && !session.is_changed() {
        return;
    }
    let Ok(mut text) = pane_text.single_mut() else {
        return;
    };
    text.0 = format_countdown_pane(session.story, session.inner.graph(), counter.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story;
    use babylon_bsl::scenario::load_scenario;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::hypergraph_store::HypergraphStore;
    use babylon_kernel::SessionId;
    use babylon_tick::TickSession;

    /// Builds a fresh carceral `TickSession` the same way `EngineSession::start`
    /// does — a bare `TickSession`, not the whole Bevy `EngineSession`
    /// resource, since these unit tests need only the graph + `advance`.
    fn carceral_session() -> TickSession<HypergraphStore> {
        let story = story::carceral();
        let rule_src = story.rule_srcs.join("\n");
        TickSession::new(
            story.scenario_src,
            &rule_src,
            HypergraphStore::new(),
            SessionId::new(story.session_id).expect("carceral session id"),
        )
        .expect("carceral session starts")
    }

    #[test]
    fn counties_declares_no_delays_so_the_pane_and_hint_are_always_empty() {
        let mut graph = HypergraphStore::new();
        let counties = story::counties();
        load_scenario(counties.scenario_src, &mut graph).expect("counties loads");
        assert_eq!(format_countdown_pane(counties, &graph, 0), "");
        assert_eq!(format_beat_hint(counties, &graph, 0), "");
    }

    #[test]
    fn at_tick_zero_the_pane_renders_not_yet_latched_with_no_digit() {
        let session = carceral_session();
        let carceral = story::carceral();
        let rendered = format_countdown_pane(carceral, session.graph(), 0);
        assert!(rendered.contains("CLASS_DECOMPOSITION"), "got {rendered:?}");
        assert!(rendered.contains("not yet latched"), "got {rendered:?}");
        assert!(
            !rendered.chars().any(|c| c.is_ascii_digit()),
            "no digit that could read as a countdown may appear before the latch flips \
             (§2.4's seeded-0 trap), got {rendered:?}"
        );
        assert_eq!(format_beat_hint(carceral, session.graph(), 0), "");
    }

    #[test]
    fn at_tick_14_class_decomposition_reads_39_ticks_naming_both_operands() {
        let mut session = carceral_session();
        let mut sink = CollectingSink::default();
        for _ in 0..14 {
            session.advance(&mut sink).expect("tick advances");
        }
        let carceral = story::carceral();
        let rendered = format_countdown_pane(carceral, session.graph(), 14);
        assert!(rendered.contains("CLASS_DECOMPOSITION"), "got {rendered:?}");
        assert!(rendered.contains("39 ticks"), "got {rendered:?}");
        assert!(
            rendered.contains("superwage-crisis-tick 1"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral/decomposition-delay 52"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral-arc-conformance.bscn:137"),
            "got {rendered:?}"
        );
        assert!(rendered.contains("tick 14"), "got {rendered:?}");

        let hint = format_beat_hint(carceral, session.graph(), 14);
        assert_eq!(hint, "B \u{2192} next beat in 39 ticks");
    }

    #[test]
    fn after_class_decomposition_fires_its_row_retires_and_control_ratio_crisis_appears() {
        let mut session = carceral_session();
        let mut sink = CollectingSink::default();
        for _ in 0..53 {
            session.advance(&mut sink).expect("tick advances");
        }
        let carceral = story::carceral();
        let rendered = format_countdown_pane(carceral, session.graph(), 53);
        assert!(
            !rendered.contains("CLASS_DECOMPOSITION"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("CONTROL_RATIO_CRISIS"),
            "got {rendered:?}"
        );
        assert!(rendered.contains("in 52 ticks"), "got {rendered:?}");
        assert!(
            rendered.contains("decomposition-fire-tick 53"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral/control-ratio-delay 52"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral-arc-conformance.bscn:138"),
            "got {rendered:?}"
        );
    }

    #[test]
    fn after_control_ratio_crisis_fires_terminal_decision_appears_with_a_one_tick_delay() {
        let mut session = carceral_session();
        let mut sink = CollectingSink::default();
        for _ in 0..105 {
            session.advance(&mut sink).expect("tick advances");
        }
        let carceral = story::carceral();
        let rendered = format_countdown_pane(carceral, session.graph(), 105);
        assert!(
            !rendered.contains("CONTROL_RATIO_CRISIS"),
            "got {rendered:?}"
        );
        assert!(rendered.contains("TERMINAL_DECISION"), "got {rendered:?}");
        assert!(rendered.contains("in 1 ticks"), "got {rendered:?}");
        assert!(
            rendered.contains("control-crisis-tick 105"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral/terminal-decision-delay 1"),
            "got {rendered:?}"
        );
        assert!(
            rendered.contains("carceral-arc-conformance.bscn:139"),
            "got {rendered:?}"
        );
    }

    #[test]
    fn after_terminal_decision_fires_the_pane_and_hint_are_both_empty() {
        let mut session = carceral_session();
        let mut sink = CollectingSink::default();
        for _ in 0..108 {
            session.advance(&mut sink).expect("tick advances");
        }
        let carceral = story::carceral();
        assert_eq!(format_countdown_pane(carceral, session.graph(), 108), "");
        assert_eq!(format_beat_hint(carceral, session.graph(), 108), "");
    }
}
