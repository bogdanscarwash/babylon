//! Shared button and dispatch decisions for one observer transport.
//! Only an acknowledged runtime commit changes the displayed durable week.

use crate::observer::{ObserverSession, SessionPhase};
use crate::observer_calendar::CampaignMonth;
use crate::observer_ui::ObserverCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlAvailability {
    Enabled,
    Disabled(&'static str),
}

pub(crate) const MONTH_ADVANCE_HELP: &str = "Campaign months are relative planning periods, not calendar dates: twelve periods span 52 weeks. Their ends round up to weekly commits (weeks 5, 9, 13, 18, and so on); lengths repeat 5, 4, 4 weeks. Run month stops at the next month boundary, or at the scenario horizon if earlier. Pause finishes the outstanding week; Resume month keeps the same target. Disclosed freight loss pauses a running month; Stop on delivery adds delivery pauses. Advance one week is a diagnostic control. Production follows the previous week's committed plan. New deliveries enter planning for the following week. Time changes only after a confirmed commit.";

const PENDING: ControlAvailability =
    ControlAvailability::Disabled("Wait for the current week to finish committing");
const HISTORICAL: ControlAvailability =
    ControlAvailability::Disabled("Return Live before advancing the campaign");
const CLOSING: ControlAvailability =
    ControlAvailability::Disabled("Closing the campaign; committed weeks are saved automatically");

fn view_availability(state: &ObserverSession) -> ControlAvailability {
    match state.phase {
        SessionPhase::Connecting => {
            ControlAvailability::Disabled("Waiting for the campaign to open")
        }
        SessionPhase::Loading => ControlAvailability::Disabled("Loading the committed observation"),
        SessionPhase::Failed | SessionPhase::Closed => {
            ControlAvailability::Disabled("Reopen the campaign to reconcile committed progress")
        }
        SessionPhase::Advancing => PENDING,
        SessionPhase::Ready | SessionPhase::Complete => ControlAvailability::Enabled,
    }
}

/// Read navigation waits for the outstanding commit and its exact observation.
pub(crate) fn inspection_availability(state: &ObserverSession) -> ControlAvailability {
    if state.quit_requested {
        CLOSING
    } else if state.advance_pending()
        && !matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed)
    {
        PENDING
    } else {
        view_availability(state)
    }
}

fn advance_availability(state: &ObserverSession) -> ControlAvailability {
    if state.viewed_tick != state.durable_tick {
        return HISTORICAL;
    }
    if state.phase == SessionPhase::Complete
        || state
            .horizon_tick
            .is_some_and(|horizon| state.durable_tick >= horizon)
    {
        return ControlAvailability::Disabled(
            "Scenario complete; committed history remains available",
        );
    }
    if state.advance_pending() && pending_finishes_scenario(state) {
        return ControlAvailability::Disabled(
            "This is the final week; further play is unavailable",
        );
    }
    if state.durable_tick.checked_add(1).is_none() {
        return ControlAvailability::Disabled("The campaign has reached its supported week limit");
    }
    ControlAvailability::Enabled
}

fn pending_finishes_scenario(state: &ObserverSession) -> bool {
    state.horizon_tick.is_some_and(|horizon| {
        state
            .durable_tick
            .checked_add(1)
            .is_some_and(|next| next >= horizon)
    })
}

