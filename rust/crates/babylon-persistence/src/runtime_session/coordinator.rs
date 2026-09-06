//! One owner serializes commands, durable acknowledgements, and Archive progress.

use std::io::{BufRead, Write};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crate::archive_driver::{ArchiveDriverEventV1, ArchiveDriverRequestErrorV1, ArchiveDriverV1};

use super::input::{InputEvent, SessionInput};
use super::{
    emit, RuntimeSessionErrorCodeV2, RuntimeSessionRequestV2, RuntimeSessionResponseV2,
    SessionBackend, RUNTIME_SESSION_PROTOCOL_VERSION_V2,
};

const EVENT_CAPACITY: usize = 8;
// Match the existing observer/launcher graceful process window, not a claim that
// synchronous PostgreSQL connect/auth/drop can be forcibly cancelled by a thread.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(150);
const COMPLETION_CHECK: Duration = Duration::from_millis(100);

type ArchiveEventSink = Box<dyn Fn(ArchiveDriverEventV1) -> bool + Send>;

#[derive(Debug)]
pub(super) enum SessionEvent {
    Input(InputEvent),
    Archive(ArchiveDriverEventV1),
}

pub(super) trait ArchiveControl {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCodeV2>;
    fn stop(&self);
    fn finished(&self) -> bool;
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV2>;
}

impl ArchiveControl for ArchiveDriverV1 {
    fn refresh(&self, request_id: u64) -> Result<(), RuntimeSessionErrorCodeV2> {
        self.request_refresh(request_id)
            .map_err(|error| match error {
                ArchiveDriverRequestErrorV1::Full => RuntimeSessionErrorCodeV2::StorageBusy,
                ArchiveDriverRequestErrorV1::Stopped => RuntimeSessionErrorCodeV2::ArchiveRefused,
            })
    }
    fn stop(&self) {
        self.request_stop();
    }
    fn finished(&self) -> bool {
        self.is_finished()
    }
    fn join_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV2> {
        match self.join_if_finished() {
            Some(Err(_) | Ok(Err(_))) => Err(RuntimeSessionErrorCodeV2::ArchiveRefused),
            Some(Ok(Ok(()))) | None => Ok(()),
        }
    }
}

pub(super) fn serve<I, W, B, D>(
    input: I,
    output: &mut W,
    backend: &mut B,
    campaign: &str,
    foundation_digest: String,
    start_archive: impl FnOnce(ArchiveEventSink) -> Result<D, RuntimeSessionErrorCodeV2>,
) -> Result<(), RuntimeSessionErrorCodeV2>
where
    I: BufRead + Send + 'static,
    W: Write,
    B: SessionBackend,
    D: ArchiveControl,
{
    let (sender, events) = mpsc::sync_channel(EVENT_CAPACITY);
    let archive_sender = sender.clone();
    let archive = start_archive(Box::new(move |event| {
        archive_sender
            .try_send(SessionEvent::Archive(event))
            .is_ok()
    }))?;
    let mut coordinator = Coordinator::new(output, backend, archive, campaign);
    // Worker completion may already be queued. Ready is still always first.
    coordinator.ready(foundation_digest)?;
    let mut input = SessionInput::start(input, sender)?;
    loop {
        let event = match events.recv_timeout(COMPLETION_CHECK) {
            Ok(event) => event,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                coordinator.check_active_driver()?;
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return coordinator.fail(None, RuntimeSessionErrorCodeV2::ArchiveRefused);
            }
        };
        match event {
            SessionEvent::Archive(ArchiveDriverEventV1::Stopped) => {
                return coordinator.fail(None, RuntimeSessionErrorCodeV2::ArchiveRefused);
            }
            SessionEvent::Archive(event) => coordinator.archive_event(&event)?,
            SessionEvent::Input(InputEvent::Frame(bytes)) => {
                if let Some(request_id) = coordinator.request(&bytes)? {
                    input.stop();
                    return coordinator.shutdown(&events, Some(request_id), SHUTDOWN_GRACE);
                }
                input.next()?;
            }
            SessionEvent::Input(InputEvent::Eof) => {
                input.stop();
                return coordinator.shutdown(&events, None, SHUTDOWN_GRACE);
            }
            SessionEvent::Input(InputEvent::Refused(code)) => {
                coordinator.refuse(None, code)?;
                return Err(code);
            }
        }
        // Check after traffic too: queued input must not starve detection of a
        // silently exited worker. This reads JoinHandle state, never PostgreSQL.
        coordinator.check_active_driver()?;
    }
}

