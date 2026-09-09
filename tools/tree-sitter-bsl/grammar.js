/**
 * @file tree-sitter grammar for BSL (the Babylon Scripting Language)
 *
 * DERIVED ARTIFACT — NOT NORMATIVE.
 *
 * This grammar is derived from `docs/reference/bsl.ebnf`, which is part of
 * the ONE normative home for BSL, `docs/reference/bsl-language.rst`, by
 * inclusion (§7). Nothing here declares anything about the language: on any
 * divergence the rst wins, this file is the defect, and the repair is here.
 * It exists for tooling — syntax highlighting, structural editing, code
 * navigation — and for nothing else. It is not a validator: every static
 * check of §3 (types, kinds, scope, ceilings, the fuel bound, the closed
 * vocabulary) lives in the reference implementation at
 * `rust/crates/babylon-bsl`, and a file this grammar parses may still be
 * rejected at load by any of them.
 *
 * Production names track `bsl.ebnf`'s (hyphens become underscores, as
 * tree-sitter rule names must be identifiers).
 *
 * THREE DELIBERATE DEPARTURES, each recorded rather than hidden:
 *
 * 1. `generic_form` — a fallback for any form whose head is a plain symbol
 *    the content grammar does not name. It exists because the tree carries
 *    two s-expression dialects the rst does NOT specify: the `.bscn`
 *    scenario-file format read by `babylon-bsl`'s `scenario.rs`, and §6.1's
 *    `vector` fixture format (which no in-tree file yet uses). Parsing them
 *    as generic forms keeps the editor working without minting a second
 *    source of truth for either.
 * 2. Reserved words. Every §5.2 form-head symbol is a keyword token here. A
 *    binding, `:as` name or payload key spelled exactly like one is legal
 *    BSL (D33 reserves them against the INTRINSIC namespace only) and is
 *    accepted in value positions via the `symbol` rule.
 *
 *    Whether a reserved word may HEAD a `generic_form` is decided by
 *    tree-sitter's context-aware lexing, and the answer differs by position
 *    — stated exactly, because an earlier draft of this comment got it
 *    wrong (Copilot review, PR #485). Where the content grammar admits the
 *    matching form, the keyword token wins and the structured rule applies:
 *    a top-level `(deffield <qname> <type> <kind>)` — the `.bscn` positional
 *    shape — is therefore an ERROR, since §2.9's `deffield` wants `:type`
 *    and `:kind`. Where the content grammar admits no form at all, as
 *    inside a `generic_form` body, the head lexes as a plain symbol and the
 *    fallback takes it: the same positional `deffield` nested in
 *    `(scenario …)` parses clean. That is why the real `.bscn` file parses
 *    and a bare scenario `deffield` would not. See README.md.
 * 3. `type_name` is a lowercase `symbol` — the reading bsl.ebnf collected as
 *    a recorded gap, and the Director ruled in its favour on 2026-08-11
 *    (ADR191 R4, bsl-language.rst D94), so this is now transcription rather
 *    than a departure. Kept in this list because it reads as one.
 *
 * NO `word` TOKEN, deliberately. Declaring `word: $._plain_symbol` turns the
 * reserved words into extracted keywords and is the usual optimization — but
 * with it, a missing `symbol` operand is repaired by the parser into a
 * zero-width node that `tree-sitter parse` does not flag: `(remove-node)`
 * parsed clean. A tool that silently repairs its input is the
 * silent-degradation shape this project rejects (III.11), so the
 * optimization is declined and the error is loud.
 *
 * @license AGPL-3.0-or-later
 */

/* Every §5.2 form-head symbol that is a valid `symbol` (the operator tags
 * cannot be spelled as symbols). Kept as one list so that a value position
 * can accept them all and `generic_form` can reject them all. */
