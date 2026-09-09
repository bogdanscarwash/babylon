# AllegianceSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `AllegianceSystem` (`src/babylon/engine/systems/allegiance.py`, 516 lines,
tick position 17.42, the electoral valve — P25 U8/ADR134) is categorically harder to port than the
self-contained Territory/Solidarity systems already inventoried: it is a nested nodes×parties
computation whose per-class writes target an **open, dynamically-keyed `dict[str, float]` node
field** (`allegiance`, one mass per `PoliticalFaction` org id) and whose per-tick inputs include
**three cross-system graph-scope dict registers** (`policy_delivery`, `electoral_disillusion`,
`popular_front`) written by PolicySystem/ElectoralSystem — none of which fit BSL's closed
`deffield` type vocabulary or its §3.8 "no sequence or map type" re-modelling recipes. Its own pure
kernel (`formulas/politics.py`) additionally calls `math.sqrt` (not a declared BSL intrinsic — the
declared set is exactly `exp`/`log`/`floor`) inside the cosine-similarity `interest_fit` every
per-(class,party) pair needs, and its hope field re-evaluates the **frozen, going-forward-retired**
`math.exp` acquiescence sigmoid (ADR172/173: "no imposed functional forms") twice per pair. Unlike
Territory, this system is **not dormant** on canonical `qa:regression` — all five P25 U13 electoral
goldens (`mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`) seed `PoliticalFaction` orgs and
exercise it live. **Verdict: BLOCKED — every one of the system's three motions is blocked, directly
or by inheritance, on the map/attributed-membership storage gap (Slice 4, ADR189-designed but
unbuilt) and/or the undeclared `sqrt` intrinsic; the sigmoid reuse is additionally a PORT-QUESTION
under ADR172/173, not a routine D-record.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/allegiance.py` | 516 | **The target.** `AllegianceSystem`, one `step()` orchestrating three per-tick motions (drift, hope/valve, political-form producer) plus five terrain-reader helpers. Read completely, line by line. |
| `src/babylon/formulas/politics.py` | 278 | The pure politics kernel `AllegianceSystem` calls directly (plain function imports, no formula-registry indirection, unlike Solidarity): `valve_multiplier`, `hope_field`, `counterfactual_hope_gain`, `platform_vector`, `allegiance_drift`, `apply_allegiance_drift`, `interest_fit` (allegiance.py:55-63). `turnout_share`/`competitiveness`/`sw_deliverable`/`delivery_ratio`/`delivery_gap` live in the same file but are **not on AllegianceSystem's call path** (ElectoralSystem/PolicySystem consume those — confirmed by grep, only the seven imported names are used here). |
| `src/babylon/formulas/survival_calculus.py` | 110 | `calculate_acquiescence_probability` (lines 21-43) — the frozen P(S\|A) logistic sigmoid `counterfactual_hope_gain` calls twice per (class,party) pair (politics.py:68-71). `math.log` at line 90 (a different function, `calculate_required_wealth` or similar) is **not** reached by AllegianceSystem's path — only the one function is imported (politics.py:19). |
| `src/babylon/engine/systems/policy.py` | 782 | `POLICY_DELIVERY_ATTR = "policy_delivery"` (line 107) — the graph-scope register AllegianceSystem reads (allegiance.py:54,140). Write site: policy.py:172-177 (read-modify), 217-218 (write) — `dict[class_id -> dict[str, Any]]`, PolicySystem @17.47, one tick AFTER AllegianceSystem runs (the I-ORD one-tick-lag grain the docstring names). |
| `src/babylon/engine/systems/electoral.py` | 1269 | `ELECTORAL_DISILLUSION_ATTR = "electoral_disillusion"` (line 109) and `POPULAR_FRONT_ATTR = "popular_front"` (line 117) — the other two graph-scope registers AllegianceSystem reads (by raw string, `allegiance.py:219,242`, to avoid a forward import). Write sites: electoral.py:1209-1250 (disillusion windows, `dict[class_id -> {opened_tick,window_ticks,bridges_present}]`); electoral.py:318-383 (`_popular_front_conjuncture`, `dict{active,since_tick,arms:dict[party_id->str],suppression}`). ElectoralSystem runs @17.45, AFTER AllegianceSystem — same one-tick-lag grain. |
| `src/babylon/engine/systems/contradiction.py` | 1127 | Line 471: `political_labor_share=graph.get_graph_attr("political_labor_share", None)` — the ONE downstream reader of AllegianceSystem's own graph-scope write (`GraphInputs` @18, ContradictionSystem). |
| `src/babylon/config/defines/politics.py` | 574 (whole module; `PoliticsDefines` model) | Coefficient source. Only 10 of the model's ~30 fields are read by this system (§2e) — `base_turnout`, `capital_tolerance`, `policy_agenda_rate`, `debt_finance_share`, `bond_discipline_threshold`, `judicial_tolerance_scale`, `preemption_envelope`, `recount_margin`, `strike_equalization_rate`, `policy_default_magnitude`, `legitimation_refresh_weight`, `betrayal_threshold`, `governance_capture_threshold`, `periphery_phi_share_floor`, `periphery_ceiling_factor`, `office_capture_rate`, `split_asset_retention`, `sect_isolation_rate`, `boycott_conversion`, `popular_front_trigger`, `popular_front_cooptation_rate`, `host_discipline_clamp_share`, `suppression_cost_weight`, `host_threat_threshold`, `legitimacy_backfire_threshold`, `solidarity_liquidation_floor` are consumed by PolicySystem/ElectoralSystem, not this system. |
| `src/babylon/config/defines/survival.py` | 323 (`SurvivalDefines` model) | `steepness_k` (lines 18-22) — the one field this system reads (via `services.defines.survival.steepness_k`, allegiance.py:109), for the reused acquiescence sigmoid. |
| `src/babylon/data/defines.yaml` | `politics:` block, lines 1082-1122; `survival:` block, line 164 | Player-editable coefficient values. |
| `src/babylon/models/entities/social_class.py` | 522 (whole module); `allegiance` field 323-333; `organization` 355-358; `fascist_alignment` 458-462; `wealth` 307-310; `subsistence_threshold` 351-354; `active` 380-384; `IdeologicalProfile` class 61-107 | `SocialClass` node model — every field this system reads/writes on `SOCIAL_CLASS` nodes. **`hope` is NOT declared here** — see §3. |
| `src/babylon/models/entities/organization.py` | 436; `PoliticalFaction` 384-406 (`org_type` 394, `ideology: str` 395-398) | `PoliticalFaction` node model — free-text `ideology` field with no enum/pattern constraint (`min_length=1` only). |
| `src/babylon/models/entities/relationship.py` | 186; `value_flow` field 92-... | `Relationship` edge model — `Currency`-typed `value_flow`, the field `_funding_shares` reads off `TRANSACTIONAL` edges. |
| `src/babylon/models/enums/social.py` | 211; `OrgType.POLITICAL_FACTION = "political_faction"` line 103 | The org-type discriminant `_political_factions` filters on. |
| `src/babylon/models/enums/topology.py` | 253; `EdgeType.MEMBERSHIP` line 109, `EdgeType.TRANSACTIONAL` line 115 | Edge-type discriminants `_membership_map`/`_funding_shares` query on. Docstring note: `TRANSACTIONAL` is documented "Organization → Community (service-for-support exchange)" (Feature 032/OODA) but AllegianceSystem repurposes it generically as donor→party funding inflow — the same edge type, a different semantic family; both usages coexist on dev (electoral_fixture.py seeds donor→party TRANSACTIONAL edges explicitly for this purpose). |
| `src/babylon/models/enums/events.py` | 234; `EventType.HOPE_SPIKE` line 179 | The one `EventType` this system emits (allegiance.py:508). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._wrap_graph` (99-117, called allegiance.py:100). `_write_clamped` (162-...) exists but **is never called** by this system — see §4. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_nodes`(258), `.query_edges`(278), `.get_graph_attr`(350), `.set_graph_attr`(365), `.update_node`(88) — every graph verb this system calls. It never calls `update_edge`/`get_node` directly. |
| `src/babylon/kernel/event_bus.py` | — | `Event` dataclass, `EventBus.publish` — allegiance.py:506-516's one emission site. |
| `src/babylon/kernel/services.py` | 88 | `ServicesProtocol` — `defines: Any`, `event_bus: EventBus`. |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType` — `context.tick`, `context.persistent_data` (dict). |
| `src/babylon/kernel/tick_partition.py` | — | `TickPartition.CONSEQUENCE` — allegiance.py:86. |
| `src/babylon/engine/simulation_engine.py` | 611; `_SYSTEM_CLASSES` 328-363 | Confirms tick position: `ConsciousnessSystem@17.0 → FascistFactionSystem@17.4 → AllegianceSystem@17.42 → ElectoralSystem@17.45 → PolicySystem@17.47 → SovereigntySystem@17.5 → ... → ContradictionSystem@18.0`. |
| `src/babylon/engine/scenarios/electoral_fixture.py` | 278 | `apply_political_terrain` — the shared party-terrain builder (4 `PoliticalFaction` orgs, donor org, `MEMBERSHIP`/`TRANSACTIONAL` edges) all five electoral goldens layer onto their material substrate. |
| `src/babylon/engine/scenarios/electoral_goldens.py` | 543 | The five golden-scenario factories (`create_mitterrand_scenario` etc.) — §5's dormancy evidence. |
| `src/babylon/models/world_state.py` | 1161; line 90 | `hope` listed in `SOCIAL_CLASS_COMPUTED_FIELDS`/exclusion set — confirms `hope` is graph-only, dropped on `from_graph()` reconstruction. |
| `src/babylon/sentinels/vocabulary/registry.py` | 754; line 206 | `EXTRA_STAMPABLE_ATTRIBUTES` cites `"hope"  # engine/systems/allegiance.py (P25 U8 — H(c), per-tick)` — the sanctioned exemption for the vocabulary sentinel. |
| `tools/regression_scenarios.py` | 2925; AllegianceSystem evidence rows at 2289, 2349, 2402, 2467, 2540 | Confirms the system is **live** (not dormant) on all five electoral goldens — §5. |

