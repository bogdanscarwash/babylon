"""Structural contract for the governed player-vocabulary authority."""

from __future__ import annotations

import re
import tomllib
from itertools import islice
from pathlib import Path
from typing import Final

import pytest
import yaml

pytestmark = [pytest.mark.unit]

_ROOT: Final[Path] = Path(__file__).resolve().parents[3]
_ADR221_PATH: Final[Path] = _ROOT / "ai" / "decisions" / "ADR221_game_first_refoundation_v4.yaml"
_ADR177_PATH: Final[Path] = (
    _ROOT / "ai" / "decisions" / "ADR177_verb_matrix_ratified_main_ruleset.yaml"
)
_SFX_MANIFEST_PATH: Final[Path] = _ROOT / "assets" / "sfx" / "manifest.toml"
_BSL_REFERENCE_PATH: Final[Path] = _ROOT / "docs" / "reference" / "bsl-language.rst"

_GOVERNED_DESTINATIONS: Final[tuple[str, str, str]] = (
    "ai/decisions/ADR177_verb_matrix_ratified_main_ruleset.yaml",
    "src/babylon/game/actions/matrix.py",
    "src/babylon/game/actions/registry.py",
)
_ACTIVE_VOCABULARY_PATHS: Final[tuple[str, ...]] = (
    "src/babylon/game/actions/matrix.py",
    "src/babylon/game/actions/registry.py",
    "src/babylon/engine/actions/educate.py",
    "src/babylon/game/tutorial.py",
    "src/babylon/projection/verbs/__init__.py",
    "src/babylon/projection/verbs/preview.py",
    "tests/unit/game/actions/test_verb_matrix.py",
    "tests/unit/projection/verbs/test_plate.py",
    "tests/contract/verbs/test_effects.py",
)
_ARCHITECTURE_AUTHORITY_PATHS: Final[tuple[str, str, str]] = (
    "src/babylon/projection/verbs/submit.py",
    "tests/unit/projection/verbs/test_submit.py",
    "tests/integration/archive/test_verb_resolution.py",
)
_RESISTANCE_SOUND_NAMES: Final[tuple[str, ...]] = (
    "resistance_organize",
    "resistance_educate",
    "resistance_agitate",
    "resistance_protest",
    "resistance_alliance",
    "resistance_strike",
    "resistance_expropriate",
    "resistance_sabotage",
    "resistance_dual_power",
    "resistance_clandestine",
)

AUTHORITY_CONTEXT_CHARS: Final[int] = 256
MAX_ARTICLE_V_MATCHES_PER_FILE: Final[int] = 32
MAX_SOUND_ROWS: Final[int] = 128
MAX_BSL_LINES: Final[int] = 12_000
MAX_NORMATIVE_BLOCK_LINES: Final[int] = 16
MAX_D82_BLOCK_LINES: Final[int] = 24

_ARTICLE_V: Final[re.Pattern[str]] = re.compile(r"\barticle[ _-]+v\b", re.IGNORECASE)
_ROSTER_CONTEXT: Final[re.Pattern[str]] = re.compile(
    r"\b(?:verbs?|roster|matrix|closure)\b|\baction[ _-]+registry\b",
    re.IGNORECASE,
)
_ARCHITECTURE_CONTEXT: Final[re.Pattern[str]] = re.compile(
    r"\b(?:atomicity|adjudicat(?:e|es|ed|ing|ion)|rollback)\b|"
    r"\bworking[ _-]+copy\b|\bdirect[ _-]+(?:graph[ _-]+)?mutation\b|"
    r"\bdeterministic[ _-]+architecture\b",
    re.IGNORECASE,
)
_NORMATIVE_START: Final[str] = "- *Why a verb at all, given Amendment AG (iii).*"
_NORMATIVE_END: Final[str] = "**[draft ruling — Phase 1 review, Amendment AG (i)]**"
_D82_START: Final[str] = "   * - D82"
_D82_END: Final[str] = "   * - D83"
_D82_PRE_AH_FIXTURE: Final[str] = """   * - D82
     - §2.8
     - ``update-membership`` writes payload of an **existing** membership,
       mirroring the other three update verbs and inheriting the range and
       I.15 disciplines. The member list stays whole-object replacement
       (D26), so ``:max-members`` keeps its single check point and VIII.9
       survives verbatim. AG (iii)'s "adds no verb" is read as the NORTH_STAR
       §0 / Article V closure list, against AG (i)'s "mutate only through
       effects" and ADR189 (iv)'s "accessor/verb surface" — the effect-position
       write is required by the amendment, not licensed against it."""


