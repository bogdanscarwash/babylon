"""Sync guard for the consolidated BSL grammar (``docs/reference/bsl.ebnf``, §7/D92).

``docs/reference/bsl-language.rst`` is the ONE normative home for BSL. §7
collects its scattered productions into ``bsl.ebnf`` and includes that file
back into the document, so the appendix is normative **by inclusion** — which
buys the rigor of a single consolidated grammar only for as long as the
collection actually tracks the sections it collects. Left unguarded, the
failure mode is silent and certain: a chapter adds a form, the section text
carries it, the appendix does not, and two implementations derived from "the
document" disagree about what the language is. That is a III.12(a) failure,
not a formatting drift.

The guard is therefore three containments, each in the safe direction:

1. every ``<name> ::=`` production the rst's §§1/2/6.1 code blocks define
   appears in ``bsl.ebnf``;
2. every form tag §5.2 enumerates appears in ``bsl.ebnf`` as a quoted
   terminal — §5.2 is the CAS-side list of head symbols, so a form with a
   tag and no production is exactly the drift above, caught from the other
   side;
3. every form head the **Rust reference implementation** knows
   (``RESERVED_FORM_TAGS`` in ``babylon-bsl``'s ``declarations.rs``, which is
   that crate's rendering of the same §5.2 list) appears in ``bsl.ebnf``.

Direction matters. The guard never demands the reverse containment — that
every production or tag in the appendix be known to Rust — because the rst
leads the implementation by design: the Amendment AG forms (``member``,
``membership-field-of``, ``update-membership``, ``rung``, ``adjunction``)
are specified and not yet implemented, and a guard that failed on that would
be demanding the spec wait for the code.

No test here parses BSL or checks a program: this is a documents-agree test.
The language's own conformance lives in the crate's vectors (§6.2).
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
EBNF = REPO_ROOT / "docs" / "reference" / "bsl.ebnf"
RST = REPO_ROOT / "docs" / "reference" / "bsl-language.rst"
DECLARATIONS_RS = REPO_ROOT / "rust" / "crates" / "babylon-bsl" / "src" / "declarations.rs"

#: Productions of the rst that are deliberately NOT in the appendix. §5.2's
#: ``atom`` / ``form`` / ``opt`` are the canonical-serialization BYTE layout —
#: a schema over octets (``0x01``, ``u8 len(kind)``, …), not a production over
#: the token stream — and collecting them into a surface grammar would be a
#: category error. They are named here rather than filtered by a pattern so
#: that a NEW §5 production cannot slip through the same hole unnoticed.
CAS_ENCODING_PRODUCTIONS = frozenset({"atom", "form", "opt"})

#: The synthetic CAS tag for a keyword option (§5.2). It is a tag with no
#: surface syntax — no BSL source ever spells ``(opt …)`` — so §5.2's list
#: carries it and the surface grammar cannot.
SYNTHETIC_FORM_TAGS = frozenset({"opt"})

#: The NINE productions the appendix collects from the rst's PROSE rather than
#: from one of its code blocks. §7 and D92 list exactly these, so the set is a
#: contract between the document and this guard, not a convenience: a tenth
#: entry here without a matching edit there is drift in the thing §7 discloses.
#:
#: An earlier spelling of this set also carried ``literal``, ``payload-item``
#: and ``string-char`` — all three of which ARE backed by rst code blocks
#: (§2.7, §2.8, §1.4). The annotations were simply wrong, and wrong in the
#: direction that weakens the guard: an exemption for a production that needs
#: none silently excuses it if the rst ever drops it. Removed (adversarial
#: verification, PR #485).
PROSE_SOURCED = frozenset(
    {
        "Char",  # §1.1, "a sequence of Unicode scalar values"
        "whitespace",  # §1.2, "exactly U+0020, U+0009, U+000A and U+000D"
        "comment",  # §1.2, "begins with ; outside a string literal"
        "delimiter",  # §1.4, "a token ends at whitespace, (, ), ; or EOF"
        "operator",  # §1.4's draft ruling — the tenth atom class
        "escape",  # §1.5's four escapes, spelled as source sequences
        "intrinsic-name",  # §2.7, "a symbol declared in the intrinsic table"
        "type-name",  # §2.9/§2.11 use it; §3.1 names the types (the gap)
        "vector-file",  # §6.1, "a vector file is a sequence of vector forms"
    }
)


def _read(path: Path) -> str:
    assert path.exists(), f"{path} is missing"
    return path.read_text(encoding="utf-8")


def _ebnf_text() -> str:
    return _read(EBNF)


def _ebnf_code() -> str:
    """The appendix with its comments stripped: the grammar and nothing else.

    Comment prose carries apostrophes and quoted phrases, and a naive scan for
    quoted terminals over the raw file pairs quotes ACROSS them — which found
    zero terminals while looking like it worked. Strip first, then scan.

    The strip is non-greedy and therefore implements the dialect's own rule
    exactly: the FIRST comment terminator closes the comment. That makes it
    correct and fragile in the same stroke — a comment containing the literal
    terminator ends early and the prose after it is read as grammar, which is
    what the appendix's own dialect table used to do (it showed the comment
    syntax by example) and why that table now spells the delimiters in words.
    ``TestTheCommentStripIsSound`` holds the invariant that makes this safe.
    """
    return re.sub(r"/\*.*?\*/", " ", _ebnf_text(), flags=re.DOTALL)


def _referenced_nonterminals() -> set[str]:
    """Every bare name appearing on a right-hand side in the appendix.

    Quoted terminals, character classes, ``#xNN`` code points and the
    production's own left-hand side are excluded; what remains is what a
    reader must be able to look up.
    """
    referenced: set[str] = set()
    for body in re.split(r"^(?=[A-Za-z][A-Za-z0-9-]*\s*::=)", _ebnf_code(), flags=re.MULTILINE):
        head = re.match(r"^([A-Za-z][A-Za-z0-9-]*)\s*::=(.*)$", body, re.DOTALL)
        if not head:
            continue
        rhs = head.group(2)
        # SINGLE quotes first, and the order is load-bearing: the appendix
        # uses single quotes exactly where a terminal CONTAINS a double quote
        # (`'"'` in `string` and `string-char`). Stripping double-quoted runs
        # first mis-pairs on that quote and then swallows the delimiters of
        # every terminal after it — which made `escape`'s "n" and "t" read as
        # undefined nonterminals when this row was first written.
        rhs = re.sub(r"'[^'\n]*'", " ", rhs)  # single-quoted terminals
        rhs = re.sub(r'"[^"\n]*"', " ", rhs)  # double-quoted terminals
        rhs = re.sub(r"\[[^\]\n]*\]", " ", rhs)  # character classes
        rhs = re.sub(r"#x[0-9A-Fa-f]+", " ", rhs)  # code points
        referenced |= set(re.findall(r"\b([A-Za-z][A-Za-z0-9-]*)\b", rhs))
    return referenced


def _ebnf_productions() -> set[str]:
    """Left-hand sides defined in the appendix."""
    return set(re.findall(r"^([A-Za-z][A-Za-z0-9-]*)\s*::=", _ebnf_code(), re.MULTILINE))


def _ebnf_production_rhs(name: str) -> str:
    """One production's right-hand side, comment-stripped.

    The generic containment tests above check only that a left-hand-side
    NAME is defined on both sides of the document/appendix split — they
    would stay green if a shared name's alternatives silently diverged
    (D98: the ``intrinsic-type-name`` production is exactly the row that
    class of drift would hit first, since it shares its ``real`` alternative
    with nothing else in the file). This is the finer-grained lookup a test
    over one production's actual content needs.
    """
    for chunk in re.split(r"^(?=[A-Za-z][A-Za-z0-9-]*\s*::=)", _ebnf_code(), flags=re.MULTILINE):
        head = re.match(rf"^{re.escape(name)}\s*::=(.*)$", chunk, re.DOTALL)
        if head:
            return head.group(1)
    raise AssertionError(f"bsl.ebnf has no {name} production")


def _rst_productions() -> set[str]:
    """Left-hand sides the rst's own code blocks define.

    The rst writes nonterminals two ways — bare in §1.4's lexical block
    (``symbol      ::= …``) and angle-bracketed everywhere else
    (``<rule>     ::= …``) — and at least one row omits the space before
    ``::=`` (``<payload-item>::=``). All three spellings are matched.

    **Case matters, and an earlier spelling of this scan got it wrong**
    (Copilot review, PR #485): §1.4's character terminals are UPPERCASE
    (``DIGIT``, ``LOWER``, ``UPPER``), so a lowercase-only pattern silently
    excused exactly the three productions the whole lexical layer rests on —
    they fell through to the prose-only exemption below and were guarded by
    nothing.
    """
    pattern = re.compile(r"^\s+<?([A-Za-z][A-Za-z0-9-]*)>?\s*::=", re.MULTILINE)
    return set(pattern.findall(_read(RST)))


def _rst_form_tags() -> set[str]:
    """The form tags §5.2 enumerates, from its own paragraph.

    §5.2 writes them as a parenthesised list of double-backquoted symbols
    followed by "plus the synthetic tag ``opt``"; the operator tags (``<``,
    ``+``, …) are in the same list and are dropped here because they are not
    spellable as symbols.
    """
    body = _read(RST)
    start = body.index("**Form tags** are the form's head symbol verbatim")
    end = body.index("A keyword option is encoded as a two-child form", start)
    section = body[start:end]
    return {
        tag
        for tag in re.findall(r"``([a-z][a-z0-9-]*)``", section)
        if tag not in SYNTHETIC_FORM_TAGS
    }


def _rust_reserved_form_tags() -> set[str]:
    """``RESERVED_FORM_TAGS`` from the Rust crate — its §5.2 rendering."""
    body = _read(DECLARATIONS_RS)
    match = re.search(r"pub const RESERVED_FORM_TAGS: \[&str; \d+\] = \[(.*?)\];", body, re.DOTALL)
    assert match, "RESERVED_FORM_TAGS not found in declarations.rs"
    return set(re.findall(r'"([a-z][a-z0-9-]*)"', match.group(1))) - SYNTHETIC_FORM_TAGS


def _quoted_terminals() -> set[str]:
    """Every quoted terminal in the appendix, both quote styles."""
    code = _ebnf_code()
    return set(re.findall(r'"([^"\n]+)"', code)) | set(re.findall(r"'([^'\n]+)'", code))


class TestTheAppendixCollectsTheSections:
    """§7's claim — "every production of §§1, 2 and 6.1" — held to."""

    def test_every_rst_production_appears_in_the_appendix(self) -> None:
        missing = _rst_productions() - _ebnf_productions() - CAS_ENCODING_PRODUCTIONS
        assert not missing, (
            "productions defined in bsl-language.rst but absent from bsl.ebnf: "
            f"{sorted(missing)}. §7 says the appendix collects EVERY production of "
            "§§1, 2 and 6.1 — add them there (the section text wins, D92)."
        )

    def test_the_cas_encoding_productions_are_still_the_only_exclusion(self) -> None:
        """The exclusion list is an allowlist, so it must stay exact.

        If §5 ever loses one of these, the exclusion silently widens; this row
        fails first and names the drift.
        """
        assert _rst_productions() >= CAS_ENCODING_PRODUCTIONS

    def test_the_appendix_defines_nothing_the_rst_does_not(self) -> None:
        """The other containment, where it IS safe to demand.

        A production in the appendix with no counterpart in the sections would
        be the appendix legislating — exactly what D92 forbids. The lexical
        helpers §1 states in prose rather than in a code block are exempt and
        listed by name.
        """
        extra = _ebnf_productions() - _rst_productions() - PROSE_SOURCED
        assert not extra, (
            f"bsl.ebnf defines productions the rst does not: {sorted(extra)}. "
            "The appendix collects; it does not amend (D92)."
        )


class TestTheAppendixIsSelfContained:
    """The grammar closes over itself: nothing referenced is undefined.

    This is the property a reader deriving an implementation actually needs —
    §7's whole claim is that the appendix can be read as a grammar, and a
    dangling name breaks that no matter how faithfully every other production
    was transcribed. It is also the row that catches a production whose
    right-hand side went missing: an RHS that is only a comment vanishes when
    comments are stripped, and its name then defines nothing while everything
    referencing it dangles (adversarial verification, PR #485, found exactly
    that on ``Char``).
    """

    def test_every_referenced_nonterminal_is_defined(self) -> None:
        undefined = _referenced_nonterminals() - _ebnf_productions()
        assert not undefined, (
            f"bsl.ebnf references nonterminals it never defines: {sorted(undefined)}. "
            "A grammar that does not close over itself cannot be derived from, "
            "which is §7's entire claim (D92)."
        )

    def test_every_production_has_a_non_empty_right_hand_side(self) -> None:
        """The same defect from the other side, named for the reader.

        ``name ::=`` followed by nothing is not a production in the declared
        W3C dialect; a literal reading collapses whatever referenced it.
        """
        empty = [
            name
            for name, rhs in re.findall(
                r"^([A-Za-z][A-Za-z0-9-]*)\s*::=([^\n]*(?:\n[ \t]+[^\n]*)*)",
                _ebnf_code(),
                re.MULTILINE,
            )
            if not rhs.strip()
        ]
        assert not empty, f"productions with an empty right-hand side: {empty}"


class TestTheCommentStripIsSound:
    """The comment strip implements the dialect, and the file respects it.

    ``_ebnf_code`` strips to the FIRST comment terminator, which is the
    dialect's own rule. The consequence is that a comment containing the
    literal terminator ends early and its remaining prose is read as grammar —
    which the appendix's dialect table used to do, by showing the comment
    syntax with a live example, leaking 26 lines of header prose into "code"
    (harmless then, latent forever). The table now spells the delimiters in
    words, and this row keeps it that way.
    """

    def test_stripping_leaves_no_comment_markers(self) -> None:
        code = _ebnf_code()
        strays = [line.strip() for line in code.splitlines() if "/*" in line or "*/" in line]
        assert not strays, (
            "comment markers survived the strip, so a comment ended early and "
            f"prose is being read as grammar: {strays[:3]}"
        )


class TestEveryFormTagHasAProduction:
    """A tag with no production is a form the appendix forgot."""

    def test_every_rst_form_tag_is_a_terminal_in_the_appendix(self) -> None:
        missing = _rst_form_tags() - _quoted_terminals()
        assert not missing, (
            "form tags §5.2 enumerates but bsl.ebnf never quotes: "
            f"{sorted(missing)}. Every form the canonical serialization can "
            "encode must have a production to be derived from."
        )

    def test_every_rust_form_head_is_a_terminal_in_the_appendix(self) -> None:
        missing = _rust_reserved_form_tags() - _quoted_terminals()
        assert not missing, (
            "form heads the Rust reference implementation knows "
            f"(RESERVED_FORM_TAGS) but bsl.ebnf never quotes: {sorted(missing)}."
        )

    def test_the_rust_tag_list_stays_a_subset_of_the_rst_list(self) -> None:
        """Rust may lag the spec; it may never lead it.

        A tag the crate knows and §5.2 does not is an implementation that
        invented a form — the direction the Constitution forbids.
        """
        invented = _rust_reserved_form_tags() - _rst_form_tags()
        assert not invented, f"babylon-bsl knows form heads §5.2 does not list: {sorted(invented)}"


class TestTheInclusionIsWired:
    """§7 is only normative if the document actually includes the file."""

    def test_the_rst_literalincludes_the_appendix(self) -> None:
        body = _read(RST)
        assert ".. literalinclude:: bsl.ebnf" in body

    def test_section_seven_exists_and_states_the_precedence_rule(self) -> None:
        body = _read(RST)
        assert "7. Consolidated grammar" in body
        assert "collects; it does not amend" in body

    def test_the_register_carries_d92(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D92$", body, re.MULTILINE), (
            "the consolidated grammar is a decision and owes a register row"
        )

    def test_the_appendix_names_its_dialect(self) -> None:
        assert "W3C EBNF" in _ebnf_text()

    def test_section_seven_discloses_every_prose_sourced_production(self) -> None:
        """§7 must name the productions that have no section production.

        The precedence rule is "the section TEXT wins", and it is only
        checkable by a reader who knows WHICH productions have no code block
        behind them — those are the ones where the appendix necessarily chose.
        §7 lists them, D92 repeats the count, and this row keeps the list from
        drifting away from the set the guard exempts (adversarial
        verification, PR #485).
        """
        body = _read(RST)
        start = body.index("7. Consolidated grammar")
        section = body[start : body.index("7.1 The rigor index", start)]
        undisclosed = {name for name in PROSE_SOURCED if f"``{name}``" not in section}
        assert not undisclosed, (
            f"§7 does not disclose these prose-sourced productions: {sorted(undisclosed)}"
        )
        assert f"**{len(PROSE_SOURCED)}**" in section or "nine" in section.lower()


class TestTheDerivedGrammarsCorpusCoversEveryForm:
    """Every §5.2 form tag is exercised by a tree-sitter corpus case.

    This row exists because the failure it catches actually happened. A corpus
    case whose source is not terminated by a ``---`` separator is folded into
    the PREVIOUS case's expected tree by ``tree-sitter test --update``, which
    then rewrites the file without it: five cases vanished silently, and
    ``tree-sitter test`` went on reporting 100%% because the cases it no longer
    knew about could not fail. A green suite that quietly shrank is exactly the
    shape III.11 rejects, and coverage measured against the SPEC's tag list —
    rather than against whatever the corpus currently happens to contain — is
    what makes the shrinkage visible.

    The corpus is derived tooling and normative for nothing; what is asserted
    here is only that it exercises what the language has.
    """

    @staticmethod
    def _corpus_sources() -> str:
        """The source halves of every corpus case, expectations excluded."""
        corpus_dir = REPO_ROOT / "tools" / "tree-sitter-bsl" / "test" / "corpus"
        sources: list[str] = []
        for path in sorted(corpus_dir.glob("*.txt")):
            for block in _read(path).split("=" * 50):
                if "\n---\n" in block:
                    sources.append(block.split("\n---\n")[0])
        return "\n".join(sources)

    def test_the_corpus_has_cases_at_all(self) -> None:
        assert self._corpus_sources().strip(), "no corpus case survived parsing"

    def test_every_form_tag_appears_in_a_corpus_source(self) -> None:
        used = set(re.findall(r"\(([a-z][a-z0-9-]*)[\s)]", self._corpus_sources()))
        missing = _rst_form_tags() - used
        assert not missing, (
            f"form tags no tree-sitter corpus case exercises: {sorted(missing)}. "
            "A dropped case is invisible to `tree-sitter test`, which cannot fail "
            "a case it no longer has — check for a source with no `---` separator."
        )


class TestTheRuledPointsStayCited:
    """The appendix's two under-determined points are RULED, and say so.

    Until 2026-08-11 this class asserted the two gap flags (``RECORDED GAP``
    for ``<type-name>``, ``TRANSCRIBED VERBATIM`` for §6.1's vector keywords)
    stayed in the file, because an edit that quietly deleted a flag would
    convert an honest gap into a silent ruling — the one thing an appendix
    must never do. The Director then ruled both points (ADR191 R4 and R5,
    recorded as D94 and D95), so the flags are gone by ceremony rather than by
    stealth and the guard follows the record: what must now stay in place is
    the CITATION of the ruling that settled each point. Deleting *that* would
    leave the appendix asserting a reading with nothing behind it, which is
    the same failure from the other side.
    """

    @pytest.mark.parametrize(
        "needle",
        [
            "RULED, D94",  # <type-name> is a lowercase symbol (ADR191 R4)
            "RULED, D95",  # §6.1's vector keywords are optional GROUPS (R5)
        ],
    )
    def test_the_ruling_is_still_cited(self, needle: str) -> None:
        assert needle in _ebnf_text()


class TestTheIntrinsicTypeNameVocabulary:
    """D98's own drift class, guarded at the RIGHT-HAND-SIDE grain.

    ``intrinsic-type-name`` is a normal production by every generic test
    above — its left-hand-side name is defined in both the rst and the
    appendix, so ``TestTheAppendixCollectsTheSections`` is satisfied the
    moment the name exists on both sides. None of those tests reads what
    the production actually SAYS. A silent regression that dropped the
    ``real`` alternative from one side (or left ``intrinsic-decl`` pointed
    at plain ``type-name`` instead of the widened production) would leave
    every generic row green while ADR188 Row 2's own ``floor`` rider became
    undeclarable again in whichever file lost it — exactly the "second
    implementor rejects the document's own example" failure the adversarial
    review found. These rows read the production bodies, not just their
    names.
    """

    def test_the_ebnf_production_admits_real(self) -> None:
        rhs = _ebnf_production_rhs("intrinsic-type-name")
        assert '"real"' in rhs, (
            "bsl.ebnf's intrinsic-type-name production dropped the `real` "
            "alternative (D98) — a deffield/metric :type still uses bare "
            "type-name; only the intrinsic-decl :params/:returns position "
            "widens"
        )

    def test_the_rst_section_defines_the_same_production(self) -> None:
        body = _read(RST)
        assert re.search(r"<intrinsic-type-name>\s*::=\s*<type-name>\s*\|\s*\"real\"", body), (
            'bsl-language.rst §2.7 must define <intrinsic-type-name> ::= <type-name> | "real"'
        )

    def test_intrinsic_decl_references_the_widened_production_not_bare_type_name(
        self,
    ) -> None:
        """The widening exists for nothing if `:params`/`:returns` still
        point at the un-widened `type-name` — this is the row that would
        have caught round 1 of this review shipping D98's TEXT with the
        grammar still saying `type-name`."""
        rhs = _ebnf_production_rhs("intrinsic-decl")
        assert "intrinsic-type-name" in rhs, (
            "bsl.ebnf's intrinsic-decl production does not reference "
            "intrinsic-type-name — :params/:returns must widen, not the "
            "bare type-name that deffield/metric still use"
        )
        # The bare `type-name` terminal must not appear standalone in this
        # production's :params/:returns operand positions — every mention
        # here must be the `intrinsic-` prefixed form.
        bare_type_name_mentions = re.findall(r"(?<!intrinsic-)\btype-name\b", rhs)
        assert not bare_type_name_mentions, (
            f"intrinsic-decl still references bare type-name somewhere: {bare_type_name_mentions}"
        )

    def test_d98_is_recorded_in_the_register(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D98$", body, re.MULTILINE), (
            "the intrinsic-type-name widening is a decision and owes a Draft-Ruling Register row"
        )


class TestTheRatioLiteralStaysInSync:
    """D99's own drift class (#492/ADR194, the declared-domain Currency scale
    operation) — guarded at the RIGHT-HAND-SIDE grain, the same class of gap
    ``TestTheIntrinsicTypeNameVocabulary`` guards for D98.

    ``suffix`` is a normal production by every generic test above — its
    left-hand-side name is defined in both the rst and the appendix, so
    ``TestTheAppendixCollectsTheSections`` is satisfied whether or not its
    alternatives actually agree. A silent regression that dropped the ``r``
    alternative from ONE side (or added it to the appendix without §1.5's
    literal-table row backing it) would leave every generic row green while
    the declared-domain scale operation quietly lost its only way to be
    WRITTEN. These rows read the production body and the section prose, not
    just the production's name.
    """

    def test_the_ebnf_suffix_production_admits_r(self) -> None:
        rhs = _ebnf_production_rhs("suffix")
        assert '"r"' in rhs, (
            "bsl.ebnf's suffix production dropped the `r` alternative (D99, "
            "#492/ADR194) — the declared-domain Ratio literal has no other "
            "way to be written"
        )

    def test_the_rst_documents_the_r_literal_row(self) -> None:
        body = _read(RST)
        assert re.search(r"\*\s*-\s*``r``\s*\n\s*-\s*``Ratio``", body), (
            "bsl-language.rst §1.5's literal table must carry the `r` / "
            "Ratio row (D99, #492/ADR194)"
        )

    def test_e_lex_027_is_the_non_positive_ratio_code(self) -> None:
        body = _read(RST)
        assert "E-LEX-027" in body, (
            "a non-positive Ratio literal needs its own lex-time code "
            "(D99, #492/ADR194) — the reference names E-LEX-027"
        )

    def test_currency_times_ratio_is_documented_in_section_3_2(self) -> None:
        body = _read(RST)
        start = body.index("3.2 Currency operator")
        end = body.index("3.3 The two numeric lanes", start)
        section = body[start:end]
        assert "Currency × Ratio" in section, (
            "§3.2 must state the Currency × Ratio operation (D99, "
            "#492/ADR194) — the whole point of the addendum"
        )
        assert "E-EVAL-041" in section, (
            "§3.2 must name the eval-time declared-domain check's code (E-EVAL-041, D99)"
        )

    def test_d99_is_recorded_in_the_register(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D99$", body, re.MULTILINE), (
            "the declared-domain Currency scale operation is a decision and "
            "owes a Draft-Ruling Register row (#492/ADR194)"
        )


class TestTheEnumCasPayloadShapeStaysInSync:
    """D117's own drift class (#528 fix round, blocker item 3) — the §5.2
    payload table's own self-contradiction, adversarial-verifier-found.
    The Q12 paragraph claimed ``defenum``/``defvocabulary`` need "no new
    atom kind" because "the ``<enum-ref>`` values they govern already
    encode with the existing atom kind" — true of an ``:enum-type``
    field's stored VALUES, false of ``defenum``/``defvocabulary``'s own
    operands (the type-name operand, the member-list items), which are
    bare ``<enum-type>``/``<enum-member>`` atoms, never ``<enum-ref>``
    pairs. RHS-grain checks, the same class ``TestTheRatioLiteralStaysInSync``
    and ``TestTheEnumRowStaysInSync`` guard, so the correction cannot
    silently regress to the old, self-contradictory single-shape reading.
    """

    def test_the_enum_payload_row_admits_both_shapes(self) -> None:
        # Scoped to §5.2's own list-table (the same scoping
        # TestTheEnumRowStaysInSync uses to avoid the unrelated §3.1 row).
        body = _read(RST)
        start = body.index("**Atom kinds and payloads:**")
        end = body.index("**Form tags**", start)
        section = body[start:end]
        assert re.search(r"^\s+\* - ``enum``", section, re.MULTILINE), (
            "the CAS atom-kind/payload table must still carry the enum row"
        )
        assert "<enum-ref>" in section, (
            "the enum row must still document the <enum-ref> (Type/MEMBER) payload shape"
        )
        assert "bare" in section.lower(), (
            "the enum row must ALSO document the bare defenum/defvocabulary "
            "operand payload shape (D117) — a single-shape table entry is "
            "the very premise the fix round found false"
        )
        assert "discrimin" in section.lower(), (
            "the enum row must name the discriminator (exactly one `/`) "
            "that keeps the two payload shapes collision-free (D117)"
        )

    def test_the_q12_paragraph_no_longer_states_the_false_premise(self) -> None:
        body = _read(RST)
        start = body.index("The Organization contract's Q12 tags obey it a fourth time")
        end = body.index("A keyword option is encoded as a two-child form", start)
        section = body[start:end]
        assert "defenum" in section and "defvocabulary" in section
        # The corrected text must acknowledge the bare-operand payload
        # shape explicitly, not just repeat the old "<enum-ref> values
        # they govern" sentence unmodified.
        assert "bare" in section.lower(), (
            "the Q12 CAS paragraph must be corrected to acknowledge "
            "defenum/defvocabulary's own BARE operand payload shape — the "
            "premise that only <enum-ref> VALUES are involved is false "
            "for these two forms' own operands (D117)"
        )
        assert "D117" in section, (
            "the corrected Q12 CAS paragraph must cite D117, the register "
            "row recording this resolution"
        )

    def test_d117_is_recorded_in_the_register(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D117$", body, re.MULTILINE), (
            "the enum CAS payload-shape correction is a decision and owes "
            "a Draft-Ruling Register row (#528 fix round)"
        )
        start = body.index("\nDraft-Ruling Register\n")
        end = body.index("\nSee Also\n", start)
        section = body[start:end]
        d117_start = section.index("D117")
        d117_row = section[d117_start : d117_start + 2000]
        assert "defenum" in d117_row or "defvocabulary" in d117_row, (
            "D117's own row text must name defenum/defvocabulary"
        )


class TestTheEnumArithmeticRefusalIsDeclaredInTheRegistry:
    """The write-boundary law's own drift class (#528 fix round, MAJOR
    item 2). ``structural_verbs.rs::refuse_arithmetic_on_enum_field``
    (``c268b83b``) has reported ``E-EVAL-042`` for an ``add``/``sub``/
    ``scale`` update-op targeting an ``:enum-type``-declared field since
    the enum write law landed, but §2.13's own E-EVAL-042 bullet and the
    §4.6 class table row both scoped the code strictly to the
    write-SHAPE law (a non-``<enum-ref>`` value reaching the write path)
    — never naming the arithmetic refusal at all. RHS-grain checks, the
    same class ``TestTheEnumCasPayloadShapeStaysInSync`` guards, so the
    widening cannot silently regress to the narrower reading.
    """

    def test_the_213_write_read_law_names_the_arithmetic_refusal(self) -> None:
        body = _read(RST)
        start = body.index("**The write/read law.**")
        end = body.index("**No aggregation kind.**", start)
        section = body[start:end]
        assert "E-EVAL-042" in section
        assert "add" in section and "sub" in section and "scale" in section, (
            "the §2.13 write/read law's E-EVAL-042 bullet must name the "
            "add/sub/scale arithmetic refusal (#528 fix round Item B), not "
            "just the write-shape law"
        )

    def test_the_46_class_table_names_the_arithmetic_refusal_too(self) -> None:
        body = _read(RST)
        start = body.index("   * - Evaluation")
        end = body.index("**Every code the R9 chapters add", start)
        section = body[start:end]
        assert "field's write path at runtime" in section
        assert "add" in section and "sub" in section and "scale" in section, (
            "the §4.6 Evaluation class-table row must ALSO name the "
            "add/sub/scale arithmetic refusal (#528 fix round Item B), not "
            "just the write-shape violation"
        )

    def test_d118_is_recorded_in_the_register(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D118$", body, re.MULTILINE), (
            "the E-EVAL-042 arithmetic-refusal widening is a decision and "
            "owes a Draft-Ruling Register row (#528 fix round)"
        )
        start = body.index("\nDraft-Ruling Register\n")
        end = body.index("\nSee Also\n", start)
        section = body[start:end]
        # Anchor on the row marker, not the first "D118" mention — D102's
        # row cross-references D118 earlier in the register (the Territory
        # port train's field-of discharge), and a bare index("D118") would
        # window onto that citation instead of the row itself.
        d118_start = section.index("* - D118")
        d118_row = section[d118_start : d118_start + 3000]
        assert "E-EVAL-042" in d118_row
        assert "add" in d118_row and "sub" in d118_row and "scale" in d118_row, (
            "D118's own row text must name the add/sub/scale refusal it resolves"
        )
        assert "check_no_arithmetic_on_enum_field" in d118_row, (
            "D118's own row text must also record the load-time half (#528 "
            "fix round Item C) — the same law moved earlier, not a second "
            "decision"
        )


class TestThe46RowNamesTheUnregisteredMemberRuntimeTwin:
    """#528 delta-verify rider R3. §4.6's Evaluation class-table row names
    three of the four ``enum_write_value`` (``structural_verbs.rs``)
    emitting sites for its §2.13 ``E-EVAL-042`` clause — the write-shape
    violation (a non-``<enum-ref>`` value, or a cross-type ``<enum-ref>``)
    and the ``add``/``sub``/``scale`` arithmetic refusal (D118). The
    FOURTH site — an ``<enum-ref>`` of the RIGHT declared type naming a
    member that type does not declare (``enum_write_value``'s
    ``self.enums.ordinal(ty, &member)`` arm) — is ``E-EVAL-042`` too, and
    is the runtime twin of ``E-LOAD-055`` (already named in the Load/link
    row), but was never named in the Evaluation row itself.
    """

    def test_the_46_row_names_the_unregistered_member_case(self) -> None:
        body = _read(RST)
        start = body.index("   * - Evaluation")
        end = body.index("**Every code the R9 chapters add", start)
        section = body[start:end]
        assert "E-LOAD-055" in section and "runtime twin" in section, (
            "the §4.6 Evaluation class-table row must name the "
            "right-type/unregistered-member runtime case as E-LOAD-055's "
            "runtime twin — #528 delta-verify rider R3"
        )


class TestTheDraftRulingRegisterHasNoDuplicateRowNumbers:
    """A second row naming a D-number already in use passes every
    existence-only check above — each of D94/D95/D98/D99's own tests just
    confirms ITS OWN number appears somewhere, which stays true whether
    that number appears once or twice. This is the guard a numbering
    COLLISION between two concurrently-drafted PRs needs: #500 (this
    addendum) and a parallel plan both independently reached for D99, and
    the collision was caught by a human noticing during review, not by any
    test — this row makes the next one mechanical.
    """

    def test_every_register_row_number_is_unique(self) -> None:
        body = _read(RST)
        start = body.index("\nDraft-Ruling Register\n")
        end = body.index("\nSee Also\n", start)
        section = body[start:end]
        numbers = re.findall(r"^   \* - D(\d+)$", section, re.MULTILINE)
        assert len(numbers) >= 90, (
            f"only found {len(numbers)} register rows — the section "
            "boundaries this test scans between may have drifted"
        )
        duplicates = sorted({n for n in numbers if numbers.count(n) > 1}, key=int)
        assert not duplicates, (
            f"the Draft-Ruling Register has duplicate row numbers: {duplicates} "
            "— two rows claiming the same D-number, exactly the collision "
            "class a concurrently-drafted PR can introduce"
        )


class TestTheEnumRowStaysInSync:
    """The enum deffield row (spec §1 Q12 of the Organization contract) —
    RHS-grain checks so the row cannot silently lose its way to be written.

    This is a Task-1 red/green pair, not a D99-pattern RHS-grain guard over
    an already-landed production: it exists to force the four textual
    moves Q12 requires (the seventh type-table row, the sealing-paragraph
    rewrite, the ebnf mirror, and the explicit D94-supersession record)
    into existence in one commit, the same way ``TestTheRatioLiteralStaysInSync``
    forced D99's four moves.
    """

    def test_the_rst_type_table_has_an_enum_row(self) -> None:
        # Scoped to §3.1's own list-table: an unscoped scan over the whole
        # document is satisfied vacuously by §5.2's UNRELATED CAS atom-kind
        # row (`* - ``enum`` / ASCII <EnumType>/<MEMBER_IDENTIFIER>``), which
        # names the same word for a different reason and exists already.
        body = _read(RST)
        start = body.index("3.1 Types")
        end = body.index("3.2 Currency operator", start)
        section = body[start:end]
        assert re.search(r"^\s+\* - ``enum``", section, re.MULTILINE), (
            "the <type-name> table must carry the enum row (spec §1 Q12)"
        )

    def test_the_sealing_paragraph_counts_seven(self) -> None:
        body = _read(RST)
        assert "seven rows" in body and "no ``<type-name>`` position can name" in body, (
            "D94's sealing paragraph must be rewritten to seven rows with the "
            "Q12 supersession recorded, not silently contradicted"
        )

    def test_the_ebnf_has_defenum_and_defvocabulary(self) -> None:
        body = _ebnf_text()
        assert "defenum" in body and "defvocabulary" in body

    def test_the_supersession_d_row_is_recorded(self) -> None:
        # Resolve the number at PR-open; the row must cite Q12 and D94 by name.
        assert re.search(r"supersed\w+ D94", _read(RST)), (
            "the register row must record that the enum row supersedes D94's "
            "exclusion by Director ruling (spec §1 Q12), never silently"
        )


class TestTheD119HydrationSplitIsRecordedInTheRegister:
    """G2 (#534 fix round 2 item 2, mutation-reproduced). D119 already
    recorded the RULE-form split (``grammar::check_enum_ref_kinds``'s
    ``E-TYPE-011`` vs ``ClosedVocabulary::check_enum_ref``'s
    ``E-LOAD-030``/``E-LOAD-031``) but said nothing about the SECOND,
    independent producer that conflated them the same way:
    ``scenario::demand_enum_kind``, the unconditional hydration kind-demand
    a ``.bscn`` file's own ``node``/``edge`` forms go through — §3.9
    clause 1 authorizes it ("hydration is not a back door into the closed
    vocabulary"). RHS-grain checks, the same class ``TestTheEnumCasPayload
    ShapeStaysInSync``/``TestTheEnumArithmeticRefusalIsDeclaredInTheRegistry``
    guard for D117/D118, so the extension cannot silently regress to the
    narrower reading.
    """

    def test_d119_is_recorded_in_the_register(self) -> None:
        body = _read(RST)
        assert re.search(r"^\s+\* - D119$", body, re.MULTILINE), (
            "D119 must still have its own Draft-Ruling Register row"
        )
        start = body.index("\nDraft-Ruling Register\n")
        end = body.index("\nSee Also\n", start)
        section = body[start:end]
        d119_start = section.index("D119")
        d119_row = section[d119_start:]
        assert "demand_enum_kind" in d119_row, (
            "D119's row text must name scenario.rs's own hydration-side "
            "kind-demand producer, not just the rule-form one"
        )
        assert "E-TYPE-011" in d119_row and "E-LOAD-030" in d119_row, (
            "D119's row text must record the SAME class split at hydration "
            "positions that it already does for rule-form positions"
        )
        assert "3.9" in d119_row, (
            "D119's row text must cite §3.9 clause 1's hydration authority "
            "('hydration is not a back door into the closed vocabulary')"
        )
        assert "WrongEnumKind" in d119_row, (
            "D119's row text must name the reference implementation's own "
            "new variant (VocabularyError::WrongEnumKind), not just the code"
        )

    def test_d119_says_any_not_one_of_the_sixteen(self) -> None:
        # G3(e) (#534 fix round 2): "at one of the sixteen" reads as if the
        # divergence were localized to a single position rather than
        # holding at any of the sixteen alike. `\s+` (not a literal space)
        # because the rst hard-wraps prose across lines, so "at" and "any"
        # can land on different source lines while still being one phrase.
        body = _read(RST)
        start = body.index("\nDraft-Ruling Register\n")
        end = body.index("\nSee Also\n", start)
        section = body[start:end]
        assert re.search(r"at\s+any\s+of\s+the\s+sixteen", section)
        assert not re.search(r"at\s+one\s+of\s+the\s+sixteen", section)