struct Coordinator<'a, W: Write, B: SessionBackend, D: ArchiveControl> {
    output: &'a mut W,
    backend: &'a mut B,
    archive: D,
    campaign: &'a str,
    verified_tick: u64,
    retention_ready: bool,
}

impl<'a, W: Write, B: SessionBackend, D: ArchiveControl> Coordinator<'a, W, B, D> {
    fn new(output: &'a mut W, backend: &'a mut B, archive: D, campaign: &'a str) -> Self {
        Self {
            output,
            backend,
            archive,
            campaign,
            verified_tick: 0,
            retention_ready: false,
        }
    }

    fn ready(&mut self, foundation_digest: String) -> Result<(), RuntimeSessionErrorCodeV2> {
        emit(
            self.output,
            &RuntimeSessionResponseV2::Ready {
                protocol_version: RUNTIME_SESSION_PROTOCOL_VERSION_V2,
                campaign_id: self.campaign.to_owned(),
                foundation_digest,
                tail: self.backend.tail(),
            },
        )
    }

    fn refuse(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCodeV2,
    ) -> Result<(), RuntimeSessionErrorCodeV2> {
        emit(
            self.output,
            &RuntimeSessionResponseV2::Error {
                request_id,
                code,
                tail: self.backend.tail(),
            },
        )
    }

    fn fail(
        &mut self,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCodeV2,
    ) -> Result<(), RuntimeSessionErrorCodeV2> {
        self.refuse(request_id, code)?;
        Err(code)
    }

    fn check_active_driver(&mut self) -> Result<(), RuntimeSessionErrorCodeV2> {
        if !self.archive.finished() {
            return Ok(());
        }
        let code = self
            .archive
            .join_finished()
            .err()
            .unwrap_or(RuntimeSessionErrorCodeV2::ArchiveRefused);
        self.fail(None, code)
    }

    fn request(&mut self, bytes: &[u8]) -> Result<Option<u64>, RuntimeSessionErrorCodeV2> {
        let Ok(request) = serde_json::from_slice::<RuntimeSessionRequestV2>(bytes) else {
            self.refuse(None, RuntimeSessionErrorCodeV2::InvalidRequest)?;
            return Ok(None);
        };
        let (version, campaign, request_id) = request.header();
        let refusal = if version != RUNTIME_SESSION_PROTOCOL_VERSION_V2 {
            Some(RuntimeSessionErrorCodeV2::UnsupportedVersion)
        } else if campaign != self.campaign {
            Some(RuntimeSessionErrorCodeV2::CampaignMismatch)
        } else {
            None
        };
        if let Some(code) = refusal {
            self.refuse(Some(request_id), code)?;
            return Ok(None);
        }
        match request {
            RuntimeSessionRequestV2::Stop { request_id, .. } => return Ok(Some(request_id)),
            RuntimeSessionRequestV2::RefreshArchive { request_id, .. } => {
                if let Err(code) = self.archive.refresh(request_id) {
                    self.refuse(Some(request_id), code)?;
                }
            }
            RuntimeSessionRequestV2::Advance {
                request_id,
                expected_tail,
                ..
            } => {
                match self.backend.advance(&expected_tail) {
                    Ok(tail) => emit(
                        self.output,
                        &RuntimeSessionResponseV2::Committed {
                            request_id,
                            campaign_id: self.campaign.to_owned(),
                            tail,
                        },
                    )?,
                    Err(code) => self.refuse(Some(request_id), code)?,
                }
                // No Archive work here. The transactional hint wakes the sole
                // driver; its events cannot be dequeued before this ACK flush.
            }
        }
        Ok(None)
    }

