# Null-Play Political Coupling — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement
> this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the five endgame recognizer axes drift over hundreds of ticks under null play, by
(B) coupling consciousness to the *sustained* level of exploitation rather than its first difference,
and (C) animating the political-topology layer that is currently seeded-but-static.

**Architecture:** The causal spine this plan builds:

```
material base (already moves)
  -> sustained wage-value defect  [B: new level term]
  -> agitation -> class_consciousness / national_identity
  -> solidarity (unfreezes: activation_threshold finally crossed)
  -> faction influence_level        [C4]
  -> winning faction -> ruling_faction_id   [C5]
  -> colonial_stance                [C6]  -> ABOLISH/IGNORE/UPHOLD stance gates
  -> state_violence_index           [C7]  -> fascist political-violence route
  -> legitimacy erosion             [C8]  -> collapse path un-dormants
  -> crisis sovereignty_type        [C9]  -> fragmented_collapse crisis gate
```

**Tech Stack:** Python 3.11, Pydantic (frozen models), rustworkx (`BabylonGraph`), pytest.

**Owner rulings governing this plan (2026-07-18):** scope = **B + C together**; target arc =
**instrument-decides** (tune via `mise run sim:pacing`, then report curves; treat as gameplay-tunable).

## Global Constraints

Every task's requirements implicitly include this section.

- **Determinism is a hard gate.** Iterate nodes/edges in `sorted()` ID order — never dict/set order.
  Accumulate floats in sorted order. Any randomness comes from `resolve_rng(services, tick)`
  (`src/babylon/kernel/system_base.py`) and **never** bare `random`. **Inserting a new RNG draw before an
  existing one shifts the whole downstream RNG sequence and breaks `qa:regression` byte-identity even
  with no intended behavior change** — prefer designs that draw no randomness at all.
- **No hardcoded coefficients.** Every new number is a field on a `GameDefines` sub-model in
  `src/babylon/config/defines/`, then `poetry run python tools/generate_defines_config.py` to
  regenerate `src/babylon/data/defines.yaml`. Guarded by `tests/unit/config/test_constants_sync.py`.
- **Mutate through the protocol.** `graph.update_node(id, **attrs)` / `graph.update_edge(src, dst,
  edge_type, **attrs)` / `graph.set_graph_attr(key, value)`. Never mutate payload dicts directly.
  Frozen Pydantic models are fine — `from_graph()` re-instantiates from the graph dict each time, so an
  invalid value surfaces as a loud `ValidationError` (Constitution III.11), which is the desired behavior.
- **TDD, red first.** Every task writes a failing test, watches it fail for the right reason, then
  implements. `@pytest.mark.red_phase` for intentionally-failing tests.
- **RST docstrings** on all new public functions/classes (Sphinx `-W` blocks CI).
- **`qa:regression` must stay byte-identical on every task EXCEPT Task 10 (the ceremony).** If a task
  moves a baseline unexpectedly, STOP and report — do not regenerate outside Task 10.
- **Volume III collision boundary.** The owner is concurrently implementing
  `specs/024-capital-volume-iii/` (scissors price-value engine, dialectic engine, money system) in
  another worktree. Task 2's read of `opposition_states['wage']['balance']` touches that surface. It
  MUST go through one narrow named helper plus a behavioral-contract test that fails loudly if the
  field's scale or semantics move. Do not spread that read across files.
- **Ground rent is OUT of scope** (it belongs to spec-024 User Story 4).

---

### Task 1: Fix the faction node-type mismatch (real pre-existing bug)

**Files:**
- Modify: `src/babylon/engine/systems/faction_influence.py:165,201`
- Modify: `src/babylon/engine/systems/reactionary.py:218`
- Test: `tests/unit/balkanization/test_faction_node_type_query.py` (create)

