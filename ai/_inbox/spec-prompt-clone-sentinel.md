# Spec Prompt — Clone Sentinel (`babylon.sentinels.clones`)

**Verified against:** `dev` @ `744f865` (2026-07-17). Re-verify file paths and census numbers before implementing.

---

## Mission and principle

Convergent reinvention is the signature failure mode of agentic engineering: each session optimizes inside a context window that doesn't contain the whole codebase, writes a *locally clean* helper, and every existing gate passes — ruff, mypy, and complexity checks are all **local** properties, satisfiable by a perfect duplicate. Live census at `744f865`: `_extract_tick` ×5, `_extract_persistent` ×3, `_coerce_role` ×3, `_get_class_consciousness_from_node` ×3, `_find_entity_by_role` ×2, four `pragma: no mutate` clamp-and-rebuild updaters — and a 586-line if/elif that grew behind a `# noqa: C901`.

The counter-sentinel therefore checks **global properties no single-file diff can satisfy alone**: cross-file structural identity, a repo-wide suppression census, and pattern bans that name the canonical replacement. This meets the family's earned-abstraction bar (`ai/_inbox/archive/abstractions.md`): the failure class is **recurring, cross-cutting, and silent**.

Build it as the next member of the ADR068 sentinel family, conforming to the shared apparatus **exactly**. Write a recording ADR (house YAML format in `ai/decisions/`, next free number) as the first commit.

---

## House apparatus to conform to — READ ALL OF THESE FIRST

```
src/babylon/sentinels/base.py             # run_sensor(name, gating, advisory, summary) -> 0/1/2; SentinelCheckError; Check = Callable[[], list[str]]
src/babylon/sentinels/_ast.py             # shared static AST helpers — extend, don't fork
src/babylon/sentinels/partition/registry.py  # registry style precedent (module docstring, Final data, layer note)
src/babylon/sentinels/coverage/           # precedent for a static-only sensor
tools/sentinel_check.py                   # CLI dispatcher; direct import for light sensors, lazy shim only for heavy ones
tests/unit/sentinels/                     # test layout; conftest.py (shared_tick NOT needed here — this sentinel is static-only)
pyproject.toml                            # import-linter contract pinning sentinels at layer 0.5 — add the new package per the existing contract style
ai/decisions/ADR068_program17_wave1_sentinels.yaml  # family doctrine: frozen-Pydantic registries, no nightly CI, "fix now then gate", efficacy-proven sensors
```

Family rules that bind this work:

- **Layer 0.5**: `sentinels/clones/` imports nothing above `babylon.models`. Check logic must be pure-Python + `ast` + `pathlib`.
- **Static-only, fast-gate resident**: no engine import, no `shared_tick`, no reference DB. Budget: full run **< 2 s** on the repo.
- **Two tiers via `run_sensor("CLONES", gating, advisory, summary)`**; exit-code contract 0/1/2; `SentinelCheckError` for infrastructure failure (unparseable source), never a false pass.
- **No nightly CI** (standing owner ruling). The alert channel is the dev fast lane + the on-demand CLI.
- **Deterministic output**: sort all finding lines (path, then line, then key) so runs diff stably.
- **Efficacy-proven**: every check must red on an injected defect in its unit tests (house pattern from ADR068 Sensor 3).

---

## Package layout

```
src/babylon/sentinels/clones/
    __init__.py
    registry.py        # declared data ONLY (frozen Pydantic rows in Final tuples + tunables)
    checks.py          # sensor implementation + main(argv) -> int
reports/clone-punchlist.md    # self-regenerating advisory report (--report)
tests/unit/sentinels/test_clones.py
```

CLI: register `"clones"` in `tools/sentinel_check.py` (direct import — this sensor is light). Task: add `check:clones` beside `check:seams` in the task registry (locate where `check:seams` is defined and colocate). Wire the gating run into the dev fast lane the same way the other static sentinels run there.

All check functions take an explicit `root: Path` parameter (defaulting to the repo `src/babylon`) so efficacy tests can point them at `tmp_path` fixtures.

---

## Registry (`registry.py`)

