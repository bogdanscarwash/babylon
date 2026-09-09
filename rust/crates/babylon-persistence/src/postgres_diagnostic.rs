//! Secret-safe diagnostics captured at `PostgreSQL` failure boundaries.

use postgres::error::SqlState;

/// Maximum UTF-8 byte length retained from one sanitized server message.
pub const MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES: usize = 256;

const MAX_ERROR_SOURCE_DEPTH: usize = 16;
const MAX_POSTGRES_DIAGNOSTIC_SCAN_BYTES: usize = MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES * 4;
const REDACTED: &str = "<redacted>";
const SENSITIVE_FIELD_NAMES: [&str; 10] = [
    "password", "passwd", "pwd", "dsn", "uri", "url", "token", "secret", "sslkey", "sslcert",
];

/// Stable classification of one `PostgreSQL` client or server failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostgresFailureClassV1 {
    /// The server rejected authentication or authorization establishment.
    Authentication,
    /// No server response was available because the endpoint could not be reached.
    Reachability,
    /// A bounded client or server operation timed out.
    Timeout,
    /// The server rejected a startup setting it does not implement.
    UnsupportedStartupSetting,
    /// The server returned a structured database error.
    ServerRejected,
    /// The client failed without a server diagnostic or transport classification.
    Client,
}

/// Bounded diagnostic retained from a `postgres::Error` without connection material.
#[derive(Clone, PartialEq, Eq)]
pub struct PostgresDiagnosticV1 {
    classification: PostgresFailureClassV1,
    sqlstate: Option<Box<str>>,
    message: Box<str>,
}

impl PostgresDiagnosticV1 {
    /// Capture a stable classification and bounded safe message.
    ///
    /// Only the primary server message is considered. Detail, hint, query text,
    /// connection configuration, and the raw client error chain are never retained.
    #[must_use]
    pub fn capture(error: &postgres::Error) -> Self {
        if let Some(server) = error.as_db_error() {
            let classification = classify_server(server);
            let message = match classification {
                PostgresFailureClassV1::Authentication => "authentication rejected".into(),
                PostgresFailureClassV1::UnsupportedStartupSetting => {
                    "unrecognized configuration parameter <redacted>".into()
                }
                PostgresFailureClassV1::Timeout => "database operation timed out".into(),
                PostgresFailureClassV1::ServerRejected => {
                    sanitize_server_message(server.message()).into_boxed_str()
                }
                PostgresFailureClassV1::Reachability | PostgresFailureClassV1::Client => {
                    unreachable!("server errors have a server classification")
                }
            };
            return Self {
                classification,
                sqlstate: Some(server.code().code().into()),
                message,
            };
        }

        let classification = if error_chain_has_timeout(error) {
            PostgresFailureClassV1::Timeout
        } else if error_chain_has_io(error) {
            PostgresFailureClassV1::Reachability
        } else {
            PostgresFailureClassV1::Client
        };
        let message = match classification {
            PostgresFailureClassV1::Timeout => "database operation timed out",
            PostgresFailureClassV1::Reachability => "database endpoint unreachable",
            PostgresFailureClassV1::Client => "database client rejected operation",
            PostgresFailureClassV1::Authentication
            | PostgresFailureClassV1::UnsupportedStartupSetting
            | PostgresFailureClassV1::ServerRejected => {
                unreachable!("client errors have a client classification")
            }
        };
        Self {
            classification,
            sqlstate: None,
            message: message.into(),
        }
    }

    /// Return the stable failure classification.
    #[must_use]
    pub const fn classification(&self) -> PostgresFailureClassV1 {
        self.classification
    }

    /// Return the five-character SQLSTATE when the server supplied one.
    #[must_use]
    pub fn sqlstate(&self) -> Option<&str> {
        self.sqlstate.as_deref()
    }

    /// Return the bounded sanitized diagnostic message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        Some(&self.message)
    }
}

impl std::fmt::Debug for PostgresDiagnosticV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PostgresDiagnosticV1")
            .field("classification", &self.classification)
            .field("sqlstate", &self.sqlstate)
            .field("message", &self.message)
            .finish()
    }
}

fn classify_server(error: &postgres::error::DbError) -> PostgresFailureClassV1 {
    if error
        .message()
        .starts_with("unrecognized configuration parameter")
    {
        PostgresFailureClassV1::UnsupportedStartupSetting
    } else if error.code().code().starts_with("28") {
        PostgresFailureClassV1::Authentication
    } else if error.code() == &SqlState::QUERY_CANCELED
        || error.code() == &SqlState::LOCK_NOT_AVAILABLE
    {
        PostgresFailureClassV1::Timeout
    } else {
        PostgresFailureClassV1::ServerRejected
    }
}

fn error_chain_has_timeout(error: &(dyn std::error::Error + 'static)) -> bool {
    error_chain_has(error, |source| {
        source
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io_error| {
                matches!(
                    io_error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                )
            })
    })
}

