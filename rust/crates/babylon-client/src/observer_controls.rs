//! Shared button and dispatch decisions for one observer transport.
//! Only an acknowledged runtime commit changes the displayed durable period.

use crate::observer::{ObserverSession, SessionPhase};
use crate::observer_ui::ObserverCommand;
use babylon_kernel::clock::{DAYS_PER_TICK, TICKS_PER_YEAR, WEEKS_PER_TICK};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ControlAvailability {
    Enabled,
    Disabled(&'static str),
}

pub(crate) fn period_advance_help() -> String {
    format!("One period advances {WEEKS_PER_TICK} weeks / {DAYS_PER_TICK} days in one simulation tick. {TICKS_PER_YEAR} periods make a 52-week model year; these are not calendar months. Advance submits one period and then pauses. Play advances periods one at a time, waiting for each committed observation. Pause finishes the outstanding period. Disclosed freight loss pauses play; Stop on delivery adds delivery pauses. Production follows the previous period's committed plan. New deliveries enter planning for the following period. Time changes only after a confirmed commit.")
}

const PENDING: ControlAvailability =
    ControlAvailability::Disabled("Wait for the current period to finish committing");
const HISTORICAL: ControlAvailability =
    ControlAvailability::Disabled("Return Live before advancing the campaign");
const CLOSING: ControlAvailability = ControlAvailability::Disabled(
    "Closing the campaign; committed periods are saved automatically",
);

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
            "This is the final period; further play is unavailable",
        );
    }
    if state.durable_tick.checked_add(1).is_none() {
        return ControlAvailability::Disabled(
            "The campaign has reached its supported period limit",
        );
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
        Live, NewCampaign, NewDelayedCampaign, NewSharedFreightAmpleCampaign,
        NewSharedFreightConstrainedCampaign, NewStatewideBaselineCampaign,
        NewStatewideBothCampaign, NewStatewideFreightConstraintCampaign,
        NewStatewidePackagingShortageCampaign, NextPeriod, Perspective, PreviousPeriod,
        ReopenCampaign, Step, TogglePlay,
    };

    if command == ObserverCommand::Quit {
        return Enabled;
    }
    if state.quit_requested {
        return CLOSING;
    }
    if matches!(
        command,
        NewCampaign
            | NewDelayedCampaign
            | NewSharedFreightAmpleCampaign
            | NewSharedFreightConstrainedCampaign
            | NewStatewideBaselineCampaign
            | NewStatewideFreightConstraintCampaign
            | NewStatewidePackagingShortageCampaign
            | NewStatewideBothCampaign
            | ReopenCampaign
    ) && state.runtime_disconnected()
    {
        return Disabled("Runtime connection unavailable; close and relaunch Babylon");
    }

    // A lost acknowledgement retains the pending request. Deliberate reopen is
    // the recovery path; it reconciles durability before any further advance.
    if matches!(command, ReopenCampaign)
        && matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed)
    {
        return Enabled;
    }
    if state.advance_pending() && command == Step {
        return if matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed) {
            Disabled("Reopen the campaign to reconcile committed progress")
        } else {
            PENDING
        };
    }
    match command {
        Step | TogglePlay if state.lifecycle_pending() => {
            Disabled("Waiting for the selected campaign to open")
        }
        TogglePlay if state.playing => Enabled,
        TogglePlay => {
            let advance = advance_availability(state);
            if advance != Enabled {
                return advance;
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
        PreviousPeriod | NextPeriod | Live | Perspective => {
            let view = inspection_availability(state);
            if view != Enabled {
                return view;
            }
            match command {
                PreviousPeriod if state.viewed_tick == 0 => {
                    Disabled("Already at the opening period")
                }
                NextPeriod | Live if state.viewed_tick >= state.durable_tick => {
                    Disabled("Already viewing the live committed period")
                }
                _ => Enabled,
            }
        }
        ObserverCommand::RoadLayer(_) | ObserverCommand::NetworkSector(_) => {
            if state.perspective == crate::observer::Perspective::FullObserver {
                inspection_availability(state)
            } else {
                Disabled("Economic networks are unavailable in player knowledge")
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
        || "the requested period".to_owned(),
        |period| format!("period {period}"),
    );
    if state.playing {
        format!("Playing; advancing to {target}...")
    } else {
        format!("Finishing {target}; paused after this period.")
    }
}

fn turn_status(state: &ObserverSession) -> String {
    if state.quit_requested {
        return if state.advance_pending()
            && !matches!(state.phase, SessionPhase::Failed | SessionPhase::Closed)
        {
            "Finishing the current period before closing. Completed periods are saved automatically."
                .into()
        } else {
            "Closing the campaign. Completed periods are saved automatically.".into()
        };
    }
    if state.phase == SessionPhase::Failed {
        return if state.runtime_disconnected() {
            state.uncertain_campaign_id().map_or_else(
                || "Runtime connection closed. Close and relaunch Babylon.".into(),
                |campaign| format!("Runtime connection closed. Relaunch and Open requested campaign {campaign} to reconcile."),
            )
        } else {
            state
                .admission_notice()
                .unwrap_or("Campaign unavailable. Choose a campaign or Reopen to reconcile.")
                .into()
        };
    }
    if state.phase == SessionPhase::Closed {
        return "Campaign closed. Reopen to continue.".into();
    }
    if state.advance_pending() || state.phase == SessionPhase::Advancing {
        return pending_status(state);
    }
    if state.lifecycle_pending() {
        return "Opening the selected campaign...".into();
    }
    let historical = state.viewed_tick < state.durable_tick;
    match state.phase {
        SessionPhase::Connecting => "Opening the campaign...".into(),
        SessionPhase::Loading if historical => format!(
            "Loading committed period {} (live {}).",
            state.viewed_tick, state.durable_tick,
        ),
        SessionPhase::Loading if state.durable_tick == 0 => {
            "Loading the opening campaign observation...".into()
        }
        SessionPhase::Loading => format!("Loading committed period {}...", state.durable_tick),
        SessionPhase::Complete => "Scenario complete. History remains available.".into(),
        SessionPhase::Ready if historical => format!(
            "History {} / live {}. Return Live to advance.",
            state.viewed_tick, state.durable_tick
        ),
        SessionPhase::Ready if state.playing => "Playing; awaiting the next period.".into(),
        SessionPhase::Ready => "Paused. Advance one four-week period when ready.".into(),
        SessionPhase::Advancing | SessionPhase::Failed | SessionPhase::Closed => {
            unreachable!("handled above")
        }
    }
}

pub(crate) fn turn_presentation(state: &ObserverSession) -> TurnPresentation {
    let pending = state.advance_pending() || state.phase == SessionPhase::Advancing;
    let next = state.durable_tick.checked_add(1);
    let historical = state.viewed_tick < state.durable_tick;
    let period = format!(
        "PERIOD {}{}\n{WEEKS_PER_TICK} weeks / {DAYS_PER_TICK} days",
        state.viewed_tick,
        if historical { " / HISTORY" } else { "" },
    );
    let play_label = if state.playing {
        "Pause"
    } else if pending && pending_finishes_scenario(state) {
        "Play unavailable"
    } else {
        "Play"
    };
    let step_label = match state.phase {
        SessionPhase::Failed | SessionPhase::Closed | SessionPhase::Connecting => {
            "Advance unavailable".into()
        }
        _ if pending => next.map_or_else(
            || "Processing period".into(),
            |period| format!("Processing period {period}"),
        ),
        _ if historical => "Advance unavailable".into(),
        SessionPhase::Loading => format!("Loading period {}", state.viewed_tick),
        SessionPhase::Complete => "Scenario complete".into(),
        _ => next.map_or_else(
            || "Period limit reached".into(),
            |period| format!("Advance to period {period}"),
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
    use babylon_persistence::identity::CampaignId;

    fn ready(tick: u64) -> ObserverSession {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        state.ready(tick, None);
        assert!(state.installed(&state.context()));
        state
    }

    #[test]
    fn period_labels_identify_one_four_week_tick_and_a_thirteen_period_model_year() {
        let mut state = ready(0);
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 0\n4 weeks / 28 days"
        );
        assert_eq!(turn_presentation(&state).play_label, "Play");
        assert_eq!(turn_presentation(&state).step_label, "Advance to period 1");
        let request = state.begin_advance().unwrap();
        assert!(state.acknowledge(request, 1, None));
        assert!(state.installed(&state.context()));
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 1\n4 weeks / 28 days"
        );
        assert!(!state.playing);
        assert!(turn_presentation(&state).status.starts_with("Paused."));
        assert_eq!(
            turn_presentation(&ready(13)).period,
            "PERIOD 13\n4 weeks / 28 days"
        );
        assert_eq!(
            turn_presentation(&ready(14)).period,
            "PERIOD 14\n4 weeks / 28 days"
        );
        assert!(period_advance_help().contains("13 periods make a 52-week model year"));
        state.horizon_tick = Some(1);
        state.complete();
        assert!(turn_presentation(&state)
            .status
            .starts_with("Scenario complete"));
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
            .contains("Finishing the current period before closing"));
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
        assert_eq!(presentation.period, "PERIOD 3\n4 weeks / 28 days");
        assert!(presentation.status.contains("Finishing period 4"));
        assert_eq!(presentation.step_label, "Processing period 4");
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
            .contains("paused after this period"));
        assert!(!state.acknowledge(request + 1, 4, None));
        assert!(!state.acknowledge(request, 5, None));
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 3\n4 weeks / 28 days"
        );
        assert!(state.acknowledge(request, 4, None));
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 4\n4 weeks / 28 days"
        );
        assert!(turn_presentation(&state)
            .status
            .contains("Loading committed period 4"));
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
            availability(ObserverCommand::NextPeriod, &state),
            ControlAvailability::Disabled(_)
        ));
        assert!(matches!(
            availability(ObserverCommand::Live, &state),
            ControlAvailability::Disabled(_)
        ));
        state.inspect_tick(0);
        assert!(turn_presentation(&state)
            .status
            .contains("Loading committed period 0"));
        assert_eq!(turn_presentation(&state).step_label, "Advance unavailable");
        assert!(matches!(
            availability(ObserverCommand::PreviousPeriod, &state),
            ControlAvailability::Disabled(_)
        ));
        assert!(state.installed(&state.context()));
        assert_eq!(availability(ObserverCommand::Step, &state), HISTORICAL);
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            HISTORICAL
        );
        assert_eq!(
            availability(ObserverCommand::NextPeriod, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(
            availability(ObserverCommand::Live, &state),
            ControlAvailability::Enabled
        );
        assert!(matches!(
            availability(ObserverCommand::PreviousPeriod, &state),
            ControlAvailability::Disabled(_)
        ));
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 0 / HISTORY\n4 weeks / 28 days"
        );
    }

    #[test]
    fn pending_blocks_context_changes_but_failure_reopen_can_reconcile() {
        let mut state = ready(3);
        state.begin_advance().unwrap();
        for command in [
            ObserverCommand::Perspective,
            ObserverCommand::PreviousPeriod,
            ObserverCommand::NextPeriod,
            ObserverCommand::Live,
        ] {
            assert_eq!(availability(command, &state), PENDING);
        }
        for command in [
            ObserverCommand::NewCampaign,
            ObserverCommand::NewDelayedCampaign,
            ObserverCommand::NewSharedFreightAmpleCampaign,
            ObserverCommand::NewSharedFreightConstrainedCampaign,
            ObserverCommand::ReopenCampaign,
        ] {
            assert_eq!(availability(command, &state), ControlAvailability::Enabled);
        }
        state.fail("lost acknowledgement".into());
        assert!(state.advance_pending());
        assert_eq!(
            availability(ObserverCommand::ReopenCampaign, &state),
            ControlAvailability::Enabled
        );
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 3\n4 weeks / 28 days"
        );
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
            availability(ObserverCommand::PreviousPeriod, &state),
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
    fn final_pending_period_can_pause_but_cannot_promise_further_play() {
        let mut state = ready(15);
        state.horizon_tick = Some(16);
        state.begin_advance().unwrap();
        assert_eq!(
            turn_presentation(&state).period,
            "PERIOD 15\n4 weeks / 28 days"
        );
        assert_eq!(turn_presentation(&state).play_label, "Play unavailable");
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Disabled("This is the final period; further play is unavailable")
        );
        state.playing = true;
        assert_eq!(turn_presentation(&state).play_label, "Pause");
        assert_eq!(
            availability(ObserverCommand::TogglePlay, &state),
            ControlAvailability::Enabled
        );
    }
}
