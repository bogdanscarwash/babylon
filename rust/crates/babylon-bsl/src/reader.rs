//! The BSL reader: lexer + s-expression parser for the normative lexical
//! grammar (`docs/reference/bsl-language.rst` §1). Hand-written, with an
//! ITERATIVE parser (explicit list stack, no recursion). It refuses nesting
//! above a fixed safety ceiling before constructing a recursively owned tree,
//! so parsing and destruction cannot turn hostile nesting into a call-stack
//! overflow. The grammar is small and a hand-rolled parser keeps the
//! dependency graph minimal (no parser-combinator crate).
//!
//! **Deviation from the Phase 1 plan's sketch, recorded:** the plan drafted
//! `Atom(String)` with classification deferred. The language reference's §1
//! is normative ("one normative home per topic", PR #363) and defines
//! tokenization as *maximal munch then classify*, with lexical error codes
//! `E-LEX-001..026` — so `Atom` here carries the **classified** token, and
//! every lexical code has an accepting and a rejecting vector in the test
//! module (the conformance-suite discipline, §Conformance). The typechecker
//! (Task 10) receives pre-classified atoms, which is exactly the static-type
//! assignment §1.5 already makes (`int-lit` : `Int`, etc.).
//!
//! Two spec-code readings this module commits to (both documented at the
//! enforcement site): a literal-out-of-representable-range error beyond the
//! `int-lit` case reuses `E-LEX-020`, and a lexically negative currency
//! literal (minus sign present, any value — including `-0$`) is `E-LEX-022`
//! because §1.5 names the *literal*, not the value, negative.

use babylon_kernel::Currency;

/// Maximum simultaneously open lists accepted by the reader.
///
/// Semantic walkers use a lower 256-level contract. This higher boundary is
/// the construction and destruction safety ceiling for the recursively owned
/// [`SExpr`] representation.
pub const MAX_READER_NESTING_DEPTH: usize = 512;

/// One s-expression: a classified atom or a parenthesised list.
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    /// A single classified token.
    Atom(Atom),
    /// A parenthesised form.
    List(Vec<SExpr>),
}

/// A classified BSL atom (§1.4–§1.5). Classification happens at lex time —
/// there is no "raw atom" escape hatch.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// `symbol` — lowercase kebab-case, max 64 chars.
    Symbol(String),
    /// `qname` — slash-joined symbols, e.g. `vitality/starvation-mortality`.
    QName(String),
    /// `keyword` — `:name`, stored WITHOUT the leading colon.
    Keyword(String),
    /// `enum-ref` — `NodeType/SOCIAL_CLASS`: the member is the enum member
    /// identifier, never its serialized value. Registry membership is a
    /// load-time check (`E-LOAD-030/031`), not the reader's.
    EnumRef {
        /// The closed enum's type name, e.g. `NodeType`.
        enum_type: String,
        /// The member identifier, e.g. `SOCIAL_CLASS`.
        member: String,
    },
    /// A bare uppercase-initial run with **no** `/` — the lexical UNION of
    /// §1.4's `<enum-type>` (`UPPER (UPPER|LOWER|DIGIT)*`) and
    /// `<enum-member>` (`UPPER (UPPER|DIGIT|"_")*`) charsets:
    /// `UPPER (UPPER|LOWER|DIGIT|"_")*`. §1.4's `<enum-type>` production
    /// already existed as the LHS half of `<enum-ref>` (`enum-ref ::=
    /// enum-type "/" enum-member`), but no atom class carried it standing
    /// alone — every position that named a type needed a member alongside
    /// it. §2.13 (the Organization contract's Q12 ruling) introduces
    /// positions that do not: `defenum`/`defvocabulary`'s own type-name
    /// operand, `deffield`'s `:enum-type` keyword operand (§2.9), and —
    /// per §2.13's own EBNF (`<defenum> ::= "(" "defenum" <enum-type> "("
    /// <enum-member>+ ")" ")"`) — `defenum`/`defvocabulary`'s MEMBER LIST
    /// itself: the list holds bare `<enum-member>` atoms
    /// (`STATE_APPARATUS`), never full `<enum-type>/<enum-member>` pairs.
    ///
    /// **#528 fix round, corrected reading.** An earlier version of this
    /// doc read the member list as full enum-refs, reasoning that
    /// `<enum-type>`'s and `<enum-member>`'s overlapping charsets
    /// (`BUSINESS` fits both) made a standalone bare-member class
    /// "lexically ambiguous" with this one — that reasoning was the bug:
    /// neither production contains the other (`OrgKind` has lowercase,
    /// `STATE_APPARATUS` has `_`), so admitting their UNION at lex time and
    /// disambiguating POSITIONALLY, at the parser, is unambiguous and is
    /// what the tree-sitter grammar (`tools/tree-sitter-bsl/grammar.js`'s
    /// own `enum_type`/`enum_member` split) and its corpus
    /// (`test/corpus/declarations.txt:144-145`) already assumed. This
    /// variant (renamed from `EnumTypeName`, which no longer describes
    /// what it carries) is that union; [`is_enum_type_shape`] and
    /// [`is_enum_member_shape`], both in this module, are the two
    /// narrower positional checks — `defenum`/`defvocabulary`'s type-name
    /// operand against the former, their member-list items against the
    /// latter — a full enum-ref written where a bare member belongs is
    /// grammar-nonconforming and refuses loudly (see `crate::declarations::
    /// parse_defenum` and `crate::scenario::load_defvocabulary`'s own
    /// docs for the concrete written forms this resolves to). `deffield`'s
    /// `:enum-type` operand needs no separate shape check: it is a
    /// REFERENCE resolved through `crate::types::EnumRegistry::resolve`,
    /// which only ever holds names `parse_defenum` already shape-checked.
    BareUpperIdent(String),
    /// `bool-lit` — `#t` / `#f`. `true`/`false` are ordinary symbols.
    Bool(bool),
    /// One of the ten operator tokens `< <= > >= = != + - * /` — the §2
    /// grammar's quoted terminals and §5.2's form tags. **Spec repair,
    /// recorded:** §1.4's atom-class table omits these, yet §2 requires
    /// `(< a b)` and the §5.6 worked example uses `<` — without this class
    /// the reader rejects the spec's own example. Lexed by exact match
    /// against the closed set (maximal munch still applies: `<x` is
    /// `E-LEX-003`). Valid only in form-head position; CAS encodes them as
    /// form tags, never as atoms.
    Operator(String),
    /// `int-lit` — must fit `i64` (`E-LEX-020`).
    Int(i64),
    /// A `$`-suffixed scaled literal, canonicalized to integer micro-units.
    Currency(Currency),
    /// An exact unsigned `m` literal, canonicalized to nanounits.
    /// `Mass` is transient finite-kernel allocation input, never a field.
    Mass(crate::probability::Mass),
    /// A `p`/`i`/`c`-suffixed scaled literal in canonical minimal-scale form.
    Scaled(ScaledLit),
    /// `string` — after escape processing; NFC-checked (`E-LEX-002`).
    Str(String),
}

/// The unit-interval scaled-literal kinds (`p` / `i` / `c` suffixes), plus
/// the `r`-suffixed `Ratio` kind (§1.5 addendum, Director ruling 2026-08-11
/// #492/ADR194): unlike `p`/`i`/`c`, `Ratio` is NOT unit-interval — its
/// domain is `(0, ∞)`, the kernel's existing `babylon_kernel::Ratio` sort.
/// It shares this struct's canonical decimal representation (and therefore
/// this reader's canonicalization and CAS encoding machinery) rather than
/// inventing a parallel literal shape, which is the whole point: scalar
/// multiplication by a declared-domain constant is machinery reusing an
/// existing closed-algebra sort, not new mathematics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaledKind {
    /// `p` suffix.
    Probability,
    /// `i` suffix.
    Intensity,
    /// `c` suffix.
    Coefficient,
    /// `r` suffix — `Ratio`, domain `(0, ∞)` (§1.5 addendum).
    Ratio,
}

/// A canonicalized `p`/`i`/`c`/`r` literal: value = `unscaled / 10^scale`,
/// trailing fractional zeros stripped, zero as `(0, 0)` (§1.5 *Decimal
/// canonicalization* — `0.50c` and `0.5c` are ONE value and hash identically).
/// `r` (`Ratio`) is excepted from the "zero as `(0,0)`" case: its domain
/// excludes zero, so a canonicalized `Ratio` literal is never `(0, 0)`
/// (`classify_ratio` rejects the input before canonicalization reaches it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledLit {
    /// Which unit-interval kind the suffix named — or `Ratio` (§1.5 addendum).
    pub kind: ScaledKind,
    /// The canonical unscaled integer.
    pub unscaled: i128,
    /// The canonical scale (fractional digit count), max 9.
    pub scale: u8,
}

