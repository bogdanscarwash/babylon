//! Byte-level validator for the Babylon Markdown profile (ADR249 R4/R5).
//!
//! The profile is GitHub-Flavored Markdown 0.29 extended with the `subject:`
//! URI scheme for internal references and nothing else: no raw HTML, a pinned
//! citation-line format, UTF-8 with LF endings, and exactly three subject-link
//! forms (granted, bare fog, and the display-time pending strikethrough). The
//! validator is deliberately byte-level — no full parser dependency enters the
//! persistence boundary; pulldown-cmark arrives later with `babylon-client`
//! for chip rendering, not here.
//!
//! The Git export rewrite (ADR249 R5) rewrites `subject:` targets to relative
//! `.md` paths and renders bare links as fog chips synthesized from the subject
//! kind and id alone, carrying zero label bytes.

/// Stable profile identity pinned in `contracts/babylon_markdown_v1.yaml`.
pub const BABYLON_MARKDOWN_PROFILE_ID_V1: &str = "babylon-markdown.v1";
/// Pinned citation-line shape: `- **{label}:** {value} — {source_id}; {locator}`.
pub const CITATION_LINE_REGEX_V1: &str =
    r"^- \*\*(?P<label>[^*]+):\*\* (?P<value>.+) — (?P<source_id>[^;]+); (?P<locator>.+)$";
/// Separator between the fog-chip kind word and the public subject id.
pub const FOG_CHIP_SEPARATOR_V1: &str = " · ";
const SUBJECT_SCHEME_PREFIX: &str = "subject:";
const PENDING_MARK: &str = "~~";
const CITATION_SEPARATOR_V1: &str = " — ";
const MAX_MARKDOWN_BYTES_V1: usize = 1_048_576;

/// One typed profile refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BabylonMarkdownErrorV1 {
    /// Markdown bytes were not exact UTF-8.
    NotUtf8,
    /// A CR byte appeared; the profile pins LF endings.
    CrlfEnding,
    /// The '<' byte is refused everywhere: no raw HTML and no autolinks.
    RawHtml,
    /// A link destination did not use the `subject:` URI scheme.
    DisallowedLinkScheme(String),
    /// A `subject:` destination or its label did not match a pinned link form.
    MalformedSubjectLink(String),
    /// A `~~` marker pair did not wrap exactly one subject link token.
    StrikethroughWithoutLink,
    /// Markdown exceeded the pinned profile byte bound.
    TooLarge,
}

impl BabylonMarkdownErrorV1 {
    /// Stable refusal code pinned in the shared vector corpus.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::NotUtf8 => "not_utf8",
            Self::CrlfEnding => "crlf_ending",
            Self::RawHtml => "raw_html",
            Self::DisallowedLinkScheme(_) => "disallowed_link_scheme",
            Self::MalformedSubjectLink(_) => "malformed_subject_link",
            Self::StrikethroughWithoutLink => "strikethrough_without_link",
            Self::TooLarge => "too_large",
        }
    }
}

impl std::fmt::Display for BabylonMarkdownErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "babylon markdown refusal: {self:?}")
    }
}

impl std::error::Error for BabylonMarkdownErrorV1 {}

fn subject_id_width(kind: &str) -> Option<usize> {
    match kind {
        "county" => Some(5),
        "place" => Some(7),
        _ => None,
    }
}

fn validate_subject_destination(
    destination: &str,
) -> Result<(String, String), BabylonMarkdownErrorV1> {
    let Some(rest) = destination.strip_prefix(SUBJECT_SCHEME_PREFIX) else {
        return Err(BabylonMarkdownErrorV1::DisallowedLinkScheme(
            destination.to_owned(),
        ));
    };
    let Some((kind, id)) = rest.split_once('/') else {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            destination.to_owned(),
        ));
    };
    let width = subject_id_width(kind)
        .ok_or_else(|| BabylonMarkdownErrorV1::MalformedSubjectLink(destination.to_owned()))?;
    if id.len() != width || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            destination.to_owned(),
        ));
    }
    Ok((kind.to_owned(), id.to_owned()))
}

/// Parse one link token beginning at `start` (a `[` byte) in validated UTF-8
/// text, returning the label, the subject kind, the subject id, and the index
/// one past the closing `)`.
fn parse_link_token(
    text: &str,
    start: usize,
) -> Result<(String, String, String, usize), BabylonMarkdownErrorV1> {
    let bytes = text.as_bytes();
    debug_assert_eq!(bytes.get(start), Some(&b'['));
    let mut cursor = start + 1;
    while cursor < bytes.len() && bytes[cursor] != b']' {
        if bytes[cursor] == b'[' {
            return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
                "nested open bracket".to_owned(),
            ));
        }
        cursor += 1;
    }
    let Some(destination_start) = cursor.checked_add(1) else {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            "unterminated label".to_owned(),
        ));
    };
    if destination_start >= bytes.len() || bytes[cursor + 1] != b'(' {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            "label without link destination".to_owned(),
        ));
    }
    let mut end = destination_start + 1;
    while end < bytes.len() && bytes[end] != b')' {
        end += 1;
    }
    if end >= bytes.len() {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            "unterminated destination".to_owned(),
        ));
    }
    let label = &text[start + 1..cursor];
    if label.contains('[') || label.contains(']') {
        return Err(BabylonMarkdownErrorV1::MalformedSubjectLink(
            "label bracket bytes".to_owned(),
        ));
    }
    let destination = &text[destination_start + 1..end];
    let (kind, id) = validate_subject_destination(destination)?;
    Ok((label.to_owned(), kind, id, end + 1))
}