**Why:** Faction nodes are stamped `_node_type="faction"` (`src/babylon/models/world_state.py:742`;
corroborated by `web/game/engine_bridge.py:744`), but these three call sites query
`node_type="balkanization_faction"`. `query_nodes` filters on `GraphNode.node_type` popped from
`_node_type` (`src/babylon/topology/adapters/query_mixin.py:50-56`) with no aliasing, so **all three
loops match zero nodes, always.** This silently disables `RED_SETTLER_TRAP_DETECTED`, secession-
eligibility faction enumeration, and `_find_fascist_faction` → `FASCIST_RECRUITMENT`. Later tasks
depend on faction enumeration working.

**Interfaces:**
- Consumes: `GraphProtocol.query_nodes(node_type=...)`.
- Produces: working faction enumeration for Tasks 4-9.

- [ ] **Step 1: Write the failing test**

Build a `WorldState` with at least one `BalkanizationFaction`, call `to_graph()`, and assert that
`query_nodes(node_type="faction")` returns it while `query_nodes(node_type="balkanization_faction")`
returns nothing — pinning the canonical string. Then assert the *system* behavior: with a faction whose
`class_reduction` exceeds the trap threshold and stance `UPHOLD`, `FactionInfluenceSystem.step()` emits
`RED_SETTLER_TRAP_DETECTED`.

- [ ] **Step 2: Run it and confirm it fails for the right reason**

Run: `mise run test:q -- tests/unit/balkanization/test_faction_node_type_query.py -v`
Expected: the emission assertion FAILS (no event), because the query matches nothing.

- [ ] **Step 3: Fix the three literals**

Replace `node_type="balkanization_faction"` with `node_type="faction"` at the three sites. Change
nothing else.

- [ ] **Step 4: Verify green + no regression**

Run: `mise run test:q -- tests/unit/balkanization/ -v` then `mise run check` then
`mise run qa:regression`.
Expected: new tests pass; `qa:regression` **5/5 byte-identical** (the goldens contain no faction nodes,
so this fix is inert there — confirm this rather than assume it, and record the confirmation).

- [ ] **Step 5: Commit**

```bash
mise run commit -- "fix(balkanization): faction queries used a node_type that never matches"
```

---

### Task 1b: Close the node-type vocabulary so this bug class cannot recur

**Owner directive (2026-07-18):** *"ensure you fix it in a way that prevents that error class from ever
happening again in the future, knowing im a solo dev using coding agents to do the coding so the
solution should be legible to agents as well as humans."*

**The error class, stated precisely:** a test fixture hand-stamps a `_node_type` string that production
never emits, then queries with that same string, matches its own fixture, and passes. The test validates
a convention that does not exist in production. Green tests, dead feature. It is a closed loop with no
external referent.

**Root enabling condition:** there is an `EdgeType` StrEnum (`src/babylon/models/enums/topology.py:12`)
but **no `NodeType` enum** — node types are raw magic strings, so nothing can catch an invented one.

**Evidence the guard is needed beyond this one bug** — stamped vocabulary (8, via literals in
`src/`+`web/`): territory, social_class, key_figure, institution, sovereign, organization, industry,
faction. Queried vocabulary (10): those 8 **plus `hex` and `community`, which no literal ever stamps**.
Query sites: `substrate.py:75`, `vol2_circulation.py:163`, `territory_diagnostics.py:70` (hex),
`domain/institution/queries.py:44` (community). If those are genuinely unstamped, `SubstrateSystem`
(MATERIAL_BASE @2.5) iterates an empty set every tick. **Task 1b must resolve this question, not assume it.**

**Files:**
- Modify: `src/babylon/models/enums/topology.py` (add `NodeType` beside `EdgeType`)
- Create: `src/babylon/sentinels/vocabulary/` (new sentinel, reusing `sentinels/_ast.py`)
- Create: `tests/unit/sentinels/test_node_type_vocabulary.py`
- Create/extend: a round-trip contract test under the existing `sentinels/roundtrip` idiom
- Modify: `CLAUDE.md` (gotchas section)

- [ ] **Step 1: Resolve the hex/community question FIRST.** Determine whether `hex` and `community`
      nodes are ever added to the graph by any path. If they are not, report it loudly — that is a
      separate real bug (dead systems), to be recorded and scoped, NOT silently fixed here.

