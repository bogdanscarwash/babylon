"""The provider seam — one set of calls for local and Cloudflare (§A8).

This module is the *entire* network surface of the intelligence layer. One
protocol (`NarratorProvider`), one OpenAI-compatible implementation
parameterized by ``(base_url, credential, model pin, timeout)``, and a
resolver that walks the §A7.6 order of precedence:

    bundled llama-server  →  detected external (Ollama et al.)
                          →  Cloudflare if keyed  →  mute

Provider commitments, preserved by ADR252 native installation:

* **The seam is the wire protocol.** llama-server, Ollama, and the
  ``babylon-api`` Worker all speak OpenAI-compatible ``/v1``. Generation
  runs on **pydantic-ai** (Amendment Y, ADR100 — BD-ruled to pass the X.6
  solo-developer filter: maintained upstream replacing hand-rolled
  transport); embeddings and health probes stay on the stock ``openai``
  client (pydantic-ai carries no embedding API). No litellm, no
  langchain — X.6 still bars frameworks that add maintenance rather than
  remove it.
* **Mute is always legal** (R4). The game is fully playable and fully
  informative silent; nothing here is ever load-bearing for the sim.
* **AI observes and narrates; the engine adjudicates** (II.5). This module
  has no free-text-to-verb parse lane: Amendment V (ratified 2026-07-20)
  rules there is no LLM in the input path (ruling R4) — player verbs enter
  only through the deterministic verb registry, and free text is flavor,
  never engine state. The drop this module originated from carried a
  ``parse()`` method (schema-validated verb JSON from free text) per the
  pre-Amendment-V ``nix-plan.md`` §A8 proposal; it was removed before this
  seam merged, per owner ruling, for Amendment V compliance.
* **Loud/quiet partition** (III.11): provider failures raise
  ``ProviderUnavailable`` — the intelligence layer catches, logs loudly, and
  degrades to mute quietly. The sim never blocks on the network.
* **Pins are law** (III.6/§3.4): prose caches key on ``(entity, tick,
  model_pin)``; embedding spaces are per-campaign — the campaign's pinned
  embed model must equal the model used here, enforced by the caller with
  the ``model_pin`` this module reports back on every result.
* **Config, not coefficients:** endpoints/timeouts live in
  ``~/.config/babylon/config.toml`` and env (§A3 precedence:
  wrapper-forced > explicit env > config.toml > defaults). Nothing here
  belongs in GameDefines — these are transport settings, not sim semantics.
  Credentials live apart in ``~/.config/babylon/credentials`` (0600), read
  only by this lane; the engine never sees them.

Wiring note: deliberately imports nothing from the rest of ``babylon`` so it
drops in ahead of the packaging train; the intelligence layer's narrator and
Archive embedder compose on top of these primitives.
"""

from __future__ import annotations

import logging
import os
import stat
import tomllib
from collections.abc import Callable, Mapping, Sequence
from enum import StrEnum
from pathlib import Path
from typing import Any, Protocol, runtime_checkable

from openai import AsyncOpenAI, OpenAI
from pydantic import BaseModel, ConfigDict, Field, SecretStr
from pydantic_ai import Agent
from pydantic_ai.models import Model
from pydantic_ai.models.openai import OpenAIChatModel
from pydantic_ai.providers.openai import OpenAIProvider
from pydantic_ai.settings import ModelSettings

logger = logging.getLogger("babylon.intelligence.providers")

__all__ = [
    "ProviderKind",
    "ProviderEndpoint",
    "IntelligenceSettings",
    "NarrationResult",
    "EmbeddingResult",
    "ProviderHealth",
    "ProviderError",
    "ProviderUnavailable",
    "NarratorProvider",
    "OpenAICompatProvider",
    "MuteProvider",
    "MockNarrator",
    "ModelFactory",
    "chat_model_for",
    "load_settings",
    "resolve_provider",
    "prose_cache_key",
]

# --------------------------------------------------------------------------
# Transport defaults. These are *defaults*, not policy: config.toml and env
# override per §A3. Not GameDefines material — no sim semantics live here.
# --------------------------------------------------------------------------

