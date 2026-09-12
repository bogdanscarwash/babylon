//! Client scheduling for the one persistent runtime control connection.

use babylon_persistence::{
    identity::CampaignId, runtime_session::RuntimeSessionErrorCode,
    runtime_session::RuntimeSessionRequest, runtime_session::RuntimeSessionScope,
    runtime_session::RuntimeSessionTail, runtime_session::RuntimeSessionTarget,
    runtime_session::RUNTIME_SESSION_PROTOCOL_VERSION,
};

use super::{ObserverSession, SessionPhase};

#[derive(Debug)]
struct PendingSwitch {
    request_id: u64,
    previous: RuntimeSessionScope,
    target: RuntimeSessionTarget,
    sent: bool,
    accepted: bool,
}

#[derive(Debug)]
pub(crate) struct LifecycleState {
    scope: Option<RuntimeSessionScope>,
    queued: Option<RuntimeSessionTarget>,
    switching: Option<PendingSwitch>,
    stop: Option<(u64, bool)>,
    disconnected: bool,
    last_error: Option<RuntimeSessionErrorCode>,
}

impl LifecycleState {
    pub(super) const fn new() -> Self {
        Self {
            scope: None,
            queued: None,
            switching: None,
            stop: None,
            disconnected: false,
            last_error: None,
        }
    }
}

fn target_campaign(target: &RuntimeSessionTarget) -> Result<CampaignId, String> {
    let (RuntimeSessionTarget::New { campaign_id, .. }
    | RuntimeSessionTarget::Open { campaign_id }) = target;
    let uuid = uuid::Uuid::parse_str(campaign_id).map_err(|_| "Invalid campaign identity")?;
    if uuid.is_nil() || uuid.to_string() != *campaign_id {
        return Err("Invalid campaign identity".into());
    }
    Ok(CampaignId::from_uuid(uuid))
}

impl ObserverSession {
    /// Queue the explicitly selected New or Open operation until the service says Hello.
    ///
    /// # Errors
    /// Refuses a noncanonical or nil campaign UUID.
    pub fn with_initial_target(target: RuntimeSessionTarget) -> Result<Self, String> {
        let mut session = Self::new(target_campaign(&target)?);
        session.queue_campaign(target)?;
        Ok(session)
    }

    pub(crate) fn queue_campaign(&mut self, target: RuntimeSessionTarget) -> Result<(), String> {
        target_campaign(&target)?;
        if self.quit_requested || self.lifecycle.disconnected {
            return Err("Runtime connection unavailable; close and relaunch Babylon.".into());
        }
        self.pause_playback();
        if self
            .lifecycle
            .switching
            .as_ref()
            .is_some_and(|pending| !pending.sent)
        {
            self.lifecycle.switching = None;
        }
        self.lifecycle.queued = Some(target);
        Ok(())
    }

    pub(crate) fn lifecycle_pending(&self) -> bool {
        self.lifecycle.queued.is_some() || self.lifecycle.switching.is_some()
    }

    pub(crate) fn switch_send_due(&self) -> bool {
        !self.quit_requested
            && !self.advance_pending()
            && !self.lifecycle.disconnected
            && self.lifecycle.scope.is_some()
            && self
                .lifecycle
                .switching
                .as_ref()
                .map_or(self.lifecycle.queued.is_some(), |pending| !pending.sent)
    }

    pub(crate) fn lifecycle_epoch(&self) -> Option<u64> {
        self.lifecycle.scope.as_ref().map(|scope| scope.epoch)
    }

    pub(crate) fn runtime_scope(&self) -> Option<&RuntimeSessionScope> {
        self.lifecycle.scope.as_ref()
    }

    pub(crate) fn next_control_request(&mut self) -> Option<u64> {
        let request = self.next_request;
        self.next_request = if let Some(next) = request.checked_add(1) {
            next
        } else {
            self.disconnect(
                "Runtime request counter exhausted; close and relaunch Babylon.".into(),
            );
            return None;
        };
        Some(request)
    }

    pub(crate) fn hello(&mut self, scope: RuntimeSessionScope) -> Result<(), String> {
        if self.lifecycle.scope.is_some() || scope.epoch != 0 || scope.campaign_id.is_some() {
            return Err("Unexpected runtime Hello scope".into());
        }
        self.lifecycle.scope = Some(scope);
        Ok(())
    }