/// The spec's lexical error codes (`E-LEX-0xx`), one variant per code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexCode {
    /// `E-LEX-001` — invalid UTF-8, or a BOM anywhere but offset 0.
    InvalidUtf8OrBom,
    /// `E-LEX-002` — a string literal not in Unicode NFC.
    NonNfcString,
    /// `E-LEX-003` — a token run matching no atom class.
    UnclassifiableToken,
    /// `E-LEX-010` — symbol longer than 64 characters.
    SymbolTooLong,
    /// `E-LEX-011` — qname over 4 segments or 128 bytes.
    QnameTooLong,
    /// `E-LEX-020` — a literal outside its representable range.
    IntOutOfRange,
    /// `E-LEX-021` — a bare non-integer literal (no kind suffix).
    BareFloat,
    /// `E-LEX-022` — a lexically negative currency literal.
    NegativeCurrency,
    /// `E-LEX-023` — more fractional digits than the suffix permits.
    ExcessScale,
    /// `E-LEX-024` — a `p`/`i`/`c` literal outside `[0, 1]`.
    UnitIntervalOutOfRange,
    /// `E-LEX-025` — a bad string escape, or a raw LF inside a string.
    BadStringEscape,
    /// `E-LEX-026` — a string over 1024 bytes after escape processing.
    StringTooLong,
    /// `E-LEX-027` — an `r` (`Ratio`) literal that is not strictly positive
    /// (§1.5 addendum, Director ruling 2026-08-11 #492/ADR194). `Ratio`'s
    /// domain is `(0, ∞)`, open at zero — matching
    /// `babylon_kernel::scalars::Ratio`'s existing law exactly — so both a
    /// lexically negative literal and a literal that canonicalizes to `0`
    /// are refused here, at lex time, the same way `p`/`i`/`c`'s `[0,1]`
    /// bound is `E-LEX-024`.
    NonPositiveRatio,
}

impl LexCode {
    /// The spec's code string, e.g. `"E-LEX-003"`.
    #[must_use]
    pub fn spec_code(self) -> &'static str {
        match self {
            Self::InvalidUtf8OrBom => "E-LEX-001",
            Self::NonNfcString => "E-LEX-002",
            Self::UnclassifiableToken => "E-LEX-003",
            Self::SymbolTooLong => "E-LEX-010",
            Self::QnameTooLong => "E-LEX-011",
            Self::IntOutOfRange => "E-LEX-020",
            Self::BareFloat => "E-LEX-021",
            Self::NegativeCurrency => "E-LEX-022",
            Self::ExcessScale => "E-LEX-023",
            Self::UnitIntervalOutOfRange => "E-LEX-024",
            Self::BadStringEscape => "E-LEX-025",
            Self::StringTooLong => "E-LEX-026",
            Self::NonPositiveRatio => "E-LEX-027",
        }
    }
}

/// What went wrong: a spec-coded lexical error, or a structural parse error
/// the spec assigns no `E-LEX` code to (recorded as such, never invented).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadErrorKind {
    /// A lexical error with its spec code.
    Lex(LexCode),
    /// Input ended where a form was required.
    UnexpectedEof,
    /// A `)` with no open list.
    UnexpectedCloseParen,
    /// EOF inside an open list.
    UnterminatedList,
    /// EOF inside a string literal.
    UnterminatedString,
    /// More lists were opened than the reader can construct and drop safely.
    NestingLimitExceeded,
}

/// A reader failure: loud, positioned, never a panic (III.11).
#[derive(Debug, Clone, PartialEq)]
pub struct ReadError {
    /// The error class.
    pub kind: ReadErrorKind,
    /// Human-readable detail.
    pub message: String,
    /// Byte offset into the source where the error was detected.
    pub position: usize,
}

/// A byte range into the FILE (BOM included — [`read_all`] already re-bases
/// past a discarded BOM, and [`read_all_spanned`]'s table agrees, applying
/// the same re-base to every entry). Half-open: `&source[span.start..span.end]`
/// is the exact token or form text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The first byte of the form or atom, inclusive.
    pub start: usize,
    /// One past the last byte, exclusive.
    pub end: usize,
}

/// A path from the top-level forest to one node: the top-level form's
/// index, then one child index per level down. `[3]` is the fourth
/// top-level form; `[3, 0]` is its head atom. Never empty.
pub type FormPath = Vec<u32>;

/// Pre-order spans for one file (§2 of the #652 bsl-ls plan). Pre-order and
/// lexicographic `FormPath` order COINCIDE — a parent's path is always a
/// strict prefix of, and therefore sorts before, every descendant's path —
/// which is what makes [`SpanTable::span_of`] a binary search and what the
/// `spans_are_in_preorder` invariant test pins. Entries are collected in
/// whatever order the parser completes nodes (a list completes after its
/// children) and then sorted by path once per call, which is exactly the
/// pre-order sequence by the coincidence above.
///
/// No `HashMap`: paths are printable ("form 3.1.2"), survive cloning or
/// moving of subtrees, and a `Vec` keeps span lookup free of hash-iteration
/// order near the diagnostics path (Constitution III.7/III.12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanTable {
    entries: Vec<(FormPath, Span)>,
}

impl SpanTable {
    /// The span recorded for the node at `path`, if any.
    #[must_use]
    pub fn span_of(&self, path: &[u32]) -> Option<Span> {
        self.entries
            .binary_search_by(|(p, _)| p.as_slice().cmp(path))
            .ok()
            .map(|i| self.entries[i].1)
    }

    /// The path and span of the most deeply nested node whose span contains
    /// `offset` (`start <= offset < end`), or `None` if no span covers it
    /// (e.g. `offset` falls in leading/trailing whitespace outside every
    /// top-level form). Spans nest strictly by construction — a child's
    /// span is always a subset of its parent's — so the containing spans
    /// for any point form a chain, and the deepest (longest path) is the
    /// unique innermost one.
    #[must_use]
    pub fn innermost_at(&self, offset: usize) -> Option<(&[u32], Span)> {
        self.entries
            .iter()
            .filter(|(_, span)| span.start <= offset && offset < span.end)
            .max_by_key(|(path, _)| path.len())
            .map(|(path, span)| (path.as_slice(), *span))
    }

    /// How many nodes (atoms and lists, at every depth) this table covers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether this table covers zero nodes (an empty file's `read_all_spanned`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse ONE top-level form from `source`, returning it and the byte offset
/// where parsing stopped — callers loop over remaining input for multi-form
/// files (or use [`read_all`], the file-level entry that also performs the
/// UTF-8/BOM checks a `&str` cannot fail). Delegates to [`read_spanned`] and
/// drops the span table: there is exactly ONE parser (plan §2).
///
/// # Errors
/// [`ReadError`] on any lexical (`E-LEX-0xx`) or structural failure —
/// including a BOM at offset 0, which only [`read_all`] discards.
pub fn read(source: &str) -> Result<(SExpr, usize), ReadError> {
    let (expr, resume, _table) = read_spanned(source)?;
    Ok((expr, resume))
}

/// [`read`], plus a [`SpanTable`] covering the one form read. The table's
/// single top-level entry has path `[0]` — every call hardcodes
/// `top_level_index = 0`, so looping this over a multi-form file gives every
/// form's table the SAME path and spans relative to each slice passed in,
/// with no re-basing back to the whole file. Multi-form callers must use
/// [`read_all_spanned`], which assigns each form its own top-level index and
/// re-bases every span to the file as a whole.
///
/// # Errors
/// Same as [`read`].
pub fn read_spanned(source: &str) -> Result<(SExpr, usize, SpanTable), ReadError> {
    let mut scanner = Scanner::new(source);
    let mut entries: Vec<(FormPath, Span)> = Vec::new();
    let expr = parse_one(&mut scanner, 0, &mut entries)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((expr, scanner.byte_pos(), SpanTable { entries }))
}

/// Parse an entire BSL source file: validate UTF-8 (`E-LEX-001`), discard an
/// offset-0 BOM, then read every top-level form to end of input. Delegates
/// to [`read_all_spanned`] and drops the span table: there is exactly ONE
/// parser (plan §2).
///
/// # Errors
/// [`ReadError`] on invalid UTF-8, a misplaced BOM, or any failure from
/// [`read`] on a top-level form.
pub fn read_all(bytes: &[u8]) -> Result<Vec<SExpr>, ReadError> {
    let (forms, _table) = read_all_spanned(bytes)?;
    Ok(forms)
}

/// [`read_all`], plus a [`SpanTable`] covering every form in the file. The
/// Nth top-level form's own span has path `[N]`; BOM re-basing (`bom_len`)
/// is applied to every span exactly as [`read_all`] applies it to errors.
///
/// # Errors
/// Same as [`read_all`].
pub fn read_all_spanned(bytes: &[u8]) -> Result<(Vec<SExpr>, SpanTable), ReadError> {
    let text = std::str::from_utf8(bytes).map_err(|e| ReadError {
        kind: ReadErrorKind::Lex(LexCode::InvalidUtf8OrBom),
        message: "source is not valid UTF-8".into(),
        position: e.valid_up_to(),
    })?;
    // Error positions are byte offsets into the FILE: after discarding an
    // offset-0 BOM, every downstream position — including a span — is
    // re-based by its width so diagnostics still point into the bytes the
    // author sees.
    let (text, bom_len) = match text.strip_prefix('\u{feff}') {
        Some(stripped) => (stripped, '\u{feff}'.len_utf8()),
        None => (text, 0),
    };
    let mut scanner = Scanner::new(text);
    let mut forms = Vec::new();
    let mut entries: Vec<(FormPath, Span)> = Vec::new();
    let mut top_level_index: u32 = 0;
    loop {
        scanner.skip_trivia();
        if scanner.peek().is_none() {
            break;
        }
        let form = parse_one(&mut scanner, top_level_index, &mut entries).map_err(|mut e| {
            e.position += bom_len;
            e
        })?;
        forms.push(form);
        top_level_index += 1;
    }
    if bom_len > 0 {
        for (_, span) in &mut entries {
            span.start += bom_len;
            span.end += bom_len;
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((forms, SpanTable { entries }))
}

const WHITESPACE: [char; 4] = [' ', '\t', '\n', '\r'];

fn is_delimiter(c: char) -> bool {
    WHITESPACE.contains(&c) || matches!(c, '(' | ')' | ';')
}

fn lex_error(code: LexCode, message: impl Into<String>, position: usize) -> ReadError {
    ReadError {
        kind: ReadErrorKind::Lex(code),
        message: message.into(),
        position,
    }
}

struct Scanner<'a> {
    src: &'a str,
    iter: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl<'a> Scanner<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            iter: src.char_indices().peekable(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.iter.peek().map(|&(_, c)| c)
    }

    fn byte_pos(&mut self) -> usize {
        self.iter.peek().map_or(self.src.len(), |&(i, _)| i)
    }

    fn bump(&mut self) {
        self.iter.next();
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if WHITESPACE.contains(&c) => self.bump(),
                Some(';') => {
                    // A comment runs to the next LF or EOF; comments are
                    // whitespace (§1.2). Bounded: consumes ≥1 char per step.
                    while !matches!(self.peek(), None | Some('\n')) {
                        self.bump();
                    }
                }
                _ => return,
            }
        }
    }
}

