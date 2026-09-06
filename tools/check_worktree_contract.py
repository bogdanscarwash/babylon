#!/usr/bin/env python3
"""Worktree environment contract (ADR181 R9c — scar class #2, >=5 incidents).

Five asserts, each mapped to a real debugging session this class cost:

1. The venv interpreter matches ``.python-version`` (worktree venvs created
   against the wrong interpreter shadow imports unpredictably).
2. The ``ops`` extra is installed (repository automation depends on the
   operator-only Ansible toolchain).
3. Every ``data/`` symlink resolves and the reference DB is present (fresh
   worktrees lack the symlink farm; tests then auto-create empty sqlite).
4. ``.env`` exists.
5. The index and working ``uv.lock`` match HEAD, or the exact single incoming
   commit during a merge. Incidental re-locks remain forbidden. Use
   ``UV_FROZEN=1`` only for ``uv sync`` or ``uv run``; ``uv lock --check``
   must run with ``UV_FROZEN`` unset.

Stdlib-only and interpreter-agnostic on purpose: it runs as the FIRST
pre-commit hook (fail_fast), before anything trusts the venv it is checking.
Also invocable as ``mise run check:worktree-contract``.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

REFERENCE_DB = Path("data/sqlite/marxist-data-3NF.sqlite")


def check_interpreter() -> str | None:
    """Venv Python must match the complete ``.python-version`` pin."""
    pin = Path(".python-version").read_text().strip()
    venv_python = Path(".venv/bin/python")
    if not venv_python.exists():
        return ".venv/bin/python missing — run `uv sync --extra ops --frozen`"
    proc = subprocess.run(
        [str(venv_python), "--version"], capture_output=True, text=True, check=False
    )
    version = proc.stdout.strip().removeprefix("Python ")
    if version != pin:
        return f"venv python {version} does not match .python-version pin {pin}"
    return None


def check_ops_extra() -> str | None:
    """The operator extra must be installed (Ansible is its marker import)."""
    proc = subprocess.run(
        [".venv/bin/python", "-c", "import ansible"], capture_output=True, check=False
    )
    if proc.returncode != 0:
        return (
            "ops extra not installed (import ansible failed) — run `uv sync --extra ops --frozen`"
        )
    return None


def check_data_symlinks() -> str | None:
    """Every data/ symlink resolves; the reference DB is reachable."""
    data = Path("data")
    if not data.is_dir():
        return "data/ missing — copy the symlink farm from the main checkout"
    dangling = [str(p) for p in sorted(data.iterdir()) if p.is_symlink() and not p.exists()]
    if dangling:
        return f"dangling data/ symlinks: {', '.join(dangling)}"
    if not REFERENCE_DB.exists():
        return f"{REFERENCE_DB} missing — fresh worktrees lack the data/ symlink farm"
    return None


def check_dotenv() -> str | None:
    """.env must exist (runtime DSNs and settings load from it)."""
    if not Path(".env").is_file():
        return ".env missing — copy it from the main checkout"
    return None


def _git_bytes(arguments: list[str]) -> bytes | None:
    """Read Git evidence, refusing unavailable or failed commands."""
    try:
        proc = subprocess.run(["git", *arguments], capture_output=True, check=False, timeout=30)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return proc.stdout if proc.returncode == 0 else None


def _lock_reference() -> str | None:
    """Select HEAD or one concrete incoming commit, including linked worktrees."""
    merge_path = _git_bytes(["rev-parse", "--git-path", "MERGE_HEAD"])
    if merge_path is None:
        return None
    try:
        heads = Path(os.fsdecode(merge_path.rstrip(b"\n"))).read_text(encoding="ascii").splitlines()
    except FileNotFoundError:
        return "HEAD"
    except (OSError, UnicodeError):
        return None
    if len(heads) != 1 or re.fullmatch(r"[0-9a-f]{40}(?:[0-9a-f]{24})?", heads[0]) is None:
        return None
    if _git_bytes(["cat-file", "-t", heads[0]]) != b"commit\n":
        return None
    return heads[0]


def check_lock_unmodified() -> str | None:
    """Both resolved index and working lock must match the selected commit bytes."""
    reference = _lock_reference()
    if reference is None:
        return "cannot verify uv.lock: MERGE_HEAD must contain exactly one existing commit"
    expected = _git_bytes(["cat-file", "blob", f"{reference}:uv.lock"])
    indexed = _git_bytes(["cat-file", "blob", ":0:uv.lock"])
    try:
        working = Path("uv.lock").read_bytes()
    except OSError:
        working = None
    if expected is not None and indexed == expected and working == expected:
        return None
    target = "HEAD" if reference == "HEAD" else "the single MERGE_HEAD commit"
    return (
        f"uv.lock must match {target} in both the resolved index and working copy — "
        "use `UV_FROZEN=1` only for `uv sync` or `uv run`; "
        "`uv lock --check` must run with `UV_FROZEN` unset"
    )


def main() -> int:
    checks = (
        check_interpreter,
        check_ops_extra,
        check_data_symlinks,
        check_dotenv,
        check_lock_unmodified,
    )
    failures = [message for check in checks if (message := check()) is not None]
    for message in failures:
        print(f"worktree-contract: {message}", file=sys.stderr)
    if not failures:
        print("worktree-contract: clean")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
