//! Repo access: root discovery + the two git reads `citation-drift` needs
//! (a tag's file content, a tag's full tree listing), both cached in-process
//! so a corpus that cites the same frozen file many times pays for one
//! `git show`/`git ls-tree`, not one per citation — the whole point of the
//! `check:bsl-sentinels` leg is that it stays fast enough to sit in
//! `rust:check` (task brief: "the leg must be fast, <5s").

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A repo checkout, rooted at `git rev-parse --show-toplevel` — so the tool
/// works from any cwd, not just a hardcoded `dir = "rust"` mise invocation.
pub struct Repo {
    pub root: PathBuf,
    tag_content_cache: RefCell<HashMap<(String, String), Result<String, String>>>,
    tag_tree_cache: RefCell<HashMap<String, Result<Vec<String>, String>>>,
    working_tree_cache: RefCell<Option<Result<Vec<String>, String>>>,
}

/// A `git` invocation with the ambient repo-override variables scrubbed.
/// Git hooks export `GIT_DIR` (pre-push runs this crate through `rust:check`);
/// inherited with no `GIT_WORK_TREE`, it makes git treat the child's cwd as
/// the toplevel — `discover()` mis-roots, and a scratch-repo test resolves
/// the REAL repo's tags. Discovery must come from the process cwd alone.
fn git_command() -> Command {
    let mut cmd = Command::new("git");
    for var in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_COMMON_DIR",
        "GIT_OBJECT_DIRECTORY",
        "GIT_PREFIX",
    ] {
        cmd.env_remove(var);
    }
    cmd
}

impl Repo {
    /// Discover the repo root via `git rev-parse --show-toplevel`, run from
    /// the process's actual cwd (never assumed).
    ///
    /// # Errors
    /// A string describing why `git` could not be run or returned non-zero.
    pub fn discover() -> Result<Self, String> {
        let out = git_command()
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| format!("git rev-parse --show-toplevel: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "git rev-parse --show-toplevel failed: {}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        let root = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        Ok(Self {
            root: PathBuf::from(root),
            tag_content_cache: RefCell::new(HashMap::new()),
            tag_tree_cache: RefCell::new(HashMap::new()),
            working_tree_cache: RefCell::new(None),
        })
    }

    /// Read a working-tree file's full text, relative to the repo root.
    ///
    /// # Errors
    /// The `std::io::Error` message if the file cannot be read.
    pub fn read_working_file(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel}: {e}"))
    }

    /// `git show <tag>:<rel>`, cached per (tag, rel) for the process lifetime.
    ///
    /// # Errors
    /// A string describing why `git` could not be run, or the path does not
    /// exist at that tag.
    pub fn show_tag_file(&self, tag: &str, rel: &str) -> Result<String, String> {
        let key = (tag.to_owned(), rel.to_owned());
        if let Some(cached) = self.tag_content_cache.borrow().get(&key) {
            return cached.clone();
        }
        let spec = format!("{tag}:{rel}");
        let out = git_command()
            .current_dir(&self.root)
            .args(["show", &spec])
            .output()
            .map_err(|e| format!("git show {spec}: {e}"));
        let result = match out {
            Ok(out) if out.status.success() => {
                Ok(String::from_utf8_lossy(&out.stdout).into_owned())
            }
            Ok(out) => Err(format!(
                "git show {spec} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => Err(e),
        };
        self.tag_content_cache
            .borrow_mut()
            .insert(key, result.clone());
        result
    }

    /// `git ls-tree -r --name-only <tag>`, cached per tag for the process
    /// lifetime — the suffix-resolution search for a bare/partial `:material-basis`
    /// citation path walks this list.
    ///
    /// # Errors
    /// A string describing why `git` could not be run or returned non-zero.
    pub fn tag_tree(&self, tag: &str) -> Result<Vec<String>, String> {
        if let Some(cached) = self.tag_tree_cache.borrow().get(tag) {
            return cached.clone();
        }
        let out = git_command()
            .current_dir(&self.root)
            .args(["ls-tree", "-r", "--name-only", tag])
            .output()
            .map_err(|e| format!("git ls-tree -r --name-only {tag}: {e}"));
        let result = match out {
            Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_owned)
                .collect()),
            Ok(out) => Err(format!(
                "git ls-tree -r --name-only {tag} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => Err(e),
        };
        self.tag_tree_cache
            .borrow_mut()
            .insert(tag.to_owned(), result.clone());
        result
    }

    /// `git ls-files` — the tracked working-tree file list a `.rs`/`.rst`
    /// citation resolves against (never the frozen tag), cached for the
    /// process lifetime.
    ///
    /// # Errors
    /// A string describing why `git` could not be run or returned non-zero.
    pub fn working_tree_files(&self) -> Result<Vec<String>, String> {
        if let Some(cached) = self.working_tree_cache.borrow().as_ref() {
            return cached.clone();
        }
        let out = git_command()
            .current_dir(&self.root)
            .args(["ls-files"])
            .output()
            .map_err(|e| format!("git ls-files: {e}"));
        let result = match out {
            Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_owned)
                .collect()),
            Ok(out) => Err(format!(
                "git ls-files failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => Err(e),
        };
        *self.working_tree_cache.borrow_mut() = Some(result.clone());
        result
    }

    /// A file's path relative to the repo root, for display — falls back to
    /// the absolute path if `p` is not under `root` (should not happen for
    /// paths this tool discovers itself).
    #[must_use]
    pub fn display_path(&self, p: &Path) -> String {
        p.strip_prefix(&self.root)
            .unwrap_or(p)
            .display()
            .to_string()
    }
}
