# T6 synthesis — TickDynamics dormancy re-read + ServicesProtocol charter

**Issue:** #563 (Program 29 train T6) · **Charter:** ADR198 · **Date:** 2026-08-14
**Inputs:** the six per-scenario re-read memos in `reports/t6-dormancy/`
(`single_county`, `mitterrand`, `syriza`, `bernie_valve`, `detroit_tri_county`,
`michigan_canada_e2e`). System under re-read: `TickDynamicsSystem` @4.0
(`src/babylon/domain/economics/tick/system/__init__.py:112`). Method: read-only
source audit + committed golden inspection; nothing executed. This synthesis
corrects two inter-memo disagreements by direct re-verification (§2.4).

## 1. The premise verdict — "tick 52" is REFUTED as stated

The issue's framing — six county-bearing scenarios "driving the annual pipeline
at tick 52" — survives contact with the evidence for exactly one scenario:

| Scenario | Harness | When the annual pipeline actually fires |
|---|---|---|
| `single_county` | qa (`simulation_engine.step`) | **Context tick 0** — the FIRST `step()` call; once per run |
| `mitterrand` | qa | **Context tick 0** — golden bytes: `financial_*` 0.0 at tick 0, full values flat from tick 1 |
| `syriza` | qa | **Context tick 0** — same golden proof |
| `bernie_valve` | qa | **Context tick 0** — same golden proof |
| `detroit_tri_county` | headless runner | **NEVER in any committed artifact** — 5t/3-tick gates run engine ticks {1..4}, all diverted by `tick % 52 != 0`; tick 52 is a counterfactual needing `--ticks ≥53` |
| `michigan_canada_e2e` | headless runner, 520 ticks | **CONFIRMED** — fires 9× at ticks 52…468 (half-open range excludes 520), years 2011–2019 |

Mechanism: the qa harness builds `TickContext(tick=state.tick)` from the
**pre-increment** tick (`src/babylon/engine/simulation_engine.py:566-569`), so a
52-call loop presents context ticks 0..51 and the only multiple of 52 is 0. The
runner persists tick 0 without the engine (`runner.py:1660-1668`) and loops
`range(1, config.ticks)` (`runner.py:1694`). Both facts are documented in-repo
(`simulation_engine.py:474-485`; `tools/regression_test.py:497-506`) — the
issue's premise simply never read them.

**Companion finding:** the withdrawn blanket-dormancy premise (the county-free
pattern at `tools/regression_scenarios.py:406-451`) does not apply to ANY of the
six. All six stamp real `county_fips` on territories and classes; `county_states`
is non-empty at every boundary. Dormancy in this estate is **wiring-driven, not
substrate-driven**.

## 2. The real dormancy ledger

TickDynamics can touch 33 `ServicesProtocol` fields (28 direct reads in
`system/__init__.py` + 5 Spec-057 fields in `imperial_rent.py`; protocol list
`src/babylon/kernel/services.py:34-88`). Three wiring classes organize them.

### 2.1 Class A — unwired in EVERY scenario (scenario-invariant)

Seven fields are `None` in both harnesses: `basket_calculator`,
`capital_calculator`, `throughput_calculator`, `employment_source`,
`housing_source`, `income_source`, `hex_grid`.

Headline kill-chain: `capital_calculator` unwired ⟹ `capital_stock ≡ 0.0`
(`system/__init__.py:747-751`) ⟹ the entire **Feature-023 per-county circulation
layer early-returns on `capital_stock <= 0`** (`:1603-1605`) — even in
michigan-e2e, where `turnover_profile_source` IS wired but sits *after* the early
return and is never reached. The circuit advance, inventory, depreciation fund,
and reproduction schema (`:1631-1732`) execute in **no gated run, anywhere**.
The flat `employment = 100_000.0` default (`:793-797`) silently equalizes every
employment-weighted aggregate in all six scenarios.

### 2.2 Class B — harness-dependent

- **qa four** (single_county/mitterrand/syriza/bernie_valve): 9–11 fields live
  (melt, tensor, distribution, fictitious, credit, assessor, rent/housing as
  sentinel paths, defines/fallbacks/event_bus). Unwired: `reserve_army_data_source`,
  `dispossession_data_source`, `transition_engine`, `turnover_profile_source`,
  the Spec-057 five ⟹ Vol I wage pressure, Vol II circulation, Feature-016
  transitions, and the Leontief imperial-rent pipeline are all dormant; one
  `CALIBRATION_QCEW_CARRY_FORWARD` sentinel event per run is the only emission.
