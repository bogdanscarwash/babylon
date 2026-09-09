# Spec 060 — Invariant Test Helpers

Helpers shared by the spec-060 value-form, software-metamorphic, and
Marxist sign/monotonicity invariant tests. See
[`specs/060-value-form-invariants/`](../../../specs/060-value-form-invariants/)
for the full spec, plan, contracts, and data model.

## Helpers in this package

| Module                    | Purpose                                                                                                | FR            | Contract                                 |
| ------------------------- | ------------------------------------------------------------------------------------------------------ | ------------- | ---------------------------------------- |
| `monetary_rescaling.py`   | `rescale_currency_fields(world, k)` — scales Currency-typed fields by k                                | FR-001/FR-002 | `contracts/monetary_rescaling.md`        |
| `uuid_relabeler.py`       | `relabel_uuids(world, alias_fn=None)` — bijective ID relabeling                                        | FR-013        | `contracts/uuid_relabeler.md`            |
| `transformation_mode.py`  | `probe_transformation_mode`, `skip_unless_active` — single source of truth for the redistribution gate | FR-008/FR-021 | `contracts/transformation_mode_probe.md` |
| `serialization.py`        | `roundtrip_via_json(world)` — `model_dump_json` round-trip                                             | FR-014        | —                                        |
| `h3_round_trip.py`        | `rollup_then_disaggregate(...)` — H3 roll-up + uniform disaggregate                                    | FR-016        | —                                        |
| `productivity_shock.py`   | `halve_snlt_in_sector(world, sector_id)` — productivity-shock perturbation                             | FR-007        | —                                        |
| `proportional_scaling.py` | `scale_c_v_preserving_s_over_v(world, k)` — scales c+v at constant s/v                                 | FR-017        | —                                        |
| `variance_trace.py`       | `ProfitRateVarianceTrace` — variance-over-time collector                                               | FR-019        | —                                        |
| `melt_consistency.py`     | `ConsistencyReport`, `EntityViolation` — per-entity MELT diagnostics                                   | FR-003        | —                                        |
| `metamorphic.py`          | `MetamorphicPair` — optional convenience wrapper                                                       | —             | —                                        |

The production-side companion is `src/babylon/config/h3_splitter.py`
(`H3SplitterRule`, `split_uniformly`) — the one production change permitted
by the FR-011 exception, codifying the de facto uniform-splitting
convention used across the codebase.

## Where the tests live

| Location                                                            | Purpose                     |
| ------------------------------------------------------------------- | --------------------------- |
| `tests/property/invariants/test_numeraire_invariance.py`            | FR-001/FR-002               |
| `tests/property/invariants/test_aggregate_equalities_property.py`   | FR-005 property arm         |
| `tests/property/invariants/test_proportional_scaling.py`            | FR-017                      |
| `tests/integration/economics/test_melt_consistency.py`              | FR-003/FR-004               |
| `tests/integration/economics/test_aggregate_equalities.py`          | FR-005                      |
| `tests/integration/economics/test_wage_occ_asymmetry.py`            | FR-006                      |
| `tests/integration/economics/test_productivity_shock_decoupling.py` | FR-007                      |
| `tests/integration/economics/test_uuid_relabel_invariance.py`       | FR-013                      |
| `tests/integration/economics/test_serialization_roundtrip.py`       | FR-014                      |
| `tests/integration/economics/test_markovian_step.py`                | FR-015                      |
| `tests/integration/economics/test_h3_round_trip.py`                 | FR-016                      |
| `tests/integration/economics/test_occ_monotonicity.py`              | FR-018                      |
| `tests/integration/economics/test_volume_iii_equalization.py`       | FR-019                      |
| `tests/unit/test_transformation_mode_probe.py`                      | FR-008/FR-021 gate behavior |

Run the whole bundle: `uv run pytest -m invariant`.

## Deliberate-bug recipes

The bundle's value depends on its sensitivity. Each helper supports a
specific bug-injection recipe from `quickstart.md`:

1. **Numeraire** — Add `1.0` to a profit-rate computation in
   `src/babylon/domain/economics/derived_metrics.py` → the numeraire test
   fails because ratios now depend on money scale.
1. **MELT consistency** — Multiply τ by `0.9` in
   `DefaultMELTCalculator.get_melt` → the per-entity test fails with
   ~10% relative error (once Feature 026 lands productive entities).
1. **UUID relabel** — Replace a `sorted(...)` reduction with raw dict
   iteration; the relabeler shows ~bit-equivalence breaks.
1. **H3 round-trip** — Make `split_uniformly` return
   `[parent_value / (n_children - 1)] * n_children` → parent-
   conservation check fails immediately.

## Today's skip footprint

8 SKIPs at landing (all spec-060 attributable):

| #   | Source                    | Gate                                                      |
| --- | ------------------------- | --------------------------------------------------------- |
| 1   | FR-004                    | NoDataSentinel by-design                                  |
| 2   | FR-003                    | two_node has no productive entities (Feature 026 unblock) |
| 3   | FR-005 proportional arm   | same                                                      |
| 4   | FR-005 redistribution arm | transformation inactive                                   |
| 5   | FR-006                    | transformation inactive                                   |
| 6   | FR-006 property           | transformation inactive                                   |
| 7   | FR-007                    | transformation inactive                                   |
| 8   | FR-019                    | transformation inactive                                   |

Spec-060 SC-007 (≤ 5 SKIPs target) becomes achievable as Feature 026
hydrates productive entities (collapses 2 SKIPs to 0 or 1) and as the
transformation engine activates (collapses 5 SKIPs to 0).
