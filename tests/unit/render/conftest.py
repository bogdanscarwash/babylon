"""Env pinning for render-seam snapshot goldens.

The repo-wide mise ``[env]`` exports ``NO_COLOR=1`` (agent ergonomics —
uncolored tool output). Textual honors ``NO_COLOR`` even under the headless
snapshot harness by desaturating every theme color to grayscale, which
would (a) make the SVG goldens blind to ksbc-palette regressions and
(b) split rendering into two incompatible lanes — mise/CI (grayscale) vs a
direct venv pytest call (truecolor) — so a golden could only ever match one
lane. Snapshot tests own this knob explicitly instead: colors are always
on, and all lanes produce identical bytes. Mirrors ``tests/unit/tui/conftest.py``.
"""

from __future__ import annotations

import pytest


@pytest.fixture(autouse=True)
def _truecolor_snapshots(monkeypatch: pytest.MonkeyPatch) -> None:
    """Render snapshot goldens in full color regardless of the host env."""
    monkeypatch.delenv("NO_COLOR", raising=False)
