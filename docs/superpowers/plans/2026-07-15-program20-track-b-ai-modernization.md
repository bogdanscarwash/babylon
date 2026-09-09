# Program 20 Track B — AI Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernize the narrator stack: versioned prompt artifacts with hash-derived versions, a
`WorkersAIClient` (Program 07 Decision 3) behind provider selection, persisted `NarrativeResult`
beats, and a real `GET /api/games/{id}/narration/` endpoint matching the cockpit's existing typed
contract — all flag-off per owner ruling D3.

**Architecture:** Everything hangs off the existing seams: `LLMProvider` protocol
(`intelligence/ai/llm_provider.py`), `NarrativeService` at the bridge boundary
(`web/game/narrative_service.py`), Django models in `web/game/models.py`, DRF-style views in
`web/game/api.py`. Narration stays out-of-tick (fire-and-forget post-`resolve_tick`) — tick hash
and `qa:regression` untouched.

**Tech Stack:** Python 3.12, Pydantic v2, openai SDK (Workers AI is OpenAI-compatible via AI
Gateway REST API), Django 5 + DRF, pytest (+pytest-django), TypeScript (endpoints registry).

## Global Constraints

- ALL subagents run on **Sonnet 5** (`model: 'sonnet'`).
- TDD: red → green per step; `@pytest.mark.red_phase` never skipped.
- Pytest from the worktree needs the venv shadow: prefix every run with `PYTHONPATH="$PWD/src"`.
- Heavy runs single-flight; scoped `mise run test:q -- <path>` over full suites.
- Flag-off behavior byte-identical: `BABYLON_LLM_NARRATOR` off ⇒ wire feed unchanged
  (existing parity tests in `tests/unit/web/test_narrative_service.py` must stay green).
- Constitution: II.5 (AI narrates, never adjudicates), III.6 (model/prompt pinning), III.11
  (loud failure — degraded ≠ silent), Mock Doctrine (unbuilt surfaces badge as MOCK).
- Model id for Workers AI: `@cf/openai/gpt-oss-20b` (verified 2026-07-15: live, function
  calling YES, 128k ctx). Chat base: `https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1`
  with header `cf-aig-gateway-id` (AI Gateway REST API, changelog 2026-05-21).
- **Branch base coordination:** `web/game/{api,urls,engine_bridge,stub_bridge}.py` are dirty on
  the owner's live spine branch. Execute Track B on a branch rebased onto
  `feature/epochs-wave1-spine` (or dev after it lands). If rebase conflicts appear in files this
  plan touches, STOP and report before resolving.
- Conventional commits with the `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer.

---

### Task B1: Prompt registry — versioned artifacts + hash-derived PROMPT_VERSION

**Files:**
- Create: `src/babylon/data/game/prompts/narrator/corporate_system.txt`,
  `liberated_system.txt`, `default_system.txt` (content MOVED VERBATIM from
  `src/babylon/intelligence/ai/director.py` lines ~44–73 `CORPORATE_SYSTEM_PROMPT`/
  `LIBERATED_SYSTEM_PROMPT` and `prompt_builder.py` `build_system_prompt()` default literal —
  copy exactly, do not retype from memory)
- Create: `src/babylon/intelligence/ai/prompt_registry.py`
- Modify: `src/babylon/intelligence/ai/director.py` (consume registry; delete literals)
- Modify: `src/babylon/intelligence/ai/prompt_builder.py` (default system prompt via registry)
- Modify: `web/game/narrative_service.py` (`PROMPT_VERSION` = registry version, delete constant)
- Test: `tests/unit/ai/test_prompt_registry.py`

**Interfaces:**
- Produces: `PromptRegistry.get(name: str) -> str`; `PromptRegistry.version() -> str`
  (format `"sha256:<first 12 hex>"` over all artifact bytes, sorted by name);
  module-level `get_prompt_registry() -> PromptRegistry` (cached singleton).

- [ ] **Step 1: Write the failing tests**

```python
"""Prompt registry: versioned narrator prompt artifacts (Constitution III.6/III.12)."""
from pathlib import Path

import pytest

from babylon.intelligence.ai.prompt_registry import PromptRegistry, get_prompt_registry


def test_registry_loads_known_prompts() -> None:
    reg = get_prompt_registry()
    for name in ("corporate_system", "liberated_system", "default_system"):
        text = reg.get(name)
        assert isinstance(text, str) and len(text) > 50