- [ ] **Step 2: Add the `NodeType` StrEnum** — single source of truth, members = the canonical stamped
      vocabulary. Place beside `EdgeType` so the two are discovered together.

- [ ] **Step 3: Replace magic strings** with `NodeType.*` at every stamp and query site in `src/` and
      `web/`. Mechanical; MyPy strict then catches misuse at typecheck time, before tests run.

- [ ] **Step 4: Write the vocabulary sentinel.** AST-scan `src/`, `web/`, **and `tests/`** and assert:
      (a) every node-type literal is a `NodeType` member — kills invented strings *including in
      fixtures*, which is the actual bug; and (b) **every queried type is stamped somewhere** — the
      stamped/queried asymmetry is precisely the signal that made this bug invisible. A query for a type
      nothing produces is dead by construction.
      **Failure message must name the offending file:line, the offending string, and the allowed set** —
      that is the agent-legibility requirement.

- [ ] **Step 5: Write the round-trip contract test** — the one that would have caught this directly.
      For every entity family in `WorldState`: after `to_graph()`, `query_nodes(node_type=NodeType.X)`
      MUST find the entity. This uses the canonical construction path, so it is fixture-independent and
      pins production's own convention rather than a fixture's.

- [ ] **Step 6: Record the rule in `CLAUDE.md`** (standing permission to edit). Short and imperative,
      because agents read it every session:
      "**Never hand-stamp `_node_type` or query with a raw string — use `NodeType.*`.** A fixture that
      stamps a type production never emits yields a green test over a dead feature. This happened:
      `balkanization_faction` vs `faction` silently disabled RED_SETTLER_TRAP, secession enumeration,
      and FASCIST_RECRUITMENT. The vocabulary sentinel now enforces this."

- [ ] **Step 7: Verify** — `mise run check` green; `mise run qa:regression` byte-identical (pure
      rename + new tests; if a baseline moves, STOP).

- [ ] **Step 8: Commit** — `refactor(topology): close the node-type vocabulary with NodeType + sentinel`

---

### Task 2: B — couple consciousness to the sustained wage-value defect

**Files:**
- Create: `src/babylon/formulas/sustained_exploitation.py`
- Modify: `src/babylon/engine/systems/ideology.py` (~line 120-124 read, ~line 212-217 term)
- Modify: `src/babylon/config/defines/consciousness.py`
- Modify: `src/babylon/data/defines.yaml` (regenerated, not hand-edited)
- Test: `tests/unit/formulas/test_sustained_exploitation.py`, `tests/unit/engine/systems/test_ideology_sustained_term.py`

**Why:** `agitation` — the sole driver of `class_consciousness`/`national_identity` — is a pure first
difference (`consciousness_routing.py:45-88`), so it decays to zero once the Imperial Circuit reaches
steady state. The sustained defect `opposition_states['wage']['balance']` is computed every tick
(`opposition.py:503-510`) and is read at `ideology.py:123` **only as a boolean sign-gate**, never as a
magnitude.

**Interfaces:**
- Produces: `sustained_exploitation_agitation(balance: float, sensitivity: float) -> float` — the ONLY
  place in the codebase that interprets the scale/semantics of `opposition_states['wage']['balance']`.
- Consumes: `GameDefines.consciousness.sustained_exploitation_sensitivity` (new).

**Ordering note to document in the docstring and the ADR:** `ConsciousnessSystem` is @17.0 and
`ContradictionSystem` (which writes `opposition_states`) is @18.0 — so this term reads the **previous
tick's** opposition state. That one-tick lag is deterministic and acceptable; it must be stated
explicitly, not discovered later.

- [ ] **Step 1: Write the failing formula test + the Volume III contract test**

