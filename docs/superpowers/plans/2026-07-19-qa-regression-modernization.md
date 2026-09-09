# qa:regression Modernization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the byte-identical regression gate declared, sentinel-proved coverage; make the
Vol III financial layer visible to it (county scenarios + dense financial channels + the
no-dead-columns rule); make failures self-attributing; and give `defines_hash` and the
two-process determinism proof real teeth inside the gate.

**Architecture:** The gate's estate becomes *data* — `ScenarioCoverage` declarations beside the
scenario definitions, proved true by a dynamic probe (exact `step()` cadence) and proved
complete by a static sentinel (AST, layer-0.5, fast gate). Financial state reaches the harness
via the existing Feature-020 `persistent_context` channel (extended by one key). All
baseline-perturbing work funnels into ONE ceremony commit with a declared drift table.

**Tech Stack:** Python 3.11, Pydantic v2 (frozen models), pytest, mise tasks, the existing
`babylon.sentinels` static-AST family, `tools/regression_test.py` harness.

**Spec:** `docs/superpowers/specs/2026-07-19-qa-regression-modernization-design.md` (E1–E5).

## Global Constraints

- **Constitution III.7 untouched**: byte-identity stays the contract. Every task marked
  *baseline-neutral* MUST end with `mise run qa:regression` green and byte-identical — that run
  IS the task's neutrality proof. If bytes move in a neutral task, STOP; do not regenerate.
- **CI never touches the babylon-data drive** (owner ruling 2026-07-14). Every new scenario
  input is a committed fixture/artifact with documented provenance (D4 pattern; precedent:
  `tests/fixtures/vol3_fred_series.json`, `tests/fixtures/vol3_melt_national.json`).
- **Sentinel family rules** (ADR088): every new check ships a MUTATION test proving it reds on
  an injected violation, plus an infrastructure test proving missing/unparseable sources exit 2
  (`SentinelCheckError`), never a silent pass. Registry rows are frozen Pydantic with
  `extra="forbid"` and loud blank-field validators. Static checks read source via `ast` only —
  never import engine code (layer 0.5).
- **Exit-code contract** (`src/babylon/sentinels/base.py`): 0 clean / 1 gating violation /
  2 infrastructure failure, via `run_sensor(TAG, _GATING_CHECKS, _ADVISORY_CHECKS, _summary)`.
- **No `test_` prefix in production code** (use `check_`/`verify_`); RST docstrings on public
  API (Sphinx `-W`); MyPy strict; explicit return types; frozen Pydantic; type-ignores carry a
  code. Ruff B905 (`strict=True` on zip).
- **Never run pytest in parallel fan-outs**; scoped runs via `mise run test:q -- <path>`.
- **Commit per task** with `mise run commit -- "type(scope): msg"`; VERIFY HEAD MOVED after
  every commit (`git log --oneline -1`) — hooks abort silently. Commit trailer:
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`. Commitizen rejects `ceremony(...)`
  — the ceremony commit uses `test(baselines): ...`.
- **ADR numbering: next free is ADR090** (082/083 = null-play, 088/089 = sentinels/vol3).
- **Runtime budget**: local `mise run qa:regression` stays **< 5 min** — measured, not assumed,
  in Task 11.
- **Worktree venv shadow**: direct `poetry run python tools/...` invocations must be prefixed
  `PYTHONPATH="$PWD/src"` (worktrees share main's editable venv); `mise run` tasks set it.
- Line numbers cited below are exact as of branch base `f59d5852`; verify before editing —
  earlier tasks in this plan shift later ones.

## File Structure (decided here, binding for all tasks)

| Path | Responsibility |
|---|---|
| `tools/regression_scenarios.py` (NEW, Task 1–2) | Scenario definitions (`SCENARIOS`, `create_scenario`) + coverage data: `ScenarioCoverage`/`SystemEvidence`/`AtRestChannel`/`CoverageGap` models, `SCENARIO_COVERAGE_DATA` literal, `COVERAGE_GAPS_DATA` literal, `CHANNEL_WRITERS` literal |
| `src/babylon/sentinels/gate_coverage/` (NEW, Task 3) | Static, gating sentinel: union-covers-all-30-systems, names-exist, bundle-evidence checks (AST + committed-JSON reads only) |
| `tools/gate_coverage_probe.py` (NEW, Task 4) | Dynamic truth probe: runs in-memory scenarios at exact `step()` cadence, verifies each `SystemEvidence` row |
| `tools/regression_test.py` (MODIFIED, Tasks 5–10) | E4 attribution, E5a hash tooth, E3 financial channels + no-dead-columns, E5b determinism leg |
| `src/babylon/engine/simulation_engine.py` (MODIFIED, Task 7) | `_save/_restore_graph_context` gain the `national_financial` key |
| `src/babylon/engine/scenarios/single_county.py` (NEW, Task 8) | `create_single_county_scenario()` — Wayne-seeded in-memory county scenario |
| `tests/fixtures/single_county_wayne.json` (NEW, Task 8) | Committed Wayne 26163 tensor + distribution-source values (D4, provenance documented) |
| `src/babylon/engine/trace_format.py` (NEW, Task 9) | Shared deterministic trace serialization (`format_trace_value`, `trace_rows_to_csv_bytes`) used by BOTH the canonical harness and the headless runner |
| `src/babylon/engine/headless_runner/` (MODIFIED, Task 9) | Per-tick dense trace written into the bundle; `compare-bundle` gains the byte leg |
| `tests/unit/sentinels/test_gate_coverage.py` (NEW, Task 3) | invariant + mutation + infra tests |
| `tests/unit/tools/test_gate_coverage_probe.py` (NEW, Task 4) | probe truth + mutation tests |
| `tests/unit/tools/test_divergence_attribution.py` (NEW, Task 5) | E4 unit tests (synthetic traces) |
| `tests/unit/tools/test_defines_hash_gate.py` (NEW, Task 6) | E5a tooth tests |
| `tests/unit/tools/test_dead_columns.py` (NEW, Task 10) | E3 rule tests |
| `ai/decisions/ADR090_qa_regression_modernization.yaml` (NEW, Task 12) | Governance record |

**Phase → task map:** Phase A (E1) = Tasks 1–4 · Phase B (E4) = Task 5 · Phase C (E5a) =
Task 6 · Phase D (E2+E3+E5b + ceremony) = Tasks 7–11 · Governance/PR = Task 12.

**Phase D red-window note:** Tasks 8–10 intentionally leave `qa:regression` red locally
(headers widen before goldens regenerate). That is the ONLY sanctioned red window; Task 11's
ceremony closes it. Tasks 1–7 must each end green.

**Spec deviation, declared:** the spec's §6 places the E5a hash regeneration "in this
program's own ceremony commit" while also requiring every phase independently landable. Those
conflict: promoting the hash to gating while stored hashes are stale reds the gate until the
ceremony. Resolution (Task 6): the promotion commit is immediately followed, same task, by a
**hash-only refresh** of the five checkpoint JSONs with a machine-verified value-identity
proof (only `defines_hash` + `generated_at` change). This is honest (no tick value moves,
proof recorded in the commit message) and keeps every phase green; Task 11's full ceremony
supersedes these files anyway. The PR body discloses this deviation.

---

### Task 1: Extract `tools/regression_scenarios.py` (pure refactor, byte-neutral)

**Files:**
- Create: `tools/regression_scenarios.py`
- Modify: `tools/regression_test.py` (remove moved code, import instead)
- Test: existing suite + `mise run qa:regression` (neutrality proof)

**Interfaces:**
- Consumes: nothing new.
- Produces: `tools/regression_scenarios.py` exporting `SCENARIOS: Final[dict[str, dict[str, Any]]]`,
  `create_scenario(name: str) -> tuple[Any, Any, GameDefines]`. Tasks 2, 4, 8 build on this
  module; `regression_test.py` keeps re-exporting both names so external imports don't break.

- [ ] **Step 1: Create the module by MOVING (not copying) code**

Move from `tools/regression_test.py` into `tools/regression_scenarios.py`, verbatim, with
their existing comments:
- the `SCENARIOS` dict (currently lines 217–252)
- `create_scenario()` (currently lines 324–351)

Module skeleton:

```python
"""Canonical qa:regression scenario definitions and coverage declarations.

Extracted from ``tools/regression_test.py`` (this is its "successor module"
per the modernization spec §E1) so the scenario estate is importable data,
AST-readable by ``babylon.sentinels.gate_coverage``, and shared with the
coverage-truth probe without dragging in the whole harness.
"""

from __future__ import annotations

from typing import Any, Final

from babylon.config.defines import GameDefines
from babylon.engine.scenarios import (
    create_imperial_circuit_scenario,
    create_two_node_scenario,
)
from tools.shared import inject_parameter

# <-- SCENARIOS dict moved here verbatim -->
# <-- create_scenario() moved here verbatim -->
```

Check how `tools/regression_test.py` imports `inject_parameter` (from `tools.shared` vs
`shared`) and mirror it exactly — tools modules are executed as scripts AND imported by tests,
so the existing import idiom is the proven one.

- [ ] **Step 2: Update `tools/regression_test.py`**

Delete the moved definitions; add near the other imports:

```python
from tools.regression_scenarios import SCENARIOS, create_scenario  # noqa: F401  (re-export)
```

(Adjust idiom to match how the determinism unit test / other tests import from the tool — run
`rg -n "from regression_test import|import regression_test|from tools.regression_test" tests tools`
and keep every existing consumer working.)

- [ ] **Step 3: Prove neutrality**

Run: `mise run qa:regression`
Expected: PASS, 5/5 scenarios, byte-identical (only `defines_hash` WARNING lines may appear —
they are pre-existing).

Run: `mise run test:q -- tests/unit/tools`
Expected: PASS (all existing tool tests, incl. the two-process determinism test).

Run: `mise run check:quick`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
mise run commit -- "refactor(qa): extract scenario definitions into tools/regression_scenarios.py"
git log --oneline -1   # VERIFY HEAD MOVED
```

---

### Task 2: Coverage data model + declarations for the five scenarios

**Files:**
- Modify: `tools/regression_scenarios.py`
- Test: `tests/unit/tools/test_regression_scenarios.py` (NEW)

**Interfaces:**
- Consumes: Task 1's module.
- Produces (used by Tasks 3, 4, 5):
  - `class SystemEvidence(BaseModel)`: fields `system: str`, `kind: Literal["event",
    "entity_delta", "economy_delta", "state_presence", "context_presence", "bundle_event",
    "bundle_field"]`, `key: str`, `claim: str`
  - `class AtRestChannel(BaseModel)`: `channel: str`, `reason: str`
  - `class ScenarioCoverage(BaseModel)`: `scenario: str`, `layers: tuple[str, ...]`,
    `systems: tuple[SystemEvidence, ...]`, `at_rest: tuple[AtRestChannel, ...]`
  - `class CoverageGap(BaseModel)`: `system: str`, `reason: str`, `remediation: str`
  - `SCENARIO_COVERAGE_DATA: Final[tuple[dict[str, Any], ...]]` — a PURE LITERAL (the static
    sentinel `ast.literal_eval`s it)
  - `SCENARIO_COVERAGE: Final[tuple[ScenarioCoverage, ...]]` — validated instances
  - `COVERAGE_GAPS_DATA: Final[tuple[dict[str, str], ...]]` — pure literal
  - `COVERAGE_GAPS: Final[tuple[CoverageGap, ...]]`
  - `CHANNEL_WRITERS: Final[dict[str, tuple[str, ...]]]` — dense-column suffix → writing-system
    class names (pure literal; used by E4 attribution and validated by the sentinel)

- [ ] **Step 1: Write the failing test**

`tests/unit/tools/test_regression_scenarios.py`:

