# ADR096 Inference Lane Mechanics Implementation Plan

> **STATUS NOTE (2026-07-20, ADR101):** this plan's "Task 2" (tag
> `llm_provider.py` as LEGACY web lane and quarantine it) is **superseded by
> deletion** — the BD ruled the legacy stack removed outright, and ADR101
> executed the full director/judge/web migration this plan recorded as a
> follow-up. The rest of the plan (llama-server lifecycle, model manifest,
> `babylon_intel` role, embedding-dimension seam, CLI `doctor`) remains
> unexecuted and valid; its import-linter contract
> (`providers` never imports `intelligence.ai`) is still owed.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps
> use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement ADR096's inference-lane mechanics — bundled CPU llama-server lifecycle,
the signed model manifest with `doctor --provision`, the `babylon_intel` least-privilege
Postgres role, the embedding-dimension schema seam, the `sentence-transformers`/`torch`
removal, and the minimal old/new provider-abstraction reconciliation.

**Architecture:** The landed §A8 provider seam (`src/babylon/intelligence/providers.py`,
framework-free OpenAI-compatible transport, precedence bundled → external → cloudflare →
mute) is extended, not rewritten. This plan adds three greenfield modules under
`src/babylon/intelligence/` (`llama_server.py` supervisor, `model_manifest.py` + shipped
`data/model_manifest.toml`, `embedding_dims.py`), a provision core, one raw SQL migration
(`0036_babylon_intel_role.sql`), and reconciliation touch-ups to the legacy web lane.

**Tech Stack:** Python 3.12 (mypy strict, Pydantic v2), Poetry 2.2.1, pytest (no-network
via injected fakes, mirroring `tests/unit/intelligence/test_providers.py`), raw SQL over
psycopg for migrations, stdlib `urllib`/`http.server`/`subprocess` (no new runtime deps),
import-linter for the seam boundary contract.

## Global Constraints

- Base branch: `dev` @ 8ee8707f. Execute in a fresh git worktree off `dev`
  (superpowers:using-git-worktrees); the owner's live checkout must never be touched.
  Pushes are owner-run; commits use conventional format with the Co-Authored-By trailer.
- Python `>=3.12,<4.0` (pyproject). PRECONDITION: this plan runs AFTER the ADR095
  packaging-train plan — the tree must already be on uv (PEP-621 dep tables, committed
  `uv.lock`, `babylon` CLI installed, poetry removed). Gate before Task 1:
  `test -f uv.lock && uv run babylon --help >/dev/null && echo "precondition OK"` —
  expected `precondition OK`; if it fails, STOP and execute the ADR095 plan first.
- Strict typing (mypy strict, function signatures fully annotated); Pydantic models for
  data objects; explicit exception types, no bare except; all loops bounded.
- Amendment V: narrator-only AI — no LLM output may enter the simulation input path.
  Amendment X.6: no LLM framework dependencies (no langchain etc.).
- Cloudflare: Workers Free plan everywhere; nothing that can bill.
- Verification battery before each commit: `uv run pytest <touched tests>`,
  `uv run ruff check src tests`, `uv run mypy src` — scale to what the task
  touched; the plan's final task runs the full gate (`mise run check` ≈ lint, lint:imports,
  typecheck, test:unit).
- NEVER read, print, or commit secrets (`.env`, `terraform.tfvars`, `*.tfstate`,
  `vault.yml`, age private keys). Model weights are never committed and never enter the
  Nix store (D3).

### Plan-specific constraints

- **Upstream dependency:** this plan consumes ADR095's CLI skeleton
  (`src/babylon/cli/doctor.py`, `app = typer.Typer(...)` in `src/babylon/cli/__init__.py`).
  On `dev` @ 8ee8707f the `src/babylon/cli/` package does NOT yet exist (verified: `ls
  src/babylon/cli/` → absent). Execution order is 095 → 094 → **096** → 097, so doctor.py
  is present at run time. The `--provision` **CLI wiring step (Task 6, Step 6.9) is GATED on
  ADR095 having landed doctor.py**; the provision *core* (`provision.py`) is fully testable
  without the CLI and carries the behavior.
- **D2 (precedence) is already implemented** on `dev` — `resolve_provider()` in
  `providers.py:481` walks bundled → external → cloudflare-if-keyed → mute with 18 passing
  tests. This plan does NOT re-implement it; Task 8 adds the lazy bundled-server *start*
  ahead of it and leaves the resolver's signature untouched.
- **D6 (premium metering) is DEFERRED by the ADR** — no work; recorded in Task 10.
- **Weights do not exist yet.** The `babylon-data` R2 bucket has no gguf uploaded
  (verified 2026-07-20). The shipped manifest therefore marks every entry
  `available = false`; `doctor --provision` reports the owner-provisioning gate loudly and
  provisions nothing until the owner flips entries to `available = true` with real
  `url`/`sha256`/`bytes`. This is the ONE honest mechanism chosen (see Task 5).
- **`hypergraph-rs` port-parity and owner-run keygen are out of scope** and untouched here.

---

### Task 1: Remove `sentence-transformers` from the dependency tree (A7.5)

`sentence-transformers = "^5.6"` has ZERO imports repo-wide (only `CANONICAL_EMBEDDING_*`
reference-pin *strings* in `llm_config.py:14-24`, which import nothing). Removing it sheds
its transitive `torch` (~2 GB off the closure). `torch` is neither declared nor imported —
nothing to remove there.

**Files:**

- Modify: `pyproject.toml` (drop `sentence-transformers = "^5.6"`, verified at line 85 on
  `dev`; ADR095 may have moved dep tables to PEP-621 `[project.dependencies]` first — locate
  the line by name, do not assume the line number).
- Modify: `uv.lock` (regenerated by the lock command below).

**Interfaces:** none (dependency/config task; verified via lock + grep, not TDD).

- [ ] Confirm the dependency has zero code imports (must print nothing):

  ```bash
  rg -n "import sentence_transformers|from sentence_transformers" src tests
  ```

  Expected output: no matches (exit 1). If any match appears, STOP — the removal is not
  safe and this is a real finding, not a mechanical edit.

- [ ] Locate the declaration resiliently (works whether it lives in the Poetry table or a
  PEP-621 array):

  ```bash
  rg -n "sentence-transformers" pyproject.toml
  ```

  Expected: one line, e.g. `85:sentence-transformers = "^5.6"`.

- [ ] Delete exactly that one declaration line in `pyproject.toml` (the
  `sentence-transformers` dependency entry). Leave `CANONICAL_EMBEDDING_MODEL_ID` in
  `llm_config.py` untouched — it is a reference-pin string, not a runtime dep.

- [ ] Re-lock without touching other versions and prune the environment:

  ```bash
  uv lock && uv sync
  ```

  Expected: lock succeeds; the resolver removes `sentence-transformers` (and its
  torch/nvidia transitive chain) from `uv.lock`. Verify (`rg -c` prints nothing on zero
  matches, so echo the zero explicitly):

  ```bash
  rg -c 'name = "sentence-transformers"' uv.lock || echo 0
  rg -c 'name = "torch"' uv.lock || echo 0
  ```

  Expected output: `0` on both lines.

- [ ] Fast import-surface sanity (nothing imported the dep, so this must stay green):

  ```bash
  uv run python -c "import babylon.config.llm_config; import babylon.persistence.pgvector_store; print('ok')"
  ```

  Expected output: `ok`.