```python
def test_sustained_term_is_zero_when_labor_is_not_losing():
    # balance >= 0 means labor is not on the losing side -> no sustained agitation
    assert sustained_exploitation_agitation(0.0, 0.5) == 0.0
    assert sustained_exploitation_agitation(1.25, 0.5) == 0.0

def test_sustained_term_scales_with_defect_magnitude():
    assert sustained_exploitation_agitation(-1.0, 0.5) == pytest.approx(0.5)
    assert sustained_exploitation_agitation(-2.0, 0.5) == pytest.approx(1.0)

def test_sustained_term_is_monotonic_in_defect():
    a = sustained_exploitation_agitation(-1.0, 0.5)
    b = sustained_exploitation_agitation(-3.0, 0.5)
    assert b > a
```

Plus the **behavioral contract test** guarding the Volume III boundary — assert the shape and sign
convention this formula depends on (that `opposition_states["wage"]["balance"]` is negative when labor
loses, and is a magnitude in the same units the sensitivity is calibrated against). It must fail loudly
with a message naming spec-024 if that convention changes.

- [ ] **Step 2: Run to verify failure**

Run: `mise run test:q -- tests/unit/formulas/test_sustained_exploitation.py -v`
Expected: FAIL — module does not exist.

- [ ] **Step 3: Implement the formula**

```python
def sustained_exploitation_agitation(balance: float, sensitivity: float) -> float:
    """Agitation generated by the *sustained* wage-value defect.

    :param balance: ``opposition_states["wage"]["balance"]`` — negative when labor is
        on the losing side of the wage opposition (the W_c > V_c condition).
    :param sensitivity: ``GameDefines.consciousness.sustained_exploitation_sensitivity``.
    :returns: Non-negative agitation increment; ``0.0`` unless labor is losing.
    """
    if balance >= 0.0:
        return 0.0
    return -balance * sensitivity
```

- [ ] **Step 4: Add the define and regenerate the YAML**

Add `sustained_exploitation_sensitivity: float` to the consciousness defines sub-model with a
conservative provisional default (Task 9 calibrates it). Then:

```bash
poetry run python tools/generate_defines_config.py
mise run test:q -- tests/unit/config/test_constants_sync.py
```

- [ ] **Step 5: Wire the term into ConsciousnessSystem**

At `ideology.py` where `new_agitation` is assembled, add the sustained term alongside the existing
delta terms. Keep the existing sign-gate behavior intact; this ADDS a magnitude term, it does not
replace the rate term.

- [ ] **Step 6: Write and run the system-level test**

Assert that with a steady state (all deltas zero) but a persistently negative `balance`, agitation is
**non-zero** and `class_consciousness` **moves** tick over tick — the exact property that is broken
today.

- [ ] **Step 7: Verify**

Run: `mise run check`. Expected green.
Run: `mise run qa:regression`. **Expected: baselines MOVE** (the goldens have social_class nodes). Do
NOT regenerate here — record the observed drift and carry it to Task 10.

- [ ] **Step 8: Commit**

```bash
mise run commit -- "feat(consciousness): sustained wage-value defect drives agitation"
```

---

### Task 3: Verify where the political layer actually exists (scoping gate)

**Files:**
- Test: `tests/unit/balkanization/test_political_layer_presence.py` (create)

**Why:** `_seed_balkanization_layer` is called only from `_build_initial_state_for_scenario`
(`web/game/engine_bridge.py:6133`), web-side, tick-0. **No core engine scenario builder populates
`WorldState.factions`.** So Tasks 4-9 are no-ops in the 5 `qa:regression` goldens and most engine
tests. This task pins that fact as an executable contract so Task 10's ceremony can state, with
evidence, which drift came from B and which from C.

- [ ] **Step 1: Write characterization tests**

Assert `WorldState.factions == {}` for the core scenarios (`us`, `wayne_county`, `two_node`,
`imperial_circuit`), and that the web bridge path DOES seed factions. Mark clearly as characterization
of current behavior, not an endorsement.

- [ ] **Step 2: Confirm the null-play campaign path has factions**