**Not exercised by allegiance.py at all:** no `src/babylon/domain/*` module. `formulas/politics.py`
is the only formula import (no formula-registry indirection — unlike Solidarity's
`services.formulas.get(...)`, AllegianceSystem imports the pure functions directly at module level,
allegiance.py:55-63).

**Reference BSL/spec text read for the storage-shape and intrinsic questions** (all read in full
for the cited ranges): `docs/reference/bsl-language.rst` §3.1 "Types" (lines ~2293-2380, the closed
7-row `deffield` vocabulary table, `Str`'s "No operations" row, `enum`'s D101 landing); §3.6
"Closed vocabulary" (lines 2640-2688, the graph-scope-as-carrier-node draft ruling); §3.8
"Deliberate absences" item 2 (lines 2860-2876, "No sequence or map type", Q13) and item 4 (lines
2889-2896, "No string payloads on `emit`"); the `<payload-item>`/`emit`/`update-membership` grammar
(lines ~1335-1351); `docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md` lines 74,85,148,
233,828 (Slice 4 = "HASH-TOUCHING (escalates)", explicitly lists `update-membership`/
`membership-field-of` out of Slice-1 scope); `rust/crates/babylon-bsl/src/declarations.rs:110`
(`DECLARABLE_INTRINSICS: [&str; 3] = ["exp","log","floor"]` — confirms `sqrt` is absent);
`rust/crates/babylon-bsl/src/evaluator.rs:2525-2551` (the one documented nested-`fold` test —
`(fold sum (nodes ...) (fold sum (nodes ...) (field-of it ...)))` is "load-legal", refused only in
the narrow empty-query/unclassifiable-identity edge case, D-row Q12 — not a general nested-fold
refusal).

## 2. COMPUTATION CATALOG (execution order, `allegiance.py:94-205`, `step()` body)

### Motion 0 — Guards + terrain precomputation (`allegiance.py:100-160`)
- **(a)** TRAP 3: if no `PoliticalFaction` org exists, the whole system is a no-op (zero reads-into-writes). Otherwise, precompute once per tick, over ALL parties/classes, the quantities the per-class loop reuses: each party's class-base reach, funding share, and derived platform; each party's viability; each class's material-interest vector; the previous tick's hope-by-class; and the three cross-system registers.
- **(b)** `parties = self._political_factions(wrapped)` (101, defined 250-259: `query_nodes(node_type=ORGANIZATION)` filtered `attrs.get("org_type") == OrgType.POLITICAL_FACTION.value`, sorted by id) → `if not parties: return` (102-106) → `classes = sorted(query_nodes(SOCIAL_CLASS) filtered by attrs.get("active", True), key=id)` (110-117) → `if not classes: return` (118-119) → `membership = self._membership_map(wrapped, {p.id for p in parties})` (121, defined 261-267: one `query_edges(edge_type=MEMBERSHIP)` pass building `party_id -> set(class_id)`) → `funding_share = self._funding_shares(wrapped, [p.id for p in parties])` (122, defined 269-278: one `query_edges(edge_type=TRANSACTIONAL)` pass summing `attrs.get("value_flow", 0.0)` per target, then normalizing by the grand total) → `interest = {node.id: self._interest_vector(node) for node in classes}` (123, defined 280-291: `(wealth - subsistence_threshold, ideology.class_consciousness)`) → `platforms = self._platforms(...)` (124, defined 293-329) → `viability = self._viability(...)` (125, defined 331-357) → `committed = self._committed_front_orgs(wrapped)` (133, defined 232-248: reads `popular_front` register, empty set if register absent/inactive/malformed) → `delivery = {...}` built from `wrapped.get_graph_attr(POLICY_DELIVERY_ATTR, None)` (140-145) → `windows = self._active_windows(wrapped, context.tick)` (154, defined 211-230: reads `electoral_disillusion` register, keeping only rows with `opened_tick + window_ticks > tick`) → `prev_hope = dict(context.persistent_data.get(_HOPE_KEY, {}))` (156-157).
- **(c) Reads:** `NodeType.ORGANIZATION` nodes' `org_type`/`id`; `NodeType.SOCIAL_CLASS` nodes' `active`/`id`/`wealth`/`subsistence_threshold`/`ideology.class_consciousness`; `EdgeType.MEMBERSHIP` edges (source/target ids only); `EdgeType.TRANSACTIONAL` edges' `value_flow`; graph-scope `popular_front`, `policy_delivery`, `electoral_disillusion`; `context.persistent_data["politics.hope_by_class"]` (prior tick, this system's own key).
- **(d) Writes:** none in this motion (all local Python collections).
- **(e) Defines:** `politics.donor_platform_weight` (0.35, `>= 0.0`, **unbounded above** — defines.yaml:1117, politics.py:391-399) — consumed inside `_platforms` (allegiance.py:327).
- **(f) Events:** none.

