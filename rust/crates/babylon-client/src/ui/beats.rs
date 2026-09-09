//! The narrative beat feed + latch card (B3 wave-1 Task 4, plan
//! `docs/superpowers/plans/2026-08-17-b3-null-hypothesis-viewer.md`
//! §2.2/§2.4/§3.6/C2): drains `EngineSession.sink` every tick into a
//! bounded [`BeatLog`] (closing #503's unbounded-growth item), classifies
//! each drained event through `crate::severity`, collapses same-tick FLOW
//! events into one line, and renders each surviving beat through
//! `crate::narration`. `TERMINAL_DECISION` renders the latch card instead
//! of an ordinary beat — a system latch, never an end card, never a
//! verdict, never the five-outcome vocabulary (§3.6).
//!
//! RED→GREEN record: authored RED when none of the production items
//! `tests/beats.rs`/`tests/autopause.rs` reference existed (every call site
//! resolving through this module failed, mirroring the `d4f353d9` "module
//! absent" RED-commit precedent); GREEN since this module landed — the
//! items below are the production beat feed those tests guard.

use crate::decision_surface::{DeclaredSurface, SurfaceId};
use crate::engine_link::EngineSession;
use crate::narration;
use crate::severity::{self, EventKind, SeverityTier};
use babylon_bsl::evaluator::Value;
use babylon_graph::substrate::NodeId;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

/// One drained event, tagged with the tick it fired on and its resolved
/// severity — `magnitude_delta` is `Some(_)` ONLY for `LIFECYCLE_TRANSITION`
/// (§2.4's Σ|Δpop-d-prime| requirement), `None` for every other event.
#[derive(Debug, Clone)]
pub struct Beat {
    pub tick: i64,
    pub event_type: String,
    pub payload: Vec<(String, Value)>,
    pub tier: SeverityTier,
    pub magnitude_delta: Option<f64>,
}

/// Oldest dropped once this many beats have accumulated (§2.2 point 2,
/// closing #503).
pub const BEAT_LOG_CAPACITY: usize = 512;

/// The client-owned, bounded beat history — `ui::time::advance_ticks` is
/// the sole writer (via [`drain_tick_into_beat_log`]).
#[derive(Resource, Default)]
pub struct BeatLog {
    pub beats: VecDeque<Beat>,
    /// Per-territory last-seen `pop-d-prime`, used ONLY to compute
    /// [`Beat::magnitude_delta`] as `LIFECYCLE_TRANSITION` beats are
    /// drained. The FIRST sighting of a territory has no prior tick to
    /// diff against, so its own delta is honestly `0.0` (never a
    /// fabricated jump from an unknown baseline) rather than skipped.
    last_pop_d_prime: HashMap<NodeId, f64>,
}

impl BeatLog {
    /// Records one drained event, computing its `magnitude_delta` (for
    /// `LIFECYCLE_TRANSITION` only) before pushing, then evicts from the
    /// front if the single push just pushed the capacity bound.
    ///
    /// **Power-of-10 rule 2 (task-4-review.md Minor 5).** `push_back` is the
    /// only place this struct grows `beats`, and it always grows it by
    /// exactly one — so immediately afterward `beats.len()` can exceed
    /// `BEAT_LOG_CAPACITY` by at most one, and a single `if` (not a loop at
    /// all, so trivially bounded) always restores the invariant. The prior
    /// `while` form was correct but unbounded-looking; this is the same
    /// eviction, stated as what it actually is.
    fn record(&mut self, tick: i64, event_type: String, payload: Vec<(String, Value)>) {
        let tier = severity::severity_for(&event_type);
        let magnitude_delta = if event_type == "LIFECYCLE_TRANSITION" {
            self.lifecycle_delta(&payload)
        } else {
            None
        };
        self.beats.push_back(Beat {
            tick,
            event_type,
            payload,
            tier,
            magnitude_delta,
        });
        if self.beats.len() > BEAT_LOG_CAPACITY {
            self.beats.pop_front();
        }
    }

