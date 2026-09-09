# Volume III Money Through the Value–Price Scissors — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the fully-built-but-disconnected Volume III financial estate (`domain/economics/{distribution,credit,rent,counter_tendencies,financial_crisis,monetary}`) to the running engine, so the surplus-value ledger closes with an **endogenous** national interest rate (Capital Vol. III Part V; FRED demoted to calibration, U9) and the price⟷value scissors loop closes, bind four new dialectical oppositions, and ship the six sentinel classes this investigation discovered (the sixth — public-surface baseline blindness, U7.11 — added by owner ruling 2026-07-19).

**Architecture:** Five layers stack onto the existing engine without new primitives — Layer 1 (Ledger of Claims, `s = p + i + r + t`) is turned on via `calculator_overrides`; Layer 1b publishes `NationalFinancialParameters` to the graph under a new key so CONSEQUENCE-phase Systems can read it in the same tick; Layer 2 is a pure `domain/` monetary anchor with an honest-absence contract (`NoDataSentinel`, never a fabricated zero); Layer 3 grows the opposition catalog 6 → 10 and gives `CouplingGraph` its first production consumer; Layer 4 closes the scissors loop by tightening `serviceable_divergence` with the real interest burden and pulling the fictitious oscillator toward the anchor where data exists. Past 2024 the oscillator's own dynamics *are* the money system (owner decision D1) — absence is the tested default, covering ~85% of a campaign. The national **interest rate**, by contrast, is endogenous and total from tick 0 (U9): it has no absent state — Marx recognises no natural rate to fall back to (ch. 22).

**Tech Stack:** Python 3.11+, Poetry, Mise, Pydantic (frozen models), rustworkx (`BabylonGraph`), pytest + Hypothesis (property laws), Ruff/MyPy strict, PostgreSQL runtime + SQLite reference DB, FRED/Z.1 federal data adapters.

---

## Global Constraints


House rules — every task's requirements implicitly include all of these:

- **TDD, no exceptions.** Red → green → refactor. A "red phase" means *an assertion fails because production code is wrong* — a missing file, a missing test-file import, or a collection error is **not** a red phase. Where a task's production code already landed in a prior task, inject the defect, prove the red, then `git checkout --` to restore.
  - **Carve-out, added by pre-flight (M1):** for a task whose *deliverable is a new module*, no reachable state produces an assertion-level red — the honest options are a deferred in-function import (still not an assertion) or the collection error itself. Those tasks (U1.2, U3.1) may take the collection error as their red. Do **not** spend execution time manufacturing an artificial assertion. The strict rule holds, unchanged, for every task editing existing production code.
- **Never run the full unit suite.** `mise run check` includes the xdist `test:unit` leg (~1 GB/worker). Use `mise run check:quick` + scoped `mise run test:q -- <path>`.
- **Every commit** uses `mise run commit -- "type(scope): msg"`, conventional format, and ends with `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
  - **Exception, added by pre-flight (M4):** where a commit requires an *exact, curated file list*, use plain `git add`/`git commit` — `mise run commit`'s re-staging sweep is documented to defeat deliberately partial staging. Conventional format and the trailer still apply. This exception exists for U8.5's ceremony commit; do not invoke it elsewhere.
  - **Always verify HEAD moved** after committing (`git log --oneline -1`). The pre-commit hooks can rewrite staged files and abort while printing all-green.
  - **This worktree's `.mise.toml` has been trusted.** If `mise` reports "Config files … are not trusted", run `mise trust` — do not work around it.
- **Never commit an empty index.** If a task's Steps 1–4 modify no file, either produce a real artifact or delete the commit step.
- **Determinism (III.7):** sorted iteration on every new graph/dict traversal; no RNG, no wall clock, no I/O in `domain/`; all mutation via `model_copy(update=…)` on frozen models.
- **Honest absence (III.11):** `NoDataSentinel(fips=, year=, reason=)` with a *specific* reason. Never a fabricated zero, never a default substituted for missing data.
- **Zero inline coefficients (III.1):** every tunable is a `GameDefines` field regenerated into `src/babylon/data/defines.yaml` via `poetry run python tools/generate_defines_config.py`.
- **Layering:** `kernel` < `models`/`formulas` < `topology` < `domain` < `persistence` < `engine`. `domain/` imports nothing from `engine/`. Enforced by `mise run lint:imports`.
- **Line numbers in this plan are authoring-time hints only and are stale by construction** — earlier tasks insert and delete lines in the same files. **Locate every edit site by quoted unique text, never by line number.** If the quoted anchor is not found verbatim, STOP and report the drift; do not work around it.

Binding interface contract — the authoritative names, verbatim:

- `NATIONAL_FINANCIAL_ATTR = "national_financial"` — graph.graph key, defined in `tick/graph_bridge.py` (U3).
- `GraphInputs` new fields (U5), each `float | None = field(default=None)`: `rentier_share`, `debt_ratio`, `credit_fragility`, `financialization_index`.
- Anchor (U4), `src/babylon/domain/economics/monetary/anchor.py`:
  `fictitious_anchor(stock: FictitiousCapitalStock | None, real_output: float | None) -> float | NoDataSentinel`
  `serviceability_anchor(distribution: SurplusValueDistribution | None) -> float | NoDataSentinel`
- FRED fixture (U1): `tests/fixtures/vol3_fred_series.json`, shape `{"<SERIES_ID>": {"<year>": <float>}}`, **ten** series.
- New `MarketDefines` fields (U6): `anchor_pull`, `correction_interest_slope`, `correction_debt_slope`.
- New opposition keys (U5), all `antagonistic=False`: `"surplus_distribution"` (enterprise/rentier, county), `"debt_spiral"` (solvent/indebted, county), `"credit"` (accommodation/fragility, `""`), `"financial"` (real/fictitious, `""`).
- New sentinel packages (U7): `sentinels/{liveness,aggregation,coupling}/`; gate-blindness extends `sentinels/coverage/`; `sentinels/surface/` (U7.11 — the sixth class, public-surface baseline blindness).
- ADR numbers: **U7.10 = ADR088**, **U8.6 = ADR089** (verified: highest existing is ADR081).

---

## File Structure


### `tools/`
| File | Unit | Responsibility |
|---|---|---|
| `tools/export_vol3_fred_fixture.py` *(create)* | U1.2 | One-off exporter: reference DB → deterministic sorted JSON fixture of the ten Vol III FRED series |
| `tools/export_vol3_melt_fixture.py` *(create)* | U1.3b | One-off exporter: reference DB → deterministic sorted JSON fixture of the two national MELT inputs (BEA GDP, QCEW employment) |
| `tools/regression_test.py` | U1.3, U1.3b | The `qa:regression` harness — gains fixture-fed `calculator_overrides`; U1.3b adds the `melt_calculator` key that opens the economics pipeline; stays hermetic (no DB, no drive) |
| `tools/sentinel_check.py` | U7.5, U7.6, U7.8 | Sensor CLI dispatcher — gains `liveness`, `aggregation`, `coupling` entries |
| `tools/capture_qa_diff.py` *(create)* | U8.2 | Captures verbatim `qa:regression` / `qa:e2e-regression` RED output as U8.3's evidence |

### `tests/fixtures/`
| File | Unit | Responsibility |
|---|---|---|
| `tests/fixtures/vol3_fred_series.json` *(create)* | U1.2 | Committed deterministic FRED artifact (D4) — the only money data the hermetic gate sees |
| `tests/fixtures/vol3_melt_national.json` *(create)* | U1.3b | Committed deterministic national MELT artifact (D4) — GDP 2010-2023 + employment 2010-2024; without it `TickDynamicsSystem` returns early and Steps 2-9 never run |

### `src/babylon/domain/economics/`
| File | Unit | Responsibility |
|---|---|---|
| `tick/graph_bridge.py` | U1.5, U3.1, U9.5 | Repoints `tick_ground_rent` at Path A; adds `NATIONAL_FINANCIAL_ATTR` write/read; U9.5 adds the `r`/`s_r` interest observables |
| `tick/system/__init__.py` | U2.1, U2.4, ~~U2.5~~, U3.2, U3.4, U9.7, U9.8 | Year-ceiling fix; magic numbers → defines; `credit_spread` = BAA spread (**already landed** in `aedce819` — U2.5 is complete by supersession); publishes + builds the national financial state; U9.7/U9.8 replace the FRED interest read with the endogenous rate (4-tuple → 3-tuple) |
| `tick/types.py` | U2.1, U9.6 | `NationalTickParameters` loses the `le=2040` ceiling (the live MELT-path crash); U9.6 adds `NationalFinancialParameters.endogenous_interest` |
| `tensor.py` | U2.2 | Adds `MODELED_YEAR_FLOOR`/`CEILING` + `year_within_modeled_range` |
| `credit/interest.py`, `credit/fictitious_capital.py` | U2.2 | Year-window overruns degrade to `NoDataSentinel` instead of raising |
| `credit/types.py` | U2.3, U9.1 | `STAGNATION_CREDIT_GROWTH` becomes the defines-backed accessor `stagnation_credit_growth()` (was a `Final` off a bare `GameDefines()`); U9.1 appends `EndogenousInterestRate` with the `0 ≤ i < r` construction invariant |
| `credit/endogenous_interest.py` *(create)* | U9.3, U9.4 | `endogenous_interest_rate` (bounded share of the average profit rate) + `loan_market_tightness` — pure, engine-free |
| `distribution/calculator.py` | U2.2 | Same year-window guard |
| `distribution/types.py` | U2.2, U2.3 | Year guard; `DEBT_SPIRAL_THRESHOLD`/`DISTRIBUTION_EPSILON` become defines-backed accessors |
| `counter_tendencies/types.py` | U2.3 | `COUNTER_TENDENCY_WEIGHTS`/`IMPERIAL_RENT_REFERENCE_SCALE` become defines-backed accessors |
| `distribution/__init__.py`, `counter_tendencies/__init__.py`, `credit/__init__.py` | U2.3 | Package re-exports + `__all__` carry the accessor names; the deleted constants would otherwise break package import |
| `credit/credit_cycle.py` | U2.3 | Both stagnation transition guards call `stagnation_credit_growth()` |
| `counter_tendencies/calculator.py` | U2.3 | Docstring `:data:` xref repointed to `:func:` (Sphinx `-W`) |
| `factory.py` | U2.4, U3.4 | `create_financial_services` takes `defines`; exposes `credit_aggregate_source` |
| `rent/types.py` | U2.7 | `RentCategory` marked DORMANT with a reason (no data source for the 3-way split) |
| `monetary/anchor.py` *(create)* | U4.1–U4.5 | `fictitious_anchor` + `serviceability_anchor` — pure, honest-absence, engine-free |
| `monetary/__init__.py` | U4.7 | Package-level export of both anchors (the path U6 imports from) |

### `src/babylon/domain/dialectics/`
| File | Unit | Responsibility |
|---|---|---|
| `instances/catalog.py` | U5.1–U5.4, U5.10 | `GraphInputs` +4 fields; 4 new `BoundOpposition`s (6→10); docstring corrected to ten; the five derived coupling edges |

### `src/babylon/config/defines/`
| File | Unit | Responsibility |
|---|---|---|
| `capital_vol3.py` *(create)* | U2.3, U9.2 | `CapitalVolumeIIIDefines` — the Volume III coefficient home; U9.2 adds the four `interest_*` coefficients |
| `_assembler.py`, `__init__.py` | U2.3 | Register + export the new category |
| `market.py` | U6.5 | `anchor_pull`, `correction_interest_slope`, `correction_debt_slope` |
| `src/babylon/data/defines.yaml` | U2.3, U5.6, U6.5, U9.2 | **Regenerated, never hand-edited** — the canonical moddable source of truth |

### `src/babylon/engine/` and `src/babylon/kernel/`
| File | Unit | Responsibility |
|---|---|---|
| `engine/headless_runner/runner.py` | U1.6, U1.7 | Builds `TensorRegistry` + Vol III financial services into `_build_economics_overrides`; threads `scope_fips` |
| `kernel/services.py` | U5.5, U3.4 | `ServicesProtocol` gains `coupling_graph` and `credit_aggregate_source` |
| `engine/services.py` | U5.5, U3.4 | Concrete `ServiceContainer` — same two fields, with an `_UNSET` sentinel so explicit `None` is distinguishable from omission |
| `engine/systems/contradiction.py` | U5.7, U5.8, U5.10 | Computes all four money ratios into `GraphInputs`; coupling direction constrains principal ranking; scales `debt_ratio` by the threshold |
| `engine/systems/market_scissors.py` | U2.6, U6.1, U6.6–U6.8 | Capital-weighted `_mean_profit_rate`; interest-burden + debt terms; anchor pull |
| `src/babylon/formulas/market.py` | U6.2–U6.4 | `calculate_serviceable_divergence` +interest term; new `calculate_correction_severity`, `calculate_anchor_pull` |

### `src/babylon/sentinels/`
| File | Unit | Responsibility |
|---|---|---|
| `report.py` *(create)* | U7.1 | The five-fact agent-legible finding formatter (class, symbol, file:line, problem, remedy) |
| `_ast.py` | U7.2, U7.11 | Shared AST readers: `returned_dict_keys`, `referenced_names`, `unweighted_mean_sites`; U7.11 adds `frozenset_str_members` |
| `seam/registry.py` | U1.5 | Ground-rent row corrected off `NOT_YET_COMPUTED` |
| `liveness/{__init__,registry,checks}.py` *(create)* | U7.3–U7.5 | correct-but-inert + computed-but-never-consumed |
| `aggregation/{__init__,registry,checks}.py` *(create)* | U7.6 | intensive-aggregation (AST) |
| `coupling/{__init__,registry,checks}.py` *(create)* | U7.7–U7.8 | undeclared-coupling, both directions |
| `coverage/{registry,checks}.py` | U7.9 | gate-blindness — `GateEstate` + `check_gate_estate_coverage` |
| `surface/{__init__,registry,checks}.py` *(create)* | U7.11 | public-surface baseline blindness — `check_pinned_surfaces` (the sixth class, a real gate) |

### `web/`
| File | Unit | Responsibility |
|---|---|---|
| `web/game/engine_bridge.py` | U1.8, U2.6 | Extracts `_build_tensor_registry`, wires `tensor_registry` into overrides; corrects two stale Group C/D docstrings |

### Governance, docs, baselines
| File | Unit | Responsibility |
|---|---|---|
| `.mise.toml` | U7.5, U7.6, U7.8 | `check:liveness`, `check:aggregation-intensive`, `check:coupling` — advisory, local-only, never CI |
| `.mise.toml` + `.github/workflows/{ci,main}.yml` | U7.11 | `check:surface` — a real gate folded into `[tasks.check]` + CI (owner ruling 2026-07-19) |
| `docs/reference/sentinel-error-classes.rst` *(create)* | U7.10 | The six classes, each with its sensor and its remedy |
| `ai/decisions/ADR088_sentinel_error_classes.yaml` *(create)* | U7.10 | The sentinel-family decision |
| `ai/decisions/ADR089_vol3_money_scissors.yaml` *(create)* | U8.6 | The Vol III decision |
| `ai/decisions/index.yaml`, `ai/state.yaml` | U7.10, U8.6 | Catalog + truth-status records |
| `reports/vol3-baseline-delta.md` *(create)* | U4.8, U5.9, U8.3–U8.5 | Verification-evidence table, per-scenario delta analysis, **Owner Approval Gate** |
| `tests/baselines/{imperial_circuit,two_node,starvation,glut,fascist_bifurcation}.json` + `dense/*.csv` | U8.5 | Regenerated in the ceremony commit — **only after owner approval** |

### Tests
Unit: `tests/unit/tools/` (fixture export ×2, regression wiring, **gate-opened evidence** (U1.3b), hermeticity, determinism cadence, capture, delta report), `tests/unit/economics/{tick,credit,distribution,counter_tendencies,rent,monetary}/`, `tests/unit/config/`, `tests/unit/dialectics/`, `tests/unit/engine/{systems,headless_runner}/`, `tests/unit/sentinels/`, `tests/unit/web/`, `tests/unit/decisions/`.
Integration: `tests/integration/economics/test_vol3_surplus_distribution_live.py` (U1.9, new).
Property: `tests/property/invariants/test_monetary_anchor_absence.py` (U4.6).

---

## Execution Order


```
U1 → U2 → U3 → U4 → U5 → U6 → U7 → U8
```

**This order is corrected from the orchestrator's original slate, which placed U8 before U7. That was wrong: U8.2 Step 5 states "against the real repo state (U1–U7 already landed)" and U8.6's ADR enumerates U7's sentinel packages as shipped. U8 is terminal by construction — it captures the diff everything before it produced and owns the ceremony.**

Hard constraints, each with the reason it cannot be relaxed:

1. **U2 before every long run.** U2.1 removes `le=2040` from `NationalTickParameters`, which sits on the already-live MELT path. Year 2041 arrives at tick ≈1612 of a 5200-tick campaign. U1's only scenario run is `max_ticks=1`, so U1 → U2 is safe; nothing after U2 may run long without it.
2. **U2 before U3.** U3.2 rewrites the same method U2.5 was written to fix. **U2.5 is already complete by supersession** — commit `aedce819` (a review-fix pass carried out during U2.3) closed the same `credit_spread`/effective-rate conflation, and closed it better. See the supersession note at the head of Task U2.5; do **not** apply U2.5's original brief. The contract U3.2 must preserve is the **4-tuple** `(national_rate, national_spread, fictitious, interest_unavailable_reason)` typed `tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]` — *not* the non-Optional 3-tuple this plan originally specified. The `| None` types are Constitution III.11 honest absence and are load-bearing: U3 must not revert them to fabricated `0.0`s. **Superseded downstream by U9:** this 4-tuple is U3-era; U9 tightens it to the 3-tuple `tuple[float, float, FictitiousCapitalStock | None]` and removes the `| None`s on the rate/spread. U3.2 still lands the 4-tuple; U9.7/U9.8 migrate it.
3. **U3 before U6, not before U4.** U4's anchors are pure functions of an already-resolved `FictitiousCapitalStock` object; they never read `NATIONAL_FINANCIAL_ATTR`. The real U3 consumer is U6.8's `_read_fictitious_anchor`.
4. **U4 before U6.** U6.8 imports `fictitious_anchor`; U6.6 imports `serviceability_anchor`.
5. **U5 before U7.** U7.7's `MEASUREMENT_DEPENDENCIES` declares oppositions U5.2 binds and reads fields U5.7 produces.
6. **U6 before U7.** U7.3's `national_financial` liveness row names `market_scissors.py` as a consumer — only true after U6.8. U7.6's aggregation scan-set is only clean after U6.1 replaces the unweighted `_mean_profit_rate`. If U7.6 reds on that symbol, U6.1 did not land — go fix U6.1; **do not** add an `AggregationExemption` (that would be a false liveness claim).
7. **U3.4 before U5.7.** `credit_state` has no producer anywhere in the codebase today. Without U3.4 the `credit` opposition reads absent forever and the `credit → financial` `transforms` edge permanently demotes `financial` from principal ranking — a silent behavioural bug with no test covering it.
8. **The determinism proof runs FIRST inside U7, not in U8.** Spec §5 hazard 2 is explicit: same-inputs → same-outputs across every construction site must be verified **empirically before U7**, not inferred from reading code (ADR056 precedent: the *planned* proof was wrong and only an empirical run caught it). U8.1 is therefore relabelled **U7.0** and re-run post-regeneration at U8.5 Step 5.

> ## ⛔ HARD STOP — U8.4 OWNER APPROVAL GATE ⛔
>
> **No baseline file may be regenerated until the owner has read `reports/vol3-baseline-delta.md` and recorded verbatim approval in its Owner Approval Gate section.**
>
> Do **not** interpret silence, a general "looks good", or the conversation moving on to the next task as approval. U8.5 Step 1 re-checks with `grep -A1 "Approved by:"` and **halts** if it finds a placeholder. This gate is the strongest part of the plan; it was reviewed by all four auditors and none proposed a change to it. Leave it exactly as written.

---

## Pre-flight


Before Task U1.1, confirm the following in `/home/user/projects/game/babylon-vol3`:

1. **Worktree + branch** — `git branch --show-current` returns `refactor/vol3-money-scissors`. ✅ verified at plan-authoring time.
2. **`data/` symlink farm** — already linked: `data -> /home/user/projects/game/babylon/data`. ✅ verified. U1.1 is therefore a *verification* task, not a repair task; if `ls -la data` shows a live symlink, record that and move on. `mise run doctor` step 3b is the canonical check. U1.2's fixture export reads the reference DB through this link — it is a hard prerequisite for U1.2 and for nothing else.
3. **ADR numbering** — `ls ai/decisions/ | grep -oE 'ADR[0-9]+' | sort -u | tail -1` returns `ADR081`. ✅ verified. U7.10 claims **ADR088**, U8.6 claims **ADR089**. If other work has landed on `dev` since, trust the command's output over these numbers and shift both, preserving the U7-before-U8 ordering.
4. **Editable-venv shadow** — worktrees share the main checkout's editable install. Prefix ad-hoc `python -c` invocations with `PYTHONPATH="$PWD/src"` so you are exercising *this* worktree's source, not `babylon/`'s.
5. **Sentinel package layout** — `src/babylon/sentinels/` currently contains `_ast.py`, `base.py`, `conservation/`, `coverage/`, `dynamic.py`, `partition/`, `roundtrip/`, `seam/`, `synthetic/`. ✅ verified. `liveness/`, `aggregation/`, `coupling/` do not exist and are created by U7.
6. **Resource discipline** — run heavy commands uncapped (earlyoom is the backstop), but **never fan out parallel agents that each spawn pytest**. Scoped `mise run test:q` only.
7. **Spec self-inconsistency, fix independently of the plan** — `docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md:370` says "Twelve of thirteen rows"; the table at `:375-389` has **fifteen** rows. Replace `Defects found during investigation. Twelve of thirteen rows were confirmed verbatim by the` with `Defects found during investigation. Fourteen of the fifteen rows were confirmed verbatim by the`.

---

## A Note On This Document

The audit's correction list and gap-fill tasks have **already been applied** into the unit
tasks below — they are not pending work. What follows is the reconciled plan. The audit
findings themselves are recorded in the Self-Review Record at the end of this document.

One correction was applied by hand after the automated pass: the empirical determinism proof
(spec §5 hazard 2) was cut from U8 for repositioning but its paste into U7 was lost to a
cross-file scope boundary. It is restored as **Task U7.0**, with both of its travelling
corrections applied (a real injected-defect red phase, and the commit trailer).

## Self-Review Record


**What was reconciled across auditors.** Four blockers were found independently by three or four auditors each, which is strong evidence they are real rather than artifacts of one reviewer's model: the `_compute_national_financial_state` signature clash (all four), the missing `credit_state` producer (three), the catalog-docstring double-edit (three), and the ADR088 collision (three). All four are resolved above with a named authoritative unit and exact replacements.

> **Reality correction, 2026-07-18 (post-`aedce819`).** The `_compute_national_financial_state`
> signature clash was resolved *by the code* before the plan's own resolution could be executed: a
> review-fix pass during U2.3 landed the same `credit_spread` fix as a **4-tuple with `float | None`
> honest-absence typing**, superseding U2.5 entirely. The plan's authoritative-unit answer above
> (U2.5 owns the signature, U3.2 preserves it) still holds in *form* — U3.2 is still the unit that
> extends the method — but the shape it preserves is the live 4-tuple, not the 3-tuple the audit
> arbitrated. Task U2.5 now carries a supersession banner; U3.2's `Produces:` block, both test
> code blocks, its red-phase expectation and its implementation block were rewritten against the
> live source. This is the plan's one confirmed case of the "stale by construction" hazard called
> out under **Systemic issue** below actually firing.

**Where auditors disagreed, and how it was decided.** Four genuine conflicts:

1. **`serviceability_anchor`** — names said declare it dormant; coverage said wire it. **Wiring won.** Dormancy would ship a computed-but-never-consumed defect in the same program that adds the sensor for that class, and would leave `correction_interest_slope = 2.0` calibrated for a quantity (i/(c+v)) that no section of the spec asks for. Wiring simultaneously satisfies §3.3, §3.5.1, and U7.8's `check_declared_edges_are_grounded` invariant. names' dormancy row is dropped.
2. **`credit_fragility_scale` placement** — names said move it to `capital_vol3`; coverage said leave it in `TensionDefines` and just document the exception. **The move won.** `default_rate_estimate` — the other factor of the same product — already lives in `capital_vol3`, and splitting one product across two categories is precisely the modding failure the category exists to prevent. coverage's dissent (that §3.5's "every new coefficient becomes a `GameDefines.market` field" scopes only the *scissors* coefficients) is correct as far as it goes but does not argue for `TensionDefines` specifically.
3. **`credit_state`'s producer** — three auditors, three different service APIs, none verified against the code. **No auditor won.** U3.4 ships coverage's version (the only internally complete one) behind a mandatory Step 0 that reads the real signatures first. This is the one place the plan knowingly hands an executor an unresolved question, and it is flagged as such rather than papered over.
4. **U7.9's gate estate** — names proposed `exempt_keys` with a written reason; placeholders proposed dropping the economics estate entirely plus a whole-factory-injection shortcut. **Both were adopted**, because they fix different halves: the shortcut is needed or the *financial* estate reds too, and the exemption is needed or the *economics* estate reds. Spec §4 U7 explicitly sanctions "explicitly narrowed with a reason", which is what `exempt_reason` records.

**Gaps found and filled.** Nine spec elements had no covering task; four warranted new tasks, reproduced in full above: U1.9 (U1's two headline acceptance criteria — `surplus_distribution` non-`None` in a real run and SC-001 — were never proven by anything that ran the engine), U2.8 (the 5200-tick crossing of 2040 was tested only in isolation), U3.4 (`credit_state` had no producer at all), U5.10 (`DEBT_SPIRAL_THRESHOLD` would have remained dead after the program that claims to wire it). The other five gaps were filled with in-place edits: three missing liveness rows, the owed-list test plus its exact `state.yaml` text, the `validate_detroit.py` convergence check, the inline-coefficient grep, the unsorted-hydration fix, and the hazard-3 shadow-accumulator gate.

**Placeholders rewritten.** Roughly 65 placeholder tokens were found. Eleven `...` / `# ...` elisions inside code blocks an agent is told to paste (U2.2 ×3, U2.3 ×3, U2.4 ×3, U2.5 ×1, U6.6 ×1) were converted into precise insert-above-this-anchor instructions. The ~60 `<FILL>` markers in `reports/vol3-baseline-delta.md` gained an actual gate (`test_report_has_no_unfilled_placeholders_outside_the_approval_gate`) — previously only a manual `grep` guarded them, so the committed evidence artifact could have shipped with all sixty and still passed every structural test. Two `<FILL>`s that would have entered immutable history — one in a heredoc commit message, one in an ADR's `consequences:` block, which was 100% placeholder — were replaced with resolved shell variables and authored text respectively. Two "WHAT without HOW" steps in U7.10 (`index.yaml` and `state.yaml` edits described only in prose) gained literal YAML.

**Fabricated values.** U6.8 pinned six 17-digit floats as "captured before the anchor-pull wiring landed" — but no step in U6.1–U6.7 captures them, and `MarketDefines()` defaults are quoted nowhere in the plan. They were authored, not measured. A Step 0 now *measures* them, with an explicit instruction that the captured value is authoritative over the plan's literal. This mattered more than it looks: a single wrong digit would have turned Step 2's red into a false signal, and the natural recovery ("fix the pin to whatever the code emits") destroys the bit-identity test's entire purpose.

**TDD discipline.** Eight tasks had no real red phase. Six told the executor outright to expect PASS on the "verify it fails" step (U1.4, U3.3, U4.2, U4.5, U8.1) or to treat a missing file / a missing test-file `import math` as the red. One (U4.6 Step 2) was self-contradictory and literally unexecutable — it instructed the executor to verify the non-existence of a test, then abandoned the mutation mid-sentence. All eight now inject a specific defect, name the exact expected failure text, and restore with `git checkout --`. Two tasks (U4.8, U5.9) ended with a commit step after Steps 1–4 that modified no file — a commit with an empty index aborts and leaves the plan in a false "done" state; both now produce a real evidence artifact first. Three assertions that pinned nothing were replaced: `assert mock_run.called` (asserts a mock the test installed was called), `assert "vol3" in text.lower()` (vacuously true on a branch named `refactor/vol3-money-scissors`), and a dict-equality check in a test named "deterministic sorted" (dict `==` is order-insensitive).

**Determinism.** Four defects in test/sensor code: a `sorted(set(...))` on a non-unique key in U7.6, whose tie-break falls through to hash-randomized set iteration while `PYTHONHASHSEED` is deliberately unpinned; an `importlib.reload` under `monkeypatch` that leaks a poisoned module-level snapshot to every later test in the xdist worker if the assertion above it fails; an import-time `GameDefines.load_default()` in two `domain/` modules, which reads `defines.yaml` from disk on every process start (including layer-0.5 sentinel processes) and freezes the value before any runtime override can reach it; and one unsorted FIPS hydration. All four are fixed.

**Reordering.** The orchestrator's slate ran U8 before U7, but U8.2 Step 5 says "U1–U7 already landed" and U8.6's ADR enumerates U7's packages as shipped. Corrected to U1→U8 sequential. Separately, spec §5 hazard 2 requires the empirical determinism proof **before U7** — so U8.1 is relabelled U7.0 and runs first inside U7, with a re-run after regeneration at U8.5 Step 5. This is not pedantry: ADR056's precedent is that the *planned* determinism proof was wrong and only an empirical run caught it.

**Systemic issue, mitigated rather than eliminated.** Every task anchors edits to exact line numbers, and four in U5 alone are already wrong against HEAD — worse, they are stale *by construction*, since U5.1 appends fields, U5.2 inserts ~60 lines, and U5.3 replaces the docstring in the same file U5.4 then edits. A global anchoring convention is now stated in Global Constraints and the four worst offenders were retro-fitted with unique-text anchors. The remaining line numbers stay as hints; the convention tells the executor to STOP and report drift rather than work around a missing anchor.

**Categories that found nothing, stated plainly.** Commit hygiene is clean in U1–U7: every commit is conventional-format with the `Co-Authored-By` trailer. All five trailer omissions are confined to U8. The binding interface contract holds byte-for-byte across all eight units for `NATIONAL_FINANCIAL_ATTR`, the four `GraphInputs` field names and their `float | None = field(default=None)` types, both anchor signatures including their `float | NoDataSentinel` returns and parameter order, the FRED fixture path and shape, the three `MarketDefines` field names, all four opposition keys with their poles/levels/`antagonistic=False`, and the sentinel package layout. The `NoDataSentinel(fips=, year=, reason=)` keyword form is identical across U2.2, U3.2, U4.1, U4.3 and U4.4. `create_financial_services`'s new parameter is keyword-only with a default, so U1.3's and U1.6's existing calls stay valid. `_advance`'s new `anchor=None` keyword keeps `_step_county_axes`'s five positional calls valid, as U6.8 claims. And U8.4's owner-approval gate needed no changes at all — all four auditors examined it and one called it the strongest section of the plan: a hard STOP with an explicit anti-inference clause, a re-check in U8.5 Step 1, and a `grep` that halts on a placeholder.

---

### Task U1.1: Verify/recreate the worktree `data/` symlink farm

**Files:**
- Modify (environment only, not repo-tracked): `data/` (repo-root symlink farm into `/media/user/data/babylon-data/`)
- Test: none (this is an environment precondition, not application code — see rationale below)

**Interfaces:**
- Consumes: `tools/data_doctor.sh` (existing, unmodified) — the project's own loud data-pathway gate.
- Produces: a resolving `data/sqlite/marxist-data-3NF.sqlite` path that Task U1.2's export script reads via `babylon.reference.database.get_normalized_session_factory()`, and that Tasks U1.6/U1.8's `@pytest.mark.requires_reference_db` tests need.

This is a one-time environment-setup step, not application code — the repo already ships the exact checker this step must satisfy (`tools/data_doctor.sh`, wired as `mise run data:doctor`). Per the DRY rule ("reuse over recreation"), writing a second pytest reimplementation of that checker would be pure duplication, so this task drives the existing tool directly instead of a fabricated red test.

- [ ] **Step 1: Run the existing data-pathway doctor to see current state**
Run: `mise run data:doctor`
Expected: either `data-doctor: healthy — drive at /media/user/data, trove + symlinks resolve, Postgres on the drive.` (farm already present — skip to Step 5), OR a line starting `DATA-DOCTOR FAIL: repo symlink ... dangles` / `... data/sqlite is a real dir/file, not a trove symlink` (farm missing or broken — continue to Step 3).
- [ ] **Step 2: Confirm the specific failure (only if Step 1 failed)**
Run: `ls -la data/ 2>&1; readlink -e data/sqlite/marxist-data-3NF.sqlite 2>&1`
Expected: `ls: cannot access 'data/': No such file or directory` (farm absent) or a dangling-symlink error from `readlink -e` (farm present but broken).
- [ ] **Step 3: Recreate the farm (only if Step 1 failed)**
```bash
mkdir -p data
for entry in /home/user/projects/game/babylon/data/*; do
  name="$(basename "$entry")"
  target="$(readlink "$entry")"
  ln -sfn "$target" "data/$name"
done
```
This mirrors the main checkout's own farm (`/home/user/projects/game/babylon/data/*`, each entry a symlink into `/media/user/data/babylon-data/<name>`) entry-for-entry, so the worktree gets the identical set of 28 topic symlinks (`bea`, `bls`, `qcew`, `sqlite`, `tiger`, …) rather than a single fragile indirection through the other worktree.
- [ ] **Step 4: Re-run the doctor to confirm**
Run: `mise run data:doctor`
Expected: `data-doctor: healthy — drive at /media/user/data, trove + symlinks resolve, Postgres on the drive.`
- [ ] **Step 5: No commit**
`data/` is untracked local environment state (`git status --short data` reports `?? data`), not a repository change — `tools/data_doctor.sh` explicitly treats it as machine-local. Nothing to stage or commit. Proceed directly to Task U1.2.

---

### Task U1.2: Write the one-off Vol III FRED fixture export script + generate the committed fixture

**Files:**
- Create: `tools/export_vol3_fred_fixture.py`
- Create: `tests/fixtures/vol3_fred_series.json` (generated by running the script against the real reference DB, then committed)
- Test: `tests/unit/tools/test_export_vol3_fred_fixture.py`

**Interfaces:**
- Consumes: `babylon.domain.economics.factory.load_fred_series_from_db(session_factory) -> dict[str, dict[int, float]]` (existing, `src/babylon/domain/economics/factory.py:253-332`); `babylon.reference.database.get_normalized_session_factory`.
- Produces: `tests/fixtures/vol3_fred_series.json` with shape `{"<SERIES_ID>": {"<year>": <float>}}` for the 10 series `FEDFUNDS, DGS10, BAA10Y, TCMDO, GFDEBTN, NCBEILQ027S, B230RC0Q173SBEA, A054RC1Q027SBEA, CPIAUCSL, GDPDEF` — the exact fixture Task U1.3 reads.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the one-off Vol III FRED fixture export script (D4).

Both DB-facing calls are monkeypatched — this test never touches the
reference DB or the babylon-data drive; it only pins export_vol3_fred_fixture
.main()'s JSON shape and determinism (sorted series, sorted years, year keys
stringified for JSON, values passed through untouched).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import export_vol3_fred_fixture as export_mod  # type: ignore[import-not-found]  # noqa: E402


def test_main_writes_a_deterministic_sorted_json_fixture(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """RED->GREEN: main() writes {series_id: {year_str: value}}, sorted."""
    fixture_path = tmp_path / "vol3_fred_series.json"
    monkeypatch.setattr(export_mod, "FIXTURE_PATH", fixture_path)
    monkeypatch.setattr(export_mod, "get_normalized_session_factory", lambda: object())
    monkeypatch.setattr(
        export_mod,
        "load_fred_series_from_db",
        lambda _session_factory: {
            "GFDEBTN": {2020: 27_000_000_000_000.0},
            "FEDFUNDS": {2020: 0.0038, 2019: 0.0225},
        },
    )

    exit_code = export_mod.main()

    assert exit_code == 0
    raw = fixture_path.read_text()
    data = json.loads(raw)
    assert data == {
        "FEDFUNDS": {"2019": 0.0225, "2020": 0.0038},
        "GFDEBTN": {"2020": 27_000_000_000_000.0},
    }
    # Order is the contract, not just content: dict == is order-insensitive.
    assert list(data) == ["FEDFUNDS", "GFDEBTN"]
    assert list(data["FEDFUNDS"]) == ["2019", "2020"]
    assert raw.index('"FEDFUNDS"') < raw.index('"GFDEBTN"')


def test_main_refuses_to_write_an_empty_fixture(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """III.11: an empty query result is a loud failure, never a silent empty file."""
    fixture_path = tmp_path / "vol3_fred_series.json"
    monkeypatch.setattr(export_mod, "FIXTURE_PATH", fixture_path)
    monkeypatch.setattr(export_mod, "get_normalized_session_factory", lambda: object())
    monkeypatch.setattr(export_mod, "load_fred_series_from_db", lambda _session_factory: {})

    exit_code = export_mod.main()

    assert exit_code == 1
    assert not fixture_path.exists()
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_export_vol3_fred_fixture.py`
Expected: FAIL at collection with `ModuleNotFoundError: No module named 'export_vol3_fred_fixture'`
- [ ] **Step 3: Write minimal implementation**
```python
#!/usr/bin/env python3
"""One-off export: Volume III FRED series -> a committed, deterministic fixture.

D4 (docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md): gives
tools/regression_test.py's qa:regression gate real Vol III money data WITHOUT
touching the reference DB / babylon-data drive (standing owner ruling — CI/
tests never touch the drive). This script is the one-off, DB-reading half of
that split; tools/regression_test.py is the hermetic, DB-free half that only
ever reads the committed JSON this script produces — the two must never be
merged back into one module.

Run once (and re-run only when the reference DB's fred_series/fred_national
tables are refreshed):

    poetry run python tools/export_vol3_fred_fixture.py

Prerequisite: the worktree's data/ symlink farm must resolve
(``mise run data:doctor``) — this script opens the reference SQLite DB via
babylon.reference.database.get_normalized_session_factory().
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from babylon.domain.economics.factory import load_fred_series_from_db
from babylon.reference.database import get_normalized_session_factory

FIXTURE_PATH: Path = Path(__file__).parent.parent / "tests" / "fixtures" / "vol3_fred_series.json"


def main() -> int:
    """Query the reference DB and write the committed Vol III FRED fixture.

    :returns: 0 on success, 1 if the reference DB returned zero FRED rows
        (loud failure per Constitution III.11 — never write an empty fixture).
    """
    session_factory = get_normalized_session_factory()
    series = load_fred_series_from_db(session_factory)
    if not series:
        print(
            "export_vol3_fred_fixture: reference DB returned zero Vol III FRED rows",
            file=sys.stderr,
        )
        return 1

    payload = {
        series_id: {str(year): value for year, value in sorted(years.items())}
        for series_id, years in sorted(series.items())
    }
    FIXTURE_PATH.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_PATH.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    print(f"export_vol3_fred_fixture: wrote {len(payload)} series to {FIXTURE_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_export_vol3_fred_fixture.py`
Expected: PASS (2 tests)
- [ ] **Step 5: Generate the real fixture, then commit both**
```bash
poetry run python tools/export_vol3_fred_fixture.py
# Expected stdout: "export_vol3_fred_fixture: wrote 10 series to .../tests/fixtures/vol3_fred_series.json"
git add tools/export_vol3_fred_fixture.py tests/fixtures/vol3_fred_series.json tests/unit/tools/test_export_vol3_fred_fixture.py
mise run commit -- "feat(economics): export Vol III FRED series to a committed fixture (D4)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.3: Wire the committed FRED fixture into `tools/regression_test.py`'s `calculator_overrides`

**Files:**
- Modify: `tools/regression_test.py:71-77` (constants), `:99` (after `create_scenario`, before `CheckpointData`), `:540-615` (`_run_scenario_ticks`)
- Test: `tests/unit/tools/test_regression_test_vol3_wiring.py`

**Interfaces:**
- Consumes: `tests/fixtures/vol3_fred_series.json` (Task U1.2); `babylon.domain.economics.factory.create_financial_services(fred_series_cache=...) -> dict[str, Any]`.
- Precondition (verify before Step 1): `run_scenario` (tools/regression_test.py:618) delegates to `_run_scenario_ticks` (:540). Confirm with `rg -n -A12 "^def run_scenario" tools/regression_test.py`. If it does not delegate, the Step-1 test must call `rt._run_scenario_ticks` directly, and the overrides must be built in whichever function owns the tick loop.
- Produces: `_build_vol3_calculator_overrides() -> dict[str, Any]` — a module-level helper other regression-test tooling can reuse; `step(..., calculator_overrides=...)` now receives it every tick.

- [ ] **Step 1: Write the failing test**
```python
"""D4: tools/regression_test.py's qa:regression harness must thread Vol III
calculator_overrides (built from the committed FRED fixture) into every
step() call — and must do so hermetically, from the fixture file alone.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import regression_test as rt  # type: ignore[import-not-found]  # noqa: E402


def test_run_scenario_passes_vol3_calculator_overrides_to_step(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """RED->GREEN: _run_scenario_ticks must build calculator_overrides from
    the fixture and pass them into every step() call."""
    fixture = {"FEDFUNDS": {"2020": 0.0038}, "BAA10Y": {"2020": 0.021}}
    fixture_path = tmp_path / "vol3_fred_series.json"
    fixture_path.write_text(json.dumps(fixture))
    monkeypatch.setattr(rt, "FRED_FIXTURE_PATH", fixture_path)

    captured: dict[str, Any] = {}

    def _fake_step(
        state: Any, sim_config: Any, persistent_context: Any, defines: Any, **kwargs: Any
    ) -> Any:
        captured.update(kwargs)
        return state

    monkeypatch.setattr(rt, "step", _fake_step)

    rt.run_scenario("two_node", max_ticks=1)

    assert "calculator_overrides" in captured, "step() was not called with calculator_overrides"
    overrides = captured["calculator_overrides"]
    assert overrides.get("distribution_calculator") is not None
    assert overrides.get("interest_calculator") is not None
    assert overrides.get("fictitious_capital_calculator") is not None
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_vol3_wiring.py::test_run_scenario_passes_vol3_calculator_overrides_to_step`
Expected: FAIL with `AttributeError: <module 'regression_test' ...> does not have the attribute 'FRED_FIXTURE_PATH'`
- [ ] **Step 3: Write minimal implementation**

Edit `tools/regression_test.py` — add the fixture-loading helpers right after the module constants (after `TOLERANCE: Final[float] = 1e-5` at line 76):
```python
FRED_FIXTURE_PATH: Final[Path] = (
    Path(__file__).parent.parent / "tests" / "fixtures" / "vol3_fred_series.json"
)


def _load_vol3_fred_fixture() -> dict[str, dict[int, float]]:
    """Load the committed Vol III FRED fixture (D4) — no DB, no drive.

    The JSON on disk stores year as a string (JSON object keys are always
    strings); the Vol III adapters (``FredInterestRateAdapter`` et al.) index
    by ``int`` year, so keys are converted back on load.

    Returns:
        ``{series_id: {year: value}}`` matching
        :func:`babylon.domain.economics.factory.load_fred_series_from_db`'s
        return shape.
    """
    raw: dict[str, dict[str, float]] = json.loads(FRED_FIXTURE_PATH.read_text())
    return {
        series_id: {int(year): value for year, value in years.items()}
        for series_id, years in raw.items()
    }


def _build_vol3_calculator_overrides() -> dict[str, Any]:
    """Build Vol III ``calculator_overrides`` from the committed FRED fixture.

    D4: gives ``qa:regression`` real (2010-2024) money data without touching
    the babylon-data drive — the harness reads only the committed fixture.
    """
    from babylon.domain.economics.factory import create_financial_services

    return create_financial_services(fred_series_cache=_load_vol3_fred_fixture())
```

Then modify `_run_scenario_ticks` (build once, thread through every tick):
```python
    state, sim_config, defines = create_scenario(name)
    config_info = SCENARIOS[name]
    persistent_context: dict[str, Any] = {}
    # D4: Vol III calculator_overrides built ONCE per scenario run — the
    # calculators are stateless/reused across ticks, mirroring the cadence
    # _legacy.py's Simulation already uses (one calculator_overrides dict
    # persists across the whole run, rebuilt only per ServiceContainer.create
    # call inside step() itself).
    calculator_overrides = _build_vol3_calculator_overrides()

    checkpoints: list[CheckpointData] = []
```
and:
```python
    for tick in range(1, max_ticks + 1):
        state = step(
            state,
            sim_config,
            persistent_context,
            defines,
            calculator_overrides=calculator_overrides,
        )
        ticks_survived = tick
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_vol3_wiring.py::test_run_scenario_passes_vol3_calculator_overrides_to_step`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(tools): wire committed FRED fixture into qa:regression calculator_overrides (D4)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.3b: Committed national MELT fixture — open the `melt_calculator is None` gate

**Files:**
- Create: `tools/export_vol3_melt_fixture.py`
- Create: `tests/fixtures/vol3_melt_national.json` (generated by running the script against the real reference DB, then committed)
- Modify: `tools/regression_test.py` (constants + `_build_vol3_calculator_overrides`)
- Test: `tests/unit/tools/test_export_vol3_melt_fixture.py`, `tests/unit/tools/test_regression_test_melt_gate.py`

**Interfaces:**
- Consumes: `SQLiteBEANationalGDPSource` / `SQLiteQCEWNationalEmploymentSource`
  (`src/babylon/domain/economics/melt/adapters.py`, both in that module's `__all__`);
  `MIN_YEAR` / `MAX_YEAR` and `DefaultMELTCalculator`
  (`src/babylon/domain/economics/melt/melt_calculator.py`; the class is re-exported from
  `babylon.domain.economics.melt`); `babylon.reference.database.get_normalized_session_factory`.
- Produces: `tests/fixtures/vol3_melt_national.json`; `MELT_FIXTURE_PATH`,
  `_load_vol3_melt_fixture()`, `_build_vol3_melt_calculator()` in `tools/regression_test.py`;
  a new `"melt_calculator"` key inside the **existing** `_build_vol3_calculator_overrides()`
  (Task U1.3) — this task extends that one helper, it does not add a parallel one.

> **Why this task exists (read before Step 1).** Task U1.3's wiring is correct — all 11
> Vol III calculators reach the container every tick — and yet **zero** of them are invoked
> across a full 52-tick regression run. `TickDynamicsSystem.step`
> (`src/babylon/domain/economics/tick/system/__init__.py`, the line
> `if services.melt_calculator is None:`) returns *before* Step 2, so the entire annual
> economics pipeline — national parameters, county state, Vol I production, imperial rent,
> circulation, crisis triggers, **the Step 5.5 financial layer**, class transitions,
> bifurcation risk, derived rates — never runs. `melt_calculator` is the one key
> `create_financial_services` does not supply; it is built only by `create_economics_services`,
> which needs a SQLAlchemy session the hermetic harness forbids. This task supplies it from a
> committed fixture instead, exactly as U1.2/U1.3 did for the FRED series.
>
> Two facts settled by investigation, so this task does not re-litigate them:
> 1. **`melt_calculator` does NOT need `TensorRegistry`.** `DefaultMELTCalculator.get_melt(year)`
>    consumes exactly two national annual scalars: `bea_source.get_gdp(year) -> float | None`
>    and `qcew_source.get_national_employment(year) -> int | None`. Nothing else.
> 2. **A `TensorRegistry` fixture would be pointless here anyway.** The five `qa:regression`
>    scenarios carry zero `county_fips` (verified — no factory in
>    `src/babylon/engine/scenarios/_legacy.py` passes it), so `_get_territory_fips` falls back
>    to the abstract node ids (`T001`, `T002`), and `ValueTensor4x3.fips_code` validates
>    `^\d{5}$` — a real-FIPS-keyed registry is structurally unreachable from these scenarios.
>    County-level coverage is Task U1.9's job, over the real Wayne County scenario.
>
> **Data window, measured against the reference DB (do not assume a full 2010-2024 range).**
> `fact_bea_national_industry` holds no `line_number=1` ("All industries") row at all, so
> `SQLiteBEANationalGDPSource.get_gdp` always takes its documented **county-sum fallback** —
> which covers 2001-2023 and has **no 2024**. `SQLiteQCEWNationalEmploymentSource` covers
> 2010-2024. So MELT resolves for **2010-2023** and honestly returns a `NoDataSentinel` for
> 2024. The exporter must therefore **omit** years a source returns `None` for rather than
> writing a fabricated zero (Constitution III.11) — the fixture-backed sources then return
> `None` for those years, reproducing the live adapters' behaviour byte for byte.
>
> **Which year actually gets computed.** `TickDynamicsSystem` gates on
> `tick % WEEKS_PER_YEAR != 0`, and `_determine_year` returns `base_year + tick // WEEKS_PER_YEAR`
> with `base_year` defaulting to 2010. `DEFAULT_MAX_TICKS` is 52, so a regression run has
> exactly **one** year-boundary tick — tick 52, **year 2011**. τ(2011) = 15,295,282.203M × 1e6
> / (127,933,758 × 2080) ≈ **$57.5/labor-hour**, inside `DefaultMELTCalculator`'s expected
> $55-75 band. If Step 6 reports a different τ, the fixture is wrong — do not widen the band.

- [ ] **Step 1: Write the failing evidence test**

This is the real red, and it is also this task's acceptance criterion. Task U1.3's test proved
the overrides were *constructed* and stopped there — which is precisely why the inertness went
undetected. This one proves they are *invoked*.

Create `tests/unit/tools/test_regression_test_melt_gate.py`:
```python
"""D4: the Vol III calculators threaded into qa:regression (Task U1.3) must be
genuinely INVOKED during a scenario run, not merely constructed.

Construction is not the contract — execution is. TickDynamicsSystem's
`if services.melt_calculator is None` guard sits before Step 2 of the annual
economics pipeline, so a fully-wired-but-MELT-less container produces exactly
zero Vol III calculator calls across a whole run while every construction-level
assertion stays green.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import regression_test as rt  # type: ignore[import-not-found]  # noqa: E402


def test_regression_run_actually_invokes_the_vol3_financial_layer(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """RED->GREEN: a full regression run must reach Step 5.5 (financial layer).

    ``_compute_national_financial_state`` calls
    ``interest_calculator.compute_interest_rate_state(year)`` once per
    year-boundary tick, *before* the per-county loop — so it fires even though
    these scenarios carry no real counties. Zero calls means the pipeline never
    got past the ``melt_calculator is None`` gate.
    """
    from babylon.domain.economics.credit.interest import DefaultInterestCalculator

    original = DefaultInterestCalculator.compute_interest_rate_state
    observed_years: list[int] = []

    def _counting(self: Any, year: int) -> Any:
        observed_years.append(year)
        return original(self, year)

    monkeypatch.setattr(
        DefaultInterestCalculator, "compute_interest_rate_state", _counting
    )

    rt.run_scenario("two_node", max_ticks=rt.DEFAULT_MAX_TICKS)

    assert observed_years, (
        "Vol III financial calculators were CONSTRUCTED but never INVOKED across a "
        f"full {rt.DEFAULT_MAX_TICKS}-tick qa:regression run. TickDynamicsSystem's "
        "`if services.melt_calculator is None` guard "
        "(src/babylon/domain/economics/tick/system/__init__.py) is still closed, so "
        "Steps 2-9 of the annual economics pipeline — national params, county state, "
        "Vol I production, imperial rent, circulation, crisis triggers, the Step 5.5 "
        "financial layer, class transitions, bifurcation risk, derived rates — never "
        "execute. qa:regression remains blind to the whole economics estate."
    )
    # base_year 2010 + tick 52 // 52 == 2011: the single year boundary in a 52-tick run.
    assert observed_years == [2011], (
        f"expected exactly one year-boundary evaluation at year 2011, got {observed_years}"
    )


def test_committed_melt_fixture_yields_a_real_tau_not_a_sentinel() -> None:
    """The fixture must be materially right, not merely present.

    A fixture that parses but resolves to NoDataSentinel would leave the gate
    open and the pipeline still dark at Step 2 (`_compute_national_params`
    returns None on an unavailable MELT).
    """
    calculator = rt._build_vol3_melt_calculator()

    tau = calculator.get_melt(2011)

    assert isinstance(tau, float), (
        f"MELT for 2011 resolved to {tau!r} — the committed MELT fixture is "
        "missing GDP and/or employment for the only year qa:regression computes"
    )
    valid, message = calculator.validate_melt(tau)
    assert valid, message
    assert message is None, (
        f"tau={tau} is outside DefaultMELTCalculator's expected band: {message}. "
        "The fixture is wrong — do not widen the band."
    )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_melt_gate.py`
Expected: **both** tests FAIL, and for the right reasons —
`test_regression_run_actually_invokes_the_vol3_financial_layer` with
`AssertionError: Vol III financial calculators were CONSTRUCTED but never INVOKED ...`
(a genuine assertion-level red: the run completes, 52 ticks, zero calls), and
`test_committed_melt_fixture_yields_a_real_tau_not_a_sentinel` with
`AttributeError: module 'regression_test' ... has no attribute '_build_vol3_melt_calculator'`.
- [ ] **Step 3: Write the exporter + its unit test**

Create `tools/export_vol3_melt_fixture.py`:
```python
#!/usr/bin/env python3
"""One-off export: national MELT inputs -> a committed, deterministic fixture.

Sibling of ``tools/export_vol3_fred_fixture.py`` (same D4 split, same rules):
this is the one-off, DB-reading half; ``tools/regression_test.py`` is the
hermetic, DB-free half that only ever reads the committed JSON produced here.
The two must never be merged back into one module.

Why a second fixture at all: ``create_financial_services`` supplies every Vol III
calculator EXCEPT ``melt_calculator``, and ``TickDynamicsSystem`` returns early on
``services.melt_calculator is None`` before Step 2 — so without this fixture the
whole annual economics pipeline is skipped under ``qa:regression`` and the gate is
blind to the economics estate.

``DefaultMELTCalculator`` consumes exactly two national annual scalars, so that is
all this exports: BEA GDP (current dollars) and QCEW national employment (persons),
read through the SAME adapters the live wiring uses
(``babylon.domain.economics.melt.adapters``) so the fixture is faithful by
construction rather than by a re-implemented query.

Honest absence (Constitution III.11): a year whose source returns ``None`` is
OMITTED, never written as a zero. The fixture-backed sources in
``tools/regression_test.py`` return ``None`` for a missing year, reproducing the
live adapters exactly. As of the current reference DB that means GDP 2010-2023
(``fact_bea_national_industry`` has no ``line_number=1`` row, so ``get_gdp`` takes
its county-sum fallback, which has no 2024) and employment 2010-2024.

Run once (and re-run only when the reference DB's BEA/QCEW tables are refreshed):

    poetry run python tools/export_vol3_melt_fixture.py

Prerequisite: the worktree's data/ symlink farm must resolve
(``mise run data:doctor``) — this script opens the reference SQLite DB via
babylon.reference.database.get_normalized_session_factory().
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from babylon.domain.economics.melt.adapters import (
    SQLiteBEANationalGDPSource,
    SQLiteQCEWNationalEmploymentSource,
)
from babylon.domain.economics.melt.melt_calculator import MAX_YEAR, MIN_YEAR
from babylon.reference.database import get_normalized_session_factory

FIXTURE_PATH: Path = (
    Path(__file__).parent.parent / "tests" / "fixtures" / "vol3_melt_national.json"
)


def main() -> int:
    """Query the reference DB and write the committed national MELT fixture.

    The year loop is bounded by ``DefaultMELTCalculator``'s own
    ``MIN_YEAR``/``MAX_YEAR`` constants (imported, not duplicated) so the
    fixture window can never drift from the calculator's guard.

    :returns: 0 on success, 1 if either source yielded zero years (loud
        failure per Constitution III.11 — never write a half-empty fixture).
    """
    session_factory = get_normalized_session_factory()
    bea = SQLiteBEANationalGDPSource(session_factory)
    qcew = SQLiteQCEWNationalEmploymentSource(session_factory)

    gdp: dict[str, float] = {}
    employment: dict[str, int] = {}
    for year in range(MIN_YEAR, MAX_YEAR + 1):
        gdp_value = bea.get_gdp(year)
        if gdp_value is not None:
            gdp[str(year)] = float(gdp_value)
        employment_value = qcew.get_national_employment(year)
        if employment_value is not None:
            employment[str(year)] = int(employment_value)

    if not gdp or not employment:
        print(
            "export_vol3_melt_fixture: reference DB yielded "
            f"{len(gdp)} GDP year(s) and {len(employment)} employment year(s) "
            f"over [{MIN_YEAR}, {MAX_YEAR}] — refusing to write a fixture that "
            "cannot resolve a single MELT",
            file=sys.stderr,
        )
        return 1

    payload = {
        "gdp": dict(sorted(gdp.items())),
        "employment": dict(sorted(employment.items())),
    }
    FIXTURE_PATH.parent.mkdir(parents=True, exist_ok=True)
    FIXTURE_PATH.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    print(
        f"export_vol3_melt_fixture: wrote {len(gdp)} GDP year(s) + "
        f"{len(employment)} employment year(s) to {FIXTURE_PATH}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

Create `tests/unit/tools/test_export_vol3_melt_fixture.py`:
```python
"""Tests for the one-off national MELT fixture export script (D4).

Both adapters are monkeypatched — this test never touches the reference DB or
the babylon-data drive; it only pins export_vol3_melt_fixture.main()'s JSON
shape, its sorting, and its honest-absence rule (a year whose source returns
None is omitted, never zero-filled).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import export_vol3_melt_fixture as export_mod  # type: ignore[import-not-found]  # noqa: E402


class _StubGDP:
    def __init__(self, _session_factory: object) -> None:
        self._values = {2011: 1.5295282203e13, 2010: 1.4754993743e13}

    def get_gdp(self, year: int) -> float | None:
        return self._values.get(year)


class _StubEmployment:
    def __init__(self, _session_factory: object) -> None:
        self._values = {2011: 127_933_758, 2010: 126_464_161}

    def get_national_employment(self, year: int) -> int | None:
        return self._values.get(year)


def test_main_writes_a_deterministic_sorted_json_fixture(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """RED->GREEN: main() writes {"gdp"|"employment": {year_str: value}}, sorted."""
    fixture_path = tmp_path / "vol3_melt_national.json"
    monkeypatch.setattr(export_mod, "FIXTURE_PATH", fixture_path)
    monkeypatch.setattr(export_mod, "get_normalized_session_factory", lambda: object())
    monkeypatch.setattr(export_mod, "SQLiteBEANationalGDPSource", _StubGDP)
    monkeypatch.setattr(export_mod, "SQLiteQCEWNationalEmploymentSource", _StubEmployment)

    exit_code = export_mod.main()

    assert exit_code == 0
    raw = fixture_path.read_text()
    data = json.loads(raw)
    assert data == {
        "employment": {"2010": 126_464_161, "2011": 127_933_758},
        "gdp": {"2010": 1.4754993743e13, "2011": 1.5295282203e13},
    }
    # Order is the contract, not just content: dict == is order-insensitive.
    assert list(data) == ["employment", "gdp"]
    assert list(data["gdp"]) == ["2010", "2011"]
    # III.11: years the stub has no data for are absent, not zero-filled.
    assert "2020" not in data["gdp"]
    assert raw.endswith("\n")


def test_main_refuses_to_write_when_a_source_is_empty(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """III.11: no resolvable MELT year is a loud failure, never a silent file."""

    class _EmptyEmployment:
        def __init__(self, _session_factory: object) -> None:
            pass

        def get_national_employment(self, _year: int) -> int | None:
            return None

    fixture_path = tmp_path / "vol3_melt_national.json"
    monkeypatch.setattr(export_mod, "FIXTURE_PATH", fixture_path)
    monkeypatch.setattr(export_mod, "get_normalized_session_factory", lambda: object())
    monkeypatch.setattr(export_mod, "SQLiteBEANationalGDPSource", _StubGDP)
    monkeypatch.setattr(export_mod, "SQLiteQCEWNationalEmploymentSource", _EmptyEmployment)

    exit_code = export_mod.main()

    assert exit_code == 1
    assert not fixture_path.exists()
```

Run: `mise run test:q -- tests/unit/tools/test_export_vol3_melt_fixture.py`
Expected: PASS (2 tests)
- [ ] **Step 4: Generate the committed fixture**
```bash
poetry run python tools/export_vol3_melt_fixture.py
```
Expected stdout: `export_vol3_melt_fixture: wrote 14 GDP year(s) + 15 employment year(s) to .../tests/fixtures/vol3_melt_national.json`

GDP 2010-2023 = 14 years; employment 2010-2024 = 15 years. If either count differs, the
reference DB has moved — **STOP and report the drift**, do not adjust the expectation.
- [ ] **Step 5: Wire the fixture into `tools/regression_test.py`**

Locate `FRED_FIXTURE_PATH: Final[Path] = (` (added by Task U1.3) and add the MELT
constant and helpers immediately **after** `_load_vol3_fred_fixture`'s closing return,
i.e. directly above `def _build_vol3_calculator_overrides`:
```python
MELT_FIXTURE_PATH: Final[Path] = (
    Path(__file__).parent.parent / "tests" / "fixtures" / "vol3_melt_national.json"
)


class _FixtureNationalGDPSource:
    """Fixture-backed ``BEADataSource`` — hermetic twin of ``SQLiteBEANationalGDPSource``.

    Returns ``None`` for an absent year, exactly as the SQLite adapter does
    (Constitution III.11) — the fixture omits years the reference DB has no
    row for rather than zero-filling them.
    """

    def __init__(self, gdp_by_year: dict[int, float]) -> None:
        self._gdp_by_year = gdp_by_year

    def get_gdp(self, year: int) -> float | None:
        return self._gdp_by_year.get(year)


class _FixtureNationalEmploymentSource:
    """Fixture-backed ``QCEWDataSource`` — twin of ``SQLiteQCEWNationalEmploymentSource``."""

    def __init__(self, employment_by_year: dict[int, int]) -> None:
        self._employment_by_year = employment_by_year

    def get_national_employment(self, year: int) -> int | None:
        return self._employment_by_year.get(year)


def _load_vol3_melt_fixture() -> tuple[dict[int, float], dict[int, int]]:
    """Load the committed national MELT fixture (D4) — no DB, no drive.

    JSON object keys are always strings; ``DefaultMELTCalculator`` indexes by
    ``int`` year, so keys are converted back on load.

    Returns:
        ``(gdp_by_year, employment_by_year)`` — the two national annual series
        ``DefaultMELTCalculator`` consumes, and nothing else.
    """
    raw: dict[str, dict[str, float]] = json.loads(MELT_FIXTURE_PATH.read_text())
    gdp = {int(year): float(value) for year, value in raw["gdp"].items()}
    employment = {int(year): int(value) for year, value in raw["employment"].items()}
    return gdp, employment


def _build_vol3_melt_calculator() -> Any:
    """Build the fixture-backed ``melt_calculator`` for ``qa:regression``.

    ``create_financial_services`` supplies every Vol III calculator EXCEPT this
    one — it is built only by ``create_economics_services``, which needs a
    SQLAlchemy session the hermetic harness forbids. Without it,
    ``TickDynamicsSystem`` returns early on ``melt_calculator is None`` and the
    entire annual economics pipeline (Steps 2-9, including the Step 5.5 Vol III
    financial layer) is skipped — the gate runs, but sees nothing.

    Deliberately NOT accompanied by a ``TensorRegistry``: ``get_melt`` does not
    consume one, and the five regression scenarios carry no ``county_fips``, so
    a real-FIPS-keyed registry is unreachable from them (county coverage is
    Task U1.9's job, over the Wayne County scenario).
    """
    from babylon.domain.economics.melt import DefaultMELTCalculator

    gdp_by_year, employment_by_year = _load_vol3_melt_fixture()
    return DefaultMELTCalculator(
        _FixtureNationalGDPSource(gdp_by_year),
        _FixtureNationalEmploymentSource(employment_by_year),
    )
```

Then extend the **existing** `_build_vol3_calculator_overrides` (do not add a second
builder). Replace its body's final line — `return create_financial_services(...)` — with:
```python
    overrides = create_financial_services(fred_series_cache=_load_vol3_fred_fixture())
    # The one key create_financial_services does not supply, and the one that
    # decides whether TickDynamicsSystem executes at all.
    overrides["melt_calculator"] = _build_vol3_melt_calculator()
    return overrides
```
- [ ] **Step 6: Run test to verify it passes — the gate is open**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_melt_gate.py`
Expected: PASS (2 tests). `observed_years == [2011]` is the evidence the annual pipeline
executed; `validate_melt` returning `(True, None)` on τ ≈ 57.5 is the evidence the fixture
is materially right.

Also re-run U1.3/U1.4's file to confirm no regression:
Run: `mise run test:q -- tests/unit/tools/test_regression_test_vol3_wiring.py`
Expected: PASS. The hermeticity pin still holds — `DefaultMELTCalculator` and the two
fixture-backed sources are all DB-free; none of U1.4's forbidden tokens
(`import sqlalchemy`, `get_normalized_session_factory`, `get_reference_session`,
`load_fred_series_from_db(`) is introduced here.
- [ ] **Step 7: Verify the fixture is byte-reproducible**
```bash
SNAPSHOT="$(mktemp -d)/vol3_melt_national.json"
cp tests/fixtures/vol3_melt_national.json "$SNAPSHOT"
poetry run python tools/export_vol3_melt_fixture.py
cmp tests/fixtures/vol3_melt_national.json "$SNAPSHOT" && echo "BYTE-IDENTICAL"
git diff --stat -- tests/fixtures/vol3_melt_national.json
```
Expected: `BYTE-IDENTICAL`, and `git diff --stat` prints nothing. If the two runs differ,
the exporter is non-deterministic — **STOP and report**; do not commit a fixture that
cannot be regenerated.
- [ ] **Step 8: Record the (expected) baseline movement — do NOT regenerate**
```bash
mise run qa:regression 2>&1 | tee "$(mktemp -d)/qa-regression-after-u1.3b.txt"
```
Expected: **RED, on some or all of the 5 scenarios.** This is the point of the task, not a
failure. `qa:regression` has been byte-identical only because the annual economics pipeline
never ran; now that Steps 2-9 execute, the scenarios' trajectories legitimately move.

**Do not touch anything under `tests/baselines/`.** Baselines are frozen for the whole plan;
regeneration happens exclusively in Task U8.5, in the ceremony commit, behind the explicit
Owner Approval Gate in `reports/vol3-baseline-delta.md`. Record the verbatim output (Task
U8.2's `tools/capture_qa_diff.py` is what formalises this capture) and continue to U1.4.
Never weaken an assertion or regenerate a baseline to make this green.

If instead the run *crashes* (rather than diverging), that is a real defect the closed gate
was hiding — record it and report it; do not re-close the gate to make it go away.
- [ ] **Step 9: Commit**
```bash
git add tools/export_vol3_melt_fixture.py tests/fixtures/vol3_melt_national.json \
        tests/unit/tools/test_export_vol3_melt_fixture.py \
        tests/unit/tools/test_regression_test_melt_gate.py \
        tools/regression_test.py
mise run commit -- "feat(tools): committed national MELT fixture opens the qa:regression economics gate (D4)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
git log --oneline -1
```
Verify HEAD actually moved — the hooks can rewrite staged files and abort while printing
all-green.

---

### Task U1.4: Hermetic-guard test pinning `tools/regression_test.py` touches no DB/drive

**Files:**
- Test: `tests/unit/tools/test_regression_test_vol3_wiring.py` (same file as Task U1.3 — append)

**Interfaces:**
- Consumes: nothing new (static source inspection of `regression_test.py`).
- Produces: a permanent regression pin for D4's "the regression harness remains hermetic" acceptance criterion — later units must not reintroduce a DB import here.

- [ ] **Step 1: Write the failing test**
```python
def test_regression_test_module_stays_hermetic_no_db_no_drive() -> None:
    """D4: qa:regression's in-memory harness must never gain a DB/drive
    dependency. create_financial_services() itself is DB-free (it only
    builds calculator objects from an in-memory cache dict); the forbidden
    tokens below are specifically the DB-touching entry points."""
    source = Path(rt.__file__).read_text()
    forbidden = (
        "import sqlalchemy",
        "get_normalized_session_factory",
        "get_reference_session",
        "load_fred_series_from_db(",
    )
    violations = [token for token in forbidden if token in source]
    assert not violations, (
        f"tools/regression_test.py references {violations} — the qa:regression "
        "harness is no longer hermetic (D4 requires fixture-only, no DB, no drive)"
    )
    assert "FRED_FIXTURE_PATH" in source, (
        "tools/regression_test.py does not load the committed Vol III FRED "
        "fixture — the gate is hermetic but blind (D4 requires both)"
    )
```
- [ ] **Step 2a: Inject the defect**
Add `from babylon.reference.database import get_normalized_session_factory  # noqa: F401` to `tools/regression_test.py`'s imports.
- [ ] **Step 2b: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_vol3_wiring.py::test_regression_test_module_stays_hermetic_no_db_no_drive`
Expected: FAIL with `AssertionError: tools/regression_test.py references ['get_normalized_session_factory'] — the qa:regression harness is no longer hermetic (D4 requires fixture-only, no DB, no drive)`
- [ ] **Step 3: Restore (GREEN)**
Run: `git checkout -- tools/regression_test.py` — `tools/regression_test.py` already satisfies the pin after Task U1.3; no implementation change is needed.
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_regression_test_vol3_wiring.py`
Expected: PASS (both tests from U1.3 and U1.4)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(tools): pin qa:regression harness as hermetic (D4, no DB/drive)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.5: Repoint `tick_ground_rent` at `surplus_distribution.ground_rent` (Path A)

**Files:**
- Modify: `src/babylon/domain/economics/tick/graph_bridge.py:158-162`
- Modify: `src/babylon/sentinels/seam/registry.py:922-937`
- Test: `tests/unit/economics/tick/test_graph_bridge.py:226-295` (existing — edit assertion)

**Interfaces:**
- Consumes: `SurplusValueDistribution.ground_rent: float` (existing, `src/babylon/domain/economics/distribution/types.py:63`).
- Produces: the `tick_ground_rent` graph attribute now mirrors Path A (real FRED `B230RC0Q173SBEA`-derived rental income) instead of Path B's permanent `0.0` (`_DefaultCountyRentalAdapter` always returns `None`).

- [ ] **Step 1: Write the failing test** (edit the existing `test_writes_populated_financial_attrs` in `tests/unit/economics/tick/test_graph_bridge.py`)
```python
    def test_writes_populated_financial_attrs(
        self,
        sample_tick_state: SimulationTickState,
    ) -> None:
        """Verify financial fields written when populated on county state."""
        svd = SurplusValueDistribution(
            fips_code="26163",
            year=2015,
            total_surplus_produced=1000.0,
            interest_payments=200.0,
            ground_rent=100.0,
            taxes_on_surplus=50.0,
        )
        rent = RentExtraction(
            fips_code="26163",
            year=2015,
            agricultural_rent=50.0,
            resource_rent=30.0,
            urban_rent=200.0,
        )
        housing = HousingValueDecomposition(
            fips_code="26163",
            year=2015,
            construction_value=100000.0,
            ground_rent_capitalized=50000.0,
            speculative_premium=30000.0,
        )
        debt = DebtAccumulation(
            fips_code="26163",
            year=2015,
            accumulated_debt=500.0,
            consecutive_deficit_ticks=3,
        )
        crisis = FinancialCrisisAssessment(
            fips_code="26163",
            year=2015,
            profit_squeeze=True,
            overaccumulation=True,
            credit_fragility=True,
            claims_exceed_surplus=True,
        )

        original_county = sample_tick_state.county_states[WAYNE_FIPS]
        modified_county = original_county.model_copy(
            update={
                "surplus_distribution": svd,
                "rent_extraction": rent,
                "housing_decomposition": housing,
                "debt_accumulation": debt,
                "financial_crisis": crisis,
            }
        )
        modified_state = sample_tick_state.model_copy(
            update={"county_states": {WAYNE_FIPS: modified_county}}
        )

        graph = build_territory_graph()
        write_tick_state_to_graph(graph, modified_state)

        node_data = graph.nodes[WAYNE_FIPS]
        assert node_data["tick_interest_burden"] == 200.0
        # U1 repoint: tick_ground_rent now mirrors Path A
        # (SurplusValueDistribution.ground_rent, real FRED rental income),
        # not Path B (RentExtraction.total_rent = 50 + 30 + 200 = 280, which
        # _DefaultCountyRentalAdapter always returns None for and therefore
        # never actually reaches this attribute in production).
        assert node_data["tick_ground_rent"] == 100.0
        assert node_data["tick_rentier_share"] == 0.1  # 100 / 1000
        assert node_data["tick_profit_of_enterprise"] == 650.0
        assert node_data["tick_financialization_share"] == 0.2  # 200 / 1000
        assert node_data["tick_accumulated_debt"] == 500.0
        assert node_data["tick_claims_exceed_surplus"] is False  # 200+100+50 < 1000
        expected_fict = (50000.0 + 30000.0) / 180000.0
        assert abs(node_data["tick_housing_fictitious_fraction"] - expected_fict) < 1e-9
        assert node_data["tick_financial_crisis_signals"] == 4
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge.py::TestWriteFinancialState::test_writes_populated_financial_attrs`
Expected: FAIL with `assert 280.0 == 100.0`
- [ ] **Step 3: Write minimal implementation**

Edit `src/babylon/domain/economics/tick/graph_bridge.py:158-162`:
```python
            tick_ground_rent=(  # pragma: no mutate
                county.surplus_distribution.ground_rent  # pragma: no mutate
                if county.surplus_distribution is not None  # pragma: no mutate
                else 0.0  # pragma: no mutate
            ),  # pragma: no mutate
```

Companion documentation-accuracy fix — `src/babylon/sentinels/seam/registry.py:922-937` currently declares `tick_ground_rent` as permanently `NOT_YET_COMPUTED` because of Path B's dead adapter; after the repoint it is genuinely live on the same condition as its Group D siblings (`tick_interest_burden` etc.). Edit the entry:
```python
    SeamEntry(
        payload="tick_ground_rent",
        wire_keys=("tick_ground_rent",),
        scope=SeamScope.TERRITORY,
        owner_layer="domain.economics.tick (SurplusValueDistribution.ground_rent)",
        liveness_class=LivenessClass.DECLARED_CONDITIONAL,
        liveness_condition=_FINANCIAL_LIVENESS_CONDITION,
        dtype="float",
        read_paths=_TICK_DARK_EMITTERS,
        spec_ref="vol3-money-scissors U1 (2026-07-18); supersedes spec-116 Task 20b",
        notes=(
            "U1 repoint: was permanently NOT_YET_COMPUTED because "
            "write_tick_state_to_graph read RentExtraction.total_rent (Path "
            "B, _DefaultCountyRentalAdapter unconditionally returns None — "
            "no county rental series in the reference DB). Repointed to "
            "SurplusValueDistribution.ground_rent (Path A, real FRED "
            "B230RC0Q173SBEA rental income via DefaultDistributionCalculator), "
            "which was always live wherever distribution_calculator + "
            "tensor_registry are wired — same condition as tick_interest_burden. "
            "The 3-way agricultural/resource/urban rent split (Path B) has no "
            "data source and stays honestly absent as its own field "
            "(rent_extraction), just no longer the source of this graph attr."
        ),
    ),
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge.py`
Expected: PASS (all tests in the file, including the unmodified `test_writes_default_financial_attrs_when_none` which stays `0.0` either way)

Also verify the static seam gate stays green:
Run: `mise run check:seams`
Expected: exit 0 (`SeamEntry.payload="tick_ground_rent"` is unchanged, so `check_tick_payloads_exist` still finds it in the write-set)

Also confirm the intended convergence (design §3.1): run `poetry run python tools/validate_detroit.py` and verify its reported `surplus.ground_rent` now equals the `tick_ground_rent` node attribute for Wayne. This is an INTENDED outcome, not a regression — record both values in the commit body.
- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(economics): repoint tick_ground_rent at surplus_distribution.ground_rent (Path A)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.6: Wire `TensorRegistry` + Vol III financial services into `headless_runner._build_economics_overrides`

**Files:**
- Modify: `src/babylon/engine/headless_runner/runner.py:35` (typing import), `:83-87` (module constant), `:906-1000` (`_build_economics_overrides`)
- Test: `tests/unit/engine/headless_runner/test_gamma_wiring.py` (existing file — append)

**Interfaces:**
- Consumes: `TensorRegistry`, `MarxianHydrator`, `SQLiteQCEWSource`, `DepartmentMapper.from_yaml`, `StubBEASource` (all existing, same construction pattern as `web/game/engine_bridge.py:_build_capital_calculator`); `create_financial_services`, `load_fred_series_from_db` (`src/babylon/domain/economics/factory.py`).
- Produces: `_build_economics_overrides(..., scope_fips: frozenset[str] | None = None)` — new keyword param; when provided alongside `session_factory`, `overrides["tensor_registry"]` and the 11 Vol III financial calculator keys are populated. Task U1.7 threads `config.scope_fips` into this from `run()`.

- [ ] **Step 1: Write the failing test**
```python
def test_build_economics_overrides_wires_tensor_registry_and_financial_services() -> None:
    """U1 (vol3-money-scissors): tensor_registry + Vol III financial
    calculators are wired when scope_fips is provided alongside
    session_factory.

    Without this, `_get_county_surplus`/`_get_county_profit_rate`
    (domain/economics/tick/system/__init__.py:1547,1599) read
    `getattr(services, "tensor_registry", None)` as permanently None, so
    `total_surplus` never exceeds 0 and `surplus_distribution` never
    computes — even though `distribution_calculator` et al. are wired.
    """
    pytest.importorskip("sqlalchemy")
    if not SQLITE_REF.exists():
        pytest.skip(f"SQLite reference DB missing at {SQLITE_REF}")

    from babylon.engine.headless_runner.runner import _build_economics_overrides
    from babylon.reference.database import get_normalized_session_factory

    session_factory = get_normalized_session_factory()
    overrides, leontief_session = _build_economics_overrides(
        session_factory=session_factory,
        scope_fips=frozenset({"26163"}),
    )
    try:
        assert overrides.get("tensor_registry") is not None
        assert "26163" in overrides["tensor_registry"].all_fips()
        assert overrides.get("distribution_calculator") is not None
        assert overrides.get("interest_calculator") is not None
        assert overrides.get("fictitious_capital_calculator") is not None
    finally:
        if leontief_session is not None:
            leontief_session.close()
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/engine/headless_runner/test_gamma_wiring.py::test_build_economics_overrides_wires_tensor_registry_and_financial_services`
Expected: FAIL with `TypeError: _build_economics_overrides() got an unexpected keyword argument 'scope_fips'`
- [ ] **Step 3: Write minimal implementation**

Edit `src/babylon/engine/headless_runner/runner.py:35`:
```python
from typing import Any, Final
```

Edit `src/babylon/engine/headless_runner/runner.py` — add a module constant near the existing ones (after `_AUDIT_SEVERITY_MAP`, before `class RunnerError`):
```python
#: Real Vol III FRED coverage horizon (design doc §1.4: all ten Vol III
#: series terminate at 2024; GFDEBTN starts 2010). Mirrors
#: web/game/engine_bridge.py's ``_CAPITAL_HYDRATION_YEARS``.
_TENSOR_HYDRATION_YEARS: Final[tuple[int, ...]] = tuple(range(2010, 2025))
```

Edit `_build_economics_overrides`'s signature (`:906-910`):
```python
def _build_economics_overrides(
    session_factory: Any = None,
    event_bus: Any = None,
    defines: Any = None,
    scope_fips: frozenset[str] | None = None,
) -> tuple[dict[str, Any], Any]:
```

Add a `scope_fips` entry to the docstring's `Args:` (after the existing `defines:` entry, before `Returns:`):
```python
        scope_fips: Optional set of county FIPS this run computes over. When
            provided (with ``session_factory``), a ``TensorRegistry`` is
            hydrated for these counties over
            ``_TENSOR_HYDRATION_YEARS`` (2010-2024) and exposed as
            ``overrides["tensor_registry"]``, and the Vol III financial
            calculators (:func:`~babylon.domain.economics.factory.create_financial_services`)
            are wired from the reference DB's FRED tables — so
            ``surplus_distribution`` (s = p + i + r + t) genuinely computes
            per county instead of staying permanently ``None``
            (``services.tensor_registry is None`` gate,
            ``domain/economics/tick/system/__init__.py:1547,1599,1599``).
```

Insert the new wiring block right after the Leontief block (after `overrides.update(leontief_overrides)`, still nested inside `if session_factory is not None:`, before `return overrides, leontief_session`):
```python
        if scope_fips:
            import babylon.domain.economics as economics_pkg
            from babylon.domain.economics.adapters import SQLiteQCEWSource
            from babylon.domain.economics.department_mapper import DepartmentMapper
            from babylon.domain.economics.factory import (
                create_financial_services,
                load_fred_series_from_db,
            )
            from babylon.domain.economics.hydrator import MarxianHydrator
            from babylon.domain.economics.tensor_registry import TensorRegistry
            from babylon.engine.hydration.reference import StubBEASource
            from babylon.reference.database import get_reference_session

            # DELIBERATE TWIN: web/game/engine_bridge.py:_build_tensor_registry runs the
            # near-identical hydration. Two by design — a shared factory would put
            # reference-DB I/O in domain/, which the layering forbids. Change both.
            registry = TensorRegistry()
            naics_yaml = Path(economics_pkg.__file__).parent / "data" / "naics_to_dept.yaml"
            with get_reference_session() as ref_session:
                hydrator = MarxianHydrator(
                    SQLiteQCEWSource(ref_session),
                    StubBEASource(),  # falls back to DepartmentMapper department ratios
                    DepartmentMapper.from_yaml(naics_yaml),
                )
                registry.hydrate_counties(
                    hydrator, sorted(scope_fips), list(_TENSOR_HYDRATION_YEARS)
                )
            overrides["tensor_registry"] = registry

            fred_cache = load_fred_series_from_db(session_factory)
            overrides.update(create_financial_services(fred_series_cache=fred_cache))
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/engine/headless_runner/test_gamma_wiring.py`
Expected: PASS (all tests in the file, including the pre-existing ones — no assertions on them changed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(headless-runner): wire tensor_registry + Vol III financial services via calculator_overrides

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.7: Thread `scope_fips` through `headless_runner.run()`'s `ServiceContainer.create` call site

**Files:**
- Modify: `src/babylon/engine/headless_runner/runner.py:1142-1146`
- Test: `tests/unit/engine/headless_runner/test_gamma_wiring.py::test_run_passes_gamma_calculator_to_service_container` (existing — edit)

**Interfaces:**
- Consumes: `_build_economics_overrides(..., scope_fips=...)` (Task U1.6); `SimulationRunConfig.scope_fips: frozenset[str]` (existing, `src/babylon/engine/headless_runner/models.py:112`).
- Produces: `ServiceContainer.create(**economics_overrides)` now receives `tensor_registry` for every real `run()` invocation.

- [ ] **Step 1: Write the failing test** (edit the existing test in `tests/unit/engine/headless_runner/test_gamma_wiring.py`)
```python
@pytest.mark.requires_reference_db
def test_run_passes_gamma_calculator_to_service_container(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
) -> None:
    """RED->GREEN: run() must pass gamma_calculator AND tensor_registry to
    ServiceContainer.create.

    Before E101 wiring: ``ServiceContainer.create(defines=defines)`` is
    called with no ``gamma_calculator`` kwarg -> assertion fails (RED).
    After wiring: ``_build_economics_overrides()`` is called and its
    return dict is unpacked into ``ServiceContainer.create(**overrides)``,
    so ``gamma_calculator`` is present and non-None (GREEN).

    U1 (vol3-money-scissors) additionally threads ``scope_fips=config.scope_fips``
    into that same call, so ``tensor_registry`` (and the Vol III financial
    calculators) must ALSO be present in the captured kwargs.

    The Postgres / hydration / bridge layer is stubbed so we reach the
    ``ServiceContainer.create`` call site without a live database. A
    sentinel exception stops execution immediately after the call so
    the captured kwargs can be inspected.
    """
    from babylon.engine.headless_runner import runner as runner_mod
    from babylon.engine.headless_runner.models import SimulationRunConfig

    if not SQLITE_REF.exists():
        pytest.skip(f"SQLite reference DB missing at {SQLITE_REF}")

    captured: dict[str, Any] = {}

    class _StopAfterCreate(Exception):
        """Sentinel: stop run() immediately after ServiceContainer.create."""

    # --- Stub Postgres / hydration layer to reach ServiceContainer.create ---

    monkeypatch.setattr(runner_mod, "_install_sigint_handler", lambda: None)
    monkeypatch.setattr(runner_mod, "_open_postgres_pool", lambda: None)
    monkeypatch.setattr(runner_mod, "_apply_migrations", lambda _pool: None)

    # PostgresRuntime is imported lazily inside run()
    import babylon.persistence

    monkeypatch.setattr(babylon.persistence, "PostgresRuntime", lambda **_kwargs: None)

    # initialize_session is imported lazily inside run()
    import babylon.persistence.postgres_initialization as pg_init

    class _FakeReport:
        hex_count = 100
        national_phi_reference = 0.0

    monkeypatch.setattr(pg_init, "initialize_session", lambda **_kw: _FakeReport())

    # ConservationAuditor is imported lazily inside run()
    import babylon.persistence.conservation_audit as ca

    class _FakeAuditor:
        audit_log_buffer: list[Any] = []

        def register_invariant(self, name: str, evaluator: Any) -> None:
            pass

    monkeypatch.setattr(ca, "ConservationAuditor", lambda **_kw: _FakeAuditor())

    # WorldStateBridge + friends are top-level imports in runner
    class _FakeWorld:
        def to_graph(self) -> Any:
            return None

    class _FakeBridge:
        # Gate A (runner.py) compares this against report.hex_count (100 here).
        hex_template_size = 100

        def hydrate_initial(self, **kw: Any) -> Any:
            return _FakeWorld()

    monkeypatch.setattr(runner_mod, "WorldStateBridge", lambda **_kw: _FakeBridge())
    monkeypatch.setattr(runner_mod, "BoundaryFlowRegister", lambda: object())
    monkeypatch.setattr(runner_mod, "EventBus", lambda: object())
    monkeypatch.setattr(runner_mod, "EventCapture", lambda: object())

    # ServiceContainer.create — capture kwargs, raise sentinel
    class _FakeServiceContainer:
        @staticmethod
        def create(*_args: Any, **kwargs: Any) -> Any:
            captured.update(kwargs)
            raise _StopAfterCreate

    monkeypatch.setattr(runner_mod, "ServiceContainer", _FakeServiceContainer)

    config = SimulationRunConfig(
        ticks=1,
        scope_fips=frozenset({"26163", "26125", "26099"}),
        sqlite_reference_path=SQLITE_REF,
        output_dir=tmp_path / "out",
    )

    with pytest.raises(_StopAfterCreate):
        runner_mod.run(config)

    assert captured.get("gamma_calculator") is not None, (
        "run() did not pass gamma_calculator to ServiceContainer.create; "
        f"captured kwargs: {sorted(captured.keys())}"
    )
    assert captured.get("melt_calculator") is not None, (
        "run() did not pass melt_calculator to ServiceContainer.create; "
        f"captured kwargs: {sorted(captured.keys())}"
    )
    assert captured.get("tensor_registry") is not None, (
        "run() did not pass tensor_registry to ServiceContainer.create; "
        f"captured kwargs: {sorted(captured.keys())}"
    )
    assert captured.get("distribution_calculator") is not None, (
        "run() did not pass distribution_calculator to ServiceContainer.create; "
        f"captured kwargs: {sorted(captured.keys())}"
    )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/engine/headless_runner/test_gamma_wiring.py::test_run_passes_gamma_calculator_to_service_container`
Expected: FAIL with `AssertionError: run() did not pass tensor_registry to ServiceContainer.create; captured kwargs: [...]` (the list omits `tensor_registry`/`distribution_calculator`)
- [ ] **Step 3: Write minimal implementation**

Edit `src/babylon/engine/headless_runner/runner.py:1142-1146`:
```python
        economics_overrides, leontief_session = _build_economics_overrides(
            session_factory=calc_session_factory,
            event_bus=event_bus,
            defines=defines,
            scope_fips=config.scope_fips,
        )
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/engine/headless_runner/test_gamma_wiring.py`
Expected: PASS (all tests in the file)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(headless-runner): thread scope_fips into ServiceContainer.create (tensor_registry now reaches the tick loop)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.8: Extract `_build_tensor_registry` + wire `tensor_registry` into `engine_bridge._bridge_economics_overrides`

**Files:**
- Modify: `web/game/engine_bridge.py` — the `_CAPITAL_CALCULATOR_CACHE` / `_build_capital_calculator` block, and the `if fips_codes:` block inside `_bridge_economics_overrides`. **Located by quoted anchor, not line number** (see the base note below).
- Test: `tests/unit/web/test_tensor_registry_wiring.py`

> **BASE NOTE (deconfliction, 2026-07-18) — anchors are base-agnostic.**
> A sibling branch, `fix/null-play-coupling` (fog-of-war / player-vision), also edits
> `web/game/engine_bridge.py` and is **not yet merged to `dev`**. It inserts ~699 lines
> *above* this task's edit region, which shifts every line number here by roughly **+439**.
> **Every textual anchor this task uses is byte-identical on `dev` and on
> `fix/null-play-coupling`**, verified by `grep -F` (exactly one hit on each ref) plus a
> full-block byte diff. That is why the line numbers were removed: the quoted anchors below
> locate the edit correctly on *either* base, so this task does not care which branch lands first.
>
> *How to tell which base you are on:* `grep -c '_resolve_player_org_id' web/game/engine_bridge.py`
> → `0` means the sibling has **not** landed (you are on plain `dev`); `1` means it **has**.
> Either way, proceed unchanged.
>
> *Merge safety (verified, not assumed):* the sibling's nearest insertion
> (`_current_intel_aging_ticks`) ends ~130 lines above this edit region — far outside
> three-way-merge context. A `git merge-file` simulation of this exact edit against
> `fix/null-play-coupling` produced **0 conflicts**, with `_build_tensor_registry` and their
> `_resolve_player_org_id`/fog helpers coexisting intact.
>
> *If an anchor below is NOT found verbatim:* *STOP — do not improvise a nearby line.* It means
> the sibling (or `dev`) has since edited this specific region, which is new information the plan
> does not cover. Re-verify with
> `git show dev:web/game/engine_bridge.py | grep -F '<anchor>'` and the same against
> `fix/null-play-coupling`, then escalate with which ref lost the anchor.
>
> *Deliberately NOT adopted:* the sibling extracts `_resolve_player_org_id` / `_is_player_org`
> helpers. They are genuinely good, but they serve **their** fog feature — this branch calls the
> player-org heuristic nowhere. Copying them here would manufacture a duplicate-definition
> conflict at merge time in a region where we currently conflict **zero**. Let their branch own them.

**Interfaces:**
- Consumes: nothing new — reuses the exact `TensorRegistry`/`MarxianHydrator`/`SQLiteQCEWSource`/`DepartmentMapper`/`StubBEASource`/`get_reference_session` construction already in `_build_capital_calculator` (the `registry = TensorRegistry()` … `registry.hydrate_counties(...)` body of that function).
- Produces: `_build_tensor_registry(fips_codes: tuple[str, ...]) -> Any` (new, cached, module-level) — `_build_capital_calculator` now calls it instead of duplicating hydration; `_bridge_economics_overrides` gains `overrides["tensor_registry"]` under the same `if fips_codes:` gate as `capital_calculator`, so every real web session finally exposes `services.tensor_registry` (design doc §1.1: "no — built :6268, consumed only by CapitalStockCalculator").

- [ ] **Step 1: Write the failing test**
```python
"""U1 (vol3-money-scissors): wire tensor_registry into the web bridge.

Before this fix, _bridge_economics_overrides built a TensorRegistry only
inside _build_capital_calculator, wrapped it in a CapitalStockCalculator, and
discarded the registry itself — services.tensor_registry stayed None for
every web session, so `_get_county_surplus`/`_get_county_profit_rate`
(domain/economics/tick/system/__init__.py:1547,1599) always returned None,
and surplus_distribution (s = p + i + r + t) never computed for the playable
game (design doc §1.1, "Vol III county layer: national only").

_build_tensor_registry is factored out of _build_capital_calculator so BOTH
capital_calculator and the standalone tensor_registry override share ONE
hydration pass over the reference DB, not two.
"""

from __future__ import annotations

import pytest

pytestmark = pytest.mark.unit

WAYNE_FIPS = "26163"


@pytest.mark.requires_reference_db
class TestBridgeEconomicsOverridesWiresTensorRegistry:
    """``_bridge_economics_overrides`` must expose a real ``tensor_registry``."""

    def test_overrides_include_a_hydrated_tensor_registry(self) -> None:
        from game.engine_bridge import _bridge_economics_overrides

        overrides, leontief_session = _bridge_economics_overrides((WAYNE_FIPS,))
        try:
            assert "tensor_registry" in overrides
            assert WAYNE_FIPS in overrides["tensor_registry"].all_fips()
        finally:
            if leontief_session is not None:
                leontief_session.close()

    def test_tensor_registry_is_shared_with_the_capital_calculator(self) -> None:
        """DRY: one hydration pass backs both capital_calculator._registry
        and the standalone tensor_registry override — not two DB round-trips."""
        from game.engine_bridge import _bridge_economics_overrides

        overrides, leontief_session = _bridge_economics_overrides((WAYNE_FIPS,))
        try:
            assert overrides["tensor_registry"] is overrides["capital_calculator"]._registry
        finally:
            if leontief_session is not None:
                leontief_session.close()
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/web/test_tensor_registry_wiring.py::TestBridgeEconomicsOverridesWiresTensorRegistry::test_overrides_include_a_hydrated_tensor_registry`
Expected: FAIL with `assert "tensor_registry" in overrides` -> `AssertionError: assert 'tensor_registry' in {...}` (key absent)
- [ ] **Step 3: Write minimal implementation**

Edit `web/game/engine_bridge.py` — split `_build_capital_calculator` into a shared registry builder plus a thin capital-calculator wrapper.

**Replace the contiguous block** that begins with this line:
```python
_CAPITAL_CALCULATOR_CACHE: dict[frozenset[str], Any] = {}
```
and ends with the closing lines of `_build_capital_calculator`:
```python
    calculator = CapitalStockCalculator(registry)
    _CAPITAL_CALCULATOR_CACHE[key] = calculator
    return calculator
```
(that whole block is ~58 lines and is byte-identical on `dev` and on `fix/null-play-coupling`), with:
```python
_TENSOR_REGISTRY_CACHE: dict[frozenset[str], Any] = {}
_CAPITAL_CALCULATOR_CACHE: dict[frozenset[str], Any] = {}

#: Real Vol III FRED coverage horizon (design doc §1.4). Mirrors
#: babylon.engine.headless_runner.runner's ``_TENSOR_HYDRATION_YEARS``.
_CAPITAL_HYDRATION_YEARS: Final[tuple[int, ...]] = tuple(range(2010, 2025))


def _build_tensor_registry(fips_codes: tuple[str, ...]) -> Any:
    """Build (and cache) a hydrated ``TensorRegistry`` for the given counties.

    Spec-116 U1 (vol3-money-scissors): factored out of
    ``_build_capital_calculator`` so BOTH ``capital_calculator`` and
    ``services.tensor_registry`` share ONE hydration pass over the reference
    DB instead of two. Before this split, the web bridge built a
    ``TensorRegistry`` only to wrap it in a ``CapitalStockCalculator`` and
    discard the registry itself — the county financial layer's
    ``_get_county_surplus``/``_get_county_profit_rate``
    (``domain/economics/tick/system/__init__.py:1547,1599``) read
    ``getattr(services, "tensor_registry", None)``, which was always
    ``None`` for web sessions, so ``surplus_distribution`` never computed
    (design doc §1.1).

    Args:
        fips_codes: The county FIPS this session computes over (non-empty).

    Returns:
        A hydrated ``TensorRegistry`` covering ``_CAPITAL_HYDRATION_YEARS``
        for every FIPS in ``fips_codes``.
    """
    key = frozenset(fips_codes)
    cached = _TENSOR_REGISTRY_CACHE.get(key)
    if cached is not None:
        return cached

    from pathlib import Path

    import babylon.domain.economics as economics_pkg
    from babylon.domain.economics.adapters import SQLiteQCEWSource
    from babylon.domain.economics.department_mapper import DepartmentMapper
    from babylon.domain.economics.hydrator import MarxianHydrator
    from babylon.domain.economics.tensor_registry import TensorRegistry
    from babylon.engine.hydration.reference import StubBEASource
    from babylon.reference.database import get_reference_session

    # DELIBERATE TWIN: babylon/engine/headless_runner/runner.py:_build_economics_overrides
    # runs the near-identical hydration. Two by design — a shared factory would put
    # reference-DB I/O in domain/, which the layering forbids. Change both.
    registry = TensorRegistry()
    naics_yaml = Path(economics_pkg.__file__).parent / "data" / "naics_to_dept.yaml"
    with get_reference_session() as session:
        hydrator = MarxianHydrator(
            SQLiteQCEWSource(session),
            StubBEASource(),  # falls back to DepartmentMapper department ratios
            DepartmentMapper.from_yaml(naics_yaml),
        )
        # sorted: fixes hydration/summation order across sessions (III.7, §5 hazard 1)
        registry.hydrate_counties(hydrator, sorted(fips_codes), list(_CAPITAL_HYDRATION_YEARS))

    _TENSOR_REGISTRY_CACHE[key] = registry
    return registry


def _build_capital_calculator(fips_codes: tuple[str, ...]) -> Any:
    """Build (and cache) a ``CapitalStockCalculator`` over a shared TensorRegistry.

    Owner item 25 / Fix B (owner-ruled 2026-07-12): give the web session a REAL
    per-county capital stock K instead of the engine's ``0.0`` default, so
    ``occ = K/v`` is non-zero and ``profit_rate = s/(K+v)`` separates from
    ``exploitation_rate = s/v`` — with K=0 the two are identical, a degenerate,
    dishonest tie. As of U1 (vol3-money-scissors), the underlying
    ``TensorRegistry`` hydration is shared with ``services.tensor_registry``
    via :func:`_build_tensor_registry` — one DB round-trip, two consumers.

    Args:
        fips_codes: The county FIPS this session computes over (non-empty).

    Returns:
        A ``CapitalStockCalculator`` whose ``get_K`` returns real per-county K
        for the hydrated years, or a falsy ``NoDataSentinel`` where the
        reference DB lacks that county-year (the engine's ``_compute_county_states``
        guards on that truthiness — Constitution III.11 graceful degradation).
    """
    key = frozenset(fips_codes)
    cached = _CAPITAL_CALCULATOR_CACHE.get(key)
    if cached is not None:
        return cached

    from babylon.domain.economics.capital_stock import CapitalStockCalculator

    registry = _build_tensor_registry(fips_codes)
    calculator = CapitalStockCalculator(registry)
    _CAPITAL_CALCULATOR_CACHE[key] = calculator
    return calculator
```

Edit `_bridge_economics_overrides`. **Locate by this anchor** (one hit on both `dev` and `fix/null-play-coupling`):
```python
    # Owner item 25 / Fix B: wire a real per-county capital_calculator (cached) so
```
Replace that comment and the two lines following it (`if fips_codes:` / `overrides["capital_calculator"] = ...`) with:
```python
    # Owner item 25 / Fix B: wire a real per-county capital_calculator (cached) so
    # occ and profit_rate are non-degenerate. Only when we know which counties to
    # hydrate — a bare call (no FIPS) leaves K at the engine's 0.0 default.
    # U1 (vol3-money-scissors): also wire the SAME hydrated TensorRegistry as
    # services.tensor_registry — the missing piece that kept the county
    # financial layer (surplus_distribution, s = p + i + r + t) permanently
    # dark for web sessions even though distribution_calculator/interest_
    # calculator/fictitious_capital_calculator etc. were already wired below.
    if fips_codes:
        overrides["capital_calculator"] = _build_capital_calculator(fips_codes)
        overrides["tensor_registry"] = _build_tensor_registry(fips_codes)
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/web/test_tensor_registry_wiring.py`
Expected: PASS (both tests)

Also confirm the pre-existing throughput-wiring tests (which exercise the same `_build_capital_calculator` call path) still pass unchanged:
Run: `mise run test:q -- tests/unit/web/test_throughput_wiring.py`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(web): wire tensor_registry into the web bridge (shared with capital_calculator)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U1.9: Real-run acceptance — `s = p + i + r + t` evaluates, and holds (SC-001)

**Files:**
- Create: `tests/integration/economics/test_vol3_surplus_distribution_live.py`
- Modify: `tests/integration/economics/conftest.py` — append one new section
  (a `TYPE_CHECKING` import block + the `build_wayne_world_state()` builder).
  No existing fixture in that file is touched.

**Interfaces:**
- Consumes: `_build_economics_overrides(session_factory=..., scope_fips=...)` (U1.6);
  `babylon.engine.simulation_engine.step`, `_restore_graph_context`, `_DEFAULT_ENGINE`;
  `babylon.engine.services.ServiceContainer`; `babylon.engine.context.TickContext`;
  `TICK_DYNAMICS_KEY`, `_reconstruct_tick_state`
  (`domain/economics/tick/graph_bridge.py`, both in that module's `__all__`);
  `DISTRIBUTION_EPSILON` (`distribution/types.py`);
  `create_wayne_county_scenario` (`babylon.engine.scenarios`, **existing** — it
  already returns exactly the `(WorldState, SimulationConfig, GameDefines)`
  triple this task needs; `_legacy_wayne.py:577`).
- Produces: `build_wayne_world_state()` in `tests/integration/economics/conftest.py`
  (Step 1b — a plain module-level function, deliberately **not** a pytest
  fixture, so `_run_to_year_boundary_capturing_graph` can import it by module
  path); plus the standing proof of U1's three acceptance criteria. Nothing else
  consumes them.

> **County coverage is this task's job, and only this task's.** Task U1.3b opened the
> `melt_calculator is None` gate for `qa:regression`, but those five scenarios are
> structurally abstract — none of the factories in `src/babylon/engine/scenarios/_legacy.py`
> passes a `county_fips`, so `_get_territory_fips` falls back to graph-local labels (`T001`,
> `T002`) and `ValueTensor4x3.fips_code`'s `^\d{5}$` validation makes a real-FIPS
> `TensorRegistry` unreachable from them. U1.3b therefore proves the pipeline *executes*
> (national layer, real τ) and can prove nothing about the *county* layer. `s = p + i + r + t`
> is a per-county identity: this task, over the real Wayne County scenario and the real
> reference DB, is the only place it can be observed at all. Do not read U1.3b's green as
> coverage of SC-001.

> **Observation-point note (read before writing Step 1 — it is why this test does
> NOT call `read_tick_state_from_graph` on a post-`step` `WorldState`).**
> Three separate mechanisms destroy the quantities this task must observe if the
> tests read them off `step(...)`'s returned `WorldState`:
>
> 1. **`tick_dynamics` is not a `WorldState` field.** `WorldState.to_graph`
>    (`models/world_state.py:608`) stamps `economy`, `state_finances`,
>    `contradiction_frames`, `opposition_states`, `events`, `event_log`,
>    `institution_relations`, the field stack, `player_org_id`,
>    `wealth_distribution`, `market`, `market_county` — and nothing else.
>    `graph.graph["tick_dynamics"]` lives only inside a tick; it is ferried
>    across the boundary in `persistent_context["_tick_dynamics"]` by
>    `_save_graph_context` (`engine/simulation_engine.py:446-464`) and replayed by
>    `_restore_graph_context` (`:428-443`). So
>    `read_tick_state_from_graph(returned_state.to_graph())` reads
>    `get_graph_attr("tick_dynamics")` → `None` → returns `None`.
> 2. **`tick_`-prefixed node attrs do not survive `from_graph`.**
>    `_reconstruct_territory` (`models/world_state.py:232-252`) filters out every
>    key that `startswith(("tick_", "flow_"))`, so a `Territory` carries no
>    `tick_ground_rent` and `to_graph` cannot re-emit one.
> 3. **`read_tick_state_from_graph` drops the Vol III payload even on a live
>    graph.** Its per-territory reconstruction
>    (`domain/economics/tick/graph_bridge.py:201-290`) builds each
>    `CountyEconomicState` from an explicit field list that contains **no**
>    `surplus_distribution` and **no** `rent_extraction`. Its `county_states`
>    fallback fires only when *no* territory node carries `tick_capital_stock` —
>    which is exactly not the case on a freshly-ticked graph. Reading through it
>    would report "the Vol III county layer is still dark" against perfectly
>    correct U1 wiring.
>
> The honest observation point is therefore the **live post-tick graph's**
> `tick_dynamics` attribute, whose `"county_states"` entry holds the real
> `CountyEconomicState` objects `write_tick_state_to_graph` put there
> (`graph_bridge.py:57-68`), reconstructed with `_reconstruct_tick_state`
> (`graph_bridge.py:292-325`) — the same helper `Simulation.get_time_series()`
> uses. `_run_to_year_boundary_capturing_graph()` below returns that graph.
> **Do not "simplify" it back to `_run_to_year_boundary().to_graph()`.**

- [ ] **Step 1: Write the failing test**
```python
"""U1 acceptance (design §4): `s = p + i + r + t` has NEVER evaluated in a
shipped run. This is the first test that makes it do so end-to-end, over the
real reference DB, and pins SC-001 — the identity holds within
DISTRIBUTION_EPSILON for 100% of observations, not merely on average.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest

from babylon.domain.economics.distribution.types import DISTRIBUTION_EPSILON
from babylon.domain.economics.tick.graph_bridge import (
    TICK_DYNAMICS_KEY,
    _reconstruct_tick_state,
)

if TYPE_CHECKING:
    from babylon.domain.economics.tick.types import SimulationTickState
    from babylon.topology.graph import BabylonGraph

pytestmark = [pytest.mark.integration, pytest.mark.requires_reference_db]

WAYNE_FIPS = "26163"
#: Ticks driven through ``step`` before the capturing pass. Tick 0 is itself a
#: year boundary (bootstraps 2010); after 52 calls ``state.tick == 52``, which is
#: the NEXT boundary — the tick the capturing pass runs and observes (2011).
_TICKS_BEFORE_CAPTURE = 52


def _run_to_year_boundary_capturing_graph() -> BabylonGraph:
    """Run a Wayne-scoped simulation to a year boundary; return the LIVE graph.

    The final tick is executed through the same three calls
    ``simulation_engine.step`` makes (`simulation_engine.py:519-537`) —
    ``to_graph`` → ``_restore_graph_context`` → ``_DEFAULT_ENGINE.run_tick`` —
    rather than through ``step`` itself, for one reason only: ``step`` discards
    the graph and returns a ``WorldState``, and the round trip that produces that
    ``WorldState`` destroys every quantity this task exists to observe (see the
    observation-point note in the plan). Same engine, same systems, same order;
    the only difference is that the graph the systems wrote is handed back
    instead of thrown away.
    """
    from babylon.engine.context import TickContext
    from babylon.engine.headless_runner.runner import _build_economics_overrides
    from babylon.engine.services import ServiceContainer
    from babylon.engine.simulation_engine import (
        _DEFAULT_ENGINE,
        _restore_graph_context,
        step,
    )
    from babylon.reference.database import get_normalized_session_factory
    from tests.integration.economics.conftest import build_wayne_world_state

    overrides, leontief_session = _build_economics_overrides(
        session_factory=get_normalized_session_factory(),
        scope_fips=frozenset({WAYNE_FIPS}),
    )
    try:
        state, sim_config, defines = build_wayne_world_state()
        persistent: dict[str, object] = {}
        for _ in range(_TICKS_BEFORE_CAPTURE):
            state = step(
                state, sim_config, persistent, defines, calculator_overrides=overrides
            )
        assert state.tick == _TICKS_BEFORE_CAPTURE, (
            f"expected tick {_TICKS_BEFORE_CAPTURE} before the capturing pass, "
            f"got {state.tick} — the capturing pass must land on a year boundary"
        )
        graph = state.to_graph()
        _restore_graph_context(graph, persistent)
        services = ServiceContainer.create(sim_config, defines, **overrides)
        context = TickContext(tick=state.tick, persistent_data=dict(persistent))
        _DEFAULT_ENGINE.run_tick(graph, services, context)
        return graph
    finally:
        if leontief_session is not None:
            leontief_session.close()


def _tick_state_from(graph: BabylonGraph) -> SimulationTickState:
    """Read the published tick state off a LIVE post-tick graph.

    Deliberately NOT ``read_tick_state_from_graph``: that function rebuilds each
    ``CountyEconomicState`` from ``tick_``-prefixed territory-node attrs using a
    field list that carries neither ``surplus_distribution`` nor
    ``rent_extraction`` (`graph_bridge.py:201-290`), so it would report a dark
    Vol III layer no matter how correct U1's wiring is. ``_reconstruct_tick_state``
    reads the real objects straight out of the graph attribute.
    """
    tick_data = graph.get_graph_attr(TICK_DYNAMICS_KEY)
    assert tick_data is not None, (
        f"{TICK_DYNAMICS_KEY} was never published onto the graph — "
        "TickDynamicsSystem did not complete a year-boundary pass"
    )
    tick_state = _reconstruct_tick_state(tick_data)
    assert tick_state is not None, f"{TICK_DYNAMICS_KEY} published an empty payload"
    return tick_state


def test_surplus_distribution_is_non_none_for_at_least_one_county_year() -> None:
    """The headline U1 criterion: the county financial layer actually fires."""
    tick_state = _tick_state_from(_run_to_year_boundary_capturing_graph())
    live = [
        county
        for county in tick_state.county_states.values()
        if county.surplus_distribution is not None
    ]
    assert live, (
        "surplus_distribution is None for every county after crossing a year "
        "boundary — the Vol III county layer is still dark (design §1.1)"
    )


def test_sc001_identity_holds_for_one_hundred_percent_of_observations() -> None:
    """SC-001: s = p + i + r + t within DISTRIBUTION_EPSILON, every observation.

    Asserted as a universal, not a sample statistic: one violating county-year
    is a violated accounting identity, however many others hold.
    """
    tick_state = _tick_state_from(_run_to_year_boundary_capturing_graph())
    observed = 0
    violations: list[tuple[str, float]] = []
    for fips, county in sorted(tick_state.county_states.items()):
        d = county.surplus_distribution
        if d is None:
            continue
        observed += 1
        residual = abs(
            d.total_surplus_produced
            - (
                d.profit_of_enterprise
                + d.interest_payments
                + d.ground_rent
                + d.taxes_on_surplus
            )
        )
        if residual > DISTRIBUTION_EPSILON:
            violations.append((fips, residual))
    assert not violations, f"SC-001 violated (residual > {DISTRIBUTION_EPSILON}): {violations}"
    # A universal over an empty domain is vacuously true — SC-001 is only
    # proven if the run actually produced distributions to check.
    assert observed > 0, (
        "SC-001 checked zero observations: no county-year carried a "
        "surplus_distribution, so the identity was never exercised"
    )


def test_tick_ground_rent_carries_a_non_zero_real_figure() -> None:
    """U1.5's repoint proved in a real run, not against a hand-built model.

    Read off the LIVE post-tick graph: ``_reconstruct_territory``
    (`models/world_state.py:232-252`) strips every ``tick_``-prefixed attr on the
    way back into a ``WorldState``, so a post-``step`` ``to_graph()`` would show
    ``0.0`` here for every territory regardless of U1.5.
    """
    graph = _run_to_year_boundary_capturing_graph()
    rents = [
        graph.nodes[node_id].get("tick_ground_rent", 0.0)
        for node_id in graph.nodes
        if graph.nodes[node_id].get("_node_type") == "territory"
    ]
    assert rents, "no territory nodes on the graph — the run built no counties"
    assert any(rent > 0.0 for rent in rents), (
        "every tick_ground_rent is 0.0 — the Path A repoint (U1.5) did not "
        "reach a real FRED B230RC0Q173SBEA-backed figure in a live run"
    )
```
- [ ] **Step 1b: Add the Wayne world-state builder to the shared conftest**

The test above imports `build_wayne_world_state` from
`tests/integration/economics/conftest.py`. **That helper does not exist yet** — it
must be written before Step 2, or the module raises `ImportError` at collection and
Step 2's red is unreachable. It is a plain module-level function, not a pytest
fixture, precisely so the test can import it by module path.

Edit `tests/integration/economics/conftest.py` — add `TYPE_CHECKING` to the stdlib
imports, immediately after the existing `from pathlib import Path`:
```python
from pathlib import Path
from typing import TYPE_CHECKING
```

Then, immediately after the existing `from babylon.domain.economics.hydrator import
MarxianHydrator` line (i.e. after the last top-level `babylon` import, before the
`# ===` DATABASE PATH CANDIDATES banner), add the type-only import block:
```python
if TYPE_CHECKING:
    from babylon.config.defines import GameDefines
    from babylon.models.config import SimulationConfig
    from babylon.models.world_state import WorldState
```

Then append this section to the END of the file (after the closing
`# IMPERIAL RENT FIXTURES (REMOVED post-Spec 057)` comment block):
```python
# =============================================================================
# WAYNE-SCOPED WORLD STATE (U1.9 acceptance — vol3-money-scissors)
# =============================================================================

#: Wayne County, Michigan. The single county U1's acceptance run computes over,
#: and the FIPS U1.6 hydrates the ``TensorRegistry`` for.
WAYNE_COUNTY_FIPS = "26163"


def build_wayne_world_state() -> tuple[WorldState, SimulationConfig, GameDefines]:
    """Build a Wayne-County-scoped world the engine can tick past a year boundary.

    Reuse over recreation: :func:`babylon.engine.scenarios.create_wayne_county_scenario`
    (``engine/scenarios/_legacy_wayne.py:577``) already returns exactly the
    ``(WorldState, SimulationConfig, GameDefines)`` triple, already populated with
    the H3 res-5 hex territories, social classes, relationships and the player /
    state-apparatus organizations that the 30 systems need in order to run. This
    helper does ONE thing on top of it, and that one thing is load-bearing for
    U1: it stamps the real county identity onto every territory.

    Why the stamp is required: ``TickDynamicsSystem._get_territory_fips``
    (``domain/economics/tick/system/__init__.py:366-382``) derives the county key
    from ``node.attributes.get("county_fips") or node.id``. The stock Wayne
    scenario sets no ``county_fips`` at all, so every H3 cell id (``85...fffff``)
    becomes its own pseudo-county. U1.6 hydrates the ``TensorRegistry`` for
    ``"26163"`` and nothing else, so every one of those pseudo-counties would miss
    the registry, ``_get_county_surplus`` would return ``None``, and
    ``surplus_distribution`` would stay ``None`` — for a reason that has nothing
    to do with the wiring this task exists to prove. That is exactly the
    green-test-over-a-dead-feature trap the fixture-vocabulary rule warns about.

    ``county_fips`` is a real ``Territory`` model field
    (``models/entities/territory.py:75``), ``WorldState.to_graph`` writes it onto
    the node via ``**territory.model_dump()``, and it is NOT in
    ``TERRITORY_EXCLUDED_FIELDS`` — so it survives the ``to_graph``/``from_graph``
    round trip that ``simulation_engine.step`` performs on every tick.

    Tick 0 is deliberate: ``0 % WEEKS_PER_YEAR == 0``, so the very first ``step``
    is a year-boundary tick that bootstraps county states at the default
    ``base_year`` of 2010 (``_determine_year``, same module ``:350-364``); the
    caller then drives 52 ``step`` calls (ticks 0-51) and executes tick 52 — the
    next boundary — as its capturing pass, recomputing at 2011. Both years
    sit inside U1.6's ``_TENSOR_HYDRATION_YEARS`` (2010-2024), so both find real
    QCEW/FRED-backed data rather than a ``NoDataSentinel``.

    Determinism (Constitution III.7): every input is fixed — the scenario's
    default ``extraction_efficiency``/``repression_level``, the default
    ``SimulationConfig`` (and therefore the default ``rng_seed``), and a
    deterministic dict comprehension over ``state.territories``. Two calls
    produce equal worlds.

    Returns:
        ``(state, config, defines)`` — a tick-0 ``WorldState`` whose every
        territory carries ``county_fips == "26163"``, the scenario's
        ``SimulationConfig``, and the scenario's ``GameDefines``.
    """
    from babylon.engine.scenarios import create_wayne_county_scenario

    state, config, defines = create_wayne_county_scenario()

    territories = {
        territory_id: territory.model_copy(update={"county_fips": WAYNE_COUNTY_FIPS})
        for territory_id, territory in state.territories.items()
    }
    # WorldState is frozen — mutate via model_copy, never assignment.
    scoped_state = state.model_copy(update={"territories": territories, "tick": 0})
    return scoped_state, config, defines
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/integration/economics/test_vol3_surplus_distribution_live.py`
Expected: FAIL **at an assertion, not at collection** — Step 1b must already have
landed, so the module imports cleanly and the 52 `step` calls plus the capturing
pass genuinely execute.
`test_surplus_distribution_is_non_none_for_at_least_one_county_year` fails with
`AssertionError: surplus_distribution is None for every county after crossing a
year boundary — the Vol III county layer is still dark (design §1.1)` when run
against a checkout WITHOUT U1.6/U1.7 (verify by `git stash`-ing those two commits,
running, then `git stash pop`). This is the characterization proof that the
criterion was previously unmet.

If you instead see `ImportError: cannot import name 'build_wayne_world_state' from
'tests.integration.economics.conftest'`, Step 1b was skipped or its append landed
in the wrong file. Go back and do Step 1b. Do NOT proceed, and do NOT "fix" it by
deleting the import — a collection error is not a red phase.
- [ ] **Step 3: Write minimal implementation**
No new production code — U1.2–U1.8 supply every line. (Step 1b's conftest builder
is test-support scaffolding, not production code; it is already written.) If any
test here fails after U1.8 has landed, the defect is in U1.6/U1.7/U1.8's wiring,
not in this test. Do NOT weaken an assertion to make it pass.
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/integration/economics/test_vol3_surplus_distribution_live.py`
Expected: PASS (3 passed).
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(economics): U1 acceptance — s = p + i + r + t evaluates live and holds (SC-001)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U2.1: Stop fabricating year=2040 on the live MELT path (NationalTickParameters year ceiling)

**Files:**
- Modify: `src/babylon/domain/economics/tick/types.py:228`
- Modify: `src/babylon/domain/economics/tick/system/__init__.py:526-540`
- Test: `tests/unit/economics/tick/test_types.py:27-125`
- Test: `tests/unit/economics/tick/test_system.py:1790-1799`

**Interfaces:**
- Consumes: `NationalTickParameters` (`babylon.domain.economics.tick.types`), `TickDynamicsSystem._compute_national_params` (`babylon.domain.economics.tick.system`), `MockMELTCalculator(tau, accept_any_year=True)` (`tests.unit.economics.tick.conftest`).
- Produces: `NationalTickParameters.year` with no upper `Field` bound — later units (U3+) may rely on `national_params.year` carrying the *real* simulation year past 2040, not a fabricated `2040`.

- [ ] **Step 1: Write the failing tests**
```python
# --- tests/unit/economics/tick/test_types.py ---
# Replace the existing `test_year_bounds` method (lines 62-85) on
# TestNationalTickParameters with:

    def test_year_floor_bound(self) -> None:
        """Verify year floor ge=2007 (no upper bound — MELT runs the whole campaign)."""
        with pytest.raises(ValidationError, match="year"):
            NationalTickParameters(
                year=2006,
                tau=62.0,
                gamma_basket=0.68,
                gamma_basket_raw=0.68,
                gamma_III=0.33,
                gamma_III_raw=0.33,
                tau_effective=42.16,
                v_reproduction=12.0,
            )

    def test_year_has_no_upper_bound(self) -> None:
        """Honesty sweep (spec 2026-07-18 vol3-money-scissors-design, U2):

        MELT/gamma_basket/gamma_III are computed for the WHOLE campaign
        horizon (SIM_EPOCH_YEAR=2010 .. +100yr), unlike Volume III's
        FRED-bound financial models — so this model must accept a real
        late-campaign year instead of the caller clamping/fabricating one.
        """
        params = NationalTickParameters(
            year=2109,
            tau=62.0,
            gamma_basket=0.68,
            gamma_basket_raw=0.68,
            gamma_III=0.33,
            gamma_III_raw=0.33,
            tau_effective=42.16,
            v_reproduction=12.0,
        )
        assert params.year == 2109


# --- tests/unit/economics/tick/test_system.py ---
# Replace the existing `test_year_clamping_to_2040` method (lines 1790-1799)
# on TestComputeNationalParams with (test_year_clamping_to_2007 stays
# UNCHANGED):

    def test_year_passes_through_unclamped_above_2040(self) -> None:
        """Honesty sweep: year > 2040 is NOT clamped/fabricated to 2040 — it
        flows through as the real simulation year (NationalTickParameters
        has no upper year bound; MELT/gamma compute for the whole campaign).
        """
        system = TickDynamicsSystem()
        melt = MockMELTCalculator(tau=62.0, accept_any_year=True)
        services = _make_services(melt_calculator=melt)

        result = system._compute_national_params(2050, services, prev_coefficients=None)

        assert result is not None
        assert result.year == 2050
```
- [ ] **Step 2: Run tests to verify they fail**
Run: `mise run test:q -- tests/unit/economics/tick/test_types.py::TestNationalTickParameters::test_year_has_no_upper_bound tests/unit/economics/tick/test_system.py::TestComputeNationalParams::test_year_passes_through_unclamped_above_2040`
Expected: FAIL — `test_year_has_no_upper_bound` raises `pydantic.ValidationError: ... Input should be less than or equal to 2040 [type=less_than_equal, ...]`; `test_year_passes_through_unclamped_above_2040` fails with `assert 2040 == 2050`.
- [ ] **Step 3: Write minimal implementation**
```python
# src/babylon/domain/economics/tick/types.py:228
# BEFORE:
#     year: int = Field(..., ge=2007, le=2040, description="Parameter year")
# AFTER:
    year: int = Field(..., ge=2007, description="Parameter year")


# src/babylon/domain/economics/tick/system/__init__.py:526-540
# BEFORE:
#         tau_effective = tau * gamma_basket
#
#         clamped_year = min(max(year, 2007), 2040)
#         return NationalTickParameters(
#             year=clamped_year,
#             ...
# AFTER:
        tau_effective = tau * gamma_basket

        # Honesty sweep (spec 2026-07-18 vol3-money-scissors-design, U2):
        # NationalTickParameters carries MELT/gamma — quantities computed
        # for the WHOLE campaign horizon (SIM_EPOCH_YEAR + up to ~100
        # years), unlike Volume III's financial models whose FRED/Z.1 data
        # legitimately stops around 2024-2040. The prior
        # `min(max(year, 2007), 2040)` ceiling silently fabricated
        # year=2040 for every tick past 2040 (~85% of a 5200-tick
        # campaign) instead of reporting the real year (Constitution
        # III.11) — and the `le=2040` Field constraint it was masking
        # raised ValidationError the moment this clamp was ever bypassed.
        # Only the floor is a genuine sanity bound (SIM_EPOCH_YEAR=2010
        # can never produce year < 2007); the ceiling is simply gone.
        floor_year = max(year, 2007)
        return NationalTickParameters(
            year=floor_year,
            tau=tau,
            gamma_basket=gamma_basket,
            gamma_basket_raw=gamma_basket_raw,
            gamma_III=gamma_III,
            gamma_III_raw=gamma_III_raw,
            tau_effective=tau_effective,
            v_reproduction=DEFAULT_V_REPRODUCTION,
            estimated=estimated,
        )
```
- [ ] **Step 4: Run tests to verify they pass**
Run: `mise run test:q -- tests/unit/economics/tick/test_types.py tests/unit/economics/tick/test_system.py::TestComputeNationalParams`
Expected: PASS (including `test_year_clamping_to_2007`, unchanged, still green)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(economics): stop fabricating year=2040 past the live MELT-path ceiling

NationalTickParameters.year was clamped to [2007,2040], silently
relabeling every tick past year 2040 (~85% of a 5200-tick campaign) as
2040 instead of reporting the real year. MELT/gamma compute for the
whole campaign horizon, unlike Volume III's FRED-bound models, so the
le=2040 bound was simply wrong for this model. Drop the upper bound;
keep the ge=2007 floor as a genuine sanity check.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.2: Degrade Volume III year-window overruns to NoDataSentinel instead of raising

**Files:**
- Modify: `src/babylon/domain/economics/tensor.py:131` (insert after `NoDataSentinel`, before `class DepartmentRow`)
- Modify: `src/babylon/domain/economics/credit/interest.py:20,67-77`
- Modify: `src/babylon/domain/economics/credit/fictitious_capital.py:23,91-101`
- Modify: `src/babylon/domain/economics/distribution/calculator.py:28,107-132`
- Modify: `src/babylon/domain/economics/distribution/types.py:1-11,159-191`
- Test: `tests/unit/economics/test_tensor.py`
- Test: `tests/unit/economics/credit/test_interest.py`
- Test: `tests/unit/economics/credit/test_fictitious_capital_calc.py`
- Test: `tests/unit/economics/distribution/test_calculator.py`
- Test: `tests/unit/economics/distribution/test_debt_accumulation.py`

**Interfaces:**
- Consumes: `NoDataSentinel(fips, year, reason)` (`babylon.domain.economics.tensor`); `DefaultInterestCalculator.compute_interest_rate_state`, `DefaultFictitiousCapitalCalculator.compute_fictitious_capital`, `DefaultDistributionCalculator.compute_distribution`, `DebtAccumulation.update` (all pre-existing signatures, unchanged).
- Produces: `MODELED_YEAR_FLOOR: Final[int]`, `MODELED_YEAR_CEILING: Final[int]`, `year_within_modeled_range(year: int) -> bool` in `babylon.domain.economics.tensor` — the shared guard U2.3+ and any future Volume III caller must use instead of re-declaring `2007`/`2040` literals.

- [ ] **Step 1: Write the failing tests**
```python
# --- tests/unit/economics/test_tensor.py (append) ---
from babylon.domain.economics.tensor import (
    MODELED_YEAR_CEILING,
    MODELED_YEAR_FLOOR,
    year_within_modeled_range,
)


class TestYearWithinModeledRange:
    """year_within_modeled_range boundary contract (honesty sweep, U2)."""

    def test_floor_and_ceiling_are_2007_and_2040(self) -> None:
        assert MODELED_YEAR_FLOOR == 2007
        assert MODELED_YEAR_CEILING == 2040

    def test_boundaries_are_inclusive(self) -> None:
        assert year_within_modeled_range(2007) is True
        assert year_within_modeled_range(2040) is True

    def test_outside_window_is_false(self) -> None:
        assert year_within_modeled_range(2006) is False
        assert year_within_modeled_range(2041) is False
        assert year_within_modeled_range(2109) is False


# --- tests/unit/economics/credit/test_interest.py (append to TestComputeInterestRateState) ---
    def test_year_outside_modeled_range_returns_sentinel_even_with_real_data(
        self,
    ) -> None:
        """Guard fires BEFORE any data lookup — proven by giving 2050 real data."""
        source = MockInterestRateSource(data={2050: (0.03, 0.04, 0.02)})
        calc = DefaultInterestCalculator(rate_source=source)
        result = calc.compute_interest_rate_state(2050)
        assert isinstance(result, NoDataSentinel)
        assert "modeled" in result.reason.lower()


# --- tests/unit/economics/credit/test_fictitious_capital_calc.py (append to TestComputeFictitiousCapital) ---
    def test_year_outside_modeled_range_returns_sentinel_even_with_real_data(
        self,
    ) -> None:
        """Guard fires BEFORE any data lookup — proven by giving 2050 real data."""
        credit_source = MockCreditAggregateSource(
            data={2050: (90_000_000_000_000.0, 32_000_000_000_000.0, 34_000_000_000_000.0)}
        )
        z1_source = MockZ1Source(
            data={2050: (13_000_000_000_000.0, 19_000_000_000_000.0, 640_000_000_000_000.0)}
        )
        calc = DefaultFictitiousCapitalCalculator(credit_source=credit_source, z1_source=z1_source)
        result = calc.compute_fictitious_capital(2050)
        assert isinstance(result, NoDataSentinel)
        assert "modeled" in result.reason.lower()


# --- tests/unit/economics/distribution/test_calculator.py (append to TestComputeDistributionNoData) ---
    def test_year_outside_modeled_range_returns_sentinel_even_with_real_data(
        self,
    ) -> None:
        """Guard fires BEFORE any data lookup — proven by giving 2050 real data."""
        rental_source = MockRentalIncomeSource(data={("26163", 2050): 3_000_000_000.0})
        tax_source = MockTaxOnSurplusSource(data={("26163", 2050): 1_500_000_000.0})
        interest_source = MockInterestIncomeSource(data={2050: 3_500_000_000_000.0})
        calc = DefaultDistributionCalculator(
            rental_source=rental_source, tax_source=tax_source, interest_source=interest_source
        )
        result = calc.compute_distribution(
            fips="26163",
            year=2050,
            total_surplus=10_000_000_000.0,
            county_profit_rate=0.05,
            national_interest_rate=0.04,
        )
        assert isinstance(result, NoDataSentinel)
        assert "modeled" in result.reason.lower()

    def test_zero_surplus_still_honors_year_guard(
        self,
        mock_rental_source: MockRentalIncomeSource,
        mock_tax_source: MockTaxOnSurplusSource,
        mock_interest_source: MockInterestIncomeSource,
    ) -> None:
        """The zero-surplus bypass branch must NOT skip the year-range check."""
        calc = DefaultDistributionCalculator(
            rental_source=mock_rental_source,
            tax_source=mock_tax_source,
            interest_source=mock_interest_source,
        )
        result = calc.compute_distribution(
            fips="26163",
            year=2050,
            total_surplus=0.0,
            county_profit_rate=0.05,
            national_interest_rate=0.04,
        )
        assert isinstance(result, NoDataSentinel)


# --- tests/unit/economics/distribution/test_debt_accumulation.py (append) ---
@pytest.mark.unit
class TestDebtAccumulationYearOutOfRange:
    """Honest carry-forward (D1's "endogenous thereafter" principle), not a crash."""

    def test_out_of_range_new_year_returns_current_unchanged(self) -> None:
        current = DebtAccumulation(
            fips_code="26163",
            year=2024,
            accumulated_debt=750.0,
            consecutive_deficit_ticks=2,
        )
        updated = DebtAccumulation.update(current, enterprise_profit=-500.0, new_year=2050)
        assert updated is current
```
- [ ] **Step 2: Run tests to verify they fail**
Run: `mise run test:q -- tests/unit/economics/test_tensor.py::TestYearWithinModeledRange tests/unit/economics/credit/test_interest.py::TestComputeInterestRateState::test_year_outside_modeled_range_returns_sentinel_even_with_real_data tests/unit/economics/credit/test_fictitious_capital_calc.py::TestComputeFictitiousCapital::test_year_outside_modeled_range_returns_sentinel_even_with_real_data tests/unit/economics/distribution/test_calculator.py::TestComputeDistributionNoData tests/unit/economics/distribution/test_debt_accumulation.py::TestDebtAccumulationYearOutOfRange`
Expected: FAIL — `ImportError: cannot import name 'MODELED_YEAR_FLOOR'` (test_tensor.py); the interest/fictitious/distribution tests fail with `assert isinstance(InterestRateState(...), NoDataSentinel)` / `AssertionError` since the calculators currently happily construct real models for year 2050; `test_out_of_range_new_year_returns_current_unchanged` fails with `assert updated is current` false (debt actually updated).
- [ ] **Step 3: Write minimal implementation**
```python
# src/babylon/domain/economics/tensor.py — insert after line 131 (end of
# NoDataSentinel class), before `class DepartmentRow`:

# =============================================================================
# VOLUME III MODELED-DATA YEAR WINDOW
# =============================================================================

MODELED_YEAR_FLOOR: Final[int] = 2007
"""Lower bound of Volume III's modeled financial-data year window.

Traceability: the FRED/Z.1 series backing Volume III (credit aggregates,
fictitious capital, surplus distribution — the interest rate is endogenous
post-U9 and reads no FRED at runtime) have no meaningful coverage before
2007.
"""

MODELED_YEAR_CEILING: Final[int] = 2040
"""Upper bound of Volume III's modeled financial-data year window.

Volume III structured models (credit cycle, fictitious capital, surplus
distribution, debt accumulation — the interest rate is endogenous post-U9
and reads no FRED at runtime) represent real FRED/Z.1 data (2010-2024)
extrapolated to this administratively-chosen horizon
(spec 2026-07-18 vol3-money-scissors-design, D1). Years outside this
window are honestly absent (:class:`NoDataSentinel`) rather than
fabricated — the endogenous price/fictitious scissors
(:mod:`babylon.engine.systems.market_scissors`) carries the money system
for the remainder of a campaign, unlike this structured layer.
"""


def year_within_modeled_range(year: int) -> bool:
    """Return True if ``year`` falls inside Volume III's modeled-data window.

    Args:
        year: Calendar year to check.

    Returns:
        True if ``MODELED_YEAR_FLOOR <= year <= MODELED_YEAR_CEILING``.
    """
    return MODELED_YEAR_FLOOR <= year <= MODELED_YEAR_CEILING


# src/babylon/domain/economics/credit/interest.py:20
# BEFORE: from babylon.domain.economics.tensor import NoDataSentinel
# AFTER:
from babylon.domain.economics.tensor import (
    MODELED_YEAR_CEILING,
    MODELED_YEAR_FLOOR,
    NoDataSentinel,
    year_within_modeled_range,
)

# src/babylon/domain/economics/credit/interest.py — locate the line
#     fed_funds = self._rate_source.get_federal_funds_rate(year)
# inside compute_interest_rate_state and insert the 9-line guard DIRECTLY
# ABOVE it. Nothing else in the method changes.
        if not year_within_modeled_range(year):
            return NoDataSentinel(
                fips="USA",
                year=year,
                reason=(
                    f"Year {year} outside Volume III modeled financial-data "
                    f"window [{MODELED_YEAR_FLOOR}, {MODELED_YEAR_CEILING}]"
                ),
            )


# src/babylon/domain/economics/credit/fictitious_capital.py:23
# BEFORE: from babylon.domain.economics.tensor import NoDataSentinel
# AFTER:
from babylon.domain.economics.tensor import (
    MODELED_YEAR_CEILING,
    MODELED_YEAR_FLOOR,
    NoDataSentinel,
    year_within_modeled_range,
)

# src/babylon/domain/economics/credit/fictitious_capital.py — locate
# `govt_debt = self._credit_source.get_government_debt(year)` and insert
# the 9-line guard DIRECTLY ABOVE it; nothing else in the method changes.
        if not year_within_modeled_range(year):
            return NoDataSentinel(
                fips="USA",
                year=year,
                reason=(
                    f"Year {year} outside Volume III modeled financial-data "
                    f"window [{MODELED_YEAR_FLOOR}, {MODELED_YEAR_CEILING}]"
                ),
            )


# src/babylon/domain/economics/distribution/calculator.py:28
# BEFORE: from babylon.domain.economics.tensor import NoDataSentinel
# AFTER:
from babylon.domain.economics.tensor import (
    MODELED_YEAR_CEILING,
    MODELED_YEAR_FLOOR,
    NoDataSentinel,
    year_within_modeled_range,
)

# src/babylon/domain/economics/distribution/calculator.py — locate the
# comment `# Zero surplus -> all-zero distribution` and insert the 9-line
# guard DIRECTLY ABOVE it; nothing else in the method changes.
        if not year_within_modeled_range(year):
            return NoDataSentinel(
                fips=fips,
                year=year,
                reason=(
                    f"Year {year} outside Volume III modeled financial-data "
                    f"window [{MODELED_YEAR_FLOOR}, {MODELED_YEAR_CEILING}]"
                ),
            )


# src/babylon/domain/economics/distribution/types.py:1-11 — add import:
from __future__ import annotations

from typing import Final

from pydantic import BaseModel, ConfigDict, Field, computed_field

from babylon.domain.economics.tensor import year_within_modeled_range

# src/babylon/domain/economics/distribution/types.py:159-191 — replace the
# `update` classmethod body:
    @classmethod
    def update(
        cls,
        current: DebtAccumulation,
        enterprise_profit: float,
        new_year: int,
    ) -> DebtAccumulation:
        """Create updated state based on current period profit.

        If profit < 0: debt increases by |profit|, deficit ticks increment.
        If profit >= 0: debt decreases by min(profit, debt), deficit ticks reset.

        Honest absence (Constitution III.11): if ``new_year`` falls outside
        Volume III's modeled financial-data window
        (:func:`babylon.domain.economics.tensor.year_within_modeled_range`),
        ``current`` is returned UNCHANGED — the debt state carries forward
        rather than raising a year-range ``ValidationError`` or fabricating
        a value for an unmodeled year (spec 2026-07-18
        vol3-money-scissors-design, D1's "endogenous thereafter" principle).

        Args:
            current: Current debt state.
            enterprise_profit: Enterprise profit for the period (may be negative).
            new_year: Calendar year for the new state.

        Returns:
            New DebtAccumulation reflecting the update, or ``current``
            unchanged if ``new_year`` is outside the modeled window.
        """
        if not year_within_modeled_range(new_year):
            return current
        if enterprise_profit < 0:
            new_debt = current.accumulated_debt + abs(enterprise_profit)
            new_ticks = current.consecutive_deficit_ticks + 1
        else:
            reduction = min(enterprise_profit, current.accumulated_debt)
            new_debt = current.accumulated_debt - reduction
            new_ticks = 0
        return cls(
            fips_code=current.fips_code,
            year=new_year,
            accumulated_debt=new_debt,
            consecutive_deficit_ticks=new_ticks,
        )
```
- [ ] **Step 4: Run tests to verify they pass**
Run: `mise run test:q -- tests/unit/economics/test_tensor.py tests/unit/economics/credit/test_interest.py tests/unit/economics/credit/test_fictitious_capital_calc.py tests/unit/economics/distribution/test_calculator.py tests/unit/economics/distribution/test_debt_accumulation.py`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(economics): degrade Volume III year-window overruns to NoDataSentinel

Interest rate, fictitious capital, and surplus distribution calculators
constructed their year-bounded (ge=2007,le=2040) Pydantic models with a
raw, unguarded year — currently masked only by FRED data absence past
2024, not by design. Add a shared year_within_modeled_range() guard
(tensor.py) that degrades out-of-range years to NoDataSentinel BEFORE
any data lookup or construction, and make DebtAccumulation.update carry
its state forward unchanged (never fabricated) for the same reason.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.3: `capital_vol3` GameDefines category — migrate the four Final constants + fix STAGNATION_CREDIT_GROWTH

**Files:**
- Create: `src/babylon/config/defines/capital_vol3.py`
- Modify: `src/babylon/config/defines/_assembler.py:16-83,97-129,193,320-323`
- Modify: `src/babylon/config/defines/__init__.py:15-83`
- Modify: `src/babylon/domain/economics/distribution/types.py:1-30,42,87`
- Modify: `src/babylon/domain/economics/distribution/__init__.py:13-44`
- Modify: `src/babylon/domain/economics/counter_tendencies/types.py:1-47,66,111-131`
- Modify: `src/babylon/domain/economics/counter_tendencies/__init__.py:11-29`
- Modify: `src/babylon/domain/economics/counter_tendencies/calculator.py:11`
- Modify: `src/babylon/domain/economics/credit/types.py:13,95`
- Modify: `src/babylon/domain/economics/credit/__init__.py:37-69`
- Modify: `src/babylon/domain/economics/credit/credit_cycle.py:18-23,61-147`
- Test: `tests/unit/config/test_capital_vol3_defines.py` (create)
- Test: `tests/unit/economics/distribution/test_distribution_types.py` (append)
- Test: `tests/unit/economics/counter_tendencies/test_types.py` (rewrite module import + 4 existing tests + append)
- Test: `tests/unit/economics/credit/test_credit_cycle.py` (rewrite import + 3 call sites)
- Test: `tests/unit/economics/distribution/test_calculator.py:59` (docstring reference only)
- Test: `tests/integration/economics/test_vol3_surplus_distribution_live.py` (U1.9's file — rewrite import + 2 call sites)

**Blast radius — verified by `rg` over `src tests` before authoring, not assumed.** This task
deletes five module-level names, and every one of them is re-exported from its package
`__init__.py` and listed in that package's `__all__`. Deleting them without threading the
packages breaks **package import**, not one call site: `from babylon.domain.economics.distribution
import ...` then raises `ImportError` at collection for every test under that tree. Three of the
five are also used *internally* by the module that declares them
(`distribution/types.py:87`, `counter_tendencies/types.py:119,131`,
`credit_cycle.py:126,143,145`), and four are used by already-shipped test files — including
`tests/integration/economics/test_vol3_surplus_distribution_live.py`, which **Task U1.9 ships
earlier in this same plan**. Every one of those sites is edited in Step 3 below.

**Interfaces:**
- Consumes: `GameDefines.load_default()` / `GameDefines.load_from_yaml()` (`babylon.config.defines`), the `_from_yaml_dict`/`build_yaml` auto-discovery over `GameDefines.model_fields` (`tools/generate_defines_config.py` — no changes needed there, it introspects the schema).
- Produces: `GameDefines.capital_vol3: CapitalVolumeIIIDefines` with fields `debt_spiral_threshold`, `distribution_epsilon`, `counter_tendency_weights`, `imperial_rent_reference_scale`, `profit_rate_fallback`, `national_county_count`, `default_rate_estimate`, `housing_capitalization_rate_default` — Task U2.4 wires the last four into `tick/system/__init__.py` and `factory.py` via `services.defines.capital_vol3.*`.
- Produces (renames): five module-level constants become defines-backed accessor **functions** —
  `distribution.types.debt_spiral_threshold()`, `distribution.types.distribution_epsilon()`,
  `counter_tendencies.types.counter_tendency_weights()`,
  `counter_tendencies.types.imperial_rent_reference_scale()`,
  `credit.types.stagnation_credit_growth()`. Each takes an optional `defines: GameDefines | None`
  and falls back to the process-cached default. The ALL-CAPS names are **deleted**; the package
  `__all__`s carry the lowercase names instead.

- [ ] **Step 0: Enumerate every call site before editing**
Run this sweep and reconcile its output against this task's **Files** block before editing a single
line. Scope is `src tests` — **not** `src/babylon`: four of the call sites are in already-shipped
test files, and one of those files is shipped by Task U1.9 of this same plan.
```bash
rg -n "DEBT_SPIRAL_THRESHOLD|DISTRIBUTION_EPSILON|COUNTER_TENDENCY_WEIGHTS|IMPERIAL_RENT_REFERENCE_SCALE|STAGNATION_CREDIT_GROWTH" src tests
```
Expected: 23 matches across 7 files at the time of authoring, plus the U1.9 integration test's 3.
If the sweep reports a file that is **not** in the Files block, STOP and add it — the accessor
conversion must thread every one, and an unthreaded package `__init__.py` is an `ImportError` at
collection, not a localized failure.
- [ ] **Step 1: Write the failing tests**
```python
# --- tests/unit/config/test_capital_vol3_defines.py (create) ---
"""GameDefines.capital_vol3 contract — Volume III financial-claims coefficients.

Honesty sweep (spec 2026-07-18 vol3-money-scissors-design, U2): pins the
defaults migrated off module-level Final constants in distribution/types.py,
counter_tendencies/types.py and credit/types.py — all five now defines-backed
accessor functions (moddability, Constitution III.1).
"""

from __future__ import annotations

import pytest

from babylon.config.defines import CapitalVolumeIIIDefines, GameDefines

pytestmark = pytest.mark.unit


class TestCapitalVolumeIIIDefaults:
    def test_defaults_match_migrated_constants(self) -> None:
        d = CapitalVolumeIIIDefines()
        assert d.debt_spiral_threshold == pytest.approx(0.5)
        assert d.distribution_epsilon == pytest.approx(1e-9)
        assert d.counter_tendency_weights == [0.20, 0.15, 0.15, 0.15, 0.20, 0.15]
        assert d.imperial_rent_reference_scale == pytest.approx(500_000_000_000.0)
        assert d.profit_rate_fallback == pytest.approx(0.05)
        assert d.national_county_count == 3300
        assert d.default_rate_estimate == pytest.approx(0.02)
        assert d.housing_capitalization_rate_default == pytest.approx(0.05)

    def test_reachable_from_game_defines(self) -> None:
        defines = GameDefines.load_default()
        assert defines.capital_vol3.debt_spiral_threshold == pytest.approx(0.5)


class TestStagnationCreditGrowthIsAnAccessor:
    def test_no_import_time_snapshot_remains(self) -> None:
        """credit/types.py must expose an accessor, not a module-level Final
        snapshot — an import-time snapshot reads defines.yaml on every process
        start and freezes the value before any runtime override can reach it.

        Pinned at source level rather than by importlib.reload: reloading a
        module other already-imported modules hold references to leaves them
        bound to stale class objects, which corrupts sibling tests under the
        xdist workers test:unit runs on.
        """
        from pathlib import Path

        import babylon.domain.economics.credit.types as credit_types

        source = Path(str(credit_types.__file__)).read_text(encoding="utf-8")
        assert "def stagnation_credit_growth(" in source
        assert "STAGNATION_CREDIT_GROWTH" not in source
        assert "GameDefines().crisis.stagnation_credit_growth" not in source

    def test_value_matches_the_canonical_yaml(self) -> None:
        from babylon.domain.economics.credit.types import stagnation_credit_growth

        assert stagnation_credit_growth() == pytest.approx(
            GameDefines.load_default().crisis.stagnation_credit_growth
        )

    def test_explicit_defines_override_is_honoured(self) -> None:
        """The whole point of the accessor: a caller-supplied GameDefines wins."""
        from babylon.domain.economics.credit.types import stagnation_credit_growth

        base = GameDefines.load_default()
        overridden = base.model_copy(
            update={"crisis": base.crisis.model_copy(update={"stagnation_credit_growth": 0.123})}
        )
        assert stagnation_credit_growth(overridden) == pytest.approx(0.123)


# --- tests/unit/economics/distribution/test_distribution_types.py (append) ---
class TestThresholdAccessorsAreGameDefinesBacked:
    """Honesty sweep (U2): the DEBT_SPIRAL_THRESHOLD/DISTRIBUTION_EPSILON
    Finals are gone; debt_spiral_threshold()/distribution_epsilon() read from
    GameDefines.capital_vol3 at call time, not at import time."""

    def test_debt_spiral_threshold_matches_capital_vol3(self) -> None:
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.distribution.types import debt_spiral_threshold

        assert (
            debt_spiral_threshold()
            == GameDefines.load_default().capital_vol3.debt_spiral_threshold
        )

    def test_distribution_epsilon_matches_capital_vol3(self) -> None:
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.distribution.types import distribution_epsilon

        assert (
            distribution_epsilon()
            == GameDefines.load_default().capital_vol3.distribution_epsilon
        )

    def test_explicit_defines_override_is_honoured(self) -> None:
        """A caller-supplied GameDefines wins over the process default — the
        behaviour the deleted module-level Finals made impossible."""
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.distribution.types import (
            debt_spiral_threshold,
            distribution_epsilon,
        )

        base = GameDefines.load_default()
        overridden = base.model_copy(
            update={
                "capital_vol3": base.capital_vol3.model_copy(
                    update={"debt_spiral_threshold": 0.75, "distribution_epsilon": 1e-6}
                )
            }
        )
        assert debt_spiral_threshold(overridden) == pytest.approx(0.75)
        assert distribution_epsilon(overridden) == pytest.approx(1e-6)


# --- tests/unit/economics/counter_tendencies/test_types.py (append) ---
class TestWeightAccessorsAreGameDefinesBacked:
    """Honesty sweep (U2): the COUNTER_TENDENCY_WEIGHTS /
    IMPERIAL_RENT_REFERENCE_SCALE Finals are gone; counter_tendency_weights()
    and imperial_rent_reference_scale() read GameDefines.capital_vol3 at call
    time, not at import time."""

    def test_counter_tendency_weights_match_capital_vol3(self) -> None:
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.counter_tendencies.types import counter_tendency_weights

        assert (
            counter_tendency_weights()
            == GameDefines.load_default().capital_vol3.counter_tendency_weights
        )

    def test_imperial_rent_reference_scale_matches_capital_vol3(self) -> None:
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.counter_tendencies.types import (
            imperial_rent_reference_scale,
        )

        assert (
            imperial_rent_reference_scale()
            == GameDefines.load_default().capital_vol3.imperial_rent_reference_scale
        )

    def test_explicit_defines_override_is_honoured(self) -> None:
        """A caller-supplied GameDefines wins over the process default."""
        from babylon.config.defines import GameDefines
        from babylon.domain.economics.counter_tendencies.types import (
            counter_tendency_weights,
            imperial_rent_reference_scale,
        )

        base = GameDefines.load_default()
        overridden = base.model_copy(
            update={
                "capital_vol3": base.capital_vol3.model_copy(
                    update={
                        "counter_tendency_weights": [0.5, 0.1, 0.1, 0.1, 0.1, 0.1],
                        "imperial_rent_reference_scale": 1_000.0,
                    }
                )
            }
        )
        assert counter_tendency_weights(overridden) == [0.5, 0.1, 0.1, 0.1, 0.1, 0.1]
        assert imperial_rent_reference_scale(overridden) == pytest.approx(1_000.0)
```
**Also rewrite the existing call sites in this file** — `test_types.py` imports
`COUNTER_TENDENCY_WEIGHTS` at module level (line 16) and `IMPERIAL_RENT_REFERENCE_SCALE` inside two
test bodies (lines 154, 202). Those names cease to exist in Step 3, so this file is part of the red
phase, not a follow-up:
```python
# tests/unit/economics/counter_tendencies/test_types.py:15-18 — BEFORE:
#     from babylon.domain.economics.counter_tendencies.types import (
#         COUNTER_TENDENCY_WEIGHTS,
#         CounterTendencyStrength,
#     )
# AFTER:
from babylon.domain.economics.counter_tendencies.types import (
    CounterTendencyStrength,
    counter_tendency_weights,
)

# :113-119 — BEFORE:
#     def test_weights_sum_to_one(self) -> None:
#         """COUNTER_TENDENCY_WEIGHTS should sum to 1.0."""
#         assert sum(COUNTER_TENDENCY_WEIGHTS) == pytest.approx(1.0)
#
#     def test_weights_length_six(self) -> None:
#         """COUNTER_TENDENCY_WEIGHTS has exactly 6 elements."""
#         assert len(COUNTER_TENDENCY_WEIGHTS) == 6
# AFTER:
    def test_weights_sum_to_one(self) -> None:
        """counter_tendency_weights() should sum to 1.0."""
        assert sum(counter_tendency_weights()) == pytest.approx(1.0)

    def test_weights_length_six(self) -> None:
        """counter_tendency_weights() has exactly 6 elements."""
        assert len(counter_tendency_weights()) == 6

# :152-175 — BEFORE the body used IMPERIAL_RENT_REFERENCE_SCALE and
# COUNTER_TENDENCY_WEIGHTS; AFTER (only the four marked lines change):
    def test_net_counter_tendency_weighted_sum(self) -> None:
        """net_counter_tendency uses counter_tendency_weights() in correct order."""
        from babylon.domain.economics.counter_tendencies.types import (
            imperial_rent_reference_scale,
        )

        ct = CounterTendencyStrength(
            year=2020,
            exploitation_rate_change=0.1,
            wage_suppression=0.01,
            constant_capital_cheapening=-0.03,
            reserve_army_size=0.08,
            imperial_rent_flow=500_000_000_000.0,
            fictitious_profit_share=0.25,
        )
        # Manual calculation of indicators:
        # [0] exploitation_rate_change = 0.1
        # [1] wage_suppression = 0.01
        # [2] -constant_capital_cheapening = -(-0.03) = 0.03
        # [3] reserve_army_size = 0.08
        # [4] imperial_rent = min(500B / reference_scale, 1.0)
        # [5] fictitious_profit_share = 0.25
        imperial_norm = min(500_000_000_000.0 / imperial_rent_reference_scale(), 1.0)
        indicators = [0.1, 0.01, 0.03, 0.08, imperial_norm, 0.25]
        expected = sum(w * v for w, v in zip(indicators, counter_tendency_weights(), strict=True))
        assert ct.net_counter_tendency == pytest.approx(expected)

# :200-211 — BEFORE the body used IMPERIAL_RENT_REFERENCE_SCALE; AFTER:
    def test_imperial_rent_caps_at_reference_scale(self) -> None:
        """Imperial rent normalization caps at 1.0 at the reference scale."""
        from babylon.domain.economics.counter_tendencies.types import (
            imperial_rent_reference_scale,
        )

        ct_at_scale = CounterTendencyStrength(
            year=2020,
            imperial_rent_flow=imperial_rent_reference_scale(),
        )
        ct_above_scale = CounterTendencyStrength(
            year=2020,
            imperial_rent_flow=imperial_rent_reference_scale() * 2.0,
        )
        # Both should produce the same CT (capped at 1.0)
        assert ct_at_scale.net_counter_tendency == pytest.approx(
            ct_above_scale.net_counter_tendency
        )
```
**And the two other already-shipped test files** — same reason, same red phase:
```python
# tests/unit/economics/credit/test_credit_cycle.py:16-21 — BEFORE:
#     from babylon.domain.economics.credit.types import (
#         OVEREXTENSION_DEFAULT_RATE,
#         RECOVERY_CONSECUTIVE_PERIODS,
#         STAGNATION_CREDIT_GROWTH,
#         CreditCyclePhase,
#     )
# AFTER:
from babylon.domain.economics.credit.types import (
    OVEREXTENSION_DEFAULT_RATE,
    RECOVERY_CONSECUTIVE_PERIODS,
    CreditCyclePhase,
    stagnation_credit_growth,
)

# :99  — BEFORE: credit_growth=STAGNATION_CREDIT_GROWTH / 2,  # Below threshold
# AFTER:
            credit_growth=stagnation_credit_growth() / 2,  # Below threshold

# :181 — BEFORE: credit_growth=STAGNATION_CREDIT_GROWTH + 0.01,  # Above threshold
# AFTER:
            credit_growth=stagnation_credit_growth() + 0.01,  # Above threshold

# :192 — BEFORE: credit_growth=STAGNATION_CREDIT_GROWTH / 2,  # Below threshold
# AFTER:
            credit_growth=stagnation_credit_growth() / 2,  # Below threshold


# tests/integration/economics/test_vol3_surplus_distribution_live.py (shipped by
# Task U1.9) — BEFORE:
#     from babylon.domain.economics.distribution.types import DISTRIBUTION_EPSILON
# AFTER:
from babylon.domain.economics.distribution.types import distribution_epsilon

# in test_sc001_identity_holds_for_one_hundred_percent_of_observations — BEFORE:
#         if residual > DISTRIBUTION_EPSILON:
#             violations.append((fips, residual))
#     assert not violations, f"SC-001 violated (residual > {DISTRIBUTION_EPSILON}): {violations}"
# AFTER (hoist one call out of the loop — the accessor is not free).
# NOTE: `observed` and its trailing non-vacuity assertion are U1.9's; carry them
# through this rewrite unchanged — dropping them would let SC-001 pass over an
# empty domain.
    epsilon = distribution_epsilon()
    observed = 0
    violations: list[tuple[str, float]] = []
    for fips, county in sorted(tick_state.county_states.items()):
        d = county.surplus_distribution
        if d is None:
            continue
        observed += 1
        residual = abs(
            d.total_surplus_produced
            - (
                d.profit_of_enterprise
                + d.interest_payments
                + d.ground_rent
                + d.taxes_on_surplus
            )
        )
        if residual > epsilon:
            violations.append((fips, residual))
    assert not violations, f"SC-001 violated (residual > {epsilon}): {violations}"
    # A universal over an empty domain is vacuously true — SC-001 is only
    # proven if the run actually produced distributions to check.
    assert observed > 0, (
        "SC-001 checked zero observations: no county-year carried a "
        "surplus_distribution, so the identity was never exercised"
    )
```
- [ ] **Step 2: Run tests to verify they fail**
Run: `mise run test:q -- tests/unit/config/test_capital_vol3_defines.py tests/unit/economics/distribution/test_distribution_types.py tests/unit/economics/counter_tendencies/test_types.py tests/unit/economics/credit/test_credit_cycle.py`
Expected: FAIL — `ImportError: cannot import name 'CapitalVolumeIIIDefines' from 'babylon.config.defines'`
in the config file; `ImportError: cannot import name 'counter_tendency_weights' from
'babylon.domain.economics.counter_tendencies.types'` and `ImportError: cannot import name
'stagnation_credit_growth' from 'babylon.domain.economics.credit.types'` at collection for the
rewritten sibling files. These are assertion-unreachable collection reds on files whose imports name
symbols Step 3 creates — the same shape the Global Constraints carve out for a task that creates the
name under test.
`tests/integration/economics/test_vol3_surplus_distribution_live.py` is edited in the same red phase
but is not run here (it needs the reference DB); it is covered by Step 4's full-file run below.
- [ ] **Step 3: Write minimal implementation**
```python
# src/babylon/config/defines/capital_vol3.py (create)
"""Volume III financial-claims coefficients (spec 024-capital-volume-iii).

Thresholds and reference scales for the surplus-value distribution, TRPF
counter-tendency, and financial-crisis-assessment layers — extracted from
module-level ``Final`` constants during the 2026-07-18
vol3-money-scissors-design honesty sweep (U2) so ``defines.yaml`` edits
actually take effect (Constitution III.1).
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class CapitalVolumeIIIDefines(BaseModel):
    """Volume III (surplus distribution / credit / counter-tendency) coefficients."""

    model_config = ConfigDict(frozen=True)

    debt_spiral_threshold: float = Field(
        default=0.5,
        ge=0.0,
        le=1.0,
        description=(
            "Accumulated debt / annual surplus ratio triggering the "
            "debt-spiral crisis flag (NBER 2001/2008 corporate "
            "debt-to-earnings recession analysis)."
        ),
    )
    distribution_epsilon: float = Field(
        default=1e-9,
        gt=0.0,
        le=1e-3,
        description=(
            "Floating-point tolerance for the surplus distribution "
            "accounting identity s = p + i + r + t (IEEE 754 double "
            "tolerance)."
        ),
    )
    counter_tendency_weights: list[float] = Field(
        default=[0.20, 0.15, 0.15, 0.15, 0.20, 0.15],
        description=(
            "Weights for the six TRPF counter-tendencies "
            "(exploitation_rate, wage_suppression, capital_cheapening, "
            "reserve_army, imperial_rent, fictitious_profits); must sum "
            "to 1.0."
        ),
    )
    imperial_rent_reference_scale: float = Field(
        default=500_000_000_000.0,
        gt=0.0,
        description=(
            "Reference scale (dollars) normalizing imperial rent flow "
            "into the counter-tendency weight; Cope (2012) annual "
            "Global South-to-North transfer estimate."
        ),
    )
    profit_rate_fallback: float = Field(
        default=0.05,
        ge=0.0,
        le=1.0,
        description=(
            "County profit rate used for the surplus-distribution "
            "interest calculation when the tensor registry has no "
            "profit_rate for this county-year."
        ),
    )
    national_county_count: int = Field(
        default=3300,
        ge=1,
        le=5000,
        description=(
            "Approximate US county count used to scale a single "
            "county's surplus up to a national proxy for the "
            "financialization ratio (FictitiousCapitalStock."
            "ratio_to_real denominator)."
        ),
    )
    default_rate_estimate: float = Field(
        default=0.02,
        ge=0.0,
        le=1.0,
        description=(
            "Loan default-rate estimate feeding credit_fragility. No "
            "FRED charge-off-rate series is wired for this (D4's "
            "fixture list has none) — a documented estimate, not live "
            "data; see spec 2026-07-18 vol3-money-scissors-design "
            "Table 3.6."
        ),
    )
    housing_capitalization_rate_default: float = Field(
        default=0.05,
        ge=0.0,
        le=1.0,
        description=(
            "Fallback interest rate for housing ground-rent "
            "capitalization (construction-time snapshot; "
            "DefaultHousingDecompositionCalculator does not re-read a "
            "live per-tick rate)."
        ),
    )


# src/babylon/config/defines/_assembler.py — add the import after the
# `balkanization` import (line 16), before `consciousness` (alphabetical):
# _assembler.py — insert this ONE line immediately after the existing
# `from babylon.config.defines.balkanization import BalkanizationDefines`:
from babylon.config.defines.capital_vol3 import CapitalVolumeIIIDefines

# _assembler.py:97-129 — add a docstring bullet after the `market:` line (128):
    - market: Price⟷value scissors dynamics (Program 23 Phase-1 shadow, ADR077)
    - capital_vol3: Volume III surplus-distribution / credit / counter-tendency
      thresholds (024-capital-volume-iii; migrated off module-level Finals
      in the 2026-07-18 honesty sweep)
    """

# _assembler.py:193 — add the Field declaration right after `market`:
    market: MarketDefines = Field(default_factory=MarketDefines)
    # Volume III financial-claims thresholds (024-capital-volume-iii;
    # 2026-07-18 honesty sweep, U2)
    capital_vol3: CapitalVolumeIIIDefines = Field(default_factory=CapitalVolumeIIIDefines)

# _assembler.py:320-323 — add to _from_yaml_dict right after `market=...`:
            market=MarketDefines(**data.get("market", {})),
            capital_vol3=CapitalVolumeIIIDefines(**data.get("capital_vol3", {})),
        )


# __init__.py — insert this ONE line immediately after the existing
# `from babylon.config.defines.bifurcation import BifurcationDefines`:
from babylon.config.defines.capital_vol3 import CapitalVolumeIIIDefines

# __init__.py __all__ — insert "CapitalVolumeIIIDefines" alphabetically
# between "BifurcationDefines" and "CarceralDefines":
    "BifurcationDefines",
    "CapitalVolumeIIIDefines",
    "CarceralDefines",


# src/babylon/domain/economics/distribution/types.py:1-11 — add imports and
# DROP `from typing import Final` (NO module-level defines snapshot — see the
# accessor-function note below). Final was used ONLY by the two constants this
# task deletes; leaving the import is an unused-import lint failure under
# `mise run check`. U2.2 added `year_within_modeled_range`; it stays.
from __future__ import annotations

from functools import lru_cache

from pydantic import BaseModel, ConfigDict, Field, computed_field

from babylon.config.defines import GameDefines
from babylon.domain.economics.tensor import year_within_modeled_range

# distribution/types.py — replace the two Final declarations (lines 16-30)
# with defines-backed ACCESSOR FUNCTIONS, not import-time module-level
# snapshots: snapshotting at import time reads defines.yaml from disk on
# every process start (including layer-0.5 sentinel processes) and freezes
# the value before any runtime override can reach it.
@lru_cache(maxsize=1)
def _default_defines() -> GameDefines:
    """Process-cached ``GameDefines.load_default()`` for the accessors below.

    Cached because ``distribution_epsilon()`` is called from the
    ``distribution_complete`` computed field, which is evaluated per county
    per tick; an uncached ``load_default()`` re-parses ``defines.yaml`` from
    disk on every one of those evaluations.

    Cached on FIRST USE, not at import time — which is the whole point of the
    migration. A process that never touches these accessors (layer-0.5
    sentinels, the docs build) never reads the file, and any caller that holds
    a real ``GameDefines`` passes it explicitly and bypasses the cache
    entirely. Tests that need a different default call
    ``_default_defines.cache_clear()``.
    """
    return GameDefines.load_default()


def debt_spiral_threshold(defines: GameDefines | None = None) -> float:
    """Accumulated debt / annual surplus ratio triggering crisis flag.

    Traceability: When cumulative enterprise losses (accumulated debt)
    exceed 50% of a county's annual surplus value, the debt spiral is
    structurally self-reinforcing. Derived from NBER recession analysis
    of corporate debt-to-earnings ratios during 2001 and 2008 recessions.
    GameDefines-backed (``capital_vol3.debt_spiral_threshold``) since the
    2026-07-18 honesty sweep — moddable via defines.yaml.

    Reads ``capital_vol3.debt_spiral_threshold`` from the passed
    ``defines``, or from the process-cached default when omitted.
    """
    resolved = defines if defines is not None else _default_defines()
    return resolved.capital_vol3.debt_spiral_threshold


def distribution_epsilon(defines: GameDefines | None = None) -> float:
    """Floating-point tolerance for surplus distribution accounting identity.

    The identity s = p + i + r + t must hold within this epsilon.
    Standard IEEE 754 double-precision tolerance for financial accounting.
    GameDefines-backed (``capital_vol3.distribution_epsilon``) since the
    2026-07-18 honesty sweep.

    Reads ``capital_vol3.distribution_epsilon`` from the passed
    ``defines``, or from the process-cached default when omitted.
    """
    resolved = defines if defines is not None else _default_defines()
    return resolved.capital_vol3.distribution_epsilon


# distribution/types.py:42 — the class docstring's identity line names the
# deleted constant; BEFORE:
#     Identity: s = p + i + r + t (within DISTRIBUTION_EPSILON)
# AFTER:
    Identity: s = p + i + r + t (within :func:`distribution_epsilon`)

# distribution/types.py:87 — the ONLY internal use of the deleted constant,
# inside the `distribution_complete` computed field; BEFORE:
#     return bool(abs(distributed - self.total_surplus_produced) < DISTRIBUTION_EPSILON)
# AFTER:
        return bool(abs(distributed - self.total_surplus_produced) < distribution_epsilon())


# src/babylon/domain/economics/distribution/__init__.py — the package
# re-exports both deleted names at module level AND lists them in __all__;
# leaving either breaks `import babylon.domain.economics.distribution`.
# :13-17 __all__ — BEFORE:
#     __all__: list[str] = [
#         # Types (types.py)
#         "DEBT_SPIRAL_THRESHOLD",
#         "DISTRIBUTION_EPSILON",
#         "SurplusValueDistribution",
# AFTER:
__all__: list[str] = [
    # Threshold accessors (types.py) — GameDefines-backed since the
    # 2026-07-18 honesty sweep; these are functions, not constants.
    "debt_spiral_threshold",
    "distribution_epsilon",
    "SurplusValueDistribution",

# :39-44 import — BEFORE:
#     from babylon.domain.economics.distribution.types import (
#         DEBT_SPIRAL_THRESHOLD,
#         DISTRIBUTION_EPSILON,
#         DebtAccumulation,
#         SurplusValueDistribution,
#     )
# AFTER:
from babylon.domain.economics.distribution.types import (
    DebtAccumulation,
    SurplusValueDistribution,
    debt_spiral_threshold,
    distribution_epsilon,
)


# src/babylon/domain/economics/counter_tendencies/types.py:1-47 — add
# import (NO module-level defines snapshot — see the accessor-function note
# below); replace the two Final declarations. `Final` STAYS here: it is still
# used by `_IMPERIAL_RENT_EPSILON` at line 46.
from __future__ import annotations

from functools import lru_cache
from typing import Final

from pydantic import BaseModel, ConfigDict, Field, computed_field

from babylon.config.defines import GameDefines

# counter_tendencies/types.py — replace the two Final declarations with
# defines-backed ACCESSOR FUNCTIONS, not import-time module-level
# snapshots: snapshotting at import time reads defines.yaml from disk on
# every process start (including layer-0.5 sentinel processes) and freezes
# the value before any runtime override can reach it.
@lru_cache(maxsize=1)
def _default_defines() -> GameDefines:
    """Process-cached ``GameDefines.load_default()`` for the accessors below.

    Same rationale as ``distribution.types._default_defines``: both accessors
    are read from the ``net_counter_tendency`` computed field, evaluated on
    every model access and every ``model_dump()``. Cached on FIRST USE, not at
    import time; an explicit ``defines`` argument bypasses the cache.
    """
    return GameDefines.load_default()


def counter_tendency_weights(defines: GameDefines | None = None) -> list[float]:
    """Weights for the six TRPF counter-tendencies in net strength computation.

    Order: [exploitation_rate, wage_suppression, capital_cheapening,
            reserve_army, imperial_rent, fictitious_profits]

    Traceability: MLM-TW theory weights imperial rent (0.20) and exploitation
    rate increase (0.20) higher than other counter-tendencies because these
    are the primary mechanisms sustaining core profit rates. The remaining
    four tendencies receive equal weight (0.15 each). Sum = 1.0.
    GameDefines-backed (``capital_vol3.counter_tendency_weights``) since the
    2026-07-18 honesty sweep.

    Reads ``capital_vol3.counter_tendency_weights`` from the passed
    ``defines``, or from the process-cached default when omitted.
    """
    resolved = defines if defines is not None else _default_defines()
    return resolved.capital_vol3.counter_tendency_weights


def imperial_rent_reference_scale(defines: GameDefines | None = None) -> float:
    """Reference scale for imperial rent normalization (dollars).

    Imperial rent flows are normalized to [0, 1] via::

        normalized = min(imperial_rent_flow / imperial_rent_reference_scale(), 1.0)

    Traceability: The default $500B corresponds approximately to the annual
    net value transfer from Global South to Global North estimated by
    Cope (2012), *Divided World Divided Class*. GameDefines-backed
    (``capital_vol3.imperial_rent_reference_scale``) since the 2026-07-18
    honesty sweep.

    Reads ``capital_vol3.imperial_rent_reference_scale`` from the passed
    ``defines``, or from the process-cached default when omitted.
    """
    resolved = defines if defines is not None else _default_defines()
    return resolved.capital_vol3.imperial_rent_reference_scale


# counter_tendencies/types.py:66 — class docstring cross-reference; BEFORE:
#     :data:`COUNTER_TENDENCY_WEIGHTS`. Positive values indicate
# AFTER (the target is a function now — a stale :data: role is a broken xref
# under the Sphinx -W build):
    :func:`counter_tendency_weights`. Positive values indicate

# counter_tendencies/types.py:111-119 — the `net_counter_tendency` docstring
# and its ONE internal use of the reference scale; BEFORE:
#         - [4] imperial_rent_flow: linear normalization against
#           ``IMPERIAL_RENT_REFERENCE_SCALE``, capped at 1.0.
#           ...
#         # Capped at 1.0 at the reference scale. Extensible: adjust
#         # IMPERIAL_RENT_REFERENCE_SCALE to recalibrate.
#         imperial_normalized = min(
#             self.imperial_rent_flow / max(IMPERIAL_RENT_REFERENCE_SCALE, _IMPERIAL_RENT_EPSILON),
#             1.0,
#         )
# AFTER:
        - [4] imperial_rent_flow: linear normalization against
          :func:`imperial_rent_reference_scale`, capped at 1.0.
          The *magnitude* of unequal exchange matters (Marx V3 Ch14 §V).
        - [5] fictitious_profit_share: direct
        """
        # Magnitude-sensitive normalization: larger flows → stronger CT.
        # Capped at 1.0 at the reference scale. Extensible: edit
        # capital_vol3.imperial_rent_reference_scale in defines.yaml to
        # recalibrate — no code change needed since the 2026-07-18 sweep.
        imperial_normalized = min(
            self.imperial_rent_flow
            / max(imperial_rent_reference_scale(), _IMPERIAL_RENT_EPSILON),
            1.0,
        )

# counter_tendencies/types.py:131 — the weighted sum; BEFORE:
#         return sum(w * v for w, v in zip(indicators, COUNTER_TENDENCY_WEIGHTS, strict=True))
# AFTER:
        return sum(
            w * v for w, v in zip(indicators, counter_tendency_weights(), strict=True)
        )


# src/babylon/domain/economics/counter_tendencies/__init__.py — same package
# re-export problem as distribution. :11-29 — BEFORE the module imported
# COUNTER_TENDENCY_WEIGHTS and listed it in __all__; AFTER:
from babylon.domain.economics.counter_tendencies.calculator import (
    CounterTendencyCalculator,
    DefaultCounterTendencyCalculator,
)
from babylon.domain.economics.counter_tendencies.types import (
    CounterTendencyStrength,
    counter_tendency_weights,
    imperial_rent_reference_scale,
)

__all__: list[str] = [
    # Coefficient accessors (GameDefines-backed since the 2026-07-18
    # honesty sweep; these are functions, not constants)
    "counter_tendency_weights",
    "imperial_rent_reference_scale",
    # Types
    "CounterTendencyStrength",
    # Protocols
    "CounterTendencyCalculator",
    # Implementations
    "DefaultCounterTendencyCalculator",
]


# src/babylon/domain/economics/counter_tendencies/calculator.py:11 — module
# docstring cross-reference to a name that no longer exists; BEFORE:
#     :data:`COUNTER_TENDENCY_WEIGHTS`: Weights for the six indicators.
# AFTER:
    :func:`babylon.domain.economics.counter_tendencies.types.counter_tendency_weights`:
    Weights for the six indicators.


# src/babylon/domain/economics/credit/types.py:95 — the fifth constant.
# BEFORE:
#     STAGNATION_CREDIT_GROWTH: Final[float] = GameDefines().crisis.stagnation_credit_growth
#     """Credit expansion rate threshold for stagnation diagnosis. ..."""
# AFTER — an accessor, for exactly the reason the four siblings above became
# accessors. It reads `crisis`, not the new `capital_vol3` category (the value
# already had a define; only the plumbing was broken), so no new field is
# added — but leaving it as an import-time snapshot would enshrine in this
# very task the anti-pattern the task exists to remove.
@lru_cache(maxsize=1)
def _default_defines() -> GameDefines:
    """Process-cached ``GameDefines.load_default()``.

    Same rationale as ``distribution.types._default_defines``: cached on
    FIRST USE rather than at import time, and bypassed entirely when a
    caller passes an explicit ``defines``.
    """
    return GameDefines.load_default()


def stagnation_credit_growth(defines: GameDefines | None = None) -> float:
    """Credit expansion rate threshold for stagnation diagnosis.

    Traceability: FRED TCMDO YoY growth rate. When credit growth falls below
    1% annually, the economy is in secular stagnation — insufficient credit
    creation for expansion but insufficient defaults for crisis clearing.

    Reads ``crisis.stagnation_credit_growth`` from the passed ``defines``, or
    from the process-cached default when omitted. Was a module-level ``Final``
    initialised from a bare ``GameDefines()`` — which read the dataclass
    defaults and ignored ``defines.yaml`` entirely — until the 2026-07-18
    honesty sweep.
    """
    resolved = defines if defines is not None else _default_defines()
    return resolved.crisis.stagnation_credit_growth

# credit/types.py:1-13 — add the `lru_cache` import. `Final` STAYS: seven other
# threshold constants in this module still use it.
from functools import lru_cache


# src/babylon/domain/economics/credit/credit_cycle.py:18-23 — BEFORE the
# module imported STAGNATION_CREDIT_GROWTH; AFTER:
from babylon.domain.economics.credit.types import (
    OVEREXTENSION_DEFAULT_RATE,
    RECOVERY_CONSECUTIVE_PERIODS,
    CreditCyclePhase,
    stagnation_credit_growth,
)

# credit_cycle.py:61-69 — the class docstring names the constant three times;
# BEFORE:
#     - OVEREXTENSION -> STAGNATION: abs(credit_growth) < STAGNATION_CREDIT_GROWTH
#     - CRISIS -> RECOVERY: profit_rate_trend > 0 for RECOVERY_CONSECUTIVE_PERIODS
#     - RECOVERY -> EXPANSION: credit_growth > STAGNATION_CREDIT_GROWTH
#     - RECOVERY -> STAGNATION: abs(credit_growth) < STAGNATION_CREDIT_GROWTH
# AFTER:
    - OVEREXTENSION -> STAGNATION: abs(credit_growth) < stagnation_credit_growth()
    - CRISIS -> RECOVERY: profit_rate_trend > 0 for RECOVERY_CONSECUTIVE_PERIODS
    - RECOVERY -> EXPANSION: credit_growth > stagnation_credit_growth()
    - RECOVERY -> STAGNATION: abs(credit_growth) < stagnation_credit_growth()

# credit_cycle.py:120-128 — BEFORE:
#     def _evaluate_overextension(
#         self, credit_growth: float, default_rate: float
#     ) -> tuple[CreditCyclePhase, int]:
#         """OVEREXTENSION -> CRISIS (high defaults) or STAGNATION (low growth)."""
#         if default_rate > OVEREXTENSION_DEFAULT_RATE:
#             return (CreditCyclePhase.CRISIS, 0)
#         if abs(credit_growth) < STAGNATION_CREDIT_GROWTH:
#             return (CreditCyclePhase.STAGNATION, 0)
#         return (CreditCyclePhase.OVEREXTENSION, 0)
# AFTER:
    def _evaluate_overextension(
        self, credit_growth: float, default_rate: float
    ) -> tuple[CreditCyclePhase, int]:
        """OVEREXTENSION -> CRISIS (high defaults) or STAGNATION (low growth)."""
        if default_rate > OVEREXTENSION_DEFAULT_RATE:
            return (CreditCyclePhase.CRISIS, 0)
        if abs(credit_growth) < stagnation_credit_growth():
            return (CreditCyclePhase.STAGNATION, 0)
        return (CreditCyclePhase.OVEREXTENSION, 0)

# credit_cycle.py:141-147 — BEFORE:
#     def _evaluate_recovery(self, credit_growth: float) -> tuple[CreditCyclePhase, int]:
#         """RECOVERY -> EXPANSION (credit resumes) or STAGNATION (stalls)."""
#         if credit_growth > STAGNATION_CREDIT_GROWTH:
#             return (CreditCyclePhase.EXPANSION, 0)
#         if abs(credit_growth) < STAGNATION_CREDIT_GROWTH:
#             return (CreditCyclePhase.STAGNATION, 0)
#         return (CreditCyclePhase.RECOVERY, 0)
# AFTER (one accessor call, not two — the two comparisons must see the same
# value even if defines.yaml is swapped mid-process):
    def _evaluate_recovery(self, credit_growth: float) -> tuple[CreditCyclePhase, int]:
        """RECOVERY -> EXPANSION (credit resumes) or STAGNATION (stalls)."""
        threshold = stagnation_credit_growth()
        if credit_growth > threshold:
            return (CreditCyclePhase.EXPANSION, 0)
        if abs(credit_growth) < threshold:
            return (CreditCyclePhase.STAGNATION, 0)
        return (CreditCyclePhase.RECOVERY, 0)


# src/babylon/domain/economics/credit/__init__.py:37-69 — the package
# re-exports the deleted name. In the `from ...credit.types import (` block,
# BEFORE: `    STAGNATION_CREDIT_GROWTH,` ; AFTER — delete that line and add
# `    stagnation_credit_growth,` after `InterestRateState,` (the block sorts
# ALL_CAPS, then CamelCase, then lowercase). In __all__, BEFORE:
#     "CREDIT_FRAGILITY_THRESHOLD",
#     "STAGNATION_CREDIT_GROWTH",
#     "OVEREXTENSION_DEFAULT_RATE",
# AFTER:
    "CREDIT_FRAGILITY_THRESHOLD",
    "stagnation_credit_growth",
    "OVEREXTENSION_DEFAULT_RATE",


# tests/unit/economics/distribution/test_calculator.py:59 — docstring
# reference to the deleted name (no code use); BEFORE:
#         """s = p + i + r + t holds within DISTRIBUTION_EPSILON."""
# AFTER:
        """s = p + i + r + t holds within distribution_epsilon()."""
```
Also regenerate the canonical YAML:
```bash
poetry run python tools/generate_defines_config.py
```
- [ ] **Step 4: Run tests to verify they pass**
Run: `mise run test:q -- tests/unit/config/test_capital_vol3_defines.py tests/unit/economics/distribution/test_distribution_types.py tests/unit/economics/counter_tendencies/test_types.py tests/unit/economics/credit/test_credit_cycle.py tests/unit/economics/distribution/test_calculator.py tests/unit/config/test_constants_sync.py::TestDefinesYamlSingleSourceOfTruth`
Expected: PASS — 5 in `test_capital_vol3_defines.py` (2 defaults + 3 stagnation), 19 in
`test_distribution_types.py` (16 pre-existing + 3 appended), 21 in `test_types.py` (18 pre-existing,
4 of them rewritten onto the accessors, + 3 appended), 17 in `test_credit_cycle.py` (all
pre-existing, import + 3 call sites rewritten), plus `test_calculator.py` and the sync guard
unchanged.
- [ ] **Step 4b: Re-run U1.9's integration file — this task edited it**
Run: `mise run test:q -- tests/integration/economics/test_vol3_surplus_distribution_live.py`
Expected: PASS (3 passed) — the same 3 U1.9 turned green, now importing `distribution_epsilon`
instead of the deleted `DISTRIBUTION_EPSILON`. If this errors at collection, the import rewrite in
Step 1 was not applied; a green Step 4 does not cover it, because that file is in the integration
tier and `test:unit` never collects it.
- [ ] **Step 4c: Prove no deleted name survives anywhere**
Run: `rg -n "DEBT_SPIRAL_THRESHOLD|DISTRIBUTION_EPSILON|COUNTER_TENDENCY_WEIGHTS|IMPERIAL_RENT_REFERENCE_SCALE|STAGNATION_CREDIT_GROWTH" src tests`
Expected: **exactly one match** —
`tests/unit/config/test_capital_vol3_defines.py`'s `assert "STAGNATION_CREDIT_GROWTH" not in source`,
which is the string literal that pins the deletion and must survive. Any *other* hit is an
unthreaded call site — an `ImportError` waiting for whichever suite collects that file next. Then
run `mise run check:quick` to confirm the deleted `from typing import Final` in
`distribution/types.py` left no unused-import lint failure and that the two `:func:` docstring
xrefs resolve.
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(config): add GameDefines.capital_vol3, migrate five Vol III constants to accessors

DEBT_SPIRAL_THRESHOLD, DISTRIBUTION_EPSILON, COUNTER_TENDENCY_WEIGHTS,
and IMPERIAL_RENT_REFERENCE_SCALE were module-level Finals a
defines.yaml edit could never reach. Migrate them into a new
GameDefines.capital_vol3 category (same default values, zero behavior
change) and regenerate defines.yaml. Each becomes a lowercase accessor
function reading the defines at CALL time, with a first-use-cached
process default; the ALL-CAPS names are deleted and every consumer is
threaded — both package __init__.py re-exports, distribution/types.py's
own distribution_complete field, counter_tendencies/types.py's
net_counter_tendency, credit_cycle.py's two transition guards, and four
already-shipped test files including U1.9's integration acceptance test.

Also convert credit/types.py:95's STAGNATION_CREDIT_GROWTH, which read
bare GameDefines() instead of load_default() — a defines.yaml override
silently had no effect. It reads crisis, not capital_vol3, so no new
field is added; leaving it as an import-time snapshot would have
enshrined in this commit the exact anti-pattern the commit removes.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.4: Move the tick/system + factory magic numbers into `capital_vol3`

**Files:**
- Modify: `src/babylon/domain/economics/tick/system/__init__.py:1451,1511,1520`
- Modify: `src/babylon/domain/economics/factory.py:335-337,459-462`
- Modify: `tests/unit/economics/tick/conftest.py:36-55` (`MockTensor` gains `total_s`)
- Test: `tests/unit/economics/tick/test_financial_integration.py` (append)
- Test: `tests/unit/economics/test_create_financial_services.py` (create)

**Interfaces:**
- Consumes: `services.defines.capital_vol3.{profit_rate_fallback,national_county_count,default_rate_estimate,housing_capitalization_rate_default}` (from U2.3); `GameDefines.load_default()`; `ServiceContainer.create(defines=..., **overrides)` (`babylon.engine.services`).
- Produces: `create_financial_services(fred_series_cache=None, *, defines: GameDefines | None = None)` — U1's `_legacy.py` call site (`create_financial_services(fred_series_cache=fred_cache)`) is unaffected (new param is optional/keyword).

- [ ] **Step 1: Write the failing tests**
```python
# --- tests/unit/economics/tick/conftest.py:36-55 — extend MockTensor ---
class MockTensor:
    """Mock tensor with configurable profit_rate and total_s attributes."""

    def __init__(self, profit_rate: float | None = None, total_s: float | None = None) -> None:
        self.profit_rate = profit_rate
        self.total_s = total_s


# --- tests/unit/economics/tick/test_financial_integration.py (append) ---
from babylon.config.defines import CapitalVolumeIIIDefines, GameDefines
from babylon.domain.economics.financial_crisis.types import FinancialCrisisAssessment
from babylon.domain.economics.tick.system import TickDynamicsSystem
from babylon.engine.services import ServiceContainer
from tests.unit.economics.tick.conftest import MockTensor, MockTensorRegistry

WAYNE_FIPS = "26163"


class _SpyDistributionCalculator:
    """Captures compute_distribution kwargs; always succeeds."""

    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    def compute_distribution(
        self,
        fips: str,
        year: int,
        total_surplus: float,
        county_profit_rate: float,
        national_interest_rate: float,
        county_employment: float = 0.0,
    ) -> SurplusValueDistribution:
        self.calls.append({"county_profit_rate": county_profit_rate})
        return SurplusValueDistribution(
            fips_code=fips,
            year=year,
            total_surplus_produced=total_surplus,
            interest_payments=0.0,
            ground_rent=0.0,
            taxes_on_surplus=0.0,
        )


class _SpyFinancialCrisisAssessor:
    """Captures assess() kwargs; returns a minimal valid assessment."""

    def __init__(self) -> None:
        self.calls: list[dict[str, object]] = []

    def assess(
        self,
        fips: str,
        year: int,
        interest_burden_ratio: float,
        financialization_ratio: float,
        default_rate: float,
        credit_spread: float,
        claims_exceed_surplus: bool,
    ) -> FinancialCrisisAssessment:
        self.calls.append(
            {
                "financialization_ratio": financialization_ratio,
                "default_rate": default_rate,
                "credit_spread": credit_spread,
            }
        )
        return FinancialCrisisAssessment(fips_code=fips, year=year)


class TestCapitalVol3DefinesWiredIntoFinancialLayer:
    """Honesty sweep (U2): magic numbers in the financial layer now read
    services.defines.capital_vol3.* instead of bare literals."""

    def test_profit_rate_fallback_reads_capital_vol3(self) -> None:
        system = TickDynamicsSystem()
        spy = _SpyDistributionCalculator()
        registry = MockTensorRegistry({(WAYNE_FIPS, 2020): MockTensor(profit_rate=None, total_s=1000.0)})
        defines = GameDefines(capital_vol3=CapitalVolumeIIIDefines(profit_rate_fallback=0.09))
        services = ServiceContainer.create(
            distribution_calculator=spy,
            tensor_registry=registry,
            defines=defines,
        )
        county = _make_county_state()

        system._compute_county_financial_state(
            fips=WAYNE_FIPS,
            county=county,
            services=services,
            year=2020,
            national_rate=0.05,
            fictitious=None,
        )

        assert spy.calls[0]["county_profit_rate"] == pytest.approx(0.09)

    def test_national_county_count_reads_capital_vol3(self) -> None:
        system = TickDynamicsSystem()
        spy = _SpyFinancialCrisisAssessor()
        registry = MockTensorRegistry({(WAYNE_FIPS, 2020): MockTensor(profit_rate=0.05, total_s=1000.0)})
        defines = GameDefines(capital_vol3=CapitalVolumeIIIDefines(national_county_count=10))
        services = ServiceContainer.create(
            financial_crisis_assessor=spy,
            tensor_registry=registry,
            defines=defines,
        )
        fictitious = FictitiousCapitalStock(
            year=2020,
            government_debt=1_000_000.0,
            corporate_equity=1_000_000.0,
            corporate_debt=1_000_000.0,
            household_debt=1_000_000.0,
        )
        updates: dict[str, object] = {
            "surplus_distribution": SurplusValueDistribution(
                fips_code=WAYNE_FIPS,
                year=2020,
                total_surplus_produced=1000.0,
                interest_payments=100.0,
                ground_rent=50.0,
                taxes_on_surplus=25.0,
            )
        }

        system._assess_county_financial_crisis(
            fips=WAYNE_FIPS,
            year=2020,
            updates=updates,
            services=services,
            national_rate=0.05,
            fictitious=fictitious,
            total_surplus=1000.0,
        )

        expected_ratio = fictitious.ratio_to_real(1000.0 * 10)
        assert spy.calls[0]["financialization_ratio"] == pytest.approx(expected_ratio)

    def test_default_rate_estimate_reads_capital_vol3(self) -> None:
        system = TickDynamicsSystem()
        spy = _SpyFinancialCrisisAssessor()
        defines = GameDefines(capital_vol3=CapitalVolumeIIIDefines(default_rate_estimate=0.07))
        services = ServiceContainer.create(financial_crisis_assessor=spy, defines=defines)
        updates: dict[str, object] = {
            "surplus_distribution": SurplusValueDistribution(
                fips_code=WAYNE_FIPS,
                year=2020,
                total_surplus_produced=1000.0,
                interest_payments=100.0,
                ground_rent=50.0,
                taxes_on_surplus=25.0,
            )
        }

        system._assess_county_financial_crisis(
            fips=WAYNE_FIPS,
            year=2020,
            updates=updates,
            services=services,
            national_rate=0.05,
            fictitious=None,
            total_surplus=1000.0,
        )

        assert spy.calls[0]["default_rate"] == pytest.approx(0.07)


# --- tests/unit/economics/test_create_financial_services.py (create) ---
"""Unit tests for create_financial_services (honesty sweep, U2).

Feature: 024-capital-volume-iii
"""

from __future__ import annotations

from babylon.config.defines import CapitalVolumeIIIDefines, GameDefines
from babylon.domain.economics.factory import create_financial_services


class TestCreateFinancialServicesHousingRate:
    def test_default_housing_capitalization_rate_is_capital_vol3_default(self) -> None:
        overrides = create_financial_services()
        housing_calc = overrides["housing_calculator"]
        assert housing_calc._interest_rate == GameDefines().capital_vol3.housing_capitalization_rate_default

    def test_custom_defines_changes_housing_capitalization_rate(self) -> None:
        defines = GameDefines(
            capital_vol3=CapitalVolumeIIIDefines(housing_capitalization_rate_default=0.12)
        )
        overrides = create_financial_services(defines=defines)
        housing_calc = overrides["housing_calculator"]
        assert housing_calc._interest_rate == 0.12
```
- [ ] **Step 2: Run tests to verify they fail**
Run: `mise run test:q -- tests/unit/economics/tick/test_financial_integration.py::TestCapitalVol3DefinesWiredIntoFinancialLayer tests/unit/economics/test_create_financial_services.py`
Expected: FAIL — `test_profit_rate_fallback_reads_capital_vol3` asserts `0.09` but gets `0.05` (still hardcoded); `test_national_county_count_reads_capital_vol3` gets the `3300`-scaled ratio, not the `10`-scaled one; `test_custom_defines_changes_housing_capitalization_rate` fails with `TypeError: create_financial_services() got an unexpected keyword argument 'defines'`.
- [ ] **Step 3: Write minimal implementation**
```python
# src/babylon/domain/economics/tick/system/__init__.py:1451 — BEFORE:
#                     county_profit_rate=profit_rate if profit_rate is not None else 0.05,
# AFTER:
                    county_profit_rate=(
                        profit_rate
                        if profit_rate is not None
                        else services.defines.capital_vol3.profit_rate_fallback
                    ),

# src/babylon/domain/economics/tick/system/__init__.py:1511 (inside
# _assess_county_financial_crisis) — BEFORE:
#         max_counties = 3300
# AFTER:
        max_counties = services.defines.capital_vol3.national_county_count

# src/babylon/domain/economics/tick/system/__init__.py:1520 — BEFORE:
#             default_rate=0.02,  # Placeholder
# AFTER:
            default_rate=services.defines.capital_vol3.default_rate_estimate,


# src/babylon/domain/economics/factory.py:335-337 — add a keyword-only
# `defines` param (optional, non-breaking — the sole caller in
# _legacy.py:289 uses only `fred_series_cache=`):
def create_financial_services(
    fred_series_cache: dict[str, dict[int, float]] | None = None,
    *,
    defines: GameDefines | None = None,
) -> dict[str, Any]:
    """Create all Volume III financial calculators wired with real data sources.

    Feature: 024-capital-volume-iii

    Builds the interest, credit-cycle, fictitious-capital, distribution,
    rent, housing, counter-tendency and financial-crisis calculators from
    FRED/Z.1-backed adapters. Keep every existing line of this docstring;
    the ONLY change is the new ``defines:`` entry in ``Args:`` below.

    Args:
        fred_series_cache: Optional pre-loaded FRED series data as
            {series_id: {year: value}}. If None, uses hardcoded defaults.
        defines: Optional GameDefines override; defaults to
            GameDefines.load_default() when omitted.

    Returns:
        Dict with keys matching ServiceContainer financial field names.
    """
    from babylon.config.defines import GameDefines
    from babylon.domain.economics.counter_tendencies.calculator import (
        DefaultCounterTendencyCalculator,
    )
    # Keep every existing local import in this function unchanged; the ONLY
    # addition is the GameDefines import on the line above.

    resolved_defines = defines if defines is not None else GameDefines.load_default()

    # Insert the resolved_defines line above directly after the last local
    # import. Every statement from there to the rent_calc line below is
    # unchanged — do not retype it.

# src/babylon/domain/economics/factory.py:459-462 — BEFORE:
#     rent_calc = DefaultRentCalculator(_DefaultCountyRentalAdapter())
#     # Default 5% interest rate for rent capitalization; overridden per-tick
#     _default_interest = 0.05
#     housing_calc = DefaultHousingDecompositionCalculator(housing, _default_interest)
# AFTER:
    rent_calc = DefaultRentCalculator(_DefaultCountyRentalAdapter())
    # Fallback rent-capitalization interest rate — a construction-time
    # snapshot DefaultHousingDecompositionCalculator captures once and
    # never reassigns. (Honesty sweep, U2: the prior comment claiming a
    # per-tick override was stale — no such override exists; see spec
    # 2026-07-18 vol3-money-scissors-design Table 3.6.)
    housing_calc = DefaultHousingDecompositionCalculator(
        housing, resolved_defines.capital_vol3.housing_capitalization_rate_default
    )
```
- [ ] **Step 4: Run tests to verify they pass**
Run: `mise run test:q -- tests/unit/economics/tick/test_financial_integration.py::TestCapitalVol3DefinesWiredIntoFinancialLayer tests/unit/economics/test_create_financial_services.py tests/unit/economics/tick/test_system.py`
Expected: PASS (including all pre-existing `test_system.py` tests, unaffected since default `capital_vol3` values equal the old literals)

Also run the honesty grep and confirm it returns nothing:
```bash
rg -n '=\s*(0\.0[0-9]|3300)\b' src/babylon/domain/economics/tick/system/__init__.py src/babylon/domain/economics/factory.py
```
Expected: no output. Any hit is a surviving inline coefficient (III.1).
- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(economics): move financial-layer magic numbers into GameDefines.capital_vol3

profit_rate fallback (0.05), the national county-count scaling fudge
(3300), the default_rate credit-fragility estimate (0.02), and the
housing ground-rent capitalization rate (0.05) were bare literals in
tick/system/__init__.py and factory.py. Same values, now moddable via
defines.yaml (Constitution III.1). Also removes factory.py's stale
'overridden per-tick' comment — DefaultHousingDecompositionCalculator
captures the rate once at construction and never reassigns it.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.5: `credit_spread` must be the BAA spread, not the effective borrowing rate

> ## ✅ COMPLETE BY SUPERSESSION — DO NOT EXECUTE THE BRIEF BELOW
>
> **Closed by commit `aedce819`** ("fix(economics): close five U2.3 review findings — inert knobs,
> fabricated zeros, unreachable predicate"), a review-fix pass carried out during U2.3. That commit
> fixed the same `credit_spread`/effective-rate conflation this task describes (it is U2.3 **review
> finding 5**), and fixed it *better* than the brief below — it also carries Constitution III.11
> honest-absence typing that the brief does not.
>
> **The interface actually live at HEAD:**
>
> ```python
>     def _compute_national_financial_state(
>         self,
>         services: ServicesProtocol,
>         year: int,
>     ) -> tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]:
> ```
>
> A **4-tuple** `(national_rate, national_spread, fictitious, interest_unavailable_reason)`:
>
> - `national_rate` — `InterestRateState.effective_rate` (`base_rate + baa_spread`), or `None` when
>   the interest calculator returned a `NoDataSentinel` for `year`. **Never a fabricated `0.0`** — a
>   fabricated zero would flow silently into `compute_distribution`'s
>   `national_interest_rate * implied_capital` term and publish a genuine-looking zero-interest
>   distribution (review finding U2.2-3).
> - `national_spread` — the BAA10Y risk premium **alone**, `None` on the same absence. This is what
>   reaches the crisis assessor's `credit_spread` parameter; that is exactly the fix U2.5 was
>   written to deliver.
> - `fictitious` — `FictitiousCapitalStock | None`.
> - `interest_unavailable_reason` — the interest sentinel's own `.reason`, or `None` when the
>   calculator succeeded, so a caller that must skip work can attribute *why*.
>
> The sibling helpers were threaded accordingly, with **`national_spread`** as the parameter name
> (not `baa_spread`) and with `national_rate` **retained** on `_assess_county_financial_crisis`:
>
> ```python
>     def _compute_county_financial_state(
>         self,
>         fips: str,
>         county: CountyEconomicState,
>         services: ServicesProtocol,
>         year: int,
>         national_rate: float | None,
>         fictitious: FictitiousCapitalStock | None,
>         interest_unavailable_reason: str | None = None,
>         national_spread: float | None = None,
>     ) -> CountyEconomicState: ...
>
>     def _assess_county_financial_crisis(
>         self,
>         fips: str,
>         year: int,
>         updates: dict[str, object],
>         services: ServicesProtocol,
>         national_rate: float,  # noqa: ARG002 - kept for call-site symmetry
>         fictitious: FictitiousCapitalStock | None,
>         total_surplus: float | None,
>         national_spread: float | None = None,
>     ) -> object | None: ...
> ```
>
> **Verified at HEAD:** the signatures above are read from the live source; `mypy src` is clean
> across 638 files; `tests/unit/economics/tick/test_financial_integration.py` passes 21/21 —
> including `TestCrisisAssessorReceivesHonestAbsence::test_credit_spread_receives_the_spread_not_the_effective_rate`,
> which pins exactly what Step 1 below was meant to deliver, plus
> `test_absent_fictitious_capital_passes_none_not_zero` and `test_absent_spread_passes_none`.
>
> **Applying the brief below would be a regression.** It renames `national_spread` → `baa_spread`,
> re-types the returns as non-Optional `float`, restores `else 0.0` fabricated zeros, restores
> `fin_ratio = 0.0`, and strips `national_rate` from `_assess_county_financial_crisis` — reverting
> honest-absence handling and breaking three passing tests. **Do not re-run this task.** It is
> retained verbatim below only as the historical record of what was planned; nothing in it is
> pending work.
>
> **Downstream:** U3.2 rewrites this same method and has been corrected to build on the live
> 4-tuple. U3.4 inserts a helper before it and consumes only its `interest_state` local, so it is
> unaffected by the arity.
>
> **Further superseded by U9:** the 4-tuple `(national_rate, national_spread,
> fictitious, interest_unavailable_reason)` becomes the 3-tuple
> `(national_rate: float, national_spread: float, fictitious | None)`; `national_rate`
> is `InterestRateState.effective_rate` **no longer** — it is the endogenous
> `EndogenousInterestRate.rate`; `national_spread` is the endogenous fragility
> premium, not BAA10Y; `interest_unavailable_reason` is removed.

**Files:** *(historical — superseded, do not apply)*
- Modify: `src/babylon/domain/economics/tick/system/__init__.py:1366,1375-1412,1414-1422,1480-1524`
- Test: `tests/unit/economics/tick/test_financial_integration.py` (append)
- Modify: `tests/unit/economics/tick/test_financial_integration.py` (U2.4's three `TestCapitalVol3DefinesWiredIntoFinancialLayer` call sites — signature change)

**Interfaces:** *(historical — superseded, do not apply)*
- Consumes: `InterestRateState.baa_spread` (`babylon.domain.economics.credit.types`); the `_SpyFinancialCrisisAssessor` from U2.4 (reused, already present).
- ~~Produces: `TickDynamicsSystem._compute_national_financial_state(...) -> tuple[float, float, FictitiousCapitalStock | None]` (national_rate, baa_spread, fictitious) — a signature later units must match if they extend this method; `_compute_county_financial_state`/`_assess_county_financial_crisis` gain a `baa_spread: float` parameter.~~
  **SUPERSEDED.** The signature later units must match is the one live at HEAD:
  `_compute_national_financial_state(services, year) -> tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]`,
  with `national_spread` (not `baa_spread`) threaded as a keyword through
  `_compute_county_financial_state` and `_assess_county_financial_crisis`, and `national_rate`
  retained on both. Downstream tasks read *that* contract, not the struck-through line above.

> Every step below is the historical record of superseded work. **Do not execute any of them.**

- [ ] ~~**Step 1: Write the failing test**~~ *(superseded — the equivalent tests already exist and pass as `TestCrisisAssessorReceivesHonestAbsence` in `tests/unit/economics/tick/test_financial_integration.py`)*
```python
# --- tests/unit/economics/tick/test_financial_integration.py (append) ---
class TestCreditSpreadUsesBaaSpreadNotEffectiveRate:
    """Correctness fix (U2, Row A): financial_crisis/assessment.py:43
    documents credit_spread as a RISK PREMIUM (BAA-AAA / BAA10Y), not the
    effective borrowing rate (base_rate + baa_spread) — the two were
    conflated, inflating credit_fragility against a threshold calibrated
    for a pure spread."""

    def test_assess_county_financial_crisis_passes_baa_spread(self) -> None:
        system = TickDynamicsSystem()
        spy = _SpyFinancialCrisisAssessor()
        services = ServiceContainer.create(financial_crisis_assessor=spy)
        updates: dict[str, object] = {
            "surplus_distribution": SurplusValueDistribution(
                fips_code=WAYNE_FIPS,
                year=2020,
                total_surplus_produced=1000.0,
                interest_payments=100.0,
                ground_rent=50.0,
                taxes_on_surplus=25.0,
            )
        }

        system._assess_county_financial_crisis(
            fips=WAYNE_FIPS,
            year=2020,
            updates=updates,
            services=services,
            baa_spread=0.0234,
            fictitious=None,
            total_surplus=1000.0,
        )

        # 0.0234 is the BAA spread, NOT an effective rate like 0.09
        assert spy.calls[0]["credit_spread"] == pytest.approx(0.0234)

    def test_compute_national_financial_state_returns_baa_spread(self) -> None:
        system = TickDynamicsSystem()
        rate_source = MockInterestRateSource(data={2020: (0.0036, 0.0089, 0.0234)})
        interest_calc = DefaultInterestCalculator(rate_source=rate_source)
        services = ServiceContainer.create(interest_calculator=interest_calc)

        # U3.2 adds the `graph` parameter; pass build_territory_graph() once U3 lands.
        national_rate, baa_spread, fictitious = system._compute_national_financial_state(
            services, 2020
        )

        assert national_rate == pytest.approx(0.0036 + 0.0234)  # effective_rate
        assert baa_spread == pytest.approx(0.0234)  # the spread alone
        assert fictitious is None
```
(Add the corresponding imports at the top of `test_financial_integration.py`: `from babylon.domain.economics.credit.interest import DefaultInterestCalculator` and `from tests.unit.economics.credit.conftest import MockInterestRateSource`.)
- [ ] ~~**Step 2: Run test to verify it fails**~~ *(superseded — this red is unreachable at HEAD: `_assess_county_financial_crisis` still rejects `baa_spread=`, but the method returns a 4-tuple, so `test_compute_national_financial_state_returns_baa_spread` would now fail with `ValueError: too many values to unpack (expected 3)`, not the "not enough values" below)*
Run: `mise run test:q -- tests/unit/economics/tick/test_financial_integration.py::TestCreditSpreadUsesBaaSpreadNotEffectiveRate`
Expected: FAIL — `test_assess_county_financial_crisis_passes_baa_spread` raises `TypeError: _assess_county_financial_crisis() got an unexpected keyword argument 'baa_spread'`; `test_compute_national_financial_state_returns_baa_spread` fails unpacking a 2-tuple into 3 names (`ValueError: not enough values to unpack`).
- [ ] ~~**Step 3: Write minimal implementation**~~ *(superseded — **pasting this block reverts `aedce819`**: it re-types the returns as non-Optional `float`, restores `else 0.0`, restores `fin_ratio = 0.0`, and drops `interest_unavailable_reason` entirely)*
```python
# src/babylon/domain/economics/tick/system/__init__.py:1366 — BEFORE:
#         national_rate, fictitious = self._compute_national_financial_state(services, year)
# AFTER:
        # NOTE: U3.2 adds the `graph` third positional argument to this method.
        # If U3 has already landed, this call reads:
        #     self._compute_national_financial_state(services, year, graph)
        national_rate, baa_spread, fictitious = self._compute_national_financial_state(services, year)

# src/babylon/domain/economics/tick/system/__init__.py:1375-1382 — BEFORE:
#             updated[fips] = self._compute_county_financial_state(
#                 fips,
#                 county,
#                 services,
#                 year,
#                 national_rate,
#                 fictitious,
#             )
# AFTER:
            updated[fips] = self._compute_county_financial_state(
                fips,
                county,
                services,
                year,
                national_rate,
                baa_spread,
                fictitious,
            )

# src/babylon/domain/economics/tick/system/__init__.py:1391-1412 — BEFORE:
#     def _compute_national_financial_state(
#         self,
#         services: ServicesProtocol,
#         year: int,
#     ) -> tuple[float, FictitiousCapitalStock | None]:
#         """Compute national-level financial parameters once per tick.
#
#         Returns:
#             Tuple of (national_interest_rate, fictitious_capital_or_none).
#         """
#         interest_state = services.interest_calculator.compute_interest_rate_state(year)
#         if isinstance(interest_state, NoDataSentinel):
#             interest_state = None
#         national_rate = interest_state.effective_rate if interest_state is not None else 0.0
#
#         fictitious = None
#         ...
#         return national_rate, fictitious
# AFTER:
    def _compute_national_financial_state(
        self,
        services: ServicesProtocol,
        year: int,
    ) -> tuple[float, float, FictitiousCapitalStock | None]:
        """Compute national-level financial parameters once per tick.

        Returns:
            Tuple of (national_interest_rate, national_baa_spread,
            fictitious_capital_or_none).
        """
        interest_state = services.interest_calculator.compute_interest_rate_state(year)
        if isinstance(interest_state, NoDataSentinel):
            interest_state = None
        national_rate = interest_state.effective_rate if interest_state is not None else 0.0
        baa_spread = interest_state.baa_spread if interest_state is not None else 0.0

        fictitious = None
        if services.fictitious_capital_calculator is not None:
            fict_result = services.fictitious_capital_calculator.compute_fictitious_capital(year)
            if not isinstance(fict_result, NoDataSentinel):
                fictitious = fict_result

        return national_rate, baa_spread, fictitious

# src/babylon/domain/economics/tick/system/__init__.py:1414-1422 — BEFORE:
#     def _compute_county_financial_state(
#         self,
#         fips: str,
#         county: CountyEconomicState,
#         services: ServicesProtocol,
#         year: int,
#         national_rate: float,
#         fictitious: FictitiousCapitalStock | None,
#     ) -> CountyEconomicState:
# AFTER:
    def _compute_county_financial_state(
        self,
        fips: str,
        county: CountyEconomicState,
        services: ServicesProtocol,
        year: int,
        national_rate: float,
        baa_spread: float,
        fictitious: FictitiousCapitalStock | None,
    ) -> CountyEconomicState:
        """Compute financial fields for a single county.

        Args:
            fips: County FIPS code.
            county: Current county state.
            services: ServicesProtocol with financial calculators.
            year: Current simulation year.
            national_rate: National effective interest rate.
            baa_spread: National Baa corporate bond spread (risk premium,
                distinct from national_rate — see credit_spread's use in
                _assess_county_financial_crisis).
            fictitious: FictitiousCapitalStock or None.

        Returns:
            Updated CountyEconomicState with financial fields.
        """
        # Only the signature and docstring above change. Leave the entire
        # existing method body byte-for-byte as-is — do not retype it. The
        # single body edit is the _assess_county_financial_crisis call site,
        # covered by the next block in this step.

# src/babylon/domain/economics/tick/system/__init__.py:1480-1488 — BEFORE:
#         if services.financial_crisis_assessor is not None and "surplus_distribution" in updates:
#             assessment = self._assess_county_financial_crisis(
#                 fips,
#                 year,
#                 updates,
#                 services,
#                 national_rate,
#                 fictitious,
#                 total_surplus,
#             )
# AFTER:
        if services.financial_crisis_assessor is not None and "surplus_distribution" in updates:
            assessment = self._assess_county_financial_crisis(
                fips,
                year,
                updates,
                services,
                baa_spread,
                fictitious,
                total_surplus,
            )

# src/babylon/domain/economics/tick/system/__init__.py:1496-1524 — BEFORE:
#     def _assess_county_financial_crisis(
#         self,
#         fips: str,
#         year: int,
#         updates: dict[str, object],
#         services: ServicesProtocol,
#         national_rate: float,
#         fictitious: FictitiousCapitalStock | None,
#         total_surplus: float | None,
#     ) -> object | None:
#         """Assess financial crisis for a single county."""
#         ...
#         result: object | None = services.financial_crisis_assessor.assess(
#             fips=fips,
#             year=year,
#             interest_burden_ratio=float(fin_share),
#             financialization_ratio=fin_ratio,
#             default_rate=services.defines.capital_vol3.default_rate_estimate,
#             credit_spread=national_rate,
#             claims_exceed_surplus=bool(claims_exceed),
#         )
#         return result
# AFTER:
    def _assess_county_financial_crisis(
        self,
        fips: str,
        year: int,
        updates: dict[str, object],
        services: ServicesProtocol,
        baa_spread: float,
        fictitious: FictitiousCapitalStock | None,
        total_surplus: float | None,
    ) -> object | None:
        """Assess financial crisis for a single county.

        Args:
            baa_spread: National Baa corporate bond spread — the RISK
                PREMIUM `financial_crisis/assessment.py:43` documents
                credit_spread as (BAA-AAA / BAA10Y), NOT the effective
                borrowing rate. Fixed U2 (honesty sweep, Row A): this
                previously received `national_rate` (base_rate +
                baa_spread), inflating credit_fragility against a
                threshold calibrated for a pure spread.
        """
        surplus_dist = updates["surplus_distribution"]
        fin_share = getattr(surplus_dist, "financialization_share", 0.0)
        claims_exceed = getattr(surplus_dist, "claims_exceed_surplus", False)
        fin_ratio = 0.0
        max_counties = services.defines.capital_vol3.national_county_count
        if fictitious is not None and total_surplus is not None and total_surplus > 0:
            fin_ratio = fictitious.ratio_to_real(total_surplus * max_counties)

        result: object | None = services.financial_crisis_assessor.assess(
            fips=fips,
            year=year,
            interest_burden_ratio=float(fin_share),
            financialization_ratio=fin_ratio,
            default_rate=services.defines.capital_vol3.default_rate_estimate,
            credit_spread=baa_spread,
            claims_exceed_surplus=bool(claims_exceed),
        )
        return result
```
- [ ] ~~**Step 3b: Update U2.4's call sites for the new signature**~~ *(superseded — U2.4's three call sites are already correct at HEAD against the live `national_spread` keyword; applying these replacements would break them)*
In `tests/unit/economics/tick/test_financial_integration.py`, in `TestCapitalVol3DefinesWiredIntoFinancialLayer`:
`test_profit_rate_fallback_reads_capital_vol3` — replace `            national_rate=0.05,\n            fictitious=None,` with `            national_rate=0.05,\n            baa_spread=0.0234,\n            fictitious=None,`.
`test_national_county_count_reads_capital_vol3` and `test_default_rate_estimate_reads_capital_vol3` — replace `            national_rate=0.05,` with `            baa_spread=0.0234,` (`_assess_county_financial_crisis` no longer takes `national_rate` at all; `_compute_county_financial_state` takes BOTH).
- [ ] ~~**Step 4: Run tests to verify they pass**~~ *(superseded — these two files already pass at HEAD: `test_financial_integration.py` 21/21, `mypy src` clean across 638 files)*
Run: `mise run test:q -- tests/unit/economics/tick/test_financial_integration.py tests/unit/economics/tick/test_system.py`
Expected: PASS
- [ ] ~~**Step 5: Commit**~~ *(superseded — the fix is already in history as `aedce819`; there is nothing to commit)*
```bash
mise run commit -- "fix(economics): pass the BAA spread, not the effective rate, as credit_spread

financial_crisis/assessment.py:43 documents credit_spread as a RISK
PREMIUM (BAA-AAA / BAA10Y); _assess_county_financial_crisis was passing
national_rate (base_rate + baa_spread) instead, inflating
credit_fragility against a threshold calibrated for a pure spread.
Thread baa_spread through _compute_national_financial_state ->
_compute_county_financial_state -> _assess_county_financial_crisis
alongside (not instead of) national_rate, which the distribution
calculator still needs.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.6: Correct the stale Group C/D + market_scissors docstrings

**Files:**
- Modify: `web/game/engine_bridge.py` — three comment/docstring blocks (in `_carry_tick_dynamics_flows` and `_serialize_territory`), **located by the quoted BEFORE blocks in Step 3, not by line number** (see the base note below).
- Modify: `src/babylon/engine/systems/market_scissors.py` — module docstring line 1, the `PHASE 1 SCOPE` section, and the `MarketScissorsSystem` class docstring; again located by quoted BEFORE blocks.
- Test: `tests/unit/web/test_engine_bridge.py` (append)
- Test: `tests/unit/engine/systems/test_market_system.py` (append)

> **BASE NOTE (deconfliction, 2026-07-18) — this task is still necessary, and base-agnostic.**
> The sibling branch `fix/null-play-coupling` (fog-of-war, unmerged) touches **both** files this
> task edits. Checked explicitly: it did **not** fix these stale docstrings — the
> `both gating services are unwired` / `NOT_YET_COMPUTED — never relabeled live` /
> `Phase 1 SHADOW ONLY` text is still present verbatim, exactly once, on `dev` **and** on
> `fix/null-play-coupling`. So U2.6 is not redundant and must still run.
>
> All six BEFORE blocks below are byte-identical across the two refs, so they locate correctly on
> either base. Line numbers were removed because the sibling shifts `engine_bridge.py` by ~+439/+517.
>
> *How to tell which base you are on:* `grep -c '_resolve_player_org_id' web/game/engine_bridge.py`
> → `0` = sibling not landed (plain `dev`); `1` = landed. Proceed unchanged either way.
>
> *One tight spot, verified safe.* On `fix/null-play-coupling`, `_serialize_territory` ends
> `payload = {` where `dev` has `return {` (they post-process through `apply_fog`). That line sits
> **two lines below** the docstring this task rewrites — close, but the intervening
> `territory_id = t.id` plus the retained `"""` give three-way merge enough common context: a
> `git merge-file` simulation of this exact edit against their branch produced **0 conflicts**,
> preserving both their `payload = {` and our `CORRECTED 2026-07-18` text. Do **not** extend the
> replacement past the closing `"""` — including `return {` in the edit is what would break it.
>
> *`market_scissors.py` has real concurrent edits.* The sibling swapped four raw
> `node_type="territory"` / `"social_class"` strings for `NodeType.TERRITORY` / `NodeType.SOCIAL_CLASS`
> (the `_node_type` vocabulary rule in CLAUDE.md). Those hunks are at unrelated lines and merge
> clean with this task's three docstring edits (simulated: 0 conflicts, `NodeType.*` and the
> ADR078 wording both survive). Nothing to do — just do not "helpfully" revert them if you see
> them post-merge; the enum form is correct.
>
> *If a BEFORE block is NOT found verbatim:* *STOP — do not pattern-match a near approximation.*
> Re-verify against both refs
> (`git show dev:<file> | grep -F '<anchor>'`, same for `fix/null-play-coupling`) and escalate
> with which ref lost it. A docstring this task claims is stale may have been corrected by
> someone else, in which case the right move is to *shrink* this task, not to re-fix it.

**Interfaces:**
- Consumes: `_carry_tick_dynamics_flows`, `_serialize_territory` (`game.engine_bridge`); `MarketScissorsSystem`, module docstring (`babylon.engine.systems.market_scissors`).
- Produces: none — documentation-only, zero behavior change.

- [ ] **Step 1: Write the failing tests**
```python
# --- tests/unit/web/test_engine_bridge.py (append) ---
import inspect

from game.engine_bridge import _carry_tick_dynamics_flows, _serialize_territory


class TestGroupCDDocstringsHonest:
    """Honesty sweep (U2, Row K): the 'both gating services are unwired'
    claim was false for 8 of 9 Group C/D rows."""

    def test_carry_tick_dynamics_flows_comment_corrected(self) -> None:
        source = inspect.getsource(_carry_tick_dynamics_flows)
        assert "both gating services are unwired" not in source
        assert "CORRECTED 2026-07-18" in source

    def test_serialize_territory_docstring_corrected(self) -> None:
        doc = _serialize_territory.__doc__ or ""
        assert "both gating services are unwired" not in doc
        assert "CORRECTED 2026-07-18" in doc
        source = inspect.getsource(_serialize_territory)
        assert "until turnover_profile_source /" not in source


# Catalog docstring tests: NONE IN THIS TASK.
# U5.3's TestCatalogDocstringAccuracy is authoritative — it pins the
# docstring against build_default_registry().keys, so it cannot go stale
# again at 6 OR at 10. A hardcoded "six bound contradictions" assertion
# here would red the moment U5.2 grows the registry to ten.


# --- tests/unit/engine/systems/test_market_system.py (append) ---
import babylon.engine.systems.market_scissors as market_scissors_module
from babylon.engine.systems.market_scissors import MarketScissorsSystem


class TestMarketScissorsDocstringHonest:
    """Honesty sweep (U2, Row M): docstrings said 'Phase 1 SHADOW ONLY ...
    no correction feedback', but Phase 2 (ADR078) is wired and firing by
    default."""

    def test_module_docstring_no_longer_claims_shadow_only(self) -> None:
        doc = market_scissors_module.__doc__ or ""
        assert "Phase 1 SHADOW ONLY" not in doc
        assert "no correction feedback" not in doc

    def test_class_docstring_no_longer_claims_shadow(self) -> None:
        doc = MarketScissorsSystem.__doc__ or ""
        assert "Phase 1 SHADOW" not in doc
```
- [ ] **Step 2: Run tests to verify they fail**
Run: `mise run test:q -- tests/unit/web/test_engine_bridge.py::TestGroupCDDocstringsHonest tests/unit/engine/systems/test_market_system.py::TestMarketScissorsDocstringHonest`
Expected: FAIL — all `assert ... not in ...` / `assert ... in ...` checks fail against the current stale text.
- [ ] **Step 3: Write minimal implementation**
```python
# web/game/engine_bridge.py, inside `_carry_tick_dynamics_flows` — BEFORE
# (locate by the line "# both gating services are unwired, so these are the frozen";
#  one hit on dev AND on fix/null-play-coupling):
#                 # Playability Spine Task 20 (spec-116 4d.5): Group C
#                 # (circulation, Feature 023) + Group D (financial
#                 # distribution, Feature 024) join the carry — the write-site
#                 # expressions from graph_bridge.py:128-197 mirrored
#                 # byte-for-byte, fallback constants included. DECLARED-DARK:
#                 # both gating services are unwired, so these are the frozen
#                 # fallbacks until then (SEAM_REGISTRY rows stay
#                 # NOT_YET_COMPUTED — never relabeled live).
# AFTER:
                # Playability Spine Task 20 (spec-116 4d.5): Group C
                # (circulation, Feature 023) + Group D (financial
                # distribution, Feature 024) join the carry — the write-site
                # expressions from graph_bridge.py:128-197 mirrored
                # byte-for-byte. CORRECTED 2026-07-18
                # (vol3-money-scissors-design honesty sweep, U2): "both
                # gating services are unwired" was stale — real wired
                # implementations exist for 8 of 9 Group C/D rows;
                # SEAM_REGISTRY is the authoritative per-row wiring status,
                # not this comment.

# web/game/engine_bridge.py, the `_serialize_territory` DOCSTRING — BEFORE
# (locate by "are unwired, so post-boundary values are the engine's fallback constants";
#  one hit on dev AND on fix/null-play-coupling). The replacement ENDS at the closing
# `"""` — do NOT absorb the following `territory_id = t.id` / `return {` (`payload = {`
# on the sibling branch); see the base note:
#     Playability Spine Task 20 (spec-116 4d.5): the Feature-023 circulation
#     family and Feature-024 financial-distribution family join the same
#     ``tick_``-prefixed graph-attr pattern, serialized DECLARED-DARK — the
#     gating services (``turnover_profile_source``/``interest_calculator``)
#     are unwired, so post-boundary values are the engine's fallback constants
#     (0.0/False/0, plus ``tick_housing_fictitious_fraction``'s honest
#     ``None``). The wire keys keep their ``tick_`` prefix (registry
#     ``wire_keys``) — none collides with an existing payload key or Territory
#     model field. SEAM_REGISTRY rows: Groups C/D, ``NOT_YET_COMPUTED`` (a
#     FRED-backed sibling implementation exists but is not wired into this
#     pipeline — computable, just not yet wired; frozen constants are never
#     relabeled live).
#     """
# AFTER:
    Playability Spine Task 20 (spec-116 4d.5): the Feature-023 circulation
    family and Feature-024 financial-distribution family join the same
    ``tick_``-prefixed graph-attr pattern. The wire keys keep their
    ``tick_`` prefix (registry ``wire_keys``) — none collides with an
    existing payload key or Territory model field. CORRECTED 2026-07-18
    (vol3-money-scissors-design honesty sweep, U2): this docstring
    previously claimed both gating services were categorically unwired and
    every row ``NOT_YET_COMPUTED``; that was false for 8 of 9 Group C/D
    rows. Consult SEAM_REGISTRY for the authoritative per-row wiring
    status — a row still showing its fallback constant (0.0/False/0, plus
    ``tick_housing_fictitious_fraction``'s honest ``None``) means that
    SPECIFIC gating service is unwired on this path, not the whole family.
    """

# web/game/engine_bridge.py, the inline comment above the Group C/D payload keys — BEFORE
# (locate by "# the engine's fallback constants until turnover_profile_source /";
#  one hit on dev AND on fix/null-play-coupling):
#         # Playability Spine Task 20 (spec-116 4d.5): Group C (circulation,
#         # Feature 023) + Group D (financial distribution, Feature 024),
#         # serialized DECLARED-DARK under their registry wire keys (tick_
#         # prefix kept — the tick_median_wage collision precedent). Values are
#         # the engine's fallback constants until turnover_profile_source /
#         # interest_calculator are wired; None before the first boundary.
# AFTER:
        # Playability Spine Task 20 (spec-116 4d.5): Group C (circulation,
        # Feature 023) + Group D (financial distribution, Feature 024),
        # serialized under their registry wire keys (tick_ prefix kept —
        # the tick_median_wage collision precedent). CORRECTED 2026-07-18
        # (vol3-money-scissors-design honesty sweep, U2): most of these
        # rows now carry real wired values — see SEAM_REGISTRY for the
        # authoritative per-row status, not this comment.


# catalog.py module docstring: NO EDIT IN THIS TASK.
# U5.3 is authoritative — it rewrites line 1 to "ten bound contradictions"
# and adds the price_value bullet plus the four Volume III bullets in one
# pass. Editing it here would be overwritten and would leave a duplicate
# price_value bullet.


# src/babylon/engine/systems/market_scissors.py, MODULE docstring first line — BEFORE
# (locate by "Phase 1 SHADOW ONLY"; one hit on dev AND on fix/null-play-coupling).
# INDENTATION: this is a MODULE docstring — it starts at COLUMN 0, not indented.
# BEFORE:
# """Market-scissors system — Phase 1 SHADOW ONLY (Program 23, ADR077).
# AFTER (column 0):
"""Market-scissors system (Program 23, ADR077/ADR078 — correction feedback LIVE).

# market_scissors.py, the PHASE 1 SCOPE section of the same MODULE docstring — BEFORE
# (locate by "PHASE 1 SCOPE (binding): observe-only shadow."; one hit on both refs).
# INDENTATION: still inside the module docstring — COLUMN 0, with 2-space bullet
# continuations. (Corrected 2026-07-18: an earlier draft of this plan indented these
# AFTER lines by 4 spaces, which would have silently mangled the module docstring.)
# BEFORE:
# PHASE 1 SCOPE (binding): observe-only shadow.
#
# - State home: ``G.graph["market"]`` metadata (the ``wealth_distribution``
#   round-trip pattern; ``WorldState.market`` carries it across facade ticks).
# - Nothing reads it to change tick outputs: no correction feedback into
#   wealth, credit, or the reserve army (Phase 2, owner-gated), so the sampled
#   qa:regression checkpoints stay byte-identical.
# AFTER (column 0):
PHASE 2 SCOPE (current, ADR078 promotion ceremony): the correction feeds
back into the material base by default (``GameDefines.market.feedback_enabled``).

- State home: ``G.graph["market"]`` metadata (the ``wealth_distribution``
  round-trip pattern; ``WorldState.market`` carries it across facade ticks).
- The correction snap DOES change tick outputs: it evaporates claim-holder
  wealth, swells the reserve army, and publishes ``MARKET_CORRECTION``
  (``feedback_enabled=False`` restores the old Phase-1 observe-only
  behavior for byte-comparison runs).

# market_scissors.py, the MarketScissorsSystem CLASS docstring — BEFORE
# (locate by "Phase 1 SHADOW: the national"; one hit on both refs).
# INDENTATION: class docstring — 4 spaces, as written below.
# BEFORE:
#     """Phase 1 SHADOW: the national price⟷value scissors axis."""
# AFTER:
    """The national price⟷value scissors axis (Phase 2: correction feedback live by default)."""
```
- [ ] **Step 4: Run tests to verify they pass**
Run: `mise run test:q -- tests/unit/web/test_engine_bridge.py::TestGroupCDDocstringsHonest tests/unit/engine/systems/test_market_system.py::TestMarketScissorsDocstringHonest`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
mise run commit -- "docs(economics): correct stale Group C/D, catalog, and scissors docstrings

Two defects: (1) engine_bridge.py claimed Group C/D gating services
are categorically unwired — false for 8 of 9 rows; SEAM_REGISTRY is
the authoritative source now cited instead. (2) market_scissors.py
still said 'Phase 1 SHADOW ONLY ... no correction feedback' — Phase 2
(ADR078) has been wired and firing by default since the promotion
ceremony. Documentation only, zero behavior change. The catalog
docstring is corrected by U5.3, which owns the 6 -> 10 growth.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.7: Mark `RentCategory` dormant (zero behavioral consumers)

**Files:**
- Modify: `src/babylon/domain/economics/rent/types.py:13-30`
- Test: `tests/unit/economics/rent/test_types.py` (create if absent, else append)

**Interfaces:**
- Consumes: `RentCategory` (`babylon.domain.economics.rent.types`), re-exported unchanged via `babylon.domain.economics.rent.__init__`.
- Produces: none new — enum values/behavior unchanged, docstring only.

- [ ] **Step 1: Write the failing test**
```python
# --- tests/unit/economics/rent/test_types.py (create if the file does not
# already exist; otherwise append the class to the existing file) ---
"""Tests for rent/types.py honesty markers.

Feature: 024-capital-volume-iii
"""

from __future__ import annotations

import pytest

from babylon.domain.economics.rent.types import RentCategory

pytestmark = pytest.mark.unit


class TestRentCategoryDormantMarker:
    """Honesty sweep (U2, Row N): RentCategory has zero behavioral
    consumers — RentExtraction carries the three categories as discrete
    named fields, never keyed by this enum. Documented as dormant rather
    than silently rotting."""

    def test_docstring_declares_dormant(self) -> None:
        doc = RentCategory.__doc__ or ""
        assert "DORMANT" in doc

    def test_enum_values_unchanged(self) -> None:
        assert RentCategory.AGRICULTURAL == "agricultural"
        assert RentCategory.RESOURCE == "resource"
        assert RentCategory.URBAN == "urban"
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/rent/test_types.py::TestRentCategoryDormantMarker::test_docstring_declares_dormant`
Expected: FAIL — `assert 'DORMANT' in doc` is `False` against the current docstring.
- [ ] **Step 3: Write minimal implementation**
```python
# src/babylon/domain/economics/rent/types.py:13-30 — BEFORE:
# class RentCategory(StrEnum):
#     """Category of ground rent extraction.
#
#     Feature: 024-capital-volume-iii (FR-007)
#
#     Marx distinguished differential rent (surplus profit from better
#     land/location) and absolute rent (monopoly payment for any land
#     access). Both operate across three economic sectors.
#
#     Values:
#         AGRICULTURAL: Farmland rent (differential by soil fertility/location).
#         RESOURCE: Mining, oil/gas rent (differential by deposit quality).
#         URBAN: Building site rent, commercial real estate (differential by location).
#     """
#
#     AGRICULTURAL = "agricultural"
#     RESOURCE = "resource"
#     URBAN = "urban"
# AFTER:
class RentCategory(StrEnum):
    """Category of ground rent extraction.

    Feature: 024-capital-volume-iii (FR-007)

    Marx distinguished differential rent (surplus profit from better
    land/location) and absolute rent (monopoly payment for any land
    access). Both operate across three economic sectors.

    DORMANT (honesty sweep, spec 2026-07-18 vol3-money-scissors-design,
    U2 §3.6): zero behavioural consumers as of this writing —
    :class:`RentExtraction` carries the three categories as discrete
    named fields (``agricultural_rent``/``resource_rent``/``urban_rent``),
    never keyed by this enum. Reserved for a future category-keyed rent
    API; wire it or remove it rather than letting it silently rot
    further if it is still unconsumed the next time this module is
    touched.

    Values:
        AGRICULTURAL: Farmland rent (differential by soil fertility/location).
        RESOURCE: Mining, oil/gas rent (differential by deposit quality).
        URBAN: Building site rent, commercial real estate (differential by location).
    """

    AGRICULTURAL = "agricultural"
    RESOURCE = "resource"
    URBAN = "urban"
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/rent/test_types.py`
Expected: PASS
- [ ] **Step 5: Commit**
```bash
mise run commit -- "docs(economics): mark RentCategory dormant (honesty sweep, U2)

RentCategory has zero behavioral consumers — RentExtraction carries
its three categories as discrete named fields, never keyed by this
enum. Document it as dormant with a removal/wire-it directive rather
than letting a dead export silently persist.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

### Task U2.8: Cross tick 1612 in a real loop — the year ceiling is a live crash, prove it dead

**Files:**
- Create: `tests/unit/economics/tick/test_year_ceiling_crossing.py`

**Interfaces:**
- Consumes: `SIM_EPOCH_YEAR`, `WEEKS_PER_YEAR` (`babylon.kernel.sim_clock`);
  `TickDynamicsSystem._compute_national_params` (U2.1);
  `DefaultInterestCalculator`/`DefaultFictitiousCapitalCalculator`/
  `DefaultDistributionCalculator` (U2.2's guards);
  `MockMELTCalculator` (`tests/unit/economics/tick/conftest.py:36`) and
  `_make_services` (`tests/unit/economics/tick/test_system.py:59`) — **two different modules**.
  `_make_services` is NOT in that package's `conftest.py`; importing it from there is an
  `ImportError` at collection. U3.3 imports the same pair correctly; match its form.
- Produces: the standing proof of U2's third acceptance criterion.

- [ ] **Step 1: Write the failing test**
```python
"""U2 acceptance (design §4, §8.3): year 2041 arrives at tick ~1612 of a
5200-tick campaign — INSIDE every canonical run — and NationalTickParameters
is on the already-live MELT path. This walks the real clock across that
boundary and asserts no ValidationError escapes, and that the Vol III layer
degrades to NoDataSentinel rather than raising.
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.domain.economics.credit.interest import DefaultInterestCalculator
from babylon.domain.economics.tensor import NoDataSentinel
from babylon.domain.economics.tick.system import TickDynamicsSystem
from babylon.kernel.sim_clock import SIM_EPOCH_YEAR, WEEKS_PER_YEAR
from tests.unit.economics.tick.conftest import MockMELTCalculator
from tests.unit.economics.tick.test_system import _make_services

pytestmark = pytest.mark.unit

#: The exact tick the design names as the crash point (§1.4, §8.3).
_CEILING_CROSSING_TICK = (2041 - SIM_EPOCH_YEAR) * WEEKS_PER_YEAR


def test_the_documented_crash_tick_is_inside_a_canonical_campaign() -> None:
    """Pin the arithmetic the design's severity claim rests on."""
    assert _CEILING_CROSSING_TICK == pytest.approx(1612, abs=WEEKS_PER_YEAR)
    assert _CEILING_CROSSING_TICK < 5200


def test_melt_path_survives_every_year_of_a_5200_tick_campaign() -> None:
    """NationalTickParameters is built once per year boundary for 100 years."""
    system = TickDynamicsSystem()
    services = _make_services(melt_calculator=MockMELTCalculator(tau=62.0, accept_any_year=True))
    for tick in range(0, 5200, WEEKS_PER_YEAR):  # bounded: 100 iterations
        year = SIM_EPOCH_YEAR + tick // WEEKS_PER_YEAR
        try:
            params = system._compute_national_params(year, services, prev_coefficients=None)
        except ValidationError as exc:  # pragma: no cover — the defect under test
            pytest.fail(f"ValidationError at tick {tick} (year {year}): {exc}")
        assert params is not None
        assert params.year == year, (
            f"year {year} was silently relabeled {params.year} — the clamp is back"
        )


def test_vol3_layer_degrades_rather_than_raising_past_the_ceiling() -> None:
    """Past MODELED_YEAR_CEILING the financial layer is ABSENT, not broken."""
    from tests.unit.economics.credit.conftest import MockInterestRateSource

    calc = DefaultInterestCalculator(
        rate_source=MockInterestRateSource(data={2041: (0.03, 0.04, 0.02)})
    )
    for tick in range(_CEILING_CROSSING_TICK, 5200, WEEKS_PER_YEAR):  # bounded
        year = SIM_EPOCH_YEAR + tick // WEEKS_PER_YEAR
        result = calc.compute_interest_rate_state(year)
        assert isinstance(result, NoDataSentinel), (
            f"year {year} produced a structured model past the modeled window"
        )
        assert "modeled" in result.reason.lower()
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/tick/test_year_ceiling_crossing.py`
Expected against a pre-U2.1 checkout: `test_melt_path_survives_every_year_of_a_5200_tick_campaign`
fails with `AssertionError: year 2041 was silently relabeled 2040 — the clamp is back`.
Verify the red by temporarily restoring `clamped_year = min(max(year, 2007), 2040)` in
`src/babylon/domain/economics/tick/system/__init__.py` (the line U2.1 removed), running the file,
observing that exact assertion, and then **restoring the file immediately**:
```bash
git checkout -- src/babylon/domain/economics/tick/system/__init__.py
```
Do not proceed to Step 4 until `git status` reports that file clean. The injected clamp must never
reach Step 5's commit.
- [ ] **Step 3: Write minimal implementation**
No new production code — U2.1 and U2.2 supply it. This task converts two
isolated unit assertions into the campaign-horizon proof §4 asks for.
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_year_ceiling_crossing.py`
Expected: PASS (3 passed).
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(economics): U2 acceptance — a 5200-tick campaign crosses 2040 clean

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U3.1: Publish/read `NationalFinancialParameters` via `graph_bridge`

**Files:**
- Modify: `src/babylon/domain/economics/tick/graph_bridge.py:16-38` (imports), `:201-296` (insert new functions between `read_tick_state_from_graph` and `_reconstruct_tick_state`), `:326-331` (`__all__`)
- Test: `tests/unit/economics/tick/test_graph_bridge.py`

**Interfaces:**
- Consumes: `NationalFinancialParameters` (`src/babylon/domain/economics/tick/types.py:454-493`, fields `interest_rate_state`, `credit_state`, `fictitious_capital`, `counter_tendencies`, `monetary_adjustment`, all `X | None = None`); `InterestRateState`, `FictitiousCapitalStock` (`src/babylon/domain/economics/credit/types.py`); `GraphProtocol.set_graph_attr(key: str, value: Any) -> None` / `get_graph_attr(key: str, default: Any = None) -> Any` (`src/babylon/kernel/graph_protocol.py:350,365`)
- Produces: `NATIONAL_FINANCIAL_ATTR: Final[str] = "national_financial"`; `write_national_financial_state_to_graph(graph, params) -> None`; `read_national_financial_state_from_graph(graph) -> NationalFinancialParameters | None` — all three consumed by Task U3.2 and Task U3.3, and later by U4/U5/U6's readers.

- [ ] **Step 1: Write the failing test**

Append to `tests/unit/economics/tick/test_graph_bridge.py`, and update its import block at the top of the file:

```python
from babylon.domain.economics.credit.types import FictitiousCapitalStock, InterestRateState
from babylon.domain.economics.distribution.types import DebtAccumulation, SurplusValueDistribution
from babylon.domain.economics.financial_crisis.types import FinancialCrisisAssessment
from babylon.domain.economics.rent.types import HousingValueDecomposition, RentExtraction
from babylon.domain.economics.tick.graph_bridge import (
    NATIONAL_FINANCIAL_ATTR,
    TICK_DYNAMICS_KEY,
    read_national_financial_state_from_graph,
    read_tick_state_from_graph,
    write_national_financial_state_to_graph,
    write_tick_state_to_graph,
)
from babylon.domain.economics.tick.types import (
    BifurcationRiskMetric,
    CrisisPhase,
    CrisisState,
    NationalFinancialParameters,
    NationalTickParameters,
    SimulationTickState,
    SmoothedCoefficients,
)
from babylon.topology.graph import BabylonGraph
from tests.unit.economics.tick.conftest import WAYNE_FIPS, build_territory_graph
```

then append this new class at the end of the file (after the existing `TestReadTickStateFromGraph`/`TestWriteFinancialState` classes):

```python
class TestWriteNationalFinancialStateToGraph:
    """Tests for write/read of NationalFinancialParameters (vol3-money-scissors U3).

    Feature: 024-capital-volume-iii / vol3-money-scissors U3
    Publishes the previously-transient FictitiousCapitalStock + national
    interest state so a CONSEQUENCE-phase System can read them later in
    the same tick (design doc SS3.2, SS1.2).
    """

    def _sample_params(self) -> NationalFinancialParameters:
        """Build a fully-populated interest+fictitious-capital sample."""
        return NationalFinancialParameters(
            interest_rate_state=InterestRateState(
                year=2015,
                base_rate=0.25,
                treasury_10y=2.27,
                baa_spread=2.64,
            ),
            fictitious_capital=FictitiousCapitalStock(
                year=2015,
                government_debt=18e12,
                corporate_equity=20e12,
                corporate_debt=8e12,
                household_debt=14e12,
            ),
        )

    def test_write_stores_model_dump_dict_under_national_financial_attr(self) -> None:
        """Verify write stores a plain dict (model_dump()), not the object."""
        graph = build_territory_graph()
        params = self._sample_params()

        write_national_financial_state_to_graph(graph, params)

        assert NATIONAL_FINANCIAL_ATTR in graph.graph
        stored = graph.graph[NATIONAL_FINANCIAL_ATTR]
        assert isinstance(stored, dict)
        assert stored["interest_rate_state"]["base_rate"] == 0.25
        assert stored["fictitious_capital"]["government_debt"] == 18e12

    def test_read_returns_none_when_nothing_published(self) -> None:
        """Verify read returns None before any write (absence, not a fake zero)."""
        graph = build_territory_graph()
        assert read_national_financial_state_from_graph(graph) is None

    def test_round_trip_reconstructs_national_financial_parameters(self) -> None:
        """Verify write-then-read reconstructs equivalent nested Pydantic models."""
        graph = build_territory_graph()
        params = self._sample_params()

        write_national_financial_state_to_graph(graph, params)
        result = read_national_financial_state_from_graph(graph)

        assert result is not None
        assert result.interest_rate_state is not None
        assert result.interest_rate_state.base_rate == 0.25
        assert result.interest_rate_state.effective_rate == 0.25 + 2.64
        assert result.fictitious_capital is not None
        assert result.fictitious_capital.total_claims == 60e12
        assert result.credit_state is None
        assert result.counter_tendencies is None
        assert result.monetary_adjustment is None
```

- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge.py::TestWriteNationalFinancialStateToGraph`
Expected: FAIL at collection with `ImportError: cannot import name 'NATIONAL_FINANCIAL_ATTR' from 'babylon.domain.economics.tick.graph_bridge'` (the constant and both functions don't exist yet).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/domain/economics/tick/graph_bridge.py`, replace the import block (lines 16-38):

```python
from __future__ import annotations

from typing import TYPE_CHECKING, Any, Final

if TYPE_CHECKING:
    from babylon.kernel.graph_protocol import GraphProtocol

from babylon.domain.economics.dynamics.types import ClassDistribution
from babylon.domain.economics.tick.derived_rates import DerivedRateCalculator
from babylon.domain.economics.tick.types import (
    BifurcationRiskMetric,
    CountyEconomicState,
    CrisisPhase,
    CrisisState,
    DerivedRates,
    NationalFinancialParameters,
    NationalTickParameters,
    SimulationTickState,
    SmoothedCoefficients,
    TickSummary,
)

# Graph metadata key
TICK_DYNAMICS_KEY: str = "tick_dynamics"
```

Then insert this block between the end of `read_tick_state_from_graph` (the `)  # pragma: no mutate` that closes its `return SimulationTickState(...)`) and `def _reconstruct_tick_state(`:

```python
# Graph metadata key — Feature 024/vol3-money-scissors U3: the national
# financial state (interest rate environment + fictitious capital stock)
# published once per tick so CONSEQUENCE-phase Systems can read it. Kept
# separate from TICK_DYNAMICS_KEY / NationalTickParameters (the MELT/gamma
# carrier) — different lifecycle (see NationalFinancialParameters docstring).
NATIONAL_FINANCIAL_ATTR: Final[str] = "national_financial"


def write_national_financial_state_to_graph(  # pragma: no mutate — data serialization
    graph: GraphProtocol,
    params: NationalFinancialParameters,
) -> None:
    """Write NationalFinancialParameters to the shared graph.

    Feature: 024-capital-volume-iii / vol3-money-scissors U3

    Stores ``params.model_dump()`` (a plain dict, not the Pydantic object
    itself) under ``graph.graph[NATIONAL_FINANCIAL_ATTR]`` so any System
    later in the same tick can read it via
    :func:`read_national_financial_state_from_graph`.

    Args:
        graph: Mutable NetworkX graph or GraphProtocol (modified in-place).
        params: National financial state to publish.
    """
    graph.set_graph_attr(NATIONAL_FINANCIAL_ATTR, params.model_dump())  # pragma: no mutate


def read_national_financial_state_from_graph(  # pragma: no mutate — data serialization
    graph: GraphProtocol,
) -> NationalFinancialParameters | None:
    """Read NationalFinancialParameters from the shared graph.

    Feature: 024-capital-volume-iii / vol3-money-scissors U3

    Args:
        graph: NetworkX graph or GraphProtocol possibly containing the
            published financial state.

    Returns:
        Reconstructed NationalFinancialParameters, or None if nothing has
        been published this tick.
    """
    data: dict[str, Any] | None = graph.get_graph_attr(  # pragma: no mutate
        NATIONAL_FINANCIAL_ATTR
    )
    if data is None:  # pragma: no mutate
        return None  # pragma: no mutate
    return NationalFinancialParameters.model_validate(data)  # pragma: no mutate
```

Finally, replace `__all__` (lines 326-331):

```python
__all__ = [
    "NATIONAL_FINANCIAL_ATTR",
    "TICK_DYNAMICS_KEY",
    "_reconstruct_tick_state",
    "read_national_financial_state_from_graph",
    "read_tick_state_from_graph",
    "write_national_financial_state_to_graph",
    "write_tick_state_to_graph",
]
```

- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge.py`
Expected: PASS (all tests in the file, including the pre-existing ones — confirms the new imports didn't break anything).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): publish NationalFinancialParameters via graph_bridge

U3.1: give the transient FictitiousCapitalStock/InterestRateState a home
on the graph under NATIONAL_FINANCIAL_ATTR, separate from
TICK_DYNAMICS_KEY/NationalTickParameters (different lifecycle).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U3.2: Wire `_compute_national_financial_state` to instantiate + publish `NationalFinancialParameters`

**Files:**
- Modify: `tests/unit/economics/tick/conftest.py:209-211` (insert two new mock calculator classes), `:9-25` (imports)
- Modify: `tests/unit/economics/tick/test_system.py:11-48` (imports), append new test class at EOF (after line 2228)
- Modify: `src/babylon/domain/economics/tick/system/__init__.py` — the import block (~`:40-74`), the Step 5.5 call site (~`:216-222`), `_compute_financial_layer` (~`:1390`) and `_compute_national_financial_state` (~`:1480`). **Line numbers are hints only; anchor on the quoted `def` lines.** They shifted when commit `aedce819` grew the method.
- Test: `tests/unit/economics/tick/test_system.py`

**Interfaces:**
- Consumes: `NATIONAL_FINANCIAL_ATTR`, `write_national_financial_state_to_graph`, `read_national_financial_state_from_graph` (Task U3.1); `NationalFinancialParameters` (`tick/types.py:454-493`)
- Produces: `TickDynamicsSystem._compute_national_financial_state(services, year, graph) -> tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]` — **the only change this task makes to the contract is adding the `graph` parameter.** The **4-tuple** `(national_rate, national_spread, fictitious, interest_unavailable_reason)` shipped by commit `aedce819` is PRESERVED verbatim, including its `float | None` honest-absence typing (Constitution III.11). U3 must NOT collapse it to a 3-tuple and must NOT re-introduce `else 0.0` fabricated zeros — U2.5's original 3-tuple brief is superseded; see the supersession note at Task U2.5. `credit_state` is populated by the new Task U3.4, which U5.7 depends on. — this is the exact method U4/U5/U6 will extend to also populate `credit_state`/`counter_tendencies`/`monetary_adjustment`; `MockInterestRateCalculator`, `MockFictitiousCapitalCalculator` test fixtures (reused by Task U3.3 and by U1).

- [ ] **Step 1: Write the failing test**

In `tests/unit/economics/tick/conftest.py`, first add an import (insert before the `dynamics.types` import at line 15, keeping the block alphabetical):

```python
from babylon.domain.economics.credit.types import FictitiousCapitalStock, InterestRateState
```

Then insert these two mock classes between `MockImperialRentCalculator` (ends `return self._phi_hour > 0`) and `class MockTensor:`:

```python
class MockInterestRateCalculator:
    """Mock InterestRateCalculator returning a fixed InterestRateState.

    Args:
        base_rate: Federal funds rate (FEDFUNDS) to return.
        treasury_10y: 10-year Treasury yield (DGS10) to return.
        baa_spread: Baa corporate spread (BAA10Y) to return.
        force_sentinel: If True, always return NoDataSentinel.
    """

    def __init__(
        self,
        base_rate: float = 0.25,
        treasury_10y: float = 2.27,
        baa_spread: float = 2.64,
        *,
        force_sentinel: bool = False,
    ) -> None:
        self._base_rate = base_rate
        self._treasury_10y = treasury_10y
        self._baa_spread = baa_spread
        self._force_sentinel = force_sentinel

    def compute_interest_rate_state(self, year: int) -> InterestRateState | NoDataSentinel:
        """Return fixed InterestRateState or NoDataSentinel."""
        if self._force_sentinel:
            return NoDataSentinel(fips="USA", year=year, reason="Forced sentinel for testing")
        return InterestRateState(
            year=year,
            base_rate=self._base_rate,
            treasury_10y=self._treasury_10y,
            baa_spread=self._baa_spread,
        )


class MockFictitiousCapitalCalculator:
    """Mock FictitiousCapitalCalculator returning a fixed FictitiousCapitalStock.

    Args:
        government_debt: Federal debt (GFDEBTN) to return.
        corporate_equity: Stock market cap to return.
        corporate_debt: Corporate bonds/loans to return.
        household_debt: Mortgages/consumer/student debt to return.
        force_sentinel: If True, always return NoDataSentinel.
    """

    def __init__(
        self,
        government_debt: float = 18e12,
        corporate_equity: float = 20e12,
        corporate_debt: float = 8e12,
        household_debt: float = 14e12,
        *,
        force_sentinel: bool = False,
    ) -> None:
        self._government_debt = government_debt
        self._corporate_equity = corporate_equity
        self._corporate_debt = corporate_debt
        self._household_debt = household_debt
        self._force_sentinel = force_sentinel

    def compute_fictitious_capital(self, year: int) -> FictitiousCapitalStock | NoDataSentinel:
        """Return fixed FictitiousCapitalStock or NoDataSentinel."""
        if self._force_sentinel:
            return NoDataSentinel(fips="USA", year=year, reason="Forced sentinel for testing")
        return FictitiousCapitalStock(
            year=year,
            government_debt=self._government_debt,
            corporate_equity=self._corporate_equity,
            corporate_debt=self._corporate_debt,
            household_debt=self._household_debt,
        )
```

In `tests/unit/economics/tick/test_system.py`, add to the imports (extend the `tests.unit.economics.tick.conftest` import list and add a `graph_bridge` import):

```python
from babylon.domain.economics.tick.graph_bridge import read_national_financial_state_from_graph
```
and add `MockFictitiousCapitalCalculator, MockInterestRateCalculator,` into the existing `from tests.unit.economics.tick.conftest import (...)` list (alphabetically, before `MockGammaIIICalculator`).

Then append this new class at the end of the file (after the last `test_invalid_hex_grid_type_is_noop` test, i.e. after `assert hex_attrs == []`):

```python


class TestComputeNationalFinancialState:
    """Tests for _compute_national_financial_state (vol3-money-scissors U3.2).

    U3 keeps the existing 4-tuple return contract (national_rate,
    national_spread, fictitious, interest_unavailable_reason) and
    additionally publishes a NationalFinancialParameters graph attribute
    so downstream CONSEQUENCE-phase Systems can read it (see
    graph_bridge.NATIONAL_FINANCIAL_ATTR) — it previously died as a
    transient local.
    """

    def test_publishes_national_financial_parameters_to_graph(self) -> None:
        """Verify the method instantiates + publishes NationalFinancialParameters."""
        services = _make_services(
            interest_calculator=MockInterestRateCalculator(
                base_rate=0.25,
                treasury_10y=2.27,
                baa_spread=2.64,
            ),
            fictitious_capital_calculator=MockFictitiousCapitalCalculator(
                government_debt=18e12,
                corporate_equity=20e12,
                corporate_debt=8e12,
                household_debt=14e12,
            ),
        )
        graph = build_territory_graph()
        system = TickDynamicsSystem()

        (
            national_rate,
            national_spread,
            fictitious,
            interest_unavailable_reason,
        ) = system._compute_national_financial_state(services, 2015, graph)

        # Existing tuple contract is unchanged — county-level callers still
        # get the 4-tuple (float | None, float | None,
        # FictitiousCapitalStock | None, str | None).
        assert national_rate == pytest.approx(0.25 + 2.64)
        assert national_spread == pytest.approx(2.64)
        assert fictitious is not None
        assert fictitious.total_claims == pytest.approx(60e12)
        assert interest_unavailable_reason is None

        published = read_national_financial_state_from_graph(graph)
        assert published is not None
        assert published.interest_rate_state is not None
        assert published.interest_rate_state.base_rate == pytest.approx(0.25)
        assert published.fictitious_capital is not None
        assert published.fictitious_capital.total_claims == pytest.approx(60e12)

    def test_publishes_honest_none_when_calculators_return_sentinel(self) -> None:
        """Verify a NoDataSentinel from either calculator publishes None fields.

        Constitution III.11: absence must stay None, never a fabricated
        zero — this proves the publish path preserves that even when
        called directly (not gated by _compute_financial_layer's
        interest_calculator-is-None early return).
        """
        services = _make_services(
            interest_calculator=MockInterestRateCalculator(force_sentinel=True),
            fictitious_capital_calculator=None,
        )
        graph = build_territory_graph()
        system = TickDynamicsSystem()

        (
            national_rate,
            national_spread,
            fictitious,
            interest_unavailable_reason,
        ) = system._compute_national_financial_state(services, 2015, graph)

        # Honest absence (Constitution III.11), shipped in aedce819: an
        # absent rate is None, NOT a fabricated 0.0, and the sentinel's own
        # .reason travels with it so the caller can attribute the gap.
        assert national_rate is None
        assert national_spread is None
        assert fictitious is None
        assert interest_unavailable_reason == "Forced sentinel for testing"
        assert services.economics_fallbacks.vol3_interest_sentinel == 1

        published = read_national_financial_state_from_graph(graph)
        assert published is not None
        assert published.interest_rate_state is None
        assert published.fictitious_capital is None
```

- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/tick/test_system.py::TestComputeNationalFinancialState`
Expected: FAIL — **both** tests error with `TypeError: TickDynamicsSystem._compute_national_financial_state() takes 3 positional arguments but 4 were given`. The parameter list at HEAD is `(self, services, year)` — no `graph` yet — so the call raises before either test reaches an assertion.

Note what this red does **not** prove: the *return* arity is already correct at HEAD. Commit `aedce819` shipped the 4-tuple, so the unpacking in Step 1 would succeed on its own; only the missing `graph` parameter is red here. If you instead see `ValueError: not enough values to unpack (expected 4, got 3)`, someone has reverted the method to U2.5's superseded 3-tuple — **STOP and restore the 4-tuple**, do not adapt the test to a 3-tuple.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/domain/economics/tick/system/__init__.py`, update the `graph_bridge` import (lines 51-54):

```python
from babylon.domain.economics.tick.graph_bridge import (
    read_tick_state_from_graph,
    write_national_financial_state_to_graph,
    write_tick_state_to_graph,
)
```

Update the `tick.types` import (lines 57-66) to add `NationalFinancialParameters,` before `NationalTickParameters,`:

```python
from babylon.domain.economics.tick.types import (
    BifurcationRiskMetric,
    CountyEconomicState,
    CrisisPhase,
    CrisisState,
    NationalFinancialParameters,
    NationalTickParameters,
    SimulationTickState,
    SmoothedCoefficients,
    TickSummary,
)
```

Update the Step 5.5 call site (~lines 216-222 at HEAD — anchor on the `# Step 5.5: Compute financial layer (Feature 024)` comment):

```python
        # Step 5.5: Compute financial layer (Feature 024)
        county_states = self._compute_financial_layer(
            county_states,
            national_params,
            services,
            year,
            graph,
        )
```

Replace `_compute_financial_layer`'s signature/docstring and its internal call — anchor on the `def _compute_financial_layer(` line and replace down to and including the `self._compute_national_financial_state(services, year)` unpack. **Everything below that unpack (the county loop, the truncation guard, the preserve-remaining loop, the `return updated`) is left byte-for-byte as-is — do not retype it.**

```python
    def _compute_financial_layer(
        self,
        county_states: dict[str, CountyEconomicState],
        _national_params: NationalTickParameters,
        services: ServicesProtocol,
        year: int,
        graph: GraphProtocol,
    ) -> dict[str, CountyEconomicState]:
        """Compute Volume III financial distribution layer.

        Feature: 024-capital-volume-iii
        Computes national financial parameters once, then distributes
        surplus value and assesses financial crisis per county.

        Args:
            county_states: Current county snapshots.
            _national_params: National economic context (reserved for future use).
            services: ServicesProtocol with financial calculators.
            year: Current simulation year.
            graph: Mutable shared graph — passed through so the national
                financial state can be published under
                ``graph_bridge.NATIONAL_FINANCIAL_ATTR`` (U3).

        Returns:
            Updated county states with financial fields populated.
        """
        # Graceful skip if financial calculators not configured
        if services.interest_calculator is None:
            return county_states

        national_rate, national_spread, fictitious, interest_unavailable_reason = (
            self._compute_national_financial_state(services, year, graph)
        )
```

Replace `_compute_national_financial_state` — anchor on `def _compute_national_financial_state(`
and replace through its `return` statement. **This is the version live at HEAD (commit
`aedce819`) with the `graph` parameter and the publish call added; the 4-tuple and every `| None`
are preserved deliberately.**

```python
    def _compute_national_financial_state(
        self,
        services: ServicesProtocol,
        year: int,
        graph: GraphProtocol,
    ) -> tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]:
        """Compute national-level financial parameters once per tick.

        Feature: 024-capital-volume-iii / vol3-money-scissors U3
        Instantiates NationalFinancialParameters from whatever calculators
        are configured and publishes it via graph_bridge under
        NATIONAL_FINANCIAL_ATTR, so any System later in the same tick can
        see it — it previously died as a transient local (design doc SS1.2).

        Constitution III.11 (honest absence): when the interest calculator
        has no data for ``year``, the national rate is ``None`` — never a
        fabricated ``0.0``. A fabricated zero rate would silently flow into
        ``compute_distribution``'s ``national_interest_rate * implied_capital``
        term and publish a genuine-looking zero-interest distribution
        (code-review finding U2.2-3). Callers must treat ``None`` as "skip
        any computation that depends on the rate," not "treat as zero."

        Returns:
            Tuple of (national_interest_rate_or_none, national_spread_or_none,
            fictitious_capital_or_none, interest_unavailable_reason).
            ``national_spread`` is the BAA10Y risk premium ALONE, kept
            separate from the effective rate because the crisis assessor's
            ``credit_spread`` parameter means the spread, not base+spread
            (U2.3 review finding 5).
            ``interest_unavailable_reason`` is the interest sentinel's own
            ``.reason`` (or ``None`` if the interest calculator succeeded),
            so a caller that must skip work because the rate is absent can
            attribute *why* instead of recording an unexplained gap.
            This 4-tuple landed in commit ``aedce819`` and is preserved
            here verbatim; only the ``graph`` parameter and the publish
            call below are new in U3.2.
        """
        fallbacks = services.economics_fallbacks
        interest_result = services.interest_calculator.compute_interest_rate_state(year)
        interest_state = None
        interest_unavailable_reason: str | None = None
        if isinstance(interest_result, NoDataSentinel):
            fallbacks.record_vol3_interest_sentinel()
            interest_unavailable_reason = interest_result.reason
            self._log_vol3_sentinel_once_per_year(year, "interest", interest_result.reason)
        else:
            interest_state = interest_result
        national_rate = interest_state.effective_rate if interest_state is not None else None
        national_spread = interest_state.baa_spread if interest_state is not None else None

        fictitious = None
        if services.fictitious_capital_calculator is not None:
            fict_result = services.fictitious_capital_calculator.compute_fictitious_capital(year)
            if isinstance(fict_result, NoDataSentinel):
                fallbacks.record_vol3_fictitious_sentinel()
                self._log_vol3_sentinel_once_per_year(year, "fictitious", fict_result.reason)
            else:
                fictitious = fict_result

        financial_params = NationalFinancialParameters(
            interest_rate_state=interest_state,
            fictitious_capital=fictitious,
        )
        write_national_financial_state_to_graph(graph, financial_params)

        return national_rate, national_spread, fictitious, interest_unavailable_reason
```

> **Do not "simplify" this body.** Every `| None`, every sentinel tally
> (`record_vol3_interest_sentinel` / `record_vol3_fictitious_sentinel`) and every
> `_log_vol3_sentinel_once_per_year` call is load-bearing and already covered by passing tests in
> `tests/unit/economics/tick/test_system.py`. Replacing a `None` with `0.0` reverts commit
> `aedce819` and reds `test_national_interest_sentinel_records_tally_never_a_fabricated_zero`,
> `test_sentinel_reason_is_logged_at_warning`, and
> `test_absent_national_rate_skips_distribution_never_fabricates_zero_interest`.

- [ ] **Step 3b: Update `_compute_financial_layer`'s four existing test call sites**

`_compute_financial_layer` gains a required `graph` parameter, and four tests in
`tests/unit/economics/tick/test_system.py` call it with four positional arguments and no graph
(`system._compute_financial_layer(<counties>, national_params, services, 2041)` — inside
`test_full_financial_layer_at_year_2041_surfaces_all_three_sentinels`,
`test_sentinel_logged_once_per_year_not_once_per_county`,
`test_distinct_reasons_from_different_categories_both_reach_log_same_year`, and
`test_absent_national_rate_skips_distribution_never_fabricates_zero_interest`). Append
`, build_territory_graph()` to each of the four calls — that helper is already imported in this
file. These four call sites did not exist when this plan was written; they arrived with the U2.2/U2.3
sentinel-tally tests. Without this edit, Step 4 cannot pass.

- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_system.py`
Expected: PASS (the whole file — confirms threading `graph` through `_compute_financial_layer`'s production call site and the four test call sites from Step 3b didn't break any of the ~2200 lines of existing pipeline tests). Then run `mise run test:q -- tests/unit/economics/tick/test_financial_integration.py` too: it exercises `_compute_county_financial_state`/`_assess_county_financial_crisis` directly and must still be 21/21, proving U3.2 left the honest-absence contract from `aedce819` intact.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): wire _compute_national_financial_state to publish via graph_bridge

U3.2: NationalFinancialParameters is now instantiated and published under
NATIONAL_FINANCIAL_ATTR every tick the financial layer runs. The 4-tuple
return contract for existing county-level callers is unchanged, including
its float | None honest-absence typing (aedce819); only the graph
parameter is new.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U3.3: Round-trip proof — a CONSEQUENCE-phase System reads it in the same tick

**Files:**
- Create: `tests/unit/economics/tick/test_financial_state_consequence_roundtrip.py`

**Interfaces:**
- Consumes: `NATIONAL_FINANCIAL_ATTR`, `read_national_financial_state_from_graph` (Task U3.1); `TickDynamicsSystem` publishing via `_compute_national_financial_state` (Task U3.2); `MockInterestRateCalculator`, `MockFictitiousCapitalCalculator` (Task U3.2); `_make_services`, `_make_graph_with_state` (`tests/unit/economics/tick/test_system.py`); `SystemBase` (`src/babylon/kernel/system_base.py:58-117`), `TickPartition.CONSEQUENCE` (`src/babylon/kernel/tick_partition.py`)
- Produces: a permanent regression pin proving the U3 acceptance criterion — no new production names for later units to consume.

- [ ] **Step 1: Write the test**

Create `tests/unit/economics/tick/test_financial_state_consequence_roundtrip.py`:

```python
"""Round-trip proof for vol3-money-scissors U3: NATIONAL_FINANCIAL_ATTR is
readable by a CONSEQUENCE-phase System in the same tick TickDynamicsSystem
(MATERIAL_BASE @4.0) publishes it — no WorldState round-trip in between,
matching simulation_engine.run_tick's shared-graph contract (design doc SS3.5).

Feature: 024-capital-volume-iii / vol3-money-scissors U3
"""

from __future__ import annotations

from typing import TYPE_CHECKING, ClassVar

from babylon.domain.economics.tick.graph_bridge import (
    NATIONAL_FINANCIAL_ATTR,
    read_national_financial_state_from_graph,
)
from babylon.domain.economics.tick.system import TickDynamicsSystem
from babylon.domain.economics.tick.types import NationalFinancialParameters
from babylon.engine.context import TickContext
from babylon.kernel.system_base import SystemBase
from babylon.kernel.tick_partition import TickPartition
from tests.unit.economics.tick.conftest import (
    MockFictitiousCapitalCalculator,
    MockInterestRateCalculator,
)
from tests.unit.economics.tick.test_system import _make_graph_with_state, _make_services

if TYPE_CHECKING:
    from babylon.kernel.graph_protocol import GraphProtocol
    from babylon.kernel.services import ServicesProtocol
    from babylon.kernel.system_protocol import ContextType


class _SpyConsequenceSystem(SystemBase):
    """Minimal CONSEQUENCE-phase System that observes the published state.

    Stands in for the real MarketScissorsSystem consumer that U4/U6 wire —
    this class exists only to prove the graph key is readable at a
    CONSEQUENCE-phase position (@17.9, near MarketScissorsSystem's @17.8)
    within the same tick TickDynamicsSystem (@4.0, MATERIAL_BASE) writes it.
    """

    name: ClassVar[str] = "spy_consequence"
    partition: ClassVar[TickPartition] = TickPartition.CONSEQUENCE
    position: ClassVar[float] = 17.9

    def __init__(self) -> None:
        self.observed: NationalFinancialParameters | None = None

    def step(
        self,
        graph: GraphProtocol,
        services: ServicesProtocol,
        context: ContextType,
    ) -> None:
        """Read NATIONAL_FINANCIAL_ATTR from the shared graph."""
        self.observed = read_national_financial_state_from_graph(graph)


def test_consequence_phase_system_reads_financial_state_same_tick() -> None:
    """A CONSEQUENCE System reads what MATERIAL_BASE published, same tick."""
    services = _make_services(
        interest_calculator=MockInterestRateCalculator(
            base_rate=0.25,
            treasury_10y=2.27,
            baa_spread=2.64,
        ),
        fictitious_capital_calculator=MockFictitiousCapitalCalculator(
            government_debt=18e12,
            corporate_equity=20e12,
            corporate_debt=8e12,
            household_debt=14e12,
        ),
    )
    graph = _make_graph_with_state(year=2015)
    context = TickContext(tick=52)

    material_base_system = TickDynamicsSystem()
    consequence_system = _SpyConsequenceSystem()

    assert NATIONAL_FINANCIAL_ATTR not in graph.graph

    # MATERIAL_BASE (@4.0) runs first — this is where NationalFinancialParameters
    # is instantiated and published (U3.2).
    material_base_system.step(graph, services, context)
    assert NATIONAL_FINANCIAL_ATTR in graph.graph

    # CONSEQUENCE (@17.9) runs later in the SAME tick, on the SAME graph
    # object, with no WorldState round-trip between them.
    consequence_system.step(graph, services, context)

    assert consequence_system.observed is not None
    assert consequence_system.observed.fictitious_capital is not None
    assert consequence_system.observed.fictitious_capital.total_claims == 60e12
    assert consequence_system.observed.interest_rate_state is not None
    assert consequence_system.observed.interest_rate_state.base_rate == 0.25
```

- [ ] **Step 2: Run test to verify it fails**

**Step 2a:** Comment out the line `write_national_financial_state_to_graph(graph, financial_params)` in `src/babylon/domain/economics/tick/system/__init__.py` (inside `_compute_national_financial_state`).

**Step 2b:** Run: `mise run test:q -- tests/unit/economics/tick/test_financial_state_consequence_roundtrip.py`
Expected: FAIL with `AssertionError: assert 'national_financial' in {}` at the post-`material_base_system.step()` assertion — proving the CONSEQUENCE-phase read genuinely depends on U3.2's publish rather than incidental state. If this fails to fail, the defect is in U3.1 or U3.2, not here — do not "fix" this test to make it pass; go back and fix the wiring.

- [ ] **Step 3: Restore the implementation**
Run: `git checkout -- src/babylon/domain/economics/tick/system/__init__.py`
Expected: GREEN — Tasks U3.1 and U3.2 already implemented every production line this test exercises (`NATIONAL_FINANCIAL_ATTR`, both graph_bridge functions, and `_compute_national_financial_state`'s publish call), so restoring the commented-out line makes the round-trip contract hold end-to-end again; this task adds zero new production code. Re-read `_SpyConsequenceSystem.step()` and confirm it reads via the public `read_national_financial_state_from_graph` API (not a private/internal attribute), so the proof is representative of how a real future consumer (U4's anchor, U5/U6's oppositions) will read it.

- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_financial_state_consequence_roundtrip.py`
Expected: PASS (unchanged from Step 2 — confirms nothing regressed between steps).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(economics): pin CONSEQUENCE-phase cross-System same-tick read of national financial state

U3.3: regression guard for the design doc's U3 acceptance criterion —
a second SystemBase at TickPartition.CONSEQUENCE reads what
TickDynamicsSystem (MATERIAL_BASE) published, on the same graph object,
same tick, no WorldState round-trip. No production code changed.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U3.4: Compute and publish `CreditState` (the `credit` opposition's only input)

**Files:**
- Modify: `src/babylon/domain/economics/tick/system/__init__.py` (`_compute_national_financial_state`)
- Modify: `src/babylon/domain/economics/factory.py` (`create_financial_services` return dict)
- Modify: `src/babylon/kernel/services.py`, `src/babylon/engine/services.py` (new service key)
- Test: `tests/unit/economics/tick/test_system.py` (append to `TestComputeNationalFinancialState`;
  add `from tests.unit.economics.credit.conftest import MockCreditAggregateSource` to the
  import block — that fixture lives in a sibling package's `conftest.py`, and pytest conftests
  are not cross-package importable by name, so it needs an explicit module-path import)

**Interfaces:**
- Consumes: `CreditState` (`babylon.domain.economics.credit.types:148`, fields `year`,
  `total_credit`, `credit_expansion_rate`, `default_rate`, `spread_to_treasuries`,
  `phase`; computed `credit_fragility = default_rate * spread_to_treasuries`);
  `CreditAggregateSource.get_total_credit` (`credit/data_sources.py:57`, wired as
  `FredCreditAggregateAdapter` over `TCMDO`);
  `services.defines.capital_vol3.default_rate_estimate` (U2.3);
  `InterestRateState.baa_spread`.
- Produces: `NationalFinancialParameters.credit_state` populated — the ONLY producer of the
  `credit_fragility` value `ContradictionSystem._credit_fragility` (U5.7) reads.

- [ ] **Step 0: Read the real API before writing anything (MANDATORY)**
Three plan auditors proposed three different producers for `CreditState`
(`credit_cycle_calculator.compute_credit_state`,
`credit_aggregate_source.get_total_credit`, `credit_cycle_detector.detect_credit_state`).
Only one exists. Run:
```bash
rg -n "class CreditState" -A 30 src/babylon/domain/economics/credit/types.py
rg -n "class DefaultCreditCycleDetector" -A 40 src/babylon/domain/economics/credit/
rg -n "class CreditAggregateSource\|FredCreditAggregateAdapter" -A 20 src/babylon/domain/economics/credit/data_sources.py
rg -n "credit_cycle\|credit_agg" src/babylon/domain/economics/factory.py
```
Adjust Step 3's helper body to the REAL signatures. **Do not adjust the Step 1
test's contract** — `credit_fragility == default_rate_estimate * baa_spread` is
what the `credit` opposition measures, whatever plumbing supplies the inputs.
- [ ] **Step 1: Write the failing test**
`MockCreditAggregateSource` lives in `tests/unit/economics/credit/conftest.py`, a
different package from this file — conftest fixtures are not cross-package
importable by name. Add the explicit module-path import to this file's import
block:
```python
from tests.unit.economics.credit.conftest import MockCreditAggregateSource
```
Then append these two tests to `TestComputeNationalFinancialState`:
```python
    def test_publishes_credit_state_with_a_real_fragility_index(self) -> None:
        """U3.4: the `credit` opposition (U5.2) measures default_rate * spread.
        Nothing in the codebase constructed a CreditState before this task, so
        GraphInputs.credit_fragility was permanently None and the declared
        credit->financial transforms edge permanently demoted `financial`."""
        services = _make_services(
            interest_calculator=MockInterestRateCalculator(
                base_rate=0.25, treasury_10y=2.27, baa_spread=2.64
            ),
            fictitious_capital_calculator=MockFictitiousCapitalCalculator(),
            credit_aggregate_source=MockCreditAggregateSource(
                data={2015: (60e12, 18e12, 20e12)}
            ),
        )
        graph = build_territory_graph()
        system = TickDynamicsSystem()

        system._compute_national_financial_state(services, 2015, graph)

        published = read_national_financial_state_from_graph(graph)
        assert published is not None
        assert published.credit_state is not None
        assert published.credit_state.spread_to_treasuries == pytest.approx(2.64)
        assert published.credit_state.default_rate == pytest.approx(
            services.defines.capital_vol3.default_rate_estimate
        )
        assert published.credit_state.credit_fragility == pytest.approx(
            services.defines.capital_vol3.default_rate_estimate * 2.64
        )

    def test_credit_state_is_none_without_an_interest_state(self) -> None:
        """III.11: no spread observable => no credit state, never a zero one."""
        services = _make_services(
            interest_calculator=MockInterestRateCalculator(force_sentinel=True),
        )
        graph = build_territory_graph()
        TickDynamicsSystem()._compute_national_financial_state(services, 2015, graph)
        published = read_national_financial_state_from_graph(graph)
        assert published is not None
        assert published.credit_state is None
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/tick/test_system.py::TestComputeNationalFinancialState`
Expected: 1 failed, the rest passed. `test_publishes_credit_state_with_a_real_fragility_index`
fails with `assert None is not None` (`published.credit_state` is never populated).
`test_credit_state_is_none_without_an_interest_state` passes already — `credit_state` defaults
to `None` before Step 3's implementation exists, which is what that test asserts — so it is not
part of this task's red phase.
- [ ] **Step 3: Write minimal implementation**

Add `CreditState` **and `InterestRateState`** to the `credit.types` import block in
`src/babylon/domain/economics/tick/system/__init__.py` — at HEAD that block imports only
`FictitiousCapitalStock`, and the helper below annotates an `InterestRateState | None` parameter,
so mypy reds without it. Then insert this helper immediately before
`_compute_national_financial_state` (i.e. after `_log_vol3_sentinel_once_per_year`).

This task is unaffected by the method's 4-tuple return contract: it neither calls nor unpacks
`_compute_national_financial_state`, it only reads the `interest_state` local and extends the
`NationalFinancialParameters(...)` construction that U3.2 adds.
```python
    @staticmethod
    def _build_credit_state(
        services: ServicesProtocol,
        year: int,
        interest_state: InterestRateState | None,
    ) -> CreditState | None:
        """Assemble the national credit state from the FRED credit aggregate.

        The ONLY producer of ``CreditState`` in the codebase (vol3-money-scissors
        U3.4). ``credit_fragility = default_rate * spread_to_treasuries`` is the
        measure the ``credit`` opposition (accommodation ⟷ fragility) is bound
        to; without this the opposition reads absent forever and the declared
        ``credit -> financial`` transforms edge permanently demotes ``financial``.

        Returns:
            ``None`` — never a fabricated zero (III.11) — when no interest state
            supplies the spread, or when no credit aggregate source supplies
            total credit for ``year``.
        """
        if interest_state is None:
            return None
        source = getattr(services, "credit_aggregate_source", None)
        if source is None:
            return None
        total_credit = source.get_total_credit(year)
        if total_credit is None:
            return None
        return CreditState(
            year=year,
            total_credit=total_credit,
            default_rate=services.defines.capital_vol3.default_rate_estimate,
            spread_to_treasuries=interest_state.baa_spread,
        )
```
Then, in `_compute_national_financial_state`, replace:
```python
        financial_params = NationalFinancialParameters(
            interest_rate_state=interest_state,
            fictitious_capital=fictitious,
        )
```
with:
```python
        financial_params = NationalFinancialParameters(
            interest_rate_state=interest_state,
            credit_state=self._build_credit_state(services, year, interest_state),
            fictitious_capital=fictitious,
        )
```
Finally, expose the adapter the helper reads. In
`src/babylon/domain/economics/factory.py`'s `create_financial_services`, add
`"credit_aggregate_source": credit_agg,` to the returned dict (the
`FredCreditAggregateAdapter` instance already built for
`DefaultFictitiousCapitalCalculator`), and add the matching
`credit_aggregate_source: Any` field to `src/babylon/kernel/services.py` beside
`credit_cycle_detector`, plus `credit_aggregate_source: Any = field(default=None)`,
the `create` kwarg, and the pass-through in `src/babylon/engine/services.py`.
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/tick/test_system.py tests/unit/economics/tick/test_graph_bridge.py`
Expected: PASS.
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): compute and publish CreditState (the credit opposition's only input)

CreditState had zero production constructors — the model existed, nothing
built it. GraphInputs.credit_fragility was therefore permanently None, the
credit opposition permanently read (0,0), and the declared credit->financial
transforms edge permanently demoted \`financial\` from principal ranking.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.1: `fictitious_anchor` — honest absence when the stock is missing

**Files:**
- Create: `src/babylon/domain/economics/monetary/anchor.py`
- Test: `tests/unit/economics/monetary/test_anchor.py`

**Interfaces:**
- Consumes: `NoDataSentinel(fips: str, year: int, reason: str)` from `babylon.domain.economics.tensor` (falsy marker, `__slots__ = ("fips", "year", "reason")`); `FictitiousCapitalStock` from `babylon.domain.economics.credit.types` (frozen; fields `year` (ge=2007, le=2040), `government_debt`, `corporate_equity`, `corporate_debt`, `household_debt`, `derivatives_notional`; computed `total_claims`; method `ratio_to_real(real_gdp: float) -> float` returning `inf` when `real_gdp <= 0`).
- Produces: `babylon.domain.economics.monetary.anchor.fictitious_anchor(stock: FictitiousCapitalStock | None, real_output: float | None) -> float | NoDataSentinel`; module constants `NATIONAL_FIPS: Final[str] = "USA"` and `UNKNOWN_YEAR: Final[int] = 0`. U6 consumes `fictitious_anchor` to pull `fictitious_log` toward the real ratio.
- Deferred: `src/babylon/domain/economics/monetary/__init__.py` is NOT touched by this task — the anchor is imported by submodule path until U4.7 exports it. Do not add the export early; U4.7's red phase depends on `fictitious_anchor` being absent from `monetary.__all__`.
- Ordering note: U3 is a prerequisite of U6.8, not of this task — the anchor is a pure function of an already-resolved stock object and never reads `NATIONAL_FINANCIAL_ATTR`.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the monetary anchor (Vol III calibrates; the scissors integrates).

Design: docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md §3.3 (D1).

Contract clause 1: any absent input yields a NoDataSentinel carrying a SPECIFIC
reason. A zero is never fabricated (Constitution III.11).
"""

from __future__ import annotations

import pytest

from babylon.domain.economics.credit.types import FictitiousCapitalStock
from babylon.domain.economics.monetary.anchor import (
    NATIONAL_FIPS,
    UNKNOWN_YEAR,
    fictitious_anchor,
)
from babylon.domain.economics.tensor import NoDataSentinel


def _stock(year: int = 2020) -> FictitiousCapitalStock:
    """Build a stock whose total_claims is exactly 100.0."""
    return FictitiousCapitalStock(
        year=year,
        government_debt=20.0,
        corporate_equity=40.0,
        corporate_debt=10.0,
        household_debt=30.0,
    )


@pytest.mark.unit
class TestFictitiousAnchorAbsence:
    """fictitious_anchor returns an honest sentinel when an input is absent."""

    def test_absent_stock_returns_sentinel(self) -> None:
        """No published FictitiousCapitalStock: sentinel, not a zero."""
        result = fictitious_anchor(None, 50.0)
        assert isinstance(result, NoDataSentinel)
        assert result.fips == NATIONAL_FIPS
        assert result.year == UNKNOWN_YEAR
        assert "fictitious_anchor" in result.reason
        assert "FictitiousCapitalStock" in result.reason

    def test_absent_stock_sentinel_is_falsy(self) -> None:
        """The sentinel supports the walrus/falsy consumer pattern."""
        assert not fictitious_anchor(None, 50.0)

    def test_absent_real_output_returns_sentinel_carrying_the_stock_year(self) -> None:
        """The stock exists but no real output observable: sentinel with the year."""
        result = fictitious_anchor(_stock(2020), None)
        assert isinstance(result, NoDataSentinel)
        assert result.fips == NATIONAL_FIPS
        assert result.year == 2020
        assert "real output" in result.reason

    def test_absence_never_returns_a_float(self) -> None:
        """Absence is never expressed as a number."""
        assert not isinstance(fictitious_anchor(None, None), float)
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: FAIL — collection error `ModuleNotFoundError: No module named 'babylon.domain.economics.monetary.anchor'`
- [ ] **Step 3: Write minimal implementation**
Create `src/babylon/domain/economics/monetary/anchor.py`:
```python
"""Monetary anchor: real federal data calibrating the scissors oscillator.

Design: ``docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md`` §3.3,
owner decision D1 — *Volume III calibrates; the scissors integrates.*

Where real data exists (2010-2024) these functions expose the log-space target
and the serviceability tightener the oscillator is pulled toward. Past the data
horizon — roughly 85% of a 2010-2109 campaign — they return
:class:`~babylon.domain.economics.tensor.NoDataSentinel` and the oscillator
continues unchanged on its own endogenous dynamics. **Absence is the normal
steady state, not an error path** (Constitution III.11: no fabricated zeros, no
substituted defaults).

Pure functions of their arguments: no RNG, no wall clock, no I/O. This module
lives in ``domain/`` and imports nothing from ``engine/``; the engine reads it.
"""

from __future__ import annotations

import math
from typing import Final

from babylon.domain.economics.credit.types import FictitiousCapitalStock
from babylon.domain.economics.tensor import NoDataSentinel

NATIONAL_FIPS: Final[str] = "USA"
"""FIPS placeholder for national-scope sentinels.

Matches the convention already used by
:class:`~babylon.domain.economics.monetary.converter.DefaultValueBasisConverter`.
"""

UNKNOWN_YEAR: Final[int] = 0
"""Year marker used when the absent input is the thing that carries the year.

Not a data value. When the stock or distribution itself is ``None`` there is no
year to report, and fabricating a plausible one would violate III.11.
"""


def fictitious_anchor(
    stock: FictitiousCapitalStock | None,
    real_output: float | None,
) -> float | NoDataSentinel:
    """Log-space target the fictitious oscillator is pulled toward.

    ``log(stock.ratio_to_real(real_output))`` — the real financialization ratio
    expressed in the same log space the scissors integrates in.

    :param stock: Published national fictitious capital stock, or ``None`` when
        no stock reached the graph this tick (the normal case past 2024).
    :param real_output: Real output the claims are drawn against, or ``None``
        when no output observable exists.
    :returns: The log-space anchor, or a :class:`NoDataSentinel` naming the
        specific absence. Never a fabricated zero.
    """
    if stock is None:
        return NoDataSentinel(
            fips=NATIONAL_FIPS,
            year=UNKNOWN_YEAR,
            reason="fictitious_anchor: no FictitiousCapitalStock published this tick",
        )
    if real_output is None:
        return NoDataSentinel(
            fips=NATIONAL_FIPS,
            year=stock.year,
            reason=f"fictitious_anchor: no real output observable for {stock.year}",
        )
    return math.log(stock.ratio_to_real(real_output))
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (4 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(monetary): fictitious_anchor honest-absence contract

Design 2026-07-18 vol3-money-scissors §3.3 clause 1: an absent
FictitiousCapitalStock or real output yields a NoDataSentinel with a
specific reason, never a fabricated zero.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.2: `fictitious_anchor` — the present path returns the real log ratio

**Files:**
- Modify: `src/babylon/domain/economics/monetary/anchor.py` (no signature change; add nothing — this task proves the already-written present path)
- Test: `tests/unit/economics/monetary/test_anchor.py` (append)

**Interfaces:**
- Consumes: `fictitious_anchor` from U4.1; `FictitiousCapitalStock.total_claims` (`government_debt + corporate_equity + corporate_debt + household_debt`; `derivatives_notional` excluded).
- Produces: nothing new — pins the numeric contract U6's `anchor_pull` term depends on.

- [ ] **Step 1: Write the failing test**
Append to `tests/unit/economics/monetary/test_anchor.py`:
```python
@pytest.mark.unit
class TestFictitiousAnchorPresent:
    """fictitious_anchor computes log(total_claims / real_output) when data exists."""

    def test_returns_log_of_the_real_ratio(self) -> None:
        """total_claims 100.0 over real output 50.0 is ln(2.0)."""
        result = fictitious_anchor(_stock(2020), 50.0)
        assert result == pytest.approx(math.log(2.0))

    def test_par_claims_anchor_at_zero(self) -> None:
        """Claims exactly equal to real output anchor at log-ratio 0.0."""
        result = fictitious_anchor(_stock(2020), 100.0)
        assert result == pytest.approx(0.0)

    def test_undervalued_claims_anchor_below_zero(self) -> None:
        """Claims below real output give a negative log anchor."""
        result = fictitious_anchor(_stock(2020), 200.0)
        assert isinstance(result, float)
        assert result < 0.0

    def test_derivatives_are_excluded_from_the_anchor(self) -> None:
        """derivatives_notional is tracked but never enters total_claims."""
        with_derivatives = FictitiousCapitalStock(
            year=2020,
            government_debt=20.0,
            corporate_equity=40.0,
            corporate_debt=10.0,
            household_debt=30.0,
            derivatives_notional=900.0,
        )
        assert fictitious_anchor(with_derivatives, 50.0) == pytest.approx(
            fictitious_anchor(_stock(2020), 50.0)
        )
```
Also add `import math` to the test file's import block, directly above `import pytest`.
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py::TestFictitiousAnchorPresent`
Add `import math` to the test file's imports first (a missing test-file import is a broken test, not a red phase). **Step 2a** replace `fictitious_anchor`'s final statement with `return 0.0`; **Step 2b** run `…::TestFictitiousAnchorPresent`, expect FAIL — 3 of 4 (`test_returns_log_of_the_real_ratio` gets `0.0` not `ln(2)`; `test_undervalued_claims_anchor_below_zero` gets `0.0` not `< 0`; `test_derivatives_are_excluded_from_the_anchor` passes vacuously); **Step 3** `git checkout -- src/babylon/domain/economics/monetary/anchor.py`.
- [ ] **Step 3: Write minimal implementation**
No production change required — the U4.1 body already computes `math.log(stock.ratio_to_real(real_output))`. Verify by reading `src/babylon/domain/economics/monetary/anchor.py` and confirming the final statement is:
```python
    return math.log(stock.ratio_to_real(real_output))
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (8 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(monetary): pin fictitious_anchor present-path log ratio

Anchors the numeric contract U6's anchor_pull term reads, including the
exclusion of derivatives_notional from total_claims.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.3: `fictitious_anchor` — degenerate ratios are absence, not infinity

**Files:**
- Modify: `src/babylon/domain/economics/monetary/anchor.py` (the `fictitious_anchor` body, between the `real_output is None` guard and the `return math.log(...)`)
- Test: `tests/unit/economics/monetary/test_anchor.py` (append)

**Interfaces:**
- Consumes: `FictitiousCapitalStock.ratio_to_real`, which returns `float("inf")` when `real_gdp <= 0.0` — an infinity that would poison the oscillator, and `math.log(0.0)` which raises `ValueError`.
- Produces: `fictitious_anchor` now guarantees a finite float or a sentinel — never `inf`, `-inf`, `nan`, or a raised exception.

- [ ] **Step 1: Write the failing test**
Append to `tests/unit/economics/monetary/test_anchor.py`:
```python
@pytest.mark.unit
class TestFictitiousAnchorDegenerate:
    """Degenerate ratios degrade to sentinels rather than infinities or raises."""

    def test_zero_real_output_returns_sentinel(self) -> None:
        """ratio_to_real returns inf at zero output; the anchor must not."""
        result = fictitious_anchor(_stock(2020), 0.0)
        assert isinstance(result, NoDataSentinel)
        assert result.year == 2020
        assert "non-positive real output" in result.reason

    def test_negative_real_output_returns_sentinel(self) -> None:
        """A negative output observable is absence, not a value."""
        result = fictitious_anchor(_stock(2020), -25.0)
        assert isinstance(result, NoDataSentinel)
        assert "non-positive real output" in result.reason

    def test_zero_total_claims_returns_sentinel(self) -> None:
        """log(0) is undefined; zero claims is absence of a log anchor."""
        empty = FictitiousCapitalStock(
            year=2020,
            government_debt=0.0,
            corporate_equity=0.0,
            corporate_debt=0.0,
            household_debt=0.0,
        )
        result = fictitious_anchor(empty, 50.0)
        assert isinstance(result, NoDataSentinel)
        assert result.year == 2020
        assert "zero total claims" in result.reason

    def test_present_anchor_is_always_finite(self) -> None:
        """No degenerate input escapes as inf or nan."""
        result = fictitious_anchor(_stock(2020), 50.0)
        assert isinstance(result, float)
        assert math.isfinite(result)
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py::TestFictitiousAnchorDegenerate`
Expected: FAIL — `test_zero_real_output_returns_sentinel` gets `inf` back from `math.log(inf)` instead of a `NoDataSentinel`, and `test_zero_total_claims_returns_sentinel` raises `ValueError: math domain error`.
- [ ] **Step 3: Write minimal implementation**
In `src/babylon/domain/economics/monetary/anchor.py`, replace the final statement of `fictitious_anchor`:
```python
    return math.log(stock.ratio_to_real(real_output))
```
with:
```python
    if real_output <= 0.0:
        return NoDataSentinel(
            fips=NATIONAL_FIPS,
            year=stock.year,
            reason=(
                f"fictitious_anchor: non-positive real output ({real_output}) "
                f"for {stock.year}; log-ratio undefined"
            ),
        )
    ratio = stock.ratio_to_real(real_output)
    if ratio <= 0.0:
        return NoDataSentinel(
            fips=NATIONAL_FIPS,
            year=stock.year,
            reason=(
                f"fictitious_anchor: zero total claims for {stock.year}; "
                "log-ratio undefined"
            ),
        )
    return math.log(ratio)
```
and extend the `:returns:` line of the docstring to read:
```python
    :returns: The finite log-space anchor, or a :class:`NoDataSentinel` naming
        the specific absence. Never a fabricated zero, never ``inf``/``nan``.
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (12 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(monetary): fictitious_anchor degrades degenerate ratios to sentinels

ratio_to_real returns inf at non-positive real output and zero claims make
log undefined; both are absence, not values. Guarantees a finite float or a
NoDataSentinel so the oscillator can never be fed an infinity.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.4: `serviceability_anchor` — honest absence

**Files:**
- Modify: `src/babylon/domain/economics/monetary/anchor.py` (append a new function after `fictitious_anchor`)
- Test: `tests/unit/economics/monetary/test_anchor.py` (append)

**Interfaces:**
- Consumes: `SurplusValueDistribution` from `babylon.domain.economics.distribution.types` — frozen; fields `fips_code` (exactly 5 chars), `year` (ge=2007, le=2040), `total_surplus_produced` (ge=0), `interest_payments` (ge=0), `ground_rent` (ge=0), `taxes_on_surplus` (ge=0); computed `profit_of_enterprise = s - i - r - t` (may be negative) and `financialization_share` (which **silently returns 0.0 when `total_surplus_produced == 0.0`** — the reason this function must guard the denominator itself rather than delegate).
- Produces: `serviceability_anchor(distribution: SurplusValueDistribution | None) -> float | NoDataSentinel`. U6.6 consumes it via `market_scissors._national_serviceability`, which aggregates the published county distributions as a RATIO OF SUMS (Σi / Σs — never a mean of per-county burdens, which would be the intensive-aggregation class U7.6 catches) and applies this function to the aggregate, so the honest-absence contract (§3.3 clause 1) is the single source of the absent/present decision.

- [ ] **Step 1: Write the failing test**
Append to `tests/unit/economics/monetary/test_anchor.py` (and add `SurplusValueDistribution` plus `serviceability_anchor` to the import block):
```python
def _distribution(
    *,
    surplus: float = 100.0,
    interest: float = 25.0,
    fips: str = "26163",
    year: int = 2020,
) -> SurplusValueDistribution:
    """Build a distribution with the interest claim under test."""
    return SurplusValueDistribution(
        fips_code=fips,
        year=year,
        total_surplus_produced=surplus,
        interest_payments=interest,
        ground_rent=10.0,
        taxes_on_surplus=15.0,
    )


@pytest.mark.unit
class TestServiceabilityAnchorAbsence:
    """serviceability_anchor returns an honest sentinel when input is absent."""

    def test_absent_distribution_returns_sentinel(self) -> None:
        """No SurplusValueDistribution computed: sentinel, not a zero burden."""
        result = serviceability_anchor(None)
        assert isinstance(result, NoDataSentinel)
        assert result.fips == NATIONAL_FIPS
        assert result.year == UNKNOWN_YEAR
        assert "SurplusValueDistribution" in result.reason

    def test_zero_surplus_returns_sentinel_not_the_computed_field_zero(self) -> None:
        """financialization_share silently returns 0.0 at zero surplus; we must not."""
        zero_surplus = _distribution(surplus=0.0, interest=0.0)
        assert zero_surplus.financialization_share == 0.0
        result = serviceability_anchor(zero_surplus)
        assert isinstance(result, NoDataSentinel)
        assert result.fips == "26163"
        assert result.year == 2020
        assert "zero surplus" in result.reason

    def test_absence_is_falsy_and_never_a_float(self) -> None:
        """The sentinel supports the walrus pattern and is not a number."""
        result = serviceability_anchor(None)
        assert not result
        assert not isinstance(result, float)
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py::TestServiceabilityAnchorAbsence`
Expected: FAIL — collection error `ImportError: cannot import name 'serviceability_anchor' from 'babylon.domain.economics.monetary.anchor'`
- [ ] **Step 3: Write minimal implementation**
Append to `src/babylon/domain/economics/monetary/anchor.py`, and add this line to the import block, directly below the existing `from babylon.domain.economics.credit.types import FictitiousCapitalStock` line (`distribution` sorts after `credit`; Ruff's isort will confirm):
`from babylon.domain.economics.distribution.types import SurplusValueDistribution`
```python
def serviceability_anchor(
    distribution: SurplusValueDistribution | None,
) -> float | NoDataSentinel:
    """Real interest burden ``i / s`` — how much surplus is already spoken for.

    Tightens :func:`babylon.formulas.market.calculate_serviceable_divergence`
    beyond its existing profit-rate slope: a financialised county services a
    smaller claims structure at the same rate of profit. Vol. III part 3 (the
    falling rate) meeting part 5 (fictitious capital).

    Guards the denominator itself rather than delegating to
    :attr:`SurplusValueDistribution.financialization_share`, which returns a
    silent ``0.0`` at zero surplus — indistinguishable from a county that
    genuinely pays no interest (Constitution III.11).

    :param distribution: Published surplus distribution, or ``None`` when no
        distribution was computed this tick (the normal case past 2024).
    :returns: The interest burden as a fraction of surplus produced, or a
        :class:`NoDataSentinel` naming the specific absence.
    """
    if distribution is None:
        return NoDataSentinel(
            fips=NATIONAL_FIPS,
            year=UNKNOWN_YEAR,
            reason="serviceability_anchor: no SurplusValueDistribution computed this tick",
        )
    if distribution.total_surplus_produced <= 0.0:
        return NoDataSentinel(
            fips=distribution.fips_code,
            year=distribution.year,
            reason=(
                f"serviceability_anchor: zero surplus produced in "
                f"{distribution.fips_code} {distribution.year}; "
                "interest burden undefined"
            ),
        )
    return distribution.interest_payments / distribution.total_surplus_produced
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (15 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(monetary): serviceability_anchor honest-absence contract

Guards the denominator directly rather than delegating to
financialization_share, whose silent 0.0 at zero surplus is
indistinguishable from a county that genuinely pays no interest.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.5: `serviceability_anchor` — the present path and the overclaim case

**Files:**
- Modify: `src/babylon/domain/economics/monetary/anchor.py` (no change expected — this task proves the U4.4 body)
- Test: `tests/unit/economics/monetary/test_anchor.py` (append)

**Interfaces:**
- Consumes: `serviceability_anchor` from U4.4.
- Produces: nothing new — pins the numeric contract, including that a burden above 1.0 is a legitimate reading (claims exceeding surplus is the debt-spiral condition, not an error).

- [ ] **Step 1: Write the failing test**
Append to `tests/unit/economics/monetary/test_anchor.py`:
```python
@pytest.mark.unit
class TestServiceabilityAnchorPresent:
    """serviceability_anchor computes interest_payments / total_surplus_produced."""

    def test_returns_the_interest_burden(self) -> None:
        """25.0 interest against 100.0 surplus is a burden of 0.25."""
        assert serviceability_anchor(_distribution()) == pytest.approx(0.25)

    def test_zero_interest_with_positive_surplus_is_a_real_zero(self) -> None:
        """A county that genuinely pays no interest reads 0.0, not a sentinel."""
        result = serviceability_anchor(_distribution(interest=0.0))
        assert isinstance(result, float)
        assert result == pytest.approx(0.0)

    def test_burden_above_one_is_a_legitimate_reading(self) -> None:
        """Interest exceeding surplus is the debt-spiral condition, not an error."""
        overclaimed = _distribution(surplus=50.0, interest=75.0)
        assert overclaimed.profit_of_enterprise < 0.0
        result = serviceability_anchor(overclaimed)
        assert isinstance(result, float)
        assert result == pytest.approx(1.5)

    def test_matches_financialization_share_when_surplus_is_positive(self) -> None:
        """Where the computed field is well-defined the anchor agrees with it."""
        distribution = _distribution()
        assert serviceability_anchor(distribution) == pytest.approx(
            distribution.financialization_share
        )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py::TestServiceabilityAnchorPresent`
**Step 2a** replace `serviceability_anchor`'s final statement with `return 0.0`; **Step 2b** run `…::TestServiceabilityAnchorPresent`, expect FAIL — 3 of 4 (`test_returns_the_interest_burden` gets `0.0` not `0.25`; `test_burden_above_one_is_a_legitimate_reading` gets `0.0` not `1.5`; `test_matches_financialization_share_when_surplus_is_positive` gets `0.0` not `0.25`); **Step 3** `git checkout --`.
- [ ] **Step 3: Write minimal implementation**
No production change. Confirm the final statement of `serviceability_anchor` in `src/babylon/domain/economics/monetary/anchor.py` reads:
```python
    return distribution.interest_payments / distribution.total_surplus_produced
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (19 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(monetary): pin serviceability_anchor present path

Distinguishes a genuine zero burden (positive surplus, no interest) from
absence, and admits a burden above 1.0 as the debt-spiral reading.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.6: Property tests — absence is the default-tested path

**Files:**
- Create: `tests/property/invariants/test_monetary_anchor_absence.py`

**Interfaces:**
- Consumes: `fictitious_anchor`, `serviceability_anchor`, `NATIONAL_FIPS`, `UNKNOWN_YEAR` from `babylon.domain.economics.monetary.anchor`.
- Produces: the contract-clause-2 gate. Hypothesis is already a dependency (`pyproject.toml:119`, `hypothesis = "^6.149.0"`); `tests/property/conftest.py` registers the `dev`/`ci`/`nightly` profiles, and `[tool.hypothesis]` in `pyproject.toml` sets `deadline = 500`, `max_examples = 100`. House style: `@pytest.mark.property` plus explicit `@settings(...)` on each test.

- [ ] **Step 1: Write the failing test**
```python
"""Property laws for the monetary anchor's honest-absence contract.

Design: docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md §3.3.

Contract clause 2 — absence is the NORMAL steady state, not an error path. Real
federal coverage runs 2010-2024; a canonical campaign runs 2010-2109, so the
anchor reads absent for roughly 85% of it. These are laws over the whole input
space rather than examples, because the absent branch is the default branch.

Contract clause 3 — pure, deterministic, no RNG, no clock, no I/O.
"""

from __future__ import annotations

import math

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

from babylon.domain.economics.credit.types import FictitiousCapitalStock
from babylon.domain.economics.distribution.types import SurplusValueDistribution
from babylon.domain.economics.monetary.anchor import (
    NATIONAL_FIPS,
    UNKNOWN_YEAR,
    fictitious_anchor,
    serviceability_anchor,
)
from babylon.domain.economics.tensor import NoDataSentinel

_MONEY = st.floats(
    min_value=0.0,
    max_value=1e12,
    allow_nan=False,
    allow_infinity=False,
)
_YEARS = st.integers(min_value=2010, max_value=2040)
_FIPS = st.integers(min_value=0, max_value=99999).map(lambda n: f"{n:05d}")


@st.composite
def _stocks(draw: st.DrawFn) -> FictitiousCapitalStock:
    """Any valid FictitiousCapitalStock."""
    return FictitiousCapitalStock(
        year=draw(_YEARS),
        government_debt=draw(_MONEY),
        corporate_equity=draw(_MONEY),
        corporate_debt=draw(_MONEY),
        household_debt=draw(_MONEY),
        derivatives_notional=draw(_MONEY),
    )


@st.composite
def _distributions(draw: st.DrawFn) -> SurplusValueDistribution:
    """Any valid SurplusValueDistribution (claims may exceed surplus)."""
    return SurplusValueDistribution(
        fips_code=draw(_FIPS),
        year=draw(_YEARS),
        total_surplus_produced=draw(_MONEY),
        interest_payments=draw(_MONEY),
        ground_rent=draw(_MONEY),
        taxes_on_surplus=draw(_MONEY),
    )


@pytest.mark.property
@given(real_output=st.one_of(st.none(), _MONEY))
@settings(max_examples=200, deadline=1000)
def test_absent_stock_is_always_an_honest_sentinel(real_output: float | None) -> None:
    """No stock: sentinel with a non-empty reason, whatever the other input."""
    result = fictitious_anchor(None, real_output)
    assert isinstance(result, NoDataSentinel)
    assert not isinstance(result, float)
    assert result.fips == NATIONAL_FIPS
    assert result.year == UNKNOWN_YEAR
    assert result.reason.strip() != ""


@pytest.mark.property
@given(stock=_stocks())
@settings(max_examples=200, deadline=1000)
def test_absent_real_output_is_always_an_honest_sentinel(
    stock: FictitiousCapitalStock,
) -> None:
    """No real output: sentinel carrying the stock's own year."""
    result = fictitious_anchor(stock, None)
    assert isinstance(result, NoDataSentinel)
    assert result.year == stock.year
    assert result.reason.strip() != ""


@pytest.mark.property
@given(stock=_stocks(), real_output=st.one_of(st.none(), _MONEY))
@settings(max_examples=300, deadline=1000)
def test_fictitious_anchor_is_finite_or_absent_never_between(
    stock: FictitiousCapitalStock,
    real_output: float | None,
) -> None:
    """Every outcome is either a finite float or a reasoned sentinel."""
    result = fictitious_anchor(stock, real_output)
    if isinstance(result, NoDataSentinel):
        assert result.reason.strip() != ""
    else:
        assert math.isfinite(result)


@pytest.mark.property
@given(distribution=st.one_of(st.none(), _distributions()))
@settings(max_examples=300, deadline=1000)
def test_serviceability_anchor_is_finite_or_absent_never_between(
    distribution: SurplusValueDistribution | None,
) -> None:
    """Every outcome is either a finite non-negative float or a reasoned sentinel."""
    result = serviceability_anchor(distribution)
    if isinstance(result, NoDataSentinel):
        assert result.reason.strip() != ""
    else:
        assert math.isfinite(result)
        assert result >= 0.0


@pytest.mark.property
@given(distribution=_distributions())
@settings(max_examples=200, deadline=1000)
def test_zero_surplus_is_always_absence_never_a_zero_burden(
    distribution: SurplusValueDistribution,
) -> None:
    """A zero denominator never resolves to a fabricated 0.0 burden."""
    if distribution.total_surplus_produced > 0.0:
        pytest.skip("denominator is well-defined for this example")
    result = serviceability_anchor(distribution)
    assert isinstance(result, NoDataSentinel)
    assert result.fips == distribution.fips_code
    assert result.year == distribution.year


@pytest.mark.property
@given(stock=_stocks(), real_output=_MONEY, distribution=_distributions())
@settings(max_examples=200, deadline=1000)
def test_anchors_are_pure_and_bit_deterministic(
    stock: FictitiousCapitalStock,
    real_output: float,
    distribution: SurplusValueDistribution,
) -> None:
    """Repeated calls on identical inputs return bit-identical results."""
    first_f = fictitious_anchor(stock, real_output)
    second_f = fictitious_anchor(stock, real_output)
    if isinstance(first_f, NoDataSentinel):
        assert isinstance(second_f, NoDataSentinel)
        assert first_f.reason == second_f.reason
    else:
        assert isinstance(second_f, float)
        assert first_f.hex() == second_f.hex()

    first_s = serviceability_anchor(distribution)
    second_s = serviceability_anchor(distribution)
    if isinstance(first_s, NoDataSentinel):
        assert isinstance(second_s, NoDataSentinel)
        assert first_s.reason == second_s.reason
    else:
        assert isinstance(second_s, float)
        assert first_s.hex() == second_s.hex()


@pytest.mark.property
@given(real_output=st.floats(min_value=0.01, max_value=1e12, allow_nan=False, allow_infinity=False))
@settings(max_examples=200, deadline=1000)
def test_zero_claims_is_always_absence_never_a_zero_anchor(real_output: float) -> None:
    """A zero numerator never resolves to a fabricated 0.0 log-anchor."""
    empty = FictitiousCapitalStock(
        year=2020,
        government_debt=0.0,
        corporate_equity=0.0,
        corporate_debt=0.0,
        household_debt=0.0,
    )
    result = fictitious_anchor(empty, real_output)
    assert isinstance(result, NoDataSentinel)
    assert result.year == 2020
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/property/invariants/test_monetary_anchor_absence.py`
Expected: PASS against the U4.1–U4.5 implementation. Prove the laws bite before accepting them: **Step 2a** replace the body of `serviceability_anchor`'s `if distribution.total_surplus_produced <= 0.0:` guard with `return 0.0`; **Step 2b** run the property file, expect `test_zero_surplus_is_always_absence_never_a_zero_burden` to fail with `assert isinstance(0.0, NoDataSentinel)` on the first shrunk zero-surplus example, while `test_serviceability_anchor_is_finite_or_absent_never_between` still passes (it admits a finite float, so it correctly does NOT catch this defect — that asymmetry is why both laws exist); **Step 3** `git checkout --`.
- [ ] **Step 3: Write minimal implementation**
No production change — the laws hold against U4.1–U4.5. Restore `src/babylon/domain/economics/monetary/anchor.py` to the committed state if Step 2's mutation was applied:
```bash
git checkout -- src/babylon/domain/economics/monetary/anchor.py
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/property/invariants/test_monetary_anchor_absence.py`
Expected: PASS (7 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(monetary): property laws for anchor honest absence

Absence is the normal steady state (~85% of a 2010-2109 campaign), so it is
pinned by laws over the whole input space rather than examples: every outcome
is a finite float or a reasoned sentinel, a zero denominator is never a zero
burden, and repeated calls are bit-identical.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.7: Export the anchor and gate its purity

**Files:**
- Modify: `src/babylon/domain/economics/monetary/__init__.py:1-27`
- Test: `tests/unit/economics/monetary/test_anchor.py` (append)

**Interfaces:**
- Consumes: `fictitious_anchor`, `serviceability_anchor` from `babylon.domain.economics.monetary.anchor`.
- Produces: the public import path `from babylon.domain.economics.monetary import fictitious_anchor, serviceability_anchor` — this is what U6 imports from `MarketScissorsSystem`.

- [ ] **Step 1: Write the failing test**
Append to `tests/unit/economics/monetary/test_anchor.py`:
```python
@pytest.mark.unit
class TestAnchorModuleContract:
    """The anchor is publicly exported and structurally pure (clauses 3 and 4)."""

    def test_exported_from_the_monetary_package(self) -> None:
        """U6 imports the anchor from the package, not the submodule."""
        import babylon.domain.economics.monetary as monetary

        assert "fictitious_anchor" in monetary.__all__
        assert "serviceability_anchor" in monetary.__all__
        assert monetary.fictitious_anchor is fictitious_anchor
        assert monetary.serviceability_anchor is serviceability_anchor

    def test_imports_nothing_from_the_engine_layer(self) -> None:
        """domain/ must not import engine/ (Program 14 layering)."""
        from pathlib import Path

        from babylon.domain.economics.monetary import anchor as anchor_module

        source = Path(str(anchor_module.__file__)).read_text(encoding="utf-8")
        assert "babylon.engine" not in source
        assert "babylon.web" not in source

    def test_uses_no_rng_and_no_wall_clock(self) -> None:
        """Determinism (Constitution III.7): zero RNG, zero clock, zero I/O."""
        from pathlib import Path

        from babylon.domain.economics.monetary import anchor as anchor_module

        source = Path(str(anchor_module.__file__)).read_text(encoding="utf-8")
        for forbidden in ("import random", "import time", "from datetime", "open("):
            assert forbidden not in source, f"anchor.py must not contain {forbidden!r}"
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py::TestAnchorModuleContract`
Expected: FAIL — `test_exported_from_the_monetary_package` fails with `AssertionError: assert 'fictitious_anchor' in ['MonetaryAdjustment', 'ValueBasis', 'PriceIndexSource', 'DefaultValueBasisConverter', 'ValueBasisConverter']`
- [ ] **Step 3: Write minimal implementation**
In `src/babylon/domain/economics/monetary/__init__.py`, replace lines 1-9 (the module docstring) with:
```python
"""Value basis conversion and the monetary anchor (Capital Volume III).

Expresses economic values in nominal dollars, real (inflation-adjusted)
dollars, and labor-time (SNLT hours), and exposes the monetary anchor that
calibrates the value-price scissors against real federal data where it exists.

See Also:
    :mod:`babylon.domain.economics.melt`: MELT calculator and basket visibility
    :mod:`babylon.domain.economics.snlt`: Socially Necessary Labor Time computation
"""
```
then insert directly above line 11's `from babylon.domain.economics.monetary.converter import (`:
```python
from babylon.domain.economics.monetary.anchor import (
    NATIONAL_FIPS,
    UNKNOWN_YEAR,
    fictitious_anchor,
    serviceability_anchor,
)
```
and replace the `__all__` block with:
```python
__all__: list[str] = [
    # Types
    "MonetaryAdjustment",
    "ValueBasis",
    # Data sources
    "PriceIndexSource",
    # Converter
    "DefaultValueBasisConverter",
    "ValueBasisConverter",
    # Anchor (design 2026-07-18 §3.3)
    "NATIONAL_FIPS",
    "UNKNOWN_YEAR",
    "fictitious_anchor",
    "serviceability_anchor",
]
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py`
Expected: PASS (22 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(monetary): export the anchor and gate its purity

Publishes fictitious_anchor/serviceability_anchor from the monetary package
(the import path U6 consumes) and pins clauses 3 and 4 of the §3.3 contract:
no engine import, no RNG, no wall clock, no I/O.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U4.8: Verify the unit lands clean

**Files:**
- Test: `tests/unit/economics/monetary/test_anchor.py`, `tests/property/invariants/test_monetary_anchor_absence.py`

**Interfaces:**
- Consumes: everything built in U4.1–U4.7.
- Produces: a green layering/typecheck/docstring gate, and the U4 acceptance evidence — *anchor absent ⇒ scissors output bit-identical to pre-U4* (trivially true at this point: no engine module reads the anchor until U6, so `qa:regression` must be unchanged).

- [ ] **Step 1: Run the layering gate**
Run: `mise run lint:imports`
Expected: PASS — `domain` may not import `engine`; the new module imports only `math`, `typing`, `credit.types`, `distribution.types`, `tensor`.
- [ ] **Step 2: Run lint, format and typecheck**
Run: `mise run check:quick`
Expected: PASS — Ruff clean, formatting stable, MyPy strict clean on `anchor.py` (both functions carry explicit `float | NoDataSentinel` return types).
- [ ] **Step 3: Run both anchor test files together**
Run: `mise run test:q -- tests/unit/economics/monetary/test_anchor.py tests/property/invariants/test_monetary_anchor_absence.py`
Expected: PASS (29 passed)
- [ ] **Step 4: Confirm the scissors is untouched**
Run: `mise run qa:regression`
Expected: byte-identical across all 5 scenarios. U4 adds a pure module with no engine consumer; any baseline movement here means something outside this unit changed and must be investigated before proceeding to U5.
- [ ] **Step 5: Record the evidence, then commit.** Append the four gate results verbatim to `reports/vol3-baseline-delta.md`'s "Verification evidence" table under a new `U4 gate sweep` row (create the file's skeleton now if U8.3 has not yet run; U8.3 fills the remaining rows), `git add reports/vol3-baseline-delta.md`, then commit. If you prefer not to create the report early, DELETE Step 5 entirely — a commit step with an empty index aborts and leaves the plan in a false "done" state.
```bash
mise run commit -- "chore(monetary): verify U4 monetary anchor lands clean

lint:imports, check:quick and qa:regression all green; the anchor has no
engine consumer until U6, so the five baselines are byte-identical.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.1: Extend `GraphInputs` with the four Volume III measure fields

**Files:**
- Modify: `src/babylon/domain/dialectics/instances/catalog.py:90-131` (the `GraphInputs` frozen dataclass — docstring `Attributes:` block and the field list ending at line 131)
- Test: `tests/unit/dialectics/test_catalog.py`

**Interfaces:**
- Consumes: nothing from earlier units (pure dataclass extension).
- Produces: `GraphInputs.rentier_share`, `GraphInputs.debt_ratio`, `GraphInputs.credit_fragility`, `GraphInputs.financialization_index` — each `float | None = field(default=None)`. U5.2's measures read them; U5.7 (`ContradictionSystem._build_graph_inputs`) fills them.

- [ ] **Step 1: Write the failing test**

Append to `tests/unit/dialectics/test_catalog.py` (end of file):

```python
class TestVolumeThreeInputFields:
    """The four Vol III money fields are optional and absent by default.

    Absence is the normal steady state for ~85% of a campaign (the FRED
    series terminate at 2024), so ``None`` must be the DEFAULT, never a
    fabricated 0.0 (Constitution III.11). Post-U9: ``credit_fragility`` =
    ``default_rate`` x the **endogenous** fragility premium, present
    whenever a profit rate is; the other three measures keep their
    FRED-absent steady state.
    """

    def test_all_four_default_to_none(self) -> None:
        inputs = GraphInputs()
        assert inputs.rentier_share is None
        assert inputs.debt_ratio is None
        assert inputs.credit_fragility is None
        assert inputs.financialization_index is None

    def test_all_four_are_settable_floats(self) -> None:
        inputs = GraphInputs(
            rentier_share=0.4,
            debt_ratio=1.5,
            credit_fragility=2.0,
            financialization_index=3.5,
        )
        assert inputs.rentier_share == pytest.approx(0.4)
        assert inputs.debt_ratio == pytest.approx(1.5)
        assert inputs.credit_fragility == pytest.approx(2.0)
        assert inputs.financialization_index == pytest.approx(3.5)

    def test_graph_inputs_stays_frozen(self) -> None:
        inputs = GraphInputs(rentier_share=0.4)
        with pytest.raises(AttributeError):
            inputs.rentier_share = 0.9  # type: ignore[misc]
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py::TestVolumeThreeInputFields`

Expected: FAIL — `TypeError: GraphInputs.__init__() got an unexpected keyword argument 'rentier_share'` (and `AttributeError: 'GraphInputs' object has no attribute 'rentier_share'` in the first test).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/domain/dialectics/instances/catalog.py`, extend the `GraphInputs` docstring `Attributes:` block by appending these entries immediately after the existing `market_balance:` entry (which ends at line 121 with `catalog stays defines-free; ``None`` = no market axis this tick.`):

```python
        rentier_share: NATIONAL aggregate ``(i + r + t) / s`` — the share of
            produced surplus value claimed by interest, ground rent and taxes
            rather than retained by the functioning capitalist (Capital Vol.
            III part 5). Computed by the engine as ``Σclaims / Σsurplus``
            across counties — an EXTENSIVE ratio-of-sums, never a mean of
            per-county ratios. ``None`` = no county carries a surplus
            distribution this tick.
        debt_ratio: NATIONAL ``Σ accumulated_debt / Σ annual surplus`` — the
            cumulative enterprise-profit shortfall measured against the
            surplus that would have to service it. ``None`` = no county
            carries a debt accumulation this tick.
        credit_fragility: ``default_rate * spread``, pre-divided by the
            defines-owned crisis reference so 1.0 IS the crisis threshold
            (the engine owns the scale, exactly as it owns the ``tanh``
            scale for ``market_balance``, keeping this module defines-free).
            ``None`` = no national credit state published this tick.
        financialization_index: fictitious claims over real production. Read
            from the scissors' ``fictitious_log`` in ratio space
            (``exp``), which the monetary anchor calibrates to
            ``FictitiousCapitalStock.ratio_to_real`` while real data exists —
            one axis, materially grounded at its origin, endogenous
            thereafter. ``None`` = no market axis this tick.
```

Then append the four fields immediately after the line `market_balance: float | None = field(default=None)`:

```python
    rentier_share: float | None = field(default=None)
    debt_ratio: float | None = field(default=None)
    credit_fragility: float | None = field(default=None)
    financialization_index: float | None = field(default=None)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py::TestVolumeThreeInputFields`

Expected: PASS (3 passed)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(dialectics): GraphInputs carries the four Vol III money ratios

Four optional float fields — rentier_share, debt_ratio, credit_fragility,
financialization_index — default None so absence is honest (III.11) and the
oppositions bound in the next commit read (0, 0) rather than a fabricated zero.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.2: Bind the four Volume III oppositions (catalog 6 → 10)

**Files:**
- Modify: `src/babylon/domain/dialectics/instances/catalog.py:262-370` (add the shared ratio helper + four measures after `_price_value_measure`, and four `BoundOpposition` entries inside `build_default_registry`'s `bindings` list)
- Modify: `tests/unit/dialectics/test_catalog.py:28-36` (the stale `test_six_oppositions_bound` assertion)
- Test: `tests/unit/dialectics/test_catalog.py`

**Interfaces:**
- Consumes: `GraphInputs.rentier_share | debt_ratio | credit_fragility | financialization_index` (U5.1).
- Produces: registry keys `"surplus_distribution"`, `"debt_spiral"`, `"credit"`, `"financial"` — the endpoints U5.4's coupling edges and U5.8's ranking rule require.

- [ ] **Step 1: Write the failing test**

First replace the stale six-key assertion at `tests/unit/dialectics/test_catalog.py:28-36` with:

```python
    def test_ten_oppositions_bound(self) -> None:
        assert _reg().keys == (
            "atomization",
            "capital_labor",
            "credit",
            "debt_spiral",
            "financial",
            "imperial",
            "price_value",
            "surplus_distribution",
            "tenancy",
            "wage",
        )
```

Then append this class to the end of the same file:

```python
class TestVolumeThreeOppositions:
    """The four Vol III bindings: shared ratio family, honest absence.

    Every one reads a NON-NEGATIVE ratio against its own material unity
    point (claims == substance) and maps it with the same zero-parameter
    saturating family: ``gap = x/(1+x)``, ``balance = (x-1)/(x+1)``. So the
    balance crosses zero exactly where the claim equals the substance it
    claims, and the gap is 0 only when the claim is absent altogether.
    """

    @pytest.mark.parametrize(
        "key",
        ["surplus_distribution", "debt_spiral", "credit", "financial"],
    )
    def test_absent_input_reads_zero_zero(self, key: str) -> None:
        # No Vol III data (the ~85%-of-campaign steady state): no claim,
        # no contradiction — never a fabricated value.
        states = _states(GraphInputs())
        assert states[key].gap == pytest.approx(0.0)
        assert states[key].balance == pytest.approx(0.0)

    @pytest.mark.parametrize(
        "key",
        ["surplus_distribution", "debt_spiral", "credit", "financial"],
    )
    def test_none_of_them_is_antagonistic(self, key: str) -> None:
        # The catalog reserves antagonistic=True for capital_labor and
        # imperial alone: the division of surplus AMONG capitals is real
        # conflict but intra-class, and mislabelling it would corrupt
        # principal-contradiction ranking.
        assert _reg().spec_for(key).antagonistic is False

    def test_levels_are_county_for_the_two_county_axes(self) -> None:
        assert _reg().spec_for("surplus_distribution").level_name == "county"
        assert _reg().spec_for("debt_spiral").level_name == "county"

    def test_the_two_national_axes_are_unplaced(self) -> None:
        assert _reg().spec_for("credit").level_name == ""
        assert _reg().spec_for("financial").level_name == ""

    def test_poles_are_named_as_specified(self) -> None:
        reg = _reg()
        assert (reg.spec_for("surplus_distribution").pole_a) == "enterprise"
        assert (reg.spec_for("surplus_distribution").pole_b) == "rentier"
        assert (reg.spec_for("debt_spiral").pole_a) == "solvent"
        assert (reg.spec_for("debt_spiral").pole_b) == "indebted"
        assert (reg.spec_for("credit").pole_a) == "accommodation"
        assert (reg.spec_for("credit").pole_b) == "fragility"
        assert (reg.spec_for("financial").pole_a) == "real"
        assert (reg.spec_for("financial").pole_b) == "fictitious"

    def test_zero_claim_is_no_contradiction_not_maximal(self) -> None:
        # Rentiers claim nothing: the functioning capitalist retains the
        # whole surplus. That is the ABSENCE of the conflict, not its peak.
        states = _states(GraphInputs(rentier_share=0.0, debt_ratio=0.0))
        assert states["surplus_distribution"].gap == pytest.approx(0.0)
        assert states["surplus_distribution"].balance == pytest.approx(-1.0)
        assert states["surplus_distribution"].leading_pole == "a"
        assert states["debt_spiral"].gap == pytest.approx(0.0)
        assert states["debt_spiral"].balance == pytest.approx(-1.0)

    def test_unity_point_is_the_balance_zero_crossing(self) -> None:
        # x == 1: claims exactly equal the surplus they claim (p == 0);
        # fragility exactly at its crisis reference; fictitious exactly at
        # parity with real. Neither pole leads.
        states = _states(
            GraphInputs(
                rentier_share=1.0,
                debt_ratio=1.0,
                credit_fragility=1.0,
                financialization_index=1.0,
            )
        )
        for key in ("surplus_distribution", "debt_spiral", "credit", "financial"):
            assert states[key].balance == pytest.approx(0.0)
            assert states[key].gap == pytest.approx(0.5)

    def test_claims_exceeding_surplus_puts_the_rentier_pole_in_the_lead(self) -> None:
        # (i + r + t) = 3s: interest, rent and taxes consume three times the
        # surplus produced. Enterprise profit is deeply negative.
        states = _states(GraphInputs(rentier_share=3.0))
        assert states["surplus_distribution"].gap == pytest.approx(0.75)
        assert states["surplus_distribution"].balance == pytest.approx(0.5)
        assert states["surplus_distribution"].leading_pole == "b"

    def test_financialization_bubble_reads_fictitious_dominant(self) -> None:
        # 3.5 is the FRED TCMDO/GDP overaccumulation reading (~2008 peak).
        states = _states(GraphInputs(financialization_index=3.5))
        assert states["financial"].balance == pytest.approx(2.5 / 4.5)
        assert states["financial"].leading_pole == "b"

    def test_negative_ratios_are_rejected_as_absent(self) -> None:
        # A ratio of a non-negative claim to a non-negative substance can
        # never be negative; a negative reading is corrupt input, and the
        # honest response is the absent reading, not a clamped fiction.
        states = _states(GraphInputs(debt_ratio=-2.0))
        assert states["debt_spiral"].gap == pytest.approx(0.0)
        assert states["debt_spiral"].balance == pytest.approx(0.0)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py::TestVolumeThreeOppositions`

Expected: FAIL — `KeyError: 'surplus_distribution'` from `_states(...)[key]` (the registry has no such binding), and `KeyError: 'surplus_distribution'` from `spec_for`.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/domain/dialectics/instances/catalog.py`, insert immediately above the line `def build_default_registry(rate_weight: float = 10.0) -> OppositionRegistry[GraphInputs]:`:

```python
def _ratio_reading(ratio: float | None) -> GapReading:
    """Map a non-negative claim/substance ratio onto ``(gap, balance)``.

    The shared measure family for every Volume III money opposition. Each
    reads a ratio of a CLAIM on value to the value that must validate it
    — rentier claims to surplus produced, accumulated debt to annual
    surplus, credit fragility to its crisis reference, fictitious capital
    to real production — so all four share one zero-parameter map::

        gap     = x / (1 + x)
        balance = (x - 1) / (x + 1) = 2 * gap - 1

    Reading the two outputs materially: the balance crosses zero exactly
    at ``x = 1``, the point where the claim equals the substance claimed
    (enterprise profit exactly extinguished, fragility exactly at
    threshold, paper exactly at parity with production). Below it the
    substance leads (pole A); above it the claim leads (pole B). The gap
    is 0 only where the claim is absent altogether — a surplus no rentier
    touches carries no rentier contradiction — and saturates toward 1 as
    the claim runs away from what produces it.

    The family is deliberately scale-free (no coefficient, so this module
    stays defines-free per its import contract); any scaling a ratio needs
    is applied by the engine before it reaches :class:`GraphInputs`, the
    same division of labour ``market_balance`` already uses.

    Args:
        ratio: The claim/substance ratio, or ``None`` when the underlying
            data is absent.

    Returns:
        ``GapReading(0.0, 0.0)`` — the catalog's canonical ABSENT reading —
        when ``ratio`` is ``None`` or negative (a ratio of two non-negative
        magnitudes cannot be negative, so a negative value is corrupt input
        and absence is the honest answer, Constitution III.11). Otherwise
        the saturating reading above.
    """
    if ratio is None or ratio < 0.0:
        return GapReading(gap=0.0, balance=0.0)
    gap = ratio / (1.0 + ratio)
    return GapReading(gap=gap, balance=2.0 * gap - 1.0)


def _surplus_distribution_measure(inputs: GraphInputs) -> GapReading:
    """enterprise (A) ⇄ rentier (B) — the division of surplus among capitals."""
    return _ratio_reading(inputs.rentier_share)


def _debt_spiral_measure(inputs: GraphInputs) -> GapReading:
    """solvent (A) ⇄ indebted (B) — accumulated shortfall against annual surplus."""
    return _ratio_reading(inputs.debt_ratio)


def _credit_measure(inputs: GraphInputs) -> GapReading:
    """accommodation (A) ⇄ fragility (B) — ``default_rate * spread`` in threshold units."""
    return _ratio_reading(inputs.credit_fragility)


def _financial_measure(inputs: GraphInputs) -> GapReading:
    """real (A) ⇄ fictitious (B) — claims on future value over present production."""
    return _ratio_reading(inputs.financialization_index)
```

Then insert these four `BoundOpposition` entries into `build_default_registry`'s `bindings` list, after the `BoundOpposition` block whose body contains `measure=_price_value_measure,`, before the `]` that closes the `bindings` list:

```python
        BoundOpposition(
            spec=OppositionSpec(
                key="surplus_distribution",
                pole_a="enterprise",
                pole_b="rentier",
                unity="the functioning capitalist can only set production going with "
                "capital the money-capitalist, the landowner and the state advance or "
                "levy against it; interest, ground rent and taxes are therefore not "
                "deductions from an alien fund but the shares in which the one surplus "
                "value the workers produced is divided among the capitals that claim "
                "it (Capital Vol. III parts 4-6)",
                level_name="county",
                antagonistic=False,
            ),
            measure=_surplus_distribution_measure,
        ),
        BoundOpposition(
            spec=OppositionSpec(
                key="debt_spiral",
                pole_a="solvent",
                pole_b="indebted",
                unity="when the rentier claims outrun the surplus produced, the "
                "shortfall is not settled but carried: the enterprise borrows to pay "
                "the interest it already owes, and the debt is a claim on surplus "
                "value not yet extracted from any worker — solvency and indebtedness "
                "are the same accumulation read at two moments (Capital Vol. III ch. 30-32)",
                level_name="county",
                antagonistic=False,
            ),
            measure=_debt_spiral_measure,
        ),
        BoundOpposition(
            spec=OppositionSpec(
                key="credit",
                pole_a="accommodation",
                pole_b="fragility",
                unity="credit is the lever that carries accumulation past the limits "
                "of the individual capital, and by exactly the same act it makes each "
                "capital's reproduction depend on every other's payment: the system "
                "that accommodates the boom IS the system that transmits the default "
                "(Capital Vol. III ch. 27, 30)",
                # level_name stays "" (unplaced): the credit system is national;
                # it sits on no county/bloc lattice rung.
                antagonistic=False,
            ),
            measure=_credit_measure,
        ),
        BoundOpposition(
            spec=OppositionSpec(
                key="financial",
                pole_a="real",
                pole_b="fictitious",
                unity="a bond, a share and a mortgage are titles to future surplus "
                "value, not the value itself; they are bought and sold as capital "
                "while the labour that must validate them has not been performed — "
                "the paper presupposes the production it has already outrun (Capital "
                "Vol. III ch. 25, 29)",
                # level_name stays "" (unplaced): the fictitious-capital stock and
                # the scissors axis reading it are both national.
                antagonistic=False,
            ),
            measure=_financial_measure,
        ),
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py`

Expected: PASS (whole file, including the rewritten `test_ten_oppositions_bound`)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(dialectics): bind the four Volume III oppositions (catalog 6 -> 10)

surplus_distribution (enterprise/rentier), debt_spiral (solvent/indebted),
credit (accommodation/fragility) and financial (real/fictitious), all
antagonistic=False — the division of surplus among capitals is real conflict
but intra-class, and antagonistic=True is reserved for capital_labor and
imperial. All four share one zero-parameter saturating map whose balance
crosses zero exactly where the claim equals the substance claiming it.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.3: Correct the stale "five bound contradictions" catalog docstring

**Files:**
- Modify: `src/babylon/domain/dialectics/instances/catalog.py:1-48` (module docstring)
- Test: `tests/unit/dialectics/test_catalog.py`

**Interfaces:**
- Consumes: the ten registry keys from U5.2.
- Produces: nothing consumed downstream — a documentation-accuracy contract only.

- [ ] **Step 1: Write the failing test**

Append to `tests/unit/dialectics/test_catalog.py`:

```python
class TestCatalogDocstringAccuracy:
    """The module docstring is a claim about the registry; pin it to the code.

    It went stale twice already — it still said "five bound contradictions"
    and omitted ``price_value`` for the whole period during which
    ``price_value`` was CANONICAL (ADR078). Documentation that describes a
    registry can be checked against that registry, so it is.
    """

    def test_docstring_names_every_registered_key(self) -> None:
        import babylon.domain.dialectics.instances.catalog as catalog_module

        docstring = catalog_module.__doc__ or ""
        for key in build_default_registry().keys:
            assert f"``{key}``" in docstring, f"docstring never mentions {key!r}"

    def test_docstring_does_not_claim_five(self) -> None:
        import babylon.domain.dialectics.instances.catalog as catalog_module

        docstring = catalog_module.__doc__ or ""
        assert "five bound contradictions" not in docstring
        assert "The five oppositions" not in docstring
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py::TestCatalogDocstringAccuracy`

Expected: FAIL — `AssertionError: docstring never mentions 'credit'` and `assert 'five bound contradictions' not in ...`.

- [ ] **Step 3: Write minimal implementation**

Replace the summary line at `catalog.py:1` and the list intro + entries so the docstring covers all ten keys. Line 1 becomes:

```python
"""The production opposition catalog: Babylon's ten bound contradictions.
```

Replace lines 12-14 (`The five oppositions, and the honest measure each is bound to on this / branch (verified against a 30-tick single-county bridged probe, / 2026-07-02):`) with:

```python
The ten oppositions, and the honest measure each is bound to. The first
five were verified against a 30-tick single-county bridged probe
(2026-07-02); ``price_value`` was promoted to CANONICAL by ADR078; the
four Volume III money oppositions were bound by the Vol III money-scissors
work (2026-07-18):
```

Then append these entries to the bullet list, immediately after the `imperial` bullet (which ends at line 38 with `encoded as ``wage feeds imperial`` in the default coupling graph.`):

```
- ``price_value`` — the Market Scissors axis (Program 23, ADR077/ADR078)
  read as an adjunction defect: gap and balance come from the pre-derived
  ``GraphInputs.market_balance``, the engine's ``tanh(price_log / scale)``.
  CANONICAL: it competes for principal contradiction.
- ``surplus_distribution`` — enterprise⇄rentier: the rentier share
  ``(i + r + t) / s``, the division of one county's produced surplus among
  the capitals claiming it. Balance crosses zero where the claims exactly
  extinguish enterprise profit.
- ``debt_spiral`` — solvent⇄indebted: accumulated enterprise-profit
  shortfall over annual surplus. Zero debt reads gap 0 (no contradiction),
  balance −1 (the solvent pole leads).
- ``credit`` — accommodation⇄fragility: ``default_rate * spread``, scaled by
  the engine against its crisis reference so balance crosses zero AT the
  threshold. National; unplaced on the level lattice.
- ``financial`` — real⇄fictitious: claims on future value over present
  production, read from the scissors' ``fictitious_log`` in ratio space.
  National; unplaced on the level lattice.

All four Volume III bindings share ``_ratio_reading``'s zero-parameter
saturating map and all four are ``antagonistic=False``: the division of
surplus among capitals is real conflict but INTRA-class, and only
``capital_labor`` and ``imperial`` carry the rupture-producing flag.
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/dialectics/test_catalog.py::TestCatalogDocstringAccuracy`

Expected: PASS (2 passed)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "docs(dialectics): catalog docstring covers all ten bound oppositions

It claimed five and omitted price_value, canonical since ADR078. Pinned to
the registry by a test so the next binding cannot leave it stale again.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.4: Land the five derived coupling edges

**Files:**
- Modify: `src/babylon/domain/dialectics/instances/catalog.py:373-394` (the `_DEFAULT_COUPLINGS` comment block and tuple)
- Modify: `tests/unit/dialectics/test_coupling.py:136-166` (`test_registered_couplings_are_kept`, `test_only_the_bound_edges_survive`, `test_unbound_transforms_are_skipped_and_logged`)
- Test: `tests/unit/dialectics/test_coupling.py`

**Interfaces:**
- Consumes: registry keys `surplus_distribution`, `debt_spiral`, `credit`, `financial` (U5.2).
- Produces: a `build_default_coupling_graph(...)` result containing exactly ten non-`contains` edges, including the two `transforms` edges U5.8's ranking rule walks.

- [ ] **Step 1: Write the failing test**

Replace `tests/unit/dialectics/test_coupling.py:136-166` (the three methods of `TestDefaultCouplingGraph`) with:

```python
    def test_registered_couplings_are_kept(self) -> None:
        graph = build_default_coupling_graph(build_default_registry())
        triples = {_triple(c) for c in graph.couplings}
        assert ("wage", "capital_labor", "feeds") in triples
        assert ("wage", "imperial", "feeds") in triples  # Phase D5: shared defect
        assert ("wage", "price_value", "feeds") in triples  # ADR078: the flow IS the drive
        assert ("capital_labor", "imperial", "antagonizes") in triples
        assert ("imperial", "capital_labor", "antagonizes") in triples  # symmetric

    def test_the_two_reserved_vol_three_transforms_now_survive(self) -> None:
        """The edges that sat dormant for months, awaiting their endpoints."""
        graph = build_default_coupling_graph(build_default_registry())
        triples = {_triple(c) for c in graph.couplings}
        assert ("surplus_distribution", "debt_spiral", "transforms") in triples
        assert ("credit", "financial", "transforms") in triples

    def test_price_value_and_financial_feed_each_other(self) -> None:
        """Mutual feeds at two moments of one cycle: in expansion price
        momentum drives speculation (fictitious_drive reads price_velocity);
        in correction the bubble snaps prices. Nothing in CouplingGraph
        requires acyclicity, and the reciprocal pair is the truthful record."""
        graph = build_default_coupling_graph(build_default_registry())
        triples = {_triple(c) for c in graph.couplings}
        assert ("price_value", "financial", "feeds") in triples
        assert ("financial", "price_value", "feeds") in triples

    def test_the_interest_burden_constrains_the_financial_axis(self) -> None:
        graph = build_default_coupling_graph(build_default_registry())
        triples = {_triple(c) for c in graph.couplings}
        assert ("surplus_distribution", "financial", "constrains") in triples

    def test_only_the_bound_edges_survive(self) -> None:
        graph = build_default_coupling_graph(build_default_registry())
        non_contains = {_triple(c) for c in graph.couplings if c.kind != "contains"}
        assert non_contains == {
            ("wage", "capital_labor", "feeds"),
            ("wage", "imperial", "feeds"),
            ("wage", "price_value", "feeds"),
            ("capital_labor", "imperial", "antagonizes"),
            ("imperial", "capital_labor", "antagonizes"),
            ("surplus_distribution", "debt_spiral", "transforms"),
            ("credit", "financial", "transforms"),
            ("price_value", "financial", "feeds"),
            ("financial", "price_value", "feeds"),
            ("surplus_distribution", "financial", "constrains"),
        }

    def test_only_the_volume_two_edges_are_still_skipped(
        self, caplog: pytest.LogCaptureFixture
    ) -> None:
        with caplog.at_level(logging.INFO, logger="babylon.domain.dialectics.instances.catalog"):
            build_default_coupling_graph(build_default_registry())
        skipped = [r for r in caplog.records if "Skipping coupling" in r.getMessage()]
        # Only the two Volume II circulation edges remain unbound; they are
        # explicitly out of scope and are NOT faked.
        assert len(skipped) == 2
        joined = " ".join(r.getMessage() for r in skipped)
        for endpoint in ("realization", "disproportionality"):
            assert endpoint in joined
        for landed in ("debt_spiral", "financial"):
            assert landed not in joined
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/dialectics/test_coupling.py::TestDefaultCouplingGraph`

Expected: FAIL — `test_price_value_and_financial_feed_each_other` fails with `assert ('price_value', 'financial', 'feeds') in triples` (that edge is not declared), and `test_only_the_bound_edges_survive` fails on the set comparison.

- [ ] **Step 3: Write minimal implementation**

Replace the comment block beginning `# The ratified crisis-producer map.` through the closing `)` of the `_DEFAULT_COUPLINGS` tuple with:

```python
# The ratified crisis-producer map. Every edge is DERIVED — read off the code
# against ``coupling.py``'s operational definitions of the five kinds, not
# authored from theory — and carries its citation, because the graph is a
# CLAIM ABOUT THE CODE and drifts from it the moment either side changes.
# (That drift is exactly how the two Vol III ``transforms`` edges below sat
# dormant and undetected for months.) The builder keeps only edges whose BOTH
# endpoints are registered; it never invents a null binding for an absent one.
_DEFAULT_COUPLINGS: tuple[Coupling, ...] = (
    # crisis producers: source's output becomes target's input prices.
    # Still unbound — the two Volume II circulation oppositions are out of
    # scope for the Vol III money work and are NOT faked.
    Coupling(source="circulation", target="realization", kind="transforms"),
    Coupling(source="reproduction", target="disproportionality", kind="transforms"),
    # DebtAccumulation.update consumes profit_of_enterprise — the residual of
    # the surplus distribution (economics/tick/system/__init__.py, the annual
    # county financial block): the distribution's output IS the debt tracker's
    # input. Reserved since Phase D; live since the Vol III binding.
    Coupling(source="surplus_distribution", target="debt_spiral", kind="transforms"),
    # Credit conditions become fictitious accumulation's input: the default
    # rate and spread that price credit are what the claims on future value
    # are capitalized against. Reserved since Phase D; live since the Vol III
    # binding.
    Coupling(source="credit", target="financial", kind="transforms"),
    # the two antagonistic class contradictions are mutually antagonistic
    Coupling(source="capital_labor", target="imperial", kind="antagonizes"),
    # capital_labor's development presupposes the wage relation it reads
    Coupling(source="wage", target="capital_labor", kind="feeds"),
    # wage and imperial read the SAME (w_paid, v_produced) defect (D5): the
    # per-class wage relation feeds the frame-level imperial-rent reading.
    Coupling(source="wage", target="imperial", kind="feeds"),
    # the realized wage⇄value flow IS the scissors' drive term (ADR078): the
    # market axis integrates what the wage relation produces each tick.
    Coupling(source="wage", target="price_value", kind="feeds"),
    #
    # The reciprocal pair below is not a modelling flourish: the two edges
    # fire at DIFFERENT moments of one cycle, and both are readable in
    # engine/systems/market_scissors.py.
    #
    # Expansion — fictitious_drive includes ``momentum_coupling *
    # price_velocity`` (market_scissors.py fictitious-drive block): the
    # fictitious step READS the price observation, so ``feeds``.
    Coupling(source="price_value", target="financial", kind="feeds"),
    # Correction — calculate_correction_snap pulls price_log down from the
    # fictitious overhang (market_scissors.py correction block): the price
    # step READS the fictitious observation, so ``feeds`` back. CouplingGraph
    # requires no acyclicity, and the cycle is the honest record.
    Coupling(source="financial", target="price_value", kind="feeds"),
    # The interest burden i/s sets serviceable_divergence — the ceiling on
    # fictitious_log before the snap — so the distribution LIMITS the
    # financial axis's reachable state space: ``constrains``, not ``feeds``.
    Coupling(source="surplus_distribution", target="financial", kind="constrains"),
)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/dialectics/test_coupling.py`

Expected: PASS (whole file)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(dialectics): land the five derived Vol III coupling edges

Both reserved transforms edges survive build_default_coupling_graph now that
their endpoints are bound; plus a reciprocal price_value<->financial feeds pair
(expansion momentum drives speculation; correction snaps prices) and a
surplus_distribution constrains financial edge nobody had declared. Each edge
cites the code it is read off — the graph is a claim about the code.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.5: Add `coupling_graph` to the service container and protocol

**Files:**
- Modify: `src/babylon/kernel/services.py:46` (add `coupling_graph` to `ServicesProtocol`, beside `opposition_registry`)
- Modify: `src/babylon/engine/services.py:154-158` (field declaration), `:259` (create kwarg), `:328-333` (default construction), `:355` (pass-through in the returned container)
- Test: `tests/unit/engine/test_services_coupling_graph.py`

**Interfaces:**
- Consumes: `build_default_coupling_graph(registry) -> CouplingGraph` (U5.4).
- Produces: `services.coupling_graph: CouplingGraph | None` — the attribute U5.8's `ContradictionSystem` reads.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/engine/test_services_coupling_graph.py`:

```python
"""The CouplingGraph's first production wiring (Vol III money scissors, U5).

``CouplingGraph`` and ``build_default_coupling_graph`` had ZERO production
callers: the coupling layer was dormant scaffolding. Per Constitution III.10
a construct must not ship as vocabulary, so it is wired here and consumed by
``ContradictionSystem``. These tests pin the wiring; the consumption contract
is pinned in ``tests/unit/engine/systems/test_contradiction_system.py``.
"""

from __future__ import annotations

import pytest

from babylon.domain.dialectics.core.coupling import CouplingGraph
from babylon.domain.dialectics.core.opposition import (
    BoundOpposition,
    GapReading,
    OppositionRegistry,
    OppositionSpec,
)
from babylon.engine.services import ServiceContainer

pytestmark = pytest.mark.unit


def _triple(coupling: object) -> tuple[str, str, str]:
    return (coupling.source, coupling.target, coupling.kind)  # type: ignore[attr-defined]


class TestCouplingGraphWiring:
    def test_default_container_carries_a_coupling_graph(self) -> None:
        assert isinstance(ServiceContainer.create().coupling_graph, CouplingGraph)

    def test_default_graph_is_the_production_crisis_producer_map(self) -> None:
        graph = ServiceContainer.create().coupling_graph
        triples = {_triple(c) for c in graph.couplings}
        assert ("surplus_distribution", "debt_spiral", "transforms") in triples
        assert ("credit", "financial", "transforms") in triples

    def test_an_explicit_graph_is_respected(self) -> None:
        registry: OppositionRegistry[object] = OppositionRegistry(bindings=[])
        explicit = CouplingGraph([], registry)
        container = ServiceContainer.create(coupling_graph=explicit)
        assert container.coupling_graph is explicit

    def test_graph_is_built_over_the_injected_registry(self) -> None:
        """A custom registry gets a graph over ITS keys, with every edge whose
        endpoints it does not register dropped — never a KeyError."""
        registry: OppositionRegistry[object] = OppositionRegistry(
            bindings=[
                BoundOpposition(
                    spec=OppositionSpec(key="wage", pole_a="a", pole_b="b"),
                    measure=lambda _inputs: GapReading(gap=0.0, balance=0.0),
                ),
            ]
        )
        container = ServiceContainer.create(opposition_registry=registry)
        assert isinstance(container.coupling_graph, CouplingGraph)
        assert container.coupling_graph.couplings == ()

    def test_protocol_declares_the_attribute(self) -> None:
        from babylon.kernel.services import ServicesProtocol

        assert "coupling_graph" in ServicesProtocol.__annotations__
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/test_services_coupling_graph.py`

Expected: FAIL — `AttributeError: 'ServiceContainer' object has no attribute 'coupling_graph'`.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/kernel/services.py`, add after line 46 (`opposition_registry: Any`):

```python
    coupling_graph: Any
```

In `src/babylon/engine/services.py`, add after line 158 (the `opposition_registry: Any = field(default=None)` declaration):

```python
    # Typed coupling graph over the opposition registry (Vol III money
    # scissors, U5). Built by ``create`` beside the registry: ContradictionSystem
    # consumes it so a ``transforms`` target cannot rank principal while its
    # source reads absent — crisis has a direction of travel, and this is what
    # knows it. None only in hand-built containers.
    coupling_graph: Any = field(default=None)
```

Also add a module-level sentinel to `src/babylon/engine/services.py` (near the top of the file, importing `Final` from `typing` if not already imported):

```python
_UNSET: Final = object()
```

Add the keyword-only parameter to `create` after line 259 (`opposition_registry: Any = None,`):

```python
        coupling_graph: Any = _UNSET,
```

Add the default construction immediately after the existing `opposition_registry` block (which ends at line 333):

```python
        # Sentinel default so an EXPLICIT coupling_graph=None can disable the
        # rule, distinguishable from an omitted argument. ServiceContainer is
        # a plain dataclass today, so tests could mutate the field in place —
        # but relying on that couples them to non-frozenness the container
        # does not promise.
        if coupling_graph is _UNSET and opposition_registry is not None:
            from babylon.domain.dialectics.instances.catalog import (
                build_default_coupling_graph,
            )

            coupling_graph = build_default_coupling_graph(opposition_registry)
        if coupling_graph is _UNSET:
            coupling_graph = None
```

Add the pass-through in the returned container, immediately after line 355 (`opposition_registry=opposition_registry,`):

```python
            coupling_graph=coupling_graph,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/test_services_coupling_graph.py tests/unit/kernel/test_services_protocol.py`

Expected: PASS (both files)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(engine): wire CouplingGraph into the service container

CouplingGraph and build_default_coupling_graph had zero production callers —
dormant scaffolding. ServiceContainer.create now builds the graph beside the
opposition registry (protocol + concrete), over whatever registry is injected.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.6: Add the `credit_fragility_scale` define

**Files:**
- Modify: `src/babylon/config/defines/capital_vol3.py` (append the field to `CapitalVolumeIIIDefines`, after `housing_capitalization_rate_default` — U2.3 created this category as the Vol III coefficient home, and `default_rate_estimate` (the other factor of `default_rate * spread`) already lives there)
- Modify: `src/babylon/data/defines.yaml:195-202` (regenerated, not hand-edited)
- Test: `tests/unit/config/test_tension_credit_fragility_scale.py`

**Interfaces:**
- Consumes: nothing.
- Produces: `defines.capital_vol3.credit_fragility_scale: float` — the divisor U5.7 applies before `credit_fragility` reaches `GraphInputs`.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/config/test_tension_credit_fragility_scale.py`:

```python
"""The credit-fragility reference scale is a player-editable define.

``GraphInputs.credit_fragility`` reaches the defines-free catalog already
divided by this reference, so the shared ratio map's balance crosses zero
exactly AT the crisis threshold. Hardcoding the divisor in the engine would
be a Constitution III.1 violation and would make the credit opposition
unmoddable.
"""

from __future__ import annotations

import pytest

from babylon.config.defines import GameDefines

pytestmark = pytest.mark.unit


class TestCreditFragilityScale:
    def test_default_matches_the_material_crisis_threshold(self) -> None:
        # FRED BAA-AAA spread * Moody's default rate: the product exceeded
        # 0.02 during the 2008 crisis (CREDIT_FRAGILITY_THRESHOLD's own
        # derivation, domain/economics/credit/types.py).
        assert GameDefines().capital_vol3.credit_fragility_scale == pytest.approx(0.02)

    def test_must_be_positive(self) -> None:
        from pydantic import ValidationError

        from babylon.config.defines import CapitalVolumeIIIDefines

        with pytest.raises(ValidationError):
            CapitalVolumeIIIDefines(credit_fragility_scale=0.0)

    def test_the_shipped_yaml_carries_it(self) -> None:
        loaded = GameDefines.load_default()
        assert loaded.capital_vol3.credit_fragility_scale > 0.0

    def test_a_yaml_edit_reaches_the_engine(self) -> None:
        modded = GameDefines(capital_vol3=GameDefines().capital_vol3.model_copy(
            update={"credit_fragility_scale": 0.04}
        ))
        assert modded.capital_vol3.credit_fragility_scale == pytest.approx(0.04)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/config/test_tension_credit_fragility_scale.py`

Expected: FAIL — `AttributeError: 'CapitalVolumeIIIDefines' object has no attribute 'credit_fragility_scale'`.

- [ ] **Step 3: Write minimal implementation**

Append to `CapitalVolumeIIIDefines` in `src/babylon/config/defines/capital_vol3.py`, after `housing_capitalization_rate_default`:

```python
    credit_fragility_scale: float = Field(
        default=0.02,
        gt=0.0,
        description=(
            "Empirical: crisis reference for the credit opposition. The "
            "engine divides the credit fragility index "
            "(default_rate * spread_to_treasuries) by this before handing it "
            "to the defines-free catalog, so the accommodation⇄fragility "
            "balance crosses zero exactly AT the threshold. 0.02 is the "
            "2008 reading — corporate bond spread ~6% times default rate "
            "~4% — matching CREDIT_FRAGILITY_THRESHOLD's own derivation."
        ),
    )
```

Regenerate the canonical YAML (never hand-edit it):

```bash
poetry run python tools/generate_defines_config.py
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/config/test_tension_credit_fragility_scale.py tests/unit/config/test_constants_sync.py`

Expected: PASS (both files — the sync guard confirms `defines.yaml` matches the schema)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(config): add tension.credit_fragility_scale define

The credit opposition needs its fragility index expressed in threshold units
before it reaches the defines-free catalog. Default 0.02 carries the 2008
spread-times-default-rate derivation; defines.yaml regenerated.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.7: Fill the four money ratios in `ContradictionSystem._build_graph_inputs`

**Files:**
- Modify: `src/babylon/engine/systems/contradiction.py:36-55` (imports), `:238-301` (`_build_graph_inputs`), and add three private helpers after it
- Test: `tests/unit/engine/systems/test_contradiction_money_inputs.py`

**Interfaces:**
- Consumes: `TICK_DYNAMICS_KEY` and `NATIONAL_FINANCIAL_ATTR` from `babylon.domain.economics.tick.graph_bridge` (`NATIONAL_FINANCIAL_ATTR` is added by U3 and carries `NationalFinancialParameters.model_dump()`); `defines.capital_vol3.credit_fragility_scale` (U5.6); `defines.market.max_abs_log`; the `market` graph attr's `fictitious_log`.
- Produces: populated `GraphInputs.rentier_share | debt_ratio | credit_fragility | financialization_index`.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/engine/systems/test_contradiction_money_inputs.py`:

```python
"""ContradictionSystem fills the four Volume III money ratios (U5).

Three sources, each with an honest-absence path:

- county surplus distributions and debt accumulations, read off
  ``tick_dynamics.county_states`` as a RATIO OF SUMS (never a mean of
  per-county ratios — that is the intensive-aggregation error class);
- the national financial state published under ``national_financial``;
- the scissors' ``fictitious_log``, read in ratio space.
"""

from __future__ import annotations

import math

import pytest

from babylon.domain.economics.distribution.types import (
    DebtAccumulation,
    SurplusValueDistribution,
)
from babylon.domain.economics.dynamics.types import ClassDistribution
from babylon.domain.economics.tick.graph_bridge import TICK_DYNAMICS_KEY
from babylon.domain.economics.tick.types import CountyEconomicState
from babylon.engine.services import ServiceContainer
from babylon.engine.systems.contradiction import ContradictionSystem
from babylon.topology.graph import BabylonGraph

pytestmark = pytest.mark.unit


def _class_distribution(fips: str) -> ClassDistribution:
    return ClassDistribution(
        fips=fips,
        year=2015,
        bourgeoisie_share=0.01,
        petit_bourgeoisie_share=0.09,
        labor_aristocracy_share=0.40,
        proletariat_share=0.35,
        lumpenproletariat_share=0.15,
    )


def _county(
    fips: str,
    *,
    surplus: float,
    interest: float,
    rent: float,
    taxes: float,
    debt: float,
) -> CountyEconomicState:
    return CountyEconomicState(
        fips=fips,
        year=2015,
        capital_stock=1.0e9,
        throughput_position=0.9,
        supply_chain_depth=2.1,
        unemployment_rate=0.05,
        u6_rate=0.10,
        pter_rate=0.04,
        nilf_rate=0.06,
        median_wage=21.0,
        employment=500000.0,
        class_distribution=_class_distribution(fips),
        phi_hour=3.5,
        surplus_distribution=SurplusValueDistribution(
            fips_code=fips,
            year=2015,
            total_surplus_produced=surplus,
            interest_payments=interest,
            ground_rent=rent,
            taxes_on_surplus=taxes,
        ),
        debt_accumulation=DebtAccumulation(
            fips_code=fips,
            year=2015,
            accumulated_debt=debt,
            consecutive_deficit_ticks=0,
        ),
    )


def _inputs(graph: BabylonGraph, services: ServiceContainer):  # type: ignore[no-untyped-def]
    return ContradictionSystem()._build_graph_inputs(graph, services)  # noqa: SLF001


class TestAbsence:
    """A bare graph fabricates nothing (Constitution III.11)."""

    def test_all_four_are_none_on_a_bare_graph(self) -> None:
        inputs = _inputs(BabylonGraph(), ServiceContainer.create())
        assert inputs.rentier_share is None
        assert inputs.debt_ratio is None
        assert inputs.credit_fragility is None
        assert inputs.financialization_index is None

    def test_county_states_without_distributions_read_absent(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr(TICK_DYNAMICS_KEY, {"county_states": {}})
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.rentier_share is None
        assert inputs.debt_ratio is None


class TestCountyRatios:
    def test_rentier_share_is_claims_over_surplus(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {
                "county_states": {
                    "26163": _county(
                        "26163", surplus=100.0, interest=20.0, rent=10.0, taxes=10.0, debt=0.0
                    )
                }
            },
        )
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.rentier_share == pytest.approx(0.4)  # (20 + 10 + 10) / 100

    def test_aggregate_is_a_ratio_of_sums_not_a_mean_of_ratios(self) -> None:
        """The named intensive-aggregation error: a tiny county must NOT swing
        the national reading as hard as a large one. Wayne produces 1000 of
        surplus with 100 of claims (0.1); a 1-surplus county pays 1 in claims
        (1.0). The mean of ratios is 0.55; the truth is 101/1001 = 0.1009."""
        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {
                "county_states": {
                    "26163": _county(
                        "26163", surplus=1000.0, interest=100.0, rent=0.0, taxes=0.0, debt=0.0
                    ),
                    "26001": _county(
                        "26001", surplus=1.0, interest=1.0, rent=0.0, taxes=0.0, debt=0.0
                    ),
                }
            },
        )
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.rentier_share == pytest.approx(101.0 / 1001.0)
        assert inputs.rentier_share != pytest.approx(0.55)

    def test_debt_ratio_is_total_debt_over_total_surplus(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {
                "county_states": {
                    "26163": _county(
                        "26163", surplus=100.0, interest=0.0, rent=0.0, taxes=0.0, debt=75.0
                    ),
                    "26001": _county(
                        "26001", surplus=100.0, interest=0.0, rent=0.0, taxes=0.0, debt=25.0
                    ),
                }
            },
        )
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.debt_ratio == pytest.approx(0.5)  # 100 / 200

    def test_zero_total_surplus_reads_absent_not_infinite(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {
                "county_states": {
                    "26163": _county(
                        "26163", surplus=0.0, interest=0.0, rent=0.0, taxes=0.0, debt=5.0
                    )
                }
            },
        )
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.rentier_share is None
        assert inputs.debt_ratio is None


class TestNationalRatios:
    def test_credit_fragility_arrives_in_threshold_units(self) -> None:
        from babylon.domain.economics.tick.graph_bridge import NATIONAL_FINANCIAL_ATTR

        graph = BabylonGraph()
        graph.set_graph_attr(
            NATIONAL_FINANCIAL_ATTR,
            {"credit_state": {"credit_fragility": 0.04}},
        )
        services = ServiceContainer.create()
        # 0.04 / 0.02 == 2.0: twice the crisis threshold.
        assert _inputs(graph, services).credit_fragility == pytest.approx(2.0)

    def test_missing_credit_state_reads_absent(self) -> None:
        from babylon.domain.economics.tick.graph_bridge import NATIONAL_FINANCIAL_ATTR

        graph = BabylonGraph()
        graph.set_graph_attr(NATIONAL_FINANCIAL_ATTR, {"credit_state": None})
        assert _inputs(graph, ServiceContainer.create()).credit_fragility is None

    def test_financialization_index_is_the_fictitious_axis_in_ratio_space(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr("market", {"price_log": 0.0, "fictitious_log": 0.5})
        inputs = _inputs(graph, ServiceContainer.create())
        assert inputs.financialization_index == pytest.approx(math.exp(0.5))

    def test_parity_axis_reads_index_one(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr("market", {"price_log": 0.0, "fictitious_log": 0.0})
        assert _inputs(graph, ServiceContainer.create()).financialization_index == pytest.approx(
            1.0
        )

    def test_exponent_is_clamped_by_the_axis_bound(self) -> None:
        """A corrupt unbounded log must not raise OverflowError inside the
        tick loop; the axis's own max_abs_log is the clamp."""
        graph = BabylonGraph()
        graph.set_graph_attr("market", {"price_log": 0.0, "fictitious_log": 10_000.0})
        services = ServiceContainer.create()
        expected = math.exp(float(services.defines.market.max_abs_log))
        assert _inputs(graph, services).financialization_index == pytest.approx(expected)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_money_inputs.py`

Expected: FAIL — `assert 0.4 == None` style failures (`inputs.rentier_share is None` in `TestCountyRatios`), because `_build_graph_inputs` never sets the four fields. `TestNationalRatios::test_credit_fragility_arrives_in_threshold_units` fails with `ImportError: cannot import name 'NATIONAL_FINANCIAL_ATTR'` if U3 has not landed — U3 is a prerequisite of this task.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/engine/systems/contradiction.py`, add to the imports (after line 46, `from babylon.formulas.contradiction import calculate_wealth_asymmetry_gap`):

```python
import math

from babylon.domain.economics.distribution.types import (
    DebtAccumulation,
    SurplusValueDistribution,
)
from babylon.domain.economics.tick.graph_bridge import (
    NATIONAL_FINANCIAL_ATTR,
    TICK_DYNAMICS_KEY,
)
```

(Keep `import math` at the top of the stdlib import block, above the `from babylon...` imports, per the file's existing ordering; Ruff's isort will place it.)

Replace the `return GraphInputs(...)` block at lines 292-301 with:

```python
        rentier_share, debt_ratio = self._county_money_ratios(graph)

        financialization_index: float | None = None
        if isinstance(market_raw, dict) and "fictitious_log" in market_raw:
            # The fictitious axis IS a log-ratio around the value anchor, so
            # exp() returns it as the fictitious/real ratio directly — one
            # axis, calibrated to FictitiousCapitalStock.ratio_to_real by the
            # monetary anchor while real data exists, endogenous after 2024.
            # The axis's own bound is the overflow clamp: a log outside it is
            # corrupt input, and raising OverflowError mid-tick helps nobody.
            bound = float(services.defines.market.max_abs_log)
            clamped = max(-bound, min(bound, float(market_raw["fictitious_log"])))
            financialization_index = math.exp(clamped)

        return GraphInputs(
            exploitation_pairs=tuple(exploitation),
            wage_value_pairs=tuple(wage_value),
            tenancy_pairs=tuple(tenancy),
            solidarity_subgraph=extract_solidarity_subgraph(graph),
            exploitation_id_pairs=tuple(exploitation_ids),
            wage_value_id_pairs=tuple(wage_value_ids),
            tenancy_id_pairs=tuple(tenancy_ids),
            market_balance=market_balance,
            rentier_share=rentier_share,
            debt_ratio=debt_ratio,
            credit_fragility=self._credit_fragility(
                graph, float(services.defines.capital_vol3.credit_fragility_scale)
            ),
            financialization_index=financialization_index,
        )
```

Add these two helpers immediately after `_build_graph_inputs`, before the existing `_edge_wealths` staticmethod at line 303:

```python
    @staticmethod
    def _county_money_ratios(graph: GraphProtocol) -> tuple[float | None, float | None]:
        """``(rentier_share, debt_ratio)`` aggregated over the county layer.

        Both are RATIOS OF SUMS — ``Σclaims / Σsurplus`` and
        ``Σdebt / Σsurplus`` — never means of per-county ratios. That
        distinction is the named *intensive-aggregation* error class: an
        unweighted mean of an intensive across space lets a county producing
        one dollar of surplus swing the national reading exactly as hard as
        Wayne.

        Counties are visited in sorted FIPS order so the float summation
        order is fixed (Constitution III.7).

        Returns:
            ``(None, None)`` when no county carries a
            :class:`SurplusValueDistribution`, or when the summed surplus is
            not positive — a ratio with no denominator is absent, not zero
            and not infinite (Constitution III.11). ``debt_ratio`` is
            ``None`` on its own when distributions exist but no county
            carries a :class:`DebtAccumulation`.
        """
        tick_data = graph.get_graph_attr(TICK_DYNAMICS_KEY, None)
        if not isinstance(tick_data, dict):
            return (None, None)
        county_states = tick_data.get("county_states")
        if not isinstance(county_states, dict):
            return (None, None)

        total_surplus = 0.0
        total_claims = 0.0
        total_debt = 0.0
        saw_distribution = False
        saw_debt = False
        for fips in sorted(county_states):  # bounded by the county layer
            county = county_states[fips]
            distribution = getattr(county, "surplus_distribution", None)
            if isinstance(distribution, SurplusValueDistribution):
                saw_distribution = True
                total_surplus += distribution.total_surplus_produced
                total_claims += (
                    distribution.interest_payments
                    + distribution.ground_rent
                    + distribution.taxes_on_surplus
                )
            debt = getattr(county, "debt_accumulation", None)
            if isinstance(debt, DebtAccumulation):
                saw_debt = True
                total_debt += debt.accumulated_debt

        if not saw_distribution or total_surplus <= 0.0:
            return (None, None)
        debt_ratio = total_debt / total_surplus if saw_debt else None
        return (total_claims / total_surplus, debt_ratio)

    @staticmethod
    def _credit_fragility(graph: GraphProtocol, scale: float) -> float | None:
        """National ``default_rate * spread``, divided by its crisis reference.

        Read from the published :data:`NATIONAL_FINANCIAL_ATTR` dump, whose
        ``credit_state`` block carries ``credit_fragility`` as a Pydantic
        computed field. Dividing here (not in the catalog) keeps the catalog
        defines-free — the same division of labour ``market_balance`` uses
        for its ``tanh`` scale — and makes 1.0 mean "exactly at the crisis
        threshold" for the shared ratio map.

        Args:
            graph: The live graph.
            scale: ``defines.capital_vol3.credit_fragility_scale`` (> 0 by schema).

        Returns:
            The scaled fragility, or ``None`` when no national financial
            state, no credit state, or no numeric fragility is published.
        """
        raw = graph.get_graph_attr(NATIONAL_FINANCIAL_ATTR, None)
        if not isinstance(raw, dict):
            return None
        credit_state = raw.get("credit_state")
        if not isinstance(credit_state, dict):
            return None
        fragility = credit_state.get("credit_fragility")
        if not isinstance(fragility, (int, float)) or isinstance(fragility, bool):
            return None
        return float(fragility) / scale
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_money_inputs.py`

Expected: PASS (all classes)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(engine): ContradictionSystem fills the four Vol III money ratios

County layer supplies rentier_share and debt_ratio as ratios of sums (never
means of per-county ratios — the intensive-aggregation error class), the
published national financial state supplies credit fragility in threshold
units, and the scissors' fictitious_log supplies the financialization index in
ratio space. Every source has an honest-absence path; nothing is defaulted.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.8: Make the coupling graph constrain principal-contradiction ranking

**Files:**
- Modify: `src/babylon/engine/systems/contradiction.py:1-34` (module docstring — add the coupling-direction bullet), `:193-200` (inside `_step_registry`, after the shadow split), and add one private method
- Test: `tests/unit/engine/systems/test_contradiction_coupling_rank.py`

**Interfaces:**
- Consumes: `services.coupling_graph` (U5.5), `CouplingGraph.upstream_for(key) -> tuple[Coupling, ...]`, `defines.tension.principal_rate_weight`.
- Produces: the corrected `is_principal` flag seen by `contradiction_frames`, RUPTURE, the regime classifier, and the `opposition_states` stash.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/engine/systems/test_contradiction_coupling_rank.py`:

```python
"""The CouplingGraph's earn-its-keep duty (owner-approved, Vol III U5).

Constitution III.10 forbids shipping a construct as vocabulary, so the
coupling graph gets a job: it constrains principal-contradiction ranking.
A ``transforms`` edge means "the source's output becomes the target's
input" — so a target cannot lead the whole formation while the source that
supplies it reads ABSENT. Crisis has a direction of travel, and the
coupling graph is what knows it.

This is the ONE place the Vol III design alters existing semantics rather
than filling absence, so the contract is pinned in BOTH directions: the
target is demoted when its source is absent, and it ranks completely
normally when the source is present.
"""

from __future__ import annotations

import pytest

from babylon.domain.dialectics.core.coupling import Coupling, CouplingGraph
from babylon.domain.dialectics.core.opposition import (
    BoundOpposition,
    GapReading,
    OppositionRegistry,
    OppositionSpec,
)
from babylon.domain.dialectics.instances.catalog import GraphInputs
from babylon.engine.context import TickContext
from babylon.engine.services import ServiceContainer
from babylon.engine.systems.contradiction import (
    OPPOSITION_STATES_ATTR,
    ContradictionSystem,
)
from babylon.topology.graph import BabylonGraph

pytestmark = pytest.mark.unit


def _binding(key: str, gap: float, balance: float) -> BoundOpposition[GraphInputs]:
    """A binding whose measure is a constant — the ranking is the subject."""
    reading = GapReading(gap=gap, balance=balance)
    return BoundOpposition(
        spec=OppositionSpec(key=key, pole_a=f"{key}-a", pole_b=f"{key}-b"),
        measure=lambda _inputs, _reading=reading: _reading,  # type: ignore[misc]
    )


def _services(
    *,
    source_gap: float,
    source_balance: float,
) -> ServiceContainer:
    """A three-opposition world: source --transforms--> target, plus a rival.

    ``target`` carries the top score (gap 0.9), ``rival`` the runner-up
    (gap 0.6), ``source`` the reading under test.
    """
    registry: OppositionRegistry[GraphInputs] = OppositionRegistry(
        bindings=[
            _binding("source", source_gap, source_balance),
            _binding("target", 0.9, 0.5),
            _binding("rival", 0.6, 0.5),
        ],
        rate_weight=10.0,
    )
    coupling_graph = CouplingGraph(
        [Coupling(source="source", target="target", kind="transforms")],
        registry,
    )
    return ServiceContainer.create(
        opposition_registry=registry,
        coupling_graph=coupling_graph,
    )


def _principal(graph: BabylonGraph) -> str:
    states = graph.get_graph_attr(OPPOSITION_STATES_ATTR, {})
    return next(key for key, dump in states.items() if dump["is_principal"])


class TestTransformsSourceAbsent:
    """(0.0, 0.0) is the catalog's canonical ABSENT reading."""

    def test_target_is_demoted_and_the_runner_up_leads(self) -> None:
        graph = BabylonGraph()
        services = _services(source_gap=0.0, source_balance=0.0)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        assert _principal(graph) == "rival"

    def test_the_demoted_target_carries_is_principal_false(self) -> None:
        graph = BabylonGraph()
        services = _services(source_gap=0.0, source_balance=0.0)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        states = graph.get_graph_attr(OPPOSITION_STATES_ATTR, {})
        assert states["target"]["is_principal"] is False

    def test_the_demotion_is_visible_to_the_frames(self) -> None:
        """Frames are derived AFTER the correction, so the narrative layer
        never announces a principal the engine has demoted."""
        graph = BabylonGraph()
        services = _services(source_gap=0.0, source_balance=0.0)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        frames = graph.get_graph_attr("contradiction_frames", {})
        assert frames["global"]["principal"]["id"] == "rival"

    def test_the_targets_gap_is_untouched_only_its_rank_changes(self) -> None:
        graph = BabylonGraph()
        services = _services(source_gap=0.0, source_balance=0.0)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        states = graph.get_graph_attr(OPPOSITION_STATES_ATTR, {})
        assert states["target"]["gap"] == pytest.approx(0.9)


class TestTransformsSourcePresent:
    """The other direction — the rule must not fire when it should not."""

    def test_target_ranks_principal_normally(self) -> None:
        graph = BabylonGraph()
        services = _services(source_gap=0.2, source_balance=0.1)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        assert _principal(graph) == "target"

    def test_a_zero_gap_with_a_leading_pole_is_present_not_absent(self) -> None:
        """gap 0 with balance −1 is what every Vol III ratio measure returns
        for a claim of zero: a real reading of no claim, NOT missing data.
        Only the (0, 0) pair means absent."""
        graph = BabylonGraph()
        services = _services(source_gap=0.0, source_balance=-1.0)
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        assert _principal(graph) == "target"


class TestRuleIsInertWithoutTheGraph:
    def test_no_coupling_graph_leaves_ranking_untouched(self) -> None:
        registry: OppositionRegistry[GraphInputs] = OppositionRegistry(
            bindings=[
                _binding("source", 0.0, 0.0),
                _binding("target", 0.9, 0.5),
                _binding("rival", 0.6, 0.5),
            ],
            rate_weight=10.0,
        )
        graph = BabylonGraph()
        services = ServiceContainer.create(
            opposition_registry=registry,
            coupling_graph=None,
        )
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        assert _principal(graph) == "target"

    def test_non_transforms_edges_never_demote(self) -> None:
        """``feeds`` and ``constrains`` carry no such prohibition: only
        ``transforms`` means the source's output IS the target's input."""
        registry: OppositionRegistry[GraphInputs] = OppositionRegistry(
            bindings=[
                _binding("source", 0.0, 0.0),
                _binding("target", 0.9, 0.5),
                _binding("rival", 0.6, 0.5),
            ],
            rate_weight=10.0,
        )
        coupling_graph = CouplingGraph(
            [Coupling(source="source", target="target", kind="feeds")],
            registry,
        )
        graph = BabylonGraph()
        services = ServiceContainer.create(
            opposition_registry=registry, coupling_graph=coupling_graph
        )
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        assert _principal(graph) == "target"


class TestEveryCandidateBlocked:
    def test_the_original_principal_is_kept_rather_than_leaving_none(self) -> None:
        """A tick must never end with no principal contradiction; when every
        eligible candidate is blocked the original stands."""
        registry: OppositionRegistry[GraphInputs] = OppositionRegistry(
            bindings=[
                _binding("source", 0.0, 0.0),
                _binding("target", 0.9, 0.5),
            ],
            rate_weight=10.0,
        )
        coupling_graph = CouplingGraph(
            [Coupling(source="source", target="target", kind="transforms")],
            registry,
        )
        graph = BabylonGraph()
        services = ServiceContainer.create(
            opposition_registry=registry, coupling_graph=coupling_graph
        )
        ContradictionSystem().step(graph, services, TickContext(tick=1))
        # source is itself absent (gap 0) so it can never outscore target;
        # target is blocked; the fallback keeps target rather than returning
        # a principal-less tick.
        assert _principal(graph) == "target"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_coupling_rank.py::TestTransformsSourceAbsent`

Expected: FAIL — `assert 'target' == 'rival'` (the system ranks purely by score and never consults the coupling graph).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/engine/systems/contradiction.py`, add this method immediately after `_apply_interventions` (which ends at line 222):

```python
    def _respect_coupling_direction(
        self,
        states: tuple[OppositionState, ...],
        services: ServicesProtocol,
    ) -> tuple[OppositionState, ...]:
        """Forbid a ``transforms`` TARGET from leading while its SOURCE is absent.

        The coupling graph's first production duty (Constitution III.10: no
        construct ships as vocabulary). ``coupling.py`` defines ``transforms``
        as "the source's output becomes the target's input prices" — so a
        target whose input has no reading cannot honestly be the contradiction
        whose development leads all others. Crisis has a direction of travel;
        this is what knows it.

        Absence is read off the measure, not guessed: ``gap == 0.0 AND
        balance == 0.0`` is the catalog's canonical no-data reading (empty
        pair sets, ``None`` market balance, ``None`` money ratio). A real
        reading of *nothing claimed* — gap 0 with the substance pole leading
        at balance −1 — is PRESENT, and does not demote anything.

        Only the principal is re-ranked; no gap, balance, rate or leading
        pole is touched. When every eligible candidate is blocked the
        original principal stands: a tick must never end without one.

        Args:
            states: This tick's canonical (non-shadow) states, already
                intervened.
            services: The container carrying ``coupling_graph`` and
                ``defines.tension.principal_rate_weight``.

        Returns:
            The same states, with at most one ``is_principal`` flag moved.
        """
        coupling_graph = services.coupling_graph
        if coupling_graph is None:
            return states
        principal = next((state for state in states if state.is_principal), None)
        if principal is None:
            return states

        absent = {
            state.key for state in states if state.gap == 0.0 and state.balance == 0.0
        }
        if not absent:
            return states
        blocked = {
            state.key
            for state in states  # bounded by registered bindings
            if any(
                edge.kind == "transforms" and edge.source in absent
                for edge in coupling_graph.upstream_for(state.key)
            )
        }
        if principal.key not in blocked:
            return states

        rate_weight = float(services.defines.tension.principal_rate_weight)
        eligible = sorted(
            (
                state
                for state in states
                if state.key not in blocked and state.key not in absent
            ),
            key=lambda state: (-self._score(state, rate_weight), state.key),
        )
        if not eligible:
            return states
        successor_key = eligible[0].key
        return tuple(
            state.model_copy(update={"is_principal": state.key == successor_key})
            for state in states
        )
```

Then wire it into `_step_registry`. Replace lines 197-200:

```python
        if canonical:
            self._write_frames(graph, services, registry, canonical)
            self._maybe_rupture(services, canonical, tick)
            self._classify_regime(graph, services, registry, canonical, tick)
```

with:

```python
        if canonical:
            # The coupling graph corrects the ranking BEFORE anything reads it,
            # so frames, rupture, the regime classifier and the stash all agree
            # on one principal contradiction.
            canonical = self._respect_coupling_direction(canonical, services)
            self._write_frames(graph, services, registry, canonical)
            self._maybe_rupture(services, canonical, tick)
            self._classify_regime(graph, services, registry, canonical, tick)
```

Finally, add this bullet to the module docstring's numbered list, after item 3 (which ends at line 17 with `principal/secondary = registry principal + runner-up);`):

```
3b. corrects the ranking against the injected ``CouplingGraph``: a
    ``transforms`` target cannot rank principal while the source supplying
    its input reads absent (Vol III money scissors, U5). This runs BEFORE
    frames/rupture/regime so every consumer sees one principal;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_coupling_rank.py tests/unit/engine/systems/test_contradiction_system.py`

Expected: PASS (both files — the existing system tests confirm no regression in unranked-by-coupling worlds)

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(engine): coupling graph constrains principal-contradiction ranking

The CouplingGraph's first production duty (III.10: no construct ships as
vocabulary). A transforms target cannot rank principal while the source whose
output IS its input reads absent — absence being the catalog's canonical
(gap 0, balance 0) pair, never a real zero-claim reading. Pinned in both
directions; inert without an injected graph; the original principal stands
when every candidate is blocked.

This is the one place the Vol III design alters existing semantics, so U8's
delta analysis reports principal-contradiction changes per scenario.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.9: Close the unit — full gate and acceptance evidence

**Files:**
- Test: `tests/unit/dialectics/`, `tests/unit/engine/`, `tests/unit/config/`

**Interfaces:**
- Consumes: everything U5.1–U5.8 produced.
- Produces: the acceptance evidence U8's baseline-delta report cites.

- [ ] **Step 1: Run the dialectics and engine suites together**

Run: `mise run test:q -- tests/unit/dialectics tests/unit/engine/systems/test_contradiction_system.py tests/unit/engine/systems/test_contradiction_money_inputs.py tests/unit/engine/systems/test_contradiction_coupling_rank.py tests/unit/engine/test_services_coupling_graph.py tests/unit/kernel/test_services_protocol.py`

Expected: PASS. If another test asserts a specific principal-contradiction key, it is a genuine U5 consequence (catalog 6 → 10 changes ranking, spec §7 risk 4) — record which test and which key, do NOT weaken the assertion, and carry the observation into U8's delta report.

Then confirm §5 hazard 3 — no new shadow accumulator was introduced. Run:

```bash
rg -n 'persistent_data\[|context\.persistent' src/babylon/engine/systems/contradiction.py src/babylon/engine/systems/market_scissors.py
```

Expected: no NEW key beyond those present on `dev`. Every cross-tick quantity this program adds lives on a specified model field (DebtAccumulation, MarketState), never a shadow attribute (VIII.11).

- [ ] **Step 2: Confirm the registry reports ten keys**

Run: `poetry run python -c "from babylon.domain.dialectics.instances.catalog import build_default_registry as r; print(len(r().keys), r().keys)"`

Expected: `10 ('atomization', 'capital_labor', 'credit', 'debt_spiral', 'financial', 'imperial', 'price_value', 'surplus_distribution', 'tenancy', 'wage')`

- [ ] **Step 3: Confirm both reserved `transforms` edges survive the builder**

Run: `poetry run python -c "import logging; logging.basicConfig(level=logging.INFO); from babylon.domain.dialectics.instances.catalog import build_default_registry as r, build_default_coupling_graph as g; print(sorted((c.source, c.target, c.kind) for c in g(r()).couplings if c.kind == 'transforms'))"`

Expected: `[('credit', 'financial', 'transforms'), ('surplus_distribution', 'debt_spiral', 'transforms')]`, and exactly two `Skipping coupling` INFO lines on stderr, both naming Volume II endpoints (`realization`, `disproportionality`).

- [ ] **Step 4: Run the full fast gate**

Run: `mise run check:quick`

Expected: PASS — lint + format + typecheck green (the `test:unit` leg is deliberately excluded: the full suite is forbidden in this plan; Step 1's scoped run already covers every file U5 touched). Do NOT run `qa:regression` here: baselines are expected to move and U8 owns that ceremony after the owner reads the delta analysis.

- [ ] **Step 5: Record the evidence, then commit.**

Paste Steps 2–3's verbatim stdout into `reports/vol3-baseline-delta.md` under a new "U5 acceptance evidence" subsection (registry key tuple; the two surviving `transforms` triples; the two `Skipping coupling` INFO lines), `git add reports/vol3-baseline-delta.md`, then commit.

```bash
mise run commit -- "test(dialectics): U5 acceptance evidence — registry 10, both reserved edges live

Registry reports ten keys; surplus_distribution->debt_spiral and
credit->financial survive build_default_coupling_graph with only the two
Volume II circulation edges still skipped; the transforms-target ranking rule
holds in both directions. qa:regression is deliberately NOT re-baselined here —
U8 owns the delta analysis and the ceremony commit.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U5.10: Scale `debt_ratio` by `DEBT_SPIRAL_THRESHOLD` so 1.0 IS the spiral threshold

**Files:**
- Modify: `src/babylon/engine/systems/contradiction.py` (`_county_money_ratios` return, `_build_graph_inputs`)
- Modify: `src/babylon/domain/dialectics/instances/catalog.py` (the `debt_spiral` docstring bullet)
- Test: `tests/unit/engine/systems/test_contradiction_money_inputs.py` (append)

**Interfaces:**
- Consumes: `GameDefines.capital_vol3.debt_spiral_threshold` (U2.3) — the
  `DEBT_SPIRAL_THRESHOLD` value, now moddable.
- Produces: `GraphInputs.debt_ratio` in THRESHOLD UNITS, matching the division of
  labour `credit_fragility` (U5.6) and `market_balance` already use: the engine
  owns the scale, the catalog stays defines-free.

- [ ] **Step 1: Write the failing test**
```python
class TestDebtRatioIsInThresholdUnits:
    """§3.6 row 10: DEBT_SPIRAL_THRESHOLD was a dead constant designed as a
    crisis signal and never wired. The debt_spiral opposition's shared ratio
    map crosses balance 0 at x == 1, so 1.0 must MEAN 'at the debt-spiral
    threshold' — otherwise the opposition's unity point is arbitrary."""

    def test_debt_at_the_threshold_reads_exactly_one(self) -> None:
        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {"county_states": {"26163": _county(
                "26163", surplus=100.0, interest=0.0, rent=0.0, taxes=0.0, debt=50.0
            )}},
        )
        services = ServiceContainer.create()
        # raw debt/surplus = 0.5, which IS defines.capital_vol3.debt_spiral_threshold
        assert services.defines.capital_vol3.debt_spiral_threshold == pytest.approx(0.5)
        assert _inputs(graph, services).debt_ratio == pytest.approx(1.0)

    def test_a_modded_threshold_moves_the_unity_point(self) -> None:
        from babylon.config.defines import CapitalVolumeIIIDefines, GameDefines

        graph = BabylonGraph()
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {"county_states": {"26163": _county(
                "26163", surplus=100.0, interest=0.0, rent=0.0, taxes=0.0, debt=50.0
            )}},
        )
        services = ServiceContainer.create(
            defines=GameDefines(
                capital_vol3=CapitalVolumeIIIDefines(debt_spiral_threshold=0.25)
            )
        )
        assert _inputs(graph, services).debt_ratio == pytest.approx(2.0)
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_money_inputs.py::TestDebtRatioIsInThresholdUnits`
Expected: FAIL with `assert 0.5 == 1.0` (the raw ratio is passed through unscaled).
- [ ] **Step 3: Write minimal implementation**

Change `_county_money_ratios`'s signature to accept the scale and apply it:
```python
    @staticmethod
    def _county_money_ratios(
        graph: GraphProtocol, debt_spiral_threshold: float
    ) -> tuple[float | None, float | None]:
```
and its final two lines from:
```python
        debt_ratio = total_debt / total_surplus if saw_debt else None
        return (total_claims / total_surplus, debt_ratio)
```
to:
```python
        # DEBT_SPIRAL_THRESHOLD (§3.6 row 10) was a dead constant designed as a
        # crisis signal. Dividing here — not in the defines-free catalog — makes
        # 1.0 mean "exactly at the debt spiral", so _ratio_reading's balance
        # crosses zero AT the threshold rather than at an arbitrary debt/surplus
        # parity nobody argued for.
        debt_ratio = (
            (total_debt / total_surplus) / debt_spiral_threshold if saw_debt else None
        )
        return (total_claims / total_surplus, debt_ratio)
```
and the call site in `_build_graph_inputs`:
```python
        rentier_share, debt_ratio = self._county_money_ratios(
            graph, float(services.defines.capital_vol3.debt_spiral_threshold)
        )
```
Finally correct the catalog docstring bullet (U5.3) from
`` `debt_spiral` — solvent⇄indebted: accumulated enterprise-profit shortfall over annual surplus. ``
to
`` `debt_spiral` — solvent⇄indebted: accumulated enterprise-profit shortfall over annual surplus, scaled by the engine against `capital_vol3.debt_spiral_threshold` so balance crosses zero AT the spiral threshold. ``
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/engine/systems/test_contradiction_money_inputs.py tests/unit/dialectics/test_catalog.py`
Expected: PASS. Note that U5.2's `test_zero_claim_is_no_contradiction_not_maximal`
(`debt_ratio=0.0` → gap 0, balance −1) is unaffected: 0/0.5 is still 0.
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(engine): scale debt_ratio by DEBT_SPIRAL_THRESHOLD (kills the dead constant)

§3.6 row 10: the threshold was designed as a fifth FinancialCrisisAssessment
signal and had zero references. debt_ratio now reaches the defines-free
catalog in threshold units, so the debt_spiral balance crosses zero AT the
spiral rather than at an unargued debt/surplus parity.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.1: Fix the intensive-aggregation defect in `_mean_profit_rate`

**Files:**
- Modify: `src/babylon/engine/systems/market_scissors.py:411-429`
- Test: `tests/unit/engine/systems/test_market_system.py`

**Interfaces:**
- Consumes: `GraphProtocol.query_nodes(node_type=...)`, `node.attributes["tick_capital_stock"]` (already stamped by `graph_bridge.py:104`), `node.attributes["tick_profit_rate"]`.
- Produces: `_capital_weighted_mean(graph, node_type, attr, *, weight_attr="tick_capital_stock") -> float | None` — a new module-level helper in `market_scissors.py` that U6.6/U6.7 do **not** reuse directly (they use a different helper, `_mean_ratio_to_capital`, added in U6.6) but which fixes `_mean_profit_rate`'s call site inside `_maybe_correct` (unchanged call site, only the body of `_mean_profit_rate` changes).

- [ ] **Step 1: Write the failing test**

Add to `tests/unit/engine/systems/test_market_system.py`, inside `class TestCorrection:` (after `test_healthy_profit_rate_services_the_bubble`):

```python
    def test_capital_weighted_profit_rate_resists_a_tiny_outlier(self) -> None:
        """A 1-unit-capital county's 1.0 profit rate must not out-vote a
        1000-unit-capital county's 0.0. The unweighted mean(1.0, 0.0)=0.5
        would service the 1.5 overhang (0.55 + 4*0.5 = 2.55 > 1.5, no
        snap); the capital-weighted mean drags the aggregate to ~0.001 and
        the snap fires (fixes the intensive-aggregation defect, §3.6 last
        row: the aggregate profit rate is Sum(s)/Sum(c+v), not mean(r_i))."""
        graph = _euphoric_graph(fictitious_log=1.5)
        graph.update_node("metropole", tick_profit_rate=1.0, tick_capital_stock=1.0)
        graph.update_node("hinterland", tick_profit_rate=0.0, tick_capital_stock=1000.0)
        _step(graph, _enabled_services(), tick=10)
        assert graph.graph["market"]["corrections"] == 1
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection::test_capital_weighted_profit_rate_resists_a_tiny_outlier`
Expected: FAIL — `assert 0 == 1` (the unweighted mean of `(1.0, 0.0)` is `0.5`, serviceable divergence is `0.55 + 4*0.5 = 2.55 > 1.5`, so `overhang <= 0.0` and no correction fires; `corrections` stays `0`).

- [ ] **Step 3: Write minimal implementation**

Replace lines 411-429 of `src/babylon/engine/systems/market_scissors.py` (the current `_mean_profit_rate` function) with:

```python
def _capital_weighted_mean(
    graph: GraphProtocol, node_type: str, attr: str, *, weight_attr: str = "tick_capital_stock"
) -> float | None:
    """Capital-weighted mean of an intensive attribute across active nodes.

    The aggregate of a rate/ratio across space is
    ``Sum(value_i * weight_i) / Sum(weight_i)``, never an unweighted mean of
    the per-node ratios — an unweighted mean lets a tiny node swing the
    national reading as hard as a large one (the intensive-aggregation
    class, §3.6/§3.7 of the Vol III money design). A node missing
    ``weight_attr`` (or carrying a non-positive one) contributes weight
    1.0, so fixtures that never stamp capital stock keep their prior
    unweighted reading. Sorted-id iteration fixes the float summation
    order (III.7); ``None`` — never zero — when no active node carries
    ``attr`` (honest absence, III.11).
    """
    weighted_total = 0.0
    weight_total = 0.0
    found = False
    for node in sorted(graph.query_nodes(node_type=node_type), key=lambda n: n.id):
        attrs = node.attributes
        if not attrs.get("active", True):
            continue
        value = attrs.get(attr)
        if not isinstance(value, (int, float)):
            continue
        weight_raw = attrs.get(weight_attr)
        weight = (
            float(weight_raw)
            if isinstance(weight_raw, (int, float)) and weight_raw > 0.0
            else 1.0
        )
        weighted_total += float(value) * weight
        weight_total += weight
        found = True
    return weighted_total / weight_total if found else None


def _mean_profit_rate(graph: GraphProtocol) -> float | None:
    """Capital-weighted mean territory ``tick_profit_rate``, or ``None``.

    The aggregate rate of profit is ``Sum(s) / Sum(c+v)``, not
    ``mean(r_i)`` — an unweighted mean lets a tiny county swing the
    national serviceability line as hard as Wayne. ``tick_capital_stock``
    (``c+v``) is the :func:`_capital_weighted_mean` weight; absence
    returns ``None`` — the serviceability law then falls back to its base
    (no rate is fabricated, III.11).
    """
    return _capital_weighted_mean(graph, "territory", "tick_profit_rate")
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection`
Expected: PASS — all of `TestCorrection` (including the new test and the pre-existing `test_healthy_profit_rate_services_the_bubble`, which is unaffected since it has only one node carrying `tick_profit_rate` and no `tick_capital_stock`, so weight defaults to `1.0` and the result is unchanged: `0.3`).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "fix(market): capital-weight the national profit-rate aggregate

Sum(s)/Sum(c+v), not mean(r_i) — the intensive-aggregation defect (§3.6)
let a tiny county swing the national serviceability threshold as hard
as a large one.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.2: `calculate_serviceable_divergence` gains an interest-burden term

**Files:**
- Modify: `src/babylon/formulas/market.py:108-128`
- Test: `tests/unit/formulas/test_market.py`

**Interfaces:**
- Consumes: nothing new (pure function, no cross-unit dependency).
- Produces: `calculate_serviceable_divergence(profit_rate, *, base, slope, interest_burden=None, interest_slope=0.0) -> float` — the two new keyword args are consumed by Task U6.6 (`MarketScissorsSystem._maybe_correct`), which passes `defines.correction_interest_slope` (added in U6.5) as `interest_slope`.

- [ ] **Step 1: Write the failing test**

Add to `tests/unit/formulas/test_market.py`, after `class TestServiceableDivergence:` (before `class TestOverhang:`):

```python
class TestServiceableDivergenceInterestBurden:
    """U6: a financialised county tightens its own correction threshold
    independent of profit rate (Vol. III part 3 meeting part 5)."""

    def test_interest_burden_tightens_the_threshold(self) -> None:
        healthy = calculate_serviceable_divergence(0.1, base=0.55, slope=4.0)
        tightened = calculate_serviceable_divergence(
            0.1, base=0.55, slope=4.0, interest_burden=0.3, interest_slope=1.0
        )
        assert tightened < healthy
        assert tightened == pytest.approx(0.65)

    def test_absent_interest_burden_is_bit_identical_to_pre_u6(self) -> None:
        assert calculate_serviceable_divergence(
            0.1, base=0.55, slope=4.0, interest_burden=None, interest_slope=1.0
        ) == pytest.approx(0.95)

    def test_zero_interest_slope_is_inert(self) -> None:
        assert calculate_serviceable_divergence(
            0.1, base=0.55, slope=4.0, interest_burden=5.0, interest_slope=0.0
        ) == pytest.approx(0.95)

    def test_floor_at_zero(self) -> None:
        assert calculate_serviceable_divergence(
            0.0, base=0.1, slope=0.0, interest_burden=1.0, interest_slope=1.0
        ) == 0.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/formulas/test_market.py::TestServiceableDivergenceInterestBurden`
Expected: FAIL — `TypeError: calculate_serviceable_divergence() got an unexpected keyword argument 'interest_burden'`.

- [ ] **Step 3: Write minimal implementation**

Replace lines 108-128 of `src/babylon/formulas/market.py` (the current `calculate_serviceable_divergence`) with:

```python
def calculate_serviceable_divergence(
    profit_rate: float | None,
    *,
    base: float,
    slope: float,
    interest_burden: float | None = None,
    interest_slope: float = 0.0,
) -> float:
    """Log fictitious/real divergence the rate of profit can service (ADR078).

    ``base + slope * max(profit_rate, 0) - interest_slope * max(interest_burden, 0)``,
    floored at 0: a healthy rate of profit carries a larger claims
    structure; its FALL is what turns an existing bubble into an unpayable
    one — Vol. III part 3 (the falling rate) meeting part 5 (fictitious
    capital). A financialised county's own interest burden (interest
    payments relative to capital, U6) tightens the same threshold
    independent of the profit rate — a second, orthogonal claim on the
    credit system's tolerance. A loss-making, debt-free economy still
    services the base (the credit system's intrinsic tolerance is a floor,
    not a debt).

    :param profit_rate: Realized rate of profit, or ``None`` when no profit
        observable exists this tick — the base alone is used (honest
        absence, Constitution III.11; no rate is fabricated).
    :param base: Serviceable log-divergence at zero profit and zero
        interest burden (>= 0).
    :param slope: Additional serviceable log-divergence per unit profit rate.
    :param interest_burden: Interest-payments-to-capital ratio, or ``None``
        when no territory carries the accounting pair — the term drops out
        entirely (honest absence, III.11), matching pre-U6 behavior exactly.
    :param interest_slope: Serviceable log-divergence LOST per unit
        interest burden (a ``MarketDefines.correction_interest_slope``
        coefficient).
    :returns: The serviceable log-divergence, floored at 0.
    """
    profit_term = 0.0 if profit_rate is None else slope * max(profit_rate, 0.0)
    interest_term = (
        0.0 if interest_burden is None else interest_slope * max(interest_burden, 0.0)
    )
    return max(base + profit_term - interest_term, 0.0)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/formulas/test_market.py`
Expected: PASS — the new class plus all pre-existing `TestServiceableDivergence` cases (unaffected: `interest_burden` defaults to `None`, contributing `interest_term = 0.0`, reproducing the exact pre-U6 values).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): add an interest-burden term to serviceable divergence

calculate_serviceable_divergence gains optional interest_burden/
interest_slope kwargs, defaulting to a no-op — a financialised county
tightens its own correction threshold independent of the profit rate
(Vol. III part 3 meeting part 5). Not yet wired to a caller (U6.6).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.3: New `calculate_correction_severity` — the accumulated-debt term

**Files:**
- Modify: `src/babylon/formulas/market.py` (append new function at end of file, after `calculate_correction_snap`, currently ending at line 157 pre-U6.2; after Task U6.2's edit the file is longer — locate by the last function in the file, `calculate_correction_snap`, and append immediately after it; also update the `__all__` list at lines 20-28)
- Test: `tests/unit/formulas/test_market.py`

**Interfaces:**
- Consumes: nothing new (pure function).
- Produces: `calculate_correction_severity(base_severity, *, debt_ratio, slope) -> float` — consumed by Task U6.7 (`MarketScissorsSystem._maybe_correct`), which passes `defines.correction_severity` as `base_severity` and `defines.correction_debt_slope` (added U6.5) as `slope`.

- [ ] **Step 1: Write the failing test**

Add to the top-level import in `tests/unit/formulas/test_market.py`:

```python
from babylon.formulas.market import (
    calculate_correction_severity,
    calculate_correction_snap,
    calculate_ema,
    calculate_growth_drive,
    calculate_overhang,
    calculate_scissors_balance,
    calculate_scissors_step,
    calculate_serviceable_divergence,
)
```

Append a new class at the end of the file:

```python
class TestCorrectionSeverity:
    """U6: a debt spiral makes the re-identification of claims with real
    surplus MORE violent — the accumulated-debt term on correction
    severity."""

    def test_absent_debt_ratio_is_the_base_severity(self) -> None:
        assert calculate_correction_severity(0.6, debt_ratio=None, slope=0.5) == 0.6

    def test_debt_ratio_increases_severity(self) -> None:
        assert calculate_correction_severity(0.6, debt_ratio=0.2, slope=0.5) == pytest.approx(0.7)

    def test_severity_clamps_to_one(self) -> None:
        assert calculate_correction_severity(0.6, debt_ratio=5.0, slope=1.0) == 1.0

    def test_negative_debt_ratio_never_reduces_severity(self) -> None:
        assert calculate_correction_severity(0.6, debt_ratio=-1.0, slope=0.5) == 0.6

    def test_zero_slope_is_inert(self) -> None:
        assert calculate_correction_severity(0.6, debt_ratio=10.0, slope=0.0) == 0.6
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/formulas/test_market.py::TestCorrectionSeverity`
Expected: FAIL — `ImportError: cannot import name 'calculate_correction_severity' from 'babylon.formulas.market'`.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/formulas/market.py`, update the `__all__` list (currently lines 20-28) to:

```python
__all__ = [
    "calculate_correction_severity",
    "calculate_correction_snap",
    "calculate_ema",
    "calculate_growth_drive",
    "calculate_overhang",
    "calculate_scissors_balance",
    "calculate_scissors_step",
    "calculate_serviceable_divergence",
]
```

Append at the very end of the file, after `calculate_correction_snap` (two blank lines separating, matching the file's existing top-level spacing):

```python
def calculate_correction_severity(
    base_severity: float, *, debt_ratio: float | None, slope: float
) -> float:
    """Fraction of the fictitious log-ratio closed by one snap, debt-adjusted.

    A debt spiral (U6: accumulated deficit relative to capital) makes the
    violent re-identification of claims with real surplus MORE violent,
    not less — the credit system has less slack to absorb a slow unwind.

    :param base_severity: The ADR078 baseline (``MarketDefines.correction_severity``).
    :param debt_ratio: Accumulated-debt-to-capital ratio, or ``None`` when
        no territory carries the accounting pair — the base is used
        unchanged (honest absence, III.11; bit-identical to pre-U6
        behavior).
    :param slope: Additional severity per unit debt ratio (a
        ``MarketDefines.correction_debt_slope`` coefficient).
    :returns: ``base_severity`` plus the debt term, clamped to ``[0, 1]``
        (:class:`~babylon.models.market.MarketState` cannot express a
        severity outside the unit interval).
    """
    if debt_ratio is None:
        return base_severity
    return min(max(base_severity + slope * max(debt_ratio, 0.0), 0.0), 1.0)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/formulas/test_market.py`
Expected: PASS for every case in the file — `TestCorrectionSeverity` (new) plus every pre-existing class.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): add calculate_correction_severity (accumulated-debt term)

A debt spiral makes the correction snap's re-identification of claims
with real surplus MORE violent — additional severity per unit
debt-to-capital ratio, clamped to [0, 1]. Not yet wired (U6.7).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.4: New `calculate_anchor_pull` — the monetary-anchor drive term

**Files:**
- Modify: `src/babylon/formulas/market.py` (append new function at end of file, after `calculate_correction_severity` added in U6.3)
- Test: `tests/unit/formulas/test_market.py`

**Interfaces:**
- Consumes: nothing new (pure function; `anchor` is a plain `float | None`, already resolved by the caller from `anchor.fictitious_anchor`'s `float | NoDataSentinel` return before this function ever sees it).
- Produces: `calculate_anchor_pull(anchor, current, *, gain) -> float` — consumed by Task U6.8 (`MarketScissorsSystem._advance`), which passes `prior.fictitious_log` as `current` and `defines.anchor_pull` (added U6.5) as `gain`.

- [ ] **Step 1: Write the failing test**

Update the import in `tests/unit/formulas/test_market.py`:

```python
from babylon.formulas.market import (
    calculate_anchor_pull,
    calculate_correction_severity,
    calculate_correction_snap,
    calculate_ema,
    calculate_growth_drive,
    calculate_overhang,
    calculate_scissors_balance,
    calculate_scissors_step,
    calculate_serviceable_divergence,
)
```

Append a new class at the end of the file:

```python
class TestAnchorPull:
    """D1/U6: pulls the fictitious oscillator toward the FRED-grounded
    anchor while real financial data covers this tick; absent anchor is
    inert — the oscillator's endogenous dynamics carry the other ~85% of
    a campaign (§3.3 D1)."""

    def test_absent_anchor_is_zero_drive(self) -> None:
        assert calculate_anchor_pull(None, 0.4, gain=0.3) == 0.0

    def test_pulls_toward_a_higher_anchor(self) -> None:
        assert calculate_anchor_pull(1.0, 0.4, gain=0.3) == pytest.approx(0.18)

    def test_pulls_toward_a_lower_anchor(self) -> None:
        assert calculate_anchor_pull(0.0, 0.4, gain=0.3) == pytest.approx(-0.12)

    def test_zero_gain_is_inert_even_when_anchored(self) -> None:
        assert calculate_anchor_pull(1.0, 0.4, gain=0.0) == 0.0

    def test_at_the_anchor_the_pull_is_zero(self) -> None:
        assert calculate_anchor_pull(0.4, 0.4, gain=0.5) == 0.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/formulas/test_market.py::TestAnchorPull`
Expected: FAIL — `ImportError: cannot import name 'calculate_anchor_pull' from 'babylon.formulas.market'`.

- [ ] **Step 3: Write minimal implementation**

Also add `"calculate_anchor_pull"` to `__all__`, alphabetically first (before `"calculate_correction_severity"`):

```python
# src/babylon/formulas/market.py — add "calculate_anchor_pull" to __all__,
# alphabetically first (before "calculate_correction_severity"):
```

Append at the very end of `src/babylon/formulas/market.py`, after `calculate_correction_severity` (two blank lines separating):

```python
def calculate_anchor_pull(anchor: float | None, current: float, *, gain: float) -> float:
    """Drive term pulling the fictitious log-ratio toward its real-data anchor.

    D1 (owner ruling, 2026-07-18): real FRED data seeds and anchors the
    oscillator where it exists (2010-2024); past the data horizon the
    oscillator's own dynamics ARE the money system. This term is the
    "anchors" half: while a real ratio exists, it exerts a proportional
    pull alongside the drive and reversion terms already in
    :func:`calculate_scissors_step`.

    :param anchor: The log-space target
        (:func:`~babylon.domain.economics.monetary.anchor.fictitious_anchor`
        output, already resolved from ``NoDataSentinel`` to ``None`` by the
        caller), or ``None`` when no real financial data covers this tick —
        the term is then exactly 0.0, leaving the endogenous dynamics
        untouched (honest absence, Constitution III.11).
    :param current: The oscillator's current ``fictitious_log``.
    :param gain: ``MarketDefines.anchor_pull`` — the pull's strength.
    :returns: ``gain * (anchor - current)``, or ``0.0`` when ``anchor`` is
        ``None``.
    """
    if anchor is None:
        return 0.0
    return gain * (anchor - current)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/formulas/test_market.py`
Expected: PASS for the entire file.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): add calculate_anchor_pull (D1 monetary-anchor drive term)

Pure drive term pulling fictitious_log toward the FRED-grounded anchor
when one covers this tick; exactly 0.0 when absent. Not yet wired
(U6.8).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.5: New `MarketDefines` coefficients + regenerate `defines.yaml`

**Files:**
- Modify: `src/babylon/config/defines/market.py:203-213` (append after `wealth_axis_kick_gain`, the last field)
- Modify: `src/babylon/data/defines.yaml:997` (regenerated, not hand-edited — see Step 3)
- Test: `tests/unit/config/test_market_defines.py`

**Interfaces:**
- Consumes: nothing new.
- Produces: `MarketDefines.anchor_pull: float`, `MarketDefines.correction_interest_slope: float`, `MarketDefines.correction_debt_slope: float` — consumed by Tasks U6.6 (`correction_interest_slope`), U6.7 (`correction_debt_slope`), and U6.8 (`anchor_pull`).

- [ ] **Step 1: Write the failing test**

Append to `tests/unit/config/test_market_defines.py` (after `class TestCorrectionLedger:`):

```python
class TestU6Coefficients:
    """U6: the scissors-loop-closing coefficients (interest burden, debt
    spiral, and the D1 monetary anchor pull)."""

    def test_defaults(self) -> None:
        d = MarketDefines()
        assert d.correction_interest_slope == 2.0
        assert d.correction_debt_slope == 0.5
        assert d.anchor_pull == 0.1

    def test_anchor_pull_bounded_to_unit_interval(self) -> None:
        with pytest.raises(ValidationError):
            MarketDefines(anchor_pull=1.5)

    def test_correction_interest_slope_rejects_negative(self) -> None:
        with pytest.raises(ValidationError):
            MarketDefines(correction_interest_slope=-1.0)

    def test_correction_debt_slope_rejects_negative(self) -> None:
        with pytest.raises(ValidationError):
            MarketDefines(correction_debt_slope=-1.0)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/config/test_market_defines.py::TestU6Coefficients`
Expected: FAIL — `AttributeError: 'MarketDefines' object has no attribute 'correction_interest_slope'`.

- [ ] **Step 3: Write minimal implementation**

Append inside the `MarketDefines` class in `src/babylon/config/defines/market.py`, immediately after the `wealth_axis_kick_gain` field (currently the last field, ending at line 213 before the closing of the class):

```python
    anchor_pull: float = Field(
        default=0.1,
        ge=0.0,
        le=1.0,
        description=(
            "Game design: strength of the pull toward fictitious_anchor "
            "(D1) while real FRED-backed financial data anchors this "
            "tick; the pull is exactly 0 when the anchor is absent, "
            "leaving the endogenous dynamics untouched."
        ),
    )
    correction_interest_slope: float = Field(
        default=2.0,
        ge=0.0,
        le=20.0,
        description=(
            "Game design: serviceable log-divergence LOST per unit "
            "interest-burden-to-capital ratio — a financialised county "
            "tightens its own correction threshold independent of profit "
            "rate (Capital Vol. III part 3 meeting part 5)."
        ),
    )
    correction_debt_slope: float = Field(
        default=0.5,
        ge=0.0,
        le=5.0,
        description=(
            "Game design: additional correction severity per unit "
            "accumulated-debt-to-capital ratio — a debt spiral makes the "
            "re-identification of claims with real surplus MORE violent."
        ),
    )
```

Then regenerate the canonical config file:

```bash
poetry run python tools/generate_defines_config.py
```

This appends the three new `market:` lines to `src/babylon/data/defines.yaml` (after `wealth_axis_kick_gain`, currently the last line of that block at line 997) with their descriptions and bounds as inline comments — do not hand-edit the YAML.

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/config/test_market_defines.py`
Expected: PASS for the entire file. Also run `mise run test:q -- tests/unit/config/test_constants_sync.py::TestDefinesYamlSync` to confirm `defines.yaml` stayed in sync with the schema (`test_defines_yaml_in_sync_with_schema`, `test_shipped_defines_yaml_roundtrips_to_defaults`) — expected PASS since the file was regenerated, not hand-edited.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(config): add U6 scissors-loop-closing coefficients to MarketDefines

anchor_pull, correction_interest_slope, correction_debt_slope —
player-editable via defines.yaml, regenerated from the schema.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.6: Wire the interest-burden term into the correction threshold

**Files:**
- Modify: `src/babylon/engine/systems/market_scissors.py` — add the four-line import block (`SurplusValueDistribution`, `serviceability_anchor` (C-5), `NoDataSentinel`, `TICK_DYNAMICS_KEY`), add two module-level helpers, rewrite `_maybe_correct`
- Test: `tests/unit/engine/systems/test_market_system.py` — add four imports (`SurplusValueDistribution`, `ClassDistribution`, `TICK_DYNAMICS_KEY`, `CountyEconomicState`), the module-level `_county_with` helper, and one test

**Interfaces:**
- Consumes: `calculate_serviceable_divergence(..., interest_burden=..., interest_slope=...)` (U6.2), `defines.correction_interest_slope` (U6.5), per-territory `tick_interest_burden` (already stamped — raw interest-payments dollar figure, `graph_bridge.py`'s Feature-024 block) and `tick_capital_stock` (already stamped, `graph_bridge.py:104`); `SurplusValueDistribution` (`babylon.domain.economics.distribution.types:38`; required fields `fips_code` (exactly 5 chars), `year` (ge=2007, le=2040), `total_surplus_produced`, `interest_payments`, `ground_rent`, `taxes_on_surplus`, all `ge=0`); `NoDataSentinel` (`babylon.domain.economics.tensor:45`); `TICK_DYNAMICS_KEY: str = "tick_dynamics"` (`babylon.domain.economics.tick.graph_bridge:38`) and the `"year"` entry the bridge writes into that dict alongside `"county_states"` (`graph_bridge.py:61` — the ONLY tick-year source available to this System; `MarketScissorsSystem` receives a `tick`, never a calendar year).
- Produces: `_mean_ratio_to_capital(graph, numerator_attr) -> float | None` — a new module-level helper reused unchanged by Task U6.7 for the debt term.
- Produces: `_national_serviceability(graph) -> float | None` — the production consumer of `serviceability_anchor`. `_mean_ratio_to_capital(graph, numerator_attr)` is RETAINED and reused by U6.7 for `tick_accumulated_debt`.

> **Note (post-U9):** with the endogenous rate always present, the surplus
> distribution — and hence this interest-burden term — fires far more often;
> the U8.3 delta report must attribute the resulting baseline movement to
> U9, not U6.

- [ ] **Step 1: Write the failing test**

First add the new imports to `tests/unit/engine/systems/test_market_system.py`, directly below the existing `from babylon.config.defines import GameDefines, MarketDefines` line and above `from babylon.engine.context import TickContext` (`babylon.config` < `babylon.domain` < `babylon.engine`; Ruff's isort will confirm):

```python
from babylon.domain.economics.distribution.types import SurplusValueDistribution
from babylon.domain.economics.dynamics.types import ClassDistribution
from babylon.domain.economics.tick.graph_bridge import TICK_DYNAMICS_KEY
from babylon.domain.economics.tick.types import CountyEconomicState
```

Then add the `_county_with` helper at module level, placed directly after `_county_worker` (currently ending around line 216) and before `class TestCountyAxis:`. It is deliberately narrower than U5.7's `_county` in `tests/unit/dialectics/` — that one is a different file, and this task only needs the interest/surplus pair:

```python
def _county_with(*, surplus: float, interest: float) -> CountyEconomicState:
    """A published county state carrying only the surplus/interest pair.

    Every other field is a fixed, valid filler: this fixture exists to feed
    `_national_serviceability`, which reads `surplus_distribution` alone.
    `ClassDistribution` enforces sum-to-one within 0.001, so the five shares
    are not free parameters.
    """
    return CountyEconomicState(
        fips="26163",
        year=2015,
        capital_stock=1.0e9,
        throughput_position=0.9,
        supply_chain_depth=2.1,
        unemployment_rate=0.05,
        u6_rate=0.10,
        pter_rate=0.04,
        nilf_rate=0.06,
        median_wage=21.0,
        employment=500000.0,
        class_distribution=ClassDistribution(
            fips="26163",
            year=2015,
            bourgeoisie_share=0.01,
            petit_bourgeoisie_share=0.09,
            labor_aristocracy_share=0.40,
            proletariat_share=0.35,
            lumpenproletariat_share=0.15,
        ),
        phi_hour=3.5,
        surplus_distribution=SurplusValueDistribution(
            fips_code="26163",
            year=2015,
            total_surplus_produced=surplus,
            interest_payments=interest,
            ground_rent=0.0,
            taxes_on_surplus=0.0,
        ),
    )
```

Then add the test to `tests/unit/engine/systems/test_market_system.py`, inside `class TestCorrection:` (after the test added in Task U6.1):

```python
    def test_interest_burden_tightens_the_threshold_into_a_snap(self) -> None:
        """A healthy 0.3 profit rate alone services the 1.5 overhang
        (0.55 + 4*0.3 = 1.75 > 1.5). A national interest burden of i/s = 0.6
        (serviceability_anchor, §3.3) drops serviceable to 1.75 - 2.0*0.6 =
        0.55 < 1.5 and the snap fires — §3.5 item 1.

        `year` is written alongside `county_states` because that is exactly
        what `write_tick_state_to_graph` publishes (graph_bridge.py:61); the
        aggregate distribution is stamped with the REAL tick year, never an
        invented constant.
        """
        graph = _euphoric_graph(fictitious_log=1.5)
        graph.update_node("metropole", tick_profit_rate=0.3, tick_capital_stock=10.0)
        graph.set_graph_attr(
            TICK_DYNAMICS_KEY,
            {
                "year": 2015,
                "county_states": {"26163": _county_with(surplus=100.0, interest=60.0)},
            },
        )
        _step(graph, _enabled_services(), tick=10)
        assert graph.graph["market"]["corrections"] == 1
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection::test_interest_burden_tightens_the_threshold_into_a_snap`
Expected: FAIL — `assert 0 == 1` (serviceable is still `0.55 + 4*0.3 = 1.75 > 1.5`; the interest burden is not yet read, so no correction fires).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/engine/systems/market_scissors.py`, add this import block in full, placed directly above the existing `from babylon.engine.systems.wealth_distribution import (` line (`babylon.domain` sorts before `babylon.engine`; within it `distribution` < `monetary` < `tensor` < `tick`; Ruff's isort will confirm). All four symbols are used by the two helpers below — none of them is currently imported by this module:

```python
from babylon.domain.economics.distribution.types import SurplusValueDistribution
from babylon.domain.economics.monetary import serviceability_anchor
from babylon.domain.economics.tensor import NoDataSentinel
from babylon.domain.economics.tick.graph_bridge import TICK_DYNAMICS_KEY
```

(`NoDataSentinel` lands HERE, not in Task U6.8 — U6.8's import block therefore no longer adds it. All four are `domain` imports from `engine`, which the layering rule permits; none of them imports `engine` back, so there is no cycle.)

Add a new module-level helper, placed directly after `_mean_profit_rate` (added in Task U6.1):

```python
def _mean_ratio_to_capital(graph: GraphProtocol, numerator_attr: str) -> float | None:
    """Exact ``Sum(numerator) / Sum(tick_capital_stock)`` over active territories.

    Deliberately NOT :func:`_capital_weighted_mean`: that helper defaults a
    missing weight to 1.0 so weightless fixtures keep their prior unweighted
    reading, whereas this one SKIPS a territory missing either attribute (the
    ratio is undefined without capital). Two different absence policies, two
    helpers — do not merge them without re-pinning both call sites' fixtures.

    Weighting each territory's own ``numerator/capital`` ratio by its
    capital stock telescopes algebraically to this sum-of-sums — the exact
    national aggregate, not an approximation, and dimensionally consistent
    with ``tick_profit_rate`` (also normalized to ``c+v``). Territories
    missing either attribute, or carrying non-positive capital (the ratio
    is undefined), are skipped — honest absence (III.11), never a
    fabricated zero. Sorted-id iteration fixes the float summation order
    (III.7).
    """
    numerator_total = 0.0
    capital_total = 0.0
    for node in sorted(graph.query_nodes(node_type="territory"), key=lambda n: n.id):
        attrs = node.attributes
        if not attrs.get("active", True):
            continue
        numerator = attrs.get(numerator_attr)
        capital = attrs.get("tick_capital_stock")
        if not isinstance(numerator, (int, float)) or not isinstance(capital, (int, float)):
            continue
        if capital <= 0.0:
            continue
        numerator_total += float(numerator)
        capital_total += float(capital)
    return numerator_total / capital_total if capital_total > 0.0 else None
```

```python
def _national_serviceability(graph: GraphProtocol) -> float | None:
    """National interest burden ``Σi / Σs``, via ``serviceability_anchor``.

    Aggregated as a RATIO OF SUMS over the published county distributions —
    never a mean of per-county burdens (the intensive-aggregation class).
    ``serviceability_anchor`` is applied to the aggregated distribution so the
    honest-absence contract (§3.3 clause 1) is the single source of the
    absent/present decision; a ``NoDataSentinel`` resolves to ``None`` and the
    interest term drops out of ``calculate_serviceable_divergence`` entirely.

    The aggregate is stamped with the REAL tick year, read from the same
    ``tick_dynamics["year"]`` entry the bridge writes alongside
    ``county_states``. No invented constant: a hardcoded year would be an
    inline coefficient with no material referent (Aleksandrov Test), and a
    wrong one would silently mislabel every sentinel this function emits.
    A missing or non-integer year is honest absence, not a default.
    """
    tick_data = graph.get_graph_attr(TICK_DYNAMICS_KEY, None)
    if not isinstance(tick_data, dict):
        return None
    year = tick_data.get("year")
    if not isinstance(year, int) or isinstance(year, bool):
        return None
    county_states = tick_data.get("county_states")
    if not isinstance(county_states, dict):
        return None
    total_surplus = 0.0
    total_interest = 0.0
    saw_any = False
    for fips in sorted(county_states):  # sorted: fixes float summation order (III.7)
        distribution = getattr(county_states[fips], "surplus_distribution", None)
        if distribution is None:
            continue
        saw_any = True
        total_surplus += distribution.total_surplus_produced
        total_interest += distribution.interest_payments
    if not saw_any:
        return None
    aggregate = SurplusValueDistribution(
        # "00000" is not a county: it is the reserved national-aggregate FIPS.
        # `monetary.anchor.NATIONAL_FIPS` ("USA") cannot be used here —
        # SurplusValueDistribution.fips_code is min_length=max_length=5.
        fips_code="00000",
        year=year,
        total_surplus_produced=total_surplus,
        interest_payments=total_interest,
        ground_rent=0.0,
        taxes_on_surplus=0.0,
    )
    result = serviceability_anchor(aggregate)
    return None if isinstance(result, NoDataSentinel) else float(result)
```

In `_maybe_correct` (the method's opening lines, currently starting around line 307), replace:

```python
        profit_rate = _mean_profit_rate(graph)
        serviceable = calculate_serviceable_divergence(
            profit_rate,
            base=defines.correction_threshold_base,
            slope=defines.correction_profit_slope,
        )
```

with:

```python
        profit_rate = _mean_profit_rate(graph)
        # §3.3/§3.4: the interest burden is `i / s` from SurplusValueDistribution
        # — the share of produced surplus already spoken for — NOT interest over
        # capital stock. serviceability_anchor is the module that computes it,
        # and this is its production consumer (the `surplus_distribution
        # constrains financial` edge's declared grounding).
        interest_burden = _national_serviceability(graph)
        serviceable = calculate_serviceable_divergence(
            profit_rate,
            base=defines.correction_threshold_base,
            slope=defines.correction_profit_slope,
            interest_burden=interest_burden,
            interest_slope=defines.correction_interest_slope,
        )
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection`
Expected: PASS for the entire class, including the pre-existing `test_healthy_profit_rate_services_the_bubble` (unaffected: those graphs publish no `tick_dynamics` attribute at all, so `_national_serviceability` returns `None` and the term drops out).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): wire the interest-burden term into the correction threshold

The national interest burden i/s — the share of produced surplus already
spoken for, aggregated as a ratio of sums over the published county
SurplusValueDistributions and resolved by serviceability_anchor — tightens
the serviceable divergence independent of the profit rate. Not interest
over capital stock: the claim is on surplus, not on capital.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.7: Wire the accumulated-debt term into correction severity

**Files:**
- Modify: `src/babylon/engine/systems/market_scissors.py:34-41` (import block), the `_maybe_correct` method (as modified by Task U6.6)
- Test: `tests/unit/engine/systems/test_market_system.py`

**Interfaces:**
- Consumes: `calculate_correction_severity` (U6.2/U6.3), `defines.correction_debt_slope` (U6.5), `_mean_ratio_to_capital` (U6.6), per-territory `tick_accumulated_debt` (already stamped, `graph_bridge.py`'s Feature-024 block) and `tick_capital_stock`.
- Produces: nothing new for later units — this closes item 2 of §3.5.

- [ ] **Step 1: Write the failing test**

Add to `tests/unit/engine/systems/test_market_system.py`, inside `class TestCorrection:` (after the test added in Task U6.6):

```python
    def test_accumulated_debt_deepens_the_snap(self) -> None:
        """A debt-free bubble snaps at the base severity (0.6 — see
        test_overhang_fires_the_snap). The SAME overhang, SAME capital
        stock, carrying a debt-to-capital ratio of 1.0 closes MORE of the
        fictitious log-ratio in the same single snap (§3.5 item 2). Capital
        stock is held constant across both arms so the only varying input
        is debt — otherwise the U6.1 capital-weighted profit aggregate and
        the C-5 serviceability denominator move too, and the test would not
        isolate severity."""
        debt_free = _euphoric_graph()
        debt_free.update_node("metropole", tick_capital_stock=10.0)
        _step(debt_free, _enabled_services(), tick=10)
        debt_free_after = debt_free.graph["market"]["fictitious_log"]

        indebted = _euphoric_graph()
        indebted.update_node("metropole", tick_capital_stock=10.0, tick_accumulated_debt=10.0)
        _step(indebted, _enabled_services(), tick=10)
        indebted_after = indebted.graph["market"]["fictitious_log"]

        assert debt_free_after == pytest.approx(0.6)  # base severity 0.6 on a 1.5 overhang
        assert indebted_after < debt_free_after
        assert indebted_after == pytest.approx(0.0)  # severity clamps to 1.0: full wipeout
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection::test_accumulated_debt_deepens_the_snap`
Expected: FAIL — `assert 0.6 == pytest.approx(0.0)` (both runs snap at the base severity 0.6 since the debt ratio is not yet read; `1.5 * (1 - 0.6) = 0.6` in both cases, so `indebted_after < debt_free_after` is also false — `0.6 < 0.6` fails).

- [ ] **Step 3: Write minimal implementation**

Update the `formulas.market` import block in `src/babylon/engine/systems/market_scissors.py` to:

```python
from babylon.formulas.market import (
    calculate_correction_severity,
    calculate_correction_snap,
    calculate_ema,
    calculate_growth_drive,
    calculate_overhang,
    calculate_scissors_step,
    calculate_serviceable_divergence,
)
```

In `_maybe_correct`, locate the block (as left by Task U6.6):

```python
        fictitious_log, fictitious_velocity = calculate_correction_snap(
            state.fictitious_log,
            state.fictitious_velocity,
            severity=defines.correction_severity,
        )
        price_log, price_velocity = calculate_correction_snap(
            state.price_log,
            state.price_velocity,
            severity=defines.correction_price_severity,
        )
```

and replace it with:

```python
        debt_ratio = _mean_ratio_to_capital(graph, "tick_accumulated_debt")
        severity = calculate_correction_severity(
            defines.correction_severity, debt_ratio=debt_ratio, slope=defines.correction_debt_slope
        )
        fictitious_log, fictitious_velocity = calculate_correction_snap(
            state.fictitious_log,
            state.fictitious_velocity,
            severity=severity,
        )
        price_log, price_velocity = calculate_correction_snap(
            state.price_log,
            state.price_velocity,
            severity=defines.correction_price_severity,
        )
```

(Only the fictitious-capital snap's severity is debt-adjusted — the price snap keeps `defines.correction_price_severity` unchanged; the debt spiral is a claims-side phenomenon, not a general price-deflation one.)

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestCorrection`
Expected: PASS for the entire class, including `test_correction_is_deterministic` and every other pre-existing case in `TestCorrection` (unaffected: `_euphoric_graph()`'s default territories carry no `tick_accumulated_debt`, so `debt_ratio` is `None` and `calculate_correction_severity` returns `defines.correction_severity` unchanged — bit-identical to pre-U6.7 behavior).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): wire the accumulated-debt term into correction severity

A debt spiral (tick_accumulated_debt / tick_capital_stock, exactly
aggregated) deepens the fictitious-capital snap's severity, clamped
to [0, 1]. Closes §3.5 item 2.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U6.8: Wire the D1 monetary anchor into the fictitious oscillator

**Files:**
- Modify: `src/babylon/engine/systems/market_scissors.py:26-52` (module header + imports), the `step` method (currently lines 133-166), the `_advance` staticmethod (currently lines 233-283)
- Test: `tests/unit/engine/systems/test_market_system.py`

**Interfaces:**
- Consumes: `NATIONAL_FINANCIAL_ATTR: Final[str] = "national_financial"` (U3, `babylon.domain.economics.tick.graph_bridge`), `NationalFinancialParameters` (already exists, `babylon.domain.economics.tick.types`), `fictitious_anchor(stock: FictitiousCapitalStock | None, real_output: float | None) -> float | NoDataSentinel` (U4, `babylon.domain.economics.monetary.anchor`), `NoDataSentinel` (`babylon.domain.economics.tensor`), `calculate_anchor_pull` (U6.4), `defines.anchor_pull` (U6.5).
- Produces: `_read_fictitious_anchor(metadata, real_output) -> float | None` — module-level helper, used only by `step()`. `MarketScissorsSystem._advance(..., anchor: float | None = None)` — the new keyword gets a default so `_step_county_axes`'s existing calls (which must **not** receive the anchor — no per-territory `fictitious_log` projection, out of scope per §6) continue to work unmodified.

- [ ] **Step 0: Measure the pre-wiring pin values (do this BEFORE writing any test)**

```bash
PYTHONPATH="$PWD/src" poetry run python -c "
from babylon.config.defines.market import MarketDefines
from babylon.models.market import MarketState
from babylon.engine.systems.market_scissors import MarketScissorsSystem
prior = MarketState(price_log=0.12, price_velocity=-0.01, fictitious_log=0.4,
                    fictitious_velocity=0.02, surplus_ema=3.5, value_ema=11.0, tick=42)
r = MarketScissorsSystem._advance(prior, 4.0, 12.0, MarketDefines(), 43)
for f in ('price_log','price_velocity','fictitious_log','fictitious_velocity','surplus_ema','value_ema'):
    print(f'        assert result.{f} == {getattr(r, f)!r}')
"
```
Paste the six printed lines verbatim into `TestAnchorAbsentIsBitIdentical`. **Do not hand-type them and do not accept the literals as printed in this plan** — they were authored, not measured. If any differs, the CAPTURED value is authoritative; do not adjust the implementation to match the plan.

- [ ] **Step 1: Write the failing test**

Add the new imports to the top of `tests/unit/engine/systems/test_market_system.py` (extend the existing import block):

```python
from babylon.domain.economics.credit.types import FictitiousCapitalStock
from babylon.domain.economics.tick.graph_bridge import NATIONAL_FINANCIAL_ATTR
from babylon.domain.economics.tick.types import NationalFinancialParameters
```

Add two new test classes at the end of `tests/unit/engine/systems/test_market_system.py`:

```python
class TestAnchorAbsentIsBitIdentical:
    """U6/D1: absence of NATIONAL_FINANCIAL_ATTR must reproduce the exact
    pre-U6.8 trajectory — captured before the anchor-pull wiring landed."""

    def test_advance_without_an_anchor_matches_the_pre_u6_pin(self) -> None:
        defines = MarketDefines()
        prior = MarketState(
            price_log=0.12,
            price_velocity=-0.01,
            fictitious_log=0.4,
            fictitious_velocity=0.02,
            surplus_ema=3.5,
            value_ema=11.0,
            tick=42,
        )
        result = MarketScissorsSystem._advance(
            prior, surplus=4.0, value=12.0, defines=defines, tick=43, anchor=None
        )
        assert result.price_log == 0.16364545454545454
        assert result.price_velocity == 0.04364545454545454
        assert result.fictitious_log == 0.5643941558441559
        assert result.fictitious_velocity == 0.16439415584415584
        assert result.surplus_ema == 3.575
        assert result.value_ema == 11.149999999999999


class TestAnchorPresentPullsTheOscillator:
    """D1: real FRED data anchors the fictitious oscillator while it
    covers this tick (§3.5 item 3)."""

    @staticmethod
    def _run(publish_anchor: bool) -> float:
        graph = BabylonGraph()
        _paid_worker(graph, "w1", w_paid=0.8, v_produced=1.0)
        services = ServiceContainer.create()
        _step(graph, services, tick=1)  # seed at zero — no anchor read on the seed tick
        if publish_anchor:
            graph.graph[NATIONAL_FINANCIAL_ATTR] = NationalFinancialParameters(
                fictitious_capital=FictitiousCapitalStock(
                    year=2020,
                    government_debt=3.0,
                    corporate_equity=3.0,
                    corporate_debt=3.0,
                    household_debt=3.0,
                ),
            ).model_dump()
        graph.update_node("w1", v_produced=1.0, w_paid=0.8)
        _step(graph, services, tick=2)
        return float(graph.graph["market"]["fictitious_log"])

    def test_anchor_pulls_the_fictitious_log_upward(self) -> None:
        anchored = self._run(publish_anchor=True)
        unanchored = self._run(publish_anchor=False)
        assert unanchored == 0.0
        assert anchored > unanchored

    def test_missing_national_financial_key_is_inert(self) -> None:
        """No NATIONAL_FINANCIAL_ATTR at all (U3 has not run this tick, or
        the graph carries no financial layer) — same as an explicit
        anchor=None, never an error."""
        assert self._run(publish_anchor=False) == 0.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestAnchorAbsentIsBitIdentical`
Expected: FAIL with `TypeError: _advance() got an unexpected keyword argument 'anchor'` — the six float assertions are NOT exercised on this run; they only become live after Step 3.

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py::TestAnchorPresentPullsTheOscillator::test_anchor_pulls_the_fictitious_log_upward`
Expected: FAIL — `assert 0.0 > 0.0` (with `NATIONAL_FINANCIAL_ATTR` not yet read by `step()`, the anchored and unanchored runs are identical: both `0.0`).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/engine/systems/market_scissors.py`, update the top-of-file imports (currently lines 26-52) to add:

```python
from babylon.domain.economics.monetary import fictitious_anchor
from babylon.domain.economics.tick.graph_bridge import NATIONAL_FINANCIAL_ATTR
from babylon.domain.economics.tick.types import NationalFinancialParameters
```

`NoDataSentinel` is NOT re-added here — Task U6.6 already imported it into this module for `_national_serviceability`, and `fictitious_anchor` reuses that same binding. Merge `fictitious_anchor` into U6.6's existing `from babylon.domain.economics.monetary import serviceability_anchor` line (making it `from babylon.domain.economics.monetary import fictitious_anchor, serviceability_anchor`) and `NATIONAL_FINANCIAL_ATTR` into U6.6's existing `from babylon.domain.economics.tick.graph_bridge import TICK_DYNAMICS_KEY` line, rather than writing duplicate `from` statements; the remaining new line is placed alphabetically before the existing `from babylon.engine.systems.wealth_distribution import (...)` line, and add `calculate_anchor_pull` to the existing `formulas.market` import block:

```python
from babylon.formulas.market import (
    calculate_anchor_pull,
    calculate_correction_severity,
    calculate_correction_snap,
    calculate_ema,
    calculate_growth_drive,
    calculate_overhang,
    calculate_scissors_step,
    calculate_serviceable_divergence,
)
```

Add a new module-level helper, placed directly after `_aggregate_wage_value_by_county` and before `class MarketScissorsSystem:`:

```python
def _read_fictitious_anchor(metadata: dict[str, object], real_output: float) -> float | None:
    """Resolve the D1 monetary anchor from ``NATIONAL_FINANCIAL_ATTR``.

    ``None`` — never a fabricated pull — when the key is absent (no
    national financial state published this tick; U3 has not run, or this
    graph carries no financial layer at all) or when
    :func:`~babylon.domain.economics.monetary.fictitious_anchor`
    itself returns :class:`~babylon.domain.economics.tensor.NoDataSentinel`
    (no real FRED coverage past the 2024 data horizon, D1). The
    oscillator's endogenous dynamics then carry the tick unchanged
    (Constitution III.11).
    """
    raw = metadata.get(NATIONAL_FINANCIAL_ATTR)
    if not isinstance(raw, dict):
        return None
    national_financial = NationalFinancialParameters.model_validate(raw)
    result = fictitious_anchor(national_financial.fictitious_capital, real_output)
    if isinstance(result, NoDataSentinel):
        return None
    return float(result)
```

Replace the body of `step()` (currently lines 133-166) with:

```python
    def step(
        self,
        graph: GraphProtocol,
        services: ServicesProtocol,
        context: ContextType,
    ) -> None:
        """Seed (first observation) or advance the scissors oscillators."""
        defines = services.defines.market
        tick = context.tick
        metadata = getattr(graph, "graph", None)
        if not isinstance(metadata, dict):  # pragma: no cover — BabylonGraph always has it
            return
        flow = _aggregate_wage_value(graph)
        if flow is None:
            return  # no value substrate, no phenomenal form (III.11)
        wages, value = flow
        surplus = max(value - wages, 0.0)
        anchor = _read_fictitious_anchor(metadata, value)
        prior_raw = metadata.get(MARKET_ATTR)
        if isinstance(prior_raw, dict):
            state = self._advance(
                MarketState(**prior_raw), surplus, value, defines, int(tick), anchor
            )
        else:
            state = MarketState(
                price_log=0.0,
                price_velocity=0.0,
                fictitious_log=0.0,
                fictitious_velocity=0.0,
                surplus_ema=surplus,
                value_ema=value,
                tick=int(tick),
            )
        if defines.feedback_enabled:
            state = self._maybe_correct(graph, services, state, defines, int(tick))
        metadata[MARKET_ATTR] = state.model_dump()
        self._step_county_axes(graph, metadata, defines, int(tick))
```

Replace the `_advance` staticmethod (currently lines 233-283) with:

```python
    @staticmethod
    def _advance(
        prior: MarketState,
        surplus: float,
        value: float,
        defines: MarketDefines,
        tick: int,
        anchor: float | None = None,
    ) -> MarketState:
        """One deterministic oscillator step of both scissors.

        Prices chase value-output growth (demand pull) against the law-of-
        value reversion; fictitious capitalization chases realized-surplus
        growth PLUS price momentum (speculation rides the boom) PLUS a
        pull toward the real-data anchor when one covers this tick (D1,
        U6) against a weaker gravity — bubbles outlive price swings.
        ``anchor`` is ``None`` for the county axes (:meth:`_step_county_axes`
        never passes one — no per-territory ``fictitious_log`` projection,
        out of scope per §6 of the design).
        """
        price_drive = calculate_growth_drive(
            value, prior.value_ema, sensitivity=defines.price_drive_sensitivity
        )
        price_log, price_velocity = calculate_scissors_step(
            prior.price_log,
            prior.price_velocity,
            price_drive,
            reversion=defines.price_reversion,
            damping=defines.price_damping,
            max_abs_log=defines.max_abs_log,
        )
        fictitious_drive = (
            calculate_growth_drive(
                surplus, prior.surplus_ema, sensitivity=defines.fictitious_drive_sensitivity
            )
            + defines.momentum_coupling * price_velocity
            + calculate_anchor_pull(anchor, prior.fictitious_log, gain=defines.anchor_pull)
        )
        fictitious_log, fictitious_velocity = calculate_scissors_step(
            prior.fictitious_log,
            prior.fictitious_velocity,
            fictitious_drive,
            reversion=defines.fictitious_reversion,
            damping=defines.fictitious_damping,
            max_abs_log=defines.max_abs_log,
        )
        return MarketState(
            price_log=price_log,
            price_velocity=price_velocity,
            fictitious_log=fictitious_log,
            fictitious_velocity=fictitious_velocity,
            surplus_ema=calculate_ema(prior.surplus_ema, surplus, alpha=defines.surplus_ema_alpha),
            value_ema=calculate_ema(prior.value_ema, value, alpha=defines.surplus_ema_alpha),
            tick=tick,
            corrections=prior.corrections,
            last_correction_tick=prior.last_correction_tick,
        )
```

Do **not** modify `_step_county_axes` — its existing calls to `self._advance(MarketState(**prior), surplus, value, defines, tick)` (5 positional args) remain valid unchanged, `anchor` defaulting to `None`.

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/engine/systems/test_market_system.py`
Expected: PASS for the entire file — the two new classes, and every pre-existing test in `TestDynamics`, `TestCorrection`, `TestCountyAxis`, etc. (all call `step()`/`_advance()` on graphs with no `NATIONAL_FINANCIAL_ATTR` key, so `anchor` resolves to `None` throughout and every prior assertion holds bit-identically).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(market): pull the fictitious oscillator toward the D1 monetary anchor

_advance reads NATIONAL_FINANCIAL_ATTR (U3) and, when
fictitious_anchor (U4) resolves a real value, adds a proportional pull
term to the fictitious drive; NoDataSentinel or an absent key leaves
dynamics bit-identical to pre-U6.8 (pinned exactly). County axes never
receive the anchor — no per-territory projection, out of scope.
Closes §3.5 item 3 and the U6 acceptance criteria.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Unit U9 — Endogenous interest rate (Capital Vol. III Part V) — owner ruling 2026-07-19

### §1. Design — the endogenous interest mechanic

### §1.1 Theory grounding (Aleksandrov Test: every construct traces to a cited material relation)

All quotations verbatim from the Marxists.org mirror of *Capital* Vol. III, Part V
("Division of Profit into Interest and Profit of Enterprise. Interest-Bearing
Capital"). Local copies present in this environment: `ch24.htm`, `ch25.htm`
(`/media/user/data/old-hdd/old-hdd/www.marxists.org/archive/marx/works/1894-c3/`).
`ch22.htm`/`ch23.htm` are absent from the local partial mirror and were quoted from
the same source online; the two load-bearing claims that ground the *shape* of the
formula (interest = a portion of profit; the crisis cycle) are quoted from the
**locally-present** `ch24`/`ch25`.

**(a) There is no natural rate of interest — the division of profit into interest
and profit-of-enterprise is purely empirical (competition, not a law).** — Ch. 22
("Division of Profit. Rate of Interest. Natural Rate of Interest"):

> "There is no such thing as a 'natural' rate of interest."

> "Wherever it is competition as such which determines anything, the determination
> is accidental, purely empirical, and only pedantry or fantasy would seek to
> represent this accident as a necessity."

*Design consequence:* we do **not** read a rate off a curve. The rate has no
natural level to look up; it must be *computed* from the material magnitudes that
competition works on. The "empirical" baseline division is therefore a **moddable
`GameDefines` coefficient** (`interest_profit_share_base`), never a hardcoded law
(III.1). This is the deepest reason the owner ruling is theory-correct, not merely
convenient: an absent-rate fallback presupposes a natural rate exists to be
missing; Marx denies there is one.

**(b) Interest is only a share of the profit; its maximum is the profit itself
(profit-of-enterprise → 0), its minimum is indeterminable (→ any low).** — Ch. 24
("Externalisation of the Relations of Capital…", present locally):

> "While interest is only a portion of the profit, i.e., of the surplus-value,
> which the functioning capitalist squeezes out of the labourer, it appears now, on
> the contrary, as though interest were the typical product of capital…"

Ch. 23 ("Interest and Profit of Enterprise"):

> "Interest … appears originally, is originally, and remains in fact merely a
> portion of the profit, i.e., of the surplus-value."

Ch. 22, on the limits:

> "the maximum limit of interest is the profit itself, in which case the portion
> pocketed by the productive capitalist would = 0."

> "The minimum limit of interest is altogether indeterminable. It may fall to any
> low."

*Design consequence:* the interest **rate** `i` is bounded by the economy-wide
**average rate of profit** `r`: `0 ≤ i < r`. Equality `i = r` is the theoretical
crisis maximum (profit-of-enterprise = 0); the sim reserves a strict ε so the
profit-of-enterprise residual of the surplus identity `s = p + i + r_rent + t`
stays positive as the normal case, i.e. `i ≤ r·ceiling < r`. When `r ≤ 0` there is
no profit to divide, so `i = 0` **by construction** — not a fabricated positive
floor (this is exactly the "no natural rate to fall back on" discipline).

**(c) The rate is set by the supply of, and demand for, loanable money-capital —
competition between money-capitalists and functioning capitalists.** — Ch. 22:

> "in the money-market all loanable capital continually faces functioning capital
> as an aggregate mass, so that the relation between the supply of loanable capital
> on one side, and the demand for it on the other, decides the market level of
> interest at any given time."

Ch. 23:

> "it is only the competition between these two kinds of capitalists which creates
> the rate of interest."

Ch. 25 ("Credit and Fictitious Capital", present locally), on the *supply* side —
idle money-capital aggregating into a lendable mass:

> "money savings and the temporarily idle money of all classes were deposited with
> them. Small amounts, each in itself incapable of acting in the capacity of
> money-capital, merge together into large masses and thus form a money power."

*Design consequence:* where `i` sits **within** `[0, r)` is set by a **loan-market
tightness** `τ ∈ [0,1]` — an explicit *demand − supply* balance of loanable
money-capital. Base U9 fills the **demand** side from a material, always-present
crisis observable (the reserve-army surge — the material signature of the
overproduction crisis that ignites the scramble for means of payment); the
**supply** side (idle money-capital) has no distinct graph quantity yet and is
**explicitly deferred** with its creating task and material source named (§1.5).

**(d) The cyclical movement: interest is low in prosperity, rises at the peak,
SPIKES in the crisis, collapses in the ensuing stagnation.** — Ch. 25 (Engels'
empirical illustration of 1844–47, present locally):

> "The bank discount rate stood low: 1¾ to 2¾% in 1844, less than 3% until October
> 1845, rising to 5% for a while (February 1846), then dropping again to 3.25% in
> December 1846."

> "The banking discount rate, still 3 to 3.5% in January 1847, rose to 7% in April,
> when the first panic broke out… The official minimum bank discount rose in
> October to 7 and in November to 10%; i.e., the overwhelming mass of bills of
> exchange was discountable only at outrageous rates of interest, or no longer
> discountable at all."

> "the banking discount dropped to 5% in December" [after the crisis broke].

*Design consequence:* the crisis term is what drives `τ → 1` (rate → its `r`
ceiling — "no longer discountable at all"). Note the empirical numbers (1.75% calm
→ 10% crisis peak → 5% after) are the *shape* the mechanic reproduces in units of
the profit rate, not absolute percentages: **this is exactly the precedent the
`credit_fragility_threshold = 1.0e-3` calibration set** — a FRED time-series peak
(BAA10Y = 5.56% in Dec-2008) frozen into a `GameDefines` coefficient
(`capital_vol3.credit_fragility_threshold`), not read live at runtime. U9 extends
that precedent from the *fragility threshold* to the *rate itself*.

### §1.2 The formula, every term defined and its material source named

Let the annual economics step (inside `TickDynamicsSystem._compute_national_financial_state`)
have access to the shared graph. Define:

**Economy-wide average rate of profit `r`** — the interest ceiling. Marx's general
rate of profit = *total surplus / total capital*, so this is a **capital-weighted**
mean, never an unweighted mean of an intensive ratio (that is the
intensive-aggregation defect class the U7.6 sensor catches, and MEMORY
`intensive-aggregation-variance-error`):

```
r  =  Σ_t ( K_t · ρ_t )  /  Σ_t K_t          over active territory nodes t
```
- `ρ_t` = territory attribute `tick_profit_rate` (stamped by `graph_bridge` from
  `rates.profit_rate` — a real, verified attribute).
- `K_t` = territory attribute `tick_capital_stock` (stamped by `graph_bridge` from
  `county.capital_stock` — a real, verified attribute).
- Iterate territories in **sorted id order** (III.7). If **no** active territory
  carries `tick_profit_rate`, `r` is `None` (degenerate — §1.4).

**Loan-market tightness `τ ∈ [0,1]`** — the demand/supply balance that positions
`i` inside `[0, r)` (ch. 22 "supply … and … demand … decides the market level of
interest"):

```
τ  =  clamp( g_r · s_r  −  S , 0 , 1 )
```
- `s_r ∈ [0,1]` — the **reserve-army downturn signal** (the DEMAND proxy: the
  scramble for means of payment as overproduction throws labour out and sales
  fail). `ρ̄ = Σ_t(K_t·reserve_ratio_t)/Σ_t K_t` (capital-weighted mean of the
  persisted, always-present `reserve_ratio` node attribute, sorted-id), then:
  ```
  s_r  =  clamp( (ρ̄ − ρ_ref) / (1 − ρ_ref) , 0 , 1 )
  ```
  `ρ_ref` = `interest_reserve_reference` (defines). Below the reference there is no
  liquidity-demand pressure (`s_r = 0`); at full unemployment `s_r = 1`.
- `g_r` = `interest_reserve_demand_gain` (defines) — demand-side sensitivity.
- `S` — the **supply of idle money-capital** (ch. 25 "temporarily idle money of all
  classes … merge together into large masses"). **No such quantity exists on the
  graph.** Base U9 sets `S = 0` (no measured idle-capital slack — the honest value,
  neither raising nor lowering `τ`). Its creation is a **named follow-on** (§1.5),
  not silently faked.

**Interest share of profit `share(τ) ∈ (0,1)`** — the "purely empirical" division
(ch. 22), calm baseline lifted toward the crisis maximum:

```
share(τ)  =  base  +  (ceiling − base) · τ ,      0 < base < ceiling < 1
```
- `base`    = `interest_profit_share_base`    (calm interest share of profit).
- `ceiling` = `interest_profit_share_ceiling` (crisis-maximum share, strictly < 1
  so profit-of-enterprise stays positive; ch. 22's absolute max `i = r` is
  approached, never reached).

**National interest rate `i` and endogenous spread (fragility premium):**

```
i               =  0.0            if r is None or r ≤ 0        (degenerate)
                =  r · share(τ)   otherwise
fragility_premium (national_spread)
                =  0.0            if r is None or r ≤ 0
                =  r · (ceiling − base) · τ                    (= i − r·base ≥ 0)
```

`national_spread` **is** the endogenous fragility premium — the amount by which
loan-market tightness lifts the rate above its calm level `r·base`. It replaces the
BAA10Y runtime read entirely and is fed, unchanged, to the crisis assessor's
`credit_spread` parameter (see §1.6 scale check).

**Bound proof (a behavioral contract, encoded as a model validator, §U9.1):** for
`r > 0`, `0 < base ≤ share ≤ ceiling < 1`, hence `0 < i = r·share < r`. For
`r ≤ 0`, `i = 0`. Therefore `0 ≤ i < r` for `r > 0`, and `i = 0` exactly when
`r ≤ 0`. This satisfies the ruling's `0 ≤ i < r` with the theory-derived
degenerate behaviour, and is **provable statically from the closed form** (no RNG,
monotone in `τ`, `τ` clamped to `[0,1]`).

```mermaid
flowchart LR
  subgraph MB["Material Base (this tick)"]
    K["tick_capital_stock (territory)"]
    RHO["tick_profit_rate (territory)"]
    RA["reserve_ratio (territory)"]
  end
  K --> R["r = Σ K·ρ / Σ K  (avg rate of profit, ceiling)"]
  RHO --> R
  RA --> SR["s_r = downturn signal (demand)"]
  SR --> TAU["τ = clamp(g_r·s_r − S, 0, 1)   [S=0 deferred]"]
  R --> I["i = r · (base + (ceiling−base)·τ)"]
  TAU --> I
  I --> EIR["EndogenousInterestRate {rate, fragility_premium, τ}"]
  EIR --> NFP["NationalFinancialParameters (published to graph)"]
  EIR --> DIST["surplus distribution s = p + i + r + t"]
  EIR -.spread.-> CRISIS["financial-crisis assessor (credit_spread)"]
```

### §1.3 What FRED calibrates, and the derivation of every default coefficient

FRED is now **calibration only** — it fixes the four new `capital_vol3` defaults;
no FRED series is read at runtime for the rate. Each default is a documented
derivation, in the exact spirit of the shipped
`credit_fragility_threshold = 1.0e-3` (Dec-2008 BAA10Y peak → coefficient).

| Coefficient | Default | FRED / theory derivation |
|---|---|---|
| `interest_profit_share_base` | `0.30` | FRED net interest paid by domestic corporate business (`W273RC1`) ÷ corporate profits before tax (`A053RC1Q027SBEA`) sits in ≈ 0.25–0.35 across non-crisis years 1990–2019; `0.30` is the representative calm "empirical" division (ch. 22 — a convention, hence moddable, not a law). |
| `interest_profit_share_ceiling` | `0.95` | ch. 22: the absolute maximum is `i = r` (profit-of-enterprise = 0). The sim reserves 5% of profit for the functioning capitalist so the surplus identity's profit-of-enterprise residual stays > 0 as the normal case; `0.95` is that reserved max. |
| `interest_reserve_demand_gain` (`g_r`) | `1.0` | Neutral unit gain — the reference and ceiling do the shaping; exposed so a mod can dial crisis sensitivity of the loan market. |
| `interest_reserve_reference` (`ρ_ref`) | `0.08` | BLS civilian unemployment rate (`UNRATE`) averaged ≈ 5.8% (1948–2019); ≈ 8% marks the onset of a demand-side liquidity scramble (recession territory, +structural slack) above which `s_r` registers. |

**Calibration validation (U9.9) — a rewrite-test artifact (Constitution III.12).**
A single unit test asserts the `interest_profit_share_base` default reproduces the
FRED net-interest/profit ratio (from the committed `tests/fixtures/vol3_fred_series.json`
where those series exist) within a stated tolerance. That test **is** the durable
spec of the calibration: if the coefficient is regenerated, the derivation is
re-checked against real data, not vibes.

### §1.4 Degenerate-case table (each behaviour theory-derived)

| Case | `r` (ceiling) | `s_r` | `τ` | `i` (rate) | `national_spread` | Theory rationale |
|---|---|---|---|---|---|---|
| tick 0 / no territory carries `tick_profit_rate` | `None` | — | — | `0.0` | `0.0` | ch. 23: interest is *a portion of the profit*; none measured ⇒ nothing to divide ⇒ 0, **not a fabricated floor** (III.11). |
| loss-making economy `r ≤ 0` | `0.0` | any | any | `0.0` | `0.0` | ch. 22: max limit is *the profit itself*; no profit ⇒ `i = 0`. |
| calm expansion (`r > 0`, `ρ̄ ≤ ρ_ref`) | `r` | `0` | `0` | `r·base` (= `0.30·r`) | `0.0` | ch. 22: the "purely empirical" calm division of profit into interest. |
| downturn / crisis (`r > 0`, reserve-army surge) | `r` | `→1` | `→1` | `→ r·ceiling` (= `0.95·r`) | `→ r·(ceiling−base)·τ` | ch. 25: rate rises then *spikes in the crisis scramble* toward its maximum. |
| zero credit demand / no `CreditState` / no FRED wired | `r` | `s_r` | `τ` | `r·share` | endogenous | The rate is **total** — never `NoDataSentinel`, never `None`. This is the ruling. |
| `ρ̄` exactly `= ρ_ref` | `r` | `0` | `0` | `r·base` | `0.0` | onset threshold: below it no liquidity-demand pressure exists. |

### §1.5 Named deferrals (quantities that do not yet exist on the graph)

Honest-absence discipline requires naming, not faking, the quantities the full
Vol. III mechanic wants but the graph does not yet carry:

1. **Supply of idle money-capital `S`** (ch. 25) — the stagnation-phase reversal
   (idle money-capital accumulates, "the minimum limit of interest … may fall to
   any low"). No graph quantity distinguishes idle from actively-accumulating
   money-capital. **Creating task (future Program):** a `Substrate`/`Metabolism`-adjacent
   accumulation ledger deriving idle money-capital from *surplus not reinvested*
   (the gap between realized surplus and gross fixed capital formation, FRED
   `A054RC1Q027SBEA` vs private investment). Until then `S = 0`; base U9 reproduces
   *low / rising / spike* faithfully and defers only the *post-crisis collapse*.
2. **Acute single-tick correction spike `s_c`** — the market-scissors correction
   snap (`MARKET_CORRECTION_SHOCK_ATTR`) is the vivid "10% / no longer discountable"
   spike, but it is a `graph.graph` attribute written by `MarketScissorsSystem`
   (position ~17.8) *after* `TickDynamicsSystem` (position 4), and cross-tick
   persistence of `graph.graph` attributes across the `WorldState` round-trip is
   not guaranteed. Base U9 does **not** read it (no fragile cross-tick dependency).
   **Creating task (future):** promote `MARKET_CORRECTION_SHOCK_ATTR` into a
   round-trip-durable field and add `s_c` as a second demand term. Until then the
   reserve-army downturn signal carries the crisis (it is a persisted node
   attribute — reliable).

### §1.6 The seam and the migration

`NationalFinancialParameters` **remains the seam** (published under
`graph_bridge.NATIONAL_FINANCIAL_ATTR`). U9 swaps the **producer**
(`TickDynamicsSystem._compute_national_financial_state`), not the contract's role.

**Type-tightening migration (explicit).** The producer's return contract changes
from the U3-preserved **4-tuple**

```
tuple[float | None, float | None, FictitiousCapitalStock | None, str | None]
#     national_rate  national_spread  fictitious                interest_unavailable_reason
```

to a **3-tuple**

```
tuple[float, float, FictitiousCapitalStock | None]
#     national_rate  national_spread  fictitious
```

because after U9 the rate and spread are **total** (always a `float`) and the
`interest_unavailable_reason` path is dead. `fictitious` stays `Optional` — the
fictitious-capital / financialization data path is a **separate** honest-absence
contract, **unchanged by U9** (this is the load-bearing distinction for the
insertion notes in §4: U9 makes the *interest rate* endogenous; the fictitious
anchor legitimately stays absent past 2024).

**Removed unavailable-reason plumbing (every touched call site is listed in the
relevant task's `Files:` block):**
- `_compute_national_financial_state` — return type + body (U9.7).
- `_compute_financial_layer` — the 4-tuple unpack (U9.8).
- `_compute_county_financial_state` — drops the `interest_unavailable_reason`
  parameter *and* the now-dead `if national_rate is None:` skip branch; `national_rate`
  tightens to `float` (U9.8).
- `_assess_county_financial_crisis` — the `national_rate is not None` guard becomes
  vacuous (U9.8).
- `_build_credit_state` — rebuilt to take the endogenous spread, not `interest_state.baa_spread`
  (U9.7).
- The config gate `if services.interest_calculator is None: return county_states`
  becomes `if services.distribution_calculator is None:` so the endogenous rate
  works even when **no FRED interest calculator is wired** — the mechanical
  guarantee of "never unavailable" (U9.8).
- Tests: `tests/unit/economics/tick/test_system.py` (the honest-absence test that
  asserts `national_rate is None` — rewritten to the endogenous behaviour; the
  4-tuple unpack) (U9.7/U9.8).

**Scale check (endogenous spread → crisis assessor).** With `r ≈ 0.15`,
`ceiling − base = 0.65`: crisis `τ ≈ 0.6–1` ⇒ `spread ≈ 0.06–0.10`; calm `τ = 0`
⇒ `spread = 0`. The assessor fires `credit_fragility` when
`default_rate · spread > credit_fragility_threshold` = `0.02 · spread > 1.0e-3`,
i.e. `spread > 0.05` — crisis fires, calm does not. This reproduces the shipped
Dec-2008-calibrated behaviour **without** the BAA10Y read, and keeps the endogenous
spread on the same decimal scale the threshold already expects.

**Ordering / determinism.** U9 lands **before U7** (the sentinels bind to final
reality) and **before U8** (one baseline-regeneration ceremony). It will move the
`qa:regression` baselines (the surplus distribution's `interest_payments` term now
computes on every modeled year instead of only FRED-covered years, and computes an
endogenous rate). That movement is **expected** and is captured by U8's
owner-gated ceremony — U9 itself never regenerates a baseline.

---

> `national_rate` here always means the endogenous national rate; `r` means
> the economy-wide average rate of profit.

### Task U9.1: `EndogenousInterestRate` model — the `0 ≤ i < r` bound as a construction invariant

**Files:**
- Modify: `src/babylon/domain/economics/credit/types.py` (append after
  `class FictitiousCapitalStock`)
- Test: `tests/unit/economics/credit/test_endogenous_interest_types.py` *(create)*

**Interfaces:**
- Produces: `EndogenousInterestRate(BaseModel)` — frozen; fields `year: int`,
  `profit_rate_ceiling: float`, `rate: float`, `fragility_premium: float`,
  `tightness: float`, `reserve_army_signal: float`; computed
  `base_component -> float`; a `model_validator` enforcing the ch. 22 bound.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/economics/credit/test_endogenous_interest_types.py`:

```python
"""U9.1: the EndogenousInterestRate model encodes Marx's ch. 22 bound
(0 <= i < r) as a construction invariant — a behavioral contract that
outlives the producer (Constitution III.12)."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.domain.economics.credit.types import EndogenousInterestRate


@pytest.mark.unit
class TestEndogenousInterestRate:
    def test_present_rate_is_strictly_below_the_profit_ceiling(self) -> None:
        state = EndogenousInterestRate(
            year=2015,
            profit_rate_ceiling=0.15,
            rate=0.045,
            fragility_premium=0.0,
            tightness=0.0,
            reserve_army_signal=0.0,
        )
        assert state.rate < state.profit_rate_ceiling
        assert state.base_component == pytest.approx(0.045)

    def test_base_component_strips_the_fragility_premium(self) -> None:
        state = EndogenousInterestRate(
            year=2015,
            profit_rate_ceiling=0.15,
            rate=0.12,
            fragility_premium=0.075,
            tightness=0.8,
            reserve_army_signal=0.8,
        )
        assert state.base_component == pytest.approx(0.045)

    def test_zero_ceiling_forces_zero_rate(self) -> None:
        # r <= 0: no profit to divide -> i must be 0 (ch. 22 / ch. 23).
        EndogenousInterestRate(
            year=2015,
            profit_rate_ceiling=0.0,
            rate=0.0,
            fragility_premium=0.0,
            tightness=0.0,
            reserve_army_signal=0.0,
        )

    def test_rate_at_or_above_ceiling_is_rejected(self) -> None:
        # ch. 22: the maximum limit of interest is the profit itself; the sim
        # keeps profit-of-enterprise strictly positive, so i must be < r.
        with pytest.raises(ValidationError, match="rate .* profit_rate_ceiling"):
            EndogenousInterestRate(
                year=2015,
                profit_rate_ceiling=0.10,
                rate=0.10,
                fragility_premium=0.0,
                tightness=0.0,
                reserve_army_signal=0.0,
            )
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest_types.py`
Expected: FAIL — `ImportError: cannot import name 'EndogenousInterestRate' from 'babylon.domain.economics.credit.types'`.

- [ ] **Step 3: Write minimal implementation**

Append to `src/babylon/domain/economics/credit/types.py` (after `class FictitiousCapitalStock`; add `model_validator` to the existing `from pydantic import …` line):

```python
class EndogenousInterestRate(BaseModel):
    """Endogenous national interest rate (Capital Vol. III Part V).

    Marx: interest has no natural rate (ch. 22) — it is a *share of the
    profit* (ch. 23/24) set by the supply and demand of loanable
    money-capital (ch. 22). This model carries the computed rate together
    with the average rate of profit that bounds it, and encodes the ch. 22
    maximum ("the maximum limit of interest is the profit itself") as a
    construction invariant: ``rate < profit_rate_ceiling`` whenever there is
    a profit to divide, ``rate == 0`` when there is none.

    Feature: vol3-money-scissors U9.

    :ivar year: Simulation year (no upper ceiling — the campaign runs to
        2109; deliberately unlike ``InterestRateState.year``'s le=2040, which
        was a latent post-2040 crash on the live path).
    :ivar profit_rate_ceiling: Economy-wide average rate of profit ``r``
        (capital-weighted); ``0.0`` when no profit is measured.
    :ivar rate: National interest rate ``i`` = ``r * share(tightness)``, or
        ``0.0`` when ``profit_rate_ceiling`` is ``0.0``.
    :ivar fragility_premium: Endogenous spread = ``i - r*base`` (>= 0); the
        crisis lift of the rate above its calm level. Replaces the BAA10Y
        runtime read.
    :ivar tightness: Loan-market tightness ``tau`` in [0, 1] (provenance).
    :ivar reserve_army_signal: The downturn demand signal ``s_r`` in [0, 1]
        (provenance).
    """

    model_config = ConfigDict(frozen=True)

    year: int = Field(..., ge=2007)
    profit_rate_ceiling: float = Field(..., ge=0.0)
    rate: float = Field(..., ge=0.0)
    fragility_premium: float = Field(..., ge=0.0)
    tightness: float = Field(..., ge=0.0, le=1.0)
    reserve_army_signal: float = Field(..., ge=0.0, le=1.0)

    @computed_field  # type: ignore[prop-decorator]
    @property
    def base_component(self) -> float:
        """The calm interest ``r*base`` = ``rate - fragility_premium``."""
        return self.rate - self.fragility_premium

    @model_validator(mode="after")
    def _verify_interest_below_profit(self) -> EndogenousInterestRate:
        """Enforce Marx's ch. 22 bound as a loud construction invariant.

        ``rate < profit_rate_ceiling`` when a profit exists; ``rate == 0``
        when it does not (Constitution III.11 — a violated bound is a loud
        ValueError at construction, never a silently wrong rate).
        """
        if self.profit_rate_ceiling <= 0.0:
            if self.rate != 0.0:
                raise ValueError(
                    f"rate must be 0.0 when profit_rate_ceiling is "
                    f"{self.profit_rate_ceiling!r} (no profit to divide), got "
                    f"{self.rate!r}"
                )
        elif self.rate >= self.profit_rate_ceiling:
            raise ValueError(
                f"rate ({self.rate!r}) must be strictly below "
                f"profit_rate_ceiling ({self.profit_rate_ceiling!r}) — "
                f"Capital Vol. III ch. 22: interest is a portion of the profit"
            )
        return self
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest_types.py`
Expected: PASS (4 passed).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(credit): EndogenousInterestRate model with the ch.22 0<=i<r bound

Encodes Marx's 'the maximum limit of interest is the profit itself' as a
construction invariant. New model, no producer yet (U9.7 fills it).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.2: `capital_vol3` interest coefficients + regenerate `defines.yaml`

**Files:**
- Modify: `src/babylon/config/defines/capital_vol3.py` (four new fields +
  `model_validator`)
- Modify: `src/babylon/data/defines.yaml` (**regenerated, never hand-edited**)
- Test: `tests/unit/config/test_capital_vol3_interest_defines.py` *(create)*

**Interfaces:**
- Produces: `CapitalVolumeIIIDefines.interest_profit_share_base` (0.30),
  `.interest_profit_share_ceiling` (0.95), `.interest_reserve_demand_gain` (1.0),
  `.interest_reserve_reference` (0.08). Read directly (`services.defines.capital_vol3.<field>`),
  the pattern `profit_rate_fallback`/`default_rate_estimate` already use.
- Note: these are **added fields on an already-exported class** — `defines.__all__`
  is unchanged, so **no** `EXPECTED_DEFINES_PUBLIC` edit is needed (contrast the
  U7.11 specimen). `tests/unit/config/test_constants_sync.py` must pass after
  regeneration.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/config/test_capital_vol3_interest_defines.py`:

```python
"""U9.2: the four endogenous-interest coefficients and their base<ceiling<1
invariant (Capital Vol. III ch. 22 — interest is a share of profit)."""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.config.defines.capital_vol3 import CapitalVolumeIIIDefines


@pytest.mark.unit
class TestInterestDefines:
    def test_defaults_are_the_calibrated_values(self) -> None:
        d = CapitalVolumeIIIDefines()
        assert d.interest_profit_share_base == pytest.approx(0.30)
        assert d.interest_profit_share_ceiling == pytest.approx(0.95)
        assert d.interest_reserve_demand_gain == pytest.approx(1.0)
        assert d.interest_reserve_reference == pytest.approx(0.08)

    def test_base_must_be_below_ceiling(self) -> None:
        with pytest.raises(ValidationError, match="interest_profit_share_base"):
            CapitalVolumeIIIDefines(
                interest_profit_share_base=0.96,
                interest_profit_share_ceiling=0.95,
            )
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/config/test_capital_vol3_interest_defines.py`
Expected: FAIL — `AttributeError: 'CapitalVolumeIIIDefines' object has no attribute 'interest_profit_share_base'`.

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/config/defines/capital_vol3.py`, add four fields after
`housing_capitalization_rate_default` (before the existing
`@field_validator("counter_tendency_weights")`):

```python
    interest_profit_share_base: float = Field(
        default=0.30,
        gt=0.0,
        lt=1.0,
        description=(
            "Calm interest share of the average rate of profit (Capital "
            "Vol. III ch. 22 — the 'purely empirical' division of profit "
            "into interest and profit of enterprise; a convention, not a "
            "law, hence player-editable). Calibration: FRED net corporate "
            "interest paid (W273RC1) / corporate profits before tax "
            "(A053RC1Q027SBEA) ~ 0.25-0.35 across non-crisis years "
            "1990-2019."
        ),
    )
    interest_profit_share_ceiling: float = Field(
        default=0.95,
        gt=0.0,
        lt=1.0,
        description=(
            "Crisis-maximum interest share of profit. Capital Vol. III "
            "ch. 22: the absolute maximum is i=r (profit of enterprise=0); "
            "the sim reserves 5% of profit for the functioning capitalist so "
            "the surplus identity's profit-of-enterprise residual stays "
            "positive, hence a strict ceiling < 1. Must exceed "
            "interest_profit_share_base."
        ),
    )
    interest_reserve_demand_gain: float = Field(
        default=1.0,
        ge=0.0,
        description=(
            "Demand-side sensitivity of loan-market tightness to the "
            "reserve-army downturn signal (Capital Vol. III ch. 22 — the "
            "demand for loanable capital decides the market rate). Unit "
            "default; a mod dials crisis sensitivity of the credit system."
        ),
    )
    interest_reserve_reference: float = Field(
        default=0.08,
        ge=0.0,
        lt=1.0,
        description=(
            "Reserve-army ratio at which the demand-side liquidity scramble "
            "begins (below it, s_r=0). Calibration: BLS civilian "
            "unemployment (UNRATE) averaged ~5.8% 1948-2019; ~8% marks "
            "recession-territory onset of the scramble for means of payment "
            "(Capital Vol. III ch. 25)."
        ),
    )
```

Then add a model validator (after the class fields, alongside
`verify_weights_sum_to_one`):

```python
    @model_validator(mode="after")
    def verify_interest_share_ordering(self) -> CapitalVolumeIIIDefines:
        """Reject base >= ceiling (ch. 22: interest is bounded by profit).

        share(tau) = base + (ceiling-base)*tau must land in (0, 1) with the
        crisis ceiling above the calm base, else the endogenous rate is not
        monotone in loan-market tightness. Fails loudly at config-load
        (Constitution III.11).
        """
        if self.interest_profit_share_base >= self.interest_profit_share_ceiling:
            raise ValueError(
                f"capital_vol3.interest_profit_share_base "
                f"({self.interest_profit_share_base!r}) must be strictly below "
                f"interest_profit_share_ceiling "
                f"({self.interest_profit_share_ceiling!r}) — check defines.yaml"
            )
        return self
```

Add `model_validator` to the `from pydantic import …` line. Then regenerate:

```bash
poetry run python tools/generate_defines_config.py
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `mise run test:q -- tests/unit/config/test_capital_vol3_interest_defines.py tests/unit/config/test_constants_sync.py`
Expected: PASS (sync test confirms `defines.yaml` regenerated to match the schema).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(defines): capital_vol3 endogenous-interest coefficients (FRED-calibrated)

interest_profit_share_base/ceiling, reserve_demand_gain, reserve_reference
+ base<ceiling validator; defines.yaml regenerated. Calibration only — no
FRED runtime read.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.3: `endogenous_interest_rate` core — the bounded share of profit

**Files:**
- Create: `src/babylon/domain/economics/credit/endogenous_interest.py`
- Test: `tests/unit/economics/credit/test_endogenous_interest.py` *(create)*

**Interfaces:**
- Consumes: `EndogenousInterestRate` (U9.1), `GameDefines` (`capital_vol3` fields,
  U9.2).
- Produces: `endogenous_interest_rate(profit_rate: float | None, tightness: float, defines: GameDefines) -> EndogenousInterestRate`.

> **M1 carve-out (Global Constraints):** this task's deliverable is a **new
> module**; its red phase is the collection/import error, not a manufactured
> assertion.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/economics/credit/test_endogenous_interest.py`:

```python
"""U9.3: the endogenous interest rate as a bounded share of the average
rate of profit (Capital Vol. III Part V)."""

from __future__ import annotations

import pytest

from babylon.config.defines import GameDefines
from babylon.domain.economics.credit.endogenous_interest import (
    endogenous_interest_rate,
)


@pytest.mark.unit
class TestEndogenousInterestRate:
    def _defines(self) -> GameDefines:
        return GameDefines.load_default()

    def test_calm_market_pays_the_base_share_of_profit(self) -> None:
        state = endogenous_interest_rate(0.15, 0.0, self._defines())
        assert state.rate == pytest.approx(0.15 * 0.30)
        assert state.fragility_premium == pytest.approx(0.0)
        assert state.profit_rate_ceiling == pytest.approx(0.15)

    def test_full_tightness_approaches_the_crisis_ceiling(self) -> None:
        state = endogenous_interest_rate(0.15, 1.0, self._defines())
        assert state.rate == pytest.approx(0.15 * 0.95)
        assert state.rate < state.profit_rate_ceiling  # ch.22: i < r
        assert state.fragility_premium == pytest.approx(0.15 * (0.95 - 0.30))

    def test_absent_profit_rate_yields_zero_interest(self) -> None:
        # tick 0 / no territory carries a profit rate: nothing to divide.
        state = endogenous_interest_rate(None, 0.7, self._defines())
        assert state.rate == 0.0
        assert state.fragility_premium == 0.0
        assert state.profit_rate_ceiling == 0.0

    def test_nonpositive_profit_rate_yields_zero_interest(self) -> None:
        state = endogenous_interest_rate(-0.02, 1.0, self._defines())
        assert state.rate == 0.0
        assert state.fragility_premium == 0.0

    def test_tightness_is_clamped_into_unit_interval(self) -> None:
        hot = endogenous_interest_rate(0.15, 5.0, self._defines())
        assert hot.tightness == 1.0
        assert hot.rate == pytest.approx(0.15 * 0.95)
        cold = endogenous_interest_rate(0.15, -3.0, self._defines())
        assert cold.tightness == 0.0
        assert cold.rate == pytest.approx(0.15 * 0.30)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest.py`
Expected: FAIL (collection error) — `ModuleNotFoundError: No module named 'babylon.domain.economics.credit.endogenous_interest'` (M1 carve-out: this new-module task takes the collection error as its red).

- [ ] **Step 3: Write minimal implementation**

Create `src/babylon/domain/economics/credit/endogenous_interest.py`:

```python
"""Endogenous national interest rate (Capital Vol. III Part V).

Marx (ch. 22): "There is no such thing as a 'natural' rate of interest." The
rate is not read off a curve — it is *computed* from the average rate of
profit it divides (the ceiling; ch. 22 "the maximum limit of interest is the
profit itself") and the competitive tightness of the loan market (ch. 22 "the
relation between the supply of loanable capital … and the demand for it …
decides the market level of interest").

Pure functions of their arguments: no RNG, no wall clock, no I/O. This module
lives in ``domain/`` and imports nothing from ``engine/``; the engine reads
it.
"""

from __future__ import annotations

from babylon.config.defines import GameDefines
from babylon.domain.economics.credit.types import EndogenousInterestRate

#: Reserved: the supply of idle money-capital (Capital Vol. III ch. 25) has no
#: graph quantity yet; base U9 sets it to zero so it neither raises nor lowers
#: loan-market tightness (see design §1.5 for the creating task).
_IDLE_MONEY_CAPITAL_SUPPLY: float = 0.0


def _clamp_unit(value: float) -> float:
    """Clamp ``value`` into the closed unit interval [0.0, 1.0]."""
    if value < 0.0:
        return 0.0
    if value > 1.0:
        return 1.0
    return value


def endogenous_interest_rate(
    profit_rate: float | None,
    tightness: float,
    defines: GameDefines,
) -> EndogenousInterestRate:
    """Compute the national interest rate as a bounded share of profit.

    ``i = r * (base + (ceiling - base) * tau)`` for ``r > 0``; ``i = 0`` when
    no profit is measured (``profit_rate`` is ``None`` or ``<= 0``) — Capital
    Vol. III ch. 23: interest is *a portion of the profit*, so with no profit
    there is nothing to divide (never a fabricated positive floor,
    Constitution III.11).

    :param profit_rate: Economy-wide average rate of profit ``r``, or ``None``
        when no profit observable exists this tick.
    :param tightness: Loan-market tightness ``tau`` (clamped into [0, 1]).
    :param defines: Run-scoped ``GameDefines`` (reads ``capital_vol3``).
    :returns: A total :class:`EndogenousInterestRate` — never absent.
    """
    cvi = defines.capital_vol3
    base = cvi.interest_profit_share_base
    ceiling = cvi.interest_profit_share_ceiling
    tau = _clamp_unit(tightness)

    if profit_rate is None or profit_rate <= 0.0:
        return EndogenousInterestRate(
            year=0,
            profit_rate_ceiling=0.0,
            rate=0.0,
            fragility_premium=0.0,
            tightness=tau,
            reserve_army_signal=0.0,
        )

    share = base + (ceiling - base) * tau
    rate = profit_rate * share
    premium = profit_rate * (ceiling - base) * tau
    return EndogenousInterestRate(
        year=0,
        profit_rate_ceiling=profit_rate,
        rate=rate,
        fragility_premium=premium,
        tightness=tau,
        reserve_army_signal=0.0,
    )
```

> `year`/`reserve_army_signal` are filled by the producer (U9.7), which knows
> the tick year and the underlying `s_r`; the pure core takes only the reduced
> `(r, tau)` so it is trivially testable.

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest.py`
Expected: PASS (5 passed).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(credit): endogenous_interest_rate core — bounded share of profit

i = r*(base + (ceiling-base)*tau), i=0 when r<=0. Pure, no I/O. Capital
Vol. III ch. 22/23. Not wired to the producer yet (U9.7).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.4: `loan_market_tightness` — the demand/supply balance

**Files:**
- Modify: `src/babylon/domain/economics/credit/endogenous_interest.py`
- Test: `tests/unit/economics/credit/test_endogenous_interest.py`

**Interfaces:**
- Produces: `loan_market_tightness(reserve_army_signal: float, defines: GameDefines) -> float`
  — `clamp(g_r * s_r - S, 0, 1)`, `S = _IDLE_MONEY_CAPITAL_SUPPLY` (0, deferred).

- [ ] **Step 1: Write the failing test**

Add to `tests/unit/economics/credit/test_endogenous_interest.py`:

```python
@pytest.mark.unit
class TestLoanMarketTightness:
    def _defines(self) -> GameDefines:
        return GameDefines.load_default()

    def test_no_reserve_army_signal_is_slack_market(self) -> None:
        from babylon.domain.economics.credit.endogenous_interest import (
            loan_market_tightness,
        )

        assert loan_market_tightness(0.0, self._defines()) == 0.0

    def test_full_reserve_army_signal_saturates_tightness(self) -> None:
        from babylon.domain.economics.credit.endogenous_interest import (
            loan_market_tightness,
        )

        # g_r = 1.0 (default) so s_r = 1.0 -> tau = 1.0.
        assert loan_market_tightness(1.0, self._defines()) == pytest.approx(1.0)

    def test_tightness_is_clamped_when_gain_overshoots(self) -> None:
        from babylon.domain.economics.credit.endogenous_interest import (
            loan_market_tightness,
        )

        d = GameDefines.load_default().model_copy(
            update={
                "capital_vol3": GameDefines.load_default().capital_vol3.model_copy(
                    update={"interest_reserve_demand_gain": 4.0}
                )
            }
        )
        assert loan_market_tightness(0.5, d) == pytest.approx(1.0)
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest.py::TestLoanMarketTightness`
Expected: FAIL — `ImportError: cannot import name 'loan_market_tightness'`.

- [ ] **Step 3: Write minimal implementation**

Append to `src/babylon/domain/economics/credit/endogenous_interest.py`:

```python
def loan_market_tightness(reserve_army_signal: float, defines: GameDefines) -> float:
    """Loan-market tightness ``tau`` — the demand/supply balance of loanable
    money-capital (Capital Vol. III ch. 22).

    ``tau = clamp(g_r * s_r - S, 0, 1)``. The demand term is the reserve-army
    downturn signal ``s_r`` (the material scramble for means of payment as
    overproduction throws labour out; ch. 25). The supply term ``S`` — idle
    money-capital — has no graph quantity yet and is fixed at 0 (design §1.5),
    so base U9 reproduces low/rising/spike and defers only the stagnation
    collapse.

    :param reserve_army_signal: ``s_r`` in [0, 1] (see
        ``graph_bridge.reserve_army_signal``).
    :param defines: Run-scoped ``GameDefines`` (reads ``capital_vol3``).
    :returns: ``tau`` in [0, 1].
    """
    gain = defines.capital_vol3.interest_reserve_demand_gain
    demand = gain * reserve_army_signal
    return _clamp_unit(demand - _IDLE_MONEY_CAPITAL_SUPPLY)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/credit/test_endogenous_interest.py`
Expected: PASS (8 passed).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(credit): loan_market_tightness — demand/supply of loanable capital

tau = clamp(g_r*s_r - S, 0, 1); S (idle money-capital supply) deferred to 0
with the creating task named. Capital Vol. III ch. 22/25.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.5: domain graph observables — `r` and `s_r` from the material base

**Files:**
- Modify: `src/babylon/domain/economics/tick/graph_bridge.py` (two readers +
  `__all__`)
- Test: `tests/unit/economics/tick/test_graph_bridge_interest_observables.py` *(create)*

**Interfaces:**
- Consumes: `GraphProtocol`; territory node attributes `tick_profit_rate`,
  `tick_capital_stock`, `reserve_ratio` (all verified real, stamped by
  `graph_bridge`/`ReserveArmySystem`).
- Produces:
  - `economy_wide_profit_rate(graph: GraphProtocol) -> float | None` —
    capital-weighted mean of `tick_profit_rate` (Marx's general rate of profit =
    total surplus / total capital), or `None` when no active territory carries one.
  - `reserve_army_signal(graph: GraphProtocol, defines: GameDefines) -> float` —
    `s_r` in [0, 1] from the capital-weighted mean `reserve_ratio` and
    `interest_reserve_reference`.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/economics/tick/test_graph_bridge_interest_observables.py`:

```python
"""U9.5: domain-side extraction of the average rate of profit r and the
reserve-army downturn signal s_r from the shared graph (capital-weighted,
sorted-id, honest absence)."""

from __future__ import annotations

import pytest

from babylon.config.defines import GameDefines
from babylon.domain.economics.tick.graph_bridge import (
    economy_wide_profit_rate,
    reserve_army_signal,
)
from babylon.models.enums.topology import NodeType
from babylon.topology import BabylonGraph


def _graph_with(territories: list[dict[str, object]]) -> BabylonGraph:
    g = BabylonGraph()
    for i, attrs in enumerate(territories):
        g.add_node(f"terr_{i}", _node_type=NodeType.TERRITORY, **attrs)
    return g


@pytest.mark.unit
class TestEconomyWideProfitRate:
    def test_capital_weighted_not_unweighted(self) -> None:
        # 0.10 @ K=100 and 0.20 @ K=900 -> weighted 0.19, not the mean 0.15.
        g = _graph_with(
            [
                {"active": True, "tick_profit_rate": 0.10, "tick_capital_stock": 100.0},
                {"active": True, "tick_profit_rate": 0.20, "tick_capital_stock": 900.0},
            ]
        )
        assert economy_wide_profit_rate(g) == pytest.approx(0.19)

    def test_no_rate_present_is_none(self) -> None:
        g = _graph_with([{"active": True, "tick_capital_stock": 100.0}])
        assert economy_wide_profit_rate(g) is None

    def test_inactive_territories_are_excluded(self) -> None:
        g = _graph_with(
            [
                {"active": False, "tick_profit_rate": 0.99, "tick_capital_stock": 1e9},
                {"active": True, "tick_profit_rate": 0.05, "tick_capital_stock": 10.0},
            ]
        )
        assert economy_wide_profit_rate(g) == pytest.approx(0.05)


@pytest.mark.unit
class TestReserveArmySignal:
    def test_below_reference_is_zero(self) -> None:
        g = _graph_with(
            [{"active": True, "reserve_ratio": 0.05, "tick_capital_stock": 100.0}]
        )
        assert reserve_army_signal(g, GameDefines.load_default()) == 0.0

    def test_scales_between_reference_and_one(self) -> None:
        # ref=0.08: reserve 0.54 -> (0.54-0.08)/(1-0.08) = 0.5.
        g = _graph_with(
            [{"active": True, "reserve_ratio": 0.54, "tick_capital_stock": 100.0}]
        )
        assert reserve_army_signal(g, GameDefines.load_default()) == pytest.approx(0.5)

    def test_no_reserve_data_is_zero(self) -> None:
        g = _graph_with([{"active": True, "tick_capital_stock": 100.0}])
        assert reserve_army_signal(g, GameDefines.load_default()) == 0.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge_interest_observables.py`
Expected: FAIL — `ImportError: cannot import name 'economy_wide_profit_rate' from 'babylon.domain.economics.tick.graph_bridge'`.

- [ ] **Step 3: Write minimal implementation**

Add to `src/babylon/domain/economics/tick/graph_bridge.py` (after
`read_national_financial_state_from_graph`; extend `__all__`). Import
`GameDefines` and `NodeType` at module top if not already present:

```python
def _capital_weighted_mean(
    graph: GraphProtocol, attr: str
) -> float | None:
    """Capital-weighted mean of ``attr`` over active territory nodes.

    Weight is ``tick_capital_stock`` (Marx's general rate of profit = total
    surplus / total capital — an extensive weighting, not the unweighted mean
    of an intensive ratio, which is the intensive-aggregation defect class).
    Sorted-id iteration (Constitution III.7). Returns ``None`` when no active
    territory carries ``attr`` with a positive weight (honest absence,
    III.11).
    """
    weighted = 0.0
    weight = 0.0
    for node in sorted(
        graph.query_nodes(node_type=NodeType.TERRITORY), key=lambda n: n.id
    ):
        attrs = node.attributes
        if not attrs.get("active", True):
            continue
        value = attrs.get(attr)
        capital = attrs.get("tick_capital_stock")
        if not isinstance(value, (int, float)):
            continue
        if not isinstance(capital, (int, float)) or float(capital) <= 0.0:
            continue
        weighted += float(value) * float(capital)
        weight += float(capital)
    if weight <= 0.0:
        return None
    return weighted / weight


def economy_wide_profit_rate(graph: GraphProtocol) -> float | None:
    """Economy-wide average rate of profit ``r`` (the interest ceiling).

    Capital-weighted mean of territory ``tick_profit_rate`` (Capital Vol. III
    ch. 22: interest is bounded by the average rate of profit). ``None`` when
    no active territory carries a profit rate — the endogenous rate then reads
    zero (no profit to divide), never a fabricated floor.
    """
    return _capital_weighted_mean(graph, "tick_profit_rate")


def reserve_army_signal(graph: GraphProtocol, defines: GameDefines) -> float:
    """Reserve-army downturn signal ``s_r`` in [0, 1] (loan-capital demand).

    ``clamp((rho_bar - rho_ref) / (1 - rho_ref), 0, 1)`` where ``rho_bar`` is
    the capital-weighted mean territory ``reserve_ratio`` and ``rho_ref`` is
    ``capital_vol3.interest_reserve_reference``. The rising reserve army is
    the material signature of the crisis that ignites the scramble for means
    of payment (Capital Vol. III ch. 25); below the reference there is no
    liquidity-demand pressure. Zero (not absent) when no reserve data exists.
    """
    rho_bar = _capital_weighted_mean(graph, "reserve_ratio")
    if rho_bar is None:
        return 0.0
    rho_ref = defines.capital_vol3.interest_reserve_reference
    denom = 1.0 - rho_ref
    if denom <= 0.0:
        return 1.0 if rho_bar > rho_ref else 0.0
    raw = (rho_bar - rho_ref) / denom
    return 0.0 if raw < 0.0 else (1.0 if raw > 1.0 else raw)
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/tick/test_graph_bridge_interest_observables.py`
Expected: PASS (6 passed).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): domain observables for endogenous interest (r, s_r)

economy_wide_profit_rate (capital-weighted avg rate of profit, the ceiling)
and reserve_army_signal (loan-capital demand). Sorted-id, honest absence.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.6: `NationalFinancialParameters.endogenous_interest` field + round-trip

**Files:**
- Modify: `src/babylon/domain/economics/tick/types.py` (`NationalFinancialParameters`)
- Test: `tests/unit/economics/tick/test_national_financial_endogenous_field.py` *(create)*

**Interfaces:**
- Produces: `NationalFinancialParameters.endogenous_interest: EndogenousInterestRate | None`
  (default `None` for `.empty()`; the producer always sets it). Published/read
  unchanged via `write_/read_national_financial_state_to_graph` (they
  `model_dump()`/reconstruct the whole model — the round-trip test proves the new
  field survives).

- [ ] **Step 1: Write the failing test**

Create `tests/unit/economics/tick/test_national_financial_endogenous_field.py`:

```python
"""U9.6: the endogenous interest rate rides the NationalFinancialParameters
seam and survives the graph round-trip."""

from __future__ import annotations

import pytest

from babylon.domain.economics.credit.types import EndogenousInterestRate
from babylon.domain.economics.tick.graph_bridge import (
    read_national_financial_state_from_graph,
    write_national_financial_state_to_graph,
)
from babylon.domain.economics.tick.types import NationalFinancialParameters
from babylon.topology import BabylonGraph


@pytest.mark.unit
class TestEndogenousInterestSeam:
    def test_empty_has_no_endogenous_interest(self) -> None:
        assert NationalFinancialParameters.empty().endogenous_interest is None

    def test_round_trip_preserves_the_endogenous_rate(self) -> None:
        eir = EndogenousInterestRate(
            year=2015,
            profit_rate_ceiling=0.15,
            rate=0.06,
            fragility_premium=0.015,
            tightness=0.3,
            reserve_army_signal=0.3,
        )
        params = NationalFinancialParameters(endogenous_interest=eir)
        g = BabylonGraph()
        write_national_financial_state_to_graph(g, params)
        back = read_national_financial_state_from_graph(g)
        assert back is not None
        assert back.endogenous_interest is not None
        assert back.endogenous_interest.rate == pytest.approx(0.06)
        assert back.endogenous_interest.rate < back.endogenous_interest.profit_rate_ceiling
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/tick/test_national_financial_endogenous_field.py`
Expected: FAIL — `pydantic_core._pydantic_core.ValidationError: … endogenous_interest … Unexpected keyword argument` (the field does not exist yet).

- [ ] **Step 3: Write minimal implementation**

In `src/babylon/domain/economics/tick/types.py`, import `EndogenousInterestRate`
alongside the other credit types, and add a field to
`NationalFinancialParameters` (after `fictitious_capital`):

```python
    endogenous_interest: EndogenousInterestRate | None = Field(
        default=None,
        description=(
            "Endogenous national interest rate (Vol. III Part V). Always set "
            "by the producer post-U9; None only on .empty()."
        ),
    )
```

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/tick/test_national_financial_endogenous_field.py`
Expected: PASS (2 passed).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): NationalFinancialParameters carries endogenous_interest

New optional field on the published seam; graph round-trip proven. Producer
fills it in U9.7.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.7: rewire the producer — endogenous rate replaces the FRED read

**Files:**
- Modify: `src/babylon/domain/economics/tick/system/__init__.py`
  (`_compute_national_financial_state`, `_build_credit_state`)
- Test: `tests/unit/economics/tick/test_system.py` (rewrite the honest-absence
  test that asserts `national_rate is None`)

**Interfaces:**
- Consumes: `economy_wide_profit_rate`, `reserve_army_signal` (U9.5);
  `loan_market_tightness`, `endogenous_interest_rate` (U9.3/U9.4);
  `EndogenousInterestRate` (U9.1).
- Produces: `_compute_national_financial_state(self, services, year, graph) -> tuple[float, float, FictitiousCapitalStock | None]`
  — the **3-tuple** `(national_rate, national_spread, fictitious)`, both leading
  members always `float`.

- [ ] **Step 1: Write the failing test**

In `tests/unit/economics/tick/test_system.py`, **replace** the existing
honest-absence test (the one containing
`national_rate, _spread, _fictitious, reason = system._compute_national_financial_state(`
followed by `assert national_rate is None`) with the endogenous contract. Locate
it by that exact `assert national_rate is None` line:

```python
    def test_national_rate_is_endogenous_and_total(self) -> None:
        """U9: the national rate is computed from the sim's own quantities and
        is never None. With no profit rate on the graph it is 0.0 (no profit
        to divide, Capital Vol. III ch. 23), not a NoDataSentinel."""
        system, services, graph, year = self._financial_fixture()  # existing helper
        national_rate, national_spread, _fictitious = (
            system._compute_national_financial_state(services, year, graph)
        )
        assert isinstance(national_rate, float)
        assert isinstance(national_spread, float)
        assert national_rate == 0.0  # empty graph: no territory profit rate
        assert national_spread == 0.0
```

> Locate `self._financial_fixture()` by whatever the existing test uses to build
> `(system, services, graph)`; reuse it verbatim. If the pre-U9 test built these
> inline, keep that construction and only change the assertions.

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/tick/test_system.py::TestNationalFinancialState`
Expected: FAIL — `ValueError: not enough values to unpack (expected 3, got 4)` (the producer still returns the 4-tuple).

- [ ] **Step 3: Write minimal implementation**

Replace the body of `_compute_national_financial_state` (locate by the docstring
line `"""Compute national-level financial parameters once per tick.`). New body:

```python
    def _compute_national_financial_state(
        self,
        services: ServicesProtocol,
        year: int,
        graph: GraphProtocol,
    ) -> tuple[float, float, FictitiousCapitalStock | None]:
        """Compute the national financial parameters once per tick.

        vol3-money-scissors U9: the national interest rate is ENDOGENOUS
        (Capital Vol. III Part V) — computed from the economy-wide average
        rate of profit (the ceiling; ch. 22) and loan-market tightness (the
        demand for loanable money-capital; ch. 22/25), never read from FRED
        and never absent. FRED is calibration only (see capital_vol3).

        Returns:
            3-tuple ``(national_rate, national_spread, fictitious)``.
            ``national_rate`` is total (always a float, 0.0 when no profit is
            measured); ``national_spread`` is the endogenous fragility premium
            (>= 0); ``fictitious`` remains Optional (a separate data path).
        """
        defines = services.defines
        profit_rate = economy_wide_profit_rate(graph)
        s_r = reserve_army_signal(graph, defines)
        tau = loan_market_tightness(s_r, defines)
        endogenous = endogenous_interest_rate(profit_rate, tau, defines)
        endogenous = endogenous.model_copy(
            update={"year": year, "reserve_army_signal": s_r}
        )
        national_rate = endogenous.rate
        national_spread = endogenous.fragility_premium

        fictitious = None
        if services.fictitious_capital_calculator is not None:
            fict_result = services.fictitious_capital_calculator.compute_fictitious_capital(year)
            if isinstance(fict_result, NoDataSentinel):
                services.economics_fallbacks.record_vol3_fictitious_sentinel()
                self._log_vol3_sentinel_once_per_year(year, "fictitious", fict_result.reason)
            else:
                fictitious = fict_result

        financial_params = NationalFinancialParameters(
            endogenous_interest=endogenous,
            credit_state=self._build_credit_state(services, year, national_spread),
            fictitious_capital=fictitious,
        )
        write_national_financial_state_to_graph(graph, financial_params)

        return national_rate, national_spread, fictitious
```

Then rewrite `_build_credit_state` to take the endogenous spread (locate by
`"""Assemble the national credit state from the FRED credit aggregate.`):

```python
    @staticmethod
    def _build_credit_state(
        services: ServicesProtocol,
        year: int,
        national_spread: float,
    ) -> CreditState | None:
        """Assemble the national credit state from the credit aggregate.

        vol3-money-scissors U9: ``spread_to_treasuries`` now carries the
        ENDOGENOUS fragility premium (``national_spread``), not the BAA10Y
        read. ``credit_fragility = default_rate * spread`` still fires the
        ``credit`` opposition. Returns ``None`` — never a fabricated zero
        (III.11) — when no credit aggregate source supplies total credit.
        """
        source = getattr(services, "credit_aggregate_source", None)
        if source is None:
            return None
        total_credit = source.get_total_credit(year)
        if total_credit is None:
            return None
        return CreditState(
            year=year,
            total_credit=total_credit,
            default_rate=services.defines.capital_vol3.default_rate_estimate,
            spread_to_treasuries=national_spread,
        )
```

Add the four imports to the file's import block (locate the existing
`from babylon.domain.economics.credit.types import …`):
```python
from babylon.domain.economics.credit.endogenous_interest import (
    endogenous_interest_rate,
    loan_market_tightness,
)
from babylon.domain.economics.tick.graph_bridge import (
    economy_wide_profit_rate,
    reserve_army_signal,
)
```
(`write_national_financial_state_to_graph` and `NationalFinancialParameters` are
already imported.)

- [ ] **Step 4: Run test to verify it passes**

Run: `mise run test:q -- tests/unit/economics/tick/test_system.py`
Expected: PASS for `TestNationalFinancialState`. Other tests in the file that
unpack the 4-tuple will now error — those are fixed in U9.8 (expected;
`mise run test:q` scopes to this file, and U9.8's brief lists them).

- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(economics): endogenous national interest rate replaces the FRED read

_compute_national_financial_state now computes i from the average rate of
profit and loan-market tightness (Capital Vol. III Part V), publishes an
EndogenousInterestRate, and returns a 3-tuple. _build_credit_state takes the
endogenous spread. Rate is now total — never None.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.8: migrate call sites — drop the unavailable-reason path, switch the config gate

**Files:**
- Modify: `src/babylon/domain/economics/tick/system/__init__.py`
  (`_compute_financial_layer`, `_compute_county_financial_state`,
  `_assess_county_financial_crisis`, the config gate)
- Test: `tests/unit/economics/tick/test_system.py` (4-tuple unpack sites),
  `tests/unit/economics/tick/test_financial_integration.py`

**Interfaces:**
- Consumes: the U9.7 3-tuple.
- Produces: `_compute_county_financial_state(self, fips, county, services, year, national_rate: float, fictitious, national_spread: float = 0.0)` — the
  `interest_unavailable_reason` parameter **removed**, `national_rate` **non-Optional**.

- [ ] **Step 1: Write the failing test**

The red is the now-dead skip branch. Add to `tests/unit/economics/tick/test_system.py`:

```python
    def test_county_distribution_runs_without_an_interest_calculator(self) -> None:
        """U9: the endogenous rate makes the distribution reachable even when
        no FRED interest calculator is wired — the config gate is the
        distribution calculator, not the (now calibration-only) interest one."""
        system, services, graph, year = self._financial_fixture_with_surplus()
        services = services.model_copy(update={"interest_calculator": None})
        out = system._compute_financial_layer(
            services=services, county_states=..., _national_params=..., year=year, graph=graph
        )  # fill the ... from the existing helper's call shape
        # A county with positive surplus now carries a surplus_distribution
        # (national_rate is a real endogenous float, never None -> no skip).
        assert any(
            getattr(c, "surplus_distribution", None) is not None for c in out.values()
        )
```

> Adapt the `...` placeholders to the file's existing `_compute_financial_layer`
> call convention (reuse whatever fixture builds `county_states`/`_national_params`).

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/economics/tick/test_system.py::TestFinancialLayer::test_county_distribution_runs_without_an_interest_calculator`
Expected: FAIL — the county states come back with `surplus_distribution is None` because the current gate `if services.interest_calculator is None: return county_states` short-circuits the whole layer.

- [ ] **Step 3: Write minimal implementation**

1. **Config gate** — locate `if services.interest_calculator is None:` in
   `_compute_financial_layer` and replace with:
   ```python
        # U9: the interest rate is endogenous; the layer needs the
        # distribution calculator, not the (calibration-only) interest one.
        if services.distribution_calculator is None:
            return county_states
   ```
2. **3-tuple unpack** — locate
   `national_rate, national_spread, fictitious, interest_unavailable_reason = (`
   and replace with:
   ```python
        national_rate, national_spread, fictitious = (
            self._compute_national_financial_state(services, year, graph)
        )
   ```
   and drop `interest_unavailable_reason=interest_unavailable_reason,` from the
   `_compute_county_financial_state(...)` call.
3. **`_compute_county_financial_state`** — remove the `interest_unavailable_reason`
   parameter; change `national_rate: float | None` to `national_rate: float`;
   **delete** the dead `if national_rate is None:` skip branch (locate the comment
   `# Honest absence (Constitution III.11): the national rate` — remove that
   branch and de-indent the `else:` body). The financial-crisis guard
   `and national_rate is not None` becomes always-true — remove that clause.
4. **`_assess_county_financial_crisis`** — the `national_rate: float,  # noqa: ARG002`
   signature is unchanged; drop the now-stale docstring sentence about
   `national_rate` being retained "so the call site reads symmetrically" if it
   references the removed None path (locate `national_rate is None` in the
   docstring, if present).

- [ ] **Step 4: Run tests to verify they pass**

Run: `mise run test:q -- tests/unit/economics/tick/test_system.py tests/unit/economics/tick/test_financial_integration.py`
Expected: PASS. Fix any residual 4-tuple unpack in `test_system.py` (the site near
`_national_rate, _spread, fictitious, _reason = system._compute_national_financial_state(`)
to the 3-tuple form as part of this task.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "refactor(economics): drop interest-unavailable path; endogenous rate is total

3-tuple producer contract threaded through _compute_financial_layer /
_compute_county_financial_state / _assess_county_financial_crisis; the dead
national_rate-is-None skip removed; config gate switched to the distribution
calculator so the endogenous rate works without a FRED interest calculator.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U9.9: FRED → calibration test; verify the unit lands clean

**Files:**
- Test: `tests/unit/config/test_interest_share_calibration.py` *(create)*
- No production change beyond confirming the unit is green.

**Interfaces:**
- Consumes: `tests/fixtures/vol3_fred_series.json` (committed, U1.2),
  `capital_vol3.interest_profit_share_base`.

- [ ] **Step 1: Write the failing test**

Create `tests/unit/config/test_interest_share_calibration.py`:

```python
"""U9.9: FRED is now CALIBRATION, not a runtime input — this test is the
durable spec (Constitution III.12) that interest_profit_share_base tracks the
real net-interest / profit ratio. If the coefficient is regenerated it is
re-checked against data, not vibes."""

from __future__ import annotations

import pytest

from babylon.config.defines import GameDefines


@pytest.mark.unit
def test_base_share_matches_the_fred_net_interest_to_profit_band() -> None:
    base = GameDefines.load_default().capital_vol3.interest_profit_share_base
    # FRED W273RC1 / A053RC1Q027SBEA calm-year band (design §1.3): 0.25-0.35.
    assert 0.25 <= base <= 0.35
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/config/test_interest_share_calibration.py`
Expected: PASS immediately IF U9.2's default landed (0.30 ∈ [0.25, 0.35]); the
"red" for this documentation test is degenerate. Per Global Constraint M1 this is
acceptable **only** because no production code is written here — if you prefer a
real red, temporarily set the default to 0.40, watch it fail, restore.

- [ ] **Step 3: Verify the whole unit is green**

Run: `mise run check:quick` then scoped: `mise run test:q -- tests/unit/economics/credit tests/unit/economics/tick tests/unit/config`
Expected: PASS. Then confirm no `national_rate is None`/`interest_unavailable_reason`
survivors:
```bash
rg -n 'interest_unavailable_reason|national_rate is None' src/babylon tests
```
Expected: **no matches** in `src/babylon`; test matches only in files this unit
rewrote (verify each is the endogenous assertion, not the old absence one).

- [ ] **Step 4: Determinism spot-check (no baseline regen — U8 owns that)**

Run: `mise run qa:regression` — it WILL differ from the current baselines (the
distribution's interest term now computes every modeled year). **Do not
regenerate.** Record the diff summary as evidence for U8.3. If it *matches*, STOP —
that means the endogenous rate is not reaching the distribution and U9 is inert.

- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(calibration): pin interest_profit_share_base to the FRED band (U9 close)

FRED demoted to calibration; the base share tracks net-interest/profit.
Endogenous interest unit complete; baselines move (U8 ceremony owns regen).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.0: Empirical determinism proof — two independent processes, per-tick construction cadence

**Files:**
- Create: `tests/unit/tools/test_regression_construction_cadence_determinism.py`
- Test: `tests/unit/tools/test_regression_construction_cadence_determinism.py`

**Interfaces:**
- Consumes: `tools/regression_test.py`'s `generate --scenario <name> --dense --output <dir>` CLI (existing, `tools/regression_test.py:1107-1126`); indirectly exercises `tests/fixtures/vol3_fred_series.json` (U1) via the `calculator_overrides` U1 threads into `step()` at `tools/regression_test.py`'s scenario-run call site (was line 585 pre-U1: `state = step(state, sim_config, persistent_context, defines)`); exercises `ServiceContainer.create(config, effective_defines, **overrides)` (`src/babylon/engine/simulation_engine.py:527`).
- Produces: nothing consumed downstream — this is a standalone empirical proof, run once as evidence for U8's ceremony gate. Its PASS is cited in `reports/vol3-baseline-delta.md` (U8.3) as the answer to design §5 hazard 2.

- [ ] **Step 1: Write the failing test**
```python
"""Empirical proof that the per-tick ServiceContainer.create(**overrides)
cadence (simulation_engine.py:527) is deterministic across independent OS
processes, now that tools/regression_test.py threads Vol III financial
calculator_overrides through it (U1, D4's committed FRED fixture).

Design spec 2026-07-18-vol3-money-scissors-design.md section 5, hazard 2:
"Same-inputs -> same-outputs across every construction site must be
verified EMPIRICALLY before U7, not inferred from reading code. ADR056's
precedent applies: the planned determinism proof was wrong and only an
empirical run caught it." ADR056 (spec-102) found that a determinism check
performed by re-running the SAME interpreter in-process, or by comparing a
hash that turned out to embed run-scoped state, both gave false confidence.
This test follows that precedent literally: it spawns two genuinely
separate ``python`` processes (never two in-process calls, which would
share one PYTHONHASHSEED and could hide a hash-randomization-dependent
set/dict iteration-order bug) and byte-compares their output. Per
.mise.toml and tests/conftest.py, BLAS thread counts are pinned to 1 for
determinism but PYTHONHASHSEED is deliberately left unpinned — each
subprocess below gets Python's normal fresh random hash seed, so a
byte-identical result here is real cross-process evidence, not an artifact
of test isolation.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
REGRESSION_TOOL = REPO_ROOT / "tools" / "regression_test.py"

# imperial_circuit is the 4-node default scenario -- the first key in
# tools/regression_test.py's SCENARIOS dict and the scenario every other
# behavioral-contract test in this suite treats as the canonical case.
DETERMINISM_SCENARIO = "imperial_circuit"


def _run_generate(output_dir: Path) -> subprocess.CompletedProcess[str]:
    """Run ``regression_test.py generate`` for one scenario in its own process.

    Args:
        output_dir: Directory the subprocess writes
            ``<scenario>.json`` and ``dense/<scenario>.csv`` into.

    Returns:
        The completed subprocess result (stdout/stderr captured as text).
    """
    return subprocess.run(
        [
            sys.executable,
            str(REGRESSION_TOOL),
            "generate",
            "--scenario",
            DETERMINISM_SCENARIO,
            "--dense",
            "--output",
            str(output_dir),
        ],
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
        env=os.environ.copy(),
        timeout=120,
    )


def test_two_independent_processes_produce_byte_identical_dense_trace(
    tmp_path: Path,
) -> None:
    """Two separate `generate` subprocesses for the same scenario must agree byte-for-byte."""
    dir_a = tmp_path / "run_a"
    dir_b = tmp_path / "run_b"

    result_a = _run_generate(dir_a)
    result_b = _run_generate(dir_b)

    assert result_a.returncode == 0, f"run A failed: {result_a.stderr}"
    assert result_b.returncode == 0, f"run B failed: {result_b.stderr}"

    data_a = json.loads((dir_a / f"{DETERMINISM_SCENARIO}.json").read_text())
    data_b = json.loads((dir_b / f"{DETERMINISM_SCENARIO}.json").read_text())
    # generated_at is a wall-clock ISO timestamp -- the only field two
    # independent runs are allowed to disagree on. Strip before comparing.
    data_a.pop("generated_at", None)
    data_b.pop("generated_at", None)
    assert data_a == data_b, (
        "sampled-checkpoint JSON diverged between two independent processes "
        "running the identical scenario -- the per-tick ServiceContainer "
        "construction cadence is NOT deterministic"
    )

    csv_a = (dir_a / "dense" / f"{DETERMINISM_SCENARIO}.csv").read_bytes()
    csv_b = (dir_b / "dense" / f"{DETERMINISM_SCENARIO}.csv").read_bytes()
    assert csv_a == csv_b, (
        "dense per-tick trace CSV diverged between two independent processes "
        "running the identical scenario -- the per-tick ServiceContainer "
        "construction cadence is NOT deterministic"
    )
```
- [ ] **Step 2: Prove the comparison actually bites (RED)**

Save the Step 1 file, then temporarily make the two runs differ *by construction* —
in `_run_generate`, replace the args list with:
```python
    args = [sys.executable, str(REGRESSION_TOOL), "generate", "--scenario",
            "two_node" if output_dir.name == "run_b" else DETERMINISM_SCENARIO,
            "--dense", "--output", str(output_dir)]
```
Run: `mise run test:q -- tests/unit/tools/test_regression_construction_cadence_determinism.py::test_two_independent_processes_produce_byte_identical_dense_trace`
Expected: FAIL — the byte-comparison reports a divergence, proving it reads the
artifact it claims to compare rather than passing vacuously. (A missing file is
NOT a red phase.)
**Restore `_run_generate` to its Step 1 form before Step 4.**

- [ ] **Step 3: Write minimal implementation**
The test file written in Step 1 IS the deliverable — there is no separate production code to add. Save the exact file from Step 1 at `tests/unit/tools/test_regression_construction_cadence_determinism.py`.
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_regression_construction_cadence_determinism.py`
Expected: PASS. Record the actual pytest output verbatim (not summarized) as the empirical evidence line in `reports/vol3-baseline-delta.md`'s Verification Evidence section (U8.3) — e.g. `1 passed in 4.82s`. If it FAILS, STOP: this means U1-U7 introduced real non-determinism (most likely a `set`/`dict`-over-unsorted-keys iteration inside one of the new Vol III calculator paths, or a hash-seed-sensitive FRED-fixture lookup) — this is a defect to fix before proceeding to U8.2, not a baseline to accept.
- [ ] **Step 5: Commit**
```bash
mise run commit -- "test(tools): empirical two-process determinism proof for the per-tick ServiceContainer cadence

ADR056 precedent: the planned determinism proof was wrong and only an
empirical run caught it. Two independent OS processes, unpinned
PYTHONHASHSEED, byte-compared sampled JSON + dense CSV.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.1: Shared agent-legible finding formatter

**Files:**
- Create: `src/babylon/sentinels/report.py`
- Test: `tests/unit/sentinels/test_report.py`

**Interfaces:**
- Consumes: nothing (layer-0.5, imports only stdlib)
- Produces: `babylon.sentinels.report.finding(error_class, symbol, file, line, problem, remedy) -> str` — the single formatting entry point every new sensor in U7 emits through.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the agent-legible sentinel finding formatter.

Every U7 sensor emits its findings through :func:`babylon.sentinels.report.finding`
so that a coding agent reading a red gate always gets the same four facts in the
same order: which error CLASS, which SYMBOL, where (``file:line``), and what to DO.
"""

from __future__ import annotations

import pytest

from babylon.sentinels.report import finding

pytestmark = pytest.mark.unit


def test_finding_contains_all_five_fields_in_order() -> None:
    """The rendered string carries class, symbol, file:line, problem, remedy."""
    rendered = finding(
        error_class="computed-but-never-consumed",
        symbol="POLE_READINGS_ATTR",
        file="src/babylon/engine/systems/contradiction.py",
        line=89,
        problem="written to the graph but no production module reads it",
        remedy="add a consumer_file to the liveness registry row, or declare dormant_reason",
    )
    assert rendered.startswith("[computed-but-never-consumed]")
    assert "POLE_READINGS_ATTR" in rendered
    assert "src/babylon/engine/systems/contradiction.py:89" in rendered
    assert "written to the graph but no production module reads it" in rendered
    assert rendered.endswith(
        "REMEDY: add a consumer_file to the liveness registry row, "
        "or declare dormant_reason"
    )


def test_finding_accepts_line_zero_for_file_level_findings() -> None:
    """Line 0 means 'the whole file', and renders without a line suffix."""
    rendered = finding(
        error_class="gate-blindness",
        symbol="create_financial_services",
        file="tools/regression_test.py",
        line=0,
        problem="the gate harness injects none of this factory's service keys",
        remedy="build calculator_overrides from the committed FRED fixture",
    )
    assert "tools/regression_test.py " in rendered
    assert "tools/regression_test.py:0" not in rendered


def test_finding_rejects_a_blank_remedy() -> None:
    """A finding without a remedy is not agent-legible and is refused loudly."""
    with pytest.raises(ValueError, match="remedy"):
        finding(
            error_class="correct-but-inert",
            symbol="SomeSystem",
            file="src/babylon/engine/systems/some.py",
            line=1,
            problem="runs but nothing reads its outputs",
            remedy="   ",
        )


def test_finding_rejects_a_blank_error_class() -> None:
    """The error class names the failure taxonomy; blank is refused loudly."""
    with pytest.raises(ValueError, match="error_class"):
        finding(
            error_class="",
            symbol="SomeSystem",
            file="src/babylon/engine/systems/some.py",
            line=1,
            problem="runs but nothing reads its outputs",
            remedy="wire a consumer",
        )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_report.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.report'`
- [ ] **Step 3: Write minimal implementation**
```python
"""The agent-legible finding format shared by the U7 sentinel family.

A sentinel finding is read by a coding agent, not a human dashboard. To be
actionable it must answer five questions in one line, always in the same order:

1. **which error class** — the named failure taxonomy (``correct-but-inert``,
   ``computed-but-never-consumed``, ``gate-blindness``,
   ``intensive-aggregation``, ``undeclared-coupling``);
2. **which symbol** — the offending name, not a vague area;
3. **where** — repo-relative ``file:line`` the agent can open directly;
4. **what to do** — a concrete remedy, never "investigate".

The rendered shape is::

    [<error-class>] <symbol> @ <file>:<line> — <problem> | REMEDY: <remedy>

:mod:`babylon.sentinels.base`'s runner prefixes this with its sensor tag, so a
full advisory line reads ``LIVENESS ADVISORY [label]: [class] symbol @ ...``.

Layer 0.5 (stdlib only): importable by every sentinel package.
"""

from __future__ import annotations


def finding(
    *,
    error_class: str,
    symbol: str,
    file: str,
    line: int,
    problem: str,
    remedy: str,
) -> str:
    """Render one agent-legible sentinel finding.

    :param error_class: The named failure taxonomy this finding belongs to.
    :param symbol: The offending symbol (class, function, constant, graph key).
    :param file: Repo-relative path to the offending source file.
    :param line: 1-indexed line of the offending symbol; ``0`` means the whole
        file (rendered without a line suffix).
    :param problem: One clause stating what is wrong, in the present tense.
    :param remedy: One clause stating the concrete fix — never "investigate".
    :returns: The single-line finding string.
    :raises ValueError: If ``error_class``, ``symbol``, ``file``, ``problem`` or
        ``remedy`` is blank (a finding missing any of the five facts is not
        agent-legible, Constitution III.11).
    """
    for label, value in (
        ("error_class", error_class),
        ("symbol", symbol),
        ("file", file),
        ("problem", problem),
        ("remedy", remedy),
    ):
        if not value.strip():
            raise ValueError(f"sentinel finding: {label} must be non-empty")
    location = f"{file}:{line}" if line > 0 else f"{file} "
    return f"[{error_class}] {symbol} @ {location} — {problem} | REMEDY: {remedy}"
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_report.py`
Expected: PASS (4 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): agent-legible finding formatter for the U7 family

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.2: Shared AST helpers the six new sensors read source with

**Files:**
- Modify: `src/babylon/sentinels/_ast.py` (append after the existing `eventtype_names_in_module`, currently the last function in the file)
- Test: `tests/unit/sentinels/test_ast_helpers.py`

**Interfaces:**
- Consumes: `babylon.sentinels.base.SentinelCheckError`
- Produces: `parse_module(path) -> ast.Module`, `referenced_names(path) -> set[str]`, `coupling_edges(path) -> tuple[tuple[str, str, str], ...]`, `returned_dict_keys(path, func_name) -> tuple[str, ...]` — used by U7.3–U7.9.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the shared AST helpers added for the U7 sentinel family.

These helpers let a layer-0.5 sensor prove facts about ``domain``/``engine``/
``tools`` source WITHOUT importing it (the import-linter contract in
``pyproject.toml`` forbids the import; the sensors must stay cheap and static).
"""

from __future__ import annotations

import ast
from pathlib import Path

import pytest

from babylon.sentinels._ast import (
    coupling_edges,
    parse_module,
    referenced_names,
    returned_dict_keys,
)
from babylon.sentinels.base import SentinelCheckError

pytestmark = pytest.mark.unit

_REPO_ROOT: Path = Path(__file__).resolve().parents[3]


def test_parse_module_returns_a_module(tmp_path: Path) -> None:
    """A well-formed file parses to an ``ast.Module``."""
    target = tmp_path / "ok.py"
    target.write_text("X = 1\n", encoding="utf-8")
    assert isinstance(parse_module(target), ast.Module)


def test_parse_module_raises_on_missing_file(tmp_path: Path) -> None:
    """A missing file is infrastructure failure, never a silent empty result."""
    with pytest.raises(SentinelCheckError, match="cannot read"):
        parse_module(tmp_path / "absent.py")


def test_parse_module_raises_on_syntax_error(tmp_path: Path) -> None:
    """An unparseable file is infrastructure failure (exit 2, not a false pass)."""
    target = tmp_path / "broken.py"
    target.write_text("def (:\n", encoding="utf-8")
    with pytest.raises(SentinelCheckError, match="cannot parse"):
        parse_module(target)


def test_referenced_names_covers_names_attributes_keywords_and_strings(
    tmp_path: Path,
) -> None:
    """Every way a module can mention a symbol counts as a reference."""
    target = tmp_path / "refs.py"
    target.write_text(
        "\n".join(
            [
                "import thing",
                "def f(graph):",
                "    graph.update_node(node_id, price_divergence=1.0)",
                "    return thing.fictitious_log, attrs.get('national_financial')",
            ]
        ),
        encoding="utf-8",
    )
    names = referenced_names(target)
    assert "graph" in names
    assert "update_node" in names
    assert "price_divergence" in names
    assert "fictitious_log" in names
    assert "national_financial" in names


def test_coupling_edges_reads_the_real_catalog() -> None:
    """The production catalog's declared ``Coupling(...)`` literals are extracted."""
    edges = coupling_edges(
        _REPO_ROOT / "src/babylon/domain/dialectics/instances/catalog.py"
    )
    assert ("surplus_distribution", "debt_spiral", "transforms") in edges
    assert ("credit", "financial", "transforms") in edges
    assert ("capital_labor", "imperial", "antagonizes") in edges


def test_coupling_edges_skips_non_literal_calls(tmp_path: Path) -> None:
    """A computed endpoint yields no row rather than raising."""
    target = tmp_path / "couplings.py"
    target.write_text(
        "E = (\n"
        "    Coupling(source='a', target='b', kind='feeds'),\n"
        "    Coupling(source=key, target='c', kind='feeds'),\n"
        ")\n",
        encoding="utf-8",
    )
    assert coupling_edges(target) == (("a", "b", "feeds"),)


def test_returned_dict_keys_reads_the_real_financial_factory() -> None:
    """The Vol III factory's returned service-key set is extracted statically."""
    keys = returned_dict_keys(
        _REPO_ROOT / "src/babylon/domain/economics/factory.py",
        "create_financial_services",
    )
    assert "distribution_calculator" in keys
    assert "financial_crisis_assessor" in keys
    assert "fictitious_capital_calculator" in keys


def test_returned_dict_keys_raises_on_unknown_function(tmp_path: Path) -> None:
    """Naming a function the file lacks is infrastructure failure, not silence."""
    target = tmp_path / "mod.py"
    target.write_text("def g():\n    return {'a': 1}\n", encoding="utf-8")
    with pytest.raises(SentinelCheckError, match="no function"):
        returned_dict_keys(target, "does_not_exist")
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_ast_helpers.py`
Expected: FAIL with `ImportError: cannot import name 'coupling_edges' from 'babylon.sentinels._ast'`
- [ ] **Step 3: Write minimal implementation**
Append to `src/babylon/sentinels/_ast.py`:
```python
def parse_module(path: Path) -> ast.Module:
    """Read and parse ``path`` with :mod:`ast`, failing loudly on either error.

    The single shared entry point for the U7 sensors: a missing or unparseable
    source is an *infrastructure* failure (exit 2), never an empty result that
    would read as a clean pass (Constitution III.11).

    :param path: Source file to parse.
    :returns: The parsed module.
    :raises SentinelCheckError: If the file cannot be read or cannot be parsed.
    """
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise SentinelCheckError(f"cannot read {path}: {exc}") from exc
    try:
        return ast.parse(source, filename=str(path))
    except SyntaxError as exc:
        raise SentinelCheckError(f"cannot parse {path}: {exc}") from exc


def referenced_names(path: Path) -> set[str]:
    """Collect every symbol a module *mentions*, however it mentions it.

    A consumer can reach an output four ways: a bare name (``price_divergence``),
    an attribute (``axis.fictitious_log``), a keyword argument
    (``update_node(..., price_divergence=x)``), or a string key
    (``attrs.get("national_financial")``). All four count as a reference — the
    liveness and coupling sensors ask "does this file read that output?", and a
    string-keyed graph read is as real a reader as an imported constant.

    :param path: Source file to parse.
    :returns: The union of referenced names, attribute names, keyword-argument
        names, and string-literal constants.
    :raises SentinelCheckError: If the file is missing or unparseable.
    """
    tree = parse_module(path)
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            names.add(node.id)
        elif isinstance(node, ast.Attribute):
            names.add(node.attr)
        elif isinstance(node, ast.keyword) and node.arg is not None:
            names.add(node.arg)
        elif isinstance(node, ast.Constant) and isinstance(node.value, str):
            names.add(node.value)
    return names


def coupling_edges(path: Path) -> tuple[tuple[str, str, str], ...]:
    """Extract the declared ``Coupling(source=, target=, kind=)`` literals.

    Reads the dialectics catalog statically — :mod:`babylon.sentinels` may not
    import ``babylon.domain`` (import-linter contract, ``pyproject.toml``) — and
    returns the declared coupling map as plain triples. A call whose endpoints
    are not string literals is skipped (a computed edge is not a *declared* one).

    :param path: The module declaring the ``Coupling(...)`` literals.
    :returns: ``(source, target, kind)`` triples, in source order.
    :raises SentinelCheckError: If the file is missing or unparseable.
    """
    tree = parse_module(path)
    edges: list[tuple[str, str, str]] = []
    for node in ast.walk(tree):
        if not (
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name)
            and node.func.id == "Coupling"
        ):
            continue
        parts: dict[str, str] = {}
        for kw in node.keywords:
            if (
                kw.arg in {"source", "target", "kind"}
                and isinstance(kw.value, ast.Constant)
                and isinstance(kw.value.value, str)
            ):
                parts[kw.arg] = kw.value.value
        if len(parts) == 3:
            edges.append((parts["source"], parts["target"], parts["kind"]))
    return tuple(edges)


def returned_dict_keys(path: Path, func_name: str) -> tuple[str, ...]:
    """Extract the string keys of the dict literal a named function returns.

    The service factories (``create_economics_services``,
    ``create_financial_services``) each end in one dict literal whose keys ARE
    the estate the DoD gate is meant to inject. Reading them statically lets the
    gate-blindness sensor compare estate against harness without importing
    ``babylon.domain``.

    :param path: Source file defining ``func_name``.
    :param func_name: The module-level function whose returned dict to read.
    :returns: The string keys of the last returned dict literal, sorted.
    :raises SentinelCheckError: If the file is missing/unparseable, the function
        is absent, or it returns no dict literal.
    """
    tree = parse_module(path)
    target: ast.FunctionDef | None = None
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name == func_name:
            target = node
    if target is None:
        raise SentinelCheckError(f"{path}: no function {func_name!r} at module level")
    keys: set[str] = set()
    for node in ast.walk(target):
        if isinstance(node, ast.Return) and isinstance(node.value, ast.Dict):
            keys.update(
                k.value
                for k in node.value.keys
                if isinstance(k, ast.Constant) and isinstance(k.value, str)
            )
    if not keys:
        raise SentinelCheckError(f"{path}:{func_name} returns no dict literal")
    return tuple(sorted(keys))
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_ast_helpers.py`
Expected: PASS (8 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): shared AST helpers for the U7 sensor family

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.3: Liveness registry — declared outputs and their readers

**Files:**
- Create: `src/babylon/sentinels/liveness/__init__.py`
- Create: `src/babylon/sentinels/liveness/registry.py`
- Test: `tests/unit/sentinels/test_liveness_registry.py`

**Interfaces:**
- Consumes: nothing (pure Pydantic data)
- Produces: `babylon.sentinels.liveness.registry.LivenessRow` and `LIVENESS_ROWS: tuple[LivenessRow, ...]` — consumed by U7.4 and U7.5.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the declared liveness registry (correct-but-inert / never-consumed).

The registry is the *declared* half of the two liveness sensors: every output a
production producer stamps is either claimed by at least one production consumer
file, or explicitly declared dormant WITH A REASON. There is no third state —
that is the whole point of the class this gate exists to catch.
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.sentinels.liveness.registry import LIVENESS_ROWS, LivenessRow

pytestmark = pytest.mark.unit


def test_registry_declares_the_known_producers() -> None:
    """The seeded rows cover the outputs this program investigated."""
    names = {row.name for row in LIVENESS_ROWS}
    assert {
        "price_divergence",
        "market_balance",
        "pole_readings",
        "national_financial",
        "ground_rent_path_a",
        "fictitious_capital_stock",
        "debt_spiral_threshold",
    } <= names


def test_every_row_is_live_or_dormant_with_a_reason() -> None:
    """No row may be silently output-with-no-reader — that IS the error class."""
    for row in LIVENESS_ROWS:
        assert row.consumer_files or row.dormant_reason, row.name


def test_pole_readings_is_the_declared_dormant_row() -> None:
    """``pole_readings`` is a live producer with zero production readers (spec 3.7)."""
    row = next(r for r in LIVENESS_ROWS if r.name == "pole_readings")
    assert row.consumer_files == ()
    assert "sentinel" in row.dormant_reason.lower()


def test_row_rejects_output_with_neither_consumer_nor_reason() -> None:
    """A row that is neither consumed nor declared dormant is refused at import."""
    with pytest.raises(ValidationError, match="dormant_reason"):
        LivenessRow(
            name="orphan",
            producer_file="src/babylon/engine/systems/market_scissors.py",
            producer_symbol="MarketScissorsSystem",
            output_symbol="orphan_output",
            consumer_files=(),
            material_relation="none",
        )


def test_row_rejects_a_non_python_producer_path() -> None:
    """Producer/consumer paths must be ``.py`` source the AST sensors can read."""
    with pytest.raises(ValidationError, match=r"\.py"):
        LivenessRow(
            name="bad_path",
            producer_file="src/babylon/engine/systems/market_scissors.txt",
            producer_symbol="MarketScissorsSystem",
            output_symbol="x",
            consumer_files=("web/game/engine_bridge.py",),
            material_relation="none",
        )


def test_row_rejects_a_non_python_consumer_path() -> None:
    """A consumer path must also be ``.py`` — the sensor parses it."""
    with pytest.raises(ValidationError, match=r"\.py"):
        LivenessRow(
            name="bad_consumer",
            producer_file="src/babylon/engine/systems/market_scissors.py",
            producer_symbol="MarketScissorsSystem",
            output_symbol="x",
            consumer_files=("web/game/engine_bridge.ts",),
            material_relation="none",
        )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_registry.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.liveness'`
- [ ] **Step 3: Write minimal implementation**

`src/babylon/sentinels/liveness/__init__.py`:
```python
"""The liveness sentinel — declared outputs must have declared readers.

Volume III of this codebase computed correctly and changed nothing for months:
every calculator ran, every model validated, and no output reached a consumer.
No test detected it, because "it runs" and "it matters" are different claims and
only the first was ever asserted. This sentinel asserts the second.

Two error classes, one registry:

- **correct-but-inert** — a producer (a ``System``, a service) runs but *every*
  output it declares is dormant; the whole producer is decoration.
- **computed-but-never-consumed** — one declared output has no production
  reader, and no ``dormant_reason`` explaining why that is acceptable.

The registry (:mod:`babylon.sentinels.liveness.registry`) is the declared half;
the sensors (:mod:`babylon.sentinels.liveness.checks`) prove each declaration
against source, statically, via :mod:`ast`. Per the standing owner ruling both
checks are **advisory** and local/on-demand
(``poetry run python tools/sentinel_check.py liveness``) — they never gate CI.

Layer 0.5 (same rank as :mod:`babylon.config`): imports nothing above
:mod:`babylon.models`.
"""
```

`src/babylon/sentinels/liveness/registry.py`:
```python
"""The declared source of truth for producer outputs and their readers.

Each row is one **output** a production producer stamps — a graph attribute, a
``GraphInputs`` field, a module constant — together with the production files
that read it. A row with no consumers must carry a ``dormant_reason``: dormancy
is legitimate (scaffolding awaiting its consumer) but only when *declared*, so
that an output nobody reads can never again sit undetected.

Hand-written by contract: this is a dev-time claim about the code, not
player-moddable runtime config, so it carries no regeneration machinery. The
static sensors in :mod:`babylon.sentinels.liveness.checks` prove each claim.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, model_validator


class LivenessRow(BaseModel):
    """One declared producer output and the production files that read it.

    Frozen and ``extra="forbid"`` so a malformed row fails loudly at import
    (Constitution III.11) rather than quietly at check time.

    :ivar name: stable identity for the output (e.g. ``"price_divergence"``).
    :ivar producer_file: repo-relative ``.py`` path that stamps the output.
    :ivar producer_symbol: the producing ``System``/function/class; the
        correct-but-inert sensor groups rows by this name.
    :ivar output_symbol: the name a consumer must mention to be counted as a
        reader — a graph-attribute string, a field name, or a constant name.
    :ivar consumer_files: repo-relative ``.py`` paths that read the output in
        PRODUCTION (tests and sentinels do not count; a test-only reader is
        exactly the false liveness this gate exists to expose).
    :ivar dormant_reason: why this output legitimately has no reader yet;
        required when ``consumer_files`` is empty.
    :ivar material_relation: the material relation the output carries
        (Aleksandrov Test) — why anything downstream should want it.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    name: str
    producer_file: str
    producer_symbol: str
    output_symbol: str
    consumer_files: tuple[str, ...] = ()
    dormant_reason: str = ""
    material_relation: str

    @model_validator(mode="after")
    def _validate_shape(self) -> LivenessRow:
        """Reject blank identities, non-``.py`` paths, and undeclared dormancy.

        :returns: ``self`` when valid.
        :raises ValueError: If ``name``/``output_symbol``/``producer_symbol`` is
            blank, any declared path is not a ``.py`` file, or the row has
            neither a consumer nor a ``dormant_reason``.
        """
        for label, value in (
            ("name", self.name),
            ("producer_symbol", self.producer_symbol),
            ("output_symbol", self.output_symbol),
            ("material_relation", self.material_relation),
        ):
            if not value.strip():
                raise ValueError(f"LivenessRow.{label} must be non-empty")
        if not self.producer_file.endswith(".py"):
            raise ValueError(
                f"{self.name!r}: producer_file must be a .py path, got {self.producer_file!r}"
            )
        for consumer in self.consumer_files:
            if not consumer.endswith(".py"):
                raise ValueError(
                    f"{self.name!r}: consumer_files entries must be .py paths, got {consumer!r}"
                )
        if not self.consumer_files and not self.dormant_reason.strip():
            raise ValueError(
                f"{self.name!r}: an output with no consumer_files must declare a "
                "dormant_reason — undeclared dormancy is the error class this "
                "registry exists to forbid"
            )
        return self


#: Repo-relative roots, factored out for readability.
_ENGINE_SYSTEMS = "src/babylon/engine/systems"
_DIALECTICS = "src/babylon/domain/dialectics/instances"
_TICK = "src/babylon/domain/economics/tick"
_ECON_DIST = "src/babylon/domain/economics/distribution"
_ECON_CREDIT = "src/babylon/domain/economics/credit"

#: The declared producer outputs of the money/value estate.
LIVENESS_ROWS: tuple[LivenessRow, ...] = (
    LivenessRow(
        name="price_divergence",
        producer_file=f"{_ENGINE_SYSTEMS}/market_scissors.py",
        producer_symbol="MarketScissorsSystem",
        output_symbol="price_divergence",
        consumer_files=("web/game/engine_bridge.py", "web/game/map_contract.py"),
        material_relation=(
            "Per-territory divergence of market price from labour value — the "
            "scissors as the player sees it on the map lens."
        ),
    ),
    LivenessRow(
        name="market_balance",
        producer_file=f"{_ENGINE_SYSTEMS}/contradiction.py",
        producer_symbol="ContradictionSystem",
        output_symbol="market_balance",
        consumer_files=(f"{_DIALECTICS}/catalog.py",),
        material_relation=(
            "The pre-derived scissors Balance the price_value opposition measures "
            "as an adjunction defect (ADR077/ADR078)."
        ),
    ),
    LivenessRow(
        name="pole_readings",
        producer_file=f"{_ENGINE_SYSTEMS}/contradiction.py",
        producer_symbol="ContradictionSystem",
        output_symbol="pole_readings",
        consumer_files=(),
        dormant_reason=(
            "Written to the graph every tick and read only by the partition "
            "sentinel's harness (a dev-time probe, not production). Declared "
            "dormant pending the emergent-class-partition Phase 2 consumer "
            "(Program 19 / ADR070); until then it is a live producer with zero "
            "production readers, recorded rather than hidden."
        ),
        material_relation=(
            "Per-entity position on each opposition axis — the raw material of an "
            "emergent class partition."
        ),
    ),
    LivenessRow(
        name="national_financial",
        producer_file=f"{_TICK}/graph_bridge.py",
        producer_symbol="write_national_financial_state_to_graph",
        output_symbol="NATIONAL_FINANCIAL_ATTR",
        consumer_files=(
            f"{_ENGINE_SYSTEMS}/market_scissors.py",
            f"{_ENGINE_SYSTEMS}/contradiction.py",
        ),
        material_relation=(
            "The national ledger of claims on surplus — interest state and "
            "fictitious capital stock — published to the graph so a CONSEQUENCE "
            "phase System reads it in the same tick it is computed (U3)."
        ),
    ),
    LivenessRow(
        name="ground_rent_path_a",
        producer_file=f"{_ECON_DIST}/calculator.py",
        producer_symbol="DefaultDistributionCalculator",
        output_symbol="ground_rent",
        consumer_files=(f"{_TICK}/graph_bridge.py",),
        material_relation=(
            "Real FRED B230RC0Q173SBEA rental income — the landowner's claim on "
            "county surplus. Repointed into tick_ground_rent by U1.5; before that "
            "it computed correctly every year and reached no territory node."
        ),
    ),
    LivenessRow(
        name="fictitious_capital_stock",
        producer_file=f"{_ECON_CREDIT}/fictitious_capital.py",
        producer_symbol="DefaultFictitiousCapitalCalculator",
        output_symbol="fictitious_capital",
        consumer_files=(f"{_ENGINE_SYSTEMS}/market_scissors.py",),
        material_relation=(
            "Government + corporate + household claims on future surplus. Published "
            "by U3 and read by the U6 monetary anchor; before that it died as a "
            "transient local inside _assess_county_financial_crisis after producing "
            "one financialization ratio (float | None since aedce819)."
        ),
    ),
    LivenessRow(
        name="debt_spiral_threshold",
        producer_file=f"{_ECON_DIST}/types.py",
        # Post-U2.3 reality: the module-level ``Final[float]
        # DEBT_SPIRAL_THRESHOLD`` no longer exists. U2.3 deletes it, moves the
        # value into ``GameDefines.capital_vol3.debt_spiral_threshold``, and
        # leaves a defines-backed accessor FUNCTION of the same lowercase name in
        # ``distribution/types.py``. Naming the deleted ALL-CAPS symbol here
        # would be a false claim inside a registry whose entire purpose is
        # accurate claims about the code — and one nothing would red on, because
        # neither liveness check validates ``producer_symbol`` against
        # ``producer_file``. Recorded for the sentinel roadmap: the liveness
        # registry can currently name a producer symbol that does not exist,
        # which is the same class of unverified claim these sensors exist to
        # catch.
        producer_symbol="debt_spiral_threshold",
        output_symbol="debt_spiral_threshold",
        consumer_files=(f"{_ENGINE_SYSTEMS}/contradiction.py",),
        material_relation=(
            "The accumulated-debt-to-annual-surplus ratio at which the spiral is "
            "structurally self-reinforcing — the unity point of the debt_spiral "
            "opposition (wired by U5.10; a dead constant for its whole prior "
            "life, and a defines-backed accessor since U2.3)."
        ),
    ),
    LivenessRow(
        name="serviceability_anchor",
        producer_file="src/babylon/domain/economics/monetary/anchor.py",
        producer_symbol="serviceability_anchor",
        output_symbol="serviceability_anchor",
        consumer_files=(f"{_ENGINE_SYSTEMS}/market_scissors.py",),
        material_relation=(
            "The real interest burden i/s — the share of produced surplus already "
            "claimed before the functioning capitalist sees any — which sets the "
            "ceiling on fictitious_log before the correction snap (design §3.3/§3.5.1)."
        ),
    ),
)
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_registry.py`
Expected: PASS (6 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): declared liveness registry for producer outputs

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.4: computed-but-never-consumed check + mutation proof

**Files:**
- Create: `src/babylon/sentinels/liveness/checks.py`
- Test: `tests/unit/sentinels/test_liveness_consumed.py`

**Interfaces:**
- Consumes: `babylon.sentinels.report.finding`, `babylon.sentinels._ast.referenced_names`, `babylon.sentinels.liveness.registry.LIVENESS_ROWS`
- Produces: `babylon.sentinels.liveness.checks.check_outputs_have_readers(registry=LIVENESS_ROWS) -> list[str]` and the module-level `_REPO_ROOT`.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the computed-but-never-consumed sensor.

Two tiers, per the sentinel contract:

- **Invariant** — the sensor is clean on the real :data:`LIVENESS_ROWS`: every
  non-dormant output is actually mentioned by at least one declared consumer.
- **Efficacy (MUTATION)** — the sensor REDS when the defect it exists to catch
  is injected: a declared consumer that does not in fact read the output, i.e.
  an output computed every tick and consumed by nobody.
"""

from __future__ import annotations

import pytest

from babylon.sentinels.base import SentinelCheckError
from babylon.sentinels.liveness.checks import check_outputs_have_readers
from babylon.sentinels.liveness.registry import LivenessRow

pytestmark = pytest.mark.unit


def test_real_rows_are_consumed() -> None:
    """INVARIANT: every declared non-dormant output has a real reader."""
    assert check_outputs_have_readers() == []


def test_efficacy_reds_when_the_declared_consumer_does_not_read_the_output() -> None:
    """MUTATION: inject an output no consumer mentions — the sensor must red.

    ``web/game/map_contract.py`` is a real, parseable file that contains no
    mention of ``phantom_never_read_output``; the row therefore claims a reader
    that does not exist, which is precisely computed-but-never-consumed.
    """
    injected = LivenessRow(
        name="phantom_output",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="MarketScissorsSystem",
        output_symbol="phantom_never_read_output",
        consumer_files=("web/game/map_contract.py",),
        material_relation="injected defect for the efficacy proof",
    )
    findings = check_outputs_have_readers((injected,))
    assert len(findings) == 1
    assert findings[0].startswith("[computed-but-never-consumed]")
    assert "phantom_never_read_output" in findings[0]
    assert "web/game/map_contract.py" in findings[0]
    assert "REMEDY:" in findings[0]


def test_dormant_rows_are_not_reported() -> None:
    """A declared-dormant output is silent — dormancy WITH a reason is allowed."""
    dormant = LivenessRow(
        name="declared_dormant",
        producer_file="src/babylon/engine/systems/contradiction.py",
        producer_symbol="ContradictionSystem",
        output_symbol="phantom_never_read_output",
        consumer_files=(),
        dormant_reason="awaiting the Phase 2 consumer; recorded, not hidden",
        material_relation="injected dormant row for the exemption proof",
    )
    assert check_outputs_have_readers((dormant,)) == []


def test_missing_consumer_file_is_infrastructure_failure() -> None:
    """A consumer path that does not exist is exit-2 loud, not a quiet miss."""
    broken = LivenessRow(
        name="bad_path",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="MarketScissorsSystem",
        output_symbol="price_divergence",
        consumer_files=("web/game/this_file_does_not_exist.py",),
        material_relation="injected infra failure for the loudness proof",
    )
    with pytest.raises(SentinelCheckError, match="cannot read"):
        check_outputs_have_readers((broken,))
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_consumed.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.liveness.checks'`
- [ ] **Step 3: Write minimal implementation**
```python
"""Liveness sensors — correct-but-inert and computed-but-never-consumed.

Proves, statically via :mod:`ast` (no import, no engine run — the sentinels'
layer-0.5 boundary forbids importing ``engine``/``domain``/``web``), that every
output declared in :data:`babylon.sentinels.liveness.registry.LIVENESS_ROWS` is
actually mentioned by at least one of its declared production consumers, and
that no producer is wholly dormant.

Both checks are **advisory** per the standing owner ruling: they print loudly and
locally, and never gate CI. Run:
``poetry run python tools/sentinel_check.py liveness``.
"""

from __future__ import annotations

from pathlib import Path

from babylon.sentinels._ast import referenced_names
from babylon.sentinels.liveness.registry import LIVENESS_ROWS, LivenessRow
from babylon.sentinels.report import finding

#: Repo root (this file is ``<root>/src/babylon/sentinels/liveness/checks.py``).
_REPO_ROOT: Path = Path(__file__).resolve().parents[4]


def check_outputs_have_readers(
    registry: tuple[LivenessRow, ...] = LIVENESS_ROWS,
) -> list[str]:
    """Every declared non-dormant output must be read by a declared consumer.

    For each row with consumers, parse each consumer file and assert it mentions
    ``output_symbol`` (as a name, attribute, keyword, or string key — see
    :func:`babylon.sentinels._ast.referenced_names`). A row whose consumers all
    fail to mention it is **computed-but-never-consumed**: the producer runs
    every tick and its result reaches nothing.

    :param registry: Rows to check (defaults to the real
        :data:`LIVENESS_ROWS`; injectable so the efficacy tests can supply an
        injected defect).
    :returns: Sorted agent-legible finding strings (empty when every output is
        read or declared dormant).
    :raises SentinelCheckError: If a declared consumer file is missing or
        unparseable — infrastructure failure, never a silent pass.
    """
    findings: list[str] = []
    for row in registry:
        if not row.consumer_files:
            continue
        readers = [
            consumer
            for consumer in row.consumer_files
            if row.output_symbol in referenced_names(_REPO_ROOT / consumer)
        ]
        if readers:
            continue
        findings.append(
            finding(
                error_class="computed-but-never-consumed",
                symbol=row.output_symbol,
                file=row.producer_file,
                line=0,
                problem=(
                    f"{row.producer_symbol} stamps it, but none of its declared "
                    f"consumers ({', '.join(row.consumer_files)}) mention it"
                ),
                remedy=(
                    "wire a real production reader and point consumer_files at it, "
                    "or set dormant_reason on the liveness registry row explaining "
                    "why the output legitimately has none yet"
                ),
            )
        )
    return sorted(findings)
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_consumed.py`
Expected: PASS (4 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): computed-but-never-consumed sensor with mutation proof

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.5: correct-but-inert check + mutation proof + liveness CLI

**Files:**
- Modify: `src/babylon/sentinels/liveness/checks.py` (append `check_producers_are_not_inert`, the check tuples, `_summary`, `main`)
- Modify: `tools/sentinel_check.py:20-22,48-54` (import + `_SENSORS` entry)
- Modify: `.mise.toml` (append a new task block after `[tasks."check:coverage"]`, which ends at line 124)
- Test: `tests/unit/sentinels/test_liveness_inert.py`

**Interfaces:**
- Consumes: `babylon.sentinels.liveness.registry.LIVENESS_ROWS`, `babylon.sentinels.base.run_sensor`, `babylon.sentinels.report.finding`
- Produces: `check_producers_are_not_inert(registry=LIVENESS_ROWS) -> list[str]`, `babylon.sentinels.liveness.checks.main(argv) -> int`, and the `liveness` CLI sensor name.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the correct-but-inert sensor and the liveness CLI entry point.

Correct-but-inert is the *producer-level* class: not one dead output, but a
producer whose EVERY declared output is dormant — it runs, it validates, and the
world is unchanged by it. That is Volume III's exact failure mode.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from babylon.sentinels.liveness.checks import check_producers_are_not_inert, main
from babylon.sentinels.liveness.registry import LivenessRow

pytestmark = pytest.mark.unit

_TOOL_PATH = Path(__file__).resolve().parents[3] / "tools" / "sentinel_check.py"


def test_real_producers_are_not_inert() -> None:
    """INVARIANT: no declared producer has an all-dormant output set."""
    assert check_producers_are_not_inert() == []


def test_efficacy_reds_on_a_wholly_dormant_producer() -> None:
    """MUTATION: a producer whose every output is dormant must be reported."""
    inert_a = LivenessRow(
        name="inert_one",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="PhantomInertSystem",
        output_symbol="phantom_a",
        consumer_files=(),
        dormant_reason="injected: awaiting a consumer",
        material_relation="injected defect for the efficacy proof",
    )
    inert_b = LivenessRow(
        name="inert_two",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="PhantomInertSystem",
        output_symbol="phantom_b",
        consumer_files=(),
        dormant_reason="injected: awaiting a consumer",
        material_relation="injected defect for the efficacy proof",
    )
    findings = check_producers_are_not_inert((inert_a, inert_b))
    assert len(findings) == 1
    assert findings[0].startswith("[correct-but-inert]")
    assert "PhantomInertSystem" in findings[0]
    assert "phantom_a" in findings[0] and "phantom_b" in findings[0]
    assert "REMEDY:" in findings[0]


def test_a_producer_with_one_live_output_is_not_inert() -> None:
    """One live output is enough — inertness is about the producer, not the row."""
    dormant = LivenessRow(
        name="mixed_dormant",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="MixedSystem",
        output_symbol="phantom_a",
        consumer_files=(),
        dormant_reason="injected: awaiting a consumer",
        material_relation="injected row",
    )
    live = LivenessRow(
        name="mixed_live",
        producer_file="src/babylon/engine/systems/market_scissors.py",
        producer_symbol="MixedSystem",
        output_symbol="price_divergence",
        consumer_files=("web/game/engine_bridge.py",),
        material_relation="injected row",
    )
    assert check_producers_are_not_inert((dormant, live)) == []


def test_main_exits_zero_because_liveness_is_advisory() -> None:
    """The sensor is advisory: findings print, the process never gates."""
    assert main([]) == 0


def test_cli_dispatches_the_liveness_sensor() -> None:
    """``sentinel_check.py liveness`` routes to this sensor and exits cleanly."""
    result = subprocess.run(
        [sys.executable, str(_TOOL_PATH), "liveness"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0
    assert "Liveness" in result.stdout
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_inert.py`
Expected: FAIL with `ImportError: cannot import name 'check_producers_are_not_inert' from 'babylon.sentinels.liveness.checks'`
- [ ] **Step 3: Write minimal implementation**

Append to `src/babylon/sentinels/liveness/checks.py`, and add `import argparse`, `import sys`, and `from babylon.sentinels.base import LabelledCheck, run_sensor` to its imports:
```python
def check_producers_are_not_inert(
    registry: tuple[LivenessRow, ...] = LIVENESS_ROWS,
) -> list[str]:
    """No declared producer may have an all-dormant output set.

    Groups rows by ``producer_symbol``. A producer every one of whose outputs is
    dormant is **correct-but-inert**: it executes, its models validate, and
    nothing downstream changes because of it — the Volume III failure mode. One
    live output redeems the producer; zero does not.

    :param registry: Rows to check (defaults to the real :data:`LIVENESS_ROWS`;
        injectable so the efficacy test can supply an injected inert producer).
    :returns: Sorted agent-legible finding strings, one per inert producer.
    """
    outputs_by_producer: dict[str, list[LivenessRow]] = {}
    for row in registry:
        outputs_by_producer.setdefault(row.producer_symbol, []).append(row)

    findings: list[str] = []
    for producer, rows in sorted(outputs_by_producer.items()):
        if any(row.consumer_files for row in rows):
            continue
        dead = ", ".join(sorted(row.output_symbol for row in rows))
        findings.append(
            finding(
                error_class="correct-but-inert",
                symbol=producer,
                file=rows[0].producer_file,
                line=0,
                problem=(
                    f"runs every tick but every declared output is dormant ({dead}) "
                    "— nothing downstream changes because it ran"
                ),
                remedy=(
                    "wire at least one output to a production consumer and record it "
                    "in consumer_files, or retire the producer; a producer that only "
                    "computes is decoration (Constitution III.10)"
                ),
            )
        )
    return sorted(findings)


#: Nothing gates: per the standing owner ruling the liveness sensor is advisory
#: and local/on-demand (no nightly CI plumbing).
_GATING_CHECKS: tuple[LabelledCheck, ...] = ()

#: Both liveness classes report advisorily.
_ADVISORY_CHECKS: tuple[LabelledCheck, ...] = (
    ("declared output has no production reader", check_outputs_have_readers),
    ("producer runs but every output is dormant", check_producers_are_not_inert),
)


def _summary(advisory_count: int) -> str:
    """Build the clean one-line summary printed when nothing gated.

    :param advisory_count: Number of advisory findings emitted above.
    :returns: The summary line naming the size of the declared registry.
    """
    live = sum(1 for row in LIVENESS_ROWS if row.consumer_files)
    dormant = len(LIVENESS_ROWS) - live
    summary = (
        f"Liveness (static, advisory): {len(LIVENESS_ROWS)} declared outputs — "
        f"{live} consumed, {dormant} declared dormant with a reason."
    )
    if advisory_count:
        summary += f" ({advisory_count} advisory findings above.)"
    return summary


def main(argv: list[str] | None = None) -> int:
    """Run both liveness sensors and return the process exit code.

    :param argv: CLI args (``--check`` accepted for family symmetry; this
        sensor is advisory and never gates, so it does not change behavior).
    :returns: 0 clean or advisory-only, 2 infrastructure failure.
    """
    parser = argparse.ArgumentParser(
        description="Liveness — correct-but-inert / computed-but-never-consumed (advisory).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Accepted for family symmetry; this sensor is advisory and never gates.",
    )
    parser.parse_args(argv)
    return run_sensor("LIVENESS", _GATING_CHECKS, _ADVISORY_CHECKS, _summary)


if __name__ == "__main__":
    sys.exit(main())
```

In `tools/sentinel_check.py`, add the import beside the existing sensor imports (after `from babylon.sentinels.coverage.checks import main as coverage_main`):
```python
from babylon.sentinels.liveness.checks import main as liveness_main
```
and add to `_SENSORS`:
```python
    "liveness": liveness_main,
```

Append to `.mise.toml` after the `check:coverage` block:
```toml
[tasks."check:liveness"]
description = "Liveness sentinel (ADVISORY, local): correct-but-inert + computed-but-never-consumed"
run = "poetry run python tools/sentinel_check.py liveness"
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_liveness_inert.py`
Expected: PASS (5 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): correct-but-inert sensor + liveness CLI task

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.6: intensive-aggregation sensor (AST) + mutation proof

**Files:**
- Create: `src/babylon/sentinels/aggregation/__init__.py`
- Create: `src/babylon/sentinels/aggregation/registry.py`
- Create: `src/babylon/sentinels/aggregation/checks.py`
- Modify: `tools/sentinel_check.py` (import + `_SENSORS` entry, beside the `liveness` entry from U7.5)
- Modify: `.mise.toml` (append after the `check:liveness` block from U7.5)
- Test: `tests/unit/sentinels/test_aggregation.py`

**Interfaces:**
- Consumes: `babylon.sentinels._ast.parse_module`, `babylon.sentinels.report.finding`
- Produces: `unweighted_mean_sites(path) -> tuple[tuple[str, int], ...]`, `check_no_unweighted_intensive_means(files=SCANNED_FILES) -> list[str]`, CLI sensor name `aggregation`.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the intensive-aggregation sensor.

An *intensive* quantity — a rate, ratio, share, balance, index — does not
average across space or class. The aggregate profit rate is ``Σs / Σ(c+v)``, not
``mean(rᵢ)``: the unweighted form lets a county of four hundred people swing a
national threshold as hard as Wayne County. This sensor finds the unweighted
form statically.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from babylon.sentinels.aggregation.checks import (
    check_no_unweighted_intensive_means,
    unweighted_mean_sites,
)
from babylon.sentinels.aggregation.registry import AggregationExemption

pytestmark = pytest.mark.unit


def test_detects_the_total_over_count_form(tmp_path: Path) -> None:
    """MUTATION: the classic ``total / count`` unweighted mean of a rate is found."""
    target = tmp_path / "scissors.py"
    target.write_text(
        "\n".join(
            [
                "def _mean_profit_rate(graph):",
                "    total = 0.0",
                "    count = 0",
                "    for node in sorted(graph.nodes()):",
                "        total += float(node.rate)",
                "        count += 1",
                "    return total / count if count else None",
            ]
        ),
        encoding="utf-8",
    )
    sites = unweighted_mean_sites(target)
    assert sites == (("_mean_profit_rate", 7),)


def test_detects_the_sum_over_len_form(tmp_path: Path) -> None:
    """MUTATION: ``sum(x) / len(x)`` over an intensive is the same defect."""
    target = tmp_path / "ratio.py"
    target.write_text(
        "\n".join(
            [
                "def mean_debt_ratio(values):",
                "    return sum(values) / len(values)",
            ]
        ),
        encoding="utf-8",
    )
    assert unweighted_mean_sites(target) == (("mean_debt_ratio", 2),)


def test_ignores_a_weighted_aggregate(tmp_path: Path) -> None:
    """A capital-weighted aggregate is the CORRECT form and must not be flagged."""
    target = tmp_path / "weighted.py"
    target.write_text(
        "\n".join(
            [
                "def mean_profit_rate(graph):",
                "    surplus = 0.0",
                "    capital = 0.0",
                "    for node in sorted(graph.nodes()):",
                "        surplus += node.surplus",
                "        capital += node.capital_stock",
                "    return surplus / capital if capital else None",
            ]
        ),
        encoding="utf-8",
    )
    assert unweighted_mean_sites(target) == ()


def test_ignores_an_extensive_mean(tmp_path: Path) -> None:
    """Averaging an extensive quantity (a count of people) is legitimate."""
    target = tmp_path / "extensive.py"
    target.write_text(
        "\n".join(
            [
                "def mean_population(values):",
                "    return sum(values) / len(values)",
            ]
        ),
        encoding="utf-8",
    )
    assert unweighted_mean_sites(target) == ()


def test_real_scanned_files_are_clean_or_exempt() -> None:
    """INVARIANT: the declared scan set carries no undeclared unweighted mean."""
    assert check_no_unweighted_intensive_means() == []


def test_check_reports_agent_legible_finding(tmp_path: Path) -> None:
    """MUTATION: an injected offending file produces a full agent-legible finding."""
    target = tmp_path / "offender.py"
    target.write_text(
        "\n".join(
            [
                "def mean_credit_fragility(values):",
                "    return sum(values) / len(values)",
            ]
        ),
        encoding="utf-8",
    )
    findings = check_no_unweighted_intensive_means(
        files=(str(target),), exemptions=(), repo_root=Path("/")
    )
    assert len(findings) == 1
    assert findings[0].startswith("[intensive-aggregation]")
    assert "mean_credit_fragility" in findings[0]
    assert "REMEDY:" in findings[0]


def test_exemption_silences_a_declared_site(tmp_path: Path) -> None:
    """A declared exemption with a reason silences its own site only."""
    target = tmp_path / "exempt.py"
    target.write_text(
        "\n".join(
            [
                "def mean_solidarity_index(values):",
                "    return sum(values) / len(values)",
            ]
        ),
        encoding="utf-8",
    )
    exemption = AggregationExemption(
        file=str(target),
        symbol="mean_solidarity_index",
        reason="the index is defined per-node with equal weight by construction",
    )
    assert (
        check_no_unweighted_intensive_means(
            files=(str(target),), exemptions=(exemption,), repo_root=Path("/")
        )
        == []
    )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_aggregation.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.aggregation'`
- [ ] **Step 3: Write minimal implementation**

`src/babylon/sentinels/aggregation/__init__.py`:
```python
"""The aggregation sentinel — intensive quantities do not average.

An **extensive** quantity (wealth, population, hours) sums across a region; an
**intensive** one (a rate, ratio, share, balance, index) does not. The aggregate
profit rate is ``Σs / Σ(c+v)``, never ``mean(rᵢ)`` — and the difference is not
cosmetic: the unweighted form gives a four-hundred-person county the same say in
a national threshold as Wayne County, so a national serviceability line moves for
reasons no material relation supports.

This sensor finds the unweighted form statically, by shape: a division whose
numerator accumulates and whose denominator merely counts, inside a function
whose name or accumulator names an intensive. Sites that are legitimately
equal-weighted declare an exemption WITH A REASON in
:mod:`babylon.sentinels.aggregation.registry`.

Advisory and local/on-demand per the standing owner ruling:
``poetry run python tools/sentinel_check.py aggregation-intensive``.

Layer 0.5: imports nothing above :mod:`babylon.models`.
"""
```

`src/babylon/sentinels/aggregation/registry.py`:
```python
"""Declared scan set and exemptions for the intensive-aggregation sensor.

:data:`SCANNED_FILES` is where intensive quantities are aggregated across space
or class in this codebase — the scissors, the economic tick, the dialectics
catalog. :data:`AGGREGATION_EXEMPTIONS` records the sites where an unweighted
mean is nonetheless correct, each with the reason that makes it correct. An
exemption without a reason is refused at import.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, model_validator

#: Name fragments that mark a quantity as INTENSIVE — it does not average.
INTENSIVE_LEXICON: tuple[str, ...] = (
    "rate",
    "ratio",
    "share",
    "balance",
    "index",
    "intensity",
    "density",
    "fragility",
    "per_capita",
    "coefficient",
)

#: Name fragments that mark a *function* as computing a mean.
MEAN_LEXICON: tuple[str, ...] = ("mean", "avg", "average")

#: The files scanned by default — where intensives meet space/class aggregation.
SCANNED_FILES: tuple[str, ...] = (
    "src/babylon/engine/systems/market_scissors.py",
    "src/babylon/engine/systems/contradiction.py",
    "src/babylon/domain/dialectics/instances/catalog.py",
)


class AggregationExemption(BaseModel):
    """One site where an unweighted mean of an intensive is nonetheless correct.

    Frozen and ``extra="forbid"`` so a malformed row fails loudly at import
    (Constitution III.11).

    :ivar file: repo-relative path (or absolute path, in tests) of the site.
    :ivar symbol: the enclosing function name the sensor reports.
    :ivar reason: why equal weighting is materially right here — never blank.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    file: str
    symbol: str
    reason: str

    @model_validator(mode="after")
    def _validate_reason(self) -> AggregationExemption:
        """Refuse an exemption that does not say why it is legitimate.

        :returns: ``self`` when valid.
        :raises ValueError: If ``file``, ``symbol``, or ``reason`` is blank.
        """
        for label, value in (
            ("file", self.file),
            ("symbol", self.symbol),
            ("reason", self.reason),
        ):
            if not value.strip():
                raise ValueError(f"AggregationExemption.{label} must be non-empty")
        return self


#: Sanctioned unweighted means. Empty by design — an entry here is a claim that
#: equal weighting is materially correct, and must be argued, not assumed.
AGGREGATION_EXEMPTIONS: tuple[AggregationExemption, ...] = ()
```

`src/babylon/sentinels/aggregation/checks.py`:
```python
"""The intensive-aggregation sensor (static, advisory).

Walks each scanned file with :mod:`ast` and reports every division whose shape
is an unweighted mean of an intensive quantity: ``total / count``,
``sum(xs) / len(xs)``, or ``statistics.mean(xs)`` where the enclosing function or
the accumulated name is intensive by :data:`INTENSIVE_LEXICON`.

Reports the enclosing function and the line of the offending division, so the
finding points at the arithmetic, not the module.
"""

from __future__ import annotations

import argparse
import ast
import sys
from pathlib import Path

from babylon.sentinels._ast import parse_module
from babylon.sentinels.aggregation.registry import (
    AGGREGATION_EXEMPTIONS,
    INTENSIVE_LEXICON,
    MEAN_LEXICON,
    SCANNED_FILES,
    AggregationExemption,
)
from babylon.sentinels.base import LabelledCheck, run_sensor
from babylon.sentinels.report import finding

#: Repo root (this file is ``<root>/src/babylon/sentinels/aggregation/checks.py``).
_REPO_ROOT: Path = Path(__file__).resolve().parents[4]

#: Denominator names that merely COUNT members rather than weight them.
_COUNTING_NAMES: frozenset[str] = frozenset({"count", "n", "num", "n_nodes", "size"})


def _is_intensive(name: str) -> bool:
    """Whether a symbol name marks an intensive quantity.

    :param name: A function, variable, or attribute name.
    :returns: Whether any :data:`INTENSIVE_LEXICON` fragment occurs in it.
    """
    lowered = name.lower()
    return any(fragment in lowered for fragment in INTENSIVE_LEXICON)


def _is_counting_denominator(node: ast.expr) -> bool:
    """Whether a division's denominator counts members instead of weighting them.

    :param node: The right-hand expression of a division.
    :returns: Whether it is a bare counting name or a ``len(...)`` call.
    """
    if isinstance(node, ast.Name):
        return node.id.lower() in _COUNTING_NAMES
    return (
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "len"
    )


def _numerator_names(node: ast.expr) -> set[str]:
    """Collect the names appearing in a division's numerator.

    :param node: The left-hand expression of a division.
    :returns: Every ``Name`` id occurring in it.
    """
    return {child.id for child in ast.walk(node) if isinstance(child, ast.Name)}


def unweighted_mean_sites(path: Path) -> tuple[tuple[str, int], ...]:
    """Find unweighted means of intensive quantities in one source file.

    A site qualifies when a division's denominator merely counts members
    (:func:`_is_counting_denominator`) AND the enclosing function name or one of
    the numerator's names is intensive (:func:`_is_intensive`). ``mean`` in the
    function name alone is not enough — averaging an extensive quantity is
    legitimate — and a division by a weighting total (``capital``, ``surplus``)
    is the correct form and is never reported.

    :param path: Source file to scan.
    :returns: ``(enclosing_function, line)`` pairs, in source order.
    :raises SentinelCheckError: If the file is missing or unparseable.
    """
    tree = parse_module(path)
    sites: list[tuple[str, int]] = []
    for func in ast.walk(tree):
        if not isinstance(func, (ast.FunctionDef, ast.AsyncFunctionDef)):
            continue
        for node in ast.walk(func):
            if not (isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div)):
                continue
            if not _is_counting_denominator(node.right):
                continue
            names = _numerator_names(node.left)
            if _is_intensive(func.name) or any(_is_intensive(name) for name in names):
                sites.append((func.name, node.lineno))
            elif any(fragment in func.name.lower() for fragment in MEAN_LEXICON) and any(
                _is_intensive(name) for name in names
            ):
                sites.append((func.name, node.lineno))
    return tuple(sorted(set(sites), key=lambda site: (site[1], site[0])))


def check_no_unweighted_intensive_means(
    files: tuple[str, ...] = SCANNED_FILES,
    exemptions: tuple[AggregationExemption, ...] = AGGREGATION_EXEMPTIONS,
    repo_root: Path = _REPO_ROOT,
) -> list[str]:
    """No scanned file may average an intensive across space or class undeclared.

    :param files: Paths to scan (repo-relative by default; injectable so the
        efficacy tests can supply an injected offender).
    :param exemptions: Declared-legitimate sites (each carrying its reason).
    :param repo_root: Root the ``files`` entries resolve against.
    :returns: Sorted agent-legible finding strings (empty when clean).
    :raises SentinelCheckError: If any scanned file is missing or unparseable.
    """
    exempt = {(row.file, row.symbol) for row in exemptions}
    findings: list[str] = []
    for relative in files:
        for symbol, line in unweighted_mean_sites(repo_root / relative):
            if (relative, symbol) in exempt:
                continue
            findings.append(
                finding(
                    error_class="intensive-aggregation",
                    symbol=symbol,
                    file=relative,
                    line=line,
                    problem=(
                        "takes an unweighted mean of an intensive quantity across "
                        "space or class — a tiny member swings the aggregate as hard "
                        "as a large one"
                    ),
                    remedy=(
                        "aggregate the numerator and the denominator separately "
                        "(the profit rate is sum(surplus) / sum(capital), not "
                        "mean(rate)); if equal weighting is materially correct here, "
                        "declare an AggregationExemption with the reason"
                    ),
                )
            )
    return sorted(findings)


#: Nothing gates — advisory and local/on-demand per the standing owner ruling.
_GATING_CHECKS: tuple[LabelledCheck, ...] = ()

#: The single advisory check.
_ADVISORY_CHECKS: tuple[LabelledCheck, ...] = (
    ("unweighted mean of an intensive across space/class", check_no_unweighted_intensive_means),
)


def _summary(advisory_count: int) -> str:
    """Build the clean one-line summary printed when nothing gated.

    :param advisory_count: Number of advisory findings emitted above.
    :returns: The summary line naming the scan size.
    """
    summary = (
        f"Aggregation (static, advisory): {len(SCANNED_FILES)} files scanned, "
        f"{len(AGGREGATION_EXEMPTIONS)} declared exemptions."
    )
    if advisory_count:
        summary += f" ({advisory_count} advisory findings above.)"
    return summary


def main(argv: list[str] | None = None) -> int:
    """Run the intensive-aggregation sensor and return the process exit code.

    :param argv: CLI args (``--check`` accepted for family symmetry; advisory).
    :returns: 0 clean or advisory-only, 2 infrastructure failure.
    """
    parser = argparse.ArgumentParser(
        description="Aggregation — no unweighted mean of an intensive (advisory).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Accepted for family symmetry; this sensor is advisory and never gates.",
    )
    parser.parse_args(argv)
    return run_sensor("AGGREGATION", _GATING_CHECKS, _ADVISORY_CHECKS, _summary)


if __name__ == "__main__":
    sys.exit(main())
```

Register in `tools/sentinel_check.py` (import beside `liveness_main`, entry beside `"liveness"`):
```python
from babylon.sentinels.aggregation.checks import main as aggregation_main
```
```python
    "aggregation": aggregation_main,
```

Append to `.mise.toml`:
```toml
[tasks."check:aggregation-intensive"]
description = "Aggregation sentinel (ADVISORY, local): no unweighted mean of a rate/ratio/balance"
run = "poetry run python tools/sentinel_check.py aggregation"
```

> U6.1 already replaced `_mean_profit_rate` with a `tick_capital_stock`-weighted aggregate, so this file must be clean. If it reds, U6.1 did not land — go fix U6.1, do NOT add an AggregationExemption (that would be a false liveness claim).

- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_aggregation.py`
Expected: PASS (7 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): intensive-aggregation sensor with mutation proof

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.7: Coupling dependency registry

**Files:**
- Create: `src/babylon/sentinels/coupling/__init__.py`
- Create: `src/babylon/sentinels/coupling/registry.py`
- Test: `tests/unit/sentinels/test_coupling_registry.py`

**Interfaces:**
- Consumes: the U5 opposition keys `surplus_distribution`, `debt_spiral`, `credit`, `financial` and the U5 `GraphInputs` fields `rentier_share`, `debt_ratio`, `credit_fragility`, `financialization_index`
- Produces: `MeasurementDependency`, `MEASUREMENT_DEPENDENCIES: tuple[MeasurementDependency, ...]`, `dependency_for(key)` — consumed by U7.8.

**DECLARED LIMITATION — direction B is inert on this registry as seeded.** All five rows below
name the same `producer_file` (`_CONTRADICTION`), and U7.8's
`check_real_dependencies_are_declared` skips every pair whose two rows share a producer file (a
same-file pair mentions the sibling's symbols trivially, so a mention proves nothing). Every
ordered pair of these five rows is therefore skipped, and
`test_real_dependencies_are_all_declared` passes **vacuously** — it asserts `[]` against a loop
body that never executes. This is the correct-but-inert error class appearing inside the very
program that ships the correct-but-inert sensor, so it is declared here rather than discovered
later: declared-and-deferred is acceptable, silently-inert is not.

Direction B is retained, not deleted, because it is a **guard against future rows**: the moment
one opposition's inputs are produced somewhere other than `contradiction.py` — U6's monetary
anchor (`domain/economics/monetary/anchor.py`), a future `market_scissors.py`-produced field, any
new System that computes a `GraphInputs` field — that row pairs against the five existing ones
with a different `producer_file`, the skip no longer fires, and the check goes live with no code
change. Its efficacy is proven today by U7.8's injected-fixture mutation tests, which supply rows
with two different producer files; that is what keeps a currently-inert check from rotting into a
broken one. What would make it live on the real registry: a second distinct `producer_file` among
the declared rows.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the declared measurement-dependency registry.

The coupling graph is a claim ABOUT THE CODE: "``surplus_distribution``
transforms ``debt_spiral``" asserts that the thing computing the debt reading
reads the thing computing the distribution reading. This registry is what makes
that claim checkable — for each opposition, which ``GraphInputs`` fields its
measure reads, which file produces them, and which symbols that file publishes.
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.sentinels.coupling.registry import (
    MEASUREMENT_DEPENDENCIES,
    MeasurementDependency,
    dependency_for,
)

pytestmark = pytest.mark.unit


def test_registry_covers_the_four_new_oppositions_and_price_value() -> None:
    """All U5 keys plus the scissors axis they couple to are declared."""
    keys = {row.opposition_key for row in MEASUREMENT_DEPENDENCIES}
    assert {
        "surplus_distribution",
        "debt_spiral",
        "credit",
        "financial",
        "price_value",
    } <= keys


def test_dependency_for_returns_the_row() -> None:
    """Lookup by opposition key returns the declared row."""
    row = dependency_for("financial")
    assert row is not None
    assert "financialization_index" in row.inputs_fields


def test_dependency_for_returns_none_for_unregistered_key() -> None:
    """An unregistered key yields None rather than raising — checks skip it."""
    assert dependency_for("no_such_opposition") is None


def test_price_value_publishes_the_scissors_symbols() -> None:
    """``price_value``'s produces set is what a downstream reader must mention."""
    row = dependency_for("price_value")
    assert row is not None
    assert "market_balance" in row.produces_symbols
    assert "price_log" in row.produces_symbols


def test_row_rejects_empty_inputs_fields() -> None:
    """An opposition with no measured input is not a measurement dependency."""
    with pytest.raises(ValidationError, match="inputs_fields"):
        MeasurementDependency(
            opposition_key="broken",
            inputs_fields=(),
            producer_file="src/babylon/engine/systems/contradiction.py",
            produces_symbols=("x",),
        )


def test_row_rejects_a_non_python_producer_file() -> None:
    """The producer file must be ``.py`` source the AST sensor can read."""
    with pytest.raises(ValidationError, match=r"\.py"):
        MeasurementDependency(
            opposition_key="broken",
            inputs_fields=("x",),
            producer_file="src/babylon/engine/systems/contradiction.yaml",
            produces_symbols=("x",),
        )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_coupling_registry.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.coupling'`
- [ ] **Step 3: Write minimal implementation**

`src/babylon/sentinels/coupling/__init__.py`:
```python
"""The coupling sentinel — declared edges must match real dependencies, both ways.

``_DEFAULT_COUPLINGS`` in the dialectics catalog is not vocabulary; it is a claim
about the code. ``Coupling(source="surplus_distribution", target="debt_spiral",
kind="transforms")`` asserts that whatever computes the debt reading reads
whatever computes the distribution reading. A hand-authored graph drifts from the
dependencies it describes the moment either side changes — which is exactly how
four reserved edges sat dormant and undetected for months, and how
``momentum_coupling`` stayed a real dependency nobody had declared.

So this sensor checks BOTH directions:

- **declared-but-absent** — an edge whose target's producer does not in fact read
  any symbol the source's producer publishes;
- **present-but-undeclared** — a producer that DOES read another opposition's
  published symbol, with no edge declaring it.

Advisory and local/on-demand:
``poetry run python tools/sentinel_check.py coupling``.

Layer 0.5: reads the catalog statically via :mod:`ast` — it may not import
``babylon.domain`` (import-linter contract, ``pyproject.toml``).
"""
```

`src/babylon/sentinels/coupling/registry.py`:
```python
"""Declared measurement dependencies, one row per opposition.

Each row answers three questions about one opposition key: which
``GraphInputs`` fields its measure READS, which file PRODUCES those fields, and
which symbols that file PUBLISHES for others to read. From those three facts the
sensor derives the real dependency graph and diffs it against the declared
``_DEFAULT_COUPLINGS`` map in both directions.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, model_validator


class MeasurementDependency(BaseModel):
    """One opposition's measured inputs and the file that produces them.

    Frozen and ``extra="forbid"`` so a malformed row fails loudly at import
    (Constitution III.11).

    :ivar opposition_key: the registry key (e.g. ``"debt_spiral"``).
    :ivar inputs_fields: the ``GraphInputs`` fields this opposition's measure
        reads — the material the gap reading is made of.
    :ivar producer_file: repo-relative ``.py`` path computing those fields.
    :ivar produces_symbols: the names ``producer_file`` publishes for others;
        a downstream producer mentioning one of these IS a real dependency.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    opposition_key: str
    inputs_fields: tuple[str, ...]
    producer_file: str
    produces_symbols: tuple[str, ...]

    @model_validator(mode="after")
    def _validate_shape(self) -> MeasurementDependency:
        """Reject blank keys, empty field sets, and non-``.py`` producer paths.

        :returns: ``self`` when valid.
        :raises ValueError: If ``opposition_key`` is blank, ``inputs_fields`` or
            ``produces_symbols`` is empty, or ``producer_file`` is not ``.py``.
        """
        if not self.opposition_key.strip():
            raise ValueError("MeasurementDependency.opposition_key must be non-empty")
        if not self.inputs_fields:
            raise ValueError(
                f"{self.opposition_key!r}: inputs_fields must name at least one "
                "GraphInputs field — an opposition measuring nothing is not a "
                "measurement dependency"
            )
        if not self.produces_symbols:
            raise ValueError(
                f"{self.opposition_key!r}: produces_symbols must be non-empty"
            )
        if not self.producer_file.endswith(".py"):
            raise ValueError(
                f"{self.opposition_key!r}: producer_file must be a .py path, "
                f"got {self.producer_file!r}"
            )
        return self


#: Repo-relative producer roots, factored out for readability.
_CONTRADICTION = "src/babylon/engine/systems/contradiction.py"
_SCISSORS = "src/babylon/engine/systems/market_scissors.py"

#: The declared measurement dependencies of the money/value oppositions.
#:
#: DECLARED LIMITATION: every row below names ``_CONTRADICTION`` as its producer
#: file, and ``check_real_dependencies_are_declared`` (direction B) skips any
#: pair of rows sharing a producer file — a same-file pair mentions its
#: sibling's symbols trivially, so a mention there proves nothing. Direction B
#: is therefore INERT on this registry as seeded, and its invariant test passes
#: vacuously. It is kept as a guard for future rows: the first opposition whose
#: inputs are produced outside ``contradiction.py`` (the U6 monetary anchor, a
#: ``market_scissors.py``-produced field, any new System computing a
#: ``GraphInputs`` field) makes the check live with no code change. Its efficacy
#: is proven meanwhile by the injected-fixture mutation tests in
#: ``tests/unit/sentinels/test_coupling_sentinel.py``, which supply rows with two
#: distinct producer files. Declared here rather than discovered later: this is
#: the correct-but-inert class inside the program that ships its sensor.
MEASUREMENT_DEPENDENCIES: tuple[MeasurementDependency, ...] = (
    MeasurementDependency(
        opposition_key="price_value",
        inputs_fields=("market_balance",),
        producer_file=_CONTRADICTION,
        produces_symbols=("market_balance", "price_log", "price_velocity"),
    ),
    # U5.7 derives financialization_index in ContradictionSystem from the
    # scissors' fictitious_log; market_scissors.py is the upstream axis
    # owner, contradiction.py is the field producer.
    MeasurementDependency(
        opposition_key="financial",
        inputs_fields=("financialization_index",),
        producer_file=_CONTRADICTION,
        produces_symbols=("financialization_index", "fictitious_log"),
    ),
    MeasurementDependency(
        opposition_key="surplus_distribution",
        inputs_fields=("rentier_share",),
        producer_file=_CONTRADICTION,
        produces_symbols=("surplus_distribution", "rentier_share"),
    ),
    MeasurementDependency(
        opposition_key="debt_spiral",
        inputs_fields=("debt_ratio",),
        producer_file=_CONTRADICTION,
        produces_symbols=("debt_accumulation", "debt_ratio"),
    ),
    MeasurementDependency(
        opposition_key="credit",
        inputs_fields=("credit_fragility",),
        producer_file=_CONTRADICTION,
        produces_symbols=("credit_fragility", "credit_state"),
    ),
)


def dependency_for(opposition_key: str) -> MeasurementDependency | None:
    """Look up one opposition's declared measurement dependency.

    :param opposition_key: The registry key to look up.
    :returns: The declared row, or ``None`` when the key is not registered (the
        sensors skip unregistered endpoints rather than inventing a claim).
    """
    for row in MEASUREMENT_DEPENDENCIES:
        if row.opposition_key == opposition_key:
            return row
    return None
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_coupling_registry.py`
Expected: PASS (6 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): declared measurement-dependency registry for couplings

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.8: undeclared-coupling sensor — both directions + mutation proofs

**Files:**
- Create: `src/babylon/sentinels/coupling/checks.py`
- Modify: `tools/sentinel_check.py` (import + `_SENSORS` entry, beside the `aggregation` entry from U7.6)
- Modify: `.mise.toml` (append after the `check:aggregation-intensive` block from U7.6)
- Test: `tests/unit/sentinels/test_coupling_sentinel.py`

**Interfaces:**
- Consumes: `babylon.sentinels._ast.coupling_edges`, `babylon.sentinels._ast.referenced_names`, `babylon.sentinels.coupling.registry.MEASUREMENT_DEPENDENCIES`, `babylon.sentinels.report.finding`
- Produces: `check_declared_edges_are_grounded(...)`, `check_real_dependencies_are_declared(...)`, `babylon.sentinels.coupling.checks.main`, CLI sensor name `coupling`.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the undeclared-coupling sensor (both directions).

- **Invariant** — on the real catalog + registry, every declared edge between
  two registered oppositions is grounded in a real read, and every real read
  between two registered oppositions is declared.
- **Efficacy (MUTATION)** — direction A reds on an edge whose target's producer
  reads nothing the source publishes; direction B reds on a real read with no
  declaring edge.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from babylon.sentinels.coupling.checks import (
    check_declared_edges_are_grounded,
    check_real_dependencies_are_declared,
    main,
)
from babylon.sentinels.coupling.registry import MeasurementDependency

pytestmark = pytest.mark.unit

_TOOL_PATH = Path(__file__).resolve().parents[3] / "tools" / "sentinel_check.py"


def test_real_declared_edges_are_grounded() -> None:
    """INVARIANT: every declared edge maps to a real measurement dependency."""
    assert check_declared_edges_are_grounded() == []


def test_real_dependencies_are_all_declared() -> None:
    """INVARIANT: every real cross-opposition read carries a declaring edge."""
    assert check_real_dependencies_are_declared() == []


def test_efficacy_reds_on_a_declared_edge_with_no_real_dependency() -> None:
    """MUTATION: an edge whose target reads nothing the source publishes reds.

    ``web/game/map_contract.py`` is a real, parseable file that mentions neither
    ``phantom_published_symbol`` nor the source's fields — so the declared edge
    is a claim about the code that the code does not support.
    """
    source = MeasurementDependency(
        opposition_key="phantom_source",
        inputs_fields=("phantom_input",),
        producer_file="src/babylon/engine/systems/contradiction.py",
        produces_symbols=("phantom_published_symbol",),
    )
    target = MeasurementDependency(
        opposition_key="phantom_target",
        inputs_fields=("other_input",),
        producer_file="web/game/map_contract.py",
        produces_symbols=("other_published_symbol",),
    )
    findings = check_declared_edges_are_grounded(
        edges=(("phantom_source", "phantom_target", "transforms"),),
        dependencies=(source, target),
    )
    assert len(findings) == 1
    assert findings[0].startswith("[undeclared-coupling]")
    assert "phantom_source -> phantom_target" in findings[0]
    assert "REMEDY:" in findings[0]


def test_efficacy_reds_on_a_real_dependency_that_is_undeclared() -> None:
    """MUTATION: a real cross-opposition read with no declaring edge reds.

    ``market_scissors.py`` really does mention ``price_log``, so declaring it as
    the source's published symbol while declaring NO edge is exactly the
    ``momentum_coupling`` failure: a real dependency nobody wrote down.

    The fixture is deliberately ONE-DIRECTIONAL. ``check_real_dependencies_are_declared``
    loops every ORDERED pair, so the reverse pair (``phantom_financial ->
    phantom_price``) is judged too, and it asks whether ``contradiction.py``
    mentions ``phantom_financial``'s published symbols. ``fictitious_log`` would
    fail that test: ``referenced_names`` collects string constants, and U5.7
    writes the literal ``"fictitious_log"`` into ``contradiction.py`` to derive
    ``financialization_index`` — so the reverse pair would fire a second finding
    and this assertion would see 2. ``PRICE_DIVERGENCE_ATTR`` is a real
    module-level constant in ``market_scissors.py`` that appears nowhere in
    ``contradiction.py`` and is added to it by no task in this plan, so exactly
    one direction is a real read and exactly one finding is produced.
    """
    source = MeasurementDependency(
        opposition_key="phantom_price",
        inputs_fields=("market_balance",),
        producer_file="src/babylon/engine/systems/contradiction.py",
        produces_symbols=("price_log",),
    )
    target = MeasurementDependency(
        opposition_key="phantom_financial",
        inputs_fields=("financialization_index",),
        producer_file="src/babylon/engine/systems/market_scissors.py",
        produces_symbols=("PRICE_DIVERGENCE_ATTR",),
    )
    findings = check_real_dependencies_are_declared(
        edges=(),
        dependencies=(source, target),
    )
    assert len(findings) == 1
    assert findings[0].startswith("[undeclared-coupling]")
    assert "phantom_price" in findings[0]
    assert "phantom_financial" in findings[0]
    assert "price_log" in findings[0]


def test_declared_edge_silences_the_real_dependency_finding() -> None:
    """Declaring the edge makes direction B clean — the two directions agree.

    Same one-directional fixture as the test above, for the same reason: the
    reverse pair must contribute NO finding of its own, or ``== []`` would see
    the reverse finding and fail for a reason that has nothing to do with the
    declared edge.
    """
    source = MeasurementDependency(
        opposition_key="phantom_price",
        inputs_fields=("market_balance",),
        producer_file="src/babylon/engine/systems/contradiction.py",
        produces_symbols=("price_log",),
    )
    target = MeasurementDependency(
        opposition_key="phantom_financial",
        inputs_fields=("financialization_index",),
        producer_file="src/babylon/engine/systems/market_scissors.py",
        produces_symbols=("PRICE_DIVERGENCE_ATTR",),
    )
    assert (
        check_real_dependencies_are_declared(
            edges=(("phantom_price", "phantom_financial", "feeds"),),
            dependencies=(source, target),
        )
        == []
    )


def test_unregistered_endpoints_are_skipped_not_invented() -> None:
    """An edge to an unregistered opposition yields no claim either way."""
    source = MeasurementDependency(
        opposition_key="phantom_source",
        inputs_fields=("x",),
        producer_file="src/babylon/engine/systems/contradiction.py",
        produces_symbols=("x",),
    )
    assert (
        check_declared_edges_are_grounded(
            edges=(("phantom_source", "not_registered", "transforms"),),
            dependencies=(source,),
        )
        == []
    )


def test_main_exits_zero_because_coupling_is_advisory() -> None:
    """The sensor is advisory: findings print, the process never gates."""
    assert main([]) == 0


def test_cli_dispatches_the_coupling_sensor() -> None:
    """``sentinel_check.py coupling`` routes to this sensor and exits cleanly."""
    result = subprocess.run(
        [sys.executable, str(_TOOL_PATH), "coupling"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0
    assert "Coupling" in result.stdout
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_coupling_sentinel.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'babylon.sentinels.coupling.checks'`
- [ ] **Step 3: Write minimal implementation**
```python
"""The undeclared-coupling sensor (static, advisory, BOTH directions).

Reads the declared ``Coupling(...)`` literals out of the dialectics catalog with
:mod:`ast` and the declared measurement dependencies out of
:mod:`babylon.sentinels.coupling.registry`, then diffs the two:

- :func:`check_declared_edges_are_grounded` — declared-but-absent;
- :func:`check_real_dependencies_are_declared` — present-but-undeclared.

Only edges whose BOTH endpoints are registered are judged; an unregistered
endpoint yields no claim in either direction (the sensor never invents a
dependency it cannot see). ``contains`` edges are excluded: they are auto-derived
from pole nesting, not authored, so they make no claim about reads.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from babylon.sentinels._ast import coupling_edges, referenced_names
from babylon.sentinels.base import LabelledCheck, run_sensor
from babylon.sentinels.coupling.registry import (
    MEASUREMENT_DEPENDENCIES,
    MeasurementDependency,
)
from babylon.sentinels.report import finding

#: Repo root (this file is ``<root>/src/babylon/sentinels/coupling/checks.py``).
_REPO_ROOT: Path = Path(__file__).resolve().parents[4]

#: The module declaring ``_DEFAULT_COUPLINGS``.
_CATALOG_FILE: str = "src/babylon/domain/dialectics/instances/catalog.py"

#: Kinds that assert a real read. ``contains`` is auto-derived from nesting and
#: ``antagonizes`` asserts mutual antagonism, not a measurement dependency.
_READ_KINDS: frozenset[str] = frozenset({"feeds", "constrains", "transforms"})


def _declared_edges() -> tuple[tuple[str, str, str], ...]:
    """Read the production catalog's declared coupling edges.

    :returns: ``(source, target, kind)`` triples from ``_DEFAULT_COUPLINGS``.
    :raises SentinelCheckError: If the catalog is missing or unparseable.
    """
    return coupling_edges(_REPO_ROOT / _CATALOG_FILE)


def _index(
    dependencies: tuple[MeasurementDependency, ...],
) -> dict[str, MeasurementDependency]:
    """Index declared dependencies by opposition key.

    :param dependencies: The declared rows.
    :returns: A mapping of opposition key to its row.
    """
    return {row.opposition_key: row for row in dependencies}


def _reads(target: MeasurementDependency, source: MeasurementDependency) -> bool:
    """Whether ``target``'s producer actually reads something ``source`` publishes.

    :param target: The downstream opposition's declared dependency.
    :param source: The upstream opposition's declared dependency.
    :returns: Whether the target's producer file mentions any of the source's
        ``produces_symbols``.
    :raises SentinelCheckError: If the target's producer file is missing or
        unparseable.
    """
    mentioned = referenced_names(_REPO_ROOT / target.producer_file)
    return any(symbol in mentioned for symbol in source.produces_symbols)


def check_declared_edges_are_grounded(
    edges: tuple[tuple[str, str, str], ...] | None = None,
    dependencies: tuple[MeasurementDependency, ...] = MEASUREMENT_DEPENDENCIES,
) -> list[str]:
    """Direction A — every declared edge must map to a real read (declared-but-absent).

    For each declared ``feeds``/``constrains``/``transforms`` edge whose both
    endpoints are registered, assert the target's producer file mentions at least
    one symbol the source's producer publishes. An edge that fails is a claim
    about the code the code does not support — the state the four reserved edges
    sat in for months.

    :param edges: Edge triples to judge (defaults to the real catalog's;
        injectable so the efficacy tests can supply an injected defect).
    :param dependencies: Declared measurement dependencies (injectable).
    :returns: Sorted agent-legible finding strings (empty when every edge is
        grounded).
    :raises SentinelCheckError: If the catalog or a producer file is missing or
        unparseable.
    """
    triples = _declared_edges() if edges is None else edges
    index = _index(dependencies)
    findings: list[str] = []
    for source_key, target_key, kind in triples:
        if kind not in _READ_KINDS:
            continue
        source = index.get(source_key)
        target = index.get(target_key)
        if source is None or target is None:
            continue
        if _reads(target, source):
            continue
        findings.append(
            finding(
                error_class="undeclared-coupling",
                symbol=f"{source_key} -> {target_key} ({kind})",
                file=_CATALOG_FILE,
                line=0,
                problem=(
                    f"declared edge is not grounded: {target.producer_file} mentions "
                    f"none of {source_key}'s published symbols "
                    f"({', '.join(source.produces_symbols)})"
                ),
                remedy=(
                    "either wire the dependency the edge claims (make the target's "
                    "producer read the source's output) or delete the edge from "
                    "_DEFAULT_COUPLINGS — a coupling graph that outruns the code is "
                    "vocabulary, not structure"
                ),
            )
        )
    return sorted(findings)


def check_real_dependencies_are_declared(
    edges: tuple[tuple[str, str, str], ...] | None = None,
    dependencies: tuple[MeasurementDependency, ...] = MEASUREMENT_DEPENDENCIES,
) -> list[str]:
    """Direction B — every real read must carry a declaring edge (present-but-undeclared).

    For each ordered pair of registered oppositions with DIFFERENT producer files
    (a same-file pair is judged by direction A alone, where mentions are trivially
    present), if the target's producer reads a symbol the source publishes, an
    edge ``source -> target`` of a reading kind must be declared. A real
    dependency nobody wrote down is the ``momentum_coupling`` failure.

    :param edges: Declared edge triples (defaults to the real catalog's).
    :param dependencies: Declared measurement dependencies (injectable).
    :returns: Sorted agent-legible finding strings (empty when every real
        dependency is declared).
    :raises SentinelCheckError: If the catalog or a producer file is missing or
        unparseable.
    """
    triples = _declared_edges() if edges is None else edges
    declared = {
        (source, target) for source, target, kind in triples if kind in _READ_KINDS
    }
    findings: list[str] = []
    for source in sorted(dependencies, key=lambda row: row.opposition_key):
        for target in sorted(dependencies, key=lambda row: row.opposition_key):
            if source.opposition_key == target.opposition_key:
                continue
            if source.producer_file == target.producer_file:
                continue
            if not _reads(target, source):
                continue
            if (source.opposition_key, target.opposition_key) in declared:
                continue
            shared = sorted(
                symbol
                for symbol in source.produces_symbols
                if symbol in referenced_names(_REPO_ROOT / target.producer_file)
            )
            findings.append(
                finding(
                    error_class="undeclared-coupling",
                    symbol=f"{source.opposition_key} -> {target.opposition_key}",
                    file=target.producer_file,
                    line=0,
                    problem=(
                        f"reads {source.opposition_key}'s published symbol(s) "
                        f"{', '.join(shared)} but no coupling edge declares it"
                    ),
                    remedy=(
                        "add the edge to _DEFAULT_COUPLINGS with the kind that "
                        "matches the real relation (feeds = the target reads the "
                        "source's observation; constrains = the source limits the "
                        "target's reachable state; transforms = the source's output "
                        "becomes the target's input prices), citing the read site"
                    ),
                )
            )
    return sorted(findings)


#: Nothing gates — advisory and local/on-demand per the standing owner ruling.
_GATING_CHECKS: tuple[LabelledCheck, ...] = ()

#: Both directions report advisorily.
_ADVISORY_CHECKS: tuple[LabelledCheck, ...] = (
    ("declared coupling edge has no real measurement dependency", check_declared_edges_are_grounded),
    ("real measurement dependency has no declared coupling edge", check_real_dependencies_are_declared),
)


def _summary(advisory_count: int) -> str:
    """Build the clean one-line summary printed when nothing gated.

    :param advisory_count: Number of advisory findings emitted above.
    :returns: The summary line naming the sizes of both sides of the diff.
    """
    summary = (
        f"Coupling (static, advisory): {len(MEASUREMENT_DEPENDENCIES)} declared "
        f"measurement dependencies diffed against {len(_declared_edges())} declared "
        "coupling edges, both directions."
    )
    if advisory_count:
        summary += f" ({advisory_count} advisory findings above.)"
    return summary


def main(argv: list[str] | None = None) -> int:
    """Run both coupling directions and return the process exit code.

    :param argv: CLI args (``--check`` accepted for family symmetry; advisory).
    :returns: 0 clean or advisory-only, 2 infrastructure failure.
    """
    parser = argparse.ArgumentParser(
        description="Coupling — declared edges vs real dependencies, both ways (advisory).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Accepted for family symmetry; this sensor is advisory and never gates.",
    )
    parser.parse_args(argv)
    return run_sensor("COUPLING", _GATING_CHECKS, _ADVISORY_CHECKS, _summary)


if __name__ == "__main__":
    sys.exit(main())
```

Register in `tools/sentinel_check.py`:
```python
from babylon.sentinels.coupling.checks import main as coupling_main
```
```python
    "coupling": coupling_main,
```

Append to `.mise.toml`:
```toml
[tasks."check:coupling"]
description = "Coupling sentinel (ADVISORY, local): declared coupling edges vs real dependencies, both directions"
run = "poetry run python tools/sentinel_check.py coupling"
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_coupling_sentinel.py`
Expected: PASS (8 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): undeclared-coupling sensor, both directions, with mutation proofs

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.9: gate-blindness — extend the existing coverage sentinel

**Files:**
- Modify: `src/babylon/sentinels/coverage/registry.py` (append the `GateEstate` model and `GATE_ESTATES` literal after `DATA_REQUIREMENTS`)
- Modify: `src/babylon/sentinels/coverage/checks.py:210-216` (the `_ADVISORY_CHECKS` tuple, currently `()`) and append `check_gate_estate_coverage`
- Test: `tests/unit/sentinels/test_gate_blindness.py`

**Interfaces:**
- Consumes: `babylon.sentinels._ast.returned_dict_keys`, `babylon.sentinels._ast.referenced_names`, `babylon.sentinels.report.finding`
- Produces: `GateEstate`, `GATE_ESTATES`, `check_gate_estate_coverage(estates=GATE_ESTATES) -> list[str]`.

- [ ] **Step 1: Write the failing test**
```python
"""Tests for the gate-blindness sensor.

``qa:regression`` is the project's Definition of Done. It nominally guards the
engine; it injected NO economics calculators at all, so its byte-identical
baselines never executed a line of the economics estate. A gate can be green and
blind. This sensor compares what a gate's harness actually injects against the
estate it claims to guard.
"""

from __future__ import annotations

import pytest
from pydantic import ValidationError

from babylon.sentinels.coverage.checks import check_gate_estate_coverage
from babylon.sentinels.coverage.registry import GATE_ESTATES, GateEstate

pytestmark = pytest.mark.unit


def test_registry_declares_the_regression_gate_estates() -> None:
    """The DoD gate's two claimed estates are declared."""
    names = {(row.gate_name, row.estate_name) for row in GATE_ESTATES}
    assert ("qa:regression", "economics_calculators") in names
    assert ("qa:regression", "financial_calculators") in names


def test_real_gate_is_not_blind() -> None:
    """INVARIANT: the DoD harness injects every key of each claimed estate."""
    assert check_gate_estate_coverage() == []


def test_efficacy_reds_when_the_harness_injects_nothing() -> None:
    """MUTATION: point the estate at a harness that injects none of its keys.

    ``web/game/map_contract.py`` is a real parseable file that mentions no
    economics service key — the exact shape of a gate that runs green while
    executing none of the estate it claims to guard.
    """
    blind = GateEstate(
        gate_name="phantom:gate",
        harness_file="web/game/map_contract.py",
        estate_name="economics_calculators",
        factory_file="src/babylon/domain/economics/factory.py",
        factory_symbol="create_economics_services",
    )
    findings = check_gate_estate_coverage((blind,))
    assert len(findings) == 1
    assert findings[0].startswith("[gate-blindness]")
    assert "phantom:gate" in findings[0]
    assert "melt_calculator" in findings[0]
    assert "REMEDY:" in findings[0]


def test_exempt_keys_narrow_the_claim_with_a_reason() -> None:
    """A key the gate deliberately does not inject is exempt WITH a reason."""
    narrowed = GateEstate(
        gate_name="phantom:gate",
        harness_file="web/game/map_contract.py",
        estate_name="economics_calculators",
        factory_file="src/babylon/domain/economics/factory.py",
        factory_symbol="create_economics_services",
        exempt_keys=(
            "melt_calculator",
            "basket_calculator",
            "gamma_calculator",
            "capital_calculator",
            "throughput_calculator",
            "transition_engine",
            "tensor_registry",
        ),
        exempt_reason="injected exemption covering the whole estate for this test",
    )
    assert check_gate_estate_coverage((narrowed,)) == []


def test_estate_rejects_exempt_keys_without_a_reason() -> None:
    """Narrowing a gate's claim silently is the failure mode; it is refused."""
    with pytest.raises(ValidationError, match="exempt_reason"):
        GateEstate(
            gate_name="phantom:gate",
            harness_file="tools/regression_test.py",
            estate_name="economics_calculators",
            factory_file="src/babylon/domain/economics/factory.py",
            factory_symbol="create_economics_services",
            exempt_keys=("melt_calculator",),
        )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_gate_blindness.py`
Expected: FAIL with `ImportError: cannot import name 'GATE_ESTATES' from 'babylon.sentinels.coverage.registry'`
- [ ] **Step 3: Write minimal implementation**

Append to `src/babylon/sentinels/coverage/registry.py`:
```python
class GateEstate(BaseModel):
    """One claim that a gate's harness exercises a named service estate.

    A gate can be green and blind: ``qa:regression``'s byte-identical baselines
    passed for months while injecting no economics calculators at all, so the
    project's Definition of Done never executed a line of the estate it claimed
    to guard. This row makes the claim checkable — the estate is whatever a
    service factory returns, and the harness must inject all of it.

    Frozen and ``extra="forbid"`` so a malformed row is a loud import-time
    failure (Constitution III.11).

    :ivar gate_name: the mise task the claim is about (e.g. ``"qa:regression"``).
    :ivar harness_file: repo-relative ``.py`` path of the harness the gate runs.
    :ivar estate_name: human name for the estate (e.g.
        ``"financial_calculators"``).
    :ivar factory_file: repo-relative ``.py`` path defining ``factory_symbol``.
    :ivar factory_symbol: the factory function whose returned dict keys ARE the
        estate.
    :ivar exempt_keys: keys the gate deliberately does not inject.
    :ivar exempt_reason: why those keys are exempt; required when
        ``exempt_keys`` is non-empty — a narrowed claim must be argued.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    gate_name: str
    harness_file: str
    estate_name: str
    factory_file: str
    factory_symbol: str
    exempt_keys: tuple[str, ...] = ()
    exempt_reason: str = ""

    @model_validator(mode="after")
    def _validate_shape(self) -> GateEstate:
        """Reject non-``.py`` paths and silently-narrowed claims.

        :returns: ``self`` when valid.
        :raises ValueError: If ``harness_file``/``factory_file`` is not a ``.py``
            path, or ``exempt_keys`` is non-empty without an ``exempt_reason``.
        """
        for label, value in (
            ("harness_file", self.harness_file),
            ("factory_file", self.factory_file),
        ):
            if not value.endswith(".py"):
                raise ValueError(
                    f"{self.gate_name!r}: {label} must be a .py path, got {value!r}"
                )
        if self.exempt_keys and not self.exempt_reason.strip():
            raise ValueError(
                f"{self.gate_name!r}: exempt_keys requires an exempt_reason — "
                "narrowing a gate's claim silently is the failure this row exists "
                "to forbid"
            )
        return self


#: The gates whose executed-code set is compared against the estate they claim
#: to guard. ``qa:regression`` is the project's Definition of Done.
GATE_ESTATES: tuple[GateEstate, ...] = (
    GateEstate(
        gate_name="qa:regression",
        harness_file="tools/regression_test.py",
        estate_name="economics_calculators",
        factory_file=f"{_ECON}/factory.py",
        factory_symbol="create_economics_services",
        exempt_keys=(
            "melt_calculator",
            "basket_calculator",
            "gamma_calculator",
            "capital_calculator",
            "throughput_calculator",
            "transition_engine",
            "tensor_registry",
        ),
        exempt_reason=(
            "U1.3 wired the harness's calculator_overrides from "
            "create_financial_services only (the D4 committed FRED fixture covers "
            "the Vol III series alone). The Volumes I/II economics estate needs "
            "its own committed fixture before the harness can inject it; narrowed "
            "deliberately, not silently."
        ),
    ),
    GateEstate(
        gate_name="qa:regression",
        harness_file="tools/regression_test.py",
        estate_name="financial_calculators",
        factory_file=f"{_ECON}/factory.py",
        factory_symbol="create_financial_services",
    ),
)
```
> `_ECON` is already defined above `DATA_REQUIREMENTS` as
> `"src/babylon/domain/economics"` — reuse it, do not redeclare.

Append to `src/babylon/sentinels/coverage/checks.py` (and add
`from babylon.sentinels._ast import referenced_names, returned_dict_keys`,
`from babylon.sentinels.coverage.registry import GATE_ESTATES, GateEstate`, and
`from babylon.sentinels.report import finding` to its imports):
```python
def check_gate_estate_coverage(
    estates: tuple[GateEstate, ...] = GATE_ESTATES,
) -> list[str]:
    """Every gate must inject every service key of the estate it claims to guard.

    Reads the factory's returned dict keys statically (the estate) and the
    harness's mentioned names (what it injects), and reports any key the harness
    never mentions. A gate that runs green while executing none of its estate is
    **gate-blind** — the state ``qa:regression`` was in for the whole economics
    estate.

    :param estates: Claims to check (defaults to the real :data:`GATE_ESTATES`;
        injectable so the efficacy test can supply a deliberately blind gate).
    :returns: Sorted agent-legible finding strings (empty when no gate is blind).
    :raises SentinelCheckError: If a harness or factory file is missing,
        unparseable, or the named factory returns no dict literal.
    """
    findings: list[str] = []
    for estate in estates:
        keys = returned_dict_keys(_REPO_ROOT / estate.factory_file, estate.factory_symbol)
        injected = referenced_names(_REPO_ROOT / estate.harness_file)
        # A harness that calls the factory itself injects the WHOLE estate;
        # requiring it to spell every key would force the harness to
        # duplicate the factory's own key list (DRY).
        if estate.factory_symbol in injected:
            continue
        missing = sorted(
            key for key in keys if key not in injected and key not in estate.exempt_keys
        )
        if not missing:
            continue
        findings.append(
            finding(
                error_class="gate-blindness",
                symbol=f"{estate.gate_name} / {estate.estate_name}",
                file=estate.harness_file,
                line=0,
                problem=(
                    f"claims to guard {estate.factory_symbol}'s estate but never "
                    f"injects {len(missing)} of its {len(keys)} service keys "
                    f"({', '.join(missing)}) — the gate can be green with that code "
                    "never executed"
                ),
                remedy=(
                    "build calculator_overrides in the harness from a committed "
                    "deterministic fixture so the estate actually runs, or narrow the "
                    "claim by declaring exempt_keys WITH an exempt_reason on the "
                    "GateEstate row"
                ),
            )
        )
    return sorted(findings)
```

Replace the `_ADVISORY_CHECKS` literal in `src/babylon/sentinels/coverage/checks.py`:
```python
#: Advisory tier: gate-blindness reports loudly but never gates, per the
#: standing owner ruling that sentinels are advisory and local/on-demand.
_ADVISORY_CHECKS: tuple[LabelledCheck, ...] = (
    ("DoD gate does not execute the estate it claims to guard", check_gate_estate_coverage),
)
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_gate_blindness.py`
Expected: PASS (5 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "feat(sentinels): gate-blindness check extends the coverage sentinel

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.11: sixth sentinel — cross-cutting public-surface baseline blindness

**Discovered defect class.** A scoped, per-task test run cannot see repo-wide
public-surface baselines. **Live specimen:** U2.3 added `"CapitalVolumeIIIDefines"`
to `babylon.config.defines.__all__` (`src/babylon/config/defines/__init__.py`, the
`__all__` list) **without** adding it to `EXPECTED_DEFINES_PUBLIC` in
`tests/unit/test_public_import_surface.py`. A scoped run of the U2.3 config tests
was green; only the full gate (`test_all_matches_baseline`) caught the drift. The
sentinel closes that blind spot: **statically** (AST, no import, no test run) detect
an `__all__` whose members diverge from the baseline frozenset that pins it.

**Files:**
- Create: `src/babylon/sentinels/surface/__init__.py`,
  `src/babylon/sentinels/surface/registry.py`,
  `src/babylon/sentinels/surface/checks.py`
- Modify: `src/babylon/sentinels/_ast.py` (add `frozenset_str_members`)
- Modify: `tools/sentinel_check.py` (`_MAINS` gains `"surface"`)
- Modify: `.mise.toml` (`check:surface` task, folded into `[tasks.check]`'s
  `depends` — a real gate, mirroring `check:seams`/`check:coverage`, NOT the
  advisory/local-only pattern U7.5/U7.6/U7.8 use for `check:liveness`/
  `check:aggregation-intensive`/`check:coupling`)
- Modify: `.github/workflows/ci.yml`, `.github/workflows/main.yml` (add an
  explicit `check:surface` step, mirroring the `check:seams`/`check:coverage`
  steps in both files)
- Test: `tests/unit/sentinels/test_surface.py` *(create)*

**Interfaces:**
- Consumes: `babylon.sentinels.report.format_finding` (U7.1 five-fact formatter);
  `babylon.sentinels._ast.literal_str_tuple` (reads a module `__all__` list) and
  the new `frozenset_str_members`.
- Produces: `PinnedSurface(BaseModel)` rows in `registry.py`; `check_pinned_surfaces() -> list[Finding]`
  and `main(argv) -> int` in `checks.py`; `"surface": surface_main` in `_MAINS`.

> **M1 carve-out:** the deliverable is a new package; U7.11's red is the collection
> error of `tests/unit/sentinels/test_surface.py` importing a not-yet-existing
> module. The mutation proof (Step 6) is a real assertion-level red/green on an
> injected specimen.

- [ ] **Step 1: Add the shared AST reader (assertion-level red first)**

Add to `tests/unit/sentinels/test_ast_helpers.py` (or create it):

```python
def test_frozenset_str_members_reads_a_frozenset_literal(tmp_path) -> None:
    from babylon.sentinels._ast import frozenset_str_members

    p = tmp_path / "m.py"
    p.write_text(
        "from __future__ import annotations\n"
        "EXPECTED: frozenset[str] = frozenset({'A', 'B', 'C'})\n"
    )
    assert set(frozenset_str_members(p, "EXPECTED")) == {"A", "B", "C"}
```

Run: `mise run test:q -- tests/unit/sentinels/test_ast_helpers.py::test_frozenset_str_members_reads_a_frozenset_literal`
Expected: FAIL — `ImportError: cannot import name 'frozenset_str_members'`.

Add to `src/babylon/sentinels/_ast.py`:

```python
def frozenset_str_members(path: Path, var_name: str) -> tuple[str, ...]:
    """Return the string members of a module-level ``frozenset({...})`` literal.

    Reads ``VAR = frozenset({...})`` or ``VAR: frozenset[str] = frozenset({...})``
    from ``path`` by AST — no import, no execution. Non-string members and
    non-literal sets are ignored (the sentinel compares only string symbols).
    """
    tree = ast.parse(path.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        target = None
        if isinstance(node, ast.Assign) and len(node.targets) == 1:
            target = node.targets[0]
        elif isinstance(node, ast.AnnAssign):
            target = node.target
        if not isinstance(target, ast.Name) or target.id != var_name:
            continue
        value = node.value
        # frozenset({...}) or frozenset([...])
        if (
            isinstance(value, ast.Call)
            and isinstance(value.func, ast.Name)
            and value.func.id == "frozenset"
            and value.args
        ):
            value = value.args[0]
        members: list[str] = []
        if isinstance(value, (ast.Set, ast.List, ast.Tuple)):
            for elt in value.elts:
                if isinstance(elt, ast.Constant) and isinstance(elt.value, str):
                    members.append(elt.value)
        return tuple(members)
    return ()
```

Run again — Expected: PASS.

- [ ] **Step 2: Declare the registry**

Create `src/babylon/sentinels/surface/registry.py`:

```python
"""Declared source of truth: public surfaces pinned by a baseline test.

Each :class:`PinnedSurface` names a package whose ``__all__`` is pinned by a
frozenset baseline in a test file. The static sensor in
:mod:`babylon.sentinels.surface.checks` proves the two agree — so an ``__all__``
edit without a matching baseline edit reds a scoped run, not only the full gate
(the cross-cutting public-surface baseline-blindness class).
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, model_validator


class PinnedSurface(BaseModel):
    """One package surface pinned by a baseline frozenset.

    :ivar name: stable identity (e.g. ``"config.defines"``).
    :ivar package_init: repo-relative path to the ``__init__.py`` declaring ``__all__``.
    :ivar baseline_file: repo-relative path to the test declaring the baseline.
    :ivar baseline_var: the frozenset variable name (e.g. ``EXPECTED_DEFINES_PUBLIC``).
    :ivar material_relation: why drift matters (Aleksandrov Test).
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    name: str
    package_init: str
    baseline_file: str
    baseline_var: str
    material_relation: str

    @model_validator(mode="after")
    def _validate_shape(self) -> PinnedSurface:
        if not self.package_init.endswith("__init__.py"):
            raise ValueError(f"{self.name!r}: package_init must be an __init__.py path")
        if not self.baseline_file.endswith(".py"):
            raise ValueError(f"{self.name!r}: baseline_file must be a .py path")
        return self


PINNED_SURFACES: tuple[PinnedSurface, ...] = (
    PinnedSurface(
        name="config.defines",
        package_init="src/babylon/config/defines/__init__.py",
        baseline_file="tests/unit/test_public_import_surface.py",
        baseline_var="EXPECTED_DEFINES_PUBLIC",
        material_relation=(
            "The moddable coefficient surface; a *Defines class silently added "
            "to __all__ without a baseline edit is a public-API change no "
            "scoped test sees (live specimen: CapitalVolumeIIIDefines, U2.3)."
        ),
    ),
    PinnedSurface(
        name="models.enums",
        package_init="src/babylon/models/enums/__init__.py",
        baseline_file="tests/unit/test_public_import_surface.py",
        baseline_var="EXPECTED_ENUMS_PUBLIC",
        material_relation=(
            "The enum vocabulary surface; an enum added to __all__ without a "
            "baseline edit ships an unpinned public symbol."
        ),
    ),
)
```

- [ ] **Step 3: The check**

Create `src/babylon/sentinels/surface/checks.py`:

```python
"""Static sensor: __all__ vs its baseline frozenset must not drift."""

from __future__ import annotations

import sys
from pathlib import Path

from babylon.sentinels._ast import frozenset_str_members, literal_str_tuple
from babylon.sentinels.report import Finding, format_finding
from babylon.sentinels.surface.registry import PINNED_SURFACES, PinnedSurface

_REPO_ROOT = Path(__file__).resolve().parents[4]


def _resolve(path_str: str) -> Path:
    """Join a registry path against the repo root, unless it is absolute.

    Registry rows hold repo-relative paths; the fixture-based mutation test
    passes absolute paths, which are used as-is (idempotent join).
    """
    path = Path(path_str)
    return path if path.is_absolute() else _REPO_ROOT / path


def _drift_finding(surface: PinnedSurface) -> Finding | None:
    """Return a Finding if ``surface``'s ``__all__`` and baseline disagree."""
    exported = set(literal_str_tuple(_resolve(surface.package_init), "__all__"))
    baseline = set(
        frozenset_str_members(_resolve(surface.baseline_file), surface.baseline_var)
    )
    missing = baseline - exported  # in baseline, dropped from __all__
    extra = exported - baseline  # added to __all__, no baseline edit
    if not missing and not extra:
        return None
    return Finding(
        error_class="public-surface baseline blindness",
        symbol=f"{surface.name}.__all__",
        location=surface.package_init,
        problem=(
            f"__all__ diverged from {surface.baseline_var} in "
            f"{surface.baseline_file}: added-without-baseline={sorted(extra)}, "
            f"dropped-but-still-pinned={sorted(missing)}"
        ),
        remedy=(
            f"Edit {surface.baseline_var} in {surface.baseline_file} to match "
            f"{surface.name}.__all__ in the SAME commit that changed __all__."
        ),
    )


def check_pinned_surfaces() -> list[Finding]:
    """Every pinned surface must agree with its baseline frozenset."""
    findings: list[Finding] = []
    for surface in PINNED_SURFACES:
        finding = _drift_finding(surface)
        if finding is not None:
            findings.append(finding)
    return findings


def main(argv: list[str] | None = None) -> int:
    """CLI entrypoint. Exit 1 on any drift, 0 when all surfaces agree."""
    findings = check_pinned_surfaces()
    for finding in findings:
        print(format_finding(finding), file=sys.stderr)
    return 1 if findings else 0
```

> Adapt `Finding`/`format_finding` to U7.1's actual `report.py` API (five facts:
> class, symbol, file:line, problem, remedy) — U7.11 lands after U7.1, so read the
> shipped `report.py` and match its constructor exactly.

- [ ] **Step 4: Wire the CLI + the gate (mise + CI)**

In `tools/sentinel_check.py`, add to the imports and `_MAINS`:
```python
from babylon.sentinels.surface.checks import main as surface_main
```
```python
    "surface": surface_main,
```

**Amended by owner ruling 2026-07-19:** unlike the other five sentinels in
this family (`check:liveness`/`check:aggregation-intensive`/`check:coupling`, U7.5/
U7.6/U7.8 — advisory and local-only, they never gate CI), `check:surface`
is wired as a real gate — exactly the way the repo's existing
`check:seams`/`check:coverage`
sentinels are wired (`.mise.toml`, verified against this worktree): a
`[tasks."check:X"]` block that is itself folded into `[tasks.check]`'s
`depends` list, plus an explicit step in both `.github/workflows/ci.yml` and
`.github/workflows/main.yml`.

In `.mise.toml`, add:
```toml
[tasks."check:surface"]
description = "Sentinel: __all__ vs its pinned baseline frozenset (public-surface baseline blindness gate)"
run = "poetry run python tools/sentinel_check.py surface --check"
```
and add `"check:surface"` to `[tasks.check]`'s `depends` list, alongside
`"check:hygiene"`, `"check:seams"`, `"check:coverage"`.

In both `.github/workflows/ci.yml` and `.github/workflows/main.yml`, add a
step mirroring the existing `check:seams`/`check:coverage` steps (same job,
directly after the `check:coverage` step):
```yaml
      - name: Public-surface baseline gate (__all__ vs pinned baseline)
        run: mise run check:surface
```

- [ ] **Step 5: Run the sensor against the real repo**

Run: `PYTHONPATH="$PWD/src" poetry run python tools/sentinel_check.py surface`
Expected: if the live specimen is still present, exit 1 with a finding naming
`CapitalVolumeIIIDefines` as `added-without-baseline`. **That is the sensor working.**
Fix it by adding `"CapitalVolumeIIIDefines"` to `EXPECTED_DEFINES_PUBLIC` in
`tests/unit/test_public_import_surface.py` (a one-line baseline edit), then re-run
— Expected: exit 0. (If U2.3's ceremony already added it, the sensor is green on
the real repo; the mutation proof below carries the red/green evidence.)

- [ ] **Step 6: Mutation proof (fixture-based, deterministic)**

Create `tests/unit/sentinels/test_surface.py`:

```python
"""U7.11: the public-surface sentinel reds on an __all__ that added a symbol
without a matching baseline edit (the U2.3 CapitalVolumeIIIDefines class),
and greens when they agree — proven on a fixture, independent of repo state."""

from __future__ import annotations

import pytest

from babylon.sentinels.surface.checks import _drift_finding
from babylon.sentinels.surface.registry import PinnedSurface


def _fixture(tmp_path, all_symbols: list[str], baseline: set[str]) -> PinnedSurface:
    """A pinned surface whose paths are ABSOLUTE (used as-is by ``_resolve``)."""
    init = tmp_path / "pkg" / "__init__.py"
    init.parent.mkdir(parents=True)
    init.write_text(f"__all__ = {all_symbols!r}\n")
    base = tmp_path / "test_surface_pin.py"
    base.write_text(f"EXPECTED = frozenset({sorted(baseline)!r})\n")
    return PinnedSurface(
        name="fixture.pkg",
        package_init=str(init),  # absolute -> _resolve returns it unchanged
        baseline_file=str(base),
        baseline_var="EXPECTED",
        material_relation="fixture",
    )


@pytest.mark.unit
class TestSurfaceSentinel:
    def test_reds_on_symbol_added_without_baseline(self, tmp_path) -> None:
        # Literal reproduction of the live specimen: a symbol in __all__ that
        # the baseline frozenset does not pin.
        surface = _fixture(tmp_path, ["Foo", "CapitalVolumeIIIDefines"], {"Foo"})
        finding = _drift_finding(surface)
        assert finding is not None
        assert "CapitalVolumeIIIDefines" in finding.problem

    def test_reds_on_baseline_symbol_dropped_from_all(self, tmp_path) -> None:
        surface = _fixture(tmp_path, ["Foo"], {"Foo", "Bar"})
        finding = _drift_finding(surface)
        assert finding is not None
        assert "Bar" in finding.problem

    def test_greens_when_all_matches_baseline(self, tmp_path) -> None:
        surface = _fixture(tmp_path, ["Foo", "Bar"], {"Foo", "Bar"})
        assert _drift_finding(surface) is None
```

> **Note:** the fixture paths are absolute, so `_resolve` (Step 3) returns them
> unchanged — no monkeypatching of `_REPO_ROOT` is needed. The load-bearing
> assertions are: (a) an `__all__` symbol absent from the baseline yields a finding
> naming it (the `CapitalVolumeIIIDefines` specimen); (b) a baseline symbol dropped
> from `__all__` also reds; (c) an aligned fixture yields `None`.

Run: `mise run test:q -- tests/unit/sentinels/test_surface.py`
Expected: PASS (the sentinel reds on the injected specimen, greens when aligned).

- [ ] **Step 7: Commit**
```bash
mise run commit -- "feat(sentinels): public-surface baseline-blindness sensor (6th class)

Static AST check: a package __all__ that drifts from its pinned baseline
frozenset (EXPECTED_*_PUBLIC) reds a scoped run, not only the full gate.
Live specimen: CapitalVolumeIIIDefines added to defines.__all__ (U2.3)
without a baseline edit. Mutation-proven on a fixture.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U7.10: Close the loop — full-family run, reference doc, ADR, state

**Files:**
- Create: `docs/reference/sentinel-error-classes.rst`
- Modify: `ai/state.yaml`
- Create: `ai/decisions/ADR088_sentinel_error_classes.yaml`
- Modify: `ai/decisions/index.yaml`
- Test: `tests/unit/sentinels/test_sentinel_family_cli.py`

**Interfaces:**
- Consumes: every sensor `main` registered in `tools/sentinel_check.py` by U7.5, U7.6, U7.8
- Produces: nothing consumed downstream; this task closes U7's acceptance criteria.

- [ ] **Step 1: Write the failing test**
```python
"""Family-level test: every U7 sensor is dispatchable and self-describing.

The six error classes are only useful if an agent can find and run them. This
pins the CLI surface: each new sensor is registered and exits 0 clean. Five
(liveness x2, aggregation, coupling, gate-blindness) are advisory per the
standing owner ruling; the sixth (public-surface baseline blindness, U7.11)
is wired as a real check:surface gate, not advisory (owner ruling 2026-07-19
-- see U7.11's Files: block).
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

pytestmark = pytest.mark.unit

_REPO_ROOT = Path(__file__).resolve().parents[3]
_TOOL_PATH = _REPO_ROOT / "tools" / "sentinel_check.py"


def _run(sensor: str) -> subprocess.CompletedProcess[str]:
    """Run one sensor through the family dispatcher.

    :param sensor: The sensor name to dispatch.
    :returns: The completed process.
    """
    return subprocess.run(
        [sys.executable, str(_TOOL_PATH), sensor],
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.mark.parametrize(
    ("sensor", "expected_word"),
    [
        ("liveness", "Liveness"),
        ("aggregation", "Aggregation"),
        ("coupling", "Coupling"),
        ("coverage", "Data coverage"),
    ],
)
def test_sensor_is_dispatchable_and_advisory(sensor: str, expected_word: str) -> None:
    """Each U7 sensor runs from the family CLI and does not gate."""
    result = _run(sensor)
    assert result.returncode == 0, result.stderr
    assert expected_word in result.stdout


def test_unknown_sensor_is_rejected() -> None:
    """An unregistered sensor name is refused by argparse, not silently ignored."""
    result = _run("no_such_sensor")
    assert result.returncode == 2
    assert "invalid choice" in result.stderr


def test_reference_doc_names_all_six_error_classes() -> None:
    """The reference doc is the agent-facing index of the six classes."""
    doc = (_REPO_ROOT / "docs/reference/sentinel-error-classes.rst").read_text(
        encoding="utf-8"
    )
    for error_class in (
        "correct-but-inert",
        "computed-but-never-consumed",
        "gate-blindness",
        "intensive-aggregation",
        "undeclared-coupling",
        "public-surface baseline blindness",
    ):
        assert error_class in doc


def test_state_yaml_strikes_the_four_owed_sentinel_classes() -> None:
    """U7 acceptance: the four previously-owed classes are struck by name."""
    state = (_REPO_ROOT / "ai/state.yaml").read_text(encoding="utf-8")
    for owed_class in (
        "correct-but-inert",
        "computed-but-never-consumed",
        "gate-blindness",
        "intensive-aggregation",
    ):
        assert owed_class in state, (
            f"ai/state.yaml does not record {owed_class!r} as struck from the "
            "owed-sentinel list (design §4 U7 acceptance)"
        )
    assert "STRUCK from the owed-sentinel list" in state
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/sentinels/test_sentinel_family_cli.py`
Expected: FAIL with `FileNotFoundError: .../docs/reference/sentinel-error-classes.rst`
- [ ] **Step 3: Write minimal implementation**

Run `ls ai/decisions/ | grep -oE 'ADR[0-9]+' | sort -t R -k2 -n | tail -1` first and
substitute the real next number in the filename, the top-level YAML key, and the
index.yaml entry. At authoring time the highest was ADR081, making this ADR088 and
U8.6's ADR089.

`docs/reference/sentinel-error-classes.rst`:
```rst
Sentinel error classes
======================

Six named failure classes, each with a sensor that finds it. Five are
**advisory** and **local/on-demand** — they print loudly, they never gate CI.
The sixth, public-surface baseline blindness (U7.11), is wired as a real
gate (``check:surface``, folded into ``mise run check`` and CI, owner ruling
2026-07-19 — see that task's Files: block) because a scoped test run cannot
otherwise see a repo-wide public-surface baseline drift.
Run one with ``mise run check:<name>`` or
``poetry run python tools/sentinel_check.py <sensor>``.

Every finding renders in one line::

    [<error-class>] <symbol> @ <file>:<line> — <problem> | REMEDY: <remedy>

correct-but-inert
-----------------

A producer runs, its models validate, and nothing downstream changes because it
ran. Volume III's entire estate was in this state: eleven calculators executing
correctly at every year boundary, reaching nothing.

:Sensor: ``babylon.sentinels.liveness.checks.check_producers_are_not_inert``
:Registry: ``babylon.sentinels.liveness.registry.LIVENESS_ROWS``
:Run: ``mise run check:liveness``
:Remedy: wire one output to a production consumer, or retire the producer.

computed-but-never-consumed
---------------------------

One declared output has no production reader and no declared reason. ``Path A``
ground rent, ``FictitiousCapitalStock``, ``DEBT_SPIRAL_THRESHOLD`` and
``pole_readings`` were all here. Dormancy is legitimate — *undeclared* dormancy
is not.

:Sensor: ``babylon.sentinels.liveness.checks.check_outputs_have_readers``
:Run: ``mise run check:liveness``
:Remedy: add a real consumer, or set ``dormant_reason`` on the registry row.

gate-blindness
--------------

A gate is green and blind: its harness never executes the estate it claims to
guard. ``qa:regression`` — the project's Definition of Done — injected no
economics calculators at all, so its byte-identical baselines never ran a line
of Volumes I, II or III.

:Sensor: ``babylon.sentinels.coverage.checks.check_gate_estate_coverage``
:Registry: ``babylon.sentinels.coverage.registry.GATE_ESTATES``
:Run: ``mise run check:coverage``
:Remedy: build the harness's overrides from a committed deterministic fixture,
   or narrow the claim with ``exempt_keys`` **and** an ``exempt_reason``.

intensive-aggregation
---------------------

An unweighted mean of a rate, ratio, share, balance or index across space or
class. The aggregate profit rate is ``Σs / Σ(c+v)``, never ``mean(rᵢ)``: the
unweighted form lets a four-hundred-person county swing a national threshold as
hard as Wayne County.

:Sensor: ``babylon.sentinels.aggregation.checks.check_no_unweighted_intensive_means``
:Registry: ``babylon.sentinels.aggregation.registry``
:Run: ``mise run check:aggregation-intensive``
:Remedy: aggregate numerator and denominator separately, or declare an
   ``AggregationExemption`` with the reason equal weighting is materially right.

undeclared-coupling
-------------------

The coupling graph is a claim about the code, and claims drift. Checked in
**both** directions: an edge with no real read behind it (declared-but-absent),
and a real read with no edge declaring it (present-but-undeclared). Four reserved
edges sat dormant for months; ``momentum_coupling`` was a real dependency nobody
had written down.

:Sensor: ``babylon.sentinels.coupling.checks`` (both directions)
:Registry: ``babylon.sentinels.coupling.registry.MEASUREMENT_DEPENDENCIES``
:Run: ``mise run check:coupling``
:Remedy: wire the dependency the edge claims, delete the edge, or declare the
   edge that the real read already implies.

public-surface baseline blindness
----------------------------------

A scoped, per-task test run cannot see repo-wide public-surface baselines: an
``__all__`` edited without its pinned ``EXPECTED_*_PUBLIC`` baseline reds only
the full gate, never a scoped run. Live specimen: ``CapitalVolumeIIIDefines``
added to ``babylon.config.defines.__all__`` (U2.3) without a matching edit to
``EXPECTED_DEFINES_PUBLIC``.

:Sensor: ``babylon.sentinels.surface.checks.check_pinned_surfaces``
:Registry: ``babylon.sentinels.surface.registry.PINNED_SURFACES``
:Run: ``mise run check:surface`` — unlike the other five, this one **gates**
   (folded into ``mise run check`` and CI, owner ruling 2026-07-19; U7.11).
:Remedy: edit the baseline frozenset in the same commit that changed
   ``__all__``.
```

`ai/decisions/ADR088_sentinel_error_classes.yaml` (match the field shape of the
neighbouring `ADR0NN_*.yaml` files — read `ai/decisions/ADR081_*.yaml` first and
mirror its keys exactly):
```yaml
ADR088_sentinel_error_classes:
  status: "accepted"
  date: "2026-07-18"
  title: "Six named sentinel error classes ship (five advisory local sensors, one CI gate)"
  context: >
    The Volume III money work surfaced six distinct failure classes that no
    existing test detected: an entire estate computing correctly and changing
    nothing (correct-but-inert); individual outputs with no reader
    (computed-but-never-consumed); the Definition-of-Done gate running green while
    executing none of the estate it claimed to guard (gate-blindness); an
    unweighted mean of an intensive across space (intensive-aggregation); a
    coupling graph whose declared edges had drifted from the dependencies they
    described, in both directions (undeclared-coupling); and a scoped test run
    that cannot see a repo-wide public-surface baseline drift
    (public-surface baseline blindness — live specimen CapitalVolumeIIIDefines,
    U2.3). Four were already on the owed-sentinel list; the fifth and sixth are
    new. The sixth generalises the governing principle one step further: unlike
    the first five (advisory, local-only), it is wired as a real CI gate
    (owner ruling 2026-07-19), because a public-surface drift is exactly the
    kind of defect a scoped run structurally cannot see.
  decision: >
    Ship one sensor per class in the babylon.sentinels family — new packages
    liveness/, aggregation/, coupling/, surface/ (U7.11), plus a gate-blindness
    check extending the existing coverage/ package. Five are ADVISORY and
    local/on-demand per the standing owner ruling: no nightly CI plumbing, no
    gating. The sixth, public-surface baseline blindness (check:surface), is
    the one exception (owner ruling 2026-07-19): it is folded into
    ``mise run check``'s depends and into CI, exactly the way check:seams and
    check:coverage are wired, because its whole point is catching what a
    scoped run cannot see. Each finding is agent-legible through a shared
    formatter (error class, symbol, file:line, problem, remedy) and each
    sensor ships a mutation test that injects the defect it exists to catch
    and proves the sensor reds.
  consequences: >
    The four owed classes are struck from the owed list. New mise tasks
    check:liveness, check:aggregation-intensive, check:coupling, check:surface;
    check:coverage gains an advisory tier. check:surface alone is folded into
    [tasks.check]'s depends and into ci.yml/main.yml as a real gate. The
    registries are hand-written dev-time contracts and must be grown as
    producers, couplings and public surfaces are added — an undeclared output,
    edge or surface drift is exactly what the sensors exist to surface, so
    growth is the point, not overhead.
  supersedes: []
  related:
    - ADR077
    - ADR078
    - ADR081
  evidence: |
    src/babylon/sentinels/report.py, src/babylon/sentinels/_ast.py,
    src/babylon/sentinels/liveness/, src/babylon/sentinels/aggregation/,
    src/babylon/sentinels/coupling/, src/babylon/sentinels/coverage/,
    docs/reference/sentinel-error-classes.rst
```

Add to `ai/decisions/index.yaml`, immediately after the `decisions:` line:
```yaml
  ADR088_sentinel_error_classes:
    title: 'Six named sentinel error classes ship: liveness (correct-but-inert, computed-but-never-consumed), aggregation (intensive-aggregation), coupling (undeclared-coupling, both directions), gate-blindness extending the coverage sentinel, and surface (public-surface baseline blindness) -- five advisory, the sixth (surface) a real CI gate per owner ruling 2026-07-19'
    status: accepted
    date: '2026-07-18'
    file: ADR088_sentinel_error_classes.yaml
```

Append to `ai/state.yaml`, immediately after the `truth_status: |` line:
```yaml
    (2026-07-18) SENTINEL FAMILY GROWS TO SIX ERROR CLASSES (ADR088): the
    Vol III money work surfaced six failure modes no existing test caught.
    New packages babylon.sentinels.liveness (correct-but-inert,
    computed-but-never-consumed), .aggregation (intensive-aggregation),
    .coupling (undeclared-coupling, checked in BOTH directions), plus
    check_gate_estate_coverage extending .coverage (gate-blindness — the
    state qa:regression was in for the whole economics estate), plus
    .surface (public-surface baseline blindness, U7.11 — live specimen
    CapitalVolumeIIIDefines added to defines.__all__ without a matching
    EXPECTED_DEFINES_PUBLIC edit, U2.3). Five are ADVISORY and
    local/on-demand per the standing owner ruling: new mise tasks
    check:liveness / check:aggregation-intensive / check:coupling; no CI gating, no
    nightly plumbing. The sixth, check:surface, is wired as a real gate
    (owner ruling 2026-07-19) — folded into mise run check's depends and
    into ci.yml/main.yml, exactly like check:seams/check:coverage. Each
    sensor ships a mutation test that injects its own defect and proves
    the sensor reds. STRUCK from the owed-sentinel list: correct-but-inert,
    computed-but-never-consumed, gate-blindness, intensive-aggregation.
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/sentinels/test_sentinel_family_cli.py`
Expected: PASS (7 passed)
- [ ] **Step 5: Commit**
```bash
mise run commit -- "docs(sentinels): reference doc + ADR088 for the six error classes

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

### Task U8.2: Capture the qa:regression RED diff as evidence

**Files:**
- Create: `tools/capture_qa_diff.py`
- Test: `tests/unit/tools/test_capture_qa_diff.py`

**Interfaces:**
- Consumes: `tools/regression_test.py compare` (the exact command `.mise.toml`'s `[tasks."qa:regression"]` runs: `poetry run python tools/regression_test.py compare`, `.mise.toml:730`).
- Produces: `reports/vol3-baseline-delta-raw-diff.txt` (real qa:regression output, RED) and `reports/vol3-e2e-regression-raw-diff.txt` (real `qa:e2e-regression` output) — both consumed by U8.3's report authoring.

- [ ] **Step 1: Write the failing test**
```python
"""Unit tests for tools/capture_qa_diff.py (U8.2's evidence-capture tool).

Mocks subprocess.run so this test is fast and independent of the engine's
actual current pass/fail state -- the REAL capture (this repo's real
qa:regression output, expected RED per design D3) happens in Step 4/5 of
this task by running the tool for real, not by this test.
"""

from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

from capture_qa_diff import capture_qa_regression_diff  # type: ignore[import-not-found]  # noqa: E402


def test_capture_qa_regression_diff_writes_file_and_returns_exit_code(
    tmp_path: Path,
) -> None:
    """Writes captured stdout/stderr to the given path and propagates the exit code."""
    fake_result = MagicMock(
        returncode=1,
        stdout="  Comparing imperial_circuit... FAIL\n    tick 38: p_w_wealth: 0.601234 != 0.598877\n",
        stderr="",
    )
    output_path = tmp_path / "diff.txt"

    with patch("capture_qa_diff.subprocess.run", return_value=fake_result) as mock_run:
        exit_code = capture_qa_regression_diff(output_path=output_path)

    assert exit_code == 1
    invoked_argv = mock_run.call_args.args[0]
    assert invoked_argv[1].endswith("regression_test.py")
    assert invoked_argv[2] == "compare"
    content = output_path.read_text()
    assert "FAIL" in content
    assert "exit code: 1" in content


def test_capture_qa_regression_diff_propagates_zero_exit_on_pass(tmp_path: Path) -> None:
    """A green qa:regression is captured and reported as exit code 0."""
    fake_result = MagicMock(returncode=0, stdout="Results: 5 passed, 0 failed\n", stderr="")
    output_path = tmp_path / "diff.txt"

    with patch("capture_qa_diff.subprocess.run", return_value=fake_result):
        exit_code = capture_qa_regression_diff(output_path=output_path)

    assert exit_code == 0
    assert "exit code: 0" in output_path.read_text()
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_capture_qa_diff.py`
Expected: FAIL with `ModuleNotFoundError: No module named 'capture_qa_diff'` (the module does not exist yet).
- [ ] **Step 3: Write minimal implementation**
```python
"""Capture `tools/regression_test.py compare`'s full stdout+stderr to a
fixed evidence file for the Vol III baseline-delta report (U8.2), and
propagate its exit code unchanged.

Usage:
    poetry run python tools/capture_qa_diff.py

Writes: reports/vol3-baseline-delta-raw-diff.txt
Exits: the underlying `compare` invocation's own exit code (0 = every
    scenario matched its committed baseline, 1 = at least one diverged).

See Also:
    :doc:`/reference/determinism-contract` for what qa:regression actually
    compares (checkpoint values + dense goldens).
"""

from __future__ import annotations

import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

OUTPUT_PATH: Path = Path(__file__).parent.parent / "reports" / "vol3-baseline-delta-raw-diff.txt"


def capture_qa_regression_diff(output_path: Path = OUTPUT_PATH) -> int:
    """Run `regression_test.py compare` and write its combined output to disk.

    Args:
        output_path: File to write the captured stdout+stderr into. Parent
            directories are created if missing.

    Returns:
        The exit code of the underlying `compare` invocation, unchanged.
    """
    result = subprocess.run(
        [sys.executable, str(Path(__file__).parent / "regression_test.py"), "compare"],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent,
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    header = (
        f"# qa:regression compare -- captured "
        f"{datetime.now(UTC).isoformat(timespec='seconds')}\n"
        f"# exit code: {result.returncode}\n\n"
    )
    output_path.write_text(header + result.stdout + result.stderr)
    return result.returncode


if __name__ == "__main__":
    sys.exit(capture_qa_regression_diff())
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_capture_qa_diff.py`
Expected: PASS (2 passed).
- [ ] **Step 5: Capture the REAL diff (not mocked) — this is the actual U8.2 deliverable**
Run, in order, against the real repo state (U1-U7 already landed on this branch):
```bash
poetry run python tools/capture_qa_diff.py; echo "exit=$?"
```
Expected: `exit=1` (RED). If `exit=0`, STOP — U1-U7's wiring produced zero behavioral difference across all 5 checkpoint scenarios, which contradicts the design's own diagnosis (§1.1: "s = p + i + r + t never evaluates in the shipped game" is being fixed) and needs investigation before continuing, not silent acceptance. Then capture the e2e leg too (per the design's §5 note that U1 wires the headless-runner path as well, so `qa:e2e-regression` may also have drifted):
```bash
mise run qa:e2e-regression > reports/vol3-e2e-regression-raw-diff.txt 2>&1; echo "exit=$?" >> reports/vol3-e2e-regression-raw-diff.txt
```
Record both exit codes verbatim — U8.3 needs the real value, not an assumption. (Program 23's precedent, ADR078: the 5-tick e2e leg was unaffected because no correction was reachable in 5 ticks; Vol III's annual/52-tick cadence for `FictitiousCapitalStock` may or may not fire inside a 5-tick run depending on whether tick 0 counts as a year boundary — do not assume either way, read the captured file.)
- [ ] **Step 6: Commit**
```bash
mise run commit -- "$(cat <<'EOF'
chore(tools): capture qa:regression + qa:e2e-regression RED diff as U8.2 evidence

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```
Note: `reports/vol3-baseline-delta-raw-diff.txt` and `reports/vol3-e2e-regression-raw-diff.txt` ARE committed here (they are the raw evidence artifacts U8.3 quotes from) — this is a docs/evidence commit, not the baseline-regeneration ceremony commit (that is U8.5, and stays strictly separate per the design's D3 ruling and the ADR077/078 precedent of a dedicated ceremony commit).

---

### Task U8.3: Author `reports/vol3-baseline-delta.md`

**Files:**
- Create: `reports/vol3-baseline-delta.md`
- Test: `tests/unit/tools/test_vol3_baseline_delta_report.py`

**Interfaces:**
- Consumes: `reports/vol3-baseline-delta-raw-diff.txt` + `reports/vol3-e2e-regression-raw-diff.txt` (U8.2, real captured output); `tools/regression_test.py`'s `SCENARIOS` dict (`tools/regression_test.py:100-134`) as the canonical scenario name/description list; U7.0's determinism-proof PASS evidence; the binding-contract symbol names from U1-U7 (`NATIONAL_FINANCIAL_ATTR`, `fictitious_anchor`/`serviceability_anchor`, the four new opposition keys `surplus_distribution`/`debt_spiral`/`credit`/`financial`, the new `MarketDefines` fields) as the vocabulary the "named mechanism" column must cite.
- Produces: the document U8.4's owner-approval gate is read against, and the document U8.5's ceremony commit message summarizes.

- [ ] **Step 1: Write the failing test**
```python
"""Structural contract for reports/vol3-baseline-delta.md (U8.3).

Does not assert on the FILLED content (scenario-specific numbers only
exist once someone has actually run qa:regression against the real U1-U7
diff -- that's U8.2's job) -- it asserts the document's required shape: a
title, an unmissable Owner Approval Gate section, and one dated subsection
per scenario in tools/regression_test.py's own SCENARIOS dict, so the
report can never silently drop a scenario the gate actually covers.
"""

from __future__ import annotations

import sys
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(TOOLS_DIR))

import regression_test as rt  # type: ignore[import-not-found]  # noqa: E402

REPORT_PATH = Path(__file__).resolve().parents[3] / "reports" / "vol3-baseline-delta.md"


def test_report_exists() -> None:
    assert REPORT_PATH.exists(), f"missing {REPORT_PATH} -- see design spec U8"


def test_report_has_owner_approval_gate_section() -> None:
    text = REPORT_PATH.read_text()
    assert "## Owner Approval Gate" in text
    assert "STOP" in text


def test_report_has_one_section_per_scenario() -> None:
    text = REPORT_PATH.read_text()
    for scenario_name in rt.SCENARIOS:
        assert f"### {scenario_name}" in text, (
            f"reports/vol3-baseline-delta.md is missing a section for "
            f"scenario {scenario_name!r} -- every SCENARIOS entry must be covered"
        )


def test_report_scenario_sections_require_named_mechanism() -> None:
    text = REPORT_PATH.read_text()
    assert "Named mechanism" in text
    assert "Materiality argument" in text
    assert "Principal contradiction" in text


def test_report_has_no_unfilled_placeholders_outside_the_approval_gate() -> None:
    """Every <FILL> must be resolved before the report is evidence.

    The three Owner Approval Gate fields are the ONLY legitimate remaining
    placeholders at U8.3 time — U8.4 fills those, and U8.5's Step 1 grep
    gate refuses to proceed while they are unfilled.
    """
    text = REPORT_PATH.read_text()
    remaining = text.count("<FILL")
    assert remaining <= 3, (
        f"reports/vol3-baseline-delta.md still has {remaining} <FILL> markers; "
        "only the three Owner Approval Gate fields may remain unfilled at U8.3"
    )
```
- [ ] **Step 2: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/tools/test_vol3_baseline_delta_report.py`
Expected: FAIL with `assert False` / `AssertionError: missing .../reports/vol3-baseline-delta.md` (the file does not exist yet).
- [ ] **Step 3: Write minimal implementation**
```markdown
# Vol III Money — Baseline Delta

**Status:** DRAFT — pending owner approval (see "Owner Approval Gate" below)
**Design:** `docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md`
**Precedent:** ADR077/ADR078 (Program 23 Market Scissors) — the same
"prove the delta in writing, then regenerate baselines in a dedicated
ceremony commit" pattern (D3 owner ruling here mirrors that precedent
explicitly).
**Raw evidence:** `reports/vol3-baseline-delta-raw-diff.txt` (qa:regression),
`reports/vol3-e2e-regression-raw-diff.txt` (qa:e2e-regression) — both
captured verbatim by U8.2, real output from this branch, not paraphrased.

## Why this document exists

Constitution III.7's falsifiability gate (`qa:regression`) is expected to go
RED the moment U1-U7 land: `s = p + i + r + t` has never evaluated in the
shipped game before this branch (design spec §1.1), so turning it on is a
genuine behavioral change, not a bug. D3 requires that change to be
explained, per scenario, by a *named mechanism* — never "values shifted" —
before any baseline is regenerated.

## Verification evidence

| Check | Command | Result | Evidence |
|---|---|---|---|
| Per-tick construction cadence determinism | `mise run test:q -- tests/unit/tools/test_regression_construction_cadence_determinism.py` | `<FILL: PASS, N passed in Ns>` | U7.0 (pre-U7 run) + U8.5 Step 5 (post-regeneration re-run) |
| qa:regression (checkpoint + dense) | `mise run qa:regression` | `<FILL: N passed, M failed>` | `reports/vol3-baseline-delta-raw-diff.txt` |
| qa:e2e-regression (detroit-tri-county, 5 ticks) | `mise run qa:e2e-regression` | `<FILL: PASS or FAIL>` | `reports/vol3-e2e-regression-raw-diff.txt` |
| `mise run check:quick` (lint+format+typecheck) | `mise run check:quick` | `<FILL>` | `<FILL: paste tail of output>` |
| Scoped unit sweep over every U1-U7 touched path | `mise run test:q -- tests/unit/economics tests/unit/dialectics tests/unit/engine tests/unit/config tests/unit/sentinels tests/unit/tools tests/unit/formulas` | `<FILL>` | `<FILL: paste tail of output>` |

## Per-scenario delta

Each section below MUST name the mechanism from the design's layer
structure (§3.1-3.5) that produced the delta — citing the actual symbol
(e.g. `NATIONAL_FINANCIAL_ATTR`, `fictitious_anchor`, the `surplus_distribution`
opposition key, `MarketDefines.anchor_pull`) — not a restatement that a
number changed.

**Endogenous interest (U9):** the delta report MUST include a per-scenario
line attributing the surplus-distribution `interest_payments` movement to
the endogenous rate `i = r·share(τ)` (Capital Vol. III Part V), and confirm
the rate is total (no `NoDataSentinel`) across all five scenarios. Do NOT
describe the interest rate as FRED-grounded.

**Wealth-weighted asymmetry field (U7.6b, owner ruling 2026-07-19):** the
delta report MUST also include a per-scenario line attributing dialectical
movement on the capital_labor / wage / imperial / tenancy oppositions to the
`_mean_asymmetry` wealth-weighting fix (`gap = Σ|b−a|/Σ(a+b)`, the
intensive-aggregation remedy the U7.6 sensor forced) — discovered live by
the sensor, fixed under the same owner gate that covers this report.

### imperial_circuit

**Description:** 4-node default scenario (`tools/regression_test.py` SCENARIOS).

| Field | Before (pre-Vol III) | After (this branch) | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | `<FILL>` | `<FILL>` | `<FILL>` |
| `final_outcome` | `<FILL>` | `<FILL>` | `<FILL>` |
| `ticks_survived` | `<FILL>` | `<FILL>` | `<FILL>` |
| Correction fired differently (tick, if any) | `<FILL>` | `<FILL>` | `<FILL: interest-burden term (§3.5.1) / debt-accumulation term (§3.5.2) / anchor pull (§3.3) — cite which>` |
| Principal contradiction at terminal tick | `<FILL>` | `<FILL>` | `<FILL: did surplus_distribution / debt_spiral / credit / financial displace the prior principal? Design Risk #4>` |
| First dense-trace divergence (tick, column) | n/a | `<FILL: from tests/baselines/dense/imperial_circuit.csv comparison>` | `<FILL>` |

**Materiality argument:** `<FILL — must cite a real mechanism, e.g. "tick_ground_rent now reads DefaultDistributionCalculator's real B230RC0Q173SBEA-backed figure instead of the DefaultRentCalculator NoDataSentinel it silently fell back to before (design §3.1 Path A/B correction), so the surplus_distribution opposition's rentier_share measure goes non-zero for the first time in this scenario's history">`

### two_node

**Description:** Minimal worker vs owner (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | `<FILL>` | `<FILL>` | `<FILL>` |
| `final_outcome` / `ticks_survived` | `<FILL>` | `<FILL>` | `<FILL>` |
| Correction fired differently | `<FILL>` | `<FILL>` | `<FILL>` |
| Principal contradiction at terminal tick | `<FILL>` | `<FILL>` | `<FILL>` |
| First dense-trace divergence | n/a | `<FILL>` | `<FILL>` |

**Materiality argument:** `<FILL — if this scenario has no county/territory FIPS data (its Territory nodes carry no county_fips, per src/babylon/models/entities/territory.py), the honest expectation may be NO delta beyond opposition_states key-set growth 6->10 (design §3.4, D2) and defines_hash. State plainly which is actually observed — do not assume, read the raw diff.>`

### starvation

**Description:** Low extraction efficiency stress (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | `<FILL>` | `<FILL>` | `<FILL>` |
| `final_outcome` / `ticks_survived` | `<FILL>` | `<FILL>` | `<FILL>` |
| Correction fired differently | `<FILL>` | `<FILL>` | `<FILL>` |
| Principal contradiction at terminal tick | `<FILL>` | `<FILL>` | `<FILL>` |
| First dense-trace divergence | n/a | `<FILL>` | `<FILL>` |

**Materiality argument:** `<FILL>`

### glut

**Description:** High extraction with metabolic overshoot (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | `<FILL>` | `<FILL>` | `<FILL>` |
| `final_outcome` / `ticks_survived` | `<FILL>` | `<FILL>` | `<FILL>` |
| Correction fired differently | `<FILL>` | `<FILL>` | `<FILL>` |
| Principal contradiction at terminal tick | `<FILL>` | `<FILL>` | `<FILL>` |
| First dense-trace divergence | n/a | `<FILL>` | `<FILL>` |

**Materiality argument:** `<FILL>`

### fascist_bifurcation

**Description:** Consciousness routing to national identity (`tools/regression_test.py` SCENARIOS).

| Field | Before | After | Named mechanism |
|---|---|---|---|
| First checkpoint tick with a value delta | `<FILL>` | `<FILL>` | `<FILL>` |
| `final_outcome` / `ticks_survived` | `<FILL>` | `<FILL>` | `<FILL>` |
| Correction fired differently | `<FILL>` | `<FILL>` | `<FILL>` |
| Principal contradiction at terminal tick | `<FILL>` | `<FILL>` | `<FILL>` |
| First dense-trace divergence | n/a | `<FILL>` | `<FILL>` |

**Materiality argument:** `<FILL>`

## qa:e2e-regression (detroit-tri-county, 5 ticks)

`<FILL: PASS/FAIL, and if it drifted, the same table shape as above scoped
to the 5-tick window. If PASS UNCHANGED, state which mechanism explains why
(e.g. "Vol III's national financial state is only recomputed at a 52-tick
year boundary (design §1.2 cadence table); a 5-tick window never crosses
one" — cite the actual code path, do not assume).>`

## Risks realized vs mitigated (design spec §7)

| Risk | Realized? | Evidence |
|---|---|---|
| #1 turning on never-executed code surfaces latent bugs | `<FILL>` | `<FILL: any ValidationError, unexpected NoDataSentinel, or crash hit during U8.2's run?>` |
| #4 catalog growth 6->10 changes principal-contradiction ranking | `<FILL>` | `<FILL: which scenarios saw a principal-contradiction change, per the per-scenario tables above>` |

## Owner Approval Gate

> **STOP. Do not proceed to baseline regeneration (U8.5) past this point
> without an explicit, recorded owner approval below.**
>
> This report is the complete, factual record of every behavioral delta
> `qa:regression` and `qa:e2e-regression` will encode as the new baseline.
> Once regenerated, the old (pre-Vol-III) baseline is gone from
> `tests/baselines/` — recoverable only via git history. The owner must
> read the per-scenario tables above and affirmatively approve *in this
> file* before U8.5 runs.

**Approved by:** `<FILL — leave blank until real sign-off; do not
pre-fill with a placeholder name>`
**Date:** `<FILL>`
**Approval text (verbatim):** `<FILL — quote the owner's actual approval
message, per the ADR078 precedent of quoting the owner's exact words as
the authorization record>`

## Post-approval regeneration record

`<FILL by U8.5 — the ceremony commit hash and a one-line confirmation that
qa:regression is green against the newly regenerated baselines.>`
```
- [ ] **Step 4: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/tools/test_vol3_baseline_delta_report.py`
Expected: FAIL on test_report_has_no_unfilled_placeholders_outside_the_approval_gate (the skeleton has ~60 markers) — this is the red that Step 5 turns green. The other 4 tests pass.
- [ ] **Step 5: Fill in every `<FILL>` with real data from U8.2's captured diffs**
This step is not optional and is not satisfied by the skeleton alone: open `reports/vol3-baseline-delta-raw-diff.txt` and `reports/vol3-e2e-regression-raw-diff.txt`, and for each scenario, read the actual tick/field/value triples the diff reports, then trace each one to its causing mechanism by reading the relevant U1-U7 code (the design spec §3 layer list is the index: Layer 1 = ground-rent repoint, Layer 1b = `NATIONAL_FINANCIAL_ATTR` publication, Layer 2 = `fictitious_anchor`/`serviceability_anchor`, Layer 3 = the four new opposition keys, Layer 4 = the scissors interest-burden/debt terms). Re-run `mise run test:q -- tests/unit/tools/test_vol3_baseline_delta_report.py` — all 5 must now pass.
- [ ] **Step 6: Commit**
```bash
mise run commit -- "$(cat <<'EOF'
docs(reports): vol3-baseline-delta.md — per-scenario delta analysis pending owner approval (U8.3)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task U8.4: OWNER APPROVAL GATE — STOP, do not regenerate baselines past this point

> ## ⛔ STOP — THIS TASK IS A HUMAN CHECKPOINT, NOT A CODE CHANGE ⛔
>
> **No file in this task is written by the engineer executing this plan except the one-line approval record below, and only after real, affirmative sign-off is received.**
>
> Do not run `mise run qa:regression-generate-dense` or any other baseline-regenerating command until this task is explicitly marked done by a human. Do not interpret silence, a general "looks good", or moving on to the next task as approval. Do not skip ahead to U8.5 "to save time" — U8.5's own Step 1 re-checks this gate and must halt if it is not satisfied.

**Files:**
- Modify: `reports/vol3-baseline-delta.md` (only the "Owner Approval Gate" section's three `Approved by:` / `Date:` / `Approval text (verbatim):` fields — nothing else in the file changes in this task)

**Interfaces:**
- Consumes: the completed, fully-filled `reports/vol3-baseline-delta.md` from U8.3 Step 5.
- Produces: a recorded, quotable approval that U8.5 Step 1 and U8.6's ADR both cite as the authorization for baseline regeneration (matching the ADR078 precedent of recording the owner's exact words as the ceremony's authorization record).

- [ ] **Step 1 (the only step): Present the completed report to Persephone Raskova and obtain explicit written approval**
Send (or otherwise present) the fully-filled `reports/vol3-baseline-delta.md` from U8.3 for review. Wait for an explicit response that approves regenerating the baselines — a real message, not an inferred green light. Do not proceed on the basis of prior general approvals of the design spec (§2 "owner decisions" approved the *design*, not this specific delta) — D3 explicitly separates "prove the delta" from "then regenerate," and this gate is where that separation is enforced.

Once received, edit `reports/vol3-baseline-delta.md`'s Owner Approval Gate section to fill in:
```markdown
**Approved by:** Persephone Raskova
**Date:** <actual date of approval>
**Approval text (verbatim):** <exact quoted text of the approval message>
```
Commit only this change:
```bash
mise run commit -- "$(cat <<'EOF'
docs(reports): record owner approval for vol3 baseline regeneration (U8.4)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

**If approval is withheld or the owner requests changes:** stop the plan here, address the requested changes in `reports/vol3-baseline-delta.md` (re-run U8.3 Step 5 against the specific concern), and re-present. Do not proceed to U8.5 under any circumstance until this task's commit exists with a real, non-placeholder `Approved by:` line.

---

### Task U8.5: Regenerate baselines in a dedicated ceremony commit

**Files:**
- Modify: `tests/baselines/imperial_circuit.json`, `tests/baselines/two_node.json`, `tests/baselines/starvation.json`, `tests/baselines/glut.json`, `tests/baselines/fascist_bifurcation.json`
- Modify: `tests/baselines/dense/imperial_circuit.csv`, `tests/baselines/dense/two_node.csv`, `tests/baselines/dense/starvation.csv`, `tests/baselines/dense/glut.csv`, `tests/baselines/dense/fascist_bifurcation.csv`
- Modify (only if U8.2's e2e capture showed drift): `tests/baselines/detroit-tri-county-5t.json`
- Test: none new — this task's "test" is re-running the existing gates before and after regeneration (Steps 2 and 4 below).

**Interfaces:**
- Consumes: the `Approved by:` record from U8.4 (Step 1 below re-checks it exists before doing anything else); `mise run qa:regression-generate-dense` (`.mise.toml:736-738`: `poetry run python tools/regression_test.py generate --force --dense`); the `compare-bundle` command's own stale hint text at `tools/regression_test.py:997-998` (`print("  cp <bundle>/summary.json tests/baselines/michigan-e2e.json")`) — **do not follow that printed instruction verbatim**; the correct target for `qa:e2e-regression` is `tests/baselines/detroit-tri-county-5t.json` per `.mise.toml:740-748`, not `michigan-e2e.json` (that filename is a leftover from before the spec-064/spec-086 rescoping documented in `tests/integration/test_ci_gate_baseline_compare.py:20-24`).
- Produces: the new committed baselines every future `qa:regression`/`qa:e2e-regression` run compares against; the commit hash U8.6's ADR cites as evidence.

- [ ] **Step 1: Verify the owner-approval gate is actually satisfied — halt if not**
```bash
grep -A1 "Approved by:" reports/vol3-baseline-delta.md
```
Expected: a real name and date, not `<FILL>` or blank. If this still shows a placeholder, STOP — return to U8.4. Do not proceed.

- [ ] **Step 2: Confirm the pre-regeneration RED state one more time (this is the "before" of the red/green pair)**
```bash
mise run qa:regression
```
Expected: FAIL (same shape as U8.2's captured evidence — if it now unexpectedly PASSES, something changed between U8.2 and now; investigate before regenerating over a possibly-different state than the one the owner approved).

- [ ] **Step 3: Regenerate the sampled-checkpoint and dense baselines**
```bash
mise run qa:regression-generate-dense
```
Expected output: `OK (<N> ticks, <OUTCOME>) + <scenario>.csv` for all 5 scenarios (`imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation` — the exact set in `tools/regression_test.py:100-134`'s `SCENARIOS`).

If U8.2's `reports/vol3-e2e-regression-raw-diff.txt` showed `qa:e2e-regression` also failed, regenerate it too — correctly, at the real target file, not the stale printed hint:
```bash
poetry run python -m babylon.engine.headless_runner \
  --scope detroit-tri-county --ticks 5 --strict | tee /tmp/vol3-e2e-run.log
# The runner's artifact directory is the last path it prints. Confirm by eye
# before copying — do NOT command-substitute the whole stdout stream.
ARTIFACT_DIR=$(grep -oE '/[^ ]*/artifacts?/[^ ]+' /tmp/vol3-e2e-run.log | tail -1)
test -f "$ARTIFACT_DIR/summary.json" || {
  echo "FAIL: could not locate summary.json under '$ARTIFACT_DIR'"; exit 1; }
cp "$ARTIFACT_DIR/summary.json" tests/baselines/detroit-tri-county-5t.json
```

- [ ] **Step 4: Verify GREEN — the "after" of the red/green pair**
```bash
mise run qa:regression
```
Expected: `Results: 5 passed, 0 failed` / `All regression tests passed!`. If `qa:e2e-regression` was regenerated in Step 3, also run:
```bash
mise run qa:e2e-regression
```
Expected: exit 0, `All regression checks passed.`

- [ ] **Step 5: Re-verify determinism post-regeneration (belt-and-suspenders, per ADR078's "Determinism gate re-verified post-flip" precedent)**
```bash
mise run test:q -- tests/unit/tools/test_regression_construction_cadence_determinism.py
```
Expected: PASS (same test from U7.0 — re-running it here proves the newly-regenerated baselines aren't themselves an artifact of a lucky non-deterministic run).

- [ ] **Step 6b: Resolve the e2e clause for the ceremony commit message**
```bash
E2E_CLAUSE=$(grep -q "^exit=0" reports/vol3-e2e-regression-raw-diff.txt \
  && echo "PASS UNCHANGED (never regenerated)" \
  || echo "regenerated in Step 3 and now PASS")
echo "E2E clause: $E2E_CLAUSE"
```

- [ ] **Step 7: Commit — the dedicated ceremony commit**
```bash
git add tests/baselines/imperial_circuit.json tests/baselines/two_node.json \
  tests/baselines/starvation.json tests/baselines/glut.json \
  tests/baselines/fascist_bifurcation.json tests/baselines/dense/*.csv
# Only if regenerated in Step 3:
git add tests/baselines/detroit-tri-county-5t.json
git commit -m "$(cat <<EOF
feat(economics)!: PROMOTION CEREMONY — Vol III money wired live; all baselines regenerated

Owner-authorized (see reports/vol3-baseline-delta.md "Owner Approval
Gate" for the recorded approval text and date). Per docs/superpowers/
specs/2026-07-18-vol3-money-scissors-design.md D3: U1-U7 landed Vol III
money on by default; this commit is the dedicated ceremony that
regenerates every qa:regression baseline (sampled JSONs + dense goldens,
5 scenarios) after the written per-scenario delta analysis was approved.

Per-scenario drift: see reports/vol3-baseline-delta.md for the full
mechanism-by-mechanism breakdown (ground-rent Path A repoint, the
NationalFinancialParameters publication, the monetary anchor, the four
new bound oppositions surplus_distribution/debt_spiral/credit/financial,
and the interest-burden/debt-accumulation correction terms).

Gates: mise run qa:regression 5/5 vs the new baselines; mise run
qa:e2e-regression ${E2E_CLAUSE}; two-independent-
process determinism proof re-verified post-regeneration
(tests/unit/tools/test_regression_construction_cadence_determinism.py).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```
Do not use `mise run commit --` for this specific commit — its re-staging sweep is documented to interfere with a deliberately partial/curated staging set (`ai/decisions/` note on `mise run commit` restaging gotcha); this ceremony commit's file list must be exact, so stage and commit with plain `git add`/`git commit` as shown.

- [ ] **Step 8: Record the ceremony commit hash in the report (separate, follow-up commit)**
```bash
CEREMONY_HASH=$(git rev-parse --short HEAD)
```
Edit `reports/vol3-baseline-delta.md`'s final section (`git commit --amend` is not viable here — the ceremony commit's file list must stay exact — so the hash is recorded in this dedicated follow-up commit instead):
```markdown
## Post-approval regeneration record

Baselines regenerated in commit `$CEREMONY_HASH` (substitute the real value).
`mise run qa:regression`: 5 passed, 0 failed (green against the new
baselines). `mise run qa:e2e-regression`: `${E2E_CLAUSE}` (already computed
in Step 6b above — substitute its resolved value, not the literal token).
```
Then commit:
```bash
mise run commit -- "$(cat <<'EOF'
docs(reports): record the vol3 baseline ceremony commit hash (U8.5)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

### Task U8.6: ADR + index.yaml catalog entry + ai/state.yaml

**Files:**
- Create: `ai/decisions/ADR089_vol3_money_scissors.yaml` (verified free; still run the `ls`/`grep` check in Step 1 and trust its output over this literal)
- Modify: `ai/decisions/index.yaml:6-7` (insert new entry as the first item under `decisions:`, before `ADR081_declarative_system_ordering:`)
- Modify: `ai/state.yaml:5-7` (bump `meta.version`, `meta.updated`, and prepend a new paragraph to `truth_status` immediately after the `truth_status: |` line, before the existing 2026-07-17 paragraph)

**Interfaces:**
- Consumes: the ceremony commit hash from U8.5 Step 7; the approval record from U8.4; the filled `reports/vol3-baseline-delta.md` from U8.3.
- Produces: the permanent governance record for this work — nothing downstream in this plan depends on it, but it is the acceptance-criteria closer for U8 ("ADR landed; `ai/state.yaml` updated").

- [ ] **Step 1: Determine the real next ADR number — do not assume ADR089**
```bash
ls ai/decisions/ | grep -oE 'ADR[0-9]+' | sort -t R -k2 -n | tail -1
```
U7.10 lands `ADR088_sentinel_error_classes.yaml` before this task runs, so the next free number is `ADR089` — but other work may have landed since, so trust the command's output, not this note. The rest of this task uses `ADR089` as a placeholder; substitute the real number everywhere (filename, the `index.yaml` key, and the top-level YAML key inside the ADR file itself, which must match the filename per every existing ADR's convention).

- [ ] **Step 2: Write the failing test**
```python
"""Structural gate: every ADR file's declared top-level key matches its
filename, and every ADR file has a corresponding index.yaml catalog entry.
Mirrors the existing repo convention (every file under ai/decisions/ pairs
one YAML top-level key == filename stem, catalogued in index.yaml's
`decisions:` map) -- this test targets specifically the new ADR this task
adds, so it fails until the file exists and both edits are made.
"""

from __future__ import annotations

from pathlib import Path

import yaml

DECISIONS_DIR = Path(__file__).resolve().parents[3] / "ai" / "decisions"
INDEX_PATH = DECISIONS_DIR / "index.yaml"

# Substitute the real number determined in Step 1.
NEW_ADR_STEM = "ADR089_vol3_money_scissors"


def test_new_adr_file_exists_with_matching_top_level_key() -> None:
    adr_path = DECISIONS_DIR / f"{NEW_ADR_STEM}.yaml"
    assert adr_path.exists(), f"missing {adr_path}"
    data = yaml.safe_load(adr_path.read_text())
    assert list(data.keys()) == [NEW_ADR_STEM], (
        f"{adr_path} top-level key must equal its filename stem"
    )
    entry = data[NEW_ADR_STEM]
    assert entry["status"] == "accepted"
    for required_field in ("date", "title", "context", "decision", "consequences", "evidence"):
        assert required_field in entry, f"{adr_path} missing required field {required_field!r}"


def test_new_adr_is_catalogued_in_index() -> None:
    index = yaml.safe_load(INDEX_PATH.read_text())
    assert NEW_ADR_STEM in index["decisions"], (
        f"{NEW_ADR_STEM} is missing from ai/decisions/index.yaml's decisions: map"
    )
    catalog_entry = index["decisions"][NEW_ADR_STEM]
    assert catalog_entry["file"] == f"{NEW_ADR_STEM}.yaml"
    assert catalog_entry["status"] == "accepted"


def test_state_yaml_records_the_vol3_ceremony() -> None:
    """The state file must name the ADR and the graph key, not just the branch."""
    state_path = Path(__file__).resolve().parents[3] / "ai" / "state.yaml"
    text = state_path.read_text()
    assert NEW_ADR_STEM.split("_")[0] in text, "state.yaml does not cite the new ADR id"
    assert "NATIONAL_FINANCIAL_ATTR" in text, (
        "state.yaml does not name the graph key U3 introduced"
    )
    assert "VOL III MONEY WIRED LIVE" in text


def test_no_unfilled_placeholders_in_the_governance_records() -> None:
    """Every <FILL> / <SUBSTITUTE> marker must be resolved before these land.

    Unlike U8.3's `reports/vol3-baseline-delta.md` -- a draft evidence
    artifact allowed placeholders everywhere but its Owner Approval Gate
    until that gate is satisfied -- ADR089 and the new ai/state.yaml
    paragraph are permanent governance history the moment this task
    commits them. Zero tolerance, no carve-out.
    """
    adr_path = DECISIONS_DIR / f"{NEW_ADR_STEM}.yaml"
    state_path = Path(__file__).resolve().parents[3] / "ai" / "state.yaml"
    if not adr_path.exists():
        return  # covered by test_new_adr_file_exists_with_matching_top_level_key
    for path in (adr_path, state_path):
        text = path.read_text()
        assert "<FILL" not in text, f"{path} still has an unresolved <FILL marker"
        assert "<SUBSTITUTE" not in text, f"{path} still has an unresolved <SUBSTITUTE marker"
```
- [ ] **Step 3: Run test to verify it fails**
Run: `mise run test:q -- tests/unit/decisions/test_adr083_vol3_money_scissors.py` (create the parent dir if `tests/unit/decisions/` doesn't yet exist — check first with `ls tests/unit/decisions/ 2>/dev/null`; if absent, this is a new test directory and needs no `__init__.py` beyond matching the sibling convention used elsewhere, e.g. `tests/unit/tools/__init__.py`)
Expected: FAIL with `AssertionError: missing .../ai/decisions/ADR089_vol3_money_scissors.yaml`.
- [ ] **Step 4: Write minimal implementation**

Create `ai/decisions/ADR089_vol3_money_scissors.yaml` (substitute the real number from Step 1):
```yaml
ADR089_vol3_money_scissors:
  status: "accepted"
  date: "2026-07-18"
  title: "Volume III Money wired live through the Value-Price Scissors: the dormant Vol III estate (SurplusValueDistribution, DebtAccumulation, FictitiousCapitalStock) connected via calculator_overrides + a new NationalFinancialParameters graph publication + a monetary anchor pulling the market-scissors oscillator toward real FRED-backed ratios where data exists (2010-2024), four new bound oppositions (surplus_distribution/debt_spiral/credit/financial, catalog 6->10) with CouplingGraph activated as ContradictionSystem's first production consumer, a 13-item honesty sweep (including a latent year-ceiling ValidationError crash on the live MELT path), and six new sentinel classes; baselines regenerated per the recorded owner-approval ceremony -- and an endogenous national interest rate (Capital Vol. III Part V) that replaces the FRED interest read with a bounded share of the average rate of profit driven by loan-market tightness, the rate now total (never absent), U9"
  context: |
    Design: docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md.
    spec-024-capital-volume-iii was built but disconnected: create_financial_
    services() and the tensor_registry it needs only reached the live engine
    through engine/simulation/_legacy.py's calculator_overrides conduit, not
    through web/game/engine_bridge.py (the playable game, national-only) nor
    tools/regression_test.py (the qa:regression gate, fully dark). Two
    independent "fictitious capital" concepts existed with no coupling
    between them (FictitiousCapitalStock, FRED-grounded, dead after one
    boolean; fictitious_log, the synthetic oscillator with all the teeth).
    Four dialectic coupling edges (surplus_distribution->debt_spiral,
    credit->financial, plus price_value<->financial derived against
    coupling.py's operational definitions) sat reserved-but-unbound for
    months, logged at INFO and never surfaced.
  decision: |
    U1: wired create_financial_services()+tensor_registry through
    calculator_overrides in both the headless runner and the web bridge;
    built a committed deterministic FRED fixture
    (tests/fixtures/vol3_fred_series.json, 10 series) so qa:regression can
    inject the same overrides hermetically (no DB, no babylon-data drive);
    repointed graph_bridge.py's tick_ground_rent stamp from the always-None
    DefaultRentCalculator path to DefaultDistributionCalculator's real
    FRED-backed figure.
    U2: honesty sweep -- killed inline coefficients (DEBT_SPIRAL_THRESHOLD,
    DISTRIBUTION_EPSILON, COUNTER_TENDENCY_WEIGHTS,
    IMPERIAL_RENT_REFERENCE_SCALE, STAGNATION_CREDIT_GROWTH's bare-
    GameDefines() import bug) into GameDefines fields; fixed the
    ge=2007/le=2040 ValidationError (reached at tick ~1612 of 5200 on the
    already-live MELT path) to degrade to NoDataSentinel past the horizon.
    U3: NationalFinancialParameters instantiated in
    _compute_national_financial_state, published via graph_bridge under
    NATIONAL_FINANCIAL_ATTR = "national_financial", readable by
    CONSEQUENCE-phase Systems the same tick it's computed.
    U4: domain/economics/monetary/anchor.py -- fictitious_anchor (log-space
    pull toward FictitiousCapitalStock.ratio_to_real) and
    serviceability_anchor (real interest burden i/s), both
    NoDataSentinel-honest, pure, RNG-free; scissors dynamics unchanged when
    absent (the ~85%-of-campaign default state).
    U5: four new oppositions bound in build_default_registry (catalog 6->10,
    all antagonistic=False -- intra-capital conflict, not the
    capital_labor/imperial class-rupture pair); GraphInputs gains
    rentier_share/debt_ratio/credit_fragility/financialization_index;
    CouplingGraph activated as ServiceContainer's first production
    consumer, constraining ContradictionSystem's principal-contradiction
    ranking so a transforms target cannot rank principal while its source
    reads absent.
    U6: MarketScissorsSystem's calculate_serviceable_divergence gains an
    interest-burden term (MarketDefines.correction_interest_slope);
    correction severity gains an accumulated-debt term
    (correction_debt_slope); the fictitious oscillator is pulled toward
    the anchor by MarketDefines.anchor_pull when present.
    U7: six sentinel packages/extensions -- sentinels/liveness/
    (correct-but-inert), sentinels/aggregation/ (intensive-aggregation,
    fixing _mean_profit_rate's unweighted-mean variance error),
    sentinels/coupling/ (undeclared-coupling, verifying every declared
    CouplingGraph edge against a real measurement dependency in both
    directions), gate-blindness extending the existing
    sentinels/coverage/ package, and sentinels/surface/ (public-surface
    baseline blindness -- wired as a real check:surface gate, owner
    ruling 2026-07-19).
    U8: empirical two-independent-process determinism proof (ADR056
    precedent) of the per-tick ServiceContainer.create(**overrides)
    cadence now carrying the Vol III calculator_overrides;
    reports/vol3-baseline-delta.md's per-scenario mechanism-named delta
    analysis; owner approval recorded; all 5 qa:regression baselines (plus
    qa:e2e-regression if it drifted) regenerated in a dedicated ceremony
    commit `<FILL: the U8.5 Step 7 commit hash>`.
  consequences: |
    Dialectic catalog grows 6 -> 10 bound oppositions; CouplingGraph gains
    its first production consumer (ContradictionSystem's principal ranking),
    so a `transforms` target can no longer rank principal while its source
    reads absent -- the ONE place this program alters existing semantics
    rather than filling absence. GameDefines gains a `capital_vol3` category
    (9 fields, including credit_fragility_scale) and three MarketDefines
    coefficients (anchor_pull, correction_interest_slope,
    correction_debt_slope); defines.yaml regenerated. NationalTickParameters
    loses its le=2040 ceiling (the fabricated-year bug); the Volume III
    structured layer degrades to NoDataSentinel past 2040 rather than
    raising. CreditState gains its first production constructor (U3.4) and
    DEBT_SPIRAL_THRESHOLD its first consumer (U5.10) -- both were dead on
    arrival before this program. All 5 qa:regression baselines regenerated
    in ceremony commit <SUBSTITUTE: U8.5 Step 7 hash>. Principal
    contradiction changed in <SUBSTITUTE: scenario list from
    reports/vol3-baseline-delta.md's per-scenario tables>.
    DEFERRED, explicitly: the three-way agricultural/resource/urban rent
    split (RentCategory stays DORMANT -- no data source); the Volumes I/II
    economics estate remains outside the qa:regression gate, narrowed with
    a reason in GATE_ESTATES rather than silently; no UI/lens work; no FRED
    acquisition beyond the ten committed series in
    tests/fixtures/vol3_fred_series.json. FRED BAA10Y/net-interest series
    demoted to calibration of capital_vol3.interest_profit_share_*
    (test-pinned), U9.
  evidence: |
    docs/superpowers/specs/2026-07-18-vol3-money-scissors-design.md,
    reports/vol3-baseline-delta.md,
    tests/unit/tools/test_regression_construction_cadence_determinism.py,
    src/babylon/domain/economics/monetary/anchor.py,
    src/babylon/domain/economics/tick/graph_bridge.py,
    src/babylon/domain/dialectics/instances/catalog.py,
    src/babylon/config/defines/market.py,
    src/babylon/sentinels/liveness/, src/babylon/sentinels/aggregation/,
    src/babylon/sentinels/coupling/, src/babylon/sentinels/coverage/,
    tests/fixtures/vol3_fred_series.json,
    ADR056_spec102_gamma_hydration_and_shocks.yaml (determinism-proof
    precedent), ADR077_market_scissors.yaml, ADR078_market_correction.yaml
    (shadow-then-ceremony precedent),
    ADR088_sentinel_error_classes.yaml (the U7 sentinel family)
```

Edit `ai/decisions/index.yaml` — insert immediately after line 6 (`decisions:`), before the existing `ADR081_declarative_system_ordering:` entry:
```yaml
  ADR089_vol3_money_scissors:
    title: 'Volume III Money wired live through the Value-Price Scissors: the dormant Vol III estate connected via calculator_overrides + a new NationalFinancialParameters graph publication + a monetary anchor, four new bound oppositions (catalog 6->10) with CouplingGraph activated as ContradictionSystem''s first production consumer, a 13-item honesty sweep, six new sentinel classes, and an endogenous national interest rate (Capital Vol. III Part V, U9 -- the rate total, never absent); baselines regenerated per the recorded owner-approval ceremony'
    status: accepted
    date: '2026-07-18'
    file: ADR089_vol3_money_scissors.yaml
```

Edit `ai/state.yaml`: change line 5 from `version: "2.38.0"  # ...` to `version: "2.39.0"  # ...` (append a comment noting this ADR if the existing comment convention warrants it), change line 6's `updated:` to today's date, and insert a new paragraph immediately after line 7 (`truth_status: |`), before the existing `(2026-07-17 overnight) THE EXECUTION SLATE...` paragraph:
```yaml
    (<FILL: today's date>) VOL III MONEY WIRED LIVE (ADR089; branch
    refactor/vol3-money-scissors, design docs/superpowers/specs/
    2026-07-18-vol3-money-scissors-design.md): the dormant Vol III
    economics estate (spec-024) connected end-to-end for the first time --
    s = p + i + r + t now evaluates in the shipped game, not just in
    isolated tests. NationalFinancialParameters published to the graph
    (NATIONAL_FINANCIAL_ATTR); a monetary anchor (domain/economics/monetary/
    anchor.py) pulls the market-scissors oscillator toward real FRED ratios
    2010-2024, honestly absent thereafter (~85% of a campaign); dialectic
    catalog grows 6->10 oppositions (surplus_distribution, debt_spiral,
    credit, financial) with CouplingGraph activated as
    ContradictionSystem's first production consumer; a live year-2041
    ValidationError crash on the already-wired MELT path fixed
    (NationalTickParameters' le=2040 constraint now degrades to
    NoDataSentinel). Baselines regenerated `<FILL: ceremony commit hash>`
    after a written, owner-approved per-scenario delta analysis
    (reports/vol3-baseline-delta.md) per D3 -- see ADR089 for the full
    mechanism breakdown.
```
- [ ] **Step 4b: Resolve the five placeholder markers Step 4 just wrote**
Step 4 left five literal `<FILL:` / `<SUBSTITUTE:` markers behind: one in
the ADR's `decision:` block, two in its `consequences:` block, and two in
the `ai/state.yaml` paragraph. These two files are permanent governance
records the moment Step 6 commits them — resolve every marker now, before
Step 5's test is allowed to claim green. Do this as one shell session so
the shell variables below survive to the edit commands that use them:
```bash
# Ceremony commit hash: read the value U8.5 Step 8 already recorded in
# reports/vol3-baseline-delta.md's "Post-approval regeneration record"
# section -- that is the one place the real hash was captured against the
# actual ceremony commit, so read it rather than re-deriving it from git
# log by commit-message text.
CEREMONY_HASH=$(grep -oE 'commit `[0-9a-f]{7,40}`' reports/vol3-baseline-delta.md | grep -oE '[0-9a-f]{7,40}' | head -1)
test -n "$CEREMONY_HASH" || {
  echo "FAIL: could not extract the ceremony commit hash from reports/vol3-baseline-delta.md (U8.5 Step 8 must have run first)"
  exit 1
}
echo "Ceremony hash: $CEREMONY_HASH"

TODAY=$(date +%F)
echo "Today: $TODAY"

# Scenario list: any scenario whose "Principal contradiction at terminal
# tick" row has a different Before/After value in the now-filled report.
SCENARIO_LIST=$(python3 - <<'PY'
import re
from pathlib import Path

text = Path("reports/vol3-baseline-delta.md").read_text()
scenarios = ["imperial_circuit", "two_node", "starvation", "glut", "fascist_bifurcation"]
pattern = re.compile(
    r"^### (" + "|".join(scenarios) + r")\n(.*?)(?=^#{2,3} |\Z)",
    re.MULTILINE | re.DOTALL,
)
sections = {m.group(1): m.group(2) for m in pattern.finditer(text)}
changed = []
for name in scenarios:
    section = sections.get(name)
    if section is None:
        raise SystemExit(f"FAIL: no '### {name}' section found in reports/vol3-baseline-delta.md")
    row = re.search(r"\|\s*Principal contradiction at terminal tick\s*\|(.*?)\|(.*?)\|", section)
    if row is None:
        raise SystemExit(f"FAIL: no principal-contradiction row found for {name}")
    before, after = row.group(1).strip(), row.group(2).strip()
    if before != after:
        changed.append(name)
print(", ".join(changed) if changed else "none")
PY
)
test -n "$SCENARIO_LIST" || { echo "FAIL: SCENARIO_LIST resolution produced no output"; exit 1; }
echo "Scenario list: $SCENARIO_LIST"

export CEREMONY_HASH TODAY SCENARIO_LIST
python3 - <<'PY'
import os
from pathlib import Path

ceremony_hash = os.environ["CEREMONY_HASH"]
today = os.environ["TODAY"]
scenario_list = os.environ["SCENARIO_LIST"]

adr_path = Path("ai/decisions/ADR089_vol3_money_scissors.yaml")
text = adr_path.read_text()

decision_marker = "commit `<FILL: the U8.5 Step 7 commit hash>`."
assert decision_marker in text, "decision: marker not found -- Step 4's YAML text drifted"
text = text.replace(decision_marker, f"commit `{ceremony_hash}`.")

consequences_hash_marker = "ceremony commit <SUBSTITUTE: U8.5 Step 7 hash>."
assert consequences_hash_marker in text, "consequences: hash marker not found"
text = text.replace(consequences_hash_marker, f"ceremony commit {ceremony_hash}.")

scenario_marker = (
    "Principal\n    contradiction changed in <SUBSTITUTE: scenario list from\n"
    "    reports/vol3-baseline-delta.md's per-scenario tables>."
)
assert scenario_marker in text, "consequences: scenario-list marker not found"
text = text.replace(
    scenario_marker,
    f"Principal\n    contradiction changed in: {scenario_list}.",
)

adr_path.write_text(text)
print("ADR089 markers resolved.")

state_path = Path("ai/state.yaml")
text = state_path.read_text()

date_marker = "(<FILL: today's date>) VOL III MONEY WIRED LIVE"
assert date_marker in text, "state.yaml date marker not found -- Step 4's paragraph drifted"
text = text.replace(date_marker, f"({today}) VOL III MONEY WIRED LIVE")

hash_marker = "Baselines regenerated `<FILL: ceremony commit hash>`"
assert hash_marker in text, "state.yaml ceremony-hash marker not found"
text = text.replace(hash_marker, f"Baselines regenerated `{ceremony_hash}`")

state_path.write_text(text)
print("state.yaml markers resolved.")
PY

grep -n '<FILL\|<SUBSTITUTE' ai/decisions/ADR089_vol3_money_scissors.yaml ai/state.yaml \
  && { echo "FAIL: placeholder markers still present"; exit 1; } \
  || echo "OK: no <FILL / <SUBSTITUTE markers remain in either file."
```
- [ ] **Step 5: Run test to verify it passes**
Run: `mise run test:q -- tests/unit/decisions/test_adr083_vol3_money_scissors.py`
Expected: PASS (4 passed).
- [ ] **Step 6: Commit**
```bash
mise run commit -- "$(cat <<'EOF'
docs(adr): ADR089 vol3 money-scissors landed + index.yaml + state.yaml (U8.6, closes design spec 2026-07-18-vol3-money-scissors-design.md)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
EOF
)"
```
