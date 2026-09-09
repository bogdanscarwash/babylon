# Market Scissors (Program 23) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register price⟷value as the sixth opposition (shadow-flagged, observes-only) with a
deterministic price/fictitious-capital dynamics system feeding it, plus timeseries exposure —
the law of value as a measured adjunction defect, per the 2026-07-17 owner conversation.

**Architecture:** Three layers, all shadow-first (WealthDistribution/P21 precedent):
(1) a generic `shadow` flag on `BoundOpposition` — shadow oppositions are measured every tick
but excluded from principal-contradiction scoring, frames, rupture, and regime; their states
land on a separate `shadow_opposition_states` graph attr. This implements the ADR072 blueprint's
shadow mechanics WITHOUT the sigma_authored channel (Amendment T stays unratified — this
opposition's poles are both material, so the recording-ADR pattern of ADR070/ADR073 applies).
(2) `MarketScissorsSystem` at position 17.8 — evolves national price-log and fictitious-log as
damped-driven oscillators around the value anchor (Marx: prices oscillate around values;
fictitious capital detaches from real capitalization), driven by the realized wage/value flow
already on the graph (`w_paid`/`v_produced`). State rides `G.graph["market"]` + an optional
`WorldState.market` field (byte-safe absent-axis pattern).
(3) `price_value` binding in the catalog reading a pre-extracted balance from `GraphInputs`.

**Tech Stack:** Python 3.11, Pydantic frozen models, rustworkx BabylonGraph, pytest, mise tasks.
Frontend: React + recharts + Zustand; Django bridge in `web/`.

## Global Constraints

- Constitution v2.10.0: determinism (III.7, no RNG, sorted iteration), honest absence (III.11),
  behavioral contracts (III.12), dialectic is primitive (Amendment S) — this is a RECORDING ADR,
  not an amendment (poles derive from existing GapReading/OppositionState shapes).
- `mise run qa:regression` MUST stay byte-identical after every engine task. If any sampled
  value moves: STOP, do not regenerate baselines (that requires owner sign-off).
- Observe-only: NOTHING may read `shadow_opposition_states`, `G.graph["market"]`,
  `WorldState.market`, or the new tick_summary columns to change tick outputs. Feedback
  (correction → rupture into material base) is Phase 2, owner-gated — documented in ADR077 only.
- Never hardcode a coefficient — everything via `MarketDefines`, regenerate
  `src/babylon/data/defines.yaml` with `poetry run python tools/generate_defines_config.py`.
- Commit after each task with `mise run commit -- "type(scope): msg"`; end messages with the
  Co-Authored-By trailer (mise commit adds context; verify HEAD moved).
- Tests single-flight only — never fan out parallel pytest runs. Use
  `mise run test:q -- <path>` for scoped runs.
- Branch: `feature/market-scissors-opposition` (already created from dev @ 4ee7bd4f).

---

### Task 1: Shadow flag in the dialectics core

**Files:**
- Modify: `src/babylon/domain/dialectics/core/opposition.py` (BoundOpposition ~line 255,
  OppositionRegistry.__init__ ~429, step ~465, _principal_key ~564)
- Test: `tests/unit/dialectics/test_opposition_registry.py` (append a `TestShadowOppositions` class)

**Interfaces:**
- Produces: `BoundOpposition.shadow: bool = False`;
  `OppositionRegistry.shadow_keys -> frozenset[str]` property;
  `_principal_key(...) -> str | None` (None when every draft is shadow);
  contract change: shadow states NEVER get `is_principal=True`; an all-shadow registry
  marks no principal at all.

- [ ] **Step 1: Write the failing tests** — append to `tests/unit/dialectics/test_opposition_registry.py`
  (reuse that file's existing helper style for building bindings; a constant-reading measure is
  `lambda _inputs: GapReading(gap=0.9, balance=0.5)`):

```python
class TestShadowOppositions:
    """Shadow bindings are measured but never adjudicate (ADR077, Program 23)."""

    @staticmethod
    def _binding(key: str, gap: float, *, shadow: bool = False) -> BoundOpposition[None]:
        return BoundOpposition(
            spec=OppositionSpec(key=key, pole_a="a-pole", pole_b="b-pole"),
            measure=lambda _inputs, _gap=gap: GapReading(gap=_gap, balance=0.1),
            shadow=shadow,
        )

    def test_shadow_defaults_off(self) -> None:
        binding = self._binding("plain", 0.5)
        assert binding.shadow is False

    def test_shadow_keys_property(self) -> None:
        registry = OppositionRegistry(
            [self._binding("canon", 0.2), self._binding("ghost", 0.9, shadow=True)]
        )
        assert registry.shadow_keys == frozenset({"ghost"})

    def test_shadow_never_principal_even_with_top_score(self) -> None:
        registry = OppositionRegistry(
            [self._binding("canon", 0.1), self._binding("ghost", 1.0, shadow=True)]
        )
        states = registry.step(None, tick=1)
        by_key = {s.key: s for s in states}
        assert by_key["ghost"].is_principal is False
        assert by_key["canon"].is_principal is True

    def test_all_shadow_registry_marks_no_principal(self) -> None:
        registry = OppositionRegistry([self._binding("ghost", 0.9, shadow=True)])
        states = registry.step(None, tick=1)
        assert all(not s.is_principal for s in states)

    def test_shadow_state_still_measured_with_rate(self) -> None:
        registry = OppositionRegistry([self._binding("ghost", 0.9, shadow=True)])
        first = registry.step(None, tick=1)
        second = registry.step(None, tick=2, previous={s.key: s for s in first})
        assert second[0].gap == pytest.approx(0.9)
        assert second[0].rate == pytest.approx(0.0)

    def test_canonical_states_identical_with_and_without_shadow_sibling(self) -> None:
        canon_only = OppositionRegistry([self._binding("canon", 0.4)])
        with_ghost = OppositionRegistry(
            [self._binding("canon", 0.4), self._binding("ghost", 1.0, shadow=True)]
        )
        lone = canon_only.step(None, tick=3)
        paired = tuple(s for s in with_ghost.step(None, tick=3) if s.key == "canon")
        assert lone == paired
```

- [ ] **Step 2: Run, confirm failure** — `mise run test:q -- tests/unit/dialectics/test_opposition_registry.py -k Shadow`
  Expected: FAIL (`TypeError: BoundOpposition.__init__() got an unexpected keyword argument 'shadow'`).

- [ ] **Step 3: Implement.** In `opposition.py`:

```python
@dataclass(frozen=True)
class BoundOpposition[I]:
    """... existing docstring; append:

    ``shadow`` (ADR077): a shadow binding is measured every tick but never
    adjudicates — excluded from principal-contradiction scoring, and the
    engine routes its states to ``shadow_opposition_states`` instead of
    ``opposition_states`` (observes-only until an owner-gated promotion).
    """

    spec: OppositionSpec
    measure: GapMeasure[I]
    pole_measure: PoleMeasure[I] | None = None
    shadow: bool = False
```

In `OppositionRegistry.__init__`, after `self._bindings = ...`:

```python
        self._shadow_keys: frozenset[str] = frozenset(
            binding.spec.key for binding in self._bindings if binding.shadow
        )
```

Add property below `keys`:

```python
    @property
    def shadow_keys(self) -> frozenset[str]:
        """Keys of shadow bindings: measured, never principal (ADR077)."""
        return self._shadow_keys
```

Rework `_principal_key` (return type `str | None`; shadow excluded BEFORE the governed
fallback so a governed fallback can never resurrect a shadow):

```python
    def _principal_key(self, drafts: Sequence[OppositionState]) -> str | None:
        """Highest score among non-shadow drafts; ties break lexicographically.

        Shadow drafts never lead (ADR077) — an all-shadow registry has NO
        principal. Governed oppositions are excluded next: a predecessor whose
        motion its successor governs never leads; the fallback to all
        non-shadow drafts keeps that pool non-empty (the terminal successor is
        always ungoverned under acyclic governance).
        """
        candidates = [draft for draft in drafts if draft.key not in self._shadow_keys]
        if not candidates:
            return None
        governed = set(self._governance)
        pool = [draft for draft in candidates if draft.key not in governed] or candidates
        best = pool[0]
        best_score = self._score(best)
        for candidate in pool[1:]:
            score = self._score(candidate)
            if score > best_score:
                best, best_score = candidate, score
        return best.key
```

`step()` needs no code change (`draft.key == principal_key` is False for every key when
`principal_key is None`) — but update its docstring's return contract: "with exactly one
``is_principal=True`` among non-shadow states (none if the registry is empty or all-shadow)".

- [ ] **Step 4: Run** the new class, then the whole file: both green.
- [ ] **Step 5: Commit** — `mise run commit -- "feat(dialectics): shadow flag on BoundOpposition — measured, never principal (ADR077)"`

---

### Task 2: ContradictionSystem routes shadow states to their own channel

**Files:**
- Modify: `src/babylon/engine/systems/contradiction.py` (`_step_registry` ~157, `_read_previous` ~200)
- Test: `tests/unit/engine/systems/test_contradiction_system.py` (append `TestShadowChannel`)

**Interfaces:**
- Produces: `SHADOW_OPPOSITION_STATES_ATTR = "shadow_opposition_states"` module constant;
  canonical states keep flowing to frames/rupture/regime/`opposition_states` EXACTLY as today;
  shadow states go only to the new attr; `_read_previous` merges both attrs for rate continuity.
- Consumes: `registry.shadow_keys` from Task 1.

- [ ] **Step 1: Failing tests.** Follow the file's existing fixture style (ServiceContainer.create()
  fixture with `container.database.close()` teardown; hand-built BabylonGraph). Build a custom
  ServiceContainer whose `opposition_registry` holds one canonical + one shadow binding over
  `GraphInputs` (constant measures as in Task 1), step the system twice, then assert:
  - `graph.get_graph_attr("opposition_states")` contains ONLY the canonical key;
  - `graph.get_graph_attr("shadow_opposition_states")` contains ONLY the shadow key;
  - the shadow state's second-tick `rate` is computed from its first-tick gap (continuity
    through the shadow attr);
  - `contradiction_frames` principal/secondary never name the shadow key;
  - with a registry containing NO shadow bindings, `shadow_opposition_states` is never written
    (`graph.get_graph_attr("shadow_opposition_states", None) is None`) — byte-safety for
    pre-ADR077 graphs.