    /// `Σ|Δpop-d-prime|`'s per-beat contribution: `|new - last_known|`, or
    /// `0.0` on this territory's first-ever sighting. `None` only if the
    /// payload is malformed (missing `territory-id`/`pop-d-prime`) — an
    /// honest "nothing to report" rather than a panic on a shape this
    /// crate does not otherwise validate.
    fn lifecycle_delta(&mut self, payload: &[(String, Value)]) -> Option<f64> {
        let id = payload.iter().find_map(|(k, v)| {
            if k == "territory-id" {
                if let Value::NodeRef(id) = v {
                    Some(*id)
                } else {
                    None
                }
            } else {
                None
            }
        })?;
        let new_value = payload.iter().find_map(|(k, v)| {
            if k == "pop-d-prime" {
                if let Value::Real(r) = v {
                    Some(*r)
                } else {
                    None
                }
            } else {
                None
            }
        })?;
        let delta = self
            .last_pop_d_prime
            .get(&id)
            .map_or(0.0, |prev| (new_value - prev).abs());
        self.last_pop_d_prime.insert(id, new_value);
        Some(delta)
    }
}

/// The outcome of one tick's drain — the two conditions
/// `ui::time::advance_ticks` checks to decide whether to autopause (§3.6):
/// `any_critical` gates on `RunState.autopause == OnCritical`;
/// `terminal_decision` pauses UNCONDITIONALLY, independent of that mode.
#[derive(Debug, Clone, Copy, Default)]
pub struct DrainOutcome {
    pub any_critical: bool,
    pub terminal_decision: bool,
}

/// Drains `session.sink.events` (§2.2's #503 fix — draining rather than
/// re-reading is what keeps the sink itself bounded) into `log`, tagging
/// each drained event with `tick`. Loop bound: `session.sink.events.len()`
/// at call time (`drain` consumes the whole `Vec`, so this cannot iterate
/// more times than that).
pub fn drain_tick_into_beat_log(
    session: &mut EngineSession,
    tick: i64,
    log: &mut BeatLog,
) -> DrainOutcome {
    let mut outcome = DrainOutcome::default();
    for (event_type, payload) in session.sink.events.drain(..) {
        if severity::severity_for(&event_type) == SeverityTier::Critical {
            outcome.any_critical = true;
        }
        if event_type == "TERMINAL_DECISION" {
            outcome.terminal_decision = true;
        }
        log.record(tick, event_type, payload);
    }
    outcome
}

/// Resolves a beat's subject display: the declared `subject_key`'s
/// `NodeRef` resolved through `node_by_fips` (a FIPS string for a
/// territory), an `entity #N` fallback for a resolvable-but-non-territory
/// node (§2.2's own precedent — vitality's `ENTITY_DEATH` never resolves
/// through a fips-keyed map either), or `"{story_title} \u{b7} world"` for
/// a subject-less row (Minor 3) or a declared-but-payload-absent subject —
/// B3 wave-1 Task 5 unlocks the `story.title` half of Minor 3's own
/// deferred `@ <story.title> \u{b7} world` form (Task 4 rendered bare
/// `"world"`, since `story.title` did not exist before this catalog).
fn resolve_subject_display(
    event_type: &str,
    payload: &[(String, Value)],
    node_by_fips: &[(String, NodeId)],
    story_title: &str,
) -> String {
    let world = || format!("{story_title} \u{b7} world");
    let Some(key) = narration::spec_for(event_type).and_then(|spec| spec.subject_key) else {
        return world();
    };
    let Some(id) = payload.iter().find_map(|(k, v)| {
        if k == key {
            if let Value::NodeRef(id) = v {
                Some(*id)
            } else {
                None
            }
        } else {
            None
        }
    }) else {
        return world();
    };
    node_by_fips
        .iter()
        .find(|(_, nid)| *nid == id)
        .map_or_else(|| format!("entity #{}", id.0), |(fips, _)| fips.clone())
}

fn format_single_beat(beat: &Beat, node_by_fips: &[(String, NodeId)], story_title: &str) -> String {
    let subject_display =
        resolve_subject_display(&beat.event_type, &beat.payload, node_by_fips, story_title);
    let rendered = narration::render(&beat.event_type, &beat.payload, &subject_display);
    match rendered.because {
        Some(because) => format!(
            "tick {}: {}\n  because: {because}",
            beat.tick, rendered.headline
        ),
        None => format!("tick {}: {}", beat.tick, rendered.headline),
    }
}

