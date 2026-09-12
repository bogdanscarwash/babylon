//! One admitted backend and exactly one Archive driver, retired together.

use super::{ArchiveControl, SessionEvent, COMPLETION_CHECK};
use crate::archive_driver::ArchiveDriverEvent;
use crate::runtime_session::{
    emit, RuntimeSessionErrorCode, RuntimeSessionResponse, RuntimeSessionScope, SessionBackend,
};
use std::io::Write;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub(super) struct Active<B: SessionBackend, D: ArchiveControl> {
    pub(super) backend: B,
    pub(super) archive: D,
    verified_tick: u64,
    joined: bool,
}
impl<B: SessionBackend, D: ArchiveControl> Active<B, D> {
    pub(super) const fn new(backend: B, archive: D) -> Self {
        Self {
            backend,
            archive,
            verified_tick: 0,
            joined: false,
        }
    }
    fn refuse(
        &self,
        output: &mut impl Write,
        scope: &RuntimeSessionScope,
        request_id: Option<u64>,
        code: RuntimeSessionErrorCode,
    ) -> Result<(), RuntimeSessionErrorCode> {
        emit(
            output,
            &RuntimeSessionResponse::Error {
                request_id,
                scope: scope.clone(),
                code,
                tail: Some(self.backend.tail()),
            },
        )
    }
    pub(super) fn event(
        &mut self,
        output: &mut impl Write,
        scope: &RuntimeSessionScope,
        event: &ArchiveDriverEvent,
    ) -> Result<(), RuntimeSessionErrorCode> {
        match event {
            ArchiveDriverEvent::Progress {
                request_id,
                durable_tick,
                verified_tick,
            } => self.progress(output, scope, *request_id, *durable_tick, *verified_tick),
            ArchiveDriverEvent::Failure {
                request_id,
                retrying: false,
                ..
            } => self.refuse(
                output,
                scope,
                *request_id,
                RuntimeSessionErrorCode::ArchiveRefused,
            ),
            ArchiveDriverEvent::Failure {
                request_id,
                retrying: true,
                ..
            } => {
                eprintln!(
                    "babylon-runtime: Archive connection interrupted; retrying durable catch-up"
                );
                request_id.map_or(Ok(()), |id| {
                    self.refuse(
                        output,
                        scope,
                        Some(id),
                        RuntimeSessionErrorCode::ArchiveRefused,
                    )
                })
            }
            ArchiveDriverEvent::Stopped => Ok(()),
        }
    }
    fn progress(
        &mut self,
        output: &mut impl Write,
        scope: &RuntimeSessionScope,
        request_id: Option<u64>,
        durable_tick: u64,
        verified_tick: u64,
    ) -> Result<(), RuntimeSessionErrorCode> {
        let tail = self.backend.tail().resolve_tick;
        if durable_tick > tail {
            return self.refuse(
                output,
                scope,
                request_id,
                RuntimeSessionErrorCode::StaleExpectedTail,
            );
        }
        if durable_tick < tail {
            return request_id.map_or(Ok(()), |id| {
                self.refuse(
                    output,
                    scope,
                    Some(id),
                    RuntimeSessionErrorCode::StaleExpectedTail,
                )
            });
        }
        if verified_tick > durable_tick || verified_tick < self.verified_tick {
            return self.refuse(
                output,
                scope,
                request_id,
                RuntimeSessionErrorCode::ArchiveRefused,
            );
        }
        self.verified_tick = verified_tick;
        emit(
            output,
            &RuntimeSessionResponse::ArchiveProgress {
                request_id,
                scope: scope.clone(),
                durable_tick,
                verified_tick,
            },
        )
    }
    pub(super) fn retire(
        &mut self,
        output: &mut impl Write,
        scope: &RuntimeSessionScope,
        events: &Receiver<SessionEvent>,
        expose_progress: bool,
        grace: Duration,
    ) -> Result<(), RuntimeSessionErrorCode> {
        self.archive.stop();
        let started = Instant::now();
        loop {
            if self.archive.finished() {
                let result = self.archive.join_finished();
                self.joined = true;
                result?;
                while let Ok(event) = events.try_recv() {
                    self.retiring_event(output, scope, event, expose_progress)?;
                }
                return Ok(());
            }
            let Some(remaining) = grace.checked_sub(started.elapsed()) else {
                return Err(RuntimeSessionErrorCode::StorageCanceled);
            };
            match events.recv_timeout(remaining.min(COMPLETION_CHECK)) {
                Ok(event) => self.retiring_event(output, scope, event, expose_progress)?,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    std::thread::park_timeout(remaining.min(COMPLETION_CHECK));
                }
            }
        }
    }
    fn retiring_event(
        &mut self,
        output: &mut impl Write,
        scope: &RuntimeSessionScope,
        event: SessionEvent,
        expose_progress: bool,
    ) -> Result<(), RuntimeSessionErrorCode> {
        if let SessionEvent::Archive {
            scope: emitted,
            event,
        } = event
        {
            if expose_progress && emitted == *scope {
                self.event(output, scope, &event)?;
            }
        }
        Ok(())
    }
}
impl<B: SessionBackend, D: ArchiveControl> Drop for Active<B, D> {
    fn drop(&mut self) {
        self.archive.stop();
        if !self.joined && self.archive.finished() {
            let _ = self.archive.join_finished();
        }
    }
}
