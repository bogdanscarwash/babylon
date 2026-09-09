# Track 1 — The Organizer's Map: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development.

**Goal:** Political state becomes visible only within organizing reach; everything
else renders as honest unknown. Solidarity edges draw as literal lines. Intel is a
session-scoped, aging ledger — never a dynamical system.

**Spec:** `docs/superpowers/specs/2026-07-17-viable-game-design.md` §5a.
**Grounding:** memory `track1-organizers-map-grounding` (all file:line verified
against `web/game/engine_bridge.py`, 9562 lines).

**Architecture:** Fog lives at the **serialization boundary**. The engine is
untouched, so determinism is preserved by construction. Visibility is a pure
function of `(graph, intel_ledger)`.

## Global Constraints

- **The engine is NOT modified.** All work is bridge/session layer. Any task that
  seems to need an engine change is wrong — re-read the grounding.
- **Honest unknown, never zero, never a stale color.** Loud Failure (III.11)
  extends to pixels. A masked field is `null` + an explicit `vision_masked` entry,
  never `0.0`.
- **Determinism:** the fog filter takes `(graph, intel_ledger)` EXPLICITLY. It must
  never read the four module-level process-global session dicts
  (`_session_action_history` :76, `_session_trap_state` :79,
  `_session_endgame_detectors` :89, `_session_causal_observers` :98). Core Track-1
  surfaces are verified clean of all four — keep them clean.
- **No hardcoded coefficients** — reach radius, aging rates, tier thresholds all go
  in `GameDefines`/`defines.yaml`.
- **Frozen e2e testids must survive** the takeover→route migration. Full list in the
  grounding memory; includes `region-map`, `framing-hex`, `inspection-stack`,
  `lens-mode-*`, `event-tray`, `action-composer`.
- **Reuse, do not reinvent:** `_apply_class_vision_gate` (:1791-1832) is the
  existing precedent and covers ~20% of the contract. `BabylonGraph.get_neighborhood`
  (topology/graph.py:702-735) is the reach primitive. Do not write a new BFS.
- Every task ends green on `mise run check`; `qa:regression` must stay
  **byte-identical** (bridge-layer work must not move engine baselines — if it
  does, STOP, that means something leaked into the engine).

---

## Task 1: Reconcile the two "player org" definitions

**Why first:** two conflicting definitions exist and nothing enforces they agree.
Fog keyed to the wrong one is a security-shaped bug — it would leak or over-mask.

**Files:**
- Modify: `web/game/engine_bridge.py:8868-8870` (`_serialize_organization`),
  `:4458-4460` (`get_inspector_org`)
- Test: `tests/unit/web/test_player_org_identity.py` (new)

Canonical definition is `WorldState.player_org_id` (world_state.py:461, graph meta
:827-828/:992-997) — already used by EpistemicHorizon and Doctrine. Retire the
structural heuristic (`class_character=="proletarian" and
org_type=="civil_society"`), which is self-documented as a stopgap at :8857-8862.

Test that the two definitions cannot silently diverge: a graph whose
`player_org_id` names an org NOT matching the heuristic must resolve to
`player_org_id`.

## Task 2: The reach primitive

**Files:**
- Create: `web/game/fog/reach.py`
- Test: `tests/unit/web/fog/test_reach.py`

```python
def organizing_reach(graph, player_org_id, radius) -> frozenset[str]:
    """Node ids within organizing reach — PRESENCE ∪ SOLIDARITY neighbourhood."""
```

Wraps `graph.get_neighborhood(player_org_id, radius=radius,
edge_types={EdgeType.PRESENCE, EdgeType.SOLIDARITY}, direction="both")`.
`Organization.territory_ids` is materialized as PRESENCE edges
(world_state.py:698-704). Radius is a new `GameDefines` coefficient.

Returns a `frozenset` — deterministic, order-independent, hashable for caching.

**Class↔territory is via TENANCY edges**, not `territory_ids` (which exists only on
`Organization`). Reuse `_tenancy_members_by_territory` (:1290-1322).

## Task 3: The intel ledger

**Files:**
- Create: `web/game/fog/ledger.py`
- Test: `tests/unit/web/fog/test_ledger.py`

Session-scoped, event-sourced from INVESTIGATE resolutions. Each entry:
`(node_id, field_group, tick_observed, value_snapshot)`. Snapshots **age visibly** —
an entry older than `intel_staleness_ticks` renders as approximate, then as unknown.

**This is NOT a dynamical system.** The ledger is append-only facts; visibility is a
pure function of it. No decay simulation, no feedback into the engine.

Distinct from the engine-side `investigation_intel` scalar (territory.py:221-233),
which is in the tick hash and stays untouched.

## Task 4: `apply_fog` and the three choke points

**Files:**
- Create: `web/game/fog/filter.py`
- Modify: `web/game/engine_bridge.py` — `_serialize_territory` (:8625-8819),
  `get_inspector_*` (:4398-4627)
- Test: `tests/unit/web/fog/test_filter.py`

```python
def apply_fog(payload, node_type, node_id, reach, ledger, tick) -> dict:
```

Mirrors `_apply_class_vision_gate`'s three tiers: exact / approximate
(`_mud_quantize` :1751) / masked. Masked fields become `null` with the field name
appended to `vision_masked`; approximate fields to `vision_approx`.

**The political field set** (from the grounding audit): `heat` (:8723),
`agitation`, `solidarity_index`, `dominant_class`, `consciousness` (:8735),
`solidarity` (:8736), `dominant_community` (:8738), faction stances.