fn format_collapsed_line(tick: i64, event_type: &str, count: usize, magnitude: f64) -> String {
    if event_type == "LIFECYCLE_TRANSITION" {
        return format!(
            "tick {tick}: {count} territories advanced the D-P-D\u{2032} circuit \
             (\u{3a3}|\u{394}pop-d\u{2032}| = {magnitude:.1})"
        );
    }
    format!("tick {tick}: {count} {event_type} events")
}

enum FeedGroup<'a> {
    Single(&'a Beat),
    Collapsed {
        tick: i64,
        event_type: &'a str,
        count: usize,
        magnitude: f64,
    },
}

/// Groups a bounded beat slice into single/collapsed render units, in
/// FORWARD (oldest-first) order. Same-tick same-type FLOW beats collapse
/// into ONE group even when NOT drain-adjacent — `lifecycle.bsl` interleaves
/// each territory's `LIFECYCLE_TRANSITION` with that SAME territory's
/// conditional `LEGITIMATION_CRISIS`/`_RECOVERY` emit before moving to the
/// next territory, so the 12 `LIFECYCLE_TRANSITION`s of one tick are NOT
/// consecutive in drain order; a strict-adjacency collapse would have split
/// them into several small groups instead of the one §2.2 requires.
///
/// Two passes, each bounded by `beats.len()` (Power-of-10 rule 2): the
/// first aggregates every FLOW beat's `(tick, event_type)` totals and
/// records the index of its FIRST occurrence; the second walks `beats`
/// once more, emitting a `Collapsed` group at each key's first-occurrence
/// index and a `Single` group for every non-FLOW beat, in original order.
fn group_beats(beats: &VecDeque<Beat>) -> Vec<FeedGroup<'_>> {
    let mut totals: HashMap<(i64, &str), (usize, f64, usize)> = HashMap::new(); // (count, magnitude, first_index)
    for (idx, beat) in beats.iter().enumerate() {
        if severity::kind_for(&beat.event_type) == Some(EventKind::Flow) {
            let entry = totals
                .entry((beat.tick, beat.event_type.as_str()))
                .or_insert((0, 0.0, idx));
            entry.0 += 1;
            entry.1 += beat.magnitude_delta.unwrap_or(0.0);
        }
    }

    let mut groups = Vec::new();
    for (idx, beat) in beats.iter().enumerate() {
        if severity::kind_for(&beat.event_type) == Some(EventKind::Flow) {
            let key = (beat.tick, beat.event_type.as_str());
            let (count, magnitude, first_index) = totals[&key];
            if idx == first_index {
                groups.push(FeedGroup::Collapsed {
                    tick: beat.tick,
                    event_type: &beat.event_type,
                    count,
                    magnitude,
                });
            }
        } else {
            groups.push(FeedGroup::Single(beat));
        }
    }
    groups
}

fn render_group(
    group: &FeedGroup<'_>,
    node_by_fips: &[(String, NodeId)],
    story_title: &str,
) -> String {
    match group {
        FeedGroup::Single(beat) => format_single_beat(beat, node_by_fips, story_title),
        FeedGroup::Collapsed {
            tick,
            event_type,
            count,
            magnitude,
        } => format_collapsed_line(*tick, event_type, *count, *magnitude),
    }
}

