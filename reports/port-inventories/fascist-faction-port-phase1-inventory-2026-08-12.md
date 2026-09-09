# FascistFactionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `FascistFactionSystem` (`src/babylon/engine/systems/reactionary.py`,
355 lines, tick position 17.4) is the fascism branch of the George Jackson bifurcation: it
computes a per-node `Fascist_Pull` from agitation/entitlement/solidarity, drifts
`fascist_alignment`, captures saturated nodes into a fascist `BalkanizationFaction`, shoves the
`capital_labor` opposition balance (ADR051 hook), and rolls a sigmoid-gated org-defection
mechanic (chauvinism accrual + `RED_BROWN_COUP`). Against the CURRENT BSL surface this system
is **more blocked than Territory was**: beyond the already-known query-lane gaps, it needs THREE
things nothing in this codebase names yet — an `EdgeRef`/edge-attribute read (Slice 2, already
anticipated), a graph-level composite-attribute store for the ADR051 opposition machinery (no
`deffield` shape exists for it at all, a NEW finding), and an event-history read-back inside a
tick (`emit` is write-only; nothing queries "did X fire this tick," a second NEW finding). It
also contains one libm sigmoid squarely inside the NO-IMPOSED-SIGMOIDS ruling, and one verified,
previously-unrecorded **defect**: the fascist-faction predicate's ideology substring match
mis-selects the anti-colonial `FAC_DECOLONIAL` over the actual fascist `FAC_RESTORATIONIST`
whenever both are seeded (the electoral goldens seed both).

