//! The `initialize` -> `initialized` -> (`didOpen`/`didChange`/`didClose`)*
//! -> `shutdown` -> `exit` lifecycle (Task 5.3) — the ONE dispatch loop
//! `main.rs`'s stdio wire-up calls, and the same function
//! `tests/protocol.rs` drives through the real binary and this module's
//! own unit tests drive through [`lsp_server::Connection::memory`].
//!
//! **Observes-only (global constraint 1).** Nothing in this loop writes
//! content or gates a load — the load path (`babylon-bsl`/`babylon-tick`)
//! stays the only door.
//!
//! **Diagnostics push + pull (Task 6, #652, plan §6.5).** Every
//! `didOpen`/`didChange` recomputes and pushes
//! `textDocument/publishDiagnostics` for the affected document — including
//! the empty-array clear on a recompute that finds nothing, since "newly
//! pushed diagnostics always replace" (§6.5). `textDocument/diagnostic`
//! and `workspace/diagnostic` answer the same pull the client can make
//! any time, `full`/`unchanged` keyed on [`crate::diagnostics::
//! compute_result_id`]'s own `resultId`.
//!
//! **No panic-catching (global constraint 8).** A panic in a handler here
//! is a bug; this module never wraps dispatch in `catch_unwind` — the
//! server logs and dies, and the client restarts it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lsp_server::{
    Connection, ErrorCode, Message, Notification as RawNotification, Request as RawRequest,
    RequestId, Response,
};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument, Exit,
    LogMessage, Notification as _, PublishDiagnostics,
};
use lsp_types::request::{
    Completion, DocumentDiagnosticRequest, HoverRequest, RegisterCapability, Request as _,
    SemanticTokensFullRequest, SignatureHelpRequest, WorkspaceDiagnosticRequest,
};
use lsp_types::{
    ClientCapabilities, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
    DidChangeWatchedFilesRegistrationOptions, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentDiagnosticParams, DocumentDiagnosticReport,
    DocumentDiagnosticReportResult, FileSystemWatcher, FullDocumentDiagnosticReport, GlobPattern,
    HoverParams, InitializeParams, LogMessageParams, MessageType, PublishDiagnosticsParams,
    Registration, RegistrationParams, RelatedFullDocumentDiagnosticReport,
    RelatedUnchangedDocumentDiagnosticReport, SemanticTokensParams, SemanticTokensResult,
    SignatureHelpParams, UnchangedDocumentDiagnosticReport, Uri, WorkspaceDiagnosticParams,
    WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport, WorkspaceUnchangedDocumentDiagnosticReport,
};

use crate::authoring::{
    completion_items, hover, semantic_tokens, signature_help, AuthoringSnapshot,
};
use crate::capabilities::server_capabilities;
use crate::content_manifest::ContentSetManifest;
use crate::diagnostics::compute_result_id;
use crate::document_store::DocumentStore;
use crate::pass::{
    analyze_probability_authoring, content_relative_path, diagnose_bsl, LiveSourceReader,
    SourceReader,
};
use crate::uri::{file_path_from_uri, uri_from_file_path};

/// Exit code for a clean `shutdown` -> `exit` sequence (the LSP spec:
/// "exit should exit... with success code 0 if... shutdown request was
/// received before").
const EXIT_CLEAN: i32 = 0;

/// Exit code for `exit` arriving without a preceding `shutdown`, or for
/// the connection dropping before either — the spec's own "non-zero
/// (i.e. error) exit code" for the improper sequence.
const EXIT_UNCLEAN: i32 = 1;

/// The fixed registration id this crate's ONE outbound
/// `client/registerCapability` request uses — a constant, not a generated
/// id, because Task 5 sends exactly one such request per server lifetime.
const WATCHED_FILES_REGISTRATION_ID: &str = "babylon-ls/watched-files";

/// Everything the dispatch loop threads through every handler: the open
/// documents, the (optional — a workspace may have none, or a malformed
/// one) content-set manifest and its own content root, and the last
/// `resultId` this server computed per URI, for pull's `unchanged` answer.
/// `result_ids`' `HashMap` iteration never feeds output order (global
/// constraint 2) — every access is a point lookup/insert by URI.
struct ServerState {
    store: DocumentStore,
    manifest: Option<ContentSetManifest>,
    content_root: Option<PathBuf>,
    result_ids: HashMap<Uri, String>,
}

