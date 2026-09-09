;; Syntax highlighting for BSL — DERIVED, NON-NORMATIVE.
;;
;; docs/reference/bsl-language.rst is the one normative home; this file
;; colours what ../grammar.js parses and declares nothing about the
;; language. Capture names follow the tree-sitter/nvim-treesitter standard
;; set so the queries drop into any host that speaks it.
;;
;; The colouring intent, stated once: a reader should be able to see the
;; three things BSL makes load-bearing — WHICH FORM this is (the head), WHAT
;; IT READS (qnames, enum-refs, metric and binding names), and WHICH
;; QUANTITY KIND a literal carries (§1.5's four suffixes are the lexical
;; enforcement of the no-bare-floats rule, so they are highlighted as a
;; class of their own rather than as generic numbers).

;; ── Form heads ──────────────────────────────────────────────────────────
;; §2.2's top-level declaration forms: the content set's own structure.
[
  "rule"
  "deffield"
  "intrinsic"
  "manifest"
  "metric"
] @keyword

;; §2.3 rule children and §2.9's manifest children.
[
  "bindings"
  "binding"
  "when"
  "effects"
  "domain"
  "anchor"
  "ceiling"
  "rung"
  "adjunction"
] @keyword

;; §2.4 / §2.7 control and aggregation forms.
[
  "and"
  "or"
  "not"
  "if"
  "exists"
  "forall"
  "fold"
  "select-max"
  "select-min"
  "guard"
  "for-each"
] @keyword.control

;; §2.6 query heads — every one of them ranges over the graph.
[
  "nodes"
  "edges"
  "neighbors"
  "hyperedges"
  "members-of"
  "hyperedges-of"
] @function.builtin

;; §2.10 accessors — reads from an element already in hand.
[
  "field-of"
  "edge-between"
  "the"
  "metric-of"
  "membership-field-of"
] @function.builtin

;; §2.8 structural verbs — the ONLY writes in the language.
[
  "update-node"
  "update-edge"
  "update-hyperedge"
  "update-membership"
  "add-node"
  "remove-node"
  "add-edge"
  "remove-edge"
  "add-hyperedge"
  "remove-hyperedge"
  "emit"
  "members"
  "member"
] @function.method

;; §2.8's four update operations, §2.7's closed fold-op set, §2.9's kinds.
[
  "add"
  "sub"
  "set"
  "scale"
] @operator
(fold_op) @function.builtin
(field_kind) @type.builtin

;; §2.4 comparisons and §2.7 arithmetic.
(cmp) @operator
(arith) @operator

;; ── Keywords (the `:name` option markers, §1.6's closed set) ────────────
[
  ":material-basis"
  ":role"
  ":evidence"
  ":fuel"
  ":field"
  ":const"
  ":metric"
  ":expr"
  ":optional"
  ":default"
  ":tick"
  ":year"
  ":tick-of-year"
  ":tick-in-cycle"
  ":kind"
  ":type"
  ":member"
  ":as"
  ":weight"
  ":strength"
  ":after"
  ":before"
  ":params"
  ":returns"
  ":cost"
  ":provider"
  ":ceiling"
  ":max-members"
  ":invariant"
  ":via"
  ":substrate"
  ":rung"
  ":weighted-by"
] @attribute
(direction) @attribute
(graph_flag) @attribute
(keyword) @attribute

;; ── Names ───────────────────────────────────────────────────────────────
;; A qname is a rule id or a field reference — always a name in the closed
;; vocabulary (§3.6), never a free identifier.
(qname) @variable.member

;; An enum-ref names a member of a closed enum; the type half and the member
;; half are one token by §1.4, so the whole reference is one capture.
(enum_ref) @constant

(type_name (symbol) @type)

;; The generic case first: later patterns override earlier ones in the
;; hosts that consume these queries, so the specific captures below win.
(symbol) @variable

;; The reserved element references (§2.5): `self` is the rule's subject and
;; `it` the innermost enclosing iterating form's element (D53). Neither is
;; ever declared, which is why they are `variable.builtin` and not
;; `variable`.
((symbol) @variable.builtin
  (#any-of? @variable.builtin "self" "it"))

(binding (symbol) @variable.parameter)
(elem_name (symbol) @variable.parameter)
(intrinsic_call (symbol) @function.builtin)
(payload_item (symbol) @property)

;; ── Literals ────────────────────────────────────────────────────────────
;; §1.5: a non-integer literal MUST carry a kind suffix ($ / p / i / c), so
;; a scaled literal is a different thing from a plain integer and reads as
;; one.
(scaled_lit) @number.float
(int_lit) @number
(bool_lit) @constant.builtin
(string) @string

;; ── Punctuation and trivia ──────────────────────────────────────────────
[
  "("
  ")"
] @punctuation.bracket

(comment) @comment @spell
