# Plan — spec-101 Trade Activation

## Approach (data flow)

```
initialize_session
  ├─ _bootstrap_external_nodes  (init-time; sqlite open)
  │    ├─ national Φ (Hickel "Intensive" aggregate, immutable_reference_hickel_drain "Intensive")
  │    ├─ read fact_bilateral_trade_annual (sqlite, start-year annual time_id)
  │    ├─ injective node→bloc crosswalk → per-node trade share
  │    ├─ phi_year_inflow[node] = national_Φ × share[node]      (D3)
  │    └─ bilateral_trade_value[node] = bloc total_trade_usd_millions × 1e6 (D3/FR-101-4)
runner.run (setup, once):
  ├─ exposure_map = load_county_exposure_map(sqlite, start_year, scope_fips)   (D2, bloc-invariant, renorm→1.0)
  ├─ county_exposure_by_external = {node: exposure_map for node in INTERNATIONAL_NODES}
  ├─ external_nodes_phi = {node: phi_year_inflow}  (read tick-0 dynamic_external_node_state)
  └─ auditor.register_invariant("imperial_rent_phi_week_distribution", evaluator)
_tick_loop → _advance_tick (every tick):
  ├─ TickContext(tick, session_id, boundary_flow_register, external_nodes_phi, county_exposure_by_external)
  ├─ engine.run_tick → ImperialRentSystem._invoke_phi_distribution_if_wired → DRAIN_EDGE rows recorded
  └─ bridge.persist_tick → flush register → envelope; auditor(context={boundary_rows, external_nodes_phi})
```

## File changes (Lane E owns `src/babylon/**`, `tests/`, baselines)

| File | Change |
|------|--------|
| `src/babylon/domain/economics/county_exposure.py` (new; written pre-Program-14 as `src/babylon/economics/county_exposure.py`, re-pointed by the domain move `5faedd2f`) | `load_county_exposure_map(*, sqlite_path, year, scope_fips) -> dict[str,float]`; pure, read-only sqlite, scope-renorm; raises on empty. |
| `src/babylon/persistence/postgres_initialization.py` | `_NODE_TO_BLOC` injective crosswalk; `_attribute_national_phi_and_trade(...)` reading `fact_bilateral_trade_annual` from sqlite; `_bootstrap_external_nodes` gains `sqlite_path`, `start_year` already present; sets phi + bilateral_trade_value. |
| `src/babylon/engine/headless_runner/runner.py` | Build exposure map + external_nodes_phi at setup; register conservation evaluator; thread 4 keys through `_tick_loop`→`_advance_tick`→`TickContext`. |
| `src/babylon/engine/headless_runner/bridge.py` | Pass `context={"boundary_rows":…, "external_nodes_phi":…}` to `auditor.audit_end_of_tick`. |
| `src/babylon/persistence/conservation_audit.py` | `phi_week_conservation_evaluator` factory (relative residual). |
| `tests/integration/test_trade_circuit.py` (new) | RED→GREEN: per-tick DRAIN_EDGE for Φ>0 blocs + conservation identity. |
| `tests/unit/economics/test_county_exposure.py` (new) | exposure loader unit tests. |
| `tests/unit/persistence/test_phi_attribution.py` (new) | crosswalk + attribution unit tests. |
| `tests/baselines/detroit-tri-county-5t.json` | R-PROOF re-baseline (external_node_flows populates). |

## Constitution v2.7.0 gate checklist (per §4)

- **III.1 no-magic-numbers** — PASS. Φ shares derive from `fact_bilateral_trade_annual`;
  weekly slice = `/52` (calendar, spec-062 FR-035); the injective crosswalk is a
  documented data-catalog mapping (dim_country ids), not a magic literal;
  exposure weights from the reference table. No fabricated coefficients.
- **III.7 determinism / frozen models** — PASS. Per-tick + auditor hashes
  byte-identical (R7). `ExternalNode`, `BoundaryFlowRegisterRow`, `TickContext`
  additions all frozen/typed. Sorted iteration (nodes, counties) for reproducible
  row order. No RNG introduced.
- **III.8 data-grounding** — PASS w/ DISCLOSURE. Φ attribution + trade value are
  grounded in `fact_hickel_erdi_annual` + `fact_bilateral_trade_annual` +
  `fact_county_exposure_by_external`. The bloc-granularity limitation and the
  Φ=0 nodes (india, latin_america — no distinct grounded bloc) are disclosed in
  spec.md D3, this plan, the ADR, and code comments; nothing fabricated.
- **III.10 earn-its-keep** — PASS. The conservation invariant is a running
  computation (per-bloc identity) with a gate, not a decorative construct.
- **II.9 dyadic morphism** — PASS. Every flow is one `source→dest` DRAIN_EDGE row.
- **II.12 authoring API / Amendment L** — PASS. No graph substrate change; no
  `networkx` import; systems untouched except the already-present wiring hook.
- **Amendment K (Lawverian)** — N/A (no contradiction-layer change).
- **R-PROOF** — one shared 101+102 proof window; written proof + baseline
  regenerated together (§ proof).
- **R-AMEND** — PASS. Blocs stay non-agentic Layer-0 register machinery; no
  constitutional amendment needed.

## Proof obligation (R-PROOF)

- WHAT changed: boundary flows now populate every tick (DRAIN_EDGE); external
  nodes carry attributed Φ + bilateral trade USD.
- WHY correct: the phi-distribution math + register flush were always built,
  merely unwired; DRAIN records mutate no hex/entity state (R7) so liveness +
  total_v are provably unchanged; conservation Σ DRAIN ≡ Φ_week per bloc holds by
  construction (weights sum to 1.0).
- MAGNITUDE: compared gate fields (counties_alive, counties_with_population,
  total_v) UNCHANGED; new outputs = `external_node_flows` summary section + N
  DRAIN_EDGE rows/tick (Michigan: 83 counties × 6 Φ>0 blocs) + per-bloc
  conservation rows (all OK severity). Determinism hashes byte-identical.

## Test strategy (TDD)

1. RED integration `test_trade_circuit.py` (`@pytest.mark.red_phase`) — assert
   DRAIN_EDGE rows per tick for Φ>0 blocs + `Σ DRAIN == Φ_week` per bloc; observe
   RED (silent no-op) pre-wiring.
2. Unit tests for exposure loader + crosswalk/attribution (pure, no DB).
3. GREEN: wire; re-run RED→GREEN; drop the marker.
4. `mise run check` → `test_bridge_income_circuit.py` → `test_trade_circuit.py` →
   `qa:e2e-regression` (new baseline) → `sim:e2e-bg` liveness 83/83.
