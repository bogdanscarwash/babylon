//! Immutable observation context and transport for one durable campaign.
//! UI interactions schedule reads or one empty-action tick; they never own
//! a simulation session or mutate material state.

use babylon_persistence::CampaignId;
use bevy::prelude::*;

use crate::observer_calendar::CampaignMonth;

/// The two explicitly distinct read capabilities in the observer product.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Perspective {
    #[default]
    FullObserver,
    PlayerKnowledge,
}

impl Perspective {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FullObserver => "FULL OBSERVER",
            Self::PlayerKnowledge => "PLAYER KNOWLEDGE",
        }
    }
}

/// Every asynchronous read is bound to one immutable context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationContext {
    pub campaign: CampaignId,
    pub perspective: Perspective,
    pub tick: u64,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionPhase {
    #[default]
    Connecting,
    Loading,
    Ready,
    Advancing,
    Complete,
    Failed,
    Closed,
}

/// Scheduling state only. Authoritative time changes solely on acknowledgement.
#[derive(Resource, Debug)]
pub struct ObserverSession {
    pub campaign: CampaignId,
    pub perspective: Perspective,
    pub durable_tick: u64,
    pub viewed_tick: u64,
    pub archive_verified_tick: u64,
    pub horizon_tick: Option<u64>,
    pub content_hash: Option<String>,
    pub foundation_digest: Option<String>,
    pub phase: SessionPhase,
    pub playing: bool,
    pub quit_requested: bool,
    pub weeks_per_second: f64,
    pub error: Option<String>,
    pub generation: u64,
    month_plan: Option<CampaignMonth>,
    pending_request: Option<u64>,
    next_request: u64,
}

impl ObserverSession {
    #[must_use]
    pub const fn new(campaign: CampaignId) -> Self {
        Self {
            campaign,
            perspective: Perspective::FullObserver,
            durable_tick: 0,
            viewed_tick: 0,
            archive_verified_tick: 0,
            horizon_tick: None,
            content_hash: None,
            foundation_digest: None,
            phase: SessionPhase::Connecting,
            playing: false,
            quit_requested: false,
            weeks_per_second: 1.0,
            error: None,
            generation: 0,
            month_plan: None,
            pending_request: None,
            next_request: 1,
        }
    }

    #[must_use]
    pub const fn context(&self) -> ObservationContext {
        ObservationContext {
            campaign: self.campaign,
            perspective: self.perspective,
            tick: self.viewed_tick,
            generation: self.generation,
        }
    }

    #[must_use]
    pub fn accepts(&self, context: &ObservationContext) -> bool {
        self.context() == *context
    }

    /// A refreshed view can be ready while the runtime still owes an acknowledgement.
    #[must_use]
    pub(crate) const fn advance_pending(&self) -> bool {
        self.pending_request.is_some()
    }

    #[must_use]
    pub const fn month_plan(&self) -> Option<CampaignMonth> {
        self.month_plan
    }

    /// A shorter scenario stops mid-month without claiming a complete month.
    #[must_use]
    pub fn month_target_tick(&self) -> Option<u64> {
        self.month_plan.map(|month| {
            self.horizon_tick.map_or(month.closing_week, |horizon| {
                horizon.min(month.closing_week)
            })
        })
    }

    /// Queue only the remainder of this planning month, never unbounded play.
    /// A pending weekly commit still has to acknowledge and load before another.
    pub fn run_or_resume_month(&mut self) -> bool {
        if self.quit_requested
            || self.viewed_tick != self.durable_tick
            || !matches!(
                self.phase,
                SessionPhase::Ready | SessionPhase::Loading | SessionPhase::Advancing
            )
            || self
                .horizon_tick
                .is_some_and(|limit| self.durable_tick >= limit)
        {
            return false;
        }
        if self
            .month_target_tick()
            .is_none_or(|target| target <= self.durable_tick)
        {
            let Some(month) = CampaignMonth::after_week(self.durable_tick) else {
                return false;
            };
            self.month_plan = Some(month);
        }
        self.playing = true;
        true
    }

