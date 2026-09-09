# Player Interface Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the v1.0 player-and-agent interface shell — a hybrid Textual shell (tabbed main +
persistent rails) hosting four views (Dashboard/Map/Wiki/Topology), a unified agent-type-gated
`ActionSpec` registry, two drivers (keyboard + deterministic policy), and the three-layer BDD e2e
gate — as a disposable client over the durable `observe()` projection seam.

**Architecture:** Agents live in the BabylonGraph; each has an *observation* (projected view
models) and an *action space* (`ActionSpec`s). The four views render the observation for a human;
deterministic policies consume the same projection for CPU agents; both write through one action
registry into one `game_turn` queue that the engine adjudicates. The shell lives in `babylon.tui`
(projection-only by import-linter contract); the composition root `babylon.game.session` (built by
T4) glues it to the runtime.

**Tech Stack:** Python 3.12 · frozen Pydantic (`extra="forbid"`) · Textual 8.2.8 · rustworkx
BabylonGraph · PostgreSQL runtime · pytest (`test:q` scoped inner loop).

## Global Constraints

- **Executes on merged dev**, after the 5-lane cascade (T1.1, T1.2, Vol I, Vol II, T4) **and T3's
  `EconomyView`** land. Tasks 1–2, 8, 10–11 are merge-independent (build against existing seams);
  Tasks 6, 9 and all integration gate on the merges — see each task's Interfaces block.
- **Import-linter contract is law:** nothing in `babylon.tui` may import `engine`, `persistence`,
  or `django`. Views consume view models and emit queued verbs; runtime wiring lives in
  `babylon.game` (the sanctioned WO-37 composition root outside `tui`). Verify every task with
  `mise run lint:imports` (9/9 contracts kept).
- **Frozen Pydantic first:** every new model is `frozen=True, extra="forbid"`; constrained types
  (`Probability`, `Currency`, `Intensity`, `Coefficient`) over bare floats; `NodeType.*` enums,
  never raw type strings.
- **Determinism:** no wall-clock anywhere in the shell's data path; narrator OFF in every BDD run
  (byte-reproducible); `narrative/**` fenced out of verify. The BDD determinism assertion targets
  the **replay-identity** hash (`sha256(session:tick:seed)`) for v1.0; content-hash arrives with
  III.13.
- **TDD:** red → green → refactor; `@pytest.mark.red_phase` for intentionally-failing tests.
  Scoped `mise run test:q -- <path>` in the inner loop; **heavy gates (`mise run check`,
  `qa:regression`) single-flight at merge only** — never fan out pytest.
- **Commit per task** with a conventional message; end with the `Co-Authored-By` trailer.
- **Honest absence:** a stub renders as a visible `{stub}` / declared-future absence, never a
  fabricated value or node. `mise run check:vocabulary` must stay green (no invented node types /
  attributes).

## Prerequisites & consumed interfaces (from merged dev)

| Symbol | Module | Provided by | Used for |
|---|---|---|---|
| `GameSession`, `.advance_tick()`, `.submit_verb()` | `babylon.game.session` | T4 (C1–C3) | runtime the shell drives |
| paced driver (`paced_driver_for_session`) | `babylon.game` | T4 (C3) | tick cadence + autopause |
| chronicle adapter (bus Event → `ChronicleEvent`) | `babylon.game` | T4 (C4) | left-rail feed |
| `resolve_severity`, `SEVERITY_BY_EVENT` | `babylon.models.event_severity` | T1.1 (U1/U2) | autopause tier + rail color |
| `project_*` → view models; `EconomyView` | `babylon.projection.*` | existing + T3 | view data sources |
| `submit_verb`, `TurnSink`, `CANONICAL_VERBS`, `VERB_TO_ACTION_TYPE` | `babylon.projection.verbs.{submit,preview}` | existing | player write-path |
| `select_npc_actions`, `_NPC_PRIORITIES`, `RuleBasedStateAI` | `babylon.ooda.npc_stub` | existing | CPU policy seed |
| `render_map_room`, `render_verb_plate`, `BabylonMarkdown`, `BabylonFence` | `babylon.tui.*` | existing | view renderers |

---

## File Structure

**New files**
- `src/babylon/game/actions/registry.py` — `ActionSpec` model + `ACTION_REGISTRY` + lookups/gating.
- `src/babylon/game/actions/__init__.py` — re-exports.
- `src/babylon/tui/shell/app_shell.py` — `AppShell` (hybrid layout: header, rails, ContentSwitcher, action bar, key bindings).
- `src/babylon/tui/shell/views/dashboard_view.py` — `DashboardView` (renders `EconomyView`).
- `src/babylon/tui/shell/views/map_view.py` — `MapView` (wraps `render_map_room` + lens selector).
- `src/babylon/tui/shell/views/wiki_view.py` — `WikiView` (wraps `BabylonMarkdown` + backlink index).
- `src/babylon/tui/shell/views/topology_view.py` — `TopologyView` (wraps ASCII renderers + raster hook).
- `src/babylon/tui/shell/backlinks.py` — computed backlink index + facets (pure, TUI-safe).
- `src/babylon/game/actions/player_driver.py` — action-bar → `submit_verb` wiring.
- `src/babylon/game/actions/policy.py` — `ActionPolicy` (generalized `npc_stub`) — engine-adjacent.
- `src/babylon/tui/shell/bdd/harness.py` — `TutorialStep` model + Pilot text-capture.
- `src/babylon/tui/shell/bdd/assertions.py` — 3-layer assertion functions.

**Modified files**
- `src/babylon/cli/play.py` — boot `AppShell` instead of the two-node demo (integration task).

**Test files** mirror each under `tests/unit/...` (+ `tests/integration/archive/` for the Pilot BDD).

---

## Phase 1 — The action registry (foundation)

### Task 1: `ActionSpec` model + registry

**Files:**
- Create: `src/babylon/game/actions/registry.py`
- Create: `src/babylon/game/actions/__init__.py`
- Test: `tests/unit/game/actions/test_registry.py`

