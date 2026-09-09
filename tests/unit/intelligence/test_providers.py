"""Behavioral contract for the provider seam (§A8, §A7.6, §A3).

No network anywhere: the openai client is replaced by fakes through the
``client_factory`` injection point. These tests pin what the seam *does* —
precedence order, mute degradation, pin reporting — per Amendment Q's
spirit: contracts over construction.

No ``parse()`` lane: Amendment V (ratified 2026-07-20, ruling R4) rules
there is no LLM in the input path, so the seam carries no free-text-to-verb
translation and this file carries no tests for one.
"""

from __future__ import annotations

import os
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest
from pydantic_ai.messages import ModelMessage, ModelResponse, TextPart
from pydantic_ai.models.function import AgentInfo, FunctionModel
from pydantic_ai.usage import RequestUsage

from babylon.intelligence.providers import (
    DEFAULT_BUNDLED_BASE_URL,
    DEFAULT_EXTERNAL_BASE_URL,
    IntelligenceSettings,
    MockNarrator,
    MuteProvider,
    NarratorProvider,
    OpenAICompatProvider,
    ProviderEndpoint,
    ProviderKind,
    ProviderUnavailable,
    load_settings,
    prose_cache_key,
    resolve_provider,
)

# ---------------------------------------------------------------------------
# Fakes standing in for openai.OpenAI
# ---------------------------------------------------------------------------


class FakeClient:
    """Just enough of the openai surface: chat.completions.create,
    embeddings.create, models.list, with_options."""

    def __init__(
        self,
        *,
        healthy: bool = True,
        chat_text: str = "prose",
        embed_dims: int = 4,
        usage: bool = True,
    ) -> None:
        self.healthy = healthy
        self.chat_text = chat_text
        self.embed_dims = embed_dims
        self.usage = usage
        self.chat_calls: list[dict[str, Any]] = []
        self.embed_calls: list[dict[str, Any]] = []

        outer = self

        class _Completions:
            def create(self, **kwargs: Any) -> Any:
                outer.chat_calls.append(kwargs)
                usage_ns = (
                    SimpleNamespace(prompt_tokens=7, completion_tokens=11, total_tokens=18)
                    if outer.usage
                    else None
                )
                return SimpleNamespace(
                    choices=[SimpleNamespace(message=SimpleNamespace(content=outer.chat_text))],
                    usage=usage_ns,
                )

        class _Embeddings:
            def create(self, **kwargs: Any) -> Any:
                outer.embed_calls.append(kwargs)
                n = len(kwargs["input"])
                return SimpleNamespace(
                    data=[
                        SimpleNamespace(embedding=[float(i)] * outer.embed_dims) for i in range(n)
                    ]
                )

        class _Models:
            def list(self) -> Any:
                if not outer.healthy:
                    raise ConnectionError("connection refused")
                return SimpleNamespace(data=[])

        self.chat = SimpleNamespace(completions=_Completions())
        self.embeddings = _Embeddings()
        self.models = _Models()

    def with_options(self, **_: Any) -> FakeClient:
        return self


def factory_for(clients: dict[str, FakeClient]):
    """Route by base_url so each lane gets its own fake."""

    def factory(*, base_url: str, **_: Any) -> FakeClient:
        try:
            return clients[base_url]
        except KeyError:  # unknown lane: behave like nothing is listening
            return FakeClient(healthy=False)

    return factory


def settings_with(**overrides: Any) -> IntelligenceSettings:
    base: dict[str, Any] = {"mode": "auto", "timeout_s": 5.0}
    base.update(overrides)
    return IntelligenceSettings(**base)


CF_URL = "https://babylon-api.example.workers.dev/v1"


# ---------------------------------------------------------------------------
# Settings (§A3)
# ---------------------------------------------------------------------------


def test_settings_defaults_are_local_first() -> None:
    s = load_settings(config_path=Path("/nonexistent/config.toml"), env={})
    assert s.mode == "auto"
    assert s.bundled_base_url == DEFAULT_BUNDLED_BASE_URL
    assert s.external_base_url == DEFAULT_EXTERNAL_BASE_URL
    assert s.cloudflare_base_url is None  # cloud is opt-in, never default (D2)
    assert s.cloudflare_api_key is None


