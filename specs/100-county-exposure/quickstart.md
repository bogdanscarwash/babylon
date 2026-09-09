# Quickstart — spec-100 County-Exposure Loader

## Run

```bash
# from the babylon worktree (mise sets PYTHONPATH=src so babylon + babylon_data
# resolve to the worktree, not the main repo)
mise run data:exposure -- --dry-run           # full pipeline + validation, no writes
mise run data:exposure -- --apply             # staged build + validated atomic swap
mise run data:exposure -- --apply --years 2024
mise run data:exposure -- --rollback-from-backup   # restore __pre_100 tables
mise run data:exposure -- --drop-backup            # remove __pre_100 backups
```

Exit codes: 0 success · 1 validation failure (canonical untouched) · 2 pre-flight/usage · 130 interrupted.

## Verify

```bash
# 1. per-(bloc, year) weights sum to 1.0
sqlite3 data/sqlite/marxist-data-3NF.sqlite \
  "SELECT time_id, external_country_id, ROUND(SUM(weight),9) FROM fact_county_exposure_by_external
   GROUP BY time_id, external_country_id;"    # every row → 1.0

# 2. determinism — run twice, table hash identical (printed in the audit JSON)
mise run data:exposure -- --dry-run --years 2024   # note table_hashes
mise run data:exposure -- --dry-run --years 2024   # same hashes

# 3. bilateral trade annual = exact monthly sum
sqlite3 data/sqlite/marxist-data-3NF.sqlite \
  "SELECT country_id, time_id, total_trade_usd_millions FROM fact_bilateral_trade_annual LIMIT 8;"
```

## Read the audit

`reports/ingest/exposure_<UTC>.{json,md}`:
- `per_year[].conservation.holds` — the weight-conservation invariant (Σraw vs
  Σcovered coeff, ±2%). An INTERNAL consistency invariant, NOT an external
  reconciliation (research R5). Computation correctness is guarded by the
  golden-value + input-perturbation regression tests.
- `per_year[].concordance_coverage.coverage_fraction` — the goods-bias metric (~0.15;
  the covered coefficient mass, documenting the map's tradeable-goods scope).
- `gates.sc005_trade_annual_exact_sum` — real check: persisted annual trade equals
  an independent `SUM(fact_trade_monthly)` (None on dry-run).
- `run_metadata.bloc_invariant` — `true` while the county distribution is the same
  across blocs (no bloc×industry resolution in the DB; intentional — research R6).
- `run_metadata.table_hashes` — the determinism hashes (order-independent, review #4).

## Tests

```bash
# babylon-data unit tests (fast, in-memory ORM fixtures)
cd /home/user/projects/game/babylon/worktrees/d100
PYTHONPATH=src uv run pytest src/babylon_data/../../  # see below
# or, targeted:
PYTHONPATH=src uv run pytest \
  ../../../babylon-data/tests/unit/data/exposure -q
# integration (Michigan tri-county against the real DB — IV gate)
PYTHONPATH=src uv run pytest \
  ../../../babylon-data/tests/integration/data/exposure -q
```