- [ ] Commit:

  ```bash
  git add pyproject.toml uv.lock
  git commit -m "chore(deps): drop sentence-transformers (A7.5, ADR096) — zero imports, sheds torch" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 2: Reconcile the legacy web lane (defaults, legacy mark, import guard)

Three minimal reconciliation trims (scope per brief): (a) align `llm_config.py`'s
Workers-AI default model to the deployed api-worker allowlist; (b) mark
`llm_provider.py` as the legacy web lane in its module docstring; (c) add an import-linter
contract forbidding the framework-free seam (`babylon.intelligence.providers`) from
importing the legacy abstraction (`babylon.intelligence.ai`). Full director/judge migration
is a RECORDED follow-up, out of scope (they serve the legacy web lane).

**Files:**

- Modify: `src/babylon/config/llm_config.py:68` (`WORKERS_AI_MODEL` default).
- Test (modify): `tests/unit/config/test_llm_config_workers_ai.py:10` (existing assertion
  pins the old default — updated first, TDD).
- Modify: `src/babylon/intelligence/ai/llm_provider.py:1-14` (module docstring).
- Modify: `pyproject.toml` `[tool.importlinter]` (new contract; existing contracts at
  `pyproject.toml:519-574` — follow that style).

**Interfaces:**

- Produces: `LLMConfig.WORKERS_AI_MODEL == "@cf/meta/llama-3.1-8b-instruct-fast"` (matches
  `providers.DEFAULT_CLOUDFLARE_CHAT_MODEL` and the api-worker `CHAT_MODELS` allowlist).
- Produces: import-linter forbidden contract `providers` ⇏ `intelligence.ai`.

- [ ] Update the existing test to the new default (RED first). Edit
  `tests/unit/config/test_llm_config_workers_ai.py` line 10:

  ```python
      assert LLMConfig.WORKERS_AI_MODEL == "@cf/meta/llama-3.1-8b-instruct-fast"
  ```

- [ ] Run — expect FAIL (code still returns the old default):

  ```bash
  uv run pytest tests/unit/config/test_llm_config_workers_ai.py::test_defaults -q
  ```

  Expected: FAIL — `assert '@cf/openai/gpt-oss-20b' == '@cf/meta/llama-3.1-8b-instruct-fast'`.

- [ ] Change the default in `src/babylon/config/llm_config.py` line 68:

  ```python
      WORKERS_AI_MODEL: Final[str] = os.getenv("WORKERS_AI_MODEL", "@cf/meta/llama-3.1-8b-instruct-fast")
  ```

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/config/test_llm_config_workers_ai.py -q
  ```

  Expected: `3 passed`.

- [ ] Mark the legacy lane. Replace the `llm_provider.py` module docstring opening
  (lines 1-14) with an honest legacy tag, preserving the existing component notes:

  ```python
  """LLM Provider strategy pattern for text generation (LEGACY web lane).

  DEPRECATION NOTE (ADR096, 2026-07-20): this is the OLD provider abstraction
  serving the legacy web/narration lane (director.py, judge.py). The current,
  framework-free inference seam is ``babylon.intelligence.providers`` (§A8):
  one OpenAI-compatible transport, precedence bundled → external → cloudflare
  → mute. New code MUST target that seam. Migrating director/judge onto it is
  a RECORDED follow-up, deliberately out of ADR096 scope — they still serve
  the legacy web lane. Do not add features here.

  This module provides the "Mouth" of the AI Observer - the interface
  through which the NarrativeDirector speaks. It follows the same
  Protocol pattern as SimulationObserver for loose coupling.

  Components:
  - LLMProvider: Protocol defining the text generation interface
  - MockLLM: Deterministic mock for testing
  - DeepSeekClient: Production client using DeepSeek API

  SYNC API: All providers implement synchronous generate() to match
  the SimulationObserver pattern. Async implementations wrap internally.
  """
  ```

- [ ] Add the import-linter contract. Append to `pyproject.toml` after the last existing
  `[[tool.importlinter.contracts]]` block (i.e. after the `sentinels` contract ending near
  line 574 — verify the true end with `rg -n "importlinter.contracts" pyproject.toml`):

  ```toml
  [[tool.importlinter.contracts]]
  name = "the framework-free seam must not import the legacy ai abstraction"
  type = "forbidden"
  source_modules = ["babylon.intelligence.providers"]
  forbidden_modules = ["babylon.intelligence.ai"]
  ```

- [ ] Run the import boundary check — expect PASS (the seam already imports nothing from
  `babylon`, so the new contract holds):

  ```bash
  uv run lint-imports
  ```

  Expected: `Contracts: N kept, 0 broken.` — the new contract listed as kept.

- [ ] Typecheck the touched module:

  ```bash
  uv run mypy src/babylon/config/llm_config.py src/babylon/intelligence/ai/llm_provider.py
  ```

  Expected: `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add pyproject.toml src/babylon/config/llm_config.py \
    src/babylon/intelligence/ai/llm_provider.py tests/unit/config/test_llm_config_workers_ai.py
  git commit -m "refactor(intel): reconcile legacy lane — align CF default, tag legacy, forbid seam→ai import (ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 3: Embedding-dimension seam (`embedding_dims.py`) (D5)

The allowlisted embedding models pin the Archive column dimension. Pure lookup + loud
failure; no schema change is commissioned here (column migration to vector(1024) is future
embedded-PG work — recorded, not done).

**Files:**

- Create: `src/babylon/intelligence/embedding_dims.py`
- Create: `tests/unit/intelligence/test_embedding_dims.py`

**Interfaces:**

- Produces: `EMBEDDING_DIMENSIONS: Final[dict[str, int]]`,
  `dimension_for(model_pin: str) -> int` (raises `KeyError` — loud),
  `assert_store_dimension(model_pin: str, store_dimension: int) -> None` (raises
  `EmbeddingDimensionMismatch`).

- [ ] Write the failing test `tests/unit/intelligence/test_embedding_dims.py`:

  ```python
  """Behavioral contract for the embedding-dimension seam (D5, ADR096).

  The allowlisted embedding models pin the Archive column dimension. Unknown
  pins fail LOUD (III.11): a silent wrong dimension corrupts the vector space.
  """

  from __future__ import annotations

  import pytest

  from babylon.intelligence.embedding_dims import (
      EMBEDDING_DIMENSIONS,
      EmbeddingDimensionMismatch,
      assert_store_dimension,
      dimension_for,
  )


  def test_allowlisted_dims_match_adr096() -> None:
      assert EMBEDDING_DIMENSIONS["@cf/baai/bge-m3"] == 1024
      assert EMBEDDING_DIMENSIONS["@cf/baai/bge-base-en-v1.5"] == 768
      assert EMBEDDING_DIMENSIONS["embeddinggemma:latest"] == 768


  def test_dimension_for_returns_pinned_dim() -> None:
      assert dimension_for("@cf/baai/bge-m3") == 1024
      assert dimension_for("embeddinggemma:latest") == 768


  def test_dimension_for_unknown_pin_raises_loud() -> None:
      with pytest.raises(KeyError):
          dimension_for("@cf/unknown/model")


  def test_assert_store_dimension_ok_when_matched() -> None:
      # No raise == pass.
      assert_store_dimension("@cf/baai/bge-base-en-v1.5", 768)


  def test_assert_store_dimension_mismatch_raises() -> None:
      with pytest.raises(EmbeddingDimensionMismatch) as exc:
          assert_store_dimension("@cf/baai/bge-m3", 768)
      assert "1024" in str(exc.value) and "768" in str(exc.value)


  def test_assert_store_dimension_unknown_pin_raises_keyerror() -> None:
      with pytest.raises(KeyError):
          assert_store_dimension("@cf/unknown/model", 768)
  ```

- [ ] Run — expect FAIL (module does not exist):

  ```bash
  uv run pytest tests/unit/intelligence/test_embedding_dims.py -q
  ```

  Expected: `ModuleNotFoundError: No module named 'babylon.intelligence.embedding_dims'`.

- [ ] Implement `src/babylon/intelligence/embedding_dims.py`:

  ```python
  """The embedding-dimension seam (D5, ADR096).

  The allowlisted embedding models pin the Archive's ``vector(N)`` column
  dimension: bge-m3 → 1024 (dense dim verified 2026-07-20 from BAAI's model
  card / config.json hidden_size), bge-base-en-v1.5 → 768, the Ollama default
  embeddinggemma → 768. All are under the pgvector HNSW 2000-dim index cap.

  A campaign's column dimension is fixed at creation (III.6 pins); changing the
  embedding model mid-campaign is a schema migration, not a config change. This
  module is the single source of truth reconciling three numbers: the pinned
  model, the store's configured dimension, and the DB column type. An unknown
  pin fails LOUD (KeyError) — a silently-wrong dimension corrupts the space.
  """

  from __future__ import annotations

  from typing import Final

  #: Allowlisted embedding pins → dense dimension. Mirrors the api-worker
  #: EMBED_MODELS allowlist plus the Ollama detected-external default.
  EMBEDDING_DIMENSIONS: Final[dict[str, int]] = {
      "@cf/baai/bge-m3": 1024,
      "@cf/baai/bge-base-en-v1.5": 768,
      "embeddinggemma:latest": 768,
  }


  class EmbeddingDimensionMismatch(ValueError):
      """A store/column dimension does not match the pinned model's dimension."""


  def dimension_for(model_pin: str) -> int:
      """Dense dimension for an allowlisted embedding pin.

      Raises:
          KeyError: the pin is not allowlisted — loud by design (III.11).
      """
      return EMBEDDING_DIMENSIONS[model_pin]


  def assert_store_dimension(model_pin: str, store_dimension: int) -> None:
      """Assert a store/column dimension matches the pinned model's dimension.

      Raises:
          KeyError: the pin is not allowlisted.
          EmbeddingDimensionMismatch: allowlisted, but the store dimension
              disagrees with the pin's dimension.
      """
      expected = dimension_for(model_pin)
      if store_dimension != expected:
          raise EmbeddingDimensionMismatch(
              f"embedding pin {model_pin!r} is {expected}-dimensional but the store "
              f"column is {store_dimension}-dimensional; the campaign's vector(N) column "
              f"is fixed at creation — changing models is a schema migration, not config."
          )
  ```

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/intelligence/test_embedding_dims.py -q
  ```

  Expected: `6 passed`.