Frozen Pydantic models (ADR068 owner ruling), instances held in `Final` tuples, partition-registry docstring style:

```python
class CloneBlessing(BaseModel):      # model_config = ConfigDict(frozen=True)
    group_id: str                    # "CLONE-0001"
    kind: Literal["fingerprint", "name"]
    key: str                         # digest hex, or "name/arity" for the name net
    members: tuple[str, ...]         # "babylon.engine.systems.struggle::_find_entity_by_role"
    reason: str                      # why this duplication is blessed rather than consolidated
    blessed: str                     # ISO date

class SuppressionBudget(BaseModel):  # frozen
    marker: str                      # "noqa:C901" | "noqa" (bare) | "type-ignore" | "pragma-no-mutate"
    scope: str                       # top-level package dir under src/babylon: "engine", "models", ...
    count: int

class TombstoneRule(BaseModel):      # frozen
    rule_id: str                     # "CT-001"
    pattern: str                     # regex, matched per source line
    scope_glob: str                  # "src/babylon/engine/**"
    canonical: str                   # the blessed symbol/idiom — MUST be non-empty
    message: str
    adr_ref: str
    active: bool
```

Tunables as `Final`: `FINGERPRINT_NODE_FLOOR: Final[int] = 10`; scan excludes (`__init__.py`, non-`.py`).

---

## Sensor 1 — Fingerprint (structural clones)

**Normalizer** (extend `sentinels/_ast.py`): for every `FunctionDef` / `AsyncFunctionDef` in every module under `src/babylon/` (excluding `__init__.py`; `tests/` is out of scope at birth — see non-goals):

1. Strip a leading docstring `Expr`; drop `decorator_list` (so `@staticmethod` variants — e.g. `sovereignty._extract_tick` — compare on equal footing with free functions).
2. Canonicalize names positionally via a scoped `NodeTransformer`: args → `a0, a1, …`; locally-bound `Name`s → `l0, l1, …` by first occurrence (rename `Store` bindings and their `Load` references; leave attributes, globals, imports, and keyword-arg names untouched).
3. Keep annotations and constants verbatim (semantic; `0xBA1AC1A`-style differences must break a match).
4. Fingerprint = `sha256(ast.dump(node, annotate_fields=False, include_attributes=False))`.
5. Skip functions with total AST node count < `FINGERPRINT_NODE_FLOOR`. Do **not** use a statement-count floor: the one-statement shims (`return int(context.get(...) if isinstance(...) else ...)`) are node-dense and are precisely the target; trivial one-line property returns are not.

**Finding rule**: a digest whose members span **≥ 2 distinct modules** is a clone group.

**Secondary name net** (high recall where the smell breeds): a `_`-prefixed function name with identical arity defined in ≥ 2 modules under `src/babylon/engine/systems/`, regardless of digest — this is what catches the near-clones exact hashing misses (the five `_extract_tick`s are *not* byte-identical: docstring and staticmethod variants).

**Tiers** (the "fix now then gate" flow):

- *Advisory*: any fingerprint or name group not present in the blessings registry. Surfaces in the punch-list for owner triage → consolidate it, or bless it with a `reason`.
- *Gating*: a `CloneBlessing` whose members no longer all exist (stale baseline — the registry only shrinks: "tighten the baseline", ADR037 skip-remediation pattern).

Seed blessings at land time by running the scanner against HEAD and triaging with the owner — do **not** silently bless everything found; each row needs a real `reason`. Groups already scheduled for consolidation by the companion refactor spec get **no blessing** (they stay advisory until deleted).

---

## Sensor 2 — Suppression ratchet

Silence is a budgeted resource. This is the check that would have prevented the 586-line converter: ADR028 complexity gating existed and was defeated by `# noqa: C901`.

**Census**: per (marker, top-level dir under `src/babylon`), count occurrences of: `# noqa: <CODE>` (split comma-separated codes, ruff style — `# noqa: E741, ARG002` counts each), bare `# noqa`, `# type: ignore` (with or without `[code]` — one marker), `# pragma: no mutate`. Line-regex based; tolerate spacing variants.

