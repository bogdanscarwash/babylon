# Heavy-Tier CI Triage — 2026-07-15

Run: `main.yml` #29462198728 (manual `workflow_dispatch` against consolidated `dev`, 193 commits
since the last heavy-tier run 2026-07-11). 4 jobs failed + 1 advisory. This report pulls the full
job logs (`gh api repos/percy-raskova/babylon/actions/jobs/<id>/logs`), cross-references the repo
(`git log`, source, `.github/workflows/main.yml`, `.mise.toml`, `tools/make_reference_subset.py`),
and gives a concrete fix per job.

**Bottom line up front:** of the 4 blocking failures, **1 is a genuine dev regression** (Playwright
E2E — a migration-ordering bug), and **3 are stale tests / already-documented data gaps** where dev's
behavior is correct. Nothing here indicates the 193-commit consolidation broke working functionality
beyond the one migration-ordering bug.

---

## 1. Postgres Integration (web bridge) — job `87507886163`

`3 failed, 66 passed, 1 warning in 105.47s`. Two distinct root causes plus one non-blocking noise item.

### 1a. `test_dashboards.py::TestStateApparatusDashboard::test_state_apparatus_dashboard_honest_empty_for_wayne_county`

```
AssertionError: assert [{'id': 'ORG002', 'name': 'Detroit Police Department', ...}] == []
```

**Root cause:** commit `70d6e3f2` *"feat(engine): invoke the state-apparatus AI — Detroit PD seeded,
RuleBasedStateAI executes for the first time (AW3.3)"* (2026-07-15) intentionally seeds `ORG002`
("Detroit Police Department", `org_type=state_apparatus`) into the `wayne_county` scenario at tick 0.
The test (`tests/integration/web/test_dashboards.py:175-189`, from `d7f253de`) asserts
`organizations == []` on the documented premise "wayne_county's sole seeded org is CIVIL_SOCIETY...
no scenario seeds a STATE_APPARATUS org" — a premise AW3.3 deliberately invalidated.

**Verdict: STALE TEST. Dev is correct** — the AI needed a seeded state-apparatus org to act on;
that's the whole point of AW3.3.

**Fix:** update `tests/integration/web/test_dashboards.py::test_state_apparatus_dashboard_honest_empty_for_wayne_county`
to assert the real ORG002 shape (org_count=1, the one dict now observed in the log, `total_repression_budget=100.0`,
`total_heat=0.3`), matching the pattern already used by the neighboring
`test_state_apparatus_dashboard_after_resolves` (same file, lines 191-197). Rename the test (it's no
longer "honest_empty") or add a new genuinely-empty-scenario test elsewhere if that coverage still
matters. **Effort: 10 minutes.**

### 1b. `test_static_economy_flow.py::TestWayneCountyFlowSurvivesWebResolve` — 2 failures

```
test_wage_flow_moves_between_tick_1_and_tick_2:      assert 552006000.0  == 84000000.0 ± 84
test_flow_keeps_accruing_across_a_third_resolve:     assert 1104012000.0 == 168000000.0 ± 168
```

Both tests hardcode (`tests/integration/web/test_static_economy_flow.py:338,354`):
```python
annual_wage = 21.0 * HOURS_PER_YEAR * 100_000.0   # the OLD 100k-employment placeholder
```

**Root cause:** commit `a2ab6e9e` *"feat(economics): wire real per-county employment into the tick
(Program 17 item-25 C)"* (2026-07-12) — "Fix C" from the Program-17 honesty gap ai/state.yaml has been
tracking — wires `SQLiteQCEWCountyNAICSSource` as `employment_source` specifically in the web bridge's
`_bridge_economics_overrides` (`web/game/engine_bridge.py:5898-5899`), replacing the 100k placeholder
with real Wayne County QCEW employment for the web/Postgres path. The test's own docstring
(lines 332-337) explicitly (and now incorrectly) claims this is "byte-for-byte the same formula as
TickDynamicsSystem._accrue_flows... mirroring the headless runner" — true when A7 (`742e7163`) wrote
this test, invalidated by Fix C's *web-bridge-specific* override 5 days later. Note:
`_bridge_economics_overrides`'s own docstring (`engine_bridge.py:5832-5838`) is ALSO stale here — it
still says employment "stays at CountyEconomicState's bootstrap defaults... 100,000 workers," which the
`employment_source` override 60 lines below it (added by the same Fix C commit) no longer makes true.