- [ ] **Step 2: Run, confirm failure** (attr missing / states in wrong attr).

- [ ] **Step 3: Implement.** Add constant near `OPPOSITION_STATES_ATTR`:

```python
#: Graph attribute holding ``{key: OppositionState.model_dump()}`` for SHADOW
#: bindings (ADR077): measured every tick, adjudicating nothing. Kept apart
#: from ``opposition_states`` so the pre-position-18 consumers and the frames/
#: rupture/regime machinery never see a shadow key. Same cross-tick channel
#: semantics (the graph persists; the facade recomputes fresh each tick).
SHADOW_OPPOSITION_STATES_ATTR = "shadow_opposition_states"
```

In `_step_registry`, replace the block from `states = self._apply_interventions(...)` through
the `graph.set_graph_attr(OPPOSITION_STATES_ATTR, ...)` call with:

```python
        states = self._apply_interventions(graph, states)

        shadow_keys = registry.shadow_keys
        canonical = tuple(s for s in states if s.key not in shadow_keys)
        shadow = tuple(s for s in states if s.key in shadow_keys)

        if canonical:
            self._write_frames(graph, services, registry, canonical)
            self._maybe_rupture(services, canonical, tick)
            self._classify_regime(graph, services, registry, canonical, tick)
        graph.set_graph_attr(
            OPPOSITION_STATES_ATTR, {state.key: state.model_dump() for state in canonical}
        )
        if shadow:
            graph.set_graph_attr(
                SHADOW_OPPOSITION_STATES_ATTR,
                {state.key: state.model_dump() for state in shadow},
            )
        self._step_pole_channel(graph, registry, inputs)
```

And `_read_previous` merges (shadow keys can never collide with canonical — registry keys are
unique):

```python
    @staticmethod
    def _read_previous(graph: GraphProtocol) -> dict[str, OppositionState]:
        """Reconstruct last tick's states from BOTH opposition attrs.

        Shadow states (ADR077) live on ``shadow_opposition_states`` but need
        the same rate/inertia continuity as canonical ones.
        """
        raw: dict[str, Any] = {
            **(graph.get_graph_attr(OPPOSITION_STATES_ATTR, {}) or {}),
            **(graph.get_graph_attr(SHADOW_OPPOSITION_STATES_ATTR, {}) or {}),
        }
        return {key: OppositionState(**value) for key, value in raw.items()}
```

