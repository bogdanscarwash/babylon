"""CLI app contract: entry-point shape, --version, and no-subcommand help
(ADR095 D1; Amendment AF / ADR186 — the play composition root and its
subcommand were deleted with the Ratatui client, so the CLI's
no-subcommand default changed from "launch the game" to "print help")."""

from __future__ import annotations

from typer.testing import CliRunner

from babylon import __version__
from babylon.cli import app

runner = CliRunner()


def test_help_lists_supported_operator_subcommands() -> None:
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    for name in ("doctor", "login", "telemetry", "uninstall"):
        assert name in result.stdout
    assert "self-update" not in result.stdout


def test_help_no_longer_lists_play() -> None:
    """`play` retired with the Ratatui client (AF iii/iv) — the shipped game
    is a standalone Bevy binary, not a subcommand of this CLI."""
    result = runner.invoke(app, ["--help"])
    assert result.exit_code == 0
    assert "play" not in result.stdout


def test_version_flag_prints_version() -> None:
    result = runner.invoke(app, ["--version"])
    assert result.exit_code == 0
    assert __version__ in result.stdout


def test_no_subcommand_prints_help() -> None:
    result = runner.invoke(app, [])
    assert result.exit_code == 0
    assert "Babylon" in result.stdout
    assert "doctor" in result.stdout