/// Iterative s-expression parser: an explicit list stack instead of
/// recursion. The explicit stack refuses [`MAX_READER_NESTING_DEPTH`] + 1
/// before building a recursively droppable tree (III.11 — a crash is not a
/// loud error). Every iteration consumes at least one character or closes a
/// list opened by a consumed character, so the loop is bounded by the input
/// length.
///
/// `top_level_index` is this call's position among its caller's top-level
/// forms (always 0 for [`read_spanned`], the loop counter for
/// [`read_all_spanned`]) — the first element of every [`FormPath`] this
/// call records into `entries`. The stack carries each open list's `(`
/// offset (`usize`) alongside its children, so the offset survives
/// push→pop to become that list's span start; `path` is a live cursor
/// maintained in lockstep — pushed with a `0` on `(`, its last element
/// incremented after each completed child, popped on `)` — so at the
/// moment any node (atom, or list at its closing paren) completes, `path`
/// holds exactly that node's own [`FormPath`]. Entries land in whatever
/// order nodes complete (a list after its children); the caller sorts by
/// path once — [`read_spanned`] after this one call, [`read_all_spanned`]
/// once per FILE, after its loop over every top-level form — which,
/// because pre-order and lexicographic path order coincide (`SpanTable`'s
/// own doc), recovers exact pre-order across the whole file.
fn parse_one(
    scanner: &mut Scanner<'_>,
    top_level_index: u32,
    entries: &mut Vec<(FormPath, Span)>,
) -> Result<SExpr, ReadError> {
    let mut stack: Vec<(usize, Vec<SExpr>)> = Vec::new();
    let mut path: FormPath = vec![top_level_index];
    loop {
        scanner.skip_trivia();
        let position = scanner.byte_pos();
        let Some(c) = scanner.peek() else {
            return Err(if stack.is_empty() {
                ReadError {
                    kind: ReadErrorKind::UnexpectedEof,
                    message: "expected a form, found end of input".into(),
                    position,
                }
            } else {
                ReadError {
                    kind: ReadErrorKind::UnterminatedList,
                    message: "end of input inside an open list".into(),
                    position,
                }
            });
        };
        let completed = match c {
            '(' => {
                if stack.len() >= MAX_READER_NESTING_DEPTH {
                    return Err(ReadError {
                        kind: ReadErrorKind::NestingLimitExceeded,
                        message: format!(
                            "list nesting exceeds the reader safety limit of \
                             {MAX_READER_NESTING_DEPTH} open lists"
                        ),
                        position,
                    });
                }
                scanner.bump();
                stack.push((position, Vec::new()));
                path.push(0);
                continue;
            }
            ')' => {
                scanner.bump();
                match stack.pop() {
                    Some((open_pos, items)) => {
                        path.pop();
                        let end = scanner.byte_pos();
                        entries.push((
                            path.clone(),
                            Span {
                                start: open_pos,
                                end,
                            },
                        ));
                        SExpr::List(items)
                    }
                    None => {
                        return Err(ReadError {
                            kind: ReadErrorKind::UnexpectedCloseParen,
                            message: "')' with no open list".into(),
                            position,
                        })
                    }
                }
            }
            '\u{feff}' => {
                return Err(lex_error(
                    LexCode::InvalidUtf8OrBom,
                    "a BOM is only accepted at offset 0 (§1.1)",
                    position,
                ))
            }
            '"' => {
                let start = scanner.byte_pos();
                let atom = lex_string(scanner)?;
                let end = scanner.byte_pos();
                entries.push((path.clone(), Span { start, end }));
                SExpr::Atom(atom)
            }
            _ => {
                let (run, start) = lex_run(scanner)?;
                let end = scanner.byte_pos();
                let atom = classify(&run, start)?;
                entries.push((path.clone(), Span { start, end }));
                SExpr::Atom(atom)
            }
        };
        match stack.last_mut() {
            None => return Ok(completed),
            Some((_, items)) => {
                items.push(completed);
                *path
                    .last_mut()
                    .expect("path holds at least top_level_index while a list is open") += 1;
            }
        }
    }
}

/// Lex a string literal (§1.5): the four escapes only, single-line, ≤1024
/// bytes after escape processing, NFC-checked.
fn lex_string(scanner: &mut Scanner<'_>) -> Result<Atom, ReadError> {
    let start = scanner.byte_pos();
    scanner.bump(); // opening quote
    let mut content = String::new();
    loop {
        let position = scanner.byte_pos();
        match scanner.peek() {
            None => {
                return Err(ReadError {
                    kind: ReadErrorKind::UnterminatedString,
                    message: "end of input inside a string literal".into(),
                    position: start,
                })
            }
            Some('"') => {
                scanner.bump();
                break;
            }
            Some('\n') => {
                return Err(lex_error(
                    LexCode::BadStringEscape,
                    "raw LF inside a string literal — strings are single-line (§1.5)",
                    position,
                ))
            }
            Some('\u{feff}') => {
                return Err(lex_error(
                    LexCode::InvalidUtf8OrBom,
                    "a BOM is only accepted at offset 0 (§1.1)",
                    position,
                ))
            }
            Some('\\') => {
                scanner.bump();
                match scanner.peek() {
                    Some('"') => content.push('"'),
                    Some('\\') => content.push('\\'),
                    Some('n') => content.push('\n'),
                    Some('t') => content.push('\t'),
                    None => {
                        return Err(ReadError {
                            kind: ReadErrorKind::UnterminatedString,
                            message: "end of input inside a string escape".into(),
                            position: start,
                        })
                    }
                    Some(other) => {
                        return Err(lex_error(
                            LexCode::BadStringEscape,
                            format!("unknown string escape '\\{other}'"),
                            position,
                        ))
                    }
                }
                scanner.bump();
            }
            Some(other) => {
                content.push(other);
                scanner.bump();
            }
        }
    }
    let after = scanner.byte_pos();
    if scanner.peek().is_some_and(|c| !is_delimiter(c)) {
        return Err(lex_error(
            LexCode::UnclassifiableToken,
            "a string literal must be followed by a delimiter (§1.4: explicit separation)",
            after,
        ));
    }
    if content.len() > 1024 {
        return Err(lex_error(
            LexCode::StringTooLong,
            format!(
                "string is {} bytes after escape processing (max 1024)",
                content.len()
            ),
            start,
        ));
    }
    if !unicode_normalization::is_nfc(&content) {
        return Err(lex_error(
            LexCode::NonNfcString,
            "string literal is not in Unicode Normalization Form C",
            start,
        ));
    }
    Ok(Atom::Str(content))
}

/// Collect one maximal-munch token run: everything to the next delimiter
/// (§1.4 — the run is classified WHOLE; `1000.5$x` is never split).
fn lex_run(scanner: &mut Scanner<'_>) -> Result<(String, usize), ReadError> {
    let start = scanner.byte_pos();
    let mut run = String::new();
    while let Some(c) = scanner.peek() {
        if is_delimiter(c) {
            break;
        }
        if c == '\u{feff}' {
            return Err(lex_error(
                LexCode::InvalidUtf8OrBom,
                "a BOM is only accepted at offset 0 (§1.1)",
                scanner.byte_pos(),
            ));
        }
        run.push(c);
        scanner.bump();
    }
    Ok((run, start))
}