    pub(crate) fn pending_switch_request(&mut self) -> Option<RuntimeSessionRequest> {
        if self.quit_requested || self.advance_pending() || self.lifecycle.disconnected {
            return None;
        }
        if self.lifecycle.switching.is_none() {
            let previous = self.lifecycle.scope.clone()?;
            self.lifecycle.queued.as_ref()?;
            let request_id = self.next_control_request()?;
            let target = self.lifecycle.queued.take()?;
            self.lifecycle.switching = Some(PendingSwitch {
                request_id,
                previous,
                target,
                sent: false,
                accepted: false,
            });
        }
        let pending = self.lifecycle.switching.as_ref()?;
        if pending.sent {
            return None;
        }
        Some(RuntimeSessionRequest::Switch {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id: pending.request_id,
            scope: pending.previous.clone(),
            target: pending.target.clone(),
        })
    }

    pub(crate) fn switch_sent(&mut self) {
        if let Some(pending) = &mut self.lifecycle.switching {
            pending.sent = true;
        }
    }

    pub(crate) fn switching(
        &mut self,
        request_id: u64,
        previous: &RuntimeSessionScope,
        scope: RuntimeSessionScope,
    ) -> Result<(), String> {
        let pending = self
            .lifecycle
            .switching
            .as_ref()
            .ok_or("Unsolicited campaign switch")?;
        let campaign = target_campaign(&pending.target)?;
        if !pending.sent
            || pending.accepted
            || pending.request_id != request_id
            || &pending.previous != previous
            || self.runtime_scope() != Some(previous)
            || previous.epoch.checked_add(1) != Some(scope.epoch)
            || scope.campaign_id.as_deref() != Some(campaign.as_uuid().to_string().as_str())
        {
            return Err("Campaign switch identity did not match its request".into());
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or("Observation generation exhausted")?;
        self.lifecycle
            .switching
            .as_mut()
            .expect("validated switch")
            .accepted = true;
        self.lifecycle.scope = Some(scope);
        self.campaign = campaign;
        self.generation = generation;
        self.durable_tick = 0;
        self.viewed_tick = 0;
        self.archive_verified_tick = 0;
        self.horizon_tick = None;
        self.content_hash = None;
        self.foundation_digest = None;
        self.error = None;
        self.lifecycle.last_error = None;
        self.pause_playback();
        self.phase = SessionPhase::Connecting;
        Ok(())
    }

    pub(crate) fn admitted(
        &mut self,
        request_id: u64,
        foundation_digest: String,
        tail: RuntimeSessionTail,
    ) -> Result<(), String> {
        let pending = self
            .lifecycle
            .switching
            .as_ref()
            .ok_or("Unsolicited campaign Ready")?;
        if !pending.accepted || pending.request_id != request_id {
            return Err("Campaign Ready did not match its accepted switch".into());
        }
        self.generation
            .checked_add(1)
            .ok_or("Observation generation exhausted")?;
        self.lifecycle.switching = None;
        self.foundation_digest = Some(foundation_digest);
        self.ready(tail.resolve_tick, tail.tick_content_hash);
        Ok(())
    }

    pub(crate) fn refuse_request(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCode,
    ) -> bool {
        if self
            .lifecycle
            .switching
            .as_ref()
            .is_some_and(|pending| Some(pending.request_id) == request_id)
        {
            self.lifecycle.switching = None;
        } else if self.pending_request.is_some() && self.pending_request == request_id {
            self.pending_request = None;
        } else if request_id.is_some() && self.lifecycle.stop.map(|(id, _)| id) != request_id {
            return false;
        }
        self.lifecycle.last_error = Some(code);
        self.fail(
            self.admission_notice()
                .map_or_else(|| code.to_string(), str::to_owned),
        );
        true
    }

    pub(crate) fn admission_notice(&self) -> Option<&'static str> {
        match self.lifecycle.last_error {
            Some(RuntimeSessionErrorCode::CampaignAbsent) => {
                Some("The selected campaign was not found. Choose another campaign or create New.")
            }
            Some(RuntimeSessionErrorCode::CampaignAlreadyExists) => {
                Some("That campaign already exists. Choose Open to continue it.")
            }
            _ => None,
        }
    }

