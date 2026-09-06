"""Behavioral contract for the cue map (``assets/CUE_MAP.md``).

The cue map binds every rendered SFX/music asset to the exact engine
vocabulary it fires on. This test is the map's rewrite test: every row's
``bind_target`` must resolve to a real enum member of the vocabulary its
``bind_kind`` names, and every asset that exists on disk (manifest.toml +
generate_music.TRACKS) must have exactly one row-set in the map — no orphan
rows for assets that don't exist, no assets missing a row.

Written red-first (#641 render/pin task): it fails on the legacy 39 SFX +
13 tracks until ``CUE_MAP.md``'s marked "IMPLEMENTER STEP" legacy section is
transcribed.
"""

from __future__ import annotations

import importlib
import importlib.util
import re
import sys
from pathlib import Path
from typing import Final

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
SFX_DIR = REPO_ROOT / "assets" / "sfx"
MUSIC_DIR = REPO_ROOT / "assets" / "music"
CUE_MAP_PATH = REPO_ROOT / "assets" / "CUE_MAP.md"

if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

_SRC_ROOT = REPO_ROOT / "src"
if str(_SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(_SRC_ROOT))

from babylon.kernel.tick_partition import TickPartition  # noqa: E402
from babylon.models.enums.actions import ActionType  # noqa: E402
from babylon.models.enums.events import EventType, GameOutcome  # noqa: E402

#: bind_kind -> (enum class, expected "Prefix." on bind_target)
_ENUM_BY_KIND: Final[dict[str, type]] = {
    "event": EventType,
    "outcome": GameOutcome,
    "phase": TickPartition,
    "verb": ActionType,
}
_PREFIX_BY_KIND: Final[dict[str, str]] = {
    "event": "EventType.",
    "outcome": "GameOutcome.",
    "phase": "TickPartition.",
    "verb": "ActionType.",
}

_ROW_COLUMNS: Final[tuple[str, ...]] = (
    "asset",
    "bind_kind",
    "bind_target",
    "gloss",
    "mix",
    "status",
)


def _parse_rows(text: str) -> list[dict[str, str]]:
    """Parse every real table row out of ``CUE_MAP.md``.

    Skips header rows (``asset`` in column 1) and markdown separator rows
    (``---`` per cell); anything else starting/ending with ``|`` and
    carrying exactly six cells is a data row.
    """
    rows: list[dict[str, str]] = []
    for line in text.splitlines():
        stripped = line.strip()
        if not (stripped.startswith("|") and stripped.endswith("|")):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) != len(_ROW_COLUMNS):
            continue
        if cells[0] == "asset":
            continue
        if all(set(cell) <= {"-"} for cell in cells if cell):
            continue
        rows.append(dict(zip(_ROW_COLUMNS, cells, strict=True)))
    return rows


def _load_sfx_manifest() -> object:
    spec = importlib.util.spec_from_file_location(
        "generate_sfx_for_cue_map", REPO_ROOT / "tools" / "audio" / "sfx" / "generate_sfx.py"
    )
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module.load_manifest(SFX_DIR / "manifest.toml")


def _expected_sfx_assets() -> dict[str, Path]:
    manifest = _load_sfx_manifest()
    return {
        f"sfx/{sound.family}/{sound.name}": SFX_DIR / sound.family / f"{sound.name}.mid"
        for sound in manifest.sounds  # type: ignore[attr-defined]
    }


def _expected_music_assets() -> dict[str, Path]:
    generator = importlib.import_module("tools.audio.music.generate_music")
    expected: dict[str, Path] = {}
    for module, suite, index in generator.TRACKS:
        score = module.compose()
        filename = f"{index:02d}_{score.name}"
        expected[f"music/{suite}/{filename}"] = MUSIC_DIR / suite / f"{filename}.mid"
    return expected


@pytest.fixture(scope="module")
def cue_map_rows() -> list[dict[str, str]]:
    return _parse_rows(CUE_MAP_PATH.read_text(encoding="utf-8"))


@pytest.fixture(scope="module")
def expected_assets() -> dict[str, Path]:
    merged = _expected_sfx_assets()
    merged.update(_expected_music_assets())
    return merged


def _rows_of_kind(rows: list[dict[str, str]], kind: str) -> list[dict[str, str]]:
    return [row for row in rows if row["bind_kind"] == kind]


