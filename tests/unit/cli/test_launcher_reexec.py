"""Launcher determinism re-exec (T1.2/K3, spine G, adversarial finding 3).

``PYTHONHASHSEED`` cannot be set once the interpreter has started, and the
shipped ``babylon`` launcher never set it anywhere — so ``babylon.cli``
re-execs itself exactly once, before any heavy import, with
``PYTHONHASHSEED=0`` and the BLAS/OpenMP thread caps pinned to ``1``.

These tests always subprocess-spawn a *fresh* interpreter (never
``CliRunner``/in-process import): ``os.execve`` replaces the calling
process image, so asserting its effect from the calling process is
impossible by construction — only a child process's stdout survives to be
inspected. ``tests/unit/cli/test_app.py`` et al. cover the in-process,
``pytest``-guarded path (see ``babylon.cli._reexec_with_sealed_environment``
docstring): that guard is exactly why importing ``babylon.cli`` in those
files never triggers any of this.
"""

from __future__ import annotations

import ast
import os
import subprocess
import sys
from pathlib import Path

# tests/unit/cli/test_launcher_reexec.py -> worktree root is three parents up;
# its src/ is what the subprocess must import from -- the shared venv's
# babylon.pth points at the *main* checkout, not this worktree (the
# "Worktree gotcha" documented in CLAUDE.md).
_WORKTREE_ROOT = Path(__file__).resolve().parents[3]
_WORKTREE_SRC = _WORKTREE_ROOT / "src"

_ENV_VARS = (
    "BABYLON_ENV_SEALED",
    "PYTHONHASHSEED",
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
    # W1.8: rustworkx centrality (graph_algorithms.py) parallelizes via rayon
    # on the tick path -- must match tests/conftest.py's `_blas_var` pin
    # (cross-checked dynamically below, not just listed here).
    "RAYON_NUM_THREADS",
)


def _extract_for_loop_tuple_strings(source: str, loop_var_name: str) -> frozenset[str]:
    """Extract the string literals of a ``for <loop_var_name> in (...):`` tuple.

    Walks the AST rather than regexing the text, so the extraction survives
    comment reflows and stays exact about which literal tuple is inspected.
    Raises if no such loop is found, so a rename anywhere fails loudly instead
    of silently comparing against an empty set.
    """
    tree = ast.parse(source)
    for node in ast.walk(tree):
        if (
            isinstance(node, ast.For)
            and isinstance(node.target, ast.Name)
            and node.target.id == loop_var_name
            and isinstance(node.iter, ast.Tuple)
        ):
            return frozenset(
                elt.value
                for elt in node.iter.elts
                if isinstance(elt, ast.Constant) and isinstance(elt.value, str)
            )
    raise AssertionError(f"no `for {loop_var_name} in (...)` loop found in source")


# Appends one line to $REEXEC_MARKER_FILE every time this script body runs,
# then imports babylon.cli (which may or may not re-exec) and prints the
# resulting env. If the re-exec fires exactly once, the marker file ends up
# with exactly 2 lines: one from the original (unsealed) process, one from
# the re-exec'd (sealed) process that then runs to completion.
_SCRIPT = (
    "import os\n"
    "from pathlib import Path\n"
    "with Path(os.environ['REEXEC_MARKER_FILE']).open('a') as fh:\n"
    "    fh.write('run\\n')\n"
    "import babylon.cli\n"
    f"for name in {_ENV_VARS!r}:\n"
    "    print(f'{name}={os.environ.get(name)}')\n"
)


def _run_script(env: dict[str, str], marker_file: Path) -> subprocess.CompletedProcess[str]:
    env = dict(env)
    env["PYTHONPATH"] = str(_WORKTREE_SRC)
    env["REEXEC_MARKER_FILE"] = str(marker_file)
    return subprocess.run(
        [sys.executable, "-c", _SCRIPT],
        env=env,
        capture_output=True,
        text=True,
        check=False,
        timeout=30,
    )


