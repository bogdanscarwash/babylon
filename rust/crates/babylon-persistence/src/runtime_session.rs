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
pub enum RuntimeSessionErrorCodeV3 {
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
impl std::fmt::Display for RuntimeSessionErrorCodeV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "runtime session refused: {self:?}")
    }
}
impl std::error::Error for RuntimeSessionErrorCodeV3 {}

trait SessionBackend {
    fn tail(&self) -> RuntimeSessionTailV3;
    fn advance(
        &mut self,
        expected: &RuntimeSessionTailV3,
    ) -> Result<RuntimeSessionTailV3, RuntimeSessionErrorCodeV3>;
}

fn emit(
    output: &mut impl Write,
    response: &RuntimeSessionResponseV3,
) -> Result<(), RuntimeSessionErrorCodeV3> {
    let mut bytes =
        serde_json::to_vec(response).map_err(|_| RuntimeSessionErrorCodeV3::PipeFailure)?;
    if bytes.len() >= RUNTIME_SESSION_MAX_LINE_BYTES_V3 {
        return Err(RuntimeSessionErrorCodeV3::PipeFailure);
    }
    bytes.push(b'\n');
    output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .map_err(|_| RuntimeSessionErrorCodeV3::PipeFailure)
}

/// Run one lifecycle service; no campaign is admitted before the first Switch.
/// # Errors
/// Refuses broken framing/pipes and fatal worker teardown. Target admission
/// failures are scoped protocol responses and permit another explicit Switch.
pub fn run_runtime_session_v3(
    config: &Config,
    defines_path: &std::path::Path,
    input: impl BufRead + Send + 'static,
    output: &mut impl Write,
) -> Result<(), RuntimeSessionErrorCodeV3> {
    coordinator::serve(
        input,
        output,
        |target| backend::open(config, target, defines_path),
        |campaign, events| {
            crate::archive_driver::ArchiveDriverV1::start(config, campaign, events)
                .map_err(|_| RuntimeSessionErrorCodeV3::ArchiveRefused)
        },
    )
}

/// Bind the lifecycle service to the inherited standard streams.
/// # Errors
/// See [`run_runtime_session_v3`].
pub fn run_runtime_session_stdio_v3(
    config: &Config,
    defines_path: &std::path::Path,
) -> Result<(), RuntimeSessionErrorCodeV3> {
    run_runtime_session_v3(
        config,
        defines_path,
        std::io::BufReader::new(std::io::stdin()),
        &mut std::io::stdout().lock(),
    )
}