enum SymbolIssue {
    Invalid,
    TooLong,
}

/// Validate one `symbol` production: `LOWER ( LOWER | DIGIT | "-" )*`,
/// max 64 characters (`E-LEX-010`).
fn validate_symbol(s: &str) -> Result<(), SymbolIssue> {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return Err(SymbolIssue::Invalid),
    }
    if !chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(SymbolIssue::Invalid);
    }
    if s.len() > 64 {
        return Err(SymbolIssue::TooLong);
    }
    Ok(())
}

/// The closed operator-token set (§2 terminals / §5.2 form tags).
const OPERATORS: [&str; 10] = ["<", "<=", ">", ">=", "=", "!=", "+", "-", "*", "/"];

/// Classify one whole token run into an [`Atom`] (§1.4's atom classes,
/// plus the operator repair documented on [`Atom::Operator`]).
fn classify(run: &str, start: usize) -> Result<Atom, ReadError> {
    if run == "#t" {
        return Ok(Atom::Bool(true));
    }
    if run == "#f" {
        return Ok(Atom::Bool(false));
    }
    if OPERATORS.contains(&run) {
        return Ok(Atom::Operator(run.to_string()));
    }
    let unclassifiable = || {
        lex_error(
            LexCode::UnclassifiableToken,
            format!("'{run}' matches no atom class"),
            start,
        )
    };
    if let Some(rest) = run.strip_prefix(':') {
        return match validate_symbol(rest) {
            Ok(()) => Ok(Atom::Keyword(rest.to_string())),
            Err(SymbolIssue::TooLong) => Err(lex_error(
                LexCode::SymbolTooLong,
                "keyword symbol exceeds 64 characters",
                start,
            )),
            Err(SymbolIssue::Invalid) => Err(unclassifiable()),
        };
    }
    match run.chars().next() {
        Some(c) if c.is_ascii_digit() || c == '-' => classify_numeric(run, start),
        Some(c) if c.is_ascii_lowercase() => classify_name(run, start),
        Some(c) if c.is_ascii_uppercase() => classify_enum_ref(run, start),
        _ => Err(unclassifiable()),
    }
}

/// Classify a lowercase-initial run as `symbol` or `qname`.
fn classify_name(run: &str, start: usize) -> Result<Atom, ReadError> {
    let segments: Vec<&str> = run.split('/').collect();
    for segment in &segments {
        match validate_symbol(segment) {
            Ok(()) => {}
            Err(SymbolIssue::TooLong) => {
                return Err(lex_error(
                    LexCode::SymbolTooLong,
                    "symbol exceeds 64 characters",
                    start,
                ))
            }
            Err(SymbolIssue::Invalid) => {
                return Err(lex_error(
                    LexCode::UnclassifiableToken,
                    format!("'{run}' matches no atom class"),
                    start,
                ))
            }
        }
    }
    if segments.len() == 1 {
        return Ok(Atom::Symbol(run.to_string()));
    }
    if segments.len() > 4 || run.len() > 128 {
        return Err(lex_error(
            LexCode::QnameTooLong,
            "qname exceeds 4 segments or 128 bytes",
            start,
        ));
    }
    Ok(Atom::QName(run.to_string()))
}

/// Classify an uppercase-initial run as `enum-ref` (`EnumType/MEMBER`) or,
/// with no `/` present, as a bare identifier (`Atom::BareUpperIdent` —
/// §2.13, see its own doc for why this fallback exists rather than an
/// error). Registry membership is load-time (`E-LOAD-030/031`), not
/// checked here.
fn classify_enum_ref(run: &str, start: usize) -> Result<Atom, ReadError> {
    let unclassifiable = || {
        lex_error(
            LexCode::UnclassifiableToken,
            format!("'{run}' matches no atom class"),
            start,
        )
    };
    let Some((enum_type, member)) = run.split_once('/') else {
        return classify_bare_upper_ident(run, start);
    };
    let type_ok = enum_type
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
        && enum_type.chars().all(char::is_alphanumeric)
        && enum_type.is_ascii();
    let member_ok = member
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase())
        && member
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !type_ok || !member_ok || member.contains('/') {
        return Err(unclassifiable());
    }
    Ok(Atom::EnumRef {
        enum_type: enum_type.to_string(),
        member: member.to_string(),
    })
}

/// Classify a `/`-free uppercase-initial run as the UNION of §1.4's
/// `<enum-type>` and `<enum-member>` charsets (`UPPER (UPPER | LOWER |
/// DIGIT | "_")*`) — `Atom::BareUpperIdent`, see its own doc for why the
/// union is lexed here and split positionally downstream.
fn classify_bare_upper_ident(run: &str, start: usize) -> Result<Atom, ReadError> {
    let ok = run.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && run.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && run.is_ascii();
    if ok {
        Ok(Atom::BareUpperIdent(run.to_string()))
    } else {
        Err(lex_error(
            LexCode::UnclassifiableToken,
            format!("'{run}' matches no atom class"),
            start,
        ))
    }
}

/// Whether `s` conforms to §1.4's own `<enum-type>` production (`UPPER
/// (UPPER|LOWER|DIGIT)*`, no underscore) — non-empty, first character
/// ASCII uppercase, every character ASCII alphanumeric. Self-contained
/// (#528 fix round Item E): every in-crate caller already runs this
/// against a string lexed as [`Atom::BareUpperIdent`] (first character
/// uppercase by construction), but this function is `pub` (via `pub mod
/// reader`) and must not assume that precondition standing alone — the
/// pre-fix version returned `true` for `""` (`.all()` on an empty
/// iterator is vacuously `true`) and for a lowercase-initial string like
/// `"orgkind"` (`is_ascii_alphanumeric` does not distinguish case).
/// Used positionally by `crate::declarations::parse_defenum`/
/// `crate::scenario::load_defvocabulary` to validate their own type-name
/// operand — never by the reader itself, which stays lex-only (§2.13).
#[must_use]
pub fn is_enum_type_shape(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

/// Whether `s` conforms to §1.4's own `<enum-member>` production (`UPPER
/// (UPPER|DIGIT|"_")*`, no lowercase) — non-empty, first character ASCII
/// uppercase, every character ASCII uppercase, digit, or underscore.
/// Self-contained for the same reason [`is_enum_type_shape`] is (#528 fix
/// round Item E): the pre-fix version returned `true` for `""` and for a
/// digit-/underscore-initial string like `"_STATE"`. Used positionally by
/// `crate::declarations::parse_defenum`/`crate::scenario::
/// load_defvocabulary` to validate their own member-list items.
#[must_use]
pub fn is_enum_member_shape(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

/// Validate a `digits` group (`DIGIT ( "_"? DIGIT )*`): underscores only
/// BETWEEN digits (§1.5), and return it with underscores removed.
fn clean_digit_group(group: &str) -> Option<String> {
    if group.is_empty()
        || group.starts_with('_')
        || group.ends_with('_')
        || group.contains("__")
        || !group.chars().all(|c| c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    Some(group.chars().filter(char::is_ascii_digit).collect())
}

/// Powers of ten up to the widest scale any literal path needs.
const POW10: [i128; 10] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
];

/// Classify a digit-or-minus-initial run: `int-lit` or `scaled-lit` (§1.5).
fn classify_numeric(run: &str, start: usize) -> Result<Atom, ReadError> {
    let unclassifiable = || {
        lex_error(
            LexCode::UnclassifiableToken,
            format!("'{run}' matches no atom class"),
            start,
        )
    };
    let (negative, body) = match run.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, run),
    };
    let int_end = body
        .find(|c: char| !c.is_ascii_digit() && c != '_')
        .unwrap_or(body.len());
    let Some(int_digits) = clean_digit_group(&body[..int_end]) else {
        return Err(unclassifiable());
    };
    let rest = &body[int_end..];
    if rest.is_empty() {
        return classify_int(&int_digits, negative, start);
    }
    if let Some(frac_and_suffix) = rest.strip_prefix('.') {
        let frac_end = frac_and_suffix
            .find(|c: char| !c.is_ascii_digit() && c != '_')
            .unwrap_or(frac_and_suffix.len());
        let Some(frac_digits) = clean_digit_group(&frac_and_suffix[..frac_end]) else {
            return Err(unclassifiable());
        };
        return match &frac_and_suffix[frac_end..] {
            "" => Err(lex_error(
                LexCode::BareFloat,
                "a non-integer literal requires a kind suffix ($, p, i, c, r, m) — §1.5",
                start,
            )),
            "$" => classify_currency(&int_digits, &frac_digits, negative, start),
            "p" | "i" | "c" => classify_unit_interval(
                &int_digits,
                &frac_digits,
                suffix_kind(&frac_and_suffix[frac_end..]),
                negative,
                start,
            ),
            "r" => classify_ratio(&int_digits, &frac_digits, negative, start),
            "m" => classify_mass(&int_digits, &frac_digits, negative, start),
            _ => Err(unclassifiable()),
        };
    }
    match rest {
        "$" => classify_currency(&int_digits, "", negative, start),
        "p" | "i" | "c" => {
            classify_unit_interval(&int_digits, "", suffix_kind(rest), negative, start)
        }
        "r" => classify_ratio(&int_digits, "", negative, start),
        "m" => classify_mass(&int_digits, "", negative, start),
        _ => Err(unclassifiable()),
    }
}