- [ ] Typecheck:

  ```bash
  uv run mypy src/babylon/intelligence/embedding_dims.py
  ```

  Expected: `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add src/babylon/intelligence/embedding_dims.py tests/unit/intelligence/test_embedding_dims.py
  git commit -m "feat(intel): embedding-dimension seam — bge-m3=1024, bge-base/gemma=768 (D5, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 4: Wire the dimension seam into the PgVectorStore insert boundary (D5)

`PgVectorStore.add_chunks` already preflights every embedding's length against
`self._dimension` (`pgvector_store.py:119-125`). D5's addition: reconcile that configured
dimension with the campaign's pinned embedding model AT CONSTRUCTION, so the store, the
column, and the pin can never silently disagree. Backward-compatible: `model_pin` is an
optional keyword; existing callers (768-default) are unaffected.

**Files:**

- Modify: `src/babylon/persistence/pgvector_store.py:68-76` (`__init__` gains
  `model_pin: str | None = None`).
- Create: `tests/unit/persistence/test_pgvector_dimension_pin.py`

**Interfaces:**

- Consumes: `babylon.intelligence.embedding_dims.assert_store_dimension`.
- Produces: `PgVectorStore(pool, dimension=..., collection=..., model_pin=...)` raising
  `EmbeddingDimensionMismatch` when `dimension != dimension_for(model_pin)`.

- [ ] Confirm the current signature before editing:

  ```bash
  sed -n '68,77p' src/babylon/persistence/pgvector_store.py
  ```

  Expected (the multi-line signature exactly as in the source):

  ```text
      def __init__(
          self,
          pool: ConnectionPool[Connection[Any]],
          dimension: int = CANONICAL_EMBEDDING_DIM,
          collection: str = "default",
      ) -> None:
          self._pool = pool
          self._dimension = dimension
          self._collection = collection
  ```

- [ ] Write the failing test `tests/unit/persistence/test_pgvector_dimension_pin.py` (no DB
  — construction guard only; pool is never touched):

  ```python
  """D5 construction guard: the store dimension must match the pinned model.

  No database: the guard fires in ``__init__`` before any pool use, so a bare
  object stands in for the psycopg pool.
  """

  from __future__ import annotations

  import pytest

  from babylon.intelligence.embedding_dims import EmbeddingDimensionMismatch
  from babylon.persistence.pgvector_store import PgVectorStore


  class _UnusedPool:
      """The guard must not touch the pool; any use is a bug."""

      def connection(self) -> object:  # pragma: no cover - must never be called
          raise AssertionError("dimension guard must not open a connection")


  def test_matching_pin_constructs() -> None:
      store = PgVectorStore(_UnusedPool(), dimension=768, model_pin="@cf/baai/bge-base-en-v1.5")
      assert store is not None


  def test_mismatched_pin_raises() -> None:
      with pytest.raises(EmbeddingDimensionMismatch):
          PgVectorStore(_UnusedPool(), dimension=768, model_pin="@cf/baai/bge-m3")


  def test_unknown_pin_raises_keyerror() -> None:
      with pytest.raises(KeyError):
          PgVectorStore(_UnusedPool(), dimension=768, model_pin="@cf/unknown/model")


  def test_no_pin_is_backward_compatible() -> None:
      # Legacy path: no pin supplied → no guard, no raise.
      store = PgVectorStore(_UnusedPool(), dimension=768)
      assert store is not None
  ```

- [ ] Run — expect FAIL (`__init__` rejects the `model_pin` kwarg):

  ```bash
  uv run pytest tests/unit/persistence/test_pgvector_dimension_pin.py -q
  ```

  Expected: FAIL — `TypeError: __init__() got an unexpected keyword argument 'model_pin'`.

- [ ] Edit `src/babylon/persistence/pgvector_store.py`. Add the import near the existing
  `from babylon.config.llm_config import CANONICAL_EMBEDDING_DIM` line (line 30):

  ```python
  from babylon.intelligence.embedding_dims import assert_store_dimension
  ```

  Then change `__init__` (lines 68-76) to:

  ```python
      def __init__(
          self,
          pool: ConnectionPool[Connection[Any]],
          dimension: int = CANONICAL_EMBEDDING_DIM,
          collection: str = "default",
          model_pin: str | None = None,
      ) -> None:
          # D5 (ADR096): when the campaign's embedding pin is known, the store's
          # configured dimension MUST equal that pin's dimension — the vector(N)
          # column, the store, and the model reconcile at construction, loudly.
          if model_pin is not None:
              assert_store_dimension(model_pin, dimension)
          self._pool = pool
          self._dimension = dimension
          self._collection = collection
  ```

- [ ] Verify no new import-boundary breakage (persistence importing intelligence): the
  existing contracts forbid `persistence` → `engine` only, not `intelligence`. Confirm:

  ```bash
  uv run lint-imports
  ```

  Expected: `Contracts: N kept, 0 broken.` If this reports a broken contract, STOP — a
  layering rule forbids this import and the guard must live elsewhere (real finding).

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/persistence/test_pgvector_dimension_pin.py -q
  ```

  Expected: `4 passed`.

