//! Bounded lifecycle and empty-action four-week control over parent-owned pipes.

use postgres::Config;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

mod backend;
mod coordinator;
mod input;
mod protocol;
pub use protocol::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSessionErrorCode {
    InvalidRequest,
    UnsupportedVersion,
    SessionMismatch,
    StaleExpectedTail,
    CampaignAbsent,
    CampaignAlreadyExists,
    CommitRefused,
    ArchiveRefused,
    StorageRefused,
    StorageBusy,
    StorageCanceled,
    ScenarioMismatch,
    DefinesMissing,
    DefinesTooLarge,
    DefinesMalformed,
    DefinesInvalid,
    PipeFailure,
    HorizonComplete,
}
impl std::fmt::Display for RuntimeSessionErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "runtime session refused: {self:?}")
    }
}
impl std::error::Error for RuntimeSessionErrorCode {}

trait SessionBackend {
    fn tail(&self) -> RuntimeSessionTail;
    fn advance(
        &mut self,
        expected: &RuntimeSessionTail,
    ) -> Result<RuntimeSessionTail, RuntimeSessionErrorCode>;
}

fn emit(
    output: &mut impl Write,
    response: &RuntimeSessionResponse,
) -> Result<(), RuntimeSessionErrorCode> {
    let mut bytes =
        serde_json::to_vec(response).map_err(|_| RuntimeSessionErrorCode::PipeFailure)?;
    if bytes.len() >= RUNTIME_SESSION_MAX_LINE_BYTES {
        return Err(RuntimeSessionErrorCode::PipeFailure);
    }
    bytes.push(b'\n');
    output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .map_err(|_| RuntimeSessionErrorCode::PipeFailure)
}

/// Run one lifecycle service; no campaign is admitted before the first Switch.
/// # Errors
/// Refuses broken framing/pipes and fatal worker teardown. Target admission
/// failures are scoped protocol responses and permit another explicit Switch.
pub fn run_runtime_session(
    config: &Config,
    defines_path: &std::path::Path,
    input: impl BufRead + Send + 'static,
    output: &mut impl Write,
) -> Result<(), RuntimeSessionErrorCode> {
    coordinator::serve(
        input,
        output,
        |target| backend::open(config, target, defines_path),
        |campaign, events| {
            crate::archive_driver::ArchiveDriver::start(config, campaign, events)
                .map_err(|_| RuntimeSessionErrorCode::ArchiveRefused)
        },
    )
}

/// Bind the lifecycle service to the inherited standard streams.
/// # Errors
/// See [`run_runtime_session`].
pub fn run_runtime_session_stdio(
    config: &Config,
    defines_path: &std::path::Path,
) -> Result<(), RuntimeSessionErrorCode> {
    run_runtime_session(
        config,
        defines_path,
        std::io::BufReader::new(std::io::stdin()),
        &mut std::io::stdout().lock(),
    )
}
