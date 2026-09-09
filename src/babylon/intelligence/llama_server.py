"""Bundled llama-server supervisor (D1, ADR096).

Lifecycle of the bundled CPU llama.cpp server as a child of the babylon
process (the D1 embedded-Postgres pattern): loopback HTTP only, one binary
serving /v1/chat/completions AND /v1/embeddings from sha256-pinned local
gguf weights. The binary is bundled by the closure (ADR094); until then it is
resolved from ``BABYLON_LLAMA_SERVER_BIN`` or PATH, with a loud
``ProviderUnavailable`` if absent. Weights come from the provisioned models
dir, never the store.

No LLM framework (X.6): this is plain ``subprocess`` + an HTTP health poll.
"""

from __future__ import annotations

import logging
import shutil
import subprocess
import time
import urllib.error
import urllib.request
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path
from types import TracebackType

from babylon.intelligence.providers import IntelligenceSettings, ProviderUnavailable

logger = logging.getLogger("babylon.intelligence.llama_server")

_HEALTH_PROBE_TIMEOUT_S: float = 1.0


def resolve_llama_binary(env: Mapping[str, str]) -> Path:
    """Resolve the llama-server binary: env override, then PATH.

    Raises:
        ProviderUnavailable: no binary found — the bundled lane is
            unavailable (caller degrades to detected-external → mute).
    """
    override = env.get("BABYLON_LLAMA_SERVER_BIN", "")
    if override:
        return Path(override)
    # shutil.which honors the provided PATH mapping when passed explicitly.
    found = shutil.which("llama-server", path=env.get("PATH"))
    if found is None:
        raise ProviderUnavailable(
            "llama-server not found (set BABYLON_LLAMA_SERVER_BIN or install the "
            "bundled closure); bundled inference lane unavailable"
        )
    return Path(found)


class LlamaServerSupervisor:
    """Owns a llama-server child process; loopback only, context-managed."""

    def __init__(
        self,
        binary: Path,
        chat_gguf: Path,
        embed_gguf: Path,
        *,
        host: str = "127.0.0.1",
        port: int = 8737,
        startup_timeout_s: float = 20.0,
        poll_interval_s: float = 0.1,
        extra_argv: Sequence[str] | None = None,
    ) -> None:
        self._binary = binary
        self._chat_gguf = chat_gguf
        self._embed_gguf = embed_gguf
        self._host = host
        self._port = port
        self._startup_timeout_s = startup_timeout_s
        self._poll_interval_s = poll_interval_s
        self._extra_argv = list(extra_argv) if extra_argv is not None else []
        self._process: subprocess.Popen[bytes] | None = None

    @property
    def _health_url(self) -> str:
        return f"http://{self._host}:{self._port}/health"

    def _argv(self) -> list[str]:
        # Faithful to llama-server flags; extra_argv lets tests substitute a
        # fake executable (python <fake>) ahead of the shared host/port flags.
        return [
            str(self._binary),
            *self._extra_argv,
            "--host",
            self._host,
            "--port",
            str(self._port),
            "-m",
            str(self._chat_gguf),
            "--embeddings",
            "-m",
            str(self._embed_gguf),
        ]

    def start(self) -> None:
        """Spawn the child and block until /health answers or timeout.

        Raises:
            ProviderUnavailable: the server did not become healthy in time.
        """
        if self._process is not None:
            return
        try:
            self._process = subprocess.Popen(self._argv())  # noqa: S603 - argv is fully controlled
        except OSError as exc:
            raise ProviderUnavailable(f"failed to spawn llama-server: {exc}") from exc

        deadline = time.monotonic() + self._startup_timeout_s
        # Bounded: the loop cannot exceed startup_timeout_s / poll_interval_s.
        max_polls = int(self._startup_timeout_s / self._poll_interval_s) + 1
        for _ in range(max_polls):
            if self._process.poll() is not None:
                raise ProviderUnavailable(
                    f"llama-server exited early (code {self._process.returncode})"
                )
            if self._probe_health():
                return
            if time.monotonic() >= deadline:
                break
            time.sleep(self._poll_interval_s)
        self.close()
        raise ProviderUnavailable(
            f"llama-server did not become healthy within {self._startup_timeout_s}s"
        )

    def _probe_health(self) -> bool:
        try:
            with urllib.request.urlopen(  # noqa: S310 - fixed loopback URL
                self._health_url, timeout=_HEALTH_PROBE_TIMEOUT_S
            ) as response:
                return bool(200 <= response.status < 300)
        except (urllib.error.URLError, OSError):
            return False

    def health_ok(self) -> bool:
        """True iff /health currently answers 2xx."""
        return self._probe_health()

    def close(self) -> None:
        """Terminate, then kill if it will not go."""
        process = self._process
        if process is None:
            return
        self._process = None
        if process.poll() is not None:
            return
        process.terminate()
        try:
            process.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5.0)

    def __enter__(self) -> LlamaServerSupervisor:
        self.start()
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        self.close()


SupervisorFactory = Callable[..., LlamaServerSupervisor]

_BUNDLED_CHAT_WEIGHT = "babylon-chat.gguf"
_BUNDLED_EMBED_WEIGHT = "babylon-embed.gguf"


def ensure_bundled_running(
    settings: IntelligenceSettings,
    *,
    models_dir: Path,
    env: Mapping[str, str] | None = None,
    supervisor_factory: SupervisorFactory | None = None,
) -> LlamaServerSupervisor | None:
    """Start the bundled server iff the bundled lane is selectable and ready.

    Called by the session bootstrap BEFORE ``resolve_provider``. Returns the
    running supervisor (caller owns ``.close()``), or ``None`` when the bundled
    lane is not selected/ready — in which case precedence falls through to
    detected-external → cloudflare → mute untouched. Never raises: a missing
    binary or a startup failure degrades quietly (loud in the log), because
    mute is always legal (Amendment V, R4).
    """
    import os

    env = os.environ if env is None else env
    if settings.mode not in {"auto", "bundled"}:
        return None

    chat = models_dir / _BUNDLED_CHAT_WEIGHT
    embed = models_dir / _BUNDLED_EMBED_WEIGHT
    if not (chat.exists() and embed.exists()):
        logger.info(
            "bundled lane: weights not provisioned in %s — run `babylon doctor --provision`",
            models_dir,
        )
        return None

    try:
        binary = resolve_llama_binary(env)
    except ProviderUnavailable as exc:
        logger.info("bundled lane unavailable: %s", exc)
        return None

    factory = supervisor_factory or (
        lambda **kwargs: LlamaServerSupervisor(**kwargs)  # noqa: E731 - trivial adapter
    )
    supervisor = factory(binary=binary, chat_gguf=chat, embed_gguf=embed)
    try:
        supervisor.start()
    except ProviderUnavailable as exc:
        logger.warning("bundled llama-server failed to start: %s — degrading", exc)
        supervisor.close()
        return None
    return supervisor