/// Renders the feed: the most recent `max_lines` render units (a single
/// beat, or one same-tick-same-type FLOW collapse), newest first.
/// `story_title` names the current story for a world-scoped beat's own
/// `@ <story_title> \u{b7} world` subject display (§2.2 Minor 3).
#[must_use]
pub fn format_beat_feed(
    log: &BeatLog,
    node_by_fips: &[(String, NodeId)],
    max_lines: usize,
    story_title: &str,
) -> String {
    let groups = group_beats(&log.beats);
    let start = groups.len().saturating_sub(max_lines);
    groups[start..]
        .iter()
        .rev()
        .map(|g| render_group(g, node_by_fips, story_title))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Severity -> color (§2.2 point 3): critical = CRIMSON, warning = GOLD,
/// informational = DIM.
#[must_use]
pub fn severity_color(tier: SeverityTier) -> Color {
    match tier {
        SeverityTier::Critical => crate::palette::CRIMSON,
        SeverityTier::Warning => crate::palette::GOLD,
        SeverityTier::Informational => crate::palette::DIM,
    }
}

/// I1 (review round 1) — `severity_color` was declared with zero call
/// sites: severity is *classified* (drives the collapse rule and
/// autopause) but was never *presented*. This crate has no per-line
/// colored-text precedent anywhere (every text entity is one flat `Text` +
/// one `TextColor` — no `TextSpan` usage exists to color individual beat
/// lines within the block), so introducing one here would be a new
/// architectural pattern, not a scoped fix. The wiring instead colors the
/// WHOLE feed panel by the most severe beat currently in the visible
/// window (`max_lines`) — a real, meaningful, testable use of both
/// `severity_color` and `Beat::tier`: the panel reads CRIMSON the instant
/// a critical beat scrolls into view and stays that way until it scrolls
/// back out, GOLD for warning-only windows, DIM otherwise. `Collapsed`
/// groups are always FLOW-kind (`group_beats`'s own gate), hence never
/// `Critical`/`Warning`, so only `Single` beats can raise the tier.
#[must_use]
fn feed_accent_tier(log: &BeatLog, max_lines: usize) -> SeverityTier {
    let groups = group_beats(&log.beats);
    let start = groups.len().saturating_sub(max_lines);
    groups[start..]
        .iter()
        .filter_map(|g| match g {
            FeedGroup::Single(beat) => Some(beat.tier),
            FeedGroup::Collapsed { .. } => None,
        })
        .max_by_key(|tier| match tier {
            SeverityTier::Informational => 0,
            SeverityTier::Warning => 1,
            SeverityTier::Critical => 2,
        })
        .unwrap_or(SeverityTier::Informational)
}

const FEED_DEPTH: usize = 10;

#[derive(Component)]
pub struct BeatFeedText;

/// `Startup` system: spawns the (initially empty) beat feed text entity —
/// replaces `loop_ui::EventFeedText`'s spawn site.
pub fn spawn_beat_feed(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            top: px(160),
            right: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::BeatFeed),
        BeatFeedText,
    ));
}

/// `Update` system: repaints [`BeatFeedText`] from [`BeatLog`] + the
/// session's own roster — reads only, `advance_ticks` is the sole writer
/// of `BeatLog` itself. Also repaints the panel's own accent color via
/// `feed_accent_tier`/[`severity_color`] (I1, review round 1).
pub fn refresh_beat_feed(
    log: Res<BeatLog>,
    session: Res<EngineSession>,
    mut feed_text: Query<(&mut Text, &mut TextColor), With<BeatFeedText>>,
) {
    if !log.is_changed() {
        return;
    }
    let Ok((mut text, mut color)) = feed_text.single_mut() else {
        return;
    };
    text.0 = format_beat_feed(&log, &session.roster, FEED_DEPTH, session.story.title);
    color.0 = severity_color(feed_accent_tier(&log, FEED_DEPTH));
}

