# MarketScissorsSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `MarketScissorsSystem` (position 17.8, 582 lines, `src/babylon/engine/systems/market_scissors.py`)
is the cleanest computational core in the estate — pure damped-driven-oscillator arithmetic, zero edge
traversal, zero libm hazard in its OWN oscillator step — but its entire state model lives in
graph-level metadata (`graph.graph["market"]`/`["market_county"]`), not on any node, and every one of
its aggregations is gated by Python dict-key **presence**, not by a declared field's value. Both facts
turn out to be genuine, precisely-named BSL gaps rather than merely-unbuilt query-lane heads: §3.6 of
`bsl-language.rst` already RULES how graph-scope state is represented (a `:ceiling 1` carrier
`NodeType`, read/written through `the`) and even names "the market-scissors axis" by name as one of
the twenty-two systems the ruling covers — but `the` is `UNSERVED_EXPRESSION_HEADS`-refused
(`evaluator.rs:506`, scoped to Slice 2, "SCOPED, NOT BUILT" per ADR197 on dev HEAD today). The
presence-based selection gap is worse: §3.5.3 rules there is **no `bound?` predicate in the language**
at all, by design — every one of this system's six aggregation helpers keys its whole selection on
`attr not in node.attributes`, which BSL cannot express even in principle without a content-modeling
workaround. Against that, the actual per-node oscillator math (growth-drive, semi-implicit Euler step,
EMA, correction snap, severity, overhang) is portable arithmetic with only ordinary D-record-class
literal/scale-op transcription work, and unlike `TerritorySystem` this system's **national** axis is
demonstrably live on the canonical `qa:regression` estate (imperial_circuit/two_node/single_county),
though one whole feedback arm (`_swell_reserve_army`'s reserve-army bump) is structurally dead on
every canonical scenario because nothing seeds `median_wage`.

**Verdict:** BLOCKED on the graph-scope-state carrier (`the`, Slice 2, not built) and on presence-based
field selection (no `bound?` predicate, ruled absent by design) — the oscillator arithmetic itself is
portable today, but the system has no BSL home to store or read from until those two gaps close.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/market_scissors.py` | 582 | **The target.** `MarketScissorsSystem`, module-level helpers `_aggregate_wage_value`, `_aggregate_wage_value_by_county`, `_read_fictitious_anchor`, `_mean_profit_rate`, `_mean_ratio_to_capital`, `_national_serviceability`. |
| `src/babylon/formulas/market.py` | 227 | Pure oscillator math: `calculate_ema`, `calculate_growth_drive`, `calculate_scissors_step`, `calculate_scissors_balance` (NOT called by this system — see §5), `calculate_serviceable_divergence`, `calculate_overhang`, `calculate_correction_snap`, `calculate_correction_severity`, `calculate_anchor_pull`. Imports `math` (`market.py:18`) but only `calculate_scissors_balance` uses it (`math.tanh`, line 107) — and that function is the one this system never calls. |
| `src/babylon/domain/economics/monetary/anchor.py` | 158 | `fictitious_anchor` (42-89, **`math.log` at line 89** — the one libm call this system's own execution path reaches) and `serviceability_anchor` (92-158, pure division, no transcendental). |
| `src/babylon/domain/economics/monetary/__init__.py`, `converter.py`, `types.py`, `data_sources.py` | 39/174/48/59 | Re-export surface + `FictitiousCapitalStock`/`EndogenousInterestRate` support; not independently exercised beyond the two anchor functions above. |
| `src/babylon/domain/economics/tensor.py` | — | `NoDataSentinel` (line 45) — the honest-absence marker `fictitious_anchor`/`serviceability_anchor` return past the data horizon. |
| `src/babylon/domain/economics/distribution/types.py` | — | `SurplusValueDistribution` (line 94) — 5 non-negative `float` fields + 2 `@computed_field` properties; constructed at `market_scissors.py:570-580` for the aggregate national interest-burden read. |
| `src/babylon/domain/economics/credit/types.py` | — | `FictitiousCapitalStock` (228, `ratio_to_real` at 256-268, pure division) and `EndogenousInterestRate` (271, `profit_rate_ceiling: float = Field(..., ge=0.0)` at 303). |
| `src/babylon/domain/economics/tick/types.py` | — | `NationalFinancialParameters` (473) — the `national_financial` graph-attr's Pydantic shape; `CountyEconomicState` (279) — one `county_states` dict value's shape. |
| `src/babylon/domain/economics/tick/graph_bridge.py` | — | `NATIONAL_FINANCIAL_ATTR = "national_financial"` (430), `TICK_DYNAMICS_KEY = "tick_dynamics"` (41) — both written by `TickDynamicsSystem` @4.0, read by this system. |
| `src/babylon/domain/economics/tick/system/__init__.py` | — | `TickDynamicsSystem` (`class` at 112, `position = 4.0` at 124) — the prior same-tick producer of both graph-metadata inputs this system reads; its annual-boundary gate (`if tick % WEEKS_PER_YEAR != 0` at line 174) governs when those inputs are populated (see §5). |
| `src/babylon/engine/systems/wealth_distribution.py` | 274 | `MARKET_CORRECTION_SHOCK_ATTR = "market_correction_shock"` (77, **declared in the consumer**, not the producer — comment explains why at 72-76); `bracket_of_role` (80-86, imported by `market_scissors.py:39`); `WealthDistributionSystem` (218, `position = 21.5`) consumes the shock stamp at 251-255/257 — see §5. |
| `src/babylon/models/market.py` | 43 | `MarketState` — frozen Pydantic model, the national/per-county oscillator's on-the-wire shape. |
| `src/babylon/config/defines/market.py` | 246 | `MarketDefines` — 21 fields, all documented with `ge`/`le` bounds. |
| `src/babylon/data/defines.yaml` | market block: 986-1012 | Player-editable values, matching `MarketDefines` field-for-field. |
| `src/babylon/models/enums/social.py` | — | `SocialRole` (12, StrEnum, 8 members, 35-43) + `.coerce` (46-64). |
| `src/babylon/models/enums/topology.py` | — | `NodeType.TERRITORY`/`SOCIAL_CLASS` (61-62); `NodeType.COUNTY` is also declared (75) but is **not instantiated anywhere in `engine/systems/`** — see §5/§6, this matters for the per-county axis's storage question. |
| `src/babylon/models/enums/events.py` | — | `EventType.MARKET_CORRECTION = "market_correction"` (80). |
| `src/babylon/models/events/spine_payloads.py` | — | `MarketCorrectionEvent` (28-45) — the event payload shape; docstring at 33-35 is **stale** (see §2 Phase-4 note). |
| `src/babylon/engine/event_builders.py` | — | `EVENT_BUILDERS[EventType.MARKET_CORRECTION]` (525-534) — dict→event reconstruction. |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_nodes` (258-266, `node_type: str \| None = None` — **"None = all types" is a first-class production feature**, not a workaround), `.get_graph_attr`/`.set_graph_attr` (350-366), `.update_node` (88). |
| `src/babylon/topology/graph.py` | — | Concrete `BabylonGraph`: `.graph` property (338-340) returns the SAME dict as `._graph_attrs`, so `metadata[MARKET_ATTR] = ...` (direct dict-item assignment) and `graph.set_graph_attr(...)` (method call) are the same store reached two different ways within this one file — a style inconsistency, not a bug (see §4). `update_node` (660) — plain dict merge, no mid-tick quantization (same fact as the Territory inventory). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` — `partition`/`position` ClassVars only; this system calls neither `self._wrap_graph` nor `self._read` (grep-confirmed against the full 582-line file) — unlike `reserve_army.py`, which does. |
| `src/babylon/engine/simulation_engine.py` | — | `_SYSTEM_CLASSES` (328-363) — confirms tick position and full ordering; `MarketScissorsSystem` sits between `SovereigntySystem` (17.5) and `ContradictionSystem` (18.0). |

**Not exercised by `market_scissors.py`:** no edge query of any kind (`query_edges` never called), no `neighbors`/adjacency traversal, no hyperedge/incidence read. Every node access is a flat `sorted(graph.query_nodes(...), key=lambda n: n.id)` scan, optionally type-filtered. This is architecturally simpler than `TerritorySystem` in exactly the dimension the Territory port was blocked on (graph-query evaluation) — and architecturally harder in a dimension Territory never touched (graph-scope, non-node state; see §5/§6).

**Reference BSL packs read for format** (skimmed for grammar/content-modeling precedent, not fully re-read line-by-line since they were already fully read for the Territory inventory): `metabolism.bsl`, `vitality.bsl`, plus `docs/reference/bsl-language.rst` §3.5 (binding resolution, 2619-2637), §3.6 (closed vocabulary + the graph-scope-state ruling, 2639-2689), and the `UNSERVED_EXPRESSION_HEADS`/`UNSERVED_QUERY_HEADS` tables in `rust/crates/babylon-bsl/src/evaluator.rs:503-512` and `rust/crates/babylon-bsl/src/query.rs:76-81`.

## 2. COMPUTATION CATALOG (execution order, `step()` at `market_scissors.py:164-200`)

### Step 0 — National flow aggregation (`_aggregate_wage_value`, `market_scissors.py:82-102`)
- **(a)** Sum wages paid and value produced across every active node in the graph (any node type) that carries BOTH `w_paid` and `v_produced` this tick; `None` (not zeros) if no node carries the pair.
- **(b)** `wages += float(attrs["w_paid"])`; `value += float(attrs["v_produced"])` over `sorted(graph.query_nodes(), key=lambda n: n.id)` (line 93), skipping `not attrs.get("active", True)` and any node missing either key (95-98). Returns `(wages, value)` iff `found` (102).
- **(c) Reads:** ALL node types (unfiltered `query_nodes()`), `active` (bool, default `True`), `w_paid` (float, presence-gated), `v_produced` (float, presence-gated).
- **(d) Writes:** none (pure read).
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 1 — Surplus + fictitious anchor (`step`, `market_scissors.py:180-181`; `_read_fictitious_anchor`, 132-151; `fictitious_anchor`, `anchor.py:42-89`)
- **(a)** `surplus = max(value − wages, 0)` — realized surplus never negative. Resolve the D1 real-data pull target: `log(total_claims / real_output)` when a `FictitiousCapitalStock` was published this tick AND `real_output > 0`, else honest `None`.
- **(b)** `surplus = max(value - wages, 0.0)` (`market_scissors.py:180`). `_read_fictitious_anchor`: `raw = metadata.get(NATIONAL_FINANCIAL_ATTR)`; if not a dict, `None` (144-146); else `NationalFinancialParameters.model_validate(raw)` then `fictitious_anchor(national_financial.fictitious_capital, real_output=value)` (147-148); a `NoDataSentinel` result becomes `None` (149-150). Inside `fictitious_anchor` (`anchor.py:42-89`): four sequential absence/finiteness guards (`stock is None` → sentinel; `real_output is None` → sentinel; non-finite/non-positive `real_output` → sentinel; non-finite/non-positive `ratio` → sentinel), then **`return math.log(ratio)`** (line 89) — the system's one libm transcendental.
- **(c) Reads:** graph-metadata `national_financial` (`NATIONAL_FINANCIAL_ATTR`, dict); `NationalFinancialParameters.fictitious_capital` (`FictitiousCapitalStock | None`, nested).
- **(d) Writes:** none.
- **(e) Defines:** none (the anchor function is defines-free; its GAIN into the oscillator is `defines.anchor_pull`, applied in Step 2).
- **(f) Events:** none.

### Step 2 — National oscillator seed/advance (`step`, 182-196; `_advance`, `market_scissors.py:267-323`)
- **(a)** First observation: seed both log-ratios and velocities at 0.0, EMAs at this tick's raw flow. Every later tick: one semi-implicit-Euler step of two coupled damped-driven oscillators — price chases value-output growth against the law-of-value restoring force; fictitious capitalization chases realized-surplus growth **plus** price-velocity momentum (speculation rides the boom) **plus** the D1 real-data anchor pull when present.
- **(b)** Seed branch (188-196): `price_log=0.0, price_velocity=0.0, fictitious_log=0.0, fictitious_velocity=0.0, surplus_ema=surplus, value_ema=value, tick=int(tick)`. Advance branch (`_advance`, 287-323): `price_drive = calculate_growth_drive(value, prior.value_ema, sensitivity=price_drive_sensitivity)` → `calculate_growth_drive` (`market.py:47-58`): `0.0` if `previous <= 1e-9` (the `_GROWTH_EPSILON` module constant, line 32) else `sensitivity * (current - previous) / previous`. `(price_log, price_velocity) = calculate_scissors_step(prior.price_log, prior.price_velocity, price_drive, reversion=price_reversion, damping=price_damping, max_abs_log=max_abs_log)` → `calculate_scissors_step` (`market.py:61-94`): `acceleration = drive - reversion*log_ratio - damping*velocity`; `new_velocity = velocity + acceleration`; `new_log = log_ratio + new_velocity`; clamp: `new_log > max_abs_log → (max_abs_log, 0.0)`, `new_log < -max_abs_log → (-max_abs_log, 0.0)`, else `(new_log, new_velocity)` — **the rail zeroes velocity, a two-sided hard clamp with a side-effect on the paired state variable**. `fictitious_drive = calculate_growth_drive(surplus, prior.surplus_ema, sensitivity=fictitious_drive_sensitivity) + momentum_coupling * price_velocity + calculate_anchor_pull(anchor, prior.fictitious_log, gain=anchor_pull)` → `calculate_anchor_pull` (`market.py:204-227`): `0.0` if `anchor is None` else `gain * (anchor - current)`. Same `calculate_scissors_step` call for the fictitious pair. `surplus_ema = calculate_ema(prior.surplus_ema, surplus, alpha=surplus_ema_alpha)`, same for `value_ema` → `calculate_ema` (`market.py:36-44`): `alpha*value + (1-alpha)*previous`. `corrections`/`last_correction_tick` pass through unchanged (321-322).
- **(c) Reads:** prior `MarketState` (graph-metadata `market`, or absent on first observation); `surplus`, `value` (Step 0/1 outputs); `anchor` (Step 1 output, national branch only — `_step_county_axes` never passes one, `market_scissors.py:283-285`).
- **(d) Writes:** none yet (returned `MarketState`, committed to metadata at line 199 after the Step 3 correction branch).
- **(e) Defines:** `price_drive_sensitivity` (0.6, `[0,5]`), `price_reversion` (0.02, `[0,1]`), `price_damping` (0.15, `[0,2]`), `max_abs_log` (2.0, `(0,5]`), `fictitious_drive_sensitivity` (0.9, `[0,5]`), `momentum_coupling` (0.5, `[0,5]`), `anchor_pull` (0.1, `[0,1]`), `fictitious_reversion` (0.01, `[0,1]`), `fictitious_damping` (0.1, `[0,2]`), `surplus_ema_alpha` (0.15, `(0,1]`) — all `defines.yaml:990-997,1010`, `config/defines/market.py:20-92,214-224`.
- **(f) Events:** none.

### Step 3 — Phase-2 correction (`_maybe_correct`, `market_scissors.py:330-412`; gated by `defines.feedback_enabled`, default `True`)
- **(a)** If the fictitious log-ratio exceeds what the realized rate of profit (tightened by the national interest burden) can service, AND the cooldown has elapsed: snap both log-ratios toward par (severity debt-adjusted for the fictitious pole), destroy claim-holder wealth, swell the reserve army where a wage relation exists, stamp a shock for `WealthDistributionSystem` @21.5, and publish `MARKET_CORRECTION`.
- **(b)** `profit_rate = _mean_profit_rate(graph)` (347, see below). `interest_burden = _national_serviceability(graph)` (353, see below). `serviceable = calculate_serviceable_divergence(profit_rate, base=correction_threshold_base, slope=correction_profit_slope, interest_burden=interest_burden, interest_slope=correction_interest_slope)` → `market.py:110-147`: `profit_term = 0.0 if profit_rate is None else slope*max(profit_rate,0.0)`; `interest_term = 0.0 if interest_burden is None else interest_slope*max(interest_burden,0.0)`; `return max(base + profit_term - interest_term, 0.0)`. `overhang = calculate_overhang(state.fictitious_log, serviceable)` → `market.py:150-158`: `max(fictitious_log - serviceable, 0.0)`. `if overhang <= 0.0: return state` (362-363, no-op). Cooldown: `if last_correction_tick is not None and tick - last_correction_tick < correction_cooldown_ticks: return state` (364-368). `debt_ratio = _mean_ratio_to_capital(graph, "tick_accumulated_debt")` (370). `severity = calculate_correction_severity(correction_severity, debt_ratio=debt_ratio, slope=correction_debt_slope)` → `market.py:179-201`: `base_severity` unchanged if `debt_ratio is None`, else `min(max(base_severity + slope*max(debt_ratio,0.0), 0.0), 1.0)` — a two-sided clamp wrapping a lower-only inner clamp. `(fictitious_log, fictitious_velocity) = calculate_correction_snap(state.fictitious_log, state.fictitious_velocity, severity=severity)` → `market.py:161-176`: `(log_ratio*(1-severity), min(velocity, 0.0))` — closes toward par, kills only UPWARD momentum. Same call for price with the FIXED `correction_price_severity` define (not the debt-adjusted `severity`). `self._evaporate_wealth(graph, overhang, defines)` (384, see Step 4). `self._swell_reserve_army(graph, overhang, defines)` (385, see Step 5). `graph.set_graph_attr(MARKET_CORRECTION_SHOCK_ATTR, {"tick": tick, "overhang": overhang})` (386). `corrected = state.model_copy(update={...corrections: state.corrections+1, last_correction_tick: tick...})` (387-396). `services.event_bus.publish(Event(type=EventType.MARKET_CORRECTION, ...))` (397-411).
- **(c) Reads:** graph-metadata `national_financial` (via `_mean_profit_rate`), `tick_dynamics` (via `_national_serviceability`), TERRITORY `tick_accumulated_debt`/`tick_capital_stock` (via `_mean_ratio_to_capital`); `state.fictitious_log`, `state.last_correction_tick`, `state.corrections`.
- **(d) Writes:** graph attr `market_correction_shock` (`{"tick": int, "overhang": float}`); (transitively, via the two helpers below) SOCIAL_CLASS `wealth`, TERRITORY `reserve_ratio`.
- **(e) Defines:** `correction_threshold_base` (0.55, `[0,2]`), `correction_profit_slope` (4.0, `[0,20]`), `correction_interest_slope` (2.0, `[0,20]`), `correction_cooldown_ticks` (8, `[1,520]`), `correction_debt_slope` (0.5, `[0,5]`), `correction_severity` (0.6, `[0,1]`), `correction_price_severity` (0.3, `[0,1]`), `feedback_enabled` (bool, default `True`) — `defines.yaml:1001-1012`, `config/defines/market.py:113-245`.
- **(f) Events:** `EventType.MARKET_CORRECTION` (`enums/events.py:80`) — payload `{overhang, serviceable, profit_rate, fictitious_log_before/after, price_log_before/after}` (`market_scissors.py:400-410`; shape pinned by `MarketCorrectionEvent`, `spine_payloads.py:28-45`). **The docstring at `spine_payloads.py:33-35` is stale**: it says `profit_rate` is None when `_mean_profit_rate` "finds no territory carrying `tick_profit_rate`" — the CURRENT implementation (`market_scissors.py:463-494`) reads `NATIONAL_FINANCIAL_ATTR.endogenous_interest.profit_rate_ceiling`, a single published national scalar, never a per-territory `tick_profit_rate` scan. Transcribed verbatim per port-as-is law; flagged as a documentation defect, not a code defect.

### Step 3a — `_mean_profit_rate` (`market_scissors.py:463-494`)
- **(a)** The single published national rate of profit, or honest `None` if no financial state was published this tick or the published ceiling is exactly `0.0` (treated as "no realized profit measured," per the docstring's own stated convention, not a fabricated non-absence).
- **(b)** `raw = graph.get_graph_attr(NATIONAL_FINANCIAL_ATTR, None)`; not a dict → `None`. `endogenous = raw.get("endogenous_interest")`; not a dict → `None`. `ceiling = endogenous.get("profit_rate_ceiling")`; not `(int,float)` or is `bool` → `None`. `return float(ceiling) if ceiling > 0.0 else None`.
- **(c) Reads:** graph-metadata `national_financial` → nested `.endogenous_interest.profit_rate_ceiling` (float, `ge=0.0`, `credit/types.py:303`).
- **(d) Writes:** none.
- **(e) Defines:** none (pure graph read).
- **(f) Events:** none.

### Step 3b — `_national_serviceability` (`market_scissors.py:532-582`)
- **(a)** The national interest burden `Σi / Σs`, aggregated as a ratio-of-sums (never a mean of per-county burdens) over the published per-county `SurplusValueDistribution` objects, via `serviceability_anchor`.
- **(b)** `tick_data = graph.get_graph_attr(TICK_DYNAMICS_KEY, None)`; not a dict → `None`. `year = tick_data.get("year")`; not `int` or is `bool` → `None`. `county_states = tick_data.get("county_states")`; not a dict → `None`. Loop `sorted(county_states)` (561, deterministic key order): `distribution = getattr(county_states[fips], "surplus_distribution", None)` — **attribute access on a live object, not `.get()` on a dict** (see §3 note); `None` → skip; else accumulate `total_surplus += distribution.total_surplus_produced`, `total_interest += distribution.interest_payments`, `saw_any = True`. `if not saw_any: return None`. Build `SurplusValueDistribution(fips_code="00000", year=year, total_surplus_produced=total_surplus, interest_payments=total_interest, ground_rent=0.0, taxes_on_surplus=0.0)` (570-580 — the two `0.0` fields are a deliberate national-aggregate simplification, not derived from anything). `result = serviceability_anchor(aggregate)` → `anchor.py:92-158`: guards `surplus <= 0.0`/non-finite → sentinel; else `ratio = interest_payments / surplus`; non-finite → sentinel; else `ratio`. `NoDataSentinel` → `None`.
- **(c) Reads:** graph-metadata `tick_dynamics` → `.year` (int), `.county_states` (dict, values are LIVE `CountyEconomicState` Pydantic objects, not dumped dicts) → each value's `.surplus_distribution` (`SurplusValueDistribution | None`) → `.total_surplus_produced`/`.interest_payments` (both `float, ge=0`).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 4 — `_evaporate_wealth` (`market_scissors.py:414-436`)
- **(a)** Destroy a fraction of claim-holder wealth proportional to the overhang, on active `social_class` nodes whose role folds into the top-1%/petty-bourgeoisie brackets. Labor brackets untouched here.
- **(b)** `fraction = min(evaporation_gain * overhang, 1.0)` (424 — **upper-only clamp**; `if fraction <= 0.0: return` (425, no-op guard, structurally redundant since `evaporation_gain >= 0` and `overhang > 0` at every call site — see §4). Loop `sorted(query_nodes(node_type=SOCIAL_CLASS), key=id)` (427): skip inactive (428); `role = SocialRole.coerce(attrs.get("role"))`; `role is None or bracket_of_role(role) not in _CLAIM_HOLDER_BRACKETS` → skip (430-431); `wealth = attrs.get("wealth")`; not `(int,float)` → skip (433-434); `graph.update_node(node.id, wealth=float(wealth) * (1.0 - fraction))` (436).
- **(c) Reads:** SOCIAL_CLASS `active`, `role` (enum discriminant), `wealth`.
- **(d) Writes:** SOCIAL_CLASS `wealth` (bracket-0/1 roles only, when overhang > 0).
- **(e) Defines:** `evaporation_gain` (0.15, `[0,0.5]`) — `defines.yaml:1007`.
- **(f) Events:** none (folded into the Step 3 `MARKET_CORRECTION` publish).

### Step 5 — `_swell_reserve_army` (`market_scissors.py:438-460`)
- **(a)** Bump `reserve_ratio` on active territories carrying a wage relation (`median_wage`), proportional to the overhang. **Structurally dormant on every canonical scenario** — see §5.
- **(b)** `influx = unemployment_gain * overhang` (448); `if influx <= 0.0: return` (449, redundant guard, same reasoning as Step 4). Loop `sorted(query_nodes(node_type=TERRITORY), key=id)` (451): skip inactive (453); `wage = attrs.get("median_wage")`; not `(int,float)` or `<= 0.0` → skip (456); `current = attrs.get("reserve_ratio")`; `base = float(current) if (int,float) else 0.0` (459); `graph.update_node(node.id, reserve_ratio=min(base + influx, 1.0))` (460 — **upper-only clamp**).
- **(c) Reads:** TERRITORY `active`, `median_wage` (presence + positivity gate), `reserve_ratio` (prior value).
- **(d) Writes:** TERRITORY `reserve_ratio` (only where `median_wage > 0`).
- **(e) Defines:** `unemployment_gain` (0.08, `[0,0.5]`) — `defines.yaml:1008`.
- **(f) Events:** none.

### Step 6 — National metadata commit (`step`, `market_scissors.py:199`)
- **(a)** Write the (seeded, advanced, and possibly corrected) national `MarketState` back to graph metadata.
- **(b)** `metadata[MARKET_ATTR] = state.model_dump()` — direct dict-item assignment on the `.graph` property, not a `set_graph_attr` call (see §1 style-inconsistency note).
- **(c) Reads:** none new.
- **(d) Writes:** graph-metadata `market` (dict, `MarketState.model_dump()` shape).
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 7 — Per-county axes (`_step_county_axes`, `market_scissors.py:202-243`; `_aggregate_wage_value_by_county`, 105-129; `_project_price_divergence`, 245-265)
- **(a)** Same seed/advance mechanism as Step 2, run once PER COUNTY over the wage/value flow restricted to nodes carrying that county's `county_fips` — grouped into a `dict[str, MarketState]`, never national-anchor-pulled, never correction-snapped. Then project each active territory's OWN county's `price_log` onto a `price_divergence` attribute (or an honest `None` if the territory carries the attribute but its county's axis vanished this tick).
- **(b)** `_aggregate_wage_value_by_county` (105-129): same presence-gated loop as `_aggregate_wage_value`, ADDITIONALLY requiring `county_fips is not None` (121-123); accumulates into `flows: dict[str, tuple[float,float]]` keyed by `str(fips)`. `_step_county_axes` (202-243): `flows = _aggregate_wage_value_by_county(graph)`; `priors = metadata.get(MARKET_COUNTY_ATTR)` or `{}`; loop `sorted(flows)` (221, bounded by counties present THIS tick — a variable, unbounded-in-general upper limit, see §4); per-county seed-or-`_advance` (same helper as Step 2, `anchor=None` always — 226); `county_states[fips] = state.model_dump()`. If any county states exist, write `metadata[MARKET_COUNTY_ATTR] = county_states` (239), else POP the key entirely (241 — an ABSENT-on-empty write, not an empty-dict write). If `county_states or priors` (242, catches the "just vanished" transition), call `_project_price_divergence`. `_project_price_divergence` (245-265): loop `sorted(query_nodes(node_type=TERRITORY), key=id)` (256); skip inactive (258); `fips = attrs.get("county_fips")`; `axis = county_states.get(str(fips)) if fips is not None else None` (261); if `axis is not None`: `update_node(node.id, price_divergence=float(axis["price_log"]))` (263); elif `PRICE_DIVERGENCE_ATTR in attrs` (264, **another presence check** — only write the de-positioning `None` if the node already carries the key, never introduce it fresh): `update_node(node.id, price_divergence=None)` (265).
- **(c) Reads:** ALL node types, `active`, `w_paid`, `v_produced` (presence-gated), `county_fips`; graph-metadata `market_county` (priors); TERRITORY `active`, `county_fips`, `price_divergence` (presence-only check, not value).
- **(d) Writes:** graph-metadata `market_county` (dict-of-dicts, or absent when empty); TERRITORY `price_divergence` (`float | None`).
- **(e) Defines:** same 10 oscillator coefficients as Step 2 (the per-county `_advance` call reuses the identical `MarketDefines` fields — no separate per-county coefficient set exists).
- **(f) Events:** none.

**Events emitted by the whole system: one distinct `EventType` — `MARKET_CORRECTION`.** Grep-confirmed (`rg -n 'EventType\.' market_scissors.py` → one hit, line 399). Unlike `TerritorySystem` (zero events), this system DOES touch the event bus, but only on the (canonically rare — see §5) correction-snap branch.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, same fact the Territory inventory recorded): `BabylonGraph.update_node`
(`topology/graph.py:660`) is a plain dict merge with no type coercion or quantization. `Currency`'s
`SnapToGrid` (1e-5 grid) applies only at Pydantic-model instantiation, never mid-tick. All in-tick
arithmetic below is raw Python `float`/`int`.

| Attribute / value | Node type / carrier | Python model type | Domain | Category |
|---|---|---|---|---|
| `w_paid` | any (stamped only on SOCIAL_CLASS in practice — see §5) | none declared — `EXTRA_STAMPABLE_ATTRIBUTES` (`sentinels/vocabulary/registry.py:209`) | unbounded, presumed `≥0` by convention, no Pydantic enforcement | **graph-only, presence-gated; NOT a declared field** |
| `v_produced` | same as `w_paid` | same | same | **graph-only, presence-gated; NOT a declared field** |
| `county_fips` | any | `str` (coerced via `str(fips)`) | open string domain (no enum) | presence-gated join key |
| `active` | any | `bool` | `{T,F}`, default `True` | boolean |
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members, `social.py:35-43`) | closed set | **Enum discriminant** |
| `wealth` | SOCIAL_CLASS | `Currency` (`social_class.py:308`) | `[0,∞)` | unbounded real, money-semantic |
| `tick_capital_stock` | TERRITORY | none declared on `Territory` — graph-only | `≥0` by convention | presence-gated real |
| `tick_accumulated_debt` | TERRITORY | none declared — grep-confirmed only written at `graph_bridge.py:275`, never a `Territory` model field | unbounded | presence-gated real |
| `median_wage` | TERRITORY | `Currency` (`entities/territory.py:216-219`, `default=0.0`) | `[0,∞)` | **declared field, but never populated by production code — see §5** |
| `reserve_ratio` | TERRITORY | `float` (`entities/territory.py:220-225`, `ge=0.0, le=1.0, default=0.0`) | `[0,1]` | unit-interval |
| `price_divergence` | TERRITORY | none on `Territory` model — `EXTRA_STAMPABLE_ATTRIBUTES` (`sentinels/vocabulary/registry.py:222`) | `float \| None` | **graph-only, tri-state (present-with-value / present-as-None / absent)** |
| `price_log`, `price_velocity`, `fictitious_log`, `fictitious_velocity` | `MarketState` (graph metadata, not a node) | `float` (`models/market.py:34-37`) | **unbounded by Pydantic Field; runtime-clamped to `[-max_abs_log, max_abs_log]` only by `calculate_scissors_step`'s own logic** | unbounded real, algorithm-enforced bound |
| `surplus_ema`, `value_ema` | `MarketState` | `float = Field(ge=0.0)` (38-39) | `[0,∞)` | non-negative unbounded real |
| `tick` | `MarketState` | `int = Field(ge=0)` (40) | `≥0` | integer |
| `corrections` | `MarketState` | `int = Field(default=0, ge=0)` (41) | `≥0` | **accumulator (event-sourced count, per its own docstring)** |
| `last_correction_tick` | `MarketState` | `int \| None` (42) | `≥0` or absent | optional integer, cooldown anchor |
| `market` / `market_county` | graph metadata (`graph.graph[...]`) | plain `dict` (`.model_dump()`) | — | **graph-scope state; no node carries it** |
| `national_financial` / `tick_dynamics` | graph metadata | plain `dict` (nested Pydantic dumps, or — for `tick_dynamics.county_states` specifically — LIVE Pydantic objects, not dumps, see §2 Step 3b) | — | **graph-scope state, produced by a different not-yet-ported system** |
| `market_correction_shock` | graph metadata | `{"tick": int, "overhang": float}` | — | **transient, single-tick, consume-and-clear pattern (`wealth_distribution.py:150`)** |

**Enum discriminant note.** `role` is the ONE genuine enum read in this system. Per the task's current-BSL-surface briefing, enum fields ARE landed (ADR195/ADR196): `defenum`, `deffield <field> enum <EnumName>`, `:field`-bound reads, `=`-compared guards; declaration order is the stored ordinal. `SocialRole`'s declaration order (`CORE_BOURGEOISIE`=0 … `CARCERAL_ENFORCER`=7, `social.py:35-43`) is **unrelated** to the wealth-bracket fold `_BRACKET_BY_ROLE` uses (0=top-1%, 1=p90-99, 2=p50-90, 3=bottom-50, `wealth_distribution.py:59-68`) — `bracket_of_role(role) in _CLAIM_HOLDER_BRACKETS` is a LOOKUP TABLE test, not an ordinal comparison, so a port needs either 8 explicit `=`-chained guards (2 true, 6 false) or a genuinely new comparison shape; `field-of` itself is refused for enum-declared fields by D102, so even a landed `role` field can only be read through `:field` binding + guard comparison, never `field-of`.

**Presence-based selection — the pervasive category, and it recurs at EVERY read site in this system.**
Six of this system's seven aggregation/selection helpers gate their whole body on Python
dict-key-absence (`"x" not in attrs`) or `isinstance(...)`-as-presence-proxy
(`isinstance(wage, (int,float))`), not on a declared field's VALUE: `_aggregate_wage_value` (97),
`_aggregate_wage_value_by_county` (119, 122), `_project_price_divergence` (264, on `price_divergence`
itself), `_evaporate_wealth` (433-434, on `wealth`), `_swell_reserve_army` (456, on `median_wage`),
`_mean_ratio_to_capital` (523). This is the single most load-bearing type-inventory fact for the
blocker table (§6) — see the ruling cited there.

**`tick_dynamics.county_states` value shape is a live Pydantic object, not a dumped dict** (§2 Step 3b),
in contrast with every other graph-metadata read in this system (`market`, `market_county`,
`national_financial`, which are all `.model_dump()`'d dicts read back with `.get()`). `getattr(...,
"surplus_distribution", None)` at `market_scissors.py:562` is attribute access, not dict access — a
genuine, verified inconsistency in HOW graph-metadata values are represented across the codebase
(dict-of-dumps vs. dict-of-live-objects), worth transcribing exactly as found (port-as-is), not
unifying.

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 except the ONE transcendental below.

**Libm hazard — exactly one, and it IS reached by this system's own execution path:**
`math.log(ratio)` at `domain/economics/monetary/anchor.py:89`, called from
`_read_fictitious_anchor` → `fictitious_anchor` (`market_scissors.py:148`). Grep-confirmed: zero
`exp`/`sigmoid`/`pow` calls anywhere in `market_scissors.py` or `formulas/market.py`.
`calculate_scissors_balance`'s `math.tanh` (`formulas/market.py:107`) exists in the SAME module but is
**never called by this system** — its sole caller is `ContradictionSystem` (`contradiction.py:427-429`,
downstream, position 18.0) reading THIS system's `price_log` output; noted here for honesty but
excluded from this system's own hazard count (see §5/§6). `log` IS a `DECLARABLE_INTRINSICS` member
(`declarations.rs:110`, `["exp","log","floor"]`) — so this is a cross-implementation reproducibility
D-record, not a hard expressibility block: per CLAUDE.md's behavioral-contracts point 4, a libm
transcendental needs "an explicit tolerance policy with a written derivation," not merely a declared
intrinsic.

**Shapes, in execution order (Steps per §2):**
1. **Presence-gated summation** (Step 0): `wages += float(...)`; `value += float(...)` over a
   `sorted`-order filtered scan — deterministic by construction (III.7), same pattern as Territory's
   spillover accumulator.
2. **Non-negative floor:** `max(value - wages, 0.0)` (`market_scissors.py:180`, also 223) — lower-only
   clamp, one subtract.
3. **Growth-drive ratio with epsilon guard:** `sensitivity * (current - previous) / previous`
   (`market.py:58`), guarded by `previous <= 1e-9` → `0.0` (56-57) — a DIVISION whose denominator-near-
   zero case is handled by an epsilon literal (`_GROWTH_EPSILON = 1e-9`, `market.py:32`), not by the
   `E-EVAL-012` div-by-zero path; this is a domain judgment call baked into the formula, not the
   language's own division-by-zero handling.
4. **Semi-implicit Euler step, two-sided hard clamp with a coupled side-effect:**
   `acceleration = drive - reversion*x - damping*v; new_v = v + acceleration; new_x = x + new_v`
   (`market.py:87-89`), then `new_x > bound → (bound, 0.0)`, `new_x < -bound → (-bound, 0.0)`, else
   `(new_x, new_v)` (90-94) — **the clamp zeroes velocity as a side effect of hitting either rail**,
   a different clamp SHAPE from every other clamp in this system (see item 8 below for the inventory of
   all distinct shapes).
5. **EMA blend:** `alpha*value + (1.0-alpha)*previous` (`market.py:44`) — one multiply-add pair, `1.0`
   a bare literal (see "bare literals" below).
6. **Linear combination of three drive terms:** `growth_drive + momentum_coupling*price_velocity +
   anchor_pull_term` (`market_scissors.py:298-304`) — plain sum, no clamp.
7. **Serviceable-divergence floor with two independent Option-gated terms:**
   `max(base + (0.0 or slope*max(profit_rate,0.0)) - (0.0 or interest_slope*max(interest_burden,0.0)),
   0.0)` (`market.py:145-147`) — a lower-only OUTER clamp wrapping two lower-only INNER clamps, each
   independently `None`-gated to `0.0` rather than propagating absence.
8. **Overhang floor:** `max(fictitious_log - serviceable, 0.0)` (`market.py:158`) — lower-only, one
   subtract.
9. **Correction snap:** `log_ratio * (1.0 - severity)` (a scale-down, no clamp needed since
   `severity ∈ [0,1]` by construction) plus `min(velocity, 0.0)` (`market.py:176`) — **one-sided
   UPPER clamp on velocity only** (kills upward momentum, leaves downward momentum untouched
   — a deliberately ASYMMETRIC clamp, the opposite asymmetry from every `min(x, 1.0)` elsewhere in
   this system).
10. **Correction severity, nested nested clamp:** `min(max(base + slope*max(debt_ratio,0.0), 0.0),
    1.0)` (`market.py:201`) — a two-sided OUTER clamp (`[0,1]`) wrapping a lower-only INNER clamp on
    the debt term.
11. **Evaporation fraction, upper-only clamp:** `min(evaporation_gain * overhang, 1.0)`
    (`market_scissors.py:424`) — followed by a REDUNDANT `if fraction <= 0.0: return` guard (425):
    since `evaporation_gain ∈ [0,0.5]` (Field-enforced) and `overhang > 0` is guaranteed at every call
    site (Step 3 already returned early at `overhang <= 0.0`), `fraction <= 0.0` can only be exactly
    `0.0` when `evaporation_gain == 0.0` — a defensive branch that is reachable only through a
    zero-coefficient mod, not through any data condition.
12. **Reserve-ratio bump, upper-only clamp:** `min(base + influx, 1.0)` (`market_scissors.py:460`) —
    same shape as item 11's clamp, same redundant-guard pattern at line 449.
13. **Real→Int: none genuine.** `int(tick)` appears at five call sites
    (`market_scissors.py:172(implicit via `tick = context.tick`)/185/195/198/200/`_step_county_axes`'s
    `int(tick)` param). `TickContext.tick: int = 0` (`engine/context.py:48`) is ALREADY an `int` —
    every `int(tick)` here is a no-op int→int cast, not a truncating Real→Int demotion. **This is a
    genuine difference from `TerritorySystem`**, whose `int(current_pop * displacement_rate)` was a
    real truncating cast; nothing in `market_scissors.py` performs an analogous truncation.
