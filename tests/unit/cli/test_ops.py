"""Operator status and native source uninstall guidance perform no mutation."""

from __future__ import annotations

from typer.testing import CliRunner

from babylon.cli import app

runner = CliRunner()


def test_retired_self_update_command_is_unavailable() -> None:
    result = runner.invoke(app, ["self-update"])
    assert result.exit_code == 2


def test_uninstall_prints_steps_deletes_nothing() -> None:
    result = runner.invoke(app, ["uninstall"])
    assert result.exit_code == 0
    assert "source checkout" in result.stdout
    assert "deletes nothing" in result.stdout
    assert "nix" not in result.stdout.lower()
    assert ".config/babylon" in result.stdout
