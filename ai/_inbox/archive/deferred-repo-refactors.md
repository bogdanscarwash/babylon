# Deferred repository refactors — notes to a future maintainer

**Context (2026-07-12).** Owner authorized subrepositories (overruling ADR063's finer-grained
rejection). Program 18's 3-critic workflow recommended **only** extracting `src/frontend` →
`babylon-cockpit` (public submodule at `src/frontend`), keeping `web/` + `src/babylon` together.
That extraction is being executed now (ADR068). The owner's steer: *minimum refactoring restrained to
this issue; address the rest soon.* This file specs the "rest" so it isn't lost.

Priority order for the future you: **(1) reference-DB reproducibility is a real blocker to the
lit-Φ goal — do it first**; (2) the two-tier web↔engine contract mechanism; (3) the `poetry build`
defect; (4) `babylon_data` as a 2nd submodule (only after 3); (5) `web/` split — likely never.

---

## 1. ⚠️ CRITICAL — make the IMPORT_USE reference-DB load reproducible (NOT a subrepo issue)

Program 17 1a-unblock loaded 31,688 BEA IMPORT_USE io-coefficients into the **local** 5.7 GB reference
DB (`data/sqlite/marxist-data-3NF.sqlite`) via the fixed `tools/ingest_bea_imports.py`. **This mutation
is local-only.** CI fetches its reference DB via `./.github/actions/fetch-reference-db` (a pre-built
"ci-data-v1 subset"); the `@requires_reference_db` crown test (`tests/integration/economics/tick/
test_imperial_rent_real_wiring.py`) runs against it in `main.yml`/`nightly.yml`. So **lit Φ reproduces
nowhere but this machine** — CI, other devs, and the Hetzner deploy all still see blank Φ.
**Spec:** wire `ingest_bea_imports.py` into whatever builds+publishes the reference-DB artifact (the
`fetch-reference-db` source + the `ci-data-v1` subset builder), regenerate + republish the artifact,
and confirm the crown test passes in CI. This is spec-086/097/098 (reference-DB remediation) territory.
Until then, the crown test is effectively local-only and the "map lights up" claim is unshipped.

## 2. The two-tier web↔engine contract mechanism (the real seam fix)

**★ Owner dropped a concrete design for this (2026-07-12): `archive/observability-seam-coverage-gate.md`.**
Frame it as a **declared observable-field registry + a 3-sensor seam-coverage gate** — the generalization
of every seam bug fought live this session (blank Φ; `profit_rate` computed→carried→read but dark on
unseeded employment; events capped 14/88). The registry is one typed source of truth (like `EventType`/
`GameDefines`): each observable declares `name` / engine-source graph-attr / bridge wire-key / frontend
sink / **liveness policy** (must-be-non-null-by-tick-N vs legitimately-null-until-condition). Three sensors
check reality against it: **Sensor 1 continuity** (static set-diff engine→bridge→frontend catches
computed-but-unserialized / serialized-but-unrendered); **Sensor 2 liveness** (run a real tick, assert each
field non-null by its declared deadline UNLESS marked conditionally-dark — this is the one that would have
screamed "Φ blank at tick 1" / "profit_rate dark at tick 60"); **Sensor 3 provenance** (rendered value's
source IS the field it claims — forbids the `extraction_intensity`-painted-as-Φ relabel; needs the
display-reads-only-typed-wire-model rule + golden field-value baselines). Maps onto the two tiers below:
Tier-1 shape = Sensor 1, Tier-2 value/behavior golden = Sensor 2, honesty guard = Sensor 3. Caveats: the
registry is load-bearing (must fail Sensor 1 on unregistered-but-serialized, like `defines.yaml` sync);
"frontend consumed" is rigorous only via generated `@babylon/api-types` (tsc, not grep); curate to
player-observables, not every scratch attr. **This is the highest-leverage anti-regression infra on the
list — it converts "find it live by luck" into "CI trips the instant a layer drops a value."**

Program 18's contract-seam critic proved the recent bugs (blank Φ, `float+=Decimal`, event whitelist,
`None` consciousness) were **web↔ENGINE / value-population** bugs, not web↔frontend *shape* bugs — so a
frontend split doesn't address them, and schema codegen alone wouldn't have caught 3 of 4. Do BOTH tiers
**in the single babylon repo** (web never leaves it), gated in CI:
- **Tier 1 (shape):** a Pydantic wire-model layer `web/game/contracts.py`, built incrementally from the
  highest-traffic endpoints (`state/`, `resolve/`, per-verb actions). Generate OpenAPI from
  `model_json_schema()` (NOT DRF introspection — only ~11 of 45 `_envelope()` sites pass through a
  serializer). `openapi-typescript` → a versioned `@babylon/api-types` npm package published from
  babylon CI, pinned+bumped by `babylon-cockpit`. `oasdiff` CI gate on breaking changes. `game.ts`
  survives as a thin presentation layer over the generated types (tsc becomes the drift detector).
- **Tier 2 (value/behavior):** fix `tests/unit/test_contract_parity.py` (its docstring claims to read
  `src/frontend/src/test/fixtures.ts` but never does) — make it a golden-response suite snapshotting the
  ACTUAL field values a real deterministic tick produces. This is the tier that catches "honestly-typed
  but always-`None`" bugs (blank Φ) that schema tools can't see.