- [ ] **Step 4: Run** the file green, then `mise run test:q -- tests/unit/dialectics tests/unit/engine/systems/test_contradiction_system.py`.
- [ ] **Step 5: qa gate** — `mise run qa:regression` byte-identical (no shadow bindings registered yet; this proves the plumbing alone is inert).
- [ ] **Step 6: Commit** — `feat(engine): shadow-opposition channel in ContradictionSystem (ADR077)`

---

### Task 3: MarketDefines category

**Files:**
- Create: `src/babylon/config/defines/market.py`
- Modify: `src/babylon/config/defines/__init__.py` (import + alphabetized `__all__`),
  `src/babylon/config/defines/_assembler.py` (field + docstring bullet + `_from_yaml_dict` branch)
- Regenerate: `src/babylon/data/defines.yaml`
- Test: covered by `tests/unit/config/test_constants_sync.py` round-trip + `--check` gate (no new file)

**Interfaces:**
- Produces: `GameDefines.market: MarketDefines` with fields exactly as below (Task 4/6/7 read them).

- [ ] **Step 1: Create `market.py`** (descriptions carry provenance tags per convention):

```python
"""Market-scissors coefficients (Program 23, ADR077).

Price and fictitious-capital dynamics: two damped-driven oscillators in
log-ratio space around the value anchor. The restoring force IS the law of
value; the drive terms are realized value/surplus growth (demand pull and
return-chasing speculation). All shadow-phase: nothing downstream consumes
the outputs yet.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class MarketDefines(BaseModel):
    """Price⟷value scissors dynamics coefficients (per-tick units)."""

    model_config = ConfigDict(frozen=True)

    price_reversion: float = Field(
        default=0.02,
        ge=0.0,
        le=1.0,
        description=(
            "Game design: law-of-value restoring stiffness on the log "
            "price-to-value ratio. Underdamped with price_damping=0.15 so "
            "prices oscillate around values (Capital Vol. III ch. 10)."
        ),
    )
    price_damping: float = Field(
        default=0.15,
        ge=0.0,
        le=2.0,
        description="Behavior-tuned: velocity damping on the price oscillator; keeps the discrete Euler step stable at dt=1 tick.",
    )
    price_drive_sensitivity: float = Field(
        default=0.6,
        ge=0.0,
        le=5.0,
        description="Game design: how strongly relative value-output growth (demand pull) accelerates the price level.",
    )
    fictitious_reversion: float = Field(
        default=0.01,
        ge=0.0,
        le=1.0,
        description="Game design: gravity pulling fictitious capitalization back to real capitalization (surplus_ema / capitalization_rate); weaker than price_reversion — bubbles outlive price swings (Vol. III part 5).",
    )
    fictitious_damping: float = Field(
        default=0.1,
        ge=0.0,
        le=2.0,
        description="Behavior-tuned: velocity damping on the fictitious-capital oscillator.",
    )
    fictitious_drive_sensitivity: float = Field(
        default=0.9,
        ge=0.0,
        le=5.0,
        description="Game design: how strongly realized-surplus growth (return-chasing) accelerates fictitious capitalization.",
    )
    momentum_coupling: float = Field(
        default=0.5,
        ge=0.0,
        le=5.0,
        description="Game design: speculation chases price momentum — the price oscillator's velocity feeds the fictitious drive (tension-on-tension).",
    )
    surplus_ema_alpha: float = Field(
        default=0.15,
        gt=0.0,
        le=1.0,
        description="Engineering: EMA smoothing for the surplus/value anchors; ~13-tick (one quarter) memory at 0.15.",
    )
    scissors_balance_scale: float = Field(
        default=0.5,
        gt=0.0,
        le=5.0,
        description="Engineering: tanh scale mapping the log price-to-value ratio onto the opposition Balance in [-1, 1]; 0.5 saturates near a 65% price-over-value divergence.",
    )
    max_abs_log: float = Field(
        default=2.0,
        gt=0.0,
        le=5.0,
        description="Engineering: hard clamp on both log ratios (e^2 ~ 7.4x divergence); momentum zeroes at the rail so the clamp cannot pump energy.",
    )
    capitalization_rate: float = Field(
        default=0.05,
        gt=0.0,
        le=1.0,
        description="Game design: expected profit rate capitalizing surplus into real capitalization K = s_ema / r (Vol. III ch. 29 interest-bearing capital).",
    )
```

- [ ] **Step 2: Wire assembler** — `market: MarketDefines = Field(default_factory=MarketDefines)`
  in `GameDefines` (alphabetical position among category fields), docstring bullet
  `- market: price⟷value scissors dynamics (Program 23 shadow)`, and in `_from_yaml_dict`:
  `market=MarketDefines(**data.get("market", {})),`. Export from `__init__.py` (sorted).
- [ ] **Step 3: Regenerate** — `poetry run python tools/generate_defines_config.py`; expect exit 0
  and a new `market:` block in `src/babylon/data/defines.yaml`.
- [ ] **Step 4: Verify** — `mise run test:q -- tests/unit/config/test_constants_sync.py` green.
- [ ] **Step 5: Commit** — `feat(config): MarketDefines category (Program 23 scissors coefficients)`

---

### Task 4: Pure scissors formulas

**Files:**
- Create: `src/babylon/formulas/market.py`
- Modify: `src/babylon/formulas/__init__.py` (imports + `__all__`)
- Test: `tests/unit/formulas/test_market.py` (create; mirror sibling formula-test style)

**Interfaces:**
- Produces (Task 6/7 consume):
  `calculate_ema(previous: float, value: float, *, alpha: float) -> float`;
  `calculate_growth_drive(current: float, previous: float, *, sensitivity: float) -> float`;
  `calculate_scissors_step(log_ratio: float, velocity: float, drive: float, *, reversion: float, damping: float, max_abs_log: float) -> tuple[float, float]`;
  `calculate_scissors_balance(log_ratio: float, *, scale: float) -> float`.

- [ ] **Step 1: Failing tests** (`tests/unit/formulas/test_market.py`):

