#!/usr/bin/env python3
"""Fast commit prerequisites: pinned Python and coherent staged dependencies.

The generic commit gate does not require operator extras, private reference
inputs, or a dotenv file. Data and operator commands validate their own inputs.
Both dependency files must match the resolved index, then ``uv lock --check``
validates the current declarations with ``UV_FROZEN`` explicitly unset.

Stdlib-only so this first pre-commit hook can check the environment it uses.
Also invocable as ``mise run check:worktree-contract``.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def check_interpreter() -> str | None:
    """Venv Python must match the complete ``.python-version`` pin."""
    try:
        pin = Path(".python-version").read_text().strip()
    except OSError:
        return "cannot read .python-version"
    venv_python = Path(".venv/bin/python")
    if not venv_python.exists():
        return ".venv/bin/python missing — run `uv sync --frozen`"
    try:
        proc = subprocess.run(
            [str(venv_python), "--version"],
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "cannot execute .venv/bin/python --version"
    if proc.returncode != 0:
        return "venv interpreter version check failed"
    version = proc.stdout.strip().removeprefix("Python ")
    if not pin or version != pin:
        return f"venv python {version} does not match .python-version pin {pin}"
    return None


def _git_bytes(arguments: list[str]) -> bytes | None:
    """Read Git evidence, refusing unavailable or failed commands."""
    try:
        proc = subprocess.run(["git", *arguments], capture_output=True, check=False, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return proc.stdout if proc.returncode == 0 else None


def check_lock_consistency() -> str | None:
    """Validate the exact staged declaration/lock pair, including intentional edits."""
    unmerged = _git_bytes(["ls-files", "--unmerged", "-z"])
    if unmerged is None:
        return "cannot inspect the Git index for unresolved paths"
    if unmerged:
        return "resolve and stage all unmerged index paths before checking dependencies"
    for name in ("pyproject.toml", "uv.lock"):
        indexed = _git_bytes(["cat-file", "blob", f":0:{name}"])
        if indexed is None:
            return f"cannot read resolved index entry for {name}"
        try:
            working = Path(name).read_bytes()
        except OSError:
            return f"cannot read working {name}"
        if indexed != working:
            return f"{name} differs between the index and working copy — stage the intended dependency files together"

    environment = os.environ.copy()
    environment.pop("UV_FROZEN", None)
    try:
        proc = subprocess.run(
            ["uv", "lock", "--check"],
            env=environment,
            capture_output=True,
            text=True,
            check=False,
            timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "cannot complete `uv lock --check` with UV_FROZEN unset"
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout).strip()
        return f"`uv lock --check` failed with UV_FROZEN unset (exit {proc.returncode}): {detail}"
    return None


def main() -> int:
    checks = (check_interpreter, check_lock_consistency)
    failures = [message for check in checks if (message := check()) is not None]
    for message in failures:
        print(f"worktree-contract: {message}", file=sys.stderr)
    if not failures:
        print("worktree-contract: clean")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