def _yaml_mapping(path: Path) -> dict[str, object]:
    """Load one required YAML mapping."""
    document = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert isinstance(document, dict), path
    return document


def _extract_bounded_block(
    lines: tuple[str, ...],
    *,
    start: str,
    end: str,
    max_block_lines: int,
) -> str:
    """Extract one uniquely delimited block from the bounded BSL reference."""
    bounded_lines = lines[:MAX_BSL_LINES]
    start_indices = tuple(index for index, line in enumerate(bounded_lines) if start in line)
    assert len(start_indices) == 1, f"expected one start delimiter: {start}"
    start_index = start_indices[0]
    permitted_end_lines = bounded_lines[start_index + 1 : start_index + max_block_lines + 1]
    end_offsets = tuple(offset for offset, line in enumerate(permitted_end_lines) if end in line)
    assert len(end_offsets) == 1, f"expected one bounded end delimiter: {end}"
    end_index = start_index + end_offsets[0] + 1
    assert start_index < end_index, f"end delimiter precedes start delimiter: {start}"
    block_lines = bounded_lines[start_index:end_index]
    assert len(block_lines) <= max_block_lines, f"block exceeds {max_block_lines} lines: {start}"
    return "\n".join(block_lines)


def _authority_context(text: str, match: re.Match[str]) -> str:
    """Return a fixed-radius authority context around one Article V match."""
    start = max(0, match.start() - AUTHORITY_CONTEXT_CHARS)
    end = min(len(text), match.end() + AUTHORITY_CONTEXT_CHARS)
    return text[start:end]


def test_v_player_transition_resolves_to_accepted_governed_destination() -> None:
    """ADR221 resolves the live roster to ADR177 and its two canonical sources."""
    adr221 = _yaml_mapping(_ADR221_PATH)["ADR221_game_first_refoundation_v4"]
    assert isinstance(adr221, dict)
    transition = adr221["transition"]
    assert isinstance(transition, dict)
    undotted_sections = transition["undotted_sections"]
    assert isinstance(undotted_sections, dict)
    player = undotted_sections["V.Player"]
    assert isinstance(player, dict)
    assert player["disposition"] == "Re-homed"
    destination = player["destination"]
    assert isinstance(destination, str)
    parts = destination.split(";")
    assert len(parts) == len(_GOVERNED_DESTINATIONS)
    actual_destinations = (parts[0].strip(), parts[1].strip(), parts[2].strip())
    assert actual_destinations == _GOVERNED_DESTINATIONS
    for relative_path in _GOVERNED_DESTINATIONS:
        assert (_ROOT / relative_path).is_file(), f"missing governed destination: {relative_path}"
    adr177 = _yaml_mapping(_ADR177_PATH)["ADR177_verb_matrix_ratified_main_ruleset"]
    assert isinstance(adr177, dict)
    assert adr177["status"] == "accepted"


def test_active_vocabulary_authority_cites_adr177() -> None:
    """Living roster copy cites ADR177 without misusing Article V authority."""
    for relative_path in _ACTIVE_VOCABULARY_PATHS:
        text = (_ROOT / relative_path).read_text(encoding="utf-8")
        assert "ADR177" in text, f"{relative_path}: missing ADR177 authority"
        matches = tuple(islice(_ARTICLE_V.finditer(text), MAX_ARTICLE_V_MATCHES_PER_FILE + 1))
        assert len(matches) <= MAX_ARTICLE_V_MATCHES_PER_FILE, relative_path
        for match in matches[:MAX_ARTICLE_V_MATCHES_PER_FILE]:
            context = _authority_context(text, match)
            if _ROSTER_CONTEXT.search(context) is not None:
                assert _ARCHITECTURE_CONTEXT.search(context) is not None, (
                    f"{relative_path}: Article V still claims vocabulary authority: {context!r}"
                )