    pub(crate) fn uncertain_campaign_id(&self) -> Option<&str> {
        let pending = self
            .lifecycle
            .switching
            .as_ref()
            .filter(|pending| pending.sent)?;
        let (RuntimeSessionTarget::New { campaign_id, .. }
        | RuntimeSessionTarget::Open { campaign_id }) = &pending.target;
        Some(campaign_id)
    }

    pub(crate) fn disconnect(&mut self, error: String) {
        let error = match self.uncertain_campaign_id() {
            Some(campaign) => format!("{error} Requested campaign: {campaign}. Relaunch and Open this identity to reconcile."),
            None => error,
        };
        self.lifecycle.disconnected = true;
        self.lifecycle.queued = None;
        if self
            .lifecycle
            .switching
            .as_ref()
            .is_some_and(|pending| !pending.sent)
        {
            self.lifecycle.switching = None;
        }
        self.fail(error);
    }

    pub(crate) fn pending_stop_request(&mut self) -> Option<RuntimeSessionRequest> {
        self.lifecycle.queued = None;
        if self
            .lifecycle
            .switching
            .as_ref()
            .is_some_and(|pending| !pending.sent)
        {
            self.lifecycle.switching = None;
        }
        if self.lifecycle.switching.is_some() || self.lifecycle.disconnected {
            return None;
        }
        let scope = self.lifecycle.scope.clone()?;
        let (request_id, sent) = if let Some(stop) = self.lifecycle.stop {
            stop
        } else {
            let id = self.next_control_request()?;
            self.lifecycle.stop = Some((id, false));
            (id, false)
        };
        if sent {
            return None;
        }
        Some(RuntimeSessionRequest::Stop {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
            request_id,
            scope,
        })
    }

    pub(crate) fn stop_sent(&mut self) {
        if let Some((_, sent)) = &mut self.lifecycle.stop {
            *sent = true;
        }
    }

    pub(crate) fn stopped(&mut self, request_id: u64) -> bool {
        if !self.quit_requested
            || self.advance_pending()
            || self.lifecycle.stop != Some((request_id, true))
        {
            return false;
        }
        self.phase = SessionPhase::Closed;
        self.playing = false;
        true
    }

    pub(crate) const fn runtime_disconnected(&self) -> bool {
        self.lifecycle.disconnected
    }

    #[cfg(test)]
    pub(crate) fn connected_fixture(&mut self) {
        self.lifecycle.scope = Some(RuntimeSessionScope {
            epoch: 1,
            campaign_id: Some(self.campaign.as_uuid().to_string()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_scope_or_observation_generation_never_reuses_an_old_context() {
        for exhausted_epoch in [false, true] {
            let campaign = CampaignId::from_uuid(uuid::Uuid::from_u128(1));
            let mut state = ObserverSession::new(campaign);
            state.connected_fixture();
            if exhausted_epoch {
                state.lifecycle.scope.as_mut().unwrap().epoch = u64::MAX;
            } else {
                state.generation = u64::MAX;
            }
            let original = state.context();
            state
                .queue_campaign(RuntimeSessionTarget::Open {
                    campaign_id: campaign.as_uuid().to_string(),
                })
                .unwrap();
            let RuntimeSessionRequest::Switch {
                request_id, scope, ..
            } = state.pending_switch_request().unwrap()
            else {
                panic!("switch request");
            };
            state.switch_sent();
            let next = RuntimeSessionScope {
                epoch: if exhausted_epoch { 0 } else { 2 },
                campaign_id: scope.campaign_id.clone(),
            };
            assert!(state.switching(request_id, &scope, next).is_err());
            assert_eq!(state.context(), original);
            assert_eq!(state.runtime_scope(), Some(&scope));
        }
    }

    #[test]
    fn request_counter_exhaustion_refuses_without_resending_or_wrapping() {
        let mut state = ObserverSession::new(CampaignId::from_uuid(uuid::Uuid::from_u128(1)));
        state.connected_fixture();
        state.next_request = u64::MAX;
        state
            .queue_campaign(RuntimeSessionTarget::Open {
                campaign_id: state.campaign.as_uuid().to_string(),
            })
            .unwrap();
        assert!(state.pending_switch_request().is_none());
        assert!(state.runtime_disconnected());
        assert!(!state.switch_send_due());
        assert_eq!(state.next_request, u64::MAX);
    }
}