Run: `mise run sim:pacing -- --ticks 5` (or the probe's smallest useful invocation) and confirm from
its output that faction/sovereign nodes are present on the web scenario path the campaign uses.
Record the finding. If factions are ABSENT there, STOP — Tasks 5-9 cannot satisfy ruling #13 and the
plan needs an engine-side seeding task inserted here.

- [ ] **Step 3: Commit**

```bash
mise run commit -- "test(balkanization): pin where the political layer is actually seeded"
```

---

### Task 4: C4 — per-tick `influence_level` writer

**Files:**
- Modify: `src/babylon/engine/systems/faction_influence.py`
- Modify: `src/babylon/config/defines/balkanization.py`
- Test: `tests/unit/balkanization/test_influence_level_writer.py`

**Why:** `influence_level` (INFLUENCES edges, `BalkanizationFaction → Territory`,
`relationship.py:124-129`) is written only at bootstrap. `FactionInfluenceSystem` @14.5 reads it and
writes only the transient `persistent_data["balkanization.winning_faction_by_territory"]` — recomputed
every tick from static inputs, so it never changes.

**Design (owner-decided fork):** **self-contained**, driven by the consciousness/solidarity that Task 2
sets in motion — NOT bridged to `Organization`. The `BalkanizationFaction`↔`Organization` linkage is a
deliberate deferral (`balkanization_faction.py:1-24`) and `Organization` is absent from most paths.

**Host:** `FactionInfluenceSystem` @14.5 — runs after `OODASystem` @14.0, already owns the read path,
and can recompute `winning_faction_by_territory` from the freshly-written value in the same `step()`.
Copy the `update_edge` idiom from `collapse_transition.py:265-271`.

- [ ] **Step 1: Write the failing test** — a faction whose aligned classes gain consciousness gains
      `influence_level` on its territories tick over tick; a faction whose base is demobilized loses it.
      Assert values stay clamped to `[0.0, 1.0]`.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Add the drift-rate define**, regenerate `defines.yaml`.
- [ ] **Step 4: Implement `_update_influence_levels`** inserted before the existing
      `_resolve_winning_factions` call. Iterate `sorted()` faction IDs then `sorted()` territory IDs.
      Clamp with `min(1.0, max(0.0, ...))` — never rely on float equality. Draw NO randomness.
- [ ] **Step 5: Verify** — `mise run check` green; `mise run qa:regression` byte-identical (goldens have
      no factions — confirm, per Task 3).
- [ ] **Step 6: Commit** — `feat(balkanization): influence_level responds to consciousness per tick`

---

### Task 5: C5 — `ruling_faction_id` tracks the winning faction

**Files:**
- Modify: `src/babylon/engine/systems/faction_influence.py` (or `sovereignty.py` — implementer's
  call, justified in the report)
- Test: `tests/unit/balkanization/test_ruling_faction_tracks_influence.py`

**Why (owner-decided fork):** the recognizer reads `ruling_faction_id → colonial_stance`, and
`ruling_faction_id` currently changes only on rare collapse/secession. Making rulership track actual
influence is semantically correct ("who rules determines the stance"); rewiring `_has_stance_majority`
to read the transient winning-faction dict would couple an observer to per-tick scratch state.

- [ ] **Step 1: Failing test** — when a faction's influence over a sovereign's claimed territories
      crosses a majority, that sovereign's `ruling_faction_id` follows, with hysteresis so it does not
      oscillate tick to tick.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Add hysteresis threshold define**, regenerate YAML.
- [ ] **Step 4: Implement**, sorted iteration, incumbent-priority tie-break (reuse the existing
      `winning_faction_for_territory` convention; do NOT add a new RNG draw).
- [ ] **Step 5: Verify** (`check`, `qa:regression` byte-identical).
- [ ] **Step 6: Commit** — `feat(balkanization): rulership follows factional influence`

---

### Task 6: C6 — `colonial_stance` drift

**Files:**
- Modify: `src/babylon/engine/systems/sovereignty.py` (@17.5)
- Modify: `src/babylon/config/defines/balkanization.py`
- Test: `tests/unit/balkanization/test_colonial_stance_drift.py`

**Why:** `colonial_stance` is the **only live working input** to all three stance-majority gates
(`endgame_detector.py:428` ABOLISH, `:535` UPHOLD, `:569` IGNORE) and has no writer.

**Host:** `SovereigntySystem` @17.5 — after `Consciousness` @17.0 and `FascistFaction` @17.4 (both
inputs fresh), before `CollapseTransition` @20.5 (which derives successor extraction policy from stance).

**Hazard:** the loop `colonial_stance → derive_extraction_policy_from_stance → extraction_policy →
metabolic_impact → habitability`. Read only pre-@17.5 state; never this pass's own output.

- [ ] **Step 1: Failing test** — a faction whose base crosses a consciousness/solidarity threshold
      drifts `UPHOLD → IGNORE → ABOLISH` as a **discrete deterministic step function** against defines
      thresholds (never a rounded continuous score); and drifts back under reaction.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Add stance-threshold defines**, regenerate YAML.
- [ ] **Step 4: Implement** via `graph.update_node(faction_id, colonial_stance=new.value)`. Reuse
      `derive_extraction_policy_from_stance`; do not duplicate the mapping.
- [ ] **Step 5: Verify** (`check`, `qa:regression` byte-identical).
- [ ] **Step 6: Commit** — `feat(balkanization): colonial stance responds to mass consciousness`

---

### Task 7: C7 — `state_violence_index` writer (wire the orphaned machinery)

**Files:**
- Modify: `src/babylon/ooda/action_effects.py` (import + dispatch to `state_ai.*`)
- Modify: `src/babylon/engine/systems/sovereignty.py` (@17.5, `set_graph_attr`)
- Modify: `src/babylon/config/defines/endgame.py` (add the max as a define)
- Modify: `src/babylon/engine/observers/endgame_detector.py:542-544` (read max from defines)
- Test: `tests/unit/engine/systems/test_state_violence_index.py`

**Why:** `state_violence_index` has **no writer anywhere** — the comment at
`endgame_detector.py:539-541` describes an intent spec-039 never implemented. Its gate is permanently
`0.0`, so the FASCIST_CONSOLIDATION political-violence route is dead weight.

**Reuse, do not reinvent:** `ooda/state_ai/repress_effects.py` (RAID/INFILTRATE/PROSECUTE/LIQUIDATE with
legitimacy multipliers) and `administer_effects.py` (`FUND` → `violence_capacity`) are fully built and
tested with **zero live callers** — `action_effects.py:11-26` never imports `state_ai.*`, and
`_resolve_repressive` (`:265-299`) only raises the *acting* org's consciousness.

`state_violence_index_max` should become an `EndgameDefines` field read directly by the detector
(mirroring `fascist_majority_fraction`), NOT a graph attribute.

- [ ] **Step 1: Failing tests** — (a) `resolve_action` on REPRESS/RAID raises the *target's*
      `repression_faced`, not just the actor's consciousness; (b) sustained repression raises
      `state_violence_index`; (c) the detector reads its max from defines.
- [ ] **Step 2: Run to verify failure.**
- [ ] **Step 3: Add `state_violence_index_max` define**, regenerate YAML.
- [ ] **Step 4: Wire the dispatcher** to `state_ai` effects; **Step 5: implement the index writer**
      (`set_graph_attr`, sorted aggregation, `min(1.0, ...)` clamp).
- [ ] **Step 6: Verify** — `check` green. `qa:regression`: the goldens have no state-apparatus orgs, so
      expect byte-identical; **if it moves, STOP and report** (that means the dispatcher change touched
      a live path).
- [ ] **Step 7: Commit** — `feat(state): wire repression effects and state violence index`

---

### Task 8: C8 — legitimacy erosion (un-dormants the collapse path)

**Files:**
- Modify: `src/babylon/engine/systems/sovereignty.py` (@17.5)
- Modify: `src/babylon/config/defines/balkanization.py`
- Test: `tests/unit/balkanization/test_legitimacy_erosion.py`

**Why:** nothing writes sovereign `legitimacy` or `balkanization.collapse_triggers`, so
`CollapseTransitionSystem`'s entire collapse-driven path (`collapse_transition.py:84-87`) is dormant —
it can only fire if a seed sovereign starts at `legitimacy <= 0.0`, and none do. Task 9's classifier
needs collapse to be reachable.

- [ ] **Step 1: Failing test** — sustained repression + falling consciousness alignment + extraction
      intensity erode `legitimacy` monotonically toward 0; a legitimate, low-repression sovereign holds
      steady. Assert clamped `[0.0, 1.0]`.
- [ ] **Step 2: Run to verify failure.** **Step 3: Add erosion-rate defines**, regenerate YAML.
- [ ] **Step 4: Implement** (sorted iteration, no RNG).
- [ ] **Step 5: Verify** (`check`, `qa:regression` byte-identical — no sovereigns in goldens).
- [ ] **Step 6: Commit** — `feat(balkanization): sovereign legitimacy erodes under repression`

---

### Task 9: C9 — crisis-sovereignty classifier

**Files:**
- Modify: `src/babylon/engine/systems/collapse_transition.py` (@20.5)
- Modify: `src/babylon/config/defines/balkanization.py`
- Modify: `src/babylon/engine/observers/endgame_detector.py` (docstring correction only)
- Test: `tests/unit/balkanization/test_crisis_sovereignty_classifier.py`

**Why:** `insurgent` / `occupation` / `emergency` are **never emitted anywhere in `src/`**, so
`_axis_fragmented_collapse`'s `crisis_gate` (`endgame_detector.py:612-613`) is a binary gate stuck at
`0.0`; progress caps at 0.75 and `matched` is provably always `False`.

**De-risks confirmed:** PG migration `0025_balkanization.sql` already permits all six values;
`sovereignty_type` round-trips losslessly; the fragmented_collapse axis is the ONLY gating consumer, so
new emissions cannot break the other four axes.

**Classification inputs (all computed before @20.5):** dual power (`SovereigntySystem` @17.5),
control-ratio crisis (`ControlRatioSystem` @12.0), UPRISING/EXCESSIVE_FORCE (`StruggleSystem` @16.0),
`dialectical_regime` (`ContradictionSystem` @18.0), percolation (`topology_monitor`), plus Task 7's
`state_violence_index` and Task 8's `legitimacy`.

- [ ] **Step 1: Failing test** — armed organized contestation ⇒ `insurgent`; external/dual-power
      administration ⇒ `occupation`; martial-law/control-ratio crisis ⇒ `emergency`; and none of these
      fire under quiescent conditions.
- [ ] **Step 2: Run to verify failure.** **Step 3: Add classifier threshold defines**, regenerate YAML.
- [ ] **Step 4: Implement** as a deterministic step function; write via `update_node`. Sorted iteration.
- [ ] **Step 5: Fix the docstring mismatch** — `_axis_fragmented_collapse`'s docstring says "no Faction
      holds the supermajority" but the code measures **ColonialStance** concentration. Correct the
      docstring to match the code (do NOT change behavior).
- [ ] **Step 6: Verify** (`check`, `qa:regression` byte-identical).
- [ ] **Step 7: Commit** — `feat(balkanization): classify crisis sovereignty types`

---

### Task 10: Calibrate the arc with the instrument

**Files:**
- Modify: `src/babylon/data/defines.yaml` (via regeneration) and the defines sub-models
- Create: `reports/null-play-arc-calibration.md`

**Owner ruling:** instrument decides; report the curves back. Treat every coefficient as
gameplay-tunable (same status as `endgame.fascist_majority_fraction`).

- [ ] **Step 1:** Run the 520-tick nationwide null-play probe: `mise run sim:pacing`.
- [ ] **Step 2:** Record, per tick, the five axis progress values plus `class_consciousness`,
      `agitation`, `solidarity_strength`, and tick-of-first-crossing per axis.
- [ ] **Step 3:** Tune `sustained_exploitation_sensitivity` (and the C drift rates) so the arc
      **drifts across hundreds of ticks and does NOT saturate to 1.0 early**. The known hazard: a
      currency-scale `balance` against normalized `[0,1]` ideology means a naive coefficient snaps the
      axis to ceiling in ~3 ticks.
- [ ] **Step 4:** Write `reports/null-play-arc-calibration.md` with the final coefficients, the curves,
      tick-of-first-crossing per axis, and an explicit statement that these are gameplay-tunable.
- [ ] **Step 5: Commit** — `chore(calibration): tune null-play arc coefficients`

---

### Task 11: THE CEREMONY — regenerate baselines with declared drift

**Files:**
- Modify: `tests/baselines/` (5 dense CSVs + 5 sampled JSONs, regenerated)
- Modify: `tests/baselines/detroit-tri-county-5t.json`, `tests/baselines/michigan-e2e.json`
- Create: `ai/decisions/ADR082_null_play_political_coupling.yaml` + `index.yaml` entry
- Modify: `ai/state.yaml`

**This is the first ceremony that actually regenerates baselines** — spine ceremony #1
(`fascist_majority_fraction` 0.75→0.9, observer-only) and the Task-23 whitelist widening both came back
byte-identical non-ceremonies.

- [ ] **Step 1:** `mise run qa:regression-generate-dense` (regenerates dense CSVs AND sampled JSONs).
- [ ] **Step 2:** `mise run qa:regression` — confirm the gate is green against the new baselines.
- [ ] **Step 3:** Regenerate the two e2e baselines.
- [ ] **Step 4:** **Declare per-scenario drift** — for EACH of the 5 scenarios, state the direction and
      magnitude of the change and attribute it to B or to C. Per Task 3, expect most/all C writers to be
      inert in the goldens (no factions/orgs); if C shows drift there, explain why before proceeding.
- [ ] **Step 5:** Write ADR082 recording: the derivative-vs-level root cause, the B+C owner ruling, both
      fork decisions with reasoning, the one-tick lag, the Volume III dependency, and the drift table.
- [ ] **Step 6:** Update `ai/state.yaml`.
- [ ] **Step 7: Commit** (ceremony commit body carries the full drift table).

---

### Task 12: Record the spine ratifications + revise #12

**Files:**
- Modify: `ai/decisions/ADR079_playability_spine.yaml`
- Modify: `ai/state.yaml`

- [ ] **Step 1:** Record all 13 owner ratifications from 2026-07-18 as an addendum.
- [ ] **Step 2:** **Revise ratification #12** — it accepted `tick_ground_rent` as ACCEPT-DARK on the
      premise that no county rental series exists. The trove audit disproved that for the urban leg
      (`fact_census_rent`, 44,997 rows). Record the correction AND that ground rent now belongs to
      spec-024 (Capital Volume III), not Track 2.
- [ ] **Step 3:** Note `fascist_majority_fraction = 0.90` as provisional/gameplay-tunable.
- [ ] **Step 4: Commit** — `docs(adr): record spine ratifications; revise ground-rent premise`

---

## Self-Review

**Spec coverage:** B (Task 2) and all four originally-named C couplings are covered — `state_violence_index`
(7), crisis sovereignty (9), `colonial_stance` (6), `influence_level` (4) — plus the two prerequisites
research surfaced: the node-type bug (1) and legitimacy erosion (8), and the rulership link (5) without
which the stance gates stay frozen regardless.

**Ordering:** Task 1 must precede 4-9 (faction enumeration). Task 3 gates 4-9 (scoping). Task 8 must
precede 9 (collapse must be reachable). Task 10 must follow all implementation. Task 11 must be last.

**Known risk:** Task 10's calibration is the one genuinely uncertain step — the scale mismatch could
require iterating. If the instrument shows saturation that no coefficient fixes, the relaxation term
sketched in the investigation (consciousness tracks rather than ratchets) is the fallback, and that is a
change to Task 2's formula requiring a re-run of Tasks 10-11.