**Verdict: BLOCKED (query-lane Slice 2 + a wholly new graph-attribute-storage lane + a wholly
new event-read-back lane), with two PORT-QUESTION rows (the imposed sigmoid; the ideology-token
defect) and one RESERVED-LINE surface (National Question fields on `BalkanizationFaction`).**
Two computations are honestly PORTABLE NOW (the entitled-role drift core sans solidarity, the
capture predicate's settler+uphold disjunct); everything else needs a named lane this codebase
does not have today.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/reactionary.py` | 355 | **The target.** `FascistFactionSystem`, both halves (drift/capture; chauvinism/defection). |
| `src/babylon/formulas/reactionary.py` | 152 | Pure formulas. Only `calculate_fascist_pull` and `calculate_defection_probability` are invoked by the system (reactionary.py:39-42). `calculate_spontaneous_riot_risk` is invoked by `StruggleSystem` (`struggle.py`), not this system. `calculate_entitlement_effective` is invoked by **nothing in production** — dead code, exercised only by `tests/unit/formulas/test_reactionary.py` (grep-confirmed, `rg -n 'calculate_entitlement_effective' src/ tests/`). |
| `src/babylon/domain/dialectics/core/coupling.py` | 225 | `StanceIntervention` (Pydantic record the system constructs, reactionary.py:184-188) and `apply_interventions` (the consumer — see §5). |
| `src/babylon/config/defines/reactionary.py` | 161 | `ReactionaryDefines` Pydantic model — all 19 fields; the system reads 10 of them (fascist pull/drift/capture, stance intervention, chauvinism/defection). The other 9 (spontaneous-riot, entitlement/volatility role defaults, `entitlement_threat_gain`, `fr_gate_epsilon`, the 4 OODA-verb coefficients) belong to `StruggleSystem`, the `SocialClass` model validator, or `src/babylon/ooda/action_effects.py` — same category, different consumers. |
| `src/babylon/data/defines.yaml` | `reactionary:` block, lines 927-949 | Player-editable values for the whole category. |
| `src/babylon/models/entities/social_class.py` | 450-465 (fields), 467-479 (validator) | `SocialClass.entitlement`/`.volatility`/`.fascist_alignment`/`.aligned_faction_id` — the node-level fields this system reads/writes, plus the role-default validator. |
| `src/babylon/models/entities/balkanization_faction.py` | 86 | `BalkanizationFaction` — the FACTION node model: `ideology` (free str), `colonial_stance` (enum), `is_settler_formation` (bool). RESERVED-LINE (§ Reserved-line surfaces below). |
| `src/babylon/models/enums/balkanization.py` | 33-47 | `ColonialStance` enum (UPHOLD/IGNORE/ABOLISH) — the National Question axis. RESERVED-LINE. |
| `src/babylon/models/enums/social.py` | 12-64 | `SocialRole` (8 members) — `_ENTITLED_ROLES` is 2 of them. |
| `src/babylon/models/entities/relationship.py` | 116-119 | `Relationship.solidarity_strength: Coefficient` — the ONE edge attribute this system reads that has a declared Pydantic field. `super_wage_bonus` and `chauvinism` have **no** declared `Relationship` field (grep-confirmed zero hits in this file) — both are graph-edge-state only. |
| `src/babylon/models/entities/organization.py` | 160 | `Organization.cadre_level: Probability` — read by `_org_discipline`; written by no System (see §5). |
| `src/babylon/kernel/system_base.py` | 35-55 (`resolve_rng`), 161-191 (`_write_clamped`, **not used** by this system — every clamp here is hand-rolled) | Shared scaffolding. |
| `src/babylon/models/enums/topology.py` | 62 (`SOCIAL_CLASS`), 67 (`FACTION`), 100 (`SOLIDARITY`), 104 (`WAGES`), 109 (`MEMBERSHIP`) | `NodeType`/`EdgeType` vocabulary the system queries. |
| `src/babylon/models/enums/events.py` | 69, 91, 96 (crisis triggers, read), 159-162 (this system's own 4 emissions) | `EventType` members. |
| `src/babylon/engine/systems/economic.py` | 458-476 | `ImperialRentSystem` (@9.0) — writes `WAGES.super_wage_bonus`, this system's cross-system read (§5). |
| `src/babylon/engine/systems/ideology.py` | 94-109 (class), 372-426 (agitation write) | `ConsciousnessSystem` (@17.0) — writes `ideology.agitation`, this system's cross-system read (§5). |
| `src/babylon/engine/actions/reproduce.py` | 91 | The `reproduce` OODA verb — the only writer of `cadre_level`, if it fires this tick (§5). |
| `src/babylon/engine/systems/contradiction.py` | 90, 99, 103, 121, 256-277 | `ContradictionSystem` (@18.0) — the sole writer of `opposition_states`/`dialectical_regime`, and the consume-once reader of `opposition_interventions` (§5). |
| `src/babylon/engine/systems/allegiance.py` | 1-22, 176-177, 377, 410-413 | `AllegianceSystem` (@17.42) — the sole downstream reader of `fascist_alignment` (§5). |
| `src/babylon/ooda/action_effects.py` | 341-366 | POGROM/VIGILANTISM/LOCKOUT OODA-verb effects. Consumes `ReactionaryDefines` but is **not called by `FascistFactionSystem.step()`** — a separate module in the OODA action-effect pipeline (@14.0). Out of scope for this port, flagged here only because it shares the defines category. |
| `src/babylon/engine/scenarios/balkanization_seed.py` | 1-33, 68-160 | `apply_balkanization_seed` — the sole production seeder of FACTION nodes (dormancy, §5). |
| `src/babylon/data/game/balkanization/seed_factions.json` | 40 | The 4 canonical `BalkanizationFaction` seed records — the data behind the verified defect (§6, §Reserved-line). |

**Not exercised by this system at all:** no `src/babylon/domain/*` module import beyond
`coupling.py`; no formula module beyond `formulas/reactionary.py`.

**Reference BSL packs / docs read for format** (fully read or targeted-read for the cited
sections): `docs/reference/bsl-language.rst` (6104 lines — §§2.3, 2.6-2.10, 2.9, 3.1, chapters
C5/C12); `rust/crates/babylon-bsl/src/structural_verbs.rs` (1-107, 369-693, 1286);
`rust/crates/babylon-bsl/src/evaluator.rs` (453-468, 1160-1219, 2148-2235);
`rust/crates/babylon-bsl/src/declarations.rs` (targeted grep for `DECLARABLE_INTRINSICS`/verb
list). The territory inventory's own reference reads (`metabolism.bsl`, `vitality.bsl`) are
adopted by citation, not re-read line-by-line here.

---

## 2. COMPUTATION CATALOG (execution order, `reactionary.py:84-103`)

### Step 0 — Setup (`step`, reactionary.py:84-103)
- **(a)** Wraps the graph, reads the tick, resolves `services.defines.reactionary`, reads last
  tick's `dialectical_regime`, finds the (at most one) fascist faction, and checks whether the
  `capital_labor` opposition is known yet — all before either of the two per-tick passes.
- **(b)** No arithmetic.
- **(c) Reads:** graph attr `dialectical_regime` (dict); graph attr `opposition_states` (dict, only
  its key-membership is checked); every `NodeType.FACTION` node's `is_settler_formation`,
  `colonial_stance`, `ideology`.
- **(d) Writes:** none.
- **(e) Defines:** none directly (passed down).
- **(f) Events:** none.

### Pass 1 — Drift + capture (`_process_drift`, reactionary.py:109-177; `_write_stance_intervention`, :179-191; `_incident_solidarity`, :193-201; `_find_fascist_faction`, :215-227 — called from `step()` at :95, its result threaded into `_process_drift` as a parameter, not called from within it)
- **(a)** For every active, entitled-role (`LABOR_ARISTOCRACY`/`COMPRADOR_BOURGEOISIE`)
  `SOCIAL_CLASS` node, sorted by id (III.7): compute `Fascist_Pull`; above threshold, bump
  `fascist_alignment` and — if the `capital_labor` opposition is known — push a
  `StanceIntervention`; at-or-above the recruitment threshold with a fascist faction present and
  no prior capture, capture the node.
- **(b)** Exact formulas:
  - `Fascist_Pull = agitation * (entitlement / (solidarity + epsilon))`
    (`formulas/reactionary.py:67`).
  - `alignment = min(1.0, alignment + defines.fascist_drift_step)` (reactionary.py:139), gated on
    `pull > defines.fascist_pull_threshold` (reactionary.py:138).
  - `magnitude = min(pull, defines.stance_intervention_cap) * defines.stance_intervention_gain`
    (reactionary.py:183), written as `StanceIntervention(target_key="capital_labor",
    delta_balance=magnitude, source=f"system:fascist_faction:{node_id}")` — always **positive**
    (toward the capital/reactionary pole).
  - Capture gate: `alignment >= defines.fascist_recruitment_threshold AND fascist_faction_id is
    not None AND aligned_faction_id is None` (reactionary.py:160-164) →
    `aligned_faction_id = fascist_faction_id` (idempotent — a captured node is never re-evented).
  - `_incident_solidarity` (reactionary.py:193-201): `best = max(best,
    edge.attributes.get("solidarity_strength", 0.0))` over EVERY `SOLIDARITY` edge in the graph
    whose source or target is this node — an O(E) scan per entitled node (no incidence index),
    defaulting to `0.0` when no incident edge exists.
  - `_find_fascist_faction` (reactionary.py:215-227): candidate FACTION node if
    `(is_settler_formation AND colonial_stance == "uphold")` OR `any(tok in ideology.lower() for
    tok in ("fascist","reaction","revanch","settler"))`; result = `min(candidates)` — **plain
    lexicographic string minimum on node id**, not a numeric selection.
- **(c) Reads:** `SOCIAL_CLASS.active` (default `True`), `.role`, `.ideology.agitation` (nested
  dict field, default `0.0`), `.entitlement` (default `0.0`), `.fascist_alignment` (default
  `0.0`), `.aligned_faction_id`; `SOLIDARITY` edges' `.solidarity_strength`; `FACTION` nodes'
  `.is_settler_formation`/`.colonial_stance`/`.ideology`; graph attr `opposition_states`
  (existence only).
- **(d) Writes:** `SOCIAL_CLASS.fascist_alignment`; `SOCIAL_CLASS.aligned_faction_id`; graph attr
  `opposition_interventions` (append).
- **(e) Defines:** `reactionary.solidarity_pull_epsilon` (0.1, `>0.0`),
  `reactionary.fascist_pull_threshold` (1.0, `>=0.0`), `reactionary.fascist_drift_step` (0.05,
  `[0,1]`), `reactionary.fascist_recruitment_threshold` (1.0, `[0,1]`),
  `reactionary.stance_intervention_cap` (1.0, `>=0.0`), `reactionary.stance_intervention_gain`
  (0.05, `[0,1]`) — all `defines.yaml:928-933`.
- **(f) Events:** `FASCIST_DRIFT` (payload: `node_id`, `fascist_pull`, `fascist_alignment`,
  `entitlement`, `solidarity`, `regime` — reactionary.py:141-155); `FASCIST_RECRUITMENT`
  (payload: `node_id`, `faction_id`, `fascist_alignment` — reactionary.py:166-177).

### Pass 2 — Chauvinism accrual + crisis defection (`_process_org_defections`, reactionary.py:233-291)
- **(a)** Every org→LABOR_ARISTOCRACY `MEMBERSHIP` edge accrues chauvinism every tick
  unconditionally; if an `ECONOMIC_CRISIS`/`SUPERWAGE_CRISIS`/`CRISIS_PHASE_TRANSITION` event
  fired THIS tick, each member additionally rolls a defection probability; a majority-defection
  org fires `RED_BROWN_COUP`.
- **(b)** Exact formulas:
  - `_accrue_chauvinism` (reactionary.py:293-312): `increment = chauvinism_base_rate +
    (chauvinism_superwage_bonus if superwaged else 0)`; `new = min(1.0, current + increment)`,
    written via `graph.update_edge(org_id, member_id, EdgeType.MEMBERSHIP, chauvinism=new)`.
  - `_is_superwaged` (reactionary.py:314-323): `any(edge.attributes.get("super_wage_bonus", 0.0) >
    0.0 for edge in WAGES edges targeting member_id)`.
  - `P_defection = calculate_defection_probability(chauvinism, discipline)` =
    `1.0 / (1.0 + exp(-(chauvinism - discipline)))`, exponent clamped to `[-500, 500]`
    (`formulas/reactionary.py:89-91`) — **a stipulated sigmoid** (see §6, PORT-QUESTION).
  - `_org_discipline` (reactionary.py:325-333): `org.attributes.get("cadre_level")` if numeric,
    else `defines.defection_default_discipline`.
  - Roll: `rng.random() < p_defect` using `resolve_rng(services, tick)` (seed
    `0xBA1AC1A + tick`, `system_base.py:35-55`), sorted org then sorted-by-target-id edge
    iteration (III.7 determinism).
  - Coup gate: `defections > defines.red_brown_coup_fraction * len(edges)` (reactionary.py:279).
  - `_crisis_this_tick` (reactionary.py:336-342): `any(e.tick == tick and e.type in
    {ECONOMIC_CRISIS, SUPERWAGE_CRISIS, CRISIS_PHASE_TRANSITION} for e in
    services.event_bus.get_history())` — reads the **event bus's accumulated history**, not the
    graph.
- **(c) Reads:** `MEMBERSHIP` edges (source=org, target=LA class) — filtered by target
  `SOCIAL_CLASS.role == LABOR_ARISTOCRACY`; `MEMBERSHIP.chauvinism` (edge attr, default `0.0`);
  `WAGES.super_wage_bonus` (edge attr, default `0.0`); `ORGANIZATION.cadre_level`; the full event
  bus history for this tick.
- **(d) Writes:** `MEMBERSHIP.chauvinism` (edge attr).
- **(e) Defines:** `reactionary.chauvinism_base_rate` (0.01, `[0,1]`),
  `reactionary.chauvinism_superwage_bonus` (0.02, `[0,1]`),
  `reactionary.defection_default_discipline` (0.3, `[0,1]`),
  `reactionary.red_brown_coup_fraction` (0.5, `[0,1]`) — `defines.yaml:934-937`.
- **(f) Events:** `ORGANIZATIONAL_FRACTURE` (payload: `org_id`, `member_id`, `chauvinism`,
  `defection_probability` — reactionary.py:266-278); `RED_BROWN_COUP` (payload: `org_id`,
  `defections`, `member_count` — reactionary.py:280-291).

**Events emitted by the whole system: 4 distinct `EventType`s** (`FASCIST_DRIFT`,
`FASCIST_RECRUITMENT`, `ORGANIZATIONAL_FRACTURE`, `RED_BROWN_COUP`), across up to 3+N emission
sites per tick. Per the WS1 (#502) ledger note in the CURRENT BSL surface, every one is an
unpinnable-today ledger row: `emit` exists in BSL (`structural_verbs.rs:369,665`,
`CollectingSink`) but `TickReport` carries no event log to compare against a golden.

---

## 3. TYPE INVENTORY

Runtime storage note (same load-bearing fact as the Territory inventory): `BabylonGraph`'s
`update_node`/`update_edge` are plain dict merges with no type coercion or quantization; all
in-tick arithmetic below is raw Python `float`/`bool`/`str`, not the Pydantic-validated type.

| Attribute | Node/Edge type | Python model type | Domain | Category |
|---|---|---|---|---|
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean gate (read-only here; written by `VitalitySystem`, `vitality.py:169`) |
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** (never written by any System — scenario-seed static) |
| `ideology.agitation` | SOCIAL_CLASS | nested `float` inside `IdeologicalProfile` (`social_class.py:101-107`) | `[0.0, ∞)` **unbounded** | unbounded real, nested-dict-shaped (not a flat field) |
| `entitlement` | SOCIAL_CLASS | `Intensity` | `[0.0, 1.0]` | unit-interval |
| `fascist_alignment` | SOCIAL_CLASS | `Intensity` | `[0.0, 1.0]` | unit-interval (this system's own write) |
| `aligned_faction_id` | SOCIAL_CLASS | `str \| None` | node-id pattern or `None` | **nullable reference**, write-once (idempotent) |
| `is_settler_formation` | FACTION | `bool` | `{T,F}` | boolean — **RESERVED-LINE** (National Question) |
| `colonial_stance` | FACTION | `ColonialStance` (StrEnum, 3 members) | `{UPHOLD,IGNORE,ABOLISH}` | **Enum discriminant, RESERVED-LINE** |
| `ideology` (FACTION) | FACTION | `str`, `min_length=1, max_length=64` | free text | **unbounded string, RESERVED-LINE** — no closed vocabulary |
| `solidarity_strength` | SOLIDARITY edge | `Coefficient` (declared, `relationship.py:116-119`) | `[0.0, 1.0]` | unit-interval, **edge-scoped** |
| `super_wage_bonus` | WAGES edge | **no declared Pydantic field** — raw graph-edge-state float | `[0.0, ∞)`, money-semantic, unbounded | unbounded real, **round-trip-lossy** (see finding below), edge-scoped |
| `chauvinism` | MEMBERSHIP edge | **no declared Pydantic field** — raw graph-edge-state float | `[0.0, 1.0]` in practice (accumulator, never explicitly lower-clamped but structurally ≥0) | unit-interval, **round-trip-lossy** (this system's own write; docstring-documented, reactionary.py:296-305), edge-scoped |
| `cadre_level` | ORGANIZATION | `Probability` | `[0.0, 1.0]` | unit-interval (read-only here; written only by the `reproduce` OODA verb, `actions/reproduce.py:91`) |
| `dialectical_regime` | graph attr | `dict[str, Any]` (`{"regime": str, "opposition": str, "rate": float, ...}`) | unconstrained | **composite/nested — no scalar field shape** |
| `opposition_states` | graph attr | `dict[str, dict[str, Any]]` | unconstrained | **composite/nested — no scalar field shape** |
| `opposition_interventions` | graph attr | `list[dict[str, Any]]` (append-only until consumed) | unconstrained, growing | **composite/list-of-records — no scalar field shape** |
| `fascist_pull_threshold`, `fascist_drift_step`, `fascist_recruitment_threshold`, `stance_intervention_gain`, `stance_intervention_cap`, `chauvinism_base_rate`, `chauvinism_superwage_bonus`, `defection_default_discipline`, `red_brown_coup_fraction` (defines) | — | `float` | `[0,1]` or `>=0` (see §2 per-field) | unit-interval / non-negative coefficients |
| `solidarity_pull_epsilon` (define) | — | `float` | `(0.0, 1.0]` (`gt=0.0`, no explicit upper bound in the Field, but `defines.yaml` comment shows no `<=` either — **effectively `>0`, unbounded above**) | **unbounded-above coefficient, div-by-zero guard** |

**Two round-trip-lossy edge attributes, not one.** The system's own docstring
(reactionary.py:296-305) documents this for `chauvinism` alone: it is graph-edge-state, not a
`Relationship` field, so `WorldState.from_graph` drops it, and the in-memory `Simulation` facade
(which rebuilds the graph from `WorldState` each tick) silently resets it to `0.0` every tick —
only the BRIDGED runner (persistent in-place graph) accrues it correctly. **Verified here:**
`super_wage_bonus` has the exact same shape (no `Relationship` field, grep-confirmed zero hits in
`relationship.py`) and therefore the exact same failure mode, but the code carries no comment
calling it out. Recorded verbatim, port-as-is: this is a second, previously-unremarked instance
of the same gotcha (CLAUDE.md "Graph round-trip loses data").

**RESERVED-LINE surfaces (describe only, never propose changes):** `FACTION.colonial_stance`,
`FACTION.is_settler_formation`, `FACTION.ideology`, `_ENTITLED_ROLES` (which `SocialRole`
members carry an imperial stake), `_FASCIST_IDEOLOGY_TOKENS`, and the entitlement role-default
map in `social_class.py:41-46` are all direct expressions of the Constitution I.1/I.4
settler-colonial and MLM-TW class-position framework. Any content-modeling workaround for the
enum/string gaps below (§6) touches this surface and needs Director sign-off, not an agent
judgment call.

---

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). **One libm transcendental exists** —
`math.exp` in `calculate_defection_probability` (`formulas/reactionary.py:91`) — grep-confirmed
the only `exp`/`log`/`pow` call anywhere in `reactionary.py` + `formulas/reactionary.py`'s
system-invoked functions. Shapes, in execution order:

1. **Division with an additive guard:** `entitlement / (solidarity + epsilon)`
   (`formulas/reactionary.py:67`) — one add, one divide. `epsilon` is `Field(gt=0.0)`-validated
   at defines-load time, so the denominator is structurally never exactly zero as long as defines
   validate; no runtime guard beyond that (unlike Territory's rent-spike path, there is no
   `max(epsilon, x)` clamp here — the whole safety property rests on the Pydantic `gt=0.0` on the
   coefficient).
2. **Multiply:** `agitation * (...)` (`formulas/reactionary.py:67`) — completes the pull formula.
3. **Threshold comparison:** `pull > defines.fascist_pull_threshold` (reactionary.py:138) — plain
   `>`.
4. **Additive accumulation, upper-only clamp:** `min(1.0, alignment + defines.fascist_drift_step)`
   (reactionary.py:139) — bare `1.0` literal (BSL "no bare non-integer literal" parser issue, same
   as Territory/Metabolism precedent). **No `_write_clamped` call** — this system never uses the
   shared clamp helper; both of its clamps (this one and item 8 below) are hand-rolled
   `min(1.0, ...)`, internally consistent with each other but inconsistent with the
   `_write_clamped` convention other Systems use.
5. **Cap-then-scale:** `min(pull, defines.stance_intervention_cap) * defines.stance_intervention_gain`
   (reactionary.py:183) — one upper-only cap (no lower bound needed; `pull >= 0` always), one
   multiply.
6. **Threshold comparison (≥):** `alignment >= defines.fascist_recruitment_threshold`
   (reactionary.py:161) — plain `>=`.
7. **Running max fold:** `best = max(best, edge.attributes.get("solidarity_strength", 0.0))`
   (reactionary.py:200), seeded `best = 0.0` — order-independent by construction (a true `max`,
   not a collect-then-apply pattern), a **favorable structural match** for BSL's `fold max` head
   IF the edge-attribute read it depends on existed (§6).
8. **Additive accumulation, upper-only clamp (again):** `min(1.0, current + increment)`
   (reactionary.py:310), where `increment = chauvinism_base_rate [+ chauvinism_superwage_bonus]`
   — a second bare `1.0` literal, same shape as item 4, same missing shared-helper use.
9. **The libm sigmoid, overflow-guarded:** `exponent = -(chauvinism - discipline)`; `exponent =
   max(-500.0, min(500.0, exponent))` (a genuine, correct **two-sided** clamp — the only
   two-sided clamp in this system, there specifically to bound `math.exp`'s argument against
   overflow); `1.0 / (1.0 + math.exp(exponent))` (`formulas/reactionary.py:89-91`). **Flagged
   twice over:** (i) `math.exp` is a libm transcendental — cross-platform/cross-language
   nondeterminism hazard per house rules; (ii) this is a **stipulated logistic form**, squarely
   inside the ADR172 ruling-5 / ADR173 "no imposed functional forms — sigmoids must EMERGE from
   P(revolution)/P(acquiescence)" prohibition — `calculate_defection_probability` imposes
   `sigmoid(chauvinism − discipline)` directly as the mechanic, not as an emergent measure over a
   population distribution. This is a **PORT-QUESTION row**, not an auto-portable formula, same
   class as the Survival-calculus P(S|A) re-derivation ADR173 already names.
10. **Real × Int-derived comparison:** `defections > defines.red_brown_coup_fraction * len(edges)`
    (reactionary.py:279) — one multiply (`Coefficient × count`), one comparison. `len(edges)` is a
    genuine per-org count, not a truncating cast.
11. **RNG seed construction:** `_SYSTEM_RNG_SEED_SALT + tick` (`system_base.py:55`) — integer
    add, deterministic, outside this system's own file but load-bearing for its determinism story.

**No Real→Int demotion anywhere in this system** — grep-confirmed zero `int(...)` truncating
casts in `reactionary.py` (unlike Territory's population/displacement casts). `defections` is a
plain incremented counter, not a cast.

**Bare non-integer literals found:** `1.0` (×2, items 4 and 8); `-500.0`/`500.0`/`1.0` (×3 more,
item 9, inside `formulas/reactionary.py`) — five bare literals total needing the `c`-suffix or
Real-zero-promotion idiom in any port.

**A latent-but-inert default-argument trap** (not exercised on any current call path, recorded
for completeness): `calculate_fascist_pull`'s `epsilon` parameter defaults to
`_REACT.solidarity_pull_epsilon`, a **module-import-time-frozen** `GameDefines()` singleton
(`formulas/reactionary.py:27-30,37`) — a different object than `services.defines` the system
actually threads through. The system always passes `epsilon=defines.solidarity_pull_epsilon`
explicitly (reactionary.py:134), so the trap never fires today; it would only matter for a
scenario that YAML-overrides `reactionary.solidarity_pull_epsilon` AND calls the formula without
the explicit kwarg — grep-confirmed no `SCENARIOS[...]["defines_overrides"]` entry touches any
`reactionary.*` key (`tools/regression_scenarios.py:37-133`).

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.4** (reactionary.py:78), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-363`): `...StruggleSystem(16.0) → ConsciousnessSystem(17.0) →
  FascistFactionSystem(17.4) → AllegianceSystem(17.42) → ElectoralSystem(17.45) →
  PolicySystem(17.47) → SovereigntySystem(17.5) → MarketScissorsSystem(17.8) →
  ContradictionSystem(18.0) → ...`.
- **Reads from same-tick prior systems:**
  - `ConsciousnessSystem` @17.0 writes `ideology.agitation` (`ideology.py:372-426`) — read same
    tick via `_agitation_of`.
  - `SOLIDARITY.solidarity_strength` is written earlier the same tick by `CommunitySystem`
    (@7.0, amplify, `community.py:534-574`), `SolidaritySystem` (@8.0, `solidarity.py:155,181`),
    `DoctrineSystem` (@14.7, decay, `doctrine.py:131-138`), and `StruggleSystem` (@16.0,
    `struggle.py:384-398`) — all four run before 17.4, so `_incident_solidarity` reads this
    tick's freshest value.
  - `ImperialRentSystem` @9.0 writes `WAGES.super_wage_bonus` (`economic.py:458-476`) — read same
    tick by `_is_superwaged`.
  - `ORGANIZATION.cadre_level` is written ONLY by the `reproduce` OODA verb
    (`actions/reproduce.py:91`), which runs inside `OODASystem` @14.0 **if and only if a
    `reproduce` action fires that tick** — grep-confirmed no `engine/systems/*.py` module writes
    `cadre_level` (only `community.py`/`doctrine.py` read it). When no `reproduce` verb fires,
    `cadre_level` is whatever the scenario seeded (or the `Probability` default `0.1`) — a
    conditional, verb-gated same-tick channel, not a guaranteed one.
  - `dialectical_regime` and `opposition_states` are BOTH written **only** by `ContradictionSystem`
    @18.0 (`contradiction.py:103,256-258`), which runs AFTER 17.4. The system's own docstring
    (reactionary.py:24-31) explicitly documents the one-tick lag for `dialectical_regime` (empty
    on tick 1, observability-only, never gates dynamics). **The `opposition_known` gate
    (reactionary.py:96-98) has the identical one-tick-stale shape — same sole writer, same
    position — but the code carries no comment naming it.** Recorded verbatim as an oddity: one
    of two structurally-identical staleness reads is documented, the other is not.
- **Writes consumed downstream:**
  - `fascist_alignment` — read same tick, immediately next system, by `AllegianceSystem` @17.42
    (its own docstring names the dependency explicitly: "a class's `fascist_alignment` (@17.4)
    pulls its allegiance toward fascist-ideology parties," `allegiance.py:19`; consumption sites
    `allegiance.py:176-177,377,410,413`, the "Obama→Trump pipeline"). Also referenced by
    `ElectoralSystem`'s docstring (`electoral.py:50`, "no bridges → `fascist_alignment`").
  - `aligned_faction_id` — grep-confirmed read by **no other System**
    (`rg -n 'aligned_faction_id' src/babylon/engine/systems/*.py` outside `reactionary.py`:
    zero hits). Also **not** what drives the `FASCIST_CONSOLIDATION` endgame pattern —
    `EndgameDetector` (`endgame_detector.py:13-18`) keys that off `national_identity >
    class_consciousness` fraction and the Sovereign-stance political-violence route, not off this
    system's writes at all. Terminal/observational output.
  - `MEMBERSHIP.chauvinism` — grep-confirmed read by no other System. Terminal/observational
    (and round-trip-lossy per §3).
  - `opposition_interventions` — consumed exactly once per tick by `ContradictionSystem` @18.0's
    `_apply_interventions` (`contradiction.py:266-277`): reads the list, applies via
    `apply_interventions` (`coupling.py:174-225`, clamps the resulting balance to `[-1,1]`), then
    **clears the attribute to `[]`** — a genuine consume-once channel, confirmed.
  - `FASCIST_DRIFT`/`FASCIST_RECRUITMENT`/`ORGANIZATIONAL_FRACTURE`/`RED_BROWN_COUP` events are
    read by narrative/projection layers only (`game/chronicle_adapter.py`,
    `projection/chronicle.py`, `models/event_severity.py`, `engine/event_builders.py`) — never by
    another engine `System`.
- **Context/service usage with no BSL equivalent:**
  - `_crisis_this_tick` reads `services.event_bus.get_history()` — the **event bus's accumulated
    history**, filtered by `tick` and `type` membership. This is not a graph read at all. Per the
    CURRENT BSL surface, `emit` (`structural_verbs.rs:369,665`) is write-only into an
    `EventSink`/`CollectingSink` (`structural_verbs.rs:68-82`) — grep-confirmed
    (`rg -n 'event_log|EventLog|emitted_events|event_history' rust/crates/babylon-bsl/src/*.rs
    rust/crates/babylon-tick/src/*.rs`: zero hits) there is **no expression-position accessor
    anywhere in the evaluator that can query "was event X emitted this tick."** This is a second,
    previously-unnamed gap, distinct from the WS1 golden-pinning issue named in the CURRENT BSL
    surface bullets (that bullet is about `TickReport` not carrying a log for goldens to compare
    against; this is about there being no in-tick read-back capability at all, for any purpose).
  - `resolve_rng(services, tick)` — deterministic `random.Random` seeding, no BSL equivalent
    needed to name (BSL's own RNG story is presumably a separate, already-solved seam; not
    investigated further here as it's shared scaffolding, not `reactionary.py`-specific).
- **DORMANCY on canonical scenarios** (`tools/regression_scenarios.py`, 2925 lines, read for the
  `SCENARIOS` registry and `COVERAGE_GAPS_DATA`):
  - **Drift (Pass 1, entitled-role pull/alignment) IS LIVE** on the `fascist_bifurcation` scenario
    (`SCENARIOS["fascist_bifurcation"]`, `regression_scenarios.py:63-70`; factory
    `create_imperial_circuit_scenario`, `_legacy.py:255-454`) — its C002 (`COMPRADOR_BOURGEOISIE`)
    and C004 (`LABOR_ARISTOCRACY`) are both entitled-role, role-defaulted entitlement
    (`social_class.py:475-479`), zero-solidarity by scenario design (the whole point of the
    scenario is to exercise wage-crisis → national-identity routing under zero solidarity
    pressure) — `FascistFactionSystem`/`fascist_drift` is explicitly named as a live claim in
    `regression_scenarios.py:1645-1650`.
  - **Capture (the `aligned_faction_id` write) is DORMANT on every scenario in `SCENARIOS`.**
    `apply_balkanization_seed` (the sole production FACTION-node seeder,
    `balkanization_seed.py:30-32`) is "NOT applied by any of the six qa:regression scenario
    factories — FACTION/SOVEREIGN nodes land only in the electoral/balkanization scenarios."
    Conversely, the electoral goldens (`weimar`/`mitterrand`/`syriza`/`debs`/`bernie_valve`, via
    `electoral_fixture.py`/`electoral_goldens.py`) DO seed FACTION nodes, but layer their political
    terrain onto substrates (`two_node` for weimar/debs/bernie_valve; Wayne `single_county` for
    mitterrand/syriza) whose `SocialClass` roles are `PERIPHERY_PROLETARIAT`/`CORE_BOURGEOISIE` —
    **neither is `_ENTITLED_ROLES`** — grep-confirmed `electoral_fixture.py` adds zero
    `SocialClass` nodes of its own (`rg -n 'SocialClass\(|SocialRole\.' electoral_fixture.py`:
    zero hits); it only reaches the substrate's pre-existing two classes. No canonical scenario
    anywhere has BOTH an entitled-role class that can saturate `fascist_alignment` AND a seeded
    FACTION node in the same graph. `FASCIST_RECRUITMENT` is therefore structurally dormant on
    every scenario in `SCENARIOS` today — a fresh finding, not previously declared in
    `COVERAGE_GAPS_DATA`.
  - **Chauvinism accrual (Pass 2, unconditional half) IS LIVE on `mitterrand`/`syriza`.** Those
    two stand on the Wayne `single_county` substrate, whose "worker" id (`LABOR_ARISTOCRACY_ID` =
    `"C004"`, role `LABOR_ARISTOCRACY` — `single_county.py:82-93`) is bound as
    `electoral_goldens.py`'s `_WAYNE_OWNER = "C004"` (`electoral_goldens.py:46-48`) — the
    parameter `apply_political_terrain` uses to target the duopoly + fascist party MEMBERSHIP
    edges (`_membership("org/party-restorationist", owner_id)`,
    `_membership("org/party-fascist", owner_id)`, `electoral_fixture.py:184-187`). So on
    mitterrand/syriza, `org/party-restorationist` → `C004` and `org/party-fascist` → `C004` ARE
    real `LABOR_ARISTOCRACY`-target MEMBERSHIP edges, and `_accrue_chauvinism` runs unconditionally
    every tick for each. **`weimar`/`debs`/`bernie_valve` do NOT exercise this** — their `two_node`
    substrate's classes are `PERIPHERY_PROLETARIAT`/`CORE_BOURGEOISIE`, neither
    `LABOR_ARISTOCRACY`, so `_process_org_defections`'s role filter
    (reactionary.py:249) drops every MEMBERSHIP edge on those three.
  - **Defection rolls / `RED_BROWN_COUP` (Pass 2, crisis-gated half): UNVERIFIED whether they
    actually fire on mitterrand/syriza** — the structural precondition (a real MEMBERSHIP edge to
    an LA-role node) holds, but firing additionally requires an `ECONOMIC_CRISIS`/
    `SUPERWAGE_CRISIS`/`CRISIS_PHASE_TRANSITION` event in the SAME tick's event-bus history. The
    search run: `rg -n 'EventType\.(ECONOMIC_CRISIS|SUPERWAGE_CRISIS|CRISIS_PHASE_TRANSITION)'
    src/babylon/engine/systems/*.py src/babylon/domain/economics/tick/system/__init__.py` finds
    the writers (`economic.py:470,737`; `decomposition.py:182`;
    `domain/economics/tick/system/__init__.py:1089`) and confirms they run at positions before
    17.4 (`ImperialRentSystem` @9.0; `TickDynamicsSystem`, earlier still), so the events CAN
    structurally exist in-tick — but whether mitterrand/syriza's specific fiscal calibration
    (`t_claim ≈ $175.7M`, `total_surplus ≈ $3.23B`, endogenous interest ≈ 1.78%,
    `electoral_goldens.py:8-13`) actually crosses those crisis thresholds on any tick is a runtime
    question this read-only inventory cannot answer without executing the scenario. Flagged
    honestly as UNVERIFIED, not silently assumed either way — a port's hand-built or harvested
    conformance fixture needs to check this directly.

---

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in the task (query lane Slice 1 landed via
`run_once_into`; Slices 2-4 — edge-attribute reads, hyperedge/metric lane, attribute-storage
widening — NOT built; enum fields landed with `field-of` refused on them; `deffield` type
vocabulary closed at `{int, bool, currency, probability, intensity, coefficient, enum}`, no
`str`/dict/list; `exp`/`log`/`floor` declarable; no imposed functional forms; `emit` write-only,
no event log in `TickReport`; two rules at one anchor position don't yet share pre-state).

| Computation | Verdict | Detail |
|---|---|---|
| Entitled-role gate + node-local reads (`active`, `role`, `ideology.agitation`, `entitlement`, `fascist_alignment`, `aligned_faction_id`) | **PORTABLE NOW** | Plain per-node `nodes`/`for-each` iteration with `field-of` over each node's own `NodeRef` — exactly Slice 1's landed shape. `agitation`'s `[0,∞)` unbounded domain needs the same bare-scaled-Int workaround Territory's `rent_level` and Metabolism's `entropy_factor` used (ADR183 declared-deviation class) — a D-record, not a blocker, and one that any port of `ConsciousnessSystem` (the field's writer) will also need to make, not a decision unique to this system. |
| `Fascist_Pull` formula (`agitation × entitlement / (solidarity + ε)`) | **BLOCKED — query lane Slice 2 (edge-attribute reads)** | The division/multiply themselves are trivial arithmetic. The blocker is `_incident_solidarity`'s dependency on `SOLIDARITY.solidarity_strength`, an edge-scoped attribute. `field-of` serves `NodeRef` referents only today; "no expression form produces [an `EdgeRef`] yet — slice 2 mints `EdgeKey`" (`evaluator.rs:1185-1191`, verified verbatim). Structurally the fold itself (`max` over incident edges) maps CLEANLY onto `fold max` over a typed `neighbors`/`edges` query — a favorable match, exactly the kind Territory's Phase-3 spillover found — but the edge-attribute READ underneath it does not exist yet. Name the exact lane: **Slice 2**. |
| Drift write + upper-clamp (`min(1.0, alignment + step)`) | **PORTABLE WITH D-RECORD** | `update-node` with a `set`/`add` op against `self`, clamp expressed as nested `if` (no scalar `min` in the grammar, same precedent as every landed pack). D-record: the bare `1.0` literal needs `c`-suffixing. |
| `StanceIntervention` construction + write to `opposition_interventions` | **BLOCKED — no graph-level composite-attribute storage lane** (NEW finding) | `deffield` declares a scalar field on a NODE type (`<deffield> ::= "(" "deffield" <qname> ":type" <type-name> ...`, `bsl-language.rst:1635-1640`) — there is no graph-scoped (not per-node) field declaration form at all, and the closed type vocabulary has no dict/list/record type to hold `{target_key, delta_balance, source}` triples, let alone a GROWING list of them. `(domain :graph)` (`bsl-language.rst:726-732`) is a **rule firing-multiplicity** modifier ("fires exactly once per tick"), not a storage mechanism — confirmed by reading its full definition. This is a distinct, deeper gap than Slice 2/3/4: those are missing ACCESSORS over an existing shape; this is a missing SHAPE. Name it: **graph-level composite-attribute storage — no lane exists, not even a numbered slice.** |
| Capture predicate, settler+uphold disjunct (`is_settler_formation AND colonial_stance == UPHOLD`) | **PORTABLE WITH D-RECORD** | `is_settler_formation` is `bool`, `colonial_stance` is a 3-member enum — both in the landed `deffield` vocabulary; enum comparison via `=` is landed (ADR195/196). `(nodes NodeType/FACTION <node-pred>)` supports exactly this filter shape (`<query> ::= "(" "nodes" <enum-ref> <node-pred>? ")"`, `bsl-language.rst:944`). D-record: the ideological content itself is RESERVED-LINE (Director sign-off needed on the encoding, not an engineering call). |
| Capture predicate, ideology-substring disjunct (`any(tok in ideology.lower() for tok in (...))`) | **BLOCKED — no string field type** | `deffield`'s type vocabulary is exactly `{int, bool, currency, probability, intensity, coefficient, enum}` — no `str`/string type at all, confirmed against `bsl-language.rst` §2.9's full grammar. `FACTION.ideology` is a free-text `str` (1-64 chars, `balkanization_faction.py:64`) with a SUBSTRING match against it — even an enum-encoding workaround (closed set, like Territory's `territory_type`) would not naturally express "contains token," since the match target is unbounded free text, not a fixed vocabulary. A port would need to PRE-COMPUTE the token-match result at content-authoring time (a derived bool field set when the FACTION is seeded) — moving the computation out of the engine, a genuine port-time design decision, not a mechanical transcription. **This exact code path is also where the verified defect below lives** — faithfully transcribing the DEFECT (port-as-is law) is not even possible without the same missing string-type lane, since the defect IS the substring match's behavior. |
| `min(candidates)` faction tie-break | **PORTABLE, favorable reformulation** | `select-max`/`select-min`'s tiebreak is "a property of the language, not of each rule... the first element in ascending id byte order wins" (`bsl-language.rst:1276-1280`, verified verbatim) — running `select-min` over the (filtered) FACTION query with a CONSTANT score expression makes every candidate tie, so the language's own built-in tiebreak becomes the entire selection — mathematically identical to Python's `min(candidates)`. This piece is portable in principle; it's gated entirely on the query filter being expressible, which needs the ideology-substring blocker above resolved first. |
| `_org_discipline` (read `cadre_level`, fallback to define) | **PORTABLE NOW** | Plain `field-of` over the org's `NodeRef` with a default-fallback `if`, `Probability`-typed. |
| `_is_superwaged` / `super_wage_bonus` read | **BLOCKED — query lane Slice 2 (edge-attribute reads)** | Same exact lane as `_incident_solidarity` — reading a scalar off a `WAGES` edge via `EdgeRef`. |
| `_accrue_chauvinism` write (`update_edge(..., chauvinism=new)`) | **BLOCKED — no edge attribute storage** (deeper than Slice 2) | Even setting the read-side gap aside, `update-edge` is grammar-recognized and **structurally refused at execution**: "`GraphSubstrate` has no storage for [a named edge field]: its edge state is one `f64` strength keyed by `(type, from, to)`... giving them storage widens the substrate's state... a **declared substrate gap**, escalated rather than silently absorbed" (`structural_verbs.rs:16-26`, verified verbatim; refusal sites `structural_verbs.rs:371,693`). This is a write-side blocker independent of Slice 2's read-side gap — even a future Slice 2 landing would not by itself unblock a MUTATING, per-tick-accruing edge field. |
| `calculate_defection_probability` (the sigmoid) | **PORT-QUESTION, not auto-portable** | `1.0 / (1.0 + exp(-(chauvinism - discipline)))` is a stipulated logistic form squarely inside the ADR172 ruling-5 / ADR173 "no imposed functional forms" prohibition. It ALSO carries a `math.exp` libm-nondeterminism hazard, and it ALSO needs the (blocked) `chauvinism` edge read as an input. Even if the edge-read and storage gaps above were closed, this formula cannot land as a direct transcription — it needs the same re-derivation-as-emergent-measure treatment ADR173 already prescribes for P(S\|A), which is a design decision for the Director/architecture line, not a mechanical port move. |
| RNG-gated defection roll + `RED_BROWN_COUP` threshold | **BLOCKED (downstream of the two blockers above)** | The roll itself (`rng.random() < p_defect`) and the coup-fraction comparison are simple once `p_defect` and `len(edges)`/`defections` (an edge-filtered `fold count`) exist, but both inputs are blocked upstream (edge storage + the sigmoid PORT-QUESTION). |
| `_crisis_this_tick` (event-bus history read) | **BLOCKED — no event read-back lane** (NEW finding, distinct from the WS1 ledger note) | `emit` is write-only into an `EventSink` (`structural_verbs.rs:68-82`); grep-confirmed no expression form anywhere in `evaluator.rs`/`structural_verbs.rs`/`declarations.rs` queries prior emissions. This blocks the entire crisis-gate, independent of the chauvinism-storage and sigmoid blockers. |
| `_read_regime` (`dialectical_regime` read, observability-only) | **BLOCKED — same graph-level composite-attribute gap as `opposition_interventions`** | Read-only, and the system's own docstring says it "NEVER gates dynamics" — lowest-stakes of the three composite-attribute blockers, but blocked by the identical missing storage shape. A port could legitimately DROP this field from the FASCIST_DRIFT payload (it is observability-only, per the frozen code's own words) rather than solve graph-level storage just to carry a debug annotation — a port-time scoping decision, not a forced blocker resolution. |

**Verdict rollup:** two computations are honestly PORTABLE NOW (node-local drift-gate reads,
`_org_discipline`); three are PORTABLE WITH D-RECORD (the drift write's clamp, the settler+uphold
capture disjunct, the tiebreak reformulation once its filter is unblocked); everything else is
BLOCKED on one of three distinct lanes — **Slice 2 (edge-attribute reads)**, **graph-level
composite-attribute storage (no lane at all today)**, and **event read-back (no lane at all
today)** — plus two PORT-QUESTION rows that need a Director/architecture ruling before any
transcription decision (the imposed sigmoid; the ideology-substring content-modeling move) rather
than an engineering fix.

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_fascist_faction_system.py` | 313 | **Primary conformance oracle.** Direct `FascistFactionSystem().step()` unit coverage: drift (crisis-pull, hegemony-zero-agitation, solidarity-suppression, comprador-drift), capture (saturation→capture, idempotence, no-faction-no-capture), the `StanceIntervention` hook (write + no-write-when-opposition-absent), chauvinism (base-rate, superwage-bonus), defection+coup (fires, no-crisis-suppresses, majority→coup, no-defections→no-coup, determinism), and the `dialectical_regime` read. **Notably absent:** no test seeds MORE THAN ONE FACTION node, so the `min(candidates)` tie-break ambiguity (§6 defect) is entirely untested — every fixture uses a single `FAC_SETTLER` faction in isolation. |
| `tests/unit/engine/systems/test_reactionary_crisis.py` | 95 | Full-engine (`_DEFAULT_SYSTEMS`) induced-crisis integration: sustained re-injected agitation over 25 ticks drives drift→saturation→capture end-to-end, plus a determinism check (same scenario twice → same event-count signature). A genuine end-to-end conformance candidate, though its fixture (hand-built, wealth=$1M to survive with no income circuit) is synthetic, not a canonical scenario. |
| `tests/unit/formulas/test_reactionary.py` | 142 | Pure-formula tests. `TestFascistPull`/`TestDefectionProbability` cover the two formulas THIS system invokes — genuine conformance candidates for those two formulas in isolation. `TestSpontaneousRiotRisk`/`TestEntitlementEffective`/`TestRLFSimplex` test formulas/functions this system does NOT call (`calculate_spontaneous_riot_risk` belongs to `StruggleSystem`; `calculate_entitlement_effective` is dead code; the RLF-simplex functions live in `consciousness_routing.py`) — out of scope for this system's own conformance set, though co-located in the same test file. |
| `tests/unit/config/test_reactionary_defines.py` | 48 | `ReactionaryDefines` contract: wiring into `GameDefines`, default values match the catalog, frozen, YAML-override round-trip. Schema-level, not tick-behavior. |
| `tests/unit/models/test_social_class_reactionary.py` | 113 | `SocialClass.entitlement`/`.volatility`/`.fascist_alignment`/`.aligned_faction_id` field defaults, role-defaulting, bounds enforcement, and graph round-trip preservation (confirms these 4 fields DO survive `to_graph`/`from_graph`, unlike `chauvinism`/`super_wage_bonus`). Schema-level. |
| `tests/unit/engine/test_system_order.py` | 300 | Verifies the full 34-system tick ordering, including `FascistFactionSystem` at position 17.4 by name (`test_system_order.py:85,189,254`). Integration-boundary test, not behavioral. |
| `tests/unit/game/test_chronicle_adapter.py` | 531 | AI-narrative layer consuming `FASCIST_DRIFT` payloads (`test_chronicle_adapter.py:66,511,520`). Narrative, not conformance. |
| `tests/unit/projection/test_field_state.py` | 268 | `observe()`-projection layer tests confirming `fascist_alignment` survives the field-state projection even when it's the only non-default attribute on a node (`test_field_state.py:34-68,254`). Projection contract, not engine math. |
| `tests/unit/ooda/test_action_effects.py` (988 lines) / `tests/unit/ooda/test_reactionary_ooda_verbs.py` (163 lines) | — | Test the ADJACENT `src/babylon/ooda/action_effects.py` module (POGROM/VIGILANTISM/LOCKOUT), which consumes the SAME `ReactionaryDefines` category but is **not invoked by `FascistFactionSystem.step()`** — a separate production module, separate port scope. Listed here only because a naive `reactionary`/defines-category grep would otherwise surface them as false positives; `tests/integration/mechanics/test_ideological_bifurcation.py` and `tests/integration/mechanics/test_imperial_dynamics.py` are the same kind of false positive (bare English word "reactionary" describing ideology values, verified by direct read — zero actual `FascistFactionSystem`/formula references). |

**qa:regression byte-gate coverage.** Per §5's dormancy findings: the byte-identical hash gate
(`tools/regression_test.py::graph_content_hash`) covers this system's Pass-1 drift arm for real
on `fascist_bifurcation` (one of the 11 canonical scenarios), and covers Pass-2's unconditional
chauvinism-accrual half for real on `mitterrand`/`syriza`. It has **zero coverage** of
`FASCIST_RECRUITMENT`/capture (no scenario seeds both an entitled-role class and a FACTION node
together) and **unverified** coverage of the crisis-gated defection/coup half (structurally
possible on mitterrand/syriza, not confirmed to actually fire without running the scenario). A
port's conformance fixtures for capture and for the crisis-defection arm will need to be
hand-built, following the same precedent Territory's inventory and PR #509-520's query-lane train
already established.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) with fresh `rg`/Read. Six corrections,
six confirmations. The report's Rust-side reads are excellent and verbatim-accurate; its
canonical-estate dormancy section is where it goes wrong, and it goes wrong in a way that makes the
verdict harsher than the tree supports on one axis and softer on another.

### CORRECTIONS

1. **CORRECTION — §5's capture-dormancy claim is self-contradicted and wrong.** It asserts: "No
   canonical scenario anywhere has BOTH an entitled-role class that can saturate `fascist_alignment`
   AND a seeded FACTION node in the same graph. `FASCIST_RECRUITMENT` is therefore structurally
   dormant on every scenario in `SCENARIOS`." Both preconditions co-occur on `mitterrand` and
   `syriza`: (a) `single_county.py:82-93` seeds `worker = SocialClass(id=LABOR_ARISTOCRACY_ID, …,
   role=SocialRole.LABOR_ARISTOCRACY, …)` — a member of `_ENTITLED_ROLES`
   (reactionary.py:54-56, `{LABOR_ARISTOCRACY, COMPRADOR_BOURGEOISIE}`); (b) both goldens layer
   `apply_political_terrain`, which calls `apply_balkanization_seed(state)` at
   `electoral_fixture.py:204`, minting all four `BalkanizationFaction` nodes. The inventory's own
   NEXT bullet states (a) explicitly ("`LABOR_ARISTOCRACY_ID` = `C004`, role `LABOR_ARISTOCRACY` —
   `single_county.py:82-93`"), so the two bullets contradict each other. Correct status: capture is
   **structurally LIVE** on `mitterrand`/`syriza`; whether `fascist_alignment` actually reaches
   `fascist_recruitment_threshold = 1.0` (twenty `fascist_drift_step = 0.05` bumps) inside the run
   horizon is **UNVERIFIED**, not "structurally dormant". `two_node`-substrate goldens
   (`weimar`/`debs`/`bernie_valve`) are correctly excluded — `_legacy.py:83,98` seeds only
   `PERIPHERY_PROLETARIAT`/`CORE_BOURGEOISIE`.

2. **CORRECTION — the `FAC_DECOLONIAL` defect is LIVE on the byte-gated estate, not a latent
   ambiguity, and its class is RESERVED-LINE, not port-as-is.** `seed_factions.json` (read in full)
   seeds four factions. `_find_fascist_faction` (reactionary.py:215-227) admits two of them:
   `FAC_RESTORATIONIST` (`is_settler_formation: true` + `colonial_stance: "uphold"` → settler_uphold;
   also `"settler" in "settler-restorationism"`) and **`FAC_DECOLONIAL`** (`is_settler_formation:
   false`, `colonial_stance: "abolish"` — but `"settler" in "anti-settler abolitionism"` → token
   match). `FAC_WORKERS_CONGRESS` and `FAC_LIBERAL_IMPERIAL` are correctly excluded
   (`colonial_stance: "ignore"`, no token). `min(candidates)` then returns `"FAC_DECOLONIAL"`
   (`D` < `R`). So on all five scenarios that run `apply_balkanization_seed`, the node the engine
   calls "the fascist faction" and writes into every captured class's `aligned_faction_id` is the
   **anti-colonial front**. The inventory rates this a "verified frozen-code defect" for
   transcription; the correction is its class and blast radius — an anti-colonial formation
   mislabelled fascist is Constitution I.1/I.4 territory, and combined with correction 1 it now sits
   on the live canonical estate. This belongs on a **Director-escalation row**, not a port-as-is
   transcription row, and it should not be silently reproduced by the port under port-as-is law
   without that ruling.

3. **CORRECTION — "graph-level composite-attribute storage — no lane exists, not even a numbered
   slice" is half wrong.** `docs/reference/bsl-language.rst:2650-2688` (§3.6, draft ruling, R9
   chapter C3) rules exactly this case: *"Graph-scope state is ordinary node state on a declared
   carrier node type … A value of graph scope is declared as an ordinary ``deffield`` owned by a
   **carrier node type** — a ``NodeType`` member whose manifest ``:ceiling`` is 1 — read with
   ``(field-of (the NodeType/…) …)`` and written with ``(update-node (the NodeType/…) …)``."* The
   lane exists and is named; its accessor is `("the", "slice 2")` at `evaluator.rs:506`. Split the
   row three ways rather than one: (a) `opposition_known` (a key-EXISTENCE test collapsing to one
   bool) and (b) `_read_regime`'s single scalar are **carrier-node + Slice-2** shapes, not lane-less;
   (c) only `opposition_interventions` — a GROWING list of `{target_key, delta_balance, source}`
   records — is genuinely un-laned, and even there the ruling's own second half ("it does not make
   every register a singleton: per-sovereign and per-county registers are ordinary nodes of ordinary
   types, reached by ordinary queries", :2686-2688) points at an intervention-as-node re-modelling.

4. **CORRECTION — the stance-intervention arm is provably DEAD on the entire canonical estate, and
   the blocker table never says so.** `opposition_known` (reactionary.py:96-98) tests
   `"capital_labor" in graph.get_graph_attr("opposition_states", {})`. `WorldState.opposition_states`
   is a **write-only seed**: `to_graph` stamps it (world_state.py:721), `from_graph` never
   reconstructs it (field docstring, world_state.py:539-552: "``from_graph`` does NOT reconstruct
   it"), `simulation_engine.step()` round-trips every tick (:552 / :606), and no `SCENARIOS` factory
   seeds it (`rg` across `engine/scenarios/*.py` + `tools/regression_*.py`: zero hits). Therefore
   `opposition_known` is `False` on every canonical tick, `_write_stance_intervention` never fires,
   and `opposition_interventions` is never written. The honest port declares it `:const false` in the
   Metabolism-D-2 / Territory-`displacement_mode` "provably uniform" class — at which point the
   *entire* "graph-level composite-attribute storage" blocker (the report's headline NEW finding)
   **drops out of the port's critical path**. That is a materially different verdict shape than
   "BLOCKED on a wholly new lane": the lane is still owed, but it does not gate this port.

5. **CORRECTION — "one of the 11 canonical scenarios" is 12.** `tools/regression_scenarios.py`'s
   `SCENARIOS` has twelve keys (…, `bernie_valve`, `org_probe`); `tests/baselines/` carries twelve
   matching JSON baselines and twelve dense CSVs; `regression_test.py:1363,1424,1777` iterate the
   dict wholesale.

6. **CORRECTION — the file is 354 lines, not 355** (`wc -l src/babylon/engine/systems/reactionary.py`
   → 354). Trivial, but it is carried in both the executive summary and §1's table.

### CONFIRMATIONS

7. **CONFIRMATION, verbatim — the `update-edge` write blocker.** `structural_verbs.rs:15-27` reads:
   *"``update-edge`` and ``update-hyperedge`` (R9 chapters C2/C12, D35/D65) are recognised here and
   refused loudly, with the reason named … ``GraphSubstrate`` has no storage for either: its edge
   state is one ``f64`` strength keyed by ``(type, from, to)`` … a **declared substrate gap**,
   escalated rather than silently absorbed."* Independently verified against the trait itself:
   `substrate.rs:111-117` (`add_edge(edge_type, from, to, strength: f64)`), `:124` (`remove_edge`),
   `:166` (`edges -> Vec<(NodeId, NodeId)>`), `:141` (`node_attribute`, with **no** edge
   counterpart). `_accrue_chauvinism`'s per-tick MEMBERSHIP field write is unreachable even past
   Slice 2. Row stands.

8. **CONFIRMATION, SHARPENED — `_is_superwaged` is a harder blocker than "same exact lane as
   `_incident_solidarity`".** SOLIDARITY needs exactly one named attribute per edge type
   (`solidarity_strength`), which the substrate's single `:strength` slot could in principle carry
   once Slice 2 mints a reader. WAGES needs `value_flow` (ConsciousnessSystem's `core_wages`) **and**
   `super_wage_bonus` on the *same* edge type — two named fields on one edge, which a one-`f64`-per-
   edge substrate cannot hold regardless of Slice 2. Name the lane precisely: **Slice 2 + edge-
   attribute storage widening**, the same hash-touching class as the `update-edge` refusal above.

9. **CONFIRMATION — the event read-back gap is genuinely new and distinct from the WS1 note.**
   `TickReport` (`babylon-tick/src/lib.rs:29-48`) carries `before`, `after`, `fired`,
   `per_rule_fired` and nothing else; `emit` runs into `EventSink`/`CollectingSink`
   (`structural_verbs.rs`, re-exported at `lib.rs:71`); there is no expression-position accessor
   anywhere that queries prior emissions. `_crisis_this_tick`'s in-tick
   `services.event_bus.get_history()` filter has no BSL analogue. The report is right that this is
   a second, separate gap from "goldens cannot pin emissions".

10. **CONFIRMATION — the imposed-sigmoid PORT-QUESTION.** `formulas/reactionary.py:89-91`:
    exponent clamp to `[-500, 500]` then `1.0 / (1.0 + math.exp(exponent))`, imposed directly as the
    defection mechanic. ADR172 ruling 5 / ADR173 class, same treatment ADR173 already prescribes for
    P(S|A). Escalation verdict correct.

11. **CONFIRMATION — tick position 17.4** (reactionary.py:78), ordering per `_SYSTEM_CLASSES`
    (simulation_engine.py:328-363). And **CONFIRMATION** that `min(candidates)` reformulates
    faithfully as `select-min` with a constant score expression, riding the language-level
    ascending-id tiebreak.

12. **CONFIRMATION with an addendum — byte-gate reach.** The report does not state what
    `graph_content_hash` actually covers for this system; for the record: it hashes
    `state.to_graph()`'s nodes/edges only and excludes graph metadata
    (`tools/regression_test.py:924-964`, docstring). So `MEMBERSHIP.chauvinism` and
    `WAGES.super_wage_bonus` — neither a declared `Relationship` field, per §3's own round-trip-lossy
    finding — are outside the gate on top of the facade reset the docstring names; and
    `opposition_interventions` / `dialectical_regime` are `g.graph` metadata, also outside. What IS
    gated is `SOCIAL_CLASS.fascist_alignment` and `.aligned_faction_id` (both declared model fields,
    round-trip-verified by `tests/unit/models/test_social_class_reactionary.py`). Note the asymmetry
    this creates with correction 2: the defect's *effect* (`aligned_faction_id = "FAC_DECOLONIAL"`)
    **is** byte-gated the moment capture fires.

### FINAL VERDICT

**BLOCKED — on a narrower and better-named set than claimed. The real gating lanes are (i) query-lane
Slice 2 *plus* the `GraphSubstrate` edge-attribute reader it presupposes, for `_incident_solidarity`
(one named field per edge type), (ii) Slice 2 + edge-attribute storage widening for
`_is_superwaged`/`_accrue_chauvinism` (two named fields on one edge; `update-edge` is already a
declared substrate gap), and (iii) event read-back, which has no lane at all. The "graph-level
composite-attribute storage" blocker is DEMOTED: §3.6's carrier-node ruling names the lane for the
scalar cases, `the` puts it in Slice 2, and `opposition_known` is provably `False` on all twelve
canonical scenarios — so the stance-intervention arm is a `:const false` in the Metabolism-D-2 class
and does not gate this port. Two escalations stand and one is upgraded: the imposed sigmoid
(ADR172/173), and the `FAC_DECOLONIAL` mis-selection, which is LIVE on the five balkanization-seeded
goldens and is a RESERVED-LINE Director item, not a port-as-is transcription row.**

### INADEQUATE-COVERAGE NOTE

§5's DORMANCY subsection must be re-read. A re-read must: (a) re-derive the capture precondition
against `single_county.py:82-93` and `electoral_fixture.py:204`, and state `mitterrand`/`syriza` as
structurally-live-but-dynamically-UNVERIFIED; (b) trace `opposition_states`' write-only round-trip
(world_state.py:539-552, :721, simulation_engine.py:552/606) and re-verdict the stance-intervention
arm as provably dead; (c) enumerate which of this system's writes `graph_content_hash` actually
reaches. Everything else in the report — the Rust-side reads especially — needs no re-work.