/// Renders the §3.6 latch card from a `TERMINAL_DECISION` beat's own
/// payload — every line is a named engine quantity or a citation; no
/// "end", no verdict, no five-outcome vocabulary anywhere on the card.
#[must_use]
pub fn format_latch_card(tick: i64, payload: &[(String, Value)]) -> String {
    let find_int = |key: &str| {
        payload.iter().find_map(|(k, v)| {
            if k == key {
                if let Value::Int(i) = v {
                    Some(*i)
                } else {
                    None
                }
            } else {
                None
            }
        })
    };
    let find_real = |key: &str| {
        payload.iter().find_map(|(k, v)| {
            if k == key {
                if let Value::Real(r) = v {
                    Some(*r)
                } else {
                    None
                }
            } else {
                None
            }
        })
    };

    // I2 (review round 1): the predecessor of this function defaulted
    // `outcome_word` to "GENOCIDE" for a missing OR out-of-range outcome
    // (`_ => "GENOCIDE"`), fabricating a claim about a value the engine
    // never put on the wire — on the single most ideologically-reserved
    // surface in the train. `outcome_line` now mirrors
    // `narration.rs::terminal_decision_template`'s own correct dispatch:
    // only `Some(0)`/`Some(1)` earn a named encoding; anything else routes
    // through the same "not computed by this port" class every other
    // honest-absence render in this crate already uses.
    let outcome = find_int("outcome");
    let outcome_line = match outcome {
        Some(1) => "outcome 1        (this pack's own numeric REVOLUTION encoding \u{2014} \
                     control-ratio.bsl:366-379)"
            .to_owned(),
        Some(0) => "outcome 0        (this pack's own numeric GENOCIDE encoding \u{2014} \
                     control-ratio.bsl:366-379)"
            .to_owned(),
        Some(other) => format!(
            "outcome {other}        (not computed by this port \u{2014} no verified encoding \
             for this value)"
        ),
        None => "outcome {absent}        (not computed by this port \u{2014} no verified \
                  encoding for this value)"
            .to_owned(),
    };
    let avg_org =
        find_real("avg-organization").map_or_else(|| "{absent}".to_owned(), |v| format!("{v:.4}"));
    let threshold = find_real("revolution-threshold")
        .map_or_else(|| "{absent}".to_owned(), |v| format!("{v:.4}"));

    format!(
        "LATCH  control-ratio/c04-terminal  \u{b7}  tick {tick}\n       \
         institution/terminal-decision-emitted  0 \u{2192} 1\n       \
         {outcome_line}\n       \
         avg-organization {avg_org}   revolution-threshold {threshold}\n\n\
         This is one system's own terminal branch, not the game's ending.\n\
         The five canonical outcomes are not computed here.\n\
         Press P to keep running."
    )
}

#[derive(Component)]
pub struct LatchCardText;

/// `Startup` system: spawns the (initially empty) latch card entity.
pub fn spawn_latch_card(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextColor(crate::palette::CRIMSON),
        Node {
            position_type: PositionType::Absolute,
            top: px(300),
            left: px(24),
            ..default()
        },
        DeclaredSurface::new(SurfaceId::TerminalLatch),
        LatchCardText,
    ));
}

/// `Update` system: repaints [`LatchCardText`] from the most recent
/// `TERMINAL_DECISION` beat in [`BeatLog`], if any. B3 wave-1 Task 5's own
/// `N`-key restart (`ui::story_card::restart_on_n_key`) resets `BeatLog` to
/// `BeatLog::default()` on every restart, so this system's own `.find`
/// naturally finds nothing and renders an empty string — the restart
/// mechanic this doc comment used to say did not exist yet.
pub fn refresh_latch_card(log: Res<BeatLog>, mut card_text: Query<&mut Text, With<LatchCardText>>) {
    if !log.is_changed() {
        return;
    }
    let Ok(mut text) = card_text.single_mut() else {
        return;
    };
    text.0 = log
        .beats
        .iter()
        .rev()
        .find(|b| b.event_type == "TERMINAL_DECISION")
        .map_or_else(String::new, |b| format_latch_card(b.tick, &b.payload));
}

/// Measured at implementation, not assumed (§2.7/I3): across this many
/// ticks the counties story's own `tests/beats.rs::
/// counties_stay_numerically_sane_to_the_validated_horizon` proves every
/// listed field stays finite and non-negative.
pub const COUNTIES_VALIDATED_HORIZON: i64 = 600;

#[cfg(test)]
mod tests {
    use super::{feed_accent_tier, format_latch_card, severity_color, Beat, BeatLog};
    use crate::severity::SeverityTier;
    use babylon_bsl::evaluator::Value;

    fn beat(tick: i64, event_type: &str, tier: SeverityTier) -> Beat {
        Beat {
            tick,
            event_type: event_type.to_owned(),
            payload: Vec::new(),
            tier,
            magnitude_delta: None,
        }
    }