```python
"""Market-scissors formula laws (Program 23, ADR077).

Behavioral contracts (Constitution III.12): the law of value as restoring
force, boundedness, and determinism are pinned as input→output laws, not
implementation choreography.
"""

import pytest

from babylon.formulas.market import (
    calculate_ema,
    calculate_growth_drive,
    calculate_scissors_balance,
    calculate_scissors_step,
)


class TestScissorsStep:
    def test_zero_state_zero_drive_stays_at_value(self) -> None:
        assert calculate_scissors_step(
            0.0, 0.0, 0.0, reversion=0.02, damping=0.15, max_abs_log=2.0
        ) == (0.0, 0.0)

    def test_law_of_value_restores_perturbation(self) -> None:
        """An opened scissors with no drive decays toward zero — Marx's gravitation of price to value."""
        log_ratio, velocity = 1.0, 0.0
        for _ in range(400):  # fixed bound (Power-of-10 rule 2)
            log_ratio, velocity = calculate_scissors_step(
                log_ratio, velocity, 0.0, reversion=0.02, damping=0.15, max_abs_log=2.0
            )
        assert abs(log_ratio) < 0.05

    def test_positive_drive_opens_scissors_upward(self) -> None:
        log_ratio, velocity = calculate_scissors_step(
            0.0, 0.0, 0.1, reversion=0.02, damping=0.15, max_abs_log=2.0
        )
        assert log_ratio > 0.0
        assert velocity > 0.0

    def test_clamp_kills_momentum_at_rail(self) -> None:
        log_ratio, velocity = calculate_scissors_step(
            1.99, 5.0, 1.0, reversion=0.0, damping=0.0, max_abs_log=2.0
        )
        assert log_ratio == 2.0
        assert velocity == 0.0

    def test_deterministic(self) -> None:
        a = calculate_scissors_step(0.3, -0.1, 0.05, reversion=0.02, damping=0.15, max_abs_log=2.0)
        b = calculate_scissors_step(0.3, -0.1, 0.05, reversion=0.02, damping=0.15, max_abs_log=2.0)
        assert a == b


class TestGrowthDrive:
    def test_zero_previous_is_honest_zero(self) -> None:
        assert calculate_growth_drive(5.0, 0.0, sensitivity=1.0) == 0.0

    def test_relative_growth(self) -> None:
        assert calculate_growth_drive(1.1, 1.0, sensitivity=1.0) == pytest.approx(0.1)

    def test_contraction_is_negative(self) -> None:
        assert calculate_growth_drive(0.9, 1.0, sensitivity=2.0) == pytest.approx(-0.2)


class TestEma:
    def test_alpha_one_tracks_value(self) -> None:
        assert calculate_ema(3.0, 7.0, alpha=1.0) == 7.0

    def test_blend(self) -> None:
        assert calculate_ema(0.0, 1.0, alpha=0.25) == pytest.approx(0.25)


class TestBalance:
    def test_zero_log_is_balanced(self) -> None:
        assert calculate_scissors_balance(0.0, scale=0.5) == 0.0

    def test_positive_log_is_price_pole(self) -> None:
        assert 0.0 < calculate_scissors_balance(0.5, scale=0.5) < 1.0

    def test_bounded(self) -> None:
        assert calculate_scissors_balance(100.0, scale=0.5) == pytest.approx(1.0)
        assert calculate_scissors_balance(-100.0, scale=0.5) == pytest.approx(-1.0)
```

- [ ] **Step 2: Run, confirm ModuleNotFoundError.**
- [ ] **Step 3: Implement** `src/babylon/formulas/market.py`:

```python
"""Market-scissors dynamics: price⟷value divergence as a damped-driven oscillator.

Program 23 (ADR077). Prices are the phenomenal form of value; the law of
value is the restoring force pulling the log price-to-value ratio back to
zero, while demand pull (value-output growth) and return-chasing speculation
(surplus growth + price momentum) drive it away. Crisis theory as mechanism:
the correction is the deterministic snap-back of an opened scissors — in
Phase 1 it is only OBSERVED, never fed back (owner-gated Phase 2).

All functions are pure, per-tick-unit (implicit dt = 1 tick), and
deterministic (Constitution III.7).
"""

from __future__ import annotations

import math

__all__ = [
    "calculate_ema",
    "calculate_growth_drive",
    "calculate_scissors_balance",
    "calculate_scissors_step",
]

_GROWTH_EPSILON = 1e-9
"""Below this anchor a growth ratio is undefined — honest zero drive (III.11)."""


def calculate_ema(previous: float, value: float, *, alpha: float) -> float:
    """Exponential moving average step.

    :param previous: The prior EMA value.
    :param value: This tick's observation.
    :param alpha: Blend weight in (0, 1]; 1 tracks the raw value.
    :returns: ``alpha * value + (1 - alpha) * previous``.
    """
    return alpha * value + (1.0 - alpha) * previous


def calculate_growth_drive(current: float, previous: float, *, sensitivity: float) -> float:
    """Relative-growth drive term: ``sensitivity * (current - previous) / previous``.

    :param current: This tick's flow (value output or realized surplus).
    :param previous: The prior anchor (EMA) of the same flow.
    :param sensitivity: Drive gain (a ``MarketDefines`` coefficient).
    :returns: The signed drive; 0.0 when the anchor is ~zero (no growth
        signal is fabricated from an absent base — Constitution III.11).
    """
    if previous <= _GROWTH_EPSILON:
        return 0.0
    return sensitivity * (current - previous) / previous


def calculate_scissors_step(
    log_ratio: float,
    velocity: float,
    drive: float,
    *,
    reversion: float,
    damping: float,
    max_abs_log: float,
) -> tuple[float, float]:
    """One semi-implicit Euler step of the damped-driven scissors oscillator.

    ``x'' = drive - reversion * x - damping * x'`` in log-ratio space; the
    reversion term IS the law of value (gravitation of price to value). The
    velocity integrates first, then the position (semi-implicit — better
    energy behavior than explicit Euler at dt = 1). Hitting the ``max_abs_log``
    rail zeroes the velocity so the clamp cannot pump energy into the system.

    :param log_ratio: Current ``ln(form / substance)`` — e.g. ln(price / value).
    :param velocity: Current d(log_ratio)/dt.
    :param drive: External drive (see :func:`calculate_growth_drive`).
    :param reversion: Restoring stiffness (>= 0).
    :param damping: Velocity damping (>= 0).
    :param max_abs_log: Hard clamp on |log_ratio| (> 0).
    :returns: ``(new_log_ratio, new_velocity)``.
    """
    acceleration = drive - reversion * log_ratio - damping * velocity
    new_velocity = velocity + acceleration
    new_log = log_ratio + new_velocity
    if new_log > max_abs_log:
        return max_abs_log, 0.0
    if new_log < -max_abs_log:
        return -max_abs_log, 0.0
    return new_log, new_velocity


def calculate_scissors_balance(log_ratio: float, *, scale: float) -> float:
    """Map a log ratio onto the opposition ``Balance`` in [-1, 1].

    Positive = the form pole (price) dominates its substance (value).

    :param log_ratio: ``ln(form / substance)``.
    :param scale: tanh scale (> 0); smaller saturates sooner.
    :returns: ``tanh(log_ratio / scale)``, clamped to [-1, 1] against float
        edge rounding.
    """
    return max(-1.0, min(1.0, math.tanh(log_ratio / scale)))
```

