//! Bounded, task-owned experiment artifacts and limited execution provenance.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_BYTES: usize = 4 * 1024 * 1024;
const FAILURE_RESERVE: usize = 4096;
const MAX_PERIODS: usize = 64;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Contract(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "artifact I/O refused: {error}"),
            Self::Json(error) => write!(f, "artifact JSON refused: {error}"),
            Self::Contract(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub fn contract(message: impl Into<String>) -> Error {
    Error::Contract(message.into())
}

struct BoundedBytes {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "experiment artifact byte limit exceeded",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(value: &impl Serialize, limit: usize) -> Result<Vec<u8>> {
    let mut output = BoundedBytes {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut output, value)?;
    output.write_all(b"\n")?;
    Ok(output.bytes)
}
fn digest_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(super::run::hex(&digest.finalize()))
}

pub struct Output {
    path: PathBuf,
    written: usize,
    periods: usize,
    files: BTreeSet<String>,
}
impl Output {
    pub fn create(path: &Path) -> Result<Self> {
        fs::create_dir(path)?;
        Ok(Self {
            path: path.to_owned(),
            written: 0,
            periods: 0,
            files: BTreeSet::new(),
        })
    }
    fn remaining(&self) -> usize {
        MAX_BYTES - FAILURE_RESERVE - self.written
    }
    pub fn write_json(&mut self, name: &str, value: &impl Serialize) -> Result<()> {
        if Path::new(name).extension() != Some(std::ffi::OsStr::new("json"))
            || name == "failure.json"
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            || name.contains("..")
        {
            return Err(contract("invalid experiment artifact name"));
        }
        let bytes = encode(value, self.remaining())?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.path.join(name))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        self.written += bytes.len();
        self.files.insert(name.to_owned());
        Ok(())
    }
    pub fn append_period(&mut self, value: &impl Serialize) -> Result<()> {
        if self.periods >= MAX_PERIODS {
            return Err(contract("experiment exceeds 64 period records"));
        }
        let bytes = encode(value, self.remaining())?;
        let mut options = OpenOptions::new();
        options.append(true);
        if self.periods == 0 {
            options.create_new(true);
        }
        let mut file = options.open(self.path.join("periods.jsonl"))?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        self.written += bytes.len();
        self.periods += 1;
        self.files.insert("periods.jsonl".to_owned());
        Ok(())
    }
    pub fn checksums(&self) -> Result<BTreeMap<String, String>> {
        self.files
            .iter()
            .filter(|name| name.as_str() != "manifest.json")
            .map(|name| Ok((name.clone(), digest_file(&self.path.join(name))?)))
            .collect()
    }
}