/// An exact `m` literal: unsigned, at most nine fractional digits, and
/// represented as checked `u64` nanounits. Lexing never rounds.
fn classify_mass(
    int_digits: &str,
    frac_digits: &str,
    negative: bool,
    start: usize,
) -> Result<Atom, ReadError> {
    if negative {
        return Err(lex_error(
            LexCode::UnclassifiableToken,
            "a negative Mass literal is not expressible (including -0m)",
            start,
        ));
    }
    if frac_digits.len() > 9 {
        return Err(lex_error(
            LexCode::ExcessScale,
            "Mass literals take at most 9 fractional digits and are never rounded at lex time",
            start,
        ));
    }
    let out_of_range = || {
        lex_error(
            LexCode::IntOutOfRange,
            "Mass literal does not fit u64 nanounits",
            start,
        )
    };
    let unscaled: u64 = format!("{int_digits}{frac_digits}")
        .parse()
        .map_err(|_| out_of_range())?;
    let padding = 9_usize - frac_digits.len();
    let factor = u64::try_from(POW10[padding]).expect("10^0..10^9 fits u64");
    let nanounits = unscaled.checked_mul(factor).ok_or_else(out_of_range)?;
    Ok(Atom::Mass(crate::probability::Mass::from_nanounits(
        nanounits,
    )))
}

fn suffix_kind(suffix: &str) -> ScaledKind {
    match suffix {
        "p" => ScaledKind::Probability,
        "i" => ScaledKind::Intensity,
        _ => ScaledKind::Coefficient,
    }
}

fn classify_int(digits: &str, negative: bool, start: usize) -> Result<Atom, ReadError> {
    let out_of_range = || {
        lex_error(
            LexCode::IntOutOfRange,
            "integer literal does not fit i64 (§1.5)",
            start,
        )
    };
    let magnitude: i128 = digits.parse().map_err(|_| out_of_range())?;
    let value = if negative { -magnitude } else { magnitude };
    let value = i64::try_from(value).map_err(|_| out_of_range())?;
    Ok(Atom::Int(value))
}

/// A `$` literal: lexically-negative rejection, ≤6 fractional digits, exact
/// canonicalization to i128 micro-units (§1.5 — never rounded at lex time).
fn classify_currency(
    int_digits: &str,
    frac_digits: &str,
    negative: bool,
    start: usize,
) -> Result<Atom, ReadError> {
    if negative {
        // §1.5 names the LITERAL negative, so `-0$` rejects too (module doc).
        return Err(lex_error(
            LexCode::NegativeCurrency,
            "a negative currency literal is not expressible (§1.5)",
            start,
        ));
    }
    if frac_digits.len() > 6 {
        return Err(lex_error(
            LexCode::ExcessScale,
            "currency literals take at most 6 fractional digits — never rounded at lex time",
            start,
        ));
    }
    // Documented E-LEX-020 reuse (module doc): out-of-representable-range.
    let out_of_range = || {
        lex_error(
            LexCode::IntOutOfRange,
            "currency literal does not fit i128 micro-units",
            start,
        )
    };
    let unscaled: i128 = format!("{int_digits}{frac_digits}")
        .parse()
        .map_err(|_| out_of_range())?;
    let micros = unscaled
        .checked_mul(POW10[6 - frac_digits.len()])
        .ok_or_else(out_of_range)?;
    Ok(Atom::Currency(Currency::from_micro_units(micros)))
}

/// A `p`/`i`/`c` literal: scale ≤ 9, value in `[0, 1]`, canonical minimal
/// scale (trailing fractional zeros stripped, zero as `(0, 0)`).
fn classify_unit_interval(
    int_digits: &str,
    frac_digits: &str,
    kind: ScaledKind,
    negative: bool,
    start: usize,
) -> Result<Atom, ReadError> {
    if frac_digits.len() > 9 {
        return Err(lex_error(
            LexCode::ExcessScale,
            "p/i/c literals take at most 9 fractional digits (§1.5)",
            start,
        ));
    }
    let out_of_range = || {
        lex_error(
            LexCode::UnitIntervalOutOfRange,
            "p/i/c literals are bounded to [0, 1]",
            start,
        )
    };
    // A magnitude that overflows i128 is certainly outside [0, 1].
    let mut unscaled: i128 = format!("{int_digits}{frac_digits}")
        .parse()
        .map_err(|_| out_of_range())?;
    if negative && unscaled != 0 {
        return Err(out_of_range());
    }
    let mut scale = frac_digits.len();
    if unscaled > POW10[scale] {
        return Err(out_of_range());
    }
    while scale > 0 && unscaled % 10 == 0 {
        unscaled /= 10;
        scale -= 1;
    }
    if unscaled == 0 {
        scale = 0;
    }
    let scale = u8::try_from(scale).expect("scale is at most 9 by the check above");
    Ok(Atom::Scaled(ScaledLit {
        kind,
        unscaled,
        scale,
    }))
}