14. **Clamp-implementation census (five distinct shapes in one 582-line file):** (i) two-sided with a
    coupled velocity-zeroing side-effect (`calculate_scissors_step`); (ii) lower-only, `None`-gated
    independently per term, nested (`calculate_serviceable_divergence`); (iii) lower-only, single term
    (`calculate_overhang`); (iv) one-sided upper-clamp-on-velocity-only, asymmetric by design
    (`calculate_correction_snap`); (v) two-sided outer / lower-only inner, nested
    (`calculate_correction_severity`); plus two more upper-only `min(x, 1.0)` clamps directly in
    `market_scissors.py` (evaporation fraction, reserve-ratio bump) that match neither of the two
    `formulas/market.py` upper-only shapes exactly (they have no paired lower guard at all, relying on
    an external non-negativity invariant instead). **Unlike Territory's two-clamp inconsistency
    (`_write_clamped` vs. a hand-written `min`), this system never uses `SystemBase`'s
    `_write_clamped` helper at all** — every clamp here is hand-written, and the five+ shapes above are
    NOT interchangeable; a port must transcribe each faithfully (port-as-is law), not unify them.
15. **Bare non-integer literals — pervasive, the same BSL-parser concern the Territory inventory
    flagged.** `0.0`/`1.0` appear as bare literals at (non-exhaustive, one per distinct role):
    `market_scissors.py:180` (`0.0` floor), `188-195` (seed state, six `0.0`s), `424`/`460` (`1.0`
    clamp ceiling), `436` (`1.0 - fraction`); `market.py:44` (`1.0 - alpha`), `56` (`1e-9` epsilon —
    itself a MODULE-LEVEL named constant, `_GROWTH_EPSILON`, not re-literalized at each call site —
    the one place this file already does what BSL's `defconst` pattern would require), `90/92` (the
    `max_abs_log` bound compared against, itself defines-sourced, so NOT bare), `147/158` (`0.0`
    floors), `176` (`1.0 - severity`, `0.0` velocity floor), `201` (`0.0`/`1.0` clamp bounds). None of
    these are `defconst`-declared today (this is Python, not BSL) — every one becomes either a
    `c`-suffixed scaled literal or the Real-zero-promotion idiom at port time, exactly Territory's
    finding, just at higher volume (this system has roughly 3× Territory's literal count for its
    smaller line count, because oscillator math is clamp-and-blend-heavy).