```python
"""Coverage-declaration data model: shape, validation, and literal-parseability."""

from __future__ import annotations

import ast
from pathlib import Path

import pytest

from tools.regression_scenarios import (
    COVERAGE_GAPS,
    SCENARIO_COVERAGE,
    SCENARIOS,
    AtRestChannel,
    CoverageGap,
    ScenarioCoverage,
    SystemEvidence,
)

pytestmark = pytest.mark.unit

_MODULE = Path(__file__).resolve().parents[3] / "tools" / "regression_scenarios.py"


def test_every_canonical_scenario_has_a_declaration() -> None:
    declared = {c.scenario for c in SCENARIO_COVERAGE}
    assert set(SCENARIOS) <= declared, sorted(set(SCENARIOS) - declared)


def test_declarations_reject_blank_fields() -> None:
    with pytest.raises(ValueError, match="claim"):
        SystemEvidence(system="VitalitySystem", kind="event", key="X", claim="")
    with pytest.raises(ValueError, match="reason"):
        AtRestChannel(channel="financial_endogenous_rate", reason=" ")
    with pytest.raises(ValueError, match="remediation"):
        CoverageGap(system="SubstrateSystem", reason="no hex nodes", remediation="")


def test_declarations_reject_unknown_fields() -> None:
    with pytest.raises(ValueError):
        ScenarioCoverage(scenario="x", layers=(), systems=(), at_rest=(), bogus=1)  # type: ignore[call-arg]


def test_coverage_data_literals_are_ast_parseable() -> None:
    """The static sentinel reads these via ast.literal_eval — prove it can."""
    tree = ast.parse(_MODULE.read_text(encoding="utf-8"))
    found = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            if node.target.id in {
                "SCENARIO_COVERAGE_DATA",
                "COVERAGE_GAPS_DATA",
                "CHANNEL_WRITERS",
            }:
                assert node.value is not None
                ast.literal_eval(node.value)  # raises if not a pure literal
                found.add(node.target.id)
    assert found == {"SCENARIO_COVERAGE_DATA", "COVERAGE_GAPS_DATA", "CHANNEL_WRITERS"}
```

Run: `mise run test:q -- tests/unit/tools/test_regression_scenarios.py`
Expected: FAIL with ImportError (models don't exist yet).

- [ ] **Step 2: Implement the models + declarations**

Append to `tools/regression_scenarios.py`:

```python
from pydantic import BaseModel, ConfigDict, model_validator
from typing import Literal


class _StrictModel(BaseModel):
    """Frozen, extra-forbidding base for coverage data (sentinel-family idiom)."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    @model_validator(mode="after")
    def _no_blank_strings(self) -> "_StrictModel":
        for name, value in self:
            if isinstance(value, str) and not value.strip():
                raise ValueError(f"{type(self).__name__}.{name} must not be blank")
        return self


class SystemEvidence(_StrictModel):
    """One checkable claim that a System exercises its logic in a scenario.

    :param system: Engine System class name (must appear in
        ``simulation_engine._SYSTEM_CLASSES`` — the sentinel enforces this).
    :param kind: How the probe verifies the claim.

        - ``event``: ``key`` is an ``EventType`` value observed in some tick's
          ``state.events``.
        - ``entity_delta``: ``key`` is ``"<entity_id>.<attr>"``; the attribute's
          value changes at least once across the run.
        - ``economy_delta``: ``key`` is an ``state.economy`` attribute that
          changes at least once across the run.
        - ``state_presence``: ``key`` is a ``WorldState`` field (e.g.
          ``market``) that is non-None by the final tick.
        - ``context_presence``: ``key`` is a ``persistent_context`` key (e.g.
          ``_tick_dynamics``) present by the final tick.
        - ``bundle_event`` / ``bundle_field``: verified statically against the
          committed e2e baseline bundle, not by the probe.
    :param key: The evidence key, interpreted per ``kind``.
    :param claim: Human sentence naming the material relation exercised.
    """

    system: str
    kind: Literal[
        "event",
        "entity_delta",
        "economy_delta",
        "state_presence",
        "context_presence",
        "bundle_event",
        "bundle_field",
    ]
    key: str
    claim: str


class AtRestChannel(_StrictModel):
    """A dense-trace channel declared legitimately all-zeros for a scenario."""

    channel: str
    reason: str


class ScenarioCoverage(_StrictModel):
    """What one canonical scenario demonstrably exercises."""

    scenario: str
    layers: tuple[str, ...]
    systems: tuple[SystemEvidence, ...]
    at_rest: tuple[AtRestChannel, ...] = ()


class CoverageGap(_StrictModel):
    """A system NO canonical scenario exercises — declared, reviewable debt.

    An uncovered system without a gap row is a gating sentinel failure; a gap
    row makes the hole loud and owner-reviewable instead of silent.
    """

    system: str
    reason: str
    remediation: str
```

Then the declarations. **These are the honest heart of E1 — the implementer must derive each
row from source, not invent it.** Method, per scenario: read each System's `step()` for its
observable writes/events; confirm against a live run (`PYTHONPATH="$PWD/src" poetry run python
tools/gate_coverage_probe.py` once Task 4 exists — for THIS task, confirm via targeted spot
checks below). Starting declarations (verified against source during scouting; the implementer
re-verifies each `key` before committing):

```python
# PURE LITERAL — the gate_coverage sentinel ast.literal_eval's this. No
# variables, no calls, no enum references. Validated into SCENARIO_COVERAGE
# below at import.
SCENARIO_COVERAGE_DATA: Final[tuple[dict[str, Any], ...]] = (
    {
        "scenario": "imperial_circuit",
        "layers": ("material_base", "consciousness", "survival", "contradiction"),
        "systems": (
            {"system": "VitalitySystem", "kind": "entity_delta", "key": "C001.wealth",
             "claim": "periphery worker wealth moves under subsistence drain"},
            {"system": "ProductionSystem", "kind": "economy_delta", "key": "imperial_rent_pool",
             "claim": "surplus extraction feeds the rent pool"},
            {"system": "ImperialRentSystem", "kind": "economy_delta", "key": "current_super_wage_rate",
             "claim": "rent pool distributes as super-wages"},
            {"system": "ConsciousnessSystem", "kind": "entity_delta", "key": "C001.class_consciousness",
             "claim": "consciousness drifts with material conditions"},
            {"system": "SurvivalSystem", "kind": "entity_delta", "key": "C001.p_revolution",
             "claim": "survival calculus updates rupture probability"},
            {"system": "ContradictionSystem", "kind": "entity_delta", "key": "C001.agitation",
             "claim": "tension accumulates as agitation"},
        ),
        "at_rest": (),
    },
    # ... one dict per remaining scenario: two_node, starvation, glut,
    # fascist_bifurcation — same shape. starvation MUST carry a death-path
    # evidence row (kind="event"); fascist_bifurcation MUST carry a
    # national_identity entity_delta row.
)

SCENARIO_COVERAGE: Final[tuple[ScenarioCoverage, ...]] = tuple(
    ScenarioCoverage(**d) for d in SCENARIO_COVERAGE_DATA
)

COVERAGE_GAPS_DATA: Final[tuple[dict[str, str], ...]] = (
    # Filled honestly by the implementer: every system of the 30 that NO
    # declaration (including detroit_tri_county's, Task 8's single_county)
    # evidences gets a row here. Expected members based on scenario anatomy
    # (verify, don't trust): SubstrateSystem (no hex nodes in any canonical
    # scenario), EpistemicHorizonSystem (observe-only shadow), and the
    # organization-dependent systems NOT proven by the detroit bundle.
    {"system": "SubstrateSystem",
     "reason": "no canonical scenario seeds hex nodes; substrate logic never fires",
     "remediation": "owner-gated nationwide scenario (task #49) or a hex-seeded canonical scenario"},
)

COVERAGE_GAPS: Final[tuple[CoverageGap, ...]] = tuple(
    CoverageGap(**d) for d in COVERAGE_GAPS_DATA
)

# Dense-column suffix -> System class names that may write it (E4 attribution;
# the sentinel proves every named system exists). PURE LITERAL.
CHANNEL_WRITERS: Final[dict[str, tuple[str, ...]]] = {
    "wealth": ("VitalitySystem", "ProductionSystem", "SurvivalSystem",
               "WealthDistributionSystem", "MarketScissorsSystem"),
    "effective_wealth": ("ProductionSystem", "ImperialRentSystem"),
    "p_acquiescence": ("SurvivalSystem",),
    "p_revolution": ("SurvivalSystem",),
    "active": ("LifecycleSystem", "DecompositionSystem"),
    "class_consciousness": ("ConsciousnessSystem",),
    "national_identity": ("ConsciousnessSystem", "FascistFactionSystem"),
    "agitation": ("ContradictionSystem", "StruggleSystem"),
    "organization": ("SolidaritySystem", "StruggleSystem"),
    "repression_faced": ("ControlRatioSystem", "StruggleSystem"),
    "value_flow": ("ProductionSystem", "ImperialRentSystem"),
    "tension": ("ContradictionSystem", "EdgeTransitionSystem"),
    "economy_imperial_rent_pool": ("ProductionSystem", "ImperialRentSystem"),
    "economy_current_super_wage_rate": ("ImperialRentSystem",),
    "economy_current_repression_level": ("ControlRatioSystem",),
}
```

The writer lists above are scouting-informed hypotheses — the implementer verifies each by
reading the named systems' `step()` methods (`rg -n "wealth" src/babylon/engine/systems/` and
`src/babylon/domain/`) and corrects them. A wrong writer list is exactly the silent rot the
sentinel exists to catch, so get it right once.

Spot-check commands for evidence keys (examples):

```bash
PYTHONPATH="$PWD/src" poetry run python - <<'EOF'
from tools.regression_scenarios import create_scenario
from babylon.engine.simulation_engine import step
state, cfg, defines = create_scenario("starvation")
ctx: dict = {}
for _ in range(20):
    state = step(state, cfg, ctx, defines)
    if state.events:
        print(state.tick, [e.type for e in state.events])
EOF
```

- [ ] **Step 3: Run tests**

Run: `mise run test:q -- tests/unit/tools/test_regression_scenarios.py`
Expected: PASS (all 4).

Run: `mise run qa:regression` → byte-identical (neutrality proof — this task adds data only).

- [ ] **Step 4: Commit**

```bash
mise run commit -- "feat(qa): ScenarioCoverage declarations for the five canonical scenarios (E1 data)"
git log --oneline -1
```

---

### Task 3: Static `gate_coverage` sentinel (gating)

**Files:**
- Create: `src/babylon/sentinels/gate_coverage/__init__.py`
- Create: `src/babylon/sentinels/gate_coverage/checks.py`
- Modify: `tools/sentinel_check.py` (dispatch entry `gate-coverage`)
- Modify: `.mise.toml` (`check:gate-coverage` task; add to `check:sentinels-static` depends)
- Modify: `.github/workflows/ci.yml` (step beside the `check:surface` step)
- Modify: `docs/reference/sentinel-error-classes.rst` (gate-blindness section: note the new
  instrument; NO new error class — this is the existing `gate-blindness` class applied to
  `qa:regression` itself)
- Test: `tests/unit/sentinels/test_gate_coverage.py`

**Interfaces:**
- Consumes: `SCENARIO_COVERAGE_DATA` / `COVERAGE_GAPS_DATA` / `CHANNEL_WRITERS` literals in
  `tools/regression_scenarios.py` (via `ast.literal_eval`, never import); the
  `_SYSTEM_CLASSES` tuple in `src/babylon/engine/simulation_engine.py` (via AST name-reads);
  `tests/baselines/detroit-tri-county-5t.json` (committed JSON, static read).
- Produces: `main(argv) -> int` registered as CLI key `gate-coverage`; check functions
  `check_union_covers_all_systems(...)`, `check_declared_names_exist(...)`,
  `check_bundle_evidence(...)` — each taking injectable path/registry args for tests.

- [ ] **Step 1: Write the failing tests**

`tests/unit/sentinels/test_gate_coverage.py`:

```python
"""gate_coverage sentinel: the gate's estate declaration is complete and true-by-name.

INVARIANT tests run the real repo; MUTATION tests inject a broken registry;
INFRA tests prove loud failure (exit 2 semantics via SentinelCheckError).
"""

from __future__ import annotations

from pathlib import Path

import pytest

from babylon.sentinels.base import SentinelCheckError
from babylon.sentinels.gate_coverage.checks import (
    check_bundle_evidence,
    check_declared_names_exist,
    check_union_covers_all_systems,
    engine_system_names,
)

pytestmark = pytest.mark.unit


def test_real_engine_has_thirty_systems() -> None:
    names = engine_system_names()
    assert len(names) == 30
    assert "MarketScissorsSystem" in names


def test_real_union_covers_all_systems() -> None:
    """Every one of the 30 systems is either evidenced or a declared gap."""
    assert check_union_covers_all_systems() == []


def test_real_declared_names_exist() -> None:
    assert check_declared_names_exist() == []


def test_real_bundle_evidence_holds() -> None:
    assert check_bundle_evidence() == []


def test_efficacy_reds_on_uncovered_system(tmp_path: Path) -> None:
    """MUTATION: a scenarios module declaring almost nothing must red."""
    module = tmp_path / "scenarios.py"
    module.write_text(
        "SCENARIO_COVERAGE_DATA = ("
        "{'scenario': 's', 'layers': (), 'systems': ("
        "{'system': 'VitalitySystem', 'kind': 'event', 'key': 'X', 'claim': 'c'},), "
        "'at_rest': ()},)\n"
        "COVERAGE_GAPS_DATA = ()\n"
        "CHANNEL_WRITERS = {}\n",
        encoding="utf-8",
    )
    findings = check_union_covers_all_systems(scenarios_path=module)
    assert findings, "29 uncovered systems must produce findings"
    assert any("MarketScissorsSystem" in f for f in findings)
    assert all("[gate-blindness]" in f for f in findings) or all("REMEDY" in f for f in findings)


def test_efficacy_reds_on_invented_system_name(tmp_path: Path) -> None:
    """MUTATION: a declaration naming a system that does not exist must red."""
    module = tmp_path / "scenarios.py"
    module.write_text(
        "SCENARIO_COVERAGE_DATA = ("
        "{'scenario': 's', 'layers': (), 'systems': ("
        "{'system': 'PhantomSystem', 'kind': 'event', 'key': 'X', 'claim': 'c'},), "
        "'at_rest': ()},)\n"
        "COVERAGE_GAPS_DATA = ()\n"
        "CHANNEL_WRITERS = {'wealth': ('AlsoPhantomSystem',)}\n",
        encoding="utf-8",
    )
    findings = check_declared_names_exist(scenarios_path=module)
    assert any("PhantomSystem" in f for f in findings)
    assert any("AlsoPhantomSystem" in f for f in findings)


def test_efficacy_reds_on_false_bundle_evidence(tmp_path: Path) -> None:
    """MUTATION: bundle evidence naming an event absent from the bundle reds."""
    module = tmp_path / "scenarios.py"
    module.write_text(
        "SCENARIO_COVERAGE_DATA = ("
        "{'scenario': 'detroit_tri_county', 'layers': (), 'systems': ("
        "{'system': 'OODASystem', 'kind': 'bundle_event', 'key': 'phantom_event_type', "
        "'claim': 'c'},), 'at_rest': ()},)\n"
        "COVERAGE_GAPS_DATA = ()\nCHANNEL_WRITERS = {}\n",
        encoding="utf-8",
    )
    findings = check_bundle_evidence(scenarios_path=module)
    assert any("phantom_event_type" in f for f in findings)


def test_infra_missing_module_is_loud(tmp_path: Path) -> None:
    with pytest.raises(SentinelCheckError):
        check_union_covers_all_systems(scenarios_path=tmp_path / "nope.py")
```

Run: `mise run test:q -- tests/unit/sentinels/test_gate_coverage.py`
Expected: FAIL — package doesn't exist.

- [ ] **Step 2: Implement the package**

`src/babylon/sentinels/gate_coverage/__init__.py`:

```python
"""Gate-coverage sentinel: qa:regression's estate is declared and complete.

The 2026-07-19 U9 inertness episode proved the byte-identical gate can run
green over a dead feature when no scenario exercises it. This sentinel is the
existing ``gate-blindness`` error class pointed at the gate itself: the
scenario estate (``tools/regression_scenarios.py``) DECLARES what it covers,
and this sensor proves the declaration set is complete over the engine's 30
Systems — statically, by AST, at layer 0.5. The companion dynamic probe
(``tools/gate_coverage_probe.py``, CLI key ``gate-coverage-truth``) proves
each declaration is TRUE at runtime.

**Scope — STATIC coherence only**: reads source files and committed baseline
JSON; NEVER imports the engine, NEVER runs a scenario.
"""

from babylon.sentinels.gate_coverage.checks import (
    check_bundle_evidence,
    check_declared_names_exist,
    check_union_covers_all_systems,
    engine_system_names,
    main,
)

__all__ = [
    "check_bundle_evidence",
    "check_declared_names_exist",
    "check_union_covers_all_systems",
    "engine_system_names",
    "main",
]
```

`src/babylon/sentinels/gate_coverage/checks.py` (complete implementation):

```python
"""Static checks: the qa:regression estate declaration is complete and well-named."""

from __future__ import annotations

import argparse
import ast
import json
from pathlib import Path
from typing import Any, Final

from babylon.sentinels.base import LabelledCheck, SentinelCheckError, run_sensor

_TAG: Final[str] = "GATE-COVERAGE"
_REPO_ROOT: Final[Path] = Path(__file__).resolve().parents[3]
_SCENARIOS_PATH: Final[Path] = _REPO_ROOT.parent / "tools" / "regression_scenarios.py"
_ENGINE_PATH: Final[Path] = _REPO_ROOT / "engine" / "simulation_engine.py"
_BUNDLE_BASELINE: Final[Path] = (
    _REPO_ROOT.parent.parent / "tests" / "baselines" / "detroit-tri-county-5t.json"
)
# NOTE for implementer: derive the three paths from the repo root the same way
# the sibling sentinels do (see coverage/checks.py's path idiom) and verify
# each resolves on disk in a unit test before relying on the parents[] count.


def _literal(path: Path, name: str) -> Any:
    """ast.literal_eval the module-level literal ``name`` in ``path``. Loud."""
    if not path.is_file():
        raise SentinelCheckError(f"source not found: {path}")
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except SyntaxError as exc:  # pragma: no cover - repo-corruption path
        raise SentinelCheckError(f"unparseable source {path}: {exc}") from exc
    for node in ast.walk(tree):
        target = None
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            target, value = node.target.id, node.value
        elif isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(
            node.targets[0], ast.Name
        ):
            target, value = node.targets[0].id, node.value
        if target == name:
            if value is None:
                raise SentinelCheckError(f"{name} in {path} has no value")
            try:
                return ast.literal_eval(value)
            except ValueError as exc:
                raise SentinelCheckError(
                    f"{name} in {path} is not a pure literal: {exc}"
                ) from exc
    raise SentinelCheckError(f"{name} not found in {path}")


def engine_system_names(engine_path: Path = _ENGINE_PATH) -> tuple[str, ...]:
    """The class names in ``_SYSTEM_CLASSES``, by AST (never imported)."""
    if not engine_path.is_file():
        raise SentinelCheckError(f"engine source not found: {engine_path}")
    tree = ast.parse(engine_path.read_text(encoding="utf-8"))
    for node in ast.walk(tree):
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
            if node.target.id == "_SYSTEM_CLASSES" and isinstance(node.value, ast.Call):
                # _SYSTEM_CLASSES: Final[...] = (A, B, ...) — value may be a
                # bare Tuple or a call like tuple(...); handle the Tuple form.
                pass
        if isinstance(node, (ast.Assign, ast.AnnAssign)):
            target = (
                node.target
                if isinstance(node, ast.AnnAssign)
                else node.targets[0]
                if len(node.targets) == 1
                else None
            )
            if isinstance(target, ast.Name) and target.id == "_SYSTEM_CLASSES":
                value = node.value
                if isinstance(value, ast.Tuple):
                    names = tuple(
                        elt.id for elt in value.elts if isinstance(elt, ast.Name)
                    )
                    if names:
                        return names
    raise SentinelCheckError(f"_SYSTEM_CLASSES tuple not found in {engine_path}")


def _declared_rows(scenarios_path: Path) -> tuple[Any, Any, Any]:
    coverage = _literal(scenarios_path, "SCENARIO_COVERAGE_DATA")
    gaps = _literal(scenarios_path, "COVERAGE_GAPS_DATA")
    writers = _literal(scenarios_path, "CHANNEL_WRITERS")
    return coverage, gaps, writers


def check_union_covers_all_systems(
    scenarios_path: Path = _SCENARIOS_PATH,
    engine_path: Path = _ENGINE_PATH,
) -> list[str]:
    """Every engine System is evidenced by some scenario OR a declared gap."""
    coverage, gaps, _ = _declared_rows(scenarios_path)
    engine = set(engine_system_names(engine_path))
    evidenced = {
        row["system"] for entry in coverage for row in entry.get("systems", ())
    }
    gapped = {row["system"] for row in gaps}
    findings: list[str] = []
    for system in sorted(engine - evidenced - gapped):
        findings.append(
            f"[gate-blindness] {system}: no canonical scenario evidences it and no "
            f"CoverageGap row declares the hole. REMEDY: add SystemEvidence to a "
            f"scenario in {scenarios_path.name}, or a CoverageGap row with reason "
            f"+ remediation."
        )
    for system in sorted(evidenced & gapped):
        findings.append(
            f"[gate-blindness] {system}: both evidenced and declared a gap — "
            f"stale gap row. REMEDY: delete the CoverageGap entry."
        )
    return findings


def check_declared_names_exist(
    scenarios_path: Path = _SCENARIOS_PATH,
    engine_path: Path = _ENGINE_PATH,
) -> list[str]:
    """Every system named anywhere in the declarations is a real engine System."""
    coverage, gaps, writers = _declared_rows(scenarios_path)
    engine = set(engine_system_names(engine_path))
    named: set[str] = set()
    named.update(row["system"] for entry in coverage for row in entry.get("systems", ()))
    named.update(row["system"] for row in gaps)
    for writer_list in writers.values():
        named.update(writer_list)
    return [
        f"[gate-blindness] declared system {name!r} does not exist in "
        f"_SYSTEM_CLASSES. REMEDY: fix the name or delete the row."
        for name in sorted(named - engine)
    ]


def check_bundle_evidence(
    scenarios_path: Path = _SCENARIOS_PATH,
    bundle_path: Path = _BUNDLE_BASELINE,
) -> list[str]:
    """bundle_event/bundle_field evidence rows hold against the committed baseline."""
    coverage, _, _ = _declared_rows(scenarios_path)
    rows = [
        row
        for entry in coverage
        for row in entry.get("systems", ())
        if row.get("kind") in ("bundle_event", "bundle_field")
    ]
    if not rows:
        return []
    if not bundle_path.is_file():
        raise SentinelCheckError(f"bundle baseline not found: {bundle_path}")
    bundle = json.loads(bundle_path.read_text(encoding="utf-8"))
    event_types = {e.get("event_type") or e.get("type") for e in bundle.get("events", [])}
    findings: list[str] = []
    for row in rows:
        if row["kind"] == "bundle_event" and row["key"] not in event_types:
            findings.append(
                f"[gate-blindness] {row['system']}: bundle_event {row['key']!r} not "
                f"present in {bundle_path.name}. REMEDY: fix the key or the claim."
            )
        if row["kind"] == "bundle_field":
            node: Any = bundle
            for part in row["key"].split("."):
                if not isinstance(node, dict) or part not in node:
                    findings.append(
                        f"[gate-blindness] {row['system']}: bundle_field "
                        f"{row['key']!r} not present in {bundle_path.name}. "
                        f"REMEDY: fix the dotted path."
                    )
                    break
                node = node[part]
    return findings


_GATING_CHECKS: Final[tuple[LabelledCheck, ...]] = (
    ("estate union covers all engine systems", check_union_covers_all_systems),
    ("declared system names exist", check_declared_names_exist),
    ("bundle evidence holds", check_bundle_evidence),
)
_ADVISORY_CHECKS: Final[tuple[LabelledCheck, ...]] = ()
# Gating like check:surface (owner ruling 2026-07-19 precedent), not advisory:
# an undeclared hole in the gate's own estate is the U9 failure mode.


def _summary(advisory_count: int) -> str:
    del advisory_count  # unused: no advisory tier for this sentinel
    return "Gate coverage: estate declaration complete and well-named."


def main(argv: list[str] | None = None) -> int:
    """CLI entry for the family dispatcher."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="CI mode (no-op alias)")
    parser.parse_args(argv)
    return run_sensor(_TAG, _GATING_CHECKS, _ADVISORY_CHECKS, _summary)
```