**Verdict: STALE TEST. Dev is correct** — real per-county employment replacing the honesty-gap
placeholder was the intended, ratified fix (Program 17 item 25).

**Fix:**
1. `tests/integration/web/test_static_economy_flow.py` — stop hardcoding `employment=100_000.0`.
   Derive the expected value from the same `SQLiteQCEWCountyNAICSSource` the bridge now wires (fetch
   Wayne County's real employment for the scenario year in the test setup), or loosen the assertion to
   a behavioral contract ("wage flow uses real employment, is provably not the retired 100k constant")
   rather than a second hardcoded magic number — a raw new constant would silently re-break on the next
   ci-data regen if the QCEW slice changes.
2. `web/game/engine_bridge.py:5832-5838` — fix the now-stale docstring comment in
   `_bridge_economics_overrides` (it currently contradicts its own code 60 lines down).
3. Side note: back-computing the CI-observed employment figure from the wage numbers gives ≈657,000,
   while `a2ab6e9e`'s commit message cites 671,985 for the same county/year from local testing against
   the full reference DB. This ~2% delta may just be rounding in my back-computation, or may indicate
   the `ci-data-v1`/`v2` subset's QCEW slice for Wayne County differs slightly from the full reference
   DB the commit was verified against. Worth a quick check when the ci-data-v3 regen (§4) lands, not
   blocking.

**Effort: 20-30 minutes** (test fix + docstring fix).

### 1c. Non-blocking noise: `Failed to persist hex_latest rows ... Database access not allowed, use the "django_db" mark`

Appears 16 times in the log, always as a caught-and-logged `ERROR`, never a test failure. This is
`_persist_hex_state_safe` (`web/game/engine_bridge.py:7375-7404`, from `39f5781d`, "P0 #7"), a
deliberate "never-raise" best-effort write of `hex_latest` via Django's ORM (`HexState.objects.bulk_create`).
None of the `tests/integration/web/*` tests using the raw-Postgres `bridge` fixture apply
`@pytest.mark.django_db`, so pytest-django's DB blocker always intercepts this specific write, every
time `create_game`/`resolve_tick` runs — a pre-existing, harmless-but-loud gap unrelated to the 3
failures above (it fires on the 66 PASSING tests too, just not printed since pytest only shows captured
output for failures).

**Verdict:** not a regression, not blocking. **Advisory only** — recommend either adding
`@pytest.mark.django_db` to the affected test classes (if `hex_latest` write-through should be
genuinely exercised there) or having `_persist_hex_state_safe` skip the Django-ORM write path when
Django DB access isn't available in-test, to stop cluttering CI logs with an always-caught exception.
Not part of the sequenced fix plan below (no test depends on this succeeding).

---

## 2. Playwright E2E (real loop) — job `87507886180` — **GENUINE DEV REGRESSION**

Fails at the `Django migrate` step, before any browser test runs:
```
django.db.utils.ProgrammingError: relation "class_snapshot" does not exist
  File "web/game/migrations/0003_spec037_simulation_tables.py", line 71, in forwards
    schema_editor.execute(idx_ddl)
```

**Root cause — a genuine migration-ordering bug:**

- `class_snapshot`'s table DDL (`CLASS_SNAPSHOT_DDL`, `src/babylon/persistence/postgres_schema.py:608`)
  is **only** ever executed via the engine's own canonical-schema list (`POSTGRES_SCHEMA_DDL`, line
  1079) — i.e. via `mise run db:bootstrap`. **No Django migration ever runs `CLASS_SNAPSHOT_DDL`.**
  `web/game/migrations/0014_classsnapshot.py` (added by `e284a65e`, W2-R3, 2026-07-15) only registers
  a `managed=False` unmanaged model — Django's schema editor is a no-op for `managed=False` models, so
  this migration writes nothing to the DB; it just assumes the table already exists.
- The SAME commit (`e284a65e`) added 3 new index statements for `class_snapshot` to
  `SPEC037_INDEXES_DDL` (`postgres_schema.py:998-1001`) — a list that is executed, unconditionally, by
  the much-earlier migration `0003_spec037_simulation_tables.py` (line 70-71, `for idx_ddl in
  SPEC037_INDEXES_DDL: schema_editor.execute(idx_ddl)`). Migration 0003 runs at position 3 in the
  chain; `0014` (which would be the natural place to create the table) runs 11 migrations later.
- So on a **fresh** DB bootstrapped purely by `python manage.py migrate` (no engine-schema step first),
  migration 0003 tries to `CREATE INDEX ... ON class_snapshot` before *anything* has created
  `class_snapshot` — crash.