#[derive(Clone, Default, Serialize)]
pub struct Progress {
    pub case: Option<String>,
    pub last_completed_period: u64,
}
pub fn write_failure(path: &Path, progress: &Progress, message: &str) -> Result<()> {
    let progress = Progress {
        case: progress
            .case
            .as_ref()
            .map(|name| name.chars().take(64).collect()),
        last_completed_period: progress.last_completed_period,
    };
    let bytes = encode(
        &serde_json::json!({
            "status": "failed", "progress": progress,
            "error": message.chars().take(256).collect::<String>(),
        }),
        FAILURE_RESERVE,
    )?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path.join("failure.json"))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn timed_out(receiver: &mpsc::Receiver<()>, duration: Duration) -> bool {
    matches!(
        receiver.recv_timeout(duration),
        Err(mpsc::RecvTimeoutError::Timeout)
    )
}
pub struct Watchdog {
    cancel: mpsc::Sender<()>,
    worker: thread::JoinHandle<()>,
}
impl Watchdog {
    pub fn start(path: &Path, progress: Arc<Mutex<Progress>>) -> Self {
        let path = path.to_owned();
        let (cancel, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            if timed_out(&receiver, Duration::from_secs(60)) {
                let progress = match progress.try_lock() {
                    Ok(progress) => progress.clone(),
                    Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner().clone(),
                    Err(std::sync::TryLockError::WouldBlock) => Progress::default(),
                };
                if let Err(error) = write_failure(
                    &path,
                    &progress,
                    "experiment exceeded 60-second invocation limit",
                ) {
                    eprintln!("experiment timeout; failure record refused: {error}");
                }
                std::process::exit(124);
            }
        });
        Self { cancel, worker }
    }
    pub fn finish(self) -> Result<()> {
        let _ = self.cancel.send(());
        self.worker
            .join()
            .map_err(|_| contract("experiment watchdog panicked"))
    }
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(repo).args(args).output()?;
    if !output.status.success() || output.stdout.len() > 65_536 {
        return Err(contract("experiment Git provenance refused"));
    }
    String::from_utf8(output.stdout).map_err(|_| contract("Git provenance was not UTF-8"))
}
pub fn provenance() -> Result<serde_json::Value> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .ok_or_else(|| contract("experiment repository path unavailable"))?;
    if !git(
        repo,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )?
    .is_empty()
    {
        return Err(contract(
            "measured experiment requires a clean tracked and untracked source tree",
        ));
    }
    let sources: [(&str, &[u8]); 5] = [
        (
            "rust/crates/babylon-persistence/examples/michigan_experiment.rs",
            include_bytes!("../michigan_experiment.rs"),
        ),
        (
            "rust/crates/babylon-persistence/examples/michigan_experiment/run.rs",
            include_bytes!("run.rs"),
        ),
        (
            "rust/crates/babylon-persistence/examples/michigan_experiment/observe.rs",
            include_bytes!("observe.rs"),
        ),
        (
            "rust/crates/babylon-persistence/examples/michigan_experiment/artifacts.rs",
            include_bytes!("artifacts.rs"),
        ),
        (
            "content/scenarios/michigan/defines.toml",
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../content/scenarios/michigan/defines.toml"
            )),
        ),
    ];
    let mut fingerprints = BTreeMap::new();
    for (name, embedded) in sources {
        let compiled = super::run::hex(&Sha256::digest(embedded));
        if digest_file(&repo.join(name))? != compiled {
            return Err(contract(format!(
                "compiled experiment source differs from checkout: {name}"
            )));
        }
        fingerprints.insert(name, compiled);
    }
    let head = git(repo, &["rev-parse", "HEAD"])?;
    Ok(serde_json::json!({
        "source_sha": head.trim(), "source_tree_clean": true,
        "binary_sha256": digest_file(&std::env::current_exe()?)?,
        "embedded_source_sha256": fingerprints,
        "scope": "clean execution checkout, matched embedded harness sources and baseline, binary digest; not full build attestation",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_bound_includes_final_newline_and_refuses_overflow() {
        assert_eq!(encode(&"abc", 6).unwrap(), b"\"abc\"\n");
        assert!(encode(&"abc", 5).is_err());
        let mut output = BoundedBytes {
            bytes: vec![1, 2],
            limit: 3,
        };
        assert!(output.write_all(&[3, 4]).is_err());
        assert_eq!(output.bytes, [1, 2]);
    }
    #[test]
    fn expired_deadline_and_cancellation_need_no_sleep() {
        let (sender, receiver) = mpsc::channel();
        assert!(timed_out(&receiver, Duration::ZERO));
        sender.send(()).unwrap();
        assert!(!timed_out(&receiver, Duration::ZERO));
    }
    #[test]
    fn existing_directory_and_artifact_are_never_overwritten() {
        let path = std::env::temp_dir().join(format!(
            "babylon-experiment-artifacts-{}",
            std::process::id()
        ));
        let mut output = Output::create(&path).unwrap();
        assert!(Output::create(&path).is_err());
        output.write_json("summary.json", &1).unwrap();
        assert!(output.write_json("summary.json", &2).is_err());
        assert_eq!(fs::read(path.join("summary.json")).unwrap(), b"1\n");
        fs::remove_file(path.join("summary.json")).unwrap();
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn period_limit_preserves_failure_budget_and_partial_evidence() {
        let path =
            std::env::temp_dir().join(format!("babylon-experiment-bound-{}", std::process::id()));
        let mut output = Output::create(&path).unwrap();
        for period in 1..=64 {
            output.append_period(&period).unwrap();
        }
        let previous = fs::read(path.join("periods.jsonl")).unwrap();
        assert!(output.append_period(&65).is_err());
        assert_eq!(fs::read(path.join("periods.jsonl")).unwrap(), previous);
        assert_eq!(
            output.checksums().unwrap()["periods.jsonl"],
            digest_file(&path.join("periods.jsonl")).unwrap()
        );
        let failure = Progress {
            case: Some("delayed-320".into()),
            last_completed_period: 16,
        };
        write_failure(&path, &failure, "bounded refusal").unwrap();
        let bytes = fs::read(path.join("failure.json")).unwrap();
        assert!(bytes.len() <= FAILURE_RESERVE);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["progress"]["case"], "delayed-320");
        assert_eq!(value["progress"]["last_completed_period"], 16);
        fs::remove_file(path.join("periods.jsonl")).unwrap();
        fs::remove_file(path.join("failure.json")).unwrap();
        fs::remove_dir(path).unwrap();
    }
}