- **`babylon-cockpit` pin-freshness canary:** warn/fail if its pinned `@babylon/api-types` falls N
  releases behind — the one new drift risk the frontend submodule introduces.

## 3. `poetry build` is broken today (pre-existing, independent)

`[tool.poetry] packages` declares `babylon_data` `from = "src"`, but `src/babylon_data` is a symlink
whose realpath escapes the project root (`/media/user/data/babylon-data/`); poetry-core ≥2.2 refuses to
package it → `poetry build` fails reproducibly. Harmless today (babylon is never built as a wheel), but
it **blocks any future engine-wheel** — which a `web/`↔engine split (candidate 5) would require. Triage
on its own terms (e.g. exclude `babylon_data` from the built package, or make it a proper dependency).

## 4. `babylon_data` as a second submodule (only after #3)

`src/babylon_data` already behaves like an external dependency (symlink to an off-repo trove, different
cadence, no code coupling to the web/frontend split). Plausible 2nd submodule. NOT run through the
3-critic gauntlet, and blocked by #3. Spec: fix #3, then evaluate `babylon_data` (and possibly the
reference-DB build from #1) as a `babylon-data` subrepo with its own cadence.

## 5. Splitting `web/` from `src/babylon` — likely NEVER (documented anti-recommendation)

The cleanest *import* cut in the repo (web→engine funnels through `engine_bridge.py`, AST-enforced by
`tests/unit/web/test_import_boundary.py`; zero backward deps) — but Program 18's three critics converged
that splitting it is **net-negative**: it removes the same-repo/same-PR/same-CI safety net from the exact
web↔engine seam that produced 3 of the 4 recent bugs, loses whole-stack `git bisect`, and adds a
cross-repo staleness window to `qa:regression`. Reconsider ONLY with: #3 fixed (engine wheel-able), the
#2 contract mechanism fully in place, and a **measured** (not estimated) cost/benefit — Program 18's own
recon mis-stated its headline "43-of-45 bypass serializers" (~4×) and the EventType enum size three
different ways, so any bigger cut needs verified numbers first. Requires its own constitutional amendment.

## 6. From the ai/_inbox (parsed 2026-07-12)

- **Protocolize the `getattr`-by-string persistence calls** (`protocolization.md`). ~8 sites in
  `engine_bridge.py` do `q = getattr(self._persistence, "query_tick_events", None); if callable(q): ...`
  because `_persistence` is either `PostgresRuntime` (has the method) or SQLite `RuntimeDatabase` (doesn't).
  Pyright flags it (`object|None`), mypy tolerates it — **not a bug, but the exact implicit, drift-silent
  coupling the seam discipline targets** (a rename silently no-ops). Fix: a `TickQueryCapable` Protocol
  next to the existing `RuntimePersistence`/`TraceCollector` in `persistence/protocols.py`; call sites
  become `if isinstance(self._persistence, TickQueryCapable):` → typed + checked. Small self-contained
  refactor; **bundle with the #2 contract-mechanism work**, especially post-split (it becomes the
  published interface across the boundary). Not urgent; discipline not defect.
- **Datasette guardrail for Waves 2–3** (`datesette-developer-view.md`). Datasette fits the **dev /
  Observatory** layer (point it at the reference SQLite / a Postgres read replica) — but it is the WRONG
  tool for the in-game UI: it IS the generic table/facet "spreadsheet overlay" the redesign is escaping,
  and it opens a **parallel raw-table read path** that bypasses `EngineBridge` (would happily surface the
  raw NULL `imperial_rent` or let a chart relabel `extraction_intensity` as Φ — undercutting honest-Φ).
  **Rule: player-facing charts stay native (recharts/DeckGL) reading THROUGH the bridge.** The in-game
  "charts" gap is data-wiring, not a rendering-tool gap. Datasette only ever behind the dev flag.
- **Event routing tables → config plane, Postgres validates only** (`postgres-event-tables.md`,
  parsed 2026-07-14). The event-keyed lookup dicts that drifted (`_EVENT_SEVERITY`, narrator
  template keys, the convert whitelist) are *configuration*, not runtime state — once the
  vocabulary triage (owner item, punch-list "adjacent" section) settles what works, formalize them
  as **one declared source in the config/registry plane** (`GameDefines`/`defines.yaml` siblings or
  the seam registry itself), generated projections everywhere else. Putting the routing rules in
  Postgres would be determinism-adjacent drift, hard to diff, and invisible to the static Sensor-1
  gate. The one genuinely useful Postgres move: make `tick_event.event_type` an **ENUM / FK to a
  reference table generated from the canonical 79** — the DB then *rejects* the `UNKNOWN`/typo
  class at the persistence layer (a 4th sensor), without ever hosting a second editable copy of
  the routing logic. Rule of thumb: **one declared source; Postgres either validates the
  vocabulary or stores event instances — never a second copy of the mapping.**

---
_See ADR069 (repository topology — Program 18 C1) and [[program-17-living-engine]]. Governance:
ADR063 rejected a DIFFERENT (kernel/topology/domain/engine) split — not reopened by any of the above._