const RESERVED_WORDS = [
  'add', 'add-edge', 'add-hyperedge', 'add-node', 'adjunction', 'anchor',
  'and', 'binding', 'bindings', 'branch', 'ceiling', 'choose', 'deffield',
  'defenum', 'defvocabulary', 'domain',
  'edge-between', 'edges', 'effects', 'emit', 'exists', 'field-of', 'fold',
  'for-each', 'forall', 'guard', 'hyperedges', 'hyperedges-of', 'if',
  'intrinsic', 'manifest', 'member', 'members', 'members-of',
  'membership-field-of', 'metric', 'metric-of', 'neighbors', 'nodes', 'not',
  'or', 'quantize-mass', 'remove-edge', 'remove-hyperedge', 'remove-node',
  'rule', 'rung',
  'scale', 'select-max', 'select-min', 'set', 'sub', 'the', 'update-edge',
  'update-hyperedge', 'update-membership', 'update-node', 'when',
];

/* §2.7 `<fold-op>`, §2.4 `<cmp>`, §2.7 `<arith>` — closed terminal sets. */
const FOLD_OPS = ['sum', 'mean', 'min', 'max', 'count'];
const CMP = ['<', '<=', '>', '>=', '=', '!='];
const ARITH = ['+', '-', '*', '/'];