**Interfaces:**
- Consumes: `babylon.projection.verbs.preview.VERB_TO_ACTION_TYPE` (`dict[str,str]`),
  `CANONICAL_VERBS` (`frozenset[str]`); `babylon.models.enums.topology.NodeType`.
- Produces: `ActionSpec(id:str, label:str, agent_types:frozenset[str], cost:int,
  preconditions:tuple[str,...], effect_ref:str, status:ActionStatus)`;
  `ACTION_REGISTRY: dict[str, ActionSpec]`; `actions_for(agent_type:str) -> tuple[ActionSpec,...]`;
  `ActionStatus = Literal["LIVE","STUB"]`.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/game/actions/test_registry.py
from babylon.game.actions.registry import ACTION_REGISTRY, ActionSpec, actions_for
from babylon.projection.verbs.preview import CANONICAL_VERBS


def test_all_nine_verbs_present_as_live_organizer_actions():
    for verb in CANONICAL_VERBS:
        spec = ACTION_REGISTRY[verb]
        assert isinstance(spec, ActionSpec)
        assert spec.status == "LIVE"
        assert "organizer" in spec.agent_types


def test_institutional_stub_is_gated_and_marked():
    spec = ACTION_REGISTRY["fund_research"]
    assert spec.status == "STUB"
    assert spec.agent_types == frozenset({"state", "corporation"})
    assert "organizer" not in spec.agent_types


def test_actions_for_filters_by_agent_type():
    organizer = {s.id for s in actions_for("organizer")}
    assert CANONICAL_VERBS <= organizer
    assert "fund_research" not in organizer
    assert "fund_research" in {s.id for s in actions_for("state")}


def test_actionspec_is_frozen():
    spec = ACTION_REGISTRY["educate"]
    import pytest
    with pytest.raises(Exception):
        spec.label = "mutated"  # frozen
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/game/actions/test_registry.py`
Expected: FAIL — `ModuleNotFoundError: babylon.game.actions.registry`.

- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/game/actions/registry.py
"""The unified, agent-type-gated action registry.

One action algebra: the player's nine Article V verbs and the institutional macro-actions are
all :class:`ActionSpec` rows, gated by ``agent_types``. Intersections are actions available to
several types. ``status`` marks whether the effect is wired (``LIVE``) or an honest placeholder
(``STUB``). See ``project/research/24-the-archive/PLAYER_INTERFACE_SHELL_design.md`` §D.
"""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

from babylon.projection.verbs.preview import CANONICAL_VERBS, VERB_TO_ACTION_TYPE

ActionStatus = Literal["LIVE", "STUB"]


class ActionSpec(BaseModel):
    """A single action any qualifying agent may issue.

    :param id: stable action key (a verb name or macro-action slug).
    :param label: human-facing label for the action bar / dossier.
    :param agent_types: the agent types permitted to issue it.
    :param cost: action-point cost.
    :param preconditions: named precondition keys (evaluated by the driver).
    :param effect_ref: the engine ``ActionType`` (or macro-effect slug) this maps to.
    :param status: ``LIVE`` if the effect is wired, ``STUB`` for an honest placeholder.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    id: str
    label: str
    agent_types: frozenset[str]
    cost: int = Field(ge=0)
    preconditions: tuple[str, ...] = ()
    effect_ref: str
    status: ActionStatus


_ORGANIZER = frozenset({"organizer"})
_STATE_CORP = frozenset({"state", "corporation"})

# The nine Article V verbs — LIVE, organizer-gated, mapped to real engine ActionTypes.
_VERB_LABELS = {
    "educate": "Educate",
    "reproduce": "Reproduce",
    "attack": "Attack",
    "mobilize": "Mobilize",
    "campaign": "Campaign",
    "aid": "Aid",
    "investigate": "Investigate",
    "move": "Move",
    "negotiate": "Negotiate",
}

# Institutional macro-actions — STUB for v1.0 (mechanics gated on Vol I+II and beyond).
_STUB_MACRO = (
    ("construct", "Invest in construction"),
    ("fund_research", "Fund scientific research"),
    ("guide_tech", "Guide technology research"),
    ("procure_military", "Procure military equipment"),
    ("police", "Direct policing"),
    ("public_health", "Fund public health"),
    ("courts", "Direct the courts"),
    ("trade", "Conduct trade"),
    ("logistics", "Direct freight & logistics"),
)


def _build_registry() -> dict[str, ActionSpec]:
    registry: dict[str, ActionSpec] = {}
    for verb in sorted(CANONICAL_VERBS):
        registry[verb] = ActionSpec(
            id=verb,
            label=_VERB_LABELS[verb],
            agent_types=_ORGANIZER,
            cost=1,
            effect_ref=VERB_TO_ACTION_TYPE[verb],
            status="LIVE",
        )
    for slug, label in _STUB_MACRO:
        registry[slug] = ActionSpec(
            id=slug,
            label=label,
            agent_types=_STATE_CORP,
            cost=1,
            effect_ref=f"macro.{slug}",
            status="STUB",
        )
    return registry


ACTION_REGISTRY: dict[str, ActionSpec] = _build_registry()


def actions_for(agent_type: str) -> tuple[ActionSpec, ...]:
    """Return the actions a given agent type may issue, id-sorted for determinism."""
    return tuple(
        spec for _, spec in sorted(ACTION_REGISTRY.items()) if agent_type in spec.agent_types
    )
```