### Motion 1 — Allegiance drift (per class, `allegiance.py:164-166,192`, `_drift_allegiance` 363-417)
- **(a)** Each class's allegiance mass over the party terrain drifts by a four-term law: material-interest alignment (`fit`), organizing contact (MEMBERSHIP reach), a reactionary coupling from `fascist_alignment` toward fascist-labeled parties, and a betrayal term (prior-tick delivery gap, zero until a governing incumbent/committed org exists in the ledger). Deltas move mass under a partition-of-unity discipline (parties ∪ abstention sums to exactly 1.0).
- **(b)** For each party: `fit = interest_fit(interest[class], platforms[party])` (399, calls `math.sqrt` twice internally — §4); `contact = 1.0 if class in membership[party] else 0.0` (400); `betrayed = gap_ratio if party is incumbent-or-committed else 0.0` (401, `gap_ratio` computed 389-393 from the `policy_delivery` row: `min(1.0, max(0.0, gap/promised))` when both are numeric and `promised > 0.0`, else `0.0`); `delta = allegiance_drift(fit, contact, align_rate, contact_rate, delivery_gap_term=betrayed, betrayal_rate)` (402-409, pure linear combination: `align_rate*fit + media_rate*media_influence + contact_rate*contact - betrayal_rate*delivery_gap_term`, politics.py:183-188 — **`media_rate`/`media_influence` are never passed at this call site, so that term is provably always exactly `0.0` on every call, honest-absence by construction, not a runtime check**) → if `fascist_alignment > 0` and the party is a fascist vehicle (`_is_fascist_vehicle`, 419-421: substring-matches `party.attrs["ideology"]` against `_FASCIST_IDEOLOGY_TOKENS = ("fascist","reaction","revanch","settler")`, allegiance.py:77), `delta += align_rate * fascist_alignment` (410-414) → `masses, _abstention = apply_allegiance_drift(current, tuple(deltas))` (416, politics.py:191-212: `updated = [max(0.0, mass+delta) ...]`; `total = sum(updated)`; `if total > 1.0: return (mass/total for mass in updated), 0.0` else `return updated, 1.0-total` — clamp-then-conditionally-rescale-to-simplex) → `return dict(zip(ordered, masses))` (417).
- **(c) Reads:** class's own `allegiance` dict (current masses, default `{}`), `fascist_alignment`; each party's `ideology` string; the precomputed `interest`/`platforms`/`membership`/`delivery`/`committed` from Motion 0.
- **(d) Writes:** none inside the helper — the returned `dict[party_id -> float]` is written by the caller at `wrapped.update_node(node.id, allegiance=allegiance, ...)` (allegiance.py:172,179).
- **(e) Defines:** `politics.allegiance_align_rate` (0.05, `[0,1]`), `politics.allegiance_contact_rate` (0.03, `[0,1]`), `politics.allegiance_betrayal_rate` (0.04, `[0,1]`) — defines.yaml:1101-1103.
- **(f) Events:** none directly (feeds Motion 3's HOPE_SPIKE indirectly via allegiance values).

### Motion 2 — Hope field H(c) and THE VALVE (per class, `allegiance.py:167,170,172-178`, `_hope` 423-444, `_convert` 446-486)
- **(a)** H(c) is the allegiance-weighted, viability-discounted, promise-grounded improvement in believed survival-by-acquiescence — the same sigmoid the survival calculus adjudicates elsewhere, evaluated once under a promised overlay and once at the status quo. H(c) then throttles the first production Agitation→Organization conversion pathway (`organization += rate·agitation·(1−v·H)`), with a topology-routed disillusion-window boost/reroute into `fascist_alignment`.
- **(b)** `_hope`: for each party, `fit = interest_fit(...)` (440, sqrt again); `promised = max(0.0, fit) * phi_social_share * subsistence` (441); `delta = counterfactual_hope_gain(wealth, subsistence, promised, steepness)` (442, politics.py:54-72: `promised_p = calculate_acquiescence_probability(wealth+promised, subsistence, steepness)`; `status_quo = calculate_acquiescence_probability(wealth, subsistence, steepness)`; `return max(0.0, promised_p - status_quo)` — **two `math.exp` calls per party per class**, survival_calculus.py:41-43); `terms.append((allegiance[party], viability[party], delta))` (443) → `return min(1.0, hope_field(tuple(terms)))` (444, politics.py:37-51: `sum(a*v*max(0.0,d) for a,v,d in terms)` — a **hand-written upper-only clamp**, not `_write_clamped`). `_convert` (THE VALVE): `agitation = ideology.get("agitation", 0.0)` (467); `if agitation <= 0: return None, 0.0` (468-469, "nothing to convert" — no write); `gain = organizing_conversion_rate * agitation * valve_multiplier(hope, valve_strength)` (471-475, `valve_multiplier` = politics.py:22-34, `min(1.0, max(0.0, 1.0 - valve_strength*hope))` — a **two-sided nested min/max clamp**, a THIRD distinct clamp style in this one system); `if gain <= 0: return None, 0.0` (476-477); if an active disillusion window exists: `bridges_present` ⟹ `boost = disillusion_conversion_boost` else `fascist_delta = (disillusion_conversion_boost - 1.0) * gain` (480-485); `return min(1.0, organization + gain*boost), fascist_delta` (486, a **fourth** hand-written upper-only clamp).
- **(c) Reads:** `wealth`, `subsistence_threshold`, `ideology.agitation`, `organization`; the precomputed `platforms`/`viability`/`interest`/`allegiance` (this tick's, from Motion 1) and `windows` (Motion 0).
- **(d) Writes:** `SOCIAL_CLASS.hope` (graph-only, not a declared model field — §3); `SOCIAL_CLASS.organization` (conditional, only when `gain > 0`); `SOCIAL_CLASS.fascist_alignment` (conditional, only when `fascist_delta > 0`, written via a **fifth** clamp: `min(1.0, current + fascist_delta)`, allegiance.py:176-178) — all three via one `wrapped.update_node(node.id, **updates)` call (179).
- **(e) Defines:** `politics.phi_social_share` (0.25, `[0,1]`), `survival.steepness_k` (10.0, `> 0.0`, **unbounded above**), `politics.organizing_conversion_rate` (0.02, `[0,1]`), `politics.valve_strength` (0.6, `[0,1]`), `politics.disillusion_conversion_boost` (2.0, `>= 1.0`, **unbounded above**).
- **(f) Events:** `HOPE_SPIKE` — `_maybe_publish_spike` (allegiance.py:181-190, defined 488-516): fires when `hope - prev_hope > hope_spike_gain` (500-501, defines.yaml:1099, `0.3`, `>= 0.0` unbounded above); payload picks the **best-fit platform by `max(sorted(platforms), key=lambda pid: (interest_fit(...), pid))`** (502-505, a Python max-by-key selection over a dict's keys — structurally a `select-max`-shaped operation) and publishes `{class_id: str, hope: float, platform_id: str}` (506-516).

### Motion 3 — The political_form producer (`allegiance.py:191-205`)
- **(a)** Closes U3's deferred W-C opposition row: `political_labor_share = (loyal - oppositional) / (loyal + oppositional)`, where `loyal` = the tick's total allegiance mass across every class/party pair, `oppositional` = the tick's total organization. Published as a graph-scope scalar for `ContradictionSystem`'s `GraphInputs` @18.
- **(b)** Accumulated inside the per-class loop: `loyal_mass += sum(allegiance.values())` (192); `oppositional_mass += organization if organization is not None else attrs.get("organization", 0.0)` (193-195) → after the loop: `total = loyal_mass + oppositional_mass` (201); `if total > 0.0: wrapped.set_graph_attr("political_labor_share", (loyal_mass - oppositional_mass) / total)` (202-205, honest-absence: no write at all when `total == 0.0`, no divide-by-zero guard needed structurally).
- **(c) Reads:** every class's (this-tick) `allegiance` values and `organization` value — a running fold across the ENTIRE per-class loop, not a single-node quantity.
- **(d) Writes:** graph-scope scalar `political_labor_share` (`[-1.0, 1.0]`, conditional on `total > 0.0`).
- **(e) Defines:** none new.
- **(f) Events:** none.

Also inside the per-class loop, always: `context.persistent_data["politics.hope_by_class"] = new_hope` (197, written once after the full loop, not per-class) — the harness-scoped carryforward `_active_windows`... no, this is `_HOPE_KEY`, feeding the NEXT tick's `prev_hope` read in Motion 0. This is `TickContext.persistent_data`, the same mechanism the CURRENT BSL surface note (mission context) flags as having no BSL equivalent (`context.get`/`context.persistent_data` per the R9 gap analysis — see §3.6's list of "twenty-two of the thirty-four frozen systems" reaching for graph-scope-via-non-graph channels, bsl-language.rst:2652-2656, which names `context.persistent_data` explicitly alongside `graph.graph[...]`/`set_graph_attr`).

**Events emitted by the whole system: one distinct `EventType` (`HOPE_SPIKE`), fired conditionally per class.** Grep-confirmed — the only `EventType.` reference in allegiance.py is line 508.

## 3. TYPE INVENTORY

Runtime storage note (same finding as every prior port inventory in this estate):
`BabylonGraph.update_node` is a plain dict merge with no type coercion or quantization — Pydantic's
`SnapToGrid` grid rounding applies only at model instantiation (scenario seed / `WorldState`
round-trip), never mid-tick. All in-tick arithmetic below is raw Python `float`/`dict`/`str`.

| Attribute | Node/scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `allegiance` | SOCIAL_CLASS | `dict[str, float]` | keys = open party-org id set; values `[0,1]` (not independently enforced — `apply_allegiance_drift` clamps each mass `>= 0` and rescales the set to sum `<= 1.0`) | **Open map-valued field — the system's central storage gap** |
| `hope` | SOCIAL_CLASS | plain `float` (graph-only — NOT a declared `SocialClass` model field; `EXTRA_STAMPABLE_ATTRIBUTES` exemption, `registry.py:206`, `world_state.py:90`) | `[0.0, 1.0]` | transient per-tick scalar |
| `organization` | SOCIAL_CLASS | `Probability` | `[0.0, 1.0]` | unit-interval, cross-system load-bearing (§5) |
| `fascist_alignment` | SOCIAL_CLASS | `Intensity` | `[0.0, 1.0]` | unit-interval |
| `wealth` | SOCIAL_CLASS | `Currency` | `[0.0, ∞)` | unbounded real, money-semantic (read-only here) |
| `subsistence_threshold` | SOCIAL_CLASS | `Currency` | `[0.0, ∞)` | unbounded real, money-semantic (read-only here) |
| `ideology.class_consciousness` | SOCIAL_CLASS (nested `IdeologicalProfile`) | `float` | `[0.0, 1.0]` | unit-interval, nested-dict read |
| `ideology.agitation` | SOCIAL_CLASS (nested) | `float` | `[0.0, ∞)` | **unbounded real** (used directly as a multiplicand in the conversion gain) |
| `active` | SOCIAL_CLASS | `bool` | `{T,F}` | boolean gate (default `True`) |
| `org_type` | ORGANIZATION | `OrgType` (StrEnum) | closed 4+-member set, filtered to `POLITICAL_FACTION` | **Enum discriminant** (now landed on dev, ADR195/196) |
| `ideology` (party) | ORGANIZATION (`PoliticalFaction`) | `str`, `min_length=1`, **no enum, no pattern** | free text (e.g. `"liberal_imperial"`, `"fascist"`) | **Free string, matched by substring against a hardcoded token tuple** — a genuinely different shape from the enum-discriminant row above |
| `value_flow` (edge) | TRANSACTIONAL edge | `Currency` | `[0.0, ∞)` | unbounded real, money-semantic |
| `policy_delivery` | graph-scope | `dict[str, dict[str, Any]]` (class_id → `{incumbent_id: str, gap: float, promised: float, ...}`) | open key set (class ids) | **Open map-of-records, cross-system register (read)** |
| `electoral_disillusion` | graph-scope | `dict[str, dict[str, Any]]` (class_id → `{opened_tick: int, window_ticks: int, bridges_present: bool}`) | open key set | **Open map-of-records, cross-system register (read)** |
| `popular_front` | graph-scope | `dict{active: bool, since_tick: int, arms: dict[str,str], suppression: float}` | `arms` keyed by open party-id set | **Open map-of-records (nested), cross-system register (read)** |
| `political_labor_share` | graph-scope | `float` | `[-1.0, 1.0]` | **singleton scalar** — the one graph-scope value this system WRITES |
| `politics.donor_platform_weight`, `disillusion_conversion_boost`, `hope_spike_gain`, `steepness_k` (defines) | — | `float` | `>= 0` / `>= 1.0`, **unbounded above** | unbounded real coefficients (D-1-class hazard analog) |
| the other 6 consumed defines | — | `float` | `[0.0, 1.0]` | unit-interval coefficients |

**The `allegiance` map field — the single most severe finding.** `SocialClass.allegiance:
dict[str, float]` (social_class.py:323-333) is keyed by an **open, scenario-defined** set of
`PoliticalFaction` org ids — not a small closed enum the way Territory's `profile`/`territory_type`
were. `bsl-language.rst` §3.8 item 2 ("No sequence or map type", Q13, lines 2860-2876) is an
explicit, settled design ruling: BSL has no map type, full stop, and states its own re-modelling
recipe for the three cases the frozen estate actually needed (FIFO agenda → `NodeType` +
`select-min`; ordered acquisition list → edge + `select-max`; a set of boolean flags → one `Bool`
field per flag) — **none of which fit a genuinely continuous per-(class,party) VALUE over an open
key set.** The architecturally-sanctioned destination is CLAUDE.md's Amendment AG / ADR189
("attributed membership": the `(member, hyperedge)` pair becomes a first-class payload-carrying
element kind; dyadic-edge attribute landing was explicitly REJECTED in favor of this) — confirmed
in-grammar: `update-membership`/`membership-field-of` verbs exist in the BNF
(bsl-language.rst:~1341) but are named by the query-evaluation plan as **Slice 4 — "HASH-TOUCHING
(escalates)"** (`docs/superpowers/plans/2026-08-11-bsl-query-evaluation-plan.md:74,85,148,233,828`),
explicitly out of the landed Slice-1 scope and Director-escalation-gated per the mission's CURRENT
BSL surface note. So the RIGHT destination is named, but unbuilt.

**The `ideology` free-string field — a distinct, smaller gap.** `Str` (bsl-language.rst §3.1) is
"UTF-8, NFC... Only `:material-basis` and vector ids. No operations" — not a `deffield`-legal type
at all (the closed seven: `int, bool, currency, probability, intensity, coefficient, enum`). Unlike
`allegiance`, this one IS resolvable without a new lane: `enum` is now landed (D101, ADR191 R4/the
Organization-as-game-object ruling), so `PoliticalFaction.ideology` can be re-declared as a small
closed `enum` (e.g. `PartyIdeology`) — but `_is_fascist_vehicle`'s SUBSTRING match against
`_FASCIST_IDEOLOGY_TOKENS` (four independent tokens, none of which is itself a full ideology label)
cannot transcribe as an enum `=` comparison directly; it needs either an OR-chain over every
enum member the tokens currently match, or (cleaner, matching the §3.8 item-1 "companion field"
precedent) a content-authored companion `bool` tag per party declaring vehicle status directly.
Either is a D-record that changes the MECHANISM (substring search → closed classification) while
preserving the OUTCOME — and naming the closed ideology-label vocabulary is itself content a
Director may want eyes on (see RESERVED-LINE note below).

**The three graph-scope dict registers — same map-type gap as `allegiance`, cross-system.**
`policy_delivery`/`electoral_disillusion`/`popular_front` are all written by OTHER systems
(PolicySystem, ElectoralSystem) and read here by raw string (to avoid a forward import,
allegiance.py:152-153,215-216,239-241). All three are dicts keyed by open node-id sets — the exact
same §3.8-item-2 gap as `allegiance`, not a variant of it.

**`political_labor_share` — the one graph-scope channel with a NAMED, if unbuilt-for-content,
mechanism.** Being a true per-graph SINGLETON scalar (not a map), it is the one channel that fits
§3.6's draft ruling ("Graph-scope state is ordinary node state on a declared carrier node type",
lines 2650-2688) — landed at the evaluator/manifest **test** level (`evaluator.rs`'s carrier-node
accumulation test, `manifest.rs`'s `:ceiling 1` test) but adopted by **zero** real content pack on
dev, and minting the carrier `NodeType` is itself "amendment territory" per the ruling's own text
(one new closed-vocabulary member).

**RESERVED-LINE note.** `_FASCIST_IDEOLOGY_TOKENS = ("fascist", "reaction", "revanch", "settler")`
(allegiance.py:77) is a hardcoded classification of which political-party ideology labels count as
"fascist vehicle" for the purposes of the reactionary-coupling drift term and the disillusion
routing's Obama→Trump-pipeline branch. This is Director-adjacent ideological/theoretical framing
(the same family as doctrine content and National Question parameters) baked directly into engine
code rather than into `defines.yaml`. Transcribed verbatim per port-as-is law; **not** proposed for
change here — flagged so a Director sign-off on the eventual enum/bool re-modelling's exact
membership set is requested at port time, not decided by the porting agent.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`). Shapes, in execution order:

1. **Cosine-similarity fit** (`interest_fit`, politics.py:215-231, called at allegiance.py:399,440,504
   — up to `2 × n_parties + 1` times per class per tick): `dot = Σ i_k·p_k`; `norm_i = sqrt(Σ i_k²)`;
   `norm_p = sqrt(Σ p_k²)`; `return dot/(norm_i*norm_p)` guarded by `norm_i==0 or norm_p==0 → 0.0`.
   **`math.sqrt` is called twice per invocation. `sqrt` is NOT in `DECLARABLE_INTRINSICS`
   (`declarations.rs:110`, exactly `["exp","log","floor"]`) — a distinct, previously-unnamed libm
   hazard with no declared BSL intrinsic at all, separate from the exp/sigmoid hazard below.**
2. **Vector accumulation + normalization** (`platform_vector`, politics.py:115-148, called once per
   party per tick, allegiance.py:324-328): `acc[i] += weight*component` (base terms), `acc[i] +=
   donor_weight*share*component` (donor term) — plain weighted sums over a **fixed 2-dimensional**
   vector (`(margin, consciousness)`, always exactly 2 components since `_interest_vector` always
   returns a 2-tuple) — then `norm = sqrt(Σ x²)`, `return tuple(x/norm for x in acc)` guarded by
   `norm==0.0`. A THIRD `sqrt` call site. Being fixed-size-2, the vector itself is mechanically
   unrollable into two named scalar `deffield`s (not itself blocked) — the block is purely `sqrt`.
3. **The frozen acquiescence sigmoid, reused for a NEW quantity**
   (`calculate_acquiescence_probability`, survival_calculus.py:21-43, called TWICE per party per
   class via `counterfactual_hope_gain`): `exponent = -steepness_k*(wealth-subsistence)`; `exponent =
   max(-500, min(500, exponent))` (an overflow-guard two-sided clamp, itself a THIRD clamp style
   variant — clamping the ARGUMENT to a transcendental, not the result); `return 1.0/(1.0 +
   math.exp(exponent))`. `exp` IS a declared intrinsic (syntactically expressible) — but per
   CLAUDE.md's Amendment AE/ADR172 ruling 5 and ADR173 (2026-07-29): **"The logistic form is the
   frozen Python reference's, NOT the going-forward law... in the Rust/BSL engine P(S\|A) is the
   measure of class members whose wealth clears subsistence — the S-curve EMERGES from
   within-class wealth dispersion; no imposed functional forms."** `counterfactual_hope_gain`
   evaluates this SAME stipulated sigmoid twice (promised-overlay minus status-quo) purely to
   synthesize H(c) — a mechanical transcription would directly re-instantiate the functional form
   the Constitution has already retired for its ORIGINAL use. **This is a PORT-QUESTION requiring a
   Director/design ruling on what H(c)'s counterfactual evaluation becomes once P(S\|A) itself is
   redefined as an emergent measure — not a routine D-record**, and it is the second-most-load-bearing
   finding in this inventory after the `allegiance` map gap.
4. **`valve_multiplier`** (politics.py:22-34): `min(1.0, max(0.0, 1.0 - valve_strength*hope))` — one
   subtract, one multiply, a two-sided nested-min/max clamp (the Territory-precedent `_write_clamped`
   SHAPE, but hand-written inline, not via the shared helper — `_write_clamped` is never called
   anywhere in this system, confirmed by grep).
5. **`hope_field`** (politics.py:37-51): `sum(a*v*max(0.0,d) for a,v,d in terms)` — a fold over
   `n_parties` triple-products, each lower-clamped individually before summing; the caller then
   applies `min(1.0, hope_field(...))` (allegiance.py:444) — an upper-only clamp on the OUTSIDE of
   the fold, a FOURTH distinct clamp shape (lower-clamp-per-term, then upper-clamp-on-sum).
6. **`allegiance_drift`** (politics.py:151-188): a plain 4-term linear combination — no hazard, but
   two of the four terms (`media_rate*media_influence`) are provably always `0.0` at this call site
   (dead parameter, honest-absence, not a runtime branch — allegiance.py never passes them).
7. **`apply_allegiance_drift`** (politics.py:191-212): `updated = [max(0.0, m+d) for m,d in
   zip(...)]` (lower-only clamp, a FIFTH style — per-element `max()` over a LIST, not a single
   scalar); `total = sum(updated)`; `if total > 1.0: return (m/total for m in updated), 0.0 else
   return updated, 1.0-total` — a conditional DIVISION (mass-conservation renormalization onto the
   unit simplex), gated by a comparison. Arithmetically simple; the block is entirely on the WRITE
   TARGET (§3's `allegiance` map), not this arithmetic.
8. **`_convert`'s gain + two more clamps** (allegiance.py:471-486): `gain = rate*agitation*
   valve_multiplier(...)`; `min(1.0, organization + gain*boost)` (upper-only, SIXTH style, on the
   `organization` write); `fascist_delta = (multiplier - 1.0) * gain` (unguarded, but structurally
   `>= 0` since `multiplier >= 1.0` and this branch only reached when `gain > 0`); `min(1.0,
   fascist_alignment + fascist_delta)` (upper-only, SEVENTH style, on the `fascist_alignment` write).
9. **`_drift_allegiance`'s inline gap-ratio clamp** (allegiance.py:389-393): `min(1.0, max(0.0,
   gap/promised))` — a two-sided nested min/max identical in SHAPE to item 4, computed inline rather
   than via a named helper (an EIGHTH clamp SITE, though the same style as item 4) — plus a
   DIVISION (`gap/promised`), guarded by `promised > 0.0` and both operands being numeric.

**No `_write_clamped` usage anywhere in this system** — every one of the eight-plus clamp sites
above is hand-written inline, spanning at least five distinct STYLES (two-sided nested min/max ×2
sites, upper-only `min` ×4 sites, lower-only `max` in a list comprehension ×1 site, lower-clamp-
then-upper-clamp-on-fold ×1 site, argument-clamp-before-a-transcendental ×1 site). Port-as-is:
transcribe each faithfully, do not unify.

**No Real→Int demotions in allegiance.py's own arithmetic path** (the two `int(row.get(...))` calls
at allegiance.py:226-227, inside `_active_windows`, read FROM the already-blocked
`electoral_disillusion` register, so this is moot pending that register's storage resolution, not an
independent blocker).

**Bare non-integer literals:** `1.0` and `0.0` appear throughout every clamp site above (the same
"no bare non-integer literal" BSL parser constraint every prior port inventory in this estate has
flagged) — mechanical, not a design question, same `c`-suffixed-`defconst`/Real-zero-promotion
precedent applies.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.42**, confirmed against `_SYSTEM_CLASSES` (simulation_engine.py:328-363):
  `ConsciousnessSystem (17.0) → FascistFactionSystem (17.4) → AllegianceSystem (17.42) →
  ElectoralSystem (17.45) → PolicySystem (17.47) → SovereigntySystem (17.5) → ... →
  ContradictionSystem (18.0)`.
- **Reads from a same-tick prior system:** `SOCIAL_CLASS.fascist_alignment` (drift toward the
  reactionary coupling — written by FascistFactionSystem @17.4, the immediately-prior system per
  the docstring, allegiance.py:3-7); `SOCIAL_CLASS.ideology.class_consciousness`/`.agitation` (this
  tick's post-decay write, from ConsciousnessSystem @17.0, per the docstring).
- **Reads from a system that runs LATER this tick (one-tick lag, "the I-ORD/T-7 grain"):**
  `policy_delivery` (PolicySystem @17.47, AFTER this system — always reads LAST tick's ledger, never
  this tick's, by construction of the tick order); `electoral_disillusion` (ElectoralSystem @17.45,
  AFTER this system — same one-tick lag); `popular_front` (also ElectoralSystem @17.45 — same lag).
  All three registers are therefore, on tick 1 of any run, necessarily empty/absent (no prior tick
  exists) — the "pre-U9"/"pre-U10"/"pre-U12" honest-absence arithmetic the docstring names is not
  merely a fallback branch, it is the LITERAL first-tick behavior on every scenario.
- **Writes consumed later this tick / downstream ticks:**
  - `SOCIAL_CLASS.organization` — read downstream by `control_ratio.py` (@12.0, runs BEFORE this
    system in absolute tick order but consumes the WRITE on the NEXT tick's pass), `struggle.py`
    (@16.0), `survival.py` (@15.0) — both also run before 17.42 in absolute position, so also
    next-tick consumers; grep-confirmed no same-tick reader after 17.42.
  - `SOCIAL_CLASS.fascist_alignment` — read by FascistFactionSystem (@17.4, next-tick), the
    ContradictionSystem's consolidation-pressure measure (@18.0, same-tick, via
    `_consolidation_pressure`'s ideology-pair scan touching every `social_class` node —
    ElectoralSystem's `_popular_front_conjuncture` reads the analogous quantity too, next tick).
  - `SOCIAL_CLASS.allegiance` — read by no OTHER System (grep-confirmed: no `engine/systems/*.py`
    other than allegiance.py itself reads the `allegiance` attribute) — but IS read by the
    AI/narrative layer and by AllegianceSystem's own NEXT-tick pass (`current_raw =
    attrs.get("allegiance")`, allegiance.py:376). A genuinely self-referential (this system reads
    its own prior-tick write) plus narrative-only downstream channel — not an engine-internal one.
  - `SOCIAL_CLASS.hope` — read by no OTHER System (transient, dropped on reconstruction, §3) and by
    no other same-tick or next-tick code path except this system's own `context.persistent_data`
    carryforward for the HOPE_SPIKE delta comparison.
  - graph-scope `political_labor_share` — read by exactly ONE downstream consumer,
    `ContradictionSystem` @18.0 (`contradiction.py:471`, `GraphInputs.political_labor_share`), the
    immediately-next system in tick order.
- **Context/service usage with no BSL equivalent:** `context.persistent_data["politics.hope_by_class"]`
  (allegiance.py:156-157,197) — a cross-TICK (not merely cross-system) carryforward dict keyed by
  class id, used ONLY by this system's own next-tick HOPE_SPIKE delta comparison. `bsl-language.rst`
  §3.6 (lines 2652-2656) names `context.persistent_data` explicitly, alongside `graph.graph[...]`/
  `set_graph_attr`, as one of the three non-graph channels the R9 gap analysis's Q6 finding covers
  ("the single most pervasive gap in the estate") — but the §3.6 ruling's own remedy (the
  carrier-node re-modelling) is stated for GRAPH-scope state; whether a cross-TICK Python dict like
  `persistent_data` is meant to fold into the same remedy, or needs its own, is not resolved by the
  text read for this inventory — flagged as an open question, not asserted either way.
- **Dormancy — AllegianceSystem is LIVE on canonical `qa:regression`, unlike Territory.**
  `tools/regression_scenarios.py` declares `AllegianceSystem` `SystemEvidence` rows on all five P25
  U13 electoral goldens: `mitterrand` (line 2289, `betrayal_integral_crossed` event-kind claim),
  `syriza` (2349, `C004.fascist_alignment` entity-delta claim), `weimar` (2402,
  `C001.fascist_alignment`), `debs` (2467, `C001.organization`), `bernie_valve` (2540, `hope_spike`
  event-kind claim). Each factory (`electoral_goldens.py`, layered via
  `apply_political_terrain`/`electoral_fixture.py`) seeds real `PoliticalFaction` orgs, `MEMBERSHIP`
  edges, and TRANSACTIONAL funding edges (`_funding("org/donor-finance", "org/party-liberal",
  100.0)` etc., `electoral_fixture.py:191-193`; `weimar` re-weights a fascist-donor edge to
  `value_flow=90.0`, `electoral_goldens.py:387-398`) — confirming BOTH `_membership_map` and
  `_funding_shares` are exercised against real, non-trivial data on a live golden. **A port's
  conformance fixtures have a real canonical-golden precedent to draw from (unlike Territory's
  ADJACENCY/spillover paths) — but the goldens themselves cannot serve as the byte-gate oracle for
  the blocked motions (map storage) until those motions are actually portable.**
- **`tools/regression_scenarios.py`'s own `at_rest` declarations independently confirm** that
  `EdgeType.TRANSACTIONAL`/`MEMBERSHIP` edges on these goldens are EXPECTED to carry `value_flow`/
  routing dynamics ("political-terrain edges... are overlays that route allegiance, claims and
  registers — never per-tick value_flow/tension dynamics", the repeated `at_rest` reason string
  across all five goldens) — i.e. the coverage-gap author already independently understood these
  edges as allegiance-routing infrastructure, corroborating this inventory's read of the system.

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| TRAP 3 guard (parties-exist / classes-exist, allegiance.py:101-119) | **PORTABLE NOW** | `exists`/`forall`-shaped over `nodes(NodeType/ORGANIZATION)` filtered by `org_type` — Slice 1's `exists`/`forall`/`nodes` cover this exactly; no storage or intrinsic dependency. |
| `_membership_map`/`_funding_shares` terrain readers (allegiance.py:261-278) | **PORTABLE WITH D-RECORD** | Pure `fold sum` over typed edges (`MEMBERSHIP`/`TRANSACTIONAL`), landed Slice-1 shape (structurally identical to Territory's favorably-reformulable Phase-3 spillover pull-fold). The RESULT (a per-party share/reach) only needs to be STORAGE-visible if consumed across rule boundaries — see the nested-fold note below. No sqrt, no sigmoid. `donor_platform_weight`'s unbounded-above domain is the familiar D-1-class hazard (bare-scaled-Int/Real-declared-domain workaround, ADR183 class) IF and only if it ever multiplies a `Currency`-typed operand — here it multiplies a Real-valued interest component, so the #500 Currency scale-op question does not even arise (same non-issue Territory's Phase-4 camp decay had). |
| `_interest_vector` (allegiance.py:280-291) | **PORTABLE NOW** | One subtraction (`wealth - subsistence`, both `:field`-sourced Reals under the ADR183 int-workaround) plus one `:field` read (`class_consciousness`). No hazard. |
| `_platforms`/`platform_vector` (allegiance.py:293-329, politics.py:115-148) | **BLOCKED — undeclared `sqrt` intrinsic** | The 2-vector unrolling itself is a content-modeling non-blocker (fixed dims=2); the norm computation's `math.sqrt` has zero declared-intrinsic coverage (`DECLARABLE_INTRINSICS = ["exp","log","floor"]`, declarations.rs:110). Name the exact missing lane: **`sqrt` intrinsic, not requested by any prior port inventory in this estate.** |
| `_viability` (allegiance.py:331-357) | **BLOCKED (partial) — inherits the `popular_front` map gap** | The base (pre-U12) arithmetic — `0.5*funding_share + 0.5*member_share` — is portable arithmetic on its own; the `committed` special-case (`viability=1.0` when the party is front-committed) reads `popular_front.arms`, which is BLOCKED (§3). Declaring `committed` `:const empty` (the honest-first-tick default, §5) would make the base arithmetic PORTABLE WITH D-RECORD, but that is a scope decision for the eventual port, not assumed here. |
| `_drift_allegiance` (allegiance.py:363-417, politics.py:151-212) | **BLOCKED — `sqrt` (via `interest_fit`) + `allegiance` map-storage gap** | Even the honest pre-U9/pre-U12 base arithmetic (betrayal term and committed-coupling forced to zero) cannot land, because its OUTPUT — the drifted `dict[party_id -> float]` — has no BSL storage representation at all (§3's headline finding). Name the exact missing lane: **Slice 4 (attributed-membership storage, ADR189-designed, `update-membership`/`membership-field-of` unbuilt).** The `policy_delivery`/`popular_front` register READS are a second, independent instance of the same map-type gap, gating only the full (not base) U9/U12 semantics. |
| `_hope`/`hope_field`/`counterfactual_hope_gain` (allegiance.py:423-444, politics.py:37-72) | **BLOCKED — `sqrt` + PORT-QUESTION (ADR172/173 sigmoid ruling)** | `interest_fit` reuse blocks on `sqrt` as above. Independent of storage: `counterfactual_hope_gain` re-evaluates the frozen, going-forward-RETIRED acquiescence sigmoid — this needs a Director/design ruling on what H(c)'s counterfactual becomes once P(S\|A) is redefined as an emergent measure, not a mechanical transcription. The `hope` WRITE TARGET itself (a scalar `[0,1]` field) is fine — the block is entirely upstream, in the value's derivation. |
| `_convert` / THE VALVE (allegiance.py:446-486) | **BLOCKED (partial) — inherits `hope`'s block; base gain arithmetic + writes are fine** | `valve_multiplier`'s own arithmetic is trivial and portable; `organization`/`fascist_alignment` are ordinary scalar `deffield`-legal writes. The block is entirely upstream (via `hope`) plus the disillusion-window branch's dependency on `electoral_disillusion` (map-type gap, same as `policy_delivery`). |
| `HOPE_SPIKE` emission (allegiance.py:488-516) | **BLOCKED — string event payload, `<payload-item>` grammar** | `platform_id: str` (a free, scenario-defined org-node id) cannot be carried in an `emit` payload: `<payload-item> ::= "(" <symbol> <expr> ")"` and `<expr>` is closed to number/bool/enum-ref (bsl-language.rst §3.8 item 4, ~line 2892). Unlike `class_id` (implicit via the firing rule's own subject node, no landed pack needs to carry it explicitly — precedent: `vitality.bsl`/`lifecycle.bsl`/`dispossession.bsl`'s emits carry only numeric/bool/enum payload items), `platform_id` genuinely NAMES a dynamic node with no enum-ref available for an open, scenario-defined party set. Also gated upstream by `hope`'s own block (the spike condition needs `hope`) and by `interest_fit`'s `sqrt` (the best-platform selection). Name the exact missing lane: **event-payload grammar has no node-reference payload kind (not merely "Slice N" — a `<payload-item>` grammar gap distinct from the query-evaluation slices).** |
| `political_labor_share` producer (allegiance.py:199-205) | **BLOCKED (partial) — inherits `allegiance`'s map gap; the write mechanism itself is D-record-able** | The scalar itself is the one graph-scope value with a NAMED mechanism (§3.6's carrier-node draft ruling, landed at test level, unused in any pack, "amendment territory" to adopt for content) — PORTABLE WITH D-RECORD on that basis alone. But its INPUT (`loyal_mass = Σ allegiance.values()`) is a fold over the blocked `allegiance` map, so the producer cannot actually compute a faithful value until that storage lands. |
| `PoliticalFaction.ideology` string classification (`_is_fascist_vehicle`, allegiance.py:419-421) | **PORTABLE WITH D-RECORD** | `Str` has no BSL storage type (§3.1) — but `enum` is landed (D101), so re-declaring `ideology` as a closed enum with a companion `bool` "is-fascist-vehicle" tag (the §3.8 item-1 companion-field precedent) preserves the classification OUTCOME while replacing the substring-search MECHANISM. RESERVED-LINE: the exact enum membership/tag assignment is Director-adjacent content (§3), name it but do not decide it here. |

**Overall verdict: BLOCKED, not partially portable.** Unlike Territory (which had a genuinely
self-contained, always-portable Phase 1), every one of AllegianceSystem's three motions is blocked
either directly (the `allegiance` map write, the `sqrt` intrinsic, the sigmoid PORT-QUESTION, the
event-payload grammar) or by inheritance from one of those four. The TRAP 3 guard and the terrain
readers (`_membership_map`/`_funding_shares`/`_interest_vector`) are the only pieces genuinely
portable in isolation today, and they compute nothing observable without the blocked motions
downstream of them.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_allegiance.py` | 269 | **Primary conformance-oracle candidate.** Direct `AllegianceSystem().step()` calls against hand-built `_electoral_graph()`/`_two_node_graph()` fixtures (not the full engine) — TRAP-3 party-less-graph-untouched proof, allegiance-mass bounds/determinism, hope bounds, THE VALVE monotonicity (`test_valve_throttles_the_conversion`, L-VALVE), HOPE_SPIKE firing/payload, `political_labor_share` bounds, round-trip honesty (`allegiance` persists, `hope` drops), and one FULL 34-system engine-tick integration test (`TestFullEngineTick`) proving the producer feeds `ContradictionSystem`'s `GraphInputs.opposition_states["political_form"]` end to end. Every scenario here is a candidate `.bscn` conformance vector, ONCE the storage/intrinsic blockers are resolved — none can be transcribed today given §6. |
| `tests/unit/formulas/test_politics.py` | 286 | **Property-law surface for the pure kernel** — the exact behavioral-contract role Territory's `test_law_territory_system.py` played: `TestLValve` (monotone, bounded, zero-hope-never-throttles), `TestLHopeMaterial` (no promise ⟹ no hope, `hope_field`/`counterfactual_hope_gain` laws), `TestLPrz` (platform-breadth/alignment trade-off, donor-pull), `TestAllegianceDrift`/`TestApplyAllegianceDrift` (sign structure, mass conservation, clamp-at-zero, oversubscription rescale). `TestTurnoutShare`/`TestCompetitiveness` cover functions NOT on AllegianceSystem's call path (ElectoralSystem's) — not part of this system's own conformance surface. |
| `tests/unit/engine/systems/test_electoral_goldens.py` | 311 | Behavioral-contract suite for the five golden scenarios (mitterrand/syriza/weimar/debs/bernie_valve) — exercises AllegianceSystem as PART OF the full electoral machine's end-to-end story, not in isolation; secondary reference for what the goldens actually prove per-scenario. |
| `tests/unit/engine/scenarios/test_electoral_goldens_factories.py` | 99 | Factory-shape tests for the five golden builders (`electoral_goldens.py`) — confirms the seeded terrain (parties, edges, doctrine stances) matches the factories' own documented intent; useful for validating a future `.bscn` transcription's seed data against the Python reference. |
| `tests/unit/engine/systems/test_electoral.py` | 850 | ElectoralSystem's own primary test file — touches `AllegianceSystem`/`allegiance_drift` only incidentally (shared fixtures / cross-system integration checks), not this system's own oracle. |
| `tests/unit/engine/systems/test_policy.py` | 728 | PolicySystem's own primary test file — the producer of `policy_delivery`, AllegianceSystem's own consumer-side test for that register lives in `test_allegiance.py`'s betrayal-term coverage (none observed as a DEDICATED case — the betrayal term's pre-U9/pre-U10 zero-default is what `test_allegiance.py` actually exercises; the live nonzero-gap path is exercised only end-to-end via `mitterrand`'s golden, not a `test_allegiance.py`-local unit case). |
| `tests/unit/engine/test_system_order.py` | 300 | Confirms tick-position ordering invariants across the full 34-system registry, including AllegianceSystem @17.42's slot — structural, not behavioral, oracle. |
| `tests/unit/sentinels/test_gate_coverage.py` | 140 | Confirms AllegianceSystem's `qa:regression` `SystemEvidence` rows (§5) are declared and satisfied — a coverage-gate test, not a math oracle. |

**No dedicated `tests/unit/engine/laws/test_law_allegiance*.py` file exists** (grep-confirmed against
`tests/unit/engine/laws/`) — the property-law role Territory's dedicated law-test file played is
instead served by `test_politics.py` (the pure-kernel laws) plus `test_allegiance.py`'s own
system-level assertions (`test_valve_throttles_the_conversion` etc.) — a genuine but real split
across two files rather than one dedicated law-test module.

**`qa:regression` byte-gate coverage.** Per §5, AllegianceSystem is LIVE (not dormant) on all five
P25 U13 electoral goldens — `tools/regression_test.py::graph_content_hash` hashes every node/edge
attribute the `WorldState→graph` projection carries, so any drift in this system's outputs on those
five scenarios is caught by the byte-identical hash gate today, in the frozen Python engine. Once
the storage/intrinsic blockers in §6 are resolved, these five goldens are the natural first source
for hand-built `.bscn` conformance fixtures — unlike Territory, no NEW scenario-seeding gap needs to
be closed first; the party terrain, funding edges, and membership edges already exist in a real,
tuned, spot-run-verified canonical scenario family.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) with fresh `rg`/Read. Five corrections,
six confirmations. The §3/§4 storage-and-intrinsic analysis is the strongest part of this report and
survives intact; §5's cross-system channel table and §7's byte-gate paragraph do not.

### CORRECTIONS

1. **CORRECTION — §5 is wrong on this system's two most load-bearing writes.** It records
   `SOCIAL_CLASS.allegiance` as "read by no OTHER System … A genuinely self-referential … plus
   narrative-only downstream channel — not an engine-internal one", and `SOCIAL_CLASS.hope` as "read
   by no OTHER System … and by no other same-tick or next-tick code path". Both are read
   **same-tick** by `ElectoralSystem` @17.45, whose own module docstring names the dependency
   verbatim: *"the tick's per-class ``allegiance`` masses and ``hope`` field — read SAME tick,
   17.42 < 17.45, because ``hope`` never survives the WorldState round-trip"* (electoral.py:4-5).
   Call sites, all fresh-grepped: `allegiance` at electoral.py:484-485 (`_front_suppression`),
   :644-645 (`_allegiance_masses`), :709-710 (`_turnout`), :732-738 (`_count_votes`), :981 (the
   plurality guard); `hope` at electoral.py:711 (`hope = float(attrs.get("hope", 0.0) or 0.0)`) and
   :716, feeding the turnout law. `world_state.py:86-90`'s own comment for `hope` says it flatly:
   *"AllegianceSystem @17.42 recomputes and ElectoralSystem @17.45 consumes same-tick (the
   threat_score precedent)"*. This inverts the section's picture: `allegiance`/`hope` are THE
   engine-internal channel this system exists to produce, and the port's conformance obligation runs
   to ElectoralSystem, not to a narrator.

2. **CORRECTION — §7's byte-gate claim is wrong for `hope` and for `political_labor_share`.**
   `graph_content_hash` (`tools/regression_test.py:924-964`) hashes `state.to_graph()`'s **nodes and
   edges only**, and its docstring excludes graph metadata ("Graph *metadata* (``g.graph``: economy,
   event log, opposition states) is also excluded"). `hope` is a `SOCIAL_CLASS_COMPUTED_FIELDS`
   member (world_state.py:86-90) — `from_graph` drops it, `to_graph` never re-emits it, so it has
   **zero** gate coverage. `political_labor_share` is a `SUPERSTRUCTURE_REGISTERS` member
   (`src/babylon/models/superstructure.py`) carried across the round trip through `G.graph`
   (`_harvest_superstructure_registers`, world_state.py:344-352; the re-stamp at :823-830) — i.e.
   graph metadata, also excluded. What IS gated on the five goldens is `allegiance` itself (a
   declared `SocialClass` field, social_class.py:323), `organization` and `fascist_alignment`. The
   sentence "any drift in this system's outputs on those five scenarios is caught by the
   byte-identical hash gate today" must be narrowed to those three.

3. **CORRECTION — the three graph-scope register rows miss §3.6's own second half, and miss that
   `the` is the actual gate.** The report cites §3.6 correctly for the `political_labor_share`
   singleton but files `policy_delivery`/`electoral_disillusion`/`popular_front` wholesale under the
   §3.8 map gap. The ruling's closing paragraph rules the opposite for keyed registers: *"It does not
   make every register a singleton: per-sovereign and per-county registers are ordinary nodes of
   ordinary types, reached by ordinary queries. ``the`` and the carrier discipline are for the values
   that are genuinely one-per-graph"* (bsl-language.rst:2686-2688). `electoral_disillusion` is keyed
   by class id and `popular_front.arms` by party id — both decompose onto the entity's own node
   (`social-class/disillusion-opened-tick` int, `…-window-ticks` int, `…-bridges-present` bool; an
   `ORGANIZATION`-scoped front-arm enum), all landed Slice-1 shapes (`nodes`/`field-of`/`update-node`,
   `SERVED_QUERY_HEADS` at evaluator.rs:527 and the `eval_form` arms at :556-559). Only
   `popular_front`'s genuinely-singleton half (`active`/`since_tick`/`suppression`) needs the carrier
   node and `the` — which is `("the", "slice 2")` at evaluator.rs:506, and is the ONE occurrence of
   the string `"the"` in that whole file. The sibling PolicySystem inventory states this split
   explicitly; this one should adopt it.

4. **CORRECTION (sharpening, not retraction) — the `sqrt` finding is right but under-specified.**
   `math.sqrt` is confirmed at politics.py:145 (`platform_vector`'s norm) and :227-228
   (`interest_fit`'s two norms), and confirmed absent from `DECLARABLE_INTRINSICS`
   (`declarations.rs:110`, `["exp","log","floor"]`). What the row should record is the *shape* of the
   ask: in both sites `sqrt` is a normalizing DENOMINATOR over a fixed 2-vector, and `interest_fit`
   returns `dot / (norm_i * norm_p)` — a cosine. There is no `exp`/`log` reconstruction that
   reproduces that rounding, so the gap is real; but "declare a `sqrt` intrinsic" is a strictly
   smaller, mechanically-scoped ask than the ADR172/173 sigmoid question sitting beside it in the same
   verdict, and conflating the two in one BLOCKED row hides that one of them is a one-line
   `DECLARABLE_INTRINSICS` amendment and the other is a Director design ruling.

5. **CORRECTION — `policy_delivery`/`electoral_disillusion` are not merely one-tick-stale; they are
   provably ABSENT on most of the estate.** Both are `SUPERSTRUCTURE_REGISTERS` members under
   honest-absence carriage, whose own module docstring states: *"Honest absence (Constitution III.11):
   a register that was never written is NOT carried as an empty value … The six party-less
   qa:regression scenarios therefore never see any of these names at all"*
   (`models/superstructure.py`). Adding `org_probe` (no party terrain) that is 7 of 12 scenarios with
   the names structurally absent; and per the sibling PolicySystem inventory, `weimar`/`debs` seed
   neither `policy_agenda` nor `electoral_governments`, so `policy_delivery` is never written there
   either. The "declare `:const` absent" fallback is therefore **exact** on 9 of 12 scenarios, in the
   Metabolism-D-2 "provably uniform" class — a stronger statement than "the LITERAL first-tick
   behavior on every scenario".

### CONFIRMATIONS

6. **CONFIRMATION — the `allegiance` open-map gap is the correct headline finding.**
   `SocialClass.allegiance: dict[str, float]` (social_class.py:323), keyed by an open,
   scenario-defined set of `PoliticalFaction` org ids. The named destination is right:
   `("membership-field-of", "slice 4")` at `evaluator.rs:511`, with the table's own comment naming it
   "the CanonicalState-widening storage lane — Director-ruled deferred to first consumer". Amendment
   AG / ADR189 is the ratified design; the accessor is unbuilt.

7. **CONFIRMATION — `hope` is graph-only.** `SOCIAL_CLASS_COMPUTED_FIELDS` (world_state.py:86-90)
   and the `EXTRA_STAMPABLE_ATTRIBUTES` exemption both confirmed; correction 1 above is about who
   reads it, not about its storage class.

8. **CONFIRMATION — the acquiescence-sigmoid PORT-QUESTION, and it is correctly ranked.**
   `survival_calculus.py:43`: `return 1.0 / (1.0 + math.exp(exponent))`, reached twice per
   (class, party) via `counterfactual_hope_gain` (politics.py:54-72). ADR173's own words retire this
   functional form for its ORIGINAL use, so re-instantiating it to synthesize a NEW quantity (H(c))
   is squarely a Director ruling, not a D-record. Agreed that this is second only to the map gap.

9. **CONFIRMATION — tick position 17.42** (allegiance.py:87), ordering per `_SYSTEM_CLASSES`
   (simulation_engine.py:328-363).

10. **CONFIRMATION — liveness on the five electoral goldens.** `apply_political_terrain`
    (`electoral_fixture.py`) seeds real `PoliticalFaction` orgs plus MEMBERSHIP and TRANSACTIONAL
    funding edges; `_solidarity(...)` at `electoral_goldens.py:474` (`debs`, 0.4) and `:534`
    (`bernie_valve`, 0.4) are the two real SOLIDARITY seeds. Unlike Territory, the conformance terrain
    already exists.

11. **CONFIRMATION with an addendum — the RESERVED-LINE flag on `_FASCIST_IDEOLOGY_TOKENS` is
    correct and the surface is wider than one file.** The identical four-token tuple
    `("fascist", "reaction", "revanch", "settler")` is hardcoded twice, independently:
    `allegiance.py:77` (party ideology labels) and `reactionary.py:71`
    (`BalkanizationFaction` ideology labels). That is one ideological classification with two
    engine-code homes and no `defines.yaml` entry — a Director-visible line item in its own right, not
    merely a per-system flag. (The sibling FascistFactionSystem adjudication records a live
    mis-classification defect arising from the second copy.)

### INADEQUATE-COVERAGE NOTE

§5's cross-system channel table was not verified against `electoral.py` and is wrong on both of this
system's primary outputs. A re-read must add: (a) every `ElectoralSystem` read of `allegiance` and
`hope`, by line (electoral.py:484-485, 644-645, 709-711, 716, 732-738, 981), with the
`world_state.py:86-90` comment as corroboration; (b) a restatement of §7's byte-gate paragraph
against `graph_content_hash`'s actual node/edge-only scope, naming the three fields it does cover;
(c) a rework of the three register rows against bsl-language.rst:2686-2688 (per-entity → ordinary
nodes → landed; singleton → carrier + `the` → Slice 2). §§1-4 and §6's `sqrt`/sigmoid rows need no
re-work beyond correction 4's sharpening.

### FINAL VERDICT

**BLOCKED — sustained, but on three separable lanes that this report fuses into one. (i) The
`allegiance` open-cardinality map has no BSL storage representation; its ratified destination is
Amendment AG attributed membership, whose accessor `membership-field-of` is Slice 4 and
Director-escalation-gated (evaluator.rs:511). (ii) `sqrt` is absent from `DECLARABLE_INTRINSICS`
(declarations.rs:110) and is required as a normalizer by `interest_fit`/`platform_vector` — a
mechanically-scoped intrinsic ask, NOT the same kind of blocker as (iii). (iii) The reused
acquiescence sigmoid (survival_calculus.py:43, twice per class-party pair) is an ADR172/173
PORT-QUESTION requiring a Director ruling on what H(c)'s counterfactual becomes once P(S|A) is an
emergent measure. The graph-scope-register half of the block is NARROWER than claimed: only
`popular_front`'s singleton fields need the carrier + `the` (Slice 2); the per-class and per-party
rows decompose onto ordinary nodes with landed Slice-1 heads, and are provably absent on 9 of 12
canonical scenarios. §5's channel table and §7's byte-gate paragraph require the re-read above.**