module.exports = grammar({
  name: 'bsl',

  /* §1.2: whitespace is exactly these four characters, and a comment is
   * whitespace. */
  extras: ($) => [/[ \t\r\n]/, $.comment],

  rules: {
    /* --- §2.2 content files ------------------------------------------ */

    source_file: ($) => repeat($._top_form),

    _top_form: ($) =>
      choice(
        $.rule,
        $.deffield,
        $.intrinsic_decl,
        $.manifest,
        $.metric_decl,
        $.defenum,
        $.defvocabulary,
        $.generic_form,
      ),

    /* --- §2.3 rules --------------------------------------------------- */

    rule: ($) =>
      seq(
        '(',
        'rule',
        field('id', $.qname),
        repeat(
          choice($.rule_role, $.evidence, $.material_basis, $.fuel, $.projects_kernel),
        ),
        optional($.domain),
        optional($.anchor),
        $.bindings,
        optional($.when),
        $.effects,
        ')',
      ),

    /* §2.3: the four valued keyword options may appear in any source order,
     * so they are collected rather than sequenced. Presence and closed-set
     * membership are load checks, not tree-sitter recovery policy. */
    rule_role: (_$) =>
      seq(':role', choice('mechanic', 'recognizer', 'external-event', 'intent')),
    evidence: (_$) =>
      seq(':evidence', choice('observed', 'derived', 'calibrated', 'designed')),
    material_basis: ($) => seq(':material-basis', $.string),
    fuel: ($) => seq(':fuel', $.int_lit),
    projects_kernel: ($) => seq(':projects-kernel', $.qname),

    domain: ($) => seq('(', 'domain', choice($.enum_ref, $.graph_flag), ')'),
    graph_flag: (_$) => ':graph',

    anchor: ($) => seq('(', 'anchor', choice(':after', ':before'), $.symbol, ')'),

    bindings: ($) => seq('(', 'bindings', repeat($.binding), ')'),
    when: ($) => seq('(', 'when', $._cond, ')'),
    effects: ($) => seq('(', 'effects', repeat1($._effect_item), ')'),

    /* --- §2.4 conditions ---------------------------------------------- */

    _cond: ($) =>
      choice(
        $.bool_lit,
        $.and_cond,
        $.or_cond,
        $.not_cond,
        $.comparison,
        $.exists,
        $.forall,
      ),

    and_cond: ($) => seq('(', 'and', repeat1($._cond), ')'),
    or_cond: ($) => seq('(', 'or', repeat1($._cond), ')'),
    not_cond: ($) => seq('(', 'not', $._cond, ')'),
    comparison: ($) => seq('(', $.cmp, $._expr, $._expr, ')'),
    cmp: (_$) => choice(...CMP),

    exists: ($) =>
      seq('(', 'exists', $._query, optional($.elem_name), optional($._cond), ')'),
    forall: ($) =>
      seq('(', 'forall', $._query, optional($.elem_name), $._cond, ')'),

    /* --- §2.5 bindings ------------------------------------------------ */

    binding: ($) => seq('(', 'binding', $.symbol, $.bind_src, repeat($.bind_opt), ')'),

    bind_src: ($) =>
      choice(
        seq(':field', $.qname),
        seq(':const', $.qname),
        seq(':metric', $.symbol),
        ':tick',
        ':year',
        ':tick-of-year',
        seq(':tick-in-cycle', $.int_lit),
        seq(':expr', $._expr),
      ),

    bind_opt: ($) => choice(':optional', seq(':default', $.literal)),

    /* --- §2.6 queries -------------------------------------------------- */

    _query: ($) =>
      choice($.nodes, $.edges, $.neighbors, $.hyperedges, $.members_of, $.hyperedges_of),

    nodes: ($) => seq('(', 'nodes', $.enum_ref, optional($._cond), ')'),
    edges: ($) => seq('(', 'edges', $.enum_ref, optional($._cond), ')'),
    hyperedges: ($) => seq('(', 'hyperedges', $.enum_ref, optional($._cond), ')'),

    /* D51: four operands — element, EdgeType traversed, direction, result
     * NodeType. The fourth is mandatory and it FILTERS. */
    neighbors: ($) =>
      seq('(', 'neighbors', $._expr, $.enum_ref, $.direction, $.enum_ref, ')'),

    members_of: ($) => seq('(', 'members-of', $._expr, $.enum_ref, ')'),
    hyperedges_of: ($) => seq('(', 'hyperedges-of', $._expr, $.enum_ref, ')'),

    direction: (_$) => choice(':out', ':in', ':any'),
    elem_name: ($) => seq(':as', $.symbol),

    /* --- §2.7 expressions ---------------------------------------------- */

    _expr: ($) =>
      choice(
        $.literal,
        $.symbol,
        $.enum_ref,
        $.arith_expr,
        $.if_expr,
        $.quantize_mass,
        $.fold,
        $._accessor,
        $.selection,
        $.intrinsic_call,
      ),

    arith_expr: ($) => seq('(', $.arith, $._expr, $._expr, ')'),
    arith: (_$) => choice(...ARITH),

    if_expr: ($) => seq('(', 'if', $._cond, $._expr, $._expr, ')'),

    quantize_mass: ($) => seq('(', 'quantize-mass', $._expr, ')'),

    fold: ($) =>
      seq(
        '(',
        'fold',
        $.fold_op,
        $._query,
        optional($.elem_name),
        $._expr,
        optional($.weight),
        ')',
      ),
    fold_op: (_$) => choice(...FOLD_OPS),
    weight: ($) => seq(':weight', $._expr),

    selection: ($) =>
      seq(
        '(',
        choice('select-max', 'select-min'),
        $._query,
        optional($.elem_name),
        $._expr,
        ')',
      ),

    /* §2.7: an intrinsic call is an ordinary form whose head is a symbol
     * declared in the intrinsic table. D33 reserves every §5.2 form tag
     * against that namespace, which is exactly why the head here is a plain
     * symbol and never a reserved word. */
    intrinsic_call: ($) =>
      seq('(', alias($._plain_symbol, $.symbol), repeat($._expr), ')'),

    /* --- §2.10 element accessors ---------------------------------------- */

    _accessor: ($) =>
      choice($.field_of, $.edge_between, $.the, $.metric_of, $.membership_field_of),

    field_of: ($) => seq('(', 'field-of', $._expr, $.qname, ')'),
    edge_between: ($) => seq('(', 'edge-between', $.enum_ref, $._expr, $._expr, ')'),
    the: ($) => seq('(', 'the', $.enum_ref, ')'),
    metric_of: ($) => seq('(', 'metric-of', $._expr, $.symbol, ')'),
    membership_field_of: ($) =>
      seq('(', 'membership-field-of', $._expr, $._expr, $.qname, ')'),

    /* --- §2.8 effects --------------------------------------------------- */

    _effect_item: ($) => choice($._verb, $.guard, $.for_each, $.choose),

    guard: ($) => seq('(', 'guard', $._cond, repeat1($._effect_item), ')'),
    for_each: ($) =>
      seq('(', 'for-each', $._query, optional($.elem_name), repeat1($._effect_item), ')'),

    choose: ($) =>
      seq(
        '(',
        'choose',
        ':sample',
        $.qname,
        ':slot',
        $.u32_lit,
        repeat1($.branch),
        ')',
      ),

    branch: ($) =>
      seq('(', 'branch', $.enum_ref, ':mass', $._expr, $.branch_effects, ')'),

    /* A branch may make no material change. The reference loader enforces
     * the AJ context restriction that every nested item is deterministic:
     * no `emit` and no nested `choose`. */
    branch_effects: ($) => seq('(', 'effects', repeat($._effect_item), ')'),

    _verb: ($) =>
      choice(
        $.update_node,
        $.update_edge,
        $.add_node,
        $.remove_node,
        $.add_edge,
        $.remove_edge,
        $.add_hyperedge,
        $.update_hyperedge,
        $.remove_hyperedge,
        $.update_membership,
        $.emit,
      ),

    update_node: ($) => seq('(', 'update-node', $._expr, $.qname, $.update_op, ')'),
    update_edge: ($) => seq('(', 'update-edge', $._expr, $.qname, $.update_op, ')'),
    update_hyperedge: ($) =>
      seq('(', 'update-hyperedge', $._expr, $.qname, $.update_op, ')'),
    update_membership: ($) =>
      seq('(', 'update-membership', $._expr, $._expr, $.qname, $.update_op, ')'),

    add_node: ($) => seq('(', 'add-node', $.enum_ref, $._expr, repeat($.field_init), ')'),
    remove_node: ($) => seq('(', 'remove-node', $._expr, ')'),

    add_edge: ($) =>
      seq(
        '(',
        'add-edge',
        $.enum_ref,
        $._expr,
        $._expr,
        $.strength,
        repeat($.field_init),
        ')',
      ),
    strength: ($) => seq(':strength', $._expr),
    remove_edge: ($) => seq('(', 'remove-edge', $.enum_ref, $._expr, $._expr, ')'),

    add_hyperedge: ($) =>
      seq('(', 'add-hyperedge', $.enum_ref, $._expr, $.members, repeat($.field_init), ')'),
    remove_hyperedge: ($) => seq('(', 'remove-hyperedge', $._expr, ')'),

    emit: ($) => seq('(', 'emit', $.enum_ref, repeat($.payload_item), ')'),

    update_op: ($) => seq('(', choice('add', 'sub', 'set', 'scale'), $._expr, ')'),

    members: ($) => seq('(', 'members', repeat1($._member_item), ')'),
    _member_item: ($) => choice($._expr, $.member),
    member: ($) => seq('(', 'member', $.enum_ref, $._expr, repeat($.field_init), ')'),

    field_init: ($) => seq('(', $.qname, $._expr, ')'),
    payload_item: ($) => seq('(', $.symbol, $._expr, ')'),

    /* --- §2.9 / §2.11 declarations --------------------------------------- */

    deffield: ($) =>
      seq(
        '(',
        'deffield',
        $.qname,
        ':type',
        $.type_name,
        ':kind',
        $.field_kind,
        optional(seq(':member', $.enum_ref)),
        ')',
      ),
    field_kind: (_$) => choice('intensive', 'extensive'),
    type_name: ($) => $.symbol,

    intrinsic_decl: ($) =>
      seq(
        '(',
        'intrinsic',
        $.symbol,
        ':params',
        seq('(', repeat($.type_name), ')'),
        ':returns',
        $.type_name,
        ':cost',
        $.int_lit,
        ')',
      ),

    manifest: ($) =>
      seq(
        '(',
        'manifest',
        $.symbol,
        repeat1($.ceiling),
        repeat($.rung),
        repeat($.adjunction),
        ')',
      ),

    ceiling: ($) =>
      seq(
        '(',
        'ceiling',
        $.enum_ref,
        ':ceiling',
        $.int_lit,
        optional(seq(':max-members', $.int_lit)),
        optional(':invariant'),
        ')',
      ),

    rung: ($) =>
      seq(
        '(',
        'rung',
        $.symbol,
        $.enum_ref,
        $.enum_ref,
        ':via',
        $.enum_ref,
        optional(':substrate'),
        ')',
      ),

    adjunction: ($) =>
      seq(
        '(',
        'adjunction',
        $.symbol,
        $.qname,
        $.qname,
        ':rung',
        $.symbol,
        optional(seq(':weighted-by', $.qname)),
        ')',
      ),

    metric_decl: ($) =>
      seq(
        '(',
        'metric',
        $.symbol,
        ':type',
        $.type_name,
        ':kind',
        $.field_kind,
        $.domain,
        ':provider',
        $.symbol,
        ')',
      ),

    /* §2.13 (Organization spec §1 Q12, D101). */
    defenum: ($) =>
      seq(
        '(',
        'defenum',
        $.enum_type,
        seq('(', repeat1($.enum_member), ')'),
        ')',
      ),

    defvocabulary: ($) =>
      seq(
        '(',
        'defvocabulary',
        $.enum_type,
        seq('(', repeat1($.enum_member), ')'),
        ')',
      ),

    /* --- the fallback (see the file header, departure 1) ----------------- */

    generic_form: ($) =>
      seq(
        '(',
        choice(alias($._plain_symbol, $.symbol), $.qname),
        repeat($._generic_item),
        ')',
      ),

    _generic_item: ($) =>
      choice(
        $.literal,
        $.string,
        $.symbol,
        $.qname,
        $.enum_ref,
        $.keyword,
        $.rule,
        $.generic_form,
      ),

    /* --- §1.4 / §1.5 atoms ------------------------------------------------ */

    literal: ($) => choice($.int_lit, $.scaled_lit, $.mass_lit, $.bool_lit),

    /* A symbol in VALUE position may be spelled like a §5.2 form tag: D33
     * reserves those names against the intrinsic namespace only. */
    symbol: ($) => choice($._plain_symbol, ...RESERVED_WORDS),
    _plain_symbol: (_$) => /[a-z][a-z0-9-]*/,

    qname: (_$) => token(/[a-z][a-z0-9-]*(\/[a-z][a-z0-9-]*)+/),

    enum_ref: (_$) => token(/[A-Z][A-Za-z0-9]*\/[A-Z][A-Z0-9_]*/),

    /* §2.13 (Organization spec §1 Q12, D101): `defenum`'s type name and
     * `defvocabulary`'s <enum-kind> operand are bare — no `/MEMBER` — so
     * they need their own tokens rather than reusing `enum_ref`, whose
     * regex is the whole `Type/MEMBER` pair as ONE atomic token. These
     * two mirror `enum_ref`'s two halves exactly (bsl.ebnf's `enum-type`
     * and `enum-member` productions, §1.4), split apart because §2.13 is
     * the first construct that needs either half standalone. */
    enum_type: (_$) => token(/[A-Z][A-Za-z0-9]*/),
    enum_member: (_$) => token(/[A-Z][A-Z0-9_]*/),

    bool_lit: (_$) => choice('#t', '#f'),

    int_lit: (_$) => token(/-?[0-9](_?[0-9])*/),

    /* AJ's slot is syntactically nonnegative; the u32 upper bound is a
     * reference-loader check. */
    u32_lit: (_$) => token(/[0-9](_?[0-9])*/),

    /* §1.5: the kind suffix is mandatory; a bare `0.5` is E-LEX-021 and is
     * therefore not a token of this grammar at all. `r` (Ratio) joins
     * `$`/`p`/`i`/`c` per the §1.5 addendum (D99, #492/ADR194) — this
     * grammar does not distinguish a literal's DOMAIN (E-LEX-024/027 are
     * lex-time domain checks the reference reader makes semantically, not
     * something this tokenizer can express), only its lexical SHAPE. */
    scaled_lit: (_$) => token(/-?[0-9](_?[0-9])*(\.[0-9](_?[0-9])*)?[$picr]/),

    /* AJ Mass is its own nonnegative exact literal class, not another
     * binary64 scalar lane. Scale and u64-nanounit bounds are checked by the
     * reference reader. */
    mass_lit: (_$) => token(/[0-9](_?[0-9])*(\.[0-9](_?[0-9])*)?m/),

    /* §1.5: the only four escapes; strings are single-line. */
    string: (_$) => token(/"([^"\\\n]|\\["\\nt])*"/),

    /* Any colon-prefixed symbol, for the fallback dialects only — the
     * content grammar spells each of its keywords literally (§1.6's set is
     * closed, and an unrecognized keyword is E-PARSE-013, never ignored). */
    keyword: (_$) => token(/:[a-z][a-z0-9-]*/),

    comment: (_$) => token(seq(';', /[^\n]*/)),
  },
});