- [ ] **Step 4: Export** in `formulas/__init__.py` (import block + 4 `__all__` entries, sorted),
  run the test file green, then `mise run test:q -- tests/unit/formulas`.
- [ ] **Step 5: Commit** — `feat(formulas): market-scissors oscillator laws (Program 23)`

---

### Task 5: MarketState model + WorldState round-trip

**Files:**
- Create: `src/babylon/models/market.py`
- Modify: `src/babylon/models/world_state.py` (field after `wealth_distribution` ~463;
  to_graph after the wealth_distribution write ~653; from_graph next to the
  wealth_distribution reconstruction ~957), plus the models `__init__` export if
  `WealthDistribution` is exported there (check `rg -n "wealth_distribution" src/babylon/models/__init__.py` and mirror).
- Test: extend `tests/unit/engine/systems/test_market_system.py` created in Task 6 with a
  `TestRoundTrip` class (written here, executed there — keep one test home for the axis).

**Interfaces:**
- Produces: `MarketState` frozen model with fields
  `price_log/price_velocity/fictitious_log/fictitious_velocity: float`,
  `surplus_ema/value_ema: float (ge=0)`, `tick: int (ge=0)`;
  `WorldState.market: MarketState | None = None`; graph metadata key `"market"`.

- [ ] **Step 1: Create `src/babylon/models/market.py`** (mirror `models/wealth_distribution.py`):

```python
"""National market-scissors state — the price⟷value axis (Program 23 Phase 1).

The phenomenal form's dynamical state: log price-to-value ratio and log
fictitious-to-real ratio with their oscillator velocities, plus the EMA
anchors of the realized value/surplus flow. ``WorldState`` holds it as an
optional field and round-trips it via ``G.graph["market"]`` (the
``wealth_distribution`` metadata pattern), written only when set so
axis-less graphs stay byte-identical.
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict, Field


class MarketState(BaseModel):
    """Price and fictitious-capital oscillator state (national, per-tick units).

    :ivar price_log: ``ln(price index / value anchor)`` — the scissors.
    :ivar price_velocity: d(price_log)/dt.
    :ivar fictitious_log: ``ln(fictitious capitalization / real capitalization)``.
    :ivar fictitious_velocity: d(fictitious_log)/dt.
    :ivar surplus_ema: EMA of realized surplus ``max(Σv_produced - Σw_paid, 0)``.
    :ivar value_ema: EMA of realized value output ``Σv_produced``.
    :ivar tick: the tick this state was computed at.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    price_log: float
    price_velocity: float
    fictitious_log: float
    fictitious_velocity: float
    surplus_ema: float = Field(ge=0.0)
    value_ema: float = Field(ge=0.0)
    tick: int = Field(ge=0)
```

- [ ] **Step 2: WorldState field** (import `MarketState` next to the `WealthDistribution` import):

```python
    market: MarketState | None = Field(
        default=None,
        description=(
            "National price⟷value scissors state (Program 23 Phase-1 shadow; "
            "MarketScissorsSystem seeds and advances it). Rides graph metadata "
            "like wealth_distribution, written only when set so axis-less "
            "graphs stay byte-identical. None means the axis has not been "
            "computed for this state."
        ),
    )
```

to_graph, directly after the wealth_distribution block:

```python
        if self.market is not None:
            G.graph["market"] = self.market.model_dump()
```

from_graph, next to the wealth_distribution reconstruction:

```python
            market=(
                MarketState(**G.graph["market"])
                if isinstance(G.graph.get("market"), dict)
                else None
            ),
```

- [ ] **Step 3: Round-trip tests** (land in `tests/unit/engine/systems/test_market_system.py`, Task 6):
  absent axis writes NO `"market"` key; a set axis survives to_graph→from_graph with exact
  field equality; `MarketState` is frozen (mutation raises `ValidationError`).
- [ ] **Step 4: Commit** with Task 6 (same unit of work) or alone if Task 6 slips:
  `feat(models): MarketState axis + WorldState round-trip (Program 23)`

---

### Task 6: MarketScissorsSystem @17.8

**Files:**
- Create: `src/babylon/engine/systems/market_scissors.py`
- Modify: `src/babylon/engine/simulation_engine.py` (insert into `_DEFAULT_SYSTEMS` between
  `SovereigntySystem()` and `ContradictionSystem()`; add class to `CONSEQUENCE_SYSTEMS`),
  `tests/unit/engine/test_system_order.py` (29 → 30, expected_order list),
  `ai/architecture.yaml` (annotated order — add 17.8 entry)
- Test: `tests/unit/engine/systems/test_market_system.py` (create)

**Interfaces:**
- Consumes: `services.defines.market` (Task 3), formulas (Task 4), `MarketState` (Task 5);
  node attrs `w_paid`/`v_produced`/`active` (wages phase, same presence rule as
  `ContradictionSystem._build_graph_inputs`).
- Produces: `G.graph["market"]` = `MarketState.model_dump()` dict — read by Task 7's
  `_build_graph_inputs` and Task 9's bridge. NOTHING else may read it (observes-only).