/// A bounded, wave-1 heuristic for finding `content-sets.toml` from a
/// workspace root (Task 5.4's own remaining scope, picked up here because
/// push/pull cannot resolve a content set without it): the plan's own
/// charter decision 3 fixes ONE location in THIS repo
/// (`rust/crates/babylon-tick/content/content-sets.toml`); a bare
/// `content-sets.toml` at the workspace root is the general fallback. Two
/// checks, not a directory walk (Power-of-10 rule 2) — a full "walk up
/// from every file being diagnosed" discovery (the plan's fuller 5.4
/// description) is a disclosed gap, not implemented here.
fn discover_manifest(workspace_root: &Path) -> Option<PathBuf> {
    let candidates = [
        workspace_root.join("rust/crates/babylon-tick/content/content-sets.toml"),
        workspace_root.join("content-sets.toml"),
    ];
    candidates.into_iter().find(|path| path.is_file())
}

/// Build the server's manifest/content-root state from `InitializeParams`'
/// own workspace-folder / `rootUri` fields (both read here, before the
/// caller's own `capabilities` extraction moves the value) — a malformed
/// manifest degrades to `manifest: None` (every file then reports as
/// having no manifest row, §6.3's own File-tier default) rather than a
/// hard failure; `initialize` must still succeed.
fn discover_state(params: &InitializeParams) -> (Option<ContentSetManifest>, Option<PathBuf>) {
    let workspace_root = params
        .workspace_folders
        .as_ref()
        .and_then(|folders| folders.first())
        .and_then(|folder| file_path_from_uri(&folder.uri))
        .or_else(|| {
            #[allow(deprecated)]
            params.root_uri.as_ref().and_then(file_path_from_uri)
        });
    let Some(workspace_root) = workspace_root else {
        return (None, None);
    };
    let Some(manifest_path) = discover_manifest(&workspace_root) else {
        return (None, None);
    };
    match ContentSetManifest::load(&manifest_path) {
        Ok(manifest) => {
            let content_root = manifest_path
                .parent()
                .map_or_else(|| workspace_root.clone(), Path::to_path_buf);
            (Some(manifest), Some(content_root))
        }
        Err(err) => {
            eprintln!(
                "babylon-ls: {} did not parse as content-sets.toml: {err}",
                manifest_path.display()
            );
            (None, None)
        }
    }
}

/// Runs the full server lifecycle over `connection` end to end and returns
/// the process exit code. `main.rs` is the only production caller; this
/// crate's own tests call it directly against
/// [`lsp_server::Connection::memory`], and `tests/protocol.rs` calls it
/// indirectly by spawning the real binary.
#[must_use]
pub fn serve(connection: &Connection) -> i32 {
    let (initialize_id, initialize_params) = match connection.initialize_start() {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("babylon-ls: initialize handshake failed: {err}");
            return EXIT_UNCLEAN;
        }
    };

    let params = match serde_json::from_value::<InitializeParams>(initialize_params) {
        Ok(params) => params,
        Err(err) => {
            eprintln!("babylon-ls: malformed InitializeParams: {err}");
            return EXIT_UNCLEAN;
        }
    };
    let (manifest, content_root) = discover_state(&params);
    let client_capabilities = params.capabilities;

    let result = serde_json::json!({ "capabilities": server_capabilities() });
    if let Err(err) = connection.initialize_finish(initialize_id, result) {
        eprintln!("babylon-ls: initialize handshake failed: {err}");
        return EXIT_UNCLEAN;
    }

    announce_watched_files(connection, &client_capabilities);

    let mut state = ServerState {
        store: DocumentStore::default(),
        manifest,
        content_root,
        result_ids: HashMap::new(),
    };
    main_loop(connection, &mut state)
}

/// What `initialized` does about `workspace/didChangeWatchedFiles` (§6.1):
/// dynamic registration when the client supports it, a loud
/// `window/logMessage` warning when it doesn't — never a silent fallback
/// to "just don't watch."
enum WatchAction {
    Register(RegistrationParams),
    Degrade(LogMessageParams),
}

fn watch_action_for(client_capabilities: &ClientCapabilities) -> WatchAction {
    let supports_dynamic = client_capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.did_change_watched_files.as_ref())
        .and_then(|caps| caps.dynamic_registration)
        .unwrap_or(false);

    if supports_dynamic {
        WatchAction::Register(watched_files_registration())
    } else {
        WatchAction::Degrade(LogMessageParams {
            typ: MessageType::WARNING,
            message: "babylon-ls: client does not support dynamic \
                workspace/didChangeWatchedFiles registration; on-disk edits \
                outside open documents are picked up only at pull time \
                (workspace/diagnostic), never pushed automatically"
                .to_owned(),
        })
    }
}