- **runner two** (detroit counterfactual / michigan live): the full estate wired
  except Class A — 25 of 33 live in michigan-e2e.

### 2.3 Class C — wired but never read by TickDynamics in ANY scenario

`interest_calculator` (calibration-only since U9, `:1763-1765`),
`credit_cycle_detector` (the `credit_cycle_phase: "expansion"` graph attr is a
**hardcoded literal**, `graph_bridge.py:106`), `counter_tendency_calculator`,
`value_basis_converter`; `z1_source`/`housing_data_source` are consumed one level
down inside calculators, never at this boundary. `ThresholdCrisisDetector`
(`_legacy_crisis_detector`, constructed `:129`) is **dead construction** — never
called anywhere.

### 2.4 Live-but-dead-end channels (computed, never consumed)

- **`flow_wage_accrued ≡ 0.0` everywhere** — RE-VERIFIED for this synthesis:
  `_accrue_flows` reads `tick_employment` with default **0.0**
  (`system/__init__.py:343`), and `tick_employment` has **no write site
  repo-wide** (grep-verified by three memos independently). The mitterrand and
  bernie_valve memos' "84,000,000.0/tick" figure is a misread — it applies the
  100_000.0 default from the *bootstrap/read-back* sites (`:506`,
  `graph_bridge.py:392`), which never feed the accrual. The accrual's own
  default is 0.0, so the wage counter can never move; `flow_phi_accrued`
  accrues genuinely only in michigan-e2e. Additionally the `flow_*` counters are
  stripped by every `WorldState` round-trip (`world_state.py:253-256`), so the
  within-year cumulative never accumulates on the `step()` path. Sole consumer
  of either counter: the legacy web bridge seam
  (`sentinels/seam/registry.py:2897-2919`); PolicySystem recomputes its Φ slice
  instead (`engine/systems/policy.py:31-35`, ADR135).
- **Precarity triple** (U-6/PTER/NILF) is derived every boundary
  (`precarity.py:44-62`) and has **no consumer** — never stamped, no reader
  outside the module.
- **Two profit rates coexist** on the graph after a boundary: the stamped
  derived `tick_profit_rate ≈ 3.7` (K=0 ⟹ s/v, an exploitation-rate lookalike)
  vs the tensor-realized 0.0594 the financial layer actually used. Consumers of
  the stamped attr get a materially different quantity.
- **`financial_s_r`/`financial_tightness` are computed zeros, not structural
  zeros** (ρ̄=0.05 < `interest_reserve_reference`=0.08 ⟹ clamped). The port must
  reproduce the computation, not the zero.
- **Bifurcation pseudo-identity:** `bifurcation.py:101-104` compares
  `node.id == fips` — `T001`-style ids never equal `"26xxx"`, so the Feature-030
  legitimation blend is unreachable in every runner scenario; legitimation
  always falls to the agitation-inverse branch.
- **Bifurcation `compute()`** never fires within any 52-tick horizon (needs a
  second boundary, tick ≥ 104). In michigan-e2e it fires from the 2nd boundary
  but is structurally NEUTRAL while crisis stays NORMAL (`bifurcation.py:93-94`),
  and the revolutionary (−) term risks being structurally zeroed: SOLIDARITY is
  deliberately never seeded (`bridge.py:842-846`, Constitution III.5/Q4) while
  the burden (+, fascist) term is not.

## 3. Port hazards surfaced (for the port packet, not ruled here)

1. **`round()` half-even ×9 sites.** Python built-in round is half-even over the
   IEEE-754 double; Rust `f64::round` is half-away. Sites: DISPOSSESSION_CASCADE
   payload (`:1164-1167`), BIFURCATION_THRESHOLD payload (`:2336-2340`), and the
   two accumulation-loop headcount roundings **to integer**
   (`reserve_army/accumulation.py:115,121`). Live today ONLY in michigan-e2e
   (the headcounts, every boundary). Event payloads are byte-compared in
   baselines, so the semantic must be SPECIFIED (a D-row), not assumed.
2. **ADR198 R7's leading-zero trap is unexercised by every gated scenario.** All
   Michigan FIPS are 26xxx; no `"06037"`-style county exists in any gate. The
   int-encoded FIPS ruling currently has zero behavioral witness; `"canada"`
   falls under R7's node-identity clause (`scopes.py:149`).