    /// Finish an outstanding week, preserving the uncompleted month target.
    pub const fn pause_month(&mut self) {
        self.playing = false;
    }

    /// An explicit weekly step or observation scope change ends the month plan.
    pub const fn cancel_month(&mut self) {
        self.pause_month();
        self.month_plan = None;
    }

    #[must_use]
    pub fn month_advance_due(&self) -> bool {
        self.playing
            && !self.quit_requested
            && self.phase == SessionPhase::Ready
            && !self.advance_pending()
            && self.viewed_tick == self.durable_tick
            && self
                .month_target_tick()
                .is_some_and(|target| self.durable_tick < target)
    }

    /// A runtime handshake reconciles any lost acknowledgement before play.
    pub fn ready(&mut self, tick: u64, hash: Option<String>) {
        self.durable_tick = tick;
        self.viewed_tick = tick;
        self.content_hash = hash;
        self.pending_request = None;
        self.error = None;
        self.cancel_month();
        self.invalidate();
    }

    pub fn installed(&mut self, context: &ObservationContext) -> bool {
        if !self.accepts(context) || self.phase != SessionPhase::Loading {
            return false;
        }
        self.phase = if self.viewed_tick == self.durable_tick
            && self
                .horizon_tick
                .is_some_and(|horizon| self.durable_tick >= horizon)
        {
            SessionPhase::Complete
        } else {
            SessionPhase::Ready
        };
        if self.phase == SessionPhase::Complete {
            self.pause_month();
        }
        true
    }

    /// A bounded scenario remains inspectable after its final committed week.
    pub fn complete(&mut self) {
        self.playing = false;
        self.pending_request = None;
        self.horizon_tick = Some(self.durable_tick);
        self.phase = SessionPhase::Complete;
    }

    pub fn begin_advance(&mut self) -> Option<u64> {
        if self.phase != SessionPhase::Ready
            || self.quit_requested
            || self.pending_request.is_some()
            || self.viewed_tick != self.durable_tick
        {
            return None;
        }
        let request = self.next_request;
        self.next_request = self.next_request.checked_add(1)?;
        self.pending_request = Some(request);
        self.phase = SessionPhase::Advancing;
        Some(request)
    }

    pub fn acknowledge(&mut self, request: u64, tick: u64, hash: Option<String>) -> bool {
        if self.pending_request != Some(request) || self.durable_tick.checked_add(1) != Some(tick) {
            return false;
        }
        self.pending_request = None;
        self.durable_tick = tick;
        self.viewed_tick = tick;
        self.content_hash = hash;
        if self
            .month_target_tick()
            .is_some_and(|target| tick >= target)
        {
            self.pause_month();
        }
        self.invalidate();
        true
    }

    pub fn set_perspective(&mut self, perspective: Perspective) {
        if self.perspective == perspective {
            return;
        }
        self.perspective = perspective;
        self.cancel_month();
        self.invalidate();
    }

    pub fn inspect_tick(&mut self, tick: u64) {
        if tick > self.durable_tick || self.pending_request.is_some() {
            return;
        }
        self.viewed_tick = tick;
        self.cancel_month();
        self.invalidate();
    }

    pub fn return_live(&mut self) {
        self.inspect_tick(self.durable_tick);
    }

    pub fn fail(&mut self, error: String) {
        self.pause_month();
        self.phase = SessionPhase::Failed;
        self.error = Some(error);
    }

