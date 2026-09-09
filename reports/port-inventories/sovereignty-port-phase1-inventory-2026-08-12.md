# SovereigntySystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `SovereigntySystem` (160 lines, tick position 17.5, Consequences
phase) is the smallest and cleanest system inventoried in this train: one file, one
formula call, zero graph-state writes, zero libm calls, zero Real→Int demotions, zero
clamp logic. But every one of its real computations — effective-controller resolution,
metabolic-impact assignment, dual-power detection — keys off `control_level`/
`legal_status`, attributes stored **on the CLAIMS edge**, not on any node. BSL's landed
query-evaluation Slice 1 (`field-of`, `select-max`, `fold`, `neighbors`) serves `NodeRef`
referents only; edge-attribute reads (`EdgeRef`, Slice 2) are not built
(`evaluator.rs:1190-1196`). That is the single blocking dependency for this whole system.
A significant, verified correction to the estate's own coverage-gap ledger: five of the
twelve canonical `qa:regression` scenarios (mitterrand/syriza/weimar/debs/bernie_valve)
**do** seed real Sovereign nodes + a live CLAIMS edge onto Wayne territory — the
declared-dormant claim in `tools/regression_scenarios.py:2776-2782` is stale.

**Verdict:** BLOCKED on BSL Query-Evaluation Slice 2 (edge-attribute reads / `EdgeRef`) —
every computation in `step()` reads a CLAIMS-edge attribute; once Slice 2 lands, the
system's other two obstacles (cross-tick `persistent_data` storage, the enum-typed
`extraction_policy` read off a dynamically-resolved node) are both D-record-portable, not
blocked.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/sovereignty.py` | 160 | **The target.** `SovereigntySystem`, one `step()` method, no phases/sub-methods besides the `_coerce_policy` staticmethod. |
| `src/babylon/formulas/balkanization.py` | 404 | `calculate_metabolic_impact` (lines 31-74) is the ONLY function this system calls out of this module (verified: `derive_extraction_policy_from_stance` and the rest of the module's ~10 other functions belong to FactionInfluenceSystem/CollapseTransitionSystem, grep-confirmed zero other imports from `sovereignty.py`). |
| `src/babylon/config/defines/balkanization.py` | 176 | `BalkanizationDefines` Pydantic model — SovereigntySystem reads only 3 of its ~20 fields (`metabolic_impact_intensify/continue/cease`, lines 46-57). |
| `src/babylon/data/defines.yaml` | (balkanization block: lines 317-350; the 3 consumed keys at 318-320) | Player-editable coefficient values. |
| `src/babylon/models/enums/balkanization.py` | 185 | `ExtractionPolicy` (lines 54-70, 3-member StrEnum) is read by SovereigntySystem. `ClaimLegalStatus` (115-136, 5-member) is the *declared* domain for `legal_status` but is **not** wired to the `Relationship.legal_status` field's type (see §3) and is never imported by `sovereignty.py`. `SovereigntyType` likewise unused here. |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.TERRITORY` (61), `NodeType.SOVEREIGN` (66), `EdgeType.CLAIMS` (125). |
| `src/babylon/models/entities/sovereign.py` | 118 | `Sovereign` Pydantic entity — field types/domains for `extraction_policy`; also carries its own `metabolic_impact` `@computed_field` (lines 76-92) that calls the *same* `calculate_metabolic_impact` formula independently of the System (a second call site, not read by `sovereignty.py`, worth noting for the port dossier so the two aren't conflated). |
| `src/babylon/models/entities/relationship.py` | 186 | `control_level`/`legal_status` field declarations (lines 134-143) — the CLAIMS-edge payload. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._wrap_graph` (99-117) and `._publish` (194-196) — the only two `SystemBase` helpers this system calls. It does **not** call `_write_clamped` or `_read` (no graph writes, see §4). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.get_node` (77), `.query_nodes` (258), `.query_territory_claims` (415-432, the CLAIMS-specific accessor). |
| `src/babylon/topology/graph.py` | 1033 (relevant: 942-972) | Concrete `BabylonGraph.query_territory_claims` (942-959) — the sort key `(-control_level, sovereign_id)` (958) that implements FR-020's tiebreak; `.query_adjacent_territories` (961-972, unused by Sovereignty) and `.bulk_partition_claims` (974-1001, used by CollapseTransitionSystem, not Sovereignty). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` (28-30). |
| `src/babylon/kernel/event_bus.py` | 288 (relevant: 33-53) | `Event` frozen dataclass. |
| `src/babylon/engine/context.py` | 113 | `TickContext.persistent_data` — the one non-graph read/write surface this system touches (both directions: it reads nothing from context besides `.tick`, but writes two keys into `.persistent_data`). |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` (328-364) confirms registry membership; `_DEFAULT_SYSTEMS` (376-378) is DERIVED by sorting on `.position`, so registry-declaration order ≠ tick order (load-bearing for §5); `step()` (502-589), specifically lines 564-580, is where `persistent_context` gets threaded across ticks — the mechanism behind the cross-tick channel documented in §5. |
| `src/babylon/models/world_state.py` | (relevant: 316-332, 775-799, 981-1042) | `to_graph()._add_political_nodes` (783-788) materializes `state.sovereigns`/`state.factions` as real graph nodes with `_node_type` stamps; `_reconstruct_sovereign` (316-332) is the graph→WorldState round-trip. |
| `src/babylon/models/events/balkanization_payloads.py` | 134 (relevant: 93-99) | `DualPowerActivePayload` — typed Pydantic mirror of the raw `Event`. |
| `src/babylon/engine/event_builders.py` | 782 (relevant: 280-286) | `EVENT_BUILDERS[EventType.DUAL_POWER_ACTIVE]` — raw `Event.payload` dict → typed `DualPowerActivePayload`. |
| `src/babylon/engine/systems/metabolism.py` | (relevant: 74-88) | **The only real downstream consumer** of `persistent["balkanization.metabolic_impact_by_territory"]` — see §5 for the cross-tick-lag finding this file makes verifiable. |
| `src/babylon/engine/systems/policy.py` + `src/babylon/domain/politics/governance_endgame.py` | (relevant: policy.py 235,488,495; governance_endgame.py 81-96) | An **independent, duplicate** reimplementation of the "≥2 active claimants" predicate (`dual_power_live`), reading the same `query_territory_claims` this system reads — see §5. |
| `src/babylon/engine/scenarios/balkanization_seed.py` | 163 | `apply_balkanization_seed` — the engine-side (non-web) Sovereign/CLAIMS seeding used by 5 of the 12 canonical scenarios (§5). |
| `src/babylon/engine/scenarios/electoral_fixture.py` | 278 | `apply_political_terrain` (113-262) — calls `apply_balkanization_seed` unconditionally (line 204), before the `include_michigan` branch. |
| `src/babylon/data/game/balkanization/seed_sovereigns.json` | 87 | The concrete seed data: `SOV_USA_FED` (`extraction_policy: intensify`, a literal claim on FIPS `26163` = Wayne, `control_level: 1.0`), `SOV_CAN_FED`, `SOV_EXTERIOR_NULL`. |
| `tools/regression_scenarios.py` | (relevant: 37-135 `SCENARIOS`, 2707-2782 `COVERAGE_GAPS_DATA`) | The canonical-scenario registry and its (stale, §5) SovereigntySystem coverage-gap declaration. |

**Not exercised by `sovereignty.py` at all:** no `src/babylon/domain/*` module (it calls exactly
one `formulas/` function). This is, after Territory, the second-cleanest file-map in the
estate — but unlike Territory, this system's core data lives on **edges**, not nodes, which
is the load-bearing difference for §6.

**Reference BSL/spec material read for this inventory (fully read):**
- `docs/reference/bsl-language.rst` §2.10 (Element accessors, lines 1805-1874), §3.1 enum
  typing + the `field-of` deferral / D102 (2247-2284), §3.6 the graph-scope-state ruling
  (2650-2685).
- `rust/crates/babylon-bsl/src/evaluator.rs` lines 486-527 (`UNSERVED_EXPRESSION_HEADS` /
  `SERVED_QUERY_HEADS`) and 1190-1213 (`eval_field_of`'s `NodeRef`-only scope).
- `rust/crates/babylon-bsl/src/declarations.rs` line 110 (`DECLARABLE_INTRINSICS`).
- `rust/crates/babylon-tick/content/rules/vitality.bsl` lines 45-64 (the bare-literal /
  Real-zero-promotion idiom, for §4).

## 2. COMPUTATION CATALOG (execution order, `sovereignty.py:86-147`)

`step()` is a single loop over territories with two logically distinct computations
interleaved, followed by two write phases. There are no named sub-phases in the source
(unlike Territory's four `_process_*` methods); this catalog splits it into Computation A
(FR-019/FR-020, per-territory) and Computation B (FR-035, per-territory, collected then
emitted) for clarity, matching the property-law file's own L1-L4 grouping.

### Computation A — Effective-controller resolution + metabolic-impact assignment (FR-019, FR-020)

- **(a)** For every Territory, find the Sovereign with the CLAIMS edge of the highest
  `control_level` (ties broken by lexicographically smallest sovereign ID). That
  Sovereign's `extraction_policy` determines a flat per-tick habitability delta —
  looked up once, never summed across multiple claimants.
- **(b)** `claims = wrapped.query_territory_claims(territory_id)` (line 91) returns rows
  pre-sorted `(-control_level, sovereign_id)` by `BabylonGraph.query_territory_claims`
  (`graph.py:958`); `controller_id, _control, _legal = claims[0]` (line 96) takes the
  first row — the sort, not this line, implements the tiebreak. `policy =
  self._coerce_policy(sov_node.attributes.get("extraction_policy"))` (line 100).
  `impact_by_territory[territory_id] = calculate_metabolic_impact(policy,
  defines=services.defines.balkanization)` (lines 109-111) — a pure 3-way dispatch, no
  arithmetic (see §4). `controller_by_territory[territory_id] = controller_id` (112).
- **(c) Reads:** `TERRITORY` node IDs only (via `query_nodes`, no TERRITORY *attributes*
  read at all — genuinely simpler than Territory's own system). `CLAIMS` edges incident
  to each territory: `control_level` (float) and `legal_status` (str, **read but
  discarded** — bound to `_legal`, never used, line 96). `SOVEREIGN.extraction_policy`
  off the *resolved* controller node (line 100).
- **(d) Writes:** `context.persistent_data["balkanization.metabolic_impact_by_territory"]`
  (dict, line 126), `context.persistent_data["balkanization.effective_controller_by_territory"]`
  (dict, line 127). **Zero graph node/edge attribute writes** — this system never calls
  `graph.update_node`/`update_edge` anywhere.
- **(e) Defines:** `balkanization.metabolic_impact_intensify` (−0.02), `.metabolic_impact_continue`
  (−0.005), `.metabolic_impact_cease` (+0.01) — defines.yaml:318-320, no `ge`/`le` bound
  declared on any of the three (`config/defines/balkanization.py:46-57`) — unbounded real,
  can be positive or negative.
- **(f) Events:** none directly (the persistent-data write is unconditional bookkeeping,
  not event-gated).

**Defect/oddity, transcribed as-is (port-as-is law):** `_coerce_policy` (149-160) returns
`None` — silently, no log, no error — when `extraction_policy` is absent or fails
`ExtractionPolicy(raw)` parsing (`ValueError`/`KeyError` swallowed at line 158-159). The
guard at line 101-102 then `continue`s, dropping the territory from **both** output dicts
with no signal that anything was skipped. `tests/unit/engine/laws/test_law_sovereignty.py`
pins this exact branch as a named caveat (`test_territory_dropped_when_controller_policy_unresolvable`,
lines 324-347) rather than folding it into a property law, precisely because it is a silent
drop, not a loud failure.

### Computation B — Dual-power detection + event emission (FR-035)

- **(a)** If two or more Sovereigns hold an active (`control_level > 0.0`) CLAIMS edge on
  the same Territory, emit one `DUAL_POWER_ACTIVE` event naming all of them and the sum
  of their control levels. Emitted only after the whole territory loop completes, in
  territory-ID sorted order (inherited from the outer loop's own sort, not a second sort).
- **(b)** `active_claimants = [row for row in claims if row[1] > 0.0]` (line 116);
  `if len(active_claimants) >= 2: dual_power_territories.append((territory_id, [row[0]
  for row in active_claimants], sum(row[1] for row in active_claimants)))` (117-124).
  Emission loop (134-147): one `Event(type=EventType.DUAL_POWER_ACTIVE, tick=tick,
  payload={"territory_id": ..., "competing_sovereign_ids": sorted(competing),
  "control_level_sum": control_sum})` per collected row, via `self._publish`.
- **(c) Reads:** the same `claims` list Computation A already fetched for this territory
  (no second `query_territory_claims` call — the two computations share one query per
  territory, a genuinely favorable structural fact: nothing here needs a second graph
  traversal).
- **(d) Writes:** none to graph/persistent_data; the event is the only output.
- **(e) Defines:** none (the `0.0` activity threshold and the `>= 2` cardinality are bare
  literals, not defines-sourced — see §4).
- **(f) Events:** `EventType.DUAL_POWER_ACTIVE` (one distinct EventType; zero to N
  emissions per tick, N = number of contested territories). Payload schema:
  `DualPowerActivePayload` (`models/events/balkanization_payloads.py:93-99`) —
  `territory_id: str`, `competing_sovereign_ids: tuple[str,...]` (`min_length=2`,
  Pydantic-enforced downstream of the System's own `>= 2` guard), `control_level_sum:
  float` (`ge=0.0`). Severity classified `"critical"` (`models/event_severity.py:996`).

**Note on the persistent-data write-back (lines 129-132):** `with
contextlib.suppress(AttributeError): context.persistent_data = persistent` re-assigns
`context.persistent_data` to the *same dict object* it already is, guarded against
`AttributeError` in case `context.persistent_data` was `None` before. Since `persistent =
context.persistent_data` was taken by reference at line 79, and `TickContext.persistent_data`
is declared `dict[str, Any] = Field(default_factory=dict)` (never `None` by construction,
`engine/context.py:49`), this line is dead defensive code on the current `TickContext`
shape — transcribed here as an observation, not a blocker (no BSL equivalent needed; it
does nothing observable).

## 3. TYPE INVENTORY

| Attribute | Node/edge | Python model type | Domain | Category |
|---|---|---|---|---|
| `extraction_policy` | SOVEREIGN (node) | `ExtractionPolicy` (StrEnum, 3 members) | `{intensify, continue, cease}` | **Enum discriminant**, read off a *dynamically resolved* node (not a static anchor — see §6) |
| `control_level` | CLAIMS (edge) | `float \| None` (`relationship.py:134-139`) | `[0.0, 1.0]` (`ge=0.0, le=1.0`) | **Edge-attribute unit-interval real** — not a `Probability`-typed alias, a plain constrained `float` |
| `legal_status` | CLAIMS (edge) | `str \| None` (`relationship.py:140-143`) | Informally `{de_jure, de_facto, disputed, occupied, ceded}` per `ClaimLegalStatus` (`models/enums/balkanization.py:115-136`) — **not type-enforced**: the `Relationship.legal_status` field is a bare `str \| None`, `ClaimLegalStatus` is never referenced by it | Free string, **read but discarded** by this system (dead read, `_legal` unused) |
| `metabolic_impact_intensify/continue/cease` (defines) | — | `float` (no `Field(ge=..., le=...)`) | Unbounded real, signed | Unbounded real coefficient |
| `effective_controller_by_territory[t]` (computed) | — (context.persistent_data value) | `str` (a graph node ID, pattern `^SOV_[A-Z][A-Z0-9_]*$` on the `Sovereign` entity itself, but stored here as a bare string) | Any live SOVEREIGN node ID — **not a closed set**: `CollapseTransitionSystem` can mint new Sovereign nodes at runtime (`collapse_transition.py:166,238`) | **Node-identity value** — no BSL closed type represents "a reference to an arbitrary, possibly-dynamically-created node" (see §6; moot in practice — this output is unconsumed, §5) |
| `control_level_sum` (event payload) | — | `float` (`Field(default=0.0, ge=0.0)`) | `[0, ∞)` — sum of up to N `[0,1]` control levels | Unbounded-above real (bounded below) |

**Enum-on-a-dynamic-referent flag (the genuinely new finding this system surfaces, distinct
from Territory's static enum-storage gap).** Territory's `profile`/`territory_type` enum
reads are always against the system's *own* node (`TERRITORY.profile` read directly off
the node the outer loop is iterating). Here, `extraction_policy` is read off `sov_node =
wrapped.get_node(controller_id)` — a node identified by a **runtime-computed** ID (the
outcome of the control-level max/tiebreak), not the loop's own anchor. Per bsl-language.rst
§3.1/D102 (2273-2284), an enum-declared field can only be read through a *static* `:field`
binding (§2.5) — `field-of`, the accessor for reading a field off a dynamically-held
reference, explicitly refuses enum-declared fields, citing D102. So even once Slice 2
lands the edge read that *finds* the controller, `extraction_policy` cannot be declared
`enum` and read this way — it would need the same int-ordinal content-modeling workaround
Territory used for `territory_type`, but for a genuinely different structural reason (D102's
`field-of` refusal, not the absence of an `enum` `deffield` row — that gap is itself now
resolved, ADR195/196).

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), no libm transcendentals anywhere in
`sovereignty.py` or in `calculate_metabolic_impact` — grep-confirmed zero
`exp`/`log`/`sigmoid`/`math\.`/`pow` in either file. Shapes, in execution order:

1. **Activity threshold comparison:** `row[1] > 0.0` (`sovereignty.py:116`) — one bare
   literal `0.0` compared against a Real-domain field (`control_level`). Same class as
   Territory's flagged bare-literal issue (§1.5's "no bare non-integer literal" rule);
   not a structural blocker — needs a `c`-suffixed literal or the Real-zero-promotion
   idiom already demonstrated in `vitality.bsl:45-64`.
2. **Summation over a filtered set:** `sum(row[1] for row in active_claimants)`
   (`sovereignty.py:122`) — a plain `fold sum`-shaped reduction, 0 to N terms (N = live
   claimants on that territory). This is exactly the shape BSL's landed `fold` head
   serves (once the edge-attribute source it folds over is reachable, §6) — a favorable
   structural match, not a deviation.
3. **Sort-key negation (not in this file, but load-bearing for this system's entire
   controller-resolution algorithm):** `rows.sort(key=lambda row: (-row[1], row[0]))`
   (`topology/graph.py:958`) — negates a `[0,1]`-domain float for descending order,
   secondary sort key is the sovereign-ID string. This is precisely the "`select-max`
   with a language-level tiebreak" pattern the CURRENT BSL surface confirms landed
   (Slice 1) — but only once the field being maxed over (`control_level`) is an
   evaluable expression, which today it is not (edge attribute, §6).
4. **`calculate_metabolic_impact` (`formulas/balkanization.py:66-74`): zero arithmetic.**
   A pure three-way `if policy is X: return defines.Y` dispatch that returns a stored
   coefficient verbatim — no addition, multiplication, or transformation of any kind.
   This is the single simplest "computation" of any system inventoried in this train.
5. **No `int()` casts anywhere** — zero Real→Int demotions. (Contrast Territory's two
   truncating casts.)
6. **No clamps anywhere** — `sovereignty.py` never calls `_write_clamped` or hand-writes
   a `min`/`max` saturation, because it never writes a graph attribute at all (§2). Zero
   clamp-implementation inconsistency risk for this system specifically (though the
   *consumer*, `MetabolismSystem`, does clamp `habitability` via `_write_clamped` when it
   applies this system's output — out of this system's own scope, noted for completeness).

**No exp/log/sigmoid anywhere — zero libm-nondeterminism hazard**, matching Territory and
unlike Metabolism.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.5** (`sovereignty.py:66`), confirmed against the position-sorted
  derivation `_DEFAULT_SYSTEMS = [cls() for cls in sorted(_SYSTEM_CLASSES, key=lambda c:
  c.position)]` (`simulation_engine.py:376-378`). Actual immediate neighbors by position
  (verified by reading every System's `position: ClassVar[float]`, not the registry
  tuple's declaration order): `PolicySystem` (17.47) immediately before,
  `MarketScissorsSystem` (17.8) immediately after.
  **Doc staleness (transcribed as-is, not fixed):** the class docstring
  (`sovereignty.py:10-11`) says "between ConsciousnessSystem at 17 and ContradictionSystem
  at 18" — true only of the two nearest *named-in-the-docstring* systems' positions
  (`ideology.py`'s `ConsciousnessSystem` at 17.0, `contradiction.py`'s `ContradictionSystem`
  at 18.0); it omits that **five** systems now sit between them
  (`FascistFactionSystem`@17.4, `AllegianceSystem`@17.42, `ElectoralSystem`@17.45,
  `PolicySystem`@17.47, then this system, then `MarketScissorsSystem`@17.8) — accretion
  from Program 25 (electoral) and Program 23 (market scissors), landed after this
  docstring was written.
- **Reads from a same-tick prior system: none verified.** `extraction_policy` is set only
  at scenario-seed time or by `CollapseTransitionSystem` (position 20.5 — *after*
  Sovereignty, so any collapse-driven Sovereign creation/policy this tick is invisible
  until next tick; grep-confirmed zero other production write sites for
  `Sovereign.extraction_policy` — the only writers are `apply_balkanization_seed`,
  `electoral_fixture.py`'s inline `Sovereign(...)` construction, and
  `collapse_transition.py:166,238`). `control_level`/`legal_status` are likewise
  seed-time or `CollapseTransitionSystem`-rewired (also downstream/next-tick:
  `collapse_transition.py:173-174,201,270`, all past position 17.5). `TERRITORY` nodes
  are immutable substrate (Constitution I.20) — no system adds/removes them mid-run.
- **A verified architectural duplication, not a data channel:** `PolicySystem` (17.47,
  immediately prior) independently recomputes the *same* "≥2 active claimants"
  structural predicate via `dual_power_live()`
  (`domain/politics/governance_endgame.py:81-96`, its own docstring: "the same structural
  predicate SovereigntySystem @17.5 emits DUAL_POWER_ACTIVE for... read live over claim
  rows rather than replayed from event history"), calling the exact same
  `graph.query_territory_claims()` this system calls (`policy.py:235,488,495`). It cannot
  consume this system's event or `persistent_data` output — it runs first. This means a
  future port that lands both systems will need the Slice-2 edge-read capability twice,
  independently, in two separate `.bscn` rule files (port-as-is: transcribe each frozen
  Python function's own query, do not factor a shared BSL predicate across systems that
  are not shared in the frozen reference).
- **Writes — three channels, one live, one dead, one event:**
  1. `persistent["balkanization.metabolic_impact_by_territory"]` — **the only production
     consumer is `MetabolismSystem`** (position 13.0, `metabolism.py:78-88`:
     `sovereign_impact = persistent.get("balkanization.metabolic_impact_by_territory",
     {})`, applied to `TERRITORY.habitability` via `_write_clamped`). Since 13.0 < 17.5,
     Metabolism reads this dict **before** Sovereignty writes it this tick — it always
     sees the value from Sovereignty's *previous* tick run. This is a genuine, verified
     1-tick production lag, not a hypothetical: `context.persistent_data` is only reset
     to `{}` at the very first `step()` call of a run (`simulation_engine.py:568`); every
     subsequent tick's `TickContext` is seeded from the caller-threaded
     `persistent_context` dict (568, synced back at 577-580), and
     `tools/regression_test.py:1023,1055-1058` threads exactly **one** such dict across
     an entire multi-tick canonical run. **No existing test exercises this real lag**:
     both integration tests that drive Sovereignty and Metabolism together
     (`test_us1_extraction_trajectory.py:97-112`'s `_tick_pipeline` and
     `test_determinism_replay.py:163-181`'s `pipeline` list) call
     `SovereigntySystem().step()` **before** `MetabolismSystem().step()` within one
     hand-built loop iteration — the inverse of the real engine's position order
     (13.0 < 17.5) — making the effect appear same-tick in every test that exists. A
     port conformance fixture that faithfully reproduces the real
     `_DEFAULT_SYSTEMS` ordering will show a trajectory delayed by exactly one tick
     relative to what `test_us1_extraction_trajectory.py` asserts (e.g. "INTENSIFY drops
     habitability ≈0.2 over 10 ticks" assumes same-tick application; the real engine's
     10-tick run would show ~9 ticks' worth of accumulated drop, the 10th tick's write
     landing but not yet read). Flagged for the port dossier's D-record, not as a bug to
     fix (port-as-is law) — and favorably, the graph-scope-state ruling (§3.6, below)
     reproduces this lag *automatically* and correctly if the port stores the value as
     ordinary TERRITORY node state rather than trying to special-case same-tick delivery.
  2. `persistent["balkanization.effective_controller_by_territory"]` — **verified dead**.
     Grep across all of `src/babylon/` for `effective_controller` finds exactly three
     hits: the class docstring, the write site, and nothing else — no System, observer,
     or projection ever reads this key. Exercised only by direct unit/property tests
     (`test_sovereignty_system.py:79-90`, `test_law_sovereignty.py:235-251`). Candidate
     for the port to drop entirely with a D-record citing this zero-consumer
     verification, rather than inventing storage for the node-identity type problem
     §3/§6 name.
  3. `EventType.DUAL_POWER_ACTIVE` — read downstream by `game/chronicle_adapter.py:352`
     (the AI narrative layer, in-bounds per Constitution's "AI narrates, never
     controls"), converted to a typed `DualPowerActivePayload` by
     `engine/event_builders.py:280-286` for the API/projection surface, and classified
     `"critical"` severity by `models/event_severity.py:996`. Not consumed by any other
     Engine System mid-tick (events are bus history, read only after the tick
     completes).
- **Context/service usage with no BSL equivalent:** `context.persistent_data` itself.
  bsl-language.rst §3.6 (the "draft ruling — Phase 1 review, R9 chapter C3") is the
  authoritative answer for this class of gap generally — graph-scope state (including
  this exact system's `persistent_data` writes, cited **by name** in the ruling's own
  text as one of the R9 gap analysis's motivating examples, `bsl-language.rst:2656`) gets
  a home as an ordinary `deffield` on a carrier node type, read/written with ordinary
  `field-of`/`update-node`. For a *per-territory* map like `metabolic_impact_by_territory`
  the natural carrier is the TERRITORY node itself — the ruling explicitly notes
  per-county/per-sovereign registers are "ordinary nodes of ordinary types," not forced
  into a ceiling-1 singleton (`the`). This is a favorable structural match (§6): no
  deviation needed for the *storage class* problem, only for the *value type* problem
  (`effective_controller_by_territory`'s Sovereign-identity values, §3).
- **DORMANCY on canonical scenarios — a verified correction to the estate's own coverage
  ledger.** `tools/regression_scenarios.py:2776-2782` declares: *"SovereigntySystem — no
  SOVEREIGN nodes are seeded; CLAIMS-based effective-controller/metabolic-impact
  resolution and DUAL_POWER_ACTIVE detection never exercise (the unconditional
  empty-dict persistent_data write every tick is bookkeeping, not material logic)."*
  This is **stale**, verified by tracing the full call chain: 5 of the 12 `SCENARIOS`
  dict entries (`mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve` —
  `tools/regression_scenarios.py:78-114`) build on `create_single_county_scenario()`
  (Wayne County, `county_fips="26163"`, `single_county.py:79`) and then call
  `apply_political_terrain(state, ...)` (`electoral_fixture.py:204` in every one of the
  five call sites — `electoral_goldens.py:226,270,334,428,504`), which
  **unconditionally** calls `apply_balkanization_seed(state)`
  (`electoral_fixture.py:204`, before the `include_michigan` branch at 233). That seeds
  `SOV_USA_FED` with `extraction_policy: "intensify"`
  (`data/game/balkanization/seed_sovereigns.json:9`) and a literal CLAIMS edge onto every
  territory carrying FIPS `26163` — Wayne's `T001` — at `control_level: 1.0`
  (`seed_sovereigns.json:19-24`, resolved via `_fips_to_territories`/`by_fips` in
  `balkanization_seed.py:105,124`). `WorldState.to_graph()._add_political_nodes`
  (`world_state.py:785-786`) materializes `SOV_USA_FED` as a real `NodeType.SOVEREIGN`
  graph node with `extraction_policy` in its attribute dump. So on **all five** of these
  canonical scenarios, `SovereigntySystem.step()` resolves a real, non-empty
  `impact_by_territory["T001"] = -0.02` (INTENSIFY) every tick from tick 0 onward, and —
  because `graph_content_hash` (`tools/regression_test.py:924-964`) hashes **every** node
  and edge attribute of the `WorldState→graph` projection, including the SOVEREIGN node
  and CLAIMS edge themselves, plus (one tick later) `MetabolismSystem`'s resulting
  `TERRITORY.habitability` write — this is **not** unobserved bookkeeping; it is live,
  byte-gated material logic. The `syriza` scenario additionally adds a second competing
  claim (`SOV_MI_STATE → T001, control_level=0.4`, `electoral_goldens.py:290-297`, "the
  dual-power organ on Wayne"), giving **two** active claimants on T001 and exercising the
  FR-035 `DUAL_POWER_ACTIVE` emission path specifically — the one part of this system the
  gap declaration's "never exercise" claim gets right for the *other four* scenarios
  (only `syriza` has a second live claimant; `weimar`/`debs` even skip `SOV_MI_STATE`
  entirely via `include_michigan=False`, `electoral_goldens.py:334,428`).
  **Caveat, stated honestly:** `graph_content_hash` explicitly *excludes* the event log
  (`tools/regression_test.py:939-943`, "Graph metadata (`g.graph`: economy, event log,
  opposition states) is also excluded") and no dense-CSV/checkpoint column tracks events
  either (grep-confirmed zero `event_log` references in `regression_test.py`) — so while
  the *metabolic-impact/effective-controller* half of this system is genuinely
  byte-gate-covered on 5 scenarios, the `DUAL_POWER_ACTIVE` *event emission itself* is
  covered by **no** byte-identical mechanism in the current harness, even on `syriza`
  where it fires every tick. This verification is by full static call-chain tracing
  (every file/line cited above was read directly); the read-only mandate for this
  inventory precludes actually running `qa:regression` to visually confirm the resulting
  `habitability` drift in the golden CSV — a live run would be the natural next
  verification step, not performed here.

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| Territory-node enumeration, sorted by ID (`sovereignty.py:87-89`) | **PORTABLE NOW** | Plain `nodes` query over `NodeType.TERRITORY`, landed Slice 1 (`SERVED_QUERY_HEADS`, `evaluator.rs:527`). |
| CLAIMS-edge claims lookup + effective-controller resolution (`control_level` max, sovereign-ID tiebreak; `sovereignty.py:91-96`, `graph.py:958`) | **BLOCKED — Slice 2 (edge-attribute reads / `EdgeRef`)** | `field-of` Slice 1 "serves `NodeRef` referents only — an `EdgeRef` referent is unreachable today (no expression form produces one yet; slice 2 mints `EdgeKey`)" (`evaluator.rs:1190-1196`). `edges`/`edge-between` are listed in `UNSERVED_EXPRESSION_HEADS` tagged `"slice 2"` (`evaluator.rs:503-511`). `control_level`/`legal_status` exist **only** as CLAIMS-edge attributes — there is no node-side substitute. This is the root blocker for essentially this entire system. |
| `extraction_policy` read off the dynamically-resolved controller (`sovereignty.py:100`) | **BLOCKED (compounding, distinct reason) — D102 `field-of`-enum refusal** | Even granting Slice 2 lands the edge read that *finds* `controller_id`, reading its `extraction_policy` requires `field-of` against a runtime-computed `NodeRef` (not a static `:field` anchor) — and `field-of` explicitly refuses enum-declared fields, citing D102 (`bsl-language.rst:2273-2284,5679-5681`). Workaround: content-model `extraction_policy` as an `int`-ordinal `deffield` instead of `enum` (same D-record class as Territory's `territory_type` workaround, but for a genuinely different structural reason — D102's accessor refusal, not the absence of an `enum` `deffield` row, which is itself now resolved by ADR195/196). |
| `calculate_metabolic_impact` 3-way dispatch (`formulas/balkanization.py:66-74`) | **PORTABLE NOW, once the policy value is in hand** | Zero arithmetic — a trivial `when`/`if` chain returning one of three `defconst`s. Exact precedent in every landed pack. |
| `metabolic_impact_by_territory` persistent-data write (`sovereignty.py:126`) | **PORTABLE WITH D-RECORD** | Store as an ordinary `deffield real` on TERRITORY (`sovereign-metabolic-impact` or similar), written via `update-node`, read by MetabolismSystem's own rule via `:field` — the §3.6 graph-scope-state ruling's exact intended use case, cited by name in the ruling's own motivating text (`bsl-language.rst:2656`). This transcription reproduces the real 1-tick lag (§5) as an emergent, correct property of position ordering — no special-casing needed, a favorable structural match. |
| `effective_controller_by_territory` persistent-data write (`sovereignty.py:127`) | **BLOCKED in principle, MOOT in practice — recommend drop with D-record** | No BSL closed field type stores "a reference to an arbitrary, possibly-dynamically-created node" (§3). But §5 verifies this output has **zero** production consumers — the honest port drops it, citing the zero-consumer verification as the D-record's justification, rather than inventing storage for dead output. |
| Dual-power active-claimant filter + control-level sum (`sovereignty.py:116-122`) | **BLOCKED — Slice 2 (same edge-attribute dependency)** | Needs the same CLAIMS-edge `control_level` reads as controller resolution, plus a `fold sum` over the filtered set — the `fold` itself is landed Slice 1 machinery, gated only on having a Slice-2 edge source to fold over. |
| `DUAL_POWER_ACTIVE` event emission (`sovereignty.py:134-147`) | **WS1 ledger item (separate axis from the data-source blocker)** | Per the current BSL surface: "TickReport carries no event log — every EventType emission is a WS1 (#502) ledger row, unpinnable by goldens today." This applies once Slice 2 makes the *detection* itself expressible; the *emission* still lands on the standing WS1 ledger like every other system's events. |

**Summary verdict for this system:** every real computation is gated on Slice 2. There is
no partial-pack subset analogous to Territory's "Phase 1 heat + Phase 4 camp decay" —
Territory's phases were independently gate-able because some read only node state;
SovereigntySystem's *entire* algorithm is edge-attribute-shaped from the first line of
real logic onward. This makes the system a clean single-dependency case: land Slice 2,
and (given the D-record workarounds above, both already precedented by other systems'
D-records) the whole system becomes portable in one pass, rather than needing its own
staged unblock.

## 7. TEST/BASELINE SURFACE

**Primary conformance-oracle candidates (SovereigntySystem's own `step()` behavior):**

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/balkanization/test_sovereignty_system.py` | 186 | **Primary direct-unit oracle.** `test_sovereignty_writes_metabolic_impact_per_territory`, `..._writes_effective_controller_per_territory`, `..._dual_power_tiebreak_only_highest_wins` (FR-020), `..._emits_dual_power_active_event` (FR-035), `..._no_dual_power_for_single_claimant`, `..._skips_territories_with_no_claims`, and a defines-passthrough call-contract pin (`..._calls_calculate_metabolic_impact_with_run_defines`, a regression test for a Task #42 defines-plumbing bug). |
| `tests/unit/engine/laws/test_law_sovereignty.py` | 346 | **Primary property-based oracle.** Four Hypothesis laws (L1 claims-gated output, L2 effective-controller max+tiebreak, L3 no-double-counting, L4 dual-power emission count-and-payload) plus one deterministic caveat regression (unresolvable-policy drop). Each law's docstring cites its own evidencing line range in `sovereignty.py` — this file is effectively a second, independently-derived specification of the system, ideal raw material for a BSL conformance fixture set. |
| `tests/integration/balkanization/test_us1_extraction_trajectory.py` | 223 | Drives SovereigntySystem + MetabolismSystem together over 5-100 ticks (US1 acceptance scenarios 1-4: INTENSIFY/CONTINUE/CEASE trajectories, multi-territory, unclaimed-territory isolation, habitability clamping). **Caveat (§5): calls the two systems in the inverse of the real engine's position order**, so it validates the formula composition, not the real cross-tick timing. Still directly useful for pinning the *magnitude* trajectory a same-tick BSL transcription would need to deliberately NOT match without the D-record noted in §5. |
| `tests/integration/balkanization/test_determinism_replay.py` | 236 | Byte-identical replay (same seed twice) across `FactionInfluenceSystem`+`SovereigntySystem`+`MetabolismSystem`+`CollapseTransitionSystem`, 10 ticks. Same reordering caveat as above (`pipeline = [FactionInfluenceSystem(), SovereigntySystem(), MetabolismSystem(), CollapseTransitionSystem()]`, Metabolism placed *after* Sovereignty). Good determinism-by-seed evidence; not a real-ordering conformance oracle. |
| `tests/unit/engine/test_system_order.py` | (relevant: 89, 187, 258) | Pins tick position 17.5 in the registry-order gate. Schema-level, not behavioral. |

**Adjacent/narrative estate (not primary SovereigntySystem oracles — the broader spec-070
balkanization surface this system is one piece of):**

| Test file | Lines |
|---|---|
| `tests/unit/balkanization/test_metabolic_impact_formula.py` | 58 |
| `tests/unit/balkanization/test_sovereign_entity.py` | 125 |
| `tests/unit/balkanization/test_balkanization_defines.py` | 119 |
| `tests/unit/balkanization/test_enums.py` | 139 |
| `tests/unit/balkanization/test_event_payloads.py` | 180 |
| `tests/unit/balkanization/test_seed_loaders.py` | 174 |
| `tests/unit/balkanization/test_seed_influences.py` | 348 |
| `tests/unit/balkanization/test_graph_protocol_extensions.py` | 223 |
| `tests/unit/balkanization/test_faction_influence_system.py` | 219 |
| `tests/unit/balkanization/test_collapse_transition_system.py` | 226 |
| `tests/unit/balkanization/test_observability_projections.py` | 167 |
| `tests/unit/balkanization/test_fracture_operation_o1.py` | 121 |
| `tests/unit/balkanization/test_faction_node_type_query.py` | 117 |
| `tests/integration/balkanization/test_seed_coverage_invariant.py` | 126 |
| `tests/integration/balkanization/test_audit_round_trip.py` | 158 |
| `tests/integration/balkanization/test_us4_secession_fracture.py` | 263 |
| `tests/integration/balkanization/test_postgres_persistence.py` | 63 |
| `tests/unit/engine/scenarios/test_balkanization_seed.py` | (not read; scenario-builder test, schema-level) |
| `tests/unit/engine/scenarios/test_electoral_fixture.py` | (not read; scenario-builder test, schema-level) |
| `tests/unit/engine/scenarios/test_electoral_goldens_factories.py` | (not read; scenario-builder test, schema-level) |
| `tests/unit/engine/systems/test_electoral_goldens.py` | (not read; golden-fixture assertions for the electoral estate broadly) |
| `tests/unit/engine/systems/test_policy.py` (relevant: 630-660) | (not fully read; contains the `dual_power_live` cross-reference cited in §5) |
| `tests/unit/projection/test_sovereign.py`, `tests/unit/projection/vault/test_render_sovereign.py`, `tests/unit/projection/vault/test_materializer_sovereign.py` | (not read; `observe()`-page rendering, not engine math) |
| `tests/unit/models/test_event_severity.py` (relevant: 420, 629, 996) | (severity-tier classification for `DUAL_POWER_ACTIVE`, not this system's math) |

**qa:regression byte-gate coverage.** `graph_content_hash` (`tools/regression_test.py:924-964`)
hashes every node and edge attribute of the `WorldState→graph` projection. Per §5's
verified correction, this **does** cover SovereigntySystem's material effect on 5 of the
12 canonical scenarios (`mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`) — both
directly (the seeded `SOV_USA_FED` node's `extraction_policy` and its CLAIMS edge's
`control_level`/`legal_status` are graph attributes, hashed) and indirectly (the resulting
`TERRITORY.habitability` write one tick later, via `MetabolismSystem`). It does **not**
cover the `DUAL_POWER_ACTIVE` event itself on any scenario (`syriza`'s the only one that
fires it) — the event log is explicitly excluded from the hash
(`tools/regression_test.py:939-943`), and no dense-CSV/checkpoint column tracks events
either. A port conformance fixture set should therefore not lean on `qa:regression` for
event-emission coverage — only `tests/unit/balkanization/test_sovereignty_system.py` and
`tests/unit/engine/laws/test_law_sovereignty.py` currently pin that behavior, and neither
is a byte-identical baseline gate.

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`), read-only, with fresh anchors.
The inventory's §5 dormancy correction is upheld in full and is this report's best work.
Its **verdict is corrected on the named unblock**: the blocker is one level below the
query-evaluation slice it names, which changes what train clears this system.

1. **CORRECTION — the blocker is a SUBSTRATE storage gap, not query-evaluation Slice 2;
   landing Slice 2 would leave this system exactly where it is.** The executive summary and
   §6's root-blocker row both name "Slice 2 (edge-attribute reads / `EdgeRef`)" and the §6
   summary concludes "land Slice 2, and … the whole system becomes portable in one pass."
   Slice 2 mints edge *references*; it does not mint edge *attributes*. The full
   `GraphSubstrate` trait surface is `rust/crates/babylon-graph/src/substrate.rs:80-248` and
   it contains **no edge-attribute accessor of any kind**: the one edge reader is
   `fn edges(&self, edge_type: &str) -> Vec<(NodeId, NodeId)>` (`substrate.rs:166`), which
   returns bare id pairs, and `fn add_edge(…, strength: f64, …)` (`substrate.rs:111-116`)
   carries one `f64` and nothing else — there is no `edge_attribute`, no `edge_strength`,
   no per-edge field map. `rust/crates/babylon-bsl/src/structural_verbs.rs:387-398` states
   the identical fact from the write side, verbatim: *"has no substrate storage:
   GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no attributes at
   all. Widening that state widens the canonical state_hash field set, which is a declared
   Phase-2/substrate decision (Constitution III.7), never a silently-dropped write"*.
   `control_level`/`legal_status` (`src/babylon/models/entities/relationship.py:134-143`)
   exist **only** as CLAIMS-edge attributes, as §3 correctly records — so the read this
   entire system keys off has no substrate to read from, Slice 2 or no Slice 2. The unblock
   is a hash-relevant, III.7-gated substrate-widening decision (the same one
   `ContradictionSystem`'s sibling inventory named for its Phase-1 `tension` write, and the
   same one `FieldDerivativeSystem`'s named for `field_gradients`); Slice 2 is necessary and
   insufficient. **Consequence for sequencing:** §6's "clean single-dependency case" claim
   does not hold — this system is not clearable by the query-evaluation train.

2. **CORRECTION — §6 row 1's "Territory-node enumeration, sorted by ID | PORTABLE NOW"
   does not carry the sort.** `sovereignty.py:87-89` enumerates
   `sorted(node.id for node in query_nodes(TERRITORY))` — **string** id order — and the
   inventory's own §2 (b) and the class docstring (`sovereignty.py:20`) make sorted-territory
   order the emission contract for FR-035. BSL's `nodes` query is specified to iterate
   **ascending `NodeId`** (`substrate.rs:153-162`: "Iteration order is part of the CONTRACT
   … ascending id order … never graph-internal storage order"), and `NodeId` is a
   load-order-minted integer (`scenario.rs:403-413`, `load_node` → `node_types.entry(minted)`),
   not the content symbol. The two coincide only if the `.bscn` declares territories in
   sorted-string order. That is a content-modeling obligation the row should state, not a
   free "PORTABLE NOW". Low-stakes in practice — the per-territory computation is
   order-independent and the only order-sensitive output is the `DUAL_POWER_ACTIVE` emission
   sequence, itself a WS1 (#502) ledger row unpinnable by goldens today — but it is exactly
   the class of silent divergence a PORTABLE-NOW verdict is supposed to have excluded.

3. **CORRECTION — the inventory carries NO RESERVED-LINE section, and this system has one.**
   Every sibling inventory in this train performs the check explicitly (MarketScissors:
   "none found"; FieldDerivative: "No RESERVED-LINE surface"; Contradiction: the `national`
   opposition flagged). This one omits the check entirely, and the surface it omits is real:
   `ExtractionPolicy` is the **mechanical output of the National Question line**.
   `src/babylon/formulas/balkanization.py:24-28` — `_STANCE_TO_POLICY = {UPHOLD: INTENSIFY,
   IGNORE: CONTINUE, ABOLISH: CEASE}` — maps `ColonialStance` (the settler-colonial axis
   ruled by ADR171, owner ruling 2026-07-15) one-to-one onto the very enum this system reads
   at `sovereignty.py:100`, and `catalog.py:958-960`'s own comment names it as "the same
   division as ``formulas.balkanization._STANCE_TO_POLICY``" when flagging its sibling
   `_STANCE_CHAUVINISM_SCORE` as RESERVED. The three coefficients this system's only formula
   returns verbatim — `metabolic_impact_intensify` −0.02 / `_continue` −0.005 / `_cease`
   **+0.01** (`defines.yaml:318-320`) — are the numeric encoding of a *theoretical* claim
   (settler extraction degrades the land; abolition heals it), not a calibration. §6's
   recommended int-ordinal `deffield` workaround for `extraction_policy` therefore fixes a
   **declaration order that is hash-bearing** (ADR195/196) on a reserved axis: it is a
   Director escalation, not a free port-time D-record.

4. **CONFIRMATION — the stale coverage-gap correction is verified end-to-end, and it is
   right.** `tools/regression_scenarios.py:2776-2782`'s "no SOVEREIGN nodes are seeded"
   declaration is stale. Traced independently: `SCENARIOS` holds exactly twelve rows
   (`imperial_circuit, two_node, starvation, glut, fascist_bifurcation, single_county,
   mitterrand, syriza, weimar, debs, bernie_valve, org_probe`); the five electoral goldens
   call `apply_political_terrain` at `electoral_goldens.py:226,270,334,428,504`;
   `electoral_fixture.py:204` calls `apply_balkanization_seed(state)`
   **unconditionally**, before the `include_michigan` branch at :233;
   `balkanization_seed.py:120-135` mints a real `EdgeType.CLAIMS` `Relationship` with
   `control_level=float(claim["control_level"])`; and
   `data/game/balkanization/seed_sovereigns.json:5-24` seeds `SOV_USA_FED` with
   `extraction_policy: "intensify"` and a `control_level: 1.0` claim on FIPS `26163`.
   `world_state.py:783-788`'s `_add_political_nodes` materialises it as a real
   `NodeType.SOVEREIGN` node with its full `model_dump()`. The correction stands.

5. **CONFIRMATION — the one-tick Metabolism lag is real, and the §6 disposition of it is
   right.** `metabolism.py:50` declares `position = 13.0`; `metabolism.py:80-82` reads
   `persistent.get("balkanization.metabolic_impact_by_territory", {})` and applies it via
   `_write_clamped` at :86 — 13.0 < 17.5, and `_DEFAULT_SYSTEMS` is derived by sorting
   `_SYSTEM_CLASSES` on `position` (`simulation_engine.py:376-378`), so the read always
   precedes the write. §6's disposition — store it as ordinary TERRITORY node state, which
   reproduces the lag as an emergent property of position ordering — is correct and is the
   §3.6 ruling's intended use (`bsl-language.rst:2650-2689`).

6. **CONFIRMATION — `effective_controller_by_territory` is dead output.** `rg` over
   `src/`, `web/`, `tools/` for `effective_controller` returns exactly three hits, all in
   `sovereignty.py` itself (:4 docstring, :56 docstring, :127 the write). Zero consumers.
   §6's "drop with a D-record citing the zero-consumer verification" is the honest call.

7. **CONFIRMATION, with the mechanism's location sharpened — D102's `field-of` refusal is a
   LOAD-time typecheck gate, not an evaluation-time one.** `typecheck.rs:246-290` raises the
   structural error *"field-of {qname}: an :enum-type-declared field is read via a :field
   binding only (§2.5) — field-of is not extended to enum-declared fields (§2.13, D102)"*,
   wired at `rule_pipeline.rs:293-301`, with `code == None` pinned at `typecheck.rs:804`.
   The evaluator itself does no enum check — `evaluator.rs:1274-1292`'s `field_of_node`
   returns `Value::Real` unconditionally — so the static gate is the *only* thing holding the
   line. Practical consequence for the pack: a rule carrying this shape **never runs at all**
   (red load), so this is a load-gate row, not a runtime-behaviour row. §3/§6's substantive
   reading and its recommended int-ordinal workaround are otherwise correct.

**FINAL VERDICT: BLOCKED — on a `GraphSubstrate` edge-attribute STORAGE decision
(hash-relevant, Constitution III.7), not on query-evaluation Slice 2.** Every computation in
`step()` keys off CLAIMS-edge `control_level`, and the substrate exposes no edge-attribute
reader at all (`substrate.rs:80-248`); Slice 2 mints references over storage that does not
exist. This system is therefore **not** clearable by the query-evaluation train, and the
§6 summary's "clean single-dependency case … land Slice 2 and the whole system becomes
portable in one pass" is withdrawn. Once that substrate decision lands, the two remaining
obstacles are as the inventory describes — `persistent_data` storage via the §3.6 carrier
ruling (portable today on landed Slice 1, no `the` required) and the `extraction_policy`
read — except that the second is **RESERVED-LINE (National Question, ADR171)** and needs
Director disposition rather than a port-time D-record.

**INADEQUATE-COVERAGE — a re-read must add:**
(a) a **RESERVED-LINE section** (every sibling inventory has one), covering
`formulas/balkanization.py:24-28`'s `_STANCE_TO_POLICY`, the `ColonialStance →
ExtractionPolicy` derivation at `:77-102`, and the three signed
`metabolic_impact_*` coefficients as theory-bearing rather than calibration;
(b) the substrate-level edge-storage adjudication above, replacing "Slice 2" as the named
unblock everywhere it appears (executive summary, §6 rows 2 and 7, §6 summary);
(c) the `nodes`-iteration-order caveat on §6 row 1;
(d) `tests/unit/engine/systems/test_policy.py:630-660`, cited in §5 as the ground of the
`dual_power_live` duplication finding but listed "(not fully read)" — the duplication claim
is the basis for "the Slice-2 capability will be needed twice, independently", a real
sequencing consequence that currently rests on an unread file.
