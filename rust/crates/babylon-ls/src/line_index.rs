//! Byte-offset <-> LSP [`Position`] (UTF-16 line/character) conversion,
//! built once per document text (Task 5.3) so every later consumer —
//! Task 6's diagnostics mapping, in particular — works from ONE
//! already-computed index instead of re-scanning the document per
//! diagnostic.
//!
//! LSP's default position encoding is UTF-16 (the spec digest §2; this
//! crate's [`crate::capabilities::server_capabilities`] omits
//! `positionEncoding`, which is exactly how a server opts into that
//! default rather than negotiating `utf-8`/`utf-32`). `babylon-bsl`'s own
//! span tracking is byte-offset native (the span side-table, issue #652
//! Task 1) — this module is the ONE seam where the two disagree, so that
//! disagreement gets resolved in one place, not at every diagnostic call
//! site.

use lsp_types::Position;

/// A line index over one document's text at the moment it was built.
/// Immutable — a `didChange` under Full sync (§6.1) replaces the whole
/// [`LineIndex`] alongside the whole text; it is never patched in place.
/// Keeping it patched against Full sync's whole-document replacement would
/// reintroduce exactly the incremental position-mapping bug class §6.1
/// documents Full sync as deleting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    /// Byte offset of the first byte of each line. `line_starts[0] == 0`
    /// always; length equals the document's line count (an unterminated
    /// final line still counts as one).
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Scans `text` once for line boundaries. The loop is bounded by
    /// `text.len()` (Power-of-10 rule 2: a `for` loop over a finite byte
    /// slice, not an unbounded wait).
    ///
    /// # Panics
    ///
    /// Panics if `text` is larger than `u32::MAX` bytes — a genuinely
    /// unsupported document size (the whole content estate is ~440 KB
    /// total), not a recoverable error this crate's callers can act on.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0u32];
        for (offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                let next_line_start = u32::try_from(offset + 1)
                    .expect("babylon-ls: document exceeds 4 GiB, unsupported");
                line_starts.push(next_line_start);
            }
        }
        Self { line_starts }
    }

    /// The zero-based line index containing `byte_offset`, clamped to the
    /// document's last line if `byte_offset` runs past the end of text.
    fn line_of(&self, byte_offset: u32) -> usize {
        match self.line_starts.binary_search(&byte_offset) {
            Ok(exact) => exact,
            Err(insertion) => insertion.saturating_sub(1),
        }
    }

    /// Converts a byte offset into `text` to an LSP [`Position`].
    ///
    /// # Panics
    ///
    /// `byte_offset` must land on a UTF-8 character boundary of `text`,
    /// and must not exceed `text.len()` — the same invariant every caller
    /// in this crate already holds, because every byte offset this module
    /// ever receives comes from `babylon-bsl`'s own span tracking over the
    /// SAME text, never from an untrusted external source.
    #[must_use]
    pub fn offset_to_position(&self, text: &str, byte_offset: u32) -> Position {
        let line = self.line_of(byte_offset);
        let line_start = self.line_starts[line];
        let column_bytes = &text[line_start as usize..byte_offset as usize];
        let character = u32::try_from(column_bytes.encode_utf16().count())
            .expect("babylon-ls: line exceeds 4G UTF-16 units, unsupported");
        let line = u32::try_from(line).expect("babylon-ls: document exceeds 4G lines, unsupported");
        Position { line, character }
    }

    /// Converts an LSP [`Position`] back to a byte offset into `text`.
    /// Returns `None` when `position` names a line or a UTF-16 column past
    /// what `text` actually holds, or lands mid-surrogate-pair — a
    /// client-side coordinate `babylon-ls` never manufactures locally, so
    /// a caller reports it loudly rather than clamping it into something
    /// that looks valid.
    #[must_use]
    pub fn position_to_offset(&self, text: &str, position: Position) -> Option<u32> {
        let line = usize::try_from(position.line).ok()?;
        let line_start = *self.line_starts.get(line)?;
        let line_end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or_else(|| u32::try_from(text.len()).unwrap_or(u32::MAX));
        let line_text = text.get(line_start as usize..line_end as usize)?;

        let mut remaining = position.character;
        let mut byte_offset = line_start;
        for ch in line_text.chars() {
            if remaining == 0 {
                return Some(byte_offset);
            }
            let units = u32::try_from(ch.len_utf16()).unwrap_or(1);
            if units > remaining {
                // The position lands mid-surrogate-pair — not a boundary
                // any real client sends; refuse rather than guess.
                return None;
            }
            remaining -= units;
            byte_offset += u32::try_from(ch.len_utf8()).unwrap_or(0);
        }
        if remaining == 0 {
            Some(byte_offset)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;
    use lsp_types::Position;

    #[test]
    fn ascii_single_line() {
        let text = "hello";
        let index = LineIndex::new(text);
        assert_eq!(index.offset_to_position(text, 0), Position::new(0, 0));
        assert_eq!(index.offset_to_position(text, 5), Position::new(0, 5));
        assert_eq!(index.position_to_offset(text, Position::new(0, 3)), Some(3));
    }

    #[test]
    fn ascii_multi_line_lf() {
        let text = "one\ntwo\nthree";
        let index = LineIndex::new(text);
        // Byte 4 is the 't' starting "two" — the first byte of line 1.
        assert_eq!(index.offset_to_position(text, 4), Position::new(1, 0));
        // Byte 8 is the 't' starting "three" — the first byte of line 2.
        assert_eq!(index.offset_to_position(text, 8), Position::new(2, 0));
        assert_eq!(index.position_to_offset(text, Position::new(1, 2)), Some(6));
    }

    #[test]
    fn crlf_line_endings() {
        let text = "one\r\ntwo\r\nthree";
        let index = LineIndex::new(text);
        // '\r' stays part of line 0's content (never stripped): line 1
        // starts right after the '\n', at byte 5.
        assert_eq!(index.offset_to_position(text, 5), Position::new(1, 0));
        // Position (0, 3) is the '\r' itself — one UTF-16 unit past "one".
        let offset = index
            .position_to_offset(text, Position::new(0, 3))
            .expect("position within line 0");
        let offset = offset as usize;
        assert_eq!(&text[offset..=offset], "\r");
    }

    #[test]
    fn multi_byte_utf8_bmp() {
        // "café" — 'é' is 2 bytes in UTF-8 but 1 UTF-16 code unit.
        let text = "café";
        let index = LineIndex::new(text);
        assert_eq!(text.len(), 5); // c-a-f-é(2 bytes)
        assert_eq!(index.offset_to_position(text, 5), Position::new(0, 4));
        assert_eq!(index.position_to_offset(text, Position::new(0, 4)), Some(5));
    }

    #[test]
    fn multi_byte_utf8_astral_surrogate_pair() {
        // U+1F600 is 4 UTF-8 bytes and a UTF-16 SURROGATE PAIR (2 code
        // units) — the case the BMP fixture above cannot exercise.
        let text = "a\u{1F600}b";
        let index = LineIndex::new(text);
        let emoji_end = u32::try_from(1 + '\u{1F600}'.len_utf8()).unwrap();
        // 1 (a) + 2 (surrogate pair) == 3 UTF-16 units before 'b'.
        assert_eq!(
            index.offset_to_position(text, emoji_end),
            Position::new(0, 3)
        );
        assert_eq!(
            index.position_to_offset(text, Position::new(0, 3)),
            Some(emoji_end)
        );
        // Landing mid-surrogate-pair (character 2) is refused, not guessed.
        assert_eq!(index.position_to_offset(text, Position::new(0, 2)), None);
    }

    #[test]
    fn bom_prefixed_file() {
        let text = "\u{FEFF}hello";
        let index = LineIndex::new(text);
        let end = u32::try_from(text.len()).unwrap();
        // The BOM counts as one ordinary UTF-16 code unit at position 0 —
        // never stripped or special-cased.
        assert_eq!(index.offset_to_position(text, end), Position::new(0, 6));
        assert_eq!(
            index.position_to_offset(text, Position::new(0, 6)),
            Some(end)
        );
    }

    #[test]
    fn position_past_end_of_document_is_refused() {
        let text = "hi";
        let index = LineIndex::new(text);
        assert_eq!(index.position_to_offset(text, Position::new(0, 99)), None);
        assert_eq!(index.position_to_offset(text, Position::new(5, 0)), None);
    }
}
