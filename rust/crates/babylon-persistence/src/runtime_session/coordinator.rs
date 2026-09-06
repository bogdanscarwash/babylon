//! One pipe owner serializes lifecycle changes, durable ACKs and Archive reports.

use std::io::{BufRead, Write};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::Duration;

use super::input::{InputEvent, SessionInput};
use super::{
    emit, RuntimeSessionErrorCodeV3, RuntimeSessionRequestV3, RuntimeSessionResponseV3,
    RuntimeSessionScopeV3, RuntimeSessionTargetV3, SessionBackend,
    RUNTIME_SESSION_PROTOCOL_VERSION_V3,
};
use crate::archive_driver::{ArchiveDriverEventV1, ArchiveDriverRequestErrorV1, ArchiveDriverV1};
use crate::CampaignId;

mod active;
use active::Active;

const EVENT_CAPACITY: usize = 8;
const SHUTDOWN_GRACE: Duration = Duration::from_secs(150);
const COMPLETION_CHECK: Duration = Duration::from_millis(100);
type ArchiveEventSink = Box<dyn Fn(ArchiveDriverEventV1) -> bool + Send>;

#[derive(Debug)]
pub(super) enum SessionEvent {
    Input(InputEvent),
    Archive {
        scope: RuntimeSessionScopeV3,
        event: ArchiveDriverEventV1,
    },
}

pub(super) trait ArchiveControl {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCodeV3>;
    fn stop(&self);
    fn finished(&self) -> bool;
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV3>;
}
impl ArchiveControl for ArchiveDriverV1 {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCodeV3> {
        self.request_refresh(request_id)
            .map_err(|error| match error {
                ArchiveDriverRequestErrorV1::Full => RuntimeSessionErrorCodeV3::StorageBusy,
                ArchiveDriverRequestErrorV1::Stopped => RuntimeSessionErrorCodeV3::ArchiveRefused,
            })
    }
    fn stop(&self) {
        self.request_stop();
    }
    fn finished(&self) -> bool {
        self.is_finished()
    }
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV3> {
        match self.join_if_finished() {
            Some(Err(_) | Ok(Err(_))) => Err(RuntimeSessionErrorCodeV3::ArchiveRefused),
            Some(Ok(Ok(()))) | None => Ok(()),
        }
    }
}

struct Factories<F, G> {
    backend: F,
    archive: G,
}