def _parse_env_lines(stdout: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in stdout.strip().splitlines():
        key, _, value = line.partition("=")
        parsed[key] = value
    return parsed


def test_unsealed_process_reexecs_exactly_once_and_seals_determinism_env(
    tmp_path: Path,
) -> None:
    marker_file = tmp_path / "marker.txt"
    # Deliberately unsealed and un-pinned starting environment: strip any of
    # our own vars that might already be set in the ambient shell/mise env,
    # so the assertions below can only pass if babylon.cli itself set them.
    env = {k: v for k, v in os.environ.items() if k not in _ENV_VARS}

    result = _run_script(env, marker_file)

    assert result.returncode == 0, result.stdout + result.stderr
    parsed = _parse_env_lines(result.stdout)
    assert parsed["BABYLON_ENV_SEALED"] == "1"
    assert parsed["PYTHONHASHSEED"] == "0"
    assert parsed["OMP_NUM_THREADS"] == "1"
    assert parsed["OPENBLAS_NUM_THREADS"] == "1"
    assert parsed["MKL_NUM_THREADS"] == "1"
    assert parsed["NUMEXPR_NUM_THREADS"] == "1"
    assert parsed["RAYON_NUM_THREADS"] == "1"

    # Exactly one re-exec: the script body ran twice (once unsealed, once
    # sealed) -- never once (no re-exec happened) and never 3+ (a loop).
    marker_lines = marker_file.read_text().splitlines()
    assert len(marker_lines) == 2, marker_lines


def test_already_sealed_process_never_reexecs_again(tmp_path: Path) -> None:
    marker_file = tmp_path / "marker.txt"
    env = {k: v for k, v in os.environ.items() if k not in _ENV_VARS}
    env["BABYLON_ENV_SEALED"] = "1"
    # A value the real re-exec would never produce (which always writes
    # "0"), so any change proves a second re-exec fired despite the guard.
    env["PYTHONHASHSEED"] = "999999"

    result = _run_script(env, marker_file)

    assert result.returncode == 0, result.stdout + result.stderr
    parsed = _parse_env_lines(result.stdout)
    assert parsed["PYTHONHASHSEED"] == "999999"

    marker_lines = marker_file.read_text().splitlines()
    assert len(marker_lines) == 1, marker_lines


def test_import_inside_pytest_process_never_reexecs() -> None:
    """The pytest-detection guard is what keeps ``test_app.py``-style
    in-process imports (``CliRunner``, ``from babylon.cli import app``)
    cheap: if it were ever removed, importing ``babylon.cli`` here would
    replace this pytest worker's own process image mid-session."""
    import babylon.cli  # noqa: F401 -- import-for-side-effect is the point

    assert os.environ.get("BABYLON_ENV_SEALED") != "1"


def test_launcher_thread_vars_match_canonical_pin() -> None:
    """The launcher's sealed thread-var set must equal the canonical pin.

    ``tests/conftest.py``'s ``_blas_var`` loop (mirrored in ``.mise.toml``
    ``[env]``) is the canonical BLAS/rayon thread-cap pin --
    see ``tests/unit/test_blas_thread_cap.py``. Comparing hardcoded literals on
    both sides would pass tautologically even if the two drifted (exactly how
    the launcher silently dropped ``RAYON_NUM_THREADS`` while conftest/mise
    both carried it). Parsing both files' actual loop tuples via AST ties
    this assertion to the real production tuple in ``babylon.cli``, not a
    hand-copied stand-in, so any future addition to one side without the other
    fails loudly here instead of only showing up as a runtime determinism bug.
    """
    conftest_source = (_WORKTREE_ROOT / "tests" / "conftest.py").read_text(encoding="utf-8")
    canonical_pin = _extract_for_loop_tuple_strings(conftest_source, "_blas_var")

    launcher_source = (_WORKTREE_SRC / "babylon" / "cli" / "__init__.py").read_text(
        encoding="utf-8"
    )
    launcher_thread_vars = _extract_for_loop_tuple_strings(launcher_source, "thread_var")

    assert launcher_thread_vars == canonical_pin
