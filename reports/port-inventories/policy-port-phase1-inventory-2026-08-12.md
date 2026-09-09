# PolicySystem Port — Phase-1 Inventory (2026-08-12)

PolicySystem (`src/babylon/engine/systems/policy.py`, 782 lines, position 17.47) is the
LEGISLATE resolver: it drains a FIFO policy agenda through a jurisdictional→judicial→fiscal
veto gauntlet, writes enacted overlays, runs a per-class delivery ledger that feeds next
tick's betrayal drift, and resolves the one-shot SYRIZA/Allende governance fork. Every
byte of its cross-tick state — five registers — lives in graph-level `dict` attributes
(`graph.graph[...]`/`set_graph_attr`), not on nodes; this is the system's dominant and
almost its only real port obstacle, everything else is plain, libm-free arithmetic. The
obstacle has a real, documented, partially-tested BSL answer (`docs/reference/bsl-language.rst`
§3.6/§4.7, "R9 chapter C3", Q6) — decompose each register onto ordinary per-entity node
fields — but the one accessor (`the`) needed for a genuinely-singleton register is itself
unevaluable today (`UNSERVED_EXPRESSION_HEADS` maps it to Slice 2), and two smaller but
real gaps compound it: one edge-attribute read and a full absence of any string-valued
field/value type in BSL for the system's several identity-carrying fields.

