use super::*;
use crate::{ArchiveWorker, NullArchiveDossierProducer};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn progress(request_id: Option<u64>, durable_tick: u64) -> ArchiveDriverEvent {
    ArchiveDriverEvent::Progress {
        request_id,
        durable_tick,
        verified_tick: durable_tick,
    }
}

#[test]
fn backpressure_preserves_manual_correlation_and_latest_automatic_scope() {
    let mut outbox = run::Outbox::default();
    outbox.push(progress(Some(41), 2));
    outbox.push(progress(None, 2));
    outbox.flush(&|_| false);
    outbox.push(progress(None, 3));
    outbox.push(progress(None, 4));
    let received = RefCell::new(Vec::new());
    outbox.flush(&|event| {
        received.borrow_mut().push(event);
        true
    });
    assert_eq!(
        *received.borrow(),
        [progress(Some(41), 2), progress(None, 4)]
    );
    outbox.flush(&|_| panic!("delivered events cannot repeat"));
}

#[test]
fn recovered_automatic_progress_replaces_unsent_transient_error_not_manual_reply() {
    let mut outbox = run::Outbox::default();
    let failure = ArchiveDriverEvent::Failure {
        request_id: Some(u64::MAX),
        failure: ArchiveDriverFailure::Disconnected,
        retrying: true,
    };
    outbox.push(failure.clone());
    outbox.push(ArchiveDriverEvent::Failure {
        request_id: None,
        failure: ArchiveDriverFailure::Disconnected,
        retrying: true,
    });
    outbox.push(progress(None, 9));
    let received = RefCell::new(Vec::new());
    outbox.flush(&|event| {
        received.borrow_mut().push(event);
        true
    });
    assert_eq!(*received.borrow(), [failure, progress(None, 9)]);
}

#[test]
fn stop_refuses_before_connecting_or_minting_a_receipt() {
    let cancellation = ArchiveWorkerCancellation::default();
    cancellation.request_stop();
    let config = "host=192.0.2.1 port=1 dbname=not_a_live_target"
        .parse()
        .expect("config syntax");
    let result = ArchiveWorker::new(&config).sweep_cancellable(
        CampaignId::from_uuid(uuid::Uuid::nil()),
        &NullArchiveDossierProducer,
        &cancellation,
    );
    assert_eq!(result, Err(SemanticArchiveError::WorkerCanceled));
}

#[test]
fn full_commands_do_not_block_stop_or_try_to_join_active_worker() {
    let (sender, _receiver) = mpsc::sync_channel(1);
    let release = Arc::new(AtomicBool::new(false));
    let worker_release = release.clone();
    let handle = thread::spawn(move || {
        while !worker_release.load(Ordering::Acquire) {
            thread::park_timeout(Duration::from_millis(1));
        }
        Ok(())
    });
    let mut driver = ArchiveDriver {
        requests: sender,
        cancellation: ArchiveWorkerCancellation::default(),
        thread: Some(handle),
    };
    assert_eq!(driver.request_refresh(1), Ok(()));
    assert_eq!(
        driver.request_refresh(2),
        Err(ArchiveDriverRequestError::Full)
    );
    driver.request_stop();
    assert_eq!(
        driver.request_refresh(3),
        Err(ArchiveDriverRequestError::Stopped)
    );
    assert!(driver.join_if_finished().is_none());
    release.store(true, Ordering::Release);
    let deadline = Instant::now() + Duration::from_secs(2);
    while !driver.is_finished() && Instant::now() < deadline {
        thread::yield_now();
    }
    assert!(driver
        .join_if_finished()
        .expect("completed worker")
        .expect("worker did not panic")
        .is_ok());
    assert!(driver.is_finished());
}

#[test]
fn integrity_refusals_are_not_automatic_transport_retries() {
    for error in [
        SemanticArchiveError::SchemaMismatch,
        SemanticArchiveError::StoredPageMismatch,
        SemanticArchiveError::ReceiptConflict,
        SemanticArchiveError::GrantConflict,
        SemanticArchiveError::ArtifactDigest,
    ] {
        assert_eq!(
            run::classify(error.clone()),
            ArchiveDriverFailure::Refused(error)
        );
    }
}

#[test]
fn fatal_refusal_survives_later_progress_until_delivery() {
    let mut outbox = run::Outbox::default();
    let refusal = ArchiveDriverEvent::Failure {
        request_id: None,
        failure: ArchiveDriverFailure::Refused(SemanticArchiveError::StoredPageMismatch),
        retrying: false,
    };
    outbox.push(refusal.clone());
    outbox.flush(&|_| false);
    outbox.push(progress(None, 9));
    assert!(
        outbox.has_refusal(),
        "later success cannot hide an undelivered integrity failure"
    );
    let received = RefCell::new(Vec::new());
    outbox.flush(&|event| {
        received.borrow_mut().push(event);
        true
    });
    assert_eq!(*received.borrow(), [refusal, progress(None, 9)]);
    assert!(
        !outbox.has_refusal(),
        "delivery releases the explicit recovery path"
    );
}
