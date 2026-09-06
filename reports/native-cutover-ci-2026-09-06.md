# Native cutover and CI closeout

The implementation is recoverable on `codex/PER-304-fast-dev`. Checkpoint
`f754af42341b7e3206363264eac0d2cd9fe2ea06` contains the main cutover; the
closeout commit containing this report completes the final persistence,
policy-test and Copilot changes. This is implementation and operator evidence,
not release approval or a completed Director comprehension session.

## Current implementation

- Rust owns current mechanics and persistence. Obsolete Python engine, browser,
  optimization code and coupled tests are retired. Current source has 43 Python
  files and 1,682 Python test definitions, compared with 820 source files and
  14,790 test definitions before retirement. Current data and operator consumers
  remain, together with necessary language-neutral behavioral evidence.
- One tick is 28 days, with one transition and one commit. New campaigns load
  typed values from `content/scenarios/michigan/defines.toml`; Open reconstructs
  saved values. Units, finite horizons, integer bounds and feasible opening
  plans are checked. Fixed topology is separate from editable numeric values.
- Existing development saves are disposable. Old-session county backfills,
  foundation adoption, Archive upgrades and obsolete recovery probes are
  removed. Current-format save/reopen, transaction locks, marker-last writes,
  failed-write rollback and Archive revision constraints remain.
- Bevy exposes committed staffing history, held periods and saved-campaign
  comparison. Persisted delivery twins exercise current material outcomes.
- PostgreSQL uses bounded COPY writes, cached invariant reads, a reusable
  pristine test template and checked disposal. Durability and constraints
  remain enabled. Tuning used PostgreSQL 17 documentation through Context7;
  the tested server is PostgreSQL 17.11.
- Dev Rust selection builds 104 test binaries instead of 209. Full release
  validation retains the complete suite. Clippy, doctests, BSL sentinels and
  security checks remain. Cache publication is limited to trusted branches;
  pull requests restore those caches. Hosted timing still needs measurement.

## Imported data

The canonical `babylon-data` checkout is clean at
`08aa54890891c2e442badf6e3e05dc6a0a0e526a`. Kimi's data was reviewed and
incorporated into the 5,170,819,072-byte reference database. The manifest's
73 files and seven Parquet outputs were checked. QWI covers 83 Michigan
counties through 2021 Q4; it is not a 2024 wage source. National ASM county
allocation is Derived. Transborder revisions and HS2/SCTG differences remain
explicit. These reference sources are not silently substituted for current
QCEW wages or Designed staffing. The canonical data checks passed: 66 tests.

## Completed checks and timing limits

| Check | Evidence |
| --- | --- |
| Fresh dev PostgreSQL smoke | 144 s including bootstrap, 15 four-week ticks, restart and verified cleanup |
| Final focused live PostgreSQL | Eight current persistence, county, Archive and catalogue checks passed; 215 s including setup and verified cleanup |
| Reduced Rust test run | 2,635 passed, 22 skipped, 104 binaries; 306.461 s reporter duration including compilation; 69.884 s test execution |
| Final persistence retirement | Strict all-target Clippy passed; 213 unit tests passed, 19 ignored; execution 10.28 s; native binaries rebuilt |
| Retained Python unit CI | 2,127 passed, eight unavailable-data/dependency skips, 30.43 s |
| Release Python remainder | 42 passed, five missing-report skips; empty slow-unit shard accepted by its explicit empty-shard handling |
| Required-check policy contracts | 184 passed, including identity, exact-head, latest-run, race and rollback guards |
| Static checks | Static gate and BSL sentinels passed; normal commit hooks validate the final changed files |

The reduced Rust run preceded the final old-session deletion; the final
persistence Clippy/unit/live checks cover that deletion. Its 5m06s test-run
measurement excludes subsequent doctest and sentinel legs, so it does not
establish a five-minute complete CI job. The 144-second smoke and 215-second
focused PostgreSQL run are distinct workloads, not full-matrix timings. An
earlier full runtime PostgreSQL run took 536 seconds. Hosted cold-cache and
complete release durations are not established by these local results.

The second power outage interrupted an earlier Rust build. That interrupted
run is not counted as passing. After restart the actual host data mount was
writable, changes survived, and the canonical reference database retained its
size and modification time. The recovered run above completed successfully.

## Native save/reopen and presentation

The final native check used an isolated database, current rebuilt binaries,
and a task-owned TOML with an eight-period horizon, 32-hour workweek, 24 opening
sheet batches and 12 opening milling batches. An earlier incoherent fixture
kept opening plans requiring a 40-hour week; New refused it with
`DefinesInvalid` before the corrected campaign was created.

Campaign `67d06d8c-9911-4a12-971a-c3742b822b2f` advanced from period 0 to 1 and
then 2, producing exactly two durable tick markers. Its history view displayed
period 1 while the durable tail stayed at 2. Visible receipts showed 24 sheet
batches/240 kg and 12 milling batches/60 kg, matching the authored fixture.
The production surface was inspected at actual 1366x768 and 1920x1080 window
sizes. Stable production samples were approximately 60 FPS; this is local
operator evidence, not a hardware-independent performance guarantee. The
original 1366x768 display mode was restored.

After a normal quit, the TOML was renamed out of the supplied path. Open
reached Ready at saved period 2 with the file absent, then successfully
advanced to period 3. Read-only SQL confirmed unchanged saved parameter bytes,
material foundation identity, and both earlier commit hashes. The additional
period explains why the final commit list has three rows. Both launcher runs
exited normally with code zero.

Detailed local evidence, binary hashes, screenshots and before/after SQL
receipts are retained in
`/media/user/data/worktrees/b90d/babylon/.claude/per327-four-week-lno0ck3g/`.
The evidence does not establish Director comprehension or final G4 acceptance.

## Copilot and policy deployment

Copilot's repeated suggestion to add a `code-review` skill is a standard footer
after completed reviews, not a setup failure. Examples include
[PR 911](https://github.com/percy-raskova/babylon/pull/911#pullrequestreview-5124945424)
and [PR 910](https://github.com/percy-raskova/babylon/pull/910#pullrequestreview-5124452134).
The current instructions now focus on Rust authority, saved TOML parameters,
28-day ticks, material causes, current consumers and concrete defects.
Copilot setup uses the repository's pinned bootstrap. No duplicate review
skill was added, and this change does not promise to remove GitHub's footer.
The primary checkout receives these changes through the normal merge.

The authorized local policy requires strict `CI Gate` success for dev and
retains full qualification for main. CodeQL still runs on dev and main. Live
GitHub rulesets have not changed: the policy tool requires evidence from the
qualified exact `origin/dev` before activation. Merge and qualification must
therefore precede the normal policy apply; bypassing that check would strand
the repository on checks that are not deployed yet.

The old Infrastructure workflow, `infra-live.yml` (workflow ID 311311585), was
already deleted on dev and has GitHub state `deleted`. No additional active
Infrastructure workflow needed disabling.

`release.yml` currently publishes release notes, not a complete native-session
distribution. No main merge, tag, release publication or final issue closure
has been performed.
