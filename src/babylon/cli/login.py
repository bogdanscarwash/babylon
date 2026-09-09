"""`babylon login` — write the Cloudflare credentials file (mode 0600).

The credentials file is the §A3 secret jurisdiction, read only by the
intelligence lane; the engine never sees it. Written 0600 from creation so
there is never a window where it is group/world-readable.
"""

from __future__ import annotations

import os

import typer

from babylon.intelligence.providers import _config_dir


def login(
    api_key: str = typer.Option(
        ...,
        "--api-key",
        prompt="Cloudflare Babylon beta key",
        hide_input=True,
        help="Beta key for the babylon-api Worker (issued by seed-beta-key.sh).",
    ),
) -> None:
    """Write ~/.config/babylon/credentials with the Cloudflare beta key (0600)."""
    cfg_dir = _config_dir(os.environ)
    cfg_dir.mkdir(parents=True, exist_ok=True)
    creds = cfg_dir / "credentials"
    content = f'[cloudflare]\napi_key = "{api_key}"\n'

    # O_CREAT with 0600, then chmod to defeat any inherited umask widening.
    fd = os.open(creds, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as fh:
        fh.write(content)
    os.chmod(creds, 0o600)
    typer.echo(f"Wrote {creds} (mode 0600).")
