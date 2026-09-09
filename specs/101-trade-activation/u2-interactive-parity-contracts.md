# P26 U2 — Interactive Parity: Seam Contracts

**Pinned before any code** (contract-pin-first, the M2/M3-proven pattern). Program:
`project/programs/26-international-trade.md` §4 U2; charter ADR160. Unit ADR: ADR161
(authored with the implementation). Lane: `101-trade-activation`.

**Thesis.** Spec-101's trade activation is live only on the headless batch path. This unit
gives the *playable* game the same wiring — via ADR109 typed W-C dataflow motions, each
closing (or updating) its sentinel row — without touching the headless path, the tick
pipeline, or any P25/M3-contested file. `qa:regression` stays byte-identical by
construction (it never runs `create_wayne_county_scenario` and the headless runner is
untouched).

## Audit facts this design stands on (verified 2026-07-27, tree 756b29d9)

- `ImperialRentSystem._invoke_phi_distribution_if_wired` (`engine/systems/economic.py:88`)
  no-ops unless `context` carries `session_id`, `boundary_flow_register`,
  `external_nodes_phi`, `county_exposure_by_external`. Only production writer:
  `headless_runner/runner.py:440-447`.
- `_invoke_vol2_circulation_if_wired` (`economic.py:158`) additionally needs `vol2_step`
  and `simulated_year`. **Zero production writers anywhere** — including the headless
  path (`simulated_year` appears only in the trace emitter). Seam row
  `vol2_circulation_vol2_step` (F-2) in `sentinels/seam_algebra/registry.py:451` has
  empty `supplier_files` plus a held-open exemption; the row itself prescribes the W-C
  motion.
- `services.boundary_register` exists on `ServiceContainer` (`engine/services.py:269`,
  default `None`); the runner constructs `BoundaryFlowRegister(session_id=...)` and
  assigns it (`runner.py:1248,1330`).
- `GameSession.advance_tick` (`game/session.py:922`) builds
  `TickContext(tick=next_tick, persistent_data={"player_actions": ...})` — no trade keys.
  `create_new_campaign` calls `ServiceContainer.create(config, defines)` with **zero
  overrides** → gamma/melt/Leontief/Vol I/II/III all silently degraded interactive-side.
- `create_wayne_county_scenario` (`engine/scenarios/_legacy_wayne.py`) seeds 4 classes
  (C001–C004), EXPLOITATION/WAGES/SOLIDARITY/TENANCY — **no TRIBUTE**, so
  `_process_tribute_phase` walks nothing in every interactive campaign.
- The canonical TRIBUTE shape already exists in `engine/scenarios/_legacy.py`
  (imperial-circuit): periphery proletariat →EXPLOITATION→ comprador →TRIBUTE→ core
  bourgeoisie, plus CLIENT_STATE core→comprador. Reusing that shape is applying the
  canonical theory pattern, not improvising a new one (IX.5-safe).
- Production Φ/exposure sources exist and are reused, not duplicated:
  `persistence/postgres_initialization.initialize_session` (+ `_attribute_phi_and_trade`,
  external-node bootstrap), `domain/economics/county_exposure.py` (reads spec-100's
  `fact_county_exposure_by_external`), `domain/economics/phi_distribution.
  distribute_phi_week_to_counties` (pure; register-buffered DRAIN_EDGE rows).
- Sole production caller of `create_new_campaign`: `cli/play.py` (low M3-collision;
  `tui/app.py` merely types against the handle). The raster-M3 lane owns `rust/` +
  `tui/` tutorial files — this unit does not touch them.

## Contract 1 — `TradeWiring` (new module `src/babylon/game/trade.py`)

Frozen dataclass (it carries live service objects — the `ServiceContainer` precedent —
not serializable game state, hence not a Pydantic game model):

```python
@dataclass(frozen=True)
class TradeWiring:
    boundary_register: BoundaryFlowRegister        # per-session buffer, runner-twin
    external_nodes_phi: Mapping[str, float]        # {node_id: phi_year_inflow_usd}
    county_exposure_by_external: Mapping[str, Mapping[str, float]]  # weights sum to 1/node
    start_year: int                                # simulated_year derivation anchor
    weeks_per_year: int                            # from defines.timescale, no literal 52
    vol2_step: Vol2CirculationStep | None = None   # None = vol2 sub-stage stays gated
```

