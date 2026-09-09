"""Behavioral contract for the bundled llama-server supervisor (D1, ADR096).

Loopback only, no real model: a fake executable answers /health. Pins process
lifecycle — start → health → close — and the loud failure when the binary is
absent. Uses an ephemeral port to avoid clashing with a real 8737 listener.
"""

from __future__ import annotations

import socket
import sys
from pathlib import Path

import pytest

from babylon.intelligence.llama_server import (
    LlamaServerSupervisor,
    resolve_llama_binary,
)
from babylon.intelligence.providers import ProviderUnavailable

_FAKE = Path(__file__).parent / "_fake_llama_server.py"


def _free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _supervisor(port: int, tmp_path: Path) -> LlamaServerSupervisor:
    # The fake ignores the gguf paths; touch real files so path validation (if
    # any) is satisfied and the contract stays honest.
    chat = tmp_path / "chat.gguf"
    embed = tmp_path / "embed.gguf"
    chat.write_bytes(b"x")
    embed.write_bytes(b"x")
    return LlamaServerSupervisor(
        binary=Path(sys.executable),
        chat_gguf=chat,
        embed_gguf=embed,
        host="127.0.0.1",
        port=port,
        startup_timeout_s=20.0,
        extra_argv=[str(_FAKE)],  # python <fake> --host --port ...
    )


def test_resolve_binary_missing_raises_provider_unavailable() -> None:
    with pytest.raises(ProviderUnavailable):
        resolve_llama_binary({"PATH": "/nonexistent", "BABYLON_LLAMA_SERVER_BIN": ""})


def test_resolve_binary_from_env_override(tmp_path: Path) -> None:
    fake_bin = tmp_path / "llama-server"
    fake_bin.write_text("#!/bin/sh\n")
    fake_bin.chmod(0o755)
    got = resolve_llama_binary({"BABYLON_LLAMA_SERVER_BIN": str(fake_bin)})
    assert got == fake_bin


def test_start_reaches_health_then_close(tmp_path: Path) -> None:
    port = _free_port()
    supervisor = _supervisor(port, tmp_path)
    supervisor.start()
    try:
        assert supervisor.health_ok() is True
    finally:
        supervisor.close()
    # After close the process is gone; health must fail.
    assert supervisor.health_ok() is False


def test_context_manager_starts_and_stops(tmp_path: Path) -> None:
    port = _free_port()
    with _supervisor(port, tmp_path) as supervisor:
        assert supervisor.health_ok() is True
    assert supervisor.health_ok() is False


# --- ensure_bundled_running (D1 lazy start) ---------------------------------

import sys as _sys  # noqa: E402 - grouped with the lazy-start tests

from babylon.intelligence.llama_server import ensure_bundled_running  # noqa: E402
from babylon.intelligence.providers import IntelligenceSettings  # noqa: E402


def _factory_using_fake(port: int):
    def factory(binary, chat_gguf, embed_gguf, *, host, port_, **_):  # noqa: ANN001
        return LlamaServerSupervisor(
            binary=Path(_sys.executable),
            chat_gguf=chat_gguf,
            embed_gguf=embed_gguf,
            host=host,
            port=port_,
            extra_argv=[str(_FAKE)],
        )

    return factory


def test_ensure_skips_when_mode_is_mute(tmp_path: Path) -> None:
    settings = IntelligenceSettings(mode="mute")
    got = ensure_bundled_running(settings, models_dir=tmp_path, env={})
    assert got is None


def test_ensure_skips_when_weights_absent(tmp_path: Path) -> None:
    # Binary resolvable but the models dir has no gguf → fall through, no start.
    settings = IntelligenceSettings(mode="auto")
    env = {"BABYLON_LLAMA_SERVER_BIN": _sys.executable}
    got = ensure_bundled_running(settings, models_dir=tmp_path, env=env)
    assert got is None


def test_ensure_skips_when_binary_absent(tmp_path: Path) -> None:
    (tmp_path / "babylon-chat.gguf").write_bytes(b"x")
    (tmp_path / "babylon-embed.gguf").write_bytes(b"x")
    settings = IntelligenceSettings(mode="bundled")
    env = {"PATH": "/nonexistent", "BABYLON_LLAMA_SERVER_BIN": ""}
    got = ensure_bundled_running(settings, models_dir=tmp_path, env=env)
    assert got is None  # loud ProviderUnavailable swallowed → degrade, not crash
