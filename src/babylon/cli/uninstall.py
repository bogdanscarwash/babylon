"""`babylon uninstall` — print the honest teardown (deletes nothing)."""

from __future__ import annotations

import typer


def uninstall() -> None:
    """Print the manual teardown steps. This command removes NOTHING itself."""
    typer.echo("Babylon uninstall — manual teardown (this command deletes nothing):")
    typer.echo("  1. Remove your source checkout after preserving its local changes and campaigns.")
    typer.echo("  2. Review ~/.local/share/babylon ($XDG_DATA_HOME/babylon): models and game data.")
    typer.echo("  3. Review ~/.config/babylon ($XDG_CONFIG_HOME/babylon): config and credentials.")
    typer.echo("  Remove only the data and configuration you intend to discard.")