#: Bundled llama-server lane (§A7.3): loopback only, port owned by the
#: inference manager; the wrapper/env is the authority once the closure ships.
DEFAULT_BUNDLED_BASE_URL = "http://127.0.0.1:8737/v1"

#: Detected-external lane (D2): the game only ever auto-detects a local
#: Ollama/compatible server at its stock port — it never installs one.
DEFAULT_EXTERNAL_BASE_URL = "http://127.0.0.1:11434/v1"

DEFAULT_TIMEOUT_S = 30.0
PROBE_TIMEOUT_S = 2.0

#: Free-tier Workers AI pins served by the babylon-api front-door.
DEFAULT_CLOUDFLARE_CHAT_MODEL = "@cf/meta/llama-3.1-8b-instruct-fast"
DEFAULT_CLOUDFLARE_EMBED_MODEL = "@cf/baai/bge-m3"
DEFAULT_LOCAL_CHAT_MODEL = "local-chat"  # llama-server serves its loaded gguf under its own name
DEFAULT_LOCAL_EMBED_MODEL = "local-embed"


class ProviderKind(StrEnum):
    BUNDLED = "bundled"
    EXTERNAL = "external"
    CLOUDFLARE = "cloudflare"
    MUTE = "mute"
    #: Test/demo lane only — never produced by :func:`resolve_provider`.
    MOCK = "mock"


class ProviderError(RuntimeError):
    """Base class for the seam's failures."""


class ProviderUnavailable(ProviderError):
    """Transport/endpoint failure. Caller degrades to mute: quietly for the
    sim, loudly in the log (III.11 partition)."""


class ProviderEndpoint(BaseModel):
    """One lane, fully pinned."""

    model_config = ConfigDict(frozen=True)

    kind: ProviderKind
    base_url: str
    api_key: SecretStr | None = None
    chat_model: str
    embed_model: str
    timeout_s: float = Field(default=DEFAULT_TIMEOUT_S, gt=0)


class NarrationResult(BaseModel):
    model_config = ConfigDict(frozen=True)

    text: str
    model_pin: str
    provider: ProviderKind
    prompt_tokens: int | None = None
    completion_tokens: int | None = None


class EmbeddingResult(BaseModel):
    model_config = ConfigDict(frozen=True)

    vectors: tuple[tuple[float, ...], ...]
    model_pin: str
    provider: ProviderKind
    dimensions: int


class ProviderHealth(BaseModel):
    model_config = ConfigDict(frozen=True)

    ok: bool
    kind: ProviderKind
    detail: str = ""


@runtime_checkable
class NarratorProvider(Protocol):
    """§A8: narrate / embed / health. Presentation composes above; nothing
    below this protocol touches adjudication.

    No ``parse()``: Amendment V (ratified 2026-07-20, ruling R4) rules there
    is no LLM in the input path — free text is flavor only and never becomes
    engine state. Player verbs enter only through the deterministic verb
    registry.
    """

    endpoint: ProviderEndpoint

    def narrate(
        self, system: str, prompt: str, *, max_tokens: int = 512, temperature: float = 0.7
    ) -> NarrationResult: ...

    def embed(self, texts: Sequence[str]) -> EmbeddingResult: ...

    def health(self) -> ProviderHealth: ...


# --------------------------------------------------------------------------
# Settings (§A3 surface)
# --------------------------------------------------------------------------


class IntelligenceSettings(BaseModel):
    model_config = ConfigDict(frozen=True)

    mode: str = "auto"  # auto | bundled | external | cloudflare | mute
    timeout_s: float = Field(default=DEFAULT_TIMEOUT_S, gt=0)

    bundled_base_url: str = DEFAULT_BUNDLED_BASE_URL
    bundled_chat_model: str = DEFAULT_LOCAL_CHAT_MODEL
    bundled_embed_model: str = DEFAULT_LOCAL_EMBED_MODEL

    external_base_url: str = DEFAULT_EXTERNAL_BASE_URL
    external_chat_model: str = DEFAULT_LOCAL_CHAT_MODEL
    external_embed_model: str = DEFAULT_LOCAL_EMBED_MODEL

    cloudflare_base_url: str | None = None
    cloudflare_chat_model: str = DEFAULT_CLOUDFLARE_CHAT_MODEL
    cloudflare_embed_model: str = DEFAULT_CLOUDFLARE_EMBED_MODEL
    cloudflare_api_key: SecretStr | None = None