**Tiers** — gating in **both** directions:

- observed > budget → `"suppression budget exceeded for {marker} in {scope}: {n} > {budget} — remove the marker or raise the budget row with a reason (owner call)"`.
- observed < budget → `"suppression budget stale for {marker} in {scope}: {n} < {budget} — ratchet the registry down"`.

Seed budgets from a fresh census at implementation time. For scale (session census, `src/babylon`, re-count before pinning): `S608` 34, `ARG002` 29, `SLF001` 22, `E741` 20, `ARG001` 17, `C416` 12, `BLE001` 8, `C901` 6; `pragma: no mutate` present in 11 files.

---

## Sensor 3 — Tombstones

Every consolidation refactor deposits a rule when it lands: the killed pattern may never re-enter, and the failure message **names the canonical replacement**. For an AI-maintained codebase the message *is* the mechanism — it lands in the next agent's context and steers it to the import instead of the reinvention. A rule with an empty `canonical` field is invalid (enforce in the model with a validator).

*Gating* when `active`; inactive rules are listed in the punch-list as "pending refactor". Violation line format:

```
CLONES VIOLATION [tombstone CT-003]: src/babylon/engine/systems/foo.py:88 matches 'def _coerce_role' — use SocialRole.coerce (ADR0xx)
```

**Seed rules** (activation is repo-state-dependent — probe before setting `active`; the companion refactor spec flips these on as its phases land):

- `CT-001` — pattern `isinstance\(context, dict\)`, scope `src/babylon/engine/**`, canonical `TickContext (context is always a TickContext)`. Active iff `rg` finds zero current matches.
- `CT-002` — pattern `def _extract_tick|def _extract_persistent`, scope `src/babylon/**`, canonical `context.tick / context.persistent_data directly`. Same activation probe.
- `CT-003` — pattern `def _coerce_role`, scope `src/babylon/**` excluding `models/enums/`, canonical `SocialRole.coerce`. Same activation probe.

---

## Punch-list report

`tools/sentinel_check.py clones --report` regenerates `reports/clone-punchlist.md` (seam-punch-list style): a header with the regeneration command, then advisory findings grouped by sensor — clone groups with `module::qualname (file:line)` members, inactive tombstones, and nothing else. `--check` never writes files; `--report` implies a check run first.

---

## Non-goals

- **No token-similarity / jscpd / pylint-R0801.** Exact-AST + name-collision + tombstones is the operating point; similarity thresholds trade precision for noise, and every consolidated family becomes exactly-guarded via its tombstone anyway.
- **No `tests/` scope at birth** — test code duplicates legitimately (fixtures, arrange blocks); note it in the ADR as a possible follow-on with a higher floor.
- **No auto-fix / rewriting.** Detect and point; never mutate source.
- **No nightly CI, no cron** (standing owner ruling).
- **No `web/` frontend scope** — TypeScript needs a different parser; out.

## Acceptance

1. Recording ADR committed first; then the package, CLI verb, task wiring, punch-list, and tests — conventional commits on a feature branch off `dev`.
2. Efficacy reds, each as a unit test against `tmp_path` fixtures: planted twin functions in two modules → fingerprint advisory; planted same-name `_helper` ×2 under a fake `engine/systems/` → name-net advisory; planted `# noqa: C901` over budget → ratchet violation; under-budget census → stale-budget violation; planted `isinstance(context, dict)` against an active `CT-001` → tombstone violation whose message contains the canonical symbol; a blessing with a missing member → stale-baseline violation.
3. Clean run on HEAD after baselines are seeded and triaged: exit 0 from `poetry run python tools/sentinel_check.py clones --check`, advisory findings (if any) enumerated in a committed `reports/clone-punchlist.md`.
4. Exit-code contract honored (0/1/2); unparseable source → `SentinelCheckError` → exit 2, proven by test.
5. Suite green, mypy/ruff clean, import-linter green (layer 0.5 pinned), `qa:regression` untouched — the sentinel adds zero runtime imports to the engine. Full sentinel run < 2 s, measured and stated in the PR body.
