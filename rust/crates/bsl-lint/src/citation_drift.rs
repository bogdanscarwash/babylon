//! `citation-drift`: every `:material-basis` string in
//! `rust/crates/babylon-tick/content/rules/*.bsl` cites a frozen-Python or
//! working-tree span (`<path>:N` or `<path>:N-M`). This check resolves each
//! cite and asks two questions babylon-bsl's reader has no way to see (it
//! reads the string as an opaque blob, per §2.2): does the target exist and
//! is it long enough (tier 1, FAILs), and does a keyword from the citing
//! sentence actually show up near the cited span (tier 2, WARNs — a span
//! that's in-bounds but names the wrong lines, the "right file, wrong line"
//! shape the task brief's two landed incidents share).
//!
//! # The citation micro-grammar this module reads
//!
//! `:material-basis` strings are free-text English, not BSL — there is no
//! existing parser for the citations INSIDE them, so this module owns that
//! extraction (the "never write a second parser" rule is about the BSL
//! grammar itself, read once via [`babylon_bsl::reader::read_all`] below).
//! Two shapes, observed across the whole corpus (2026-08-18 survey):
//!
//! - **Full**: `<path>:N` or `<path>:N-M`, `<path>` ending `.py`/`.rs`/`.rst`
//!   (`solidarity.py:97-203`, `formulas/solidarity.py:36`).
//! - **Bare**: `:N` or `:N-M` with no path (`(:126-130)`), which inherits
//!   the nearest earlier full RANGE citation's path — a single-line full
//!   citation (no `-M`) is read as a parenthetical aside and does NOT
//!   change that anchor (verified against every citation in the corpus:
//!   `solidarity.bsl:171`'s `formulas/solidarity.py:36` aside sits between
//!   two anchor spans and must not steal the bare citations after it).
//!
//! Known gap, not attempted: a comma-continuation with no repeated colon
//! (`decomposition.py:150-208, 296-299` — the `296-299` half) is silently
//! un-extracted. False negative, not a false failure — acceptable for a
//! WARN/FAIL tool whose job is to catch drift, not transcribe every cite.

use crate::finding::{Finding, Severity};
use crate::repo::Repo;
use babylon_bsl::{read_all, Atom, SExpr};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const CHECK: &str = "citation-drift";

/// The frozen-Python reference: Amendment AE (ADR172) pins the Python
/// engine's source at this tag; the working tree "may diverge or be
/// deleted later" (task brief), so `.py` cites resolve against the tag,
/// never the working tree.
const FREEZE_TAG: &str = "p27-python-freeze";

/// §2.2's own cap on `:material-basis` — E-LEX-026 fails the reader at
/// 1024 bytes; this check warns well before that (900), the task brief's
/// number, chosen against measured near-cap blocks (987/981/959 bytes).
const MATERIAL_BASIS_WARN_BYTES: usize = 900;

/// A keyword token must be at least this long to count (task brief: "a
/// keyword token (len >= 4, non-stopword)").
const MIN_KEYWORD_LEN: usize = 4;

/// Common short/function words filtered out of the tier-2 keyword set —
/// none of these would ever discriminate "is this really the cited span",
/// so treating them as evidence would make tier 2 pass on noise.
const STOPWORDS: &[&str] = &[
    "this", "that", "with", "from", "when", "then", "have", "were", "also", "into", "onto", "than",
    "which", "where", "after", "before", "only", "each", "being", "while", "every", "still",
    "first", "never", "under", "above", "about", "again", "their", "there", "these", "those",
    "would", "could", "should", "doing", "does", "read", "reads", "write", "writes", "self",
    "here", "both", "same", "form", "line", "lines",
];

fn full_citation_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"([A-Za-z0-9_./-]+\.(?:py|rs|rst)):(\d+)(?:-(\d+))?")
            .expect("static citation regex must compile")
    })
}

fn bare_citation_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r":(\d+)(?:-(\d+))?").expect("static citation regex must compile"))
}

fn keyword_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[A-Za-z]+").expect("static keyword regex must compile"))
}