fn error_chain_has_io(error: &(dyn std::error::Error + 'static)) -> bool {
    error_chain_has(error, |source| {
        source.downcast_ref::<std::io::Error>().is_some()
    })
}

fn error_chain_has(
    error: &(dyn std::error::Error + 'static),
    predicate: fn(&(dyn std::error::Error + 'static)) -> bool,
) -> bool {
    let mut current = Some(error);
    for _depth in 0..MAX_ERROR_SOURCE_DEPTH {
        let Some(source) = current else {
            return false;
        };
        if predicate(source) {
            return true;
        }
        current = source.source();
    }
    false
}

fn sanitize_server_message(message: &str) -> String {
    let mut normalized = String::with_capacity(MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES);
    let mut quote = None;
    let mut prior_space = false;
    let mut scanned_bytes = 0_usize;

    for character in message.chars() {
        let character_bytes = character.len_utf8();
        if scanned_bytes.saturating_add(character_bytes) > MAX_POSTGRES_DIAGNOSTIC_SCAN_BYTES {
            break;
        }
        scanned_bytes += character_bytes;

        if let Some(active_quote) = quote {
            if character == active_quote {
                if normalized.len().saturating_add(character_bytes)
                    > MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES
                {
                    break;
                }
                normalized.push(active_quote);
                quote = None;
            }
            continue;
        }
        if matches!(character, '\'' | '"') {
            if normalized
                .len()
                .saturating_add(character_bytes)
                .saturating_add(REDACTED.len())
                > MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES
            {
                break;
            }
            normalized.push(character);
            normalized.push_str(REDACTED);
            quote = Some(character);
            prior_space = false;
            continue;
        }
        if character.is_control() || character.is_whitespace() {
            if !prior_space && !normalized.is_empty() {
                if normalized.len() == MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES {
                    break;
                }
                normalized.push(' ');
            }
            prior_space = true;
            continue;
        }
        if normalized.len().saturating_add(character_bytes) > MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES
        {
            break;
        }
        normalized.push(character);
        prior_space = false;
    }
    if let Some(active_quote) = quote {
        if normalized.len().saturating_add(active_quote.len_utf8())
            <= MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES
        {
            normalized.push(active_quote);
        }
    }

    redact_normalized_message(&normalized)
}

fn redact_normalized_message(normalized: &str) -> String {
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut redacted = String::with_capacity(normalized.len());
    let mut index = 0_usize;
    while index < tokens.len() {
        let token = tokens[index];
        let lowercase = token.to_ascii_lowercase();
        if token.contains("://") {
            push_token(&mut redacted, "<redacted-uri>");
            index += 1;
        } else if is_sensitive_field_name(&lowercase)
            && tokens
                .get(index + 1)
                .is_some_and(|next| next.starts_with('='))
        {
            push_token(&mut redacted, token);
            push_token(&mut redacted, "=");
            push_token(&mut redacted, REDACTED);
            index += if tokens[index + 1] == "=" { 3 } else { 2 };
        } else if is_sensitive_assignment(&lowercase) {
            push_token(&mut redacted, REDACTED);
            index += if lowercase.ends_with('=') && index + 1 < tokens.len() {
                2
            } else {
                1
            };
        } else {
            push_token(&mut redacted, token);
            index += 1;
        }
    }
    truncate_utf8(&mut redacted, MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES);
    redacted
}

fn push_token(output: &mut String, token: &str) {
    if !output.is_empty() {
        output.push(' ');
    }
    output.push_str(token);
}

fn is_sensitive_field_name(token: &str) -> bool {
    SENSITIVE_FIELD_NAMES.contains(&token)
}

fn is_sensitive_assignment(token: &str) -> bool {
    SENSITIVE_FIELD_NAMES.iter().any(|field| {
        token
            .find(field)
            .is_some_and(|offset| token[offset + field.len()..].starts_with('='))
    })
}

fn truncate_utf8(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut boundary = max_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
}

#[cfg(test)]
mod tests {
    use super::{sanitize_server_message, MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES};

    #[test]
    fn server_message_redacts_quoted_values_sensitive_assignments_and_uri_userinfo() {
        let message = "role \"PER288_ROLE_CANARY\" password=PER288_PASSWORD_CANARY \
                       postgres://PER288_USER:PER288_URI_PASSWORD@localhost/db";
        let sanitized = sanitize_server_message(message);

        assert_eq!(sanitized, "role \"<redacted>\" <redacted> <redacted-uri>");
    }

    #[test]
    fn server_message_redacts_spaced_assignments_and_uris_without_userinfo() {
        let message = "password = PER288_SPACED_CANARY dsn= PER288_DSN_CANARY \
                       postgres://localhost/db?password=PER288_URI_CANARY";
        let sanitized = sanitize_server_message(message);

        assert_eq!(sanitized, "password = <redacted> <redacted> <redacted-uri>");
        assert!(!sanitized.contains("PER288_SPACED_CANARY"));
        assert!(!sanitized.contains("PER288_DSN_CANARY"));
        assert!(!sanitized.contains("PER288_URI_CANARY"));
    }

    #[test]
    fn server_message_is_bounded_on_a_utf8_boundary() {
        let sanitized = sanitize_server_message(&"λ".repeat(MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES));

        assert!(sanitized.len() <= MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES);
        assert!(sanitized.is_char_boundary(sanitized.len()));
    }

    #[test]
    fn server_message_scanning_is_bounded_before_redaction() {
        let message = format!(
            "{} password=PER288_BEYOND_SCAN_CANARY",
            "x".repeat(MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES * 64)
        );
        let sanitized = sanitize_server_message(&message);

        assert_eq!(sanitized.len(), MAX_POSTGRES_DIAGNOSTIC_MESSAGE_BYTES);
        assert!(!sanitized.contains("PER288_BEYOND_SCAN_CANARY"));
    }

    #[test]
    fn unterminated_quoted_value_keeps_its_original_delimiter_without_the_value() {
        assert_eq!(
            sanitize_server_message("role 'PER288_UNTERMINATED_CANARY"),
            "role '<redacted>'"
        );
    }
}