16. **Style inconsistency, not an arithmetic hazard:** the `.graph` direct-dict-mutation
    (`metadata[MARKET_ATTR] = ...`, line 199) vs. `graph.set_graph_attr(...)` (line 386) both reach
    the identical `_graph_attrs` dict (`topology/graph.py:338-340,892-898`) through two different
    code paths within this one 582-line file — recorded per port-as-is honesty, not a bug.

## 5. CROSS-SYSTEM CHANNELS

**Tick position: 17.8** (`market_scissors.py:158`), confirmed against `_SYSTEM_CLASSES`
(`simulation_engine.py:328-363`): `... → AllegianceSystem(17.42) → ElectoralSystem(17.45) →
PolicySystem(17.47) → SovereigntySystem(17.5) → MarketScissorsSystem(17.8) → ContradictionSystem(18.0)
→ ContradictionFieldSystem(19.0) → ... → WealthDistributionSystem(21.5) → EpistemicHorizonSystem(22.0)`.

**Reads from prior same-tick systems:**
- `w_paid`/`v_produced` — written ONLY by `ImperialRentSystem` (`economic.py`, position **9.0**), on
  the WAGES edge's TARGET node, only on ticks the edge actually fires (`economic.py:513-531`; both
  endpoints active AND `bourgeoisie_wealth > 0`). Grep-confirmed the only writer repo-wide
  (`rg -n 'w_paid\s*=|v_produced\s*='`). In every canonical scenario the WAGES-edge target is always a
  `social_class` node (Labor Aristocracy), never a territory — so `_aggregate_wage_value`'s unfiltered
  `query_nodes()` scan finds these attrs on SOCIAL_CLASS nodes only in practice, though the code makes
  no such assumption.