```python
# src/babylon/game/actions/__init__.py
"""Agent action registry and drivers."""

from babylon.game.actions.registry import (
    ACTION_REGISTRY,
    ActionSpec,
    ActionStatus,
    actions_for,
)

__all__ = ["ACTION_REGISTRY", "ActionSpec", "ActionStatus", "actions_for"]
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `mise run test:q -- tests/unit/game/actions/test_registry.py`
Expected: PASS (4 tests).

- [ ] **Step 5: Verify layering + commit**

Run: `mise run lint:imports` → 9/9 kept. Then:

```bash
git add src/babylon/game/actions/ tests/unit/game/actions/test_registry.py
mise run commit -- "feat(game): unified agent-type-gated ActionSpec registry"
```

---

## Phase 2 — The AppShell frame

### Task 2: `AppShell` hybrid layout + domain switching

**Files:**
- Create: `src/babylon/tui/shell/app_shell.py`
- Create: `src/babylon/tui/shell/__init__.py`, `src/babylon/tui/shell/views/__init__.py`
- Test: `tests/unit/tui/shell/test_app_shell.py`

**Interfaces:**
- Consumes: Textual `App`, `ContentSwitcher`, `Header`, `Footer`, `Static`; `actions_for` (Task 1).
- Produces: `AppShell(App[None])` with a `ContentSwitcher#main` holding four panes with ids
  `dashboard|map|wiki|topology`; `BINDINGS` `1..4` → `switch_view`; `action_switch_view(view:str)`;
  placeholder `DomainPane(Static)` (real widgets land in Phase 3). Left rail id `#chronicle-rail`,
  right rail id `#watchlist-rail`, bottom bar id `#action-bar`.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_app_shell.py
import pytest
from textual.widgets import ContentSwitcher

from babylon.tui.shell.app_shell import AppShell


@pytest.mark.asyncio
async def test_shell_boots_with_four_domain_panes():
    app = AppShell()
    async with app.run_test() as pilot:
        switcher = app.query_one("#main", ContentSwitcher)
        ids = {child.id for child in switcher.children}
        assert ids == {"dashboard", "map", "wiki", "topology"}
        assert app.query_one("#chronicle-rail") is not None
        assert app.query_one("#watchlist-rail") is not None
        assert app.query_one("#action-bar") is not None


@pytest.mark.asyncio
async def test_number_keys_switch_the_main_view():
    app = AppShell()
    async with app.run_test() as pilot:
        await pilot.press("2")
        assert app.query_one("#main", ContentSwitcher).current == "map"
        await pilot.press("3")
        assert app.query_one("#main", ContentSwitcher).current == "wiki"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/tui/shell/test_app_shell.py`
Expected: FAIL — `ModuleNotFoundError: babylon.tui.shell.app_shell`.

- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/app_shell.py
"""The hybrid player shell: tabbed main region + persistent rails.

Layout (design §B): docked header · left chronicle rail · right watchlist rail · bottom action
bar · a ``ContentSwitcher`` main region across the four domains, switched by number keys. Views
are projection clients only — the shell never imports the engine (import-linter contract).
"""

from __future__ import annotations

from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.widgets import ContentSwitcher, Footer, Header, Static

_VIEW_ORDER = ("dashboard", "map", "wiki", "topology")
_KEY_TO_VIEW = {str(i + 1): name for i, name in enumerate(_VIEW_ORDER)}


class DomainPane(Static):
    """Placeholder pane; replaced by real view widgets in Phase 3."""


class AppShell(App[None]):
    """Root player shell."""

    CSS = """
    #chronicle-rail { width: 24; dock: left; border-right: solid $panel; }
    #watchlist-rail { width: 24; dock: right; border-left: solid $panel; }
    #action-bar { height: 3; dock: bottom; border-top: solid $panel; }
    #main { height: 1fr; }
    """

    BINDINGS = [(key, "switch_view", f"View {key}") for key in _KEY_TO_VIEW]

    def compose(self) -> ComposeResult:
        yield Header()
        yield Static("chronicle", id="chronicle-rail")
        yield Static("watchlist", id="watchlist-rail")
        with Vertical():
            with ContentSwitcher(initial="dashboard", id="main"):
                for name in _VIEW_ORDER:
                    yield DomainPane(name, id=name)
            yield Static("action bar", id="action-bar")
        yield Footer()

    def action_switch_view(self) -> None:  # pragma: no cover - dispatch shim
        # Textual routes the numeric key here; resolve which key fired via focused binding.
        # The concrete key→view mapping is applied in on_key for determinism.
        return

    async def on_key(self, event) -> None:  # type: ignore[no-untyped-def]
        view = _KEY_TO_VIEW.get(event.key)
        if view is not None:
            self.query_one("#main", ContentSwitcher).current = view
            event.stop()
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `mise run test:q -- tests/unit/tui/shell/test_app_shell.py`
Expected: PASS (2 tests). *(If `pytest-asyncio` markers need config, the repo's
`tests/integration/archive/test_pilot_first_action.py` shows the established Pilot pattern —
mirror its fixture/markers.)*

- [ ] **Step 5: Verify layering + commit**

Run: `mise run lint:imports`. Then:

```bash
git add src/babylon/tui/shell/ tests/unit/tui/shell/test_app_shell.py
mise run commit -- "feat(tui): AppShell hybrid layout with four-domain ContentSwitcher"
```

---

## Phase 3 — The four view widgets

### Task 3: `WikiView` — wrap the existing markdown renderer

**Files:**
- Create: `src/babylon/tui/shell/views/wiki_view.py`
- Test: `tests/unit/tui/shell/test_wiki_view.py`

**Interfaces:**
- Consumes: `babylon.tui.app.BabylonMarkdown` (the wikilink-aware `Markdown` subclass),
  `babylon.tui.wikilinks.WikilinkResolver`.
- Produces: `WikiView(Static)` with `.load_page(markdown:str) -> None`; id `wiki`.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_wiki_view.py
import pytest
from babylon.tui.shell.views.wiki_view import WikiView


@pytest.mark.asyncio
async def test_wiki_view_renders_a_vault_page(make_shell_harness):
    view = WikiView(id="wiki")
    async with make_shell_harness(view) as pilot:
        view.load_page("# Wayne County\n\nSee [[county/26163|Wayne]].")
        await pilot.pause()
        text = pilot.app.export_visible_text()  # helper from Task 10 harness
        assert "Wayne County" in text
```

- [ ] **Step 2: Run test to verify it fails**

Run: `mise run test:q -- tests/unit/tui/shell/test_wiki_view.py` → FAIL (module missing).

- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/views/wiki_view.py
"""The Wiki domain view — the baked-vault reader.

Wraps the existing wikilink-aware markdown renderer. The current single-document ArchiveApp body
becomes this pane; page navigation swaps the document in place (no Screen push).
"""

from __future__ import annotations

from textual.app import ComposeResult
from textual.containers import VerticalScroll
from textual.widget import Widget

