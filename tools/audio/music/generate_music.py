#!/usr/bin/env python3
"""Render the Babylon soundtrack estate: 17 tracks, 6 suites, one command.

Each module in ``tracks/`` exposes a pure ``compose() -> Score``; this entry
point renders them all to ``<suite>/<nn>_<name>.mid`` deterministically.
Byte-identity is pinned by ``tests/unit/assets/test_music_assets.py``.

Usage::

    uv run python tools/audio/music/generate_music.py [--out-dir DIR]
    mise run midi:generate-soundtrack

:license: CC0-1.0 (see ``assets/audio-LICENSE``).
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Final

_REPO_ROOT = Path(__file__).resolve().parents[3]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from tools.audio.music.tracks import (  # noqa: E402
    beast_engine,
    dissection,
    dual_power,
    history_breathing,
    iron_consolidation,
    officeholder,
    overshoot,
    red_dawn,
    shattered_map,
    superwage,
    the_ballot,
    the_long_winter,
    the_mask,
    the_reform_ceiling,
    the_silent_spring,
    tribute_bleed,
    unequal_exchange,
)

#: Render order: (module, suite, index). Suites mirror the game states the
#: music serves; indices give stable, sortable filenames.
TRACKS: Final = (
    (history_breathing, "ambient", 1),
    (the_ballot, "superstructure", 1),
    (the_reform_ceiling, "superstructure", 2),
    (officeholder, "superstructure", 3),
    (unequal_exchange, "periphery", 1),
    (superwage, "periphery", 2),
    (overshoot, "rift", 1),
    (the_silent_spring, "rift", 2),
    (red_dawn, "endgame", 1),
    (the_long_winter, "endgame", 2),
    (iron_consolidation, "endgame", 3),
    (dual_power, "endgame", 4),
    (shattered_map, "endgame", 5),
    (beast_engine, "entity", 1),
    (tribute_bleed, "entity", 2),
    (dissection, "entity", 3),
    (the_mask, "entity", 4),
)

#: Wall-clock sanity bounds per suite (seconds) — a track outside these is a
#: composition bug, failed loud before any file is written.
SUITE_DURATION_BOUNDS: Final[dict[str, tuple[float, float]]] = {
    "ambient": (150.0, 330.0),
    "superstructure": (90.0, 200.0),
    "periphery": (90.0, 220.0),
    "rift": (90.0, 220.0),
    "endgame": (80.0, 200.0),
    "entity": (120.0, 330.0),
}


def main(argv: list[str] | None = None) -> int:
    """Render every track; print a census table."""
    here = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description="Render the Babylon soundtrack estate.")
    parser.add_argument("--out-dir", type=Path, default=here.parents[2] / "assets" / "music")
    args = parser.parse_args(argv)
    print(f"{'track':28} {'suite':15} {'beats':>7} {'mm:ss':>6} {'events':>7} {'bytes':>7}")
    for module, suite, index in TRACKS:
        score = module.compose()
        if score.suite != suite:
            raise ValueError(f"{score.name}: module suite {score.suite!r} != registry {suite!r}")
        low, high = SUITE_DURATION_BOUNDS[suite]
        seconds = score.duration_seconds()
        if not low <= seconds <= high:
            raise ValueError(f"{score.name}: {seconds:.0f}s outside {suite} bounds [{low},{high}]")
        target = args.out_dir / suite / f"{index:02d}_{score.name}.mid"
        target.parent.mkdir(parents=True, exist_ok=True)
        score.render().save(str(target))
        events = len(score._notes) + len(score._ccs) + len(score._bends)  # noqa: SLF001
        minutes, secs = divmod(round(seconds), 60)
        print(
            f"{score.name:28} {suite:15} {score.end_beats():7.1f} {minutes:3d}:{secs:02d}"
            f" {events:7d} {target.stat().st_size:7d}"
        )
    print(f"Rendered {len(TRACKS)} tracks.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