/// Both the visible button and its command handler consult this same result.
pub(crate) fn availability(
    command: ObserverCommand,
    state: &ObserverSession,
) -> ControlAvailability {
    use ControlAvailability::{Disabled, Enabled};
    use ObserverCommand::{
        Live, NewCampaign, NewDelayedCampaign, NextWeek, Perspective, PreviousWeek, ReopenCampaign,
        Step, TogglePlay,
    };

    if command == ObserverCommand::Quit {
        return Enabled;
    }
    if state.quit_requested {
        return CLOSING;
    }

    // A lost acknowledgement retains the pending request. Deliberate reopen is
    // the recovery path; it reconciles durability before any further advance.
    if matches!(command, ReopenCampaign)
        && matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed)
    {
        return Enabled;
    }
    if state.advance_pending()
        && matches!(
            command,
            Step | NewCampaign | NewDelayedCampaign | ReopenCampaign
        )
    {
        return if matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed) {
            Disabled("Reopen the campaign to reconcile committed progress")
        } else {
            PENDING
        };
    }
    match command {
        TogglePlay if state.playing => Enabled,
        TogglePlay => {
            let advance = advance_availability(state);
            if advance != Enabled {
                return advance;
            }
            if CampaignMonth::after_week(state.durable_tick).is_none() {
                return Disabled("The campaign has reached its supported month limit");
            }
            match state.phase {
                // This queues transport, never a second outstanding advance.
                SessionPhase::Ready | SessionPhase::Loading | SessionPhase::Advancing => Enabled,
                _ => view_availability(state),
            }
        }
        Step => {
            let view = view_availability(state);
            if view == Enabled {
                advance_availability(state)
            } else {
                view
            }
        }
        PreviousWeek | NextWeek | Live | Perspective => {
            let view = inspection_availability(state);
            if view != Enabled {
                return view;
            }
            match command {
                PreviousWeek if state.viewed_tick == 0 => Disabled("Already at the opening week"),
                NextWeek | Live if state.viewed_tick >= state.durable_tick => {
                    Disabled("Already viewing the live committed week")
                }
                _ => Enabled,
            }
        }
        // Presentation controls and deliberate campaign choices remain usable.
        _ => Enabled,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TurnPresentation {
    pub period: String,
    pub status: String,
    pub play_label: &'static str,
    pub step_label: String,
}

fn pending_status(state: &ObserverSession) -> String {
    let target = state.durable_tick.checked_add(1).map_or_else(
        || "the requested week".to_owned(),
        |week| format!("week {week}"),
    );
    if state.playing {
        state.month_plan().map_or_else(
            || format!("Advancing to {target}..."),
            |month| format!("Running month {}; processing {target}...", month.number),
        )
    } else {
        format!("Finishing {target}; paused after this week.")
    }
}

fn turn_status(state: &ObserverSession) -> String {
    if state.quit_requested {
        return if state.advance_pending()
            && !matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed)
        {
            "Finishing the current week before closing. Completed weeks are saved automatically."
                .into()
        } else {
            "Closing the campaign. Completed weeks are saved automatically.".into()
        };
    }
    if state.phase == SessionPhase::Failed {
        return "Connection failed. Reopen to reconcile.".into();
    }
    if state.phase == SessionPhase::Closed {
        return "Campaign closed. Reopen to continue.".into();
    }
    if state.advance_pending() || state.phase == SessionPhase::Advancing {
        return pending_status(state);
    }
    let historical = state.viewed_tick < state.durable_tick;
    match state.phase {
        SessionPhase::Connecting => "Opening the campaign...".into(),
        SessionPhase::Loading if historical => format!(
            "Loading committed week {} (live {}).",
            state.viewed_tick, state.durable_tick,
        ),
        SessionPhase::Loading if state.durable_tick == 0 => {
            "Loading the opening campaign observation...".into()
        }
        SessionPhase::Loading => format!("Loading committed week {}...", state.durable_tick),
        SessionPhase::Complete => "Scenario complete. History remains available.".into(),
        SessionPhase::Ready if historical => format!(
            "History {} / live {}. Return Live to advance.",
            state.viewed_tick, state.durable_tick
        ),
        SessionPhase::Ready => month_status(state),
        SessionPhase::Advancing | SessionPhase::Failed | SessionPhase::Closed => {
            unreachable!("handled above")
        }
    }
}

fn month_status(state: &ObserverSession) -> String {
    if let Some(month) = state.month_plan() {
        if state.playing {
            return format!("Running campaign month {}.", month.number);
        }
        if state.durable_tick == month.closing_week {
            return format!(
                "Campaign month {} complete. Plan the next month.",
                month.number
            );
        }
    }
    CampaignMonth::after_week(state.durable_tick).map_or_else(
        || "Campaign month limit reached.".into(),
        |month| {
            if month.opening_week < state.durable_tick {
                format!(
                    "Paused within campaign month {}. Resume when ready.",
                    month.number
                )
            } else {
                format!("Paused. Run campaign month {} when ready.", month.number)
            }
        },
    )
}