3. **Carry-forward asymmetry.** `_get_best_tensor_year` carries missing years
   forward (financial layer) while `_get_organic_composition` does not
   (accumulation layer) — same registry, two absence semantics, both golden-visible.
4. **Three encodings of "absent".** `None` (unwired service), `NoDataSentinel`
   (wired, data missing), and documented defaults (wired, field missing) each
   drive DIFFERENT downstream behavior and different golden bytes. The port's
   Option/Result types must not collapse them.
5. **Year offset.** The first in-runner boundary computes year 2011, not 2010
   (tick 0 never runs the engine) — the 2010 annual frame is hydration-owned.
   The qa harness's tick-0 boundary computes 2010. Same system, two year
   bases, both golden-pinned.

## 4. The ServicesProtocol charter — VERDICT: one charter, wiring-class-organized

The issue allowed an honest split verdict. The evidence supports ONE charter:
the 33-field surface is coherent; what varies is wiring, and wiring is harness
policy, not protocol shape. The charter's organizing insight is that **the
qa-harness dormancy pattern must NOT be reified as the protocol's contract** —
the qa overrides are a degenerate wiring that happens to be golden-pinned, not
the semantics the port serves.

Charter articles for the TickDynamics port's services boundary:

1. **Surface:** the 33 fields, grouped by the step structure they serve
   (national params / county sources / Vol I / Vol II circulation / Vol III
   financial / Spec-057 Leontief / Feature-016 transitions / always-on:
   defines, event_bus, economics_fallbacks), each with its absent-semantics
   named explicitly per §3.4.
2. **Defaults are behavior.** The documented defaults (employment 100k, U-3
   0.05, wage 21.0, deflator 1.0, the 0.01/0.09/0.40/0.35/0.15 bootstrap
   shares, γ_basket 0.68 / γ_III 0.33) are golden-visible in four scenarios.
   The port either reproduces them byte-exactly or diverges by declared
   ceremony; it may not "fix" them silently.
3. **Computed zeros are pinned as computations** (s_r/tightness, §2.4) —
   mutation-test targets, not constants.
4. **Dead-ends port verbatim or not at all.** The unread fields (§2.3) and
   dead-end channels (§2.4) are carried as documented dormancy or retired on
   the WS4 ledger per register row 24's ruling — no third option.
5. **The gate and the clock are part of the contract:** `tick % 52`, the
   pre-increment TickContext, and the two year bases (§3.5) are semantics the
   port must state once, in one place.

**Director-gated residue (presented, not ruled):** whether the runner's
Class-A unwiring (especially `employment_source` — see §5.1) is deliberate
scope-trimming or an oversight is a data/theory call above the workforce.

## 5. Open questions routed

1. **`employment_source` runner gap (escalate).** The runner wires wage +
   unemployment but not employment, so every county gets the flat 100k default
   at boundaries — materially wrong for a scope whose real counties differ 2×
   in population, and it poisons `reserve_army_signal`'s employment weighting.
   Looks like a data bug, not a design. (detroit memo OQ2; affects michigan-e2e
   goldens.)
2. **No ≥53-tick detroit-tri-county gated artifact exists** — the Class-A
   estate is exercised for Wayne/Oakland/Macomb only by inference from
   single_county (qa) and michigan-canada (520-tick). A 53-tick detroit gate
   would close that gap.
3. **`_WAYNE_WORKER`/`_WAYNE_OWNER` naming inversion** (electoral_goldens.py:46-48
   vs `entity_registry.py:35,38`): the "worker" re-seed lands on the
   core-bourgeoisie id. Invisible to TickDynamics; flagged for the electoral
   estate's owners.
4. **bernie_valve stale docstring** (electoral_goldens.py:14-17 claims a
   two_node substrate; code + goldens prove Wayne single_county) — doc drift,
   flagged not fixed.
5. **michigan-e2e baseline is LFS-unhydrated in this workspace** — crisis-event
   presence, SOLIDARITY emergence, and 2018–2019 phi provenance are UNVERIFIED
   (michigan memo's open questions stand).

## 6. Verdict for #563

The re-read is delivered (six memos + this synthesis); the premise is refuted
as stated and replaced with the wiring-class ledger; the ServicesProtocol
charter is a GO as one document per §4. The charter's implementation (the Rust
trait surface itself) is Wave-work beyond T6's design-doc scope; §5.1 is the
only finding that wants a Director answer before the port's golden semantics
can be fully pinned.