- Every OTHER Spec-037 table this same migration 0003 indexes (`territory_snapshot`, `org_snapshot`,
  `edge_snapshot`, `hex_substrate`, `hex_latest`, `simulation_event`, ...) has its `CREATE TABLE`
  executed directly inside 0003's own `forwards()` (lines 39-60) — `class_snapshot` is the one
  exception where the index landed in the shared list but the table-creation didn't.
- This is exactly why `postgres-integration` and `qa-e2e-regression` jobs (which run
  `mise run db:bootstrap` — i.e. the full `POSTGRES_SCHEMA_DDL`, including `CLASS_SNAPSHOT_DDL` — as an
  explicit step **before** `Django migrate`, see `.github/workflows/main.yml:164-165` and `310-311`)
  don't hit this: the table already exists by the time migration 0003 runs. `playwright-e2e`
  (`main.yml:192-286`) is the one heavy-tier job with **no** "Apply engine schema (canonical DDL)" step
  before `Django migrate` (`main.yml:216-224`) — and migration 0003's own docstring
  ("Django uses unmanaged models... This migration executes raw SQL to ensure the tables exist... 
  regardless of whether the engine has already initialized the schema") states that `Django migrate`
  alone is supposed to be sufficient. `e284a65e` broke that self-sufficiency contract for
  `class_snapshot` specifically.

**Verdict: DEV REGRESSED.** `e284a65e` (2026-07-15, class_snapshot / W2-R3) introduced a real
schema-authoring bug — it split "create the table" and "index the table" across two migrations without
also fixing table-creation, breaking Django-migrate-only bootstraps (which the E2E job, and any future
production Django-only deploy, relies on).

**Concrete fix (two files, both currently touchable — `0014` was authored today and has not shipped to
any real deployment, only fresh CI databases):**

1. `web/game/migrations/0014_classsnapshot.py` — add a `migrations.RunPython(forwards, backwards)`
   operation (mirroring migration 0003's own pattern) that executes `CLASS_SNAPSHOT_DDL` plus the 3
   `class_snapshot` index statements, guarded by the same
   `if schema_editor.connection.vendor != "postgresql": return` idempotency check 0003 uses.
2. `src/babylon/persistence/postgres_schema.py:998-1001` — remove the 3 `class_snapshot` index lines
   from `SPEC037_INDEXES_DDL` (migration 0003 must not touch a table that doesn't exist yet at
   migration-0003 time); they move into the new `0014` `RunPython` step from (1).

This restores "Django migrate alone is sufficient" without needing to touch `main.yml` at all. (A
belt-and-suspenders alternative — adding the missing "Apply engine schema" step to `playwright-e2e`,
mirroring `postgres-integration`'s `main.yml:164-165` — would also unblock this specific job, but would
NOT fix the underlying self-sufficiency contract migration 0003 is documented to guarantee, so it's a
workaround, not the real fix. Recommend the migration fix as primary.)

**Effort: 30-45 minutes** (small migration + DDL-list edit; verify locally against a fresh Postgres
container before pushing — `docker compose up -d babylon-pg && mise run db:bootstrap`-free
`python manage.py migrate` from zero).

---

## 3. Reference-Data Tests (ci-data-v1 subset) — job `87507886160`

```
test_imperial_rent_real_wiring.py::test_tick_dynamics_computes_real_varying_phi_hour_across_counties
AssertionError: {'26163': 0.0, '26125': 0.0, '26099': 0.0}
```

**Root cause — already fully diagnosed and documented in the test's own docstring** (this test was
added by `5e569fcf`, "Program 17 / Item 1a", and is explicitly labeled **KNOWN-RED**):

> `fact_bea_io_coefficient` has 131,239 rows split only between `table_type='USE'` (57,876) and
> `'TOTAL_REQ'` (73,363) — ZERO rows with `table_type='IMPORT_USE'`, for any year. `DBImportShareSource`
> therefore always computes `m_j = 0.0`... which zeroes `phi_vector`, which zeroes every county's
> `tick_phi_hour` — deterministically... A loader for this exact data exists
> (`tools/ingest_bea_imports.py`, writes `table_type='IMPORT_USE'`) and its source archive IS present
> in the trove... it was simply never run against the canonical reference DB.

Important nuance: this is **not** a subset-generator policy gap — `tools/make_reference_subset.py:346`
already marks `fact_bea_io_coefficient` as `"full"` (copies every row). The IMPORT_USE rows don't exist
in ANY reference DB, canonical or subset, because the *loader itself* was never run against the
canonical DB before this test was written. A ci-data regen alone will not fix this unless the regen
pipeline first runs `tools/ingest_bea_imports.py` against the canonical DB / trove archive.

**Verdict: PRE-EXISTING, ALREADY-DOCUMENTED DATA GAP. Not a regression** — the test has been red since
the hour it was created (2026-07-12), by design, per Constitution III.11 (loud failure over hiding the
gap). It is simply not marked `xfail`, so it shows as a hard CI red instead of an expected/tracked one.

**Fix — two parts:**
1. **Immediate / cosmetic:** mark the test `@pytest.mark.xfail(reason="IMPORT_USE data not loaded — tools/ingest_bea_imports.py never run against the canonical reference DB; see docstring", strict=True)` so CI reflects the already-known state honestly instead of presenting a surprise red on every heavy-tier run. Low effort (5 minutes), makes the signal legible.
2. **Real fix (feeds the ci-data-v3 regen, §4):** run `tools/ingest_bea_imports.py` against the
   canonical `data/sqlite/marxist-data-3NF.sqlite` (source archive already staged at
   `/media/user/data/babylon-data/bea/MAKE-USE-IMPORTS (BEFORE REDEFINITIONS).zip`) to populate
   `IMPORT_USE` rows, then regenerate the ci-data subset — no subset-policy change needed since
   `fact_bea_io_coefficient` is already `"full"`.

**Side note (cosmetic, not blocking):** the job is labeled "Reference-Data Tests (ci-data-v1 subset)"
in `main.yml:325`, but `.github/actions/fetch-reference-db/action.yml:18`'s default `tag` is already
`ci-data-v2` (bumped 2026-07-11 for `fact_faf_commodity_flow`). The job's display name is stale by one
version; worth a one-line rename whenever convenient.

---

## 4. Non-Unit Tests (integration, scenarios, property, contract) — job `87507886122`

```
[gw1/gw3 progress reached 88%, all PASSED/SKIPPED, no assertion failures anywhere in the log]
[3-minute gap with zero test output]
[test:rest-ci] ERROR sh exited with non-zero status: no exit status
##[error]The runner has received a shutdown signal. This can happen when the runner service is
stopped, or a manually started runner is canceled.
```

**Root cause — a KNOWN, already-documented, pre-existing CI infra flake**, not a code regression.
`.mise.toml:187-193` (comment written 2026-07-11, "Program 15") describes this EXACT signature:

> NO coverage on this shard... pytest-cov under `-n 4` xdist instruments all of `src/babylon` in every
> worker, and stacked on the memory-heavy integration/property suites it OOM-killed a worker on the
> hosted runner — a silent SIGKILL ("sh exited with non-zero status: no exit status", no traceback), at
> a DIFFERENT test each attempt (cumulative memory, not a test bug: every test passes standalone)...
> restoring [coverage] needs a proper per-worker memory budget (**owner item 54**).

Coverage was already stripped from this shard as a partial mitigation, but the residual OOM (cumulative
memory growth across ~2,500+ tests over 4 xdist workers with no per-worker memory cap) still occurs —
exactly matching what's in this log: no test assertion failed anywhere in the visible output, progress
died mid-suite, and the runner was reclaimed/shut down.

**Verdict: NOT a regression.** Known, tracked, pre-existing hosted-runner memory ceiling (owner item
54, still open). Unrelated to the 193-commit gap.

**Fix:** re-run the job (transient by nature — dies at a different test each time per the mise.toml
comment). If it recurs often enough to be worth a stopgap before item 54's real fix (a per-worker
memory budget) lands, consider dropping `-n 4` to `-n 2` for `test:rest-ci` in `.mise.toml:200,206` as
a temporary mitigation — halves peak concurrent memory at the cost of wall-clock time. **Effort: 0
minutes (just re-run) or 5 minutes (worker-count stopgap).**

