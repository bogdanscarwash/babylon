---
name: specification-validate
description: Test a Babylon change with Constitution v4 and its ratified spec.
---

# Specification Test Phase

Use this phase after the `specification-build` phase. Read the ratified spec and
the changed `code` before you start.

Read `CONSTITUTION.md` v4.0.0 and `NORTH_STAR.md` first. Babylon is an
entertainment-first game, not a forecast. The Bevy client is an admin viewer.
It has no player action.

Do not test planned Gate 3 as if it exists.

## Laws

- Historical cases test causal signatures and counterfactual response.
- Historical cases also test varied effects and hysteresis.
- A historical scenario does not set the game result.
- Equal input bytes must produce equal output bytes and hashes.
- Determinism proves computational identity, not scientific truth.
- Mutation tests must find a severed causal link.
- A shock must not write its downstream result.
- Mark each value `Observed`, `Derived`, `Calibrated`, or `Designed`.
- A game display must answer one decision question.
- An admin display cannot pass a game test.

## Steps

1. Read the ratified spec and list its observable requirements.
2. Read the changed `code` and compare it with each contract.
3. Write one `PASS` or `FAIL` test for each contract.
4. Add boundary, property, replay, and mutation tests when they apply.
5. Run the smallest test set that can first find a fault.
6. Run the full required gates after the small tests pass.
7. Record each command, result, open fault, and blocker.

Use these test marks when they apply to the contract:

- `@pytest.mark.math` for pure formulas
- `@pytest.mark.ledger` for economic or political data
- `@pytest.mark.topology` for relation operations
- `@pytest.mark.integration` for storage and boundary tests
- `@pytest.mark.unit` for the default small tests
- `@pytest.mark.red_phase` for the intentional TDD red step

Use the repository tasks from `CLAUDE.md`. Run these tasks when they apply:

```bash
mise run test:unit
mise run test:int
mise run test:scenario
```

Correct each fault. Do not skip it or change a contract to hide it.

## Historical benchmark

For a game-rule change, run these scenarios:

- the no-shock control
- the declared shock envelope
- one strong or weak capacity scenario
- one policy counterfactual when a policy changes the path

Compare onset, causal signatures, varied effects, hysteresis, and outcome writes.
Classify a difference as source, calibration, defect, or designed liberty.

Calibrated agreement is not an observation or a prediction.

## Tick identity

Run the required determinism tasks from `CLAUDE.md`. Compare bytes and hashes
across two processes. Stop when equal inputs produce different results.

## Record

Write a checklist in `.specify/checklists`. Use `{YYYYMMDD}-{topic}.md` for its
name. Record these items:

- each contract result
- test results by contract type
- results from the historical benchmark and counterfactual
- results from the tick identity check
- open faults and blockers

If a behavioral contract has no test, load `specification-specify` again.
After all gates pass, use `specification-govern` for a compliance check.