    fn invalidate(&mut self) {
        if let Some(next) = self.generation.checked_add(1) {
            self.generation = next;
            self.phase = SessionPhase::Loading;
        } else {
            self.fail("Observation generation exhausted; reopen the campaign".into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(week: u64) -> ObserverSession {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(71)));
        state.ready(week, None);
        assert!(state.installed(&state.context()));
        state
    }

    fn commit(state: &mut ObserverSession) {
        let week = state.durable_tick + 1;
        let request = state.begin_advance().unwrap();
        assert!(state.begin_advance().is_none());
        assert!(state.acknowledge(request, week, None));
        assert!(
            !state.month_advance_due(),
            "exact observation must install first"
        );
        assert!(state.installed(&state.context()));
    }

    #[test]
    fn month_transport_stops_at_five_then_nine_without_an_extra_commit() {
        let mut state = ready(0);
        for endpoint in [5, 9] {
            assert!(state.run_or_resume_month());
            assert_eq!(state.month_target_tick(), Some(endpoint));
            while state.durable_tick < endpoint {
                assert!(state.month_advance_due());
                commit(&mut state);
            }
            assert_eq!(state.durable_tick, endpoint);
            assert!(!state.playing);
            assert!(!state.month_advance_due());
        }
    }

    #[test]
    fn paused_pending_week_finishes_but_resume_keeps_the_original_month_endpoint() {
        let mut state = ready(2);
        assert!(state.run_or_resume_month());
        let request = state.begin_advance().unwrap();
        state.pause_month();
        assert!(!state.acknowledge(request + 1, 3, None));
        assert!(!state.acknowledge(request, 4, None));
        assert_eq!(state.durable_tick, 2);
        assert!(state.acknowledge(request, 3, None));
        assert!(state.installed(&state.context()));
        assert!(!state.month_advance_due());
        assert_eq!(state.month_target_tick(), Some(5));
        assert!(state.run_or_resume_month());
        commit(&mut state);
        commit(&mut state);
        assert_eq!(state.durable_tick, 5);
        assert!(!state.month_advance_due());
    }

    #[test]
    fn scope_changes_discard_month_intent_without_discarding_an_outstanding_commit() {
        let mut state = ready(6);
        assert!(state.run_or_resume_month());
        let old = state.context();
        let request = state.begin_advance().unwrap();
        state.set_perspective(Perspective::PlayerKnowledge);
        assert!(state.month_plan().is_none());
        assert!(!state.accepts(&old));
        assert!(state.advance_pending());
        assert!(state.acknowledge(request, 7, None));
        assert!(state.installed(&state.context()));
        assert!(!state.month_advance_due());
        assert!(state.run_or_resume_month());
        state.inspect_tick(2);
        assert!(state.month_plan().is_none());
        assert!(!state.run_or_resume_month());
        state.return_live();
        assert!(state.installed(&state.context()));
        assert!(state.run_or_resume_month());
        assert_eq!(state.month_target_tick(), Some(9));
    }

    #[test]
    fn scenario_horizon_stops_mid_month_and_quit_never_schedules_more_work() {
        let mut state = ready(13);
        state.horizon_tick = Some(16);
        assert!(state.run_or_resume_month());
        assert_eq!(state.month_plan().unwrap().closing_week, 18);
        assert_eq!(state.month_target_tick(), Some(16));
        for _ in 0..3 {
            commit(&mut state);
        }
        assert_eq!(state.phase, SessionPhase::Complete);
        assert!(!state.run_or_resume_month());
        let mut closing = ready(2);
        assert!(closing.run_or_resume_month());
        let request = closing.begin_advance().unwrap();
        closing.quit_requested = true;
        assert!(closing.acknowledge(request, 3, None));
        assert!(closing.installed(&closing.context()));
        assert!(!closing.month_advance_due());
        assert!(!closing.run_or_resume_month());
    }

    #[test]
    fn lost_acknowledgement_reopen_uses_durable_progress_without_restarting_a_full_month() {
        let mut state = ready(6);
        assert!(state.run_or_resume_month());
        state.begin_advance().unwrap();
        state.fail("acknowledgement lost".into());
        assert!(state.advance_pending());
        assert!(!state.run_or_resume_month());
        state.ready(7, None);
        assert!(state.month_plan().is_none());
        assert!(state.installed(&state.context()));
        assert!(state.run_or_resume_month());
        assert_eq!(state.month_target_tick(), Some(9));
    }
}