/// The `RegistrationParams` for `**/*.{bsl,bscn}` and
/// `**/content-sets.toml` (§6.1) — "the spec's own argument against
/// rolling our own watcher".
fn watched_files_registration() -> RegistrationParams {
    let watchers = vec![
        FileSystemWatcher {
            glob_pattern: GlobPattern::String("**/*.{bsl,bscn}".to_owned()),
            kind: None,
        },
        FileSystemWatcher {
            glob_pattern: GlobPattern::String("**/content-sets.toml".to_owned()),
            kind: None,
        },
    ];
    let options = DidChangeWatchedFilesRegistrationOptions { watchers };
    RegistrationParams {
        registrations: vec![Registration {
            id: WATCHED_FILES_REGISTRATION_ID.to_owned(),
            method: DidChangeWatchedFiles::METHOD.to_owned(),
            register_options: Some(
                serde_json::to_value(options).expect(
                    "babylon-ls: DidChangeWatchedFilesRegistrationOptions always serializes",
                ),
            ),
        }],
    }
}

fn announce_watched_files(connection: &Connection, client_capabilities: &ClientCapabilities) {
    match watch_action_for(client_capabilities) {
        WatchAction::Register(params) => {
            let request = RawRequest::new(
                RequestId::from(WATCHED_FILES_REGISTRATION_ID.to_owned()),
                RegisterCapability::METHOD.to_owned(),
                params,
            );
            // Best-effort: a send failure means the client pipe is already
            // gone, which the main loop's own channel-disconnect exit path
            // handles.
            let _ = connection.sender.send(request.into());
        }
        WatchAction::Degrade(log_params) => {
            let note = RawNotification::new(LogMessage::METHOD.to_owned(), log_params);
            let _ = connection.sender.send(note.into());
        }
    }
}

fn main_loop(connection: &Connection, state: &mut ServerState) -> i32 {
    // Power-of-10 rule 2 bound: this loop is bounded by the connection's
    // channel lifetime, not a count — it terminates on the `exit` protocol
    // return below, or when the client closes the channel and the iterator
    // ends. The event-loop analog of the rule's scheduler exemption; every
    // other loop in this crate carries a literal bound (MAX_FRAMES,
    // per-document line count).
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if let Some(exit_code) = handle_request(connection, state, &req) {
                    return exit_code;
                }
            }
            Message::Notification(note) => {
                if note.method.as_str() == Exit::METHOD {
                    // `exit` without a preceding `shutdown`: the spec
                    // mandates a non-zero exit code here.
                    return EXIT_UNCLEAN;
                }
                dispatch_notification(connection, state, &note);
            }
            Message::Response(_) => {
                // The only outbound request this crate sends
                // (`announce_watched_files`) has no follow-up action on
                // its result.
            }
        }
    }
    // The channel disconnected (the client closed the pipe) without a
    // clean shutdown/exit sequence — the same "no clean shutdown" exit
    // code as the bare-`exit` branch above.
    EXIT_UNCLEAN
}

/// Handles one request. Returns `Some(exit_code)` when the request ends
/// the server's lifecycle (`shutdown`), `None` otherwise.
fn handle_request(
    connection: &Connection,
    state: &mut ServerState,
    req: &RawRequest,
) -> Option<i32> {
    match connection.handle_shutdown(req) {
        Ok(true) => return Some(EXIT_CLEAN),
        Ok(false) => {}
        Err(err) => {
            eprintln!("babylon-ls: shutdown handshake failed: {err}");
            return Some(EXIT_UNCLEAN);
        }
    }
    match req.method.as_str() {
        m if m == DocumentDiagnosticRequest::METHOD => {
            handle_document_diagnostic(connection, state, req);
        }
        m if m == WorkspaceDiagnosticRequest::METHOD => {
            handle_workspace_diagnostic(connection, state, req);
        }
        m if m == Completion::METHOD => handle_completion(connection, state, req),
        m if m == HoverRequest::METHOD => handle_hover(connection, state, req),
        m if m == SignatureHelpRequest::METHOD => handle_signature_help(connection, state, req),
        m if m == SemanticTokensFullRequest::METHOD => {
            handle_semantic_tokens(connection, state, req);
        }
        _ => {
            let resp = Response::new_err(
                req.id.clone(),
                ErrorCode::MethodNotFound as i32,
                format!("babylon-ls: unhandled request method {}", req.method),
            );
            let _ = connection.sender.send(resp.into());
        }
    }
    None
}