pub(super) fn serve<I, W, B, D, F, G>(
    input: I,
    output: &mut W,
    backend: F,
    archive: G,
) -> Result<(), RuntimeSessionErrorCodeV3>
where
    I: BufRead + Send + 'static,
    W: Write,
    B: SessionBackend,
    D: ArchiveControl,
    F: FnMut(&RuntimeSessionTargetV3) -> Result<(B, String), RuntimeSessionErrorCodeV3>,
    G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCodeV3>,
{
    let (sender, events) = mpsc::sync_channel(EVENT_CAPACITY);
    let mut coordinator = Coordinator::new(output);
    emit(
        coordinator.output,
        &RuntimeSessionResponseV3::Hello {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V3,
            scope: coordinator.scope.clone(),
        },
    )?;
    let mut input = SessionInput::start(input, sender.clone())?;
    let mut factories = Factories { backend, archive };
    loop {
        match events.recv_timeout(COMPLETION_CHECK) {
            Ok(SessionEvent::Archive { scope, event }) => {
                coordinator.archive_event(&scope, &event)?;
            }
            Ok(SessionEvent::Input(InputEvent::Frame(bytes))) => {
                if let Some(request_id) =
                    coordinator.request(&bytes, &events, &sender, &mut factories)?
                {
                    input.stop();
                    return coordinator.shutdown(&events, Some(request_id), SHUTDOWN_GRACE);
                }
                input.next()?;
            }
            Ok(SessionEvent::Input(InputEvent::Eof)) => {
                input.stop();
                return coordinator.shutdown(&events, None, SHUTDOWN_GRACE);
            }
            Ok(SessionEvent::Input(InputEvent::Refused(code))) => {
                return coordinator.fail(None, code)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return coordinator.fail(None, RuntimeSessionErrorCodeV3::PipeFailure)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        coordinator.check_active_driver()?;
    }
}

struct Coordinator<'a, W: Write, B: SessionBackend, D: ArchiveControl> {
    output: &'a mut W,
    scope: RuntimeSessionScopeV3,
    active: Option<Active<B, D>>,
    last_request_id: u64,
}
impl<'a, W: Write, B: SessionBackend, D: ArchiveControl> Coordinator<'a, W, B, D> {
    fn new(output: &'a mut W) -> Self {
        Self {
            output,
            scope: RuntimeSessionScopeV3::default(),
            active: None,
            last_request_id: 0,
        }
    }

    fn refuse(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCodeV3,
    ) -> Result<(), RuntimeSessionErrorCodeV3> {
        emit(
            self.output,
            &RuntimeSessionResponseV3::Error {
                request_id,
                scope: self.scope.clone(),
                code,
                tail: self.active.as_ref().map(|active| active.backend.tail()),
            },
        )
    }
    fn fail(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCodeV3,
    ) -> Result<(), RuntimeSessionErrorCodeV3> {
        self.refuse(request_id, code)?;
        Err(code)
    }
    fn check_active_driver(&mut self) -> Result<(), RuntimeSessionErrorCodeV3> {
        if let Some(active) = &mut self.active {
            if active.archive.finished() {
                let code = active
                    .archive
                    .join_finished()
                    .err()
                    .unwrap_or(RuntimeSessionErrorCodeV3::ArchiveRefused);
                return self.fail(None, code);
            }
        }
        Ok(())
    }
    fn request<F, G>(
        &mut self,
        bytes: &[u8],
        events: &Receiver<SessionEvent>,
        sender: &SyncSender<SessionEvent>,
        factories: &mut Factories<F, G>,
    ) -> Result<Option<u64>, RuntimeSessionErrorCodeV3>
    where
        F: FnMut(&RuntimeSessionTargetV3) -> Result<(B, String), RuntimeSessionErrorCodeV3>,
        G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCodeV3>,
    {
        let Ok(request) = serde_json::from_slice::<RuntimeSessionRequestV3>(bytes) else {
            self.refuse(None, RuntimeSessionErrorCodeV3::InvalidRequest)?;
            return Ok(None);
        };
        let (version, request_id, scope) = request.header();
        let refusal = if version != RUNTIME_SESSION_PROTOCOL_VERSION_V3 {
            Some(RuntimeSessionErrorCodeV3::UnsupportedVersion)
        } else if *scope != self.scope {
            Some(RuntimeSessionErrorCodeV3::SessionMismatch)
        } else if request_id <= self.last_request_id {
            Some(RuntimeSessionErrorCodeV3::InvalidRequest)
        } else {
            None
        };
        if let Some(code) = refusal {
            self.refuse(Some(request_id), code)?;
            return Ok(None);
        }
        // Scope-valid IDs are consumed even when dispatch fails; switches never reset them.
        self.last_request_id = request_id;
        match request {
            RuntimeSessionRequestV3::Stop { request_id, .. } => return Ok(Some(request_id)),
            RuntimeSessionRequestV3::Switch { target, .. } => {
                self.switch(request_id, &target, events, sender, factories)?;
            }
            RuntimeSessionRequestV3::Advance { expected_tail, .. } => {
                let result = self
                    .active
                    .as_mut()
                    .ok_or(RuntimeSessionErrorCodeV3::CampaignAbsent)
                    .and_then(|active| active.backend.advance(&expected_tail));
                match result {
                    Ok(tail) => emit(
                        self.output,
                        &RuntimeSessionResponseV3::Committed {
                            request_id,
                            scope: self.scope.clone(),
                            tail,
                        },
                    )?,
                    Err(code) => self.refuse(Some(request_id), code)?,
                }
            }
            RuntimeSessionRequestV3::RefreshArchive { .. } => {
                let result = self
                    .active
                    .as_ref()
                    .ok_or(RuntimeSessionErrorCodeV3::CampaignAbsent)
                    .and_then(|active| active.archive.refresh(request_id));
                if let Err(code) = result {
                    self.refuse(Some(request_id), code)?;
                }
            }
        }
        Ok(None)
    }

    fn switch<F, G>(
        &mut self,
        request_id: u64,
        target: &RuntimeSessionTargetV3,
        events: &Receiver<SessionEvent>,
        sender: &SyncSender<SessionEvent>,
        factories: &mut Factories<F, G>,
    ) -> Result<(), RuntimeSessionErrorCodeV3>
    where
        F: FnMut(&RuntimeSessionTargetV3) -> Result<(B, String), RuntimeSessionErrorCodeV3>,
        G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCodeV3>,
    {
        let campaign = match target.campaign() {
            Ok(campaign) => campaign,
            Err(code) => return self.refuse(Some(request_id), code),
        };
        let Some(epoch) = self.scope.epoch.checked_add(1) else {
            return self.refuse(Some(request_id), RuntimeSessionErrorCodeV3::InvalidRequest);
        };
        let previous_scope = self.scope.clone();
        self.scope = RuntimeSessionScopeV3 {
            epoch,
            campaign_id: Some(campaign.as_uuid().to_string()),
        };
        emit(
            self.output,
            &RuntimeSessionResponseV3::Switching {
                request_id,
                previous_scope,
                scope: self.scope.clone(),
            },
        )?;
        // Old state is never relabeled; retirement succeeds before target admission.
        self.retire(events, false, Some(request_id), SHUTDOWN_GRACE)?;
        let (backend, foundation_digest) = match (factories.backend)(target) {
            Ok(value) => value,
            Err(code) => return self.refuse(Some(request_id), code),
        };
        let archive_sender = sender.clone();
        let scope = self.scope.clone();
        let sink: ArchiveEventSink = Box::new(move |event| {
            archive_sender
                .try_send(SessionEvent::Archive {
                    scope: scope.clone(),
                    event,
                })
                .is_ok()
        });
        let archive = match (factories.archive)(campaign, sink) {
            Ok(value) => value,
            Err(code) => return self.refuse(Some(request_id), code),
        };
        let tail = backend.tail();
        self.active = Some(Active::new(backend, archive));
        // Driver reports can be queued, but this ACK is always flushed first.
        emit(
            self.output,
            &RuntimeSessionResponseV3::Ready {
                request_id,
                scope: self.scope.clone(),
                foundation_digest,
                tail,
            },
        )
    }

    fn archive_event(
        &mut self,
        scope: &RuntimeSessionScopeV3,
        event: &ArchiveDriverEventV1,
    ) -> Result<(), RuntimeSessionErrorCodeV3> {
        if *scope != self.scope {
            return Ok(());
        }
        if matches!(event, ArchiveDriverEventV1::Stopped) {
            return self.fail(None, RuntimeSessionErrorCodeV3::ArchiveRefused);
        }
        if let Some(active) = &mut self.active {
            active.event(self.output, scope, event)?;
        }
        Ok(())
    }

    fn shutdown(
        &mut self,
        events: &Receiver<SessionEvent>,
        request_id: Option<u64>,
        grace: Duration,
    ) -> Result<(), RuntimeSessionErrorCodeV3> {
        self.retire(events, true, request_id, grace)?;
        if let Some(request_id) = request_id {
            emit(
                self.output,
                &RuntimeSessionResponseV3::Stopped {
                    request_id,
                    scope: self.scope.clone(),
                },
            )?;
        }
        Ok(())
    }

    fn retire(
        &mut self,
        events: &Receiver<SessionEvent>,
        expose_progress: bool,
        request_id: Option<u64>,
        grace: Duration,
    ) -> Result<(), RuntimeSessionErrorCodeV3> {
        let Some(mut active) = self.active.take() else {
            return Ok(());
        };
        if let Err(code) = active.retire(self.output, &self.scope, events, expose_progress, grace) {
            return self.fail(request_id, code);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
