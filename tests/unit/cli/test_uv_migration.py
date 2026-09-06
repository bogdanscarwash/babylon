"""uv single-lock migration invariants (ADR095 D3)."""

from __future__ import annotations

import tomllib
from pathlib import Path

ROOT = Path(".")
PYPROJECT = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
MISE = (ROOT / ".mise.toml").read_text(encoding="utf-8")
PRECOMMIT = (ROOT / ".pre-commit-config.yaml").read_text(encoding="utf-8")


def test_dependency_groups_are_pep735() -> None:
    groups = PYPROJECT["dependency-groups"]
    assert "dev" in groups and "docs" in groups
    assert "poetry" not in PYPROJECT.get("tool", {}), "legacy [tool.poetry] table remains"


def test_uv_lock_committed_poetry_lock_gone() -> None:
    assert (ROOT / "uv.lock").exists(), "uv.lock must be generated + committed"
    assert not (ROOT / "poetry.lock").exists(), (
        "poetry.lock must be deleted at the single-lock fork"
    )


def test_mise_has_no_poetry_invocations() -> None:
    assert "poetry run" not in MISE
    assert "poetry install" not in MISE
    # [tools] pin swapped poetry -> uv
    assert "\npoetry = " not in MISE
    assert "uv = " in MISE


def test_precommit_lock_hook_is_uv() -> None:
    assert "uv lock --check" in PRECOMMIT
    assert "poetry check --lock" not in PRECOMMIT
    assert "poetry run" not in PRECOMMIT


def test_build_system_is_not_poetry() -> None:
    """Backend regression guard: the 2026-07-22 hatchling cutover removed the
    last Poetry artifact from the toolchain; it must never come back."""
    build = PYPROJECT["build-system"]
    assert build["build-backend"] == "hatchling.build"
    assert all("poetry" not in req for req in build["requires"])


def test_rust_client_wheel_is_gone() -> None:
    """Amendment AF (ADR186) deletion ceremony: the babylon-tui maturin wheel
    that the M7 packaging flip (BD-5 / ADR150 Task 44) had made a default
    dependency is deleted along with the Ratatui client it built. ``uv sync``
    must need no cargo/rustc; no doc or incantation may keep pointing at the
    retired wheel or its opt-in ``tui`` dependency-group predecessor.
    """
    deps = PYPROJECT["project"]["dependencies"]
    assert "babylon-tui" not in deps, (
        "babylon-tui must not be a [project] dependency post-AF deletion"
    )
    assert "tui" not in PYPROJECT["dependency-groups"], (
        "the opt-in tui group is retired at the M7 packaging flip"
    )
    assert "babylon-tui" not in PYPROJECT.get("tool", {}).get("uv", {}).get("sources", {}), (
        "the [tool.uv.sources] path source for the wheel must be removed"
    )


def test_python_version_pin_consistency() -> None:
    """Native mise and uv share an exact patch; static checkers share its minor."""
    pin = (ROOT / ".python-version").read_text(encoding="utf-8").strip()
    assert pin == "3.12.14"
    minor = ".".join(pin.split(".")[:2])
    assert PYPROJECT["tool"]["mypy"]["python_version"] == minor
    assert PYPROJECT["tool"]["ruff"]["target-version"] == "py" + minor.replace(".", "")
    assert PYPROJECT["project"]["requires-python"].startswith(">=" + minor)
    assert f'python = "{pin}"' in MISE