- [ ] Typecheck:

  ```bash
  uv run mypy src/babylon/persistence/pgvector_store.py
  ```

  Expected: `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add src/babylon/persistence/pgvector_store.py tests/unit/persistence/test_pgvector_dimension_pin.py
  git commit -m "feat(persistence): reconcile PgVectorStore dimension with pinned embed model (D5, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 5: Signed model manifest module + shipped data file (D3)

Weights NEVER enter the Nix store. The manifest ships INSIDE the package (`data/`), hence
inside the signed closure — **the closure narinfo signature IS the manifest signature**
(no separate signing step; that is the D3 "signed manifest" mechanism). Entries carry
`available = false` until the owner uploads weights to the `babylon-data` R2 bucket and
flips them; provision (Task 6) reports the gate loudly and provisions nothing meanwhile.

**Files:**

- Create: `src/babylon/intelligence/model_manifest.py`
- Create: `src/babylon/intelligence/data/__init__.py` (empty — makes `data/` a regular
  package so `importlib.resources.files("babylon.intelligence.data")` never depends on
  implicit-namespace resolution)
- Create: `src/babylon/intelligence/data/model_manifest.toml`
- Create: `tests/unit/intelligence/test_model_manifest.py`
- Modify: `pyproject.toml` `[tool.poetry]` — ensure the `.toml` data file ships with the
  package (add `include` if the packaging does not already capture non-`.py` files).

**Interfaces:**

- Produces: `ModelKind` StrEnum (`CHAT`, `EMBED`); `ModelEntry` (Pydantic:
  `name: str`, `kind: ModelKind`, `available: bool = False`, `url: str | None`,
  `sha256: str | None`, `bytes: int | None`, `dims: int | None`); `ModelManifest`
  (`models: list[ModelEntry]`, helpers `available_entries()`, `chat_entries()`,
  `embed_entries()`); `load_bundled_manifest() -> ModelManifest`.

- [ ] Write the failing test `tests/unit/intelligence/test_model_manifest.py`:

  ```python
  """Behavioral contract for the signed model manifest (D3, ADR096).

  The manifest ships in the package (data/model_manifest.toml) → inside the
  signed Nix closure. Entries are owner-provisioned: until the owner uploads
  weights to R2 and flips ``available = true`` with real url/sha256/bytes, no
  entry is available and provision is a loud no-op.
  """

  from __future__ import annotations

  import pytest
  from pydantic import ValidationError

  from babylon.intelligence.model_manifest import (
      ModelEntry,
      ModelKind,
      ModelManifest,
      load_bundled_manifest,
  )


  def test_bundled_manifest_loads_and_parses() -> None:
      manifest = load_bundled_manifest()
      assert isinstance(manifest, ModelManifest)
      # Ships one chat + one embed entry (owner-provisioned placeholders).
      assert len(manifest.chat_entries()) >= 1
      assert len(manifest.embed_entries()) >= 1


  def test_bundled_entries_are_owner_provisioned_not_yet_available() -> None:
      # Weights not uploaded to R2 yet (verified 2026-07-20) → nothing available.
      manifest = load_bundled_manifest()
      assert manifest.available_entries() == []


  def test_available_entry_requires_url_sha256_bytes() -> None:
      with pytest.raises(ValidationError):
          ModelEntry(name="chat", kind=ModelKind.CHAT, available=True)  # missing fields


  def test_available_entry_with_full_fields_validates() -> None:
      entry = ModelEntry(
          name="chat",
          kind=ModelKind.CHAT,
          available=True,
          url="https://data.example/chat.gguf",
          sha256="a" * 64,
          bytes=123,
      )
      assert entry.available and entry.url is not None


  def test_embed_entry_requires_dims() -> None:
      with pytest.raises(ValidationError):
          ModelEntry(name="embed", kind=ModelKind.EMBED, available=False)  # dims missing


  def test_unavailable_chat_entry_allows_empty_source() -> None:
      entry = ModelEntry(name="chat", kind=ModelKind.CHAT, available=False)
      assert entry.url is None and not entry.available
  ```

- [ ] Run — expect FAIL (module absent):

  ```bash
  uv run pytest tests/unit/intelligence/test_model_manifest.py -q
  ```

  Expected: `ModuleNotFoundError: No module named 'babylon.intelligence.model_manifest'`.

- [ ] Create the package dir with an empty `__init__.py`, then the data file
  `src/babylon/intelligence/data/model_manifest.toml`:

  ```bash
  mkdir -p src/babylon/intelligence/data
  touch src/babylon/intelligence/data/__init__.py
  ```

  ```toml
  # Babylon bundled model manifest (D3, ADR096).
  #
  # This file ships INSIDE the package → inside the signed Nix closure. The
  # closure's narinfo signature is therefore the manifest's signature; there is
  # no separate signing step. Weights themselves NEVER enter the store — they
  # are fetched by `babylon doctor --provision` into ~/.local/share/babylon/models/.
  #
  # Every entry is `available = false` until the owner uploads the gguf to the
  # babylon-data R2 bucket and fills url/sha256/bytes, then flips available.
  # Provision reports the owner-provisioning gate loudly while nothing is available.

  [[model]]
  name = "babylon-chat"
  kind = "chat"
  available = false

  [[model]]
  name = "babylon-embed"
  kind = "embed"
  available = false
  dims = 768
  ```

- [ ] Implement `src/babylon/intelligence/model_manifest.py`:

  ```python
  """The signed model manifest (D3, ADR096).

  The manifest ships in the package (``data/model_manifest.toml``) → inside the
  signed Nix closure, so the closure narinfo signature IS the manifest
  signature. Weights never enter the store; ``babylon doctor --provision``
  (see ``provision.py``) fetches them per this manifest into
  ``~/.local/share/babylon/models/``.

  Entries are owner-provisioned: an entry is fetched only when ``available`` is
  true, which the model validator ties to a complete ``url``/``sha256``/``bytes``
  triple. Until the owner uploads weights to the babylon-data R2 bucket, every
  entry is unavailable and provision is a loud no-op.
  """

  from __future__ import annotations

  import tomllib
  from enum import StrEnum
  from importlib import resources
  from typing import Any

  from pydantic import BaseModel, ConfigDict, model_validator


  class ModelKind(StrEnum):
      CHAT = "chat"
      EMBED = "embed"


  class ModelEntry(BaseModel):
      model_config = ConfigDict(frozen=True)

      name: str
      kind: ModelKind
      available: bool = False
      url: str | None = None
      sha256: str | None = None
      bytes: int | None = None
      dims: int | None = None

      @model_validator(mode="after")
      def _check_completeness(self) -> "ModelEntry":
          # An AVAILABLE entry must carry a complete, verifiable source triple —
          # provision refuses to fetch anything it cannot sha256-verify (III.11).
          if self.available and (self.url is None or self.sha256 is None or self.bytes is None):
              raise ValueError(
                  f"model {self.name!r} is available=true but missing url/sha256/bytes; "
                  "an available entry must be fully verifiable"
              )
          # Embedding entries must pin their dimension (the vector(N) column, D5).
          if self.kind is ModelKind.EMBED and self.dims is None:
              raise ValueError(f"embed model {self.name!r} must declare dims")
          return self


  class ModelManifest(BaseModel):
      model_config = ConfigDict(frozen=True)

      models: list[ModelEntry]

      def available_entries(self) -> list[ModelEntry]:
          return [entry for entry in self.models if entry.available]

      def chat_entries(self) -> list[ModelEntry]:
          return [entry for entry in self.models if entry.kind is ModelKind.CHAT]

      def embed_entries(self) -> list[ModelEntry]:
          return [entry for entry in self.models if entry.kind is ModelKind.EMBED]


  def parse_manifest(raw: dict[str, Any]) -> ModelManifest:
      """Parse a TOML mapping (``[[model]]`` array) into a validated manifest."""
      return ModelManifest(models=list(raw.get("model", [])))


  def load_bundled_manifest() -> ModelManifest:
      """Load the manifest shipped inside the package (the signed closure)."""
      data = resources.files("babylon.intelligence.data").joinpath("model_manifest.toml")
      with resources.as_file(data) as path:
          with path.open("rb") as handle:
              raw = tomllib.load(handle)
      return parse_manifest(raw)
  ```

- [ ] Ensure the data file ships with the package. Check current packaging then add an
  include if needed:

  ```bash
  rg -n "include|package-data|\.toml" pyproject.toml | rg -i "include|data"
  ```

  If `[tool.poetry]` does not already capture package data files, add under `[tool.poetry]`:

  ```toml
  include = [{ path = "src/babylon/intelligence/data/*.toml", format = ["sdist", "wheel"] }]
  ```

  (Poetry ships files under a package directory in the wheel by default; add the explicit
  `include` only if the verification step below fails to find the file in an install.)

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/intelligence/test_model_manifest.py -q
  ```

  Expected: `6 passed`.