`simulated_year(tick) -> int` method: `start_year + tick // weeks_per_year`.

Production builder `build_interactive_trade_wiring(*, session_id, runtime, defines,
sqlite_path, start_year, counties)`: thin composition of `initialize_session` + the
tick-0 external-node Φ query + the county-exposure reader. Raises
`TradeDataUnavailableError` (loud, typed) when the reference DB is absent — **never** a
silent skip. Unit tests never call the builder against real data; they construct
`TradeWiring` directly with deterministic fakes.

## Contract 2 — session seams (W-C motion #1: the four Φ keys)

- `GameSession.__init__(..., trade: TradeWiring | None = None)`;
  `create_new_campaign(..., trade: TradeWiring | None = None,
  economics_overrides: Mapping[str, Any] | None = None)`.
- `create_new_campaign` threads `economics_overrides` as
  `ServiceContainer.create(config=..., defines=..., **(economics_overrides or {}))`, and
  when `trade` is given also assigns `services.boundary_register =
  trade.boundary_register` (the runner-twin assignment).
- `advance_tick`, when `self._trade is not None`, stamps into the context before
  `run_tick`: `session_id`, `boundary_flow_register`, `external_nodes_phi`,
  `county_exposure_by_external`, and `simulated_year` (int; derivation above), and
  `vol2_step` when `trade.vol2_step is not None` (W-C motion #2).
- `trade=None` (every existing test, and degraded environments) is the byte-identical
  pre-U2 path: no context key is stamped, both sub-stages stay gated exactly as today.

## Contract 3 — Wayne imperial circuit (TRIBUTE seeding)

`create_wayne_county_scenario(include_imperial_circuit: bool = False)` — additive kwarg;
the default build stays **byte-identical** (SC-007 byte-equality pin and the M3 lane's
frame snapshots are built on the default). When True, adds the canonical circuit mirrored
from `_legacy.py`'s imperial-circuit values (cited, not invented): C005 periphery
proletariat (abroad), C006 comprador bourgeoisie (abroad), edges
C005→EXPLOITATION→C006, C006→TRIBUTE→C003 (Wayne core bourgeoisie),
C003→CLIENT_STATE→C006. `WayneCountyScenario.build(**kwargs)` already forwards kwargs.
The lobby default flips to `include_imperial_circuit=True` **only** in `cli/play.py`
(composition root), so harness/tests building the scenario directly are untouched.

## Contract 4 — Leontief/economics overrides (ruling-or-wiring: WIRED, disclosed-degraded)

`cli/play.py` composition root: when the reference DB is present, build the full
overrides via the headless runner's `_build_economics_overrides` (documented
DELIBERATE-TWIN import — a third copy is worse than a cross-layer import the CLI already
sits above) and `build_interactive_trade_wiring`; when absent, emit ONE loud warning
naming every degraded service (the `gamma_calculator unwired` precedent, extended) and
proceed without trade wiring. Loud degradation is the estate's sanctioned pattern; silent
stubs remain forbidden.

## Contract 5 — sentinel row closure

`sentinels/seam_algebra/registry.py`: `vol2_circulation_vol2_step.supplier_files` gains
`("src/babylon/game/session.py",)` and the held-open `SentinelExemption` row is removed
(the gate now has a genuine production supplier). The Φ-distribution row needs no change
(it was never flagged; the session becomes its second legitimate writer).

## Gates & drift policy

- `mise run check` green; scoped `test:q` per unit of work.
- `qa:regression` byte-identical (headless path untouched; verified pre/post).
- `qa:vault-regression-ci`: cannot drift from this unit on CI/this box (trade wiring
  requires the reference DB, absent here — the loud-degraded path is byte-identical);
  first dev-box run with the drive mounted MAY drift interactive vault goldens → that is
  a **declared §6.5 ceremony**, disclosed in the PR body, not a surprise.
- Cross-lane: M3's parity harness builds Wayne via default args → unaffected. Flagged in
  the PR body anyway (merge-order coordination note).

## Environment blockers recorded (not this unit's scope)

The babylon-data LUKS volume is locked/absent on this box (`data:doctor` skip;
`data/sqlite` dangling): real-data verification of Contract 4's full-override path and
all of U3 (FAF freight) wait on the Director unlocking the drive
(`sudo tools/heal_data_mount.sh`).
