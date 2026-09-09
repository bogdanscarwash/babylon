# tests/baselines/

Persisted JSON snapshots used by `tools/regression_test.py` and the
`qa:regression` / `qa:e2e-regression` mise tasks to detect unintended
behavioral drift.

## Files

| File                       | Origin                                                                 | Regeneration command                    |
| -------------------------- | ---------------------------------------------------------------------- | --------------------------------------- |
| `imperial_circuit.json`    | Legacy in-memory engine, imperial-circuit scenario                     | `mise run qa:regression-generate`       |
| `two_node.json`            | Legacy in-memory engine, two-node scenario                             | `mise run qa:regression-generate`       |
| `starvation.json`          | Legacy in-memory engine, starvation scenario                           | `mise run qa:regression-generate`       |
| `glut.json`                | Legacy in-memory engine, glut scenario                                 | `mise run qa:regression-generate`       |
| `fascist_bifurcation.json` | Legacy in-memory engine                                                | `mise run qa:regression-generate`       |
| `single_county.json`       | ADR090: Wayne County (FIPS 26163) scenario, real MarxianHydrator-extracted fixture — the smallest graph where the Vol III financial layer fires | `mise run qa:regression-generate`       |
| `mutation_baseline.json`   | Mutation-testing baseline (mutmut)                                     | `mise run qa:mutation-baseline`         |
| **`michigan-e2e.json`**    | **Spec-064: headless Postgres runner**                                 | See below                               |
| **`dense/<scenario>.csv`** | **Program 13 item 2 / ADR090 E3: dense per-tick traces for the 6 scenarios above** | `mise run qa:regression-generate-dense` |
| **`dense/detroit_tri_county.csv`** | **ADR090 E2: the qa:e2e-regression bundle's dense trace — schema DIFFERS from the harness dense CSVs above** (29 bundle-native columns: `county_<fips>_*` × 3 counties + 4 `financial_*`; no entity/edge columns) | Copy `dense_trace.csv` from one canonical strict run — see below. **NEVER** `qa:regression-generate-dense` (wrong schema). |

## `dense/` — dense per-tick golden traces (Program 13 item 2)

The six `*.json` files above sample ~9 variables every 10th tick (~54
numbers for a 52-tick scenario) — a plausible-but-wrong engine could
reproduce those without reproducing the actual per-tick dynamics.
`dense/<scenario>.csv` pins **every tick** of the same run, plus every
entity's wealth/tension-relevant fields and every relationship's
value_flow/tension. Compared **byte-identically** (not tolerance-bounded)
by `qa:regression` whenever the file exists — a mismatch reports the exact
first divergent tick and column.

Column contract, float/bool formatting, and the topology-static assumption
are documented in `docs/reference/determinism-contract.rst` ("Dense Golden
Traces"). Regenerate with `mise run qa:regression-generate-dense` (this
also regenerates the sampled `*.json` baselines above, since both share one
simulation run per scenario — only the `generated_at` timestamp field in
the JSON will move if behavior is unchanged).

`dense/detroit_tri_county.csv` is the one exception: it is **not** produced
by `qa:regression-generate-dense` — its schema is the qa:e2e-regression
bundle's own dense-trace contract (bundle-native `county_<fips>_*` +
`financial_*` columns, no entity/edge columns), not the harness scenario
contract the other six files share. Regenerate from one canonical strict
headless-runner run and copy its `dense_trace.csv` verbatim:

```bash
BABYLON_TEST_PG_DSN='...' uv run python -m babylon.engine.headless_runner \
  --scope detroit-tri-county --ticks 5 --strict
cp "$ARTIFACT_DIR/dense_trace.csv" tests/baselines/dense/detroit_tri_county.csv
```

Run it twice independently and diff the two `dense_trace.csv` outputs before
committing (ADR090's ceremony did this — both matched md5
`a293f41f31299dca44cfa662b4b0eee2`) to prove determinism before minting.

## Regenerating michigan-e2e.json (spec-064)

This baseline is the `summary.json` produced by the canonical headless
runner invocation. The MVP baseline uses the Detroit tri-county scope
(faster than full Michigan; same artifact contract).

```bash
# 1. Run the headless runner with the canonical small scope
BABYLON_TEST_PG_DSN='...' uv run python -m babylon.engine.headless_runner \
  --scope detroit-tri-county --ticks 5 \
  --output-dir /tmp/spec064-baseline

# 2. Replace the committed baseline
cp /tmp/spec064-baseline/summary.json tests/baselines/michigan-e2e.json

# 3. Commit the regeneration in the same change as the engine math edit
git add tests/baselines/michigan-e2e.json
git commit -m "test(baseline): regenerate michigan-e2e after <intentional change>"
```

A future spec will scale this baseline up to the full
`michigan-canada` scope once the engine math is wired into the
headless tick loop and the wallclock budget is validated at the
83-county scale.