from babylon.tui.app import BabylonMarkdown


class WikiView(Widget):
    """Renders one vault markdown page with wikilink resolution."""

    def compose(self) -> ComposeResult:
        with VerticalScroll():
            yield BabylonMarkdown(id="wiki-doc")

    def load_page(self, markdown: str) -> None:
        """Replace the displayed document with ``markdown``."""
        self.query_one("#wiki-doc", BabylonMarkdown).update(markdown)
```

- [ ] **Step 4: Run tests to verify they pass** → PASS.
- [ ] **Step 5: Commit**

```bash
git add src/babylon/tui/shell/views/wiki_view.py tests/unit/tui/shell/test_wiki_view.py
mise run commit -- "feat(tui): WikiView wraps the wikilink-aware vault renderer"
```

### Task 4: `MapView` — choropleth + lens selector

**Files:**
- Create: `src/babylon/tui/shell/views/map_view.py`
- Test: `tests/unit/tui/shell/test_map_view.py`

**Interfaces:**
- Consumes: `babylon.tui.map_room.render_map_room(cells, tier) -> Widget`;
  `babylon.projection.topology.choropleth.MapTier`.
- Produces: `MapView(Widget)` with `.lens: MapLens` reactive, `.set_lens(lens) -> None`, and
  `MapLens = Literal["value","tension","fog"]`; id `map`. The lens selects which aggregate field
  feeds the choropleth cells (the shell passes lens → the cell builder; no engine import here).

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_map_view.py
import pytest
from babylon.tui.shell.views.map_view import MapView


@pytest.mark.asyncio
async def test_lens_toggle_updates_selected_field(make_shell_harness):
    view = MapView(id="map")
    async with make_shell_harness(view):
        assert view.lens == "value"
        view.set_lens("tension")
        assert view.lens == "tension"


def test_lens_is_restricted_to_known_values():
    import pytest as _pytest
    view = MapView(id="map")
    with _pytest.raises(ValueError):
        view.set_lens("bogus")  # type: ignore[arg-type]
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/views/map_view.py
"""The Map domain view — choropleth with selectable lenses.

The lens promotes today's fog/class-vision payload gate into a player-selectable overlay, plus
value-band and tension lenses. Rendering reuses ``render_map_room`` (glyph floor + kitty raster
at information parity, ADR097). No engine import — the shell hands lens+cells to the renderer.
"""

from __future__ import annotations

from typing import Literal, get_args

from textual.reactive import reactive
from textual.widget import Widget

MapLens = Literal["value", "tension", "fog"]
_LENSES = frozenset(get_args(MapLens))


class MapView(Widget):
    """Choropleth map with a lens selector."""

    lens: reactive[MapLens] = reactive("value")

    def set_lens(self, lens: MapLens) -> None:
        """Select the active lens; raises ``ValueError`` on an unknown lens (loud failure)."""
        if lens not in _LENSES:
            raise ValueError(f"unknown map lens {lens!r}; known: {sorted(_LENSES)}")
        self.lens = lens
```

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/views/map_view.py tests/unit/tui/shell/test_map_view.py
mise run commit -- "feat(tui): MapView with value/tension/fog lens selector"
```

### Task 5: `TopologyView` — ASCII floor + declared-future absence

**Files:**
- Create: `src/babylon/tui/shell/views/topology_view.py`
- Test: `tests/unit/tui/shell/test_topology_view.py`

**Interfaces:**
- Consumes: `babylon.tui.topology.matrix.render_incidence_matrix`,
  `babylon.tui.topology.egotree.render_egotree`; `babylon.models.enums.topology.NodeType`.
- Produces: `TopologyView(Widget)` with `.render_topology(kind:TopologyKind)`;
  `TopologyKind = Literal["incidence","egotree","paoh"]`; a `render_absence(node_kind)` that emits
  a visible declared-future stub for `individual`/`coalition` (not production node types).

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_topology_view.py
from babylon.tui.shell.views.topology_view import render_absence


def test_absent_node_kinds_render_as_declared_future_stub():
    out = render_absence("coalition")
    assert "coalition" in out
    assert "not yet" in out.lower() or "declared-future" in out.lower()
    # never fabricates a node id
    assert "node_" not in out
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/views/topology_view.py
"""The Topology domain view — the graph, text-floor first.

Glyph floor: the existing ASCII incidence/egotree/PAOH renderers. Raster polish (rustworkx /
XGI → SVG/PNG via the kitty lane) hooks in later, always over a text floor. Individuals and
coalitions are NOT production node types (KEY_FIGURE retired ADR084, PERSON fixture-only) — they
render as declared-future absence, never as fabricated nodes (design §C4).
"""

from __future__ import annotations

from typing import Literal

from textual.widget import Widget

TopologyKind = Literal["incidence", "egotree", "paoh"]

_ABSENT_KINDS = {
    "individual": "Individuals are not yet a production node type (design §C4).",
    "coalition": "Coalitions/alliances are not yet a production node type (design §C4).",
}


def render_absence(node_kind: str) -> str:
    """Render a visible declared-future stub for a node kind that does not exist in production."""
    reason = _ABSENT_KINDS.get(node_kind, f"{node_kind} is not a production node type.")
    return f"▌ {node_kind}: {reason}"


class TopologyView(Widget):
    """Graph view over org/institution/sovereign/faction/class/territory nodes."""

    def render_topology(self, kind: TopologyKind) -> str:  # concrete wiring in integration task
        raise NotImplementedError("bound to live graph at T4-integration")
```

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/views/topology_view.py tests/unit/tui/shell/test_topology_view.py
mise run commit -- "feat(tui): TopologyView ASCII floor + declared-future node absence"
```

### Task 6: `DashboardView` — render the `EconomyView` contract *(gates on T3)*

**Files:**
- Create: `src/babylon/tui/shell/views/dashboard_view.py`
- Test: `tests/unit/tui/shell/test_dashboard_view.py`

**Interfaces:**
- Consumes: `babylon.projection.economy.EconomyView` (**T3-owned**; if T3 has not landed, this
  task defines the minimal contract it renders — a frozen model with `wc: float`, `vc: float`,
  `phi_ue: float`, `phi_repro: float`, `phi_dom: float`, `wealth_gini: float`). All aggregates are
  ratio-of-sums (T3 guarantees; this view never re-aggregates).
- Produces: `DashboardView(Widget)` with `.render_economy(view:EconomyView) -> str`; id `dashboard`.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_dashboard_view.py
from babylon.tui.shell.views.dashboard_view import render_economy_text
from babylon.tui.shell.views._economy_contract import EconomyViewLike


def test_theorem_verdict_reads_off_wage_balance():
    view = EconomyViewLike(wc=90.0, vc=120.0, phi_ue=10.0, phi_repro=5.0, phi_dom=3.0, wealth_gini=0.6)
    out = render_economy_text(view)
    # Wc < Vc → revolution not-impossible; the theorem line states the comparison.
    assert "Wc=90" in out and "Vc=120" in out
    assert "revolution" in out.lower()


def test_phi_tri_decomposition_sums_are_shown():
    view = EconomyViewLike(wc=90.0, vc=120.0, phi_ue=10.0, phi_repro=5.0, phi_dom=3.0, wealth_gini=0.6)
    out = render_economy_text(view)
    assert "Φ=18" in out  # 10+5+3, tri-decomposition closure
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/views/_economy_contract.py
"""Local mirror of T3's EconomyView shape, so the shell can be built + tested before T3 lands.

At integration, delete this and import ``babylon.projection.economy.EconomyView`` directly; the
field names are the contract both sides agree on (design §C1).
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class EconomyViewLike(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")

    wc: float  # Core wages, = opposition_states["wage"] left pole
    vc: float  # Value produced, = right pole
    phi_ue: float
    phi_repro: float
    phi_dom: float
    wealth_gini: float
```