fn source_for_authoring(
    state: &ServerState,
    uri: &Uri,
) -> Option<(String, crate::line_index::LineIndex)> {
    if let Some(document) = state.store.get(uri) {
        return Some((document.text.clone(), document.line_index.clone()));
    }
    let path = file_path_from_uri(uri)?;
    let text = std::fs::read_to_string(path).ok()?;
    let line_index = crate::line_index::LineIndex::new(&text);
    Some((text, line_index))
}

fn probability_snapshot(state: &ServerState, uri: &Uri) -> AuthoringSnapshot {
    let (Some(content_root), Some(manifest)) =
        (state.content_root.as_ref(), state.manifest.as_ref())
    else {
        return AuthoringSnapshot::default();
    };
    let Some(path) = content_relative_path(content_root, uri) else {
        return AuthoringSnapshot::default();
    };
    let reader = LiveSourceReader {
        content_root,
        store: &state.store,
    };
    analyze_probability_authoring(&path, manifest, &reader)
}

fn handle_completion(connection: &Connection, state: &ServerState, req: &RawRequest) {
    let params: CompletionParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let uri = params.text_document_position.text_document.uri;
    let result = source_for_authoring(state, &uri).and_then(|(text, line_index)| {
        let offset =
            line_index.position_to_offset(&text, params.text_document_position.position)?;
        let snapshot = probability_snapshot(state, &uri);
        Some(CompletionResponse::Array(completion_items(
            &text,
            &snapshot,
            usize::try_from(offset).unwrap_or(usize::MAX),
        )))
    });
    let response = Response::new_ok(req.id.clone(), result);
    let _ = connection.sender.send(response.into());
}

fn handle_hover(connection: &Connection, state: &ServerState, req: &RawRequest) {
    let params: HoverParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let uri = params.text_document_position_params.text_document.uri;
    let result = source_for_authoring(state, &uri).and_then(|(text, line_index)| {
        let offset =
            line_index.position_to_offset(&text, params.text_document_position_params.position)?;
        let snapshot = probability_snapshot(state, &uri);
        hover(
            &text,
            &line_index,
            &snapshot,
            usize::try_from(offset).unwrap_or(usize::MAX),
        )
    });
    let response = Response::new_ok(req.id.clone(), result);
    let _ = connection.sender.send(response.into());
}

fn handle_signature_help(connection: &Connection, state: &ServerState, req: &RawRequest) {
    let params: SignatureHelpParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let uri = params.text_document_position_params.text_document.uri;
    let result = source_for_authoring(state, &uri).and_then(|(text, line_index)| {
        let offset =
            line_index.position_to_offset(&text, params.text_document_position_params.position)?;
        let snapshot = probability_snapshot(state, &uri);
        signature_help(
            &text,
            &snapshot,
            usize::try_from(offset).unwrap_or(usize::MAX),
        )
    });
    let response = Response::new_ok(req.id.clone(), result);
    let _ = connection.sender.send(response.into());
}

fn handle_semantic_tokens(connection: &Connection, state: &ServerState, req: &RawRequest) {
    let params: SemanticTokensParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let uri = params.text_document.uri;
    let result = source_for_authoring(state, &uri).map(|(text, line_index)| {
        let snapshot = probability_snapshot(state, &uri);
        SemanticTokensResult::Tokens(semantic_tokens(&text, &line_index, &snapshot))
    });
    let response = Response::new_ok(req.id.clone(), result);
    let _ = connection.sender.send(response.into());
}

fn dispatch_notification(connection: &Connection, state: &mut ServerState, note: &RawNotification) {
    match note.method.as_str() {
        m if m == DidOpenTextDocument::METHOD => apply_did_open(connection, state, note),
        m if m == DidChangeTextDocument::METHOD => apply_did_change(connection, state, note),
        m if m == DidCloseTextDocument::METHOD => apply_did_close(state, note),
        _ => {} // Unhandled notifications are ignored per the LSP spec's own tolerance.
    }
}

fn apply_did_open(connection: &Connection, state: &mut ServerState, note: &RawNotification) {
    match serde_json::from_value::<DidOpenTextDocumentParams>(note.params.clone()) {
        Ok(params) => {
            let uri = params.text_document.uri;
            state.store.open(
                uri.clone(),
                params.text_document.version,
                params.text_document.text,
            );
            push_diagnostics_for(connection, state, &uri);
        }
        Err(err) => eprintln!("babylon-ls: malformed didOpen params: {err}"),
    }
}

