"""Behavioral contracts for the soundtrack estate (``assets/music``).

Same rewrite-test doctrine as the SFX suite: the composition modules are the
spec, rendering is pure, and the committed ``.mid`` files must byte-match a
fresh render — regenerate with ``mise run midi:generate-soundtrack`` after
any composition change.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
MUSIC_DIR = REPO_ROOT / "assets" / "music"

EXPECTED_SUITES = {
    "ambient": 1,
    "superstructure": 3,
    "periphery": 2,
    "rift": 2,
    "endgame": 5,
    "entity": 4,
}


def _load_generator() -> ModuleType:
    src_root = str(REPO_ROOT)
    if src_root not in sys.path:
        sys.path.insert(0, src_root)
    return importlib.import_module("tools.audio.music.generate_music")


def test_registry_census_and_suite_placement() -> None:
    generator = _load_generator()
    suites: dict[str, int] = {}
    for module, suite, _index in generator.TRACKS:
        score = module.compose()
        assert score.suite == suite
        suites[suite] = suites.get(suite, 0) + 1
    assert suites == EXPECTED_SUITES


def test_regeneration_is_byte_identical(tmp_path: Path) -> None:
    generator = _load_generator()
    assert generator.main(["--out-dir", str(tmp_path)]) == 0
    fresh = {path.relative_to(tmp_path) for path in tmp_path.rglob("*.mid")}
    committed = {
        path.relative_to(MUSIC_DIR)
        for suite in EXPECTED_SUITES
        for path in (MUSIC_DIR / suite).glob("*.mid")
    }
    assert fresh == committed, "committed .mid set drifted from the composition modules"
    for relative in sorted(fresh):
        expected = (MUSIC_DIR / relative).read_bytes()
        actual = (tmp_path / relative).read_bytes()
        assert actual == expected, f"{relative} is not byte-identical on regeneration"


def test_durations_inside_suite_bounds() -> None:
    generator = _load_generator()
    for module, suite, _index in generator.TRACKS:
        score = module.compose()
        low, high = generator.SUITE_DURATION_BOUNDS[suite]
        assert low <= score.duration_seconds() <= high, score.name


def test_kit_fails_loud_on_unsanctioned_controller() -> None:
    _load_generator()
    composer = importlib.import_module("tools.audio.music.composer")
    score = composer.Score(name="bad_cc", suite="rift", concept="x", bpm=100)
    with pytest.raises(ValueError, match="not sanctioned"):
        score.cc(0, 5, 0.0, 64)


def test_kit_fails_loud_on_program_reassignment() -> None:
    _load_generator()
    composer = importlib.import_module("tools.audio.music.composer")
    score = composer.Score(name="bad_program", suite="rift", concept="x", bpm=100)
    score.program(0, 42)
    with pytest.raises(ValueError, match="reassigned"):
        score.program(0, 6)
