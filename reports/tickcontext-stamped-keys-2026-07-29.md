# TickContext Stamped-Key Census (Program 27 Phase 0, Task 6)

**Date:** 2026-07-29
**Scanner:** `tools/tickcontext_key_census.py` (AST-based; regex misses
annotated dicts — this scans `src/babylon` for `ast.Assign` nodes whose
target is a subscript `context[<str-literal>]` or attribute `context.<attr>`
on a name bound as `context`/`ctx`).
**Guard:** `tests/unit/engine/test_tickcontext_key_census.py` —
`DECLARED_STAMPED_KEYS` frozenset; a new stamped key or a removed one both
fail the test (drift in either direction is loud).

## Context

`TickContext` (`src/babylon/kernel/*` / `engine/simulation_engine.py`) is a
`extra="allow"` Pydantic-adjacent container with three declared fields
(`tick`, `persistent_data`, `displacement_mode`). Beyond those, callers stamp
ad hoc dict keys onto the same object as informal side channels between the
runner/session layer and mid-engine systems. These ad hoc keys are
**undeclared `run_tick` parameters** — Program 27 §6.5 requires them
enumerated exhaustively before the Rust port, because Rust's `TickContext`
struct (Phase 3) needs a typed field for each one; there is no `extra="allow"`
equivalent to fall back on in Rust.

The census below is the exhaustive Phase-0 result: **7 stamped keys**, all
originating from the Program 26 international-trade wiring (spec-101 /
Vol II Circulation). No other ad hoc keys exist anywhere in `src/babylon`.

## Census table

| Key | Writer site(s) | Reader site(s) | Value type | Proposed Rust field type |
|---|---|---|---|---|
| `session_id` | `src/babylon/engine/headless_runner/runner.py:444`, `src/babylon/game/session.py:1445` | `src/babylon/engine/systems/economic.py:118` (`_invoke_phi_distribution_if_wired`), `:177` (`_invoke_vol2_circulation_if_wired`) | `uuid.UUID` | `uuid::Uuid` |
| `boundary_flow_register` | `runner.py:445`, `session.py:1446` | `economic.py:117`, `:176` | `BoundaryFlowRegister` (`src/babylon/domain/economics/boundary_flow_register.py:47`) — per-tick mutable buffer of `BoundaryFlowRegisterRow` | `Rc<RefCell<BoundaryFlowRegister>>` (or an `&mut` threaded explicitly instead of context-stashed, per Rust's no-shared-mutable-ambient-state norm) |
| `external_nodes_phi` | `runner.py:446`, `session.py:1447` | `economic.py:119` | `dict[str, float]` — `{external_node_id: phi_year_inflow_usd}` | `HashMap<String, f64>` |
| `county_exposure_by_external` | `runner.py:447`, `session.py:1448` | `economic.py:120` | `dict[str, dict[str, float]]` — `{external_node_id: {county_fips: weight}}` | `HashMap<String, HashMap<String, f64>>` |
| `simulated_year` | `session.py:1449` (via `self._trade.simulated_year(next_tick)`) | `economic.py:178` | `int` (calendar year) | `i32` |
| `vol2_step` | `session.py:1451` (conditional: `if self._trade.vol2_step is not None`) | `economic.py:175` | `Vol2CirculationStep` (`src/babylon/engine/systems/vol2_circulation.py:112`) — holds the LODES OD loader + hex↔county `ScaleAdjunction`, exposes `.step(...)` | Not a plain-data field — a constructed sub-stage service. Rust equivalent: an `Option<Vol2CirculationStep>` field holding the loader handle + adjunction, matching the Python "presence gates the sub-stage" pattern (FR-009/010/011/030a). |
| `vol2_circulation_result` | `src/babylon/engine/systems/economic.py:199` (written back into context after `vol2_step.step(...)` runs) | `src/babylon/persistence/conservation_audit.py:356` (`ConservationAuditor`'s `circulation_preserves_sum_v` invariant check) | `CirculationStepResult` (`vol2_circulation.py`, dataclass: `conservation_residual: float`, `wall_time_ms: float`, plus OD-year-used and per-hex deltas) | `CirculationStepResult { conservation_residual: f64, wall_time_ms: f64, od_year_used: i32, .. }` — a plain struct, this one IS data (an audit record), not a service handle |

## Notes for the Rust `TickContext` port (Phase 3 consumer)

1. **Six of seven keys are Program 26 trade-wiring only** — they are all
   `None`/absent on every non-trade tick (`self._trade is None` in
   `session.py`, or the runner's `session_id is None` guard). The Rust
   struct should model this as `Option<T>` fields, not required ones — the
   silent-no-op-when-absent behavior (`economic.py:121-125`, `:179`) is a
   load-bearing back-compat contract per spec-101/spec-062 FR-009.
2. **`vol2_step` and `vol2_circulation_result` are asymmetric**: `vol2_step`
   is an input service handle (constructed once outside the tick loop,
   stamped in if non-`None`); `vol2_circulation_result` is an output record
   (written by the system that consumes `vol2_step`, read downstream by the
   conservation auditor same-tick). Phase 3 should NOT collapse these into
   one field — they have different lifetimes and different writers/readers.
3. **`boundary_flow_register` is the only genuinely mutable shared buffer**
   among the seven — it accumulates `BoundaryFlowRegisterRow`s across the
   phi-distribution and vol2-circulation sub-stages within the same tick,
   then is flushed by the persistence layer. In Rust this is the one key
   that should NOT be a context field at all if an explicit `&mut` threading
   pattern is available by Phase 3 — stashing a mutable-buffer handle in a
   context struct is the Python-only workaround for lacking real parameter
   threading through `run_tick`'s fixed system-list signature.
4. **No new keys appeared in the scan beyond the plan's known five**
   (`session_id`, `boundary_flow_register`, `external_nodes_phi`,
   `county_exposure_by_external`, `vol2_step`) — the scanner additionally
   found `simulated_year` and `vol2_circulation_result`, both real,
   confirmed-legitimate stamped keys on the same trade-wiring context object
   that the plan's five-key expectation had not enumerated. All seven are
   captured in `DECLARED_STAMPED_KEYS`.
5. **Scan scope:** `tools/tickcontext_key_census.py` roots at `src/babylon`
   only (excludes `tests/`, `tools/`, `rust/`) by design — the census is a
   contract on production `run_tick` call sites, not test fixtures.
