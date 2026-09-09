"""`babylon login` writes the credentials file at mode 0600 (ADR095 D1)."""

from __future__ import annotations

import stat
import tomllib

from typer.testing import CliRunner

from babylon.cli import app

runner = CliRunner()


def test_login_writes_0600_credentials(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
    monkeypatch.setenv("BABYLON_CONFIG_DIR", str(tmp_path))
    result = runner.invoke(app, ["login", "--api-key", "bk_test_123"])
    assert result.exit_code == 0
    creds = tmp_path / "credentials"
    assert creds.exists()
    mode = stat.S_IMODE(creds.stat().st_mode)
    assert mode == 0o600, f"expected 0600, got {mode:o}"
    data = tomllib.loads(creds.read_text())
    assert data["cloudflare"]["api_key"] == "bk_test_123"


def test_login_creates_missing_config_dir(monkeypatch, tmp_path) -> None:  # type: ignore[no-untyped-def]
    nested = tmp_path / "deep" / "babylon"
    monkeypatch.setenv("BABYLON_CONFIG_DIR", str(nested))
    result = runner.invoke(app, ["login", "--api-key", "bk_x"])
    assert result.exit_code == 0
    assert (nested / "credentials").exists()
