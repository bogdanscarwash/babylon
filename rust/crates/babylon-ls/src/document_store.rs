//! The in-memory, [`Uri`]-keyed document store (Task 5.3): the ONE place
//! `babylon-ls` holds text a client has open, refreshed wholesale on every
//! Full-sync `didChange` and dropped on `didClose`. Nothing here persists
//! past the process's lifetime — global constraint 1 (observes-only): the
//! server holds no state the loader does not, and the loader holds none of
//! this at all (it reads a file path, not an editor buffer).

use std::collections::HashMap;

use lsp_types::Uri;

use crate::line_index::LineIndex;

/// One open document's text, version, and pre-built [`LineIndex`] — built
/// once, here, alongside the text (Task 5.3's own phrasing), never
/// recomputed piecemeal on a later read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub text: String,
    pub version: i32,
    pub line_index: LineIndex,
}

/// The document store. `HashMap` iteration order never feeds any output
/// this crate produces (global constraint 2) — every access here is a
/// point lookup by a specific [`Uri`], never an iteration over the map.
#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<Uri, Document>,
}

impl DocumentStore {
    /// `textDocument/didOpen`: inserts a fresh [`Document`], building its
    /// [`LineIndex`] once, here, from the just-received text.
    pub fn open(&mut self, uri: Uri, version: i32, text: String) {
        let line_index = LineIndex::new(&text);
        self.documents.insert(
            uri,
            Document {
                text,
                version,
                line_index,
            },
        );
    }

    /// `textDocument/didChange` under Full sync (§6.1): replaces the whole
    /// document — text, version, and line index together — never a patch.
    /// Returns `false` if `uri` was never opened (a client protocol
    /// violation; the caller decides how loud to be about it).
    #[must_use]
    pub fn change_full(&mut self, uri: &Uri, version: i32, text: String) -> bool {
        let Some(document) = self.documents.get_mut(uri) else {
            return false;
        };
        let line_index = LineIndex::new(&text);
        *document = Document {
            text,
            version,
            line_index,
        };
        true
    }

    /// `textDocument/didClose`: drops the document. Returns it so a caller
    /// CAN inspect the final state if it ever needs to (today, nobody
    /// does — `lifecycle::apply_did_close` discards it explicitly).
    #[must_use]
    pub fn close(&mut self, uri: &Uri) -> Option<Document> {
        self.documents.remove(uri)
    }

    /// Point lookup — the only read this crate's dispatch loop performs.
    #[must_use]
    pub fn get(&self, uri: &Uri) -> Option<&Document> {
        self.documents.get(uri)
    }

    /// Every currently-open document's URI (Task 6, #652: `workspace/
    /// diagnostic`'s own "every open document" clause, §6.5). Iteration
    /// order over the underlying `HashMap` never feeds any output this
    /// crate produces on its own — `crate::lifecycle::workspace_diagnostic_paths`
    /// (private to that module) collects the result into a `BTreeSet`
    /// before using it.
    pub fn open_uris(&self) -> impl Iterator<Item = lsp_types::Uri> + '_ {
        self.documents.keys().cloned()
    }

    /// How many documents are currently open (test/diagnostic helper).
    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::DocumentStore;
    use lsp_types::Uri;

    fn uri(s: &str) -> Uri {
        s.parse::<Uri>().expect("valid test URI")
    }

    #[test]
    fn open_then_get_roundtrips_text() {
        let mut store = DocumentStore::default();
        store.open(uri("file:///a.bsl"), 1, "(rule)".to_owned());
        let doc = store.get(&uri("file:///a.bsl")).expect("document present");
        assert_eq!(doc.text, "(rule)");
        assert_eq!(doc.version, 1);
    }

    #[test]
    fn change_full_replaces_text_and_rebuilds_line_index() {
        let mut store = DocumentStore::default();
        store.open(uri("file:///a.bsl"), 1, "one".to_owned());
        assert!(store.change_full(&uri("file:///a.bsl"), 2, "one\ntwo".to_owned()));
        let doc = store.get(&uri("file:///a.bsl")).expect("document present");
        assert_eq!(doc.text, "one\ntwo");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn change_full_on_unopened_document_reports_false() {
        let mut store = DocumentStore::default();
        assert!(!store.change_full(&uri("file:///missing.bsl"), 1, "x".to_owned()));
        assert!(store.is_empty());
    }

    #[test]
    fn close_removes_the_document() {
        let mut store = DocumentStore::default();
        store.open(uri("file:///a.bsl"), 1, "x".to_owned());
        assert_eq!(store.len(), 1);
        assert!(store.close(&uri("file:///a.bsl")).is_some());
        assert!(store.get(&uri("file:///a.bsl")).is_none());
        assert!(store.is_empty());
    }

    #[test]
    fn close_on_unopened_document_returns_none() {
        let mut store = DocumentStore::default();
        assert!(store.close(&uri("file:///missing.bsl")).is_none());
    }
}
