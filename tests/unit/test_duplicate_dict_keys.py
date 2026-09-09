"""Duplicate dict keys that ruff cannot see (sentinel for a live error class).

Discovered 2026-07-30 in ``src/babylon/models/event_severity.py``: PR #395
reclassified ``FASCIST_RECRUITMENT`` from TERMINAL_ADJACENT to
TERMINAL_APPROACH and added a new ``_DRIFT_RATIONALES`` entry, but left the
old entry in place. Python's dict literal silently keeps the LAST value, so
the live rationale became the stale, contradictory text — no error, no
warning, no test failure.

**Why the linter missed it.** Ruff/pyflakes flag repeated *literal* keys
(``F601``) and repeated *variable* keys (``F602``). They do NOT flag repeated
**attribute** keys — and every taxonomy mapping in this codebase is keyed by
an enum member (``EventType.X``, ``NodeType.Y``), which is an attribute
expression. Verified empirically: a dict with ``{E.A: 1, E.B: 2, E.A: 3}``
passes ``ruff check --select F`` clean, while the same dict with string keys
raises F601. So the single most common mapping shape in this repo had zero
duplicate-key protection.

This closes that gap: an AST sweep for repeated dotted-name keys inside one
dict literal, across ``src/``.
"""

from __future__ import annotations

import ast
from pathlib import Path

import pytest

#: Resolved from ``__file__``, not the cwd — matching the sentinel-suite
#: precedent (tests/unit/sentinels/test_ast_helpers.py:38 et al.) so a
#: pytest run from a subdirectory cannot silently sweep nothing.
_REPO_ROOT: Path = Path(__file__).resolve().parents[2]
SRC = _REPO_ROOT / "src" / "babylon"


def _dotted_name(node: ast.expr) -> str | None:
    """Render an attribute chain (``EventType.FASCIST_RECRUITMENT``) as text.

    :param node: The dict-key expression to render.
    :returns: The dotted source name, or ``None`` when the key is not a plain
        attribute chain (a literal, a call, a subscript — someone else's
        problem, or ruff's).
    """
    parts: list[str] = []
    current: ast.expr = node
    while isinstance(current, ast.Attribute):
        parts.append(current.attr)
        current = current.value
    if not isinstance(current, ast.Name) or not parts:
        return None
    parts.append(current.id)
    return ".".join(reversed(parts))


def duplicate_attribute_keys(tree: ast.AST, filename: str) -> list[str]:
    """One message per dict literal that repeats an attribute key.

    :param tree: Parsed module AST.
    :param filename: Display name for the messages.
    :returns: Human-readable violations; empty when the module is clean.
    """
    violations: list[str] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Dict):
            continue
        seen: dict[str, int] = {}
        for key in node.keys:
            # `**spread` entries carry a None key — nothing to compare.
            if key is None:
                continue
            name = _dotted_name(key)
            if name is None:
                continue
            if name in seen:
                violations.append(
                    f"{filename}:{key.lineno}: duplicate dict key {name} "
                    f"(first at line {seen[name]}) — the later value silently wins"
                )
            else:
                seen[name] = key.lineno
    return violations


@pytest.mark.skipif(not SRC.is_dir(), reason="src/babylon not present")
class TestNoDuplicateAttributeKeys:
    """No dict literal in src/ repeats an enum-member key."""

    def test_src_tree_is_clean(self) -> None:
        violations: list[str] = []
        for path in sorted(SRC.rglob("*.py")):
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            violations.extend(duplicate_attribute_keys(tree, str(path)))
        assert not violations, "\n".join(violations)

    def test_checker_catches_the_event_severity_shape(self) -> None:
        # Mutation validation: the exact historical bug, reduced.
        tree = ast.parse(
            "RATIONALES = {\n"
            "    EventType.FASCIST_DRIFT: 'a',\n"
            "    EventType.FASCIST_RECRUITMENT: 'correct',\n"
            "    EventType.ORGANIZATIONAL_FRACTURE: 'b',\n"
            "    EventType.FASCIST_RECRUITMENT: 'stale, and this one WINS',\n"
            "}\n"
        )
        assert duplicate_attribute_keys(tree, "event_severity.py") == [
            "event_severity.py:5: duplicate dict key EventType.FASCIST_RECRUITMENT "
            "(first at line 3) — the later value silently wins"
        ]

    def test_distinct_members_and_spreads_are_not_flagged(self) -> None:
        tree = ast.parse("X = {EventType.A: 1, EventType.B: 2, **other, NodeType.A: 3}\n")
        assert duplicate_attribute_keys(tree, "x.py") == []

    def test_ruff_already_covers_literal_keys_so_we_do_not(self) -> None:
        # Literal duplicates are F601's job; this checker deliberately
        # ignores them rather than double-reporting.
        tree = ast.parse("X = {'a': 1, 'a': 2}\n")
        assert duplicate_attribute_keys(tree, "x.py") == []
