# Spec Prompt — Second-Generation Duplication Sweep (`engine/systems`)

**Verified against:** `dev` @ `744f865` (2026-07-17). Re-verify every file:line below before editing — this codebase moves fast.

---

## Mission

The big abstraction already landed: `kernel/system_base.py` (ADR-003, Spec 059) lifted `name` / `_read` / `_publish` / `_wrap_graph` / `resolve_rng`, and all 31 system classes inherit it. What remains is **second-generation duplication** that grew after that lift, plus one structural consolidation. Four phases, strictly ordered, each independently landable:

1. **ContextType collapse** — delete the dict/TickContext dual-type and its 8 copy-pasted shim functions.
2. **Event-converter registry** — turn the 586-line if/elif in `simulation_engine.py` into a dispatch table. **Coverage stays exactly 47 EventTypes** — widening is owner-gated (ADR068 §d).
3. **Node-payload helper lift** — consolidate `_coerce_role` ×3, `_get_class_consciousness_from_node` ×3, `_find_entity_by_role` ×2, and the 4 clamp-and-rebuild updaters into canonical homes.
4. **Declarative system ordering** (ADR-gated) — collapse the three parallel ordering representations into ClassVar declarations on the systems themselves.

Phases 1–3 are **byte-identical refactors**: `qa:regression` must pass 5/5 byte-identical after each. Phase 4 changes registration mechanics only; tick output must also remain byte-identical.

---

## Process constraints (non-negotiable)

- Branch `refactor/systems-dedup` from `dev`. Never commit to `dev`/`main` directly. Conventional commits, one per phase minimum (`refactor(engine): ...`, `docs(adr): ...`).
- Gates after **every** phase: full test suite green, `mypy` clean, `ruff` clean, `qa:regression` 5/5 byte-identical, import-linter contracts green.
- TDD for any *new* code (the builder registry, `_write_clamped`): red test first.
- Amendment Q: tests pin what the system *does*. Do not weaken, skip, or delete an existing behavioral test to make a phase pass. If a test breaks, the refactor is wrong or the test was pinning the deleted shim — distinguish explicitly in the commit message.
- Loud Failure (III.11): no new silent fallbacks. Phase 2 preserves the existing `None`-for-unregistered behavior exactly (it is a known, owner-triaged gap — do not "fix" it here).
- No MVP cuts within a phase: each phase ships complete or not at all.

---

## What already exists — READ ALL OF THESE FIRST

```
src/babylon/kernel/system_protocol.py     # System Protocol + ContextType union (Phase 1 root cause)
src/babylon/kernel/system_base.py         # SystemBase: _read/_publish/_wrap_graph/resolve_rng (Phase 3 lands beside these)
src/babylon/engine/context.py             # TickContext: has .tick, .persistent_data, __getitem__, __setitem__, __contains__, .get, extra="allow"
src/babylon/engine/simulation_engine.py   # 1168 lines; run_tick context branch ~L246-252; _convert_bus_event_to_pydantic ~L508+ (586 lines); _DEFAULT_SYSTEMS ~L382-417; partition frozensets + import-time assertions ~L430-500
src/babylon/engine/phase.py               # Spec-040 Phase IntEnum — vestigial (Phase 4 investigates)
src/babylon/models/enums/events.py        # EventType — 92 members
src/babylon/models/enums/social.py        # SocialRole (Phase 3: coerce() lands here)
src/babylon/engine/systems/               # 32 modules, ~11k LOC
```

Duplicate-site census (Phase 1 and 3 targets — re-verify with `rg`):

```
_extract_tick                      collapse_transition.py:285  doctrine.py:312  faction_influence.py:251  reactionary.py:346  sovereignty.py:140 (staticmethod)
_extract_persistent                collapse_transition.py:289  faction_influence.py:255  sovereignty.py:145 (staticmethod)
_coerce_role                       epistemic_horizon.py:50  reactionary.py:350  wealth_distribution.py:88
_get_class_consciousness_from_node economic.py:23  solidarity.py:43  struggle.py:78
_find_entity_by_role               decomposition.py:52  struggle.py:196
clamp+rebuild updaters             solidarity.py:66 (_update_ideology_class_consciousness)
                                   struggle.py:100/132/164 (_update_class_consciousness / _update_national_identity / _update_agitation)
                                   (all four carry `# pragma: no mutate — node updater (clamp + dict rebuild)`)
