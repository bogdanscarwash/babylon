# Flake evidence: tutorial-pilot screen-transition race under xdist load

**Filed 2026-07-22 ~12:50 EDT by the verification session (bg feb021be), for whichever
session next touches the pilot harness (the interface-refinement train owns that area now).**

**Symptom:** `tests/unit/tui/test_tutorial_pilot.py::TestEachStepOfTheWayneOpeningArc::
test_step_resolves_its_own_then[...economy dossier...Fundamental Theorem verdict...]`
(target_index=7) failed ONCE on dev tip `633de83f` during a full `mise run check`
(12:25 EDT run, concurrent with the interface-refinement train's own workload):

```
textual.css.query.NoMatches: No nodes match '#status' on Screen(id='_default')
```

i.e. `_replay_through` queried the campaign screen's `#status` label while the app's
screen stack was still on the default screen — a Pilot screen-push timing race, not a
state assertion failure.

**Evidence it is a load/timing flake, not an order-pollution or #254 regression:**
- Scoped rerun (`mise run test:q -- tests/unit/tui/test_tutorial_pilot.py::TestEachStepOfTheWayneOpeningArc`): 9/9 green, 19.7s.
- Second full `mise run test:unit` on the SAME tree (12:45 EDT, box quiet): 14,132 passed / 0 failed.
- No file touched by PR #254 is in the failure path (harness + ArchiveApp are T6/T4 code).
- test:unit runs pytest-randomly (order shuffles per run) + xdist `-n 4 --dist loadscope`;
  the failing step's durations run 2.0–2.7s — the slowest in the class — and the first
  failing run shared the box with another session's active workload.

**Suggested fix direction (owner of the harness decides):** condition-based waiting on the
screen push before querying (`await pilot.pause()` loops until `app.screen` is the campaign
screen, bounded), instead of fixed pauses — the Gauntlet ruling already names wall-clock
timing as determinism poison in tests.

Full failure record: the 12:25 run's `reports/test-results/unit/report.json` in the
`verify-north-star` worktree (deleted after verification; longrepr preserved here in
summary). Memory: [[player-interface-shell-design]] session notes.

---

## Addendum 2026-07-22 ~17:20 (interface-refinement train, CI run 29957598863)

Second occurrence, richer evidence — the PR #277 CI unit shard failed the
`learn_the_dashboard_pane` replay AT step `palette_to_the_repression_ledger`:

- **Primary (root):** `AssertionError: expected nav.current == 'social_class/C001',
  got 'organization/ORG002'` — while the dossier text local in the SAME frame already
  showed C001's full rendered statblock. The palette chain
  (dismiss-callback → `_navigate` → dossier re-render → nav/status update) spans
  several message-pump cycles; the completion assert ran mid-chain.
- **Secondary (the '#status' NoMatches both reports led with):** raised DURING
  unwinding — the still-in-flight navigation's status update racing app teardown.
  Noise, not the disease.

**Fix shipped (this branch):** `_settled` now drain-retries the WHOLE completion
check on `(AssertionError, NoMatches)`, bounded at `_MAX_SETTLE_RETRIES` pump
drains. The bound is what keeps the Then honest — settled-and-wrong still fails
with the identical verdict; only in-flight states are absorbed. No wall-clock.