- [ ] Typecheck:

  ```bash
  uv run mypy src/babylon/intelligence/model_manifest.py
  ```

  Expected: `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add src/babylon/intelligence/model_manifest.py \
    src/babylon/intelligence/data/__init__.py \
    src/babylon/intelligence/data/model_manifest.toml \
    tests/unit/intelligence/test_model_manifest.py pyproject.toml
  git commit -m "feat(intel): signed model manifest shipped in the closure (D3, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 6: `provision.py` core + `doctor --provision` wiring (D3)

Provision reads the manifest and downloads available entries into
`$XDG_DATA_HOME/babylon/models/` (default `~/.local/share/babylon/models/`), resumable via
HTTP Range on a `.part` file, sha256-verified before rename-into-place, bounded retries.
The fetcher is injected (mirrors `providers.py`'s `client_factory` seam), so the core is
tested with zero network. The CLI wiring (Step 6.9) consumes ADR095's `doctor.py` and is
GATED on that file existing.

**Files:**

- Create: `src/babylon/intelligence/provision.py`
- Create: `tests/unit/intelligence/test_provision.py`
- Modify (GATED on ADR095): `src/babylon/cli/doctor.py` (add `--provision` option).

**Interfaces:**

- Consumes: `model_manifest.load_bundled_manifest`, `ModelManifest`, `ModelEntry`.
- Produces: `default_models_dir(env: Mapping[str, str] | None = None) -> Path`;
  `Fetcher = Callable[[str, int], Iterator[bytes]]`;
  `provision_models(manifest, dest_dir, *, fetcher=None, max_retries=3) -> list[ProvisionResult]`;
  `ProvisionResult(name, status, detail)` where `status ∈ {"downloaded","skipped","gated"}`.

- [ ] Write the failing test `tests/unit/intelligence/test_provision.py`:

  ```python
  """Behavioral contract for model provisioning (D3, ADR096).

  Zero network: the fetcher is injected (mirroring the providers seam's
  client_factory). Pins what provision DOES — sha256 gate, .part resume,
  rename-into-place, and the loud owner-provisioning gate when nothing is
  available.
  """

  from __future__ import annotations

  import hashlib
  from collections.abc import Iterator
  from pathlib import Path

  import pytest

  from babylon.intelligence.model_manifest import ModelEntry, ModelKind, ModelManifest
  from babylon.intelligence.provision import (
      default_models_dir,
      provision_models,
  )


  def _sha256(data: bytes) -> str:
      return hashlib.sha256(data).hexdigest()


  def test_default_models_dir_prefers_xdg_data_home(tmp_path: Path) -> None:
      got = default_models_dir({"XDG_DATA_HOME": str(tmp_path)})
      assert got == tmp_path / "babylon" / "models"


  def test_default_models_dir_falls_back_to_local_share() -> None:
      got = default_models_dir({})
      assert got.parts[-3:] == (".local", "share") + ("babylon",) or got.name == "models"
      assert got.name == "models" and got.parent.name == "babylon"


  def test_gated_when_nothing_available(tmp_path: Path) -> None:
      manifest = ModelManifest(
          models=[ModelEntry(name="babylon-embed", kind=ModelKind.EMBED, available=False, dims=768)]
      )

      def unused_fetcher(url: str, start: int) -> Iterator[bytes]:  # pragma: no cover
          raise AssertionError("must not fetch a gated entry")

      results = provision_models(manifest, tmp_path, fetcher=unused_fetcher)
      assert len(results) == 1
      assert results[0].status == "gated"
      assert not any(tmp_path.iterdir())


  def test_downloads_and_verifies_available_entry(tmp_path: Path) -> None:
      payload = b"gguf-bytes-x" * 100
      digest = _sha256(payload)
      manifest = ModelManifest(
          models=[
              ModelEntry(
                  name="babylon-chat",
                  kind=ModelKind.CHAT,
                  available=True,
                  url="https://data.example/chat.gguf",
                  sha256=digest,
                  bytes=len(payload),
              )
          ]
      )

      def fetcher(url: str, start: int) -> Iterator[bytes]:
          # Honor Range: yield only the tail from `start`.
          yield payload[start:]

      results = provision_models(manifest, tmp_path, fetcher=fetcher)
      assert results[0].status == "downloaded"
      final = tmp_path / "babylon-chat.gguf"
      assert final.read_bytes() == payload
      assert not (tmp_path / "babylon-chat.gguf.part").exists()


  def test_resumes_from_existing_part_file(tmp_path: Path) -> None:
      payload = b"resumable-payload" * 50
      digest = _sha256(payload)
      # Pre-seed a partial .part with the first 100 bytes.
      part = tmp_path / "babylon-chat.gguf.part"
      part.write_bytes(payload[:100])
      seen_starts: list[int] = []

      def fetcher(url: str, start: int) -> Iterator[bytes]:
          seen_starts.append(start)
          yield payload[start:]

      manifest = ModelManifest(
          models=[
              ModelEntry(
                  name="babylon-chat",
                  kind=ModelKind.CHAT,
                  available=True,
                  url="https://data.example/chat.gguf",
                  sha256=digest,
                  bytes=len(payload),
              )
          ]
      )
      results = provision_models(manifest, tmp_path, fetcher=fetcher)
      assert results[0].status == "downloaded"
      assert seen_starts == [100]  # resumed, did not restart from 0
      assert (tmp_path / "babylon-chat.gguf").read_bytes() == payload


  def test_sha256_mismatch_raises_and_keeps_no_final(tmp_path: Path) -> None:
      payload = b"corrupt-content" * 10
      manifest = ModelManifest(
          models=[
              ModelEntry(
                  name="babylon-chat",
                  kind=ModelKind.CHAT,
                  available=True,
                  url="https://data.example/chat.gguf",
                  sha256="0" * 64,  # deliberately wrong
                  bytes=len(payload),
              )
          ]
      )

      def fetcher(url: str, start: int) -> Iterator[bytes]:
          yield payload[start:]

      with pytest.raises(ValueError, match="sha256"):
          provision_models(manifest, tmp_path, fetcher=fetcher, max_retries=1)
      assert not (tmp_path / "babylon-chat.gguf").exists()
  ```

- [ ] Run — expect FAIL (module absent):

  ```bash
  uv run pytest tests/unit/intelligence/test_provision.py -q
  ```

  Expected: `ModuleNotFoundError: No module named 'babylon.intelligence.provision'`.

- [ ] Implement `src/babylon/intelligence/provision.py`:

  ```python
  """Model provisioning: fetch weights per the signed manifest (D3, ADR096).

  ``babylon doctor --provision`` calls :func:`provision_models`. Downloads are
  resumable (HTTP Range onto a ``.part`` file), sha256-verified before an atomic
  rename-into-place, and bounded-retry. The fetcher is injected — the default
  uses stdlib ``urllib`` with a Range header; tests inject a fake, so the core
  carries zero network dependency.

  Weights land in ``$XDG_DATA_HOME/babylon/models/`` (default
  ``~/.local/share/babylon/models/``); they never enter the Nix store.
  """

  from __future__ import annotations

  import hashlib
  import logging
  import os
  import urllib.request
  from collections.abc import Callable, Iterator, Mapping
  from pathlib import Path

  from pydantic import BaseModel, ConfigDict

  from babylon.intelligence.model_manifest import ModelEntry, ModelKind, ModelManifest

  logger = logging.getLogger("babylon.intelligence.provision")

  #: (url, start_byte) -> byte chunks starting at ``start_byte`` (Range resume).
  Fetcher = Callable[[str, int], Iterator[bytes]]

  #: Bounded read loop cap: files are large but chunk count is bounded by size /
  #: chunk; this fixed upper bound guards against a non-terminating stream.
  _MAX_CHUNKS: int = 10_000_000
  _CHUNK_BYTES: int = 1 << 20  # 1 MiB


  class ProvisionResult(BaseModel):
      model_config = ConfigDict(frozen=True)

      name: str
      status: str  # "downloaded" | "skipped" | "gated"
      detail: str = ""


  def default_models_dir(env: Mapping[str, str] | None = None) -> Path:
      """``$XDG_DATA_HOME/babylon/models`` else ``~/.local/share/babylon/models``."""
      env = os.environ if env is None else env
      xdg = env.get("XDG_DATA_HOME")
      base = Path(xdg) if xdg else Path.home() / ".local" / "share"
      return base / "babylon" / "models"


  def _ext_for(kind: ModelKind) -> str:
      return ".gguf"  # both chat and embed lanes ship gguf weights


  def _default_fetcher(url: str, start: int) -> Iterator[bytes]:
      request = urllib.request.Request(url)
      if start > 0:
          request.add_header("Range", f"bytes={start}-")
      with urllib.request.urlopen(request) as response:  # noqa: S310 - manifest-pinned URL
          for _ in range(_MAX_CHUNKS):
              chunk = response.read(_CHUNK_BYTES)
              if not chunk:
                  return
              yield chunk
          raise ValueError(f"download of {url} exceeded {_MAX_CHUNKS} chunks — refusing")


  def _download_one(
      entry: ModelEntry, dest_dir: Path, fetcher: Fetcher, max_retries: int
  ) -> ProvisionResult:
      assert entry.url is not None and entry.sha256 is not None  # available => validated
      final_path = dest_dir / f"{entry.name}{_ext_for(entry.kind)}"
      part_path = dest_dir / f"{entry.name}{_ext_for(entry.kind)}.part"
      if final_path.exists():
          return ProvisionResult(name=entry.name, status="skipped", detail="already present")

      last_error = ""
      for _attempt in range(max_retries):
          start = part_path.stat().st_size if part_path.exists() else 0
          hasher = hashlib.sha256()
          if start > 0:
              hasher.update(part_path.read_bytes())
          with part_path.open("ab") as handle:
              for chunk in fetcher(entry.url, start):
                  handle.write(chunk)
                  hasher.update(chunk)
          digest = hasher.hexdigest()
          if digest == entry.sha256:
              part_path.replace(final_path)  # atomic rename-into-place
              return ProvisionResult(name=entry.name, status="downloaded", detail=digest)
          last_error = f"sha256 mismatch: got {digest}, expected {entry.sha256}"
          part_path.unlink(missing_ok=True)  # corrupt — restart clean next attempt
          logger.warning("provision %s: %s (attempt %d)", entry.name, last_error, _attempt + 1)
      raise ValueError(f"provision {entry.name} failed after {max_retries} attempts: {last_error}")


  def provision_models(
      manifest: ModelManifest,
      dest_dir: Path,
      *,
      fetcher: Fetcher | None = None,
      max_retries: int = 3,
  ) -> list[ProvisionResult]:
      """Provision every manifest entry into ``dest_dir``.

      Unavailable (owner-provisioned) entries are reported ``gated`` and never
      fetched — the loud signal that the owner has not yet uploaded weights to
      the babylon-data R2 bucket. Available entries are downloaded (resumable),
      sha256-verified, and renamed into place.
      """
      dest_dir.mkdir(parents=True, exist_ok=True)
      fetch = fetcher or _default_fetcher
      results: list[ProvisionResult] = []
      for entry in manifest.models:
          if not entry.available:
              results.append(
                  ProvisionResult(
                      name=entry.name,
                      status="gated",
                      detail="owner-provisioned: no weights uploaded to R2 yet",
                  )
              )
              continue
          results.append(_download_one(entry, dest_dir, fetch, max_retries))
      return results
  ```

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/intelligence/test_provision.py -q
  ```

  Expected: `6 passed`.

