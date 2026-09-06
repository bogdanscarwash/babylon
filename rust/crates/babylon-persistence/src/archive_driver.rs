//! Archive-only notification listener and canonical ordered worker driver.
//!
//! Notifications are empty, lossy wake hints. Startup and reconnect first commit
//! LISTEN and then reconcile durable state. No SQL runs merely because an idle
//! notification wait timed out. The owner must never join an unfinished driver:
//! synchronous authentication and socket teardown are not a hard deadline API.

mod run;
#[cfg(test)]
mod tests;

use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};

use postgres::Config;

use crate::{ArchiveWorkerCancellationV1, CampaignId, SemanticArchiveErrorV1};

const COMMAND_CAPACITY: usize = 8;

/// One coherent maintenance result or explicit failure from the dedicated driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchiveDriverEventV1 {
    /// Marker tail, contiguous processed prefix and validated seal share a snapshot.
    Progress {
        /// Present only for the explicit refresh that requested this observation.
        request_id: Option<u64>,
        /// Durable marker tail in the driver campaign.
        durable_tick: u64,
        /// Contiguous completed Archive prefix, never above the durable tail.
        verified_tick: u64,
        /// Whether exact adoption validation has completed.
        retention_ready: bool,
    },
    /// A typed failure; integrity errors never become successful progress.
    Failure {
        /// Explicit refresh correlation when applicable.
        request_id: Option<u64>,
        /// Classified transport or canonical worker refusal.
        failure: ArchiveDriverFailureV1,
        /// Whether a bounded retry is scheduled without requiring another hint.
        retrying: bool,
    },
    /// Worker has stopped initiating work. Thread completion remains authoritative.
    Stopped,
}

/// Closed distinction between retryable transport failure and integrity refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchiveDriverFailureV1 {
    /// The dedicated listening connection closed without a server error.
    Disconnected,
    /// A specifically admitted transient database error.
    Transient(SemanticArchiveErrorV1),
    /// Authentication, source, schema or publication refusal; no automatic retry.
    Refused(SemanticArchiveErrorV1),
}

/// Thread creation and untrusted target admission failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDriverStartErrorV1 {
    /// The connection target violates the existing local target boundary.
    InvalidTarget,
    /// The host refused creation of the one dedicated driver thread.
    Spawn,
}

/// A refresh request was not accepted; no success response will be fabricated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDriverRequestErrorV1 {
    /// All bounded request slots are occupied.
    Full,
    /// Stop was requested or the worker has already exited.
    Stopped,
}

/// One owned listener/worker thread and its nonblocking control boundary.
pub struct ArchiveDriverV1 {
    requests: SyncSender<u64>,
    cancellation: ArchiveWorkerCancellationV1,
    thread: Option<JoinHandle<Result<(), ArchiveDriverFailureV1>>>,
}

impl ArchiveDriverV1 {
    /// Start one fixed-campaign driver. The sink returns false on backpressure.
    ///
    /// The driver retains unsent correlated results and coalesces automatic
    /// progress; the sink must be nonblocking and must not perform database work.
    ///
    /// # Errors
    /// Refuses an unsafe target or inability to create the dedicated thread.
    pub fn start(
        config: &Config,
        campaign: CampaignId,
        sink: impl Fn(ArchiveDriverEventV1) -> bool + Send + 'static,
    ) -> Result<Self, ArchiveDriverStartErrorV1> {
        crate::validate_legacy_connection_target(config)
            .map_err(|_| ArchiveDriverStartErrorV1::InvalidTarget)?;
        let config = run::bounded_config(config);
        let cancellation = ArchiveWorkerCancellationV1::default();
        let worker_stop = cancellation.clone();
        let (requests, receiver) = mpsc::sync_channel(COMMAND_CAPACITY);
        let thread = thread::Builder::new()
            .name("babylon-archive".into())
            .spawn(move || run::run(&config, campaign, &receiver, &worker_stop, &sink))
            .map_err(|_| ArchiveDriverStartErrorV1::Spawn)?;
        Ok(Self {
            requests,
            cancellation,
            thread: Some(thread),
        })
    }

    /// Request reconciliation without blocking the runtime coordinator.
    ///
    /// # Errors
    /// Returns `Full` or `Stopped` rather than dropping request correlation.
    pub fn request_refresh(&self, request_id: u64) -> Result<(), ArchiveDriverRequestErrorV1> {
        if self.cancellation.is_stopped() || self.is_finished() {
            return Err(ArchiveDriverRequestErrorV1::Stopped);
        }
        self.requests
            .try_send(request_id)
            .map_err(|error| match error {
                TrySendError::Full(_) => ArchiveDriverRequestErrorV1::Full,
                TrySendError::Disconnected(_) => ArchiveDriverRequestErrorV1::Stopped,
            })
    }

    /// Request cooperative shutdown even when every message queue is full.
    pub fn request_stop(&self) {
        self.cancellation.request_stop();
    }

    /// Observe actual thread completion, independent of event queue delivery.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Reap only an already-finished thread; this never waits for active SQL.
    /// The inner result preserves any unrecovered fatal Archive refusal even
    /// when the event sink could not deliver it before stop.
    pub fn join_if_finished(
        &mut self,
    ) -> Option<thread::Result<Result<(), ArchiveDriverFailureV1>>> {
        if !self.is_finished() {
            return None;
        }
        self.thread.take().map(JoinHandle::join)
    }
}

impl Drop for ArchiveDriverV1 {
    fn drop(&mut self) {
        self.request_stop();
    }
}