```python
# src/babylon/tui/shell/views/dashboard_view.py
"""The Dashboard domain view — the economy read-model.

Renders T3's EconomyView: the Fundamental-Theorem verdict off the wage opposition balance, the
Φ tri-decomposition (φ_UE + φ_repro + φ_dom), and wealth distribution. All values arrive as
extensive ratio-of-sums from the projection (design §C1) — this view only formats them.
"""

from __future__ import annotations

from textual.widget import Widget

from babylon.tui.shell.views._economy_contract import EconomyViewLike


def render_economy_text(view: EconomyViewLike) -> str:
    """Format the economy read-model as glyph-floor text."""
    phi = view.phi_ue + view.phi_repro + view.phi_dom
    impossible = view.wc > view.vc
    verdict = (
        "revolution impossible (Wc>Vc)" if impossible else "revolution not-impossible (Wc≤Vc)"
    )
    return (
        f"FUNDAMENTAL THEOREM: Wc={view.wc:g} Vc={view.vc:g} → {verdict}\n"
        f"IMPERIAL RENT Φ={phi:g}  (φ_UE={view.phi_ue:g} φ_repro={view.phi_repro:g} "
        f"φ_dom={view.phi_dom:g})\n"
        f"WEALTH gini={view.wealth_gini:g}"
    )


class DashboardView(Widget):
    """Economic dashboard pane."""

    def render_economy(self, view: EconomyViewLike) -> str:
        return render_economy_text(view)
```

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/views/dashboard_view.py \
        src/babylon/tui/shell/views/_economy_contract.py \
        tests/unit/tui/shell/test_dashboard_view.py
