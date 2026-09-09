//! The exact server capabilities `initialize` advertises: the diagnostic
//! baseline plus the finite-probability authoring surface. This module's own
//! test pins the serialized JSON so accidental capability drift is a failing
//! diff, not a silent behavior change.

use lsp_types::{
    CompletionOptions, DiagnosticOptions, DiagnosticServerCapabilities, HoverProviderCapability,
    OneOf, SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, SignatureHelpOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, WorkDoneProgressOptions, WorkspaceFoldersServerCapabilities,
    WorkspaceServerCapabilities,
};

/// Stable semantic-token type indices returned by the probability authoring
/// surface. Changing this order changes every token response, so the exact
/// serialized legend is pinned below.
pub const TOKEN_TYPE_KEYWORD: u32 = 0;
pub const TOKEN_TYPE_FUNCTION: u32 = 1;
pub const TOKEN_TYPE_NUMBER: u32 = 2;
pub const TOKEN_TYPE_ENUM_MEMBER: u32 = 3;
pub const TOKEN_TYPE_VARIABLE: u32 = 4;

/// The [`ServerCapabilities`] `initialize` advertises.
///
/// Full sync, not Incremental (§6.1): the loader re-reads the whole file on
/// every change regardless, the content estate is small (largest file
/// 24 KB against a ~440 KB total), and Full sync deletes a whole class of
/// position-mapping bugs — Incremental is a wave-2 optimization to be taken
/// only against a measurement.
///
/// Push AND pull diagnostics: `interFileDependencies: true` and
/// `workspaceDiagnostics: true` both hold because a content set spans
/// files (a manifest-declared set, not one file in isolation) and
/// `workspace/diagnostic` is the capability a coding agent needs — one
/// synchronous "entire current error state" call after an edit.
///
/// `positionEncoding` is left unset (omitted from the JSON), which the LSP
/// spec makes the client default to UTF-16 — the encoding
/// [`crate::line_index`] assumes throughout.
#[must_use]
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: None,
            },
        )),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: Some("bsl".to_owned()),
            inter_file_dependencies: true,
            workspace_diagnostics: true,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        })),
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["(".to_owned(), ":".to_owned(), " ".to_owned()]),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        signature_help_provider: Some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_owned(), " ".to_owned()]),
            retrigger_characters: Some(vec![" ".to_owned()]),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::NUMBER,
                        SemanticTokenType::ENUM_MEMBER,
                        SemanticTokenType::VARIABLE,
                    ],
                    token_modifiers: vec![SemanticTokenModifier::READONLY],
                },
                range: None,
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: None,
        }),
        ..ServerCapabilities::default()
    }
}

#[cfg(test)]
mod tests {
    use super::server_capabilities;

    /// Exact golden JSON for capability drift, not just a smoke test.
    #[test]
    fn capability_advertisement_is_pinned_exactly() {
        let value = serde_json::to_value(server_capabilities()).expect("capabilities serialize");
        let expected = serde_json::json!({
            "textDocumentSync": { "openClose": true, "change": 1 },
            "hoverProvider": true,
            "completionProvider": {
                "resolveProvider": false,
                "triggerCharacters": ["(", ":", " "]
            },
            "signatureHelpProvider": {
                "triggerCharacters": ["(", " "],
                "retriggerCharacters": [" "]
            },
            "semanticTokensProvider": {
                "legend": {
                    "tokenTypes": ["keyword", "function", "number", "enumMember", "variable"],
                    "tokenModifiers": ["readonly"]
                },
                "full": true
            },
            "diagnosticProvider": {
                "identifier": "bsl",
                "interFileDependencies": true,
                "workspaceDiagnostics": true
            },
            "workspace": {
                "workspaceFolders": { "supported": true, "changeNotifications": true }
            }
        });
        assert_eq!(value, expected);
    }
}