def _config_dir(env: Mapping[str, str]) -> Path:
    if "BABYLON_CONFIG_DIR" in env:
        return Path(env["BABYLON_CONFIG_DIR"])
    xdg = env.get("XDG_CONFIG_HOME")
    base = Path(xdg) if xdg else Path.home() / ".config"
    return base / "babylon"


def _read_toml(path: Path) -> dict[str, Any]:
    try:
        with path.open("rb") as fh:
            return tomllib.load(fh)
    except FileNotFoundError:
        return {}
    except (OSError, tomllib.TOMLDecodeError) as exc:
        # Loud: a config file that exists but cannot be read is remediable
        # operator error, never silently ignored (III.11).
        raise ProviderError(f"cannot read {path}: {exc}") from exc


def _load_credentials(path: Path) -> SecretStr | None:
    if not path.exists():
        return None
    mode = stat.S_IMODE(path.stat().st_mode)
    if mode & 0o077:
        # Warn loudly, still read: dev machines are messy, but §A3 says 0600.
        logger.warning(
            "credentials file %s has mode %o; expected 0600 — run: chmod 600 %s",
            path,
            mode,
            path,
        )
    data = _read_toml(path)
    key = data.get("cloudflare", {}).get("api_key")
    return SecretStr(key) if isinstance(key, str) and key else None


def load_settings(
    config_path: Path | None = None,
    env: Mapping[str, str] | None = None,
) -> IntelligenceSettings:
    """Assemble settings with §A3 precedence.

    ``wrapper-forced > explicit env > config.toml > defaults`` — the wrapper
    forces its pins *through* the environment, so env-over-file implements
    both of the top layers here.
    """
    env = os.environ if env is None else env
    cfg_dir = _config_dir(env)
    cfg = _read_toml(config_path if config_path is not None else cfg_dir / "config.toml")
    intel: dict[str, Any] = dict(cfg.get("intelligence", {}))

    credentials = _load_credentials(cfg_dir / "credentials")

    def pick(env_key: str, toml_key: str, default: Any) -> Any:
        if env_key in env and env[env_key] != "":
            return env[env_key]
        if toml_key in intel:
            return intel[toml_key]
        return default

    api_key_env = env.get("BABYLON_INTEL_CLOUDFLARE_KEY")
    api_key = SecretStr(api_key_env) if api_key_env else credentials

    return IntelligenceSettings(
        mode=str(pick("BABYLON_INTEL_MODE", "mode", "auto")).lower(),
        timeout_s=float(pick("BABYLON_INTEL_TIMEOUT_S", "timeout_s", DEFAULT_TIMEOUT_S)),
        bundled_base_url=str(
            pick("BABYLON_INTEL_BUNDLED_URL", "bundled_base_url", DEFAULT_BUNDLED_BASE_URL)
        ),
        bundled_chat_model=str(
            pick("BABYLON_INTEL_BUNDLED_CHAT_MODEL", "bundled_chat_model", DEFAULT_LOCAL_CHAT_MODEL)
        ),
        bundled_embed_model=str(
            pick(
                "BABYLON_INTEL_BUNDLED_EMBED_MODEL",
                "bundled_embed_model",
                DEFAULT_LOCAL_EMBED_MODEL,
            )
        ),
        external_base_url=str(
            pick("BABYLON_INTEL_EXTERNAL_URL", "external_base_url", DEFAULT_EXTERNAL_BASE_URL)
        ),
        external_chat_model=str(
            pick(
                "BABYLON_INTEL_EXTERNAL_CHAT_MODEL", "external_chat_model", DEFAULT_LOCAL_CHAT_MODEL
            )
        ),
        external_embed_model=str(
            pick(
                "BABYLON_INTEL_EXTERNAL_EMBED_MODEL",
                "external_embed_model",
                DEFAULT_LOCAL_EMBED_MODEL,
            )
        ),
        cloudflare_base_url=(
            str(pick("BABYLON_INTEL_CLOUDFLARE_URL", "cloudflare_base_url", "")) or None
        ),
        cloudflare_chat_model=str(
            pick(
                "BABYLON_INTEL_CLOUDFLARE_CHAT_MODEL",
                "cloudflare_chat_model",
                DEFAULT_CLOUDFLARE_CHAT_MODEL,
            )
        ),
        cloudflare_embed_model=str(
            pick(
                "BABYLON_INTEL_CLOUDFLARE_EMBED_MODEL",
                "cloudflare_embed_model",
                DEFAULT_CLOUDFLARE_EMBED_MODEL,
            )
        ),
        cloudflare_api_key=api_key,
    )