fn apply_did_change(connection: &Connection, state: &mut ServerState, note: &RawNotification) {
    match serde_json::from_value::<DidChangeTextDocumentParams>(note.params.clone()) {
        Ok(mut params) => {
            // Full sync (§6.1): exactly one change event holding the
            // whole document text, never a range-based delta.
            if let Some(change) = params.content_changes.pop() {
                let uri = params.text_document.uri;
                let known =
                    state
                        .store
                        .change_full(&uri, params.text_document.version, change.text);
                if known {
                    push_diagnostics_for(connection, state, &uri);
                } else {
                    eprintln!(
                        "babylon-ls: didChange for a document never opened: {}",
                        uri.as_str()
                    );
                }
            }
        }
        Err(err) => eprintln!("babylon-ls: malformed didChange params: {err}"),
    }
}

fn apply_did_close(state: &mut ServerState, note: &RawNotification) {
    match serde_json::from_value::<DidCloseTextDocumentParams>(note.params.clone()) {
        Ok(params) => {
            let _ = state.store.close(&params.text_document.uri);
            state.result_ids.remove(&params.text_document.uri);
        }
        Err(err) => eprintln!("babylon-ls: malformed didClose params: {err}"),
    }
}

/// Compute `uri`'s current diagnostics (empty when the server has no
/// manifest/content-root, or `uri` names no content-root-relative path —
/// both are legitimate "nothing to report" states, not errors) and its
/// `resultId`, cache the id, and return `(diagnostics, resultId)` — the
/// one computation push (`push_diagnostics_for`) and pull (`handle_
/// document_diagnostic`/`handle_workspace_diagnostic`) both call through.
fn compute_diagnostics(state: &mut ServerState, uri: &Uri) -> (Vec<lsp_types::Diagnostic>, String) {
    let Some(content_root) = state.content_root.clone() else {
        return (Vec::new(), compute_result_id(&[], &[]));
    };
    let Some(manifest) = state.manifest.as_ref() else {
        return (Vec::new(), compute_result_id(&[], &[]));
    };
    let Some(path) = content_relative_path(&content_root, uri) else {
        return (Vec::new(), compute_result_id(&[], &[]));
    };
    let reader = LiveSourceReader {
        content_root: &content_root,
        store: &state.store,
    };
    let diagnostics = diagnose_bsl(uri, &path, manifest, &reader);
    // §6.1's own `resultId` definition: sha256 over the ordered (uri,
    // bytes) tuples of the SET plus the manifest bytes. Wave 1's
    // approximation covers the ONE file being diagnosed (not every
    // sibling in its content set) — a real inter-file-dependency change
    // elsewhere still gets picked up on ITS OWN didChange/pull, just not
    // as an automatic re-push of every file that depends on it; a
    // disclosed simplification, not the plan's literal per-set hash.
    let bytes = reader.read(&path).unwrap_or_default();
    let manifest_bytes = std::fs::read(content_root.join("content-sets.toml")).unwrap_or_default();
    let result_id = compute_result_id(&[(uri, bytes.as_bytes())], &manifest_bytes);
    (diagnostics, result_id)
}

/// Push `textDocument/publishDiagnostics` for `uri` — ALWAYS, even an
/// empty array, so "newly pushed diagnostics always replace" (§6.5): a
/// fix that clears every diagnostic still needs a push, or the client's
/// stale list never clears.
fn push_diagnostics_for(connection: &Connection, state: &mut ServerState, uri: &Uri) {
    let version = state.store.get(uri).map(|doc| doc.version);
    let (diagnostics, result_id) = compute_diagnostics(state, uri);
    state.result_ids.insert(uri.clone(), result_id);
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version,
    };
    let note = RawNotification::new(PublishDiagnostics::METHOD.to_owned(), params);
    let _ = connection.sender.send(note.into());
}

fn handle_document_diagnostic(connection: &Connection, state: &mut ServerState, req: &RawRequest) {
    let params: DocumentDiagnosticParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let uri = params.text_document.uri;
    let (diagnostics, result_id) = compute_diagnostics(state, &uri);
    state.result_ids.insert(uri.clone(), result_id.clone());
    let unchanged = params.previous_result_id.as_deref() == Some(result_id.as_str());
    let report: DocumentDiagnosticReportResult = if unchanged {
        DocumentDiagnosticReport::Unchanged(RelatedUnchangedDocumentDiagnosticReport {
            related_documents: None,
            unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport { result_id },
        })
        .into()
    } else {
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            related_documents: None,
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                result_id: Some(result_id),
                items: diagnostics,
            },
        })
        .into()
    };
    let response = Response::new_ok(req.id.clone(), report);
    let _ = connection.sender.send(response.into());
}

