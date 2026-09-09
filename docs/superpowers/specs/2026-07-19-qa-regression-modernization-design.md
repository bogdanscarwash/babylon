# qa:regression Modernization — Design

**Status:** EXECUTED (ADR090, 2026-07-20).
**Execution:** POST-MERGE program. Runs on a fresh branch off `dev` AFTER the
`refactor/vol3-money-scissors` merge — never mid-program, because changing the
measuring instrument while an approval gate is in flight churns its evidence.

## 1. Why this program exists (the evidence, all from 2026-07-19)

The byte-identical gate is the repo's falsifiability contract (Constitution
III.7). The Vol III program proved the contract is fine and the **coverage is
not**:

1. **The gate cannot see the county layer.** The U9 endogenous interest rate
   shipped structurally inert (`i ≡ 0.0` on all real data, a regression vs the
   FRED path). Every per-task review passed; `qa:regression` stayed green in
   both the broken and fixed states because the five canonical scenarios carry
   no `county_fips` (`tools/regression_test.py:159`) — and the entire Vol III
   financial layer lives at county level. "Byte-identical" means "nothing the
   scenarios exercise changed," and what they exercise is undeclared.
   This is the gate-blindness error class (`sentinels/coverage/`, U7.9)
   applied to the gate itself.
2. **The e2e summary metric does not span the financial layer.**
   `qa:e2e-regression` (detroit-tri-county) reported Δ=0.000% while county
   `interest_payments` went 0 → 1.179e9, because its summary pins
   `total_v`/`population`/`counties_alive` — none downstream of interest
   (verified by the 2026-07-19 adversarial pass; the consumption chain is
   `market_scissors.py:567 → _national_serviceability → serviceable
   divergence`, which the 5-tick summary never serializes).
3. **The `defines_hash` guard is empirically toothless.** All five stored
   baselines carry stale, mutually-different `defines_hash` values, yet the
   gate passes — the mismatch is emitted as `WARNING:` (`regression_test.py:
   919-921`) and excluded from the pass filter (`:944`). A guard that never
   gates is a false liveness claim by the tool about itself.
4. **Failure output is unattributed.** When bytes move, the tool prints raw
   diffs; U8.2/U8.3 spent an entire task hand-deriving *which* system, tick,
   and column first diverged. Attribution is mechanical and should be the
   tool's job.
5. **Determinism proof lives outside the gate.** The two-independent-process
   byte-comparison (U7.0, `tests/unit/tools/
   test_regression_construction_cadence_determinism.py`, ADR056 precedent)
   runs as a unit test, not as a leg of the gate it protects.

## 2. Principles

- **Byte-identity stays.** III.7 is untouched; determinism is the contract.
  This program modernizes *coverage* and *observability*, not strictness.
- **The gate must declare what it watches, and a sentinel must prove the
  declaration is complete** — the same declared-invariant pattern as the six
  sentinel classes (ADR088). A gate whose estate is implicit rots silently.
- **A baseline that captures a channel is an inertness detector for that
  channel.** All-zeros is a signal, never a default.
- **CI never touches the babylon-data drive** (owner ruling 2026-07-14): every
  new scenario's inputs ship as committed deterministic artifacts (the D4 /
  `ci-data-v6` pattern).

## 3. Design elements

### E1. Declared coverage, sentinel-enforced

Each canonical scenario declares a `ScenarioCoverage` (frozen Pydantic model):
the systems (of the 30 in `_DEFAULT_SYSTEMS`) and layers (financial, market,
sovereignty, …) its run demonstrably exercises. A new check in
`sentinels/coverage/` proves (a) each declaration is TRUE (the declared system
observably fires in that scenario — e.g. via the SessionRecorder trace), and
(b) the UNION covers every system and every declared layer. An uncovered
system is a loud sentinel failure naming the hole. Declarations live beside
the scenario definitions in `tools/regression_test.py`'s successor module —
data, not comments.

### E2. County-bearing canonical scenarios

Promote into the byte-identical tier:
- `single_county` — a Wayne-seeded minimal county scenario (committed
  artifact, not a drive read): the smallest graph where the Vol III financial
  layer, MELT path, and distribution identity all fire.