- [ ] Typecheck:

  ```bash
  uv run mypy src/babylon/intelligence/provision.py
  ```

  Expected: `Success: no issues found`.

- [ ] **GATE (ADR095): wire `--provision` into `doctor.py`.** Confirm the skeleton exists:

  ```bash
  test -f src/babylon/cli/doctor.py && echo PRESENT || echo "GATED: ADR095 doctor.py not landed"
  ```

  If PRESENT, add a `--provision` option to the doctor command that calls
  `provision_models(load_bundled_manifest(), default_models_dir())`, prints one line per
  `ProvisionResult` (loud on `gated`), and returns non-zero only on a raised
  provisioning error. Match ADR095's Typer/CliRunner test style under `tests/unit/cli/`.
  If GATED, STOP this step, record the gate in the commit body, and leave the wiring for a
  follow-up once 095 lands — the provision core above is complete and tested regardless.

- [ ] Commit:

  ```bash
  git add src/babylon/intelligence/provision.py tests/unit/intelligence/test_provision.py
  # add src/babylon/cli/doctor.py tests/unit/cli/... ONLY if the ADR095 gate was PRESENT
  git commit -m "feat(intel): resumable sha256-verified model provisioning (D3, ADR096)" \
    -m "doctor --provision CLI wiring gated on ADR095 doctor.py; provision core complete and tested." \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 7: Bundled llama-server supervisor (`llama_server.py`) (D1)

Greenfield — no subprocess management exists anywhere in `src/` (verified: 0 Popen/pexpect
hits). The supervisor owns a llama-server child: loopback only (`--host 127.0.0.1 --port
8737` to match `DEFAULT_BUNDLED_BASE_URL`), starts with chat + embed gguf paths, health-polls
`/health` with bounded retries, terminate→kill on close, context-manager protocol. Tested
with a tiny fake executable (a Python script standing in for llama-server) — no real model,
no network beyond loopback.

**Files:**

- Create: `src/babylon/intelligence/llama_server.py`
- Create: `tests/unit/intelligence/test_llama_server.py`
- Create: `tests/unit/intelligence/_fake_llama_server.py` (test fixture executable)

**Interfaces:**

- Consumes: `providers.ProviderUnavailable`.
- Produces: `resolve_llama_binary(env) -> Path` (raises `ProviderUnavailable` if absent);
  `LlamaServerSupervisor(binary, chat_gguf, embed_gguf, *, host="127.0.0.1", port=8737,
  startup_timeout_s=20.0, poll_interval_s=0.1)` with `.start()`, `.health_ok() -> bool`,
  `.close()`, and `__enter__`/`__exit__`.

- [ ] Create the fake executable `tests/unit/intelligence/_fake_llama_server.py` (a real
  loopback HTTP server that answers `/health` — the stand-in for llama-server):

  ```python
  """A tiny stand-in for llama-server used by supervisor tests.

  Parses ``--host``/``--port`` like the real binary, then serves 200 on
  ``/health`` over loopback. No model, no inference — the supervisor contract
  under test is process lifecycle + health polling, not real generation.
  """

  from __future__ import annotations

  import argparse
  from http.server import BaseHTTPRequestHandler, HTTPServer


  class _Handler(BaseHTTPRequestHandler):
      def do_GET(self) -> None:  # noqa: N802 - http.server API
          if self.path == "/health":
              self.send_response(200)
              self.end_headers()
              self.wfile.write(b'{"status":"ok"}')
          else:
              self.send_response(404)
              self.end_headers()

      def log_message(self, *_: object) -> None:  # silence test noise
          return


  def main() -> None:
      parser = argparse.ArgumentParser()
      parser.add_argument("--host", default="127.0.0.1")
      parser.add_argument("--port", type=int, default=8737)
      args, _unknown = parser.parse_known_args()
      HTTPServer((args.host, args.port), _Handler).serve_forever()


  if __name__ == "__main__":
      main()
  ```

- [ ] Write the failing test `tests/unit/intelligence/test_llama_server.py`:

  ```python
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
  ```

- [ ] Run — expect FAIL (module absent):

  ```bash
  uv run pytest tests/unit/intelligence/test_llama_server.py -q
  ```

  Expected: `ModuleNotFoundError: No module named 'babylon.intelligence.llama_server'`.

- [ ] Implement `src/babylon/intelligence/llama_server.py`:

  ```python
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
  from collections.abc import Mapping, Sequence
  from pathlib import Path
  from types import TracebackType

  from babylon.intelligence.providers import ProviderUnavailable

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

      def __enter__(self) -> "LlamaServerSupervisor":
          self.start()
          return self

      def __exit__(
          self,
          exc_type: type[BaseException] | None,
          exc: BaseException | None,
          tb: TracebackType | None,
      ) -> None:
          self.close()
  ```

- [ ] Run — expect PASS:

  ```bash
  uv run pytest tests/unit/intelligence/test_llama_server.py -q
  ```

  Expected: `4 passed` (a few seconds — real child processes on ephemeral ports).

- [ ] Typecheck:

  ```bash
  uv run mypy src/babylon/intelligence/llama_server.py
  ```

  Expected: `Success: no issues found`.

- [ ] Commit:

  ```bash
  git add src/babylon/intelligence/llama_server.py \
    tests/unit/intelligence/test_llama_server.py tests/unit/intelligence/_fake_llama_server.py
  git commit -m "feat(intel): bundled llama-server supervisor — loopback lifecycle (D1, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 8: Lazy bundled-lane start ahead of resolution (D1)

`resolve_provider()` (landed, 18 tests) only *probes* lanes; it does not start a server.
D1's lifecycle piece is a thin, tested composition helper the session bootstrap calls
BEFORE `resolve_provider`: when the mode selects the bundled lane (`auto` or `bundled`) and
the binary + both weights are present, start the supervisor so the subsequent probe finds a
healthy bundled lane; otherwise return `None` and let precedence fall through to
detected-external → cloudflare → mute. `resolve_provider`'s signature is left UNTOUCHED
(surgical change; its 18 tests must stay green).

**Files:**

- Modify: `src/babylon/intelligence/llama_server.py` (append `ensure_bundled_running`).
- Modify: `tests/unit/intelligence/test_llama_server.py` (append integration tests).

**Interfaces:**

- Consumes: `providers.IntelligenceSettings`, `resolve_llama_binary`,
  `LlamaServerSupervisor`.
- Produces: `ensure_bundled_running(settings, *, models_dir, env=None,
  supervisor_factory=None) -> LlamaServerSupervisor | None`.

- [ ] Append failing tests to `tests/unit/intelligence/test_llama_server.py`:

  ```python
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
  ```