pub(crate) fn turn_presentation(state: &ObserverSession) -> TurnPresentation {
    let pending = state.advance_pending() || state.phase == SessionPhase::Advancing;
    let next = state.durable_tick.checked_add(1);
    let historical = state.viewed_tick < state.durable_tick;
    let month = CampaignMonth::at_week(state.viewed_tick);
    let period = if historical {
        format!("CAMPAIGN MONTH {} / HISTORY", month.number)
    } else {
        format!("CAMPAIGN MONTH {}", month.number)
    };
    let play_label = if state.playing {
        "Pause"
    } else if pending && pending_finishes_scenario(state) {
        "Month unavailable"
    } else if CampaignMonth::after_week(state.durable_tick)
        .is_some_and(|month| month.opening_week < state.durable_tick)
    {
        "Resume month"
    } else {
        "Run month"
    };
    let step_label = match state.phase {
        SessionPhase::Failed | SessionPhase::Closed | SessionPhase::Connecting => {
            "Advance unavailable".into()
        }
        _ if pending => next.map_or_else(
            || "Processing week".into(),
            |week| format!("Processing week {week}"),
        ),
        _ if historical => "Advance unavailable".into(),
        SessionPhase::Loading => format!("Loading week {}", state.viewed_tick),
        SessionPhase::Complete => "Scenario complete".into(),
        _ => next.map_or_else(
            || "Week limit reached".into(),
            |week| format!("Advance to week {week}"),
        ),
    };
    TurnPresentation {
        period,
        status: turn_status(state),
        play_label,
        step_label,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::Perspective;
    use babylon_persistence::CampaignId;

    fn ready(tick: u64) -> ObserverSession {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        state.ready(tick, None);
        assert!(state.installed(&state.context()));
        state
    }

    #[test]
    fn month_labels_distinguish_resume_completion_and_partial_scenario_horizon() {
        let mut state = ready(0);
        assert_eq!(turn_presentation(&state).period, "CAMPAIGN MONTH 1");
        assert_eq!(turn_presentation(&state).play_label, "Run month");
        assert!(state.run_or_resume_month());
        for week in 1..=5 {
            let request = state.begin_advance().unwrap();
            assert!(state.acknowledge(request, week, None));
            assert!(state.installed(&state.context()));
            if week == 2 {
                state.pause_month();
                assert_eq!(turn_presentation(&state).play_label, "Resume month");
                assert!(turn_presentation(&state)
                    .status
                    .contains("within campaign month 1"));
                assert!(state.run_or_resume_month());
            }
        }
        assert_eq!(turn_presentation(&state).play_label, "Run month");
        assert!(turn_presentation(&state)
            .status
            .contains("month 1 complete"));
        state.ready(15, None);
        state.horizon_tick = Some(16);
        assert!(state.installed(&state.context()));
        assert!(state.run_or_resume_month());
        let request = state.begin_advance().unwrap();
        assert!(state.acknowledge(request, 16, None));
        assert!(state.installed(&state.context()));
        let presentation = turn_presentation(&state);
        assert_eq!(presentation.period, "CAMPAIGN MONTH 4");
        assert!(presentation.status.starts_with("Scenario complete"));
        assert!(!presentation.status.contains("month 4 complete"));
    }

    #[test]
    fn quit_remains_available_while_pending_or_failed_and_has_persistent_status() {
        let mut state = ready(3);
        let request = state.begin_advance().unwrap();
        assert_eq!(
            availability(ObserverCommand::Quit, &state),
            ControlAvailability::Enabled
        );
        state.quit_requested = true;
        assert_eq!(inspection_availability(&state), CLOSING);
        assert!(turn_presentation(&state)
            .status
            .contains("Finishing the current week before closing"));
        assert!(matches!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Disabled(_)
        ));
        assert!(state.acknowledge(request, 4, None));
        assert!(state.installed(&state.context()));
        assert!(state.begin_advance().is_none());
        assert!(turn_presentation(&state)
            .status
            .starts_with("Closing the campaign"));
        state.fail("Lost connection".into());
        assert_eq!(
            availability(ObserverCommand::Quit, &state),
            ControlAvailability::Enabled
        );
    }

    #[test]
    fn pending_request_overrides_ready_after_a_perspective_refresh() {
        let mut state = ready(3);
        state.begin_advance().unwrap();
        state.set_perspective(Perspective::PlayerKnowledge);
        assert!(state.installed(&state.context()));
        assert_eq!(state.phase, SessionPhase::Ready);
        assert_eq!(availability(ObserverCommand::Step, &state), PENDING);
        assert_eq!(availability(ObserverCommand::Perspective, &state), PENDING);
        let presentation = turn_presentation(&state);
        assert_eq!(presentation.period, "CAMPAIGN MONTH 1");
        assert!(presentation.status.contains("Finishing week 4"));
        assert_eq!(presentation.step_label, "Processing week 4");
    }

    #[test]
    fn pause_and_resume_remain_available_until_inflight_acknowledgement() {
        let mut state = ready(3);
        let request = state.begin_advance().unwrap();
        state.playing = true;
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(turn_presentation(&state).play_label, "Pause");
        state.playing = false;
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Enabled
        );
        assert!(turn_presentation(&state)
            .status
            .contains("paused after this week"));
        assert!(!state.acknowledge(request + 1, 4, None));
        assert!(!state.acknowledge(request, 5, None));
        assert_eq!(turn_presentation(&state).period, "CAMPAIGN MONTH 1");
        assert!(state.acknowledge(request, 4, None));
        assert_eq!(turn_presentation(&state).period, "CAMPAIGN MONTH 1");
        assert!(turn_presentation(&state)
            .status
            .contains("Loading committed week 4"));
        assert!(matches!(
            availability(ObserverCommand::Step, &state),
            ControlAvailability::Disabled(_)
        ));
        state.playing = true;
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Enabled
        );
        assert!(state.installed(&state.context()));
        assert_eq!(
            availability(ObserverCommand::Step, &state),
            ControlAvailability::Enabled
        );
    }

    #[test]
    fn history_bounds_and_loading_have_explicit_reasons() {
        let mut state = ready(3);
        assert!(matches!(
            availability(ObserverCommand::NextWeek, &state),
            ControlAvailability::Disabled(_)
        ));
        assert!(matches!(
            availability(ObserverCommand::Live, &state),
            ControlAvailability::Disabled(_)
        ));
        state.inspect_tick(0);
        assert!(turn_presentation(&state)
            .status
            .contains("Loading committed week 0"));
        assert_eq!(turn_presentation(&state).step_label, "Advance unavailable");
        assert!(matches!(
            availability(ObserverCommand::PreviousWeek, &state),
            ControlAvailability::Disabled(_)
        ));
        assert!(state.installed(&state.context()));
        assert_eq!(availability(ObserverCommand::Step, &state), HISTORICAL);
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            HISTORICAL
        );
        assert_eq!(
            availability(ObserverCommand::NextWeek, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(
            availability(ObserverCommand::Live, &state),
            ControlAvailability::Enabled
        );
        assert!(matches!(
            availability(ObserverCommand::PreviousWeek, &state),
            ControlAvailability::Disabled(_)
        ));
        assert_eq!(
            turn_presentation(&state).period,
            "CAMPAIGN MONTH 1 / HISTORY"
        );
    }

    #[test]
    fn pending_blocks_context_changes_but_failure_reopen_can_reconcile() {
        let mut state = ready(3);
        state.begin_advance().unwrap();
        for command in [
            ObserverCommand::Perspective,
            ObserverCommand::PreviousWeek,
            ObserverCommand::NextWeek,
            ObserverCommand::Live,
            ObserverCommand::NewCampaign,
            ObserverCommand::NewDelayedCampaign,
            ObserverCommand::ReopenCampaign,
        ] {
            assert_eq!(availability(command, &state), PENDING);
        }
        state.fail("lost acknowledgement".into());
        assert!(state.advance_pending());
        assert_eq!(
            availability(ObserverCommand::ReopenCampaign, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(turn_presentation(&state).period, "CAMPAIGN MONTH 1");
        assert!(turn_presentation(&state)
            .status
            .contains("Reopen to reconcile"));
        state.ready(4, None);
        assert!(!state.advance_pending());
        assert!(state.installed(&state.context()));
        assert_eq!(
            availability(ObserverCommand::Step, &state),
            ControlAvailability::Enabled
        );
    }

    #[test]
    fn completion_keeps_history_and_disables_further_advances() {
        let mut state = ready(16);
        state.complete();
        for command in [ObserverCommand::Step, ObserverCommand::TogglePlay] {
            assert!(matches!(
                availability(command, &state),
                ControlAvailability::Disabled(_)
            ));
        }
        assert_eq!(
            availability(ObserverCommand::PreviousWeek, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(
            availability(ObserverCommand::History, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(turn_presentation(&state).step_label, "Scenario complete");
    }

    #[test]
    fn history_disclosure_remains_usable_during_pending_and_unavailable_views() {
        let mut state = ready(3);
        state.begin_advance().unwrap();
        for phase in [
            SessionPhase::Advancing,
            SessionPhase::Loading,
            SessionPhase::Ready,
            SessionPhase::Failed,
            SessionPhase::Closed,
        ] {
            state.phase = phase;
            assert!(state.advance_pending());
            assert_eq!(
                availability(ObserverCommand::History, &state),
                ControlAvailability::Enabled
            );
        }
    }

    #[test]
    fn final_pending_week_can_pause_but_cannot_promise_further_play() {
        let mut state = ready(15);
        state.horizon_tick = Some(16);
        state.begin_advance().unwrap();
        assert_eq!(turn_presentation(&state).period, "CAMPAIGN MONTH 4");
        assert_eq!(turn_presentation(&state).play_label, "Month unavailable");
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Disabled("This is the final week; further play is unavailable")
        );
        state.playing = true;
        assert_eq!(turn_presentation(&state).play_label, "Pause");
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Enabled
        );
    }
}