- `national_financial` — written ONLY by `TickDynamicsSystem` (`domain/economics/tick/system/__init__.py`,
  position **4.0**), via `write_national_financial_state_to_graph` (`graph_bridge.py:433-450`).
- `tick_dynamics` — written by the same `TickDynamicsSystem` @4.0, via `write_tick_state_to_graph`
  (`graph_bridge.py:83-104`). **Both of these are gated by the annual-boundary check**
  `if tick % WEEKS_PER_YEAR != 0: self._accrue_flows(graph); return`
  (`domain/economics/tick/system/__init__.py:174`, `WEEKS_PER_YEAR = 52`,
  `defines.yaml:374`/`formulas/constants.py:37`) — on a non-boundary tick, `TickDynamicsSystem` returns
  WITHOUT calling either write function at all, so these two graph-metadata keys are only ever
  (re-)populated on boundary ticks. **For the `qa:regression` harness specifically** (not the
  headless-runner harness — the two differ, see the dormancy note below), the harness's own tick loop
  is `for tick in range(1, max_ticks + 1)` (`tools/regression_test.py:1054`) and the harness's own
  documented understanding (`tools/regression_test.py:497-505`, `_dense_row`'s docstring) is that "the
  annual pipeline fires exactly once per 52-tick run, on the first `step()` call, since `context.tick`
  is the pre-increment `state.tick`" — i.e. the boundary fires at the START of a `qa:regression` run,
  not the end, so both keys are populated from the FIRST tick onward for the whole 52-tick canonical
  run (a materially different — and better — picture than a naive `tick % 52 == 0` reading would
  suggest for a 52-tick run).
- `tick_capital_stock` — written by `TickDynamicsSystem` @4.0 (`graph_bridge.py:175`) and re-written by
  `PolicySystem` @17.47 (`policy.py:759`, moving capital between territories) — the LATTER runs
  immediately before this system in the same tick, so `_mean_ratio_to_capital`'s `tick_capital_stock`
  read is the freshest same-tick value.
- `tick_accumulated_debt` — written ONLY by `TickDynamicsSystem` @4.0 (`graph_bridge.py:275`); no
  Consequence-phase system re-touches it before 17.8.
- `median_wage` — potentially written by `ReserveArmySystem` @5.0 (`reserve_army.py:104-106`), but
  ONLY if `median_wage` was ALREADY `> 0.0` going in — see the dormancy finding below; on every
  canonical scenario this write never fires either.

**Writes consumed downstream (this-tick and cross-tick):**
- `market` (national axis) — read same-tick by `ContradictionSystem` @18.0
  (`contradiction.py:425-429` → `market_balance` via `calculate_scissors_balance`, feeding the
  CANONICAL `price_value` opposition, `dialectics/instances/catalog.py:529-541`; `contradiction.py:446-455`
  → `financialization_index` via `math.exp(clamp(fictitious_log, ±max_abs_log))` — **a second libm
  transcendental, `math.exp`, reached ONLY by the downstream consumer, never by this system itself**).
  Grep-confirmed: no other `engine/systems/*.py` reads the `"market"` graph attr.
- `market_county`/`price_divergence` — grep-confirmed read by NO other System; `price_divergence` is
  read only by `projection/veil.py` (the `observe()`-page/AI-narrative layer), an out-of-tick consumer.
- `market_correction_shock` — consumed exactly once, by `WealthDistributionSystem` @21.5
  (`wealth_distribution.py:150-158`, `_consume_market_shock`): pops the stamp, computes
  `kick = kick_gain * overhang` (`kick_gain = defines.market.wealth_axis_kick_gain`, 0.02, `[0,0.1]`,
  `defines.yaml:1009`), applies `w1 -= kick; w2/w3/w4 += kick/3` to the national wealth-share ODE's
  VELOCITY vector — Σimpulses = 0, conservation-preserving (spec-114 FR-114-4 impulse form). **This is
  the "shock-stamp production consumed at 21.5" the assignment asked me to catalog.**
- `SOCIAL_CLASS.wealth` (evaporation write) — SAME-tick downstream readers: `ContradictionSystem`
  @18.0 (`contradiction.py:204,213,403,528,881`) and `ContradictionFieldSystem` @19.0
  (`contradiction_field.py:115,150`, feeding `_previous_wealth`/velocity tracking) both run AFTER
  17.8 and see the evaporated value THIS tick. Every other `wealth` reader in the engine
  (`allegiance.py`@17.42, `decomposition.py`@11.0, `dispossession_events.py`@10.0, `economic.py`@9.0,
  `ideology.py`(ConsciousnessSystem)@17.0, `production.py`@3.0, `struggle.py`@16.0, `survival.py`@15.0,
  `vitality.py`@1.0) runs BEFORE 17.8, so they see either the PRIOR tick's evaporated value or none —
  never this tick's fresh evaporation.
- `TERRITORY.reserve_ratio` — the intended downstream reader is `ReserveArmySystem` @5.0
  (`reserve_army.py:75`), which runs at 5.0 < 17.8 — so any bump this system writes is read NEXT
  tick, not this one, matching the module docstring's own claim ("@5 system converts the ratio into
  wage pressure NEXT tick," `market_scissors.py:446`).
- `EventType.MARKET_CORRECTION` — consumed by `game/chronicle_adapter.py:177` (narrative layer),
  `models/event_severity.py:958` (classified `"warning"`) /`:1092`; no engine System subscribes to it.

**Context/service usage with no BSL equivalent:** none beyond `services.defines.market` (a defines
lookup, already a portable pattern) and `services.event_bus.publish` (the event-emission gap already
declared systemic — see §6). This system does not read `TickContext` beyond `.tick`, and does not call
`self._wrap_graph`/`self._read` (§1).

**DORMANCY on canonical scenarios** (`tools/regression_scenarios.py`, `SCENARIO_COVERAGE_DATA`):
- **National axis (Step 2) is demonstrably LIVE**, unlike Territory's near-total dormancy. Explicit
  positive claims: `imperial_circuit` — `market_correction` event fires
  ("the national price-value scissors axis snaps a correction once its divergence threshold is
  crossed," `regression_scenarios.py:368-373`); `two_node` — `market` `state_presence`, "the axis is
  live" though the correction never crosses threshold there (`regression_scenarios.py:789-795`);
  `single_county` — `market` `state_presence`, "price-value axis live over a county-bearing graph"
  (`regression_scenarios.py:2205-2209`). No scenario in `SCENARIO_COVERAGE_DATA`'s `at_rest` blocks
  declares any market/price/financialization/reserve-ratio channel dead (grep-confirmed, zero hits for
  `"channel": "*market*|*price*|*financializ*|*reserve_ratio*|*overhang*"`).
- **`_swell_reserve_army` (Step 5) is structurally DEAD on the entire canonical estate.** `median_wage`
  defaults to `0.0` on the `Territory` Pydantic model (`entities/territory.py:216-219`); grep-confirmed
  NO scenario factory (`_legacy.py`, `single_county.py`) ever constructs a `Territory(...)` with a
  non-zero `median_wage`; the only production writer of a territory-node wage attribute,
  `graph_bridge.py:192`, writes the DIFFERENTLY-NAMED `tick_median_wage` (prefixed), never the plain
  `median_wage` this system (and `ReserveArmySystem`) actually reads. `_swell_reserve_army`'s own gate
  (`wage <= 0.0 → continue`, `market_scissors.py:456`) is therefore never satisfied on any canonical
  scenario, on any tick — a genuine, previously-unnamed dead-channel-by-naming-mismatch, exercised only
  by the hand-built unit fixture `test_snap_swells_the_reserve_army_where_wages_exist`
  (`tests/unit/engine/systems/test_market_system.py:430-435`).
- **Per-county axis (Step 7) liveness is scenario-dependent on `county_fips` presence**, which the
  same file's OTHER comments (lines 406, 419, 432, 445, etc., for `imperial_circuit`) mark absent
  ("county-free scenario: no territory carries `county_fips`"); `single_county` is the one canonical
  scenario that DOES carry a real county (Wayne, FIPS 26163), so `market_county`/`price_divergence`
  are plausibly live there and structurally dormant on `imperial_circuit`/`two_node` — this inventory
  did not independently trace whether the WAGES-edge target social_class node in `single_county` itself
  carries `county_fips` (a further, unverified step); flagged here as UNVERIFIED rather than asserted.
- **The two harnesses disagree on annual-boundary timing**, and this matters for whether
  `_mean_profit_rate`/`_national_serviceability` (Step 3a/3b) see real data: the headless runner
  (`engine/headless_runner/runner.py`, used for `detroit_tri_county`'s 5-tick dense golden) numbers
  `context.tick` in `{1,2,3,4,...}` with "tick 0" a pre-engine persist-only row
  (`regression_scenarios.py:2059-2063` comment), so a short headless run NEVER crosses a 52-tick
  boundary and `NATIONAL_FINANCIAL_ATTR`/`tick_dynamics` stay entirely unpopulated; the
  `qa:regression` harness (`tools/regression_test.py`, used for `imperial_circuit`/`two_node`/
  `single_county`'s 52-tick baselines) fires the boundary on its first `step()` call per its own
  docstring (above), so the SAME two graph-metadata keys are live from tick 1 onward there. This
  inventory did not re-derive the discrepancy from first principles (it is stated in each harness's
  own code comments, not independently proven by running anything) — recorded as a documented fact,
  flagged UNVERIFIED-BY-EXECUTION per the read-only constraint.
- **`capitalization_rate`** (`config/defines/market.py:113-122`, `defines.yaml:1000`) is declared,
  documented ("K = s_ema / r, Capital Vol. III ch. 29"), and **never read by any call site anywhere in
  the repository** (grep-confirmed zero hits for `.capitalization_rate` outside its own declaration and
  an unrelated `capital_vol3.housing_capitalization_rate_default` field). A dead coefficient, the same
  class of finding as Territory's dead AUTO-mode defines — a WS4 adjudication candidate (bless as
  reserved, or retire), not a port blocker for this system's own arithmetic.

## 6. BLOCKER ASSESSMENT (adjudicated against the CURRENT BSL surface, dev HEAD 2026-08-12)

| Computation | Verdict | Detail |
|---|---|---|
| Step 0/7 flow aggregation — presence-gated selection (`_aggregate_wage_value`/`_aggregate_wage_value_by_county`, `market_scissors.py:82-129`) | **BLOCKED — no presence semantics** | §3.5.3 of `bsl-language.rst` (2628-2633) rules a binding is either non-optional (load error if any node could lack it) or `:optional` **with a mandatory `:default`** — "there is consequently no `bound?` predicate in the language." This system's selection rule is "does this node carry `w_paid`/`v_produced` THIS TICK" (Python dict-key presence), which has no BSL analogue: an `:optional` field with a default is ALWAYS readable, indistinguishable from "never touched." Not a query-evaluation gap (Slice 1 IS landed for `fold`/`nodes`); it is a closed design ruling excluding the exact semantics every read site in this system depends on. |
| Step 0 grouping by `county_fips` (`_aggregate_wage_value_by_county`, 105-129) | **BLOCKED — no group-by fold** | Beyond the presence gap above: this helper builds a `dict[str, tuple[float,float]]` keyed by a DYNAMIC, open-domain string field. The landed `fold` (Slice 1) is a single-key SCALAR reduction (sum/mean/min/max/count) over one filtered node set — not a multi-key partition. §3.6's own ruling (2684-2688) anticipates per-county state as "ordinary nodes of ordinary types," which would sidestep this (fold once per county-carrier node instead of grouping dynamically) — but no such carrier exists live in production (see next row). |
| National/per-county oscillator STATE storage (`market`/`market_county` graph metadata; every read/write of `MarketState` fields) | **BLOCKED — Slice 2, not built** | `bsl-language.rst` §3.6 (2650-2689) RULES this exact case — it names "the market-scissors axis" explicitly among the twenty-two frozen systems whose graph-scope state this ruling covers — and specifies the mechanism: a `:ceiling 1` carrier `NodeType`, read via `(field-of (the NodeType/…) …)`, written via `(update-node (the NodeType/…) …)`. The primitive this depends on, `the`, is in `UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:506`, `("the", "slice 2")`), and ADR197 (2026-08-11) confirms Slice 2 is "SCOPED, NOT BUILT" on dev HEAD. The ruling exists; its landing does not. |
| `national_financial`/`tick_dynamics` graph-metadata reads (`_mean_profit_rate`, `_national_serviceability`, `market_scissors.py:463-582`) | **BLOCKED — same Slice-2 gap, doubly so** | Same carrier-node/`the` dependency as above, PLUS these particular reads are of NESTED structured values (`.endogenous_interest.profit_rate_ceiling`; `.county_states[fips].surplus_distribution.total_surplus_produced`) that a flat `deffield` set cannot represent even once a carrier node exists — and these two graph-metadata keys are produced by `TickDynamicsSystem` (@4.0), itself not yet ported, so this row is also gated on a system this inventory did not scope. |
| Oscillator arithmetic itself (Step 2/3, `calculate_growth_drive`/`calculate_scissors_step`/`calculate_ema`/`calculate_anchor_pull`/`calculate_serviceable_divergence`/`calculate_overhang`/`calculate_correction_snap`/`calculate_correction_severity`) | **PORTABLE WITH D-RECORD (once a storage home exists)** | Pure scalar arithmetic — multiply/add/subtract/divide-with-epsilon-guard, five distinct clamp shapes (§4 item 14, each transcribed faithfully, not unified), one declared-intrinsic `log` call. Every non-integer literal (§4 item 15) becomes a `defconst`/scaled-literal D-record, same class as every landed pack. This row is arithmetically ready; it is BLOCKED only transitively, by the storage-home row above. |
| `math.log` in `_read_fictitious_anchor`/`fictitious_anchor` (`anchor.py:89`) | **PORTABLE WITH D-RECORD** | `log` IS `DECLARABLE_INTRINSICS` (`declarations.rs:110`). Needs the cross-implementation tolerance-policy D-record per CLAUDE.md's behavioral-contracts point 4 (libm transcendentals do not reproduce byte-identically across implementations) — not a hard block. |
| `role`-based bracket lookup (`_evaporate_wealth`, `market_scissors.py:427-436`; `bracket_of_role`) | **PORTABLE WITH D-RECORD** | `SocialRole` is a landed-enum-lane candidate (ADR195/196) — `deffield role enum SocialRole`, `:field`-bound reads, `=`-compared guards. The bracket-fold lookup table (8→4 mapping, unrelated to declaration ordinal) needs re-expression as an explicit chain of `=` comparisons (2 true / 6 false for `_CLAIM_HOLDER_BRACKETS`), a content-modeling decision with its own D-record, not a language gap — `field-of` stays refused for enum fields (D102) but `:field` binding + guard suffices here. |
| `_swell_reserve_army`'s `median_wage`-gated write (`market_scissors.py:438-460`) | **PORTABLE WITH D-RECORD, but structurally provable-dead** | The arithmetic is trivial (a presence gate — same BLOCKED class as row 1 — feeding an upper-clamped add). Since NO canonical scenario ever seeds a non-zero `median_wage` (§5), this branch is a candidate for the same "provably uniform, never fires in production" D-record class Metabolism's D-2 and the Territory inventory's `displacement_mode` used — a port could declare it dead-by-construction rather than porting the live-but-unreachable logic, PENDING a Director/owner ruling on whether that is honest transcription or silent scope-shrink (this inventory does not adjudicate that call). |
| `EventType.MARKET_CORRECTION` emission (`market_scissors.py:397-411`) | **BLOCKED — WS1 (#502)** | `TickReport` carries no event log; every `EventType` emission is an unpinnable WS1 ledger row per the task's standing brief, matching the Territory inventory's own event-emission treatment (there: zero events; here: one, same disposition). |
| `_project_price_divergence`'s per-territory county lookup (`market_scissors.py:245-265`) | **BLOCKED — transitive + its own keyed-lookup shape** | Depends on the `market_county` dict's existence (blocked above); independently, it is a "read a value out of a dict keyed by THIS node's own field" pattern with no landed query-lane analogue (not `field-of` over a fixed `NodeRef`, not a `fold`/`select-*` over a query) — would need re-derivation once/if a per-county carrier-node design lands. |

**RESERVED-LINE check:** none found. This system's entire surface is Capital Vol. III price/credit/
crisis mechanics (price⟷value, fictitious capital, correction snaps, the reserve army) — no doctrine
content, no National Question parameter, no outcome definition is read or written anywhere in
`market_scissors.py` or its formula module. The `SocialRole`/wealth-bracket fold it touches
(`_CLAIM_HOLDER_BRACKETS`) is ADR075's ratified wealth-bracket correspondence, a settled non-ideological
data mapping, not the National Question's B+C+I partition.

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_market_system.py` | 658 | **Primary conformance-oracle candidate.** 11 test classes covering wiring/tick-order (`TestWiring`), honest-absence (`TestHonestAbsence`), seeding (`TestSeeding`), oscillator dynamics + determinism (`TestDynamics`), round-trip (`TestRoundTrip`, `TestRoundTripFrozen`), the per-county axis (`TestCountyAxis`, 8 tests incl. `test_price_divergence_is_a_declared_transient_field`), the correction (`TestCorrection`, 12 tests incl. cooldown, debt-adjusted severity, evaporation, reserve-army swell, shock stamp), docstring-honesty regression tests (`TestMarketScissorsDocstringHonest`), and the D1 anchor (`TestAnchorAbsentIsBitIdentical`, `TestAnchorPresentPullsTheOscillator`). |
| `tests/unit/formulas/test_market.py` | 245 | Formula-level conformance oracle — one `TestX` class per pure function in `formulas/market.py` (10 classes: `TestServiceableDivergence`, `TestServiceableDivergenceInterestBurden`, `TestOverhang`, `TestCorrectionSnap`, `TestScissorsStep`, `TestGrowthDrive`, `TestEma`, `TestBalance`, `TestCorrectionSeverity`, `TestAnchorPull`) — the closest thing to a per-computation golden this system has, and the natural template for BSL `.bscn` conformance vectors' expected values. |
| `tests/unit/formulas/test_market_calibration.py` | 200 | `TestRestoration` + `TestCorrectionDiscipline` — behavioral-contract-style tests (restoring-force convergence, correction-discipline invariants) rather than exact-value pins; closer to a property test than a golden. |
| `tests/unit/config/test_market_defines.py` | 118 | `TestPhase2Defaults`, `TestPhase2Bounds`, `TestCorrectionLedger`, `TestU6Coefficients` — pins `MarketDefines`' default values and `ge`/`le` bounds; the schema-level oracle for §2's "(e) defines consumed" table. |
| `tests/unit/economics/tick/test_financial_state_consequence_roundtrip.py`, `test_u9_9_national_financial_layer_propagation.py` | — (not counted; adjacent, not this system's own) | Exercise `national_financial` round-trip and propagation — relevant CONTEXT for §5's cross-system channel but test `TickDynamicsSystem`'s producer side, not `MarketScissorsSystem` itself. |
| `tests/unit/engine/test_event_conversion.py`, `test_system_order.py`, `test_graph_context_financial.py`, `test_wealth_distribution_system.py` | — | Peripheral: event dict→object conversion (incl. `MARKET_CORRECTION`), tick-order regression, financial-context plumbing, and the shock-stamp CONSUMER's own test file respectively — none are this system's primary oracle but each pins one cross-system fact §5 relies on. |

**No property/law test file exists for Market Scissors** — unlike `TerritorySystem`'s
`tests/unit/engine/laws/test_law_territory_system.py`. Grep-confirmed: `tests/unit/engine/laws/`
contains no `*market*` file, and no `tests/contract/`/`tests/integration/` file references
`MarketState`/`market_scissors` at all. This system's behavioral contracts live entirely in
`test_market_system.py` (unit-level, System-in-isolation) and `test_market.py`/
`test_market_calibration.py` (formula-level) — a narrower ORACLE-TYPE spread than Territory had
(Territory additionally had property-based invariant laws and an AI-contract-surface test), though
`test_market_system.py` alone (658 lines) is the single largest test file either system has.

**`qa:regression` byte-gate coverage is WEAKER for this system than for Territory, by design, not by
gap.** `graph_content_hash` (`tools/regression_test.py:924-964`) hashes every node/edge attribute of
the `WorldState→graph` projection but **explicitly excludes graph metadata** — the docstring states
plainly (939-943): "Graph *metadata* (`g.graph`: economy, event log, opposition states) is also
excluded, because the spec's field set is nodes/edges/actions." Since this system's entire oscillator
STATE (`price_log`, `fictitious_log`, both velocities, both EMAs, `corrections`,
`last_correction_tick`) lives ONLY in graph metadata, **none of it is covered by the byte-identical
hash gate** — only its node-level SIDE EFFECTS are: `TERRITORY.price_divergence`,
`SOCIAL_CLASS.wealth` (evaporation), `TERRITORY.reserve_ratio` (dormant per §5). `CheckpointData`
(`tools/regression_test.py:284-297`) likewise carries no `market`-related field at all — only four
hardcoded entities' `wealth`, `imperial_rent_pool`, `exploitation_tension`, and one entity's
consciousness/p_revolution. A port's conformance oracle for the oscillator core therefore rests
ENTIRELY on `test_market_system.py`/`test_market.py`'s own pinned values, not on the canonical
`qa:regression`/dense-golden estate — a genuinely different and more fragile oracle situation than
Territory's (whose full node/edge attribute set WAS hash-covered, even where dormant).

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`), read-only, with fresh anchors.
The computation catalog, float-op census and channel table are the most thorough in this
train and survive checking. **Both of the verdict's two named blockers are corrected** — one
is not a blocker at all, the other is the right instinct pointed at the wrong ruling — and
the surviving blockers are a different set.

1. **CORRECTION — `the` is NOT required for the §3.6 carrier, and graph-scope oscillator
   state is storable TODAY under landed Slice 1.** §6's storage row and the verdict rest
   entirely on `("the", "slice 2")` (`evaluator.rs:506`). But §3.6 rules the *mechanism* to
   be "an ordinary `deffield` owned by a **carrier node type** — a `NodeType` member whose
   manifest `:ceiling` is 1" (`bsl-language.rst:2664-2668`); `the` is the sugar §3.6 happens
   to write it with, not the only route to it. Three legs, each verified on dev:
   - **Self-read/self-write needs no accessor at all.** `tick.rs:159-181`'s
     `subject_type_of` derives a rule's subject `NodeType` from the namespace of its
     `:field` bindings ("a field is a field OF self's node type"), and `run_tick` iterates
     `graph.nodes(&subject_type)` (`tick.rs:536-538`). A rule whose bindings are all
     `market/*` therefore runs over exactly the carrier population — at `:ceiling 1`,
     exactly one subject — reading with `:field` and writing with `update-node self`. No
     `the`, no Slice 2.
   - **Cross-type reads are a landed fold.** `(fold sum (nodes NodeType/MARKET-AXIS)
     (field-of it market/price-log))` — `nodes` is in `SERVED_QUERY_HEADS`
     (`evaluator.rs:527`) precisely as a fold operand, and `field-of` over a `NodeRef` is
     landed (`evaluator.rs:1197-1223`). At ceiling 1 that fold *is* `(the …)`, semantically.
   - **Cross-type writes are the landed ADR197 shape.** `(update-node (select-max (nodes
     NodeType/MARKET-AXIS) …) …)` is the same "select-max feeding update-node against a
     computed reference" vector Task 15 proved through the real `run_once_into` seam
     (`rust/crates/babylon-tick/tests/query_lane_e2e.rs:229-240`).
   And the carrier `NodeType` is **content-declarable today**, not amendment-blocked in
   code: `(defvocabulary NodeType (…))` is a landed `.bscn` form (`scenario.rs:389-395`,
   `load_defvocabulary` at `scenario.rs:811-850`) already in use on dev
   (`content/scenarios/organization-foundation.bscn:41`). §3.6 does price the member as
   amendment territory (`bsl-language.rst:2668-2671`) — that is governance weight on a
   buildable thing, not a missing primitive.

2. **CORRECTION — the `bound?` row misreads its own citation, and the genuinely blocking
   fact sits one level down.** §6 row 1 reads §3.5 as "a closed design ruling excluding the
   exact semantics every read site depends on." The ruling's text does the opposite of
   excluding: it **prescribes the replacement** — "`:optional` **requires** `:default` …
   requiring the pair removes the need for a dominance analysis over `bound?` guards, keeps
   every expression total, and means no rule ever observes absence — it observes a declared
   default. There is consequently no `bound?` predicate" (`bsl-language.rst:2626-2633`). For
   `_aggregate_wage_value`'s SUM the declared-default route is behaviour-preserving wherever
   `w_paid`/`v_produced` are co-written, and they provably are: `rg -n 'w_paid\s*=|v_produced\s*='`
   over `src/` (tests excluded) finds exactly one production writer, `economic.py:529-530`,
   which writes both in one `update_node` call — the same fact §5 already established for a
   different purpose. That makes this a D-record class, not a design-closed block.
   **The real constraint, which the row missed:** `:optional`/`:default` covers a
   *subject's own* `:field` bindings only (`tick.rs:278-291` — the default fallback lives in
   `bind_subject`). `field-of` over a **fold element** has no optional route whatsoever:
   `evaluator.rs:1274-1292` calls `GraphSubstrate::node_attribute` (`substrate.rs:142`,
   `Result<f64, GraphError>`) and maps absence to `E-EVAL-033` *"absence is not a value"*.
   So every node materialised by a folded query must **carry** the field. That is a
   scenario-seeding obligation with a stated divergence (frozen: skip; BSL: read a seeded
   `0`), not a language refusal — and it is the fact a port's `.bscn` author must be told.

3. **CORRECTION — `price_divergence`'s tri-state is a first-class blocker, not a transitive
   one.** §2 Step 7 records the shape exactly right (present-with-value / present-as-`None`
   / absent; the `elif PRICE_DIVERGENCE_ATTR in attrs` guard at `market_scissors.py:264`
   deliberately never introduces the key fresh), but §6 files
   `_project_price_divergence` only as "transitive + its own keyed-lookup shape". The
   explicit `None` de-position write (`market_scissors.py:265`) has **no representation at
   all**: `deffield`'s type vocabulary is seven scalar rows with no nullable variant
   (`bsl-language.rst:2373`, "The first seven rows — ``int``, ``bool``, ``currency``…"), and
   `node_attribute` is total-or-error, so there is no way to store "positioned but
   undefined" and no way to distinguish it from a seeded `0`. This is the same
   Option-representation gap `FieldDerivativeSystem`'s inventory raised to a named D-record
   for `df_dt`/`d2f_dt2`; here it is unnamed and sits inside a row marked merely transitive.

4. **CONFIRMATION — `_swell_reserve_army` is dead by naming mismatch on the whole canonical
   estate.** Verified independently: `rg -n median_wage` over `src/` and `tools/` (tests
   excluded) finds no scenario factory ever constructing a `Territory(...)` with a non-zero
   `median_wage`; `entities/territory.py:216` declares it `Currency` defaulting `0.0`; and
   the only production territory-node wage write is the differently-named,
   `tick_`-prefixed `tick_median_wage` — which `sentinels/seam/registry.py:783-784` states
   in its own words is *"deliberately kept tick_-prefixed (not 'median_wage') to avoid
   colliding with the real, distinct Territory.median_wage field"*. §5's dead-channel
   finding is correct and is now doubly sourced.

5. **CONFIRMATION — `capitalization_rate` is a dead coefficient.** `rg -n capitalization_rate`
   over `src/` (tests excluded) returns its `defines.yaml:1000` row, its
   `config/defines/market.py:113` declaration, a mention inside
   `market.py:54`'s *docstring for a different field*, and the unrelated
   `capital_vol3.housing_capitalization_rate_default`. Zero call sites. WS4 adjudication row
   as filed.

6. **CONFIRMATION, and the §5 UNVERIFIED item RESOLVED in the affirmative — the per-county
   axis IS live on `single_county`.** §5 flagged as unverified "whether the WAGES-edge target
   social_class node in `single_county` itself carries `county_fips`". It does:
   `single_county.py:82-93` constructs the `LABOR_ARISTOCRACY_ID` `SocialClass` — the
   declared WAGES **target** at `single_county.py:104-111` — with
   `county_fips=WAYNE_COUNTY_FIPS`, as does the `CORE_BOURGEOISIE` at `:70-80` and the
   territory at `:113-120`. So `_aggregate_wage_value_by_county`'s `county_fips is not None`
   gate (`market_scissors.py:121-123`) is satisfied and `market_county`/`price_divergence`
   are genuinely live on that scenario, not merely "plausibly".

7. **CONFIRMATION — the `market_correction_shock` channel and the RESERVED-LINE finding.**
   The stamp is written at `market_scissors.py:386` and consumed exactly once, by
   `wealth_distribution.py:150`'s `metadata.pop(MARKET_CORRECTION_SHOCK_ATTR, None)` at
   position 21.5 (`MARKET_CORRECTION_SHOCK_ATTR` declared in the consumer at
   `wealth_distribution.py:77`) — grep-confirmed no other reader. Tick position 17.8
   confirmed at `market_scissors.py:158` against `_SYSTEM_CLASSES`/`_DEFAULT_SYSTEMS`
   (`simulation_engine.py:328-378`). §6's "RESERVED-LINE: none found" is upheld: nothing in
   `market_scissors.py` or `formulas/market.py` reads doctrine content, a National Question
   parameter, or an outcome definition. (Noted for the record, not as a defect: the
   `tanh`-shaped `calculate_scissors_balance` this system *never calls* is a stipulated
   saturating curve, and its actual caller `ContradictionSystem`@18.0 carries the ADR172
   ruling-5 question — adjudicated on that system's own inventory, not this one's.)

**FINAL VERDICT: BLOCKED — but on a materially different set than the inventory names, and
the national oscillator core is PORTABLE WITH D-RECORDS today.** The two named blockers are
withdrawn: graph-scope state has a landed home (a `:ceiling 1` carrier `NodeType`, reachable
under Slice 1 alone — corrections 1), and presence-gating is a prescribed
`:optional`/`:default` transcription with a seeding obligation, not a design-closed refusal
(correction 2). Steps 0-2, 3, 3a, 4 and 6 — the flow aggregation, the two coupled
damped-driven oscillators, the correction snap, wealth evaporation and the metadata commit —
are transcribable now on a carrier node, with the ordinary D-record slate (five distinct
clamp shapes transcribed faithfully; the `c`-suffixed-literal sweep; `log` as a declared
intrinsic plus its cross-implementation tolerance derivation; the `SocialRole` enum lane).
What remains genuinely blocked: **(i)** the per-county axis (Step 7), whose partition key
`county_fips` is an **open-domain string** and `deffield` has no string type — grouping must
be re-modelled as county carrier nodes plus an incidence edge, a real content redesign, not
a transcription; **(ii)** `price_divergence`'s tri-state, which has no representation at all
(correction 3); **(iii)** Steps 3a/3b's **nested structured** reads of
`national_financial`/`tick_dynamics`, whose producer `TickDynamicsSystem`@4.0 is unported and
whose `county_states` values are live Pydantic objects rather than attribute-shaped data;
**(iv)** `MARKET_CORRECTION` on the standing WS1 (#502) ledger.

**INADEQUATE-COVERAGE — a re-read must add:**
(a) the §3.6-carrier adjudication above, replacing the `the`-dependency everywhere it
appears (executive summary, verdict, §6 storage rows 3 and 4);
(b) `rust/crates/babylon-bsl/src/tick.rs` `subject_type_of`/`run_tick`
(`:159-181`, `:524-560`) and `scenario.rs`'s `defvocabulary` path (`:389-395`, `:811-850`) to
the reference-sources list — the inventory read `evaluator.rs`/`query.rs`/`bsl-language.rst`
§3.5-§3.6 but never the file that decides what a rule's subject population *is*, which is
what makes the carrier route work;
(c) a first-class §6 row for the `price_divergence` Option gap;
(d) the `field-of`-in-a-fold seeding obligation (`substrate.rs:142` / `E-EVAL-033`) as the
replacement for the `bound?` framing;
(e) the resolution of §5's two UNVERIFIED flags — one is closed above (correction 6); the
other (the two harnesses' annual-boundary timing) remains open and is worth closing, since
Steps 3a/3b's liveness on the canonical estate turns on it.