- [ ] **Step 1: Failing tests.** Classes, following `test_wealth_distribution_system.py` structure
  (ServiceContainer fixture with teardown; hand-built BabylonGraph; `{"tick": N}` dict context;
  helper `_paid_worker(graph, node_id, w_paid, v_produced)` adding an active social_class node):
  - `TestWiring`: system in `_DEFAULT_SYSTEMS`; in `CONSEQUENCE_SYSTEMS`; index of
    `MarketScissorsSystem` == index of `ContradictionSystem` − 1 (runs immediately before @18);
    module imports `calculate_scissors_step` (orphan guard, the P21 idiom).
  - `TestHonestAbsence`: graph with NO `w_paid`/`v_produced` nodes → step → no `"market"` key.
  - `TestSeeding`: first step with two paid workers (w=0.8/v=1.0 and w=0.5/v=1.0) → state has
    `price_log == 0.0`, velocities 0.0, `value_ema == 2.0`, `surplus_ema == pytest.approx(0.7)`,
    `tick` stamped.
  - `TestDynamics`: (a) constant flows over 30 ticks → `price_log` stays ~0 (no drive, no
    divergence); (b) grow `v_produced` 2%/tick for 30 ticks → `price_log > 0` (boom opens the
    scissors upward) and `fictitious_log > 0` (speculation chases); (c) then freeze growth for
    120 ticks → `abs(price_log)` decays below its boom peak (the correction — law of value);
    (d) two identical runs → identical final metadata dict (determinism).
  - `TestRoundTrip` (from Task 5).
- [ ] **Step 2: Run, confirm failure.**
- [ ] **Step 3: Implement** `src/babylon/engine/systems/market_scissors.py`:

```python
"""Market-scissors system — Phase 1 SHADOW ONLY (Program 23, ADR077).

Position 17.8: after the consequence systems that settle this tick's wage/
value accounting readers, immediately BEFORE ContradictionSystem @18 so the
``price_value`` shadow opposition measures a fresh scissors.

Evolves the national price⟷value axis: the log price-to-value ratio and log
fictitious-to-real ratio as damped-driven oscillators
(:mod:`babylon.formulas.market`), driven by the realized value flow already
on the graph — ``Σ v_produced`` (demand pull on prices) and
``Σ max(v_produced − w_paid, 0)`` (return-chasing on fictitious capital),
with price momentum feeding speculation (``momentum_coupling``).

PHASE 1 SCOPE (binding): observe-only shadow.

- State home: ``G.graph["market"]`` metadata (the ``wealth_distribution``
  round-trip pattern; ``WorldState.market`` carries it across facade ticks).
- Nothing reads it to change tick outputs: no correction feedback into
  wealth, credit, or the reserve army (Phase 2, owner-gated), so the sampled
  qa:regression checkpoints stay byte-identical.
- Honest absence: a graph with no paid-worker accounting gets NO market —
  the phenomenal form cannot precede its substance (Constitution III.11).
- Deterministic: seeded from defines, nodes iterated in sorted-id order,
  zero RNG (Constitution III.7).
"""

from __future__ import annotations

from typing import TYPE_CHECKING, ClassVar

from babylon.formulas.market import (
    calculate_ema,
    calculate_growth_drive,
    calculate_scissors_step,
)
from babylon.kernel.system_base import SystemBase
from babylon.kernel.system_protocol import ContextType
from babylon.models.market import MarketState

if TYPE_CHECKING:
    from babylon.kernel.graph_protocol import GraphProtocol
    from babylon.kernel.services import ServicesProtocol

#: Graph-metadata key carrying the axis (matches ``WorldState.market``).
MARKET_ATTR = "market"


def _aggregate_wage_value(graph: GraphProtocol) -> tuple[float, float] | None:
    """``(Σ w_paid, Σ v_produced)`` over active paid-worker nodes, or ``None``.

    Same selection rule as ``ContradictionSystem._build_graph_inputs``:
    presence of BOTH attrs marks a paid worker class; inactive nodes skip.
    Sorted-id iteration fixes the float summation order (III.7). ``None``
    (not zeros) when no node carries the accounting pair — honest absence.
    """
    wages = 0.0
    value = 0.0
    found = False
    for node in sorted(graph.query_nodes(), key=lambda n: n.id):
        attrs = node.attributes
        if not attrs.get("active", True):
            continue
        if "w_paid" not in attrs or "v_produced" not in attrs:
            continue
        wages += float(attrs["w_paid"])
        value += float(attrs["v_produced"])
        found = True
    return (wages, value) if found else None


class MarketScissorsSystem(SystemBase):
    """Phase 1 SHADOW: the national price⟷value scissors axis."""

    name: ClassVar[str] = "Market Scissors"
    # Spec 053 INV-001: does not mutate hex c+v+s; opted in by default-deny.
    creates_value: ClassVar[bool] = False

    def step(
        self,
        graph: GraphProtocol,
        services: ServicesProtocol,
        context: ContextType,
    ) -> None:
        """Seed (first observation) or advance the scissors oscillators."""
        defines = services.defines.market
        tick = context.get("tick", 0) if isinstance(context, dict) else getattr(context, "tick", 0)
        metadata = getattr(graph, "graph", None)
        if not isinstance(metadata, dict):  # pragma: no cover — BabylonGraph always has it
            return
        flow = _aggregate_wage_value(graph)
        if flow is None:
            return  # no value substrate, no phenomenal form (III.11)
        wages, value = flow
        surplus = max(value - wages, 0.0)
        prior = metadata.get(MARKET_ATTR)
        if prior is None:
            state = MarketState(
                price_log=0.0,
                price_velocity=0.0,
                fictitious_log=0.0,
                fictitious_velocity=0.0,
                surplus_ema=surplus,
                value_ema=value,
                tick=int(tick),
            )
        else:
            state = self._advance(MarketState(**prior), wages, value, surplus, defines, int(tick))
        metadata[MARKET_ATTR] = state.model_dump()

    @staticmethod
    def _advance(
        prior: MarketState,
        wages: float,
        value: float,
        surplus: float,
        defines: object,
        tick: int,
    ) -> MarketState:
        """One deterministic oscillator step of both scissors."""
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
        )
```

(Note: `defines: object` with attribute access — check how `wealth_distribution.py` types its
defines param (`Any`) and match; if MyPy strict complains, use `MarketDefines` under
TYPE_CHECKING import instead.)

- [ ] **Step 4: Register** in `simulation_engine.py`:

```python
    SovereigntySystem(),  # 17.5. ...existing comment...
    MarketScissorsSystem(),  # 17.8. Price⟷value scissors (Program 23 Phase-1 shadow; writes only its own axis, runs just before the registry measures it)
    ContradictionSystem(),  # 18. Tension aggregation
```