# --------------------------------------------------------------------------
# Providers
# --------------------------------------------------------------------------

ClientFactory = Callable[..., Any]
"""Signature-compatible with ``openai.OpenAI`` — injected for tests.

Carries the embed/health lanes only; narration runs on pydantic-ai via
:data:`ChatModelFactory`."""

ChatModelFactory = Callable[[ProviderEndpoint], Model]
"""Builds the pydantic-ai chat model for an endpoint — injected for tests
(``lambda ep: FunctionModel(...)``). Called fresh per ``narrate()`` so no
loop-bound connection pool outlives its event loop."""

ModelFactory = Callable[[], Model]
"""Zero-arg factory returning a fresh pydantic-ai ``Model``.

The seam for structured-output agents (the judge). A factory rather than a
``Model`` instance on purpose: each ``Agent.run_sync`` call runs in its own
event loop, so the underlying ``AsyncOpenAI``/httpx client must be rebuilt
per call to avoid stale loop-bound connection pools. Tests inject
``lambda: TestModel(...)``/``FunctionModel`` — returning the same in-memory
model each time is fine, test models never touch the network."""


def _default_client_factory(**kwargs: Any) -> Any:
    return OpenAI(**kwargs)


def chat_model_for(endpoint: ProviderEndpoint) -> Model:
    """A fresh pydantic-ai chat model for one lane (Amendment Y).

    Public so structured-output agents (the judge) can ride the resolved
    lane: ``chat_model_for(resolve_provider().endpoint)``. Builds a fresh
    ``AsyncOpenAI`` client per call — see :data:`ModelFactory`.
    """
    client = AsyncOpenAI(
        base_url=endpoint.base_url,
        # Local lanes are credential-free; the openai client demands a
        # string, so a sentinel stands in. Never a real secret.
        api_key=endpoint.api_key.get_secret_value() if endpoint.api_key else "babylon-local",
        timeout=endpoint.timeout_s,
        max_retries=1,
    )
    return OpenAIChatModel(endpoint.chat_model, provider=OpenAIProvider(openai_client=client))


