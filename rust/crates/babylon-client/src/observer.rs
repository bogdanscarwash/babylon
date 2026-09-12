//! Immutable observation context and transport for one durable campaign.
//! UI interactions schedule reads or one empty-action tick; they never own
//! a simulation session or mutate material state.

mod lifecycle;

use babylon_persistence::identity::CampaignId;
use bevy::prelude::*;

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
    pub periods_per_second: f64,
    pub error: Option<String>,
    pub generation: u64,
    pending_request: Option<u64>,
    next_request: u64,
    pub(crate) lifecycle: lifecycle::LifecycleState,
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
            periods_per_second: 1.0,
            error: None,
            generation: 0,
            pending_request: None,
            next_request: 1,
            lifecycle: lifecycle::LifecycleState::new(),
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

    /// Play one four-week period at a time, awaiting its commit and observation.
    pub fn start_playback(&mut self) -> bool {
        if self.quit_requested
            || self.viewed_tick != self.durable_tick
            || self.lifecycle_pending()
            || self.durable_tick.checked_add(1).is_none()
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
        self.playing = true;
        true
    }

    /// Finish an outstanding period; do not schedule another.
    pub const fn pause_playback(&mut self) {
        self.playing = false;
    }

    #[must_use]
    pub fn playback_due(&self) -> bool {
        self.playing
            && !self.quit_requested
            && self.phase == SessionPhase::Ready
            && !self.advance_pending()
            && !self.lifecycle_pending()
            && self.viewed_tick == self.durable_tick
            && self.durable_tick.checked_add(1).is_some()
            && self
                .horizon_tick
                .is_none_or(|limit| self.durable_tick < limit)
    }

    /// A runtime handshake reconciles any lost acknowledgement before play.
    pub fn ready(&mut self, tick: u64, hash: Option<String>) {
        self.durable_tick = tick;
        self.viewed_tick = tick;
        self.content_hash = hash;
        self.pending_request = None;
        self.error = None;
        self.pause_playback();
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
            self.pause_playback();
        }
        true
    }

    /// A bounded scenario remains inspectable after its final committed period.
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
            || self.lifecycle_pending()
            || self.viewed_tick != self.durable_tick
            || self.durable_tick.checked_add(1).is_none()
            || self
                .horizon_tick
                .is_some_and(|limit| self.durable_tick >= limit)
        {
            return None;
        }
        let request = self.next_control_request()?;
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
        self.invalidate();
        true
    }

    pub fn set_perspective(&mut self, perspective: Perspective) {
        if self.perspective == perspective {
            return;
        }
        self.perspective = perspective;
        self.pause_playback();
        self.invalidate();
    }

    pub fn inspect_tick(&mut self, tick: u64) {
        if tick > self.durable_tick || self.pending_request.is_some() {
            return;
        }
        self.viewed_tick = tick;
        self.pause_playback();
        self.invalidate();
    }

    pub fn return_live(&mut self) {
        self.inspect_tick(self.durable_tick);
    }

    pub fn fail(&mut self, error: String) {
        self.pause_playback();
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

    fn ready(period: u64) -> ObserverSession {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(71)));
        state.ready(period, None);
        assert!(state.installed(&state.context()));
        state
    }

    fn commit(state: &mut ObserverSession) {
        let period = state.durable_tick + 1;
        let request = state.begin_advance().unwrap();
        assert!(state.begin_advance().is_none());
        assert!(state.acknowledge(request, period, None));
        assert!(
            !state.playback_due(),
            "exact observation must install first"
        );
        assert!(state.installed(&state.context()));
    }

    #[test]
    fn one_advance_has_one_acknowledged_four_week_period() {
        let mut state = ready(0);
        let request = state.begin_advance().unwrap();
        assert!(state.begin_advance().is_none());
        assert!(!state.acknowledge(request, 4, None));
        assert!(!state.acknowledge(request + 1, 1, None));
        assert_eq!(state.durable_tick, 0);
        assert!(state.acknowledge(request, 1, None));
        assert_eq!(state.durable_tick, 1);
        assert_eq!(babylon_kernel::clock::DAYS_PER_TICK, 28);
        assert_eq!(babylon_kernel::clock::WEEKS_PER_TICK, 4);
        assert!(state.installed(&state.context()));
        assert!(
            !state.playback_due(),
            "a single advance cannot queue another"
        );
    }

    #[test]
    fn playback_waits_for_each_commit_and_pause_finishes_only_the_outstanding_period() {
        let mut state = ready(2);
        assert!(state.start_playback());
        commit(&mut state);
        assert!(state.playback_due());
        let request = state.begin_advance().unwrap();
        state.pause_playback();
        assert!(state.advance_pending());
        assert!(state.acknowledge(request, 4, None));
        assert!(state.installed(&state.context()));
        assert!(!state.playback_due());
        assert!(state.start_playback());
        commit(&mut state);
        assert_eq!(state.durable_tick, 5);
    }

    #[test]
    fn scope_changes_pause_playback_without_discarding_an_outstanding_commit() {
        let mut state = ready(6);
        assert!(state.start_playback());
        let old = state.context();
        let request = state.begin_advance().unwrap();
        state.set_perspective(Perspective::PlayerKnowledge);
        assert!(!state.playing);
        assert!(!state.accepts(&old));
        assert!(state.advance_pending());
        assert!(state.acknowledge(request, 7, None));
        assert!(state.installed(&state.context()));
        assert!(!state.playback_due());
        assert!(state.start_playback());
        state.inspect_tick(2);
        assert!(!state.playing);
        assert!(!state.start_playback());
        state.return_live();
        assert!(state.installed(&state.context()));
        assert!(state.start_playback());
    }

    #[test]
    fn scenario_horizon_and_quit_stop_playback_without_an_extra_commit() {
        let mut state = ready(3);
        state.horizon_tick = Some(4);
        assert!(state.start_playback());
        commit(&mut state);
        assert_eq!(state.phase, SessionPhase::Complete);
        assert!(!state.playback_due());
        assert!(!state.start_playback());
        let mut closing = ready(2);
        assert!(closing.start_playback());
        let request = closing.begin_advance().unwrap();
        closing.quit_requested = true;
        assert!(closing.acknowledge(request, 3, None));
        assert!(closing.installed(&closing.context()));
        assert!(!closing.playback_due());
        assert!(!closing.start_playback());
    }

    #[test]
    fn lost_acknowledgement_reopen_reconciles_without_replaying_a_period() {
        let mut state = ready(6);
        assert!(state.start_playback());
        state.begin_advance().unwrap();
        state.fail("acknowledgement lost".into());
        assert!(state.advance_pending());
        assert!(!state.start_playback());
        state.ready(7, None);
        assert!(!state.playing);
        assert!(!state.advance_pending());
        assert!(state.installed(&state.context()));
        assert_eq!(state.durable_tick, 7);
        assert!(!state.playback_due());
        assert!(state.start_playback());
        commit(&mut state);
        assert_eq!(state.durable_tick, 8);
    }

    #[test]
    fn exhausted_period_counter_cannot_schedule_playback() {
        let mut state = ready(u64::MAX);
        assert!(!state.start_playback());
        assert!(state.begin_advance().is_none());
        state.playing = true;
        assert!(!state.playback_due());
    }
}