**Before finalizing:** read `src/babylon/sentinels/base.py` and one sibling (`surface/checks.py`)
and align exact signatures (`LabelledCheck` import name, `run_sensor` argument order, the
event-list key in the bundle JSON — inspect `tests/baselines/detroit-tri-county-5t.json`'s
`events` entries and use the real key name). Fix the path constants per the sibling idiom and
add a unit assertion that each resolves (`test_real_engine_has_thirty_systems` implicitly
covers `_ENGINE_PATH`).

- [ ] **Step 3: Register + wire**

In `tools/sentinel_check.py`: add to the eager imports
`from babylon.sentinels.gate_coverage.checks import main as gate_coverage_main` and to
`_SENSORS`: `"gate-coverage": gate_coverage_main,`.

In `.mise.toml`, after `check:surface`:

```toml
[tasks."check:gate-coverage"]
description = "Sentinel: qa:regression estate declaration is complete over all 30 engine systems (gate-blindness gate)"
run = "poetry run python tools/sentinel_check.py gate-coverage --check"
```

Add `"check:gate-coverage"` to `check:sentinels-static`'s `depends` list (`.mise.toml:170-172`)
and bump its description's sentinel count.

In `.github/workflows/ci.yml`, after the `Public-surface baseline gate` step (lines ~58-62):

```yaml
      - name: Gate-coverage sentinel (estate declaration)
        run: mise run check:gate-coverage
```

In `docs/reference/sentinel-error-classes.rst`, in the gate-blindness section, add a short
paragraph: the class now has a second instrument, `check:gate-coverage`, pointed at
`qa:regression`'s own scenario estate (declarations in `tools/regression_scenarios.py`).

In `tests/unit/sentinels/test_sentinel_family_cli.py`, add to the gating tests (mirroring
`test_surface_sensor_is_dispatchable_and_gates`):

```python
def test_gate_coverage_sensor_is_dispatchable_and_gates() -> None:
    """gate-coverage runs from the family CLI and reports clean on the repo."""
    result = _run("gate-coverage")
    assert result.returncode == 0, result.stderr
    assert "Gate coverage" in result.stdout
```

- [ ] **Step 4: Run tests — expect the union check to RED first**

Run: `mise run test:q -- tests/unit/sentinels/test_gate_coverage.py`