    fn archive_event(
        &mut self,
        event: &ArchiveDriverEventV1,
    ) -> Result<(), RuntimeSessionErrorCodeV2> {
        match event {
            ArchiveDriverEventV1::Progress {
                request_id,
                durable_tick,
                verified_tick,
                retention_ready,
            } => self.progress(*request_id, *durable_tick, *verified_tick, *retention_ready),
            ArchiveDriverEventV1::Failure {
                request_id,
                retrying: false,
                ..
            } => self.refuse(*request_id, RuntimeSessionErrorCodeV2::ArchiveRefused),
            ArchiveDriverEventV1::Failure {
                request_id,
                retrying: true,
                ..
            } => {
                eprintln!(
                    "babylon-runtime: Archive connection interrupted; retrying durable catch-up"
                );
                // A failed manual attempt keeps its identity. Later automatic
                // catch-up cannot masquerade as that request's successful result.
                request_id.map_or(Ok(()), |id| {
                    self.refuse(Some(id), RuntimeSessionErrorCodeV2::ArchiveRefused)
                })
            }
            ArchiveDriverEventV1::Stopped => Ok(()),
        }
    }

    fn progress(
        &mut self,
        request_id: Option<u64>,
        durable_tick: u64,
        verified_tick: u64,
        retention_ready: bool,
    ) -> Result<(), RuntimeSessionErrorCodeV2> {
        let tail = self.backend.tail().resolve_tick;
        if durable_tick > tail {
            // The coordinator already flushed every commit it performed. A
            // future durable result means another writer advanced this session.
            return self.refuse(request_id, RuntimeSessionErrorCodeV2::StaleExpectedTail);
        }
        if durable_tick < tail {
            // Never relabel old progress with the current tail. Correlated callers
            // get an explicit refusal; obsolete automatic hints are discarded.
            return request_id.map_or(Ok(()), |id| {
                self.refuse(Some(id), RuntimeSessionErrorCodeV2::StaleExpectedTail)
            });
        }
        if verified_tick > durable_tick
            || verified_tick < self.verified_tick
            || (self.retention_ready && !retention_ready)
        {
            return self.refuse(request_id, RuntimeSessionErrorCodeV2::ArchiveRefused);
        }
        self.verified_tick = verified_tick;
        self.retention_ready = retention_ready;
        // Equal P can still represent a newly committed partial page batch.
        emit(
            self.output,
            &RuntimeSessionResponseV2::ArchiveProgress {
                request_id,
                campaign_id: self.campaign.to_owned(),
                durable_tick,
                verified_tick,
                retention_ready,
            },
        )
    }

    fn shutdown(
        &mut self,
        events: &Receiver<SessionEvent>,
        request_id: Option<u64>,
        grace: Duration,
    ) -> Result<(), RuntimeSessionErrorCodeV2> {
        self.archive.stop();
        let started = Instant::now();
        loop {
            if self.archive.finished() {
                if let Err(code) = self.archive.join_finished() {
                    return self.fail(request_id, code);
                }
                // Reports already queued by the finished driver still precede Stopped.
                while let Ok(event) = events.try_recv() {
                    if let SessionEvent::Archive(event) = event {
                        self.archive_event(&event)?;
                    }
                }
                if let Some(request_id) = request_id {
                    emit(
                        self.output,
                        &RuntimeSessionResponseV2::Stopped { request_id },
                    )?;
                }
                return Ok(());
            }
            let Some(remaining) = grace.checked_sub(started.elapsed()) else {
                // No false Stopped. The stdio process/launcher remains the final
                // boundary for a sync driver blocked in connect/auth/drop.
                return self.fail(request_id, RuntimeSessionErrorCodeV2::StorageCanceled);
            };
            match events.recv_timeout(remaining.min(COMPLETION_CHECK)) {
                Ok(SessionEvent::Archive(event)) => self.archive_event(&event)?,
                Ok(SessionEvent::Input(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Closure captures can drop before thread completion becomes
                    // observable. Keep the same grace; disconnection is not a
                    // failed transaction or permission to join an active thread.
                    std::thread::park_timeout(remaining.min(COMPLETION_CHECK));
                }
            }
        }
    }
}

impl<W: Write, B: SessionBackend, D: ArchiveControl> Drop for Coordinator<'_, W, B, D> {
    fn drop(&mut self) {
        self.archive.stop();
        if self.archive.finished() {
            let _ = self.archive.join_finished();
        }
    }
}

#[cfg(test)]
mod tests;