def test_event_rows_resolve_to_a_real_event_type(cue_map_rows: list[dict[str, str]]) -> None:
    """Check 1: every ``event`` row's bind_target is a real EventType member."""
    rows = _rows_of_kind(cue_map_rows, "event")
    assert rows, "expected at least one event-kind row"
    for row in rows:
        target = row["bind_target"]
        assert target.startswith("EventType."), f"{row['asset']}: {target!r} not EventType.*"
        member = target.removeprefix("EventType.")
        assert member in EventType.__members__, f"{row['asset']}: EventType.{member} does not exist"


def test_outcome_rows_resolve_to_a_real_game_outcome(cue_map_rows: list[dict[str, str]]) -> None:
    """Check 2: every ``outcome`` row's bind_target is a real GameOutcome member."""
    rows = _rows_of_kind(cue_map_rows, "outcome")
    assert rows, "expected at least one outcome-kind row"
    for row in rows:
        target = row["bind_target"]
        assert target.startswith("GameOutcome."), f"{row['asset']}: {target!r} not GameOutcome.*"
        member = target.removeprefix("GameOutcome.")
        assert member in GameOutcome.__members__, (
            f"{row['asset']}: GameOutcome.{member} does not exist"
        )


def test_phase_rows_resolve_to_a_real_tick_partition(cue_map_rows: list[dict[str, str]]) -> None:
    """Check 3: every ``phase`` row's bind_target is a real TickPartition member."""
    rows = _rows_of_kind(cue_map_rows, "phase")
    assert rows, "expected at least one phase-kind row"
    for row in rows:
        target = row["bind_target"]
        assert target.startswith("TickPartition."), (
            f"{row['asset']}: {target!r} not TickPartition.*"
        )
        member = target.removeprefix("TickPartition.")
        assert member in TickPartition.__members__, (
            f"{row['asset']}: TickPartition.{member} does not exist"
        )


def test_verb_rows_resolve_to_a_real_action_type(cue_map_rows: list[dict[str, str]]) -> None:
    """Check 4: every ``verb`` row's bind_target is a real ActionType member."""
    rows = _rows_of_kind(cue_map_rows, "verb")
    assert rows, "expected at least one verb-kind row"
    for row in rows:
        target = row["bind_target"]
        assert target.startswith("ActionType."), f"{row['asset']}: {target!r} not ActionType.*"
        member = target.removeprefix("ActionType.")
        assert member in ActionType.__members__, (
            f"{row['asset']}: ActionType.{member} does not exist"
        )


def test_every_asset_has_exactly_one_row_set_and_names_a_real_file(
    cue_map_rows: list[dict[str, str]], expected_assets: dict[str, Path]
) -> None:
    """Check 5: bijection between real assets and CUE_MAP.md asset groupings.

    Every asset in ``manifest.toml`` + ``generate_music.TRACKS`` must appear
    (at least once) as a row's ``asset`` cell, every ``asset`` cell in the
    map must name a real asset, and every named asset's file must exist on
    disk.
    """
    assert not re.search(r"IMPLEMENTER STEP", CUE_MAP_PATH.read_text(encoding="utf-8")), (
        "CUE_MAP.md still carries its IMPLEMENTER STEP marker — legacy transcription incomplete"
    )
    rows_by_asset: dict[str, list[dict[str, str]]] = {}
    for row in cue_map_rows:
        rows_by_asset.setdefault(row["asset"], []).append(row)

    mapped_assets = set(rows_by_asset)
    real_assets = set(expected_assets)

    missing = real_assets - mapped_assets
    assert not missing, f"real assets with no CUE_MAP.md row: {sorted(missing)}"

    orphaned = mapped_assets - real_assets
    assert not orphaned, f"CUE_MAP.md rows naming nonexistent assets: {sorted(orphaned)}"

    for asset, path in expected_assets.items():
        assert path.is_file(), f"{asset}: no rendered file at {path}"


_SUPPORTED_KINDS: Final[frozenset[str]] = frozenset(
    {"event", "outcome", "phase", "verb", "ui", "state"}
)


def test_every_row_uses_a_supported_bind_kind(cue_map_rows: list[dict[str, str]]) -> None:
    """A typo'd ``bind_kind`` (e.g. ``evnt``) must fail loudly, not slip past the
    kind-scoped checks above by matching none of them (Copilot harvest, PR #668)."""
    bad = [
        (row["asset"], row["bind_kind"])
        for row in cue_map_rows
        if row["bind_kind"] not in _SUPPORTED_KINDS
    ]
    assert not bad, f"rows with unsupported bind_kind: {bad}"
