"""Babylon command-line interface (ADR095 D1).

Entry point: ``[project.scripts] babylon = "babylon.cli:app"``. ``babylon``
with no subcommand prints help (Amendment AF / ADR186: the game launches as
a standalone Bevy binary, not through this CLI). The subcommand modules
reuse the landed §A8 provider seam and config-dir conventions — nothing
here reinvents them.
"""

from __future__ import annotations

import os
import sys


def _reexec_with_sealed_environment() -> None:
    """Re-exec the process once with a determinism-sealed environment.

    ``PYTHONHASHSEED`` cannot be changed once the interpreter has started —
    CPython reads it exactly once, at start-up, to seed ``str``/``bytes``/
    ``datetime`` hash randomization — and the shipped launcher never set it
    (adversarial finding 3, ``ai/_inbox/PROGRAM_v1_0_0_playable_archive.md``
    spine G). The only way to pin it for a process that already exists is to
    replace that process image with a fresh interpreter that has the
    variable pre-set: an :func:`os.execve` re-exec. It also pins the
    BLAS/OpenMP/rayon thread caps to ``1`` — the same five-variable pin
    ``tests/conftest.py`` (its ``_blas_var`` loop), ``.mise.toml``'s
    ``[env]`` already enforce (deterministic FP
    reduction order, Constitution III.7; see the dev-box-freeze history
    in ``CLAUDE.md``). The rayon variable covers rustworkx's centrality
    functions (``topology/graph_algorithms.py``), which parallelize via
    rayon above their node-count threshold and run live on the tick path
    (``ooda/attention/sparrow.py``, ``domain/bifurcation/resilience.py``,
    ``domain/organizations/topology.py`` — W1.8, also guarded by
    ``tests/unit/test_blas_thread_cap.py``). So a real ``babylon play``
    run gets the identical single-threaded, byte-identical arithmetic the
    test suite already proves, not just the tests.

    Guarded two ways so it fires **at most once** per process tree:

    - ``BABYLON_ENV_SEALED`` is set only on the re-exec'd process's own
      environment, so that process always takes this function's early
      return and never re-execs again — a single, statically-bounded
      transition (unsealed -> exec -> sealed), not an unbounded loop.
    - ``sys.modules`` already containing ``pytest`` skips the re-exec
      entirely. Under the test harness this package is imported in-process
      many times over (``CliRunner``, ``from babylon.cli import app`` in
      ``tests/unit/cli/test_app.py`` et al.); replacing the pytest worker's
      own process image mid-collection would restart test collection
      instead of exercising a subcommand. The real launch path (the
      installed ``babylon`` script, or a subprocess a test spawns
      deliberately — see ``tests/unit/cli/test_launcher_reexec.py``) never
      has ``pytest`` imported, so this guard never fires there.
    """
    if os.environ.get("BABYLON_ENV_SEALED") == "1" or "pytest" in sys.modules:
        return

    sealed_env = dict(os.environ)
    sealed_env["BABYLON_ENV_SEALED"] = "1"
    sealed_env["PYTHONHASHSEED"] = "0"
    for thread_var in (
        "OMP_NUM_THREADS",
        "OPENBLAS_NUM_THREADS",
        "MKL_NUM_THREADS",
        "NUMEXPR_NUM_THREADS",
        # W1.8: rustworkx centrality parallelizes via rayon above its
        # parallel_threshold (50 nodes) -- same per-core oversubscription
        # and FP-reduction-order determinism hazard as the BLAS vars above.
        # Mirrors tests/conftest.py's `_blas_var` tuple / .mise.toml [env],
        # kept in parity by
        # tests/unit/cli/test_launcher_reexec.py::
        # test_launcher_thread_vars_match_canonical_pin.
        "RAYON_NUM_THREADS",
    ):
        sealed_env[thread_var] = "1"

    # sys.orig_argv (3.10+) is the exact argv the interpreter was invoked
    # with -- interpreter path, any -m/-c form, and all original arguments --
    # so re-exec reproduces the original command line byte-for-byte, just
    # under a sealed environment. argv[0] is always an absolute path here
    # (however the OS resolved the shebang/interpreter that started this
    # process), so execve needs no PATH search and argv is our own captured
    # invocation, never attacker-controlled input.
    argv = list(sys.orig_argv)
    os.execve(argv[0], argv, sealed_env)  # noqa: S606 — re-exec IS the feature; argv is our own sys.orig_argv


_reexec_with_sealed_environment()

import typer  # noqa: E402

from babylon import __version__  # noqa: E402
from babylon.config.logging_config import setup_logging  # noqa: E402
from babylon.render.session import set_render_override  # noqa: E402
from babylon.render.tiers import RenderTier  # noqa: E402

# Observability Spine (T1.2/K1): the single entry point for the shipped
# `babylon` command initializes central logging before any subcommand
# module is imported/executed — mirrors babylon/__main__.py's demo-entry
# pattern so both real launch paths go through the same dictConfig.
setup_logging()

app = typer.Typer(
    name="babylon",
    help="Babylon — The Fall of America. A Marxist simulation engine.",
    add_completion=False,
    no_args_is_help=False,
)


def _register() -> None:
    """Register subcommands. Lazy imports keep the root ``--help`` fast and
    avoid import cycles between subcommand modules and the seam."""
    from babylon.cli import doctor as doctor_cmd
    from babylon.cli import login as login_cmd
    from babylon.cli import telemetry as telemetry_cmd
    from babylon.cli import uninstall as uninstall_cmd

    app.command(name="doctor")(doctor_cmd.doctor)
    app.command(name="login")(login_cmd.login)
    app.command(name="telemetry")(telemetry_cmd.telemetry)
    app.command(name="uninstall")(uninstall_cmd.uninstall)


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(__version__)
        raise typer.Exit()


@app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    version: bool = typer.Option(  # noqa: ARG001 — consumed via eager callback, not body
        False,
        "--version",
        callback=_version_callback,
        is_eager=True,
        help="Show the Babylon version and exit.",
    ),
    # noqa: B008 below — typer's DI-style Option() default; ruff's bugbear allowlist for
    # Option/Argument defaults doesn't cover Enum-typed params (verified: str/bool are exempt).
    render: RenderTier | None = typer.Option(  # noqa: B008
        None,
        "--render",
        help="Override the persisted render tier for this session (glyph|pixel).",
    ),
) -> None:
    """Babylon CLI root. With no subcommand, prints help (Amendment AF /
    ADR186: the play path retired with the Ratatui client; the v1.0 client
    is the standalone `babylon-client` Bevy binary, not this CLI)."""
    set_render_override(render)
    if ctx.invoked_subcommand is None:
        typer.echo(ctx.get_help())


_register()