/// Every `.bsl`/`.bscn` the manifest names, plus every open document not
/// already covered by it (§6.5) — a `BTreeSet` (not a `HashMap`/`HashSet`
/// iteration order feeding output, global constraint 2) so the report's
/// own item order is deterministic.
fn workspace_diagnostic_paths(
    state: &ServerState,
    content_root: &Path,
) -> std::collections::BTreeSet<String> {
    let mut paths = std::collections::BTreeSet::new();
    if let Some(manifest) = state.manifest.as_ref() {
        for set in &manifest.sets {
            paths.insert(set.scenario.clone());
            paths.extend(set.prelude.iter().cloned());
            paths.extend(set.rules.iter().cloned());
        }
    }
    for uri in state.store.open_uris() {
        if let Some(path) = content_relative_path(content_root, &uri) {
            paths.insert(path);
        }
    }
    paths
}

fn handle_workspace_diagnostic(connection: &Connection, state: &mut ServerState, req: &RawRequest) {
    let params: WorkspaceDiagnosticParams = match serde_json::from_value(req.params.clone()) {
        Ok(params) => params,
        Err(err) => {
            respond_invalid_params(connection, req, &err.to_string());
            return;
        }
    };
    let Some(content_root) = state.content_root.clone() else {
        let response = Response::new_ok(
            req.id.clone(),
            WorkspaceDiagnosticReportResult::Report(WorkspaceDiagnosticReport {
                items: Vec::new(),
            }),
        );
        let _ = connection.sender.send(response.into());
        return;
    };
    // `Uri`'s interior `Cell` is parse-laziness bookkeeping, not key
    // semantics: lsp-types defines `Eq`/`Hash` over `as_str()`, so it is a
    // sound, stable map key despite the lint's interior-mutability heuristics.
    #[allow(clippy::mutable_key_type)]
    let previous: HashMap<Uri, String> = params
        .previous_result_ids
        .into_iter()
        .map(|p| (p.uri, p.value))
        .collect();
    // Bounded by the manifest's own row count plus open-document count —
    // both finite, read once per call (Power-of-10 rule 2).
    let paths = workspace_diagnostic_paths(state, &content_root);
    let mut items = Vec::with_capacity(paths.len());
    for path in paths {
        let Some(uri) = uri_from_file_path(&content_root.join(&path)) else {
            continue;
        };
        let (diagnostics, result_id) = compute_diagnostics(state, &uri);
        state.result_ids.insert(uri.clone(), result_id.clone());
        let version = state.store.get(&uri).map(|doc| i64::from(doc.version));
        let unchanged = previous.get(&uri) == Some(&result_id);
        let item = if unchanged {
            WorkspaceDocumentDiagnosticReport::Unchanged(
                WorkspaceUnchangedDocumentDiagnosticReport {
                    uri,
                    version,
                    unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                        result_id,
                    },
                },
            )
        } else {
            WorkspaceDocumentDiagnosticReport::Full(WorkspaceFullDocumentDiagnosticReport {
                uri,
                version,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some(result_id),
                    items: diagnostics,
                },
            })
        };
        items.push(item);
    }
    let response = Response::new_ok(
        req.id.clone(),
        WorkspaceDiagnosticReportResult::Report(WorkspaceDiagnosticReport { items }),
    );
    let _ = connection.sender.send(response.into());
}

fn respond_invalid_params(connection: &Connection, req: &RawRequest, detail: &str) {
    let resp = Response::new_err(
        req.id.clone(),
        ErrorCode::InvalidParams as i32,
        format!("babylon-ls: malformed params for {}: {detail}", req.method),
    );
    let _ = connection.sender.send(resp.into());
}

#[cfg(test)]
mod tests {
    use super::{ServerState, EXIT_CLEAN, EXIT_UNCLEAN};
    use crate::content_manifest::ContentSetManifest;
    use crate::document_store::DocumentStore;
    use lsp_server::{
        Connection, ErrorCode, Message, Notification as RawNotification, Request as RawRequest,
        RequestId,
    };
    use lsp_types::notification::{Exit, Initialized, LogMessage, Notification as _};
    use lsp_types::request::{
        Completion, HoverRequest, Initialize, RegisterCapability, Request as _,
        SemanticTokensFullRequest, Shutdown, SignatureHelpRequest,
    };
    use lsp_types::{
        ClientCapabilities, DidChangeWatchedFilesClientCapabilities, InitializeParams,
        InitializedParams, Uri, WorkspaceClientCapabilities,
    };
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::Duration;

    const TIMEOUT: Duration = Duration::from_secs(5);