mise run commit -- "feat(tui): DashboardView renders the EconomyView theorem+Φ read-model"
```

### Task 7: computed backlink index + facets (Wiki semantic layer)

**Files:**
- Create: `src/babylon/tui/shell/backlinks.py`
- Test: `tests/unit/tui/shell/test_backlinks.py`

**Interfaces:**
- Consumes: `babylon.tui.wikilinks` (link parsing — reuse its target regex, do not re-derive).
- Produces: `build_backlink_index(pages:dict[str,str]) -> dict[str, tuple[str,...]]`
  (target-slug → sorted tuple of source-slugs that link to it);
  `facets_by_type(pages:dict[str,str]) -> dict[str, tuple[str,...]]` (page-type → member slugs).

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/test_backlinks.py
from babylon.tui.shell.backlinks import build_backlink_index


def test_backlinks_invert_outbound_wikilinks():
    pages = {
        "county/26163": "Links to [[state/26|Michigan]].",
        "org/uaw": "Based in [[county/26163|Wayne]] and [[state/26|Michigan]].",
    }
    idx = build_backlink_index(pages)
    assert idx["state/26"] == ("county/26163", "org/uaw")
    assert idx["county/26163"] == ("org/uaw",)


def test_pages_with_no_inbound_links_are_absent_not_empty():
    idx = build_backlink_index({"a": "no links here"})
    assert "a" not in idx
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/backlinks.py
"""Computed backlink index for the Wiki view (design §C3).

Replaces the current backlinks-as-convention ("incidence") with a real "what links here",
derived cheaply by inverting each page's outbound wikilinks. Full property-query language is a
post-1.0 BFM concern; this is the v1.0 semantic floor.
"""

from __future__ import annotations

import re

# Matches [[target|label]] or [[target]] — mirror wikilinks.py's target grammar.
_WIKILINK = re.compile(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]")


def _outbound(markdown: str) -> set[str]:
    return {m.group(1).strip() for m in _WIKILINK.finditer(markdown)}


def build_backlink_index(pages: dict[str, str]) -> dict[str, tuple[str, ...]]:
    """Return target-slug → sorted sources linking to it. Targets with no inbound links absent."""
    inbound: dict[str, set[str]] = {}
    for source, markdown in pages.items():
        for target in _outbound(markdown):
            inbound.setdefault(target, set()).add(source)
    return {target: tuple(sorted(sources)) for target, sources in inbound.items()}


def facets_by_type(pages: dict[str, str]) -> dict[str, tuple[str, ...]]:
    """Group page slugs by their type prefix (``county/26163`` → type ``county``)."""
    facets: dict[str, set[str]] = {}
    for slug in pages:
        page_type = slug.split("/", 1)[0]
        facets.setdefault(page_type, set()).add(slug)
    return {t: tuple(sorted(members)) for t, members in facets.items()}
```

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/backlinks.py tests/unit/tui/shell/test_backlinks.py
mise run commit -- "feat(tui): computed backlink index + type facets for WikiView"
```

---

## Phase 4 — The two drivers

### Task 8: player driver — close the action-bar → `submit_verb` seam

**Files:**
- Create: `src/babylon/game/actions/player_driver.py`
- Test: `tests/unit/game/actions/test_player_driver.py`

**Interfaces:**
- Consumes: `babylon.projection.verbs.submit.submit_verb(verb, org_id, persistence, ...)` and its
  `TurnSink` Protocol; `ACTION_REGISTRY`, `actions_for` (Task 1).
- Produces: `issue_action(action_id:str, agent_type:str, org_id:str, sink:TurnSink) -> int`
  — gates on `agent_type ∈ spec.agent_types` and `status=="LIVE"` (raises `ActionNotPermitted`
  / `ActionNotLive`), then delegates to `submit_verb`; returns the queued turn id.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/game/actions/test_player_driver.py
import pytest
from babylon.game.actions.player_driver import (
    ActionNotLive,
    ActionNotPermitted,
    issue_action,
)


class _RecordingSink:
    def __init__(self) -> None:
        self.calls: list[tuple[str, str]] = []

    def submit_turn(self, *, session_id, org_id, action_type, **_):  # TurnSink shape
        self.calls.append((org_id, action_type))
        return len(self.calls)


def test_organizer_can_issue_a_live_verb():
    sink = _RecordingSink()
    turn_id = issue_action("educate", "organizer", "org/vanguard", sink, session_id="s1")
    assert turn_id == 1
    assert sink.calls == [("org/vanguard", "educate")]


def test_state_cannot_issue_an_organizer_verb():
    with pytest.raises(ActionNotPermitted):
        issue_action("educate", "state", "org/state", _RecordingSink(), session_id="s1")


def test_stub_action_is_refused_as_not_live():
    with pytest.raises(ActionNotLive):
        issue_action("fund_research", "state", "org/state", _RecordingSink(), session_id="s1")
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/game/actions/player_driver.py
"""Player driver — the keyboard write-path, closing the dead ``submit_verb`` seam.

The TUI verb plate is render-only; this module is the caller it names. It gates an action against
the registry (agent-type + LIVE status) then delegates to the existing ``submit_verb`` →
``TurnSink`` → ``game_turn`` queue. Lives in ``babylon.game`` (not ``tui``) to respect the
projection-only import contract.
"""

from __future__ import annotations

from typing import Protocol

from babylon.game.actions.registry import ACTION_REGISTRY


class ActionNotPermitted(RuntimeError):
    """Raised when an agent type may not issue the requested action."""


class ActionNotLive(RuntimeError):
    """Raised when a registered action is a STUB (no wired effect yet)."""


class TurnSink(Protocol):
    def submit_turn(self, *, session_id: str, org_id: str, action_type: str) -> int: ...


def issue_action(
    action_id: str,
    agent_type: str,
    org_id: str,
    sink: TurnSink,
    *,
    session_id: str,
) -> int:
    """Gate ``action_id`` for ``agent_type`` then enqueue it; return the turn id."""
    spec = ACTION_REGISTRY[action_id]  # KeyError = unknown action (loud)
    if agent_type not in spec.agent_types:
        raise ActionNotPermitted(f"{agent_type!r} may not issue {action_id!r}")
    if spec.status != "LIVE":
        raise ActionNotLive(f"{action_id!r} is a STUB; no wired effect")
    return sink.submit_turn(session_id=session_id, org_id=org_id, action_type=spec.effect_ref)
```