- [ ] Run — expect FAIL (`ensure_bundled_running` undefined):

  ```bash
  uv run pytest tests/unit/intelligence/test_llama_server.py -k ensure -q
  ```

  Expected: `ImportError: cannot import name 'ensure_bundled_running'`.

- [ ] Append to `src/babylon/intelligence/llama_server.py` (after the class). Add
  `from collections.abc import Callable` to the existing `collections.abc` import line and
  the `IntelligenceSettings` import:

  ```python
  # add to existing imports at the top of the module:
  #   from collections.abc import Callable, Mapping, Sequence
  #   from babylon.intelligence.providers import IntelligenceSettings, ProviderUnavailable

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
  ```

  Note: `test_ensure_skips_when_weights_absent` returns before the binary check, so the
  `supervisor_factory` is unused in the three failing tests above — they assert the
  fall-through (`None`) paths, which do not spawn a process. The happy-path start is already
  covered by Task 7's `test_context_manager_starts_and_stops`.

- [ ] Run — expect PASS (both the earlier lifecycle tests and the new lazy-start tests):

  ```bash
  uv run pytest tests/unit/intelligence/test_llama_server.py -q
  ```

  Expected: `7 passed`.

- [ ] Confirm `resolve_provider`'s existing contract is untouched (its 18 tests stay green):

  ```bash
  uv run pytest tests/unit/intelligence/test_providers.py -q
  ```

  Expected: `18 passed`.

- [ ] Typecheck + import boundary:

  ```bash
  uv run mypy src/babylon/intelligence/llama_server.py && uv run lint-imports
  ```

  Expected: `Success: no issues found`; `Contracts: N kept, 0 broken.`

- [ ] Commit:

  ```bash
  git add src/babylon/intelligence/llama_server.py tests/unit/intelligence/test_llama_server.py
  git commit -m "feat(intel): lazy bundled-lane start ahead of resolve_provider (D1, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 9: `babylon_intel` least-privilege role migration (D4)

One raw SQL migration creating a LOGIN connection role with SELECT on the existing
projection/composition views and INSERT (+SELECT) on `document_chunk` ONLY — no
UPDATE/DELETE/DDL/Ledger. "Hoist" does not exist yet; the grant targets today's projection
surface (the composition views) and says so. `NarrationRecord` is a Django-side table that
may be absent from the sim DB — its GRANT is guarded by `to_regclass`. The migration is
idempotent (guarded `CREATE ROLE`, re-runnable GRANTs) per the applier's re-run contract.

**Files:**

- Create: `src/babylon/persistence/migrations/0036_babylon_intel_role.sql`
- Create: `tests/integration/persistence/test_babylon_intel_role.py`

**Interfaces:**

- Consumes: the `[0-9]*.sql` appliers (`runner.py:_apply_migrations` +
  `web/game/engine_bridge.py` mirror; advisory-locked, digest-stamped).
- Produces: role `babylon_intel` (LOGIN, NOSUPERUSER) with SELECT on the ten projection
  views and INSERT+SELECT on `document_chunk`.

- [ ] Confirm the highest existing migration prefix (must be 0035):

  ```bash
  ls src/babylon/persistence/migrations/ | rg '^00' | tail -3
  ```

  Expected: `...0035_playability_series.sql` is the highest.

- [ ] Create `src/babylon/persistence/migrations/0036_babylon_intel_role.sql`:

  ```sql
  -- 0036_babylon_intel_role.sql
  -- ADR096 D4 (queue item 13): the babylon_intel least-privilege connection role.
  --
  -- Database-enforced backing for Amendment V's observes-never-adjudicates
  -- posture (A7.2's three-way boundary): the intelligence lane connects AS this
  -- role and can therefore ONLY read the projection surface and append to the
  -- narrator-prose / embedding tables. No UPDATE, no DELETE, no DDL, no Ledger.
  --
  -- "Hoist" (the proposed dedicated projection layer) does not exist yet, so the
  -- SELECT grant targets today's projection surface: the five composition views
  -- (postgres_schema.py) plus the five current-state / value-aggregate views
  -- (0030_views_current.sql). When Hoist lands, a later migration narrows this.
  --
  -- Idempotent: guarded CREATE ROLE + re-runnable GRANTs. Both appliers re-run
  -- every migration on each start; a to_regclass guard skips grants for objects
  -- absent from a given database (the sim DB lacks Django's narration_record).

  DO $babylon_intel_role$
  BEGIN
      IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'babylon_intel') THEN
          -- LOGIN: this is a CONNECTION role (the lane authenticates as it),
          -- not a NOLOGIN group. NOSUPERUSER/NOCREATEDB/NOCREATEROLE are the
          -- least-privilege floor. Password is set out-of-band by the owner
          -- (ALTER ROLE ... PASSWORD), never in a committed migration.
          CREATE ROLE babylon_intel LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;
      END IF;
  END
  $babylon_intel_role$;

  -- SELECT on the composition views (today's projection surface).
  DO $grant_composition$
  DECLARE
      v text;
      composition_views text[] := ARRAY[
          'v_hex_economic', 'v_hex_mobilize', 'v_hex_aid', 'v_hex_heat', 'v_hex_intel',
          'v_hex_state_asof', 'v_county_value_aggregate', 'v_state_value_aggregate',
          'v_national_value_aggregate', 'v_global_phi_balance'
      ];
  BEGIN
      FOREACH v IN ARRAY composition_views LOOP
          IF to_regclass(v) IS NOT NULL THEN
              EXECUTE format('GRANT SELECT ON %I TO babylon_intel', v);
          END IF;
      END LOOP;
  END
  $grant_composition$;

  -- INSERT + SELECT on the embedding table ONLY (append narrator embeddings).
  DO $grant_document_chunk$
  BEGIN
      IF to_regclass('document_chunk') IS NOT NULL THEN
          GRANT SELECT, INSERT ON document_chunk TO babylon_intel;
      END IF;
  END
  $grant_document_chunk$;

  -- INSERT + SELECT on the Django-side narrator-prose table WHEN PRESENT. Absent
  -- from the sim DB (applied by Django migrate on the web DB); the guard keeps
  -- this migration green on both databases.
  DO $grant_narration_record$
  BEGIN
      IF to_regclass('game_narrationrecord') IS NOT NULL THEN
          GRANT SELECT, INSERT ON game_narrationrecord TO babylon_intel;
      END IF;
  END
  $grant_narration_record$;
  ```

- [ ] Confirm the Django narration table's real relation name before trusting the guard
  above (Django names tables `<app>_<model>`; the model is `NarrationRecord` in app
  `game`). Verify:

  ```bash
  rg -n "class NarrationRecord|db_table" web/game/migrations/0015_narrationrecord.py
  ```

  If `db_table` is set, use that exact name in the `to_regclass('...')` guard instead of
  `game_narrationrecord`. (The guard makes a wrong guess harmless — the GRANT is simply
  skipped — but pin the real name so the grant actually lands on the web DB.)

- [ ] Write the integration test `tests/integration/persistence/test_babylon_intel_role.py`
  (mirrors `test_migration_idempotency.py`'s fresh-DB + skip-if-no-PG pattern):

  ```python
  """D4 (ADR096): the babylon_intel least-privilege role migration.

  Applies the full migration set on a fresh database (mirroring the runner's
  real contract) and asserts the role exists with exactly the intended grants:
  SELECT on the projection views, SELECT+INSERT on document_chunk, and NO
  UPDATE/DELETE. Skips when PostgreSQL is unavailable.
  """

  from __future__ import annotations

  import re
  import uuid
  from collections.abc import Generator
  from typing import Any

  import psycopg
  import pytest
  from psycopg import sql

  from babylon.engine.headless_runner.runner import _apply_migrations

  pytestmark = pytest.mark.integration


  @pytest.fixture()
  def fresh_db_pool(pg_dsn: str) -> Generator[Any, None, None]:
      from psycopg_pool import ConnectionPool

      db_name = f"intel_role_{uuid.uuid4().hex[:12]}"
      try:
          admin = psycopg.connect(pg_dsn, autocommit=True)
      except psycopg.OperationalError:
          pytest.skip("PostgreSQL not available (set BABYLON_TEST_PG_DSN)")
      with admin:
          admin.execute(sql.SQL("CREATE DATABASE {}").format(sql.Identifier(db_name)))
      fresh_dsn = re.sub(r"dbname=\S+", f"dbname={db_name}", pg_dsn)
      pool = ConnectionPool(conninfo=fresh_dsn, min_size=1, max_size=2, open=True)
      try:
          yield pool
      finally:
          pool.close()
          with psycopg.connect(pg_dsn, autocommit=True) as admin:
              # DROP OWNED cleans grants so the shared role can be dropped, then
              # drop the DB. The role is cluster-global; leave it if other DBs use it.
              admin.execute(sql.SQL("DROP DATABASE {} WITH (FORCE)").format(sql.Identifier(db_name)))


  def _bootstrap(pool: Any) -> None:
      from babylon.persistence.postgres_schema import POSTGRES_SCHEMA_DDL

      with pool.connection() as conn:
          conn.autocommit = True
          for ddl in POSTGRES_SCHEMA_DDL:
              conn.execute(ddl)


  def test_role_created_with_expected_grants(fresh_db_pool: Any) -> None:
      _bootstrap(fresh_db_pool)
      _apply_migrations(fresh_db_pool)
      with fresh_db_pool.connection() as conn:
          conn.autocommit = True
          role = conn.execute(
              "SELECT rolcanlogin, rolsuper FROM pg_roles WHERE rolname = 'babylon_intel'"
          ).fetchone()
          assert role is not None, "babylon_intel role must exist"
          assert role[0] is True, "babylon_intel must be a LOGIN (connection) role"
          assert role[1] is False, "babylon_intel must NOT be superuser"

          # SELECT on a composition view; NO write on it.
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'v_hex_intel', 'SELECT')"
          ).fetchone()[0] is True
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'v_hex_intel', 'INSERT')"
          ).fetchone()[0] is False

          # INSERT + SELECT on document_chunk; NO UPDATE/DELETE.
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'document_chunk', 'INSERT')"
          ).fetchone()[0] is True
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'document_chunk', 'SELECT')"
          ).fetchone()[0] is True
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'document_chunk', 'UPDATE')"
          ).fetchone()[0] is False
          assert conn.execute(
              "SELECT has_table_privilege('babylon_intel', 'document_chunk', 'DELETE')"
          ).fetchone()[0] is False


  def test_migration_reapplies_idempotently(fresh_db_pool: Any) -> None:
      from babylon.persistence.postgres_schema import SCHEMA_STAMP_TABLE

      _bootstrap(fresh_db_pool)
      _apply_migrations(fresh_db_pool)
      with fresh_db_pool.connection() as conn:
          conn.autocommit = True
          conn.execute(f"DROP TABLE {SCHEMA_STAMP_TABLE}")  # force full re-run
      _apply_migrations(fresh_db_pool)  # guarded CREATE ROLE must not error
      with fresh_db_pool.connection() as conn:
          conn.autocommit = True
          assert conn.execute(
              "SELECT 1 FROM pg_roles WHERE rolname = 'babylon_intel'"
          ).fetchone() is not None
  ```

- [ ] Run — expect PASS if a test Postgres is reachable, else SKIP (both are green outcomes;
  the migration idempotency prefix-uniqueness test in `test_migration_idempotency.py` also
  now covers 0036 automatically):

  ```bash
  uv run pytest tests/integration/persistence/test_babylon_intel_role.py \
    tests/integration/persistence/test_migration_idempotency.py::test_migration_numeric_prefixes_are_unique -q
  ```

  Expected: `2 passed` (or role tests `skipped` if `BABYLON_TEST_PG_DSN` is unset; the
  prefix-uniqueness test always runs and must pass — proving 0036 has a unique prefix).

- [ ] Commit:

  ```bash
  git add src/babylon/persistence/migrations/0036_babylon_intel_role.sql \
    tests/integration/persistence/test_babylon_intel_role.py
  git commit -m "feat(persistence): babylon_intel least-privilege role migration (D4, ADR096)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