```

---

## Phase 1 — ContextType collapse

**Root cause.** `kernel/system_protocol.py`:

```python
ContextType = Union[dict[str, Any], "TickContext"]
```

Production constructs exactly one `TickContext` (`simulation_engine.py:1123`). The `dict` arm exists only for legacy test fixtures, yet it taxes every consumer: 5 copies of `_extract_tick`, 3 of `_extract_persistent`, and `run_tick`'s own hasattr/isinstance triple-branch (~L246-252). `TickContext` already implements the full dict-style surface (`get`, `__getitem__`, `__setitem__`, `__contains__`, `extra="allow"`), so the shims defend against a caller that should no longer exist.

**Steps.**

1. Narrow the alias **in place** — do NOT move `TickContext` into the kernel (it runtime-imports `babylon.models.enums`, and kernel < models per the Program-14 layering; the current `TYPE_CHECKING`-only forward-ref pattern is how the layering is honored). Keep the name `ContextType` so all 34 importing modules stay valid:

   ```python
   # kernel/system_protocol.py — before
   ContextType = Union[dict[str, Any], "TickContext"]
   # after (PEP 613 string alias; TickContext stays TYPE_CHECKING-imported)
   ContextType: TypeAlias = "TickContext"
   ```

2. Migrate test fixtures **before** deleting shims. Census at spec time: **41 test files** pass raw-dict contexts (`rg -l '\.step\([^)]*\{|context\s*=\s*\{' tests/`). Mechanical rewrite: `{"tick": 5, ...}` → `TickContext(tick=5, ...)`; for opaque legacy dicts, `TickContext(**d)` works because `extra="allow"`. Suite must be green here, with shims still present.

3. Delete the shims and their call-site indirection: all 5 `_extract_tick`, all 3 `_extract_persistent` (sites above). Call sites become `context.tick` / `context.persistent_data`.

4. Simplify `run_tick`'s branch to `tick = context.tick`. Sweep for stragglers: `rg 'isinstance\(context, dict\)' src/` must return zero.

5. Update the `System` protocol / `SystemBase.step` docstrings (they still advertise dict support).

**Gate.** Suite green, mypy/ruff clean, `qa:regression` 5/5 byte-identical. Commit: `refactor(engine): collapse ContextType to TickContext, delete 8 context shims`.

**Tombstone deposit (cross-ref clone-sentinel spec):** if `babylon.sentinels.clones` exists, activate rules `CT-001` (`isinstance(context, dict)` banned under `src/babylon/engine/**`) and `CT-002` (`def _extract_tick` / `def _extract_persistent` banned repo-wide); else append the rules to that spec's seed list as a TODO note in the commit body.

---

## Phase 2 — Event-converter registry

**Root cause.** `_convert_bus_event_to_pydantic` is 586 of `simulation_engine.py`'s 1168 lines: a manual if/elif over 47 of 92 `EventType`s, whose docstring is a changelog of monotonic growth (34→44→47). Every new event edits a half-thousand-line function.

**Steps.**

1. New module `src/babylon/engine/event_builders.py`:

   ```python
   EventBuilder = Callable[[Event], SimulationEvent]
   EVENT_BUILDERS: Final[Mapping[EventType, EventBuilder]] = MappingProxyType({...})
   ```

   One small builder function per currently-handled type, bodies moved **verbatim** from the branches (same `payload.get` defaults, same field order). Move the growth-changelog docstring here.

2. `_convert_bus_event_to_pydantic` shrinks to: normalize the string→enum (keep the existing `ValueError → None` behavior), `builder = EVENT_BUILDERS.get(event_type)`, `return builder(event) if builder else None`.

3. **Do NOT add builders for the other 45 EventTypes.** That vocabulary triage is a pending owner product decision (ADR068 §d). Add a module-docstring line stating exactly this, with the ADR reference, so the next agent doesn't "helpfully" widen it.

4. Unit test (red-first): registry covers exactly the 47 types the old function handled (pin the set literal in the test); one round-trip test per representative event category may be added, but do not duplicate existing converter tests — find and re-point them.

**Gate.** As Phase 1. Commit: `refactor(engine): extract event-builder registry from _convert_bus_event_to_pydantic`. Expected diff shape: `simulation_engine.py` loses ~570 lines.

---

## Phase 3 — Node-payload helper lift

**Root cause.** Graph node attrs are `dict[str, Any]` while the Ledger mandates frozen Pydantic, so systems re-derive defensive read/write helpers at that seam. Canonical homes, chosen to respect layering:

1. **`SocialRole.coerce(raw: object) -> SocialRole | None`** — classmethod on the enum in `models/enums/social.py` (it needs `SocialRole` at runtime, so models is the only legal floor). Delete the 3 copies; call sites become `SocialRole.coerce(raw)`.

2. **`_get_class_consciousness_from_node` ×3 and `_find_entity_by_role` ×2** — read the three implementations of each first; they may differ in defaults/signature. Lift the true common core into `src/babylon/kernel/node_access.py` (layer-legal: they operate on payload dicts / `GraphProtocol`, taking any enum values as *parameters* — no runtime models import; mirror `system_base.py`'s TYPE_CHECKING pattern for annotations). If an implementation genuinely diverges, keep the divergent one local and say so in the commit body — do not force-unify different behavior.

3. **`SystemBase._write_clamped(graph, node_id, key, value, *, lo=0.0, hi=1.0)`** — read the four `pragma: no mutate` updaters first and confirm they share the clamp-then-rebuild-dict shape. If yes, lift once onto `SystemBase` (beside `_read`), delete the four, and drop their `pragma: no mutate` markers (the shared helper gets a real unit test instead — red-first, including both clamp bounds). If the shapes diverge materially, lift only the shared core and report the divergence.

4. Leave `_get_float` (`dispossession_events.py:134`, single site) unless it merges naturally into `node_access.py` with zero call-site behavior change.

**Gate.** As Phase 1. Commit: `refactor(engine): lift node-payload helpers to canonical homes (SocialRole.coerce, kernel.node_access, SystemBase._write_clamped)`. Tombstone deposit: `CT-003` (`def _coerce_role` banned outside `models/enums/` → canonical `SocialRole.coerce`), same conditional handling as Phase 1.

---

## Phase 4 — Declarative system ordering (ADR-gated)

**Root cause.** Ordering is declared in three places: (a) the literal `_DEFAULT_SYSTEMS` list whose fractional *comments* (2.5, 14.5, 17.4, 17.8, 20.5, 21.5) are the de-facto schema; (b) the Spec-056 partition frozensets (`MATERIAL_BASE_SYSTEMS` / `ACTION_PHASE_SYSTEMS` / `CONSEQUENCE_SYSTEMS`) with import-time assertions; (c) the Spec-040 `Phase` IntEnum in `engine/phase.py`, which exactly one system (`production.py:75-83`, `self.phase = Phase.PRODUCTION`) still sets and — verify this — nothing consumes. Adding a system is currently a four-edit ritual.

**Step 0 — write the ADR first and commit it before any code.** The ordering *is* the theory (materialist causality); changing where it is declared is an architectural decision. The ADR must record: the new ClassVar scheme, that runtime order is unchanged, the disposition of `engine/phase.py`, and that the Spec-056 assertions are retained as derived cross-checks. Follow the house ADR YAML format in `ai/decisions/`; take the next free number.

**Steps.**

1. New enum `TickPartition` in the kernel: `MATERIAL_BASE = 0, ACTION = 1, CONSEQUENCE = 2` (do **not** reuse the name `Phase` — avoid collision with the enum being retired).
2. Each of the 27 registered systems declares two ClassVars: `partition: ClassVar[TickPartition]` and `position: ClassVar[float]`, values transcribed **exactly** from the current list order and its fractional comments.
3. Engine derives everything: `_DEFAULT_SYSTEMS = sorted(instances, key=lambda s: s.position)`; the three partition sets derive from the `partition` ClassVar. Keep the Spec-056 import-time assertions, now checking the *derived* sets (completeness, disjointness, and the new invariant: position order never interleaves partitions out of MATERIAL_BASE → ACTION → CONSEQUENCE order).
4. Pin the current order as a golden: a unit test asserting the derived sequence of class names equals the frozen literal list from `744f865`. This is the byte-identity insurance.
5. `engine/phase.py`: `rg '\.phase\b'` for consumers of the system-level attribute. If none beyond `production.py`'s own assignment, retire the enum and the assignment in the same commit, noting it in the ADR. If consumers exist, stop, report, and leave it — do not guess.

**Gate.** As Phase 1, plus the golden-order test. Commits: `docs(adr): ADR0xx declarative system ordering` then `refactor(engine): derive system order and partitions from ClassVar declarations`.

---

## Hard non-goals — do not touch, even if adjacent

- **No EventType coverage widening** (47 stays 47) and **no emission-at-source rework** of `SystemBase._publish` / the EventBus payload shape — both are follow-on ADRs.
- **No decomposition** of `economic.py` (824), `struggle.py` (718), `community.py` (665) for LOC aesthetics.
- **No `ShadowSystem` base class** for MarketScissors / WealthDistribution / EpistemicHorizon.
- **No ServicesProtocol / optional-calculator rework** (the 77 `is None` guards stay).
- **No `edge_transition/_legacy.py` split** — pre-existing sanctioned TODO in its own `__init__` docstring, separate ticket.
- **No test-suite restructuring** beyond the Phase-1 fixture migration.

## Overall acceptance

1. Four phases landed as ≥5 conventional commits on `refactor/systems-dedup`; PR targets `dev`.
2. `rg 'isinstance\(context, dict\)' src/` → 0; `rg 'def _extract_tick|def _extract_persistent|def _coerce_role' src/babylon/engine/` → 0; `_convert_bus_event_to_pydantic` ≤ 30 lines.
3. Suite green, mypy/ruff/import-linter clean, `qa:regression` 5/5 byte-identical at **every** phase boundary, golden-order test green.
4. A closing summary in the PR body listing: lines deleted per phase, any divergent-helper exceptions kept local (Phase 3), the `engine/phase.py` disposition (Phase 4), and the tombstone rules deposited or queued.
