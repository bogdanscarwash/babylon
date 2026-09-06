use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use postgres::{fallible_iterator::FallibleIterator as _, Client, Config, NoTls};

use super::{ArchiveDriverEventV1, ArchiveDriverFailureV1};
use crate::archive::database;
use crate::{
    ArchiveWorkerCancellationV1, ArchiveWorkerV1, CampaignId, CompositeArchiveDossierProducerV1,
    CountyDossierProducerV1, PlaceDossierProducerV1, PostgresFailureClassV1,
    SemanticArchiveErrorV1, ARCHIVE_WAKEUP_CHANNEL_V1,
};

const WAIT: Duration = Duration::from_millis(125);
const MAX_RETRY: Duration = Duration::from_secs(2);
const MAX_HINTS: usize = 256;

pub(super) fn bounded_config(config: &Config) -> Config {
    let mut bounded = config.clone();
    bounded.connect_timeout(Duration::from_secs(5))
        .tcp_user_timeout(Duration::from_secs(5))
        .options("-c statement_timeout=10000ms -c lock_timeout=2000ms -c idle_in_transaction_session_timeout=30000ms");
    bounded
}

pub(super) fn classify(error: SemanticArchiveErrorV1) -> ArchiveDriverFailureV1 {
    let transient = match &error {
        SemanticArchiveErrorV1::Database { diagnostic, .. } => {
            matches!(
                diagnostic.classification(),
                PostgresFailureClassV1::Reachability | PostgresFailureClassV1::Timeout
            ) || diagnostic.sqlstate().is_some_and(|code| {
                code.starts_with("08")
                    || matches!(
                        code,
                        "40001" | "40P01" | "55P03" | "57014" | "57P01" | "57P02" | "57P03"
                    )
            })
        }
        _ => false,
    };
    if transient {
        ArchiveDriverFailureV1::Transient(error)
    } else {
        ArchiveDriverFailureV1::Refused(error)
    }
}

fn listener(config: &Config) -> Result<Client, ArchiveDriverFailureV1> {
    let mut listener_config = config.clone();
    listener_config.application_name("babylon-archive-listener-v1");
    let mut client = listener_config
        .connect(NoTls)
        .map_err(|error| classify(database("connect Archive listener", &error)))?;
    crate::archive_wakeup::validate(&mut client).map_err(classify)?;
    // Autocommit completes registration before the worker's fresh catch-up reads.
    client
        .batch_execute("LISTEN babylon_archive_wakeup_v1")
        .map_err(|error| classify(database("register Archive listener", &error)))?;
    Ok(client)
}

fn notification(client: &mut Client) -> Result<bool, ArchiveDriverFailureV1> {
    // Recreate this iterator: postgres 0.19.14 does not reset after timeout None.
    let first = client
        .notifications()
        .timeout_iter(WAIT)
        .next()
        .map_err(|error| classify(database("wait for Archive wakeup", &error)))?;
    let mut wake = first.as_ref().is_some_and(is_hint);
    if first.is_none() && client.is_closed() {
        return Err(ArchiveDriverFailureV1::Disconnected);
    }
    if first.is_some() {
        let mut notifications = client.notifications();
        let mut buffered = notifications.iter();
        for _ in 1..MAX_HINTS {
            match buffered
                .next()
                .map_err(|error| classify(database("coalesce Archive wakeups", &error)))?
            {
                Some(value) => wake |= is_hint(&value),
                None => break,
            }
        }
    }
    Ok(wake)
}

fn is_hint(value: &postgres::Notification) -> bool {
    value.channel() == ARCHIVE_WAKEUP_CHANNEL_V1 && value.payload().is_empty()
}

/// At most one correlated result and one automatic result can wait here.
#[derive(Default)]
pub(super) struct Outbox(VecDeque<ArchiveDriverEventV1>);
impl Outbox {
    pub(super) fn push(&mut self, event: ArchiveDriverEventV1) {
        if request_id(&event).is_none() {
            self.0
                .retain(|old| request_id(old).is_some() || is_refusal(old));
        }
        self.0.push_back(event);
    }
    pub(super) fn flush(&mut self, sink: &impl Fn(ArchiveDriverEventV1) -> bool) {
        while let Some(front) = self.0.front() {
            if !sink(front.clone()) {
                break;
            }
            self.0.pop_front();
        }
    }
    pub(super) fn has_refusal(&self) -> bool {
        self.0.iter().any(is_refusal)
    }
    fn has_request(&self) -> bool {
        self.0.iter().any(|event| request_id(event).is_some())
    }
}
fn is_refusal(event: &ArchiveDriverEventV1) -> bool {
    matches!(
        event,
        ArchiveDriverEventV1::Failure {
            retrying: false,
            ..
        }
    )
}
fn request_id(event: &ArchiveDriverEventV1) -> Option<u64> {
    match event {
        ArchiveDriverEventV1::Progress { request_id, .. }
        | ArchiveDriverEventV1::Failure { request_id, .. } => *request_id,
        ArchiveDriverEventV1::Stopped => None,
    }
}