Add `MarketScissorsSystem` to `CONSEQUENCE_SYSTEMS` (import-time RuntimeError enforces this).
Update `tests/unit/engine/test_system_order.py`: count 29→30, `expected_order` gains
`"Market Scissors"` before `"Contradiction Tension"`.
- [ ] **Step 5: Run** — the new file + `test_system_order.py` + `test_contradiction_system.py` green.
- [ ] **Step 6: Commit** — `feat(engine): MarketScissorsSystem @17.8 — shadow price/fictitious oscillators (Program 23)`

---

### Task 7: The `price_value` opposition binding

**Files:**
- Modify: `src/babylon/domain/dialectics/instances/catalog.py` (GraphInputs field, measure,
  binding), `src/babylon/engine/systems/contradiction.py` (`_build_graph_inputs` computes the
  balance from `G.graph["market"]` + defines scale — change its signature to take `services`)
- Test: `tests/unit/dialectics/test_catalog.py` (binding count/keys/shadow),
  `tests/unit/engine/systems/test_contradiction_system.py` (end-to-end: market metadata →
  shadow state with expected gap/balance)

**Interfaces:**
- Consumes: `G.graph["market"]["price_log"]` (Task 6), `calculate_scissors_balance` (Task 4),
  `services.defines.market.scissors_balance_scale` (Task 3), shadow routing (Tasks 1–2).
- Produces: `GraphInputs.market_balance: float | None = None`; registry key `"price_value"`
  (shadow); shadow state visible at `graph.get_graph_attr("shadow_opposition_states")["price_value"]`.

- [ ] **Step 1: Failing tests.**
  - `test_catalog.py`: `build_default_registry().keys == ("atomization", "capital_labor",
    "imperial", "price_value", "tenancy", "wage")`; `shadow_keys == frozenset({"price_value"})`;
    `_price_value_measure(GraphInputs()) == GapReading(gap=0.0, balance=0.0)` (absent market →
    no contradiction); `_price_value_measure(GraphInputs(market_balance=0.6))` → gap 0.6,
    balance 0.6; negative balance symmetric.
  - `test_contradiction_system.py::TestShadowChannel` extension: set
    `graph.graph["market"] = {..., "price_log": 0.5, ...}` before stepping the REAL default
    registry; assert `shadow_opposition_states["price_value"]["balance"] ==
    pytest.approx(math.tanh(0.5 / scale))` with scale from `GameDefines().market`, and
    `opposition_states` still has exactly the 5 canonical keys.
- [ ] **Step 2: Run, confirm failure.**
- [ ] **Step 3: Implement.** `catalog.py` — GraphInputs gains (docstring attribute entry too):

```python
    market_balance: float | None = field(default=None)
```

with the docstring line: `market_balance: pre-derived scissors Balance in [-1, 1] from the
Market Scissors axis (ADR077) — the engine computes tanh(price_log / scale) so the catalog
stays defines-free; None = no market axis this tick.`

Measure + binding (binding appended after `imperial` in the list; keys are sorted by the
registry anyway):

```python
def _price_value_measure(inputs: GraphInputs) -> GapReading:
    """value (A) ⇄ price (B) — the scissors as a measured adjunction defect.

    Reads the pre-derived Balance (the engine owns the tanh scale — see
    ``GraphInputs.market_balance``). ``None`` → ``(0, 0)``: no market axis,
    no contradiction (a phenomenal form cannot diverge from an absent
    substance). Positive balance = price above value — the form pole
    dominant, fictitious validation outrunning production.
    """
    if inputs.market_balance is None:
        return GapReading(gap=0.0, balance=0.0)
    balance = max(-1.0, min(1.0, inputs.market_balance))
    return GapReading(gap=abs(balance), balance=balance)
```

```python
        BoundOpposition(
            spec=OppositionSpec(
                key="price_value",
                pole_a="value",
                pole_b="price",
                unity="the price-form presupposes the value it expresses; MELT is the "
                "unit of their adjunction and the scissors its measured defect "
                "(Capital Vol. I ch. 1 §3 / Vol. III ch. 10)",
                antagonistic=False,
            ),
            measure=_price_value_measure,
            shadow=True,
        ),
```

(`level_name` stays "" — unplaced; the national scissors sits on no county/bloc lattice yet.
The coupling graph is deliberately NOT extended in Phase 1 — noted in ADR077 as future work.)

`contradiction.py` — `_build_graph_inputs(self, graph)` becomes
`_build_graph_inputs(self, graph, services)` (update the one call site) and returns the extra
field; add the import `from babylon.formulas.market import calculate_scissors_balance`:

```python
        market_balance: float | None = None
        market_raw = graph.get_graph_attr("market", None)
        if isinstance(market_raw, dict) and "price_log" in market_raw:
            market_balance = calculate_scissors_balance(
                float(market_raw["price_log"]),
                scale=float(services.defines.market.scissors_balance_scale),
            )
```

and `market_balance=market_balance` in the `GraphInputs(...)` construction.
**Verify first** that `graph.get_graph_attr("market", None)` reads the same store
`metadata["market"]` writes (BabylonGraph.graph dict vs get_graph_attr) — if not, read via
`getattr(graph, "graph", {}).get("market")` matching the writer.
- [ ] **Step 4: Run** dialectics + engine systems suites green.
- [ ] **Step 5: Commit** — `feat(dialectics): price_value shadow opposition — the scissors registered (Program 23)`

---

### Task 8: Full gate + annotations

- [ ] **Step 1:** `mise run check` — lint + format + typecheck + test:unit all green. Fix fallout
  (docstring RST, mypy strictness, B905 zips) without weakening tests.
- [ ] **Step 2:** `mise run qa:regression` — MUST be byte-identical. The shadow axis writes new
  graph attrs but no sampled field. If ANY value moves: STOP, investigate which system perturbed
  sampled state; do NOT regenerate baselines.
- [ ] **Step 3:** Update `ai/architecture.yaml` systems annotation (17.8 Market Scissors entry,
  count 30) and `CLAUDE.md` engine section ("28 Systems" → "30 Systems, + Substrate @2.5";
  mention Market Scissors @17.8 shadow alongside Doctrine @14.7 and the P21/EH shadows).
- [ ] **Step 4: Commit** — `docs(arch): record MarketScissorsSystem @17.8 (30 systems)`

---

### Task 9: Persistence + bridge exposure (the storytelling backend)