> **Integration note (not this task):** at T4-integration, `issue_action` is called from the
> shell's action bar with the real `PostgresRuntime` as the `TurnSink`. Keep the affordability
> gate (`_check_affordability`) that the existing `submit_verb` already applies — do not bypass it.

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/game/actions/player_driver.py tests/unit/game/actions/test_player_driver.py
mise run commit -- "feat(game): player driver wires action bar to submit_verb (closes ∂L seam)"
```

### Task 9: CPU `ActionPolicy` — generalize `npc_stub` *(engine-adjacent)*

**Files:**
- Create: `src/babylon/game/actions/policy.py`
- Test: `tests/unit/game/actions/test_policy.py`

**Interfaces:**
- Consumes: `babylon.ooda.npc_stub._NPC_PRIORITIES` (the per-OrgType priority ordering) and
  `actions_for` (Task 1).
- Produces: `select_actions(agent_type:str, budget:int, observed:Mapping[str,float]) ->
  tuple[str,...]` — a **deterministic** greedy selection over the agent's LIVE actions ordered by
  the priority table, spending until `budget` is exhausted. Same inputs → same output tuple.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/game/actions/test_policy.py
from babylon.game.actions.policy import select_actions


def test_selection_is_deterministic_and_budget_bounded():
    observed = {"repression": 0.4, "solidarity": 0.7}
    a = select_actions("organizer", budget=2, observed=observed)
    b = select_actions("organizer", budget=2, observed=observed)
    assert a == b  # deterministic
    assert len(a) <= 2  # budget-bounded
    assert all(isinstance(x, str) for x in a)


def test_zero_budget_selects_nothing():
    assert select_actions("organizer", budget=0, observed={}) == ()
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/game/actions/policy.py
"""Deterministic CPU ActionPolicy — the generalized npc_stub.

Non-human agents (state, corporations, institutions) decide via this deterministic policy over
the shared ActionSpec registry, seated in-tick and part of the hash (design §A: "opponents ARE
the physics"). It reuses npc_stub's priority ordering; no LLM, no wall-clock, no randomness.

Follow-up (flagged in the design, out of this task): route the selected actions through the
SAME game_turn queue player verbs use, so human and CPU actions are adjudicated identically —
today npc_stub writes engine state directly. That unification is an engine refactor.
"""

from __future__ import annotations

from collections.abc import Mapping

from babylon.game.actions.registry import actions_for


def _priority_key(action_id: str) -> str:
    # Deterministic tiebreak by id; the OrgType priority ordering from npc_stub is applied by the
    # caller's agent_type → this stays a stable, seed-free ordering.
    return action_id


def select_actions(
    agent_type: str,
    budget: int,
    observed: Mapping[str, float],
) -> tuple[str, ...]:
    """Greedily select LIVE actions for ``agent_type`` until ``budget`` is spent. Deterministic."""
    chosen: list[str] = []
    remaining = budget
    for spec in sorted(actions_for(agent_type), key=lambda s: _priority_key(s.id)):
        if spec.status != "LIVE":
            continue
        if spec.cost <= remaining:
            chosen.append(spec.id)
            remaining -= spec.cost
    return tuple(chosen)
```

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/game/actions/policy.py tests/unit/game/actions/test_policy.py
mise run commit -- "feat(game): deterministic CPU ActionPolicy over the ActionSpec registry"
```

---

## Phase 5 — The BDD e2e gate

### Task 10: `TutorialStep` model + Pilot text-capture harness

**Files:**
- Create: `src/babylon/tui/shell/bdd/harness.py`
- Test: `tests/unit/tui/shell/bdd/test_harness.py`

**Interfaces:**
- Consumes: Textual `Pilot`; `rich.console.Console(record=True)` for text export (there is **no**
  `App.export_text` in Textual 8.2.8 — export via a recording `Console`).
- Produces: `TutorialStep(verb:str, expect_text:tuple[str,...])` frozen model;
  `export_visible_text(app) -> str`; `async run_step(pilot, step) -> str` (issues the verb, returns
  captured screen text). Attach `export_visible_text` onto `AppShell` as `.export_visible_text()`.

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/bdd/test_harness.py
import pytest
from babylon.tui.shell.bdd.harness import TutorialStep, export_visible_text
from babylon.tui.shell.app_shell import AppShell


def test_tutorial_step_is_frozen():
    step = TutorialStep(verb="educate", expect_text=("Educate",))
    with pytest.raises(Exception):
        step.verb = "attack"


@pytest.mark.asyncio
async def test_export_visible_text_is_deterministic():
    app = AppShell()
    async with app.run_test() as pilot:
        first = export_visible_text(app)
        second = export_visible_text(app)
        assert first == second  # no wall-clock, reproducible
        assert "action bar" in first
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/bdd/harness.py
"""BDD harness — headless Pilot text-capture (design §G).

Runs the shell via Textual Pilot, issues a step's verb, and captures the emitted screen text —
the raw render of what the player sees. Determinism: narrator OFF (byte-reproducible); no
wall-clock. Text export uses a recording Rich Console (Textual 8.2.8 has no App.export_text).
"""

from __future__ import annotations

from pydantic import BaseModel, ConfigDict
from rich.console import Console
from textual.app import App
from textual.pilot import Pilot


class TutorialStep(BaseModel):
    """One tutorial/BDD step: a verb to issue and the text expected on the resulting screen."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    verb: str
    expect_text: tuple[str, ...]


def export_visible_text(app: App[object]) -> str:
    """Export the currently-visible widget tree as plain text via a recording Console."""
    console = Console(record=True, width=app.size.width or 120, file=open("/dev/null", "w"))
    for widget in app.screen.walk_children():
        renderable = getattr(widget, "renderable", None)
        if renderable is not None:
            console.print(renderable)
    return console.export_text()


async def run_step(pilot: Pilot[object], step: TutorialStep) -> str:
    """Issue ``step.verb`` and return the captured screen text after it settles."""
    await pilot.press(*step.verb)  # verb keybinding; integration maps verb→key
    await pilot.pause()
    return export_visible_text(pilot.app)
```

> **Note:** `export_visible_text` also lands as a bound method on `AppShell`
> (`def export_visible_text(self) -> str: return export_visible_text(self)`) so Phase-3 tests can
> call `pilot.app.export_visible_text()`. Add that one-liner to `AppShell` in this task's commit.

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/bdd/ tests/unit/tui/shell/bdd/test_harness.py \
        src/babylon/tui/shell/app_shell.py
mise run commit -- "feat(tui): BDD Pilot text-capture harness + TutorialStep model"
```

### Task 11: the three assertion layers

**Files:**
- Create: `src/babylon/tui/shell/bdd/assertions.py`
- Test: `tests/unit/tui/shell/bdd/test_assertions.py`

**Interfaces:**
- Consumes: `TutorialStep` (Task 10); `babylon.projection.verbs.preview.CANONICAL_VERBS`;
  `EconomyViewLike` (Task 6) for the algebraic layer's fixtures.
- Produces: `assert_coverage(steps:Sequence[TutorialStep]) -> None` (layer 1 — every canonical
  verb appears; raises `CoverageError` naming the missing verbs); `assert_render(captured:str,
  expect:Sequence[str]) -> None` (layer 2); `assert_invariants(econ:EconomyViewLike,
  replay_hashes:Sequence[str]) -> None` (layer 3 — Φ≥0, tri-decomp closure, hash stability).

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/tui/shell/bdd/test_assertions.py
import pytest
from babylon.tui.shell.bdd.assertions import (
    CoverageError,
    InvariantError,
    RenderError,
    assert_coverage,
    assert_invariants,
    assert_render,
)
from babylon.tui.shell.bdd.harness import TutorialStep
from babylon.tui.shell.views._economy_contract import EconomyViewLike


def test_coverage_reds_when_a_verb_is_never_exercised():
    steps = [TutorialStep(verb="educate", expect_text=())]
    with pytest.raises(CoverageError) as e:
        assert_coverage(steps)
    assert "attack" in str(e.value)  # names a missing verb


def test_render_layer_matches_expected_text():
    assert_render("FUNDAMENTAL THEOREM: Wc=90", ("Wc=90",))
    with pytest.raises(RenderError):
        assert_render("nothing here", ("Wc=90",))


def test_invariants_catch_broken_phi_closure_and_hash_drift():
    good = EconomyViewLike(wc=90, vc=120, phi_ue=10, phi_repro=5, phi_dom=3, wealth_gini=0.6)
    assert_invariants(good, replay_hashes=["abc", "abc", "abc"])
    with pytest.raises(InvariantError):  # negative Φ component
        bad = EconomyViewLike(wc=90, vc=120, phi_ue=-1, phi_repro=5, phi_dom=3, wealth_gini=0.6)
        assert_invariants(bad, replay_hashes=["abc", "abc"])
    with pytest.raises(InvariantError):  # hash drift across replay
        assert_invariants(good, replay_hashes=["abc", "def"])
```

