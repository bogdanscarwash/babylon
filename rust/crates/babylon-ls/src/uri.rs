//! `file:` URI <-> filesystem path conversions for `lsp-types` 0.97's
//! `fluent-uri`-backed [`Uri`] — the two conversions `url::Url` used to
//! provide (`from_file_path`/`to_file_path`) that 0.97 dropped when it
//! replaced the `url` re-export with the `Uri` newtype. The only scheme
//! this crate ever resolves to a path is `file:`; every other URI is a
//! legitimate "not a local file" condition, answered with `None`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use lsp_types::Uri;

/// The URI path bytes that need no percent-encoding: the unreserved set
/// (RFC 3986 §2.3) plus `/` — this crate only ever runs on Unix
/// (`content_relative_path`'s own note), where `Path`'s components already
/// use `/`.
const PATH_ENCODE_KEEP: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~/";

/// Build the `file:` URI for an absolute `path`: percent-encode every byte
/// outside `PATH_ENCODE_KEEP` as `%XX` (uppercase hex, RFC 3986 §2.1's
/// own consistency recommendation) and parse `file://` + the encoded path
/// — the no-authority `file:///abs/path` form every LSP client sends.
/// `None` for a relative path or a string the platform cannot losslessly
/// represent (the same refusal conditions `url::Url::from_file_path` had).
#[must_use]
pub fn uri_from_file_path(path: &Path) -> Option<Uri> {
    let path = path.to_str()?;
    if !path.starts_with('/') {
        return None;
    }
    let mut encoded = String::with_capacity(path.len() + "file://".len());
    encoded.push_str("file://");
    for byte in path.bytes() {
        if PATH_ENCODE_KEEP.contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    Uri::from_str(&encoded).ok()
}

/// The filesystem path a `file:` URI names (RFC 8089's no-authority, or
/// `localhost`, form), percent-decoded. `None` for any other scheme or a
/// remote authority.
#[must_use]
pub fn file_path_from_uri(uri: &Uri) -> Option<PathBuf> {
    if !uri.scheme()?.eq_lowercase("file") {
        return None;
    }
    if let Some(authority) = uri.authority() {
        // `file:///path` carries an EMPTY authority; RFC 8089 also allows an
        // explicit `localhost`. Any other host is a remote file, not a path.
        let host = authority.host().as_str();
        if !host.is_empty() && !host.eq_ignore_ascii_case("localhost") {
            return None;
        }
    }
    Some(PathBuf::from(percent_decode(uri.path().as_str())))
}

/// Decode `%XX` escapes (RFC 3986 §2.1). Infallible by construction:
/// `Uri`'s own parser has already refused a `%` not introducing two hex
/// digits before this crate ever sees the string.
fn percent_decode(encoded: &str) -> String {
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            // SAFETY-free indexing: the parser guaranteed the two hex
            // digits exist.
            let hi = hex_digit(bytes[index + 1]);
            let lo = hex_digit(bytes[index + 2]);
            decoded.push(hi << 4 | lo);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    // The encoded string was valid UTF-8 and escapes decode to full code
    // points only when the parser validated the URI; a lone surrogate or
    // partial sequence cannot survive `Uri::parse`.
    String::from_utf8(decoded).expect("Uri parser validated percent escapes")
}

fn hex_digit(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'A'..=b'F' => b - b'A' + 10,
        b'a'..=b'f' => b - b'a' + 10,
        // Unreachable: `Uri::parse` validated every `%XX` escape.
        _ => unreachable!("Uri parser validated percent escapes"),
    }
}

#[cfg(test)]
mod tests {
    use super::{file_path_from_uri, uri_from_file_path};
    use std::path::Path;

    fn uri(s: &str) -> lsp_types::Uri {
        s.parse::<lsp_types::Uri>().expect("valid test URI")
    }

    #[test]
    fn absolute_unix_path_roundtrips_through_file_uri() {
        let path = Path::new("/virtual/rules/probe.bsl");
        let uri = uri_from_file_path(path).expect("absolute path encodes");
        assert_eq!(uri.as_str(), "file:///virtual/rules/probe.bsl");
        assert_eq!(file_path_from_uri(&uri).as_deref(), Some(path));
    }

    #[test]
    fn non_ascii_bytes_are_percent_encoded_and_decoded() {
        let path = Path::new("/tmp/naïve rule.bsl");
        let uri = uri_from_file_path(path).expect("absolute path encodes");
        assert_eq!(uri.as_str(), "file:///tmp/na%C3%AFve%20rule.bsl");
        assert_eq!(file_path_from_uri(&uri).as_deref(), Some(path));
    }

    #[test]
    fn relative_path_and_non_file_scheme_refuse() {
        assert!(uri_from_file_path(Path::new("relative/probe.bsl")).is_none());
        assert!(file_path_from_uri(&uri("https://example.com/a.bsl")).is_none());
        assert!(file_path_from_uri(&uri("file://remotehost/a.bsl")).is_none());
    }

    #[test]
    fn malformed_percent_escape_is_refused_at_uri_parse_time() {
        // `Uri`'s own parser rejects a `%` not introducing two hex digits,
        // so the malformed-escape case `url::Url::to_file_path` used to
        // surface at conversion time never reaches `file_path_from_uri`.
        assert!("file:///rules/%zz.bsl".parse::<lsp_types::Uri>().is_err());
        assert!("file:///rules/%2.bsl".parse::<lsp_types::Uri>().is_err());
    }
}