struct DriverState {
    listener: Option<Client>,
    dirty: bool,
    retry_at: Option<Instant>,
    backoff: Duration,
    request: Option<u64>,
    outbox: Outbox,
    last_refusal: Option<ArchiveDriverFailureV1>,
}
impl DriverState {
    fn new() -> Self {
        Self {
            listener: None,
            dirty: true,
            retry_at: None,
            backoff: WAIT,
            request: None,
            outbox: Outbox::default(),
            last_refusal: None,
        }
    }
    fn failure(&mut self, failure: ArchiveDriverFailureV1) {
        let retrying = !matches!(failure, ArchiveDriverFailureV1::Refused(_));
        if !retrying {
            self.last_refusal = Some(failure.clone());
        }
        self.outbox.push(ArchiveDriverEventV1::Failure {
            request_id: self.request.take(),
            failure,
            retrying,
        });
        self.dirty = false;
        self.retry_at = retrying.then(|| Instant::now() + self.backoff);
        self.backoff = self.backoff.saturating_mul(2).min(MAX_RETRY);
        self.listener = None;
    }
    fn finish(
        mut self,
        sink: &impl Fn(ArchiveDriverEventV1) -> bool,
    ) -> Result<(), ArchiveDriverFailureV1> {
        // Best effort only; the join result retains a fatal refusal independently
        // of both saturated and accepted-but-not-yet-observed queue entries.
        self.outbox.flush(sink);
        let _ = sink(ArchiveDriverEventV1::Stopped);
        self.last_refusal.take().map_or(Ok(()), Err)
    }
    fn request(&mut self, requests: &Receiver<u64>) -> bool {
        if self.request.is_some() || self.outbox.has_request() {
            return true;
        }
        match requests.try_recv() {
            Ok(id) => {
                self.request = Some(id);
                self.dirty = true;
                self.retry_at = None;
                true
            }
            Err(TryRecvError::Empty) => true,
            Err(TryRecvError::Disconnected) => false,
        }
    }
    fn ready(&mut self) -> bool {
        if self.retry_at.is_some_and(|time| time <= Instant::now()) {
            self.retry_at = None;
            self.dirty = true;
        }
        self.dirty
    }
}

pub(super) fn run(
    config: &Config,
    campaign: CampaignId,
    requests: &Receiver<u64>,
    cancellation: &ArchiveWorkerCancellationV1,
    sink: &impl Fn(ArchiveDriverEventV1) -> bool,
) -> Result<(), ArchiveDriverFailureV1> {
    let mut state = DriverState::new();
    let mut worker = ArchiveWorkerV1::new(config);
    while !cancellation.is_stopped() {
        state.outbox.flush(sink);
        // A fatal refusal must reach the coordinator before later recovery can
        // supersede it. Stop remains independent of this delivery backpressure.
        if state.outbox.has_refusal() {
            std::thread::park_timeout(WAIT);
            continue;
        }
        if !state.request(requests) {
            break;
        }
        if state.ready() {
            maintain(&mut state, config, campaign, &mut worker, cancellation);
        } else if let Some(client) = &mut state.listener {
            match notification(client) {
                Ok(wake) => state.dirty = wake,
                Err(failure) => state.failure(failure),
            }
        } else {
            std::thread::park_timeout(WAIT);
        }
    }
    // Completion remains unobservable until any synchronous Client teardown has
    // actually finished; the coordinator never joins an unfinished thread.
    state.finish(sink)
}

fn maintain(
    state: &mut DriverState,
    config: &Config,
    campaign: CampaignId,
    worker: &mut ArchiveWorkerV1,
    cancellation: &ArchiveWorkerCancellationV1,
) {
    if state.listener.is_none() {
        match listener(config) {
            Ok(client) => state.listener = Some(client),
            Err(failure) => {
                state.failure(failure);
                return;
            }
        }
    }
    let result = producer(config)
        .and_then(|producer| worker.sweep_cancellable(campaign, &producer, cancellation));
    match result {
        Ok(report) => {
            state.outbox.push(ArchiveDriverEventV1::Progress {
                request_id: state.request.take(),
                durable_tick: report.durable_tick(),
                verified_tick: report.verified_tick(),
                retention_ready: report.retention_ready(),
            });
            state.last_refusal = None;
            state.dirty = report.has_pending_work();
            state.retry_at = None;
            state.backoff = WAIT;
        }
        Err(SemanticArchiveErrorV1::WorkerCanceled) => state.dirty = false,
        Err(error) => state.failure(classify(error)),
    }
}

fn producer(config: &Config) -> Result<CompositeArchiveDossierProducerV1, SemanticArchiveErrorV1> {
    Ok(CompositeArchiveDossierProducerV1::new(vec![
        Box::new(CountyDossierProducerV1::try_new(config)?),
        Box::new(PlaceDossierProducerV1::try_new(config)?),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn stopped_completion_preserves_refusal_when_event_sink_is_full() {
        let refusal = ArchiveDriverFailureV1::Refused(SemanticArchiveErrorV1::StoredPageMismatch);
        let mut state = DriverState::new();
        state.failure(refusal.clone());
        state.outbox.flush(&|_| false);
        assert!(state.outbox.has_refusal());
        assert_eq!(state.finish(&|_| false), Err(refusal));
    }

    #[test]
    fn queued_but_unobserved_refusal_remains_in_completion_result() {
        let refusal = ArchiveDriverFailureV1::Refused(SemanticArchiveErrorV1::ReceiptConflict);
        let mut state = DriverState::new();
        state.failure(refusal.clone());
        let queue = RefCell::new(Vec::new());
        state.outbox.flush(&|event| {
            queue.borrow_mut().push(event);
            true
        });
        assert!(
            !state.outbox.has_refusal(),
            "sink accepted the event, coordinator may still be busy"
        );
        assert_eq!(state.finish(&|_| true), Err(refusal));
        assert!(matches!(
            queue.borrow().as_slice(),
            [ArchiveDriverEventV1::Failure {
                retrying: false,
                ..
            }]
        ));
    }
}