    fn log_of(beats: Vec<Beat>) -> BeatLog {
        let mut log = BeatLog::default();
        for b in beats {
            log.beats.push_back(b);
        }
        log
    }

    // ---- I1: severity_color is wired into the feed's own accent tier ----

    #[test]
    fn an_empty_log_has_the_informational_accent() {
        let log = log_of(vec![]);
        assert_eq!(feed_accent_tier(&log, 10), SeverityTier::Informational);
        assert_eq!(
            severity_color(feed_accent_tier(&log, 10)),
            crate::palette::DIM
        );
    }

    #[test]
    fn the_accent_is_the_most_severe_beat_in_the_visible_window() {
        let log = log_of(vec![
            beat(1, "LEGITIMATION_RECOVERY", SeverityTier::Informational),
            beat(2, "LEGITIMATION_CRISIS", SeverityTier::Warning),
        ]);
        assert_eq!(feed_accent_tier(&log, 10), SeverityTier::Warning);
        assert_eq!(
            severity_color(feed_accent_tier(&log, 10)),
            crate::palette::GOLD
        );

        let log = log_of(vec![
            beat(1, "LEGITIMATION_CRISIS", SeverityTier::Warning),
            beat(2, "SUPERWAGE_CRISIS", SeverityTier::Critical),
        ]);
        assert_eq!(feed_accent_tier(&log, 10), SeverityTier::Critical);
        assert_eq!(
            severity_color(feed_accent_tier(&log, 10)),
            crate::palette::CRIMSON
        );
    }

    #[test]
    fn a_critical_beat_outside_the_visible_window_does_not_raise_the_accent() {
        // Ten informational beats push the one critical beat (index 0)
        // out of a 5-line visible window.
        let mut beats = vec![beat(0, "SUPERWAGE_CRISIS", SeverityTier::Critical)];
        for i in 1..=10 {
            beats.push(beat(
                i,
                "LEGITIMATION_RECOVERY",
                SeverityTier::Informational,
            ));
        }
        let log = log_of(beats);
        assert_eq!(
            feed_accent_tier(&log, 5),
            SeverityTier::Informational,
            "the critical beat scrolled out of the 5-line window and must not still \
             color the panel"
        );
    }

    // ---- I2: an absent/unverified outcome never fabricates GENOCIDE ----

    #[test]
    fn format_latch_card_renders_the_verified_genocide_encoding() {
        let payload = vec![("outcome".to_owned(), Value::Int(0))];
        let card = format_latch_card(106, &payload);
        assert!(card.contains("outcome 0"));
        assert!(card.contains("numeric GENOCIDE encoding"));
        assert!(!card.contains("REVOLUTION"));
    }

    #[test]
    fn format_latch_card_renders_the_verified_revolution_encoding() {
        let payload = vec![("outcome".to_owned(), Value::Int(1))];
        let card = format_latch_card(106, &payload);
        assert!(card.contains("outcome 1"));
        assert!(card.contains("numeric REVOLUTION encoding"));
        assert!(!card.contains("GENOCIDE"));
    }

    #[test]
    fn format_latch_card_never_fabricates_genocide_for_a_missing_outcome() {
        let card = format_latch_card(106, &[]);
        assert!(
            !card.contains("GENOCIDE") && !card.contains("REVOLUTION"),
            "a missing outcome must render through the honest not-computed class, \
             never a fabricated encoding — got {card:?}"
        );
        assert!(card.contains("outcome {absent}"));
        assert!(card.contains("not computed by this port"));
    }

    #[test]
    fn format_latch_card_never_fabricates_genocide_for_an_out_of_range_outcome() {
        let payload = vec![("outcome".to_owned(), Value::Int(7))];
        let card = format_latch_card(106, &payload);
        assert!(
            !card.contains("GENOCIDE") && !card.contains("REVOLUTION"),
            "outcome 7 is neither the GENOCIDE (0) nor REVOLUTION (1) encoding — it must \
             render through the honest not-computed class, got {card:?}"
        );
        assert!(card.contains("outcome 7"));
        assert!(card.contains("not computed by this port"));
    }
}
