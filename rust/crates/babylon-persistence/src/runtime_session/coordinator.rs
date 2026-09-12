//! One pipe owner serializes lifecycle changes, durable ACKs and Archive reports.

use std::io::{BufRead, Write};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::Duration;

use super::input::{InputEvent, SessionInput};
use super::{
    emit, RuntimeSessionErrorCode, RuntimeSessionRequest, RuntimeSessionResponse,
    RuntimeSessionScope, RuntimeSessionTarget, SessionBackend, RUNTIME_SESSION_PROTOCOL_VERSION,
};
use crate::archive_driver::{ArchiveDriver, ArchiveDriverEvent, ArchiveDriverRequestError};
use crate::identity::CampaignId;

mod active;
use active::Active;

const EVENT_CAPACITY: usize = 8;
const SHUTDOWN_GRACE: Duration = Duration::from_secs(150);
const COMPLETION_CHECK: Duration = Duration::from_millis(100);
type ArchiveEventSink = Box<dyn Fn(ArchiveDriverEvent) -> bool + Send>;

#[derive(Debug)]
pub(super) enum SessionEvent {
    Input(InputEvent),
    Archive {
        scope: RuntimeSessionScope,
        event: ArchiveDriverEvent,
    },
}

pub(super) trait ArchiveControl {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCode>;
    fn stop(&self);
    fn finished(&self) -> bool;
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCode>;
}
impl ArchiveControl for ArchiveDriver {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCode> {
        self.request_refresh(request_id)
            .map_err(|error| match error {
                ArchiveDriverRequestError::Full => RuntimeSessionErrorCode::StorageBusy,
                ArchiveDriverRequestError::Stopped => RuntimeSessionErrorCode::ArchiveRefused,
            })
    }
    fn stop(&self) {
        self.request_stop();
    }
    fn finished(&self) -> bool {
        self.is_finished()
    }
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCode> {
        match self.join_if_finished() {
            Some(Err(_) | Ok(Err(_))) => Err(RuntimeSessionErrorCode::ArchiveRefused),
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
) -> Result<(), RuntimeSessionErrorCode>
where
    I: BufRead + Send + 'static,
    W: Write,
    B: SessionBackend,
    D: ArchiveControl,
    F: FnMut(&RuntimeSessionTarget) -> Result<(B, String), RuntimeSessionErrorCode>,
    G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCode>,
{
    let (sender, events) = mpsc::sync_channel(EVENT_CAPACITY);
    let mut coordinator = Coordinator::new(output);
    emit(
        coordinator.output,
        &RuntimeSessionResponse::Hello {
            protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION,
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
                return coordinator.fail(None, RuntimeSessionErrorCode::PipeFailure)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        coordinator.check_active_driver()?;
    }
}

struct Coordinator<'a, W: Write, B: SessionBackend, D: ArchiveControl> {
    output: &'a mut W,
    scope: RuntimeSessionScope,
    active: Option<Active<B, D>>,
    last_request_id: u64,
}
impl<'a, W: Write, B: SessionBackend, D: ArchiveControl> Coordinator<'a, W, B, D> {
    fn new(output: &'a mut W) -> Self {
        Self {
            output,
            scope: RuntimeSessionScope::default(),
            active: None,
            last_request_id: 0,
        }
    }

    fn refuse(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCode,
    ) -> Result<(), RuntimeSessionErrorCode> {
        emit(
            self.output,
            &RuntimeSessionResponse::Error {
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
        code: RuntimeSessionErrorCode,
    ) -> Result<(), RuntimeSessionErrorCode> {
        self.refuse(request_id, code)?;
        Err(code)
    }
    fn check_active_driver(&mut self) -> Result<(), RuntimeSessionErrorCode> {
        if let Some(active) = &mut self.active {
            if active.archive.finished() {
                let code = active
                    .archive
                    .join_finished()
                    .err()
                    .unwrap_or(RuntimeSessionErrorCode::ArchiveRefused);
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
    ) -> Result<Option<u64>, RuntimeSessionErrorCode>
    where
        F: FnMut(&RuntimeSessionTarget) -> Result<(B, String), RuntimeSessionErrorCode>,
        G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCode>,
    {
        let Ok(request) = serde_json::from_slice::<RuntimeSessionRequest>(bytes) else {
            self.refuse(None, RuntimeSessionErrorCode::InvalidRequest)?;
            return Ok(None);
        };
        let (version, request_id, scope) = request.header();
        let refusal = if version != RUNTIME_SESSION_PROTOCOL_VERSION {
            Some(RuntimeSessionErrorCode::UnsupportedVersion)
        } else if *scope != self.scope {
            Some(RuntimeSessionErrorCode::SessionMismatch)
        } else if request_id <= self.last_request_id {
            Some(RuntimeSessionErrorCode::InvalidRequest)
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
            RuntimeSessionRequest::Stop { request_id, .. } => return Ok(Some(request_id)),
            RuntimeSessionRequest::Switch { target, .. } => {
                self.switch(request_id, &target, events, sender, factories)?;
            }
            RuntimeSessionRequest::Advance { expected_tail, .. } => {
                let result = self
                    .active
                    .as_mut()
                    .ok_or(RuntimeSessionErrorCode::CampaignAbsent)
                    .and_then(|active| active.backend.advance(&expected_tail));
                match result {
                    Ok(tail) => emit(
                        self.output,
                        &RuntimeSessionResponse::Committed {
                            request_id,
                            scope: self.scope.clone(),
                            tail,
                        },
                    )?,
                    Err(code) => self.refuse(Some(request_id), code)?,
                }
            }
            RuntimeSessionRequest::RefreshArchive { .. } => {
                let result = self
                    .active
                    .as_ref()
                    .ok_or(RuntimeSessionErrorCode::CampaignAbsent)
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
        target: &RuntimeSessionTarget,
        events: &Receiver<SessionEvent>,
        sender: &SyncSender<SessionEvent>,
        factories: &mut Factories<F, G>,
    ) -> Result<(), RuntimeSessionErrorCode>
    where
        F: FnMut(&RuntimeSessionTarget) -> Result<(B, String), RuntimeSessionErrorCode>,
        G: FnMut(CampaignId, ArchiveEventSink) -> Result<D, RuntimeSessionErrorCode>,
    {
        let campaign = match target.campaign() {
            Ok(campaign) => campaign,
            Err(code) => return self.refuse(Some(request_id), code),
        };
        let Some(epoch) = self.scope.epoch.checked_add(1) else {
            return self.refuse(Some(request_id), RuntimeSessionErrorCode::InvalidRequest);
        };
        let previous_scope = self.scope.clone();
        self.scope = RuntimeSessionScope {
            epoch,
            campaign_id: Some(campaign.as_uuid().to_string()),
        };
        emit(
            self.output,
            &RuntimeSessionResponse::Switching {
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
            &RuntimeSessionResponse::Ready {
                request_id,
                scope: self.scope.clone(),
                foundation_digest,
                tail,
            },
        )
    }

    fn archive_event(
        &mut self,
        scope: &RuntimeSessionScope,
        event: &ArchiveDriverEvent,
    ) -> Result<(), RuntimeSessionErrorCode> {
        if *scope != self.scope {
            return Ok(());
        }
        if matches!(event, ArchiveDriverEvent::Stopped) {
            return self.fail(None, RuntimeSessionErrorCode::ArchiveRefused);
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
    ) -> Result<(), RuntimeSessionErrorCode> {
        self.retire(events, true, request_id, grace)?;
        if let Some(request_id) = request_id {
            emit(
                self.output,
                &RuntimeSessionResponse::Stopped {
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
    ) -> Result<(), RuntimeSessionErrorCode> {
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