class OpenAICompatProvider:
    """The one real implementation: any /v1-speaking endpoint, fully pinned."""

    def __init__(
        self,
        endpoint: ProviderEndpoint,
        client_factory: ClientFactory | None = None,
        chat_model_factory: ChatModelFactory | None = None,
    ) -> None:
        self.endpoint = endpoint
        factory = client_factory or _default_client_factory
        self._chat_model_factory = chat_model_factory or chat_model_for
        self._client = factory(
            base_url=endpoint.base_url,
            # Local lanes are credential-free; the openai client demands a
            # string, so a sentinel stands in. Never a real secret.
            api_key=endpoint.api_key.get_secret_value() if endpoint.api_key else "babylon-local",
            timeout=endpoint.timeout_s,
            max_retries=1,
        )

    # -- protocol ----------------------------------------------------------

    def narrate(
        self, system: str, prompt: str, *, max_tokens: int = 512, temperature: float = 0.7
    ) -> NarrationResult:
        agent: Agent[None, str] = Agent(
            self._chat_model_factory(self.endpoint), instructions=system
        )
        try:
            result = agent.run_sync(
                prompt,
                model_settings=ModelSettings(max_tokens=max_tokens, temperature=temperature),
            )
        except Exception as exc:  # transport, auth, quota, empty output — all one fate
            # Empty narration arrives here too: pydantic-ai refuses empty
            # model output (Loud Failure III.11 — nothing fabricated).
            raise ProviderUnavailable(f"{self.endpoint.kind} narrate failed: {exc}") from exc

        usage = result.usage
        return NarrationResult(
            text=result.output,
            model_pin=self.endpoint.chat_model,
            provider=self.endpoint.kind,
            prompt_tokens=usage.input_tokens,
            completion_tokens=usage.output_tokens,
        )

    def embed(self, texts: Sequence[str]) -> EmbeddingResult:
        if not texts:
            raise ProviderError("embed() requires at least one text")
        try:
            response = self._client.embeddings.create(
                model=self.endpoint.embed_model,
                input=list(texts),
            )
        except Exception as exc:
            raise ProviderUnavailable(f"{self.endpoint.kind} embed failed: {exc}") from exc
        vectors = tuple(tuple(float(x) for x in item.embedding) for item in response.data)
        if not vectors or not vectors[0]:
            raise ProviderUnavailable(f"{self.endpoint.kind} returned empty embeddings")
        return EmbeddingResult(
            vectors=vectors,
            model_pin=self.endpoint.embed_model,
            provider=self.endpoint.kind,
            dimensions=len(vectors[0]),
        )

    def health(self) -> ProviderHealth:
        try:
            self._client.with_options(timeout=PROBE_TIMEOUT_S).models.list()
        except Exception as exc:
            return ProviderHealth(ok=False, kind=self.endpoint.kind, detail=str(exc))
        return ProviderHealth(ok=True, kind=self.endpoint.kind, detail=self.endpoint.base_url)


class MuteProvider:
    """R4 made executable: always legal, always available, never networked.

    * ``narrate`` returns empty prose attributed to the mute pin — the TUI
      renders silence; the record stays honest (nothing fabricated).
    * ``embed`` raises ``ProviderUnavailable``: Archive search degrades to
      lexical. The caller renders that degradation loudly.
    """

    def __init__(self) -> None:
        self.endpoint = ProviderEndpoint(
            kind=ProviderKind.MUTE,
            base_url="about:mute",
            chat_model="mute",
            embed_model="mute",
        )

    def narrate(
        self,
        system: str,  # noqa: ARG002 — NarratorProvider shape
        prompt: str,  # noqa: ARG002 — NarratorProvider shape
        *,
        max_tokens: int = 512,  # noqa: ARG002 — NarratorProvider shape
        temperature: float = 0.7,  # noqa: ARG002 — NarratorProvider shape
    ) -> NarrationResult:
        return NarrationResult(text="", model_pin="mute", provider=ProviderKind.MUTE)

    def embed(self, texts: Sequence[str]) -> EmbeddingResult:  # noqa: ARG002 — NarratorProvider shape
        raise ProviderUnavailable("mute lane cannot embed; Archive search degrades to lexical")

    def health(self) -> ProviderHealth:
        return ProviderHealth(ok=True, kind=ProviderKind.MUTE, detail="silence is always available")


class MockNarrator:
    """Deterministic scripted narrator for tests and demos (never networked).

    The NarratorProvider-shaped successor to the retired ``MockLLM``
    (ADR101): returns pre-configured texts in FIFO order, then a fixed
    default, and records every call. Never produced by
    :func:`resolve_provider` — tests and demo tools construct it directly.

    * ``narrate`` returns the next scripted text attributed to the ``mock``
      pin (:attr:`ProviderKind.MOCK`).
    * ``embed`` raises :class:`ProviderUnavailable` — nothing here embeds.
    * ``health`` is always ok.
    """

    def __init__(
        self,
        responses: list[str] | None = None,
        default_response: str = "Mock LLM response",
    ) -> None:
        self.endpoint = ProviderEndpoint(
            kind=ProviderKind.MOCK,
            base_url="about:mock",
            chat_model="mock",
            embed_model="mock",
        )
        self._responses: list[str] = list(responses) if responses else []
        self._default = default_response
        self._call_count = 0
        self._call_history: list[dict[str, Any]] = []

    @property
    def call_count(self) -> int:
        """Number of times narrate() was called."""
        return self._call_count

    @property
    def call_history(self) -> list[dict[str, Any]]:
        """History of all narrate() calls (a copy; keys: system, prompt,
        max_tokens, temperature)."""
        return list(self._call_history)

    def narrate(
        self, system: str, prompt: str, *, max_tokens: int = 512, temperature: float = 0.7
    ) -> NarrationResult:
        self._call_count += 1
        self._call_history.append(
            {
                "system": system,
                "prompt": prompt,
                "max_tokens": max_tokens,
                "temperature": temperature,
            }
        )
        text = self._responses.pop(0) if self._responses else self._default
        return NarrationResult(text=text, model_pin="mock", provider=ProviderKind.MOCK)

    def embed(self, texts: Sequence[str]) -> EmbeddingResult:  # noqa: ARG002 — NarratorProvider shape
        raise ProviderUnavailable("mock lane cannot embed")

    def health(self) -> ProviderHealth:
        return ProviderHealth(ok=True, kind=ProviderKind.MOCK, detail="scripted")


