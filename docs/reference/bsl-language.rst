BSL Language Reference (Program 27)
=====================================

The normative, language-agnostic specification of the **Babylon Scripting
Language (BSL)** — the deterministic, total, fuel-metered s-expression DSL that
Program 27 makes the substrate for game rules.

**Status.** Phase-1 draft, produced under
``docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`` §10
("Phase 1 — Language & Kernel: the BSL specification as the language-agnostic
reference"). The design document is Director-approved; **this document is not
code authorization**. The v3.0.0 amendment (design §4) that Phase 1 was gated
on was **ratified 2026-07-29** — ``CONSTITUTION.md`` v3.0.0, Amendment AE (The
Refoundation), PR #365 — so that gate is closed; this document remains the
reference, not the implementation.

**Amendment D — ruled, and this document revised.** The first revision was
written against the *dyadic working assumption*, with §2.6 (queries), §2.8
(structural verbs) and §3.7 (the edge-side ceiling) flagged as revisable if
Phase 0 ruled otherwise. On **2026-07-29 the Director ruled NATIVE
HYPEREDGE**: hyperedges are **first-class objects** in ``babylon-graph``'s
exposed model and type system, membership is a single typed hyperedge — never
a clique expansion, and never *exposed* as a bipartite incidence encoding
(Levi/incidence is sanctioned as an internal storage strategy only). Source:
Amendment AE clause (vi) (``CONSTITUTION.md`` v3.0.0) recording the ruling in
``ai/_inbox/amendment-d-analysis-p27.md`` §9 (PR #353), sub-rulings D-1…D-7.
Those three sections carry the hyperedge shape as of this revision. The dyadic
query forms and edge verbs are **unchanged and coexist** with it: II.9's
morphism layer stays strictly dyadic, and the two layers are separated by
*type* inside one substrate (sub-ruling D-2 — one substrate, typed homes).

**Third revision — the R9 gap-fill chapters (2026-08-10).** Twelve of the
seventeen systems targeted at pure BSL rule packs carried at least one hard
language blocker: they were BSL-shaped and unauthorable. Thirteen chapters
(C1–C13), planned in ``reports/bsl-gap-analysis-2026-08-10.md`` §7, close them
in place across §§1.6, 2.2–2.11, 3.1, 3.4, 3.6–3.10, 4.2, 4.5–4.7, 5.2, 5.3,
5.5 and 6.2, adding rows **D29–D71** to the register below and vector families
**10–22** to §6.2. One planned item is **not** closed and is recorded as such:
edge-endpoint accessors (gap item Q2), §3.8 item 8 and D78. A three-lens
adversarial verification of the chapters (2026-08-10) added rows **D72–D78**,
which repair what it found: query-result multiplicity, the edge-key uniqueness
obligation at hydration, the enum-ref operand class rule, error-code hygiene,
manifest completeness, and the fuel meter's scope under per-subject firing.

The chapters were held to a stated reach: **query forms, bindings,
iteration and selection structure, and graph-scope state are licensed**
(Amendment AE clause (ii), which re-opens the formalism surface for BSL and for
nothing else), while a new generator, constructor, adjunction, level lattice or
severity rule is not — and neither is an addition to the intrinsic table beyond
``{exp, log}``. Two items reached that boundary and were **escalated rather
than specced**: per-membership hyperedge payload (§2.8's note — a change to the
exposed hyperedge model Amendment AE clause (vi) ruled) and the minting of new
scale-lattice rungs or adjunctions (§3.9's note). **Amendment AG has since
ruled both** (``CONSTITUTION.md`` v3.2.0, ADR189, ratified 2026-08-10), and the
fourth revision below converts the two notes from escalations into
specifications. The rider slate in §3.10 is
recorded as proposals and declares nothing. **This revision is additive**: no
form changes meaning, and §5.6's canonical bytes and both its digests are
unchanged — the one deliberate exception is ``neighbors``, which gains a
mandatory operand (D51) against evidence that no conformance vector exercises
it.

**Fourth revision — the Amendment AG spec sections (2026-08-10).** Amendment AG
(``CONSTITUTION.md`` v3.2.0, ADR189) ruled the two items the third revision
escalated and obliges this document to spec them (clause (iv)). Clause (i)
makes the *(member, hyperedge)* incidence pair a first-class **attributed
membership** carrying declared, typed payload fields; §2.12 states what that
element kind is, and its three language surfaces land in the sections that own
them — the declaration in §2.9 (``deffield``'s ``:member`` operand), the read
in §2.10 (``membership-field-of``), the write in §2.8
(``update-membership``, and ``add-hyperedge``'s annotated member items). Rows
**D79–D84** record the decisions and §6.2 family **23** pins them. Clause (ii)
lets content **declare** scale-lattice rungs and ``allocate``/``aggregate``
adjunction *instances* of the existing schema: the declaration forms join the
``manifest`` in §2.9 and their load-time validation — including the two
standing rulings the amendment binds every rung to — is §3.9's, with rows
**D85–D89** and family **24**. A four-lens adversarial verification of these
sections added row **D90**, which repairs what it found: §3.4's table stated a
result kind for four of its five fold rows and left the weighted intensive
``mean`` blank, and §2.12's worked shape had folded ``sum`` over exactly that
value. A post-merge audit added row **D91**, scoping §6.1's vector-file
format outside §5's canonical encoding (the ``:graph`` flag-vs-valued
spelling collision found by the PR #480 re-verification). This
revision is additive on the same terms as the third: no form changes meaning,
§5.6's canonical bytes and both its digests are unchanged, and the intrinsic
table is untouched.

**Fifth revision — the consolidated grammar (2026-08-10).** §7 collects every
production of §§1, 2 and 6.1 into ``docs/reference/bsl.ebnf`` and includes it
here, adding rows **D92** and **D93** and a *rigor index* (§7.1) that says
where each artifact of the language's rigor lives. The collection **adds no
form and changes no meaning**: it states one EBNF dialect, records in comments
the context conditions a context-free grammar cannot carry, and where the
sections are silent or disagree with themselves it **collects the reference
implementation's reading and flags it** — nine of its productions come from
prose rather than from a code block, so the appendix does choose where it must,
and says which choice it made and what cuts against it. §5.6's bytes and both
its digests are again unchanged.

**Standard this document is written to.** Constitution III.12(a) — the *rewrite
test*: two independent implementations (the Rust ``babylon-bsl`` crate now,
anything later) must be derivable from this document alone, without reading
either implementation's source. Every under-determined point in the design
document is decided here and marked **[draft ruling — Phase 1 review]**; the
complete list is collected in *Draft-Ruling Register* at the end, and each row
is a Phase-1 review item, not a settled law.

**Boundary with the determinism contract.** This document owns the *language*:
lexis, grammar, typing, evaluation, fuel, and the canonical AST byte layout that
``rules_hash`` is computed over. It does **not** restate the tick hash,
``defines_hash``, or the ``ContentDigest`` composition — those live in
:doc:`/reference/determinism-contract`, which Program 27 Phase 0 extends with
three new chapters (tick-hash field set, ``rules_hash``'s place in
``ContentDigest``, and the ``ContentDigest`` byte layout). Where this document
says "``rules_hash``", it defines *what bytes are hashed*; the determinism
contract defines *what that hash is combined with and compared against*.

.. contents:: Contents
   :local:
   :depth: 2

Amendment AJ finite probability addendum
------------------------------------------

**Status: ratified and normative, 2026-09-01.** Constitution v4.1.0 and
ADR248 add finite material transition kernels. This addendum is the current
language law where older sections below describe ``rng-draw`` as
author-visible, close the suffix set without ``m``, or say BSL has exactly two
numeric lanes. Historical text remains readable, but it grants no alternate
path.

AJ keeps four categories separate:

``Mass``
   Exact, nonnegative allocation input for one finite kernel.

``Probability``
   The existing bounded binary64 material scalar.

``Intensity``
   A bounded state measure.

dialectical weight
   The ``w`` of ``D = (A, Ā, w, T, σ)``.

No implicit conversion joins them. A kernel is subordinate to the dialectic
and ranges over material effect bundles, not events or outcomes of history.

Lexis and surface grammar
~~~~~~~~~~~~~~~~~~~~~~~~~

The scaled-literal suffix production gains ``m``:

.. code-block:: text

   suffix      ::= "$" | "p" | "i" | "c" | "r" | "m"
   mass-lit    ::= digits ( "." digits )? "m"

.. vale Vale.Spelling = NO

An ``m`` literal has static type ``Mass`` and cannot be negative. It permits at
most nine fractional decimal digits. The reader converts it without loss to an
unsigned nanounit integer:

.. vale Vale.Spelling = YES

.. code-block:: text

   mass_units = unscaled * 10^(9 - scale)

Leading zeros and digit separators follow the other numeric literals. A
negative literal, excess scale, or value above ``u64::MAX`` nanounits refuses
at read time. Equivalent spellings such as ``1m``, ``1.0m``, and
``1.000000000m`` have the same canonical value and bytes. No read-time
rounding occurs.

Expressions gain one built-in language form, not an intrinsic:

.. code-block:: text

   <expr> ::= ...
            | "(" "quantize-mass" <expr> ")"

``quantize-mass`` accepts an ordinary numeric expression in the binary64 lane.
It refuses a negative, nonfinite, or overflowing result. It multiplies the
exact represented binary64 value by ``10^9`` and rounds to the nearest integer,
breaking an exact tie toward the even integer. It returns ``Mass``. This is the
only dynamic conversion to ``Mass``. No implicit conversion exists from
``Probability``, ``Intensity``, ``Coefficient``, ``Real``, ``Ratio``,
``Currency``, or ``Int``.

The rule and effect grammar gains the following forms:

.. code-block:: text

   <u32-lit> ::= <digits>
   <rule> ::= "(" "rule" <qname>
                  ":role" <rule-role>
                  ":evidence" <evidence-class>
                  ":material-basis" <string>
                  ":fuel" <int-lit>
                  ( ":projects-kernel" <qname> )?
                  <domain>? <anchor>? <bindings> <when>? <effects> ")"

   <effect-item> ::= ... | <choose>
   <choose> ::= "(" "choose" ":sample" <qname> ":slot" <u32-lit>
                    <branch>+ ")"
   <branch> ::= "(" "branch" <enum-ref> ":mass" <mass-expr>
                    "(" "effects" <deterministic-effect-item>* ")" ")"

``<u32-lit>`` is a nonnegative integer literal no greater than
``4294967295``. ``<mass-expr>`` is a ``Mass`` literal, binding, constant,
checked addition or subtraction, or ``quantize-mass`` result.
``<deterministic-effect-item>`` is an existing bounded Mechanic effect item
whose complete tree contains neither ``choose`` nor ``emit``.

Mass semantics
~~~~~~~~~~~~~~

``Mass`` is deliberately non-storable. It can occur only as a literal, a
constant, a binding, a branch mass, the result of ``quantize-mass``, or an
operand or result of checked ``Mass + Mass`` and ``Mass - Mass``. Subtraction
below zero refuses. Addition above ``u64::MAX`` also refuses.

``Mass`` cannot be a ``deffield`` or metric type. It cannot be a graph value,
update value, or event-payload value. It cannot be a fold accumulator,
comparison operand,
intrinsic parameter or result, or serialized campaign scalar. Multiplication
and division on ``Mass`` do not exist.

``Mass`` is not normalized at authoring time. Branch masses need not sum to
``1m``. Their ratios set the executable measure. This permits exact
relative weights without conflating mass with the existing ``Probability``
scalar.

Finite kernel static semantics
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

A loaded ``choose`` compiles once into ``FiniteKernelV1`` with
``KernelBranchV1`` children. The typed structures keep their source path,
form path, and spans. Evaluation, forecasting, diagnostics, and tooling use
that structure. These consumers do not rediscover kernels by independently
walking raw s-expressions.

The complete content set must meet these conditions:

* Only a ``mechanic`` rule can contain ``choose``.
* A rule has at most one ``choose``. It must be a direct child of that rule's
   top-level ``effects`` form, never under ``guard``, ``for-each``, or another
   ``choose``.
* ``:sample`` is a QName unique across the whole content set. ``:slot`` is a
   literal ``u32``. Once shipped, a slot remains assigned to that semantic
   choice. Later samples use a new slot rather than renumbering an old one.
* All branches name members of one declared enum. They contain every member
   exactly once and in the enum declaration order. Source order becomes the
   canonical outcome order.
* Each branch independently passes ordinary Mechanic authority, type,
  footprint, structural, and fuel checks. A branch cannot contain ``emit`` or
  another ``choose`` at any depth.
* A kernel Mechanic paired with a finite projection keeps all branch and
  sibling material writes carrier-local. Only ``update-node self`` is legal.
  A cross-node, edge, hyperedge, or graph-shape write requires one joint
  carrier kernel and refuses V1 enumeration.
* The finite kernel grants no stochastic authority to ``recognizer``,
   ``external-event``, or ``intent``. Those roles remain deterministic and
   exact-allowlist.

One material cause uses one kernel over its joint carrier. The language does
not infer independence and has no tensor, parallel-choice, or
independent-sample declaration.

Evaluation and ticket allocation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

For one firing, every branch mass evaluates in enum order before a draw. No
branch body evaluates during that step. A zero total, arithmetic overflow, or
invalid dynamic quantization aborts the tick before material effects.

.. vale Vale.Spelling = NO

Let ``m_i`` be the exact nanounits for branch ``i``, ``M = sum(m_i)``, and
``T = 2^64``. Allocation uses exact wide integer arithmetic:

.. vale Vale.Spelling = YES

.. code-block:: text

   floor_i     = floor(m_i * T / M)
   remainder_i = (m_i * T) mod M
   left        = T - sum(floor_i)

The allocator awards one more ticket to the ``left`` branches with the largest
``remainder_i`` values. The enum order breaks ties between equal values. A
positive mass that would receive zero tickets refuses. A zero mass can own an
empty interval. The enum-ordered, half-open intervals partition
``[0, 2^64)`` exactly. Counts and endpoints use an integer representation that
can express ``2^64``.

Realization calls ``KernelRng::next_u64`` exactly once and selects the interval
containing that ticket. Only the selected branch body then evaluates. Its
effects retain their ordinary source order. A no-change branch still realizes
an outcome and produces a choice receipt.

The evaluator charges fuel conservatively and independently of the selected
outcome:

.. code-block:: text

   cost(choose) = draw_cost
                + sum(cost(branch_mass_i))
                + max(cost(branch_body_i))

This keeps the load-time bound valid without evaluating unselected branch
bodies.

Finite event projections
~~~~~~~~~~~~~~~~~~~~~~~~

``:projects-kernel <sample-qname>`` is legal only on an ordinary
``recognizer`` rule. It compiles into ``FiniteProjectionV1``. The referenced
kernel must resolve immediately before that recognizer in schedule order. The
recognizer must be subject-local, deterministic, and emit-only. It cannot write
a latch, mutate graph shape, inspect another sample, or contain a
``choose``. Its existing restricted-role event allowance remains mandatory.

Forecasting evaluates exact alternatives, not random samples. For each branch,
the forecaster clones the relevant detached state and applies that branch's
deterministic effects. It runs the real linked recognizer and records whether
the requested event appears. It sums the favorable branch ticket counts. The
public ``EventLikelihoodV1`` result contains event type, enum-ordered favorable
outcomes, an exact numerator in ``[0, 2^64]``, and the fixed denominator
``2^64``.

The read-only ``analyze_content_set`` and ``forecast_event_likelihoods``
boundaries refuse a recognizer outside this exact projection shape. They do not
approximate arbitrary recognizers, cross-sample joins, sequences,
conjunctions, payload distributions, or whole ticks. Where state does not
yield exact branch masses, tooling reports that the likelihood is
state-dependent rather than inventing a number.

Canonical identity and deliberate absences
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``FiniteKernelV1``, its branches, exact mass atoms, ``quantize-mass``, and
``FiniteProjectionV1`` enter canonical BSL serialization with explicit
versioned tags, lengths, and enum order. File boundaries do not affect their
bytes. The implementation's cross-language vectors own the exact tag values
and asymmetric byte examples.

The author-visible ``rng-draw`` intrinsic is removed. No content declaration,
binding, guard, or effect may call it. ``KernelRng`` remains private to finite
kernel realization. BSL has no stochastic recognizer, stochastic event,
empirical-distribution sampler, universal sigmoid, authored event probability,
or terminal-outcome branch. Those absences enforce Amendment AJ rather than
postpone a second stochastic API.

Design intent (non-normative)
-------------------------------

BSL replaces three disjoint substrates that exist in the Python engine today:

- the doctrine trap-condition string DSL
  (``src/babylon/domain/doctrine/mechanics.py``, a recursive-descent evaluator
  over ``OR/AND/NOT``, six comparisons, ``TAG``/``PRACTICE`` variables, and
  ``@coeff`` references);
- the flat Pydantic event-precondition tree
  (``src/babylon/models/entities/event_template.py`` +
  ``src/babylon/engine/event_evaluator.py``: ``NodeCondition`` with a
  dot-notation path, a node filter and a seven-member aggregation;
  ``EdgeCondition``; ``GraphCondition`` over six named graph metrics; an
  ``all``/``any`` combinator);
- the four-operation effect enum (``increase``/``decrease``/``set``/
  ``multiply`` in ``TemplateEffect``).

BSL is a superset of the two condition grammars' **expressible sets**, not of
their failure semantics (see *Grammar-superset honesty* under *Conformance*).
Rules are homoiconic data: stored as content files, diffed in PRs, inspectable
in-game, rewritable by tools, and hashed as declared input.

1. Lexical grammar
--------------------

1.1 Source text
~~~~~~~~~~~~~~~~~

A BSL source file is a sequence of Unicode scalar values encoded as UTF-8.
A byte sequence that is not valid UTF-8 is a load error (``E-LEX-001``).
A UTF-8 BOM at offset 0 is accepted and discarded; a BOM anywhere else is
``E-LEX-001``.

**[draft ruling — Phase 1 review]** String literals must be in Unicode
Normalization Form C. A non-NFC string literal is ``E-LEX-002``. This is what
makes "the source bytes of the string" a canonical form (§5.4) rather than an
implementation accident; it is checked at load, not normalized silently.

1.2 Whitespace and comments
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Whitespace is exactly ``U+0020`` (space), ``U+0009`` (tab), ``U+000A`` (LF) and
``U+000D`` (CR). No other character is whitespace; in particular ``U+000C`` and
Unicode space separators are not. Whitespace separates tokens and is otherwise
insignificant.

A comment begins with ``;`` outside a string literal and extends to the next LF
or to end of file. Comments are whitespace.

There are no block comments and no reader macros. **[draft ruling — Phase 1
review]**: rejected because both create formatting/hash coupling the canonical
serialization would then have to define away.

1.3 Delimiters
~~~~~~~~~~~~~~~~

``(`` and ``)`` are the only structural delimiters. There are no square
brackets, no quote/quasiquote reader syntax, no dotted pairs, and no vectors.
Every non-atomic construct is a parenthesised **form** whose first element is a
symbol naming the form.

The Rust reader accepts at most 512 simultaneously open lists. The 513th open
list is a positioned, uncoded structural read error. This safety limit applies
before the reader constructs the recursively owned syntax tree. Semantic load
walkers apply a lower 256-level rule limit, so in-bound parsing does not grant
unbounded semantic work.

1.4 Atoms
~~~~~~~~~~~

Lexical productions are given in the same BNF notation as §2 (never as a
regular-expression dialect — dialects differ across implementations). The
character terminals are:

.. code-block:: text

   DIGIT   ::= "0" … "9"
   LOWER   ::= "a" … "z"
   UPPER   ::= "A" … "Z"

The atom classes:

.. code-block:: text

   symbol      ::= LOWER ( LOWER | DIGIT | "-" )*
   qname       ::= symbol ( "/" symbol )+
   keyword     ::= ":" symbol
   enum-ref    ::= enum-type "/" enum-member
   enum-type   ::= UPPER ( UPPER | LOWER | DIGIT )*
   enum-member ::= UPPER ( UPPER | DIGIT | "_" )*
   bool-lit    ::= "#t" | "#f"
   digits      ::= DIGIT ( "_"? DIGIT )*
   int-lit     ::= "-"? digits
   scaled-lit  ::= "-"? digits ( "." digits )? suffix
   suffix      ::= "$" | "p" | "i" | "c"
   string      ::= '"' string-char* '"'
   string-char ::= any scalar value except '"', "\" and LF
                 | "\\" | '\"' | "\n" | "\t"

.. list-table::
   :header-rows: 1
   :widths: 22 78

   * - Atom class
     - Notes
   * - ``symbol``
     - Form heads, binding names, verb names, intrinsic names, fold operators.
       Lowercase kebab-case only. Maximum length 64 (``E-LEX-010``).
   * - ``qname``
     - Rule ids (``vitality/starvation-mortality``) and field references
       (``social-class/wealth``). Exactly the segment alphabet of ``symbol``,
       joined by ``/``. Maximum 4 segments, 128 bytes total (``E-LEX-011``).
   * - ``keyword``
     - See §1.6.
   * - ``enum-ref``
     - ``NodeType/SOCIAL_CLASS``, ``EdgeType/SOLIDARITY``,
       ``HyperedgeType/ECONOMIC_SECTOR``,
       ``DoctrineTag/CLASS_ANALYSIS``, ``PracticeVariable/CO_OPTIVE_SHARE``.
       The member is the **enum member identifier**, never its serialized
       value: ``NodeType/SOCIAL_CLASS``, never ``NodeType/social_class``.
   * - ``bool-lit``
     - The only two boolean tokens. ``true``/``false`` are ordinary symbols and
       are **not** booleans.
   * - ``int-lit``
     - See §1.5.
   * - ``scaled-lit``
     - See §1.5.
   * - ``string``
     - See §1.5.

**[draft ruling — Phase 1 review, implementation-discovered 2026-07-30]**
The ten operator tokens ``<``, ``<=``, ``>``, ``>=``, ``=``, ``!=``, ``+``,
``-``, ``*``, ``/`` form a distinct atom class, ``operator``, lexed by exact
match against this closed set. §2 quotes them as terminals and §5.2 lists
them as form tags, but this table omitted them — by its letter the reader
had to reject ``(< a b)`` and §5.6's own worked example. An ``operator`` is
valid only in form-head position; CAS encodes it as a form tag, never as an
atom (an operator anywhere else is unencodable and fails loudly per §5.4).
Maximal munch is unchanged: ``<x`` is still ``E-LEX-003``, and ``-5``
still lexes as an integer literal (the exact-match check precedes the
numeric path only for the bare token).

A token ends at whitespace, ``(``, ``)``, ``;``, or end of input. A character
sequence that matches no atom class is ``E-LEX-003``.

Tokenization is **maximal munch within a token run**: the lexer reads to the
next delimiter and then classifies the whole run. ``1000.5$x`` is therefore one
run that classifies as nothing and is ``E-LEX-003`` — it is never split into
``1000.5$`` followed by ``x``. This differs deliberately from the doctrine
DSL's ``re.findall`` tokenizer, which silently drops nothing but does accept
adjacency without separators; BSL requires explicit separation.

1.5 Literals
~~~~~~~~~~~~~~

**Integer literals.** ``int-lit`` per §1.4. Underscores are digit-group
separators and are removed before interpretation; a leading or trailing
underscore, or two adjacent underscores, is ``E-LEX-003``. Leading zeros are
permitted and insignificant (``007`` is ``7``). The value must fit ``i64``
(``E-LEX-020``). Static type: ``Int``.

**Scaled literals.** A decimal numeral with a mandatory one-character **kind
suffix**. There are no unsuffixed non-integer literals — a bare ``0.5`` is
``E-LEX-021``. This is the lexical enforcement of the house prohibition on bare
floats.

.. list-table::
   :header-rows: 1
   :widths: 10 20 22 48

   * - Suffix
     - Static type
     - Closed range
     - Representation
   * - ``$``
     - ``Currency``
     - ``[0, ∞)``
     - Fixed point, ``i128`` **micro-units** (scale 6). A negative currency
       literal is ``E-LEX-022``; a literal with more than 6 fractional digits
       is ``E-LEX-023`` (never rounded at lex time). Value in micro-units =
       ``unscaled × 10^(6 − scale)``; must fit ``i128``.
   * - ``p``
     - ``Probability``
     - ``[0.0, 1.0]``
     - Decimal, canonicalized to ``(unscaled: i128, scale: u8)``.
   * - ``i``
     - ``Intensity``
     - ``[0.0, 1.0]``
     - As ``p``.
   * - ``c``
     - ``Coefficient``
     - ``[0.0, 1.0]``
     - As ``p``.
   * - ``r``
     - ``Ratio``
     - ``(0, ∞)``
     - As ``p``, canonicalized to ``(unscaled: i128, scale: u8)`` — but open
       at zero and unbounded above rather than unit-interval. A non-positive
       (zero or negative) literal is ``E-LEX-027``.

The ranges are those of the kernel scalars as they exist today
(``src/babylon/models/types.py``): ``Probability``, ``Intensity`` and
``Coefficient`` are all ``[0.0, 1.0]``, and ``Currency`` is ``[0.0, ∞)``.
A ``p``/``i``/``c`` literal outside ``[0,1]`` is ``E-LEX-024``. Maximum scale
for ``p``/``i``/``c`` is 9 (``E-LEX-023``); **[draft ruling — Phase 1 review]**
9 is chosen because ``binary64`` carries ~15–17 significant decimal digits and
the engine's existing quantization grid is ``1e-5`` (``SnapToGrid``), so 9
digits is comfortably beyond any authored precision while remaining exactly
representable in the ``i128`` unscaled form.

**Which literals a** ``real``\ **-declared field accepts** (Train B item 6,
#591 — D155 in the register): an ``int`` literal (exact to 2⁵³), any
``p``/``i``/``c`` literal, and any ``r`` literal — each already lex-bounded
in its own lane above — all converting to ``binary64`` by the one contract,
``unscaled / 10^scale``. A ``$`` literal is refused (the typed-storage
deferral every field type states, §3.2). There is NO arbitrary-precision
fractional literal: a bare ``0.5`` stays ``E-LEX-021`` under a ``real``
declaration too — that is #591 item 5's territory, deliberately not this
train's.

**``r`` — ``Ratio`` (Director ruling 2026-08-11, #492/ADR194: the
declared-domain Currency scale operation).** ``babylon_kernel::scalars::Ratio``
is a **pre-existing** kernel sort (`𝔾 ∩ (0, ∞)`, grid-quantized like every
other bounded scalar) that this document did not previously expose to BSL —
this row and §3.2's addendum are what expose it, minting no new mathematics
("scalar multiplication isn't new mathematics" is the ruling's own framing).
It exists to name a positive scale factor outside ``Coefficient``'s
``[0,1]`` domain — the construct gap director-gate #492 named Blocker-1: five
runtime-moddable coefficients had a declared domain beyond ``[0,1]`` with
**no BSL representation at all** before this addendum — not merely "cannot
multiply Currency", literally "cannot be written":

.. list-table::
   :header-rows: 1
   :widths: 30 20 50

   * - Consumer
     - Declared domain
     - Ratio lane carries
   * - Territory's ``rent_spike_multiplier``
     - ``(0, ∞)``
     - the whole domain — no ``:cap``/``:floor`` needed
   * - Metabolism's ``entropy_factor``
     - ``(1.0, 3.0]``
     - the whole domain — ``:floor 1r :cap 3r``
   * - Lifecycle's ``pareto_alpha``
     - ``(0, 10]``
     - the whole domain — ``:cap 10r``
   * - Lifecycle's ``early_mortality_modifier``
     - ``[0, 10]``
     - the OPEN INTERIOR ``(0, 10]`` only — ``:cap 10r``; the zero
       endpoint's disposition follows separately, below
   * - Lifecycle's ``carceral_transition_modifier``
     - ``[0, 10]``
     - the OPEN INTERIOR ``(0, 10]`` only — ``:cap 10r``; as above

Maximum scale is 9, the same cap and the same reuse of ``E-LEX-023`` as
``p``/``i``/``c`` (§1.5's own documented reuse discipline for "too many
fractional digits"). Unlike ``p``/``i``/``c``, ``Ratio`` is **not** one of
the six ``deffield``/``metric`` ``:type`` names (§3.1) — it joins ``Real``
as a typechecker type no ``<type-name>`` position can name, not a widening
of that closed table.

**A bare (undeclared) ``Ratio`` literal is legal in ordinary expression
position, with no ``defconst`` involved at all** — the reader classifies
every literal class position-independently (§1.4's whole design;
``Currency`` itself is the precedent: a ``$`` literal is legal directly in a
rule body, not only via ``:const``), and gating ``r`` specifically to
"inside a ``defconst`` only" would need a context-sensitive lexer rule this
reader's architecture does not have and this addendum does not add. The
ruling's own wording, "a defconst-declared positive coefficient", names the
INTENDED use for moddability (§3.2's `Why this closes director-gate #492`
paragraph) — it is not a syntactic restriction, and this document does not
read it as one.

*Declared bounds, narrower than* ``(0, ∞)``. ``Ratio``'s own domain is
already stated by construction — this literal's row IS the "domain stated at
declaration" the ruling asks for, for a consumer whose real domain is
genuinely unbounded (``rent_spike_multiplier``). A consumer with a tighter
domain narrows it further via the Rust reference's ``defconst``
``:floor``/``:cap`` extension (``rust/crates/babylon-bsl/src/
scenario.rs::load_defconst``) — **stated here for orientation, not specified
here**: ``defconst``, like the rest of the ``.bscn`` scenario-file dialect, is
outside this document's grammar (§7, D93: "no section specifies it"), so
neither keyword is an rst production or a §1.6 keyword-table row. What §3.2
DOES specify, because it belongs to the language proper, is the operator
both keywords exist to feed: ``Currency × Ratio``. ``:cap`` narrows the
UPPER end, INCLUSIVE (``pareto_alpha``'s ``(0, 10]``: ``:cap 10r``);
``:floor`` narrows the LOWER end, EXCLUSIVE — matching ``Ratio``'s own
open-at-zero law — for a consumer whose floor is load-bearing, not
decorative: ``entropy_factor``'s domain is ``(1.0, 3.0]`` because "extraction
costs more than it yields" is the entire thermodynamic point, and a modded
value at or below ``1.0`` is the exact violation the floor exists to forbid.
The worked example, both bounds together:
``(defconst metabolism/entropy-factor 1.5r :floor 1r :cap 3r)``.

**The zero endpoint — what the Ratio lane does NOT carry, and why.**
``early_mortality_modifier``'s and ``carceral_transition_modifier``'s
declared domain is ``[0, 10]`` — CLOSED at zero — and the frozen engine
actively PRODUCES ``0.0`` for them (``mobility.py:187-188``): the mortality
channel OFF, semantically meaningful, not an error case. ``Ratio`` cannot
represent that ``0`` — not as a gap in this addendum's coverage but as the
same structural law that makes the whole construct a "POSITIVE coefficient"
(the ruling's own phrase) rather than a general bounded scalar: the reader's
``E-LEX-027`` refuses a ``0r`` literal before it is even a token, and
``Ratio::new(0.0)`` refuses at the kernel layer independently. Widening the
sort to admit zero — the seemingly obvious fix — is explicitly declined:
"zero-scaling is the multiply's ABSENCE, not a scale," and admitting a
zero-valued ``Ratio`` would be exactly the sort-widening the ruling's
"scalar multiplication isn't new mathematics" framing forbids re-opening.

The content-layer answer needs no new construct: ``guard`` (§2.8) already
gates whether an EFFECT fires at all, on a signal separate from the
``Ratio``-typed magnitude — never "the ``Ratio`` equals zero" (unwritable),
always "the multiply does not run this tick." A rule wanting the frozen
engine's "``0.0`` ⇒ no effect" behavior guards the ``Currency × Ratio``
effect on an ordinary ``Bool`` ``:const`` (or a field, or a doctrine-derived
condition — whatever the content authors), for instance
``(guard channel-active (emit … (scaled-mortality (* base-rate modifier))))``.
This is proved end-to-end, not merely reasoned about: the guarded-effect
mechanism clears the whole ``run_once`` seam in both directions — the
effect fires and the exact product is observed when the flag is ``#t``, and
the multiply never evaluates at all (an empty event list, not a zero-valued
one) when the flag is ``#f`` — in
``rust/crates/babylon-tick/tests/currency_scale_op_e2e.rs``
(``a_guard_gated_ratio_multiply_fires_when_the_channel_is_active`` /
``…_never_evaluates_when_the_channel_is_off``). The absence holds
**because the multiply sits inline in the guarded effect body**: guarded
effect bodies are genuinely lazy, while ``:expr`` BINDINGS resolve before
any guard runs (§4.2's binding-then-guard order), so a binding-shaped
multiply would still evaluate — and fuel-charge, and surface any eval
error — under a false guard. Content wanting the absence must write the
multiply inline, exactly as the example above does. **What this does NOT
settle**: WHICH signal a real Lifecycle port gates on — a dedicated
activation const, a doctrine tag, a field read — is that port's own
content-modeling decision, the same way this whole addendum ports no
consumer. This paragraph proves the mechanism is available and typechecks
cleanly through the real entry point; it does not pre-choose Lifecycle's
answer.

*Decimal canonicalization.* Every scaled literal is reduced to its minimal
scale: trailing zeros of the fractional part are stripped, and zero canonicalizes
to ``(0, 0)``. ``0.50c``, ``0.5c`` and ``.5c`` (the last is ``E-LEX-003``, a
leading digit is mandatory) are therefore not three values but one — the first
two hash identically. Currency literals canonicalize further, to their integer
micro-unit value.

**String literals.** ``"`` … ``"``. The only escapes are ``\"``, ``\\``,
``\n`` (LF) and ``\t`` (TAB); any other backslash sequence is ``E-LEX-025``.
A raw LF inside a string literal is ``E-LEX-025`` (strings are single-line).
Maximum length 1024 bytes after escape processing (``E-LEX-026``). Strings
appear only in ``:material-basis`` and in conformance-vector identifiers; there
is no string concatenation, comparison, or interpolation in the language — the
``${var}`` substitution of today's ``TemplateEffect``/``EventEmission`` does
**not** carry over (its job is done by binding references).

**Enum member references.** An ``<enum-ref>``'s type name must be registered
and its member must exist in that registration; both checks are load-time.
There is no "unknown enum member reads as default" behavior anywhere in the
language. **Which registry checks it is a property of where the
``<enum-ref>`` sits, never a guess between two candidates** (§2.13 states
this fully): in a §2.6 class-rule operand position (a query head, ``the``,
a structural verb's type operand, ``emit``) it is checked against the
**closed graph vocabulary** (§3.6, populated by ``defvocabulary`` —
``E-LOAD-030``/``E-LOAD-031``); in an ``:enum-type``-declared field's value
position (§2.9, §2.13) it is checked against that field's declared
**content-declared enum type** (populated by ``defenum`` — ``E-LOAD-054``/
``E-LOAD-055``). The two registries are never interchangeable: §3.6 sorts
and deduplicates its members, while a ``defenum`` type's member order is
the field's storage ordinal (§3.1's ``enum`` row) and must never be
reordered.

1.6 Keywords
~~~~~~~~~~~~~~

A keyword is a colon-prefixed symbol. Keywords are never values: they may only
appear in the option position of a form, always immediately followed by their
operand (except for the flag keywords listed below, which take no operand).
A keyword in value position is ``E-PARSE-010``. **[draft ruling — Phase 1
review, R9 verification repair]** So is a **string literal** in expression
position, for the same reason and under the same code: §1.5's strings are
well-formed atoms admitted at ``:material-basis`` and at conformance-vector
identifiers only, ``<expr>`` (§2.7) has no string form, and ``Str`` has no
operations (§3.1). Both are atoms rejected by *position*, not by lexis.

.. list-table::
   :header-rows: 1
   :widths: 22 12 66

   * - Keyword
     - Operand
     - Meaning
   * - ``:material-basis``
     - string
     - **Mandatory on every rule.** The Aleksandrov Test's parse-time half.
       The parser enforces **presence and non-emptiness only** (a string of
       length 0, or one consisting solely of whitespace, is ``E-PARSE-011``).
       The semantic III.8 obligation — does the named material process
       actually ground this construct — is *not* checked here and stays with
       Director review and the sentinel successor's aleksandrov family. The
       string is inside ``rules_hash``: editing a material basis is declared
       input drift.
   * - ``:fuel``
     - integer
     - **Mandatory on every rule.** The rule's fuel budget (§4.5). Must be
       ``> 0`` and ``≤ 1_000_000`` (``E-PARSE-012``).
   * - ``:field``
     - qualified name
     - Binding source: read a declared field of a node (§2.5).
   * - ``:const``
     - qualified name
     - Binding source: read a coefficient from the defines environment. This
       is the successor of the doctrine DSL's ``@snake_case`` sigil; the
       ``@`` sigil does not survive.
   * - ``:metric``
     - symbol
     - Binding source: read a named graph-level metric (§2.5).
   * - ``:expr``
     - expression
     - Binding source: a computed value, a pure function of the bindings
       declared before it (§2.5).
   * - ``:optional``
     - *flag*
     - Marks a binding permitted to be absent at evaluation. Requires
       ``:default`` (§3.5).
   * - ``:default``
     - literal
     - The value an absent ``:optional`` binding takes. Only literals — never
       an expression.
   * - ``:tick``
     - *flag*
     - Binding source: the current tick, as ``Int``.
   * - ``:year`` / ``:tick-of-year``
     - *flags*
     - Binding sources: kernel-computed calendar reads, as ``Int`` (§2.5).
   * - ``:tick-in-cycle``
     - integer
     - Binding source: the current tick's position in a cycle of the given
       length, as ``Int``. The length must be ``> 0`` (``E-PARSE-014``).
   * - ``:kind``
     - ``intensive`` | ``extensive``
     - Per-field intensivity declaration, on ``deffield`` forms only (§3.4).
   * - ``:type``
     - type name
     - Scalar type, on ``deffield``/``intrinsic`` forms.
   * - ``:member``
     - enum-ref
     - The member ``NodeType`` half of an attributed membership's owner pair,
       on ``deffield`` forms only (§2.9, §2.12). Its presence is what makes a
       ``deffield`` a membership-payload declaration; elsewhere it is
       ``E-PARSE-013``.
   * - ``:as``
     - symbol
     - Names the current element of an iterating form, so a nested body can
       still reach it (§2.6).
   * - ``:weight``
     - expression
     - The mandatory explicit weight term of a weighted aggregation (§3.4).
   * - ``:strength``
     - expression
     - Edge strength operand of ``add-edge``.
   * - ``:after`` / ``:before``
     - symbol
     - Ordering anchors (§2.8). A raw position float is not expressible.
   * - ``:out`` / ``:in`` / ``:any``
     - *flags*
     - Traversal direction, on ``neighbors`` queries only (§2.6).
   * - ``:graph``
     - *flag*
     - Graph domain, on ``domain`` forms only (§2.3). Illegal elsewhere
       (``E-PARSE-013``).
   * - ``:params`` / ``:returns`` / ``:cost``
     - see §2.7
     - Intrinsic declaration fields.
   * - ``:provider``
     - symbol
     - The kernel service a ``metric`` declaration binds to (§2.11).
   * - ``:ceiling``
     - integer
     - Declared cardinality ceiling, on ``manifest`` forms (§3.7).
   * - ``:max-members``
     - integer
     - Declared **member-count** ceiling of one hyperedge type, on the
       ``manifest`` ``ceiling`` rows whose ``<enum-ref>`` is a
       ``HyperedgeType`` member (§3.7). Mandatory there, illegal elsewhere.
   * - ``:invariant``
     - *flag*
     - Marks a ``NodeType`` or ``EdgeType`` ``ceiling`` row as invariant
       substrate that no structural verb may add to or remove from (§3.9).
   * - ``:via``
     - enum-ref
     - The ``EdgeType`` carrying a declared scale rung's relation, on
       ``rung`` forms only (§2.9, §3.9).
   * - ``:substrate``
     - *flag*
     - Marks a ``rung`` as running over the invariant substrate, which
       obliges every type it names to carry ``:invariant`` (§3.9).
   * - ``:rung``
     - symbol
     - The declared rung an ``adjunction`` instance runs along (§3.9).
   * - ``:weighted-by``
     - qualified name
     - The extensive field an intensive ``adjunction`` weights by (§3.9).
       Distinct from ``:weight``, whose operand is an expression.

**[draft ruling — Phase 1 review, R9 chapter C4]** The three ``neighbors``
directions were used as bare flags by §2.6 from this document's first revision
and were missing from this table — the same class of omission the operator-atom
ruling of §1.4 records, and it is fixed the same way, by writing down what the
grammar already required. ``:graph`` joins them as a flag keyword so that
``(domain :graph)`` is a legal form rather than a keyword in value position
(``E-PARSE-010``); it encodes as an ``opt`` under D20 like every other flag.

The keyword set is **closed**. An unrecognized keyword is ``E-PARSE-013``; it
is never ignored. Adding a keyword is a language revision and re-blesses the
conformance vectors (§6).

2. Syntactic grammar
----------------------

2.1 Notation
~~~~~~~~~~~~~~

BNF with ``::=`` for production, ``|`` for alternation, ``*`` for zero-or-more,
``+`` for one-or-more, ``?`` for optional. Terminals are quoted. Lexical
classes from §1 appear as ``<symbol>``, ``<qname>``, ``<keyword>``,
``<enum-ref>``, ``<int-lit>``, ``<scaled-lit>``, ``<string>``, ``<bool-lit>``.

2.2 Content files
~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   <file>        ::= <top-form>*
   <top-form>    ::= <rule> | <deffield> | <intrinsic-decl> | <manifest>
                   | <metric-decl>

A content set is the union of all files under the declared content roots. File
boundaries and file names carry **no semantics**: the same forms split across
different files produce the same ``rules_hash`` (§5.5). Duplicate rule ids,
duplicate field declarations, duplicate intrinsic declarations, or — since
Amendment AG (ii) — duplicate ``rung`` or ``adjunction`` names across the
content set are ``E-LOAD-001``.

2.3 Rules
~~~~~~~~~~~

.. code-block:: text

   <rule>     ::= "(" "rule" <qname>
                      ":role" <rule-role>
                      ":evidence" <evidence-class>
                      ":material-basis" <string>
                      ":fuel" <int-lit>
                      <domain>?
                      <anchor>?
                      <bindings>
                      <when>?
                      <effects>
                  ")"

   <rule-role> ::= "mechanic" | "recognizer" | "external-event" | "intent"
   <evidence-class> ::= "observed" | "derived" | "calibrated" | "designed"

   <domain>   ::= "(" "domain" ( <enum-ref> | ":graph" ) ")"
   <anchor>   ::= "(" "anchor" ( ":after" | ":before" ) <symbol> ")"
   <bindings> ::= "(" "bindings" <binding>* ")"
   <when>     ::= "(" "when" <cond> ")"
   <effects>  ::= "(" "effects" <effect-item>+ ")"

The four valued keyword options ``:role``, ``:evidence``,
``:material-basis``, and ``:fuel`` are mandatory and may appear in any source
order; the canonical serialization sorts them (§5.3). A missing option is a
malformed rule surface. A role or evidence value outside its closed set is
``E-PARSE-015``.

``:role`` declares causal authority; it is not descriptive decoration.
``mechanic`` derives endogenous state through the ordinary typed effect
algebra. ``recognizer`` describes a governed state pattern. ``external-event``
is the role for exogenous shocks and may introduce only an approved burden,
capacity change, or pressure. ``intent`` may introduce only an approved
next-week intent carrier. The latter three roles are default-deny: every
possible field write and emitted event in their complete effect tree must
match an exact governed allowance row for that rule id. An unapproved effect,
including one under an untaken ``guard`` or ``for-each`` branch, is
``E-LOAD-060``. Graph-shape effects are never allowed for these roles. Calling
a downstream result a pressure does not grant authority; matching uses exact
qualified names, never substrings.

For built-in content, the production CI sentinel requires exact set equality
between each non-``Mechanic`` rule's complete effect footprint and its
``GOVERNED_EFFECT_ALLOWANCES`` rows. It also requires unique allowance keys.
Thus a missing permission, an extra dead permission, or a duplicate key cannot
survive the production corpus check.

``:evidence`` carries Article IV's evidentiary class for the rule itself:
``observed``, ``derived``, ``calibrated``, or ``designed``. Evidence class
does not grant effect authority. A receipt preserves both role and evidence so
a later projection can explain what kind of rule moved a value without
misstating designed or calibrated content as observed fact.

The declared pair is the first attribution ledger. A built-in production rule
must also match the independent
``causal_contract::GOVERNED_RULE_ATTRIBUTIONS`` manifest. Each governed row
is a ``GovernedRuleAttribution`` that pins the exact rule id, ``RuleRole``,
``EvidenceClass``, owner, date, and ADR224. A known ID whose declaration
differs refuses load as the typed, unnumbered
``ContractError::GovernedAttributionMismatch``. A CI sentinel proves exact set
equality between the 60-rule production corpus and the manifest, catching an
unlisted built-in or a row with no built-in rule. Unknown fixture and mod IDs
remain self-declared. Thus the second ledger prevents first-party production
content from escalating its own authority; it does not make arbitrary mod
content an untrusted sandbox.

**Omitted ``<when>``.** A rule with no ``<when>`` is unconditional. An
**empty** condition is not expressible: ``(when)`` is ``E-PARSE-020``, and a
rule that means "always" writes ``(when #t)`` or omits the clause. This is one
of the four deliberate III.11 corrections (§6.3).

*Executable anchor placement (PER-17, ADR222).* A rule with no ``<anchor>``
belongs to the system named by the first segment of its rule id and takes that
system's governed position. A rule whose first id segment names no registered
system, and which carries no anchor, is ``E-LOAD-002``. An explicit anchor
selects the boundary immediately before or after its named system; ``:after``
one slot and ``:before`` the next share one boundary. Rules at one resolved
position use D16's rule-id byte order.

The registry follows the frozen 34-system causal spine: 15 Material Base
slots, one Action slot, and 18 Consequence slots. Four compatibility names
keep governed homes: ``class-dynamics`` maps to ``tick-dynamics``,
``economics`` to ``contradiction``, ``organization`` to ``ooda``, and
``social-class`` to ``solidarity``. Mods cannot land a rule
"nowhere" and cannot express a raw position float. The boundary before
Vitality and the boundary after Metabolism are legal. An explicit anchor that
cuts through the interior of Material Base is ``E-LOAD-003``. Placement
compiles for the complete content set before caller-owned hydration or
mutation. Validation may hydrate a disposable graph first to preserve the
established scenario-error precedence, but an invalid composition never
touches caller-owned world state.

**[draft ruling — Phase 1 review, R9 chapter C4]** *The rule domain — what a
rule fires over, and how many times.* §4.2 says a rule evaluates against "the
subject node" and §5.6 lets the reader *infer* the subject's type from a
binding's qname prefix; the document never stated the inference rule and said
nothing at all about a rule whose only bindings are ``:const``, ``:metric`` and
``:tick``. Six systems (R9 gap analysis §2, Q12) perform exactly one
graph-level check per tick — ControlRatio's four-phase machine, Metabolism's
overshoot check, FieldDerivative's principal-contradiction pick — and under a
per-node reading those would fire once per node.

``<domain>`` is **optional**, with the inference below as its default. It is
optional rather than mandatory for the same reason ``<anchor>`` is (D5): the
document already carries a default-inference convention for rule-level
placement, mandating a new child would make every existing rule form
non-conforming, and §5.6's pinned canonical bytes would need recomputation for
no gain in expressiveness.

*The inference, stated so it is computable at load.* Let ``U`` be the set of
node types owning (§2.9):

- every ``:field`` binding referenced at least once **outside every query
  body**, and
- every ``<qname>`` of an ``update-node`` verb whose element operand is the
  symbol ``self``, and of every ``field-of`` whose element operand is ``self``.

Then ``|U| = 1`` gives the domain; ``|U| = 0`` (nothing is self-scoped) and
``|U| > 1`` (two node types are) are both ``E-LOAD-004``, and both are repaired
by writing ``<domain>`` explicitly. **The surprise the gap analysis names is
removed by construction**: a binding referenced only inside a fold body never
enters ``U``, so adding one cannot change how many times a rule fires.

*Explicit domains.* ``(domain NodeType/SOCIAL_CLASS)`` replaces the inference
outright. Its operand is a ``NodeType`` member — an ``EdgeType`` or
``HyperedgeType`` member there is ``E-TYPE-011`` under §2.6's enum-ref class
rule — and a self-scoped reference owning off a different node type is
``E-TYPE-010``, the existing code for a foreign node type read outside a fold
body over that type, which is exactly what such a reference is.

*The graph domain.* ``(domain :graph)`` fires the rule **exactly once per
tick**, at its anchor position. ``self`` is not bound in a graph-domain rule:
any reference to ``self``, and any ``:field`` binding referenced outside a
query body, is ``E-TYPE-015``. Graph-domain rules read the graph through
queries and through §2.10's accessors, which is what chapter C3's carrier
ruling is for; they read nothing else the language did not already give them.
``(domain :graph)`` is therefore a firing-multiplicity declaration and not a
new capability — it removes an ``N``-fold repetition, it does not reach
further.

2.4 Conditions
~~~~~~~~~~~~~~~~

.. code-block:: text

   <cond>   ::= <bool-lit>
              | "(" "and" <cond>+ ")"
              | "(" "or"  <cond>+ ")"
              | "(" "not" <cond> ")"
              | "(" <cmp> <expr> <expr> ")"
              | "(" "exists" <query> <elem-name>? <cond>? ")"
              | "(" "forall" <query> <elem-name>? <cond> ")"

   <cmp>    ::= "<" | "<=" | ">" | ">=" | "=" | "!="

``and`` and ``or`` are variadic with at least one operand; ``(and)`` and
``(or)`` are ``E-PARSE-021`` (there is no implicit identity element — the same
correction as the empty precondition set).

**[draft ruling — Phase 1 review, R9 chapter C12]** *References compare by
identity, with* ``=`` *and* ``!=`` *only.* Two ``NodeRef``\ s (or two
``EdgeRef``\ s, or two ``HyperedgeRef``\ s) may be compared for identity;
comparing a reference with an ordering operator, or with a reference of a
different kind, or with any non-reference, is ``E-TYPE-017``. This document
had left reference comparison undefined, which made the intersection idiom of
§2.7 unwritable and would have left two implementations free to differ. There
is no ordering on references *in the language* — §2.6's iteration order is the
executor's, and exposing it as a comparison would invite content to depend on
id assignment.

``exists``/``forall`` bind no variable of their own; the query's element is
referred to inside the body as ``it`` **[draft ruling — Phase 1 review]** —
a reserved binding name that may not be declared or shadowed
(``E-PARSE-022``). ``(exists <query>)`` with no body is "the query is
non-empty".

*Coverage of the existing grammars.* This set expresses everything the two
Python grammars express:

.. list-table::
   :header-rows: 1
   :widths: 42 58

   * - Existing construct
     - BSL form
   * - doctrine ``TAG >= 3``
     - ``(>= tag-total 3)`` over a ``:field``/``:metric`` binding
   * - doctrine ``@coeff`` threshold
     - a ``:const`` binding
   * - doctrine ``AND``/``OR``/``NOT``/parens
     - ``and``/``or``/``not`` (parenthesisation is structural)
   * - ``PreconditionSet.logic = "all"`` / ``"any"``
     - ``and`` / ``or``
   * - ``NodeCondition`` aggregation ``any``
     - ``(exists (nodes …) <cmp>)``
   * - ``NodeCondition`` aggregation ``all``
     - ``(forall (nodes …) <cmp>)``
   * - ``NodeCondition`` aggregation ``count``/``sum``/``avg``/``max``/``min``
     - ``(<cmp> (fold count|sum|mean|max|min …) <threshold>)``
   * - ``NodeFilter`` (node type / role / id pattern)
     - ``<node-pred>`` (§2.6)
   * - ``EdgeCondition`` ``count``
     - ``(fold count (edges EdgeType/SOLIDARITY) 1)``
   * - ``EdgeCondition`` ``sum_strength`` / ``avg_strength``
     - ``(fold sum|mean (edges EdgeType/SOLIDARITY)
       (field-of it solidarity/strength))`` — the ``field-of`` accessor of
       §2.10.
   * - ``GraphCondition`` six named metrics
     - ``:metric`` bindings, one per registered metric

**A contradiction this table used to carry, now closed (R9 chapter C1).** The
two ``EdgeCondition`` rows above were one row until the R9 gap analysis
(``reports/bsl-gap-analysis-2026-08-10.md`` §2, Q1), and that row promised
``sum_strength``/``avg_strength`` transcribe as a fold over ``edges`` — while
§2.5 scoped ``:field`` to node types, §2.9's ``deffield`` had no edge case, and
no §2.7 production could read anything off an ``EdgeRef``. The document
committed to a capability its grammar could not express. §2.9 (edge- and
hyperedge-qualified ``deffield``) and §2.10 (``field-of``) close it; the row is
split above because ``count`` needs no attribute read and the other two do.
The frozen Python site the row transcribes is
``engine/event_evaluator.py:174-175``, which reads ``solidarity_strength`` and
defaults it to ``0.0`` — the default is the §6.3 honest-null delta, not part of
the form.

2.5 Bindings
~~~~~~~~~~~~~~

.. code-block:: text

   <binding>   ::= "(" "binding" <symbol> <bind-src> <bind-opt>* ")"

   <bind-src>  ::= ":field"  <qname>
                 | ":const"  <qname>
                 | ":metric" <symbol>
                 | ":tick"
                 | ":year"
                 | ":tick-of-year"
                 | ":tick-in-cycle" <int-lit>
                 | ":expr"   <expr>

   <bind-opt>  ::= ":optional" | ":default" <literal>

A binding names a value the rule reads. **A plain (non-``:optional``) declared
binding that is unbound at load is a load error** (``E-LOAD-010``) — the
opt-in to absence is content, not a test list.

``:field`` reads a declared field of ``self``'s node type unless the qualified
name's first segment names a different node type, in which case it is only
legal inside a fold body over that type (``E-TYPE-010``). ``:const`` reads a
coefficient from the defines environment. ``:metric`` reads a registered
metric whose declared domain is ``:graph`` (§2.11); an unregistered metric name
is ``E-LOAD-011`` — never ``0.0`` (§6.3) — and an element-indexed metric read
through a ``:metric`` binding is ``E-LOAD-012``, since its value depends on an
element a binding does not name. ``:tick`` binds the current tick as ``Int``.

**[draft ruling — Phase 1 review, R9 chapter C13]** *Calendar reads are
bindings, not arithmetic.* Four call sites in the frozen estate compute
``tick % interval`` and ``base_year + tick // ticks_per_year``, and
``<arith>`` provides neither integer modulo nor floor division. The two obvious
repairs are an intrinsic rider for ``mod``/``floor-div`` or three more
bind-srcs; this document takes the second. ``:year``, ``:tick-of-year`` and
``:tick-in-cycle`` all bind ``Int``, all are computed by the kernel's clock,
and all are **seams to a kernel service** rather than mathematics — which is
the category R10 already sanctions without a rider, and a calendar is not a
curve. The epoch and the ticks-per-year figure are the kernel's, pinned in
:doc:`/reference/determinism-contract`; content does not choose them, which is
also what stops a mod-by-anything operator from arriving through the back door.
``:tick-in-cycle``'s length is a **literal**, not an expression, so the value
is a static function of the tick and the content bytes.

**[draft ruling — Phase 1 review, R9 chapter C1]** *A* ``:field`` *binding is
node-scoped, and stays node-scoped.* An edge's or a hyperedge's declared field
is read by the ``field-of`` accessor of §2.10, never by a ``:field`` binding.
Two reasons, both taken from the rest of this document rather than invented
here: a binding is declared once at rule scope and resolved *implicitly*
against an enclosing body, which needs exactly one candidate body to be
unambiguous — and the systems that read edge attributes read several edge types
in one rule; and §2.10's accessor takes its owning type as a qname operand,
which is D24's fix for the identical problem on ``members-of``. The R9 gap
analysis §2 (Q1) sketched an edge-typed ``:field`` binding instead; this
document's conventions override the sketch, and D29 records the divergence.

**[draft ruling — Phase 1 review, R9 chapter C1]** *Implicit resolution
requires a unique candidate.* A foreign-node-type ``:field`` binding
referenced where **two or more** enclosing bodies range over that same node
type is ``E-TYPE-013`` — the reference is ambiguous and the author must write
``field-of`` against a named element (§2.6's ``:as``) instead. This is the
narrow, loud form of the resolution rule §2.5 previously left to inference;
single-body code is unaffected.

Two symbols are **reserved and always in scope**, never declared and never
shadowed (``E-PARSE-022``): ``self``, the node the rule is being evaluated
for (``NodeRef``), and ``it``, the current element inside a query predicate or
fold body (``NodeRef``, ``EdgeRef`` or ``HyperedgeRef``, per the query it
ranges over). ``it`` outside a query context is ``E-TYPE-012``.

Binding names are otherwise rule-scoped; a duplicate name in one rule is
``E-PARSE-030``.

**[draft ruling — Phase 1 review, R9 chapter C7]** ``:expr`` — *a rule may name
an intermediate value.* Every other ``<bind-src>`` names an **external**
source, so before this chapter a rule could not name anything it computed
itself, and every non-trivial rule repeated its sub-expressions verbatim. That
costs three things at once: fuel (a repeated fold is charged at its ceiling
every time it is written), correctness (a transcription that must restate the
same algebra in four places will eventually restate it differently in one), and
reviewability — and the third is the one that matters most here, because the
standing no-imposed-functional-forms line can only be enforced against algebra
a reviewer can read.

.. code-block:: scheme

   (bindings
     (binding wealth      :field social-class/wealth)
     (binding subsistence :const vitality/subsistence-cost)
     (binding drained     :expr (- wealth subsistence)))

- *Type and kind* come from the expression, computed bottom-up like any other
  (§3.1, §3.4). There is no annotation and no inference beyond that.
- *Resolution is in declaration order.* A ``:expr`` may reference bindings
  declared **before** it and no others; a forward reference or a self-reference
  is ``E-PARSE-032``. Order therefore matters inside ``<bindings>``, which is
  the one place in this document where a list's order carries meaning and is
  not a formatting concern — §5.3's group 3 already emits it in source order,
  so CAS needs no change.
- *No cycles are expressible*, by construction: the forward-reference ban makes
  the dependency graph a DAG in source order, so nothing needs a cycle
  analysis.
- *A* ``:expr`` *is evaluated at rule scope*, so ``it`` is ``E-TYPE-012``
  inside one, and referencing a foreign-node-type ``:field`` binding — legal
  only inside a fold body over that type — is ``E-TYPE-010``. A ``:expr``
  **may** contain a fold, a selection or an accessor of its own; that is the
  fuel win, since the ceiling factor is then paid once.
- ``:optional`` *and* ``:default`` *are illegal on a* ``:expr`` (``E-PARSE-033``).
  A computed value is never absent: its operands were resolved at load or the
  rule did not load, and §3.5's whole point is that absence is opted into at
  the external source rather than inherited by everything downstream.

Critically, ``:expr`` does **not** weaken §4.2's law that a rule never observes
its own effects. A computed binding is a pure function of pre-state bindings,
evaluated before any effect applies — it is an abbreviation, not a sequencing
construct, and nothing in it can read a value this rule wrote.

2.6 Queries
~~~~~~~~~~~~~

.. code-block:: text

   <query>      ::= "(" "nodes" <enum-ref> <node-pred>? ")"
                  | "(" "edges" <enum-ref> <edge-pred>? ")"
                  | "(" "neighbors" <expr> <enum-ref> <direction> <enum-ref> ")"
                  | "(" "hyperedges" <enum-ref> <hedge-pred>? ")"
                  | "(" "members-of" <expr> <enum-ref> ")"
                  | "(" "hyperedges-of" <expr> <enum-ref> ")"

   <elem-name>  ::= ":as" <symbol>
   <direction>  ::= ":out" | ":in" | ":any"
   <node-pred>  ::= <cond>
   <edge-pred>  ::= <cond>
   <hedge-pred> ::= <cond>

**[draft ruling — Phase 1 review, R9 verification repair]** *Every*
``<enum-ref>`` *operand position is typed, and* ``E-TYPE-011`` *is the code for
all of them.* The rule is stated once here as a class rather than per form,
because the R9 chapters added four such positions and a per-form restatement
left each new one without a rejection:

- ``NodeType`` — ``nodes``, ``neighbors``' **fourth** operand, ``the``
  (§2.10), ``(domain <enum-ref>)`` (§2.3), ``deffield``'s ``:member`` operand
  (§2.9), the annotation of an ``add-hyperedge`` member item (§2.8) and both
  positional operands of a ``rung`` (§2.9);
- ``EdgeType`` — ``edges``, ``neighbors``' **second** operand,
  ``edge-between`` (§2.10) and a ``rung``'s ``:via`` operand (§2.9);
- ``HyperedgeType`` — ``hyperedges``, ``members-of`` and ``hyperedges-of``;
- ``EventType`` — ``emit`` (§2.8).

An operand naming a member of any other enum kind is ``E-TYPE-011``, checked
statically. It is a *kind* check and nothing more: whether the named type and
member exist at all is ``E-LOAD-030``/``E-LOAD-031`` (§1.5). ``neighbors``
therefore takes two — an ``EdgeType`` for the relation traversed and a
``NodeType`` for the elements yielded (below) — and swapping them is
``E-TYPE-011`` at both positions. Predicates and bodies refer to the candidate
element as ``it``, or by the ``:as`` name of the iterating form.

Element and result types:

.. list-table::
   :header-rows: 1
   :widths: 24 18 20 38

   * - Query
     - Result
     - ``it`` is
     - Ranges over
   * - ``nodes``
     - ``NodeSet``
     - ``NodeRef``
     - Every node of the given ``NodeType``.
   * - ``edges``
     - ``EdgeSet``
     - ``EdgeRef``
     - Every dyadic edge of the given ``EdgeType``.
   * - ``neighbors``
     - ``NodeSet``
     - ``NodeRef``
     - Nodes **of the annotated** ``NodeType`` reachable from the operand
       across that ``EdgeType`` in the given direction. Each such node appears
       **once**, however many qualifying edges reach it (below).
   * - ``hyperedges``
     - ``HyperedgeSet``
     - ``HyperedgeRef``
     - Every hyperedge of the given ``HyperedgeType``.
   * - ``members-of``
     - ``NodeSet``
     - ``NodeRef``
     - The members of the hyperedge the operand denotes.
   * - ``hyperedges-of``
     - ``HyperedgeSet``
     - ``HyperedgeRef``
     - The hyperedges of that type the operand node belongs to.

**Hyperedges are first-class** (Amendment D ruled **NATIVE HYPEREDGE**
2026-07-29; Amendment AE clause (vi), analysis §9 sub-ruling D-1). A hyperedge
is a typed graph object with an identity and a member list — not sugar for a
set of dyadic edges and not a bipartite encoding the language can see.
``(members-of h HyperedgeType/ECONOMIC_SECTOR)`` is therefore **one lookup
against one object**, and there is no member↔member edge anywhere for
``edges``/``neighbors`` to walk: Anti-Pattern VIII.9 is preserved by
construction rather than by policy. The dyadic forms above are untouched —
``edges`` and ``neighbors`` range over the strictly dyadic morphism layer
(II.9), which coexists with the hyperedge layer in one substrate under
sub-ruling D-2 (type-level separation, one substrate, typed homes). Whether an
implementation *stores* hyperedges as a Levi/incidence bipartite graph is its
own business and is unobservable here.

**[draft ruling — Phase 1 review]** ``members-of`` and ``hyperedges-of`` take
the ``HyperedgeType`` as a **mandatory second operand**, even though the first
operand already denotes a hyperedge (or a node incident to one). BSL has no
type variables (§3.1), so a ``HyperedgeRef`` does not carry its type
statically; the annotation is what makes ``ceiling(query)`` computable at load
(§3.7), and therefore what keeps a fold over members statically bounded. At
evaluation, a ``members-of`` whose referent is not of the annotated type is
``E-EVAL-032`` — never a silently empty set.

**Iteration order is part of the contract.** A query yields its elements in
**ascending node-id / (source-id, target-id, edge-type) / hyperedge-id
lexicographic byte order** **[draft ruling — Phase 1 review]** — never in
graph-internal storage order. This is the language-level answer to the
cross-language iteration-order trap; it makes fold results independent of the
**underlying graph library** — of its bucket order, its adjacency order and
its insertion-versus-storage bookkeeping alike.

**[Director ruling 2026-08-11, ADR191 R2]** *It does not make them independent
of a scenario's declaration order, and it is not asked to.* An earlier
revision of the sentence above promised independence of "insertion history"
as well, and that half was never delivered: ``NodeId`` is a monotonic mint
counter, so a scenario's nodes take their ids top to bottom and shuffling two
``node`` declarations permutes the ids, the iteration order and the tick hash
with them. The Director ruled the promise weakened rather than identity made
content-derived, on the ground that **a scenario is a canonical committed
artifact and its declaration order is part of its identity**: a reordered
scenario is a different scenario, and a different tick hash is the honest
answer rather than a defect to engineer around. What the order above
guarantees is therefore exactly this — a *total* order, identical in every
conforming implementation, over whatever ids the scenario minted — and no test
may assert that shuffling a scenario's declarations leaves the tick hash
unchanged. Recorded as D96.

That order is *total* only because each of the three keys identifies at most
one element. For nodes and hyperedges the id is the identity. For edges the key
is the ``(source-id, target-id, edge-type)`` triple, and it is a **key** rather
than a sort field because both points at which an edge can enter the graph
refuse a second one: an ``add-edge`` duplicating an existing edge is
``E-EVAL-031`` (§2.8) and a hydration seeding the same triple twice is
``E-LOAD-044`` (§3.9).

**[draft ruling — Phase 1 review]** The same rule applies *inside* a
hyperedge: ``members-of`` yields members in ascending node-id byte order, and a
hyperedge's **declared** member order — the order ``add-hyperedge`` (§2.8) or a
scenario hydration listed them in — is never observable. A member list is a
set, not a sequence.

**[draft ruling — Phase 1 review, R9 verification repair]** *A* ``NodeSet``
*is a set, and* ``neighbors`` *yields each node exactly once.* A node reachable
from the operand across two or more qualifying edges — two ``SOLIDARITY`` edges
in opposite directions under ``:any``, an ``:out`` and an ``:in`` edge of one
type — appears **once** in the result. This is the only reading compatible with
the total order above: the order is ascending id byte order and a duplicated id
has no defined position relative to its twin, so a multiset result would leave
``(fold count (neighbors self EdgeType/SOLIDARITY :any NodeType/SOCIAL_CLASS))``
reading 1 or 2 by implementation choice — two conforming implementations
disagreeing on a tick hash, which is the failure this whole section exists to
prevent. The alternative reading (a multiset, one element per traversed edge)
is recorded as **rejected** for that reason (D72).

The consequence is worth stating in the form an author meets it: **a fold over**
``neighbors`` **counts and sums per node, never per edge.** A rule that means
"once per contributing edge" — a per-edge emission, a total of tie strengths —
folds over ``edges`` instead, or iterates one with ``for-each`` (§2.8), and
reads the relation there; ``neighbors`` answers *which nodes*, not *how many
ways*. ``members-of`` and ``hyperedges-of`` carry the same property for the same
reason — which D25 above already said of a member list.

**[draft ruling — Phase 1 review, R9 chapter C8]** ``neighbors`` *carries its
result* ``NodeType`` *as a mandatory fourth operand.* §2.5 permits a foreign
node type's ``:field`` "only inside a fold body over that type", and a fold
over ``nodes`` carries that annotation in the query's operand. ``neighbors``
did not: it yielded an untyped ``NodeSet``, so this document never said whether
``(fold mean (neighbors self EdgeType/TENANCY :in) social-class/consciousness
:weight …)`` typechecks — and six systems need exactly that read. This is
**D24's problem verbatim**, and it takes D24's fix: the type becomes an
operand, because §3.1 gives references no static type and an annotation is the
only thing that can supply one. With it, a fold body over ``neighbors``
legalises the annotated type's fields exactly as a fold over ``nodes`` does,
and a neighbour that is not of the annotated type is simply **not in the set** —
this operand *filters*, where ``members-of``'s D24 operand *asserts*, because a
node's edge may legitimately reach several node types while a hyperedge has one
type.

The operand is mandatory rather than optional, and that is a **breaking change
to a form that already existed**. What the estate holds, stated exactly (all
verified 2026-08-10): **no conformance vector and no content rule exercises**
``neighbors`` — zero occurrences under ``rust/crates/babylon-bsl/tests/``
(twelve ``.bsl`` vectors) and under ``rust/crates/babylon-tick/content/`` — so
no blessed expectation moves and there is no vector re-bless to pay. The
``babylon-bsl`` crate, however, **does** implement the pre-change form:
``bound_checker.rs`` reads the ceiling operand of ``neighbors`` at index 2 and
bounds it against the edge type alone, and a unit test pins that three-operand
spelling together with the edge-type-only bound D52 revises. Those are updated
when Phase 1 implements this chapter — the correction costs a grammar edit and
a crate change, which is the cheapest of the three prices it could have carried
and the reason to pay it now rather than defer. An optional operand would have
left the untyped reading legal and the under-determination alive.

``ceiling(neighbors)`` is correspondingly tightened in §3.7.

**[draft ruling — Phase 1 review, R9 chapter C8]** ``:as`` — *naming the
element, and what* ``it`` *actually means.* Two passages of this document
disagreed: §2.5 declares ``it`` "reserved and always in scope, never declared
and never shadowed (``E-PARSE-022``)", while §3.7's cost model discusses "a
fold over members nested inside a fold over hyperedges", which needs two live
elements at once. Four systems need a two-hop rule and none could be authored
with confidence.

The resolution is a reading, not a repeal. **``it`` always denotes the element
of the innermost enclosing iterating form.** That is rebinding by construction
— ``it`` is never *declared*, so there is no declaration for an inner form to
shadow, and ``E-PARSE-022``'s prohibition (content may not declare or shadow
``it``) is untouched and still means what it said. What was missing was a way
to reach an *outer* element, and ``:as`` supplies it:

.. code-block:: scheme

   (fold sum (hyperedges HyperedgeType/ECONOMIC_SECTOR) :as sector
         (fold sum (members-of sector HyperedgeType/ECONOMIC_SECTOR)
               (field-of it social-class/wealth)))

- A ``:as`` name is in scope for the whole body of its form, **including
  nested bodies**, and has the query's element type.
- Names share the rule's binding namespace: colliding with a binding or another
  ``:as`` name is ``E-PARSE-030``, and naming ``self`` or ``it`` is
  ``E-PARSE-022``.
- A ``:as`` name referenced outside its body is ``E-TYPE-012``, the same code
  and the same reason as ``it`` outside a query context.
- Naming is optional everywhere. Single-level rules keep reading ``it``, and
  **no existing form changes meaning** — which is the property that made
  naming the safer of the two candidate resolutions, against rebinding rules
  that would have had to carve an exception into ``E-PARSE-022``.

2.7 Expressions, intrinsics, folds, guards
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   <expr>      ::= <literal> | <symbol> | <enum-ref>
                 | "(" <arith> <expr> <expr> ")"
                 | "(" <intrinsic-name> <expr>* ")"
                 | "(" "if" <cond> <expr> <expr> ")"
                 | <fold>
                 | <accessor>                       ; §2.10
                 | <selection>

   <arith>     ::= "+" | "-" | "*" | "/"
   <literal>   ::= <int-lit> | <scaled-lit> | <bool-lit>

   <fold>      ::= "(" "fold" <fold-op> <query> <elem-name>?
                       <expr> ( ":weight" <expr> )? ")"
   <fold-op>   ::= "sum" | "mean" | "min" | "max" | "count"

   <selection> ::= "(" "select-max" <query> <elem-name>? <expr> ")"
                 | "(" "select-min" <query> <elem-name>? <expr> ")"

Arithmetic is strictly binary; ``(+ a b c)`` is ``E-PARSE-040``. This keeps
the reduction order explicit in the source rather than implied by a
left-fold convention — a cross-language float trap the design document names.

**[draft ruling — Phase 1 review, R9 verification repair]** *Arity and closed
terminal sets each get a code, rather than an unnumbered prose prohibition.*
Every form's operand count is fixed by its production, and a count that differs
from it is ``E-PARSE-042``; ``E-PARSE-040`` remains the arithmetic-specific
spelling of that class, and the three-operand ``neighbors`` of the pre-C8
grammar (§2.6, D51) is the case this revision creates. A head symbol that is
not a member of a closed terminal set — ``<fold-op>`` above, ``<cmp>`` (§2.4),
``<update-op>`` (§2.8) or ``<arith>`` — is ``E-PARSE-015``, which is where two
of §6.3's four silent-degradation corrections (unknown aggregation, unknown
comparison operator) land. Both were previously written as prohibitions with no
code, which left the conformance families that must pin them unable to name
what they expect (§6.2 families 17 and 19).

**Guards** are ``(if <cond> <a> <b>)`` in expression position and
``(guard <cond> <effect-item>+)`` in effect position (§2.8). Both branches of
``if`` must have the same static type (``E-TYPE-020``).

**Intrinsic calls** are ordinary forms whose head is a symbol declared in the
intrinsic table. Transcendentals and ``round-half-even`` are **never** language
primitives — they exist only as named intrinsics with pinned deterministic
implementations. The names this document has used to illustrate that
(``sigmoid``, ``exp``, ``log``, ``tanh``, ``sqrt``, ``entropy``) are
**illustrative of the class, not a table of intrinsics that exist** — a
reading the R9 gap analysis found this document inviting. What is declarable is
governed by §3.10: the ``{exp, log}`` transcendental cap (R10), the separately
authorized ``floor``, the ADR219 exact-arithmetic sextet
(``sqrt``, ``round-half-even``, ``min``, ``max``, ``abs``, ``clamp``) — and
``sigmoid`` ruled prohibited outright. BSL cannot define an intrinsic;
``intrinsic`` forms only
*declare* what the kernel provides, so that the typechecker and the fuel
bound-checker are computable from content alone:

.. code-block:: text

   <intrinsic-decl> ::= "(" "intrinsic" <symbol>
                            ":params" "(" <intrinsic-type-name>* ")"
                            ":returns" <intrinsic-type-name>
                            ":cost" <int-lit>
                        ")"

   <intrinsic-type-name> ::= <type-name> | "real"

**[draft ruling — Phase 1 review, Draft-Ruling Register D98]**
``<intrinsic-type-name>`` widens ``<type-name>`` with ``real`` — legal
**only** in an ``<intrinsic-decl>``'s ``:params``/``:returns`` position. A
``deffield``'s or a ``metric``'s ``:type`` still writes bare ``<type-name>``
(§2.9, §2.11) and still cannot name ``Real``: §3.1 rules it "Not storable",
and this widening does not touch that production. The need is structural,
not cosmetic — every binary64 expression's static type is ``Real`` (§3.3),
so before this row no intrinsic's argument could be declared at all, which
made ADR188 Row 2's own ``floor`` rider (§3.10) undeclarable in content.

A declaration whose signature disagrees with the kernel's registration is
``E-LOAD-020``; a call to an undeclared intrinsic is ``E-LOAD-021``. The
*contents* of the intrinsic table are Program 27 Phase 2 work and are not fixed
by this document — only the calling convention, the declaration form, the
reserved-name prohibition below, and the "never a primitive" prohibition are
normative here. §3.10 records the cap the intrinsic table is held to and the
rider slate proposed against it.

**[draft ruling — Phase 1 review, R9 chapter C1]** *Form-head symbols are
reserved against the intrinsic namespace.* An intrinsic call is
``"(" <intrinsic-name> <expr>* ")"``, so an intrinsic whose name collided with
a form head would make ``(field-of it x/y)`` ambiguous between an accessor and
a call. Every head symbol listed as a form tag in §5.2 is therefore reserved:
declaring an intrinsic with one of those names is ``E-LOAD-024``. The
prohibition is checked at load against the §5.2 list, not against a separate
registry, so adding a form tag automatically reserves it.

**Selection** returns the *element* that extremises a score, where a fold
returns the extremised *value*. Eleven systems pick an element and then act on
it — the winning claimant, the winning faction, the best platform, the FPTP
winner, the top claims holder, the principal field (R9 gap analysis §2, Q4) —
and until this chapter no expression form yielded a ``NodeRef`` other than
``self`` and §2.8's effect-list-scoped minted names, so none of them was
authorable.

.. code-block:: scheme

   (update-node (select-max (nodes NodeType/ORGANIZATION)
                            (field-of it organization/claim-strength))
                organization/holds-office (set #t))

- The result type is the query's element type: ``NodeRef`` for ``nodes``,
  ``neighbors`` and ``members-of``; ``EdgeRef`` for ``edges``;
  ``HyperedgeRef`` for ``hyperedges`` and ``hyperedges-of``. ``it`` is bound in
  the score expression exactly as it is in a fold body.
- **The tiebreak is a property of the language, not of each rule.** Ties are
  broken by §2.6's iteration order: the **first** element in ascending id byte
  order wins, for ``select-max`` and ``select-min`` alike. Every frozen system
  that picks an element carries its own tiebreak today, and several carry none;
  hoisting it here means a transcribed rule cannot forget one.
- An empty query is ``E-EVAL-021`` — the same code, for the same reason, as
  ``min``/``max`` over an empty set (§4.4). There is no element to return and
  there is no null.
- The score expression must have a **comparable scalar** static type — ``Int``,
  ``Currency``, ``Probability``, ``Intensity``, ``Coefficient`` or ``Real``.
  ``Bool``, ``Enum<T>``, ``Str``, references and sets are ``E-TYPE-016``.
- **Kind is unconstrained on the score** and the result is kind-neutral (a
  reference has no extent). This is not a hole in §3.4: that law polices
  *aggregation*, where an unweighted mean of an intensive quantity across
  classes or space is the recorded variance error. Ranking elements by an
  intensive field aggregates nothing — it orders — so the weighted-mean
  obligation has nothing to attach to.

**[draft ruling — Phase 1 review, R9 chapter C12]** *No set-algebra operator,
and the deferral is now honest.* One system needs a set intersection (shared
memberships between two nodes). This document adds no ``intersect``,
``union`` or ``difference``: a dedicated operator would need a result type
that is a ``NodeSet`` **not** produced by a ``<query>``, which §3.1's "only
consumable by ``fold``, ``exists``, ``forall``" line and §3.7's
``ceiling(query)`` both assume away. Intersection is expressible today, at
quadratic fuel cost, with C8's naming and C12's reference identity:

.. code-block:: scheme

   (fold count (hyperedges-of a HyperedgeType/COMMUNITY) :as ha
         (if (exists (hyperedges-of b HyperedgeType/COMMUNITY) (= it ha))
             1 0))

The earlier judgement to defer rested on a form that could not actually be
written — ``it`` meant the inner element in both positions and references had
no comparison. Both holes are closed above, so the deferral now stands on its
own: **revisit when a second system asks**, and pay the quadratic cost
visibly in ``:fuel`` until then, where a reviewer can see it.

**Folds and selection are the only expression-position iteration
constructs**, and §2.8's ``for-each`` is the only one in effect position.
There is no recursion, no ``while``, no ``loop``, no user-defined function, and
no way to name a rule from inside a rule. Totality is therefore syntactic, and
the static bound of §3.7 is computable.

2.8 Effects — the typed structural verbs
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   <effect-item> ::= <verb>
                   | "(" "guard"    <cond>  <effect-item>+ ")"
                   | "(" "for-each" <query> <elem-name>? <effect-item>+ ")"

   <verb> ::= "(" "update-node"  <expr> <qname> <update-op> ")"
            | "(" "update-edge"  <expr> <qname> <update-op> ")"
            | "(" "add-node"     <enum-ref> <expr> <field-init>* ")"
            | "(" "remove-node"  <expr> ")"
            | "(" "add-edge"     <enum-ref> <expr> <expr> ":strength" <expr>
                                 <field-init>* ")"
            | "(" "remove-edge"  <enum-ref> <expr> <expr> ")"
            | "(" "add-hyperedge"    <enum-ref> <expr> <members> <field-init>* ")"
            | "(" "update-hyperedge" <expr> <qname> <update-op> ")"
            | "(" "remove-hyperedge" <expr> ")"
            | "(" "update-membership" <expr> <expr> <qname> <update-op> ")"
            | "(" "emit"         <enum-ref> <payload-item>* ")"

   <update-op>   ::= "(" "add"   <expr> ")"
                   | "(" "sub"   <expr> ")"
                   | "(" "set"   <expr> ")"
                   | "(" "scale" <expr> ")"
   <members>     ::= "(" "members" <member-item>+ ")"
   <member-item> ::= <expr>
                   | "(" "member" <enum-ref> <expr> <field-init>* ")"
   <field-init>  ::= "(" <qname> <expr> ")"
   <payload-item>::= "(" <symbol> <expr> ")"

The four ``<update-op>`` forms are exactly today's four-operation effect enum
— ``add`` = ``increase``, ``sub`` = ``decrease``, ``set`` = ``set``,
``scale`` = ``multiply``. The set is closed: a fifth head there — the
``(unset …)`` the frozen estate reaches for (§3.8) — is ``E-PARSE-015``.
Of the **ten** structural verbs, **five** are the
addition the design document's §6.4 audit found necessary (20 of 39 system
modules mutate graph structure); **two** — ``add-hyperedge`` and
``remove-hyperedge`` — are what the Amendment D ruling adds, since if a
hyperedge is a first-class object, minting and retiring one is a first-class
verb; **two** — ``update-edge`` and ``update-hyperedge`` — are what R9
chapters C2 and C12 add, below; and **one** — ``update-membership`` — is what
Amendment AG clause (i) adds, below.

**[draft ruling — Phase 1 review, R9 chapter C2]** ``update-edge``, *and why
the dyadic layer differs from D26.* Eight systems overwrite a standing edge's
attributes (R9 gap analysis §2, Q3): ``value_flow`` on four edge types,
SOLIDARITY decay, MEMBERSHIP accrual, ``field_gradients``, the EdgeTransition
mode fields. Before this chapter nothing in the verb set could, and the
question that had to be answered first is whether D26's refusal to mutate a
hyperedge in place carries over. **It does not.** D26's stated rationale is
that a partially-mutated *member list* must be unrepresentable, so membership
changes go through whole-object replacement and the member-count check stays at
a single point. A dyadic edge has no member list; there is no partial state for
in-place mutation to leave behind, and the ``:max-members`` check it protects
does not exist on the dyadic layer. The rationale is therefore specific to the
construct it was written about, and re-using it here would be an argument from
symmetry rather than from the reason. (C12 revisits the *other* half of D26 —
mutation of a hyperedge's own declared fields — on exactly this reasoning.)

**[draft ruling — Phase 1 review, R9 chapter C2]** ``update-edge`` *takes an*
``EdgeRef``, *not a type-and-endpoints triple.* It mirrors ``update-node``
operand for operand — element, field qname, update-op — because the effect
positions that will supply it are ``it`` inside a ``for-each`` over ``edges``
(§2.8, chapter C6) and the result of ``select-max`` (§2.7, chapter C5), both of
which yield refs. Giving the verb a second arity taking ``<enum-ref> <expr>
<expr>`` would have made it the grammar's only overloaded head — the exact
objection D26 raises against an overloaded ``add-edge``. Rules that hold the
endpoints instead of the ref reach the edge through §2.10's ``edge-between``
accessor, which is one form used in expression position rather than a second
shape of a verb. The R9 gap analysis §2 (Q3) sketched the triple form; D36
records the divergence.

A ``<qname>`` whose owning type (§2.9) is not the element type the verb's
``<enum-ref>`` names is ``E-TYPE-014`` — a **static** check on ``add-node``,
``add-edge`` and ``add-hyperedge``, whose element type is an operand. On
``update-node``/``update-edge`` the element is a reference, which §3.1 gives no
static type, so the same disagreement surfaces at evaluation as ``E-EVAL-033``
(§2.10). A ``<field-init>`` on ``add-edge`` naming the implicit
``<edge-type>/strength`` field is ``E-PARSE-041``: the ``:strength`` operand is
that field's only writer at mint time, and two writers for one field in one
form is an authoring bug rather than a precedence question.

``update-edge`` inherits the structural-verb discipline unchanged: it obeys the
I.15 edge-mode state machine, so a write that would take an edge to a mode the
machine does not admit from its current one is ``E-EVAL-030``, and a store
outside the target field's declared range is ``E-EVAL-020`` (§3.3) — never a
clamp, never a silent no-op.

``add-hyperedge``'s ``<enum-ref>`` is a ``HyperedgeType`` member, its ``<expr>``
is the new hyperedge's id (as ``add-node``'s is a node id), and ``<members>``
names its member nodes. The grammar's ``<member-item>+`` makes a **zero-member
hyperedge unexpressible**; the upper end is the declared ``:max-members``
ceiling of §3.7, checked statically. A member item is a bare ``<expr>`` where
the hyperedge type carries no membership payload, and the annotated ``member``
form where it does — the Amendment AG ruling below.

**[draft ruling — Phase 1 review]** *Id operands are effect-list-scoped
names* (implementation-discovered, 2026-07-30, Phase 1 Task 16). The id
operand above is written as ``<expr>``, but no expression form yields a
*fresh* identity and §2.5 gives rules no way to declare one. The executor
therefore reads it as a **symbol introducing an effect-list-scoped name**
for the minted object: later effect items in the same list may reference it
(roster replacement needs exactly this), the substrate mints the actual
identity, and the no-shadowing discipline of ``E-PARSE-022`` extends to
these names. A non-symbol id operand is a loud error pending the review.

**[draft ruling — Phase 1 review]** *Two verbs, not an overloaded* ``add-edge``.
Membership is minted by its own typed verb rather than by an ``add-edge``
variant carrying a member set. Three reasons, each following the rest of this
document rather than inventing a rule for the occasion: the head symbol *names*
the form (§1.3), so one tag with two arities would be the grammar's only
exception; ``add-edge``'s first operand is an ``EdgeType`` and
``add-hyperedge``'s is a ``HyperedgeType``, so overloading needs a union type
§3.1 does not have; and the member-count ceiling attaches to the hyperedge verb
alone. It also keeps the ruling legible in the grammar itself — **a clique
expansion is not expressible**, because no verb takes a member set and emits
edges.

**[draft ruling — Phase 1 review;** *field-mutation half superseded by the C12
ruling below* **]** *Membership changes are whole-hyperedge replacement.* There
is no ``add-member``/``remove-member`` verb: a rule that changes a formation's
roster emits ``(remove-hyperedge h)`` and then ``(add-hyperedge …)`` in one
effect list, applied in source order (below). This keeps the member-count check
at a single point (§3.7) and makes a partially-mutated hyperedge
unrepresentable. The cost was stated rather than hidden: **per-membership
payload** (the role/strength/visibility fields today's Python
``CommunityMembership`` carries) and **mutation of a hyperedge's own declared
fields** were both inexpressible in the revision that made this ruling. The
second of those is retired immediately below (D65); the first was escalated by
D66 and is retired by the Amendment AG rulings below
(D79–D84), which change what a membership *carries* and leave this row's
member-list discipline exactly as written. Neither was a silent omission.

**[draft ruling — Phase 1 review, R9 chapter C12]** *D26's second half is
closed:* ``update-hyperedge`` *writes a hyperedge's own declared fields.* The
verb mirrors ``update-node`` and ``update-edge`` operand for operand, and it is
added on exactly the reasoning C2 sets out above: D26's rationale is that a
partially-mutated **member list** must be unrepresentable, and writing a
declared field of a hyperedge leaves no member list partially anything. The
member list stays whole-object replacement (``remove-hyperedge`` then
``add-hyperedge`` in one effect list), so the ``:max-members`` check stays at
its single point and D26's actual guarantee is untouched. What is retired is
only the *incidental* consequence that a formation's own state was frozen for
the life of the object.

A ``<qname>`` whose owning type is not the referent's ``HyperedgeType`` is
``E-EVAL-033`` (§2.10), since a ``HyperedgeRef`` carries no static type; the
range and I.15 disciplines apply as they do to the other two update verbs.

**[draft ruling — Phase 1 review, Amendment AG (i)]** ``update-membership`` —
*D26's first half is closed, and the member list still crosses whole.* The
verb writes a **payload field of an existing membership** and nothing else. It
mirrors the other three update verbs in its trailing operands (a ``<qname>``
and an ``<update-op>``) and differs only in its element position, which takes
**two** operands rather than one: BSL mints no reference kind for a
membership, so the pair is named by its key — the hyperedge first, the member
second (§2.12).

.. code-block:: scheme

   (for-each (members-of c HyperedgeType/COMMUNITY)
     (update-membership c it community/visibility (scale 0.9c)))

- **The member list is untouched.** There is still no
  ``add-member``/``remove-member`` verb, and a roster change is still
  ``remove-hyperedge`` then ``add-hyperedge`` in one effect list (D26). The
  ``:max-members`` check therefore stays at its single point, a partially
  mutated member list stays unrepresentable, and Anti-Pattern VIII.9 survives
  verbatim — this verb changes what a membership *carries*, never how many
  objects cross. The payload of a member dropped by a roster replacement does
  not survive it, and re-stating the payload is that idiom's price, unchanged.
- **A pair that is not a membership is** ``E-EVAL-038`` — the member node is
  not in that hyperedge's member list — never a silent no-op and never a
  quietly minted membership. A hyperedge operand that is not of the
  ``<qname>``'s owning ``HyperedgeType``, and a member operand that is not of
  the payload declaration's ``:member`` ``NodeType``, are both ``E-EVAL-033``
  (§2.10 discipline 1, which now reads across two operands rather than one);
  neither is statically checkable, because §3.1 gives references no type.
- The range and I.15 disciplines apply exactly as they do to the other update
  verbs: a store outside the payload field's declared range is ``E-EVAL-020``,
  never a clamp.
- *Why a verb at all, given Amendment AG (iii).* Clause (iii)'s "adds no verb"
  is the ADR177-governed player-action contract:
  ``ai/decisions/ADR177_verb_matrix_ratified_main_ruleset.yaml``,
  ``src/babylon/game/actions/matrix.py``, and
  ``src/babylon/game/actions/registry.py``. The matrix and named files are the
  same closed list as "no intrinsic, no severity rule, no constructor family".
  Clause (i) of the same amendment obliges payload to **mutate only through
  effects**, and ADR189 clause (iv) names the "accessor/verb surface" as
  exactly what this document owes. An effect-position write is therefore
  required by the amendment, not licensed against it; what stays closed is the
  player/state verb registry and the algebra, neither of which this touches.

**[draft ruling — Phase 1 review, Amendment AG (i)]** *Payload is initialised
at mint, totally, against an annotated member item.* Where a hyperedge type
declares any membership payload (§2.9), every item of ``add-hyperedge``'s
``<members>`` list is the annotated form ``(member <enum-ref> <expr>
<field-init>*)``, and its field-inits are **exactly** the declared payload
fields of that *(hyperedge type, member node type)* pair — no more and no
fewer.

.. code-block:: scheme

   (add-hyperedge HyperedgeType/COMMUNITY h
     (members (member NodeType/SOCIAL_CLASS c1 (community/strength 0.4c)
                                               (community/visibility 0.5p))
              (member NodeType/SOCIAL_CLASS c2 (community/strength 0.7c)
                                               (community/visibility 0.2p))))

- *The annotation is D24's fix, for the third time.* The owed field set
  depends on the member's node type, and §3.1 gives a ``NodeRef`` no static
  type, so without the ``<enum-ref>`` the completeness check would be a
  runtime discovery. With it the check is **static**: a missing payload field,
  or a bare member item under a hyperedge type that declares payload, is
  ``E-LOAD-047``. The annotation *asserts* rather than filters, exactly as
  D24's does: a member that is not of the annotated type is ``E-EVAL-033``.
- *Bare items stay legal where nothing is owed*, so every existing
  ``add-hyperedge`` form keeps its meaning and its bytes. A bare item under a
  payload-declaring type is not a shorthand for defaults — there are none.
- A field-init naming a field owned by another type is ``E-TYPE-014`` (the
  static check the minting verbs already carry) and one naming the same
  payload field twice in one member item is ``E-PARSE-041``, the existing
  two-writers-for-one-field code.
- *Why total rather than optional.* ``add-node`` and ``add-edge`` carry
  ``<field-init>*`` and no completeness rule because hydration and later
  effects can seed a node's field; a membership minted at runtime has exactly
  one writer at that moment and no ``:default`` anywhere in the element-state
  half of the language (§3.5's opt-in is a *binding* property). The
  alternative — permit partial mint and let the first read be ``E-EVAL-033`` —
  converts a decidable authoring error into a runtime one, which is the
  direction §4.6 says a chapter must never move the language. ``add-edge``'s
  mandatory ``:strength`` operand is the same reasoning already applied to the
  one field that had no other writer at mint. Hydration carries the identical
  obligation (§3.9 clause 6).

``emit``'s ``<enum-ref>`` is an ``EventType`` member — a member of another
enum kind there is ``E-TYPE-011`` under §2.6's class rule; payload items are
name/expression pairs. There is no string interpolation in a payload.

**[draft ruling — Phase 1 review, R9 chapter C6]** ``for-each`` — *bounded
iteration in effect position.* Nine systems apply a verb once per matching
element (R9 gap analysis §2, Q5): an emit per contributing SOLIDARITY edge,
a write to both endpoints of every value-bearing edge, a decay on every
incident edge, a transition per territory, the two sibling nodes of a class
split. None was expressible: ``<effect-item>+`` fixes arity at parse time,
folds live in expression position only, and ``it`` outside a query context is
``E-TYPE-012`` — so "for every matching element, apply a verb" had no form at
all, and the surveys reached for one-summed-emit fallbacks that are content
rulings in disguise.

.. code-block:: scheme

   (for-each (edges EdgeType/SOLIDARITY)
     (update-edge it solidarity/strength (scale 0.95c))
     (emit EventType/SOLIDARITY_DECAYED (strength (field-of it
                                                            solidarity/strength))))

- ``it`` is bound inside the body to the current element, with the query's
  element type (§2.6's result table). Outside a ``for-each``, ``for-each``'s
  body, or a query predicate or fold body, ``it`` remains ``E-TYPE-012``.
- **The query is materialized against the rule's pre-state**, in §2.6's
  iteration order, before any effect in the rule's list is applied. This is not
  a convenience: §4.2's law that *a rule can never observe its own effects*
  would otherwise be silently repealed by an iteration whose membership
  depended on an earlier verb in the same list. Every expression anywhere in an
  effects list — a verb's operands, a ``guard``'s condition, a ``for-each``'s
  query — is evaluated against the pre-state, and the collected effects are
  then applied.
- **Application order is total.** The body runs once per element in iteration
  order (outer), and the body's own items apply in source order (inner). Nested
  ``for-each`` composes the same way. With §4.2's subject order this leaves no
  unordered reduction anywhere in the language.
- **An empty query applies nothing, and is not an error.** This is the one
  place where an empty set is quiet, and the distinction is principled: §4.4's
  ``mean``/``min``/``max`` must produce a *value* and have none to produce, so
  they are ``E-EVAL-021``; an iteration is a *command*, and "do it to each of
  no elements" is completely determined. The reading that would make it loud
  would also make a correct rule fail on a legitimately empty graph.
- **Totality holds.** The set is materialized before the body runs and its size
  is bounded by the declared ceiling, so this is a bounded iteration and not a
  loop; §3.7 charges it exactly as it charges ``exists``/``forall``.

One consequence worth recording where the port train will look for it: with
``for-each`` in hand, a bulk structural operation (re-parenting every claim of
a collapsing sovereign, severing every exploitation edge) is **expressible in
content**. A Rust bulk primitive for the same job therefore survives only as a
*performance* escape, which needs measurement rather than assertion.

**Effect ordering and application.** Effects are collected in source order and
applied in source order at the point the rule fires. Structural verbs obey the
I.15 edge-mode state machine; a verb that would violate it is an evaluation
error (``E-EVAL-030``), never a silent no-op. Removing a node that does not
exist, adding a node id that already exists, or adding an edge that already
exists are all ``E-EVAL-031`` — absence is never treated as success. The
hyperedge verbs inherit that discipline exactly: removing a hyperedge that does
not exist, adding a hyperedge id that already exists, naming a member node that
does not exist, and naming the **same member twice** in one ``<members>`` list
are all ``E-EVAL-031``. Members are a set; a duplicate is an authoring bug, and
it is never silently deduplicated.

**Prohibited.** There is no I/O, no time source other than a ``:tick``
binding, no
general randomness primitive. Chance enters BSL only through the governed
finite ``choose`` effect with the private realization key that the
:doc:`/reference/determinism-contract` specifies. BSL permits no graph mutation
outside this verb set, no reflection, and nothing unbounded. In particular,
BSL has **no clique
expansion**. The verb set has no member-list-to-pairwise-edge conversion, so
BSL cannot represent the object that Anti-Pattern VIII.9 bans (Amendment D,
D-1).

2.9 Field and manifest declarations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   <deffield> ::= "(" "deffield" <qname>
                      ":type" <type-name>
                      ( ":kind" ( "intensive" | "extensive" )
                      | ":enum-type" <enum-type> )
                      ( ":member" <enum-ref> )?
                  ")"

   <manifest>   ::= "(" "manifest" <symbol> <ceiling>+ <rung>* <adjunction>* ")"
   <ceiling>    ::= "(" "ceiling" <enum-ref> ":ceiling" <int-lit>
                        ( ":max-members" <int-lit> )? ":invariant"? ")"
   <rung>       ::= "(" "rung" <symbol> <enum-ref> <enum-ref>
                        ":via" <enum-ref> ":substrate"? ")"
   <adjunction> ::= "(" "adjunction" <symbol> <qname> <qname>
                        ":rung" <symbol> ( ":weighted-by" <qname> )? ")"

The ``rung`` and ``adjunction`` rows are Amendment AG clause (ii)'s: a scenario
declares the rungs of its scale lattice and the ``allocate``/``aggregate``
instances that run along them. Their meaning, their validation and the two
standing rulings that bind every rung are §3.9's; what belongs here is only
that they are **manifest children** — the same form, and therefore the same
digest, as the ``:invariant`` ceiling rows the substrate ruling keys off
(D85).

A ``ceiling`` row's ``<enum-ref>`` is a ``NodeType``, ``EdgeType`` or
``HyperedgeType`` member. ``:max-members`` is **mandatory** on a
``HyperedgeType`` row and **illegal** on the other two; ``:invariant`` is legal
on a ``NodeType`` or ``EdgeType`` row and illegal on a ``HyperedgeType`` row
(§3.9). Any of those mismatches is ``E-LOAD-042``. The semantics of the two
numbers are §3.7's; the flag's are §3.9's.

**[draft ruling — Phase 1 review, R9 verification repair]** *The manifest must
be complete for the types the content set actually uses.* The grammar demands
``<ceiling>+`` — one row or more — and nothing until now said which rows were
owed. A content set that queries a type, names it in a structural verb,
reaches it through ``the`` (§2.10), or (since Amendment AG (ii)) names it in a
``rung`` declaration, and whose manifest carries no row for that type, is
``E-LOAD-045``. The omission is not survivable by defaulting:
``ceiling(query)`` (§3.7) is not computable without the row, so ``bound(rule)``
has nothing to compare against ``:fuel``; ``the``'s ``E-LOAD-043`` tests for a
ceiling "other than 1" and a *missing* row is neither 1 nor other than 1; and
``:invariant`` (§3.9) is a flag on a row that does not exist, so its
``E-LOAD-013`` check silently never fires. The obligation is scoped to the
vocabulary the content set mentions, not to the whole registry — a scenario
owes no row for a type nothing touches.

**[draft ruling — Phase 1 review]** The design document says intensivity is "a
per-field declaration (``:kind intensive|extensive``) on model fields" without
saying where the declaration lives. It lives here, in BSL content, as
``deffield``. Rationale: the typechecker must be derivable from content alone
for this document to satisfy III.12(a); a kind that lived only in Rust
attribute macros would make a second implementation impossible to write from
the spec. A ``deffield`` whose type or kind disagrees with the kernel's model
registration is ``E-LOAD-022`` — the two must agree, and the kernel is checked
against content, not the reverse.

**[draft ruling — Phase 1 review, R9 chapter C1]** *A field's owner may be a
node type, an* ``EdgeType`` *or a* ``HyperedgeType``. The first segment of a
``deffield``'s ``<qname>`` names the owning graph-element type; until this
revision it could only name a node type, which is what left §2.4's
``EdgeCondition`` row unwritable. A first segment naming no registered
``NodeType``, ``EdgeType`` or ``HyperedgeType`` member is ``E-LOAD-023``.

*The segment↔member correspondence, stated because it was only ever implied.*
``social-class/wealth`` owns off ``NodeType/SOCIAL_CLASS`` by a rendering the
document used from its first revision and never wrote down: **lowercase the
enum member identifier and replace each** ``_`` **with** ``-``. The result must
be a valid ``symbol`` per §1.4; a member whose rendering is not (a leading
digit, say) is ``E-LOAD-033``. Because one namespace of renderings now spans
three enum types, the registry must keep them **pairwise disjoint**: a
``NodeType`` and an ``EdgeType`` (or either and a ``HyperedgeType``) rendering
to the same symbol is ``E-LOAD-032``, checked at load over the whole closed
vocabulary. Disjointness is a property of the vocabulary, so the check runs
once per content set rather than per field.

**[draft ruling — Phase 1 review, R9 chapter C1]** *Every* ``EdgeType``
*carries one implicitly declared field,* ``<edge-type>/strength``, with
``:type coefficient`` and ``:kind extensive``. It needs no ``deffield`` — it is
the field ``add-edge``'s ``:strength`` operand writes (§2.8), and before this
revision the language could write it and never read it back. Re-declaring it
explicitly is ``E-LOAD-001`` (a duplicate field declaration), so there is
exactly one home for its type and kind.

The ``Coefficient`` type is the frozen engine's: the ``sum_strength`` /
``avg_strength`` metric of ``engine/event_evaluator.py:174-175`` reads
``solidarity_strength``, declared ``Coefficient`` in
``models/entities/relationship.py:116``. The ``extensive`` kind is the
load-bearing half of the ruling and is chosen so §2.4's coverage row is
honoured rather than half-honoured: under §3.4 an intensive fold body makes
``sum`` an ``E-TYPE-041`` and ``mean`` legal only with a ``:weight``, so an
intensive ``strength`` would have left ``sum_strength`` inexpressible after all.
The extent being aggregated is the **edge population** — a total tie-weight
over a set of edges is additive in exactly the way an intensity across classes
or space is not, which is the distinction §3.4 exists to police. Authors
wanting a genuinely intensive per-edge attribute (``tension``, a rate)
``deffield`` it ``:kind intensive`` and carry the ``:weight`` obligation, and
the recorded variance error stays caught.

**[draft ruling — Phase 1 review, Amendment AG (i)]** *A membership payload
field is a* ``deffield`` *with a* ``:member`` *operand, not a new top-level
form.* The owner of such a field is a **pair** — a member ``NodeType`` inside a
``HyperedgeType`` — and the question the amendment leaves to this document is
whether ``deffield``'s owner axis can carry a pair. It can, because half the
pair is already where a field's owner has always been: the ``<qname>``'s first
segment renders the ``HyperedgeType`` under D31's rendering rule, unchanged,
and ``:member`` names the other half.

.. code-block:: scheme

   (deffield community/strength   :type coefficient :kind extensive
             :member NodeType/SOCIAL_CLASS)
   (deffield community/visibility :type probability :kind intensive
             :member NodeType/SOCIAL_CLASS)

*Why not a dedicated top-level form.* Two reasons from this document's own
machinery, and neither is ergonomic:

1. §5.5 hashes ``deffield``, ``intrinsic``, ``manifest`` and ``metric`` forms
   into **sibling digests that** ``ContentDigest`` **combines** — and
   ``ContentDigest``'s composition is
   :doc:`/reference/determinism-contract`'s, not this document's. A new
   top-form would owe a new sibling digest and therefore an edit to a document
   this one deliberately does not reach into. A keyword option lands the
   declaration inside a digest that already exists.
2. A keyword encodes as an ``opt`` form under D20, so the declaration needs no
   new form tag, no numeric id and no new atom kind — the same
   cheapest-available proof of additivity §5.2 relies on, and the reason
   §5.6's bytes survive this revision too.

The rules, all decidable at load:

- The ``<qname>``'s first segment must render a ``HyperedgeType`` member. A
  ``:member`` on a ``deffield`` whose first segment renders a ``NodeType`` or
  an ``EdgeType`` is ``E-LOAD-048``; a first segment that renders nothing
  registered is ``E-LOAD-023`` as before. ``:member``'s own operand is a
  ``NodeType`` member — any other enum kind there is ``E-TYPE-011`` under
  §2.6's class rule.
- **One namespace, one declaration.** A membership payload field shares the
  field namespace with the hyperedge type's own fields, so a ``<qname>``
  resolves to exactly one ``deffield`` and a duplicate is ``E-LOAD-001`` like
  any other. Reading or writing a membership field through ``field-of`` /
  ``update-hyperedge``, or a hyperedge's own field through
  ``membership-field-of`` / ``update-membership``, is ``E-LOAD-046`` — static,
  because the declaration and the reading form are both content, and the same
  shape as §2.11's ``E-LOAD-012``.
- **One declaration is one pair.** A hyperedge type whose members are of two
  node types, both needing an axis of the same name, declares two qnames. The
  cost is real and is taken deliberately: it keeps ``<qname>`` → declaration a
  total function, which is what makes both the wrong-form check above and
  §2.8's mint-time completeness check static rather than runtime.
- Kernel agreement is ``E-LOAD-022``, as for any other ``deffield``: the type,
  the kind and the member half are all checked against the kernel's
  registration of the attributed-membership object.

**[Director ruling 2026-08-11, Organization-as-game-object spec §1 Q12]**
*The* ``:kind`` *slot widens to an exclusive-or with* ``:enum-type``, *not
an additional option.* Every ``deffield`` before this ruling carried a
mandatory ``:kind``; an ``enum``-typed field carries none, because there is
no meaningful extensive-or-intensive reading of a member identity — an
aggregation *kind* answers "how does this quantity add up across a
population", and a member has no such answer. §3.4's aggregation-law table
is never even consulted for one: ``Enum<T>`` supports no operation beyond
``=``/``!=`` (§3.1), so an enum-typed value never reaches a fold with an
arithmetic kind to check in the first place — the refusal is structural,
not a special case bolted onto §3.4. The two slots are therefore mutually
exclusive and jointly exhaustive against ``:type``: any ``:type`` other
than ``enum`` requires ``:kind`` and forbids ``:enum-type`` (unchanged from
before this ruling); ``:type enum`` requires ``:enum-type`` — naming a
type ``defenum`` has declared (§2.13) — and forbids ``:kind``. Either
violation is ``E-LOAD-053``. Full declaration and read/write law: §2.13.

**[Train B item 6, #591 — D155]** *The* ``:type`` *slot's eighth name is*
``real``: the finite ``binary64`` lane as a STORED field type, carrying no
declared-domain range law (§3.3's ``E-EVAL-020`` stays the three
unit-interval types' own check). A ``real`` field is ordinary under every
other clause of this section — it takes ``:kind`` like any non-``enum``
type, its owner may be a node, ``EdgeType`` or ``HyperedgeType`` member,
and kernel agreement stays ``E-LOAD-022``. Seed law: §1.5's acceptance
paragraph; the full row: §3.1. A knowing supersession of D98's "not
storable", recorded in the register — not a silent reopening.

2.10 Element accessors
~~~~~~~~~~~~~~~~~~~~~~~~

Accessors are the expression forms that read *from a graph element the rule
already holds a reference to*, as against ``:field``/``:const``/``:metric``
bindings, which read from the rule's own environment. They are the R9
chapter-C1/C2/C3/C9 additions and they all share one discipline, stated once
here.

.. code-block:: text

   <accessor> ::= "(" "field-of"     <expr> <qname> ")"
                | "(" "edge-between" <enum-ref> <expr> <expr> ")"
                | "(" "the"          <enum-ref> ")"
                | "(" "metric-of"    <expr> <symbol> ")"
                | "(" "membership-field-of" <expr> <expr> <qname> ")"

.. list-table::
   :header-rows: 1
   :widths: 20 18 62

   * - Form
     - Result
     - Reads
   * - ``field-of``
     - the field's declared type
     - A declared field of the node, edge or hyperedge the ``<expr>``
       denotes. The ``<qname>``'s first segment names the owning type
       (§2.9).
   * - ``edge-between``
     - ``EdgeRef``
     - The edge of the given ``EdgeType`` from the first node operand to the
       second. Absence is ``E-EVAL-034``.
   * - ``the``
     - ``NodeRef``
     - The unique node of a ``NodeType`` whose manifest ``:ceiling`` is 1.
       A ceiling other than 1 is ``E-LOAD-043``; a graph holding no such node
       is ``E-EVAL-035``.
   * - ``metric-of``
     - the metric's declared type
     - A registered **element-indexed** metric, evaluated at the element the
       ``<expr>`` denotes (§2.11).
   * - ``membership-field-of``
     - the field's declared type
     - A declared payload field of the attributed membership keyed by the
       hyperedge the **first** ``<expr>`` denotes and the member node the
       **second** denotes (§2.12). The ``<qname>``'s first segment names the
       owning ``HyperedgeType`` and its declaration names the member
       ``NodeType`` (§2.9). A pair that is not a membership is
       ``E-EVAL-038``.

**The shared discipline.**

1. *The qname carries the type annotation, the reference does not.* §3.1 gives
   ``NodeRef``/``EdgeRef``/``HyperedgeRef`` no type variables, so a reference
   does not carry which ``NodeType``/``EdgeType``/``HyperedgeType`` it belongs
   to. The owning type therefore comes from the ``<qname>``, exactly as D24
   makes the ``HyperedgeType`` a mandatory operand of ``members-of`` for the
   same reason. A ``field-of`` whose referent is not of the qname's owning type
   is ``E-EVAL-033`` at evaluation — **never** a default value and never an
   absent read.
2. *Absence is not a value.* A ``field-of`` against an element that carries no
   value for a declared field is ``E-EVAL-033`` as well; there is no
   ``:optional``/``:default`` on an accessor, because the opt-in to absence is
   a property of a *binding* (§3.5) and an accessor names its element at the
   point of use. §3.8 gives the re-modelling for genuinely optional axes.
3. *Accessors are reads.* No accessor mutates. The verbs of §2.8 are the only
   writes, and §2.8's ``update-edge`` (R9 chapter C2) is the write dual of
   ``field-of`` over an ``EdgeRef``.
4. *Kind and type propagate from the declaration.* A ``field-of`` expression
   has the ``deffield``'s ``:type`` as its static type and its ``:kind`` as its
   kind (§3.4), identically to a ``:field`` binding of the same field.
5. *The enum-ref operand is kind-checked like any other.* ``edge-between``'s
   operand is an ``EdgeType`` member and ``the``'s is a ``NodeType`` member;
   either naming a member of another enum kind is ``E-TYPE-011`` under §2.6's
   class rule, statically and at load.

**Worked shape.** The §2.4 coverage row, written out:

.. code-block:: scheme

   (fold mean (edges EdgeType/EXPLOITATION)
         (field-of it exploitation/tension)
         :weight (field-of it exploitation/value-flow))

``it`` is an ``EdgeRef`` inside a fold over ``edges`` (§2.6's result table), so
both accessors read the edge under the fold. The ``:weight`` is mandatory here
because ``exploitation/tension`` would be declared ``:kind intensive`` — §3.4's
rule is untouched by this chapter and applies to edge fields exactly as it
applies to node fields.

**[draft ruling — Phase 1 review, R9 chapter C2]** ``edge-between`` *is
well-defined because the triple is a key.* §2.6 fixes the edge iteration order
at ascending ``(source-id, target-id, edge-type)`` lexicographic byte order —
which is a *total* order only if no two edges share that triple, and the two
points at which an edge enters the graph both refuse a second one: §2.8 makes
"adding an edge that already exists" ``E-EVAL-031``, and §3.9's hydration
contract makes a scenario seeding one triple twice ``E-LOAD-044``. Parallel
edges of one type between one ordered pair are therefore not representable
(a ceiling-violating hydration is separately ``E-LOAD-041``), and "the edge
between a and b of type T" denotes at most one element. When it denotes none,
that is ``E-EVAL-034`` — the accessor never yields an absent reference and never
degrades to a no-op write, which is what would happen if ``update-edge`` had
been given endpoint operands and left to skip quietly.

.. code-block:: scheme

   (update-edge (edge-between EdgeType/SOLIDARITY self other)
                solidarity/strength (scale 0.95c))

**[draft ruling — Phase 1 review, R9 chapter C3]** ``the`` *reaches a singleton
carrier without a degenerate fold.* Graph-scope state lives on carrier nodes
(§3.6), and a rule whose domain is some other type has to name one. The
alternative the language already had — ``(fold sum (nodes NodeType/POLITY)
(field-of it polity/imperial-rent-pool))`` — reads as an aggregation, costs a
ceiling factor it does not need, and is a fold whose result happens to be a
single element only because the ceiling is 1. ``the`` says that directly and
proves it statically: legality is conditioned on the manifest's ``:ceiling``
being exactly 1 (``E-LOAD-043``), which is the same declared number §3.7
already uses for the fuel bound, so the ruling adds no second registry. A
graph that holds no such node at evaluation is ``E-EVAL-035`` — a carrier the
scenario forgot to hydrate fails loudly rather than reading as zero.

.. code-block:: scheme

   (update-node (the NodeType/POLITY)
                polity/imperial-rent-pool (sub drawn))

**[draft ruling — Phase 1 review, Amendment AG (i)]** ``membership-field-of``
*keys the pair with two operands and annotates with the* ``<qname>`` — D24's
pattern for the third time, and an accessor rather than a binding source.
A ``:membership`` ``<bind-src>`` was the obvious alternative and is
**rejected** on D56's reasoning verbatim: a bind-src encodes as a two-child
``opt`` under D20 and this one needs two element operands, so it would have
been the only bind-src of its shape; and a binding resolves *implicitly*
against an enclosing body, while a rule that scores memberships holds the
hyperedge and the member explicitly (§2.12's example holds both at once
through ``:as``). The accessor names its element at the point of use, which is
the property §2.10 was written around.

.. code-block:: scheme

   (fold mean (members-of c HyperedgeType/COMMUNITY)
         (membership-field-of c it community/visibility)
         :weight (membership-field-of c it community/strength))

- *Operand order is hyperedge-then-member.* Amendment AG names the object as
  the *(member, hyperedge)* pair, and that spelling is the model's naming, not
  an operand order: here the order runs from the half the ``<qname>``
  annotates — the owning ``HyperedgeType`` — outward to the member, which is
  also ``members-of``'s and ``update-membership``'s order. One order, three
  forms.
- *Both operands are subject to discipline 1.* A hyperedge operand not of the
  qname's owning type and a member operand not of the declaration's
  ``:member`` type are each ``E-EVAL-033``; a pair that is a well-typed
  non-membership — both elements exist, the member is simply not in that
  hyperedge's list — is ``E-EVAL-038``, its own code because it is a different
  fact about the graph and a reader should not have to guess which one a code
  meant.
- *Absence past that is unreachable*, not silent: §2.8 makes payload
  initialisation total at mint and §3.9 clause 6 makes it total at hydration,
  so a membership that exists carries every declared payload field of its
  pair. Discipline 2's "a field the element carries no value for" therefore has
  no membership case to cover, which is the point of paying for the completeness
  check at load.
- *It is a keyed lookup* (D38): charged at 1 + its two element operands, never
  multiplied by a ceiling (§3.7). Type and kind propagate from the declaration
  under discipline 4, so the payload's ``:kind`` reaches §3.4 exactly as a
  ``field-of``'s does — the ``:weight`` above is mandatory because
  ``community/visibility`` is declared intensive.

2.11 Metric registration
~~~~~~~~~~~~~~~~~~~~~~~~~~

§2.5 said ``:metric`` "reads a registered graph-level metric" and stopped
there. It never said who may register one, what determinism obligations a
registration carries, whether a Rust domain crate's per-tick output qualifies,
or whether a metric may be **indexed by element**. The last question is the one
that blocks work: every topological score the OODA seam needs — degree and
betweenness centrality, articulation-point cutsets, isolation — is per-node,
and a graph-scope scalar cannot carry any of them to content.

.. code-block:: text

   <metric-decl> ::= "(" "metric" <symbol>
                         ":type" <type-name>
                         ":kind" ( "intensive" | "extensive" )
                         <domain>
                         ":provider" <symbol>
                     ")"

``<domain>`` is §2.3's production, reused unchanged: ``(domain :graph)``
declares a graph-scope metric read by a ``:metric`` binding, and ``(domain
NodeType/ORGANIZATION)`` declares an element-indexed metric read by
``metric-of``.

.. code-block:: scheme

   (metric betweenness-centrality
     :type coefficient :kind intensive
     (domain NodeType/ORGANIZATION)
     :provider topology-scores)

   (binding centrality :expr (metric-of self betweenness-centrality))

**[draft ruling — Phase 1 review, R9 chapter C9]** *A* ``metric`` *form
declares, it does not define* — exactly as ``intrinsic`` does, and for exactly
the reason D9 gives for ``deffield``: the typechecker and the fuel-bound
checker must be computable from **content alone** for this document to satisfy
III.12(a), and a metric whose type, kind and domain lived only in a Rust
registration would make a second implementation underivable from the spec. The
kernel provides the value; the declaration is checked against the kernel's
registration and a disagreement is ``E-LOAD-025``. An unregistered metric name
remains ``E-LOAD-011`` — never ``0.0``, which is one of §6.3's four corrected
silent degradations.

**Reading a metric through the wrong form for its declared domain is
``E-LOAD-012``** — a graph metric via ``metric-of``, or an element-indexed
metric via a ``:metric`` binding. Both are static: the declaration and the
reading form are both content. At evaluation, a ``metric-of`` whose referent is
not of the declared domain type is ``E-EVAL-036``, and a metric the provider
produced no value for is ``E-EVAL-037`` — the same discipline as §2.10's
accessors, and for the same reason: absence is never a zero.

**Determinism obligations a registration carries.** These are the substance of
the contract, and they are obligations on the *provider*, enforced by review
and by the determinism contract's golden vectors rather than by the
typechecker:

1. A metric is a **pure function of the graph pre-state at the anchor position
   at which it is read**. Not of wall clock, not of RNG, not of I/O, and not of
   any rule's effects within the position.
2. Its value is **stable across every read at one position**. Whether a
   provider recomputes between positions is its own business; what it may not
   do is return two values to two rules at one position.
3. Its arithmetic obeys §4.3 — IEEE-754 basic operations, correctly rounded,
   no FMA contraction, no transcendental that is not a pinned intrinsic. A
   provider that cannot meet that bit-exactly must either declare an ``Int``
   ordinal (which is exact) or carry golden vectors with a **written tolerance
   derivation** in :doc:`/reference/determinism-contract`.
4. **A Rust domain crate's per-tick output qualifies as a provider** only if
   the crate is inside that document's pinned-toolchain set and carries those
   vectors. This is the answer to the question three surveys asked
   independently: the seam is legitimate, and it is not free.

**What enters which hash.** A metric's *name, type, kind and domain* are
content: they are ``metric`` forms, hashed into their own digest exactly as
``deffield``/``intrinsic``/``manifest`` are (§5.5). A metric's *value* is
runtime and appears in **no** content hash — it enters the tick hash only
through the fields rules write from it. An implementation that hashed metric
values directly would be hashing the provider's schedule rather than the
game's state.

**Fuel.** The provider's computation is **not** metered against the reading
rule: a rule cannot bound a betweenness computation, and pretending otherwise
would put a number in ``:fuel`` that means nothing. The *read* costs
``1 + cost(operand)`` (§3.7), the same as any other accessor. The kernel's own
budget for provider work lives in the determinism contract, which is where the
cost honestly went — this document declines to hide it in a rule's meter.

2.12 Attributed membership
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Amendment AG clause (i) (``CONSTITUTION.md`` v3.2.0, ADR189, ratified
2026-08-10) adds a **third element kind** to the exposed hypergraph model of
Amendment D: the *(member, hyperedge)* incidence pair is a first-class
**attributed membership** carrying declared, typed payload fields — the
role/strength/visibility family the frozen ``CommunityMembership`` carries
(``src/babylon/models/entities/community.py``), and the reason a system whose
scoring depends entirely on per-membership payload was unauthorable in the
revision that recorded D66. The amendment rejects the other candidate landing
in its own words: a per-*(member, hyperedge)* dyadic edge would re-expose
precisely the incidence encoding sub-ruling D-1 confines to internal storage.

This section states what the kind **is**. Its three language surfaces live in
the sections that own them: the declaration in §2.9, the read in §2.10, the
write in §2.8.

**Implementation status (Community port train, Task 12):** the kind is
ratified language, **unserved in the engine** — ``membership-field-of``,
``update-membership``, ``deffield``'s ``:member`` operand and §6.2 family 23
all refuse with the AG(i) lane's citation, blocked on the attributed-
membership ceremony, **issue #653** (the evaluator's
``UNSERVED_EXPRESSION_HEADS`` row and the refusal texts name it by number;
the Community port's own un-need is D220 — its worlds carry no payload by
design). The Community train's requirements against the ceremony are
``reports/community-membership-shape-note-2026-08-18.md``.

**[draft ruling — Phase 1 review, Amendment AG (i)]** *A membership is denoted
by its key, and the language mints no fourth reference kind.* A membership is
identified by the pair *(hyperedge, member)* exactly as a dyadic edge is
identified by ``(source-id, target-id, edge-type)``, and every form that
reaches one takes both halves as operands. §3.1 gains no ``MembershipRef``,
§2.6 gains no ``memberships-of`` query head, and §3.1 gains no fourth set
type.

*The rejected alternative, recorded so it is not re-proposed.* A reference
kind would have dragged in a set type, a query head, a ceiling row for a
cardinality ``:max-members`` already declares, and an **ordering key for an
object that has no id** — and the only honest order available is the member
list's, which the keyed forms already give. It would have bought no
expressiveness the pair-keyed forms lack, at the price of four new obligations
in a chapter whose amendment re-seals the closure (AG (iii)). The element kind
is first-class in the *model*, where the amendment puts it; in the *language*
it is named by its key, which is how the language already treats the only
other keyed element it has.

**What the kind carries, stated against the amendment's obligations:**

- **Typing.** Payload fields are typed exactly like node, edge and hyperedge
  fields: one ``deffield``, one ``:type`` from §3.1's table, one ``:kind``
  under §3.4. Nothing about §3.2's currency lane, §3.3's promotion rule or
  §3.4's aggregation law is special-cased for them.
- **Iteration order.** The memberships of one hyperedge iterate in the member
  list's ruled order — **ascending member node-id byte order** (D25) — so a
  fold over ``members-of`` reading payload is ordered, and it is ordered by
  the key §2.6 fixed in this document's first revision. The kind introduces no
  new order and no new tiebreak.
- **Ceiling.** ``:max-members`` remains the *only* membership cardinality axis
  (AG (i)). A payload field adds no axis, ``ceiling(members-of)`` is unchanged
  (§3.7), and a fold over members reading payload is bounded by the same
  declared number as a fold over members reading nothing.
- **Hashing.** Payload is element **state**, hashed exactly as a node's, an
  edge's or a hyperedge's declared field is. This document does not restate
  the tick-hash field set — that is
  :doc:`/reference/determinism-contract`'s — it fixes only that membership
  payload is state of the same standing as the other three kinds' fields and
  is not exempt from it. The *declaration* is content and hashes into the
  ``deffield`` digest (§5.5), whose shape is unchanged because ``:member``
  encodes as an ordinary ``opt`` (D20).
- **Mutation.** Effects only: ``update-membership`` (§2.8) is the sole writer
  after mint, and no accessor mutates (§2.10 discipline 3).
- **Anti-Pattern VIII.9 survives verbatim.** A member list crosses **whole**,
  never ``C(n,2)``. No verb converts a member list into pairwise anything,
  attributed membership changes what a membership carries rather than how many
  objects cross, and the fuel bound over an attributed member list is still
  ``Σ|members|`` at the declared ceilings (§3.7) — linear in the incidence
  count, as it was before the payload existed.

**Worked shape — the read this kind exists for.** A two-hop fold naming the
outer element with ``:as`` (D54), reading payload off the inner one:

.. code-block:: scheme

   (binding peak-exposure :expr
     (fold max (hyperedges-of self HyperedgeType/COMMUNITY) :as c
           (fold mean (members-of c HyperedgeType/COMMUNITY)
                 (membership-field-of c it community/visibility)
                 :weight (membership-field-of c it community/strength))))

Neither operator is decoration. The inner ``:weight`` is mandatory because
``community/visibility`` is declared ``:kind intensive``, so §3.4 makes the
unweighted mean ``E-TYPE-042``. The outer operator is ``max`` — kind-neutral,
legal over a body of any kind — and it is deliberately **not** ``sum``: the
inner weighted mean is itself intensive (D90), and summing per-community
visibilities across the communities one class belongs to is ``E-TYPE-041``,
which is the recorded variance error in the very shape this section exists to
enable. What the binding names is therefore the class's *peak* community
exposure, a measure; a total would have to be built from an extensive payload,
and ``community/strength`` is the extensive one.

The kind law bites on membership payload exactly as it bites on node and edge
fields — which is what "typed exactly like node/edge fields" costs, and the
reason the amendment wrote it that way.

2.13 Declaring enums and the graph vocabulary
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**[Director ruling 2026-08-11, Organization-as-game-object spec §1 Q12 —
approved twice: the live brainstorming popup and the written spec review]**
§3.1's seventh ``<type-name>`` row needed a declaration form and a
governing law of its own; this section is that form. It is also, because
the same ruling's Q1/Q15 consumers needed the graph vocabulary populated
rather than left permanently empty (§3.6's own text: that registry "today
has zero production call sites"), the closed graph vocabulary's own
declaration form. **This section's** ``enum`` **row explicitly supersedes D94's
exclusion** — D94 sealed §3.1's table at six rows and wrote, in its own
words, that "no ``<type-name>`` position can name" a seventh kind.
Recorded, with the supersession named explicitly, as D101 below.

.. code-block:: text

   <defenum>       ::= "(" "defenum" <enum-type>
                            "(" <enum-member>+ ")" ")"
   <defvocabulary> ::= "(" "defvocabulary" <enum-type>
                            "(" <enum-member>+ ")" ")"

Both are ``<top-form>``\ s (§2.2), parsed and validated at content load
like ``deffield``/``manifest``/``metric``. ``<enum-type>`` is §1.4's
existing production (an uppercase-initial identifier — the
reader's existing ``<enum-ref>`` lexing, unchanged by this section) and
``<enum-member>`` is §1.4's existing ``<enum-member>`` production, also
unchanged. ``defvocabulary``'s type-name operand is additionally
**semantically** restricted to §3.6's own closed set — ``NodeType`` /
``EdgeType`` / ``HyperedgeType`` / ``EventType`` — which needs no dedicated
grammar production: the restriction is a load-time check
(``E-LOAD-030`` if the name is not one of the four, the same code §3.6
already uses for an unregistered ``<enum-ref>`` type name), not a lexical
one, exactly as a ``ceiling`` row's ``<enum-ref>`` operand is syntactically
any enum-ref but semantically restricted to a registered graph-element
type (§2.9). A duplicate ``defenum`` type name, or two ``defvocabulary``
forms naming the same ``<enum-kind>``, is ``E-LOAD-001`` like any other
duplicate declaration; **member order inside a** ``defenum`` **is
normative** — it is the ordinal §3.1's ``enum`` row stores — while member
order inside a ``defvocabulary`` is not (§3.6's registry has never had an
ordinal; membership is the only fact it carries).

**Two registries, one lexical production, never a guess between them.**
An ``<enum-ref>`` (§1.4, ``Type/MEMBER``) is checked against exactly one
of two registries, and *where the reference sits* — never a fallback
search — decides which:

- In a §2.6 class-rule operand position (D74) — a query head, ``the``, a
  structural verb's type operand, ``emit`` — the type name is checked
  against the **closed graph vocabulary** (§3.6), populated by
  ``defvocabulary``. Unknown-type and unknown-member there stay
  ``E-LOAD-030`` / ``E-LOAD-031``, exactly as §3.6 already states; a
  content set that declares no ``defvocabulary`` for a kind leaves that
  kind's checking exactly as inert as it is today, which is what makes
  this section's arrival backward-compatible with every existing content
  set (none of them declares one).
- In an ``:enum-type``-declared field's value position (§2.9) — a
  ``.bscn`` node/edge body seeding it, or a structural-verb write to it —
  the type name is checked against the **content-declared**
  ``EnumRegistry`` type ``defenum`` names, resolved at the field's own
  ``:enum-type`` declaration. Unknown type there is ``E-LOAD-054``;
  unknown member is ``E-LOAD-055``.

The two registries are deliberately not one, and reusing one for the
other's job would break a real invariant each way: ``ClosedVocabulary``
(§3.6) sorts and deduplicates its members by design, which is correct for
a set with no ordinal to preserve and wrong for one where declaration
order IS the stored value; an ``EnumRegistry`` type's member order is
that ordinal (§3.1's ``enum`` row) and must never be reordered. Forcing
the structural kinds through an ordinal-preserving registry would gain
them nothing; forcing a ``defenum`` type through the sort-and-dedup
registry would silently corrupt whatever field values its ordinals
already back.

**The write/read law.** An ``:enum-type``-declared field is written and
read **only** as an ``<enum-ref>`` of its declared type, in every
direction, with no fallback and no default member:

- A ``.bscn`` node or edge body seeding such a field with anything other
  than a matching ``<enum-ref>`` — a bare number, an ``<enum-ref>`` of a
  different declared type, any other atom — is ``E-LOAD-056``. The
  ordinal is never a surface value; there is no "seed it as a number and
  let the engine resolve the member" path, the same "a name outside the
  registry is a load error, never a fallback" law §3.6 already states for
  the structural kinds (D31's own module doc).
- A structural-verb write (``update-node`` and its siblings, §2.8) to
  such a field, evaluating to anything other than a matching
  ``<enum-ref>`` — most concretely a bare ``Real``, the eval-time form of
  the same mistake — is ``E-EVAL-042``. This is the same law re-checked
  at the one boundary content cannot be checked once and for all at
  load, the same two-site pattern the store boundary and range checks
  already use (§3.7, §4.3). ``E-EVAL-042`` also covers an ``add``/
  ``sub``/``scale`` update-op targeting such a field (D118): ``Enum<T>``
  supports no arithmetic (the "No aggregation kind" paragraph below
  states the same fact for folds), so combining a stored ordinal with an
  operand's is never a coherent write against this field's declared type
  — the same write-shape law, not a special case bolted onto it.
- A read of such a field (via a ``:field`` binding, §2.5) renders the
  stored ordinal back to its member as a ``Value::Enum`` — never a bare
  number, so a ``when`` guard or any other comparison always compares
  members, never ordinals a rule author could confuse for magnitude. A
  stored value that fails to round-trip against the field's declared
  type — non-integral, or outside ``[0, member-count)`` — is a loud
  integrity failure at the read boundary, never a clamp and never a
  silently-substituted default member: Constitution III.11 binds a
  corrupted store exactly as it binds a malformed load.

**No aggregation kind.** §2.9's widened ``<deffield>`` production
structurally omits ``:kind`` for the ``enum`` branch; the full statement
of why — ``Enum<T>`` supports no arithmetic, so an enum-typed value never
reaches §3.4's aggregation-law table with a kind to check — is recorded
there, beside the production it explains, rather than repeated here.

**The** ``field-of`` **read path (D102, discharged).** An
``:enum-type``-declared field is read through a ``:field`` binding
exactly as any other node field is (§2.5), **and** through §2.10's
``field-of`` accessor — the two read routes agree, rendering the same
``Value::Enum`` through the same ordinal→member conversion. D102
originally deferred ``field-of`` on an enum-declared field, refusing it
loudly at load with no error code; the Territory port train (P27)
discharged the deferral once Territory's ``_find_sink_node`` became its
chartered first consumer — a rule reading a *neighbor's* enum-declared
field has no ``:field`` binding to reach it through, exactly the gap
D102 named. Discharging the read path narrows nothing else: a
``field-of``-sourced ``Enum<T>`` value is still refused as a
``select-max``/``select-min`` score (D46, ``E-TYPE-016``, §2.7) and still
refused in every arithmetic lane (§2.13's no-arithmetic law, D101/D118)
— those two refusals were always independent mechanisms, never D102's
own job, so nothing about them changed. See D102 below for the full
history and the reference implementation.

**Declaration sharing via a prelude (D157, Train B item 4, issue #591).**
Every declaration this section describes — ``defenum``, ``defvocabulary``,
``defconst``, ``deffield`` — is otherwise scenario-scoped: §3.9's own
one-``(scenario …)``-form-per-source rule means a second scenario wanting
the same ``defenum`` type has always had to re-declare it verbatim,
byte-for-byte, or refuse to load at all. A **declaration prelude** is the
escape from that: a source string holding ONLY these four top-forms — no
``(scenario …)`` wrapper, and never ``node`` / ``edge`` / ``edge-attr`` (a
prelude declares, it never seeds a graph). The Rust engine seam,
``babylon_bsl::scenario::load_scenario_with_prelude(prelude_src,
scenario_src, graph)``, reads the prelude first and threads its
registries — the ``EnumRegistry``, the ``defvocabulary``/vocabulary trio,
``consts``, ``fields`` — into the scenario load that follows, so a field
declared ``enum <Type>`` or a node/edge enum-ref resolves a
prelude-declared type exactly as if the scenario itself had declared it.

This does not relax the closed-declaration discipline the rest of this
section states, and **the relaxation it DOES grant is scoped to**
``defenum`` **alone** — a scenario re-declaring a ``defenum`` type its
prelude already declared must match EXACTLY (same name, same members, same
order) to be recognized as the same fact rather than a conflict —
``EnumRegistry::declare``'s identical-recognition arm returns the
prelude's own ``EnumTypeId``; a re-declaration that reorders, renames,
adds, or drops a member still refuses with ``E-LOAD-001``'s
``DuplicateType``, exactly as two colliding ``defenum`` forms inside one
file always have. The other three prelude-eligible forms gained NO such
arm: ``deffield``, ``defconst``, and ``defvocabulary`` each still refuse
ANY second declaration of the same name unconditionally, whether or not it
is identical to the first — a scenario re-declaring a prelude-supplied
``deffield``/``defconst``/``defvocabulary``, even byte-for-byte verbatim,
still refuses. Content sharing one of those three via a prelude must NOT
re-declare it in the consuming scenario. Nothing about the two-registry law
above, the write/read law, or the no-aggregation-kind rule changes when a
``defenum`` type arrives via a prelude rather than a bare declaration — a
prelude only changes WHERE a declaration's text lives, never what
declaring it means.

3. Static semantics
---------------------

Every check in this chapter runs at **content load**, before any tick executes.
All of them are loud (Constitution III.11): there is no warning level, no
degraded mode, and no rule that loads "partially".

3.1 Types
~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 20 30 50

   * - Type name
     - Domain
     - Notes
   * - ``int``
     - ``i64``
     - Counts, ticks, thresholds. Overflow is loud (§4.3).
   * - ``bool``
     - ``{#t, #f}``
     - The result type of every ``<cond>``.
   * - ``currency``
     - ``i128`` micro-units, ``[0, ∞)``
     - Fixed point. See §3.2.
   * - ``probability``
     - ``binary64``, ``[0.0, 1.0]``
     - Kernel scalar.
   * - ``intensity``
     - ``binary64``, ``[0.0, 1.0]``
     - Kernel scalar.
   * - ``coefficient``
     - ``binary64``, ``[0.0, 1.0]``
     - Kernel scalar.
   * - ``enum``
     - one member of a content-declared closed ``defenum`` type
     - **§2.13 addendum** (Director ruling 2026-08-11, Organization-as-
       game-object spec §1 Q12 — approved twice, live popup and written
       spec): the declared-order ordinal of the member, stored in the
       existing binary64 attribute lane (§5.2's ``0x02`` atom row —
       zero bytes of any existing golden move). The ordinal is never a
       surface value: written and read ONLY as ``<EnumType>/<MEMBER>``
       (an ``<enum-ref>``, §1.4); comparable with ``=``/``!=`` only, and
       only to the same declared type; carries no aggregation ``:kind``
       (§2.9, §3.4). Supersedes D94's exclusion of a declarable
       closed-enum row — see D101 in the register below.
   * - ``real``
     - ``binary64``, finite
     - **Train B item 6 (#591) — the eighth declarable row, D155:** the
       honest declared home for what the store already does — a verbatim
       finite ``binary64`` attribute carrying NO declared-domain range law
       (§3.3's ``E-EVAL-020`` is the unit-interval types' check, not this
       row's). Seeds accept int / ``p``/``i``/``c`` / ``r`` literals, each
       converted by the one scaled-literal contract (``unscaled /
       10^scale``, §1.5); writes store any finite ``binary64`` verbatim.
       There is no arbitrary-precision fractional literal — ``E-LEX-021``
       still refuses bare floats (#591 item 5's territory, not this
       train's). Supersedes D98's "not storable" — see D155 in the
       register below.
   * - ``Real``
     - ``binary64``, finite
     - The **unbounded intermediate** type (§3.3) — the typechecker
       classification of every binary64 expression. The STORABLE spelling
       is the lowercase ``real`` row above (D155); this capitalized row
       remains the expression-type name, exactly the ``enum``/``Enum<T>``
       split's two-jobs-one-root pattern.
   * - ``Ratio``
     - ``binary64``, ``(0, ∞)``
     - **§1.5 addendum** (Director ruling 2026-08-11, #492/ADR194): a
       declared-domain positive scale factor. Not storable. Its one legal
       operation is ``Currency × Ratio`` (§3.2).
   * - ``Enum<T>``
     - members of closed enum ``T``
     - Comparable with ``=``/``!=`` only, and only to the same ``T``.
   * - ``NodeRef`` / ``EdgeRef`` / ``HyperedgeRef``
     - one graph element
     - Produced by ``self``, ``add-node``, ``add-hyperedge``, and query
       elements (``it``, and the ``:as`` names of §2.6). **No** reference
       carries its ``NodeType``/``EdgeType``/``HyperedgeType`` statically —
       there are no type variables — which is why §2.6's hyperedge queries
       take the type as an operand and why §2.10's accessors take the owning
       type in their ``<qname>``. Consumable by §2.10's accessors and by the
       element-position operands of §2.8's verbs, and comparable for
       **identity** with ``=``/``!=`` against a reference of the same kind
       (``E-TYPE-017`` otherwise, §2.4). There is no ordering on references.
   * - ``NodeSet`` / ``EdgeSet`` / ``HyperedgeSet``
     - the result of a ``<query>``
     - Only consumable by ``fold``, ``exists``, ``forall`` (and §2.7's
       selections, which take a ``<query>`` in the same position). A set holds
       each element **once**: a query result is duplicate-free whatever
       multiplicity of edges or memberships produced it, so a ``count`` over
       ``neighbors`` counts nodes and not traversals (§2.6).
   * - ``Str``
     - UTF-8, NFC
     - Only ``:material-basis`` and vector ids. No operations.

There are no type variables, no subtyping, no coercions, and no user-defined
types. Every expression has exactly one static type, computed bottom-up.

**[Director ruling 2026-08-11, ADR191 R4; the seventh row minted the same
day by a separate, later ruling — see below]** *A* ``<type-name>`` *is a
lowercase* ``symbol``, and the seven rows above that name one are its whole
vocabulary. The first seven rows — ``int``, ``bool``, ``currency``,
``probability``, ``intensity``, ``coefficient``, and ``enum`` — are the
types a ``deffield`` or a ``metric`` may write after ``:type``; the first
six are spelled here exactly as §1.4 lexes them, which the capitalized
spellings this table used to carry were not — no capitalized run matches
any §1.4 atom class, so that spelling was unlexable as written. That
six-row closure was recorded as D94, and its own text sealed the table by
name: no ``<type-name>`` position can name a seventh kind. **The seventh
row, ``enum``, was minted afterward by the Director's own ruling** (the
Organization-as-game-object spec, §1 Q12, 2026-08-11 — approved twice,
once live in the brainstorming session and once again reviewing the
written spec, "sealed twice this month" being the question put to her
directly) — a knowing supersession of D94's closure, not a silent
reopening. §2.13 is the row's declaration form and full law; D101 in the
register below records the supersession explicitly, by name, so a reader
of D94 alone is never left believing the table is still six rows.
``Enum<T>`` (the next row) is unaffected: it remains the *typechecker*
classification of an ``<enum-ref>`` expression's static type, while
``enum`` is what a ``deffield`` declares to make a field HOLD one member —
two different jobs that happen to share a root word, not one construct
under two names. The remaining rows — ``Real``, ``Enum<T>``, the three
reference kinds, the three set kinds and ``Str`` — are typechecker types
that no ``<type-name>`` position can name: ``Real`` is the unbounded
intermediate and is not storable (§3.3), the reference and set kinds are
produced by expressions and never declared, and ``Str`` appears only at
``:material-basis`` and in vector ids. They keep their capitalized names
because those names are this document's, not tokens; nothing lexes them,
so nothing needed re-spelling. D94 itself is left unedited (Immutability
of History: it captured what was true through that review), and this
paragraph is the current-law correction sitting beside it.

**[Train B item 6, #591 — the eighth row, D155]** *The eighth declarable
row,* ``real``, *was minted by Train B item 6* — a knowing supersession of
D98's "``Real`` is not storable", the same posture D101 took toward D94.
``real`` (lowercase, §1.4-lexable, D94's own spelling law) is what a
``deffield`` or a ``metric`` writes after ``:type`` to declare a field in
the finite ``binary64`` lane with NO declared-domain range law; ``Real``
(capitalized) remains the *typechecker* classification of every binary64
expression — the ``enum``/``Enum<T>`` two-jobs-one-root pattern exactly.
The "remaining rows" sentence above is left unedited as history; the
current law is: the reference and set kinds, ``Enum<T>``, ``Str`` and
``Ratio`` are the typechecker types no ``<type-name>`` position can name,
and ``Real`` is no longer among them. D155 in the register below records
the supersession explicitly, by name.

``Ratio`` (added by the §1.5/§3.2 addendum below, #492/ADR194 — recorded as
D99, not a re-opening of D94) holds the non-storable-literal category
alone after D155: it is produced by a literal and consumed by one
operator, never declared as a field's or a metric's type.

3.2 Currency operator and rounding table
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Copied **verbatim** from the design document §6.1, which is the authority:

    Currency is **i128 micro-units** with ``checked_*`` arithmetic everywhere;
    overflow is a loud III.11 failure — never wrapping, never saturating.
    […] Operator semantics are pinned, never implicit:
    ``Currency ± Currency → Currency`` (checked);
    ``Currency × Coefficient → Currency``, rounded half-even to micro-units;
    ``Currency ÷ Currency → Coefficient``, computed at i256 intermediate width
    then rounded half-even; ``Currency ÷ integer → Currency``, half-even.
    Truncation is never implicit; intermediate widths and rounding points live
    in the III.12(a) reference with conformance vectors.

Those are the **only** legal operations mixing ``Currency`` with anything
else — **plus the one addendum below** (Director ruling 2026-08-11,
#492/ADR194): ``Currency × Ratio``. Every other mixed-lane expression is
``E-TYPE-030``. In particular ``Currency + Real``, ``Currency × Currency``,
and ``Currency × Int`` are type errors; multiply by a ``Coefficient`` or a
declared-domain ``Ratio``, or divide by an ``Int``, instead. An UNDECLARED
coefficient greater than ``1`` in Currency-multiply position — a bare
``Real`` (a computed value, or a ``p``/``i``/``c`` binding) outside
``[0,1]`` — stays exactly ``E-TYPE-030``: the fifth operation is legal only
for the ``Ratio`` type specifically, never for an out-of-range ``Real``.

Filling in the three points the verbatim table leaves to this document:

- ``Currency ± Currency`` is checked on **both** ends: a result below ``0`` is
  ``E-EVAL-010`` (the domain is ``[0, ∞)``, matching the ``ge=0`` constraint
  the Python ``Currency`` type carries today), and a result exceeding ``i128``
  range is ``E-EVAL-011``.
- ``Currency ÷ Currency`` with a zero divisor is ``E-EVAL-012``; the
  ``Coefficient`` result must land in ``[0,1]`` or it is ``E-EVAL-013``.
- ``Currency ÷ integer`` with a zero or negative divisor is ``E-EVAL-012``.

*Half-even, defined.* Given an exact rational result ``r`` and a target
granularity ``g`` (``10^-6`` for micro-units), the stored value is the multiple
of ``g`` nearest ``r``; when ``r`` is exactly midway between two multiples, the
one whose micro-unit integer is **even** is chosen. Implementations must
compute this from exact integer arithmetic at the stated intermediate width
(``i256`` for ``Currency ÷ Currency``) — never by converting to ``binary64``
and rounding there. The same algorithm is what the ``round-half-even``
intrinsic exposes to rules.

**Addendum — the declared-domain scale operation (Director ruling
2026-08-11, #492/ADR194).** ``Currency × Ratio → Currency``, rounded
half-even to micro-units — the identical algorithm and rounding law as
``Currency × Coefficient`` above, over ``Ratio``'s ``(0, ∞)`` domain (§1.5)
instead of ``Coefficient``'s ``[0,1]``. Commutative in source position, like
``Currency × Coefficient``: both ``(* currency ratio)`` and
``(* ratio currency)`` are legal. This is the ONE new operation the ruling
selected — "scalar multiplication isn't new mathematics" — added to the
verbatim table above rather than inside it, so the design document's own
text stays an exact quotation.

*Why this closes director-gate #492.* Territory's eviction pipeline
(``rent_spike_multiplier``, declared domain ``(0, ∞)``), Metabolism's
``entropy_factor`` (``(1.0, 3.0]``) and Lifecycle's ``pareto_alpha``
(``(0, 10]``), ``early_mortality_modifier`` and
``carceral_transition_modifier`` (``[0, 10]``, of which this addendum's
``Ratio`` lane carries the open interior ``(0, 10]`` — the zero endpoint's
disposition follows separately, §1.5) are runtime-moddable coefficients whose
declared domain exceeds ``[0,1]`` — §1.5's ``p``/``i``/``c`` cap and §3.2's
original table gave them no legal way to multiply ``Currency`` **or even to
be written as a literal at all** (`content/rules/lifecycle.bsl`'s header,
recorded while porting Lifecycle: "there is no legal ``defconst`` for a
value like ``1.5``… that is not itself a dollar amount"). ``Ratio`` closes
the literal gap (§1.5); this addendum closes the operator gap. Porting
these five rules to BSL is explicitly OUT of scope for the change that adds
this addendum — it lands the machinery only, in its own right-sized,
reviewable unit; each consumer ports in its own train.

*Domain rule and error codes.* ``Ratio``'s own domain is ``(0, ∞)`` — a
literal outside it is ``E-LEX-027`` (§1.5), loud at lex time, exactly as
``p``/``i``/``c``'s ``[0,1]`` cap is ``E-LEX-024``. A consumer needing a
TIGHTER domain narrows it via the Rust reference's ``defconst``
``:floor``/``:cap`` extension — Rust/``.bscn``-dialect machinery, not an rst
production (§1.5's addendum explains the D93 boundary this respects).
``:cap`` narrows the UPPER end INCLUSIVE (``pareto_alpha``'s ``(0, 10]``:
``:cap 10r``); ``:floor`` narrows the LOWER end EXCLUSIVE
(``entropy_factor``'s ``(1.0, 3.0]``: ``:floor 1r``, load-bearing — the
floor IS the thermodynamic point, not decoration). Each declared bound is
checked **twice**, per III.11's loud-failure standard applied at both
checkpoints rather than trusted once: at the ``defconst``'s own declaration
(``E-LOAD-052`` — the declared literal itself must not violate the domain it
declares, including ``floor < cap`` self-consistency when both are present)
and again at every ``Currency × Ratio`` evaluation that consumes it
(``E-EVAL-041`` — the operation re-checks rather than trusting a value
handed to it, the same defense-in-depth the store boundary already
practices at ``E-EVAL-020``). Through ``defconst`` — the only producer of a
bounded ``Ratio`` this revision wires — the two checks can never disagree
(the value is fixed between load and use), so ``E-EVAL-041``'s live case
today is a value reaching the operator through any OTHER path a future
revision adds; the check exists now so that path inherits it rather than
needing to invent it. Reference implementation: ``reader::classify_ratio``
(``E-LEX-027``), ``scenario::load_defconst``/``load_ratio_defconst``
(``:floor``/``:cap``, ``E-LOAD-052``), ``evaluator::Value::Ratio``/
``currency_mul_ratio`` (``E-EVAL-041``), ``babylon_kernel::Currency::
mul_ratio`` (the arithmetic, mirroring ``mul_coefficient``) — all in
``rust/crates/babylon-bsl/src/`` and
``rust/crates/babylon-kernel/src/currency.rs``. Recorded as **D99**; see the
Draft-Ruling Register.

3.3 The two numeric lanes
~~~~~~~~~~~~~~~~~~~~~~~~~~~

BSL has exactly two numeric lanes and they never mix implicitly:

1. **Fixed point** — ``Currency``, ``i128`` micro-units, exact, table of §3.2.
2. **Binary64** — ``Probability``, ``Intensity``, ``Coefficient``, ``Real``,
   and ``Int`` when it appears in a binary64 expression.

**[draft ruling — Phase 1 review]** *Bounded-scalar arithmetic promotes to*
``Real``. ``(+ p q)`` where both are ``Probability`` has type ``Real``, not
``Probability``: adding two probabilities can leave ``[0,1]``, and silently
clamping is exactly the kind of quiet degradation III.11 forbids. The range
check happens once, at the **store boundary**: an ``update-node`` whose
resulting value falls outside the target field's declared range is
``E-EVAL-020`` — a loud runtime failure, never a clamp. Comparisons accept
``Real`` on either side. ``Int`` promotes to ``Real`` in a binary64 expression
but never to ``Currency``.

3.4 The intensivity kind rule
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Every ``deffield`` declares ``:kind intensive`` or ``:kind extensive``. Kind is
a property of the **field**, not of the scalar type: ``wealth`` (Currency) is
extensive, ``consciousness`` (Intensity) is intensive, and there is no type at
which that is decidable. Kind propagates through expressions:

- a literal is **kind-neutral**;
- a ``:field`` binding and a ``field-of`` accessor (§2.10) both carry the
  ``deffield``'s declared kind, whether the owning type is a node type, an
  ``EdgeType`` or a ``HyperedgeType`` — the kind rule does not care which, and
  the implicit ``<edge-type>/strength`` field is ``extensive`` (§2.9); a
  ``membership-field-of`` accessor carries its payload ``deffield``'s declared
  kind the same way **[draft ruling — Phase 1 review, Amendment AG (i)]**, so
  an intensive payload folded with an unweighted ``mean`` is ``E-TYPE-042``
  like any other intensive field;
  a ``:const`` binding is kind-neutral **[draft ruling — Phase 1 review]** (a
  coefficient has no extent); a ``:metric`` binding and a ``metric-of``
  accessor carry the **declared** ``:kind`` of their §2.11 registration
  **[draft ruling — Phase 1 review, R9 chapter C9]**, which supersedes the
  metric half of D12 — the earlier revision made them kind-neutral only
  because there was nowhere to declare a kind, and §2.11 is that place;
- ``+``/``-`` require both operands to have the same kind, or one to be
  kind-neutral; the result carries the non-neutral kind. Mixing intensive with
  extensive is ``E-TYPE-040``;
- ``*``: kind-neutral absorbs either way; whichever operand is non-neutral,
  its kind carries through unchanged. Extensive × intensive is legal too, result extensive — a
  stock scaled by a fraction or a rate (``population × per-capita-rate``) is
  the ordinary case, not an area-of-an-area, since multiplying by a
  dimensionless factor does not create a new dimension the way multiplying an
  extensive quantity by itself does. Intensive × intensive is
  **[draft ruling — Phase 1 review, #491 T1, controller adjudication
  2026-08-18]** now **licensed** too, result **intensive** — a rate scaled by
  a dimensionless coefficient is still a rate (the same "scale by a
  dimensionless factor" reasoning as extensive × intensive, applied to the
  other pairing); see the Draft-Ruling Register. Extensive × extensive alone
  stays ``E-TYPE-040`` (an area-of-an-area) — the one case this bullet has
  always named, and the only same-kind-squared combination still refused;
- ``/``: kind-neutral absorbs the same way. Extensive ÷ extensive is
  **[draft ruling — Phase 1 review, #491 T1]** **licensed**, result
  **intensive**: ``w̄ = wealth ÷ population`` is the textbook definition of an
  intensive quantity (density = mass ÷ volume), the same unit-algebra
  standing D90 (§2.12) already took for the symmetric weighted-mean gap —
  see the Draft-Ruling Register. Intensive ÷ intensive is
  **[controller adjudication, 2026-08-18]** now **licensed** too, result
  **intensive** — a ratio of two intensive quantities is a dimensionless
  share; simplex normalization (dividing a part by a whole built from parts
  of the same kind) is the canonical intensive operation. An extensive
  operand paired with an intensive one, in either position, is now
  ``E-TYPE-040`` — the pre-split bullet named this pair TWICE,
  contradictorily ("extensive if exactly one operand is extensive" and
  "intensive if exactly one is intensive" both matched it, and it was
  assigned neither), so this is a newly-minted prohibition resolving that
  internal contradiction conservatively, not a continuation of a prior
  refusal: deliberately conservative, a Phase-1 review item, and division
  is not commutative, so ``*``'s licensed mixed case does not carry over
  to it;
- ``if`` requires both branches to have the same kind, or one to be
  kind-neutral (the same absorption rule as ``+``/``-``/``*``/``/``) —
  mismatched non-neutral kinds are ``E-TYPE-040``.

The aggregation law, per fold operator:

.. list-table::
   :header-rows: 1
   :widths: 14 30 56

   * - Fold op
     - Body kind
     - Rule
   * - ``sum``
     - extensive or neutral
     - Legal. Result extensive.
   * - ``sum``
     - intensive
     - **``E-TYPE-041``** — summing an intensive quantity is meaningless.
   * - ``mean``
     - extensive or neutral
     - Legal unweighted. Result carries the body kind.
   * - ``mean``
     - intensive
     - Legal **only** with an explicit ``:weight`` whose expression is
       extensive-kinded. Unweighted is ``E-TYPE-042``; a weight that is not
       extensive is ``E-TYPE-043``. **Result intensive** (D90).
   * - ``min`` / ``max``
     - extensive, intensive or neutral
     - Kind-neutral operation. Result carries the body kind.
   * - ``sum`` / ``mean`` / ``min`` / ``max``
     - enum (``:enum-type``-declared, D101)
     - **``E-TYPE-044``** (#551, discharged by the Territory port train,
       P27) — ``Enum<T>`` has no aggregation kind: ``sum``/``mean`` need
       arithmetic and ``min``/``max`` need ordering, and §2.13/§3.1 give it
       neither. Distinct from the row above: "kind-neutral" there means
       intensive-vs-extensive doesn't matter, never "any type works."
   * - ``count``
     - any, including enum
     - Result ``Int``, extensive. The body is never evaluated, so an
       enum-declared body is legal — inert content, not a content error.

This is the narrow, true form of the law: it rejects the unweighted mean of an
intensive field across classes or space (the recorded variance error), and it
does **not** reject correct weighted code.

**[draft ruling — Phase 1 review, AG verification repair]** *The weighted
intensive* ``mean`` *has a result kind, and it is intensive.* The row above
stated legality and stopped, while the other four rows all state a result —
which left the kind of ``(fold mean … :weight …)`` over an intensive body
undetermined, and undetermined only until such a fold appears in a
kind-checked position, as §2.12's two-hop shape does. Two implementations
free to read the blank differently would disagree on whether that program
loads, which is a III.12(a) failure rather than a style question. The value is
``Σ(w × x) / Σ(w)``, which is in the units of ``x``: a weight-normalised mean
of an intensity is an intensity. Stating it is unit algebra, not new
mathematics, and it is stated **here** because deriving it through the
``*``/``/`` bullet is deliberately unavailable — that bullet rejects
extensive ÷ extensive as ``E-TYPE-040``, which is why the fold operators carry
their result kinds in this table instead of leaving them to decomposition.

*Exemptions.* A field may carry an exemption row in the declared
``EXTENSIVE_INTENSIVE_EXEMPTIONS`` ledger with a mandatory reason string.
Adding a row takes the same sign-off as adding a sentinel exemption. An
exemption suppresses ``E-TYPE-041``/``042``/``043`` **for the named field
only** and is itself content, inside ``rules_hash``.

3.5 Binding resolution
~~~~~~~~~~~~~~~~~~~~~~~~

At load, for every rule and every declared binding:

1. The source must resolve — the field/const/metric must exist in the closed
   vocabulary (``E-LOAD-010`` / ``E-LOAD-011`` / ``E-LOAD-030``).
2. A binding that is **not** ``:optional`` and whose source can be absent for
   any node the rule can be evaluated against is a load error (``E-LOAD-010``).
3. **[draft ruling — Phase 1 review]** ``:optional`` **requires**
   ``:default``. A bare ``:optional`` is ``E-PARSE-031``. The design document
   pairs the two in its single example; requiring the pair removes the need
   for a dominance analysis over ``bound?`` guards, keeps every expression
   total, and means no rule ever observes absence — it observes a declared
   default. There is consequently no ``bound?`` predicate in the language.
4. Every ``:default`` declaration must appear in the migration corpus's
   allowlist (the trap DSL's pinned absent-reads-as-0 sites). A ``:default``
   outside the allowlist is a lint failure requiring Director sign-off — not a
   load error, because the allowlist is program state, not language state.

3.6 Closed vocabulary
~~~~~~~~~~~~~~~~~~~~~~~

Enum types, node types, edge types, **hyperedge types**, event types, field
names, metric names and intrinsic names are all **closed**: a name that is not
in the registry is a load error, never a fallback. Adding a member is
amendment territory, not modding territory (design §5, modding boundary).
Modders author rules and coefficients
over the closed vocabulary; fuel + the closed intrinsic set + no I/O is a
sandbox with no escape to express.

**[draft ruling — Phase 1 review, R9 chapter C3]** *Graph-scope state is
ordinary node state on a declared carrier node type.* Twenty-two of the
thirty-four frozen systems read or write state that belongs to the graph rather
than to any node — the opposition registry, the market-scissors axis, the
electoral registers, the national wealth vector, the imperial-rent pool, the
phase latches — through ``graph.graph[...]``, ``set_graph_attr`` or
``context.persistent_data`` (R9 gap analysis §2, Q6: the single most pervasive
gap in the estate). None of it had a BSL home: ``<bind-src>`` closes at
``:field``/``:const``/``:metric``/``:tick``, ``:metric`` is read-only by
construction, and no §2.8 verb writes anything but a node, an edge, a hyperedge
or an event.

The ruling adds **no new grammar and no new storage class**. A value of
graph scope is declared as an ordinary ``deffield`` owned by a **carrier node
type** — a ``NodeType`` member whose manifest ``:ceiling`` is 1 — read with
``(field-of (the NodeType/…) …)`` and written with ``(update-node (the
NodeType/…) … )``. Everything the rest of this document says about node state
then applies unchanged: the value is hashed as node state, iterated in the
order of §2.6, bounded by the same ceilings, visible to the inspector and to
the write log, and subject to §3.4's kind rule. Adding a carrier ``NodeType``
member costs exactly one closed-vocabulary member and is therefore **amendment
territory** under this section — which is the right weight for a decision this
load-bearing, and is the ruling's price rather than a side effect of it.

*The rejected route, recorded so it is not re-proposed.* The alternative was a
``:global`` bind-src plus an ``update-global`` verb. It was rejected because it
invents a second storage class whose determinism, iteration, hashing, kind and
inspection obligations would every one of them have to be restated — a
document's worth of duplicated law to avoid one enum member — and because
state that is not node state is invisible to the two mechanisms the engine
already built for exactly this scrutiny (the derivation inspector's write log,
and the ceiling-bounded content hash). A closed verb set is a property worth
more than the convenience of writing ``update-global``.

*What the ruling does not do.* It does not make every register a singleton:
per-sovereign and per-county registers are ordinary nodes of ordinary types,
reached by ordinary queries. ``the`` and the carrier discipline are for the
values that are genuinely one-per-graph. And it does not introduce a staging
or double-buffering construct — see §4.7.

3.7 The load-time fuel bound check
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The bound is computed against **declared cardinality ceilings, not the runtime
graph**. Each scenario manifest declares per-``NodeType``, per-``EdgeType`` and
per-``HyperedgeType`` ``:ceiling`` values, themselves inside the content hash.

**[draft ruling — Phase 1 review]** *The member-count axis* (Amendment D). A
hyperedge type has **two** independent cardinalities, so its manifest row
declares two numbers: how many hyperedges of that type may exist
(``:ceiling``) and how many members any one of them may carry
(``:max-members``, §2.9). Without the second number a fold over ``members-of``
would have no static bound at all. This is the ceiling revision the ruling
forces: under the dyadic working assumption an n-member formation was going to
be *some number of edges*, and one edge-type ceiling covered it; under a native
hyperedge the honest cost is ``Σ|members|``, and ``:max-members`` is what makes
that quantity **declarable** at load rather than discovered at hydration.

Define ``cost(n)`` over the AST:

.. code-block:: text

   cost(literal)                = 0
   cost(variable-ref)           = 1
   cost(arith | cmp | bool)     = 1 + Σ cost(children)
   cost(intrinsic call)         = 5 + declared_cost(callee) + Σ cost(args)
   cost(fold)                   = 2 + cost(query)
                                    + ceiling(query) × (cost(body) + cost(weight))

The first five base numbers — literal 0, variable-ref 1,
arithmetic/comparison/boolean 1, intrinsic call 5 + callee, fold
2 + ceiling × body — are copied from the design document's Phase-0 cost model
and are **pinned by conformance vector; revising them is a vector re-bless**
(§6). The remaining rows are this document's completion of that model and are
**[draft ruling — Phase 1 review]**:

.. code-block:: text

   cost(if)                     = 1 + cost(cond) + max(cost(then), cost(else))
   cost(exists | forall)        = 2 + cost(query) + ceiling(query) × cost(body)
   cost(query)                  = 1 + cost(element predicate, if any)
   cost(update-op)              = 1 + cost(operand)      ; add|sub|set|scale
   cost(structural verb)        = 3 + Σ cost(operands)
   cost(members list)           = Σ cost(members)        ; grouping, no base cost
   cost(guard)                  = 1 + cost(cond) + Σ cost(effect-items)
   cost(field path | enum-ref)  = 0                      ; static, like a literal
   cost(field-of)               = 1 + cost(element expr) ; §2.10, R9 C1
   cost(edge-between)           = 1 + Σ cost(operands)   ; §2.10, R9 C2
   cost(the)                    = 1                      ; §2.10, R9 C3
   cost(metric-of)              = 1 + cost(element expr) ; §2.10, R9 C9
                                                         ; provider work is
                                                         ; not rule-metered
   cost(membership-field-of)                             ; §2.10, AG (i)
                                = 1 + Σ cost(element exprs)
   cost(member item)            = Σ cost(children)       ; §2.8, AG (i)
                                                         ; grouping, no base cost
   cost(domain)                 = 0                      ; §2.3, R9 C4
   cost(select-max | select-min)                         ; §2.7, R9 C5
                                = 2 + cost(query)
                                    + ceiling(query) × cost(score)
   cost(for-each)                                        ; §2.8, R9 C6
                                = 2 + cost(query)
                                    + ceiling(query) × Σ cost(effect-items)
   cost(:expr binding)          = cost(expr)             ; §2.5, R9 C7
   cost(:as name)               = 0                      ; §2.6, R9 C8
                                                         ; a reference costs 1
   bound(rule)                  = Σ cost(:expr bindings)
                                    + cost(cond of <when>)
                                    + Σ cost(effect-items)

**[draft ruling — Phase 1 review, R9 chapters C1–C2]** *Accessors are keyed
lookups, not iterations.* Every §2.10 accessor charges a variable-reference
base of 1 plus its operands and is **never multiplied by a ceiling**, because
none of them ranges over a set: ``field-of`` reads one element's one field, and
``edge-between`` resolves one ``(source, target, type)`` key. The static bound
of a rule using them is therefore the same shape as before — the accessors add
constants, and only the iteration constructs (``fold``, ``exists``/``forall``,
and the chapter-C5/C6 forms) carry ceiling factors. That is what keeps the
Power-of-10 Rule 2 claim static as the accessor set grows.

**[draft ruling — Phase 1 review, Amendment AG (i)]** *Attributed membership
adds a lookup, not an axis.* ``membership-field-of`` is a keyed lookup like
the rest and is charged as one; ``update-membership`` charges under the
``cost(structural verb)`` row above, ``3 + Σ cost(operands)``, exactly as the
other three update verbs do, and is given no row of its own because a second
normative statement of one cost is one too many. The **ceiling side is
unchanged**: ``:max-members`` remains the only membership cardinality axis, so
a fold over ``members-of`` whose body reads payload is bounded by the same
declared number as one whose body reads nothing, and the three ceiling axes
below stay three.

**[draft ruling — Phase 1 review, Amendment AG (ii)]** *Lattice declarations
are unmetered.* A ``rung`` or an ``adjunction`` (§2.9, §3.9) is manifest-class
content: no rule AST contains one, so ``bound(rule)`` never sees one and
neither is charged or given a cost row. The folds and ``for-each``\ s that
realise a rung are priced exactly where they already were —
``ceiling(neighbors)``, D52's lesser of the ``:via`` edge type's and the
result node type's ceilings — so declaring a lattice adds no ceiling axis, no
cost row and no term to any bound.

**[draft ruling — Phase 1 review]** *Query operand charging* (implementation-
discovered, 2026-07-30, Phase 1 Task 13). The ``cost(query)`` row names only
the element predicate, but three query heads (``neighbors``, ``members-of``,
``hyperedges-of``) also carry an *operand expression* that §4.5 charges when
it is evaluated; omitting it would make the static bound under-count the
runtime meter — the loud-failure inversion. The bound checker therefore reads
the row as ``1 + Σ cost(children)``: identical for the predicate queries
(enum-refs and direction keywords cost 0), and additionally charging the
operand where one exists.

``ceiling(query)`` is the manifest ceiling of the queried type — which is why
§2.9 makes a queried type carrying no manifest row ``E-LOAD-045`` rather than a
defaulted or skipped bound; for ``neighbors`` it is **[draft ruling — Phase 1
review, revised by R9 chapter C8]** the *lesser* of the queried edge type's
ceiling and the annotated result node type's ceiling — neither bound can be
exceeded, so the smaller is the
honest one, and the annotation C8 makes mandatory is what makes the second
number available. (A per-node degree ceiling would be tighter still and remains
the Phase-1 review item D15 recorded.) For the three hyperedge queries
**[draft ruling — Phase 1 review]**: ``hyperedges`` uses the hyperedge type's
``:ceiling``; ``members-of`` uses that type's ``:max-members``; and
``hyperedges-of`` uses the type's ``:ceiling`` — a per-node *incidence-degree*
ceiling would be tighter there, the exact dual of the ``neighbors`` review item
above, and is deferred with it.

The bound therefore composes over **three** ceiling axes rather than two:
node-type and edge-type cardinality as before, plus per-hyperedge member count
wherever a rule folds over ``members-of``. A fold over members nested inside a
fold over ``hyperedges`` costs ``ceiling(T) × max-members(T) × cost(body)``,
which is exactly ``Σ|members|`` at the declared ceilings — **linear in the
incidence count**, and never the ``C(n,2)`` a clique expansion would have cost.
That is the fuel-side consequence of the ruling: the representation that VIII.9
mandates is also the one whose static bound stays computable at national scale.

``bound(rule) > :fuel`` is ``E-LOAD-040`` — rejected **at content load**, so
the Power-of-10 Rule 2 claim is a static property rather than a dynamic trap.
An ``add-hyperedge`` whose ``<members>`` list is longer than the declared
``:max-members`` is ``E-LOAD-042`` — the list's length is fixed in the source
text, so that check is **static**, not a runtime one. A hydration that
exceeds a declared ceiling — including a hydrated hyperedge carrying more
members than ``:max-members`` — is itself a III.11 load failure
(``E-LOAD-041``). The runtime meter of §4.5 remains as the backstop.

3.8 Deliberate absences and their re-modellings
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Seven things the frozen estate reaches for do not exist in BSL and are not
going to. Each is recorded here **with the re-modelling that replaces it**, so
that a future port reads the absence as a decision rather than an oversight and
does not re-propose the construct. None of these adds grammar; several delete a
question. An eighth item follows them under its own heading: it is *not* a
settled absence, and saying so is the point of recording it here.

**1. Absence, and writing it back** (R9 gap analysis §2, Q17). There is no null
literal, no ``unset`` update-op, and no ``bound?`` predicate — D13 removed the
last of those on purpose. But two systems genuinely need to say "this node has
no value on this axis": one writes ``None`` when a node loses an axis, and one
skips permanently when a stock was never seeded.

``:optional``/``:default`` is **not** the answer, and reading it as the answer
changes behaviour silently: ``:default 0.0`` converts *never seeded* into
*seeded with zero stock*, which is a different eligibility population and a
different game. The landing is a **companion presence field**: a
``deffield``-declared ``Bool`` alongside each genuinely optional axis, written
by the same effects list that writes the value, under one ``guard`` so the pair
moves together or not at all. Absence is then representable, hashable and
inspectable — and "unset the axis" is an ordinary ``(set #f)`` on the presence
field rather than a verb the language does not have. The EpistemicHorizon
discipline (all three attributes or none) is already exactly this shape.

**2. No sequence or map type** (Q13). Three systems keep ordered or list-valued
state: a FIFO agenda, an order-significant acquisition list, a set of
suppressed field names. A sequence type would drag ordering into CAS, into the
kind rule and into the fuel model, and a map type would need a key ordering
this document has deliberately confined to graph-element ids. Each case
re-models with what §2 already has:

- a FIFO agenda becomes its own bounded ``NodeType`` carrying a
  ``queued-at-tick`` field, and "the next item" becomes ``select-min`` on that
  field (§2.7);
- an order-significant acquisition list becomes an edge type carrying
  ``acquired-at-tick``, and "the most recent" becomes ``select-max``;
- a set of suppressed field names becomes one ``Bool`` field per suppressible
  field.

All three land inside the closed vocabulary, inside the content hash, and
inside the ceiling-bounded fuel model, which a list type would have left.

**3. No same-tick event-history query** (Q16). ``emit`` is write-only and
§2.6's query heads do not include an emission log; two systems ask "was a
crisis emitted this tick?". The re-modelling is better than the feature: the
**emitting** rule also stamps a field (a ``…-crisis-tick`` on a carrier or
subject node), and the consuming rule reads it as an ordinary ``:field``. That
makes the cross-system dependency visible in content, hashable, and
inspectable — three properties an event-log query would not have, since a query
over emissions would make the dependency invisible in every artifact except a
runtime trace. It also means a crisis-gated system must be ported **together
with its producer**, or the consuming rule reads a field nothing writes.

**4. No string payloads on** ``emit`` (R9 gap analysis §3, B3). ``Str`` has no
operations (§3.1) and ``<expr>`` has no string literal — one in a payload is
``E-PARSE-010``, an atom rejected by position (§1.6) — so every
``<payload-item>`` expression is a number, a bool or an enum-ref. Transcribed
systems carrying ``predicate`` or ``description`` strings, or a field name as a
string, convert them to enum-refs or drop them — the **rule id already
identifies the rule**, and an event whose payload restates its own provenance
in prose is carrying a log line, not state.

**5. No ledger or receipt binding** (B6). §2.8 prohibits I/O outright, and
three surveys independently reached for a "ledger-write binding" anyway. There
is nothing to add to BSL: a receipt is a **kernel observation of an effect that
already happened**. The rule emits or writes; the kernel records. Making the
rule write the receipt would put the same fact in the content hash twice and
give a content author a way to record an effect that did not occur.

ADR224 makes that observation executable. After each successful rule, the
kernel reduces the actual collection-boundary events followed by the actual
apply-boundary writes into one continuous ``AuditReceipt`` sequence. Each row
carries the rule id, ``RuleRole``, ``EvidenceClass``, a ``u32`` ordinal, and a
canonical identity-free effect signature. It carries no graph-element id and no
written value. The receipts publish only when the detached whole tick succeeds;
a later failure publishes none. This is Gate 2 in-memory audit evidence, not
Gate 3's durable envelope, dirty-subject receipt, or Archive outbox.

**6. No cascade semantics in the verb table** (B7). What ``remove-node`` does
to incident edges and memberships is an **engine-level observable**, specified
outside this document (ADR185 R2: incident edges removed, memberships dropped,
one write-log record per cascaded item). §2.8's verb table deliberately does
not restate it, because two normative statements of one behaviour is one too
many.

**7. No bounded numeric iteration** (Q15). One system runs a five-iteration
clamp-then-renormalise with an early-exit convergence check. A loop construct —
even a bounded one — would break the *syntactic* totality argument §2.7 rests
on, which is the property that makes the Power-of-10 Rule 2 claim static rather
than analysed. With ``:expr`` (§2.5) the five iterations unroll into five named
bindings, which is verbose and honest; failing that it is a legitimate Rust
domain-crate binding. Declaring a bespoke ``renormalize`` intrinsic would be
the worst of the three: it hides a mechanism inside the kernel, where neither
the content diff nor the inspector can see it, for a single call site.

**8. No edge-endpoint accessors — an open item, not a settled absence**
(R9 gap analysis §2, Q2; recorded on verification, 2026-08-10). No form yields
an ``EdgeRef``'s source or target node. Twelve systems reach for one, the R9
chapter plan (report §7) assigned Q2 to no chapter, and it is written down here
because the alternative is that it reads as an oversight — which, unlike the
seven above, is closer to the truth: this is the one item of the gap analysis
this revision neither closes nor deliberately refuses.

What is expressible today reaches only the case where one endpoint is already
in hand. A rule holding ``self`` walks to its counterparties and resolves each
edge by key:

.. code-block:: scheme

   (fold sum (neighbors self EdgeType/SOLIDARITY :in NodeType/SOCIAL_CLASS)
         (field-of (edge-between EdgeType/SOLIDARITY it self)
                   solidarity/strength))

That is correct and priced — §3.7 charges the extra keyed lookup per
neighbour — and it does **not** reach the general case: an ``EdgeRef`` taken
from a fold or a ``for-each`` over ``edges`` has *both* endpoints unknown,
which is what most of the twelve call sites need. Two landings are available,
and choosing between them is a chapter rather than a sentence: a
``source-of``/``target-of`` pair in §2.10 — reads of the triple §2.6 already
treats as the edge's key, minting no new kind of object — or leaving those
systems on the ``self``-anchored idiom and accepting that edge-iterating rules
cannot name their endpoints. Until it is chosen, a system that iterates edges
it did not start from is **not authorable in BSL**, and that is a port blocker
(D78) — the standing D66's had until Amendment AG discharged it, and now the
only one this document carries. Amendment AG does not reach this item, and
nothing in the sections it adds should be read as ruling it.

3.9 Invariant substrate, hydrated data, and the scale lattice
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Five systems aggregate up a scale lattice — county to commuting zone to state
to nation, hex link to county pair to national, per-county sums then a reverse
join — and one asks whether the language needs a grouping operator. It does
not, and this section says why, what it needs instead, and where the boundary
of this document's authority to say so lies.

**No** ``group-by``, **and no keyed collection.** A fold that grouped elements
by a runtime attribute value would have to return a *map*, which §3.1
deliberately lacks and §3.8 declines to add: a map drags a key ordering into
CAS, into the kind rule and into the fuel model. Everything the lattice needs
is instead expressible as **graph content**, because a scale level is a thing
in the world and membership in it is a relation:

.. code-block:: scheme

   (fold sum (neighbors self EdgeType/IN_SCALE :in NodeType/TERRITORY)
         (field-of it territory/wage-bill))

Aggregation up one rung is then an ordinary one-hop fold, and distribution down
one rung is an ordinary ``for-each`` over the same query. This costs
closed-vocabulary members and a hydration contract; it costs no grammar.

.. note::

   **The boundary of this section, as Amendment AG (ii) redrew it.** Declaring
   a scale-lattice **rung**, and declaring ``allocate``/``aggregate``
   **instances** of the existing adjunction schema, are content acts — the
   forms are §2.9's and their validation is below. What stays **closed** under
   Amendment AE (ii) is everything that would change the schema rather than
   instantiate it: minting a new adjunction **kind**, altering the
   **conservation obligation** between rungs, and any new level-lattice
   **algebra**. A content set reaching for one of those is proposing
   mathematics, not writing a scenario, and it costs an amendment as it always
   did.

**[draft ruling — Phase 1 review, R9 chapter C11]** *Invariant substrate is
declared, and structural verbs cannot touch it.* The spatial substrate is
immutable (Constitution: political claims are overlays), and the Director's
2026-07-30 spatial-adjacency ruling puts the invariant relations in static
lookup tables rather than per-tick state. A ``manifest`` ``ceiling`` row may
therefore carry the flag ``:invariant``:

.. code-block:: text

   <ceiling>  ::= "(" "ceiling" <enum-ref> ":ceiling" <int-lit>
                      ( ":max-members" <int-lit> )? ":invariant"? ")"

``:invariant`` is legal on a ``NodeType`` or ``EdgeType`` row and illegal on a
``HyperedgeType`` row (``E-LOAD-042``, with the ``:max-members`` mismatches it
already covers). Its meaning is narrow and static: **an** ``add-node``,
``remove-node``, ``add-edge`` **or** ``remove-edge`` **naming an invariant type
is** ``E-LOAD-013``, checked at load off the verb's ``<enum-ref>`` operand.
Field writes are unaffected — a territory's stock changes every tick while the
territory's *existence* and its rung in the lattice do not, and it is exactly
that distinction the flag encodes. The check is worth having in the language
rather than in a sentinel because it is decidable from the content set alone
and because "a rule rewired the substrate" is the failure it prevents.

**[draft ruling — Phase 1 review, R9 chapter C11]** *The hydration contract,
and why there is no* ``:reference`` *bind-src.* Five systems read external
keyed reference data — roughly fourteen ``(fips, year)``-keyed series, a
tensor registry, a county crosswalk — through host calls, and a naive port
would invent a bind-src to match. It must not. ADR174 already draws the
boundary: data sources are Python-glue concerns, and values enter as declared
BSL bindings. The consistent landing is that the **data-build pipeline
materialises keyed series as declared node fields at hydration**, so a rule
reads them with an ordinary ``:field``. That keeps §2.8's no-I/O prohibition
intact, keeps the values inside the content hash rather than beside it, and
needs no language change at all.

What hydration may do, stated once because three chapters now depend on it:

1. It creates elements of declared types and writes declared fields, and
   nothing else. An undeclared field or type at hydration is the same
   ``E-LOAD-0xx`` class as it would be in content — hydration is not a
   back door into the closed vocabulary (§3.6).
2. It is bounded by the declared ceilings, including ``:max-members``; an
   over-ceiling hydration is ``E-LOAD-041`` (§3.7), unchanged.
3. It is the **only** writer of ``:invariant`` structure, which is what makes
   the ``E-LOAD-013`` check above meaningful rather than merely restrictive.
4. A field a rule declares a plain ``:field`` binding against must be seeded by
   hydration or the rule does not load (``E-LOAD-010``, §3.5). This is the
   clause that makes "the reference-series hydration contract" a **blocking
   dependency** for the systems that read those series, rather than a source of
   zeros at tick 1.
5. **[draft ruling — Phase 1 review, R9 verification repair]** It may not seed
   two dyadic edges sharing one ``(source-id, target-id, edge-type)`` triple; a
   scenario that does is ``E-LOAD-044``. §2.8 already refuses the duplicate at
   the verb (``E-EVAL-031``), and hydration is the only other way an edge
   enters the graph, so this is the clause that makes the triple a **key**
   rather than a sort field. Without it §2.6's edge iteration order is not a
   total order — a duplicated triple has no defined position relative to its
   twin — and §2.10's ``edge-between`` has no rule for resolving *two*, having
   one only for none (``E-EVAL-034``). Node ids and hyperedge ids need no such
   clause: they are identities, and seeding one twice is not expressible.
6. **[draft ruling — Phase 1 review, Amendment AG (i)]** A hydrated membership
   carries its **complete** declared payload. A scenario seeding a hyperedge
   whose members' types declare payload fields (§2.9) writes every one of them
   for every member, and an omission is ``E-LOAD-047`` — the same code and the
   same obligation ``add-hyperedge`` carries at mint (§2.8). Hydration and the
   verb are the only two ways a membership enters the graph, so a partly
   attributed membership has to be unrepresentable from both or from neither;
   this clause is the second half of that, and it is what lets §2.10 say that
   an existing membership never reads absent.
7. **[draft ruling — Train B item 3, issue #591 — D156]** It seeds an edge
   attribute only through the scenario dialect's
   ``(edge-attr <EdgeType/MEMBER> <from-name> <to-name> <field-qname> <value>)``
   form, and only under four conditions, each a loud refusal: both endpoints
   and the ``(source, target, edge-type)`` triple itself were seeded ABOVE the
   form (the top-to-bottom law — an attribute write never mints an edge,
   exactly as ``update_edge`` never mints an attribute row for one); the
   qname's owner segment names the edge's own type (§2.10 discipline 1,
   checked at hydration exactly as ``check_edge_referent_type`` checks it at
   evaluation); the qname does not end in ``/strength`` (D32's implicit field
   seeds via the ``edge`` form's own 4th slot only — one datum, one writer;
   the guard is unconditional because ``load_deffield`` accepts an explicit
   ``(deffield <edge-type>/strength …)`` that only ``prepare_rules``'s
   ``E-LOAD-001`` refuses later, and without the guard the write would fall
   into the substrate's ``/strength`` fork (D143) and silently rewrite the
   mint strength slot); and the field was declared by ``deffield``, the value
   converting through the same per-type literal law a node attribute does
   (``Currency`` refused identically). A scenario seeding one
   ``(edge-type, source, target, field)`` quadruple twice is ``E-LOAD-057`` —
   clause 5's key argument one axis wider: the file would carry two values
   for one datum with only the later surviving.

**[draft ruling — Phase 1 review, Amendment AG (ii)]** *Rungs and adjunction
instances are* ``manifest`` *children, not a top-level form of their own.* Two
reasons, the first D79's verbatim: §5.5 hashes ``deffield``, ``intrinsic``,
``manifest`` and ``metric`` into sibling digests that ``ContentDigest``
combines, and ``ContentDigest``'s composition is
:doc:`/reference/determinism-contract`'s — a new top-form would owe a new
sibling digest and an edit to a document this one does not reach into. The
second is locality, and it is worth stating exactly because the two standing
rulings do not check against the same thing. The substrate ruling is checked
against ``:invariant`` ``ceiling`` rows — children of the very form a rung
joins — so putting the rung among them keeps that check inside one object. The
weighting ruling is checked against ``deffield`` kinds, so it spans the
manifest and field digests exactly as any content-wide check does; what the
manifest landing buys there is not one digest but **no new one**, which is
reason 1 again. Both checks read *declared content* only, which is the
property that matters for III.12(a).

.. code-block:: scheme

   ; illustrative; the delineation counts are a scenario's, not this document's
   (manifest usa-2026
     (ceiling NodeType/TERRITORY       :ceiling 3143 :invariant)
     (ceiling NodeType/COMMUTING_ZONE  :ceiling 741  :invariant)
     (ceiling EdgeType/IN_SCALE        :ceiling 3143 :invariant)
     (rung county-cz NodeType/TERRITORY NodeType/COMMUTING_ZONE
           :via EdgeType/IN_SCALE :substrate)
     (adjunction wage-bill
                 territory/wage-bill commuting-zone/wage-bill
                 :rung county-cz)
     (adjunction unemployment
                 territory/unemployment-rate commuting-zone/unemployment-rate
                 :rung county-cz :weighted-by territory/labor-force))

**What each form declares.**

- A ``rung`` names **one step** of a lattice. Its two positional
  ``<enum-ref>``\ s are ``NodeType`` members, **finer first and coarser
  second**, and ``:via`` is the ``EdgeType`` member carrying the relation,
  directed **finer → coarser** (a county's edge points at its commuting zone).
  That one convention serves both forms and both directions of travel, so the
  ``neighbors`` reads at the top of this section are unambiguous without a
  second annotation: aggregation *from* the coarser element is
  ``(neighbors self <via> :in <finer>)``, and distribution is the same query
  under ``for-each``.
- An ``adjunction`` names **one instance** of the ``allocate`` ⊣ ``aggregate``
  schema along a declared rung. Its two positional ``<qname>``\ s are the
  fields at the rung's finer and coarser ends **in the rung's own order**,
  ``:rung`` cites the rung by name, and ``:weighted-by`` names the extensive
  field the intensive case is weighted by.
- Names are content-set-unique (``E-LOAD-001``, §2.2), and every type a rung
  names owes a manifest ``ceiling`` row like any other type the content set
  uses (``E-LOAD-045``). An ``<enum-ref>`` of the wrong kind at any of the
  three positions is ``E-TYPE-011`` under §2.6's class rule.

**[draft ruling — Phase 1 review, Amendment AG (ii)]** *The first binding
ruling: a substrate rung resolves through the static lookup estate.* The
Director's 2026-07-30 spatial-adjacency ruling puts the invariant relations in
static per-resolution lookup tables and never in per-tick state, and Amendment
AG (ii) binds it to every declared rung. ``:substrate`` is how a rung says it
is one of those, and the flag obliges all three of its types — finer, coarser
and ``:via`` — to carry ``:invariant`` manifest rows. A ``:substrate`` rung any
of whose three types does not is ``E-LOAD-049``.

With the flag in place the ruling *is* the language's, by composition and not
by promise: ``:invariant`` makes ``add-node``/``remove-node``/``add-edge``/
``remove-edge`` naming those types ``E-LOAD-013`` (D63), and clause 3 of the
hydration contract above makes hydration their **only** writer. A substrate
rung is therefore built once, from the lookup estate the data-build pipeline
materialises, and no rule can rewire it mid-run. The flag is declared rather
than inferred from the three ``:invariant`` rows for the same reason
``:invariant`` is itself declared: an inferred classification would silently
reclassify a rung the day someone marked its edge type invariant for an
unrelated reason. A rung *without* the flag is perfectly legal — the social
lattice's rungs are not substrate — and carries no invariance obligation.

**[draft ruling — Phase 1 review, Amendment AG (ii)]** *The second binding
ruling: an intensive adjunction is weighted or it does not load.* An
``adjunction`` whose two fields are declared ``:kind intensive`` and which
carries no ``:weighted-by`` is ``E-LOAD-050`` — the load error the amendment
names in so many words, and the recorded variance error (an unweighted mean of
an intensive quantity across classes or space) caught one level earlier than
§3.4 catches it. The weight field must be ``:kind extensive``
(``E-TYPE-043``, the code §3.4 already uses for a non-extensive weight).

The two checks are not redundant. §3.4's ``E-TYPE-042`` catches the **fold
that implements** an aggregation; ``E-LOAD-050`` catches the **lattice that
specifies** one, which exists before any rule is written and outlives any
particular transcription of it. A scenario that declares an unweighted
intensive rung has made the error whether or not a rule pack has yet realised
it, and the amendment puts the obligation on the declaration.

**Fit to the schema, in one code.** ``E-LOAD-051`` is "this instance does not
fit the schema at its declared rung": a ``:rung`` naming no declared rung; a
positional ``<qname>`` whose owning type is not the rung's node type at that
end; the two fields disagreeing in ``:kind``; a ``:weighted-by`` on an
extensive pair (there is nothing to weight) or one whose field is not owned by
the rung's finer type. One code because they are one fact — the declaration
and the rung do not agree — and minting five synonyms is the hygiene defect
D75 ruled against.

**The conservation obligation is inherited, and there is nowhere to restate
it.** The grammar gives an ``adjunction`` no clause, no keyword and no
expression in which to state a conservation law: an instance names its rung
and its fields and stops. That is how "instances, not kinds" is enforced *by
construction* rather than by review — the only conservation obligation
available to a declared pair is the schema's, and altering it stays AE (ii)
territory exactly as the note above says.

**What a declaration does not do.** It declares; it does not execute. There is
no new verb, no evaluator behaviour and no runtime object: aggregation up a
rung is still the ordinary one-hop fold at the top of this section and
distribution down it still an ordinary ``for-each``, and this document does
not — cannot — decide which fold in a rule pack *is* a declared adjunction.
What the declarations buy is that the lattice becomes checkable at load, that
the two standing rulings become load errors instead of review notes, and that
a reviewer has a named obligation to read a rule pack against.

**Determinism.** Rungs and adjunctions are validated, and their failures
reported, in **ascending declared-name byte order** — the key §4.2 already
uses for rule ids — so two conforming implementations reject a malformed
lattice with the same first error. Nothing about them is evaluated during a
tick, so none of it reaches a tick hash; the ``manifest`` bytes that carry them
reach ``ContentDigest`` exactly as the ceiling rows do (§5.5).

3.10 The intrinsic cap and the rider slate
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

§2.7 fixes the *calling convention* for intrinsics and says their contents are
Phase-2 work. This section fixes what that leaves dangerous: how many
intrinsics there may be, which authority says so, and why passing that test is
not the same as being allowed.

**The cap, and its authority chain, recorded exactly.** The Program 28
roadmap's R10 row holds the intrinsic set at ``{exp, log}`` **at most**, citing
ADR176 r21. r21's own text pins a *mechanism* — transcendentals cross via a
pinned soft-float libm crate with golden vectors per intrinsic — and does not
enumerate a membership; the ``{exp, log}`` enumeration is the roadmap's
rendering of it. **R10 is operative** for R9 and R10 purposes and this document
is written to it. The discrepancy is recorded rather than resolved, because
resolving it is the Director's.

Concretely, as of this revision, **nine names are declarable**.

R10 caps the transcendental pair ``exp`` and ``log``. ADR188 Row 2 separately
authorizes the ``floor`` intrinsic. That authority leaves R10's
``{exp, log}`` enumeration unchanged.

The **ADR219 exact-arithmetic sextet** (Director ruling 2026-08-22) completes
the set. ``sqrt`` takes Row 6's fallback rider. ``round-half-even`` lands Row
3's ratified housekeeping rider and resolves D70's "recorded, not resolved."
``min``/``max``/``abs``/``clamp`` supersede Rows 4/5's "no rider"
dispositions on #591 item 2's accumulated port evidence. Like ``floor``, all
six cross via IEEE-754 exactly-specified operations. They use neither pinned
soft-float libm nor a §4.3 golden-vector family. The normative paragraphs below
record each disposition.

The transcendental roster stays the R10 pair alone. Amendment AJ removes the
former ``rng-draw`` declaration. The historical carrier-key record below
grants no current authoring authority.

**Cap-legality is not doctrine-legality, and this is the load-bearing
sentence.** ``exp`` sits inside the cap. Three of the five ``exp`` call sites
in the frozen estate stipulate a logistic sigmoid that ADR173 and the standing
2026-07-29 no-imposed-functional-forms ruling retire: ``P(S|A)``, whose S-curve
must **emerge** from within-class wealth dispersion rather than be asserted; a
defection probability; a wage-pressure sigmoid. A verbatim transcription of
those formulas would pass the cap check and violate the theory line. There are
therefore **two gates**, and only one of them is mechanical:

1. *Is the intrinsic declarable?* Mechanical, checked at load against the set
   above (``E-LOAD-021`` for an undeclared call).
2. *Does this use stipulate a functional form?* Not mechanical, and not
   checkable by any typechecker. It belongs to Director review, and the
   question it asks is always the same: **can this be re-derived as a measure
   instead?**

**[draft ruling — Phase 1 review, R9 chapter C13]** ``sigmoid`` *is a reserved
prohibited intrinsic name.* Declaring an intrinsic named ``sigmoid`` is
``E-LOAD-024`` — the same code §2.7 uses for a reserved form-head collision,
and here for a stronger reason: declaring ``sigmoid`` would hand content the
exact mechanism ADR172 ruling 5 forbids, pre-packaged and named. It is the one
part of gate 2 that *can* be made mechanical, so it is.

**The rider slate.** The table below records the R9 gap analysis §4 proposals
**as proposals**. It is **not normative and declares nothing** on its own; every
row was a question for the Director, and the "Proposal" column is the
analysis's recommendation, not by itself a ruling. **ADR188 (2026-08-10)
disposed all twelve rows** — the Director's "i approve all", transcribed row by
row. Row 2 (``floor``/``trunc``) is **RATIFIED and landed**: see the normative
paragraph and Draft-Ruling Register D97 immediately after this table, and
``declarations::DECLARABLE_INTRINSICS`` in the Rust reference implementation.
**ADR219 (2026-08-22) then landed rows 3-6**: Row 3 (``round-half-even``) is
landed (D70 resolved); Rows 4/5 (``abs``, ``min``/``max`` — and ``clamp``
with them) are superseded by riders on #591 item 2's accumulated evidence;
Row 6's (``sqrt``) fallback rider is taken. The remaining rows' dispositions
are recorded in ADR188 itself; landing each one's own normative text (a table
row here, a §6.2 vector family, a canonical-name elimination) is separate
work, so the table below is left exactly as the R9 gap analysis wrote it — a
non-normative proposal record — except where a mark records Rows 2, 3, 4, 5
and 6.

.. list-table::
   :header-rows: 1
   :widths: 4 20 10 66

   * - #
     - Candidate
     - In cap?
     - Proposal (non-normative)
   * - 1
     - ``mod``, ``floor-div``
     - No
     - **No rider.** Superseded by the calendar bindings of §2.5 — a seam, not
       mathematics.
   * - 2
     - ``floor`` / ``trunc``
     - No
     - **RIDER RATIFIED AND LANDED** (ADR188 Row 2; D97, below). §3.1 declares
       no coercions and ``Int`` promotes to ``Real`` one way only, so there was
       no demotion path at all before this row. What landed is ``floor``
       alone, not a second ``trunc`` intrinsic — see D97 for why one name
       covers the ratified domain.
   * - 3
     - ``round-half-even``
     - No
     - **RATIFIED AND LANDED** (ADR188 Row 3; landed by ADR219, 2026-08-22 —
       D70 resolved). §3.2 and §2.7 already oblige the kernel to expose it to
       rules; the enumeration omitted it. Affirming it was not a widening;
       landing it was the owed work.
   * - 4
     - ``abs``
     - No
     - **SUPERSEDED — RIDER RATIFIED AND LANDED** (ADR219, 2026-08-22, on
       #591 item 2's accumulated evidence: every pack reinventing ``abs`` as
       nested ``if``). The original analysis stood on expressibility —
       ``(if (>= a b) (- a b) (- b a))`` — which was never the point the
       port record pressed: legibility, and the end of per-pack reinvention.
   * - 5
     - scalar ``min`` / ``max``
     - No
     - **SUPERSEDED — RIDERS RATIFIED AND LANDED** (ADR219, 2026-08-22:
       ``min``/``max``, and ``clamp`` with them as the legible saturation
       this row's own reasoning pointed at). Original: **No rider.** Nested
       ``if`` expresses them, and §3.3 already frames silent clamping as
       forbidden quiet degradation — an explicit ``if`` makes the saturation
       legible. The ergonomic cost is real and recorded — and on the
       accumulated evidence, no longer accepted.
   * - 6
     - ``sqrt``
     - No
     - **FALLBACK TAKEN — RIDER RATIFIED AND LANDED** (ADR219, 2026-08-22).
       Original: **Both presented.** Preferred: re-derive platform fit as a
       measure (the share of a class's interest dimensions a platform
       satisfies), which needs no norm — that branch still governs the
       platform-fit CALL SITE. Fallback: a rider — taken by the Director's
       ruling, governing the LANGUAGE. A silent switch to squared magnitudes
       changes the metric's scale and must never happen by default.
   * - 7
     - ``exp``
     - **Yes**
     - **No rider needed; a theory ruling is.** Three of five sites are
       imposed sigmoids under ADR173. Ask the Director to dispose each:
       re-derive as a measure, or except it explicitly as a bounded auxiliary.
   * - 8
     - ``tanh``
     - No
     - **Elimination presented first.** Squashing a log-ratio into ``[-1,1]``
       is a stipulated bounded form; re-derive the scissors balance as a
       measure. Rider only if the Director keeps the squash.
   * - 9
     - ``sigmoid``
     - No
     - **Never declarable** — ruled above, ``E-LOAD-024``.
   * - 10
     - ``entropy``
     - No
     - **No proposal.** Nothing in the thirty-four systems asks for it.
   * - 11
     - RNG draw
     - n/a
     - **Not a rider.** §2.8 already sanctions it as a kernel intrinsic; the
       key convention is below.
   * - 12
     - bespoke ``renormalize``
     - No
     - **Recommend against**, per §3.8 item 7: it hides a mechanism in the
       kernel for one call site.

**The** ``floor`` **intrinsic (ADR188 Row 2, ratified 2026-08-10).** Unlike
``exp``/``log``, ``floor`` is not a transcendental: it crosses via
``roundToIntegralTowardNegative``, exactly specified by IEEE-754 itself —
**not** by §4.3, whose closed basic-op list is addition, subtraction,
multiplication, division and comparison only, and whose golden-vector clause
is for the transcendental pair. This is the specific point at which ADR188's
consequences note ("the two ratified riders get normative intrinsic-table
rows + §6.2 vector families + libm golden vectors at implementation")
**does not apply to this rider**: there is no libm crossing here to pin a
golden vector against, so that half of the consequence is declined rather
than owed, and this sentence is that disposition made explicit rather than
a silent omission. It joins the declarable set (§3.10's opening paragraph)
under this rider's own authority, not R10's, and is wired into
``babylon_tick::run_once`` — the seam the CLI driver and ``babylon-client``'s
engine link both call — via ``intrinsic_host::KernelIntrinsicHost``, not
merely exercised by a unit test.

- **Signature.** One binary64-lane argument (the result of ordinary
  arithmetic, whose static type is ``Real`` — §3.3), returning ``int``.
  ``real`` is admitted as a ``<type-name>`` **only** in an
  ``<intrinsic-decl>``'s ``:params``/``:returns`` position (Draft-Ruling
  Register D98) — a ``deffield``'s or a ``metric``'s ``:type`` still cannot
  name it, since §3.1 rules ``Real`` "Not storable" for a field. A worked
  declaration: ``(intrinsic floor :params (real) :returns int :cost N)``.
  **Train B note (2026-08-15, D155):** the "still cannot name it" half of
  this bullet is superseded — ``real`` became a storable ``:type``
  spelling via D155 (§3.1's eighth row, the register below). D98's
  intrinsic-position law itself is unchanged: ``:params``/``:returns``
  still admit ``real``, and this signature stands as written.
- **A non-``Real``-lane argument is refused, not coerced.** §3.3 promotes
  ``Int`` to ``Real`` only *within* a binary64 expression; it does not reach
  the intrinsic-call boundary, and no full static typechecker exists yet to
  compare a call's argument types against its declaration (§2.7: content is
  Program 27 Phase 2 work). A bare ``(floor 5)`` — an ``Int`` literal, not
  the result of arithmetic — is therefore refused as a malformed call, with
  no numbered code (the reference names none for this class, matching the
  no-invented-codes precedent), consistent with the no-coercions rule
  (§3.1). Every ratified call site passes the *result* of
  ``population × rate``, which is already ``Real`` by the time it reaches
  ``floor``.
- **Domain: ``[0, ∞)``.** The rider's own candidate name pairs ``floor``
  with ``trunc`` because the two ratified call sites demote extensive
  people counts, never wealth: the frozen estate's ``vitality.py``
  (``_calculate_deaths``: ``population`` guarded ``> 0``, ``attrition_rate``
  clamped to ``[0, 1]`` before ``deaths = int(population * attrition_rate)``)
  and ``decomposition.py`` (``la_population <= 0`` returns early;
  ``enforcer_fraction``/``proletariat_fraction`` are pydantic-constrained
  non-negative fractions before ``enforcer_pop_gain``/``proletariat_pop`` —
  the WEALTH lines at the same call site, ``enforcer_wealth_gain`` and
  ``proletariat_wealth``, are **not** ``int()``-demoted). Not a claim of
  §3.4 — the intensivity kind rule states nothing about sign. On this
  domain ``floor`` and ``trunc`` are the **same** function. This document
  declares one intrinsic, named ``floor``, and does not extend it to
  negative reals, where the two conventions diverge and ADR188 Row 2 names
  neither.
- **Errors, all** ``E-EVAL-039``\ **, never a silent pick.** A negative
  argument, a non-finite argument (``NaN``, ``±inf`` — already
  unrepresentable at any observable point per §4.3, so this is defense in
  depth), or a result that does not fit ``Int``'s ``i64`` domain (§3.1) is a
  loud evaluation failure. III.11 and this document's own precedent
  (§3.3's store-boundary range check, never a clamp) both forbid resolving
  any of the three silently — by rounding negative inputs one way or the
  other, or by wrapping/saturating an out-of-range result.

**[draft ruling — Phase 1 review]** The name and domain choice above is this
revision's reading of a rider that ratifies *that* a Real→Int demotion enters
the intrinsic table without specifying *which* of ``floor``/``trunc`` or how
it treats negative reals (ADR188 Row 2's text pairs the two names and is
silent on the negative domain). Recorded as D97; see the Draft-Ruling
Register. The choice is conservative by construction — it commits to nothing
ADR188 did not already imply for the ratified (non-negative) domain — but it
is a workforce reading, not itself a Director ruling, and is open to
correction the same way every other Phase-1-review row in this register is.
The ``real`` type-name-position widening this signature needed is a
separate reading, recorded as D98.

**The** ``sqrt`` **intrinsic (ADR219, ratified 2026-08-22).** Like
``floor`` and unlike ``exp``/``log``, ``sqrt`` is not a transcendental: it
crosses via ``f64::sqrt`` — IEEE-754's own ``squareRoot``, **correctly
rounded by the standard's own mandate** — so there is no libm crossing to
pin a golden vector against, and §4.3's golden-vector clause is DECLINED
for this rider, exactly as ADR188 Row 2's landing declined it; this
sentence is that disposition made explicit rather than a silent omission.
Signature: one binary64-lane argument returning ``real`` —
``(intrinsic sqrt :params (real) :returns real :cost N)``. **Domain:
``[0, ∞)``.** A negative argument is ``E-EVAL-044``
(``IntrinsicOutOfDomain``; the §4.6 register), never a silent ``NaN``;
``-0.0`` is in-domain (``-0.0 < 0.0`` is ``false``, the D97 precedent),
and IEEE's ``squareRoot(-0)`` is ``-0``, so the zero's sign is pinned by
the standard. A non-finite argument is refused at the same input guard —
load-bearing, not redundant: ``sqrt(+inf)`` through the crossing is
``+inf`` (which would fail with the wrong code), and ``sqrt(NaN)`` would
silently propagate. **No result-side re-check exists**: ``sqrt`` of a
finite in-domain argument is finite and exactly specified, so a result
guard would be dead code — unlike ``exp``'s overflow lane
(``E-EVAL-014``). The no-coercions boundary is ``floor``'s: a bare
``(sqrt 4)`` — an ``Int`` literal — is a malformed call, refused, never
promoted.

**The** ``round-half-even`` **intrinsic (ADR219, landing ADR188 Row 3).**
§3.2's "Half-even, defined" paragraph closes with the obligation this
lands: "the same algorithm is what the ``round-half-even`` intrinsic
exposes to rules." The crossing is ``f64::round_ties_even`` — IEEE-754's
``roundTiesToEven``, exactly specified — so the libm/golden-vector
disposition is ``sqrt``'s. **The signature reading (register D227, open
to Director correction):** ``(round-half-even x) : real → real`` — the
nearest integer VALUE to ``x`` within the binary64 lane, ties to the even
neighbor (``2.5 → 2``, ``3.5 → 4``, ``-2.5 → -2``, ``-0.5 → -0.0``; the
sign is the standard's). §3.2's exact-integer-arithmetic law is satisfied
by construction: a binary64 argument is already an exact rational, so
"exactly midway" is decidable exactly and no rational-to-binary64
conversion ever occurs. The Real→Int demotion remains ``floor``'s alone
(ADR188 Row 2); a granularity-general ``(round-half-even x g)`` form is
declined as speculative until a call site needs one. **Domain: total over
finite reals** — at magnitudes ≥ 2^52 every binary64 is already integral,
so the intrinsic is the identity there. A non-finite argument is
``E-EVAL-044``; there is no result-side check (dead code — ``sqrt``'s
disposition).

**The** ``min``/``max`` **intrinsics (ADR219, superseding ADR188 Rows
4/5).** Each ``(real real) → real``. **The comparison rule is pinned, and
the library functions are deliberately not used:** Rust's
``f64::min``/``f64::max`` are licensed to return *either* zero on a
±0.0 tie and to propagate ``NaN`` — implementation-defined answers of
exactly the kind this document exists to forbid. The landed rule is built
from IEEE comparisons alone, which are exactly specified: ``min(a, b)``
is ``b`` when ``b < a``, else ``a``; ``max(a, b)`` is ``b`` when ``a <
b``, else ``a``. On an equal comparison — ``+0.0`` vs ``-0.0`` included,
the two compare equal — **the first argument wins**, bit-pinned by the
conformance tests (D228). A non-finite argument on either side is
``E-EVAL-044`` — refused at the input, never silently propagated nor
silently dropped.

**The** ``abs`` **intrinsic (ADR219, superseding ADR188 Row 4).**
``(real) → real`` via ``f64::abs`` — IEEE-754's sign-bit ``abs``, exact,
with nothing platform-variant to pin (``sqrt``'s disposition).
``abs(-0.0)`` is ``+0.0`` — the canonical zero, pinned to the bit. A
non-finite argument is ``E-EVAL-044`` (``abs(±inf)`` would be ``+inf``
and ``abs(NaN)`` would be ``NaN`` — neither escapes).

**The** ``clamp`` **intrinsic (ADR219).** ``(clamp x lo hi) : (real real
real) → real`` — the LEGIBLE saturation §3.3's silent-clamping
prohibition points at: the author writes the bound, and an argument error
is loud. ``lo > hi`` is ``E-EVAL-044``, never a silent swap of the
bounds; ``lo == hi`` is legal (the result is that bound); an in-range
``x`` returns bit-identical — the identity, never a re-rounded copy; a
crossed bound saturates to the bound. It composes the pinned comparisons
(``min(max(x, lo), hi)``), so the signed-zero disposition is the same
first-argument-wins rule (D228). A non-finite argument in any of the
three positions is ``E-EVAL-044``.

**Costs.** Each new declaration's ``:cost`` is content-declared (§2.7),
first-declarer-sets-the-number per the ``:cost`` provenance paragraph
below — the sextet's arrival changes nothing about fuel accounting.

.. note::
   The following RNG carrier-key discussion records the retired D69/D176
   author-visible path. Amendment AJ and ADR248 supersede it: ``rng-draw`` is
   no longer declarable, and finite-kernel realization is the only private RNG
   seam.

**[draft ruling — Phase 1 review, R9 chapter C13]** *The RNG carrier-key
convention.* §2.8 sanctions RNG as a kernel intrinsic with per-(session, tick,
salt) seeding and stops there; the rst never showed the convention, and five
systems need draws. The signature stays Phase-2 work (§2.7), but the **key**
is language-visible and is fixed here:

- The carrier key is ``(session, tick, domain, stable_key)``.
- ``session`` and ``tick`` are kernel-supplied and are **never operands** — a
  rule cannot name them and therefore cannot replay a stream.
- ``domain`` is a closed-vocabulary enum operand, so content cannot mint a new
  stream; adding one is §3.6 amendment territory like any other member.
- ``stable_key`` derives from the identities of the call's reference operands,
  in operand order, using the same id bytes §2.6 orders by. It is therefore
  stable across runs and independent of insertion history.
- **A draw is a pure function of its key, not a position in a stream.** This is
  the property that matters, and it is stronger than the obvious alternative
  ("the kernel draws only when the guard consuming it passes"): because there
  is no stream position, a guard that skips a draw cannot shift any other
  draw, and §4.1's input-dependent short-circuiting can never perturb the RNG.
  The determinism obligation holds unconditionally rather than by discipline.

**Landed — #576 Task 5, superseding this D69 bullet's ``domain`` clause
above.** The enum-operand shape for ``domain`` this rider proposed is
undeclarable without a §5.6-CAS-touching grammar widening (no
``<intrinsic-decl>`` `:params` position admits an enum type). The train that
lands the signature uses **the firing rule's own id string** instead —
content cannot even *name* a stream this way, only mint a new rule, which
is already hash-covered content, which is *stronger* than the enum operand
on the "content cannot mint a new stream" axis while preserving every other
property (including the load-bearing "pure function of its key" clause
above, verbatim). This supersession is formally recorded as **D176** (#576
Task 6, below), which also states the "undeclarable as written" argument
this note only gestures at.

**The normative intrinsic table.** ADR188's own consequences paragraph
promised "the two ratified riders get normative intrinsic-table rows" (the
``floor`` prose above is that promise kept in long form); this table is the
same promise kept as an at-a-glance reference, extended to the transcendental
pair R10 caps and — under ADR219 (2026-08-22) — to the exact-arithmetic sextet.
The table is normative for all nine names.
``declarations::kernel_signature`` checks each name's ``:params`` and
``:returns`` at load.

Except for ``floor`` and ``log``, every ``:cost`` cell below stays
"author-declared." ``fuel.rs`` hard-codes no per-intrinsic cost. The first pack
to declare a name sets its number, and its conformance vector then pins that
number. The Community port's c07 first declared ``log`` with ``:cost 40``
(``community.bsl``, D210's divisor note). ``exp`` and the sextet await their
first declarations.

.. list-table::
   :header-rows: 1
   :widths: 14 16 12 8 50

   * - Name
     - ``:params``
     - ``:returns``
     - ``:cost``
     - Crossing, domain, and authority
   * - ``floor``
     - ``(real)``
     - ``int``
     - 5
     - IEEE-754 ``roundToIntegralTowardNegative``; domain ``[0, ∞)``; ADR188 Row 2 / D97.
       No libm crossing, so no golden vector (consequence declined, not omitted).
   * - ``exp``
     - ``(real)``
     - ``real``
     - author-declared; the first pack sets it, pinned by vector thereafter
     - ``libm 0.2.16`` soft-float, ``default-features = false``, via
       ``babylon_kernel::transcendental::exp``; a non-finite argument is
       ``E-EVAL-043`` (``TranscendentalOutOfDomain``), a non-finite result is
       ``E-EVAL-014``; R10/ADR176 r21 + ADR188 cap. Golden vectors:
       ``rust/crates/babylon-kernel/tests/transcendental_goldens.rs``.
   * - ``log``
     - ``(real)``
     - ``real``
     - author-declared; the first pack sets it, pinned by vector thereafter
     - As ``exp``, via ``babylon_kernel::transcendental::ln``; natural log; domain
       ``(0, ∞)`` — a non-finite argument or ``x <= 0.0`` (``-0.0`` included) is
       ``E-EVAL-043``, a non-finite result is ``E-EVAL-014``.
   * - ``sqrt``
     - ``(real)``
     - ``real``
     - author-declared
     - IEEE-754 correctly-rounded ``squareRoot`` (``f64::sqrt``); domain
       ``[0, ∞)`` — ``-0.0`` in-domain, the D97 precedent; a negative or
       non-finite argument is ``E-EVAL-044`` (``IntrinsicOutOfDomain``);
       ADR219 / D226. No libm crossing, so no golden vector (consequence
       declined, D97's shape); no result-side re-check (finite in-domain
       input gives a finite, exactly-specified output).
   * - ``round-half-even``
     - ``(real)``
     - ``real``
     - author-declared
     - IEEE-754 ``roundTiesToEven`` (``f64::round_ties_even``): the nearest
       integer VALUE to the argument within the binary64 lane, ties to the
       even neighbor (``2.5 → 2``, ``3.5 → 4``, ``-2.5 → -2``,
       ``-0.5 → -0.0``); total over finite reals (identity at
       already-integral magnitudes); a non-finite argument is
       ``E-EVAL-044``; ADR219, landing ADR188 Row 3 / D227. No libm
       crossing, no golden vector.
   * - ``min``
     - ``(real real)``
     - ``real``
     - author-declared
     - Comparison-based, never ``f64::min`` (which may return either zero
       on a ±0.0 tie and propagates ``NaN``): the second argument wins only
       on the strict comparison ``b < a``; on an equal comparison —
       ``+0.0`` vs ``-0.0`` included — the FIRST argument wins, bit-pinned
       (D228). A non-finite argument on either side is ``E-EVAL-044``;
       ADR219, superseding ADR188 Row 4.
   * - ``max``
     - ``(real real)``
     - ``real``
     - author-declared
     - As ``min``, picking the greater (strict ``a < b``, first argument
       wins on a tie, D228); a non-finite argument on either side is
       ``E-EVAL-044``; ADR219, superseding ADR188 Row 5.
   * - ``abs``
     - ``(real)``
     - ``real``
     - author-declared
     - IEEE-754 sign-bit ``abs`` (``f64::abs``), exact; ``abs(-0.0)`` is
       ``+0.0``, the canonical zero, bit-pinned; a non-finite argument is
       ``E-EVAL-044``; ADR219, superseding ADR188 Row 4 (the same row's
       nested-``if`` evidence as ``min``).
   * - ``clamp``
     - ``(real real real)``
     - ``real``
     - author-declared
     - ``(clamp x lo hi)`` — the LEGIBLE saturation §3.3's silent-clamping
       prohibition points at. Composes the pinned comparisons
       (``min(max(x, lo), hi)``, D228's first-argument-wins rule); ``lo >
       hi`` is ``E-EVAL-044``, never a silent swap of the bounds; ``lo ==
       hi`` is legal (the result is that bound); an in-range ``x`` returns
       bit-identical; a non-finite argument in any position is
       ``E-EVAL-044``; ADR219 / D228.

**``:cost`` provenance.** Shipped content declares two intrinsics today:
``floor`` — ``(intrinsic floor :params (real) :returns int :cost 5)``,
declared byte-identically in THREE packs (``territory.bsl``,
``decomposition.bsl``, ``vitality-attrition.bsl``; the co-load refusal that
triplication courts is #646, a loader matter this table does not govern) —
and ``log`` — ``(intrinsic log :params (real) :returns real :cost 40)``,
``community.bsl``'s first declaration (the Community port's c07, D210's
divisor note). The ``floor`` declarations are also the proof that the whole
intrinsic path is production-wired end to end, not merely
unit-tested. ``floor``'s row reads **5** and ``log``'s **40**, quoted from
content.

``fuel.rs`` hard-codes no per-intrinsic cost. No kernel-fixed number
exists for ``exp`` or the ADR219 sextet. The first content pack to declare each
name sets its ``:cost``. Its conformance vector then pins that number.

4. Dynamic semantics
----------------------

4.1 Evaluation order
~~~~~~~~~~~~~~~~~~~~~~

Evaluation is **strict, call-by-value, left to right**, depth-first. For any
form, operands are evaluated in the order they appear in the *source* (which,
after §5.3's canonicalization, is the order they appear in the canonical AST).

Exceptions, all deliberate:

- ``and`` short-circuits on the first ``#f``; ``or`` on the first ``#t``.
- ``if`` and ``guard`` evaluate only the taken branch.

Short-circuiting makes *consumed* fuel input-dependent; it does not make it
non-deterministic, and the *bound* of §3.7 is a static worst case. Conformance
vectors therefore pin an exact consumed-fuel figure per (rule, environment)
pair (§6.1).

4.2 The environment
~~~~~~~~~~~~~~~~~~~~~

A rule evaluates against: the graph (read-only during condition evaluation),
the defines environment (coefficients), the current tick, the subject node, and
the fuel meter. Bindings resolve first, in declaration order — the external
sources by lookup, the ``:expr`` bindings (§2.5) by evaluation against the
bindings already resolved. The engine applies effects after it evaluates the
complete condition. **A rule cannot observe its own effects.** The language
gives every subject firing of that rule the same prestate. Rules at one
resolved system position, however, compose sequentially: a later rule observes
the working-copy writes of every earlier rule, including an earlier
same-position rule (D116, ADR224). This is the governed composition model, not
a temporary divergence from a shared-prestate target.

Rules execute in the 34-slot causal order from §2.3. D16 orders rules at one
resolved position by rule-id bytes. The engine applies their effects in that
order. File order and load order cannot affect execution.

**[draft ruling — Phase 1 review, R9 chapter C4]** *Subject enumeration order,
which this document had not specified.* A rule whose domain is a node type
(§2.3) fires once per node of that type, in **ascending node-id byte order** —
the same key §2.6 uses for queries. All firings of one rule observe the same
pre-state (the law above is unchanged), and the effects they collect are
applied in that subject order, and within one subject in source order. A
``(domain :graph)`` rule fires once and takes its place among the rules at its
anchor by rule id like any other.

The order is not a formality. Accumulation into a shared target — every class
adding its slice to a carrier node (§3.6) — reduces in exactly this order, and
the binary64 lane of §3.3 is not associative, so an unspecified subject order
would make the result implementation-dependent while every other clause of this
chapter held. Currency's exact integer lane is immune; the bounded scalars are
not, which is why the order is stated rather than left to the executor.

**Task W2 (BSL Hygiene Knock-out, 2026-08-18) — same-tick ordering as LOAD
REFUSALS.** The resolved phase order, D16's same-position rule-id tie-break,
and D116's recorded cross-rule same-tick visibility make a rule's read order
*content-visible*: whether an ``:optional :default`` binding's declared
default is ever observed depends on where its field's writers execute. Two
load refusals make that dependency a checked property of the content set
rather than a silent hazard, in the same family as ``E-LOAD-001``
(content-set-wide, checked at load, over whatever set is actually loaded —
never a hypothetical wider one).

*Refusal 1 —* ``E-LOAD-058`` *, stale-default read.* For a binding
``(binding b :field f :optional :default d)`` in rule ``R``, the *writer
set* of ``f`` is every rule ``W`` in the loaded content set such that
``W`` contains an ``(update-node <target> f …)`` effect, **excluding**
``W`` **where** ``W.rule_id == R.rule_id``. The exclusion is on rule
identity alone. It is independent of the write target — ``self``,
``it``, a ``select-max`` result, or any ``NodeRef`` — and independent of
whether the read and the write resolve to the same node (justification:
this section's own two clauses above — "a rule can never observe its own
effects" **and** "all firings of one rule observe the same pre-state";
the second is the operative one when the read and write targets differ,
which is exactly ``solidarity/p0-transmit``'s shape: it reads ``self``
and writes a neighbour ``it``, one rule, still self-excluded). If any
writer does not execute before ``R`` under the resolved phase rank and D16
tie-break, the loader refuses the content. The error names the field and both
rules.

*Refusal 2 —* ``E-LOAD-059`` *, unreset fan-in.* A field written by two or
more distinct rules in the content set (via ``set``, or via fan-in
``add``/``sub``/``scale``) requires an earlier **unconditional** ``set``.
The rule has no ``(when …)`` at all, or its ``(when …)`` is the literal
``#t``, and the ``set`` is a direct member of the rule's effects sequence.
A ``set`` beneath ``guard``, ``for-each``, or another construct that can
execute zero times is conditional and cannot discharge this refusal. The
analyzer performs no finer guard-dominance analysis between a reset and its
fan-in writers. The alternative is the D127 unconditional-recompute shape
(a writer meeting the same unconditional test whose ``:material-basis``
cites D127). Sort the writers first by resolved execution rank and then by D16
rule-id bytes. The first writer must pass one of the two tests above.
Otherwise, the loader refuses the content and names the field and writer rules.

**ADR224 activates both refusals.** The analyzer consumes resolved phase ranks
from ``babylon-tick`` over the complete aggregate content set after
concatenation. Rank orders first; D16 rule-id bytes break only same-rank ties.
The former raw-ID, per-source adapter remains historical test support and does
not decide a production load. Thus Solidarity in Material Base correctly
precedes Consciousness in Consequences even though their raw ids sort in the
opposite order, while real same-position and later-writer hazards still refuse.
The public ranked analyzer also verifies its own input before it computes a
finding: every supplied rule id must equal the qname inside that rule form, and
the set of ids must be unique. A non-rule form, mismatched id, or duplicate id
returns a deterministic typed load refusal and no E-LOAD-058 or E-LOAD-059
finding.

Intentional findings use exact governed dispositions, not a reusable content
escape hatch. An E-LOAD-058 row keys the reader rule, binding, field, and
complete allowed non-earlier writer set. An E-LOAD-059 row keys the field,
complete allowed writer superset, each allowed execution-earliest writer, and
that writer's reset-relevant shape. A new or near-match reader, binding, field,
writer, earliest writer, or changed reset shape refuses. Every row carries its
reason, Director owner, date, and ADR224 authority. The initial tables contain
13 E-LOAD-058 rows and 11 E-LOAD-059 field rows.

The historical Amendment AI draft is not ratified and grants no vocabulary.
``:prior-tick``, ``:complementary-writers``, and
``:sequential-accumulator`` are not active declarations. ADR224 preserves the
draft as design evidence but replaces its proposed implementation path with the
exact disposition tables above.

4.3 Arithmetic
~~~~~~~~~~~~~~~~

- Binary64 operations are the IEEE-754 **basic** operations only:
  addition, subtraction, multiplication, division, and comparison, each
  correctly rounded round-to-nearest-even. These reproduce bit-exactly across
  conforming implementations.
- Fixed-point operations are exact integer operations at the widths of §3.2.
- **No transcendental is a language operation.** Any that exists is a named
  intrinsic whose implementation is pinned by the kernel and validated by
  golden vectors with a written tolerance derivation. Which ones may exist is
  §3.10's, and it is a shorter list than the illustrative names above
  suggest. The polynomial-vs-libm choice is **ruled, not open**: ADR176
  ruling 21, reaffirmed by ADR188's decision paragraph, mandates a **pinned
  soft-float libm crate with per-intrinsic golden vectors** — ``exp`` and
  ``log`` cross via ``libm 0.2.16`` (``default-features = false``),
  wrapped in ``babylon_kernel::transcendental``; see
  :doc:`/reference/determinism-contract`'s *Transcendental Crossing —
  exp/log* chapter for the verified dispatch analysis and the written
  tolerance derivation.
- No fused multiply-add. An implementation that contracts ``a*b+c`` into an FMA
  is non-conforming.
- ``Int`` overflow is ``E-EVAL-011``.
- A binary64 operation producing a non-finite result (``inf``, ``-inf``,
  ``NaN``) is ``E-EVAL-014``. Division by zero in the binary64 lane is
  ``E-EVAL-012``. Non-finite values are therefore **not representable** at any
  observable point, which is what makes the JSON/serialization inf-NaN trap
  unreachable by construction.
- The ``floor`` intrinsic's argument outside its ratified domain — negative,
  non-finite, or a result that would not fit ``Int``'s ``i64`` domain — is
  ``E-EVAL-039`` (§3.10, ADR188 Row 2, D97).

4.4 Query evaluation
~~~~~~~~~~~~~~~~~~~~~~

A query is materialized in the sort order of §2.6 before the fold body runs.
The graph is not mutated during condition evaluation, so the materialized set
cannot change under the fold. ``mean`` over an empty set is ``E-EVAL-021``
(never ``0``); ``sum`` over an empty set is the additive identity of the body
type; ``count`` over an empty set is ``0``; ``min``/``max`` over an empty set
are ``E-EVAL-021``. ``exists`` over an empty set is ``#f``; ``forall`` over an
empty set is ``#t``.

4.5 Fuel accounting
~~~~~~~~~~~~~~~~~~~~~

Each AST node charges its **base** cost when it is evaluated — the same base
numbers §3.7 uses to compute the static bound, without the ceiling
multiplication (the multiplication is the static worst case; at runtime each
actual iteration charges the body's cost once). The meter starts at the rule's
declared ``:fuel`` and decrements. A ``:expr`` binding charges its expression
**once**, when the binding resolves; each later *reference* to it charges a
variable-reference 1 like a reference to any other binding. That asymmetry is
the whole of the fuel win C7 buys, and it is why the same algebra written twice
inline costs strictly more than the same algebra named once.

Reaching or passing zero is ``E-EVAL-040``, which aborts the tick (§4.6) — it
never truncates a fold or returns a partial result.

**[draft ruling — Phase 1 review, R9 verification repair]** *The meter is per
firing.* A node-domain rule fires once per subject (§4.2), and **each firing
starts a fresh meter at the declared** ``:fuel``: the budget is a property of
one evaluation, never of a rule's whole pass over its subjects. That is the
only reading consistent with ``bound(rule)`` (§3.7), which carries no
subject-count factor, and the only one under which a rule's legality at load
does not depend on how many nodes the scenario it is later hydrated against
happens to hold. Chapter C4's subject enumeration made the multiplicity
explicit and therefore made this sentence necessary. It follows that a
conformance vector's ``:fuel-used`` (§6.1) is also per firing; a vector whose
rule fires over several subjects — §6.2 family 12's accumulation vector — states
the figure for the **first** subject in §4.2's order, which is well defined
because that order is.

The cost table is **pinned by conformance vector; any revision is a vector
re-bless** (design §5 Totality). This is what makes fuel a stable, hashable
property of content rather than an implementation detail.

**[draft ruling — Phase 1 review]** *The §3.7/§4.5 boundary is off by one*
(implementation-discovered, 2026-07-30, Phase 1 Task 14). §3.7 rejects only
``bound(rule) > :fuel`` at load, while this section's meter must stay
**strictly positive** — "reaching or passing zero" — so a rule whose worst
case consumes exactly its ``:fuel`` loads and then ``E-EVAL-040``\ s at
runtime. Both checks are implemented to the letter; authors should budget
``:fuel ≥ bound + 1``. The Phase-1 review may align the two by making the
load check ``bound ≥ :fuel``.

4.6 Error taxonomy
~~~~~~~~~~~~~~~~~~~~

Every error in this document is loud (Constitution III.11). There are exactly
two times at which an error can occur.

.. list-table::
   :header-rows: 1
   :widths: 18 22 60

   * - Class
     - Codes
     - When, and what happens
   * - Lexical
     - ``E-LEX-0xx``
     - Content load. The content set is rejected; no engine starts.
   * - Syntactic
     - ``E-PARSE-0xx``
     - Content load. As above.
   * - Static/type
     - ``E-TYPE-0xx``
     - Content load. As above.
   * - Load/link
     - ``E-LOAD-0xx``
     - Content load — unresolved bindings, unknown vocabulary, fuel bound
       exceeded, ceiling violated at hydration, a missing or misplaced
       ``:max-members`` or ``:invariant``, a member list over that ceiling,
       anchor interleaved into the Material Base partition, kernel/content
       disagreement; and, from the R9 chapters, an undeterminable or ambiguous
       rule domain, a field owner that names no registered type, a
       node/edge/hyperedge type-rendering collision, a metric read through the
       wrong form for its declared domain, a structural verb naming an
       ``:invariant`` type, a reserved or prohibited intrinsic name, ``the``
       against a type whose ceiling is not 1, a hydration seeding one edge key
       twice, and a manifest with no row for a type the content set uses; and,
       from the Amendment AG sections, a membership payload field read or
       written through the wrong form, a membership minted or hydrated with an
       incomplete payload, a ``:member`` declaration whose owner segment is
       not a hyperedge type, a ``:substrate`` rung over a type that is not
       ``:invariant``, an intensive adjunction with no weight, and an
       adjunction that does not fit the schema at its declared rung; and, from
       the §3.2 addendum (#492/ADR194), a ``defconst``'s own ``Ratio`` value
       exceeding its declared ``:cap`` ceiling; and, from §2.13 (Organization
       spec §1 Q12), a ``:type enum``/``:enum-type`` shape violation, a
       ``defenum``-registry type or member an ``:enum-type`` or ``<enum-ref>``
       does not resolve, and a non-``<enum-ref>`` value seeding an
       ``:enum-type``-declared field in a ``.bscn`` body; and, from Train B
       item 3 (#591), a hydration seeding one
       ``(edge-type, source, target, field)`` edge-attribute key twice; and,
       from Task W2 (BSL Hygiene Knock-out), a same-tick-ordering violation
       — an ``:optional :default`` binding a same-content-set writer can
       observe as stale, or a multi-writer field with no rule-identifiable
       reset — now enforced through ADR224's exact dispositions (§4.2).
   * - Evaluation
     - ``E-EVAL-0xx``
     - During a tick — checked-arithmetic failure, range violation at a store,
       non-finite result, empty aggregate, edge-mode violation, hyperedge type
       mismatch, fuel exhaustion; and, from the R9 chapters, an accessor whose
       referent is of the wrong type or carries no value for the named field,
       an ``edge-between`` that resolves to no edge, a ``the`` against an
       unhydrated carrier, and a ``metric-of`` against the wrong element type
       or a value the provider did not produce; and, from the Amendment AG
       sections, a named *(hyperedge, member)* pair that is not a membership;
       and, from the §3.2 addendum (#492/ADR194), a ``Currency × Ratio``
       operand exceeding its declared domain ceiling; and, from §2.13
       (Organization spec §1 Q12), a non-``<enum-ref>`` value (or a
       cross-type ``<enum-ref>``) reaching an ``:enum-type``-declared
       field's write path at runtime, an ``<enum-ref>`` of the right
       declared type naming a member that type does not declare
       (``E-LOAD-055``'s runtime twin), and an ``add``/``sub``/``scale``
       update-op targeting such a field (D118) — ``Enum<T>`` supports no
       arithmetic.

**Every code the R9 chapters add continues an existing sequence, and no code
that existed before this revision is renumbered.** The new codes are
``E-LOAD-0xx`` (eleven, because most of what those chapters add is decidable
from the content set alone), ``E-PARSE-0xx`` (six), ``E-TYPE-0xx`` (five) and
``E-EVAL-0xx`` (five, all of them the "absence is never a value" discipline of
§2.10 applied at a new referent). That the load class grew fastest is the
intended shape: a chapter that made a new failure mode *runtime*-only would
have moved the language in the wrong direction.

**The Amendment AG sections continue the same sequences.** They add six
``E-LOAD-0xx`` codes — three per construct — and one ``E-EVAL-0xx``, and
**no** ``E-LEX``,
``E-PARSE`` or ``E-TYPE`` code at all: every parse- and type-class failure
they can produce already had one — ``E-PARSE-013`` for a keyword outside its
form, ``E-PARSE-041`` for two writers of one field in one form,
``E-TYPE-011`` for an ``<enum-ref>`` of the wrong kind, ``E-TYPE-014`` for a
field-init owning off the wrong type, ``E-TYPE-041``/``042``/``043`` for the
kind law — and minting a synonym for an existing class is exactly the hygiene
defect D75 ruled against.

Sequence continuation is meant literally, and is checkable by inspection: every
decade block of every family is **contiguous**, with no reserved and no
skipped number — ``E-LOAD`` 001–004, 010–013, 020–025, 030–033, 040–059;
``E-PARSE`` 010–015, 020–022, 030–033, 040–042; ``E-TYPE`` 010–017, 020, 030,
040–044; ``E-EVAL`` 010–014, 020–021, 030–043; ``E-LEX`` 001–003,
010–011, 020–027. ``E-TYPE-044`` (Territory port train, P27, #551 closure)
continues the SAME overrun block — the next free ``E-TYPE`` number at the
time of allocation, per the same rule. The ``E-LOAD`` 040 block now runs past its own decade, and
deliberately: opening a fresh block at 050 for the Amendment AG codes would
have **reserved** 046–049, and a reserved number is precisely what the rule
above forbids. Contiguity outranks decade tidiness. The §3.2 addendum
(#492/ADR194) continues the SAME overrun block with ``E-LOAD-052``, and
continues ``E-EVAL`` with ``E-LEX-027`` and ``E-EVAL-041`` — the next free
number in each family at the time of allocation, per the same rule.
**Repair, found while allocating those numbers:** this paragraph's ``E-EVAL``
range previously read "030–038, 040", silently skipping ``E-EVAL-039``
(``floor``'s domain check, D97/ADR188 Row 2) — a transcription gap in this
paragraph specifically, not a gap in the actual sequence (``039`` has been
allocated and implemented since D97 landed). Folded into the corrected range
above rather than left standing as a second inconsistency next to a new one.
The R9 chapters allocated per chapter and left two holes in
the ``E-TYPE`` sequence; they were closed by renumbering the two offending
**new** codes before any implementation pinned them, which is a liberty
available exactly once and only to codes this revision minted.

**§2.13 (Organization spec §1 Q12) continues the same sequences a third
time.** It adds four ``E-LOAD-0xx`` codes — ``E-LOAD-053`` (the
``:kind``/``:enum-type`` shape exclusivity), ``E-LOAD-054`` (an
``:enum-type`` naming an unregistered ``defenum`` type), ``E-LOAD-055``
(an ``<enum-ref>`` naming a member its resolved type does not declare),
``E-LOAD-056`` (a non-``<enum-ref>`` value seeding an ``:enum-type``-declared
field at content-load time) — and one ``E-EVAL-0xx``, ``E-EVAL-042`` (the
same law's runtime re-check, §3.7/§4.3's two-site pattern), continuing the
same overrun ``E-LOAD`` block past 052 for the same reason the §3.2
addendum did: a fresh 060 block would reserve 057–059 for nothing. It
mints **no** ``E-LEX``, ``E-PARSE`` or ``E-TYPE`` code: a ``defvocabulary``
membership violation is not a new failure class at all — it reuses
``E-LOAD-030``/``E-LOAD-031``, the codes §3.6's ``ClosedVocabulary``
already carries, activated at three producers rather than left
unreachable (Task 8 of the Organization foundation plan); minting
parallel numbers for an identical check against a different registry
would be exactly the hygiene defect D75 ruled against, the same reasoning
the Amendment AG sections' paragraph above already applies to its own
parse- and type-class reuse.

**Train B item 3 (#591) continues the same** ``E-LOAD`` **block a fourth
time.** ``E-LOAD-057`` — a hydration seeding one
``(edge-type, source, target, field)`` edge-attribute key twice (§3.9
clause 7, D156) — is the next free number at the time of allocation, per
the same rule.

**Task W2 (BSL Hygiene Knock-out, 2026-08-18) continued the same**
``E-LOAD`` **block a fifth time.** ``E-LOAD-058`` (stale-default read) and
``E-LOAD-059`` (unreset fan-in) are the next two free numbers at the time
of allocation, per the same rule; full normative text at §4.2's own end
(same-tick evaluation order is that section's subject). ADR224 activates both
rows over the post-phase-compile aggregate content set and governs the exact
intentional findings through disposition tables. No ordering vocabulary was
added. See §4.2 and ADR224.

**PER-19 continues** ``E-LOAD`` **a sixth time.** ``E-LOAD-060`` is an
unauthorized effect by a restricted causal role. The loader walks the complete
nested effect tree. A ``Recognizer``, ``ExternalEvent``, or ``Intent`` field or
event effect must match an exact governed rule/role/effect row; every shape
effect refuses. ``Mechanic`` retains the ordinary typed effect surface.

**The T2 practice-contract groundwork continues** ``E-LOAD`` **with five
pre-mutation topology refusals.** ``E-LOAD-061`` refuses the 4,097th exact
``NodeType/ORGANIZATION`` row. ``E-LOAD-062`` refuses the 257th exact
organization-to-``NodeType/SOCIAL_CLASS`` ``EdgeType/SOLIDARITY`` edge for
one organization. ``E-LOAD-063`` refuses an active organization without an
``organization/action-budget`` value. ``E-LOAD-064`` refuses non-canonical
ActionBudget storage. ``E-LOAD-065`` requires both
``organization/action-budget`` and ``organization/active`` to have the exact
``int intensive`` declaration signature. These checks run before the loader's
first graph operation. A duplicate canonical solidarity triple retains
``E-LOAD-044``; this addition does not assign it a second identity.

**The #576 intrinsic-host train (Task 2) continues** ``E-EVAL``.
``E-EVAL-043`` — ``TranscendentalOutOfDomain``: a non-finite ``exp``/``log``
argument, or a non-positive ``log`` argument (§3.10, R10/ADR176 r21, ADR188
cap) — is the next free number at the time of allocation, per the same
rule.

**The ADR219 exact-arithmetic rider train (2026-08-22) continues**
``E-EVAL`` **once more.** ``E-EVAL-044`` — ``IntrinsicOutOfDomain``: an
argument outside one of the six exact-arithmetic intrinsics' ratified
domain — a negative argument to ``sqrt`` (``-0.0`` excepted, the D97
precedent), ``clamp``'s ``lo > hi``, or a non-finite argument to any of
``sqrt``/``round-half-even``/``min``/``max``/``abs``/``clamp`` — is the
next free number at the time of allocation, per the same rule. Reusing
``E-EVAL-043`` was considered and rejected: these six are not
transcendentals, and a code whose name says ``Transcendental`` would lie
about half its referents.

**Load-time errors** report the offending file, line, column, form, and code,
and reject the whole content set — there is no partial load and no "skip the
bad rule" mode.

**Evaluation errors** abort the tick. The whole per-tick envelope transaction
rolls back; there are no partial commits (design §9). The error carries the
rule id, the AST path to the offending node, the binding environment, and the
fuel remaining. An implementation must not convert an evaluation error into a
default value, a skipped effect, or a log line.

4.7 Cross-system registers and one-tick handoffs
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Once graph-scope state is node state (§3.6), a whole class of construct stops
being necessary, and this section exists to say so rather than to add anything.

Several values in the frozen estate are **one-tick-lagged handoffs** between
systems at different anchor positions — Sovereignty's output read by Metabolism,
MarketScissors' correction read by WealthDistribution, Production's value read
by ImperialRent. In the Python engine those live in ``persistent_data`` and the
lag is a property of where the write and the read happen to sit.

**[draft ruling — Phase 1 review, R9 chapter C3]** *There is no staging,
double-buffering or "previous tick" construct, and none is needed.* A rule
writes a carrier field at its anchor position; a rule at a later anchor
position in the same tick reads the new value; a rule at an earlier position
reads the value the previous tick left. Tick ordering already **is** the
staging mechanism, and it is the one mechanism whose behaviour §4.2 has
specified since this document's first revision. Adding a ``previous`` accessor
or a staged-write verb would introduce a second notion of "when", and every rule
in the estate would then have to be read twice to know which one it meant.

Two obligations follow, and they are content obligations rather than language
ones:

1. **The lag is declared by the anchor, so the anchor is load-bearing.** A rule
   pack that moves a read across the writing system's position silently
   converts a one-tick lag into a same-tick read. The anchor is inside
   ``rules_hash``, so the change is visible in the diff — but it is visible as
   an anchor edit, not as a semantic one, and review should treat it as the
   latter.
2. **A carrier field read before anything has written it reads what hydration
   seeded**, which is why §3.5's plain-binding rule matters here: a carrier
   field is an ordinary declared field, so a rule reading one that the scenario
   never seeded is ``E-LOAD-010`` at load, not a zero at tick 1.

5. Canonical AST serialization
--------------------------------

``rules_hash`` is ``SHA-256`` over the canonical AST serialization (CAS) of the
content set. CAS is **whitespace-insensitive, comment-insensitive, file-layout
insensitive, and option-order insensitive**: a formatting edit produces
identical bytes, while any change to a rule's meaning — including its
``:material-basis`` string and its ``:fuel`` budget — produces different bytes.

This chapter defines the bytes. What ``rules_hash`` is *combined with*
(``ContentDigest``), and how it relates to the tick hash and ``defines_hash``,
belongs to :doc:`/reference/determinism-contract`.

5.1 Primitive encodings
~~~~~~~~~~~~~~~~~~~~~~~~~

All multi-byte integers are **big-endian**, two's complement where signed.

.. list-table::
   :header-rows: 1
   :widths: 16 14 70

   * - Name
     - Width
     - Meaning
   * - ``u8``
     - 1
     - Unsigned length or tag byte.
   * - ``u32``
     - 4
     - Unsigned length or child count.
   * - ``i64``
     - 8
     - Signed integer literal value.
   * - ``i128``
     - 16
     - Signed unscaled decimal / Currency micro-units.
   * - ASCII
     - variable
     - Names; always length-prefixed, never NUL-terminated.
   * - UTF-8
     - variable
     - String payloads; NFC; always length-prefixed.

5.2 Node encodings
~~~~~~~~~~~~~~~~~~~~

There are exactly two node shapes.

.. code-block:: text

   atom ::= 0x01  u8 len(kind)  kind_ascii  u32 len(payload)  payload
   form ::= 0x02  u8 len(tag)   tag_ascii   u32 nchildren     child*

Both are self-delimiting, so the encoding is unambiguously parseable back to the
AST — a property implementations should exercise as a round-trip property test.

**Atom kinds and payloads:**

.. list-table::
   :header-rows: 1
   :widths: 14 86

   * - ``kind``
     - payload
   * - ``int``
     - ``i64`` value
   * - ``cur``
     - ``i128`` micro-units (scale is implicit and always 6)
   * - ``prob``
     - ``i128`` unscaled ``||`` ``u8`` scale (minimal scale; zero is ``0,0``)
   * - ``intn``
     - as ``prob``
   * - ``coef``
     - as ``prob``
   * - ``bool``
     - one byte: ``0x00`` = ``#f``, ``0x01`` = ``#t``
   * - ``sym``
     - ASCII symbol, no sigil
   * - ``qname``
     - ASCII qualified name including its ``/`` separators
   * - ``kw``
     - ASCII keyword name **without** the leading ``:``
   * - ``enum``
     - ASCII ``<EnumType>/<MEMBER_IDENTIFIER>`` (an ``<enum-ref>`` value,
       §1.4) **or**, for ``defenum``/``defvocabulary``'s own BARE
       operands (§2.13's type-name operand and member-list items — never
       ``<enum-ref>`` pairs), the bare identifier alone with no ``/``.
       Collision-free by construction: an ``<enum-ref>`` payload always
       contains exactly one ``/`` (§1.4's ``enum-type``/``enum-member``
       charsets exclude it), a bare operand's payload never does, so the
       PRESENCE of exactly one ``/`` is the discriminator a decoder reads
       the payload back with — no numeric flag or second kind tag needed
       (D117).
   * - ``str``
     - UTF-8 bytes after escape processing, NFC

**Form tags** are the form's head symbol verbatim (``rule``, ``bindings``,
``binding``, ``when``, ``effects``, ``and``, ``or``, ``not``, ``<``, ``<=``,
``>``, ``>=``, ``=``, ``!=``, ``+``, ``-``, ``*``, ``/``, ``if``, ``fold``,
``exists``, ``forall``, ``nodes``, ``edges``, ``neighbors``, ``hyperedges``,
``members-of``, ``hyperedges-of``, ``field-of``, ``edge-between``, ``the``,
``domain``, ``select-max``, ``select-min``, ``metric``, ``metric-of``,
``quantize-mass``, ``guard``, ``for-each``, ``choose``, ``branch``,
``update-node``, ``update-edge``,
``add-node``, ``remove-node``, ``add-edge``, ``remove-edge``,
``add-hyperedge``, ``update-hyperedge``, ``remove-hyperedge``, ``members``,
``member``, ``membership-field-of``, ``update-membership``,
``emit``, ``add``, ``sub``, ``set``, ``scale``, ``anchor``, ``deffield``,
``intrinsic``, ``manifest``, ``ceiling``, ``rung``, ``adjunction``,
``defenum``, ``defvocabulary``), plus the synthetic tag ``opt`` for a
keyword option.

The six tags the Amendment D revision added — ``hyperedges``, ``members-of``,
``hyperedges-of``, ``add-hyperedge``, ``remove-hyperedge``, ``members`` — obey
that same rule (the tag *is* the head symbol), so they need no registry entry,
no numeric id, and **no new atom kind**; ``:max-members`` is an ordinary
keyword and encodes as an ``opt`` form like every other option. Nothing else in
this chapter changes. In particular the §5.6 worked example contains none of
these forms, so **its then-current 421 canonical bytes and both digests were
unchanged by this revision** — a hyperedge-bearing example would need its own vector
(§6.2), not a recomputation of that one.

**The R9 tags obey the same rule, and the worked example survives them too.**
Every form tag the R9 spec chapters add — listed in the paragraph above
alongside the originals — is its own head symbol, needs no registry entry, no
numeric id and no new atom kind; every keyword they add encodes as an ``opt``
form. None of them appears in §5.6's example, and none of the R9 chapters makes
a previously-optional child of ``rule`` mandatory, so **§5.6's then-current
421 bytes and both digests stayed unchanged**. That invariance was deliberate: it is
the cheapest available proof that the chapters are additive.

**The Amendment AG tags obey it a third time.** ``member``,
``membership-field-of`` and ``update-membership`` (§2.8, §2.10, §2.12) are
their own head symbols, needing no registry entry, no numeric id and no new
atom kind; ``:member`` encodes as an ``opt`` form under D20 like every other
keyword. ``deffield`` gains one **optional** child and ``add-hyperedge``'s
``<members>`` gains an alternative item shape that a bare member list never
uses, so no existing form's encoding moves. Clause (ii)'s ``rung`` and
``adjunction`` are the same story one level up: their tags are their head
symbols, their keywords are ``opt`` forms, and they are **optional children of
a form that already has a digest** (§5.5's ``manifest``), so a manifest that
declares no lattice encodes byte-for-byte as it did. None of these appears in
§5.6's example, and no previously-optional child of ``rule`` becomes mandatory
— **§5.6's then-current 421 bytes and both digests stayed unchanged**.

**The Organization contract's Q12 tags obey it a fourth time — with one
payload-shape correction, D117 (#528 fix round, adversarial-verifier
found).** ``defenum`` and ``defvocabulary`` (§2.13) are their own head
symbols, needing no registry entry and no numeric id. **"No new atom
kind" is still true — the kind tag stays the SAME "enum" string — but an
earlier revision of this paragraph overstated WHY**, claiming "the ``<enum-ref>``
values they govern already encode with the existing atom kind", as though
only values these forms GOVERN were ever encoded. That is true of an
``:enum-type``-declared field's own stored value (a genuine ``<enum-ref>``
pair, unchanged), but false of ``defenum``/``defvocabulary``'s OWN
operands: the type-name operand and each member-list item are BARE
``<enum-type>``/``<enum-member>`` atoms (§2.13's own EBNF — no ``/`` at
all), not ``<enum-ref>`` pairs, so encoding them at all requires that
SAME kind's payload to admit a SECOND shape — declared above in the
atom-kind/payload table, discriminated by the presence of exactly one
``/``. The correction is additive, not a new kind tag: every existing
``<enum-ref>`` payload is byte-identical to before, and the bare shape is
new bytes for a construct (``defenum``/``defvocabulary``) that did not
exist before Q12 either. ``deffield`` gains one more **optional**
alternative in an existing keyword slot (``:enum-type`` beside ``:kind``,
§2.9), not a new child position, so no existing ``deffield`` encoding
moves. None of these forms appears in §5.6's example, and neither
``deffield`` nor any other form gains a newly *mandatory* child —
**§5.6's then-current 421 bytes and both digests stayed unchanged**, the same
proof of additivity every prior extension in this
section carries.

**ADR224 moves the worked example deliberately.** ``:role`` and
``:evidence`` are mandatory valued options on every rule. They use the existing
``opt`` shape and ASCII option ordering, but their mandatory presence adds two
children to §5.6's rule. The current vector is therefore 500 bytes with eight
children. Its CAS digest is
``9b49c1e87a63ccb64dee96bd179c8ffecd3fe476c81a4f81771daac237156396``;
§5.6 records the one-rule ``rules_hash``.
The earlier 421-byte statements above remain historical invariance claims
about their own additive revisions; they do not override the current vector.

A keyword option is encoded as a two-child form:

.. code-block:: text

   opt ::= form("opt", atom("kw", <name>), <value node>)

A flag keyword (``:optional``) encodes as ``form("opt", atom("kw","optional"),
atom("bool", 0x01))`` **[draft ruling — Phase 1 review]** so that every option
has the same shape.

5.3 Canonical child order
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Within any form, children are emitted in three groups:

1. the form's **fixed positional operands**, in the order the §2 grammar
   declares them;
2. all **keyword options**, sorted by keyword name in ascending ASCII byte
   order;
3. the **variadic body**, in source order.

Group 2's sort is what makes option order a formatting concern. Group 3's
source order is load-bearing: order is structure (the effect application order
of §2.8 is exactly this order).

**[draft ruling — Phase 1 review, R9 chapter C4]** ``<domain>`` *encodes in
its grammar position.* Like ``<anchor>``, ``<bindings>``, ``<when>`` and
``<effects>``, a ``domain`` form is a child of ``rule`` emitted in the order
the §2.3 grammar declares it — immediately before ``anchor``. It is optional
and absent from §5.6's example, so that example's byte count and both its
digests are unaffected. Its own children follow the general rule: an
``<enum-ref>`` domain encodes as one ``atom enum`` child, and ``(domain
:graph)`` as one ``opt`` form carrying the ``graph`` flag under D20.

5.4 Determinism obligations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Comments and whitespace never appear in CAS.
- Decimal literals are minimal-scale (§1.5), so ``0.50c`` and ``0.5c`` produce
  identical bytes.
- Currency literals are micro-unit integers, so ``1000.5$`` and ``1000.500$``
  produce identical bytes.
- Strings are NFC by load-time check, so "the source bytes" is a canonical
  form.
- No floating-point value is ever serialized. There is no ``binary64`` in CAS
  and therefore no float-formatting ambiguity anywhere in the hash path.
- ``str()``-style fallbacks are banned outright: every node kind has an
  explicit encoding above, and an encoder that meets an unencodable value must
  fail loudly rather than stringify it.

5.5 ``rules_hash``
~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   rules_hash = SHA-256( 0x03 || u32 N || CAS(r_1) || … || CAS(r_N) )

where ``r_1 … r_N`` are all ``rule`` forms in the content set sorted by rule id
in ascending ASCII byte order, and ``N`` is their count. ``deffield``,
``intrinsic``, ``manifest`` and ``metric`` forms are hashed the same way into
their own digests, which ``ContentDigest`` combines
(:doc:`/reference/determinism-contract`). The digest is rendered as **64
lowercase hex characters** — never truncated. (Truncation to 16 hex is exactly
the defect the ``defines_hash`` triad carried; see the determinism contract's
worked-example chapter.)

5.6 Worked example
~~~~~~~~~~~~~~~~~~~~

Source form:

.. code-block:: scheme

   ; a rule is data; this comment is not part of the hash
   (rule demo/hunger
     :role mechanic
     :evidence derived
     :material-basis "subsistence deficit at the point of reproduction"
     :fuel 64
     (bindings
       (binding wealth :field social-class/wealth))
     (when (< wealth 1000.5$))
     (effects
       (update-node self social-class/agitation (add 0.05i))))

Canonical AST after §5.3 reordering (the option keys appear in ASCII order:
``evidence``, ``fuel``, ``material-basis``, ``role``):

.. code-block:: text

   form "rule" (8 children)
     atom qname "demo/hunger"
     form "opt" : atom kw "evidence",       atom sym "derived"
     form "opt" : atom kw "fuel",           atom int 64
     form "opt" : atom kw "material-basis",
                  atom str "subsistence deficit at the point of reproduction"
     form "opt" : atom kw "role",           atom sym "mechanic"
     form "bindings" (1)
       form "binding" (2)
         atom sym "wealth"
         form "opt" : atom kw "field", atom qname "social-class/wealth"
     form "when" (1)
       form "<" (2)
         atom sym "wealth"
         atom cur 1000500000            ; 1000.5 × 10^6 micro-units
     form "effects" (1)
       form "update-node" (3)
         atom sym "self"
         atom qname "social-class/agitation"
         form "add" (1)
           atom intn (unscaled 5, scale 2)

Canonical bytes — 500 bytes, hex (wrapped at 64 characters for display only):

.. code-block:: text

   020472756c65000000080105716e616d650000000b64656d6f2f68756e676572
   02036f70740000000201026b770000000865766964656e6365010373796d0000
   00076465726976656402036f70740000000201026b77000000046675656c0103
   696e7400000008000000000000004002036f70740000000201026b770000000e
   6d6174657269616c2d626173697301037374720000003073756273697374656e
   636520646566696369742061742074686520706f696e74206f6620726570726f
   64756374696f6e02036f70740000000201026b7700000004726f6c6501037379
   6d000000086d656368616e6963020862696e64696e677300000001020762696e
   64696e6700000002010373796d000000067765616c746802036f707400000002
   01026b77000000056669656c640105716e616d6500000013736f6369616c2d63
   6c6173732f7765616c746802047768656e0000000102013c0000000201037379
   6d000000067765616c7468010363757200000010000000000000000000000000
   3ba26b2002076566666563747300000001020b7570646174652d6e6f64650000
   0003010373796d0000000473656c660105716e616d6500000016736f6369616c
   2d636c6173732f616769746174696f6e0203616464000000010104696e746e00
   0000110000000000000000000000000000000502

Reading the first 32 bytes: ``02`` (form) ``04`` ``72756c65`` (``"rule"``)
``00000008`` (8 children) ``01`` (atom) ``05`` ``716e616d65`` (``"qname"``)
``0000000b`` (11 payload bytes) ``64656d6f2f68756e676572``
(``"demo/hunger"``).

Digest of this single rule alone:

.. code-block:: text

   SHA-256(CAS(rule)) = 9b49c1e87a63ccb64dee96bd179c8ffecd3fe476c81a4f81771daac237156396

``rules_hash`` for a content set consisting of exactly this one rule — that is,
``SHA-256(0x03 || 0x00000001 || CAS(rule))``, 505 bytes of input:

.. code-block:: text

   rules_hash = bf2f15517d1b4d10e44af36983c76249b49d8fa2dc71e6d8c1d2607b37b2f20f

Static fuel bound for this rule, by §3.7:
``cost(when) = 1 + 1 + 0 = 2``;
``cost(update-node) = 3 + 1 + 0 + (1 + 0) = 5``;
``bound = 7`` ≤ the declared ``:fuel 64``. The rule loads.

Both digests above are reproducible from this document alone; the reference
encoder used to produce them is 90 lines and derives entirely from §5.1–§5.5.

6. Conformance
----------------

6.1 Vector-file format
~~~~~~~~~~~~~~~~~~~~~~~~

Conformance vectors are written in BSL's surface syntax — homoiconic,
diffable, and hashable at the file level — but they are fixtures, not
canonical content (D91 below). A vector file is a sequence of ``vector``
forms:

.. code-block:: text

   <vector>  ::= "(" "vector" <string>
                     ":rule"   <rule>
                     ":env"    "(" <env-entry>* ")"
                     ( ":graph" <graph-lit> )?
                     <outcome>
                     ( ":fuel-used" <int-lit> )?
                     ( ":cas"    <string> )?
                 ")"

   <env-entry> ::= "(" <symbol> <literal> ")"
   <outcome>   ::= ":expect" <literal> | ":expect-error" <symbol>
   <graph-lit> ::= "(" "graph" <node-lit>* <edge-lit>* ")"
   <node-lit>  ::= "(" "node" <symbol> <enum-ref> <field-init>* ")"
   <edge-lit>  ::= "(" "edge" <enum-ref> <symbol> <symbol> <expr> ")"

**[draft ruling — Phase 1 review, post-R9 audit]** *Vector files are
fixtures, not content.* A vector file is read by §1's reader, but no
``vector`` form is canonical content: §5 encodes the **content** forms of
§2, and a ``vector`` form is never encoded under §5 nor hashed into any
content digest — "hashable" above is file identity, the byte hash of the
vector file itself. The flag/valued dichotomy of §1.6 and its closed flag
table (D42) therefore govern content forms only. Within a ``vector`` form,
``:graph``, ``:fuel-used`` and ``:cas`` are **optional vector-format
keywords**, and the collision with ``:graph``'s content classification as a
flag — here it takes a ``<graph-lit>``; there it takes nothing — is a scope
boundary, not an amendment to it. An implementation's canonical encoder never
sees a ``vector`` form; handing it one is a caller error, not a defined
encoding (D91).

**[Director ruling 2026-08-11, ADR191 R5]** *The* ``?`` *binds to the
keyword-and-value GROUP, not to the value.* The production above used to
write ``":graph" <graph-lit>?``, which makes the keyword itself mandatory and
its value optional — the reverse of what the bullets below state
(``:fuel-used`` is mandatory on every non-error vector and therefore absent on
error vectors; ``:cas`` is optional outright) and a shape no reader of this
section would expect. The repair is the grammar's, not the prose's: each of
the three is now written as an optional group, so **the keyword is optional
and its value is mandatory whenever the keyword appears**. A ``:graph``,
``:fuel-used`` or ``:cas`` keyword with no operand is ``E-PARSE-010``, the
existing code for a keyword in value position, exactly as it would be in a
content form. This closes the second of the two repairs §7 deferred to the
Phase-1 review, and §7's appendix carries the same three groups. Recorded as
D95.

Semantics of a vector: load ``:rule`` (the load must succeed unless
``:expect-error`` names a load-class code), hydrate ``:graph`` if present, bind
``:env``, evaluate, and compare.

- ``:expect`` — the rule's condition value, or for effect-bearing vectors the
  resulting field value, must equal the literal **exactly** (bit-exact for
  binary64, integer-exact for fixed point). Conformance is not
  tolerance-bounded; tolerances belong to the *cross-implementation numeric*
  layer described in :doc:`/reference/determinism-contract`, not to the
  evaluator.
- ``:expect-error`` — the named error code must be raised, at the class's
  declared time (load vs evaluation). Raising the right code at the wrong time
  is a failure.
- ``:fuel-used`` — mandatory on every non-error vector. This is the mechanism
  that pins the cost table: changing any base cost changes these numbers, and a
  cost-model revision is therefore a **vector re-bless ceremony**, visible in
  the diff.
- ``:cas`` — optional expected canonical serialization, as lowercase hex. Every
  syntactic construct in §2 must be covered by at least one ``:cas`` vector so
  the byte layout is pinned independently of the evaluator.

Vector files live under a declared conformance root, are sorted by vector id
for execution, and every implementation must run all of them. A vector that no
implementation can satisfy is a spec defect, not a vector defect.

6.2 Required vector families
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

At minimum, an implementation claiming conformance passes:

1. **Lexical** — one accepting and one rejecting vector per ``E-LEX-0xx`` code.
2. **Literal canonicalization** — ``0.50c`` / ``0.5c`` byte-equality;
   ``1000.5$`` / ``1000.500$`` byte-equality; scale and range boundaries.
3. **Currency operators** — every row of §3.2, including half-even ties in both
   directions, the ``i256`` intermediate width for ``Currency ÷ Currency``, and
   both overflow ends.
4. **Kind rule** — the six rows of §3.4's table, accepting and rejecting,
   including the weighted-intensive ``mean``'s **result** kind (D90): the same
   weighted fold nested under an outer ``sum``, which must reject
   ``E-TYPE-041``, and under an outer ``max``, which must accept.
5. **Fuel** — the static bound for a fold at a declared ceiling; a rule
   rejected at load for exceeding its budget; a rule exhausting fuel at
   evaluation; a query against a type the manifest declares no row for
   (``E-LOAD-045``); a node-domain rule fired over three subjects whose
   ``:fuel-used`` is one firing's and not three (§4.5); and per-vector
   ``:fuel-used``.
6. **CAS** — one ``:cas`` vector per form tag and per atom kind, plus the §5.6
   worked example verbatim.
7. **Transcription** — §6.3.
8. **Determinism** — the whole vector set replayed twice in one process and
   once in a fresh process, byte-identical.
9. **Hyperedge** (Amendment D) — the iteration order of ``hyperedges``,
   ``members-of`` and ``hyperedges-of``, including a hyperedge hydrated with
   its members in descending id order to prove declared order is unobservable;
   the ``E-EVAL-032`` type mismatch; ``add-hyperedge`` at exactly
   ``:max-members`` (loads) and one over it (``E-LOAD-042``); a manifest row
   missing ``:max-members`` on a ``HyperedgeType`` and one carrying it on a
   ``NodeType`` (both ``E-LOAD-042``); and a fold over ``members-of`` whose
   static bound equals the declared ``:max-members``.

10. **Edge and hyperedge attributes** (chapter C1) — ``field-of`` over an
    ``EdgeRef``, a ``HyperedgeRef`` and a ``NodeRef``; the §2.4 coverage row
    written as a ``sum`` and as a weighted ``mean`` over
    ``<edge-type>/strength``; ``sum`` over an intensive edge field rejected
    ``E-TYPE-041`` and the same fold accepted with a ``:weight``; a
    ``field-of`` whose referent is of another type (``E-EVAL-033``); a
    ``deffield`` whose first segment names no registered type
    (``E-LOAD-023``); a vocabulary with a ``NodeType``/``EdgeType`` rendering
    collision (``E-LOAD-032``); a re-declaration of
    ``<edge-type>/strength`` (``E-LOAD-001``); an ambiguous foreign-type
    ``:field`` reference under two same-type bodies (``E-TYPE-013``); and an
    ``intrinsic`` declared with a reserved form-head name
    (``E-LOAD-024``).
11. **Edge mutation** (chapter C2) — ``update-edge`` under each of the four
    ``<update-op>`` forms, against ``<edge-type>/strength`` and against a
    ``deffield``-declared edge field; the same write reaching a range boundary
    (``E-EVAL-020``) and an I.15-illegal mode transition (``E-EVAL-030``);
    ``edge-between`` resolving, and failing to resolve (``E-EVAL-034``);
    ``add-edge`` carrying ``<field-init>``\ s, one of them naming ``strength``
    (``E-PARSE-041``) and one owning off the wrong type (``E-TYPE-014``); an
    ``update-edge`` whose referent is of another edge type (``E-EVAL-033``);
    an ``edge-between`` whose enum-ref names a ``NodeType`` (``E-TYPE-011``);
    and a hydration seeding two edges of one type between one ordered pair
    (``E-LOAD-044``).
12. **Graph-scope carriers** (chapter C3) — ``the`` resolving against a
    ``:ceiling 1`` manifest row; ``the`` against a row whose ceiling is not 1
    (``E-LOAD-043``); ``the`` against a graph that hydrated no such node
    (``E-EVAL-035``); a read and a write of one carrier field through
    ``field-of``/``update-node``; ``the`` against an ``EdgeType`` member
    (``E-TYPE-011``); a manifest carrying no row for the carrier type
    (``E-LOAD-045``); and an **accumulation vector** — three subject nodes each
    adding a bounded scalar to one carrier — whose expected value pins the §4.2
    subject order by being sensitive to it, and whose ``:fuel-used`` is the
    first subject's firing (§4.5).
13. **Rule domain** (chapter C4) — an inferred node domain (the §5.6 rule,
    unchanged, is one); a rule with no self-scoped reference and no
    ``<domain>`` (``E-LOAD-004``); a rule whose self-scoped references name two
    node types (``E-LOAD-004``); an explicit ``(domain NodeType/…)`` overriding
    a would-be ambiguity; a rule whose ``<domain>`` and self-scoped reference
    disagree (``E-TYPE-010``); ``(domain :graph)`` firing exactly once against
    a multi-node graph; ``self`` referenced in a graph-domain rule
    (``E-TYPE-015``); ``:graph`` used outside a ``domain`` form
    (``E-PARSE-013``); a ``(domain EdgeType/…)`` (``E-TYPE-011``); and a
    ``:cas`` vector for each of the two ``domain`` shapes.
14. **Element selection** (chapter C5) — ``select-max`` and ``select-min`` over
    each of the six query heads, proving the result's element type; a
    **tie vector** whose two elements score equally, pinning that the lower id
    wins for both operators; selection over an empty query
    (``E-EVAL-021``); a ``Bool`` and an ``Enum<T>`` score (``E-TYPE-016``); a
    selection whose result is the element operand of ``update-node`` and of
    ``field-of``; and a selection over an intensive score, which must
    **accept** (the kind rule polices aggregation, not ordering).
15. **Effect-position iteration** (chapter C6) — ``for-each`` over ``edges``
    applying ``update-edge`` and ``emit`` per element, with the expected
    per-element results in iteration order; ``for-each`` whose query would
    have changed had it seen an earlier verb's effect, proving pre-state
    materialization; nested ``for-each`` and its static bound; ``for-each``
    over an empty query, which applies nothing and **must not** error; and a
    ``for-each`` whose ``:fuel`` is one short of its static bound
    (``E-LOAD-040``).
16. **Computed bindings** (chapter C7) — a ``:expr`` over earlier bindings,
    with ``:fuel-used`` proving the expression is charged once and each
    reference 1; the same rule written with the algebra inlined twice, whose
    ``:fuel-used`` must be strictly larger; a forward reference and a
    self-reference (``E-PARSE-032``); ``:optional`` on a ``:expr``
    (``E-PARSE-033``); ``it`` inside a ``:expr`` (``E-TYPE-012``); a foreign
    node type's ``:field`` referenced from a ``:expr`` (``E-TYPE-010``); and a
    ``:expr`` whose kind is intensive feeding an unweighted ``mean``
    (``E-TYPE-042``), proving kind propagates through the binding.
17. **Typed neighbours and element naming** (chapter C8) — the first
    ``neighbors`` vectors this document has ever required: a fold over
    ``neighbors`` reading the annotated type's field (which must typecheck), a
    graph whose traversal reaches two node types proving the operand *filters*,
    a three-operand ``neighbors`` (``E-PARSE-042``, arity), the two operands
    swapped (``E-TYPE-011`` at both positions), a **multiplicity vector** — a
    graph where two qualifying edges reach one node, whose ``fold count`` over
    ``neighbors`` must be ``1`` — and a static bound
    equal to the **lesser** of the two ceilings; and for ``:as``, a two-hop
    nested fold naming the outer element, ``it`` inside the inner body
    resolving to the *inner* element, a ``:as`` name referenced outside its
    body (``E-TYPE-012``), ``:as it`` and ``:as self`` (``E-PARSE-022``), and a
    ``:as`` colliding with a binding name (``E-PARSE-030``).
18. **Metric registration** (chapter C9) — a graph-domain ``metric`` read by a
    ``:metric`` binding and an element-indexed one read by ``metric-of``; each
    read through the other's form (``E-LOAD-012``); a declaration disagreeing
    with the kernel's registration (``E-LOAD-025``); an unregistered name
    (``E-LOAD-011``, the §6.3 correction, re-proved for both forms); a
    ``metric-of`` against a referent of another type (``E-EVAL-036``) and one
    the provider produced no value for (``E-EVAL-037``); a **stability
    vector** — two rules at one anchor position reading one metric, whose
    expected values must be equal; a metric declared ``:kind intensive``
    feeding an unweighted ``mean`` (``E-TYPE-042``), proving the declared kind
    propagates; and a ``:cas`` vector for the ``metric`` form under both
    ``<domain>`` shapes.
19. **Deliberate absences** (chapter C10) — a family of *rejecting* vectors, so
    the absences of §3.8 are pinned as loudly as the presences: ``(bound? x)``
    (``E-LOAD-021``, an undeclared intrinsic); a string literal in an ``emit``
    payload (``E-PARSE-010`` — the string lexes, the position rejects it); an
    ``(unset …)`` update-op (``E-PARSE-015``); and one *accepting* pair
    proving each re-modelling
    works — a presence-field guard writing value and presence together, a
    ``select-min`` over ``queued-at-tick`` returning the FIFO head, and a
    producer-stamped tick field read by a consumer rule at a later anchor.
20. **Invariant substrate and the lattice** (chapter C11) — a one-hop
    aggregation fold over an ``IN_SCALE`` relation and the ``for-each``
    distribution that mirrors it; ``add-edge`` and ``remove-edge`` naming an
    ``:invariant`` edge type, and ``add-node``/``remove-node`` naming an
    ``:invariant`` node type (all ``E-LOAD-013``); an ``update-node`` field
    write on an invariant type, which must **accept**, proving the flag
    constrains structure and not state; ``:invariant`` on a
    ``HyperedgeType`` row (``E-LOAD-042``); and a hydration that seeds a
    keyed reference series as declared fields against a rule reading them
    with a plain ``:field``, plus the same rule against a hydration that
    omits the series (``E-LOAD-010``).
21. **Hyperedge fields and reference identity** (chapter C12) —
    ``update-hyperedge`` under each ``<update-op>``, and one whose ``<qname>``
    owns off another hyperedge type (``E-EVAL-033``); a roster change proving
    membership is still whole-object replacement; ``=`` and ``!=`` on two
    references of the same kind, both outcomes; a reference compared with
    ``<`` and one compared across kinds (both ``E-TYPE-017``); and the
    intersection idiom above, whose ``:fuel-used`` must show the quadratic
    cost the deferral is paying.
22. **The intrinsic cap and calendar bindings** (chapter C13) — a call to an
    intrinsic outside the declared set (``E-LOAD-021``); an ``intrinsic``
    declaration named ``sigmoid`` (``E-LOAD-024``); ``:year``,
    ``:tick-of-year`` and ``:tick-in-cycle`` at a known tick, with a boundary
    case at each cycle wrap; ``:tick-in-cycle 0`` and a negative length (both
    ``E-PARSE-014``). The retired ``rng-draw`` vectors remain historical
    implementation evidence only; they do not belong to the current language
    acceptance set. **The** ``floor``
    **intrinsic** (ADR188 Row 2, D97) — a zero argument, an exact-integer
    argument, and a fractional argument, proving the round-toward-zero
    result on the ratified ``[0, ∞)`` domain (a wrong-direction
    implementation must fail this row); negative zero, which must accept as
    ``0`` (IEEE-754: ``-0.0 < 0.0`` is false — a sign-bit test rather than a
    value test must fail this row); the largest ``f64`` strictly below
    ``2^63`` (the exact ``i64``-domain accept boundary — a coarsely-mutated
    ceiling constant must fail this row, unlike a boundary vector at a much
    smaller magnitude); a negative argument, a non-finite argument, and an
    argument whose floor reaches or exceeds ``2^63``, all three
    ``E-EVAL-039`` and none of them silently coerced or wrapped; and a bare
    non-``Real`` argument (an ``Int`` literal, not the result of
    arithmetic), refused as a malformed call. **The ADR219 sextet**
    (chapter C13's own host suite, D226-D228) — ``sqrt``: a perfect square
    exact, a non-square pinned to the correctly-rounded bit, ``-0.0``
    accepted with its sign pinned, a negative argument and each non-finite
    argument ``E-EVAL-044``; ``round-half-even``: the tie cases ``2.5 →
    2``, ``3.5 → 4``, ``-2.5 → -2``, ``-0.5 → -0.0`` bit-pinned, non-tie
    nearest, the identity at already-integral magnitude, the ``Real``
    (never ``Int``) return variant, a non-finite argument ``E-EVAL-044``;
    ``min``/``max``: the extreme in both argument orders, the ±0.0 tie
    returning the FIRST argument bit-pinned (never ``f64::min``/\
    ``f64::max``'s implementation-defined pick), a non-finite argument on
    either side ``E-EVAL-044``; ``abs``: the sign dropped, ``-0.0``
    canonicalized to ``+0.0`` bit-pinned, non-finite ``E-EVAL-044``;
    ``clamp``: the in-range identity bit-exact, saturation to each crossed
    bound, the bounds themselves accepted, ``lo > hi`` loud
    (``E-EVAL-044``, never a silent swap), ``lo == hi`` returning the
    bound, non-finite in any position ``E-EVAL-044``; for every one of the
    six, a non-``Real`` argument and a wrong arity refused as malformed
    calls; and the lockstep vector — every ``DECLARABLE_INTRINSICS`` member
    carrying a ``kernel_signature`` row and a dispatch arm, so a cap
    widening that forgets either half fails the suite rather than shipping
    unchecked.

23. **Attributed membership** (Amendment AG (i)) — ``membership-field-of``
    reading a payload field of each declared scalar type, inside a fold over
    ``members-of`` and from an effects list; the read against a well-typed
    pair that is not a membership (``E-EVAL-038``); the read whose hyperedge
    operand is of another ``HyperedgeType`` and the read whose member operand
    is of another ``NodeType`` (both ``E-EVAL-033``); ``update-membership``
    under each of the four ``<update-op>`` forms, one reaching a range
    boundary (``E-EVAL-020``) and one against a non-membership pair
    (``E-EVAL-038``); ``add-hyperedge`` whose annotated ``member`` items
    initialise every declared payload field, one omitting a field and one
    using a bare item under a payload-declaring hyperedge type (both
    ``E-LOAD-047``), one initialising a field twice (``E-PARSE-041``), one
    naming a field owned by another type (``E-TYPE-014``) and one whose member
    is not of the annotated type (``E-EVAL-033``); a hydration omitting a
    payload field (``E-LOAD-047``); a ``field-of`` naming a membership qname
    and a ``membership-field-of`` naming a hyperedge's own field (both
    ``E-LOAD-046``); a ``deffield`` carrying ``:member`` whose first segment
    renders a ``NodeType`` (``E-LOAD-048``) and one whose ``:member`` names an
    ``EdgeType`` member (``E-TYPE-011``); an **ordering vector** — a hyperedge
    hydrated with its members in descending id order, folded with a
    non-commutative-in-binary64 body, pinning that iteration follows D25's
    ascending member id; an intensive payload under an unweighted ``mean``
    (``E-TYPE-042``) and the same fold accepted with a ``:weight``; §2.12's
    two-hop worked shape verbatim, which must **load** — its outer ``max`` is
    kind-neutral where a ``sum`` over the same intensive result would be
    ``E-TYPE-041`` (D90); a roster
    replacement proving a dropped member's payload does not survive it; and a
    fold over ``members-of`` reading payload whose static bound equals the
    declared ``:max-members`` — the ceiling axis unchanged by the payload.

24. **Lattice instances** (Amendment AG (ii)) — a ``manifest`` declaring a
    substrate rung with an extensive and an intensive adjunction along it,
    loading clean, with this section's opening aggregation fold and its
    mirrored ``for-each`` distribution evaluated over the declared rung; a
    ``:substrate`` rung one of whose three types carries no ``:invariant`` row
    (``E-LOAD-049``); an intensive adjunction with no ``:weighted-by``
    (``E-LOAD-050``) and the same declaration accepted with one; a
    ``:weighted-by`` naming an intensive field (``E-TYPE-043``); an adjunction
    citing an undeclared rung, one whose ``<qname>`` is owned by the other
    end's node type, one whose two fields disagree in ``:kind``, and a
    ``:weighted-by`` on an extensive pair (all ``E-LOAD-051``); two rungs and
    two adjunctions sharing a name (``E-LOAD-001``); a rung whose positional
    operand names an ``EdgeType`` and one whose ``:via`` names a ``NodeType``
    (both ``E-TYPE-011``); a rung naming a type the manifest carries no row
    for (``E-LOAD-045``); a **diagnostic-order vector** — a manifest carrying
    two independently malformed declarations, whose reported first failure is
    the lower declared name in byte order; an ``update-node`` field write on a
    substrate rung's node type, which must **accept** (the rung constrains
    structure, not state); and ``:cas`` vectors for ``rung`` and
    ``adjunction`` under both the flagged and unflagged, weighted and
    unweighted shapes.

Families 10–22 are the R9 spec chapters' (the chapter letters cite
``reports/bsl-gap-analysis-2026-08-10.md`` §7); families 23 and 24 are
Amendment AG's, one per clause. Two obligations are stated
once here rather than repeated in every family: each new form tag also owes a
``:cas`` vector under family 6, and each new construct owes its exact
``:fuel-used`` figure under family 5. Both make a chapter's landing a
``rules_hash`` surface change and therefore a **vector re-bless ceremony**,
which the teams should plan rather than discover.

6.3 Transcription contract
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The conformance **seed** is the existing Python evaluator test estate: 271
lines in ``tests/unit/domain/doctrine/test_mechanics.py`` plus 628 lines in
``tests/unit/engine/test_event_evaluator.py`` — 899 lines, verified by
``wc -l`` on 2026-07-29.

Each existing test case transcribes to one or more ``vector`` forms. The
transcription is recorded in a ledger with one row per source test:

.. list-table::
   :header-rows: 1
   :widths: 22 18 20 40

   * - Column
     - Type
     - Example
     - Meaning
   * - ``source``
     - test id
     - ``test_mechanics.py::TestTrapCondition::test_and``
     - The Python test being transcribed.
   * - ``vectors``
     - list of vector ids
     - ``doctrine/and-both-true``
     - The BSL vectors that replace it.
   * - ``verdict``
     - ``preserved`` | ``corrected`` | ``retired``
     - ``preserved``
     - Whether BSL reproduces the observed behavior, deliberately differs, or
       the case ceases to exist (e.g. a test of the ``@`` sigil's tokenizer).
   * - ``note``
     - string
     - —
     - Mandatory when ``verdict`` is ``corrected`` or ``retired``.

**Grammar-superset honesty.** BSL is a superset of the two existing grammars'
expressible sets, **not of their failure semantics**. The design document
(§5) names exactly four silent-degradation behaviors that are deliberately
broken; copied verbatim:

    Four silent-degradation behaviors are deliberately broken as III.11
    corrections: unknown graph metric → 0.0 (``event_evaluator.py:313``),
    unknown aggregation → False (``:439``), unknown comparison operator →
    False (``:405``), empty precondition set → True (``:103``). The ~899 lines
    of existing evaluator tests (271 doctrine + 628 event-evaluator)
    transcribe as the conformance seed **with a documented delta** at exactly
    those four points.

Their BSL dispositions:

.. list-table::
   :header-rows: 1
   :widths: 34 22 44

   * - Python behavior
     - BSL behavior
     - Where enforced
   * - unknown graph metric → ``0.0``
     - ``E-LOAD-011`` at load
     - §2.5 (``:metric`` resolution against the closed registry)
   * - unknown aggregation → ``False``
     - ``E-PARSE-015`` at parse
     - §2.7 (``<fold-op>`` is a closed five-member terminal set)
   * - unknown comparison operator → ``False``
     - ``E-PARSE-015`` at parse
     - §2.4 (``<cmp>`` is a closed six-member terminal set)
   * - empty precondition set → ``True``
     - ``E-PARSE-020`` at parse
     - §2.3 (``(when)`` is not expressible; write ``(when #t)``)

Every transcribed test that asserted one of these four behaviors takes
``verdict = corrected`` with a note naming the row. There is a fifth,
lower-profile delta the transcription must also record: the doctrine DSL's
**absent variable reads as ``0``** (honest-null, ``mechanics.py`` module
docstring) becomes an explicit ``:optional``/``:default 0`` declaration
(§3.5), so transcribed doctrine vectors that relied on it carry
``verdict = corrected`` and their rules carry an allowlisted ``:default``.

6.4 Review gates on the transcription
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Transcription is where the design document's failure classes F1–F4 bite. The
ledger is reviewed against them: eyeball-before-golden (F1 — a vector's
expected value is read by a human before it is blessed, never captured from a
run), tautology ban (F2 — a vector whose expectation is computed by the same
code path it tests is invalid), transcription review with an idiom-mismatch
checklist (F3), and spec-completeness (F4 — every construct §2 defines has at
least one vector).

7. Consolidated grammar
-------------------------

Every production of §§1, 2 and 6.1, collected into one file and included here.
**The grammar collects; it does not amend.** On any divergence between the
§1/§2/§6 **text** and this appendix, the **section text wins**, and the
divergence is a defect in the appendix to be repaired there. Nothing in this
section adds a form, retires one, or changes what one means — an inclusion is
not a second home, which is the property that keeps this document the *one*
normative home its Program-27 charter makes it (D92).

*Text, not merely production.* The trigger is deliberately the section **text**
rather than a section production, because nine of the appendix's productions
have no code block behind them at all — the sections state them in **prose**,
and a grammar needs a right-hand side. They are ``Char`` (§1.1),
``whitespace``, ``comment`` and ``delimiter`` (§1.2, §1.4), ``operator``
(§1.4's draft ruling), ``escape`` (§1.5), ``intrinsic-name`` (§2.7),
``type-name`` (§2.9/§2.11 use it, §3.1 names the types) and ``vector-file``
(§6.1). Under a production-only trigger those nine would adjudicate
themselves — the appendix legislating in exactly the places where no section
production exists to check it against, which is the reverse of what this
section is for.

Three things the file carries that the sections do not. It states a **single
EBNF dialect** — W3C EBNF, the XML 1.0 §6 notation whose operator set §1.4 and
§2.1 already use — and applies it uniformly, so that a reader is never left
inferring which dialect a ``?`` came from. It supplies the nine prose-sourced
right-hand sides above, each carrying its citation. And it collects, in
comments, the **context conditions an EBNF cannot express**: maximal munch
within a token run, the string literal's exemption from it, the closed keyword
set, the reserved element names ``self`` and ``it``, and the whole of §3's
static semantics. Read without them the file **both** accepts programs this
language rejects and rejects programs it accepts — the token-run rule applied
naively to a ``string`` would cut every non-trivial ``:material-basis``
apart — and the file says so at the top rather than leaving either direction
to be discovered.

Where the sections are silent or disagree with themselves, the file **collects
the reference implementation's reading and flags it**. That wording is exact,
and an earlier "records the gap instead of choosing" was not: a production must
have a right-hand side, so writing one *is* choosing, and the honest record
names the reading taken and what cuts against it. Two places arose, and **both
were ruled by the Director on 2026-08-11 (ADR191 R4 and R5); the appendix now
collects a ruling rather than flagging a gap**:

- ``<type-name>`` carried no §1.4 atom class, while §3.1 spelled the type names
  capitalized and **§2.11's own worked example wrote** ``:type Coefficient``.
  No capitalized spelling is lexable under §1.4, so the appendix collected the
  reference implementation's lowercase-``symbol`` reading and recorded the
  worked example that contradicted it. **Ruled: the lowercase reading is
  blessed** — §3.1's six declarable type names and §2.11's example are
  re-spelled lowercase, and §1.4 gains no atom class for capitalized type
  names (D94).
- §6.1's ``:graph`` / ``:fuel-used`` / ``:cas`` were written with the ``?``
  binding to the *value*, which made the keywords themselves mandatory against
  §6.1's own prose. Here the section production existed, so the appendix
  transcribed it verbatim and named the divergence in place. **Ruled: the three
  become optional GROUPS** — keyword optional, value mandatory when the keyword
  appears — in §6.1 and in the appendix together (D95).

*What the appendix does not reach.* The ``.bscn`` scenario-file dialect is
outside its scope — no section specifies it, and D91's fixture/content split
is why — so the shape conflict that follows is **recorded as D93, not resolved
here**.

.. literalinclude:: bsl.ebnf
   :language: bnf
   :caption: ``docs/reference/bsl.ebnf`` — the consolidated grammar

7.1 The rigor index
~~~~~~~~~~~~~~~~~~~~~

Where each artifact of this language's rigor lives. **Pointers only**: every
row's content belongs to the place it names, and this table restates none of
it.

.. list-table::
   :header-rows: 1
   :widths: 24 30 46

   * - Artifact
     - Home
     - What it fixes
   * - Consolidated grammar
     - §7 / ``docs/reference/bsl.ebnf``
     - Every production of §§1, 2 and 6.1 in one stated dialect. Normative
       by inclusion; the section text wins on divergence.
   * - Lexis and syntax
     - §1, §2
     - The productions themselves, with the rulings that shaped each.
   * - Static semantics
     - §3
     - Types (§3.1), the currency lane (§3.2–§3.3), the intensivity kind
       rule (§3.4), binding resolution (§3.5), the closed vocabulary
       (§3.6), deliberate absences (§3.8), the invariant substrate and
       scale lattice (§3.9), the intrinsic cap (§3.10).
   * - Dynamic semantics
     - §4
     - Evaluation order (§4.1), the environment and subject order (§4.2),
       arithmetic (§4.3), query evaluation (§4.4), cross-system handoffs
       (§4.7).
   * - Cost model
     - §3.7 (static bound), §4.5 (runtime meter)
     - ``cost(n)`` per AST node and ``bound(rule)``; pinned by conformance
       vector, so a revision is a re-bless ceremony.
   * - Error-code register
     - §4.6
     - The five families (``E-LEX`` / ``E-PARSE`` / ``E-TYPE`` /
       ``E-LOAD`` / ``E-EVAL``), when each fires, and the contiguity rule
       every decade block is checkable against.
   * - Canonical encoding
     - §5, worked example §5.6
     - The CAS byte layout, canonical child order, and ``rules_hash``.
   * - Conformance vectors
     - §6.1 (format), §6.2 (24 required families), §6.3 (transcription)
     - What an implementation must pass to claim conformance, and the four
       silent degradations deliberately broken.
   * - Decision register
     - *Draft-Ruling Register*, below
     - Every point the design document under-determined, D1–D92, each a
       Phase-1 review item.
   * - Reference implementation
     - ``rust/crates/babylon-bsl``
     - The executable reading: ``reader.rs`` (§1), ``grammar.rs`` (§2's
       static shape rules), ``canonical_ast.rs`` (§5), ``bound_checker.rs``
       (§3.7), ``rule_pipeline.rs`` (the §4.6 class ordering). It is *an*
       implementation, not the spec — III.12(a) requires this document to
       be derivable without reading it.
   * - Editor tooling
     - ``tools/tree-sitter-bsl``
     - A tree-sitter grammar **derived** from ``bsl.ebnf`` and normative
       for nothing, with a corpus drawn from real in-tree content. It
       parses; it does not check §3.
   * - Determinism contract
     - :doc:`/reference/determinism-contract`
     - What ``rules_hash`` is combined with and compared against: the
       tick-hash field set, ``ContentDigest`` composition, and the
       float-tolerance regimes.

Draft-Ruling Register
-----------------------

Every decision this document made where the design document under-determined
the language. Each is a Phase-1 review item.

.. note::
   Amendment AJ supersedes the register's D69 and D176–D179 ``rng-draw``
   decisions. Those rows remain historical rationale only; the ratified finite
   kernel addendum above is the complete current randomness surface.

**Resolved — the Amendment D question (was open question 3 on this document's
first revision).** That question read: *"Amendment D is unratified, so*
``<query>``\ *'s data shape is written against the dyadic working assumption.
If Phase 0 rules hyperedge, §2.6, §2.8 and the edge-side ceiling in §3.7 all
need revision."* Phase 0 ruled: **NATIVE HYPEREDGE**, 2026-07-29, Amendment AE
clause (vi) (``CONSTITUTION.md`` v3.0.0), recording
``ai/_inbox/amendment-d-analysis-p27.md`` §9 (PR #353) sub-rulings D-1…D-7.
All three sections were revised in this document's second revision; the dyadic
forms coexist unchanged (D-2). Rows **D24–D28** below are the new
under-determined points that revision introduced — the question is closed, its
consequences are the ordinary kind of review item.

.. list-table::
   :header-rows: 1
   :widths: 8 30 62

   * - #
     - Section
     - Ruling
   * - D1
     - §1.1
     - String literals must be NFC; non-NFC is a load error.
   * - D2
     - §1.2
     - No block comments, no reader macros.
   * - D3
     - §1.5
     - Kind-suffixed literals (``$``, ``p``, ``i``, ``c``); a bare
       non-integer literal is a load error; max scale 9 for ``p``/``i``/``c``,
       6 for ``$``.
   * - D4
     - §1.5
     - Decimal canonicalization to minimal scale; Currency to integer
       micro-units.
   * - D5
     - §2.3
     - Anchorless rules inherit the system named by their rule-id's first
       segment.
   * - D6
     - §2.4
     - ``it`` is the reserved query-element binding; ``exists``/``forall``
       bind no name.
   * - D7
     - §2.6
     - Query iteration order is ascending id byte order, not storage order.
   * - D8
     - §2.7
     - Arithmetic is strictly binary; no variadic ``+``.
   * - D9
     - §2.9
     - ``:kind`` lives in BSL ``deffield`` content, not in host-language
       annotations.
   * - D10
     - §3.2
     - ``Currency`` subtraction below zero is a loud evaluation error, not a
       clamp — the domain stays ``[0, ∞)``.
   * - D11
     - §3.3
     - Bounded-scalar arithmetic promotes to ``Real``; the range check happens
       once, at the store boundary, and never clamps.
   * - D12
     - §3.4
     - ``:const``/``:metric`` bindings are kind-neutral; ``extensive ×
       extensive`` is a type error. **The** ``:metric`` **clause is superseded
       by D55**: a ``:metric`` binding and a ``metric-of`` accessor carry the
       kind their §2.11 registration declares. The ``:const`` clause and the
       ``extensive × extensive`` rule stand.
   * - D13
     - §3.5
     - ``:optional`` requires ``:default``; there is no ``bound?`` predicate.
   * - D14
     - §3.7
     - Cost rows for ``if``, ``exists``/``forall``, ``query``, structural
       verbs, ``guard``, and the ``bound(rule)`` composition — the five base
       rows are the design document's and are **not** a draft ruling.
   * - D15
     - §3.7
     - ``neighbors`` uses the edge-type ceiling; a per-node degree ceiling is
       a review item.
   * - D16
     - §4.2
     - Rules at one anchor position evaluate in ascending rule-id byte order.
   * - D17
     - §4.3
     - No FMA contraction; non-finite results are unrepresentable.
   * - D18
     - §4.4
     - ``mean``/``min``/``max`` over an empty set are loud errors, not ``0``.
   * - D19
     - §5.2
     - The two-shape (atom/form) length-prefixed binary CAS, big-endian, with
       ASCII tag names rather than a numeric tag registry.
   * - D20
     - §5.2
     - Flag keywords encode as ``opt`` with a ``#t`` value so every option has
       one shape.
   * - D21
     - §5.3
     - Canonical child order: positional, then options sorted by keyword name,
       then variadic body in source order.
   * - D22
     - §5.5
     - ``rules_hash`` covers ``rule`` forms only, sorted by id, with
       ``deffield``/``intrinsic``/``manifest`` hashed into sibling digests that
       ``ContentDigest`` combines.
   * - D23
     - §6.1
     - Conformance vectors are BSL content; ``:fuel-used`` is mandatory on
       non-error vectors.
   * - D24
     - §2.6
     - ``members-of``/``hyperedges-of`` take the ``HyperedgeType`` as a
       mandatory operand — ``HyperedgeRef`` carries no static type, and the
       annotation is what makes ``ceiling(query)`` computable; a mismatch at
       evaluation is ``E-EVAL-032``.
   * - D25
     - §2.6
     - A hyperedge's declared member order is unobservable; ``members-of``
       yields ascending node-id byte order. A member list is a set.
   * - D26
     - §2.8
     - Two typed verbs (``add-hyperedge``/``remove-hyperedge``) rather than an
       overloaded ``add-edge``; membership change is whole-hyperedge
       replacement, so per-membership payload and hyperedge-field mutation
       were both inexpressible when this row was written. **The
       hyperedge-field half is superseded by D65** (``update-hyperedge``); the
       per-membership half was escalated by D66 and is **superseded by
       D79–D84** (Amendment AG (i)). The member-list discipline this row rules
       — whole-object replacement, one ``:max-members`` check, no partial
       roster — is untouched by both.
   * - D27
     - §2.9, §3.7
     - A hyperedge manifest row declares two numbers — ``:ceiling`` and
       ``:max-members`` — mandatory together on a ``HyperedgeType`` row and
       illegal elsewhere (``E-LOAD-042``); ``add-hyperedge``'s member count is
       checked against ``:max-members`` **statically**.
   * - D28
     - §3.7
     - ``hyperedges-of`` uses the hyperedge type's ``:ceiling``; a per-node
       incidence-degree ceiling would be tighter and is deferred alongside
       D15's per-node degree ceiling for ``neighbors``.
   * - D29
     - §2.5, §2.10
     - Edge and hyperedge fields are read by the ``field-of`` accessor, not by
       an edge-typed ``:field`` binding. **Divergence recorded:** the R9 gap
       analysis §2 (Q1) sketched the binding form; a binding resolves
       implicitly against an enclosing body and the demanding systems read
       several edge types per rule, so the annotated accessor — D24's fix for
       ``members-of`` — wins.
   * - D30
     - §2.5
     - A foreign-node-type ``:field`` reference under two or more enclosing
       bodies of that type is ``E-TYPE-013``; the author names an element
       (§2.6 ``:as``) and uses ``field-of``.
   * - D31
     - §2.9
     - A ``deffield``'s first segment may name a ``NodeType``, ``EdgeType`` or
       ``HyperedgeType`` member. The segment↔member rendering (lowercase,
       ``_``→``-``) is stated normatively; renderings must be pairwise
       disjoint across the three enum types (``E-LOAD-032``), must be valid
       ``symbol``\ s (``E-LOAD-033``), and an unregistered first segment is
       ``E-LOAD-023``.
   * - D32
     - §2.9
     - ``<edge-type>/strength`` is implicitly declared on every ``EdgeType``:
       ``Coefficient``, ``extensive``, re-declaration is ``E-LOAD-001``. The
       ``extensive`` kind is what makes §2.4's ``sum_strength`` row honourable
       under §3.4 without an exemption.
   * - D33
     - §2.7
     - Every §5.2 form-head symbol is reserved against the intrinsic
       namespace; declaring an intrinsic with one is ``E-LOAD-024``.
   * - D34
     - §2.10
     - Accessors take their owning type in the ``<qname>``; a referent of
       another type, or a field the element carries no value for, is
       ``E-EVAL-033``. ``:optional``/``:default`` are binding options and
       never apply to an accessor.
   * - D35
     - §2.8
     - ``update-edge`` exists. D26's whole-object discipline is specific to the
       member list it protects and does **not** carry to the dyadic layer,
       which has no partial state to leave behind.
   * - D36
     - §2.8, §2.10
     - ``update-edge`` takes an ``EdgeRef`` and has one shape, not two;
       endpoint-holding rules reach the edge through ``edge-between``, whose
       well-definedness follows from §2.6's ``(source, target, type)`` order
       key. Absence is ``E-EVAL-034``. **Divergence recorded:** the R9 gap
       analysis §2 (Q3) sketched a type-and-endpoints verb.
   * - D37
     - §2.8
     - ``add-edge`` carries ``<field-init>*``; a ``<field-init>`` naming the
       implicit ``strength`` field is ``E-PARSE-041``, and one whose owning
       type is not the verb's ``<enum-ref>`` type is ``E-TYPE-014``
       (statically on the minting verbs, ``E-EVAL-033`` on the updating ones).
   * - D38
     - §3.7
     - Accessors are keyed lookups charged at 1 + operands and never
       multiplied by a ceiling, so only iteration constructs carry ceiling
       factors in the static bound.
   * - D39
     - §3.6
     - Graph-scope state is ordinary node state on a carrier ``NodeType``
       whose ceiling is 1 — no new grammar, no second storage class. Adding a
       carrier type is amendment territory. The ``:global``/``update-global``
       route is recorded as rejected.
   * - D40
     - §2.10
     - ``the`` names the unique node of a ``:ceiling 1`` type; a ceiling other
       than 1 is ``E-LOAD-043`` (static, off the manifest) and an unhydrated
       carrier is ``E-EVAL-035``.
   * - D41
     - §4.7
     - No staging, double-buffering or ``previous`` construct: tick ordering
       is the handoff mechanism, and the anchor is where a one-tick lag is
       declared.
   * - D42
     - §1.6
     - ``:out``/``:in``/``:any`` are flag keywords the table had omitted;
       ``:graph`` joins them, legal only in a ``domain`` form.
   * - D43
     - §2.3
     - ``<domain>`` is optional with a stated inference (the unique node type
       of the rule's self-scoped references); ``|U| ≠ 1`` is ``E-LOAD-004``.
       ``(domain :graph)`` fires once per tick and unbinds ``self``
       (``E-TYPE-015``). Explicit declarations override inference; a
       disagreeing self-scoped reference is ``E-TYPE-010``.
   * - D44
     - §4.2
     - A node-domain rule fires over its subjects in ascending node-id byte
       order and applies their effects in that order — previously
       unspecified, and load-bearing because the binary64 lane is not
       associative.
   * - D45
     - §2.7
     - ``select-max``/``select-min`` return the extremising **element**; ties
       break to the first element in §2.6 iteration order, making the
       deterministic tiebreak a property of the language rather than of each
       rule. An empty query is ``E-EVAL-021``.
   * - D46
     - §2.7
     - The score expression must be a comparable scalar (``E-TYPE-016``); its
       kind is unconstrained and the result is kind-neutral, because §3.4
       polices aggregation and selection orders rather than aggregates.
   * - D47
     - §2.8
     - ``for-each`` is bounded iteration in effect position, charged like
       ``exists``/``forall``. ``it`` is bound in its body.
   * - D48
     - §2.8
     - Every expression in an effects list — verb operands, ``guard``
       conditions, ``for-each`` queries — is evaluated against the rule's
       pre-state, which is what keeps §4.2's no-self-observation law true in
       the presence of iteration. Application order is element order (outer)
       then source order (inner), and an empty ``for-each`` query applies
       nothing rather than erroring.
   * - D49
     - §2.5
     - ``:expr`` computed bindings, resolved in declaration order; forward and
       self references are ``E-PARSE-032``, so no cycle analysis is needed.
       ``:optional``/``:default`` on a ``:expr`` is ``E-PARSE-033``.
   * - D50
     - §3.7, §4.5
     - A ``:expr`` is charged once at binding time and 1 per reference, and
       ``bound(rule)`` gains ``Σ cost(:expr bindings)``. A computed binding
       cannot observe the rule's own effects — it is an abbreviation, not a
       sequencing construct.
   * - D51
     - §2.6
     - ``neighbors`` takes a mandatory result-``NodeType`` operand — D24's fix
       applied to D24's problem. It **filters** (a node may reach several
       types) where D24's operand **asserts**. Breaking, and made anyway: no
       conformance vector and no content rule exercises ``neighbors``, so
       nothing is re-blessed, while the ``babylon-bsl`` bound checker *does*
       carry the three-operand form and its edge-type-only bound and is
       updated when Phase 1 implements the chapter.
   * - D52
     - §3.7
     - ``ceiling(neighbors)`` is the lesser of the edge-type and
       result-node-type ceilings, revising D15's edge-type-only reading; the
       per-node degree ceiling stays deferred.
   * - D53
     - §2.5, §2.6
     - ``it`` denotes the element of the **innermost** enclosing iterating
       form. This is rebinding by construction, not shadowing of a
       declaration, so ``E-PARSE-022`` is untouched — the §2.5/§3.7 tension is
       resolved by reading rather than by repeal.
   * - D54
     - §2.6
     - ``:as`` optionally names an iterating form's element, in scope through
       nested bodies, sharing the rule's binding namespace
       (``E-PARSE-030``/``E-PARSE-022``) and ``E-TYPE-012`` outside its body.
       Naming was chosen over rebinding rules because it changes the meaning
       of no existing form.
   * - D55
     - §2.11
     - A ``metric`` form declares what the kernel provides — type, kind,
       domain, provider — so the typechecker and fuel checker stay derivable
       from content (D9's rationale). Kernel disagreement is ``E-LOAD-025``.
       Supersedes D12's clause making ``:metric`` bindings kind-neutral: they
       now carry the declared kind.
   * - D56
     - §2.10, §2.11
     - Element-indexed metrics are read by the ``metric-of`` **accessor**, not
       by a ``:metric-of`` bind-src. **Divergence recorded:** the R9 gap
       analysis §3 (B4) sketched the bind-src; a bind-src encodes as a
       two-child ``opt`` under D20 and this one needs two operands, so it
       would have been the only bind-src of its shape. Wrong-form reads are
       ``E-LOAD-012``; runtime failures are ``E-EVAL-036``/``E-EVAL-037``.
   * - D57
     - §2.11
     - Provider work is **not** metered against the reading rule (the read
       costs 1 + operand); metric *values* enter no content hash and reach the
       tick hash only through the fields rules write from them; a Rust domain
       crate qualifies as a provider only inside the determinism contract's
       pinned toolchain and with golden vectors.
   * - D58
     - §3.8
     - Optional axes take a **companion presence field** written under one
       ``guard`` with their value. ``:optional``/``:default`` is explicitly
       *not* the mechanism, because a default converts "never seeded" into
       "seeded with zero" and changes the eligibility population.
   * - D59
     - §3.8
     - No sequence or map type. FIFO agendas become a ``NodeType`` plus
       ``select-min``; ordered acquisitions become an edge plus
       ``select-max``; name sets become one ``Bool`` per name.
   * - D60
     - §3.8
     - No same-tick event-history query. The emitting rule stamps a field and
       the consumer reads it, which makes the dependency content rather than
       trace — and makes a crisis-gated system unportable apart from its
       producer.
   * - D61
     - §3.8
     - Four standing "nothing to add" rulings recorded so they stop
       returning: no string payloads on ``emit``; no ledger/receipt binding
       (a receipt is a kernel observation); no cascade restatement in the verb
       table (ADR185 R2 owns it); no bounded numeric iteration, and no
       bespoke ``renormalize`` intrinsic in its place.
   * - D62
     - §3.9
     - No ``group-by`` and no keyed collection. A scale lattice is graph
       content — carrier types plus a typed membership relation — so
       aggregation is a one-hop fold and distribution a ``for-each``. The
       no-``group-by`` clause stands. The minting clause — *"minting a rung or
       an adjunction is amendment territory and outside this document"* — is
       **superseded by D85–D89**: Amendment AG (ii) opened rung and
       adjunction-*instance* declaration to content, and the amendment
       territory that remains is the schema itself (new adjunction kinds,
       altered conservation, new level-lattice algebra).
   * - D63
     - §2.9, §3.9
     - ``:invariant`` on a ``NodeType``/``EdgeType`` ``ceiling`` row makes
       ``add-*``/``remove-*`` naming that type ``E-LOAD-013``, statically.
       Field writes are unaffected: the flag constrains structure, not state.
   * - D64
     - §3.9
     - No ``:reference`` bind-src. Keyed reference series are materialised as
       declared node fields at hydration and read with an ordinary ``:field``
       (ADR174's boundary), which makes the hydration contract a blocking
       dependency rather than a source of zeros.
   * - D65
     - §2.8
     - ``update-hyperedge`` closes the second half of D26 on C2's reasoning:
       writing a declared field leaves no member list partially anything.
       Membership change stays whole-object replacement, so D26's actual
       guarantee is untouched.
   * - D66
     - §2.8
     - **Not ruled here** *(when written)*. Per-membership payload — D26's
       first half — is a missing *kind of object*, not a missing verb, and
       changing the exposed hyperedge model is Amendment-AE-(vi) territory.
       Recorded as a **port blocker** with its two candidate landings named
       and neither specced. **Ruled since:** Amendment AG clause (i)
       (``CONSTITUTION.md`` v3.2.0, ADR189, 2026-08-10) took the first of the
       two landings and rejected the second in its own words. **Superseded by
       D79–D84**; the port blocker is discharged.
   * - D67
     - §2.4, §3.1, §2.7
     - References compare by identity with ``=``/``!=`` only
       (``E-TYPE-017``); there is no ordering on references. With that and
       D54's naming, the intersection idiom becomes writable, and the
       deferral of a dedicated set-algebra operator stands on a form that can
       actually be written.
   * - D68
     - §2.5
     - Calendar reads land as ``:year``/``:tick-of-year``/``:tick-in-cycle``
       bindings rather than as ``mod``/``floor-div`` operators or an intrinsic
       rider — a kernel seam, not mathematics. The reach is bounded rather than
       nil, and §2.5 states the bound precisely: ``:tick-in-cycle`` makes
       ``tick mod k`` available for a **literal** ``k`` only, on the tick only,
       with the epoch staying the kernel's — so what cannot arrive behind it is
       a general mod operator over arbitrary expressions.
   * - D69
     - §3.10
     - The RNG carrier key is ``(session, tick, domain, stable_key)``, with
       ``session``/``tick`` never operands and ``domain`` a closed-vocabulary
       member. **A draw is a pure function of its key, not a stream
       position**, so a skipped draw cannot shift any other and §4.1's
       short-circuiting can never perturb the RNG. **The ``domain``-operand-
       shape clause above (enum operand) is superseded by D176** (#576 Task
       6, landed ``rng-draw``) — ``domain`` is the firing rule's own id
       string, not a declared enum operand. **This row's purity clause — "a
       draw is a pure function of its key, not a stream position" — is
       untouched and survives verbatim in D176's own text.**
   * - D70
     - §2.7, §3.10
     - The cap's authority chain is recorded as it actually reads: R10 holds
       ``{exp, log}`` citing ADR176 r21, whose own text pins the *mechanism*
       rather than the membership. §2.7's transcendental list is illustrative
       of the class, not a table of intrinsics that exist. The rider slate is
       recorded as **proposals**; this document declares none of them.
   * - D71
     - §3.10
     - ``sigmoid`` is a reserved **prohibited** intrinsic name
       (``E-LOAD-024``). Cap-legality is not doctrine-legality: the two gates
       are separate, gate 2 is Director review, and this is the one part of it
       that can be made mechanical.
   * - D72
     - §2.6, §3.1
     - A query result is a **set**: ``neighbors`` yields a node reachable by
       several qualifying edges exactly once, and a fold over it counts and
       sums per node rather than per edge. **Alternative rejected:** the
       multiset reading (one element per traversed edge), because a duplicated
       id has no defined position in §2.6's ascending-id order, so two
       conforming implementations could differ on a ``count`` and therefore on
       a tick hash. Per-edge work folds over ``edges`` or iterates it with
       ``for-each``.
   * - D73
     - §2.6, §2.10, §3.9
     - A hydration seeding two edges with one ``(source-id, target-id,
       edge-type)`` triple is ``E-LOAD-044``. This is what makes the triple a
       key rather than a sort field, and it is the clause §2.6's total order
       and ``edge-between``'s well-definedness were both resting on without
       citing: the pre-existing citations covered the verb (``E-EVAL-031``)
       and the ceiling (``E-LOAD-041``) but not hydration.
   * - D74
     - §2.3, §2.6, §2.8, §2.10
     - ``E-TYPE-011`` is stated once as a class rule covering **every**
       ``<enum-ref>`` operand position — ``NodeType`` for ``nodes``,
       ``neighbors``' fourth operand, ``the`` and ``(domain <enum-ref>)``;
       ``EdgeType`` for ``edges``, ``neighbors``' second operand and
       ``edge-between``; ``HyperedgeType`` for the three hyperedge queries;
       ``EventType`` for ``emit``. The R9 chapters added four positions and
       the per-form phrasing had left each of them without a rejection.
   * - D75
     - §1.6, §2.7, §4.6
     - Error-code hygiene, ruled once. Arity gets ``E-PARSE-042`` (with
       ``E-PARSE-040`` as its arithmetic-specific spelling); an unrecognized
       member of a closed terminal set gets ``E-PARSE-015``; a string literal
       in expression position is ``E-PARSE-010``, the existing
       atom-in-the-wrong-position code. The three ``E-PARSE-0xx`` placeholders
       in §6.2/§6.3 are retired, and the two ``E-TYPE`` holes the chapters left
       are closed by renumbering the offending **new** codes — a liberty that
       exists only before an implementation pins them and is spent here.
   * - D76
     - §2.9, §3.7
     - A manifest owes a ``ceiling`` row for every type the content set
       queries, mutates, reaches with ``the`` or (per D85) names in a
       ``rung``; an omission is
       ``E-LOAD-045``. Without the row ``ceiling(query)`` is not computable,
       ``E-LOAD-043``'s "other than 1" test cannot fire on a missing row, and
       ``:invariant``'s check silently never runs.
   * - D77
     - §4.5, §6.1
     - The fuel meter is **per firing**: each subject of a node-domain rule
       starts a fresh meter at the declared ``:fuel``, which is the only
       reading consistent with ``bound(rule)`` carrying no subject-count
       factor and with load-time legality being independent of graph size. A
       vector's ``:fuel-used`` is one firing's, reported for the first subject
       in §4.2's order.
   * - D78
     - §3.8
     - **Not ruled here.** Edge-endpoint accessors (gap item Q2) are recorded
       as an *open* item rather than a deliberate absence: the chapter plan
       assigned Q2 to no chapter, the ``self``-anchored
       ``neighbors``/``edge-between`` idiom covers only one endpoint-known
       case, and a rule iterating ``edges`` cannot name endpoints at all. The
       two landings — a ``source-of``/``target-of`` pair in §2.10, or leaving
       the systems on the idiom — are named and neither is specced; recorded
       as a **port blocker**. It stays open and unspecced: Amendment AG does
       not reach it, and since AG discharged D66's blocker this is the only
       one this document still carries.
   * - D79
     - §2.9, §2.12
     - A membership payload field is declared by ``deffield`` with a
       ``:member`` ``NodeType`` operand, the ``<qname>``'s first segment
       rendering the owning ``HyperedgeType`` — **not** by a new top-level
       form. Two reasons from this document's machinery: a new top-form would
       owe a new sibling digest and therefore an edit to
       :doc:`/reference/determinism-contract`'s ``ContentDigest`` composition,
       and a keyword encodes as an ``opt`` under D20 so nothing needs a new
       form tag or atom kind. One shared field namespace (duplicate =
       ``E-LOAD-001``); wrong-form read or write = ``E-LOAD-046``;
       ``:member`` on a non-hyperedge owner = ``E-LOAD-048``. One declaration
       is one *(HyperedgeType, NodeType)* pair, which is the price of keeping
       ``<qname>`` → declaration total and both checks static.
   * - D80
     - §2.12, §3.1, §2.6
     - **No fourth reference kind and no** ``memberships-of`` **query.** A
       membership is denoted by its key *(hyperedge, member)*, as a dyadic
       edge is denoted by its triple. **Alternative rejected:** a
       ``MembershipRef`` type would have dragged in a set type, a query head,
       and an ordering key for an object with no id — whose only honest order
       is the member list's, which the keyed forms already give — buying no
       expressiveness at the price of four obligations, in a chapter whose
       amendment re-seals the closure (AG (iii)).
   * - D81
     - §2.10
     - ``membership-field-of`` reads payload from two element operands plus an
       annotating ``<qname>`` — D24/D29's pattern, hyperedge first and member
       second so that one operand order serves ``members-of``, the accessor
       and the verb. **Alternative rejected:** a ``:membership`` bind-src, on
       D56's reasoning verbatim (a bind-src is a two-child ``opt``; this needs
       two element operands; and a binding resolves implicitly where this
       reference is explicit). A wrong-type referent at either operand is
       ``E-EVAL-033``; a well-typed non-membership pair is ``E-EVAL-038``; the
       accessor is a keyed lookup charged at 1 + operands (D38).
   * - D82
     - §2.8
     - ``update-membership`` writes payload of an **existing** membership,
       mirroring the other three update verbs and inheriting the range and
       I.15 disciplines. The member list stays whole-object replacement
       (D26), so ``:max-members`` keeps its single check point and VIII.9
       survives verbatim. AG (iii)'s "adds no verb" is read as the NORTH_STAR
       §0 / Article V closure list, against AG (i)'s "mutate only through
       effects" and ADR189 (iv)'s "accessor/verb surface" — the effect-position
       write is required by the amendment, not licensed against it.
       **Living-authority supersession (AH).** The text above preserves a
       pre-AH v3.2.0 historical citation. ADR221 ``V.Player`` assigns
       player-vocabulary authority to
       ``ai/decisions/ADR177_verb_matrix_ratified_main_ruleset.yaml``,
       ``src/babylon/game/actions/matrix.py``, and
       ``src/babylon/game/actions/registry.py``. The supersession changes
       authority only. D82's technical ruling applies.
   * - D83
     - §2.8, §3.9
     - Mint-time payload initialisation is **total and annotated**:
       ``(member <enum-ref> <expr> <field-init>*)``, whose field-inits are
       exactly the declared payload of that pair. The annotation supplies the
       member type §3.1 gives no reference, making completeness **static**
       (``E-LOAD-047``, at mint and at hydration alike); duplicate init is
       ``E-PARSE-041`` and a foreign qname ``E-TYPE-014``. Bare member items
       stay legal for payload-free hyperedge types, so no existing form
       changes meaning. **Alternative rejected:** partial mint with a first
       read failing ``E-EVAL-033``, because it converts a decidable authoring
       error into a runtime one.
   * - D84
     - §2.12, §3.4, §3.7
     - What the kind carries: payload types and kinds exactly as node/edge
       fields do (an intensive payload under an unweighted ``mean`` is
       ``E-TYPE-042``); memberships iterate in the member list's ruled order
       (D25, ascending member node id), introducing no new order;
       ``:max-members`` stays the only cardinality axis and the fuel bound
       stays ``Σ|members|``; payload is element **state** of the same standing
       as the other three kinds' fields, this document deliberately not
       restating the tick-hash field set that
       :doc:`/reference/determinism-contract` owns.
   * - D85
     - §2.9, §3.9
     - Scale-lattice **rungs** and ``allocate``/``aggregate`` **instances**
       are declared as ``manifest`` children (``rung``, ``adjunction``), not
       as top-level forms. **The reason is D79's verbatim**: §5.5 gives
       ``deffield``/``intrinsic``/``manifest``/``metric`` sibling digests that
       ``ContentDigest`` combines, and ``ContentDigest``'s composition belongs
       to :doc:`/reference/determinism-contract`, so a new top-form would owe
       a new sibling digest and an edit to a document this one must not reach
       into; a child of an existing form owes neither. The secondary reason is
       narrower than an earlier draft of this row claimed: **one** of the two
       standing rulings — the substrate one — is checked against
       ``:invariant`` ``ceiling`` rows, which are children of the very form a
       rung joins, so that check has a manifest referent and stays local to
       it. The weighting ruling is checked against ``deffield`` kinds and
       spans two digests, as any content-wide check does; §3.9's body states
       the split correctly and this row now matches it. Names are content-set
       unique (``E-LOAD-001``); a rung's types owe ceiling rows
       (``E-LOAD-045``); enum-ref kinds are ``E-TYPE-011`` under the class
       rule.
   * - D86
     - §3.9
     - One orientation convention serves both forms: a ``rung``'s positional
       operands are **finer first, coarser second**, its ``:via`` relation is
       directed finer → coarser, and an ``adjunction``'s two ``<qname>``\ s
       are the fields at those ends **in the rung's order**. That is what
       makes the section's ``neighbors`` reads unambiguous without a second
       annotation.
   * - D87
     - §3.9
     - ``:substrate`` obliges a rung's finer type, coarser type and ``:via``
       type each to carry ``:invariant`` (``E-LOAD-049``), which is how the
       Director's 2026-07-30 spatial-adjacency ruling becomes a property of
       the language: ``:invariant`` (D63) bars the structural verbs and
       hydration clause 3 makes hydration the only writer, so a substrate rung
       is built once from the static lookup estate and no rule rewires it. The
       flag is declared rather than inferred from the three rows, for the
       reason ``:invariant`` is itself declared.
   * - D88
     - §3.9, §3.4
     - An ``adjunction`` over an intensive field pair carries
       ``:weighted-by`` or it is ``E-LOAD-050`` — the amendment's
       intensive-aggregation rule as a load error, catching the **lattice
       that specifies** an aggregation where §3.4's ``E-TYPE-042`` catches the
       **fold that implements** one. A non-extensive weight is ``E-TYPE-043``,
       the existing code. The two checks are not redundant: a declared
       adjunction exists before any rule realises it.
   * - D89
     - §3.9, §3.7
     - An ``adjunction`` **declares, it does not execute**: no verb, no
       evaluator behaviour, no cost row (declarations are manifest-class and
       unmetered), and the grammar gives conservation **no place to be
       restated** — which is how "instances, not kinds" is enforced by
       construction. ``E-LOAD-051`` is the single fit-to-schema code
       (undeclared rung, foreign field owner, kind disagreement, misplaced
       weight); validation and diagnostics run in ascending declared-name byte
       order so two implementations report the same first failure.
   * - D90
     - §3.4, §2.12
     - A **weighted** ``mean`` over an intensive body has result kind
       **intensive** — the one cell §3.4's table left blank while stating a
       result for its four other rows. Unit algebra (``Σ(w × x) / Σ(w)`` is in
       the units of ``x``), not new mathematics, and stated in the table
       because the ``*``/``/`` bullet deliberately rejects the decomposition
       (extensive ÷ extensive is ``E-TYPE-040``). Recorded on adversarial
       verification of the AG sections, which caught §2.12's worked shape
       folding ``sum`` over exactly this value — ``E-TYPE-041``, and the
       recorded variance error itself; the example now folds ``max``, whose
       row is kind-neutral over any body.
   * - D91
     - §6.1, §1.6
     - Scope split between the vector-file format and canonical content:
       ``vector`` forms are reader-parsed **fixtures**, never encoded under
       §5 or hashed into a content digest, so §1.6's closed flag table
       (D42) governs content forms only, and ``:graph <graph-lit>?`` in a
       vector form — optional-valued, a shape the content dichotomy has no
       room for — collides with but does not amend ``:graph``'s flag
       classification. Surfaced by the PR #480 re-verification: the
       implementation's closed flag table would mis-encode a ``vector``
       form if one ever reached the canonical encoder, so the encoder now
       has license to treat that as a caller error. Re-spelling the vector
       keywords was rejected as churn in a fixture format §5 never touches.
   * - D92
     - §7, §1.1, §1.2, §1.4, §2.1, §2.11, §6.1
     - **The grammar is consolidated as an INCLUDED ASSET, not a second
       home.** ``docs/reference/bsl.ebnf`` collects every production of §§1,
       2 and 6.1 and is part of this document by ``literalinclude`` (§7); it
       **collects and does not amend**, so on any divergence the §1/§2/§6
       text wins and the appendix is the defect. The precedence trigger is
       the section **text**, not merely a section production: **nine**
       productions are collected from PROSE rather than from a code block
       (``Char``, ``whitespace``, ``comment``, ``delimiter``, ``operator``,
       ``escape``, ``intrinsic-name``, ``type-name``, ``vector-file``), and
       a production-only trigger would leave exactly those nine adjudicating
       themselves. The dialect is **W3C EBNF**
       (the XML 1.0 §6 notation), chosen because §1.4 and §2.1 already write
       their productions in its operator set — collecting them is
       transcription, where ISO 14977 would have meant translating a
       normative text into comma-concatenation and ``{ }`` repetition:
       transcription risk bought for nothing. Two notation-level deviations
       are stated in the file and are spellings rather than changes —
       nonterminals lose §2's angle brackets, and §1.4's ``"0" … "9"``
       ranges become W3C character classes. **Two under-determined points
       are collected with the reference implementation's reading and
       flagged** — not "recorded instead of chosen", since a production
       needs a right-hand side and writing one is a choice: ``<type-name>``
       has no §1.4 atom class and no capitalized spelling is lexable, so the
       lowercase-``symbol`` reading is taken and **§2.11's own worked
       example** (``:type Coefficient``) is recorded as cutting against it;
       and §6.1's ``:graph`` / ``:fuel-used`` / ``:cas`` are written with
       ``?`` binding to the value, which makes the keywords themselves
       mandatory against §6.1's own prose, so the production is transcribed
       verbatim with the divergence named. Both repairs belong to the
       Phase-1 review, not to an appendix. A derived, non-normative
       tree-sitter grammar lives at ``tools/tree-sitter-bsl``; a sync guard
       (``tests/unit/reference/test_bsl_grammar_sync.py``) requires every
       §5.2 form tag and every ``::=`` production of §§1/2/6.1 to appear in
       the appendix, so the collection cannot silently fall behind the
       sections it collects.
   * - D93
     - §7, §2.9, §6.1
     - **The** ``.bscn`` **scenario dialect is out of the appendix's scope,
       and its** ``deffield`` **is a DIFFERENT form sharing a name with
       §2.9's.** The Rust reference implementation's scenario loader
       (``rust/crates/babylon-bsl/src/scenario.rs``) reads
       ``(scenario …)`` files containing ``(node …)``, ``(edge …)`` and
       ``(deffield <qname> <type-symbol> <kind-symbol>)`` — the last one
       **positional**, where §2.9's ``deffield`` takes ``:type`` and
       ``:kind`` keyword options. No section of this document specifies any
       of those forms. The scope ruling follows D91's fixture/content split
       rather than inventing a second one: a scenario file is a **fixture**
       that hydrates a world (§3.9's hydration contract governs what it may
       do), not canonical content: §5 never encodes it and §7's appendix
       never collects it. **This row records the shape conflict;
       it does not resolve it.** Two repairs are available and both are the
       Director's: rename the scenario form so one name does not denote two
       shapes, or give the scenario format a chapter of its own here. Until
       one is chosen, the conflict is written down in a normative place
       rather than living only in ``tools/tree-sitter-bsl``, which this
       document declares normative for nothing. Recorded on adversarial
       verification of PR #485.
   * - D94
     - §3.1, §2.11, §2.9, §7
     - **A** ``<type-name>`` **is a lowercase** ``symbol`` **— the first of
       D92's two deferred repairs, ruled** (Director, 2026-08-11, ADR191 R4).
       The two landings D92 named were "§1.4 gains an atom class for
       capitalized type names" and "§3.1's table and §2.11's example are
       re-spelled lowercase"; the second is taken, so the language mints no
       lexical class for one construct and the reference implementation's
       reading (``declarations.rs::parse_type_name``, which accepts ``int`` /
       ``bool`` / ``currency`` / ``probability`` / ``intensity`` /
       ``coefficient``) becomes the spec's. The re-spelling reaches every
       ``<type-name>`` **position** — §3.1's six declarable rows, §2.11's
       worked example, and §2.9's ``strength`` ruling and membership-payload
       example. It does **not** reach §3.1's other rows: ``Real``,
       ``Enum<T>``, the reference and set kinds and ``Str`` are typechecker
       types no ``<type-name>`` position can name, so nothing lexes them and
       nothing needed re-spelling. ``bsl.ebnf``'s ``type-name`` production
       stops recording a gap and cites this row; the derived tree-sitter
       grammar already read the nonterminal this way.
   * - D95
     - §6.1, §7
     - **§6.1's vector keywords are optional GROUPS, not optional values — the
       second of D92's two deferred repairs, ruled** (Director, 2026-08-11,
       ADR191 R5). ``":graph" <graph-lit>?`` becomes
       ``( ":graph" <graph-lit> )?``, and likewise for ``:fuel-used`` and
       ``:cas``: the keyword is optional, its value is mandatory whenever the
       keyword appears, and a bare keyword is ``E-PARSE-010``. This makes the
       production say what §6.1's own bullets already said (``:fuel-used``
       mandatory on non-error vectors, ``:cas`` optional outright) rather than
       changing either, so no vector in the estate moves. The appendix is
       updated in the same commit as the section, which the §7 precedence rule
       and the sync guard both require. D91's scope ruling is untouched and
       still carries: a ``vector`` form remains a fixture, never encoded under
       §5, and ``:graph``'s content classification as a flag still collides
       with its valued use here — what D95 removes is only the
       optional-*valued* shape D91 described in passing.
   * - D96
     - §2.6, §3.9
     - **The tick hash is NOT invariant under a reordering of a scenario's
       declarations, and §2.6's order guarantee is weakened to what it
       delivers** (Director, 2026-08-11, ADR191 R2). The sentence promised
       fold results "independent of insertion history and of the underlying
       graph library"; ``NodeId`` is a monotonic mint counter and a scenario's
       nodes are minted top to bottom, so the first half was undeliverable —
       shuffling two ``node`` forms permutes the ids, the iteration order and
       the hash. The measured finding behind it is the storage-swap capability
       delta's CD5: under a monotone-mint bimap, ascending-``NodeId`` order
       *is* insertion order, so the sort satisfied the letter of the substrate
       contract while delivering none of this sentence's stated purpose. Of
       the two available repairs — weaken the sentence, or make identity a
       deterministic function of the stable domain id (a far larger ruling
       touching III.7 and III.12) — the Director took the first, on the ground
       that **a scenario is a canonical committed artifact whose declaration
       order is part of its identity**: a reordered scenario is a different
       scenario. What survives is the property that defeats the cross-language
       trap — a total order, identical in every conforming implementation,
       over whatever ids the scenario minted, independent of the storage
       library. The consequence for the conformance estate is stated so it
       cannot be re-derived wrongly: **no test may assert that shuffling a
       scenario's declarations leaves the tick hash unchanged**; what a vector
       must still discriminate is sorted-versus-storage order, which needs an
       id case crossing a decade boundary and a declared order differing from
       id order.
   * - D97
     - §3.10, §4.3, §6.2
     - **The** ``floor`` **intrinsic lands, under ADR188 Row 2** (the
       intrinsic-cap rider slate, Director-disposed 2026-08-10, transcribing
       the "i approve all" ruling row by row) **and ADR191 R3's consequence
       note** (2026-08-11: the mortality family's mechanical half — the
       Real→Int demotion — rides this rider, discharging the Grinding
       Attrition port's doctrinal block; landing the spec text is this row's
       job, not R3's). ADR188 Row 2 ratifies *that* a Real→Int demotion path
       enters the intrinsic table — its own text pairs the candidate name
       ``floor``/``trunc`` and does not choose between them, and says nothing
       about negative reals, where the two conventions disagree. **What this
       row adds, as a Phase-1-review reading rather than a further Director
       ruling:** one intrinsic, named ``floor`` (not a second ``trunc``),
       domain-restricted to ``[0, ∞)`` — the ratified call sites are
       non-negative **people** counts, not wealth: ``vitality.py``'s
       ``population`` (guarded ``> 0``) times a ``[0, 1]``-clamped
       ``attrition_rate``, and ``decomposition.py``'s population splits
       (guarded ``la_population > 0``, non-negative fraction defines) — its
       WEALTH lines at the same call site are not ``int()``-demoted. Not a
       claim of §3.4, which asserts nothing about sign. On that domain
       ``floor`` and ``trunc`` coincide, so naming one is not silently
       resolving their disagreement, it is declining to extend the intrinsic
       into the domain where they disagree. A negative, non-finite, or
       ``i64``-overflowing argument is ``E-EVAL-039`` — a loud failure
       (III.11), never a silently-chosen rounding convention and never a
       wraparound; a bare non-``Real`` argument (an ``Int`` literal, not the
       result of arithmetic) is a separate, uncoded structural refusal — §3.3
       promotes ``Int`` to ``Real`` only inside a binary64 expression, not at
       the intrinsic-call boundary, and no static typechecker exists yet to
       enforce it there (§2.7, Phase 2). Reference implementation:
       ``declarations::DECLARABLE_INTRINSICS`` (the cap membership),
       ``declarations::parse_intrinsic_decl`` (the content-level
       ``(intrinsic …)`` parse, refusing an uncapped name AT the parse — D98
       covers its ``:params``/``:returns`` type-name reading),
       ``intrinsic_host::KernelIntrinsicHost`` (the evaluator, this crate's
       first non-empty, non-test-only ``IntrinsicHost`` — wired into
       ``babylon_tick::run_once`` itself, not merely exercised by a unit
       test) and ``evaluator::EvalCode::DemotionOutOfDomain``
       (``rust/crates/babylon-bsl/src/``). Unlike ``exp``/``log``, ``floor``
       needs no pinned soft-float libm crate: it is IEEE-754's own
       ``roundToIntegralTowardNegative``, exactly specified by the standard
       and reproducible bit-exactly on that authority — not by §4.3, whose
       closed basic-op list is ``+ − × ÷`` and comparison only. ADR188's
       "libm golden vectors at implementation" consequence therefore does
       not apply to this rider; that is this row's explicit disposition of
       it, not a silent omission. ``round-half-even`` (ADR188 Row 3) is a
       separate, not-yet-landed rider; this row does not perform its
       landing.
   * - D98
     - §2.7, §3.1, §7
     - **``real`` is admitted as an ``<intrinsic-decl>`` ``:params``/
       ``:returns`` type-name, and only there, through a dedicated
       ``<intrinsic-type-name> ::= <type-name> | "real"`` production** — a
       Phase-1-review reading needed to make D97's own ``floor`` rider
       declarable at all, since §3.1's six-row ``<type-name>`` table (D94:
       lowercase symbols, ADR191 R4) has no ``real`` row and every
       intrinsic's argument routinely IS ``Real``-typed (§3.3). Not a
       widening of ``<type-name>`` itself (``deffield``'s / a ``metric``'s
       ``:type``): §3.1 rules ``Real`` "Not storable", so a field or a
       registered metric still cannot be ``Real``-typed, and neither §2.9's
       nor §2.11's production changed — only ``<intrinsic-decl>``'s two
       positions were re-pointed at the new production. This reading treats
       ``real`` as machinery rather than new mathematics — §3.3/§4.3
       already name the binary64 lane's unbounded intermediate kind; this
       only makes it spellable in one more grammar position — and is, like
       D97, a workforce reading open to correction, not a Director ruling.
       ``bsl.ebnf`` carries the same production and the same re-pointed
       ``intrinsic-decl``, landed in the same commit as this row (§7's
       precedence discipline: section text first, appendix follows in
       lockstep) — a dedicated sync-guard row
       (``TestTheIntrinsicTypeNameVocabulary`` in
       ``tests/unit/reference/test_bsl_grammar_sync.py``) checks the
       appendix's ``intrinsic-type-name`` production's own right-hand side
       for the ``"real"`` alternative, since the generic production-
       containment tests only check that a left-hand-side NAME exists on
       both sides, not that a shared name's alternatives stayed in sync.

       **Signatures are enforced, not merely declared** — the companion
       Rust landing (``declarations::kernel_signature``,
       ``parse_intrinsic_decl``) checks every ``(intrinsic …)``'s declared
       ``:params``/``:returns`` against the kernel's registration for that
       name at LOAD time and refuses ``E-LOAD-020`` on any disagreement
       (wrong arity, wrong parameter type, or wrong return type), so §2.7's
       and the appendix's own promise — "a declaration whose signature
       disagrees with the kernel's registration is `E-LOAD-020`" — is now
       upheld rather than merely stated.

       **Declaration-malformity errors stay uncoded**, per the crate's
       standing no-invented-codes precedent: a missing ``:params``/
       ``:returns``/``:cost``, a repeated keyword inside one declaration
       (``:cost`` twice — the second occurrence would otherwise silently
       win and silently change fuel accounting), or an unrecognized
       ``:params``/``:returns`` type-name are all loud rejections with
       ``code: None`` — the reference names no numbered code for a
       malformed *shape*, only for the named conditions above (`E-LOAD-001`
       duplicate name, `E-LOAD-020` signature mismatch, `E-LOAD-024`
       reserved/prohibited name). Reference implementation:
       ``declarations::{IntrinsicTypeName, parse_intrinsic_type_name,
       IntrinsicDecl, parse_intrinsic_decl, parse_intrinsic_decls,
       kernel_signature}``; the composed load path is
       ``rule_pipeline::split_content`` + ``load_rule_form``, wired into
       ``babylon_tick::run_once``.
   * - D99
     - §1.5, §3.1, §3.2, §4.6
     - **The declared-domain Currency scale operation** (Director ruling
       2026-08-11, #492/ADR194 — "Add one legal operation: Currency scaled
       by a defconst-declared positive coefficient whose domain is stated at
       declaration and enforced loudly at load/eval. Scalar multiplication
       isn't new mathematics — this reads as machinery, keeps defines'
       moddable shape intact"). Closes director-gate #492 (Blocker-1: a
       runtime-moddable coefficient with a declared domain beyond ``[0,1]``
       had no BSL representation at all — not merely "cannot multiply
       Currency", literally unwritable).

       **Design chosen, and what was rejected.** A NEW literal suffix,
       ``r`` → ``Ratio`` (§1.5), reusing ``babylon_kernel::scalars::Ratio`` —
       a kernel sort that ALREADY existed (`𝔾 ∩ (0, ∞)`, grid-quantized like
       every other bounded scalar) and simply had no BSL literal exposing
       it — rather than widening ``Coefficient``'s own ``[0,1]`` cap or
       introducing a distinct bounded-scalar type. Rejected: (a) widening
       ``p``/``i``/``c``'s existing ``E-LEX-024`` cap, which would silently
       change the domain of every EXISTING coefficient in the language, the
       exact "no silent widening of the whole language" director-gate #492
       forbade; (b) a context-sensitive lexer rule ("uncapped only inside
       ``defconst``"), which the reader's position-independent classification
       architecture cannot express without becoming a parser (every other
       literal class classifies identically regardless of where it appears —
       ``Currency`` itself is legal in general expression position, and
       ``Ratio`` follows that same precedent rather than being singled out);
       (c) minting a brand-new bounded-scalar kernel type, when ``Ratio``
       already states exactly the needed law and reuse is what "not new
       mathematics" means concretely.

       **The operation**, added to §3.2 rather than inside its verbatim
       design-doc quotation: ``Currency × Ratio → Currency``, rounded
       half-even — identical arithmetic to ``Currency × Coefficient``,
       commutative in source position. ``Ratio`` gets exactly this one
       operator and no other (no ``+``, no ``Ratio × Ratio``, no ordering) —
       "scalar multiplication isn't new mathematics" read literally: the
       addendum does not grow a second arithmetic subsystem alongside the
       binary64 lane, it adds one multiply rule to the existing Currency
       lane. An undeclared ``Real`` above ``1`` in Currency-multiply
       position remains ``E-TYPE-030``, unchanged — the new operation is
       keyed to the ``Ratio`` type specifically, not to "any number over 1".

       **The domain declaration.** ``Ratio``'s own lexical domain,
       ``(0, ∞)``, already satisfies "domain stated at declaration" for a
       consumer whose real domain is unbounded (``rent_spike_multiplier``).
       A consumer needing a tighter bound states it via the Rust reference's
       ``defconst`` ``:floor``/``:cap`` extension — ``:cap`` narrows the
       UPPER end INCLUSIVE (``pareto_alpha``'s ``(0, 10]``), ``:floor``
       narrows the LOWER end EXCLUSIVE (``entropy_factor``'s ``(1.0, 3.0]``
       — the floor IS the thermodynamic point "extraction costs more than
       it yields", not decoration a ceiling-only design could omit; a first
       revision of this row shipped ``:cap`` alone and was found
       ceiling-only by adversarial review before this ruling landed, exactly
       the defect ``entropy_factor``'s own domain exists to catch). Both
       keywords are deliberately **not** rst productions or §1.6
       keyword-table rows: ``defconst``, like ``node``/``edge``/``scenario``,
       is the ``.bscn`` scenario-file dialect §7 already records as out of
       this document's grammar (D93 — "no section specifies it"). Extending
       that boundary — making ``defconst`` rst-normative — would have been a
       second, larger and unrelated decision riding along with this one;
       D93 stands as recorded, and both keywords are documented in
       ``scenario.rs`` itself, at the construct they belong to, cited here
       for orientation only. This IS "keeps defines' moddable shape intact":
       no new top-level form, no change to how a scenario declares a
       coefficient — one existing form (``defconst``) gains two optional
       keywords.

       **A bare (undeclared) ``r`` literal needs no ``defconst`` at all** —
       it is an ordinary ``<expr>`` atom, legal wherever any other literal
       is, matching ``Currency``'s own precedent (a ``$`` literal is legal
       directly in a rule body). The ruling's "defconst-declared" wording
       names the INTENDED authoring pattern for moddability, not a syntactic
       gate the reader enforces — rejected explicitly as design choice (b)
       above, for the same position-independence reason.

       **The zero endpoint.** ``early_mortality_modifier`` and
       ``carceral_transition_modifier`` are ``[0, 10]`` — CLOSED at zero —
       and the frozen engine actively PRODUCES ``0.0`` for them
       (``mobility.py:187-188``, the mortality channel OFF, semantically
       meaningful). ``Ratio`` cannot hold that ``0``: not a gap in this
       row's coverage but the SAME structural law that makes the construct a
       "positive coefficient" at all (``E-LEX-027`` at the reader,
       ``Ratio::new(0.0)`` refusing independently at the kernel). This row
       does NOT widen the sort to admit zero — that would be exactly the
       re-opened mathematics the ruling's own framing forbids. Instead:
       "zero-scaling is the multiply's ABSENCE, not a scale" — the content-
       layer answer is ``guard`` (§2.8), which already gates whether an
       EFFECT fires at all, on a signal separate from the ``Ratio``-typed
       magnitude. Proved end-to-end rather than merely reasoned about (two
       tests in ``currency_scale_op_e2e.rs``: the guarded effect fires and
       the product is observed when a `Bool` `:const` is `#t`; the multiply
       never evaluates — an empty event list — when it is `#f`, which holds
       because the multiply sits INLINE in the guarded effect body; a
       binding-shaped multiply would resolve before the guard). WHICH
       signal a real Lifecycle port gates on is that port's own
       content-modeling decision, left open by design; this row proves only
       that the mechanism is available.

       **Error codes**, contiguous with the existing sequences (§4.6):
       ``E-LEX-027`` (a non-positive ``r`` literal — lex time, every
       occurrence, position-independent, exactly as ``E-LEX-024`` gates
       ``p``/``i``/``c``), ``E-LOAD-052`` (a ``defconst``'s own ``Ratio``
       value violating a declared bound, OR the bounds disagreeing with
       each other (``floor`` not strictly below ``cap``) — load time,
       self-consistency of the declaration, checked in THAT order so an
       inconsistent declaration is diagnosed as inconsistent rather than
       misreported against whichever bound the value happens to fail),
       ``E-EVAL-041`` (a ``Currency × Ratio`` operand falling outside its
       declared domain, at or below an EXCLUSIVE floor or above an
       INCLUSIVE cap — evaluation time, the operation's own re-check). The
       load- and eval-time checks are DELIBERATE duplication, not redundant
       dead code: through ``defconst`` — the only producer of a bounded
       ``Ratio`` this revision wires — they can never disagree (III.11's
       loud-failure standard applied at both checkpoints anyway, the same
       discipline the store boundary already practices at ``E-EVAL-020``
       alongside a field's own load-time type check); ``E-EVAL-041``'s
       independent value is for whatever OTHER path a later revision adds to
       produce a ``Value::Ratio``, which inherits the check rather than
       needing to invent it.

       **CAS round-trip.** A ``Ratio`` literal is `Atom::Scaled`
       (`reader.rs`) with a new `ScaledKind::Ratio`, encoding with its own
       `"ratio"` kind tag (`canonical_ast.rs`) — purely additive, proved by
       pinning §5.6's then-current worked example unmoved (421 bytes, both
       digests) for that revision before
       asserting the new atom encodes at all
       (`a_ratio_literal_encodes_with_its_own_kind_tag_additively`).
       Canonicalization (minimal scale, e.g. ``2.50r`` ≡ ``2.5r``) follows
       the identical rule every other scaled literal already has (§1.5).

       Reference implementation: ``reader::classify_ratio`` (lexing,
       `E-LEX-027`), ``scenario::load_defconst``/``load_ratio_defconst``/
       ``parse_bound_keywords`` (`:floor`/`:cap`, `E-LOAD-052`),
       ``evaluator::Value::Ratio``/``currency_mul_ratio`` (`E-EVAL-041`),
       ``canonical_ast::encode_atom`` (the `"ratio"` CAS tag),
       ``babylon_kernel::Currency::mul_ratio`` (the arithmetic, mirroring
       ``mul_coefficient`` exactly) — ``rust/crates/babylon-bsl/src/`` and
       ``rust/crates/babylon-kernel/src/currency.rs``. End-to-end proof
       through ``babylon_tick::run_once``, including the floored-and-capped
       ``entropy_factor`` worked example and the guard-gated zero-endpoint
       disposition:
       ``rust/crates/babylon-tick/tests/currency_scale_op_e2e.rs``.
   * - D100
     - §2.2, §4.2
     - **ADR222/PER-17 supersedes this scheduling record.** The multi-rule
       admission and duplicate-id refusal remain. The global byte-order
       fallback and inert-anchor clauses below describe the pre-ADR222 driver.

       **The content-set loader admits more than one** ``(rule …)``
       **top-form, executed in ascending rule-id byte order (register row
       D16, §4.2), duplicate ids refused** — a driver-level fix (Program 28
       B2), not a spec change. §2.2's grammar (``<top-form>*``) and prose
       ("Duplicate rule ids… across the content set are ``E-LOAD-001``")
       never limited a content set to one rule; ``babylon-bsl::
       rule_pipeline::split_content`` did, by an implementation-level
       cardinality check with no textual basis in this section. This row
       lifts that check to match the grammar it was always supposed to
       implement, and applies an EXISTING ruling to the result — D16
       already says "rules at the same anchor position evaluate in
       ascending rule-id byte order… file order and load order are never
       observable"; a slice-1 content set carries no anchor-position
       registry to differentiate positions across systems (Phase 3,
       unbuilt), so every rule in the set sits, in effect, at one shared,
       unresolved position, and D16's byte-order fallback governs.
       **This row mints no new ordering law** — this row applies D16 to a
       case D16's own text already covers, not a second rule alongside it.
       ``(anchor …)`` forms still parse and ``check_anchor`` still
       validates them (unchanged) but stay inert for ordering under this
       row, exactly as before it — resolving them into a cross-system total
       order remains Phase 3's job, deferred with a name.

       **Disclosure (adversarial-panel finding FB7, Program 28 B2's own fix
       round).** Byte order INVERTS the frozen Python engine's own tick
       order across the current estate: the frozen engine runs Vitality
       (position 1) before Lifecycle (7) before Dispossession (10) before
       Metabolism (13), while byte order sorts
       ``dispossession/* < lifecycle/* < metabolism/* < vitality/*`` —
       not the frozen sequence, nor its exact reverse: it disagrees with
       the frozen order on four of the six pack pairs, and Vitality moves
       from first to last. Safety today rests entirely on a hand-check that each
       shipped pack's own fields stay separate from every other pack's (no
       field one pack writes, another pack ever reads — checked by hand
       for vitality/lifecycle at this row's own landing, see the B2 plan's
       "Field and local-name collisions" section) — a property of today's
       four packs, not a guarantee the driver enforces mechanically. A
       future pack pair that shares a node type or cross-reads a field
       would NOT enjoy this safety, and the driver would not catch the
       divergence at load time. A mechanical check — refusing loudly at
       load time when two rules in a content set share a read/write on the
       same field — remains future work, tracked on `#503
       <https://github.com/percy-raskova/babylon/issues/503>`_ alongside
       this program's other named follow-ups; this row does not build it.

       Reference implementation: ``rule_pipeline::split_content``,
       ``canonical_ast::rule_id`` (widened to ``pub(crate)``),
       ``lib::prepare_rules`` (Program 28 B2, ``docs/superpowers/plans/
       2026-08-11-b2-tick-loop-plan.md`` Phase A Tasks 2–4).
   * - D101
     - §2.13, §3.1
     - **The** ``enum`` ``<type-name>`` **row and its declaration form**
       (Director ruling 2026-08-11, Organization-as-game-object spec §1
       Q12 — approved twice, live brainstorm popup and written spec
       review). A seventh ``<type-name>`` row — the declared-order
       ordinal of a content-declared closed enum, stored in the existing
       binary64 attribute lane and surfaced only as an ``<enum-ref>`` —
       joins the six D94 sealed. **This row explicitly supersedes D94's
       exclusion**: D94 read "no ``<type-name>`` position can name" a
       seventh kind, ruled the same review that closed the table at six;
       Q12 is the Director's own later ruling minting a seventh, made
       knowingly (the question put to her named "sealed twice this
       month"). D94 itself is left unedited (Immutability of History: it
       captured what was true through that review) and its six-row
       reading stands as a record of what was true before Q12; §3.1's
       table and the sealing paragraph beside it now state the current,
       seven-row law. ``Enum<T>`` (§3.1's next row, the *typechecker*
       classification of an ``<enum-ref>`` expression) is unchanged — Q12
       mints a **storage** row, not a new expression type.
       ``defenum``/``defvocabulary`` (§2.13), the widened ``<deffield>``
       production (§2.9), and error codes ``E-LOAD-053`` through
       ``E-LOAD-056`` and ``E-EVAL-042`` (§4.6) are this ruling's
       consequences. First consumer: ``organization/kind`` (spec §1 Q1),
       landing with the Phase-2 vocabulary-ceremony ADR.
   * - D102
     - §2.13, §2.10
     - **Discharged (Territory port train, P27, 2026-08-12).**
       ``field-of`` (§2.10) was **not** extended to ``:enum-type``-declared
       fields by D101 — such a field could be read only via a ``:field``
       binding (§2.5); a ``field-of`` naming one refused loudly, citing
       this row, with no error code (a deferred gap, not a new failure
       class). Deferred, not forbidden: an enum-declared EDGE or
       HYPEREDGE field (this document does not rule one out) would need
       ``field-of`` access precisely because it has no single owning
       body to bind a ``:field`` against — the same reason §2.10 exists
       for dyadic and hyperedge fields at all — and this row named the
       gap so a future implementer would not have to rediscover it as a
       silent "the enum-ref case must have been an oversight" bug.
       **First consumer, and the discharge:** Territory's
       ``_find_sink_node`` (``src/babylon/engine/systems/territory.py``)
       reads a *neighbor* territory's ``territory_type`` — an
       enum-declared field with no ``:field`` binding available (a
       binding's ``:field`` source can only name ``self``'s own field,
       §2.5), so the sink-priority query needs exactly the accessor D102
       withheld. The discharge is narrow: ``field-of`` on an
       enum-declared field now typechecks (``score_class::classify``
       types it ``Enum``, not ``Real`` or ``Unknown``) and evaluates
       (``evaluator::field_of_node`` renders the stored ordinal to
       ``Value::Enum`` via the SAME shared helper — ``tick::
       bind_field_value``, widened ``pub(crate)`` — a ``:field``
       binding's read of the identical field already used) for real. Its
       two surviving refusals are each independent of D102 and untouched
       by the discharge: **score position** (D46, ``E-TYPE-016``) still
       refuses an ``Enum``-classed ``select-max``/``select-min`` score,
       via ``score_class::classify`` and ``typecheck::
       check_selection_scores`` exactly as it already refused a
       ``:field``-bound enum score; **arithmetic** (§2.13's no-arithmetic
       law, D101/D118) still refuses ``Value::Enum`` in every arithmetic
       lane, via ``evaluator::apply_arith``'s unconditional fallthrough
       refusal (expression position) and ``structural_verbs::
       numeric_write_value``'s catch-all (an ``update-node`` write
       operand). ``typecheck::check_no_field_of_on_enum_field`` — the
       unconditional load-time deferral gate this row named — is deleted
       rather than narrowed: with the deferral discharged and both
       surviving refusals independently mechanised, there was nothing
       left for a third gate to decide.
   * - D103
     - §4.2, §2.8
     - **Resolved (query-evaluation plan Q1).** ``tick.rs::run_tick``'s
       pre-Task-12 divergence — mutating the graph in place as it walked
       subjects, so a later subject in one rule's firing order observed
       an earlier subject's ALREADY-APPLIED effects — contradicted chapter
       C4's own text ("all firings of one rule observe the same
       pre-state … applied in that subject order") from the moment this
       document ratified it; ``tick.rs``'s own module doc admitted the
       gap rather than asserting it as a feature. **The spec's text is
       followed, not amended**: ``run_tick`` now runs two passes —
       collect every subject's effects against one shared, unmutated
       pre-tick borrow, then apply the collected writes in subject order
       — verified byte-neutral for every rule pack landed at the time of
       the repair (``rust/crates/babylon-tick/tests/tick_goldens.rs``,
       unmoved). Reference implementation: ``structural_verbs::
       PendingWrite``, ``EffectExecutor::{collect_effects,
       apply_pending_write}``, ``tick::run_tick``'s two-pass split.
       Mutation-verified: reverting to inline per-subject application
       flips ``all_firings_of_one_rule_observe_the_same_pre_state`` red
       while ``tick_goldens`` stays green, proving the guard is a guard
       and not merely a restatement of the fix. See also D116 (Q14), the
       narrower, still-open sibling of this same divergence one anchor
       level up.
   * - D104
     - §2.8, §4.2
     - **Resolved (query-evaluation plan Q2).** Under D103's
       collect-then-apply split, ``add``/``sub``/``scale`` read the
       write target's CURRENT value at **apply** time, not at collect
       time. Collect-time reads would be unsatisfiable against §4.2's own
       accumulation clause: several subjects each contributing a slice to
       one shared carrier node must reduce all of their contributions,
       and a collect-time read would let a later subject's collected
       write silently overwrite (not accumulate atop) an earlier one's,
       losing every contribution but the last. Reference implementation:
       ``PendingWrite``'s own doc comment states this explicitly as the
       reason it carries an *operand*, never a pre-resolved target value;
       ``EffectExecutor::apply_pending_write`` performs the read.
       ``accumulation_into_a_shared_target_reduces_in_subject_order_and_
       keeps_every_contribution`` (§6.2 family 12) is the vector.
   * - D105
     - §2.6, §2.10
     - **Query-evaluation plan Q3, left deliberately uncoded.** A query
       OPERAND naming no live element — a dangling ``NodeRef`` fed to
       ``(neighbors <expr> …)`` — has no ``E-EVAL`` code of its own: §2.6
       codes the *annotation* mismatch (``E-EVAL-032``) and §2.10 codes
       *accessor referents* (``E-EVAL-033``), but neither covers a query
       head's own source-element operand. The landed evaluator
       (``query::materialize_neighbors``) raises a loud, UNCODED error
       naming this row rather than inventing a number, per the crate's
       standing "no invented codes" rule
       (``neighbors_of_a_dangling_node_is_loud_not_empty`` asserts
       ``err.code == None``). **Numbering note, itself an instance of the
       collision this document's own Task-16 filing discipline warns
       about:** the query-evaluation plan's draft text proposed
       ``E-EVAL-042`` as "the next free number" at the time it was
       written; D101 (§2.13, the Organization spec's Q12 enum-field
       ruling) allocated ``E-EVAL-042`` to an unrelated law in the
       interim, before this row was filed. Should this row ever graduate
       from uncoded to coded, the number to allocate is whatever is next
       free **at that time** — as of this row's own filing, that is
       ``E-EVAL-043`` — never ``042``, and never hard-coded off this
       row's own text without re-checking the register first.
   * - D106
     - §2.6
     - **Query-evaluation plan Q4, recorded not ruled.** Whether a
       self-loop edge places a node in its own ``neighbors … :any`` set
       is unstated by this document; the substrate's current
       implementation yields the node itself when such an edge exists.
       No landed conformance vector or content rule exercises a
       self-loop topology, so this is a silent, implementation-defined
       answer of exactly the kind §2.6 exists to close — recorded here so
       a future vector can pin it explicitly, either way, rather than
       leave it accidental.
   * - D107
     - §2.7, §3.4, §4.2
     - **Resolved (query-evaluation plan Q5).** Weighted ``mean``'s
       reduction shape is ``Σ(wᵢ·xᵢ) ÷ Σwᵢ``, with both sums reduced in
       §2.6 iteration order — not an incremental running-average update,
       which would associate differently in binary64 and is therefore a
       genuinely distinct computation, not a rewrite of the same one.
       Pinned with exact bits:
       ``weighted_mean_is_sum_of_products_over_sum_of_weights``
       (``evaluator.rs``, Task 5).
   * - D108
     - §3.3, §4.3, §4.4
     - **RULED (Director, 2026-08-11, query-evaluation plan Q6).**
       ``fold mean`` over an ``Int``-typed body REFUSES LOUDLY, citing
       this row — ``mean`` serves ``Real``-typed bodies only. §4.3 leaves
       ``Int ÷ Int`` with "no pinned semantics" as a loud evaluation
       error outside a binary64 expression, and whether ``mean``'s own
       division counts as one was genuinely silent between §3.3 and
       §4.3; the Director's ruling closes it by refusal rather than by
       inventing a promote-then-divide reading nothing in this document
       licensed. ``count``'s ``Int`` result (§3.4 row 6) and ``sum``'s
       type-preserving result are untouched — this row narrows ``mean``
       alone. Reference implementation:
       ``mean_over_an_int_body_refuses_by_name`` (``evaluator.rs``,
       Task 5), which comments this row's number rather than deciding
       silently.
   * - D109
     - §3.2 addendum, §4.4
     - **Resolved (query-evaluation plan Q7).** ``fold sum`` over an
       empty ``Ratio``-typed body is an ILLEGAL fold-body type, not a
       case needing an additive identity. §4.4's "additive identity of
       the body type" predates the ``Ratio`` variant (§3.2 addendum,
       #492/ADR194), whose only legal operator is ``Currency × Ratio``
       and whose own domain is open at zero — minting an identity for it
       would place a value outside the type's own declared domain, which
       is precisely what the ``Ratio`` ruling exists to forbid elsewhere.
   * - D110
     - §2.7, §2.8
     - **Resolved (query-evaluation plan Q8) — a misdiagnosis corrected,
       not a spec gap filled.** ``(guard …)`` in expression position was
       refused pre-Task-1 as an unimplemented Phase-2 seam, naming the
       wrong reason: §2.7's ``<expr>`` production has no ``guard``
       alternative at all, and §2.8 makes ``guard`` effect-position-only
       by construction — the refusal the evaluator gave was never a
       missing capability, only a wrong error message. Fixed by Task 1's
       ``EFFECT_POSITION_ONLY``/``UNSERVED_EXPRESSION_HEADS`` split. This
       row exists so a future implementer re-reading the old refusal text
       does not re-derive "guard needs a Phase-2 query evaluator" as a
       fact about the language.
   * - D111
     - §2.9, §2.13
     - **Query-evaluation plan Q9, recorded not resolved — Territory-port
       facing.** ``deffield`` has no representation for a SMALL CLOSED
       field like the frozen Territory system's ``profile`` (2-valued) or
       ``territory_type`` (5-valued), independent of D101/D102's later
       ``enum`` ``<type-name>`` row (Organization spec §1 Q12, filed
       above as D101): that row exists and could in principle serve this
       case, but this plan makes no ruling connecting the two — it is
       explicitly out of this train's scope (see this plan's "Explicitly
       out of scope" section) and is restated here, numbered, so the
       Territory port's own D-record inherits it named rather than
       silently rediscovering it.
   * - D112
     - §3.7, §4.5
     - **Resolved (query-evaluation plan Q10) — a property stated, not a
       change made.** §3.7's ``cost(query) = 1`` charges query
       MATERIALIZATION at a flat rate while the runtime work underneath
       (a ``neighbors`` filter walking every incident edge) is
       O(candidates); the implementation conforms to the letter of §3.7
       and §4.5 as written. Fuel exists to guarantee TOTALITY and
       DETERMINISM under declared cardinality ceilings, not to model
       wall-clock cost — stated explicitly here so a future
       national-scale profiling finding that disagrees becomes a
       measured, vector-re-blessed change, never a silently edited
       constant.
   * - D113
     - §2.7, §3.4
     - **Resolved (query-evaluation plan Q11) — a fix-round finding,
       corrected before this document's own text was checked against the
       implementation.** §2.7's ``<fold>`` grammar admits an optional
       ``:weight`` operand unconditionally on every ``<fold-op>``, but
       §3.4's per-operator semantic table gives ``:weight`` a reading for
       ``mean`` alone; the pre-fix evaluator silently evaluated and
       discarded a ``:weight`` operand on ``sum``/``min``/``max``/
       ``count``, which is a species of the "over-admitted at the
       grammar layer, silently dropped at evaluation" defect this
       document exists to prevent elsewhere. **Ruling: refuse loudly at
       evaluation** for every non-``mean`` fold-op supplying ``:weight``,
       naming the operator and citing this row — no grammar change (a
       grammar admitting more than its semantics use is acceptable; an
       evaluator silently discarding what it admits is not).
   * - D114
     - §2.7, §4.4
     - **Resolved (query-evaluation plan Q12) — an implementation
       boundary named, not a spec gap.** §4.4's empty-``fold-sum``
       additive-identity rule says "the additive identity of the body
       type" without saying every conforming implementation must be able
       to COMPUTE that identity for every §2.7-legal body shape. The
       landed classifier (``evaluator::static_additive_identity``) serves
       literals, ``field-of`` reads, and homogeneous arithmetic over
       them; a nested ``fold`` or a bare binding-symbol body are both
       load-legal shapes it does not attempt. **Ruling: an unclassifiable
       body shape refuses loudly, citing this row**, rather than guessing
       an identity or widening the classifier speculatively ahead of a
       real consumer needing it.
   * - D115
     - §3.4 row 6, §3.7, §4.5
     - **Resolved (query-evaluation plan Q13).** ``fold count``'s §3.7
       STATIC bound still charges ``ceiling(query) × (cost(body) +
       cost(weight))`` like every other fold operator — the formula is
       not special-cased — but §3.4 row 6 makes ``count``'s RESULT
       independent of the body's VALUE, and the landed RUNTIME meter
       (``evaluator::fold_count``) does not evaluate the body at all.
       This is not a divergence: §4.5's runtime meter exists as a
       backstop against the static bound being UNSOUND (charging too
       LITTLE, letting a rule exceed its declared ``:fuel``), never
       against it being merely conservative (charging too MUCH, which
       every other static over-approximation in this document already
       does). Under-charging ``count`` at the static-bound layer would be
       unsound; charging it there while not charging it at runtime is the
       safe direction and stays as implemented.
   * - D116
     - §4.2
     - **Query-evaluation plan Q14, recorded not fixed — a second,
       narrower instance of D103's same divergence, one anchor level up.**
       D103 repaired the WITHIN-one-rule half of §4.2's own sentence
       ("rules within one system position observe the same pre-state"):
       every subject firing of ONE rule now observes that rule's shared
       pre-tick state. The RULE-to-rule half is still open:
       ``babylon-tick``'s ``run_once_into``/``TickSession::advance`` run
       each rule in a content set to COMPLETION — collect and apply —
       before the next rule starts, against the SAME mutable graph, so a
       second rule at the same anchor position observes the FIRST rule's
       already-applied writes from this tick, not the tick's shared
       pre-state. Latent today: every landed rule pack keeps its system
       position to exactly one rule (``vitality.bsl``'s own header
       records the reasoning: a multi-rule decomposition would have to
       restate the drain algebra across rules), so no shipped content
       exercises the gap; ``query_lane_e2e.rs`` (Task 15) does not either
       — every one of its four vectors loads exactly one rule per tick,
       specifically to stay on the repaired side of D103 and clear of
       this row. Re-scoping ``run_once_into``/``TickSession::advance`` to
       collect-across-rules-then-apply is a second collect-then-apply
       repair, structurally identical to D103's but one anchor level up,
       carrying the same golden-baseline exposure — deferred to its own
       train. Until then, ``lib.rs``/``session.rs``'s own doc comments
       state the in-place cross-rule order as a RECORDED GAP citing this
       row, never as "the frozen engine's semantics, inherited for free."

       **Correction (Task W2, BSL Hygiene Knock-out, 2026-08-18): the
       "Latent today" premise above is FALSE.** It generalized from
       ``vitality.bsl``'s own one-rule-per-system-position design choice
       to "every landed rule pack" — but a direct count
       (``rg -c '^(rule ' content/rules/*.bsl``) finds
       ``consciousness.bsl`` alone carries TEN rules at its single anchor
       position (Consequences @17.0), and ``decomposition.bsl`` (six),
       ``territory.bsl`` (five), ``production.bsl`` (five) and
       ``control-ratio.bsl`` (four) are ALSO multi-rule packs at one
       position. **Corrected (W2 fix round 1, review finding M1):** by
       PACK count this is 5 of 13 (38%), not a majority — the rule-
       weighted reading is what supports "the norm": 30 of the corpus's
       38 rules sit in packs with siblings at one anchor position (79%),
       against 8 rules in the 8 remaining single-rule packs (``vitality``,
       ``lifecycle``, ``dispossession``, ``metabolism``, ``organization``,
       ``solidarity``, ``worldview``, ``fundamental-theorem``) — this
       row's premise assumed the single-rule shape was the corpus's
       default, and by either count it is not the majority of the
       corpus's actual rule content. The W2 pre-audit table (task report:
       ``.superpowers/sdd/2026-08-18-bsl-hygiene-knockout/task-w2-
       report.md``) found the gap this row calls latent is EXERCISED
       throughout ``consciousness.bsl`` — 13 bindings whose
       ``:optional :default`` is mechanically exposed to a same-tick
       writer sorting on/after the reader (rows 3-9, 14-16, 18, 20, 22),
       every one of them mitigated by guard structure or a documented
       one-tick-lag idiom rather than by the gap being unreached — plus a
       6-field multi-writer census with zero literal unconditional
       resets pre-repair. §4.2's own end now carries two load refusals
       (``E-LOAD-058``/``E-LOAD-059``) that check exactly this gap,
       content-set-wide, at load. This row's history stands unedited
       above (Immutability of History) — the premise it recorded was
       accurate for the estate as it stood when D116 was written and is
       false for the estate as it now stands.
   * - D117
     - §5.2
     - **Resolved (#528 fix round, PR #528's own adversarial-verifier
       finding — the CAS resolution for ``defenum``/``defvocabulary``'s
       bare atom).** §5.2's own Q12 paragraph stated "no new atom kind"
       needed for ``defenum``/``defvocabulary`` because "the
       ``<enum-ref>`` values they govern already encode with the existing
       atom kind" — a premise that is FALSE for these two forms' own
       operands: the type-name operand and each member-list item (§2.13's
       own EBNF: ``defenum ::= "(" "defenum" enum-type "(" enum-member+
       ")" ")"``) are BARE ``<enum-type>``/``<enum-member>`` atoms, never
       ``<enum-ref>`` ``Type/MEMBER`` pairs — the document contradicted
       itself the moment a reader takes ``defenum``'s own grammar literally.
       **Resolved by declaring, not inventing**: the SAME "enum" CAS kind
       tag (unchanged — still no NEW tag) widens to admit a SECOND payload
       shape, the bare identifier with no ``/``, discriminated from the
       existing ``<EnumType>/<MEMBER_IDENTIFIER>`` shape by the presence
       of exactly one ``/`` — collision-free because §1.4's
       ``enum-type``/``enum-member`` charsets exclude ``/`` entirely, so
       an ``<enum-ref>`` payload always contains exactly one and a bare
       operand's payload never does. §5.2's atom-kind/payload table and
       its own Q12 paragraph are both corrected to state this explicitly
       rather than the false single-shape premise. Reference
       implementation: ``rust/crates/babylon-bsl/src/reader.rs``'s
       ``Atom::BareUpperIdent`` (renamed from ``EnumTypeName``, which
       under-described what it now carries — see that variant's own doc)
       and ``canonical_ast.rs``'s ``encode_atom`` match arm, which already
       encoded exactly this discriminated-union shape before this row
       existed to declare it; the round-trip test
       (``canonical_ast.rs::tests::the_enum_atom_kind_round_trips_both_payload_shapes_without_confusion``)
       reconstructs ATOMS (not just kind+payload bytes) from both shapes to
       prove the discriminator is load-bearing, not merely byte-distinct by
       accident.
   * - D118
     - §2.13, §3, §4.6
     - **Resolved (#528 fix round, MAJOR item 2 + MINOR item 3) — a
       declared class widened, then the SAME law moved earlier, neither
       move minting a new code.** ``E-EVAL-042`` is the code
       ``structural_verbs.rs::refuse_arithmetic_on_enum_field``
       (``c268b83b``) has reported for an ``add``/``sub``/``scale``
       update-op targeting an ``:enum-type``-declared field since the
       enum write law landed, but neither §2.13's own E-EVAL-042 bullet
       nor the §4.6 class table row named that refusal — both scoped the
       code strictly to the write-SHAPE law (a non-``<enum-ref>`` value,
       or a cross-type one, reaching the write path), even though
       ``Enum<T>`` supporting no arithmetic is already stated, uncoded,
       at this section's own "No aggregation kind" paragraph and at
       §2.9's ruling. **Resolved by declaring, not inventing**: both
       citing passages now name the arithmetic refusal explicitly under
       the SAME code — minting a second E-EVAL number for another
       write-path violation of the identical law would be exactly the
       hygiene defect D75 ruled against. The refusal itself reaches all
       three sites a write can originate from (defense in depth,
       unchanged by this move): ``update_node``'s immediate execute path,
       ``collect_update_node``'s collect path, and
       ``apply_pending_write``'s independent re-check at apply time.
       **Second half — the law is statically decidable, so it now ALSO
       runs at load** (§3's own law: "every check in this chapter runs
       at content load"), matching the precedent D102's own (since-
       discharged) ``field-of`` deferral gate set for exactly this shape:
       ``typecheck.rs::check_no_arithmetic_on_enum_field``, wired into
       ``rule_pipeline.rs``, refuses an ``add``/``sub``/``scale``
       ``update-op`` targeting an ``:enum-type``-declared field at load,
       citing this row — the three eval-time sites above STAY, unchanged,
       as defense in depth, exactly as D102's own load-time gate left its
       eval-time siblings standing (before the Territory port train's
       discharge deleted that gate; ``check_no_arithmetic_on_enum_field``
       itself is untouched — see D102's row for the discharge).
   * - D119
     - §2.13, §3.6
     - **Resolved (#534 fix round item 1, adversarial-panel finding,
       mutation-reproduced).** ``ClosedVocabulary::check_enum_ref``
       (``rust/crates/babylon-bsl/src/vocabulary.rs``) conflated two
       distinct facts under one refusal: a kind ABSENT from the
       vocabulary — no ``defvocabulary`` for it at all — and a DECLARED
       kind whose members simply do not include the one written. Both
       fell through the same ``self.members.get(&kind).is_some_and(…)``
       lookup returning ``false``, so a scenario declaring ``NodeType``
       and ``EdgeType`` but not ``EventType`` refused ``EventType/…`` at
       ``E-LOAD-031`` — a kind it never opted into checking at all — which
       is exactly the shape §2.13's own text above already rules against
       ("a content set that declares no ``defvocabulary`` for a kind
       leaves THAT KIND's checking exactly as inert as it is today").
       **Resolved by reading the registry's OWN presence, not just
       membership within it**: an absent kind returns ``Ok`` (inert,
       unchanged from before this section existed); a declared kind —
       including one declared with zero members — keeps enforcing
       ``E-LOAD-031`` exactly as before. This is SCOPE, not a fallback:
       §3.6's "a name that is not in the registry is a load error, never
       a fallback" law binds a name checked AGAINST a registry that
       exists for its kind; a kind with no registry at all is simply not
       being checked, the same distinction §1.5 already draws between "no
       default" and "no check". No test in the landing PR (#534's Group D
       predecessor) pinned the all-or-nothing behavior this row corrects
       — the gap was latent, not a regression of a prior guarantee.
       **Recorded, not changed, in the same fix round (F8, item 8):** at
       any of the sixteen §2.6 class-rule operand positions this section
       names, an UNKNOWN type name — not merely the wrong structural kind
       — is ``E-TYPE-011`` in the reference implementation
       (``grammar::check_enum_ref_kinds``, D74, which runs earlier and
       unconditionally), never the ``E-LOAD-030`` this section's own text
       above states; ``ClosedVocabulary::check_enum_ref``'s ``E-LOAD-030``
       branch is unreachable at these specific sixteen positions for
       exactly that reason. Pre-existing since D74 landed, previously
       recorded only in ``check_enum_ref_membership``'s own Rust doc
       comment (grammar.rs) — folded into the register here so the
       divergence is honest at spec level too.

       **Extended (G2, #534 fix round 2 item 2, mutation-reproduced).**
       The SAME conflation this row already records for the sixteen §2.6
       rule-form positions recurred, independently, at the SCENARIO
       HYDRATION positions — a ``.bscn`` file's own ``node``/``edge``
       forms. §3.9 clause 1 authorizes exactly this check, unconditionally
       ("It creates elements of declared types and writes declared
       fields, and nothing else. An undeclared field or type at hydration
       is the same ``E-LOAD-0xx`` class as it would be in content —
       hydration is not a back door into the closed vocabulary (§3.6)."):
       a node/edge form's own type operand demands its position's kind
       (``NodeType``/``EdgeType`` respectively) independent of whether
       any ``defvocabulary`` was declared for it at all — mirroring the
       sixteen rule-form positions' own unconditional reading, one level
       down. The reference implementation's hydration-side producer,
       ``scenario::demand_enum_kind`` (``rust/crates/babylon-bsl/src/
       scenario.rs``), originally reported EVERY kind mismatch here as
       ``E-LOAD-030`` — including a syntactically-real type at the wrong
       position (``(node x EdgeType/SOLIDARITY)`` under a vocabulary that
       registers both kinds), whose message ("EdgeType is not a
       registered enum type") was false for exactly that case: ``EdgeType``
       plainly IS a registered structural kind, just not this position's.
       **Resolved the same way this row's own §2.6 half already is**: a
       written type that is REAL — one of the four structural kinds, or a
       type this scenario declared via ``defenum`` — but wrong for the
       position is ``E-TYPE-011`` (``VocabularyError::WrongEnumKind``); a
       written type that names nothing real at all, anywhere, stays
       ``E-LOAD-030`` (``VocabularyError::UnknownEnumType``, whose message
       is now true of every case it fires for). This split is POSITIONAL,
       not vocabulary-gated, exactly as the unconditional §3.9 clause 1
       reading demands: it holds whether or not the written kind was
       itself ``defvocabulary``-declared in this scenario, and whether or
       not ANY ``defvocabulary`` form appears in it at all — the SEPARATE
       membership check a threaded ``ClosedVocabulary`` performs (this
       row's own F1 half, above) is the only opt-in half of the hydration
       path.

       **Disclosed (H2, #534 fix round 3): the split is ALSO
       registry-relative, not just positional.** A scenario-declared
       ``defenum`` type participates in "is this a REAL type" only from
       its OWN declaration point down — the same "declaration must
       precede use" discipline ``deffield``/``defconst``/
       ``defvocabulary`` already enforce, and consistent with
       ``vocabulary_so_far``'s own running-snapshot reading (F1's half,
       above). ``(defenum OrgKind (BUSINESS)) (node x OrgKind/BUSINESS)``
       is ``E-TYPE-011`` (``OrgKind`` genuinely exists by the time the
       node form runs); the SAME probe with the ``defenum`` moved AFTER
       the ``node`` form is ``E-LOAD-030`` (``OrgKind`` genuinely names
       nothing YET at that point in the load) — not a bug, the same
       ordering sensitivity every other declared-registry lookup in this
       loader already has, undisclosed until now.

   * - D120
     - §4.2
     - **Territory port train (P27 PR B, Task 8) — recorded, relying on
       D116, not a repair.** ``territory/p1-heat-dynamics`` <
       ``territory/p2-eviction-pipeline`` < ``territory/p3-spillover`` <
       ``territory/p4-camp-decay`` < ``territory/p4-penal-suppression``,
       the ascending rule-id byte order (§4.2, D16) the four frozen
       phases' own SEQUENTIAL design needs: eviction reads this-tick
       post-Phase-1 heat, camp decay eats this-tick displaced arrivals
       (``territory_conformance.rs::p4_camp_decay_eats_this_ticks_
       displaced_arrivals`` is the end-to-end proof — a camp seeded 500
       plus a 100 same-tick arrival ends ``floor(600 × 0.8) = 480``,
       which could only be right if p4 observes p2's already-applied
       write). This is D116's cross-rule divergence (today's
       ``run_once_into``/``TickSession::advance`` run each rule in a
       content set to completion before the next starts, against the
       SAME graph) turned into a DEPENDENCY rather than fought: the pack
       deliberately keeps its four phases at ONE anchor position,
       ordered by id, and would break if the anchor registry ever
       resequenced them. Phase boundaries become POSITION boundaries —
       four distinct sub-positions of ``territory`` @2.0, not four rules
       sharing one — the moment the Q14 repair train (D116's own text)
       lands a real anchor registry; this row is that train's
       acceptance-criterion input for Territory specifically.
   * - D121
     - §2.9, §3.1
     - **Territory port train, Task 5.** ``territory/under-eviction`` is
       an ``int`` field storing ``0``/``1``, never a ``bool``. Confirmed
       at the loader: ``scenario.rs::load_deffield``'s ``ty.as_str()``
       match is exhaustive over exactly five names (``int`` /
       ``probability`` / ``intensity`` / ``coefficient`` / ``currency``)
       and has no ``bool`` arm — the reference-document-canonical
       ``declarations.rs::FieldRegistry`` dialect admits ``bool``, but
       nothing on the live
       ``.bscn`` pipeline reads that dialect. Independently,
       ``structural_verbs.rs``'s write path (``numeric_write_value``)
       has no ``Bool``-typed store arm at all — ``GraphSubstrate::
       update_node`` stores ``f64`` only. Both halves of "declare a bool
       field, write it via update-node" stay unavailable regardless of
       which gap closes first. This follows the
       ``social-class/active`` / ``organization/active`` precedent
       exactly (guarded ``(= flag 1)``, written ``(set 1)``/``(set 0)``).
   * - D122
     - §3.2, §3.10
     - **Territory port train, Task 5.** ``territory/rent-level-x1e6``
       stores the rent level × 1,000,000 as a plain ``int`` — the SAME
       scaled bare-``Int`` escape hatch ``metabolism.bsl``'s
       ``entropy-factor-x1e6`` (D-1 there) and ``dispossession.bsl``'s
       D-2/D-4 already use and document. Rent has no ``[0,1]`` domain
       (it starts at 1.0 and grows without bound via the 1.5× eviction
       spike), so it cannot be a ``probability``/``intensity``/
       ``coefficient``-typed field, and there is no ``Currency``-typed
       field storage on the live pipeline (the Director's 2026-08-11
       ruling defers that to Currency's first real consumer). A bare,
       unsuffixed ``Int`` ``:const``/field carries no domain check at all
       (``E-LEX-024`` only bounds scaled/suffixed literals). Retires when
       #502 workstream 3 (post-port refactor program) lands a genuine
       ``Real × Ratio`` operator or ``Ratio``-typed field storage.
   * - D123
     - §2.6
     - **Territory port train, Task 5 vs Task 6 — the frozen asymmetry,
       transcribed, not repaired.** The eviction sink walk
       (``territory/p2-eviction-pipeline``) queries ``(neighbors self
       EdgeType/ADJACENCY :out NodeType/TERRITORY)`` — DIRECTED, matching
       the frozen ``_find_sink_node``'s own ``edge.source_id ==
       source_node_id`` filter (``territory.py:174``). The spillover walk
       (``territory/p3-spillover``) queries the SAME edge type ``:any`` —
       matching the frozen ``_process_spillover``'s own header
       (``territory.py:279-284``): "ADJACENCY is conceptually
       bidirectional and the producer (ADR179 T1) stores ONE canonical
       edge per unordered pair, so each edge spills in BOTH directions."
       One edge type, two different directional readings, both faithful
       to the frozen engine's own two different walks over it.
   * - D124
     - §2.7, D45
     - **Territory port train, Task 5 — a comparison this port owes,
       resolved by NOT resolving it (both stand, independently correct
       within their own systems).** The frozen ``_find_sink_node``'s
       same-priority tiebreak among adjacent sinks is a Python ``dict``
       keyed by ``TerritoryType`` — enumeration-order LAST-WINS
       (``territory.py:186-188``: later edges overwrite earlier ones in
       ``adjacent_sinks``). This language's ``select-max`` tiebreak
       (§2.7, D45) is ascending-id byte-order FIRST-WINS
       (``query_lane_e2e.rs``'s own Shape B vector proved this end to
       end before the port started). This fixture never exercises the
       divergent case (``latch-tick-source``'s two sinks, ``sink-penal``
       and ``sink-reservation``, differ in PRIORITY score — 3 vs 2 — so
       the tiebreak rule never actually decides between them; a future
       fixture with two SAME-priority sinks would need to choose which
       engine's answer it pins).
   * - D125
     - §3.3, §3.10
     - **Territory port train, Task 4 vs Task 6 — transcribed faithfully,
       not repaired.** ``territory/p1-heat-dynamics`` clamps ``[0,1]``
       both sides — the frozen ``system_base.py::_write_clamped``'s own
       ``max(lo, min(hi, value))`` shape, nested-if in this language
       (§3.10's "no scalar min/max" ruling). ``territory/p3-spillover``
       clamps UPPER-ONLY — the frozen ``_process_spillover``'s own
       ``min(1.0, current_heat + spillover)`` (``territory.py:315``),
       which never floors a negative result (spillover is always
       non-negative, so the frozen author simply never needed a floor
       there). Two clamp shapes for the same field, inherited from two
       different call sites in the frozen source; this port keeps both
       exactly as written rather than unifying them.
   * - D126
     - §1.5, §3.5
     - **Territory port train, Task 3.** The frozen engine reads every
       attribute via ``attrs.get(key, default)`` — ``current_heat =
       attrs.get("heat", 0.0)``, ``under_eviction = attrs.get(
       "under_eviction", False)``, and so on throughout all four phases.
       §1.5's "no defaults" law admits no transcription of that
       affordance: an unwritten field errors on read
       (``scenario.rs:56-58``). ``territory-conformance.bscn`` seeds all
       six declared territory fields (profile, territory-type, heat,
       rent-level-x1e6, under-eviction, population) on every one of the
       twelve territories for exactly this reason —
       ``territory_conformance.rs::every_territory_seeds_all_six_
       declared_fields`` is the load-time proof.
   * - D127
     - §2.8, §4.2
     - **Territory port train, Task 5 (p2) and Task 6 (p3) — hash-neutral
       no-op writes where the frozen engine skips the write entirely.**
       (a) p2's no-sink fallback: when a latched territory has no
       qualifying adjacent sink, this pack's ``update-node`` still fires
       against ``self`` with ``(add 0)`` — the frozen engine's
       ``transfers`` dict simply never gains an entry for that source, no
       write happens at all. (b) p3's isolated-territory fallback: this
       pack's ``update-node`` still fires ``(set clamped)`` where
       ``clamped`` is bit-identical to the pre-tick ``heat`` (the
       ``(- 0 0c)`` no-inflow branch), where the frozen engine's own
       ``spillover_amounts`` dict never gains an entry for an isolated
       node and its apply loop never visits it. Both are NUMERICALLY and
       HASH-NEUTRAL — ``territory_conformance.rs::p3_isolated_territory_
       heat_is_byte_unmoved_by_spillover`` proves (b) directly — but a
       byte-for-byte ``CanonicalState`` diff against a hypothetical
       skip-the-write frozen-faithful engine would still show a write
       occurred, even though no observable value moved. §2.8 admits no
       "conditionally skip this effect" construct finer than the rule's
       own ``(when …)`` guard, so this is the shape every BSL port of a
       ``dict``-collected, sparse-apply frozen loop will have.
   * - D128
     - §4.3
     - **Territory port train, Tasks 5-6 — measured BSL expecteds are the
       oracle (ADR183), never chased to bit-match the frozen engine's own
       operation sequence.** Two sites: (a) the rent-spike lane computes
       ``(/ (* rent-x1e6 spike-x1e6) 1000000)`` — a scaled multiply-then-
       divide — where the frozen engine computes ``current_rent *
       rent_spike_multiplier`` as ONE multiply — a different binary64
       PROGRAM for the same real-valued function (the same class
       metabolism.bsl's own D-1 names, "correctly-rounded operations
       composed in a different order are not guaranteed to agree"); this
       fixture's exact-integer inputs happen to divide evenly, so this
       row's own divergence never shows here, but the lane is not proven
       exact in general. (``rent-x1e6``, an ``int``-declared field, reads
       back as ``Value::Real`` at evaluation — ``tick.rs::
       bind_field_value`` renders every non-enum-declared field this way
       — so the multiply and division both run in the binary64 lane
       directly; no promotion of any kind is needed or present.) (b) p3's
       pull-side fold computes ``Σ(heatᵢ) × rate`` (one multiply, after
       the sum) where the frozen ``_process_spillover`` accumulates
       ``Σ(heatᵢ × rate)`` (one multiply per edge, summed) —
       mathematically the same function, a different rounding path in
       general; measured bit-identical for this fixture's specific
       inputs (``territory_conformance.rs``'s own comments record the
       exact bits), not proven identical as a property of the lane.
   * - D129
     - N/A (frozen ``context``/defines surface, not a BSL construct)
     - **Territory port train, Task 3 — provably uniform, transcribed
       directly rather than built as an override.** The frozen
       ``_process_eviction_pipeline`` reads ``context.get(
       "displacement_mode", DisplacementPriorityMode.EXTRACTION)``, but
       no production call site sets ``displacement_mode``
       (grep-verified across the engine at
       port time), so the override branch is DEAD on every real run —
       the same "provably uniform" test Dispossession's own D-1 applies
       before collapsing a per-call parameter to a constant. **Mechanism,
       corrected (fix round, 2026-08-12) — the "never sets" claim above is
       right, but not for the reason an earlier draft of this row gave.**
       ``TickContext.get``'s own default is never reached at all:
       ``TickContext.__contains__`` (``context.py:96``) returns ``True``
       unconditionally for the literal key ``"displacement_mode"``,
       set or not, and ``__getitem__``
       (``context.py:67-68``) returns ``self.displacement_mode`` — which
       defaults to ``None`` — for that same key, so ``get``'s
       ``try: return self[key] except KeyError: return default``
       (``context.py:110-112``) never raises and never returns its own
       ``default`` argument. ``mode`` is ``None`` on every
       production call as a result, not the ``EXTRACTION`` the ``.get(...,
       DisplacementPriorityMode.EXTRACTION)`` call SITE visually suggests.
       EXTRACTION is actually delivered one line later, by
       ``_PRIORITY_BY_MODE.get(mode, self._PRIORITY_BY_MODE[
       DisplacementPriorityMode.EXTRACTION])`` (``territory.py:166-168``):
       ``None`` is not a key in ``_PRIORITY_BY_MODE`` (which holds only
       ``EXTRACTION``/``CONTAINMENT``/``ELIMINATION``), so THIS ``.get``
       falls to ITS OWN default — the same conclusion, a different
       mechanism. This pack transcribes the EXTRACTION priority order
       (``PENAL_COLONY > RESERVATION > CONCENTRATION_CAMP``) directly,
       with no BSL construct for the other two modes (``CONTAINMENT``,
       ``ELIMINATION``) or the unimplemented ``AUTO`` mode. The override
       machinery itself, ``defines.yaml:243``
       (``displacement_priority_mode``), the fourth ``AUTO`` member that
       dead-ends at "not implemented in Sprint 3.7.1" in the frozen
       source (``src/babylon/models/enums/territory.py:107,115``), AND
       the four AUTO-mode threshold defines this row's earlier draft
       missed — ``defines.yaml:244`` (``elimination_rent_threshold``),
       ``:245`` (``elimination_tension_threshold``), ``:246``
       (``containment_rent_threshold``), ``:247``
       (``containment_tension_threshold``), confirmed read by NO
       ``TerritorySystem`` code path (grep-verified) — are all
       WS1-ledgered together: #502's Events-in-BSL/mode-override
       workstream owns reviving them if a future spec ever makes the
       override reachable. NO ``TerritorySystem`` code path reads
       ``defines.yaml:241`` (``clarity_profile_coefficient``) at all
       (verified) — it belongs to a separate fog/clarity estate —
       deliberately not a ``defconst`` in this pack, not WS1-ledgered
       (nothing here to revive).
   * - D130
     - §2.5, §2.6, §2.7, §2.10
     - **Resolved (Territory port train, Task 6) — P6, the graph-threading
       gap ADR197 recorded but left open.**
       ``rule_pipeline::resolve_expr_bindings`` unconditionally passed
       ``graph: None`` to the ``EvalEnv`` it built for a ``:expr``
       binding, since no rule landed before this train needed a query
       form (``exists``/``fold``/``select-*``/``field-of``/…) INSIDE a
       ``:expr`` binding's own expression — every query form in every
       prior pack lived directly in a guard or an effect operand, both of
       which already carried the real, graph-bearing environment.
       ``territory/p3-spillover``'s ``inflow`` binding is the first
       counter-example: its exists-guarded pull-side fold over ADJACENCY
       neighbours COULD inline directly in the effect's ``(set …)``
       operand instead, which already carries the graph — the real §4.5
       argument for an ``:expr`` binding here is narrower than "has to
       be": inlining would materialize the
       neighbour query four times (``raw`` is referenced twice in the
       clamp ``if``, and ``inflow`` itself holds two query forms — the
       ``exists`` guard's materialization and the ``fold`` body's),
       where the binding materializes it twice. ``:expr`` is the only
       shape that names the intermediate once. **Resolved by threading, not by relocating the
       query out of the binding**: ``resolve_expr_bindings`` gained a
       ``graph: Option<&dyn GraphSubstrate>`` parameter, threaded
       alongside the already-threaded ``types``/``enums`` registries
       (never alone — the PR A verifier fix round's own discipline for
       this exact ``EvalEnv``, closing the ``Option``-``None`` hazard
       class by construction rather than by coincidence a second time).
       ``tick.rs::collect_pass`` passes ``Some(graph)`` from its own live,
       read-only Pass-1 borrow; the two pure-expression R9-chapter
       callers (arithmetic-only conformance vectors building no
       substrate at all) pass ``None``.
   * - D131
     - §3.4
     - **Territory port train, Task 3 (fix round, 2026-08-12) — a
       typecheck-forced storage choice, filed after the fact: the
       original fixture landed the choice without a register row and
       mis-cited D111/Q9 for it.** ``territory-conformance.bscn`` declares
       ``territory/heat`` ``EXTENSIVE``, not ``intensive`` — the kind a
       naive reading of the frozen system's per-node heat LEVEL would
       suggest. Forced by §3.4's aggregation law:
       ``typecheck.rs::typecheck_aggregation`` emits ``E-TYPE-041`` (this
       module's own ``TypeCode::SumOfIntensive``, ``typecheck.rs:50,70``)
       for ``sum`` over an intensive-kinded body, and
       ``territory/p3-spillover``'s pull-side fold body,
       ``(field-of it territory/heat)``, reduces to the bare qname
       ``territory/heat`` for exactly this check
       (``rule_pipeline.rs::field_ref_for``'s ``field-of`` arm,
       ``rule_pipeline.rs:657-661``, "the accessor carries the
       declaration's kind, identically to a ``:field`` binding") — so
       §3.4 sees the declared kind directly, not an opaque expression.
       The §3.4 EXEMPTIONS escape (``TypeEnv.exemptions``, which
       suppresses ``E-TYPE-041`` for a named field) is not
       content-reachable on the live pipeline: ``babylon-tick``'s
       ``prepare_rules`` hard-codes ``exemptions: &[]`` unconditionally
       (``lib.rs:129``), so no scenario or rule pack can opt a field into
       it today. Declaring ``EXTENSIVE`` is the only way to keep the fold
       loading. Precedent: ``query-lane-e2e.bscn:44-47`` (the
       query-evaluation train's own synthetic fixture) already made and
       documented this identical choice for the identical field, before
       the port started. **This is a storage choice the typechecker
       forces, not a ruling that heat IS extensive** — whether a
       per-territory heat LEVEL should aggregate as a summed extensive
       quantity at all is a separate, still-open question (the field-kind
       ruling D111/Q9 references is a DIFFERENT question — small-closed-
       field representation, not aggregation kind — and is itself
       DISCHARGED by this same port train, D102/ADR195/ADR196, not "still
       open" as an earlier draft of this fixture's own header claimed).
   * - D132
     - §4.2
     - **Production port train (issue #565, Task 5) — recorded, relying on
       D116, not a repair. CATCHES UP (fix round, 2026-08-12) with the
       producer-side push redesign D136 records: this row once described
       the SUPERSEDED pull-fold design (four rules, a
       per-territory fold reading ``social-class/production-value``
       directly) — corrected here to the FIVE rules and the THREE-stage
       dependency the landed pack actually has.** ``production/
       p0-production-total-reset`` < ``production/p1-direct-production`` <
       ``production/p2-employed-routing`` < ``production/p3-employed-
       fallback`` < ``production/p4-extraction-intensity``, the ascending
       rule-id byte order (§4.2, D16) this pack's own cross-rule dataflow
       needs. ``p0``'s own byte position is the MOST load-bearing of the
       five: it must run BEFORE ``p1``/``p2``/``p3`` or the
       ``territory/production-total`` accumulator compounds across ticks
       instead of resetting (mutation-verified, below). The dependency
       chain is THREE stages, strictly stronger than the two-stage one
       this row originally recorded: ``p0`` zeroes
       ``territory/production-total`` on every territory; ``p1``/``p2``/
       ``p3`` each push their own computed ``output`` — ``(add output)`` —
       onto the D45-tiebreak-selected territory ref (D135) their
       ``bio``/``max-bio`` bindings already compute; ``p4`` reads the
       result back via a PLAIN ``:field territory/production-total``
       binding — no fold, no filter (D138). Each stage depends on the
       PRIOR stage's already-applied writes from THIS SAME TICK, so
       every leg of the chain relies on D116, not just the p1-p3-to-p4 leg
       this row originally named. This is D116's cross-rule divergence
       (today's ``run_once_into``/``TickSession::advance`` run each rule
       in a content set to completion before the next starts, against the
       SAME graph) turned into a DEPENDENCY rather than fought — the same
       shape Territory's own D120 names for its own five-rule chain.
       **The end-to-end proof is necessarily a TWO-TICK test, not a
       single-tick one**: every input to this fixture's own computation is
       tick-invariant, so a single tick cannot distinguish "p0 resets
       correctly" from "p0 does nothing" — ``production_conformance.rs::
       p0_reset_keeps_extraction_intensity_stable_across_two_ticks`` is
       the actual end-to-end proof (``t-alpha``'s extraction-intensity,
       ``0.027692307692307697``, bit-identical across both ticks of a real
       ``TickSession``). Mutation-verified, not merely asserted: breaking
       ``p0``'s own reset effect makes the SECOND tick's value EXACTLY
       DOUBLE the first (``0.05538461538461539``) — the accumulator
       compounds instead of starting fresh, which is precisely what a
       byte-order violation of this row's own claim would produce. Phase
       boundaries become POSITION boundaries the moment the Q14 repair
       train (D116's own text) lands a real anchor registry; this row is
       that train's acceptance-criterion input for Production specifically
       — accuracy here is not optional, since a stale row would feed a
       wrong acceptance criterion into that future train.
   * - D133
     - N/A (frozen ``context``/defines surface, not a BSL construct)
     - **Production port train, Task 5 — provably dead, transcribed by
       omission (same class as Territory's D129, Dispossession's D-1/D-2).**
       The frozen ``effective_labor_power`` tensor-registry branch
       (``production.py:160-172``) reads ``territory_attrs.get(
       "fips_code")``, but ``Territory``'s real field is ``county_fips``
       (``territory.py:81-91``; ``resolve_county_identity``'s own
       docstring, ``graph_bridge.py:44-48``: "The county identity of a
       territory lives in its ``county_fips`` attribute and nowhere
       else") — ``fips_code`` is ``None`` on every live territory node, so
       the branch never fires on any scenario in the estate, confirmed
       independently by the scout dossier (``reports/production-bsl-
       surface-facts-2026-08-12.md`` §7) against the phase-1 inventory's
       own finding. ``effective_labor_power`` is always Computation-1's
       ``base_labor_power`` fallback. **Omitted from the pack entirely** —
       no BSL construct exists for an external keyed-cache lookup
       (``TensorRegistry.get(fips, year)``) in any case, so there is
       nothing to transcribe even as inert content.
   * - D134
     - §2.9, §3.1
     - **Production port train, Tasks 2-3 — the ``la_production``
       graph-scope channel, reformulated as an ordinary node field.** The
       frozen engine publishes ``la_production[node.id] = produced_value``
       (``production.py:129,194``) as a ``dict[str, float]`` on
       ``graph.set_graph_attr("la_production", ...)``, read back only by
       ``ImperialRentSystem`` (``economic.py:438,453``, keyed by worker
       node id — out of this port's own scope). No ``GraphSubstrate``
       graph-scope-scratch construct exists in either Rust crate, and the
       on-paper carrier-``NodeType`` route's own accessor, ``the``, is
       Slice-2 UNSERVED (``evaluator.rs:504-506``) — but the channel is not
       a genuine graph-level aggregate to begin with: every read is
       already keyed by a specific node, so it is per-node data wearing a
       graph-scope costume (scout dossier §6). This port declares
       ``social-class/production-value`` (``int extensive``) instead — an
       ordinary ``deffield``, written ``(set ...)`` by ALL THREE producer
       rules (p1/p2/p3), not just the employed branch the frozen ledger
       covers. The write WIDENS relative to the frozen ``la_production``
       (every producer role, not only LA-with-employer); the READ stays
       exactly as narrow as the frozen ``la_production.get(edge.target_id,
       ...)`` call already is — MINOR-B (fix round): this port's own
       ``p4`` is not that reader (D136's push redesign gave ``p4`` a
       plain ``:field territory/production-total`` binding — no fold, and
       it never reads ``social-class/production-value`` at all); the
       narrow WAGES-keyed read is the FUTURE ``ImperialRentSystem`` port's
       own job, out of this port's scope, since only LA workers ever carry
       an incoming WAGES edge for that future port to find. Side benefit: the per-node
       reformulation moves this channel INSIDE the qa:regression byte-
       gate's coverage (``graph_content_hash`` hashes node attributes,
       never ``g.graph`` metadata) — a correctness-relevant fact the
       frozen ``la_production`` channel never had (phase-1 inventory §7).
   * - D135
     - §2.7, D45
     - **Production port train, Task 2 — a comparison this port owes,
       resolved by NOT resolving it (same class as Territory's D124, both
       stand independently correct within their own systems).** The frozen
       ``_find_tenancy_target``/``_find_employer`` (``production.py:
       212-244``) return the FIRST match in ``query_edges`` iteration
       (insertion) order. This language's ``select-max`` tiebreak (§2.7,
       D45) is ascending-id byte-order FIRST-WINS. ``worker-pp-two-lands``
       (TWO TENANCY edges, to ``t-alpha`` then ``t-beta``) exercises the
       divergence directly for its OWN bio-ratio computation: this pack's
       ``select-max(..., 1)`` picks ``t-alpha`` (the lower NodeId, declared
       first) — the SAME territory the frozen engine's own first-
       insertion-order walk would pick here, since ``t-alpha`` is also
       declared/inserted first in this fixture, so the two engines happen
       to AGREE on this particular fixture's own tiebreak outcome (unlike
       Territory's own D124, whose fixture never exercised the divergent
       case at all — this one is LIVE but coincidentally non-
       discriminating, confirmed by measurement, not assumed). A fixture
       where the D45-winning edge is NOT the first-inserted edge would need
       to choose which engine's answer it pins; this fixture does not
       construct that case. See D136, which RECORDED a DIFFERENT,
       genuinely discriminating consequence of this same multi-tenancy
       topology — since RESOLVED there by a producer-side redesign
       (TRIVIAL, fix round), not left standing the way this row's own
       tiebreak comparison deliberately is.
   * - D136
     - §2.6, §4.2
     - **Production port train, Task 4 — RESOLVED (fix round, 2026-08-12):
       adversarial verification caught the original row's "no `.bsl`-level
       fix is available within port-as-is" conclusion as FABRICATED,
       corrected here rather than silently overwritten —
       the same "mechanism, corrected" discipline D129's own row uses for
       an earlier fix round's false claim.** The original OBSERVATION
       stands, independently re-confirmed: a territory-side PULL ``fold
       sum`` over ``(neighbors self EdgeType/TENANCY :in
       NodeType/SOCIAL_CLASS)`` reading ``social-class/production-value``
       double-counts a multi-tenancy producer (``worker-pp-two-lands``,
       two ``TENANCY`` edges) into EVERY territory it touches, where the
       frozen engine's ``territory_production[territory_id] +=
       produced_value`` (``production.py:200-204``) credits exactly one.
       **What was FALSE**: that fixing this would need a new field this
       port's port-as-is mandate does not license. The adversarial
       verifier built a scratch probe implementing a PRODUCER-SIDE PUSH
       design against the ALREADY-LANDED grammar — no new GRAMMAR
       construct at all (MINOR-A, fix round: the probe DOES mint a second
       field, ``territory/production-total``, below — minting it is
       LICENSED on D134's own precedent, the SAME precedent that already
       established ``social-class/production-value`` as a legitimate
       per-node field mint under port-as-is; denying that license is
       exactly what the original fabrication got wrong) — and it loaded,
       ran, and reproduced the frozen engine's own ``t-beta`` value
       (``0.009615384615384616``) bit for bit on first execution.
       **The resolution, landed:** ``territory/production-total`` (``int
       extensive``, seeded ``0`` on every territory) replaces the pull
       fold. A new rule, ``production/p0-production-total-reset``
       (byte-sorted before p1, subject TERRITORY, ``(when #t)`` —
       unconditional, the same idiom ``territory/p1-heat-dynamics`` uses),
       zeroes it every tick. ``p1``/``p2``/``p3`` each gain a THIRD effect
       writing to it — ``(update-node (select-max (neighbors self
       EdgeType/TENANCY :out NodeType/TERRITORY) 1)
       territory/production-total (add output))`` — the SAME tiebreak-
       selected ref (D135) their ``bio``/``max-bio`` bindings already
       compute, so a multi-tenancy producer's contribution lands on
       EXACTLY ONE territory, matching the frozen engine's single-territory
       attribution up to D135's own (here, non-discriminating) tiebreak.
       ``p4``'s own ``total`` binding becomes a plain ``:field
       territory/production-total`` read — no fold, no filter, D138's
       fold-body restriction never engaged by the LANDED design (D138's
       own row is corrected in turn to record this — it now explains why
       ``production-value`` was minted as a per-node field in the first
       place, not what ``p4`` does today). One semantic cost, honestly
       recorded: the new third effect's own ``select-max`` target ref
       lives in EFFECTS position and ABORTS on an empty candidate set
       (the same E-EVAL-021 class p2's WAGES ref already guards against),
       so ``p1``/``p2``/``p3`` each gain a THIRD ``when`` conjunct,
       ``(exists (neighbors self EdgeType/TENANCY :out
       NodeType/TERRITORY))`` — a tenancy-less producer, which USED TO
       fire with ``output`` forced to ``0`` by the existing ``active``-gate
       idiom (writing ``production-value`` to ``0`` every tick,
       hash-neutral), now does NOT FIRE AT ALL, so its ``production-value``
       goes STALE (holds whatever the previous tick left, or its seed)
       rather than resetting every tick. **This is a reasoned scope call,
       not an unexamined gap (verifier rider, fix round):** no fixture
       node exercises it today because the divergence needs BOTH a real
       reader of ``social-class/production-value`` — this pack has none;
       ``p4`` reads ``production-total`` only (D134/MINOR-B) — AND a
       ``TENANCY`` edge REMOVED between ticks, which nothing in this pack
       or the frozen engine ever does (``remove-edge`` is a served BSL
       structural verb, ``structural_verbs.rs::remove_edge``, but the
       frozen ``ProductionSystem`` itself calls no edge-removal method at
       all — grep-confirmed, zero hits). The divergence stays latent
       until a FUTURE ``ImperialRentSystem`` port becomes the real
       reader AND some other system starts removing ``TENANCY`` edges
       mid-game — though a stale value would perturb this crate's own
       goldens the moment such a scenario existed, D134's byte-gate side
       benefit cutting both ways; a future fixture combining both would need to choose
       which reading it wants. Measured,
       not assumed: ``production_conformance.rs::p4_extraction_matches_
       frozen_single_territory_attribution`` pins ``t-beta``'s extraction-
       intensity at ``0.009615384615384616`` — bit-identical to the frozen
       mirror, where the ORIGINAL landing's own pin
       (``0.01730769230769231``) was the double-counted divergence this
       row wrongly claimed had no available fix. Mutation-verified: breaking
       ``p0``'s reset (mutating its effect to a no-op) makes a second
       tick's ``t-alpha`` extraction-intensity EXACTLY DOUBLE the first
       (``0.05538461538461539`` vs ``0.027692307692307697`` — the
       accumulator compounds instead of resetting); restored
       byte-identical.

       **A grammar-forced narrowing, inert on the estate (MINOR-3, fix
       round).** ``neighbors``' mandatory fourth operand — a result
       ``NodeType`` annotation (D51, ``grammar.rs:642``: ``("neighbors", 4,
       4, "exactly 4")``) — means every ``(neighbors self EdgeType/TENANCY
       :out NodeType/TERRITORY)`` query this row's own push design uses is
       narrower than the frozen ``_find_tenancy_target``/``_find_employer``
       walks it transcribes: those iterate ``graph.query_edges(edge_type=
       ...)`` filtered ONLY by edge type and endpoint id, with no type
       filter on the OTHER endpoint at all. This narrowing is
       grammar-forced, not a design choice, and it is INERT on the actual
       estate — independently verified (not merely asserted): every
       ``WAGES`` edge creation site repo-wide connects two ``SOCIAL_CLASS``
       entities (``_legacy.py:139-142``, ``:430-433``;
       ``_legacy_wayne.py:510-513`` — grepped directly), so a truly
       untyped walk would never have found a different-typed candidate a
       typed one excludes. Declared here since this row's own query shape
       is where the narrowing would first bite were that ever untrue.
   * - D137
     - §1.5, §3.2
     - **Production port train, Task 1 — a fragility flagged at
       declaration, not yet triggered.** ``economy/base-labor-power-
       annual`` is declared ``(defconst economy/base-labor-power-annual
       1.0c)`` — a ``coefficient`` literal at its OWN domain's exact
       boundary. ``economy.base_labor_power``'s real Pydantic domain is
       ``[0.0, ∞)``, unbounded above (``economy_basic.py:169-173`` — the
       ``Field(...)`` call itself starts at ``:169``, not ``:168`` (that
       line is the preceding comment); TRIVIAL-1, fix round —
       ``default=1.0, ge=0.0``, no ``le`` — independently re-read, not just
       cited from the scout dossier §10), while the BSL ``coefficient``
       type's own domain is the closed ``[0,1]`` (``E-LEX-024``,
       ``reader.rs:171`` — ``UnitIntervalOutOfRange``, "a p/i/c literal
       outside [0, 1]"). At the define's DEFAULT value (``1.0``), the
       literal ``1.0c`` is exactly at the closed interval's upper bound —
       legal today, LOUDLY refused at load the moment a modded
       ``defines.yaml`` raises ``base_labor_power`` above ``1.0`` (no
       silent clamp, no silent wraparound — a load-time ``E-LEX-024``).
       Retires the same way Territory's own D122/``metabolism.bsl``'s
       ``entropy-factor-x1e6``-class scaled-Int escape hatches do: when
       #502 workstream 3 (post-port refactor program) lands a genuine
       ``Real x Ratio`` operator or unbounded-domain coefficient storage.
   * - D138
     - §3.4
     - **Production port train, Task 4 — the scout dossier's own headline
       correction (``reports/production-bsl-surface-facts-2026-08-12.md``
       §5), transcribed into the register. CORRECTED (fix round,
       2026-08-12, MINOR-1): this row's own account of what ``p4`` does is
       now HISTORY, not the live design — D136's producer-side push
       redesign discharges the fold entirely, so this row's own
       ``field_ref_for`` restriction is no longer what ``p4`` works
       around; the restriction explains why ``social-class/production-
       value`` (D134) exists as a per-node, already-filtered field in the
       FIRST place, before the push design existed.** A naive per-
       territory ``fold sum`` over ``(neighbors self EdgeType/TENANCY :in
       NodeType/SOCIAL_CLASS)`` reading a neighbour's ``population`` or
       ``role`` directly would need to FILTER which neighbours count (only
       the two producer roles, ``PERIPHERY_PROLETARIAT``/
       ``LABOR_ARISTOCRACY``) — but a fold body may not be a compound
       (conditional) expression at all: ``rule_pipeline.rs::field_ref_for``
       (§3.4) reduces a fold body to a declared field's kind through
       exactly THREE shapes (a bare ``<qname>``, a ``field-of`` accessor,
       or a nested fold), returning ``None`` — the loud, uncoded
       ``compound_fold_error`` — for anything else, including an
       ``if``-based role filter. No ``(fold sum <query> <body> :when
       <predicate>)`` form exists, and ``:weight`` goes through the
       identical restriction, so neither can smuggle a computed 0/1
       eligibility flag. The flagship scenario's own ``TENANCY`` topology
       is not role-restricted (``comprador``, a COMPRADOR_BOURGEOISIE
       tenant of ``t-alpha``, is the fixture's own filter-honesty witness),
       so this restriction was load-bearing for the ORIGINAL pull-fold
       design, not academic. **What that original design resolved: the
       filter moved out of the fold body entirely**, onto the per-node
       ``social-class/production-value`` field p1-p3 already compute and
       filter via their OWN ``when`` guards (role-scoped, computed once per
       subject, no fold involved). **What the fix round's push redesign
       does with that same field: reads it via a plain ``:field`` binding
       inside ``territory/production-total`` (D136) — no fold anywhere in
       this pack at all**, so this row's own restriction never engages the
       LANDED ``p4`` in either direction; it remains true and load-bearing
       only as the reason a per-node field was the right shape to begin
       with. A fresh mutation proves the role/active filter's own load-
       bearing status against the CURRENT (push) design, not merely
       carrying the claim over from the original: widening ``p1``'s
       ``when`` guard to admit ``COMPRADOR_BOURGEOISIE``, re-run against
       the landed push design, flips ``production_conformance.rs::
       p0_p1_p3_push_t_alphas_production_total_correctly`` red (``t-alpha``'s
       ``production-total`` inflates from ``comprador``'s own push);
       restored byte-identical.
   * - D139
     - §2.9, §3.4
     - **T2 (issue #559), Task 2 — the D32 implicit-strength seeding is
       WIRED, and its trigger is a deliberate NARROWING of D32's literal
       text.** The implicit ``<edge-type>/strength`` registry
       (``declarations.rs::FieldRegistry::with_implicit_edge_strength`` —
       fully built and tested since the R9 chapters, zero production
       callers until now) gained its first real caller:
       ``babylon-tick::prepare_rules``'s ``TypeEnv`` construction seeds one
       ``<edge-type>/strength`` row per ``defvocabulary EdgeType`` member
       the scenario declares — keyed off ``scenario.vocabulary``, NOT
       ``scenario.edge_types`` (the hydration census the scout dossier's
       own text pointed at) — and refuses an explicit ``deffield``
       re-declaring an implicit field as ``E-LOAD-001`` (D32's own named
       violation, checked at the seeding site because ``scenario.rs``'s
       simpler ``load_deffield`` has no notion of "implicit" to check
       against). RULED, recorded honestly as a divergence rather than
       smoothed over: seeding fires ONLY under a declared
       ``(defvocabulary EdgeType …)`` block — a scenario declaring edges
       with no ``defvocabulary`` at all gets NO seeding
       (``scenario.vocabulary`` is ``None``, the opt-in-per-scenario
       contract) — which NARROWS D32's unconditional "needs no
       ``deffield``". Chosen for reuse of the already-tested
       ``FieldRegistry`` machinery and the Phase-2 content-pack-registry
       direction (``declarations.rs``'s own module doc), not because a
       ``FieldDecl`` could not be built from the census directly (it
       trivially could). The BARE accessor
       (``field-of it <edge-type>/strength``, no aggregation) still works
       without the declaration (``tick::bind_field_value``'s
       unregistered-field fallback); only an UNWEIGHTED FOLD/aggregation
       needs the seeding (``typecheck.rs::resolve_field`` hard-fails an
       unknown field) — ``edge-lane-e2e.bscn``'s own
       ``(defvocabulary EdgeType (SOLIDARITY))`` plus its Shape-1
       ``(fold sum (edges …) (field-of it solidarity/strength))`` vector is
       the requirement's positive, executed proof.
   * - D140
     - §2.6
     - **T2 (issue #559), Task 3 — ``Element``'s cross-kind Ord RULED:
       ``Node`` sorts before ``Edge``, by declaration order.** §2.6 defines
       a total order WITHIN each query kind's own result set only — it is
       silent on comparing a ``Node`` to an ``Edge``, and no production
       ``materialize()`` call ever mixes kinds (``edges`` returns only
       ``Edge`` elements; ``nodes``/``neighbors`` only ``Node``), so the
       cross-kind order is UNREACHABLE in production by construction —
       pinned anyway, per the enum's own standing instruction (CT4P A5,
       issue #525): an arbitrary but DELIBERATE, DOCUMENTED, TESTED choice
       (``query.rs::tests::node_sorts_before_edge_regardless_of_id``
       value-pins kind-dominates-value), never silently inherited from
       wherever ``#[derive(Ord)]``'s declaration order happens to put a new
       arm. Companion ruling at the same landing: ``Element`` drops
       ``Copy`` (``EdgeKey`` owns a ``String``); the nine Copy-dependent
       call sites (five ``for &element in …`` loops, one index-and-move,
       three ``to_value()`` receivers resolved by the ``&self`` signature
       change) were fixed by enumeration, plus one test-module
       exhaustiveness arm the enum's own compile-time trip-wire
       (``element_kind_name``) forced — exactly the conscious-decision
       mechanism that trip-wire exists to fire.
   * - D141
     - §2.10, §2.9
     - **T2 (issue #559), Task 1 — ``GraphSubstrate::edge_attribute`` takes
       the FULL QNAME, checks ONLY the ``/strength`` suffix, and NEVER the
       owner segment.** The one new substrate method mirrors
       ``node_attribute``'s own convention exactly: ``attribute`` is the
       full qname (e.g. ``"solidarity/strength"``), never a bare segment.
       What IS checked: the qname must END IN ``/strength`` — T2 stores
       exactly one value per edge (D32's implicit field, already hashed in
       ``CanonicalState`` section ``0x03``), so any other edge-owned qname
       reads as "never written", the SAME ``GraphError`` shape a
       never-written node field gives — which buys ``field_of_edge`` a
       zero-special-casing ``E-EVAL-033`` unification (declared-but-
       unstored and never-written degrade identically). What is NOT
       checked: whether the qname's OWNER segment actually names
       ``edge_type`` — deliberately, exactly as ``node_attribute`` performs
       no ownership check of its own; that half of §2.10 discipline 1 is
       the CALLER's obligation (``field_of_edge``'s
       ``check_edge_referent_type``, upstream of every call), the same
       division of labor ``node_attribute``/``check_node_referent_type``
       already have — pinned as deliberate by the shared conformance row
       ``edge_attribute_does_not_check_the_owner_segment`` (both backends)
       so a future reader does not "fix" the suffix check into an owner
       check by surprise. The suffix test is a plain string test, not a
       call into ``babylon-bsl::vocabulary::render_member`` —
       ``babylon-graph`` sits BELOW ``babylon-bsl`` in the crate graph and
       cannot import it. T3 (ADR198 R1) must PRESERVE this contract when it
       widens storage: the suffix check widens to a real
       per-``(edge, qname)`` lookup, ownership validation stays the
       caller's job, and the SIGNATURE does not change.
   * - D142
     - §2.10
     - **T2 (issue #559), Task 7 — the ``the`` disposition: NOT folded into
       T2; its own micro-train (T2.5), because serving it surfaces an
       unscoped load-pipeline design gap.** ``the``'s spec-normative
       load-time legality gate — ``E-LOAD-043`` (declared ``:ceiling``
       other than 1) and ``E-LOAD-045`` (no manifest row) — is implemented
       ONLY in ``babylon-bsl/src/manifest.rs``'s
       ``Manifest``/``check_rule_against_manifest``, exercised ONLY by
       ``tests/r9_chapters.rs``'s own test-only harness:
       ``rule_pipeline.rs`` and ``babylon-tick/src/lib.rs`` reference
       ``manifest``/``Manifest`` NOWHERE outside comments (re-executed at
       this row's landing: three comment hits,
       ``rule_pipeline.rs:339,356,384``, zero calls; zero hits at all in
       ``babylon-tick``'s driver). The production driver builds
       ``CardinalityCeilings`` straight from the scenario's own hydrated
       node/edge COUNTS, never from a declared ``(manifest …)`` form's
       ``:ceiling`` row — so ``the``'s legality check is NOT REACHABLE
       through ``run_once_into`` at all today, in either direction (no
       check fires, and no declared-ceiling concept exists to check
       against). Serving ``the`` for real therefore needs a design decision
       T2's dyadic-edge-focused charter does not license: either (a) wire
       the ``(manifest …)`` form into the load pipeline — new surface,
       non-trivial — or (b) redefine ``the``'s load-time legality against
       the ALREADY-wired census number — a real semantic choice (the two
       numbers coincide today by accident and diverge the moment a scenario
       declares a larger-than-hydrated ceiling). That choice deserves its
       own scoped review: issue #572 stays OPEN for the T2.5 micro-train,
       with ADR201 as the evidence record — a stronger reason than #572's
       original framing, which reasoned only from ``the``'s lack of
       technical dependency on ``EdgeKey``.
   * - D143
     - §2.8, §2.9
     - **T3 (issue #560), PR A — the double-storage ruling: an
       ``update-edge`` against ``<edge-type>/strength`` writes the EXISTING
       strength slot, never a fifth-section row.** ADR198 R1 rules full
       symmetric edge attributes and R2 rules the fifth canonical section
       holding them, but no ruling text named where a ``/strength`` WRITE
       lands: strength already lives in ``add_edge``'s mandatory
       ``:strength`` operand and hashes in ``CanonicalState`` section
       ``0x03``, so an ``update-edge … solidarity/strength (scale 0.95c)``
       (§2.10's own worked example) minting a ``0x05`` row would give one
       datum two hashed homes, free to diverge. RULED:
       ``GraphSubstrate::update_edge`` routes on the attribute SUFFIX — a
       qname ending in ``/strength`` overwrites the edge's existing
       strength slot (the write is a replacement, never a mint; a strength
       write against a nonexistent edge is loud, never a mint); any other
       qname writes the fifth-section store. The fork keys on the suffix
       ONLY — exactly the division D141 pinned for the read side: whether
       the qname's owner segment names the edge's type stays the caller's
       obligation (``check_edge_referent_type`` upstream), never the
       substrate's. Pinned by the shared conformance row
       ``update_edge_against_strength_writes_the_existing_slot_never_a_fifth_section_row``
       on both backends, including the hash half: a store whose strength
       was written via ``update_edge`` encodes byte-identically to one
       whose strength was set at ``add_edge`` time.
   * - D144
     - §2.9
     - **T3 (issue #560), PR A — the fifth canonical section's shape,
       RULED: tag ``0x05``; entries ``str type ‖ u64 from ‖ u64 to ‖ str
       qname ‖ u64 value-bits``; ascending ``(type, from, to, qname)``;
       ELIDED WHEN EMPTY; and the hash contract is VERSIONED from this
       train.** ADR198 R2 fixed the what (a fifth section, hashed only
       when at least one edge attribute exists) and left the layout to
       the implementing train. The entry layout mirrors section ``0x02``'s
       ``(id, name)`` with the edge's ``(type, from, to)`` identity in
       place of the node id; the sort key is the ruled mirror of every
       other section's. The elision asymmetry R2 ordered recorded:
       sections ``0x01``–``0x04`` write UNCONDITIONALLY — a graph with
       zero hyperedges still writes ``0x04 ‖ 0x00000000``, version-1
       behavior left unchanged, because retrofitting elision onto ``0x04``
       would itself move every existing hash — while section ``0x05``
       writes ONLY when non-empty: elision is what makes the widening
       byte-compatible, and every FUTURE section follows the ``0x05``
       rule. Versioning had no anchor before this train (the
       canonical-layout doc carried no version marker), so T3 minted the
       convention: ``CANONICAL_LAYOUT_VERSION = 2`` in
       ``babylon-graph/src/state_hash.rs`` — never written into the
       encoding (the bytes stay self-describing by tag); it versions the
       CONTRACT for cross-language reimplementations, and a pinned unit
       test (``the_canonical_layout_version_is_pinned``) makes any bump a
       deliberate, reviewable act. The layout is byte-pinned by
       ``the_fifth_section_is_pinned_byte_for_byte``; the elision is
       behavior-pinned by
       ``an_empty_edge_attribute_listing_writes_no_fifth_section_bytes``
       plus the four pre-existing pin sites left byte-identical,
       untouched.
   * - D145
     - §4.2, §2.8
     - **T3 (issue #560), PR B — ``PendingWrite``'s target is a SUM TYPE
       (``WriteTarget::Node | WriteTarget::Edge``), not a parallel
       edge-write batch: the collected batch's application law settles the
       widening shape.** ADR198 R3 charters ``update-edge`` "through the
       SAME ``PendingWrite`` collect-then-apply machinery", and the
       pre-T3 type keyed every write to a ``NodeId``, so the charter
       forces a widening — but the batch's own documented algebra picks
       the shape: the batch is the free monoid on writes and its
       application is a monoid ACTION that may not be reordered
       (``Add``/``Scale`` do not commute — the CT4P B1 paragraph). A
       parallel ``Vec<PendingEdgeWrite>`` joined at apply time — the only
       alternative considered — cannot represent "node write, then edge
       write, then node write" in collection order: two batches interleave
       only under an additional merge rule, and any merge rule is a
       reordering the application law forbids. The sum-typed target keeps
       ONE flat batch whose order is meaningful data, unchanged. The
       referent type is T2's ``EdgeKey``, shared unmodified (D36's "the
       two trains share the type"). The enum guards
       (``enum_write_value``/``refuse_arithmetic_on_enum_field``, §2.13)
       and §3.3's ``store_range_check`` ride unchanged — R3's "enum set
       included" is a call-site addition, exactly as the guard's own doc
       anticipated — with the verb label threaded into their diagnostics
       so a refusal names the form that raised it. Execution proof:
       ``update_edge_add_accumulates_at_apply_time_across_writes`` (the
       D104 carrier-accumulation law on an edge target, mutation-verified
       by skipping the CURRENT read) and the two run_once_into e2e vectors
       (``edge_write_lane_e2e.rs``). The §6.2 chapter-C2 vector family is
       served MINUS its I.15 leg: no E-EVAL-030 can fire because the I.15
       edge-mode machine is a declared Phase-2 gap no ruling charters
       here — recorded in ADR205, not improvised.
   * - D146
     - N/A (frozen-engine divergence by construction, not a BSL construct)
     - **W10 class-surface ternary port (issue #588), Tasks 3-4 — the
       RE-POINTED ACCUMULATOR (the headliner): the (r, l, f) ternary is
       stored and updated directly on the class node** — ``r += Δr``,
       ``l += Δl`` (APPLIED — the frozen engine discards Δl at the class
       call-site, ``engine/systems/ideology.py:394``), ``f += Δf·(1 −
       suppression)`` — with closure by a verbatim
       ``normalize_to_simplex`` transcription
       (``formulas/consciousness_routing.py:373-409``) replacing the
       frozen per-axis ``min(1,·)`` clamps (``ideology.py:410-411``). The
       cc/ni estate and its read-time bridge
       (``projection/aggregation.py:86-98``) are retired (ADR204 W1/W11).
       Trajectories diverge from frozen BY CONSTRUCTION; the conformance
       oracle is the dual implementation
       (``rust/crates/babylon-tick/content/scenarios/consciousness_ternary_conformance.py``,
       mirrored binding-order term-for-term and pinned bit-exactly by
       ``rust/crates/babylon-tick/tests/consciousness_ternary_conformance.rs``),
       not frozen floats
       (ADR183: the frozen engine is a structure/ordering contract, not a
       byte oracle).
   * - D147
     - N/A (frozen-formula surface — ADR202 R7's retirement, not a BSL
       construct)
     - **W10 (issue #588), Task 3 — the Curve-5 Gaussian is NOT
       transcribed; the linear chauvinist pass-through IS.** The
       wage-balance agitation MAGNITUDE component (the frozen Gaussian at
       ``formulas/sustained_exploitation.py:198``) is absent — not
       zero-stubbed; the magnitude-only E/P/S partition replacement rides
       #491. R7's site is the Gaussian ONLY (Director flag 2, ruled
       2026-08-15): the separate linear term inside the routing law —
       ``chauvinist_pressure = max(0, balance) ·
       chauvinist_pressure_scale`` (``defines.yaml:228``; the
       Emmanuel/MIM direction content, ADR016 untouched) — is transcribed
       verbatim.
   * - D148
     - N/A (frozen graph-attribute surface, not a BSL construct)
     - **W10 (issue #588), Task 3 — ``wage_deterioration`` is stubbed
       ``0.0c``:** the frozen engine reads last tick's
       ``opposition_states["wage"]`` graph attribute
       (``ideology.py:153-157``), and a graph-scope attribute register has
       no BSL surface (the archaeology digest's gap 5). The port declares
       ``consciousness/wage-deterioration-stub 0.0c`` — a content-visible
       stub const, never a hidden default.
   * - D149
     - N/A (frozen electoral-register surface, not a BSL construct)
     - **W10 (issue #588), Task 3 — ``popular_front_suppression`` is
       stubbed ``0.0c``:** the frozen engine reads the electoral register
       one tick stale (``ideology.py:401-409``); the register is absent in
       the ported estate. Exact under register-absent content — the
       frozen's own :401-409 note: absent ⟹ 0.0 ⟹ the bit-for-bit
       pre-U12 arithmetic.
   * - D150
     - N/A (frozen services-side buffer write — no ported consumers; not
       a BSL construct)
     - **W10 (issue #588), Task 3 — the ``material_conditions`` buffer
       write is not ported** (``ideology.py:424-437``:
       ``exploitation_visibility``, ``reification_buffer``,
       ``working_day_modifier``). No ported consumer reads these
       channels; each lands with its consumer system's own train.
   * - D151
     - §2.6, §4.2
     - **W10 (issue #588), Task 3 — solidarity pull→push redesign (the
       D136 fix-round pattern), with three recorded narrowings.** Exact
       vs the frozen fold-sums (``ideology.py:337-356``): each SOLIDARITY
       edge is pushed exactly once by its unique source. Narrowing 1:
       per-(source, target) multi-edge content would sum per-neighbor
       here vs per-edge in the frozen engine — but that content is
       UNREPRESENTABLE in the ported estate: both ``GraphSubstrate``
       backends key edges on ``(type, from, to)`` and refuse a duplicate
       loudly (``hypergraph_store.rs:220-226``, ``memory.rs:177-183``),
       so the divergence is unreachable by construction — recorded so no
       future reader treats the frozen per-edge fold as semantics to
       restore. Narrowing 2: the frozen class-sourced
       arm's ``strength <= 0`` skip (``ideology.py:343-344``) is NOT
       transcribed — inert on declared content, but a ≤0-strength edge
       would subtract here where frozen skips. Narrowing 3: the frozen
       incoming-WAGES ``value_flow`` fold-sum (``ideology.py:299-309``)
       narrows to the declared class-side ``social-class/wages-received``
       flow value — exact for single-employer content — because
       scenario-side edge-attribute seeding is unserved
       (``load_edge`` is strength-only, ``scenario.rs:1296-1302``); the
       seeding gap is filed as #591 item 3. Narrowing 3 itself was
       RETIRED by Train B item 3 (issue #591, PR B): ``social-class/
       wages-received`` is deleted; the wage flow now rides seeded WAGES
       edges' ``wages/value-flow`` (D156's ``(edge-attr ...)`` form,
       which closed the loader-side half of this gap) pushed into
       ``social-class/wages-inbox`` by the new ``consciousness/
       p2-wages-push`` rule — this row's narrowing no longer holds as
       live law.
   * - D152
     - N/A (a gate read re-pointed at the stored ternary — the W1
       unification re-home, not a BSL construct)
     - **W10 (issue #588), Task 3 — the class-source percolation gate
       re-pointed:** the frozen gate reads the source's
       ``class_consciousness`` (the cc axis, ``ideology.py:339-356``); the
       port gates on the source's revolutionary share — the same quantity
       post-W1 unification, re-homed to the stored ternary field
       ``social-class/revolutionary``.
   * - D153
     - §3.5
     - **W10 (issue #588), Task 3 — positioned-only agitation:**
       ``consciousness/p5-agitation``'s guard is anchored ∧ positioned
       (the ternary sum-guard over the declared ``:optional :default
       0.0p`` bindings). The frozen step accumulated agitation on every
       ACTIVE class (``ideology.py:206-208``'s active check only); under
       the port's L-ABS law an unpositioned class never accumulates —
       absence is not organization.
   * - D154
     - §4.2
     - **W10 (issue #588), Task 3 — same-tick closure heal observed
       (D116 reliance):** ``consciousness/p6-route``'s remainder branch
       heals a simplex defect (sum < 1 − eps) by assigning the remainder
       to liberal THIS tick, and ``consciousness/p8-dominant-worldview``'s
       readout reflects the healed ternary the same tick (D116's
       cross-rule same-tick visibility). The conformance world's
       tv-tie-all-true (sum 0.999999) is healed ``l += 1e-6`` before its
       LIBERAL readout — lawful A-001 behavior, pinned bit-exactly by the
       conformance test.
   * - D155
     - §3.1, §2.9, §1.5
     - **Train B item 6 (issue #591) —** ``real`` **is the eighth
       declarable** ``<type-name>``**:** a ``deffield``/``metric``
       ``:type`` spelling the finite ``binary64`` lane as a STORED type,
       carrying no declared-domain range law — §3.3's ``E-EVAL-020``
       stays the three unit-interval types' own check, and
       ``store_range_check``'s ``matches!`` is deliberately NOT widened.
       Seed law: int / ``p``/``i``/``c`` / ``r`` literals accepted, each
       already lex-bounded in its own lane (int exact to 2⁵³,
       ``p``/``i``/``c`` ``[0,1]`` at lex, ``r`` ``(0, ∞)`` at lex),
       converted by the crate's one scaled-literal contract
       (``unscaled / 10^scale`` — a basic IEEE-754 op, reproducing
       bit-identically); ``$`` refused (the typed-storage deferral every
       arm states). Write law: any finite ``binary64`` lands verbatim —
       no range check, never a clamp; negative writes are lawful even
       though no negative fractional seed literal exists. There is NO
       arbitrary-precision fractional literal: ``E-LEX-021`` still
       refuses bare floats — #591 item 5's territory, deliberately not
       this train's. A knowing supersession of D94's sealed closure
       (already seven rows by D101) and of D98's "``Real`` is not
       storable": both rows stand unedited (Immutability of History);
       this row is the correction, and §3.1 carries the current-law
       paragraph beside them. Not new mathematics — §3.3/§4.3 already
       name the binary64 lane; ``real`` mints no new type, only a
       declaration spelling for what the store already does (Amendment
       AE clause (ii), representation-level). Byte-neutrality: no
       committed content declares ``real`` yet, and type tags are never
       hashed, so every existing golden passes unedited. (Written at
       the mint; the train's Task-2 migration in the same PR then
       re-typed the primary conformance scenario of each of six named
       packs (production ``wealth``, vitality ``wealth``, lifecycle
       ``pop-p``, metabolism ``biocapacity``, dispossession
       ``dispossession-intensity``, consciousness ``agitation``/
       ``wage-balance``/``solidarity-inbox``) to ``real`` — every golden
       still passes unedited, carried by the same mechanism: type tags
       are never hashed. **Scope correction (final whole-branch review,
       2026-08-17, item 5):** this migration was ROSTER-scoped, not a
       full sweep of every declaration site — the digest that drove it
       indexed write sites per rule pack, and each pack has several
       scenarios, so roughly 15 sibling scenarios of these same six packs
       still declare the identical qnames ``int`` while their own tests
       pin non-integral values bit-exactly (e.g.
       ``metabolism-rounding-divergence-conformance.bscn:18-19`` declares
       ``territory/biocapacity``/``territory/max-biocapacity`` ``int``
       while its own test pins ``1.4``/``99.985``). Nothing behaves
       wrongly — ``BslType`` tags are never hashed regardless — but
       completing the sweep across every declaration site (~26
       byte-neutral edits) is a PARKED option pending a Director look,
       not something this train or its immediate follow-up executed.)
   * - D156
     - §3.9, §4.6
     - **Train B item 3 (issue #591) — the** ``.bscn`` **dialect's**
       ``(edge-attr <EdgeType/MEMBER> <from-name> <to-name> <field-qname>
       <value>)`` **form:** edge-attribute seeding in the scenario loader,
       closing the gap D151's narrowing 3 filed (``load_edge`` was
       strength-only). §3.9's hydration contract gains clause 7 with the
       whole law; this row records the decisions. The form list lives here
       rather than in D93's row, which stands unedited (Immutability of
       History — its list was already illustrative, naming neither
       ``defconst`` nor ``defenum``/``defvocabulary`` when those landed).
       **The quadruple key:** ``(edge-type, source, target, field)`` is a
       KEY exactly as D73's triple is; a scenario seeding one twice is
       ``E-LOAD-057`` (§4.6's contiguous ``E-LOAD`` overrun block, the next
       free number at allocation). **The strength refusal is
       unconditional,** not merely the deffield-registry miss: D32's
       implicit ``<edge-type>/strength`` field is never in
       ``scenario.fields``, but ``load_deffield`` would ACCEPT an explicit
       re-declaration (only ``prepare_rules``'s ``E-LOAD-001`` refuses it,
       later — D139), and without the guard the write would fall into the
       substrate's ``/strength`` fork (D143) and silently rewrite the mint
       strength slot. One datum, one writer: the ``edge`` form's own 4th
       slot. **The top-to-bottom edge-existence law:** the ``(member,
       from, to)`` triple must already be in the scenario's own seeded set
       — an attribute write never mints an edge, mirroring
       ``update_edge``'s never-mint-a-row discipline (§2.8). The qname's
       owner segment must name the edge's own type — §2.10 discipline 1 at
       hydration, through the same ``namespace_to_node_type`` rendering
       ``check_edge_referent_type`` uses at evaluation; no scenario-side
       owner check existed before this form (node-side fields are their
       owner's by construction of the ``node`` form's subject), so this
       check is stated here explicitly. The value converts through
       ``attribute_value``, the crate's ONE per-type literal law —
       ``Currency`` refused identically; its error-descriptor parameter
       carries the noun AND its quoting from the call site ("node \`…\`" /
       "edge (… → …)"), with the family's twelve format strings naming
       ``{local}`` bare — so no refusal calls an edge a "node", and the
       node path's emitted bytes are the pre-form rendering EXACTLY, one
       wording apart: the unreachable defense-in-depth arm's "node
       attributes" generalizes to "attributes" (correct for both element
       kinds). The byte-identity is pinned executably
       (``a_fractional_seed_into_an_int_field_is_refused_with_the_pinned_verbatim_message``,
       a full-string assertion — added in fix round 1, after the descriptor
       refactor's two-backtick drift escaped review behind a comment-only
       quote at ``consciousness-ternary-conformance.bscn:101-106``). Byte-neutrality: additive grammar — no
       committed scenario uses ``edge-attr``, so every committed scenario
       parses identically and every golden passes unedited.
   * - D157
     - §2.13
     - **Train B item 4 (issue #591) — scenario-declaration sharing via a**
       ``load_scenario_with_prelude`` **prelude, plus** ``EnumRegistry::
       declare``'s **identical-recognition arm.** A **declaration prelude**
       is a source string holding ONLY the four declaration top-forms
       (``defenum`` / ``defvocabulary`` / ``defconst`` / ``deffield``) — no
       ``(scenario …)`` wrapper, and never ``node`` / ``edge`` / ``edge-attr``
       (a prelude declares, it never seeds a graph; any other head is a
       loud refusal naming it). ``load_scenario_with_prelude(prelude_src,
       scenario_src, graph)`` reads the prelude first, THEN the scenario,
       against the SAME registries — a scenario field/node/edge resolving a
       prelude-declared type sees it exactly as if the scenario had
       declared it itself. Companion driver seam:
       ``run_once_with_prelude(scenario_src, prelude_src, rule_src)`` /
       ``run_once_into_with_prelude`` (argument order ``(scenario, prelude,
       rule)``, matching ``run_once``'s own lead argument); every
       pre-existing entry point (``run_once``, ``run_once_into``,
       ``TickSession::new``) passes no prelude and is behaviorally
       unchanged. **The identical-recognition law covers ONLY** ``defenum``
       **— not the other three prelude-eligible forms.** A scenario MAY
       re-declare a ``defenum`` type its prelude already declared, verbatim:
       ``EnumRegistry::declare`` (§2.13's own registry) gained an
       IDENTICAL-RECOGNITION arm — same type name, same member list, same
       order — that returns the EXISTING ``EnumTypeId`` rather than
       ``E-LOAD-001``'s ``DuplicateType`` refusal; a re-declaration that
       disagrees (reordered, renamed, added, or dropped a member) still
       refuses exactly as any other colliding ``defenum`` always has —
       ``Vec<String>: PartialEq``'s exact-order, exact-length comparison
       decides which, needing no new comparison logic. ``deffield``,
       ``defconst``, and ``defvocabulary`` gained NO equivalent arm: each
       still refuses ANY second declaration of the same name unconditionally
       — ``load_deffield``'s ``fields.insert(...).is_some()`` check,
       ``load_defconst``'s ``consts.insert(...).is_some()`` check, and
       ``load_defvocabulary``'s ``E-LOAD-001`` kind-guard all fire on a
       collision regardless of whether the re-declaration is identical —
       so a scenario re-declaring a prelude-supplied ``deffield``/
       ``defconst``/``defvocabulary`` refuses even byte-for-byte verbatim. A
       content author factoring one of those three into a prelude must
       either NOT re-declare it in the consuming scenario, or accept the
       refusal; only ``defenum`` sharing composes with local re-declaration.
       First production use:
       ``content/declarations/worldview.bscn`` (the WorldView mint,
       DUPLICATING ``worldview-foundation.bscn:34``'s own
       ``(defenum WorldView …)`` — **correction, final whole-branch
       review, 2026-08-17, item 3:** earlier text here said "factored out
       of" and "byte-identical to" the mint scenario; neither holds.
       ``worldview-foundation.bscn:34`` STILL declares ``WorldView``
       itself — nothing was factored out of it — and the two forms are
       not byte-identical either: the prelude's sits at column 0, the
       mint's is indented two spaces inside its ``(scenario …)`` form, so
       only the declaration TEXT matches, not the line bytes), consumed by
       ``consciousness-ternary-conformance.bscn`` — whose own
       ``(defenum WorldView …)`` re-declaration is DELETED (this train), so
       ``tick_goldens.rs``'s ``consciousness_ternary_worldview_member_order_
       is_the_ruled_ordinal`` test is a DECLARED TEST DEATH: the
       re-declaration it guarded is now impossible for this file. The
       mint's own ``worldview_member_order_is_the_ruled_ordinal`` survives
       as one ordinal home; ``worldview_prelude_member_order_is_the_ruled_
       ordinal`` (added by the final-review fix-forward) is a second,
       asserting the ordinal AS DECLARED BY the prelude itself through the
       real loader — the executable enforcement the declared test death's
       justification previously rested on a comment alone to provide (the
       same failure mode a Task 3 fix round had already closed once in
       this train, recurring unnoticed).
       Byte-neutrality: ``defenum`` declarations are unhashed and the graph
       content is identical, so every tick-hash golden this switch touches
       is UNCHANGED — verified, not assumed
       (``consciousness_ternary_foundation_hashes_are_pinned`` passes with
       the SAME pre/post hashes and the SAME ``fired`` count as the D151
       re-pin it follows; ``consciousness_ternary_conformance.rs``'s
       dual-implementation value-level oracle also passes unedited once its
       own ``run_once_into``/``TickSession::new`` call sites are re-pointed
       at the ``_with_prelude`` siblings — a fact the plan itself did not
       anticipate: this file is a SECOND real consumer of the scenario
       beyond the golden, discovered only at execution, so ``TickSession::
       new_with_prelude``/``run_once_into_with_prelude`` were added
       alongside ``run_once_with_prelude`` rather than deferred — THREE
       ``run_once_into_with_prelude`` call sites plus ONE
       ``TickSession::new_with_prelude`` call site, four total, not five
       (final whole-branch review, item 4 — a prior arithmetic error here
       and in ``ADR209`` counted five)).
       alongside ``run_once_with_prelude`` rather than deferred).
   * - D158
     - N/A (a write re-pointed at the stored ternary — the W1 unification
       re-home, not a BSL construct)
     - **Wave C (issue #605) — the Solidarity @8.0 port re-points frozen**
       ``ideology.class_consciousness`` **to the already-declared**
       ``social-class/revolutionary`` **field; no**
       ``social-class/class-consciousness`` **scalar is minted.** The
       frozen engine makes this identification itself:
       ``ideology.py:382-386``'s own comment reads "class_consciousness <-
       revolutionary (delta_r)", implemented at ``ideology.py:410``
       (``new_class = min(1.0, current_profile["class_consciousness"] +
       delta_r)``). ADR204 W1/W11 struck the legacy cc/ni estate (D146); a
       new scalar field would resurrect a struck surface and would be dead
       — nothing in the ported estate reads ``class-consciousness``, only
       ``social-class/revolutionary`` (``consciousness.bsl``'s own read
       surface). D152 already re-pointed the SOURCE-side gate of this exact
       comparison (the same ``activation_threshold``/``0.3`` define) for
       ``consciousness/p3-class-solidarity-push``; re-pointing the WRITE
       side to a different field within one train would be incoherent.
       Landed: ``solidarity.bsl``'s ``:effects`` write
       ``social-class/revolutionary`` via ``set`` (header lines 18-37).
   * - D159
     - §2.8, §4.2
     - **Wave C (issue #605) — multi-inbound-edge last-write-wins, a**
       **genuine behavioural divergence from frozen.** The frozen engine
       applies each SOLIDARITY edge's delta sequentially, each against the
       PREVIOUS write (``solidarity.py``'s per-edge loop). The port
       collects every subject's writes against the SAME pre-tick graph
       (§4.2's collect-then-apply ordering; ``tick.rs:41-52``) and ``set``
       makes the LAST subject in ascending-node-id order win outright — not
       an accumulation. **What forces** ``set`` **here is not**
       **collect-then-apply alone: an** ``add`` **update-op would still**
       **accumulate correctly at apply time** (§2.8's own
       read-current-at-apply-time semantics). What forces ``set`` is the
       frozen clamp (``solidarity.py:164-165``,
       ``max(0.0, min(1.0, target + delta))``) composed with
       ``social-class/revolutionary``'s ``probability`` declared range: a
       store landing outside ``[0,1]`` is ``E-EVAL-020`` (§2.8, §3.3) — a
       tick-fatal range violation, never an implicit clamp — and a clamp is
       expressible only on a COMPUTED result, i.e. via ``set``. Quantified
       against the frozen oracle fixture
       ``TestSolidaritySystemEdgeCases::test_multiple_solidarity_edges``
       (``tests/unit/engine/systems/test_solidarity_system.py:347-392``):
       two 0.3-strength edges from sources at 0.9 and 0.8 into a target at
       0.1 — frozen yields **0.478** (0.1 -> 0.34 -> 0.478, sequential);
       the port yields **0.31000000000000005** (both deltas computed
       against the unchanged 0.1; the higher-node-id source's write
       survives). Both numbers are EXECUTED, not asserted in prose: the
       conformance world seeds this exact 0.9/0.8/0.1 shape
       (``solidarity-conformance.bscn``'s ``multi-source-a``/
       ``multi-source-b``/``multi-target`` witness), and
       ``solidarity_conformance.rs``'s
       ``multi_inbound_edges_diverge_from_the_frozen_sequential_apply``
       pins the port value while the standalone Python oracle
       (``solidarity_conformance.py``, Task 4) transcribes the SAME
       collect-then-apply/last-write-wins semantics term-for-term (not
       frozen's sequential apply) as its ground truth, per ADR183's
       dual-implementation-not-frozen-floats doctrine.
   * - D160
     - §2.10, §4.2
     - **Wave C (issue #605) — the first rule pack to read another node's**
       **field, making the collect-then-apply pre-state semantics**
       **observable for the first time.** The transmission delta needs the
       TARGET's current value, so ``solidarity/p0-transmit`` uses
       ``(field-of it social-class/revolutionary)`` over a query-yielded
       ``NodeRef`` (§2.10), not ``self``. Every rule pack landed before
       this one reads only its own subject's fields, so §4.2's
       collect-then-apply ordering law (all firings of one rule observe
       the same pre-state) was correct but UNOBSERVABLE —
       ``tick.rs``'s own words, quoted verbatim in ``solidarity.bsl``'s
       header (lines 95-101): "Verified byte-neutral for every rule pack
       landed at the time of the repair — none reads another node's
       field, so the divergence was unobservable UNTIL A RULE DOES." This
       is that rule; D159's multi-inbound divergence is the first
       concrete instance the observability makes visible.
   * - D161
     - N/A (a domain content-model consequence of D158's re-point, three
       verified mitigations — not a BSL construct)
     - **Wave C (issue #605) — the open simplex window, position 8.0 to**
       **position 17.0.** ``solidarity.bsl``'s write is an
       unconstrained-magnitude ``[0,1]``-clamped scalar delta into ONE
       axis of the three-axis simplex ``r + l + f = 1`` (D146's ternary);
       it does not redistribute ``l``/``f``, so it can open a window in
       which a class's ternary sums to more than 1 between this system's
       position (8.0) and ``consciousness/p6-route``'s
       same-tick-or-later closure (17.0). Three verified mitigations, not
       assumed: (1) no pack besides ``consciousness.bsl`` reads the
       ternary today (grep-confirmed; plan §1.4); (2)
       ``consciousness/p6-route``'s verbatim ``normalize_to_simplex``
       closure heals the window the SAME tick
       (``consciousness.bsl:323-330``; D154 already records this heal
       observed for a different write path); (3) nothing enforces the
       simplex at the store — ``probability`` deffields range-check each
       field to ``[0,1]`` independently (``E-EVAL-020``), with no sum
       invariant in the substrate, so the open window is a
       representable, non-fatal state, not a load-time or
       evaluation-time error. Whether periphery-to-core solidarity
       transmission should INFLATE the revolutionary share (port-as-is,
       the path taken here) or DISPLACE liberal/fascist share so the
       simplex never opens is a theory question, not a mechanical one —
       filed non-blocking as `issue #607
       <https://github.com/percy-raskova/babylon/issues/607>`_
       ("director-gate: solidarity transmission — inflate vs displace the
       r-write across the open simplex window"); displacing would require
       inventing redistribution mathematics the frozen engine does not
       have, forbidden by ADR172 ruling 5.
   * - D162
     - N/A (a verified content-model narrowing — one rule, not two; not a
       BSL construct)
     - **Wave C (issue #605) — org-sourced SOLIDARITY edges are provably**
       **inert for this system, so** ``solidarity/p0-transmit`` **is ONE**
       **rule, not two.** The frozen ``SolidaritySystem`` iterates every
       SOLIDARITY edge regardless of endpoint type, then reads
       ``class_consciousness_from_node(src_attrs)``.
       ``ideology`` on organization nodes is a plain ``str`` field
       (declared on the ``PoliticalFaction`` subclass,
       ``src/babylon/models/entities/organization.py:389,395`` — e.g. "Marxism-Leninism"; the
       ``Organization`` base declares none, so its nodes hit the
       ``is None`` branch), and either way
       ``class_consciousness_from_node`` falls through its
       ``isinstance(ideology, dict)`` check and returns ``0.0``
       unconditionally (``src/babylon/kernel/node_access.py:31-37``); ``0.0 <=
       activation_threshold (0.3)`` is always true, so the frozen gate
       skips every organization-sourced edge, every tick, with no
       exception. Unlike ``consciousness.bsl`` — which needs both
       ``p2-org-solidarity-push`` AND ``p3-class-solidarity-push``
       because organizations DO write consciousness there — this port
       needs exactly one rule with one subject type, ``SOCIAL_CLASS``
       (``solidarity-conformance.bscn`` declares no ``ORGANIZATION`` node
       type at all).
   * - D163
     - §2.10, §3.5
     - **Wave C (issue #605) — target liveness AND target consciousness**
       **must both be seeded; the port narrows, it does not diverge in**
       **behaviour.** Frozen's per-edge TARGET read uses TWO permissively-
       defaulting accessors on the same node, not one:
       ``tgt_node.attributes.get("active", True)`` (``solidarity.py:127-
       130``) and ``class_consciousness_from_node(tgt_attrs)``
       (``solidarity.py:148`` — the SAME function the SOURCE-side read at
       ``:140`` also uses), which defaults an absent or non-``dict``
       ``ideology`` payload to ``0.0`` unconditionally
       (``src/babylon/kernel/node_access.py:15-37``). Neither has a BSL equivalent on
       a bare accessor read: ``(field-of it social-class/active)`` and
       ``(field-of it social-class/revolutionary)`` over the TARGET (a
       query-yielded ``NodeRef``, §2.10) both go through the SAME
       accessor discipline — ``:optional``/``:default`` exists only on
       declared ``bindings`` (§3.5), which apply to the SUBJECT'S own
       environment, not to per-edge accessor reads against another node —
       so an element carrying no value for either declared field is
       ``E-EVAL-033`` (§2.10 discipline 2, "absence is not a value"), a
       tick-fatal load/evaluation error, never a default. The ported
       conformance world seeds BOTH ``social-class/active`` and
       ``social-class/revolutionary`` on all 22 nodes to stay within
       declared content; this is a NARROWING of representable content,
       not a behavioural divergence, since frozen's own permissive
       defaults only ever matter for content that never writes either
       field at all — and failing loud on an unseeded world is the
       INTENDED narrowing this row records, not an accident to route
       around. The SUBJECT-side (``self``) reads keep frozen's permissive
       defaults faithfully, via the declared bindings
       ``:optional :default 1`` (``active``) and ``:optional :default
       0.0p`` (``r``, i.e. ``revolutionary``) — only the per-edge TARGET
       reads have no such option.
   * - D164
     - N/A (frozen dead-coefficient surface — zero call sites in
       ``solidarity.py``; not a BSL construct)
     - **Wave C (issue #605) —** ``scaling_factor`` **(0.5) and**
       ``superwage_impact`` **(1.0) are declared on the frozen**
       ``SolidarityDefines`` Pydantic model but have ZERO call sites
       anywhere in ``solidarity.py`` (grep-confirmed against
       ``config/defines/consciousness.py`` and ``defines.yaml:182-187``,
       which lists both alongside the three defconsts this port DOES
       declare — ``activation_threshold``, ``mass_awakening_threshold``,
       ``negligible_transmission``). Not declared as ``:const``s here: a
       declared-but-unread coefficient would itself be dead content, the
       inverse of the ``check:unconsumed``/``check:formula_registration``
       failure mode this estate's sentinels exist to catch on the Python
       side.
   * - D165
     - N/A (a content-model reformulation — find-first graph scan versus
       fold-sum aggregation; not a BSL construct)
     - **Decomposition+ControlRatio port train (issue #591 family), the
       census FIND-FIRST -> PER-NODE-SUM reformulation** —
       registered here per the controller-routed obligation from the
       Task 2 review; prose-only, until this row, in
       ``decomposition.bsl``'s ``p01-la-census`` ``:material-basis``.
       The frozen
       ``_find_entity_by_role(graph, LABOR_ARISTOCRACY)``
       (``decomposition.py:53-85``, called at ``:143``) is a FIND-FIRST
       graph-scope lookup: it iterates ``query_nodes(SOCIAL_CLASS)`` in
       the graph's own iteration order, skips inactive entities by
       default, and returns the FIRST role-matching entity — one node,
       whichever the iteration happens to reach first.
       ``decomposition/p01-la-census`` reformulates this as a PER-NODE
       gated write: every ``SOCIAL_CLASS`` subject fires, and a non-LA
       (or inactive LA) writes zero to all four published fields (the
       D127 hash-neutral idiom, ``p01``'s own no-``when`` shape), forced
       by ``field_ref_for``'s compound-fold-body refusal (D138) — the
       role/active filter cannot live inside a fold body at all.
       ``decomposition/p03-trigger`` then ``fold sum``\ s those per-node
       contributions across every ``SOCIAL_CLASS`` node onto the
       carrier. This is a FOLD-SUM over ALL matching nodes, not a
       FIND-FIRST over one — the two are numerically identical only when
       a world contains at most one active ``LABOR_ARISTOCRACY`` node,
       which is true of every conformance fixture this train ships
       (``decomposition-conformance.bscn``,
       ``decomposition-delay-conformance.bscn`` each seed exactly one).
       ``p03`` inherits this SAME divergence through the identical
       reformulation: in a world with two or more active
       ``LABOR_ARISTOCRACY`` nodes, the frozen engine's ``la_wealth``/
       ``la_pop``/``subsistence``/``consumption`` (and so
       ``la_approaching_death``/``la_about_to_die``, and — downstream,
       in ``_execute_decomposition``, the actual ``enforcer_pop_gain``/
       ``proletariat_pop`` split, since ``_find_entity_by_role`` is
       called again at ``:280-284`` with the SAME find-first semantics)
       would come from whichever single LA node the graph's iteration
       order reaches first, while the ported
       ``institution/la-population``/``la-wealth``/
       ``la-approaching-count``/``la-dying-count`` SUM across every
       active LA node instead — an aggregate, not a single entity's
       state. No landed fixture in this train exercises more than one
       active ``LABOR_ARISTOCRACY`` node at once, so the divergence is
       DESCRIBED, not measured; a future multi-LA fixture would need to
       choose which engine's answer to pin, the same open posture
       D124's own same-priority-tiebreak row leaves standing.
   * - D166
     - N/A (a content-model reformulation — the non-graph persistent_data dict to a
       graph-backed singleton carrier; not a single BSL construct)
     - Decomposition+ControlRatio port train (issue #591 family) — the frozen
       ``TickContext.persistent_data`` state machine (``_superwage_crisis_tick``,
       ``_decomposition_complete``, ``_class_decomposition_tick`` on the
       Decomposition side; ``_terminal_decision_emitted``,
       ``_control_crisis_emitted``, ``_control_ratio_crisis_tick`` on the
       ControlRatio side — ``decomposition.py:49``, ``control_ratio.py``'s six
       ``persistent.get``/``persistent[...] = `` sites) becomes ``institution/*``
       fields on a single ``carceral-register`` singleton ``NodeType/INSTITUTION``
       carrier, read via ``(field-of (select-max (nodes NodeType/INSTITUTION) 1)
       institution/…)`` and written via ``(update-node (select-max (nodes
       NodeType/INSTITUTION) 1) institution/… (set …))`` — the D103/D104
       accumulate-into-a-non-self-target lane, subject-type derivation via the
       carrier's own ``:field`` bindings (``tick.rs:159-182``), never ``(domain
       :graph)`` at execution (Task-0 dossier §6 row 1, §7). The ``the`` accessor
       both 2026-08-12 inventories floated for this exact reformulation is REJECTED:
       it sits in ``UNSERVED_EXPRESSION_HEADS`` tagged ``"slice 2"``
       (``evaluator.rs:523-530``) and its ``E-LOAD-043`` singleton guard fires only
       behind a declared ``manifest`` form, which zero landed content uses (Task-0
       dossier §1, §3 item 7) — the ``select-max``-over-``nodes`` idiom is the one
       that evaluates today, matching Territory's and Solidarity's own carrier
       precedent. Every ``None``-sentinel the frozen dict carries on a tick-valued key (0 is a
       real tick, not an absence) becomes a companion ``*-known`` int 0/1 flag
       (III.11 loud-absence encoding — ``superwage-crisis-known`` paired with
       ``superwage-crisis-tick``, ``decomposition-fired-known`` paired with
       ``decomposition-fire-tick``) rather than a magic sentinel value. The frozen
       dict's own boolean latches (``_control_crisis_emitted``,
       ``_terminal_decision_emitted``, ``_decomposition_complete``) carry no such
       companion — they become ordinary 0/1 ``int`` fields directly
       (``control-crisis-emitted``, ``terminal-decision-emitted``,
       ``decomposition-complete``), since a plain 0/1 value already distinguishes
       not-yet from happened without a second field to attest to it. No
       ``:optional``/``:default`` route exists for a
       ``.bscn``-seeded field either way (``scenario.rs::load_deffield``, Task-0 dossier §6:
       seven type tokens, no ``:default``).
   * - D167
     - N/A (a structural-verb refusal — ``add-node`` is refused at content load; not
       a landed construct)
     - The frozen ``_create_target_entity``/``_derive_entity_id`` create-on-demand
       path (``decomposition.py:225-261,36-50``, spec-071) is OMITTED entirely:
       ``add-node`` is one of the six ``DEFERRED_SHAPE_VERBS`` refused at content
       load (``structural_verbs.rs:1723-1730``, ``check_no_deferred_shape_verbs``,
       called unconditionally at ``rule_pipeline.rs:269``) — a MINTING verb needs a
       placeholder-id scheme the collect-then-apply pre-state repair does not specify
       (Task-0 dossier §1.1-1.2). Every conformance world this train ships pre-seeds
       its own ``CARCERAL_ENFORCER``/``INTERNAL_PROLETARIAT`` targets instead
       (``decomposition-conformance.bscn``'s own header, BLOCKER-1);
       ``decomposition/p04-enforcer-intake``/``p05-ip-intake`` read/write the
       pre-seeded nodes and never create one. A world lacking either pre-seeded
       target is UNPORTED for that branch, not equivalent to the frozen engine's
       on-demand creation — the frozen system tolerates an absent target (it creates
       one); the port does not (the intake rule's own ``when`` role-filter simply
       never matches, a silent no-op rather than a create). Follow-on: **#562** (the
       structural-verb execution surface / T5, Program 29) is the placeholder-id
       design that would let a future revision serve ``add-node`` for this branch;
       until it lands, seeding both targets is a standing authoring obligation on
       every world this pack loads, not a corner case.
   * - D168
     - N/A (an omitted read path — no BSL ``<bind-src>`` names an event ledger; not a
       construct)
     - The frozen engine's ``services.event_bus.get_history()`` scan for
       ``SUPERWAGE_CRISIS`` events (``decomposition.py:164-175``), which recovers
       ``_superwage_crisis_tick`` from event history on a tick where
       ``persistent_data`` alone lost it, is OMITTED: §2.5's closed ``<bind-src>``
       set (``:field | :const | :metric | :tick | :year | :tick-of-year |
       :tick-in-cycle | :expr``) names no event ledger.
       ``decomposition/p02-superwage-warning``'s own SAME-TICK carrier latch
       (``superwage-crisis-known``/``-tick``, written the same tick it emits) is the
       sole source of truth this port relies on instead — the re-modelling this
       document's own gap item 3 ("the emitting rule also stamps a field") already
       prescribes, not an invented shortcut. The read is PROVABLY UNREACHABLE for
       this port specifically because ``ImperialRentSystem`` (@9.0) is the ONLY OTHER
       frozen production emitter of ``SUPERWAGE_CRISIS`` (``economic.py:462-487``,
       the pool-exhaustion path, independent of Decomposition's own early-warning
       emission) and ``ImperialRentSystem`` is itself unported — no second
       same-tick-or-earlier emitter exists in the ported estate for ``p02``'s history
       scan to ever need to recover. Explicit re-open trigger: when
       ``ImperialRentSystem`` (@9.0) ports, its own ``SUPERWAGE_CRISIS`` emission
       becomes a second producer the carrier's single same-tick latch cannot
       distinguish from ``p02``'s — that port's own D-record must re-examine whether
       the carrier-latch re-modelling still suffices or needs a producer-tagged
       variant.
   * - D169
     - N/A (companion to D165 — the int-ordinal rejection and the fold-body laws that
       force the per-node reformulation; not a new construct)
     - Companion to D165 (the census FIND-FIRST -> PER-NODE-SUM reformulation itself,
       which this row does not repeat) — records the REMAINING content both
       2026-08-12 phase-1 inventories' Adjudications recommend and this port train
       REJECTS. Both inventories (decomposition §6 row ``_find_entity_by_role``;
       control-ratio §6 Phase-2 row, corrected in Adjudication item 4) recommend an
       int-ordinal ``SocialRole``/``role`` encoding (Territory's/``lifecycle.bsl``'s
       convention) specifically to preserve ``field-of``-based query-predicate
       filterability under D102's THEN-unconditional deferral. That premise no longer
       holds: D102 is DISCHARGED (Task 1, P27 territory-port train;
       ``rule_pipeline.rs:293-301``, ``typecheck.rs:862-868,877-919`` in this tree —
       Task-0 dossier §2.1 corrects the plan's own stale ``typecheck.rs:246-289``
       citation) — ``field-of`` over an enum-declared field now typechecks and
       evaluates for real. The int-ordinal route is rejected because TWO OTHER,
       INDEPENDENT laws close it regardless of D102: (a) ``field_ref_for``'s
       compound-fold-body refusal (D138, ``rule_pipeline.rs:633-686,764-778``)
       refuses any fold body beyond a bare ``<qname>``/``field-of`` accessor/nested
       carrying-fold — an ``if``-based role/active FILTER cannot live inside a fold
       body at all; (b) ``TypeCode::EnumFoldBody``/``E-TYPE-044``
       (``rule_pipeline.rs``'s ``enum_fold_body_tests:1021-1128``, both the
       ``:field``-bound-symbol and the ``field-of``-accessor routes) separately
       refuses a ``fold sum``/``mean``/``min``/``max`` whose body IS a legal
       enum-typed field reference. Together these close BOTH escape routes for an
       enum-typed ``role`` field inside a fold body — read it directly (E-TYPE-044
       refuses) or wrap it in a filter to dodge the enum check (D138 catches that
       instead) — exactly why ``decomposition/p01-la-census`` and
       ``control-ratio/c01-prisoner-census`` gate ``role`` on the SUBJECT side
       (per-node ``:expr`` bindings), never inside a fold body, and why this train
       declares ``social-class/role enum SocialRole`` (the real enum, not an
       int-ordinal) at all. The SAME compound-fold-body law (D138) plus
       ``E-TYPE-044`` are what force the ``pop × organization`` PRODUCT
       (``control-ratio/c01``'s ``prisoner-org-contribution``) out of ``c02``'s
       carrier-side fold and onto ``c01``'s own per-node ``:expr`` binding — a
       bare-accessor fold body cannot compute a product any more than it can filter.
       Finally: ``fold mean :weight`` (``grammar.rs:797-816``'s ``(fold <fold-op>
       <query> <elem-name>? <expr> (:weight <expr>)?)`` form) is the REJECTED
       alternative for ``avg_organization`` — it would compute the entire weighted
       mean inside ONE fold call, hiding the two-step sum-then-divide arithmetic
       (``prisoner-org-weighted / prisoner-population``, ``control_ratio.py:171``'s
       own ternary) the frozen system's arithmetic must be checked against bit for
       bit; ``c01_premultiplies_population_by_organization``
       (``control_ratio_conformance.rs``) asserts the per-node products bit-exact for
       exactly this reason — the rejection was checked, not merely asserted.
       A companion field-type choice this same train recorded (Task 1):
       ``social-class/organization`` itself declares ``coefficient intensive``, not
       the plan's suggested ``int extensive``, because a fractional seed
       (``lumpen``'s ``0.2``) refuses at load under an ``int``-declared field —
       ``probability`` was also viable at that same adjudication point, weighed
       against ``coefficient`` and not chosen (ADR212 §8).
   * - D170
     - N/A (an observable-state-surface widening — the port publishes every tick
       where the frozen system gates behind readiness; not a construct)
     - ``control-ratio/c01-prisoner-census`` + ``c02-publish-census`` publish the
       guard/prisoner census EVERY tick, unconditionally — neither rule reads a
       single carrier readiness latch. The frozen ``ControlRatioSystem.step()``
       computes its census (``control_ratio.py:137-138``) only PAST three
       early-return gates (``_terminal_decision_emitted``, ``:124-125``;
       ``_class_decomposition_tick is None``, ``:128-130``; the delay-elapsed check,
       ``:132-134``). The port WIDENS the observable state surface relative to the
       frozen engine: the carrier now always carries a live, current census, even in
       ticks/worlds where the frozen engine would never have computed one at all —
       ``c02_publishes_the_three_aggregates_unconditionally``
       (``control_ratio_conformance.rs``) proves it directly against an inline
       NOT-READY world (``decomposition-fired-known 0``). ``c02``'s own
       ``institution/decomposition-fire-tick`` binding is a SUBJECT-TYPE ANCHOR ONLY
       (``tick.rs::subject_type_of`` requires ≥1 ``:field`` binding to derive a
       carrier-anchored rule's subject type), never a gate — reading it and never
       using it is the honest shape, not a design accident (``control-ratio.bsl``'s
       own D-record 2).
   * - D171
     - N/A (``emit`` payload-shape divergences from the frozen event dicts; not a
       single construct)
     - Four distinct payload divergences across this pack's four ``emit`` sites, all
       forced by ``<payload-item>``'s flat ``(<symbol> <expr>)`` shape (no dict, no
       string): **(1) flattened nested dicts** — ``CLASS_DECOMPOSITION``'s frozen
       ``population_transferred``/``wealth_transferred`` sub-dicts
       (``decomposition.py:352-359``) become four top-level numeric keys
       (``population-transferred-to-enforcer``/``-to-proletariat``,
       ``wealth-transferred-to-enforcer``/``-to-proletariat``) on
       ``decomposition/p06-la-deactivate``'s emit. **(2) dropped narrative_hints** —
       every ``narrative_hint`` string (``decomposition.py:189-192,361-365``) and
       ``CLASS_DECOMPOSITION``'s ``trigger_event`` string (``:360``, always
       ``"superwage_crisis"``) are omitted from all four event types
       (``SUPERWAGE_CRISIS``, ``CLASS_DECOMPOSITION``, ``CONTROL_RATIO_CRISIS``,
       ``TERMINAL_DECISION``) — ``trigger_event`` is the same class of omission as
       ``narrative_hint``, not a separate divergence, since the carrier's own
       ``fire-tick`` latch already makes the trigger unambiguous. **(3) the numeric
       ``outcome`` encoding** — ``control-ratio/c04-terminal``'s frozen ``outcome``
       string (``"revolution"``/``"genocide"``, ``control_ratio.py:222,228``) becomes
       ``(outcome 1)``/``(outcome 0)``, payload key order transcribed verbatim from
       ``:239-245`` (``outcome``, ``avg_organization``, ``revolution_threshold``,
       ``prisoner_population``, ``enforcer_population`` — five keys,
       ``narrative_hint`` dropped, not six). **(4) the omitted ratio keys in the
       zero-enforcer case** — ``control-ratio/c03-crisis``'s guard-split emit omits
       ``actual-ratio``/``control-ratio`` entirely when ``enforcer-population == 0``
       (loud absence, not a fabricated value), since the frozen ``float("inf")``
       (``control_ratio.py:185``) has no BSL literal form (BLOCKER-4,
       ``control-ratio.bsl``'s own D-record 4). Deliberately NOT a divergence:
       ``c03`` PRESERVES the frozen payload's own redundant duplicate key —
       ``control-ratio`` duplicates ``actual-ratio`` verbatim
       (``control_ratio.py:198-199``), port-as-is, not simplified away.
   * - D172
     - N/A (D100's class — the global rule-id byte-order sort inverting a frozen
       tick-position order; not a new construct)
     - **ADR222/PER-17 supersedes this implementation record.** The executable
       registry now runs Decomposition before ControlRatio. The zero-delay
       acceptance test requires the same-tick crisis at tick 1. This text
       records the former global-byte-order hazard and its prior rationale.

       ``control-ratio/*`` sorts BEFORE ``decomposition/*`` in ascending rule-id byte
       order (D100's class — the comparison resolves at the NAMESPACE segment,
       ``'c'`` (``control-ratio/``) < ``'d'`` (``decomposition/``), before ever
       reaching the rule-local ``c01`` vs ``p01`` prefixes), inverting the frozen
       @11.0-then-@12.0 system order: within EVERY tick, ``c01``-``c04`` run to
       completion before ``p01``-``p06`` start. The delay-protection argument: the
       only cross-pack datum ``control-ratio/*`` reads is the carrier's
       ``institution/decomposition-fire-tick``/``-fired-known`` (written by
       ``decomposition/p03-trigger``), so on the tick decomposition actually fires,
       ``c03``'s readiness gate reads those fields' PRE-this-tick values, one
       rule-order "behind" — but this is invisible on every shipped world because
       ``control_ratio_delay`` is the SHIPPED 52, not 0, so the earliest ``c03``
       could possibly fire is 51+ ticks after the one-rule-order lag resolves itself
       on the next tick. The authoring constraint this reliance imposes: NO Pack
       B scenario may set ``carceral/control-ratio-delay`` to 0 while ALSO relying on
       ``decomposition-fire-tick`` being written by a co-loaded
       ``decomposition/p03-trigger`` that SAME tick — every scenario in this pack
       seeds the fire-tick fields directly instead, so the hazard never engages
       (``control-ratio.bsl``'s own D-record 6). The test that enforces it,
       executably:
       ``the_byte_order_inversion_delays_a_same_tick_race_by_exactly_one_tick``
       (``carceral_arc_conformance.rs``) — an isolated, minimal fixture with
       ``control-ratio-delay 0`` and an UNSEEDED fire tick, where the one-rule-order
       lag is the ONLY thing standing between "no crisis" and "a crisis" on the
       firing tick, proving no crisis fires at tick 1 (the SEEDED-0 read) and one
       fires at tick 2 (the write becomes visible one rule-order later).
   * - D173
     - N/A (four inherited frozen-code defects, transcribed verbatim per port-as-is
       law, ADR183; not BSL constructs)
     - **(1) Docstring drift, THREE sites** (fixed forward, final review M5 — a third
       site added; the first two remain off the SAME stale 30/70 split).
       ``decomposition.py``'s module docstring (``:4-6``, class docstring ``:90-96``)
       claims "30% of Labor Aristocracy becomes CARCERAL_ENFORCER" / "70% falls into
       INTERNAL_PROLETARIAT", but the CODE reads ``CarceralDefines``
       (``enforcer_fraction = 0.15``, ``proletariat_fraction = 0.85``,
       ``defines.yaml:295-296``) — 15%/85%, not 30%/70%. A SECOND,
       independently-stale docstring compounds the same error: ``CarceralDefines``'s
       own class docstring (``src/babylon/config/defines/territory.py:265-267``)
       states "With 70/30 decomposition, prisoner/enforcer = 2.33:1, so:
       control_capacity <= 2: Crisis triggers immediately; control_capacity >= 3: No
       crisis" — arithmetic computed off the STALE 30/70 split. At the SHIPPED 15/85
       values the true ratio is 85/15 = 5.67:1, so the shipped ``control_capacity =
       4`` default DOES produce a crisis, directly contradicting the docstring's own
       "No crisis" claim. Transcribed as the shipped 15%/85% behavior in both
       ``decomposition/p03-trigger`` and its own defconsts; both stale comments are
       recorded here, neither "corrected" in the comment itself, per ADR183 (the code
       is the port's oracle, not its own docstring). A THIRD, independently-stale
       docstring sits in the ControlRatio frozen source itself, off a DIFFERENT stale
       ratio (not the 30/70 lineage): ``control_ratio.py:7`` states "User
       specification: 1:20 ratio (1 guard can control 20 prisoners)" while the
       shipped ``carceral.control_capacity`` is **4** (``defines.yaml:294``) and the
       SAME file's own ``:145`` comment already says "US average ~4:1" — the module
       docstring was never updated when the shipped default moved to 4. Transcribed
       as the shipped ``control_capacity = 4`` in ``control-ratio/c03-crisis`` and its
       own defconst; the stale ``:7`` comment is recorded here, not corrected in the
       frozen source, per ADR183. **(2) Additive/overwrite
       asymmetry** — ``decomposition/p04-enforcer-intake`` is ADDITIVE (``current +
       gain``, ``decomposition.py:327-332``) while ``decomposition/p05-ip-intake`` is
       a flat OVERWRITE (``decomposition.py:334-336``), transcribed as two DIFFERENT
       update-op shapes (``add`` vs ``set``) rather than unified into one idiom, per
       port-as-is law. **(3) LA population/wealth non-conservation** —
       ``enforcer_pop_gain = int(la_population * enforcer_fraction)`` and
       ``proletariat_pop = int(la_population * proletariat_fraction)``
       (``decomposition.py:298-299``) floor INDEPENDENTLY; their sum can be strictly
       LESS than ``la_population`` for a population not evenly split by 0.15/0.85,
       and LA's own ``wealth``/``population`` are never zeroed on deactivation
       (``decomposition.py:339`` touches only ``active``) —
       ``decomposition/p03-trigger`` computes both amounts with the same two
       independent ``floor`` calls, transcribing the loss verbatim; this train's own
       fixtures happen to split exactly (150+850=1000) and do not witness the loss.
       **(4) The bare ``2`` literal** — ``carceral/approaching-consumption-multiple``
       (defconst value ``2``) has NO ``CarceralDefines`` backing anywhere in the
       frozen source: it is a bare ``2 * consumption`` literal at
       ``decomposition.py:155``, never named as a define — transcribed as a bare
       ``2c`` constant with this D-record as its "no defines backing" note, not
       invented a define for it.
   * - D174
     - N/A (a RESERVED-LINE transcription statement — Constitution IX.5 / ADR070 /
       Program 19; not a BSL construct)
     - ``control-ratio/c04-terminal`` transcribes the frozen
       ``_emit_terminal_decision`` revolution-vs-genocide branch
       (``control_ratio.py:210-247``) VERBATIM: the SAME threshold source
       (``carceral/revolution-threshold``), the SAME ``>=`` comparison
       (``avg_organization >= revolution_threshold`` -> REVOLUTION, else -> GENOCIDE,
       ``:222,228``), BOTH outcomes preserved as a guard-split emit differing only in
       the numeric ``outcome`` encoding (D171 item 3), and the SAME two prisoner
       roles feeding the census (``INTERNAL_PROLETARIAT``, ``LUMPENPROLETARIAT`` —
       ``control-ratio/c01-prisoner-census``'s ``prisoner-gate``, transcribing
       ``control_ratio.py:32-37``'s ``_PRISONER_ROLES`` frozenset). **ADR070 /
       Program 19 rules ControlRatioSystem's revolution-vs-genocide branch explicitly
       LAST in the emergent-class-partition cutover** (ADR070's own "Cutover roadmap"
       section, ``ai/decisions/ADR070_emergent_class_partition.yaml:100-103``: "no
       exception, only after low flip-count evidence, with a dedicated high-effort
       review" — corrected by the 2026-08-17 final whole-branch review's I3 finding;
       the phrase is ADR070's, not Constitution IX.5's, which names the correct
       authority for *a question touching the ideological line escalates to the
       Director* but is not this sentence's source) — this port therefore
       deliberately does NOT re-base the branch onto a
       derived class-cell partition; it transcribes the current ``SocialRole``-keyed
       reads as-is. The transcription is consistent with ADR070's own
       slots-as-positions ruling (``ADR070_emergent_class_partition.yaml``: "the
       seeded SocialRole typology demoted to slots-as-positions seed vocabulary" —
       role is real, persistent seed vocabulary, not something a port should derive).
       The emergent-class-partition cutover remains Director-gated and OPEN — this
       port describes the reserved branch, it does not propose or pre-empt the
       cutover. Cited: the 2026-08-12 ruling (both phase-1 inventories' Adjudication
       §6/§7 confirmations, control-ratio inventory's own RESERVED-LINE row) and its
       reaffirmation in **ADR208 R29 / C-03** ("the Decomposition+ControlRatio train
       verified fully independent of #576"), plus **register row 12 of #564**
       ("ControlRatio revolution-vs-genocide branch — Program 19 rules it LAST; does
       the joint Class-D train respect that or split?" — answered here: it respects
       it, the train does not split the branch onto a derived partition, and row 12
       is now checked off #564).
   * - D175
     - §4.3, §3.10
     - ``exp``/``log`` cross into the Rust engine via the ``libm`` crate,
       version-pinned at ``0.2.16`` with ``default-features = false``
       (``babylon_kernel::transcendental``; ADR176 ruling 21, reaffirmed by
       ADR188's decision paragraph) — never via ``f64::exp``/``f64::ln``, which
       route to the *platform* libm (glibc/musl/Apple's) and are banned
       workspace-wide by ``rust/clippy.toml``'s ``disallowed-methods`` row.
       Verified dispatch: ``log`` (``libm::log``) carries no
       architecture-select code at all; ``exp``'s only dispatch arm
       (``x86_no_sse``) is unreachable on both of Babylon's targets
       (``x86_64``, ``aarch64``) — the crossing is therefore bit-identical
       across OS/libc/CPU by inspection of the dispatch predicates, turned into
       an executable guard by
       ``rust/crates/babylon-kernel/tests/transcendental_goldens.rs``'s
       per-intrinsic ``assert_eq!`` on ``f64::to_bits()``. **Two-regime
       tolerance (III.12 corollary (b)):** WITHIN the Rust engine, tolerance is
       ZERO — any drift is a red gate. AGAINST the frozen Python engine only,
       glibc and this MUSL port disagree in the last 1-2 ULP, a relative bound
       of roughly ``4.44e-16`` per crossing, seven-plus orders of magnitude
       inside the ``qa:regression`` checkpoint tolerance of ``1e-5``. Full
       derivation: :doc:`/reference/determinism-contract`'s *Transcendental
       Crossing* chapter.
   * - D176
     - §3.10
     - ``rng-draw``'s declared signature is ``(intrinsic rng-draw :params (int)
       :returns real :cost <author-declared>)`` (ADR188 Row 11), landed by #576
       Task 5. **This row formally supersedes D69's operand-shape clause for**
       ``domain`` **— D69's purity clause survives verbatim (item iv, below).**
       D69 fixed the carrier key as ``(session, tick, domain, stable_key)``,
       with ``domain`` a "closed-vocabulary enum operand" and ``stable_key``
       deriving "from the identities of the call's reference operands"; neither
       survives as declared. (i) **Undeclarable as written** —
       ``<intrinsic-decl>``'s ``:params`` vocabulary is parsed by
       ``parse_intrinsic_type_name`` (``declarations.rs:801-813``), which
       delegates every non-``real`` name to ``parse_type_name``
       (``declarations.rs:662-699``); together they admit SEVEN of §3.1's eight
       rows (``int``, ``bool``, ``currency``, ``probability``, ``intensity``,
       ``coefficient``, ``real``), refuse ``enum`` outright at that grammar
       position (no ``:enum-type`` companion slot), and carry no row at all for
       a node/edge reference. (ii) **Closing that gap is deliberately not done
       here** — it would widen ``<intrinsic-decl>``'s grammar, moving §5.6
       canonical-AST bytes and ``rules_hash``, out of scope for this train.
       (iii) The landed design instead uses **the firing rule's own id string**
       as ``domain`` (kernel-derived, never a call operand) — *stronger* than
       the enum operand on the content-cannot-mint-a-stream axis: content
       cannot even name a stream this way, only mint an entirely new rule,
       itself already hash-covered content. (iv) D69's load-bearing clause — a
       draw is a pure function of its key, not a stream position — survives
       unamended: the host holds no state, a fresh ``KernelRng`` is built per
       call and discarded, so a guard-skipped draw cannot shift any other
       subject's draw. Implementation: ``eval_rng_draw``,
       ``rust/crates/babylon-bsl/src/intrinsic_host.rs:468-497``.
   * - D177
     - §3.10
     - ``stable_key`` is the injective composition ``framed`` applies to three
       segment groups, in this exact order: the subject's content id; the
       resolved element-content-id chain, OUTERMOST-to-INNERMOST (the §2.6
       chapter C8 element stack's own order — an ``Element::Node`` resolves to
       its bare content id, an ``Element::Edge`` resolves to its two endpoints'
       content ids **and its own** ``edge_type``, **all three** composed by one
       nested ``framed`` call into a single chain entry (review round 2, #576
       I1 — ``EdgeKey`` carries ``(source, target, edge_type)``; the
       pre-round-2 composition dropped ``edge_type``, so two parallel edges of
       DIFFERENT types between the SAME node pair drew bit-identical values —
       fixed by ``evaluator::element_content_id``'s Edge arm, verified by the
       c14 conformance family's own edge-element-parallel-type row); against a
       NEVER-HYDRATED (``None``) ``node_content_ids`` — hand-built test
       fixtures only, never a scenario-hydrated graph, even one hydrated with
       zero ``(node …)`` forms (review round 2, #576 I2: the gate is
       ``Option``-typed, not ``is_empty()``-gated, precisely so a hydrated-but-
       empty map is distinguishable from a never-hydrated one) — an element
       renders as its bare ``NodeId`` ``Debug`` text instead, the same
       ``None``-typed gate at both call sites, ``tick.rs:756`` and
       ``evaluator.rs:1618``; and the draw slot's ``Int`` argument, rendered as
       its plain decimal text (Rust ``i64::to_string()``, no separators, a
       leading ``-`` for negative values). **The ``framed`` byte layout**
       (``rust/crates/babylon-bsl/src/intrinsic_host.rs:119-126``): given
       segments ``s_1 .. s_n``, each is rendered as its UTF-8 byte length in
       decimal, a colon, then the segment's own UTF-8 bytes; the rendered
       segments are joined by the single byte ``|``. Example: ``framed(["ab",
       "c"]) = "2:ab|1:c"``. **Injectivity argument:** the length prefix makes
       each segment self-delimiting — a reader consumes the decimal length up
       to the colon, then reads exactly that many bytes as the segment,
       recursively — so no segment boundary is ever inferred from content, and
       two distinct segment sequences can never render to the same string;
       naive concatenation (``"ab"+"c" = "a"+"bc" = "abc"``) is exactly the
       collision this framing rules out. Once composed, ``stable_key`` becomes
       ``KernelRng::for_carrier``'s fourth argument, itself one input to
       ``seed_for``'s SHA-256 (``rust/crates/babylon-kernel/src/rng.rs:53-63``,
       unchanged by this train): ``SHA256(session_utf8 ‖ tick_le8 ‖ salt_le8 ‖
       len_le8(domain) ‖ domain_utf8 ‖ len_le8(stable_key) ‖
       stable_key_utf8)``, where ``tick``/``salt``/each length prefix is 8
       bytes little-endian and ``salt`` is the pinned constant ``SEED_SALT =
       0x0BA1_AC1A`` (``rng.rs:41``, structurally mirroring the frozen Python
       engine's ``_SYSTEM_RNG_SEED_SALT``, not stream-compatibly — R8). The
       resulting 32-byte digest seeds ``ChaCha8Rng`` — the ``rand_chacha``
       crate, **version-pinned at** ``0.10`` **(``rust/crates/babylon-
       kernel/Cargo.toml:13``; review round 2, #576 M6 — the crate the
       ``seed_for`` half's own language-agnostic byte contract left
       unpinned)**, zero nonce, zero counter — directly, one stream per
       carrier, counter-mode. The draw itself
       (``rng.rs:87-95``) is the top 53 bits of one ``next_u64()`` (a
       right-shift by 11), scaled by ``2⁻⁵³`` — every representable value an
       exact multiple of ``2⁻⁵³``, so the ``u64``→``f64`` mapping is
       bit-deterministic across platforms with no libm and no rounding-mode
       dependence. :doc:`/reference/determinism-contract`'s *Fuel Cost Model
       and RNG Seeding* chapter carries the full algorithm rationale (ChaCha8
       over ChaCha20, per-carrier not per-tick streams). Verified executably:
       ``framed_renders_each_segment_length_prefixed_and_pipe_joined``,
       ``framed_is_injective_where_naive_concatenation_would_collide``
       (``intrinsic_host.rs:786-802``), and the c14 conformance family's own
       key-framing-injectivity, edge-element-parallel-type, and
       hydrated-but-empty-map rows.
   * - D178
     - §3.10
     - ``rng-draw``'s ``stable_key`` composes over CONTENT ids, never
       ``NodeId`` handles. ``babylon_graph::substrate::NodeId`` (``pub struct
       NodeId(pub u64)``) is an opaque handle minted in insertion order by
       ``add_node``; keying a draw on it would be replay-deterministic but
       insertion-history-dependent — inserting one more scenario node ahead of
       others shifts every later handle, precisely the butterfly ADR176 r20's
       per-carrier-stream design forbids and precisely what D69's "independent
       of insertion history" clause rules out. **Disposition: retain the
       existing hydration map; do not widen the substrate.** ``babylon-bsl``'s
       scenario loader already builds a local ``named: HashMap<String,
       NodeId>`` during hydration and discarded it at function return; #576
       Task 3 retains its inverse as ``pub node_content_ids: HashMap<NodeId,
       String>`` on ``LoadedScenario`` (``scenario.rs:311``, built by
       ``invert_content_ids``, ``scenario.rs:657``), which asserts injectivity
       loudly at construction — two distinct content ids resolving to one
       ``NodeId`` panics as a hydration bug, an unconstructible state through
       every reachable loader call site today — threaded through
       ``PreparedRules`` (``babylon-tick/src/lib.rs:125-149``) onto
       ``TickSession`` and the one-shot drivers' tick seam. **The graph's
       canonical state hash is untouched:** zero ``babylon-graph`` change, zero
       ``CanonicalState::state_hash`` (``babylon-graph/src/state_hash.rs:405``)
       change — ``PreparedRules::node_content_ids``'s own doc states it plainly
       ("carries no canonical-state weight") — verified by
       ``fundamental_theorem_tick.rs``'s state hash staying byte-identical
       across Tasks 3-5 (no shipped content calls ``rng-draw`` yet, so nothing
       could have entered any hash through this seam). **Escalation path, not a
       blocker:** a reviewer who prefers the stable identity live IN the
       substrate — a queryable content-id accessor on ``GraphSubstrate`` itself
       — is naming a Program 29 substrate-widening item carrying its own
       Constitution III.7 hash question (does canonical state cover a node's
       content id?), not something this train improvises. The retained-map
       approach is deliberately the non-hash-touching one.
   * - D179
     - §3.10
     - A campaign's ``rng-draw`` session id is **deterministic-only** (III.7:
       never a UUID, never a wall-clock read).
       ``babylon_kernel::SessionId::new`` accepts any non-empty string
       (refusing empty with ``EmptySessionId``) and performs no other
       validation — the *choice* of string is the policy this row records, not
       the type. **The one-shot conformance/CLI drivers** (``run_once``,
       ``run_once_into``, ``run_once_with_prelude`` and their ``_into``
       siblings, all pinned at tick 1) use the fixed literal
       ``SessionId::new("run-once")``
       (``rust/crates/babylon-tick/src/lib.rs:469``) — one non-random literal
       shared by every one-shot call, never derived per invocation. **A live
       campaign's own session id is the ``ContentDigest`` hex, or the scenario
       id** — minted by the CLIENT, never the kernel, and never a UUID or a
       wall-clock read. ``TickSession::new``'s ``session: SessionId`` parameter
       (``rust/crates/babylon-tick/src/session.rs:52-57``, the parameter itself
       at ``:56``) is the seam this decision lands through; the type accepts
       whatever the caller supplies.
       ``babylon-client``'s own B2 demo call site (``engine_link.rs:130``)
       currently passes a placeholder literal,
       ``SessionId::new("babylon-client-b2-demo")``, documented inline as
       exactly the placeholder this row's convention is meant to replace —
       adopting the ``ContentDigest``-hex/scenario-id convention at that call
       site is separate, undirected follow-on work, not a blocker this record
       resolves.
   * - D180
     - §2.7, §3.10
     - ``PROHIBITED_INTRINSIC_NAMES = ["sigmoid"]`` (``declarations.rs:128``)
       is a **name-level** gate only, checked at ``intrinsic`` declaration
       time. It cannot see that ``exp`` plus ordinary arithmetic already
       expresses the prohibited shape out of two PERMITTED intrinsics: the
       logistic form ``1 / (1 + exp(-x))`` and ``tanh(x) = 2 × sigmoid(2x) −
       1`` are both constructible from ``exp`` alone now that ``exp``/``log``
       dispatch (D175, #576 Task 2) — exactly the hazard
       ``reports/port-estate-survey-2026-08-12.md:305`` names for the
       ``tanh``-eliminated Contradiction @18.0 site. **Gate 2 stays Director
       review** (D71's own position: cap-legality is not doctrine-legality);
       this row records the mechanical gate's known blind spot rather than
       claiming it closes doctrine enforcement. **Recommended, not built in
       this train:** an emergence-audit sentinel that pattern-matches
       logistic-shaped subexpressions — an ``exp``-of-negated-argument
       reciprocal-sum shape, or the ``tanh``-from-``exp`` identity — in LOADED
       content, flagging a Director-review candidate at load or CI time rather
       than at declaration time. This is its own issue; the recommendation
       states so explicitly and is not implemented here.

       **Round-2 gate restoration addendum (2026-08-17 final whole-branch review,
       finding I2; fix-forward).** The verbatim-transcription claim above was
       INCOMPLETE at first landing: ``control-ratio/c04-terminal``'s original
       ``when`` transcribed only the readiness/latch gates
       (``control_ratio.py:124-125,154-159,166-168``) and silently dropped TWO more
       of the frozen ``step()``'s five early returns — ``if prisoner_pop == 0:
       return`` (``:141-142``) and ``if prisoner_pop <= max_controllable: return``
       (``:150-151``, the SAME ``<=`` boundary ``c03`` already transcribes) — both
       re-evaluated on the terminal tick against a freshly recomputed census, since
       ``step()`` is one function re-executed from the top on every call. Fixed
       forward by restoring both as added ``when`` conjuncts (``(> prisoner-population
       0)``, ``(> prisoner-population max-controllable)``, the latter needing a new
       ``control-capacity`` ``:const``/``max-controllable`` ``:expr`` binding pair);
       ``c04``'s ``:fuel`` moved 46 -> 54 (measured bound 53 + 1, §4.5). Proven by two
       new mutation-killed fixtures in ``control_ratio_conformance.rs`` —
       ``c04_does_not_emit_when_the_terminal_tick_census_has_zero_prisoners`` and
       ``c04_does_not_emit_when_the_terminal_tick_census_falls_back_within_capacity``
       — each cross-checked against the frozen engine (``control_ratio_conformance.py``'s
       ``WORLDS_WITH_PRIOR_CRISIS``), which emits nothing in both worlds when
       ``persistent_data`` is pre-seeded as though the crisis already fired. Every
       ``tick_goldens.rs`` pin stayed byte-identical — no committed scenario exercises
       either dropped-gate path. The RESERVED branch's own comparison, threshold
       source, and role partition are UNCHANGED by this fix; only the surrounding
       ``when``'s completeness moved.
   * - D181
     - §3.4
     - The ``*``/``/`` bullet is **split** (#491 T1, discharging ADR202 R1(c)'s
       ``E-TYPE-040`` half and OQ-I): ``*`` keeps ``E-TYPE-040`` for
       extensive × extensive (an area-of-an-area, the case that stays) and
       gains a licensed extensive × intensive case (result extensive — a
       stock scaled by a fraction or a rate, e.g. ``population ×
       per-capita-rate``, found live in ``lifecycle.bsl``'s committed
       ``new-wealth-d-prime`` binding while gating this repair); ``/`` gets
       its own rule, **extensive ÷ extensive → intensive**, no error —
       ``w̄ = wealth ÷ population`` is the textbook definition of an
       intensive quantity (density = mass ÷ volume), the same unit-algebra
       standing **D90** already took for the symmetric weighted-mean gap
       (§2.12: that row's own text names *why* the derivation had to go
       through the fold table instead of the ``*``/``/`` bullet — this row
       is what removes that obstruction). Intensive × intensive — a
       same-kind-squared combination the old bullet genuinely never named
       — stays conservatively refused, ``E-TYPE-040``, the bullet's own
       "deliberately conservative, Phase-1 review item" framing extended
       rather than invented. Extensive mixed with intensive under ``/``
       specifically, in either operand position, is **[review finding F5,
       #491 T1]** more precisely a NEWLY-MINTED prohibition, not a
       continuity: the pre-split bullet named that pair TWICE
       ("extensive if exactly one operand is extensive" and "intensive if
       exactly one is intensive" both matched it) and assigned it neither
       kind nor ``E-TYPE-040`` — this split resolves that internal
       contradiction conservatively, the same posture as the case the
       bullet did clearly name, but a genuinely new prohibition rather
       than an extension of one. ``if``
       now carries the same kind-neutral absorption rule ``+``/``-``/``*``/
       ``/`` already state explicitly — found live in the SAME ``lifecycle.bsl``
       rule's ``surviving-fraction`` binding, whose two ``if`` branches are
       Intensive and Neutral respectively; a strict "branches must match"
       reading would have wrongly rejected this committed content, and
       nothing in the bullet's prior wording forced that reading.
       Implemented: ``TypeCode::KindMixing``, ``rust/crates/babylon-bsl/
       src/typecheck.rs`` (``expr_kind``/``list_kind``/``add_sub_kind``/
       ``mul_div_kind``/``if_kind``/``fold_kind``/``check_kind_mixing``) —
       a SEPARATE walk from ``typecheck_aggregation``'s fold-specific one,
       extending the crate's existing typecheck dispatch rather than
       restructuring the fold arm; wired into ``rule_pipeline::load_rule``.
       ``ai/bsl-architecture-standard.md``'s error-surface table is
       repaired to stop grouping ``E-TYPE-040`` with the (already
       implemented) aggregation-law codes ``041``/``042``/``043`` as though
       all four shared one implementation. **Intensive × intensive's
       "stays conservatively refused" disposition here is SUPERSEDED IN
       PART by D182** — ``*`` only, ``/`` unaffected.
   * - D182
     - §3.4
     - ``*``'s intensive × intensive combination — refused by D181 as one of
       the "same-kind-squared" cases that bullet left conservative — is now
       **licensed**, result **intensive**: a rate scaled by a dimensionless
       coefficient is still a rate, the same "scale by a dimensionless
       factor" reasoning D181 already used for extensive × intensive,
       applied to the other pairing. Controller adjudication, 2026-08-18,
       under the Director's overnight compass delegation (delegated
       provenance, morning-reviewable) — the THIRD instance of the same
       defect class the Director had already ruled repair-now on (#491 T1's
       kind-straddle dossier, ``reports/kind-straddle-repair-options-
       2026-08-18.md``), found live in ``consciousness.bsl``'s committed
       ``p6-route``: ``delta-r = (* (* consumed eff-sol) routing-scale)``,
       where ``consumed`` (``agitation × consumption-rate``) and
       ``eff-sol`` (a solidarity/chauvinist-derived ratio) are both
       intensive, and the product feeds ``r1 = (+ r delta-r)`` — ``r``
       (``social-class/revolutionary``) is itself declared intensive, so
       the consumer already expected an intensive result. Unlike
       ``p5-agitation``/``solidarity``'s two straddles, this instance's
       correct repair sits in the ARM, not the content — VALUE-PRESERVING,
       no arithmetic changes, only the computed kind for an expression whose
       runtime result was always the same number. Licensed for ``*`` only:
       intensive ÷ intensive stays refused, undecided, not this ruling's
       question (D181's own text stands there unchanged). Implemented:
       ``mul_div_kind``'s ``(Intensive, Intensive) if op == "*"`` arm,
       ``rust/crates/babylon-bsl/src/typecheck.rs``. **This row's "intensive
       ÷ intensive stays refused, undecided" disposition is SUPERSEDED BY
       D183**, same sitting.
   * - D183
     - §3.4
     - ``/``'s intensive ÷ intensive combination — left refused,
       "undecided," by D182 — is now **licensed** too, result **intensive**:
       a ratio of two intensive quantities is a dimensionless share; simplex
       normalization (dividing a part by a whole built from parts of the
       same kind) is the canonical intensive operation this coarse two-kind
       algebra has a name for. Controller adjudication, 2026-08-18, same
       sitting as D182, same delegated Director provenance — the FOURTH
       straddle site: licensing D182's ``*`` arm exposed ``p6-route``'s own
       next site, found live in the SAME committed rule — ``r2``/``l2``/
       ``f2``'s simplex renormalization, ``r2 = (if (> total (+ 1 eps)) (/
       r1 total) r1)`` and siblings (``consciousness.bsl:344-347``), where
       ``r1``/``l1``/``f1``/``total`` are all intensive (each of ``r``/
       ``l``/``f`` plus the now-D182-licensed ``delta-r``/``delta-l``/
       ``delta-f``, summed by same-kind ``+``). Value-preserving, same
       reasoning and defect class as D182. Confirmed by a COMPLETE static
       sweep of every ``<arith>``/``if`` site across all 13 committed rule
       files (not just ``p6-route``) before landing — no other site the
       arm's shape after D182 would still refuse (the sweep script and its
       verdict: #491 T1's task report,
       ``.superpowers/sdd/2026-08-17-491-rung-ladder/task-1-report.md``).
       An extensive operand paired with an intensive one under ``/``, in
       either position, is a DIFFERENT, still-undecided question and stays
       refused (D181's own text). Implemented: ``mul_div_kind``'s
       ``(Intensive, Intensive)`` arm widened to cover both operators (no
       ``op`` guard needed — ``*`` and ``/`` now agree on this cell),
       ``rust/crates/babylon-bsl/src/typecheck.rs``.
   * - D184
     - §3.4
     - **S1 relocates a kind straddle rather than resolving it — recorded,
       not fixed, per review finding F2 (#491 T1's task review,
       ``.superpowers/sdd/2026-08-17-491-rung-ladder/task-1-review.md``
       §"F2").** ``solidarity/p0-transmit``'s S1 restructure (the convex
       combination ``(1 - strength) * target + strength * source``, D181's
       extensive × intensive licensing applied twice, summed same-kind)
       types **Extensive** end to end, and it is written whole into
       ``social-class/revolutionary``, declared ``probability intensive``.
       It loads today ONLY because this arm checks ``<arith>``/``if`` NODES
       and has no STORE-BOUNDARY kind rule — nothing in §3.4 as written
       compares a ``set`` expression's own computed kind against its target
       field's declared kind. The root cause this relocates rather than
       resolves is §2.9's language-mandated extensive implicit
       ``<edge-type>/strength`` field, untouched by S1 (the dossier's own
       §2.0 "Note on scope" already named this as out of a single rule's
       reach). **Re-open trigger, stated explicitly:** the natural next
       rung of this same arm — a store-boundary write-kind check, comparing
       a ``(set <expr>)``'s computed kind against its target field's
       declared kind — would immediately re-red ``p0-transmit``. Until such
       a check lands (or §2.9's implicit-strength default is itself
       revisited), S1 is arm-satisfying, not dimension-resolving; C1, by
       contrast, is coherent at both the arithmetic AND the store boundary
       (its own trace closes: every term of ``increment`` resolves
       Intensive, and ``social-class/agitation`` is itself ``real
       intensive``). No content or arm change lands with this row — it is
       the disclosure the review asked for, filed so the next write-kind
       rung inherits the context rather than rediscovering it.
   * - D185
     - N/A (game-content unit reconciliation, not a BSL grammar construct)
     - **#491 T2 — the subsistence-unit reconciliation**
       (``reports/subsistence-unit-reconciliation-2026-08-17.md``): three
       previously uncommensurable quantities —
       ``SurvivalDefines.default_subsistence`` (dimensionless ``[0,1]``,
       ``src/babylon/config/defines/survival.py:23-28``),
       ``EconomyDefines.base_subsistence`` (a per-member-per-tick rate,
       ``src/babylon/config/defines/economy_basic.py:267-275``), and
       ``s_bio + s_class`` (Currency per member per tick,
       ``rust/crates/babylon-tick/content/rules/vitality.bsl:49-50,74``) —
       are dimensionally reconciled: a STOCK (``wealth``, ``w̄ = wealth ÷
       population``) vs. FLOW (``s_bio``/``s_class``) mismatch the frozen
       engine's own ``coverage_ratio`` (``formulas/vitality.py:38-43``)
       already diagnoses, without licensing a transcription of its value.
       The declared unit system adds the mass rows: ``mass_k = (mass^hh_k
       · η_k) / Σ_j (mass^hh_j · η_j)``, the household→person crossing,
       whose whole content is ``η_k ≡ 1`` (rung-independent household
       size) — unmeasurable in-repo (no B25010/B11016/B19019-family
       table; ``fact_census_housing`` is tenure, not size; ``dim_county``
       carries no population column). Also named: the ``population == 1``
       fixture-default accident
       (``src/babylon/engine/systems/survival.py:128``;
       ``tests/unit/engine/systems/test_survival.py:45,134-138``) that
       collapses ``wealth_per_capita`` to ``wealth`` and masks the
       dimensional mismatch at the one
       population value where the two are numerically indistinguishable.
       **DP-7 = A** — the declared identity crossing (a content
       ``defconst``, dual ``mass_household``/``mass_member`` artifact
       columns with an empty ``person_equivalence``, and an Aleksandrov
       row) — ruled 2026-08-18 (posted #491), Director-delegated to the
       standing gameplay-and-pedagogy compass, controller-adjudicated.
   * - D186
     - N/A (game-content unit reconciliation, not a BSL grammar construct)
     - **τ, the subsistence horizon** (§4 of the same record). The frozen
       ``coverage_ratio ≥ 1 + inequality`` test cannot license τ:
       ADR183/ADR210 R13 rule the frozen engine a structure contract, not
       a threshold oracle, and κ's explicit frozen-magnitude licence (R14)
       does not extend to τ. Read at R13's own level-set split — mortality
       ``s_bio`` alone, acquiescence ``s_bio + s_class``
       (``formulas/vitality.py:29``) — preserving the frozen death level
       implies ``τ_bio = (s_bio + s_class)/s_bio``, class-varying and ≠ 1,
       re-importing ``s_class`` into the level set R13 just separated; and
       it contradicts H3's flow-derived ``L = reserve_army_stock ÷
       absorption_flow`` (producer hardcoded to ``0``,
       ``src/babylon/domain/economics/reserve_army/accumulation.py:133``).
       **DP-5 = A now, C a named revisit** — τ ≡ 1 tick ships as a
       definitional accounting identity (the tick is the reproduction
       period; no frozen authority invoked), with the H3-symmetric
       flow-derived horizon (option C) recorded as a named,
       not-yet-executed revisit at the ReserveArmy port once the
       absorption-flow producer exists. Ruled 2026-08-18 (posted #491),
       same provenance as D185's DP-7 ruling. τ's home stays unaffected by
       the disposition: a ``.bscn`` ``defconst``
       (``(defconst vitality/subsistence-horizon …)``), never a
       ``GameDefines`` field, for the identical ``defines_hash`` reason
       ADR210 R14 gives for κ.
   * - D187
     - §3.2
     - **The money-vs-money law** (§5 of the same record):
       ``E-EVAL-013`` as LAW, not merely an evaluator error code. Never
       compute ``S / w̄`` — ``Currency ÷ Currency → Coefficient`` must
       land in ``[0,1]`` (``bsl-language.rst:2533-2534``;
       ``rust/crates/babylon-bsl/src/evaluator.rs:1795-1817``) or the
       expression raises ``E-EVAL-013``. A class below subsistence has
       ``S · τ > w̄`` by construction, so the dimensionless spelling
       fails at runtime for exactly the class the measure exists to
       describe — invisible on every input where a class is doing fine,
       the happy path a conformance suite naturally exercises first, and
       only surfacing at the below-subsistence tail the whole project
       cares about. Worse than an ordinary bug for exactly that reason.
       The comparison is money-vs-money, always: ``cut_{k-1} · w̄ ≥ S ·
       τ``, never ``w̄ / (S·τ)`` against a normalised threshold.
   * - D188
     - N/A (game-content unit reconciliation, not a BSL grammar construct)
     - **The level-set assignment (ADR210 R13) and its owed divergence
       D-row** (§8.2 of the same record). R13
       (``ai/decisions/ADR210_checkpoint_a_campaign_rulings.yaml:153-159``)
       assigns mortality ``s_bio`` alone and acquiescence ``s_bio +
       s_class``, and explicitly owes a divergence D-row from the frozen
       ``s_bio + s_class`` death threshold (ADR183: the frozen engine is a
       structure contract, not a threshold oracle) — this row discharges
       that obligation. The frozen engine's actual mortality computation
       (``src/babylon/engine/systems/vitality.py:230-232``,
       ``_calculate_deaths``) sets ``subsistence_needs = s_bio + s_class``
       as its death threshold — the combined, acquiescence-side level
       set, not the ``s_bio``-alone level set R13 assigns to mortality.
       T5/T6, which build the Grinding-Attrition mortality rule proper
       under the split level sets, form the landing that executes this
       departure. (The currently-landed ``vitality.bsl``'s block-of-one
       Reaper ``consumption-needs = s-bio + s-class`` term, line 74, stays
       untouched — it guards the *starvation* branch, a different,
       deliberately narrower mechanism the file's own header, lines
       12-40, states does not yet transcribe Grinding Attrition.) Also
       recorded: #546 item 6 (county-varying subsistence) stays open and
       not foreclosed — ``s_bio``/``s_class`` are already per-class
       intensive fields, so a per-``(class, county)`` ``S`` needs zero
       redesign.
   * - D189
     - §1.5, §2.9
     - **T3 (#491, OQ-J), Half 2 of the typed-attribute-seeding design —
       Currency's node-attribute canonical section lands as tag ``0x06``,
       NOT the design doc's assumed ``0x05``: entries ``u64 id ‖ str name
       ‖ i128 value`` (16 raw bytes, big-endian, via ``i128::to_be_bytes()``),
       ascending ``(id, name)``, ELIDED WHEN EMPTY, ``CANONICAL_LAYOUT_VERSION``
       bumped 2 → 3.** The design doc
       (``reports/typed-attribute-seeding-design-2026-08-11.md`` §B) specced
       section ``0x05`` for Currency, written before the SEPARATE T3/#560
       edge-attribute train (D144) claimed that same tag for its own fifth
       section — a real cross-train collision, not a drafting slip: both
       trains independently reached for "the next tag after 0x04." RESOLVED
       by taking the actual next-free tag against this checkout at landing
       time (``0x06``), mirroring D144's own shape verbatim one tag over:
       the entry layout mirrors section ``0x02``'s ``(id, name)`` key (the
       SAME node-attribute space, partitioned by declared type rather than
       widened in place — a ``currency``-declared field never reports in
       BOTH ``all_attributes`` and ``all_currency_attributes``, mirroring
       D143's one-datum-one-hashed-home discipline for the strength/
       edge-attribute split); the elision rule is D144's, extended one
       section further (every FUTURE section follows this rule, per D144's
       own text). ``babylon-graph``'s two new ``GraphSubstrate`` methods
       (``update_node_currency``/``node_attribute_currency``) are
       NODE-scoped only — no edge/hyperedge Currency lane exists or is
       licensed by this row. Layout byte-pinned by
       ``the_sixth_section_is_pinned_byte_for_byte``; elision behavior-
       pinned by ``an_empty_currency_attribute_listing_writes_no_sixth_section_bytes``;
       every pre-existing golden (``tick_goldens.rs``, ``qa:regression``)
       left byte-identical, since no committed content declares a
       ``currency``-typed ``deffield`` yet.
   * - D190
     - §4.2, §2.8
     - **T3 (#491, OQ-J) — ``PendingWrite``'s reduced operand widens to a
       SUM TYPE (``WriteOperand::Real(f64) | WriteOperand::Currency``), not
       a parallel Currency-write batch: the SAME reasoning D145 already
       settled for ``WriteTarget``, applied one field over.** Currency's
       typed storage needs ``update-node``'s collected operand to sometimes
       carry an ``i128`` value rather than an ``f64`` one, and the batch's
       own documented algebra (the free-monoid-collect /
       monoid-action-apply split D145's own text states) again picks the
       shape: a SEPARATE ``Vec<PendingCurrencyWrite>`` could not preserve
       the single interleaved collection order the application law
       requires, exactly the argument that ruled out a parallel
       ``Vec<PendingEdgeWrite>`` in D145. A Currency ``set`` sits in the
       identical batch position an ``f64`` ``set`` would; only ``set`` is
       licensed on the Currency lane (``add``/``sub``/``scale`` would need to
       pick which of Currency's five legal §3.2 operators applies, which
       this row does not license — mirrors the enum lane's identical
       ``set``-only narrowness, §2.13). ``EffectExecutor::update_node``,
       ``collect_update_node`` and ``apply_pending_write`` each fork on the
       field's declared type BEFORE reaching the ``f64``
       ``numeric_write_value`` lane; ``update-edge`` takes no such fork —
       there is no edge-scoped Currency lane, so a ``WriteOperand::Currency``
       reaching the edge arm of ``apply_pending_write`` is a named
       collect/apply wiring-bug error, never reachable from content.
   * - D191
     - §2.8, §3.2
     - **T4 (#491, Phase 1 — "the carrier, inert"; ADR194 R1) — the K=16
       wealth-mass carrier's declared shape and its two governing laws,
       I1 and I2.** ``content/scenarios/vitality-attrition-conformance.bscn``
       declares sixteen ``social-class/wealth-mass-01``…``-16`` fields
       (``coefficient intensive`` — a per-member SHARE, never an unweighted
       cross-class mean, §3.4), fifteen ``wealth-sketch/cut-01``…``-15``
       defconsts (``Ratio``, strictly positive or ``E-LEX-027``, §1.5
       addendum/#492/ADR194), the household→person crossing
       (``wealth-sketch/household-person-equivalence``) and the subsistence
       horizon (``vitality/subsistence-horizon``) — both ``Ratio``
       defconsts precisely so neither moves ``defines_hash`` (the same
       reasoning ADR210 R14 gives κ). Unit-system authority for every one
       of these constructs is
       ``reports/subsistence-unit-reconciliation-2026-08-17.md`` (the #491
       T2 deliverable) — this row cites that record rather than
       re-deriving it. This row declares **I1** — ``Σ mass_k = 1`` exactly
       in the stored ``f64`` — law for the carrier's own posture (pinned,
       ``vitality_attrition_conformance.rs``); its residue-apportionment
       RULE is **largest-remainder, never the open top rung** (design doc
       §5.4 item 2/§5.3c M-5 — the top rung's midratio is the most
       assumption-laden quantity in the eventual grid, the worst possible
       silent sink) — this row registers the rule here because the
       carrier is where I1 first governs committed content; the actual
       apportionment COMPUTATION belongs to A2's generator, T7/T8, which
       has not landed. This task's own conformance fixture satisfies I1 by
       construction (every class's nonzero masses are binary64-exact
       dyadic fractions summing to exactly ``1.0`` with no residue to
       apportion at all), so no apportionment code exists yet to test — a
       gap named, not hidden. **I2** — the fifteen-cut grid is monotone
       non-decreasing and strictly positive — is LOAD-TIME checked by this
       task's own posture suite (not by the reader or the scenario loader,
       which impose no ordering relationship BETWEEN separate ``defconst``
       forms); a future content pack declaring a decreasing grid loads
       clean today and only a consumer's own assertion (or, later, a
       dedicated load-time check T7/T8 may add) would catch it — recorded
       as the current state, not a claim of a language-level guarantee.
       **The Currency lane.** This scenario is the FIRST committed content
       in this crate to declare a ``currency`` node attribute
       (``wealth``/``s-bio``/``s-class``, re-seeded from
       ``vitality-conformance.bscn``'s int/real originals, T3/OQ-J) — so
       ``CanonicalState`` section ``0x06`` (D189/D190) MATERIALIZES for
       this world; its bytes are a NEW, first-measured pin
       (``tick_goldens.rs::vitality_attrition_carrier_hashes_are_pinned``),
       never derived from any of the eighteen pre-existing pins, whose
       worlds seed no currency fields and stay byte-identical (verified,
       both directions, by the full ``babylon-tick`` suite before and
       after this row's own commit).
   * - D192
     - §3.4, §5.4
     - **T4 (#491, Phase 1) — I3 and I4's status at the carrier; the
       absence fence.** **I3** (``Σ mass_k · midratio_k = 1`` against a
       universal midratio grid) is NOT tested by this task at all — design
       doc §5.3c (C-3 repair) demotes it to a PUBLISHED RESIDUAL with a
       declared bound, never an exactness claim, because a determinate
       county re-binning under a universal grid satisfies I1 by
       construction and cannot also meet I3 exactly except by coincidence.
       That residual (``mean_consistency_residual``) is A2's own artifact
       column, T7/T8, which has not landed; this row exists so a future
       reader does not go looking for an I3 check in the carrier itself
       and wrongly conclude the task skipped one — none belongs here.
       **I4** — a ``coefficient``-declared field's value must land in
       ``[0,1]`` at the store boundary or the write is ``E-EVAL-020``
       (§3.3, pre-existing law, not new to this task) — already covers
       every ``wealth-mass-*`` field by virtue of its declared type alone;
       no new mechanism, cited here rather than re-tested, since the
       existing store-boundary check has its own coverage elsewhere in
       this document and this crate. **The absence fence.** ``remnant``
       (the fixture's sixth class) seeds NONE of the sixteen mass fields —
       every direct read is ``Err`` (III.11: absence is a value, never a
       fabricated zero), and the pack's own posture test computes the
       optional-bind-style sum (each absent field read as ``0.0``, the
       SAME semantics a future ``:optional :default 0.0c`` BSL binding
       would apply) and asserts the sum is exactly ``0.0`` — "no
       distribution," never a fabricated uniform share. This is the SAME
       idiom H1 (design
       doc §6.2) cites by name: ``content/rules/consciousness.bsl``'s
       UNPOSITIONED pattern (``:optional`` + ``:default 0.0p`` + an
       explicit sum-guard), applied to the mass carrier one construct over.
   * - D193
     - §3.5, §6.3
     - **T4 (#491, Phase 1) — the ``:default`` allowlist posture, and this
       task's own scope boundary.** This task ships NO rule binding
       ``:optional``/``:default`` against the sixteen mass fields — the
       carrier's own probe rule
       (``content/rules/vitality-attrition.bsl``) binds only
       ``social-class/population``, a plain required read, so
       ``default_lint.rs``'s allowlist finding is not raised by anything
       this task commits. Recorded here as a FORWARD CONTRACT for T5
       (Phase 3a) and T6 (Phase 3b), whose rules WILL bind these fields
       ``:optional :default 0.0c`` (the absence-fence idiom D192 names):
       this row expects those bindings to ride the SAME non-blocking lint
       posture ``content/rules/consciousness.bsl``'s own ``:default``
       bindings already do (``default_lint.rs``'s module doc: "a lint
       failure requiring Director sign-off — not a load error"), and
       neither this task nor a future one may add a ``DEFAULT_ALLOWLIST``
       row without that sign-off — the allowlist is program state, not
       language state. **Scope boundary, stated explicitly per design doc
       §12 item 1:** this task supplies the carrier ONLY. It does not
       discharge OQ-D — the question of whether a population measure over
       an intra-class distribution belongs among Axiom A0's enumerated
       G-members (``ai/bsl-architecture-standard.md:684``) — which stays
       open, potentially Amendment-AE(ii)-gated, exactly as it was before
       this task ran.
   * - D194
     - §6.2, §3.4
     - **T5 (#491, Phase 3a) — the H2' dual pair lands as the ladder's
       declared resolution, replacing OQ-B's single-quantity reading.**
       ``vitality/subsistence-clearing``
       (``content/rules/vitality-attrition.bsl``) computes ``clearing``
       (STEP, rungs 2..16 against ``cut_{k-1}``, ``>=``-inclusive) and its
       dual ``failing-certain`` (rungs 1..15 against ``cut_k``, strict
       ``<``) over the SAME fifteen ``cut-01``..``-15`` grid at opposite
       edges; ``straddle-band = mass-sum - clearing - failing-certain``
       (complemented against the BOUND total, never a stipulated ``1``, so
       a partially-seeded mass vector cannot fabricate unseeded mass)
       publishes the one rung the threshold actually cuts through rather
       than folding it into either side (C-7 repair). Rung 1's lower edge is
       the implicit, unspellable ``0`` — ``0.0r``/negative is
       ``E-LEX-027`` (D191's own citation, re-confirmed here) — so
       ``mass-01`` NEVER contributes to ``clearing``, by construction (no
       ``cut-00`` binding exists), not by an omission a reader must take
       on faith. **The bottom-rung floor this repairs, measured** (§6.2
       of the design doc, cited here as the register's own record of the
       numbers): national bottom-rung mass 5.04%, county median 4.92%,
       county max 35.68%, min 0 — revision 1's single-quantity reading
       would have read EVERY one of those shares as certain, permanent
       failure; the dual pair instead reads it as UNRESOLVED
       (``straddle-band``) wherever the grid cannot establish either
       fate. Both duals are exposed via ``emit`` only — see D197 for why
       neither reaches graph state.
   * - D195
     - §6.2
     - **T5 — the H3 hold-out-horizon identity, as a contract.** Dividing
       H2's rung condition through by ``S`` gives ``H_k = (cut_{k-1} *
       w-bar) / S``, rung *k*'s own hold-out horizon in TICKS, with
       ``c_k = 1 iff H_k >= tau`` — so ``clearing(S, tau)`` reads
       identically as "the mass whose hold-out horizon reaches tau."
       This is EXPOSITION ONLY: the rule never computes that quotient,
       because ``S / w-bar``-shaped division lands outside ``[0,1]`` for
       exactly the below-subsistence class and trips ``E-EVAL-013``
       (D187's money-vs-money law, cited not re-derived) — the
       implementation stays money-vs-money throughout, ``cut * w-bar >=
       S * tau``, never the other order. **The contract, so a future
       consumer does not re-derive a curve under a different name:**
       ReserveArmy's wage-pressure ``L`` and Allegiance/FascistFaction
       each read the SAME ``clearing``/``failing-certain`` construction
       at their OWN horizon in place of ``tau`` — one measure, many
       horizons, never a second implementation. This rule reads only the
       tick's own horizon (``vitality/subsistence-horizon``, DP-5 = A);
       landing the OTHER horizons' consumers is future work this row
       does not claim to discharge.
   * - D196
     - §9/T5.1(3)
     - **T5 — ADR202 R2 is CARRIED, not satisfied, by this train (DP-10),
       and the limitation is structural, not a shortfall in effort
       (C-6).** R2 asserts more intra-class dispersion implies less
       switch-like rupture; ``vitality_attrition_conformance.rs``'s
       ``adr202_r2_more_dispersion_means_a_less_switch_like_transition``
       test proves the sign HOLDS for the measure's own algebra, in a
       hand-authored fixture where masses are free to vary. It cannot
       prove it against the SEEDED world: A2 gives one wealth-sketch
       shape per county, shared by every class in that county, so
       intra-class dispersion is a COUNTY CONSTANT there — no
       class-varying degree of freedom exists for this sign to be right
       or wrong about. The real-data companion is T8a.6, not this train.
   * - D197
     - §3.2
     - **T5 — the real-lane population finding, measured empirically (the
       same species of gap T4.3's Currency-drain spike found, one
       operator over).** ``w-bar = wealth / population`` reads as
       licensed by the KIND axis alone (D181: extensive ÷ extensive ->
       intensive) — but ``tick.rs::bind_field_value`` renders EVERY
       non-enum, non-currency field as ``Value::Real`` at runtime
       regardless of its declared type (D101), so an ``int``-declared
       ``population`` is ``Value::Real`` at the evaluator, and a plain
       ``(/ wealth population)`` is ``Currency / Real`` — refused
       unconditionally as ``E-TYPE-030`` (measured directly against this
       rule: no ``Currency / Real`` fallback exists for ``/``, unlike
       ``*``'s in-``[0,1]``-coefficient arm). **Fix, the same tool
       ``vitality.bsl``'s own header already names for Grinding
       Attrition's identical blocker:** ``(intrinsic floor :params (real)
       :returns int :cost 5)`` demotes the real-lane read to a genuine
       ``Int`` (ADR188 Row 2, D97; exact for a field that is always
       non-negative and integer-valued), and ``w-bar`` divides by THAT,
       landing on the pinned "÷ integer" leg. **Cost, disclosed:**
       ``expr_kind`` cannot see through an intrinsic call, but the kind
       story is subtler than "undetermined": ``population-int``'s else
       branch is the kind-neutral literal ``0``, and ``if_kind``'s
       single-determined-branch arm propagates it, so ``population-int``
       resolves kind-neutral and every binding downstream of ``w-bar``
       (all fifteen ``edge-k`` terms) carries a DETERMINED kind by
       neutral-literal absorption — determined-but-not-validated: the
       determination flows from the fallback literal, never from checking
       what ``floor`` actually returns. The values are unaffected; the
       static kind-mixing net still covers these bindings' downstream
       uses, but a kind error hidden inside the intrinsic path itself
       would pass unnoticed. **Co-load hazard, disclosed:** ``territory.bsl`` and
       ``decomposition.bsl`` already declare a byte-identical
       ``(intrinsic floor ...)`` and collide with each other under
       ``E-LOAD-001`` if ever loaded together (filed as #646); this rule
       joins that same collision SURFACE but not (yet) its live blast
       radius — ``vitality-attrition.bsl`` loads alone, paired only with
       ``vitality-attrition-conformance.bscn``, in every test this crate
       runs against it today.
   * - D198
     - N/A (game-content derivation record, not a BSL grammar construct)
     - **T6.3 — κ derived, not picked (ADR210 R14's obligation
       discharged).** ``vitality/kappa`` is a ``1.0c`` defconst in
       ``vitality-attrition-conformance.bscn``, fitted at the ``calibration``
       node — the frozen engine's own canonical total-attrition conformance
       point (``coverage_ratio = 1.0``, ``inequality = 0.8``,
       ``population = 100``;
       ``tests/unit/formulas/test_vitality.py::test_coverage_below_threshold_causes_attrition``,
       rate clamped to 1.0) — the one point where the frozen form
       (``deficit × (attrition_base_factor + inequality)``,
       ``formulas/vitality.py:35-49``) and the ported form agree in
       SEMANTICS, not just magnitude ("everyone certainly failing dies
       this tick"). The fixture is chosen with ``s_class = 0`` so the
       frozen level set (``s_bio + s_class``) and the R13 measure's level
       set coincide at the reference point and κ absorbs no level-set
       contamination (design doc §3.5 requirement 2); all its mass sits in
       rung 1, wholly below the threshold (``cut-01 × w-bar = 0.18 <
       s-stock = 1.0$``), so ``failing-certain = 1.0`` exactly by
       construction and no measurement noise enters the fit. κ =
       frozen-rate₀ ÷ failing₀ = 1.0 ÷ 1.0 = **1.0c** — exact, no rounding
       epsilon, no clamp on the ported side needed (the product
       ``failing-certain × κ`` stays in ``[0, 1]`` by construction).
       Replaces ``attrition_base_factor``, which ADR191 R3 rules NOT
       transcribed — its slope-shaping duty is retired into the measured
       distribution, and this derivation record is what makes that
       retirement honest. **The divergence surface is published, not just
       the fit point** (§3.5 requirement 3):
       ``rust/crates/babylon-tick/content/scenarios/vitality_attrition_conformance.py``'s
       ``print_divergence_surface()`` prints frozen vs ported
       deaths-per-tick rates over a declared ``(coverage_ratio,
       inequality)`` sweep — the ported column (over the declared uniform
       reference distribution) is g-flat and step-shaped in coverage,
       where the frozen columns bend smoothly and steepen with inequality
       (which enters the frozen threshold AND slope; the ported form has
       no inequality input — §3.3b's retired dispersion surrogate). Exact
       at the reference by construction; the table is the record of
       everywhere else. The same oracle's ``mortality_expectation`` is the
       provenance of the T6 suite's pinned vectors.
   * - D199
     - N/A (game-content divergence record, not a BSL grammar construct)
     - **T6.4's DP-6 = B departure from OQ-H, recorded (the plan's "one of
       the two rows ships; never neither").** The mortality driver in
       ``vitality/subsistence-mortality`` is ``failing-certain`` — H2′'s
       dual, the mass the ladder can establish CANNOT reproduce itself —
       NOT OQ-H's originally-ruled ``failing = 1 − clearing`` (DP-6 = B,
       the delegated Director provenance posted on #491, 2026-08-18). The
       departure's content: under OQ-H's form the unresolved straddle band
       would silently count as failing (``1 − clearing`` folds it in);
       under the shipped form deaths follow what is ESTABLISHED, never
       what is unresolved — the straddle band is published as
       ``straddle-band``, never silently assigned. Because the driver is
       re-derived in-rule from the same H2′ chain (BSL has no cross-rule
       binding reuse), the two rules cannot disagree about the measure
       they share. The option-A alternative (and its rung-1 floor, with
       measured magnitudes national 5.04 % / county median 4.92 % / max
       35.68 %) is NOT shipped, and this row is the record that the choice
       was made knowingly.
   * - D200
     - §2.9
     - **``:max-members`` is DERIVED FROM THE SEEDED POPULATION, never
       stipulated** (Community port train, Task 4, 2026-08-21; the plan's
       ``D-NF+22``). The driver's ``CardinalityCeilings`` now chains
       ``LoadedScenario.hyperedge_types`` into the ceiling map as
       ``HyperedgeType/<member>`` entries and feeds
       ``max_members_seen`` — the longest member list of that hyperedge
       type in the world being loaded — into the ``:max-members`` axis
       (``lib.rs::build_shared_load_inputs``; the prefix lands at that
       seam, the census maps stay member-keyed mirroring
       ``node_types``/``edge_types``). Rationale, with provenance: (a)
       symmetry with the landed law — ``node_types``/``edge_types``
       already take ceilings from "the population the scenario ACTUALLY
       built … rather than an invented one" (``scenario.rs``'s own
       census-map doc), and a stipulated hyperedge cap would be the only
       invented number in the map; (b) the Director's
       gameplay-and-pedagogy compass — community membership is *content*
       (which classes belong to NEW_AFRIKAN is the political claim the
       pack teaches), and a modder's world must be bounded by the
       modder's world, never by a constant no player can see; where it
       does fail — a world larger than a rule's declared ``:fuel`` — it
       fails loudly at LOAD with a number to raise (III.11); (c) the fuel
       consequence stated plainly: a rule's static bound is per-world, so
       a pack's declared ``:fuel`` is the maximum over every world that
       loads it, re-measured whenever a world is added. **Rejected
       alternative:** a content ``manifest`` form declaring
       ``:max-members`` (``manifest.rs``'s route) — no landed content
       declares a manifest at all today, and that route would put an
       invented cap in fixtures. Re-open trigger: the first content set
       that genuinely needs a cap larger than any world it ships with (an
       unbounded runtime population). Mutation-proven at landing: the
       empty-map restoration reds all three ``hyperedge_ceilings.rs``
       proofs on E-LOAD-042, the dropped ``HyperedgeType/`` prefix reds
       the type-wide-head proof, and a constant 99 in the axis moves the
       measured E-LOAD-040 bound off its pinned 15.
   * - D201
     - §2.6
     - **Slice 3's cross-kind Ord ruling and the two-table move**
       (Community port train, Task 3, 2026-08-22; the plan's ``D-NF+24``).
       ``query::Element`` gains ``Hyperedge(HyperedgeId)`` declared
       **THIRD**: ``Node`` sorts before ``Edge`` sorts before
       ``Hyperedge``, by declaration order — arbitrary, deliberate, tested
       (the companions ``edge_sorts_before_hyperedge_regardless_of_id``
       and ``the_three_kind_order_holds_regardless_of_ids``, plus the
       compile-time trap this enum already carries: an exhaustive match,
       no wildcard, so a new variant breaks compilation at the pin
       before a reviewer can notice). The three §2.6 hyperedge heads are
       served by ``query::materialize`` — ``hyperedges``/``hyperedges-of``
       yield ``Element::Hyperedge`` in ascending ``HyperedgeId`` order
       (D25 — the substrate's own contract), ``members-of`` yields plain
       ``Element::Node`` in ascending ``NodeId`` order. **The two-table
       move:** ``query.rs``'s ``UNSERVED_QUERY_HEADS`` is DELETED with its
       refusal arm (a zero-row table would be the plan's own M4 failure
       mode — a duplicated table named once; the at-the-byte decision,
       recorded here), and ``evaluator.rs``'s ``UNSERVED_EXPRESSION_HEADS``
       drops the three heads (``metric-of`` stays; ``membership-field-of``'s
       "slice 4" note is amended to cite #653 by number, per the plan's
       own instruction). The query-operand-only law holds unchanged: a
       served head in bare ``<expr>`` position still refuses with the
       §2.7 text (pinned by
       ``hyperedge_query_heads.rs::a_served_head_in_bare_expression_position_still_refuses_with_the_2_7_law``).
       Mutation-proven at landing: inverting the materializer's order reds
       the ascending-id pin; re-declaring ``Hyperedge`` first reds both
       Ord companions; dropping the served-heads' bare-expr guard reds the
       §2.7 position pin. **Added at PR review (#684):** ``members-of`` now
       checks the annotated type against the referent's ACTUAL type at
       materialize time — ``E-EVAL-032`` (``HyperedgeTypeMismatch``, D24's
       pre-minted code) — via the new read-only
       ``GraphSubstrate::hyperedge_type_of`` accessor (both backends, two
       conformance rows); without it a referent of a different type would
       iterate past the annotated type's census-fed max-members bound
       unchecked. The check is a soundness obligation, not a filter.
   * - D202
     - §2.8, §2.10, §2.5
     - **Task 6's read/write surface for hyperedge own-fields** (Community
       port train, E2b, 2026-08-22; the plan's ``D-NF+25``).
       ``update-hyperedge`` is **served in BOTH dispatch sites** — the
       execute path and the collect path defer through
       ``WriteTarget::Hyperedge`` with the same ``set``/``add``/``sub``/
       ``scale`` parity and enum-``set`` inclusion as the other two update
       verbs, applying in the APPLY phase against the Task-5 substrate lane
       (the pre-state law holds for this element kind exactly as for the
       other two), and recording ``Write::HyperedgeAttribute``. The §2.10
       discipline-1 referent check is ONE function,
       ``evaluator::check_hyperedge_referent_type``, shared by the write
       side and by ``field-of``'s new ``HyperedgeRef`` arm — which serves a
       hyperedge's OWN declared field (Task-5 storage), retiring the
       Task-1-era "not meaningful" refusal; the surviving membership-payload
       lane cites **#653**, never "slice 4". **The D29 mechanism** (§2.5's
       node-scoped-binding law, the Community plan §8c guard 2's machinery):
       ``tick::subject_type_of`` gains the owner-kind filter, threaded the
       scenario's ``ClosedVocabulary`` through ``run_tick`` — a ``:field``
       binding whose qname resolves to an EdgeType/HyperedgeType owner is
       INVISIBLE to subject derivation, so a rule whose only field bindings
       are hyperedge-owned refuses with the subject-type error instead of
       iterating an empty population in silence. The filter acts on a
       **positive** owner-kind answer only: a segment the (possibly
       partial) vocabulary does not know passes through — unknown-owner is
       E-LOAD-023's lane at load, never this filter's, and filtering it
       would empty every subject population of every scenario declaring a
       vocabulary for one kind only. ``(hyperedge-attr <name> <qname>
       <literal>)`` seeds a declared own-field at hydration, mirroring
       ``(edge-attr …)``: the local name resolves through the
       name→``HyperedgeId`` table Task 1 grew for exactly this consumer,
       the owner check reads the hyperedge's ACTUAL type back from the
       substrate (``hyperedge_type_of``), the ``(hyperedge, field)`` pair
       is a KEY (``E-LOAD-057``), and the value converts through
       ``attribute_value``'s one per-type literal law (the int/fractional
       contract and the enum-ordinal lane included; Currency refuses — no
       hyperedge-scoped Currency lane, the edge lane's own ruling one kind
       over). Hydration writes carry no write log, exactly as
       ``(edge-attr …)``'s direct ``update_edge`` does not. **Typecheck
       parity:** the wrong-scalar write refuses at ``numeric_write_value``'s
       one funnel and the enum-field read in arithmetic position refuses at
       ``apply_arith`` (``bind_field_value`` renders ``Value::Enum`` — the
       D102 discharge one element kind over); the *static* enum-arithmetic
       check stays ``update-node``-only, ``update-edge``'s own parity
       level, deliberately not widened here. Mutation-proven at landing:
       killing the execute arm reds both execute-side proofs
       (``evaluator::tests::update_edge_and_update_hyperedge_are_both_served``,
       ``structural_verbs::tests::update_hyperedge_writes_a_declared_hyperedge_field_on_both_dispatch_sites``
       site 1); killing the collect arm reds the collect-side proofs (site
       2 and the content-driver
       ``hyperedge_surface.rs::update_hyperedge_writes_through_the_tick``).
       Mint-time ``<field-init>``s on ``add-hyperedge`` STAY refused, with
       the reason corrected: the storage exists; the init sugar is not
       routed to it in this train.
   * - D203
     - §4.2, §2.8
     - **The Community pack's census decomposition (Community port train,
       Task 8, 2026-08-22; the plan's D-NF+3/D-NF+25).** ``community.bsl``
       c00-c04 port frozen ``CommunitySystem``'s census + org-landscape
       weighting (``community.py:392-461``) as: per-class org accumulators
       (``org-{r,l,f}-weight``, ``org-count``, reset per tick by c02) pushed
       by three tendency-partitioned org rules (``c03f``/``c03l``/``c03r`` —
       the tendency gate is a rule-level ``when`` because fold bodies are
       bare accessors (D138); frozen's tendency-less org skip
       (:405-407) is inexpressible as a guard, and no world may seed a
       tendency-less org), then divided by the SAME-TICK census into the
       hyperedge raw accumulators by c04. **c04's ``active`` gate is ruled
       fidelity, not caution**: frozen builds ``community_agents`` from the
       active-only membership set (``community.py:472-474`` → :392-397), so
       an inactive class's org weights must never enter the sum — c03's
       push onto an inactive class stays inert under the gate, and c02's
       ungated reset clears it next tick. **c03 byte order**: rule-id byte
       order runs ``c03f`` → ``c03l`` → ``c03r``, unobservable because the
       three rules write disjoint weight fields and the shared count is
       integer-exact additive — the mirror transcribes the true order and
       is byte-identical either way (proven at Task 7). **Quantization
       divergence recorded here too** (found by the Task 7 corroboration
       artifact): frozen snaps every Probability to the 10⁻⁶
       ROUND_HALF_UP grid at the Pydantic boundary
       (``kernel/math.py:41``, ``_PRECISION = 6``); this pack's lane stores
       the raw f64 chain, so the mirror — never frozen's stored prints —
       is the oracle. Seven mutation vectors proven at landing: dropping
       c01's active gate reds the census and every downstream divisor test;
       c03r iterating all classes (no edge-driven skip) reds the four
       weight-total tests; cadre+cohesion for cadre×cohesion reds exactly
       the two l-weight readers; a literal c04 divisor reds exactly the
       divisor test; deleting c02 fails the tick loudly (E-EVAL-031 — the
       reset is what makes the accumulators exist; the compounding-across-
       ticks half is Task 10's arc-world vector); collapsing c03l into
       c03r fails at LOAD (E-LOAD-040: the merged rule's static bound 74
       exceeds the split rules' declared 72 — the partition is load-bearing
       before any value moves); deleting c00 fails the tick the same way
       (c01's add reads a never-written field). World 1's fired arithmetic:
       18 = c00:1 + c01:4 + c02:5 + c03f:1 + c03l:1 + c03r:2 (n9 FIRES with
       an empty for-each — frozen's :421 skip is structural) + c04:4.
       Per-rule fuel, measured by `bsl-fuel-report` over world 1 and
       declared at bound+1 (D-NF+22 — per-world bounds, since
       `:max-members` is census-derived): c00 79, c01 22, c02 21,
       c03f/c03l/c03r 72 each, c04 115. (c00 was 81 at first landing; the
       #688 harvest dropped the anchor's comparison guard to `#t`, the
       computed bound fell to 78, and 79 is the declared figure now.)
   * - D204
     - §4.2, §2.8, §6.2
     - **The c06 split and the ADR214 floor's landing (Community port
       train, Task 9, 2026-08-22).** The plan's c06 was ONE rule — "the
       pack's ONLY 14-arm dispatch" driving the ``r < floor``
       redistribution — but the pre-state law (§4.2 C4) makes a
       write-then-read WITHIN one rule's body impossible: a floor written
       by ``update-hyperedge`` is invisible to a later operand's
       ``field-of`` in the same collect pass. The dispatch therefore lands
       as TWO rules: ``c06a-floor-dispatch`` (the pack's only 14-arm
       ``community/kind`` chain, writing the per-community
       ``community/substrate-floor`` cache) and ``c06b-floor-redistribute``
       (reading it the same tick — a cross-rule read, the §8b shape spike
       (f) proved at Task 7). The math is frozen's
       (``formulas/consciousness.py:98-107``) unchanged; the pack's rule
       count moves 14 -> 15, and DG-2's "12 if no-publish" becomes 13.
       §8a gains a row: c05's total/unorganized expressions are inlined
       per write (no ``defexpr`` exists) — the bit-exact mirror assertions
       are the copies-agree mechanism. §8b gains a row: c06b reads c06a's
       cache (the skip gate keeps a skipped community's cache UNWRITTEN,
       honest-null — §9 item 5's law one field over). **The landing
       evidence:** world 2 (``community-floor-conformance.bscn``) binds
       NEW_AFRIKAN (0.136) and FIRST_NATIONS (0.155) on one tick — ADR214
       Ruling 3's ordering EXECUTED — with h0's nonzero f arm proving the
       redistribution's proportionality, and SETTLER as the
       identically-0.0 control; world 3
       (``community-degenerate-conformance.bscn``) fires the degenerate
       branch (zero-cadre org; c05 emits (0,1,0), c06 routes the floor
       bit-identically to frozen's fused branch — §6.2 I5) and the no-org
       skip gate (the prior ternary preserved byte-exactly). The
       ``floor_redistribution_handles_zero_lf`` pin drives c06b's else arm
       through a test-scoped rig rule (byte-ordered c05 < c05z < c06a):
       the arm is unreachable from c05's normalized outputs BY
       CONSTRUCTION (r < floor <= 0.18 forces l+f > 0.82), so the only
       honest coverage is synthetic. **Fuel is per-world** (D-NF+22 —
       ``:max-members`` is census-derived): measured by ``bsl-fuel-report``
       over all three worlds, declared at max+1: c00 104, c01 27, c02 21,
       c03f/c03l/c03r 72, c04 151, c05 596, c06a 252, c06b 272 (world 2's
       wider census is the worst case for c00/c01/c04-c06b; world 1 for
       c03\*). World-1 fired arithmetic after the extension: 21 = 18 +
       c05:1 + c06a:1 + c06b:1. Mutation vectors (six, all proven): the
       NEW_AFRIKAN arm swapped to floor-chicano reds both world-2 floor
       pins; a one-world FN-table typo (0.155 -> 0.145) reds the
       cross-world parity test AND the ordering pin; the CHICANO arm
       swapped to floor-elder reds both world-3 pins; the converse —
       replacing the chain's final else with a 0.99p literal — stays GREEN
       on all twenty tests (no world seeds ELDER: the unexercised arms
       provably cannot fire); dropping c06b's ``/ lf`` reds the
       proportionality pin; gutting the epsilon quotient to ``(< T 0.0c)``
       fails the degenerate world at the driver (0/0 NaN refused
       non-finite) and every floor pin with it. c07/c08 are DG-2-GATED and
       do not exist until the Director's ruling — §8a row 4's epsilon
       independence (c05's total vs c07's per-component entropy) is stated
       but only c05's half is testable this task.
   * - D205
     - §4.2, §1.2
     - **Cost modifiers, the decay, and the five Task-10 worlds (Community
       port train, 2026-08-22).** ``c09-cost-modifier-reset`` and
       ``c10-cost-modifier-accumulate`` BOTH carry the ``(= active 1)``
       guard frozen's ``_collect_memberships`` imposes
       (``community.py:472-474``, the C4 law — an inactive class is
       written NOTHING, the honest-null read, never 1.0); c10's product
       compounds in ascending ``HyperedgeId`` order (D25, D-NF+13).
       ``c11-state-decay`` writes heat/cohesion/education-pressure as
       ``max(0, x·(1−α))`` per community per tick with NO skip gate
       (frozen's decay loop, ``community.py:648-675``, runs over the whole
       dict); the infrastructure arm is ABSENT — §5's hard sequencing: its
       CORE_ORGANIZER maintenance term makes frozen's law non-monotone and
       waits on #653 (DG-7). **The world-6 divergence, recorded against
       the plan's own phrasing:** the plan's §1.2 L2 analogue claimed the
       all-inactive world is "a byte-exact no-op on every community
       field" — its own c11 row (ungated) and c00's reset writes
       contradict that. Frozen's early return (``community.py:337-338``)
       is a GLOBAL control-flow shortcut no per-rule pack reproduces, so
       the honest pins are the LANES: the census reads 0, the seeded
       simplex is preserved byte-exactly, no cost modifier is written, AND
       the decay applies (heat 0.5 → 0.475) — the test is named
       ``all_inactive_members_skip_every_membership_lane``, not the plan's
       name, which would assert the contradiction. **L1's analogue** (no
       config) is a LOAD REFUSAL, not a no-op: a world with no COMMUNITY
       hyperedge supplies no ceiling, and every carrier rule fails
       ``E-LOAD-045`` at load — the estate's preferred direction (fail
       loudly), pinned as a refusal test. **The seam world (5b)** co-loads
       solidarity.bsl + community.bsl over the landed solidarity world
       plus the community additions: every SOLIDARITY strength is
       byte-identical against the solidarity-only run (the amplification
       half that would move them is #653-blocked), and
       ``seam_world_community_half_actually_ran`` proves the seam cannot
       pass vacuously (heat decayed). **The carrier-collision world (5c)**
       co-loads control-ratio.bsl + community.bsl over ONE INSTITUTION
       node carrying both packs' anchors — per-rule-id fired arithmetic 37
       (control-ratio 9 + community 28, measured via ``per_rule_fired``)
       and the decay-applied-once read (0.475, never 0.45125).
       **Fuel is per-world** (D-NF+22): measured by ``fuel_bound_report``
       over ALL EIGHT worlds (the 5c co-load's ceilings included),
       declared at max+1: c00 104, c01 27, c02 21, c03* 72, c04 151, c05
       596, c06a 252, c06b 272, c09 8, c10 35, c11 208 (world 2's wider
       census is the worst case for c00/c01/c04-c06b/c10/c11; world 1 for
       c03*). **Golden pins: 8 new** (one per content world, before+after
       in one test; the arc's after is its tick-3 value) — measured, never
       derived, touching none of the sixteen prior pins. **Mutation
       vectors (eleven, all proven):** deleting c09 compounds the arc's
       cost modifier (tick 2 reads 0.765625) — the Task-8-deferred vector;
       dropping c09's active guard reds the C4 detector; each decay α
       independently reds the independence pin (heat's swap also reds the
       worlds pinning heat); each max(0,·) clamp REMOVED stays GREEN (the
       converse — the clamps are present-but-inert for non-negative
       intermediates, present because frozen's law carries them); a second
       INSTITUTION node in world 5c reds §8c guard 4 then the collision
       arithmetic (37 → 42); removing world 5b's carrier reds guard 4, the
       seam-ran pin, AND cascades E-EVAL-031 through c01 (the reset's
       load-bearingness one more way); activating a world-6 member reds
       the lanes-skipped pin. §8c guards 1 and 4 now iterate every world
       this pack loads into (the main suite covers worlds 1-4, the arc
       suite worlds 5-6).
   * - D206
     - §2.6
     - **The ephemeral XGI rebuild collapses into the substrate (D-NF+1).**
       Frozen rebuilds a throwaway ``xgi.Hypergraph`` every tick
       (``community.py:346``); in Rust the hypergraph IS the substrate
       (Amendment D), so communities are seeded once by the scenario and
       persist — a structural reformulation, not an optimization. Landed
       with Task 1's ``(hyperedge …)`` form (``scenario.rs``), which extends
       the id-order law with an independent counter (D216).
   * - D207
     - §2.5
     - **The ``community-register`` carrier is not a community node
       (D-NF+2).** The world's singleton INSTITUTION registry gives
       carrier-subject rules a subject; the ``community/`` namespace is
       banned from ``:field`` bindings — the D29 mechanism (D202) is the
       mechanical face of VIII.9. The singleton half is D223's.
   * - D208
     - §4.2
     - **The D116 same-tick cross-rule reliance, recorded as the Q14
       train's acceptance-criterion input for this pack (D-NF+4).** Five
       reads (the plan's §8b table), two of them FATAL if
       collect-across-rules ever lands (c05 reading pre-reset
       accumulators; c06a/c06b's cache hand-off). The pack's rules run in
       rule-id byte order (D16); this row exists so the Q14 repair train
       has the correct criterion in hand before it changes that.
   * - D209
     - §3.7
     - **The 500-org cap does not port (D-NF+5).** Frozen truncates
       ``org_data`` at 500 in INSERTION order (``community.py:401,424-426``)
       — non-deterministic under dict order and inexpressible here (no
       early-break construct). The port's bound is fuel: an over-large org
       census refuses loudly at load (E-LOAD-040/E-LOAD-045) instead of
       truncating silently. Re-open trigger: none — a world with >500 orgs
       fails at load with the reason named.
   * - D210
     - §2.7
     - **Shannon entropy lands on the ``log`` intrinsic (D-NF+7) —
       LANDED** at the DG-2 PUBLISH ruling (the 2026-08-18 sitting):
       ``c07-contestation`` divides by ``(log (* 1.0c 3))`` — computed
       through the pack's declared ``log`` intrinsic (the pinned libm
       soft-float crossing), never pasted; the ``1e-10`` per-component
       guard transcribes verbatim (``entities/consciousness.py:274-291``).
       The pack's ``(intrinsic log :params (real) :returns real :cost 40)``
       is the FIRST content declaration of ``log`` — per the spec's
       "``:cost`` provenance" paragraph, this pack set the number (the
       crate's own declaration test's value), pinned by the conformance
       vectors thereafter. **The 1-ulp divisor divergence, recorded:**
       libm's ``log(3)`` is ``1.0986122886681096``, one ulp BELOW CPython's
       glibc ``math.log(3)`` (``1.0986122886681098``) — the engine's pinned
       crossing is the law of this port (ADR176 r21/ADR188), the mirror
       divides by the engine's value, and the conformance pins are
       bit-exact against the PORT's arithmetic. On the landed vectors the
       per-component ``p*log(p)`` terms' ulp differences happen to cancel —
       a future world whose vector does not cancel re-opens with a written
       tolerance policy (the transcendental contract's own rule).
   * - D211
     - §2.7
     - **A second declared home for the hegemonic tie-break (D-NF+8) —
       LANDED at DG-2 = PUBLISH.** ``consciousness.bsl``'s "ONE DECLARED
       HOME" comment carries the one-line amendment (TWO DECLARED HOMES,
       parity-guarded, with its own frozen-anchor correction ``:177-192`` →
       ``:167-191``, epsilon ``:189``), and §8a row 1's copies-agree test
       (``dominant_tendency_ties_match_the_class_surface_rule``) pins both
       homes to LIBERAL on the same three-way tie — the class surface
       through ``consciousness/p8``, the community surface through ``c08``,
       in one co-loaded world (``community-tie-conformance.bscn``, the
       ninth content world, added at this landing).
   * - D212
     - §3.6
     - **Unread published state is not declared (D-NF+9).** Frozen
       publishes ``visibility``, ``rent_access_modifier`` and ``category``
       onto the hyperedge (``community.py:99,102,104``) and reads none of
       them in ``step()``; declaring them — or transcribing
       ``HyperedgeCategory`` for ``category``'s sake — would enter the tick
       hash write-only, against declare-what-you-read. Re-open trigger:
       the first reader of any of the three.
   * - D213
     - §5
     - **The #653-gated half, itemized (D-NF+10).** Threat scoring
       (``community.py:579-608``), solidarity amplification (:527-576) and
       the infrastructure maintenance term (:655-661) each need
       per-membership payload (AG(i), #653). Filed as issue #692 with the
       plan §5 table as its body and "cannot open until #653 lands" as its
       gate; the co-sponsored requirements note is
       ``reports/community-membership-shape-note-2026-08-18.md``.
   * - D214
     - §5
     - **The off-``step()`` estate is not ported (D-NF+11).** The four
       repression helpers (``community.py:210-279``),
       ``community_overlap_matrix``, ``communities_spanning_axis``, and
       ``calculate_solidarity_potential`` are never called from ``step()``;
       they await a verb layer, and ``solidarity_potential``'s two defines
       (``community_overlap_bonus``, ``rent_differential_penalty``,
       ``config/defines/organizations.py:42-51``) stay undeclared.
   * - D215
     - §3.7
     - **``CanonicalState`` section ``0x07`` (D-NF+14, landed one section
       LATER than the plan's ``0x06``).** The III.7 widening mirroring
       ADR198 R1/ADR203 landed as the SEVENTH listing
       (``all_hyperedge_attributes``) at section ``0x07`` because T3's
       Currency lane took ``0x06`` first — the plan's ``0x06`` is
       superseded by the landing, with the caller-side empty elision
       keeping every landed pin byte-identical (the elision proof attached
       at Task 5, #682). ``CANONICAL_LAYOUT_VERSION = 4``.
   * - D216
     - §3.6
     - **Hyperedge scenario seeding, and why no rule mints one
       (D-NF+15).** ``(hyperedge …)`` extends ``scenario.rs``'s id-order
       law with an independent counter; ``DEFERRED_SHAPE_VERBS`` stays
       intact because the port needs no minting verb — #536's rider 1 is
       unspent. Re-open trigger: the first design that mints a community
       mid-tick.
   * - D217
     - §2.6
     - **One ``HyperedgeType`` member + a ``community/kind`` enum field
       (D-NF+16).** Type-as-identity (14 members) would force 14
       near-identical copies of every uniform law, since ``(hyperedges …)``
       is type-scoped (``grammar.rs:202-212``). The kind dispatch is the
       14-arm chain (D204's c06a) — ONE copy of each law, the table read
       per community.
   * - D218
     - §5
     - **``community-cost-modifier`` is a dead write, ported anyway
       (D-NF+17).** Frozen declares and writes it with zero readers
       anywhere in ``src/``; ported because it is frozen's observable
       output. Re-open trigger: the reproduction-cost consumer (the
       vitality/reproduction lane) reading it — at which point the field
       becomes load-bearing and this row's note should be re-read first.
   * - D219
     - §2.11
     - **The pack emits no events and reads no ``TickContext`` (D-NF+18).**
       Frozen emits nothing (whole-file grep over ``community.py``) and
       names its context parameter ``_context``; the absence is declared so
       a later reader does not assume an emitter was dropped. The
       conformance suites assert value-level state, never an event stream,
       for this pack.
   * - D220
     - §3.6
     - **Memberships are UNATTRIBUTED in this train (D-NF+19).** No
       payload exists, so the worlds carry no role/strength/visibility
       columns — deliberately absent, never defaulted, since a defaulted
       role would fabricate the exact data #653 exists to carry. The
       seeded ``(hyperedge …)`` form carries a bare member list, whole
       (VIII.9), and Task 6's ``(hyperedge-attr …)`` writes the
       hyperedge's OWN fields only — never a per-member payload.
   * - D221
     - §3.4
     - **The duplication ledger (D-NF+20).** The §8a table's copied
       expressions (the tie-break, the decay triplication, the
       ``/ member-count`` divisor ×4, the degeneracy epsilons, and — added
       at landing — c05's inlined total/unorganized expressions);
       single-sourcing is unavailable in the language (no ``defexpr``, no
       macro, no cross-rule binding), so each pair owes a copies-agree row
       and a perturb-one-copy vector — the bit-exact mirror assertions are
       that mechanism for the c05 inlines.
   * - D222
     - §2.6
     - **The type-scoped hyperedge enumerator widens ``GraphSubstrate``,
       not ``CanonicalState`` (D-NF+21) — a declared crate-boundary
       crossing (§3.6).** ``(hyperedges …)`` had no accessor at all;
       routing through ``CanonicalState::all_hyperedges`` was rejected on
       that trait's own ruling (``state_hash.rs:294-299`` keeps its
       listings a serialization capability, off the structural-verb
       surface), so the substrate gained symmetric ``hyperedges`` /
       ``hyperedges_of`` / ``hyperedge_type_of`` with conformance rows on
       both backends (#681-#684).
   * - D223
     - §3.7
     - **The singleton INSTITUTION carrier is an estate-wide invariant;
       this pack mints no second one (D-NF+23).** ``subject_type_of``
       makes every INSTITUTION-subject rule iterate every INSTITUTION node
       and an unwritten bound field is a III.11 hard error, so a second
       carrier double-applies this pack's hyperedge writes AND breaks the
       landed carceral packs in any co-loaded world. World 5c executes the
       co-load; §8c row 4 guards every world the pack loads into (the two
       suites split the eight worlds between them). **Re-open trigger:** a
       design needing two carriers needs a subject SELECTOR the language
       does not have — an escalation, not a fixture change. **Related
       latent gap found at #688's harvest:** ``resolve_domain`` (load) and
       ``subject_type_of`` (tick) derive the subject independently and
       nothing cross-checks them — an unreferenced anchor under a vacuous
       guard is E-LOAD-004-undeterminable at load while the tick would
       derive it fine; this pack declares ``(domain NodeType/INSTITUTION)``
       explicitly on every vacuous-guard carrier rule, and the
       cross-check is a named follow-up.
   * - D224
     - §2.3
     - **The c06 split's companion note (D-NF+6's landing shape).** The
       floor table is a ``defconst`` table plus exactly one 14-arm
       dispatch — ADR214 R4's ruled entry path — and the dispatch lives in
       ``c06a`` (not the plan's single ``c06``): the pre-state law forces
       the split (D204), and c05's degenerate branch routes through it
       bit-identically (``×1.0``/``÷1.0`` are exact in IEEE-754 — the §6.2
       I5 argument, executed in world 3). The cross-world parity test
       (``the_floor_table_is_byte_identical_across_all_three_worlds``,
       extended to all eight at Task 10's guard extension) keeps the 14
       ruled values from drifting.
   * - D225
     - §2.7, §2.10
     - **The DG-2 readout landed (the 2026-08-18 sitting: PUBLISH).**
       ``c07-contestation`` (the normalized Shannon entropy, the
       per-component ``1e-10`` guard, the computed ``log(3)`` divisor) and
       ``c08-dominant-tendency`` (the argmax with the ruled tie order
       LIBERAL > REVOLUTIONARY > FASCIST at ``1e-6``) write
       ``community/contestation`` (probability) and
       ``community/dominant-tendency`` (enum ``ConsciousnessTendency``) on
       every non-skip-gated community. Frozen's skip gate holds for the
       readout too: a no-org community is written NOTHING (honest-null,
       proven in world 3). The class-surface copies-agree pair is §8a row
       1's test, landed. Fuel (per-world, max+1, over all nine worlds
       including the tie world's co-load): c07 900, c08 336. World-1 fired
       arithmetic: 32 = 30 + c07:1 + c08:1; the collision world's: 39 = 37
       + the two readout rules on the shared carrier. The seam, collision
       and empty worlds' golden pins did NOT move at this landing (their
       communities are all skip-gated — c07/c08 write nothing there), and
       the five moved pins (worlds 1-4 + the arc's tick-3 pair) were
       re-measured with the pack's growth named in the commit — this
       crate's own goldens, never a §6.5 ceremony.
   * - D226
     - §3.10, §4.6
     - **The** ``sqrt`` **rider landing (ADR219, Director ruling
       2026-08-22).** ADR188 Row 6's fallback rider is taken: ``sqrt``
       joins the declarable set with signature ``(real) → real``, domain
       ``[0, ∞)`` — a negative argument is the newly minted
       ``E-EVAL-044``, ``-0.0`` is accepted (``-0.0 < 0.0`` is ``false``,
       the D97 precedent), and IEEE's ``squareRoot(-0)`` is ``-0``, so the
       zero's sign is the standard's own. The crossing is ``f64::sqrt`` —
       correctly rounded by IEEE-754's mandate — so the pinned soft-float
       libm and §4.3's golden-vector clause are DECLINED, the D97
       disposition shape, recorded in §3.10's normative paragraph rather
       than silently omitted. No result-side re-check exists (finite
       in-domain input ⇒ finite exactly-specified output; a guard would be
       dead code) — recorded here for the same reason.
   * - D227
     - §2.7, §3.2, §3.10
     - **The** ``round-half-even`` **signature reading (ADR219, landing
       ADR188 Row 3, resolving D70).** §3.2 obliges the kernel to expose
       "the same algorithm" to rules but does not pin a signature. The
       landed reading: ``(round-half-even x) : real → real`` — the nearest
       integer VALUE to ``x`` within the binary64 lane, ties to the even
       neighbor, via ``f64::round_ties_even`` (IEEE's
       ``roundTiesToEven``). §3.2's exact-integer-arithmetic law is
       satisfied by construction: a binary64 argument is already an exact
       rational, so "exactly midway" is decidable exactly. The Real→Int
       demotion remains ``floor``'s alone (ADR188 Row 2); a
       granularity-general ``(round-half-even x g)`` form is declined as
       speculative. A workforce reading, open to Director correction, same
       as every Phase-1-review row.
   * - D228
     - §3.10
     - **The** ``min``/``max``/``abs``/``clamp`` **landing (ADR219,
       superseding ADR188 Rows 4/5 on #591 item 2's accumulated
       evidence).** The comparison rule is pinned and the library
       functions deliberately unused: Rust's ``f64::min``/``f64::max``
       may return either zero on a ±0.0 tie and propagate ``NaN`` — the
       implementation-defined shape this register exists to exclude. The
       landed rule: strict-comparison pick, **first argument wins on an
       equal comparison** (``+0.0``/``-0.0`` included), bit-pinned by the
       host's conformance tests. ``clamp`` composes the same two picks
       (``min(max(x, lo), hi)``); ``lo > hi`` is ``E-EVAL-044``, never a
       silent swap (§3.3's silent-clamping prohibition — this intrinsic
       is the legible form); an in-range ``x`` returns bit-identical.
       ``abs`` canonicalizes ``-0.0`` to ``+0.0``. Non-finite arguments
       are ``E-EVAL-044`` everywhere in the sextet — refused at the
       input, never propagated.

Authoring idioms (non-normative)
-----------------------------------

Five patterns recur across the landed rule packs under
``rust/crates/babylon-tick/content/rules/`` — Material Base and
Consequences-phase content alike. None mints new language surface —
each is a consequence already provable from §§1-4 of this document, restated
here because today the only way to discover it is grepping another pack's own
header comment. That cost is not hypothetical: a survey of the landed tree
(2026-08-18) found the first idiom below shipped both its wrong forms in one
PR before the right form landed. This section consolidates the shape, the
reason, and the landed citation for each; it settles nothing §§1-4 did not
already settle.

Int→Real promotion
~~~~~~~~~~~~~~~~~~~~

An ``if``'s two branches must share one static type (§2.7, ``E-TYPE-020``),
so a Real-typed branch paired with a bare ``0``/``1`` needs the ``Int``
literal promoted first, per §3.3's rule that ``Int`` promotes to ``Real``
in a binary64 expression. Mixing a genuine ``Coefficient`` operand into
the subtraction promotes the whole form to ``Real``:

.. code-block:: scheme

   (binding intensity-floor :expr
     (if (> raw-intensity 0) raw-intensity (- 0 0c)))
   (binding intensity :expr
     (if (< intensity-floor 1) intensity-floor (- 1 0c)))

86 matching lines across 9 of the 13 landed packs (measured, ``rg -c
'\b0c\b'`` over ``rust/crates/babylon-tick/content/rules/*.bsl``,
2026-08-18; four of those lines carry two ``0c`` tokens each, so the raw
token count is 90, via ``rg -o``) — every one a clamp or a zero/one
branch forced to agree with a Real sibling. The canonical write-up is
``lifecycle.bsl``'s own header
(``lifecycle.bsl:280-302``), cited forward from
``dispossession.bsl:355-357`` and reused verbatim by every later pack. It
records the two forms an
earlier revision of this exact clamp shipped and Copilot review caught wrong
on PR #493, worth keeping as the cautionary pair: ``(- 1 0)`` stays ``Int`` —
``Int - Int`` never promotes on its own, only an ``Int`` operand paired with
a genuine binary64 type inside one arithmetic form does — and Copilot's own
suggested fix, a bare ``1c``, stays typed ``Coefficient``, a third static
type matching neither branch. ``metabolism.bsl:386-387`` (explained at
``metabolism.bsl:68-73``) records the one operand-order subtlety the idiom
does not resolve by itself: when both operands are genuine ``:field`` reads
(already ``Real``, needing no promotion at all) rather than ``:const``-
sourced ``Int``\ s, multiplying first and dividing after is a DIFFERENT
floating-point program from the frozen engine's one-shot multiply —
correctly-rounded operations composed in a different order are not
guaranteed to agree, a real and measured divergence, not a promotion bug.

Sources: ``dispossession.bsl:355-357`` (the header comment),
``dispossession.bsl:361-364`` (the clamp code shown above),
``lifecycle.bsl:280-302``, ``metabolism.bsl:68-73,386-387``.

Eager ``:expr`` bindings
~~~~~~~~~~~~~~~~~~~~~~~~~~

§4.2 resolves every one of a rule's bindings, in declaration order, before
the ``when`` condition runs at all — stated directly, in the document's
own words: "``:expr`` BINDINGS resolve before any guard runs (§4.2's
binding-then-guard order)." A division inside a
binding therefore evaluates on every subject the rule visits, guard or no
guard: a bare ``(/ a b)`` aborts the whole tick with ``E-EVAL-012`` the
moment ``b`` reaches zero for any subject, not merely the one the guard was
meant to protect. The fix lives at the binding, never as a clamp on the
result afterward:

.. code-block:: scheme

   (binding actual-ratio :expr (if (= enforcer-population 0)
                                   (- 0 0c)
                                   (/ prisoner-population enforcer-population)))

``control-ratio.bsl``'s ``c03-crisis`` rule needed this lesson twice: a fix
round found the original claim — that the ``when``-guard's own shape already
protected the division — was wrong, and retracted it in
``decomposition_conformance.rs:172-176``: "every ``:expr`` binding … evaluates
eagerly and unconditionally, once per subject per tick, regardless of what
any later binding or guard does." Mutation evidence pins the point exactly:
dropping the ``if``-protector at the binding — not merely the effects'
``guard``-split alongside it — is what reproduces the frozen engine's
unrepresentable ``float("inf")`` as a named ``E-EVAL-012`` failure rather
than a silent no-emit.

Sources: ``control-ratio.bsl:126-156`` (the mutation-tested account),
``control-ratio.bsl:305-313`` (the binding itself),
``decomposition_conformance.rs:172-176``, §4.2 of this document (the
quoted clause itself is at ``bsl-language.rst:445-446``).

Empty-query protection
~~~~~~~~~~~~~~~~~~~~~~~~

A fold's ``mean``/``min``/``max`` and a ``select-max``/``select-min`` over an
empty query are all ``E-EVAL-021`` (§2.7, §4.4) — there is no element to
return and no null to stand in for one. A rule whose effect targets a
tiebreak-selected neighbor must never let that neighbor's query run empty,
and no protector for this exists inside effects position — only ``when`` can
keep a rule from firing at all, so the existence check has to happen there,
split across rules where the frozen source had one:

.. code-block:: scheme

   (binding bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                          (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                    territory/biocapacity)
                          (- 0 0c)))
   ...
   (when (and (= role SocialRole/LABOR_ARISTOCRACY)
              (exists (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS))
              (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))))

``production.bsl``'s ``p2``/``p3`` pair exists for exactly this reason: p2's
effect-position ``select-max`` over the WAGES neighbor set aborts on an
employer-less subject, so employer existence has to be split at the ``when``
level — p2 guards ``(exists …)``, p3 guards ``(not (exists …))`` — before
either rule's ``field-of (select-max …)`` bindings become legal. The
assumption underneath ``select-max``/``select-min`` here — one relevant
target, reached by tiebreak, as good as a singleton carrier for this
subject — holds when a subject has at least one candidate and cannot be
made when the query can be empty, which is why this idiom's every
``select-max``/``select-min`` carries its own paired ``exists`` guard rather
than a bare one.

Sources: ``production.bsl:27-39`` (the rule split, citing ``E-EVAL-021``
itself), ``production.bsl:193-222`` (``p2-employed-routing``, worked),
§2.7 and §4.4 of this document.

Fuel bounds by declare-low-and-read-back
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``:fuel`` is never a guess. ``bound(rule) > :fuel`` is a load-time refusal,
``E-LOAD-040`` (§3.7), and the refusal message states the checker's own
computed static bound — so declare-low-and-read-back is faster and more
honest than estimating: declare a deliberately low ``:fuel``, load the
content, read the exact bound the refusal reports, then declare ``:fuel``
one above it.

.. code-block:: scheme

   (rule solidarity/p0-transmit
     :role mechanic
     :evidence derived
     :material-basis "..."
     :fuel 4114
     ...

``solidarity.bsl``'s own header records the workflow run twice against the
same rule: Task 2 measured ``1126`` from ``E-LOAD-040: rule
solidarity/p0-transmit static bound 1126 exceeds its declared :fuel 1``;
adding two event emits in Task 3 pushed the true bound to ``3502``,
re-measured the identical way — declaring the now-too-low ``1126`` and
reading the checker's refusal back verbatim, never guessed or rounded up.
ADR211 records the resulting jump, 1126 → 3502 (+211% for two emits), as an
architectural trend rather than a defect: every per-target quantity is
repeated inline because no per-iteration binding form exists, so a rule's
fuel cost compounds with its emit count, not merely its binding count — worth
revisiting only if a future rule needs three or more emits per edge. A
report mode that prints the computed bound directly, without the
declare-low round-trip, is landing separately.

Later language work brought the live bound to ``4079``. Amendment AJ then
required the three direct
Probability observations in those emits to become explicitly derived Real
values. The same refusal measured the current ``4114`` bound. The historical
ADR211 measurement remains 1126 → 3502, while ``4114`` is the executable
declaration now shown above.

Sources: ``solidarity.bsl:153-172``, ``solidarity.bsl:193-200``,
``decomposition_conformance.rs:148,222`` and
``control_ratio_conformance.rs:134,197,307`` (the same declare-low-and-
read-back technique used elsewhere in the estate), ADR211, §3.7 of this
document.

Same-tick freshness
~~~~~~~~~~~~~~~~~~~~~

Rules at one anchor position evaluate — and apply their effects — in
ascending rule-id byte order (§4.2); a rule later in that order observes an
earlier rule's already-applied writes from THIS tick, not the tick's shared
pre-state (D116). A ``:field`` binding declared ``:optional :default`` on a
field another rule writes at the same anchor is therefore fresh or stale
purely by the two rules' ids, never by anything either rule's own text says:

.. code-block:: scheme

   ; consciousness/p6-route (byte-orders before p8, same anchor)
   (binding l2 :expr (if (> total (+ 1 eps)) (/ l1 total)
                       (if (< total (- 1 eps)) (+ l1 (- 1 total)) l1)))
   ...
   (update-node self social-class/liberal (set l-out))

   ; consciousness/p8-dominant-worldview reads the healed value THIS tick
   (binding l :field social-class/liberal :optional :default 0.0p)

The companion pattern is the tick-reset: a rule with an unconditional
``(when #t)`` guard that zeroes a carrier field before later same-anchor
rules accumulate into it, the idiom ``production/p0-production-total-reset``
and ``territory/p1-heat-dynamics`` both use. §2.8 admits no effect
finer-grained than a rule's own ``when``, so a reset-then-accumulate carrier
and a same-tick-healed read are one mechanism — anchor-and-byte-order
sequencing — read from two directions; D127 names the corollary this forces
on every port of a sparse, dict-collected frozen loop: the write fires
unconditionally even on the branch where nothing changes, because BSL has no
"skip this effect" construct finer than ``when``. Load refusals that would
catch a same-tick ordering mistake at content-load time, rather than leaving
it to be found by re-reading two rules' ids, are landing separately in the
language.

Sources: ``consciousness.bsl:323-336`` (the heal computation through its
``update-node`` write at :336), ``consciousness.bsl:353-370`` (the
same-tick read), ``production.bsl:154-161`` (the reset pattern), D116,
D127, D154 (this document's Draft-Ruling Register), §4.2 of this document.

See Also
----------

- :doc:`/reference/determinism-contract` — the constitutional hash catalog,
  the three float-tolerance regimes, and (from Program 27 Phase 0) the
  tick-hash field set, ``ContentDigest`` composition, and ``rules_hash``'s
  place in it. This document defines the bytes ``rules_hash`` covers; that one
  defines what it is compared against.
- :doc:`/reference/precision` — the quantization Gatekeeper Pattern
  (``SnapToGrid``, ``1e-5``), the drift-prevention mechanism BSL's fixed-point
  Currency lane partially supersedes.
- :doc:`/reference/configuration` — ``GameDefines`` and ``defines.yaml``, the
  coefficient environment that ``:const`` bindings read.
- :doc:`/reference/topology` — ``NodeType``/``EdgeType``, the closed graph
  vocabulary ``<enum-ref>`` draws on.
- :doc:`/reference/error-codes` — the engine's existing error-code estate; the
  ``E-LEX``/``E-PARSE``/``E-TYPE``/``E-LOAD``/``E-EVAL`` families defined here
  are registered there when Phase 1 lands code.
- ``docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`` —
  the Director-approved design; §5 (the language), §6 (the kernel), §8
  (correctness), §9 (error handling and determinism), §10 (sequencing).
- ``CONSTITUTION.md`` III.11 (Loud Failure), III.12 (Behavioral Contracts,
  Amendment Q), III.8 (Aleksandrov Test), **Amendment AE clause (vi)**
  (Amendment D — native hyperedge, ratified v3.0.0), II.9 (the strictly dyadic
  morphism layer that coexists with it), VIII.9 (the clique-expansion
  anti-pattern the ruling discharges structurally), and **Amendment AG**
  (attributed membership and lattice instances, ratified v3.2.0, recorded in
  ``ai/decisions/ADR189_amendment_ag_attributed_membership_lattice_instances.yaml``)
  — the amendment this document's fourth revision specifies.
- ``ai/_inbox/amendment-d-analysis-p27.md`` — the Phase-0 Amendment D analysis
  (PR #353); §9 records the Director's ruling and sub-rulings D-1…D-7 that
  §2.6, §2.8 and §3.7 of this document implement.