/// An `r` literal (§1.5 addendum, Director ruling 2026-08-11 #492/ADR194):
/// scale ≤ 9 (`E-LEX-023`, the same reuse `classify_unit_interval` documents
/// for the same reason — a second numbered code for "too many fractional
/// digits" would duplicate the one that already exists), value strictly
/// positive (`E-LEX-027`), canonical minimal scale. Unlike
/// [`classify_unit_interval`] there is no upper bound: `Ratio`'s domain is
/// `(0, ∞)`, matching `babylon_kernel::scalars::Ratio` exactly.
fn classify_ratio(
    int_digits: &str,
    frac_digits: &str,
    negative: bool,
    start: usize,
) -> Result<Atom, ReadError> {
    if frac_digits.len() > 9 {
        return Err(lex_error(
            LexCode::ExcessScale,
            "r literals take at most 9 fractional digits, the same cap p/i/c \
             use (§1.5)",
            start,
        ));
    }
    let non_positive = || {
        lex_error(
            LexCode::NonPositiveRatio,
            "r literals are bounded to (0, ∞) — strictly positive, matching \
             babylon_kernel::Ratio's domain",
            start,
        )
    };
    if negative {
        // §1.5 names the LITERAL negative for Currency (`-0$` rejects too);
        // the same reading applies here — a lexically negative Ratio
        // literal is refused before its magnitude is even inspected.
        return Err(non_positive());
    }
    let mut unscaled: i128 = format!("{int_digits}{frac_digits}")
        .parse()
        .map_err(|_| lex_error(LexCode::IntOutOfRange, "r literal does not fit i128", start))?;
    if unscaled == 0 {
        // Zero is in range but not in Ratio's OPEN lower bound.
        return Err(non_positive());
    }
    let mut scale = frac_digits.len();
    while scale > 0 && unscaled % 10 == 0 {
        unscaled /= 10;
        scale -= 1;
    }
    // A positive literal strictly below the kernel grid's half-step
    // (5e-7) quantizes to 0.0 under `babylon_kernel::grid::quantize`'s
    // half-up law (`(v * 1e6 + 0.5).floor()`), which `Ratio::new` then
    // rejects — non-positive AFTER the sort's law, so the reader refuses
    // it here rather than letting the loader's "the reader should have
    // refused this" path claim otherwise. Exactly 0.0000005 rounds UP to
    // the first grid point and stays legal.
    if scale >= 7 && 2 * unscaled < 10_i128.pow(u32::try_from(scale).unwrap() - 6) {
        return Err(lex_error(
            LexCode::NonPositiveRatio,
            "r literal quantizes to zero on the kernel's 1e-6 grid — \
             non-positive after the sort's law (positive values below \
             0.0000005 have no representable magnitude)",
            start,
        ));
    }
    let scale = u8::try_from(scale).expect("scale is at most 9 by the check above");
    Ok(Atom::Scaled(ScaledLit {
        kind: ScaledKind::Ratio,
        unscaled,
        scale,
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        read, read_all, read_all_spanned, Atom, FormPath, LexCode, ReadErrorKind, SExpr,
        ScaledKind, ScaledLit, MAX_READER_NESTING_DEPTH,
    };
    use babylon_kernel::Currency;

    fn one(source: &str) -> SExpr {
        read(source).expect("should parse").0
    }

    fn err_kind(source: &str) -> ReadErrorKind {
        read(source).expect_err("should fail").kind
    }

    fn lex_err(source: &str) -> LexCode {
        match err_kind(source) {
            ReadErrorKind::Lex(code) => code,
            other => panic!("expected a lexical error, got {other:?}"),
        }
    }

    fn atom(source: &str) -> Atom {
        match one(source) {
            SExpr::Atom(a) => a,
            SExpr::List(_) => panic!("expected an atom"),
        }
    }

    // ---- structure ----

    #[test]
    fn parses_a_flat_list() {
        assert_eq!(
            one("(add 1 2)"),
            SExpr::List(vec![
                SExpr::Atom(Atom::Symbol("add".into())),
                SExpr::Atom(Atom::Int(1)),
                SExpr::Atom(Atom::Int(2)),
            ])
        );
    }

    #[test]
    fn parses_nested_lists() {
        // NOTE: the plan's sketch wrote `social_class` — an ILLEGAL symbol
        // (underscore is not in the symbol alphabet, §1.4); kebab-case here.
        let expr = one("(fold (node social-class) (sum wealth))");
        assert!(matches!(expr, SExpr::List(items) if items.len() == 3));
    }

    #[test]
    fn reader_accepts_its_nesting_boundary_and_refuses_the_next_open_list() {
        let accepted = format!(
            "{}{}",
            "(".repeat(MAX_READER_NESTING_DEPTH),
            ")".repeat(MAX_READER_NESTING_DEPTH)
        );
        read(&accepted).expect("the exact reader safety boundary must parse");

        let rejected = format!(
            "{}{}",
            "(".repeat(MAX_READER_NESTING_DEPTH + 1),
            ")".repeat(MAX_READER_NESTING_DEPTH + 1)
        );
        let error = read(&rejected).expect_err("the next open list must refuse before tree build");
        assert_eq!(error.kind, ReadErrorKind::NestingLimitExceeded);
        assert_eq!(error.position, MAX_READER_NESTING_DEPTH);
        assert!(error.message.contains("reader safety limit"));
    }

    #[test]
    fn comments_are_whitespace() {
        assert_eq!(one("(add ; a comment\n 1 2)"), one("(add 1 2)"));
    }

    #[test]
    fn keyword_atoms_lex_without_their_colon() {
        let expr = one("(:material-basis \"exploitation of labor\")");
        assert_eq!(
            expr,
            SExpr::List(vec![
                SExpr::Atom(Atom::Keyword("material-basis".into())),
                SExpr::Atom(Atom::Str("exploitation of labor".into())),
            ])
        );
    }

    #[test]
    fn bool_literals_are_hash_t_and_hash_f_only() {
        assert_eq!(atom("#t"), Atom::Bool(true));
        assert_eq!(atom("#f"), Atom::Bool(false));
        // `true` is an ordinary symbol, NOT a boolean (§1.4).
        assert_eq!(atom("true"), Atom::Symbol("true".into()));
        assert_eq!(lex_err("#true"), LexCode::UnclassifiableToken);
    }

    #[test]
    fn unterminated_list_is_a_loud_error_not_a_panic() {
        assert_eq!(err_kind("(add 1 2"), ReadErrorKind::UnterminatedList);
    }

    #[test]
    fn unexpected_close_paren_is_loud() {
        assert_eq!(err_kind(")"), ReadErrorKind::UnexpectedCloseParen);
    }

    #[test]
    fn unterminated_string_at_eof_is_loud() {
        assert_eq!(
            err_kind("(:material-basis \"unterminated"),
            ReadErrorKind::UnterminatedString
        );
    }

    #[test]
    fn empty_input_is_unexpected_eof() {
        assert_eq!(
            err_kind("   ; only a comment"),
            ReadErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn read_returns_the_resume_position() {
        let source = "(a) (b)";
        let (first, resume) = read(source).unwrap();
        assert_eq!(
            first,
            SExpr::List(vec![SExpr::Atom(Atom::Symbol("a".into()))])
        );
        let (second, _) = read(&source[resume..]).unwrap();
        assert_eq!(
            second,
            SExpr::List(vec![SExpr::Atom(Atom::Symbol("b".into()))])
        );
    }

    #[test]
    fn read_all_reads_every_top_level_form() {
        let forms = read_all(b"(a) ; comment\n(b 1)").unwrap();
        assert_eq!(forms.len(), 2);
    }

    // ---- E-LEX-001: UTF-8 and BOM ----

    #[test]
    fn invalid_utf8_is_e_lex_001() {
        let err = read_all(&[0xff, 0xfe, b'(', b')']).unwrap_err();
        assert_eq!(err.kind, ReadErrorKind::Lex(LexCode::InvalidUtf8OrBom));
    }

    #[test]
    fn a_bom_at_offset_zero_is_discarded() {
        let forms = read_all("\u{feff}(a)".as_bytes()).unwrap();
        assert_eq!(forms.len(), 1);
    }

    /// Error positions are byte offsets into the FILE, not into the
    /// BOM-stripped view — a stripped BOM must not skew every subsequent
    /// diagnostic by its 3 bytes.
    #[test]
    fn error_positions_count_a_stripped_bom() {
        let plain = read_all(b"(a) \x01").unwrap_err();
        let bommed = read_all("\u{feff}(a) \u{01}".as_bytes()).unwrap_err();
        assert_eq!(bommed.kind, plain.kind);
        assert_eq!(
            bommed.position,
            plain.position + '\u{feff}'.len_utf8(),
            "the BOM's bytes vanished from the reported position"
        );
    }

    #[test]
    fn a_bom_anywhere_else_is_e_lex_001() {
        assert_eq!(lex_err("(a \u{feff}b)"), LexCode::InvalidUtf8OrBom);
    }

    // ---- E-LEX-002: NFC ----

    #[test]
    fn a_non_nfc_string_is_e_lex_002() {
        // "e" + COMBINING ACUTE = the decomposed form of "é" — not NFC.
        assert_eq!(lex_err("\"e\u{0301}\""), LexCode::NonNfcString);
        // The composed form IS NFC.
        assert_eq!(atom("\"\u{e9}\""), Atom::Str("\u{e9}".into()));
    }

    // ---- E-LEX-003: unclassifiable runs (maximal munch) ----

    #[test]
    fn maximal_munch_never_splits_a_run() {
        // §1.4's own example: one run, classifies as nothing.
        assert_eq!(lex_err("1000.5$x"), LexCode::UnclassifiableToken);
    }

    #[test]
    fn a_leading_digit_is_mandatory_in_decimals() {
        assert_eq!(lex_err(".5c"), LexCode::UnclassifiableToken);
    }

    #[test]
    fn bad_underscore_placement_is_e_lex_003() {
        assert_eq!(lex_err("1__0"), LexCode::UnclassifiableToken);
        assert_eq!(lex_err("_1"), LexCode::UnclassifiableToken);
        assert_eq!(lex_err("1_"), LexCode::UnclassifiableToken);
    }

    #[test]
    fn snake_case_is_not_a_symbol() {
        assert_eq!(lex_err("social_class"), LexCode::UnclassifiableToken);
    }

    #[test]
    fn a_string_needs_a_delimiter_after_its_close_quote() {
        assert_eq!(lex_err("\"a\"x"), LexCode::UnclassifiableToken);
    }

    // ---- E-LEX-010 / E-LEX-011: identifier lengths ----

    #[test]
    fn symbol_length_caps_at_64() {
        let ok = "a".repeat(64);
        assert_eq!(atom(&ok), Atom::Symbol(ok.clone()));
        let long = "a".repeat(65);
        assert_eq!(lex_err(&long), LexCode::SymbolTooLong);
    }

    #[test]
    fn qname_caps_at_4_segments_and_128_bytes() {
        assert_eq!(
            atom("vitality/starvation-mortality"),
            Atom::QName("vitality/starvation-mortality".into())
        );
        assert_eq!(lex_err("a/b/c/d/e"), LexCode::QnameTooLong);
        let wide = format!("{}/{}", "a".repeat(64), "b".repeat(64));
        assert_eq!(lex_err(&wide), LexCode::QnameTooLong);
    }

    // ---- E-LEX-020: integer range ----

    #[test]
    fn int_literals_must_fit_i64() {
        assert_eq!(atom("9223372036854775807"), Atom::Int(i64::MAX));
        assert_eq!(lex_err("9223372036854775808"), LexCode::IntOutOfRange);
        assert_eq!(atom("-9223372036854775808"), Atom::Int(i64::MIN));
    }

    // ---- E-LEX-021: bare floats ----

    #[test]
    fn a_bare_decimal_without_a_kind_suffix_is_e_lex_021() {
        assert_eq!(lex_err("0.5"), LexCode::BareFloat);
    }

    // ---- E-LEX-022: negative currency ----

    #[test]
    fn a_lexically_negative_currency_literal_is_e_lex_022() {
        assert_eq!(lex_err("-1$"), LexCode::NegativeCurrency);
        // §1.5 names the LITERAL negative, so even `-0$` rejects.
        assert_eq!(lex_err("-0$"), LexCode::NegativeCurrency);
        assert_eq!(atom("0$"), Atom::Currency(Currency::from_micro_units(0)));
    }

    // ---- E-LEX-023: scale limits ----

    #[test]
    fn currency_takes_at_most_6_fractional_digits() {
        assert_eq!(
            atom("1.123456$"),
            Atom::Currency(Currency::from_micro_units(1_123_456))
        );
        assert_eq!(lex_err("1.1234567$"), LexCode::ExcessScale);
    }

    #[test]
    fn unit_interval_literals_take_at_most_scale_9() {
        assert_eq!(
            atom("0.123456789p"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Probability,
                unscaled: 123_456_789,
                scale: 9
            })
        );
        assert_eq!(lex_err("0.1234567891p"), LexCode::ExcessScale);
    }

    // ---- E-LEX-024: unit-interval range ----

    #[test]
    fn unit_interval_literals_reject_values_outside_zero_one() {
        assert_eq!(lex_err("1.5p"), LexCode::UnitIntervalOutOfRange);
        assert_eq!(lex_err("2p"), LexCode::UnitIntervalOutOfRange);
        assert_eq!(lex_err("-0.5c"), LexCode::UnitIntervalOutOfRange);
        assert_eq!(
            atom("1.0i"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Intensity,
                unscaled: 1,
                scale: 0
            })
        );
    }

    // ---- E-LEX-027: Ratio literals (§1.5 addendum, #492/ADR194) ----

    #[test]
    fn ratio_literals_lex_with_no_upper_bound() {
        assert_eq!(
            atom("2.0r"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Ratio,
                unscaled: 2,
                scale: 0
            })
        );
        assert_eq!(
            atom("10r"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Ratio,
                unscaled: 10,
                scale: 0
            })
        );
        // Comfortably beyond [0,1] — the whole point of the sort.
        assert_eq!(
            atom("1000000r"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Ratio,
                unscaled: 1_000_000,
                scale: 0
            })
        );
    }

    #[test]
    fn ratio_literals_reject_zero_and_negative() {
        assert_eq!(lex_err("0r"), LexCode::NonPositiveRatio);
        assert_eq!(lex_err("0.0r"), LexCode::NonPositiveRatio);
        assert_eq!(lex_err("-1r"), LexCode::NonPositiveRatio);
        assert_eq!(lex_err("-0r"), LexCode::NonPositiveRatio);
    }

    /// A positive literal strictly below the kernel grid's half-step
    /// (5e-7) quantizes to 0.0 and `Ratio::new` would reject it — the
    /// reader refuses it as `E-LEX-027` so the loader's "the reader
    /// should have refused this" framing stays true. Exactly `0.0000005r`
    /// rounds UP to the first grid point and is legal; killing the
    /// under-grid check accepts `0.0000004r` and flips this test.
    #[test]
    fn ratio_literals_reject_values_that_quantize_to_zero_on_the_grid() {
        assert_eq!(lex_err("0.0000004r"), LexCode::NonPositiveRatio);
        assert_eq!(lex_err("0.0000001r"), LexCode::NonPositiveRatio);
        assert_eq!(lex_err("0.000000001r"), LexCode::NonPositiveRatio);
        // The half-step itself rounds UP and stays legal.
        assert_eq!(
            atom("0.0000005r"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Ratio,
                unscaled: 5,
                scale: 7,
            })
        );
    }

    #[test]
    fn ratio_literals_take_at_most_scale_9() {
        assert_eq!(
            atom("0.123456789r"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Ratio,
                unscaled: 123_456_789,
                scale: 9
            })
        );
        assert_eq!(lex_err("0.1234567891r"), LexCode::ExcessScale);
    }

    #[test]
    fn ratio_literals_canonicalize_to_minimal_scale() {
        assert_eq!(atom("2.50r"), atom("2.5r"));
    }

    // ---- E-LEX-025: string escapes ----

    #[test]
    fn only_the_four_escapes_exist() {
        assert_eq!(
            atom(r#""a\"b\\c\nd\te""#),
            Atom::Str("a\"b\\c\nd\te".into())
        );
        assert_eq!(lex_err(r#""a\qb""#), LexCode::BadStringEscape);
    }

    #[test]
    fn a_raw_lf_inside_a_string_is_e_lex_025() {
        assert_eq!(lex_err("\"line one\nline two\""), LexCode::BadStringEscape);
    }

    // ---- E-LEX-026: string length ----

    #[test]
    fn strings_cap_at_1024_bytes_after_escape_processing() {
        let ok = format!("\"{}\"", "x".repeat(1024));
        assert!(read(&ok).is_ok());
        let long = format!("\"{}\"", "x".repeat(1025));
        assert_eq!(lex_err(&long), LexCode::StringTooLong);
    }

    // ---- canonicalization (§1.5) ----

    #[test]
    fn scaled_literals_canonicalize_to_minimal_scale() {
        assert_eq!(atom("0.50c"), atom("0.5c"));
        assert_eq!(
            atom("0.0p"),
            Atom::Scaled(ScaledLit {
                kind: ScaledKind::Probability,
                unscaled: 0,
                scale: 0
            })
        );
        assert_eq!(atom("-0.0p"), atom("0p"));
    }

    #[test]
    fn leading_zeros_and_underscores_are_insignificant() {
        assert_eq!(atom("007"), Atom::Int(7));
        assert_eq!(atom("1_000"), Atom::Int(1000));
    }

    #[test]
    fn currency_canonicalizes_to_micro_units() {
        assert_eq!(
            atom("1.5$"),
            Atom::Currency(Currency::from_micro_units(1_500_000))
        );
        assert_eq!(
            atom("2$"),
            Atom::Currency(Currency::from_micro_units(2_000_000))
        );
    }

    // ---- operators (the §1.4 repair — see Atom::Operator) ----

    #[test]
    fn the_ten_operator_tokens_lex_as_operators() {
        for op in ["<", "<=", ">", ">=", "=", "!=", "+", "-", "*", "/"] {
            assert_eq!(atom(op), Atom::Operator(op.into()));
        }
        // Maximal munch still applies: adjacency is not separation.
        assert_eq!(lex_err("<x"), LexCode::UnclassifiableToken);
        // `-` alone is an operator; `-5` is still an int literal.
        assert_eq!(atom("-5"), Atom::Int(-5));
    }

    #[test]
    fn comparison_forms_parse() {
        // The §5.6 worked example's own condition shape.
        let expr = one("(< wealth 1000.5$)");
        assert!(matches!(expr, SExpr::List(items) if items.len() == 3
            && matches!(&items[0], SExpr::Atom(Atom::Operator(op)) if op == "<")));
    }

    // ---- enum refs and qnames ----

    #[test]
    fn enum_refs_carry_type_and_member() {
        assert_eq!(
            atom("NodeType/SOCIAL_CLASS"),
            Atom::EnumRef {
                enum_type: "NodeType".into(),
                member: "SOCIAL_CLASS".into()
            }
        );
    }

    #[test]
    fn an_enum_member_is_the_identifier_never_the_serialized_value() {
        // `NodeType/social_class` matches no atom class (§1.4's own example
        // of the rule: the member must be UPPER-alphabet).
        assert_eq!(
            lex_err("NodeType/social_class"),
            LexCode::UnclassifiableToken
        );
        assert_eq!(lex_err("foo/Bar"), LexCode::UnclassifiableToken);
    }

    // ---- §2.13 bare uppercase identifiers (Organization contract, Q12)
    // ---- and #528's own fix round (the bare-<enum-member> repair) -------

    #[test]
    fn a_bare_uppercase_run_with_no_slash_is_a_bare_upper_ident() {
        assert_eq!(atom("OrgKind"), Atom::BareUpperIdent("OrgKind".into()));
        assert_eq!(atom("NodeType"), Atom::BareUpperIdent("NodeType".into()));
    }

    #[test]
    fn a_bare_upper_ident_admits_the_union_of_enum_type_and_enum_member_charsets() {
        // §1.4's <enum-type> ::= UPPER (UPPER|LOWER|DIGIT)* permits
        // lowercase but not underscore; <enum-member> ::=
        // UPPER (UPPER|DIGIT|"_")* is the mirror (underscore, no
        // lowercase). The READER lexes the UNION of both — shape
        // conformance to one production or the other is the PARSER's job
        // (`is_enum_type_shape`/`is_enum_member_shape`, exercised in
        // `declarations.rs`/`scenario.rs`), never the lexer's.
        assert_eq!(
            atom("HexResolution2"),
            Atom::BareUpperIdent("HexResolution2".into())
        );
        assert_eq!(
            atom("Org_Kind"),
            Atom::BareUpperIdent("Org_Kind".into()),
            "a lexer-level charset union admits underscore even though \
             <enum-type> alone would not — the parser rejects this as a \
             type-name operand, not the reader"
        );
        assert_eq!(
            atom("STATE_APPARATUS"),
            Atom::BareUpperIdent("STATE_APPARATUS".into())
        );
    }

    #[test]
    fn a_slash_terminated_or_double_slash_run_still_refuses() {
        // The fallback only fires for a TRULY slash-free run; a malformed
        // enum-ref (trailing/empty segments) must still be unclassifiable,
        // never silently reread as a bare identifier.
        assert_eq!(lex_err("OrgKind/"), LexCode::UnclassifiableToken);
        assert_eq!(lex_err("/BUSINESS"), LexCode::UnclassifiableToken);
    }

    // ---- #528 fix round: the two tree-sitter corpus lines, verbatim
    // (test/corpus/declarations.txt:144-145). Before the fix,
    // `STATE_APPARATUS` failed to lex at all (E-LEX-003): the pre-fix
    // `classify_enum_type_name` refused underscore, because this crate had
    // read a `defenum`/`defvocabulary` member list as full `Type/MEMBER`
    // refs rather than the bare `<enum-member>`s §2.13's own EBNF
    // declares.

    #[test]
    fn every_member_of_the_defenum_corpus_line_lexes() {
        // (defenum OrgKind (STATE_APPARATUS BUSINESS POLITICAL_FACTION
        //  CIVIL_SOCIETY)) — test/corpus/declarations.txt:144.
        for member in [
            "OrgKind",
            "STATE_APPARATUS",
            "BUSINESS",
            "POLITICAL_FACTION",
            "CIVIL_SOCIETY",
        ] {
            assert_eq!(
                atom(member),
                Atom::BareUpperIdent(member.into()),
                "{member} must lex as a bare uppercase identifier"
            );
        }
    }

    #[test]
    fn every_member_of_the_defvocabulary_corpus_line_lexes() {
        // (defvocabulary NodeType (SOCIAL_CLASS TERRITORY ORGANIZATION)) —
        // test/corpus/declarations.txt:145.
        for member in ["NodeType", "SOCIAL_CLASS", "TERRITORY", "ORGANIZATION"] {
            assert_eq!(
                atom(member),
                Atom::BareUpperIdent(member.into()),
                "{member} must lex as a bare uppercase identifier"
            );
        }
    }

    // ---- is_enum_type_shape / is_enum_member_shape (the parser-level
    // positional split a bare-upper-ident's two consumers need) ----------

    #[test]
    fn is_enum_type_shape_accepts_lowercase_and_rejects_underscore() {
        assert!(super::is_enum_type_shape("OrgKind"));
        assert!(super::is_enum_type_shape("HexResolution2"));
        assert!(!super::is_enum_type_shape("Org_Kind"));
        assert!(!super::is_enum_type_shape("STATE_APPARATUS"));
        // #528 fix round Item E: both predicates are `pub` (via `pub mod
        // reader`), so they must be self-contained rather than trusting a
        // caller already ran them through the lexer — "" and a
        // lowercase-initial string are the two shapes only the
        // already-lexed precondition ruled out.
        assert!(
            !super::is_enum_type_shape(""),
            "empty string is not a shape"
        );
        assert!(
            !super::is_enum_type_shape("orgkind"),
            "a lowercase-initial string is not a shape, even if every char is alphanumeric"
        );
    }

    #[test]
    fn is_enum_member_shape_accepts_underscore_and_rejects_lowercase() {
        assert!(super::is_enum_member_shape("STATE_APPARATUS"));
        assert!(super::is_enum_member_shape("BUSINESS"));
        assert!(!super::is_enum_member_shape("OrgKind"));
        assert!(!super::is_enum_member_shape("HexResolution2"));
        // #528 fix round Item E: same self-containment repair.
        assert!(
            !super::is_enum_member_shape(""),
            "empty string is not a shape"
        );
        assert!(
            !super::is_enum_member_shape("_STATE"),
            "an underscore-initial string is not a shape — the first char must be UPPER"
        );
    }

    // ---- span table invariants (Task 1, #652 bsl-ls plan §2, Task 1.4) ----
    // Unit tests, not integration tests: (a) needs the private `entries`
    // field directly (this module is a descendant of `reader`, so it can
    // see it) rather than inferring order from public-API round-trips.

    fn spanned(source: &str) -> (Vec<SExpr>, super::SpanTable) {
        read_all_spanned(source.as_bytes()).expect("fixture should parse")
    }

    /// Every `(FormPath, &SExpr)` pair in the parsed forest, in pre-order —
    /// an INDEPENDENT walk (not the parser's own bookkeeping), used as the
    /// oracle for `every_node_has_one_entry`/`spans_slice_back_to_the_source`.
    fn hand_walk(forms: &[SExpr]) -> Vec<(FormPath, &SExpr)> {
        fn walk<'a>(expr: &'a SExpr, path: &[u32], out: &mut Vec<(FormPath, &'a SExpr)>) {
            out.push((path.to_vec(), expr));
            if let SExpr::List(items) = expr {
                for (i, item) in items.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(u32::try_from(i).expect("child index fits u32"));
                    walk(item, &child_path, out);
                }
            }
        }
        let mut out = Vec::new();
        for (i, form) in forms.iter().enumerate() {
            let top = vec![u32::try_from(i).expect("top-level index fits u32")];
            walk(form, &top, &mut out);
        }
        out
    }

    /// Extended past the plan's own two-rule sketch (review round 1, #652
    /// Task 1, finding I1) to exercise every span-emitting site at least
    /// once: a leading `;` comment (so "span starts AFTER trivia" is
    /// load-bearing, not vacuous), a string atom (`(note "hi")` —
    /// `lex_string`'s closing-quote consumption is the one `end` that
    /// depends on a callee, not just `byte_pos()` after a maximal munch),
    /// an empty list `()` (a list whose span covers zero children), and a
    /// scaled literal (`0.05i`) alongside the pre-existing plain int.
    const NESTED_FIXTURE: &str = "; a comment\n(rule demo/x (bindings (binding a :field b/c)) \
         (when (< a 1)) (note \"hi\") (price 0.05i) ())\n(rule demo/y)";

    #[test]
    fn spans_are_in_preorder() {
        let (forms, table) = spanned(NESTED_FIXTURE);
        // (a) Direct pin against an independent oracle (review round 1,
        // finding M1): `hand_walk` performs a genuine pre-order traversal,
        // so equality with it pins the table's sequence exactly, not via
        // the sortedness+lemma conjunction the strict-`<` check alone
        // established.
        let walked = hand_walk(&forms);
        assert!(
            table
                .entries
                .iter()
                .map(|(p, _)| p)
                .eq(walked.iter().map(|(p, _)| p)),
            "table entries must appear in exactly hand_walk's independent pre-order sequence"
        );
        // (b) Strict sortedness, checked directly too: this is the actual
        // precondition `span_of`'s binary search depends on, and is a
        // distinct property from "matches hand_walk" (a buggy hand_walk
        // that happened to still be pre-order-shaped wouldn't catch a
        // duplicate or misordered path the way this does).
        for window in table.entries.windows(2) {
            let (a_path, _) = &window[0];
            let (b_path, _) = &window[1];
            assert!(
                a_path < b_path,
                "consecutive entries must be strictly lexicographically \
                 increasing: {a_path:?} !< {b_path:?}"
            );
        }
    }

    #[test]
    fn every_node_has_one_entry() {
        let (forms, table) = spanned(NESTED_FIXTURE);
        let walked = hand_walk(&forms);
        assert_eq!(table.len(), walked.len());
        assert!(!table.is_empty());
        for (path, _) in &walked {
            assert!(
                table.span_of(path).is_some(),
                "hand-walked path {path:?} has no table entry"
            );
        }
    }

    #[test]
    fn spans_slice_back_to_the_source() {
        let (forms, table) = spanned(NESTED_FIXTURE);
        for (path, expr) in hand_walk(&forms) {
            let span = table
                .span_of(&path)
                .expect("hand-walked path is in the table");
            let slice = &NESTED_FIXTURE[span.start..span.end];
            let (reparsed, resume) = read(slice)
                .unwrap_or_else(|e| panic!("{slice:?} (path {path:?}) must re-read: {e:?}"));
            assert_eq!(
                &reparsed, expr,
                "path {path:?}'s span {span:?} = {slice:?} did not re-read to the same node"
            );
            // Review round 1 finding M2: without this, an over-long span
            // (end extending into trailing whitespace or the next
            // sibling) would still re-read to the same leading node and
            // pass unnoticed — `read` stops at the first form and ignores
            // trailing bytes. Pinning the resume offset against the
            // slice's own length catches exactly that.
            assert_eq!(
                resume,
                slice.len(),
                "path {path:?}'s span {span:?} = {slice:?} is over-long: read stopped at byte \
                 {resume} but the slice is {} bytes",
                slice.len()
            );
        }
    }

    /// (e) The delegation-wiring row (plan §2.4/Task 1.4(e)) — a GLUE check
    /// only: once `read_all` is defined as `read_all_spanned` with the
    /// table dropped, this cannot detect a bug in the shared tree-building
    /// logic (both sides run the identical parser), only miswired glue —
    /// e.g. returning `.1` instead of `.0`, or applying the BOM re-base to
    /// one path and not the other.
    #[test]
    fn read_all_delegates_to_read_all_spanned_glue_check_only() {
        for source in [
            "(a) (b)",
            "\u{feff}(rule demo/x)\n(rule demo/y)",
            NESTED_FIXTURE,
        ] {
            let direct = read_all(source.as_bytes()).expect("read_all must parse the fixture");
            let (via_spanned, _table) = read_all_spanned(source.as_bytes())
                .expect("read_all_spanned must parse the fixture");
            assert_eq!(
                direct, via_spanned,
                "read_all(src) must equal read_all_spanned(src).0 exactly"
            );
        }
    }
}