/// Run `citation-drift` over `paths` (files and/or directories of `.bsl`
/// files; empty = the default content/rules directory).
///
/// # Errors
/// A string when a `.bsl` file cannot be listed, read, or parsed — an
/// infrastructure failure, distinct from a citation finding.
pub fn run(repo: &Repo, paths: &[String]) -> Result<Vec<Finding>, String> {
    let files = discover_bsl_files(repo, paths)?;
    let mut findings = Vec::new();
    for file in files {
        findings.extend(check_file(repo, &file)?);
    }
    Ok(findings)
}

fn discover_bsl_files(repo: &Repo, paths: &[String]) -> Result<Vec<PathBuf>, String> {
    if paths.is_empty() {
        let dir = repo.root.join("rust/crates/babylon-tick/content/rules");
        return list_bsl_files(&dir);
    }
    let mut files = Vec::new();
    for p in paths {
        let full = repo.root.join(p);
        if full.is_dir() {
            files.extend(list_bsl_files(&full)?);
        } else {
            files.push(full);
        }
    }
    files.sort();
    Ok(files)
}

fn list_bsl_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "bsl"))
        .collect();
    files.sort();
    Ok(files)
}

fn check_file(repo: &Repo, file: &Path) -> Result<Vec<Finding>, String> {
    let display_file = repo.display_path(file);
    let bytes = std::fs::read(file).map_err(|e| format!("{display_file}: {e}"))?;
    let forms = read_all(&bytes).map_err(|e| format!("{display_file}: {e:?}"))?;
    let raw_text = String::from_utf8_lossy(&bytes).into_owned();
    let raw_lines: Vec<&str> = raw_text.lines().collect();

    let mut findings = Vec::new();
    let mut search_from = 0usize;
    for form in &forms {
        let Some(material_basis) = rule_material_basis(form) else {
            continue;
        };
        let Some(line) = locate_line(&raw_lines, search_from, material_basis) else {
            // Can't place it precisely; still worth checking, at line 0
            // ("unknown") rather than dropping the rule's citations.
            findings.extend(check_material_basis(
                repo,
                &display_file,
                0,
                material_basis,
            )?);
            continue;
        };
        search_from = line; // monotonic: next rule's field is further down
        findings.extend(check_material_basis(
            repo,
            &display_file,
            line,
            material_basis,
        )?);
    }
    Ok(findings)
}

/// `(rule … :role mechanic :evidence derived :material-basis "…" …)`'s string value, if `form` is a rule.
fn rule_material_basis(form: &SExpr) -> Option<&str> {
    let SExpr::List(items) = form else {
        return None;
    };
    if !matches!(items.first(), Some(SExpr::Atom(Atom::Symbol(s))) if s == "rule") {
        return None;
    }
    items.windows(2).find_map(|pair| match pair {
        [SExpr::Atom(Atom::Keyword(kw)), SExpr::Atom(Atom::Str(s))] if kw == "material-basis" => {
            Some(s.as_str())
        }
        _ => None,
    })
}

/// Find the 1-based line, at or after `search_from` (0-based index), whose
/// text contains `:material-basis` and a leading probe of `content` — the
/// reader's AST carries no spans, so this correlates the parsed string back
/// to its source line by content match rather than reparsing.
fn locate_line(lines: &[&str], search_from: usize, content: &str) -> Option<usize> {
    let probe: String = content.chars().take(24).collect();
    lines
        .iter()
        .enumerate()
        .skip(search_from)
        .find(|(_, line)| line.contains(":material-basis") && line.contains(probe.as_str()))
        .map(|(i, _)| i + 1)
}

fn check_material_basis(
    repo: &Repo,
    file: &str,
    line: usize,
    text: &str,
) -> Result<Vec<Finding>, String> {
    let mut findings = Vec::new();
    if text.len() > MATERIAL_BASIS_WARN_BYTES {
        findings.push(Finding {
            check: CHECK,
            file: file.to_owned(),
            line,
            what: format!(
                "material-basis is {} bytes, over the {MATERIAL_BASIS_WARN_BYTES}-byte warn floor",
                text.len()
            ),
            evidence: "E-LEX-026 hard-caps the reader at 1024 bytes".to_owned(),
            severity: Severity::Warn,
        });
    }
    for span in resolve_citations(text) {
        findings.extend(check_citation(repo, file, line, text, &span)?);
    }
    Ok(findings)
}

