"""Report operator configuration, provider availability, and database reachability."""

from __future__ import annotations

import os

import typer
from rich.console import Console

from babylon.intelligence.model_manifest import load_bundled_manifest
from babylon.intelligence.providers import (
    ProviderError,
    _config_dir,
    load_settings,
    resolve_provider,
)
from babylon.intelligence.provision import default_models_dir, provision_models

#: soft_wrap avoids Rich's default 80-column word-wrap splitting long config
#: paths (e.g. deep tmp_path fixtures, nested XDG dirs) across lines; disabling
#: the ReprHighlighter (highlight=False) stops it from injecting a colour
#: escape code mid-path (directory vs. filename). Both would otherwise break
#: substring assertions on captured CliRunner stdout.
console = Console(soft_wrap=True, highlight=False)


def check_database(dsn: str | None) -> tuple[bool, str]:
    """Best-effort reachability probe. Never raises: an unreachable DB is a
    reported condition, not a doctor crash (III.11 loud-but-degrade)."""
    if not dsn:
        return (
            False,
            "no DSN configured (set BABYLON_RUNTIME_DSN)",
        )
    try:
        import psycopg

        with psycopg.connect(dsn, connect_timeout=2) as conn:
            conn.execute("SELECT 1")
    except ImportError as exc:
        return (False, f"psycopg not installed: {exc}")
    except psycopg.Error as exc:
        return (False, f"unreachable: {exc}")
    return (True, "reachable")


def doctor(
    provision: bool = typer.Option(
        False,
        "--provision",
        help="Fetch available model weights per the signed manifest (D3, ADR096).",
    ),
) -> None:
    """Diagnose the local Babylon install (config, provider lane, database,
    model provisioning)."""
    cfg_dir = _config_dir(os.environ)
    config_toml = cfg_dir / "config.toml"
    credentials = cfg_dir / "credentials"

    console.print(f"[bold]config dir:[/bold] {cfg_dir}")
    console.print(
        f"  config.toml: {'present' if config_toml.exists() else 'absent (defaults apply)'}"
    )
    console.print(
        f"  credentials: {'present' if credentials.exists() else 'absent (mute/local only)'}"
    )

    try:
        settings = load_settings()
        console.print(f"[bold]intelligence mode:[/bold] {settings.mode}")
    except ProviderError as exc:
        console.print(f"[bold red]config error:[/bold red] {exc}")

    provider = resolve_provider()
    health = provider.health()
    lane = provider.endpoint.kind.value
    mark = "ok" if health.ok else "degraded"
    console.print(f"[bold]provider lane:[/bold] {lane} ({mark}: {health.detail})")

    db_ok, db_detail = check_database(os.environ.get("BABYLON_RUNTIME_DSN"))
    console.print(f"[bold]database:[/bold] {'ok' if db_ok else 'unavailable'} — {db_detail}")

    if provision:
        console.print("[bold]provisioning models:[/bold]")
        try:
            results = provision_models(load_bundled_manifest(), default_models_dir())
        except ValueError as exc:
            console.print(f"[bold red]provisioning error:[/bold red] {exc}")
            raise typer.Exit(code=1) from exc
        for result in results:
            style = "yellow" if result.status == "gated" else "green"
            console.print(f"  [{style}]{result.name}: {result.status}[/{style}] — {result.detail}")

    raise typer.Exit(code=0)