- `detroit_tri_county` — the existing e2e scenario, promoted from Δ%-summary
  to full byte-identical dense-trace membership.

The **nationwide** canonical scenario (Amendment R/S: canonical test scale =
NATIONWIDE; task #49) remains OWNER-GATED: this spec reserves the slot
(`nationwide` in the estate declaration marked `deferred: owner-tabled
2026-07-15`) but does not implement it. Un-tabling is the owner's call; if
un-tabled, it lands as this program's final phase with its own runtime budget.

### E3. Financial dense-trace channels + the no-dead-columns rule

Dense traces gain per-tick channels for the financial layer: the endogenous
rate, `profit_rate_ceiling`, `s_r`, tightness `τ`, county `interest_payments`,
the distribution components (`p`, `i`, `r_rent`, `t`), and the serviceability
inputs. Rule: **a channel that is all-zeros/all-absent across an entire
scenario run FAILS the gate unless the scenario's coverage declaration marks
that channel `at_rest` with a reason** (the county-free scenarios legitimately
declare the financial channels at rest; `single_county` may not). U9's
inertness becomes unmissable: the `i` column would have been visibly dead in
every baseline.

### E4. First-divergence attribution

On FAIL, the tool reports — machine-readably and human-readably — the first
`(tick, system, channel, county)` where the trace departs the baseline, plus
the magnitude and the last-agreeing value. Implementation: ordered walk of the
dense trace (the data is already per-tick; no bisection re-runs needed).
Output feeds delta reports directly, replacing the U8.2/U8.3 hand-derivation.

### E5. Gate-integrated determinism + a real `defines_hash` leg

- Fold the U7.0 two-process byte-comparison into `qa:regression` itself as a
  per-run leg (one scenario suffices for the cadence proof; keep the unit test
  as the fast-tier mirror).
- Promote `defines_hash` from advisory WARNING to a gating leg: a mismatch
  FAILS with the message naming the regeneration ceremony
  (`generate_all_baselines` + the owner-gated approval flow). The five stale
  stored hashes get regenerated once, in this program's own ceremony commit.

## 4. Non-goals

- No change to Constitution III.7 or the byte-identity contract.
- No baseline regeneration on the vol3 branch (that belongs to vol3's U8.5).
- No mutation testing in CI (owner ruling 2026-07-16: local-only).
- No nationwide scenario implementation without the owner un-tabling #49.
- The employment-100k placeholder and the Wayne `capital_stock=0` calculator
  gap are DISCLOSED inputs, not this program's scope (Program 17 honesty-gap
  lineage) — though E3's channels will make both visibly at-rest.

## 5. Risks

- **One-time baseline churn**: E2/E3 mint new baselines and widen dense
  traces — a single regeneration ceremony with owner approval, after which
  the gate is stricter forever. Mitigation: land E1 (declarations) and E4
  (attribution) first; they are baseline-neutral.
- **Runtime budget**: county scenarios + dense channels grow gate wall-clock.
  Budget: the full gate stays under the two-tier CI envelope (Program 15);
  `single_county` capped at a tick count that keeps qa:regression < 5 min
  local. Measured, not assumed, in the plan.
- **Committed-artifact size**: county inputs as deterministic artifacts (D4
  pattern); LFS-pointer traps documented in test-estate memory apply.
- **Coverage-declaration truthfulness**: E1's check (a) exists precisely so a
  declaration cannot rot into a false claim — the sentinel family's mutation
  rule applies (each new check ships with a proof it catches an injected
  violation).

## 6. Execution shape (for the plan phase)

Sequenced so every phase is independently landable, baseline-neutral phases
first: E1 → E4 → E5(hash leg) → E2+E3+E5(two-process leg) as the single
baseline-minting ceremony. Standard flow: this spec → owner review →
`writing-plans` → subagent-driven execution on `refactor/qa-regression-modernization`
off post-merge `dev`. Every phase ships its sentinel (standing owner rule:
sentinel every error class, mutation-validated).