**Trap:** `consciousness`/`solidarity`/`dominant_community` are hardcoded `None`
TODAY — honest by accident, not by fog. When later wired they become political and
must already route through this gate. Add them to the political set NOW.

## Task 5: Gate the hex rollup pair — both halves

**Files:**
- Modify: `web/game/engine_bridge.py` — `_hex_feature_properties` (:8010-8051),
  `_aggregate_hex_features` (:2212-2540), `get_inspector_hex` (:4620-4626)

`_aggregate_hex_features` **independently re-derives** the same fields
(`weighted_mean_metrics` :2289-2299). Gating only `_hex_feature_properties` misses
every aggregated zoom level. **Both must be gated, or refactored to one
filtered-then-aggregated pipeline** — prefer the refactor.

Note `weighted_mean_metrics` averages intensive quantities. Per the type theorem
(memory `intensive-aggregation-variance-error`), that is contravariantly correct but
correct *by accident*. Leave the math alone in this task; do not "fix" it here.

## Task 5b: Fog the state-apparatus dashboard (added mid-execution)

**Why:** Task 5 correctly gated `_serialize_organization`, but
`get_state_apparatus_dashboard` (:3465) discarded its graph (`state, _graph =`)
and never threaded `reach` — leaving the whole state-apparatus screen unfogged.

Audited all four unthreaded `_serialize_organization` call sites. Only this one
is a leak: `create_game` (:2023/:2099) and `_persist_snapshots_safe` (:8201) are
internal **persistence** paths that MUST write true state — fogging them would
corrupt the database. Fog is a serialization-boundary concern, not a storage one.

**The trap:** `_build_state_apparatus_dashboard` (:1195) does
`float(o.get("heat", 0.0))`. Fog sets masked fields to `None`, so `float(None)`
raises TypeError — the naive fix 500s the dashboard. Verified empirically before
committing to a design.

**Owner ruling (2026-07-18): partial aggregate + masked count.** `total_heat`
sums only orgs whose heat is visible; the payload carries `heat_orgs_visible` /
`heat_orgs_masked`. All-masked yields `total_heat = None`, never `0.0` — a zero
reads as "the police are under no pressure," which is a lie (III.11).

Consistent with the precedent Task 5 set for hex rollups, where `heat_pop`
excludes masked hexes from the mean's denominator rather than letting them
contribute a false zero. **Both aggregation paths now handle partial coverage
identically** — that symmetry is the point, and Task 10's sentinel should pin it.

## Task 6: Solidarity edges as literal lines

**Files:**
- Modify: cockpit map layer; `web/game/engine_bridge.py` edge serialization
- Test: e2e assertion on the frozen `region-map` testid

SOLIDARITY edges carry `solidarity_strength` — render weight from it. Edges outside
reach are not drawn at all (their existence is itself political information).

## Task 7: No fogged dead ends

**Files:** cockpit inspector; `web/game/engine_bridge.py`

Clicking an unknown yields a card naming **what** is unknown and **how to learn it**,
linking to the INVESTIGATE composer. Uses the existing `inspection-card` /
`action-composer` testids.

## Task 8: Fix EDUCATE (bridge-layer, one function)

**Files:**
- Modify: `web/game/engine_bridge.py::get_educate_targets` (:5250-5310)
- Test: `tests/unit/web/test_educate_targets.py`

`get_educate_targets` resolves classes via `_nodes_in_territory` (:714-723), which
**structurally never returns a social_class node** — `territory_ids` exists only on
`Organization`, so `data.get("territory_ids", [])` is always `[]`. Every territory
lands in `unavailable_communities`.

Swap to `_tenancy_members_by_territory` (:1290-1322), mirroring what
`_dominant_class_by_territory` / `_solidarity_index_by_territory` already do.

**This revises ratification #10**, which assumed an engine change. Adding
`territory_ids` to `SocialClass` would duplicate graph state and violate "don't
invent primitives". TENANCY is already the correct linkage.

## Task 9: De-mock `get_investigate_targets`

**Files:**
- Modify: `web/game/engine_bridge.py:5730-5817`
- Test: `tests/unit/web/test_investigate_targets.py`

Currently substantially mocked: hardcoded `observe_capability` (:5750), a literal
`"org-police-union"` (:5774-5793), hardcoded `active_moles_suspected=1` (:5796-5803).
Only `territory_scans` (:5753-5772) reads the real graph.

Per the no-compromise directive: read real graph state. Mock ONLY if the live DB
genuinely cannot provide it — and if so, say which field and why.

## Task 10: The fog-containment sentinel

**Files:**
- Modify: `src/babylon/sentinels/seam/registry.py`, `.../seam/types.py`
- Create: `src/babylon/sentinels/seam/checks_fog.py`
- Test: `tests/unit/sentinels/test_fog_containment.py`

Add a `FogClass` enum (`MATERIAL` / `POLITICAL` / `FOG_METADATA`) to `SeamEntry`
and mark every row. Then `check_fog_containment` as **both**:
1. a registry-driven parametrized test (every POLITICAL wire key is gated), and
2. a Hypothesis property over random graphs × out-of-reach (org, territory) pairs
   asserting no political field escapes.

**Mutation-validate it** (standing rule): remove the gate from one political field
and confirm the sentinel goes red. An unmutated gate is an unproven gate.

A new political field that forgets the gate must fail as loudly as an unregistered
wire key.