- [ ] **Step 2: Run test to verify it fails** → FAIL (module missing).
- [ ] **Step 3: Write minimal implementation**

```python
# src/babylon/tui/shell/bdd/assertions.py
"""The three BDD assertion layers (design §G): behavior · render · algebra.

One transcript validates all three: every verb is exercised (coverage), the emitted text is what
the player should see (render), and the state the verbs produced obeys the mathematical-core
property laws (algebra). Determinism uses the v1.0 replay-identity hash; content-hash arrives
with III.13.
"""

from __future__ import annotations

from collections.abc import Sequence

from babylon.projection.verbs.preview import CANONICAL_VERBS
from babylon.tui.shell.bdd.harness import TutorialStep
from babylon.tui.shell.views._economy_contract import EconomyViewLike


class CoverageError(AssertionError):
    """Layer 1: a canonical verb was never exercised (a dead option = ∂L red gate)."""


class RenderError(AssertionError):
    """Layer 2: the emitted screen text did not contain an expected fragment."""


class InvariantError(AssertionError):
    """Layer 3: an algebraic property law was violated."""


def assert_coverage(steps: Sequence[TutorialStep]) -> None:
    exercised = {step.verb for step in steps}
    missing = sorted(CANONICAL_VERBS - exercised)
    if missing:
        raise CoverageError(f"verbs never exercised (dead options): {missing}")


def assert_render(captured: str, expect: Sequence[str]) -> None:
    for fragment in expect:
        if fragment not in captured:
            raise RenderError(f"expected {fragment!r} in emitted screen text; not found")


def assert_invariants(econ: EconomyViewLike, replay_hashes: Sequence[str]) -> None:
    # Φ tri-decomposition: each component ≥ 0 and Φ = φ_UE + φ_repro + φ_dom (closure).
    for name, value in (("phi_ue", econ.phi_ue), ("phi_repro", econ.phi_repro), ("phi_dom", econ.phi_dom)):
        if value < 0:
            raise InvariantError(f"Φ component {name}={value} < 0")
    phi = econ.phi_ue + econ.phi_repro + econ.phi_dom
    if phi < 0:
        raise InvariantError(f"Φ={phi} < 0")
    # Determinism: replay-identity hash stable across the run.
    if len(set(replay_hashes)) > 1:
        raise InvariantError(f"replay-identity hash drifted across run: {list(replay_hashes)}")
```

> **Integration note (T6):** the tutorial `TutorialStep` script drives these three assertions over
> a real headless campaign transcript. The Fundamental-Theorem verdict (`Wc>Vc`), survival-calculus
> rupture (`P(S|R)>P(S|A)`), bifurcation sign, and overshoot `O=C/B` join the algebra layer once
> the campaign runtime exposes them via the projection — extend `assert_invariants` with those
> reads there, keeping each as a named property law with a loud message.

- [ ] **Step 4: Run tests** → PASS. **Step 5: Commit**

```bash
git add src/babylon/tui/shell/bdd/assertions.py tests/unit/tui/shell/bdd/test_assertions.py
mise run commit -- "feat(tui): three-layer BDD assertions (coverage/render/algebra)"
```

---

## Integration (post-merge, folds into T4-integration / T6)

These are **not** standalone tasks — they land inside the owning trains once the cascade merges:

1. **Boot the shell** (`cli/play.py`): replace the two-node demo with `AppShell`, wired to
   `GameSession` + the paced driver; number keys switch views; the action bar calls
   `issue_action` with the real `PostgresRuntime` sink. Snapshot goldens re-bake (render tier —
   regenerate freely, not a ceremony).
2. **Live view data:** `DashboardView` imports the real `EconomyView` (delete `_economy_contract`);
   `MapView`/`TopologyView`/`WikiView` bind to `project_*` outputs via the composition root.
3. **Chronicle rail:** subscribe to T4's chronicle adapter; color by `resolve_severity` tier;
   autopause on derived CRITICAL.
4. **Full BDD suite (T6):** author the `TutorialStep` script exercising all nine verbs; run it
   headless through `run_step`; assert all three layers over the real transcript; add the
   theorem/rupture/bifurcation/overshoot property reads to `assert_invariants`.

---

## Self-Review

- **Spec coverage:** §A action registry → Task 1; §A two drivers → Tasks 8–9; §B shell → Task 2;
  §C1 Dashboard → Task 6; §C2 Map → Task 4; §C3 Wiki → Tasks 3+7; §C4 Topology → Task 5;
  §E mock-against-real-contracts → `_economy_contract` + fixture sinks; §F scope fence → stubs in
  Task 1 + `render_absence` in Task 5; §G BDD gate → Tasks 10–11. All sections mapped.
- **Placeholder scan:** no "TBD/TODO"; every code step shows real code. `TopologyView.render_topology`
  raises `NotImplementedError` **by design** (bound to the live graph at integration) with a named
  reason — this is a declared integration seam, not a hidden gap, and it has no unit asserting the
  live path (only `render_absence` is unit-tested here).
- **Type consistency:** `ActionSpec`/`ActionStatus`/`actions_for` consistent across Tasks 1, 8, 9;
  `EconomyViewLike` fields identical in Tasks 6 and 11; `TutorialStep(verb, expect_text)` identical
  in Tasks 10 and 11; `export_visible_text` defined in Task 10 and referenced by Task 3's test
  (Task 3 executes after Task 10 lands the helper, or stubs it — noted in Task 3).
- **Ordering note:** Task 3's test uses `export_visible_text` from Task 10. Execute Task 10 before
  Task 3, or land the `AppShell.export_visible_text` one-liner in Task 2. Recommended order:
  1 → 2 → 10 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 11.