def test_unknown_prompt_fails_loud() -> None:
    reg = get_prompt_registry()
    with pytest.raises(KeyError):
        reg.get("does_not_exist")


def test_version_is_content_hash(tmp_path: Path) -> None:
    d = tmp_path / "prompts"
    d.mkdir()
    (d / "a.txt").write_text("alpha")
    reg1 = PromptRegistry(d)
    v1 = reg1.version()
    assert v1.startswith("sha256:") and len(v1) == len("sha256:") + 12
    (d / "a.txt").write_text("alpha CHANGED")
    assert PromptRegistry(d).version() != v1  # drift is impossible to hide


def test_director_prompts_come_from_registry() -> None:
    from babylon.intelligence.ai import director

    reg = get_prompt_registry()
    assert director.CORPORATE_SYSTEM_PROMPT == reg.get("corporate_system")
    assert director.LIBERATED_SYSTEM_PROMPT == reg.get("liberated_system")
```

- [ ] **Step 2: Run to verify failure**

`PYTHONPATH="$PWD/src" poetry run pytest tests/unit/ai/test_prompt_registry.py -v`
Expected: FAIL (`ModuleNotFoundError: prompt_registry`).

- [ ] **Step 3: Move the literals into artifacts** — copy the exact string bodies (strip the
Python quoting only) into the three `.txt` files.

- [ ] **Step 4: Implement `prompt_registry.py`**

```python
"""Versioned narrator prompt artifacts.

Prompts are data, not code (Constitution III.12 — durable spec artifacts). The
registry version is derived from content bytes, so an edited prompt can never
ship with a stale version pin (closes the manual PROMPT_VERSION drift gap).
"""

from __future__ import annotations

import hashlib
from functools import lru_cache
from pathlib import Path
from typing import Final

_DEFAULT_DIR: Final[Path] = (
    Path(__file__).resolve().parents[2] / "data" / "game" / "prompts" / "narrator"
)


class PromptRegistry:
    """Load-once registry of narrator prompt artifacts.

    :param root: directory containing ``*.txt`` prompt artifacts.
    :raises FileNotFoundError: if the directory is missing (loud, III.11).
    """

    def __init__(self, root: Path | None = None) -> None:
        self._root: Final[Path] = root or _DEFAULT_DIR
        if not self._root.is_dir():
            raise FileNotFoundError(f"Prompt artifact dir missing: {self._root}")
        self._prompts: Final[dict[str, str]] = {
            p.stem: p.read_text(encoding="utf-8") for p in sorted(self._root.glob("*.txt"))
        }
        if not self._prompts:
            raise FileNotFoundError(f"No prompt artifacts in {self._root}")

    def get(self, name: str) -> str:
        """Return prompt text by artifact stem; KeyError on unknown name."""
        return self._prompts[name]

    def version(self) -> str:
        """Content-derived version: sha256 over (name, bytes) pairs, sorted."""
        h = hashlib.sha256()
        for name in sorted(self._prompts):
            h.update(name.encode("utf-8"))
            h.update(b"\x00")
            h.update(self._prompts[name].encode("utf-8"))
        return f"sha256:{h.hexdigest()[:12]}"


@lru_cache(maxsize=1)
def get_prompt_registry() -> PromptRegistry:
    """Process-wide registry over the default artifact directory."""
    return PromptRegistry()