### Task 10: Full gate + honest-status record

Run the complete battery and record the ADR clauses that this plan does NOT execute (D2
already implemented; D6 deferred by the ADR).

**Files:**

- Modify: `ai/STATE.md` or `ai/backlog.md` (append the recorded follow-ups) — verify which
  file the repo uses for work-item records before writing:
  `rg -n "backlog|STATE" ai/*.md | head`.

**Interfaces:** none (verification + record task).

- [ ] Run the touched-suite battery (fast, no PG required):

  ```bash
  uv run pytest tests/unit/intelligence tests/unit/persistence tests/unit/config -q
  ```

  Expected: all green (Tasks 2-8 unit tests + existing intelligence tests).

- [ ] Run the full fast gate:

  ```bash
  uv run ruff check src tests && uv run mypy src && uv run lint-imports && \
    uv run pytest -m "not ai and not integration" -q
  ```

  Expected: ruff clean; `Success: no issues found`; `Contracts: N kept, 0 broken.`; all
  unit tests pass. If a test Postgres is available, also run
  `uv run pytest -m integration tests/integration/persistence -q`.

- [ ] Append the recorded follow-ups to the repo's work-item file (the RECORDED, out-of-scope
  items — never silently assumed done):

  ```markdown
  ## ADR096 recorded follow-ups (out of scope, honest status)

  - D2 (provider precedence) is ALREADY IMPLEMENTED on dev (`resolve_provider`, 18 tests);
    no work done here.
  - D6 (premium billing/metering on the Cloudflare lane) is DELIBERATELY DEFERRED by
    ADR096 — lands with the real ingest API, not before. No metering added.
  - director.py / judge.py migration off the legacy `intelligence.ai` lane onto the §A8
    seam: recorded follow-up, out of ADR096 scope (they serve the legacy web lane).
  - Model weights: the babylon-data R2 bucket has no gguf uploaded (2026-07-20). The
    manifest ships every entry `available = false`; `doctor --provision` reports the
    owner-provisioning gate loudly. OWNER GATE: upload weights, fill url/sha256/bytes,
    flip `available = true`.
  - `doctor --provision` CLI wiring is GATED on ADR095's `doctor.py`; the provision core is
    complete and tested. Land the wiring once 095 is merged (execution order 095 → 096).
  - vector(1024) column migration (bge-m3) is NOT commissioned by ADR096 — campaign-creation
    binding is future embedded-PG work.
  ```

- [ ] Commit:

  ```bash
  git add ai/
  git commit -m "docs(intel): record ADR096 out-of-scope follow-ups (D2 done, D6 deferred)" \
    -m "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
  ```

---

## ADR096 clause coverage

| ADR096 clause | Covered by | Notes |
| --- | --- | --- |
| D1 — bundled llama-server lifecycle (loopback, child process, one binary) | Task 7 (supervisor), Task 8 (lazy start ahead of resolution) | Binary from PATH/env until ADR094's closure bundles it |
| D2 — provider precedence bundled→external→cloudflare→mute | Recorded (Task 10) | Already implemented on dev (`resolve_provider`, 18 tests); untouched |
| D3 — signed model manifest + `doctor --provision` | Task 5 (manifest in closure), Task 6 (resumable sha256 provision; CLI wiring gated on 095) | Closure narinfo signature IS the manifest signature |
| D4 — `babylon_intel` least-privilege role | Task 9 (`0036_babylon_intel_role.sql` + integration test) | LOGIN role; SELECT on projection views, INSERT+SELECT on document_chunk only |
| D5 — embedding-dimension seam (bge-m3=1024, bge-base=768) | Task 3 (`embedding_dims.py`), Task 4 (PgVectorStore construction guard) | Column migration to 1024 not commissioned (recorded) |
| D6 — premium metering | Recorded (Task 10) | Deliberately deferred by the ADR; no work |
| A7.5 — drop sentence-transformers/torch | Task 1 | Zero imports; sheds torch transitively |
| Reconciliation debt (align CF default, tag legacy, seam↛ai contract) | Task 2 | Full director/judge migration recorded, out of scope |

## Upstream / gate summary

- **Consumes ADR095:** CLI skeleton (`src/babylon/cli/doctor.py`) for the `--provision`
  wiring (Task 6, Step 6.9) — GATED; provision core is complete and tested independently.
- **Independent of ADR094:** the llama-server binary is resolved from env/PATH here; ADR094
  later bundles it into the closure. No hard dependency.
- **Owner gates (never executed here):** upload weights to R2 + flip manifest entries to
  `available = true`; set the `babylon_intel` role password out-of-band (`ALTER ROLE ...
  PASSWORD`, never committed); owner-run pushes.
