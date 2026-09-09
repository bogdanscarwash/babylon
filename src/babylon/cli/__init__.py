"""Operator commands for Babylon's reference data and model configuration."""

from __future__ import annotations

import typer

from babylon import __version__
from babylon.config.logging_config import setup_logging

setup_logging()

app = typer.Typer(
    name="babylon",
    help="Babylon operator tools. Launch the game with babylon-client.",
    add_completion=False,
    no_args_is_help=False,
)


def _register() -> None:
    """Register the operator subcommands."""
    from babylon.cli import doctor as doctor_cmd
    from babylon.cli import login as login_cmd
    from babylon.cli import uninstall as uninstall_cmd

    app.command(name="doctor")(doctor_cmd.doctor)
    app.command(name="login")(login_cmd.login)
    app.command(name="uninstall")(uninstall_cmd.uninstall)


def _version_callback(value: bool) -> None:
    if value:
        typer.echo(__version__)
        raise typer.Exit()


@app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    version: bool = typer.Option(  # noqa: ARG001 — consumed by the eager callback
        False,
        "--version",
        callback=_version_callback,
        is_eager=True,
        help="Show the Babylon tools version and exit.",
    ),
) -> None:
    """Print operator command help when no subcommand is selected."""
    if ctx.invoked_subcommand is None:
        typer.echo(ctx.get_help())


_register()