/// One citation after anchor resolution: the path token to resolve
/// (explicit on a full citation, inherited from the last range anchor on a
/// bare one) plus the byte span in `text` used for tier 2's sentence cut.
struct ResolvedSpan {
    path_token: String,
    start: u32,
    end: u32,
    text_start: usize,
    text_end: usize,
}

struct RawCite {
    offset: usize,
    end_offset: usize,
    path: Option<String>,
    start: u32,
    end: u32,
    is_range: bool,
}

fn resolve_citations(text: &str) -> Vec<ResolvedSpan> {
    let mut full_ranges: Vec<(usize, usize)> = Vec::new();
    let mut raws: Vec<RawCite> = Vec::new();

    for caps in full_citation_re().captures_iter(text) {
        let whole = caps.get(0).expect("group 0 always matches");
        let start: u32 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let end = caps
            .get(3)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(start);
        full_ranges.push((whole.start(), whole.end()));
        raws.push(RawCite {
            offset: whole.start(),
            end_offset: whole.end(),
            path: Some(caps[1].to_owned()),
            start,
            end,
            is_range: caps.get(3).is_some(),
        });
    }
    for caps in bare_citation_re().captures_iter(text) {
        let whole = caps.get(0).expect("group 0 always matches");
        if full_ranges
            .iter()
            .any(|&(s, e)| whole.start() >= s && whole.start() < e)
        {
            continue;
        }
        let start: u32 = caps
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let end = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(start);
        raws.push(RawCite {
            offset: whole.start(),
            end_offset: whole.end(),
            path: None,
            start,
            end,
            is_range: caps.get(2).is_some(),
        });
    }
    raws.sort_by_key(|r| r.offset);

    let mut anchor: Option<String> = None;
    let mut resolved = Vec::new();
    for raw in raws {
        let path_token = match raw.path {
            Some(p) => {
                if raw.is_range {
                    anchor = Some(p.clone());
                }
                p
            }
            None => match &anchor {
                Some(a) => a.clone(),
                None => continue,
            },
        };
        resolved.push(ResolvedSpan {
            path_token,
            start: raw.start,
            end: raw.end,
            text_start: raw.offset,
            text_end: raw.end_offset,
        });
    }
    resolved
}

enum ResolveErr {
    NotFound,
    Ambiguous(Vec<String>),
}

/// Resolve a cited path (bare filename or partial suffix) to exactly one
/// entry of `listing`. A bare filename first tries `direct_prefix` (the
/// dominant "this is my own System file" convention every header comment
/// in this corpus states literally); failing that, a unique suffix match
/// wins. Zero or 2+ matches are both refusals — silently guessing among
/// ambiguous candidates would be worse than flagging it.
fn resolve_in_listing(
    cited: &str,
    listing: &[String],
    direct_prefix: Option<&str>,
) -> Result<String, ResolveErr> {
    if !cited.contains('/') {
        if let Some(prefix) = direct_prefix {
            let direct = format!("{prefix}{cited}");
            if listing.iter().any(|p| p == &direct) {
                return Ok(direct);
            }
        }
    }
    let suffix = format!("/{cited}");
    let matches: Vec<String> = listing
        .iter()
        .filter(|p| p.as_str() == cited || p.ends_with(&suffix))
        .cloned()
        .collect();
    match matches.len() {
        0 => Err(ResolveErr::NotFound),
        1 => Ok(matches.into_iter().next().expect("len checked == 1")),
        _ => Err(ResolveErr::Ambiguous(matches)),
    }
}

