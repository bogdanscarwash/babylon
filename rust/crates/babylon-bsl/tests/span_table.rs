//! Task 1 (#652 bsl-ls, PR A) — the span side-table's public contract
//! (plan §2, Task 1.1(b)). `read_all_spanned`/`read_spanned` are new entry
//! points beside `read_all`/`read` (Charter decision 1, plan §2): same ONE
//! parser, plus a `SpanTable` of pre-order `(FormPath, Span)` entries.
//! These tests exercise only the PUBLIC contract — no access to
//! `reader.rs`'s internal stack shape.
use babylon_bsl::reader::{read_all_spanned, SpanTable};

/// The plan's own fixture (§2, Task 1.1(b)): two top-level `rule` forms,
/// each `(rule <qname>)`.
const TWO_RULES: &str = "(rule demo/x)\n(rule demo/y)";

fn table_for(src: &str) -> SpanTable {
    read_all_spanned(src.as_bytes())
        .expect("fixture source must parse")
        .1
}

#[test]
fn span_of_the_first_top_level_form_is_its_full_paren_range() {
    let table = table_for(TWO_RULES);
    let span = table.span_of(&[0]).expect("path [0] must have an entry");
    assert_eq!(&TWO_RULES[span.start..span.end], "(rule demo/x)");
}

#[test]
fn span_of_the_second_forms_qname_child_is_its_own_atom_range() {
    let table = table_for(TWO_RULES);
    let span = table
        .span_of(&[1, 1])
        .expect("path [1, 1] must have an entry");
    assert_eq!(&TWO_RULES[span.start..span.end], "demo/y");
}

#[test]
fn innermost_at_inside_demo_y_resolves_to_its_leaf_path() {
    let table = table_for(TWO_RULES);
    // An offset strictly inside the "demo/y" token, not on either boundary.
    let token_start = TWO_RULES.rfind("demo/y").expect("fixture contains demo/y");
    let offset = token_start + 2;
    let (path, span) = table
        .innermost_at(offset)
        .expect("offset sits inside a token");
    assert_eq!(path, &[1, 1]);
    assert_eq!(&TWO_RULES[span.start..span.end], "demo/y");
}

#[test]
fn a_leading_bom_shifts_every_span_by_exactly_its_byte_width() {
    let shift = '\u{feff}'.len_utf8();
    assert_eq!(shift, 3, "a BOM is 3 bytes in UTF-8");
    let bommed = format!("\u{feff}{TWO_RULES}");

    let plain = table_for(TWO_RULES);
    let (_, bom_table) =
        read_all_spanned(bommed.as_bytes()).expect("BOM-prefixed source must parse");

    let plain_span = plain.span_of(&[1, 1]).expect("plain path [1, 1]");
    let bom_span = bom_table.span_of(&[1, 1]).expect("bommed path [1, 1]");
    assert_eq!(bom_span.start, plain_span.start + shift);
    assert_eq!(bom_span.end, plain_span.end + shift);
    assert_eq!(&bommed[bom_span.start..bom_span.end], "demo/y");
}