# --------------------------------------------------------------------------
# Resolution (§A7.6 precedence)
# --------------------------------------------------------------------------


def _candidates(settings: IntelligenceSettings) -> list[ProviderEndpoint]:
    lanes = [
        ProviderEndpoint(
            kind=ProviderKind.BUNDLED,
            base_url=settings.bundled_base_url,
            chat_model=settings.bundled_chat_model,
            embed_model=settings.bundled_embed_model,
            timeout_s=settings.timeout_s,
        ),
        ProviderEndpoint(
            kind=ProviderKind.EXTERNAL,
            base_url=settings.external_base_url,
            chat_model=settings.external_chat_model,
            embed_model=settings.external_embed_model,
            timeout_s=settings.timeout_s,
        ),
    ]
    if settings.cloudflare_base_url and settings.cloudflare_api_key:
        lanes.append(
            ProviderEndpoint(
                kind=ProviderKind.CLOUDFLARE,
                base_url=settings.cloudflare_base_url,
                api_key=settings.cloudflare_api_key,
                chat_model=settings.cloudflare_chat_model,
                embed_model=settings.cloudflare_embed_model,
                timeout_s=settings.timeout_s,
            )
        )
    return lanes


def resolve_provider(
    settings: IntelligenceSettings | None = None,
    client_factory: ClientFactory | None = None,
) -> NarratorProvider:
    """Walk bundled → external → cloudflare-if-keyed → mute; return the first
    lane whose health probe answers. Resolution happens once per session
    (doctor semantics: probe once, record, no mid-session lane switching —
    the caller owns that discipline; III.6 pins make switches safe anyway).
    """
    settings = settings if settings is not None else load_settings()

    if settings.mode == "mute":
        logger.info("intelligence lane: mute (forced by config)")
        return MuteProvider()

    wanted = settings.mode if settings.mode in {"bundled", "external", "cloudflare"} else None
    lanes = _candidates(settings)
    if wanted is not None:
        lanes = [ep for ep in lanes if ep.kind.value == wanted]
        if not lanes:
            # cloudflare requested but no key/url configured → the failure
            # names its remediation (III.11), then the game plays on, silent.
            logger.warning(
                "intelligence lane %r requested but not configured "
                "(missing base_url or credential); degrading to mute",
                wanted,
            )
            return MuteProvider()

    for endpoint in lanes:
        provider = OpenAICompatProvider(endpoint, client_factory=client_factory)
        health = provider.health()
        if health.ok:
            logger.info(
                "intelligence lane: %s (%s | chat=%s embed=%s)",
                endpoint.kind.value,
                endpoint.base_url,
                endpoint.chat_model,
                endpoint.embed_model,
            )
            return provider
        logger.debug("lane %s unavailable: %s", endpoint.kind.value, health.detail)

    logger.warning("no intelligence lane reachable; running mute (fully playable, R4)")
    return MuteProvider()


# --------------------------------------------------------------------------
# III.6 helper
# --------------------------------------------------------------------------


def prose_cache_key(entity_id: str, tick: int, model_pin: str) -> str:
    """The canonical narrator-cache key: switching providers can never corrupt
    the record — it just writes new attributed blocks (III.6)."""
    return f"{entity_id}:{tick}:{model_pin}"