fn check_citation(
    repo: &Repo,
    file: &str,
    line: usize,
    text: &str,
    span: &ResolvedSpan,
) -> Result<Vec<Finding>, String> {
    let is_frozen = span.path_token.ends_with(".py");
    let cited_display = if span.end == span.start {
        format!("{}:{}", span.path_token, span.start)
    } else {
        format!("{}:{}-{}", span.path_token, span.start, span.end)
    };

    let resolution = if is_frozen {
        resolve_in_listing(
            &span.path_token,
            &repo.tag_tree(FREEZE_TAG)?,
            Some("src/babylon/engine/systems/"),
        )
    } else {
        resolve_in_listing(&span.path_token, &repo.working_tree_files()?, None)
    };

    let resolved_path = match resolution {
        Ok(p) => p,
        Err(ResolveErr::NotFound) => {
            return Ok(vec![Finding {
                check: CHECK,
                file: file.to_owned(),
                line,
                what: format!("cites {cited_display}, no such file"),
                evidence: if is_frozen {
                    format!("not found under {FREEZE_TAG}'s tree")
                } else {
                    "not found in the working tree".to_owned()
                },
                severity: Severity::Fail,
            }]);
        }
        Err(ResolveErr::Ambiguous(matches)) => {
            return Ok(vec![Finding {
                check: CHECK,
                file: file.to_owned(),
                line,
                what: format!("cites {cited_display}, ambiguous filename"),
                evidence: format!("matches: {}", matches.join(", ")),
                severity: Severity::Fail,
            }]);
        }
    };

    let content = if is_frozen {
        repo.show_tag_file(FREEZE_TAG, &resolved_path)?
    } else {
        repo.read_working_file(&resolved_path)?
    };
    let target_lines: Vec<&str> = content.lines().collect();
    let line_count = target_lines.len();

    if span.start < 1 || span.end as usize > line_count || span.start > span.end {
        return Ok(vec![Finding {
            check: CHECK,
            file: file.to_owned(),
            line,
            what: format!("cites {cited_display}, out of bounds"),
            evidence: format!("{resolved_path} has {line_count} lines"),
            severity: Severity::Fail,
        }]);
    }

    Ok(tier2_keyword_finding(
        file,
        line,
        text,
        span,
        &cited_display,
        &resolved_path,
        &target_lines,
    )
    .into_iter()
    .collect())
}

/// The prose around a citation, WITH the citation's own matched text
/// (`solidarity.py:97-203`) excised — its path token trivially contains a
/// real word (`solidarity`) that would then trivially appear near its own
/// target span, turning every citation into a guaranteed tier-2 pass
/// regardless of whether the SURROUNDING description names the right
/// content.
fn sentence_around(text: &str, start: usize, end: usize) -> String {
    let before = text[..start].rfind(['.', ';']).map_or(0, |i| i + 1);
    let after = text[end..].find(['.', ';']).map_or(text.len(), |i| end + i);
    format!("{} {}", text[before..start].trim(), text[end..after].trim())
}

fn keyword_tokens(sentence: &str) -> Vec<String> {
    keyword_re()
        .find_iter(sentence)
        .map(|m| m.as_str().to_lowercase())
        .filter(|w| w.len() >= MIN_KEYWORD_LEN && !STOPWORDS.contains(&w.as_str()))
        .collect()
}

fn tier2_keyword_finding(
    file: &str,
    line: usize,
    text: &str,
    span: &ResolvedSpan,
    cited_display: &str,
    resolved_path: &str,
    target_lines: &[&str],
) -> Option<Finding> {
    let sentence = sentence_around(text, span.text_start, span.text_end);
    let keywords = keyword_tokens(&sentence);
    if keywords.is_empty() {
        return None; // nothing to check against — not this tier's problem
    }
    let lo = span.start.saturating_sub(5).max(1) as usize;
    let hi = (span.end as usize + 5).min(target_lines.len());
    if lo > hi || hi == 0 {
        return None;
    }
    let window: String = target_lines[(lo - 1)..hi].join("\n").to_lowercase();
    let hit = keywords.iter().any(|k| window.contains(k.as_str()));
    if hit {
        return None;
    }
    Some(Finding {
        check: CHECK,
        file: file.to_owned(),
        line,
        what: format!("cites {cited_display}, no keyword nearby"),
        evidence: format!(
            "none of [{}] appear in {resolved_path}:{lo}-{hi}",
            keywords.join(", ")
        ),
        severity: Severity::Warn,
    })
}