```

- [ ] **Step 5: Rewire consumers.** In `director.py`: replace the two literal assignments with
`CORPORATE_SYSTEM_PROMPT = get_prompt_registry().get("corporate_system")` (same for liberated);
in `prompt_builder.py` `build_system_prompt()`: default branch returns
`get_prompt_registry().get("default_system")`; in `narrative_service.py`: replace
`PROMPT_VERSION = "v1"` with `PROMPT_VERSION = get_prompt_registry().version()` and update its
comment (auto-derived; manual bumps retired). Keep the module attribute name — tests assert it.

- [ ] **Step 6: Green + targeted regression**

```bash
PYTHONPATH="$PWD/src" poetry run pytest tests/unit/ai/test_prompt_registry.py tests/unit/ai/test_prompt_builder.py tests/unit/ai/test_narrative_director.py tests/unit/web/test_narrative_service.py -v
```
Expected: all PASS (the existing suites pin substrings, not the literal's home).

- [ ] **Step 7: Commit** — `feat(ai): prompt registry — narrator prompts as versioned artifacts, hash-derived PROMPT_VERSION`

---

### Task B1b: RIOT archetype registry — AI-fillable event templates

**Files:**
- Create: `src/babylon/data/game/prompts/archetypes/riot.json`, `unrest_wave.json`,
  `rupture.json` (first three archetypes), `archetype.schema.json`
- Modify: `src/babylon/intelligence/ai/prompt_registry.py` (add archetype loading)
- Modify: `src/babylon/intelligence/ai/prompt_builder.py` (`_build_rag_section` sibling: new
  `_build_archetype_section` folded into `build_context_block` when an event matches)
- Test: `tests/unit/ai/test_event_archetypes.py`

**Interfaces:**
- Produces: `EventArchetype` (frozen Pydantic model: `id: str`, `event_types: list[str]`,
  `slots: list[str]`, `guidance: str` — the AI-fillable structure, per the emergent-endgames
  ruling: structure without scripting); `PromptRegistry.archetype_for(event_type: str) ->
  EventArchetype | None`. Archetypes are pure JSON **data** so the web-layer
  DeterministicNarrator (which imports zero `babylon.*` by design) can read the same files
  directly later — this task wires the LLM path only; the deterministic path keeps its own
  templates until the Wire triptych work (X2) picks the JSON up.

- [ ] **Step 1: Failing tests** — schema validation (bad archetype JSON → loud
  `jsonschema.ValidationError` at load), `archetype_for("RIOT")` returns the riot archetype,
  unknown event type returns `None`, and `build_context_block` includes the guidance text +
  slot names when a matching typed event is passed (reuse `test_prompt_builder.py`'s event
  fixtures — read that file first).
- [ ] **Step 2: FAIL.** **Step 3:** write `archetype.schema.json` (object; required: id,
  event_types, slots, guidance; additionalProperties false) and the three archetype files —
  e.g. `riot.json`:

```json
{
  "id": "riot",
  "event_types": ["RIOT", "UNREST", "SPONTANEOUS_UPRISING"],
  "slots": ["location", "trigger_condition", "class_composition", "state_response", "aftermath"],
  "guidance": "Narrate an unplanned eruption. Fill every slot from the observed engine state only — never invent casualties, counts, or outcomes the tick data does not contain. The eruption has material causes; name them. No moralizing, no editorializing the player."
}
```

  (`unrest_wave.json`, `rupture.json` follow the same shape with slots fitting their event
  types — `rupture` maps to the Survival-Calculus rupture events: slots
  `["organization", "repression_ratio", "tipping_class", "territory", "immediate_consequence"]`.)
  Loader: `PromptRegistry` gains `_archetypes` loaded from `archetypes/*.json`, each validated
  against the schema at load (loud, III.11), indexed by every `event_types` entry;
  `version()` hashes archetype bytes too (they are prompt content).
- [ ] **Step 4:** prompt_builder integration: when `build_context_block` receives typed events,
  the first event whose type has an archetype adds a section
  `--- EVENT ARCHETYPE: {id} ---\n{guidance}\nSlots to fill: {', '.join(slots)}`.
- [ ] **Step 5: Green** (new tests + `test_prompt_builder.py` + `test_prompt_registry.py` —
  note `test_version_is_content_hash` must still pass with an empty archetypes dir in tmp_path:
  make archetype loading tolerate an ABSENT directory (data simply not present ≠ failure) but
  NEVER tolerate an invalid file (loud).)
- [ ] **Step 6: Commit** — `feat(ai): RIOT-style event archetypes — AI-fillable structure per emergent-endgames ruling`

---

### Task B2: LLMConfig — provider selection + Workers AI settings

**Files:**
- Modify: `src/babylon/config/llm_config.py`
- Modify: `.env.example` (add block; ALSO delete the stale `CHROMADB_PERSIST_DIR` line)
- Test: `tests/unit/config/test_llm_config_workers_ai.py`

**Interfaces:**
- Produces (class attrs on `LLMConfig`): `PROVIDER: str` (env `LLM_PROVIDER`, default
  `"deepseek"`); `WORKERS_AI_ACCOUNT_ID/TOKEN/MODEL/GATEWAY_ID: str`;
  `WORKERS_AI_TIMEOUT: float`; classmethods `is_workers_ai() -> bool`,
  `workers_ai_base_url() -> str`.

- [ ] **Step 1: Failing test**

```python
"""Workers AI config surface (program-20 Track B, Program 07 Decision 3)."""
import pytest

from babylon.config import LLMConfig


def test_defaults(monkeypatch: pytest.MonkeyPatch) -> None:
    assert LLMConfig.PROVIDER in {"deepseek", "workers_ai", "mock"}
    assert LLMConfig.WORKERS_AI_MODEL == "@cf/openai/gpt-oss-20b"
    assert LLMConfig.WORKERS_AI_GATEWAY_ID == "babylon-narrator"


def test_base_url_requires_account_id(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(LLMConfig, "WORKERS_AI_ACCOUNT_ID", "")
    with pytest.raises(ValueError):
        LLMConfig.workers_ai_base_url()


def test_base_url_shape(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(LLMConfig, "WORKERS_AI_ACCOUNT_ID", "acct123")
    assert (
        LLMConfig.workers_ai_base_url()
        == "https://api.cloudflare.com/client/v4/accounts/acct123/ai/v1"
    )
```

- [ ] **Step 2: Verify FAIL** (`AttributeError: PROVIDER`).

- [ ] **Step 3: Implement** — append to `LLMConfig` (mirror existing style: `Final`, env-driven):

```python
    # === Provider selection (program-20): deepseek | workers_ai | mock ===
    PROVIDER: Final[str] = os.getenv("LLM_PROVIDER", "deepseek")

    # === Cloudflare Workers AI via AI Gateway (Program 07 Decision 3) ===
    WORKERS_AI_ACCOUNT_ID: Final[str] = os.getenv("WORKERS_AI_ACCOUNT_ID", "")
    WORKERS_AI_TOKEN: Final[str] = os.getenv("WORKERS_AI_TOKEN", "")
    WORKERS_AI_MODEL: Final[str] = os.getenv("WORKERS_AI_MODEL", "@cf/openai/gpt-oss-20b")
    WORKERS_AI_GATEWAY_ID: Final[str] = os.getenv("WORKERS_AI_GATEWAY_ID", "babylon-narrator")
    WORKERS_AI_TIMEOUT: Final[float] = float(os.getenv("WORKERS_AI_TIMEOUT", "15.0"))

    @classmethod
    def is_workers_ai(cls) -> bool:
        """True when the selected chat provider is Cloudflare Workers AI."""
        return cls.PROVIDER.lower() == "workers_ai"

    @classmethod
    def workers_ai_base_url(cls) -> str:
        """OpenAI-compatible chat base URL through the AI Gateway (loud when unconfigured)."""
        if not cls.WORKERS_AI_ACCOUNT_ID:
            raise ValueError(
                "WORKERS_AI_ACCOUNT_ID not configured — required for LLM_PROVIDER=workers_ai."
            )
        return (
            "https://api.cloudflare.com/client/v4/accounts/"
            f"{cls.WORKERS_AI_ACCOUNT_ID}/ai/v1"
        )
```

`.env.example`: delete the `CHROMADB_PERSIST_DIR` line (pgvector replaced ChromaDB, spec-037);
add under the LLM section:

```bash
# --- Chat provider selection (program-20): deepseek | workers_ai | mock ---
LLM_PROVIDER=deepseek
# --- Cloudflare Workers AI narrator (Program 07 Decision 3; flag-off per D3) ---
# WORKERS_AI_ACCOUNT_ID=
# WORKERS_AI_TOKEN=
# WORKERS_AI_MODEL=@cf/openai/gpt-oss-20b
# WORKERS_AI_GATEWAY_ID=babylon-narrator
# WORKERS_AI_TIMEOUT=15.0
```

- [ ] **Step 4: Green** — same pytest command scoped to the new file, plus
`rg -n 'CHROMADB' src/ web/ .env.example` → only historical docs may remain; no code hits.

- [ ] **Step 5: Commit** — `feat(config): LLM provider selection + Workers AI settings; retire stale CHROMADB env`

---

### Task B3: WorkersAIClient

**Files:**
- Modify: `src/babylon/intelligence/ai/llm_provider.py` (add class + factory)
- Test: `tests/unit/ai/test_workers_ai_client.py`

**Interfaces:**
- Produces: `WorkersAIClient(config=None, client=None)` implementing `LLMProvider`
  (`.name == "WorkersAI"`, `.generate(prompt, system_prompt, temperature) -> str`);
  `build_llm_provider(config=None) -> LLMProvider` selecting on `LLMConfig.PROVIDER`.
- Consumes: `LLMConfig.workers_ai_base_url()`, `WORKERS_AI_TOKEN/MODEL/GATEWAY_ID/TIMEOUT` (B2).

- [ ] **Step 1: Failing tests** (inject a fake OpenAI client — same seam style as MockLLM tests
in `tests/unit/ai/test_llm_provider.py`; read that file's fixtures first and reuse its fake
response builder if one exists):

```python
"""WorkersAIClient: Workers AI via AI Gateway, OpenAI-compatible (program-20)."""
from types import SimpleNamespace
from typing import Any

import pytest

from babylon.config import LLMConfig
from babylon.intelligence.ai.llm_provider import WorkersAIClient, build_llm_provider
from babylon.kernel.exceptions import LLMGenerationError


class _FakeCompletions:
    def __init__(self, content: str | None) -> None:
        self._content = content
        self.last_kwargs: dict[str, Any] | None = None

    def create(self, **kwargs: Any) -> Any:
        self.last_kwargs = kwargs
        msg = SimpleNamespace(content=self._content)
        return SimpleNamespace(choices=[SimpleNamespace(message=msg)])


def _fake_client(content: str | None) -> Any:
    return SimpleNamespace(chat=SimpleNamespace(completions=_FakeCompletions(content)))


@pytest.fixture(autouse=True)
def _configured(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(LLMConfig, "WORKERS_AI_ACCOUNT_ID", "acct123")
    monkeypatch.setattr(LLMConfig, "WORKERS_AI_TOKEN", "tok")


def test_generate_returns_content() -> None:
    fake = _fake_client("narrated text")
    client = WorkersAIClient(client=fake)
    assert client.generate("prompt", system_prompt="sys") == "narrated text"
    kwargs = fake.chat.completions.last_kwargs
    assert kwargs["model"] == LLMConfig.WORKERS_AI_MODEL
    assert kwargs["messages"][0] == {"role": "system", "content": "sys"}


def test_empty_response_is_loud() -> None:
    client = WorkersAIClient(client=_fake_client(None))
    with pytest.raises(LLMGenerationError):
        client.generate("prompt")


def test_missing_token_is_loud(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(LLMConfig, "WORKERS_AI_TOKEN", "")
    with pytest.raises(LLMGenerationError):
        WorkersAIClient()


def test_factory_selects_provider(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(LLMConfig, "PROVIDER", "workers_ai")
    provider = build_llm_provider()
    assert provider.name == "WorkersAI"
    monkeypatch.setattr(LLMConfig, "PROVIDER", "mock")
    assert build_llm_provider().name == "MockLLM"
```

(`test_factory_selects_provider`'s `workers_ai` branch must construct WITHOUT a network call —
the factory passes no client; constructor validates config only. If constructing the real
`OpenAI` object requires an api_key, the token check precedes construction, so `tok` suffices.)

- [ ] **Step 2: Verify FAIL** (ImportError).

- [ ] **Step 3: Implement** in `llm_provider.py` after `DeepSeekClient` (mirror its shape —
same error taxonomy LLM_001/002/003, same sync API):

```python
class WorkersAIClient:
    """Cloudflare Workers AI client via AI Gateway (OpenAI-compatible REST).

    Program 07 Decision 3 (owner, 2026-07-03): the narrator runs on Workers AI.
    Requests route through the babylon-narrator AI Gateway for logging/rate
    limiting (``cf-aig-gateway-id`` header). Model: ``@cf/openai/gpt-oss-20b``.
    """

    def __init__(
        self,
        config: type[LLMConfig] | None = None,
        client: OpenAI | None = None,
    ) -> None:
        self._config = config or LLMConfig
        self._name: Final[str] = "WorkersAI"

        if not self._config.WORKERS_AI_TOKEN:
            raise LLMGenerationError(
                "Workers AI token not configured. Set WORKERS_AI_TOKEN.",
                error_code="LLM_001",
            )

        self._client = client or OpenAI(
            api_key=self._config.WORKERS_AI_TOKEN,
            base_url=self._config.workers_ai_base_url(),
            timeout=self._config.WORKERS_AI_TIMEOUT,
            max_retries=self._config.MAX_RETRIES,
            default_headers={"cf-aig-gateway-id": self._config.WORKERS_AI_GATEWAY_ID},
        )

    @property
    def name(self) -> str:
        """Provider identifier for logging."""
        return self._name

    def generate(
        self,
        prompt: str,
        system_prompt: str | None = None,
        temperature: float = 0.7,
    ) -> str:
        """Generate text synchronously; loud LLMGenerationError on failure."""
        messages: list[ChatCompletionMessageParam] = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        try:
            response = self._client.chat.completions.create(
                model=self._config.WORKERS_AI_MODEL,
                messages=messages,
                temperature=temperature,
            )
            content: str | None = response.choices[0].message.content
            if content is None:
                raise LLMGenerationError(
                    "Workers AI returned empty response", error_code="LLM_001"
                )
            return content
        except APITimeoutError as e:
            raise LLMGenerationError(
                f"Workers AI request timed out: {e}",
                error_code="LLM_002",
                details={"timeout": self._config.WORKERS_AI_TIMEOUT},
            ) from e
        except RateLimitError as e:
            raise LLMGenerationError(
                f"Workers AI rate limit exceeded: {e}", error_code="LLM_003"
            ) from e
        except APIError as e:
            raise LLMGenerationError(
                f"Workers AI API error: {e}",
                error_code="LLM_001",
                details={"status_code": getattr(e, "status_code", None)},
            ) from e


def build_llm_provider(config: type[LLMConfig] | None = None) -> LLMProvider:
    """Select the chat provider from ``LLMConfig.PROVIDER`` (loud on unknown)."""
    cfg = config or LLMConfig
    provider = cfg.PROVIDER.lower()
    if provider == "workers_ai":
        return WorkersAIClient(config=cfg)
    if provider == "deepseek":
        return DeepSeekClient(config=cfg)
    if provider == "mock":
        return MockLLM()
    raise LLMGenerationError(
        f"Unknown LLM_PROVIDER: {cfg.PROVIDER!r} (expected deepseek|workers_ai|mock)",
        error_code="LLM_001",
    )
```

Export both from `src/babylon/intelligence/ai/__init__.py` (follow its existing `__all__` style).

- [ ] **Step 4: Green** — scoped pytest on the new file + `tests/unit/ai/test_llm_provider.py`.

- [ ] **Step 5: Rewire `narrative_service.py`** — find where `DeepSeekClient()` is constructed
lazily inside `_generate`/`_build` (read the file; recon: lines ~1757–1770 pattern in
engine_bridge call `self._narrative_service`); replace direct construction with
`build_llm_provider()`. Existing injected-provider test seam (`llm=` kwarg) stays.
Run: `PYTHONPATH="$PWD/src" poetry run pytest tests/unit/web/test_narrative_service.py -v` → PASS.

- [ ] **Step 6: Commit** — `feat(ai): WorkersAIClient via AI Gateway + provider factory (program-07 D3)`

---

### Task B4: NarrationRecord persistence

**Files:**
- Modify: `web/game/models.py` (new model, follow TickEvent's style), new migration
- Modify: `web/game/narrative_service.py` (persist on completion)
- Test: `tests/unit/web/test_narration_record.py`

**Interfaces:**
- Produces: Django model `NarrationRecord` with fields: `session` (FK `GameSession`,
  `on_delete=CASCADE`, `related_name="narration_records"`), `tick: IntegerField`,
  `beat_id: CharField(64)`, `scope: CharField(16)` (choices event|tick|county|endgame),
  `subject_ref: CharField(128, null=True)`, `headline: TextField`, `body: TextField`,
  `register: CharField(16)` (choices wire|analysis), `model_id: CharField(128)`,
  `prompt_version: CharField(32)`, `degraded: BooleanField(default=False)`,
  `error: TextField(blank=True, default="")`, `created_at: DateTimeField(auto_now_add=True)`;
  `unique_together = ("session", "tick", "beat_id")`, ordering `("tick", "beat_id")`.
  Model docstring MUST record the III.6 limitation honestly: DeepSeek and Workers AI expose no
  tokenizer version over the API, so the pin set is `model_id` + `prompt_version` (hash-derived);
  tokenizer_version is documented-unavailable, not silently omitted.
- Produces: `NarrativeService` writes two records per successful generation —
  corporate → `register="wire"`, liberated → `register="analysis"` (v1 mapping; the Gramscian
  triptych extends the register enum later — documented in the model docstring), both
  `scope="tick"`, `subject_ref=None`, `beat_id=f"{register}-{tick}"`, headline = first line of
  the narrative (up to 120 chars), body = remainder (or full text when single-line, headline
  then `"Tick {tick}"`). Degraded generations persist ONE record with `degraded=True`,
  `register="wire"`, headline `"NARRATOR DEGRADED"`, body = the error string (III.11: visible,
  never silent).

- [ ] **Step 1: Failing tests** (pytest-django; use existing web test fixtures — read
`tests/unit/web/conftest.py` first and reuse its session factory if present):

```python
"""NarrationRecord persistence (program-20 Track B)."""
import pytest

from web.game.models import GameSession, NarrationRecord


@pytest.mark.django_db
def test_record_roundtrip() -> None:
    session = GameSession.objects.create()  # extend kwargs per the model's required fields
    rec = NarrationRecord.objects.create(
        session=session, tick=3, beat_id="wire-3", scope="tick", subject_ref=None,
        headline="RENT EXTRACTED", body="…", register="wire",
        model_id="@cf/openai/gpt-oss-20b", prompt_version="sha256:abc123def456",
    )
    assert list(session.narration_records.all()) == [rec]


@pytest.mark.django_db
def test_unique_beat_per_session_tick() -> None:
    from django.db import IntegrityError
    session = GameSession.objects.create()
    kwargs = dict(session=session, tick=1, beat_id="wire-1", scope="tick",
                  headline="h", body="b", register="wire",
                  model_id="m", prompt_version="v")
    NarrationRecord.objects.create(**kwargs)
    with pytest.raises(IntegrityError):
        NarrationRecord.objects.create(**kwargs)
```

(Adjust `GameSession.objects.create()` kwargs after reading the model — it may require
scenario/name fields; the test factory in conftest is preferred if one exists.)

- [ ] **Step 2: FAIL** (no model). **Step 3:** implement the model exactly as the Interfaces
block specifies + `poetry run python web/manage.py makemigrations game` (from `web/`, or via the
mise task if one exists — check `.mise.toml` for `web:migrate`). **Step 4:** wire persistence
into `NarrativeService._generate`'s completion path (both success and degraded branches — the
service already builds `NarrativeResult`; add a `_persist(result, session_id, tick)` that maps
to records inside `transaction.atomic()`, keyed idempotently with `update_or_create` on
`(session, tick, beat_id)`). **Step 5:** green scoped run incl. the existing
`test_narrative_service.py` parity tests. **Step 6:** commit
`feat(web): persist narrator beats — NarrationRecord model + service wiring`.

---

### Task B5: GET /api/games/{id}/narration/ endpoint

**Files:**
- Modify: `web/game/urls.py` (one path), `web/game/api.py` (one view — read `game_wire`'s
  view/envelope/auth pattern FIRST and mirror it exactly)
- Modify: `src/frontend/src/api/endpoints.ts` (register entry), `src/frontend/src/lib/narration/client.ts`
  (route through the registry entry; keep exported signature identical)
- Test: `tests/unit/web/test_narration_endpoint.py`

**Interfaces:**
- Consumes: `NarrationRecord` (B4); `NarrativeService.is_enabled()` (existing).
- Produces (the contract `client.ts` documents — data payload inside the standard envelope):
  `{"status": "offline"|"pending"|"ready", "beats": [{"id","tick","scope","subjectRef","headline","body","register"}]}`
  — camelCase `subjectRef` on the wire, matching `types/narration.ts`. Semantics:
  flag off → `offline` with `[]`; flag on + no records ≥ since_tick → `pending`;
  flag on + records → `ready` (degraded records ARE returned — loud failure is visible).
  `?since_tick=N` filters `tick >= N`.

- [ ] **Step 1: Failing tests** — four cases: flag-off offline; flag-on empty → pending;
flag-on with seeded records → ready + exact beat dict shape (assert `subjectRef` key spelling);
`since_tick` filtering. Use the app's existing API test client pattern from a neighbouring
endpoint test (read `tests/unit/web/test_engine_bridge.py` or the api test module for the
session/auth fixture; mirror it).

- [ ] **Step 2: FAIL (404 route).**

- [ ] **Step 3: Implement.** urls.py — add alongside the other game routes:
`path("games/<uuid:game_id>/narration/", api.game_narration, name="game-narration"),`
(match the exact converter the sibling routes use — read them; if they use `<str:game_id>`,
follow suit). View in `api.py`:

```python
def game_narration(request: HttpRequest, game_id: str) -> JsonResponse:
    """AI narration beats for a session (program-20; contract: types/narration.ts).

    Flag-off is an honest, labeled "offline" — never an empty fake (III.11).
    """
    session = _get_session_or_404(game_id)  # use api.py's existing session helper — read it
    from web.game.narrative_service import NarrativeService

    if not NarrativeService.is_enabled():
        return _ok({"status": "offline", "beats": []})  # use api.py's envelope helper

    since_tick = int(request.GET.get("since_tick", 0))
    qs = session.narration_records.filter(tick__gte=since_tick).order_by("tick", "beat_id")
    beats = [
        {
            "id": r.beat_id, "tick": r.tick, "scope": r.scope,
            "subjectRef": r.subject_ref, "headline": r.headline,
            "body": r.body, "register": r.register,
        }
        for r in qs
    ]
    return _ok({"status": "ready" if beats else "pending", "beats": beats})
```

(`_get_session_or_404` / `_ok` are stand-ins for api.py's REAL helper names — read the file and
use its actual envelope/session-lookup helpers; `is_enabled` may be instance-level — mirror how
`get_wire_feed` reaches the service.)

- [ ] **Step 4: endpoints.ts + client.ts.** Add to the registry (mirror the `wire:` entry):
`narration: ep<NarrationFetchResult>("/api/games/:id/narration/"),` (import the type); refactor
`fetchNarration` to call through the registry entry instead of the literal URL — exported
signature and OFFLINE degradation unchanged. Run `npx tsc --noEmit` + `npx vitest run --silent`
scoped per repo convention (`cd src/frontend`).

- [ ] **Step 5: Green** (Django tests + frontend typecheck). **Step 6: Commit** —
`feat(web): real /narration/ endpoint to the cockpit's typed contract + registry entry`

---

### Task B6: Stale-doc cleanup + parity sweep

**Files:**
- Modify: `ai/architecture.yaml` (rewrite the `ai_narrative` flow block ~lines 873–893: pgvector
  not ChromaDB, NarrativeDirector flag-gated per D3, prompt registry, Workers AI provider option,
  narration endpoint; delete the "No LLM client implementation" claim)
- Modify: `src/babylon/config/llm_config.py` — comment block for `CANONICAL_EMBEDDING_*`:
  state plainly that the RUNTIME default embedder is Ollama `embeddinggemma:latest` (768-dim,
  matches `CANONICAL_EMBEDDING_DIM`); the mpnet id+revision pin documents the canonical
  sentence-transformers reference model, not the runtime default. No dimension change.
- Test: none (docs); verification is grep-based.

- [ ] **Step 1:** Make both edits. **Step 2:** `poetry run yamllint ai/architecture.yaml`;
`rg -n 'ChromaDB|chromadb' ai/architecture.yaml` → only historical mentions labeled as such.
**Step 3:** Full targeted sweep:

```bash
PYTHONPATH="$PWD/src" poetry run pytest tests/unit/ai tests/unit/web -q
mise run check:quick
```
Expected: green, plus paste the flag-off parity test names into the report.
**Step 4: Commit** — `docs(ai): architecture ai_narrative block matches reality; embedding pin comment honest`

---

### Task B7: Close-out

- [ ] Single-flight full gate: `mise run check` (only this once, nothing else running heavy).
- [ ] Owner-run note in the report: `qa:regression` recommended before merge (Track B touches no
  engine/economics/defines code — regression is expected byte-identical; if anything moves, STOP).
- [ ] Update `ai/state.yaml` Track B status; memory entry (provider factory, prompt registry,
  narration endpoint contract); update `reports/seam-wiring-punchlist.md` narration rows if
  present.