**Files:**
- Create: `src/babylon/persistence/migrations/00NN_market_scissors_summary.sql` (next free NN —
  `ls src/babylon/persistence/migrations/` first; content: idempotent
  `ALTER TABLE tick_summary ADD COLUMN IF NOT EXISTS price_log DOUBLE PRECISION;` +
  `ADD COLUMN IF NOT EXISTS fictitious_log DOUBLE PRECISION;`)
- Modify: `src/babylon/persistence/postgres_schema.py` (tick_summary CREATE TABLE ~374 gains the
  two nullable columns — verify how ensure_ddl_applied treats an edited base DDL + migration
  pair; follow whatever the 0011→healing precedent does, never bare-loop),
  `src/babylon/persistence/postgres_runtime/_legacy.py` (`persist_tick_summary` INSERT/UPSERT +
  `query_tick_summary_series` SELECT gain both columns),
  `web/game/engine_bridge.py` (`_build_tick_summary` ~7005 adds
  `"price_log": state.market.price_log if state.market else None` and same for fictitious;
  `get_game_timeseries` ~2568 maps `value_produced` ← total_v, `surplus` ← total_s,
  `profit_rate` ← profit_rate, `price_index` ← exp(price_log) (None-safe),
  `fictitious_ratio` ← exp(fictitious_log) (None-safe))
- Test: extend the existing tick-summary integration tests
  (`rg -l "persist_tick_summary" tests/`) with the two columns; unit-test the bridge mapping if
  a `_build_tick_summary` test exists (`rg -l "_build_tick_summary" tests/ web/`).

Notes: honest NULL when the axis is absent — never 0.0. SQLite `runtime_db.record_tick_summary`
is a different, explicit-args API not on the web timeseries path — leave untouched (mention in
ADR). New columns are runtime-generated, not reference data: no data-catalog.yaml entry
(ADR075 sentinel keys on reference tables — verify the sentinel test passes untouched).

- [ ] Steps: failing test → migrate/implement → `mise run test:q --` scoped → integration leg if
  PG available locally (`mise run db:sql -- "SELECT price_log FROM tick_summary LIMIT 1"` smoke) →
  commit `feat(persistence): tick_summary carries the scissors (price_log/fictitious_log)`.

---

### Task 10: Frontend — the scissors chart

**Files:**
- Modify: `src/frontend/src/types/game.ts` (`TimeseriesPayload` += `value_produced`, `surplus`,
  `profit_rate`, `price_index`, `fictitious_ratio` — all `(number | null)[]`),
  `src/frontend/src/components/timeseries/TimeseriesChart.tsx` (extend `ChartRow` + `toChartRows`)
- Create: `src/frontend/src/components/timeseries/ScissorsChart.tsx` — a second `LineChart` over
  the same panel data: `price_index` and `fictitious_ratio` lines around a `ReferenceLine y={1}`
  labeled "value" (the substance baseline); `connectNulls={false}` (honest gaps); CSS-var strokes
  consistent with the existing chart (crimson for price, gold for fictitious per the ksbc
  palette vars already in `theme/`).
- Modify: whichever view renders `<TimeseriesChart>` (`rg -l "TimeseriesChart" src/frontend/src`)
  to render `<ScissorsChart>` beside it.
- Test: match existing frontend test conventions for chart components
  (`ls src/frontend/src/components/timeseries/*.test.*`); if none exist for TimeseriesChart,
  add none (convention-following, not convention-setting).

- [ ] Steps: types → toChartRows → component → render site → `cd src/frontend && npm run build`
  (or the mise task — check `mise tasks | grep -i front`) green → commit
  `feat(frontend): the scissors chart — price/fictitious vs the value baseline (Program 23)`.

---

### Task 11: Governance artifacts

- [ ] **ADR077** — `ai/decisions/ADR077_market_scissors.yaml`, exact ADR076 schema (status
  "accepted", date 2026-07-17, title, context/decision/consequences/evidence block scalars).
  Decision must record: recording-ADR rationale (poles derive from GapReading/OppositionState —
  Amendment S trigger does not fire; Amendment T NOT touched, sigma_authored NOT used); the
  generic shadow flag as the reusable slice of ADR072's blueprint (chauvinism⟷internationalism
  registers on the same mechanism later); Phase-2 feedback (correction → rupture, wealth-axis
  coupling) owner-gated with the promotion ceremony (regenerated baselines + authorizing ADR);
  deliberately deferred: coupling-graph edge, per-territory/sectoral scissors, pole_measure,
  FRED empirical anchors for the coefficients, diegetic narrator ticker (narration panel is
  owner-frozen unwired).
- [ ] **index.yaml** — prepend ADR077 entry (title/status/date/file), bump meta.version to
  1.25.0, meta.updated 2026-07-17.
- [ ] **spec-115** — `specs/115-market-scissors/spec.md`: the design from this plan's header +
  the phase table (observation now, feedback Phase 2) + the UI storytelling intent (X-ray lens
  framing, dramatic irony) so the vision survives the session.
- [ ] **ai/state.yaml** — prepend a `recently_completed` entry (dense prose, newest-first).
- [ ] **Commit** — `docs(governance): ADR077 + spec-115 — Program 23 Market Scissors recorded`

---

### Task 12: Final verification + PR

- [ ] `mise run check` green; `mise run qa:regression` byte-identical; defines `--check` green.
- [ ] Re-read the full diff (`git diff dev --stat` then review each file) for observe-only
  violations: nothing outside tests/bridge reads `shadow_opposition_states` / `"market"`.
- [ ] Push branch, open PR to `dev` (gh is owner-delegated) titled
  `feat: Program 23 — Market Scissors (price⟷value shadow opposition, ADR077)` with the
  standard generated-with footer; do NOT merge — owner reviews in the morning.
- [ ] Update auto-memory (`market-scissors-program-23.md` + MEMORY.md line).

## Self-Review Notes

- Spec coverage: shadow mechanism (T1–2), defines (T3), math (T4), state (T5), system (T6),
  opposition (T7), gates (T8), persistence (T9), UI (T10), governance (T11), delivery (T12).
  Deferred-by-design items are recorded in ADR077, not silently dropped.
- Type consistency: `MarketState` field names are identical across model, system writer,
  bridge reader, and SQL columns (`price_log`/`fictitious_log`).
- Known verify-at-execution points (flagged inline): get_graph_attr vs `.graph` store identity
  (T7), ensure_ddl_applied + edited base DDL interplay (T9), frontend build task name (T10),
  models `__init__` export (T5).