---

## 5. Image Scan (trivy — advisory until postgis bump) — job `87507886183`

`Total: 97 (HIGH: 78, CRITICAL: 19)` on the `postgis/postgis:16-3.5` base image OS packages, plus a
separate `Total: 49 (HIGH: 45, CRITICAL: 4)` on an embedded Go binary. This matches the job's own
label and `main.yml:418-423`'s comment exactly: "Advisory until the postgis base-image bump lands...
upstream postgis:16-3.5 (bullseye) carries ~141 fixable HIGH/CRITICAL today — a red nobody can act on
until the bump." `continue-on-error: true` is already set; the job's "failure" is by design (an
explicit, named advisory, not a hidden one).

**Verdict: no action needed.** Already correctly triaged and tracked by the workflow's own comments;
not part of this triage's scope beyond confirming it's the known/expected state.

---

## Sequenced fix plan

None of these fixes overlap in files with PR #190 (`feature/doctrine-system` → `dev`, the Doctrine Tree
work — `domain/doctrine/`, `defines.yaml`, doctrine tests/frontend). PR #190 is `MERGEABLE` today and
should proceed independently; `main.yml` (the heavy tier) only runs on `main`/`workflow_dispatch`, not
on PRs into `dev`, so nothing here blocks PR #190's merge gate. Recommend a **separate small branch off
`dev`** for this cleanup (not bundled into `feature/doctrine-system`) once PR #190 lands, to keep each
PR's diff legible and avoid entangling doctrine-system review with unrelated CI/test fixes.

