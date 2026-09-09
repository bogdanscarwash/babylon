# tree-sitter-bsl

A [tree-sitter](https://tree-sitter.github.io/) grammar for **BSL**, the
Babylon Scripting Language.

## Derivation status: DERIVED, NON-NORMATIVE

`docs/reference/bsl-language.rst` is the **one normative home** for BSL. Its
§7 collects every production into `docs/reference/bsl.ebnf`, which is part of
that document by inclusion. **`grammar.js` is derived from `bsl.ebnf`**, and
it declares nothing:

- on any divergence between this grammar and the rst, **the rst wins** and
  this grammar is the defect;
- this is a **parser for tooling**, not a validator. Every static check of
  §3 — types, the intensivity kind rule, scope, ceilings, the load-time fuel
  bound, the closed vocabulary — lives in the reference implementation at
  `rust/crates/babylon-bsl`. A file this grammar parses may still be rejected
  at content load by any of them, loudly (III.11);
- adding a form here does not add it to the language. The route for that is
  the rst, its decision register, and — where the change reaches the
  formalism surface — an amendment.

## Layout

| Path                     | What it is                                              |
| ------------------------ | ------------------------------------------------------- |
| `grammar.js`             | The grammar. The only hand-written source here.         |
| `queries/highlights.scm` | Highlight captures (standard nvim-treesitter names).    |
| `test/corpus/*.txt`      | Corpus tests, sourced from real in-tree content.        |
| `tree-sitter.json`       | CLI metadata: scope, file types (`.bsl`, `.bscn`).      |
| `src/`                   | **Generated**, git-ignored. Never edit; regenerate.     |

## Regenerate and test

```bash
mise run bsl:grammar-test        # generate + run the corpus (the sanctioned path)
```

or, directly, from this directory:

```bash
tree-sitter generate             # writes src/parser.c and friends
tree-sitter test                 # runs test/corpus
tree-sitter parse <file.bsl>     # print a parse tree; exits non-zero on error
```

The CLI ships with the repo's Rust toolchain (`cargo install tree-sitter-cli`,
already present at `~/.cargo/bin/tree-sitter` on the dev box); node is
installed at the pinned version by `mise install`. The generated
parser is **not committed** — it is a build product of `grammar.js`, and
committing it would make the grammar's real source ambiguous.

Verified with tree-sitter CLI 0.25.10, node v26.5.1.

## What the corpus covers

Every form in §2, plus the atom classes of §1.4–§1.5, **sourced from** real
content rather than invented examples: the conformance vectors under
`rust/crates/babylon-bsl/tests/conformance/`, the content rule under
`rust/crates/babylon-tick/content/rules/`, the scenario under
`.../content/scenarios/`, and the worked examples the rst itself carries
(§2.5, §2.6, §2.8, §2.9, §2.10, §2.11, §2.12, §3.9, §5.6).

**Sourced from is not identical to, and every case title says which.** Cases
are marked `VERBATIM`, `ABRIDGED` (a `:material-basis` string or a binding
list shortened for readability) or — for the scenario case — `TRIMMED and
AUGMENTED`: that one adds an `(edge …)` form taken from `scenario.rs`'s
module doc, because nothing in the tree exercises the scenario dialect's edge
form. The corpus is a **grammar witness, never a byte-regression** against
those files. For the latter, parse the files themselves
(`tree-sitter parse <file>`) — which is how the table below was produced.

Against the in-tree estate, as of this grammar's landing:

- 11 of the 12 conformance `.bsl` vectors and the one content rule parse
  clean;
- `empty_when.bsl` **does not parse**, and must not: `(when)` is
  `E-PARSE-020`, a rejecting vector by design. The parser reports a missing
  `#t`, which is the language's own advice for spelling "always";
- `two-classes.bscn` parses entirely through the `generic_form` fallback —
  see the next section.

## Known limitations, each deliberate

1. **The `.bscn` scenario format has no normative home** — now recorded as
   **register row D93** in `bsl-language.rst`, which is where the conflict
   belongs; this file is normative for nothing and cannot carry it. The rst
   specifies content forms (§2) and the §6.1 vector fixture format, and says
   nothing about `(scenario …)`, `(node …)` or `(edge …)` — yet
   `rust/crates/babylon-bsl/src/scenario.rs` reads exactly those. They parse
   here as `generic_form`, the fallback for a form whose head the content
   grammar does not name. That is honest tooling support, not a
   specification: nothing about the scenario dialect is derivable from
   `bsl.ebnf`. Note in particular that its `deffield` is **positional** —
   `(deffield <qname> <type> <kind>)` — where §2.9's takes `:type` and
   `:kind` keywords; the two forms share a name and not a shape.
2. **§6.1 `vector` forms** likewise land in `generic_form`. They *are*
   specified (§6.1, collected in `bsl.ebnf` §3) — but no in-tree file uses
   one yet, so structuring them here would be a grammar with no corpus to
   hold it honest. It is a follow-up, not an omission.
3. **Reserved words.** Every §5.2 form-head symbol is a keyword token here.
   Value positions accept them through the `symbol` rule (a binding named
   `set` is legal BSL — D33 reserves those names against the *intrinsic*
   namespace only).

   Whether one may **head** a `generic_form` depends on position, and the two
   cases differ:

   - where the content grammar admits the matching form, the keyword wins and
     the structured rule applies. A **top-level** `(deffield <qname> <type>
     <kind>)` — the `.bscn` positional shape — is therefore an error, because
     §2.9's `deffield` takes `:type` and `:kind`;
   - where the content grammar admits no form at all — inside a
     `generic_form` body — the head lexes as a plain symbol and the fallback
     takes it. That same positional `deffield` nested in `(scenario …)`
     parses clean, which is why the real `.bscn` file parses end to end.

   So limitation 1's divergence stays *visible* without making the scenario
   file unopenable.
4. **`type_name` is a lowercase symbol**, per the gap `bsl.ebnf` records:
   §3.1 spells the type names capitalized, and a bare capitalized run matches
   no §1.4 atom class. The reference implementation reads them lowercase and
   records the same gap.
5. **No `word` token.** The usual keyword-extraction optimization is declined
   because with it the parser silently repairs a missing `symbol` operand
   into a zero-width node that `tree-sitter parse` does not flag — a tool
   that quietly fixes its input is the silent-degradation shape this project
   rejects.

## Editor wiring

Not packaged yet — no npm publish, no nvim-treesitter registration, no VS Code
extension. `tree-sitter.json` carries the scope (`source.bsl`) and file types
a host needs, and `queries/highlights.scm` is written to the standard capture
set, so wiring it into a local `nvim-treesitter` install is a
`parser_config` entry plus a `tree-sitter generate`. Packaging is a follow-up.