def test_settings_config_toml_over_defaults(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text(
        f'[intelligence]\nmode = "external"\ntimeout_s = 9.5\ncloudflare_base_url = "{CF_URL}"\n'
    )
    s = load_settings(config_path=cfg, env={})
    assert s.mode == "external"
    assert s.timeout_s == 9.5
    assert s.cloudflare_base_url == CF_URL


def test_settings_env_over_config(tmp_path: Path) -> None:
    cfg = tmp_path / "config.toml"
    cfg.write_text('[intelligence]\nmode = "external"\n')
    s = load_settings(config_path=cfg, env={"BABYLON_INTEL_MODE": "mute"})
    assert s.mode == "mute"  # wrapper/env authority beats file (§A3)


def test_credentials_file_loads_key_and_warns_on_loose_perms(
    tmp_path: Path,
    caplog: pytest.LogCaptureFixture,
) -> None:
    cfg_dir = tmp_path / "babylon"
    cfg_dir.mkdir()
    creds = cfg_dir / "credentials"
    creds.write_text('[cloudflare]\napi_key = "bk_testtoken1234567890"\n')
    os.chmod(creds, 0o644)  # deliberately loose
    with caplog.at_level("WARNING", logger="babylon.intelligence.providers"):
        s = load_settings(
            config_path=cfg_dir / "config.toml",
            env={"BABYLON_CONFIG_DIR": str(cfg_dir)},
        )
    assert s.cloudflare_api_key is not None
    assert s.cloudflare_api_key.get_secret_value() == "bk_testtoken1234567890"
    assert any("0600" in r.message for r in caplog.records)  # loud, not fatal


def test_env_key_overrides_credentials_file(tmp_path: Path) -> None:
    cfg_dir = tmp_path / "babylon"
    cfg_dir.mkdir()
    (cfg_dir / "credentials").write_text('[cloudflare]\napi_key = "bk_from_file"\n')
    s = load_settings(
        config_path=cfg_dir / "config.toml",
        env={
            "BABYLON_CONFIG_DIR": str(cfg_dir),
            "BABYLON_INTEL_CLOUDFLARE_KEY": "bk_from_env",
        },
    )
    assert s.cloudflare_api_key is not None
    assert s.cloudflare_api_key.get_secret_value() == "bk_from_env"


# ---------------------------------------------------------------------------
# Resolution (§A7.6: bundled → external → cloudflare-if-keyed → mute)
# ---------------------------------------------------------------------------


def test_resolve_prefers_bundled_when_healthy() -> None:
    clients = {
        DEFAULT_BUNDLED_BASE_URL: FakeClient(healthy=True),
        DEFAULT_EXTERNAL_BASE_URL: FakeClient(healthy=True),
    }
    p = resolve_provider(settings_with(), client_factory=factory_for(clients))
    assert p.endpoint.kind is ProviderKind.BUNDLED


def test_resolve_falls_through_to_external() -> None:
    clients = {
        DEFAULT_BUNDLED_BASE_URL: FakeClient(healthy=False),
        DEFAULT_EXTERNAL_BASE_URL: FakeClient(healthy=True),
    }
    p = resolve_provider(settings_with(), client_factory=factory_for(clients))
    assert p.endpoint.kind is ProviderKind.EXTERNAL


def test_resolve_cloudflare_only_when_keyed() -> None:
    clients = {CF_URL: FakeClient(healthy=True)}
    # url configured but NO key → lane not even considered → mute
    p = resolve_provider(
        settings_with(cloudflare_base_url=CF_URL),
        client_factory=factory_for(clients),
    )
    assert isinstance(p, MuteProvider)
    # keyed → lane considered and wins over dead local lanes
    p2 = resolve_provider(
        settings_with(cloudflare_base_url=CF_URL, cloudflare_api_key="bk_x" * 6),
        client_factory=factory_for(clients),
    )
    assert p2.endpoint.kind is ProviderKind.CLOUDFLARE


def test_resolve_everything_dead_yields_mute_never_raises() -> None:
    p = resolve_provider(settings_with(), client_factory=factory_for({}))
    assert isinstance(p, MuteProvider)
    assert p.health().ok  # silence is always available


def test_mode_override_mute_skips_probing() -> None:
    def exploding_factory(**_: Any) -> FakeClient:  # pragma: no cover
        raise AssertionError("mute mode must not build clients")

    p = resolve_provider(settings_with(mode="mute"), client_factory=exploding_factory)
    assert isinstance(p, MuteProvider)


def test_mode_override_cloudflare_unconfigured_degrades_to_mute() -> None:
    p = resolve_provider(settings_with(mode="cloudflare"), client_factory=factory_for({}))
    assert isinstance(p, MuteProvider)


# ---------------------------------------------------------------------------
# OpenAICompatProvider behavior
# ---------------------------------------------------------------------------


def make_provider(
    client: FakeClient,
    kind: ProviderKind = ProviderKind.BUNDLED,
    chat_model: Any = None,
) -> OpenAICompatProvider:
    ep = ProviderEndpoint(
        kind=kind,
        base_url="http://x/v1",
        chat_model="chat-pin",
        embed_model="embed-pin",
        timeout_s=5.0,
    )
    return OpenAICompatProvider(
        ep,
        client_factory=lambda **_: client,
        chat_model_factory=(lambda _ep: chat_model) if chat_model is not None else None,
    )


def _narrate_model(text: str) -> FunctionModel:
    """A pydantic-ai model returning fixed prose with pinned token usage."""

    def fn(messages: list[ModelMessage], info: AgentInfo) -> ModelResponse:
        return ModelResponse(
            parts=[TextPart(text)],
            usage=RequestUsage(input_tokens=7, output_tokens=11),
        )

    return FunctionModel(fn)


def test_narrate_reports_pin_and_provider() -> None:
    model = _narrate_model("the metropole trembles")
    r = make_provider(FakeClient(), chat_model=model).narrate("sys", "prompt")
    assert r.text == "the metropole trembles"
    assert r.model_pin == "chat-pin"  # III.6 cache key material
    assert r.provider is ProviderKind.BUNDLED
    assert r.prompt_tokens == 7 and r.completion_tokens == 11


def test_narrate_empty_output_is_loud_failure() -> None:
    with pytest.raises(ProviderUnavailable):
        make_provider(FakeClient(), chat_model=_narrate_model("")).narrate("sys", "prompt")


def test_narrate_passes_system_as_instructions() -> None:
    seen: dict[str, Any] = {}

    def fn(messages: list[ModelMessage], info: AgentInfo) -> ModelResponse:
        seen["instructions"] = info.instructions
        seen["max_tokens"] = (info.model_settings or {}).get("max_tokens")
        return ModelResponse(parts=[TextPart("prose")])

    make_provider(FakeClient(), chat_model=FunctionModel(fn)).narrate("sys", "prompt")
    assert seen["instructions"] == "sys"
    assert seen["max_tokens"] == 512


def test_embed_reports_dimensions_and_pin() -> None:
    r = make_provider(FakeClient(embed_dims=1024)).embed(["a", "b"])
    assert r.dimensions == 1024  # per-campaign pgvector pin discipline (§3.4)
    assert r.model_pin == "embed-pin"
    assert len(r.vectors) == 2


def test_transport_failure_is_provider_unavailable() -> None:
    def dead(messages: list[ModelMessage], info: AgentInfo) -> ModelResponse:
        raise TimeoutError("upstream gone")

    with pytest.raises(ProviderUnavailable):
        make_provider(FakeClient(), chat_model=FunctionModel(dead)).narrate("s", "p")


# ---------------------------------------------------------------------------
# Mute lane (R4)
# ---------------------------------------------------------------------------


def test_mute_narrates_silence_honestly() -> None:
    r = MuteProvider().narrate("s", "p")
    assert r.text == "" and r.model_pin == "mute"  # nothing fabricated, ever


def test_mute_embed_degrades_loudly() -> None:
    m = MuteProvider()
    with pytest.raises(ProviderUnavailable):
        m.embed(["x"])


# ---------------------------------------------------------------------------
# Mock lane (ADR101 — the scripted successor to the retired MockLLM)
# ---------------------------------------------------------------------------


def test_mock_narrator_is_narrator_provider() -> None:
    assert isinstance(MockNarrator(), NarratorProvider)


def test_mock_narrator_scripts_fifo_then_default() -> None:
    m = MockNarrator(responses=["First", "Second"], default_response="Default")
    assert m.narrate("s", "p1").text == "First"
    assert m.narrate("s", "p2").text == "Second"
    assert m.narrate("s", "p3").text == "Default"


def test_mock_narrator_records_calls() -> None:
    m = MockNarrator()
    m.narrate("system text", "prompt text", temperature=0.5)
    assert m.call_count == 1
    assert m.call_history[0]["system"] == "system text"
    assert m.call_history[0]["prompt"] == "prompt text"
    assert m.call_history[0]["temperature"] == 0.5


def test_mock_narrator_call_history_is_copy() -> None:
    m = MockNarrator()
    m.narrate("s", "p")
    history = m.call_history
    history.clear()
    assert len(m.call_history) == 1


def test_mock_narrator_reports_mock_pin() -> None:
    r = MockNarrator().narrate("s", "p")
    assert r.model_pin == "mock" and r.provider is ProviderKind.MOCK


def test_mock_narrator_embed_degrades_loudly() -> None:
    with pytest.raises(ProviderUnavailable):
        MockNarrator().embed(["x"])


def test_resolver_never_yields_mock() -> None:
    """The mock lane is test/demo-only: no settings mode reaches it."""
    p = resolve_provider(settings_with(mode="mock"), client_factory=factory_for({}))
    assert not isinstance(p, MockNarrator)


# ---------------------------------------------------------------------------
# Test-tier network guard (Amendment Y hygiene)
# ---------------------------------------------------------------------------


def test_model_requests_are_disallowed() -> None:
    """pydantic-ai's global request guard is off for the whole test tier."""
    from pydantic_ai import models

    assert models.ALLOW_MODEL_REQUESTS is False


# ---------------------------------------------------------------------------
# III.6 helper
# ---------------------------------------------------------------------------


def test_prose_cache_key_is_stable_and_pin_scoped() -> None:
    k1 = prose_cache_key("class:proletariat-ohio", 1312, "chat-pin")
    assert k1 == "class:proletariat-ohio:1312:chat-pin"
    assert k1 != prose_cache_key("class:proletariat-ohio", 1312, "other-pin")