**Fully independent (any order, can even be one commit each), no data dependency:**
1. `test_dashboards.py` stale-pin update (§1a) — 10 min.
2. `test_static_economy_flow.py` stale-pin update + `engine_bridge.py` docstring fix (§1b) — 20-30 min.
3. Playwright E2E migration-ordering fix — `0014_classsnapshot.py` + `postgres_schema.py`
   `SPEC037_INDEXES_DDL` (§2) — 30-45 min. **Highest priority — the only real regression, and it's
   currently red on the ONLY job that exercises the real browser-driven game loop.**
4. `test_imperial_rent_real_wiring.py` — add `xfail(strict=True)` (§3, part 1) — 5 min, purely
   cosmetic/honesty, no code behavior change.
5. Non-Unit Tests — just re-run; optionally drop `-n 4`→`-n 2` in `.mise.toml` as a stopgap (§4).
6. `hex_latest` Django-DB-blocker noise cleanup (§1c) — advisory, do whenever convenient.

**Depends on the ci-data-v3 regen (a data/artifact task, not a code fix — owner-scheduled, larger
effort, touches `tools/ingest_bea_imports.py` + subset regen tooling directly rather than application
code):**
7. IMPORT_USE data load + regen (§3, part 2) — this is the REAL fix behind item 4's xfail; only this
   makes `test_tick_dynamics_computes_real_varying_phi_hour_across_counties` actually pass.

**One regen, three riders — confirmed, all three land in the same `ci-data-v3` artifact:**
- **This triage:** `fact_bea_io_coefficient` needs its `IMPORT_USE` rows loaded (run
  `tools/ingest_bea_imports.py` against the canonical DB) before the next subset regen — no
  subset-policy flip needed, the table is already `"full"` in `tools/make_reference_subset.py:346`.
- **Unemployment wiring workstream:** `fact_bls_unemployment_decomposition` needs its
  `tools/make_reference_subset.py:393` policy flipped `"skip"` → `"full"`.
- **Census workstream:** 13 `fact_census_*` tables (currently all `"skip"`, e.g.
  `tools/make_reference_subset.py:394-401`) need their policy flipped `"skip"` → `"michigan"`.

All three are policy/data changes to the same generator (`tools/make_reference_subset.py`) feeding the
same `.github/actions/fetch-reference-db` pin — one coordinated regen (new tag, e.g. `ci-data-v3`, new
asset + sha256 pins in `action.yml`) should carry all three rather than three separate regen cycles.

---

## What genuinely regressed vs what's stale — summary table

| Job | Failure | Verdict | Dev correct? |
|---|---|---|---|
| Postgres Integration | `test_dashboards` ORG002 | Stale test pin | Yes — AW3.3 intentional |
| Postgres Integration | `test_static_economy_flow` ×2 | Stale test pin | Yes — Fix C intentional |
| Postgres Integration | `hex_latest` ERROR noise | Pre-existing test-infra gap | N/A, non-blocking |
| **Playwright E2E** | **`class_snapshot` migration crash** | **Genuine regression** | **No — migration-ordering bug in `e284a65e`** |
| Reference-Data Tests | `phi_hour` all 0.0 | Pre-existing, documented data gap | Yes — code is correct, data absent |
| Non-Unit Tests | Runner shutdown/cancel | Known CI infra flake (item 54) | N/A, unrelated to commits |
| Image Scan | Trivy HIGH/CRITICAL count | Known advisory (postgis bump pending) | N/A, by design |