`test_real_union_covers_all_systems` will likely FAIL initially — that is the sentinel doing
its job: it names every system the five declarations + gaps don't reach. Resolve each named
system HONESTLY: add a real `SystemEvidence` row (verify by spot-run) where the scenario does
exercise it, else a `CoverageGap` row with a true reason. Do NOT invent evidence. Re-run until
green. This resolution loop happens in `tools/regression_scenarios.py` (Task 2's file) and is
expected to take several iterations — it is the point of the program.

Then: `mise run check:gate-coverage` → exit 0.
Then: `mise run qa:regression` → byte-identical (neutrality proof).
Then: `mise run check` → green.

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(sentinels): gate-coverage sentinel — qa:regression estate declared and complete (E1)"
git log --oneline -1
```

---

### Task 4: Dynamic coverage-truth probe

**Files:**
- Create: `tools/gate_coverage_probe.py`
- Modify: `tools/sentinel_check.py` (lazy wrapper, key `gate-coverage-truth`)
- Modify: `.mise.toml` (`check:gate-coverage-truth` task)
- Modify: `.github/workflows/ci.yml` (add to the job that runs `qa:regression`, NOT the fast
  sentinel step — this probe runs the engine)
- Test: `tests/unit/tools/test_gate_coverage_probe.py`

**Interfaces:**
- Consumes: `SCENARIO_COVERAGE`, `SCENARIOS`, `create_scenario` from
  `tools.regression_scenarios`; `step` from `babylon.engine.simulation_engine`.
- Produces: `run_probe(coverage=SCENARIO_COVERAGE, max_ticks=52) -> list[str]` (findings) and
  `main(argv) -> int`.

- [ ] **Step 1: Write the failing test**

`tests/unit/tools/test_gate_coverage_probe.py`:

```python
"""Coverage-truth probe: declared evidence is verified against a real run."""

from __future__ import annotations

import pytest

from tools.gate_coverage_probe import run_probe
from tools.regression_scenarios import ScenarioCoverage, SystemEvidence

pytestmark = pytest.mark.unit


def _cov(**kwargs) -> ScenarioCoverage:
    base = {"scenario": "two_node", "layers": ("material_base",), "systems": (), "at_rest": ()}
    base.update(kwargs)
    return ScenarioCoverage(**base)


def test_true_declaration_passes() -> None:
    cov = _cov(systems=(
        SystemEvidence(system="VitalitySystem", kind="entity_delta",
                       key="C001.wealth", claim="worker wealth moves"),
    ))
    assert run_probe(coverage=(cov,), max_ticks=10) == []


def test_efficacy_false_event_declaration_reds() -> None:
    """MUTATION: an event that never fires in two_node must produce a finding."""
    cov = _cov(systems=(
        SystemEvidence(system="OODASystem", kind="event",
                       key="organizational_action", claim="orgs act (they cannot: none seeded)"),
    ))
    findings = run_probe(coverage=(cov,), max_ticks=10)
    assert any("organizational_action" in f and "two_node" in f for f in findings)


def test_efficacy_false_delta_declaration_reds() -> None:
    """MUTATION: an entity attr that never changes must produce a finding."""
    cov = _cov(systems=(
        SystemEvidence(system="LifecycleSystem", kind="entity_delta",
                       key="C001.active", claim="worker deactivates (it does not in 5 ticks)"),
    ))
    findings = run_probe(coverage=(cov,), max_ticks=5)
    assert any("C001.active" in f for f in findings)


def test_unknown_scenario_is_loud() -> None:
    cov = _cov(scenario="no_such_scenario")
    with pytest.raises(KeyError):
        run_probe(coverage=(cov,), max_ticks=2)
```

Run: `mise run test:q -- tests/unit/tools/test_gate_coverage_probe.py`
Expected: FAIL — module missing.

- [ ] **Step 2: Implement the probe**

`tools/gate_coverage_probe.py`:

```python
"""Dynamic gate-coverage truth probe (CLI key: gate-coverage-truth).

Runs each declared in-memory scenario at the EXACT cadence the byte-identical
gate uses (``step()``: to_graph → run_tick → from_graph per tick, with the
harness-held ``persistent_context``) and verifies every ``SystemEvidence``
row: the declared observable actually occurs in that scenario's run. A false
declaration is a gating finding — the static sentinel proves the estate is
COMPLETE; this probe proves it is TRUE.

bundle_event/bundle_field rows are skipped here (verified statically against
the committed e2e baseline by ``babylon.sentinels.gate_coverage``).

Runs the engine — advisory-speed, wired into the qa CI job, NOT the static
fast gate.
"""

from __future__ import annotations

import argparse
import sys
from typing import Any, Final

from tools.regression_scenarios import (
    SCENARIO_COVERAGE,
    SCENARIOS,
    ScenarioCoverage,
    create_scenario,
)

_DEFAULT_MAX_TICKS: Final[int] = 52


def _entity_attr(state: Any, dotted: str) -> Any:
    entity_id, _, attr = dotted.partition(".")
    entity = state.entities.get(entity_id)
    if entity is None:
        return None
    if attr in ("class_consciousness", "national_identity", "agitation"):
        return getattr(entity.ideology, attr, None)
    return getattr(entity, attr, None)


def run_probe(
    coverage: tuple[ScenarioCoverage, ...] = SCENARIO_COVERAGE,
    max_ticks: int = _DEFAULT_MAX_TICKS,
) -> list[str]:
    """Verify every runtime-verifiable evidence row; return findings."""
    from babylon.engine.simulation_engine import step  # heavy import, local

    findings: list[str] = []
    for cov in coverage:
        rows = [r for r in cov.systems if r.kind not in ("bundle_event", "bundle_field")]
        if not rows:
            continue
        if cov.scenario not in SCENARIOS:
            raise KeyError(f"declared scenario {cov.scenario!r} not in SCENARIOS")
        state, sim_config, defines = create_scenario(cov.scenario)
        context: dict[str, Any] = {}
        seen_events: set[str] = set()
        initial = {r.key: _entity_attr(state, r.key) for r in rows if r.kind == "entity_delta"}
        initial_econ = {
            r.key: getattr(state.economy, r.key, None)
            for r in rows
            if r.kind == "economy_delta"
        }
        changed: set[str] = set()
        for _ in range(max_ticks):
            state = step(state, sim_config, context, defines)
            seen_events.update(str(e.type) for e in state.events)
            for r in rows:
                if r.kind == "entity_delta" and r.key not in changed:
                    if _entity_attr(state, r.key) != initial[r.key]:
                        changed.add(r.key)
                if r.kind == "economy_delta" and r.key not in changed:
                    if getattr(state.economy, r.key, None) != initial_econ[r.key]:
                        changed.add(r.key)
        for r in rows:
            ok = (
                (r.kind == "event" and r.key in seen_events)
                or (r.kind in ("entity_delta", "economy_delta") and r.key in changed)
                or (r.kind == "state_presence" and getattr(state, r.key, None) is not None)
                or (r.kind == "context_presence" and r.key in context)
            )
            if not ok:
                findings.append(
                    f"[gate-blindness] {cov.scenario}: {r.system} evidence "
                    f"({r.kind}: {r.key!r}) did NOT occur in {max_ticks} ticks — "
                    f"claim was: {r.claim}. REMEDY: fix the declaration or the "
                    f"scenario; do not weaken the evidence."
                )
    return findings


def main(argv: list[str] | None = None) -> int:
    """CLI entry for the family dispatcher (lazy-imported wrapper)."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="CI mode (no-op alias)")
    parser.add_argument("--max-ticks", type=int, default=_DEFAULT_MAX_TICKS)
    args = parser.parse_args(argv)
    findings = run_probe(max_ticks=args.max_ticks)
    for finding in findings:
        print(f"GATE-COVERAGE-TRUTH VIOLATION: {finding}", file=sys.stderr)
    if findings:
        return 1
    print(f"Gate coverage truth: all declared evidence verified over {len(SCENARIO_COVERAGE)} scenarios.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
```

Match the event-type comparison to reality: inspect `SimulationEvent.type`'s runtime type
(`EventType` StrEnum vs str) with one spot-run and align `str(e.type)` vs `e.type.value`.

- [ ] **Step 3: Register + wire**

`tools/sentinel_check.py` — add a lazy wrapper beside `_aggregation_main` (same idiom, with a
docstring explaining WHY lazy: imports the engine):

```python
def _gate_coverage_truth_main(argv: list[str] | None) -> int:
    """Lazy wrapper: the truth probe imports the engine (layer > 0.5)."""
    from tools.gate_coverage_probe import main as probe_main

    return probe_main(argv)
```

and `"gate-coverage-truth": _gate_coverage_truth_main,` in `_SENSORS`.

`.mise.toml`:

```toml
[tasks."check:gate-coverage-truth"]
description = "Dynamic gate-coverage truth probe: every declared SystemEvidence actually occurs in its scenario (runs the engine, ~one gate-length)"
run = "poetry run python tools/sentinel_check.py gate-coverage-truth --check"
```

`ci.yml`: add `mise run check:gate-coverage-truth` as a step in the same job/step-sequence
that runs `qa:regression` (find it with `rg -n "qa:regression" .github/workflows/`).

- [ ] **Step 4: Run everything**

Run: `mise run test:q -- tests/unit/tools/test_gate_coverage_probe.py` → PASS.
Run: `mise run check:gate-coverage-truth` → exit 0; if any real declaration fails, fix the
DECLARATION in `tools/regression_scenarios.py` honestly (this is the Task 3 Step 4 loop's
runtime twin).
Run: `mise run qa:regression` → byte-identical. `mise run check` → green.

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(qa): dynamic gate-coverage truth probe (E1 check-a)"
git log --oneline -1
```

---

### Task 5: E4 — first-divergence attribution

**Files:**
- Modify: `tools/regression_test.py` (`_first_dense_divergence` region, lines ~545-582; the
  dense-FAIL reporting path in `compare_dense_trace`/`compare_all_baselines`)
- Test: `tests/unit/tools/test_divergence_attribution.py`

**Interfaces:**
- Consumes: `CHANNEL_WRITERS` from `tools.regression_scenarios`.
- Produces:
  - `@dataclass(frozen=True) DivergenceReport`: `scenario: str`, `tick: int`, `column: str`,
    `channel: str`, `county: str | None`, `expected: str`, `actual: str`,
    `magnitude: float | None`, `last_agreeing_tick: int | None`,
    `candidate_systems: tuple[str, ...]`
  - `attribute_divergence(scenario, header, expected_rows, actual_rows) -> DivergenceReport | None`
  - `divergence_report_json(report) -> dict[str, Any]`
  - On dense FAIL: human-readable one-liner printed + machine-readable JSON written to
    `reports/qa-first-divergence.json` (list — one entry per failing scenario per run;
    truncated/rewritten each compare run).

- [ ] **Step 1: Write the failing tests**

`tests/unit/tools/test_divergence_attribution.py`:

```python
"""E4: on dense divergence the tool attributes (tick, system, channel, county)."""

from __future__ import annotations

import pytest

from tools.regression_test import DivergenceReport, attribute_divergence

pytestmark = pytest.mark.unit

_HEADER = ["tick", "economy_imperial_rent_pool", "C001_wealth", "county_26163_interest",
           "edge_C001_C002_tension"]


def _rows(*rows: list[str]) -> list[list[str]]:
    return [list(r) for r in rows]


def test_no_divergence_returns_none() -> None:
    rows = _rows(["0", "1.0", "2.0", "3.0", "0.1"], ["1", "1.1", "2.1", "3.1", "0.2"])
    assert attribute_divergence("s", _HEADER, rows, rows) is None


def test_first_divergence_is_attributed() -> None:
    expected = _rows(["0", "1.0", "2.0", "0.0", "0.1"], ["1", "1.1", "2.5", "0.0", "0.2"])
    actual = _rows(["0", "1.0", "2.0", "0.0", "0.1"], ["1", "1.1", "2.9", "0.0", "0.2"])
    report = attribute_divergence("imperial_circuit", _HEADER, expected, actual)
    assert report is not None
    assert report.tick == 1
    assert report.column == "C001_wealth"
    assert report.channel == "wealth"
    assert report.county is None
    assert report.magnitude == pytest.approx(0.4)
    assert report.last_agreeing_tick == 0
    assert "VitalitySystem" in report.candidate_systems


def test_county_column_yields_county() -> None:
    expected = _rows(["0", "1", "2", "0.0", "0.1"], ["1", "1", "2", "5.0", "0.1"])
    actual = _rows(["0", "1", "2", "0.0", "0.1"], ["1", "1", "2", "6.0", "0.1"])
    report = attribute_divergence("single_county", _HEADER, expected, actual)
    assert report is not None
    assert report.county == "26163"
    assert report.channel == "interest"


def test_row_count_mismatch_attributes_the_missing_tick() -> None:
    expected = _rows(["0", "1", "2", "0", "0"], ["1", "1", "2", "0", "0"])
    actual = _rows(["0", "1", "2", "0", "0"])
    report = attribute_divergence("s", _HEADER, expected, actual)
    assert report is not None
    assert report.tick == 1
    assert report.channel == "<missing row>"


def test_non_numeric_divergence_has_none_magnitude() -> None:
    expected = _rows(["0", "1", "2", "0", "0"], ["1", "1", "True", "0", "0"])
    actual = _rows(["0", "1", "2", "0", "0"], ["1", "1", "False", "0", "0"])
    report = attribute_divergence("s", _HEADER, expected, actual)
    assert report is not None
    assert report.magnitude is None
```

Run: `mise run test:q -- tests/unit/tools/test_divergence_attribution.py` → FAIL (names
missing).

- [ ] **Step 2: Implement**

In `tools/regression_test.py` (near `_first_dense_divergence`, reusing its cell-walk logic —
refactor the shared walk into one place rather than duplicating; `_first_dense_divergence`'s
existing callers keep working or are migrated to the new function):

```python
@dataclass(frozen=True)
class DivergenceReport:
    """First point where an actual dense trace departs its golden baseline."""

    scenario: str
    tick: int
    column: str
    channel: str
    county: str | None
    expected: str
    actual: str
    magnitude: float | None
    last_agreeing_tick: int | None
    candidate_systems: tuple[str, ...]


def _parse_column(column: str) -> tuple[str, str | None]:
    """Return (channel suffix, county fips or None) for a dense column name."""
    if column.startswith("county_"):
        parts = column.split("_", 2)  # county, <fips>, <suffix>
        if len(parts) == 3:
            return parts[2], parts[1]
    if column.startswith(("economy_", "financial_")):
        return column, None
    if column.startswith("edge_"):
        return column.rsplit("_", 1)[1], None
    _, _, suffix = column.partition("_")  # C001_wealth -> wealth
    return suffix or column, None


def attribute_divergence(
    scenario: str,
    header: list[str],
    expected_rows: list[list[str]],
    actual_rows: list[list[str]],
) -> DivergenceReport | None:
    """Locate and attribute the first cell where actual departs expected."""
    from tools.regression_scenarios import CHANNEL_WRITERS

    n = min(len(expected_rows), len(actual_rows))
    for i in range(n):
        exp_row, act_row = expected_rows[i], actual_rows[i]
        for j, column in enumerate(header):
            exp = exp_row[j] if j < len(exp_row) else "<absent>"
            act = act_row[j] if j < len(act_row) else "<absent>"
            if exp == act:
                continue
            channel, county = _parse_column(column)
            try:
                magnitude: float | None = abs(float(act) - float(exp))
            except ValueError:
                magnitude = None
            return DivergenceReport(
                scenario=scenario,
                tick=int(exp_row[0]),
                column=column,
                channel=channel,
                county=county,
                expected=exp,
                actual=act,
                magnitude=magnitude,
                last_agreeing_tick=int(expected_rows[i - 1][0]) if i > 0 else None,
                candidate_systems=CHANNEL_WRITERS.get(channel, ()),
            )
    if len(expected_rows) != len(actual_rows):
        i = n
        longer = expected_rows if len(expected_rows) > n else actual_rows
        return DivergenceReport(
            scenario=scenario,
            tick=int(longer[i][0]),
            column="<row count>",
            channel="<missing row>",
            county=None,
            expected=str(len(expected_rows)),
            actual=str(len(actual_rows)),
            magnitude=None,
            last_agreeing_tick=int(expected_rows[n - 1][0]) if n else None,
            candidate_systems=(),
        )
    return None
```

Wire into the dense-FAIL path (`compare_dense_trace` currently calls
`_first_dense_divergence` for its diagnostic string — replace that diagnostic with
`attribute_divergence` output): on mismatch, parse both byte blobs back through `csv.reader`,
build the report, print

```python
print(
    f"  FIRST DIVERGENCE: tick {r.tick}, {r.column}"
    + (f" (county {r.county})" if r.county else "")
    + f": {r.expected} -> {r.actual}"
    + (f" (Δ={r.magnitude!r})" if r.magnitude is not None else "")
    + f"; last agreed tick {r.last_agreeing_tick}; "
    + f"candidate systems: {', '.join(r.candidate_systems) or 'unmapped channel'}"
)
```

and append `dataclasses.asdict(r)` to a module-level list flushed by `compare_all_baselines`
to `reports/qa-first-divergence.json` (`json.dumps(..., indent=2)`, parent mkdir, only when
non-empty; remove any stale file at compare start so a green run leaves no misleading report).

- [ ] **Step 3: Run tests + neutrality**

Run: `mise run test:q -- tests/unit/tools/test_divergence_attribution.py` → PASS.
Run: `mise run test:q -- tests/unit/tools` → PASS (old `_first_dense_divergence` tests may
need migrating to the new attribution output — migrate, don't delete coverage).
Run: `mise run qa:regression` → byte-identical (attribution only fires on FAIL).

- [ ] **Step 4: Commit**

```bash
mise run commit -- "feat(qa): first-divergence attribution on dense-gate failure (E4)"
git log --oneline -1
```

---

### Task 6: E5a — defines_hash becomes a gating leg (+ value-identical hash refresh)

**Files:**
- Modify: `tools/regression_test.py:918-929` (WARNING block) — the `:944` pass filter stays
  (other WARNINGs remain advisory)
- Modify: `tests/baselines/*.json` × 5 (hash refresh, value-identical)
- Test: `tests/unit/tools/test_defines_hash_gate.py`

- [ ] **Step 1: Write the failing test**

`tests/unit/tools/test_defines_hash_gate.py`:

```python
"""E5a: a defines_hash mismatch FAILS the gate and names the ceremony."""

from __future__ import annotations

import dataclasses

import pytest

from tools.regression_test import BaselineData, compare_baselines

pytestmark = pytest.mark.unit


def _baseline(defines_hash: str) -> BaselineData:
    return BaselineData(
        scenario="s", description="d", generated_at="2026-01-01T00:00:00+00:00",
        defines_hash=defines_hash, max_ticks=1, checkpoints=[], final_outcome="SURVIVED",
        ticks_survived=1,
    )


def test_hash_mismatch_fails_and_names_the_ceremony() -> None:
    passed, diffs = compare_baselines(_baseline("aaaa"), _baseline("bbbb"))
    assert passed is False
    joined = "\n".join(diffs)
    assert "defines_hash" in joined
    assert "qa:regression-generate-dense" in joined


def test_hash_match_passes() -> None:
    passed, _ = compare_baselines(_baseline("aaaa"), _baseline("aaaa"))
    assert passed is True
```

(Adapt the `_baseline` constructor to `BaselineData`'s real signature and to how
`compare_baselines` handles empty checkpoint lists — if it requires ≥1 checkpoint, give both
sides one identical `CheckpointData`.)

Run: `mise run test:q -- tests/unit/tools/test_defines_hash_gate.py`
Expected: `test_hash_mismatch_fails_and_names_the_ceremony` FAILS (passes today — WARNING).

- [ ] **Step 2: Implement the tooth**

Replace `tools/regression_test.py:918-929` (both WARNING lines + the continuation comment)
with:

```python
    # E5a (modernization program): defines_hash gates. A GameDefines change
    # without a baseline ceremony is exactly the silent-drift the gate exists
    # to catch; the five stale hashes that motivated this were refreshed
    # (value-identical) in the same commit that armed this tooth.
    if expected.defines_hash != actual.defines_hash:
        diffs.append(
            f"defines_hash mismatch ({expected.defines_hash} -> {actual.defines_hash}): "
            f"GameDefines changed without a baseline ceremony. If intentional, run "
            f"'mise run qa:regression-generate-dense' and commit the regenerated "
            f"baselines with a declared drift table (test(baselines): ...)."
        )
```

Search for tests pinning the old WARNING behavior (`rg -n "WARNING" tests/unit/tools/`) and
invert them (they now assert failure), keeping their scenario coverage.

- [ ] **Step 3: Refresh the five stale hashes, value-identically**

```bash
git stash list  # ensure clean tree first
mise run qa:regression-generate     # JSON only, no --dense: dense CSVs untouched
# Value-identity proof: only defines_hash + generated_at may differ.
for s in imperial_circuit two_node starvation glut fascist_bifurcation; do
  python3 - "$s" <<'EOF'
import json, subprocess, sys
name = sys.argv[1]
new = json.load(open(f"tests/baselines/{name}.json"))
old = json.loads(subprocess.run(
    ["git", "show", f"HEAD:tests/baselines/{name}.json"],
    capture_output=True, text=True, check=True).stdout)
for d in (new, old):
    d.pop("defines_hash"), d.pop("generated_at")
assert new == old, f"{name}: VALUES MOVED — STOP, do not commit"
print(f"{name}: value-identical ✓")
EOF
done
git diff --stat tests/baselines/   # expect exactly 5 files, small diffs
```

If ANY scenario reports VALUES MOVED: STOP. That means dev's engine and the stored checkpoints
disagree beyond the hash — a real regression or an unceremonied drift. Surface it to the
owner; do not regenerate around it.

- [ ] **Step 4: Verify + run the full gate**

Run: `mise run test:q -- tests/unit/tools/test_defines_hash_gate.py` → PASS.
Run: `mise run qa:regression` → PASS, **no defines_hash output at all** (hashes now current).
Run: `mise run test:q -- tests/unit/tools` → PASS.

- [ ] **Step 5: Commit (one commit: tooth + refresh + proof in the message)**

```bash
mise run commit -- "test(baselines): arm the defines_hash gate; refresh 5 stale hashes value-identically (E5a)"
git log --oneline -1
```

Commit body MUST include the five `value-identical ✓` proof lines and the old→new hash pairs
(the declared drift table for this mini-ceremony).

---

### Task 7: Round-trip `national_financial` through `persistent_context`

**Files:**
- Modify: `src/babylon/engine/simulation_engine.py:428-465`
  (`_restore_graph_context` / `_save_graph_context`)
- Test: `tests/unit/engine/test_graph_context_financial.py` (NEW)

**Interfaces:**
- Consumes: `NATIONAL_FINANCIAL_ATTR` (= `"national_financial"`) from
  `babylon.domain.economics.tick.graph_bridge`.
- Produces: `persistent_context["_national_financial"]` — the harness (Task 9's channels) and
  the probe (`context_presence` evidence) read it.

- [ ] **Step 1: Write the failing test**

`tests/unit/engine/test_graph_context_financial.py`:

```python
"""Feature-020 extension: national_financial survives the WorldState round-trip."""

from __future__ import annotations

import pytest

from babylon.engine.simulation_engine import _restore_graph_context, _save_graph_context
from babylon.topology import BabylonGraph

pytestmark = pytest.mark.unit


def test_national_financial_saves_and_restores() -> None:
    graph = BabylonGraph()
    payload = {"endogenous_interest": {"rate": 0.019855, "profit_rate_ceiling": 0.0597}}
    graph.graph["national_financial"] = payload
    context: dict = {}
    _save_graph_context(graph, context, tick=1)
    assert context["_national_financial"] == payload

    fresh = BabylonGraph()
    _restore_graph_context(fresh, context)
    assert fresh.graph["national_financial"] == payload


def test_absent_attr_is_not_invented() -> None:
    graph = BabylonGraph()
    context: dict = {}
    _save_graph_context(graph, context, tick=1)
    assert "_national_financial" not in context
    fresh = BabylonGraph()
    _restore_graph_context(fresh, context)
    assert "national_financial" not in fresh.graph
```

(Adjust `graph.graph[...]` writes to `BabylonGraph`'s real graph-attr API —
`set_graph_attr`/`graph` property — check `src/babylon/topology/` and match; also check
whether `_save_graph_context`'s existing guard `"tick_dynamics" not in G.graph` early-returns
before a financial-only save — restructure the guard so each key saves independently.)

Run: `mise run test:q -- tests/unit/engine/test_graph_context_financial.py` → FAIL.

- [ ] **Step 2: Implement**

In `_restore_graph_context` (after the `_tick_dynamics` block):

```python
    if "_national_financial" in persistent_context:
        G.graph["national_financial"] = persistent_context["_national_financial"]
```

In `_save_graph_context`, restructure so the early-return no longer swallows the new key:

```python
def _save_graph_context(
    G: BabylonGraph,
    persistent_context: dict[str, Any] | None,
    tick: int,
) -> None:
    """Save tick_dynamics + national_financial into persistent_context after systems run.

    Feature 020 (+ qa-modernization E3-pre): persists graph-level economic
    state so it survives the WorldState round-trip. Without this, the Vol III
    financial layer was tick-internal only — invisible to the harness, the
    coverage probe, and every persisted session (SessionRecorder snapshots
    to_graph() output, which lacked it).
    """
    if persistent_context is None:
        return
    if "national_financial" in G.graph:
        persistent_context["_national_financial"] = G.graph["national_financial"]
    if "tick_dynamics" not in G.graph:
        return
    tick_dynamics_data = G.graph["tick_dynamics"]
    persistent_context["_tick_dynamics"] = tick_dynamics_data
    if tick % 52 == 0:
        snapshots: list[dict[str, Any]] = persistent_context.setdefault(
            "_tick_dynamics_snapshots", []
        )
        snapshots.append(tick_dynamics_data)
```

Use the string literals the existing function uses (it uses `"tick_dynamics"` directly —
match that; do NOT import `graph_bridge` if the existing code doesn't, to keep the layering
exactly as-is).

- [ ] **Step 3: Neutrality proof (critical)**

Run: `mise run test:q -- tests/unit/engine/test_graph_context_financial.py` → PASS.
Run: `mise run qa:regression` → **byte-identical**. The five scenarios never stamp
`national_financial` (no county tensors), so restore never fires — the gate run IS the proof.
If bytes move: STOP, the assumption is wrong; investigate which system now sees restored
state, report before proceeding.
Run: `mise run test:q -- tests/unit/engine` → PASS.

- [ ] **Step 4: Commit**

```bash
mise run commit -- "feat(engine): national_financial survives ticks via persistent_context (E3-pre)"
git log --oneline -1
```

---

### Task 8: `single_county` scenario (Wayne, in-memory, committed fixture)

**Files:**
- Create: `src/babylon/engine/scenarios/single_county.py`
- Modify: `src/babylon/engine/scenarios/__init__.py` (export)
- Create: `tests/fixtures/single_county_wayne.json`
- Modify: `tools/regression_scenarios.py` (SCENARIOS entry + dispatch + coverage declaration)
- Modify: `tools/regression_test.py` (`_build_vol3_calculator_overrides` gains the
  scenario-conditional Wayne services)
- Test: `tests/unit/engine/scenarios/test_single_county.py` (NEW)

**Interfaces:**
- Consumes: `SocialClass`, `Territory`, `Relationship`, `WorldState`, `GameDefines` (mirror
  `_legacy.py`'s constructions); `TensorRegistry`, `ValueTensor4x3`,
  `SurplusDistributionCalculator` + the three source Protocols
  (`src/babylon/domain/economics/distribution/data_sources.py`).
- Produces: `create_single_county_scenario() -> tuple[WorldState, SimulationConfig, GameDefines]`
  (same triple contract as the other factories); scenario name `"single_county"`;
  `build_single_county_overrides(defines) -> dict[str, Any]` in the harness returning
  calculator_overrides that include `tensor_registry` + `distribution_calculator` (+ reusing
  the existing melt/FRED fixture services).

- [ ] **Step 1: The committed fixture, with provenance**

`tests/fixtures/single_county_wayne.json` — REAL Wayne County 2011 values from the reference
tensors (D4: committed, not a drive read). The Vol III program's delta report pinned these
(reports/vol3-baseline-delta.md): `profit_rate=0.0597`, endogenous rate `i=0.019855`,
`interest_payments=1.179e9` (33.3% of surplus). The implementer extracts the exact tensor
values with the reference DB available locally:

```bash
mise run db:sql -- "SELECT 1"   # sanity: DB reachable (dev box only)
PYTHONPATH="$PWD/src" poetry run python - <<'EOF'
# Extraction script: pull Wayne 26163 / 2011 realized tensor + distribution
# source values via the SAME loaders production uses, print JSON to paste
# into the fixture. Find the loader entry points with:
#   rg -n "class TensorRegistry|def get\(" src/babylon/domain/economics/ | head
# and the distribution sources' real implementations with:
#   rg -rn "RentalIncomeSource|TaxOnSurplusSource|InterestIncomeSource" src/ | head
EOF
```

Fixture shape (keys consumed by Step 3's builder — exact values from the extraction):

```json
{
  "_provenance": "Wayne County MI (FIPS 26163), year 2011. Extracted 2026-07-19 from data/sqlite/marxist-data-3NF.sqlite realized reference tensors + distribution sources via <script>; values byte-frozen here per D4 (CI never reads the drive). Pins match reports/vol3-baseline-delta.md.",
  "fips": "26163",
  "year": 2011,
  "tensor": {"total_s": 0.0, "total_v": 0.0, "total_c": 0.0, "profit_rate": 0.0597},
  "rental_income": 0.0,
  "corporate_tax": 0.0,
  "national_net_interest": 0.0
}
```

(The `0.0`s above are placeholders IN THE PLAN ONLY — the fixture on disk must carry the real
extracted numbers; a fixture with zeros would put the financial channels at rest and Task 10's
no-dead-columns rule would red it, by design. The `tensor` dict's keys must match
`ValueTensor4x3`'s real constructor — read the model first.)

- [ ] **Step 2: Write the failing scenario test**

`tests/unit/engine/scenarios/test_single_county.py`:

```python
"""single_county: the smallest graph where the Vol III financial layer fires."""

from __future__ import annotations

import pytest

from babylon.engine.scenarios import create_single_county_scenario

pytestmark = pytest.mark.unit


def test_scenario_carries_a_real_county_fips() -> None:
    state, _config, _defines = create_single_county_scenario()
    fips = [t.county_fips for t in state.territories.values() if t.county_fips]
    assert fips == ["26163"]


def test_scenario_is_deterministic() -> None:
    a = create_single_county_scenario()[0]
    b = create_single_county_scenario()[0]
    assert a.model_dump() == b.model_dump()


def test_financial_layer_fires_through_the_production_path() -> None:
    """After stepping with the Wayne overrides, county interest is nonzero.

    This is the U9-inertness detector in miniature: the interest number must
    come out of TickDynamicsSystem's real calculator chain (tensor_registry ->
    SurplusDistributionCalculator), not a stamped fixture.
    """
    from babylon.engine.simulation_engine import step
    from tools.regression_test import build_single_county_overrides

    state, config, defines = create_single_county_scenario()
    overrides = build_single_county_overrides(defines)
    context: dict = {}
    for _ in range(3):
        state = step(state, config, context, defines, calculator_overrides=overrides)
    tick_dynamics = context.get("_tick_dynamics")
    assert tick_dynamics, "TickDynamicsSystem never stamped county state"
    county_states = tick_dynamics.get("county_states", {})
    assert "26163" in county_states
    distribution = county_states["26163"].surplus_distribution
    assert distribution is not None
    assert distribution.interest_payments > 0.0
    financial = context.get("_national_financial")
    assert financial, "national_financial never persisted (Task 7 regression)"
    assert financial["endogenous_interest"]["rate"] > 0.0
```

Run: `mise run test:q -- tests/unit/engine/scenarios/test_single_county.py` → FAIL.

- [ ] **Step 3: Implement the factory + overrides builder**

`src/babylon/engine/scenarios/single_county.py` — mirror `create_two_node_scenario`'s
structure (`_legacy.py:41-193`) exactly, with: 2 social classes (`W001` worker,
`O001` owner — county-stamped: `county_fips="26163"` if `SocialClass` declares that field —
**verify with `rg -n "county_fips" src/babylon/models/entities/social_class.py`; if it does
NOT declare it, do NOT stamp it** (vocabulary sentinel Rule c); county membership then rides
only on the Territory + TENANCY edges), 1 territory
(`Territory(id="26163", name="Wayne County", county_fips="26163", biocapacity=100.0, ...)`),
3 edges (EXPLOITATION W001→O001, WAGES O001→W001, TENANCY W001→26163). Docstring: RST, names
the material relation (Wayne as the imperial-core county where the financial layer's
distribution identity `s = p + i + r + t` is exercised).

In `tools/regression_test.py`:

```python
def build_single_county_overrides(defines: GameDefines) -> dict[str, Any]:
    """Wayne-county calculator_overrides from the committed fixture (D4).

    Extends the Vol III override set with a real-FIPS ``tensor_registry`` and
    a ``distribution_calculator`` whose sources replay the committed Wayne
    2011 values — the production calculator chain runs for real; only the
    LEAF data inputs are fixture-fed (mocking-is-debt boundary).
    """
    overrides = _build_vol3_calculator_overrides(defines)
    fixture = json.loads(
        (Path(__file__).parent.parent / "tests" / "fixtures" / "single_county_wayne.json")
        .read_text(encoding="utf-8")
    )
    # Implementer: construct TensorRegistry + ValueTensor4x3 + the three
    # fixture-backed source objects satisfying the Protocols in
    # src/babylon/domain/economics/distribution/data_sources.py, then
    # SurplusDistributionCalculator(...). Read the Protocols and the
    # calculator's __init__ FIRST; reuse existing fixture-source classes if
    # any exist (rg -n "class.*IncomeSource|class.*TaxSource" src/ tests/).
    ...
    return overrides
```

and in `_run_scenario_ticks` / `create_scenario`'s caller path, select the overrides builder
per scenario:

```python
    calculator_overrides = (
        build_single_county_overrides(defines)
        if name == "single_county"
        else _build_vol3_calculator_overrides(defines)
    )
```

In `tools/regression_scenarios.py`: add the SCENARIOS entry

```python
    "single_county": {
        "description": "Wayne-seeded minimal county: Vol III financial layer, MELT path, and distribution identity all fire",
        "factory": "create_single_county_scenario",
        "defines_overrides": {},
    },
```

extend `create_scenario`'s dispatch with the new factory, and add the coverage declaration
(financial evidence rows are MANDATORY — this scenario exists to cover them):

```python
    {
        "scenario": "single_county",
        "layers": ("material_base", "financial", "market"),
        "systems": (
            {"system": "TickDynamicsSystem", "kind": "context_presence", "key": "_tick_dynamics",
             "claim": "county economic state computed from realized Wayne tensors"},
            {"system": "TickDynamicsSystem", "kind": "context_presence", "key": "_national_financial",
             "claim": "endogenous interest rate (Vol III Part V) computed and published"},
            {"system": "MarketScissorsSystem", "kind": "state_presence", "key": "market",
             "claim": "price-value axis live over a county-bearing graph"},
        ),
        "at_rest": (),
    },
```

(Verify the `market` presence claim against a spot-run; if MarketScissors needs more than one
county's data to stamp `market`, replace with an evidence row that IS true — honesty over
convenience.)

- [ ] **Step 4: Run tests**

Run: `mise run test:q -- tests/unit/engine/scenarios/test_single_county.py` → PASS.
Run: `mise run check:gate-coverage` → PASS (single_county's declaration references real
systems). `mise run check:gate-coverage-truth` → PASS (the probe now runs single_county too).
Run: `mise run qa:regression` → the 5 originals byte-identical; `single_county` reports
MISSING BASELINE (generate happens in Task 11's ceremony — confirm the tool's missing-baseline
behavior: if it hard-fails compare, gate `compare_all_baselines`'s scenario list to baselines
that exist and print a loud `PENDING CEREMONY: single_county` line instead).

- [ ] **Step 5: Commit**

```bash
mise run commit -- "feat(qa): single_county Wayne scenario — financial layer reaches the byte-identical tier (E2a)"
git log --oneline -1
```

---

### Task 9: E3 — financial dense channels + no-dead-columns rule

**Files:**
- Modify: `tools/regression_test.py` (`_dense_header`, `_dense_row`, new financial field
  lists, `check_dead_columns`, generate+compare wiring)
- Test: `tests/unit/tools/test_dead_columns.py` (NEW)

**Interfaces:**
- Consumes: `persistent_context["_tick_dynamics"]` / `["_national_financial"]` (Task 7);
  `AtRestChannel` declarations (Task 2).
- Produces: dense columns `financial_endogenous_rate`, `financial_profit_rate_ceiling`,
  `financial_s_r`, `financial_tightness` (all scenarios) and per-county
  `county_<fips>_total_s`, `county_<fips>_interest`, `county_<fips>_ground_rent`,
  `county_<fips>_taxes`, `county_<fips>_profit_enterprise` (county-bearing scenarios);
  `check_dead_columns(scenario, header, rows, coverage) -> list[str]`.

**This task opens the sanctioned red window** — after it, dense headers no longer match the
committed goldens. Do not regenerate here; Task 11 does.

- [ ] **Step 1: Write the failing tests**

`tests/unit/tools/test_dead_columns.py`:

```python
"""E3: an all-zeros channel FAILS unless declared at_rest with a reason."""

from __future__ import annotations

import pytest

from tools.regression_scenarios import AtRestChannel, ScenarioCoverage
from tools.regression_test import check_dead_columns

pytestmark = pytest.mark.unit

_COV = ScenarioCoverage(
    scenario="s", layers=(), systems=(),
    at_rest=(AtRestChannel(channel="financial_endogenous_rate",
                           reason="county-free scenario: no tensors, no interest"),),
)


def test_live_columns_pass() -> None:
    header = ["tick", "C001_wealth"]
    rows = [["0", "1.0"], ["1", "2.0"]]
    assert check_dead_columns("s", header, rows, (_COV,)) == []


def test_declared_at_rest_dead_column_passes() -> None:
    header = ["tick", "financial_endogenous_rate"]
    rows = [["0", "0.0"], ["1", "0.0"]]
    assert check_dead_columns("s", header, rows, (_COV,)) == []


def test_undeclared_dead_column_fails_naming_it() -> None:
    header = ["tick", "county_26163_interest"]
    rows = [["0", "0.0"], ["1", "0.0"]]
    findings = check_dead_columns("s", header, rows, (_COV,))
    assert len(findings) == 1
    assert "county_26163_interest" in findings[0]
    assert "at_rest" in findings[0]


def test_all_false_bool_column_is_dead() -> None:
    header = ["tick", "C001_active"]
    rows = [["0", "False"], ["1", "False"]]
    findings = check_dead_columns("s", header, rows, ())
    assert len(findings) == 1


def test_at_rest_channel_that_is_actually_live_fails() -> None:
    """A stale at_rest declaration over a live channel is itself a defect."""
    header = ["tick", "financial_endogenous_rate"]
    rows = [["0", "0.0"], ["1", "0.019855"]]
    findings = check_dead_columns("s", header, rows, (_COV,))
    assert len(findings) == 1
    assert "stale at_rest" in findings[0]
```

Run: `mise run test:q -- tests/unit/tools/test_dead_columns.py` → FAIL.

- [ ] **Step 2: Implement the channels**

In `tools/regression_test.py`:

1. `_dense_header(state)` gains parameters — it must also derive the county set:
   `sorted(t.county_fips for t in state.territories.values() if t.county_fips)` — and append,
   after the economy columns: the 4 `financial_*` columns always, then per county the 5
   `county_<fips>_*` columns. Return the county list alongside entity/edge keys (extend the
   returned tuple; update the topology-drift guard in `_dense_row` to also compare the
   county set derived from state each tick).
2. `_dense_row(state, tick, entity_ids, edge_keys, counties, context)` gains the harness
   context (thread it through from `_run_scenario_ticks`, which owns `persistent_context`):

```python
    financial = (context or {}).get("_national_financial") or {}
    endo = financial.get("endogenous_interest") or {}
    row.append(_format_dense_value(float(endo.get("rate", 0.0))))
    row.append(_format_dense_value(float(endo.get("profit_rate_ceiling", 0.0))))
    row.append(_format_dense_value(float(endo.get("s_r", 0.0))))
    row.append(_format_dense_value(float(endo.get("tightness", 0.0))))
    county_states = ((context or {}).get("_tick_dynamics") or {}).get("county_states", {})
    for fips in counties:
        cs = county_states.get(fips)
        dist = getattr(cs, "surplus_distribution", None) if cs is not None else None
        for value in (
            getattr(dist, "total_surplus_produced", 0.0),
            getattr(dist, "interest_payments", 0.0),
            getattr(dist, "ground_rent", 0.0),
            getattr(dist, "taxes_on_surplus", 0.0),
            getattr(dist, "profit_of_enterprise", 0.0),
        ):
            row.append(_format_dense_value(float(value)))
```

   Verify the four keys inside `national_financial["endogenous_interest"]` against the real
   stamp (`rg -n "endogenous_interest" src/babylon/domain/economics/tick/` — use the real key
   names: if the stamp calls it `tau` not `tightness`, the COLUMN stays
   `financial_tightness` but the read uses the real key). Same for
   `profit_of_enterprise` — if `SurplusValueDistribution` computes `p` as a property with a
   different name, read the model and use it.

3. `check_dead_columns` (complete):

```python
def check_dead_columns(
    scenario: str,
    header: list[str],
    rows: list[list[str]],
    coverage: tuple[Any, ...],
) -> list[str]:
    """E3: every column must move, or carry a declared at_rest reason.

    A channel that is all-zeros/all-absent across an entire run is an
    inertness signal (the U9 failure mode) — never a default.
    """
    at_rest: dict[str, str] = {}
    for cov in coverage:
        if cov.scenario == scenario:
            at_rest = {c.channel: c.reason for c in cov.at_rest}
            break
    dead_values = {"0.0", "-0.0", "0", "False", ""}
    findings: list[str] = []
    for j, column in enumerate(header):
        if column == "tick":
            continue
        dead = all(row[j] in dead_values for row in rows)
        if dead and column not in at_rest:
            findings.append(
                f"{scenario}: channel {column!r} is at rest across the entire run "
                f"but not declared at_rest in ScenarioCoverage. Either the channel "
                f"is dead (U9-class inertness — investigate) or declare it with a "
                f"reason in tools/regression_scenarios.py."
            )
        if not dead and column in at_rest:
            findings.append(
                f"{scenario}: stale at_rest declaration — channel {column!r} is "
                f"live but declared at rest ({at_rest[column]!r}). Delete the row."
            )
    return findings
```

4. Wire into BOTH paths: `generate` (after building a dense trace, before saving — findings
   abort the write with exit 1) and `compare` (on the freshly computed actual trace, findings
   are non-WARNING diffs → gate FAIL).

5. **Mint the at_rest declarations for the five county-free scenarios** in
   `tools/regression_scenarios.py`: the four `financial_*` channels each get
   `{"channel": "financial_endogenous_rate", "reason": "county-free scenario: no tensor registry, interest layer legitimately dormant"}`
   (adjust wording per channel). Then AUDIT the existing channels: run

```bash
PYTHONPATH="$PWD/src" poetry run python - <<'EOF'
# For each scenario: generate a dense trace in-memory, run check_dead_columns
# with empty at_rest, print findings — these are the pre-existing dead
# channels needing HONEST declarations (or bug reports).
EOF
```

   Every pre-existing dead channel found (e.g. a `repression_faced` that never moves in
   `two_node`) gets either an at_rest row with a TRUE reason or — if it looks like a real
   inertness bug — a note surfaced in the task report for owner triage. Do not blanket-declare.

- [ ] **Step 3: Run tests**

Run: `mise run test:q -- tests/unit/tools/test_dead_columns.py` → PASS.
Run: `mise run test:q -- tests/unit/tools tests/unit/sentinels` → PASS.
`mise run qa:regression` → now RED (headers widened) — expected; the red window is open.
Confirm the failure output shows the E4 attribution naming the header change, proving Task 5's
work composes.

- [ ] **Step 4: Commit**

```bash
mise run commit -- "feat(qa): financial dense channels + no-dead-columns rule (E3; red window until ceremony)"
git log --oneline -1
```

---

### Task 10: E2b + E5b — detroit dense golden + in-gate two-process determinism leg

**Files:**
- Create: `src/babylon/engine/trace_format.py` (shared serializer, MOVED from
  `tools/regression_test.py`'s `_format_dense_value` + `dense_trace_to_csv_bytes`)
- Modify: `tools/regression_test.py` (import the moved serializer; `_determinism_leg`;
  compare-bundle `--dense-baseline` leg)
- Modify: `src/babylon/engine/headless_runner/` (per-tick dense capture into the bundle as
  `dense_trace.csv`)
- Modify: `.mise.toml` `qa:e2e-regression` (pass `--dense-baseline tests/baselines/dense/detroit_tri_county.csv`)
- Test: `tests/unit/engine/test_trace_format.py` (NEW), existing determinism unit test stays
  as the fast-tier mirror

- [ ] **Step 1: Extract the shared serializer (byte-neutral move)**

`src/babylon/engine/trace_format.py` receives `_format_dense_value` →
`format_trace_value(value: float | bool) -> str` and `dense_trace_to_csv_bytes` →
`trace_rows_to_csv_bytes(header: list[str], rows: list[list[str]]) -> bytes`, code moved
verbatim (same `repr()` float policy, same `csv.writer(..., lineterminator="\n",
quoting=csv.QUOTE_MINIMAL)`, same trailing newline). `tools/regression_test.py` imports and
delegates. RST docstrings state the byte contract explicitly (this is a serialization used as
a contract — canonical byte layout SPECIFIED).

`tests/unit/engine/test_trace_format.py`:

```python
"""The trace byte contract: shortest-repr floats, RFC4180-minimal CSV, LF, trailing newline."""

from __future__ import annotations

import pytest

from babylon.engine.trace_format import format_trace_value, trace_rows_to_csv_bytes

pytestmark = pytest.mark.unit


def test_bool_renders_python_style() -> None:
    assert format_trace_value(True) == "True"
    assert format_trace_value(False) == "False"


def test_float_renders_shortest_roundtrip_repr() -> None:
    assert format_trace_value(0.1) == "0.1"
    assert format_trace_value(1.0) == "1.0"
    assert format_trace_value(1.179e9) == "1179000000.0"


def test_csv_bytes_contract() -> None:
    got = trace_rows_to_csv_bytes(["tick", "x"], [["0", "1.0"], ["1", "2.0"]])
    assert got == b"tick,x\n0,1.0\n1,2.0\n"
```

Run scoped test → PASS. Then `mise run test:q -- tests/unit/tools` → PASS (the move broke
nothing; the determinism unit test still passes because bytes are unchanged).

- [ ] **Step 2: Headless per-tick dense capture**

Read `src/babylon/engine/headless_runner/` first (`__main__`/runner module — find the tick
loop and the bundle writer that emits `summary.json`). Add a `dense_trace.csv` written into
the same `ARTIFACT_DIR` bundle: per tick, per county (sorted FIPS): columns

```
tick, county_<fips>_total_v, county_<fips>_total_c, county_<fips>_total_s, county_<fips>_total_k,
county_<fips>_population, county_<fips>_interest, county_<fips>_ground_rent, county_<fips>_taxes,
financial_endogenous_rate, financial_profit_rate_ceiling, financial_s_r, financial_tightness
```

sourced from the same per-county aggregates `county_terminal_snapshot` already computes at the
terminal tick (find that computation and lift it to per-tick capture) and the
`national_financial` graph attr (the runner holds the graph — read it directly post-tick).
Serialize via `trace_rows_to_csv_bytes` — one serializer, one byte contract.

- [ ] **Step 3: compare-bundle byte leg**

`compare-bundle` gains `--dense-baseline <Path>` (default
`tests/baselines/dense/detroit_tri_county.csv`): byte-compare the bundle's `dense_trace.csv`
against it; on mismatch, run `attribute_divergence("detroit_tri_county", ...)` and fail.
Missing baseline file → loud `PENDING CEREMONY` line, not silent pass, and (pre-ceremony
only) non-fatal. Update `.mise.toml`'s `qa:e2e-regression` run block to pass the flag.

- [ ] **Step 4: The in-gate two-process leg (E5b)**

In `tools/regression_test.py`, lift the unit test's mechanism into the tool:

```python
def _determinism_leg(scenario: str = "imperial_circuit") -> tuple[bool, list[str]]:
    """Two independent OS processes generate the same scenario; bytes must match.

    Folded from tests/unit/tools/test_regression_construction_cadence_determinism.py
    (U7.0) into the gate itself (E5b). PYTHONHASHSEED is stripped so each child
    randomizes its own hash seed — two processes sharing one seed would be a
    false-positive determinism proof.
    """
    import tempfile

    problems: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        dirs = [Path(tmp) / "a", Path(tmp) / "b"]
        for d in dirs:
            result = subprocess.run(
                [sys.executable, str(Path(__file__).resolve()), "generate",
                 "--scenario", scenario, "--dense", "--output", str(d)],
                capture_output=True, text=True,
                cwd=Path(__file__).parent.parent,
                env={k: v for k, v in os.environ.items() if k != "PYTHONHASHSEED"},
                timeout=300,
            )
            if result.returncode != 0:
                return False, [f"determinism leg: generate failed in {d}: {result.stderr[-500:]}"]
        a = json.loads((dirs[0] / f"{scenario}.json").read_text())
        b = json.loads((dirs[1] / f"{scenario}.json").read_text())
        a.pop("generated_at"), b.pop("generated_at")
        if a != b:
            problems.append(f"determinism leg: {scenario}.json differs between two processes")
        csv_a = (dirs[0] / "dense" / f"{scenario}.csv").read_bytes()
        csv_b = (dirs[1] / "dense" / f"{scenario}.csv").read_bytes()
        if csv_a != csv_b:
            problems.append(f"determinism leg: dense CSV differs between two processes")
    return not problems, problems
```

Wire into the `compare` command after the scenario loop; failures are gate failures. Print its
wall-clock so Task 11 can budget it. Keep the unit test untouched (fast-tier mirror per spec).

- [ ] **Step 5: Run + commit**

Run: `mise run test:q -- tests/unit/engine/test_trace_format.py tests/unit/tools` → PASS.
`qa:regression` remains red (window still open — determinism leg runs but header mismatch
persists).

```bash
mise run commit -- "feat(qa): detroit dense golden leg + in-gate two-process determinism (E2b+E5b)"
git log --oneline -1
```

---

### Task 11: THE CEREMONY — one baseline-minting commit + budget measurement

**Files:**
- Regenerate: `tests/baselines/*.json` (6: five + single_county),
  `tests/baselines/dense/*.csv` (6), `tests/baselines/dense/detroit_tri_county.csv` (NEW)
- Modify: `tests/baselines/README.md` (regeneration instructions cover the new members)

- [ ] **Step 1: Regenerate everything**

```bash
mise run qa:regression-generate-dense          # 6 JSON + 6 dense CSVs
# Detroit dense golden — one strict headless run, copy the bundle's trace:
ARTIFACT_DIR=$(PYTHONPATH="$PWD/src" poetry run python -m babylon.engine.headless_runner \
  --scope detroit-tri-county --ticks 5 --strict)
cp "$ARTIFACT_DIR/dense_trace.csv" tests/baselines/dense/detroit_tri_county.csv
```

- [ ] **Step 2: Close the red window and prove the whole gate**

Run: `mise run qa:regression`
Expected: PASS — 6/6 scenarios byte-identical, defines_hash silent, no-dead-columns green
(single_county's financial channels LIVE — this line is the program's raison d'être: U9's
inertness would now be a visible dead `county_26163_interest` column), determinism leg green.

Run: `mise run qa:e2e-regression`
Expected: PASS including the new dense byte leg.

Run: `mise run check:gate-coverage && mise run check:gate-coverage-truth` → both green.

Run: `mise run check` → green. Run `mise run test:q -- tests/unit` if `check`'s test leg
doesn't already cover the new files.

- [ ] **Step 3: Measure the budget (Global Constraint)**

```bash
time mise run qa:regression
```

Record real time in the commit body. If ≥ 5 min: reduce `single_county` tick count (SCENARIOS
gains a per-scenario `max_ticks` override honored by `generate`/`compare` — smallest honest
fix) and re-mint in THIS same ceremony; do not ship an over-budget gate.

- [ ] **Step 4: Declared drift table + commit**

The commit body carries the drift table — per file: what changed and WHY (new columns, new
scenario, refreshed hash), plus the measured runtime. Example rows:

```
tests/baselines/dense/imperial_circuit.csv  | +4 financial_* columns (all at_rest, declared) | E3
tests/baselines/single_county.json          | NEW scenario baseline (Wayne 26163)            | E2a
tests/baselines/dense/detroit_tri_county.csv| NEW dense golden (5 ticks, per-county+financial)| E2b
imperial_circuit defines_hash               | unchanged since Task 6 refresh                 | E5a
```

```bash
mise run commit -- "test(baselines): modernization ceremony — county scenarios + financial channels + determinism leg (E2+E3+E5b)"
git log --oneline -1
```

(Ceremonies are pre-authorized queue-wide with declared drift tables — owner audits post-hoc.)

---

### Task 12: Governance + PR

**Files:**
- Create: `ai/decisions/ADR090_qa_regression_modernization.yaml` (top-level key MUST equal
  the filename stem — the ADR governance test pattern from
  `tests/unit/decisions/test_adr083_vol3_money_scissors.py`; write the sibling test
  `tests/unit/decisions/test_adr090_qa_regression_modernization.py` by mirroring it, with
  `NEW_ADR_STEM = "ADR090_qa_regression_modernization"`)
- Modify: `ai/decisions/index.yaml` (append entry; VALIDATE no duplicate numbers, no dangling
  `file:` refs — the deconflict-sweep hazard)
- Modify: `ai/state.yaml` (truth_status paragraph: gate modernized, coverage declared+proved,
  financial layer visible, hash+determinism teeth armed; strike nothing)
- Modify: `CLAUDE.md` §Definition-of-done: `qa:regression` now = 6 scenarios + e2e dense leg +
  determinism leg + no-dead-columns; note `check:gate-coverage`/`check:gate-coverage-truth`
- Modify: `docs/superpowers/specs/2026-07-19-qa-regression-modernization-design.md` Status
  line → EXECUTED (ADR090)

- [ ] **Step 1: Write ADR090** — decision: the five spec elements, the E5a same-task
  hash-refresh deviation (declared), the CoverageGap governance (an uncovered system without a
  gap row gates; gap rows are owner-reviewable debt), the persistent_context financial
  round-trip, the single serializer contract, runtime measurement. Cite evidence files.
- [ ] **Step 2: Run the governance test + full local gate one last time**

```bash
mise run test:q -- tests/unit/decisions/test_adr090_qa_regression_modernization.py
mise run check && mise run qa:regression && mise run qa:e2e-regression
```

- [ ] **Step 3: Commit + PR**

```bash
mise run commit -- "docs(adr): ADR090 — qa:regression modernization program record"
git log --oneline -1
git push -u origin refactor/qa-regression-modernization
gh pr create --base dev --title "refactor(qa): modernize the byte-identical gate (E1-E5, ADR090)" --body-file <(...)
```

PR body: the five elements with evidence, the declared E5a deviation, the ceremony drift
table, measured runtime, honest-gaps list (CoverageGap rows verbatim). End with the
🤖 Generated-with line. Impl PRs self-merge on green (owner autonomy ruling 2026-07-19);
gitleaks 5xx failures in ~12s are GitHub transients — reread the job log before diagnosing,
rerun bounded.

---

## Self-Review (performed at plan-write time)

- **Spec coverage:** E1 → Tasks 1–4 (declared coverage, static completeness, dynamic truth).
  E2 → Tasks 8 (single_county) + 10 (detroit promotion); nationwide slot stays owner-tabled —
  represented as the SubstrateSystem/nationwide `CoverageGap` row (Task 2/3), satisfying
  "reserves the slot" without implementing. E3 → Tasks 7+9 (channels, no-dead-columns,
  at_rest). E4 → Task 5. E5 → Tasks 6 (hash) + 10 (two-process). Ceremony → Task 11.
  Governance → Task 12. Non-goals respected: no III.7 change, no mutation-in-CI, no
  nationwide implementation, employment-100k/capital_stock=0 remain disclosed inputs (their
  at-rest visibility lands via E3 automatically).
- **Placeholder scan:** the two `...` ellipses (Task 8 overrides builder body, Task 9 audit
  script body) are deliberate implementer-verification points with exact discovery commands —
  the surrounding interfaces are fully specified. Fixture zeros are explicitly marked
  plan-only with the real-extraction procedure given.
- **Type consistency:** `ScenarioCoverage`/`SystemEvidence`/`AtRestChannel`/`CoverageGap`
  names and fields match across Tasks 2/3/4/8/9; `create_single_county_scenario` and
  `build_single_county_overrides` names match across Tasks 8/9; `format_trace_value`/
  `trace_rows_to_csv_bytes` match across Task 10's steps; `attribute_divergence` signature
  matches Tasks 5/10.