**Verdict:** PORTABLE WITH D-RECORDS for the resolver math and four of five registers
(content-modeling only, no unbuilt lane) — BLOCKED on Slice 2 for exactly two computations
(`_org_bridges`'s edge-attribute read; the `national_financial` singleton input via `the`)
and UNVERIFIED for agenda drain rates above 1 (the shipped default is 1; only one golden
scenario exercises a higher rate).

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/policy.py` | 782 | **The target.** `PolicySystem`, all 5 registers, all 15 computations below. |
| `src/babylon/domain/politics/policy.py` | 275 | `resolve_legislate` — the pure §2.4 gauntlet pipeline (preemption / judicial / funding-identity / capital-strike flag); `PolicyAgendaItem`, `FiscalTerrain`, `VetoGauntlet`, `PolicyResolution`, `PolicyResolutionKind` models; `policy_incidence`. |
| `src/babylon/domain/politics/governance_endgame.py` | 140 | Pure SYRIZA-fork math: `resolve_governance_arm`, `rupture_geometry`, `phi_share`, `betrayal_crossed`, `dual_power_live`. |
| `src/babylon/domain/economics/distribution/sovereign_fiscal.py` | 112 | `SovereignFiscalState` model, `sovereign_debt_service`, `bond_discipline_binds`, `finance_shortfall`, `borrow` — the debt-ledger half of L-CEILING. |
| `src/babylon/domain/economics/substrate/equalization.py` | 263 (only `equalization_deltas`, lines 46-91, is called) | The grain-agnostic `Δc = α(r−r̄)c` capital-migration law, reused at county grain for the capital-strike arm. `DefaultHexEqualizationComputer`/`_compute_capital_weighted_rates` (the rest of the file) are **not** in PolicySystem's call path. |
| `src/babylon/domain/economics/node_kinds.py` | 68 | `NodeKind`, `BoundaryEdgeKind` — the boundary-flow-register vocabulary `_record_funding_receipt`/`_split_delivery` use. |
| `src/babylon/domain/economics/boundary_flow_register.py` | 148 | `BoundaryFlowRegister.record` — the L-RECEIPTS append-only flow ledger `services.boundary_register` exposes. Not a graph write; a service-side buffer flushed into the per-tick transaction envelope elsewhere. |
| `src/babylon/formulas/politics.py` | 279 (only `sw_deliverable`/`delivery_gap`/`delivery_ratio` are imported by `domain/politics/policy.py`) | The three pure funding-identity formulas. The file's other functions (`valve_multiplier`, `hope_field`, `counterfactual_hope_gain`, `allegiance_drift`, `platform_vector`, `interest_fit`, `turnout_share`, `competitiveness`) are **not** used by PolicySystem — they belong to Allegiance/Electoral. |
| `src/babylon/formulas/constants.py` | 37 | `HOURS_PER_YEAR` (2080), `WEEKS_PER_YEAR` (52) — sourced from `GameDefines.timescale`, used in the Φ-inflow annualization. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._wrap_graph` (used, `policy.py:150`). `_read`/`_write_clamped`/`_publish`/`_get_persistent_data` are **not** used — PolicySystem reads node attributes with raw `.attributes.get(...)` and emits via its own `_emit` staticmethod, not `SystemBase._publish`. |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` — the system's declared partition. |
| `src/babylon/kernel/event_bus.py` | 288 (only `Event`, lines 33-56, is touched) | The frozen `Event` dataclass PolicySystem's `_emit` constructs and publishes. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` — `get_graph_attr`/`set_graph_attr` (lines 350-372, the register I/O), `query_nodes`/`query_edges` (258-298), `get_node`/`update_node` (77-98), `query_territory_claims` (415-432). |
| `src/babylon/engine/context.py` | 113 | `TickContext.tick` is the only field PolicySystem reads (`context.tick`, `step`'s third parameter); `displacement_mode`/`persistent_data` are untouched. |
| `src/babylon/engine/services.py` + `src/babylon/kernel/services.py` | 288 total (relevant slice only) | `ServicesProtocol`/`ServiceContainer` — `services.defines.politics`, `services.defines.survival.default_subsistence`, `services.event_bus`, `services.boundary_register` (`Any`, may be `None`). |
| `src/babylon/models/entities/institution.py` | (relevant: `InternalBalanceOfForces`, lines 46-71) | `liberal_technocratic` field domain — the judicial-bench weight `_gauntlet` reads. |
| `src/babylon/models/entities/organization.py` | (relevant: `institutional_pull` line 233, `acquired_doctrine_ids` line 207) | Field domains for the two Organization attributes PolicySystem reads. |
| `src/babylon/models/entities/relationship.py` | (relevant: `solidarity_strength` line 116) | `Coefficient`-typed SOLIDARITY edge field `_org_bridges` reads. |
| `src/babylon/models/enums/politics.py` | 49 | `PolicyAxis` — 7 members (`wage_floor`, `social_wage`, `labor_law`, `police_budget`, `border_regime`, `war_posture`, `trade_tariff`). |
| `src/babylon/models/enums/organizations.py` | (relevant: `ApparatusType.RSA_JUDICIAL`, line 221) | The judicial-bench discriminant `_gauntlet` filters `INSTITUTION` nodes by. |
| `src/babylon/models/enums/topology.py` | (relevant: `EdgeType.ADMINISTERS`=127, `EdgeType.SOLIDARITY`=100, `NodeType.TERRITORY`/`SOCIAL_CLASS`/`INSTITUTION`) | Edge/node type constants used in queries. |
| `src/babylon/models/enums/events.py` | (relevant: the 7 `EventType` members PolicySystem emits, lines 174-187) | `POLICY_ENACTED`, `POLICY_STRUCK`, `POLICY_PREEMPTED`, `CAPITAL_STRIKE`, `DELIVERY_GAP_CROSSED`, `BETRAYAL_INTEGRAL_CROSSED`, `GOVERNANCE_FORK_RESOLVED`. |
| `src/babylon/models/superstructure.py` | 49 | `SUPERSTRUCTURE_REGISTERS` — the 12-register vocabulary; 5 are PolicySystem's own (`policy_agenda`, `policy_overlays`, `sovereign_fiscal`, `policy_delivery`, `governance_endgame`). |
| `src/babylon/sentinels/superstructure/registry.py` | (relevant excerpt) | `SUPERSTRUCTURE_ATTR_OWNERS` — statically declares PolicySystem as the sole write site for its 5 registers; enforced by `tests/unit/sentinels/test_superstructure.py`. |
| `src/babylon/models/world_state.py` | (relevant: `superstructure_registers` field 600-620, `_harvest_superstructure_registers` 335-352, the `to_graph`/`from_graph` re-stamp 823-833) | The `WorldState`-level round-trip carrier: registers are ordinary graph attrs during a tick, and a named Pydantic field only across the `WorldState` round-trip. |
| `src/babylon/domain/economics/tick/graph_bridge.py` | (relevant: `NATIONAL_FINANCIAL_ATTR`, `write_national_financial_state_to_graph`, lines 430-450) | Writer of the `national_financial` graph attr `_interest_rate` reads — owned by TickDynamicsSystem @4.0, **not** PolicySystem; a genuine cross-system input, out of this system's own write scope. |
| `src/babylon/engine/systems/ooda.py` | (relevant: lines 404-423) | Calls `policy.enqueue_agenda_item` from the state-AI LEGISLATE dispatch @14 — the agenda's only OTHER writer, upstream of PolicySystem in the same tick. |
| `src/babylon/engine/systems/electoral.py` | (relevant: lines 870-917) | Downstream one-tick-stale reader of `governance_endgame` and `policy_delivery` — see §5. |
| `src/babylon/engine/systems/doctrine.py` | (relevant: lines 142-152, 583) | Downstream reader of `policy_delivery` (the `_delivery_gap` helper, feeding CLASS_ANALYSIS theory-rot). |
| `src/babylon/engine/systems/market_scissors.py` | (relevant: lines 485-522) | Downstream same-tick reader of `national_financial` (shared input) and of `tick_capital_stock` — the node attribute PolicySystem's capital-strike arm writes. |
| `src/babylon/domain/economics/trade_policy.py` | (relevant: `effective_trade`, `tariff_dampening`) | Reads `policy_overlays`' `trade_tariff` axis, but **only at scenario init/re-init**, never in-tick — out of scope for a tick-level port, cited for completeness. |
| `src/babylon/engine/scenarios/electoral_fixture.py` | 278 | `apply_political_terrain` — builds the SOVEREIGN/INSTITUTION/ADMINISTERS terrain the electoral goldens (and hence PolicySystem) exercise. |
| `src/babylon/engine/scenarios/electoral_goldens.py` | 543 | `create_mitterrand_scenario`/`create_syriza_scenario`/`create_bernie_valve_scenario` — the three golden factories that seed `policy_agenda`/`electoral_governments` and exercise PolicySystem live (§5). |

**Not exercised by policy.py at all:** no `src/babylon/topology/*` module beyond `GraphProtocol`
itself; no formula outside the three named above; no libm anywhere in the call graph (§4).

**Reference BSL/spec material read for this inventory** (all fully or substantially read):
- `docs/reference/bsl-language.rst` — §3.6 "graph-scope state" ruling (R9 chapter C3, lines
  2650-2688), §4.7 "cross-system registers and one-tick handoffs" (lines 3686-3721), and the
  conformance-vector plan's chapters C1/C3/C5/C8/C10 (lines 4150-4310).
- `rust/crates/babylon-graph/src/substrate.rs` (full) — the `GraphSubstrate` trait: confirms
  no graph-level attribute method exists anywhere, node attributes are `f64`-only, and there
  is no `edge_attribute` method.
- `rust/crates/babylon-bsl/src/evaluator.rs` (`Value` enum lines 53-105; `UNSERVED_EXPRESSION_HEADS`/
  `EVALUATOR_SERVED`/`SERVED_QUERY_HEADS` lines 486-527, 2256-2267) — the authoritative,
  currently-true evaluability split.
- `rust/crates/babylon-bsl/tests/r9_chapters.rs` (full module list; c1, c3, c5, c8, c10 read in
  detail) — the landed static-layer (grammar/manifest/cost) test suite for the not-yet-evaluable
  heads, and the one **evaluable** re-modeling precedent (D59, the FIFO-agenda pattern).
- `rust/crates/babylon-bsl/src/manifest.rs`, `src/declarations.rs` — carrier-manifest checks,
  `DECLARABLE_INTRINSICS`, the closed `deffield` type vocabulary (`int`/`bool`/`currency`/
  `probability`/`intensity`/`coefficient`/`enum` — no `string`).
- `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` — confirms
  `NodeType/ORGANIZATION` has a real landed precedent (Q1, "content-declared kinds").

## 2. COMPUTATION CATALOG (execution order)

### 0 — Empty-register byte-safety guard (`step`, policy.py:150-158)
- **(a)** If the graph carries neither an agenda nor a fiscal register, PolicySystem does
  nothing: zero reads of class/territory state, zero writes, zero events, every tick.
- **(b)** `if not agenda_raw and not fiscal_raw: return` (line 153, after `agenda_raw =
  wrapped.get_graph_attr(POLICY_AGENDA_ATTR, None)` and `fiscal_raw = wrapped.get_graph_attr(
  SOVEREIGN_FISCAL_ATTR, None)`).
- **(c) Reads:** graph attrs `policy_agenda`, `sovereign_fiscal` (presence only).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none. This is the documented reason all six base qa:regression scenarios
  (none of which carry either register) are byte-unchanged with the system live (policy.py:39-42).

### 1 — Fiscal-terrain gathering (`_fiscal_terrain`/`_claimed_territories`, policy.py:226-279)
- **(a)** Sum the enacting sovereign's live per-territory fiscal facts (tax claim, Φ-inflow,
  measured total surplus) over every territory where that sovereign is the top CLAIMS-holder,
  plus the interest rate and the carried debt stock.
- **(b)** `t_claim = Σ tick_taxes_on_surplus`; `total_surplus = Σ tick_total_surplus`;
  `phi_inflow = Σ tick_phi_hour · HOURS_PER_YEAR / WEEKS_PER_YEAR` (policy.py:258-265, the
  `2080/52` annualization); `phi_measured = True` iff ANY claimed territory carries a
  `tick_phi_hour` attribute at all (isinstance check, line 261-264 — presence, not value);
  `debt_stock = prior.debt_stock if prior is not None else 0.0` (line 271, `prior =
  fiscal.get(sovereign_id)`).
- **(c) Reads:** `TERRITORY.tick_taxes_on_surplus`, `.tick_total_surplus`, `.tick_phi_hour`
  (all default-0.0-via-`_numeric` except presence, absent-honestly, III.11); `query_territory_claims`
  per territory (top row only); `sovereign_fiscal[sovereign_id].debt_stock`; `national_financial`
  graph attr (`_interest_rate`, lines 281-294 — `raw.get("endogenous_interest", {}).get("rate")`,
  defensively `0.0` if the layer is absent).
- **(d) Writes:** none (pure gather).
- **(e) Defines:** none directly (the interest rate is data, not a define).
- **(f) Events:** none.

### 2 — Veto-gauntlet assembly (`_gauntlet`, policy.py:296-316)
- **(a)** Collect the sovereign's ADMINISTERS parent (if any — the apex has none) and every
  RSA_JUDICIAL institution's `liberal_technocratic` weight, sorted for determinism.
- **(b)** `parents = sorted(edge.source_id for edge in query_edges(ADMINISTERS) if
  edge.target_id == sovereign_id)`, take `parents[0]` (lines 298-302, 314); `benches` = every
  `INSTITUTION` node with `apparatus_type == RSA_JUDICIAL.value` whose `internal_balance` dict
  carries a numeric `liberal_technocratic` (lines 304-312), sorted by node id.
- **(c) Reads:** `EdgeType.ADMINISTERS` edges; `NodeType.INSTITUTION` nodes' `apparatus_type`,
  `internal_balance.liberal_technocratic`.
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### 3 — `resolve_legislate` arm 4: federal preemption (domain/politics/policy.py:192-218)
- **(a)** A periphery-mirror bar contraction, then: if a lower sovereign's overlay magnitude
  exceeds the (possibly contracted) preemption envelope, the item is nullified outright.
- **(b)** `peripheral = phi_measured and phi_share(phi_inflow, total_surplus) <
  periphery_phi_share_floor`; `bar = periphery_ceiling_factor if peripheral else 1.0`; if
  `administers_parent is not None and item.magnitude > preemption_envelope * bar`, return
  `PREEMPTED`.
- **(c) Reads:** `FiscalTerrain` fields, `VetoGauntlet.administers_parent`, `item.magnitude`.
- **(d) Writes:** none (pure).
- **(e) Defines:** `politics.periphery_phi_share_floor` (0.05, `[0,1]`), `politics.
  periphery_ceiling_factor` (0.25, `(0,1]`), `politics.preemption_envelope` (0.2, `[0,1]`) —
  `config/defines/politics.py:302-326,125-137`; defines.yaml:1109-1110,1094.
- **(f) Events:** none here (the caller emits `POLICY_PREEMPTED` — computation 6).

### 4 — `resolve_legislate` arm 3: judicial strike-down (domain/politics/policy.py:220-230)
- **(a)** The first (sorted) judicial bench whose class-balance-scaled tolerance the
  policy's incidence exceeds voids the overlay.
- **(b)** `incidence = policy_incidence(item, total_surplus)` (see computation 5's shared
  helper below); for each `(institution_id, liberal_weight)` in `judicial_benches`:
  `tolerance = judicial_tolerance_scale * liberal_weight * bar`; if `incidence > tolerance`,
  return `STRUCK`.
- **(c) Reads:** `judicial_tolerance_scale`, the gauntlet's benches, `incidence`, `bar`.
- **(d) Writes:** none.
- **(e) Defines:** `politics.judicial_tolerance_scale` (0.5, `≥0.0`) — defines.py:113-124,
  defines.yaml:1093.
- **(f) Events:** none here (`POLICY_STRUCK` — computation 6).

### 5 — `policy_incidence` + `resolve_legislate` arm 2: funding identity / bond discipline (domain/politics/policy.py:158-176, 232-264)
- **(a)** Policy incidence on measured surplus: for `social_wage`, the promise as a share of
  surplus (clamped to 1.0 against zero surplus); for the three regulatory-redistributive axes
  (`wage_floor`/`labor_law`/`trade_tariff`), the magnitude IS the incidence; for the three
  state-apparatus axes, zero. For a funded (`promised > 0`) item: fund it up to
  `t_claim + φ_share·Φ_inflow − debt_service`, finance the shortfall under bond discipline,
  and flag a capital strike if the incidence clears tolerance.
- **(b)** `policy_incidence`: `social_wage` → `min(1.0, promised/total_surplus)` if
  `total_surplus > 0` else `1.0` if `promised > 0` else `0.0`; regulatory axes → `item.magnitude`;
  else `0.0` (lines 158-176). `resolve_legislate` funded branch: `service =
  sovereign_debt_service(debt_stock, interest_rate)` (= `max(0,stock)*max(0,rate)`,
  sovereign_fiscal.py:48-56); `phi_slice = phi_social_share * phi_inflow`; `funded =
  sw_deliverable(promised, t_claim, phi_slice, service)` = `min(max(0,promised),
  max(0, t_claim+phi_slice−service))` (formulas/politics.py:75-91); `shortfall =
  delivery_gap(promised, funded)` = `max(0, promised−funded)`; `disciplined =
  bond_discipline_binds(service, t_claim, bond_discipline_threshold*bar)` — `True` if
  `service>0` and (`t_claim<=0` or `service/t_claim > threshold`); `borrowed =
  finance_shortfall(shortfall, debt_finance_share, disciplined)` = `0.0` if disciplined else
  `max(0,shortfall) * clamp(finance_share,0,1)`; `delivered = min(promised, funded+borrowed)`;
  `ratio = delivery_ratio(delivered, promised)` = `1.0` if `promised<=0` else
  `clamp(delivered/promised, 0, 1)`; `gap = delivery_gap(promised, delivered)`;
  `capital_strike = incidence > capital_tolerance*bar`. Unfunded (`promised == 0`) branch:
  `ENACTED` with only `incidence`/`capital_strike` set (lines 260-264).
- **(c) Reads:** `item.axis`, `.promised`, `.magnitude`; `terrain.{t_claim,phi_inflow,
  debt_stock,interest_rate,total_surplus}`.
- **(d) Writes:** none (pure — the caller applies the verdict, computations 8-13).
- **(e) Defines:** `politics.phi_social_share` (0.25, `[0,1]`), `politics.
  debt_finance_share` (0.5, `[0,1]`), `politics.bond_discipline_threshold` (0.25, `>0.0`),
  `politics.capital_tolerance` (0.15, `(0,1]`) — defines.py:62-112,50-61; defines.yaml:1089,
  1091-1092,1088.
- **(f) Events:** none here (computations 8/10/15).

### 6 — Preempted/struck dispatch (`_apply`, policy.py:337-366)
- **(a)** On a preempted or struck resolution, emit the terminal event and route into the
  governance fork with the corresponding `contact` tag, then stop — nothing else in `_apply`
  runs for this item.
- **(b)** `PREEMPTED` → `_emit(POLICY_PREEMPTED, {sovereign_id, policy_axis, preempting_sovereign})`
  then `_resolve_fork(..., "preempted", ...)`. `STRUCK` → `_emit(POLICY_STRUCK, {sovereign_id,
  policy_axis, striking_institution})` then `_resolve_fork(..., "struck", ...)`. Both `return`
  immediately after (lines 351, 366).
- **(c) Reads:** `resolution.kind`, `.preempting_sovereign`/`.striking_institution`.
- **(d) Writes:** none directly (the fork resolution, computation 12, may write `governance_endgame`).
- **(e) Defines:** none directly.
- **(f) Events:** `EventType.POLICY_PREEMPTED`, `EventType.POLICY_STRUCK`.

### 7 — Host-discipline clamp (`_apply`, policy.py:368-397)
- **(a)** An entryist org's enacted platform is clamped toward the standing overlay's median
  by a fixed share, and every subsequent delivered/gap/ratio/borrowed quantity for this item
  is rescaled by the same factor — capital's strike response is deliberately NOT rescaled
  (it reacts to the attempted platform, not the permitted fraction).
- **(b)** If `_is_host_disciplined(item)` (computation 9's own logic — see below):
  `standing = overlays.get(sovereign_id,{}).get(axis,{}).get("magnitude", 0.0)`;
  `enacted_magnitude = standing + (item.magnitude − standing) * (1.0 − host_discipline_clamp_share)`.
  `factor = enacted_magnitude / item.magnitude if item.magnitude > 0.0 else 1.0`; if
  `factor != 1.0`: `resolution = resolution.model_copy(update={delivered: resolution.delivered*
  factor, gap: promised−delivered, ratio: delivered/promised if promised>0 else 1.0, borrowed:
  resolution.borrowed*factor})` (lines 379-397).
- **(c) Reads:** `overlays[sovereign_id][axis]` (prior standing magnitude, possibly absent →
  `0.0`), `item.magnitude`, `resolution.{delivered,promised,borrowed}`.
- **(d) Writes:** none (rewrites the local `resolution`/`enacted_magnitude` bindings only;
  the actual overlay write is computation 8).
- **(e) Defines:** `politics.host_discipline_clamp_share` (0.5, `[0,1]`) —
  defines.py:400-413, defines.yaml:1118.
- **(f) Events:** none directly.

### 8 — Overlay write + POLICY_ENACTED emission (`_apply`, policy.py:399-415)
- **(a)** Stamp the enacted overlay for this (sovereign, axis) pair and emit the enactment
  event.
- **(b)** `overlays.setdefault(sovereign_id, {})[axis.value] = {magnitude: enacted_magnitude,
  enacted_tick: tick, promised: resolution.promised, delivered: resolution.delivered}`;
  `_emit(POLICY_ENACTED, {sovereign_id, policy_axis, magnitude, delivery_ratio: resolution.ratio})`.
- **(c) Reads:** `enacted_magnitude`, `tick`, `resolution.{promised,delivered,ratio}`.
- **(d) Writes:** `policy_overlays[sovereign_id][axis]` — 4 subfields.
- **(e) Defines:** none.
- **(f) Events:** `EventType.POLICY_ENACTED`.

### 9 — Host-discipline eligibility test (`_is_host_disciplined`, policy.py:676-694)
- **(a)** True iff the drafting org holds the "entryism" doctrine stance and has not been
  derecognized by the host machine.
- **(b)** `if not item.source_org_id: return False`; `org = graph.get_node(source_org_id)`;
  `if org is None: return False`; `if "entryism" not in tuple(org.attributes.get(
  "acquired_doctrine_ids") or ()): return False`; `derecognized = frozenset(graph.get_graph_attr(
  "electoral_derecognized", ()) or ())`; `return source_org_id not in derecognized`.
- **(c) Reads:** `ORGANIZATION.acquired_doctrine_ids` (tuple[str,...]); graph attr
  `electoral_derecognized` (owned by ElectoralSystem, read here one tick stale by pipeline
  position since ElectoralSystem @17.45 < PolicySystem @17.47).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### 10 — Deficit-financing borrow (`_apply`, policy.py:416-420)
- **(a)** If this enactment borrowed, advance the sovereign's fiscal ledger by the borrowed
  principal.
- **(b)** `if resolution.borrowed > 0.0: prior = fiscal.get(sovereign_id,
  SovereignFiscalState(sovereign_id=sovereign_id)); fiscal[sovereign_id] = borrow(prior,
  resolution.borrowed)` where `borrow` = `SovereignFiscalState(debt_stock = prior.debt_stock +
  max(0,borrowed), last_borrowed = max(0,borrowed))` (sovereign_fiscal.py:91-103).
- **(c) Reads:** `resolution.borrowed`, `fiscal[sovereign_id]` (or a fresh zero state).
- **(d) Writes:** `sovereign_fiscal[sovereign_id]` — `debt_stock`, `last_borrowed`.
- **(e) Defines:** none directly (the borrowed amount itself came from computation 5's
  `debt_finance_share`).
- **(f) Events:** none.

### 11 — L-RECEIPTS funding-receipt row (`_record_funding_receipt`, policy.py:540-577)
- **(a)** Record the Φ slice this enactment actually drew from the tribute pool (the domestic
  tax claim, net of debt service, is charged first; whatever remains is charged to Φ) — a
  no-op unless a live boundary-flow-register session is bound.
- **(b)** `register = getattr(services, "boundary_register", None); if register is None or
  register.session_id is None: return`; `service = sovereign_debt_service(debt_stock,
  interest_rate)`; `net_domestic = max(0.0, t_claim − service)`; `funded = delivered −
  borrowed`; `phi_consumed = max(0.0, funded − net_domestic)`; `if phi_consumed <= 0.0: return`;
  `register.record(session_id, tick, source="USA"/NATIONAL, dest=sovereign_id/SOVEREIGN,
  flow_type=FISCAL_FUNDING, magnitude=phi_consumed)`.
- **(c) Reads:** `terrain.{debt_stock,interest_rate,t_claim}`, `resolution.{delivered,borrowed}`,
  `services.boundary_register.session_id`.
- **(d) Writes:** none on the graph — appends one row to `services.boundary_register`'s
  in-memory buffer (a service-side ledger, not `set_graph_attr`).
- **(e) Defines:** none.
- **(f) Events:** none (this is a ledger row, not an `EventType`).

### 12 — Per-class delivery split (`_split_delivery`, policy.py:579-673)
- **(a)** Split the item's promise/delivery/gap across every active social class by its
  subsistence-threshold weight, accumulate each class's betrayal integral, emit a
  DELIVERY_GAP_CROSSED row per class with positive gap, and fire BETRAYAL_INTEGRAL_CROSSED
  exactly once when a class's integral first crosses the threshold.
- **(b)** `classes = sorted(SOCIAL_CLASS nodes where attrs.get("active", True))`; `weights[c] =
  attrs.get("subsistence_threshold", default_subsistence)`; `total_weight = Σweights`; if
  `total_weight <= 0.0: return` (no-op — an honest-absence guard, not a fabricated equal
  split). For each class: `share = weights[c]/total_weight`; `promised_c = promised*share`;
  `delivered_c = delivered*share`; `gap_c = gap*share`; if `delivered_c > 0` and a register is
  bound: `register.record(source=sovereign_id/SOVEREIGN, dest=class_id/SOCIAL_CLASS,
  flow_type=SOCIAL_WAGE, magnitude=delivered_c)`; `integral = prior_integral + gap_c`;
  `delivery[class_id] = {incumbent_id, promised_c, delivered_c, gap_c, integral, tick}`; if
  `gap_c > 0`: emit `DELIVERY_GAP_CROSSED`; if `betrayal_crossed(integral, threshold) and not
  betrayal_crossed(prior_integral, threshold)`: emit `BETRAYAL_INTEGRAL_CROSSED` (the edge
  trigger — fires exactly once per class, patience never resets, lines 657-673).
  `betrayal_crossed(x,t) = t>0.0 and x>=t` (governance_endgame.py:70-78).
- **(c) Reads:** `SOCIAL_CLASS.active` (default `True`), `.subsistence_threshold` (default
  `services.defines.survival.default_subsistence`); `delivery[class_id].integral` (prior tick);
  `_incumbent(sovereign_id)` (computation 14's helper).
- **(d) Writes:** `policy_delivery[class_id]` — `incumbent_id`, `promised`, `delivered`, `gap`,
  `integral`, `tick`, for every active class every time ANY item's `promised > 0` executes.
- **(e) Defines:** `survival.default_subsistence` (0.3, `[0,1]`) — `config/defines/survival.py:
  23-27`, defines.yaml:165; `politics.betrayal_threshold` (1.0, `>0.0`) —
  defines.py:281-289, defines.yaml:1107.
- **(f) Events:** `EventType.DELIVERY_GAP_CROSSED` (per class with `gap_c>0`),
  `EventType.BETRAYAL_INTEGRAL_CROSSED` (edge-triggered).

### 13 — Governance-endgame fork resolution (`_resolve_fork`, policy.py:455-526)
- **(a)** On first fiscal-ceiling contact by a SEATED governing party, resolve once — forever
  — whether it capitulates or ruptures, and (on rupture) which geometry it meets. A no-op if
  there is no seated incumbent, the sovereign governs itself, or the org already has a
  resolution (the register is absorbing).
- **(b)** `incumbent = _incumbent(sovereign_id)` (reads graph attr `electoral_governments`,
  one tick stale, defaulting to `sovereign_id` itself — policy.py:697-714); `if incumbent ==
  sovereign_id or incumbent in governance: return`; `org = graph.get_node(incumbent); if org is
  None: return`; `claims = {territory_id: query_territory_claims(territory_id) for
  territory_id in claimed_ids}`; `arm = resolve_governance_arm(institutional_pull=org.
  institutional_pull, capture_threshold=governance_capture_threshold, organs_live=
  dual_power_live(claims))` = `RUPTURE if organs_live and institutional_pull <
  capture_threshold else CAPITULATE` (governance_endgame.py:99-116); if `arm is RUPTURE`:
  `starved = phi_measured and phi_share(phi_inflow,total_surplus) < periphery_phi_share_floor`;
  `geometry = rupture_geometry(bridges_present=_org_bridges(incumbent), phi_starved=starved)`
  = `SYNTHESIS if bridges_present and phi_starved else ALLENDE` (governance_endgame.py:119-129).
  `governance[incumbent] = {sovereign_id, opened_tick: tick, arm, geometry, contact}`; emit
  `GOVERNANCE_FORK_RESOLVED`. `_org_bridges` (policy.py:528-538): iterate `EdgeType.SOLIDARITY`
  edges incident to the org, `True` on the first with `attributes.get("solidarity_strength",
  0.0) > 0.0`.
- **(c) Reads:** `electoral_governments` graph attr; `ORGANIZATION.institutional_pull`;
  `query_territory_claims` per claimed territory; **`SOLIDARITY.solidarity_strength` — an
  EDGE attribute** (`_org_bridges`); `terrain.{phi_measured,phi_inflow,total_surplus}`;
  `governance` (absorbing-register membership test).
- **(d) Writes:** `governance_endgame[incumbent]` — `sovereign_id`, `opened_tick`, `arm`,
  `geometry`, `contact`. Written exactly once per org, ever (the caller's `incumbent in
  governance` guard).
- **(e) Defines:** `politics.governance_capture_threshold` (0.5, `[0,1]`) — defines.py:
  290-301, defines.yaml:1108; `politics.periphery_phi_share_floor` (as computation 3).
- **(f) Events:** `EventType.GOVERNANCE_FORK_RESOLVED`.

### 14 — Capital-strike application (`_apply_capital_strike`, policy.py:716-762)
- **(a)** Arm 1 of the gauntlet: the incidence enters the enacting sovereign's CLAIMED
  counties' profit rates as a penalty; the equalization operator migrates capital toward
  unpenalized geographies. With every capital-bearing territory claimed, the gradient is
  uniform and nothing moves (a national policy cannot be fled domestically).
- **(b)** `capitals[t] = attrs.tick_capital_stock`, `rates[t] = attrs.tick_profit_rate −
  (incidence if t in claimed else 0.0)`, for every TERRITORY with both attributes numeric AND
  `capital > 0.0` (lines 738-748, `if not capitals: return 0.0`). `deltas =
  equalization_deltas(capitals, rates, strike_equalization_rate)` — the grain-agnostic law:
  `r_avg = Σ(rate·capital)/Σcapital` (capital-weighted, `0.0` if `Σcapital<=0`); `proposed[t] =
  alpha*(rate[t]−r_avg)*capital[t]`; non-negativity enforced by ONE proportional `scale` factor
  computed over every negative proposed delta (`scale = min(c_i/(-delta_i))`, or the whole
  result zeroes if any zero-capital unit proposes a negative delta) — `equalization.py:69-91`.
  For each `territory_id, delta`: if `delta != 0.0`: `update_node(territory_id,
  tick_capital_stock = capitals[territory_id] + delta)`; `outflow += -delta` for claimed
  territories with `delta < 0.0`.
- **(c) Reads:** `TERRITORY.tick_capital_stock`, `.tick_profit_rate` (both presence-gated,
  `isinstance` numeric check, honest absence).
- **(d) Writes:** `TERRITORY.tick_capital_stock` (every territory with a nonzero delta —
  claimed AND unclaimed; this is a **cross-node write across the whole national territory
  set**, not scoped to the enacting sovereign's claims).
- **(e) Defines:** `politics.strike_equalization_rate` (0.05, `[0,1]`) — defines.py:152-165,
  defines.yaml:1096.
- **(f) Events:** `EventType.CAPITAL_STRIKE` (emitted by the caller, `_apply`, lines 430-442,
  with payload `{sovereign_id, incidence, tolerance, outflow}`).

### 15 — Register write-back (`step`, policy.py:206-220)
- **(a)** Persist the drained agenda's remainder and every register that has been populated
  this tick — each register only if it is non-empty (or, for the agenda, if it existed at all
  this tick), preserving honest absence for registers a run never touches.
- **(b)** `if agenda_raw is not None or remaining: set_graph_attr(policy_agenda,
  [item.model_dump() for item in remaining])`; `if fiscal: set_graph_attr(sovereign_fiscal, ...)`;
  `if overlays: set_graph_attr(policy_overlays, overlays)`; `if delivery: set_graph_attr(
  policy_delivery, delivery)`; `if governance: set_graph_attr(governance_endgame, governance)`.
- **(c) Reads:** the five local dict/list bindings accumulated across the tick.
- **(d) Writes:** `policy_agenda`, `sovereign_fiscal`, `policy_overlays`, `policy_delivery`,
  `governance_endgame` (each conditionally).
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: 7 distinct `EventType` values** — `POLICY_PREEMPTED`,
`POLICY_STRUCK`, `POLICY_ENACTED`, `CAPITAL_STRIKE`, `DELIVERY_GAP_CROSSED`,
`BETRAYAL_INTEGRAL_CROSSED`, `GOVERNANCE_FORK_RESOLVED` (grep-confirmed, `events.py:174-187`).
Per the CURRENT BSL surface, `TickReport` carries no event log at all (confirmed by direct
read, `rust/crates/babylon-tick/src/lib.rs:29-48` — `before`/`after`/`fired`/`per_rule_fired`
only) — every emission here is a WS1 (#502) ledger row, unpinnable by any current golden.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, same as every prior port inventory in this batch):
`BabylonGraph.update_node`/`set_graph_attr` are plain dict merges with no type coercion or
quantization; all in-tick arithmetic below is raw Python `float`/`int`/`bool`/`str`/`dict`,
never the Pydantic-validated grid-quantized shape.

| Attribute | Node/scope | Python type | Domain | Category |
|---|---|---|---|---|
| `axis` | (PolicyAgendaItem field) | `PolicyAxis` (StrEnum, 7 members) | closed set | **Enum discriminant** |
| `magnitude` | (PolicyAgendaItem field) | `float` | `[0.0, 1.0]` | unit-interval |
| `promised` | (PolicyAgendaItem field) | `float` (NOT `Currency`-annotated) | `[0.0, ∞)` | **unbounded real, money-semantic by meaning, plain-float by type** |
| `drafted_tick` | (PolicyAgendaItem field) | `int` | `≥0` | integer |
| `sovereign_id`, `source_org_id` | (PolicyAgendaItem field) | `str` | free-form node-id string | **string identity — no BSL analog** |
| `tick_taxes_on_surplus`, `tick_total_surplus`, `tick_phi_hour` | TERRITORY | `float` (0.0 default via `_numeric`) | `[0,∞)` typically | unbounded real, money/labor-semantic; written by TickDynamicsSystem @4.0 (`graph_bridge.py`), see the sibling `tick-dynamics-port-phase1-inventory-2026-08-12.md` |
| `tick_capital_stock`, `tick_profit_rate` | TERRITORY | `float` (presence-gated) | `[0,∞)` / real | unbounded real / real; read AND written here |
| `national_financial` | **graph-scope** | nested `dict` (`{"endogenous_interest": {"rate": float}}`) | — | **graph-level opaque dict, genuinely one-per-graph** |
| `policy_agenda` | **graph-scope** | `list[dict]` (each a `PolicyAgendaItem.model_dump()`) | variable length, FIFO order by `drafted_tick` | **graph-level ordered collection of records — no BSL analog** |
| `policy_overlays` | **graph-scope** | `dict[str, dict[str, dict]]` — sovereign→axis→`{magnitude, enacted_tick, promised, delivered}` | — | **graph-level, per-(sovereign,axis)-keyed nested dict** |
| `sovereign_fiscal` | **graph-scope** | `dict[str, dict]` — sovereign→`{debt_stock, last_borrowed}` | `debt_stock≥0`, `last_borrowed≥0` | **graph-level, per-sovereign-keyed dict — but each value is genuinely 2 scalars** |
| `policy_delivery` | **graph-scope** | `dict[str, dict]` — class→`{incumbent_id(str), promised, delivered, gap, integral, tick(int)}` | promised/delivered/gap/integral `≥0` | **graph-level, per-class-keyed dict, one field (`incumbent_id`) itself a string identity** |
| `governance_endgame` | **graph-scope** | `dict[str, dict]` — org→`{sovereign_id(str), opened_tick(int), arm(2-valued str), geometry(2-valued-or-empty str), contact(4-valued str)}` | absorbing (write-once) | **graph-level, per-org-keyed dict, three string/enum-ish fields** |
| `under_eviction` … n/a | — | — | — | (not touched by this system) |
| `active` | SOCIAL_CLASS | `bool` (default `True`) | `{T,F}` | boolean |
| `subsistence_threshold` | SOCIAL_CLASS | `float` (default `services.defines.survival.default_subsistence`) | `[0,1]` | unit-interval |
| `apparatus_type` | INSTITUTION | `ApparatusType` (StrEnum) compared via `.value` string equality against `RSA_JUDICIAL` | closed set | **Enum discriminant (read as raw string equality, not a typed enum compare)** |
| `internal_balance.liberal_technocratic` | INSTITUTION | `float`, nested one level inside a `dict`-typed attribute | `[0.0,1.0]` (`InternalBalanceOfForces`'s own Pydantic bound; unenforced on the raw graph dict) | real, unit-interval — **nested-in-attribute, not a flat node field** |
| `institutional_pull` | ORGANIZATION | `float` (default 0.0 via `_numeric`) | `[0.0,1.0]` | unit-interval |
| `acquired_doctrine_ids` | ORGANIZATION | `tuple[str, ...]` — tested for membership (`"entryism" in ...`) | open-ended, doctrine-tree-authored id strings | **string SET — no BSL analog, and the vocabulary itself is content-authored (open, not closed)** |
| `solidarity_strength` | **SOLIDARITY edge** | `Coefficient` (`Annotated[float,...]`) | `[0,1]` typically | real — **EDGE attribute, read via `edge.attributes.get(...)`** |
| `electoral_governments`, `electoral_derecognized` | **graph-scope** (foreign, owned by ElectoralSystem) | `dict`/`frozenset`-from-tuple | — | graph-level, read-only here, one tick stale |
| `politics.*` defines (18 distinct fields touched, see §2 (e) rows) | — | `float`/`int` | see §2 | coefficients, one `int` (`policy_agenda_rate`, `≥1`, **no declared upper bound**) |
| `survival.default_subsistence` | — | `float` | `[0,1]` | coefficient |
| `HOURS_PER_YEAR`, `WEEKS_PER_YEAR` | — | `int` (2080, 52) | `≥1` | integer physical constants |

**Enum discriminant flag — same class as Territory's/every prior report's finding.**
`deffield`'s closed vocabulary (`int`/`bool`/`currency`/`probability`/`intensity`/
`coefficient`/`enum`) does have an `enum` row today (landed, ADR195/ADR196, org-foundation
train) — `PolicyAxis` (7 members), `PolicyResolutionKind` (3, resolver-internal only, never
stored), `GovernanceArm` (2), `RuptureGeometry` (2, or the empty-string "unresolved" third
state on non-rupture arms — a genuine 3-state field encoded as a 2-member enum plus an
absence convention) all fit the landed `deffield ... enum` mechanism directly (§3.1,
`declarations.rs:648-663`).

**String-identity flag — a genuine, previously-unnamed-for-this-system gap, verified at the
`Value`-enum level.** `rust/crates/babylon-bsl/src/evaluator.rs`'s `Value` enum (lines 53-105)
has exactly seven variants — `Int`, `Currency`, `Real`, `Ratio`, `Bool`, `Enum{closed}`,
`NodeRef`, `HyperedgeRef` — **no `String` variant anywhere**, and `deffield`'s type
vocabulary has no `string` row either. A landed test proves this is enforced, not merely
undeclared: `r9_chapters.rs:2216-2229`, `a_string_literal_in_an_emit_payload_is_e_parse_010`
(D75) — a string literal in an `emit` payload is rejected at PARSE time (`E-PARSE-010`), while
the identical rule with a numeric payload accepts. PolicySystem's `sovereign_id`,
`source_org_id`, `preempting_sovereign`, `striking_institution`, `incumbent_id`, and
`acquired_doctrine_ids`' membership test are all plain Python `str`/`tuple[str,...]` — none
has a BSL-storable representation as a field VALUE. The only route is identity-as-structure:
a `NodeRef` reached via a query, or a relationship expressed as an edge. This is a genuine,
system-wide content-modeling burden (6+ distinct fields), not a single D-record — see §6.

## 4. FLOAT-OP INVENTORY

Every arithmetic operation in PolicySystem's own call graph is binary64 (`float`) or `int`.
**Grep-confirmed zero `exp`/`log`/`sigmoid`/`pow`/`math.*` calls** in `policy.py`,
`domain/politics/policy.py`, `domain/politics/governance_endgame.py`,
`domain/economics/distribution/sovereign_fiscal.py`, and `formulas/politics.py`'s three
imported functions. `domain/economics/substrate/equalization.py` DOES import `math`
(`math.ldexp`, `math.isfinite`), but both live in `_compute_capital_weighted_rates` — the
function PolicySystem's call path never reaches (only the libm-free `equalization_deltas`,
lines 46-91, is called). **This system has zero libm-nondeterminism hazard**, matching
Territory and unlike Metabolism/Allegiance (whose `calculate_acquiescence_probability`
sigmoid, imported by `formulas/politics.py`'s OTHER functions, is never touched by
PolicySystem).

Shapes, in execution order:

1. **Loop-bound derivation:** `rate = max(1, int(defines.policy_agenda_rate))` (policy.py:185)
   — `policy_agenda_rate` is already Pydantic-`int`-typed (`ge=1`), so this cast is
   type-narrowing, not a Real→Int demotion of computed data. **No declared upper bound** on
   the define itself, though — the loop's static bound is whatever the scenario's defines
   override sets (1 in every canonical default; 24 in the `mitterrand` golden's calibration
   override, `tools/regression_scenarios.py:83`).
2. **Annualization multiply-divide:** `phi_raw * HOURS_PER_YEAR / WEEKS_PER_YEAR` (policy.py:265)
   — `float * int / int`, one multiply one divide, no libm.
3. **Bare-literal-heavy min/max clamps:** `min(1.0, item.promised/total_surplus)`
   (domain/policy.py:172), `max(0.0, t_claim + phi_slice − service)` (`sw_deliverable`,
   formulas/politics.py:90), `min(1.0, max(0.0, delivered/promised))` (`delivery_ratio`,
   formulas/politics.py:101), `max(0.0, promised − delivered)` (`delivery_gap`,
   formulas/politics.py:112), `max(0.0, shortfall) * min(max(finance_share,0.0),1.0)`
   (`finance_shortfall`, sovereign_fiscal.py:88), `max(0.0, debt_stock) * max(0.0,
   interest_rate)` (`sovereign_debt_service`, sovereign_fiscal.py:56) — **every bare `0.0`/
   `1.0` literal here is the same BSL-parser hazard every prior report in this batch has
   flagged**: the "no bare non-integer literal" rule means each needs a `c`-suffixed
   `defconst`/currency-suffixed literal or the Real-zero-promotion idiom.
4. **Periphery-bar contraction, three uses:** `bar = periphery_ceiling_factor if peripheral
   else 1.0`, then `preemption_envelope * bar`, `judicial_tolerance_scale * liberal_weight *
   bar`, `bond_discipline_threshold * bar`, `capital_tolerance * bar` — five distinct
   multiplies sharing one conditionally-selected coefficient (domain/policy.py:206,210,223,
   245,257).
5. **Threshold comparisons (no libm):** `item.magnitude > preemption_envelope*bar`,
   `incidence > tolerance`, `service/t_claim > threshold` (guarded `t_claim<=0` first —
   division-by-zero avoided by branching, not by an epsilon), `incidence > capital_tolerance*bar`.
6. **Host-discipline clamp — the same shape as arm 4's contraction, one level up:**
   `enacted_magnitude = standing + (item.magnitude − standing) * (1.0 − host_discipline_clamp_share)`
   (policy.py:384-386) — a linear interpolation with a bare `1.0`. `factor =
   enacted_magnitude/item.magnitude if item.magnitude>0.0 else 1.0` (line 387) — another bare
   `1.0`, and a division guarded by the SAME branch-not-epsilon convention.
7. **Proportional rescale of a whole resolution (3 multiplies, 1 divide, 1 subtract):**
   `delivered*factor`, `promised−delivered`, `delivered/promised if promised>0.0 else 1.0`,
   `borrowed*factor` (policy.py:389-396).
8. **Weighted split, per class (3 multiplies, running sums):** `share = weight/total_weight`;
   `promised*share`, `delivered*share`, `gap*share` (policy.py:618-621) — `total_weight` itself
   a plain `Σ` over a `dict.values()` (sorted-node-iteration order, III.7-safe).
9. **Betrayal-integral accumulation:** `integral = prior_integral + gap_c` (policy.py:636) —
   one add, monotone non-decreasing by construction (`gap_c >= 0` always, from `delivery_gap`'s
   own `max(0, ...)`).
10. **Real→Int storage, none found as a truncating demotion.** Every `int(...)` call in
    PolicySystem's own file is either a define-narrowing cast (item 1 above) or a `dict.get`
    default cast (`int(row.get("opened_tick", -2))`, `electoral.py:876` — a DOWNSTREAM
    consumer, not PolicySystem itself). Unlike Territory's displacement/decay `int(x*rate)`
    truncations, **PolicySystem has no computed-Real→Int demotion of its own** — a genuinely
    cleaner float-op surface than the prior port targets in this batch.
11. **Equalization law (the reused `equalization_deltas`, equalization.py:69-91):**
    `r_avg = Σ(rate_i*capital_i) / Σcapital_i` (guarded `c_total>0` else `0.0`);
    `proposed_i = alpha*(rate_i − r_avg)*capital_i`; the non-negativity `scale` pass: `cap =
    c_i/(-delta)` per negative proposed delta, `scale = min(scale, cap)`, then `scale*delta`
    for every unit — a **division inside a loop over a `dict`, order-dependent only in that
    the caller's `capitals`/`rates` dicts must already be deterministically ordered** (the
    function's own docstring says so explicitly, equalization.py:62-63); PolicySystem supplies
    them via `sorted(graph.query_nodes(...), key=lambda n: n.id)` (policy.py:738), so this is
    satisfied.
12. **Clamp implementation — none in PolicySystem's own file.** Unlike Territory (two
    inconsistent clamp shapes for the same `heat` field), PolicySystem never calls
    `SystemBase._write_clamped` and has no hand-written `min`/`max` clamp on a single
    conceptually-repeated field — every clamp above (item 3) belongs to a distinct quantity,
    so there is no cross-clamp inconsistency to transcribe.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 17.47** (`class PolicySystem`, `partition = CONSEQUENCE`, `position = 17.47`,
  policy.py:135-136), confirmed against `_SYSTEM_CLASSES` (`simulation_engine.py:45,56,62,65,
  70,351-355`): `... → AllegianceSystem (17.42) → ElectoralSystem (17.45) → PolicySystem
  (17.47) → SovereigntySystem (17.5) → MarketScissorsSystem (17.8) → ...`.
- **Reads from a same-tick prior system:** `TERRITORY.tick_taxes_on_surplus`/
  `.tick_total_surplus`/`.tick_phi_hour`/`.tick_capital_stock`/`.tick_profit_rate` — all
  written by **TickDynamicsSystem @4.0** (`domain/economics/tick/graph_bridge.py`, confirmed
  by the sibling `tick-dynamics-port-phase1-inventory-2026-08-12.md`, its own §2/§3 rows for
  these exact attributes); `national_financial` graph attr — also TickDynamicsSystem @4.0
  (`graph_bridge.py:433-450`). `electoral_governments`/`electoral_derecognized` graph attrs —
  written by **ElectoralSystem @17.45**, same tick, immediately prior position (`electoral.py`,
  `ELECTORAL_GOVERNMENTS_ATTR`/`ELECTORAL_DERECOGNIZED_ATTR` at lines 103/124) — these are
  genuinely same-tick-fresh reads (ElectoralSystem runs first).
- **Reads from a prior tick (I-ORD, cross-position lag):** `policy_agenda` — written earlier
  in the SAME tick by `enqueue_agenda_item` if the OODA dispatch @14 (`ooda.py:404-423`)
  selected LEGISLATE this tick, but the register PERSISTS across ticks (a FIFO, not a
  per-tick buffer), so most of what PolicySystem drains was enqueued on a PRIOR tick.
- **Writes consumed later this tick:** `TERRITORY.tick_capital_stock` (capital-strike arm) —
  read by **MarketScissorsSystem @17.8**, same tick, immediately-later position
  (`market_scissors.py:498-522`, confirmed: `"Exact Sum(numerator)/Sum(tick_capital_stock)
  over active territories"`) — this is the ONE genuinely same-tick downstream channel
  PolicySystem produces.
- **Writes consumed one tick later (I-ORD):**
  - `governance_endgame` — read by **ElectoralSystem @17.45** next tick
    (`_consume_governance_endgame`, `electoral.py:849-897`), gated on `tick == opened_tick+1`
    (punctuality — fires exactly once, confirmed by direct read); on `RUPTURE`/`ALLENDE` it
    deletes the seated government row and suspends the electoral clock; on `RUPTURE`/
    `SYNTHESIS` it emits `HOPE_SPIKE` for every bridged class.
  - `policy_delivery` — read by **ElectoralSystem @17.45** next tick (`_open_betrayal_windows`,
    `electoral.py:917`) AND by **DoctrineSystem @14.7** next tick (`_delivery_gap`,
    `doctrine.py:142-152,207,583` — feeds `CLASS_ANALYSIS` theory-rot via
    `reformist_theory_decay`).
  - `policy_overlays` — grep-confirmed **read by no in-tick System** (`rg` across
    `engine/systems/*.py` returns zero hits outside `policy.py` itself). The only OTHER
    reader anywhere is `domain/economics/trade_policy.py`'s `effective_trade` (the
    `trade_tariff` axis), and that module's own docstring states it runs **only at scenario
    init/re-init, never in-tick** — a real consumer, but not a tick-pipeline channel.
  - `sovereign_fiscal` — grep-confirmed **read by no other System at all**; it is
    self-referential (PolicySystem reads its own prior-tick write back via `_fiscal_terrain`'s
    `fiscal.get(sovereign_id)`, computation 1).
- **Context/service usage with no BSL equivalent:**
  - All five owned registers (`policy_agenda`, `policy_overlays`, `sovereign_fiscal`,
    `policy_delivery`, `governance_endgame`) plus the two foreign registers read
    (`electoral_governments`, `electoral_derecognized`) plus `national_financial` — **eight
    distinct `graph.graph[...]`/`get_graph_attr`/`set_graph_attr` names touched by this one
    system**. `sentinels/superstructure/registry.py:30-56` independently confirms PolicySystem
    is the sole write site (`SUPERSTRUCTURE_ATTR_OWNERS`) for its five, enforced by
    `tests/unit/sentinels/test_superstructure.py`.
  - `services.boundary_register` (`Any`-typed, `ServiceContainer.boundary_register`,
    default `None`) — a service-side append-only ledger (`BoundaryFlowRegister`), not a graph
    write at all; two record sites (computations 11, 12) guarded by `session_id is not None`,
    a clean no-op in every unit test and every qa:regression scenario that never binds a
    session.
- **DORMANCY on canonical scenarios — corrected against `tools/regression_scenarios.py`.**
  PolicySystem is **NOT structurally dormant** — unlike Territory's spillover/sink-routing
  phases, it is genuinely, deliberately exercised by three of the five "electoral golden"
  scenarios (`tools/regression_scenarios.py:2270-2338,2547-2553`, ADR140):
  - `mitterrand` — `capital_strike` event, 24 enactments past the periphery-contracted
    capital tolerance (the calibration burst).
  - `syriza` — `governance_fork_resolved` event, CAPITULATE with dual-power organs live
    (capture dominates).
  - `bernie_valve` — `delivery_gap_crossed` event, the seated reform's ledger accruing gap.

  These three factories (`src/babylon/engine/scenarios/electoral_goldens.py`, read in full)
  seed real terrain via `apply_political_terrain` (`electoral_fixture.py`, read in full): the
  **first production `Sovereign` node anywhere** (`SOV_MI_STATE`, via `apply_balkanization_seed`
  and directly), the **first production `Institution` node anywhere** (`INST_FED_JUDICIARY`,
  `RSA_JUDICIAL`, `liberal_technocratic=0.6`), the **first `ADMINISTERS` edge ever built**
  (`SOV_USA_FED → SOV_MI_STATE`), a seeded `policy_agenda` (6-24 items depending on scenario),
  and a seeded `electoral_governments` (a socdem party seated at tick 0). This exercises
  computations 0-3, 5, 6, 7 (mitterrand only, via its "entryism" doctrine stamp — wait,
  mitterrand seeds no entryism stance; `bernie_valve` does, via `_with_doctrine(_SOCDEM,
  "entryism")`), 8, 10, 12, 13, and 14 live. **What is NOT exercised anywhere in the canonical
  estate:** the judicial strike-down arm specifically firing `STRUCK` (all three goldens sit
  under the judicial bar by design, per the mitterrand docstring: "under the judicial bar");
  a `PREEMPTED` verdict (only `syriza` keeps the ADMINISTERS edge with a live claim, and its
  own docstring says the fork resolves via fiscal contact, not preemption); the `RUPTURE`
  arm of the governance fork at all (all three named goldens document a `CAPITULATE`
  resolution — `syriza`'s own docstring: "the fork must still resolve CAPITULATE: capture
  dominates organs"); `_org_bridges` returning `True` (no golden's incumbent org carries a
  live positive-strength SOLIDARITY edge in the scenarios read). **This is a materially
  different dormancy picture than the base six qa:regression scenarios** (which the
  `tools/regression_scenarios.py:2778,2843` declared-gap rows correctly mark as seeding no
  `SOVEREIGN` nodes at all) — the base six and the electoral five are disjoint families;
  PolicySystem's conformance oracle is real but lives entirely in the latter.
- **Baseline oracle:** `tests/unit/engine/systems/test_electoral_goldens.py` (311 lines) drives
  all five goldens through the real `simulation_engine.step()` loop and pins named events —
  this is the strongest conformance-oracle candidate in the whole system (§7).

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface as stated in the brief, verified independently
against `rust/crates/babylon-graph/src/substrate.rs` (full read), `babylon-bsl/src/evaluator.rs`
(the `Value` enum and the `EVALUATOR_SERVED`/`SERVED_QUERY_HEADS`/`UNSERVED_EXPRESSION_HEADS`
tables, lines 53-105 and 486-527), and `babylon-bsl/tests/r9_chapters.rs` (the landed
static-layer test suite). **The dominant, cross-cutting finding:** every one of PolicySystem's
five owned registers is graph-scope state (`graph.graph[...]`), which `GraphSubstrate` has NO
storage primitive for (confirmed: no graph-level attribute method anywhere in the trait). This
is the SAME gap `docs/reference/bsl-language.rst` §3.6 names "R9 chapter C3" / Q6 ("the single
most pervasive gap in the estate," §3.6:2656) and rules on: **route graph-scope values through
ordinary per-node `deffield`s on a declared carrier `NodeType`** — a genuinely-singleton value
uses a `:ceiling 1` carrier and the `the` accessor (§2.10); a per-entity-keyed value (the
ruling's own words: "per-sovereign and per-county registers are ordinary nodes of ordinary
types, reached by ordinary queries," §3.6:2684-2686) is decomposed onto the entity's own node.
**Critically, these two routes have DIFFERENT evaluability today**, verified directly against
`evaluator.rs`: `the` is listed in `UNSERVED_EXPRESSION_HEADS` mapped to **Slice 2**
(`evaluator.rs:506`) — genuinely unevaluable now, same tier as edge attributes — while
`nodes`/`select-max`/`select-min`/`field-of`/`update-node` (everything the per-entity route
needs) are in `EVALUATOR_SERVED`/`SERVED_QUERY_HEADS` — **landed** (Slice 1, ADR197). This
single fact is why PolicySystem's four per-entity registers and its FIFO agenda land
differently from its one true singleton input (`national_financial`).

| Computation | Verdict | Detail |
|---|---|---|
| 0. Empty-register guard | **PORTABLE NOW** | A presence test on two carrier-decomposed registers; expressible once the registers exist as nodes (trivially — "no node of this type exists" is an ordinary `exists`/`fold count` query, landed). |
| 1. Fiscal-terrain gathering | **PORTABLE WITH D-RECORD** | Territory-node reads (`tick_taxes_on_surplus` etc.) are plain `:field`/`fold sum` over `query_territory_claims`-filtered TERRITORY nodes — landed Slice-1 shape. The `sovereign_fiscal.debt_stock` read and the `national_financial` read are the two register inputs — see rows below; this computation's OWN arithmetic is portable, its INPUTS are not both portable yet. |
| 2. Veto-gauntlet assembly | **PORTABLE WITH D-RECORD** | `query_edges(ADMINISTERS)` + `query_nodes(INSTITUTION)` filtered by `apparatus_type` are landed Slice-1 shapes (`fold`/`select-min` over sorted node id — the "first parent" reduces to a `select-min` on id). `internal_balance.liberal_technocratic` is a NESTED dict-inside-an-attribute on the Python side — needs flattening to its own `deffield` (`institution/internal-balance-liberal-technocratic`), a D-record, not a blocker (no query-lane gap; ordinary node field). `NodeType/SOVEREIGN`/`NodeType/INSTITUTION` have zero landed-pack precedent (verified: zero hits in `content/scenarios/*.bscn`, `NodeType/ORGANIZATION` DOES have precedent via `organization-foundation.bscn`) — declarable via the same closed-vocabulary `.bscn` mechanism Territory used for `TERRITORY`, not amendment-gated (only the CARRIER route is called "amendment territory" by the ruling's own text, §3.6:2670-2672; an ordinary entity NodeType is not). |
| 3. Federal preemption arm | **PORTABLE NOW** | Pure arithmetic over already-gathered `FiscalTerrain`/`VetoGauntlet` values — comparisons and multiplies only, no query-lane dependency once the inputs exist. Bare `1.0` literals need `c`-suffixed `defconst`s (same class every prior report flags). |
| 4. Judicial strike-down arm | **PORTABLE NOW** | Same as row 3 — a loop over an already-materialized `judicial_benches` tuple, expressible as `fold`/`exists` over the gathered sequence, or inlined if the gauntlet assembly (row 2) is itself expressed as a query. |
| 5. Funding identity / bond discipline | **PORTABLE NOW** | Every operation is `max`/`min`/`+`/`-`/`*`/`/` over already-gathered Real values — no libm, no query dependency. The bare `0.0`/`1.0` literals throughout `sw_deliverable`/`delivery_ratio`/`delivery_gap`/`bond_discipline_binds`/`finance_shortfall`/`sovereign_debt_service` are the parser-literal D-record class (5+ sites). |
| 6. Preempted/struck dispatch | **PORTABLE WITH D-RECORD** | The `emit` calls themselves are landed (structural_verbs.rs); the payload's `sovereign_id`/`policy_axis`/`preempting_sovereign`/`striking_institution` fields are all `str` — **no `Value::String` variant exists** (verified, `evaluator.rs:53-105`) and a landed test (`r9_chapters.rs:2216`, D75, `a_string_literal_in_an_emit_payload_is_e_parse_010`) proves string payload values are REJECTED, not merely absent. Since `TickReport` carries no event log at all today, this is moot for near-term conformance (a WS1 ledger row per prior reports' convention) but is a real, verified, additional design burden whenever WS1 lands — the payload would need every identity field re-expressed as a `NodeRef`/`Enum`, not a string. |
| 7. Host-discipline clamp | **PORTABLE NOW** | Pure arithmetic (linear interpolation, one division guarded by a branch) once `_is_host_disciplined` (row 9) and the standing overlay value (row 8's own storage) are available. |
| 8. Overlay write + POLICY_ENACTED | **PORTABLE WITH D-RECORD** | `policy_overlays[sovereign][axis] = {4 fields}` decomposes onto the SOVEREIGN node as `7 axes × 4 subfields = 28` `deffield`s (`sovereign/wage-floor-magnitude`, etc.) — mechanically landed (`update-node` on an ordinary queried/self node), content-modeling only. `enacted_tick` needs int-as-a-field (landed, `deffield int`). The event's own string payload is row 6's gap, restated. |
| 9. Host-discipline eligibility | **BLOCKED — string membership + doctrine-content vocabulary** | `"entryism" in tuple(acquired_doctrine_ids)` tests membership in an OPEN, content-authored string set (doctrine ids are themselves TOML-authored game content, not a closed engine enum) — this is a strictly harder version of the string-identity gap (rows 6/8): even a `NodeRef`/`Enum` translation needs the doctrine-tree's OWN port (unstarted, out of scope here) to have first minted a closed `DoctrineId` enum or an `ACQUIRED_DOCTRINE` edge type. `electoral_derecognized` (a `frozenset[str]` graph attr) inherits the same per-entity-register translation as row 8, PORTABLE WITH D-RECORD on its own, but gated here behind the doctrine-membership half. |
| 10. Deficit-financing borrow | **PORTABLE WITH D-RECORD** | `sovereign_fiscal[sovereign] = {debt_stock, last_borrowed}` — 2 scalar `deffield`s on the SOVEREIGN node (`sovereign/debt-stock`, `sovereign/last-borrowed`), `update-node` with `add`/`set` update-ops — landed shape, content-modeling only. |
| 11. L-RECEIPTS funding receipt | **NOT-A-PACK (service-side, no graph write)** | `services.boundary_register.record(...)` is a Python service call with no `set_graph_attr`/`update_node` at all — it appends to an in-memory ledger flushed into the persistence envelope elsewhere. There is no BSL analog for "a service call with no graph effect," and none is needed: this computation produces no observable graph state and is out of scope for a graph-state port by construction, not blocked by any slice. |
| 12. Per-class delivery split | **PORTABLE WITH D-RECORD, ONE STRING FIELD BLOCKED** | `promised_c`/`delivered_c`/`gap_c`/`integral`/`tick` (5 scalars) decompose cleanly onto the SOCIAL_CLASS node (an already-precedented NodeType) — landed shape. `incumbent_id` (a `str`, the sixth field) has no field-value representation; it needs an `INCUMBENT_OF`-style edge from the class (or the sovereign) to the org, mutated via `add-edge`/`remove-edge` (landed structural verbs) on each change — mechanically available but a genuinely fresh per-field design with no existing precedent anywhere in the estate. The `active` guard (SOCIAL_CLASS, default `True`) and `subsistence_threshold` (default via a define) are both landed shapes (`:optional`+`:default`, §3.5). |
| 13. Governance-endgame fork resolution | **BLOCKED — Slice 2 (edge attributes)** | `_org_bridges`'s `SOLIDARITY.solidarity_strength` read is a genuine edge-attribute read: `GraphSubstrate` has NO `edge_attribute` method (verified, full trait read — only `node_attribute` exists), and `edges`/`edge-between` are both in `UNSERVED_EXPRESSION_HEADS` → Slice 2. This is a hard, named-lane block, not a content-modeling question — matches the brief's own instruction that edge-attribute reads are a REAL blocker. Everything else in this computation (`resolve_governance_arm`, `rupture_geometry`, `dual_power_live`, `phi_share`, the `governance_endgame[incumbent] = {5 fields}` write onto the ORGANIZATION node) is PORTABLE WITH D-RECORD once row 13's edge read is served — `arm`/`geometry` are 2-valued enums (landed `deffield enum`), `contact` is a 4-valued closed string set from PolicySystem's OWN vocabulary (`"preempted"/"struck"/"fiscal"/"capital_strike"`, NOT content-authored) so it is a legitimate `deffield enum` target unlike row 9's doctrine ids; `sovereign_id` is the recurring identity-as-edge gap. |
| 14. Capital-strike application | **PORTABLE WITH D-RECORD** | `equalization_deltas` is pure `float` arithmetic (no libm) over two already-gathered dicts — expressible as a `fold sum`-derived weighted average plus a second pass, all over ordinary TERRITORY nodes (already-precedented NodeType, already exercises `fold`/fold-count query shapes Territory's own report scoped out as "structurally dormant" — here it is LIVE, per §5). The cross-node write (every territory with a nonzero delta, not just claimed ones) is an ordinary `for-each` over a query result — landed effect-position shape. |
| 15. Register write-back | **PORTABLE WITH D-RECORD** | Once every register above is decomposed onto its carrier node(s), "write back" is simply "the rule already wrote the node fields it touched" — there is no separate write-back step in the ported form; this row collapses into rows 8/10/12/13/(the agenda row below) by construction. |
| — `policy_agenda` FIFO drain, `policy_agenda_rate == 1` (the shipped default) | **PORTABLE NOW, LANDED PRECEDENT** | `docs/reference/bsl-language.rst` §3.8/D59 names this EXACT re-modeling — "a FIFO agenda becomes its own bounded `NodeType` carrying a `queued-at-tick` field, and 'the next item' becomes `select-min`" — and a landed, PASSING test proves it costed and well-formed: `r9_chapters.rs:2259-2271`, `the_fifo_agenda_remodelling_is_expressible_and_bounded`, `(select-min (nodes NodeType/ORGANIZATION) (field-of it organization/queued-at-tick))` = `Ok(83)` fuel. `select-min`/`nodes`/`field-of` are all in `EVALUATOR_SERVED`/`SERVED_QUERY_HEADS` (landed, Slice 1). Draining one item per tick — the shipped `policy_agenda_rate: 1` default (`defines.yaml:1090`) — is genuinely portable today, mechanism-wise. |
| — `policy_agenda` FIFO drain, `policy_agenda_rate > 1` (the `mitterrand` golden's 24-item burst) | **UNVERIFIED** | Taking the K lowest-`drafted_tick` items in one rule/tick (K a runtime `defconst`, not statically 1) has no landed or documented precedent found — D59's own test selects exactly one item. Whether K sequential `select-min`+consume steps compose within BSL's one-rule-one-firing-per-anchor model, or whether this needs K separately-anchored rules unrolled from the `defconst` at content-authoring time, is an open design question this inventory did not resolve; searched `bsl-language.rst` for "top-k"/"take-n"/"limit" (no results beyond `select-max`/`select-min` themselves). Flagged honestly rather than asserted either way — this is the ONE row in this table I could not adjudicate. |
| — `national_financial` singleton read (`_interest_rate`) | **BLOCKED — Slice 2 (`the`)** | A genuinely one-per-graph value — the R9-C3 ruling's own worked example is `polity/imperial-rent-pool`, structurally identical. The STATIC layer is real and tested (`r9_chapters.rs:478-580`, `c3_graph_scope_carriers` — manifest/grammar/cost checks all pass), and the evaluator DOES have a dedicated runtime error path for it (`EvalCode::UnhydratedCarrier`/`E-EVAL-035`, `evaluator.rs:160-163,206`) — but `"the"` itself has **no raise site anywhere in `evaluator.rs`** (grep-confirmed: the string `"the"` appears exactly once in the whole file, in the `UNSERVED_EXPRESSION_HEADS` table mapping it to Slice 2). This corrects an initial over-optimistic reading of the ruling: designed, statically tested, evaluator-aware — but not yet evaluable. PolicySystem does not OWN this register (TickDynamicsSystem does); it is a real cross-system input dependency, not a defect in PolicySystem's own design. |
| — `internal_balance.liberal_technocratic` (INSTITUTION, nested-dict-in-attribute) | **PORTABLE WITH D-RECORD** | Not a graph-scope register — an ordinary node's attribute happens to be Python-typed as a 4-field nested dict. Flattens to 4 `deffield`s on INSTITUTION (only `liberal_technocratic` is actually read by PolicySystem); same class of translation as every register row above, just already at single-node scope. |

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_policy.py` | 728 | **Primary system-level conformance oracle.** 32 test functions across 13 classes (`TestSystemIdentity`, `TestEmptyRegisterGuard`, `TestAgendaMechanics`, `TestTradeTariffOverlay`, `TestVetoGauntletOnTerrain`, `TestFundingIdentityOnTerrain`, `TestCapitalStrike`, `TestLegislateSeam`, `TestOverlayConsumers`, `TestFullEngineTick`, `TestHostDisciplineClamp`, `TestBetrayalCrossing`, `TestGovernanceFork`) — exercises `PolicySystem.step()` directly against hand-built graphs and the `electoral_fixture` scenario. Behavioral-contract style (own docstring: "THE LEGISLATE unit... Byte-safety is the empty-register guard"). |
| `tests/unit/engine/systems/test_electoral_goldens.py` | 311 | **Strongest conformance-oracle candidate in the system.** Drives all five golden scenarios through the REAL `simulation_engine.step()` loop (not a hand-built graph) and pins named events per scenario — `TestMitterrandGolden`, `TestSyrizaGolden`, `TestDebsGolden`, `TestBernieValveGolden`, `TestWeimarGolden` (per its own docstring, lines 1-15). Full-engine, cross-system, byte-identical-baseline-style. |
| `tests/unit/domain/politics/test_policy.py` | 317 | **Pure-law conformance oracle for `resolve_legislate` itself** — "these laws never see a graph" (own docstring). The single best target for a language-agnostic property-law port (rewrite-test candidate per the CLAUDE.md Tests-as-Behavioral-Contracts standard). |
| `tests/unit/domain/politics/test_governance_endgame.py` | 123 | Pure-law oracle for `resolve_governance_arm`/`rupture_geometry`/`phi_share`/`betrayal_crossed`/`dual_power_live` — zero graph/engine dependency. |
| `tests/unit/economics/distribution/test_sovereign_fiscal.py` | 88 | Pure-law oracle for `sovereign_debt_service`/`bond_discipline_binds`/`finance_shortfall`/`borrow`. |
| `tests/unit/economics/substrate/test_equalization_deltas.py` | 104 | Pure-law oracle for the reused `equalization_deltas` (conservation `ΣΔc=0`, non-negativity). |
| `tests/unit/formulas/test_politics.py` | 286 | Property laws for the whole `formulas/politics.py` module; only the `sw_deliverable`/`delivery_gap`/`delivery_ratio` subset is PolicySystem-relevant (the file also covers Allegiance/Electoral's `valve_multiplier`/`hope_field`/`turnout_share`/etc., which PolicySystem never imports). |
| `tests/unit/economics/substrate/test_equalization.py` | 276 | Tests `DefaultHexEqualizationComputer`/ground-rent integration — **not** PolicySystem's call path (only `equalization_deltas` is shared); schema/integration-level, not a PolicySystem oracle. |
| `tests/unit/economics/substrate/test_equalization_rent.py` | 197 | Ground-rent extraction tests — same scope note as above, not PolicySystem-relevant. |
| `tests/unit/sentinels/test_superstructure.py` | (not read in full; cited by `sentinels/superstructure/registry.py`'s own docstring) | Enforces the write-ownership sentinel (`SUPERSTRUCTURE_ATTR_OWNERS`) — a structural/vocabulary-family test, not a behavioral oracle, but directly evidences the register-ownership facts §5 relies on. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` hashes every
node/edge attribute of the `WorldState→graph` projection on every canonical scenario — so any
PolicySystem output change on `mitterrand`/`syriza`/`bernie_valve` is caught by the byte-identical
hash gate, and (per §5) these three DO exercise real PolicySystem computation, unlike the base
six. `weimar`/`debs` (the other two of the five electoral goldens) do NOT seed `policy_agenda`
or `electoral_governments` (grep-confirmed: neither factory sets `superstructure_registers` at
all) — PolicySystem's empty-register guard (computation 0) makes them behave identically to the
base six for this system specifically, even though they exercise other Program-25 systems live.

## Method note

Every claim above traces to a file:line anchor the agent read directly (Read tool, full-file
reads for every module in §1's table except the explicitly-scoped exclusions, which are named
and reasoned about, not silently skipped). Where the CURRENT BSL surface brief's own framing
needed correction — the graph-scope-carrier ruling's evaluability specifically — the correction
is shown with its own verification trail (the `"the"` grep against `evaluator.rs`, contradicting
an initial reading of the manifest-layer tests as sufficient) rather than asserted. The one
item this inventory could not resolve (`policy_agenda_rate > 1` drain) is marked UNVERIFIED
with the search performed, per the assignment's honesty rule, rather than guessed in either
direction.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`) with fresh `rg`/Read. Four corrections,
six confirmations. This is the most complete report in its batch, and its independent correction of
the `the`/carrier-node evaluability question is the single best piece of verification in the group —
the sibling Allegiance and Electoral inventories should be read against it.

### CORRECTIONS

1. **CORRECTION — §7 is wrong: PolicySystem has ZERO `graph_content_hash` coverage.** Two
   independent reasons, both verified:
   - **All five owned registers are graph metadata.** `policy_agenda`, `policy_overlays`,
     `sovereign_fiscal`, `policy_delivery`, `governance_endgame` round-trip through
     `WorldState.superstructure_registers` and are re-stamped onto `G.graph`
     (`_harvest_superstructure_registers`, world_state.py:344-352; re-stamp at :823-830), never onto a
     node. `graph_content_hash` (`tools/regression_test.py:924-964`) hashes `state.to_graph()`'s
     **nodes and edges** and its own docstring excludes graph metadata ("Graph *metadata* (``g.graph``:
     economy, event log, opposition states) is also excluded").
   - **`TERRITORY.tick_capital_stock` — §5's one genuine same-tick downstream node write — is dropped
     on reconstruction by PREFIX.** `_reconstruct_territory` filters
     `if k not in TERRITORY_EXCLUDED_FIELDS and not k.startswith(("tick_", "flow_"))`
     (world_state.py:256), so every `tick_*` attribute is excluded and never re-enters the projection.
   So "any PolicySystem output change on `mitterrand`/`syriza`/`bernie_valve` is caught by the
   byte-identical hash gate" is false in both halves. The real oracles are the ones §5 already cites
   correctly: `tests/unit/engine/systems/test_electoral_goldens.py`'s per-tick event assertions and the
   `SystemEvidence` gate-coverage rows (`tools/regression_scenarios.py:2270-2338,2547-2553`). Worth
   stating plainly for the port train: **this system's conformance oracle is behavioral, not
   byte-identical**, so a BSL transcription cannot be validated by re-running `qa:regression`.

2. **CORRECTION — `SUPERSTRUCTURE_REGISTERS` is 11 names, not 12.** Read in full
   (`src/babylon/models/superstructure.py`): `policy_agenda`, `policy_overlays`, `sovereign_fiscal`,
   `policy_delivery`, `governance_endgame`, `electoral_governments`, `electoral_disillusion`,
   `electoral_derecognized`, `popular_front`, `political_form_org_positions`,
   `political_labor_share`. The "5 are PolicySystem's own" half is correct.

3. **CORRECTION — computation 12's `incumbent_id` row understates the blocker by one step.** The
   proposed `INCUMBENT_OF`-style edge is mutated with `add-edge`/`remove-edge`, which are landed
   effect verbs — but the READ side ("which org is this class's incumbent") needs
   `edges`/`edge-between`/`the`, every one of which is Slice 2 (`UNSERVED_EXPRESSION_HEADS`,
   evaluator.rs:503-512), and a typed `neighbors` walk only recovers the org when exactly one such
   edge exists and its type is unshared. The row's verdict should be **PORTABLE WITH D-RECORD on the
   write, BLOCKED on Slice 2 for the read-back**, not a clean D-record. This makes the Slice-2
   blocker count **three**, not two, and the executive-summary/verdict line should say so.

4. **CORRECTION (extension, not contradiction) — the empty-register guard is provably the WHOLE
   behavior on 9 of 12 scenarios, which is stronger than "not structurally dormant".**
   `models/superstructure.py`'s own docstring rules the absence: *"Honest absence (Constitution
   III.11): a register that was never written is NOT carried as an empty value … The six party-less
   qa:regression scenarios therefore never see any of these names at all."* Adding `org_probe` (no
   party terrain) that is 7 of 12 with the names structurally absent, and §7's own verified
   observation extends it to 9 of 12 (`weimar`/`debs` seed neither `policy_agenda` nor
   `electoral_governments`). Computation 0 is therefore a Metabolism-D-2-class "provably uniform"
   fact on three quarters of the estate, not a live branch — which is exactly the framing that lets
   the port's first pack be honest about what it does and does not exercise.

### CONFIRMATIONS

5. **CONFIRMATION — the `the`-is-Slice-2 finding, independently reproduced, and it is the best
   verification in this batch.** `rg -c '"the"' rust/crates/babylon-bsl/src/evaluator.rs` returns
   **1**, and that single occurrence is `("the", "slice 2")` at line 506 inside
   `UNSERVED_EXPRESSION_HEADS`. Meanwhile `SERVED_QUERY_HEADS` (`["nodes", "neighbors"]`, :527) and
   the `eval_form` arms for `fold`/`exists`/`forall`/`select-max`/`select-min`/`field-of` (:556-559)
   are landed. So the split this report draws — **singleton → carrier + `the` → Slice 2; per-entity →
   ordinary node fields → landed Slice 1** — is exactly right, and it is the correct reading of
   §3.6's own closing paragraph (bsl-language.rst:2686-2688, "per-sovereign and per-county registers
   are ordinary nodes of ordinary types, reached by ordinary queries"). The Allegiance and Electoral
   inventories in this batch both file their keyed registers under a blanket "no storage class" and
   need this correction applied to them.

6. **CONFIRMATION, STRENGTHENED — computation 13's `_org_bridges` Slice-2 block.**
   `policy.py:529-538` reads `edge.attributes.get("solidarity_strength", 0.0)` off every SOLIDARITY
   edge incident to the org. Verified deeper than the row states: `GraphSubstrate` has no
   edge-attribute reader **at all** — its whole edge surface is
   `add_edge(edge_type, from, to, strength: f64)` (substrate.rs:111-117), `remove_edge` (:124),
   `edges(edge_type) -> Vec<(NodeId, NodeId)>` (:166); `node_attribute` exists at :141 with no edge
   counterpart, and even the mandatory `:strength` has no reader (`rg -n "strength" substrate.rs` →
   lines 22, 105, 116, all write-side). Slice 2 must mint the substrate read method as well as the
   `EdgeRef`; record that in the lane's scope.

7. **CONFIRMATION — the agenda drain and its UNVERIFIED verdict.** `defines.yaml:1090` ships
   `policy_agenda_rate: 1`; the drain is
   `rate = max(1, int(defines.policy_agenda_rate)); executed, remaining = agenda[:rate], agenda[rate:]`
   (policy.py:185-186) — a genuine top-K take, not a repeated single-item select. D59's landed test
   (`r9_chapters.rs`, `the_fifo_agenda_remodelling_is_expressible_and_bounded`) selects exactly one
   item and does not cover K>1. Marking this UNVERIFIED with the search shown is the right call and
   the right house standard.

8. **CONFIRMATION — tick position 17.47** (policy.py:136), ordering per `_SYSTEM_CLASSES`
   (simulation_engine.py:328-363).

9. **CONFIRMATION — the `NOT-A-PACK` verdict for `services.boundary_register.record`.** It performs
   no `set_graph_attr`/`update_node` and produces no observable graph state; scoping it out by
   construction rather than force-fitting a BSL analogue is correct, and it is the only row in this
   batch that reaches that verdict honestly.

10. **CONFIRMATION — the RESERVED-LINE handling of computation 9.** `"entryism" in
    tuple(acquired_doctrine_ids)` tests membership in an open, content-authored doctrine-id set; the
    row correctly refuses to invent a closed `DoctrineId` enum and defers to the doctrine tree's own
    (unstarted) port. Correct discipline — this is Director-reserved content, not an engineering
    call.

### FINAL VERDICT

**PORTABLE WITH D-RECORDS for the resolver math and the four per-entity registers (content-modeling
only — the per-entity decomposition rides landed Slice-1 heads); BLOCKED on Slice 2 for THREE
computations, not two — `_org_bridges`'s edge-attribute read (which additionally needs the
`GraphSubstrate` edge-attribute reader Slice 2 must mint), the `national_financial` singleton input
via `the`, and computation 12's `incumbent_id` read-back; UNVERIFIED for `policy_agenda_rate > 1`.
Two facts to carry forward that the report does not state: the empty-register guard is provably the
whole behavior on 9 of 12 canonical scenarios (Metabolism-D-2 class), and PolicySystem carries ZERO
`graph_content_hash` coverage — all five registers are `g.graph` metadata and `tick_capital_stock`
is dropped by the `tick_*` prefix filter at world_state.py:256 — so the port's conformance oracle is
`test_electoral_goldens.py`'s behavioral assertions, never a `qa:regression` byte comparison.**