/// Validate Markdown bytes against the Babylon Markdown profile.
///
/// # Errors
/// Refuses the first profile violation: non-UTF-8 bytes, a CR byte, a '<'
/// byte, a non-`subject:` link scheme, a malformed subject link, or a `~~`
/// marker pair that does not wrap exactly one subject link token.
pub fn validate_babylon_markdown_v1(markdown: &[u8]) -> Result<(), BabylonMarkdownErrorV1> {
    if markdown.len() > MAX_MARKDOWN_BYTES_V1 {
        return Err(BabylonMarkdownErrorV1::TooLarge);
    }
    let text = std::str::from_utf8(markdown).map_err(|_| BabylonMarkdownErrorV1::NotUtf8)?;
    if text.contains('\r') {
        return Err(BabylonMarkdownErrorV1::CrlfEnding);
    }
    if text.contains('<') {
        return Err(BabylonMarkdownErrorV1::RawHtml);
    }
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'[' {
            let (_, _, _, end) = parse_link_token(text, cursor)?;
            cursor = end;
            continue;
        }
        if text[cursor..].starts_with(PENDING_MARK) {
            let link_start = cursor + PENDING_MARK.len();
            if bytes.get(link_start) != Some(&b'[') {
                return Err(BabylonMarkdownErrorV1::StrikethroughWithoutLink);
            }
            let (_, _, _, end) = parse_link_token(text, link_start)?;
            if !text[end..].starts_with(PENDING_MARK) {
                return Err(BabylonMarkdownErrorV1::StrikethroughWithoutLink);
            }
            cursor = end + PENDING_MARK.len();
            continue;
        }
        let Some(next_char) = text[cursor..].chars().next() else {
            break;
        };
        cursor += next_char.len_utf8();
    }
    Ok(())
}

/// Synthesize the fog chip for a bare link from public structure alone.
///
/// The chip carries zero label bytes: only the subject kind word, the pinned
/// separator, and the public subject id (ADR249 R5).
#[must_use]
pub fn fog_chip_v1(subject_kind: &str, subject_id: &str) -> String {
    format!("unknown {subject_kind}{FOG_CHIP_SEPARATOR_V1}{subject_id}")
}

/// Return whether one line matches the pinned citation-line format
/// `- **{label}:** {value} — {source_id}; {locator}`.
///
/// The detection accepts exactly the language of [`CITATION_LINE_REGEX_V1`]:
/// the label is one or more non-`*` bytes ending at the pinned `:** `
/// boundary, the value is one or more bytes greedily ending at the last
/// ` — ` that leaves a `[^;]+`; ` ` tail, and the locator is one or more
/// bytes to the end.
#[must_use]
pub fn is_citation_line_v1(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("- **") else {
        return false;
    };
    let Some((label, rest)) = rest.split_once(":** ") else {
        return false;
    };
    if label.is_empty() || label.contains('*') {
        return false;
    }
    let mut end = rest.len();
    while let Some(position) = rest[..end].rfind(CITATION_SEPARATOR_V1) {
        let (value, tail) = rest.split_at(position);
        end = position;
        if value.is_empty() {
            continue;
        }
        let Some((source_id, locator)) = tail[CITATION_SEPARATOR_V1.len()..].split_once("; ")
        else {
            continue;
        };
        if !source_id.is_empty() && !source_id.contains(';') && !locator.is_empty() {
            return true;
        }
    }
    false
}

/// Rewrite one validated link token into its Git export form.
///
/// The granted and pending forms rewrite the destination to a relative `.md`
/// path; the bare form renders the fog chip, which carries zero label bytes.
fn push_export_token(
    output: &mut String,
    text: &str,
    start: usize,
) -> Result<usize, BabylonMarkdownErrorV1> {
    let (label, kind, id, end) = parse_link_token(text, start)?;
    if label.is_empty() {
        output.push_str(&fog_chip_v1(&kind, &id));
    } else {
        output.push('[');
        output.push_str(&label);
        output.push_str("](./");
        output.push_str(&kind);
        output.push('/');
        output.push_str(&id);
        output.push_str(".md)");
    }
    Ok(end)
}

/// Validate and rewrite Markdown bytes into the Git export form (ADR249 R5).
///
/// # Errors
/// Propagates the first [`BabylonMarkdownErrorV1`] profile refusal.
pub fn git_export_markdown_v1(markdown: &str) -> Result<String, BabylonMarkdownErrorV1> {
    validate_babylon_markdown_v1(markdown.as_bytes())?;
    let mut output = String::with_capacity(markdown.len());
    let bytes = markdown.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] == b'[' {
            let end = push_export_token(&mut output, markdown, cursor)?;
            cursor = end;
            continue;
        }
        if markdown[cursor..].starts_with(PENDING_MARK) {
            output.push_str(PENDING_MARK);
            let link_start = cursor + PENDING_MARK.len();
            let end = push_export_token(&mut output, markdown, link_start)?;
            output.push_str(PENDING_MARK);
            cursor = end + PENDING_MARK.len();
            continue;
        }
        let Some(next_char) = markdown[cursor..].chars().next() else {
            break;
        };
        output.push(next_char);
        cursor += next_char.len_utf8();
    }
    Ok(output)
}