    fn init_params(dynamic_watch: Option<bool>) -> InitializeParams {
        let mut capabilities = ClientCapabilities::default();
        if let Some(dynamic) = dynamic_watch {
            capabilities.workspace = Some(WorkspaceClientCapabilities {
                did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
                    dynamic_registration: Some(dynamic),
                    relative_pattern_support: None,
                }),
                ..Default::default()
            });
        }
        InitializeParams {
            capabilities,
            ..Default::default()
        }
    }

    /// Drives `initialize` -> `initialized` and, when `dynamic_watch` is
    /// `false`/`None`, drains the resulting `window/logMessage`
    /// degradation warning. Returns the message that follows `initialized`
    /// so a test can assert on it (the `client/registerCapability`
    /// request when `dynamic_watch` is `Some(true)`).
    fn handshake(client: &Connection, dynamic_watch: Option<bool>) -> Message {
        client
            .sender
            .send(
                RawRequest::new(
                    RequestId::from(1),
                    Initialize::METHOD.to_owned(),
                    init_params(dynamic_watch),
                )
                .into(),
            )
            .expect("send initialize");
        client
            .receiver
            .recv_timeout(TIMEOUT)
            .expect("initialize response");

        client
            .sender
            .send(RawNotification::new(Initialized::METHOD.to_owned(), InitializedParams {}).into())
            .expect("send initialized");

        client
            .receiver
            .recv_timeout(TIMEOUT)
            .expect("post-initialized message")
    }

    fn clean_shutdown(client: &Connection) {
        client
            .sender
            .send(RawRequest::new(RequestId::from(2), Shutdown::METHOD.to_owned(), ()).into())
            .expect("send shutdown");
        client
            .receiver
            .recv_timeout(TIMEOUT)
            .expect("shutdown response");
        client
            .sender
            .send(RawNotification::new(Exit::METHOD.to_owned(), ()).into())
            .expect("send exit");
    }

    fn probability_authoring_state() -> (ServerState, Uri, String) {
        let manifest = ContentSetManifest::parse(
            Path::new("content-sets.toml"),
            r#"
schema = 2
[[kernel_slot]]
ordinal = 0
rule = "vitality/probe"
sample = "struggle/spark"
slot = 0
[[set]]
id = "probe/probability"
scenario = "scenario.bscn"
prelude = []
rules = ["rules/probe.bsl"]
consumers = []
note = "LSP request fixture"
"#,
        )
        .expect("valid manifest");
        let scenario_uri = "file:///virtual/scenario.bscn"
            .parse::<Uri>()
            .expect("valid scenario URI");
        let rule_uri = "file:///virtual/rules/probe.bsl"
            .parse::<Uri>()
            .expect("valid rule URI");
        let source = "(rule vitality/probe :role mechanic :evidence designed \
            :material-basis \"bounded spark\" :fuel 64 \
            (domain NodeType/SOCIAL_CLASS) \
            (bindings (binding current :field social-class/value)) (effects \
            (choose :sample struggle/spark :slot 0 \
              (branch SparkOutcome/YES :mass 1m (effects)) \
              (branch SparkOutcome/NO :mass 3m (effects)))))"
            .to_owned();
        let mut store = DocumentStore::default();
        store.open(
            scenario_uri,
            1,
            "(scenario ft/probe \
                (defvocabulary NodeType (SOCIAL_CLASS)) \
                (defenum SparkOutcome (YES NO)) \
                (deffield social-class/value int extensive) \
                (node worker NodeType/SOCIAL_CLASS (social-class/value 0)))"
                .to_owned(),
        );
        store.open(rule_uri.clone(), 1, source.clone());
        (
            ServerState {
                store,
                manifest: Some(manifest),
                content_root: Some(PathBuf::from("/virtual")),
                result_ids: HashMap::new(),
            },
            rule_uri,
            source,
        )
    }

    fn response_value(client: &Connection) -> serde_json::Value {
        let message = client
            .receiver
            .recv_timeout(TIMEOUT)
            .expect("authoring response");
        let Message::Response(response) = message else {
            panic!("expected response")
        };
        response.response_result.expect("successful response")
    }

    #[test]
    fn probability_authoring_requests_dispatch_loader_owned_results() {
        let (server, client) = Connection::memory();
        let (mut state, uri, source) = probability_authoring_state();
        let choose = u32::try_from(source.find("choose").expect("choose token") + 1)
            .expect("fixture offset fits u32");
        let branch_position = u32::try_from(
            source
                .find("(branch")
                .expect("first branch")
                .saturating_sub(1),
        )
        .expect("fixture offset fits u32");

        let completion = RawRequest::new(
            RequestId::from(10),
            Completion::METHOD.to_owned(),
            serde_json::json!({
                "textDocument": {"uri": uri.clone()},
                "position": {"line": 0, "character": branch_position}
            }),
        );
        assert!(super::handle_request(&server, &mut state, &completion).is_none());
        let completion_value = response_value(&client);
        assert!(completion_value
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["label"] == "branch")));

        let hover_request = RawRequest::new(
            RequestId::from(11),
            HoverRequest::METHOD.to_owned(),
            serde_json::json!({
                "textDocument": {"uri": uri.clone()},
                "position": {"line": 0, "character": choose}
            }),
        );
        assert!(super::handle_request(&server, &mut state, &hover_request).is_none());
        let hover_value = response_value(&client);
        assert!(hover_value["contents"]["value"]
            .as_str()
            .is_some_and(|value| value.contains("4611686018427387904")));

        let signature = RawRequest::new(
            RequestId::from(12),
            SignatureHelpRequest::METHOD.to_owned(),
            serde_json::json!({
                "textDocument": {"uri": uri.clone()},
                "position": {"line": 0, "character": choose}
            }),
        );
        assert!(super::handle_request(&server, &mut state, &signature).is_none());
        let signature_value = response_value(&client);
        assert!(signature_value["signatures"][0]["label"]
            .as_str()
            .is_some_and(|label| label.starts_with("(choose")));

        let tokens = RawRequest::new(
            RequestId::from(13),
            SemanticTokensFullRequest::METHOD.to_owned(),
            serde_json::json!({"textDocument": {"uri": uri}}),
        );
        assert!(super::handle_request(&server, &mut state, &tokens).is_none());
        let token_value = response_value(&client);
        assert!(token_value["data"]
            .as_array()
            .is_some_and(|data| !data.is_empty()));
    }

    #[test]
    fn full_lifecycle_clean_shutdown_exits_zero() {
        let (server, client) = Connection::memory();
        let handle = thread::spawn(move || super::serve(&server));

        let degrade = handshake(&client, None);
        assert!(matches!(degrade, Message::Notification(n) if n.method == LogMessage::METHOD));

        clean_shutdown(&client);

        assert_eq!(handle.join().expect("serve thread"), EXIT_CLEAN);
    }

    #[test]
    fn exit_without_shutdown_exits_uncleanly() {
        let (server, client) = Connection::memory();
        let handle = thread::spawn(move || super::serve(&server));

        handshake(&client, None);
        client
            .sender
            .send(RawNotification::new(Exit::METHOD.to_owned(), ()).into())
            .expect("send exit");

        assert_eq!(handle.join().expect("serve thread"), EXIT_UNCLEAN);
    }

    #[test]
    fn dynamic_registration_true_sends_register_capability_request() {
        let (server, client) = Connection::memory();
        let handle = thread::spawn(move || super::serve(&server));

        let registration = handshake(&client, Some(true));
        match registration {
            Message::Request(req) => assert_eq!(req.method, RegisterCapability::METHOD),
            other => panic!("expected client/registerCapability request, got {other:?}"),
        }

        clean_shutdown(&client);
        assert_eq!(handle.join().expect("serve thread"), EXIT_CLEAN);
    }

    #[test]
    fn dynamic_registration_false_degrades_loudly() {
        let (server, client) = Connection::memory();
        let handle = thread::spawn(move || super::serve(&server));

        let degrade = handshake(&client, Some(false));
        match degrade {
            Message::Notification(n) => assert_eq!(n.method, LogMessage::METHOD),
            other => panic!("expected window/logMessage, got {other:?}"),
        }

        clean_shutdown(&client);
        assert_eq!(handle.join().expect("serve thread"), EXIT_CLEAN);
    }

    #[test]
    fn request_before_initialize_gets_server_not_initialized() {
        let (server, client) = Connection::memory();
        let handle = thread::spawn(move || super::serve(&server));

        client
            .sender
            .send(RawRequest::new(RequestId::from(99), Shutdown::METHOD.to_owned(), ()).into())
            .expect("send premature shutdown");
        let msg = client
            .receiver
            .recv_timeout(TIMEOUT)
            .expect("error response");
        match msg {
            Message::Response(resp) => {
                let err = resp
                    .response_result
                    .expect_err("expected an error response before initialize");
                assert_eq!(err.code, ErrorCode::ServerNotInitialized as i32);
            }
            other => panic!("expected an error Response, got {other:?}"),
        }

        // `initialize_start` keeps looping past the rejected request —
        // walk it through a real handshake so the spawned thread
        // terminates cleanly.
        handshake(&client, None);
        clean_shutdown(&client);
        assert_eq!(handle.join().expect("serve thread"), EXIT_CLEAN);
    }
}
