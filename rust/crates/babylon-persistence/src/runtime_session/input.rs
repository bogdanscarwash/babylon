//! Owned, bounded input adapter. It has no database or simulation authority.

use std::io::{BufRead, Read as _};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};

use super::coordinator::SessionEvent;
use super::{RuntimeSessionErrorCodeV2, RUNTIME_SESSION_MAX_LINE_BYTES_V2};

#[derive(Debug)]
pub(super) enum InputEvent {
    Frame(Vec<u8>),
    Refused(RuntimeSessionErrorCodeV2),
    Eof,
}

pub(super) struct SessionInput {
    permit: Option<SyncSender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl SessionInput {
    pub(super) fn start(
        input: impl BufRead + Send + 'static,
        events: SyncSender<SessionEvent>,
    ) -> Result<Self, RuntimeSessionErrorCodeV2> {
        let (permit, next) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name("runtime-control-input".into())
            .spawn(move || pump(input, &events, &next))
            .map_err(|_| RuntimeSessionErrorCodeV2::PipeFailure)?;
        Ok(Self {
            permit: Some(permit),
            handle: Some(handle),
        })
    }

    pub(super) fn next(&self) -> Result<(), RuntimeSessionErrorCodeV2> {
        self.permit
            .as_ref()
            .ok_or(RuntimeSessionErrorCodeV2::PipeFailure)?
            .try_send(())
            .map_err(|_| RuntimeSessionErrorCodeV2::PipeFailure)
    }

    pub(super) fn stop(&mut self) {
        self.permit = None;
    }

    pub(super) fn join_if_finished(&mut self) -> Result<(), RuntimeSessionErrorCodeV2> {
        if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
            self.handle
                .take()
                .expect("finished input handle exists")
                .join()
                .map_err(|_| RuntimeSessionErrorCodeV2::PipeFailure)?;
        }
        Ok(())
    }
}

impl Drop for SessionInput {
    fn drop(&mut self) {
        // Dropping the permit releases a reader awaiting the next record. A reader
        // blocked in external input cannot be joined unconditionally; it owns no
        // authority and is ended by the stdio process's existing exit boundary.
        self.stop();
        let _ = self.join_if_finished();
    }
}

fn pump(mut input: impl BufRead, events: &SyncSender<SessionEvent>, next: &Receiver<()>) {
    loop {
        let event = read_frame(&mut input);
        let terminal = !matches!(event, InputEvent::Frame(_));
        if events.send(SessionEvent::Input(event)).is_err() || terminal || next.recv().is_err() {
            return;
        }
    }
}

fn read_frame(input: &mut impl BufRead) -> InputEvent {
    let mut line = Vec::new();
    match input
        .take((RUNTIME_SESSION_MAX_LINE_BYTES_V2 + 1) as u64)
        .read_until(b'\n', &mut line)
    {
        Ok(0) => InputEvent::Eof,
        Ok(size) if size <= RUNTIME_SESSION_MAX_LINE_BYTES_V2 && line.ends_with(b"\n") => {
            InputEvent::Frame(line)
        }
        Ok(_) => InputEvent::Refused(RuntimeSessionErrorCodeV2::InvalidRequest),
        Err(_) => InputEvent::Refused(RuntimeSessionErrorCodeV2::PipeFailure),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    #[test]
    fn record_permit_prevents_reading_past_accepted_stop() {
        let (events, received) = mpsc::sync_channel(2);
        let mut input =
            SessionInput::start(Cursor::new(b"stop\nadvance\n".to_vec()), events).unwrap();
        assert!(matches!(
            received.recv().unwrap(),
            SessionEvent::Input(InputEvent::Frame(_))
        ));
        assert!(matches!(
            received.try_recv(),
            Err(mpsc::TryRecvError::Empty)
        ));
        input.stop();
        assert!(matches!(
            received.recv_timeout(Duration::from_secs(1)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    #[test]
    fn eof_and_oversized_frames_are_terminal_without_a_permit() {
        for (bytes, expected) in [
            (Vec::new(), None),
            (
                vec![b' '; RUNTIME_SESSION_MAX_LINE_BYTES_V2 + 1],
                Some(RuntimeSessionErrorCodeV2::InvalidRequest),
            ),
        ] {
            let (events, received) = mpsc::sync_channel(2);
            let _input = SessionInput::start(Cursor::new(bytes), events).unwrap();
            match (received.recv().unwrap(), expected) {
                (SessionEvent::Input(InputEvent::Eof), None) => {}
                (SessionEvent::Input(InputEvent::Refused(actual)), Some(expected)) => {
                    assert_eq!(actual, expected);
                }
                other => panic!("unexpected framed input: {other:?}"),
            }
            assert!(matches!(
                received.recv_timeout(Duration::from_secs(1)),
                Err(mpsc::RecvTimeoutError::Disconnected)
            ));
        }
    }
}