def test_resistance_sound_hints_cite_governed_vocabulary() -> None:
    """The ten verb-bound resistance cues name ADR177 as their authority."""
    manifest = tomllib.loads(_SFX_MANIFEST_PATH.read_text(encoding="utf-8"))
    sounds = manifest["sound"]
    assert isinstance(sounds, list)
    assert len(sounds) <= MAX_SOUND_ROWS
    selected: dict[str, list[dict[str, object]]] = {name: [] for name in _RESISTANCE_SOUND_NAMES}
    for sound in sounds[:MAX_SOUND_ROWS]:
        assert isinstance(sound, dict)
        name = sound.get("name")
        if isinstance(name, str) and name in selected:
            selected[name].append(sound)
    for name in _RESISTANCE_SOUND_NAMES:
        rows = selected[name]
        assert len(rows) == 1, f"{name}: expected exactly one manifest row"
        trigger_hint = rows[0].get("trigger_hint")
        assert isinstance(trigger_hint, str)
        assert "ADR177" in trigger_hint, f"{name}: missing ADR177 authority"
        assert _ARTICLE_V.search(trigger_hint) is None, f"{name}: stale Article V authority"


def test_bsl_distinguishes_live_authority_from_frozen_d82_history() -> None:
    """Normative BSL cites ADR177 while D82 preserves and supersedes its history."""
    lines = tuple(_BSL_REFERENCE_PATH.read_text(encoding="utf-8").splitlines())
    assert len(lines) <= MAX_BSL_LINES
    normative = _extract_bounded_block(
        lines,
        start=_NORMATIVE_START,
        end=_NORMATIVE_END,
        max_block_lines=MAX_NORMATIVE_BLOCK_LINES,
    )
    historical = _extract_bounded_block(
        lines,
        start=_D82_START,
        end=_D82_END,
        max_block_lines=MAX_D82_BLOCK_LINES,
    )
    bounded_text = "\n".join(lines[:MAX_BSL_LINES])
    assert bounded_text.count("adds no verb") == 2
    assert normative.count("adds no verb") == 1
    assert historical.count("adds no verb") == 1
    for destination in _GOVERNED_DESTINATIONS:
        assert destination in normative
    assert _ARTICLE_V.search(normative) is None
    assert historical.startswith(_D82_PRE_AH_FIXTURE)
    supersession = historical[len(_D82_PRE_AH_FIXTURE) :]
    assert "Living-authority supersession" in supersession
    assert "pre-AH v3.2.0 historical citation" in supersession
    assert "ADR221" in supersession
    assert "V.Player" in supersession
    assert "authority only" in supersession
    assert "D82's technical ruling applies" in supersession
    for destination in _GOVERNED_DESTINATIONS:
        assert destination in supersession


def test_article_v_architecture_citations_remain_legal() -> None:
    """Current Article V citations remain classified as architecture authority."""
    for relative_path in _ARCHITECTURE_AUTHORITY_PATHS:
        text = (_ROOT / relative_path).read_text(encoding="utf-8")
        matches = tuple(islice(_ARTICLE_V.finditer(text), MAX_ARTICLE_V_MATCHES_PER_FILE + 1))
        assert matches, f"{relative_path}: expected an Article V architecture citation"
        assert len(matches) <= MAX_ARTICLE_V_MATCHES_PER_FILE, relative_path
        for match in matches[:MAX_ARTICLE_V_MATCHES_PER_FILE]:
            context = _authority_context(text, match)
            assert _ARCHITECTURE_CONTEXT.search(context) is not None, (
                f"{relative_path}: Article V citation is not architecture context: {context!r}"
            )
